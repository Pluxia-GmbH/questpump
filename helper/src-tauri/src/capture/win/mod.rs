//! The Windows backend (rust-helper `windows-capture-and-install.md`): 390 × 96 physical pixels from the top-left of
//! the game's client area (the strip's band, and room for a windowed game's title bar), read through the best backend
//! the system has, chosen for each game session and falling to the next when one fails: Windows.Graphics.Capture on
//! the window where its border can be turned off (build 20348 on), DXGI Desktop Duplication of the display on the
//! adapter that drives it (Windows 10, hybrid GPUs), GDI last. BGRA, or an HDR desktop's scRGB brought back to its SDR
//! white; at most 60 frames a second, 10 on battery power and through GDI. Each frame is decoded where it arrives into
//! a value and sent on; the streams run on threads of their own, started and stopped by one thread named "capture"
//! that looks at the game every 2 s, as the macOS backend does.

pub mod banner;
mod clock;
mod display;
mod dxgi;
mod gdi;
mod wgc;
mod window;

use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc as std_mpsc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use questpump_core::protocol::ErrorCode;
use questpump_core::strip::{self, Read, Scrgb};
use tokio::sync::mpsc::UnboundedSender;
use windows::Win32::Foundation::{E_ACCESSDENIED, HWND};
use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
use windows::Win32::UI::HiDpi::{DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, SetProcessDpiAwarenessContext};

use super::Event;
use crate::game;

/// GDI's frames a second at most: each is a screen copy on the CPU.
const GDI_RATE: u32 = 10;
/// How long a pulling stream's thread may take to start.
const START_WAIT: Duration = Duration::from_secs(5);
/// How long a stream on the shown game may go without a frame, while the player's desktop takes the input, before it
/// counts as failed and the next backend takes over: the game presents frame after frame while it shows, so silence
/// means a broken stream (a WGC handler's dropped error, a duplication that never updates), never a quiet screen.
const STALL: Duration = Duration::from_secs(10);

/// The band read at the client area's top-left, in pixels: the strip's width, and its rows plus a windowed game's
/// title bar.
#[expect(clippy::cast_possible_truncation, reason = "390 and 96 pixels")]
const BAND: (u32, u32) = (strip::WIDTH as u32, strip::CAPTURE_HEIGHT as u32);

/// The capture thread's handle: tell it which client runs.
pub struct Capture {
	running: std_mpsc::Sender<Option<u32>>,
}

impl Capture {
	pub fn start(events: UnboundedSender<Event>) -> Self {
		let (tx, rx) = std_mpsc::channel();
		let spawned = std::thread::Builder::new().name("capture".into()).spawn(move || run(&events, &rx));
		if let Err(e) = spawned {
			log::error!("capture: no thread: {e}");
		}
		Self { running: tx }
	}

	/// The process of the client the helper follows (`game::running`), whose window the capture reads; none when no
	/// client runs.
	pub fn set_game(&self, pid: Option<u32>) {
		let _ = self.running.send(pid);
	}
}

fn run(events: &UnboundedSender<Event>, running: &std_mpsc::Receiver<Option<u32>>) {
	dpi_aware();
	// while a client runs (its process) and a backend is left; none after every one failed, until a client starts
	let mut chain: Option<(u32, Chain)> = None;
	let mut live: Option<Live> = None;
	let mut look: Option<Look> = None;
	loop {
		match running.recv_timeout(super::TICK) {
			Ok(pid) => chain = pid.map(|pid| (pid, Chain::new(wgc::probe()))),
			Err(std_mpsc::RecvTimeoutError::Disconnected) => break,
			Err(std_mpsc::RecvTimeoutError::Timeout) => {}
		}
		let Some((pid, session)) = chain.as_mut() else {
			end(&mut live, None);
			continue;
		};
		watch(*pid, session, &mut live, &mut look, events);
		if session.left.is_empty() {
			let _ = events.send(Event::Error(ErrorCode::StreamStopped, session.why.join("; ")));
			chain = None;
		}
	}
	end(&mut live, None);
}

