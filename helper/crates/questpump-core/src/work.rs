//! Work mode (docs/plan.md slice 67): the player's one switch, its settings and its clock's anchor, one state for all
//! the player's devices (`Work`: contract/companion-api.yaml's `Work`, its writes and its merge rule), and the work
//! clock a computer or a browser runs since that anchor (`Clock`), each device its own. Pure: the device hands each
//! look its clock, the anchor, its idle seconds and what holds it now (`Hold`); the clock says whether it counts,
//! holds, is due or asks (`State`). The phones' cores keep their own clocks (`WorkState`: the work apps' minutes, or
//! plain time a walk starts over).
//!
//! The rules, as the plan's slice 67 states them: the clock always runs down, whatever the typing (David, 2026-11-10:
//! "it should just always run down. no matter if there is typing?!"; it counted presence until then, and a pause in
//! the input started the count over): a block is due Every (`work_every_min` by default) after the anchor, less the
//! time held; a hold (a call, presenting, full screen, a game, a pump open, quiet hours, the felt-unwell lock) and a
//! standing ask freeze it, and nothing asks while one holds; once due it asks at the first pause (`work_moment_s`
//! without input), the moment a hold ends, or `work_late_s` after it came due, whichever comes first; a standing ask
//! stands until it is answered (never a second ask, never a bank by itself). Only a move of the anchor (a pump on any
//! device, Later, Start, Stop) starts the count again or drops a block that is due, so an ask nobody answered waits
//! for the player's return, and a changed Every counts from the next anchor. A device keeps a due block across a
//! restart (`Clock::due_from`).

use serde::{Deserialize, Serialize};

use crate::body::{self, PlaceSetup};
use crate::{library, link, places};

/// The longest mover's id a state keeps (`reset_by`, the contract's `maxLength`).
const BY_MOST: usize = 64;
/// The template whose equipment work mode holds until the player changes it (contract/places.json's `office`).
const TEMPLATE: &str = "office";
/// The kind of contract/kinds.json a work pump is (`work`, "A work block"): one work mode composes banks, one picked by
/// hand does not (call 4).
pub const WORK: &str = "work";

/// Work mode's tunables (contract/library.json, slice 67): Every's default and its choices (minutes), the pumps of a
/// set (the last the long one), the seconds without input at which a due pump asks and after which it asks anyway,
/// and the sitting minutes from which a work pump opens on its warm move.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tunables {
	pub every_min: u16,
	pub every_choices: &'static [u16],
	pub set_blocks: u8,
	pub moment_s: i64,
	pub late_s: i64,
	pub warm_min: i64,
}

/// The tunables; none when the built-in constants do not read (`make check` keeps them readable), and then work mode
/// asks nothing.
#[must_use]
pub fn tunables() -> Option<Tunables> {
	library::constants().map(|c| Tunables {
		every_min: c.work_every_min,
		every_choices: &c.work_every_choices,
		set_blocks: c.work_set_blocks.max(1),
		moment_s: c.work_moment_s,
		late_s: c.work_late_s,
		warm_min: c.work_warm_min,
	})
}

/// The player's work state (contract/companion-api.yaml `Work`): times in unix milliseconds on the clock of the device
/// that made the change, 0 before any change.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(from = "Read")]
#[expect(clippy::struct_excessive_bools, reason = "the contract's `Work`: the switch and the equipment's switches")]
pub struct Work {
	/// The switch only the player turns (Start, Stop), and when it last turned.
	pub on: bool,
	pub on_at: i64,
	/// Every (minutes), Moves per block (a pace) and work mode's own equipment, the four fields a place holds (its
	/// gear, keys of contract/sheet.json's `gear` in its `gear_order`, one this helper does not know kept and passed on;
	/// Floor moves, Room to move and Quiet mode), and when one of them last changed.
	pub every_min: u16,
	pub pace: String,
	pub gear: Vec<String>,
	pub floor: bool,
	pub space: bool,
	pub quiet: bool,
	pub settings_at: i64,
	/// The work pumps of this set done or put off with Later (0 to `set_blocks` less 1), the clock's anchor and the id
	/// of the device that moved it.
	pub block: u8,
	pub reset_at: i64,
	pub reset_by: String,
}

impl Default for Work {
	/// Off, Every and Moves per block at their defaults and the office template's equipment (`office`); a kept state
	/// that names no equipment reads as this (one from before 2026-11-09 named a work place instead).
	fn default() -> Self {
		let office = office();
		Self {
			on: false,
			on_at: 0,
			every_min: tunables().map_or(0, |t| t.every_min),
			pace: body::DEFAULT_PACE.to_owned(),
			gear: office.gear,
			floor: office.floor,
			space: office.space,
			quiet: office.quiet,
			settings_at: 0,
			block: 0,
			reset_at: 0,
			reset_by: String::new(),
		}
	}
}

