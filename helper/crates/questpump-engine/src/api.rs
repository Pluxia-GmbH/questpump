//! The engine's JSON at its door ([`crate::Engine::call`]), typed: what each op takes (crate-private: the typed helpers
//! build them) and what it answers. Keys are the contract's camelCase, as `contract/fixtures/engine.json` and the
//! phones' cores carry them; a flag is present only when true and an absent value is absent. Indexes of a pump's moves
//! count from 0 here, as the phones' do; `tools/apppump.lua` counts from 1 inside, and the engine converts between.
//! Times are unix seconds; row texts are the addon's grammar (DB.lua `ns.EncodeRow`).

use serde::{Deserialize, Serialize};

#[expect(clippy::trivially_copy_pass_by_ref, reason = "serde's skip_serializing_if passes a reference")]
fn is_false(b: &bool) -> bool {
	!*b
}

/// How the player trains, as the engine reads it (the phones' `EngineProfile`, DB.lua's profile): the equipment, the
/// place and the Body switches, the experience question's answer, the moves left out and the opt-in moves checked,
/// the favourites, the title chosen and "Moves per match" (`matchPace`: `light` or `extreme`, absent for `balanced`;
/// slice 48), the pace of a pump composed with [`Match`] facts. Every other app pump is Balanced.
#[expect(
	clippy::struct_excessive_bools,
	reason = "the contract's profile: one switch a field, as the phones' EngineProfile"
)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
	/// Equipment keys (Library.lua's `needs`, `stable_chair` and `table` among them).
	pub gear: Vec<String>,
	pub floor: bool,
	pub space: bool,
	pub quiet: bool,
	pub no_jump: bool,
	pub no_strain: bool,
	pub no_head_down: bool,
	/// The experience question's answer: `none`, `some` or `training` (the start rungs).
	pub experience: String,
	pub exclude: Exclusions,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub include: Vec<u16>,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub favourites: Vec<u16>,
	/// The title chosen, by its key; shown on the app's own sheet while it is earned.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub title: Option<String>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub match_pace: Option<String>,
}

/// What a pump started from a game's end, or a match picked by hand, composes with (`A.compose`'s `match`, slices 48
/// and 57; docs/computer-games.md §5): the match's whole minutes, its cardio's active minutes (the sitting clock's
/// where no match was measured), whether it opens on its warm move (the sitting clock held `match_sat_min`; present
/// only when true) and the arms rest after a game marked `aim` (present only when true).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Match {
	pub minutes: u16,
	#[serde(default, skip_serializing_if = "is_false")]
	pub warm: bool,
	#[serde(default, skip_serializing_if = "is_false")]
	pub arms: bool,
}

/// The unfinished moves a pump a match sized carries (`A.compose`'s `rows` and `queue`, docs/plan.md slice 61): the
/// owed rows its device may pay (`owed`, row texts as `Session::payable_kept` orders them, which the engine takes
/// oldest first as the game does, `ns.OwedList`; a `Rider`'s `row` is the place in this list) and every owed row the
/// game's store keeps (`queue`), on which it is composed as the game composes a quest's pump, so no fresh pick takes a
/// queued move, also where it may pay none. Empty for every other pump.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct Unfinished {
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub owed: Vec<String>,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub queue: Vec<String>,
}

/// The moves the player unchecked: by id, and by group (a library group's key).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct Exclusions {
	pub ids: Vec<u16>,
	pub groups: Vec<String>,
}

/// What a goal stands for (Progress.lua `ns.Goal`'s kind): one step past the record, one past the player's form, a
/// lighter repeat the same day, or on a loaded move the reps that match the player's form at a heavier weight than it
/// was set with (`heavier`, the plan's step up among them).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Goal {
	Record,
	Form,
	Repeat,
	Heavier,
}

/// A move of a pump as the engine composed it (the contract's slot: `tools/apppump.lua` `A.slotJSON`, the phones'
/// `EngineSlot`).
#[expect(
	clippy::struct_excessive_bools,
	reason = "the contract's slot: one flag a field, as A.slotJSON and the phones' EngineSlot"
)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Slot {
	pub r#move: u16,
	/// The row kind: 1 rep, 2 hold, 3 cardio, 4 flow.
	pub kind: u8,
	/// The ladder and the rung it is dosed at (ladder 0 off the ladders; the cardio rung for cardio).
	pub ladder: u8,
	pub rung: u8,
	pub tier: u8,
	/// The count the goal asks for and its kind; none on a max set, whose count becomes the record.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub target: Option<i64>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub goal: Option<Goal>,
	/// A first max set of the anchor ends at `hi`; `cap` asks "Really?" past twice it.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub hi: Option<i64>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub cap: Option<i64>,
	/// A timed move's seconds (an all-out window's from its go: the build and the fast part, or a counted set's count).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub seconds: Option<i64>,
	/// A pose that asks "How close to the goal pose?" once it ran its whole time: the least answer that counts the day
	/// (`ns.Enum.mark`: 3 Good, 4 Perfect below a risky step; a stretch slot's check's where no day counts).
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub ask: Option<u8>,
	/// The answer its own goal pose waits for (Engine.lua `markSlot`'s `goalAsk`, slice 42's fix round, 2026-10-17: its
	/// own note's, beside `ask`).
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub goal_ask: Option<u8>,
	/// The waiting stretch check a stretch or yoga slot's pose runs (Engine.lua `checkSlot`, slice 42's fix round).
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub check: Option<StepNeed>,
	#[serde(default, skip_serializing_if = "is_false")]
	pub max_set: bool,
	#[serde(default, skip_serializing_if = "is_false")]
	pub placing: bool,
	#[serde(default, skip_serializing_if = "is_false")]
	pub allout: bool,
	/// A closing move: the all-out window that takes the place of a pump's first steady cardio move from cardio rung
	/// `closing_rung`, or a move started all out, dosed as a finisher.
	#[serde(default, skip_serializing_if = "is_false")]
	pub closing: bool,
	#[serde(default, skip_serializing_if = "is_false")]
	pub trial: bool,
	#[serde(default, skip_serializing_if = "is_false")]
	pub anchor: bool,
	/// Swap's easier version: its row carries the easier bit.
	#[serde(default, skip_serializing_if = "is_false")]
	pub easier: bool,
	/// Swap's backup, below the move it backs up.
	#[serde(default, skip_serializing_if = "is_false")]
	pub lower: bool,
	/// A pose that names the mark of the player's step.
	#[serde(default, skip_serializing_if = "is_false")]
	pub mark: bool,
	/// A counted set: the all-out window of a countable form, its count typed if the player likes, never a count to
	/// reach (slice 42, 2026-10-16).
	#[serde(default, skip_serializing_if = "is_false")]
	pub counted: bool,
	/// The steady move after which the pump asks "Did you keep that pace to the end?" (slice 52; Engine.lua
	/// `paceSlot`: where the steady ledger asks, `ns.LedgerAsk`, and after every Long Haul), unless Keep going ran it
	/// past its time, which saves its own answer.
	#[serde(default, skip_serializing_if = "is_false")]
	pub pace: bool,
	/// The full move's seconds Keep going runs a steady move on to (slice 52; Engine.lua `keepSlot`: a steady move
	/// under the full move, never an all-out window, a warm move or a Long Haul).
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub keep: Option<i64>,
	/// A Long Haul's steady move (slice 55; Engine.lua `chosenSlot`: its seconds its length, its pace rung 1's).
	#[serde(default, skip_serializing_if = "is_false")]
	pub haul: bool,
	/// A loaded move's weight planned, in tenths of a kg (Progress.lua `ns.LoadPlan`; none before a set with a weight),
	/// whether the plan steps the weight up (beside the plan, false or true), and on a step up the working weight it
	/// came from (Engine.lua dose; a placement plans the working weight, never a step up).
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub load: Option<u32>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub load_up: Option<bool>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub load_from: Option<u32>,
}

/// Swap's options for a move on its Ready view (Engine.lua `ns.SwapOptions`), each a slot or absent; after a swap the
/// way back alone (`back`: the slot it replaced, as it was, while the filters still pass its move; Events.lua
/// `Lc.SwapOptions`, since 2026-10-26).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SwapOptions {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub another: Option<Slot>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub easier: Option<Slot>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub backup: Option<Slot>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub back: Option<Slot>,
}

impl SwapOptions {
	/// The option `take` names, none where it is not offered.
	#[must_use]
	pub fn get(&self, take: Take) -> Option<&Slot> {
		match take {
			Take::Another => self.another.as_ref(),
			Take::Easier => self.easier.as_ref(),
			Take::Backup => self.backup.as_ref(),
			Take::Back => self.back.as_ref(),
		}
	}
}

/// Which of the Swap options shown last the player took.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Take {
	Another,
	Easier,
	Backup,
	Back,
}

/// The pump on screen (the phones' `AppPump`): its id (`d-<uuid>` on the desktop), the plain kind's key or `quick`,
/// the engine's family for it (none for the Quick pump), its size (1 Light, 2 Balanced, 3 Extreme), its moves as Swap,
/// Skip and the Ready view's Pain leave them, the moves Swap changed (each offering its way back until it is taken)
/// and, on a pump a match sized, its riders (slice 61, `A.ridersJSON`; none on every other pump).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Pump {
	pub id: String,
	pub kind: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub family: Option<String>,
	pub tier: u8,
	pub slots: Vec<Slot>,
	pub swapped: Vec<usize>,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub riders: Vec<Rider>,
}

