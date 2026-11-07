//! The Book on the computer (docs/ui.md, "Hero, the Book and the Log"; docs/plan.md slice 35; the phones' Book, slices
//! 28 to 30 and 34: `BookTab`, `LadderDetail`, `MoveJournal` and their cores), as two topics. `book.words` holds what
//! never changes in a build: the moves' names, setups, How, Tips and stop lines (contract/moves.json), the library's
//! facts the pages read (contract/library.json: each move's ladder, rung, dose, region, block, group, muscles,
//! equipment and weight; the ladders' steps and their rungs' qualities) and the sheet's words (contract/sheet.json: the
//! qualities, the regions, blocks, muscles and stats, each pose's goal pose and the felt-unwell word the Log shows),
//! each move's search terms, and the families the Moves grid lists in the ladders' order by stat (Core.lua
//! `ns.LADDER_ORDER`), all from the built-in facts (`facts.rs`; a file that does not read leaves the Book and the
//! Log empty). `book` holds what the one progression says now, from the addon's own engine (`questpump-engine`'s
//! `book`, the game's own rules beside the game's sheet the app shows): every ladder's row (a ladder that waits whole
//! too, since the talent trees draw them all: slice 38), its fourteen steps with what each step's card starts (`start`,
//! `ns.StartFor`) and its Path (the road to its top, every step free since 2026-10-14, its pace Pro's), its entry in
//! its tree (the ring, the day, `waits`, `placing`, `appRung`), the step-up closest to done of all the trees (Pump
//! home's row: `closest`, docs/ui.md "The Book's ladders and paths"), every move's record (a loaded move's with its
//! weight), familiarity and switches, the move that takes its place while it is set aside and what its window's Start
//! starts (`Engine::starts`), this computer's favourites, the share card's style, the shown
//! sheet's character, and, with Pro (`Desk::pro_on`), the open ladder page's charts from the game's sheets and rows (`history`). It is built again whenever the engine's state
//! moves (a fold landed, rows merged, the profile changed), Pro turns on or off, the page opens another ladder, or the
//! moment its answer changes by the clock alone passes (a climb's day, a fresh set, the felt-unwell lock's end, the
//! game day's turn: `BookLog::due`, at the loop's look); while the engine folds it says `ready: false`.
//!
//! Intents: `book.ladder {ladder}` (the ladder page shown, or none: its charts), `book.favourite {move, on}` (this
//! computer's favourites, which the engine's profile takes), `book.include {move, on}` (the body every app device
//! shares: its moves' exclusions and opt-in moves, changed now on this computer), `book.clear {move}` and
//! `book.restart {ladder}` (a move's record cleared, a strength ladder started over: a clear row, refused while
//! a pump is on screen or the engine folds), `book.move {move, shown}` (a move's page stands or goes: the Book carries
//! the sets Correct a count's window lists for each move whose page stands, `sets`), `book.correct {seq, move, ts,
//! quest, slot, value?, load?, answer?, confirm?}` (Correct a count's Save, slice 44: the set named by its row's time,
//! quest, slot and move corrected to `value` at `load` or to a pose's `answer`, checked as the count step checks a
//! count, "Is that right?" once and its Yes `confirm`; refused while a pump is on screen or the engine folds; its
//! outcome is the `book.correct` topic's, by the page's `seq`), `book.card {refused?}` (a step's card opened: the counts' `tree_day` takes the card and,
//! where its Start was refused or greyed, the reason: setup, off, game or reading),
//! `share.style {style}` (the share card's style), `share.card {card}` (the day's card, saga or bill) and
//! `share.format {format}` (story, post, square or sticker; the Saga, 2026-10-15), each kept on this computer. The
//! scene cards' standing is `desk/standing.rs`'s (`standing.look`). The share card itself is the page's (`lib/share.js`): its PNG reaches the shell's
//! `save_image`. A Start is the pump's (`pump.move`).

use std::collections::BTreeMap;
use std::sync::LazyLock;

use questpump_core::body::Body;
use questpump_core::codes::{answer, outcome};
use questpump_core::link::Sheet;
use questpump_engine::{
	Book, BookLadder, BookMove, BookTree, Check, Clear, CorrectCheck, CorrectKind, CorrectSet, Corrected,
	CorrectionCheck, EngineError, GameLadders, NeedEntry, NeedState, OwnSide, SetKey, SetReading, SetupStep, Start,
	Stays, StepChanges, StepState, Target,
};
use serde::Serialize;

use super::facts::{ById, Facts, Move, Named, Need, Rung, SheetTree, Window, facts};
use super::hero::GameSheet;
use super::history::{self, GameDay, LadderPoint};
use super::log::LogState;
use super::menu_pages::setup::stamped;
use super::pump::{Against, CheckView, CountStep, Outcome, Typed, check_view, count_most, max_line};
use super::seams::StoreMark;
use crate::settings::{ShareCard, ShareFormat, ShareStyle};
use crate::{Desk, Intent, Platform};

/// The Book's topics, and Correct a count's outcome (slice 44).
const WORDS: &str = "book.words";
const BOOK: &str = "book";
const CORRECT: &str = "book.correct";
/// The clear rows the Book writes, as the log names them (`Desk::keep_own`).
const CLEAR_ROW: &str = "record cleared"; // copy-lint: not shown (a log line's words)
const RESTART_ROW: &str = "ladder started over"; // copy-lint: not shown (a log line's words)
/// The rows a correction writes, as the log names them.
const CORRECT_ROW: &str = "count corrected"; // copy-lint: not shown (a log line's words)

// The library's, the moves' and the sheet's words --------------------------------------------------------------------

/// A ladder as the Book's words name it: its family's name, its dose, its rungs with each one's step, quality (1
/// Starter … 7 Artifact) and windows (its anchor's, and each of its other moves' own: the tree's key line, "Record 22 ·
/// 20-30"), and the links into it (slice 36's design, the arrows a selected step draws): the steps of other ladders
/// that open it or that a climb of it waits for.
#[derive(Serialize)]
struct LadderWords {
	id: u8,
	name: String,
	dose: String,
	steps: Vec<u8>,
	qualities: Vec<u8>,
	windows: Vec<RungWindow>,
	links: Vec<Link>,
}

/// A rung's windows, its anchor's (none on cardio and on a pose family) with that anchor, whose unit it counts in (a
/// move of another unit has no window there but its own: Progress.lua `ns.OwnUnit`), and its other moves' own, by move,
/// and the moves of its climb's path, every stage's in order (slice 42; a step's card lists them among the step's
/// other moves).
#[derive(Serialize)]
struct RungWindow {
	#[serde(skip_serializing_if = "Option::is_none")]
	lo: Option<i64>,
	#[serde(skip_serializing_if = "Option::is_none")]
	hi: Option<i64>,
	#[serde(skip_serializing_if = "Option::is_none")]
	anchor: Option<u16>,
	#[serde(skip_serializing_if = "BTreeMap::is_empty")]
	alt: BTreeMap<u16, Window>,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	path: Vec<u16>,
}

/// A link into a ladder's step `into` (on the one scale) from another ladder's step: an entry of the family's opening
/// (`opens`, into its first step) or a step its climb waits for (Progress.lua's rung `needs`, into the step that rung
/// stands on), with the record of `count` on `move` (else that step's anchor) or a pose's least answer (`mark`) where
/// it names one; an `any` group (slice 42) whole, done when one of its steps is.
#[derive(Serialize)]
struct Link {
	into: u8,
	#[serde(flatten)]
	from: NeedEntry,
	#[serde(skip_serializing_if = "std::ops::Not::not")]
	opens: bool,
}

