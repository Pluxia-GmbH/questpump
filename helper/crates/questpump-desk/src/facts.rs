//! The contract's files the desktop app reads, built in and parsed once (docs/plan.md slice 35): the game's words of
//! `contract/sheet.json` (the addon's own strings, as the phones' `SheetVocab` reads them), the library's facts and
//! tunables of `contract/library.json` (Library.lua, Progress.lua and Constants.lua through tools/gen-engine.lua) and
//! the moves' words of `contract/moves.json` (tools/gen-fixtures.lua). Every area reads its view of them here: Hero and
//! Pump home (`hero.rs`, `home.rs`), the Book and the Log (`book.rs`, `log.rs`), the Menu's pages (`menu_pages`), the
//! pump (`pump.rs`, `pump/view.rs`, `pump/words.rs`) and the figures' fault words (`figures.rs`); and the library's
//! rules more than one of them reads live here once (a move's joints, a step's and a rung's quality, a move's group).
//!
//! The shapes are the generators': a field every entry of a file carries is required, so a renamed key or a changed
//! type fails the whole file rather than reading as empty; one some entries lack is optional. `make gen-check` keeps
//! the files the addon's at this build's commit; a file that does not read (never in a build `make check` passed) is
//! logged once, here, at the first read, and reads as empty. Each caller says at its site what it shows then: the pump,
//! Hero, Pump home, the Menu's pages and the figures show what is left, the Book and the Log nothing (`Facts::whole`).

use std::collections::BTreeMap;
use std::sync::LazyLock;

use questpump_engine::NeedEntry;
use serde::{Deserialize, Serialize};

// contract/sheet.json ------------------------------------------------------------------------------------------------

/// A thing of the sheet's by its key and its name in the game's words (a stat, a region, a block).
#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq, Eq)]
pub struct Named {
	pub key: String,
	pub name: String,
}

/// A group of the equipment (2026-11-03): its key and its pieces' keys in `gear_order`.
#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq, Eq)]
pub struct GearGroup {
	pub key: String,
	pub gear: Vec<String>,
}

/// A thing of the sheet's by its id and its name (a muscle, a joint).
#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq, Eq)]
pub struct ById {
	pub id: u8,
	pub name: String,
}

/// A focus as the game's focus window names it: its code, its key and its name (no line under it since 2026-10-22).
#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq, Eq)]
pub struct Focus {
	pub id: u8,
	pub key: String,
	pub name: String,
}

/// A title: its code (`ns.Enum.title`), its key (the profile's `title`) and its name.
#[derive(Deserialize, Debug, Default)]
pub struct Title {
	pub id: u8,
	pub key: String,
	pub name: String,
}

/// A ladder in the sheet's words: its family's name (none on a retired id), its kind and dose, the stat it feeds and
/// each rung's step on the one scale.
#[derive(Deserialize, Debug, Default)]
pub struct SheetLadder {
	pub id: u8,
	#[serde(default)]
	pub name: String,
	pub kind: String,
	pub dose: String,
	pub stat: Option<String>,
	pub steps: Vec<u8>,
}

/// A talent tree (slice 36; Core.lua `ns.TREES`): its key, its name in the game's words (the focus's), and its branches
/// in their order, each with its ladders in column order.
#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq, Eq)]
pub struct SheetTree {
	pub key: String,
	pub name: String,
	pub branches: Vec<Branch>,
}

/// A branch of a talent tree: its key (no name shows: its gap is the branch) and its ladders in column order.
#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq, Eq)]
pub struct Branch {
	pub key: String,
	pub ladders: Vec<u8>,
}

/// The check-in's words (Panel.lua's `RenderCheckin`).
#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq, Eq)]
pub struct CheckinWords {
	pub question: String,
	pub fine: String,
	pub sore: String,
	pub dark: String,
}

/// The joints question's words: its title.
#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq, Eq)]
pub struct PainWords {
	#[serde(rename = "where")]
	pub asked: String,
}

/// The count step's words.
#[derive(Deserialize, Debug, Default)]
pub struct CountWords {
	pub max: String,
	pub confirm: String,
	pub confirm_first: String,
	/// A weight far over the one planned (Progress.lua `ns.Validate`'s `heavy`): the weight typed, then the plan, in kg.
	pub confirm_kg: String,
	pub confirm_low: String,
	/// A counted set's one question (slice 42's fix round, 2026-10-17; the game's `ui.result.confirmBest`): the clean
	/// count far above the form's best, then that best.
	pub confirm_best: String,
	pub really_yes: String,
	pub really_keep: String,
	pub really_change: String,
	pub seconds: String,
	pub empty: String,
	pub could_not: String,
	pub could_not_hold: String,
}

