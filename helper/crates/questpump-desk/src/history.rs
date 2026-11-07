//! The history the Book and the Log read on the computer (docs/plan.md slice 35, "The view rules the phones keep in
//! their cores": the Log's list in Rust), ported from `QuestPumpCore`'s `HistoryRow`, `HistoryPump`, `LogPumps`,
//! `HistorySeries`, `GameCalendar` and `GameDay` (the Android core's twins, the same names and rules) over the rows the
//! helper holds: the game's rows and the catch-ups (`rows.jsonl`, as `GET /v1/history` serves them, each pump row with
//! its heart-rate proof), every device's app rows (`app.jsonl`: the phones', the watches' and this computer's own) and
//! the game's sheets the helper read (`sheets.jsonl`). The computer keeps no pump of its own beside its rows, so every
//! app pump is its rows alone, grouped by the device that ran it, as the phones group the desktop app's
//! (`HistoryPump.desktop`); the phones' Health minutes stay theirs. contract/fixtures/history.json pins the grouping, the
//! days and the weeks.

use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, HashSet};

use questpump_core::codes::{answer, by, flags, kind, outcome};
use questpump_core::protocol::{ExpeditionStage, PumpKind, RowRecord};
use serde::{Deserialize, Serialize};

use super::facts::facts;

const DAY_S: i64 = 86_400;
/// A week of the region's weekly resets, in seconds.
pub const WEEK_S: i64 = 604_800;
/// The zone's offset is read once for each quarter of an hour (every clock change in use falls on one's edge).
const ZONE_BUCKET_S: i64 = 900;
/// The unix epoch as a Julian day.
const EPOCH_JULIAN: i64 = 2_440_588;

/// `a` over `b` rounded down, never toward zero.
fn floor_div(a: i64, b: i64) -> i64 {
	a.div_euclid(b)
}

// Game days ------------------------------------------------------------------------------------------------------------

/// The computer's game days (the phones' `GameDay`, ns.day): a game day is the local date `offset` earlier, so it turns
/// at 04:00 by the clock (03:00 or 05:00 on a night the clocks change). The zone's offset is the platform's at each
/// instant (`Platform::zone`), cached for the build that reads it.
pub struct GameDay {
	offset: i64,
	/// The zone's offset from UTC at a unix second, in seconds east.
	at: fn(i64) -> i64,
	zone: RefCell<HashMap<i64, i64>>,
}

impl GameDay {
	/// Game days that turn `offset_h` hours after local midnight (library.json's `day_offset_h`) in the zone `zone`
	/// (`Platform::zone`).
	pub fn new(offset_h: i64, zone: fn(i64) -> i64) -> Self {
		Self { offset: offset_h * 3_600, at: zone, zone: RefCell::new(HashMap::new()) }
	}

	/// The zone's offset at `t`: a quarter of an hour's own when it holds over the whole quarter, else `t`'s.
	fn offset_at(&self, t: i64) -> i64 {
		let bucket = floor_div(t, ZONE_BUCKET_S);
		if let Some(&o) = self.zone.borrow().get(&bucket) {
			return o;
		}
		let (from, to) = ((self.at)(bucket * ZONE_BUCKET_S), (self.at)(bucket * ZONE_BUCKET_S + ZONE_BUCKET_S - 1));
		if from == to {
			self.zone.borrow_mut().insert(bucket, from);
			return from;
		}
		(self.at)(t)
	}

	/// The local date (days since 1970-01-01) of the game day `ts` falls in.
	pub fn index(&self, ts: i64) -> i64 {
		let t = ts - self.offset;
		floor_div(t + self.offset_at(t), DAY_S)
	}

	/// The game day `ts` falls in, as "yyyy-MM-dd".
	pub fn name(&self, ts: i64) -> String {
		civil(self.index(ts))
	}

	/// The hour the game days turn at (library.json's `day_offset_h`).
	pub fn offset_h(&self) -> i64 {
		self.offset / 3_600
	}

	/// The local wall clock's hour and minute at `ts` (seconds dropped).
	pub fn wall(&self, ts: i64) -> (i64, i64) {
		let into = (ts + self.offset_at(ts)).rem_euclid(DAY_S);
		(into / 3_600, into % 3_600 / 60)
	}

	/// The first instant whose local reading is `seconds` into the local date `date`: the earlier one where the clocks
	/// went back across it, the end of the skipped hour where they went forward across it (Foundation's
	/// `date(bySettingHour:)` with its default policies, `java.time`'s `atStartOfDay` alike).
	fn at_wall(&self, date: i64, seconds: i64) -> i64 {
		let wall = date * DAY_S + seconds;
		let (before, after) = (self.offset_at(wall - DAY_S), self.offset_at(wall + DAY_S));
		let fits = |o: i64| {
			let t = wall - o;
			(self.offset_at(t) == o).then_some(t)
		};
		match (fits(before), fits(after)) {
			(Some(a), Some(b)) => a.min(b),
			(Some(t), None) | (None, Some(t)) => t,
			(None, None) => {
				let (mut lo, mut hi) = (wall - after.max(before), wall - after.min(before));
				while hi - lo > 1 {
					let mid = lo + (hi - lo) / 2;
					if self.offset_at(mid) == after {
						hi = mid;
					} else {
						lo = mid;
					}
				}
				hi
			}
		}
	}

	/// The game day's turn, the offset hour on the date of the game day holding `ts`, as a calendar day shows it (where
	/// a paired player's weeks start).
	pub fn start(&self, ts: i64) -> i64 {
		self.at_wall(self.index(ts), self.offset)
	}

	/// The middle of the game day `n` days after the one `ts` falls in (ns.DayMid): stepped from the middle, a day of
	/// 23 or 25 hours never lands on the wrong one.
	pub fn mid(&self, ts: i64, n: i64) -> i64 {
		let t = ts - self.offset;
		let into = (t + self.offset_at(t)).rem_euclid(DAY_S);
		ts + DAY_S / 2 - into + n * DAY_S
	}

