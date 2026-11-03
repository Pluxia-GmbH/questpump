//! Windows.Graphics.Capture through `windows-capture` 2.0.1 (rust-helper `windows-capture-and-install.md`): the game's
//! window as the item, the border and the cursor off and the 24H2 minimum update interval where the system has them
//! (a setting it lacks fails the start, so each follows its probe), `Bgra8`, or `Rgba16F` on an HDR output, and a crop
//! of the band at the client area's top-left inside the window's frame. The crate runs the session on a thread of its
//! own (`start_free_threaded`: `start` would take this thread's apartment) and calls `on_frame_arrived` there;
//! `CaptureControl::stop` joins it, so the capture thread calls it.

use std::sync::Once;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use tokio::sync::mpsc::UnboundedSender;
use windows::Foundation::Metadata::ApiInformation;
use windows::Graphics::Capture::{GraphicsCaptureAccess, GraphicsCaptureAccessKind};
use windows::Security::Authorization::AppCapabilityAccess::AppCapabilityAccessStatus;
use windows::Win32::Foundation::HWND;
use windows::core::HSTRING;
use windows_capture::capture::{CaptureControl, Context, GraphicsCaptureApiHandler};
use windows_capture::frame::Frame;
use windows_capture::graphics_capture_api::{self as api, GraphicsCaptureApi, InternalCaptureControl};
use windows_capture::settings::{
	ColorFormat, CursorCaptureSettings, DirtyRegionSettings, DrawBorderSettings, MinimumUpdateIntervalSettings,
	SecondaryWindowSettings, Settings,
};
use windows_capture::window::Window;

use super::{BAND, Band, Event, Format, Heard, Origin, Target, clock, send};

/// What this system's Windows.Graphics.Capture can do: capture at all, and turn the border, the cursor and the update
/// rate (Windows 10 1803, build 20348, 2004 and 24H2).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[expect(clippy::struct_excessive_bools, reason = "one answer for each of the crate's probes")]
pub struct Probes {
	pub supported: bool,
	pub border: bool,
	pub cursor: bool,
	pub interval: bool,
}

/// Asks the system what it supports; a probe that fails reads as unsupported.
pub fn probe() -> Probes {
	let ask = |what: &str, probe: fn() -> Result<bool, api::Error>| {
		probe().unwrap_or_else(|e| {
			log::info!("capture: Windows.Graphics.Capture {what}: {e}");
			false
		})
	};
	Probes {
		supported: ask("support", GraphicsCaptureApi::is_supported),
		// copy-lint: not shown (a log line)
		border: ask("border setting", GraphicsCaptureApi::is_border_settings_supported) && !border_denied(),
		// copy-lint: not shown (a log line)
		cursor: ask("cursor setting", GraphicsCaptureApi::is_cursor_settings_supported),
		// copy-lint: not shown (a log line)
		interval: ask("update interval", GraphicsCaptureApi::is_minimum_update_interval_supported),
	}
}

/// The borderless request was denied: the border would stay around the game however the session is set (Microsoft
/// Learn, `IsBorderRequired`: "setting this property to false will succeed, but the value will be ignored").
static BORDER_DENIED: AtomicBool = AtomicBool::new(false);

/// Whether the system or the player denied borderless capture: WGC then leaves the chain, since DXGI draws no border.
pub fn border_denied() -> bool {
	BORDER_DENIED.load(Ordering::Relaxed)
}

/// Asks once per process for borderless capture, which the crate never does (Microsoft Learn: the border stays unless
/// the app asks `RequestAccessAsync(Borderless)`, which "displays a prompt"; what an unpackaged app sees is
/// unverified). The answer is logged whenever it comes, and a denial kept (`border_denied`); the first session waits
/// `ACCESS_WAIT` for it at most, so a prompt behind a fullscreen game never holds the capture (ELDEN RING's boss
/// banners' session asks the same, `banner.rs`).
pub(super) fn ask_borderless() {
	static ASKED: Once = Once::new();
	ASKED.call_once(|| {
		let name = HSTRING::from("Windows.Graphics.Capture.GraphicsCaptureAccess");
		if !ApiInformation::IsTypePresent(&name).unwrap_or(false) {
			log::info!("capture: no GraphicsCaptureAccess on this system");
			return;
		}
		let (answered, answer) = std::sync::mpsc::sync_channel(1);
		let asked = GraphicsCaptureAccess::RequestAccessAsync(GraphicsCaptureAccessKind::Borderless).and_then(|op| {
			op.when(move |status| {
				match status {
					Ok(status) => {
						log::info!("capture: borderless capture access {status:?}");
						let denied =
							[AppCapabilityAccessStatus::DeniedByUser, AppCapabilityAccessStatus::DeniedBySystem];
						BORDER_DENIED.store(denied.contains(&status), Ordering::Relaxed);
					}
					Err(e) => log::info!("capture: borderless capture access: {e}"),
				}
				let _ = answered.send(());
			})
		});
		if let Err(e) = asked {
			log::info!("capture: borderless capture access not asked: {e}");
		} else if answer.recv_timeout(ACCESS_WAIT).is_err() {
			log::info!("capture: no borderless answer yet; capturing meanwhile");
		}
	});
}

/// How long the first session waits for the borderless answer.
const ACCESS_WAIT: Duration = Duration::from_secs(5);

/// A running session on the game's window, and when its last frame arrived.
pub struct Session {
	control: CaptureControl<Strip, anyhow::Error>,
	heard: Heard,
}

impl Session {
	pub fn finished(&self) -> bool {
		self.control.is_finished()
	}