/// A move as the Book's words name it.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MoveWords {
	#[serde(rename = "move")]
	id: u16,
	key: String,
	name: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	ladder: Option<u8>,
	rung: u8,
	dose: String,
	/// A pose's seconds (each side's on a side move).
	#[serde(skip_serializing_if = "Option::is_none")]
	secs: Option<i64>,
	/// Done on each side, one after the other (moves.json `side`): a stage's opening top reads "25 a side" (slice 42's
	/// fix round, 2026-10-17).
	#[serde(skip_serializing_if = "std::ops::Not::not")]
	side: bool,
	#[serde(skip_serializing_if = "Option::is_none")]
	region: Option<String>,
	block: String,
	/// A countable cardio form: its all-out window keeps a count, as a rep or hold move does, never a goal (slice 42).
	#[serde(skip_serializing_if = "std::ops::Not::not")]
	count: bool,
	/// A loaded move: `total` or `each` (a weight in each hand), its sets kept with their weight.
	#[serde(skip_serializing_if = "Option::is_none")]
	load: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	setup: Option<String>,
	/// How's steps (`Facts::how`) and Tips with the move's own advice last (`Facts::tips`): its page's, its window's and
	/// a step card's How and Tips; the setup line and the steps for each implement that carries its load where it has
	/// its own (`setups`, `hows`, by gear key), which the page picks by the player's implement (`BookMove::implement`).
	#[serde(skip_serializing_if = "BTreeMap::is_empty")]
	setups: BTreeMap<String, String>,
	how: Vec<String>,
	#[serde(skip_serializing_if = "BTreeMap::is_empty")]
	hows: BTreeMap<String, Vec<String>>,
	tips: Vec<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	stop: Option<String>,
	/// A pose's goal pose, its own mark's sentence (`Facts::mark`).
	#[serde(skip_serializing_if = "Option::is_none")]
	mark: Option<String>,
	/// The steps of other ladders it waits for (library.json's `checks`, slice 42: a stretch check's row and "Do the
	/// check" on its card and window, a record's need row), whose states the Book's moves carry in their order.
	#[serde(skip_serializing_if = "Vec::is_empty")]
	checks: Vec<NeedEntry>,
	/// Its own advice (moves.json's `hint`, slice 42), which a step's card shows after its How, as the game's card does
	/// (Tree.lua `CardMore`; a line under its Helps row until 2026-10-22); the pose that helps it is the Book's
	/// (`BookMove::hint`).
	#[serde(skip_serializing_if = "Option::is_none")]
	hint: Option<String>,
	mp: Vec<u8>,
	ms: Vec<u8>,
	/// The words the Moves search looks in: its name, its block's and group's, its muscles' and its equipment's.
	terms: Vec<String>,
}

/// A family's moves as the Moves grid lists them: a ladder's (0: the moves off the ladders, "Not on a ladder") in its
/// climb's order.
#[derive(Serialize)]
struct Family {
	ladder: u8,
	moves: Vec<u16>,
}

/// `book.words`: what the Book's pages name, the same through a build.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Words {
	qualities: Vec<String>,
	ladders: Vec<LadderWords>,
	/// The three talent trees (slice 36), their branches and ladders in column order.
	trees: Vec<SheetTree>,
	moves: Vec<MoveWords>,
	families: Vec<Family>,
	regions: Vec<Named>,
	blocks: Vec<Named>,
	muscles: Vec<ById>,
	stats: Vec<Named>,
	gear: BTreeMap<String, String>,
	floor_move: u16,
	/// The game days that make a move learned (`fam_learned_days`), the days a pose family's step takes, counted by any
	/// goal pose of the step (`pose_unlock_days`, the hero card's "of"; slice 47: what of them is still short on the
	/// step's main pose and answered Perfect comes with the engine's climb), a timed cardio rung's full moves
	/// (`cardio_full_days`, slice 52, "2 full moves" under a timed step's Pace; its ready days, slice 42, until then), a
	/// rung's stars (`star_fractions`) and the weeks' marks' steps (`map_steps`), and a counted set's seconds all out
	/// from its go (`count_s`: a counted cardio step's card's measure, "3 clean days of 30 s all out").
	learned_days: i64,
	pose_days: i64,
	full_days: i64,
	count_seconds: i64,
	stars: usize,
	map_steps: Vec<i64>,
	/// The hour the game days turn at (`day_offset_h`, 04:00), by which the Book dates a moment as the game's `ns.DayMid`
	/// does: a check passed and a step mastered at 01:00 show the day before (slice 42's last review, 2026-10-18: the
	/// same date on every surface).
	day_offset_h: i64,
	/// The felt-unwell word of a pump stopped for feeling unwell, in its Log row and on its page (the game's, as the
	/// phones' Log says it).
	unwell: String,
	/// What Correct a count's window counts with (slice 44).
	correct: CorrectWords,
}

/// What Correct a count's window counts with (slice 44, docs/plan.md "One correction, from any device"), the count
/// step's limits and its words in the game's (contract/sheet.json `words.count`): one set's most for a rep set, a hold
/// and a counted set, each with its "Max 100: count one set"; Could not do one (a hold's Could not hold it), the 0 a
/// rep or hold set saves, and Done without a count, how a rep or hold set at 0 the fold reads done says it (saved with
/// an empty box, or corrected to 0: slice 44's review, 2026-10-18); the weight box's most and step (tenths of a kg,
/// `load_max_kg`, `load_box_step`); and the game days back a set is listed (`correct_days`, the window's "Your sets of
/// the last 14 days").
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CorrectWords {
	reps: Most,
	seconds: Most,
	counted: Most,
	could_not: String,
	could_not_hold: String,
	empty: String,
	kg_most: u32,
	kg_step: u32,
	days: i64,
}

/// One set's most and its line.
#[derive(Serialize)]
struct Most {
	most: i64,
	line: String,
}

impl Most {
	fn of(counted: bool, hold: bool) -> Self {
		let most = count_most(counted, hold);
		Self { most, line: max_line(most) }
	}
}

impl CorrectWords {
	fn of(f: &Facts) -> Self {
		let (count, c) = (&f.sheet.words.count, f.constants());
		Self {
			reps: Most::of(false, false),
			seconds: Most::of(false, true),
			counted: Most::of(true, false),
			could_not: count.could_not.clone(),
			could_not_hold: count.could_not_hold.clone(),
			empty: count.empty.clone(),
			kg_most: c.load_max_kg * 10,
			kg_step: c.load_box_step,
			days: c.correct_days,
		}
	}
}

/// The Book's words from the built-in facts (`facts.rs`): none while a contract file does not read (logged once
/// there), and then the Book and the Log stay empty.
fn words() -> Result<Words, String> {
	let f = facts();
	f.whole()?;
	let (sheet, c) = (&f.sheet, f.constants());
	let ladders = sheet
		.order
		.iter()
		.filter_map(|&id| f.ladder_words(id))
		.filter(|l| l.kind != "retired" && l.dose != "none")
		.map(|l| {
			let rungs = f.library.ladders.iter().find(|r| r.id == l.id).map_or(&[][..], |r| r.rungs.as_slice());
			LadderWords {
				id: l.id,
				name: l.name.clone(),
				dose: l.dose.clone(),
				steps: l.steps.clone(),
				qualities: rungs.iter().map(|rung| f.step_quality(rung.step).unwrap_or(0)).collect(),
				windows: windows(l.id, rungs),
				links: links(l.id),
			}
		})
		.collect();
	let block = |key: &str| sheet.blocks.iter().find(|b| b.key == key).map(|b| b.name.clone());
	// by id, a cardio step's forms in road order (slice 54)
	let mut listed: Vec<(&u16, &Move)> = f.library.moves.iter().collect();
	listed.sort_by_key(|&(_, m)| road_rank(f, m));
	let moves = listed
		.into_iter()
		.map(|(&id, m)| {
			let text = f.text(id);
			let mut terms: Vec<String> = text
				.map(|t| t.name.clone())
				.into_iter()
				.chain(block(&m.block))
				.chain(sheet.groups.get(&m.group).cloned())
				.chain(m.mp.iter().chain(&m.ms).filter_map(|&muscle| f.muscle_name(muscle).map(str::to_owned)))
				.chain(m.needs.iter().flat_map(Need::keys).filter_map(|k| sheet.gear.get(k).cloned()))
				.collect();
			terms.sort();
			terms.dedup();
			MoveWords {
				id,
				key: text.map(|t| t.key.clone()).unwrap_or_default(),
				name: text.map(|t| t.name.clone()).unwrap_or_default(),
				ladder: m.ladder,
				rung: m.rung,
				dose: m.dose.clone(),
				secs: m.secs,
				side: text.is_some_and(|t| t.side),
				region: m.region.clone(),
				block: m.block.clone(),
				count: m.count,
				load: m.load.clone(),
				setup: text.map(|t| t.setup.clone()),
				setups: text.map(|t| t.setups.clone()).unwrap_or_default(),
				how: f.how(id, None).to_vec(),
				hows: text.map(|t| t.hows.clone()).unwrap_or_default(),
				tips: f.tips(id).into_iter().map(str::to_owned).collect(),
				stop: text.and_then(|t| t.stop.clone()),
				mark: f.mark(id).map(str::to_owned),
				checks: m.checks.clone(),
				hint: text.and_then(|t| t.hint.clone()),
				mp: m.mp.clone(),
				ms: m.ms.clone(),
				terms,
			}
		})
		.collect();
	Ok(Words {
		qualities: sheet.qualities.clone(),
		ladders,
		trees: sheet.trees.clone(),
		moves,
		families: families(f),
		regions: sheet.regions.clone(),
		blocks: sheet.blocks.clone(),
		muscles: sheet.muscles.clone(),
		stats: sheet.stats.clone(),
		gear: sheet.gear.clone(),
		floor_move: f.library.floor_move,
		learned_days: c.fam_learned_days,
		pose_days: c.pose_unlock_days,
		full_days: c.cardio_full_days,
		count_seconds: c.count_s,
		stars: c.star_fractions.len(),
		map_steps: c.map_steps.clone(),
		day_offset_h: c.day_offset_h,
		unwell: sheet.words.unwell.title.clone(),
		correct: CorrectWords::of(f),
	})
}

