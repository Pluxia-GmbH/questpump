//! The Saga on the computer (the Log's place under its players' name since 2026-10-15; docs/ui.md, "The Saga"; the
//! phones' `LogTab`), as the `log` topic: today's chapter as `desk::saga` makes the day (its rank, strip, kinds, muscles,
//! records and unlocks, its XP, level and streak, its week and its pumps as the Log lists them, each with where it came
//! from and the XP its sheet paid), Pro's weeks with Pro (`Helper::pro_on`), the earlier chapters, newest first and
//! paged (`CHAPTERS` more at each `log.more`), the chapter whose page is open (`log.open {day}`, none with `day` null),
//! the focus's plan for the kinds bar, the best-day moment once a game day on this computer and never under the
//! felt-unwell lock (shown, then `log.best {day}`, kept in the settings' `desk.best_day`, never synced), and whether
//! today's page plays its reveal (until `log.revealed`, once a game day on this computer, kept in the settings'
//! `desk.revealed`). Pump home's today's chapter comes from the same build (`LogState::home`), never a second count,
//! and so do the moves on its recent games (`recent_rows`). All of it from the rows the helper holds
//! (`history`: the game's rows and
//! catch-ups, every device's app rows), never a phone's Apple Health or Health Connect minutes. It is due again
//! whenever the stores take rows or a sheet comes, Pro turns on or off, the engine's own XP today moves, and when a new
//! game day begins, and built once at the end of the loop's turn (`Helper::log_flush`: a build over every row takes tens
//! of milliseconds, which the pump's clock never waits on), never while a desktop pump runs (it covers the window) but
//! at its end. A pump's XP is the one the game's sheets name (the sheets the strip brought, then the game's sheet the
//! app shows, `Helper::game_shown`, which without the strip is the engine's fold of the game's save); an app pump's
//! comes from the engine's own fold (its summary's `lx`) while no game sheet shows. A day's XP and level are
//! contract/fixtures/history.json `saga` rule 15's, read from the game's sheet the app shows (the newest: a strip
//! sheet, or the engine's fold of the save that `GET /v1/sheet` serves): the XP the game's fold counted on that game
//! day, which the sheet carries as `dx` over the game days from its `dxFrom` (the save is the account's, so never one
//! character's sheets less another's; a day before `dxFrom`, or a week reaching before it, unknown: no XP, no level
//! line, no week's XP), its level at the day's start and end from the lifetime XP less that day's and later days' XP
//! through the level curve, which the engine answers (its `levels` op, Stats.lua's levelOf, never a Rust copy: the
//! build waits for an answer it lacks while the engine is ready); a sheet without `dx` (an older addon) names no day's
//! XP, and each day's level line is that day's newest sheet's level alone; with the app's own sheet alone today's XP
//! only (the engine's own `todayXp`), none on an earlier day or a week; a week's XP its days' summed. A day's streak is
//! the one it ended with (rule 16: the newest logged sheet of that day, any character's; today's the streak's board).
//! The chapters and the old best are anchored on the game sheet that knows the most active days (rules 3 and 4: the
//! largest `ad` of the sheets the helper logged and the one it shows, with that sheet's `bd`). The days a rest token
//! kept in today's game week come from the streak's board, by date. Under the felt-unwell lock the page stays still
//! (no reveal).

use std::collections::{BTreeSet, HashMap};

use questpump_core::app_rows::after_start_over;
use questpump_core::verdict::Verdict;
use questpump_engine::LevelAt;
use serde::Serialize;
use serde_json::Value;
use tauri::Manager as _;

use super::facts::{Facts, facts};
use super::hero::{DayXp, Summary};
use super::history::{self, GameDay, Sheet, Week, Xp, civil, date_index};
use super::saga::{Bar, ChapterRow, Level, OldBest, Quality, Saga, SagaDay, Told};
use super::{Desk, Intent, figures, reel};
use crate::lan;
use crate::runtime::Helper;

/// The Log's topic, the Saga's.
const LOG: &str = "log";
/// The earlier chapters one page of the list adds.
const CHAPTERS: usize = 30;
/// The kinds in the kinds bar's order (library.json's blocks).
const KINDS: [&str; 4] = ["strength", "cardio", "mobility", "yoga"];

