//! The macOS backend (rust-helper `macos-capture-and-tcc.md`): ScreenCaptureKit through the `screencapturekit` crate
//! (11.1.1), a display filter that includes the game's window alone, placed on the display it is on, `sourceRect` in
//! points at the window's top-left corner, 390 × 96 output pixels (one per screen pixel: the strip's band, and room for
//! a windowed game's title bar), BGRA, sRGB, at most 60 frames a second (10 on battery power, the laptop mode) and only
//! when the corner changes. The window is the main one of the client process the helper follows (`game::running`), so
//! nothing that covers its corner on the screen reaches the capture: not the menu bar (a display filter that excludes
//! windows shows it by default, and over a full-size game whenever another app is active), not this app's window or
//! another app's, not a second client. Each frame is decoded in the callback into a value and sent on; the stream lives
//! on its own thread, starts again when the window moves or is another, its display changes mode, and after the system
//! stopped it while the grant holds (`Retry`); without the grant the capture ends until the helper restarts. Each tick
//! looks at the client's window in the window server's list (`Look`) and asks ScreenCaptureKit for its content only
//! when that look changed, a stream is due to start, or every `LOOK`.

use std::fmt::Write as _;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc as std_mpsc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use objc2_core_graphics::{CGPreflightScreenCaptureAccess, CGRequestScreenCaptureAccess};
use questpump_core::protocol::ErrorCode;
use questpump_core::strip;
use screencapturekit::cm::SCFrameStatus;
use screencapturekit::cv::CVPixelBufferLockFlags;
use screencapturekit::error::SCStreamErrorCode;
use screencapturekit::prelude::*;
use screencapturekit::stream::configuration::color_space;
use tokio::sync::mpsc::UnboundedSender;

use super::Event;
use crate::games::procs;

/// Whether the Mac draws from its battery now: the source `IOKit`'s power-source snapshot says provides power (what
/// `pmset -g ps` prints first), asked without starting a process, since the capture asks every `LOOK` while a game runs
/// (on 2026-10-09 the Mac froze when a process was mid-exec, the kernel's exec deadlocking with runningboardd).
fn on_battery() -> bool {
	use std::ptr::NonNull;

	use objc2_core_foundation::{CFRetained, CFString, CFType};

	/// `kIOPMBatteryPowerKey`.
	const BATTERY: &str = "Battery Power"; // copy-lint: not shown (IOKit's key)

	// SAFETY: the C signatures of <IOKit/ps/IOPowerSources.h>, CFTypeRef and CFStringRef as their objc2 types; IOKit
	// ships with every macOS.
	#[link(name = "IOKit", kind = "framework")]
	unsafe extern "C" {
		fn IOPSCopyPowerSourcesInfo() -> *mut CFType;
		fn IOPSGetProvidingPowerSourceType(snapshot: &CFType) -> *const CFString;
	}

	// SAFETY: takes no arguments; the result is null on an error, else a snapshot this call owns (the Create rule).
	let Some(snapshot) = NonNull::new(unsafe { IOPSCopyPowerSourcesInfo() }) else {
		return false;
	};
	// SAFETY: a Copy function's result with its +1 retain count, released once when `snapshot` drops.
	let snapshot = unsafe { CFRetained::from_raw(snapshot) };
	// SAFETY: `snapshot` is the live blob the function takes; the result is null or one of IOKit's constant strings,
	// which this call does not own (the Get rule).
	let source = unsafe { IOPSGetProvidingPowerSourceType(&snapshot) };
	// SAFETY: a non-null result is a valid CFString at least while `snapshot` lives.
	unsafe { source.as_ref() }.is_some_and(|s| s.to_string() == BATTERY)
}

/// The capture's rate now: the battery rate while the Mac draws from its battery.
fn rate() -> i32 {
	i32::from(super::rate(on_battery()))
}

/// The capture's rate as the power source last asked for it, and when it was read.
struct Power {
	rate: i32,
	read_at: Instant,
}

impl Power {
	fn read() -> Self {
		Self { rate: rate(), read_at: Instant::now() }
	}

	/// Reads the power source again once `LOOK` has passed (unplugged or plugged in: the stream starts again at the new
	/// rate); whether it did.
	fn again(&mut self) -> bool {
		let due = self.read_at.elapsed() >= super::LOOK;
		if due {
			*self = Self::read();
		}
		due
	}
}