/// The felt-unwell title.
#[derive(Deserialize, Debug, Default)]
pub struct UnwellWords {
	pub title: String,
}

/// Swap's words (Panel.lua `swapWords` and its Swap menu's title), each offer's with the move's name for its `%s`.
#[derive(Deserialize, Debug, Default)]
pub struct SwapWords {
	pub title: String,
	pub another: String,
	pub easier: String,
	pub backup: String,
	pub closest: String,
	pub back: String,
}

/// The Ready view's target lines; since slice 42 the next-step line under the goal where the climb waits on the move's
/// stage of its path alone (Panel.lua `nextStepLine`): what opens the next step on a strength or hold move (`%s` the
/// amount, `%d` the days), a pose's days close enough or Perfect, a counted form's clean days, each with its one form.
#[derive(Deserialize, Debug, Default)]
pub struct TargetWords {
	pub first: String,
	pub first_hold: String,
	pub first_hi: String,
	pub first_hi_hold: String,
	pub placing: String,
	pub allout: String,
	pub count: String,
	pub next: String,
	pub next_one: String,
	pub next_pose: String,
	pub next_pose_one: String,
	pub next_perfect: String,
	pub next_perfect_one: String,
	pub next_clean: String,
	pub next_clean_one: String,
}

/// A goal's amount in the game's goal row (Panel.lua `goalOf`, slice 42's next-step line): the count with its plus
/// (`%d+`), its unit by the count (a rep, reps, a side move's each side, seconds) and a loaded climb's weight (`%s`
/// the amount, `%d` the kg).
#[derive(Deserialize, Debug, Default)]
pub struct GoalWords {
	pub at_least: String,
	pub reps: String,
	pub reps_one: String,
	pub reps_side: String,
	pub reps_side_one: String,
	pub secs: String,
	pub secs_side: String,
	pub climb: String,
	pub climb_each: String,
}

/// The game's words (`sheet.json` `words`); `%d` and `%s` stand for a number and a word (`pump::words::fill`).
#[derive(Deserialize, Debug, Default)]
pub struct SheetWords {
	/// The name line's grammar, which Hero's page cuts at its seam.
	pub titled: Option<String>,
	pub titled_suffix: Option<String>,
	pub checkin: CheckinWords,
	pub pain: PainWords,
	pub count: CountWords,
	pub unwell: UnwellWords,
	pub swap: SwapWords,
	pub target: TargetWords,
	pub goal: GoalWords,
}

/// `contract/sheet.json`: the names of the game's sheet and the addon's words the screens show.
#[derive(Deserialize, Debug, Default)]
pub struct Sheet {
	pub stats: Vec<Named>,
	pub muscles: Vec<ById>,
	/// The qualities' words, 1 Starter … 7 Artifact (`qualities[0]` is Starter's).
	pub qualities: Vec<String>,
	pub ladders: Vec<SheetLadder>,
	/// Every pose's mark by its move's key (Progress.lua `ns.MarkText`): the sentence the players read as its goal pose.
	pub marks: BTreeMap<String, String>,
	/// The ladders' order by stat (Core.lua `ns.LADDER_ORDER`), which the Book's list keeps.
	pub order: Vec<u8>,
	/// The three talent trees (slice 36), each of the live ladders in one of them once.
	pub trees: Vec<SheetTree>,
	pub regions: Vec<Named>,
	pub blocks: Vec<Named>,
	pub focus: Vec<Focus>,
	pub titles: Vec<Title>,
	/// The beasts by their build's key.
	pub builds: BTreeMap<String, String>,
	pub joints: Vec<ById>,
	/// The groups' names by their key (Your moves, the Moves search).
	pub groups: BTreeMap<String, String>,
	/// The equipment's names by their key, and the order the game lists them in.
	pub gear: BTreeMap<String, String>,
	pub gear_order: Vec<String>,
	/// The equipment's groups in their order (2026-11-03, Library.lua `ns.GearGroups`): each its key (its words
	/// `places.gear.group.<key>`) and its pieces in `gear_order`, every piece in one.
	pub gear_groups: Vec<GearGroup>,
	/// Each piece of equipment's explanation by its key, the game's equipment tooltip's words (what counts as it and
	/// what it is for; ns.Settings.GearTip).
	pub gear_tips: BTreeMap<String, String>,
	/// The experience question (`title`) and its answers by their keys.
	pub experience: BTreeMap<String, String>,
	/// The safety note, word for word the addon's: the Menu's Safety note page alone shows it (David, 2026-10-24: only
	/// what is legally necessary).
	pub safety: String,
	pub words: SheetWords,
}

// contract/library.json ----------------------------------------------------------------------------------------------

/// A piece of equipment a move needs, or any one of several.
#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Need {
	One(String),
	Any(Vec<String>),
}