/// One look at the window of the client process `pid`: the stream keeps running and follows the window inside its
/// display, starts again for another window, display, colour format or rate or after it stopped, and stops while the
/// window does not show.
fn watch(
	pid: u32,
	chain: &mut Chain,
	live: &mut Option<Live>,
	look: &mut Option<Look>,
	events: &UnboundedSender<Event>,
) {
	let Some(game) = game::window(pid) else {
		hide(live, chain, events);
		return;
	};
	let hwnd = game.hwnd();
	// minimised, cloaked or on no display: the protocol's no_strip (rust-helper `protocol-and-mirror.md`, statuses)
	let monitor = if window::shows(hwnd) { display::of(hwnd) } else { None };
	let Some(monitor) = monitor else {
		hide(live, chain, events);
		return;
	};
	let look = match look.take_if(|l| !l.stale(&monitor)) {
		Some(fresh) => look.insert(fresh),
		None => look.insert(Look::read(&monitor)),
	};
	// a locked PC or a UAC prompt holds the input: no backend sees the game then, which is no stall and no failure
	if !window::desktop_active() {
		if let Some(l) = live.as_ref() {
			l.stream.heard().now();
		}
		return;
	}
	if let Some(stalled) = live.take_if(|l| l.stream.stalled()) {
		// copy-lint: not shown (a log line)
		let why = stalled.stream.stop().err().unwrap_or_else(|| format!("no frame in {} s", STALL.as_secs()));
		log::warn!("capture: {} stopped: {why}", stalled.kind);
		chain.fail(stalled.kind, &why);
	}
	if let Some(bordered) = live.take_if(|l| l.kind == Kind::Wgc && wgc::border_denied()) {
		let _ = bordered.stream.stop();
		chain.fail(Kind::Wgc, "borderless capture was denied, so a border would show around the game");
	}
	let target = Target {
		window: hwnd.0.addr(),
		monitor: monitor.handle.0.addr(),
		hdr: look.hdr,
		white: look.white,
		rate: look.rate,
	};
	if let Some(l) = live.as_ref()
		&& l.target == target
		&& !l.stream.finished()
	{
		if let Some(at) = l.kind.origin(hwnd) {
			l.origin.set(at); // moved, or switched between windowed and fullscreen: no restart
		}
		return;
	}
	end(live, Some(chain));
	*live = start(chain, hwnd, &monitor, target, events);
}

/// Starts the first backend in the chain that starts, dropping each that does not; none when every one failed or the
/// window went away meanwhile.
fn start(
	chain: &mut Chain,
	hwnd: HWND,
	monitor: &display::Monitor,
	target: Target,
	events: &UnboundedSender<Event>,
) -> Option<Live> {
	while let Some(kind) = chain.left.first().copied() {
		let Some(at) = kind.origin(hwnd) else {
			if !window::shows(hwnd) {
				return None; // the window went away: the next look tries again
			}
			chain.fail(kind, "no client origin for the shown window");
			continue;
		};
		let origin = Origin::new(at);
		let rate = kind.rate(target.rate);
		// before the stream starts, so the runtime takes the stream's first frame as its first
		// copy-lint: not shown (a log line)
		let name = format!("{} through {kind}, {rate} frames a second at most", monitor.name);
		let _ = events.send(Event::Display { name, scale: window::scale(hwnd), hdr: target.hdr, backend: kind.key() });
		let started = match kind {
			Kind::Wgc => wgc::start(hwnd, chain.probes, &target, origin.clone(), events).map(Stream::Wgc),
			Kind::Dxgi => dxgi::start(monitor.handle, &target, origin.clone(), events).map(Stream::Worker),
			Kind::Gdi => gdi::start(rate, origin.clone(), events).map(Stream::Worker),
		};
		match started {
			Ok(stream) => return Some(Live { kind, target, origin, stream }),
			Err(e) if !window::shows(hwnd) => {
				log::info!("capture: {kind} did not start on a window that went away meanwhile: {e}");
				return None; // the backend is not to blame: the next look tries it again
			}
			Err(e) => {
				log::warn!("capture: {kind} did not start: {e}");
				chain.fail(kind, &e);
			}
		}
	}
	None
}