/// A move of a match pump that pays an unfinished move (slice 61; engine.json's `riders`, counted from 1 there): its
/// place among the pump's moves, the place of its row in the [`Unfinished`] `owed` the pump was composed with, and the
/// time that row was owed, which its payment names.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rider {
	pub index: usize,
	pub row: usize,
	pub ts: i64,
}

/// A pump composed: the pump and the rows written (the start rungs before a first pump, then its offered row, which a
/// pump with no move never writes).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Composed {
	pub pump: Pump,
	pub rows: Vec<String>,
}

/// What the player did with a move (the phones' `EngineResult`): the typed count (reps, a hold's seconds) or a timed
/// move's seconds, the seconds it took, whether a timed move was stopped short, whether "Could not do one" stored a 0,
/// whether a "Really?" count was kept, the felt-unwell reason (`ns.Enum.stopped.unwell`) when that stop ended a
/// running timed move, a pose's answer to "How close to the goal pose?" (`ns.Enum.mark`, where its move `ask`s), a
/// counted set's answer to "All out and clean?" (`answer`, `ns.Enum.answer`: 1 Form slipped, slice 42, 2 Not all out,
/// slice 53, each done, never a record or a clean day; none Yes, to the end), a steady move's answer to "Did you keep
/// that pace to the end?" (`pace`, `ns.Enum.pace`: 1 Yes, just, 2 I eased off, 3 Yes, easily, where its move asks it,
/// or Keep going's, slice 52), a hold's answer to "Could you have held longer?" (`longer`, `ns.Enum.longer`: 1 No, 2
/// A little, 3 A lot, where its set asks it, slice 52), and the weight a loaded move's set was done with, in tenths of
/// a kg (none: no weight; its load row comes before its row).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct MoveResult {
	pub value: i64,
	pub elapsed: i64,
	#[serde(default)]
	pub stopped: bool,
	#[serde(default)]
	pub could_not: bool,
	#[serde(default)]
	pub confirmed: bool,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub reason: Option<u8>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub mark: Option<u8>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub answer: Option<u8>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub pace: Option<u8>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub longer: Option<u8>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub load: Option<u32>,
}

/// A count checked against the game's limits before it is written (Progress.lua `ns.Validate`): past one set's
/// most or a weight past the box's most (refused), a weight far over the one planned (`heavy`, its own question), far
/// above the record or a first max set's top ("Really?"), or under half the record ("Is that right?"); with a weight
/// and a weighted record, the record is its reps at that weight. Each but `max` is asked once, as the game asks any
/// answer but `max` (Events.lua `Lc.Enter`), an answer this build does not know (`other`, a newer engine's) too.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Check {
	Max,
	Heavy,
	Really,
	Low,
	#[serde(other)]
	Other,
}

/// A move of the pump as a device runs it (the phones' `PlannedMove`): its slot (from 0), move, kind and count to
/// reach, a timed move's seconds, an all-out window's build from its go (none on a counted set, all out from its go),
/// whether it runs one side, then the other (`side`), the record its goal beats (a counted set's best clean count,
/// never a goal since slice 42) and the goal's kind (Progress.lua `ns.SlotGoal`'s; none on a max set), on a form goal
/// the player's form its Ready line names (`form`, `ns.Form`'s count; with a weight planned on a move with a weighted
/// record, a goal read in strength, `ns.LoadForm`'s reps at its weight `form_load`), a first max set's top, a
/// placement's record, whether it is a first max set (`first_max`, Progress.lua `ns.FirstMaxSet`: a count under the
/// minimum saves as "Could not do one", never a bank), whether a steady move asks "Did you keep that pace to the end?"
/// (`pace`, slice 42), the full move's seconds Keep going runs it on to (`keep`, slice 52), whether it is a Long Haul (`haul`, slice 55), whether it is a match pump's warm move, dosed easy (`warm`, slice 48's double-check: a cardio move at the stand-up's easy
/// cadence, `cadence.standup`, a rep move at the repeat share of its form), a pose's mark and, where it asks "How close
/// to the goal pose?" once it ran its whole time, the least answer that counts the day (`ask`: 3 Good, 4 Perfect; what
/// it counts toward its Ready view reads, [`Toward`]); on a loaded move (Progress.lua `ns.Loaded`) its weight: the one
/// planned and whether it steps up (from the working weight `load_from`), the reps its fresh sets reach twice before a
/// step up (`load_top`, `ns.LoadTop`), a weight in each hand, the library's start range (whole kg, for the first set
/// with a weight), the implement that carries it (`ns.Implement` with the player's gear; none: a makeshift load), the
/// weighted record and the last set's weight with its reps. Weights are tenths of a kg. Since slice 42, where the climb
/// waits on this move's stage of its path alone (Progress.lua `ns.PathDue`), what still opens the next step (`next`:
/// the Ready view's next-step line). On a rep or hold move with a count to reach, the sets its ladder took this game
/// day before it (`today`, Progress.lua `ns.SetsToday`, the repeat share's count; none for none): its Ready view's
/// "Goal · set N today" (2026-10-28).
#[expect(clippy::struct_excessive_bools, reason = "the phones' PlannedMove: one flag a field")]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PlannedMove {
	pub slot: usize,
	pub r#move: u16,
	pub kind: u8,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub target: Option<i64>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub goal: Option<Goal>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub seconds: Option<i64>,
	#[serde(default)]
	pub allout: bool,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub build: Option<i64>,
	#[serde(default)]
	pub side: bool,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub record: Option<i64>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub form: Option<i64>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub form_load: Option<u32>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub top: Option<i64>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub placing: Option<i64>,
	#[serde(default, skip_serializing_if = "is_false")]
	pub first_max: bool,
	#[serde(default)]
	pub counted: bool,
	#[serde(default)]
	pub pace: bool,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub keep: Option<i64>,
	#[serde(default, skip_serializing_if = "is_false")]
	pub haul: bool,
	#[serde(default)]
	pub warm: bool,
	#[serde(default)]
	pub mark: bool,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub ask: Option<u8>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub load: Option<u32>,
	#[serde(default, skip_serializing_if = "is_false")]
	pub load_up: bool,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub load_from: Option<u32>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub load_top: Option<i64>,
	#[serde(default, skip_serializing_if = "is_false")]
	pub load_each: bool,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub start: Option<[u32; 2]>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub implement: Option<String>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub top_load: Option<u32>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub top_reps: Option<i64>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub last_load: Option<u32>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub last_reps: Option<i64>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub next: Option<NextStep>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub today: Option<u32>,
}

/// What opens the next step where its climb waits on a move's stage of its path alone (Progress.lua `ns.PathDue`, slice
/// 42; Panel.lua `nextStepLine`, the line under a Ready view's goal): the move's top there ([`Top`]: reps or a hold's
/// seconds, a pose's least answer to "How close to the goal pose?", a countable form's clean window), the game days at
/// it the stage still needs and on a loaded move the weight its climb asks (`kg`, whole kg, `each` in each hand:
/// `ns.ClimbKg`).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct NextStep {
	pub top: Top,
	pub left: i64,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub kg: Option<i64>,
	#[serde(default, skip_serializing_if = "is_false")]
	pub each: bool,
}

/// What a pose's answer to "How close to the goal pose?" counts toward (Progress.lua `ns.MarkToward`, 2026-10-14;
/// Engine.lua `markSlot` sets the ask), read on the move's Ready view since slice 47 ([`crate::Engine::toward`]): the
/// colour of the orange or gold step its family's main pose stands on, which no hold has earned yet and only Perfect
/// earns (`earn`, `ns.ToEarn`); the step above with the days counted toward it, `have` of `need` (a goal pose of the
/// family's step the climb's, `ns.UnlockProgress`, and since slice 47 what those days still lack: `main` and `perfect`
/// the days still to come on the step's main pose and answered Perfect, `mainPose` that main pose, `ns.PoseMain`, none
/// where a stand-in from below stands for the step; `ns.PoseNeeds`, Panel.lua `poseDaysLine`; a pose of the step's path
/// the days its stage asks, `ns.TopDays` of `ns.StageDays`, since slice 42, with none of the three); the step above
/// unnamed (`next`), where the answer counts no day toward a step the family can climb to (the setup or a pain flag
/// shuts it, it waits for another ladder's step, or the pose is on no stage still out); since slice 42 a stretch check
/// above its family's step, which an answer at least the check asks lifts the family to (`check`), or the pose's own
/// goal pose still to answer (`goal`, toward its step's Mastered or a check's mark on it).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Toward {
	Earn,
	Step {
		have: i64,
		need: i64,
		#[serde(default, skip_serializing_if = "Option::is_none")]
		main: Option<i64>,
		#[serde(default, skip_serializing_if = "Option::is_none")]
		perfect: Option<i64>,
		#[serde(default, rename = "mainPose", skip_serializing_if = "Option::is_none")]
		main_pose: Option<u16>,
	},
	Next,
	Check,
	Goal,
}