/// The Saga's state in the loop.
#[derive(Default)]
pub(crate) struct LogState {
	/// How many earlier chapters the list shows: `CHAPTERS`, and `CHAPTERS` more at each `log.more`.
	chapters: usize,
	/// The game day the Log was last built on (days since 1970-01-01).
	day: i64,
	/// The chapter whose page is open (`log.open`), by its game day.
	open: Option<i64>,
	/// Pump home's today's chapter, as the last build made it.
	pub(crate) home: Option<TodayChapter>,
	/// The engine's own fold's XP by pump, while no game sheet came; a summary call on its way, and one more after it.
	app_xp: Option<HashMap<i64, i64>>,
	asking: bool,
	again: bool,
	/// The levels the last build's level lines read.
	levels: Levels,
	/// The Log is due: built at the end of the loop's turn once no desktop pump runs.
	due: bool,
}

/// The level each lifetime XP the last build read reaches (a day's at its start and its end, the engine's `levels` op,
/// Stats.lua's levelOf), none where the engine did not answer; and whether an ask is on its way.
#[derive(Default)]
struct Levels {
	known: HashMap<i64, Option<LevelAt>>,
	asking: bool,
}

/// Pump home's today's chapter (`home`'s `today`): today's game day, chapter, moves, word and best day, its records
/// and its strip: the Saga's own numbers.
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub(crate) struct TodayChapter {
	day: String,
	chapter: Option<i64>,
	moves: i64,
	quality: Option<Quality>,
	best: bool,
	records: i64,
	bars: Vec<Bar>,
	buckets: [i64; 4],
}

/// The focus's plan beside the kinds bar: its name (contract/sheet.json's) and its shares by kind, in percent.
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
struct Plan {
	name: String,
	shares: [i64; 4],
}

/// The best-day moment: today's key and moves and the old best they beat.
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct BestMoment {
	day: String,
	moves: i64,
	old_best: OldBest,
}

/// `log`: Pro's weeks with Pro (none without Pro or before a first pump), whether any pump was ever done, today's
/// chapter, the open chapter's page, the earlier chapters so far and whether more wait, the focus's plan, the best-day
/// moment while it waits to show, and whether today's page plays its reveal.
#[derive(Serialize, Debug, Clone, PartialEq)]
#[expect(clippy::struct_excessive_bools, reason = "the Saga's state: one yes or no of the page a field")]
struct LogTopic {
	pro: bool,
	weeks: Vec<Week>,
	ever: bool,
	today: SagaDay,
	open: Option<SagaDay>,
	chapters: Vec<ChapterRow>,
	more: bool,
	plan: Plan,
	best: Option<BestMoment>,
	reveal: bool,
}

impl From<&SagaDay> for TodayChapter {
	fn from(d: &SagaDay) -> Self {
		Self {
			day: d.day.clone(),
			chapter: d.chapter,
			moves: d.moves,
			quality: d.quality,
			best: d.best,
			records: d.records.len().try_into().unwrap_or(0),
			bars: d.bars.clone(),
			buckets: d.buckets,
		}
	}
}

/// The engine's summary's `lx`: [pump time, XP] pairs.
#[derive(serde::Deserialize, Default)]
#[serde(default)]
struct Logged {
	lx: Vec<Vec<i64>>,
}