/// A move's place by its road (slice 54, `Library::roads`: run, strike, jump, floor, standing, gear, from 1; 0 for a move
/// with none), by which a cardio step's forms read in road order, then by id, in the Book's lists and its search.
fn road_rank(f: &Facts, m: &Move) -> usize {
	m.road.as_ref().and_then(|r| f.library.roads.iter().position(|x| x == r)).map_or(0, |i| i + 1)
}

/// Each family's moves in its climb's order (the rung, the road, `road_rank`, then the id), in the ladders' order by
/// stat; the moves off the ladders last.
fn families(f: &Facts) -> Vec<Family> {
	let mut by_ladder: BTreeMap<u8, Vec<(u8, usize, u16)>> = BTreeMap::new();
	for (&id, m) in &f.library.moves {
		by_ladder.entry(m.ladder.unwrap_or(0)).or_default().push((m.rung, road_rank(f, m), id));
	}
	f.sheet
		.order
		.iter()
		.copied()
		.chain([0])
		.filter_map(|l| {
			let mut list = by_ladder.get(&l)?.clone();
			list.sort_unstable();
			Some(Family { ladder: l, moves: list.into_iter().map(|(_, _, id)| id).collect() })
		})
		.collect()
}

/// Ladder `id`'s rungs' windows, each with the anchor its own window counts for.
fn windows(id: u8, rungs: &[Rung]) -> Vec<RungWindow> {
	let anchor = |rung: u8| {
		facts()
			.library
			.moves
			.iter()
			.find(|(_, m)| m.anchor && m.ladder == Some(id) && m.rung == rung)
			.map(|(&id, _)| id)
	};
	rungs
		.iter()
		.zip(1..)
		.map(|(r, rung)| RungWindow {
			lo: r.lo,
			hi: r.hi,
			anchor: r.lo.and_then(|_| anchor(rung)),
			alt: r.alt.clone(),
			path: r.path.iter().flat_map(|stage| stage.ids.iter().copied()).collect(),
		})
		.collect()
}

/// The links into ladder `id`: each entry of its family's opening into its first step, then each rung's needs into the
/// step that rung stands on, in the rungs' order.
fn links(id: u8) -> Vec<Link> {
	let Some(ladder) = facts().library.ladders.iter().find(|l| l.id == id) else { return Vec::new() };
	let first = ladder.rungs.first().map(|r| r.step);
	let opening = first
		.into_iter()
		.flat_map(|into| ladder.opens.iter().map(move |o| Link { into, from: o.clone(), opens: true }));
	opening
		.chain(
			ladder
				.rungs
				.iter()
				.flat_map(|r| r.needs.iter().map(|from| Link { into: r.step, from: from.clone(), opens: false })),
		)
		.collect()
}

/// The Book's words, built once.
static WORDS_BUILT: LazyLock<Result<Words, String>> = LazyLock::new(words);

// The Book's state -------------------------------------------------------------------------------------------------

/// What the helper's stores held when the Book and the Log were last built (`Desk::store_mark`), and the sheets the
/// helper logged.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Seen {
	stores: StoreMark,
	sheets: usize,
}

/// The game's sheets the helper logged, read once each: the charts' points, the Log's XP and the weekly reset.
#[derive(Default)]
pub struct Sheets {
	read: usize,
	floor: Option<u32>,
	pub list: Vec<history::Sheet>,
}

/// The engine's answer to a Book build: the Book, what each move's window starts (`Engine::starts`, by move), what
/// one more piece of the setup opens on each ladder (`Engine::ladder_opens`, by ladder) and each step another ladder
/// waits for as the engine reads it (`Engine::needs`: done, the move and record that do it, a stretch check's ask and
/// when it was passed, with a stretch check's Start; by ladder in its words' `links` order, a ladder with links alone,
/// and since slice 42 each move's `checks`, by move in their order). The last three are empty where their call failed:
/// the Book shows without them (a window without its Start, a card without its gear line and its need and check rows'
/// checks and waits lines), never the Book of an older state. Beside them, for each move whose page stands
/// (`BookLog::pages`), the sets Correct a count's window lists (`Engine::correctable`; empty where the call failed).
struct Answer {
	book: Book,
	starts: BTreeMap<u16, Start>,
	opens: BTreeMap<u8, Vec<SetupStep>>,
	needs: BTreeMap<u8, Vec<NeedRow>>,
	checks: BTreeMap<u16, Vec<NeedRow>>,
	sets: BTreeMap<u16, Vec<CorrectSet>>,
}

/// A step of another ladder as the engine reads it (`NeedState`) and, on a stretch check not done, what its row's "Do
/// the check" starts (`Engine::start` of the check the row shows, an `any` group's choice: `ns.StartFor`, as the phones
/// ask it; 2026-10-19, the program's double-check), whose refusal (the setup, a pain flag) the row says in place of the
/// button; none where the call failed (the button then as before).
#[derive(Serialize, Debug, Clone, PartialEq)]
struct NeedRow {
	#[serde(flatten)]
	state: NeedState,
	#[serde(skip_serializing_if = "Option::is_none")]
	start: Option<Start>,
}

/// The step of a stretch check `entry` the row shows (the choice the setup offers in an `any` group, the engine's
/// `choice` from 1), as `Target::Check` names it; none for a need that is no check, or one done.
fn check_target(entry: &NeedEntry, state: &NeedState) -> Option<Target> {
	if state.check.is_none() || state.done {
		return None;
	}
	let at = state.choice.and_then(|c| c.checked_sub(1)).unwrap_or(0);
	entry.steps().get(at).map(|&check| Target::Check { check })
}

/// An extra call's answer for the Book, or, where it failed, none (its error kept in `failed`, named by `what`).
fn extra<T: Default>(failed: &mut Vec<String>, what: &str, answer: Result<T, EngineError>) -> T {
	answer.unwrap_or_else(|e| {
		failed.push(format!("{what}: {e}"));
		T::default()
	})
}

/// The steps the words' links come from, every ladder's in the words' order, then every move's `checks` in the words'
/// order: what `Engine::needs` is asked.
fn needs_of(words: &Words) -> Vec<NeedEntry> {
	let links = words.ladders.iter().flat_map(|l| l.links.iter().map(|link| link.from.clone()));
	links.chain(words.moves.iter().flat_map(|m| m.checks.iter().cloned())).collect()
}

