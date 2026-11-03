//! The strip's numeric codes (wow-addon-dev `pixel-strip.md`, the addon's `Bridge.lua` and `Core.lua` `ns.Enum`). An
//! event or state code the table does not name is a decode error, never a silently mapped value; row kinds, outcomes
//! and flags stay numbers, so a row from a newer addon still reaches the mirror exactly as it was written.

use serde::{Deserialize, Serialize};

/// The strip layout's version (cell 2, byte 0); a frame of another version is refused. Version 2 (2026-09-30) paints the
/// same 96 bytes as calibrated 2-bit symbols (`strip.rs`), where version 1 painted each byte as a colour value.
pub const STRIP_VERSION: u8 = 2;

/// What a frame announces (cell 3, byte 0).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum StripEvent {
	Heartbeat,
	CountdownStart,
	QuestComplete,
	TurnIn,
	PumpOffered,
	PumpDone,
	Row,
	Profile,
	Leaving,
	Ping,
	Sheet,
	/// A message the player sent from the game's feedback window (slice 24): painted once, never again.
	Feedback,
	/// Keep going pressed on the running move (slice 52's double-check, 2026-10-24): its slot fields, the seconds the
	/// full move's the clock now runs to.
	Keep,
}

impl StripEvent {
	const ALL: [Self; 13] = [
		Self::Heartbeat,
		Self::CountdownStart,
		Self::QuestComplete,
		Self::TurnIn,
		Self::PumpOffered,
		Self::PumpDone,
		Self::Row,
		Self::Profile,
		Self::Leaving,
		Self::Ping,
		Self::Sheet,
		Self::Feedback,
		Self::Keep,
	];

	#[must_use]
	pub fn from_code(code: u8) -> Option<Self> {
		Self::ALL.get(usize::from(code)).copied()
	}

	/// The profile, the sheet and a feedback message travel in chunks: their frames carry payload where the others
	/// carry fields.
	#[must_use]
	pub fn is_blob(self) -> bool {
		matches!(self, Self::Profile | Self::Sheet | Self::Feedback)
	}
}

/// The pump's state as the addon paints it (cell 11, byte 2).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "snake_case")]
pub enum StripState {
	#[default]
	Idle,
	Countdown,
	Running,
	Input,
	Leaving,
}

impl StripState {
	#[must_use]
	pub fn from_code(code: u8) -> Option<Self> {
		[Self::Idle, Self::Countdown, Self::Running, Self::Input, Self::Leaving].get(usize::from(code)).copied()
	}
}

/// How the game's pump ended (`pump_done`'s outcome byte, cell 12 byte 2; `Bridge.lua` `B.ENDED`, slice 29), which a
/// device reads from the `pump_done` command's `ended`: done shows the pump's result, the others a calm card.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "snake_case")]
pub enum PumpEnd {
	/// Its moves ran, or Bank the rest came after one (an addon before slice 29 paints 0 here, so all its pumps read
	/// as done).
	#[default]
	Done,
	/// Bank it, Bank the rest before a move ran, or the moves gone owed: they come back in a later pump.
	Banked,
	/// The pump's Pause ended it.
	Paused,
	/// "I feel unwell" ended it.
	Unwell,
	/// It ended before a move ran (its last unfinished move skipped, or every move held back meanwhile), an instance or
	/// a mixed party dropped a stand-up pump before it started (it pays no turn-in, so nothing is owed), or a reset
	/// ended it with the save that held it (`/qp reset all`, a restored backup: its moves are owed to no one).
	Skipped,
}

impl PumpEnd {
	#[must_use]
	pub fn from_code(code: u8) -> Option<Self> {
		[Self::Done, Self::Banked, Self::Paused, Self::Unwell, Self::Skipped].get(usize::from(code)).copied()
	}
}

