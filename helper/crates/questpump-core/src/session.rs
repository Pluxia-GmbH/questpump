//! The helper's view of the game (docs/plan.md slice 5): every decoded strip frame goes through `Session::on_frame`,
//! which stores the rows, rebuilds each offered pump, keeps the latest profile and sheet per character (and logs each
//! new sheet for the phone's charts), hands on each feedback message the game sent, learns the felt-unwell lock and says
//! when the mirror is due. The phones' app rows come in beside the game's (`Session::take_app_rows`, slice 26) and go
//! back out to every paired phone (`AppRows::page`, slice 28), and each game install's mirror is built in that
//! install's own numbering of the characters (`Session::mirrored`). No clock, file or socket here: the app passes the
//! time in and acts on the returned `Effects`.

use std::collections::{BTreeMap, HashMap, HashSet};

use ciborium::Value;
use serde::{Deserialize, Serialize};

use crate::app_rows::{self, AppRows};
use crate::backup::GameRow;
use crate::blob::{self, Assembler};
use crate::body;
use crate::codes::{PumpEnd, StripEvent, StripState, cause, flags, kind, longer, outcome};
use crate::far::{self, FarRows};
use crate::feedback;
use crate::library;
use crate::link::{self, AppRow, Device, HistoryPage, HistoryProof, OwedMove, Sheet, SheetCharacter};
use crate::mirror::{self, Character, Held};
use crate::moves;
use crate::proof::{self, Book, HrSource};
use crate::protocol::{Command, Line, OfferedMove, PumpKind, RowRecord, SheetRecord, Source, StripLine, Yards};
use crate::row::Row;
use crate::sheets::SheetLog;
use crate::store::{self, Added, Store};
use crate::strip::{Body, Fields, RawFrame};
use crate::verdict::{Class, Outcome};

const WRAP: u64 = 1 << 24;
const DAY_S: i64 = 86_400;
/// The game day turns 4 hours after local midnight (`ns.C.day_offset_h`): 04:00 by the clock, 03:00 or 05:00 on a
/// night the clocks change.
const DAY_OFFSET_S: i64 = 4 * 3_600;
pub const COUNTDOWN_TICKS: u8 = 5;

/// The helper's clock at one moment: unix seconds and the same instant in RFC 3339 (the lines' `ts`).
#[derive(Debug, Clone)]
pub struct Now {
	pub unix: i64,
	pub rfc3339: String,
}

/// When the mirror should be written next.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MirrorDue {
	/// After the debounce (2 s after the last of a batch of strip rows, 10 s after its first at most).
	Soon,
	/// At once (a reset, a catch-up, the game's exit, a pairing).
	Now,
}

/// What one input changed, for the app to act on.
#[derive(Debug, Default)]
#[expect(clippy::struct_excessive_bools, reason = "independent effects the app acts on one by one, never one state")]
pub struct Effects {
	pub lines: Vec<Line>,
	/// What the device that holds the link should hear (the app queues them in `link::Link`).
	pub commands: Vec<Command>,
	pub mirror: Option<MirrorDue>,
	/// Some(true) when the addon leaves the world (the mirror waits), Some(false) when the strip is back.
	pub suspend: Option<bool>,
	/// The game sent `/qp ping`: the app answers it out loud, in its own words (its copy, never the core's).
	pub pinged: bool,
	/// The characters, profiles or sheets changed: the app saves them.
	pub people_changed: bool,
	/// Rows that could not be written to `rows.jsonl` or `app.jsonl`, each naming its file: a game row stays in memory
	/// and reaches the mirror; an app row is not kept, and the sync answer then carries no `rows_store`, so the phone
	/// sends it again.
	pub store_errors: Vec<String>,
	/// Rows refused for their time (`library::row_timely`: dated before QuestPump, more than a day past the helper's
	/// clock, or a reset or felt-unwell row more than `row_ahead_floor_s` past it), for the log: none of them reaches a
	/// store, the mirror or a device.
	pub untimely: usize,
	/// Sheets that could not be written to `sheets.jsonl` (they stay in memory and are served all the same).
	pub sheet_errors: Vec<String>,
	/// A pump's heart-rate proof changed: the app saves the book (`Book::FILE`).
	pub proof_changed: bool,
	/// An addon load ended and the next began (a `/reload`, a logout and login): the client wrote its save between
	/// them, so the app takes the save in (the rows that never reached the strip: an import, a restore, a row still
	/// queued at the `/reload`).
	pub load_ended: bool,
	/// A payload that assembled but did not decode or check out, for the log, by its event and the rule it broke (never a
	/// feedback message's text): a profile's or a sheet's once per load and event, every feedback message's.
	pub blob_errors: Vec<String>,
	/// A message the player sent from the game's feedback window, checked against the route's rules: the app keeps it
	/// in its outbox and posts it once (slice 24).
	pub feedback: Option<feedback::Message>,
	/// A message from the game's feedback window did not decode or check out: it is dropped as the route's 400 drops
	/// one, and the app's note says so as it does for a 400 (its `blob_errors` line names the rule).
	pub feedback_dropped: bool,
	/// A pump whose whole list arrived, for the log: its moves as the strip named them (David, 2026-10-04: the addon's
	/// own log reaches the disk only at a `/reload` or a logout, so a pump on offer could not be read while it showed).
	pub offered: Option<String>,
	/// The game's body (its Body switches and its Exercises page's list), as the profile a `profile` event carried
	/// holds it (the character the player plays), or as a file taken in holds it in the profile its newest character
	/// uses: the app joins a change of it within that profile to the body every device shares (`body::Kept::game`,
	/// slice 35). None for a profile older than the copy the helper holds (`profile_newer`): a stale one is no change.
	pub game_body: Option<GameBodySeen>,
}

/// The game's body in one profile, as the strip or a file showed it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameBodySeen {
	/// The profile's install and name (`<install>/<profile name>`): only a change within one profile is the player's.
	pub profile: String,
	pub seen: body::GameBody,
	/// When the game showed it, unix milliseconds: a file's time for a save or a mirror taken in; none for the strip,
	/// which shows it now (the app's clock).
	pub at: Option<i64>,
}

impl Effects {
	fn due(&mut self, due: MirrorDue) {
		self.mirror = self.mirror.max(Some(due));
	}
}

/// The characters, their profiles and the latest sheet each, as the strip's `profile` and `sheet` events carry them.
/// Kept as CBOR values, as the addon wrote them: a profile's exclusion list is keyed by move id, and JSON would turn
/// those integer keys into text the addon no longer matches. The app saves it as CBOR (`People::FILE`), so a restarted
/// helper still writes them into the mirror.
///
/// Each game install numbers its own characters (docs/plan.md slice 26), so the helper keeps one list of its own across
/// every install (`chars`, the `char` of the stored rows, the history rows and the sheets) and, per install, which of
/// them each index of that install is, with that install's profiles (`installs`). A field added since is read with its
/// default from an older file, so an update never empties it.
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct People {
	/// `chars[i]` is the helper's character `i + 1`, across every install; None for a placeholder (an install's index
	/// whose rows came before any profile or file named it).
	pub chars: Vec<Option<Character>>,
	/// The profiles by name from before slice 26, when the helper knew one install: the first install it follows takes
	/// them (`Session::set_install`), and they are empty from then on.
	pub profiles: BTreeMap<String, Value>,
	/// The profile each character uses, by GUID, when it is not Default; from before slice 26, as `profiles`.
	pub profile_keys: BTreeMap<String, String>,
	/// The latest sheet summary per character (the helper's numbering; served to the apps from slice 7).
	pub sheets: BTreeMap<u8, Value>,
	/// When the helper last learnt `profiles`, from before slice 26, as `profiles`.
	#[serde(default)]
	pub profiles_at: i64,
	/// The owed moves the game held back at its latest sheet (`held`: a pain flag, a Body switch, the equipment or an
	/// excluded move, the rung rule; Engine.lua `ns.OwedHeld`): a catch-up pays none of them. Until 2026-10-04 the
	/// game also held every owed move before the player's first move, while its first pump asked the pain question.
	#[serde(default)]
	pub held: Vec<u16>,
	/// Those of `held` the game's place leaves out with no move of their level to stand in (Engine.lua `ns.OwedAside`,
	/// 2026-11-09): no list shows them nor counts them as held until the setup lets them in again.
	#[serde(default)]
	pub aside: Vec<u16>,
	/// Each game install the helper has followed or taken a file in from, by its AddOns folder.
	#[serde(default)]
	pub installs: BTreeMap<String, Install>,
	/// A placeholder a file or a profile showed to be a character the helper knew, while its rows and logged sheets
	/// wait to be filed under that character (`Session::settle_aliases`, which drops it once both files are rewritten):
	/// until then it reads as that character everywhere, and its id is not free.
	#[serde(default)]
	pub aliases: BTreeMap<u8, u8>,
	/// How many app rows the helper held (`Session::app_mark`) when the load that painted each character's latest sheet
	/// (`sheets`) began, by the same index: the game read those through the mirror at that load, and none it took
	/// since (`Session::sheet_lacks_app_rows`; one live character, 2026-10-09). None for a sheet of an older helper.
	#[serde(default)]
	pub sheet_apps: BTreeMap<u8, usize>,
	/// The characters of the account's other computers (slice 60), by each computer's install id (`hid`) and the index
	/// that computer's rows name them by (its helper's numbering), as its newest `device` batch named them: the rows an
	/// older desktop app sent without `chars` (`far::FarRows`) reach each mirror as the character with that GUID here.
	#[serde(default)]
	pub far: BTreeMap<String, BTreeMap<u8, Character>>,
}

/// One game install as the helper knows it: which of the helper's characters each of its character indexes is, and
/// its own profiles. A profile belongs to its install (`/qp reset char` stays there): the addon's merge takes a
/// mirror's profiles by name, so another install's "Default" must never reach it.
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct Install {
	/// The install's index → the helper's character: its own characters at their indexes, and each other character of
	/// the helper at a free index of this install from the first mirror that carried it on, so every later mirror
	/// writes its rows as the same strings.
	#[serde(default)]
	pub chars: BTreeMap<u8, u8>,
	#[serde(default)]
	pub profiles: BTreeMap<String, Value>,
	/// The profile each character uses, by GUID, when it is not Default.
	#[serde(default)]
	pub profile_keys: BTreeMap<String, String>,
	/// When the helper last learnt this install's profiles (a `profile` event, or a file taken in): a file saved later
	/// holds the fresher ones.
	#[serde(default)]
	pub profiles_at: i64,
}

/// What one install's `State.lua` holds of the session (`Session::mirrored`), in that install's numbering.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Mirrored {
	/// The game's rows and the app's, merged by time as the addon's merge orders two lists.
	pub rows: Vec<Row>,
	/// `chars[i]` is the install's index `i + 1`: its own characters and the others it carries.
	pub chars: Vec<Option<Character>>,
	pub profiles: BTreeMap<String, Value>,
	pub profile_keys: BTreeMap<String, String>,
}

/// What the newest save of an install the helper read with a baseline (one its client reads back: mirror.rs
/// `parse_save`) holds beside its rows (`Session::adopt`): its time (`savedAt`), its baseline (DB.lua `ns.Trim`: the
/// history its ring let go, folded, with its horizon) and the characters that save numbers its rows and that baseline
/// by. Kept in memory only: the helper reads the saves again at its start (`mirror::SavesRead`), the followed
/// install's before its first mirror write, another's once its game runs or quits.
#[derive(Debug, Clone, PartialEq)]
struct SaveBase {
	saved_at: i64,
	baseline: Value,
	chars: Vec<Option<Character>>,
}

/// A character as the engine's merge maps it (DB.lua `charMap`: by its GUID); a placeholder is an empty table, so the
/// list keeps its places.
#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Guid<'a> {
	#[serde(skip_serializing_if = "Option::is_none")]
	guid: Option<&'a str>,
}

/// The save the engine folds a character's sheet from as the game folds it at its next load (`Session::game_save`;
/// tools/apppump.lua `Engine:gameSummary`'s `save`), for the engine's `gameSave` op as CBOR, so the tables of a profile
/// and a baseline keep their integer keys (a move's id, a ladder's).
#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GameSave<'a> {
	/// The addon's profile the character uses (DB.lua `ns.ProfileFor`: its profile key's, else Default) as the helper last
	/// learnt it (a `profile` event, a file taken in); none for a character the helper cannot name, and before it learnt
	/// that install's profiles.
	#[serde(skip_serializing_if = "Option::is_none")]
	profile: Option<&'a Value>,
	/// The body every device shares as the mirror carries it (`mirror::body_value`), which the game takes into that
	/// profile at its next load (DB.lua `ns.TakeBody`, as `ns.BindDB` does).
	body: Value,
	/// The install's newest save's baseline the helper read since it started (`SaveBase`); none before it read one.
	#[serde(skip_serializing_if = "Option::is_none")]
	baseline: Option<&'a Value>,
	/// With the baseline, the characters that save numbers its rows and that baseline by; empty without it.
	chars: Vec<Guid<'a>>,
	/// With the baseline, the helper's own characters (an alias as the character it stands for), which number the rows
	/// the engine folds (`GET /v1/history`'s `char`), so the merge maps each row to that save's number as the mirror's
	/// rows are mapped; empty without it.
	row_chars: Vec<Guid<'a>>,
}

/// Each character of a list as the engine's merge maps it (`Guid`).
fn guids(chars: &[Option<Character>]) -> Vec<Guid<'_>> {
	chars.iter().map(|c| Guid { guid: c.as_ref().map(|c| c.guid.as_str()) }).collect()
}

/// The body of the profile a file's newest character uses (the character of its newest game row, else its first; its
/// profile by GUID, else Default), keyed as the strip's `profile` event keys it, at the file's time: a no jumping set
/// in the game reaches the body every device shares even where the strip was never read (no Screen Recording grant).
/// None for a file without that profile, and for one whose profile the install holds a later copy of (`adopt_profiles`
/// took the copy only when the file's changed later).
fn held_body(held: &Held, install: &str, kept: &Install) -> Option<GameBodySeen> {
	let newest = held.rows.iter().filter(|r| Source::from_flags(r.flags) != Source::App).max_by_key(|r| r.ts);
	let character = newest
		.and_then(|r| usize::from(r.char).checked_sub(1))
		.and_then(|i| held.chars.get(i)?.as_ref())
		.or_else(|| held.chars.iter().flatten().next());
	let name = character.and_then(|c| held.profile_keys.get(&c.guid)).map_or("Default", String::as_str);
	let render = |v: &Value| mirror::lua_value(v, "");
	let profile = held.profiles.get(name).filter(|&p| kept.profiles.get(name).map(render) == Some(render(p)))?;
	Some(GameBodySeen {
		profile: format!("{install}/{name}"),
		seen: body::GameBody::of_profile(profile),
		at: Some(held.saved_at.saturating_mul(1_000)),
	})
}

/// A profile's own change stamp (`changedAt`, unix seconds: DB.lua `ns.Touch`); none for one from before the stamps,
/// or never changed since.
fn changed_at(profile: &Value) -> Option<u64> {
	blob::get(profile, "changedAt").and_then(blob::as_u64)
}

/// Whether a profile the strip or a file shows (`theirs`) takes the place of the install's copy (`mine`), as the
/// addon's merge takes the mirror's (DB.lua `ns.ProfileNewer`): when there is none, or `theirs` changed later by their
/// own stamps (at the same time too with `ties`: the strip shows the profile the game holds now); a stamped one beats
/// one without, and of two without `unstamped` decides, as before the stamps.
fn profile_newer(theirs: &Value, mine: Option<&Value>, ties: bool, unstamped: bool) -> bool {
	let Some(mine) = mine else { return true };
	match (changed_at(theirs), changed_at(mine)) {
		(Some(a), Some(b)) => a > b || (ties && a == b),
		(Some(_), None) => true,
		(None, Some(_)) => false,
		(None, None) => unstamped,
	}
}

/// The character an alias stands for (a chain cannot loop: an alias always names a character that has one).
fn resolve(aliases: &BTreeMap<u8, u8>, mut id: u8) -> u8 {
	for _ in 0..=u8::MAX {
		match aliases.get(&id) {
			Some(&to) => id = to,
			None => break,
		}
	}
	id
}

impl People {
	pub const FILE: &str = "people.cbor";

	/// Reads the saved copy from the data folder: an absent file is an empty one, and so is a damaged one, kept aside
	/// first (`mirror::read_data`), with why it did not read for the caller's log.
	#[must_use]
	pub fn load(dir: &std::path::Path) -> (Self, Option<String>) {
		let read = crate::mirror::read_data(dir, Self::FILE, |b| ciborium::from_reader(b).map_err(|e| e.to_string()));
		match read {
			Ok(people) => (people.unwrap_or_default(), None),
			Err(why) => (Self::default(), Some(why)),
		}
	}

	/// The character an id stands for (itself unless it is an alias).
	#[must_use]
	pub fn resolve(&self, id: u8) -> u8 {
		resolve(&self.aliases, id)
	}

	/// The helper's character an install's index is, when it is bound.
	#[must_use]
	pub fn bound(&self, install: &str, index: u8) -> Option<u8> {
		self.installs.get(install)?.chars.get(&index).map(|&id| self.resolve(id))
	}

	/// The helper's character with this GUID.
	fn by_guid(&self, guid: &str) -> Option<u8> {
		let at = self.chars.iter().position(|c| c.as_ref().is_some_and(|c| c.guid == guid))?;
		u8::try_from(at + 1).ok()
	}

	/// An id no character fills yet: a placeholder (an index no profile or file has named).
	fn placeholder(&self, id: u8) -> bool {
		id > 0 && !self.aliases.contains_key(&id) && self.chars.get(usize::from(id) - 1).is_some_and(Option::is_none)
	}

	/// An id nothing holds: no character fills it and no alias, binding or sheet names it (a placeholder filed under
	/// the character it turned out to be, or one a reset left unbound).
	fn free(&self, id: u8) -> bool {
		self.placeholder(id)
			&& !self.sheets.contains_key(&id)
			&& !self.installs.values().any(|i| i.chars.values().any(|&bound| bound == id))
	}

	/// A new placeholder of the helper's: the lowest free id no stored row or logged sheet names (`named`), else the
	/// next one (none past 255).
	fn add(&mut self, named: &HashSet<u8>) -> Option<u8> {
		let ids = (1..=self.chars.len()).filter_map(|i| u8::try_from(i).ok());
		if let Some(id) = ids.into_iter().find(|&id| self.free(id) && !named.contains(&id)) {
			return Some(id);
		}
		let id = u8::try_from(self.chars.len() + 1).ok()?;
		self.chars.push(None);
		Some(id)
	}

	/// The install's index → character list as its mirror numbers them: the helper's character to its index there.
	fn numbering(&self, install: &str) -> HashMap<u8, u8> {
		let mut map = HashMap::new();
		for (&index, &id) in self.installs.get(install).map(|i| &i.chars).into_iter().flatten() {
			map.entry(self.resolve(id)).or_insert(index);
		}
		map
	}
}