enum Cmd {
	/// The process of the client the helper follows; none when no client runs.
	Game(Option<u32>),
	/// The stream stopped by itself (`Stopped`), with the stop's message: without the grant the capture ends; with it a
	/// window gone or replaced starts again at once and quietly, any other stop is an error and starts again (`Retry`).
	Lost(ErrorCode, String),
}

/// The capture thread's handle: tell it which client runs.
pub struct Capture {
	cmds: std_mpsc::Sender<Cmd>,
}

impl Capture {
	pub fn start(events: UnboundedSender<Event>) -> Self {
		let (tx, rx) = std_mpsc::channel();
		let own = tx.clone();
		let spawned = std::thread::Builder::new().name("capture".into()).spawn(move || run(&events, &rx, &own));
		if let Err(e) = spawned {
			log::error!("capture: no thread: {e}");
		}
		Self { cmds: tx }
	}

	/// The process of the client the helper follows (`game::running`), whose window the capture reads; none when no
	/// client runs.
	pub fn set_game(&self, pid: Option<u32>) {
		let _ = self.cmds.send(Cmd::Game(pid));
	}
}

#[repr(C)]
struct MachTimebaseInfo {
	numer: u32,
	denom: u32,
}

// SAFETY: both are libSystem functions with these C signatures (<mach/mach_time.h>), always linked on macOS.
unsafe extern "C" {
	fn mach_absolute_time() -> u64;
	fn mach_timebase_info(info: *mut MachTimebaseInfo) -> i32;
}

/// A frame's `displayTime` (host time in Mach ticks) on the wall clock, in unix seconds.
fn wall_seconds(display_time: u64) -> f64 {
	let mut tb = MachTimebaseInfo { numer: 1, denom: 1 };
	// SAFETY: `tb` is a valid, writable MachTimebaseInfo for the call.
	let ok = unsafe { mach_timebase_info(&raw mut tb) } == 0 && tb.denom != 0;
	// SAFETY: mach_absolute_time takes no arguments and reads the host clock.
	let now_ticks = unsafe { mach_absolute_time() };
	let age_ticks = now_ticks.saturating_sub(display_time);
	let age_ns =
		if ok { u128::from(age_ticks) * u128::from(tb.numer) / u128::from(tb.denom) } else { u128::from(age_ticks) };
	let now = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0.0, |d| d.as_secs_f64());
	#[expect(clippy::cast_precision_loss, reason = "a frame's age in nanoseconds is far below 2^52")]
	let age = age_ns as f64 / 1e9;
	now - age
}

struct Output {
	events: UnboundedSender<Event>,
}

/// What the stream delivered since the last summary: the frames the corner changed in, the ones read, the ones without
/// the anchor (with the first one's anchor column), and the ones a check dropped (with the last reason), so a strip that
/// never reads (a colour pipeline that shifts it, an HDR output, something else in the corner) shows in the log instead
/// of only as a status that never changes.
struct Counts {
	frames: AtomicU32,
	read: AtomicU32,
	none: AtomicU32,
	rejected: AtomicU32,
	reason: Mutex<Option<String>>,
	corner: Mutex<Option<String>>,
}

static SEEN: Counts = Counts {
	frames: AtomicU32::new(0),
	read: AtomicU32::new(0),
	none: AtomicU32::new(0),
	rejected: AtomicU32::new(0),
	reason: Mutex::new(None),
	corner: Mutex::new(None),
};

/// Captured pixels for a log line: red, green and blue, one pixel after another.
fn pixels(cells: &[[u8; 3]]) -> String {
	cells.iter().map(|px| format!("{},{},{}", px[0], px[1], px[2])).collect::<Vec<_>>().join(" · ")
}

/// How often the counts go into the log while the strip does not read.
const SUMMARY: Duration = Duration::from_secs(60);

impl Counts {
	/// The first row's anchor, four calibration cells and eight data cells as captured, from the band the decoder found,
	/// for a rejected frame's line: how the game's colour settings and the colour pipeline changed them.
	fn note(&self, bgra: &[u8], width: usize, height: usize, stride: usize) {
		let cells = pixels(&strip::first_cells(bgra, width, height, stride, 13));
		if let Ok(mut reason) = self.reason.lock()
			&& let Some(r) = reason.as_mut()
		{
			let _ = write!(r, "; the first cells read {cells}");
		}
	}