	/// The seven game days from the one `start` falls in, as "yyyy-MM-dd".
	pub fn days_from(&self, start: i64) -> Vec<String> {
		(0..7).map(|n| self.name(self.mid(start, n))).collect()
	}

	/// The offset hour on the Monday of the game day holding `ts`: where the weeks start outside the game.
	pub fn monday(&self, ts: i64) -> i64 {
		let date = self.index(ts);
		self.at_wall(date - weekday(date), self.offset)
	}
}

/// A local date (days since 1970-01-01) as "yyyy-MM-dd".
pub fn civil(date: i64) -> String {
	i32::try_from(date + EPOCH_JULIAN)
		.ok()
		.and_then(|j| time::Date::from_julian_day(j).ok())
		.map_or_else(String::new, |d| format!("{:04}-{:02}-{:02}", d.year(), u8::from(d.month()), d.day()))
}

/// A "yyyy-MM-dd" date as a local date (days since 1970-01-01); none for another text.
pub fn date_index(day: &str) -> Option<i64> {
	let mut parts = day.splitn(3, '-').map(str::parse::<i32>);
	let (Some(Ok(y)), Some(Ok(m)), Some(Ok(d))) = (parts.next(), parts.next(), parts.next()) else { return None };
	let month = time::Month::try_from(u8::try_from(m).ok()?).ok()?;
	let date = time::Date::from_calendar_date(y, month, u8::try_from(d).ok()?).ok()?;
	Some(i64::from(date.to_julian_day()) - EPOCH_JULIAN)
}

/// A local date's weekday, 0 Monday … 6 Sunday.
#[must_use]
pub fn weekday(date: i64) -> i64 {
	(date + 3).rem_euclid(7)
}

/// Game weeks from an anchor, earlier weeks at negative indices (the phones' `HistorySeries.Weeks`): with `days`,
/// seven game days at a time from the one the anchor falls in, each from its first day's turn; without, fixed weeks of
/// the region's weekly reset.
pub struct Weeks<'a> {
	anchor: i64,
	days: Option<&'a GameDay>,
}

impl Weeks<'_> {
	fn index(&self, t: i64) -> i64 {
		match self.days {
			Some(d) => floor_div(d.index(t) - d.index(self.anchor), 7),
			None => floor_div(t - self.anchor, WEEK_S),
		}
	}

	fn start(&self, index: i64) -> i64 {
		match self.days {
			Some(d) => d.start(d.mid(self.anchor, index * 7)),
			None => self.anchor + index * WEEK_S,
		}
	}
}

/// The start of the game week `back` weeks before the one holding `ts` (`HistorySeries.weekStart`): seven game days
/// from the one the region's weekly reset `ws` falls in, at that day's turn; without one, outside the game, Monday's.
pub fn week_start(ts: i64, ws: Option<i64>, days: &GameDay, back: i64) -> i64 {
	let weeks = Weeks { anchor: ws.unwrap_or_else(|| days.monday(ts)), days: Some(days) };
	weeks.start(weeks.index(ts) - back)
}

// Rows ---------------------------------------------------------------------------------------------------------------

/// A row as the Log and the charts read it (the phones' `HistoryRow`): its grammar's fields and, on a pump row, whether
/// its heart-rate proof says proven.
#[derive(Debug, Clone, Copy)]
pub struct Row {
	pub ts: i64,
	/// The character the row names (0 an app's).
	pub char: u8,
	/// The quest; an undo rung row's (since 2026-10-19) the time of the row it reverses, 0 on one from before.
	pub quest: u32,
	slot: u8,
	/// The ladder and the rung: a move row's where it was dosed, a rung row's new rung.
	pub ladder: u8,
	pub rung: u8,
	pub mv: u16,
	pub kind: u8,
	/// A rung row's cause, a clear row's scope (`codes::kind::CLEAR`), else the move's count.
	pub reps: i64,
	seconds: i64,
	outcome: u8,
	pub flags: u16,
	tier: u8,
	/// A pump row whose heart-rate proof says proven.
	pub proven: bool,
	/// The row locks pumps for feeling unwell (`questpump_core::row::Row::locks`, the phones' `UnwellLock.locks`).
	unwell: bool,
	/// The offered row's kind of a pump this computer ran where it changes the title (a move the player started from
	/// this computer's Book, which the Log titles as asked; an expedition's round); the phones keep their own pumps'
	/// kinds in their own history.
	pump_kind: Option<PumpKind>,
}

impl Row {
	/// A kept row, its proof not read.
	#[must_use]
	pub fn of(r: &RowRecord) -> Self {
		Self {
			ts: i64::from(r.row_ts),
			char: r.char,
			quest: r.quest_id,
			slot: r.slot,
			ladder: r.ladder,
			rung: r.rung,
			mv: r.move_id,
			kind: r.kind,
			reps: i64::from(r.reps),
			seconds: i64::from(r.seconds),
			outcome: r.outcome,
			flags: r.flags,
			tier: r.tier,
			proven: false,
			unwell: r.row().locks(),
			pump_kind: r.pump_kind,
		}
	}

	fn has(&self, bit: u16) -> bool {
		self.flags & bit != 0
	}

	/// A move done: owed, banked and paid rows are none.
	#[must_use]
	pub fn performed(&self) -> bool {
		kind::is_move(self.kind) && outcome::is_performed(self.outcome)
	}

	/// A pump the player was asked for: its pump row (a quest's or a level-up's) with the outcome offered.
	#[must_use]
	pub fn offered_pump(&self) -> bool {
		kind::is_pump(self.kind) && self.outcome == outcome::OFFERED
	}

	/// A catch-up's row: the watch, the phone or the computer ran it (a device's bit without the app's).
	#[must_use]
	pub fn by_device(&self) -> bool {
		(self.has(flags::WATCH) || self.has(flags::PHONE) || self.has(flags::COMPUTER)) && !self.has(flags::APP)
	}

