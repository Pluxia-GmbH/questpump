//! The Saga's day on the computer (the Saga, 2026-10-15; contract/fixtures/history.json `saga`, whose `about` states
//! every rule once for the game's fold, the phones' cores and this port, checked against its cases by a probe outside
//! the repo): a game day's moves from every source the helper holds, each counted on the game day of its own row's time
//! (a pump across the day's turn gives each day its own, a level-up's count too); its chapter and its rank among the
//! player's own days (the quality word and the best day), anchored on the game's all-time active days and best days
//! where the rows begin later (`Saga::anchored`); its strip of pumps by the hour; its moves by kind and the
//! muscles they trained; its records and unlocks and the move that stands for it; the week holding it; and its reel
//! (rule 18, slice 45: `Saga::reel`, the moves the day reel films and its sounds, which `desk::reel` makes). One owner:
//! the Saga's topic and Pump home's today's chapter both read it (`desk::log`), never a second count.
//!
//! The pumps are the ones the Log shows (`history::shown`). An unlock is a climb's rung row (`cause::climbs`, a stretch
//! check's lift among them) not undone (DB.lua `pairing`) to a rung above the one its ladder started on that was not
//! reached before the day, a start-over included (Stats.lua `markReached`, which calls a climb back Rekindled): the
//! sheet shown names each rung's day and the start rung (`Summary::earned`, the fold's own), the rows the climbs before
//! the day it does not date (an orange or gold rung's, whose day there is its earning; a rung climbed again after a
//! start-over, which it dates anew).

use std::collections::{BTreeMap, HashMap, HashSet};

use questpump_core::codes::{cause, clear, kind};
use serde::{Deserialize, Serialize};

use super::facts::Facts;
use super::history::{Entry, GameDay, Pump, Row, civil, weekday};

/// A day's word among the player's own days (rule 7): Legendary is a best day, shown as Best day ever.
#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Quality {
	Common,
	Uncommon,
	Rare,
	Epic,
	Legendary,
}

/// The tunables of the day's rank (library.json's constants, Constants.lua's).
#[derive(Debug, Clone, Copy)]
pub(crate) struct Tune {
	/// The earlier days with a move a day needs before it gets a word or a best day.
	pub(crate) min_days: i64,
	/// The calendar game days before a day its rank looks back over.
	pub(crate) rank_days: i64,
	/// The shares (percent) of those days that may beat a day for it to be Epic, Rare and Uncommon.
	pub(crate) shares: [i64; 3],
}

impl Tune {
	pub(crate) fn of(f: &Facts) -> Self {
		let c = f.constants();
		let share = |i: usize| c.day_rank_shares.get(i).copied().unwrap_or(0);
		Self { min_days: c.best_day_min_days, rank_days: c.day_rank_days, shares: [share(0), share(1), share(2)] }
	}
}

/// A day's place among the player's own days (rules 1-8): its chapter (only an active day has one), the most moves on
/// one active day before it and the earliest day that reached it (moves, day), the best day and its word.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Rank {
	pub(crate) chapter: Option<i64>,
	pub(crate) old_best: Option<(i64, i64)>,
	pub(crate) best: bool,
	pub(crate) quality: Option<Quality>,
}

/// The better of a best day so far and another day (moves, day): the most moves, of two alike the earlier day (a later
/// equal day did not beat the day that set it; rule 4).
fn better(old: Option<(i64, i64)>, (moves, day): (i64, i64)) -> Option<(i64, i64)> {
	if moves < 1 || old.is_some_and(|(most, at)| moves < most || (moves == most && day >= at)) {
		return old;
	}
	Some((moves, day))
}

/// The rank from the earlier days' count and their best, the window read from `days`.
fn rank_from(days: &BTreeMap<i64, i64>, d: i64, moves: i64, earlier: i64, old: Option<(i64, i64)>, tune: Tune) -> Rank {
	let (mut window, mut above) = (0, 0);
	for (_, &m) in days.range(d - tune.rank_days..d) {
		window += 1;
		if m > moves {
			above += 1;
		}
	}
	let active = moves >= 1;
	let gate = earlier >= tune.min_days;
	let best = gate && active && old.is_none_or(|(most, _)| moves > most);
	let quality = if !gate || !active {
		None
	} else if best {
		Some(Quality::Legendary)
	} else if window == 0 {
		None
	} else {
		// compared in whole numbers: exactly 10 % is rare, 25 % uncommon, 50 % common
		let [epic, rare, uncommon] = tune.shares;
		Some(if above * 100 < epic * window {
			Quality::Epic
		} else if above * 100 < rare * window {
			Quality::Rare
		} else if above * 100 < uncommon * window {
			Quality::Uncommon
		} else {
			Quality::Common
		})
	};
	Rank { chapter: active.then_some(earlier + 1), old_best: old, best, quality }
}

/// A strip's part of the day by the local hour its pump started (rule 9).
#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Bucket {
	Morning,
	Afternoon,
	Evening,
	Night,
}

impl Bucket {
	fn of(hour: i64) -> Self {
		match hour {
			4..=11 => Self::Morning,
			12..=16 => Self::Afternoon,
			17..=21 => Self::Evening,
			_ => Self::Night,
		}
	}