/// What a move of the pump on screen can earn before its Start, as the phones' Ready view puts it on the quest page
/// (Stats.lua `ns.RewardPreview`'s XP and its record chip): the XP it pays done and, on a move that asks for a count
/// past a record standing, the stat that record raises (`stat`, its key in `contract/sheet.json`'s stats) by `amount`
/// if the set beats `beat` (the chip's `at`: the record, a hold's in seconds, or on a loaded move the reps at the
/// planned weight that match the weighted record); and whether the move was done before (`seen`): its first appearance
/// shows the right-versus-wrong pair.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Preview {
	pub xp: i64,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub stat: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub amount: Option<i64>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub beat: Option<i64>,
	#[serde(default)]
	pub seen: bool,
}

/// What a clear row clears (`ns.Enum.clear`): one move's record, or one ladder, started over.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Clear {
	Move,
	Ladder,
}

/// The one progression folded: the joints the game's own rows flag (ascending: the game's save's pain flags, which the
/// `gameFlags` op answers alone). The lock is not in it: the engine's `locked` answers it, and the desktop's is the
/// helper's.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Folded {
	pub game_flags: Vec<u8>,
}

/// Rows from elsewhere taken in (`tools/apppump.lua` `E:merge`): how they went in.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Merged {
	pub how: MergeHow,
}

/// How rows from elsewhere went in: applied in turn (`order`; rows that bring nothing into the state go in so too),
/// the ring merged and folded again from the baseline (`ring`), or everything folded again (`fold`).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MergeHow {
	Order,
	Ring,
	Fold,
}

/// The game's own sheet as the game folds its save at its next load (the `gameSummary` op): the summary (Stats.lua
/// `ns.SheetSummary`, v 6, its `saved` the save's newest game row; the XP it counted on each game day its `dx`).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GameSummary {
	pub sheet: serde_json::Value,
}

/// The game's feedback numbers (the `feedbackNumbers` op; Stats.lua `ns.FeedbackNumbers`, contract/feedback-api.yaml
/// "The counting rules"): the whole game days since the earliest row, over the last `feedback_days` game days the pumps
/// finished and banked, the moves unfinished now and the median of the max sets per ladder on a play day (the moves'
/// counts the numbers carry too are left unread).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct FeedbackNumbers {
	pub days: i64,
	pub finished: u32,
	pub banked: u32,
	pub owed: u32,
	pub max_sets: f64,
}

/// The level a lifetime XP reaches (Stats.lua `levelOf`, `ns.LevelOf`), the XP into it and what the next one needs; at
/// the cap the XP into the next Paragon rank and its need, and the Paragon rank (0 under the cap).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LevelAt {
	pub level: i64,
	pub into: i64,
	pub need: i64,
	#[serde(default)]
	pub paragon: i64,
}

/// Whether every pump is locked now (the latest felt-unwell stop's game day), and that stop's time. The lock's end is
/// the caller's: the next game day's first second in the computer's time zone.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Locked {
	pub locked: bool,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub unwell: Option<i64>,
}

/// A move's record (as the fold reads the move) and the goal an app pump asks of it now; on a loaded move the weighted
/// record, the last set's weight and the weight planned now (`load`), at which the goal is asked (tenths of a kg).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Record {
	pub r#move: u16,
	pub record: i64,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub goal: Option<i64>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub top_load: Option<u32>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub top_reps: Option<i64>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub last_load: Option<u32>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub load: Option<u32>,
}

/// A loaded move's next fresh set after the sets so far (Progress.lua `ns.NextLoad`): its weight in tenths of a kg and
/// the reps it asks at that weight; the Result's "Next time" line.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct NextLoad {
	pub load: u32,
	pub target: i64,
}

/// A saved set as Correct a count names it (slice 44, 2026-10-18): its row as stored, by its time, its quest, its slot
/// and its move (an old merged id stays as stored), which every correction row of it repeats.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SetKey {
	pub ts: i64,
	pub quest: u32,
	pub slot: u8,
	pub r#move: u16,
}

/// What a correction of a set may change (Progress.lua `ns.CorrectableRow`): a rep set's or a counted set's count, a
/// hold's seconds, or a pose's answer to "How close to the goal pose?"; an answer this build does not know (`other`, a
/// newer engine's) too.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CorrectKind {
	Count,
	Seconds,
	Answer,
	#[serde(other)]
	Other,
}

/// A saved set as the fold reads it now (DB.lua `ns.SetValue`, `ns.SetOutcome`, `ns.SetLoad`; slice 44): its value (a
/// count, a hold's seconds, a pose's answer, its winning correction's where one names it: a capped max set corrected
/// over its window's top keeps the top), its outcome (`questpump_core::codes::outcome`: done where a count of 0 was
/// saved, or corrected, without a count; slice 44's review, 2026-10-18) and its weight (tenths of a kg; none without
/// one).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetReading {
	pub value: i64,
	pub outcome: u8,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub load: Option<u32>,
}

/// A set Correct a count's window lists (`tools/apppump.lua` `A.correctableJSON` over Progress.lua `ns.Correctable`:
/// of the move's sets of the last `correct_days` game days after the window's reach, DB.lua `ns.CorrectReach`, a margin
/// of rows past the game's ring horizon, the newest `correct_sets` and the record's set where it is older): the set,
/// what a correction may change, the set as the fold reads it now, the round a correction of it takes, whether it
/// holds the record, and on an app set whose winning correction or load row is another side's, how its own side reads
/// it (`own_side`).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CorrectSet {
	#[serde(flatten)]
	pub set: SetKey,
	pub kind: CorrectKind,
	#[serde(flatten)]
	pub reads: SetReading,
	pub round: u8,
	#[serde(default, skip_serializing_if = "is_false")]
	pub record: bool,
	#[serde(default, rename = "ownSide", skip_serializing_if = "Option::is_none")]
	pub own_side: Option<OwnSide>,
}

/// How a set's own side reads it where the correction or the load row that wins is another side's and that side's
/// replay reads it otherwise (DB.lua `ns.OwnSide`, 2026-10-19, the program's double-check): its count (a hold's
/// seconds, a pose's answer) and weight (tenths of a kg). Its window offers Save at the value the set reads now, a new
/// round of it that asks the step rules again on this side.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct OwnSide {
	pub value: i64,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub load: Option<u32>,
}

/// The check a correction meets (`tools/apppump.lua` `Engine:correctCheck`, Progress.lua `ns.CorrectionCheck`: the
/// count step's limits and "Is that right?", against the state just before the set, never about a low count on a
/// counted set, as its count step asks none) and what its question names: the record before the set (none on a
/// placement or before the first), the top a placement or a first max set is checked against, the weighted record
/// before the set (tenths of a kg and its reps), the set's own weight, and whether the set is a hold (its seconds) or a
/// counted set (its best clean count).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CorrectCheck {
	pub check: Check,
	#[serde(default, skip_serializing_if = "is_false")]
	pub hold: bool,
	#[serde(default, skip_serializing_if = "is_false")]
	pub counted: bool,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub record: Option<i64>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub top: Option<i64>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub top_load: Option<u32>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub top_reps: Option<i64>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub plan: Option<u32>,
}

/// What a correction's check came to: the set has left the window (`gone`: its game days ran out, a clear row or a
/// newer horizon came), it asks nothing (`clear`), or it meets a check (`asks`: refused past one set, or asked once).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(tag = "outcome", rename_all = "lowercase")]
pub enum CorrectionCheck {
	Gone,
	Clear,
	Asks { check: CorrectCheck },
}

/// Why a corrected set's steps stay where they are, though the step rules asked again with the corrected value would
/// move them and the mastered sweep after the correction leaves them without that change (Progress.lua
/// `ns.CorrectionRows`' `stays`): the set is the game's (only a device of its own source asks them again), or a later
/// change of any character stands on them; an answer this build does not know (`other`) too.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Stays {
	Source,
	Later,
	#[serde(other)]
	Other,
}

/// A correction written (`tools/apppump.lua` `Engine:correctRows`): the rows in their order (the load row where the
/// weight changed, the correction row, the step rows the corrected set calls for), why the steps stay where they do,
/// the set as the fold reads it after them (`now`: what the window's list shows next, which its done view says) and
/// what the rows changed on the steps (`steps`, none where they changed none), which the done view says too; or
/// nothing, the set having left the window (`gone`).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "outcome", rename_all = "lowercase")]
pub enum Corrected {
	Gone,
	Written {
		rows: Vec<String>,
		#[serde(default, skip_serializing_if = "Option::is_none")]
		stays: Option<Stays>,
		now: SetReading,
		#[serde(default, skip_serializing_if = "Option::is_none")]
		steps: Option<StepChanges>,
	},
}

/// What a correction's rows changed on the steps (Progress.lua `ns.CorrectionRows`' third return, 2026-10-19, the
/// program's double-check; contract/fixtures/engine.json's correct event `steps`): each ladder that moved, and each
/// step no longer Mastered or Mastered now.
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct StepChanges {
	#[serde(default)]
	pub rungs: Vec<RungChange>,
	#[serde(default)]
	pub mastered: Vec<MasteredChange>,
}

/// A ladder a correction moved: its rung before and after, and the step (the one scale, `ns.Step`) of each.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RungChange {
	pub ladder: u8,
	pub from: u8,
	pub to: u8,
	pub from_step: u8,
	pub to_step: u8,
}