	/// A beaten record no correction names.
	fn record(&self, read: &Read) -> bool {
		self.outcome == outcome::BEAT && !read.corrected
	}
}

/// A row as read: its correction applied, whether it holds a typed count, whether a correction named it, and the weight
/// a set of a loaded move was done with (tenths of a kg).
#[derive(Debug, Clone, Copy)]
pub struct Read {
	pub row: Row,
	pub typed: bool,
	corrected: bool,
	load: Option<u32>,
}

/// The set a correction or a load row names: its row's time, quest, slot and move, as stored.
type SetAt = (i64, u32, u8, u16);

fn set_at(r: &Row) -> SetAt {
	(r.ts, r.quest, r.slot, r.mv)
}

/// The corrections and the weights the reads apply (slice 44, 2026-10-18), one map over every store's rows, as every
/// fold reads them (DB.lua `corrections` and `loads` by the round, `ns.CorrectionBeats`): of the correction rows naming
/// one set, or its load rows, the one of the highest round (its outcome) wins; of two of one round above 0 the smaller
/// value (a correction's reps, then its seconds; a load row's weight), of two alike the one met first; of round 0
/// (written before the rounds: `/qp undo`'s, a set's own load row) the last met, the stores in the order given (the
/// game's rows, then the app rows) and each in its own (contract/fixtures/history.json). So a set reads one correction
/// whichever store holds it, as the game, the phones and this computer's engine read it.
#[derive(Debug, Default)]
pub struct Fixes {
	fixes: HashMap<SetAt, Row>,
	loads: HashMap<SetAt, Row>,
}

impl Fixes {
	/// The winning correction and load row of each set among `stores`, in their order.
	#[must_use]
	pub fn of(stores: &[&[Row]]) -> Self {
		let mut out = Self::default();
		for r in stores.iter().flat_map(|rows| rows.iter()) {
			let held = match r.kind {
				kind::CORRECTION => &mut out.fixes,
				kind::LOAD => &mut out.loads,
				_ => continue,
			};
			let won = held.get(&set_at(r)).is_none_or(|b| beats(r, b));
			if won {
				held.insert(set_at(r), *r);
			}
		}
		out
	}
}

/// Whether correction (or load) row `a` takes the place of `b`, the one held (DB.lua `ns.CorrectionBeats`).
fn beats(a: &Row, b: &Row) -> bool {
	if a.outcome != b.outcome {
		return a.outcome > b.outcome;
	}
	if a.outcome == 0 {
		return true;
	}
	if a.reps != b.reps {
		return a.reps < b.reps;
	}
	a.seconds < b.seconds
}

/// The rows by time (rows of one time keep their given order), each move done with the correction `fixes` holds for
/// it applied: a hold's seconds, a pose's answer or another move's reps (the other field kept), which then count as
/// typed, its outcome the one it was written with, but for a count of 0 (a hold's 0 s), which reads as typing 0 does,
/// done without a count: no typed attempt and no count (slice 44's review, 2026-10-18; DB.lua `correct`); never a timed
/// cardio row (a cardio row without the counted bit) or a set stopped short, which no correction changes (DB.lua
/// `fixOf`); a capped max set (`capped`) keeps its stored top for a correction at or over it; and with the weight its
/// load row there says. A correction and a load row are no move: they come through as read, which every list leaves out
/// (`Row::performed`).
#[must_use]
pub fn read(rows: &[Row], fixes: &Fixes) -> Vec<Read> {
	let mut ordered: Vec<(usize, &Row)> = rows.iter().enumerate().collect();
	ordered.sort_by_key(|&(i, r)| (r.ts, i));
	ordered
		.into_iter()
		.map(|(_, row)| {
			let mut r = *row;
			let load = fixes
				.loads
				.get(&set_at(&r))
				.filter(|_| r.performed())
				.and_then(|l| u32::try_from(l.reps).ok())
				.filter(|&l| l > 0);
			let timed = r.kind == kind::CARDIO && !r.has(flags::COUNTED);
			let takes = r.performed() && !timed && r.outcome != outcome::STOPPED;
			match fixes.fixes.get(&set_at(&r)).filter(|_| takes) {
				Some(fix) => {
					let hold = r.kind == kind::HOLD;
					let (stored, typed) = if hold { (r.seconds, fix.seconds) } else { (r.reps, fix.reps) };
					let value = if typed >= stored && capped(&r, stored) { stored } else { typed };
					if hold {
						r.seconds = value;
					} else {
						r.reps = value;
					}
					if value == 0 {
						r.outcome = outcome::DONE;
					}
					Read { row: r, typed: value != 0, corrected: true, load }
				}
				None => Read { row: r, typed: outcome::is_typed(r.outcome), corrected: false, load },
			}
		})
		.collect()
}

/// Whether censored row `r`, its stored count `stored` (a hold's seconds, any other kind's reps), is a capped max set
/// (contract/fixtures/history.json's censored rule; slice 44's review, 2026-10-18): stored at its window's top
/// (`Facts::window_top`, contract/library.json), where the game's `ns.Judge` ends a max set. Any other censored row is
/// a long set (`ns.Classify`'s `censor_s`), which takes its correction whatever it is. The fold (DB.lua `ns.CappedSet`)
/// also asks that the move held no record before the set or its ladder was being placed again, which only a fold of
/// the whole save can tell: a long set stopped exactly at its top is the one censored row read otherwise here.
fn capped(r: &Row, stored: i64) -> bool {
	r.outcome == outcome::CENSORED && facts().window_top(r.ladder, r.rung, r.mv) == Some(stored)
}

/// Where a pump was done (the phones' `PumpSource`), by its pump row's device bits: the app bit makes an app pump (the
/// watch's, the computer's, else a browser's where its answer bits say so, `by::BROWSER`, else the phone's); without it
/// a device's bit marks a catch-up; none, the game's.
#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Source {
	Game,
	CatchUp,
	Phone,
	Watch,
	Computer,
	Web,
}