	/// The anchor column of the summary's first frame without the anchor, from the corner down, for its line: what was
	/// in the corner instead of the strip.
	fn note_corner(&self, bgra: &[u8], width: usize, height: usize, stride: usize) {
		if let Ok(mut corner) = self.corner.lock()
			&& corner.is_none()
		{
			*corner = Some(pixels(&strip::anchor_column(bgra, width, height, stride)));
		}
	}

	fn count(&self, read: &strip::Read) {
		self.frames.fetch_add(1, Ordering::Relaxed);
		match read {
			strip::Read::Frame(_) => self.read.fetch_add(1, Ordering::Relaxed),
			strip::Read::NoStrip => self.none.fetch_add(1, Ordering::Relaxed),
			strip::Read::Rejected(e) => {
				if let Ok(mut reason) = self.reason.lock() {
					*reason = Some(e.to_string());
				}
				self.rejected.fetch_add(1, Ordering::Relaxed)
			}
		};
	}

	/// The counts since the last call, when some frame did not read; they start again from zero.
	fn summary(&self) -> Option<String> {
		let frames = self.frames.swap(0, Ordering::Relaxed);
		let read = self.read.swap(0, Ordering::Relaxed);
		let none = self.none.swap(0, Ordering::Relaxed);
		let rejected = self.rejected.swap(0, Ordering::Relaxed);
		let reason = self.reason.lock().ok().and_then(|mut r| r.take());
		let corner = self.corner.lock().ok().and_then(|mut c| c.take());
		(frames > read).then(|| {
			let why = reason.map_or_else(String::new, |r| format!(" (last: {r})"));
			// copy-lint: not shown (a log line)
			let seen = corner.map_or_else(String::new, |c| format!(" (the first one's anchor column, top down: {c})"));
			// copy-lint: not shown (a log line)
			format!("{frames} frames in a minute: {read} read, {none} without the strip{seen}, {rejected} dropped{why}")
		})
	}
}

impl SCStreamOutputTrait for Output {
	fn did_output_sample_buffer(&self, sample: CMSampleBuffer, of_type: SCStreamOutputType) {
		if of_type != SCStreamOutputType::Screen || sample.frame_status() != Some(SCFrameStatus::Complete) {
			return; // Idle: the corner did not change
		}
		let Some(display_time) = sample.display_time() else { return };
		let Some(pixels) = sample.pixel_buffer() else { return };
		let Ok(guard) = pixels.lock(CVPixelBufferLockFlags::READ_ONLY) else { return };
		let (width, height, stride) = (guard.width(), guard.height(), guard.bytes_per_row());
		// SAFETY: the read-only lock guard lives until the end of this function, keeping the buffer mapped and
		// unchanged; nothing here unlocks it, writes to it, or keeps the slice past the decode below.
		let Some(bytes) = (unsafe { guard.as_slice() }) else { return };
		let read = strip::read_band(bytes, width, height, stride);
		SEEN.count(&read);
		match read {
			strip::Read::Rejected(_) => SEEN.note(bytes, width, height, stride),
			strip::Read::NoStrip => SEEN.note_corner(bytes, width, height, stride),
			strip::Read::Frame(_) => {}
		}
		let _ = self.events.send(super::event(read, || wall_seconds(display_time)));
	}
}

struct Stopped {
	events: UnboundedSender<Event>,
	own: std_mpsc::Sender<Cmd>,
}

impl SCStreamDelegateTrait for Stopped {
	fn did_stop_with_error(&self, error: SCError) {
		// a grant revoked while the stream runs may stop it with another code (-3817, -3821): the grant decides, as at
		// the start
		let declined = error.stream_error_code() == Some(SCStreamErrorCode::UserDeclined);
		if declined || !CGPreflightScreenCaptureAccess() {
			let _ = self.events.send(Event::Error(ErrorCode::ScreenPermission, error.to_string()));
			let _ = self.own.send(Cmd::Lost(ErrorCode::ScreenPermission, String::new()));
			return;
		}
		// with the grant held the capture thread decides whether it is an error: Apple stops an including filter's
		// stream when its window closes (a game that quits or makes its window again), which is none (`run`)
		let _ = self.own.send(Cmd::Lost(ErrorCode::StreamStopped, error.to_string()));
	}
}