/// A step a correction took the Mastered seal from (`won` false) or gave it (`won`): its ladder, rung and step.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct MasteredChange {
	pub ladder: u8,
	pub rung: u8,
	pub step: u8,
	pub won: bool,
}

/// A ladder's rung now, its best rung, and the days its top was reached at it (a pose family's: the game days a goal
/// pose of its step counted, slice 47; its marking pose's alone until 2026-10-19), with on a pose family those on its
/// step's main pose (`main`) and those answered Perfect (`perfect`) among them (DB.lua `markAttempt`'s `anchorDays` and
/// `perfectDays`; 0 on every other ladder).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rung {
	pub ladder: u8,
	pub rung: u8,
	pub best: u8,
	pub days: i64,
	#[serde(default)]
	pub main: i64,
	#[serde(default)]
	pub perfect: i64,
}

/// Whether a family is open now (Progress.lua `ns.Open`).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Open {
	pub ladder: u8,
	pub open: bool,
}

/// A piece of equipment not added and what it opens (Progress.lua `ns.SetupOpens`, the phones' `SetupOpen`): its key
/// (contract/sheet.json's `gear`), the moves it opens past the setup (a step of `setupSteps` past the steps before it
/// too), and their best quality as the screens count it (1 Starter to 7 Artifact), none where none of them has one.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SetupStep {
	pub key: String,
	pub count: u16,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub quality: Option<u8>,
}

/// Whether a move passes the player's filters now (Progress.lua `ns.Allowed`).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Allowed {
	pub r#move: u16,
	pub allowed: bool,
}

// The Book ------------------------------------------------------------------------------------------------------------

/// The game's sheet the Book reads beside the one progression (the sheet the desktop app shows): its summary's ladder
/// entries (`lad`: rung, the move standing for it, its record, its stars; a pose family's days toward its step, slice
/// 47), earned steps (`ear`: the best rung, then each rung's 0 from the start, 1 kept or the game day it was reached)
/// and progress (`nx`: have, need, have2, need2, on a pose family since slice 47 its main pose's and Perfect days still
/// to come), each by ladder id from 1 (Stats.lua `ns.SheetSummary`, v 6), and its version (`v`, by which the engine
/// reads an older sheet's rungs on a ladder that lost its first rung since, Stats.lua `ns.ShiftSummary`). A sheet from
/// an older addon may lack `ear` and `nx`, and an `nx` entry its fifth and sixth numbers.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct GameLadders {
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub v: Option<i64>,
	pub lad: Vec<Vec<i64>>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub ear: Option<Vec<Vec<i64>>>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub nx: Option<Vec<Vec<i64>>>,
}

/// The Book from the one progression beside the game's sheet (`tools/apppump.lua` `E:book`; the phones' `SheetBook`,
/// `HeroBody` and `BookModel`): every ladder that holds a move, every move's facts, what of the game's sheet it set aside
/// for the app's own fold and the three talent trees (slice 36).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Book {
	pub ladders: Vec<BookLadder>,
	pub moves: Vec<BookMove>,
	pub set_aside: SetAside,
	#[serde(default)]
	pub trees: Vec<BookTree>,
}

/// A talent tree as the Book's Up next reads it (slice 36; `E:book`'s trees, from Progress.lua `ns.TreeGlow`): its key
/// (contract/sheet.json's `trees`), the ladder whose ring glows and why (`kind`: `climb`, the highest ring among the
/// climbs a set can move now, or `trial`, the tree's ladder the Trial places next), or why none glows (`reason`:
/// `rest`, `day` with `at` the earliest moment a set counts again, `pain`, `break` or `none`); the rows `E:book` adds
/// (`ns.TreeRows`) are the engine fixture's, for the iPhone's port, and no page here reads them. Each word a string,
/// never a variant a later kind would break (one unknown variant fails the whole Book).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct BookTree {
	pub tree: String,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub glow: Option<u8>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub kind: Option<String>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub reason: Option<String>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub at: Option<i64>,
}

/// The ring round a ladder's step (Progress.lua `ns.StepRing` through `ns.TreeEntry`, slice 36): its fill in per
/// mille, none while a strength or hold step has no mark yet ("Your first set sets your mark"); the ticks of its stars
/// on a strength or hold step; and the move and the record filling it (a counted cardio step's form nearest its top).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Ring {
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub fill: Option<i64>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub ticks: Option<Vec<i64>>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub r#move: Option<u16>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub record: Option<i64>,
}

/// What a Start from a step's card or a move's window starts (slice 36; Progress.lua `ns.StartFor` through
/// `tools/apppump.lua` `A.start`): the move, a counted set or another all-out window (`allout`), its kind (`as`: own,
/// held, climb, path, check, earn or place; `path` the `proof` until slice 42), the ladder and the rung its slot is
/// dosed at, its goal and the goal's kind (none on a max set, a pose or a timed move), when a set is fresh again,
/// whether the day's sets on it made it `tired`, when and why the climb's day counts again (`dayKind`: session or day),
/// an orange or gold step's earning count, a check's least answer to "How close to the goal pose?" (`ask`, slice 42),
/// the ladder whose climb it runs instead (`via`: opens, needs, reach, check or path), whether it climbs back to a
/// step the player had (`back`) and a Long Haul's length (`haul`, seconds; slice 55); or why it starts nothing (`refused`: setup, off or pain) with what stops it (its `move`, `ladder` and
/// `rung`), Your setup's first rule that leaves that move out (`gap`) and whether the player's own list leaves it out
/// too (`off`). Every word a string, never a variant a later kind would break.
#[expect(clippy::struct_excessive_bools, reason = "the driver's start: one flag a field, as A.start answers it")]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct Start {
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub r#move: Option<u16>,
	#[serde(default, rename = "as", skip_serializing_if = "Option::is_none")]
	pub kind: Option<String>,
	#[serde(default, skip_serializing_if = "is_false")]
	pub allout: bool,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub ladder: Option<u8>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub rung: Option<u8>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub goal: Option<i64>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub goal_kind: Option<String>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub fresh_from: Option<i64>,
	#[serde(default, skip_serializing_if = "is_false")]
	pub tired: bool,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub day_in: Option<i64>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub day_kind: Option<String>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub earn: Option<i64>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub ask: Option<u8>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub via: Option<String>,
	#[serde(default, skip_serializing_if = "is_false")]
	pub back: bool,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub haul: Option<i64>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub refused: Option<String>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub gap: Option<SetupGap>,
	#[serde(default, skip_serializing_if = "is_false")]
	pub off: bool,
}

/// What a Start is asked for: a step by its ladder and rung (a step's card), a move (a move's window), since slice 42
/// a stretch check (a card's "Do the check": the pose step or goal pose a rung's `needs` or a move's `checks` name,
/// Progress.lua `ns.IsCheck`, as `ns.StartFor` takes it), or since slice 55 the Long Haul (the player's cardio step's
/// card, `{haul: true}`: `tools/apppump.lua` `A.haul`, Progress.lua `ns.HaulStart`).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(untagged)]
pub enum Target {
	Step { ladder: u8, rung: u8 },
	Move { r#move: u16 },
	Check { check: StepNeed },
	Haul { haul: bool },
}

/// A move started from a step's card or a move's window ([`crate::Engine::chosen`]): the start the engine read for it
/// at that moment (`tools/apppump.lua` `A.start`) and, unless it refused, the pump it composed (`A.pick`), now the one
/// on screen.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Picked {
	pub start: Start,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub pump: Option<Composed>,
}

/// What the one progression sets aside of the game's sheet after a clear row or a correction, which shows them as they
/// stood until the game folds the row at its next load (`tools/apppump.lua` `setAside`): the ladders started over,
/// whose entries the app's own fold stands for, and the moves whose record was cleared or whose sets this fold
/// corrected, where the fold's record and stars stand instead of the game sheet's (each ascending; the corrected moves
/// since 2026-10-19, the program's double-check).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct SetAside {
	pub ladders: Vec<u8>,
	pub moves: Vec<u16>,
}