	fn place(self) -> usize {
		self as usize
	}
}

/// The kinds of moves by their block (library.json's), in the strip's order from the axis.
const BLOCKS: [&str; 4] = ["strength", "cardio", "mobility", "yoga"];

/// A move's kind: its block's place in `BLOCKS`; none for a move the library lacks.
fn kind_of(f: &Facts, mv: u16) -> Option<usize> {
	let block = &f.tags(mv)?.block;
	BLOCKS.iter().position(|b| b == block)
}

/// A bar of the day's strip: a pump with a move done on the day, the minute past the day's turn its start's wall clock
/// reads (0 for a pump begun the day before), its part of the day, and its moves of the day by kind.
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub(crate) struct Bar {
	pub(crate) minute: i64,
	pub(crate) bucket: Bucket,
	pub(crate) kinds: [i64; 4],
}

/// A day's strip (rules 9-11): its bars in their pumps' order, the parts' totals (morning, afternoon, evening,
/// night), its moves by kind, and the muscles they trained most and some.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Strip {
	pub(crate) bars: Vec<Bar>,
	pub(crate) buckets: [i64; 4],
	pub(crate) kinds: [i64; 4],
	pub(crate) most: Vec<u8>,
	pub(crate) some: Vec<u8>,
}

/// A set's value as the Saga writes it: its typed count (a hold's seconds, a counted set's count, else its reps) and,
/// for a loaded move's, the weight it was done with in tenths of a kg.
#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Set {
	pub(crate) value: i64,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub(crate) load: Option<u32>,
}

impl Set {
	/// Better than `other`: by load score where either carries a weight (Progress.lua `ns.LoadScore`), else by its
	/// count; the earlier keeps a tie.
	fn beats(self, other: Self, epley: i64) -> bool {
		let score = |s: Self| s.load.map_or(0, |l| i64::from(l) * (epley + s.value));
		if self.load.is_some() || other.load.is_some() { score(self) > score(other) } else { self.value > other.value }
	}
}

/// A record of the day (rule 12): its move, its best set of the day and its record at the day's start (none for a
/// first), and whether it counts seconds.
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub(crate) struct RecordLine {
	#[serde(rename = "move")]
	pub(crate) mv: u16,
	pub(crate) best: Set,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub(crate) was: Option<Set>,
	pub(crate) hold: bool,
}

/// A step the day reached for the first time (rule 13): the move standing for the rung, the ladder and the rung, and
/// its quality (1 Starter … 7 Artifact).
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub(crate) struct Unlock {
	#[serde(rename = "move")]
	pub(crate) mv: u16,
	pub(crate) ladder: u8,
	pub(crate) rung: u8,
	pub(crate) quality: u8,
}

/// Where the move that stands for a day or a week came from (rule 14).
#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum HeroFrom {
	Unlock,
	Record,
	Most,
}

/// The move that stands for a day or a week (its knight) and where it came from.
#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Hero {
	#[serde(rename = "move")]
	pub(crate) mv: u16,
	pub(crate) from: HeroFrom,
}

/// The line under the number: the unlock that stands for the day, or its record; none for a move only done most.
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub(crate) enum Highlight {
	Unlock {
		#[serde(rename = "move")]
		mv: u16,
		quality: u8,
	},
	Record {
		#[serde(rename = "move")]
		mv: u16,
		best: Set,
		hold: bool,
	},
}

/// A day's records and unlocks, the move that stands for it and the line under its number.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Highlights {
	pub(crate) records: Vec<RecordLine>,
	pub(crate) unlocks: Vec<Unlock>,
	pub(crate) hero: Option<Hero>,
	pub(crate) highlight: Option<Highlight>,
}

/// A day of the week strip and its moves.
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub(crate) struct WeekDay {
	pub(crate) day: String,
	pub(crate) moves: i64,
}

/// The XP a day or a week gained and the level it ended on, from the sheets (rule 15): the level it began on where a
/// sheet before it names one, and the XP into the level and the level's need, its bar.
#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Level {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub(crate) from: Option<i64>,
	pub(crate) to: i64,
	pub(crate) into: i64,
	pub(crate) need: i64,
}

/// The game week holding a day (rule 17): its first day, its seven days, the day ringed, the days a rest token kept (by
/// their place; today's week's alone, from the streak's board by date), the dashed Old best line, and what the week's
/// card shows: its moves, its days with a move, its records, its XP, its knight and its line.
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WeekView {
	pub(crate) start: String,
	pub(crate) days: Vec<WeekDay>,
	pub(crate) ringed: usize,
	pub(crate) rested: Vec<usize>,
	pub(crate) old_best: Option<i64>,
	pub(crate) moves: i64,
	pub(crate) active: i64,
	pub(crate) records: i64,
	pub(crate) xp: Option<i64>,
	pub(crate) hero: Option<Hero>,
	pub(crate) highlight: Option<Highlight>,
}

