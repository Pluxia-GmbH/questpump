//! The desktop app's anonymous counts (contract/telemetry-api.yaml, "The anonymous counts"; contract/telemetry.json
//! `counts`, built in as `telemetry_catalog`): the points, each checked against the catalog (a name, a blob or a double
//! it lacks, or a value outside a blob's list, is dropped, and a debug build stops there), the bodies posted to `ROUTE`
//! through the outbox (`OUTBOX`: kept at most `KEEP_DAYS` days from the batch's writing), the buckets the catalog's
//! lists name, what the counts keep between batches (`Counts`, `counts.json` in the data folder: the facts of each game
//! day not sent yet, never the regional default of the switch, never an id), and the rows of one log as
//! contract/feedback-api.yaml's counting rules read them, per game day (`tally`). Two sources go from the computer:
//! `desktop`, the computer's own pumps and the app's own facts, and `game`, the game's rows the app holds (the addon
//! sends nothing itself); a row another device made is that device's to count. No clock, file or platform call here:
//! the app reads the stores and the time and passes them in.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::codes::{flags, kind, outcome};
use crate::outbox::Rules;
use crate::row::Row;
use crate::telemetry_catalog::{MAX_POINTS, POINTS, PointSpec};

pub use crate::telemetry_catalog::COPY_VERSION;

/// The route the counts go to.
pub const ROUTE: &str = "https://questpump.com/api/counts";
/// The largest body the route reads, in bytes.
const BODY_MAX: usize = 16_384;
/// A batch the route has not taken stays in the outbox at most this many days (contract/telemetry-api.yaml), and the
/// counts keep a game day's facts no longer either.
pub const KEEP_DAYS: i64 = 7;
/// The most batches the outbox keeps: two sources a game day for a week, each split at most once, with room.
const KEEP_MAX: usize = 64;
/// The counts' outbox (`<data dir>/counts-outbox/`): 400, 401 and 413 drop a batch, since the route never takes it.
pub static OUTBOX: Rules = Rules {
	route: ROUTE,
	folder: "counts-outbox",
	keep_days: KEEP_DAYS,
	keep_max: KEEP_MAX,
	refused: &[400, 401, 413],
};
/// The longest blob value the route takes.
const BLOB_MAX: usize = 40;
/// The pumps in the review's window below which no `review_numbers` point goes.
pub const REVIEW_PUMPS: u32 = 5;
/// The days away from which a start is a return.
pub const RETURN_DAYS: i64 = 3;
/// The unix epoch as a Julian day.
const EPOCH_JULIAN: i64 = 2_440_588;

/// Who the counts are of: the computer's own, or the game's rows the computer holds.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Source {
	Desktop,
	Game,
}

impl Source {
	pub const ALL: [Self; 2] = [Self::Desktop, Self::Game];

	/// Its name in the body and the catalog.
	#[must_use]
	pub fn name(self) -> &'static str {
		match self {
			Self::Desktop => "desktop",
			Self::Game => "game",
		}
	}
}

/// One fact a point (contract/telemetry-api.yaml `Point`): a point of the catalog with each of its blobs, from its
/// closed list, and each of its doubles, within its bound.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Point {
	pub name: String,
	#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
	pub blobs: BTreeMap<String, String>,
	#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
	pub doubles: BTreeMap<String, f64>,
}

/// A point's spec in the catalog.
fn spec(name: &str) -> Option<&'static PointSpec> {
	POINTS.iter().find(|p| p.name == name)
}

/// A value in a blob's list; an open one (an empty list) is an ISO week for `cohort_week`, else a library move's key.
fn blob_ok(name: &str, list: &[&str], value: &str) -> bool {
	if !list.is_empty() {
		return list.contains(&value);
	}
	if name == "cohort_week" { is_iso_week(value) } else { is_key(value) }
}

/// An ISO week as the catalog writes one, `2026-W45`.
fn is_iso_week(value: &str) -> bool {
	let b = value.as_bytes();
	b.len() == 8 && b[..4].iter().all(u8::is_ascii_digit) && &b[4..6] == b"-W" && b[6..].iter().all(u8::is_ascii_digit)
}