/// A ladder as Book's row lists it: the rung of the higher entry of the game's sheet and the app's own (0 for a family
/// not open yet, with what opens it), the move standing for it (the rung's anchor where none does), its record (lifted
/// by the app's own) and the goal past it, its stars (a pose family's: the days its mark counted), the rung's quality (1
/// Starter … 7 Artifact, 0 none), its fourteen steps from the top down and, for an open family, its road to the top
/// (the Path); and its entry in its tree (slice 36, Progress.lua `ns.TreeEntry` on the source its steps read): the
/// ring round its step (none while it waits whole or is back from a break), when and why its climb's day counts again
/// (`dayKind`: session, gap or day), when its standing move is fresh again, why it waits whole (`waits`: closed, setup
/// or off), whether it is back from a break (`placing`; on the cardio ladder `placingBy`, what finds its step again:
/// `window`, its all-out window, from `closing_rung`, or `steady`, a steady move at the step, as Progress.lua
/// `ns.TreeEntry` reads DB.lua `cardioMeasure`), whether a pain flag holds its climb (`pain`: the Tooltip's
/// pain line on every step above the player's, as Tree.lua `Tree.HeldLine` says it) and the app's own rung where the
/// game's sheet stands on another (`appRung`, the step a Start trains); since slice 52 the Stamina Ledger's word on the
/// player's step: what a hold or power ladder's player can do now of the move standing for it (`now`, Progress.lua
/// `ns.HoldNow`: low, middle and high in that move's unit, seconds or power's reps; none before its first record) and
/// Pro's date for the climb (`forecast`, from half to twice its middle as the Path says a span, [`Pace`]:
/// `ns.HoldTopIn` to the move's top on those ladders, `ns.SteadyForecast` on a timed cardio step; none at the top); on
/// the cardio ladder the proving window (`proving`, slice 53) and the Long Haul (`haul`, slice 55).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BookLadder {
	pub ladder: u8,
	pub rung: u8,
	pub r#move: u16,
	pub record: i64,
	/// The ladder's next fresh goal (Progress.lua `ns.FreshGoal`); none before a record.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub next: Option<i64>,
	pub stars: i64,
	pub quality: u8,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub opens: Option<Opening>,
	pub steps: Vec<LadderStep>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub path: Option<LadderPath>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub ring: Option<Ring>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub day_in: Option<i64>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub day_kind: Option<String>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub fresh_from: Option<i64>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub waits: Option<String>,
	#[serde(default, skip_serializing_if = "is_false")]
	pub placing: bool,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub placing_by: Option<String>,
	#[serde(default, skip_serializing_if = "is_false")]
	pub pain: bool,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub app_rung: Option<u8>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub now: Option<[i64; 3]>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub forecast: Option<Pace>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub proving: Option<Proving>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub haul: Option<Haul>,
}

/// Where the proving window stands on the player's cardio step (slice 53, Progress.lua `ns.Proving`): `due` (the next
/// all-out window is it), `next` (the clean days came in this session) or `waits` (a try did not pass this session:
/// "No step up yet."), with the form it names ("Next: {move}, a clean 30-s all-out set.").
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Proving {
	pub kind: String,
	pub r#move: u16,
}

/// The Long Haul on the player's cardio step's card (slice 55): its length open now in seconds (Progress.lua `ns.Haul`,
/// the button's "Long Haul 1:30") and its start (`ns.HaulStart` through `tools/apppump.lua` `startShape`, which a
/// [`Target::Haul`] Start reads again), never refused.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Haul {
	pub seconds: i64,
	pub start: Start,
}

/// A ladder's road from the rung now to its top (the Path, 2026-10-09; `tools/apppump.lua` `path`, the phones'
/// `LadderPath`): every step above the rung now, the next first; the rung it leads to (the first orange or gold step
/// above the rung now, else the top; none on the top step); and the pace to it at the player's own pace, once enough of
/// the ladder's climbs carry a date (`path_climbs`).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct LadderPath {
	pub steps: Vec<PathStep>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub target: Option<u8>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub pace: Option<Pace>,
}

/// One step of the Path: its rung, its step on the one scale and its quality (1 Starter … 7 Artifact), the move
/// standing for it (as its steps name it), what reaching it takes, the days or sets counted toward it (the next step's
/// alone) and what shuts its climb ([`StepState::Shut`], [`StepState::Off`], [`StepState::Waits`] or
/// [`StepState::WaitsStep`]).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PathStep {
	pub rung: u8,
	pub step: u8,
	pub quality: u8,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub r#move: Option<u16>,
	pub need: PathNeed,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub have: Option<i64>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub shut: Option<StepState>,
}

/// What reaching a step of the Path takes from the step under it (Progress.lua `ns.UnlockProgress`'s rules, nothing
/// counted): a first max set that places the ladder; `move` at `top` on `days` game days with the climb's path, a
/// loaded move's top at `kg` or more (`each` hand's; `ns.ClimbKg`, 2026-10-12); a goal pose of the step under it (any,
/// since slice 47; `move` that step's main pose, `ns.MainPose`, whose goal pose the line names) answered close enough
/// on `days`, `ask` the least answer to "How close to the goal pose?" that counts one (`ns.Enum.mark`: 3 Good, 4 Perfect
/// alone below a risky step, `ns.MarkNeeds`), with its path; since slice 42 a counted cardio rung's clean days (`days`,
/// `ns.CleanDays`: game days with a window of one of its `forms` run its `seconds` and saved clean), with its path; a
/// timed cardio rung's full moves under the kind its ready days had (`days`, `cardio_full_days`; slice 52).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PathNeed {
	Unplaced,
	Strength {
		#[serde(default, skip_serializing_if = "Option::is_none")]
		r#move: Option<u16>,
		#[serde(default, skip_serializing_if = "Option::is_none")]
		top: Option<i64>,
		days: i64,
		#[serde(default, skip_serializing_if = "Option::is_none")]
		path: Option<Vec<Stage>>,
		#[serde(default, skip_serializing_if = "Option::is_none")]
		kg: Option<i64>,
		#[serde(default, skip_serializing_if = "is_false")]
		each: bool,
	},
	Mark {
		#[serde(default, skip_serializing_if = "Option::is_none")]
		r#move: Option<u16>,
		days: i64,
		ask: u8,
		#[serde(default, skip_serializing_if = "Option::is_none")]
		path: Option<Vec<Stage>>,
	},
	Clean {
		forms: Vec<u16>,
		days: i64,
		seconds: i64,
		#[serde(default, skip_serializing_if = "Option::is_none")]
		path: Option<Vec<Stage>>,
	},
	Ready {
		days: i64,
	},
}

/// The Path's pace: `low` to `high` of `unit` (`path_units`), the same number at both ends where they round alike.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pace {
	pub unit: PaceUnit,
	pub low: i64,
	pub high: i64,
}

/// The unit a Path's pace counts in.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PaceUnit {
	Days,
	Weeks,
	Months,
	Years,
}

/// What opens a family not open yet, its first entry not done (Progress.lua `ns.OpensAt`; a list since slice 42): a
/// move's record at the family's count, its entry's `count` or else the floor of its window (`seconds`: a hold's), a
/// pose's goal pose answered at least `ask` (`pose`, slice 42: the planche's wrist lean, Perfect), or a ladder's step.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Opening {
	Record { r#move: u16, count: i64, seconds: bool },
	Pose { r#move: u16, ask: u8 },
	Step { ladder: u8, step: u8 },
}

/// A step of another ladder a ladder waits for, as contract/library.json names it (Progress.lua's rung `needs`, a
/// move's `checks` and a ladder's `opens`): that ladder and its step on the one scale, with the record of `count` on
/// `move` (else that step's anchor) where it names one, or since slice 42 `move`, a pose, answered at least `mark` to
/// "How close to the goal pose?" (`ns.Enum.mark`). Whether the player has done it and what it waits for are the
/// engine's ([`crate::Engine::needs`]).
#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct StepNeed {
	pub ladder: u8,
	pub step: u8,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub count: Option<i64>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub r#move: Option<u16>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub mark: Option<u8>,
}

/// An entry of contract/library.json's `needs`, `checks` or `opens` (slice 42): a step of another ladder, or a group
/// done when any one of its steps is (`any`: the tuck flag's 8 pull-ups or 8 paused table or ring rows).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum NeedEntry {
	Any { any: Vec<StepNeed> },
	Step(StepNeed),
}

impl NeedEntry {
	/// Its steps: the one, or an `any` group's in their order.
	#[must_use]
	pub fn steps(&self) -> &[StepNeed] {
		match self {
			NeedEntry::Any { any } => any,
			NeedEntry::Step(step) => std::slice::from_ref(step),
		}
	}
}

/// A step of another ladder as the engine reads it now ([`crate::Engine::needs`], slice 38; `tools/apppump.lua`
/// `Engine:needs`): whether the player has done it (Progress.lua `ns.NeedDone`, the test a climb's `needs`, a move's
/// `checks` and a family's opening walk) and what it waits for (`ns.NeedAt` with the player's filter: the move whose
/// record does it and that record, the first move of the step's chain the setup offers where it leaves the anchor out,
/// or on a pose or cardio step its anchor and no count, a check's pose; none for a step no rung holds), each need row's
/// own check and waits line on a step's card; on a stretch check (slice 42, `ns.IsCheck`; an `any` group's where the
/// choice the setup offers is one, that choice's place in the group from 1, `choice`, the game's `Tree.Alternative`, and
/// its pose) the least answer to "How close to the goal pose?" it counts (`check`, `ns.CheckAsk`) and, once done, how
/// its row says it (`ns.CheckPassed`, as the game and the phones): when it was first passed (`at`, unix seconds, the
/// earliest over the group's checks: "Passed {date}"), else, where no answer did it, the stretch ladder of the choice
/// that is done and its step now (`ladder`, `step`: "Passed: your {ladder} is at step {n}.").
#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct NeedState {
	pub done: bool,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub r#move: Option<u16>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub count: Option<i64>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub check: Option<u8>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub choice: Option<usize>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub at: Option<i64>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub ladder: Option<u8>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub step: Option<u8>,
}