impl Source {
	fn of(flags: u16) -> Self {
		let (watch, computer) = (flags & flags::WATCH != 0, flags & flags::COMPUTER != 0);
		if flags & flags::APP != 0 {
			return if watch {
				Self::Watch
			} else if computer {
				Self::Computer
			} else if flags::answer(flags) == by::BROWSER {
				Self::Web
			} else {
				Self::Phone
			};
		}
		if watch || computer || flags & flags::PHONE != 0 { Self::CatchUp } else { Self::Game }
	}

	fn app(self) -> bool {
		matches!(self, Self::Phone | Self::Watch | Self::Computer | Self::Web)
	}
}

/// A pump of the game's rows the app rows show already: an app pump the save took in from its mirror (the Log, the
/// Saga and Pump home's today show it once, from the app rows).
fn held_twice(p: &Pump, app_ids: &HashSet<i64>) -> bool {
	p.source.app() && app_ids.contains(&p.id)
}

/// The pumps the Log shows, every one once: every device's app pumps from their rows (`app`), then the game's pumps and
/// catch-ups the rows hold (`game`) but those the app rows show.
#[must_use]
pub fn shown<'a>(game: &'a [Pump], app: &'a [Pump]) -> Vec<&'a Pump> {
	let app_ids: HashSet<i64> = app.iter().map(|p| p.id).collect();
	app.iter().chain(game.iter().filter(|p| !held_twice(p, &app_ids))).collect()
}

/// A move of a pump (the phones' `HistoryPump.Move`): its row's own time, its count (the typed count of a rep move, a
/// counted set's count above 0 whatever its outcome, as corrected; none for a move done by its seconds or a counted set
/// with none) with the weight it was done with (tenths of a kg), whether that count is a typed attempt (outcome typed,
/// or corrected: the Saga's records read only these, rule 12), its seconds (a hold's corrected), whether it was a
/// record, and whether it is a counted set and its answer (2026-10-19, the program's double-check: the Log words it as
/// the game's Log does, its count or Not counted, then "form slipped" or, since slice 53, "not all out").
#[derive(Debug, Clone, Copy)]
pub struct Move {
	pub ts: i64,
	pub mv: u16,
	pub reps: Option<i64>,
	pub load: Option<u32>,
	pub typed: bool,
	pub seconds: i64,
	pub record: bool,
	pub counted: bool,
	/// A counted set's answer to "All out and clean?" (`codes::answer`: 0 Yes, to the end, 1 Form slipped, 2 Not all
	/// out since slice 53).
	pub answer: u8,
}

/// A pump from the rows (the phones' `HistoryPump`): its pump row and the move rows that follow it.
#[expect(clippy::struct_excessive_bools, reason = "the phones' HistoryPump: one yes or no of its pump row a field")]
#[derive(Debug, Clone)]
pub struct Pump {
	/// The pump row's own time.
	pub id: i64,
	level_up: bool,
	tier: u8,
	from_device: bool,
	/// The moves its pump row names (a level-up's: its move rows).
	count: i64,
	pub moves: Vec<Move>,
	proven: bool,
	source: Source,
	watch: bool,
	computer: bool,
	/// Banked in the game, for any reason but felt unwell.
	banked: bool,
	/// A felt-unwell row follows its pump row before the next one: a pump stopped for feeling unwell, kept with no move
	/// done as the phones keep their own.
	unwell: bool,
	/// Its offered row's `pump_kind`.
	kind: Option<PumpKind>,
}

impl Pump {
	/// The moves done in it.
	#[must_use]
	pub fn moves_done(&self) -> usize {
		self.moves.len()
	}

	/// One of the game's own pumps, a quest's or a level-up's (its pump row with no device's bit): no device's
	/// catch-up, nor an app pump the save took in from its mirror, whose moves Pump home counts under its own game
	/// (World of Warcraft's moves, `desk::recent`).
	#[must_use]
	pub fn of_game(&self) -> bool {
		self.source == Source::Game
	}
}

/// Rows (in any order) grouped into pumps, newest first, read with `fixes` (every store's corrections): a row before any
/// pump row belongs to none, a banked pump row marks the pump before it banked and a felt-unwell row unwell (the phones'
/// `HistoryPump.group`).
#[must_use]
pub fn group(rows: &[Row], fixes: &Fixes) -> Vec<Pump> {
	let mut pumps: Vec<Pump> = Vec::new();
	for read in read(rows, fixes) {
		let r = read.row;
		if r.unwell
			&& let Some(p) = pumps.last_mut()
		{
			p.unwell = true;
		}
		if r.offered_pump() {
			let level_up = r.kind == kind::DING;
			pumps.push(Pump {
				id: r.ts,
				level_up,
				tier: r.tier,
				from_device: r.by_device(),
				count: if level_up { 0 } else { r.reps },
				moves: Vec::new(),
				proven: r.proven,
				source: Source::of(r.flags),
				watch: r.has(flags::WATCH),
				computer: r.has(flags::COMPUTER),
				banked: false,
				unwell: false,
				kind: r.pump_kind,
			});
		} else if r.performed()
			&& let Some(p) = pumps.last_mut()
		{
			let counted = r.kind == kind::CARDIO && r.has(flags::COUNTED);
			let reps = if counted {
				Some(r.reps).filter(|&n| n > 0)
			} else {
				(r.kind == kind::REP && read.typed).then_some(r.reps)
			};
			p.moves.push(Move {
				ts: r.ts,
				mv: r.mv,
				reps,
				load: read.load.filter(|_| reps.is_some()),
				typed: read.typed,
				seconds: r.seconds,
				record: r.record(&read),
				counted,
				// a counted set's answer in bits 14-15 (a steady move's there is its pace)
				answer: if counted { flags::answer(r.flags) } else { answer::YES },
			});
			if p.level_up {
				p.count += 1;
			}
		} else if kind::is_pump(r.kind)
			&& r.outcome == outcome::BANKED
			&& r.flags & flags::REASON_MASK != flags::UNWELL
			&& let Some(p) = pumps.last_mut()
		{
			p.banked = true;
		}
	}
	pumps.reverse();
	pumps
}