/// A move of the pump being offered, from its `pump_offered` frame.
#[derive(Debug, Clone)]
struct Move {
	fields: Fields,
	tier: u8,
	kind: u8,
}

/// The pump the addon offered and has not finished (only one runs at a time).
#[derive(Debug, Clone)]
struct Pump {
	offered_ts: u32,
	count: u8,
	moves: BTreeMap<u8, Move>,
	/// Slots a move row closed.
	closed: Vec<u8>,
	/// The slot whose count the player is typing (the result view): the addon records it itself at a logout.
	input: Option<u8>,
	/// Felt unwell ended it: nothing of it is owed.
	ended: bool,
}

/// The counted move of a load.
#[derive(Debug, Clone, Copy)]
struct Moving {
	/// Its pump's offered time.
	ts: u32,
	slot: u8,
	/// Its result view showed: the device heard it done, and its row may not be written yet, so the game's Back there
	/// can still take it back (2026-10-05). The game goes on to the next slot only once it wrote the row.
	ended: bool,
}

/// One addon load (a login or a /reload): its install, its clock, its pump and the events it has run.
#[derive(Default)]
struct Load {
	/// The install the helper followed when the strip first showed the load: its rows and characters are in that
	/// install's numbering, whatever install the helper follows when they arrive.
	install: String,
	/// The last unwrapped game clock, in milliseconds.
	clock: Option<u64>,
	pump: Option<Pump>,
	/// The event counters acted on: a frame shown again (after a cinematic, a loading screen) acts once.
	events: HashSet<u16>,
	/// The Starts whose countdown went out, by pump (offered time), slot and start: every move of a pump has its own
	/// Start, and a Start again after the countdown was cancelled is a new one; a heartbeat repeats the same start, so a
	/// helper that missed the `countdown_start` frame (it started mid-count) joins from it once.
	counted: HashSet<(u32, u8, u64)>,
	/// The move whose countdown went out last, until the game keeps it (its row, the pump going on, the load's end) or
	/// takes it back to its Ready.
	moving: Option<Moving>,
	/// The capture lost sight of the strip during this load and no heartbeat has shown the truth since: the pump it
	/// knows may have ended unseen, so the logout rule owes nothing for it (the client's save, taken in at the next
	/// load, carries the addon's own owed rows).
	stale: bool,
	/// The payload events whose decode failure was reported in this load.
	blob_warned: HashSet<StripEvent>,
	/// When the capture first showed this load (unix seconds): a save the client wrote after it ended this load
	/// (`Session::save_written`).
	first_shown: Option<f64>,
	/// The pad's walked count where the load's stretch began on the pad: its first reading in the load, then each
	/// stretch row's; none before that first reading, so a walk the helper did not see between two loads (the game
	/// closed, the pad off in the helper or held by its own app) counts toward no stretch.
	pad_from: Option<u64>,
	/// How many app rows the helper held when the capture first showed this load (`Session::app_mark`): the load read
	/// those through the mirror, never one taken since.
	apps: usize,
}

/// The walking pad's walk in one game day (`Session::pad_today`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PadDay {
	/// The metres the day's stretch rows hold: the walk kept with its quests.
	pub kept: u32,
	/// The metres of the stretch the current load has open, which its next stretch row takes (a new load starts the
	/// count again, as the addon's own stretch does); none while the helper knows no load of the game (the game closed
	/// or not seen since it started, its save written and the next load not seen yet). A load whose corner is out of
	/// sight stays the current one, and the walk still counts toward it.
	pub open: Option<u32>,
}

/// The walking pad's distance since the helper first read it (docs/plan.md slice 25). The pad's FTMS Total Distance
/// counts from the pad's own session start, so the odometer adds what each reading moved on from the last; the last
/// reading outlives a disconnect, so within a load a pad that kept its session counts the walk the helper did not
/// see (a new load's stretch starts at the pad's first reading in it, `Load::pad_from`).
#[derive(Debug, Default)]
struct PadOdometer {
	/// The last reading, in metres; none before the pad's first in this process.
	last: Option<u32>,
	/// The metres the pad counted since the first reading.
	walked: u64,
}

impl PadOdometer {
	fn read(&mut self, distance_m: u32) {
		if let Some(last) = self.last {
			// a reading below the last: the pad began a new session, counted from 0 again
			self.walked += u64::from(distance_m.checked_sub(last).unwrap_or(distance_m));
		}
		self.last = Some(distance_m);
	}
}

/// A catch-up pump from the phone, the watch or the desktop app (rust-helper `protocol-and-mirror.md`, "Catch-up
/// rows"): each move pays the oldest owed row of the move it names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatchupPump {
	/// The phone, the watch or the computer (`Source::Computer`, docs/plan.md slice 35), whose flag bit its rows carry.
	pub source: Source,
	/// `p-<uuid>` from the phone, `w-<uuid>` from the watch, `d-<uuid>` from the desktop app.
	pub id: String,
	/// The device's start, converted to unix seconds on the helper's clock.
	pub start: u32,
	pub moves: Vec<CatchupMove>,
	/// The verdict a watch measured itself, which the helper's own scoring may better.
	pub measured: Option<Outcome>,
	/// The player stopped for felt unwell at this unix second, on the helper's clock: a banked pump row with the reason,
	/// dated then, locks the game, the helper and every app until the next game day after the stop.
	pub unwell_at: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct CatchupMove {
	pub seconds: u16,
	/// The owed move it ran (its Library.lua id, as `GET /v1/sheet` listed it): the helper pays an owed row of that
	/// move.
	#[serde(rename = "move")]
	pub move_id: u16,
	/// The owed row's time as the unfinished list gave it: a skip's names the row it skips (`Session::skip`), a
	/// catch-up's (since 2026-10-12) the row it was dosed from, which it pays while that row is still owed, before any
	/// move of the pump falls back, else the oldest of the move still free, its `set` then dropped
	/// (`Session::catch_up`). None: the oldest of the move.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub ts: Option<u32>,
	/// A catch-up's (since 2026-10-12): the set whose count the player saved on the device's count step (since slice
	/// 42's fix round, 2026-10-17, also a counted set saved with no count or Form slipped, and a steady move's answered
	/// pace), which the helper writes as the game's own catch-up writes it (`CatchupSet::rows`); none on a pose or a move
	/// done without one, which is written done, as before.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub set: Option<CatchupSet>,
	/// A catch-up's (since 2026-10-26): the move Swap ran in place of the owed one on the device's Ready view, or since
	/// 2026-10-29 the move an owed backup the setup leaves unneeded rides as (Engine.lua `ridesAs`), which pays that
	/// owed row as the game's swapped rider does (`Session::catch_up`); none where the owed move itself ran.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub ran: Option<CatchupRan>,
	/// A `bank`'s (docs/plan.md slice 58): the move's row kind (`codes::kind`), the ladder and the rung its slot is dosed
	/// at (0 none) and whether it is an all-out window, which the banked row repeats (`WristEvent::bank`); a catch-up
	/// and a skip send none, and the helper reads none of theirs.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub kind: Option<u8>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub ladder: Option<u8>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub rung: Option<u8>,
	#[serde(default, skip_serializing_if = "std::ops::Not::not")]
	pub allout: bool,
}

/// The move Swap put in place of a catch-up's owed move (contract/companion-api.yaml `pump_done.moves[].ran`, since
/// 2026-10-26; David: "for me the offered frog stand will be too hard but I cannot siwtch to an easier version"), as
/// its swapped slot names it (Engine.lua `ns.SwapOptions` for an owed slot: its backup or another move at its level,
/// its easier version, or for an all-out window another form of its finisher rung or above), or since 2026-10-29 the
/// move an owed backup of its rung the setup leaves unneeded rides as (Engine.lua `ridesAs`), as its slot names it,
/// Swap's way back to it included: the move, its row kind (`codes::kind`), the ladder and the rung it is dosed at (0
/// none) and whether it is Swap's easier version, whose row carries the easier bit (`flags::EASIER`, Progress.lua
/// `ns.ResultRow`), with the step down its result makes.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CatchupRan {
	#[serde(rename = "move")]
	pub move_id: u16,
	pub kind: u8,
	pub ladder: u8,
	pub rung: u8,
	#[serde(default, skip_serializing_if = "std::ops::Not::not")]
	pub easier: bool,
	/// On Swap's easier version only: the ladder and the rung its result steps down to (Progress.lua `ns.EasierDown`, the
	/// attempt at the rung a placed ladder stands on), with or without a set, as the game's `Lc.WriteResult` writes it
	/// after every outcome; the helper writes it as a rung row of cause 8 after the move's rows (`CatchupRan::down_row`).
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub down: Option<SetStep>,
}

impl CatchupRan {
	/// Owed row `r` as this move ran it (`Lc.WriteResult`'s `ns.ResultRow` of the swapped slot): its move, kind, ladder
	/// and rung; the rest as `r` keeps it.
	fn row(self, r: &Row) -> Row {
		Row { move_id: self.move_id, kind: self.kind, ladder: self.ladder, rung: self.rung, ..*r }
	}

	/// The bit its move row carries beside the owed flag: the easier bit on Swap's easier version, else none.
	fn bits(self) -> u16 {
		if self.easier { flags::EASIER } else { 0 }
	}

	/// The rung row its step down writes after the move's rows (`ns.EasierDown`'s, cause 8, the device's bit): on Swap's
	/// easier version alone, on `ladder` (the owed row's, whose attempt it took) or one of its successors, of
	/// character `char` and dated `ts` as the move row; none otherwise.
	fn down_row(self, ladder: u8, char: u8, ts: u32, bit: u16) -> Option<Row> {
		let down = self.down.filter(|d| self.easier && d.rung > 0 && on_ladder(ladder, d.ladder))?;
		Some(Row {
			ts,
			char,
			kind: kind::RUNG,
			ladder: down.ladder,
			rung: down.rung,
			reps: cause::EASIER,
			flags: bit,
			..Row::default()
		})
	}
}

/// A match pump's Bank all on a phone, a watch or the desktop app (docs/plan.md slice 58; David,
/// 2026-10-26: "I miss the bank (and bank the rest) button after I played a round?"): the moves of its own not done yet
/// go to the game's owed queue as the game's Bank all puts a quest's there (`Session::bank`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BankPump {
	/// The phone, the watch or the computer, whose flag bit its rows carry (never the game's or the app's).
	pub source: Source,
	/// The pump's own id (`f-<uuid>` from a phone, `w-<uuid>` from a watch, `d-<uuid>` from the desktop app), which
	/// its rows keep, so a bank sent again is written once; a short move's, banked alone as it ends, `<pump id>~s<slot>`
	/// (`min_reps`, `min_seconds`), so the pump's own stays free for its Bank all.
	pub id: String,
	/// The pump's start (its offered row's time), unix seconds on the helper's clock: the rows' time.
	pub start: u32,
	/// The pump's size, 1 Light, 2 Balanced, 3 Extreme, which every row carries.
	pub tier: u8,
	/// The rows' questID: 0 for a computer's own bank, a bank number for one of the account's unfinished list made on
	/// a device without a computer (`bank_quest`, slice 67).
	pub quest: u32,
	/// The moves banked, in the pump's order.
	pub moves: Vec<BankMove>,
}

/// The bank number of a pump a device without a computer banks (the browser, a phone not linked to the desktop app;
/// contract/sync-api.yaml, "The account's unfinished list", slice 67): `bank_quest_min` plus the first eight hex digits
/// of the pump id's UUID (after its `<letter>-`) modulo `bank_quest_min`, so two devices' banks of one second never
/// share an owed key. None for an id of another shape or when the constants do not read.
#[must_use]
pub fn bank_quest(pump_id: &str) -> Option<u32> {
	let min = library::constants()?.bank_quest_min;
	let (letter, uuid) = pump_id.split_once('-')?;
	let hex = uuid.get(..8).filter(|h| letter.len() == 1 && h.bytes().all(|b| b.is_ascii_hexdigit()))?;
	let n = u32::from_str_radix(hex, 16).ok()?;
	min.checked_add(n % min.max(1))
}

/// Whether a game row is one of the account's unfinished list (slice 67): an owed, banked or paid move row whose
/// questID is a bank number, which every device folds whichever device wrote it.
#[must_use]
pub fn is_account_bank(row: &Row) -> bool {
	kind::is_move(row.kind)
		&& matches!(row.outcome, outcome::OWED | outcome::BANKED | outcome::PAID)
		&& library::constants().is_some_and(|c| row.quest_id >= c.bank_quest_min)
}

/// A move a bank puts off, as its pump dosed it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BankMove {
	pub move_id: u16,
	/// The row kind (`codes::kind`): a rep, hold, cardio or flow move.
	pub kind: u8,
	pub ladder: u8,
	pub rung: u8,
	/// A timed move's (cardio, flow) planned seconds; the row keeps none on any other kind.
	pub seconds: u16,
	/// An all-out window, owed as its window (`reps` 1) under its own move, a counted set's too, never with a count.
	pub allout: bool,
}

/// The set of a catch-up's rep move, hold or counted set (contract/companion-api.yaml `WristEvent`'s `set`, since
/// 2026-10-12; David: it "did not ask me how many I could do"), and since slice 42's fix round (2026-10-17) of a counted
/// set saved Yes, to the end with no count or Form slipped (or, since slice 53, Not all out) and of a steady cardio
/// move whose pace the device asked (or, since slice 52, that Keep going ran past its plan): the device dosed the owed
/// move as the game doses one (Engine.lua `riderSlot`, at the ladder and rung the unfinished list gave; a steady move's
/// pace through `paceSlot`), asked the count (the form, the pace, whether a hold could have gone longer) with the
/// game's limits and judged it as `Lc.WriteResult` does (`ns.Judge`, `ns.ResultRow`, `ns.LoadRow`, `ns.Progression`):
/// the move row's reps, seconds, outcome and judge's bits, the weight, the moment it was saved, the rung rows the judge
/// added and the steps it mastered, all of the owed move itself (an owed window is its own move's since 2026-10-15,
/// never a form picked again), or since 2026-10-26 of the move run in its place (`CatchupMove::ran`: Swap's, or since
/// 2026-10-29 an owed backup's stand-in; it carries the step down Swap's easier version makes, never the set).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct CatchupSet {
	/// A rep move's or a counted set's typed count; on a hold its answer to "Could you have held longer?"
	/// (`codes::longer`, slice 52; 0 not asked).
	pub reps: u16,
	pub seconds: u16,
	/// The judged outcome: a typed count's (`outcome::is_typed`), or done on a counted set saved with no count or Form
	/// slipped and on a steady move's answered pace (since slice 42's fix round).
	pub outcome: u8,
	/// The judge's own bits on the move row (`flags::JUDGED`: confirmed and, on a cardio row, counted and its answer:
	/// a counted set's Form slipped since 2026-10-16 and Not all out since slice 53, a steady move's pace since slice
	/// 42's fix round, `codes::pace` since slice 52); the helper keeps these alone.
	pub flags: u16,
	/// A loaded set's weight in tenths of a kg: its load row comes before the move row.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub load: Option<u16>,
	/// When the count was saved, unix ms on the helper's clock: the time of the set's rows.
	pub at: i64,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub rungs: Vec<SetRung>,
	/// The steps the set mastered (since slice 42's fix round, 2026-10-17: `ns.Progression`'s mastered rows), each
	/// written as a mastered row after the rung rows.
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub mastered: Vec<SetStep>,
}

/// A rung row a catch-up's set brings (`ns.Progression`'s): the ladder, the rung and its cause (Progress.lua
/// `ns.RungCause`).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetRung {
	pub ladder: u8,
	pub rung: u8,
	pub cause: u16,
}

/// A ladder's step a catch-up names: one its set mastered (`ns.Progression`'s mastered row, kind 19), or the rung Swap's
/// easier version steps down to (`CatchupRan::down`): its ladder and rung.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetStep {
	pub ladder: u8,
	pub rung: u8,
}

impl CatchupSet {
	/// Whether the set is one the owed row `r` asks: a typed count (`outcome::is_typed`) of a rep move or a hold, with
	/// no answer in the flags, a hold's `reps` its answer to "Could you have held longer?" (`codes::longer`, 1 to 3,
	/// slice 52) on a ladder of the Stamina Ledger (`library::hold_goal`, or a retired ladder whose successor is one),
	/// else 0; the counted set of an all-out window (its counted bit), a typed count or, since slice 42's fix round
	/// (2026-10-17), done (saved Yes, to the end with no count, or Form slipped or, since slice 53, Not all out, its
	/// answer in the flags: `flags::answer` 0 to 2); a steady cardio move's answered pace (done, its pace answer in the
	/// flags, `codes::pace` 1 to 3, no counted bit). Any other set (an outcome done on a rep move or a hold, a counted
	/// bit on a steady move, an answer on a rep move or past 2 on a counted set, a hold's reps past 3 or above 0 off the
	/// Stamina Ledger's ladders) is ignored and the move is written done, as a move without one.
	fn fits(&self, r: &Row) -> bool {
		let (counted, answer) = (self.flags & flags::COUNTED != 0, flags::answer(self.flags));
		let ledger = |l: u8| library::hold_goal(l) || library::successors(l).iter().any(|&s| library::hold_goal(s));
		let asked = self.reps == 0 || (self.reps <= longer::LOT && ledger(r.ladder));
		match r.kind {
			kind::REP => outcome::is_typed(self.outcome) && !counted && answer == 0,
			kind::HOLD => outcome::is_typed(self.outcome) && !counted && answer == 0 && asked,
			kind::CARDIO if all_out(r) => {
				counted
					&& answer <= flags::answer(flags::NOT_ALL_OUT)
					&& (outcome::is_typed(self.outcome) || self.outcome == outcome::DONE)
			}
			kind::CARDIO => !counted && answer != 0 && self.outcome == outcome::DONE,
			_ => false,
		}
	}

	/// When its rows are dated: the moment the count was saved, kept between the pump's `start` and `now` (unix
	/// seconds).
	fn ts(&self, start: u32, now: i64) -> u32 {
		let at = self.at.div_euclid(1000).clamp(i64::from(start), now.max(i64::from(start)));
		u32::try_from(at).unwrap_or(start)
	}