/// What the game's sheet the app shows says of a day's XP and level (rule 15): the game's, with the XP its fold
/// counted on each game day it covers (`dx` from `dxFrom`); the game's from an older addon, without it, whose days show
/// their own newest sheet's level alone (the sheets the helper logged and the one it shows); or the app's own alone (its
/// summary and the XP it counted today).
enum Gains<'a> {
	Game { summary: &'a Summary, dx: &'a DayXp },
	Older { summary: &'a Summary, sheets: &'a [Sheet], day: &'a GameDay },
	Own { summary: Option<&'a Summary>, today_xp: i64 },
}

impl Gains<'_> {
	/// The XP game day `d` gained (`today` today's); none at 0, before `dxFrom` or where the sheet names none.
	fn xp(&self, d: i64, today: i64) -> Option<i64> {
		match self {
			Self::Game { dx, .. } => dx.on(d),
			Self::Older { .. } => None,
			Self::Own { today_xp, .. } => (d == today).then_some(*today_xp).filter(|&x| x > 0),
		}
	}

	/// The lifetime XP at game day `d`'s end and at its start (`dx`: less the XP of the days after it, and of it too),
	/// where the level line reads them; none before `dxFrom`, before a level is known or without `dx`.
	fn ends(&self, d: i64, today: i64) -> Option<(i64, i64)> {
		match *self {
			Self::Game { summary: s, dx } => {
				(s.level > 0 && d >= dx.from).then_some(())?;
				let end = s.xp - dx.days.range(d + 1..).map(|(_, xp)| xp).sum::<i64>();
				Some((end, end - dx.days.get(&d).copied().unwrap_or(0)))
			}
			Self::Older { .. } => None,
			Self::Own { summary, today_xp } => {
				summary.filter(|s| s.level > 0 && d == today).map(|s| (s.xp, s.xp - today_xp))
			}
		}
	}

	/// The lifetime XP whose levels game day `d`'s level line needs from the engine: its end's and its start's, less
	/// the sheet's own (its level as it stands).
	fn wants(&self, d: i64, today: i64) -> Vec<i64> {
		let own = self.sheet().map(|s| s.xp);
		self.ends(d, today)
			.map_or_else(Vec::new, |(end, start)| [end, start].into_iter().filter(|&xp| Some(xp) != own).collect())
	}

	/// The sheet the level reads from.
	fn sheet(&self) -> Option<&Summary> {
		match *self {
			Self::Game { summary, .. } | Self::Older { summary, .. } => Some(summary),
			Self::Own { summary, .. } => summary,
		}
	}

	/// Game day `d`'s level at its end with the bar's XP into it and its need, and at its start (Level 23 → 24), the
	/// levels of lifetime XP from `levels` (the engine's answers), the sheet's own where the lifetime XP is its own; its
	/// start unknown where the engine has not answered. Without `dx` the level of that day's newest sheet alone (the one
	/// shown on its own day, else the newest logged). None where no level is known.
	fn level(&self, d: i64, today: i64, levels: &Levels) -> Option<Level> {
		if let Self::Older { summary: s, sheets, day } = *self {
			let logged = sheets.iter().filter(|x| day.index(x.at) == d).max_by_key(|x| x.at);
			let (to, into, need) = match logged {
				Some(x) if day.index(s.at) != d || x.at > s.at => (x.lvl, x.into, x.need),
				_ if day.index(s.at) == d => (s.level, s.into, s.need),
				_ => return None,
			};
			return (to > 0).then_some(Level { from: None, to, into, need });
		}
		let s = self.sheet()?;
		let at = |xp: i64| {
			if xp == s.xp {
				Some(LevelAt { level: s.level, into: s.into, need: s.need, paragon: 0 })
			} else {
				levels.known.get(&xp).copied().flatten()
			}
		};
		let (end, start) = self.ends(d, today)?;
		let to = at(end)?;
		Some(Level { from: at(start).map(|l| l.level), to: to.level, into: to.into, need: to.need })
	}

	/// The XP the game week from `start` gained, its days' summed; none with the app's own sheet alone, without `dx`,
	/// for a week reaching before `dxFrom` or at 0.
	fn week(&self, start: i64) -> Option<i64> {
		let Self::Game { dx, .. } = self else { return None };
		(start >= dx.from).then(|| dx.days.range(start..=start + 6).map(|(_, xp)| xp).sum::<i64>()).filter(|&x| x > 0)
	}
}

/// The Saga anchored on the game sheet that knows the most active days (rules 3 and 4), of the game's sheets the helper
/// logged and the one it shows (a strip sheet's `ad` counts the save's baseline, and so does the engine's fold of the
/// save once the helper read that save since it started), the newer of two alike: its count, its two best days and its
/// game day; as it was without an `ad` (an older addon).
fn anchored<'a>(saga: Saga<'a>, logged: &[Sheet], shown: Option<&Summary>, day: &GameDay) -> Saga<'a> {
	let log = logged.iter().filter_map(|s| Some(((s.ad?, s.at), s))).max_by_key(|&(key, _)| key);
	let shown = shown.and_then(|s| Some(((s.active_days?, s.at), s)));
	let ((ad, at), best_days) = match (log, shown) {
		(Some((key, s)), None) => (key, s.best_days()),
		(Some((key, s)), Some((other, _))) if key > other => (key, s.best_days()),
		(_, Some((key, s))) => (key, s.best_days.clone()),
		(None, None) => return saga,
	};
	saga.anchored(ad, &best_days, day.index(at))
}

