//! The streak page on the computer (slice 51; docs/ui.md, "Away Mode", the streak page; the phones' `pump.streak`):
//! the streak as Pump home's board counts it (`home::Board`), its game months from the one the streak began in to this
//! one, each day by the board's mark, the rest tokens with the active days to the next one, the next of
//! `streak_milestones` and the best streak. It goes out with Pump home's topic (`home_publish`); the page steps
//! through the months it is given and sends nothing.

use questpump_core::link::Sheet;
use serde::Serialize;
use tauri::Manager as _;

use super::facts::facts;
use super::history::{civil, date_index, weekday};
use super::home::{Board, Mark};
use crate::runtime::Helper;

/// The streak page's topic.
pub(crate) const STREAK: &str = "streak";

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
	/// The next of `streak_milestones` and the one before it (0 before the first); none past the last.
	milestone: Option<Milestone>,
	best: i64,
	/// The game months from the one the streak began in (this one without a streak), the earliest first.
	months: Vec<MonthMarks>,
}

#[derive(Serialize, Debug)]
struct Milestone {
	from: i64,
	next: i64,
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

impl Helper {
	/// The streak page's topic from the sheet Hero shows and its board (`home_publish`): the board again with its marks
	/// back to the first of the month the streak began in, reached in steps that double while the run goes past them.
	pub(crate) fn publish_streak(&self, sheet: &Sheet, board: &Board, now: i64) {
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
		let next = tune.streak_milestones.iter().copied().filter(|&m| m > board.days).min();
		let from = tune.streak_milestones.iter().copied().filter(|&m| m <= board.days).max().unwrap_or(0);
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
			milestone: next.map(|next| Milestone { from, next }),
			best: marked.best,
			months,
		};
		self.app.state::<super::Desk>().publisher.publish(STREAK, &view);
	}
}