	/// The rows of the set in the order the game's catch-up writes them (Events.lua `Lc.WriteResult`), from the move
	/// row a move without one writes (`done`: the owed row as the store keeps it, its character, quest, tier, ladder,
	/// rung as it reads, move and kind, with its slot in the pump, the device's bit and the owed flag), as
	/// `ns.ResultRow` writes the rider slot's: a loaded rep set's load row (`ns.LoadRow`, the device's bit) when it
	/// counted one or more, the move row with the set's reps (a hold's its answer to "Could you have held longer?",
	/// slice 52; none until then), seconds, outcome and the judge's bits it keeps (`flags::JUDGED`, counted and the
	/// answer on a cardio row alone), then each rung row of a ladder that is its own (`on_ladder`) with a cause
	/// its rules give (`cause::after_set`), then each step it mastered on such a ladder as a mastered row (kind 19, its
	/// ladder and rung, the device's bit; since slice 42's fix round); all dated the moment the count was saved
	/// (`CatchupSet::ts`).
	fn rows(&self, done: &Row, bit: u16, start: u32, now: i64) -> Vec<Row> {
		let ts = self.ts(start, now);
		let keep =
			if done.kind == kind::CARDIO { flags::JUDGED } else { flags::JUDGED & !(flags::COUNTED | flags::ANSWER) };
		let set = Row {
			ts,
			reps: self.reps,
			seconds: self.seconds,
			outcome: self.outcome,
			flags: done.flags | (self.flags & keep),
			..*done
		};
		let load = self.load.filter(|&l| l > 0 && set.reps > 0 && done.kind == kind::REP).map(|load| Row {
			kind: kind::LOAD,
			reps: load,
			seconds: 0,
			outcome: 0,
			flags: bit,
			..set
		});
		let own = |l: u8| on_ladder(set.ladder, l);
		let rungs =
			self.rungs.iter().filter(|g| own(g.ladder) && g.rung > 0 && cause::after_set(g.cause)).map(|g| Row {
				ts,
				char: set.char,
				kind: kind::RUNG,
				ladder: g.ladder,
				rung: g.rung,
				reps: g.cause,
				flags: bit,
				..Row::default()
			});
		let mastered = self.mastered.iter().filter(|m| own(m.ladder) && m.rung > 0).map(|m| Row {
			ts,
			char: set.char,
			kind: kind::MASTERED,
			ladder: m.ladder,
			rung: m.rung,
			flags: bit,
			..Row::default()
		});
		load.into_iter().chain([set]).chain(rungs).chain(mastered).collect()
	}
}

/// Whether ladder `l` (never 0) is `base` or one of a retired `base`'s successors (`library::successors`), as a device
/// doses an owed row the way the fold reads it.
fn on_ladder(base: u8, l: u8) -> bool {
	l > 0 && (l == base || library::successors(base).contains(&l))
}

/// An owed cardio row that is an all-out window of its own move (Engine.lua `isFinisherRow`): reps 1, or reps 2, which
/// an addon from before 2026-10-15 wrote for a closing move under the jog that led it, now a window of that jog.
fn all_out(r: &Row) -> bool {
	r.kind == kind::CARDIO && (r.reps == 1 || r.reps == 2)
}

/// The plain owed rows `plain` (oldest first) in the order a catch-up plays them (Engine.lua `spreadRiders`; Rapha,
/// 2026-10-28; David, 2026-11-01): oldest first, except that a move owed twice (its move as owed now; the game and the
/// phones count an owed backup as the move it rides as, Engine.lua `ridesAs`, which this list cannot tell) comes again
/// only once every other move among them has come, and none comes straight after an effort of its own body part
/// (`library::part` of its move as owed now: a rep or hold move's region, else its pattern) while another can go there.
/// At each place the oldest of the first of these that holds one: a move not placed yet whose part differs from the
/// effort just before it, a move not placed yet, a part that differs, any; the effort before the first is the move
/// `before` (the player's latest set). Nothing is dropped.
fn spread(plain: Vec<Row>, before: Option<u16>) -> Vec<Row> {
	let mut left = plain;
	let mut out = Vec::with_capacity(left.len());
	let mut placed: Vec<u16> = Vec::new();
	let mut last = before.and_then(library::part);
	while !left.is_empty() {
		let rank = |r: &Row| {
			let id = owed_now(r).move_id;
			u8::from(placed.contains(&id)) * 2 + u8::from(last.is_some() && library::part(id) == last)
		};
		let at = left.iter().enumerate().min_by_key(|&(i, r)| (rank(r), i)).map_or(0, |(i, _)| i);
		let r = left.remove(at);
		let id = owed_now(&r).move_id;
		last = library::part(id);
		placed.push(id);
		out.push(r);
	}
	out
}

/// The owed row each move of a catch-up pays (its place in `queue`, the owed rows as owed now in the order a catch-up
/// pays them), and whether it is the row the move names (`ts`): first every move that names a row takes it while it is
/// still owed, then every other move (its named row went, or it named none) the oldest row of its move still free, so
/// a move that falls back never takes the row a later move was dosed from; every all-out window the catch-up ran pays
/// its row, as every owed window rides the game's next pump (David, 2026-10-14: "It should just be all coming right
/// after the pump and when im done I can always bank the rest"). A move that falls back takes a window's row only
/// while no move named one: a device that names rows names every window it ran, so the plain move whose row the game
/// paid meanwhile never pays a window owed under the same move. None: the move pays nothing.
fn paid_rows(moves: &[CatchupMove], queue: &[Row]) -> Vec<Option<(usize, bool)>> {
	let mut picks = vec![None; moves.len()];
	let mut used = vec![false; queue.len()];
	let mut named_window = false;
	for named in [true, false] {
		for (k, m) in moves.iter().enumerate() {
			if picks[k].is_some() || (named && m.ts.is_none()) {
				continue;
			}
			let free = |i: &usize| {
				!used[*i] && m.move_id == queue[*i].move_id && (named || !named_window || !all_out(&queue[*i]))
			};
			let found = if named {
				(0..queue.len()).filter(free).find(|&i| m.ts == Some(queue[i].ts))
			} else {
				(0..queue.len()).find(free)
			};
			let Some(i) = found else { continue };
			used[i] = true;
			named_window |= named && all_out(&queue[i]);
			picks[k] = Some((i, named));
		}
	}
	picks
}

/// A row of the owed queue (owed or banked) as it is owed now, by the fold's two rules for the move it names (DB.lua
/// `ns.Translate`'s `mergedMove` and `timedNow`). A merged move's names the move that took it (`moves::current`), the
/// id the unfinished list, the game's `held` list and the devices' catch-ups and skips name. One of reps or a hold
/// whose move runs by its time now (`library::flow`: the carries, the good morning, since 2026-11-11 the bird dog and
/// its kin, which reach no form failure) is the row a bank of that move writes today: a flow row at the ladder and
/// rung a fresh pick of it carries and at the move's own seconds, a side move's on each side. The unfinished list, a
/// catch-up and its payment take it as the timed move, never as a set one step past the record its old sets left. The
/// rest as kept; a set done is never read here and stays as written.
#[must_use]
pub fn owed_now(r: &Row) -> Row {
	let now = Row { move_id: moves::current(r.move_id), ..*r };
	match library::flow(now.move_id) {
		Some(f) if now.kind == kind::REP || now.kind == kind::HOLD => {
			Row { kind: kind::FLOW, ladder: f.ladder, rung: f.rung, seconds: f.seconds, ..now }
		}
		_ => now,
	}
}

impl OwedMove {
	/// An owed row as the unfinished list shows it (`Session::payable`'s rows): a cardio or flow move's planned seconds,
	/// the ladder and rung it was owed at, and whether it is an all-out window. A window's seconds are at most the
	/// longest window (`library::allout_max_s`, 40 s), as the game doses an owed window from its go (Engine.lua
	/// `riderSlot`): a closing row an addon from before 2026-10-15 owed under its jog kept the jog's length and more,
	/// and the device builds from before then cap a window only when it is a closing move's.
	#[must_use]
	pub fn of(r: &Row) -> Self {
		let timed = r.kind == kind::CARDIO || r.kind == kind::FLOW;
		let allout = all_out(r);
		let cap = if allout { library::allout_max_s().unwrap_or(u16::MAX) } else { u16::MAX };
		Self {
			move_id: r.move_id,
			kind: r.kind,
			seconds: Some(r.seconds.min(cap)).filter(|&s| timed && s > 0),
			ladder: Some(r.ladder),
			rung: Some(r.rung),
			allout,
			ts: Some(r.ts),
		}
	}
}

/// A row an app wrote, as the store of app rows keeps it: the app bit beside exactly one device's (the phone's, the
/// watch's or, docs/plan.md slice 35, the computer's), no character, and no reset (the floor is the game's alone: `/qp
/// reset all`). A row with two devices' bits is none: a phone that stamped a computer row it took in from the helper's
/// copy with its own bit too would make a second text of one pump, which every fold would count twice.
fn app_row(r: &Row) -> bool {
	Source::from_flags(r.flags) == Source::App
		&& (r.flags & (flags::PHONE | flags::WATCH | flags::COMPUTER)).is_power_of_two()
		&& r.char == 0
		&& r.kind != kind::RESET
}

/// A phone's "Start over in the app" as it sends it (docs/plan.md slice 28; kept since 2026-10-09): a reset row with
/// the app bit beside the phone's, no character. The store of app rows keeps it and serves it to every phone, the
/// mirror never carries it (the game keeps what it has: `mirrored`), and the app's own history starts at the latest one
/// (`app_rows::after_start_over`).
fn start_over_row(r: &Row) -> bool {
	r.kind == kind::RESET
		&& r.char == 0
		&& r.flags & (flags::APP | flags::PHONE | flags::WATCH | flags::COMPUTER) == flags::APP | flags::PHONE
}

/// Whether a row may reach a store at `now` (unix seconds, the helper's clock): every row the helper keeps comes from
/// outside its own writing (the strip, a save or a mirror taken in, a phone, a backup file, another computer's batch),
/// so each is held to `library::row_timely`; one refused is counted for the log (`Effects::untimely`).
fn timely(row: &Row, now: i64, fx: &mut Effects) -> bool {
	let timely = library::row_timely(row, now);
	fx.untimely += usize::from(!timely);
	timely
}

/// Two lists of rows as one, ordered as the addon's merge orders two logs (`DB.lua` `ns.MergeMirror`, `takeAll`): each
/// row at its list's latest time so far (a paid row or a correction that repeats an older time stays after the rows
/// written before it), then by that time, the first list's rows first where two times are equal.
fn by_time(first: Vec<Row>, second: Vec<Row>) -> Vec<Row> {
	let mut timed = Vec::with_capacity(first.len() + second.len());
	for list in [first, second] {
		let mut at = 0;
		for row in list {
			at = at.max(row.ts);
			timed.push((at, row));
		}
	}
	timed.sort_by_key(|&(at, _)| at); // stable: each list keeps its order, and the first comes first at a tie
	timed.into_iter().map(|(_, row)| row).collect()
}

/// What a sync's app rows came to (`Session::take_app_rows`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AppTaken {
	pub kept: usize,
	pub dropped: usize,
	/// A row `app.jsonl` could not take: the sync's response confirms none (no `rows_store`), so the phone sends them
	/// again.
	pub unwritten: bool,
}

/// What `Session::keep_app` did with an app row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppKept {
	New,
	/// That copy is kept already, or it is dated before the latest reset.
	Known,
	/// `app.jsonl` could not take a row that comes again (a phone's, a file's): not kept.
	Unwritten,
	/// Dated where no row may be (`library::row_timely`): dropped for good, as one before the reset is.
	Untimely,
}

/// Why a catch-up pump, a skip or a bank wrote nothing.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CatchupError {
	#[error("pumps are locked until {0} (felt unwell)")]
	Locked(i64),
	/// No move of it is owed (a catch-up's, a skip's), or a bank named no move.
	#[error("no move is owed")]
	NothingOwed,
	/// The pump's rows are stored already: a device resends a pump until the helper confirms it.
	#[error("the pump {0} is paid already")]
	AlreadyPaid(String),
	/// The pump started before the save was reset: it belongs to the deleted save.
	#[error("the pump started before the last reset")]
	BeforeReset,
	/// The pump's time is one no row may carry (`library::row_timely`: more than a day past the helper's clock).
	#[error("the pump's time is past the helper's clock")]
	Untimely,
}

/// The computer's time zone as the session reads it: the offset of local time from UTC, in seconds, at a unix second
/// (the app reads it from the system's zone at each instant it asks about, so a lock across a clock change ends as the
/// phones' `UnwellLock` ends it).
pub struct Zone(Box<dyn Fn(i64) -> i64 + Send>);

impl Zone {
	#[must_use]
	pub fn new(offset_at: impl Fn(i64) -> i64 + Send + 'static) -> Self {
		Self(Box::new(offset_at))
	}

	/// A zone with one offset all year.
	#[must_use]
	pub fn fixed(offset_s: i64) -> Self {
		Self::new(move |_| offset_s)
	}

	fn offset_at(&self, t: i64) -> i64 {
		(self.0)(t)
	}

	/// The first second of the game day after the one `t` falls in (`lock.json`'s `ends` and `zoned`; the phones'
	/// `GameDay.next`): the game day is the local date 4 hours earlier, and the next one begins 4 hours after the local
	/// midnight that starts the next date, so 04:00 by the clock, 03:00 or 05:00 on a night the clocks change.
	#[must_use]
	pub fn next_game_day(&self, t: i64) -> i64 {
		self.start_of(self.game_date(t) + 1) + DAY_OFFSET_S
	}

	/// The first second of the game day `t` falls in: 4 hours after its date's first instant.
	#[must_use]
	pub fn game_day_start(&self, t: i64) -> i64 {
		self.start_of(self.game_date(t)) + DAY_OFFSET_S
	}

	/// The local date (days since 1970-01-01) `t` falls on.
	#[must_use]
	pub fn date(&self, t: i64) -> i64 {
		(t + self.offset_at(t)).div_euclid(DAY_S)
	}

	/// The instant a local date's clock shows `minute` minutes after its midnight (the places' schedule): the date's
	/// first instant and the minutes on, moved by a change of the clocks between them, and a minute in an hour the
	/// clocks skip an hour on (as the phones' `LocalDateTime.atZone`).
	#[must_use]
	pub fn at_minute(&self, date: i64, minute: i64) -> i64 {
		let start = self.start_of(date);
		let at = start + minute * 60;
		let moved = at - (self.offset_at(at) - self.offset_at(start));
		if self.offset_at(moved) == self.offset_at(at) { moved } else { at }
	}

	/// The local date (days since 1970-01-01) of the game day `t` falls in: the date 4 hours earlier.
	fn game_date(&self, t: i64) -> i64 {
		let shifted = t - DAY_OFFSET_S;
		(shifted + self.offset_at(shifted)).div_euclid(DAY_S)
	}

	/// The first instant of a local date (days since 1970-01-01): its midnight, the earlier one where the clocks went
	/// back across it, or the end of the skipped hour where they went forward across it (`Calendar.startOfDay`,
	/// `LocalDate.atStartOfDay`).
	fn start_of(&self, date: i64) -> i64 {
		let midnight = date * DAY_S;
		// the offsets a day either side: no zone changes its clocks twice within two days
		let (before, after) = (self.offset_at(midnight - DAY_S), self.offset_at(midnight + DAY_S));
		let fits = |offset: i64| {
			let t = midnight - offset;
			(self.offset_at(t) == offset).then_some(t)
		};
		match (fits(before), fits(after)) {
			(Some(a), Some(b)) => a.min(b),
			(Some(t), None) | (None, Some(t)) => t,
			// midnight never shows on the clock: the date begins at the change, the first second on the later offset
			(None, None) => {
				let (mut lo, mut hi) = (midnight - after.max(before), midnight - after.min(before));
				while hi - lo > 1 {
					let mid = lo + (hi - lo) / 2;
					if self.offset_at(mid) == after { hi = mid } else { lo = mid }
				}
				hi
			}
		}
	}
}

/// The helper's state between frames.
pub struct Session {
	store: Store,
	/// The rows the phones' apps sent (slice 26): never paying or owing, folded by the game beside its own.
	app: AppRows,
	/// The other computers' game rows the sync brought (slice 60), folded by the game as rows of another source.
	far: FarRows,
	/// The install the helper follows (its AddOns folder, `set_install`): a load the strip shows from now on is its.
	install: String,
	/// Every sheet read that differs from its character's last one, for the phone's charts (`GET /v1/sheets`).
	sheet_log: SheetLog,
	people: People,
	loads: HashMap<u16, Load>,
	current: Option<u16>,
	assembler: Assembler,
	locked_until: Option<i64>,
	/// The computer's time zone: the game day turns 4 hours after local midnight.
	zone: Zone,
	leaving: bool,
	/// The last capture acted on (load id, sequence, event counter, flags): the capture delivers each paint many times
	/// (the corner's lower rows show the moving game world), and a paint acts once.
	last_paint: Option<(u16, u8, u16, u8)>,
	/// The capture saw the strip without a break since the last paint: a new paint's first capture then came within a
	/// frame of its display. The first capture after the stream starts or the strip comes back into view may show a
	/// paint made up to a second before, which dates no countdown.
	in_sight: bool,
	/// The camera test's correction, in milliseconds (`SocketCommand::DisplayLag`): 0 until it is learnt.
	display_lag_ms: i32,
	/// The heart-rate proofs and the moves waiting for theirs (slice 8).
	book: Book,
	/// The walking pad's distance, for the stretch rows' `pad_metres` (slice 25).
	pad: PadOdometer,
	/// Each install's newest save the helper read since it started (`SaveBase`), by the install's key: the baseline the
	/// engine's fold of that save starts from (`game_save`).
	bases: HashMap<String, SaveBase>,
}

impl Session {
	#[must_use]
	pub fn new(store: Store, people: People, zone: Zone) -> Self {
		let mut s = Self {
			store,
			app: AppRows::in_memory(),
			far: FarRows::in_memory(),
			install: String::new(),
			sheet_log: SheetLog::in_memory(),
			people,
			loads: HashMap::new(),
			current: None,
			assembler: Assembler::default(),
			locked_until: None,
			zone,
			leaving: false,
			last_paint: None,
			in_sight: false,
			display_lag_ms: 0,
			book: Book::default(),
			pad: PadOdometer::default(),
			bases: HashMap::new(),
		};
		s.locked_until = s.lock_from_rows();
		s
	}

	/// Every stored row that locks, the game's and the app's.
	fn locking_rows(&self) -> impl Iterator<Item = Row> {
		let game = self.store.records().iter().map(RowRecord::row);
		game.chain(self.app.records().iter().map(RowRecord::row)).filter(Row::locks)
	}

	/// The felt-unwell lock the stored rows set (after a load or a reset).
	fn lock_from_rows(&self) -> Option<i64> {
		self.locking_rows().map(|r| self.lock_end(r.ts)).max()
	}

	/// A stored row that felt unwell locks every pump until the next game day after it.
	fn extend_lock(&mut self, row: &Row) {
		if row.locks() {
			self.lock_after(row.ts);
		}
	}

	/// Felt unwell at `ts` (unix seconds, the helper's clock): every pump locks until the next game day after it, the
	/// lock the stop's row sets once it is stored. The desktop app's own stop takes it at once (docs/plan.md slice 35),
	/// before its engine has written that row; a reset, which reads the lock from the stored rows again, keeps it only
	/// through that row.
	pub fn lock_after(&mut self, ts: u32) {
		let end = self.lock_end(ts);
		self.locked_until = Some(self.locked_until.map_or(end, |t| t.max(end)));
	}

	#[must_use]
	pub fn store(&self) -> &Store {
		&self.store
	}

	/// The app rows the phones sent (`rows_store`, the mirror, `GET /v1/app_rows`).
	#[must_use]
	pub fn app_rows(&self) -> &AppRows {
		&self.app
	}