/// A library move's key: lower-case words joined by underscores, at most `BLOB_MAX` characters.
fn is_key(value: &str) -> bool {
	(1..=BLOB_MAX).contains(&value.len())
		&& value.bytes().next().is_some_and(|b| b.is_ascii_lowercase())
		&& value.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}

/// Whether a blob's value is one the catalog lists for that point (or fits its open blob), with no assertion: for a
/// value that comes from outside the code (the page, a game's profile), which is dropped when it is not.
#[must_use]
pub fn listed(point: &str, blob: &str, value: &str) -> bool {
	spec(point)
		.and_then(|s| s.blobs.iter().find(|(name, _)| *name == blob))
		.is_some_and(|(_, list)| blob_ok(blob, list, value))
}

/// The points that count a feature used today, by its name in their `feature` list: `feature_day`, and
/// `fitness_feature_day` for one about a pump or health (fitness: a phone sends it only with the player's own yes).
const FEATURE_POINTS: [&str; 2] = ["feature_day", "fitness_feature_day"];

/// The point whose `feature` list holds `feature` (each of `FEATURE_POINTS` holds it once), none outside them.
#[must_use]
pub fn feature_point(feature: &str) -> Option<&'static str> {
	FEATURE_POINTS.into_iter().find(|point| listed(point, "feature", feature))
}

impl Point {
	/// A point of the catalog with exactly the blobs and doubles it names: none for a name, a blob or a double the
	/// catalog lacks, one of them missing, or a blob's value outside its list, which a debug build stops at (the code
	/// asked for what the catalog does not hold); a double is a whole count held to its bound.
	#[must_use]
	pub fn new(name: &str, blobs: &[(&str, &str)], doubles: &[(&str, f64)]) -> Option<Self> {
		let point = Self {
			name: name.to_owned(),
			blobs: blobs.iter().map(|(k, v)| ((*k).to_owned(), (*v).to_owned())).collect(),
			doubles: doubles.iter().map(|(k, v)| ((*k).to_owned(), *v)).collect(),
		};
		let checked = point.checked();
		debug_assert!(checked.is_some(), "telemetry: {point:?} is outside the catalog");
		checked
	}

	/// The point as the catalog holds it now (a point kept in `counts.json` by an older build is checked again): none
	/// when it breaks a rule `new` checks, each double held to its bound.
	fn checked(&self) -> Option<Self> {
		let spec = spec(&self.name)?;
		let blobs_ok = self.blobs.len() == spec.blobs.len()
			&& spec.blobs.iter().all(|(name, list)| self.blobs.get(*name).is_some_and(|v| blob_ok(name, list, v)));
		if !blobs_ok || self.doubles.len() != spec.doubles.len() {
			return None;
		}
		let mut doubles = BTreeMap::new();
		for (name, bound) in spec.doubles {
			let value = *self.doubles.get(*name)?;
			if !value.is_finite() {
				return None;
			}
			doubles.insert((*name).to_owned(), value.round().clamp(0.0, *bound));
		}
		Some(Self { name: self.name.clone(), blobs: self.blobs.clone(), doubles })
	}

	/// A count as a point's double.
	#[must_use]
	pub fn count(n: u32) -> f64 {
		f64::from(n)
	}
}

/// The body as the route reads it (contract/telemetry-api.yaml `Counts`), in its declaration order.
#[derive(Serialize)]
struct Body<'a> {
	source: Source,
	version: &'a str,
	day: &'a str,
	points: &'a [Point],
}

/// A game day's name as the body's `day` holds it, `2026-10-12`.
fn is_day(day: &str) -> bool {
	let b = day.as_bytes();
	b.len() == 10 && b.iter().enumerate().all(|(i, c)| if i == 4 || i == 7 { *c == b'-' } else { c.is_ascii_digit() })
}