impl Need {
	/// The equipment's keys that do.
	#[must_use]
	pub fn keys(&self) -> &[String] {
		match self {
			Need::One(key) => std::slice::from_ref(key),
			Need::Any(keys) => keys,
		}
	}
}

/// A move as the library tags it.
#[derive(Deserialize, Debug, Default)]
pub struct Move {
	pub ladder: Option<u8>,
	pub rung: u8,
	/// The move standing for its ladder's rung, the one its window is set for (Progress.lua `ns.Anchor`): the rung's
	/// unlock shows it, and a window counts in its unit.
	#[serde(default)]
	pub anchor: bool,
	pub dose: String,
	pub pattern: String,
	pub region: Option<String>,
	pub block: String,
	pub group: String,
	pub needs: Vec<Need>,
	/// The muscles it trains as primary and as secondary.
	pub mp: Vec<u8>,
	pub ms: Vec<u8>,
	/// The joints it loads, by their keys (`Library::joints`).
	pub pain: Vec<String>,
	/// Offered only once checked.
	#[serde(default, rename = "optIn")]
	pub opt_in: bool,
	/// A pose's seconds (each side's on a side move).
	pub secs: Option<i64>,
	/// A countable cardio form: its all-out window keeps a count, never a goal (slice 42: its clean windows count its
	/// step's clean days; its window of counts until then).
	#[serde(default)]
	pub count: bool,
	/// How many times the plain steady length a steady form runs (Progress.lua `ns.Span`: a pad walk's 2); none: once.
	pub span: Option<i64>,
	/// A loaded move: `total` (one weight) or `each` (a weight in each hand), its sets typed with a weight in kg.
	pub load: Option<String>,
	/// The steps of other ladders it waits for before any pump, Swap or Start offers it (slice 42, Progress.lua
	/// `ns.MoveOpen`): a stretch check (a pose step or goal pose) or a record; whether each is done is the engine's.
	#[serde(default)]
	pub checks: Vec<NeedEntry>,
	/// A cardio form's road (slice 54, `Library::roads`): a cardio step's card lists its forms in the roads' order.
	pub road: Option<String>,
}

/// A count's window, its floor and its top (a rung's for its anchor, or for one move of it).
#[derive(Deserialize, Serialize, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Window {
	pub lo: i64,
	pub hi: i64,
}

/// A rung of a ladder: its step on the one scale, its anchor's window (a strength or hold rung's: none on cardio and
/// on a pose family) and its other moves' own windows, the steps of other ladders its climb waits for and its path.
#[derive(Deserialize, Debug, Default)]
pub struct Rung {
	pub step: u8,
	pub lo: Option<i64>,
	pub hi: Option<i64>,
	#[serde(default)]
	pub alt: BTreeMap<u16, Window>,
	#[serde(default)]
	pub needs: Vec<NeedEntry>,
	/// The stages the climb from this rung waits for, in order (Progress.lua's rung `path`, slice 42).
	#[serde(default)]
	pub path: Vec<Stage>,
}

/// A stage of a rung's path: its moves, any one of which does it (when it opens and its days are the engine's:
/// `questpump_engine::Stage`).
#[derive(Deserialize, Debug, Default)]
pub struct Stage {
	pub ids: Vec<u16>,
}

/// A ladder of the library, and what opens it where it is a family not open from the start: every entry of its
/// `opens` done (a list since slice 42).
#[derive(Deserialize, Debug, Default)]
pub struct Ladder {
	pub id: u8,
	pub key: String,
	pub rungs: Vec<Rung>,
	#[serde(default)]
	pub opens: Vec<NeedEntry>,
}