/// Stops the stream, if one runs, and joins its threads; one that stopped on an error of its own leaves the chain.
fn end(live: &mut Option<Live>, chain: Option<&mut Chain>) {
	let Some(Live { kind, stream, .. }) = live.take() else { return };
	if let Err(e) = stream.stop() {
		log::warn!("capture: {kind} stopped: {e}");
		if let Some(chain) = chain {
			chain.fail(kind, &e);
		}
	}
}

/// The window does not show: the stream stops, the strip is out of sight once, and the loop keeps looking.
fn hide(live: &mut Option<Live>, chain: &mut Chain, events: &UnboundedSender<Event>) {
	if live.is_some() {
		end(live, Some(chain));
		let _ = events.send(Event::NoStrip);
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
	Wgc,
	Dxgi,
	Gdi,
}

impl Kind {
	/// Where the band starts for this backend: inside the window's frame for WGC, on the screen for DXGI and GDI.
	fn origin(self, hwnd: HWND) -> Option<(i32, i32)> {
		match self {
			Self::Wgc => window::client_in_frame(hwnd),
			Self::Dxgi | Self::Gdi => window::client_origin(hwnd),
		}
	}

	fn rate(self, rate: u32) -> u32 {
		if self == Self::Gdi { rate.min(GDI_RATE) } else { rate }
	}

	/// The backend as the anonymous counts' `capture_day` names it.
	fn key(self) -> &'static str {
		match self {
			Self::Wgc => "wgc",
			Self::Dxgi => "dxgi",
			Self::Gdi => "gdi",
		}
	}
}

impl fmt::Display for Kind {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(match self {
			Self::Wgc => "Windows.Graphics.Capture",
			Self::Dxgi => "DXGI Desktop Duplication", // copy-lint: not shown (a log line)
			Self::Gdi => "GDI",
		})
	}
}

/// The backends left to try in this game session, best first, and why the others left: one that fails to start, or
/// stops on an error of its own, leaves it.
struct Chain {
	probes: wgc::Probes,
	left: Vec<Kind>,
	why: Vec<String>,
}

impl Chain {
	fn new(probes: wgc::Probes) -> Self {
		Self { probes, left: order(probes), why: Vec::new() }
	}

	fn fail(&mut self, kind: Kind, why: &str) {
		self.left.retain(|&k| k != kind);
		self.why.push(format!("{kind}: {why}"));
	}
}

/// The backends to try, best first: Windows.Graphics.Capture only where it captures and can turn its border off
/// (Windows 10 would draw a yellow border around the game).
fn order(probes: wgc::Probes) -> Vec<Kind> {
	let wgc = probes.supported && probes.border;
	[Kind::Wgc, Kind::Dxgi, Kind::Gdi].into_iter().filter(|&k| k != Kind::Wgc || wgc).collect()
}

/// What a stream was started for; any change starts it again (a move inside the display only moves its origin).
#[derive(Debug, Clone, Copy, PartialEq)]
struct Target {
	window: usize,
	monitor: usize,
	hdr: bool,
	/// The display's SDR white, as a multiple of 80 nits (1.0 on SDR).
	white: f32,
	rate: u32,
}

/// The running stream and what it was started for.
struct Live {
	kind: Kind,
	target: Target,
	origin: Origin,
	stream: Stream,
}

enum Stream {
	Wgc(wgc::Session),
	Worker(Worker),
}

impl Stream {
	fn finished(&self) -> bool {
		match self {
			Self::Wgc(session) => session.finished(),
			Self::Worker(worker) => worker.finished(),
		}
	}

	/// A running stream that delivers nothing (`STALL`).
	fn stalled(&self) -> bool {
		self.heard().silence() >= STALL
	}

	/// When the stream last had a frame: WGC's frame arrived, DXGI's desktop image updated, GDI copied the screen (a
	/// pulling copy always has one, so the last backend never stalls).
	fn heard(&self) -> &Heard {
		match self {
			Self::Wgc(session) => session.heard(),
			Self::Worker(worker) => &worker.heard,
		}
	}

