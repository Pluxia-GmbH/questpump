//! One body everywhere (docs/plan.md slice 35, "One body on every app device", and since 2026-10-09 the game too;
//! `contract/companion-api.yaml`, `POST /v1/body` and `Body`): the free mode's engine profile's Body switches, its
//! experience, its moves (the exclusions and the opt-in moves), its favourites, its chosen title and since slice 48 its
//! Moves per match (the desktop app's Games page, the phones' Your setup) as the helper and every paired phone share
//! them, each field with when it last changed, so the latest change wins field by field and a tie goes to the helper's.
//! A field nobody changed (a default, a restored profile) is absent and never travels. The game's Body switches and its
//! Exercises page's list (`GameBody`), which the strip's profile and the files taken in show, join by the time the game
//! changed each (its `bodyAt`), or, for a field from before those times, as a change whenever the game's value differs
//! from the last one the helper saw in the same profile; the mirror carries the switches and the moves back
//! (`mirror::State::body`), and the game takes each one that changed later than its own change of it at its next load
//! (DB.lua `ns.TakeBody`). Since slice 51 (2026-10-21, `contract/places.json`) the places travel too: Home's
//! equipment and switches as `home` between the game, the computer and the phones (the game's one setup is Home: the
//! profile's `gear`, `floor`, `space` and `quiet`, joining at its `bodyAt.home`, and the mirror carries it back), the
//! other places as `places` between the phones and the computer only, and since 2026-11-03 their plan (the default
//! place and the weekly schedule) as `place_plan` beside them; the place in use stays each device's own and is never
//! here (until slice 51 the whole place did). The pain flags and the focus are rows, so they are not here either.
//! The helper keeps it in `body.json`.

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::LazyLock;

use ciborium::Value;
use serde::{Deserialize, Serialize};

use crate::{blob, library, link, places};

/// The experience a profile holds before the question is answered: DB.lua's default and the phones', the lowest start
/// rung.
pub const DEFAULT_EXPERIENCE: &str = "none";

/// Whether `word` answers the experience question: a key of library.json's `start_rung` (DB.lua's `experience`,
/// contract/sheet.json's `experience` words), none when the built-in constants do not read.
#[must_use]
pub fn is_experience(word: &str) -> bool {
	library::constants().is_some_and(|c| c.start_rung.contains_key(word))
}

/// Moves per match while no device chose it (slice 48): Balanced, the pace of every other app pump.
pub const DEFAULT_PACE: &str = "balanced";

/// Whether `word` is a pace of Moves per match: a key of library.json's `pace_moves` (Constants.lua's, the paces of
/// Moves per quest: light, balanced, extreme), none when the built-in constants do not read.
#[must_use]
pub fn is_pace(word: &str) -> bool {
	library::constants().is_some_and(|c| c.pace_moves.contains_key(word))
}

/// Whether every size of a pump takes the same moves at the pace `pace` (library.json's `pace_moves`: Light's one move
/// at Light, Balanced and Extreme), so a match's pump at it offers no size to choose (slice 48's double-check); false
/// for a pace the built-in constants do not know.
#[must_use]
pub fn sizes_alike(pace: &str) -> bool {
	library::constants()
		.and_then(|c| c.pace_moves.get(pace))
		.is_some_and(|moves| moves.windows(2).all(|pair| pair[0] == pair[1]))
}

/// The experience question's answers in the question's order, the lowest start rung first (library.json's
/// `start_rung`); none when the built-in constants do not read.
#[must_use]
pub fn experiences() -> Vec<&'static str> {
	let mut words: Vec<(&'static str, u8)> = library::constants()
		.map(|c| c.start_rung.iter().map(|(word, rung)| (word.as_str(), *rung)).collect())
		.unwrap_or_default();
	words.sort_by_key(|&(word, rung)| (rung, word));
	words.into_iter().map(|(word, _)| word).collect()
}

/// The most entries a list of the moves may hold: past it the field is dropped (the library has about 360 moves).
pub const MOST: usize = 512;
/// The longest group name, in characters (Library.lua's groups are a word each).
pub const GROUP_CHARS: usize = 32;
/// The longest title key, in characters (contract/sheet.json's `titles`' keys are a word or two in camel case).
pub const TITLE_CHARS: usize = 48;

/// Whether `word` is 1 to `most` lower-case letters, digits or underscores (a group name, a title key, a piece of
/// equipment).
pub(crate) fn word(w: &str, most: usize) -> bool {
	(1..=most).contains(&w.chars().count())
		&& w.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}

