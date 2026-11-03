//! The places (docs/plan.md slice 51, "Places"; `contract/places.json`, built in as `library.json` is): the limits, the
//! empty place, the glyphs and the templates every surface seeds its list from, Home first. The list a device keeps and
//! the body's `home` and `places` fields that carry it are `body::PlaceSetup` and `body::Place`, its plan (the default
//! place and the weekly schedule, 2026-11-03) `body::PlacePlan`, and the place in use at a moment `in_use`.

use std::sync::LazyLock;

use serde::{Deserialize, Serialize};

use crate::body::{Place, PlacePlan, PlaceSetup, PlaceTime};
use crate::session::Zone;

/// `contract/places.json`.
#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Places {
	/// The most places a list holds, Home in it (12).
	pub places_max: usize,
	/// The longest name a player gives a place, in Unicode scalars once trimmed (24).
	pub place_name_max: usize,
	/// The most times the weekly schedule holds (21).
	pub schedule_max: usize,
	/// Home's id (`home`).
	pub home: String,
	/// What an empty place starts with (Add a place, Empty).
	pub empty: EmptyPlace,
	/// The glyphs a place may take, pieces of `contract/design/art`.
	pub glyphs: Vec<String>,
	/// The templates in their order, Home first.
	pub templates: Vec<Template>,
}

/// An empty place's glyph, equipment and switches.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct EmptyPlace {
	pub glyph: String,
	pub gear: Vec<String>,
	pub floor: bool,
	pub space: bool,
	pub quiet: bool,
}

/// A template: its id, its words' key (`places.template.<id>`), its glyph, its equipment and switches (none on Home,
/// which takes the device's own setup).
#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Template {
	pub id: String,
	pub words: String,
	pub glyph: String,
	pub gear: Option<Vec<String>>,
	pub floor: Option<bool>,
	pub space: Option<bool>,
	pub quiet: Option<bool>,
}

static PLACES: LazyLock<Option<Places>> =
	LazyLock::new(|| serde_json::from_str::<Places>(include_str!("../../../../contract/places.json")).ok());

/// The places as built in; none when they do not read (`make check`'s `copytext.py --check` keeps them readable).
#[must_use]
pub fn places() -> Option<&'static Places> {
	PLACES.as_ref()
}

impl Template {
	/// The template as a place the seeding made: its id, the template it came from, no name of its own (its words), its
	/// glyph, equipment and switches (the addon's defaults where it names none).
	#[must_use]
	pub fn place(&self) -> Place {
		let defaults = PlaceSetup::default();
		Place {
			id: self.id.clone(),
			template: Some(self.id.clone()),
			name: None,
			glyph: self.glyph.clone(),
			setup: PlaceSetup {
				gear: self.gear.clone().unwrap_or_default(),
				floor: self.floor.unwrap_or(defaults.floor),
				space: self.space.unwrap_or(defaults.space),
				quiet: self.quiet.unwrap_or(defaults.quiet),
			},
		}
	}
}

/// The list a device that holds no places starts with, Home aside (it is the device's own setup): every other template
/// in its order, as a place the seeding made. Empty when the places do not read.
#[must_use]
pub fn seeded() -> Vec<Place> {
	places().map_or_else(Vec::new, |p| p.templates.iter().filter(|t| t.id != p.home).map(Template::place).collect())
}

/// The days a time's span may have started on and still reach `t`'s date: a span runs less than a day.
const SPAN_DAYS: i64 = 1;
/// The days back and on a time's boundaries are looked for: each comes once a week.
const WEEK_DAYS: i64 = 7;

/// A local date's ISO weekday, 1 Monday to 7 Sunday (1970-01-01, date 0, was a Thursday).
fn iso_weekday(date: i64) -> u8 {
	u8::try_from((date + 3).rem_euclid(7) + 1).unwrap_or(1)
}

/// Each span of `time` that starts on a date from `first` to `last`: its start and its end (unix seconds), a `to` before
/// `from` ending on the next date.
fn spans(time: &PlaceTime, first: i64, last: i64, zone: &Zone) -> impl Iterator<Item = (i64, i64)> {
	(first..=last).filter(|&date| time.days.contains(&iso_weekday(date))).map(move |date| {
		let end_date = if time.to > time.from { date } else { date + 1 };
		(zone.at_minute(date, i64::from(time.from)), zone.at_minute(end_date, i64::from(time.to)))
	})
}

/// The schedule's latest start or end at or before `now` (unix seconds); none with no time.
#[must_use]
pub fn last_change(plan: &PlacePlan, now: i64, zone: &Zone) -> Option<i64> {
	let today = zone.date(now);
	plan.times
		.iter()
		.flat_map(|time| spans(time, today - WEEK_DAYS - SPAN_DAYS, today, zone))
		.flat_map(|(start, end)| [start, end])
		.filter(|&at| at <= now)
		.max()
}

/// The place in use at `now` (unix seconds; `contract/places.json`'s `about`, the place in use at a moment): the place
/// the player last picked on this device (`pick`: its id and when, unix seconds), while that pick was made at or after
/// the schedule's latest start or end and the plan's last change (`plan_at`, unix seconds: a new default or time starts
/// from the plan); else the place of the time running now (of two that overlap, the one that
/// started later, then the later in the list); else the plan's default; a place named there that `listed` does not
/// hold falls through to the next rule, and Home is the last.
#[must_use]
pub fn in_use<'a>(
	plan: &'a PlacePlan,
	plan_at: Option<i64>,
	pick: Option<(&'a str, i64)>,
	listed: impl Fn(&str) -> bool,
	now: i64,
	zone: &Zone,
) -> &'a str {
	let change = last_change(plan, now, zone).max(plan_at);
	let held = pick.filter(|&(id, at)| listed(id) && change.is_none_or(|change| at >= change));
	if let Some((id, _)) = held {
		return id;
	}
	let today = zone.date(now);
	let running = plan
		.times
		.iter()
		.enumerate()
		.filter(|(_, time)| listed(&time.place))
		.filter_map(|(i, time)| {
			let start = spans(time, today - SPAN_DAYS, today, zone)
				.filter(|&(start, end)| start <= now && now < end)
				.map(|(start, _)| start)
				.max()?;
			Some((start, i, time.place.as_str()))
		})
		.max();
	if let Some((_, _, id)) = running {
		return id;
	}
	if listed(&plan.default) {
		return &plan.default;
	}
	places().map_or("home", |p| p.home.as_str())
}
