//! The `pump` topic as the page draws it (`helper/dist/views/pump/`): the pump on screen in the phones' words and
//! values (their `PumpSession`'s Ready view, the move up next in full, its count step and Result, `RewardsLine`'s
//! rewards), with the game's own words from contract/sheet.json filled in here and the app's words left to the page's
//! copy keys; the Result's rewards from the app's own sheet as the engine folded it, and its one reveal's instants and
//! sounds (`ResultReveal`, on the motion tokens), whose sounds the loop plays on its wake.

use std::collections::BTreeMap;
use std::sync::LazyLock;

use questpump_core::codes::{answer, cause, kind, mark, outcome};
use questpump_core::row::Row as WrittenRow;
use questpump_core::session::CatchupSet;
use questpump_engine::{
	CardioDay, Check, CheckOutcome, CheckWait, Goal, LevelAt, NextLoad, PlannedMove, Slot, SwapOptions, Top, Toward,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::run::{self, Beats, MoveRun, Side, Typed};
use super::{
	Bank, Cut, EASY_CADENCE, Gate, HAUL_CADENCE, Payment, Recorded, Run, Stage, StartedFrom, What, cadence_rung,
	rep_beat, rep_count, words,
};
use crate::facts::facts;
use crate::home::Board;
use crate::{Desk, Platform};

pub(super) mod pour;

/// The topic.
#[derive(Serialize, PartialEq)]
pub(super) struct Topic {
	gate: Option<Gate>,
	locked_until: Option<i64>,
	/// The owed moves a catch-up here pays now (none a pain flag on this computer rests).
	pub(super) owed: usize,
	/// Where the player pumps on this computer, asked before the first own pump.
	place: bool,
	choose: Option<ChooseView>,
	/// The joints and their flags, for Something hurts in the Ready view's Pause menu (the joints not flagged), under the
	/// game's question.
	pain: Vec<Joint>,
	pain_ask: &'static str,
	/// The starts that went nowhere on this run (`Desk::pump_nowhere`: refused as the engine read them again,
	/// composing no move, closed by a gate while they waited for the place, or ended by their match offer's recall
	/// while they waited for it or composed, `Desk::pump_recalled`), a count: the page that pressed one sees it move
	/// with no pump on screen and takes its card or window back.
	refused: u32,
	/// A pump runs, from its Ready view until its moves are done (`Desk::desk_pump_running`; its Result is not
	/// running): the one predicate by which Rust refuses Correct a count's Save and the page greys it.
	running: bool,
	pump: Option<PumpView>,
}

#[derive(Serialize, PartialEq)]
struct ChooseView {
	kind: Option<String>,
	size: u8,
	offer: Option<String>,
	/// The plain kinds' keys in their order (contract/kinds.json).
	kinds: Vec<&'static str>,
	/// The aim game that runs with its arms rest on now (`Desk::arms_game`, slice 48's double-check): a pump from an
	/// offer or of kind `match` rests the arms for it, which the window says (`pump.rest.arms`).
	arms: Option<String>,
	/// Every size of a match's pump takes the same moves (Moves per match at Light, `Desk::match_one_size`): a pump
	/// from an offer or of kind `match` shows no size choice, its size still sent.
	one_size: bool,
}

/// A joint as Something hurts lists it: its words, and whether a flag rests it already (which leaves it out).
#[derive(Serialize, PartialEq)]
struct Joint {
	id: u8,
	name: &'static str,
	flagged: bool,
}

#[derive(Serialize, PartialEq)]
#[expect(clippy::struct_excessive_bools, reason = "the pump's flags as the page reads them, one field each")]
struct PumpView {
	id: String,
	what: &'static str,
	kind: Option<String>,
	/// A move pump's start, which its Result's Continue names and returns to (`super::StartedFrom`).
	from: Option<StartedFrom>,
	/// A stretch check's pump (slice 42: "Do the check"): its Ready view says Stretch check over the pose's name.
	check: bool,
	/// The aim game whose arms rest it composed with (slice 48's double-check): its Ready view names it
	/// (`pump.rest.arms`).
	arms: Option<String>,
	busy: bool,
	stage: &'static str,
	current: usize,
	#[serde(flatten)]
	ways: Ways,
	go: Option<i64>,
	digit: Option<i64>,
	start: Option<i64>,
	limit: Option<f64>,
	side: Option<&'static str>,
	asked: Option<AskedView>,
	marking: Option<MarkingView>,
	/// Keep going shows over Done (slice 52: a steady move's last `keep_going_s` under the full move).
	keep_going: bool,
	/// Keep going was pressed on the running side (`MoveRun::kept`): its dial counts up to `limit`, the full move, and
	/// Space presses no Done for the rest of the move, as in the game.
	kept: bool,
	moves: Vec<MoveView>,
	rewards: Vec<Pill>,
	lines: Vec<Line>,
	/// The move on's words until the pump's end, as the game's Ready view shows them (2026-10-24): its setup line with
	/// the implement that carries its load, How's steps (the implement's own where it has them, `Facts::how`) and Tips
	/// with its own advice last (`Facts::tips`), which its Ready view and its "How to do it" window show.
	setup: Option<&'static str>,
	how: &'static [String],
	tips: Vec<&'static str>,
	/// The read-aloud button's state (`super::guide`): `idle`, `reading` (held by a line said over it too) or `paused`;
	/// none on a stage the guide reads on no longer, or where the install the app follows holds no take of the move's.
	guide: Option<&'static str>,
	swap: Option<SwapView>,
	count: Option<CountView>,
	result: Option<Earned>,
	/// A work pump's Result reads Back to work (slice 67, `Run::back`): its Continue brings the app it came from back.
	back: bool,
	/// Work mode's work pump (slice 67, `Run::work`): its Pause menu holds Your body alone, never the offers' pauses
	/// (`Desk::pump_pause` refuses them on it).
	work: bool,
	/// The pump ended done (`Run::ended_done`): an app pump over with a move done and no felt-unwell stop, which a
	/// browser's Save offer and its `play` topic count (questpump-web `Play::seen`, docs/plan.md slice 63); never one
	/// that ended before a move ran (a pain flag or a setup change over Ready that left no move).
	ended_done: bool,
	reveal: Option<Reveal>,
	/// The Legendary moment on show over the Result: its row's index among the result's rows.
	legend: Option<usize>,
	calm: Option<CalmView>,
	/// A move's pour, its rewards and its step's reveal (slices 65 and 66), on its `saved` stage.
	saved: Option<pour::SavedView>,
	/// The expedition it is a round of (slice 51): its bar, its breather and its Result's facts.
	expedition: Option<super::expedition::TripView>,
}

/// The quiet ways a stage offers beside its primary: Back once a move started (`Run::can_back`), on a rider's Ready
/// view Skip for its move (`pump.skip`: a catch-up's since 2026-10-09, a match pump's rider since slice 61), and on a
/// match pump's or a catch-up's Ready view its way to put the moves off (`pump.bank`, slice 58: `it`, Bank all, or
/// `rest`, Bank the rest; `Desk::pump_bank_way`). `rest`: only riders are left of a catch-up or a match pump
/// (`Run::only_riders_left`, slice 61), so the "···"'s end reads Bank the rest and Leave speaks of the unfinished moves.
#[derive(Serialize, PartialEq)]
struct Ways {
	back: bool,
	skip: bool,
	bank: Option<Bank>,
	rest: bool,
}

#[derive(Serialize, PartialEq)]
struct AskedView {
	seconds: i64,
	full: i64,
}

/// A pose's "How close to the goal pose?" (2026-10-09; its words 2026-10-14, David: "whats to achieve, the "mark"
/// (please rename for clarity), is not very clear"): its goal pose (`mark`, the pose's own mark); no note under the
/// answers since 2026-10-22.
#[derive(Serialize, PartialEq)]
struct MarkingView {
	mark: Option<&'static str>,
}

/// A move of the pump and what its stages show of it.
#[derive(Serialize, PartialEq)]
#[expect(clippy::struct_excessive_bools, reason = "the move's flags as the page reads them, one field each")]
struct MoveView {
	slot: usize,
	#[serde(rename = "move")]
	move_id: u16,
	/// It pays an unfinished move (`Run::rider`): a catch-up's every move, a match pump's rider (slice 61), which the
	/// steps bar draws in the unfinished tone until done and its read-out counts, and by which the page words Skip, End
	/// and Leave as a catch-up's.
	rider: bool,
	key: &'static str,
	/// The gear key whose implement carries its load (`PlannedMove::implement`): the page draws the figure of that
	/// gear where the figures' table has one (`lib/figure.js` `figureName`), else the move's own.
	implement: Option<String>,
	name: &'static str,
	quality: Option<u8>,
	word: Option<&'static str>,
	kind: u8,
	hold: bool,
	/// Its target as its Ready view and Big view show it: `goal` (its count), `time` (seconds), `max` (a max set),
	/// `set` (one set), and since slice 42 `allout` (a counted set: All out over its seconds, no number to reach).
	target: TargetView,
	/// The record its goal beats, which its window names and its Ready view beside its goal, a side move's each side
	/// (its line until 2026-10-24); a counted set's best clean count, which its Ready view says as a line instead
	/// (`Line::Best`, slice 42), as the phones'.
	record: Option<i64>,
	/// A loaded move's weight: its plan, its weighted record (which its Ready view names in the record's place, a side
	/// move's too).
	load: Option<LoadView>,
	mark: Option<&'static str>,
	/// The line under its goal on Ready: a first max set's how far to go (the game's words).
	note: Option<String>,
	sides: bool,
	allout: bool,
	counted: bool,
	seconds: Option<i64>,
	build: Option<i64>,
	state: &'static str,
	/// Its stop line, the form break that ends its set; none on a timed move that has none (David, 2026-10-24: only
	/// what is legally necessary).
	stop: Option<&'static str>,
	/// Its first appearance on its Ready view (never done, or a placement): the view shows the right-versus-wrong pair.
	first: bool,
	/// A set of reps' beat (ms), its figure's loop at real tempo (`rep_beat`): Big view keeps the figure in the ring
	/// through the running move, phased to the go, the pace the voice counts on; none for every other kind and without
	/// a loop.
	beat: Option<i64>,
	/// A steady cardio move's cadence rung (slice 42's fix round, 2026-10-17: `cadence_rung`), whose pace in words
	/// (`pump.pace.rung<n>`) Big view shows under the running move, 0 a match pump's warm move's easy cadence
	/// (`EASY_CADENCE`, `pump.pace.easy`; slice 48's double-check), a Long Haul's rung 1 (slice 55); none on any other
	/// move or an all-out window.
	pace: Option<u8>,
	/// A set of reps the voice counts (2026-10-13): its count's rule (`rep_count`: `{beat, ends, most}`, the ms in the
	/// loop each rep ends and one set's most), by which Big view shows the reps its figure has ended on the side's clock
	/// each frame (`stages.js` `ended`, `Beats::ended`'s twin) in place of its goal or its word; none where `beat` is
	/// none or the player's switches keep the voice from counting (`Desk::counts_chosen`).
	rep_count: Option<Beats>,
}

#[derive(Serialize, PartialEq)]
struct TargetView {
	kind: &'static str,
	value: Option<i64>,
	/// A count or a hold's time to reach on a ladder that had sets this game day: the set of it this one is that day,
	/// its sets so far plus one (`PlannedMove::today`), which the goal's word names ("Goal · set 2 today", 2026-10-28).
	set: Option<u32>,
}

/// A loaded move's weight on its Ready view: the weight planned (none before a set with a weight), whether it steps up,
/// a weight in each hand, and the weighted record. Weights are tenths of a kg.
#[derive(Serialize, PartialEq)]
struct LoadView {
	plan: Option<u32>,
	up: bool,
	each: bool,
	top: Option<Lifted>,
}

/// A set with a weight: the weight (tenths of a kg) and the reps.
#[derive(Serialize, Clone, Copy, PartialEq)]
struct Lifted {
	load: u32,
	reps: i64,
}

impl Lifted {
	fn of(load: Option<u32>, reps: Option<i64>) -> Option<Self> {
		Some(Self { load: load?, reps: reps? })
	}
}

/// A pill of the Ready view's rewards.
#[derive(Serialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Pill {
	Xp {
		amount: i64,
	},
	/// The stat a move's record would raise, if its set beats `beat` (a count, a hold's seconds): a pill a move.
	Stat {
		amount: i64,
		key: &'static str,
		stat: &'static str,
		beat: i64,
		hold: bool,
	},
	Record,
}

/// A line on the Ready view about the move on (Panel.lua `goalOf`), its values alone: since 2026-10-22 no
/// line that only explained (a first clean count's, a stretch check's, a lighter goal's reason).
#[derive(Serialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Line {
	/// The game's words.
	Words {
		text: String,
	},
	Carry {
		seconds: i64,
	},
	CarrySide {
		seconds: i64,
	},
	FlowSide {
		seconds: i64,
	},
	/// A counted set's goal (slice 42): all out, clean reps, no number to reach.
	AlloutClean,
	/// A counted set's best clean count under its goal (slice 42), a record, never a goal.
	Best {
		count: i64,
	},
	/// Under a pose's goal pose where its answer counts its family's day (slice 47, Panel.lua `poseDaysLine`): any goal
	/// pose of the step counts one, Perfect alone below a harder step (`perfect`, the move's ask), `days` of the `total`
	/// the step takes so far, and what they still lack (`short`).
	PoseDays {
		perfect: bool,
		days: i64,
		total: i64,
		short: Short,
	},
	/// A steady move's pace in words, the talk test of its cadence (`pump.pace.rung<n>`, slice 42's fix round,
	/// 2026-10-17: the pace ask's "that pace" stated without the voice; a match pump's warm move's easy cadence, rung
	/// 0, `pump.pace.easy`).
	Pace {
		rung: u8,
	},
	/// A goal from the player's form, which reaches the record at most: their best fresh set lately, a count (a hold's
	/// seconds) or with a weight (tenths of a kg).
	Form {
		form: i64,
		hold: bool,
		load: Option<u32>,
		each: bool,
	},
	/// A loaded move with no weight logged yet: the library's start range, whole kg.
	LoadStart {
		lo: u32,
		hi: u32,
		each: bool,
	},
	/// A loaded move whose plan steps up: the reps its fresh sets reached twice (`load_top`) at the working weight it
	/// steps up from (`load_from`).
	LoadHeavier {
		reps: i64,
		load: u32,
	},
}

