//! A move's pour (slice 65's hold, every move since slice 66, `desk.pump.saved`): what its save saved (a set of reps'
//! reps, a hold's seconds, a counted set's clean reps, a timed move's seconds) flies one grain a unit from the dial into
//! the count bar against the step's range; the row is written as the pour lands, and the move's own rewards (its row,
//! the step it opened with its reveal, its stats and XP) show from the app's own sheet before and after that one write,
//! as the phones' per-move card reads them (`RewardsLine.since`). Rust plans every instant and plays the sounds as they
//! come due, as the Result's reveal does (`Reveal`); the page draws from the instants.

use questpump_core::codes::{answer, kind};
use questpump_engine::{MoveResult, PlannedMove};
use serde::Serialize;

use super::super::run;
use super::super::{Recorded, Run};
use super::{MOTION, Reward, Row, Sets, Sum, Tile, moved, stats, tile, unlocks};
use crate::desk::facts::facts;

/// A tick closer than this after the last plays nothing (the game's `pour_tick_gap_s`, its twin in Constants.lua; no
/// motion token carries it, as a sound's spacing is no motion).
const TICK_GAP_MS: i64 = 28;
/// The grains' starts close in as they go: each at the pour's span times (i / n) to this power.
const CLOSING: f64 = 0.62;
/// The most of a ladder's step tiles the reveal shows, the current one and the one it opens among them.
const TILES: u8 = 7;

/// The count bar (`CountBar`, every move's since slice 66; the hold bar of slice 65): the step's range from 0 to its top
/// (`range` where a step has none), its label's ladder and step in the step's quality, the posts at the goal, at the
/// record before this set (none once that stands at the top or past it) and at the top, and the unit its words are in
/// (`seconds`: m:ss, else a number).
#[derive(Serialize, Clone, PartialEq)]
pub(in crate::desk::pump) struct CountBar {
	ladder: Option<&'static str>,
	step: Option<i64>,
	quality: Option<u8>,
	goal: Option<i64>,
	best: Option<i64>,
	top: Option<i64>,
	range: i64,
	seconds: bool,
}

impl CountBar {
	fn of(ladder: u8, rung: u8, posts: Posts, held: i64) -> Self {
		let Posts { goal, best, top, seconds } = posts;
		let name = Some(facts().ladder_name(ladder)).filter(|n| !n.is_empty());
		let step = facts().step(ladder, rung).map(i64::from);
		let quality = Some(facts().rung_quality(ladder, i64::from(rung))).filter(|&q| q > 0);
		let range = top.unwrap_or_else(|| goal.into_iter().chain(best).chain([held, 1]).max().unwrap_or(1));
		let best = best.filter(|&b| top.is_none_or(|t| b < t));
		Self { ladder: name, step, quality, goal, best, top, range, seconds }
	}
}

/// What a move's bar stands its save against, by what was saved (slice 66): a set of reps or a hold its goal, its record
/// before this set (none with a weight, whose record is a score the engine judges from the row: the count step shows
/// no chip for it either) and its step's top (a first max set's where its step has no window); a counted set its best
/// clean count alone, and only on "Yes, to the end" (a slipped or not all-out count is never a record, `ns.Judge`), no
/// count to reach and no top; seconds (a timed move, a pose, a counted set stopped before its end) their goal alone,
/// never a record or a best; and its unit.
#[derive(Clone, Copy, PartialEq)]
struct Posts {
	goal: Option<i64>,
	best: Option<i64>,
	top: Option<i64>,
	seconds: bool,
}