/// One of a ladder's fourteen steps (the Talents tab's step: Tree.lua `Tree.State`, `Tree.KeyLines` and the step's
/// card): its step on the one scale and its quality (1 Starter … 7 Artifact), the rung standing on it, the move
/// standing for that rung and the rung's anchor (none on an empty step), its place against the rung now, when its rung
/// became the player's (none above the best rung) and what its card's Start starts; since slice 42 its path's stages
/// (the card's "Then, for the next step" and, on the player's step, the beads), when it was mastered and, on a step the
/// player has that is not, what mastering it still takes, and on a cardio rung that climbs by clean days the days it
/// takes (`clean`, Progress.lua `ns.CleanDays`); on a timed cardio rung its Pace (slice 52, Progress.lua
/// `ns.SteadyPace`: the seconds its forms keep their pace now, `pace`, of the full move's, `full`).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct LadderStep {
	pub step: u8,
	pub quality: u8,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub rung: Option<u8>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub r#move: Option<u16>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub anchor: Option<u16>,
	pub state: StepState,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub reached: Option<Reached>,
	/// What the step's card starts (slice 36, [`Start`]); none on an empty step.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub start: Option<Start>,
	/// On the cardio ladder, the countable forms of the step's rung the setup allows (Progress.lua `ns.CountForms`
	/// with the player's filter, its pain flags too; slice 40): the forms the rung climbs on, which its card's measure
	/// names; empty on a timed rung and on a counted one whose every form the setup shuts, which climbs by full moves
	/// as a timed one does (slice 52; ready days from slice 42); none on another ladder or an empty step.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub forms: Option<Vec<u16>>,
	/// The stages of the climb from its rung's path in order (`tools/apppump.lua` `pathAt`, Progress.lua `ns.Stages`;
	/// every one done on a step the sheet the steps read stands above); none for a rung without a path.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub stages: Option<Vec<Stage>>,
	/// When the step was mastered (Progress.lua `ns.Mastered`, unix seconds), kept for good.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub mastered: Option<i64>,
	/// On a step the player has that is not mastered yet: its moves the setup offers that reached their top once, of
	/// all of them (`ns.MasteryMoves`, `ns.MasteryLeft`).
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub mastery: Option<Mastery>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub clean: Option<i64>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub pace: Option<i64>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub full: Option<i64>,
}

/// What mastering a step still takes (slice 42): `have` of its `need` moves at their top once.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mastery {
	pub have: i64,
	pub need: i64,
}

/// A step's place against the rung now: no rung on it, held (under the rung now, or reached before a rung down), the
/// rung now (its stars where a move with a window stands for it, and on an orange or gold rung its climb only opened
/// what earns it), the rung above (what its climb takes; shut by the setup, with the rule that shuts it on the app's
/// own source; off by the player's own switch on `move`; waiting for a step of another ladder, a record of `count` on
/// `move`, that step's anchor (`seconds`: a hold's), `ns.RungNeed`, or on a pose or cardio step `ladder` reaching the step
/// `move` stands on (`waitsStep`, 2026-10-10); waiting on a pain flag), or further up.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum StepState {
	Empty,
	Held,
	Now {
		#[serde(default, skip_serializing_if = "Option::is_none")]
		stars: Option<i64>,
		#[serde(default, skip_serializing_if = "Option::is_none")]
		earn: Option<Earn>,
	},
	Next {
		climb: Climb,
	},
	Shut {
		#[serde(default, skip_serializing_if = "Option::is_none")]
		gap: Option<SetupGap>,
	},
	Off {
		r#move: u16,
	},
	Waits {
		r#move: u16,
		count: i64,
		seconds: bool,
	},
	WaitsStep {
		r#move: u16,
		ladder: u8,
	},
	Pain,
	Locked,
}

/// What earns an orange or gold rung the ladder stands on and no set has earned (`tools/apppump.lua` `toEarn`,
/// Progress.lua `ns.EarnFloor`): one fresh set of `move`, the move standing for it, at `count` (`seconds`: a hold's),
/// on a pose family its pose held whole and answered Perfect (`mark`, always true), or on the cardio ladder a fresh
/// clean window of its own form (`clean`, always true; slice 42), each in the driver's shape.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(untagged)]
pub enum Earn {
	Set { r#move: u16, count: i64, seconds: bool },
	Mark { mark: bool },
	Clean { clean: bool },
}

/// What the climb to the next step takes (the Talents tab's next step: Tree.lua `Tree.OpensLine`, `Tree.ClimbHead`;
/// Progress.lua `ns.UnlockProgress`): a first max set that places the ladder; a strength ladder's game days at `top`
/// with `move` and the climb's path, a loaded move's top at `kg` or more (`each` hand's; `ns.ClimbKg`, 2026-10-12); a
/// pose family's days toward its step (slice 47: any goal pose of the step counts one, `ns.PoseNeeds`' `have` of
/// `need`; its marking pose's days alone until 2026-10-19), `move` the step's main pose (`ns.MainPose`; none where a
/// stand-in from below stands for it), the days still to come on that main pose and answered Perfect (`main`,
/// `perfect`: `ns.PoseNeeds`' `mainLeft` and `perfectLeft`, 0 from an older addon's sheet; the card's and the ladder
/// page's line under the climb, Tree.lua `Tree.PoseShort`) with the main pose that line names (`main_pose`, 2026-10-20:
/// `ns.PoseMain`, the one the pumps bring, where the app's state stands on the rung, else `move`, else the step's
/// anchor), `ask` the least answer that counts one (`ns.MarkNeeds`: Perfect, 4, below a
/// harder pose, which the card's climb line and the hover name), with its path; since slice 42 a counted cardio rung's
/// clean days (`ns.CleanDays`) of its countable `forms`, with its path, and a timed cardio rung's full moves under the
/// kind its ready days had (`cardio_full_days`; slice 52). Each path with its next stage (`next`, [`PathNext`]; 2026-10-19).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum Climb {
	Unplaced,
	Strength {
		have: i64,
		need: i64,
		#[serde(default, skip_serializing_if = "Option::is_none")]
		r#move: Option<u16>,
		#[serde(default, skip_serializing_if = "Option::is_none")]
		top: Option<i64>,
		#[serde(default, skip_serializing_if = "Option::is_none")]
		path: Option<Vec<Stage>>,
		#[serde(default, skip_serializing_if = "Option::is_none")]
		kg: Option<i64>,
		#[serde(default, skip_serializing_if = "is_false")]
		each: bool,
		#[serde(default, skip_serializing_if = "Option::is_none")]
		next: Option<PathNext>,
	},
	Mark {
		have: i64,
		need: i64,
		#[serde(default, skip_serializing_if = "Option::is_none")]
		r#move: Option<u16>,
		#[serde(default, skip_serializing_if = "Option::is_none")]
		main_pose: Option<u16>,
		#[serde(default)]
		main: i64,
		#[serde(default)]
		perfect: i64,
		#[serde(default, skip_serializing_if = "Option::is_none")]
		ask: Option<u8>,
		#[serde(default, skip_serializing_if = "Option::is_none")]
		path: Option<Vec<Stage>>,
		#[serde(default, skip_serializing_if = "Option::is_none")]
		next: Option<PathNext>,
	},
	Clean {
		have: i64,
		need: i64,
		forms: Vec<u16>,
		#[serde(default, skip_serializing_if = "Option::is_none")]
		path: Option<Vec<Stage>>,
		#[serde(default, skip_serializing_if = "Option::is_none")]
		next: Option<PathNext>,
	},
	Ready {
		have: i64,
		need: i64,
	},
}

/// The climb's next stage as every Next and key line names it (Progress.lua `ns.PathNext` through `tools/apppump.lua`
/// `pathNext`, 2026-10-19, the program's double-check: the game's `Tree.PathNext` and the phones' ports read the same
/// rule): its place in the path from 1 (`stage`), the move to name (of an open stage the one the setup offers nearest
/// its days; where none is open, the move that holds the climb), its top (`ns.MoveTop`) with a loaded move's weight
/// (`kg`, whole kg, `each` in each hand: `ns.ClimbKg`, as the game's `Sheet.ClimbTop` words it), the game days the
/// stage takes and those it has, and where that move is not open what holds it (`waits`: a check of its own, or the
/// move whose top opens its stage); none where the path is in or the rung has none.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PathNext {
	pub stage: usize,
	pub r#move: u16,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub top: Option<Top>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub kg: Option<i64>,
	#[serde(default, skip_serializing_if = "is_false")]
	pub each: bool,
	pub days: i64,
	pub have: i64,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub waits: Option<MoveWaits>,
}

/// A stage of a climb's path (slice 42; Progress.lua `ns.Stages` through `tools/apppump.lua` `pathAt`), in order: its
/// state, the move to name (of those the setup offers the nearest its days; on a skipped stage the one its skip line
/// names), its moves (`ids`, any one of which does it), the game days at its top it takes and those it has (none on a
/// done stage, which counts nothing more: slice 42's fix round, 2026-10-17), its top (a count, a pose's least answer, or
/// a countable form's clean window, `true`) and each of its moves' own (`tops`: an alternative's may differ), its moves
/// the setup offers (`offered`), on a skipped stage why (`skip`: `off`, the player's own list, or `setup`) and, waiting,
/// the move whose top opens it with that top (`after`, `after_top`) and a loaded opener's weight (`after_kg`, whole kg,
/// `after_each` in each hand: `ns.ClimbKg`).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Stage {
	pub state: StageState,
	pub r#move: u16,
	pub ids: Vec<u16>,
	pub days: i64,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub have: Option<i64>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub top: Option<Top>,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub tops: Vec<StageTop>,
	/// Its moves the setup offers (the setup alone: a stage's line joins only these, slice 42's fix round).
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub offered: Vec<u16>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub skip: Option<String>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub after: Option<u16>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub after_top: Option<Top>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub after_kg: Option<i64>,
	#[serde(default, skip_serializing_if = "is_false")]
	pub after_each: bool,
}