	/// The app rows read from the data folder, at the start.
	pub fn set_app_rows(&mut self, app: AppRows) {
		self.app = app;
		self.locked_until = self.lock_from_rows();
	}

	/// The other computers' game rows (slice 60).
	#[must_use]
	pub fn far_rows(&self) -> &FarRows {
		&self.far
	}

	/// The other computers' game rows read from the data folder, at the start.
	pub fn set_far_rows(&mut self, far: FarRows) {
		self.far = far;
	}

	/// A `rows` batch's game rows from the computer `hid` with its characters' GUIDs by index (`chars`, slice 60): its
	/// latest reset first, which is the player's (contract/sync-api.yaml, "The batch"), taken as this computer's own
	/// through `keep` (every store and every install's game reset as slice 26 resets them: the mirror carries the reset
	/// row) and kept in the far store as one that came in, so it is never sent again; then a GUID this helper does not
	/// know yet made a character of its own (its rows reach every mirror at an index of its own: `bind_all`; its
	/// computer's `device` batch names it later: `far_characters`); then each row kept once by its text, character and
	/// copy number from the store's latest reset on (`FarRows::take`), and the mirror due at once when one is new; a row
	/// dated where none may be (`timely`) left out first, the reset among them. The
	/// number kept, and whether `far.jsonl` took them (`Effects::store_errors` names the file otherwise).
	pub fn take_far(
		&mut self,
		hid: &str,
		rows: &[GameRow],
		chars: &BTreeMap<u8, String>,
		now: &Now,
	) -> (usize, bool, Effects) {
		let mut fx = Effects::default();
		// a row dated where none may be never reaches the far store, nor its reset the stores' floor
		let rows: Vec<GameRow> = rows.iter().copied().filter(|r| timely(&r.row(), now.unix, &mut fx)).collect();
		let rows = rows.as_slice();
		let reset = rows.iter().map(GameRow::row).filter(|r| r.kind == kind::RESET).max_by_key(|r| r.ts);
		if let Some(reset) = reset.filter(|r| self.store.accepts(r)) {
			let row = Row { char: 0, ..reset };
			self.add(RowRecord::new(&row, &now.rfc3339, Source::Game), now.unix, &mut fx);
		}
		for guid in chars.values() {
			if self.people.by_guid(guid).is_none()
				&& let Some(id) = self.new_id()
			{
				let c = Character { guid: guid.clone(), name: None, realm: None, class: None };
				self.people.chars[usize::from(id) - 1] = Some(c);
				fx.people_changed = true;
			}
		}
		let (kept, written) = self.far.take(hid, rows, chars, self.store.floor());
		if let Err(e) = &written {
			fx.store_errors.push(format!("{}: {e}", far::FILE));
		}
		if kept > 0 || fx.people_changed {
			fx.due(MirrorDue::Now);
		}
		(kept, written.is_ok(), fx)
	}

	/// The account's unfinished list in another device's rows batch, whichever device wrote it (slice 67,
	/// contract/sync-api.yaml "The account's unfinished list"): each owed, banked or paid row of a bank number
	/// (`is_account_bank`) this store lacks, taken in at character 0 as its maker's (by its device bit; a game's as
	/// another computer's), so it rides this device's pumps, reaches the game through the mirror and, paid anywhere,
	/// settles its owed row; a row dated where none may be is skipped (`keep`). Its maker alone sends it (the desk's
	/// `rows_due`). The rows taken.
	pub fn take_account_banks(&mut self, rows: &[GameRow], now: &Now) -> (usize, Effects) {
		let mut fx = Effects::default();
		let mut held: HashSet<Row> = self.store.records().iter().map(RowRecord::row).filter(is_account_bank).collect();
		let mut taken = 0;
		for row in rows.iter().map(GameRow::row).filter(is_account_bank) {
			let row = Row { char: 0, ..row };
			if !held.insert(row) {
				continue;
			}
			// a game's row (no device bit) is another computer's here, never this computer's game's
			let source = match Source::from_flags(row.flags) {
				Source::Game => Source::Computer,
				other => other,
			};
			let record = RowRecord::new(&row, &now.rfc3339, source);
			taken += usize::from(self.add(record, now.unix, &mut fx) != Added::Skipped);
		}
		if taken > 0 {
			fx.due(MirrorDue::Now);
		}
		(taken, fx)
	}

	/// Another computer's characters as its `device` batch names them (slice 60): each by its index in that computer's
	/// numbering, kept with its GUID (`People::far`, how an older desktop app's rows without `chars` find theirs), one
	/// with a GUID this helper does not know yet made the helper's own, so its rows reach every mirror at an index of
	/// its own (`bind_all`), and one a rows batch named by its GUID alone given its name, realm and class; one without a
	/// GUID names none.
	pub fn far_characters(&mut self, hid: &str, characters: &[SheetCharacter]) -> Effects {
		let mut fx = Effects::default();
		let named: Vec<(u8, Character)> = characters
			.iter()
			.filter_map(|c| {
				let guid = c.guid.clone()?;
				Some((
					c.index,
					Character { guid, name: c.name.clone(), realm: c.realm.clone(), class: c.class.clone() },
				))
			})
			.collect();
		for (_, c) in &named {
			match self.people.by_guid(&c.guid) {
				None => {
					if let Some(id) = self.new_id() {
						self.people.chars[usize::from(id) - 1] = Some(c.clone());
						fx.people_changed = true;
					}
				}
				Some(id) => {
					let Some(Some(known)) = self.people.chars.get_mut(usize::from(id) - 1) else { continue };
					if known.name.is_none() && c.name.is_some() {
						(known.name, known.realm, known.class) = (c.name.clone(), c.realm.clone(), c.class.clone());
						fx.people_changed = true;
					}
				}
			}
		}
		let kept = self.people.far.entry(hid.to_owned()).or_default();
		for (index, c) in named {
			if kept.get(&index) != Some(&c) {
				kept.insert(index, c);
				fx.people_changed = true;
			}
		}
		if fx.people_changed {
			fx.due(MirrorDue::Now);
		}
		fx
	}

	/// The other computers' rows from the `from`-th kept on, as this computer folds them (`far::as_app`), each
	/// character as `place` gives the helper's character its GUID names (none: not placed yet, the row left out until a
	/// `device` batch names it), a row of no character as 0; the GUIDs read once a call. Left out: a row dated before
	/// the store's latest reset, a reset (the store keeps it as this computer's), a stretch row whose turn-in opened no
	/// pump there (every move of it owed, which that computer keeps: its entry in the game's Log would hold none), and a
	/// row that is, its device bits aside, one of this computer's own game rows (its data folder came back from its own
	/// batches under a new install id).
	fn far_placed(&self, from: usize, place: impl Fn(u8) -> Option<u8>) -> Vec<Row> {
		let all = self.far.records();
		let Some(records) = all.get(from..).filter(|r| !r.is_empty()) else { return Vec::new() };
		let floor = self.store.floor();
		let ids: HashMap<&str, u8> = (1..=u8::MAX)
			.zip(&self.people.chars)
			.filter_map(|(id, c)| Some((c.as_ref()?.guid.as_str(), self.people.resolve(id))))
			.collect();
		let id_of = |r: &far::FarRecord, row: &Row| -> Option<u8> {
			if row.char == 0 {
				return Some(0);
			}
			let guid = match &r.guid {
				Some(guid) => guid.as_str(),
				None => self.people.far.get(&r.hid)?.get(&row.char)?.guid.as_str(),
			};
			ids.get(guid).copied()
		};
		let pumped: HashSet<(far::Who, u32)> = all
			.iter()
			.filter_map(|r| Some((r, r.parsed()?)))
			.filter(|(_, row)| kind::is_pump(row.kind) && row.outcome == outcome::OFFERED)
			.map(|(r, row)| (r.who(&row), row.ts))
			.collect();
		let own: HashSet<Row> = self
			.store
			.records()
			.iter()
			.map(|r| far::bare(&Row { char: self.people.resolve(r.char), ..r.row() }))
			.collect();
		records
			.iter()
			.filter_map(|r| {
				let row = r.parsed()?;
				if row.kind == kind::RESET || floor.is_some_and(|f| row.ts < f) {
					return None;
				}
				if row.kind == kind::STRETCH && !pumped.contains(&(r.who(&row), row.ts)) {
					return None;
				}
				let id = id_of(r, &row)?;
				if own.contains(&far::bare(&Row { char: id, ..row })) {
					return None;
				}
				let char = if id == 0 { 0 } else { place(id)? };
				Some(far::as_app(&row, char))
			})
			.collect()
	}

	/// The other computers' rows from the `from`-th kept on in the helper's numbering of the characters (`GET
	/// /v1/history`'s), as the desktop app's engine folds them beside the app rows, rows of another source as every
	/// mirror carries them.
	#[must_use]
	pub fn far_history(&self, from: usize) -> Vec<Row> {
		self.far_placed(from, Some)
	}

	/// `far_history` as kept records of the app's source, for the desktop app's readers of the stores (the Saga, Pump
	/// home's days, Hero's charts), each beside the app rows.
	#[must_use]
	pub fn far_records(&self, from: usize) -> Vec<RowRecord> {
		self.far_history(from).iter().map(|row| RowRecord::new(row, "", Source::App)).collect()
	}

	/// The install the helper follows from now on (the running game's AddOns folder, else the folder it writes the
	/// mirror into): each load the strip shows next is that install's. The first install the helper follows takes the
	/// characters and profiles kept before slice 26, when the helper knew one install, each character at its own index.
	pub fn set_install(&mut self, install: &str) -> Effects {
		let mut fx = Effects::default();
		self.ensure_install(install, &mut fx);
		install.clone_into(&mut self.install);
		fx
	}

	/// The install's entry, made when it is new. The first one takes the people kept before slice 26: every character
	/// and every index a stored row or sheet names, bound to itself, and the profiles, so the rows of the folder the
	/// helper wrote until then stay the same strings.
	fn ensure_install(&mut self, install: &str, fx: &mut Effects) {
		if self.people.installs.contains_key(install) {
			return;
		}
		let mut entry = Install::default();
		if self.people.installs.is_empty() {
			let people = &mut self.people;
			let named = self.store.records().iter().map(|r| r.char).chain(people.sheets.keys().copied()).max();
			let last = named.unwrap_or_default().max(u8::try_from(people.chars.len()).unwrap_or(u8::MAX));
			if people.chars.len() < usize::from(last) {
				people.chars.resize(usize::from(last), None);
			}
			entry = Install {
				chars: (1..=last).map(|i| (i, i)).collect(),
				profiles: std::mem::take(&mut people.profiles),
				profile_keys: std::mem::take(&mut people.profile_keys),
				profiles_at: std::mem::take(&mut people.profiles_at),
			};
		}
		self.people.installs.insert(install.to_owned(), entry);
		fx.people_changed = true;
	}

	/// The install's entry (made when it is new).
	fn install_mut(&mut self, install: &str, fx: &mut Effects) -> &mut Install {
		self.ensure_install(install, fx);
		self.people.installs.entry(install.to_owned()).or_default()
	}

	/// Binds an install's index to one of the helper's characters, and no other index of that install to it.
	fn bind(&mut self, install: &str, index: u8, id: u8, fx: &mut Effects) {
		self.ensure_install(install, fx);
		let People { installs, aliases, .. } = &mut self.people;
		let Some(entry) = installs.get_mut(install) else { return };
		let before = entry.chars.len();
		entry.chars.retain(|&i, &mut other| i == index || resolve(aliases, other) != id);
		let unbound = before - entry.chars.len();
		if entry.chars.insert(index, id) != Some(id) || unbound > 0 {
			fx.people_changed = true;
		}
	}

	/// The helper's character an install's index of a row or a sheet is: its binding, else a new placeholder bound to it
	/// (the index's profile has not come yet) that the profile names later. Index 0 (no character) stays 0, and so does
	/// an index past the helper's 255 characters.
	fn global(&mut self, install: &str, index: u8, fx: &mut Effects) -> u8 {
		if index == 0 {
			return 0;
		}
		// the first install takes the people kept before slice 26 before its bindings are read, whoever calls first
		self.ensure_install(install, fx);
		if let Some(id) = self.people.bound(install, index) {
			return id;
		}
		let Some(id) = self.new_id() else { return 0 };
		self.install_mut(install, fx).chars.insert(index, id);
		fx.people_changed = true;
		id
	}

	/// The helper's character a character of an install (its index, its GUID) is: the one with that GUID, else the
	/// placeholder the index held (now named), else a new one; bound to the index. A placeholder the index held for
	/// another known character (each `/qp reset all` makes one: the new save's index has no profile yet) is filed under
	/// that character and its id freed (`settle_aliases`). `fresh` (a profile event) takes the character's name, realm
	/// and class as they are now.
	fn bind_character(&mut self, install: &str, index: u8, c: &Character, fresh: bool, fx: &mut Effects) -> u8 {
		let held = self.people.bound(install, index).filter(|&id| self.people.placeholder(id));
		if let Some(id) = self.people.by_guid(&c.guid) {
			let merged = held.filter(|&p| p != id);
			if let Some(p) = merged {
				self.people.aliases.insert(p, id);
				fx.people_changed = true;
			}
			let known = &mut self.people.chars[usize::from(id) - 1];
			if fresh && known.as_ref() != Some(c) {
				*known = Some(c.clone());
				fx.people_changed = true;
			}
			self.bind(install, index, id, fx);
			if merged.is_some() {
				self.settle_aliases(fx); // the index is the character's now, so the placeholder is bound nowhere
			}
			return id;
		}
		let Some(id) = held.or_else(|| self.new_id()) else { return 0 };
		self.people.chars[usize::from(id) - 1] = Some(c.clone());
		fx.people_changed = true;
		self.bind(install, index, id, fx);
		id
	}

	/// A new placeholder id (`People::add`): a freed one is taken only when no stored row or logged sheet still names
	/// it.
	fn new_id(&mut self) -> Option<u8> {
		let rows = self.store.records().iter().map(|r| r.char);
		let named: HashSet<u8> = rows.chain(self.sheet_log.records().iter().map(|r| r.char)).collect();
		self.people.add(&named)
	}

	/// Files each alias's rows (`rows.jsonl`), latest sheet and logged sheets (`sheets.jsonl`) under the character it
	/// stands for, and drops the alias once both files are rewritten, which frees its id for the next placeholder
	/// (`new_id`). The rows of the other installs' mirrors that named it then name that character's index there, as
	/// the next write numbers them. An alias whose rewrite failed stays, read as its character everywhere, and is
	/// filed again at the next merge (the files then still hold its rows under it).
	fn settle_aliases(&mut self, fx: &mut Effects) {
		let aliases: Vec<(u8, u8)> = self.people.aliases.keys().map(|&p| (p, self.people.resolve(p))).collect();
		for (p, id) in aliases {
			let rows = self.store.rechar(p, id);
			if let Err(e) = &rows {
				fx.store_errors.push(format!("{}: {e}", store::FILE));
			}
			let sheets = self.sheet_log.rechar(p, id);
			if let Err(e) = &sheets {
				fx.sheet_errors.push(e.to_string());
			}
			if let Some(sheet) = self.people.sheets.remove(&p) {
				self.people.sheets.entry(id).or_insert(sheet);
			}
			if let Some(apps) = self.people.sheet_apps.remove(&p) {
				self.people.sheet_apps.entry(id).or_insert(apps);
			}
			if rows.is_ok() && sheets.is_ok() {
				self.people.aliases.remove(&p);
			}
			fx.people_changed = true;
			fx.due(MirrorDue::Soon);
		}
	}

	/// An install's `/qp reset all`: its save starts again with new indexes and no profile, so its bindings and
	/// profiles go (its characters stay the helper's, found again by GUID; the other installs keep theirs).
	fn forget_install(&mut self, install: &str, fx: &mut Effects) {
		*self.install_mut(install, fx) = Install::default();
		fx.people_changed = true;
	}

	/// Every character of the helper with a GUID that the install has no index for gets the lowest free one, kept from
	/// then on; a placeholder of another install has no GUID to carry and none.
	fn bind_all(&mut self, install: &str, fx: &mut Effects) {
		self.ensure_install(install, fx);
		let numbering = self.people.numbering(install);
		let ids = (1..=self.people.chars.len()).filter_map(|i| u8::try_from(i).ok());
		let unbound: Vec<u8> = ids
			.filter(|&id| !self.people.aliases.contains_key(&id) && !numbering.contains_key(&id))
			.filter(|&id| self.people.chars[usize::from(id) - 1].is_some())
			.collect();
		for id in unbound {
			let entry = self.install_mut(install, fx);
			let Some(free) = (1..=u8::MAX).find(|i| !entry.chars.contains_key(i)) else { return };
			entry.chars.insert(free, id);
			fx.people_changed = true;
		}
	}

	/// The game store's rows in the install's numbering as it stands, each with whether its character is placed there:
	/// each character at its index there, 0 for no character (placed) and 0 for one the install has no index for (not
	/// placed: a placeholder of another install that no GUID has named yet, since `bind_all` gives every named one an
	/// index). The mirror leaves the unplaced rows out until a GUID names their character, but a reset row: written as
	/// 0 they would stay in the save as 0 and come back beside their copies under the character's index once named,
	/// counted twice, while a reset row's character is read nowhere and its second copy at the same time resets
	/// nothing (`DB.lua` `ns.MirrorReset` wants a later time, the store's `accepts` too), so another install hears the
	/// reset at once (`mirrored`). The take-in counts them as 0, so a save's 0 copies an earlier mirror wrote are not
	/// new (`adopt`).
	fn game_rows_in(&self, install: &str) -> Vec<(Row, bool)> {
		let numbering = self.people.numbering(install);
		let index = |id: u8| if id == 0 { Some(0) } else { numbering.get(&self.people.resolve(id)).copied() };
		self.store
			.records()
			.iter()
			.map(|r| {
				let at = index(r.char);
				(Row { char: at.unwrap_or(0), ..r.row() }, at.is_some())
			})
			.collect()
	}

	/// What the install's `State.lua` holds of the session (docs/plan.md slice 26): the game's rows in that install's
	/// numbering (another install's unnamed placeholder's only with its reset row, until a GUID names it:
	/// `game_rows_in`) and the app's rows with the other computers' (slice 60: `far_placed`, each character at its index
	/// here) in their own time order (a stable sort: one phone's older rows the helper
	/// took after another device's newer ones fold before them, as every device folds them, 2026-10-09; a Start over
	/// stays out, the game keeps what it has), merged by time as the addon's merge orders two lists (`by_time`), so the
	/// rows of that install come back as the strings its save holds; its characters, then every other character of the
	/// helper with a GUID at an index of its own (`bind_all`); and that install's profiles only.
	pub fn mirrored(&mut self, install: &str) -> (Mirrored, Effects) {
		let mut fx = Effects::default();
		self.bind_all(install, &mut fx);
		let rows = self.game_rows_in(install).into_iter();
		let game = rows.filter_map(|(row, placed)| (placed || row.kind == kind::RESET).then_some(row)).collect();
		let mut app: Vec<Row> = self.app.rows().into_iter().filter(|r| r.kind != kind::RESET).collect();
		let numbering = self.people.numbering(install);
		app.extend(self.far_placed(0, |id| numbering.get(&id).copied()));
		app.sort_by_key(|r| r.ts); // stable
		let rows = by_time(game, app);
		let entry = self.install_mut(install, &mut fx).clone();
		let last = entry.chars.keys().max().copied().unwrap_or_default();
		let chars = (1..=last)
			.map(|i| {
				let id = self.people.resolve(*entry.chars.get(&i)?);
				self.people.chars.get(usize::from(id).checked_sub(1)?).cloned().flatten()
			})
			.collect();
		(Mirrored { rows, chars, profiles: entry.profiles, profile_keys: entry.profile_keys }, fx)
	}