/// The tunables the desktop app reads (Constants.lua's, by the same names).
#[derive(Deserialize, Debug, Default)]
pub struct Constants {
	/// The game days' turn.
	pub day_offset_h: i64,
	/// The game days the feedback numbers count, today's among them (the anonymous counts' review).
	pub feedback_days: i64,
	/// A step's colour on the one scale (0 Starter … 6 Artifact), by step from 1.
	pub quality_by_step: Vec<u8>,
	/// The weeks' marks' steps.
	pub map_steps: Vec<i64>,
	/// The rest tokens every player starts holding (slice 51, the streak's freeze).
	pub token_start: i64,
	pub token_days: i64,
	pub token_max: i64,
	pub xp_base: f64,
	pub xp_exp: f64,
	pub level_cap: i64,
	pub rested_min_h: f64,
	pub rested_per_day: f64,
	pub rested_cap_levels: f64,
	/// The level the focus window asks at, once (Sheet.lua's `AskFocus`).
	pub focus_level: i64,
	pub fam_learned_days: i64,
	pub pose_unlock_days: i64,
	/// A timed cardio rung's climb: its full moves (slice 52: sessions with a Yes on the step's own form at the steady
	/// ledger's top M; its ready days, slice 42, until then).
	pub cardio_full_days: i64,
	pub star_fractions: Vec<f64>,
	pub timed_ask_share: f64,
	/// A count the app counted (a hold's stopwatch, a set of reps' counted reps or the number it has to beat where that
	/// is higher) takes at most this share more (slice 66, `CountStep::cap`).
	pub count_over_share: f64,
	/// A move ended under these reps or seconds counts nothing and is banked without a word (`Desk::pump_short`).
	pub min_reps: i64,
	pub min_seconds: i64,
	/// A move's time in a pump as the game's owed tip estimates it (Sheet.lua), by which work mode's How page names a
	/// pump's minutes (slice 67, `MenuWork::minutes`).
	pub move_estimate_s: i64,
	/// A saved move's result holds this long before the next move goes on by itself (`Pour::goes_on_at`).
	pub record_hold_s: f64,
	pub mark_s: i64,
	pub mark_to: i64,
	pub mark_voice_s: i64,
	pub mark_gap_s: i64,
	pub count_s: i64,
	pub count_max: i64,
	pub max_seconds: i64,
	pub max_reps: i64,
	pub allout_build_s: i64,
	pub cardio_cap_s: f64,
	/// The steady ledger's M range in thousandths (slice 52: the dose's multiplier, which took P's place).
	pub ledger_m: [f64; 2],
	/// The streak's milestone days (the Result's, `StreakBoard.milestone`).
	pub streak_milestones: Vec<i64>,
	/// The reminders (`desk::reminders`): the streak reminder's time by default, minutes after local midnight; its
	/// taper after an absence, in game days; the minutes this run lasts before it shows (2026-10-27); the sitting cue's
	/// hour, in minutes.
	pub remind_streak_min: u16,
	pub remind_taper_days: i64,
	pub remind_taper_every: i64,
	pub remind_weekly_days: i64,
	pub remind_run_min: u64,
	pub sit_after_min: i64,
	/// The game days past which an unfinished move puts no dot on the Pump tab and makes Catch up no primary
	/// (`desk::home`, the phones' `CatchUp.fresh`).
	pub owed_fresh_days: i64,
	/// An expedition (slice 51, `desk::pump::expedition`): the breather between two rounds (seconds), and the lengths
	/// the Menu's Quick pump length offers (minutes, 2026-11-03: the first until the player picks).
	pub expedition_rest_s: i64,
	pub expedition_lengths_min: Vec<u16>,
	/// The weekly vault's goals (Hero's vault card): moves done (pumps finished until 2026-10-27), records or stars,
	/// and since slice 55 the embers' tiers (the cardio minutes until then).
	pub vault_moves: i64,
	pub vault_records: i64,
	pub vault_embers: Vec<i64>,
	/// The most embers a game day lights (slice 55, Stats.lua `ns.LightsEmber`): the "Cardio done today" Toast's pips.
	pub embers_day: i64,
	/// Keep going shows in a steady move's last seconds (slice 52), and an all-out window's shouts come at these
	/// seconds left (slice 53, `Speech.lua` shout).
	pub keep_going_s: i64,
	pub shout_at_s: Vec<i64>,
	/// The voice's callouts (Stats.lua `ns.PickCallout`, which the engine runs): each fact's takes (a milestone's per
	/// milestone) and the pause after the move's line before one, in seconds.
	pub callout_takes: BTreeMap<String, usize>,
	pub callout_gap_s: f64,
	/// A loaded move's weight box (Progress.lua `ns.LoadStep`): the most it takes and a kettlebell's sizes, which its
	/// keys step through (whole kg), and any other weight's step (tenths of a kg).
	pub load_max_kg: u32,
	pub load_bells: Vec<u32>,
	pub load_box_step: u32,
	/// A loaded set's strength is its weight times this plus its reps (Progress.lua `ns.LoadScore`).
	pub load_epley: i64,
	/// The game days back Correct a count lists a set (slice 44, Progress.lua `ns.Correctable`): its window's words.
	pub correct_days: i64,
	/// The Saga's day (contract/fixtures/history.json `saga`, `desk::saga`): the earlier game days with a move a day
	/// needs before it gets a quality word or a best day, the calendar game days its rank looks back over, and the
	/// shares of those days (percent) that may beat it for it to be Epic, Rare and Uncommon.
	pub best_day_min_days: i64,
	pub day_rank_days: i64,
	pub day_rank_shares: Vec<i64>,
	/// Each focus's plan: its share of the day's moves (percent) by block (the Saga's kinds bar's ticks).
	pub focus_share: BTreeMap<String, BTreeMap<String, i64>>,
}