/// The streak earlier game day `d` ended with (rule 16): its newest logged sheet's, any character's (the save is the
/// account's); none where no sheet of that day came.
fn streak_on(sheets: &[Sheet], day: &GameDay, d: i64) -> Option<i64> {
	sheets.iter().filter(|s| day.index(s.at) == d).max_by_key(|s| s.at).map(|s| s.streak).filter(|&s| s > 0)
}

/// The plan of the focus a sheet names (contract/sheet.json's code; Balanced before a focus row), with its shares by
/// kind (library.json's `focus_share`).
fn plan(f: &Facts, focus: Option<u8>) -> Plan {
	let focus = focus.and_then(|code| f.focus(code)).or_else(|| f.sheet.focus.first());
	let shares = focus.and_then(|fo| f.constants().focus_share.get(&fo.key));
	Plan {
		name: focus.map(|fo| fo.name.clone()).unwrap_or_default(),
		shares: KINDS.map(|k| shares.and_then(|s| s.get(k)).copied().unwrap_or(0)),
	}
}

/// Pro's weeks from the game's rows and the app's rows and pumps (`history::calendar`), the rows read with every
/// store's corrections (`fixes`).
fn pro_weeks(
	game: &[history::Row],
	app: &[history::Row],
	fixes: &history::Fixes,
	app_pumps: &[history::Pump],
	ws: Option<i64>,
	day: &GameDay,
	now: i64,
) -> Vec<Week> {
	let (game, app) = (history::read(game, fixes), history::read(app, fixes));
	history::calendar(&game, app_pumps, &app, ws, day, &facts().constants().map_steps, now)
}

impl Helper {
	/// The Log again from the rows: with no game sheet shown (`Helper::game_shown`), after the engine's own fold names
	/// the app pumps' XP (once at a time; while it folds, without it, and its landing asks).
	pub(crate) fn log_refresh(&mut self) {
		let at = lan::now_ms().div_euclid(1_000);
		if self.game_shown(&self.session.sheet(at)).is_some() || !self.engine_ready() {
			self.book_log.log.app_xp = None;
			self.log_publish();
			return;
		}
		if self.book_log.log.asking {
			self.book_log.log.again = true;
			return;
		}
		self.book_log.log.asking = true;
		self.engine_call(
			move |engine| engine.summary(at),
			|helper, answer| {
				let log = &mut helper.book_log.log;
				log.asking = false;
				match answer.map(serde_json::from_value::<Logged>) {
					Ok(Ok(logged)) => {
						log.app_xp = Some(logged.lx.iter().filter_map(|p| Some((*p.first()?, *p.get(1)?))).collect());
					}
					Ok(Err(e)) => log::warn!("log: the engine's summary does not read: {e}"),
					Err(e) => log::warn!("log: the engine did not answer: {e}"),
				}
				if std::mem::take(&mut log.again) {
					helper.log_refresh();
				}
				helper.log_publish();
			},
		);
	}

	/// The pair tick: a new game day moves today's chapter on.
	pub(crate) fn log_day(&mut self) {
		let f = facts();
		if f.whole().is_err() {
			return;
		}
		if GameDay::new(f.constants().day_offset_h).index(lan::now_ms().div_euclid(1_000)) != self.book_log.log.day {
			self.log_publish();
		}
	}

	/// The `log` topic is due again: built at the end of the loop's turn (`log_flush`).
	pub(crate) fn log_publish(&mut self) {
		self.book_log.log.due = true;
	}

	/// The end of the loop's turn: the Log, when it is due and no desktop pump covers the window (the turn after the
	/// pump's end builds it).
	pub(crate) fn log_flush(&mut self) {
		if !self.book_log.log.due || self.desk_pump_running() {
			return;
		}
		self.book_log.log.due = false;
		self.log_build();
	}