/// A field's value and when it last changed: unix milliseconds on the clock of the device that changed it (the
/// helper's for a change the game showed), never shifted by a clock offset.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Changed<T> {
	pub value: T,
	pub changed_at: i64,
}

/// The moves and the groups unchecked in the Book or on the game's Exercises page (Library.lua ids and group names).
/// Both lists are required: moves without one do not read, and their field is dropped alone, never taken as a change that
/// clears every exclusion.
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct Exclusions {
	pub ids: Vec<u16>,
	pub groups: Vec<String>,
}

/// The moves' part of the body: the exclusions (required) and the opt-in moves the player checked (the headstand),
/// which the Book's "Include in pumps" writes together.
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct Moves {
	pub exclude: Exclusions,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub include: Vec<u16>,
}

impl Moves {
	/// Whether a move is checked (the Book's Include in pumps; Settings.lua `Settings_.Excluded`, the phones'
	/// `EngineProfile.checks`): an opt-in move once it is in `include`, any other while neither it nor its `group` is
	/// excluded.
	#[must_use]
	pub fn checks(&self, id: u16, group: &str, opt_in: bool) -> bool {
		if opt_in {
			return self.include.contains(&id);
		}
		!self.exclude.ids.contains(&id) && !self.exclude.groups.iter().any(|g| g == group)
	}

	/// Checks or unchecks a move (the Book's Include in pumps; `Settings_.SetExcluded`, the phones'
	/// `EngineProfile.check`): an opt-in move joins or leaves `include`, any other leaves or joins the exclusion list.
	/// A move checked in an excluded group (the game's Exercises page excludes whole groups) takes its group off the
	/// list and the group's other moves (`group_moves`, the library's moves of that group) onto it, so they stay out as
	/// before.
	pub fn check(&mut self, id: u16, on: bool, group: &str, opt_in: bool, group_moves: &[u16]) {
		if opt_in {
			self.include.retain(|&m| m != id);
			if on {
				self.include.push(id);
				self.include.sort_unstable();
			}
			return;
		}
		if on && self.exclude.groups.iter().any(|g| g == group) {
			self.exclude.groups.retain(|g| g != group);
			let mut others: Vec<u16> =
				group_moves.iter().copied().filter(|&m| m != id && !self.exclude.ids.contains(&m)).collect();
			others.sort_unstable();
			self.exclude.ids.extend(others);
		}
		self.exclude.ids.retain(|&m| m != id);
		if !on {
			self.exclude.ids.push(id);
		}
	}

	/// Whether every list is within `MOST`, every move id names a move (0 names none) and every group is 1 to
	/// `GROUP_CHARS` lower-case letters, digits or underscores.
	fn checks_out(&self) -> bool {
		[self.exclude.ids.len(), self.exclude.groups.len(), self.include.len()].iter().all(|&n| n <= MOST)
			&& self.exclude.ids.iter().chain(&self.include).all(|&id| id != 0)
			&& self.exclude.groups.iter().all(|g| word(g, GROUP_CHARS))
	}
}

/// The most pieces of equipment a place holds (`PlaceSetup`).
pub const GEAR_MOST: usize = 64;

/// contract/sheet.json's `gear_order`, built in: the order a place lists its equipment in (`PlaceSetup`); empty when
/// it does not read (`make check` keeps it readable).
static GEAR_ORDER: LazyLock<Vec<String>> = LazyLock::new(|| {
	#[derive(Deserialize)]
	struct Sheet {
		gear_order: Vec<String>,
	}
	serde_json::from_str::<Sheet>(include_str!("../../../../contract/sheet.json"))
		.map_or_else(|_| Vec::new(), |s| s.gear_order)
});

/// `gear` in `gear_order`, a key it does not name (a newer device's) after them in its own order.
fn in_gear_order(gear: &mut [String]) {
	gear.sort_by_cached_key(|k| (GEAR_ORDER.iter().position(|g| g == k).unwrap_or(usize::MAX), k.clone()));
}

/// Whether `w` is `^[a-z]{1,most}$`.
fn letters(w: &str, most: usize) -> bool {
	(1..=most).contains(&w.len()) && w.bytes().all(|b| b.is_ascii_lowercase())
}