/// `contract/library.json`: the library the engine composes from, as the addon holds it.
#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Library {
	pub moves: BTreeMap<u16, Move>,
	pub ladders: Vec<Ladder>,
	pub constants: Constants,
	/// The floor move, which has no switch of its own.
	pub floor_move: u16,
	/// The cardio ladder.
	pub cardio: u8,
	/// Each region's ladder.
	pub region_ladder: BTreeMap<String, u8>,
	/// The strength and hold ladders, in the order the engine walks them (the feedback numbers' max sets count on
	/// them).
	#[serde(default)]
	pub strength_order: Vec<u8>,
	/// The joints' ids by their keys (`ns.Enum.joint`).
	pub joints: BTreeMap<String, u8>,
	/// The cardio forms' roads in their order (slice 54, Library.lua `ns.ROADS`).
	#[serde(default)]
	pub roads: Vec<String>,
	/// The moves merged into another, by id: a row of one reads as the move that took it (Library.lua `ns.Merged`).
	#[serde(default)]
	pub merged: BTreeMap<u16, u16>,
	/// The moves that switch sides since a time, by id, with that time (unix seconds; Library.lua `ns.Sided`): a set of
	/// one done before it held both sides on one clock.
	#[serde(default)]
	pub sided: BTreeMap<u16, i64>,
	/// The ladders that lost their first rung, by id, each with the moment it went (unix seconds; Library.lua
	/// `ns.Shifted`, the wall sit's 135° sit on 2026-10-26): a rung written before it reads one lower, the first as the
	/// first (`Facts::row_rung`, `Facts::shift_entry`, `Facts::shift_earned`).
	#[serde(default)]
	pub shifted: BTreeMap<u8, i64>,
}

/// The sheet summary's version from which a ladder that lost its first rung counts its rungs from the rung first now
/// (Stats.lua `SHIFTED_V`, 2026-10-26; the cores' `SheetSummary.shiftedRungs` and `SHIFTED_RUNGS`): an older one reads
/// through `Facts::shift_entry` and `Facts::shift_earned`.
pub const SHIFTED_V: i64 = 6;

// contract/moves.json ------------------------------------------------------------------------------------------------

/// A move's words.
#[derive(Deserialize, Debug, Default)]
pub struct MoveText {
	pub key: String,
	pub name: String,
	pub setup: String,
	/// The setup line for each implement that can carry the move's load (Library.lua `holds`), by its gear key.
	#[serde(default)]
	pub setups: BTreeMap<String, String>,
	/// How's steps, one a line (Locales' `move.<key>.how`, the rows of the game's Ready view, each a take of the
	/// voice's guide), and the steps for each implement that carries the move's load where it has its own
	/// (`move.<key>.how.<gear>`), by its gear key.
	#[serde(default)]
	pub how: Vec<String>,
	#[serde(default)]
	pub hows: BTreeMap<String, Vec<String>>,
	/// Tips' lines (`move.<key>.tips`).
	#[serde(default)]
	pub tips: Vec<String>,
	/// The stop line, the form break that ends a set: every move of reps or a hold has one, a timed move only when it
	/// has a form break (David, 2026-10-24: only what is legally necessary, so no line that only warns of a symptom).
	pub stop: Option<String>,
	/// A side move: one side, then Switch sides and the other, each side its own run.
	#[serde(default)]
	pub side: bool,
	/// The advice beside the pose that helps it (slice 42, 2026-10-17: Locales' `move.<key>.hint`, the last of its
	/// Tips, `Facts::tips`, and a step card's line after its How); which pose that is and whether the setup offers it
	/// are the engine's (the Book's move facts' `hint`).
	pub hint: Option<String>,
}

/// `contract/moves.json`: the moves' words and the fault pairs' captions by their keys.
#[derive(Deserialize, Debug, Default)]
pub struct Moves {
	pub moves: BTreeMap<u16, MoveText>,
	pub faults: BTreeMap<String, String>,
}

// The facts ----------------------------------------------------------------------------------------------------------

/// The three files as the desktop app reads them.
pub struct Facts {
	pub sheet: Sheet,
	pub library: Library,
	pub moves: Moves,
	/// The files that did not read: their parts above are empty.
	unread: Vec<&'static str>,
}

/// A built-in file, or none with its error logged.
fn read<T: for<'de> Deserialize<'de>>(file: &'static str, text: &str, unread: &mut Vec<&'static str>) -> Option<T> {
	match serde_json::from_str(text) {
		Ok(value) => Some(value),
		Err(e) => {
			log::error!("desk: the built-in contract/{file} does not read: {e}");
			unread.push(file);
			None
		}
	}
}