/// The engine's answers for the steps `needs_of` lists, back by ladder in each one's links' order (a ladder with no
/// link has none), then by move in each one's checks' order (a move with no check has none).
fn needs_by_ladder<T>(
	words: &Words,
	mut needs: impl Iterator<Item = T>,
) -> (BTreeMap<u8, Vec<T>>, BTreeMap<u16, Vec<T>>) {
	let ladders = words
		.ladders
		.iter()
		.filter(|l| !l.links.is_empty())
		.map(|l| (l.id, needs.by_ref().take(l.links.len()).collect()))
		.collect();
	let moves = words
		.moves
		.iter()
		.filter(|m| !m.checks.is_empty())
		.map(|m| (m.id, needs.by_ref().take(m.checks.len()).collect()))
		.collect();
	(ladders, moves)
}

/// The first moment after `at` at which the Book's answer changes by the clock alone: a tree's next counted day
/// (`at`), a ladder's or a Start's day coming in or a set fresh again (`dayIn`, `freshFrom`), the felt-unwell lock's
/// end (`locked`) and the next game day's turn (`turn`, which ends a day's tiredness and its repeat goals).
fn due(answer: &Answer, at: i64, locked: Option<i64>, turn: i64) -> i64 {
	let book = &answer.book;
	let checks = answer.needs.values().chain(answer.checks.values()).flatten().filter_map(|n| n.start.as_ref());
	let starts = book
		.ladders
		.iter()
		.flat_map(|l| l.steps.iter().filter_map(|s| s.start.as_ref()))
		.chain(answer.starts.values())
		.chain(checks)
		.flat_map(|s| [s.day_in, s.fresh_from]);
	book.trees
		.iter()
		.map(|t| t.at)
		.chain(book.ladders.iter().flat_map(|l| [l.day_in, l.fresh_from]))
		.chain(starts)
		.chain([locked])
		.flatten()
		.filter(|&t| t > at)
		.fold(turn, i64::min)
}

/// The Book's and the Log's state in the loop.
#[derive(Default)]
pub struct BookLog {
	/// The engine's last answer.
	answer: Option<Answer>,
	/// A `book` call is on its way; `again` when the state moved meanwhile, so one more follows its answer.
	asking: bool,
	again: bool,
	/// The ladder whose page the page shows (its charts with Pro).
	ladder: Option<u8>,
	/// The moment the last answer changes by the clock alone (unix seconds, [`due`]): the loop's look builds the Book
	/// again once it has passed.
	due: Option<i64>,
	/// Whether the last build's extra calls (`starts`, `ladder_opens`, `needs`, `correctable`) failed: logged once
	/// until they answer again.
	extras_failed: bool,
	/// The moves whose pages stand (`book.move`, once a page: a move's page pushed over another's of the same move
	/// counts twice), whose sets the Book carries; none once the main window is gone (`Desk::book_pages_gone`).
	pages: Vec<u16>,
	/// Correct a count as the loop runs it.
	correct: Correcting,
	pub seen: Seen,
	pub sheets: Sheets,
	pub log: LogState,
}

/// The open ladder page's charts (with Pro): the ladder's points over the game's sheets (each change of its rung, its
/// move, its record or its stars, by game day), the typed counts of the move standing for its rung, the points that
/// raised that move's record, and now (the lines reach it).
#[derive(Serialize, Debug, Clone, PartialEq)]
struct LadderChart {
	ladder: u8,
	points: Vec<LadderPoint>,
	/// [time, count] of each typed count of the standing move (reps, or a hold's seconds).
	counts: Vec<[i64; 2]>,
	/// The times of the points that raised the standing move's record (the phones' `rises(ladder:move:)`).
	rises: Vec<i64>,
	now: i64,
}

/// A ladder's row (`BookLadder`) with what the words add: its rung's step on the one scale, its record in seconds (a
/// hold's), a pose family's goal pose at its rung (`rung_mark`) and its pose's hold, what one more piece of the setup
/// opens on it (`gear`, a step's card's line, as the game's ladder tooltip), and each step of another ladder its
/// words' `links` come from as the engine reads it (`needs`, in their order: done, `ns.NeedDone`, the check a card
/// shows beside each, and the move and record that do it, `ns.NeedAt`, its waits line; none where the engine did not
/// answer, so the page draws no check it does not know).
#[derive(Serialize, Debug, Clone, PartialEq)]
struct LadderRow {
	#[serde(flatten)]
	ladder: BookLadder,
	step: u8,
	seconds: bool,
	#[serde(skip_serializing_if = "Option::is_none")]
	mark: Option<&'static str>,
	#[serde(skip_serializing_if = "Option::is_none")]
	hold: Option<i64>,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	gear: Vec<SetupStep>,
	#[serde(skip_serializing_if = "Option::is_none")]
	needs: Option<Vec<NeedRow>>,
}

/// A move's facts (`BookMove`) with what its window's Start starts (slice 36) and, since slice 42, each step of another
/// ladder its library `checks` name as the engine reads it (`Engine::needs`, in their order; none where the move has
/// none or the engine did not answer, so the page draws no check it does not know).
#[derive(Serialize, Debug, Clone, PartialEq)]
struct MoveRow {
	#[serde(flatten)]
	fact: BookMove,
	#[serde(skip_serializing_if = "Option::is_none")]
	start: Option<Start>,
	#[serde(skip_serializing_if = "Option::is_none")]
	checks: Option<Vec<NeedRow>>,
	/// Correct a count's sets (slice 44), on a move whose page stands: present, if empty, once the engine was asked.
	#[serde(skip_serializing_if = "Option::is_none")]
	sets: Option<Vec<SetRow>>,
}

/// A set Correct a count's window lists, as the page shows it: the set, what a correction may change, how it reads
/// now, whether it holds the record and, on a set of this computer's whose winning correction came from another side
/// while this side still reads it otherwise (`ns.OwnSide`, 2026-10-19), how this side reads it (`ownSide`): its window
/// then offers Save at the value it reads now, a new round of it that asks the step rules again here.
#[derive(Serialize, Debug, Clone, Copy, PartialEq)]
struct SetRow {
	#[serde(flatten)]
	set: SetKey,
	kind: CorrectKind,
	#[serde(flatten)]
	reads: Reads,
	#[serde(skip_serializing_if = "std::ops::Not::not")]
	record: bool,
	#[serde(rename = "ownSide", skip_serializing_if = "Option::is_none")]
	own_side: Option<OwnSide>,
}

impl SetRow {
	fn of(c: &CorrectSet) -> Self {
		Self { set: c.set, kind: c.kind, reads: Reads::of(c.reads), record: c.record, own_side: c.own_side }
	}
}

/// A set as Correct a count's window words it (slice 44's review, 2026-10-18): its value and weight as the fold reads
/// them, and whether the fold reads it done (`outcome::DONE`, Progress.lua `ns.SetOutcome`), so a rep or hold set at 0
/// says Done without a count where it was saved with an empty box or corrected to 0, and Could not do one (a stored
/// 0) where not.
#[derive(Serialize, Debug, Clone, Copy, PartialEq)]
struct Reads {
	value: i64,
	#[serde(skip_serializing_if = "Option::is_none")]
	load: Option<u32>,
	#[serde(skip_serializing_if = "std::ops::Not::not")]
	done: bool,
}

impl Reads {
	fn of(r: SetReading) -> Self {
		Self { value: r.value, load: r.load, done: r.outcome == outcome::DONE }
	}
}