/// A place's equipment and its three switches (slice 51, `contract/companion-api.yaml` `PlaceSetup`): what the engine's
/// profile reads of a place. `gear` holds keys of contract/sheet.json's `gear` (a key this helper does not know, a
/// newer device's, is kept and passed on); the default is the addon's: no equipment, Floor moves and Room to move on,
/// Quiet mode off.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PlaceSetup {
	pub gear: Vec<String>,
	/// Floor moves.
	pub floor: bool,
	/// Room to move.
	pub space: bool,
	/// Quiet mode.
	pub quiet: bool,
}

impl Default for PlaceSetup {
	fn default() -> Self {
		Self { gear: Vec::new(), floor: true, space: true, quiet: false }
	}
}

impl PlaceSetup {
	/// At most `GEAR_MOST` pieces, each 1-32 lower-case letters, digits or underscores (a place's, and work mode's own
	/// equipment's, `crate::work`).
	pub(crate) fn checks_out(&self) -> bool {
		self.gear.len() <= GEAR_MOST && self.gear.iter().all(|k| word(k, 32))
	}

	/// The same setup, whatever the order its equipment is listed in (the game's profile keeps it as a set).
	#[must_use]
	pub fn same(&self, other: &Self) -> bool {
		let sorted = |s: &Self| {
			let mut g = s.gear.clone();
			g.sort_unstable();
			g
		};
		(self.floor, self.space, self.quiet) == (other.floor, other.space, other.quiet) && sorted(self) == sorted(other)
	}
}

/// A place but Home (slice 51, `contract/companion-api.yaml` `Place`): `id` a template's id for a place the seeding
/// made, else `p-` and 8 lower-case hex digits; `template` the template it came from (none for an empty one); `name`
/// none while it is the template's words, else the player's own; `glyph` one of `contract/places.json`'s `glyphs` (one
/// this helper does not know is kept and passed on); its equipment and switches.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Place {
	pub id: String,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub template: Option<String>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub name: Option<String>,
	pub glyph: String,
	#[serde(flatten)]
	pub setup: PlaceSetup,
}

impl Place {
	/// Whether the place reads as the contract says: its id (`^([a-z]{1,16}|p-[0-9a-f]{8})$`, never Home's), its
	/// template (`^[a-z]{1,16}$`), its name (1 to `place_name_max` Unicode scalars, trimmed already), its glyph
	/// (`^[a-z_]{1,32}$`) and its setup.
	#[must_use]
	pub fn checks_out(&self) -> bool {
		let home = places::places().map_or("home", |p| p.home.as_str());
		let most = places::places().map_or(24, |p| p.place_name_max);
		let name = self.name.as_ref().is_none_or(|n| (1..=most).contains(&n.chars().count()) && n.trim() == n);
		let glyph =
			(1..=32).contains(&self.glyph.len()) && self.glyph.bytes().all(|b| b.is_ascii_lowercase() || b == b'_');
		place_id(&self.id)
			&& self.id != home
			&& self.template.as_ref().is_none_or(|t| letters(t, 16))
			&& name && glyph
			&& self.setup.checks_out()
	}
}

/// Whether `id` reads as a place's id: `^([a-z]{1,16}|p-[0-9a-f]{8})$` (Home's among them).
fn place_id(id: &str) -> bool {
	let hex = |h: &str| h.len() == 8 && h.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b));
	letters(id, 16) || id.strip_prefix("p-").is_some_and(hex)
}

/// The minutes of a day: a time's `from` and `to` are below it.
const DAY_MIN: u16 = 1_440;

/// One time of the places' schedule (2026-11-03, `contract/companion-api.yaml` `PlaceTime`): `days` the ISO weekdays
/// it runs on (1 Monday to 7 Sunday, each once, at least one), `from` and `to` minutes after local midnight (never
/// equal; a `to` before `from` runs past midnight into the next day, the span belonging to the day it starts), `place`
/// a place's id (one this device does not list is kept and passed on, and falls through: `places::in_use`).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PlaceTime {
	pub days: Vec<u8>,
	pub from: u16,
	pub to: u16,
	pub place: String,
}

impl PlaceTime {
	/// Whether the time reads as the contract says.
	#[must_use]
	pub fn checks_out(&self) -> bool {
		let mut days = self.days.clone();
		days.sort_unstable();
		days.dedup();
		!self.days.is_empty()
			&& days.len() == self.days.len()
			&& self.days.iter().all(|d| (1..=7).contains(d))
			&& self.from < DAY_MIN
			&& self.to < DAY_MIN
			&& self.from != self.to
			&& place_id(&self.place)
	}
}