/// Every device's app pumps from the app rows (the phones', the watches' and this computer's), each device's rows
/// grouped apart, newest first, counted as the phones count the desktop app's (`HistoryPump.desktop`) and their own:
/// one with a move done, or stopped for feeling unwell; one left before its first move is none; read with `fixes`
/// (every store's corrections). The one rule wherever the computer counts app pumps.
#[must_use]
pub fn app_pumps(app: &[Row], fixes: &Fixes) -> Vec<Pump> {
	let mut all: Vec<Pump> = [flags::PHONE, flags::WATCH, flags::COMPUTER]
		.into_iter()
		.flat_map(|bit| group(&app.iter().filter(|r| r.has(bit)).copied().collect::<Vec<_>>(), fixes))
		.filter(|p| !p.moves.is_empty() || p.unwell)
		.collect();
	all.sort_by_key(|p| std::cmp::Reverse(p.id));
	all
}

// The Log's pumps ------------------------------------------------------------------------------------------------------

/// What a pump was: its title and its glyph say it (`Move`: a move the player started from this computer's Book,
/// titled as asked as both phones title theirs, docs/plan.md slice 38; `Expedition`: an expedition's round, titled by
/// its stage and number, slice 51).
#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum EntryKind {
	Game,
	LevelUp,
	CatchUp,
	App,
	Move,
	Expedition,
}

/// How a game pump ended with no move done.
#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum NoMove {
	Banked,
	Other,
}

/// A move as the Log shows it: its count (`Move::reps`) with the weight it was done with (tenths of a kg), its seconds,
/// whether it beat its record, whether it is a hold, and whether it is a counted set and its answer (its count or Not
/// counted, then "form slipped" or "not all out", as the game's Log words it).
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct EntryMove {
	#[serde(rename = "move")]
	mv: u16,
	#[serde(skip_serializing_if = "Option::is_none")]
	reps: Option<i64>,
	#[serde(skip_serializing_if = "Option::is_none")]
	load: Option<u32>,
	seconds: i64,
	record: bool,
	hold: bool,
	counted: bool,
	answer: u8,
}

/// A pump as the Log lists it (the phones' `LogPumps.Entry`): its id once in the list, when it was done and the game day
/// it began on (its key and how many game days before today's), its kind, its quest's size, the moves its title counts,
/// where it was done (and on the watch or a computer where a catch-up ran there), its moves, its seconds, the
/// heart-rate check, an app pump stopped for feeling unwell, the XP its sheet paid, and how a game pump with no move done
/// ended.
#[expect(clippy::struct_excessive_bools, reason = "the phones' LogPumps.Entry: one yes or no of the pump a field")]
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
	pub id: String,
	pub at: i64,
	day: String,
	ago: i64,
	kind: EntryKind,
	tier: u8,
	count: i64,
	source: Source,
	watch: bool,
	computer: bool,
	moves: Vec<EntryMove>,
	seconds: i64,
	proven: bool,
	unwell: bool,
	#[serde(skip_serializing_if = "Option::is_none")]
	xp: Option<i64>,
	#[serde(skip_serializing_if = "Option::is_none")]
	no_move: Option<NoMove>,
	/// An expedition round's stage and its number from 1 (`Expedition`).
	#[serde(skip_serializing_if = "Option::is_none")]
	stage: Option<ExpeditionStage>,
	#[serde(skip_serializing_if = "Option::is_none")]
	round: Option<u8>,
}

/// A game day of the Log: its key, how many game days before today's it is, and its pumps, newest first: those begun on
/// it and those with a move done on it (a pump across the day's turn under each day it holds a move on, the iPhone's
/// `Saga.book`).
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Day {
	pub id: String,
	pub ago: i64,
	pub entries: Vec<Entry>,
}

/// The XP the rows show by a pump's offered time: the game's sheets' (`named`), and the app's own fold's, read for an
/// app pump only while no game sheet came (none else).
pub struct Xp {
	pub game: HashMap<i64, i64>,
	pub app: Option<HashMap<i64, i64>>,
}