/// The bodies of one source's points of one game day, as they are posted: at most `MAX_POINTS` points each, every one
/// checked against the catalog again (a point outside it, or one of a source the catalog does not list it for, is
/// dropped: a debug build stops there); none without a point, or for a version or a day the route refuses.
#[must_use]
pub fn bodies(source: Source, version: &str, day: &str, points: &[Point]) -> Vec<Vec<u8>> {
	if !crate::feedback::version(version) || !is_day(day) {
		debug_assert!(false, "telemetry: the version {version:?} or the day {day:?} breaks the route's pattern");
		return Vec::new();
	}
	let kept: Vec<Point> = points
		.iter()
		.filter_map(|p| {
			let checked = p.checked().filter(|_| spec(&p.name).is_some_and(|s| s.sources.contains(&source.name())));
			debug_assert!(checked.is_some(), "telemetry: {p:?} is no {} point of the catalog", source.name());
			checked
		})
		.collect();
	kept.chunks(MAX_POINTS)
		.filter_map(|points| {
			let bytes = serde_json::to_vec(&Body { source, version, day, points }).ok()?;
			(bytes.len() <= BODY_MAX).then_some(bytes)
		})
		.collect()
}

// The buckets ---------------------------------------------------------------------------------------------------------

/// A local date (days since 1970-01-01) as a `time::Date`.
fn date(day: i64) -> Option<time::Date> {
	time::Date::from_julian_day(i32::try_from(day + EPOCH_JULIAN).ok()?).ok()
}

/// A game day's name, `2026-10-12`, from its local date (days since 1970-01-01).
#[must_use]
pub fn day_name(day: i64) -> Option<String> {
	date(day).map(|d| format!("{:04}-{:02}-{:02}", d.year(), u8::from(d.month()), d.day()))
}

/// The ISO week a game day falls in, `2026-W45`.
#[must_use]
pub fn iso_week(day: i64) -> Option<String> {
	date(day).map(|d| {
		let (year, week, _) = d.to_iso_week_date();
		format!("{year:04}-W{week:02}")
	})
}

/// `active`'s `age`: the game days since the first run.
#[must_use]
pub fn age(days: i64) -> &'static str {
	match days {
		..=0 => "0",
		1 => "1",
		2 => "2",
		3 => "3",
		4 => "4",
		5 => "5",
		6 => "6",
		7 => "7",
		8..=13 => "8-13",
		14..=29 => "14-29",
		30..=59 => "30-59",
		60..=89 => "60-89",
		90..=179 => "90-179",
		_ => "180+",
	}
}

/// `first_pump`'s `delay`: the seconds from the first run to the first pump.
#[must_use]
pub fn delay(seconds: i64) -> &'static str {
	match seconds {
		..300 => "<5m",
		300..1_800 => "5-30m",
		1_800..7_200 => "30-120m",
		7_200..86_400 => "2-24h",
		86_400..604_800 => "1-7d",
		_ => ">7d",
	}
}

/// `return`'s `away`: the game days since the last active one; none below `RETURN_DAYS`.
#[must_use]
pub fn away(days: i64) -> Option<&'static str> {
	match days {
		..RETURN_DAYS => None,
		3..=6 => Some("3-6"),
		7..=13 => Some("7-13"),
		14..=29 => Some("14-29"),
		_ => Some("30+"),
	}
}

/// `review_numbers`' `done_ratio`: the pumps finished of those finished or banked, in percent; none without one.
#[must_use]
pub fn done_ratio(finished: u32, banked: u32) -> Option<&'static str> {
	let all = finished + banked;
	if all == 0 {
		return None;
	}
	Some(match finished * 100 / all {
		..50 => "<50",
		50..60 => "50-59",
		60..70 => "60-69",
		70..80 => "70-79",
		80..90 => "80-89",
		_ => "90-100",
	})
}

/// `review_numbers`' `max_sets`: the median of the max sets on one ladder on a play day, rounded up, so a median
/// above four (the review trigger, docs/game-design.md) never reads as four.
#[must_use]
pub fn max_sets(median: f64) -> &'static str {
	match median.ceil() {
		m if m <= 0.0 => "0",
		m if m <= 1.0 => "1",
		m if m <= 2.0 => "2",
		m if m <= 3.0 => "3",
		m if m <= 4.0 => "4",
		m if m <= 6.0 => "5-6",
		_ => "7+",
	}
}

/// `review_numbers`' `age_week`: the weeks since the first run.
#[must_use]
pub fn age_week(days: i64) -> &'static str {
	if days / 7 <= 12 { "0-12" } else { "13+" }
}

/// `setup_week`'s `equipment`: how much equipment the setup holds.
#[must_use]
pub fn equipment(count: usize) -> &'static str {
	match count {
		0 => "0",
		1..=2 => "1-2",
		3..=5 => "3-5",
		_ => "6+",
	}
}