/// When a stream that stopped with the grant held starts again: 2 s after it stopped, twice as long after each stop in
/// a row up to a minute; a game start, or a stream that ran a minute before it stopped, starts the count again.
#[derive(Default)]
struct Retry {
	at: Option<Instant>,
	wait: Duration,
}

impl Retry {
	const FIRST: Duration = Duration::from_secs(2);
	const MOST: Duration = Duration::from_mins(1);

	fn failed(&mut self) {
		self.wait = if self.at.is_some() { (self.wait * 2).min(Self::MOST) } else { Self::FIRST };
		self.at = Some(Instant::now() + self.wait);
	}

	/// Whether a stream may start now.
	fn due(&self) -> bool {
		self.at.is_none_or(|t| Instant::now() >= t)
	}
}

/// The running stream and what it was started for: the client's process, its window's id and corner, the display's
/// frame (a new resolution or arrangement changes the points the source rect is in) and the rate.
struct Live {
	stream: SCStream,
	pid: u32,
	window: u32,
	corner: CGPoint,
	display: CGRect,
	rate: i32,
	since: Instant,
}

impl Live {
	fn stop(self) {
		let _ = self.stream.stop_capture();
	}
}

/// What a stream starts on: the followed client's main window, the one window the capture includes, and the display it
/// is on.
struct Found {
	display: SCDisplay,
	window: SCWindow,
}

/// What the window server shows of the client at one look (`procs`): its largest on-screen window of the normal layer,
/// by number and bounds, and the bounds of the display at its corner. A tick asks ScreenCaptureKit for its content only
/// when this changed, because each content query has replayd list every window and app on the Mac and check the grant
/// with tccd (on 2026-10-08 about 600 of the helper's log lines and 8 of tccd's a query, 30 queries a minute), while
/// the window server's list takes a few milliseconds and asks tccd only once a process. A new main window of the client,
/// or one back from another Space, changes the look, so the stream starts on it at the next tick.
#[derive(PartialEq)]
struct Look {
	pid: u32,
	window: Option<(u32, objc2_core_foundation::CGRect)>,
	display: Option<objc2_core_foundation::CGRect>,
}

impl Look {
	fn take(pid: u32) -> Self {
		let window = procs::largest(&procs::windows(), pid);
		let display = window.and_then(|(_, bounds)| procs::display_at(bounds.origin));
		Self { pid, window, display }
	}

	/// Whether a tick may leave the content query out: the last query that answered saw this look, and the stream runs
	/// for this client's window at the power source's rate, or none runs and none is due (that query found no window to
	/// start it on, or `Retry`'s wait). The window's number keeps a stream that followed a window the client has since
	/// closed from passing for settled (a game that makes its window again, the old one still there at the query).
	fn settled(&self, seen: Option<&(Self, bool)>, stream: Option<&Live>, rate: i32, retry: &Retry) -> bool {
		let Some((look, found)) = seen else { return false };
		look == self
			&& match stream {
				Some(live) => {
					live.pid == self.pid && live.rate == rate && self.window.map(|(id, _)| id) == Some(live.window)
				}
				None => !found || !retry.due(),
			}
	}
}

/// The main window of the client process `pid` (its largest on-screen window of the normal layer, of two as large the
/// lower number, as the look takes it) and the display at its corner. The query lists on-screen windows alone: the
/// stream includes the client's window and nothing else, so an off-screen one has nothing to give.
fn locate(pid: u32) -> Result<Option<Found>, SCError> {
	let content = SCShareableContent::create().with_on_screen_windows_only(true).get()?;
	let area = |w: &SCWindow| w.frame().size.width * w.frame().size.height;
	let window = content
		.windows()
		.into_iter()
		.filter(|w| {
			w.owning_application().and_then(|app| u32::try_from(app.process_id()).ok()) == Some(pid)
				&& w.window_layer() == 0
				&& w.is_on_screen()
		})
		.max_by(|a, b| area(a).total_cmp(&area(b)).then(b.window_id().cmp(&a.window_id())));
	let Some(window) = window else { return Ok(None) };
	let corner = window.frame().origin;
	let display = content.displays().into_iter().find(|d| d.frame().contains_point(corner));
	Ok(display.map(|display| Found { display, window }))
}