impl Posts {
	fn of(m: &PlannedMove, (ladder, rung): (u8, u8), shown: Recorded) -> Self {
		let load = match shown {
			Recorded::Counted(_, said) => {
				return Self { goal: None, best: m.record.filter(|_| said == answer::YES), top: None, seconds: false };
			}
			Recorded::Time(_) => return Self { goal: m.seconds, best: None, top: None, seconds: true },
			Recorded::Count(_, load) => load,
			Recorded::Done => None,
		};
		if m.counted || (m.kind != kind::REP && m.kind != kind::HOLD) {
			return Self { goal: m.seconds, best: None, top: None, seconds: true };
		}
		let top = facts().window(ladder, rung, m.r#move).map(|w| w.hi).or(m.top);
		let best = m.record.filter(|_| load.is_none());
		Self { goal: super::goal(m), best, top, seconds: m.kind == kind::HOLD }
	}
}

/// A move's save on its way to its pour (`Helper::pump_pour`): what it saved and what its line shows, its slot's ladder
/// and rung, the app's own sheet before the write (`before`; `fresh` where this is the pump's first write, whose
/// summary's last pump is another pump's, so this save's gains are the whole of the last pump's after it) and the next
/// move's name (none on the pump's last).
pub(in crate::desk::pump) struct Saved {
	pub(in crate::desk::pump) result: MoveResult,
	pub(in crate::desk::pump) shown: Recorded,
	pub(in crate::desk::pump) step: (u8, u8),
	pub(in crate::desk::pump) before: Sum,
	pub(in crate::desk::pump) fresh: bool,
	pub(in crate::desk::pump) next: Option<&'static str>,
	/// A window already stands over the page as the pour starts (`Pumps::window`).
	pub(in crate::desk::pump) covered: bool,
}

/// The step-up reveal (`StepReveal`) over the figure's room: the ladder's tiles (`from` the current step's, `to` the one
/// the save opened), the light from `at`, the next tile opening at `opens`, the fold into the unlock row at `folds`;
/// `colour` where the step brings a new quality, which adds the rays.
#[derive(Serialize, Clone, PartialEq)]
pub(in crate::desk::pump) struct StepReveal {
	at: i64,
	opens: i64,
	folds: i64,
	tiles: Vec<Tile>,
	from: usize,
	to: usize,
	step: i64,
	quality: u8,
	word: &'static str,
	hero: Tile,
	colour: bool,
	landed: bool,
}

/// What this save earned, once it is written: its rows (the move, the step it opened, its stats, its XP), each coming
/// in at its instant in `rows_at`, after the reveal where one plays, and the bar the reveal rolls over to (`next_bar`).
#[derive(Serialize, Clone, PartialEq)]
pub(in crate::desk::pump) struct SavedEarned {
	rows: Vec<Row>,
	rows_at: Vec<i64>,
	reveal: Option<StepReveal>,
	next_bar: Option<CountBar>,
}

/// The pour as the page draws it: the count saved (`value`, in the bar's unit) against the dial's whole (`against`), the
/// record the dial's chip shows past (none on a timed move), the bar, each grain's start (`starts`, each landing `fly`
/// after), whether it landed, the next move's name (none on the pump's last: Finish) and, once written, what it earned.
#[derive(Serialize, Clone, PartialEq)]
pub(in crate::desk::pump) struct SavedView {
	value: i64,
	against: i64,
	record: Option<i64>,
	bar: CountBar,
	at: i64,
	starts: Vec<i64>,
	fly: i64,
	landed: bool,
	next: Option<&'static str>,
	earned: Option<SavedEarned>,
}

/// The move's one jingle (`jingleOf`): the record's or the top's at its grain, else after the write (`After`: the
/// reveal's `unlockUncommon` where the save opened a step, else the top's `unlock` where the top was reached by a set
/// that may climb, else `done` as the rows come in).
#[derive(Clone, Copy, PartialEq)]
enum Jingle {
	Grain(&'static str),
	After { top: bool },
}

/// The windows over a pour: one stands, or the last one closed at this instant (ms; 0 before one has).
#[derive(Clone, Copy, PartialEq)]
enum Over {
	Window,
	Closed(i64),
}

/// A move's pour on screen, from its plan to Next.
pub(in crate::desk::pump) struct Pour {
	pub(in crate::desk::pump) slot: usize,
	pub(in crate::desk::pump) result: MoveResult,
	/// What the move's line shows once its row is written (`Helper::pump_write`).
	pub(in crate::desk::pump) shown: Recorded,
	view: SavedView,
	sounds: Vec<(i64, &'static str)>,
	jingle: Jingle,
	jingled: bool,
	still: bool,
	end: i64,
	ladder: u8,
	rung: u8,
	/// The sheet before the write that this save's gains are counted from: none on the pump's first write.
	before: Option<Sum>,
	/// A click on the stage came after the pour landed and before what the save earned (`land_reveal`).
	hurried: bool,
	/// The windows over the pour (`cover`).
	over: Over,
}

impl Pour {
	/// The pour of move `m`'s save from `now`: one grain a unit of what it saved (a rep, a second), against the posts
	/// its kind stands it against (`Posts`).
	pub(in crate::desk::pump) fn plan(now: i64, m: &PlannedMove, saved: Saved) -> Self {
		let Saved { result, shown, step: (ladder, rung), before, fresh, next, covered } = saved;
		let motion = &*MOTION;
		let held = result.value.max(0);
		let posts = Posts::of(m, (ladder, rung), shown);
		let Posts { goal, best, top, .. } = posts;
		let span = motion.pour.min(motion.pour_base + motion.pour_step * held);
		let n = run::real(held.max(1));
		let starts: Vec<i64> = (0..held)
			.map(|i| now + motion.pour_hold + run::millis(run::secs(span) * (run::real(i) / n).powf(CLOSING)))
			.collect();
		let end = starts.last().map_or(now, |&s| s + motion.pour_fly);
		let climbs = top.is_some_and(|t| held >= t) && may_climb(m, ladder, rung, &before);
		let jingle = match (top, best) {
			(Some(t), _) if held >= t && !climbs => Jingle::Grain("unlock"),
			_ if climbs => Jingle::After { top: true },
			(_, Some(b)) if held > b => Jingle::Grain("record"),
			_ => Jingle::After { top: false },
		};
		// each landing second's one sound, strongest first: the jingle that is its, the top's, a post's, else a tick
		let mut sounds = Vec::new();
		let mut last_tick = i64::MIN;
		for (i, &start) in starts.iter().enumerate() {
			let k = i64::try_from(i).unwrap_or(0) + 1;
			let lands = start + motion.pour_fly;
			let at_top = top == Some(k);
			let at_best = best.is_some_and(|b| k == b + 1);
			let event = match jingle {
				Jingle::Grain(j @ "unlock") if at_top => Some(j),
				Jingle::Grain(j @ "record") if at_best => Some(j),
				_ if at_top => Some("pourTop"),
				_ if at_best || goal == Some(k) => Some("pourPost"),
				_ if lands.saturating_sub(last_tick) >= TICK_GAP_MS => {
					last_tick = lands;
					Some("pourTick")
				}
				_ => None,
			};
			sounds.extend(event.map(|e| (lands, e)));
		}
		let bar = CountBar::of(ladder, rung, posts, held);
		Self {
			slot: m.slot,
			result,
			shown,
			view: SavedView {
				value: held,
				against: goal.or(best).filter(|&a| a > 0).unwrap_or(held.max(1)),
				record: best,
				bar,
				at: now,
				starts,
				fly: motion.pour_fly,
				landed: false,
				next,
				earned: None,
			},
			sounds,
			jingle,
			jingled: false,
			still: false,
			end,
			ladder,
			rung,
			before: (!fresh).then_some(before),
			hurried: false,
			over: if covered { Over::Window } else { Over::Closed(0) },
		}
	}

