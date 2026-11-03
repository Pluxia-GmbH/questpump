//! Distances and speeds in the player's road units, for the walking pad's line in the window (docs/plan.md slice 25):
//! miles where the player's settings count roads in miles, kilometres elsewhere. On macOS the measurement system the
//! player chose in Language & Region decides (it follows the region unless they changed it, as the iPhone app's
//! `Format.distance` follows it), with yards for short distances in the UK region; on Windows the region decides (the
//! United States, Liberia and Myanmar in miles and feet, the United Kingdom in miles and yards: CLDR's road
//! preference, which the Android app reads through ICU, `shared` `Distance`). The decimals are written with a full
//! stop: the window is English.

const MILE: f64 = 1609.344;
const FOOT: f64 = 0.3048;
const YARD: f64 = 0.9144;

/// A player's road units, read once for a line so its speed and its distance agree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Road {
	Metric,
	Us,
	Uk,
}

impl Road {
	/// The player's road units now: a change in the settings shows at the next line.
	#[must_use]
	pub fn now() -> Self {
		read()
	}

	/// A distance to three significant digits, so half a mile never rounds up to one: the short unit below a tenth of
	/// a mile (or a kilometre), the long one from there.
	#[must_use]
	pub fn distance(self, metres: f64) -> String {
		let (value, unit) = match self {
			Self::Metric if metres < 1000.0 => (metres, "m"),
			Self::Metric => (metres / 1000.0, "km"),
			Self::Us if metres < MILE / 10.0 => (metres / FOOT, "ft"),
			Self::Uk if metres < MILE / 10.0 => (metres / YARD, "yd"),
			Self::Us | Self::Uk => (metres / MILE, "mi"),
		};
		format!("{} {unit}", significant(value))
	}

	/// A belt's speed to a tenth: km/h, or mph where the roads count in miles.
	#[must_use]
	pub fn speed(self, kmh: f64) -> String {
		match self {
			Self::Metric => format!("{kmh:.1} km/h"),
			Self::Us | Self::Uk => format!("{:.1} mph", kmh * 1000.0 / MILE),
		}
	}
}

#[cfg(target_os = "macos")]
fn read() -> Road {
	// a pool of its own: a Foundation call may autorelease, and the loop's thread never drains a pool
	let metric =
		objc2::rc::autoreleasepool(|_| objc2_foundation::NSLocale::autoupdatingCurrentLocale().usesMetricSystem());
	if metric {
		return Road::Metric;
	}
	match region().as_deref() {
		Some("GB") => Road::Uk,
		_ => Road::Us,
	}
}

/// The region the system's settings name (Language & Region's, `NSLocale`'s `regionCode`), none when they name none:
/// the road units read it, and the crash reports' opt-in regions (`crash::opt_in`).
#[cfg(target_os = "macos")]
pub fn region() -> Option<String> {
	// a pool of its own, as above
	objc2::rc::autoreleasepool(|_| {
		objc2_foundation::NSLocale::autoupdatingCurrentLocale().regionCode().map(|code| code.to_string())
	})
}

#[cfg(windows)]
fn read() -> Road {
	match region().as_deref() {
		Some("US" | "LR" | "MM") => Road::Us,
		Some("GB") => Road::Uk,
		_ => Road::Metric,
	}
}

/// The region the system's settings name (ISO 3166 alpha-2, or a UN M.49 number where the region has no code), none
/// when they name none: the road units read it, and the crash reports' opt-in regions (`crash::opt_in`).
#[cfg(windows)]
pub fn region() -> Option<String> {
	let mut name = [0u16; 16];
	// SAFETY: the buffer is the helper's own and its length goes with it (`GetUserDefaultGeoName`, Windows 10 1709 and
	// later); the count it returns includes the closing null, and 0 means it failed
	let len = unsafe { windows::Win32::Globalization::GetUserDefaultGeoName(&mut name) };
	let len = usize::try_from(len).ok()?.checked_sub(1)?;
	String::from_utf16(name.get(..len)?).ok()
}

#[cfg(not(any(target_os = "macos", windows)))]
fn read() -> Road {
	Road::Metric
}

/// A number to three significant digits, without the zeros that end a fraction (`1.20` is `1.2`); 0 for nothing or a
/// value that is no number.
fn significant(value: f64) -> String {
	if !value.is_finite() || value <= 0.0 {
		return "0".to_owned();
	}
	#[expect(clippy::cast_possible_truncation, reason = "the floor of a finite distance's log10, a few units")]
	let magnitude = value.log10().floor() as i32;
	let decimals = usize::try_from(2 - magnitude).unwrap_or(0);
	let text = format!("{value:.decimals$}");
	if text.contains('.') { text.trim_end_matches('0').trim_end_matches('.').to_owned() } else { text }
}