/// `standing`'s `streak_best`: the best streak's game days as a band.
#[must_use]
pub fn streak_best(days: i64) -> &'static str {
	match days {
		..=0 => "0",
		1 => "1",
		2 => "2",
		3..=6 => "3-6",
		7..=13 => "7-13",
		14..=29 => "14-29",
		30..=59 => "30-59",
		60..=99 => "60-99",
		100..=179 => "100-179",
		180..=364 => "180-364",
		_ => "365+",
	}
}

/// A yes or no as a blob.
#[must_use]
pub fn yes(on: bool) -> &'static str {
	if on { "yes" } else { "no" }
}

// What the counts keep ------------------------------------------------------------------------------------------------

/// One value for each source.
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
#[serde(default)]
pub struct Both<T> {
	pub desktop: T,
	pub game: T,
}

impl<T> Both<T> {
	#[must_use]
	pub fn of(&self, source: Source) -> &T {
		match source {
			Source::Desktop => &self.desktop,
			Source::Game => &self.game,
		}
	}

	pub fn of_mut(&mut self, source: Source) -> &mut T {
		match source {
			Source::Desktop => &mut self.desktop,
			Source::Game => &mut self.game,
		}
	}
}

/// The capture on one game day the game ran: `capture_day`'s blobs.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct CaptureDay {
	pub os: String,
	pub backend: String,
	pub state: String,
}

/// The capture's states from the least to the most telling: the day keeps the most telling it saw (`reading` once
/// the strip read at all).
const CAPTURE_ORDER: [&str; 5] = ["no_strip", "error", "unreadable", "denied", "reading"];

/// What one source did on one game day, as it happened (the rows' counts come from the stores at the batch).
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
#[serde(default)]
pub struct Facts {
	/// In use: the desktop app's window, a pump, the game read or a device served (`desktop`); the game read or its
	/// rows (`game`).
	pub active: bool,
	/// The install's first game day.
	pub install: bool,
	/// The run before a start of this day crashed (the crash sweep found its report).
	pub crashed: bool,
	/// `return`'s days away, when the day's first activity came after `RETURN_DAYS` or more.
	pub away: Option<i64>,
	/// `link_day`'s state: what the link did that day.
	pub link: Option<String>,
	/// `link_ble`'s blobs (slice 43): the Bluetooth link on a game day a phone was paired.
	pub link_ble: Option<LinkBle>,
	pub capture: Option<CaptureDay>,
	/// The features used, `feature_day`'s and `fitness_feature_day`'s (`feature_point`).
	pub features: BTreeSet<String>,
	/// `tree_day`'s counts (the desktop app's talent trees, slice 38).
	pub tree: TreeDay,
	/// The points of things that happened (`pairing`, `game_offer_desk`), as they came.
	pub events: Vec<Point>,
	/// Unfinished moves skipped that day.
	pub skipped: u32,
	/// The moves owed at the day's end, as last seen that day.
	pub owed: Option<u32>,
}

impl Facts {
	/// The capture as seen now: the day keeps the most telling state (`CAPTURE_ORDER`), with the backend of that
	/// sighting.
	pub fn capture(&mut self, os: &str, backend: &str, state: &str) {
		let rank = |s: &str| CAPTURE_ORDER.iter().position(|c| *c == s);
		let Some(now) = rank(state) else { return };
		if self.capture.as_ref().is_some_and(|c| rank(&c.state).is_some_and(|before| before > now)) {
			return;
		}
		self.capture = Some(CaptureDay { os: os.to_owned(), backend: backend.to_owned(), state: state.to_owned() });
	}

	/// The link as seen now: `ok` once a device linked, else the latest state.
	pub fn link(&mut self, state: &str) {
		if self.link.as_deref() != Some("ok") {
			self.link = Some(state.to_owned());
		}
	}
}