/// Work mode's equipment before the player changes it: contract/places.json's office template's (no equipment, Quiet
/// mode alone on), the addon's defaults where the places do not read (`make check` keeps them readable).
fn office() -> PlaceSetup {
	places::places()
		.and_then(|p| p.templates.iter().find(|t| t.id == TEMPLATE))
		.map_or_else(PlaceSetup::default, |t| t.place().setup)
}

/// A `Work` as the wire and the kept file give it: a field it lacks takes its default, and one that lacks any of the
/// equipment's four fields (a build of 2026-11-08 sent and kept `place` in their stead) reads with its settings part
/// dropped, Every, Moves per block and the equipment at their defaults and `settings_at` 0, so a merge takes another
/// device's alone (the contract's `Work`; the Apple and Android cores read it alike).
#[derive(Deserialize)]
#[serde(default)]
struct Read {
	on: bool,
	on_at: i64,
	every_min: u16,
	pace: String,
	gear: Option<Vec<String>>,
	floor: Option<bool>,
	space: Option<bool>,
	quiet: Option<bool>,
	settings_at: i64,
	block: u8,
	reset_at: i64,
	reset_by: String,
}

impl Default for Read {
	fn default() -> Self {
		let w = Work::default();
		Self {
			on: w.on,
			on_at: w.on_at,
			every_min: w.every_min,
			pace: w.pace,
			gear: None,
			floor: None,
			space: None,
			quiet: None,
			settings_at: w.settings_at,
			block: w.block,
			reset_at: w.reset_at,
			reset_by: w.reset_by,
		}
	}
}

impl From<Read> for Work {
	fn from(r: Read) -> Self {
		let mut w = Self {
			on: r.on,
			on_at: r.on_at,
			block: r.block,
			reset_at: r.reset_at,
			reset_by: r.reset_by,
			..Self::default()
		};
		if let (Some(gear), Some(floor), Some(space), Some(quiet)) = (r.gear, r.floor, r.space, r.quiet) {
			(w.every_min, w.pace, w.settings_at) = (r.every_min, r.pace, r.settings_at);
			w.set_setup(PlaceSetup { gear, floor, space, quiet });
		}
		w
	}
}

/// A write's time: `now`, or just past the time it replaces where this device's clock stands behind it, so a device's
/// own later write always wins its own merge.
fn later(held: i64, now: i64) -> i64 {
	now.max(held.saturating_add(1))
}

impl Work {
	/// Whether any part holds a time: a state none of whose times is set is never sent.
	#[must_use]
	pub fn heard(&self) -> bool {
		self.on_at != 0 || self.settings_at != 0 || self.reset_at != 0
	}

	/// Work mode's own equipment as a place's setup: what every pump a device composes while `on` takes in place of
	/// the place in use's (the engine's profile reads the same four fields).
	#[must_use]
	pub fn setup(&self) -> PlaceSetup {
		PlaceSetup { gear: self.gear.clone(), floor: self.floor, space: self.space, quiet: self.quiet }
	}

	/// Work mode's equipment set to `setup`.
	fn set_setup(&mut self, setup: PlaceSetup) {
		(self.gear, self.floor, self.space, self.quiet) = (setup.gear, setup.floor, setup.space, setup.quiet);
	}

	/// The next work pump's place in the set, 1 to `set_blocks` (the engine's `work.block`; the last the long one).
	#[must_use]
	pub fn next_block(&self, t: &Tunables) -> u8 {
		self.block.min(t.set_blocks - 1) + 1
	}

	/// Start at `now` (ms) by device `by`: on, and the clock's anchor moved.
	pub fn start(&mut self, now: i64, by: &str) {
		self.on = true;
		self.on_at = later(self.on_at, now);
		self.anchor(now, by);
	}

	/// Stop at `now` by `by`: off, the set started again and the anchor moved.
	pub fn stop(&mut self, now: i64, by: &str) {
		self.on = false;
		self.on_at = later(self.on_at, now);
		self.block = 0;
		self.anchor(now, by);
	}

	/// A work pump done on its ask, or a Later, at `now` by `by`: one block more (the set again after its last), and
	/// the anchor moved.
	pub fn block_done(&mut self, now: i64, by: &str, t: &Tunables) {
		self.block = (self.block + 1) % t.set_blocks;
		self.anchor(now, by);
	}

