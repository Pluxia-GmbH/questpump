//! The streak page on the computer (slice 51; docs/ui.md, "Away Mode", the streak page; the phones' `pump.streak`):
//! the streak as Pump home's board counts it (`home::Board`), its game months from the one the streak began in to this
//! one, each day by the board's mark, the rest tokens with the active days to the next one, the next of
//! `streak_milestones` and the best streak. It goes out with Pump home's topic (`home_publish`); the page steps
//! through the months it is given and sends nothing.

use questpump_core::body::Body;
use questpump_core::link::Sheet;
use serde::Serialize;
use serde_json::Value;

use super::facts::facts;
use super::history::{civil, date_index, weekday};
use super::home::{Board, Mark};
use super::menu_pages::setup::stamped;
use crate::{Desk, Intent, Platform};

/// The streak page's topic.
pub const STREAK: &str = "streak";

/// The `streak` topic.
#[derive(Serialize, Debug)]
struct StreakView {
	days: i64,
	/// Today's game day has a performed move.
	today: bool,
	/// Felt unwell locks the pumps.
	resting: bool,
	tokens: i64,
	token_max: i64,
	token_days: i64,
	/// The active days to the next rest token; none while the most are held.
	next_token: Option<i64>,
	/// What the streak's day stands against (`Board::against`, slice 74): the player's goal while it is on or ahead
	/// of the day, else the next of `streak_milestones`, and the milestone before it (0 before the first); none past
	/// the last with no goal ahead.
	milestone: Option<Milestone>,
	/// The page's row of goals, as the phones' streak pages have it: `goal` the one that stands, the player's while it
	/// is on or ahead of the day, else 0 (None); `steps` the ones to choose from after None, those of
	/// `streak_goal_steps` above the day and the standing goal itself, a step or not (21 from another build).
	goal: i64,
	steps: Vec<i64>,
	best: i64,
	/// The game months from the one the streak began in (this one without a streak), the earliest first.
	months: Vec<MonthMarks>,
}

#[derive(Serialize, Debug)]
struct Milestone {
	from: i64,
	next: i64,
	/// It is the player's own goal, not a milestone's turn.
	own: bool,
}

/// A game month: its days' marks from its first, `lead` days of its first week before it (0 Monday … 6 Sunday), and
/// today's day of it while it is this month (the days after it are to come).
#[derive(Serialize, Debug)]
struct MonthMarks {
	year: i32,
	month: u8,
	lead: i64,
	today: Option<i64>,
	days: Vec<Mark>,
}

/// A game day's year and month, and the game day of that month's first.
fn month_of(d: i64) -> Option<(i32, u8, i64)> {
	let day = civil(d);
	let mut parts = day.splitn(3, '-').map(str::parse::<i32>);
	let (Some(Ok(y)), Some(Ok(m))) = (parts.next(), parts.next()) else { return None };
	let month = u8::try_from(m).ok()?;
	Some((y, month, date_index(&format!("{y:04}-{month:02}-01"))?))
}

/// The game day of the month after the one starting on `first`.
fn next_month(first: i64) -> Option<i64> {
	month_of(first + 31).map(|(_, _, f)| f)
}

/// The first game day of the run the board counts: back from today (from the day before while today is open) over the
/// days it marks, no further than `floor`; today without a streak.
fn run_start(board: &Board, today: i64, floor: i64) -> i64 {
	if board.days == 0 {
		return today;
	}
	let mut d = if board.mark(today) == Mark::Open { today - 1 } else { today };
	while d > floor && board.mark(d) != Mark::Open {
		d -= 1;
	}
	d + 1
}

impl<P: Platform> Desk<P> {
	/// The streak's goal in days (slice 74; the body's `streak_goal`, which every device shares), none before any
	/// device chose one and once the player chose none: what the streak page shows and `Board::against` takes.
	#[must_use]
	pub fn streak_goal(&self) -> Option<i64> {
		self.streak_goal_said().filter(|&g| g > 0)
	}

	/// What the player said of the streak's goal, on any device (the body's `streak_goal` as it stands): none while
	/// nothing was said, 0 once a None was said (with its time), else the goal's days. `Board::goal_steps` offers by
	/// it; `Board::against` reads none and 0 alike.
	#[must_use]
	pub fn streak_goal_said(&self) -> Option<i64> {
		self.body.body.streak_goal.as_ref().map(|g| i64::from(g.value))
	}