	#[must_use]
	pub fn sheet_log(&self) -> &SheetLog {
		&self.sheet_log
	}

	/// The sheet log read from the data folder, at the start.
	pub fn set_sheet_log(&mut self, sheet_log: SheetLog) {
		self.sheet_log = sheet_log;
	}

	/// The addon load the strip showed last (its load id), until it ends (a save the client wrote after it, the game's
	/// exit).
	#[must_use]
	pub fn load(&self) -> Option<u16> {
		self.current
	}

	#[must_use]
	pub fn people(&self) -> &People {
		&self.people
	}

	/// The heart-rate proofs (the mirror's `proof` table, the saved book).
	#[must_use]
	pub fn book(&self) -> &Book {
		&self.book
	}

	/// The book saved in the data folder, at the start.
	pub fn set_book(&mut self, book: Book) {
		self.book = book;
	}

	/// A heart sample from a watch's sync or the strap, at `at` ms on the helper's clock (None: a watch window without a
	/// reading).
	pub fn heart(&mut self, source: HrSource, at: i64, bpm: Option<u16>) {
		self.book.heart(source, at, bpm);
	}

	/// The strap turned on or off in the helper's window at `at` (ms on the helper's clock).
	pub fn set_strap(&mut self, on: bool, at: i64) {
		self.book.set_strap(on, at);
	}

	/// A walking pad's Total Distance reading (FTMS 0x2ACD, metres since the pad's own session began): the odometer
	/// adds what the pad walked since its last reading, and the pad's first reading in the load the strip shows starts
	/// that load's stretch. Only the stretch rows' `pad_metres` read it: the pad never changes a pump.
	pub fn pad(&mut self, distance_m: u32) {
		self.pad.read(distance_m);
		if let Some(load) = self.current.and_then(|id| self.loads.get_mut(&id)) {
			load.pad_from.get_or_insert(self.pad.walked);
		}
	}

	/// The walking pad's walk in the game day `now` falls in, as the helper's window shows it: the `pad_metres` of that
	/// day's stretch rows, and the stretch the current load has open, whenever it began.
	#[must_use]
	pub fn pad_today(&self, now: i64) -> PadDay {
		let start = self.zone.game_day_start(now);
		let kept: u64 = self
			.store
			.records()
			.iter()
			.filter(|r| r.kind == kind::STRETCH && i64::from(r.row_ts) >= start)
			.filter_map(|r| r.pad_metres)
			.map(u64::from)
			.sum();
		let metres = |m: u64| u32::try_from(m).unwrap_or(u32::MAX);
		let open = self
			.current
			.and_then(|id| self.loads.get(&id))
			.map(|load| metres(load.pad_from.map_or(0, |from| self.pad.walked.saturating_sub(from))));
		PadDay { kept: metres(kept), open }
	}

	/// The metres the pad counted over the stretch a stretch row of the load closes, when it counted any.
	fn pad_metres(&self, load_id: u16) -> Option<u32> {
		let from = self.loads.get(&load_id)?.pad_from?;
		let metres = self.pad.walked.saturating_sub(from);
		(metres > 0).then(|| u32::try_from(metres).unwrap_or(u32::MAX))
	}

	/// The verdicts that are out at `now` (ms on the helper's clock): a `verdict` line and a `verdict` command for each
	/// move, and the pump's proof saved and in the mirror when it changed.
	pub fn tick(&mut self, now: i64) -> Effects {
		let mut fx = Effects::default();
		let (settled, changed) = self.book.tick(now);
		for s in settled {
			let o = s.outcome;
			fx.lines.push(Line::Verdict {
				pump_id: s.pump_id.clone(),
				row_ts: s.ts,
				slot: s.slot,
				verdict: o.verdict,
				peak: o.peak,
				delta: o.delta,
				source: s.source,
				rise: s.rise,
			});
			fx.commands.push(Command::Verdict {
				pump_id: s.pump_id,
				slot: s.slot,
				verdict: o.verdict,
				peak: o.peak,
				delta: o.delta,
			});
		}
		if changed {
			fx.proof_changed = true;
			fx.due(MirrorDue::Soon);
		}
		fx
	}

	/// The owed rows in the order a catch-up pump pays them: the plain moves oldest first, spread so that none comes
	/// straight after an effort of its own body part while another can go there (`spread`, after the player's latest
	/// set), then every all-out window oldest first, a pump's all-out windows always its last efforts, as in the game
	/// (Engine.lua `pickRiders`, `spreadRiders`).
	/// Each is the row as it is owed now (`owed_now`: a merged move's row is the merged-into move's, the id the game's
	/// `held` list, the devices and their catch-up pumps use, and one of reps or a hold whose move is timed now is that
	/// move's flow row); its paid row repeats the owed row's key, which holds no move.
	#[must_use]
	pub fn payable(&self) -> Vec<Row> {
		self.payable_kept().iter().map(owed_now).collect()
	}

	/// The rows `payable` lists, each as the store keeps it (a merged move's under its own old id, one whose move is
	/// timed now with the kind and ladder it was owed with), which a fold reads as it reads that owed row (DB.lua
	/// `ns.Translate`: the merged-into move, on that move's own rung where the row sat above it; the timed move's flow
	/// row): what the engine doses a catch-up on the computer from (`Engine::catch_up`), and what a catch-up's move row
	/// is written from (`Session::catch_up`).
	#[must_use]
	pub fn payable_kept(&self) -> Vec<Row> {
		let (plain, windows): (Vec<Row>, Vec<Row>) = self
			.store
			.owed()
			.into_iter()
			.filter(|r| !self.people.held.contains(&owed_now(r).move_id))
			.partition(|r| !all_out(r));
		spread(plain, self.latest_set()).into_iter().chain(windows).collect()
	}

	/// The move of the latest performed row the helper holds, the game's or an app's (DB.lua's `lastMove`: the latest
	/// in time, the later one held of two at the same second), as owed now; none before any.
	fn latest_set(&self) -> Option<u16> {
		let game = self.store.records().iter().map(RowRecord::row);
		let rows = game.chain(self.app.records().iter().map(RowRecord::row));
		let floor = self.store.floor().unwrap_or(0);
		rows.filter(|r| r.performed() && r.ts >= floor).max_by_key(|r| r.ts).map(|r| moves::current(r.move_id))
	}

	/// How many app rows the store holds that reach the game (a Start over never does: `mirrored`).
	fn app_mark(&self) -> usize {
		self.app.mirrored_len()
	}

	/// Whether the latest sheet the strip brought of a character (the helper's numbering, `sheet`'s characters) lacks
	/// app rows the helper holds: rows it took after the load that painted the sheet began, which that load never read
	/// (the game reads the mirror at a load's start), so the game's next load folds more than the sheet shows (one live
	/// character, 2026-10-09: the desktop app and `GET /v1/sheet` show the engine's fold of the save then). False for a
	/// sheet from before the helper counted them, and for a character with no sheet.
	#[must_use]
	pub fn sheet_lacks_app_rows(&self, char: u8) -> bool {
		self.people.sheet_apps.get(&char).is_some_and(|&n| self.app_mark() > n)
	}