	/// Any other pump done on any device while on, at `now` by `by`: the anchor alone moved.
	pub fn anchor(&mut self, now: i64, by: &str) {
		self.reset_at = later(self.reset_at, now);
		self.reset_by = by.chars().take(BY_MOST).collect();
	}

	/// The Work mode page's or the first Start's choices at `now`: Every, Moves per block and work mode's equipment
	/// (`setup`), each kept where it reads (an equipment whose gear does not read keeps the held one); whether anything
	/// changed.
	pub fn choose(&mut self, every_min: u16, moves_pace: &str, setup: PlaceSetup, now: i64, t: &Tunables) -> bool {
		let every_min = if t.every_choices.contains(&every_min) { every_min } else { self.every_min };
		let moves_pace = if body::is_pace(moves_pace) { moves_pace } else { &self.pace };
		let setup = if setup.checks_out() { setup } else { self.setup() };
		if every_min == self.every_min && moves_pace == self.pace && setup == self.setup() {
			return false;
		}
		(self.every_min, self.pace) = (every_min, moves_pace.to_owned());
		self.set_setup(setup);
		self.settings_at = later(self.settings_at, now);
		true
	}

	/// Another device's state merged into this one at `now` (ms, this device's clock), its three parts apart: the
	/// switch by the later `on_at` (off at a tie), the settings by the later `settings_at` (the held ones at a tie),
	/// the set and the anchor by the later `reset_at` (the greater `reset_by` by its bytes at a tie); a part dated more
	/// than a day past `now`, or whose value does not read, dropped alone. Whether anything changed.
	pub fn merge(&mut self, heard: &Self, now: i64, t: &Tunables) -> bool {
		let timely = |at: i64| at >= 1 && link::not_ahead(at, now);
		let before = self.clone();
		if timely(heard.on_at) && (heard.on_at > self.on_at || (heard.on_at == self.on_at && self.on && !heard.on)) {
			(self.on, self.on_at) = (heard.on, heard.on_at);
		}
		let settings_read =
			t.every_choices.contains(&heard.every_min) && body::is_pace(&heard.pace) && heard.setup().checks_out();
		if timely(heard.settings_at) && settings_read && heard.settings_at > self.settings_at {
			(self.every_min, self.pace) = (heard.every_min, heard.pace.clone());
			self.set_setup(heard.setup());
			self.settings_at = heard.settings_at;
		}
		let set_reads = heard.block < t.set_blocks && heard.reset_by.len() <= BY_MOST;
		let wins = heard.reset_at > self.reset_at
			|| (heard.reset_at == self.reset_at && heard.reset_by.as_bytes() > self.reset_by.as_bytes());
		if timely(heard.reset_at) && set_reads && wins {
			(self.block, self.reset_at, self.reset_by) = (heard.block, heard.reset_at, heard.reset_by.clone());
		}
		*self != before
	}
}

/// The moves of a set's short and long work pump at Moves per block `pace`, as the How page names them: Engine.lua
/// `workQuota`'s, the Normal size's of library.json's `pace_moves` and one more on the set's last; none for a pace
/// that does not read.
#[must_use]
pub fn block_moves(pace: &str) -> Option<(u8, u8)> {
	let short = *library::constants()?.pace_moves.get(pace)?.get(1)?;
	Some((short, short.saturating_add(1)))
}

/// What holds the work clock: nothing counts and nothing asks while one does, and the work card and the tray show it
/// (its words: contract/copy/en.json `work.hold.*`, by its serde name). Each device reads only what needs no
/// permission (slice 67).
#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Hold {
	/// The microphone or the camera in use.
	Call,
	/// Presenting or mirroring the screen.
	Present,
	/// A window in full screen in front.
	#[serde(rename = "fullscreen")]
	FullScreen,
	/// A game in front (World of Warcraft's window too), a match running, a match's offer standing or the game's quest
	/// pump open.
	Game,
	/// A pump open.
	Pump,
	/// The player's quiet hours.
	Quiet,
	/// The felt-unwell lock (this device's own).
	Unwell,
}

/// One look's facts: now in unix seconds, the clock's anchor (`Work::reset_at`, unix milliseconds, so every move of it
/// is seen, one of a millisecond too), the device's seconds without input (none where it reads none: a due block's
/// moment, never its count) and what holds it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Seen {
	pub now: i64,
	pub anchor_ms: i64,
	pub idle_s: Option<u32>,
	pub hold: Option<Hold>,
}

