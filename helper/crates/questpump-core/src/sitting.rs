//! The sitting clock (docs/computer-games.md §3; docs/plan.md slices 48 and 57): the minutes the player has sat at
//! this computer, one clock for the desktop app's sitting cue (`desk/reminders.rs`) and its match offers' warm first
//! move (`offers::Tunables::warm`: since slice 57 it gates and sizes no offer, which every match makes at the size its
//! own minutes give, `tier`). It runs from the latest of the newest performed move on any device, the latest game
//! offer on any device (the sitting cue's alone: an offer is a stand-up ask itself, while the warm move reads the clock
//! without the offers, so an offer let pass stands nobody up) and the end of a pause of `away_s` in this computer's
//! input, which also bounds a match's minutes (`offers::Gate::back_at`), and from the game day's start (04:00) only
//! where the input's idle time never read in this run (since slice 48's double-check: macOS and Windows always read
//! it, so a session across 04:00 counts whole). The app reads the input's idle time on every look
//! (`runtime/sitting.rs`: Quartz's HID state on macOS, `GetLastInputInfo` on Windows, neither behind a permission) and
//! hands it to `Sitting::look`: an idle time that does not read is never a pause, a look that comes more than `away_s`
//! after the one before (the computer slept) is one, a look earlier than the one before (the clock set back) starts the
//! clock again, and while a watched game's window is in front or a match runs its idle time counts as fresh, since a
//! game played on a pad or with raw input may leave the system's counter standing (a game left open behind other
//! windows lets a pause count), but only while the system's own idle time is under `PLAYING_CAP_MS`'s 3 hours (the
//! sync's `playing` cap, `games.rs`): a game left in front overnight or through a day away ends as a pause. Nothing of
//! a game is read: the queue, the pick phase and the loading screen count, as the player sits through them too.

use crate::games::PLAYING_CAP_MS;
use crate::library;

/// An input pause this long (seconds, library.json's `sit_away_s`, slice 67: Rust's `AWAY_S` until then) is a time
/// away from the computer: the clock starts again at its end. Work mode takes it as nobody being there for a due
/// pump's opening (the desk's `work_open`), never as a pause of its clock, which always runs down.
/// Never when the built-in constants do not read (`make check` keeps them readable): the clock then runs from its first
/// look and the moves alone, and work mode asks nothing (`work::tunables`).
#[must_use]
pub fn away_s() -> i64 {
	library::constants().map_or(i64::MAX, |c| c.sit_away_s)
}

/// The input's part of the clock, on the helper's loop.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Sitting {
	/// The end of the latest pause, unix seconds: the first look of this run (or the clock a quit kept), else the
	/// latest look in a pause, after a sleep or after the clock was set back; none before the first look.
	since: Option<i64>,
	/// The last look, unix seconds.
	looked: Option<i64>,
	/// The last look a watched game counted as input at (its window in front, or a match running, while the system's
	/// idle time was under `PLAYING_CAP_MS`), unix seconds.
	game_seen: Option<i64>,
	/// The last look's idle time as the system gave it (whole seconds, no game counted in): the match offer's card
	/// asks whether the player left the desk by it (`offers::Tunables::away_s`).
	idle_s: Option<u32>,
	/// The idle time read at a look of this run: the clock then needs no bound at 04:00.
	read: bool,
}

impl Sitting {
	/// The clock a quit at `quit_at` kept (its `since` and `game_seen`), at a start at `now`: taken on when the start
	/// came within `away_s` of the quit (an update, Quit and reopen), else a new clock that starts at the first look.
	/// The last look a watched game counted at goes on with it, so a match played on a pad, which leaves the system's
	/// idle time standing, is no pause at the start's first look (2026-10-29).
	#[must_use]
	pub fn restored(since: i64, game_seen: Option<i64>, quit_at: i64, now: i64) -> Self {
		let kept = since <= quit_at && (0..away_s()).contains(&(now - quit_at));
		let game_seen = game_seen.filter(|&at| kept && at <= quit_at);
		Self { since: kept.then_some(since), game_seen, ..Self::default() }
	}

