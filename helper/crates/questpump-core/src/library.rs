//! The constants of `contract/library.json` the helper reads, built in: `tools/gen-engine.lua` writes them from the
//! addon's `Constants.lua`, and `make gen-check` keeps the built-in file current. One read of the file for every module
//! that needs one: the offers' numbers (`offers::tunables`, and through them the life of a `match_end` on the link),
//! the experience question's answers (`body::is_experience`), the paces of Moves per match (`body::is_pace`, slice 48),
//! the retired ladders' successors (`successors`: the ladders a catch-up set's rung rows may name,
//! `session::CatchupSet`), the Stamina Ledger's hold ladders (`hold_goal`: the ladders a catch-up hold's answer to
//! "Could you have held longer?" may come from, slice 52), the longest all-out window (`allout_max_s`: the
//! unfinished list's cap, `OwedMove::of`), the ladders that lost their first rung (`read_rung`: the rung a
//! catch-up's move row takes from its owed row, `Session::catch_up`), each move's body part (`part`: the spread of
//! a catch-up's owed moves, `Session::payable_kept`), each move that runs by its time (`flow`: an owed row whose move
//! is timed now, `session::owed_now`) and the times a row from outside the helper may carry (`row_timely`: every
//! store's intake, `Session::keep`), and since
//! slice 67 the sitting clock's pause (`sitting::away_s`), work mode's tunables (`work::tunables`) and the account's
//! bank numbers (`session::bank_quest`).

use std::collections::BTreeMap;
use std::sync::LazyLock;

use serde::Deserialize;

use crate::codes::kind;
use crate::row::Row;

/// The constants, as library.json names them.
#[derive(Deserialize)]
pub(crate) struct Constants {
	pub link_offer_max_min: i64,
	pub match_offer_min: i64,
	pub match_min_s: i64,
	pub match_min_league_s: i64,
	pub match_checkpoint_min: i64,
	pub match_lead_league_min: i64,
	pub match_relaunch_s: i64,
	pub match_sat_min: i64,
	pub match_step_min: i64,
	pub match_short_min: i64,
	pub match_away_s: i64,
	pub remind_cap: u32,
	pub remind_cap_max: u32,
	pub remind_quiet_from_min: u16,
	pub remind_quiet_to_min: u16,
	pub pause_hour_s: i64,
	pub pause_four_hours_s: i64,
	pub allout_build_s: u16,
	pub allout_s: u16,
	/// The steady ledger's M range in thousandths (slice 52: the dose's multiplier, which took P's place).
	pub ledger_m: [u32; 2],
	/// The experience question's answers (DB.lua's `experience`), each with the rung it starts the ladders on.
	pub start_rung: BTreeMap<String, u8>,
	/// The paces (Moves per quest's and, since slice 48, Moves per match's: light, balanced, extreme), each with the
	/// most moves a pump of each size takes at it (Engine.lua's `quota`).
	pub pace_moves: BTreeMap<String, Vec<u8>>,
	/// The earliest time a row may carry, unix seconds: before QuestPump wrote any row (`row_timely`).
	pub row_min_ts: i64,
	/// How far past the helper's clock a row's time may lie, in seconds (`row_timely`).
	pub row_ahead_s: i64,
	/// How far past it a row that floors or locks the stores may lie, in seconds (`row_timely`).
	pub row_ahead_floor_s: i64,
	/// The seconds of no input (or between two looks) after which the sitting clock starts again, and with it the work
	/// clock's presence (`sitting::away_s`, slice 67).
	pub sit_away_s: i64,
	/// Work mode's (slice 67, `work::Tunables`): Every's default and its choices in minutes, the seconds without
	/// input at which a due work pump asks and after which it asks anyway, and the sitting minutes from which it opens
	/// on its warm move.
	pub work_every_min: u16,
	pub work_every_choices: Vec<u16>,
	pub work_moment_s: i64,
	pub work_late_s: i64,
	pub work_warm_min: i64,
	/// The least bank number, which an owed row of the account's unfinished list carries in its questID
	/// (`session::bank_quest`, slice 67).
	pub bank_quest_min: u32,
}

#[derive(Deserialize)]
struct Library {
	constants: Constants,
	/// Each retired ladder by its id (the pull and the core hold, split on 2026-10-04), with the ladders that took its
	/// moves.
	#[serde(default)]
	split: BTreeMap<String, Split>,
	/// The Stamina Ledger's hold ladders (Progress.lua `ns.HOLD_GOAL`, slice 52).
	#[serde(default, rename = "holdGoal")]
	hold_goal: Vec<u8>,
	/// The ladders that lost their first rung, by id, each with the moment it went (unix seconds; Library.lua
	/// `ns.Shifted`, the wall sit's 135° sit on 2026-10-26).
	#[serde(default)]
	shifted: BTreeMap<u8, i64>,
	/// Each move by its id, with the tags the owed moves are read by.
	#[serde(default)]
	moves: BTreeMap<u16, MoveTags>,
}