/// What the game's sheets say of a day (rules 15-16; `desk::log` reads them): the XP it gained, its level, the streak it
/// ended with and the XP its week gained; each none where no sheet knows it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Told {
	pub(crate) xp: Option<i64>,
	pub(crate) level: Option<Level>,
	pub(crate) streak: Option<i64>,
	pub(crate) week_xp: Option<i64>,
}

/// The most moves on one earlier day and the day that set it.
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub(crate) struct OldBest {
	pub(crate) moves: i64,
	pub(crate) day: String,
}

/// What a reel's shot shows in the highlight slot (rule 18), always the move whose film plays under it: its Quest Bill
/// line, the day's highlight (a hero with no bill line; the end where the lead is the highlight's move), or, on the end
/// of a day with no record and no unlock, its name.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Words {
	Line,
	Highlight,
	Name,
}

/// A shot of the reel: its move (the figure's film plays from its key pose at its real tempo), its start in the reel
/// (ms) and its slot's words.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Shot {
	#[serde(rename = "move")]
	pub(crate) mv: u16,
	pub(crate) from: i64,
	pub(crate) words: Words,
}

/// A sound of the reel (contract/design/sound/sounds.json): the number's pop, a best day's fanfare, a record's or an
/// unlock's jingle.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ReelSound {
	Reveal,
	Fanfare,
	Record,
}

impl ReelSound {
	/// Its sounds.json id, the bundle's `resources/sounds/<id>.ogg`.
	pub(crate) fn id(self) -> &'static str {
		match self {
			Self::Reveal => "reveal",
			Self::Fanfare => "fanfare",
			Self::Record => "record",
		}
	}
}

/// A sound at its time in the reel (ms).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Cue {
	pub(crate) sound: ReelSound,
	pub(crate) at: i64,
}

/// The day reel (rule 18): the move that leads it, its shots, the end on the lead's film again, its length (ms) and its
/// sounds as with Sound on (the maker leaves them out with it off).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub(crate) struct Reel {
	pub(crate) lead: u16,
	pub(crate) moves: Vec<Shot>,
	pub(crate) end: Shot,
	pub(crate) ms: i64,
	pub(crate) sounds: Vec<Cue>,
}

/// What of a day the reel's picks, words and sounds read (rule 18): the move that stands for it, whether a highlight
/// stands for it (an unlock or a record, the hero's), whether it is a best day and whether it holds a record or an
/// unlock.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ReelDay {
	pub(crate) hero: Option<Hero>,
	pub(crate) highlight: bool,
	pub(crate) best: bool,
	pub(crate) marked: bool,
}

/// The reel's times (contract/design/tokens.json, `desk::reel::Timing` reads them): a move's shot and the end (ms), the
/// Saga's reveal's start in the reel and its length (ms), and the most moves.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ReelTune {
	pub(crate) move_ms: i64,
	pub(crate) end_ms: i64,
	pub(crate) reveal_from_ms: i64,
	pub(crate) reveal_ms: i64,
	pub(crate) moves: usize,
}

/// A day as the Saga and its day page show it (`log`'s `today` and `open`): its key and how many game days before
/// today's it is, its chapter, moves, word and best day with the old best, its strip (its bars and the parts' totals),
/// its kinds, its muscles (Most, Some), its records, unlocks, knight and line, its Quest Bill's lines, its reel (none
/// where no move of the day has its film on this computer), its XP, level and streak where known, its week, and its
/// pumps as the Log lists them.
#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SagaDay {
	pub(crate) day: String,
	pub(crate) ago: i64,
	pub(crate) chapter: Option<i64>,
	pub(crate) moves: i64,
	pub(crate) quality: Option<Quality>,
	pub(crate) best: bool,
	pub(crate) old_best: Option<OldBest>,
	/// The strip's pumps, a bar each (contract/fixtures/history.json `saga` rules 9-11, which pin it): no page counts a
	/// day by its pumps since its moves stand for it (David, 2026-10-27: "Instead of pumps count moves please").
	pub(crate) pumps: i64,
	pub(crate) bars: Vec<Bar>,
	pub(crate) buckets: [i64; 4],
	pub(crate) kinds: [i64; 4],
	pub(crate) most: Vec<u8>,
	pub(crate) some: Vec<u8>,
	pub(crate) records: Vec<RecordLine>,
	pub(crate) unlocks: Vec<Unlock>,
	pub(crate) hero: Option<Hero>,
	pub(crate) highlight: Option<Highlight>,
	pub(crate) bill: Vec<BillLine>,
	pub(crate) reel: Option<Reel>,
	pub(crate) xp: Option<i64>,
	pub(crate) level: Option<Level>,
	pub(crate) streak: Option<i64>,
	pub(crate) week: WeekView,
	pub(crate) entries: Vec<Entry>,
}

/// An earlier day's row under Earlier chapters: a chapter's (its day, chapter, moves, word and best day), or a quiet
/// one for a game day with pumps and no move done (its day, its moves 0; no chapter, word or best day). The page's held
/// book counts the new moves by them (slice 44).
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub(crate) struct ChapterRow {
	pub(crate) day: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub(crate) chapter: Option<i64>,
	pub(crate) moves: i64,
	pub(crate) quality: Option<Quality>,
	pub(crate) best: bool,
}