/// `link_ble`'s facts of one game day a phone was paired (slice 43; contract/telemetry-api.yaml): the computer's
/// Bluetooth link as the helper last said it that day, and the transports a phone's or a watch's sync came on. The
/// desktop app's `link` is `linked` once a sync came over Bluetooth that day, else `unlinked`; never a time or a count
/// of what went over it.
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
#[serde(default)]
pub struct LinkBle {
	/// `Helper.ble`'s word as last said that day; none: `unknown`.
	pub ble: Option<String>,
	/// A sync came on the LAN that day.
	pub lan: bool,
	/// A sync came over Bluetooth that day.
	pub over_ble: bool,
}

impl LinkBle {
	/// The day's `link_ble` point.
	#[must_use]
	pub fn point(&self) -> Option<Point> {
		let transport = match (self.lan, self.over_ble) {
			(true, true) => "both",
			(true, false) => "lan",
			(false, true) => "ble",
			(false, false) => "none",
		};
		let link = if self.over_ble { "linked" } else { "unlinked" };
		let ble = self.ble.as_deref().unwrap_or("unknown");
		Point::new("link_ble", &[("ble", ble), ("link", link), ("transport", transport)], &[])
	}
}

/// `tree_day`'s counts of one game day (slice 36; contract/telemetry-api.yaml): the step cards opened, each Start made
/// from a step's card or a move's window by its kind (Progress.lua `ns.StartFor`'s `as`), and each card opened whose
/// Start was refused or greyed by its reason; a refusal for a pain flag or felt unwell is counted under no name.
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
#[serde(default)]
pub struct TreeDay {
	pub cards: u32,
	pub starts: BTreeMap<String, u32>,
	pub refused: BTreeMap<String, u32>,
}

impl TreeDay {
	/// The kinds a Start counts under (`starts_<kind>`; `path` the climb's turn for its path and `check` a stretch check
	/// since slice 42, `proof` the path's kind until then; `haul` a Long Haul since slice 55).
	pub const STARTS: [&str; 8] = ["own", "held", "climb", "path", "earn", "place", "check", "haul"];
	/// The reasons a refused or greyed Start counts under (`refused_<reason>`).
	pub const REFUSALS: [&str; 5] = ["setup", "off", "game", "reading", "busy"];

	/// A step's card opened, with its Start's refusal or greyed reason where it has one of `REFUSALS`.
	pub fn card(&mut self, refused: Option<&str>) {
		self.cards = self.cards.saturating_add(1);
		if let Some(reason) = refused.filter(|r| Self::REFUSALS.contains(r)) {
			let n = self.refused.entry(reason.to_owned()).or_default();
			*n = n.saturating_add(1);
		}
	}

	/// A Start of `kind`; false, nothing counted, for a kind outside `STARTS` (a newer engine's).
	pub fn start(&mut self, kind: &str) -> bool {
		if !Self::STARTS.contains(&kind) {
			return false;
		}
		let n = self.starts.entry(kind.to_owned()).or_default();
		*n = n.saturating_add(1);
		true
	}

	/// The day's `tree_day` point, every double named (a count the day had none of is 0); none on a day with no card
	/// and no Start.
	#[must_use]
	pub fn point(&self) -> Option<Point> {
		if self.cards == 0 && self.starts.is_empty() {
			return None;
		}
		let of = |list: &BTreeMap<String, u32>, key: &str| Point::count(list.get(key).copied().unwrap_or(0));
		let mut doubles = vec![("cards".to_owned(), Point::count(self.cards))];
		doubles.extend(Self::STARTS.iter().map(|k| (format!("starts_{k}"), of(&self.starts, k))));
		doubles.extend(Self::REFUSALS.iter().map(|k| (format!("refused_{k}"), of(&self.refused, k))));
		let named: Vec<(&str, f64)> = doubles.iter().map(|(k, v)| (k.as_str(), *v)).collect();
		Point::new("tree_day", &[], &named)
	}
}

/// One game day's facts.
pub type Day = Both<Facts>;

/// How far the counts have looked at the stores' rows (`Counts::seen`): the game's store and the app's rows.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, PartialEq, Eq)]
#[serde(default)]
pub struct Seen {
	pub game: usize,
	pub app: usize,
}