	/// The `log` topic, and Pump home's today's chapter with it, from the rows and the sheets now.
	fn log_build(&mut self) {
		// a contract file that does not read leaves the Saga empty, as the Book (`desk/facts.rs`)
		let f = facts();
		if f.whole().is_err() {
			return;
		}
		let now = lan::now_ms().div_euclid(1_000);
		let day = GameDay::new(f.constants().day_offset_h);
		let today = day.index(now);
		let now_sheet = self.session.sheet(now);
		let shown = self.game_shown(&now_sheet);
		self.logged_sheets();
		let logged = self.book_log.sheets.list.as_slice();
		// the sheet shown: the game's, else the app's own fold's
		let summary = shown.as_ref().map(|g| &g.summary).or(self.home.summary.as_ref());
		let gains = match shown.as_ref().map(|g| (&g.summary, g.summary.day_xp.as_ref())) {
			Some((summary, Some(dx))) => Gains::Game { summary, dx },
			Some((summary, None)) => Gains::Older { summary, sheets: logged, day: &day },
			None => Gains::Own { summary: self.home.summary.as_ref(), today_xp: self.home.today_xp },
		};
		// the levels today's and the open page's level lines read, from the engine: the build waits for its answer
		let open_day = self.book_log.log.open.filter(|&d| d < today);
		let want: BTreeSet<i64> = open_day.into_iter().chain([today]).flat_map(|d| gains.wants(d, today)).collect();
		let levels = &mut self.book_log.log.levels;
		levels.known.retain(|xp, _| want.contains(xp));
		let missing: Vec<i64> = want.iter().copied().filter(|xp| !levels.known.contains_key(xp)).collect();
		if !missing.is_empty() && self.engine_ready() {
			if !self.book_log.log.levels.asking {
				self.log_levels(missing);
			}
			return;
		}
		let (game, app) = self.log_rows();
		// the game's sheet the app shows names the XP last: the engine's fold of the save names pumps no logged sheet has
		let mut game_xp = history::named(logged);
		game_xp.extend(shown.as_ref().map(|g| g.summary.pump_xp.clone()).unwrap_or_default());
		let xp = Xp { game: game_xp, app: self.book_log.log.app_xp.clone() };
		let ws = history::reset(logged);
		let fixes = history::Fixes::of(&[&game, &app]); // one map over both stores (slice 44)
		let (game_pumps, app_pumps) = (history::group(&game, &fixes), history::app_pumps(&app, &fixes));
		let log_days = history::log_days(&game_pumps, &app_pumps, &xp, &|m| f.hold(m), &day, now);
		let pro = self.pro_on() == Some(true);
		let weeks = if pro { pro_weeks(&game, &app, &fixes, &app_pumps, ws, &day, now) } else { Vec::new() };
		let rows: Vec<history::Row> = game.iter().chain(&app).copied().collect();
		let earned = summary.map(|s| s.earned.as_slice()).unwrap_or_default();
		let saga = Saga::new(f, &day, history::shown(&game_pumps, &app_pumps), &rows, earned);
		let saga = anchored(saga, logged, shown.as_ref().map(|g| &g.summary), &day);
		let entries = |d: i64| -> Vec<history::Entry> {
			let key = history::civil(d);
			log_days.iter().find(|x| x.id == key).map(|x| x.entries.clone()).unwrap_or_default()
		};
		let start = saga.week_start(today, ws);
		// the days of today's game week a rest token kept (it may begin before this Monday, at the region's reset)
		let board = self.board_from(&now_sheet, now, start);
		let rested: Vec<i64> = board.rested.iter().copied().collect();
		let levels = &self.book_log.log.levels;
		let told = |d: i64, start: i64| Told {
			xp: gains.xp(d, today),
			level: gains.level(d, today, levels),
			streak: if d == today { Some(board.days).filter(|&s| s > 0) } else { streak_on(logged, &day, d) },
			week_xp: gains.week(start),
		};
		// a day's reel films only the moves whose film this computer has, as the Share window's maker reads them
		let filmed = |mv: u16| figures::filmed(f, mv);
		let tune = reel::tune();
		let view = |d: i64, start: i64| saga.view(d, today, start, told(d, start), &rested, entries(d), &filmed, tune);
		let today_view = view(today, start);
		let open = open_day.map(|d| view(d, saga.week_start(d, ws)));
		let shown_chapters = self.book_log.log.chapters.max(CHAPTERS);
		// the earlier game days the Log lists pumps on: a quiet row each for those without a move done
		let listed: Vec<i64> = log_days.iter().filter(|x| x.ago > 0).map(|x| today - x.ago).collect();
		let mut chapters = saga.chapters(today, &listed);
		let more = chapters.len() > shown_chapters;
		chapters.truncate(shown_chapters);
		// shown once a game day on this computer, never under the felt-unwell lock (Pump home's rest card reads it alike)
		let desk = &self.settings.desk;
		let seen = desk.best_day.as_deref() == Some(today_view.day.as_str());
		let locked = now_sheet.locked_until.is_some();
		let best = today_view
			.old_best
			.clone()
			.filter(|_| today_view.best && !seen && !locked)
			.map(|old_best| BestMoment { day: today_view.day.clone(), moves: today_view.moves, old_best });
		// the reveal plays once a game day on this computer, and never under the felt-unwell lock: a still page
		let reveal = today_view.moves > 0 && !locked && desk.revealed.as_deref() != Some(today_view.day.as_str());
		let home = TodayChapter::from(&today_view);
		let topic = LogTopic {
			pro,
			weeks,
			ever: !game_pumps.is_empty() || !app_pumps.is_empty(),
			today: today_view,
			open,
			chapters,
			more,
			plan: plan(f, summary.and_then(|s| s.focus)),
			best,
			reveal,
		};
		let log = &mut self.book_log.log;
		log.chapters = shown_chapters;
		log.day = today;
		self.app.state::<Desk>().publisher.publish(LOG, &topic);
		// Pump home's recent games take their moves from the same build, asked whatever the chapter did
		if (self.book_log.log.home.as_ref() != Some(&home)) | self.recent_rows(&game_pumps, &app_pumps, now) {
			self.book_log.log.home = Some(home);
			self.home_publish();
		}
	}