static FACTS: LazyLock<Facts> = LazyLock::new(|| {
	let mut unread = Vec::new();
	let sheet = read("sheet.json", include_str!("../../../../contract/sheet.json"), &mut unread);
	let library = read("library.json", include_str!("../../../../contract/library.json"), &mut unread);
	let moves = read("moves.json", include_str!("../../../../contract/moves.json"), &mut unread);
	Facts {
		sheet: sheet.unwrap_or_default(),
		library: library.unwrap_or_default(),
		moves: moves.unwrap_or_default(),
		unread,
	}
});

/// The built-in facts.
#[must_use]
pub fn facts() -> &'static Facts {
	&FACTS
}

impl Facts {
	/// Every file read; else which did not (each logged once at the first read).
	///
	/// # Errors
	/// The files that did not read.
	pub fn whole(&self) -> Result<(), String> {
		if self.unread.is_empty() { Ok(()) } else { Err(format!("contract/{} did not read", self.unread.join(", "))) }
	}

	/// The tunables.
	#[must_use]
	pub fn constants(&self) -> &Constants {
		&self.library.constants
	}

	/// A move's words; none for a move the book lacks.
	#[must_use]
	pub fn text(&self, id: u16) -> Option<&MoveText> {
		self.moves.moves.get(&id)
	}

	/// A move's setup line with the implement that carries its load (`setups`, else its own line), none without words.
	#[must_use]
	pub fn setup(&self, id: u16, implement: Option<&str>) -> Option<&str> {
		let text = self.text(id)?;
		Some(implement.and_then(|key| text.setups.get(key)).unwrap_or(&text.setup))
	}

	/// A move's How, its steps for the implement that carries its load (`hows`, else its own: Panel.lua `MoveText`);
	/// none without words.
	#[must_use]
	pub fn how(&self, id: u16, implement: Option<&str>) -> &[String] {
		self.text(id).map_or(&[], |text| implement.and_then(|key| text.hows.get(key)).unwrap_or(&text.how))
	}

	/// A move's Tips: its tips, and last, where the move names a pose that helps it, its own advice (`hint`; Panel.lua
	/// `MoveTips`); none without words.
	#[must_use]
	pub fn tips(&self, id: u16) -> Vec<&str> {
		self.text(id)
			.map(|text| text.tips.iter().map(String::as_str).chain(text.hint.as_deref()).collect())
			.unwrap_or_default()
	}

	/// A move's name; none for a move the book lacks (a newer library's).
	#[must_use]
	pub fn name(&self, id: u16) -> Option<&str> {
		self.text(id).map(|t| t.name.as_str())
	}

	/// The move a row of `id` reads as: the move that took a merged one (Library.lua `ns.Merged`, `ns.Translate`), else
	/// itself.
	#[must_use]
	pub fn taken(&self, id: u16) -> u16 {
		self.library.merged.get(&id).copied().unwrap_or(id)
	}

	/// The body map's muscles (ids 1 to 16: the heart, the breath and balance are none) the moves `ids` train most and
	/// some (each move's `mp` and `ms`), each sorted, a muscle one move trains most never among the some: the Saga's day
	/// strip and an expedition's Result.
	pub fn muscles(&self, ids: impl IntoIterator<Item = u16>) -> (Vec<u8>, Vec<u8>) {
		const MUSCLES: std::ops::RangeInclusive<u8> = 1..=16;
		let (mut most, mut some) = (Vec::new(), Vec::new());
		for t in ids.into_iter().filter_map(|id| self.tags(id)) {
			most.extend(t.mp.iter().copied().filter(|id| MUSCLES.contains(id)));
			some.extend(t.ms.iter().copied().filter(|id| MUSCLES.contains(id)));
		}
		most.sort_unstable();
		most.dedup();
		some.retain(|id| !most.contains(id));
		some.sort_unstable();
		some.dedup();
		(most, some)
	}

	/// A move as the library tags it, a merged move's as the move that took it (`taken`); none for a move the library
	/// lacks.
	#[must_use]
	pub fn tags(&self, id: u16) -> Option<&Move> {
		self.library.moves.get(&self.taken(id))
	}

	/// The move standing for a ladder's rung (library.json's `anchor`); none for a rung without one.
	#[must_use]
	pub fn anchor(&self, ladder: u8, rung: u8) -> Option<u16> {
		self.library
			.moves
			.iter()
			.find(|(_, m)| m.anchor && m.ladder == Some(ladder) && m.rung == rung)
			.map(|(&id, _)| id)
	}

	/// A pose's goal pose, the sentence of its own mark (Progress.lua `ns.MarkText`, sheet.json's `marks` by its key);
	/// none for a move without one.
	pub fn mark(&self, id: u16) -> Option<&str> {
		self.sheet.marks.get(&self.text(id)?.key).map(String::as_str)
	}