	/// The save the engine folds the game's sheet of a character (`char`, the helper's numbering; an alias stands for its
	/// character) from as the game folds it at its next load (`GameSave`; `Helper::game_shown`'s fold of the save): the
	/// install it plays in is the one whose profile keys name it (a character that chose a profile other than Default),
	/// else the one the helper follows (every install binds every character the helper knows, `bind_all`, and the
	/// mirrors carry each into the others, so the binding cannot tell); there, the profile it uses with `body`, the body
	/// every device shares, which the mirror carries into it, and that install's newest save's baseline with the
	/// characters that save numbers. A character the helper cannot name (0 among them) takes the followed install's
	/// baseline alone. None before the helper follows an install.
	#[must_use]
	pub fn game_save(&self, char: u8, body: &body::Body) -> Option<GameSave<'_>> {
		let people = &self.people;
		let of = |id: u8| usize::from(people.resolve(id)).checked_sub(1).and_then(|i| people.chars.get(i)?.as_ref());
		let guid = of(char).map(|c| &c.guid);
		let installs = &people.installs;
		let (install, kept) = guid
			.and_then(|g| installs.iter().find(|(_, i)| i.profile_keys.contains_key(g)))
			.or_else(|| installs.get_key_value(&self.install))?;
		let profile = guid.and_then(|g| kept.profiles.get(kept.profile_keys.get(g).map_or("Default", String::as_str)));
		let base = self.bases.get(install);
		Some(GameSave {
			profile,
			body: mirror::body_value(body),
			baseline: base.map(|b| &b.baseline),
			chars: base.map_or_else(Vec::new, |b| guids(&b.chars)),
			row_chars: if base.is_some() {
				(1..=u8::MAX)
					.take(people.chars.len())
					.map(|id| Guid { guid: of(id).map(|c| c.guid.as_str()) })
					.collect()
			} else {
				Vec::new()
			},
		})
	}

	/// `GET /v1/sheet`: the owed moves a catch-up pump can pay (the catch-ups written included) in the order it pays
	/// them, how many the game holds back, the lock, and every character the helper knows across its installs (its own
	/// numbering, the history rows' `char`) with its latest sheet, as JSON. An alias is no character of its own (the
	/// history serves its rows as the character's), and a free id is none at all.
	#[must_use]
	pub fn sheet(&self, now: i64) -> Sheet {
		let people = &self.people;
		let known = u8::try_from(people.chars.len()).unwrap_or(u8::MAX);
		let mut indexes: Vec<u8> = (1..=known).chain(people.sheets.keys().copied()).collect();
		indexes.sort_unstable();
		indexes.dedup();
		let characters = indexes
			.into_iter()
			.filter(|&i| i > 0 && !people.aliases.contains_key(&i) && !people.free(i))
			.map(|i| {
				let c = people.chars.get(usize::from(i) - 1).and_then(Option::as_ref);
				SheetCharacter {
					index: i,
					guid: c.map(|c| c.guid.clone()),
					name: c.and_then(|c| c.name.clone()),
					realm: c.and_then(|c| c.realm.clone()),
					class: c.and_then(|c| c.class.clone()),
					sheet: people.sheets.get(&i).map(blob::to_json),
				}
			})
			.collect();
		let payable = self.payable();
		// the rest of the owed queue is what the game holds back (`payable` leaves only those out), less what its place
		// sets aside, which no list counts
		let aside = self.store.owed().iter().filter(|r| self.people.aside.contains(&owed_now(r).move_id)).count();
		let held = self.store.owed().len() - payable.len() - aside;
		let owed: Vec<OwedMove> = payable.iter().map(OwedMove::of).collect();
		Sheet {
			unfinished: u32::try_from(owed.len()).unwrap_or(u32::MAX),
			held: Some(u32::try_from(held).unwrap_or(u32::MAX)).filter(|&n| n > 0),
			locked_until: self.locked_until(now),
			// the session knows no running client: the runtime says it (`Helper::wow_with_addon`)
			game_asks: false,
			owed,
			characters,
		}
	}

	/// `GET /v1/history`: the page after the `after` rows a device holds, in the store's order, never by the rows' own
	/// time (a save taken in, a late catch-up or a paid row brings rows dated before newer ones, and a page by time
	/// would miss them), each pump row with its heart-rate proof and its character as the sheet lists it (an alias's
	/// rows as the character it stands for). The game's rows and the catch-up rows only: every phone's app rows come
	/// apart, in `GET /v1/app_rows` (`AppRows::page`). With `proofs_since` (unix seconds) also every proof of a pump
	/// dated at or after it: a verdict comes about a minute after the go, so a phone that read the pump row before it
	/// asks again for the last day's. At most `size` rows (`link::page_size`: `HISTORY_PAGE`, or the request's limit).
	#[must_use]
	pub fn history(&self, after: u64, proofs_since: Option<u64>, size: usize) -> HistoryPage {
		let (at, next, more) = link::page_bounds(self.store.records().len(), after, size);
		let mut rows: Vec<RowRecord> = self.history_rows(at.start).take(at.len()).collect();
		for r in &mut rows {
			let pump = kind::is_pump(r.kind) && r.outcome == outcome::OFFERED;
			if let Some(p) = self.book.proof(r.row_ts).filter(|_| pump) {
				r.hr_verdict = Some(p.verdict);
				r.hr_peak = p.peak;
				r.hr_delta = p.delta;
			}
		}
		let proofs = proofs_since.map(|since| {
			// a row time is a u32: a second past its range starts the range at its end, and the filter keeps it exact
			self.book
				.proofs_from(u32::try_from(since).unwrap_or(u32::MAX))
				.filter(|&(row_ts, _)| u64::from(row_ts) >= since)
				.map(|(row_ts, p)| HistoryProof { row_ts, hr_verdict: p.verdict, hr_peak: p.peak, hr_delta: p.delta })
				.collect()
		});
		HistoryPage { store: self.store.id(), next, more, proofs, rows }
	}

	/// The game's rows and the catch-up rows from the `from`-th on, in the store's order, each with its character as the
	/// sheet lists it (an alias's rows as the character it stands for): the rows `GET /v1/history` serves (`history`,
	/// which pages them and adds the pumps' proofs) and the desktop app's engine folds.
	pub fn history_rows(&self, from: usize) -> impl Iterator<Item = RowRecord> + '_ {
		self.store
			.records()
			.get(from..)
			.unwrap_or_default()
			.iter()
			.map(|r| RowRecord { char: self.people.resolve(r.char), ..r.clone() })
	}

	/// Felt unwell locks every pump until this unix time (the start of the next game day after the row), while it
	/// lies ahead.
	#[must_use]
	pub fn locked_until(&self, now: i64) -> Option<i64> {
		self.locked_until.filter(|&t| t > now)
	}

	/// The camera test's correction (the settings' `display_lag_ms`).
	pub fn set_display_lag(&mut self, ms: i32) {
		self.display_lag_ms = ms;
	}

	/// Why a catch-up or a bank that started at `start` pays or owes no move at `now` (unix seconds, the helper's clock):
	/// it started before the latest reset, at a time no row may carry (`library::row_timely`), or inside the felt-unwell
	/// lock (`locked_at`), in that order.
	fn refuses_start(&self, start: u32, now: i64) -> Option<CatchupError> {
		let row = Row { ts: start, kind: kind::PUMP, ..Row::default() };
		if !self.store.accepts(&row) {
			Some(CatchupError::BeforeReset)
		} else if !library::row_timely(&row, now) {
			Some(CatchupError::Untimely)
		} else {
			self.locked_at(start).map(CatchupError::Locked)
		}
	}

	/// The end of the felt-unwell lock a pump starting at `ts` falls in: from a locking row up to the next game day. A
	/// pump that started before the row (earlier that day, the day before) is no pump of the lock.
	fn locked_at(&self, ts: u32) -> Option<i64> {
		self.locking_rows().filter(|r| r.ts <= ts).map(|r| self.lock_end(r.ts)).filter(|&end| end > i64::from(ts)).max()
	}

	/// The end of the lock a row at `ts` sets: the start of the next game day in the computer's zone.
	#[must_use]
	pub fn lock_end(&self, ts: u32) -> i64 {
		self.zone.next_game_day(i64::from(ts))
	}

	/// One decoded frame, painted at `display_time` (seconds on the helper's clock).
	/// The capture lost sight of the strip (its stream started again, the corner showed no strip, the window left the
	/// screen): the next new paint may be old when first seen.
	pub fn lost_sight(&mut self) {
		self.in_sight = false;
		// a load that painted `leaving` has shown its last state: its loading screen hides nothing
		if !self.leaving
			&& let Some(load) = self.current.and_then(|id| self.loads.get_mut(&id))
		{
			load.stale = true;
		}
	}

	pub fn on_frame(&mut self, f: &RawFrame, display_time: f64, now: &Now) -> Effects {
		let mut fx = Effects::default();
		let dated = std::mem::replace(&mut self.in_sight, true);
		let paint = (f.load_id, f.seq, f.event_no, f.flags);
		if self.last_paint == Some(paint) {
			return fx; // the same paint captured again: its first capture carried the display time
		}
		self.last_paint = Some(paint);
		if self.current != Some(f.load_id) {
			if let Some(old) = self.current {
				self.end_load(old, now, &mut fx);
				fx.load_ended = true;
			}
			self.current = Some(f.load_id);
			let apps = self.app_mark();
			let load = self.loads.entry(f.load_id).or_default();
			if load.first_shown.is_none() {
				// a frame the old client painted, still in flight while the helper turns to another client, starts a
				// load of the new install only when its load id is new to the helper
				load.first_shown = Some(display_time);
				load.apps = apps;
				self.install.clone_into(&mut load.install);
			}
		}
		let fields = match &f.body {
			Body::Chunk(chunk) => {
				// a chunk shown again only sets its part again: a copy completes once
				if let Some(packed) = self.assembler.add(f.load_id, f.event, chunk) {
					self.on_blob(f.load_id, f.event, &packed, now, &mut fx);
				}
				return fx;
			}
			Body::Fields(fields) => fields,
		};
		let leaving = fields.state == StripState::Leaving;
		if leaving != self.leaving {
			self.leaving = leaving;
			fx.suspend = Some(leaving);
		}
		let clock = self.unwrap_clock(f.load_id, fields.ts_client);
		let start_at = (fields.start_at != 0).then(|| relative(clock, fields.ts_client, fields.start_at));
		fx.lines.push(Line::Strip(strip_line(f, fields, clock, start_at, display_time)));
		// when the paint reached the glass, on the helper's clock in milliseconds
		let at = round_ms(display_time * 1000.0 - f64::from(self.display_lag_ms));
		if f.is_heartbeat() {
			self.on_heartbeat(f.load_id, fields, at, &mut fx);
			// a move counting down or running whose countdown the helper missed (it started mid-count): the heartbeat
			// repeats the start, so the device still joins at the right digit or runs the move from its go
			if dated && matches!(fields.state, StripState::Countdown | StripState::Running) {
				self.countdown(f, fields, clock, start_at, display_time, &mut fx);
			}
			self.follow_move(f.load_id, fields, at, &mut fx);
			return fx;
		}
		if !self.loads.entry(f.load_id).or_default().events.insert(f.event_no) {
			return fx; // a frame shown again: acted on already (a stored row also dedupes after a restart)
		}
		match f.event {
			StripEvent::Row => self.on_row(f, fields, clock, at, now, &mut fx),
			StripEvent::PumpOffered => self.on_offered(f, fields, &mut fx),
			StripEvent::PumpDone => {
				let load = self.loads.entry(f.load_id).or_default();
				if load.pump.as_ref().is_some_and(|p| p.offered_ts == fields.row_ts) {
					load.pump = None;
				}
				// the pump's end ends its move (a timed last move goes straight on to it), on the device too
				if load.moving.is_some_and(|m| m.ts == fields.row_ts) {
					fx.commands.extend(self.move_done(f.load_id, at));
				}
				// a code a newer addon adds reads as done, as a device reads a command without `ended`
				let ended = PumpEnd::from_code(fields.outcome).unwrap_or_default();
				fx.commands.push(Command::PumpDone { pump_id: pump_id(f.load_id, fields.row_ts), ended });
			}
			// seen first after a break, the Start is dated from the next heartbeat instead, within a second
			StripEvent::CountdownStart if dated => self.countdown(f, fields, clock, start_at, display_time, &mut fx),
			StripEvent::Keep => fx.commands.extend(self.keep_going(f.load_id, fields)),
			StripEvent::TurnIn => fx.commands.push(Command::TurnIn { quest_id: fields.quest_id, tier: f.tier() }),
			StripEvent::QuestComplete => fx.commands.push(Command::QuestComplete { quest_id: fields.quest_id }),
			StripEvent::Ping => fx.pinged = true,
			StripEvent::CountdownStart
			| StripEvent::Heartbeat
			| StripEvent::Leaving
			| StripEvent::Profile
			| StripEvent::Sheet
			| StripEvent::Feedback => {}
		}
		fx
	}

	/// The countdown of the pump a frame names, once per pump and load: its start and go on the helper's clock,
	/// through the paint's display time less the camera test's correction, `start_at_helper = display_time −
	/// display_lag + (start_at_ms − ts_client)`. The move's heart-rate verdict waits from its go, by the frame's kind
	/// and move.
	fn countdown(
		&mut self,
		f: &RawFrame,
		fields: &Fields,
		clock: u64,
		start_at: Option<u64>,
		display_time: f64,
		fx: &mut Effects,
	) {
		let Some(start_at) = start_at.filter(|_| fields.row_ts != 0) else { return };
		let this = (fields.row_ts, fields.slot);
		let load = self.loads.entry(f.load_id).or_default();
		if !load.counted.insert((fields.row_ts, fields.slot, start_at)) {
			return;
		}
		match load.moving {
			// a move still counted ended before this Start (its result never showed: the strip was out of view)
			Some(m) if (m.ts, m.slot) != this => {
				let at = round_ms(display_time * 1000.0 - f64::from(self.display_lag_ms));
				fx.commands.extend(self.move_done(f.load_id, at));
			}
			// the same move Started again from its result view: the game's Back came between, its paint unseen (Back
			// and Start within one paint), so the device drops the set it ended before this one counts down
			Some(m) if m.ended => fx.commands.extend(self.move_back(f.load_id)),
			// none, or the same move counting down again before its result (a side move's other side)
			_ => {}
		}
		self.loads.entry(f.load_id).or_default().moving = Some(Moving { ts: this.0, slot: this.1, ended: false });
		let start_ms = display_time * 1000.0 - f64::from(self.display_lag_ms) + ms_between(clock, start_at);
		let start_at = round_ms(start_ms);
		let go_at = start_at + i64::from(COUNTDOWN_TICKS) * 1000;
		let id = pump_id(f.load_id, fields.row_ts);
		fx.commands.push(Command::Countdown {
			pump_id: id.clone(),
			slot: fields.slot,
			start_at,
			go_at,
			ticks: COUNTDOWN_TICKS,
		});
		fx.proof_changed |= self.book.start(proof::Move {
			pump_id: id,
			ts: fields.row_ts,
			slot: Some(fields.slot),
			class: proof::class_of(f.kind),
			after_set: moves::after_set(fields.move_id),
			go: go_at,
			end: None,
			measured: None,
		});
	}

	/// The client wrote a save at `written_at` (unix seconds), and it writes one only as an addon load ends (a
	/// `/reload`, a logout, the game's exit): the load the strip showed last ends here when the capture first showed it
	/// before that write, whether or not the next load is ever seen, as a new load's first frame would end it (the
	/// logout rule owes what the addon owed at its logout). The app takes the save in after it, so the save's copies of
	/// those owed rows count as the helper's, never as rows of their own.
	pub fn save_written(&mut self, written_at: f64, now: &Now) -> Effects {
		let mut fx = Effects::default();
		let ended = |id: &u16| self.loads.get(id).and_then(|l| l.first_shown).is_some_and(|t| t < written_at);
		if let Some(old) = self.current.filter(ended) {
			self.current = None;
			self.end_load(old, now, &mut fx);
		}
		fx
	}

	/// The game quit: the current load ended (a move still open ends first), then the device hears the session is over.
	pub fn end_game(&mut self, now: &Now) -> Effects {
		let mut fx = Effects::default();
		if let Some(old) = self.current.take() {
			self.end_load(old, now, &mut fx);
		}
		fx.commands.push(Command::SessionEnd);
		if self.leaving {
			self.leaving = false;
			fx.suspend = Some(false);
		}
		fx.due(MirrorDue::Now);
		fx
	}

	fn unwrap_clock(&mut self, load_id: u16, ts: u32) -> u64 {
		let load = self.loads.entry(load_id).or_default();
		let ts = u64::from(ts);
		let clock = match load.clock {
			None => ts,
			Some(last) => {
				let mut c = (last & !(WRAP - 1)) | ts;
				if c + WRAP / 2 < last {
					c += WRAP;
				} else if c > last + WRAP / 2 && c >= WRAP {
					c -= WRAP;
				}
				c
			}
		};
		load.clock = Some(load.clock.map_or(clock, |l| l.max(clock)));
		clock
	}

	/// The counted move as the heartbeats show it. Back at its own Ready on the same slot (Escape, a popup, the game
	/// menu, combat, the game's Back; since 2026-10-05 the Back from its result view too, before its row: the game goes
	/// on to the next slot only once it wrote the row), the device hears `move_end` with `cancelled` and the verdict
	/// drops the move (`move_back`). In its result view (its count being typed, or "Did you do the whole move?") the
	/// move is done: the device hears `move_end` without `cancelled` at `at`, and the verdict holds its set until the
	/// game keeps it. Gone on to another move (a timed move that ran its time goes straight on, and the result view
	/// never paints), the move is done and kept (`move_done`).
	fn follow_move(&mut self, load_id: u16, fields: &Fields, at: i64, fx: &mut Effects) {
		let Some(load) = self.loads.get_mut(&load_id) else { return };
		let Some(m) = load.moving.filter(|m| m.ts == fields.row_ts) else { return };
		if fields.slot != m.slot {
			fx.commands.extend(self.move_done(load_id, at));
		} else if fields.state == StripState::Idle {
			fx.commands.extend(self.move_back(load_id));
		} else if fields.state == StripState::Input && !m.ended {
			load.moving = Some(Moving { ended: true, ..m });
			let id = pump_id(load_id, m.ts);
			self.book.end_unsaved(&id, m.slot, at);
			fx.commands.push(Command::MoveEnd { pump_id: id, slot: m.slot, cancelled: false });
		}
	}

	/// The game kept the load's counted move (its row came, it went on, the pump or the load ended): the verdict takes
	/// its set as ended at `at` (ms on the helper's clock), or at its result view when that showed, and a device that
	/// has not heard its end hears `move_end` without `cancelled`, so it finishes the move then. None when no move
	/// counts or the device heard its end at its result view.
	fn move_done(&mut self, load_id: u16, at: i64) -> Option<Command> {
		let m = self.loads.get_mut(&load_id)?.moving.take()?;
		let id = pump_id(load_id, m.ts);
		self.book.end(&id, m.slot, at);
		(!m.ended).then_some(Command::MoveEnd { pump_id: id, slot: m.slot, cancelled: false })
	}

	/// The game's Keep going on the load's counted move, still running (slice 52's double-check, 2026-10-24): the device
	/// that follows it hears `keep` with the full move's seconds its clock now counts up to. The frame's own event counter
	/// makes it once, as every event frame's; a Keep for any other move (one whose result showed, another slot or pump, a
	/// move whose countdown the helper never saw) is ignored.
	fn keep_going(&self, load_id: u16, fields: &Fields) -> Option<Command> {
		let m = self.loads.get(&load_id)?.moving?;
		((m.ts, m.slot) == (fields.row_ts, fields.slot) && !m.ended).then(|| Command::Keep {
			pump_id: pump_id(load_id, m.ts),
			slot: m.slot,
			seconds: fields.seconds,
		})
	}

	/// The game took the load's counted move back to its Ready, nothing written: the verdict drops it, and the device
	/// hears `move_end` with `cancelled`, so it drops the move, counting down, running or ended at its result view.
	fn move_back(&mut self, load_id: u16) -> Option<Command> {
		let m = self.loads.get_mut(&load_id)?.moving.take()?;
		let id = pump_id(load_id, m.ts);
		self.book.cancel(&id, m.slot);
		Some(Command::MoveEnd { pump_id: id, slot: m.slot, cancelled: true })
	}

	/// A heartbeat paints the pump as it stands every second, so it is the truth: a pump that no longer shows (no
	/// pump, or another one) is over even when its `pump_done` frame was missed, and so is its counted move.
	fn on_heartbeat(&mut self, load_id: u16, fields: &Fields, at: i64, fx: &mut Effects) {
		let load = self.loads.entry(load_id).or_default();
		load.stale = false;
		match load.pump.as_mut() {
			Some(p) if p.offered_ts == fields.row_ts && fields.row_ts != 0 => {
				p.input = (fields.state == StripState::Input).then_some(fields.slot);
			}
			Some(_) => load.pump = None,
			None => {}
		}
		if load.moving.is_some_and(|m| m.ts != fields.row_ts) {
			fx.commands.extend(self.move_done(load_id, at));
		}
	}

	fn record(f: &RawFrame, fields: &Fields, clock: u64, row: &Row, now: &Now) -> RowRecord {
		let mut r = RowRecord::new(row, &now.rfc3339, Source::Game);
		r.load_id = Some(f.load_id);
		r.event_no = Some(f.event_no);
		r.ts_client_ms = Some(clock);
		r.yards = Some(Yards {
			foot: fields.yards_foot,
			mount: fields.yards_mount,
			swim: fields.yards_swim,
			taxi: fields.yards_taxi,
		});
		r
	}

	/// Stores a row and its consequences (`keep`: a reset drops the rows, app rows, sheets, proofs and held list from
	/// before it; a felt-unwell row locks) and writes its `row` line.
	fn add(&mut self, r: RowRecord, now: i64, fx: &mut Effects) -> Added {
		let added = self.keep(r.clone(), now, fx);
		if added != Added::Skipped {
			fx.lines.push(Line::Row(r));
		}
		added
	}

	/// Stores a game or catch-up row and its consequences. A reset row is the player's, everywhere (slice 26): the app
	/// rows, the sheets and the held list from before it go too; the install it came from forgets its numbering and
	/// profiles before its row arrives here (`forget_install`), and the other installs keep theirs. A row dated where
	/// none may be at `now` (the helper's clock, unix seconds: `timely`) is skipped.
	fn keep(&mut self, r: RowRecord, now: i64, fx: &mut Effects) -> Added {
		let row = r.row();
		if !timely(&row, now, fx) {
			return Added::Skipped;
		}
		let (added, written) = self.store.add(r);
		if let Err(e) = written {
			fx.store_errors.push(format!("{}: {e}", store::FILE));
		}
		match added {
			Added::Skipped => return added,
			Added::Reset => {
				// the rows written since the reset stay (a reset read late from a file), and so do their lock and
				// their pumps' proofs
				if let Err(e) = self.app.reset(row.ts) {
					fx.store_errors.push(format!("{}: {e}", app_rows::FILE));
				}
				self.people.sheets.clear();
				self.people.sheet_apps.clear();
				self.people.held.clear();
				self.people.aside.clear();
				self.locked_until = self.lock_from_rows();
				self.book.forget_before(row.ts);
				if let Err(e) = self.sheet_log.reset(row.ts) {
					fx.sheet_errors.push(e.to_string());
				}
				// the other computers' rows from before it go too (slice 60): the reset is the player's
				if let Err(e) = self.far.floor(row.ts) {
					fx.store_errors.push(format!("{}: {e}", far::FILE));
				}
				fx.people_changed = true;
				fx.proof_changed = true;
				fx.due(MirrorDue::Now);
				return added;
			}
			Added::Row => fx.due(MirrorDue::Soon),
		}
		self.extend_lock(&row);
		added
	}

	/// Whether an app row belongs to the progression that holds now: dated at or after the game store's latest reset,
	/// the one floor of both stores.
	fn app_accepts(&self, row: &Row) -> bool {
		self.store.floor().is_none_or(|f| row.ts >= f)
	}

	/// Keeps an app row (its copy number set) and the lock it sets: `Known` when that copy is kept already or it is
	/// dated before the latest reset, `Unwritten` when `app.jsonl` could not take a phone's or a file's (not kept: it
	/// comes again). The computer's own (`own`) nothing sends again: it is kept whatever the write (`AppRows::add_own`),
	/// so its lock holds at once and it reaches the mirror. `Untimely` for one dated where none may be at `now` (the
	/// helper's clock, unix seconds: `timely`).
	fn keep_app(&mut self, r: RowRecord, own: bool, now: i64, fx: &mut Effects) -> AppKept {
		let row = r.row();
		if !timely(&row, now, fx) {
			return AppKept::Untimely;
		}
		if !self.app_accepts(&row) {
			return AppKept::Known;
		}
		let (kept, written) = if own { self.app.add_own(r) } else { self.app.add(r) };
		if let Err(e) = &written {
			fx.store_errors.push(format!("{}: {e}", app_rows::FILE));
		}
		match (kept, written.is_ok()) {
			(false, false) => AppKept::Unwritten,
			(false, true) => AppKept::Known,
			(true, _) => {
				self.extend_lock(&row);
				fx.due(MirrorDue::Now);
				AppKept::New
			}
		}
	}

	/// The rows a phone's app sent in a sync (docs/plan.md slice 26; `contract/companion-api.yaml`, `AppRow`): each
	/// kept once by its text and copy number in the store of app rows, with a `row` line (`source` app) and the lock it
	/// sets, and the mirror due at once. A row that does not read as the grammar writes it, has no copy number, is no
	/// app row (`app_row`: the app bit and exactly one device's, no character, no reset) and no phone's Start over
	/// (`start_over_row`), is dated before the latest reset or where no row may be (`timely`) is dropped for good. The desktop app's rows a phone took
	/// in from the helper's copy come back as the computer's (11|12), the text the store holds them as, so they are
	/// kept once. A row kept already counts as neither
	/// kept nor dropped, and the response's `rows_store` confirms all of them, unless one could not be written
	/// (`AppTaken::unwritten`).
	pub fn take_app_rows(&mut self, device: Device, rows: &[AppRow], now: &Now) -> (AppTaken, Effects) {
		self.take_rows(Some(device), rows, now)
	}

	/// The app rows a backup file brought (docs/plan.md slice 35, the desktop app's Backup; `backup::File::app_rows`),
	/// taken as `take_app_rows` takes a sync's: each kept once by its text and copy number, so a file taken in twice, or
	/// one whose rows the phones sent already, changes nothing. `from` is the phone that made the file, none for the
	/// computer's own.
	pub fn restore_app_rows(&mut self, from: Option<Device>, rows: &[AppRow], now: &Now) -> (AppTaken, Effects) {
		self.take_rows(from, rows, now)
	}

	/// `take_app_rows` and `restore_app_rows`: the rows kept with the device that sent them, if any.
	fn take_rows(&mut self, device: Option<Device>, rows: &[AppRow], now: &Now) -> (AppTaken, Effects) {
		let mut fx = Effects::default();
		let mut taken = AppTaken::default();
		for sent in rows {
			let row = sent.row.parse::<Row>().ok().filter(|r| r.to_string() == sent.row);
			let Some(row) = row.filter(|r| sent.n > 0 && (app_row(r) || start_over_row(r)) && self.app_accepts(r))
			else {
				taken.dropped += 1;
				continue;
			};
			let mut r = RowRecord::new(&row, &now.rfc3339, Source::App);
			(r.copy, r.device) = (Some(sent.n), device);
			match self.keep_app(r.clone(), false, now.unix, &mut fx) {
				AppKept::New => {
					fx.lines.push(Line::Row(r));
					taken.kept += 1;
				}
				AppKept::Known => {}
				AppKept::Unwritten => taken.unwritten = true,
				AppKept::Untimely => taken.dropped += 1,
			}
		}
		(taken, fx)
	}

	/// Keeps the rows of a pump the desktop app ran (docs/plan.md slice 35; the desk core's, slice 63) as its own app
	/// rows: each stamped as a phone stamps its own (`char` 0, the app bit beside `device`'s, the desktop app's
	/// `Source::Computer`), its copy number the first one of its text the store of app rows does not hold yet
	/// (`AppRows::next_copy`), so a pump written in parts, a row at a time, numbers its rows as one list would, and kept
	/// with the pump's id (`d-<uuid>` on the computer) and the lock it sets (a felt-unwell stop, a pump row banked with
	/// the reason, locks every pump at once), with a `row` line (`source` app) and the mirror due at once, as
	/// `take_app_rows` keeps a phone's. A row that is no app row once stamped (a reset row, one with another device's
	/// bit) or is dated before the latest reset is not kept; one `app.jsonl` could not take is kept all the same, since
	/// nothing sends it again (`AppRows::add_own`: the next write rewrites the file with it), and its write error is in
	/// `Effects::store_errors`. The pump's offered row carries what the pump was (`what`) where that names it (a move
	/// started from the Book, which the desktop's Saga titles as asked). Returns the number of rows kept.
	pub fn keep_computer(
		&mut self,
		device: Source,
		pump: &str,
		what: Option<PumpKind>,
		rows: &[Row],
		now: &Now,
	) -> (usize, Effects) {
		let mut fx = Effects::default();
		let mut kept = 0;
		for row in rows {
			let row = Row { char: 0, flags: row.flags | flags::APP | device.bit(), ..*row };
			if !app_row(&row) {
				continue;
			}
			let mut r = RowRecord::new(&row, &now.rfc3339, Source::App);
			(r.copy, r.pump) = (Some(self.app.next_copy(&row)), Some(pump.to_owned()));
			r.pump_kind = what.filter(|_| kind::is_pump(row.kind) && row.outcome == outcome::OFFERED);
			if self.keep_app(r.clone(), true, now.unix, &mut fx) == AppKept::New {
				fx.lines.push(Line::Row(r));
				kept += 1;
			}
		}
		(kept, fx)
	}

	/// Takes in what a file of an install holds (the mirror already in its folder before the helper's first write there,
	/// the game's own save after it quits), so a lost data folder, a session played while the helper was off or a reset
	/// made then never costs or keeps the wrong rows. A file saved at or after the last reset gives the install's
	/// numbering as it stands: each of its characters is the helper's with that GUID, at its index (`bind_character`).
	/// A game row the store lacks, counted by copies as the addon's merge counts them against the store's rows in this
	/// install's numbering (`game_rows_in`, another install's unnamed placeholder's rows as 0; the take-in binds no
	/// other install's character, so an index the file names no character for stays its own), is stored without a
	/// `row` line, since it is not new, with the source its flags name and its character as the helper's; an app row
	/// (the app bit), counted by copies the same way, goes to the store of app rows, never the game's. Such a file also
	/// brings the install's profiles and profile keys: all of them when it is newer than what the helper learnt last of
	/// that install, else the ones it lacks; and a save its baseline, kept while it is that install's newest read
	/// (`keep_base`, for `game_save`).
	/// Returns the number of rows taken in.
	pub fn adopt(&mut self, held: &Held, install: &str, now: &Now) -> (usize, Effects) {
		let mut fx = Effects::default();
		let current = self.store.floor().is_none_or(|f| held.saved_at >= i64::from(f));
		if current {
			self.bind_held(install, held, &mut fx);
		}
		let copies = |rows: Vec<(Row, bool)>| {
			let mut n: HashMap<Row, usize> = HashMap::new();
			for (row, _) in rows {
				*n.entry(row).or_default() += 1;
			}
			n
		};
		let mut stored = copies(self.game_rows_in(install));
		let mut apps: HashMap<Row, u16> = HashMap::new();
		// another computer's rows of no character read as app rows of this computer's in the file (`far::as_app`): the
		// far store keeps them, never the store of app rows
		let far: HashSet<Row> = self.far_placed(0, |_| None).into_iter().collect();
		let mut taken = 0;
		for held_row in &held.rows {
			if Source::from_flags(held_row.flags) == Source::App && far.contains(held_row) {
				continue;
			}
			if Source::from_flags(held_row.flags) == Source::App {
				// the n-th copy of the text among the file's app rows, as the phone numbered the rows the mirror wrote
				let n = apps.entry(*held_row).or_default();
				*n = n.saturating_add(1);
				let mut r = RowRecord::new(held_row, &now.rfc3339, Source::App);
				r.copy = Some(*n);
				if app_row(held_row) && self.keep_app(r, false, now.unix, &mut fx) == AppKept::New {
					taken += 1;
				}
				continue;
			}
			if !self.store.accepts(held_row) {
				continue;
			}
			if let Some(n) = stored.get_mut(held_row).filter(|n| **n > 0) {
				*n -= 1;
				continue;
			}
			// refused for its time first, so a reset dated where none may be never starts the numbering again
			if !timely(held_row, now.unix, &mut fx) {
				continue;
			}
			if held_row.kind == kind::RESET {
				self.forget_install(install, &mut fx); // a reset this store lacks: the file's numbering starts again
			}
			let row = Row { char: self.held_char(install, held, held_row.char, &mut fx), ..*held_row };
			match self.keep(RowRecord::new(&row, &now.rfc3339, Source::from_flags(row.flags)), now.unix, &mut fx) {
				Added::Skipped => {}
				Added::Row => taken += 1,
				Added::Reset => {
					// the install's numbering went with the reset: its characters are placed again, and the counts are
					// the kept rows'
					taken += 1;
					if current {
						self.bind_held(install, held, &mut fx);
					}
					stored = copies(self.game_rows_in(install));
					stored.entry(*held_row).and_modify(|n| *n = n.saturating_sub(1));
				}
			}
		}
		if current {
			self.adopt_profiles(held, install, &mut fx);
			fx.game_body = self.people.installs.get(install).and_then(|kept| held_body(held, install, kept));
			self.keep_base(held, install);
		}
		let floor = self.store.floor().unwrap_or(0);
		let proofs = held.proof.iter().filter(|&(&ts, _)| ts >= floor).map(|(&ts, p)| (ts, p.clone())).collect();
		fx.proof_changed |= self.book.adopt(&proofs);
		if taken > 0 || fx.people_changed || fx.proof_changed {
			fx.due(MirrorDue::Now);
		}
		(taken, fx)
	}

	/// A save's baseline and characters as its install's newest (`SaveBase`), unless the helper read a later save of that
	/// install since it started; a mirror, and a save its client never read back (the beta: `mirror::parse_save`),
	/// carry no baseline and change nothing.
	fn keep_base(&mut self, held: &Held, install: &str) {
		let Some(baseline) = &held.baseline else { return };
		if self.bases.get(install).is_some_and(|b| b.saved_at > held.saved_at) {
			return;
		}
		let base = SaveBase { saved_at: held.saved_at, baseline: baseline.clone(), chars: held.chars.clone() };
		self.bases.insert(install.to_owned(), base);
	}

	/// Every character of a file at its index of the install, by GUID (`bind_character`).
	fn bind_held(&mut self, install: &str, held: &Held, fx: &mut Effects) {
		for (i, c) in (1..=u8::MAX).zip(&held.chars) {
			if let Some(c) = c {
				self.bind_character(install, i, c, false, fx);
			}
		}
	}

	/// The helper's character an index of a file is: the file's character there by GUID, else the install's binding
	/// (an index the file names no character for), else a new placeholder.
	fn held_char(&mut self, install: &str, held: &Held, index: u8, fx: &mut Effects) -> u8 {
		match usize::from(index).checked_sub(1).and_then(|k| held.chars.get(k)) {
			Some(Some(c)) => self.bind_character(install, index, c, false, fx),
			_ => self.global(install, index, fx),
		}
	}

	/// A file's profiles and profile keys into its install's (2026-10-09: by each profile's own change stamp, never by
	/// the file's `savedAt` alone, which every load stamps: a beta that loaded a mirror one `/reload` behind saved its
	/// stale profiles with a newer `savedAt`, and they flip-flopped with the fresh ones at every `/reload`): a profile
	/// the install lacks, or one that changed later than the install's copy (`profile_newer`; of two without a stamp,
	/// the file's when it is newer than what the helper learnt last of that install, as before the stamps); a
	/// character's profile key the install lacks, or one naming a profile that changed later than the one the install
	/// names (a switch of profiles stamps the profile taken: `Settings_.UseProfile`).
	fn adopt_profiles(&mut self, held: &Held, install: &str, fx: &mut Effects) {
		let entry = self.install_mut(install, fx);
		let newer = held.saved_at > entry.profiles_at;
		let render = |v: &Value| mirror::lua_value(v, "");
		let mut changed = false;
		for (guid, name) in &held.profile_keys {
			let mine = entry.profile_keys.get(guid);
			let switched = mine.is_some_and(|mine| {
				mine != name
					&& held.profiles.get(name).is_some_and(|p| profile_newer(p, entry.profiles.get(mine), false, newer))
			});
			if mine.is_none() || switched {
				entry.profile_keys.insert(guid.clone(), name.clone());
				changed = true;
			}
		}
		for (name, profile) in &held.profiles {
			let mine = entry.profiles.get(name);
			if profile_newer(profile, mine, false, newer) && mine.map(render) != Some(render(profile)) {
				entry.profiles.insert(name.clone(), profile.clone());
				changed = true;
			}
		}
		if newer && changed {
			entry.profiles_at = held.saved_at;
		}
		fx.people_changed |= changed;
	}

	/// The install a load belongs to (`Load::install`); the one the helper follows for a load it has not seen.
	fn load_install(&self, load_id: u16) -> String {
		self.loads.get(&load_id).map_or_else(|| self.install.clone(), |l| l.install.clone())
	}

	/// A row the strip showed, in its load's install's numbering: stored under the helper's own character. A reset row
	/// starts that install's numbering again first, so its character is the new save's (a placeholder until the new
	/// save's profile names it).
	fn on_row(&mut self, f: &RawFrame, fields: &Fields, clock: u64, at: i64, now: &Now, fx: &mut Effects) {
		let install = self.load_install(f.load_id);
		let mut row = Row {
			ts: fields.row_ts,
			char: 0,
			quest_id: fields.quest_id,
			tier: f.tier(),
			slot: fields.slot,
			ladder: fields.ladder,
			rung: fields.rung,
			move_id: fields.move_id,
			kind: f.kind,
			reps: fields.reps,
			seconds: fields.seconds,
			outcome: fields.outcome,
			flags: fields.row_flags,
		};
		// refused for its time first, so a reset dated where none may be never starts the numbering again
		if !timely(&row, now.unix, fx) {
			return;
		}
		if row.kind == kind::RESET && self.store.accepts(&row) {
			self.forget_install(&install, fx);
		}
		row.char = self.global(&install, fields.char_index, fx);
		let mut record = Self::record(f, fields, clock, &row, now);
		let stretch = row.kind == kind::STRETCH;
		if stretch {
			record.pad_metres = self.pad_metres(f.load_id);
		}
		if self.add(record, now.unix, fx) == Added::Skipped {
			return;
		}
		// the stretch row closes the load's stretch on the pad too; one stored already (a restart's) closes nothing
		if stretch && let Some(from) = self.loads.get_mut(&f.load_id).and_then(|l| l.pad_from.as_mut()) {
			*from = self.pad.walked;
		}
		if let Some(p) = self.loads.entry(f.load_id).or_default().pump.as_mut() {
			if row.locks() {
				p.ended = true; // felt unwell ended it
			}
			// the pump's own result row closes its slot: the move's quest and move, from its offered time on; an owed,
			// banked or paid row (another pump's owed moves, a rider's payment or its skip) closes nothing
			let result = !matches!(row.outcome, outcome::OWED | outcome::BANKED | outcome::PAID);
			let own = p.moves.get(&row.slot).is_some_and(|m| {
				m.fields.quest_id == row.quest_id && m.fields.move_id == row.move_id && row.ts >= p.offered_ts
			});
			if kind::is_move(row.kind) && result && own {
				p.closed.push(row.slot);
				// the counted move's own result: done and kept, even when its result view never painted (a timed move)
				let offered = p.offered_ts;
				let counted = self.loads.get(&f.load_id).and_then(|l| l.moving);
				if counted.is_some_and(|m| (m.ts, m.slot) == (offered, row.slot)) {
					fx.commands.extend(self.move_done(f.load_id, at));
				}
				// the count typed in the game, for a phone that follows the pump (slice 51), after the move's end
				if outcome::is_typed(row.outcome) || row.outcome == outcome::DONE {
					fx.commands.push(Command::MoveCounted {
						pump_id: pump_id(f.load_id, offered),
						slot: row.slot,
						// a pose's flow row keeps its "How close?" answer in `reps` (`mark`), no count
						reps: if row.kind == kind::FLOW { 0 } else { row.reps },
						seconds: row.seconds,
						outcome: row.outcome,
					});
				}
			}
		}
	}

	fn on_offered(&mut self, f: &RawFrame, fields: &Fields, fx: &mut Effects) {
		let load = self.loads.entry(f.load_id).or_default();
		if load.pump.as_ref().is_none_or(|p| p.offered_ts != fields.row_ts) {
			load.pump = Some(Pump {
				offered_ts: fields.row_ts,
				count: fields.outcome,
				moves: BTreeMap::new(),
				closed: Vec::new(),
				input: None,
				ended: false,
			});
		}
		let Some(p) = load.pump.as_mut() else { return };
		// a pump composed again (a new pain flag, a pain stop's re-roll) is painted again: the new count and moves win,
		// and the completed list goes out again
		let count = fields.outcome;
		p.count = count;
		if fields.slot == 0 {
			p.moves.clear(); // the addon paints a composition from its first move on
		}
		p.moves.insert(fields.slot, Move { fields: fields.clone(), tier: f.tier(), kind: f.kind });
		p.moves.retain(|&slot, _| slot < count);
		let command = offer(f.load_id, p, &self.people, &load.install);
		if command.is_some() {
			fx.offered = Some(offered_line(f.load_id, p));
		}
		fx.commands.extend(command);
	}

	/// The pump the load the strip shows offered and has not finished, as its `pump_offered` once every move arrived:
	/// the device that holds the link without having heard it gets it again (`Link::reoffer`). None while felt unwell
	/// ended it or a loss of sight leaves it in doubt (the next heartbeat, within a second, tells).
	#[must_use]
	pub fn offered(&self) -> Option<Command> {
		let id = self.current?;
		let load = self.loads.get(&id).filter(|l| !l.stale)?;
		offer(id, load.pump.as_ref().filter(|p| !p.ended)?, &self.people, &load.install)
	}

	/// The load the strip shows has a pump on that no `pump_done` frame or heartbeat ended yet, however stale the sight
	/// and however many of its moves arrived (where `offered` says none): what holds a quit asked on the helper's
	/// socket, failing closed.
	#[must_use]
	pub fn pump_on(&self) -> bool {
		self.current.and_then(|id| self.loads.get(&id)).and_then(|l| l.pump.as_ref()).is_some_and(|p| !p.ended)
	}

	fn on_blob(&mut self, load_id: u16, event: StripEvent, packed: &[u8], now: &Now, fx: &mut Effects) {
		let value = match blob::decode(packed) {
			Ok(value) => value,
			Err(e) => return self.blob_error(load_id, event, &e.to_string(), fx),
		};
		match event {
			StripEvent::Profile => {
				if !self.on_profile(load_id, &value, now, fx) {
					self.blob_error(load_id, event, "no index, name or profile", fx);
				}
			}
			StripEvent::Sheet => {
				let index = blob::get(&value, "char").and_then(blob::as_u64).and_then(|i| u8::try_from(i).ok());
				if let Some(i) = index {
					let sheet = blob::to_json(&value);
					// a copy of a save reset since (its chunks painted on after the reset row) is neither logged nor served
					if SheetRecord::new(now.rfc3339.clone(), i, &sheet).is_ok_and(|r| !self.sheet_log.accepts(&r)) {
						return;
					}
					// logged and served under the helper's own character, as the history rows name it
					let i = self.global(&self.load_install(load_id), i, fx);
					let logged = SheetRecord::new(now.rfc3339.clone(), i, &sheet);
					// the latest sheet says what the game holds back now (none when it lists nothing)
					let moves = |key: &str| -> Vec<u16> {
						blob::get(&value, key)
							.map(blob::to_json)
							.unwrap_or_default()
							.as_array()
							.map(|a| a.iter().filter_map(|v| v.as_u64()?.try_into().ok()).collect())
							.unwrap_or_default()
					};
					self.people.held = moves("held");
					self.people.aside = moves("aside");
					if let Err(e) = logged.map_err(std::io::Error::from).and_then(|r| self.sheet_log.add(r)) {
						fx.sheet_errors.push(e.to_string());
					}
					self.people.sheets.insert(i, value);
					let apps = self.loads.get(&load_id).map_or_else(|| self.app_mark(), |l| l.apps);
					self.people.sheet_apps.insert(i, apps);
					fx.people_changed = true;
				}
			}
			StripEvent::Feedback => match feedback::Message::game(load_id, &value) {
				Ok(message) => fx.feedback = Some(message),
				Err(e) => self.blob_error(load_id, event, &e.to_string(), fx),
			},
			_ => {}
		}
	}

	/// A payload that did not decode or check out. A profile's or a sheet's is reported once per load and event, since
	/// their copies repaint every few seconds: a codec mismatch with the live client shows in the log instead of as
	/// profiles that never reach the mirror. Every feedback message's is reported, each a message the player sent that
	/// is lost.
	fn blob_error(&mut self, load_id: u16, event: StripEvent, why: &str, fx: &mut Effects) {
		fx.feedback_dropped |= event == StripEvent::Feedback;
		if event == StripEvent::Feedback || self.loads.entry(load_id).or_default().blob_warned.insert(event) {
			fx.blob_errors.push(format!("{event:?}: {why}"));
		}
	}

	/// Keeps a profile event's character and profile; false when the payload lacks one of its fields.
	fn on_profile(&mut self, load_id: u16, value: &Value, now: &Now, fx: &mut Effects) -> bool {
		let index = blob::get(value, "index").and_then(blob::as_u64).and_then(|i| u8::try_from(i).ok());
		let name = blob::get(value, "name").and_then(blob::as_text).map(str::to_owned);
		let text = |c: &Value, k: &str| blob::get(c, k).and_then(blob::as_text).map(str::to_owned);
		let character = blob::get(value, "char").and_then(|c| {
			Some(Character {
				guid: text(c, "guid")?,
				name: text(c, "name"),
				realm: text(c, "realm"),
				class: text(c, "class"),
			})
		});
		let (Some(index), Some(name), Some(profile)) = (index, name, blob::get(value, "profile")) else {
			return false;
		};
		if index == 0 {
			return false;
		}
		// the load's install: its index of the character, and its profiles
		let install = self.load_install(load_id);
		if let Some(c) = &character {
			self.bind_character(&install, index, c, true, fx);
		}
		let entry = self.install_mut(&install, fx);
		if let Some(c) = character {
			if name == "Default" {
				entry.profile_keys.remove(&c.guid);
			} else {
				entry.profile_keys.insert(c.guid, name.clone());
			}
		}
		// the profile the game holds now, unless the helper holds a later change of it: a beta that loaded a mirror one
		// /reload behind shows its stale copy until it loads again (`profile_newer`)
		if profile_newer(profile, entry.profiles.get(&name), true, true) {
			fx.game_body = Some(GameBodySeen {
				profile: format!("{install}/{name}"),
				seen: body::GameBody::of_profile(profile),
				at: None,
			});
			entry.profiles.insert(name, profile.clone());
			entry.profiles_at = now.unix;
		}
		fx.people_changed = true;
		fx.due(MirrorDue::Soon);
		true
	}

	/// The logout rule (rust-helper `protocol-and-mirror.md`): no frame renders after `PLAYER_LOGOUT`, so the rows the
	/// addon writes there never reach the strip. For the pump it offered and did not finish, the helper writes one
	/// owed row per move the addon would owe (not a rider, not a pump the player asked for) that no row closed and
	/// that the player was not typing a count for, dated at the pump's offered time like the addon's, so the merge
	/// keeps one copy on retail and the beta keeps them at all.
	fn end_load(&mut self, load_id: u16, now: &Now, fx: &mut Effects) {
		let Some(load) = self.loads.remove(&load_id) else { return };
		if let Some(m) = load.moving {
			let id = pump_id(load_id, m.ts);
			if m.ended {
				// in its result view: the addon writes its row as done at the logout (`Lc.OweOpen`), so its set counts,
				// ended where its result view showed
				self.book.end(&id, m.slot, now.unix * 1000);
			} else {
				// a /reload, a logout or the game's quit mid-move: the device ends it as it stands, and the verdict
				// drops it, since the addon owes a move that never reached its result
				self.book.cancel(&id, m.slot);
				fx.commands.push(Command::MoveEnd { pump_id: id, slot: m.slot, cancelled: false });
			}
		}
		let Some(p) = load.pump else {
			return;
		};
		if p.ended || load.stale {
			return;
		}
		for (&slot, m) in &p.moves {
			let f = &m.fields;
			if f.owes != 1 || p.closed.contains(&slot) || p.input == Some(slot) {
				continue;
			}
			// the addon's oweSlot: keys (ts, questID, slot) stay unique in the owed queue
			let mut s = slot;
			while self.store.is_owed((p.offered_ts, f.quest_id, s)) {
				s = s.saturating_add(1);
			}
			let row = Row {
				ts: p.offered_ts,
				char: self.global(&load.install, f.char_index, fx),
				quest_id: f.quest_id,
				tier: m.tier,
				slot: s,
				ladder: f.ladder,
				rung: f.rung,
				move_id: f.move_id,
				kind: m.kind,
				// an addon from before 2026-10-15 (until its next /reload) paints 2 on a closing move, whose cell 16
				// names its form: reps 1 makes the row that form's window, which an older and a newer addon both read
				// as one window, where reps 2 under the form would be an older addon's closing move led by that form
				reps: u16::from(f.owed_reps.min(1)),
				seconds: f.seconds,
				outcome: outcome::OWED,
				flags: 0,
			};
			self.add(RowRecord::new(&row, &now.rfc3339, Source::Game), now.unix, fx);
		}
		fx.due(MirrorDue::Now);
	}

	/// A catch-up pump from the phone, the watch or the desktop app pays owed moves one for one: first every move that
	/// names an owed row (its `ts`, since 2026-10-12, as `Session::skip` finds its row) pays that row while it is still
	/// owed, then every other move (its named row went, or it named none) the oldest owed row of the move it names
	/// still free (one the game paid meanwhile pays nothing), so a move that falls back never takes the row a later
	/// move was dosed from; every all-out window it ran pays (since 2026-10-14, when every owed window began to ride
	/// the game's next pump). When it paid a move it writes a pump row, then for each paid move, in the pump's order,
	/// the move row and the paid row that names its owed row: a move whose count the player saved (its `set`, since
	/// 2026-10-12, when it fits the move row and was dosed from the row it pays: a set of a move whose named row went
	/// is dropped) as the game's own catch-up writes it (`CatchupSet::rows`: its load row, the move row with the count,
	/// its rung rows and the steps it mastered, dated the moment the count was saved), any other done with the seconds
	/// it took (an all-out window with reps 1, as the game pays it). A move row repeats its owed row as the store keeps
	/// it (`payable_kept`: a merged move's under its old id and rung, a retired ladder's on it), so every fold reads it
	/// as it reads that owed row (DB.lua `ns.Translate`): a merged move's as the move that took it, on that move's own
	/// rung and ladder; the rung of a ladder that lost its first rung after the owed row is written as that row reads
	/// (`library::read_rung`), since the move row's own time is after the loss, and a row of reps or a hold whose move
	/// is timed now as the flow row it is owed as (`owed_now`: its kind, ladder and rung), since a fold reads only an
	/// unfinished row that way (DB.lua `timedNow`) and a set done as written; since 2026-10-26 a move run in the
	/// owed move's place (its `ran`, a move of the grammar: Swap's, or an owed backup's stand-in since 2026-10-29)
	/// writes its move row as that move, its kind, ladder and rung,
	/// with the easier bit on Swap's easier version, as the game's `Lc.WriteResult` writes a swapped owed slot's, its
	/// set is judged against that row, and the step down Swap's easier version makes (`ran.down`, set or not) follows
	/// the move's rows as a rung row of cause 8 (`CatchupRan::down_row`). Its XP is not the game's: the game writes the
	/// rider slot dosed from such a row on the lowered rung (`ns.ResultRow`) and pays that rung's XP, while this row, a
	/// merged move's owed before 2026-10-04, pays the XP of the rung it was owed at (the fold's `xpRung`, where that rung
	/// sat above the move's own). The pump row and a move row done are dated the pump's start, and each paid row repeats
	/// its owed row's time, quest and slot (`ns.PaidRow`) with the move it is owed as now (`owed_now`); a felt-unwell
	/// stop adds a pump row banked with the reason, dated the stop. Every row carries the device's source bit; a move
	/// that pays nothing stays in the app. Answers the rows it wrote, or why its moves paid nothing (a pump paid
	/// already, one from before the last reset, one dated past the helper's clock (`library::row_timely`), the
	/// felt-unwell lock, nothing owed), beside the effects of what it wrote: a felt-unwell stop is written even when its
	/// moves are refused for the reset, the time or the lock (a stop dated where no row may be never is), so the lock it
	/// sets reaches the game, the mirror and every device (a pump paid already carries its stop).
	pub fn catch_up(&mut self, pump: &CatchupPump, now: &Now) -> (Result<Vec<Row>, CatchupError>, Effects) {
		let (source, bit) = (pump.source, pump.source.bit());
		let mut fx = Effects::default();
		if self.store.has_pump(&pump.id) {
			return (Err(CatchupError::AlreadyPaid(pump.id.clone())), fx);
		}
		let refused = self.refuses_start(pump.start, now.unix);
		let kept = self.payable_kept();
		let queue: Vec<Row> = kept.iter().map(owed_now).collect();
		// the move a paid move ran in the owed move's place (`ran`: Swap's or an owed backup's stand-in) where it names
		// one of the grammar (a move, of a move kind, as `Session::bank` takes its moves), else the owed move
		let swap_of = |m: &CatchupMove| m.ran.filter(|r| r.move_id > 0 && kind::is_move(r.kind));
		// each paid move's owed row (its place in the queue), the move, and its set where it was dosed from that row
		let picks = if refused.is_none() { paid_rows(&pump.moves, &queue) } else { Vec::new() };
		let pairs: Vec<(usize, &CatchupMove, Option<&CatchupSet>)> = pump
			.moves
			.iter()
			.zip(picks)
			.filter_map(|(m, pick)| {
				let (i, named) = pick?;
				let row = swap_of(m).map_or(queue[i], |x| x.row(&queue[i]));
				Some((i, m, m.set.as_ref().filter(|set| (m.ts.is_none() || named) && set.fits(&row))))
			})
			.collect();
		if pairs.is_empty() && pump.unwell_at.is_none() {
			return (Err(refused.unwrap_or(CatchupError::NothingOwed)), fx);
		}
		let char = pairs.first().map(|p| queue[p.0].char).or_else(|| queue.first().map(|r| r.char)).unwrap_or(1);
		let mut rows = Vec::new();
		if !pairs.is_empty() {
			rows.push(Row {
				ts: pump.start,
				char,
				kind: kind::PUMP,
				reps: u16::try_from(pairs.len()).unwrap_or(u16::MAX),
				outcome: outcome::OFFERED,
				flags: bit,
				..Row::default()
			});
		}
		for (slot, &(i, m, set)) in pairs.iter().enumerate() {
			let (r, swapped) = (&queue[i], swap_of(m));
			let done = Row {
				ts: pump.start,
				slot: u8::try_from(slot).unwrap_or(u8::MAX),
				reps: u16::from(all_out(r)),
				seconds: m.seconds,
				outcome: outcome::DONE,
				flags: bit | flags::OWED | swapped.map_or(0, CatchupRan::bits),
				// the move run in its place as the device names it, else the owed row under the id the store keeps,
				// dated now, with the kind and ladder it is owed with now (a set done is read as written, so one whose
				// move is timed now is written as the flow row it reads as), on the rung that row reads (a ladder that
				// lost its first rung since)
				..swapped.map_or_else(
					|| Row { move_id: kept[i].move_id, rung: library::read_rung(r.ladder, r.rung, r.ts), ..*r },
					|x| x.row(&kept[i]),
				)
			};
			let ts = if let Some(set) = set {
				rows.extend(set.rows(&done, bit, pump.start, now.unix));
				set.ts(pump.start, now.unix)
			} else {
				rows.push(done);
				done.ts
			};
			// the easier version's step down, set or not, on the ladder of the attempt it took, the owed row's
			// (`ns.EasierDown`), dated as the move row
			rows.extend(swapped.and_then(|x| x.down_row(r.ladder, done.char, ts, bit)));
			rows.push(Row { reps: 0, seconds: 0, outcome: outcome::PAID, flags: bit | flags::OWED, ..*r });
		}
		// the stop as the fold locks on it, dated its own moment: a catch-up started before 04:00 and stopped after it
		// locks the new game day
		if let Some(at) = pump.unwell_at {
			rows.push(Row {
				ts: at,
				char,
				kind: kind::PUMP,
				outcome: outcome::BANKED,
				flags: bit | flags::UNWELL,
				..Row::default()
			});
		}
		for row in &rows {
			let mut record = RowRecord::new(row, &now.rfc3339, source);
			record.pump = Some(pump.id.clone());
			self.add(record, now.unix, &mut fx);
		}
		// its heart rate, scored as a game move's from its start (the pump's end its set's); dynamic when a move it paid
		// is; a stop that paid nothing ran no move, so it has none to score
		if !pairs.is_empty() {
			let go = i64::from(pump.start) * 1000;
			let ran: i64 = pairs.iter().map(|(_, m, _)| i64::from(m.seconds)).sum();
			let dynamic = pairs
				.iter()
				.any(|&(i, m, _)| proof::class_of(swap_of(m).map_or(queue[i].kind, |x| x.kind)) == Class::Dynamic);
			fx.proof_changed |= self.book.start(proof::Move {
				pump_id: pump.id.clone(),
				ts: pump.start,
				slot: None,
				class: if dynamic { Class::Dynamic } else { Class::Iso },
				after_set: false,
				go,
				end: Some(go + ran * 1000),
				measured: pump.measured,
			});
		}
		fx.due(MirrorDue::Now);
		(refused.map_or(Ok(rows), Err), fx)
	}

	/// The player's Skip of unfinished moves on a phone, a watch or the desktop app (since 2026-10-09; David,
	/// 2026-10-04: "the skip should be per unfinished move without a confirm"), as the game's `Lc.Skip` and
	/// `Lc.SkipHeld` write it (`ns.PaidRow(o, true)`): for each move the paid row of its owed row, the one at the `ts` the
	/// list gave while it is still owed, else the oldest of that move (a held move too: the game's HUD skips those), with
	/// the skipped flag and the device's bit; no move row and no pump row, so it earns nothing and costs nothing, and no
	/// heart rate is scored. A skip is no pump: the felt-unwell lock leaves it be. Answers the rows it wrote, or why it
	/// wrote none (sent again, from before the last reset, nothing of those moves owed).
	pub fn skip(&mut self, skip: &CatchupPump, now: &Now) -> (Result<Vec<Row>, CatchupError>, Effects) {
		let mut fx = Effects::default();
		if self.store.has_pump(&skip.id) {
			return (Err(CatchupError::AlreadyPaid(skip.id.clone())), fx);
		}
		if !self.store.accepts(&Row { ts: skip.start, kind: kind::PUMP, ..Row::default() }) {
			return (Err(CatchupError::BeforeReset), fx);
		}
		// the whole queue by the move it is owed as now, as `payable` names it, the held moves included
		let queue: Vec<Row> = self.store.owed().iter().map(owed_now).collect();
		let mut used = vec![false; queue.len()];
		let mut rows = Vec::new();
		for m in &skip.moves {
			let free = |i: &usize| !used[*i] && queue[*i].move_id == m.move_id;
			let exact = (0..queue.len()).filter(free).find(|&i| m.ts == Some(queue[i].ts));
			let Some(i) = exact.or_else(|| (0..queue.len()).find(free)) else { continue };
			used[i] = true;
			let flags = skip.source.bit() | flags::OWED | flags::SKIPPED;
			rows.push(Row { reps: 0, seconds: 0, outcome: outcome::PAID, flags, ..queue[i] });
		}
		if rows.is_empty() {
			return (Err(CatchupError::NothingOwed), fx);
		}
		for row in &rows {
			let mut record = RowRecord::new(row, &now.rfc3339, skip.source);
			record.pump = Some(skip.id.clone());
			self.add(record, now.unix, &mut fx);
		}
		fx.due(MirrorDue::Now);
		(Ok(rows), fx)
	}

	/// A match pump's Bank all, or the desktop's quit during one (docs/plan.md slice 58), as the game's `Lc.Bank` owes
	/// a quest's moves (Events.lua `oweSlot` with outcome banked): for each move banked one game owed row with outcome
	/// banked, the device's bit and no app bit (so every fold takes it into the owed queue and the game day's banked
	/// count, and DB.lua's start of the game moves for none: `applyRow`), no character (0, placed in every install's
	/// mirror), quest 0, dated the pump's start, its slot the move's place among the banked moves made unique in the
	/// owed queue as `oweSlot` makes it, the pump's tier, the move's ladder, rung and kind, `reps` 1 on an all-out
	/// cardio window and the seconds of a timed move (cardio, flow); no pump row and no heart rate. So the moves show
	/// on every unfinished list with Skip and ride the next pump, the game's quest pump and a device's catch-up alike.
	/// Answers the rows it wrote, or why it wrote none: sent again (a row of that pump id is stored), from before the
	/// last reset, dated past the helper's clock (`library::row_timely`), inside the felt-unwell lock (one dated at or
	/// before the pump's start, `locked_at`, as a catch-up's), or no move. Never held while a World of Warcraft client with QuestPump runs: its moves are in no queue the game
	/// asks, and reach it at its next load through the mirror.
	pub fn bank(&mut self, bank: &BankPump, now: &Now) -> (Result<Vec<Row>, CatchupError>, Effects) {
		let mut fx = Effects::default();
		if self.store.has_pump(&bank.id) {
			return (Err(CatchupError::AlreadyPaid(bank.id.clone())), fx);
		}
		if let Some(refused) = self.refuses_start(bank.start, now.unix) {
			return (Err(refused), fx);
		}
		let mut rows = Vec::new();
		for (i, m) in bank.moves.iter().filter(|m| m.move_id > 0 && kind::is_move(m.kind)).enumerate() {
			// the addon's oweSlot: keys (ts, questID, slot) stay unique in the owed queue
			let mut slot = u8::try_from(i).unwrap_or(u8::MAX);
			while slot < u8::MAX && self.store.is_owed((bank.start, bank.quest, slot)) {
				slot += 1;
			}
			let timed = m.kind == kind::CARDIO || m.kind == kind::FLOW;
			let row = Row {
				ts: bank.start,
				char: 0,
				quest_id: bank.quest,
				tier: bank.tier,
				slot,
				ladder: m.ladder,
				rung: m.rung,
				move_id: m.move_id,
				kind: m.kind,
				reps: u16::from(m.allout && m.kind == kind::CARDIO),
				seconds: if timed { m.seconds } else { 0 },
				outcome: outcome::BANKED,
				flags: bank.source.bit(),
			};
			let mut record = RowRecord::new(&row, &now.rfc3339, bank.source);
			record.pump = Some(bank.id.clone());
			self.add(record, now.unix, &mut fx);
			rows.push(row);
		}
		if rows.is_empty() {
			return (Err(CatchupError::NothingOwed), fx);
		}
		fx.due(MirrorDue::Now);
		(Ok(rows), fx)
	}
}