/// What the counts keep on the computer between batches (`counts.json`): the app's first run and the game's first
/// sight (unix seconds: a cohort's week and an age, never sent as a time), whether each source's install waits to be
/// counted (one that came while the switch read off counts on the first day it reads on), the last game day a batch
/// went for, each source's last active day, the ISO week its weekly points last went, the sources whose first pump was
/// counted, how far the stores' rows were looked at, and the facts of each game day not sent yet. The facts a later
/// point reads (the first run, the game's first sight, the installs due, the last active days) are kept whatever Usage
/// statistics says, and only the points wait for it (contract/telemetry-api.yaml); the days' facts are kept only while
/// it reads on: turning it off empties them (and the outbox), and a day counted while it was off never goes.
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
#[serde(default)]
pub struct Counts {
	pub first_run: Option<i64>,
	pub game_first: Option<i64>,
	pub install_due: Both<bool>,
	pub sent_through: Option<i64>,
	pub last_active: Both<Option<i64>>,
	pub week_sent: Both<Option<String>>,
	pub first_pump: Both<bool>,
	pub seen: Seen,
	pub days: BTreeMap<i64, Day>,
}

impl Counts {
	pub const FILE: &str = "counts.json";

	/// One source's facts of a game day, made when missing.
	pub fn facts(&mut self, source: Source, day: i64) -> &mut Facts {
		self.days.entry(day).or_default().of_mut(source)
	}

	/// A source in use on game day `day`: true for its first activity that day, and then a return after
	/// `RETURN_DAYS` or more away is noted.
	pub fn active(&mut self, source: Source, day: i64) -> bool {
		if self.days.get(&day).is_some_and(|d| d.of(source).active) {
			return false;
		}
		let last = *self.last_active.of(source);
		let facts = self.facts(source, day);
		facts.active = true;
		if let Some(last) = last.filter(|l| *l < day) {
			facts.away = away(day - last).map(|_| day - last);
		}
		self.used(source, day);
		true
	}

	/// A source used on game day `day`: its last active day moves on (kept whatever the switch says, for a later
	/// `return`). True when it moved.
	pub fn used(&mut self, source: Source, day: i64) -> bool {
		let last = self.last_active.of_mut(source);
		if last.is_some_and(|l| l >= day) {
			return false;
		}
		*last = Some(day);
		true
	}

	/// The game days before `today` whose facts wait, oldest first; one older than `KEEP_DAYS` is dropped, and so is
	/// one at or before the last day a batch went for.
	pub fn finished(&mut self, today: i64) -> Vec<(i64, Day)> {
		let sent = self.sent_through;
		self.days.retain(|day, _| today - day <= KEEP_DAYS && sent.is_none_or(|s| *day > s));
		let done: Vec<i64> = self.days.keys().copied().filter(|d| *d < today).collect();
		done.into_iter().filter_map(|d| self.days.remove(&d).map(|facts| (d, facts))).collect()
	}

	/// Usage statistics turned off: every day's facts go, so a day counted while it was off never goes.
	pub fn clear(&mut self) {
		self.days.clear();
	}
}

// The rows ------------------------------------------------------------------------------------------------------------

/// What one log's rows hold of one game day by contract/feedback-api.yaml's counting rules: the pumps finished and
/// banked that were offered that day, each move's easier picks and early stops (by the move it is now,
/// `moves::current`), and each strength or hold ladder's max sets.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct DayTally {
	pub finished: u32,
	pub banked: u32,
	pub moves: BTreeMap<u16, (u32, u32)>,
	pub sets: BTreeMap<u8, u32>,
}

/// One log's rows by time, rows of one time in their given order.
fn ordered(rows: &[Row]) -> Vec<&Row> {
	let mut ordered: Vec<(usize, &Row)> = rows.iter().enumerate().collect();
	ordered.sort_by_key(|&(i, r)| (r.ts, i));
	ordered.into_iter().map(|(_, r)| r).collect()
}

/// A pump of a log: its offered row's time, whether it was banked and whether one of its own moves was done.
#[derive(Debug, Clone, Copy)]
struct Span {
	at: i64,
	banked: bool,
	done: bool,
}

impl Span {
	/// Finished: not banked, and one of its own moves done.
	fn finished(self) -> bool {
		!self.banked && self.done
	}
}

