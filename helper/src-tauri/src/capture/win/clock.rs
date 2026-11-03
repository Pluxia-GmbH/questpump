//! Frame times on the wall clock. Windows.Graphics.Capture stamps a frame with `SystemRelativeTime`, the QPC time the
//! compositor rendered it in 100-ns units, and DXGI with `LastPresentTime`, a `QueryPerformanceCounter` value
//! (Microsoft Learn, `Direct3D11CaptureFrame.SystemRelativeTime` and `DXGI_OUTDUPL_FRAME_INFO`); both become unix
//! seconds as now minus the frame's age on the QPC clock, as `macos.rs` does with `displayTime`.

use std::time::{SystemTime, UNIX_EPOCH};

use windows::Win32::System::Performance::{QueryPerformanceCounter, QueryPerformanceFrequency};

/// Ticks a second of a `TimeSpan` (100-ns units): Windows.Graphics.Capture's frame times.
pub const HUNDRED_NS: i64 = 10_000_000;

/// Now, in unix seconds.
pub fn unix_now() -> f64 {
	SystemTime::now().duration_since(UNIX_EPOCH).map_or(0.0, |d| d.as_secs_f64())
}

/// The QPC counter and its ticks a second.
pub fn qpc() -> Option<(i64, i64)> {
	let (mut now, mut per_second) = (0i64, 0i64);
	// SAFETY: each call writes one i64 through a pointer to a local that lives for the call.
	let read = unsafe {
		QueryPerformanceCounter(&raw mut now).is_ok() && QueryPerformanceFrequency(&raw mut per_second).is_ok()
	};
	read.then_some((now, per_second))
}

/// When a frame stamped `ticks` on the QPC clock, counted `per_second` ticks a second, was shown, in unix seconds.
pub fn shown(ticks: i64, per_second: i64) -> f64 {
	let unix = unix_now();
	qpc().map_or(unix, |(now, qpc_per_second)| unix - age(now, qpc_per_second, ticks, per_second))
}

/// A frame's age in seconds: the counter `now` at `now_per_second` minus the frame's `ticks` at `per_second`, never
/// below zero.
fn age(now: i64, now_per_second: i64, ticks: i64, per_second: i64) -> f64 {
	if now_per_second <= 0 || per_second <= 0 {
		return 0.0;
	}
	let ns = |t: i64, rate: i64| i128::from(t) * 1_000_000_000 / i128::from(rate);
	let age = (ns(now, now_per_second) - ns(ticks, per_second)).max(0);
	#[expect(clippy::cast_precision_loss, reason = "a frame's age in nanoseconds is far below 2^52")]
	let age = age as f64 / 1e9;
	age
}