	/// The levels of the lifetime XP `xps` from the engine (Stats.lua's levelOf through the `levels` op), once at a time;
	/// the Saga is built again with its answer (none for each where the engine did not answer).
	fn log_levels(&mut self, xps: Vec<i64>) {
		self.book_log.log.levels.asking = true;
		let asked = xps.clone();
		self.engine_call(
			move |engine| engine.levels(&xps),
			move |helper, answer| {
				let levels = &mut helper.book_log.log.levels;
				levels.asking = false;
				let answered = answer.unwrap_or_else(|e| {
					log::warn!("log: the engine did not answer the levels: {e}");
					Vec::new()
				});
				for (i, xp) in asked.into_iter().enumerate() {
					levels.known.insert(xp, answered.get(i).copied());
				}
				helper.log_publish();
			},
		);
	}

	/// The rows the Saga reads: the game's (each pump row with its heart-rate proof, as `GET /v1/history` serves it)
	/// and every device's app rows from the app's latest Start over on (a phone's, slice 28: the app's history starts
	/// there), with the other computers' game rows beside them as rows of another source (slice 60, `far_records`: as
	/// every mirror carries them to the game, which keeps them past a phone's Start over). The Book's ladder chart reads
	/// them too.
	pub(crate) fn log_rows(&self) -> (Vec<history::Row>, Vec<history::Row>) {
		let proof = self.session.book();
		let game = self
			.session
			.store()
			.records()
			.iter()
			.map(|r| {
				let mut row = history::Row::of(r);
				row.proven = row.offered_pump() && proof.proof(r.row_ts).is_some_and(|p| p.verdict == Verdict::Proven);
				row
			})
			.collect();
		let own = self.session.app_rows();
		let start_over = own.start_over();
		let mut app: Vec<history::Row> =
			own.records().iter().filter(|r| after_start_over(&r.row(), start_over)).map(history::Row::of).collect();
		app.extend(self.session.far_records(0).iter().map(history::Row::of));
		(game, app)
	}

	/// The `log` area's intents: `log.more` (the next page of the earlier chapters), `log.open {day}` (the chapter whose
	/// page shows, "yyyy-MM-dd", or null as it closes), `log.best {day}` (the best-day moment showed for that game day)
	/// and `log.revealed` (today's page played its reveal, kept for the game day the Saga was last built on).
	pub(crate) fn log_intent(&mut self, intent: &Intent) -> Result<(), String> {
		let log = &mut self.book_log.log;
		match intent.kind() {
			"log.more" => log.chapters = log.chapters.max(CHAPTERS) + CHAPTERS,
			"log.open" => {
				log.open = match intent.arg("day") {
					Some(Value::Null) | None => None,
					Some(day) => Some(day.as_str().and_then(date_index).ok_or("log.open takes a day, yyyy-MM-dd")?),
				};
			}
			"log.best" => {
				let day = intent.arg("day").and_then(Value::as_str).ok_or("log.best takes a day")?.to_owned();
				date_index(&day).ok_or("log.best takes a day, yyyy-MM-dd")?;
				self.settings.desk.best_day = Some(day);
				self.save_settings();
			}
			"log.revealed" => {
				self.settings.desk.revealed = Some(civil(log.day));
				self.save_settings();
			}
			_ => return Err(intent.unknown()),
		}
		self.log_publish();
		Ok(())
	}
}