	pub(in crate::desk::pump) fn view(&self) -> &SavedView {
		&self.view
	}

	pub(in crate::desk::pump) fn landed(&self) -> bool {
		self.view.landed
	}

	/// Its next instant: a sound's, the pour's landing, or the move going on by itself.
	pub(in crate::desk::pump) fn wake(&self) -> Option<i64> {
		let lands = (!self.view.landed).then_some(self.end);
		self.sounds.first().map(|&(at, _)| at).into_iter().chain(lands).chain(self.goes_on_at()).min()
	}

	/// When the move goes on by itself: the game's `record_hold_s` after its last row came in (after the reveal's fold
	/// where it has no row, after the landing where neither), as a set saved with no pour does (the game's `Lc.GoOn`;
	/// Rapha, 2026-11-04: "yes try for now with auto progress"; slices 65 and 66 waited for Next until then), or after
	/// the last window over the pour closed where that came later (`cover`); none before what the save earned is known,
	/// nor while a window stands.
	pub(in crate::desk::pump) fn goes_on_at(&self) -> Option<i64> {
		let Over::Closed(closed) = self.over else { return None };
		let earned = self.view.earned.as_ref().filter(|_| self.view.landed)?;
		let last = earned.rows_at.last().copied().or_else(|| earned.reveal.as_ref().map(|r| r.folds));
		Some(last.unwrap_or(self.end).max(self.end).max(closed) + run::millis(facts().constants().record_hold_s))
	}

	/// A window came over the pour, or the last one over it went at `now` (`Pumps::window`; David, 2026-11-04: "yes and
	/// yes", QA-8: the "..."'s row stood where the next Ready's Start lands as the move went on under it).
	pub(in crate::desk::pump) fn cover(&mut self, open: bool, now: i64) {
		self.over = match (open, self.over) {
			(true, _) => Over::Window,
			(false, Over::Window) => Over::Closed(now),
			(false, over) => over,
		};
	}