/// The Log's days, newest first (the phones' `LogPumps.build` on the computer, which keeps no pump beside its rows):
/// the game's pumps and catch-ups the rows hold (`game`), every device's app pumps from their rows (`app`, an app pump
/// the game's rows hold too shown once), each with the XP its sheet paid; a game pump with no move done says how it
/// ended; every id once; holds by `hold`; days by `day` before `now`.
pub fn log_days(game: &[Pump], app: &[Pump], xp: &Xp, hold: &dyn Fn(u16) -> bool, day: &GameDay, now: i64) -> Vec<Day> {
	let paid = |at: i64, app: bool| {
		xp.game
			.get(&at)
			.copied()
			.or_else(|| if app { xp.app.as_ref()?.get(&at).copied() } else { None })
			.filter(|&x| x > 0)
	};
	let seconds = |p: &Pump| -> i64 { p.moves.iter().map(|m| m.seconds).sum() };
	let today = day.index(now);
	let later = |p: &Pump, start: i64| later_days(p, start, day);
	let mut entries: Vec<(Entry, Vec<i64>)> = Vec::new();
	let app_ids: HashSet<i64> = app.iter().map(|p| p.id).collect();
	for p in app {
		let start = day.index(p.id);
		let (kind, stage, round) = match p.kind {
			Some(PumpKind::Move) => (EntryKind::Move, None, None),
			Some(PumpKind::Expedition { stage, round }) => (EntryKind::Expedition, Some(stage), round),
			None => (EntryKind::App, None, None),
		};
		let entry = Entry {
			id: row_id(p.id),
			at: p.id,
			day: civil(start),
			ago: today - start,
			kind,
			tier: 0,
			count: i64::try_from(p.moves.len()).unwrap_or(i64::MAX),
			source: p.source,
			watch: false,
			computer: false,
			moves: entry_moves(p, hold),
			seconds: seconds(p),
			proven: p.proven,
			unwell: p.unwell,
			xp: paid(p.id, true),
			no_move: None,
			stage,
			round,
		};
		entries.push((entry, later(p, start)));
	}
	for p in game {
		if held_twice(p, &app_ids) {
			continue; // the app rows show it
		}
		let app = p.source.app();
		let kind = if p.level_up {
			EntryKind::LevelUp
		} else if app {
			EntryKind::App
		} else if p.from_device {
			EntryKind::CatchUp
		} else {
			EntryKind::Game
		};
		let catch_up = p.source == Source::CatchUp;
		let start = day.index(p.id);
		let entry = Entry {
			id: row_id(p.id),
			at: p.id,
			day: civil(start),
			ago: today - start,
			kind,
			tier: p.tier,
			count: p.count,
			source: p.source,
			watch: p.watch && catch_up,
			computer: p.computer && catch_up,
			moves: entry_moves(p, hold),
			seconds: seconds(p),
			proven: p.proven,
			// a pump of the game's rows says felt unwell only on a phone that kept it (`LogPumps.join`)
			unwell: false,
			xp: paid(p.id, app),
			no_move: (kind == EntryKind::Game && p.moves.is_empty()).then_some(if p.banked {
				NoMove::Banked
			} else {
				NoMove::Other
			}),
			stage: None,
			round: None,
		};
		entries.push((entry, later(p, start)));
	}
	// every id once, in the order built: a pump offered in the same second as another shares its rows' time
	let mut ids = HashSet::new();
	for (e, _) in &mut entries {
		let (base, mut n) = (e.id.clone(), 1);
		while !ids.insert(e.id.clone()) {
			n += 1;
			e.id = format!("{base}-{n}");
		}
	}
	listed(entries, today)
}

/// A pump's moves as the Log shows them, holds by `hold`.
fn entry_moves(p: &Pump, hold: &dyn Fn(u16) -> bool) -> Vec<EntryMove> {
	p.moves
		.iter()
		.map(|m| EntryMove {
			mv: m.mv,
			reps: m.reps,
			load: m.load,
			seconds: m.seconds,
			record: m.record,
			hold: hold(m.mv),
			counted: m.counted,
			answer: m.answer,
		})
		.collect()
}

/// The game days a pump is listed on after the one it began on (`start`): each other one a move of it was done on.
fn later_days(p: &Pump, start: i64, day: &GameDay) -> Vec<i64> {
	let mut days: Vec<i64> = Vec::new();
	for d in p.moves.iter().map(|m| day.index(m.ts)) {
		if d != start && !days.contains(&d) {
			days.push(d);
		}
	}
	days
}

/// The Log's days, newest first, from the pumps with the later days each is listed on: each day's pumps newest first,
/// a pump under the day it began on and a copy of it under each later one.
fn listed(mut entries: Vec<(Entry, Vec<i64>)>, today: i64) -> Vec<Day> {
	entries.sort_by(|(a, _), (b, _)| (b.at, &b.id).cmp(&(a.at, &a.id)));
	let mut by_day: BTreeMap<i64, Vec<Entry>> = BTreeMap::new();
	for (e, later) in entries {
		for d in later {
			by_day.entry(d).or_default().push(e.clone());
		}
		by_day.entry(today - e.ago).or_default().push(e);
	}
	by_day.into_iter().rev().map(|(d, entries)| Day { id: civil(d), ago: today - d, entries }).collect()
}

/// The id of the pump the rows hold under its offered time.
fn row_id(at: i64) -> String {
	format!("row-{at}")
}

// The weeks and the charts' series -----------------------------------------------------------------------------------

/// A sheet the game folded, as the charts, the Log and the Saga read it: the character the helper logged it under, when
/// it was folded, the region's weekly reset it names, its ladders by id (v 4 on: an older sheet laid them out
/// otherwise, and no player kept one past slice 21), the XP its log names by pump (`lx`), its level with the XP into
/// it and the level's need, its whole XP and its streak (a day's level from an older addon and its streak:
/// `desk::log`), and the save's all-time active days and two best days (`ad`, `bd`: the Saga's anchor, rules 3 and 4).
#[derive(Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct Sheet {
	#[serde(skip)]
	pub char: u8,
	pub v: i64,
	pub at: i64,
	pub ws: Option<i64>,
	pub lad: Vec<Vec<i64>>,
	pub lx: Vec<Vec<i64>>,
	pub lvl: i64,
	pub xp: i64,
	pub into: i64,
	pub need: i64,
	pub streak: i64,
	pub ad: Option<i64>,
	pub bd: Vec<(String, i64)>,
}

impl Sheet {
	/// The game's two best days the sheet names (`bd`: game day, moves), the most first; a day that does not read is
	/// left out.
	#[must_use]
	pub fn best_days(&self) -> Vec<(i64, i64)> {
		self.bd.iter().filter_map(|(day, moves)| Some((date_index(day)?, *moves))).collect()
	}
}

/// Each pump's XP as the newest sheet that names it gives it, the sheets by the time they were folded.
#[must_use]
pub fn named(sheets: &[Sheet]) -> HashMap<i64, i64> {
	let mut ordered: Vec<&Sheet> = sheets.iter().collect();
	ordered.sort_by_key(|s| s.at);
	let mut out = HashMap::new();
	for s in ordered {
		for p in &s.lx {
			if let [ts, xp, ..] = p[..] {
				out.insert(ts, xp);
			}
		}
	}
	out
}