/// A move done, as the day reads it: when and on which game day, which, its set (none without a count), its reps and
/// seconds as the row holds them, and whether it was a record.
#[derive(Debug, Clone, Copy)]
struct Done {
	ts: i64,
	day: i64,
	mv: u16,
	set: Option<Set>,
	reps: Option<i64>,
	seconds: i64,
	record: bool,
}

/// A line of the day's Quest Bill: a move, how many times it was done that day, and its total over them: for a move
/// dosed in reps the reps its counted sets add up to (none, a blank amount, while no set of it has a count; never "0"),
/// for any other its seconds (a counted cardio set's too); and when it was first done that day, which orders the lines
/// (tokens' `QuestBill`: the moves with a record or an unlock that day first, then the others, each group the most done
/// first, then the one done first, then the lower move id).
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub(crate) struct BillLine {
	#[serde(rename = "move")]
	pub(crate) mv: u16,
	pub(crate) count: i64,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub(crate) reps: Option<i64>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub(crate) seconds: Option<i64>,
	#[serde(skip)]
	first: i64,
}

/// The Saga's days over the rows the helper holds.
pub(crate) struct Saga<'a> {
	f: &'a Facts,
	day: &'a GameDay,
	tune: Tune,
	/// The pumps the Log shows, by their start (stable).
	pumps: Vec<&'a Pump>,
	/// Every active game day's moves, by its index.
	days: BTreeMap<i64, i64>,
	/// Every move done, by time (so by game day too): the records' and the knights' sets.
	done: Vec<Done>,
	/// The climbs not undone (rung rows of an up, a plateau, a placement high or a check's lift), by time.
	climbs: Vec<Climb>,
	/// Records cleared and ladders started over, by time.
	clears: Vec<Cleared>,
	/// The sheet shown's earned steps (`Summary::earned`), by ladder id from 1.
	earned: &'a [Vec<i64>],
	/// The active days the anchoring game sheet knows and the rows do not (rule 3: the rows begin after the player's
	/// first days), added to every chapter and earlier count.
	extra: i64,
	/// The game's two best days of all time as that sheet names them (game day, moves; rule 4), weighed into each old
	/// best.
	best_days: Vec<(i64, i64)>,
}

impl<'a> Saga<'a> {
	/// The Saga over the pumps the Log shows (`pumps`), every row of both stores (`rows`: their rung and clear rows) and
	/// the sheet shown's earned steps.
	pub(crate) fn new(
		f: &'a Facts,
		day: &'a GameDay,
		mut pumps: Vec<&'a Pump>,
		rows: &[Row],
		earned: &'a [Vec<i64>],
	) -> Self {
		pumps.sort_by_key(|p| p.id);
		let mut days = BTreeMap::new();
		let mut done = Vec::new();
		for p in &pumps {
			for m in &p.moves {
				*days.entry(day.index(m.ts)).or_insert(0) += 1;
				// a merged move's set reads as the move that took it, as the fold reads it (its records, its clears), and a
				// hold of a move that switches sides from before that held both sides at each side's half; a set its typed
				// count alone (rule 12: a counted set's count saved with no record attempt is none)
				let mv = f.taken(m.mv);
				let value = if f.hold(mv) {
					Some(f.side_seconds(mv, m.ts, m.seconds)).filter(|&s| s > 0)
				} else {
					m.reps.filter(|_| m.typed)
				};
				done.push(Done {
					ts: m.ts,
					day: day.index(m.ts),
					mv,
					set: value.map(|value| Set { value, load: m.load }),
					reps: m.reps,
					seconds: m.seconds,
					record: m.record,
				});
			}
		}
		done.sort_by_key(|d| d.ts);
		let (climbs, mut clears) = rungs(f, rows);
		for c in clears.iter_mut().filter(|c| c.scope == clear::MOVE) {
			c.id = f.taken(c.id);
		}
		Self { f, day, tune: Tune::of(f), pumps, days, done, climbs, clears, earned, extra: 0, best_days: Vec::new() }
	}

	/// The Saga anchored on a game sheet (rules 3 and 4; `desk::log` picks the one that knows the most active days): its
	/// all-time count of active days (`ad`) less the active days the rows hold up to its game day `sheet_day`, never
	/// below 0, and its two best days (`bd`).
	pub(crate) fn anchored(mut self, active_days: i64, best_days: &[(i64, i64)], sheet_day: i64) -> Self {
		let held = i64::try_from(self.days.range(..=sheet_day).count()).unwrap_or(i64::MAX);
		self.extra = (active_days - held).max(0);
		self.best_days = best_days.to_vec();
		self
	}

	/// Day `d`'s moves.
	pub(crate) fn moves(&self, d: i64) -> i64 {
		self.days.get(&d).copied().unwrap_or(0)
	}

	/// The game's best days (`bd`) dated before day `d` weighed into the rows' best before it.
	fn old_best(&self, rows: Option<(i64, i64)>, d: i64) -> Option<(i64, i64)> {
		self.best_days.iter().filter(|&&(day, _)| day < d).fold(rows, |old, &(day, moves)| better(old, (moves, day)))
	}

