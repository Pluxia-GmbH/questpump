//! The pump on the computer (docs/plan.md slice 35; docs/ui.md, "The pump on the computer"): the phones' flow screen
//! for screen (their `PumpRunner`'s app pumps, Quick pumps and catch-ups), with Rust owning the state and the time. The
//! machine keeps the pump's stages (Ready, the countdown, the running side, the timed ask, the count (a counted set's
//! form step since slice 42), a pose's "How close to the goal pose?", a steady move's "Did you keep that pace to the
//! end?" (slice 42), the end) and every instant as unix ms; each deadline (the next digit, the go, a timed side's
//! end, a mark, the Result's reveal) is the next wake of the runtime loop's select (`Helper::pump_wake`, a sleep made
//! again after every change, and at least every `BEAT_MS` through a countdown or a timed move), never a page timer, so
//! a hidden or covered window keeps the clock and the voice; a countdown or a timed move the computer slept through
//! (the loop woke over `LATE_WAKE_MS` late) goes back to its Ready view, nothing written. The addon's own engine
//! composes, judges and folds through the ordered door (`desk/engine.rs`): `quick`, `compose` (a plain kind's family
//! from contract/kinds.json, as the phones map it) and, since slice 38, `move` (a move the player started from a
//! step's card or a move's window: its start read again as Start is pressed, `Engine::chosen`, one Light pump that pays
//! and owes nothing, its row an asked pump's), Swap and its way back, the Ready view's Pain and what it leaves, a count
//! checked against the game's limits and finished as the game judges it, the felt-unwell stop; their rows go to
//! `Session::keep_computer` as written, whatever is on screen when the engine answers (the engine holds them already).
//! Felt unwell locks every pump at once (`Session::lock_after`), before the stop's row is written. A catch-up pays the
//! owed moves no pain flag rests (the flags asked of the engine as it starts) through `Session::catch_up` with the
//! computer's source, only while no World of Warcraft client with QuestPump runs, which asks them itself (one that
//! starts ends the catch-up at once, its moves left owed, as the game's own pump ends one); its felt-unwell stop is
//! written all the same, and so it is when a reset or the lock refuses its moves. Since slice 58 a match's pump banks
//! like a quest's: Bank all puts its own moves left in the game's owed queue (`Session::bank`) and ends it as End the
//! pump here does, and the system's quit banks them too (`pump_quit`). Since slice 61 it carries the unfinished moves
//! this computer may pay as a quest's pump in the game carries them: composed with the queue (`Unfinished`), its
//! riders come after its own moves (an own all-out window last where no owed window rides), each judged and paid as a
//! catch-up's move (the rider mark, `Run::rider`) under an id of their own at every end, Bank all banks its own moves
//! alone, and Bank the rest on a rider ends it as done with its Result, the rest left unfinished. Since 2026-10-12
//! (David: it "did not ask me how many I could do") a catch-up asks the count of a rep move, a hold or a counted set
//! (an owed window of a countable form) as an app pump does, each dosed on the engine as
//! the game doses an owed move (`Engine::catch_up`: every owed row the catch-up may pay read as the fold reads it and
//! dosed as the riders of one pump, each at its owed ladder and rung) and its count judged there as the game's own
//! catch-up judges it (`Engine::judge`), written nowhere: the payment names each move's owed row and sends each saved
//! count as its move's set (`CatchupSet`), and the engine lets the judged counts go just before the rows the session
//! wrote for them merge in (`Engine::settle`); a timed move, a pose and a window of a form that counts nothing run by
//! their time, as before. Every rider the engine places swaps on its Ready view as an app pump's move does, with the
//! options the game gives an owed move (a catch-up's that asks its count since 2026-10-26, David: "for me the offered
//! frog stand will be too hard but I cannot siwtch to an easier version"; since 2026-11-02 one of every kind, a match
//! pump's too, David: "Why is there still no alternative exercise to switch to like in wow addon"): the move swapped in
//! asks only what its own kind asks, and the move Swap ran pays its owed row (`CatchupMove::ran`), as since 2026-10-29
//! the move an owed backup the setup leaves unneeded rides as does (Engine.lua `ridesAs`).
//! While the engine checks or judges a count, Back waits and an early end waits for its answer
//! (`Held`), so a count saved is never lost. The gates: no start while felt unwell locks pumps, while that client runs,
//! before the engine has folded, or before the place was answered on this computer (`settings.place_asked`, the
//! welcome's place window), and the lock or that client closes the Start a pump window and drops a start waiting for
//! the place; a match offer taken back on its game's start again ends what it opened while nothing ran, and a watched
//! game's new run or a new match ends whatever was opened from outside the window while it stands untouched (slice 48's
//! double-check; `pump_recalled`: that window, that start, a compose, the pump at Ready, a match's banking its moves
//! since 2026-10-27); "Open the pump after each match" ends the pump here while it is idle, as Leave ends it, and
//! never one whose move runs (`pump_make_way`, 2026-10-28); a pump the game offers on the strip ends a pump here, as
//! it ends one on the wrist. The cues go
//! to DESK-MEDIA's
//! `Helper::cue` (the countdown's digits, the go and a timed end, a move's name on Ready and its callout after it,
//! Stats.lua's `ns.PickCallout` through the engine, the cadence, an all-out window's line, "Switch sides.", the marks,
//! a timed side's tick on each of its last five seconds and a hold's chime as it passes its goal (Panel.lua's
//! `lastFive` and its `win`), a catch-up's record or the game's small chime as its count is saved (Panel.lua
//! `winSound`), and a running set of reps' count on its figure's beat, `figures::beat`, a side move's on each side's
//! own run, with the Sound and voice page's "Count reps aloud": `Helper::counts_on`), each `Cue::Event` for an event's
//! desktop sound and `Cue::Line` for a voice line; a move's How and Tips, read aloud at the player's button, go to the
//! guide's own player (`guide.rs`). While a pump runs the window's close and QuestPump's quit ask "Leave the pump?"
//! (`Desk::guard`), the computer stays awake (`desk/awake.rs`), and the tray says a pump is on; with none, the tray
//! offers Quick pump and Catch up where they can start.
//!
//! **The `pump` topic** (`view.rs` holds its shapes): `gate` (`lock`, `game`, `engine` or null: why no pump starts
//! now), `locked_until` (ms), `owed` (the moves a catch-up here pays), `refused` (the starts that went nowhere on this
//! run, a count the page that pressed one reads), the windows Rust asks for (`place`: where the player pumps, which the
//! welcome's page draws; `choose`: the Start a pump window with its preset), `pain` (the joints with their flags)
//! and `pump`, the pump on screen: its id, what it is (`quick`, `free`
//! with its `kind`, `move` with `from`, where its Result returns, or `catchup`), its stage with the instants the page
//! draws from (`go` and `digit`, `start` and `limit`, `asked`), a side move's `side` (`first` or `other`, the side on:
//! Big view's name and countdown line say it, its figure is mirrored on the other and Done reads Switch sides on the
//! first), its moves as their lines show them (each rider marked, slice 61; a loaded move's with its weight; a set of
//! reps the voice counts with its count's rule, `rep_count`, from which Big view draws the reps counted each frame, so
//! no rep changes the topic), the
//! Ready view's `rewards`, `lines` and `swap` (the swap row's offers in the game's order), the move on's `setup`, `how`
//! and `tips` until the end and the read-aloud
//! button's `guide`, the count step (a loaded move's with its weight box; a set of reps the voice counted from the reps
//! counted by Done), and at its end `result` (the fanfare, the XP and its bar, the reward rows, the moves done, a
//! loaded move's next set, the reveal's instants) or `calm`. The day-2 check-in is Pump home's (`desk/home.rs`).
//!
//! **The intents** (`{"type": "pump.<what>", ...}`): `pump.quick`, `pump.start {kind, size, offer?}`, `pump.catchup`,
//! `pump.move {target: {ladder, rung} | {move} | {check}, from: {place: ladder|move|list, ladder?, step?,
//! opens?}}` (a step's card's or a move's window's Start, slice 38; since slice 42 a stretch check's, "Do the check",
//! with what it opens; the gates and the place window hold it as every app pump),
//! `pump.choose {kind?, size?, offer?}` (the Start a pump window), each with `outside` when the tray or the card asked
//! (QuestPump's window opens for it), and a size missing taken from the offer (its tier, slice 48:
//! `offer_tier`), else Balanced; a start from an offer that goes through answers its card (`offer_started`); a plain
//! kind's pump from an offer, or a match picked by hand, composes with a game's end's facts (`match_facts`: the
//! match's minutes and its warm move, slice 57, and the arms rest while an aim game with its switch on runs as it
//! starts, which its Ready view names, `pump.rest.arms`); `pump.dismiss {window: choose}`;
//! `pump.ready.start`,
//! `pump.done` (Done, or Switch sides), `pump.back`, `pump.timed {whole}`,
//! `pump.mark {answer}` (a pose's answer to "How close to the goal pose?", 1 Not yet to 4 Perfect),
//! `pump.pace {answer}` (a steady move's answer to "Did you keep that pace to the end?", `ns.Enum.pace`: 1 Yes, just,
//! 2 I eased off, 3 Yes, easily; slice 52), `pump.keep` (Keep going, slice 52), `pump.longer {answer}` (a hold's answer
//! to "Could you have held longer?", `ns.Enum.longer`: 1 No, 2 A little, 3 A lot; slice 52), `pump.count {value,
//! load?, answer?}` (`load` a loaded move's weight in tenths of a kg, none or 0: no weight; on a counted set its
//! answer, `ns.Enum.answer`: 0 Yes, to the end, 1 Form slipped, 2 Not all out, slice 53),
//! `pump.count.clear` ("Change it"), `pump.count.none` (Done without a count),
//! `pump.count.couldnot`, `pump.swap {take: another|easier|backup|back}` (the swap row and its window, on any move the
//! engine gives options), `pump.pain {joints}` (the Ready view's Pain: its Pause menu's Something hurts, on every
//! pump since 2026-11-05, a catch-up resting the moves it loads), `pump.pause {kind}` (its Pause menu's how long: the
//! offers paused as `games.pause` pauses them, then the pump ends as `pump.end` ends it),
//! `pump.skip` (a rider's Ready view's Skip: a catch-up's since 2026-10-09, a match pump's rider since slice 61),
//! `pump.bank` (Bank all on a match pump's Ready view while a move of its own is left, Bank the rest on a catch-up's
//! and once only a match pump's riders are left, slices 58 and 61: `Helper::pump_bank`),
//! `pump.unwell`, `pump.end` (End the pump here past the Ready view, on a rider the "···"'s Bank the rest; the Ready
//! bar's Close on a pump that banks nothing),
//! `pump.leave {then: close|quit}` and `pump.stay` ("Leave the pump?"'s
//! answers), `pump.fullscreen {on}` (Big view's own toggle), `pump.land {still?}` (the Result's reveal at its end),
//! `pump.saved.land {still?}` (a move's pour at its end, then its step's reveal; slices 65 and 66),
//! `pump.saved.next` (its Next or Finish), `pump.window {open}` (a window stands over the page, or none does any more:
//! `Pumps::window`; a page that starts says none, which frees a wait a page that went left),
//! `pump.continue` and `pump.guide` (the read-aloud button on Ready and in Big view's foot: begin, pause or go on
//! reading the move's How and Tips, `guide.rs`). A start takes no linked game (`link`): the desktop's pumps reach every
//! Log from their rows alone, which carry none (a phone keeps its own pump's link in its own history).

mod expedition;
mod guide;
mod run;
mod view;
mod words;

pub(crate) use run::{CountStep, Outcome, Typed};
pub(crate) use view::{Against, CheckView, check_view, count_most, max_line};

use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use questpump_core::body::{self, DEFAULT_PACE};
use questpump_core::codes::{answer, clear, flags, kind, outcome, pace};
use questpump_core::games::MATCH;
use questpump_core::link::OwedMove;
use questpump_core::offers::Offer;
use questpump_core::protocol::{ExpeditionStage, PumpKind, RowRecord, Source};
use questpump_core::row::Row;
use questpump_core::session::{
	BankMove, BankPump, CatchupError, CatchupMove, CatchupPump, CatchupRan, CatchupSet, SetRung, SetStep, owed_now,
};
use questpump_core::sitting;
use questpump_engine::{
	CardioDay, Check, CheckOpens, CheckOutcome, Composed, Dosed, Engine, EngineError, Finished, Judged, Judgement,
	Match, MoveResult, PlannedMove, Preview, Pump, Rider, Slot, StartedCheck, SwapOptions, Take, Target, Toward,
	Unfinished,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::menu::MenuItem;
use tauri::{AppHandle, Manager as _};

use super::awake;
use super::facts::facts;
use super::figures;
use super::home::Board;
use super::seams::{Cue, Recalled, StoreMark};
use super::tray::TrayLines;
use super::windows::{self, Landing, MAIN};
use super::{Desk, DeskAsk, Intent};
use crate::copy;
use crate::lan;
use crate::runtime::{self, Helper};
use crate::ui::{self, Shared, WindowAsk};
use run::{Beats, Cap, CatchMove, Due, End, Entered, Mark, Marks, MoveRun};

/// The topic.
const TOPIC: &str = "pump";
/// The tray's items' ids, `pump.` and the intent's what.
const TRAY_ON: &str = "pump.on";
const TRAY_QUICK: &str = "pump.quick";
const TRAY_CATCHUP: &str = "pump.catchup";
/// A mark due longer ago than this when the loop wakes (a sleep, a stall) is skipped, never said late; so is a rep's
/// count, or one due longer ago than half the way to the next count, which said later would run into it.
const MARK_LATE_S: f64 = 1.0;
/// How often at least the loop looks at a countdown or a running timed move (ms, `Helper::pump_wake`), beside its own
/// instants, so the time a sleep took always shows as a late wake.
const BEAT_MS: i64 = 1_000;
/// A wake of the loop this much later than the instant the pump asked for (ms) is the computer's sleep or a jump of its
/// clock forward, never the loop's own delay (its turns take milliseconds): a countdown or a running timed move then
/// goes back to its Ready view (`Helper::pump_slept`), since the wall clock its instants keep ran on through the sleep,
/// and slept time never counts as a move's (no more than this bound and one beat of it can).
const LATE_WAKE_MS: i64 = 5_000;
/// The size an app pump starts at when none is named and no offer gives one (Balanced, tier 2).
const NORMAL: u8 = 2;
/// The felt-unwell reason a running timed move's row carries (`ns.Enum.stopped.unwell`).
const UNWELL: u8 = 4;
/// The kind of contract/kinds.json a quest's turn-in pumps (the cores' `PumpKinds.quest`; `tools/copytext.py` keeps it
/// a kind): its Result says Quest complete, every other app pump's Pump complete.
const QUEST: &str = "quest";

/// The engine's answers to an app pump's start: its moves as a device runs them, what each can earn, Swap's options for
/// its first move, and the app's own sheet and titles as it begins.
struct Started {
	moves: Vec<PlannedMove>,
	previews: Vec<Preview>,
	swaps: Option<SwapOptions>,
	before: Value,
	titles: Vec<u8>,
}

impl Started {
	/// The engine's answers for the app pump it just composed, at `at`.
	fn read(e: &Engine, at: i64) -> Result<Self, EngineError> {
		Ok(Self {
			moves: e.moves()?,
			previews: e.preview(at)?,
			swaps: e.swap_options(0, at)?,
			before: e.summary(at)?,
			titles: e.titles()?,
		})
	}
}

/// An app pump being composed: what it is, its id, the match offer that opened it, where it was asked, the aim game its
/// arms rest is for and, on a match's pump, the owed rows it was composed with (slice 61: `Unfinished::owed`, as the
/// store keeps them).
struct Asking {
	what: What,
	id: String,
	offer: Option<String>,
	outside: bool,
	arms: Option<String>,
	riding: Vec<Row>,
}

/// A pump the engine composes now (an app pump or a catch-up), until it answers: its id, the match offer that opened
/// it, and whether it was asked from outside the window. A recall takes the mark (`Helper::pump_recalled`), and the
/// pump then shows nowhere.
struct Composing {
	id: String,
	offer: Option<String>,
	outside: bool,
}

/// What starts a pump: the engine's one Quick pump (the welcome's first pump alone since 2026-11-03, when every other
/// Quick pump became the timed pump, `Expedition`), an app pump for a plain kind at a size (with the match offer that
/// opened it), a move started from a step's card or a move's window (slice 38: its `target` as the Book asked it, and
/// where its Result returns to), a catch-up, or the Start a pump window with its preset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Start {
	Quick,
	Free {
		kind: String,
		size: u8,
		offer: Option<String>,
	},
	Move {
		target: Target,
		from: StartedFrom,
	},
	CatchUp,
	Choose(Choose),
	/// An expedition of `minutes` (slice 51, `expedition.rs`): its rounds one after another.
	Expedition {
		minutes: u16,
	},
}

impl Start {
	/// The match offer that opened it.
	fn offer(&self) -> Option<&str> {
		match self {
			Self::Free { offer, .. } | Self::Choose(Choose { offer, .. }) => offer.as_deref(),
			Self::Quick | Self::Move { .. } | Self::CatchUp | Self::Expedition { .. } => None,
		}
	}
}

/// Where a move pump started, which its Result's Continue names and returns to (slice 38): the tree (the started
/// step, by its ladder and its step on the one scale: the step whose card started it, or for a start that ran another
/// ladder, `via`, that ladder's step), a ladder's page, a move's window or page, the Book's list (a ladder that
/// waits whole, whose card opened from it) or Pump home's cardio row (2026-11-03); and on a stretch check (slice 42), what passing it opens, by which the
/// engine finds the check the pump runs (`StartedCheck`, `ns.RunsCheck`), which the Result names.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct StartedFrom {
	place: Place,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	ladder: Option<u8>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	step: Option<u8>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	opens: Option<Opens>,
}

/// What a stretch check opens (slice 42; the card or the window that started it knows): the move waiting on it, or a
/// climb's check the move standing for the step whose climb it opens with that step on the one scale (`step`) and,
/// since slice 42's fix round (2026-10-17), that climb's ladder and rung, or a family not open yet (`family`, with its
/// first step's move). The engine puts the check that names it first among the pose's checks (`Engine::checks`).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Opens {
	#[serde(rename = "move")]
	move_id: u16,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	step: Option<u8>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	ladder: Option<u8>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	rung: Option<u8>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	family: Option<u8>,
}

impl Opens {
	/// As the engine's checks name what a check opens: the climb into `rung` of `ladder`, a family, or the move.
	fn engine(self) -> CheckOpens {
		match (self.ladder, self.rung, self.family) {
			(Some(ladder), Some(rung), _) => {
				CheckOpens { ladder: Some(ladder), rung: Some(rung), ..CheckOpens::default() }
			}
			(_, _, Some(family)) => CheckOpens { family: Some(family), ..CheckOpens::default() },
			_ => CheckOpens { r#move: Some(self.move_id), ..CheckOpens::default() },
		}
	}
}

/// The page a move pump started from ([`StartedFrom`]).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum Place {
	Tree,
	Ladder,
	Move,
	List,
	Home,
}

/// The Start a pump window's preset: the kind chosen (a match's offer) or none (a program's close, Pump home's quiet
/// Start a pump), the size, and the offer that opened it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Choose {
	pub(crate) kind: Option<String>,
	pub(crate) size: u8,
	pub(crate) offer: Option<String>,
}

/// A start and where it was asked: `outside` the window (the tray, the card), which opens the window for it and whose
/// pump a recall ends while it stands untouched.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Asked {
	start: Start,
	outside: bool,
}

/// What the pump on screen is: the Quick pump, a plain kind's, a move the player started (`from` where, `target` what
/// its Start asked, `doses` what the start read then doses), or a catch-up.
#[derive(Debug, Clone, PartialEq, Eq)]
enum What {
	Quick,
	Free {
		kind: String,
	},
	Move {
		from: StartedFrom,
		target: Target,
		doses: Option<Doses>,
	},
	CatchUp,
	/// A round of an expedition (slice 51): its stage and, for a round between the warm-up and the cool-down, its number.
	Expedition {
		stage: ExpeditionStage,
		round: Option<u8>,
	},
}

/// What a move pump's start doses (`questpump_engine::Start`): its move, its kind, the window alone, the ladder and the
/// rung it is dosed at and its goal. A ladder started over on another device before its first move may move it
/// (`Helper::pump_started_over`); the start's other fields (its day and fresh lines) change nothing on screen.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Doses {
	r#move: Option<u16>,
	kind: Option<String>,
	allout: bool,
	ladder: Option<u8>,
	rung: Option<u8>,
	goal: Option<i64>,
}

impl Doses {
	fn of(start: &questpump_engine::Start) -> Self {
		Self {
			r#move: start.r#move,
			kind: start.kind.clone(),
			allout: start.allout,
			ladder: start.ladder,
			rung: start.rung,
			goal: start.goal,
		}
	}
}

impl What {
	/// An app pump, its own moves written as the app's rows (a match pump's riders among them pay as a catch-up's moves,
	/// slice 61: `Run::rider`); every pump but a catch-up.
	fn app(&self) -> bool {
		!matches!(self, Self::CatchUp)
	}

	/// What its offered row keeps of it (`Session::keep_computer`), which the Saga titles: a move pump's kind as asked,
	/// an expedition's round with its stage (slice 51).
	fn kind(&self) -> Option<PumpKind> {
		match *self {
			Self::Move { .. } => Some(PumpKind::Move),
			Self::Expedition { stage, round } => Some(PumpKind::Expedition { stage, round }),
			Self::Quick | Self::Free { .. } | Self::CatchUp => None,
		}
	}

	/// A stretch check's pump (slice 42: the start's kind `check`, "Do the check"), with the check its target named (a
	/// check row's; a card's or a move window's own Start of a check names its step or its move) and what the card or
	/// window that started it names it opening, from which `Engine::checks` puts the check the pump runs first
	/// (`ns.RunsCheck`: the one its climb waits for first where one does); none for any other.
	fn check(&self) -> Option<StartedCheck> {
		match self {
			Self::Move { from, target, doses: Some(d) } if d.kind.as_deref() == Some("check") => {
				let need = match *target {
					Target::Check { check } => Some(check),
					Target::Step { .. } | Target::Move { .. } | Target::Haul { .. } => None,
				};
				Some(StartedCheck { need, opens: from.opens.map(Opens::engine) })
			}
			_ => None,
		}
	}
}

/// What an app pump is composed from: the Quick pump, a plain kind's family (with the match offer that opened it), or a
/// move from a step's card or a move's window, read again from its target as the engine composes it.
enum Compose {
	Quick,
	Free { family: &'static str, offer: Option<String> },
	Move { target: Target },
}

/// What the engine answered a compose: the pump with its start's facts (none for a pump with no move) and, on a move
/// pump, the start it read (`ns.StartFor`: its `as` for `tree_day`, what it doses for `Doses`); or why a move's start
/// refuses now.
enum Made {
	Pump(Box<(Composed, Option<Started>, Option<questpump_engine::Start>)>),
	Refused(Option<String>),
}

/// The stage on screen, its instants in unix ms.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Stage {
	Ready,
	/// Towards the go; the digit once the first has come (none in the beat before it).
	Countdown {
		go: i64,
		digit: Option<i64>,
	},
	/// The running side since its go.
	Running {
		start: i64,
	},
	/// A timed move Done under the ask's share of its time: "Did you do the whole move?".
	Asking {
		seconds: i64,
		full: i64,
	},
	/// The count of a rep move, a hold or a counted set that ran its time.
	Counting,
	/// A pose whose move asks ran its whole time (`seconds`): "How close to the goal pose?" (2026-10-09), nothing written
	/// until its answer (`pump_mark`) or the pump's early end (`pump_unanswered`).
	Marking {
		seconds: i64,
	},
	/// A steady move with its pace ask (slice 42, 2026-10-17: the slot's `pace`, the game day's first on the player's
	/// own timed cardio step) finished its time (`seconds`): "Did you keep that pace to the end?", nothing written until
	/// its answer (`pump_pace`) or the pump's early end (`pump_unanswered`).
	Pacing {
		seconds: i64,
	},
	/// A fresh hold near its goal saved (slice 52, the engine's `ns.HoldAsk`): "Could you have held longer?", the set as
	/// typed and checked (`confirmed`: "Is that right?" asked and its Yes kept it), nothing written until its answer
	/// (`pump_longer`) or the pump's early end (`pump_unanswered`), which writes it with none.
	Longer {
		typed: Typed,
		confirmed: bool,
	},
	/// A move saved (slice 65's hold, every move since slice 66): its count or seconds pour into the count bar
	/// (`Run::pour`), its row written as the pour lands (`pump_pour_land`), then its rewards, until Next or Finish
	/// (`pump_saved_next`).
	Saved,
	/// The pump is over: its Result or its calm card.
	Done,
}

/// What a finished move recorded, for its line: a count with the weight it was done with (tenths of a kg), a counted
/// set's clean reps (0: not counted) and its answer to "All out and clean?" (`ns.Enum.answer`: 0 Yes, to the end, 1
/// Form slipped, 2 Not all out; slice 42, the third since slice 53), a timed move's seconds, or done without a count.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Recorded {
	Count(i64, Option<u32>),
	Counted(i64, u8),
	Time(i64),
	Done,
}

/// Why a pump ended before its end, short of feeling unwell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cut {
	/// The player ended it (End the pump here, Leave) with a move done.
	Left,
	/// The game offered a pump on the strip.
	Game,
}

/// How a catch-up's moves went to the game's store.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Payment {
	Paid,
	/// The game or another device paid them meanwhile: nothing written.
	Unpaid,
	/// The progress was reset after the catch-up began (its owed moves were the progress reset): nothing written.
	Reset,
	/// Felt unwell locked the pumps before it began (a stop that reached QuestPump while it ran): nothing written.
	Locked,
	/// A World of Warcraft client with QuestPump runs: it asks these moves itself.
	GameRuns,
}

/// An early end asked while the engine checks or judges a catch-up's count, or works out the step down of a move done
/// without one (`Run::saving`), done once the check keeps the count step or the engine answers (`Helper::pump_unhold`),
/// so the count saved is paid as its move's set and the step down goes with it: End the pump here, a World of Warcraft
/// client with QuestPump starting, the game's offer or felt unwell (`stop`; felt unwell never gives way to another),
/// and "Leave the pump?"'s Leave with the quit that follows it (`leave`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct Held {
	stop: Option<Stop>,
	leave: Option<bool>,
}

/// The side's cues: the last mark made on this side's clock and the next one, the last of a timed side's last five
/// seconds ticked (`run::last_five`, its seconds left), whether a hold's goal chimed, on a set of reps that counts its
/// beats and its next count (`Due`), whether Keep going shows (`keep`, slice 52: a steady move's last
/// `keep_going_s` under the full move, from `keep_from`).
#[derive(Debug, Clone, Default, PartialEq)]
struct Cues {
	last: Option<f64>,
	next: Option<(f64, Mark)>,
	five: Option<i64>,
	goal: bool,
	beats: Option<Beats>,
	rep: Option<Due>,
	keep: bool,
}