/// A move's tags as the owed moves are read by them: its body part's for their spread (Engine.lua `effortPart`), and
/// what a pick of a move that runs by its time carries (`flow`).
#[derive(Deserialize)]
struct MoveTags {
	#[serde(default)]
	region: Option<String>,
	#[serde(default)]
	pattern: Option<String>,
	/// `rep`, `hold`, `cardio` or `flow` (Library.lua's `dose`).
	#[serde(default)]
	dose: Option<String>,
	/// Its ladder; none off the ladders.
	#[serde(default)]
	ladder: Option<u8>,
	#[serde(default)]
	rung: u8,
	/// A flow move's seconds, each side's on a side move (`side`).
	#[serde(default)]
	secs: Option<u16>,
	#[serde(default)]
	side: bool,
}

/// A move that runs by its time (Library.lua's dose `flow`: a pose, a carry, a drill that reaches no form failure) as
/// a fresh pick of it carries it: its ladder (0 off the ladders) and its own rung (Engine.lua `slotOf`), and its
/// seconds, a side move's on each side (Engine.lua `flowSeconds`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Flow {
	pub ladder: u8,
	pub rung: u8,
	pub seconds: u16,
}

#[derive(Deserialize)]
struct Split {
	to: Vec<u8>,
}

static LIBRARY: LazyLock<Option<Library>> =
	LazyLock::new(|| serde_json::from_str::<Library>(include_str!("../../../../contract/library.json")).ok());

/// The constants; none when they do not read (`make check` keeps them readable), and then the computer offers no pump
/// after a game, queues no `match_end` and takes no experience and no Moves per match a device sends.
pub(crate) fn constants() -> Option<&'static Constants> {
	LIBRARY.as_ref().map(|l| &l.constants)
}

/// The longest all-out window, in seconds: its build and its all-out end at the steady ledger's highest M (Engine.lua
/// `ns.AlloutSeconds` at `ledger_m[1]`, 40 s; slice 52, P's cap until then); none when the constants do not read.
pub(crate) fn allout_max_s() -> Option<u16> {
	let c = constants()?;
	let end = (u32::from(c.allout_s) * c.ledger_m[1] + 500) / 1000;
	Some(c.allout_build_s.saturating_add(u16::try_from(end).unwrap_or(u16::MAX)))
}

/// Whether a row from outside the helper's own writing may carry its time at `now` (unix seconds, the helper's clock):
/// no earlier than `row_min_ts` and at most `row_ahead_s` past `now`, and a reset (a phone's Start over among them) or a
/// felt-unwell row (`Row::locks`), which floors the stores or locks every pump until its game day ends, at most
/// `row_ahead_floor_s` (DB.lua `ns.RowTimely`, the phones' `RowTimes`; 2026-10-31, the release's security review), so no
/// crafted one dated in the future floors or locks the stores and every device they reach for long. Every time when the
/// constants do not read (`make check` keeps them readable), as before the bound: a store that kept nothing would lose
/// the player's rows.
pub(crate) fn row_timely(row: &Row, now: i64) -> bool {
	constants().is_none_or(|c| {
		let ahead = if row.kind == kind::RESET || row.locks() { c.row_ahead_floor_s } else { c.row_ahead_s };
		i64::from(row.ts) >= c.row_min_ts && i64::from(row.ts) <= now.saturating_add(ahead)
	})
}

/// The ladders that took a retired ladder's moves, on which the fold reads its rows (DB.lua `ns.Translate`); none for
/// any other ladder.
pub(crate) fn successors(ladder: u8) -> &'static [u8] {
	LIBRARY.as_ref().and_then(|l| l.split.get(&ladder.to_string())).map_or(&[], |s| s.to.as_slice())
}

/// Whether `ladder` is one of the Stamina Ledger's hold ladders (the wall sit, the core, the side plank, the L-sit:
/// `ns.HOLD_GOAL`, slice 52), whose fresh holds may ask "Could you have held longer?" (`ns.HoldAsk`), its answer in the
/// hold row's reps; false for any other ladder or when the library does not read.
pub(crate) fn hold_goal(ladder: u8) -> bool {
	LIBRARY.as_ref().is_some_and(|l| l.hold_goal.contains(&ladder))
}

/// The body part move `move_id` loads (Engine.lua `effortPart`): a rep or hold move's region (push, pull, legs or
/// trunk), else its pattern; none for a move the library lacks.
pub(crate) fn part(move_id: u16) -> Option<&'static str> {
	let m = LIBRARY.as_ref()?.moves.get(&move_id)?;
	m.region.as_deref().or(m.pattern.as_deref())
}

/// Move `move_id` where it runs by its time (`Flow`); none for a move of another dose or one the library lacks.
pub(crate) fn flow(move_id: u16) -> Option<Flow> {
	let m = LIBRARY.as_ref()?.moves.get(&move_id).filter(|m| m.dose.as_deref() == Some("flow"))?;
	let seconds = m.secs.unwrap_or(0).saturating_mul(if m.side { 2 } else { 1 });
	Some(Flow { ladder: m.ladder.unwrap_or(0), rung: m.rung, seconds })
}

/// A row's rung as the fold reads it (DB.lua `ns.ShiftRow`): on a ladder that lost its first rung (`shifted`), one
/// written before that moment (`ts`, unix seconds) one lower, the first as the first; any other as it is.
pub(crate) fn read_rung(ladder: u8, rung: u8, ts: u32) -> u8 {
	match LIBRARY.as_ref().and_then(|l| l.shifted.get(&ladder)) {
		Some(&at) if i64::from(ts) < at && rung >= 1 => rung.saturating_sub(1).max(1),
		_ => rung,
	}
}