	/// The library's move ids, ascending.
	#[must_use]
	pub fn move_ids(&self) -> Vec<u16> {
		self.library.moves.keys().copied().collect()
	}

	/// Whether the library knows a move (a newer addon's is none of the fold's).
	#[must_use]
	pub fn knows(&self, id: u16) -> bool {
		self.library.moves.contains_key(&id)
	}

	/// A move a hold: its record and its counts are seconds.
	#[must_use]
	pub fn hold(&self, id: u16) -> bool {
		self.library.moves.get(&id).is_some_and(|m| m.dose == "hold")
	}

	/// A move dosed in reps (a counted cardio set is dosed in seconds, its count kept besides).
	#[must_use]
	pub fn rep(&self, id: u16) -> bool {
		self.library.moves.get(&id).is_some_and(|m| m.dose == "rep")
	}

	/// A rung row's rung as the fold reads it (DB.lua `ns.ShiftRow`): on a ladder that lost its first rung (`shifted`),
	/// one written before that moment one lower, the first as the first.
	#[must_use]
	pub fn row_rung(&self, ladder: u8, rung: u8, ts: i64) -> u8 {
		match self.library.shifted.get(&ladder) {
			Some(&at) if ts < at && rung >= 1 => rung.saturating_sub(1).max(1),
			_ => rung,
		}
	}

	/// Whether a summary of version `v` predates ladder `ladder`'s loss of its first rung (`shifted`, `SHIFTED_V`).
	fn summary_shifts(&self, ladder: u8, v: i64) -> bool {
		v < SHIFTED_V && self.library.shifted.contains_key(&ladder)
	}

	/// Ladder `ladder`'s entry of a summary of version `v` (`lad`: rung, move, record, stars) as one from `SHIFTED_V`
	/// on reads it (Stats.lua `ns.ShiftSummary`): on a ladder that lost its first rung since, the rung one lower, the
	/// first as the first, and one that stood on the rung that went with no move, record or stars (its move is retired:
	/// a reader takes the rung's own, as for a 0).
	pub fn shift_entry(&self, ladder: u8, v: i64, entry: &mut [i64]) {
		let Some(rung) = entry.first().copied().filter(|&r| r >= 1 && self.summary_shifts(ladder, v)) else { return };
		if rung == 1 {
			entry.iter_mut().skip(1).take(3).for_each(|n| *n = 0);
		}
		entry[0] = (rung - 1).max(1);
	}

	/// Ladder `ladder`'s earned steps of a summary of version `v` (`ear`: the best rung, then each rung's day) as one
	/// from `SHIFTED_V` on reads them (Stats.lua `ns.ShiftSummary`): on a ladder that lost its first rung since, from the
	/// rung first now, the first rung's day kept as the earlier.
	pub fn shift_earned(&self, ladder: u8, v: i64, earned: &mut Vec<i64>) {
		let Some(best) = earned.first().copied().filter(|&b| b >= 1 && self.summary_shifts(ladder, v)) else { return };
		if best >= 2 && earned.len() >= 3 {
			earned.remove(2);
		}
		earned[0] = (best - 1).max(1);
	}

	/// A hold's seconds as the fold reads them (DB.lua `sidedHalf`): a set of a move that switches sides (`sided`) done
	/// before its time held both sides on one clock, so each side's is half of it, rounded down.
	#[must_use]
	pub fn side_seconds(&self, id: u16, ts: i64, seconds: i64) -> i64 {
		match self.library.sided.get(&id) {
			Some(&at) if ts < at => seconds.div_euclid(2),
			_ => seconds,
		}
	}

	/// The joints a move loads (library.json's `pain`, by the joints' ids, `ns.Enum.joint`): a pain flag on any of them
	/// rests it.
	pub fn move_joints(&self, id: u16) -> Vec<u8> {
		self.library
			.moves
			.get(&id)
			.map_or_else(Vec::new, |m| m.pain.iter().filter_map(|key| self.library.joints.get(key).copied()).collect())
	}

	/// A move's group, whether it is offered only once checked, and every move of that group, ascending: what Include
	/// in pumps reads (`body::Moves::check`); none for a move the library lacks.
	#[must_use]
	pub fn group(&self, id: u16) -> Option<(&str, bool, Vec<u16>)> {
		let m = self.library.moves.get(&id)?;
		let members = self.library.moves.iter().filter(|(_, other)| other.group == m.group).map(|(&k, _)| k).collect();
		Some((m.group.as_str(), m.opt_in, members))
	}

	/// A rung's step on the one scale (ns.Step); none for a rung the ladder lacks.
	#[must_use]
	pub fn step(&self, ladder: u8, rung: u8) -> Option<u8> {
		self.rung(ladder, rung).map(|r| r.step)
	}

