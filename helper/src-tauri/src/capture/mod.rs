//! Reading the strip off the screen. The platform backend delivers `Event`s over a channel and never blocks in its
//! frame callback; a refused or revoked grant, a stopped stream or a missing display stops only the capture (an
//! `Error` each time; a stream that stopped with the grant held starts again), while the mirror and pairing keep
//! running (rust-helper `SKILL.md`, rule 5).

use std::time::Duration;

use questpump_core::protocol::ErrorCode;
use questpump_core::strip::{DecodeError, RawFrame, Read};

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::{Capture, permission_asked, request_permission, screen_granted};
#[cfg(windows)]
mod win;
#[cfg(windows)]
pub use win::{Capture, banner};

/// Windows needs no grant to read the game's window.
#[cfg(not(target_os = "macos"))]
pub fn request_permission(_data_dir: &std::path::Path) {}

/// Windows needs no grant to read the screen.
#[cfg(not(target_os = "macos"))]
pub fn screen_granted() -> bool {
	true
}

/// Windows needs no grant, so it never asked for one.
#[cfg(not(target_os = "macos"))]
pub fn permission_asked(_data_dir: &std::path::Path) -> bool {
	false
}

/// What the capture saw.
#[derive(Debug)]
pub enum Event {
	/// A decoded frame and when the window server showed it, in unix seconds.
	Frame(RawFrame, f64),
	/// No anchor in the corner: a movie, a cinematic, a hidden interface, the addon not loaded.
	NoStrip,
	/// The anchor is in the corner but the frame fails a check: a capture in the middle of a repaint now and then, and
	/// every frame when the strip is of another version or its colours do not read (the runtime says so once it lasts).
	Unreadable(DecodeError),
	/// The display the game is on, when the stream starts on it; `hdr` when it is an HDR output (Windows); `backend`
	/// the capture's, as the anonymous counts' `capture_day` names it (`sck`, `wgc`, `dxgi`, `gdi`).
	Display { name: String, scale: f64, hdr: bool, backend: &'static str },
	/// The capture stopped: for good on a missing grant; else it tries again (macOS by itself, Windows at the game's next
	/// start).
	Error(ErrorCode, String),
}

/// Frames a second at most, on power and on battery (docs/companion.md, "Countdown sync"), on both systems.
const RATE: u16 = 60;
const BATTERY_RATE: u16 = 10;
/// How often the capture thread looks at the game.
const TICK: Duration = Duration::from_secs(2);
/// How often the capture reads the power source again (on Windows the display's HDR state and SDR white too, on macOS
/// ScreenCaptureKit's content).
const LOOK: Duration = Duration::from_secs(30);

/// The capture's rate: the battery rate while the computer draws from its battery.
fn rate(on_battery: bool) -> u16 {
	if on_battery { BATTERY_RATE } else { RATE }
}

/// What a read of the band shows: the frame with when it was shown, the strip out of sight, or a strip that did not read.
fn event(read: Read, shown: impl FnOnce() -> f64) -> Event {
	match read {
		Read::Frame(frame) => Event::Frame(frame, shown()),
		Read::NoStrip => Event::NoStrip,
		Read::Rejected(why) => Event::Unreadable(why),
	}
}