/// The pump on screen.
#[expect(clippy::struct_excessive_bools, reason = "the pump's own facts, one field each")]
struct Run {
	id: String,
	what: What,
	outside: bool,
	moves: Vec<PlannedMove>,
	/// The engine's slot of each move, by its `slot` (the rung it is dosed at and a first max set's top): an app pump's
	/// every move's, a catch-up's for the moves that ask a count (`Engine::catch_up`) and a move Swap put in
	/// (`pump_swapped`), none for the rest.
	slots: Vec<Option<Slot>>,
	/// The cardio rung of each move by its position in `moves` (a catch-up's Skip takes it out with its move), for its
	/// cadence and its pace's words (an app pump's from its slot, a catch-up's from its owed row).
	rungs: Vec<u8>,
	/// The owed row each move that pays one pays, by the move's `slot` (the owed move and its row's time, which its
	/// payment and its Skip name): every move of a catch-up (its plan's), and a match pump's riders (slice 61: the
	/// engine's `Pump::riders`, read from `riding` by `Run::ride`); none for a move of an app pump's own. Its keys are
	/// the rider mark (`Run::rider`).
	owed: BTreeMap<usize, CatchMove>,
	/// The owed rows a match pump was composed with (`Unfinished::owed`, slice 61), each as a catch-up runs it, by its
	/// place there, from which its riders' owed rows are read again after every change to its moves; none on every
	/// other pump.
	riding: Vec<CatchMove>,
	/// The id a match pump's riders are paid under (slice 61: `Session::catch_up`, a second `d-` id made with the pump,
	/// never `id`, under which `Session::bank` banks its own moves, since each refuses an id the store holds); none
	/// without riders (a catch-up is paid under its own `id`).
	riders_id: Option<String>,
	/// The go of a match pump's first rider (unix ms), which its riders' payment is dated from (a catch-up's from
	/// `pump_start`).
	riders_start: Option<i64>,
	previews: Vec<Preview>,
	current: usize,
	stage: Stage,
	side: Option<MoveRun>,
	ran: BTreeMap<usize, i64>,
	recorded: BTreeMap<usize, Recorded>,
	/// A set with a weight: whether the engine judged it a record (its row's outcome), which the Result names, since a
	/// weighted record is a score the reps alone cannot tell.
	beat: BTreeMap<usize, bool>,
	/// The riders' counts saved (a catch-up's, a match pump's riders'), as the engine judged them (`Engine::judge`), by
	/// slot: each paid as its move's set.
	sets: BTreeMap<usize, CatchupSet>,
	/// The riders' step downs, by slot (since 2026-10-26): the ladder and the rung Swap's easier version's result steps
	/// down to as the engine judged it (`Engine::judge`, or `Engine::empty_done` for a move done without a count), each
	/// paid in its move's `ran.down`.
	downs: BTreeMap<usize, SetStep>,
	/// The answers by slot: a pose's to "How close to the goal pose?" (`ns.Enum.mark`) and a steady move's to "Did you
	/// keep that pace to the end?" (`ns.Enum.pace`), which the Result reads (a stretch check passed).
	answers: BTreeMap<usize, u8>,
	/// The rows the engine wrote for the pump's moves, in order: the Result reads the steps they mastered and a check's
	/// lift from them (slice 42).
	written: Vec<Row>,
	/// The cardio days its moves brought, in order, each by its move's slot (`Engine::finish`'s, Progress.lua
	/// `ns.CardioDay`): its move's Result row's clean day, full move, Long Haul day, a measure's step held or back, and the
	/// climb that waits.
	days: Vec<(usize, CardioDay)>,
	/// The week's embers where one of its moves lit the game day's last ember in the app's own fold
	/// (`Finished::kindled`, slice 55, once a game day of that fold): the Result's "Cardio done today" toast, its week
	/// never fewer than Hero's vault card says (`pump_result`).
	kindled: Option<i64>,
	/// Today's embers as it began (`Helper::embers_today`) and the embers its own moves lit since (`Finished::lit`, a
	/// catch-up's judged at its end by `Engine::lights`): its Result's Cardio today counts them where neither the app's
	/// fold nor the game's sheet holds them yet (a catch-up's rows, the game's sheet before its next load; 2026-10-29).
	embers_before: i64,
	embers_lit: i64,
	/// A pose's stretch checks by slot (slice 42's fix round, 2026-10-17): the slots whose pose could pass one as its ask
	/// opened (`Engine::checks`, `pump_checks`), and what they came to after its answer (`Engine::checked`: the Result's
	/// "Check passed" rows, or what still holds what one opens).
	checks: BTreeSet<usize>,
	checked: BTreeMap<usize, Vec<CheckOutcome>>,
	/// What the shown pose's answer counts toward as its Ready view read it (slice 47: the family's days its day note
	/// counts, Panel.lua `poseDaysLine`), by the slot and the move it was read for (`pump_ready_days`): an app pump's
	/// pose that asks how close it got, read again for the move a Swap or the Ready view's Pain brings.
	ready_toward: Option<(usize, u16, Toward)>,
	/// A rider's count on its way to its set, from the check of the count typed (`pump_count`) to the judge's answer
	/// (`pump_judge`), or until the check keeps the count step, and a move done without a count until the engine names
	/// its step down (`pump_empty`): Back waits, and an early end is `held` until then.
	saving: bool,
	/// An app pump's own move saved and on its way to its pour while the sheet before its write is read (`pump_pour`),
	/// with what its line shows: an early end writes it as its landing would (`pump_pour_left`).
	pouring: Option<(MoveResult, Recorded)>,
	held: Held,
	count: CountStep,
	/// Where the count's stepper starts (`pump_finish_move`): a set of reps the voice counted at the reps counted by
	/// Done, a hold at its stopwatch's seconds, else at the count to reach, the record or 0.
	count_from: i64,
	/// What the app counted of the move on its count step (`pump_finish_move`): a set of reps' reps the figure ended by
	/// Done, a hold's stopwatch, the weaker side's on a side move; none where it counted nothing (slice 66).
	counted: Option<i64>,
	/// The engine holds it (an app pump's compose, a catch-up's `Engine::catch_up` that answered), so Swap's options
	/// are asked of it (`pump_swaps`); not a catch-up whose `Engine::catch_up` failed, whose moves each run by their
	/// time.
	on_engine: bool,
	/// Swap's options for the move on Ready as the engine gave them last (the swap row's offers), none until they come.
	swaps: Option<SwapOptions>,
	/// The moves Swap changed, by slot (a catch-up's since 2026-10-26, a match pump's since slice 61), as the engine
	/// says after each swap (`Dosed::swapped`, its `Pump.swapped`): a rider among them pays its owed row as the move it
	/// ran (`Run::ran_instead`), until its way back is taken (to a move that is not the owed one, it pays as that move
	/// all the same).
	swapped: BTreeSet<usize>,
	/// A catch-up's moves that run by their time (`timed_move`, no slot) while Swap put another in their place, by
	/// slot, each as it showed with its cardio rung: the way back puts it back so, since the engine holds no slot of
	/// it (`pump_swapped`).
	unswapped: BTreeMap<usize, (PlannedMove, u8)>,
	cut: Option<Cut>,
	unwell: bool,
	pump_start: Option<i64>,
	last_end: Option<i64>,
	move_start: Option<i64>,
	/// The timed move asked about: its side's go, the seconds into that side, and its end.
	asked: Option<(i64, f64, i64)>,
	/// What the current move's Start found, which Back restores: the pump's first go, its riders' first go and its last
	/// move's end.
	before_start: Option<(Option<i64>, Option<i64>, Option<i64>)>,
	/// Engine calls under way for it.
	busy: u8,
	/// The app's own sheet and its titles as the pump began.
	before: Option<view::Sum>,
	titles_before: Vec<u8>,
	/// The move last read aloud on its Ready view (index and move).
	spoken: Option<(usize, u16)>,
	/// Its callout while it waits on its Ready view: when it is due (ms, after the move's line), its family and its take;
	/// and when the callout said last ends (ms), which a Start before then hushes (none on the countdown).
	callout: Option<(i64, String, String)>,
	callout_ends: Option<i64>,
	cues: Cues,
	/// Big view's full screen was put on for this pump.
	full: bool,
	done_at: Option<i64>,
	result: Option<view::Earned>,
	reveal: Option<view::Reveal>,
	/// A move's pour on its `Saved` stage (slice 65's hold, every move's since slice 66).
	pour: Option<view::pour::Pour>,
	/// The Legendary moment on show over the Result (2026-10-09): the index of its orange or gold step's row among the
	/// Result's rows; Continue walks them in order before the Result closes.
	legend: Option<usize>,
	paid: Option<Payment>,
	/// The streak's board as it began (Pump home's; the phones' `streakBefore`), and what a catch-up reached by it as it
	/// paid (an app pump's goes into its Result as that is read).
	streak_before: Option<Board>,
	reached: view::Reached,
	/// The match offer that opened it (`m-<uuid>`, `m-test`), which a recall of it ends while it is untouched, and
	/// whose pump banks like a quest's (slice 58: `Helper::pump_bank_way`).
	offer: Option<String>,
	/// The aim game whose arms rest it composed with (its name), which its Ready view names (`pump.rest.arms`); the
	/// unfinished moves that rest holds back and every owed all-out window wait for a later pump, never riding it (slice
	/// 61, Engine.lua `pickRiders`).
	arms: Option<String>,
	/// Its offered row's time (unix seconds) and tier, which the rows of the moves it banks carry (`Session::bank`).
	offered: Option<(u32, u8)>,
}

impl Run {
	/// A pump on its first move's Ready view.
	fn new(
		id: String,
		what: What,
		outside: bool,
		moves: Vec<PlannedMove>,
		slots: Vec<Option<Slot>>,
		rungs: Vec<u8>,
	) -> Self {
		Self {
			id,
			what,
			outside,
			moves,
			slots,
			rungs,
			owed: BTreeMap::new(),
			riding: Vec::new(),
			riders_id: None,
			riders_start: None,
			previews: Vec::new(),
			current: 0,
			stage: Stage::Ready,
			side: None,
			ran: BTreeMap::new(),
			recorded: BTreeMap::new(),
			beat: BTreeMap::new(),
			sets: BTreeMap::new(),
			downs: BTreeMap::new(),
			answers: BTreeMap::new(),
			written: Vec::new(),
			days: Vec::new(),
			kindled: None,
			embers_before: 0,
			embers_lit: 0,
			checks: BTreeSet::new(),
			checked: BTreeMap::new(),
			ready_toward: None,
			saving: false,
			pouring: None,
			held: Held::default(),
			count: CountStep::default(),
			count_from: 0,
			counted: None,
			on_engine: true,
			swaps: None,
			swapped: BTreeSet::new(),
			unswapped: BTreeMap::new(),
			cut: None,
			unwell: false,
			pump_start: None,
			last_end: None,
			move_start: None,
			asked: None,
			before_start: None,
			busy: 0,
			before: None,
			titles_before: Vec::new(),
			spoken: None,
			callout: None,
			callout_ends: None,
			cues: Cues::default(),
			full: false,
			done_at: None,
			result: None,
			reveal: None,
			pour: None,
			legend: None,
			paid: None,
			streak_before: None,
			reached: view::Reached::default(),
			offer: None,
			arms: None,
			offered: None,
		}
	}

	/// The pump runs: from its Ready view to its end (its Result or calm card is no longer running).
	fn running(&self) -> bool {
		self.stage != Stage::Done
	}

	/// Its first move waits on its Ready view with nothing done and no engine call under way.
	fn untouched(&self) -> bool {
		self.stage == Stage::Ready
			&& self.current == 0
			&& self.ran.is_empty()
			&& self.recorded.is_empty()
			&& self.busy == 0
	}

	/// Nothing runs in it and nothing is asked: a Ready view not started (its first move's, one between moves, a
	/// catch-up's) or its end, with no count being saved and no engine call under way. A game's pump takes its place
	/// (`Helper::pump_make_way`).
	fn idle(&self) -> bool {
		matches!(self.stage, Stage::Ready | Stage::Done) && !self.saving && self.busy == 0
	}

	fn shown(&self) -> Option<&PlannedMove> {
		self.moves.get(self.current)
	}

	/// The full move the shown steady move runs on to while Keep going runs it past its plan (slice 52,
	/// `PlannedMove::keep`), whose end saves it as its Done does (`Helper::pump_kept`); none for any other move.
	fn kept_to(&self) -> Option<i64> {
		self.side.as_ref().filter(|s| s.kept()).and(self.shown()?.keep)
	}

	/// A kept steady move's end the engine judges as a rider's set (`Helper::pump_kept`): its run over, the stage
	/// still `Running` until the answer.
	fn kept_judged(&self) -> bool {
		matches!(self.stage, Stage::Running { .. }) && self.side.is_none()
	}

	/// Whether the engine dosed move `m` (`slots`): it asks its count as the game does, an app pump's every move (its
	/// riders too), a catch-up's rep moves, holds and counted sets the engine doses and a move Swap put in a catch-up,
	/// which asks what its own kind asks (a pose or a timed move nothing: `pump_finish_move`).
	fn dosed(&self, m: &PlannedMove) -> bool {
		self.slots.get(m.slot).is_some_and(Option::is_some)
	}

	/// Whether the move in `slot` pays an unfinished move (`owed`): every move of a catch-up and a match pump's rider
	/// (slice 61), each judged on the engine and paid as its owed row, never finished as an app row; a move of an app
	/// pump's own is none.
	fn rider(&self, slot: usize) -> bool {
		self.owed.contains_key(&slot)
	}

	/// The move on screen is a rider (`Run::rider`).
	fn on_rider(&self) -> bool {
		self.shown().is_some_and(|m| self.rider(m.slot))
	}

	/// Only riders are left (the game's `Lc.OnlyRidersLeft`, slice 61): every move from the one on screen on is a
	/// rider or one of the pump's own with its result, so a way out banks nothing new and reads Bank the rest.
	fn only_riders_left(&self) -> bool {
		self.moves.iter().skip(self.current).all(|m| self.rider(m.slot) || self.recorded.contains_key(&m.slot))
	}

	/// The pump's end reads Bank the rest (`Ways::rest`, `pump_end`): only riders are left of a catch-up or of a
	/// match pump (one the player started holds none).
	fn ends_rest(&self) -> bool {
		(self.offer.is_some() || !self.what.app()) && self.only_riders_left()
	}

	/// A match pump's riders as the engine's pump names them now (`Pump::riders`, slice 61): each slot's owed row, read
	/// from the rows it was composed with (`riding`), as it was composed and after every Swap, Skip or Pain that moved
	/// its moves. Only for an app pump (a catch-up's `owed` is its plan's).
	fn ride(&mut self, riders: &[Rider]) {
		self.owed = riders.iter().filter_map(|r| Some((r.index, *self.riding.get(r.row)?))).collect();
	}

	/// A match pump's rider in `slot` leaves it (its Skip, or a World of Warcraft client with QuestPump that starts and
	/// takes it back, slice 61), as the engine's `Pump:skip` takes it out of the slots: the move, its slot, its rung, its
	/// preview and its swap go, and every later move one place down with the owed row it pays and its swap (the engine's
	/// `shiftDown` of `Pump.swapped`). Every move after it is still to come, so none holds a result yet.
	fn drop_move(&mut self, slot: usize) {
		let Some(i) = self.moves.iter().position(|m| m.slot == slot) else { return };
		self.moves.remove(i);
		for m in self.moves.iter_mut().filter(|m| m.slot > slot) {
			m.slot -= 1;
		}
		if slot < self.slots.len() {
			self.slots.remove(slot);
		}
		if i < self.rungs.len() {
			self.rungs.remove(i);
		}
		if i < self.previews.len() {
			self.previews.remove(i);
		}
		self.owed = std::mem::take(&mut self.owed)
			.into_iter()
			.filter(|&(s, _)| s != slot)
			.map(|(s, o)| (if s > slot { s - 1 } else { s }, o))
			.collect();
		self.swapped = std::mem::take(&mut self.swapped)
			.into_iter()
			.filter(|&s| s != slot)
			.map(|s| if s > slot { s - 1 } else { s })
			.collect();
	}

	/// A catch-up's move at `i` leaves it unrun (its Skip, its Pain): the move, its rung and the owed row it pays go,
	/// its other moves keeping their slots, as the engine's catch-up keeps their places.
	fn leave_catch_up(&mut self, i: usize) {
		if i >= self.moves.len() {
			return;
		}
		let slot = self.moves.remove(i).slot;
		if i < self.rungs.len() {
			self.rungs.remove(i);
		}
		self.owed.remove(&slot);
	}

	/// The id a rider's payment goes under: a match pump's `riders_id`, a catch-up's own `id`.
	fn pay_id(&self) -> &str {
		self.riders_id.as_deref().unwrap_or(&self.id)
	}

	/// The move a rider `slot` (a catch-up's move or a match pump's rider) ran in place of the owed move it pays
	/// (`CatchupMove::ran`): where Swap put it there (since 2026-10-26), or its slot's move is another than that owed
	/// move (since 2026-10-29: an owed backup of its rung the setup leaves unneeded rides as that rung's move, Engine.lua
	/// `ridesAs`, which Swap's Back to returns to): the slot's move, row kind, ladder and rung, whether it is Swap's
	/// easier version and the step down its result made; none where the owed move runs. The owed move is `owed`'s, which
	/// the payment and Skip name.
	fn ran_instead(&self, slot: usize) -> Option<CatchupRan> {
		let s = self.slots.get(slot)?.as_ref()?;
		let owed = self.owed.get(&slot)?;
		if !self.swapped.contains(&slot) && s.r#move == owed.move_id {
			return None;
		}
		Some(CatchupRan {
			move_id: s.r#move,
			kind: s.kind,
			ladder: s.ladder,
			rung: s.rung,
			easier: s.easier,
			down: self.downs.get(&slot).copied(),
		})
	}

	/// A max set (`PlannedMove.maxSet`): a rep move or a hold the engine dosed with no count to reach and no record to
	/// beat; a catch-up's move the engine did not dose is one set.
	fn max_set(&self, m: &PlannedMove) -> bool {
		self.dosed(m) && m.target.is_none() && m.record.is_none() && (m.kind == kind::REP || m.kind == kind::HOLD)
	}

	/// The most the count of `m` takes on its count step where the app counted it (`CountStep::cap`, slice 66), every
	/// count path through it: the dial's + stops there, and a count entered past it is saved as it (`Helper::pump_count`).
	/// A hold's count is its stopwatch, real time, which stands alone. A set of reps' runs at the figure's pace, far slower
	/// than a fast honest set, so its share also counts from the number it has to beat, the one the dial's arc stands
	/// against (stages.js `against`: the goal, else the record); a first max set and a placement plan neither, and their
	/// count, the player's baseline, takes no share at all.
	fn count_cap(&self, m: &PlannedMove) -> Option<Cap> {
		let c = facts().constants();
		let most = view::count_most(m.counted, m.kind == kind::HOLD);
		if m.kind == kind::HOLD {
			return CountStep::cap(self.counted, None, most, c.count_over_share, c.min_seconds);
		}
		let beat = view::goal(m).into_iter().chain(m.record).max()?;
		CountStep::cap(self.counted, Some(beat), most, c.count_over_share, c.min_reps)
	}

	/// A first max set (Progress.lua `ns.FirstMaxSet`, `PlannedMove::first_max`), as the game's short rule reads it: a
	/// rep move or a hold the engine dosed as a placement, or as a max set with no record. Never `max_set` alone: that
	/// also holds for a move with a record whose form changed since (`ns.Goal`), which banks when short as any set does.
	fn first_max_set(&self, m: &PlannedMove) -> bool {
		self.dosed(m) && m.first_max && (m.kind == kind::REP || m.kind == kind::HOLD)
	}

	/// Back is on every stage after Start, until the count is saved (a rider's count or pace, or a kept steady
	/// move's end, while the engine checks or judges it; a saved move's while its pour runs, its row unwritten, slice 65).
	fn can_back(&self) -> bool {
		match self.stage {
			Stage::Counting | Stage::Pacing { .. } | Stage::Longer { .. } | Stage::Running { .. } => !self.saving,
			Stage::Countdown { .. } | Stage::Asking { .. } | Stage::Marking { .. } => true,
			Stage::Saved => self.pour.as_ref().is_some_and(|p| !p.landed()),
			Stage::Ready | Stage::Done => false,
		}
	}

	/// The pump ended done, as the QP 10 focus ask reads it (the phones' `HomeView.ownDone`, the game's `focusDue` after a
	/// pump done or one that banked its rest): an app pump over, with no felt-unwell stop and no cut but the player's
	/// own end with a move done, and a move ran or was counted (a Pain on the Ready view that ends it ends it with none).
	fn ended_done(&self) -> bool {
		self.stage == Stage::Done
			&& self.what.app()
			&& !self.unwell
			&& matches!(self.cut, None | Some(Cut::Left))
			&& !(self.ran.is_empty() && self.recorded.is_empty())
	}

	/// The pump ended with its result: no calm end and a move ran.
	fn resulted(&self) -> bool {
		self.stage == Stage::Done
			&& !self.unwell
			&& self.cut.is_none()
			&& !(self.ran.is_empty() && self.recorded.is_empty())
	}

	/// Its bank (slice 58, the game's `Lc.Bank`): each move of its own with no row yet, from the one on screen on (a
	/// rider stays the owed row it is, slice 61), as its slot dosed it, under its id with the computer's source, dated
	/// and sized by its offered row (now and Balanced without one); and the slot of each move banked, in their order.
	fn bank(&self) -> (BankPump, Vec<usize>) {
		let now = u32::try_from(unix_s(lan::now_ms()).max(0)).unwrap_or(u32::MAX);
		let (start, tier) = self.offered.unwrap_or((now, NORMAL));
		let n = |v: Option<i64>| u16::try_from(v.unwrap_or(0).max(0)).unwrap_or(u16::MAX);
		let left: Vec<&PlannedMove> = self
			.moves
			.iter()
			.skip(self.current)
			.filter(|m| !self.rider(m.slot) && !self.recorded.contains_key(&m.slot))
			.collect();
		let moves = left
			.iter()
			.map(|m| {
				let dosed = self.slots.get(m.slot).and_then(Option::as_ref);
				BankMove {
					move_id: m.r#move,
					kind: m.kind,
					ladder: dosed.map_or(0, |s| s.ladder),
					rung: dosed.map_or(0, |s| s.rung),
					seconds: n(m.seconds),
					allout: m.allout,
				}
			})
			.collect();
		let pump = BankPump { source: Source::Computer, id: self.id.clone(), start, tier, moves };
		(pump, left.iter().map(|m| m.slot).collect())
	}
}

/// The pump on the computer: the one on screen and what the page is asked to show beside it (`Helper::pumps`).
#[derive(Default)]
#[expect(clippy::struct_excessive_bools, reason = "the loop's marks between turns, each a yes or no of its own")]
pub(crate) struct Pumps {
	run: Option<Run>,
	/// The expedition the pump on screen is a round of (slice 51).
	trip: Option<expedition::Trip>,
	/// A start the engine composes now: none more until it answers.
	starting: bool,
	/// The pump the engine composes now, until it answers: a recall takes one asked from outside
	/// (`Helper::pump_recalled`), and the pump then shows nowhere.
	composing: Option<Composing>,
	/// A start waiting for the place window's answer.
	waiting: Option<Asked>,
	/// The starts that went nowhere on this run (`Helper::pump_nowhere`, `Pumps::nowhere`), counted for the `pump`
	/// topic.
	refused: u32,
	/// The Start a pump window asked for, and whether from outside the window.
	choose: Option<(Choose, bool)>,
	/// The pain flags as the engine last said (`pain_flags`), for the topic's owed moves and Something hurts' joints;
	/// a catch-up asks them again as it starts.
	flags: Vec<u8>,
	/// The stores as the flags were last asked for (`Helper::store_mark`): rows came since when they differ, and the
	/// flags are `stale` until the engine can answer.
	read: Option<StoreMark>,
	stale: bool,
	/// The last voice line other than a count (ms): a mark's count due within `mark_gap_s` of it ticks instead (a rep's
	/// count keeps its beat: nothing else speaks while a set of reps runs).
	line_at: i64,
	/// Each callout family's take said last on this computer: the next is another (Speech.lua's `lastTake`).
	takes: BTreeMap<String, String>,
	/// The computer is held awake.
	awake: bool,
	/// A pump runs that a setup change waits for (anything but an app pump on its Ready view, `Helper::pump_refits`),
	/// as Your setup was last told (its `pump_on`).
	setup_waits: bool,
	/// The tray's items as last drawn: a pump on, Quick pump, Catch up.
	tray: (bool, bool, bool),
	/// When the loop last looked at the pump's clock (ms): a countdown or a timed move asks to be looked at again within
	/// `BEAT_MS` (`pump_wake`), so a look far later is a sleep (`pump_slept`).
	looked: i64,
	/// An app pump that ended done was put away (its Result's Continue): the QP 10 focus ask's cue, read once by Hero's
	/// next look (`Helper::pump_focus_due`), as the phones ask as that Result leaves.
	focus_due: bool,
	/// The move on's How and Tips read aloud (`guide.rs`), and whether its button shows: the stage reads one, and the
	/// install the app follows holds the move's takes (read again at each publish).
	guide: Option<guide::Guide>,
	offered: bool,
	/// A window stands over the page, as the page last said (`Pumps::window`).
	covered: bool,
}

impl Pumps {
	/// A window came over the page or the last one left it, as the page says (`pump.window`; its start says none, and the
	/// main window's end does in its place, so a window that went without a word holds nothing): a move's pour waits
	/// under one (`Pour::cover`), and one that stood before the pour was planned counts (a "Leave the pump?" as a timed
	/// move ends by its clock).
	pub(crate) fn window(&mut self, open: bool, now: i64) {
		self.covered = open;
		if let Some(pour) = self.run.as_mut().and_then(|r| r.pour.as_mut()) {
			pour.cover(open, now);
		}
	}

	/// A pump is on screen and runs: from its Ready view until its moves are done, or an expedition's breather between
	/// two rounds.
	fn on(&self) -> bool {
		self.run.as_ref().is_some_and(Run::running) || self.trip.as_ref().is_some_and(expedition::Trip::resting)
	}

	/// A start went nowhere (`Helper::pump_nowhere`, a gate closing on a start that waits for the place, a recall of
	/// its match offer): one more on the topic's count.
	fn nowhere(&mut self) {
		self.refused = self.refused.wrapping_add(1);
	}
}

/// The tray's items, made once on the main thread.
struct Tray {
	on: MenuItem<tauri::Wry>,
	quick: MenuItem<tauri::Wry>,
	catch_up: MenuItem<tauri::Wry>,
	shown: [AtomicBool; 3],
}

/// Why no pump starts now.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Gate {
	/// Felt unwell locks every pump until the next game day.
	Lock,
	/// A World of Warcraft client with QuestPump runs: the game asks the pumps.
	Game,
	/// The engine folds every row (at the start, after a reset) or did not start.
	Engine,
}

/// The Ready view's way to put its moves off (slice 58, the game's bar, `Panel.lua` `renderBar`): Bank all on a match's
/// pump while a move of its own is left, Bank the rest once only its riders are left (slice 61, `Lc.OnlyRidersLeft`),
/// and Bank the rest on every catch-up.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
enum Bank {
	It,
	Rest,
}

/// The start of the id of every pump this computer runs (`new_id`).
const OWN_PUMP: &str = "d-";

/// A pump's id on the computer: `d-` and a version-4 UUID.
pub(crate) fn new_id() -> String {
	let mut random = [0u8; 16];
	rand::fill(&mut random);
	format!("{OWN_PUMP}{}", uuid::Builder::from_random_bytes(random).into_uuid())
}

/// Whether an app row is of a pump this computer ran (`Session::keep_computer` with a `new_id`): one the server, a
/// backup file or a phone brought has no pump id, and another computer's pump reaches this one as its game rows.
pub(crate) fn ran_here(r: &RowRecord) -> bool {
	r.pump.as_deref().is_some_and(|p| p.starts_with(OWN_PUMP))
}

/// The helper's clock in unix seconds.
fn unix_s(ms: i64) -> i64 {
	ms.div_euclid(1000)
}

/// Whether a count entered on `m` is too short to count (`Helper::pump_short`): 1 or 2 reps (a counted set's count
/// too, `min_reps`), a hold under `min_seconds`; 0 stays no count or "Could not do one".
fn short_count(m: &PlannedMove, value: i64) -> bool {
	let c = facts().constants();
	value > 0 && value < if m.kind == kind::HOLD && !m.counted { c.min_seconds } else { c.min_reps }
}

/// The id a short move's bank goes under (`Helper::pump_short`, contract/companion-api.yaml `WristEvent`'s `bank`):
/// `Session::bank` writes once an id, and the pump's own stays free for its Bank all, its quit and its make way.
fn short_id(pump: &str, slot: usize) -> String {
	format!("{pump}~s{slot}")
}

/// A timed move's result (`PlannedMove.result`): its seconds up to the planned ones, stopped short when it ended before
/// them.
fn timed_result(m: &PlannedMove, seconds: i64) -> MoveResult {
	match m.seconds {
		Some(planned) => MoveResult {
			value: seconds.min(planned),
			elapsed: seconds,
			stopped: seconds < planned,
			..MoveResult::default()
		},
		None => MoveResult { value: seconds, elapsed: seconds, ..MoveResult::default() },
	}
}