	/// Ends the stream and joins its threads; what ended it when it stopped on an error of its own.
	fn stop(self) -> Result<(), String> {
		match self {
			Self::Wgc(session) => session.stop(),
			Self::Worker(worker) => worker.stop(),
		}
	}
}

/// A pulling stream's thread (DXGI, GDI): it runs until halted, and returns what ended it when it stopped on its own.
struct Worker {
	halt: Arc<AtomicBool>,
	heard: Heard,
	thread: JoinHandle<Result<(), String>>,
}

impl Worker {
	/// Runs `setup` on a new thread and then `run` with what it made, which stays on that thread (COM objects, GDI
	/// handles), and the stream's `Heard` to mark each frame on; a failed setup is the start's error.
	fn spawn<S>(
		name: &str,
		setup: impl FnOnce() -> Result<S, String> + Send + 'static,
		run: impl FnOnce(S, &AtomicBool, &Heard) -> Result<(), String> + Send + 'static,
	) -> Result<Self, String> {
		let halt = Arc::new(AtomicBool::new(false));
		let flag = Arc::clone(&halt);
		let heard = Heard::new();
		let marks = heard.clone();
		let (ready_tx, ready_rx) = std_mpsc::sync_channel(1);
		let thread = std::thread::Builder::new()
			.name(name.into())
			.spawn(move || match setup() {
				Ok(state) => {
					let _ = ready_tx.send(Ok(()));
					run(state, &flag, &marks)
				}
				Err(e) => {
					let _ = ready_tx.send(Err(e));
					Ok(())
				}
			})
			.map_err(|e| format!("no thread: {e}"))?;
		match ready_rx.recv_timeout(START_WAIT) {
			Ok(Ok(())) => Ok(Self { halt, heard, thread }),
			Ok(Err(e)) => {
				let _ = thread.join();
				Err(e)
			}
			Err(std_mpsc::RecvTimeoutError::Disconnected) => {
				let _ = thread.join();
				Err("its thread ended while starting".into())
			}
			Err(std_mpsc::RecvTimeoutError::Timeout) => {
				halt.store(true, Ordering::Relaxed); // it ends when its setup returns
				Err(format!("no start in {} s", START_WAIT.as_secs()))
			}
		}
	}

	fn finished(&self) -> bool {
		self.thread.is_finished()
	}

	fn stop(self) -> Result<(), String> {
		self.halt.store(true, Ordering::Relaxed);
		self.thread.join().unwrap_or_else(|_| Err("its thread panicked".into()))
	}
}

/// When a stream's last frame came, in milliseconds after it started (0 until the first, so a stream that never has
/// one stalls `STALL` after its start).
#[derive(Clone)]
struct Heard {
	since: Instant,
	at: Arc<AtomicU64>,
}

impl Heard {
	fn new() -> Self {
		Self { since: Instant::now(), at: Arc::new(AtomicU64::new(0)) }
	}

	fn now(&self) {
		let ms = u64::try_from(self.since.elapsed().as_millis()).unwrap_or(u64::MAX);
		self.at.store(ms, Ordering::Relaxed);
	}

	fn silence(&self) -> Duration {
		self.since.elapsed().saturating_sub(Duration::from_millis(self.at.load(Ordering::Relaxed)))
	}
}

/// What the loop reads again every 30 s and when the window moves to another display: the power source's rate, and
/// whether the display is an HDR output and its SDR white.
struct Look {
	at: Instant,
	monitor: usize,
	rate: u32,
	hdr: bool,
	white: f32,
}

impl Look {
	fn read(monitor: &display::Monitor) -> Self {
		let hdr = display::is_hdr(monitor);
		let white = if hdr { display::sdr_white(monitor) } else { 1.0 };
		Self { at: Instant::now(), monitor: monitor.handle.0.addr(), rate: rate(), hdr, white }
	}

	fn stale(&self, monitor: &display::Monitor) -> bool {
		self.at.elapsed() >= super::LOOK || self.monitor != monitor.handle.0.addr()
	}
}