/// A game pump's id for the devices: the load and the offered time.
#[must_use]
pub fn pump_id(load_id: u16, offered_ts: u32) -> String {
	format!("g-{load_id}-{offered_ts}")
}

/// A pump's `pump_offered` once every move of its composition arrived: the moves, the quest and the character's streak
/// as its latest sheet says (the character the load's install numbers so, as the helper's).
fn offer(load_id: u16, p: &Pump, people: &People, install: &str) -> Option<Command> {
	if p.moves.len() != usize::from(p.count) {
		return None;
	}
	let moves = p.moves.iter().map(|(&slot, m)| OfferedMove { slot, move_id: m.fields.move_id, target: m.fields.reps });
	let first = p.moves.values().next().map(|m| &m.fields);
	let streak = first
		.and_then(|f| people.sheets.get(&people.bound(install, f.char_index)?))
		.and_then(|sheet| blob::get(sheet, "streak"))
		.and_then(blob::as_u64)
		.and_then(|days| u16::try_from(days).ok());
	Some(Command::PumpOffered {
		pump_id: pump_id(load_id, p.offered_ts),
		quest_id: first.map(|f| f.quest_id),
		moves: moves.collect(),
		streak,
	})
}

/// A whole pump on offer as one log line: per move its id, the target (0: a max set or a timed move) and its tags, the
/// quest's own moves apart from the unfinished ones riding along (the strip's "owed at a logout" cell, 0 on a rider and
/// in a pump the player asked for) and an all-out window.
fn offered_line(load_id: u16, p: &Pump) -> String {
	let first = p.moves.values().next().map(|m| &m.fields);
	let moves: Vec<String> = p
		.moves
		.values()
		.map(|m| {
			let f = &m.fields;
			let mut tags = Vec::new();
			if f.owes == 0 {
				tags.push("unfinished or asked for");
			}
			if matches!(f.owed_reps, 1 | 2) {
				tags.push("all-out");
			}
			let tags = if tags.is_empty() { String::new() } else { format!(" [{}]", tags.join(", ")) };
			format!("move {} tier {} target {}{tags}", f.move_id, m.tier, f.reps)
		})
		.collect();
	format!(
		"pump {} offered, quest {}: {}",
		pump_id(load_id, p.offered_ts),
		first.map_or(0, |f| f.quest_id),
		moves.join(", ")
	)
}