/// The places' plan (2026-11-03, `contract/places.json` `plan`, `contract/companion-api.yaml` `PlacePlan`): the place
/// in use whenever no time applies (`default`, Home at first) and the weekly schedule's times in the player's order.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PlacePlan {
	pub default: String,
	pub times: Vec<PlaceTime>,
}

impl Default for PlacePlan {
	fn default() -> Self {
		Self { default: places::places().map_or("home", |p| p.home.as_str()).to_owned(), times: Vec::new() }
	}
}

impl PlacePlan {
	/// Whether the plan reads: its default a place's id, at most `schedule_max` times, each reading.
	#[must_use]
	pub fn checks_out(&self) -> bool {
		let most = places::places().map_or(21, |p| p.schedule_max);
		place_id(&self.default) && self.times.len() <= most && self.times.iter().all(PlaceTime::checks_out)
	}
}

/// Whether a list of places reads: at most `places_max` less one (Home is never in it), each place reading, no id
/// named twice.
fn places_check(list: &[Place]) -> bool {
	let most = places::places().map_or(11, |p| p.places_max.saturating_sub(1));
	let mut ids: Vec<&str> = list.iter().map(|p| p.id.as_str()).collect();
	ids.sort_unstable();
	ids.dedup();
	list.len() <= most && ids.len() == list.len() && list.iter().all(Place::checks_out)
}

/// The body every app device shares: each field present once a device changed it. A field that does not read is
/// dropped alone as the body arrives (`link::lenient`), one that reads but does not check out by `validate`.
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct Body {
	#[serde(default, skip_serializing_if = "Option::is_none", deserialize_with = "crate::link::lenient")]
	pub no_jump: Option<Changed<bool>>,
	#[serde(default, skip_serializing_if = "Option::is_none", deserialize_with = "crate::link::lenient")]
	pub no_strain: Option<Changed<bool>>,
	#[serde(default, skip_serializing_if = "Option::is_none", deserialize_with = "crate::link::lenient")]
	pub no_head_down: Option<Changed<bool>>,
	/// An answer of the experience question (`is_experience`).
	#[serde(default, skip_serializing_if = "Option::is_none", deserialize_with = "crate::link::lenient")]
	pub experience: Option<Changed<String>>,
	#[serde(default, skip_serializing_if = "Option::is_none", deserialize_with = "crate::link::lenient")]
	pub moves: Option<Changed<Moves>>,
	/// The moves marked favourites in the apps (the Book's Favourite, on the computer and the phones), ascending
	/// (2026-10-09: the engine takes them first, so every device composes alike from the same rows). The game's move
	/// menu keeps its own in its profile (DB.lua `favourites`), which never crosses: `GameBody` reads none and the
	/// game's `ns.TakeBody` takes none.
	#[serde(default, skip_serializing_if = "Option::is_none", deserialize_with = "crate::link::lenient")]
	pub favourites: Option<Changed<Vec<u16>>>,
	/// The title chosen on Hero for the app's own sheet (a key of contract/sheet.json's `titles`), empty for none.
	#[serde(default, skip_serializing_if = "Option::is_none", deserialize_with = "crate::link::lenient")]
	pub title: Option<Changed<String>>,
	/// Moves per match (slice 48, `is_pace`): the pace of every pump composed after a game's end, a match's on the
	/// computer or a phone's, and of one of kind `match` picked by hand; `DEFAULT_PACE` while absent.
	#[serde(default, skip_serializing_if = "Option::is_none", deserialize_with = "crate::link::lenient")]
	pub match_pace: Option<Changed<String>>,
	/// Home's equipment and switches (slice 51), the game's one setup: between the game, the computer and the phones.
	#[serde(default, skip_serializing_if = "Option::is_none", deserialize_with = "crate::link::lenient")]
	pub home: Option<Changed<PlaceSetup>>,
	/// Every place but Home in the player's order (slice 51), whole: between the phones and the computer, never in the
	/// mirror.
	#[serde(default, skip_serializing_if = "Option::is_none", deserialize_with = "crate::link::lenient")]
	pub places: Option<Changed<Vec<Place>>>,
	/// The places' plan (2026-11-03), the default place and the weekly schedule, whole: between the phones and the
	/// computer, never in the mirror; the place in use never travels.
	#[serde(default, skip_serializing_if = "Option::is_none", deserialize_with = "crate::link::lenient")]
	pub place_plan: Option<Changed<PlacePlan>>,
}