	/// The pour ran its last grain by `now` and lands.
	pub(in crate::desk::pump) fn lands_by(&self, now: i64) -> bool {
		!self.view.landed && now >= self.end
	}

	/// The sounds due by `now`.
	pub(in crate::desk::pump) fn take_due(&mut self, now: i64) -> Vec<&'static str> {
		let due: Vec<&'static str> = self.sounds.iter().take_while(|&&(at, _)| at <= now).map(|&(_, e)| e).collect();
		self.sounds.drain(..due.len());
		due.into_iter().map(|e| self.fire(e)).collect()
	}

	/// An event as the one-jingle rule lets it play.
	fn fire(&mut self, event: &'static str) -> &'static str {
		let jingle = matches!(event, "record" | "unlock" | "unlockUncommon" | "done");
		self.jingled |= jingle;
		event
	}

	/// The pour at its end now (its last grain, a click, Next or Finish): the grains' sounds go, and a jingle of the
	/// pour's own that has not played plays now (none under `still`, Reduce Motion, whose one jingle comes with the
	/// rows: `earn`).
	pub(in crate::desk::pump) fn land(&mut self, now: i64, still: bool) -> Option<&'static str> {
		if self.view.landed {
			return None;
		}
		self.view.landed = true;
		self.still = still;
		self.end = self.end.min(now);
		self.sounds.clear();
		match self.jingle {
			Jingle::Grain(j) if !self.jingled && !still => Some(self.fire(j)),
			_ => None,
		}
	}

	/// What the save earned, from the app's own sheet after the write (`after`) less the one before it, from `now`: the
	/// move's row (`moved`), the steps it opened (a reveal for one of its own ladder, climbed or earned), the stats it
	/// raised and its XP; and the sound due at once under Reduce Motion (the move's one jingle, its strongest).
	pub(in crate::desk::pump) fn earn(
		&mut self,
		now: i64,
		after: Option<&Sum>,
		run: &Run,
		goals: &[(usize, i64)],
	) -> Option<&'static str> {
		let motion = &*MOTION;
		let m = run.moves.iter().find(|m| m.slot == self.slot)?;
		let sets = Sets {
			moves: std::slice::from_ref(m),
			recorded: &run.recorded,
			ran: &run.ran,
			beat: &run.beat,
			goals,
			nexts: &[],
			days: &run.days,
			after,
		};
		let mut rows = moved(&sets, &|m: &PlannedMove| run.max_set(m));
		let before = self.before.as_ref();
		let mut reveal = None;
		let mut next_bar = None;
		if let Some(after) = after {
			for (u, row) in unlocks(after, before) {
				if reveal.is_none() && !row.reward.quiet() {
					(reveal, next_bar) = self.reveal(now, u).unzip();
				}
				rows.push(row);
			}
			rows.extend(stats(after, before));
			let xp = after.last(0) - before.map_or(0, |b| b.last(0));
			if xp > 0 {
				rows.push(Row { reward: Reward::Xp { xp }, tile: None });
			}
		}
		let first = reveal.as_ref().map_or(now + motion.pour_hold, |r: &StepReveal| r.folds);
		let rows_at: Vec<i64> =
			(0..rows.len()).map(|i| first + i64::try_from(i).unwrap_or(0) * motion.stagger).collect();
		// the jingle that comes after the write: the reveal's as its step opens, else the top's, a record's the engine
		// judged (a weighted one, a score the pour cannot know) or done as the rows do
		let record = rows.iter().any(|r| matches!(r.reward, Reward::Record { .. }));
		let jingle = match self.jingle {
			_ if self.jingled => None,
			Jingle::Grain(j) => Some((first, j)),
			Jingle::After { .. } if reveal.is_some() => reveal.as_ref().map(|r| (r.opens, "unlockUncommon")),
			Jingle::After { top } => Some((
				first,
				if top {
					"unlock"
				} else if record {
					"record"
				} else {
					"done"
				},
			)),
		};
		self.view.earned = Some(SavedEarned { rows, rows_at, reveal, next_bar });
		// Reduce Motion, or a click that came before what the save earned: its end at once, with the jingle
		if self.still || self.hurried {
			self.land_all(now, self.still);
			return jingle.map(|(_, e)| self.fire(e));
		}
		self.sounds.extend(jingle);
		None
	}