/// The second on the running side's clock from which Keep going shows (slice 52, Events.lua `Lc.KeepShows`): the last
/// `keep_going_s` of a steady move whose slot runs it on to the full move (`keep`, never a window, a counted set, a warm
/// move or a Long Haul: Engine.lua `keepSlot`), on one side and not kept yet; none once it shows or for any other move.
fn keep_from(run: &Run, m: &PlannedMove) -> Option<f64> {
	m.keep?;
	let side = run.side.as_ref().filter(|s| s.side().is_none() && !s.kept() && !run.cues.keep)?;
	Some(side.limit()? - run::real(facts().constants().keep_going_s))
}

/// The longest steady move, at the steady ledger's top M (Engine.lua `ns.CardioSeconds`; slice 52, P's cap until then):
/// an owed steady move runs no longer (`CatchMove::of`).
fn cardio_max() -> i64 {
	let c = facts().constants();
	run::whole((c.cardio_cap_s * c.ledger_m[1] / 1000.0).round())
}

/// A catch-up's move that asks no count (a timed move, a pose, an all-out window of a form that counts nothing, or one
/// the engine did not dose) as a device runs it at `slot`: done by its time, with no goal and no weight.
fn timed_move(slot: usize, m: &CatchMove) -> PlannedMove {
	let text = facts().text(m.move_id);
	PlannedMove {
		slot,
		r#move: m.move_id,
		kind: m.kind,
		target: None,
		goal: None,
		seconds: m.seconds,
		allout: m.allout,
		build: m.build,
		side: text.is_some_and(|t| t.side),
		record: None,
		form: None,
		form_load: None,
		top: None,
		placing: None,
		first_max: false,
		counted: false,
		pace: false,
		keep: None,
		haul: false,
		warm: false,
		mark: false,
		ask: None,
		load: None,
		load_up: false,
		load_from: None,
		load_top: None,
		load_each: false,
		start: None,
		implement: None,
		top_load: None,
		top_reps: None,
		last_load: None,
		last_reps: None,
		next: None,
		today: None,
	}
}

/// A catch-up's set as the engine judged it, as its payment sends it (`WristEvent`'s `set`): the judge's own bits it
/// carries (`flags::JUDGED`), its rung rows and the steps it mastered, saved at `at` (unix ms on the helper's clock).
fn catchup_set(judged: &Judged, at: i64) -> CatchupSet {
	let n = |v: i64| u16::try_from(v.max(0)).unwrap_or(u16::MAX);
	CatchupSet {
		reps: n(judged.reps),
		seconds: n(judged.seconds),
		outcome: judged.outcome,
		flags: judged.flags & flags::JUDGED,
		load: judged.load.map(|l| u16::try_from(l).unwrap_or(u16::MAX)),
		at,
		rungs: judged.rungs.iter().map(|g| SetRung { ladder: g.ladder, rung: g.rung, cause: g.cause }).collect(),
		mastered: judged.mastered.iter().map(|m| SetStep { ladder: m.ladder, rung: m.rung }).collect(),
	}
}