/// Where a stream reads the band, in its own coordinates (the window's frame for WGC, the screen for DXGI and GDI):
/// one cell shared with the stream's thread, so the loop moves it without a restart and a read never mixes two moves.
#[derive(Clone)]
struct Origin(Arc<AtomicU64>);

impl Origin {
	fn new(at: (i32, i32)) -> Self {
		Self(Arc::new(AtomicU64::new(pack(at))))
	}

	fn set(&self, at: (i32, i32)) {
		self.0.store(pack(at), Ordering::Relaxed);
	}

	fn get(&self) -> (i32, i32) {
		unpack(self.0.load(Ordering::Relaxed))
	}
}

fn pack((x, y): (i32, i32)) -> u64 {
	(u64::from(x.cast_unsigned()) << 32) | u64::from(y.cast_unsigned())
}

fn unpack(cell: u64) -> (i32, i32) {
	let half = |v: u64| u32::try_from(v & u64::from(u32::MAX)).map_or(0, u32::cast_signed);
	(half(cell >> 32), half(cell))
}

/// How a frame's pixels are laid out: BGRA, or `Rgba16F` scRGB from an HDR desktop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Format {
	Bgra8,
	Rgba16F,
}

/// A stream's reader of the band: BGRA as painted, or scRGB brought back to the display's SDR white.
struct Band {
	white: f32,
	scrgb: Option<Scrgb>,
}

impl Band {
	/// The scRGB table is made at the start on an HDR display, and on the first `Rgba16F` frame elsewhere.
	fn new(hdr: bool, white: f32) -> Self {
		Self { white, scrgb: hdr.then(|| Scrgb::new(white)) }
	}

	/// Reads the band's 390 × 96 pixels, `pitch` bytes a row.
	fn read(&mut self, format: Format, pixels: &[u8], pitch: usize) -> Read {
		let (width, height) = (strip::WIDTH, strip::CAPTURE_HEIGHT);
		match format {
			Format::Bgra8 => strip::read_band(pixels, width, height, pitch),
			Format::Rgba16F => {
				let white = self.white;
				self.scrgb.get_or_insert_with(|| Scrgb::new(white)).read_band(pixels, width, height, pitch)
			}
		}
	}
}

/// Sends what a read of the band shows (`capture::event`, as `macos.rs` sends it).
fn send(events: &UnboundedSender<Event>, read: Read, shown: impl FnOnce() -> f64) {
	let _ = events.send(super::event(read, shown));
}

/// The capture's rate now: the power source read with `GetSystemPowerStatus`.
fn rate() -> u32 {
	let mut status = SYSTEM_POWER_STATUS::default();
	// SAFETY: GetSystemPowerStatus writes one SYSTEM_POWER_STATUS into `status`, which lives for the call.
	match unsafe { GetSystemPowerStatus(&raw mut status) } {
		Ok(()) => rate_for(status.ACLineStatus),
		Err(e) => {
			log::info!("capture: the power source: {e}");
			u32::from(super::RATE)
		}
	}
}

/// The capture's rate for an AC line status: the battery rate while the PC draws from its battery (0), the full rate
/// on power (1) and when Windows does not know (255).
fn rate_for(ac_line: u8) -> u32 {
	u32::from(super::rate(ac_line == 0))
}

/// Makes the process per-monitor-v2 DPI aware, so window, screen and DXGI coordinates are all physical pixels and
/// `DuplicateOutput1` accepts the process (windows-capture's `DxgiDuplicationApi` does the same); `E_ACCESSDENIED`
/// means the awareness is set already.
fn dpi_aware() {
	// SAFETY: a predefined awareness context; the call only sets the process's DPI mode.
	match unsafe { SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) } {
		Ok(()) => {}
		Err(e) if e.code() == E_ACCESSDENIED => {}
		Err(e) => log::warn!("capture: per-monitor DPI awareness: {e}"),
	}
}

/// A Win32 struct's size for its `cbSize` field or a byte count (0, which the call refuses, past `u32`).
fn byte_size<T>() -> u32 {
	u32::try_from(size_of::<T>()).unwrap_or(0)
}