/// The pumps of a log in time order: each offered pump row's span (the rows up to the next offered pump row) is banked
/// when it holds a pump row banked for a reason other than feeling unwell with its own moves put off (`reps` above
/// 0), and done when one of its own moves (no owed bit) was performed.
fn spans(rows: &[&Row]) -> Vec<Span> {
	let mut out: Vec<Span> = Vec::new();
	for r in rows {
		if kind::is_pump(r.kind) && r.outcome == outcome::OFFERED {
			out.push(Span { at: i64::from(r.ts), banked: false, done: false });
		} else if let Some(span) = out.last_mut() {
			if kind::is_pump(r.kind) && r.outcome == outcome::BANKED && r.reason() != flags::UNWELL && r.reps > 0 {
				span.banked = true;
			} else if r.performed() && r.flags & flags::OWED == 0 {
				span.done = true;
			}
		}
	}
	out
}

/// One log's rows (in any order; rows of one time keep theirs) per game day, `day_of` naming a time's game day and
/// `strength` the strength and hold ladders (library.json `strengthOrder`): its pumps (`spans`) banked or finished by
/// the day they were offered; a performed move row's easier pick (flag bit 6) and early stop (outcome stopped); a
/// performed rep or hold row of a strength ladder, owed or not, one of that ladder's max sets that day, on the ladder
/// it names.
#[must_use]
pub fn tally(rows: &[Row], day_of: impl Fn(i64) -> i64, strength: &[u8]) -> BTreeMap<i64, DayTally> {
	let rows = ordered(rows);
	let mut days: BTreeMap<i64, DayTally> = BTreeMap::new();
	for span in spans(&rows) {
		let t = days.entry(day_of(span.at)).or_default();
		if span.banked {
			t.banked += 1;
		} else if span.done {
			t.finished += 1;
		}
	}
	for r in rows.into_iter().filter(|r| r.performed()) {
		let t = days.entry(day_of(i64::from(r.ts))).or_default();
		let (easier, stopped) = (r.flags & flags::EASIER != 0, r.outcome == outcome::STOPPED);
		if easier || stopped {
			let m = t.moves.entry(crate::moves::current(r.move_id)).or_default();
			m.0 += u32::from(easier);
			m.1 += u32::from(stopped);
		}
		if (r.kind == kind::REP || r.kind == kind::HOLD) && strength.contains(&r.ladder) {
			*t.sets.entry(r.ladder).or_default() += 1;
		}
	}
	days
}

/// The review's numbers of the last game days to today (library.json's `feedback_days`, contract/feedback-api.yaml's
/// window; a day dated ahead counts as today's): the pumps finished and banked, and the median of each ladder's max
/// sets on a day it was trained (the mean of the two middle ones for an even count; 0 with none).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Review {
	pub finished: u32,
	pub banked: u32,
	pub max_sets: f64,
}

impl Review {
	/// The review of tallied days over the `window` game days to `today`.
	#[must_use]
	pub fn of(days: &BTreeMap<i64, DayTally>, today: i64, window: i64) -> Self {
		let (mut finished, mut banked, mut sets) = (0, 0, Vec::new());
		for (_, t) in days.range(today - window + 1..) {
			finished += t.finished;
			banked += t.banked;
			sets.extend(t.sets.values().copied());
		}
		sets.sort_unstable();
		let max_sets = match sets.len() {
			0 => 0.0,
			n if n % 2 == 1 => f64::from(sets[n / 2]),
			n => f64::midpoint(f64::from(sets[n / 2 - 1]), f64::from(sets[n / 2])),
		};
		Self { finished, banked, max_sets }
	}

	/// The `review_numbers` point, `days_since_first` the game days since the source's first run; none below
	/// `REVIEW_PUMPS` pumps.
	#[must_use]
	pub fn point(&self, days_since_first: i64) -> Option<Point> {
		if self.finished + self.banked < REVIEW_PUMPS {
			return None;
		}
		let ratio = done_ratio(self.finished, self.banked)?;
		Point::new(
			"review_numbers",
			&[("done_ratio", ratio), ("max_sets", max_sets(self.max_sets)), ("age_week", age_week(days_since_first))],
			&[],
		)
	}
}

/// The first finished pump in one log's rows: its offered row's time.
#[must_use]
pub fn first_pump(rows: &[Row]) -> Option<i64> {
	spans(&ordered(rows)).into_iter().find(|s| s.finished()).map(|s| s.at)
}