	/// Day `d`'s rank as it stands (today's with its moves so far; rules 1-8, its word and its best day only rise),
	/// anchored on the game's all-time days (`anchored`).
	pub(crate) fn rank(&self, d: i64) -> Rank {
		let (mut earlier, mut old) = (self.extra, None);
		for (&e, &m) in self.days.range(..d) {
			earlier += 1;
			old = better(old, (m, e));
		}
		rank_from(&self.days, d, self.moves(d), earlier, self.old_best(old, d), self.tune)
	}

	/// The earlier days' rows before `before`, newest first: each active day ranked as it stood at its end, and a quiet
	/// row for each game day of `listed` (the game days the Log lists pumps on) with no move done.
	pub(crate) fn chapters(&self, before: i64, listed: &[i64]) -> Vec<ChapterRow> {
		let mut out: Vec<(i64, ChapterRow)> = Vec::new();
		let (mut earlier, mut old): (i64, Option<(i64, i64)>) = (self.extra, None);
		for (&d, &moves) in self.days.range(..before) {
			let r = rank_from(&self.days, d, moves, earlier, self.old_best(old, d), self.tune);
			let row = ChapterRow { day: civil(d), chapter: Some(earlier + 1), moves, quality: r.quality, best: r.best };
			out.push((d, row));
			earlier += 1;
			old = better(old, (moves, d));
		}
		for &d in listed.iter().filter(|&&d| d < before && self.moves(d) == 0) {
			out.push((d, ChapterRow { day: civil(d), chapter: None, moves: 0, quality: None, best: false }));
		}
		out.sort_by_key(|&(d, _)| std::cmp::Reverse(d));
		out.into_iter().map(|(_, row)| row).collect()
	}

	/// Day `d`'s strip (rules 9-11).
	pub(crate) fn strip(&self, d: i64) -> Strip {
		let (mut bars, mut buckets, mut kinds) = (Vec::new(), [0; 4], [0; 4]);
		let mut done = Vec::new();
		for p in &self.pumps {
			let mut bar = [0; 4];
			let mut any = false;
			for m in p.moves.iter().filter(|m| self.day.index(m.ts) == d) {
				any = true;
				if let Some(k) = kind_of(self.f, m.mv) {
					bar[k] += 1;
				}
				done.push(m.mv);
			}
			if !any {
				continue;
			}
			// at its start's wall clock on the day it started; a pump begun the day before at the day's turn
			let (minute, bucket) = if self.day.index(p.id) == d {
				let (h, m) = self.day.wall(p.id);
				((h - self.day.offset_h()).rem_euclid(24) * 60 + m, Bucket::of(h))
			} else {
				(0, Bucket::Morning)
			};
			let count: i64 = p.moves.iter().filter(|m| self.day.index(m.ts) == d).count().try_into().unwrap_or(0);
			buckets[bucket.place()] += count;
			for (k, n) in bar.iter().enumerate() {
				kinds[k] += n;
			}
			bars.push(Bar { minute, bucket, kinds: bar });
		}
		let (most, some) = self.f.muscles(done);
		Strip { bars, buckets, kinds, most, some }
	}

	/// The moves done on the days `from..=to`, by time.
	fn sets(&self, from: i64, to: i64) -> impl Iterator<Item = &Done> {
		let (a, b) = (self.done.partition_point(|s| s.day < from), self.done.partition_point(|s| s.day <= to));
		self.done.get(a..b).unwrap_or_default().iter()
	}

	/// A move's record at the start of day `d` (rule 12's was): its best set before the day since its record was last
	/// cleared or its ladder started over before the day; none without one.
	fn record_before(&self, mv: u16, d: i64) -> Option<Set> {
		let ladder = self.f.tags(mv).and_then(|m| m.ladder);
		let cleared = self.cleared_before(d, |c| {
			(c.scope == clear::MOVE && c.id == mv)
				|| (c.scope == clear::LADDER && ladder.is_some_and(|l| c.id == u16::from(l)))
		});
		let epley = self.f.constants().load_epley;
		self.done
			.get(..self.done.partition_point(|s| s.day < d))
			.unwrap_or_default()
			.iter()
			.filter(|s| s.mv == mv && cleared.is_none_or(|c| s.ts > c))
			.filter_map(|s| s.set)
			.fold(None, |best: Option<Set>, s| if best.is_none_or(|b| s.beats(b, epley)) { Some(s) } else { best })
	}

	/// The records of the days `from..=to` (rule 12): one a move with a record row there, in the order of its first,
	/// with its best set there and its record before `from`.
	fn records(&self, from: i64, to: i64) -> Vec<RecordLine> {
		let epley = self.f.constants().load_epley;
		let mut order: Vec<u16> = Vec::new();
		for s in self.sets(from, to).filter(|s| s.record) {
			if !order.contains(&s.mv) {
				order.push(s.mv);
			}
		}
		order
			.into_iter()
			.filter_map(|mv| {
				let best = self.sets(from, to).filter(|s| s.mv == mv).filter_map(|s| s.set).fold(
					None,
					|best: Option<Set>, s| if best.is_none_or(|b| s.beats(b, epley)) { Some(s) } else { best },
				)?;
				Some(RecordLine { mv, best, was: self.record_before(mv, from), hold: self.f.hold(mv) })
			})
			.collect()
	}