/// What a pose family's days toward its step still lack (slice 47, Tree.lua `Tree.PoseShort`; the shape of the Book's
/// pose climb, `views/pump/words.js` `poseShort`): the days still to come on the step's main pose (`main`), by its name
/// (`pose`: none where the words lack it), and those still to come answered Perfect (`perfect`).
#[derive(Serialize, PartialEq)]
struct Short {
	main: i64,
	pose: Option<&'static str>,
	perfect: i64,
}

/// Swap's offers for the move on Ready (Panel.lua `renderSwap` and `swapItems`; the one place to swap, as in the game,
/// since 2026-10-26): in the game's order, the first the swap row's under the goal, the rest its chevron's window's,
/// which `title` heads (the game's "Swap this move").
#[derive(Serialize, PartialEq)]
struct SwapView {
	title: &'static str,
	options: Vec<SwapOption>,
}

#[derive(Serialize, PartialEq)]
struct SwapOption {
	take: &'static str,
	#[serde(rename = "move")]
	move_id: u16,
	key: &'static str,
	name: &'static str,
	quality: Option<u8>,
	/// The offer in one line with the move named, the swap row's words ("Easier version: Frog stand from books",
	/// Panel.lua `swapWords`).
	line: String,
	/// What kind of swap it is, the game's words without the move ("Easier version"), under its name in the window.
	kind: String,
}

/// The count step, set on the dial since slice 66 (and the count a hold's "Could you have held longer?" asks about,
/// which the dial shows still); on a counted set since slice 42 its form step (the clean reps if counted, Yes, to the
/// end, Form slipped and since slice 53 Not all out), its record the form's best clean count.
#[derive(Serialize, PartialEq)]
struct CountView {
	/// `reps`, `reps_side`, `hold`, `hold_side` or `form` (a counted set's).
	question: &'static str,
	name: &'static str,
	seconds: i64,
	from: i64,
	/// The most the dial takes: half again what the app counted, on a set of reps or its number to beat, where that
	/// binds (`Run::count_cap`), else one set's most.
	most: i64,
	/// Where the share set that most (slice 66): it and what set it, which the line names while the count stands at it.
	cap: Option<CapView>,
	hold: bool,
	record: Option<i64>,
	chip: Option<i64>,
	max_set: bool,
	max_line: String,
	check: Option<CheckView>,
	change: &'static str,
	empty: &'static str,
	could_not: Option<&'static str>,
	/// A loaded move's weight box beside the reps.
	kg: Option<KgView>,
}

/// The most a count takes and what set it (`run::CapBy`): what the app counted (`pump.count.cap`'s {max} and {counted}),
/// or the number a set of reps has to beat (`pump.count.cap.goal`'s {max} and {goal}).
#[derive(Serialize, PartialEq)]
struct CapView {
	max: i64,
	counted: Option<i64>,
	goal: Option<i64>,
}

impl CapView {
	fn of(max: i64, by: run::CapBy) -> Self {
		match by {
			run::CapBy::Counted(n) => Self { max, counted: Some(n), goal: None },
			run::CapBy::Goal(n) => Self { max, counted: None, goal: Some(n) },
		}
	}
}

/// A loaded move's weight box (tenths of a kg), its - and + as Progress.lua `ns.LoadStep` steps them: a weight in each
/// hand, where it starts (the weight typed before a check, none where the player emptied the box, else `first`; Rapha,
/// 2026-11-02: "it shuold prefil the kg"), the weight either key offers while it is empty (the plan, else the start
/// range's low end, a kettlebell's first size at or above it), its
/// most, a kettlebell's sizes its keys step through and stay inside (none: `step`, `load_box_step`), and the weighted
/// record, whose score the reps' record chip cannot show.
#[derive(Serialize, PartialEq)]
struct KgView {
	each: bool,
	from: Option<u32>,
	first: u32,
	most: u32,
	sizes: Vec<u32>,
	step: u32,
	top: Option<Lifted>,
}

/// "Is that right?" about the count (and the weight) it names, which its Yes sends again with a counted set's answer
/// (`answer`, `ns.Enum.answer`), in the game's words.
#[derive(Serialize, PartialEq, Debug)]
pub struct CheckView {
	value: i64,
	load: Option<u32>,
	answer: u8,
	#[serde(flatten)]
	words: CheckWords,
}

/// The check's words: the game's filled in, or for a count with a weight the game's with `%s` for the amounts, which
/// the page fills with its weighted ones ("11 kg × 16"): the count, then the record it was checked against (the
/// weighted record where the move has one, else the count's); or the weight's own question (`heavy`), the weight typed
/// and the weight planned in kg, its Yes naming the set with its weight.
#[derive(Serialize, PartialEq, Debug)]
#[serde(tag = "words", rename_all = "snake_case")]
enum CheckWords {
	Filled { line: String, yes: String },
	Weighted { line: &'static str, yes: &'static str, record: i64, record_load: Option<u32>, each: bool },
	Heavy { line: &'static str, yes: &'static str, plan: u32, each: bool },
}

/// A pump that ended short: why, and what its card says beside the page's words.
#[derive(Serialize, PartialEq)]
struct CalmView {
	/// `unwell`, `left`, `game` (the game offered a pump) or `ended` (no move ran).
	why: &'static str,
	until: Option<i64>,
	title: Option<&'static str>,
	/// The game reads the rest at its next load (a game folder with QuestPump is followed).
	reload: bool,
	/// A catch-up's payment: `paid`, `unpaid` (paid elsewhere meanwhile), `reset` (the progress reset since it began),
	/// `locked` (felt unwell before it began) or `game` (the game asks them itself).
	paid: Option<&'static str>,
}

/// The app's own sheet summary as the Result reads it (Stats.lua `ns.SheetSummary` v 5: the level, the XP into it and
/// what the next needs, the Paragon rank, the lifetime XP, the rested pool, the last pump's XP, points and stars, its
/// stats by stat, its unlocks, its day, the weekly vault's goals it reached and the vault's counts, whose week's
/// embers name the tier an embers goal reached, slice 55), and each ladder's rung with the move that stands for it
/// (`lad`) and its days toward the next step (`nx`, ns.UnlockProgress), whose days at the top a move's row counts
/// (2026-10-29).
#[derive(Deserialize, Default, Clone, PartialEq)]
#[serde(default)]
pub(super) struct Sum {
	lvl: i64,
	into: i64,
	need: i64,
	par: i64,
	rest: i64,
	last: Vec<i64>,
	ls: Vec<i64>,
	lu: Vec<Vec<i64>>,
	ld: Vec<i64>,
	lv: Vec<i64>,
	va: Vec<i64>,
	lad: Vec<Vec<i64>>,
	nx: Vec<Vec<i64>>,
}

impl Sum {
	pub(super) fn read(v: &Value) -> Option<Self> {
		serde_json::from_value(v.clone()).map_err(|e| log::warn!("pump: the summary does not read: {e}")).ok()
	}

	fn last(&self, i: usize) -> i64 {
		self.last.get(i).copied().unwrap_or(0)
	}

	fn day(&self, i: usize) -> i64 {
		self.ld.get(i).copied().unwrap_or(0)
	}
}

/// A point on the XP bar.
#[derive(Serialize, Clone, Copy, PartialEq)]
struct Point {
	level: i64,
	into: i64,
	need: i64,
	paragon: i64,
}

impl Point {
	fn of(s: &Sum) -> Self {
		Self { level: s.lvl, into: s.into, need: s.need, paragon: s.par }
	}

	fn at(l: LevelAt) -> Self {
		Self { level: l.level, into: l.into, need: l.need, paragon: l.paragon }
	}

	/// Its place on the bar's one axis: the level and each Paragon rank, and the share into it.
	fn place(self) -> f64 {
		let share = if self.need > 0 { (run::real(self.into) / run::real(self.need)).clamp(0.0, 1.0) } else { 1.0 };
		run::real(self.level + self.paragon) + share
	}
}

#[derive(Serialize, Clone, PartialEq)]
struct Bar {
	#[serde(flatten)]
	now: Point,
	rested: i64,
	from: Option<Point>,
}

/// The fanfare's words: `quest`, `moves` (a catch-up's), `level`, `paragon`, `streak` or `expedition` (its minutes),
/// with its number.
#[derive(Serialize, Clone, Copy, PartialEq)]
struct Banner {
	kind: &'static str,
	number: i64,
}

/// A reward row (`RewardRow.Kind`), its values for the page to word; `Vault` a weekly vault goal the pump reached
/// (`moves`, `records`, `embers`) with the goal it is (an embers tier's count, slice 55). A record's `from` is the
/// record it beat, a count or a hold's seconds (none for a set with a weight, whose record is a score), which the share
/// card's headline names.
#[derive(Serialize, Clone, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Reward {
	Record {
		value: i64,
		hold: bool,
		load: Option<u32>,
		each: bool,
		#[serde(skip_serializing_if = "Option::is_none")]
		from: Option<i64>,
	},
	FirstRecord {
		value: i64,
		hold: bool,
		next: Option<i64>,
		load: Option<u32>,
		each: bool,
	},
	Enough {
		value: i64,
		hold: bool,
	},
	Placed {
		value: i64,
		record: i64,
		hold: bool,
		load: Option<u32>,
		record_load: Option<u32>,
		each: bool,
	},
	Star {
		count: i64,
	},
	Stat {
		amount: i64,
		key: &'static str,
		stat: &'static str,
	},
	Points {
		count: i64,
	},
	Title {
		title: &'static str,
	},
	Unlock {
		ladder: &'static str,
		step: i64,
		quality: u8,
		word: &'static str,
		rekindled: bool,
		way: &'static str,
	},
	Day {
		line: &'static str,
		value: i64,
	},
	Milestone {
		days: i64,
	},
	Vault {
		goal: &'static str,
		count: i64,
	},
	/// A step mastered (slice 42: every move the setup offers there reached its top once), its ladder and step.
	Mastered {
		ladder: &'static str,
		step: i64,
	},
	/// A stretch check passed (slice 42), the move it opens, open after the set (slice 42's fix round, 2026-10-17: only
	/// then); a climb's check, the climb to `ladder`'s `step` open; a check passed whose move or climb another wait
	/// still holds, that wait in its own words (`wait`); the stretch ladder it lifted to a step; a check that waits.
	Check {
		r#move: &'static str,
	},
	CheckClimb {
		ladder: &'static str,
		step: i64,
	},
	CheckHeld {
		wait: WaitWords,
	},
	CheckPlaced {
		ladder: &'static str,
		step: i64,
	},
	CheckWaits,
	/// A counted set's clean count over the form's best (or its first), a clean count under it (the best that stands),
	/// a set saved Form slipped (slice 42) and one saved Not all out (slice 53).
	BestNew {
		count: i64,
	},
	Best {
		count: i64,
	},
	Slipped,
	NotAllOut,
	/// The clean day (slice 42) or the full move (slice 52) a move brought toward the cardio climb's `step`, `have` of
	/// `need`.
	CleanDay {
		have: i64,
		need: i64,
		step: i64,
	},
	FullMove {
		have: i64,
		need: i64,
		step: i64,
	},
	/// A Long Haul with a Yes at its length (slice 55): its `time` in seconds, its day `have` of `need` toward the next
	/// length `next`.
	HaulDay {
		time: i64,
		have: i64,
		need: i64,
		next: i64,
	},
	/// Back from a break, the cardio step's measure (slice 53): the step it holds, or the one it goes `back` to.
	Return {
		back: bool,
		step: i64,
	},
	/// A proving window that did not pass (slice 53): "No step up yet.".
	ClimbWaits,
	/// A move the pump did, as one row on its tile (2026-10-29, Rapha: the Result shows progress): what it did (`Done`'s
	/// count, clock, time, weight), its record's `tag` (`record`, a count past the one before; `first`, a move's first
	/// record) with the record it passed (`was`), its own `notes` worded as their rows are (a first record's next goal,
	/// enough for the next step, a placement, a counted set's best that stands, Form slipped, Not all out, a measure back
	/// from a break, the climb that waits), the next weight (`again`), and the `steps` it counts toward a step up, each
	/// with its days as pips (a clean day, a full move, a Long Haul's day, which this pump brought in; the days at the
	/// top, `TopDay`).
	Moved {
		count: Option<i64>,
		clock: bool,
		time: Option<i64>,
		load: Option<u32>,
		each: bool,
		tag: Option<&'static str>,
		was: Option<Was>,
		notes: Vec<Reward>,
		again: Option<Again>,
		steps: Vec<Reward>,
	},
	/// On the move that stands for its strength ladder's step, its game days at the top of its range so far, `have` of
	/// the `need` the climb to `step` takes.
	TopDay {
		have: i64,
		need: i64,
		step: i64,
	},
	/// The cardio moves done in full this game day (embers, slice 55), `have` of the `need` a day lights, on a pump that
	/// lit one.
	Cardio {
		have: i64,
		need: i64,
	},
	/// The stats the pump raised, in one row of chips (each a `Stat`).
	Stats {
		stats: Vec<Reward>,
	},
	/// The XP one save earned, on a move's pour (slice 65; the Result carries the pump's on its bar).
	Xp {
		xp: i64,
	},
}