/// A move of a stage with its own top (`Stage::tops`, Progress.lua `ns.MoveTop`).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct StageTop {
	pub r#move: u16,
	pub top: Top,
}

/// A stage's state: in, skipped (the setup shuts every move of it), open (its moves offered) or waiting for the move
/// before it (`ns.Stages`); one this build does not know (`other`, a newer engine's).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StageState {
	Done,
	Skipped,
	Open,
	Waiting,
	#[serde(other)]
	Other,
}

/// A move's top on a climb (Progress.lua `ns.MoveTop`): a count (reps, a hold's seconds, a pose's least answer) or, for
/// a countable cardio form, a clean window (`true`).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(untagged)]
pub enum Top {
	Count(i64),
	Clean(bool),
}

/// When a step's rung became the player's: from the start, kept (on a day the sheet does not know), or first reached
/// on a game day (days since 1970-01-01, a civil date).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Reached {
	Start,
	Kept,
	Day { day: i64 },
}

/// The first rule of the player's setup that leaves a move out (Your setup's words name it by `key`, contract/sheet.json
/// `gear`): a piece of equipment not added (`gear`), a switch off that lets it in (`off`: floor, space), a switch on
/// that leaves it out (`on`: quiet, nojump, nostrain, noheaddown).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SetupGap {
	pub kind: GapKind,
	pub key: String,
}

/// How a setup's rule leaves a move out ([`SetupGap`]).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum GapKind {
	Gear,
	Off,
	On,
}

/// A move as the Book shows it: its record and last count, its familiarity's step (0 new … 3 learned) and its days, it
/// is trained (finished once, or a record here or on the game's sheet), above the player's reach, switched off in the
/// pumps (left out, or an opt-in move not switched on), the setup's rule that leaves it out, its next goal (one step
/// past its record), its own rung's quality and step on its own ladder and the stat it raises; on a loaded move the
/// weighted record and the last set's weight (tenths of a kg); the move that takes its place in the player's pumps
/// while one is set aside (`better`, Progress.lua `ns.BetterOpen`); and since slice 42 whether it reached its top once
/// (`topped`, `ns.TopOnce`: a step card's topped mark), what it waits for while it is not open (`waits`,
/// `ns.MoveOpen`), and where that is a check of its own the stage of a step's path that holds it too (`path_waits`,
/// `ns.PathWaits`, slice 42's fix round: its window's line beside the check's), the pose that helps it where the
/// setup offers that pose (`hint`, the card's soft need) and the gear that carries a loaded move's load for the
/// player's setup (`implement`, `ns.Implement`: the How and setup line its page and window show, 2026-10-24).
#[expect(clippy::struct_excessive_bools, reason = "the phones' SheetBook move: one flag a field")]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BookMove {
	pub r#move: u16,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub record: Option<i64>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub last: Option<i64>,
	pub familiarity: u8,
	pub days: i64,
	pub trained: bool,
	pub above: bool,
	pub off: bool,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub gap: Option<SetupGap>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub next: Option<i64>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub quality: Option<u8>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub step: Option<u8>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub stat: Option<String>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub top_load: Option<u32>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub top_reps: Option<i64>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub last_load: Option<u32>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub better: Option<u16>,
	#[serde(default, skip_serializing_if = "is_false")]
	pub topped: bool,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub waits: Option<MoveWaits>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub path_waits: Option<MoveWaits>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub hint: Option<u16>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub implement: Option<String>,
}

/// What a move not open yet waits for (slice 42, Progress.lua `ns.MoveOpen`): a check of its own (`check`, its place
/// among its library `checks` from 1), or the stage of a step's path it is in (that step's `ladder` and `rung`, the
/// stage from 1) with the move whose top opens it (`after`), that top and a loaded opener's weight (`kg`, `each`:
/// `ns.ClimbKg`).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum MoveWaits {
	Check {
		check: usize,
	},
	Path {
		ladder: u8,
		rung: u8,
		stage: usize,
		#[serde(default, skip_serializing_if = "Option::is_none")]
		after: Option<u16>,
		#[serde(default, skip_serializing_if = "Option::is_none")]
		top: Option<Top>,
		#[serde(default, skip_serializing_if = "Option::is_none")]
		kg: Option<i64>,
		#[serde(default, skip_serializing_if = "is_false")]
		each: bool,
	},
}

/// An owed move that rides as another (`tools/apppump.lua` `Engine:ridesAs`, Engine.lua `ns.RidesAs`, 2026-10-29: an
/// owed backup of its rung the setup leaves unneeded rides as the move that stands for that rung): the move as owed
/// now and the move it rides as.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct RidesAs {
	pub r#move: u16,
	#[serde(rename = "as")]
	pub stand_in: u16,
}

/// A catch-up's move that asks its count (`tools/apppump.lua` `A.catchup`, since 2026-10-12): its place in the
/// catch-up (from 0), its slot as the game doses an owed move (Engine.lua `riderSlot`, at the owed row's ladder and
/// rung) and the move as a device runs it; as the `dosed` op answers it after Swap (since 2026-10-26; since 2026-11-02
/// the move Swap put in place of one that runs by its time too, which asks what its own kind asks), whether Swap put
/// that slot in place of the owed move's (`Pump.swapped`, the engine's own record; present only when true).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Dosed {
	pub index: usize,
	pub slot: Slot,
	#[serde(rename = "move")]
	pub planned: PlannedMove,
	#[serde(default, skip_serializing_if = "is_false")]
	pub swapped: bool,
}

/// A move's result written ([`crate::Engine::finish`], `tools/apppump.lua` `Engine:finish`): the rows as written (its
/// load row, its row, the rung rows the rules add, an easier pick's step down), the cardio day the set brought, as
/// the game's Result names it (Events.lua `Lc.WriteResult`'s `day`, Progress.lua `ns.CardioDay`), none where it
/// brought none, the week's embers where its row lit the game day's last ember (slice 55, the game's "Three
/// embers" toast: Stats.lua's `embers` event), none otherwise, and whether its row lights an ember (Stats.lua
/// `ns.LightsEmber`; the Result's Cardio today, 2026-10-29).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Finished {
	pub rows: Vec<String>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub day: Option<CardioDay>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub kindled: Option<i64>,
	#[serde(default)]
	pub lit: bool,
}

/// The cardio day a set brought (Progress.lua `ns.CardioDay`): a clean window's (slice 42) or a full move's (slice 52)
/// day toward the next cardio step, `have` of `need`, into `step`; a Long Haul's day at `time` seconds toward its next
/// length `next` (slice 55); a step's measure back from a break, the step it `holds` or goes `back` to (slice 53); or a
/// proving window that did not pass (`waits`, slice 53: "No step up yet."). Every word a string kind, an unknown one
/// read as none (`Other`).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum CardioDay {
	Clean {
		have: i64,
		need: i64,
		step: i64,
	},
	Full {
		have: i64,
		need: i64,
		step: i64,
	},
	Haul {
		time: i64,
		have: i64,
		need: i64,
		next: i64,
	},
	Holds {
		step: i64,
	},
	Back {
		step: i64,
	},
	Waits,
	#[serde(other)]
	Other,
}

/// Today's embers (slice 55, Stats.lua `ns.Embers`): the cardio moves done in full this game day (`today`) of the
/// most a day lights (`most`, `embers_day`), Pump home's embers row; and the week's omen on the player's setup and
/// cardio step (`ns.Omen`, slice 54: its key, `hare` … `oak`, the weekly vault's "Hare week").
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct Embers {
	pub today: i64,
	pub most: i64,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub omen: Option<String>,
}

/// A catch-up's set judged as the game's own catch-up judges it (`Engine:judge`: `ns.Judge`, `ns.ResultRow`,
/// `ns.LoadRow`, `ns.Progression`) and written nowhere: a typed count, and since slice 42's fix round (2026-10-17) a
/// counted window saved with no count, Form slipped or (slice 53) Not all out and a steady move's answered pace
/// (outcome done, its answer in the flags); its move row's reps, seconds, outcome and flags, the weight its load row
/// carries (tenths of a kg; none without one), the rung rows the rules add and the steps it mastered, which the helper
/// writes as the catch-up's (contract/companion-api.yaml `WristEvent`'s `set`).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Judged {
	pub reps: i64,
	pub seconds: i64,
	pub outcome: u8,
	pub flags: u16,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub load: Option<u32>,
	#[serde(default)]
	pub rungs: Vec<JudgedRung>,
	#[serde(default)]
	pub mastered: Vec<JudgedStep>,
}

/// A rung row a catch-up's count calls for (Progress.lua `ns.Progression`): the ladder, the rung and its cause
/// (`ns.RungCause`).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct JudgedRung {
	pub ladder: u8,
	pub rung: u8,
	pub cause: u16,
}