/// Row kinds (`ns.Enum.kind`): the ones the helper acts on.
pub mod kind {
	pub const REP: u8 = 1;
	pub const HOLD: u8 = 2;
	pub const CARDIO: u8 = 3;
	pub const FLOW: u8 = 4;
	pub const DING: u8 = 5;
	pub const PUMP: u8 = 6;
	/// A pain flag set or cleared (`ns.Enum.kind.pain`).
	pub const PAIN: u8 = 9;
	/// A correction of a move's typed count: it repeats the time, quest, slot and move of the row it names (DB.lua's
	/// `corrections`).
	pub const CORRECTION: u8 = 7;
	/// A ladder's rung changed (`ns.Enum.kind.rung`): its ladder and rung, the cause in `reps` (Progress.lua
	/// `ns.RungCause`); a catch-up's set brings the ones its judge added (`Session::catch_up`).
	pub const RUNG: u8 = 8;
	/// The stretch a pump's quests paid (Core.lua `ns.Enum.kind.stretch`, Events.lua `writeStretch`): the yards on foot
	/// in `reps`, the active seconds in `seconds`; the helper adds the walking pad's metres (docs/plan.md slice 25).
	pub const STRETCH: u8 = 13;
	pub const RESET: u8 = 14;
	/// The player's focus (`ns.Enum.kind.focus`).
	pub const FOCUS: u8 = 16;
	/// A record cleared or a ladder started over (`ns.Enum.kind.clear`).
	pub const CLEAR: u8 = 17;
	/// The weight a set of a loaded move was done with (`ns.Enum.kind.load`, 2026-10-09): written just before its set
	/// row, it repeats that row's time, quest, slot and move, with the weight in tenths of a kg in `reps` (DB.lua's
	/// `loads`, folded as a correction is). No move row: the stores pass it as any other kind but a reset.
	pub const LOAD: u8 = 18;
	/// A step mastered (`ns.Enum.kind.mastered`, 2026-10-16, docs/plan.md slice 42): its `ladder` and `rung`, written
	/// once by `ns.Progression` after the set that brought the last move the setup offers there to its top (DB.lua's
	/// fold keeps the first). No move row: the stores pass it as any other kind but a reset.
	pub const MASTERED: u8 = 19;

	/// A move row: a rep, hold, cardio or flow move.
	#[must_use]
	pub fn is_move(kind: u8) -> bool {
		(REP..=FLOW).contains(&kind)
	}

	/// A pump row: a quest's pump or a level-up's.
	#[must_use]
	pub fn is_pump(kind: u8) -> bool {
		kind == PUMP || kind == DING
	}
}

/// Row outcomes (`ns.Enum.outcome`): the ones the helper acts on.
pub mod outcome {
	/// A typed count past the record: a record.
	pub const BEAT: u8 = 1;
	pub const MISS: u8 = 2;
	pub const PLACEMENT: u8 = 3;
	pub const CENSORED: u8 = 4;
	pub const DONE: u8 = 5;
	pub const STOPPED: u8 = 6;
	pub const OFFERED: u8 = 7;
	pub const OWED: u8 = 8;
	pub const BANKED: u8 = 9;
	pub const PAID: u8 = 10;
	pub const LASTBEAT: u8 = 11;

	/// A move row with a typed count (the phones' `RowOutcome.typed`): beat, miss, a placement, censored, last beat.
	#[must_use]
	pub fn is_typed(outcome: u8) -> bool {
		matches!(outcome, BEAT | MISS | PLACEMENT | CENSORED | LASTBEAT)
	}

	/// A move done (`RowOutcome.performed`): typed, done or stopped; owed, banked and paid rows are none.
	#[must_use]
	pub fn is_performed(outcome: u8) -> bool {
		is_typed(outcome) || outcome == DONE || outcome == STOPPED
	}
}

/// A pose's answer to "How close to the goal pose?" (`ns.Enum.mark`, 2026-10-09), in its flow row's `reps`: the least a
/// move's `ask` names counts the day toward its family's next step.
pub mod mark {
	pub const NOT_YET: u8 = 1;
	pub const OKAY: u8 = 2;
	pub const GOOD: u8 = 3;
	pub const PERFECT: u8 = 4;
}