	/// The reveal of unlock `u` (`[ladder, rung, move, quality, …]`) where it is a step of this move's own ladder, with
	/// the bar of the step it opened.
	fn reveal(&self, now: i64, u: &[i64]) -> Option<(StepReveal, CountBar)> {
		let motion = &*MOTION;
		let (&ladder, &rung, &hero, &quality) = (u.first()?, u.get(1)?, u.get(2)?, u.get(3)?);
		let (ladder, rung) = (u8::try_from(ladder).ok()?, u8::try_from(rung).ok()?);
		if ladder != self.ladder || rung <= self.rung {
			return None;
		}
		let (hero, quality) = (u16::try_from(hero).ok().filter(|&h| h > 0)?, u8::try_from(quality).ok()?);
		let rungs = facts().rungs(ladder);
		let last = rung.saturating_add(TILES / 2).min(rungs).max(rung);
		let first = last.saturating_sub(TILES - 1).max(1).min(self.rung);
		let (mut tiles, mut from, mut to) = (Vec::new(), None, None);
		for r in first..=last {
			let id = if r == rung { Some(hero) } else { facts().anchor(ladder, r) };
			let Some(id) = id else { continue };
			if r == self.rung {
				from = Some(tiles.len());
			}
			if r == rung {
				to = Some(tiles.len());
			}
			let quality = Some(facts().rung_quality(ladder, i64::from(r))).filter(|&q| q > 0);
			tiles.push(Tile { quality, ..tile(id) });
		}
		let (from, to) = (from?, to?);
		let colour = facts().rung_quality(ladder, i64::from(self.rung)) != quality;
		let at = now + motion.pour_hold;
		let step = i64::from(facts().step(ladder, rung)?);
		let reveal = StepReveal {
			at,
			opens: at + motion.reveal_from,
			folds: at + if colour { motion.reveal_end } else { motion.reveal },
			tiles,
			from,
			to,
			step,
			quality,
			word: facts().quality_word(quality)?,
			hero: Tile { quality: Some(quality), ..tile(hero) },
			colour,
			landed: false,
		};
		let w = facts().window(ladder, rung, hero);
		let posts = Posts { goal: w.map(|w| w.lo), best: None, top: w.map(|w| w.hi), seconds: self.view.bar.seconds };
		let bar = CountBar::of(ladder, rung, posts, 0);
		Some((reveal, bar))
	}

	/// A click on the stage once the pour landed: the reveal at its end and every row in now, with the jingle still to
	/// come played at once.
	pub(in crate::desk::pump) fn land_reveal(&mut self, now: i64) -> Vec<&'static str> {
		self.hurried |= self.view.earned.is_none();
		let due: Vec<&'static str> = self.sounds.drain(..).map(|(_, e)| e).collect();
		self.land_all(now, false);
		due.into_iter().map(|e| self.fire(e)).collect()
	}

	/// The reveal landed and the rows in from `now`, one after another (`still`, Reduce Motion: all at once).
	fn land_all(&mut self, now: i64, still: bool) {
		let stagger = if still { 0 } else { MOTION.stagger };
		if let Some(earned) = self.view.earned.as_mut() {
			if let Some(r) = earned.reveal.as_mut() {
				r.landed = true;
			}
			for (i, at) in earned.rows_at.iter_mut().enumerate() {
				*at = (*at).min(now + i64::try_from(i).unwrap_or(0) * stagger);
			}
		}
	}

	/// Next or Finish: the move's one jingle where none has played (the pour's own, else `done`).
	pub(in crate::desk::pump) fn leave(&mut self) -> Option<&'static str> {
		self.sounds.clear();
		if self.jingled {
			return None;
		}
		let event = match self.jingle {
			Jingle::Grain(j) => j,
			Jingle::After { .. } => "done",
		};
		Some(self.fire(event))
	}
}

/// A set at its step's top that may open the next step (the summary before the write, `before`): its strength
/// ladder's move that stands for the step, on that step, its days at the top one short of the climb's.
fn may_climb(m: &PlannedMove, ladder: u8, rung: u8, before: &Sum) -> bool {
	let Some(i) = usize::from(ladder).checked_sub(1) else { return false };
	let stands = before
		.lad
		.get(i)
		.is_some_and(|l| l.first().copied() == Some(i64::from(rung)) && l.get(1).copied() == Some(i64::from(m.r#move)));
	stands
		&& before.nx.get(i).is_some_and(|n| matches!(n.as_slice(), &[have, need, ..] if need > 0 && have + 1 >= need))
}