/// A field is taken from `theirs` when it changed later than `mine` (at the same time too when `ties`), or `mine` has
/// none; whether `mine` changed.
fn take<T: Clone + PartialEq>(mine: &mut Option<Changed<T>>, theirs: Option<&Changed<T>>, ties: bool) -> bool {
	let Some(theirs) = theirs else { return false };
	let later =
		mine.as_ref().is_none_or(|m| theirs.changed_at > m.changed_at || (ties && theirs.changed_at == m.changed_at));
	if !later || mine.as_ref() == Some(theirs) {
		return false;
	}
	*mine = Some(theirs.clone());
	true
}

/// A field that changed at no time after 0, or later than `link::AHEAD_MS` past the helper's `now` (a device clock set
/// ahead), is no change.
fn dated<T>(field: &mut Option<Changed<T>>, now: i64) {
	if field.as_ref().is_some_and(|f| f.changed_at <= 0 || !link::not_ahead(f.changed_at, now)) {
		*field = None;
	}
}

impl Body {
	/// Drops each field that does not check out, alone: an experience that does not answer the question, moves that do
	/// not check out, a pace that is none of Moves per match's, a Home or a list of places that does not read (`Place`),
	/// a plan that does not read (`PlacePlan`), a change dated at 0 or before or more than a day past
	/// the helper's `now` (unix milliseconds).
	pub fn validate(&mut self, now: i64) {
		if self.experience.as_ref().is_some_and(|e| !is_experience(&e.value)) {
			self.experience = None;
		}
		if self.match_pace.as_ref().is_some_and(|p| !is_pace(&p.value)) {
			self.match_pace = None;
		}
		if self.moves.as_ref().is_some_and(|m| !m.value.checks_out()) {
			self.moves = None;
		}
		if self.favourites.as_ref().is_some_and(|f| f.value.len() > MOST || f.value.contains(&0)) {
			self.favourites = None;
		}
		let title = |t: &str| {
			t.is_empty() || (t.len() <= TITLE_CHARS && t.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_'))
		};
		if self.title.as_ref().is_some_and(|t| !title(&t.value)) {
			self.title = None;
		}
		if self.home.as_ref().is_some_and(|h| !h.value.checks_out()) {
			self.home = None;
		}
		if self.places.as_ref().is_some_and(|p| !places_check(&p.value)) {
			self.places = None;
		}
		if self.place_plan.as_ref().is_some_and(|p| !p.value.checks_out()) {
			self.place_plan = None;
		}
		dated(&mut self.no_jump, now);
		dated(&mut self.no_strain, now);
		dated(&mut self.no_head_down, now);
		dated(&mut self.experience, now);
		dated(&mut self.moves, now);
		dated(&mut self.favourites, now);
		dated(&mut self.title, now);
		dated(&mut self.match_pace, now);
		dated(&mut self.home, now);
		dated(&mut self.places, now);
		dated(&mut self.place_plan, now);
	}

	/// Takes each field of `theirs` that changed later than this body's (at the same time too when
	/// `theirs_wins_ties`); whether this body changed. The helper takes a device's fields with ties to its own.
	pub fn merge(&mut self, theirs: &Body, theirs_wins_ties: bool) -> bool {
		let ties = theirs_wins_ties;
		// every field, never stopping at the first that changed
		[
			take(&mut self.no_jump, theirs.no_jump.as_ref(), ties),
			take(&mut self.no_strain, theirs.no_strain.as_ref(), ties),
			take(&mut self.no_head_down, theirs.no_head_down.as_ref(), ties),
			take(&mut self.experience, theirs.experience.as_ref(), ties),
			take(&mut self.moves, theirs.moves.as_ref(), ties),
			take(&mut self.favourites, theirs.favourites.as_ref(), ties),
			take(&mut self.title, theirs.title.as_ref(), ties),
			take(&mut self.match_pace, theirs.match_pace.as_ref(), ties),
			take(&mut self.home, theirs.home.as_ref(), ties),
			take(&mut self.places, theirs.places.as_ref(), ties),
			take(&mut self.place_plan, theirs.place_plan.as_ref(), ties),
		]
		.contains(&true)
	}
}

/// When the game last changed each field of its body (DB.lua's profile `bodyAt`, `ns.BodyStamp`), in unix
/// milliseconds (the addon's seconds × 1000); none for a field from before those times or never changed in the game.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, PartialEq, Eq)]
#[serde(default)]
pub struct GameStamps {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub no_jump: Option<i64>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub no_strain: Option<i64>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub no_head_down: Option<i64>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub moves: Option<i64>,
	/// Home's setup (slice 51: `bodyAt.home`, Settings.lua's stamp of an equipment piece, a place switch or a preset).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub home: Option<i64>,
}

impl GameStamps {
	fn is_empty(&self) -> bool {
		*self == Self::default()
	}
}

/// A Lua table's entries, whether its CBOR (or the save's text read back) holds a map or a list (a table keyed 1 to n
/// may be written as one).
fn entries(v: Option<&Value>) -> Vec<(Value, Value)> {
	match v {
		Some(Value::Map(m)) => m.clone(),
		Some(Value::Array(a)) => (1_u64..).zip(a.iter().cloned()).map(|(i, x)| (Value::Integer(i.into()), x)).collect(),
		_ => Vec::new(),
	}
}

/// The game's body in one profile, as the strip paints it or a file holds it (DB.lua's `noJump`, `noStrain`,
/// `noHeadDown`, `exclude` and `include`, and since slice 51 Home's `gear`, `floor`, `space` and `quiet`; the active
/// profile of the character the player plays), with when the game changed each (`at`).
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
#[serde(default)]
pub struct GameBody {
	pub no_jump: bool,
	pub no_strain: bool,
	pub no_head_down: bool,
	/// The Exercises page's list (the moves and groups unchecked, the opt-in moves checked); none in a `body.json` from
	/// before 2026-10-09, and for a list the body cannot carry (`Moves::checks_out`).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub moves: Option<Moves>,
	/// Home's setup (slice 51): the profile's equipment in `gear_order` (the profile keeps a set), and its three
	/// switches; the addon's defaults in a `body.json` from before slice 51.
	pub home: PlaceSetup,
	#[serde(skip_serializing_if = "GameStamps::is_empty")]
	pub at: GameStamps,
}

impl GameBody {
	/// The body of a profile as the addon painted or saved it: a switch the profile leaves out is off and a list it
	/// leaves out empty, the addon's defaults; a stamp is read in seconds.
	#[must_use]
	pub fn of_profile(profile: &Value) -> Self {
		let on = |key| matches!(blob::get(profile, key), Some(Value::Bool(true)));
		let set = |key| entries(blob::get(profile, key)).into_iter().filter(|(_, v)| matches!(v, Value::Bool(true)));
		let mut exclude = Exclusions::default();
		for (k, _) in set("exclude") {
			match k {
				Value::Text(group) => exclude.groups.push(group),
				k => exclude.ids.extend(blob::as_u64(&k).and_then(|id| u16::try_from(id).ok())),
			}
		}
		let mut include: Vec<u16> = set("include").filter_map(|(k, _)| u16::try_from(blob::as_u64(&k)?).ok()).collect();
		let mut gear: Vec<String> = set("gear")
			.filter_map(|(k, _)| match k {
				Value::Text(key) => Some(key),
				_ => None,
			})
			.collect();
		in_gear_order(&mut gear);
		let defaults = PlaceSetup::default();
		let switch = |key, default| match blob::get(profile, key) {
			Some(Value::Bool(b)) => *b,
			_ => default,
		};
		let home = PlaceSetup {
			gear,
			floor: switch("floor", defaults.floor),
			space: switch("space", defaults.space),
			quiet: switch("quiet", defaults.quiet),
		};
		exclude.ids.sort_unstable();
		exclude.groups.sort_unstable();
		include.sort_unstable();
		let moves = Moves { exclude, include };
		let stamps = blob::get(profile, "bodyAt");
		let stamp = |key| {
			let s = blob::as_u64(blob::get(stamps?, key)?)?;
			i64::try_from(s).ok()?.checked_mul(1_000)
		};
		Self {
			no_jump: on("noJump"),
			no_strain: on("noStrain"),
			no_head_down: on("noHeadDown"),
			moves: moves.checks_out().then_some(moves),
			home,
			at: GameStamps {
				no_jump: stamp("noJump"),
				no_strain: stamp("noStrain"),
				no_head_down: stamp("noHeadDown"),
				moves: stamp("moves"),
				home: stamp("home"),
			},
		}
	}
}

/// A field of the game's body as a change of the body: at the game's own time when it has one, else at `at` when its
/// value differs from the one the helper saw last in that profile (`before`).
fn game_field<T: Clone + PartialEq>(now: &T, before: &T, stamp: Option<i64>, at: i64) -> Option<Changed<T>> {
	match stamp {
		Some(t) => Some(Changed { value: now.clone(), changed_at: t }),
		None => (now != before).then(|| Changed { value: now.clone(), changed_at: at }),
	}
}

/// What the helper keeps in `body.json`: the body every app device shares, and the game's body as the strip or a file
/// last showed it in each profile.
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct Kept {
	#[serde(default)]
	pub body: Body,
	/// The game's body the helper saw last in each profile, by its install and its name (`<install>/<profile name>`,
	/// the session's `Effects::game_body`), so another character's profile, or another install's, is never a change;
	/// none before the strip first painted a profile.
	#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
	pub game: BTreeMap<String, GameBody>,
	/// Why the file did not read at this start, when it was there and did not (kept aside as `body.json.unread`): the
	/// body started empty, which Your setup says until a device fills it again.
	#[serde(skip)]
	pub unread: Option<String>,
}

impl Kept {
	pub const FILE: &str = "body.json";

