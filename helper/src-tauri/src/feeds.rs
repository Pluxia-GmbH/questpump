//! The schedule both release feeds are checked on, QuestPump's own (`update.rs`) and the addon's (`addon.rs`), and the
//! limits their requests share. The first check comes `FIRST` after every start (a player's, at login, after an update,
//! after a crash's restart), or at a wake before it. After a check that succeeded the next is due `EVERY` later; after
//! one that failed, `RETRY`'s n-th wait later for the n-th failure in a row, then `RETRY_LAST` until one succeeds; each
//! wait spread by `SPREAD_OK` or `SPREAD_FAILED` either way, so computers that started together do not ask together.
//!
//! The waits run on the wall clock (`Due`), never on tokio's: on macOS its clock is `CLOCK_UPTIME_RAW`, which stops while
//! the Mac sleeps, so an hour there would be an hour awake. A task waits at most `LOOK` at a time (less when the due
//! time is nearer) and looks at the clock, so a computer that slept past the due time checks within `LOOK` of waking,
//! and a clock set back before the last check makes the next one due at once. The move films' download waits the same
//! way between its tries (`sleep`). A wake (`Checks::now`: the game closed, a game without QuestPump found, the
//! player's Add QuestPump to the game before a feed came) checks at once, unless a check succeeded less than `FRESH`
//! ago.

use std::sync::Arc;
use std::time::{Duration, SystemTime};

use tokio::sync::Notify;

const FIRST: Duration = Duration::from_secs(10);
const EVERY: Duration = Duration::from_hours(1);
const RETRY: [Duration; 6] = [
	Duration::from_secs(30),
	Duration::from_mins(1),
	Duration::from_mins(2),
	Duration::from_mins(5),
	Duration::from_mins(10),
	Duration::from_mins(15),
];
const RETRY_LAST: Duration = Duration::from_mins(30);
/// The share a wait is spread by, either way: ±10 % after a success, ±20 % after a failure.
const SPREAD_OK: f64 = 0.1;
const SPREAD_FAILED: f64 = 0.2;
/// The longest a task waits before it looks at the wall clock again.
const LOOK: Duration = Duration::from_mins(1);
/// A wake checks at once unless a check succeeded less than this ago.
const FRESH: Duration = Duration::from_mins(10);

/// The longest a feed's small file may take, from the connect to its last byte: the updater's `latest.json`, the
/// addon's `addon.json` and its signature. The big downloads (QuestPump's own installer or app, the addon's zip) have
/// no such bound, only `READ_TIMEOUT` and `CONNECT_TIMEOUT`, so a slow line still brings them.
pub const FILE_TIMEOUT: Duration = Duration::from_mins(2);
/// The longest a feed's request waits for its next bytes (each read; a stalled download ends, and the next check
/// comes).
pub const READ_TIMEOUT: Duration = Duration::from_mins(1);
pub const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);

/// A moment on the wall clock a wait ends at: passed once the clock reaches it, or once the clock is set back to
/// before the wait began.
#[derive(Debug, Clone, Copy)]
pub struct Due {
	from: SystemTime,
	at: SystemTime,
}

impl Due {
	/// `wait` from now.
	pub fn after(wait: Duration) -> Self {
		let from = SystemTime::now();
		Self { from, at: from.checked_add(wait).unwrap_or(from) }
	}

	pub fn passed(&self) -> bool {
		let now = SystemTime::now();
		now >= self.at || now < self.from
	}

	/// The time left until it passes on the wall clock now (none once it has passed).
	fn left(&self) -> Duration {
		if self.passed() { Duration::ZERO } else { self.at.duration_since(SystemTime::now()).unwrap_or_default() }
	}
}

/// Waits `wait` on the wall clock, looking at it at least every `LOOK` (a Mac that slept through it goes on within
/// `LOOK` of waking).
pub async fn sleep(wait: Duration) {
	let due = Due::after(wait);
	while !due.passed() {
		tokio::time::sleep(LOOK.min(due.left())).await;
	}
}

/// Wakes a feed's checks (`now`); its task waits on it between them (`first`, `Schedule::next`).
#[derive(Clone, Default)]
pub struct Checks(Arc<Notify>);

impl Checks {
	/// A check at once, or right after the one that runs, unless one succeeded less than `FRESH` ago.
	pub fn now(&self) {
		self.0.notify_one();
	}

	/// Waits for the first check (`FIRST` after the start, or a wake before it), then hands over the schedule of the
	/// checks after it.
	pub async fn first(&self) -> Schedule {
		let _ = tokio::time::timeout(FIRST, self.0.notified()).await;
		Schedule { wake: self.0.clone(), ok_at: None, failures: 0 }
	}
}

/// When a feed's next check is due.
pub struct Schedule {
	wake: Arc<Notify>,
	/// The wall clock of the last check that succeeded.
	ok_at: Option<SystemTime>,
	/// Checks that failed in a row.
	failures: usize,
}

impl Schedule {
	/// Waits after a check that succeeded (`ok`) or failed until the next one is due, or a wake asks for it.
	pub async fn next(&mut self, ok: bool) {
		let due = if ok {
			self.failures = 0;
			self.ok_at = Some(SystemTime::now());
			Due::after(spread(EVERY, SPREAD_OK))
		} else {
			self.failures += 1;
			let wait = RETRY.get(self.failures - 1).copied().unwrap_or(RETRY_LAST);
			Due::after(spread(wait, SPREAD_FAILED))
		};
		loop {
			let woken = tokio::time::timeout(LOOK.min(due.left()), self.wake.notified()).await.is_ok();
			if due.passed() || (woken && !self.fresh()) {
				return;
			}
		}
	}

	/// A check succeeded less than `FRESH` ago (never while the clock reads before it).
	fn fresh(&self) -> bool {
		self.ok_at.is_some_and(|at| SystemTime::now().duration_since(at).is_ok_and(|age| age < FRESH))
	}
}

/// `wait` spread by `share` either way.
fn spread(wait: Duration, share: f64) -> Duration {
	wait.mul_f64(rand::random_range(1.0 - share..=1.0 + share))
}