fn start_stream(
	found: &Found,
	rate: i32,
	events: &UnboundedSender<Event>,
	own: &std_mpsc::Sender<Cmd>,
) -> Result<SCStream, SCError> {
	let (display, window) = (&found.display, &found.window);
	// the game's window alone, placed on its display: Apple's including filter leaves out the menu bar (its default
	// for this filter), the desktop and every other window, and `sourceRect` stays in the display's points
	let filter = SCContentFilter::create().with_display(display).with_including_windows(&[window]).build()?;
	let scale = f64::from(filter.point_pixel_scale()).max(1.0);
	let (corner, origin) = (window.frame().origin, display.frame().origin);
	let (w, h) = (strip::WIDTH, strip::CAPTURE_HEIGHT);
	#[expect(clippy::cast_precision_loss, reason = "390 and 96 pixels")]
	let size = CGSize { width: w as f64 / scale, height: h as f64 / scale };
	let config = SCStreamConfiguration::new()
		.with_source_rect(CGRect { origin: CGPoint { x: corner.x - origin.x, y: corner.y - origin.y }, size })
		.with_width(u32::try_from(w).unwrap_or(390))
		.with_height(u32::try_from(h).unwrap_or(96))
		.with_minimum_frame_interval(&CMTime::new(1, rate))
		.with_pixel_format(PixelFormat::BGRA)
		.with_queue_depth(3)
		.with_shows_cursor(false)
		.with_scales_to_fit(false)
		.with_color_space_name(color_space::SRGB)?;
	let mut stream =
		SCStream::new_with_delegate(&filter, &config, Stopped { events: events.clone(), own: own.clone() })?;
	stream.add_output_handler(Output { events: events.clone() }, SCStreamOutputType::Screen)?;
	// before the stream starts, so the runtime takes the stream's first frame as its first (as the Windows backend)
	let (screen, id) = (display.display_id(), window.window_id());
	// copy-lint: not shown (a log line)
	let name = format!("display {screen}, window {id}, {rate} frames a second at most");
	let _ = events.send(Event::Display { name, scale, hdr: false, backend: "sck" }); // the stream asks for sRGB
	stream.start_capture()?;
	Ok(stream)
}

/// The marker in the data folder that says the window's button asked for the grant once.
const ASKED: &str = "screen-asked";

/// Asks for the Screen Recording grant, from the window's button. The first click asks macOS, which shows its prompt
/// only the first time, and leaves a marker in the data folder (`ASKED`); every click after it, while the marker is
/// there, opens the Screen Recording pane in System Settings, where the switch is. A grant takes effect when the helper
/// starts again, which the window then offers.
pub fn request_permission(data_dir: &std::path::Path) {
	if CGPreflightScreenCaptureAccess() {
		return;
	}
	let marker = data_dir.join(ASKED);
	let asked = marker.exists();
	let _ = CGRequestScreenCaptureAccess();
	if let Err(e) = std::fs::write(&marker, b"") {
		log::warn!("screen-asked: {e}");
	}
	if asked {
		let pane = "x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture";
		if let Err(e) = crate::desk::menu_pages::about::open_url(pane) {
			log::warn!("System Settings: {e}");
		}
	}
}

/// Whether this process may read the screen: a grant given while it ran shows only in a process started after it.
pub fn screen_granted() -> bool {
	CGPreflightScreenCaptureAccess()
}

/// Whether the window's button asked for the grant before (`request_permission`'s marker): the window then offers the
/// restart macOS needs to apply it.
pub fn permission_asked(data_dir: &std::path::Path) -> bool {
	data_dir.join(ASKED).exists()
}

/// Stops a stream the system stopped with the grant held (`Cmd::Lost`) and says whether the window it held closed or the
/// client made another (the look names no window or another number): the next tick then starts on the new one, with no
/// error and no wait. Else the system stopped it (-3821) or a display went away, an error, and the next look starts it
/// again after `Retry`'s wait (a stream that ran a minute starts that count again).
fn closed(live: Option<Live>, game: Option<u32>, retry: &mut Retry) -> bool {
	let Some(live) = live else { return false };
	let replaced = game.is_none_or(|pid| Look::take(pid).window.map(|(id, _)| id) != Some(live.window));
	if live.since.elapsed() >= Retry::MOST {
		*retry = Retry::default();
	}
	live.stop();
	replaced
}