	/// The streak's goal chosen on this computer, a change of the body made now (`body_changed`): `days` one of
	/// library.json's `streak_goal_steps`, or 0 for None, the player's word too (the streak page's None, a Result ended
	/// with its choices shown and none picked), which is written with its time where nothing was said yet. What stands
	/// said again changes nothing.
	///
	/// # Errors
	/// A number of days that is no step.
	pub fn set_streak_goal(&mut self, days: u16) -> Result<(), String> {
		// what stands said again, the goal of another build that is no step here too (the streak page's own chip)
		if self.streak_goal_said() == Some(i64::from(days)) {
			return Ok(());
		}
		if days != 0 && !facts().constants().streak_goal_steps.contains(&i64::from(days)) {
			return Err(format!("no streak goal of {days} days"));
		}
		let changed = stamped(days, self.body.body.streak_goal.as_ref(), self.p.now_ms());
		self.body_changed(&Body { streak_goal: Some(changed), ..Body::default() });
		Ok(())
	}

	/// The body changed (this computer's own choice, a phone's exchange, the sync's stamps, the game's body; each
	/// merges, saves and calls `profile_changed`): where what the player said of the streak's goal is another than
	/// what shows it last read (a None in nothing's place too, which takes a Result's choices away as a goal ahead
	/// does), a pick the Result on screen holds under choices that went with it is written first
	/// (`pump_goal_overtaken`: the player's latest word, stamped past what arrived), and Pump home with the streak
	/// page and the pump's Result read the goal again.
	pub(crate) fn streak_goal_read(&mut self) {
		let goal = self.streak_goal_said();
		if self.home.goal_read == goal {
			return;
		}
		self.home.goal_read = goal;
		self.pump_goal_overtaken();
		// the pick written: that change read the goal again and published it
		if self.streak_goal_said() != goal {
			return;
		}
		self.home_publish();
		self.pump_publish();
	}

	/// The streak's intents: `streak.goal` with `days`, the goal the player chose on a Result's streak moment or on
	/// the streak page's row (slice 74; 0 for none there), set through `set_streak_goal` and shown at once.
	///
	/// # Errors
	/// Why the intent was refused.
	pub fn streak_intent(&mut self, intent: &Intent) -> Result<(), String> {
		match intent.kind() {
			"streak.goal" => {
				let days = intent
					.arg("days")
					.and_then(Value::as_u64)
					.and_then(|d| u16::try_from(d).ok())
					.ok_or("streak.goal takes days")?;
				self.set_streak_goal(days)?;
				self.home_publish();
				Ok(())
			}
			_ => Err(intent.unknown()),
		}
	}

	/// The streak page's topic from the sheet Hero shows and its board (`home_publish`): the board again with its marks
	/// back to the first of the month the streak began in, reached in steps that double while the run goes past them.
	pub fn publish_streak(&self, sheet: &Sheet, board: &Board, now: i64) {
		let tune = facts().constants();
		let today = self.home.days.day.index(now);
		let mut reach = board.days + 7;
		let (marked, start) = loop {
			let floor = month_of(today - reach).map_or(today - reach, |(_, _, first)| first);
			let marked = self.board_from(sheet, now, floor);
			let start = run_start(&marked, today, floor);
			if start > floor || reach > board.days * 2 + 400 {
				break (marked, start);
			}
			reach *= 2;
		};
		let mut months = Vec::new();
		let mut first = month_of(start).map(|(_, _, f)| f);
		while let Some(f) = first.filter(|&f| f <= today) {
			let (Some((year, month, _)), Some(next)) = (month_of(f), next_month(f)) else { break };
			let days = (f..next).map(|d| marked.mark(d)).collect();
			let on = (today < next).then_some(today - f + 1);
			months.push(MonthMarks { year, month, lead: weekday(f), today: on, days });
			first = Some(next);
		}
		let goal = self.streak_goal();
		let standing = goal.filter(|&g| g >= board.days);
		let mut offered: Vec<i64> =
			tune.streak_goal_steps.iter().copied().filter(|&n| n > board.days).chain(standing).collect();
		offered.sort_unstable();
		offered.dedup();
		let next = Board::against(board.days, goal);
		let own = next.is_some() && next == goal;
		// the bar's start: the milestone before what the day stands against
		let below = |n: i64| tune.streak_milestones.iter().copied().filter(|&m| m < n && m <= board.days).max();
		let from = next.and_then(below).unwrap_or(0);
		let full = board.tokens >= tune.token_max || tune.token_days <= 0;
		let next_token = board.counted.filter(|_| !full).map(|c| tune.token_days - c.rem_euclid(tune.token_days));
		let view = StreakView {
			days: board.days,
			today: board.today,
			resting: sheet.locked_until.is_some(),
			tokens: board.tokens,
			token_max: tune.token_max,
			token_days: tune.token_days,
			next_token,
			milestone: next.map(|next| Milestone { from, next, own }),
			goal: standing.unwrap_or(0),
			steps: offered,
			best: marked.best,
			months,
		};
		self.publisher.publish(STREAK, &view);
	}
}