/// The record a move's record passed: its count (a hold's seconds), with a loaded move's weight.
#[derive(Serialize, Clone, Copy, PartialEq)]
struct Was {
	value: i64,
	hold: bool,
	load: Option<u32>,
	each: bool,
}

/// A loaded move's next fresh set (`NextLoad`): "Next time: 11 kg, 15+ reps".
#[derive(Serialize, Clone, Copy, PartialEq)]
struct Again {
	load: u32,
	target: i64,
}

/// What still holds what a passed check opens (`CheckWait`), for its Result row in the wait's own words (slice 42's fix
/// round, 2026-10-17; the page words it with the card's keys): another stretch check (`check`: its pose and whether only
/// Perfect passes it), a record of another ladder (`record`: the move and its count, a hold's in seconds), a step of
/// another ladder its climb does (`step`: the move and its ladder), or the stage of a step's path (`stage`: the move
/// whose top opens it with that top, a hold's in seconds, a side move's "a side", a loaded move's weight, `kg` and `each`
/// as the game's `Sheet.ClimbTop` adds it, or a pose's first Good or Perfect day; none for a countable form's clean
/// window).
#[derive(Serialize, Clone, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum WaitWords {
	Check {
		pose: &'static str,
		perfect: bool,
	},
	Record {
		r#move: &'static str,
		count: i64,
		seconds: bool,
	},
	Step {
		r#move: &'static str,
		ladder: &'static str,
	},
	Stage {
		r#move: &'static str,
		top: Option<i64>,
		seconds: bool,
		side: bool,
		pose: bool,
		perfect: bool,
		#[serde(skip_serializing_if = "Option::is_none")]
		kg: Option<i64>,
		#[serde(skip_serializing_if = "std::ops::Not::not")]
		each: bool,
	},
}