fn run(events: &UnboundedSender<Event>, cmds: &std_mpsc::Receiver<Cmd>, own: &std_mpsc::Sender<Cmd>) {
	if !CGPreflightScreenCaptureAccess() {
		// no prompt here: the window says why first, and its button asks (`request_permission`)
		let _ = events.send(Event::Error(ErrorCode::ScreenPermission, "Screen Recording is not allowed yet".into()));
		return;
	}
	let mut game: Option<u32> = None;
	let mut stream: Option<Live> = None;
	let mut retry = Retry::default();
	let mut seen: Option<(Look, bool)> = None; // the last answered content query's look, and whether it found the window
	let mut power = Power::read();
	let mut summary_at = Instant::now();
	loop {
		if summary_at.elapsed() >= SUMMARY {
			summary_at = Instant::now();
			if let Some(line) = SEEN.summary() {
				log::info!("capture: {line}");
			}
		}
		match cmds.recv_timeout(super::TICK) {
			Ok(Cmd::Game(pid)) => {
				game = pid;
				retry = Retry::default(); // a game start tries at once
			}
			Ok(Cmd::Lost(ErrorCode::ScreenPermission, _)) | Err(std_mpsc::RecvTimeoutError::Disconnected) => break,
			Ok(Cmd::Lost(code, detail)) => {
				if closed(stream.take(), game, &mut retry) {
					let _ = events.send(Event::NoStrip);
					seen = None;
					continue;
				}
				let _ = events.send(Event::Error(code, detail));
				retry.failed();
			}
			Err(std_mpsc::RecvTimeoutError::Timeout) => {}
		}
		let Some(pid) = game else {
			if let Some(live) = stream.take() {
				live.stop();
			}
			continue;
		};
		let slow = power.again();
		// the content query only when the client's window or the display changed, a stream is due to start, or at the
		// slow look, where a failed query also finds a grant gone while no stream runs (a running one stops: `Stopped`)
		let look = Look::take(pid);
		if !slow && look.settled(seen.as_ref(), stream.as_ref(), power.rate, &retry) {
			continue;
		}
		// while the grant dialog is up the content query fails: try again at the next tick, unless the grant is gone
		// (revoked, or the monthly consent declined), which the query's own error never says
		let found = match locate(pid) {
			Ok(found) => found,
			Err(e) => {
				if !CGPreflightScreenCaptureAccess() {
					let _ = events.send(Event::Error(ErrorCode::ScreenPermission, e.to_string()));
					break;
				}
				continue;
			}
		};
		seen = Some((look, found.is_some()));
		let Some(found) = found else {
			if let Some(live) = stream.take() {
				live.stop();
				let _ = events.send(Event::NoStrip); // the window left the screen (another Space, minimised)
			}
			continue;
		};
		let (window, corner, screen) = (found.window.window_id(), found.window.frame().origin, found.display.frame());
		if stream.as_ref().is_some_and(|l| {
			l.pid == pid && l.window == window && l.corner == corner && l.display == screen && l.rate == power.rate
		}) {
			// the same window of the same client where the stream looks, on the same display mode, at the rate the power
			// source asks
			continue;
		}
		if !retry.due() {
			continue;
		}
		if let Some(live) = stream.take() {
			live.stop();
		}
		match start_stream(&found, power.rate, events, own) {
			Ok(s) => {
				let since = Instant::now();
				stream = Some(Live { stream: s, pid, window, corner, display: screen, rate: power.rate, since });
			}
			Err(e) => {
				// no error on this path carries a stream code (start_capture's is a message): the grant decides
				let granted = CGPreflightScreenCaptureAccess();
				let code = if granted { ErrorCode::StreamStopped } else { ErrorCode::ScreenPermission };
				let _ = events.send(Event::Error(code, e.to_string()));
				if !granted {
					return;
				}
				retry.failed();
			}
		}
	}
	if let Some(live) = stream.take() {
		live.stop();
	}
}