	/// The ladder's start rung as the sheet shown names it: the highest rung its earned steps mark at or under the start
	/// (0); 0 without one.
	fn start(&self, ladder: u8) -> i64 {
		let Some(steps) = usize::from(ladder).checked_sub(1).and_then(|i| self.earned.get(i)) else { return 0 };
		steps.iter().skip(1).take_while(|&&day| day == 0).count().try_into().unwrap_or(0)
	}

	/// Whether a rung was reached before day `d`, a start-over included (it forgets none): the sheet shown dates it
	/// before the day (1, a day the save does not know, is before every day), or a climb before the day.
	fn reached_before(&self, ladder: u8, rung: u8, d: i64) -> bool {
		let dated = usize::from(ladder)
			.checked_sub(1)
			.and_then(|i| self.earned.get(i))
			.and_then(|steps| steps.get(usize::from(rung)))
			.is_some_and(|&day| day == 1 || (day > 1 && day < d));
		dated || self.climbs.iter().any(|c| c.ladder == ladder && c.rung == rung && self.day.index(c.ts) < d)
	}

	/// The time of the latest clear row before day `d` that `which` takes; none without one.
	fn cleared_before(&self, d: i64, which: impl Fn(&Cleared) -> bool) -> Option<i64> {
		self.clears.iter().filter(|c| self.day.index(c.ts) < d && which(c)).map(|c| c.ts).max()
	}

	/// The steps the days `from..=to` reached for the first time (rule 13), by time.
	fn unlocks(&self, from: i64, to: i64) -> Vec<Unlock> {
		let mut out: Vec<Unlock> = Vec::new();
		for &Climb { ts, ladder, rung } in &self.climbs {
			if !(from..=to).contains(&self.day.index(ts)) {
				continue;
			}
			let Some(mv) = self.f.anchor(ladder, rung) else { continue };
			if i64::from(rung) <= self.start(ladder)
				|| self.reached_before(ladder, rung, from)
				|| out.iter().any(|u| u.ladder == ladder && u.rung == rung)
			{
				continue;
			}
			out.push(Unlock { mv, ladder, rung, quality: self.f.rung_quality(ladder, i64::from(rung)) });
		}
		out
	}

	/// The days `from..=to`'s records, unlocks, knight and line (rules 12-14): the unlock of the highest quality (the
	/// later on a tie), else the move of the last record row, else the move done most (of two as many, the one done
	/// last).
	pub(crate) fn highlights(&self, from: i64, to: i64) -> Highlights {
		let records = self.records(from, to);
		let unlocks = self.unlocks(from, to);
		let top = unlocks.iter().fold(None, |top: Option<&Unlock>, u| match top {
			Some(t) if t.quality > u.quality => Some(t),
			_ => Some(u),
		});
		let (hero, highlight) = if let Some(u) = top {
			(Some(Hero { mv: u.mv, from: HeroFrom::Unlock }), Some(Highlight::Unlock { mv: u.mv, quality: u.quality }))
		} else if let Some(last) = self.sets(from, to).filter(|s| s.record).last() {
			let line = records.iter().find(|r| r.mv == last.mv);
			(
				Some(Hero { mv: last.mv, from: HeroFrom::Record }),
				line.map(|r| Highlight::Record { mv: r.mv, best: r.best, hold: r.hold }),
			)
		} else {
			// the count of each move and its last place; the most, the later on a tie
			let mut counts: HashMap<u16, (usize, usize)> = HashMap::new();
			for (i, s) in self.sets(from, to).enumerate() {
				let c = counts.entry(s.mv).or_insert((0, 0));
				*c = (c.0 + 1, i);
			}
			let most = counts.into_iter().max_by_key(|&(_, count_last)| count_last).map(|(mv, _)| mv);
			(most.map(|mv| Hero { mv, from: HeroFrom::Most }), None)
		};
		Highlights { records, unlocks, hero, highlight }
	}

	/// Day `d`'s Quest Bill: a line a move done that day, the moves of `marked` (its records' and unlocks') first.
	fn bill(&self, d: i64, marked: &HashSet<u16>) -> Vec<BillLine> {
		let mut lines: Vec<BillLine> = Vec::new();
		for s in self.sets(d, d) {
			let rep = self.f.rep(s.mv);
			let at = lines.iter().position(|l| l.mv == s.mv).unwrap_or_else(|| {
				lines.push(BillLine { mv: s.mv, count: 0, reps: None, seconds: (!rep).then_some(0), first: s.ts });
				lines.len() - 1
			});
			let Some(line) = lines.get_mut(at) else { continue };
			line.count += 1;
			if rep {
				// a set without a count adds none, and a line with none shows none
				if let Some(reps) = s.reps {
					line.reps = Some(line.reps.unwrap_or(0) + reps);
				}
			} else {
				line.seconds = line.seconds.map(|t| t + s.seconds);
			}
		}
		lines.sort_by_key(|l| (!marked.contains(&l.mv), std::cmp::Reverse(l.count), l.first, l.mv));
		lines
	}

