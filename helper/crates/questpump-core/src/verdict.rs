//! The heart-rate verdict (docs/plan.md slice 8, here from slice 5 so every source shares one rule): a pump counted
//! when the heart rate rose over the median of the minute before it, by 20 bpm for a dynamic move and 12 for an
//! isometric or mobility one, in two samples in a row within 60 s of the start. Push-ups and burpees are scored on the
//! 30 s after the set, because wrist optical heart rate drops out during them. The verdict only says whether a pump
//! counted: it never blocks, never changes XP and is never a safety signal. `contract/fixtures/verdict.json` holds
//! worked cases of the rule.
//!
//! The rise calibrates itself per source: a minute of ordinary play ten minutes before each pump is scored the same
//! way (a null window), and once `NULL_MIN` of them are in, a pump needs the nulls' 95th percentile, kept within a
//! band around the class's default.

use serde::{Deserialize, Serialize};

/// The rise over the baseline a pump needs, by the class of its move.
pub const DYNAMIC_BPM: u16 = 20;
pub const ISO_BPM: u16 = 12;
/// The window after the start in which the rise must show, in seconds.
pub const WINDOW_S: f64 = 60.0;
/// The baseline is the median of the samples in this many seconds before the start.
pub const BASELINE_S: f64 = 60.0;
/// A move scored after its set: the window is this long from the set's end.
pub const AFTER_SET_S: f64 = 30.0;
/// A null window starts this long before the pump's start (its baseline the minute before that).
pub const NULL_BEFORE_S: f64 = 600.0;
/// The null windows a source needs before its rise is calibrated, and the most it keeps (the newest).
pub const NULL_MIN: usize = 20;
pub const NULL_KEPT: usize = 200;
/// The band a calibrated rise stays in, by class: three quarters to one and a half times the default.
pub const DYNAMIC_BAND: (u16, u16) = (15, 30);
pub const ISO_BAND: (u16, u16) = (9, 18);
/// The heart rates a reading may carry, from any source: a strap without skin contact reports 0, the WHOOP 5.0's
/// broadcast bug reads 0 bpm on some receivers, and a watch window is never below or above these either.
pub const BPM_RANGE: std::ops::RangeInclusive<u16> = 25..=250;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Class {
	Dynamic,
	/// An isometric or mobility move.
	Iso,
}

impl Class {
	/// The rise a pump of this class needs before its source is calibrated.
	#[must_use]
	pub fn default_rise(self) -> u16 {
		match self {
			Self::Dynamic => DYNAMIC_BPM,
			Self::Iso => ISO_BPM,
		}
	}

	fn band(self) -> (u16, u16) {
		match self {
			Self::Dynamic => DYNAMIC_BAND,
			Self::Iso => ISO_BAND,
		}
	}

	/// The rise a pump of this class needs from a source whose null windows gave `nulls`: the default until
	/// `NULL_MIN` are in, then their 95th percentile (nearest rank) within the class's band.
	#[must_use]
	pub fn rise(self, nulls: &[i32]) -> u16 {
		if nulls.len() < NULL_MIN {
			return self.default_rise();
		}
		let mut sorted = nulls.to_vec();
		sorted.sort_unstable();
		let (low, high) = self.band();
		let p95 = crate::percentile(&sorted, 95).unwrap_or_default();
		u16::try_from(p95.clamp(i32::from(low), i32::from(high))).unwrap_or(low)
	}
}

/// A heart-rate sample: seconds from the pump's start (negative before it) and beats per minute.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct Sample {
	pub t: f64,
	pub bpm: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Input {
	pub class: Class,
	/// Whether a heart-rate source was paired (the watch, a strap on the helper): a stream missing from one is
	/// unproven, no source at all is "no HR" (phone-only without a strap).
	pub source: bool,
	/// When set, the move is scored on the 30 s after the set, which ended this many seconds after the start.
	pub set_end: Option<f64>,
	pub samples: Vec<Sample>,
	/// The rise this pump needs, when its source is calibrated; the class's default otherwise.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub rise: Option<u16>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
	/// No heart-rate source paired.
	NoHr,
	/// A source, but no rise, too few samples to tell, or a stream that dropped out.
	Unproven,
	/// The rise showed: a check mark.
	Proven,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Outcome {
	pub verdict: Verdict,
	/// The highest sample in the window.
	pub peak: Option<u16>,
	/// The peak over the baseline.
	pub delta: Option<i32>,
}

fn median(mut v: Vec<u16>) -> Option<u16> {
	if v.is_empty() {
		return None;
	}
	v.sort_unstable();
	let n = v.len();
	Some(if n % 2 == 1 { v[n / 2] } else { v[n / 2 - 1].midpoint(v[n / 2]) })
}

/// The median of the minute before `at` (seconds from the start).
#[must_use]
pub fn baseline(samples: &[Sample], at: f64) -> Option<u16> {
	median(samples.iter().filter(|s| s.t < at && s.t >= at - BASELINE_S).map(|s| s.bpm).collect())
}

/// The samples from `from` to `to`, in time order.
fn window(samples: &[Sample], from: f64, to: f64) -> Vec<Sample> {
	let mut window: Vec<Sample> = samples.iter().copied().filter(|s| s.t >= from && s.t <= to).collect();
	window.sort_by(|a, b| a.t.total_cmp(&b.t));
	window
}

/// The highest level two samples in a row reached: the rise a window held for the rule's two samples.
fn held(window: &[Sample]) -> Option<u16> {
	window.array_windows().map(|[a, b]| a.bpm.min(b.bpm)).max()
}

/// The verdict for one pump's samples, over the median of the minute before its start.
#[must_use]
pub fn verdict(input: &Input) -> Outcome {
	verdict_over(input, baseline(&input.samples, 0.0))
}

/// The verdict for a move over a baseline taken elsewhere: the helper measures every move of a pump against the
/// minute before the pump's first go.
#[must_use]
pub fn verdict_over(input: &Input, baseline: Option<u16>) -> Outcome {
	if !input.source {
		return Outcome { verdict: Verdict::NoHr, peak: None, delta: None };
	}
	let (from, to) = match input.set_end {
		Some(end) => (end, end + AFTER_SET_S),
		None => (0.0, WINDOW_S),
	};
	let window = window(&input.samples, from, to);
	let peak = window.iter().map(|s| s.bpm).max();
	let Some(base) = baseline else {
		return Outcome { verdict: Verdict::Unproven, peak, delta: None };
	};
	let need = base.saturating_add(input.rise.unwrap_or_else(|| input.class.default_rise()));
	let proven = held(&window).is_some_and(|level| level >= need);
	Outcome {
		verdict: if proven { Verdict::Proven } else { Verdict::Unproven },
		peak,
		delta: peak.map(|p| i32::from(p) - i32::from(base)),
	}
}

/// The rise a null window held: the minute that starts `NULL_BEFORE_S` before the pump's start, over the median of the
/// minute before it, the level two samples in a row reached less that median (negative when the heart rate fell).
/// None with fewer than three baseline samples or two window samples.
#[must_use]
pub fn null_rise(samples: &[Sample]) -> Option<i32> {
	let at = -NULL_BEFORE_S;
	let base_count = samples.iter().filter(|s| s.t < at && s.t >= at - BASELINE_S).count();
	if base_count < 3 {
		return None;
	}
	Some(i32::from(held(&window(samples, at, at + WINDOW_S))?) - i32::from(baseline(samples, at)?))
}