/// A steady move's answer to "Did you keep that pace to the end?" (`ns.Enum.pace`; three answers since slice 52,
/// 2026-10-23: the ask lists them easily, just, eased on keys 1-3), in its cardio row's [`flags::ANSWER`] bits:
/// Keep going saves [`pace::EASILY`] at the full move and [`pace::JUST`] where Done ended it past the plan.
pub mod pace {
	/// "Yes, just" (slice 52; "Yes, the whole time" until then).
	pub const JUST: u8 = 1;
	pub const EASED: u8 = 2;
	/// "Yes, easily" (slice 52).
	pub const EASILY: u8 = 3;
}

/// A counted set's answer to "All out and clean?" (`ns.Enum.answer`, slice 42; Not all out since slice 53), as the
/// engine takes it in a move's result and [`flags::answer`] reads it from a row.
pub mod answer {
	/// "Yes, to the end": its clean day counts.
	pub const YES: u8 = 0;
	pub const SLIPPED: u8 = 1;
	pub const NOT_ALL_OUT: u8 = 2;
}

/// A hold's answer to "Could you have held longer?" (`ns.Enum.longer`, slice 52), in its hold row's `reps` (0 not
/// asked): asked after a fresh hold near its goal on a ladder of the Stamina Ledger (`ns.HoldAsk`,
/// `library::hold_goal`).
pub mod longer {
	pub const NO: u16 = 1;
	pub const LITTLE: u16 = 2;
	pub const LOT: u16 = 3;
}

/// What a clear row clears (`ns.Enum.clear`), in its `reps`: one move's record (its `move`) or one ladder, started over
/// (its `ladder`).
pub mod clear {
	pub const MOVE: u16 = 1;
	pub const LADDER: u16 = 2;
}

/// Why a ladder's rung changed (Progress.lua `ns.RungCause`), in its rung row's `reps`.
pub mod cause {
	pub const UP: u16 = 1;
	pub const PLATEAU: u16 = 2;
	pub const DOWN: u16 = 3;
	pub const PLACEMENT_LOW: u16 = 5;
	pub const PLACEMENT_HIGH: u16 = 6;
	/// `/qp undo`: it reverses the latest rung row of its own character on its ladder, where the rung differs (DB.lua
	/// `undone`), and neither happened.
	pub const UNDO: u16 = 7;
	/// The step down an easier pick makes (`ns.EasierDown`, 2026-10-13): the result of Swap's easier version taken for
	/// the attempt at the rung a placed ladder stands on moves that ladder one rung down, after the set's own rows.
	pub const EASIER: u16 = 8;
	/// A check answered close enough lifts a pose family to the check pose's step (2026-10-16, docs/plan.md slice 42;
	/// `ns.Progression`): a placement the folds take as a climb, never lowering a rung.
	pub const CHECK: u16 = 9;

	/// A climb's cause (Stats.lua `climbRow`): up, a plateau, a placement high and a check's lift (9, a stretch check,
	/// counted since the program's double-check of 2026-10-19, as the game counts it), never a step down, a placement
	/// low, a start or an undo. The one set of them wherever the helper reads a climb (the Saga's unlocks).
	#[must_use]
	pub fn climbs(cause: u16) -> bool {
		matches!(cause, UP | PLATEAU | PLACEMENT_HIGH | CHECK)
	}

	/// A cause a set's own rules give its ladder (Progress.lua `ns.Progression`): up, a plateau, down, a placement's
	/// low or high and a check's (9, 2026-10-16), never character creation's start (4), `/qp undo`'s (7) or the step
	/// down an easier pick on the ladder's own attempt makes ([`EASIER`], which a catch-up brings beside its set for the
	/// move Swap took to its easier version, never in it: `CatchupRan::down`, since 2026-10-26).
	#[must_use]
	pub fn after_set(cause: u16) -> bool {
		matches!(cause, UP | PLATEAU | DOWN | PLACEMENT_LOW | PLACEMENT_HIGH | CHECK)
	}
}