	/// Rung `rung` (from 1) of ladder `ladder`; none for a rung the library lacks.
	fn rung(&self, ladder: u8, rung: u8) -> Option<&Rung> {
		let ladder = self.library.ladders.iter().find(|l| l.id == ladder)?;
		ladder.rungs.get(usize::from(rung).checked_sub(1)?)
	}

	/// The top of `window` (Progress.lua `ns.Window`'s hi).
	#[must_use]
	pub fn window_top(&self, ladder: u8, rung: u8, id: u16) -> Option<i64> {
		self.window(ladder, rung, id).map(|w| w.hi)
	}

	/// The window a set of move `id` counts at rung `rung` of ladder `ladder` (Progress.lua `ns.Window`): the move's own
	/// in the rung's `alt` where it has one (a merged move's as the move that took it), else the rung's; none on a rung
	/// without a window (cardio's, a pose family's) or one the library lacks.
	#[must_use]
	pub fn window(&self, ladder: u8, rung: u8, id: u16) -> Option<Window> {
		let r = self.rung(ladder, rung).filter(|r| r.lo.is_some())?;
		r.alt.get(&self.taken(id)).copied().or_else(|| Some(Window { lo: r.lo?, hi: r.hi? }))
	}

	/// The rungs ladder `ladder` has (0 for one the library lacks).
	#[must_use]
	pub fn rungs(&self, ladder: u8) -> u8 {
		self.library
			.ladders
			.iter()
			.find(|l| l.id == ladder)
			.map_or(0, |l| u8::try_from(l.rungs.len()).unwrap_or(u8::MAX))
	}

	/// A step's colour as the screens count it (1 Starter … 7 Artifact: `quality_by_step` plus 1); none past the scale.
	#[must_use]
	pub fn step_quality(&self, step: u8) -> Option<u8> {
		self.constants().quality_by_step.get(usize::from(step).checked_sub(1)?).map(|q| q + 1)
	}

	/// A rung's colour as the screens count it (1 Starter … 7 Artifact, a rung past either end read as that end's); 0
	/// for a ladder without rungs (`EngineLibrary.rungQuality`).
	#[must_use]
	pub fn rung_quality(&self, ladder: u8, rung: i64) -> u8 {
		let Some(rungs) =
			self.library.ladders.iter().find(|l| l.id == ladder).map(|l| &l.rungs).filter(|r| !r.is_empty())
		else {
			return 0;
		};
		let at = usize::try_from(rung.clamp(1, i64::try_from(rungs.len()).unwrap_or(1)) - 1).unwrap_or(0);
		rungs.get(at).and_then(|r| self.step_quality(r.step)).unwrap_or(0)
	}

	/// A move's quality (`EngineLibrary.quality(of:)`): its rung's colour on its ladder; none where the library places
	/// the move on no ladder.
	#[must_use]
	pub fn move_quality(&self, id: u16) -> Option<u8> {
		let m = self.library.moves.get(&id)?;
		Some(self.rung_quality(m.ladder?, i64::from(m.rung))).filter(|&q| q > 0)
	}

	/// A quality's word (1 Starter … 7 Artifact); none for 0 and past the list.
	pub fn quality_word(&self, quality: u8) -> Option<&str> {
		self.sheet.qualities.get(usize::from(quality).checked_sub(1)?).map(String::as_str)
	}

	/// A ladder in the sheet's words; none for an id it lacks.
	#[must_use]
	pub fn ladder_words(&self, id: u8) -> Option<&SheetLadder> {
		self.sheet.ladders.iter().find(|l| l.id == id)
	}

	/// A ladder's name; empty for one the sheet names none.
	#[must_use]
	pub fn ladder_name(&self, id: u8) -> &str {
		self.ladder_words(id).map_or("", |l| l.name.as_str())
	}

	/// A muscle's name; none where the sheet names none.
	#[must_use]
	pub fn muscle_name(&self, id: u8) -> Option<&str> {
		self.sheet.muscles.iter().find(|m| m.id == id).map(|m| m.name.as_str())
	}

	/// A title by its code; none for a code these words lack (a newer addon's).
	#[must_use]
	pub fn title(&self, code: u8) -> Option<&Title> {
		self.sheet.titles.iter().find(|t| t.id == code)
	}

	/// A focus by its code; none for a code these words lack.
	#[must_use]
	pub fn focus(&self, id: u8) -> Option<&Focus> {
		self.sheet.focus.iter().find(|f| f.id == id)
	}

	/// A joint by its id; none for an id these words lack.
	#[must_use]
	pub fn joint(&self, id: u8) -> Option<&ById> {
		self.sheet.joints.iter().find(|j| j.id == id)
	}
}