	/// The day reel (rule 18; the phones' `SagaReel.plan`): the hero, then the Quest Bill's moves in its order (`bill`,
	/// as `bill` sorts it), each once, a move without its film on this computer (`filmed`) left out, the first
	/// `tune.moves` of them, the first the lead; none without one. Each move's shot is `move_ms` long, its words its
	/// bill line, or the day's highlight for a move with no bill line (a hero an unlock made of a move not done that
	/// day); the end, `end_ms` on the lead's film again, names that film: the day's highlight where the lead is the
	/// highlight's move (the hero, from an unlock or a record, with its film), else the lead's bill line on a day with
	/// a record or an unlock (its hero has no film here), else the lead's name (a hero from most). Its sounds as with
	/// Sound on (the maker, `desk::reel`, leaves them out with it off): `reveal` at the number's pop and, as the end
	/// begins, `fanfare` on a best day, else `record` on a day with a record or an unlock.
	pub(crate) fn reel(bill: &[BillLine], day: ReelDay, filmed: &dyn Fn(u16) -> bool, tune: ReelTune) -> Option<Reel> {
		let mut picked: Vec<u16> = Vec::new();
		for mv in day.hero.map(|h| h.mv).into_iter().chain(bill.iter().map(|l| l.mv)) {
			if picked.len() == tune.moves {
				break;
			}
			if !picked.contains(&mv) && filmed(mv) {
				picked.push(mv);
			}
		}
		let lead = *picked.first()?;
		let shots: Vec<Shot> = picked
			.iter()
			.zip(0_i64..)
			.map(|(&mv, k)| Shot {
				mv,
				from: k * tune.move_ms,
				words: if bill.iter().any(|l| l.mv == mv) { Words::Line } else { Words::Highlight },
			})
			.collect();
		let ends = i64::try_from(shots.len()).unwrap_or(i64::MAX).saturating_mul(tune.move_ms);
		let end = if day.highlight && day.hero.is_some_and(|h| h.mv == lead) {
			Words::Highlight
		} else if day.marked {
			Words::Line
		} else {
			Words::Name
		};
		let mut sounds = vec![Cue { sound: ReelSound::Reveal, at: tune.reveal_from_ms + tune.reveal_ms }];
		if day.best {
			sounds.push(Cue { sound: ReelSound::Fanfare, at: ends });
		} else if day.marked {
			sounds.push(Cue { sound: ReelSound::Record, at: ends });
		}
		Some(Reel {
			lead,
			moves: shots,
			end: Shot { mv: lead, from: ends, words: end },
			ms: ends + tune.end_ms,
			sounds,
		})
	}

	/// The first day of the game week holding day `d` (rule 17): the game day of the region's weekly reset `ws` plus a
	/// whole number of weeks, the latest on or before `d`; without one, the Monday on or before it.
	pub(crate) fn week_start(&self, d: i64, ws: Option<i64>) -> i64 {
		match ws {
			Some(ws) => {
				let first = self.day.index(ws);
				first + 7 * (d - first).div_euclid(7)
			}
			None => d - weekday(d),
		}
	}

	/// Day `d` as the Saga shows it, `today`'s index beside it, its week from `week_start` with the days a rest token kept
	/// (`rested`, game days), with what the sheets say of it, its pumps as the Log lists them, and its reel over the
	/// moves whose film this computer has (`filmed`), its sounds as with Sound on (the maker leaves them out with it
	/// off); none without the reel's times (`tune`: tokens that do not read).
	#[expect(clippy::too_many_arguments, reason = "the day's own facts and what the sheets and the films add")]
	pub(crate) fn view(
		&self,
		d: i64,
		today: i64,
		week_start: i64,
		told: Told,
		rested: &[i64],
		entries: Vec<Entry>,
		filmed: &dyn Fn(u16) -> bool,
		tune: Option<ReelTune>,
	) -> SagaDay {
		let Told { xp, level, streak, week_xp } = told;
		let moves = self.moves(d);
		let rank = self.rank(d);
		let Strip { bars, buckets, kinds, most, some } = self.strip(d);
		let h = self.highlights(d, d);
		let marked: HashSet<u16> = h.records.iter().map(|r| r.mv).chain(h.unlocks.iter().map(|u| u.mv)).collect();
		let week_end = week_start + 6;
		let week_days: Vec<WeekDay> =
			(week_start..=week_end).map(|w| WeekDay { day: civil(w), moves: self.moves(w) }).collect();
		let wh = self.highlights(week_start, week_end);
		let week = WeekView {
			start: civil(week_start),
			ringed: usize::try_from(d - week_start).unwrap_or(0),
			rested: (week_start..=week_end).zip(0..).filter(|(w, _)| rested.contains(w)).map(|(_, i)| i).collect(),
			old_best: rank.old_best.map(|(m, _)| m),
			moves: week_days.iter().map(|w| w.moves).sum(),
			active: week_days.iter().filter(|w| w.moves > 0).count().try_into().unwrap_or(0),
			// a record row a move a day, as each day's records tile counts them
			records: (week_start..=week_end).map(|w| self.records(w, w).len()).sum::<usize>().try_into().unwrap_or(0),
			xp: week_xp,
			hero: wh.hero,
			highlight: wh.highlight,
			days: week_days,
		};
		let bill = self.bill(d, &marked);
		let reel_day =
			ReelDay { hero: h.hero, highlight: h.highlight.is_some(), best: rank.best, marked: !marked.is_empty() };
		let reel = tune.and_then(|tune| Self::reel(&bill, reel_day, filmed, tune));
		SagaDay {
			day: civil(d),
			ago: today - d,
			chapter: rank.chapter,
			moves,
			quality: rank.quality,
			best: rank.best,
			old_best: rank.old_best.map(|(moves, day)| OldBest { moves, day: civil(day) }),
			pumps: bars.len().try_into().unwrap_or(0),
			bars,
			buckets,
			kinds,
			most,
			some,
			records: h.records,
			unlocks: h.unlocks,
			hero: h.hero,
			highlight: h.highlight,
			bill,
			reel,
			xp,
			level,
			streak,
			week,
			entries,
		}
	}
}