/// A set of reps' beat (ms): its figure's loop at its real tempo (`figures::beat`; a timing variant has a film of its
/// own), which Big view's figure is phased to on every set of reps and the voice counts on (`rep_count`; the phones'
/// `RepCount.counts`); none for any other kind (a counted set is a cardio move) or a move whose figure has no loop.
fn rep_beat(m: &PlannedMove) -> Option<i64> {
	if m.kind != kind::REP || m.counted {
		return None;
	}
	figures::beat(&facts().text(m.r#move)?.key).map(run::millis).filter(|&ms| ms > 0)
}

/// The ms in a set of reps' loop at which each rep but the last ends (`figures::rep_ends`: a dead bug's two sides);
/// none for a loop of one rep.
fn rep_ends(m: &PlannedMove) -> Vec<i64> {
	facts().text(m.r#move).map(|t| figures::rep_ends(&t.key).into_iter().map(run::millis).collect()).unwrap_or_default()
}

/// A running set of reps' count (Speech.lua repCount; the phones' `RepCount`): on its figure's beat (`rep_beat`) up to
/// `max_reps`, past the goal, a max set's too, a side move's on each side's own run from its go (David, 2026-10-10:
/// "Does the bulgarian split squat have counting? (it should and others like it too)"; 2026-10-13: "dont just stop
/// counting and switching at the goal"); none where `rep_beat` has none. Whether a count is heard is asked as it comes
/// (`Helper::counts_on`); whether Big view shows the count rising (the `pump` topic's `rep_count`) and the count step
/// starts from it (`pump_finish_move`) follows the player's switches alone (`Helper::counts_chosen`).
fn rep_count(m: &PlannedMove) -> Option<Beats> {
	Some(Beats::new(rep_beat(m)?, &rep_ends(m), facts().constants().max_reps))
}

/// A hold's goal on its running side, in seconds from its go (Panel.lua updateRunning: a hold's goal is the least,
/// never the most, so the small chime comes once a side as the stopwatch passes it and the clock counts on): none for a
/// timed side (`span`, its limit), a move that is no hold or has no goal, or one that chimed on this side already.
fn hold_goal(m: &PlannedMove, span: Option<f64>, chimed: bool) -> Option<i64> {
	(m.kind == kind::HOLD && span.is_none() && !chimed).then_some(m.target).flatten()
}

/// The game's sound for a count saved (Panel.lua `winSound`): a record's, or its small chime (`win`) for a count past
/// last time's or a placement; none for any other outcome.
fn win_sound(code: u8) -> Option<&'static str> {
	match code {
		outcome::BEAT => Some("record"),
		outcome::LASTBEAT | outcome::PLACEMENT => Some("win"),
		_ => None,
	}
}

/// The cadence a steady cardio move of cardio rung `rung` runs at (`Speech.lua` `Speech.PaceLine`): its rung's, up to
/// rung 6, whose line goes on from rung 7 (slice 52, 2026-10-23: the steady top is step 6's; rung 3's until then), which
/// the voice says at its go and its Ready and Running views word (`pump.pace.rung<n>`, slice 42's fix round).
pub(super) fn cadence_rung(rung: u8) -> u8 {
	rung.clamp(1, 6)
}

/// A Long Haul's cadence (slice 55, `Speech.lua` `HAUL_PACE`): rung 1's, "Quick and steady, too out of breath for full
/// sentences.", whatever the player's step.
pub(super) const HAUL_CADENCE: u8 = 1;

/// The stand-up's easy cadence, under rung 1's (`cadence.standup`, "Easy and steady, just keep moving."): a match
/// pump's warm cardio move's (`PlannedMove::warm`, slice 48's double-check: the first move after a long sit runs
/// easy), which the voice says at its go and its Ready and Running views word as `pump.pace.easy`.
pub(super) const EASY_CADENCE: u8 = 0;

/// An all-out window's line at its start (`Speech.lua` windowLine): the build's, or a counted set's all out at once
/// with its count.
fn window_line(m: &PlannedMove) -> &'static str {
	if m.counted { "cadence.count" } else { "cadence.allout" }
}

/// The count `pump.count` saves: its value, a loaded move's weight in whole tenths of a kg (none or 0: no weight) and a
/// counted set's answer to "All out and clean?" (`ns.Enum.answer`, 0 to 2; none: Yes, to the end).
fn typed_of(intent: &Intent) -> Result<Typed, String> {
	let value = intent.arg("value").and_then(Value::as_i64).ok_or("pump.count takes a value")?;
	let answer = match intent.arg("answer") {
		None | Some(Value::Null) => answer::YES,
		Some(a) => a
			.as_u64()
			.and_then(|a| u8::try_from(a).ok())
			.filter(|&a| a <= answer::NOT_ALL_OUT)
			.ok_or("pump.count's answer is 0, 1 or 2")?,
	};
	let load = match intent.arg("load") {
		None | Some(Value::Null) => None,
		Some(load) => Some(
			load.as_u64()
				.and_then(|l| u32::try_from(l).ok())
				.ok_or("pump.count's load is a weight in whole tenths of a kg")?,
		),
	};
	Ok(Typed { value, load: load.filter(|&l| l > 0), answer })
}

/// The row texts as rows; one that does not read is logged and left out.
fn rows(texts: &[String]) -> Vec<Row> {
	texts
		.iter()
		.filter_map(|t| {
			t.parse::<Row>().map_err(|e| log::warn!("pump: the engine wrote a row that does not read: {e}")).ok()
		})
		.collect()
}

impl Helper {
	/// The `pump` area's intents (the module's doc lists them).
	pub(crate) fn pump_intent(&mut self, intent: &Intent) -> Result<(), String> {
		let text = |name: &str| intent.arg(name).and_then(Value::as_str).map(str::to_owned);
		match intent.kind() {
			"pump.quick" | "pump.catchup" | "pump.start" | "pump.choose" | "pump.move" => {
				self.pump_start_intent(intent)
			}
			"pump.dismiss" => {
				match text("window").as_deref() {
					Some("choose") => self.pumps.choose = None,
					_ => return Err("pump.dismiss takes a window".to_owned()),
				}
				self.pump_publish();
				Ok(())
			}
			"pump.ready.start" => self.pump_go(),
			"pump.done" => self.pump_done(),
			"pump.back" => self.pump_back(),
			"pump.skip" => self.pump_skip(),
			"pump.bank" => self.pump_bank(),
			"pump.timed" => {
				let whole = intent.arg("whole").and_then(Value::as_bool).ok_or("pump.timed takes whole")?;
				self.pump_timed(whole)
			}
			"pump.count" => self.pump_count(typed_of(intent)?),
			"pump.mark" => {
				let answer = intent
					.arg("answer")
					.and_then(Value::as_u64)
					.and_then(|a| u8::try_from(a).ok())
					.filter(|a| (1..=4).contains(a))
					.ok_or("pump.mark takes an answer, 1 to 4")?;
				self.pump_mark(answer)
			}
			"pump.pace" => {
				let answer = intent
					.arg("answer")
					.and_then(Value::as_u64)
					.and_then(|a| u8::try_from(a).ok())
					.filter(|a| (1..=3).contains(a))
					.ok_or("pump.pace takes an answer, 1 to 3")?;
				self.pump_pace(answer)
			}
			"pump.keep" => self.pump_keep_going(),
			"pump.longer" => {
				let answer = intent
					.arg("answer")
					.and_then(Value::as_u64)
					.and_then(|a| u8::try_from(a).ok())
					.filter(|a| (1..=3).contains(a))
					.ok_or("pump.longer takes an answer, 1 to 3")?;
				self.pump_longer(answer)
			}
			"pump.count.clear" => self.pump_count_clear(),
			"pump.count.none" => self.pump_without_count(),
			"pump.count.couldnot" => self.pump_could_not(),
			"pump.swap" => {
				let take = match text("take").as_deref() {
					Some("another") => Take::Another,
					Some("easier") => Take::Easier,
					Some("backup") => Take::Backup,
					Some("back") => Take::Back,
					_ => return Err("pump.swap takes another, easier, backup or back".to_owned()),
				};
				self.pump_swap(take)
			}
			"pump.pain" => {
				let joints: Vec<u8> = intent
					.arg("joints")
					.and_then(Value::as_array)
					.ok_or("pump.pain takes joints")?
					.iter()
					.filter_map(|j| j.as_u64().and_then(|j| u8::try_from(j).ok()))
					.collect();
				self.pump_pain(joints)
			}
			"pump.unwell" => self.pump_stop(Stop::Unwell),
			"pump.pause" => self.pump_pause(&text("kind").ok_or("pump.pause takes a kind")?),
			"pump.end" => self.pump_end(),
			"pump.leave" => {
				let quit = text("then").as_deref() == Some("quit");
				self.pump_leave(quit)
			}
			"pump.stay" => {
				ui::update(&self.app, |v| v.shell.leave = None);
				Ok(())
			}
			"pump.fullscreen" | "pump.land" | "pump.saved.land" | "pump.window" => self.pump_page_intent(intent),
			"pump.saved.next" => self.pump_saved_next(),
			"pump.continue" => self.pump_continue(),
			"pump.guide" => self.pump_guide(),
			_ => Err(intent.unknown()),
		}
	}

	/// The intents about the page itself: Big view's toggle, a reveal or a pour at its end, the windows over it.
	fn pump_page_intent(&mut self, intent: &Intent) -> Result<(), String> {
		let flag = |name: &str| intent.arg(name).and_then(Value::as_bool);
		match intent.kind() {
			"pump.fullscreen" => self.pump_toggle_full(flag("on").ok_or("pump.fullscreen takes on")?),
			"pump.land" => {
				self.pump_land(flag("still").unwrap_or(false));
				Ok(())
			}
			"pump.saved.land" => self.pump_saved_land(flag("still") == Some(true)),
			"pump.window" => {
				self.pumps.window(flag("open").ok_or("pump.window takes open")?, lan::now_ms());
				Ok(())
			}
			_ => Err(intent.unknown()),
		}
	}

	/// The intents that start a pump or open the Start a pump window, `outside` when the tray or the card asked; one from
	/// a match offer that goes through answers the offer's card (`offer_started`).
	fn pump_start_intent(&mut self, intent: &Intent) -> Result<(), String> {
		let text = |name: &str| intent.arg(name).and_then(Value::as_str).map(str::to_owned);
		let offer = text("offer");
		let asked = self.pump_start_asked(intent, offer.clone());
		if asked.is_ok()
			&& let Some(offer) = &offer
		{
			self.offer_started(offer);
		}
		asked
	}

	/// The start an intent asks (`pump_start_intent`), from the match offer `offer` when it names one.
	fn pump_start_asked(&mut self, intent: &Intent, offer: Option<String>) -> Result<(), String> {
		let text = |name: &str| intent.arg(name).and_then(Value::as_str).map(str::to_owned);
		let outside = intent.arg("outside").and_then(Value::as_bool).unwrap_or(false);
		let size = intent
			.arg("size")
			.and_then(Value::as_u64)
			.and_then(|s| u8::try_from(s).ok())
			.filter(|s| (1..=3).contains(s))
			.or_else(|| offer.as_deref().and_then(|id| self.offer_tier(id)))
			.unwrap_or(NORMAL);
		match intent.kind() {
			// the Quick pump is the timed pump at the Menu's Quick pump length (2026-11-03)
			"pump.quick" => {
				let minutes = self.quick_length().ok_or("pump.quick: no lengths")?;
				self.pump_ask(Start::Expedition { minutes }, outside)
			}
			"pump.catchup" => self.pump_ask(Start::CatchUp, outside),
			"pump.move" => {
				let arg = |name: &str| intent.arg(name).cloned();
				let target: Target = arg("target")
					.and_then(|v| serde_json::from_value(v).ok())
					.ok_or("pump.move takes a target: {ladder, rung}, {move} or {check}")?;
				let from: StartedFrom = arg("from")
					.and_then(|v| serde_json::from_value(v).ok())
					.ok_or("pump.move takes from: {place, ladder?, step?, opens?}")?;
				self.pump_ask(Start::Move { target, from }, outside)
			}
			"pump.start" => {
				let kind = text("kind").ok_or("pump.start takes a kind")?;
				// the Start a pump window the tray or the card opened: its pump is theirs, which a recall ends untouched
				let outside = outside || self.pumps.choose.as_ref().is_some_and(|(_, out)| *out);
				self.pump_ask(Start::Free { kind, size, offer }, outside)
			}
			_ => self.pump_ask(Start::Choose(Choose { kind: text("kind"), size, offer }), outside),
		}
	}

	/// The size a pump from the match offer `offer` starts at when none was picked (every way to start one: the card,
	/// Pump home's offer card, the tray, the Games tab's Now card, Start a pump opened from it, "Open the pump when the
	/// game closes"): the standing offer's tier, by its match's minutes; none for the test's and for an offer that no
	/// longer stands (Balanced then: slice 57, the sitting clock sizes nothing).
	fn offer_tier(&self, offer: &str) -> Option<u8> {
		self.standing.current().filter(|o| o.id == offer).map(|o| o.tier)
	}

	/// The facts a plain kind's (`kind`) pump composes with at `at` (slices 48 and 57, docs/computer-games.md §5): from
	/// the match offer `offer` that opened it, its match's minutes and whether the player had sat `match_sat_min` as it
	/// was made (`Offer::warm`); with no match measured (the test's, one that no longer stands, a match picked by hand),
	/// the minutes sat now on the sitting clock without the offers (`seated_s`, which the offer's warm move reads too)
	/// and whether they reach `match_sat_min`; and the arms rest while a game marked `aim` with its switch on runs as it
	/// starts (`arms_now`, slice 48's double-check: never the offer's own, which a device reads away from the
	/// computer). None for every other pump, which stays Balanced.
	fn match_facts(&self, offer: Option<&str>, kind: Option<&str>, at: i64) -> Option<Match> {
		let arms = self.arms_now();
		if let Some(o) = offer.and_then(|id| self.standing.current().filter(|o| o.id == id)) {
			return Some(Match { minutes: o.minutes, warm: o.warm, arms });
		}
		(offer.is_some() || kind == Some(MATCH)).then(|| {
			let sat_s = self.seated_s(at);
			let warm = self.games.as_ref().is_some_and(|g| g.tunables().warm(sat_s));
			Match { minutes: sitting::minutes(sat_s), warm, arms }
		})
	}

	/// Whether every size of a match's pump takes the same moves now: Moves per match (the body's `match_pace`,
	/// `DEFAULT_PACE` while no device chose it) is a pace like Light, so the card and the Start a pump window for a
	/// match show no size to choose (slice 48's double-check).
	pub(crate) fn match_one_size(&self) -> bool {
		body::sizes_alike(self.body.body.match_pace.as_ref().map_or(DEFAULT_PACE, |p| p.value.as_str()))
	}

	/// Whether every size of the pump from the match offer `offer` takes the same moves: `match_one_size`, or the
	/// standing offer's match was a short one, which composes at Light (`Tunables::short`, 2026-11-04).
	pub(crate) fn offer_one_size(&self, offer: Option<&str>) -> bool {
		let short = |o: &Offer| self.games.as_ref().is_some_and(|g| g.tunables().short(o.minutes));
		self.match_one_size() || offer.and_then(|id| self.standing.current().filter(|o| o.id == id)).is_some_and(short)
	}

	/// The streak's board now, as Pump home draws it.
	fn board_now(&self) -> Board {
		let now = unix_s(lan::now_ms());
		self.board(&self.session.sheet(now), now)
	}

	/// A pump runs on this computer, from its Ready view until its end (`Helper::desk_pump_running`).
	pub(crate) fn pump_running(&self) -> bool {
		self.pumps.on()
	}

	/// The engine composes or doses a pump now (`Helper::desk_pump_starting`).
	pub(crate) fn pump_starting(&self) -> bool {
		self.pumps.starting
	}

	/// A move runs in the pump on this computer, or one is being made: a pump on screen that is not idle (`Run::idle`:
	/// a countdown, a running side, a count or an answer asked or being saved, an engine call under way) or a compose
	/// under way (`Helper::desk_pump_mid_move`).
	pub(crate) fn pump_mid_move(&self) -> bool {
		self.pumps.starting || self.pumps.run.as_ref().is_some_and(|r| !r.idle())
	}

	/// A game's pump opens in the place of the pump on screen (Rapha, 2026-10-28, after a game's close while a
	/// catch-up waited: "it should though, no? cause i just finished a game and the catch up was thus idle"), while that
	/// pump is idle (`Run::idle`): it ends as "Leave the pump?"'s Leave ends it (`pump_leave`, the window kept for the
	/// game's pump), what ran counting and the riders done paid (a catch-up's, a match pump's) with the rest left
	/// unfinished, a match's pump on its Ready view banking its own moves left first as a watched game's new run banks
	/// them (`pump_recalled`), and a Result leaves the screen. False, with nothing changed, while a move runs or a pump
	/// is being made (`pump_mid_move`), which the game's pump never interrupts (`Helper::desk_pump_make_way`).
	pub(crate) fn pump_make_way(&mut self) -> bool {
		if self.pump_mid_move() {
			return false;
		}
		let Some(run) = self.pumps.run.as_ref() else { return true };
		log::info!("pump {}: idle, it ends for a game's pump", run.id);
		if run.running() {
			if let Some((bank, _)) = (self.pump_bank_way(run) == Some(Bank::It)).then(|| run.bank()) {
				self.pump_write_bank(&bank);
			}
			if let Err(e) = self.pump_stop(Stop::Left) {
				log::warn!("pump: the idle pump did not end: {e}");
			}
		}
		self.pump_dismiss();
		true
	}

	/// Why no app pump starts now: felt unwell locks pumps, the game asks them (`wow_asks_pumps`), or the engine has not
	/// folded.
	pub(crate) fn pump_gate(&self) -> Option<Gate> {
		self.gate(self.wow_asks_pumps())
	}

	/// Why no catch-up starts now: as an app pump's, but a client whose folder holds QuestPump holds it whether or not
	/// its squares were read (`wow_with_addon`), so a payment never goes on a guess.
	fn catch_up_gate(&self) -> Option<Gate> {
		self.gate(self.wow_with_addon())
	}

	/// The gate of a start of this kind.
	fn start_gate(&self, start: &Start) -> Option<Gate> {
		if matches!(start, Start::CatchUp) { self.catch_up_gate() } else { self.pump_gate() }
	}

	fn gate(&self, game: bool) -> Option<Gate> {
		if self.session.locked_until(unix_s(lan::now_ms())).is_some() {
			Some(Gate::Lock)
		} else if game {
			Some(Gate::Game)
		} else if !self.engine_ready() {
			Some(Gate::Engine)
		} else {
			None
		}
	}

	/// Asks for a pump, or for the Start a pump window, from the page, the tray or the card (`outside`: QuestPump's
	/// window opens for it): refused while a gate holds or a pump runs;
	/// on a computer that never asked where the player pumps, the welcome's place window first, the start waiting for
	/// its answer.
	///
	/// # Errors
	/// Why no pump starts now.
	pub(crate) fn pump_ask(&mut self, start: Start, outside: bool) -> Result<(), String> {
		if let Some(gate) = self.start_gate(&start) {
			return Err(format!("no pump while {gate:?} holds")); // copy-lint: not shown (an intent's error)
		}
		if self.pumps.starting || self.pumps.on() {
			return Err("a pump is on screen already".to_owned());
		}
		if outside {
			let handle = self.app.clone();
			self.app
				.run_on_main_thread(move || windows::open_main(&handle, Landing::Here))
				.map_err(|e| e.to_string())?;
		}
		if !self.settings.place_asked {
			self.pumps.waiting = Some(Asked { start, outside });
			self.pump_publish();
			return Ok(());
		}
		self.pump_begin(start, outside);
		Ok(())
	}

	/// The place was answered (`Helper::place_answered`): the start that waited for it goes on, while no gate holds.
	pub(crate) fn pump_waited(&mut self) {
		match self.pumps.waiting.take() {
			Some(Asked { start, outside }) if self.start_gate(&start).is_none() => self.pump_begin(start, outside),
			Some(_) => self.pump_nowhere(),
			None => self.pump_publish(),
		}
	}

	/// A start went nowhere: the engine refused it as it read it again or composed no move, or a gate closed on it while
	/// it waited for the place. The `pump` topic counts it (`refused`), so the page whose Start it was takes its card or
	/// window back, and the Book is built again to say why (`Helper::book_refresh`).
	fn pump_nowhere(&mut self) {
		self.pumps.nowhere();
		self.book_refresh();
		self.pump_publish();
	}

	fn pump_begin(&mut self, start: Start, outside: bool) {
		// the place in use at this moment (a time of the places' schedule may have started or ended since the loop's
		// last look): the engine takes its profile before it composes
		self.profile_changed();
		match start {
			Start::Choose(choose) => {
				self.pumps.choose = Some((choose, outside));
				self.pump_publish();
			}
			Start::Quick => self.pump_compose(What::Quick, Compose::Quick, NORMAL, outside),
			Start::Free { kind, size, offer } => match words::family(&kind) {
				Some(family) => self.pump_compose(What::Free { kind }, Compose::Free { family, offer }, size, outside),
				None => log::warn!("pump: {kind} is no kind of contract/kinds.json"),
			},
			// a Light pump (A.pick's tier 1, whatever size is asked)
			Start::Move { target, from } => {
				self.pump_compose(What::Move { from, target, doses: None }, Compose::Move { target }, 1, outside);
			}
			Start::CatchUp => self.pump_catch_up(outside),
			Start::Expedition { minutes } => self.expedition_begin(minutes, outside),
		}
	}

	/// An app pump composed on the engine: the Quick pump, a plain kind's at a size (its family with the match offer
	/// that opened it, and a game's end's facts: `match_facts`), or a move started from a step's card or a move's window
	/// (`Engine::chosen`: its start read again now, a refusal composing nothing, `pump_nowhere`: counted on the topic and
	/// the Book built again to say why); its rows kept as the engine wrote them, then its Ready view. A match offer's
	/// pump composes on the whole unfinished queue (`Unfinished`, slice 61), so its own picks count the queue's moves as
	/// the game's quest pump does, and carries the ones this computer may pay now (the catch-up's gate open) as that
	/// pump carries them: the riders come back in its own slots in the game's order. On the computer only a plain kind's
	/// pump comes from an offer, so the Quick pump carries none.
	fn pump_compose(&mut self, what: What, compose: Compose, size: u8, outside: bool) {
		let id = new_id();
		let at = unix_s(lan::now_ms());
		let key = match &what {
			What::Free { kind, .. } => Some(kind.clone()),
			_ => None,
		};
		let (compose, offer) = match compose {
			Compose::Free { family, offer } => (Compose::Free { family, offer: None }, offer),
			other => (other, None),
		};
		let facts = matches!(compose, Compose::Free { .. })
			.then(|| self.match_facts(offer.as_deref(), key.as_deref(), at))
			.flatten();
		let arms = facts.as_ref().is_some_and(|f| f.arms).then(|| self.arms_game()).flatten();
		let riding =
			if offer.is_some() && self.catch_up_gate().is_none() { self.session.payable_kept() } else { Vec::new() };
		let unfinished = Unfinished {
			owed: riding.iter().map(ToString::to_string).collect(),
			queue: if offer.is_some() {
				self.session.store().owed().iter().map(ToString::to_string).collect()
			} else {
				Vec::new()
			},
		};
		let pump_id = id.clone();
		self.pumps.starting = true;
		self.pumps.composing = Some(Composing { id: id.clone(), offer: offer.clone(), outside });
		self.pumps.choose = None;
		self.engine_call(
			move |e| {
				let (composed, start) = match compose {
					Compose::Quick => (e.quick(&pump_id, at, Unfinished::default())?, None),
					Compose::Free { family, .. } => {
						let kind = key.as_deref().unwrap_or_default();
						(e.compose(&pump_id, kind, family, size, at, facts, unfinished)?, None)
					}
					Compose::Move { target } => {
						let picked = e.chosen(&pump_id, target, at)?;
						match picked.pump {
							Some(composed) => (composed, Some(picked.start)),
							None => return Ok(Made::Refused(picked.start.refused)),
						}
					}
				};
				if composed.pump.slots.is_empty() {
					return Ok(Made::Pump(Box::new((composed, None, start))));
				}
				Ok(Made::Pump(Box::new((composed, Some(Started::read(e, at)?), start))))
			},
			move |h, answer| {
				h.pumps.starting = false;
				// a recall while it composed took the mark (`pump_recalled`)
				let recalled = h.pumps.composing.take().is_none_or(|c| c.id != id);
				match answer {
					Ok(Made::Pump(made)) if recalled => {
						// the engine holds its rows already, as the pump at Ready a recall ends keeps them
						h.pump_keep(&id, what.kind(), &rows(&made.0.rows));
						h.pumps.nowhere();
						h.pump_publish();
					}
					Ok(Made::Pump(made)) => {
						let (composed, started, start) = *made;
						if let (Some(kind), Some(_)) = (start.as_ref().and_then(|s| s.kind.as_ref()), &started) {
							h.counts_tree_start(kind);
						}
						let what = match what {
							What::Move { from, target, .. } => {
								What::Move { from, target, doses: start.as_ref().map(Doses::of) }
							}
							other => other,
						};
						h.pump_composed(composed, started, Asking { what, id, offer, outside, arms, riding });
					}
					Ok(Made::Refused(why)) => {
						log::info!("pump {id}: the move's start is refused now ({why:?}), no pump");
						h.pump_nowhere();
					}
					Err(e) => {
						log::warn!("pump: the engine composed no pump: {e}");
						h.pump_nowhere();
					}
				}
			},
		);
	}

	/// The composed pump on screen: its rows kept (its offered row's time noted, which a bank's rows carry), the match
	/// offer that opened it taken, its riders' owed rows read (slice 61: `Run::ride`, paid under an id of their own),
	/// its Ready view.
	fn pump_composed(&mut self, mut composed: Composed, started: Option<Started>, asking: Asking) {
		let Asking { what, id, offer, outside, arms, riding } = asking;
		let written = rows(&composed.rows);
		self.pump_keep(&id, what.kind(), &written);
		let offered =
			written.iter().find(|r| kind::is_pump(r.kind) && r.outcome == outcome::OFFERED).map(|r| (r.ts, r.tier));
		let Some(Started { moves, previews, swaps, before, titles }) = started else {
			log::info!("pump {id}: the engine composed no move");
			self.pump_nowhere();
			return;
		};
		if let Some(offer) = &offer {
			self.offer_taken(offer);
		}
		// a pump of its own after an expedition's Result: the expedition is over and leaves the screen with it
		if !matches!(what, What::Expedition { .. }) {
			self.pumps.trip = None;
		}
		let rungs = composed.pump.slots.iter().map(|s| s.rung).collect();
		let slots = composed.pump.slots.into_iter().map(Some).collect();
		let riders = std::mem::take(&mut composed.pump.riders);
		log::info!("pump {id} ({what:?}): {} moves, {} riding", moves.len(), riders.len());
		let now = unix_s(lan::now_ms());
		let embers_before = self.embers_today(&self.session.sheet(now), now);
		let (build, cardio_max) = (facts().constants().allout_build_s, cardio_max());
		let riding = riding
			.iter()
			.enumerate()
			.map(|(row, r)| CatchMove::of(row, &OwedMove::of(&owed_now(r)), build, cardio_max))
			.collect();
		let mut run = Run {
			embers_before,
			previews,
			swaps: swaps.filter(view::any_option),
			before: view::Sum::read(&before),
			titles_before: titles,
			streak_before: Some(self.board_now()),
			offer,
			arms,
			offered,
			riding,
			riders_id: (!riders.is_empty()).then(new_id),
			..Run::new(id, what, outside, moves, slots, rungs)
		};
		run.ride(&riders);
		self.pumps.run = Some(run);
		// an expedition's next round waits under the breather: its first move is read as the breather ends
		if !self.expedition_resting() {
			self.pump_read_aloud();
		}
		self.pump_publish();
	}

	/// A catch-up: the pain flags asked of the engine at this instant (after every pain row written before it, so a
	/// joint flagged a moment ago on Pump home, in Your setup, on a phone or in the game rests its moves), then the
	/// catch-up over the owed moves they leave.
	fn pump_catch_up(&mut self, outside: bool) {
		let id = new_id();
		self.pumps.starting = true;
		self.pumps.composing = Some(Composing { id: id.clone(), offer: None, outside });
		self.pumps.choose = None;
		self.engine_call(Engine::pain_flags, move |h, answer| {
			h.pumps.starting = false;
			match answer {
				Ok(flags) => {
					h.pumps.flags = flags;
					// a recall while the flags were asked took the mark (`pump_recalled`)
					if h.pumps.composing.as_ref().is_some_and(|c| c.id == id) {
						h.pump_plan_catch_up(id, outside);
					} else {
						h.pump_publish();
					}
				}
				Err(e) => {
					log::warn!("pump: no catch-up without the pain flags: {e}");
					h.pumps.composing = None;
					h.pump_publish();
				}
			}
		});
		self.pump_publish();
	}

	/// The owed rows a catch-up on this computer pays now, in the helper's payment order, as the store keeps them
	/// (`Session::payable_kept`): those whose move as owed now (`owed_now`) no pain flag in `flags` rests (a move loading
	/// a flagged joint waits, as it does on the phones).
	pub(crate) fn pump_payable(&self, flags: &[u8]) -> Vec<Row> {
		self.session
			.payable_kept()
			.into_iter()
			.filter(|r| facts().move_joints(owed_now(r).move_id).iter().all(|j| !flags.contains(j)))
			.collect()
	}

	/// The catch-up over the owed moves the flags as the engine just said leave (`CatchUp.plan`, over each move as owed
	/// now), the whole list dosed on the engine as the game doses the owed moves riding a pump and the plan's rows taken
	/// from it (`Engine::catch_up`), then `pump_caught_up` (pump `id`), unless a recall took its mark meanwhile
	/// (`composing`).
	fn pump_plan_catch_up(&mut self, id: String, outside: bool) {
		let kept = self.pump_payable(&self.pumps.flags);
		let owed: Vec<OwedMove> = kept.iter().map(|r| OwedMove::of(&owed_now(r))).collect();
		if owed.is_empty() {
			log::info!("pump: no owed move this computer may pay");
			self.pumps.composing = None;
			self.pump_publish();
			return;
		}
		let plan = run::catch_up_plan(&owed, facts().constants().allout_build_s, cardio_max());
		let rows: Vec<Row> = plan.iter().filter_map(|m| kept.get(m.row).copied()).collect();
		let texts = kept.iter().map(ToString::to_string).collect();
		let pays = plan.iter().map(|m| m.row).collect();
		let at = unix_s(lan::now_ms());
		let pump_id = id.clone();
		self.pumps.starting = true;
		self.engine_call(
			move |e| e.catch_up(&pump_id, texts, pays, at),
			move |h, answer| {
				h.pumps.starting = false;
				// a recall while it was dosed took the mark (`pump_recalled`): it shows nowhere, nothing written
				if h.pumps.composing.take().is_none_or(|c| c.id != id) {
					h.pump_publish();
					return;
				}
				let dosed = answer
					.inspect_err(|e| {
						log::warn!(
							"pump {id}: the engine dosed none of the catch-up's moves, each runs by its time: {e}"
						);
					})
					.ok();
				h.pump_caught_up(id, outside, &rows, &plan, dosed);
			},
		);
		self.pump_publish();
	}

	/// The catch-up on screen: its moves in the plan's order (`rows`, their owed rows), a rep move, a hold or a counted
	/// set the engine dosed as the game doses it (`dosed`, by its place; none where `Engine::catch_up` failed, the
	/// engine then holding no pump of it: `Run::on_engine`), which asks its count, and every other move done by its
	/// time; each move timed, then the moves done paid at its end. One asked from the window whose window closed while
	/// it was made shows nowhere: nothing ran, its moves stay unfinished, and it never waits hidden on its Ready view
	/// (the computer held awake, the tray's pump on, every new Start refused). An app pump's rows are written as it
	/// lands (`pump_composed`: its offer taken), so it lands all the same.
	fn pump_caught_up(
		&mut self,
		id: String,
		outside: bool,
		rows: &[Row],
		plan: &[CatchMove],
		dosed: Option<Vec<Dosed>>,
	) {
		if !outside && self.app.get_webview_window(MAIN).is_none() {
			log::info!("pump {id} (catch-up): the window closed as it was made; it shows nowhere");
			self.pump_publish();
			return;
		}
		let on_engine = dosed.is_some();
		let mut dosed: BTreeMap<usize, Dosed> = dosed.into_iter().flatten().map(|d| (d.index, d)).collect();
		let mut slots = Vec::with_capacity(plan.len());
		let mut moves = Vec::with_capacity(plan.len());
		for (slot, m) in plan.iter().enumerate() {
			if let Some(Dosed { slot: dose, planned, .. }) = dosed.remove(&slot) {
				slots.push(Some(dose));
				moves.push(planned);
			} else {
				slots.push(None);
				moves.push(timed_move(slot, m));
			}
		}
		let rungs = rows.iter().map(|r| r.rung).collect();
		let asks = slots.iter().flatten().count();
		log::info!("pump {id} (catch-up): {} owed moves, {asks} asking their count", plan.len());
		let streak_before = Some(self.board_now());
		let now = unix_s(lan::now_ms());
		let embers_before = self.embers_today(&self.session.sheet(now), now);
		let owed = plan.iter().copied().enumerate().collect();
		self.pumps.run = Some(Run {
			owed,
			embers_before,
			on_engine,
			streak_before,
			..Run::new(id, What::CatchUp, outside, moves, slots, rungs)
		});
		self.pump_swaps();
		self.pump_read_aloud();
		self.pump_publish();
	}

	/// An engine call for the pump on screen begins: `busy` until it answers. The pump's id, none without a pump.
	fn pump_busy(&mut self) -> Option<String> {
		let run = self.pumps.run.as_mut()?;
		run.busy += 1;
		Some(run.id.clone())
	}

	/// An engine call for pump `id` answered: one call less under way; whether that pump is still on screen.
	fn pump_answered(&mut self, id: &str) -> bool {
		let Some(run) = self.pumps.run.as_mut().filter(|r| r.id == id) else { return false };
		run.busy = run.busy.saturating_sub(1);
		true
	}

	/// An engine read for the pump on screen, `busy` meanwhile: answered only while that pump is still on screen.
	fn pump_call<T, C, F>(&mut self, call: C, then: F)
	where
		T: Send + 'static,
		C: FnOnce(&Engine) -> Result<T, EngineError> + Send + 'static,
		F: FnOnce(&mut Helper, T) + Send + 'static,
	{
		let Some(id) = self.pump_busy() else { return };
		self.engine_call(call, move |h, answer| {
			if !h.pump_answered(&id) {
				return;
			}
			match answer {
				Ok(value) => then(h, value),
				Err(e) => {
					log::warn!("pump {id}: the engine: {e}");
					h.pump_publish();
				}
			}
		});
	}

	/// An engine write for the pump on screen (`finish`, `stop`), `busy` meanwhile: the rows it wrote are kept under the
	/// pump's id whatever is on screen when it answers (Continue, Leave or a new pump may come first; the engine holds
	/// them already), and while that pump is still on screen `after` reads them and the page hears it.
	fn pump_write_rows<C, A>(&mut self, call: C, after: A)
	where
		C: FnOnce(&Engine) -> Result<Finished, EngineError> + Send + 'static,
		A: FnOnce(&mut Run, &[Row], &Finished) + Send + 'static,
	{
		let Some(id) = self.pump_busy() else { return };
		self.engine_call(call, move |h, answer| {
			let shown = h.pump_answered(&id);
			match answer {
				Ok(finished) => {
					let written = rows(&finished.rows);
					h.pump_keep(&id, None, &written);
					if let Some(run) = h.pumps.run.as_mut().filter(|_| shown) {
						after(run, &written, &finished);
					}
				}
				Err(e) => log::warn!("pump {id}: the engine wrote nothing: {e}"),
			}
			if shown {
				h.pump_publish();
			}
		});
	}

	/// Rows the engine wrote for the pump `id`, kept as the computer's app rows (stamped as written; its offered row with
	/// what the pump was where that names it, `what`, the Saga's title); the session's effects bring the flags, the owed
	/// moves and the lock up to date (`Helper::apply`).
	fn pump_keep(&mut self, id: &str, what: Option<PumpKind>, rows: &[Row]) {
		if rows.is_empty() {
			return;
		}
		let (kept, fx) = self.session.keep_computer(id, what, rows, &runtime::now());
		if kept < rows.len() {
			log::warn!("pump {id}: kept {kept} of {} rows", rows.len());
		}
		self.apply(fx);
	}

	/// The current move's Ready view: its goal set again (`pump_regoal`), its Swap options asked (`pump_swaps`), a
	/// pose's day note comes (`pump_ready_days`), and the move's name is read once.
	fn pump_ready(&mut self) {
		let Some(run) = self.pumps.run.as_mut() else { return };
		run.stage = Stage::Ready;
		// a move's save (its pour, slice 65) never reaches past its own move
		run.pouring = None;
		run.pour = None;
		self.pump_regoal();
		self.pump_swaps();
		self.pump_ready_days();
		self.pump_read_aloud();
	}

	/// The shown move's goal set again as its Ready view opens (`Engine::ready`, Engine.lua `ns.ReadyGoal`, the game's
	/// `Lc.Next`; 2026-10-28): a rep or hold move with a count to reach that the engine dosed, asked before its Swap
	/// options (the engine answers in order) with the Start held meanwhile (`busy`); its slot and the move as it now
	/// stands take the shown one's place (`pump_redosed`): a lower goal where a set earlier in the pump made it a repeat
	/// of its ladder, and the sets that ladder took today, which its goal's word names ("Goal · set N today").
	fn pump_regoal(&mut self) {
		let Some(run) = self.pumps.run.as_ref() else { return };
		let asks = |m: &&PlannedMove| {
			(run.what.app() || run.dosed(m)) && m.target.is_some() && (m.kind == kind::REP || m.kind == kind::HOLD)
		};
		let Some(slot) = run.shown().filter(asks).map(|m| m.slot) else { return };
		let at = unix_s(lan::now_ms());
		self.pump_call(move |e| e.ready(slot, at), move |h, dosed| h.pump_redosed(slot, dosed));
	}

	/// The move at slot `index` as the engine dosed it again on its Ready view (`dosed`; none where the pump lacks it):
	/// its slot and the move in place of the ones shown while that move still waits there, its preview as planned.
	fn pump_redosed(&mut self, index: usize, dosed: Option<Dosed>) {
		let Some(Dosed { slot, planned, .. }) = dosed else { return };
		let Some(run) = self.pumps.run.as_mut().filter(|r| r.stage == Stage::Ready) else { return };
		let Some(pos) = run.moves.iter().position(|m| m.slot == index && m.r#move == planned.r#move) else { return };
		if let Some(s) = run.slots.get_mut(index) {
			*s = Some(slot);
		}
		run.moves[pos] = planned;
		self.pump_publish();
	}

	/// Swap's options for the move on Ready (the game's `Lc.SwapOptions`, on every move's Ready view): an app pump's own
	/// move's and a rider's of every kind, an owed move's as the game offers them (a catch-up's that asks its count since
	/// 2026-10-26; since 2026-11-02 a pose's, a timed move's and an uncounted window's too, David: "Why is there still no
	/// alternative exercise to switch to like in wow addon"), none where the engine placed no slot, never asked of a
	/// catch-up the engine holds no pump of (`Run::on_engine`), asked beside the Start and never in its way; the swap row
	/// under the goal shows once they come, and none without an option. Asked again after each swap, when a swapped move
	/// offers its way back alone.
	fn pump_swaps(&mut self) {
		let Some(run) = self.pumps.run.as_mut() else { return };
		run.swaps = None;
		let Some(slot) = run.shown().filter(|_| run.on_engine).map(|m| m.slot) else { return };
		let (at, id) = (unix_s(lan::now_ms()), run.id.clone());
		self.engine_call(
			move |e| e.swap_options(slot, at),
			move |h, answer| match answer {
				Ok(swaps) => {
					let ready = |r: &&mut Run| {
						r.id == id && r.stage == Stage::Ready && r.shown().is_some_and(|m| m.slot == slot)
					};
					if let Some(run) = h.pumps.run.as_mut().filter(ready) {
						run.swaps = swaps.filter(view::any_option);
						h.pump_publish();
					}
				}
				Err(e) => log::warn!("pump: Swap's options: {e}"),
			},
		);
	}

	/// What the shown pose's answer counts toward, read for its Ready view's day note (slice 47, Panel.lua
	/// `poseDaysLine`: any goal pose of the step counts the family's day, with the days so far and what they still
	/// lack): asked of an app pump's own pose that names its family's mark and asks how close it got (the game's own
	/// condition; a rider runs by its time, as a catch-up's pose), beside the Start and never in its way, from the state
	/// now (the pump's earlier sets in), and kept while that move waits on its Ready view, Back included; a move that
	/// asks nothing, or one a Swap or the Ready view's Pain brings in its place, reads its own or none.
	fn pump_ready_days(&mut self) {
		let Some(run) = self.pumps.run.as_mut() else { return };
		let asks = run.shown().filter(|m| !run.rider(m.slot) && m.mark && m.ask.is_some()).map(|m| (m.slot, m.r#move));
		let Some((slot, mv)) = asks else {
			run.ready_toward = None;
			return;
		};
		if run.ready_toward.is_some_and(|(s, m, _)| (s, m) == (slot, mv)) {
			return;
		}
		run.ready_toward = None;
		let id = run.id.clone();
		self.engine_call(
			move |e| e.toward(slot),
			move |h, answer| {
				let toward = match answer {
					Ok(toward) => toward,
					Err(e) => return log::warn!("pump {id}: what the pose's Ready view counts: {e}"),
				};
				let ready = |r: &&mut Run| {
					r.id == id && r.stage == Stage::Ready && r.shown().is_some_and(|m| (m.slot, m.r#move) == (slot, mv))
				};
				if let (Some(toward), Some(run)) = (toward, h.pumps.run.as_mut().filter(ready)) {
					run.ready_toward = Some((slot, mv, toward));
					h.pump_publish();
				}
			},
		);
	}

	/// The move on Ready said once (`Speech.lua`: a swapped move is read again; one back on Ready is not), its callout
	/// after it.
	fn pump_read_aloud(&mut self) {
		let Some(run) = self.pumps.run.as_mut() else { return };
		let Some(m) = run.moves.get(run.current) else { return };
		let id = (run.current, m.r#move);
		if run.spoken == Some(id) {
			return;
		}
		run.spoken = Some(id);
		run.callout = None;
		if let Some(text) = facts().text(m.r#move) {
			let line = format!("move.{}", text.key); // copy-lint: not shown (a voice take's id)
			let length = Self::line_seconds(&line);
			self.pump_say(line);
			if let Some(length) = length {
				self.pump_callout(length);
			}
		}
	}

	/// The callout of the move just read aloud (Speech.lua's; the engine's Stats.lua `ns.PickCallout`, the streak from
	/// the board the pump began with): asked beside the Start, never in its way, and due `callout_gap_s` after the
	/// move's line (`length` seconds) ends, one take of its family other than the one said last; said only while that
	/// move still waits on its Ready view (`pump_due`). With the Callouts switch off nothing is asked.
	fn pump_callout(&mut self, length: f64) {
		if !self.callouts_on() {
			return;
		}
		let Some(run) = self.pumps.run.as_ref() else { return };
		let (pump, index, spoken) = (run.id.clone(), run.current, run.spoken);
		// a catch-up's move reads the slot the engine dosed it as (`place`, as the game's callout reads its rider's), a
		// move it did not dose none: only the pump's own facts speak then (the count names them)
		let count = (!run.what.app()).then_some(run.moves.len());
		let place = run.shown().filter(|m| count.is_some() && run.dosed(m)).map(|m| m.slot);
		let (before, after) =
			run.streak_before.as_ref().map_or((0, None), |b| (b.days, (!b.today).then_some(b.after_pump)));
		let said = lan::now_ms();
		let at = unix_s(said);
		self.engine_call(
			move |e| e.callout(index, count, place, at, before, after),
			move |h, answer| {
				let family = match answer {
					Ok(Some(family)) => family,
					Ok(None) => return,
					Err(e) => return log::warn!("pump: the callout: {e}"),
				};
				let Some(take) = h.callout_take(&family) else { return };
				let ready = |r: &&mut Run| r.id == pump && r.stage == Stage::Ready && r.spoken == spoken;
				// a guide begun meanwhile leaves the callout unsaid (`guide_begin`)
				if h.pumps.guide.is_none()
					&& let Some(run) = h.pumps.run.as_mut().filter(ready)
				{
					let gap = facts().constants().callout_gap_s;
					run.callout = Some((said + run::millis(length + gap), family, take));
				}
			},
		);
	}

	/// A take of a callout family (`record`, `day30`) that has a recording, other than the one said last where it has
	/// another (`callout_takes`, by the fact the family's letters name); none for a family with no take.
	fn callout_take(&self, family: &str) -> Option<String> {
		let fact = family.trim_end_matches(|c: char| c.is_ascii_digit());
		let n = facts().constants().callout_takes.get(fact).copied().unwrap_or(0);
		let takes: Vec<String> = (1..=n)
			.map(|k| format!("callout.{family}.t{k}")) // copy-lint: not shown (a voice take's id)
			.filter(|id| Self::line_seconds(id).is_some())
			.collect();
		let last = self.pumps.takes.get(family);
		let fresh: Vec<&String> = takes.iter().filter(|id| Some(*id) != last).collect();
		let pool = if fresh.is_empty() { takes.iter().collect() } else { fresh };
		(!pool.is_empty()).then(|| pool[rand::random_range(0..pool.len())].clone())
	}

	/// An all-out window's shout at `left` seconds left (slice 53, `Speech.lua` shout: one of `shout_at_s`'s): a fresh
	/// take of its family (`shout<left>`, never the one played last where it has more), kept as the family's last; none
	/// at any other second or with no take recorded, where the mark stays as it was.
	fn shout_take(&mut self, left: i64) -> Option<String> {
		if !facts().constants().shout_at_s.contains(&left) {
			return None;
		}
		let family = format!("shout{left}"); // copy-lint: not shown (a callout family's id)
		let take = self.callout_take(&family)?;
		self.pumps.takes.insert(family, take.clone());
		Some(take)
	}

	/// A voice line (never a count), whose moment the marks keep their distance from.
	fn pump_say(&mut self, line: String) {
		self.pumps.line_at = lan::now_ms();
		self.cue(Cue::Line(line));
	}

	/// Start on Ready: the countdown to the current move, from its first side; Big view's full screen at the pump's
	/// first countdown where the player keeps it.
	fn pump_go(&mut self) -> Result<(), String> {
		if self.pump_gate() == Some(Gate::Lock) {
			return Err("pumps rest".to_owned());
		}
		let fullscreen = self.settings.fullscreen;
		let now = lan::now_ms();
		let run = self.pumps.run.as_mut().ok_or("no pump")?;
		if run.stage != Stage::Ready || run.busy > 0 {
			return Err("no Start now".to_owned());
		}
		// no callout on the countdown: one still waiting stays unsaid, one still speaking stops
		run.callout = None;
		if run.callout_ends.take().is_some_and(|ends| ends > now) {
			self.hush();
		}
		let run = self.pumps.run.as_mut().ok_or("no pump")?;
		let (kind, seconds, side) = run.shown().map(|m| (m.kind, m.seconds, m.side)).ok_or("no move")?;
		run.side = Some(MoveRun::new(kind, seconds, side));
		run.before_start = Some((run.pump_start, run.riders_start, run.last_end));
		run.move_start = None;
		run.cues = Cues::default();
		run.stage = Stage::Countdown { go: run::go_after(now), digit: None };
		self.pumps.looked = now;
		let first = run.pump_start.is_none() && !run.full;
		if first && fullscreen {
			run.full = true;
			self.pump_full_screen(true);
		}
		self.pump_publish();
		Ok(())
	}

	/// Big view's own full-screen toggle: the window's full screen as `view.fullscreen` sets it (the choice kept for the
	/// next pump), and the pump on screen leaves it at its end when it was put on here.
	fn pump_toggle_full(&mut self, on: bool) -> Result<(), String> {
		self.view_intent(&Intent::from_value(serde_json::json!({"type": "view.fullscreen", "on": on}))?)?;
		if let Some(run) = self.pumps.run.as_mut().filter(|r| r.running()) {
			run.full = on;
		}
		Ok(())
	}

	/// The window's own full screen on (Big view, as the player keeps it) or off (the pump's end).
	fn pump_full_screen(&self, on: bool) {
		if let Some(window) = self.app.get_webview_window(MAIN)
			&& window.is_fullscreen().is_ok_and(|now| now != on)
			&& let Err(e) = window.set_fullscreen(on)
		{
			log::warn!("pump: full screen: {e}");
		}
	}

	/// When the loop next wakes for the pump (ms): its stage's instant (`stage_wake`), the guide's next take or an
	/// expedition's rest's end; none while nothing waits.
	pub(crate) fn pump_wake(&self) -> Option<i64> {
		[self.stage_wake(), self.guide_wake(), self.pumps.trip.as_ref().and_then(expedition::Trip::rest_until)]
			.into_iter()
			.flatten()
			.min()
	}

	/// The stage's next instant (ms): a Ready view's callout, the countdown's next digit or its go, a running side's
	/// next mark, tick of its last five seconds, hold's goal, rep count or end, the Result's next sound.
	fn stage_wake(&self) -> Option<i64> {
		let run = self.pumps.run.as_ref()?;
		let now = lan::now_ms();
		// a countdown and a timed move are looked at every beat at least, so a sleep shows as a late wake (`pump_slept`)
		let beat = self.pumps.looked + BEAT_MS;
		match run.stage {
			Stage::Ready => run.callout.as_ref().map(|(at, ..)| *at),
			Stage::Countdown { go, .. } => Some(run::next_digit(go, now).min(beat)),
			Stage::Running { start } => {
				let m = run.shown()?;
				let at = |s: f64| start + run::millis(s);
				let span = run.side.as_ref().and_then(MoveRun::limit);
				let limit = span.map(at);
				let mark = run.cues.next.map(|(s, _)| at(s));
				let five =
					span.and_then(|l| run::last_five(l, run::secs(now - start), run.cues.five)).map(|(s, _)| at(s));
				let goal = hold_goal(m, span, run.cues.goal).map(|t| start + t * 1000);
				let rep = run.cues.rep.map(|due| start + due.at);
				let keep = keep_from(run, m).map(at);
				let look = limit.map(|_| beat);
				[limit, mark, five, goal, rep, keep, look].into_iter().flatten().min()
			}
			Stage::Saved => run.pour.as_ref().and_then(view::pour::Pour::wake),
			Stage::Done => run.reveal.as_ref().and_then(view::Reveal::next_sound),
			_ => None,
		}
	}

	/// The loop woke for the pump: whatever is due now happens.
	pub(crate) fn pump_due(&mut self) {
		let now = lan::now_ms();
		if self.pump_slept(now) {
			return;
		}
		self.guide_due(now);
		if self.expedition_rest_due(now) {
			return;
		}
		let Some(run) = self.pumps.run.as_mut() else { return };
		match run.stage {
			Stage::Ready => {
				// the callout once due, never said late (a sleep, a stall), as a mark
				let Some((at, family, take)) = run.callout.take_if(|(at, ..)| now >= *at) else { return };
				if run::secs(now - at) >= MARK_LATE_S {
					return;
				}
				let ends = now + run::millis(Self::line_seconds(&take).unwrap_or(0.0));
				if let Some(run) = self.pumps.run.as_mut() {
					run.callout_ends = Some(ends);
				}
				self.pumps.takes.insert(family, take.clone());
				self.pump_say(take);
			}
			Stage::Countdown { go, digit } => {
				if now >= go {
					self.pump_reach_go(go);
				} else if let Some(d) = run::digit(go, now).filter(|&d| Some(d) != digit) {
					run.stage = Stage::Countdown { go, digit: Some(d) };
					self.cue(Cue::Event("tick"));
					self.pump_publish();
				}
			}
			Stage::Running { start } => self.pump_running_due(start, now),
			Stage::Saved => {
				let Some(pour) = run.pour.as_mut() else { return };
				let due = pour.take_due(now);
				let lands = pour.lands_by(now);
				let goes_on = pour.goes_on_at().is_some_and(|at| now >= at);
				for event in due {
					self.cue(Cue::Event(event));
				}
				if lands {
					self.pump_pour_land(false, true);
				}
				if goes_on {
					// the move goes on by itself once its rows are in, as Next would (`Pour::goes_on_at`)
					let _ = self.pump_saved_next();
				}
			}
			Stage::Done => {
				let due = run.reveal.as_mut().map(|r| r.take_due(now)).unwrap_or_default();
				for event in due {
					self.cue(Cue::Event(event));
				}
			}
			_ => {}
		}
	}

	/// What is due on the running side `start`ed at: its marks, a set of reps' count, and the side's end at its limit.
	fn pump_running_due(&mut self, start: i64, now: i64) {
		let elapsed = run::secs(now - start);
		let c = facts().constants();
		let gap = c.mark_gap_s * 1000;
		let marks = Marks { step: run::real(c.mark_s), to: run::real(c.mark_to), voice: run::real(c.mark_voice_s) };
		let Some(run) = self.pumps.run.as_mut() else { return };
		let Some(m) = run.moves.get(run.current).cloned() else { return };
		let limit = run.side.as_ref().and_then(MoveRun::limit);
		let mut cues = Vec::new();
		while let Some((at, mark)) = run.cues.next.filter(|&(at, _)| at <= elapsed) {
			if elapsed - at < MARK_LATE_S {
				cues.push((at, mark));
			}
			run.cues.last = Some(at);
			run.cues.next = marks.next(m.kind, limit, at, Some(at), m.counted);
		}
		// a rep's count once due, one overdue skipped (a sleep, a stall), the count going on at the next beat past now
		let mut rep = None;
		if let (Some(beats), Some(due)) = (&run.cues.beats, run.cues.rep.filter(|due| due.at <= now - start)) {
			let late = now - start - due.at;
			run.cues.rep = beats.next(now - start, Some(due));
			let fresh = late < ((beats.at(due.n + 1) - due.at) / 2).min(run::millis(MARK_LATE_S));
			rep = fresh.then(|| format!("count.{}", due.n)); // copy-lint: not shown (a voice take's id)
		}
		// a timed side's last five seconds, one tick each (one the loop woke late for still comes, within its second)
		let five = limit.and_then(|l| run::last_five(l, elapsed, run.cues.five)).filter(|&(at, _)| at <= elapsed);
		if let Some((_, left)) = five {
			run.cues.five = Some(left);
		}
		let goal = hold_goal(&m, limit, run.cues.goal).is_some_and(|t| run::real(t) <= elapsed);
		run.cues.goal |= goal;
		// Keep going shows from the steady move's last keep_going_s (slice 52)
		let keep = keep_from(run, &m).is_some_and(|from| from <= elapsed);
		run.cues.keep |= keep;
		let line_at = self.pumps.line_at;
		for (at, mark) in cues {
			// an all-out window's shout in its mark's place (slice 53), a mark with no take as it was
			let left = limit.map(|l| run::whole((l - at).round()));
			if let Some(take) = left.filter(|_| m.allout).and_then(|left| self.shout_take(left)) {
				self.cue(Cue::Line(take));
				continue;
			}
			match mark {
				// copy-lint: not shown (a voice take's id)
				Mark::Say(n) if now - line_at >= gap => self.cue(Cue::Line(format!("count.{n}"))),
				_ => self.cue(Cue::Event("tick")),
			}
		}
		if keep {
			self.pump_publish();
		}
		if five.is_some() {
			self.cue(Cue::Event("lastFive"));
		}
		if goal {
			self.cue(Cue::Event("win"));
		}
		// outside the marks' gap, and never to a screen reader (`counts_on`): skipped, the next keeps its number
		if let Some(line) = rep.filter(|_| self.counts_on()) {
			self.cue(Cue::Line(line));
		}
		if let Some(limit) = limit.filter(|&l| l <= elapsed) {
			self.pump_end_side(start, limit, start + run::millis(limit));
			// the timed end's sound where the move waits for its count; a move that ended done sounded `done` for it
			// (one sound at the end, as the phones play one), and a side move's first side says "Switch sides."
			if self.pumps.run.as_ref().is_some_and(|r| r.stage == Stage::Counting) {
				self.cue(Cue::Event("timeUp"));
			}
		}
	}

	/// A side's go: its clock, its marks, a set of reps' count (`rep_count`, from 1 on each side of a side move) and,
	/// on a cardio move, its cadence (an all-out window's line instead, said at its go).
	fn pump_reach_go(&mut self, go: i64) {
		let c = facts().constants();
		let marks = Marks { step: run::real(c.mark_s), to: run::real(c.mark_to), voice: run::real(c.mark_voice_s) };
		let Some(run) = self.pumps.run.as_mut() else { return };
		let Some(m) = run.moves.get(run.current).cloned() else { return };
		run.stage = Stage::Running { start: go };
		let limit = run.side.as_ref().and_then(MoveRun::limit);
		let beats = rep_count(&m);
		let next = marks.next(m.kind, limit, 0.0, None, m.counted);
		let rep = beats.as_ref().and_then(|b| b.next(0, None));
		run.cues = Cues { next, beats, rep, ..Cues::default() };
		if run.move_start.is_none() {
			run.move_start = Some(go);
			run.pump_start.get_or_insert(go);
			if run.rider(m.slot) {
				run.riders_start.get_or_insert(go);
			}
		}
		let rung = run.rungs.get(run.current).copied().unwrap_or(1);
		self.cue(Cue::Event("go"));
		if m.kind == kind::CARDIO {
			// the rung's cadence (`Speech.PaceLine`), a match pump's warm move the stand-up's easy one, a Long Haul rung
			// 1's; an all-out window opens with its line
			let line = if m.allout {
				window_line(&m).to_owned()
			} else if m.warm {
				"cadence.standup".to_owned() // copy-lint: not shown (a voice take's id)
			} else if m.haul {
				format!("cadence.rung{HAUL_CADENCE}") // copy-lint: not shown (a voice take's id)
			} else {
				format!("cadence.rung{}", cadence_rung(rung)) // copy-lint: not shown (a voice take's id)
			};
			self.pump_say(line);
		}
		self.pump_publish();
	}

	/// Done on a running move: its side ends now (a side move's first side switches).
	fn pump_done(&mut self) -> Result<(), String> {
		let now = lan::now_ms();
		if self.pump_slept(now) {
			return Ok(());
		}
		let run = self.pumps.run.as_ref().ok_or("no pump")?;
		let Stage::Running { start } = run.stage else { return Err("no move runs".to_owned()) };
		self.pump_end_side(start, run::secs(now - start), now);
		Ok(())
	}

	/// Keep going (slice 52, Events.lua `Lc.KeepGoing`; its button or Space while it shows): the clock runs on past the
	/// plan to the full move (`PlannedMove::keep`, 45 s) and stops there by itself, its marks and last five seconds on
	/// the new end; nothing asks after it (`pump_finish_move`).
	fn pump_keep_going(&mut self) -> Result<(), String> {
		let now = lan::now_ms();
		if self.pump_slept(now) {
			return Ok(());
		}
		let c = facts().constants();
		let marks = Marks { step: run::real(c.mark_s), to: run::real(c.mark_to), voice: run::real(c.mark_voice_s) };
		let run = self.pumps.run.as_mut().ok_or("no pump")?;
		let Stage::Running { start } = run.stage else { return Err("no move runs".to_owned()) };
		let m = run.shown().cloned().ok_or("no move")?;
		let to = m.keep.filter(|_| run.cues.keep).ok_or("no Keep going now")?;
		let side = run.side.as_mut().ok_or("no move runs")?;
		if !side.keep(to) {
			return Err("no Keep going now".to_owned());
		}
		let limit = side.limit();
		run.cues.keep = false;
		run.cues.five = None;
		run.cues.next = marks.next(m.kind, limit, run::secs(now - start), run.cues.last, m.counted);
		log::info!("pump {}: move {}, keep going to {to} s", run.id, m.r#move);
		self.pump_publish();
		Ok(())
	}

	/// The computer slept, or its clock jumped forward, during a countdown or a running timed move: the loop's wake (or
	/// the player's first press after it) comes more than `LATE_WAKE_MS` after the instant the pump asked for. The move
	/// goes back to its Ready view as Back takes it, nothing written, so no slept time counts. True when it went back;
	/// else this is the clock's latest look.
	fn pump_slept(&mut self, now: i64) -> bool {
		let Some(run) = self.pumps.run.as_ref() else { return false };
		let timed = match run.stage {
			Stage::Countdown { .. } => true,
			Stage::Running { .. } => run.side.as_ref().and_then(MoveRun::limit).is_some(),
			_ => false,
		};
		let Some(late) = self.stage_wake().map(|at| now - at).filter(|&late| timed && late > LATE_WAKE_MS) else {
			self.pumps.looked = now;
			return false;
		};
		log::info!(
			"pump {}: the loop woke {late} ms late (a sleep, or the clock moved): the move is on its Ready view",
			run.id
		);
		self.pump_back().is_ok()
	}

	/// A side ended, by Done or at its limit: a side move's first side switches, counted in again from now; any other
	/// side ends the move, a timed move or a hold Done under `min_seconds` banked without a word (`pump_short`) but a
	/// hold's first max set, which goes on to its count (`Run::first_max_set`, the game's `Lc.Done`), a timed one Done
	/// under the ask's share of its seconds asking first whether it was done whole.
	fn pump_end_side(&mut self, start: i64, elapsed: f64, end: i64) {
		let c = facts().constants();
		let Some(run) = self.pumps.run.as_mut() else { return };
		// the move's kind and planned seconds: a move of reps is judged by its count, a timed move that ran its time
		// never short
		let planned = run.shown().map(|m| (m.kind, m.seconds, run.first_max_set(m)));
		let short = |seconds: i64| {
			planned.is_some_and(|(k, plan, first)| {
				k != kind::REP && !first && seconds < c.min_seconds && plan.is_none_or(|p| seconds < p)
			})
		};
		let Some(side) = run.side.as_mut() else { return };
		match side.end(elapsed, c.timed_ask_share) {
			End::Done { seconds, .. } if short(seconds) => self.pump_short(),
			End::SwitchSides => {
				run.stage = Stage::Countdown { go: run::go_after(lan::now_ms()), digit: None };
				run.cues = Cues::default();
				self.pump_say("side.switch".to_owned()); // copy-lint: not shown (a voice take's id)
				self.pump_publish();
			}
			End::Done { seconds, full: Some(full) } => {
				run.asked = Some((start, elapsed, end));
				run.stage = Stage::Asking { seconds, full };
				self.pump_publish();
			}
			End::Done { .. } => self.pump_finish_move(elapsed, end, None, true),
		}
	}

	/// "Did you do the whole move?": Yes saves it done in its whole seconds (a counted set then asks its count), No
	/// stopped at the clock's.
	fn pump_timed(&mut self, whole: bool) -> Result<(), String> {
		let run = self.pumps.run.as_ref().ok_or("no pump")?;
		let (Stage::Asking { full, .. }, Some((_, elapsed, end))) = (run.stage, run.asked) else {
			return Err("nothing asked".to_owned());
		};
		self.pump_finish_move(elapsed, end, whole.then_some(full), true);
		Ok(())
	}

	/// The move ended `elapsed` seconds into its running side, at `end`: it counts what its run says (or `full`, a
	/// timed move's whole seconds the player said were done). `advance`: the pump goes on, to the count of a rep move, a
	/// hold or a counted set that ran its time where the engine dosed it (a catch-up's since 2026-10-12, a match pump's
	/// rider since slice 61), to a steady move's pace ask where its slot asks it (a rider's too since slice 42's fix
	/// round, 2026-10-17), else a move of an app pump's own to its row and a rider to the next move.
	fn pump_finish_move(&mut self, elapsed: f64, end: i64, full: Option<i64>, advance: bool) {
		let chosen = self.counts_chosen();
		let Some(run) = self.pumps.run.as_mut() else { return };
		let Some(m) = run.moves.get(run.current).cloned() else { return };
		let seconds =
			full.unwrap_or_else(|| run.side.as_ref().map_or_else(|| run::whole(elapsed), |s| s.value(elapsed)));
		// the reps the voice's count reached by now where the player's switches let it count (`rep_count`), on the
		// run's time to the ms (a side move's weaker side's: the count never falls as time runs)
		let counted = run
			.side
			.as_ref()
			.zip(rep_count(&m).filter(|_| chosen))
			.map(|(side, beats)| beats.ended(side.ran_ms(elapsed)));
		let kept = run.kept_to();
		run.move_start = None;
		run.side = None;
		run.asked = None;
		run.ran.insert(m.slot, seconds);
		run.last_end = Some(end);
		if !advance {
			return;
		}
		if let Some(to) = kept
			&& self.pump_kept(&m, seconds, to, true)
		{
			return;
		}
		let Some(run) = self.pumps.run.as_mut() else { return };
		let counts = m.kind == kind::REP || m.kind == kind::HOLD || (m.counted && seconds >= m.seconds.unwrap_or(0));
		let paces = m.pace && seconds >= m.seconds.unwrap_or(0);
		// a rider (a catch-up's move, a match pump's rider) asks its count or its pace only where the engine dosed it,
		// else it is done by its time (a pose too: no "How close to the goal pose?")
		if run.rider(m.slot) && !((counts || paces) && run.dosed(&m)) {
			return self.pump_advance(true);
		}
		// a pose whose move asks, run its whole time, asks how close the player got before its row (2026-10-09)
		if m.kind == kind::FLOW && m.ask.is_some() && seconds >= m.seconds.unwrap_or(0) {
			run.stage = Stage::Marking { seconds };
			self.pump_publish();
			return self.pump_checks(m.slot);
		}
		// a steady move whose slot asks its pace, run its whole time (or "Yes, all of it"), asks whether the player kept
		// it before its row (slice 42, 2026-10-17), the voice asking with it; a catch-up's where the engine dosed it so
		if paces {
			run.stage = Stage::Pacing { seconds };
			self.pump_say("ask.pace".to_owned()); // copy-lint: not shown (a voice take's id)
			return self.pump_publish();
		}
		if !counts {
			return self.pump_record(timed_result(&m, seconds), false);
		}
		run.count.clear();
		run.stage = Stage::Counting;
		// a hold's count starts at its stopwatch's seconds, the player's to change since the clock stops only once
		// they reach Done (David, 2026-11-04: "it takes time to stand up"; Done saved them with no step from slice 65
		// until then), or none far under its goal (a placement's, under the record that stays); a counted set's
		// optional count at none, Not counted (slice 42: no count to reach, its best never a goal); a set of reps the
		// voice counted at the reps counted by Done (2026-10-13), one with none counted yet or uncounted at its goal,
		// else its record
		let held = (m.kind == kind::HOLD)
			.then(|| CountStep::held_count(seconds, m.target.or(m.placing), facts().constants().timed_ask_share));
		// what the app counted, the count's most counts from (slice 66, `Run::count_cap`): none on a counted set, whose
		// count the app never counts
		run.counted = if m.counted { None } else { held.or(counted).filter(|&n| n > 0) };
		run.count_from = if m.counted {
			0
		} else {
			held.or_else(|| counted.filter(|&n| n > 0).or_else(|| view::goal(&m)).or(m.record)).unwrap_or(0)
		};
		self.pump_publish();
	}

	/// A steady move Keep going ran on past its plan to the full move `to` (slice 52, Events.lua `Lc.Done`), ended
	/// after `seconds`: once it ran its plan, saved at once with no ask, Yes, easily at the full move and Yes, just
	/// where Done ended it between its plan and 45 s (the steady ledger reads both, DB.lua steadyRow); a move of an app
	/// pump's own written, a rider judged as its move's set where the engine dosed it (its run over, nothing asked:
	/// `pump_judged`), else paid done by its time. Its Done or its clock's end goes on to the next move (`advance`);
	/// the pump's end under it saves it the same (`pump_stop`: End the pump here, Leave, the game's offer or its start;
	/// slice 52's double-check). False for a kept move still short of its plan, which ends as a plain one.
	fn pump_kept(&mut self, m: &PlannedMove, seconds: i64, to: i64, advance: bool) -> bool {
		let Some(run) = self.pumps.run.as_mut().filter(|_| seconds >= m.seconds.unwrap_or(0)) else { return false };
		let answer = if seconds >= to { pace::EASILY } else { pace::JUST };
		let result = MoveResult { value: seconds, elapsed: seconds, pace: Some(answer), ..MoveResult::default() };
		run.answers.insert(m.slot, answer);
		if !run.rider(m.slot) && advance {
			self.pump_own(result, Recorded::Time(seconds));
			return true;
		}
		if !run.rider(m.slot) {
			self.pump_write(result, Recorded::Time(seconds));
		} else if run.dosed(m) {
			self.pump_judge(result, Recorded::Time(seconds));
			return true;
		}
		if advance {
			self.pump_advance(true);
		}
		true
	}

	/// A count entered (Progress.lua ns.Validate through the engine), with the weight beside it on a loaded move:
	/// refused past one set, asked about once far from the record (or a weight far over the plan), else recorded as
	/// typed. A counted set's 0 is the set saved with its answer and no count (a rider's judged as its move's set since
	/// slice 42's fix round, 2026-10-17, so its clean day counts, or none on Form slipped); a weight on a move that takes
	/// none is refused. A rider's count is saving from its check on (`Run::saving`): an early end waits until the check
	/// keeps the count step or the judge answers.
	fn pump_count(&mut self, typed: Typed) -> Result<(), String> {
		let run = self.pumps.run.as_mut().ok_or("no pump")?;
		if run.stage != Stage::Counting || run.busy > 0 {
			return Err("no count now".to_owned());
		}
		let m = run.shown().cloned().ok_or("no move")?;
		if typed.load.is_some() && !view::loaded(&m) {
			return Err("this move takes no weight".to_owned());
		}
		// at most the count's cap (slice 66): a count past it is saved as it, as the dial would set it
		let typed = Typed { value: run.count_cap(&m).map_or(typed.value, |cap| typed.value.min(cap.most)), ..typed };
		if short_count(&m, typed.value) {
			// a first max set is never banked: it saves as "Could not do one" (the game's `Lc.Enter`)
			if run.first_max_set(&m) {
				return self.pump_could_not();
			}
			self.pump_short();
			return Ok(());
		}
		let max_set = run.max_set(&m);
		match run.count.enter(typed, max_set, m.counted) {
			Entered::Stays => Ok(()),
			// a counted set saved Not counted, with its answer: an app pump's own written, a rider judged as its set
			Entered::WithoutCount => {
				let (app, elapsed) = (!run.rider(m.slot), run.ran.get(&m.slot).copied().unwrap_or(0));
				let result = MoveResult { value: 0, elapsed, answer: Some(typed.answer), ..MoveResult::default() };
				if app {
					self.pump_record(result, true);
				} else {
					self.pump_judge(result, Recorded::Counted(0, typed.answer));
				}
				Ok(())
			}
			Entered::Check(typed) => {
				self.pump_validate(m.slot, typed);
				self.pump_publish();
				Ok(())
			}
		}
	}

	/// The engine checks the count entered on move `index` (`pump_checked` with its answer); the page hears it from
	/// the caller.
	fn pump_validate(&mut self, index: usize, typed: Typed) {
		let Some(id) = self.pump_busy() else { return };
		// a rider's count is on its way to its set from here (an app pump's own is written at once)
		if let Some(run) = self.pumps.run.as_mut() {
			run.saving = run.rider(index);
		}
		self.engine_call(
			move |e| e.validate(index, typed.value, typed.load),
			move |h, answer| {
				if !h.pump_answered(&id) {
					return;
				}
				match answer {
					Ok(check) => h.pump_checked(typed, check),
					Err(e) => {
						log::warn!("pump {id}: the engine checked no count: {e}");
						h.pump_publish();
						h.pump_unhold();
					}
				}
			},
		);
	}

	/// The engine's check of a count came back: refused or asked about, it stays; a hold's set asks "Could you have held
	/// longer?" first where the engine asks it (`pump_hold_ask`, slice 52); else it is saved (`pump_save`).
	fn pump_checked(&mut self, typed: Typed, check: Option<Check>) {
		let Some(run) = self.pumps.run.as_mut().filter(|r| r.stage == Stage::Counting) else {
			return self.pump_unhold();
		};
		let counted = run.shown().is_some_and(|m| m.counted);
		let hold = run.shown().is_some_and(|m| m.kind == kind::HOLD);
		match run.count.checked(typed, CountStep::asks(check, counted)) {
			Outcome::Stays => {
				self.pump_publish();
				self.pump_unhold();
			}
			Outcome::Record { typed, confirmed } if hold && typed.value > 0 => self.pump_hold_ask(typed, confirmed),
			Outcome::Record { typed, confirmed } => self.pump_save(typed, confirmed, None),
		}
	}

	/// Whether the hold set typed asks "Could you have held longer?" before it is written (slice 52, Events.lua
	/// `Lc.Enter`'s longerAsks; the engine's `ns.HoldAsk`: a fresh hold of a ledger ladder near its goal, once a ladder a
	/// game day), asked behind every set the pump wrote before it: asked, the ask waits (`Stage::Longer`, an early end
	/// held meanwhile done there, `pump_unanswered`); else, or where the engine fails to answer, the set is saved.
	fn pump_hold_ask(&mut self, typed: Typed, confirmed: bool) {
		let Some(index) = self.pumps.run.as_ref().and_then(Run::shown).map(|m| m.slot) else { return };
		let Some(id) = self.pump_busy() else { return };
		let at = unix_s(lan::now_ms());
		self.engine_call(
			move |e| e.hold_ask(index, typed.value, at),
			move |h, answer| {
				if !h.pump_answered(&id) {
					return;
				}
				let asks = answer.unwrap_or_else(|e| {
					log::warn!("pump {id}: whether the hold asks how much longer it could have held: {e}");
					false
				});
				let waiting = |r: &&mut Run| r.stage == Stage::Counting && r.shown().is_some_and(|m| m.slot == index);
				let Some(run) = h.pumps.run.as_mut().filter(waiting) else { return h.pump_unhold() };
				if !asks {
					return h.pump_save(typed, confirmed, None);
				}
				run.stage = Stage::Longer { typed, confirmed };
				run.saving = false;
				h.pump_publish();
				h.pump_unhold();
			},
		);
		self.pump_publish();
	}

	/// The answer to "Could you have held longer?" (`ns.Enum.longer`: 1 No, 2 A little, 3 A lot; slice 52, Events.lua
	/// `Lc.Longer`): the hold saved as typed with the answer in its row's reps (DB.lua holdLedger), a rider's judged as
	/// its move's set.
	fn pump_longer(&mut self, answer: u8) -> Result<(), String> {
		let run = self.pumps.run.as_ref().ok_or("no pump")?;
		let Stage::Longer { typed, confirmed } = run.stage else { return Err("nothing asked".to_owned()) };
		if run.busy > 0 {
			return Err("the engine answers first".to_owned());
		}
		self.pump_save(typed, confirmed, Some(answer));
		Ok(())
	}

	/// A count checked and kept, with a hold's answer to "Could you have held longer?" (`longer`): an app pump records
	/// its own move's (`pump_record`, whose count pours), and a rider's is judged (`pump_judge`), a typed 0 there done
	/// without a count (Events.lua `Lc.CouldNot`'s note: "Could not do one" stores the 0). An early end held for a
	/// rider's count (`Held`) waits on through its judge, or on a typed 0 through the engine's step down
	/// (`pump_empty`), the move done on its count step (no next move first), so the end pays it done.
	fn pump_save(&mut self, typed: Typed, confirmed: bool, longer: Option<u8>) {
		let Some(run) = self.pumps.run.as_mut() else { return self.pump_unhold() };
		let elapsed = run.shown().and_then(|m| run.ran.get(&m.slot)).copied().unwrap_or(0);
		let counted = run.shown().is_some_and(|m| m.counted);
		let result = MoveResult {
			value: typed.value,
			elapsed,
			confirmed,
			load: typed.load,
			answer: Some(typed.answer),
			longer,
			..MoveResult::default()
		};
		if !run.on_rider() {
			self.pump_record(result, true);
		} else if typed.value > 0 {
			let shown = if counted {
				Recorded::Counted(typed.value, typed.answer)
			} else {
				Recorded::Count(typed.value, typed.load)
			};
			self.pump_judge(result, shown);
		} else {
			// done without a count, through the engine for the step down (`pump_empty`); a held end waits for it
			self.pump_empty();
		}
	}

	/// An app pump's own move saved (slice 65's hold, every move since slice 66): what it saved pours into its count bar
	/// (`pump_pour`), its row written as the pour lands; a count of 0 pours nothing and is written at once.
	fn pump_own(&mut self, result: MoveResult, shown: Recorded) {
		if result.value > 0 {
			self.pump_pour(result, shown);
		} else {
			self.pump_written(result, shown);
		}
	}

	/// An app pump's own move written at once with what its line shows (a pose's stretch checks read behind its row,
	/// `pump_checked_after`), and the next move.
	fn pump_written(&mut self, result: MoveResult, shown: Recorded) {
		let Some(slot) = self.pumps.run.as_ref().and_then(Run::shown).map(|m| m.slot) else { return };
		self.pump_write(result, shown);
		self.pump_checked_after(slot);
		self.pump_advance(true);
	}

	/// An app pump's own move saved (`pump_own`): the app's own sheet before its write is read, and its count or seconds
	/// pour (`Stage::Saved`), the row unwritten until the pour lands (`pump_pour_land`); meanwhile the stage it was saved
	/// on stays, taking no second answer (`Run::pouring`). An engine that fails to read it has the move written at once,
	/// as before the pour.
	fn pump_pour(&mut self, result: MoveResult, shown: Recorded) {
		let Some(run) = self.pumps.run.as_mut().filter(|r| r.pouring.is_none() && r.pour.is_none()) else { return };
		let Some(slot) = run.shown().map(|m| m.slot) else { return };
		run.pouring = Some((result, shown));
		// a timed move's run is over: no mark of its side comes while the sheet is read
		run.cues = Cues::default();
		let Some(id) = self.pump_busy() else { return };
		let at = unix_s(lan::now_ms());
		self.engine_call(
			move |e| e.summary(at),
			move |h, answer| {
				if !h.pump_answered(&id) {
					return;
				}
				// Back, the next move or the pump's early end took it meanwhile (`pump_pour_left`)
				let waiting = |r: &&mut Run| r.shown().is_some_and(|m| m.slot == slot) && r.pouring.is_some();
				let Some(run) = h.pumps.run.as_mut().filter(waiting) else { return };
				run.pouring = None;
				// a sheet that does not read leaves the save's own rewards unknown: written at once, as before the pour
				let before = match answer {
					Ok(summary) => view::Sum::read(&summary),
					Err(e) => {
						log::warn!("pump {id}: the sheet before the move's save: {e}");
						None
					}
				};
				let Some(before) = before else { return h.pump_written(result, shown) };
				let Some(m) = run.shown().cloned() else { return };
				let step = run.slots.get(slot).and_then(Option::as_ref).map_or((0, 0), |s| (s.ladder, s.rung));
				let next = run.moves.get(run.current + 1).and_then(|n| facts().name(n.r#move));
				let fresh = run.written.is_empty();
				let covered = h.pumps.covered;
				let saved = view::pour::Saved { result, shown, step, before, fresh, next, covered };
				run.pour = Some(view::pour::Pour::plan(lan::now_ms(), &m, saved));
				run.stage = Stage::Saved;
				h.pump_publish();
			},
		);
	}

	/// The move's pour lands (its last grain, a click on the stage, Next or Finish; `still`: Reduce Motion): its row is
	/// written now (a pose's stretch checks read behind it), and with `earn` its jingle plays where one is due and what
	/// the save earned is read after the write (`pump_saved_earned`); Next and Finish read nothing, as the move goes on.
	fn pump_pour_land(&mut self, still: bool, earn: bool) {
		let now = lan::now_ms();
		let Some(pour) = self.pumps.run.as_mut().and_then(|r| r.pour.as_mut()).filter(|p| !p.landed()) else { return };
		let event = pour.land(now, still);
		let (slot, result, shown) = (pour.slot, pour.result, pour.shown);
		if let Some(event) = event.filter(|_| earn) {
			self.cue(Cue::Event(event));
		}
		self.pump_write(result, shown);
		self.pump_checked_after(slot);
		if !earn {
			return;
		}
		let Some(run) = self.pumps.run.as_ref() else { return };
		let own: Vec<PlannedMove> = run.moves.iter().filter(|m| m.slot == slot).cloned().collect();
		let firsts = view::firsts(&own, &run.recorded, |m| run.max_set(m));
		let at = unix_s(now);
		// a sheet that does not read after the write still shows the move's row and plays its jingle
		self.pump_call(
			move |e| {
				let summary = e.summary(at).map_err(|e| log::warn!("pump: the sheet after the move's save: {e}")).ok();
				let goals = firsts
					.iter()
					.map(|&(slot, move_id, n)| Ok((slot, e.next_goal(move_id, n)?)))
					.collect::<Result<Vec<_>, EngineError>>()
					.unwrap_or_default();
				Ok((summary, goals))
			},
			move |h, (summary, goals)| h.pump_saved_earned(slot, summary.as_ref(), &goals),
		);
		self.pump_publish();
	}

	/// What the move's save earned (the app's own sheet after its write; without one, the move's row alone), while its
	/// pour is still on screen: its rows and its step's reveal, with the jingle due at once under Reduce Motion.
	fn pump_saved_earned(&mut self, slot: usize, summary: Option<&Value>, goals: &[(usize, i64)]) {
		let after = summary.and_then(view::Sum::read);
		let Some(run) = self.pumps.run.as_mut().filter(|r| r.stage == Stage::Saved) else { return };
		let Some(mut pour) = run.pour.take_if(|p| p.slot == slot) else { return };
		let event = pour.earn(lan::now_ms(), after.as_ref(), run, goals);
		run.pour = Some(pour);
		if let Some(event) = event {
			self.cue(Cue::Event(event));
		}
		self.pump_publish();
	}

	/// A click on the stage while a move's pour shows, or Reduce Motion from its first frame (`still`): the pour at its
	/// end and its row written; once landed, its step's reveal and its rows at their end.
	fn pump_saved_land(&mut self, still: bool) -> Result<(), String> {
		let run = self.pumps.run.as_mut().filter(|r| r.stage == Stage::Saved).ok_or("no pour")?;
		let pour = run.pour.as_mut().ok_or("no pour")?;
		if !pour.landed() {
			self.pump_pour_land(still, true);
			return Ok(());
		}
		for event in pour.land_reveal(lan::now_ms()) {
			self.cue(Cue::Event(event));
		}
		self.pump_publish();
		Ok(())
	}

	/// The pump ends while a move's pour runs (`pump_stop`: the window closed, End the pump here, Leave, the game's offer,
	/// felt unwell, the app quitting; slice 65): the pour lands and its row is written with what it saved, as its landing
	/// would write it, with the felt-unwell `reason` where that ended it, whose row then locks; only Back drops it. True
	/// where it wrote that row.
	fn pump_pour_left(&mut self, reason: Option<u8>) -> bool {
		let Some(run) = self.pumps.run.as_mut().filter(|r| !r.on_rider()) else { return false };
		if run.stage == Stage::Saved {
			let pending = run.pour.as_ref().is_some_and(|p| !p.landed());
			if let Some(pour) = run.pour.as_mut().filter(|_| pending) {
				pour.result.reason = reason;
				self.pump_pour_land(false, false);
			}
			if let Some(run) = self.pumps.run.as_mut() {
				run.pour = None;
			}
			return pending;
		}
		// on its way to the pour (the sheet before it read): the move as saved
		let Some((result, shown)) = run.pouring.take() else { return false };
		let slot = run.shown().map(|m| m.slot);
		let result = MoveResult { reason, ..result };
		// written here as its pour's landing would, so neither an ask still waiting (`pump_unanswered`) nor the stage it
		// was saved on (a counted set's count step, `pump_stop`) writes it again
		run.stage = Stage::Saved;
		self.pump_write(result, shown);
		if let Some(slot) = slot {
			self.pump_checked_after(slot);
		}
		true
	}

	/// Next or Finish on a move's pour (slice 65, live from its first frame): the pour lands and its row is written
	/// first where it has not, the move's one jingle plays where none has, and the next move's Ready view shows, or the
	/// Result after the pump's last move.
	fn pump_saved_next(&mut self) -> Result<(), String> {
		let run = self.pumps.run.as_mut().filter(|r| r.stage == Stage::Saved).ok_or("no pour")?;
		let event = run.pour.as_mut().ok_or("no pour")?.leave();
		self.pump_pour_land(false, false);
		if let Some(run) = self.pumps.run.as_mut() {
			run.pour = None;
		}
		if let Some(event) = event {
			self.cue(Cue::Event(event));
		}
		self.pump_advance(false);
		Ok(())
	}

	/// Whether the answer to "How close to the goal pose?" of move `slot` can pass a stretch check (`Engine::checks`,
	/// slice 42's fix round, 2026-10-17: a card's check first as its Start runs it, `ns.RunsCheck`, then the stretch
	/// slot's, then every other live check the pose answers: the game's one list, `ns.PoseChecks`, since slice 42's
	/// last review, 2026-10-18), asked of the engine as the ask opens, behind every set the pump wrote before it, so a
	/// set that changed the climb (a record that reached a step another family's waits for, a proof pose that completed
	/// a climb) counts. The engine keeps them for what they came to after the answer (`pump_checked_after`); an engine
	/// that fails to answer leaves none.
	fn pump_checks(&mut self, slot: usize) {
		let Some((id, started)) = self.pumps.run.as_ref().map(|r| (r.id.clone(), r.what.check())) else { return };
		self.engine_call(
			move |e| e.checks(slot, started),
			move |h, answer| {
				let count = answer.unwrap_or_else(|e| {
					log::warn!("pump {id}: the checks the pose's answer can pass: {e}");
					0
				});
				let shown = |r: &&mut Run| r.id == id && r.shown().is_some_and(|m| m.slot == slot);
				let marking = |r: &&mut Run| matches!(r.stage, Stage::Marking { .. });
				if let Some(run) = h.pumps.run.as_mut().filter(shown).filter(marking) {
					if count > 0 {
						run.checks.insert(slot);
					} else {
						run.checks.remove(&slot);
					}
				}
			},
		);
	}

	/// The answer to "How close to the goal pose?" (`ns.Enum.mark`; David, 2026-10-09: "Maybe 4 options or so on how well
	/// the pose could be done between perfect, good, okay, bad?"): the pose written done in its seconds, the answer in its
	/// row (DB.lua markAttempt counts the day at Good or better, Perfect alone below a risky step), then what the checks
	/// it could pass came to (`pump_checked_after`), read before the next move or the Result asks the engine anything.
	/// Each answer is a line in helper.log, as the game's log has its own (Events.lua `Lc.Mark`).
	fn pump_mark(&mut self, answer: u8) -> Result<(), String> {
		let run = self.pumps.run.as_ref().ok_or("no pump")?;
		let Stage::Marking { seconds } = run.stage else { return Err("nothing asked".to_owned()) };
		if run.pouring.is_some() {
			return Err("answered already".to_owned());
		}
		let m = run.shown().ok_or("no move")?;
		log::info!("pump {}: move {}, how close: {answer}", run.id, m.r#move);
		let slot = m.slot;
		let result = MoveResult { mark: Some(answer), ..timed_result(m, seconds) };
		if let Some(run) = self.pumps.run.as_mut() {
			run.answers.insert(slot, answer);
		}
		self.pump_own(result, Recorded::Time(result.value));
		Ok(())
	}

	/// What the stretch checks pose `slot` could pass came to after its answer (`Engine::checked`, slice 42's fix round,
	/// 2026-10-17: passed or not, and what a passed one opens open now or still held), asked behind its row's write, so
	/// the Result reads it; none where its ask read no check.
	fn pump_checked_after(&mut self, slot: usize) {
		if self.pumps.run.as_ref().is_none_or(|r| !r.checks.contains(&slot)) {
			return;
		}
		self.pump_call(
			move |e| e.checked(slot),
			move |h, outcomes| {
				if let Some(run) = h.pumps.run.as_mut() {
					run.checked.insert(slot, outcomes);
				}
			},
		);
	}

	/// The answer to "Did you keep that pace to the end?" (`ns.Enum.pace`: 1 Yes, just, 2 I eased off, 3 Yes, easily;
	/// slice 42, 2026-10-17, the third answer since slice 52): the steady move written done in its seconds, the answer in
	/// its row's flag bits (DB.lua steadyRow: the steady ledger's estimate and doubt, a full move on a Yes); a rider's
	/// judged as its move's set (`pump_judge`, slice 42's fix round), which the payment writes. None while the engine
	/// still judges it.
	fn pump_pace(&mut self, answer: u8) -> Result<(), String> {
		let run = self.pumps.run.as_ref().ok_or("no pump")?;
		let Stage::Pacing { seconds } = run.stage else { return Err("nothing asked".to_owned()) };
		if run.busy > 0 && run.on_rider() {
			return Err("the engine judges it".to_owned());
		}
		if run.pouring.is_some() {
			return Err("answered already".to_owned());
		}
		let app = !run.on_rider();
		let m = run.shown().ok_or("no move")?;
		let slot = m.slot;
		let result = MoveResult { pace: Some(answer), ..timed_result(m, seconds) };
		if let Some(run) = self.pumps.run.as_mut() {
			run.answers.insert(slot, answer);
		}
		if app {
			self.pump_record(result, false);
		} else {
			self.pump_judge(result, Recorded::Time(result.value));
		}
		Ok(())
	}

	/// The pump ends while "How close to the goal pose?", "Did you keep that pace to the end?" or "Could you have held
	/// longer?" waits (`pump_stop`: End the pump here, Leave, the game's offer, felt unwell): a timed move ran its whole
	/// time, so it is written done in its seconds with no answer, practice for its mark as the game's is (DB.lua
	/// markAttempt) and no full move, with the felt-unwell `reason` when that ended it, whose row then locks; a rider's
	/// steady move is paid done by its time, with no set. A hold's set is saved as typed with no answer
	/// (`pump_hold_left`). True where it wrote a row of the pump's own (with `reason`, a lock); false where nothing is
	/// asked or a rider was asked.
	fn pump_unanswered(&mut self, reason: Option<u8>) -> bool {
		let Some(run) = self.pumps.run.as_mut() else { return false };
		if let Stage::Longer { typed, confirmed } = run.stage {
			return self.pump_hold_left(typed, confirmed, reason);
		}
		let (Stage::Marking { seconds } | Stage::Pacing { seconds }) = run.stage else { return false };
		let Some(m) = run.shown() else { return false };
		let result = MoveResult { reason, ..timed_result(m, seconds) };
		// a rider's steady move unanswered is paid done by its time, with no set (the engine writes none of its rows)
		if run.rider(m.slot) {
			let slot = m.slot;
			run.recorded.insert(slot, Recorded::Time(result.value));
			return false;
		}
		self.pump_write(result, Recorded::Time(result.value));
		true
	}

	/// A hold left at "Could you have held longer?" (Events.lua `Lc.OweOpen`: its typed set, no answer): an app pump's
	/// own written with the felt-unwell `reason` where that ended it (true: a row of the pump's own); a rider judged as
	/// its move's set, the end held until the judge answers (`Held`), its count step again meanwhile (`pump_judged`).
	fn pump_hold_left(&mut self, typed: Typed, confirmed: bool, reason: Option<u8>) -> bool {
		let Some(run) = self.pumps.run.as_mut() else { return false };
		let Some(slot) = run.shown().map(|m| m.slot) else { return false };
		let elapsed = run.ran.get(&slot).copied().unwrap_or(0);
		let result =
			MoveResult { value: typed.value, elapsed, confirmed, load: typed.load, reason, ..MoveResult::default() };
		let shown = Recorded::Count(typed.value, typed.load);
		if run.rider(slot) {
			self.pump_judge(result, shown);
			return false;
		}
		self.pump_write(result, shown);
		true
	}

	/// "Change it": the count asked about goes (or, asked about the weight, the weight planned comes back), nothing
	/// written.
	fn pump_count_clear(&mut self) -> Result<(), String> {
		let run = self.pumps.run.as_mut().filter(|r| r.stage == Stage::Counting).ok_or("no count now")?;
		let plan = run.shown().and_then(|m| m.load);
		run.count.change(plan);
		self.pump_publish();
		Ok(())
	}

	/// "Done without a count" (Events.lua Lc.EmptyDone): the move done, no record changed; a rider is paid done,
	/// with no set, once the engine says the step down Swap's easier version makes (`pump_empty`). None while the engine
	/// still checks or judges a count, and none on a counted set's form step, which only its three answers save (the
	/// game's and the phones' offer it nowhere there): its row with no answer would read Yes, to the end (`ns.Enum.answer`
	/// 0), a clean window the player never said (2026-10-31).
	fn pump_without_count(&mut self) -> Result<(), String> {
		let run =
			self.pumps.run.as_mut().filter(|r| r.stage == Stage::Counting && r.busy == 0).ok_or("no count now")?;
		let m = run.shown().ok_or("no move")?;
		if m.counted {
			return Err(format!("pump {}: a counted set's answers save it, never done without a count", run.id));
		}
		let slot = m.slot;
		if run.rider(slot) {
			self.pump_empty();
			return Ok(());
		}
		let elapsed = run.ran.get(&slot).copied().unwrap_or(0);
		self.pump_write(MoveResult { value: 0, elapsed, ..MoveResult::default() }, Recorded::Done);
		self.pump_advance(true);
		Ok(())
	}

	/// "Could not do one" ("Could not hold it"; Events.lua Lc.CouldNot): the stored 0, an app pump's own written at once,
	/// a rider's judged (`pump_judge`). None while the engine still checks or judges a count.
	fn pump_could_not(&mut self) -> Result<(), String> {
		let run =
			self.pumps.run.as_ref().filter(|r| r.stage == Stage::Counting && r.busy == 0).ok_or("no count now")?;
		let elapsed = run.shown().and_then(|m| run.ran.get(&m.slot)).copied().unwrap_or(0);
		let result = MoveResult { value: 0, elapsed, could_not: true, ..MoveResult::default() };
		if run.on_rider() {
			self.pump_judge(result, Recorded::Count(0, None));
		} else {
			self.pump_write(result, Recorded::Count(0, None));
			self.pump_advance(false);
		}
		Ok(())
	}

	/// A rider's count saved (a catch-up's since 2026-10-12, David: it "did not ask me how many I could do"; a match
	/// pump's rider since slice 61), the moment the
	/// payment dates its rows: judged on the engine as the game's own catch-up judges an owed move's (`Engine::judge`)
	/// and written nowhere (the payment sends it as the move's set, `pump_paid`; an app row would pay its XP twice).
	fn pump_judge(&mut self, result: MoveResult, shown: Recorded) {
		self.pump_judge_by(shown, result.could_not, move |e, slot, at| e.judge(slot, result, at));
	}

	/// A rider done on its count step without a count (Events.lua `Lc.EmptyDone`), paid done with no set: it
	/// goes through the engine as a count does (`pump_judge_by`), whose answer is the step down Swap's easier version
	/// makes whatever the outcome (`Engine::empty_done`, since 2026-10-26), so the payment at the end has it; an early
	/// end held meanwhile is done after it.
	fn pump_empty(&mut self) {
		self.pump_judge_by(Recorded::Done, false, |e, slot, at| {
			Ok(Judgement { set: None, down: e.empty_done(slot, at)? })
		});
	}

	/// A rider saved through the engine (`judge`: its count judged, or a move done without a count's step down)
	/// at the moment the payment dates its rows. The next move comes once the engine answers (`pump_judged`), so the
	/// payment at the end has every set and step down; until then Back waits and an early end is held (`Held`), done
	/// after the answer. An engine that fails to judge leaves the count step as it was, to save again; a kept steady
	/// move's end, which has no step, is paid done by its time as a move the engine did not dose (`pump_kept`); a move
	/// done without a count is done all the same, with no step down.
	fn pump_judge_by<J>(&mut self, shown: Recorded, could_not: bool, judge: J)
	where
		J: FnOnce(&Engine, usize, i64) -> Result<Judgement, EngineError> + Send + 'static,
	{
		let Some(slot) = self.pumps.run.as_ref().and_then(Run::shown).map(|m| m.slot) else { return };
		let Some(id) = self.pump_busy() else { return };
		if let Some(run) = self.pumps.run.as_mut() {
			run.saving = true;
		}
		let saved = lan::now_ms();
		let at = unix_s(saved);
		self.engine_call(
			move |e| judge(e, slot, at),
			move |h, answer| {
				if !h.pump_answered(&id) {
					return;
				}
				if let Some(run) = h.pumps.run.as_mut() {
					run.saving = false;
				}
				match answer {
					Ok(judgement) => h.pump_judged(slot, saved, shown, could_not, judgement),
					Err(e) if shown == Recorded::Done => {
						log::warn!("pump {id}: done without a count, its step down unknown: {e}");
						h.pump_judged(slot, saved, shown, could_not, Judgement::default());
					}
					Err(e) => {
						log::warn!("pump {id}: the engine judged no count: {e}");
						let kept = h.pumps.run.as_ref().filter(|r| r.kept_judged() && r.held == Held::default());
						if kept.and_then(Run::shown).is_some_and(|m| m.slot == slot) {
							h.pump_advance(true);
						} else {
							h.pump_publish();
						}
					}
				}
				h.pump_unhold();
			},
		);
		self.pump_publish();
	}

	/// A rider saved through the engine, while it still waits on its count (or a kept steady move on its end): its set
	/// kept as the move's (saved at `saved`, unix ms) and the step down Swap's easier version made, and the next move,
	/// unless an early end waits (`Held`); the game's sound for it at once (`win_sound`: a rider has no reveal;
	/// none when felt unwell ends the pump after it, as no cheer rides on a pump the body said no to), else the move's
	/// done, never after "Could not do one".
	fn pump_judged(&mut self, slot: usize, saved: i64, shown: Recorded, could_not: bool, judgement: Judgement) {
		// a count or a pace asked, or a kept steady move's end (`pump_kept`: its run over, nothing asked)
		let asked = |r: &Run| {
			matches!(r.stage, Stage::Counting | Stage::Pacing { .. } | Stage::Longer { .. }) || r.kept_judged()
		};
		let waiting = |r: &&mut Run| asked(r) && r.shown().is_some_and(|m| m.slot == slot);
		let Some(run) = self.pumps.run.as_mut().filter(waiting) else { return };
		run.recorded.insert(slot, shown);
		run.count.clear();
		// a hold judged from its ask (an early end there) waits for that end on its count step, its set recorded
		if matches!(run.stage, Stage::Longer { .. }) {
			run.stage = Stage::Counting;
		}
		if let Some(judged) = &judgement.set {
			run.sets.insert(slot, catchup_set(judged, saved));
		}
		if let Some(down) = judgement.down {
			run.downs.insert(slot, SetStep { ladder: down.ladder, rung: down.rung });
		}
		let held = run.held != Held::default();
		let unwell = run.held.stop == Some(Stop::Unwell);
		let win = judgement.set.and_then(|j| win_sound(j.outcome)).filter(|_| !unwell);
		if let Some(event) = win {
			self.cue(Cue::Event(event));
		}
		if held {
			return self.pump_publish();
		}
		self.pump_advance(win.is_none() && !could_not);
	}

	/// The early end held while the engine checked or judged a rider's count, now that the count is a set or stays on
	/// its step (or the engine failed it): the count saves no more, the pump ends as asked, and a Leave closes the window
	/// or quits after it.
	fn pump_unhold(&mut self) {
		let Some(held) = self.pumps.run.as_mut().map(|r| {
			r.saving = false;
			std::mem::take(&mut r.held)
		}) else {
			return;
		};
		if let Some(why) = held.stop
			&& let Err(e) = self.pump_stop(why)
		{
			log::warn!("pump: the end held for the count: {e}");
		}
		if let Some(quit) = held.leave
			&& let Err(e) = self.pump_leave(quit)
		{
			log::warn!("pump: the Leave held for the count: {e}");
		}
	}

	/// An app pump's move result (`typed`: a count from the dial, else a timed move's seconds), poured and then written
	/// (`pump_own`); a typed 0 is done without a count, a counted set's its clean reps (0: not counted) with its answer.
	fn pump_record(&mut self, result: MoveResult, typed: bool) {
		let counted = self.pumps.run.as_ref().and_then(Run::shown).is_some_and(|m| m.counted);
		let shown = if !typed {
			Recorded::Time(result.value)
		} else if counted {
			Recorded::Counted(result.value, result.answer.unwrap_or(answer::YES))
		} else if result.value > 0 {
			Recorded::Count(result.value, result.load)
		} else {
			Recorded::Done
		};
		self.pump_own(result, shown);
	}

	/// The current move's result through the engine (its rows kept as they come back), and what its line shows; a set
	/// with a weight keeps whether its row is a record.
	fn pump_write(&mut self, result: MoveResult, shown: Recorded) {
		let Some(run) = self.pumps.run.as_mut() else { return };
		let Some(slot) = run.shown().map(|m| m.slot) else { return };
		run.recorded.insert(slot, shown);
		run.count.clear();
		let at = unix_s(lan::now_ms());
		let weighed = result.load.is_some();
		self.pump_write_rows(
			move |e| e.finish(slot, result, at),
			move |run, rows, finished| {
				// its set row, after the load row that comes before it
				if let Some(set) = rows.iter().find(|r| weighed && kind::is_move(r.kind)) {
					run.beat.insert(slot, set.outcome == outcome::BEAT);
				}
				// the steps it mastered and a check's lift, which the Result names, the cardio day it brought and the
				// day's last ember
				run.written.extend_from_slice(rows);
				run.days.extend(finished.day.map(|day| (slot, day)));
				run.kindled = finished.kindled.or(run.kindled);
				run.embers_lit += i64::from(finished.lit);
			},
		);
		// the steps its set left mastered with no row (slice 42's fix round), quietly, never on the Result
		self.engine_sweep();
	}

	/// The next move, or the pump's end after the last. `done`: the move ended done (a sound says so), never after
	/// "Could not do one" or feeling unwell.
	fn pump_advance(&mut self, done: bool) {
		if done {
			self.cue(Cue::Event("done"));
		}
		let Some(run) = self.pumps.run.as_mut() else { return };
		if run.current + 1 < run.moves.len() {
			run.current += 1;
			self.pump_ready();
			self.pump_publish();
		} else {
			self.pump_finish(false);
		}
	}

	/// Back (David, 2026-10-05: "go back and have it like I never started the exercise"): the move's Ready view as if
	/// Start never came, nothing written, the pump's first go and last end as the Start found them, and Swap's options
	/// asked again where none are kept (an answer that came after the Start, which the stale guard dropped).
	fn pump_back(&mut self) -> Result<(), String> {
		let run = self.pumps.run.as_mut().filter(|r| r.can_back()).ok_or("no Back now")?;
		if let Some(slot) = run.shown().map(|m| m.slot) {
			run.ran.remove(&slot);
		}
		run.side = None;
		run.move_start = None;
		run.asked = None;
		run.count.clear();
		run.pour = None;
		run.pouring = None;
		run.cues = Cues::default();
		if let Some((start, riders, end)) = run.before_start.take() {
			(run.pump_start, run.riders_start, run.last_end) = (start, riders, end);
		}
		run.stage = Stage::Ready;
		if run.swaps.is_none() {
			self.pump_swaps();
		}
		self.hush();
		self.pump_publish();
		Ok(())
	}

	/// Skip on a rider's Ready view (a catch-up's since 2026-10-09, a match pump's rider since slice 61; the game's
	/// `Lc.Skip`): the shown move's owed row skipped at once (`pump_skip_owed`), the move leaves the pump, on the engine
	/// too (`Engine::skip`, asked before the next move's Swap options, so they count it neither in the pump nor owed; a
	/// match pump's later moves one place down there and here, `Run::drop_move`), and the next one shows; with none left
	/// the pump ends, as done where a move of it ran (its moves paid), else it leaves the screen.
	fn pump_skip(&mut self) -> Result<(), String> {
		let run = self
			.pumps
			.run
			.as_ref()
			.filter(|r| r.stage == Stage::Ready && r.busy == 0 && r.on_rider())
			.ok_or("no Skip now")?;
		let slot = run.shown().map(|m| m.slot).ok_or("no move")?;
		let owed = run.owed.get(&slot).copied().ok_or("no move")?;
		let (id, merged) = (run.id.clone(), run.what.app());
		self.pump_skip_owed(owed.move_id, owed.ts)?;
		self.pump_engine_skip(&id, slot);
		let run = self.pumps.run.as_mut().ok_or("no pump")?;
		if merged {
			run.drop_move(slot);
		} else {
			run.leave_catch_up(run.current);
		}
		self.pump_after_leaving();
		Ok(())
	}

	/// The move on the Ready view left the pump (Skip, a catch-up's Pain): the next move's Ready view, else the pump
	/// ends, leaving the screen with no card where no move of it ran.
	fn pump_after_leaving(&mut self) {
		let Some(run) = self.pumps.run.as_mut() else { return };
		run.spoken = None;
		if run.current < run.moves.len() {
			self.pump_ready();
			self.pump_publish();
		} else if run.ran.is_empty() {
			self.pump_dismiss();
		} else {
			self.pump_finish(false);
		}
	}

	/// A move ended too short to count (`min_reps`, `min_seconds`: `short_count`, `pump_end_side`; Rapha, 2026-11-03,
	/// the game's `Lc.ShortBank`): banked without a word or a row of its own. A match pump's own move goes in the game's
	/// owed queue as Bank all puts it there (`Run::bank`, `pump_write_bank`), under an id of its own (`short_id`), a
	/// rider stays the unfinished move it is, a move of a pump the player started owes nothing; its run is forgotten
	/// (`ran`), so no Result and no payment counts it, and the next move shows (after the last the pump ends, with no
	/// card where no move of it was kept, as Bank all closes the game's, `Lc.Finish("banked")`; an expedition's round
	/// goes on to the next).
	fn pump_short(&mut self) {
		let Some(run) = self.pumps.run.as_mut() else { return };
		let Some(slot) = run.shown().map(|m| m.slot) else { return };
		log::info!("pump {}: move {slot} ended too short to count, banked", run.id);
		run.ran.remove(&slot);
		run.side = None;
		run.move_start = None;
		run.asked = None;
		run.count.clear();
		let banks = run.what.app() && run.offer.is_some() && !run.rider(slot);
		if let Some((mut bank, slots)) = banks.then(|| run.bank()) {
			bank.moves = bank.moves.into_iter().zip(slots).filter(|&(_, s)| s == slot).map(|(m, _)| m).collect();
			bank.id = short_id(&bank.id, slot);
			self.pump_write_bank(&bank);
		}
		self.pump_advance(false);
		let empty = |r: &Run| r.ran.is_empty() && r.recorded.is_empty() && !matches!(r.what, What::Expedition { .. });
		if self.pumps.run.as_ref().is_some_and(|r| !r.running() && empty(r)) {
			self.pump_dismiss();
		}
	}

	/// A rider at `slot` leaves the pump on the engine (`Engine::skip`): the engine's catch-up holds its other moves'
	/// places, a match pump's moves its later moves one place down.
	fn pump_engine_skip(&self, id: &str, slot: usize) {
		let id = id.to_owned();
		self.engine_call(
			move |e| e.skip(slot),
			move |_, skipped| match skipped {
				Ok(true) => {}
				Ok(false) => log::warn!("pump {id}: the engine's pump holds no rider {slot} to skip"),
				Err(e) => log::warn!("pump {id}: the skip of move {slot} on the engine: {e}"),
			},
		);
	}

	/// A Skip of an unfinished move, from Pump home's list or a rider's Ready view (since 2026-10-09; David,
	/// 2026-10-04: "the skip should be per unfinished move without a confirm"): the game's skipped paid row through
	/// `Session::skip` with the computer's source, which earns nothing and costs nothing; none while a World of Warcraft
	/// client with QuestPump runs, which asks the move itself (the page shows no Skip then).
	pub(crate) fn pump_skip_owed(&mut self, move_id: u16, ts: Option<u32>) -> Result<(), String> {
		if self.wow_with_addon() {
			return Err("the game asks its unfinished moves".to_owned());
		}
		let skip = CatchupPump {
			source: Source::Computer,
			id: new_id(),
			start: u32::try_from(unix_s(lan::now_ms()).max(0)).unwrap_or(u32::MAX),
			moves: vec![CatchupMove {
				seconds: 0,
				move_id,
				ts,
				set: None,
				ran: None,
				kind: None,
				ladder: None,
				rung: None,
				allout: false,
			}],
			measured: None,
			unwell_at: None,
		};
		let (skipped, fx) = self.session.skip(&skip, &runtime::now());
		self.apply(fx);
		match skipped {
			Ok(_) => {
				log::info!("pump {}: skipped move {move_id}, unfinished since {}", skip.id, ts.unwrap_or(0));
				Ok(())
			}
			Err(e) => Err(format!("skip {}: {e}", skip.id)),
		}
	}

	/// The Ready view's way to put the moves off (slice 58; the game's Bank all and Bank the rest): on a match's pump
	/// (`Run::offer`, a match offer's or the test offer's, whatever the player picked from its Start a pump window) Bank
	/// it while a move of its own without a result is left from the one on Ready on (its own all-out window after its
	/// riders keeps it), Bank the rest once only its riders are left (slice 61, the game's `Lc.OnlyRidersLeft`); on
	/// every catch-up Bank the rest (its moves left stay unfinished); none on a pump the player started (call 4: it owes
	/// nothing), off its Ready view, or while felt unwell locks pumps (its Ready view shows Close). A World of Warcraft
	/// client with QuestPump running holds no bank: the moves banked are in no queue it asks, and reach it at its next
	/// load through the mirror as every row of the helper's does.
	fn pump_bank_way(&self, run: &Run) -> Option<Bank> {
		if run.stage != Stage::Ready || self.session.locked_until(unix_s(lan::now_ms())).is_some() {
			return None;
		}
		if !run.what.app() {
			return Some(Bank::Rest);
		}
		run.offer.as_ref()?;
		Some(if run.only_riders_left() { Bank::Rest } else { Bank::It })
	}

	/// Bank all or Bank the rest (`pump.bank`, slice 58; the game's `Lc.Bank`, one click, no confirm): Bank all puts
	/// every move of a match pump's own not done yet, from the one on Ready on, in the game's owed queue, a banked row
	/// (`Run::bank`, `pump_write_bank`), then the pump ends as End the pump here ends it (`pump_stop`: the moves done
	/// count and its riders done are paid, with none done it leaves the screen, its offer stays taken), also where the
	/// store refuses the rows; a match pump's Bank the rest, on a rider, ends it as done, its Result with its riders
	/// done paid and the rest left unfinished (slice 61, the game's `Lc.Finish("rest")`: `Stop::Rest`); a
	/// catch-up's leaves its moves left unfinished as End the pump here left them. Refused where `pump_bank_way` shows
	/// none, and while an engine call for the pump is under way (a Swap's moves).
	fn pump_bank(&mut self) -> Result<(), String> {
		let run = self.pumps.run.as_ref().filter(|r| r.busy == 0).ok_or("no Bank now")?;
		match self.pump_bank_way(run).ok_or("no Bank now")? {
			Bank::It => {
				let (bank, _) = run.bank();
				self.pump_write_bank(&bank);
				self.pump_stop(Stop::Left)
			}
			Bank::Rest if run.what.app() => self.pump_stop(Stop::Rest),
			Bank::Rest => self.pump_stop(Stop::Left),
		}
	}

	/// End the pump here (the "···"'s past the Ready view, `pump.end`; the Ready bar's Close on a pump that banks
	/// nothing, Pause's how long, `pump_pause`; Leave's on its ×): as Leave ends it, but on a match pump whose riders
	/// alone are left, where it reads Bank the rest, as that ends it (`Stop::Rest`, slice 61).
	fn pump_end(&mut self) -> Result<(), String> {
		let rest = self.pumps.run.as_ref().is_some_and(|r| r.what.app() && r.ends_rest());
		self.pump_stop(if rest { Stop::Rest } else { Stop::Left })
	}

	/// Pause's how long on the Ready view (`pump.pause {kind}`; the game's `Lc.PausePump`, where the character pauses
	/// and the pump ends): the offers pause as the Games tab's Pause offers pauses them (`pause_offers`: an hour, four
	/// hours, until the next game day, while the watched game running now runs, until resumed), then the pump ends as
	/// its bar's Close ends it (`pump_end`: its own moves not done count for nothing, a catch-up's and its riders stay
	/// unfinished, with only riders left a match pump ends done). Refused, nothing changed, off a Ready view and where
	/// that pause cannot be (no offers on this computer, no watched game runs): the pump is looked at first and the
	/// pause is set whole or not at all, so the two happen together or neither does.
	fn pump_pause(&mut self, kind: &str) -> Result<(), String> {
		self.pumps.run.as_ref().filter(|r| r.stage == Stage::Ready).ok_or("no Pause now")?;
		self.pause_offers(kind)?;
		self.pump_end()
	}

	/// A match pump's moves `bank` names written in the game's owed queue (`Session::bank`: the computer's source, dated
	/// its offered row's time, at its tier), or why none was (a reset since it began, the felt-unwell lock), logged.
	fn pump_write_bank(&mut self, bank: &BankPump) {
		let (banked, fx) = self.session.bank(bank, &runtime::now());
		self.apply(fx);
		match banked {
			Ok(rows) => log::info!("pump {}: {} moves banked, unfinished from {}", bank.id, rows.len(), bank.start),
			Err(e) => log::warn!("pump {}: no move banked: {e}", bank.id),
		}
	}

	/// Swap on Ready (the swap row or its window, at one click: the game's `Lc.Swap`): the move becomes the option taken
	/// from those shown, or with `Take::Back` the move it replaced, as it was; its options are asked again, a swapped
	/// move's its way back alone, so the two can be traded until Start. An app pump's moves are read again, its riders
	/// and the moves Swap changed with them (a match pump's rider pays the same owed row, slice 61); a catch-up's swapped
	/// move alone (`pump_swapped`), which pays the same owed row.
	fn pump_swap(&mut self, take: Take) -> Result<(), String> {
		let run = self
			.pumps
			.run
			.as_mut()
			.filter(|r| {
				r.stage == Stage::Ready && r.busy == 0 && r.swaps.as_ref().is_some_and(|o| o.get(take).is_some())
			})
			.ok_or("no Swap now")?;
		run.swaps = None;
		let index = run.shown().map(|m| m.slot).ok_or("no move")?;
		let at = unix_s(lan::now_ms());
		if run.what.app() {
			self.pump_call(
				move |e| {
					e.swap(index, take)?;
					Ok((e.pump()?, e.moves()?, e.preview(at)?))
				},
				move |h, (pump, moves, previews)| {
					h.pump_moves(pump, moves, previews, |i| i == index);
					h.pump_swaps();
				},
			);
		} else {
			self.pump_call(
				move |e| {
					let took = e.swap(index, take)?;
					Ok((took, if took { e.dosed(index)? } else { None }))
				},
				move |h, (took, dosed)| h.pump_swapped(index, took, dosed),
			);
		}
		self.pump_publish();
		Ok(())
	}

	/// A catch-up's move at slot `index` as Swap left it, where the engine took the option (`took`; else the move
	/// stays): the move as the engine now doses it (`dosed`), its slot, the move as it runs and its cardio rung in place
	/// of the one shown, the slot marked swapped for its payment's `ran` as the engine has it (`Dosed::swapped`: none
	/// again once its way back was taken; a way back to an owed backup's stand-in still pays as it:
	/// `Run::ran_instead`), and a move it replaces that runs by its time kept as it showed (`Run::unswapped`); none on
	/// the way back to such a move, which shows again as it was, with no slot and no `ran`. Then its options are asked
	/// again and its name read.
	fn pump_swapped(&mut self, index: usize, took: bool, dosed: Option<Dosed>) {
		let Some(run) = self.pumps.run.as_mut() else { return };
		let Some(pos) = run.moves.iter().position(|m| m.slot == index) else { return };
		match (took, dosed) {
			(_, Some(Dosed { slot, planned, swapped, .. })) => {
				if swapped {
					run.swapped.insert(index);
				} else {
					run.swapped.remove(&index);
				}
				if run.slots.get(index).is_some_and(Option::is_none) {
					let rung = run.rungs.get(pos).copied().unwrap_or(1);
					run.unswapped.insert(index, (run.moves[pos].clone(), rung));
				}
				if let Some(rung) = run.rungs.get_mut(pos) {
					*rung = slot.rung;
				}
				if let Some(s) = run.slots.get_mut(index) {
					*s = Some(slot);
				}
				run.moves[pos] = planned;
			}
			(true, None) => {
				if let Some((shown, rung)) = run.unswapped.remove(&index) {
					run.swapped.remove(&index);
					if let Some(r) = run.rungs.get_mut(pos) {
						*r = rung;
					}
					if let Some(s) = run.slots.get_mut(index) {
						*s = None;
					}
					run.moves[pos] = shown;
				}
			}
			(false, None) => {}
		}
		self.pump_swaps();
		self.pump_read_aloud();
		self.pump_publish();
	}

	/// An app pump's moves as the engine has them now (`pump`, after Swap or the Ready view's Pain), a match pump's
	/// riders and the moves Swap changed with them (slice 61: a swapped rider pays its owed row as the move it ran,
	/// `Run::ran_instead`), the shown pose's day note read again. A move's preview stays as it stood when the move was
	/// planned (the phones' `PlannedMove` preview): only the slots `planned` names take the new one, so a move recorded
	/// earlier in the pump still says "+1 Strength if you beat 12".
	fn pump_moves(
		&mut self,
		pump: Pump,
		moves: Vec<PlannedMove>,
		mut previews: Vec<Preview>,
		planned: impl Fn(usize) -> bool,
	) {
		let Some(run) = self.pumps.run.as_mut() else { return };
		for (i, p) in previews.iter_mut().enumerate() {
			if let Some(old) = run.previews.get(i).filter(|_| !planned(i)) {
				p.clone_from(old);
			}
		}
		run.ride(&pump.riders);
		// riders a refit brought in (a setup change let an owed move fit) pay under an id of their own, as those of its
		// first composition do: the pump's own id is its Bank all's, which `Session::catch_up` takes as paid
		if run.riders_id.is_none() && !run.owed.is_empty() {
			run.riders_id = Some(new_id());
		}
		run.swapped = pump.swapped.into_iter().collect();
		run.rungs = pump.slots.iter().map(|s| s.rung).collect();
		(run.slots, run.moves, run.previews) = (pump.slots.into_iter().map(Some).collect(), moves, previews);
		// read again for what is on its Ready view now: a pain flag can hold the climb a same move counted toward
		run.ready_toward = None;
		self.pump_ready_days();
		self.pump_read_aloud();
		self.pump_publish();
	}

	/// The Ready view's Pain (Pause's Something hurts, on every pump's Ready view as on the game's): the joints the
	/// player named flagged, their rows kept whatever is on screen when the engine answers; an app pump then composed
	/// again or refit (`pump_refit`), a catch-up resting the moves they load (`pump_rest_pained`).
	fn pump_pain(&mut self, joints: Vec<u8>) -> Result<(), String> {
		let run = self.pumps.run.as_ref().filter(|r| r.stage == Stage::Ready).ok_or("no Pain now")?;
		let app = run.what.app();
		if joints.is_empty() {
			self.pump_publish();
		} else if app {
			self.pump_refit(Refit::Pain(joints));
		} else {
			self.pump_rest_pained(joints);
		}
		Ok(())
	}

	/// An app pump waits on its Ready view (`desk_pump_refits`): a setup change reaches it at once (`pump_setup_refit`).
	pub(crate) fn pump_refits(&self) -> bool {
		self.pumps.run.as_ref().is_some_and(|r| r.stage == Stage::Ready && r.what.app())
	}

	/// What the pumps' filters read changed in Your setup (`setup_refilter`; the game's refilter on its "Settings"
	/// message): the profile's (`profile`), or the pain flags, which the engine holds already when this is asked, so the
	/// pump moves only where they differ from those it knew (`Pumps::flags`). An app pump on its Ready view; a
	/// catch-up on its Ready view rests the moves the flags now load, as its Something hurts does (`pump_rest_pained`),
	/// and keeps its moves through a profile's change; a pump past its Ready view keeps its moves (Your setup says so:
	/// its `pump_on`).
	pub(crate) fn pump_setup_refit(&mut self, profile: bool) {
		if self.pump_refits() {
			self.pump_refit(if profile { Refit::Profile } else { Refit::Flags(self.pumps.flags.clone()) });
		} else if !profile && self.pumps.run.as_ref().is_some_and(|r| r.stage == Stage::Ready) {
			self.pump_rest_pained(Vec::new());
		}
	}

	/// An app pump on its Ready view after `refit` (the game's refilter, Events.lua `refilter`; a match pump's riders
	/// too): before any move ended the pump is composed again, after one, a move banked too short among them
	/// (`pump_short`), the rest keeps what the filters allow now (a rider they refuse leaves it to wait, slice 61; the
	/// engine's `Pump:afterPain`); a pump with no move left from the one shown on ends. Pain's joints are flagged first,
	/// their rows kept whatever is on screen when the engine answers, and nothing else changes where its flags were
	/// set already.
	fn pump_refit(&mut self, refit: Refit) {
		let Some(run) = self.pumps.run.as_ref().filter(|r| r.stage == Stage::Ready && r.what.app()) else { return };
		let (from, recorded) = (run.shown().map_or(0, |m| m.slot), run.current > 0);
		let at = unix_s(lan::now_ms());
		let Some(id) = self.pump_busy() else { return };
		self.engine_call(
			move |e| {
				let (pain, flags, changed) = match refit {
					Refit::Pain(joints) => {
						let before = e.pain_flags()?;
						let pain = e.pain_rows(&joints, true, at)?;
						let flags = e.pain_flags()?;
						let changed = flags != before;
						(pain, flags, changed)
					}
					Refit::Profile => (Vec::new(), e.pain_flags()?, true),
					Refit::Flags(known) => {
						let flags = e.pain_flags()?;
						let changed = flags != known;
						(Vec::new(), flags, changed)
					}
				};
				if !changed {
					return Ok((pain, flags, None));
				}
				e.after_pain(from, recorded, at)?;
				let pump = e.pump()?;
				let moves = if pump.slots.is_empty() { Vec::new() } else { e.moves()? };
				let previews = if pump.slots.is_empty() { Vec::new() } else { e.preview(at)? };
				let swaps = if pump.slots.len() > from { e.swap_options(from, at)? } else { None };
				Ok((pain, flags, Some((pump, moves, previews, swaps))))
			},
			move |h, answer| {
				let shown = h.pump_answered(&id);
				let (pain, flags, planned) = match answer {
					Ok(answer) => answer,
					Err(e) => {
						log::warn!("pump {id}: the moves again for the filters: {e}");
						return h.pump_publish();
					}
				};
				h.pump_keep(&id, None, &rows(&pain));
				h.pumps.flags = flags;
				let Some((pump, moves, previews, swaps)) = planned.filter(|_| shown) else { return h.pump_publish() };
				// none left from the one shown on (a rider the filters refuse leaves to wait): the pump ends
				let ends = moves.len() <= h.pumps.run.as_ref().map_or(0, |r| r.current);
				h.pump_moves(pump, moves, previews, |i| !recorded || i >= from);
				if ends {
					return h.pump_finish(false);
				}
				if let Some(run) = h.pumps.run.as_mut() {
					run.swaps = swaps.filter(view::any_option);
				}
				h.pump_publish();
			},
		);
		self.pump_publish();
	}

	/// A catch-up's Pain (the game's Something hurts, on every pump; since 2026-11-05), or the flags Your setup set over
	/// it (`joints` empty): the joints flagged, their rows kept whatever is on screen when the engine answers, then every
	/// move from the one on Ready on whose move loads a joint flagged now leaves the catch-up, its owed row left
	/// unfinished as a catch-up's start leaves such a move (`pump_payable`) and its other moves keeping their slots as
	/// its Skip leaves them (`Run::leave_catch_up`), nothing written for it. Where the move on Ready left, the next
	/// move's Ready view shows, or the catch-up ends as Bank the rest ends it; where it stays, only Swap's options are
	/// asked again, since the flags changed what they may offer.
	fn pump_rest_pained(&mut self, joints: Vec<u8>) {
		let at = unix_s(lan::now_ms());
		let Some(id) = self.pump_busy() else { return };
		self.engine_call(
			move |e| {
				let pain = if joints.is_empty() { Vec::new() } else { e.pain_rows(&joints, true, at)? };
				Ok((pain, e.pain_flags()?))
			},
			move |h, answer| {
				let shown = h.pump_answered(&id);
				let (pain, flags) = match answer {
					Ok(answer) => answer,
					Err(e) => {
						log::warn!("pump {id}: the pain flags: {e}");
						return h.pump_publish();
					}
				};
				h.pump_keep(&id, None, &rows(&pain));
				h.pumps.flags = flags;
				let Some(run) = h.pumps.run.as_mut().filter(|r| shown && r.stage == Stage::Ready) else {
					return h.pump_publish();
				};
				let hurts = |m: &PlannedMove| facts().move_joints(m.r#move).iter().any(|j| h.pumps.flags.contains(j));
				let rested: Vec<usize> = (run.current..run.moves.len()).filter(|&i| hurts(&run.moves[i])).collect();
				let shown_left = rested.first() == Some(&run.current);
				if !rested.is_empty() {
					log::info!("pump {id}: {} moves rest for the pain flagged", rested.len());
				}
				for &i in rested.iter().rev() {
					run.leave_catch_up(i);
				}
				if shown_left {
					h.pump_after_leaving();
				} else {
					h.pump_swaps();
					h.pump_publish();
				}
			},
		);
		self.pump_publish();
	}

	/// An end that waits for the engine (`Held`, done by `pump_unhold` once it answers): while it checks or judges a
	/// rider's count (felt unwell is kept), a rider's hold left at "Could you have held longer?" (its set judged first,
	/// `pump_unanswered`), and a rider Swap took to its easier version left on its count step, paid done as every such
	/// move (the game's `Lc.OweOpen` writes it done) with the step down the engine says it makes (`pump_empty`, since
	/// 2026-10-26). A rider is a catch-up's move or a match pump's (slice 61). False: the end goes on now.
	fn pump_stop_held(&mut self, why: Stop) -> bool {
		if let Some(run) = self.pumps.run.as_mut().filter(|r| r.saving) {
			if run.held.stop != Some(Stop::Unwell) {
				run.held.stop = Some(why);
			}
			return true;
		}
		if let Some(run) = self.pumps.run.as_mut().filter(|r| r.on_rider() && matches!(r.stage, Stage::Longer { .. })) {
			run.held.stop = Some(why);
			self.pump_unanswered(None);
			return true;
		}
		if let Some(run) = self.pumps.run.as_mut().filter(|r| r.on_rider() && r.stage == Stage::Counting)
			&& let Some(slot) = run.shown().map(|m| m.slot)
			&& !run.recorded.contains_key(&slot)
			&& run.ran_instead(slot).is_some_and(|r| r.easier)
		{
			run.recorded.insert(slot, Recorded::Done);
			run.held.stop = Some(why);
			self.pump_empty();
			return true;
		}
		false
	}

	/// Ends the pump before its end (End the pump here, Leave, QuestPump's quit, the game's offer, felt unwell): the
	/// moves run so far count; an app pump's own rep move or hold whose count was never saved writes nothing (a
	/// rider's is paid done with no set, as the game records a move that waits on its count done without a count when
	/// its pump ends: Events.lua `Lc.OweOpen`), an app pump's own counted set left at its count and form or at "Did you
	/// do the whole move?" is written done with no count and Form slipped, as the game's logout there writes it
	/// (`Lc.OweOpen`; its form never answered, it counts no clean day: 2026-10-19, the program's double-check), a pose
	/// waiting for "How close to the goal pose?" is written with no answer (`pump_unanswered`), and a steady move Keep
	/// going ran on past its plan is saved as its Done saves it, with its answer (`pump_kept`; slice 52's double-check:
	/// it ran its plan, as a move the plan ended would have been kept), where a plain running move of an app pump's own
	/// writes no row. Felt unwell locks every pump until the next game day at once (`Session::lock_after`, the lock its
	/// row sets): an app pump writes its own running timed move's or its waiting pose's row with the unwell reason,
	/// else the pump banked with it (a running rider's time counts as ran, its riders done paid with no lock of their
	/// own); a catch-up's stop goes with its moves. A match pump's Bank the rest on a rider (`Stop::Rest`) ends it as
	/// done with its Result (slice 61, the game's `Lc.Finish("rest")`). A pump left with no move run, at Bank the rest too
	/// (its own moves all ended too short: the game's `Lc.Bank` closes it, `Lc.Finish("banked")`), and a catch-up left at
	/// Bank the rest with its moves done paid, leave the screen with no calm card (Rapha,
	/// 2026-10-28: Bank the rest says it already). While the engine checks or judges a rider's count (or a kept move's
	/// set) the end waits until the count is a set or stays on its step (`Held`, `pump_stop_held`).
	fn pump_stop(&mut self, why: Stop) -> Result<(), String> {
		if self.pump_stop_held(why) {
			return Ok(());
		}
		let now = lan::now_ms();
		// a move the computer slept through counts none of it
		self.pump_slept(now);
		// a move whose count or seconds were saved is written as its pour's landing would (slice 65); only Back drops it
		let poured = self.pump_pour_left((why == Stop::Unwell).then_some(UNWELL));
		let run = self.pumps.run.as_mut().filter(|r| r.running()).ok_or("no pump runs")?;
		// an app pump's counted set left at its ask: done with no count and Form slipped, once it ended (below); felt
		// unwell writes its own row (a stop at the ask, else the pump banked)
		let slipped = why != Stop::Unwell
			&& !run.on_rider()
			&& matches!(run.stage, Stage::Counting | Stage::Asking { .. })
			&& run.shown().is_some_and(|m| m.counted);
		// a count never given: an app pump's own move writes nothing
		if run.stage == Stage::Counting
			&& !run.on_rider()
			&& !slipped
			&& !poured
			&& let Some(slot) = run.shown().map(|m| m.slot)
		{
			run.ran.remove(&slot);
		}
		// the running move ends now (one whose run is over, a kept move's set judged meanwhile, ended already); a timed
		// one asked about ended at its Done, as stopped there (the answer never came)
		let ending = match run.stage {
			Stage::Running { start } if run.side.is_some() => Some((run::secs(now - start), now)),
			Stage::Asking { .. } => run.asked.map(|(_, elapsed, end)| (elapsed, end)),
			_ => None,
		};
		// a steady move Keep going ran on past its plan ends as its Done ends it (`pump_kept`); felt unwell writes its
		// own row (below)
		let kept = run.kept_to().filter(|_| why != Stop::Unwell);
		let posed = poured || self.pump_unanswered((why == Stop::Unwell).then_some(UNWELL));
		let mut timed = None;
		if let Some((elapsed, end)) = ending {
			self.pump_finish_move(elapsed, end, None, false);
			let run = self.pumps.run.as_mut().ok_or("no pump")?;
			if let Some(m) = run.shown().cloned().filter(|m| !run.rider(m.slot)) {
				if m.kind == kind::REP || m.kind == kind::HOLD {
					run.ran.remove(&m.slot); // its count never comes
				} else {
					timed = run.ran.get(&m.slot).map(|&s| (m.slot, timed_result(&m, s)));
				}
			}
			// a rider Swap took to its easier version, ended running, is paid done as it ran (`pump_paid`) with the
			// step down the engine says it makes (`pump_empty`), as on its count step (`pump_stop_held`); the end waits
			// for it. Only a rep move or a hold, which ask a count and alone carry a step down (Engine.lua
			// `easierSlot`'s `down`: a strength or hold ladder's attempt); a timed one is paid done as it ran
			if let Some(run) = self.pumps.run.as_mut().filter(|r| r.on_rider())
				&& let Some(slot) = run.shown().filter(|m| m.kind == kind::REP || m.kind == kind::HOLD).map(|m| m.slot)
				&& run.ran.contains_key(&slot)
				&& !run.recorded.contains_key(&slot)
				&& run.ran_instead(slot).is_some_and(|r| r.easier)
			{
				run.recorded.insert(slot, Recorded::Done);
				run.stage = Stage::Counting;
				run.held.stop = Some(why);
				self.pump_empty();
				return Ok(());
			}
		}
		if let Some(to) = kept
			&& let Some(run) = self.pumps.run.as_ref()
			&& let Some(m) = run.shown().cloned()
			&& let Some(&seconds) = run.ran.get(&m.slot)
			&& self.pump_kept(&m, seconds, to, false)
			&& let Some(run) = self.pumps.run.as_mut().filter(|r| r.saving)
		{
			// a rider's set judged first, the end held until it is
			run.held.stop = Some(why);
			return Ok(());
		}
		if slipped
			&& let Some(run) = self.pumps.run.as_ref()
			&& let Some(&elapsed) = run.shown().and_then(|m| run.ran.get(&m.slot))
		{
			let result = MoveResult { value: 0, elapsed, answer: Some(answer::SLIPPED), ..MoveResult::default() };
			self.pump_write(result, Recorded::Counted(0, answer::SLIPPED));
		}
		let run = self.pumps.run.as_mut().ok_or("no pump")?;
		let leaves = matches!(why, Stop::Left | Stop::Rest) && (run.ran.is_empty() || !run.what.app());
		match why {
			Stop::Unwell => {
				run.unwell = true;
				let (app, at) = (run.what.app(), unix_s(now));
				self.session.lock_after(u32::try_from(at.max(0)).unwrap_or(u32::MAX));
				// a waiting pose's own row locks
				if app && !posed {
					self.pump_write_rows(
						move |e| match timed {
							Some((slot, mut result)) => {
								result.reason = Some(UNWELL);
								e.finish(slot, result, at)
							}
							None => e.stop(at).map(|rows| Finished { rows, day: None, kindled: None, lit: false }),
						},
						|_, _, _| {},
					);
				}
			}
			Stop::Left | Stop::Game => {
				if why == Stop::Game || !run.ran.is_empty() {
					run.cut = Some(if why == Stop::Game { Cut::Game } else { Cut::Left });
				}
			}
			// done, its riders left unfinished
			Stop::Rest => {}
		}
		self.hush();
		self.pump_finish(true);
		// an expedition's rounds done before it end on their Result
		if leaves && !self.expedition_kept() {
			self.pump_dismiss();
		}
		Ok(())
	}

	/// The pump is over: a running move ends as far as it ran (unless `stopped` ended it already), the computer sleeps
	/// again, Big view leaves the full screen, a catch-up pays what it ran (the engine dropping the counts it judged
	/// first, `pump_paid`), and an app pump's Result asks the engine what it paid; a match pump's riders done are paid
	/// once the Result's reads are asked (slice 61), so the engine still reads the app pump's own as it answers.
	fn pump_finish(&mut self, stopped: bool) {
		let now = lan::now_ms();
		if !stopped
			&& let Some(run) = self.pumps.run.as_ref().filter(|r| r.side.is_some())
			&& let Stage::Running { start } = run.stage
		{
			self.pump_finish_move(run::secs(now - start), now, None, false);
		}
		let Some(run) = self.pumps.run.as_mut().filter(|r| r.running()) else { return };
		run.stage = Stage::Done;
		run.done_at = Some(now);
		run.swaps = None;
		run.count.clear();
		if run.full {
			run.full = false;
			self.pump_full_screen(false);
		}
		let Some(run) = self.pumps.run.as_ref() else { return };
		if run.what.app() {
			let riders = !run.owed.is_empty();
			self.expedition_round_ends(stopped);
			if riders {
				self.pump_lights();
			}
			self.pump_result();
			if riders {
				self.pump_paid();
			}
		} else {
			// the embers its cardio moves light, judged before the payment lets its judged sets go (`pump_settle`)
			self.pump_lights();
			self.pump_paid();
			// a catch-up's Result plays the same reveal, its rows known once its payment is (`view::caught_up`)
			if let Some(run) = self.pumps.run.as_mut().filter(|r| r.resulted()) {
				run.reveal = Some(view::Reveal::plan(now, &view::caught_up(run), true));
			}
		}
		self.pump_publish();
	}

	/// A rider's cardio moves that light an ember (a catch-up's, a match pump's riders': `Engine::lights`, each as its
	/// payment's row reads: a set as judged from its run, its count and answer or its pace; any other move done in the
	/// seconds it ran, stopped short or not), counted on its Result's Cardio today once the engine answers, beside the
	/// ones a match pump's own moves lit (`Finished::lit`).
	fn pump_lights(&mut self) {
		let Some(run) = self.pumps.run.as_ref() else { return };
		let results: Vec<(usize, MoveResult)> = run
			.moves
			.iter()
			.filter(|m| m.kind == kind::CARDIO && run.rider(m.slot))
			.filter_map(|m| {
				let ran = *run.ran.get(&m.slot)?;
				let result = match run.recorded.get(&m.slot) {
					Some(&Recorded::Counted(n, said)) => {
						MoveResult { value: n, elapsed: ran, answer: Some(said), ..MoveResult::default() }
					}
					_ => MoveResult { pace: run.answers.get(&m.slot).copied(), ..timed_result(m, ran) },
				};
				Some((m.slot, result))
			})
			.collect();
		if results.is_empty() {
			return;
		}
		let id = run.id.clone();
		let at = unix_s(lan::now_ms());
		self.engine_call(
			move |e| {
				results.into_iter().map(|(slot, result)| e.lights(slot, result, at)).collect::<Result<Vec<_>, _>>()
			},
			move |h, answer| match answer {
				Ok(lit) => {
					if let Some(run) = h.pumps.run.as_mut().filter(|r| r.id == id) {
						run.embers_lit += i64::try_from(lit.iter().filter(|&&l| l).count()).unwrap_or(0);
						h.pump_publish();
					}
				}
				Err(e) => log::warn!("pump {id}: the engine judged no ember: {e}"),
			},
		);
	}

	/// The riders done paid in the game's store (`Session::catch_up`, the computer's source): a catch-up's moves under
	/// its id with its felt-unwell stop, a match pump's riders under their own id (slice 61, `Run::pay_id`) with none,
	/// since its own stop locks; each naming the owed row it was dosed from (its move and that row's time), the move run
	/// in its place (`ran`: Swap's since 2026-10-26, an owed backup's stand-in since 2026-10-29, `Run::ran_instead`) and
	/// each count saved as its move's set. While a World of Warcraft client with QuestPump runs it asks these moves
	/// itself and none is paid here, and a reset since the pump began or the lock refuses them, but a catch-up's stop is
	/// written all the same (its row alone), so the lock reaches the game, the mirror and every device. The engine lets
	/// the counts it judged go first (`pump_settle`), so the rows written for them merge in once (`Helper::apply`).
	fn pump_paid(&mut self) {
		let now = lan::now_ms();
		let wow = self.wow_with_addon();
		let Some(run) = self.pumps.run.as_mut() else { return };
		let ran: Vec<CatchupMove> = run
			.moves
			.iter()
			.filter_map(|m| {
				let owed = run.owed.get(&m.slot)?;
				let seconds = *run.ran.get(&m.slot)?;
				Some(CatchupMove {
					seconds: u16::try_from(seconds.max(0)).unwrap_or(u16::MAX),
					move_id: owed.move_id,
					ts: owed.ts,
					set: run.sets.get(&m.slot).cloned(),
					ran: run.ran_instead(m.slot),
					kind: None,
					ladder: None,
					rung: None,
					allout: false,
				})
			})
			.collect();
		let some_ran = !ran.is_empty();
		if wow && some_ran {
			run.paid = Some(Payment::GameRuns);
		}
		let moves = if wow { Vec::new() } else { ran };
		// a match pump's own stop locks (`pump_stop`): its riders' payment carries none
		let unwell = run.unwell && !run.what.app();
		if moves.is_empty() && !unwell {
			self.pump_settle();
			return;
		}
		let total: i64 = moves.iter().map(|m| i64::from(m.seconds)).sum();
		let first = if run.what.app() { run.riders_start } else { run.pump_start };
		let start = first.map_or_else(|| unix_s(now) - total, unix_s);
		let start_s = u32::try_from(start.max(0)).unwrap_or(u32::MAX);
		let unwell_at = unwell.then(|| u32::try_from(unix_s(now).max(start)).unwrap_or(u32::MAX));
		let pump = CatchupPump {
			source: Source::Computer,
			id: run.pay_id().to_owned(),
			start: start_s,
			moves,
			measured: None,
			unwell_at,
		};
		// its felt-unwell stop is written even when the moves are refused (`Session::catch_up`)
		let (paid, fx) = self.session.catch_up(&pump, &runtime::now());
		self.pump_settle();
		self.apply(fx);
		// after the merge of the rows it wrote, the steps they left mastered with no row (slice 42's fix round)
		self.engine_sweep();
		let paid = match paid {
			Ok(written) => {
				log::info!("pump {}: the riders' payment wrote {} rows", pump.id, written.len());
				Payment::Paid
			}
			Err(e) => {
				log::warn!("pump {}: the riders' payment paid nothing: {e}", pump.id);
				match e {
					CatchupError::AlreadyPaid(_) => Payment::Paid,
					CatchupError::BeforeReset => Payment::Reset,
					CatchupError::Locked(_) => Payment::Locked,
					// a start past this computer's own clock (`Untimely`) only a clock set back can give: nothing
					// written, as when nothing was owed
					CatchupError::NothingOwed | CatchupError::Untimely => Payment::Unpaid,
				}
			}
		};
		// the moves' payment, where moves ran and this computer paid them, and what the streak's board says they reached
		let after = (paid == Payment::Paid).then(|| self.board_now());
		if let Some(run) = self.pumps.run.as_mut().filter(|_| some_ran && !wow) {
			run.paid = Some(paid);
			if let Some(after) = &after {
				run.reached = view::Reached::of(run.streak_before.as_ref(), after);
			}
		}
	}

	/// The riders' payment: the counts the engine judged leave its state (`Engine::settle`), asked just before the merge
	/// of the rows the payment wrote (`Helper::apply` asks it, and the Book's and the Log's reads after it), so each counts
	/// once and no read sees one twice.
	fn pump_settle(&self) {
		self.engine_call(Engine::settle, |_, answer| {
			if let Err(e) = answer {
				log::warn!("pump: the catch-up's counts stay in the engine's state until its next fold: {e}");
			}
		});
	}

	/// An app pump's Result: what the app's own sheet paid (its summary, the titles it holds now, a first record's next
	/// step, each move of its own done with a weight's next fresh set), read once the pump's rows are in, a match pump's
	/// riders done after its own moves with how they were paid (slice 61: their payment is made once these reads are
	/// asked, `pump_finish`), and the reveal's instants.
	fn pump_result(&mut self) {
		let Some(run) = self.pumps.run.as_ref() else { return };
		if run.unwell {
			return;
		}
		let at = unix_s(lan::now_ms());
		let own: Vec<PlannedMove> = run.moves.iter().filter(|m| !run.rider(m.slot)).cloned().collect();
		let firsts = view::firsts(&own, &run.recorded, |m| run.max_set(m));
		let weighed = view::weighed(&own, &run.recorded);
		// where Hero shows the game's sheet, its lifetime XP, on which the bar shows the pump's until the game pays it
		let game_xp = self.game_shown(&self.session.sheet(at)).map(|g| g.summary.xp);
		self.pump_call(
			move |e| {
				let summary = e.summary(at)?;
				let titles = e.titles()?;
				let embers = e.embers(at)?.today;
				let gained = summary["last"][0].as_i64().unwrap_or(0);
				let projected = match game_xp.filter(|_| gained > 0) {
					Some(xp) => match e.levels(&[xp, xp + gained])?.as_slice() {
						&[from, to] => Some((from, to)),
						_ => None,
					},
					None => None,
				};
				let goals = firsts
					.iter()
					.map(|&(slot, move_id, n)| Ok((slot, e.next_goal(move_id, n)?)))
					.collect::<Result<Vec<_>, EngineError>>()?;
				let mut nexts = Vec::new();
				for move_id in weighed {
					if let Some(next) = e.next_load(move_id, at)? {
						nexts.push((move_id, next));
					}
				}
				Ok((summary, titles, goals, nexts, embers, projected))
			},
			|h, (summary, titles, goals, nexts, embers, projected)| {
				let now = lan::now_ms();
				let game = h.game_shown(&h.session.sheet(unix_s(now)));
				// today's embers: the app's own fold's, or the game's sheet's where Hero shows it and it counted more,
				// never fewer than the count as it began and the embers its own moves lit
				let embers = embers.max(game.as_ref().map_or(0, |g| h.game_embers(g, unix_s(now))));
				// shown only where its own moves lit one
				let embers = h
					.pumps
					.run
					.as_ref()
					.filter(|r| r.embers_lit > 0)
					.map(|r| embers.max(r.embers_before + r.embers_lit));
				// Hero shows the app's own sheet: no game sheet, the strip's or the save's (`Helper::game_shown`)
				let hero_own = game.is_none();
				// else the streak's board says what the pump reached, its rows in (the game folds them at its next load)
				let after = h.board_now();
				// the "Cardio done today" toast's week: never fewer than Hero's vault card says (the game's sheet too)
				let card = h.vault_embers(game.as_ref(), unix_s(now));
				// the very first title the app's own sheet earned wears itself while none was ever chosen (the phones'
				// `wearFirstTitle` and the game's `Sheet.lua`, 2026-10-21's review)
				let first = h.pumps.run.as_ref().is_some_and(|r| r.titles_before.is_empty());
				if hero_own
					&& first && h.body.body.title.is_none()
					&& h.settings.title.is_none()
					&& let Some(key) = titles.first().and_then(|&code| facts().title(code)).map(|t| t.key.clone())
				{
					h.wear_title(Some(key));
				}
				let Some(run) = h.pumps.run.as_mut() else { return };
				let reached = view::Reached::of(run.streak_before.as_ref(), &after);
				let (riders, own): (Vec<PlannedMove>, Vec<PlannedMove>) =
					run.moves.iter().cloned().partition(|m| run.rider(m.slot));
				let earned = view::earned(
					&view::Paid {
						moves: &own,
						recorded: &run.recorded,
						ran: &run.ran,
						before: run.before.as_ref(),
						after: view::Sum::read(&summary).as_ref(),
						titles_before: &run.titles_before,
						titles_after: &titles,
						goals: &goals,
						beat: &run.beat,
						nexts: &nexts,
						hero_own,
						reached,
						written: &run.written,
						answers: &run.answers,
						days: &run.days,
						kindled: run.kindled.map(|week| week.max(card)),
						embers,
						projected,
						check: run.what.check().is_some(),
						checked: &run.checked,
						quest: matches!(&run.what, What::Free { kind } if kind == QUEST),
						riders: &riders,
						sets: &run.sets,
						payment: run.paid,
					},
					|m| run.max_set(m),
				);
				if matches!(run.what, What::Expedition { .. }) {
					h.expedition_round_earned(earned);
					return;
				}
				if run.resulted() {
					run.reveal = Some(view::Reveal::plan(now, &earned, true));
				}
				run.result = Some(earned);
				h.pump_publish();
			},
		);
	}

	/// The Result's reveal at its end (a click on it, or Reduce Motion from its first frame): one event, the turn's
	/// where it has not sounded, else `reveal` (`still`: the pump's one strongest), and nothing after.
	fn pump_land(&mut self, still: bool) {
		let Some(reveal) = self.pumps.run.as_mut().and_then(|r| r.reveal.as_mut()) else { return };
		if let Some(event) = reveal.land(lan::now_ms(), still) {
			self.cue(Cue::Event(event));
		}
		self.pump_publish();
	}

	/// Continue on the Result or a calm card: the pump leaves the screen and the window stays on the page under it, a
	/// pump the tray or the card started too (it handed the focus back and closed the window until 2026-10-25, which
	/// looked like a crash once the game it came from had closed; David: "I just pressed continue after a pump and it
	/// seems it crashed the app"). An app pump that ended done cues the QP 10 focus ask, which Hero raises at once where
	/// it is due (a match pump's riders rode in it since slice 61, so nothing goes on after it). A Result put away asks
	/// "Remind me once a day?" (`publish_reminders`), and the QP 10 ask waits until no pump is on screen or starting
	/// (`pump_focus_due`).
	fn pump_continue(&mut self) -> Result<(), String> {
		let run = self.pumps.run.as_mut().filter(|r| !r.running()).ok_or("no pump to close")?;
		// an orange or gold step the Result showed: its Legendary moment first, one a step, the voice naming the move;
		// never after a calm end (a felt-unwell stop, a pump left part way or cut), as the phones' `celebrates`
		if let Some((index, key)) =
			run.result.as_ref().filter(|_| run.resulted()).and_then(|r| r.legend_after(run.legend))
		{
			run.legend = Some(index);
			if let Some(key) = key {
				self.cue(Cue::Line(format!("legend.{key}")));
			}
			self.pump_publish();
			return Ok(());
		}
		let (done, resulted) = (run.ended_done(), run.resulted());
		self.pump_dismiss();
		if done {
			self.pumps.focus_due = true;
			self.home_publish();
		}
		// a Result put away: "Remind me once a day?" once (`desk::reminders`)
		if resulted {
			self.reminder_offer_due();
		}
		Ok(())
	}

	/// The QP 10 focus ask's cue, taken once no pump is on screen or starting: an app pump that ended done was just put
	/// away (`pump_continue`); Hero's next look asks where the ask is due then, or never for that pump.
	pub(crate) fn pump_focus_due(&mut self) -> bool {
		self.pumps.run.is_none() && !self.pumps.starting && std::mem::take(&mut self.pumps.focus_due)
	}

	/// The pump leaves the screen.
	fn pump_dismiss(&mut self) {
		self.hush();
		self.pumps.trip = None;
		if let Some(run) = self.pumps.run.take()
			&& run.full
		{
			self.pump_full_screen(false);
		}
		self.pump_publish();
	}

	/// "Leave the pump?"'s Leave (the window's close or QuestPump's quit while a pump runs): the pump ends as End the
	/// pump here ends it and leaves the screen, then the window closes or QuestPump quits (and starts the moved copy
	/// the ask carried, `ui::quit_for`); while the engine checks or judges a rider's count, once the count is a set or
	/// stays on its step (`Held`).
	fn pump_leave(&mut self, quit: bool) -> Result<(), String> {
		if self.pumps.run.as_ref().is_some_and(|r| r.running() && !r.saving) {
			self.pump_stop(Stop::Left)?;
		}
		// a count the engine checks or judges now (a kept steady move's set too, which that end judges): the Leave once
		// it is a set or stays on its step
		if let Some(run) = self.pumps.run.as_mut().filter(|r| r.saving) {
			run.held.leave = Some(quit);
			return Ok(());
		}
		self.pump_dismiss();
		let mut start = None;
		ui::update(&self.app, |v| start = v.shell.leave.take().and_then(|l| l.start));
		let handle = self.app.clone();
		self.app
			.run_on_main_thread(move || if quit { ui::quit_for(&handle, start) } else { windows::close_main(&handle) })
			.map_err(|e| e.to_string())
	}

	/// QuestPump quits with a pump on screen and nothing asked first (the system's own quit: a logout, a restart, a
	/// shutdown): the pump ends as Leave ends it, so what it ran is written before the engine's calls settle
	/// (`Helper::quit`), and a match's pump banks its own moves left as the game's logout owes an open quest pump's
	/// (Events.lua `Lc.OweOpen`; slice 58): every move of it with no row from the one on screen on, the one the end
	/// writes aside (a counted set kept Form slipped); a running move, or one at its count, an app pump's end writes
	/// nothing of, banks, and its riders done are paid (slice 61). A catch-up's moves left stay unfinished, as its Leave
	/// leaves them.
	pub(crate) fn pump_quit(&mut self) {
		let Some(run) = self.pumps.run.as_ref().filter(|r| r.running()) else { return };
		log::info!("pump: QuestPump quits; the pump here ends");
		let banks = (run.what.app() && run.offer.is_some()).then(|| (run.id.clone(), run.bank()));
		if let Err(e) = self.pump_stop(Stop::Left) {
			log::warn!("pump: the pump did not end at the quit: {e}");
		}
		let Some((id, (mut bank, slots))) = banks else { return };
		// the pump as the end left it (none where it left the screen with nothing done)
		let run = self.pumps.run.as_ref().filter(|r| r.id == id);
		let written = |slot: usize| run.is_some_and(|r| r.recorded.contains_key(&slot));
		bank.moves = bank.moves.into_iter().zip(slots).filter(|&(_, slot)| !written(slot)).map(|(m, _)| m).collect();
		if !bank.moves.is_empty() {
			self.pump_write_bank(&bank);
		}
	}

	/// A recall: the match offer `offer` was taken back (`Helper::offer_recalled`), or, with none, a watched game's new
	/// run or a new match began (`Helper::outside_recalled`). What it, or with none anything, opened from outside the
	/// window (the tray, the card, the auto-open's `start_offer`, and a start from the window one of them opened) ends
	/// while nothing ran, nothing written, and one the player opened from the page stays: the Start a pump window
	/// closes, the start waiting for the place goes with the place window, the pump the engine composes shows nowhere
	/// as it lands (an app pump's rows kept as the engine wrote them), or the pump waiting on its Ready view untouched
	/// ends, as a ladder started over elsewhere ends one (`pump_started_over`), except that a match's pump a new run or
	/// match ends banks its moves first (`pump_bank`'s rows; David, 2026-10-27: a League pump left at Ready "was gone"
	/// after the next game), as the game owes an open quest pump's when the player plays on (Events.lua `Lc.OweOpen`),
	/// while its own offer's recall (the match goes on) writes nothing. A start that went nowhere counts on the topic, as
	/// a gate's does (`pump_refresh`). What ended, if anything.
	pub(crate) fn pump_recalled(&mut self, offer: Option<&str>) -> Option<Recalled> {
		let ours = |from: Option<&str>| offer.is_none_or(|id| from == Some(id));
		let p = &mut self.pumps;
		let ended = if p.choose.as_ref().is_some_and(|(c, out)| *out && ours(c.offer.as_deref())) {
			p.choose = None;
			Recalled::Choose
		} else if p.waiting.as_ref().is_some_and(|w| w.outside && ours(w.start.offer())) {
			p.waiting = None;
			p.nowhere();
			Recalled::Waiting
		} else if p.starting && p.composing.as_ref().is_some_and(|c| c.outside && ours(c.offer.as_deref())) {
			p.composing = None;
			Recalled::Composing
		} else {
			let opened = p.run.as_ref().is_some_and(|r| r.outside && ours(r.offer.as_deref()) && r.untouched());
			if !opened || p.starting {
				return None;
			}
			let banks = self.pumps.run.as_ref().filter(|r| offer.is_none() && self.pump_bank_way(r) == Some(Bank::It));
			if let Some((bank, _)) = banks.map(Run::bank) {
				self.pump_write_bank(&bank);
			}
			return self.pump_stop(Stop::Left).is_ok().then_some(Recalled::Ready);
		};
		self.pump_publish();
		Some(ended)
	}

	/// A pump the game offered on the strip: the pump here ends, as it ends one on the wrist (one pump at a time).
	pub(crate) fn pump_game_offered(&mut self) {
		if self.pumps.run.as_ref().is_some_and(Run::running) {
			log::info!("pump: the game offered a pump; the pump here ends");
			let _ = self.pump_stop(Stop::Game);
		}
		self.expedition_gate_closed();
	}

	/// The gates, the owed moves and the flags may have changed (the session's effects, the game's tick): the topic and
	/// the tray again, and the pain flags asked of the engine again once the stores took rows (or a reset) since the
	/// last ask, as soon as it can answer.
	pub(crate) fn pump_refresh(&mut self) {
		// a World of Warcraft client with QuestPump started: the riders here pay nothing now, so a catch-up ends at once
		// as the game's own pump ends one, its moves left owed for the game to ask, and so does a match pump on a rider
		// or past one (slice 61); one still on its own moves goes on without its riders not done
		if self.wow_with_addon()
			&& let Some(run) = self.pumps.run.as_ref().filter(|r| r.running() && !r.owed.is_empty())
		{
			let reached = run.moves.iter().take(run.current + 1).any(|m| run.rider(m.slot));
			if !run.what.app() || reached {
				log::info!("pump: the game runs; the pump here ends, its riders left for the game");
				let _ = self.pump_stop(Stop::Game);
			} else {
				self.pump_leave_riders();
			}
		}
		// the lock or the game closes the Start a pump window and drops a start waiting for the place, which closes the
		// place window: their Start or Continue would lead nowhere, as Pump home behind them says why; a fold's gate
		// lasts a second or two and leaves them be (a Start pressed in it is refused, and pressed again once it has
		// landed starts)
		let closed = |gate: Option<Gate>| matches!(gate, Some(Gate::Lock | Gate::Game));
		if closed(self.pump_gate()) {
			self.pumps.choose = None;
			self.expedition_gate_closed();
		}
		if self.pumps.waiting.as_ref().is_some_and(|w| closed(self.start_gate(&w.start))) {
			self.pumps.waiting = None;
			self.pumps.nowhere();
		}
		let read = Some(self.store_mark());
		if read != self.pumps.read {
			let started_over = self.started_over_since(self.pumps.read.as_ref());
			self.pumps.read = read;
			self.pumps.stale = true;
			if started_over {
				self.pump_started_over();
			}
		}
		if self.pumps.stale && self.engine_ready() {
			self.pumps.stale = false;
			self.engine_call(Engine::pain_flags, |h, answer| match answer {
				Ok(flags) => {
					h.pumps.flags = flags;
					h.pump_publish();
				}
				Err(e) => log::warn!("pump: the pain flags: {e}"),
			});
		}
		self.pump_publish();
	}

	/// A match pump still on its own moves as a World of Warcraft client with QuestPump starts (slice 61): its riders,
	/// none done yet, leave it unpaid, the game asking them (on the engine too, `Engine::skip`, last first, so every
	/// later read of the pump keeps them out), and its own moves go on.
	fn pump_leave_riders(&mut self) {
		let Some(run) = self.pumps.run.as_mut() else { return };
		let slots: Vec<usize> = run.owed.keys().rev().copied().collect();
		log::info!("pump {}: the game runs; its {} riders leave it", run.id, slots.len());
		let id = run.id.clone();
		for &slot in &slots {
			run.drop_move(slot);
		}
		for slot in slots {
			self.pump_engine_skip(&id, slot);
		}
		self.pump_publish();
	}

	/// Whether a row the stores took since the mark `before` starts a ladder over (a clear row of a ladder: another
	/// device's, since the computer refuses `book.restart` while a pump runs); none to read after a reset or before a
	/// first mark.
	fn started_over_since(&self, before: Option<&StoreMark>) -> bool {
		let Some(before) = before.filter(|b| b.floor == self.session.store().floor()) else { return false };
		let ladder = |r: &RowRecord| r.kind == kind::CLEAR && r.reps == clear::LADDER;
		let game = self.session.store().records().get(before.game..).unwrap_or_default();
		let own = self.session.app_rows().records().get(before.own..).unwrap_or_default();
		game.iter().chain(own).any(ladder)
	}

	/// A ladder started over on another device while a move pump started here waits on its Ready view with nothing
	/// done: its start is read again (`Engine::start`), as the game's `Lc.Cleared` asks `ns.StartFor` again; a start it
	/// now refuses ends the pump with nothing written (left at its Ready view), one that doses another move, goal or
	/// rung composes it anew in its place, and an unchanged one stays.
	fn pump_started_over(&mut self) {
		let Some(run) = self.pumps.run.as_ref().filter(|r| r.untouched()) else { return };
		let What::Move { from, target, doses } = run.what.clone() else { return };
		let (id, outside, at) = (run.id.clone(), run.outside, unix_s(lan::now_ms()));
		self.engine_call(
			move |e| e.start(target, at),
			move |h, answer| {
				let start = match answer {
					Ok(start) => start,
					Err(e) => return log::warn!("pump {id}: its start read again: {e}"),
				};
				// still that pump, still untouched
				if !h.pumps.run.as_ref().is_some_and(|r| r.id == id && r.untouched()) || h.pumps.starting {
					return;
				}
				if start.refused.is_some() || start.r#move.is_none() {
					log::info!("pump {id}: a ladder started over elsewhere; its start is refused now, so it ends");
					let _ = h.pump_stop(Stop::Left);
				} else if doses.as_ref() != Some(&Doses::of(&start)) {
					log::info!("pump {id}: a ladder started over elsewhere; its start moved, so it is composed again");
					h.pump_compose(What::Move { from, target, doses: None }, Compose::Move { target }, 1, outside);
				}
			},
		);
	}

	/// The `pump` topic (a guide whose move's run is over stopped first), the window's guard, the computer kept awake
	/// while a pump runs, and the tray's items.
	pub(crate) fn pump_publish(&mut self) {
		self.pump_keep_guide();
		let topic = view::topic(self);
		self.app.state::<Desk>().publisher.publish(TOPIC, &topic);
		let running = self.pumps.on();
		self.app.state::<Desk>().guard.store(running, Ordering::Relaxed);
		if running != self.pumps.awake {
			self.pumps.awake = running;
			awake::hold(&self.app, running);
			// a pump starting cancels a day reel being made (slice 45)
			if running {
				super::reel::cancel(&self.app);
			}
			self.hold_sound(running);
			// Backup says whether a pump is on screen
			self.publish_backup();
			// the game's save folds again for the sheet the app shows once the pump ended (`home_ask` holds that fold
			// back while a pump runs)
			if !running {
				self.home_stale();
			}
		}
		// Your setup says whether a change waits for the next pump (its `pump_on`)
		let waits = running && !self.pump_refits();
		if waits != self.pumps.setup_waits {
			self.pumps.setup_waits = waits;
			self.publish_setup();
		}
		let free = !running && self.pump_gate().is_none();
		let tray = (running, free, !running && self.catch_up_gate().is_none() && topic.owed > 0);
		if tray != self.pumps.tray {
			self.pumps.tray = tray;
			let handle = self.app.clone();
			let _ = self.app.run_on_main_thread(move || draw_tray(&handle, tray));
		}
	}
}

/// What moves an app pump on its Ready view (`Helper::pump_refit`): the player's Pain, the joints to flag (Pause's
/// Something hurts); Your setup's profile (the place, the Body switches, the moves left out or checked); or the pain
/// flags Your setup set, against those the pump knew.
enum Refit {
	Pain(Vec<u8>),
	Profile,
	Flags(Vec<u8>),
}

/// How a pump ends early.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stop {
	Left,
	/// A match pump's Bank the rest on a rider (its bar's, or the "···"'s End that reads so; slice 61, the game's
	/// `Lc.Finish("rest")`): it ends as done, with its Result, the riders left unfinished.
	Rest,
	Game,
	Unwell,
}

/// The tray's pump items: "A pump is on" (no control) while one runs, Quick pump and Catch up where they can start;
/// made on the main thread the first time they show, each just above Open QuestPump.
fn draw_tray(app: &AppHandle, (on, quick, catch_up): (bool, bool, bool)) {
	let Some(lines) = app.try_state::<TrayLines>() else { return };
	if app.try_state::<Tray>().is_none() {
		let made = (|| {
			Ok::<_, tauri::Error>(Tray {
				on: MenuItem::with_id(app, TRAY_ON, copy::desk_pump_tray_on(), false, None::<&str>)?,
				quick: MenuItem::with_id(app, TRAY_QUICK, copy::pump_quick(), true, None::<&str>)?,
				catch_up: MenuItem::with_id(app, TRAY_CATCHUP, copy::pump_waiting_catchup(), true, None::<&str>)?,
				shown: [AtomicBool::new(false), AtomicBool::new(false), AtomicBool::new(false)],
			})
		})();
		match made {
			Ok(tray) => {
				app.manage(tray);
			}
			Err(e) => {
				log::warn!("pump: the tray's items: {e}");
				return;
			}
		}
	}
	let Some(tray) = app.try_state::<Tray>() else { return };
	for (item, shown, want) in [
		(&tray.on, &tray.shown[0], on),
		(&tray.quick, &tray.shown[1], quick),
		(&tray.catch_up, &tray.shown[2], catch_up),
	] {
		let at = lines.above_open();
		lines.show(item, shown, at, want);
	}
}

/// The tray's pump items: Quick pump and Catch up open QuestPump's window and start that pump.
pub(crate) fn on_tray(app: &AppHandle, id: &str) {
	let kind = match id {
		TRAY_QUICK => "pump.quick",
		TRAY_CATCHUP => "pump.catchup",
		_ => return,
	};
	if let Ok(intent) = Intent::from_value(serde_json::json!({"type": kind, "outside": true})) {
		let (done, _) = tokio::sync::oneshot::channel();
		app.state::<Shared>().ask(WindowAsk::Desk(DeskAsk::Intent { intent, done }));
	}
}

/// The loop's wait for the pump's next instant (`Helper::pump_wake`, unix ms): forever while none waits.
pub(crate) async fn wake(at: Option<i64>) {
	match at {
		Some(at) => tokio::time::sleep(Duration::from_millis(u64::try_from(at - lan::now_ms()).unwrap_or(0))).await,
		None => std::future::pending().await,
	}
}