/// The row flags (`ns.Enum.flagBit`): bits 0-2 a reason, then single flags, bits 14-15 a cardio move's answer.
pub mod flags {
	/// Bits 0-2: the stopped or bank reason.
	pub const REASON_MASK: u16 = 0b111;
	/// The reason "felt unwell", the same code as a stop and as a bank reason.
	pub const UNWELL: u16 = 4;
	/// On a typed count: "Is that right?" asked about it and its Yes kept it.
	pub const CONFIRMED: u16 = 1 << 3;
	pub const WATCH: u16 = 1 << 4;
	pub const OWED: u16 = 1 << 5;
	/// A move the player took as Swap's easier version in place of the one shown (contract/feedback-api.yaml's
	/// `easier`).
	pub const EASIER: u16 = 1 << 6;
	pub const PHONE: u16 = 1 << 7;
	/// On a paid row: the player skipped the owed move in the game (`Events.lua` `Lc.Skip`, 2026-10-04), so no move row
	/// comes with it. The store clears the owed row as on any paid row; nothing in the helper reads the bit.
	pub const SKIPPED: u16 = 1 << 9;
	/// A cardio row of a counted set (2026-10-05): its count in `reps`.
	pub const COUNTED: u16 = 1 << 10;
	/// A row an app wrote (docs/plan.md slice 26), with exactly one device's bit: [`PHONE`] on the phone's own, [`WATCH`]
	/// on a watch's (through its phone, or from a watch holding the link), [`COMPUTER`] on the desktop app's. It never
	/// pays or owes a move, so the helper keeps it apart from the game's rows (`app_rows::AppRows`).
	pub const APP: u16 = 1 << 11;
	/// The computer's (docs/plan.md slice 35): with [`APP`] a row of a pump the desktop app ran
	/// (`Session::keep_computer`), without it a row of a catch-up the desktop app ran (`Session::catch_up` with
	/// `Source::Computer`).
	pub const COMPUTER: u16 = 1 << 12;
	/// Bits 14-15 on a cardio move row (2026-10-16, docs/plan.md slice 42; `ns.Answer`): the player's answer after the
	/// move, on a counted set its answer to "All out and clean?" ([`SLIPPED`] Form slipped, [`NOT_ALL_OUT`] since slice
	/// 53; 0 "Yes, to the end", and every counted row from before), on a steady move the answer to "Did you keep that
	/// pace to the end?" ([`super::pace`]; 0 none). The strip carries the row's 16 bits whole.
	pub const ANSWER: u16 = 0b11 << 14;
	/// A counted set saved "Form slipped": its count kept in `reps`, no record and no clean day (DB.lua `cardioMove`).
	pub const SLIPPED: u16 = 1 << 14;
	/// A counted set saved "Not all out" (slice 53, 2026-10-23): as Form slipped, no record and no clean day.
	pub const NOT_ALL_OUT: u16 = 2 << 14;
	/// A counted set's answer code as the engine takes it (`ns.Enum.answer`: 0 Yes, to the end, 1 Form slipped, 2 Not
	/// all out) from its row's flags.
	#[must_use]
	pub fn answer(flags: u16) -> u8 {
		u8::try_from((flags & ANSWER) >> 14).unwrap_or(0)
	}
	/// The judge's own bits a catch-up's set carries onto its move row (`CatchupSet::flags`, since 2026-10-12):
	/// confirmed and, on a counted set, counted and its answer (2026-10-16); never easier, which a catch-up's Swap names
	/// in `CatchupMove::ran` (since 2026-10-26).
	pub const JUDGED: u16 = CONFIRMED | COUNTED | ANSWER;
}