/// A ladder's step a catch-up's result names: one its set mastered (`ns.Progression`'s mastered row, kind 19), or the
/// rung an easier pick steps down to ([`Judgement::down`]): its ladder and rung.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct JudgedStep {
	pub ladder: u8,
	pub rung: u8,
}

/// A catch-up's move judged ([`crate::Engine::judge`], `tools/apppump.lua` `Engine:judge`): its set, none (nothing of it
/// folded) for a result that is no set, and, set or not, the step down an easier pick makes (Progress.lua
/// `ns.EasierDown`, as the game's `Lc.WriteResult` writes it after every outcome: since 2026-10-26 a move Swap took to
/// its easier version), folded in the state as the set is, none where it makes none.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct Judgement {
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub set: Option<Judged>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub down: Option<JudgedStep>,
}

/// What a stretch check opens (Progress.lua `ns.AllChecks`' `opens`): a move, a family's first step, or the climb into
/// `rung` of `ladder`.
#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CheckOpens {
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub r#move: Option<u16>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub family: Option<u8>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub ladder: Option<u8>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub rung: Option<u8>,
}

/// The check a card or a window started ("Do the check", slice 42): its entry where the start named it (a check row's;
/// none for a card's or a move window's own Start of a check) and what that card names it opening (none where it named
/// none), from which [`crate::Engine::checks`] puts the check the pump runs first (`ns.RunsCheck`, slice 42's last
/// review: the one its climb waits for first where one does).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct StartedCheck {
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub need: Option<StepNeed>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub opens: Option<CheckOpens>,
}

/// What a check a pose could pass came to after its answer (`Pump:checked`, slice 42's fix round): its entry, passed or
/// not and, passed, whether what it opens is open now (`ns.OpensNow`: "Check passed: {move} is open" only then) with
/// what still holds it, the move or climb it opens (`name`: the move it opens, a family's first step's anchor, or the
/// climb's `ladder` and `step`), and whether it is the check a check's pump runs (`runs`, slice 42's last review,
/// 2026-10-18: `ns.RunsCheck` from the check the card or the window started, which may be the one its climb waits for
/// first; "Not there yet." only where that one did not pass).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CheckOutcome {
	pub need: StepNeed,
	pub passed: bool,
	#[serde(default)]
	pub open: bool,
	#[serde(default)]
	pub runs: bool,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub name: Option<u16>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub ladder: Option<u8>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub step: Option<u8>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub waits: Option<CheckWait>,
}

/// What still holds what a passed check opens, as the Result words it (`ns.OpensNow`'s answer): another entry not done
/// (`check`: the move the setup offers for it, its record's count where one does it, its least answer where it is a
/// stretch check, its move's ladder, a hold's count in seconds), or the stage of a step's path (`path`: the move whose
/// top opens it, that top and a loaded opener's weight, `kg` and `each`: `ns.ClimbKg`).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum CheckWait {
	Check {
		#[serde(default, skip_serializing_if = "Option::is_none")]
		r#move: Option<u16>,
		#[serde(default, skip_serializing_if = "Option::is_none")]
		count: Option<i64>,
		#[serde(default, skip_serializing_if = "Option::is_none")]
		ask: Option<u8>,
		#[serde(default, skip_serializing_if = "Option::is_none")]
		ladder: Option<u8>,
		#[serde(default)]
		seconds: bool,
	},
	Path {
		#[serde(default, skip_serializing_if = "Option::is_none")]
		after: Option<u16>,
		#[serde(default, skip_serializing_if = "Option::is_none")]
		top: Option<Top>,
		#[serde(default, skip_serializing_if = "Option::is_none")]
		kg: Option<i64>,
		#[serde(default, skip_serializing_if = "is_false")]
		each: bool,
	},
}

// The ops' arguments -------------------------------------------------------------------------------------------------

#[derive(Serialize, Deserialize)]
pub(crate) struct Init {
	pub profile: Profile,
	pub stamp: u16,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct SetProfile {
	pub profile: Profile,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct At {
	pub at: i64,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct GameAt {
	pub at: i64,
	pub reset: Option<i64>,
}

/// The `gameSave` op's argument: the save's CBOR in standard Base64, or none.
#[derive(Serialize, Deserialize)]
pub(crate) struct GameSaveAt {
	pub save: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Fold {
	pub own: Vec<String>,
	pub game: Vec<String>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub sheet_focus: Option<u8>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Merge {
	pub own: Vec<String>,
	pub game: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Pain {
	pub joints: Vec<u8>,
	pub on: bool,
	pub at: i64,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Code {
	pub code: u8,
	pub at: i64,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ClearRow {
	pub scope: Clear,
	pub id: u16,
	pub at: i64,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct MovesAt {
	pub moves: Vec<u16>,
	pub at: i64,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Ladders {
	pub ladders: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Count {
	pub count: u8,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Moves {
	pub moves: Vec<u16>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Xps {
	pub xps: Vec<i64>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Compose {
	pub id: String,
	pub kind: String,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub family: Option<String>,
	pub tier: u8,
	pub at: i64,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub r#match: Option<Match>,
	#[serde(flatten)]
	pub unfinished: Unfinished,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Quick {
	pub id: String,
	pub at: i64,
	#[serde(flatten)]
	pub unfinished: Unfinished,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ExpeditionAsk {
	pub id: String,
	pub stage: ExpeditionStage,
	pub regions: Vec<String>,
	pub at: i64,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ExpeditionAt {
	pub budget: i64,
	pub elapsed: i64,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub last: Option<ExpeditionStage>,
	pub regions: Vec<String>,
	pub at: i64,
}

/// An expedition as `A.expeditionNext` reads it (`ExpeditionAt` less its instant).
#[derive(Serialize)]
pub(crate) struct Exp {
	pub budget: i64,
	pub elapsed: i64,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub last: Option<ExpeditionStage>,
	pub regions: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Minutes {
	pub minutes: u16,
	pub at: i64,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Regions {
	pub regions: Vec<String>,
}

/// An expedition round's stage (slice 51; Engine.lua `ns.EXPEDITION_STAGES`): the warm-up, an inner round (the Normal
/// pump), the last round (the Big pump, the expedition's one all-out effort) and the cool-down.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ExpeditionStage {
	Warmup,
	Inner,
	Last,
	Cooldown,
}

/// A round of an expedition's preview: its stage and its moves.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PreviewRound {
	pub stage: ExpeditionStage,
	pub slots: Vec<Slot>,
}

/// The plan of an expedition of some minutes (slice 51, `A.expeditionPreview`: the expedition's rules played on a copy
/// of the engine, every move done at its goal; "never a promise", its words say "about"): its rounds, the moves in all,
/// the records to beat, the record moves' regions and the estimate in seconds, the breathers in.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ExpeditionPreview {
	pub rounds: Vec<PreviewRound>,
	pub moves: u32,
	pub records: u32,
	pub regions: Vec<String>,
	pub seconds: i64,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Chosen {
	pub id: String,
	pub target: Target,
	pub at: i64,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Starts {
	pub moves: Vec<u16>,
	pub at: i64,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct StartAt {
	pub target: Target,
	pub at: i64,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct NeedsAt {
	pub needs: Vec<NeedEntry>,
	pub at: i64,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct RowsAt {
	pub rows: Vec<String>,
	pub at: i64,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Catchup {
	pub id: String,
	pub rows: Vec<String>,
	pub pays: Vec<usize>,
	pub at: i64,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct IndexAt {
	pub index: usize,
	pub at: i64,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Index {
	pub index: usize,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ChecksAt {
	pub index: usize,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub started: Option<StartedCheck>,
}

/// A move's callout on its Ready view: its index (from 0), the moves of a catch-up (`count`; none for the pump on
/// screen) and a catch-up's move's place in it (`place`, from 0, `Dosed::index`: the slot the catch-up on screen dosed
/// it as; none for a move it did not dose), the moment, and the streak the board shows (`before`) and makes with the
/// day's first performed move (`after`, none once today has one).
#[derive(Serialize, Deserialize)]
pub(crate) struct CalloutAt {
	pub index: usize,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub count: Option<usize>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub place: Option<usize>,
	pub at: i64,
	pub before: i64,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub after: Option<i64>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Swap {
	pub index: usize,
	pub take: Take,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct AfterPain {
	pub from: usize,
	pub recorded: bool,
	pub at: i64,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct NextGoal {
	pub r#move: u16,
	pub value: i64,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct NextLoadAt {
	pub r#move: u16,
	pub at: i64,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Validate {
	pub index: usize,
	pub value: i64,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub load: Option<u32>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct CorrectableAt {
	pub r#move: u16,
	pub at: i64,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub horizon: Option<i64>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct CorrectCheckAt {
	pub set: SetKey,
	pub value: i64,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub load: Option<u32>,
	pub at: i64,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub horizon: Option<i64>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct CorrectRowsAt {
	pub set: SetKey,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub value: Option<i64>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub load: Option<u32>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub answer: Option<u8>,
	pub at: i64,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub horizon: Option<i64>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct BookAt {
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub game: Option<GameLadders>,
	pub at: i64,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Finish {
	pub index: usize,
	pub result: MoveResult,
	pub at: i64,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct HoldAskAt {
	pub index: usize,
	pub value: i64,
	pub at: i64,
}