	/// The last look a watched game counted as input at (unix seconds), as a quit keeps it (`restored`).
	#[must_use]
	pub fn game_seen(&self) -> Option<i64> {
		self.game_seen
	}

	/// The end of the latest pause (unix seconds), as a quit keeps it (`restored`) and as no match counts minutes from
	/// before it (`offers::Gate::back_at`).
	#[must_use]
	pub fn kept(&self) -> Option<i64> {
		self.since
	}

	/// The idle time the last look read, as the system gave it (none before the first look and where the system gives
	/// no number).
	#[must_use]
	pub fn idle_s(&self) -> Option<u32> {
		self.idle_s
	}

	/// One look at unix second `now`: `idle_s` the whole seconds since this computer's last input (none where the
	/// system gives no number), `game_fresh` whether a watched game counts as input now (its window in front, or a
	/// match running), which it does only while `idle_s` is under `PLAYING_CAP_MS` (a game left in front with nobody at
	/// the computer, overnight or through a day away). A pause starts the clock again at its end: an idle time of
	/// `away_s` (counted from the later of the last input and the last look a watched game counted at), or a look
	/// `away_s` after the one before; so does a look earlier than the one before (the clock was set back, and what the
	/// clock knew lies ahead of now).
	pub fn look(&mut self, now: i64, idle_s: Option<u32>, game_fresh: bool) {
		let back = self.looked.is_some_and(|at| now < at);
		if back {
			self.game_seen = None;
		}
		let left = idle_s.is_some_and(|s| i64::from(s) * 1_000 >= PLAYING_CAP_MS);
		if game_fresh && !left {
			self.game_seen = Some(now);
		}
		let away = away_s();
		let slept = self.looked.is_some_and(|at| now - at > away);
		self.looked = Some(now);
		self.idle_s = idle_s;
		self.read |= idle_s.is_some();
		let idle = idle_s.map(|s| {
			let s = i64::from(s);
			self.game_seen.map_or(s, |seen| s.min(now - seen))
		});
		if back || slept || idle.is_some_and(|s| s >= away) || self.since.is_none() {
			self.since = Some(now);
		}
	}

	/// The seconds sat at `now`: since the latest of the end of the latest pause, the newest performed move on any
	/// device (`last_move`) and the latest game offer on any device (`last_offer`: none for a match's warm move, which
	/// no offer restarts), all unix seconds, and the game day's start (`day_start`) only while no look of this run read
	/// the idle time; another device's time later than `now` (its clock ahead) starts nothing, and this computer's own
	/// pause end later than `now` (its clock set back since) is `now`.
	#[must_use]
	pub fn sat_s(&self, now: i64, last_move: Option<i64>, last_offer: Option<i64>, day_start: i64) -> i64 {
		let own = self.since.map(|at| at.min(now));
		let devices = [last_move, last_offer].into_iter().flatten().filter(|&at| at <= now);
		let bound = (!self.read).then_some(day_start);
		own.into_iter().chain(devices).chain(bound).max().map_or(0, |since| (now - since).max(0))
	}
}

/// The whole minutes in `seconds` (none below 0): the sitting clock's, or a match's.
#[must_use]
pub fn minutes(seconds: i64) -> u16 {
	u16::try_from(seconds.max(0) / 60).unwrap_or(u16::MAX)
}

/// The pump's size after a match of `minutes` (slice 57; the minutes sat until then): one step per `step_min` minutes
/// (`match_step_min`), rounded half up, from 1 (Light) to 3 (Extreme): under 18 minutes Light, 18-29 Balanced, 30 or
/// more Extreme at 12 a step (since slice 59). The phones' `PhoneSessions.tier` is the same rule over the matches a player names
/// (contract/fixtures/phone.json).
#[must_use]
pub fn tier(minutes: i64, step_min: i64) -> u8 {
	let step = step_min.max(1);
	match (2 * minutes.max(0) + step) / (2 * step) {
		..=1 => 1,
		2 => 2,
		_ => 3,
	}
}