	/// When the last frame arrived: an error inside the crate's frame handler (a removed device) is dropped without
	/// ending the session (windows-capture 2.0.1 `graphics_capture_api.rs`, `FrameArrived`), so only the silence tells.
	pub fn heard(&self) -> &Heard {
		&self.heard
	}

	/// Ends the session and joins its thread; what ended it when it stopped on an error of its own.
	pub fn stop(self) -> Result<(), String> {
		self.control.stop().map_err(|e| e.to_string())
	}
}

/// Starts a session on the game's window; the crate returns once `StartCapture` succeeded on its thread, so every
/// setup error comes back here.
pub fn start(
	hwnd: HWND,
	probes: Probes,
	target: &Target,
	origin: Origin,
	events: &UnboundedSender<Event>,
) -> Result<Session, String> {
	ask_borderless();
	// three quarters of a step, so the callback's grid decides: a gate of a whole step since the last frame would let
	// 48 frames a second through on a 144 Hz display
	let interval = Duration::from_secs(1) * 3 / (4 * target.rate.max(1));
	let heard = Heard::new();
	let settings = Settings::new(
		Window::from_raw_hwnd(hwnd.0),
		if probes.cursor { CursorCaptureSettings::WithoutCursor } else { CursorCaptureSettings::Default },
		if probes.border { DrawBorderSettings::WithoutBorder } else { DrawBorderSettings::Default },
		SecondaryWindowSettings::Default,
		if probes.interval {
			MinimumUpdateIntervalSettings::Custom(interval)
		} else {
			MinimumUpdateIntervalSettings::Default
		},
		DirtyRegionSettings::Default,
		if target.hdr { ColorFormat::Rgba16F } else { ColorFormat::Bgra8 },
		Flags {
			events: events.clone(),
			origin,
			band: Band::new(target.hdr, target.white),
			rate: target.rate,
			heard: heard.clone(),
		},
	);
	let control = Strip::start_free_threaded(settings).map_err(|e| e.to_string())?;
	Ok(Session { control, heard })
}

pub struct Flags {
	events: UnboundedSender<Event>,
	origin: Origin,
	band: Band,
	rate: u32,
	heard: Heard,
}

/// The session's frame handler: throttle, crop, decode, send; it never blocks.
pub struct Strip {
	events: UnboundedSender<Event>,
	origin: Origin,
	band: Band,
	throttle: Throttle,
	heard: Heard,
}

impl GraphicsCaptureApiHandler for Strip {
	type Flags = Flags;
	type Error = anyhow::Error;

	fn new(ctx: Context<Self::Flags>) -> Result<Self, Self::Error> {
		let Flags { events, origin, band, rate, heard } = ctx.flags;
		Ok(Self { events, origin, band, throttle: Throttle::new(rate, clock::HUNDRED_NS), heard })
	}

	/// An error here ends the session (the loop then tries the next backend), so only a failed GPU readback returns
	/// one; a frame that cannot be read is skipped.
	fn on_frame_arrived(&mut self, frame: &mut Frame, _control: InternalCaptureControl) -> Result<(), Self::Error> {
		self.heard.now();
		let Ok(at) = frame.timestamp() else { return Ok(()) };
		// the game repaints its whole window at the display's refresh, and each crop is a GPU readback: skip first
		if !self.throttle.due(at.Duration) {
			return Ok(());
		}
		let format = match frame.color_format() {
			ColorFormat::Bgra8 => Format::Bgra8,
			ColorFormat::Rgba16F => Format::Rgba16F,
			ColorFormat::Rgba8 => return Ok(()), // never asked for
		};
		let Some((x, y)) = crop(self.origin.get(), (frame.width(), frame.height())) else { return Ok(()) };
		let (w, h) = BAND;
		let mut pixels = frame.buffer_crop(x, y, x + w, y + h)?;
		let pitch = usize::try_from(pixels.row_pitch())?;
		let read = self.band.read(format, pixels.as_raw_buffer(), pitch);
		send(&self.events, read, || clock::shown(at.Duration, clock::HUNDRED_NS));
		Ok(())
	}
}

/// Where the band's crop starts in a frame of `size`: the client area's top-left, when the band fits inside the frame
/// from there. `buffer_crop` takes end coordinates and checks no bounds, and a copy box outside the texture is
/// undefined (Microsoft Learn, `CopySubresourceRegion`), so a frame without room is skipped.
fn crop((x, y): (i32, i32), (width, height): (u32, u32)) -> Option<(u32, u32)> {
	let (x, y) = (u32::try_from(x).ok()?, u32::try_from(y).ok()?);
	let (w, h) = BAND;
	(x.checked_add(w)? <= width && y.checked_add(h)? <= height).then_some((x, y))
}

/// At most `rate` frames a second on a grid of 1/rate steps: a frame is due from an eighth of a step before the next
/// step (a variable-refresh display's jitter), and one a whole step late starts the grid again from itself. A display
/// whose refresh is no multiple of the rate (144 Hz at 60) keeps the full rate, where "a step since the last frame"
/// would give 48. ELDEN RING's boss banners keep their 4 a second on it (`banner.rs`).
pub(super) struct Throttle {
	step: i64,
	next: Option<i64>,
}

impl Throttle {
	pub(super) fn new(rate: u32, per_second: i64) -> Self {
		Self { step: per_second / i64::from(rate.max(1)), next: None }
	}

	pub(super) fn due(&mut self, t: i64) -> bool {
		match self.next {
			Some(next) if t < next - self.step / 8 => false,
			Some(next) if t < next + self.step => {
				self.next = Some(next + self.step);
				true
			}
			_ => {
				self.next = Some(t + self.step);
				true
			}
		}
	}
}