/// The region's weekly reset the latest sheet names, by the time the addon folded it; none outside the game.
#[must_use]
pub fn reset(sheets: &[Sheet]) -> Option<i64> {
	sheets.iter().filter(|s| s.ws.is_some()).max_by_key(|s| s.at).and_then(|s| s.ws)
}

/// A game day with a move done, an offered pump, an app pump or an app pump's record (the phones'
/// `HistorySeries.Day`): its key, its pumps, its moves, its records and its moves' step.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SeriesDay {
	pub day: String,
	pub pumps: i64,
	pub moves: i64,
	pub records: i64,
	pub step: u8,
}

/// A game week, empty ones included (the phones' `HistorySeries.Week`): its start and its moves done by where they were
/// done: in the game's pumps, in its catch-ups and in its app pumps (together the moves its game days count).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SeriesWeek {
	pub start: i64,
	pub game: i64,
	pub catch_up: i64,
	pub free: i64,
}

/// One day of a week as its marks draw it (`GameCalendar.Cell`): its key, its moves as a step (0 none, 1 to 3 by
/// `map_steps`), a record day, today, a day to come.
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Cell {
	pub day: String,
	pub step: u8,
	pub record: bool,
	pub today: bool,
	pub future: bool,
}

/// A game week as Pro's weeks show it (`GameCalendar.Week`): its start, its seven days, its moves done by where they
/// were done (the game, a catch-up, an app), its moves and its records.
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Week {
	pub start: i64,
	pub days: Vec<Cell>,
	pub game: i64,
	pub catch_up: i64,
	pub app: i64,
	pub moves: i64,
	pub records: i64,
}

/// The weeks Pro's weeks show: a quarter.
const WEEKS_SHOWN: i64 = 13;

/// A count of moves as the weeks' marks step it: 0 for none, 1 below the first map step, 2 below the second, 3 from
/// it.
fn step(moves: i64, map_steps: &[i64]) -> u8 {
	if moves <= 0 {
		return 0;
	}
	if moves < map_steps.first().copied().unwrap_or(i64::MAX) {
		return 1;
	}
	if moves < map_steps.get(1).copied().unwrap_or(i64::MAX) { 2 } else { 3 }
}

/// The days and the weeks of the phones' `HistorySeries.build` on the computer: the game's rows (`rows`, read) with
/// their pumps and moves, the app pumps (`free`: each one's start and its moves done, counted as the phones count their
/// free pumps) and the app rows' beaten records (`app`, read); the weeks from the region's weekly reset `ws`, else from
/// Monday's turn of the first input's week, every one from the first input's to the later of now's and the last
/// input's, each with its moves done (contract/fixtures/history.json): a move row in the week of its own time, a
/// catch-up's where its pump (the offered pump row latest before it) is a device's, else the game's; a free pump's
/// moves in the week of its start.
pub fn series(
	rows: &[Read],
	free: &[(i64, i64)],
	app: &[Read],
	ws: Option<i64>,
	day: &GameDay,
	map_steps: &[i64],
	now: i64,
) -> (Vec<SeriesDay>, Vec<SeriesWeek>) {
	let mut by_day: HashMap<String, SeriesDay> = HashMap::new();
	let mut add = |t: i64, pumps: i64, moves: i64, records: i64| {
		let key = day.name(t);
		let d = by_day.entry(key.clone()).or_insert_with(|| SeriesDay { day: key, ..SeriesDay::default() });
		(d.pumps, d.moves, d.records) = (d.pumps + pumps, d.moves + moves, d.records + records);
	};
	for read in rows {
		let r = &read.row;
		if r.offered_pump() {
			add(r.ts, 1, 0, 0);
		}
		// a corrected beat is no record: the weeks never claim a record the fold might not
		if r.performed() {
			add(r.ts, 0, 1, i64::from(r.record(read)));
		}
	}
	for &(start, moves) in free {
		add(start, 1, moves, 0);
	}
	for read in app.iter().filter(|r| r.row.performed() && r.row.record(r)) {
		add(read.row.ts, 0, 0, 1);
	}
	let mut days: Vec<SeriesDay> = by_day.into_values().collect();
	days.sort_by(|a, b| a.day.cmp(&b.day));
	for d in &mut days {
		d.step = step(d.moves, map_steps);
	}
	let times: Vec<i64> = rows.iter().map(|r| r.row.ts).chain(free.iter().map(|&(t, _)| t)).collect();
	let (Some(&first), Some(&last)) = (times.iter().min(), times.iter().max()) else { return (days, Vec::new()) };
	let week = Weeks { anchor: ws.unwrap_or_else(|| day.monday(first)), days: Some(day) };
	let from = week.index(first);
	let to = week.index(now).max(week.index(last));
	let mut weeks: Vec<SeriesWeek> =
		(from..=to).map(|i| SeriesWeek { start: week.start(i), ..SeriesWeek::default() }).collect();
	let at = |t: i64| usize::try_from(week.index(t) - from).ok();
	// the rows by time: a move row belongs to the pump of the offered pump row latest before it (none: the game's)
	let mut device = false;
	for read in rows {
		let r = &read.row;
		if r.offered_pump() {
			device = r.by_device();
		} else if r.performed()
			&& let Some(w) = at(r.ts).and_then(|i| weeks.get_mut(i))
		{
			if device {
				w.catch_up += 1;
			} else {
				w.game += 1;
			}
		}
	}
	for &(start, moves) in free {
		if let Some(w) = at(start).and_then(|i| weeks.get_mut(i)) {
			w.free += moves;
		}
	}
	(days, weeks)
}