/// A climb: a rung row of an up, a plateau, a placement high or a check's lift (`cause::climbs`), not undone.
#[derive(Debug, Clone, Copy)]
struct Climb {
	ts: i64,
	ladder: u8,
	rung: u8,
}

/// A clear row: a move's record cleared or a ladder started over (`scope`, `codes::clear`), `id` the move or the
/// ladder.
#[derive(Debug, Clone, Copy)]
struct Cleared {
	ts: i64,
	scope: u16,
	id: u16,
}

/// The climbs not undone and the clear rows of both stores, by time (stable). An undo row (a corrected count's, the
/// game's `/qp undo` and Correct a count, and since slice 44 an app's Correct a count of an app set) reverses a rung row
/// of its own character on its ladder at another rung, and neither happened (DB.lua `pairing`): one naming that row by
/// its time in its quest (2026-10-19, the program's double-check) pairs with it alone, the newest of that time among the
/// rows of its key not paired and no undo, and reverses nothing where the rows lack it; one from before (quest 0) with
/// the latest row of its key, none once that latest was paired. An app's undo, character 0, pairs with the app's rung
/// rows, never with a game's, nor a game's undo with an app's. Each rung as the fold reads it (`Facts::row_rung`).
fn rungs(f: &Facts, rows: &[Row]) -> (Vec<Climb>, Vec<Cleared>) {
	// each rung row's rung as the fold reads it (a ladder that lost its first rung since, DB.lua `ns.ShiftRow`)
	let read: Vec<Row> = rows
		.iter()
		.filter(|r| r.kind == kind::RUNG || r.kind == kind::CLEAR)
		.map(|&r| {
			let mut read = r;
			if r.kind == kind::RUNG {
				read.rung = f.row_rung(r.ladder, r.rung, r.ts);
			}
			read
		})
		.collect();
	let mut ordered: Vec<&Row> = read.iter().collect();
	ordered.sort_by_key(|r| r.ts);
	let undo = i64::from(cause::UNDO);
	let mut undone: HashSet<usize> = HashSet::new();
	// each key's latest row, and the rows a named undo may pair with (not paired, no undo) in their order
	let mut last: HashMap<(u8, u8), usize> = HashMap::new();
	let mut open: HashMap<(u8, u8), Vec<usize>> = HashMap::new();
	for (i, r) in ordered.iter().enumerate().filter(|(_, r)| r.kind == kind::RUNG) {
		let key = (r.ladder, r.char);
		let named = r.quest;
		let rows = open.entry(key).or_default();
		if r.reps != undo {
			last.insert(key, i);
			rows.push(i);
			continue;
		}
		let prev = if named > 0 {
			rows.iter().rev().copied().find(|&k| ordered[k].ts == i64::from(named) && ordered[k].rung != r.rung)
		} else {
			last.get(&key).copied().filter(|&k| ordered[k].rung != r.rung)
		};
		match prev {
			Some(k) => {
				undone.extend([k, i]);
				rows.retain(|&o| o != k);
				if last.get(&key) == Some(&k) {
					last.remove(&key);
				}
			}
			None if named > 0 => {
				undone.insert(i);
			}
			None => {
				last.insert(key, i);
			}
		}
	}
	let climb = |reps: i64| u16::try_from(reps).is_ok_and(cause::climbs);
	let mut climbs = Vec::new();
	let mut clears = Vec::new();
	for (i, r) in ordered.iter().enumerate() {
		if r.kind == kind::RUNG && climb(r.reps) && !undone.contains(&i) {
			climbs.push(Climb { ts: r.ts, ladder: r.ladder, rung: r.rung });
		} else if r.kind == kind::CLEAR {
			let scope = u16::try_from(r.reps).unwrap_or(0);
			let id = if scope == clear::LADDER { u16::from(r.ladder) } else { r.mv };
			clears.push(Cleared { ts: r.ts, scope, id });
		}
	}
	(climbs, clears)
}