/// What the work clock came to at a look.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
	/// Counting: the seconds of presence left before the next block is due.
	Counting { left_s: i64 },
	/// Held, with the seconds left as they stood.
	Held { hold: Hold, left_s: i64 },
	/// Due, waiting for its moment.
	Due,
	/// An ask stands; `new` at the look that made it.
	Ask { new: bool },
}

/// The work clock of one device: the time since the anchor, less the time held since then.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Clock {
	/// The anchor the count runs after (`Seen::anchor_ms`); none before the first look.
	anchor_ms: Option<i64>,
	/// The moment the count runs from (unix seconds): the anchor, or the first look's `now` where the anchor stood
	/// ahead of this device's clock (or the clock was set back past it since).
	base: i64,
	/// The seconds held since `base`.
	held_s: i64,
	/// The last look, and whether a hold or an ask held it.
	last: Option<(i64, bool)>,
	/// Whether a hold held the last look (its end is a moment).
	hold_last: bool,
	/// When the block came due.
	due_at: Option<i64>,
	/// An ask stands.
	asking: bool,
}

impl Clock {
	/// One look at `seen` with Every at `every_min`. The count starts again (and a due block or a standing ask goes)
	/// only when the anchor moved: no pause in the input and no time without a look does. An anchor ahead of `now`
	/// (another device's clock ahead, this one set back) counts from the first look that saw it.
	pub fn look(&mut self, seen: &Seen, every_min: u16, t: &Tunables) -> State {
		let now = seen.now;
		let (last, hold_last) = (self.last, self.hold_last);
		if self.anchor_ms != Some(seen.anchor_ms) {
			*self = Self { anchor_ms: Some(seen.anchor_ms), base: seen.anchor_ms.div_euclid(1_000), ..Self::default() };
		}
		self.base = self.base.min(now);
		let base = self.base;
		let freed = hold_last && seen.hold.is_none();
		if let Some((at, true)) = last {
			self.held_s += (now - at.max(base)).max(0);
		}
		self.last = Some((now, seen.hold.is_some() || self.asking));
		self.hold_last = seen.hold.is_some();
		// a block due stays due whatever Every reads now: a changed Every counts from the next anchor
		let left_s = if self.due_at.is_some() {
			0
		} else {
			(i64::from(every_min) * 60 - (now - base - self.held_s).max(0)).max(0)
		};
		if let Some(hold) = seen.hold {
			// a hold puts a standing ask aside (a match's offer goes first, a pump opened, full screen): it asks again
			// the moment the hold ends
			self.asking = false;
			return State::Held { hold, left_s };
		}
		if self.asking {
			return State::Ask { new: false };
		}
		if left_s > 0 {
			return State::Counting { left_s };
		}
		let due_at = *self.due_at.get_or_insert(now);
		let paused = seen.idle_s.is_some_and(|s| i64::from(s) >= t.moment_s);
		if freed || paused || now - due_at >= t.late_s {
			self.asking = true;
			return State::Ask { new: true };
		}
		State::Due
	}

	/// The standing ask answered (Pump now, Later, Stop): the clock waits for the anchor to move, and asks again at its
	/// next moment while it does not (a pump left with nothing done).
	pub fn answered(&mut self) {
		self.asking = false;
	}

	/// The ask stands again where the block is due (a Later that banked nothing: an engine that failed, no move to
	/// bank), so its Pump now and Later stay.
	pub fn stand(&mut self) {
		self.asking |= self.due_at.is_some();
	}

	/// The anchor a due block (its ask standing or waiting for its moment, or held since) counts after
	/// (`Seen::anchor_ms`); none while the block still counts.
	#[must_use]
	pub fn due(&self) -> Option<i64> {
		self.due_at.and(self.anchor_ms)
	}

	/// The anchor of a due block and the moment it came due (unix seconds), which a device keeps across a restart
	/// (`Clock::due_from`); none while the block still counts.
	#[must_use]
	pub fn due_since(&self) -> Option<(i64, i64)> {
		Some((self.anchor_ms?, self.due_at?))
	}

	/// The clock of a block that came due at `due_at` (unix seconds) after the anchor `anchor_ms`, as a device kept it
	/// before a restart or a reload: due still at its first look while the anchor stands, so it asks at its next moment
	/// or at once where `work_late_s` passed since; a moved anchor starts the count again as any look does.
	#[must_use]
	pub fn due_from(anchor_ms: i64, due_at: i64) -> Self {
		Self { anchor_ms: Some(anchor_ms), base: anchor_ms.div_euclid(1_000), due_at: Some(due_at), ..Self::default() }
	}
}