/// Pro's weeks (the phones' `GameCalendar.build`): newest first, from the one holding `now` back to the first day
/// with a move or a pump, at most `WEEKS_SHOWN`, from the series' `days` and `weeks` (ascending), started at the
/// region's weekly reset `ws`, else Monday's turn; none before a first day.
pub fn game_calendar(days: &[SeriesDay], weeks: &[SeriesWeek], ws: Option<i64>, day: &GameDay, now: i64) -> Vec<Week> {
	let today = day.name(now);
	let by_day: HashMap<&str, &SeriesDay> = days.iter().map(|d| (d.day.as_str(), d)).collect();
	let by_start: HashMap<i64, &SeriesWeek> = weeks.iter().map(|w| (w.start, w)).collect();
	let make = |start: i64| -> Week {
		let cells: Vec<Cell> = day
			.days_from(start)
			.into_iter()
			.map(|d| {
				let counts = by_day.get(d.as_str());
				Cell {
					step: counts.map_or(0, |c| c.step),
					record: counts.is_some_and(|c| c.records > 0),
					today: d == today,
					future: d > today,
					day: d,
				}
			})
			.collect();
		let counts = by_start.get(&start);
		let sum =
			|f: fn(&SeriesDay) -> i64| cells.iter().filter_map(|c| by_day.get(c.day.as_str())).map(|d| f(d)).sum();
		Week {
			start,
			game: counts.map_or(0, |w| w.game),
			catch_up: counts.map_or(0, |w| w.catch_up),
			app: counts.map_or(0, |w| w.free),
			moves: sum(|d| d.moves),
			records: sum(|d| d.records),
			days: cells,
		}
	};
	let Some(first) = days.first().map(|d| d.day.as_str()) else { return Vec::new() };
	let mut shown = Vec::new();
	for back in 0..WEEKS_SHOWN {
		let w = make(week_start(now, ws, day, back));
		let reaches = w.days.iter().any(|c| c.day.as_str() <= first);
		shown.push(w);
		if reaches {
			break;
		}
	}
	shown
}

/// Pro's weeks on the computer (the Saga's): the series of the game's rows (`rows`, read), every device's app pumps
/// from their rows (`app_pumps`) but those the game's rows hold already, which count once, from them (the phones'
/// `LogPumps.unheld`), and the app rows' records (`app`, read), laid out as Pro's weeks.
pub fn calendar(
	rows: &[Read],
	app_pumps: &[Pump],
	app: &[Read],
	ws: Option<i64>,
	day: &GameDay,
	map_steps: &[i64],
	now: i64,
) -> Vec<Week> {
	let held: HashSet<i64> =
		rows.iter().filter(|r| r.row.offered_pump() && Source::of(r.row.flags).app()).map(|r| r.row.ts).collect();
	let free: Vec<(i64, i64)> = app_pumps
		.iter()
		.filter(|p| !held.contains(&p.id))
		.map(|p| (p.id, i64::try_from(p.moves.len()).unwrap_or(i64::MAX)))
		.collect();
	let (days, weeks) = series(rows, &free, app, ws, day, map_steps, now);
	game_calendar(&days, &weeks, ws, day, now)
}

/// A ladder as a sheet shows it, where its rung, its move, its record or its stars changed (the phones'
/// `HistorySeries.LadderPoint`).
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct LadderPoint {
	pub at: i64,
	pub rung: i64,
	#[serde(rename = "move")]
	pub mv: i64,
	pub record: i64,
	pub stars: i64,
}

/// A ladder's points over the game's sheets: each game day's last sheet, a point wherever the ladder's entry changed
/// (an empty entry, rung 0, adds none).
pub fn ladder_points(sheets: &[Sheet], ladder: u8, day: &GameDay) -> Vec<LadderPoint> {
	let mut ordered: Vec<&Sheet> = sheets.iter().filter(|s| s.v >= 4).collect();
	ordered.sort_by_key(|s| s.at);
	// the day's last sheet
	let daily: Vec<&Sheet> = ordered
		.iter()
		.enumerate()
		.filter(|&(i, s)| ordered.get(i + 1).is_none_or(|next| day.index(next.at) != day.index(s.at)))
		.map(|(_, s)| *s)
		.collect();
	let mut points: Vec<LadderPoint> = Vec::new();
	for s in daily {
		let Some(mut e) = usize::from(ladder).checked_sub(1).and_then(|i| s.lad.get(i)).cloned() else { continue };
		// an older sheet's rungs as the current one counts them (a ladder that lost its first rung since)
		super::facts::facts().shift_entry(ladder, s.v, &mut e);
		let (rung, mv, record, stars) = (
			e.first().copied().unwrap_or(0),
			e.get(1).copied().unwrap_or(0),
			e.get(2).copied().unwrap_or(0),
			e.get(3).copied().unwrap_or(0),
		);
		if rung <= 0 {
			continue;
		}
		let same = points.last().is_some_and(|p| (p.rung, p.mv, p.record, p.stars) == (rung, mv, record, stars));
		if !same {
			points.push(LadderPoint { at: s.at, rung, mv, record, stars });
		}
	}
	points
}

/// The ladder's points that raised their move's record above an earlier one (a first max set places a record and
/// raises none).
#[must_use]
pub fn rises(points: &[LadderPoint]) -> Vec<&LadderPoint> {
	let mut last: HashMap<i64, i64> = HashMap::new();
	let mut out = Vec::new();
	for p in points {
		if last.get(&p.mv).is_some_and(|&before| before > 0 && p.record > before) {
			out.push(p);
		}
		last.insert(p.mv, p.record);
	}
	out
}

/// A typed count of `mv` (reps, or a hold's seconds) at its row's time: never a win or a miss, which only the sheets'
/// folded records say.
#[must_use]
pub fn attempts(rows: &[Read], mv: u16) -> Vec<(i64, i64)> {
	rows.iter()
		.filter(|r| {
			let row = &r.row;
			row.mv == mv && (row.kind == kind::REP || row.kind == kind::HOLD) && row.performed() && r.typed
		})
		.map(|r| (r.row.ts, if r.row.kind == kind::HOLD { r.row.seconds } else { r.row.reps }))
		.collect()
}