impl WaitWords {
	/// A wait's words (none where the words lack its move or its ladder).
	fn of(wait: CheckWait) -> Option<Self> {
		Some(match wait {
			CheckWait::Check { r#move, count, ask, ladder, seconds } => {
				let name = facts().name(r#move?)?;
				match (ask, count) {
					(Some(ask), _) => Self::Check { pose: name, perfect: ask >= mark::PERFECT },
					(None, Some(count)) => Self::Record { r#move: name, count, seconds },
					(None, None) => Self::Step { r#move: name, ladder: facts().ladder_name(ladder?) },
				}
			}
			CheckWait::Path { after, top, kg, each } => {
				let after = after?;
				let m = facts().library.moves.get(&after)?;
				let count = match top {
					Some(Top::Count(n)) => Some(n),
					_ => None,
				};
				Self::Stage {
					r#move: facts().name(after)?,
					top: count,
					seconds: m.dose == "hold",
					side: facts().text(after).is_some_and(|t| t.side),
					pose: m.dose == "flow",
					perfect: count.is_some_and(|n| n >= i64::from(mark::PERFECT)),
					kg,
					each,
				}
			}
		})
	}
}

impl Reward {
	/// An unlock said quietly, with no fanfare, shine or share: Rekindled, or an orange or gold step its climb only
	/// opened (`way` since 2026-10-09: `climb`, `opened`, or `earned`, such a step a set earned, "Earned: …").
	fn quiet(&self) -> bool {
		matches!(self, Self::Unlock { rekindled, way, .. } if *rekindled || *way == "opened")
	}

	/// Its place in the list (Sheet.lua `rank`): a milestone, a Legendary or Artifact unlock, a title, an Epic unlock
	/// and a stretch check passed with what it opens, a star, a step mastered, a check's lift and any other unlock (a
	/// quiet one too), a record (a new best clean count), then the rest (a check that waits or whose opening still
	/// waits, a best that stands, a slipped form or one not all out, a clean day, a full move, a Long Haul's day, a
	/// measure's step held or back, the climb that waits, the cardio today, the stats). The moves the pump did lead with
	/// the milestone, in the pump's order (2026-10-29).
	fn rank(&self) -> i8 {
		match self {
			Self::Milestone { .. } | Self::Moved { .. } => -1,
			Self::Unlock { .. } if self.quiet() => 2,
			Self::Unlock { quality, .. } if *quality >= LEGENDARY => 0,
			Self::Unlock { quality, .. } if *quality == EPIC => 1,
			Self::Title { .. } | Self::Check { .. } | Self::CheckClimb { .. } => 1,
			Self::Unlock { .. }
			| Self::Star { .. }
			| Self::Vault { .. }
			| Self::Mastered { .. }
			| Self::CheckPlaced { .. } => 2,
			Self::Record { .. }
			| Self::FirstRecord { .. }
			| Self::Enough { .. }
			| Self::Placed { .. }
			| Self::BestNew { .. } => 3,
			Self::Stat { .. }
			| Self::Points { .. }
			| Self::Day { .. }
			| Self::CheckWaits
			| Self::CheckHeld { .. }
			| Self::Best { .. }
			| Self::Slipped
			| Self::NotAllOut
			| Self::CleanDay { .. }
			| Self::FullMove { .. }
			| Self::HaulDay { .. }
			| Self::Return { .. }
			| Self::ClimbWaits
			| Self::TopDay { .. }
			| Self::Cardio { .. }
			| Self::Stats { .. }
			| Self::Xp { .. } => 4,
		}
	}

	/// The feedback event it fires as the reveal shows it.
	fn event(&self) -> &'static str {
		match self {
			Self::Milestone { .. } => "milestone",
			Self::Record { .. } | Self::BestNew { .. } | Self::Moved { tag: Some("record"), .. } => "record",
			// a step mastered: the game's small chime, as its toast's (slice 42)
			Self::Mastered { .. } => "win",
			Self::Check { .. } | Self::CheckClimb { .. } => "unlock",
			Self::Title { .. } => "title",
			Self::Unlock { quality, .. } => {
				if self.quiet() || *quality <= COMMON {
					"unlock"
				} else if *quality == UNCOMMON {
					"unlockUncommon"
				} else if *quality == RARE {
					"unlockRare"
				} else if *quality >= LEGENDARY {
					"unlockLegendary"
				} else {
					"unlockEpic"
				}
			}
			_ => "reveal",
		}
	}

	/// An Epic or higher unlock, not quiet: its tile glows once more after its shine.
	fn pulses(&self) -> bool {
		matches!(self, Self::Unlock { quality, .. } if *quality >= EPIC && !self.quiet())
	}
}

/// The gear key of a kettlebell (Library.lua's `ns.Gear`): a loaded move it carries steps through its sizes.
const KETTLEBELL: &str = "kettlebell";
/// A weight's unit in the rows and the engine: tenths of a kg (the library's ranges and the box's bounds are whole kg).
const TENTHS: u32 = 10;

/// The qualities as the screens count them (contract/sheet.json `qualities` from 1).
const COMMON: u8 = 2;
const UNCOMMON: u8 = 3;
const RARE: u8 = 4;
const EPIC: u8 = 5;
const LEGENDARY: u8 = 6;

#[derive(Serialize, Clone, PartialEq)]
struct Tile {
	name: &'static str,
	key: &'static str,
	quality: Option<u8>,
}

#[derive(Serialize, Clone, PartialEq)]
struct Row {
	reward: Reward,
	#[serde(rename = "move")]
	tile: Option<Tile>,
}

/// What the pump earned and did: the fanfare (none on a calm card), its turn, the XP (none for a catch-up), the bar
/// (the app's own sheet's while Hero shows it, else the game's level with the pump's XP on it; none for a catch-up),
/// the reward rows in their order (each move the pump did first, `Reward::Moved`), a catch-up's payment, and the embers
/// where a move lit the game day's last ember in the app's own fold (the "Cardio done today" Toast, slice 55; its week
/// never fewer than Hero's vault card says).
#[derive(Serialize, Clone, PartialEq)]
pub(super) struct Earned {
	banner: Option<Banner>,
	turn: Option<Banner>,
	xp: Option<i64>,
	bar: Option<Bar>,
	rows: Vec<Row>,
	paid: Option<&'static str>,
	embers: Option<Kindled>,
}

/// The "Cardio done today" Toast's news (slice 55): the week's embers under its words, and the game day's
/// (`embers_day`), every one lit, its pips as the phones' Toast leads with them.
#[derive(Serialize, Clone, Copy, PartialEq, Eq)]
pub(super) struct Kindled {
	week: i64,
	day: i64,
}

impl Kindled {
	/// The Toast's news from the week's embers a move's set made (`Run::kindled`).
	fn of(week: i64) -> Self {
		Self { week, day: facts().constants().embers_day }
	}
}

impl Earned {
	/// The records it names: a record beaten, a first record, a weighted set placed past one.
	pub(super) fn records(&self) -> usize {
		self.rows.iter().filter(|r| matches!(r.reward, Reward::Record { .. } | Reward::FirstRecord { .. })).count()
	}

	/// The XP it paid (none for a catch-up's).
	pub(super) fn xp(&self) -> Option<i64> {
		self.xp
	}

	/// The share card's headline facts (docs/plan.md slice 51, "The share cards tell one true thing"): the highest
	/// colour a step it opened reached past `top`, the highest the player had reached before (none while unknown, so
	/// no colour is claimed new), Common and up; and its record with the biggest step past the record it beat, a first
	/// record or one with a weight (no step to measure) only where none beat a counted record.
	pub(super) fn brag(&self, top: Option<u8>) -> Brag {
		let colour = top.and_then(|top| {
			self.rows
				.iter()
				.filter_map(|row| match row.reward {
					Reward::Unlock { ladder, quality, word, .. } if quality > top.max(COMMON - 1) => {
						Some(ColourBrag { quality, word, r#move: row.tile.as_ref().map_or(ladder, |t| t.name) })
					}
					_ => None,
				})
				.max_by_key(|c| c.quality)
		});
		let step = |b: &RecordBrag| b.from.map(|from| run::real(b.value - from) / run::real(from.max(1)));
		let record = self
			.rows
			.iter()
			.filter_map(|row| {
				let r#move = row.tile.as_ref()?.name;
				match row.reward {
					Reward::Record { value, hold, load, each, from } => {
						Some(RecordBrag { r#move, value, hold, load, each, from })
					}
					Reward::FirstRecord { value, hold, load, each, .. } => {
						Some(RecordBrag { r#move, value, hold, load, each, from: None })
					}
					_ => None,
				}
			})
			.fold(None, |best: Option<RecordBrag>, b| match &best {
				Some(kept) if step(&b).unwrap_or(-1.0) <= step(kept).unwrap_or(-1.0) => best,
				_ => Some(b),
			});
		Brag { colour, record }
	}

	/// The next orange or gold step's row after the one `shown` (the Legendary moment's, from the first; 2026-10-09):
	/// a row whose reveal plays `unlockLegendary`, so a step said quietly has none; its index and its move's figure key
	/// (the voice's `legend.<key>`), none where no move stands for the step.
	pub(super) fn legend_after(&self, shown: Option<usize>) -> Option<(usize, Option<&'static str>)> {
		let from = shown.map_or(0, |i| i + 1);
		self.rows
			.iter()
			.enumerate()
			.skip(from)
			.find(|(_, row)| row.reward.event() == "unlockLegendary")
			.map(|(i, row)| (i, row.tile.as_ref().map(|t| t.key).filter(|k| !k.is_empty())))
	}
}

/// The share card's headline facts (`Earned::brag`).
#[derive(Serialize, Clone, Default, PartialEq)]
pub(super) struct Brag {
	colour: Option<ColourBrag>,
	record: Option<RecordBrag>,
}

/// A colour reached for the first time: its quality (2 Common … 7 Artifact) and word, and the move that stands for the
/// step that opened it.
#[derive(Serialize, Clone, PartialEq)]
struct ColourBrag {
	quality: u8,
	word: &'static str,
	r#move: &'static str,
}

/// A record: its move, its count or seconds (a set with a weight, its reps and the weight), and the record it beat.
#[derive(Serialize, Clone, PartialEq)]
struct RecordBrag {
	r#move: &'static str,
	value: i64,
	hold: bool,
	load: Option<u32>,
	each: bool,
	from: Option<i64>,
}

/// What the Result reads from a pump that paid on the app's own sheet.
pub(super) struct Paid<'a> {
	pub(super) moves: &'a [PlannedMove],
	pub(super) recorded: &'a BTreeMap<usize, Recorded>,
	pub(super) ran: &'a BTreeMap<usize, i64>,
	pub(super) before: Option<&'a Sum>,
	pub(super) after: Option<&'a Sum>,
	pub(super) titles_before: &'a [u8],
	pub(super) titles_after: &'a [u8],
	/// A first record's next step, by slot.
	pub(super) goals: &'a [(usize, i64)],
	/// A set with a weight, by slot: whether the engine judged it a record.
	pub(super) beat: &'a BTreeMap<usize, bool>,
	/// Each move done with a weight and its next fresh set.
	pub(super) nexts: &'a [(u16, NextLoad)],
	/// Hero shows the app's own sheet (no game sheet): its level, bar, day and titles are the Result's.
	pub(super) hero_own: bool,
	/// Else what the streak's board says it reached.
	pub(super) reached: Reached,
	/// The rows the engine wrote for its moves (the steps they mastered, a check's lift), its answers by slot (a pose's
	/// mark, a steady move's pace), whether it is a stretch check's pump (slice 42; the check it runs is the outcome the
	/// engine marks `runs`), and what the checks each pose could pass came to after its answer (`Engine::checked`, slice
	/// 42's fix round).
	pub(super) written: &'a [WrittenRow],
	pub(super) answers: &'a BTreeMap<usize, u8>,
	/// The cardio days its moves brought, by slot (`Run::days`), and the week's embers where one lit the day's last
	/// (`Run::kindled`).
	pub(super) days: &'a [(usize, CardioDay)],
	pub(super) kindled: Option<i64>,
	/// Today's embers on the sheet Hero shows after the pump (Pump home's count), for the cardio row; none where the pump
	/// lit none (`Run::embers_lit`).
	pub(super) embers: Option<i64>,
	/// Where Hero shows the game's sheet: its level with the other app pumps it has not counted yet before the pump
	/// (`Summary::uncounted_xp`), and with the pump's XP on that, all of which the game pays at its next load (the bar
	/// the Result fills until then; an expedition's from its first round's start to its last round's end, `join`).
	pub(super) projected: Option<(LevelAt, LevelAt)>,
	pub(super) check: bool,
	pub(super) checked: &'a BTreeMap<usize, Vec<CheckOutcome>>,
	/// Start a pump's A quest: its banner says Quest complete, every other app pump's Pump complete (2026-10-21).
	pub(super) quest: bool,
	/// A match pump's riders (slice 61; none on every other app pump), whose done ones follow its own moves' rows as a
	/// catch-up's Result draws them (`riders_moved`, their records as the engine judged them, `sets`), and how they were
	/// paid.
	pub(super) riders: &'a [PlannedMove],
	pub(super) sets: &'a BTreeMap<usize, CatchupSet>,
	pub(super) payment: Option<Payment>,
}

/// What a pump that turned its game day done reached by the streak's board (`Board::milestone`, `Board::rested`), as it
/// began and as it ended: the milestone (0 for none) and the rest tokens that covered the game days before it; a
/// paired player's app pump's and a catch-up's, whose rows the game folds at its next load (the phones'
/// `boardMilestone`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct Reached {
	milestone: i64,
	rested: i64,
}

impl Reached {
	pub(super) fn of(before: Option<&Board>, after: &Board) -> Self {
		before.map_or_else(Self::default, |b| Self { milestone: b.milestone(after), rested: b.rested(after) })
	}

	/// Its rows: the milestone (unless the banner carries it) and the rest tokens.
	fn rows(self, milestone: bool) -> Vec<Row> {
		let mut rows = Vec::new();
		if milestone && self.milestone > 0 {
			rows.push(Row { reward: Reward::Milestone { days: self.milestone }, tile: None });
		}
		if self.rested > 0 {
			rows.push(Row { reward: Reward::Day { line: "rested", value: self.rested }, tile: None });
		}
		rows
	}
}

/// The motion of the Result's one reveal (contract/design/tokens.json's `motion`) and the rows that sound, and a hold's
/// pour's (slice 65: `pour`, `pourFly` and `stepReveal`, `pour.rs`).
struct Motion {
	words: i64,
	fill: i64,
	shine: i64,
	stagger: i64,
	result: i64,
	pulse: i64,
	sounded: usize,
	pour: i64,
	pour_hold: i64,
	pour_base: i64,
	pour_step: i64,
	pour_fly: i64,
	reveal: i64,
	reveal_end: i64,
	reveal_from: i64,
}

static MOTION: LazyLock<Motion> = LazyLock::new(|| {
	let tokens: Value =
		serde_json::from_str(include_str!("../../../../../contract/design/tokens.json")).unwrap_or_default();
	let ms = |name: &str, key: &str| tokens["motion"][name][key].as_i64().unwrap_or(0);
	Motion {
		words: ms("fanfareWords", "ms"),
		fill: ms("xpFill", "ms"),
		shine: ms("chipShine", "ms"),
		stagger: ms("chip", "staggerMs"),
		result: ms("result", "ms"),
		pulse: ms("pulse", "ms"),
		sounded: tokens["surface"]["RewardRow"]["soundedRevealCount"]
			.as_u64()
			.and_then(|n| usize::try_from(n).ok())
			.unwrap_or(0),
		pour: ms("pour", "ms"),
		pour_hold: ms("pour", "holdMs"),
		pour_base: ms("pour", "baseMs"),
		pour_step: ms("pour", "stepMs"),
		pour_fly: ms("pourFly", "ms"),
		reveal: ms("stepReveal", "ms"),
		reveal_end: ms("stepReveal", "endMs"),
		reveal_from: ms("stepReveal", "revealFromMs"),
	}
});

/// The events of which a Result plays one: a later row whose jingle is taken plays `reveal`.
const JINGLES: [&str; 7] = ["record", "title", "unlockRare", "unlockEpic", "unlockLegendary", "levelUp", "milestone"];
/// From the strongest, for the one event a still Result plays.
const STRENGTH: [&str; 9] = [
	"levelUp",
	"milestone",
	"unlockLegendary",
	"unlockEpic",
	"unlockRare",
	"unlockUncommon",
	"unlock",
	"title",
	"record",
];

/// The Result's one reveal (`ResultReveal`): from its first frame (`at`) the fanfare, from `gain` the XP counting up as
/// the bar fills, the banner turning at `turn` (a level's or a rank's end of the fill, a streak milestone halfway), the
/// rows at `rows`, all over by `end`; the first rows sound their own event, one jingle a Result; `landed` stands every
/// part at its end. The page draws from these instants; the loop plays the sounds as they come due.
#[derive(Serialize, Clone, PartialEq)]
#[expect(clippy::struct_excessive_bools, reason = "the reveal's moments, each its own fact")]
pub(super) struct Reveal {
	at: i64,
	gain: i64,
	turn: Option<i64>,
	rows: Vec<i64>,
	end: i64,
	landed: bool,
	#[serde(skip)]
	sounds: Vec<(i64, &'static str)>,
	#[serde(skip)]
	turn_event: Option<&'static str>,
	#[serde(skip)]
	events: Vec<&'static str>,
	#[serde(skip)]
	jingled: bool,
	#[serde(skip)]
	turn_sounded: bool,
	#[serde(skip)]
	quiet: bool,
}

impl Reveal {
	/// The reveal of `earned` from `now`; `sounds` false for a catch-up's (the game pays it at its next load).
	pub(super) fn plan(now: i64, earned: &Earned, sounds: bool) -> Self {
		let m = &*MOTION;
		let gain = now + m.words / 2;
		let rows_at = gain - now + m.fill;
		let pulse_at = rows_at + m.shine;
		let n = earned.rows.len();
		let last_pulse = earned.rows.iter().rposition(|r| r.reward.pulses());
		let room = m.result - rows_at - m.shine;
		let mut stagger = m.stagger.min(room / i64::try_from(n.saturating_sub(1).max(1)).unwrap_or(1));
		if let Some(last) = last_pulse.filter(|&l| l > 0) {
			stagger = stagger.min(m.shine / i64::try_from(last).unwrap_or(1));
		}
		let stagger = stagger.max(0);
		let rows: Vec<i64> =
			(0..n).map(|i| now + rows_at + i64::try_from(i).unwrap_or(0) * stagger).collect::<Vec<_>>();
		let shines = rows.last().map_or(now + rows_at, |&last| last + m.shine);
		let end = if last_pulse.is_some() { shines.max(now + pulse_at + m.pulse) } else { shines };
		let (turn, turn_event) = match earned.turn {
			Some(Banner { kind: "streak", .. }) => (Some(gain + m.fill / 2), Some("milestone")),
			Some(_) => {
				let share = earned.bar.as_ref().and_then(|b| turn_share(b.from?, b.now)).unwrap_or(1.0);
				(Some(gain + run::millis(run::secs(m.fill) * share)), Some("levelUp"))
			}
			None => (None, None),
		};
		let events: Vec<&'static str> = earned.rows.iter().map(|r| r.reward.event()).collect();
		let mut timed: Vec<(i64, &'static str)> = turn.zip(turn_event).into_iter().collect();
		timed.extend(rows.iter().zip(&events).take(m.sounded).map(|(&at, &e)| (at, e)));
		timed.sort_by_key(|&(at, _)| at);
		Self {
			at: now,
			gain,
			turn,
			rows,
			end,
			landed: false,
			sounds: timed,
			turn_event,
			events,
			jingled: false,
			turn_sounded: false,
			quiet: !sounds,
		}
	}

	/// When the next sound comes due.
	pub(super) fn next_sound(&self) -> Option<i64> {
		self.sounds.first().map(|&(at, _)| at).filter(|_| !self.quiet)
	}

	/// The sounds due by `now`, one jingle a Result.
	pub(super) fn take_due(&mut self, now: i64) -> Vec<&'static str> {
		let due: Vec<&'static str> =
			self.sounds.iter().take_while(|&&(at, _)| at <= now).map(|&(_, e)| e).collect::<Vec<_>>();
		self.sounds.drain(..due.len());
		if self.quiet {
			return Vec::new();
		}
		due.into_iter().map(|e| self.fire(e)).collect()
	}

	/// An event as the one-jingle rule lets it play.
	fn fire(&mut self, event: &'static str) -> &'static str {
		if Some(event) == self.turn_event {
			self.turn_sounded = true;
		}
		let jingle = JINGLES.contains(&event);
		let played = if jingle && self.jingled { "reveal" } else { event };
		self.jingled |= jingle;
		played
	}

	/// Every part at its end now, with one event (`still`: the pump's one strongest; else the turn's where it has not
	/// sounded, else `reveal`), and nothing after.
	pub(super) fn land(&mut self, now: i64, still: bool) -> Option<&'static str> {
		if self.landed {
			return None;
		}
		self.landed = true;
		self.sounds.clear();
		self.end = self.end.min(now);
		if self.quiet {
			return None;
		}
		let event = if still {
			let all: Vec<&'static str> = self.turn_event.into_iter().chain(self.events.iter().copied()).collect();
			STRENGTH.into_iter().find(|e| all.contains(e)).unwrap_or("reveal")
		} else {
			self.turn_event.filter(|_| !self.turn_sounded).unwrap_or("reveal")
		};
		Some(self.fire(event))
	}
}

/// Where along the fill the banner turns (`XPBar.turn`): the share of the way from `from` to `to` at which the bar
/// reaches the old level's end; none without one.
fn turn_share(from: Point, to: Point) -> Option<f64> {
	let (start, end) = (from.place(), to.place());
	let edge = end.floor();
	(edge > start && end > start).then(|| (edge - start) / (end - start))
}

/// A count as the game's count step writes it (`Sheet.Amount`): a hold's seconds ("45 s"), else the number.
fn amount(value: i64, hold: bool) -> String {
	let n = value.to_string();
	if hold { words::fill(&facts().sheet.words.count.seconds, &[&n]) } else { n }
}

/// A move's tile: its name, key and quality.
fn tile(move_id: u16) -> Tile {
	Tile {
		name: facts().name(move_id).unwrap_or_default(),
		key: facts().text(move_id).map_or("", |t| t.key.as_str()),
		quality: facts().move_quality(move_id),
	}
}

/// The count to reach as planned: the target; none on a counted set, which reaches for no count (slice 42).
pub(super) fn goal(m: &PlannedMove) -> Option<i64> {
	if m.counted { None } else { m.target }
}

/// A loaded move (Progress.lua `ns.Loaded`): its sets take a weight, and the engine names its start range on every one.
pub(super) fn loaded(m: &PlannedMove) -> bool {
	m.start.is_some()
}

/// The moves a pump did with a weight, each once in the pump's order: the Result's "Next time" lines.
pub(super) fn weighed(moves: &[PlannedMove], recorded: &BTreeMap<usize, Recorded>) -> Vec<u16> {
	let mut ids: Vec<u16> = Vec::new();
	for m in moves {
		if matches!(recorded.get(&m.slot), Some(Recorded::Count(_, Some(_)))) && !ids.contains(&m.r#move) {
			ids.push(m.r#move);
		}
	}
	ids
}

/// Swap's options have one.
pub(super) fn any_option(o: &SwapOptions) -> bool {
	o.another.is_some() || o.easier.is_some() || o.backup.is_some() || o.back.is_some()
}

/// The first records the Result names with their next step (a first max set's count, a placement with no record before
/// it), as (slot, move, count); one done with a weight names its next set in its "Next time" line instead. A counted
/// set's first clean count is its first best (`counted`), never a goal (slice 42).
pub(super) fn firsts(
	moves: &[PlannedMove],
	recorded: &BTreeMap<usize, Recorded>,
	max_set: impl Fn(&PlannedMove) -> bool,
) -> Vec<(usize, u16, i64)> {
	moves
		.iter()
		.filter_map(|m| match recorded.get(&m.slot) {
			Some(&Recorded::Count(n, None))
				if n > 0
					&& m.record.is_none()
					&& max_set(m) && m.top.is_none_or(|top| n < top)
					&& m.placing.is_none() =>
			{
				Some((m.slot, m.r#move, n))
			}
			_ => None,
		})
		.collect()
}

/// An app pump's records (the phones' Result `records`): a count past its record (with a weight, as the engine judged
/// its row: a weighted record is a score); a first max set's count, its first record with the next step, or at its
/// rung's top enough for the next rung; a placement past the record that stays, or placed at or under it (with a
/// weight, as the engine judged it, the weighted record named where there is one). A counted set's clean count is its
/// best's, `counted`'s (slice 42).
fn records(p: &Sets<'_>, max_set: &impl Fn(&PlannedMove) -> bool) -> Vec<(usize, Reward)> {
	p.moves
		.iter()
		.filter_map(|m| {
			let Some(&Recorded::Count(n, load)) = p.recorded.get(&m.slot) else { return None };
			let (hold, each) = (m.kind == kind::HOLD, m.load_each);
			if n <= 0 {
				return None;
			}
			if let Some(record) = m.record {
				let beat = if load.is_some() { p.beat.get(&m.slot) == Some(&true) } else { n > record };
				let from = load.is_none().then_some(record);
				return beat.then_some((m.slot, Reward::Record { value: n, hold, load, each, from }));
			}
			if !max_set(m) {
				return None;
			}
			if let Some(top) = m.top.filter(|&top| n >= top) {
				return Some((m.slot, Reward::Enough { value: top, hold }));
			}
			// with a weight the engine's outcome says whether the placement beat the record (a weighted one is a score)
			let past = |stays: i64| if load.is_some() { p.beat.get(&m.slot) == Some(&true) } else { n > stays };
			let top = Lifted::of(m.top_load, m.top_reps).filter(|_| load.is_some());
			Some((
				m.slot,
				match m.placing {
					Some(stays) if past(stays) => {
						Reward::Record { value: n, hold, load, each, from: load.is_none().then_some(stays) }
					}
					Some(stays) => Reward::Placed {
						value: n,
						record: top.map_or(stays, |t| t.reps),
						hold,
						load,
						record_load: top.map(|t| t.load),
						each,
					},
					None => {
						let next = p.goals.iter().find(|&&(slot, _)| slot == m.slot).map(|&(_, g)| g);
						Reward::FirstRecord { value: n, hold, next, load, each }
					}
				},
			))
		})
		.collect()
}

/// A counted set's news (slice 42, the phones' Result): Form slipped or Not all out (slice 53), saved with no clean
/// day; else a clean count over the form's best (or its first) a new best, under it the best that stands; none for a
/// clean set not counted.
fn counted(m: &PlannedMove, recorded: &BTreeMap<usize, Recorded>) -> Option<Reward> {
	let Some(&Recorded::Counted(n, said)) = recorded.get(&m.slot) else { return None };
	Some(match m.record {
		_ if said == answer::SLIPPED => Reward::Slipped,
		_ if said == answer::NOT_ALL_OUT => Reward::NotAllOut,
		_ if n <= 0 => return None,
		Some(best) if n <= best => Reward::Best { count: best },
		_ => Reward::BestNew { count: n },
	})
}

/// The steps the pump's sets mastered (slice 42: the engine's mastered rows, `ns.Progression`'s), each with its ladder
/// and step and the step's tile (its rung's anchor, the tile its tree draws, which the row seals).
fn mastered(written: &[WrittenRow]) -> Vec<Row> {
	written
		.iter()
		.filter(|r| r.kind == kind::MASTERED)
		.filter_map(|r| {
			let step = i64::from(facts().step(r.ladder, r.rung)?);
			let reward = Reward::Mastered { ladder: facts().ladder_name(r.ladder), step };
			Some(Row { reward, tile: facts().anchor(r.ladder, r.rung).map(tile) })
		})
		.collect()
}

/// A passed check's row (slice 42's fix round, 2026-10-17, `ns.OpensNow` after the set): "Check passed: {move} is
/// open" with the move's tile, or the climb's, only where it is open now; else what still holds it in the wait's own
/// words, on the waiting move's tile where it opens a move.
fn passed_row(o: &CheckOutcome) -> Option<Row> {
	if !o.open {
		return Some(Row { reward: Reward::CheckHeld { wait: WaitWords::of(o.waits?)? }, tile: o.name.map(tile) });
	}
	if let (Some(ladder), Some(step)) = (o.ladder, o.step) {
		return Some(Row {
			reward: Reward::CheckClimb { ladder: facts().ladder_name(ladder), step: i64::from(step) },
			tile: None,
		});
	}
	let name = o.name?;
	Some(Row { reward: Reward::Check { r#move: facts().name(name)? }, tile: Some(tile(name)) })
}

/// The wait a held check's row words (`CheckHeld`), none for any other row.
fn held(row: &Row) -> Option<&WaitWords> {
	match &row.reward {
		Reward::CheckHeld { wait } => Some(wait),
		_ => None,
	}
}

/// The stretch checks' outcome (slice 42; on every pump's pose since slice 42's fix round, 2026-10-17, as the game's
/// `afterLines`): each check a pose's answer passed (`Engine::checked`), its row by what it opens (`passed_row`), each
/// row once and a wait's words once (slice 42's last review, 2026-10-18, as the game says them); the
/// stretch ladder a pose above its family's step lifted to a step (the engine's rung row with the check's cause); and
/// on a stretch check's pump whose pose was answered without passing the check it runs (the outcome marked `runs`,
/// `ns.RunsCheck`, slice 42's last review, 2026-10-18; any check, where none is marked), that the check waits.
fn checked(p: &Paid<'_>) -> Vec<Row> {
	let outcomes = || p.moves.iter().filter_map(|m| p.checked.get(&m.slot)).flatten();
	let mut rows: Vec<Row> = Vec::new();
	for row in outcomes().filter(|o| o.passed).filter_map(passed_row) {
		// each row once (an `any` group's two choices one pose answers open the same), and one pass may hold more than
		// one opening on the same wait (the 90-90's dragon pistol and squat step 12 on ankle step 6): its words once
		if !rows.contains(&row) && held(&row).is_none_or(|wait| rows.iter().all(|r| held(r) != Some(wait))) {
			rows.push(row);
		}
	}
	rows.extend(p.written.iter().filter(|r| r.kind == kind::RUNG && r.reps == cause::CHECK).filter_map(|r| {
		let step = i64::from(facts().step(r.ladder, r.rung)?);
		Some(Row { reward: Reward::CheckPlaced { ladder: facts().ladder_name(r.ladder), step }, tile: None })
	}));
	let answered = p.moves.iter().any(|m| m.ask.is_some() && p.answers.contains_key(&m.slot));
	let marked = outcomes().any(|o| o.runs);
	if p.check && answered && !outcomes().any(|o| o.passed && (o.runs || !marked)) {
		rows.push(Row { reward: Reward::CheckWaits, tile: None });
	}
	rows
}

/// A cardio day a move brought (`Engine::finish`'s, Progress.lua `ns.CardioDay`, as the game's `afterLines` reads
/// `Lc.WriteResult`'s), on its move's row: a clean day (slice 42) or a full move (slice 52) toward the next step (the
/// last one too, beside the unlock's row where it completed the climb) and a Long Haul's day (slice 55), each a step
/// with its days (`true`); a measure back from a break holding the step or going back one (slice 53) and a proving
/// window that did not pass, each a note.
fn cardio_day(day: CardioDay) -> Option<(Reward, bool)> {
	Some(match day {
		CardioDay::Clean { have, need, step } => (Reward::CleanDay { have, need, step }, true),
		CardioDay::Full { have, need, step } => (Reward::FullMove { have, need, step }, true),
		CardioDay::Haul { time, have, need, next } => (Reward::HaulDay { time, have, need, next }, true),
		CardioDay::Holds { step } => (Reward::Return { back: false, step }, false),
		CardioDay::Back { step } => (Reward::Return { back: true, step }, false),
		CardioDay::Waits => (Reward::ClimbWaits, false),
		CardioDay::Other => return None,
	})
}

/// What the Result reads of a pump's sets, an app pump's and a catch-up's alike: its moves, what each recorded and ran,
/// whether a set with a weight was a record (the engine's judgement) and a first record's next step, by slot, each move
/// done with a weight's next fresh set, the cardio days its moves brought, by slot, and the app's own sheet after it
/// (each ladder's days at the top; none for a catch-up, whose rows the game folds).
struct Sets<'a> {
	moves: &'a [PlannedMove],
	recorded: &'a BTreeMap<usize, Recorded>,
	ran: &'a BTreeMap<usize, i64>,
	beat: &'a BTreeMap<usize, bool>,
	goals: &'a [(usize, i64)],
	nexts: &'a [(u16, NextLoad)],
	days: &'a [(usize, CardioDay)],
	after: Option<&'a Sum>,
}

/// The moves a pump did, in its order.
fn did<'a>(
	moves: &'a [PlannedMove],
	recorded: &'a BTreeMap<usize, Recorded>,
	ran: &'a BTreeMap<usize, i64>,
) -> impl Iterator<Item = &'a PlannedMove> {
	moves.iter().filter(|m| recorded.contains_key(&m.slot) || ran.contains_key(&m.slot))
}

/// The record a record of `m` passed: a set with a weight's weighted record with its weight, else its count before the
/// pump or the record a placement stood under; none for a first.
fn was(m: &PlannedMove, weighed: bool) -> Option<Was> {
	let hold = m.kind == kind::HOLD;
	if let Some(top) = Lifted::of(m.top_load, m.top_reps).filter(|_| weighed) {
		return Some(Was { value: top.reps, hold, load: Some(top.load), each: m.load_each });
	}
	m.record.or(m.placing).map(|value| Was { value, hold, load: None, each: false })
}

/// On the move that stands for its strength ladder's step on the app's own sheet (its `lad` move), the game days at the
/// top of its range toward the next step (`nx`, what the Book's ladder counts), while the climb is open and its days
/// are not all in.
fn top_day(m: &PlannedMove, after: &Sum) -> Option<Reward> {
	let ladder = facts().library.moves.get(&m.r#move)?.ladder?;
	if !facts().library.strength_order.contains(&ladder) {
		return None;
	}
	let i = usize::from(ladder).checked_sub(1)?;
	let lad = after.lad.get(i)?;
	if lad.get(1).copied() != Some(i64::from(m.r#move)) {
		return None;
	}
	let (have, need) = match after.nx.get(i)?.as_slice() {
		&[have, need, ..] if need > 0 && have < need => (have, need),
		_ => return None,
	};
	let rung = u8::try_from(lad.first()?.checked_add(1)?).ok()?;
	Some(Reward::TopDay { have, need, step: i64::from(facts().step(ladder, rung)?) })
}

/// Each move the pump did as one row on its tile, in the pump's order (2026-10-29, Rapha: "something indicating
/// progress or if we hit new record"): what it did (its count, a hold's as a clock, with the weight it was done with, a
/// timed move's seconds, or a check); its record (`records`: a record's tag with the record it passed, a first record's
/// with its next goal or enough for the next step, a placement's words), a counted set's (a new best a record's tag,
/// the best that stands, Form slipped or Not all out), the cardio day it brought (a step with its days, or a measure's
/// note), its next weight and the days at the top of its ladder's step (`top_day`).
fn moved(p: &Sets<'_>, max_set: &impl Fn(&PlannedMove) -> bool) -> Vec<Row> {
	let records = records(p, max_set);
	did(p.moves, p.recorded, p.ran)
		.map(|m| {
			let (count, load, time) = match p.recorded.get(&m.slot) {
				Some(&Recorded::Count(n, load)) => (Some(n), load, None),
				Some(&Recorded::Counted(n, _)) => (Some(n).filter(|&n| n > 0), None, None),
				Some(&Recorded::Time(s)) => (None, None, Some(s)),
				Some(Recorded::Done) => (None, None, None),
				None => (None, None, p.ran.get(&m.slot).copied().filter(|_| m.kind >= kind::CARDIO)),
			};
			let (mut tag, mut was_then, mut notes, mut steps) = (None, None, Vec::new(), Vec::new());
			match records.iter().find(|(slot, _)| *slot == m.slot).map(|(_, r)| r) {
				Some(Reward::Record { .. }) => (tag, was_then) = (Some("record"), was(m, load.is_some())),
				Some(first @ (Reward::FirstRecord { .. } | Reward::Enough { .. })) => {
					tag = Some("first");
					notes.push(first.clone());
				}
				Some(other) => notes.push(other.clone()),
				None => {}
			}
			match counted(m, p.recorded) {
				Some(Reward::BestNew { count }) => {
					tag = Some("record");
					was_then = m.record.filter(|&best| best < count).map(|value| Was {
						value,
						hold: false,
						load: None,
						each: false,
					});
				}
				Some(note) => notes.push(note),
				None => {}
			}
			for &(_, day) in p.days.iter().filter(|(slot, _)| *slot == m.slot) {
				match cardio_day(day) {
					Some((reward, true)) if !steps.contains(&reward) => steps.push(reward),
					Some((reward, false)) if !notes.contains(&reward) => notes.push(reward),
					_ => {}
				}
			}
			steps.extend(p.after.and_then(|after| top_day(m, after)));
			let again = load
				.and(p.nexts.iter().find(|(id, _)| *id == m.r#move))
				.map(|&(_, n)| Again { load: n.load, target: n.target });
			let reward = Reward::Moved {
				count,
				clock: m.kind == kind::HOLD,
				time,
				load,
				each: m.load_each,
				tag,
				was: was_then,
				notes,
				again,
				steps,
			};
			Row { reward, tile: Some(tile(m.r#move)) }
		})
		.collect()
}

/// Today's embers with their pips (Pump home's count), on a pump that lit one (`have`, none where it lit none: a
/// cardio move that lights no ember shows no "0 of 3", Rapha's Pixel check of 2026-10-29): `have` of the `embers_day` a
/// day lights.
fn cardio(have: Option<i64>) -> Option<Row> {
	let have = have?;
	let need = facts().constants().embers_day;
	Some(Row { reward: Reward::Cardio { have: have.clamp(0, need), need }, tile: None })
}

/// What an app pump earned as the app's own sheet paid it: the XP, the level it turned (on the app's own sheet where
/// Hero shows it, else on the game's with the app pumps it has not counted yet and the pump's XP on it, which the game
/// pays at its next load), then the rows: each move of its own it did (`moved`), each rider done (a match pump's,
/// slice 61: `riders_moved`, with how they were paid) and a streak milestone the banner does not carry, the stars, the
/// titles earned since it began and the weekly vault's goals it reached (both where Hero shows the app's own sheet),
/// the unlocks, the stats raised in one row (or the points together), the cardio done today and the day's lines, in
/// the game's order.
pub(super) fn earned(p: &Paid<'_>, max_set: impl Fn(&PlannedMove) -> bool) -> Earned {
	let sets = Sets {
		moves: p.moves,
		recorded: p.recorded,
		ran: p.ran,
		beat: p.beat,
		goals: p.goals,
		nexts: p.nexts,
		days: p.days,
		after: p.after,
	};
	let mut extra = moved(&sets, &max_set);
	extra.extend(riders_moved(p.riders, p.recorded, p.ran, p.sets, &max_set));
	// slice 42: a stretch check's outcome and the steps mastered
	extra.extend(checked(p));
	extra.extend(mastered(p.written));
	extra.extend(cardio(p.embers));
	let banner = Some(Banner { kind: if p.quest { "quest" } else { "pump" }, number: 0 });
	let Some(after) = p.after else {
		return Earned {
			banner,
			turn: None,
			xp: None,
			bar: None,
			rows: extra,
			paid: p.payment.map(paid_word),
			embers: p.kindled.map(Kindled::of),
		};
	};
	let bar = if p.hero_own {
		Some(Bar { now: Point::of(after), rested: after.rest, from: p.before.map(Point::of) })
	} else {
		p.projected.map(|(from, to)| Bar { now: Point::at(to), rested: 0, from: Some(Point::at(from)) })
	};
	let turn = bar.as_ref().and_then(|b| {
		let (from, to) = (b.from?, b.now);
		if to.paragon > from.paragon {
			Some(Banner { kind: "paragon", number: to.paragon })
		} else if to.level > from.level {
			Some(Banner { kind: "level", number: to.level })
		} else {
			None
		}
	});
	let milestone = if p.hero_own { after.day(4) } else { p.reached.milestone };
	let streak = milestone > 0 && turn.is_none();
	let mut rows = Vec::new();
	if milestone > 0 && !streak {
		rows.push(Row { reward: Reward::Milestone { days: milestone }, tile: None });
	}
	if after.last(2) > 0 {
		rows.push(Row { reward: Reward::Star { count: after.last(2) }, tile: None });
	}
	if p.hero_own {
		// the titles the app's fold holds now and did not as the pump began
		for &code in p.titles_after.iter().filter(|c| !p.titles_before.contains(c)) {
			if let Some(title) = facts().title(code).map(|t| t.name.as_str()) {
				rows.push(Row { reward: Reward::Title { title }, tile: None });
			}
		}
		// the weekly vault's goals it reached, the app's own sheet's week (a paired player's is the game's)
		rows.extend(after.lv.iter().filter_map(|&goal| vault(goal, after)));
	}
	rows.extend(unlocks(after, None).map(|(_, row)| row));
	if !after.ls.is_empty() {
		rows.extend(stats(after, None));
	} else if after.last(1) > 0 {
		rows.push(Row { reward: Reward::Points { count: after.last(1) }, tile: None });
	}
	if p.hero_own {
		extra.extend(day(after));
	} else {
		// the milestone is the banner's or a row of its own above; the rest tokens the board spent
		extra.extend(p.reached.rows(false));
	}
	rows.extend(extra);
	rows.sort_by_key(|row| row.reward.rank());
	Earned {
		banner,
		turn: if streak { Some(Banner { kind: "streak", number: milestone }) } else { turn },
		xp: Some(after.last(0)),
		bar,
		rows,
		paid: p.payment.map(paid_word),
		embers: p.kindled.map(Kindled::of),
	}
}

/// An expedition's Result (slice 51): every round's Result together, `minutes` the minutes it took on a whole one
/// (the Fanfare's "Timed pump complete"; none on one ended part way, which ends calm). The XP of every round, the bar
/// from the first round's start to the last's end with the banner turning at a level or a Paragon rank they crossed,
/// else at a streak milestone a round reached; one row a reward: a round's stars, stats and points summed, a day's
/// line once (the latest round's), every other row as its round gave it (each move done, `Reward::Moved`, among
/// them), in the game's order; and the "Three embers" Toast of the last round that lit one.
pub(super) fn join(rounds: &[Earned], minutes: Option<i64>) -> Earned {
	let mut rows: Vec<Row> = Vec::new();
	for row in rounds.iter().flat_map(|e| e.rows.iter().cloned()) {
		let same = rows.iter_mut().find(|kept| match (&kept.reward, &row.reward) {
			(Reward::Star { .. }, Reward::Star { .. }) | (Reward::Points { .. }, Reward::Points { .. }) => true,
			(Reward::Stat { key: a, .. }, Reward::Stat { key: b, .. })
			| (Reward::Day { line: a, .. }, Reward::Day { line: b, .. }) => a == b,
			_ => false,
		});
		match (same, &row.reward) {
			(Some(kept), Reward::Star { count: more } | Reward::Points { count: more }) => {
				if let Reward::Star { count } | Reward::Points { count } = &mut kept.reward {
					*count += more;
				}
			}
			(Some(kept), Reward::Stat { amount: more, .. }) => {
				if let Reward::Stat { amount, .. } = &mut kept.reward {
					*amount += more;
				}
			}
			(Some(kept), Reward::Day { .. }) => *kept = row,
			_ => rows.push(row),
		}
	}
	rows.sort_by_key(|row| row.reward.rank());
	let first = rounds.iter().find_map(|e| e.bar.as_ref());
	let last = rounds.iter().rev().find_map(|e| e.bar.as_ref());
	let bar = first.zip(last).map(|(f, l)| Bar { now: l.now, rested: l.rested, from: f.from });
	let turn = bar
		.as_ref()
		.and_then(|b| {
			let from = b.from?;
			if b.now.paragon > from.paragon {
				Some(Banner { kind: "paragon", number: b.now.paragon })
			} else if b.now.level > from.level {
				Some(Banner { kind: "level", number: b.now.level })
			} else {
				None
			}
		})
		.or_else(|| rounds.iter().rev().find_map(|e| e.turn.filter(|t| t.kind == "streak")));
	let xp = rounds.iter().filter_map(|e| e.xp).reduce(|a, b| a + b);
	Earned {
		banner: minutes.map(|number| Banner { kind: "expedition", number }),
		turn: minutes.and(turn),
		xp,
		bar,
		rows,
		paid: None,
		embers: rounds.iter().rev().find_map(|e| e.embers),
	}
}

/// The unlocks of the summary's last pump (`lu`) as rows, with each one's numbers; with `before` (a summary of the
/// same pump before one save, slice 65's pour) only the ones that save brought.
fn unlocks<'a>(after: &'a Sum, before: Option<&'a Sum>) -> impl Iterator<Item = (&'a [i64], Row)> + 'a {
	after
		.lu
		.iter()
		.filter(move |u| before.is_none_or(|b| !b.lu.contains(u)))
		.filter_map(|u| Some((u.as_slice(), unlock(u)?)))
}

/// The stats the summary's last pump raised (`ls`) in one row of chips; with `before` (as `unlocks`) only what one
/// save raised.
fn stats(after: &Sum, before: Option<&Sum>) -> Option<Row> {
	let was = |i: usize| before.and_then(|b| b.ls.get(i)).copied().unwrap_or(0);
	let stats: Vec<Reward> = facts()
		.sheet
		.stats
		.iter()
		.zip(&after.ls)
		.enumerate()
		.map(|(i, (s, &amount))| (s, amount - was(i)))
		.filter(|&(_, amount)| amount > 0)
		.map(|(s, amount)| Reward::Stat { amount, key: &s.key, stat: &s.name })
		.collect();
	(!stats.is_empty()).then_some(Row { reward: Reward::Stats { stats }, tile: None })
}

/// A weekly vault goal's row (`lv`: 1 moves done, 2 records or stars, 3 an embers tier, slice 55) with the goal it
/// is: an embers tier's the highest of `vault_embers` the week's embers (the summary's `va`) reached.
fn vault(goal: i64, after: &Sum) -> Option<Row> {
	let c = facts().constants();
	let (goal, count) = match goal {
		1 => ("moves", c.vault_moves),
		2 => ("records", c.vault_records),
		3 => {
			let week = after.va.get(2).copied().unwrap_or(0);
			("embers", c.vault_embers.iter().copied().filter(|&t| t <= week).max()?)
		}
		_ => return None,
	};
	Some(Row { reward: Reward::Vault { goal, count }, tile: None })
}

/// An unlock's row (`RewardRow.Unlock`): the move that stands for the step now (its ladder in its place where none
/// does), its ladder and step on the one scale with the quality's word, and how it came (`Reward::quiet`; the sixth
/// number since 2026-10-09, an older sheet's five read as a climb).
fn unlock(u: &[i64]) -> Option<Row> {
	let [ladder, rung, move_id, quality, rekindled, ref rest @ ..] = *u else { return None };
	let (ladder, rung) = (u8::try_from(ladder).ok()?, u8::try_from(rung).ok()?);
	let quality = u8::try_from(quality).ok()?;
	let way = match rest.first() {
		Some(1) => "opened",
		Some(2) => "earned",
		_ => "climb",
	};
	let reward = Reward::Unlock {
		ladder: facts().ladder_name(ladder),
		step: i64::from(facts().step(ladder, rung)?),
		quality,
		word: facts().quality_word(quality)?,
		rekindled: rekindled == 1,
		way,
	};
	let tile = u16::try_from(move_id).ok().filter(|&m| m > 0).map(|m| Tile { quality: Some(quality), ..tile(m) });
	Some(Row { reward, tile })
}

/// An app pump's day as its own sheet counted it (`SheetSummary.LastDay`): Welcome back's XP, else the streak's day
/// (a new streak's first; none on a milestone's day), the rest tokens that covered the days before it, a token earned.
fn day(s: &Sum) -> Vec<Row> {
	let (streak, welcome, token, rested, milestone) = (s.day(0), s.day(1), s.day(2), s.day(3), s.day(4));
	let mut lines = Vec::new();
	if welcome > 0 {
		lines.push(("welcome", welcome));
	} else if streak > 1 && milestone == 0 {
		lines.push(("streak", streak));
	} else if streak == 1 {
		lines.push(("streak_new", 0));
	}
	if rested > 0 {
		lines.push(("rested", rested));
	}
	if token > 0 {
		lines.push(("token", 0));
	}
	lines.into_iter().map(|(line, value)| Row { reward: Reward::Day { line, value }, tile: None }).collect()
}

/// A catch-up's payment, as the page words it.
fn paid_word(paid: Payment) -> &'static str {
	match paid {
		Payment::Paid => "paid",
		Payment::Unpaid => "unpaid",
		Payment::Reset => "reset",
		Payment::Locked => "locked",
		Payment::GameRuns => "game",
	}
}

/// A catch-up's end: each move it did with its record (a count past the move's record, as the engine judged it, or its
/// first) and its counted set's news, the streak milestone and the rest tokens its game day reached (the game counts
/// them at its next load), today's embers with the ones its cardio moves lit where they lit one (`Run::embers_lit` on
/// the count as it began, as the phone's catch-up counts them; 2026-10-29), and how they were paid; no XP and no
/// embers Toast (its sets are judged, never written by `Engine::finish`, and the game pays them at its next load).
pub(super) fn caught_up(run: &Run) -> Earned {
	let paid = run.paid.map(paid_word);
	let moves = riders_moved(&run.moves, &run.recorded, &run.ran, &run.sets, &|m: &PlannedMove| run.max_set(m));
	let count = i64::try_from(moves.len()).unwrap_or(0);
	// the milestone leads, as on an app pump's Result
	let mut rows = run.reached.rows(true);
	rows.extend(moves);
	rows.extend(cardio((run.embers_lit > 0).then_some(run.embers_before + run.embers_lit)));
	rows.sort_by_key(|row| row.reward.rank());
	Earned {
		banner: Some(Banner { kind: "moves", number: count }),
		turn: None,
		xp: None,
		bar: None,
		rows,
		paid,
		embers: None,
	}
}

/// The rows of the moves a pump paid as owed rows (`riders`: a catch-up's moves, a match pump's riders, slice 61), in
/// its order: each done as `moved` draws it, its record as the engine judged it (`sets`), with no goal, next weight,
/// cardio day or step days (its rows are judged, never written by `Engine::finish`, and the game folds them at its next
/// load).
fn riders_moved(
	riders: &[PlannedMove],
	recorded: &BTreeMap<usize, Recorded>,
	ran: &BTreeMap<usize, i64>,
	sets: &BTreeMap<usize, CatchupSet>,
	max_set: &impl Fn(&PlannedMove) -> bool,
) -> Vec<Row> {
	let beat: BTreeMap<usize, bool> = sets.iter().map(|(&slot, set)| (slot, set.outcome == outcome::BEAT)).collect();
	let sets = Sets { moves: riders, recorded, ran, beat: &beat, goals: &[], nexts: &[], days: &[], after: None };
	moved(&sets, max_set)
}

/// A move's target as its Ready view shows it (`PumpRunner.target(of:)`): a counted set all out over its seconds (slice
/// 42, no number to reach); with no count to reach, a max set where the engine dosed it, else one set (a catch-up's
/// move that asks no count).
fn target(run: &Run, m: &PlannedMove) -> TargetView {
	if m.counted {
		return TargetView { kind: "allout", value: m.seconds, set: None };
	}
	if let Some(limit) = MoveRun::new(m.kind, m.seconds, m.side).limit() {
		return TargetView { kind: "time", value: Some(run::whole(limit)), set: None };
	}
	if let Some(target) = m.target {
		let shown = if m.kind == kind::HOLD { "time" } else { "goal" };
		return TargetView { kind: shown, value: Some(target), set: m.today.map(|n| n + 1) };
	}
	TargetView { kind: if run.dosed(m) { "max" } else { "set" }, value: None, set: None }
}

/// A first max set's line (Panel.lua `goalOf`, the game's words): stop one rep before the form breaks, with the count
/// that is enough for the next rung where its rung has a top; a placement's that the record stays.
fn first_line(m: &PlannedMove) -> String {
	let w = &facts().sheet.words.target;
	let hold = m.kind == kind::HOLD;
	if let Some(record) = m.placing {
		return words::fill(&w.placing, &[&amount(record, hold)]);
	}
	match m.top {
		Some(top) => words::fill(if hold { &w.first_hi_hold } else { &w.first_hi }, &[&amount(top, hold)]),
		None => (if hold { &w.first_hold } else { &w.first }).clone(),
	}
}

/// A move's first appearance (never done, `seen` false, or a placement): its Ready view shows the right-versus-wrong
/// pair and its swap row offers the easier version before the backup.
fn new_move(m: &PlannedMove, seen: bool) -> bool {
	m.placing.is_some() || !seen
}

/// Move `i` as its stages show it; `counts`: the player's switches let the voice count a set of reps
/// (`Desk::counts_chosen`).
fn move_view(run: &Run, i: usize, m: &PlannedMove, seen: bool, counts: bool) -> MoveView {
	let text = facts().text(m.r#move);
	let hold = m.kind == kind::HOLD;
	let state = match run.recorded.get(&m.slot) {
		Some(_) => "done",
		None if run.ran.contains_key(&m.slot) && run.rider(m.slot) => "done",
		None if run.stage == Stage::Done && (i >= run.current || !run.ran.contains_key(&m.slot)) => "missed",
		None => "pending",
	};
	let note = (i == run.current && run.stage == Stage::Ready && run.max_set(m)).then(|| first_line(m));
	let quality = facts().move_quality(m.r#move);
	MoveView {
		slot: m.slot,
		move_id: m.r#move,
		rider: run.rider(m.slot),
		key: text.map_or("", |t| t.key.as_str()),
		implement: m.implement.clone(),
		name: facts().name(m.r#move).unwrap_or_default(),
		quality,
		word: quality.and_then(|q| facts().quality_word(q)),
		kind: m.kind,
		hold,
		target: target(run, m),
		record: m.record,
		load: loaded(m).then(|| LoadView {
			plan: m.load,
			up: m.load_up,
			each: m.load_each,
			top: Lifted::of(m.top_load, m.top_reps),
		}),
		mark: if !run.rider(m.slot) && m.mark { facts().mark(m.r#move) } else { None },
		note,
		sides: m.side,
		allout: m.allout,
		counted: m.counted,
		seconds: m.seconds,
		build: m.build,
		state,
		stop: text.and_then(|t| t.stop.as_deref()),
		first: run.stage == Stage::Ready && new_move(m, seen),
		beat: rep_beat(m),
		pace: pace_rung(run, i, m),
		rep_count: rep_count(m).filter(|_| counts),
	}
}

/// A steady cardio move's cadence rung (`cadence_rung` of its cardio rung, an app pump's from its slot, a catch-up's
/// from its owed row), the move at position `i` of the pump: its pace's words on Ready and Running, as the voice reads
/// them at the go (`run.rungs` by position, never by the move's `slot`, which a catch-up's Skip leaves at its first
/// place while it takes the move out of `moves` and `rungs`), a match pump's warm move the stand-up's easy cadence
/// (`EASY_CADENCE`), a Long Haul rung 1's (`HAUL_CADENCE`, slice 55); none for any other move, an all-out window
/// included.
fn pace_rung(run: &Run, i: usize, m: &PlannedMove) -> Option<u8> {
	(m.kind == kind::CARDIO && !m.allout).then(|| {
		if m.warm {
			EASY_CADENCE
		} else if m.haul {
			HAUL_CADENCE
		} else {
			cadence_rung(run.rungs.get(i).copied().unwrap_or(1))
		}
	})
}

/// The lines on the Ready view about the move on (Panel.lua `goalOf`, in its order), a steady move's pace in
/// words (`pace`, slice 42's fix round), a pose's family's days (`days`, slice 47) and the next-step line where the
/// climb waits on the move's stage of its path alone (slice 42, `next_line`).
fn lines(m: &PlannedMove, pace: Option<u8>, days: Option<Line>) -> Vec<Line> {
	let mut lines = goal_lines(m);
	lines.extend(pace.map(|rung| Line::Pace { rung }));
	lines.extend(days);
	lines.extend(next_line(m));
	lines
}

/// The day note under a pose's goal pose on its Ready view (slice 47, Panel.lua `poseDaysLine`): where its answer
/// counts its family's day toward the step above ([`Toward::Step`] with what the family's days lack, as its Ready view
/// read it, `Run::ready_toward`) and those days are not in yet, the days so far and what they still lack, the step's
/// main pose named (`mainPose`, else the anchor of the rung its slot is dosed at, as Tree.lua `Tree.PoseShort` names
/// it); none elsewhere, on a pose of the step's path and before the engine answered.
fn pose_days(run: &Run, m: &PlannedMove) -> Option<Line> {
	let (slot, mv, toward) = run.ready_toward?;
	let Toward::Step { have, need, main: Some(main), perfect: Some(perfect), main_pose, .. } = toward else {
		return None;
	};
	if (slot, mv) != (m.slot, m.r#move) || have >= need {
		return None;
	}
	let anchor = || run.slots.get(m.slot)?.as_ref().and_then(|s| facts().anchor(s.ladder, s.rung));
	let pose = if main > 0 { main_pose.or_else(anchor).and_then(|id| facts().name(id)) } else { None };
	Some(Line::PoseDays {
		perfect: m.ask == Some(mark::PERFECT),
		days: have,
		total: need,
		short: Short { main, pose, perfect },
	})
}

/// A goal's amount in the game's goal row (Panel.lua `goalOf`): "20+ reps each side", a hold's "45+ seconds", with a
/// loaded climb's weight ("… with 15 kg or more").
fn goal_amount(m: &PlannedMove, top: i64, kg: Option<i64>, each: bool) -> String {
	let w = &facts().sheet.words.goal;
	let one = top == 1;
	let unit = match (m.kind == kind::HOLD, m.side) {
		(true, true) => &w.secs_side,
		(true, false) => &w.secs,
		(false, true) if one => &w.reps_side_one,
		(false, true) => &w.reps_side,
		(false, false) if one => &w.reps_one,
		(false, false) => &w.reps,
	};
	let amount = format!("{} {unit}", words::fill(&w.at_least, &[&top.to_string()]));
	match kg {
		Some(kg) => words::fill(if each { &w.climb_each } else { &w.climb }, &[&amount, &kg.to_string()]),
		None => amount,
	}
}

/// The next-step line under the goal (Panel.lua `nextStepLine`, slice 42; `PlannedMove::next`, Progress.lua
/// `ns.PathDue`): what opens the next step on this move and the game days at it still needed, in the game's words (a
/// pose's days close enough, or Perfect where only Perfect counts; a counted form's clean days); none where the climb
/// waits on more than this move's stage.
fn next_line(m: &PlannedMove) -> Option<Line> {
	let next = m.next?;
	let w = &facts().sheet.words.target;
	let days = next.left.to_string();
	let pick = |many: &String, one: &String| if next.left == 1 { one.clone() } else { many.clone() };
	let text = if m.counted {
		words::fill(&pick(&w.next_clean, &w.next_clean_one), &[&days])
	} else if m.kind == kind::FLOW {
		let perfect = matches!(next.top, Top::Count(ask) if ask >= i64::from(mark::PERFECT));
		let (many, one) =
			if perfect { (&w.next_perfect, &w.next_perfect_one) } else { (&w.next_pose, &w.next_pose_one) };
		words::fill(&pick(many, one), &[&days])
	} else {
		let Top::Count(top) = next.top else { return None };
		words::fill(&pick(&w.next, &w.next_one), &[&goal_amount(m, top, next.kg, next.each), &days])
	};
	Some(Line::Words { text })
}

/// The lines about a move's goal on Ready (Panel.lua `goalOf`, in its order): a counted set's goal, all out with clean
/// reps and no number to reach, its best clean count once it has one, and the game's line of its window (slice 42); a
/// timed move's, a carry's and a side flow move's lines, the player's form a goal reaches at most and a loaded move's
/// weight line (a side move's record and goal each side stand in its goal and record, `MoveView`, as the phones').
fn goal_lines(m: &PlannedMove) -> Vec<Line> {
	let w = &facts().sheet.words.target;
	let c = facts().constants();
	if m.counted {
		let how = words::fill(&w.count, &[&c.count_s.to_string()]);
		let best = m.record.map(|count| Line::Best { count });
		return [Line::AlloutClean].into_iter().chain(best).chain([Line::Words { text: how }]).collect();
	}
	match m.kind {
		kind::CARDIO => {
			let (Some(seconds), Some(build), true) = (m.seconds, m.build, m.allout) else { return Vec::new() };
			vec![Line::Words { text: words::fill(&w.allout, &[&build.to_string(), &(seconds - build).to_string()]) }]
		}
		kind::FLOW => {
			let Some(each) = MoveRun::new(m.kind, m.seconds, m.side).limit().map(run::whole) else {
				return Vec::new();
			};
			match (m.side, words::carry(m.r#move)) {
				(true, true) => vec![Line::CarrySide { seconds: each }],
				(false, true) => vec![Line::Carry { seconds: each }],
				(true, false) => vec![Line::FlowSide { seconds: each }],
				(false, false) => Vec::new(),
			}
		}
		kind::HOLD | kind::REP => form_line(m).into_iter().chain(load_line(m)).collect(),
		_ => Vec::new(),
	}
}

/// The player's form a goal from it reaches at most (Panel.lua `recordLine`): their best lately (`ns.Form`; a loaded
/// move's with a weight planned at its weight, `ns.LoadForm`); none for any other goal (since 2026-10-22 a repeat's
/// lighter goal says no why) or a step up (whose own line names the weight, `load_line`).
fn form_line(m: &PlannedMove) -> Option<Line> {
	if m.load_up || m.goal? != Goal::Form {
		return None;
	}
	Some(Line::Form { form: m.form?, hold: m.kind == kind::HOLD, load: m.form_load, each: m.load_each })
}

/// A loaded move's line on its Ready view: the start range while no weight was logged, or when the plan steps up
/// the top reached twice at the working weight it steps up from ("Heavier: 20 reps twice with 10 kg").
fn load_line(m: &PlannedMove) -> Option<Line> {
	let [lo, hi] = m.start?;
	match (m.load, m.load_top, m.load_from) {
		(None, ..) => Some(Line::LoadStart { lo, hi, each: m.load_each }),
		(Some(_), Some(reps), Some(load)) if m.load_up => Some(Line::LoadHeavier { reps, load }),
		_ => None,
	}
}

/// What the move up next pays on its Ready view, as the game's rewards line names it (Panel.lua `rewardsLine`, Stats.lua
/// ns.RewardPreview of its slot; the whole pump's on its quest page until 2026-10-24): an app pump's XP for the move
/// done, the stat its record would raise with the count it has to beat (the game's chip "+1 Strength" over "if you
/// beat 12") and a record to set where its goal is a record attempt (the engine's kind: a weighted record is a score);
/// none on a rider, which pays as an owed row (a catch-up's move, a match pump's rider, slice 61).
fn rewards(run: &Run) -> Vec<Pill> {
	let (Some(p), Some(m)) = (run.previews.get(run.current), run.moves.get(run.current)) else { return Vec::new() };
	if run.rider(m.slot) {
		return Vec::new();
	}
	let mut pills = Vec::new();
	if p.xp > 0 {
		pills.push(Pill::Xp { amount: p.xp });
	}
	let stat = p.stat.as_deref().and_then(|key| facts().sheet.stats.iter().find(|s| s.key == key));
	if let (Some(s), Some(amount), Some(beat)) = (stat, p.amount.filter(|&a| a > 0), p.beat) {
		pills.push(Pill::Stat { amount, key: &s.key, stat: &s.name, beat, hold: m.kind == kind::HOLD });
	}
	if m.goal == Some(Goal::Record) {
		pills.push(Pill::Record);
	}
	pills
}

/// Swap's offers in the game's order (Panel.lua `SWAP_ORDER`: the way back, the backup, the easier version, another
/// move; on the move's `first` appearance the easier version before the backup, `SWAP_FIRST`), each in the game's
/// words (`swapWords`: the backup below the move's level "Closest your setup allows").
fn swap_view(o: &SwapOptions, first: bool) -> SwapView {
	let w = &facts().sheet.words.swap;
	let option = |take: &'static str, s: &Slot, text: &str| {
		let name = facts().name(s.r#move).unwrap_or_default();
		SwapOption {
			take,
			move_id: s.r#move,
			key: facts().text(s.r#move).map_or("", |t| t.key.as_str()),
			name,
			quality: facts().move_quality(s.r#move),
			line: words::fill(text, &[name]),
			kind: words::fill(text, &[""])
				.trim_matches(|c: char| c.is_whitespace() || c.is_ascii_punctuation())
				.to_owned(),
		}
	};
	let backup = o.backup.as_ref().map(|s| option("backup", s, if s.lower { &w.closest } else { &w.backup }));
	let easier = o.easier.as_ref().map(|s| option("easier", s, &w.easier));
	let (second, third) = if first { (easier, backup) } else { (backup, easier) };
	let (back, another) = (
		o.back.as_ref().map(|s| option("back", s, &w.back)),
		o.another.as_ref().map(|s| option("another", s, &w.another)),
	);
	let options = [back, second, third, another].into_iter().flatten().collect();
	SwapView { title: &w.title, options }
}

/// One set's most (Progress.lua `ns.CountMax`): a counted set's clean reps, a hold's seconds or a rep set's reps.
#[must_use]
pub fn count_most(counted: bool, hold: bool) -> i64 {
	let c = facts().constants();
	if counted {
		c.count_max
	} else if hold {
		c.max_seconds
	} else {
		c.max_reps
	}
}

/// "Max 100: count one set", the game's words for one set's `most`, where a count reaches it.
#[must_use]
pub fn max_line(most: i64) -> String {
	words::fill(&facts().sheet.words.count.max, &[&most.to_string()])
}

/// What "Is that right?" names beside the count it asks about: the move's record (none on a placement or before the
/// first, whose count is checked against `top`, the rung's top a first max set ends at: Progress.lua `ns.FirstTop`, its
/// hi or a variant's cap), its weighted record (tenths of a kg and its reps), the weight planned (the weight's own
/// question's), and whether the move is a hold (its seconds), a counted set (its best clean count, in its own words)
/// and a loaded move's weight in each hand. The count step's from its move, Correct a count's from the state before the
/// set (slice 44, `book.rs`).
pub struct Against {
	pub record: Option<i64>,
	pub top: Option<i64>,
	pub top_load: Option<u32>,
	pub top_reps: Option<i64>,
	pub plan: Option<u32>,
	pub hold: bool,
	pub counted: bool,
	pub each: bool,
}

/// "Is that right?" about `typed` as the engine checked it (`check`, any but `max`, which refuses and asks nothing), in
/// the game's words.
#[must_use]
pub fn check_view(check: Check, typed: Typed, a: &Against) -> CheckView {
	let w = &facts().sheet.words.count;
	let Typed { value, load, answer } = typed;
	// a weight far over the plan: the game's own question about the weight (Panel.lua showReally's heavyAsk)
	if let (Check::Heavy, Some(_), Some(plan)) = (check, load, a.plan) {
		let words = CheckWords::Heavy { line: &w.confirm_kg, yes: &w.really_yes, plan, each: a.each };
		return CheckView { value, load, answer, words };
	}
	let low = check == Check::Low;
	let first = a.record.is_none();
	let against = a.record.or(a.top).unwrap_or(0);
	// a counted set asks only about a clean count far above its best, in its own words, never a record's (slice 42's fix
	// round, 2026-10-17: the game's ui.result.confirmBest, "Yes, keep N")
	let line = if a.counted {
		&w.confirm_best
	} else if low {
		&w.confirm_low
	} else if first {
		&w.confirm_first
	} else {
		&w.confirm
	};
	let yes = if low || a.counted { &w.really_keep } else { &w.really_yes };
	// a weight checked against the weighted record (Progress.lua ns.Validate), else against the count's
	let words = match (load, Lifted::of(a.top_load, a.top_reps).filter(|_| !first)) {
		(Some(_), Some(top)) => {
			CheckWords::Weighted { line, yes, record: top.reps, record_load: Some(top.load), each: a.each }
		}
		(Some(_), None) => CheckWords::Weighted { line, yes, record: against, record_load: None, each: a.each },
		(None, _) => CheckWords::Filled {
			line: words::fill(line, &[&amount(value, a.hold), &amount(against, a.hold)]),
			yes: words::fill(yes, &[&amount(value, a.hold)]),
		},
	};
	CheckView { value, load, answer, words }
}

/// The count step of the move being counted (the phones' `CountView`): the question, where the stepper starts, its
/// most, the record and its chip, one set's most in the game's words, "Is that right?" with its Yes, the step's other
/// answers in the game's words, and a loaded move's weight box.
fn count_view(run: &Run, m: &PlannedMove) -> CountView {
	let w = &facts().sheet.words.count;
	let c = facts().constants();
	let hold = m.kind == kind::HOLD;
	let most = count_most(m.counted, hold);
	// the line names the half again only where that sets the most; at the floor (`min_reps`, `min_seconds`) none
	let capped = run.count_cap(m);
	let cap = capped.and_then(|cap| cap.by.map(|by| CapView::of(cap.most, by)));
	let question = match (m.counted, hold, m.side) {
		(true, ..) => "form",
		(false, true, true) => "hold_side",
		(false, true, false) => "hold",
		(false, false, true) => "reps_side",
		(false, false, false) => "reps",
	};
	let slot = run.slots.get(m.slot).and_then(Option::as_ref);
	let check = run.count.shown.filter(|(check, _)| *check != Check::Max).map(|(check, typed)| {
		// against the record, or a first count against the rung's top (Progress.lua ns.FirstTop: its hi or a variant's
		// cap)
		let against = Against {
			record: m.record.filter(|_| !slot.is_some_and(|s| s.placing)),
			top: slot.and_then(|s| s.hi.or(s.cap)),
			top_load: m.top_load,
			top_reps: m.top_reps,
			plan: m.load,
			hold,
			counted: m.counted,
			each: m.load_each,
		};
		check_view(check, typed, &against)
	});
	let kg = m.start.map(|[lo, _]| {
		let sizes: Vec<u32> = if m.implement.as_deref() == Some(KETTLEBELL) {
			c.load_bells.iter().map(|kg| kg * TENTHS).collect()
		} else {
			Vec::new()
		};
		let lo = lo * TENTHS;
		let first = m.load.unwrap_or_else(|| sizes.iter().copied().find(|&s| s >= lo).unwrap_or(lo));
		KgView {
			each: m.load_each,
			from: run.count.typed.map_or(Some(first), |t| t.load),
			first,
			most: c.load_max_kg * TENTHS,
			sizes,
			step: c.load_box_step,
			top: Lifted::of(m.top_load, m.top_reps),
		}
	});
	// never past it: a hold held past `max_seconds` starts at that most, which Save takes
	let top = capped.map_or(most, |cap| cap.most);
	CountView {
		question,
		name: facts().name(m.r#move).unwrap_or_default(),
		seconds: c.count_s,
		from: match run.stage {
			Stage::Longer { typed, .. } => typed.value,
			_ => run.count.typed.map_or(run.count_from, |t| t.value),
		}
		.min(top),
		most: top,
		cap,
		hold,
		record: m.record,
		chip: m.record.or(m.placing),
		max_set: run.max_set(m),
		max_line: if capped.is_some() { String::new() } else { max_line(most) },
		check,
		change: &w.really_change,
		empty: &w.empty,
		could_not: (!m.counted).then_some(if hold { w.could_not_hold.as_str() } else { w.could_not.as_str() }),
		kg,
	}
}

/// The pump's kind as the page names it.
fn what_name(what: &What) -> &'static str {
	match what {
		What::Quick => "quick",
		What::Free { .. } => "free",
		What::Move { .. } => "move",
		What::CatchUp => "catchup",
		What::Expedition { .. } => "expedition",
	}
}

/// The pump's stage as the page names it: an expedition's rest between rounds before any.
fn stage_name<P: Platform>(h: &Desk<P>, run: &Run) -> &'static str {
	match run.stage {
		_ if h.expedition_resting() => "rest",
		Stage::Ready => "ready",
		Stage::Countdown { .. } => "countdown",
		Stage::Running { .. } => "running",
		Stage::Asking { .. } => "asking",
		Stage::Counting => "counting",
		Stage::Marking { .. } => "marking",
		Stage::Pacing { .. } => "pacing",
		Stage::Longer { .. } => "longer",
		Stage::Saved => "saved",
		Stage::Done => "done",
	}
}

fn pump_view<P: Platform>(h: &Desk<P>, run: &Run) -> PumpView {
	let now = h.p.now_ms();
	let shown = run.shown();
	let side = run.side.as_ref().and_then(MoveRun::side).map(|s| if s == Side::First { "first" } else { "other" });
	let (go, digit) = match run.stage {
		Stage::Countdown { go, digit } => (Some(go), digit),
		_ => (None, None),
	};
	// a kept steady move's end the engine judges keeps the kept dial until the answer (`Run::kept_judged`)
	let judged = run.kept_judged().then(|| shown.and_then(|m| m.keep)).flatten();
	let (start, limit) = match run.stage {
		Stage::Running { start } => (Some(start), run.side.as_ref().and_then(MoveRun::limit).or(judged.map(run::real))),
		_ => (None, None),
	};
	let asked = match run.stage {
		Stage::Asking { seconds, full } => Some(AskedView { seconds, full }),
		_ => None,
	};
	let marking = match (run.stage, shown) {
		(Stage::Marking { .. }, Some(m)) => Some(MarkingView { mark: facts().mark(m.r#move) }),
		_ => None,
	};
	let running = run.side.as_ref().filter(|_| matches!(run.stage, Stage::Running { .. }));
	let kept = running.is_some_and(MoveRun::kept) || judged.is_some();
	let keep_going = run.cues.keep && running.is_some_and(|s| !s.kept());
	// a rider's owed move was offered before; an app pump's own move is new where the fold never saw it
	let seen =
		|i: usize| run.moves.get(i).is_some_and(|m| run.rider(m.slot)) || run.previews.get(i).is_none_or(|p| p.seen);
	let counts = h.counts_chosen();
	let ready = run.stage == Stage::Ready;
	let words = shown.filter(|_| run.stage != Stage::Done);
	let calm = (run.stage == Stage::Done).then(|| calm_view(h, run, now)).flatten();
	PumpView {
		id: run.id.clone(),
		what: what_name(&run.what),
		kind: match &run.what {
			What::Free { kind, .. } => Some(kind.clone()),
			_ => None,
		},
		from: match run.what {
			What::Move { from, .. } => Some(from),
			_ => None,
		},
		check: run.what.check().is_some(),
		arms: run.arms.clone(),
		busy: run.busy > 0,
		stage: stage_name(h, run),
		current: run.current,
		ways: Ways {
			back: run.can_back(),
			skip: ready && run.on_rider(),
			bank: h.pump_bank_way(run),
			rest: run.ends_rest(),
		},
		go,
		digit,
		start,
		limit,
		side,
		asked,
		marking,
		keep_going,
		kept,
		moves: run.moves.iter().enumerate().map(|(i, m)| move_view(run, i, m, seen(i), counts)).collect(),
		rewards: rewards(run),
		lines: shown
			.filter(|_| ready)
			.map(|m| lines(m, pace_rung(run, run.current, m), pose_days(run, m)))
			.unwrap_or_default(),
		setup: words.and_then(|m| facts().setup(m.r#move, m.implement.as_deref())),
		how: words.map_or(&[], |m| facts().how(m.r#move, m.implement.as_deref())),
		tips: words.map(|m| facts().tips(m.r#move)).unwrap_or_default(),
		guide: h.pump_guide_state(),
		swap: run
			.swaps
			.as_ref()
			.filter(|_| ready)
			.map(|o| swap_view(o, shown.is_some_and(|m| new_move(m, seen(run.current))))),
		count: shown
			.filter(|_| matches!(run.stage, Stage::Counting | Stage::Longer { .. }))
			.map(|m| count_view(run, m)),
		result: if run.what.app() {
			run.result.clone()
		} else {
			(run.stage == Stage::Done && !run.unwell && run.cut.is_none()).then(|| caught_up(run))
		},
		back: run.back,
		work: run.work.is_some(),
		ended_done: run.ended_done(),
		reveal: run.reveal.clone(),
		legend: run.legend,
		calm,
		saved: run.pour.as_ref().filter(|_| run.stage == Stage::Saved).map(|p| p.view().clone()),
		expedition: h.trip_view(run),
	}
}

/// A pump that ended short, calm: felt unwell (its lock; the safety note only on the Menu's Safety note page, David,
/// 2026-10-24: only what is legally necessary), left part way, ended by the game's own pump, or ended before a move
/// ran.
fn calm_view<P: Platform>(h: &Desk<P>, run: &Run, now: i64) -> Option<CalmView> {
	let why = if run.unwell {
		"unwell"
	} else if run.cut == Some(Cut::Game) {
		"game"
	} else if run.cut == Some(Cut::Left) {
		"left"
	} else if run.ran.is_empty() && run.recorded.is_empty() {
		"ended"
	} else {
		return None;
	};
	let unwell = run.unwell;
	Some(CalmView {
		why,
		until: h.session.locked_until(now.div_euclid(1000)).map(|s| s * 1000).filter(|_| unwell),
		title: unwell.then(|| facts().sheet.words.unwell.title.as_str()),
		reload: unwell && h.p.game().followed,
		paid: run.paid.map(paid_word),
	})
}

/// The topic as it stands.
pub(super) fn topic<P: Platform>(h: &Desk<P>) -> Topic {
	let p = &h.pumps;
	let now = h.p.now_ms();
	let flags = &p.flags;
	let w = &facts().sheet.words;
	Topic {
		gate: h.pump_gate(),
		locked_until: h.session.locked_until(now.div_euclid(1000)).map(|s| s * 1000),
		owed: h.pump_payable(flags).len(),
		place: p.waiting.is_some() && !h.settings.place_asked,
		choose: p.choose.as_ref().map(|(c, _)| ChooseView {
			kind: c.kind.clone(),
			size: c.size,
			offer: c.offer.clone(),
			kinds: words::kinds().iter().map(|k| k.key.as_str()).collect(),
			arms: h.arms_game(),
			one_size: h.offer_one_size(c.offer.as_deref()),
		}),
		pain: facts()
			.sheet
			.joints
			.iter()
			.map(|j| Joint { id: j.id, name: &j.name, flagged: flags.contains(&j.id) })
			.collect(),
		pain_ask: &w.pain.asked,
		refused: p.refused,
		running: h.desk_pump_running(),
		pump: p.run.as_ref().map(|run| pump_view(h, run)),
	}
}