	/// The kept body from the data folder, each field checked again at `now` (a hand edit); an absent file is an empty
	/// one, and so is a damaged one, kept aside first (`mirror::read_data`), which the phones' next calls fill again.
	#[must_use]
	pub fn load(dir: &Path, now: i64) -> Self {
		let read = crate::mirror::read_data(dir, Self::FILE, |b| serde_json::from_slice(b).map_err(|e| e.to_string()));
		let mut kept = match read {
			Ok(kept) => kept.unwrap_or_default(),
			Err(why) => Self { unread: Some(why), ..Self::default() },
		};
		kept.body.validate(now);
		kept
	}

	/// The file's bytes.
	///
	/// # Errors
	/// Never for these types; serde's error otherwise.
	pub fn to_json(&self) -> serde_json::Result<Vec<u8>> {
		serde_json::to_vec(self)
	}

	/// A device's body, checked already (`Body::validate`), into the shared one: each field it changed later wins, a tie
	/// keeps the helper's. Whether the shared body changed.
	pub fn take(&mut self, theirs: &Body) -> bool {
		self.body.merge(theirs, false)
	}

	/// The game's body as the strip or a file shows it in `profile` (`<install>/<profile name>`) at `at` (the helper's
	/// clock, or the file's time, milliseconds): each field the game stamped joins the body at the game's own time
	/// (`GameStamps`), so the latest change wins whichever device made it, and a value the game took from the body
	/// brings back its own time and changes nothing; each field from before the stamps that differs from the last the
	/// helper saw in that profile is the player's change in the game and joins at `at`, the first the helper sees of a
	/// profile counting against the addon's defaults (every switch off, nothing left out), so a switch the player
	/// turned on before counts and an untouched one never overrides a phone's. Another profile's body (another
	/// character's, another install's) is never a change. Whether anything to keep changed (the body or the game's body
	/// seen).
	pub fn game(&mut self, profile: &str, seen: &GameBody, at: i64) -> bool {
		let last = self.game.insert(profile.to_owned(), seen.clone());
		if last.as_ref() == Some(seen) {
			return false;
		}
		let was = last.unwrap_or_default();
		let mut theirs = Body {
			no_jump: game_field(&seen.no_jump, &was.no_jump, seen.at.no_jump, at),
			no_strain: game_field(&seen.no_strain, &was.no_strain, seen.at.no_strain, at),
			no_head_down: game_field(&seen.no_head_down, &was.no_head_down, seen.at.no_head_down, at),
			moves: seen
				.moves
				.as_ref()
				.and_then(|m| game_field(m, &was.moves.clone().unwrap_or_default(), seen.at.moves, at)),
			// the profile keeps its equipment as a set: its order alone is no change
			home: match seen.at.home {
				Some(t) => Some(Changed { value: seen.home.clone(), changed_at: t }),
				None => (!seen.home.same(&was.home)).then(|| Changed { value: seen.home.clone(), changed_at: at }),
			},
			..Body::default()
		};
		theirs.validate(at);
		self.body.merge(&theirs, true);
		true
	}
}