/// A tree's climb closest to its next step (2026-10-29; the twin of the phones' core `upNext`'s first, SheetBoard.kt):
/// of its ladders a set can move now (Progress.lua `ns.TreeGlow`'s test: open, its day not in, no pain flag holding
/// it, a ring with a fill) whose player's step has a step above it, the one with the fullest ring (`ns.StepRing`, the
/// share of the climb done), a tie in the tree's order, with that fill; none while felt unwell locks every pump
/// (`rest`).
fn tree_climb(words: &Words, tree: &BookTree, ladders: &[BookLadder]) -> Option<(u8, i64)> {
	if tree.reason.as_deref() == Some("rest") {
		return None;
	}
	let sheet = words.trees.iter().find(|t| t.key == tree.tree)?;
	sheet
		.branches
		.iter()
		.flat_map(|b| b.ladders.iter())
		.filter_map(|&id| {
			let row = ladders.iter().find(|l| l.ladder == id)?;
			let fill = row.ring.as_ref()?.fill?;
			let open = row.rung > 0 && row.opens.is_none() && row.waits.is_none() && row.day_in.is_none() && !row.pain;
			let now = row.steps.iter().find(|s| matches!(s.state, StepState::Now { .. }))?.rung?;
			let above = row.steps.iter().any(|s| s.r#move.is_some() && s.rung.is_some_and(|r| r > now));
			(open && above).then_some((id, fill))
		})
		.fold(None, fuller)
}

/// The fuller of two climbs, the first on a tie.
fn fuller(best: Option<(u8, i64)>, climb: (u8, i64)) -> Option<(u8, i64)> {
	if best.is_none_or(|(_, top)| climb.1 > top) { Some(climb) } else { best }
}

/// The step-up closest to done across the trees (2026-11-02, Pump home's row; the twin of the phones' core
/// `upNextFirst`, SheetBoard.kt): of each tree's closest climb ([`tree_climb`]), the one with the fullest ring, a tie
/// in the trees' order; none while nothing climbs (felt unwell rests every tree).
fn closest(words: &Words, trees: &[BookTree], ladders: &[BookLadder]) -> Option<u8> {
	trees.iter().filter_map(|t| tree_climb(words, t, ladders)).fold(None, fuller).map(|(id, _)| id)
}

/// `book`: what the one progression says now.
#[derive(Serialize, Debug, Clone, PartialEq)]
struct BookTopic {
	/// The engine has answered: until then the page shows its pips.
	ready: bool,
	/// Pro's extras: the ladder page's charts and the share card's styles beyond Parchment.
	pro: bool,
	style: ShareStyle,
	/// The day's card and the format the Share window last drew (the Saga).
	card: ShareCard,
	format: ShareFormat,
	/// The day reel's Sound (slice 45).
	sound: bool,
	/// The game's sheet the app shows: its character, whom the share card names, its body's figure (female or male),
	/// which a move's body map takes, and the colour the game draws its figure in ("#RRGGBB", the streak card's
	/// standard-bearer's; none: crimson).
	#[serde(skip_serializing_if = "Option::is_none")]
	character: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	figure: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	tint: Option<String>,
	ladders: Vec<LadderRow>,
	/// The step-up closest to done of all the trees, Pump home's row ([`closest`]): its ladder.
	#[serde(skip_serializing_if = "Option::is_none")]
	closest: Option<u8>,
	moves: Vec<MoveRow>,
	favourites: Vec<u16>,
	#[serde(skip_serializing_if = "Option::is_none")]
	chart: Option<LadderChart>,
}

impl<P: Platform> Desk<P> {
	/// The highest quality (1 Starter … 7 Artifact) of a step any ladder holds, the step now or one held under it or
	/// reached before a rung down, as the colours card counted them until it went on 2026-11-02 (`standing`'s
	/// `colour_top`, 0 before any ladder is placed); none before the engine's first Book answer.
	pub fn book_top_colour(&self) -> Option<u8> {
		let answer = self.book_log.answer.as_ref()?;
		let held = answer.book.ladders.iter().flat_map(|l| &l.steps);
		let held = held.filter(|s| matches!(s.state, StepState::Held | StepState::Now { .. }));
		Some(held.map(|s| s.quality).max().unwrap_or(0))
	}

	/// At the start: the words once, the Log, and the Book once the engine has folded every row.
	pub fn book_log_start(&mut self) {
		match &*WORDS_BUILT {
			Ok(words) => self.publisher.publish(WORDS, words),
			Err(e) => log::error!("the Book's words: {e}"),
		}
		self.book_publish();
		self.log_refresh();
	}

	/// After every change the session made (`Desk::apply`): when the stores took rows or a sheet came, the Book and
	/// the Log are built again (the Book's engine call queued after the merge of those rows).
	pub fn book_log_rows(&mut self, people: bool) {
		let seen = Seen { stores: self.store_mark(), sheets: self.session.sheet_log().records().len() };
		if !people && seen == self.book_log.seen {
			return;
		}
		self.book_log.seen = seen;
		self.book_refresh();
		self.log_refresh();
	}

	/// Pro turned on or off on this computer: Pro's weeks, the charts and the share card's styles show or go.
	pub fn book_log_pro(&mut self) {
		self.book_publish();
		self.log_publish();
	}

	/// The engine's state moved (a fold landed, the profile changed): the Book asks it again, once at a time; while a
	/// fold runs the fold's landing asks.
	pub fn book_refresh(&mut self) {
		if !self.engine_ready() {
			return;
		}
		if self.book_log.asking {
			self.book_log.again = true;
			return;
		}
		self.book_log.asking = true;
		let at = self.p.now_ms().div_euclid(1_000);
		let game =
			self.game_shown(&self.session.sheet(at)).and_then(|g| serde_json::from_value::<GameLadders>(g.sheet).ok());
		let needs = WORDS_BUILT.as_ref().map(needs_of).unwrap_or_default();
		let (mut pages, horizon) = (self.book_log.pages.clone(), self.correct_horizon(at));
		pages.sort_unstable();
		pages.dedup();
		self.engine_call(
			move |engine| {
				let book = engine.book(game.as_ref(), at)?;
				// the extra calls feed a window's Start, a card's gear line and its need rows: one that fails leaves its
				// part empty (the need rows then with no check, no state and no waits line) and the Book still shows
				let mut failed = Vec::new();
				let ids: Vec<u16> = book.moves.iter().map(|m| m.r#move).collect();
				let ladders: Vec<u8> = book.ladders.iter().map(|l| l.ladder).collect();
				let starts = extra(&mut failed, "starts", engine.starts(&ids, at));
				let opens = extra(&mut failed, "ladder_opens", engine.ladder_opens(&ladders));
				let states = extra(&mut failed, "needs", engine.needs(&needs, at).map(Some));
				// each stretch check's Start beside its state, as its row's "Do the check" starts it
				let states = states.map(|states| {
					let rows = needs.iter().zip(states).map(|(entry, state)| {
						let start = check_target(entry, &state)
							.and_then(|target| extra(&mut failed, "check_starts", engine.start(target, at).map(Some)));
						NeedRow { state, start }
					});
					rows.collect::<Vec<_>>()
				});
				let sets = pages
					.into_iter()
					.map(|m| (m, extra(&mut failed, "correctable", engine.correctable(m, at, horizon))))
					.collect();
				let (needs, checks) = WORDS_BUILT
					.as_ref()
					.ok()
					.zip(states)
					.map(|(w, states)| needs_by_ladder(w, states.into_iter()))
					.unwrap_or_default();
				let answer = Answer {
					starts: ids.iter().copied().zip(starts).collect(),
					opens: ladders.iter().copied().zip(opens).collect(),
					needs,
					checks,
					sets,
					book,
				};
				Ok((answer, failed))
			},
			move |helper, answer| {
				helper.book_log.asking = false;
				// the next game day's turn after the build's own moment at the latest, an answer or not: a build asked
				// just before 04:00 and answered after it still rebuilds at that turn, and the lock it read is the one
				// that held then
				let day = &helper.home.days.day;
				let turn = day.start(day.mid(at, 1));
				let locked = helper.session.locked_until(at);
				match answer {
					Ok((answer, failed)) => {
						if failed.is_empty() {
							helper.book_log.extras_failed = false;
						} else if !std::mem::replace(&mut helper.book_log.extras_failed, true) {
							log::warn!("book: shown without what the engine did not answer: {}", failed.join("; "));
						}
						helper.book_log.due = Some(due(&answer, at, locked, turn));
						helper.book_log.answer = Some(answer);
					}
					Err(e) => {
						log::warn!("book: the engine did not answer: {e}");
						// the answer kept still changes by the clock at its own moments
						let kept = helper.book_log.answer.as_ref();
						helper.book_log.due = Some(kept.map_or(turn, |kept| due(kept, at, locked, turn)));
					}
				}
				if std::mem::take(&mut helper.book_log.again) {
					helper.book_refresh();
				}
				helper.book_publish();
			},
		);
	}

	/// The loop's look (every 2 s, `Desk::home_look`): the Book built again once the moment its answer changes by the
	/// clock alone has passed ([`due`]; the game day's turn at the latest).
	pub fn book_look(&mut self, now: i64) {
		if self.book_log.due.is_some_and(|due| now >= due) {
			self.book_log.due = None;
			self.book_refresh();
		}
	}

	/// The game's sheets the helper logged, read once each (a reset, which compacts the log, reads them all again).
	pub fn logged_sheets(&mut self) -> &[history::Sheet] {
		let (records, floor) = (self.session.sheet_log().records(), self.session.store().floor());
		let cache = &mut self.book_log.sheets;
		if records.len() < cache.read || floor != cache.floor {
			*cache = Sheets { floor, ..Sheets::default() };
		}
		for r in records.get(cache.read..).unwrap_or_default() {
			match serde_json::from_str::<history::Sheet>(r.sheet.get()) {
				Ok(sheet) => cache.list.push(history::Sheet { char: r.char, ..sheet }),
				Err(e) => log::debug!("book: a logged sheet does not read: {e}"),
			}
		}
		cache.read = records.len();
		&cache.list
	}

	/// The `book` topic from the engine's last answer and this computer's settings.
	fn book_publish(&mut self) {
		let Ok(words) = &*WORDS_BUILT else { return };
		let pro = self.pro_on() == Some(true);
		let sheet = self.session.sheet(self.p.now_ms().div_euclid(1_000));
		let shown = self.game_shown(&sheet);
		let figure = shown.as_ref().and_then(|g| g.sheet.get("body")?.as_str().map(str::to_owned));
		let tint = shown.as_ref().and_then(|g| figure_tint(g, &sheet));
		let character = shown.and_then(|g| g.name);
		let chart = if pro { self.ladder_chart() } else { None };
		let topic = match &self.book_log.answer {
			Some(Answer { book, starts, opens, needs, checks, sets }) => BookTopic {
				ready: true,
				pro,
				style: self.settings.share_style,
				card: self.settings.share_card,
				format: self.settings.share_format,
				sound: self.settings.share_sound,
				character,
				figure,
				tint: tint.clone(),
				ladders: rows(words, &book.ladders, opens, needs),
				closest: closest(words, &book.trees, &book.ladders),
				moves: book
					.moves
					.iter()
					.map(|m| MoveRow {
						fact: m.clone(),
						start: starts.get(&m.r#move).cloned(),
						checks: checks.get(&m.r#move).cloned(),
						sets: sets.get(&m.r#move).map(|list| list.iter().map(SetRow::of).collect()),
					})
					.collect(),
				favourites: self.favourites(),
				chart,
			},
			None => BookTopic {
				ready: false,
				pro,
				style: self.settings.share_style,
				card: self.settings.share_card,
				format: self.settings.share_format,
				sound: self.settings.share_sound,
				character,
				figure,
				tint: tint.clone(),
				ladders: Vec::new(),
				closest: None,
				moves: Vec::new(),
				favourites: self.favourites(),
				chart: None,
			},
		};
		self.publisher.publish(BOOK, &topic);
	}

	/// The open ladder page's charts: its points over the game's sheets and the typed counts of the move standing for
	/// its rung in the game's rows, each read with every store's corrections (a phone's or this computer's correction of
	/// a game set too).
	fn ladder_chart(&mut self) -> Option<LadderChart> {
		let ladder = self.book_log.ladder?;
		let mv = self.book_log.answer.as_ref()?.book.ladders.iter().find(|l| l.ladder == ladder)?.r#move;
		let day = GameDay::new(facts().constants().day_offset_h, self.p.zone());
		let points = history::ladder_points(self.logged_sheets(), ladder, &day);
		// the standing move's own rises, as its record line draws them
		let rises = history::rises(&points).into_iter().filter(|p| p.mv == i64::from(mv)).map(|p| p.at).collect();
		let (rows, app) = self.log_rows();
		let fixes = history::Fixes::of(&[&rows, &app]);
		let counts = history::attempts(&history::read(&rows, &fixes), mv).into_iter().map(|(at, v)| [at, v]).collect();
		Some(LadderChart { ladder, points, counts, rises, now: self.p.now_ms().div_euclid(1_000) })
	}

	/// The `book` and `share` areas' intents.
	///
	/// # Errors
	/// Why the intent was refused.
	pub fn book_intent(&mut self, intent: &Intent) -> Result<(), String> {
		let number = |name: &str| intent.arg(name).and_then(serde_json::Value::as_u64);
		let on =
			|| intent.arg("on").and_then(serde_json::Value::as_bool).ok_or_else(|| format!("{}: on", intent.kind()));
		match intent.kind() {
			"book.ladder" => {
				self.book_log.ladder = number("ladder").and_then(|l| u8::try_from(l).ok());
				self.book_publish();
			}
			"book.favourite" => {
				let mv = number("move").and_then(|m| u16::try_from(m).ok()).ok_or("book.favourite: move")?;
				let on = on()?;
				let mut favourites = self.favourites();
				favourites.retain(|&m| m != mv);
				if on {
					favourites.push(mv);
					favourites.sort_unstable();
				}
				// a change of the body every device shares (2026-10-09): the list this computer kept alone goes
				let change = Body {
					favourites: Some(stamped(favourites, self.body.body.favourites.as_ref(), self.p.now_ms())),
					..Body::default()
				};
				if !self.settings.favourites.is_empty() {
					self.settings.favourites.clear();
					self.save_settings();
				}
				self.body_changed(&change);
				self.book_publish();
			}
			"book.include" => {
				let mv = number("move").and_then(|m| u16::try_from(m).ok()).ok_or("book.include: move")?;
				let on = on()?;
				self.include(mv, on)?;
			}
			"book.clear" => {
				let mv = number("move").and_then(|m| u16::try_from(m).ok()).ok_or("book.clear: move")?;
				self.clear(intent, Clear::Move, mv)?;
			}
			"book.restart" => {
				let ladder = number("ladder").and_then(|l| u8::try_from(l).ok()).ok_or("book.restart: ladder")?;
				self.clear(intent, Clear::Ladder, u16::from(ladder))?;
			}
			"book.move" => {
				let mv = number("move").and_then(|m| u16::try_from(m).ok()).ok_or("book.move: move")?;
				let shown = intent.arg("shown").and_then(serde_json::Value::as_bool).ok_or("book.move: shown")?;
				self.book_page(mv, shown);
			}
			"book.correct" => self.correct(intent)?,
			"book.card" => {
				// a step's card opened: the count of the cards opened and of a refused or greyed Start's reason
				// (`tree_day`)
				let refused = intent.arg("refused").and_then(serde_json::Value::as_str);
				self.p.tree_card(refused);
			}
			"share.style" => {
				let style: ShareStyle = intent
					.arg("style")
					.and_then(|s| serde_json::from_value(s.clone()).ok())
					.ok_or("share.style: parchment, night or gilded")?;
				self.settings.share_style = style;
				self.save_settings();
				self.book_publish();
			}
			"share.card" => {
				let card: ShareCard = intent
					.arg("card")
					.and_then(|s| serde_json::from_value(s.clone()).ok())
					.ok_or("share.card: saga or bill")?;
				self.settings.share_card = card;
				self.save_settings();
				self.book_publish();
			}
			"share.sound" => {
				self.settings.share_sound = on()?;
				self.save_settings();
				self.book_publish();
			}
			"share.format" => {
				let format: ShareFormat = intent
					.arg("format")
					.and_then(|s| serde_json::from_value(s.clone()).ok())
					.ok_or("share.format: story, post, square, sticker or reel")?;
				self.settings.share_format = format;
				self.save_settings();
				self.book_publish();
			}
			_ => return Err(intent.unknown()),
		}
		Ok(())
	}

	/// A move's record cleared or a ladder started over (`scope`, the move's or the ladder's `id`): a clear row the
	/// engine writes (it refuses a move its library lacks and a ladder that is no strength or pose ladder open now), kept
	/// as this computer's own (`Desk::keep_own`), whose new mark of the stores builds the Book, Hero and Pump home
	/// again; every paired phone takes the row at its next exchange, and the game through the mirror at its next load.
	/// Refused while a pump is on screen or the engine folds.
	fn clear(&mut self, intent: &Intent, scope: Clear, id: u16) -> Result<(), String> {
		if self.desk_pump_running() {
			return Err(format!("{}: a pump is on screen", intent.kind()));
		}
		self.engine_ready_or(intent)?;
		let at = self.p.now_ms().div_euclid(1_000);
		let what = match scope {
			Clear::Move => CLEAR_ROW,
			Clear::Ladder => RESTART_ROW,
		};
		self.engine_call(move |e| e.clear_row(scope, id, at), move |h, row| h.keep_own(what, row.map(|r| vec![r])));
		Ok(())
	}

	/// Include in pumps: the body's moves with `mv` checked or not (`Moves::check`), a change of the body made on this
	/// computer through Your moves' owner (`Desk::change_moves`), which every paired phone takes at its next
	/// exchange.
	fn include(&mut self, mv: u16, on: bool) -> Result<(), String> {
		let (group, opt_in, members) = facts().group(mv).ok_or_else(|| format!("book.include: no move {mv}"))?;
		self.change_moves(|moves| moves.check(mv, on, group, opt_in, &members));
		Ok(())
	}
}

// Correct a count (slice 44, 2026-10-18) ------------------------------------------------------------------------------

/// Correct a count as the loop runs it (docs/plan.md slice 44, "One correction, from any device"; the window is
/// `views/book/correct.js`): the set the window corrects and the count step's check of its value (the pump's own
/// `CountStep`: refused past one set, or "Is that right?" once, its Yes keeping the count it asked about), and the
/// page's latest `seq`, so an answer for an earlier Save is never shown as the latest.
#[derive(Default)]
pub struct Correcting {
	set: Option<SetKey>,
	step: CountStep,
	seq: u64,
}

/// What Correct a count's Save asks (`book.correct`).
struct CorrectAsk {
	seq: u64,
	set: SetKey,
	change: Change,
	/// "Is that right?"'s Yes: the count it asked about, kept.
	confirm: bool,
}

/// A correction's new value: a count (a rep set's or a counted set's, a hold's seconds) with a loaded rep set's weight
/// (tenths of a kg, 0 none; none keeps the set's own), or a pose's answer (`ns.Enum.mark`, 1 Not yet to 4 Perfect).
#[derive(Debug, Clone, Copy)]
enum Change {
	Count(Typed),
	Answer(u8),
}

impl CorrectAsk {
	fn of(intent: &Intent) -> Result<Self, String> {
		let kind = intent.kind();
		let int = |name: &str| intent.arg(name).and_then(serde_json::Value::as_i64);
		let field = |name: &str| format!("{kind}: {name}");
		let seq = intent.arg("seq").and_then(serde_json::Value::as_u64).ok_or_else(|| field("seq"))?;
		let set = SetKey {
			ts: int("ts").ok_or_else(|| field("ts"))?,
			quest: int("quest").and_then(|q| u32::try_from(q).ok()).ok_or_else(|| field("quest"))?,
			slot: int("slot").and_then(|q| u8::try_from(q).ok()).ok_or_else(|| field("slot"))?,
			r#move: int("move").and_then(|q| u16::try_from(q).ok()).ok_or_else(|| field("move"))?,
		};
		let change = match (int("value"), int("answer")) {
			(Some(value), None) if value >= 0 => {
				let load = match intent.arg("load") {
					None | Some(serde_json::Value::Null) => None,
					Some(l) => Some(l.as_u64().and_then(|l| u32::try_from(l).ok()).ok_or_else(|| field("load"))?),
				};
				Change::Count(Typed { value, load, answer: answer::YES })
			}
			(None, Some(answer)) => Change::Answer(u8::try_from(answer).map_err(|_| field("answer"))?),
			_ => return Err(format!("{kind}: a value or an answer")),
		};
		let confirm = intent.arg("confirm").and_then(serde_json::Value::as_bool).unwrap_or(false);
		Ok(Self { seq, set, change, confirm })
	}
}

/// What a correction came to, as the window shows it.
#[derive(Serialize, Debug, PartialEq)]
#[serde(tag = "outcome", rename_all = "snake_case")]
enum CorrectOutcome {
	/// Written: the set reads its new value on this computer now (`now`, as the fold reads it, which the list shows
	/// next: a capped max set corrected over its window's top keeps the top, a count corrected to 0 is done), and every
	/// device and the game take the rows. `stays` says why its step stays where it is, where the step rules asked again
	/// would move it: `game` (a set of the game's, whose steps only the game asks again) or `later` (its ladder moved on
	/// since the set); `steps` what the rows changed on the steps (a climb taken back, a step no longer Mastered: the
	/// engine's `steps`, 2026-10-19), which the done view says under its first line.
	Done {
		#[serde(skip_serializing_if = "Option::is_none")]
		stays: Option<&'static str>,
		#[serde(flatten)]
		now: Reads,
		#[serde(skip_serializing_if = "Option::is_none")]
		steps: Option<StepChanges>,
	},
	/// "Is that right?" once, in the count step's words (its Yes sends the count again with `confirm`), and its "Change
	/// it".
	Asks { check: CheckView, change: &'static str },
	/// Refused past one set, the count step's "Max 100: count one set".
	Max { line: String },
	/// The set has left the window meanwhile: nothing written.
	Gone,
	/// Nothing written: the engine did not answer (logged), or a pump began while the count was checked; the window's
	/// Save is ready again (and waits while a pump runs).
	Failed,
}

/// The `book.correct` topic: the outcome of the Save the page numbered `seq`, for the set it names.
#[derive(Serialize, Debug, PartialEq)]
struct CorrectView {
	seq: u64,
	#[serde(flatten)]
	set: SetKey,
	#[serde(flatten)]
	outcome: CorrectOutcome,
}

/// What "Is that right?" names for a corrected count, from the state just before the set (`Engine::correct_check`),
/// a loaded move's weight in each hand from its library facts.
fn against(check: &CorrectCheck, set: &SetKey) -> Against {
	let f = facts();
	let each = f.library.moves.get(&f.taken(set.r#move)).is_some_and(|m| m.load.as_deref() == Some("each"));
	Against {
		record: check.record,
		top: check.top,
		top_load: check.top_load,
		top_reps: check.top_reps,
		plan: check.plan,
		hold: check.hold,
		counted: check.counted,
		each,
	}
}

impl<P: Platform> Desk<P> {
	/// The game's ring horizon as Correct a count reads it (slice 44, contract/companion-api.yaml `SheetResponse.rh`):
	/// the largest `rh` among the newest sheets of every character this computer knows at `at`, as the phones read them
	/// (`serve_shown`: the engine's fold of the save in its character's place, which carries its save's horizon since
	/// 2026-10-19), past which the engine keeps the window's reach a margin of rows (DB.lua `ns.CorrectReach`, slice
	/// 44's review), so the window never lists a set some save has folded away or a trim before the game's next load
	/// could fold (whose correction the game would ignore); none while no sheet names one (no row has left a ring, or an
	/// older addon).
	fn correct_horizon(&self, at: i64) -> Option<i64> {
		let mut sheet = self.session.sheet(at);
		self.serve_shown(&mut sheet);
		sheet.characters.iter().filter_map(|c| c.sheet.as_ref()?.get("rh")?.as_i64()).filter(|&rh| rh > 0).max()
	}

	/// Correct a count's Save (`book.correct`): the set read again in the window, its new count checked as the count
	/// step checks one (`Engine::correct_check`: refused past one set, "Is that right?" once, never a pose's answer),
	/// then its rows written (`Engine::correct_rows`), each answer on the loop in turn and its outcome on the
	/// `book.correct` topic. A Save that is no Yes asks again; refused while a pump is on screen (the window's Save
	/// waits until it ends) or the engine folds.
	fn correct(&mut self, intent: &Intent) -> Result<(), String> {
		let ask = CorrectAsk::of(intent)?;
		if self.desk_pump_running() {
			return Err(format!("{}: a pump is on screen", intent.kind()));
		}
		self.engine_ready_or(intent)?;
		let at = self.p.now_ms().div_euclid(1_000);
		let horizon = self.correct_horizon(at);
		let c = &mut self.book_log.correct;
		if c.set != Some(ask.set) || !ask.confirm {
			c.step.clear();
		}
		(c.set, c.seq) = (Some(ask.set), ask.seq);
		let CorrectAsk { seq, set, .. } = ask;
		match ask.change {
			Change::Answer(answer) => self.correct_write(seq, set, Change::Answer(answer), at, horizon),
			Change::Count(typed) => self.engine_call(
				move |e| e.correct_check(set, typed.value, typed.load, at, horizon),
				move |h, answer| h.correct_checked(seq, set, typed, answer, (at, horizon)),
			),
		}
		Ok(())
	}

	/// A corrected count's check came back (dropped where a later Save came meanwhile): the set gone, refused or asked
	/// about once (the count step's own rule, `CountStep::checked`, on `ns.CorrectionCheck`'s check, which asks no low
	/// count of a counted set as its count step asks none), or written.
	fn correct_checked(
		&mut self,
		seq: u64,
		set: SetKey,
		typed: Typed,
		answer: Result<CorrectionCheck, EngineError>,
		(at, horizon): (i64, Option<i64>),
	) {
		if self.book_log.correct.seq != seq {
			return;
		}
		// a pump began on this computer while the check was asked: refused as at the Save (nothing written mid-pump)
		if self.desk_pump_running() {
			return self.correct_publish(seq, set, CorrectOutcome::Failed);
		}
		let check = match answer {
			Ok(CorrectionCheck::Gone) => return self.correct_publish(seq, set, CorrectOutcome::Gone),
			Ok(CorrectionCheck::Clear) => return self.correct_write(seq, set, Change::Count(typed), at, horizon),
			Ok(CorrectionCheck::Asks { check }) => check,
			Err(e) => {
				log::warn!("book: no check of the correction of {set:?}: {e}");
				return self.correct_publish(seq, set, CorrectOutcome::Failed);
			}
		};
		let step = &mut self.book_log.correct.step;
		if let Outcome::Record { .. } = step.checked(typed, Some(check.check)) {
			return self.correct_write(seq, set, Change::Count(typed), at, horizon);
		}
		let outcome = match step.asked() {
			Some((Check::Max, _)) => CorrectOutcome::Max { line: max_line(count_most(check.counted, check.hold)) },
			// a weight box emptied (0: no weight) is no weight in the question's words
			Some((asks, typed)) => CorrectOutcome::Asks {
				check: check_view(asks, Typed { load: typed.load.filter(|&l| l > 0), ..typed }, &against(&check, &set)),
				change: &facts().sheet.words.count.really_change,
			},
			None => CorrectOutcome::Failed,
		};
		self.correct_publish(seq, set, outcome);
	}

	/// A correction's rows (`Engine::correct_rows`), kept as this computer's own (`Desk::keep_own`: every paired phone
	/// takes them at its next exchange, the game through the mirror at its next load, and the stores' new mark builds
	/// the Book, Hero, the Saga and Pump home again), then the mastered sweep, as after a set (`Desk::engine_sweep`).
	/// Rows written are kept whatever Save came meanwhile.
	fn correct_write(&mut self, seq: u64, set: SetKey, change: Change, at: i64, horizon: Option<i64>) {
		let (value, load, answer) = match change {
			Change::Count(typed) => (Some(typed.value), typed.load, None),
			Change::Answer(answer) => (None, None, Some(answer)),
		};
		self.engine_call(
			move |e| e.correct_rows(set, value, load, answer, at, horizon),
			move |h, written| {
				let outcome = match written {
					Ok(Corrected::Written { rows, stays, now, steps }) => {
						// each correction written goes to helper.log, as each answer a pump took does
						match answer {
							Some(a) => log::info!("book: corrected {set:?}, how close: {a}"),
							None => log::info!("book: corrected {set:?}, count: {}", value.unwrap_or_default()),
						}
						h.keep_own(CORRECT_ROW, Ok(rows));
						h.engine_sweep();
						h.book_log.correct.step.clear();
						let stays = match stays {
							Some(Stays::Source) => Some("game"),
							Some(Stays::Later) => Some("later"),
							Some(Stays::Other) | None => None,
						};
						CorrectOutcome::Done { stays, now: Reads::of(now), steps }
					}
					Ok(Corrected::Gone) => CorrectOutcome::Gone,
					Err(e) => {
						log::warn!("book: the correction of {set:?} was not written: {e}");
						CorrectOutcome::Failed
					}
				};
				h.correct_publish(seq, set, outcome);
			},
		);
	}

	/// A move's page stands (`shown`) or goes: the Book is built again with that move's sets for Correct a count when
	/// it is the first of its pages.
	fn book_page(&mut self, mv: u16, shown: bool) {
		let pages = &mut self.book_log.pages;
		if shown {
			let first = !pages.contains(&mv);
			pages.push(mv);
			if first {
				self.book_refresh();
			}
		} else if let Some(i) = pages.iter().position(|&m| m == mv) {
			pages.remove(i);
		}
	}

	/// The main window is gone, and every move page in it (`DeskAsk::Gone`): the Book carries no move's sets until a
	/// page stands again.
	pub fn book_pages_gone(&mut self) {
		self.book_log.pages.clear();
	}

	/// The `book.correct` topic.
	fn correct_publish(&self, seq: u64, set: SetKey, outcome: CorrectOutcome) {
		self.publisher.publish(CORRECT, &CorrectView { seq, set, outcome });
	}
}

/// A pose family's goal pose at the rung it stands on, as its row shows it: the own goal pose of the pose standing for
/// the rung where it is of that rung (its main pose, the anchor or a twin: Progress.lua `ns.MainPose`, as the Talents
/// tab's card names it, Tree.lua `Tree.CardNow`; any goal pose of the step counts the family's day since slice 47);
/// none under a stand-in from below, whose holds count nothing toward the climb.
fn rung_mark(l: &BookLadder) -> Option<&'static str> {
	let f = facts();
	let own = f.library.moves.get(&l.r#move).is_some_and(|m| m.ladder == Some(l.ladder) && m.rung == l.rung);
	if own { f.mark(l.r#move) } else { None }
}

/// The colour the game draws the character of the sheet shown in (the summary's `fig`, 0xRRGGBB, which only the strip's
/// sheet carries: an engine's fold of the save takes it from that character's), as "#RRGGBB"; none where neither
/// names one (the figures' crimson then).
fn figure_tint(shown: &GameSheet, sheet: &Sheet) -> Option<String> {
	let strip = || sheet.characters.iter().find(|c| c.index == shown.char)?.sheet.as_ref()?.get("fig");
	let fig = shown.sheet.get("fig").or_else(strip)?.as_u64()?;
	Some(format!("#{:06X}", fig & 0xFF_FFFF))
}

/// The engine's ladders as the Book lists them: in the ladders' order by stat (`ns.LADDER_ORDER`), each one the words
/// name (never a retired id or one without moves), a ladder that waits whole among them (slice 38: the trees draw every
/// ladder, its `waits` saying why), with its step, its record's unit, a pose family's goal pose and hold, its gear line
/// and the engine's reading of its links' steps.
fn rows(
	book: &Words,
	ladders: &[BookLadder],
	opens: &BTreeMap<u8, Vec<SetupStep>>,
	needs: &BTreeMap<u8, Vec<NeedRow>>,
) -> Vec<LadderRow> {
	book.ladders
		.iter()
		.filter_map(|words| {
			let l = ladders.iter().find(|l| l.ladder == words.id)?;
			let step = usize::from(l.rung).min(words.steps.len()).checked_sub(1).map_or(0, |i| words.steps[i]);
			let mv = book.moves.iter().find(|m| m.id == l.r#move);
			let pose = words.dose == "pose";
			let mark = if pose && l.rung > 0 { rung_mark(l) } else { None };
			Some(LadderRow {
				ladder: l.clone(),
				step,
				seconds: mv.is_some_and(|m| m.dose == "hold"),
				mark,
				hold: if pose { mv.and_then(|m| m.secs) } else { None },
				gear: opens.get(&l.ladder).cloned().unwrap_or_default(),
				needs: needs.get(&l.ladder).cloned(),
			})
		})
		.collect()
}