/// `value` (24 bits, the same units as `ts`) unwrapped next to `clock`, the unwrapped `ts`.
fn relative(clock: u64, ts: u32, value: u32) -> u64 {
	let diff = (i64::from(value) - i64::from(ts)).rem_euclid(i64::from(1 << 24));
	let diff = if diff >= 1 << 23 { diff - (1 << 24) } else { diff };
	u64::try_from(i64::try_from(clock).unwrap_or(i64::MAX).saturating_add(diff)).unwrap_or(0)
}

#[expect(clippy::cast_precision_loss, reason = "game-clock milliseconds stay far below 2^52")]
fn ms_between(from: u64, to: u64) -> f64 {
	to as f64 - from as f64
}

#[expect(clippy::cast_possible_truncation, reason = "a wall-clock time in milliseconds fits an i64")]
fn round_ms(ms: f64) -> i64 {
	ms.round() as i64
}

fn strip_line(f: &RawFrame, fields: &Fields, clock: u64, start_at: Option<u64>, display_time: f64) -> StripLine {
	StripLine {
		seq: f.seq,
		load_id: f.load_id,
		event_no: f.event_no,
		event: f.event,
		state: fields.state,
		quest_id: fields.quest_id,
		level: f.level,
		kind: f.kind,
		tier: f.tier(),
		slot: fields.slot,
		ladder: fields.ladder,
		rung: fields.rung,
		row_flags: fields.row_flags,
		move_id: fields.move_id,
		char_index: fields.char_index,
		outcome: fields.outcome,
		row_ts: (fields.row_ts != 0).then_some(fields.row_ts),
		ts_client_ms: clock,
		start_at_ms: start_at,
		yards_foot: fields.yards_foot,
		yards_mount: fields.yards_mount,
		yards_swim: fields.yards_swim,
		yards_taxi: fields.yards_taxi,
		reps: fields.reps,
		seconds: fields.seconds,
		heartbeat: f.is_heartbeat(),
		reemit: f.flags & 2 == 2,
		display_time,
	}
}
