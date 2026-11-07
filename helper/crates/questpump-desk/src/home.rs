//! Pump home on the computer (docs/ui.md, "Pump home on the computer"; slice 32's Pump home, iOS `PumpTab`, Android's
//! twin): led by the streak (`Board`, a port of the core's `StreakBoard` over the rows the helper holds: every device's
//! app rows and the game's rows, the same game days as the phones), this computer's status Chip only while something
//! needs fixing (`Fix`: the page words it from This computer's state, `desk::tray::state`, or its own line), the recent
//! games with their moves this week (`desk::recent`, 2026-10-27), the rest card while the felt-unwell
//! lock holds, else the moves waiting for a catch-up or today's chapter (the Saga's day, `desk::log`'s
//! `TodayChapter`: one number from one owner), the day-2 check-in, one card that
//! offers the next part (`next_card`: pairing a phone while none is paired and the phone apps are in the stores, else
//! adding the addon while the game is here without QuestPump), and the foot's one gold primary in
//! this order: none while the lock holds or the engine failed (the foot says pumps could not start, beside the Menu's
//! report for help); a
//! setup step the player has started (the Screen Recording grant once a World of Warcraft install is found and a phone
//! is paired; on Windows the firewall's step once the player started it on the Phones page); Catch up while a move
//! waits that has waited at most `owed_fresh_days` game days (older ones stay listed, each with its Skip, and Catch up
//! stays a quiet button);
//! the match's offer while it stands (`Standing`), whose words stay above the foot under another primary, its Start the
//! second quiet button; else Quick pump, with a quiet Start a pump. While a World of Warcraft client with QuestPump
//! runs (`wow_asks_pumps`) it starts no pump and says the game asks them; while its squares have not shown yet
//! (`wow_unseen`) the app pumps start here, and Catch up still waits for the game
//! (`wow_with_addon`); the pumps themselves are DESK-PUMP's (the page sends `pump.quick`, `pump.choose` for the Start
//! a pump window and `pump.catchup`).
//!
//! Rust owns the state: the rows' game days (`history.rs`'s `GameDay`) are kept as the stores grow (`Days`, read
//! again from the start after a reset), today's chapter is the Saga's last build (`LogState::home`), and the app's own
//! fold's summary, records, check-in, pain flags, Book rows, titles earned and today's XP (the Saga's XP tile's) are
//! asked of
//! the engine through its ordered door once rows or a sheet came, the game day turned, the profile changed or a fold
//! left them stale (`Home::dirty`; an ask that failed is asked again at a later look, later after each failure in a
//! row). The topics go out at every change of the rows and on the loop's look every 2 s (the publisher sends only a
//! topic that changed), with Hero's (`hero.rs`) and the streak page's (`streak.rs`).
//!
//! The `home` topic is `HomeView`. Intents: `home.checkin {answer}` (1 fine, 2 sore, 3 very sore or dark urine; an
//! answer the engine did not write brings the card back) and `home.pain {joints}` (the joints the check-in's last
//! answer names; a write the engine failed comes back in `pain_failed`, which opens the window again with them ticked,
//! until a Save writes them or puts the window away with none), each refused while the engine folds; `home.later {card:
//! "pair"|"addon"}` (that card's Not now: it shows again in `LATER_S`); `home.skip {move, ts}` (a waiting move's Skip,
//! since 2026-10-09: `Desk::pump_skip_owed`); `home.span {span: "week"|"all"}` (the recent games' This week | All
//! time, kept in settings.json's `desk.played_span`, 2026-10-28). The games found are the Games tab's
//! (`desk/offers.rs`, slice 56): Pump home neither offers them nor asks about one.

use std::collections::{BTreeSet, HashMap};

use questpump_core::app_rows::after_start_over;
use questpump_core::codes::{kind, outcome};
use questpump_core::games::MATCH;
use questpump_core::link::{OwedMove, Sheet};
use questpump_core::protocol::RowRecord;
use questpump_engine::{BookLadder, Embers, GameLadders, GameSummary, RidesAs, SetAside};
use serde::Serialize;
use serde_json::Value;

use super::facts::{CheckinWords, PainWords, facts};
use super::hero::{ChartsKey, GameSheet, Summary};
use super::history::{self, GameDay};
use super::log::TodayChapter;
use super::recent::{Recent, RecentGames};
use super::seams::StoreMark;
use crate::platform::{EngineState, Fix, FoundGame, Step};
use crate::settings::PlayedSpan;
use crate::{Desk, Intent, Platform};

/// Pump home's topic.
pub const HOME: &str = "home";

/// What the log calls the rows Pump home keeps.
const CHECKIN_ROW: &str = "check-in"; // copy-lint: not shown (a log line's word)
const PAIN_ROW: &str = "pain flag"; // copy-lint: not shown (a log line's word)

/// The game days the rows the helper keeps hold, as `GameDay.index` counts them (days since 1970-01-01 of the local date
/// 4 hours earlier), kept as the stores grow.
pub struct Days {
	/// How far the stores were read (`Desk::store_mark`).
	read: StoreMark,
	/// Days with a performed move, every source (`AgnosticEngine.activeDays`).
	active: BTreeSet<i64>,
	/// Days the app's own rows hold a performed move on, every app device's (`ownActiveDays`).
	own: BTreeSet<i64>,
	/// Days with a row that locks for feeling unwell (`lockedDays`).
	locked: BTreeSet<i64>,
	/// The moves done in any row (the core's `EngineState.seen`): Hero's ladder lines pick a muscle's move by them.
	pub done: BTreeSet<u16>,
	/// The computer's game days (the history's, the Log's and the charts' one rule).
	pub day: GameDay,
}

impl Days {
	/// No day read yet, the game days in the zone `zone` (`Platform::zone`).
	fn new(zone: fn(i64) -> i64) -> Self {
		Self {
			read: StoreMark::default(),
			active: BTreeSet::new(),
			own: BTreeSet::new(),
			locked: BTreeSet::new(),
			done: BTreeSet::new(),
			day: GameDay::new(facts().constants().day_offset_h, zone),
		}
	}

	/// The rows the stores took since the last look (the other computers' from the one `far` is asked for on,
	/// `Session::far_records`), or every row again after a reset, a new Start over (the app's own days start at it,
	/// `app_rows::after_start_over`; every source's keep the app days before it, which the game keeps) or another
	/// computer's characters named anew (their rows kept before take their places).
	fn take(
		&mut self,
		game: &[RowRecord],
		own: &[RowRecord],
		far: impl FnOnce(usize) -> Vec<RowRecord>,
		mark: StoreMark,
	) {
		if mark.floor != self.read.floor
			|| mark.start_over != self.read.start_over
			|| mark.far_named != self.read.far_named
			|| mark.game < self.read.game
			|| mark.own < self.read.own
			|| mark.far < self.read.far
		{
			for set in [&mut self.active, &mut self.own, &mut self.locked] {
				set.clear();
			}
			self.done.clear();
			self.read = StoreMark { floor: mark.floor, start_over: mark.start_over, ..StoreMark::default() };
		}
		let f = facts();
		let (g, o) = (self.read.game, self.read.own);
		let far = far(self.read.far);
		for (r, mine) in game
			.get(g..)
			.unwrap_or_default()
			.iter()
			.map(|r| (r, false))
			.chain(own.get(o..).unwrap_or_default().iter().map(|r| (r, true)))
			.chain(far.iter().map(|r| (r, true)))
		{
			let row = r.row();
			if row.locks() {
				self.locked.insert(self.day.index(i64::from(row.ts)));
			}
			if !kind::is_move(row.kind) || !outcome::is_performed(row.outcome) || !f.knows(row.move_id) {
				continue;
			}
			let day = self.day.index(i64::from(row.ts));
			self.active.insert(day);
			if mine && after_start_over(&row, mark.start_over) {
				self.own.insert(day);
			}
			self.done.insert(row.move_id);
		}
		self.read = mark;
	}

	/// The last game day with a performed move, every source's (the reminders' taper after an absence).
	pub fn last_active(&self) -> Option<i64> {
		self.active.last().copied()
	}
}

/// A day of the streak's week as its marks show it (the core's `StreakBoard.Mark`, the week dots' days state;
/// 2026-10-09): pumped, kept by a rest token (one the fold spent, or one the tokens hold while today is still open),
/// skipped by the streak after feeling unwell with no pump, or none of them; the page reads its lower-case name.
#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Mark {
	Open,
	Done,
	Rested,
	Resting,
}

/// The streak at a moment (the core's `StreakBoard.make`): a summary's streak with the active days it has not folded
/// yet (those after its last active day, `rt`), a gap its rest tokens cover spending them and a wider one starting
/// again at 1, the locked days skipped, a token earned at each active day that makes the summary's count of them a
/// multiple of `token_days` (`token_max` held at most), then broken where the gap to today is wider than the tokens.
/// The week's marks as the core's: a performed move's day done; a locked day without one resting; rested a day the
/// tokens covered as the board counted it, one they hold while today is open, and before the summary's last active
/// day one the run its streak counts back from there spans.
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Board {
	pub days: i64,
	pub tokens: i64,
	/// Today's game day has a performed move.
	pub today: bool,
	/// This game week's days from Monday, each as its mark shows it.
	pub week: [Mark; 7],
	/// Today's place in `week`, 0 Monday … 6 Sunday.
	pub today_index: i64,
	/// The streak one pump today would make.
	pub after_pump: i64,
	/// The days a rest token kept, as far back as the board was asked to reach (this week's Monday at the least): the
	/// Saga's week strip reads them by date (its game week may begin before this Monday, at the region's reset).
	#[serde(skip)]
	pub rested: BTreeSet<i64>,
	/// The days with a performed move and the resting days, as far back as `rested` (the streak page's month).
	#[serde(skip)]
	pub pumped: BTreeSet<i64>,
	#[serde(skip)]
	pub resting: BTreeSet<i64>,
	/// The fold's count of active days that earns the tokens, as the board counted it; none from a summary before `ad`.
	#[serde(skip)]
	pub counted: Option<i64>,
	/// The longest streak the days made (`best`), at least this one.
	#[serde(skip)]
	pub best: i64,
}

impl Board {
	/// The board on game day `today` (the core's `StreakBoard.make`, its `now` as its game day), its rested days marked
	/// back to `from` where that is before this week's Monday; `day_of` the game day of a unix second.
	pub fn make(
		summary: Option<&Summary>,
		active: &BTreeSet<i64>,
		locked: &BTreeSet<i64>,
		today: i64,
		from: i64,
		day_of: impl Fn(i64) -> i64,
	) -> Self {
		let tune = facts().constants();
		let mut active = active.clone();
		let (mut days, mut tokens) = (0, 0);
		// the fold's count of active days, which earns the tokens; unknown in a summary from before `ad`
		let mut counted = Some(0);
		let mut last: Option<i64> = None;
		let mut skipped = locked.clone();
		if let Some(s) = summary {
			days = if s.streak > 0 { s.streak } else { s.raw_streak.unwrap_or(0) };
			(tokens, counted) = (s.tokens, s.active_days);
			let anchor = day_of(s.rt.unwrap_or(s.at));
			if s.rt.is_some() || s.streak > 0 {
				last = Some(anchor);
			}
			skipped.retain(|&d| d > anchor);
			skipped.extend(s.locked_days.iter().map(|d| anchor + d));
			if let Some(rt) = s.rt {
				active.insert(day_of(rt));
			}
		}
		let resting: BTreeSet<i64> = locked.iter().chain(&skipped).copied().filter(|d| !active.contains(d)).collect();
		skipped.retain(|d| !active.contains(d));
		let today_index = (today + 3).rem_euclid(7);
		let monday = today - today_index;
		let reach = from.min(monday);
		let mut rested = BTreeSet::new();
		if let Some(anchor) = last {
			// the run the summary's streak counts back from its last active day, as far as the board reaches
			let (mut left, mut d) = (days - 1, anchor - 1);
			while left > 0 && d >= reach {
				if !resting.contains(&d) {
					if !active.contains(&d) {
						rested.insert(d);
					}
					left -= 1;
				}
				d -= 1;
			}
		}
		// ascending, so each day is past the one before it as it is past the summary's last
		let after: Vec<i64> = active.iter().copied().filter(|&d| last.is_none_or(|l| d > l) && d <= today).collect();
		for d in after {
			let held = tokens;
			(days, tokens) = step(days, tokens, last, d, &skipped);
			// a gap the tokens covered: its free days rested
			if let Some(l) = last
				&& tokens < held
			{
				rested.extend((l + 1..d).filter(|x| !skipped.contains(x)));
			}
			if let Some(c) = counted {
				let now = c + 1;
				if tune.token_days > 0 && now % tune.token_days == 0 && tokens < tune.token_max {
					tokens += 1;
				}
				counted = Some(now);
			}
			last = Some(d);
		}
		let done = active.contains(&today);
		let mut after_pump = days;
		if !done {
			after_pump = step(days, tokens, last, today, &skipped).0;
			if let Some(l) = last {
				let gap = free(l, today, &skipped);
				if gap > tokens {
					days = 0;
				} else if gap > 0 {
					// the tokens hold the gap while today is open
					rested.extend((l + 1..today).filter(|x| !skipped.contains(x)));
				}
			}
		}
		rested.retain(|d| *d >= reach && !active.contains(d) && !resting.contains(d));
		let best = best(&active, &resting, today).max(days);
		let (pumped, resting) = (active.range(reach..).copied().collect(), resting.range(reach..).copied().collect());
		let mut board = Self {
			days,
			tokens,
			today: done,
			week: [Mark::Open; 7],
			today_index,
			after_pump,
			rested,
			pumped,
			resting,
			counted,
			best,
		};
		board.week = std::array::from_fn(|i| board.mark(monday + i64::try_from(i).unwrap_or(0)));
		board
	}

	/// Game day `d`'s mark, for a day the board reaches (`rested`): a performed move's done, else a locked day resting,
	/// else one a rest token kept rested.
	#[must_use]
	pub fn mark(&self, d: i64) -> Mark {
		if self.pumped.contains(&d) {
			Mark::Done
		} else if self.resting.contains(&d) {
			Mark::Resting
		} else if self.rested.contains(&d) {
			Mark::Rested
		} else {
			Mark::Open
		}
	}
}

/// The longest streak the active days up to `today` made (Stats.lua's `best`, which no summary carries): `step` over
/// each from the first, `token_start` tokens at first and one earned every `token_days` active days (`token_max` held
/// at most), the `resting` days skipped.
fn best(active: &BTreeSet<i64>, resting: &BTreeSet<i64>, today: i64) -> i64 {
	let tune = facts().constants();
	let (mut days, mut tokens, mut best) = (0, tune.token_start, 0);
	let mut last = None;
	for (n, &d) in (1_i64..).zip(active.range(..=today)) {
		(days, tokens) = step(days, tokens, last, d, resting);
		if tune.token_days > 0 && n % tune.token_days == 0 && tokens < tune.token_max {
			tokens += 1;
		}
		best = best.max(days);
		last = Some(d);
	}
	best
}

/// What a pump that turned its game day done reached, from the board as it began (`self`) and as it ended: a paired
/// player's app pump and every catch-up (the game folds their rows at its next load).
impl Board {
	/// The streak milestone (the core's `StreakBoard.milestone`, D24): the largest of `streak_milestones` past the streak
	/// before and within the streak after; 0 for none, and for a pump on a day done already.
	#[must_use]
	pub fn milestone(&self, after: &Self) -> i64 {
		if self.today || !after.today {
			return 0;
		}
		let past = |&&m: &&i64| self.days < m && m <= after.days;
		facts().constants().streak_milestones.iter().filter(past).max().copied().unwrap_or(0)
	}

	/// The rest tokens that covered the game days before it (`pump.reward.rested`): those one pump would spend as it
	/// began (the streak it made over the streak before and its own day), where it made that streak; 0 for none.
	#[must_use]
	pub fn rested(&self, after: &Self) -> i64 {
		if self.today || !after.today || self.days == 0 || after.days != self.after_pump {
			return 0;
		}
		(self.after_pump - self.days - 1).max(0)
	}
}

/// The streak and the tokens after a performed move on game day `d` (Stats.lua activeDay), the last active day `last`:
/// one more across a gap the locked days fill, the free days rested on tokens that cover them, else 1.
fn step(days: i64, tokens: i64, last: Option<i64>, d: i64, locked: &BTreeSet<i64>) -> (i64, i64) {
	let Some(last) = last else { return (1, tokens) };
	let free = free(last, d, locked);
	if free <= 0 {
		(days + 1, tokens)
	} else if free <= tokens {
		(days + free + 1, tokens - free)
	} else {
		(1, tokens)
	}
}

/// The game days strictly between `last` and `d` that no locked day fills (none, or fewer than none, where `d` is not
/// past the day after `last`).
fn free(last: i64, d: i64, locked: &BTreeSet<i64>) -> i64 {
	let between = if d > last + 1 { locked.range(last + 1..d).count() } else { 0 };
	d - last - 1 - i64::try_from(between).unwrap_or(0)
}

/// What the engine's fold says for Pump home and Hero, asked through its door.
struct Asked {
	summary: Option<Summary>,
	records: Vec<questpump_engine::Record>,
	checkin: bool,
	pain: Vec<u8>,
	/// The engine's Book rows and what of the game's sheet they set aside; none where the Book did not answer.
	book: Option<(Vec<BookLadder>, SetAside)>,
	titles: Vec<u8>,
	today_xp: i64,
	/// Today's embers on the app's own sheet and the week's omen (slice 55); none where the engine did not answer.
	embers: Option<Embers>,
	/// The unfinished moves that ride as another move (`Engine::rides_as`); none where the engine did not answer.
	rides: Option<Vec<RidesAs>>,
	/// The game's save as the game folds it, asked only while the game's sheet the app shows needs it.
	game: Option<GameSummary>,
}

/// The longest wait before an ask that failed is asked again, in seconds.
const RETRY_MAX_S: i64 = 300;

/// How long Not now puts the card that pairs a phone or the one that offers the addon away: a week.
const LATER_S: i64 = 7 * 24 * 60 * 60;

/// Pump home's and Hero's state between looks (a field of [`Desk`]).
#[expect(clippy::struct_excessive_bools, reason = "the loop's marks between looks, each a yes or no of its own")]
pub struct Home {
	pub days: Days,
	/// How many sheets the game's log held at the last look: a new one asks the engine's Book rows again.
	sheets: usize,
	/// The app's own fold's summary, its last answer.
	pub summary: Option<Summary>,
	/// The engine's fold of the game's save, its last answer while the game's sheet the app shows needed it
	/// (`Desk::game_shown`).
	pub game: Option<GameSummary>,
	/// Each move's record in the engine's fold (the core's `EngineState.record`).
	pub records: HashMap<u16, i64>,
	/// The engine's Book rows (the phones' `HeroBody.ladder`): Hero's ladder lines.
	pub ladders: Vec<BookLadder>,
	/// What of the game's sheet the engine's Book set aside after a clear row or a correction ([`SetAside`]), which
	/// Hero's body map sets aside too.
	pub set_aside: SetAside,
	/// The titles the app's own fold holds earned, by their codes, in the deeds' order.
	pub titles: Vec<u8>,
	/// The XP the app's own sheet counted today (the engine's `todayXp`), the Saga's XP tile's without a game sheet.
	pub today_xp: i64,
	/// Today's embers on the app's own sheet (`Engine::embers`, slice 55: Pump home's embers row where Hero shows no
	/// game sheet) and the week's omen (Hero's vault card's title).
	pub embers: Option<Embers>,
	/// The move each unfinished move rides as where it is another, by its move as owed now (`Engine::rides_as`,
	/// Engine.lua `ns.RidesAs`, 2026-10-29): the waiting card names that move, as the game's HUD does, and its Skip the
	/// owed one.
	rides: HashMap<u16, u16>,
	/// The place in use as the loop last looked (`Desk::places_tick`): a start or an end of a time that changes it
	/// gives the engine its profile and Pump home's place button its glyph.
	pub place: Option<String>,
	/// The day-2 check-in is due.
	checkin: bool,
	/// The joints flagged now (the engine's pain flags).
	pain: Vec<u8>,
	/// An ask is on its way to the engine.
	asking: bool,
	/// The engine's answers are stale: rows came, the game day turned, the profile changed, or a fold ran since.
	dirty: bool,
	/// The game day of the last ask.
	asked_day: Option<i64>,
	/// The asks that failed in a row, and the time (unix seconds) the next may go.
	failed: u32,
	retry_at: i64,
	/// The joints of the last Save the engine did not write (`home.pain`) and that failure's number (`pain_failures`):
	/// none once a Save wrote them or the player put the window away with none.
	pain_failed: Option<(u64, Vec<u8>)>,
	/// The Saves the engine did not write so far: the page opens the window once per failure.
	pain_failures: u64,
	/// What Pro's charts were last built from.
	pub charts_key: Option<ChartsKey>,
	/// The QP 10 focus ask showed this launch (`HomeView.focusAsked`): it comes once.
	pub focus_asked: bool,
	/// The recent games between looks (`desk::recent`).
	pub recent: Recent,
}

impl Home {
	/// Nothing asked yet, the game days in the zone `zone` (`Platform::zone`).
	pub fn new(zone: fn(i64) -> i64) -> Self {
		Self {
			days: Days::new(zone),
			sheets: 0,
			summary: None,
			game: None,
			records: HashMap::new(),
			ladders: Vec::new(),
			set_aside: SetAside::default(),
			titles: Vec::new(),
			today_xp: 0,
			embers: None,
			rides: HashMap::new(),
			place: None,
			checkin: false,
			pain: Vec::new(),
			asking: false,
			dirty: false,
			asked_day: None,
			failed: 0,
			retry_at: 0,
			pain_failed: None,
			pain_failures: 0,
			charts_key: None,
			focus_asked: false,
			recent: Recent::default(),
		}
	}
}

/// The match offer that stands, as the foot offers it.
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
struct Offer {
	id: String,
	/// The game's name (never translated).
	name: String,
	/// The kind its Start opens the pump at Ready for: `match`, every game's offer's (`games::MATCH`).
	kind: &'static str,
}

/// The foot's one gold primary.
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Primary {
	/// macOS: allow screen reading (`asked`: macOS asked before, the button opens System Settings).
	Grant {
		asked: bool,
	},
	/// Windows: the firewall's one-time step that lets the phone in.
	Firewall,
	/// Catch up the moves waiting; none payable while this computer's pain flags rest them all.
	Catchup {
		payable: bool,
	},
	/// The match's offer (`HomeView::offer`): Start a pump with its kind chosen.
	Offer,
	Quick,
}

/// The one card that offers the next part on Pump home, in this order.
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum NextCard {
	/// Pairing a phone: none is paired and Not now has not put the card away (it opens the pairing code and says what
	/// the phone adds, Pro's extras with it).
	Pair,
	/// The addon: the game is on this computer without QuestPump and Not now has not put the card away (Add QuestPump
	/// to the game, its line from This computer's `addon`).
	Addon,
}

/// The moves waiting for a catch-up.
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
struct Waiting {
	moves: Vec<WaitingMove>,
	/// More the game holds back for its next pump.
	held: u32,
	/// This computer's pain flags rest every one of them.
	rested: bool,
	/// Each move's Skip (`home.skip`, since 2026-10-09): none while a client with QuestPump runs, which asks them.
	skip: bool,
}

/// A move on the waiting card: its name and its tile's move (the move it rides as, `Home::rides`), and the owed move
/// and its row's time, which its Skip names back.
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
struct WaitingMove {
	name: String,
	shown: u16,
	#[serde(rename = "move")]
	move_id: u16,
	ts: Option<u32>,
}

/// The check-in's card and its last answer's window: the words in the game's, the joints and those flagged (no safety
/// note: the Menu's Safety note page alone shows it, David, 2026-10-24: only what is legally necessary).
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
struct Checkin {
	words: CheckinWords,
	pain: PainWords,
	joints: Vec<Joint>,
}

/// A Save of the joints the engine did not write (`Home::pain_failed`): the page opens the joints window again with
/// them ticked (the check-in's words, as the card has them) and says they were not saved; `seq` moves on with each
/// failure, so a page opens it once per failure.
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
struct PainFailed {
	seq: u64,
	joints: Vec<u8>,
	checkin: Checkin,
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
struct Joint {
	id: u8,
	name: String,
	flagged: bool,
	/// The game's own rows flag it (the fold's `game_flags`): its row says "(in the game)".
	game: bool,
}

/// Pump home's embers row under the `StreakBoard` (slice 55, Stats.lua `ns.Embers`): today's embers, `have` of `need`
/// (`embers_day`), and the player's cardio step (`ladder`, `step`) whose card a click opens in the Book.
#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
struct EmbersRow {
	have: i64,
	need: i64,
	ladder: u8,
	step: u8,
}

/// The `home` topic.
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
struct HomeView {
	/// A World of Warcraft client with QuestPump runs and asks the pumps (`wow_asks_pumps`): the foot starts none.
	game: bool,
	/// What the status Chip asks the player to fix; none while all is well, when no Chip shows.
	fix: Option<Fix>,
	/// The recent games and the span they count (This week | All time): every one running first, then the most moves
	/// (`Desk::recent_games`); none before the first played.
	recent: RecentGames,
	/// The felt-unwell lock's end (unix ms) while it holds: the rest card, an empty foot.
	locked_until: Option<i64>,
	/// A pump runs on this computer (its cover is up).
	pump: bool,
	/// The engine's state (the `shell` topic's): while it folds every row the pump buttons wait, and the check-in's
	/// answers with them; failed (it did not start or fold, until a reset), no pump starts on this computer, and the
	/// foot says so beside the Menu's report for help.
	engine: EngineState,
	/// The match's offer while it stands and the foot offers pumps: its words above the foot, and its Start the primary
	/// or, under another primary (a step, Catch up), the second quiet button.
	offer: Option<Offer>,
	/// The one card that offers the next part (`Desk::next_card`).
	card: Option<NextCard>,
	streak: Board,
	/// Today's embers under the `StreakBoard` (slice 55); none while pumps rest.
	embers: Option<EmbersRow>,
	/// The streak card's (slice 45): today's game day (`YYYY-MM-DD`, the date it is shared on) and the counts that are
	/// milestones (contract/library.json `streak_milestones`: its Milestone tag, and the streak board's Share from the
	/// least of them).
	day: String,
	milestones: &'static [i64],
	waiting: Option<Waiting>,
	/// The Pump tab's dot: the unfinished moves a catch-up here pays that waited at most `owed_fresh_days` game days.
	dot: usize,
	/// The owed moves the game's next pump takes, under today's chapter.
	held: u32,
	/// Today's chapter, the Saga's (none before its first build).
	today: Option<TodayChapter>,
	/// The day-2 check-in, while it is due.
	checkin: Option<Checkin>,
	/// The joints of a Save the engine did not write, until a Save writes them or puts the window away.
	pain_failed: Option<PainFailed>,
	primary: Option<Primary>,
}

impl<P: Platform> Desk<P> {
	/// The stores took rows (`Desk::apply`): their game days, and the engine asked again once it has them.
	pub fn home_rows(&mut self) {
		let mark = self.store_mark();
		let sheets = self.session.sheet_log().records().len();
		// every frame the strip brings comes through here: only new rows, a reset or a new sheet are news
		if mark == self.home.days.read && sheets == self.home.sheets {
			return;
		}
		if mark != self.home.days.read {
			let (game, own) = (self.session.store().records(), self.session.app_rows().records());
			self.home.days.take(game, own, |from| self.session.far_records(from), mark);
		}
		self.home.sheets = sheets;
		self.home_stale();
	}

	/// What the engine answers Pump home and Hero is stale (rows or a sheet came, the profile changed): asked again now,
	/// the views with it.
	pub fn home_stale(&mut self) {
		self.home.dirty = true;
		self.desk_views();
	}

	/// The loop's look (every 2 s): the views again (the clock, the game, the phone, the offer, the lock), the engine
	/// asked again on a new game day, and the Book once a time it shows has passed.
	pub fn home_look(&mut self) {
		let now = self.p.now_ms().div_euclid(1_000);
		let today = self.home.days.day.index(now);
		if self.home.asked_day.is_some_and(|d| d != today) {
			self.home.dirty = true;
		}
		self.book_look(now);
		self.p.show_found();
		self.desk_views();
	}

	/// The one card that offers the next part, in this order: pairing a phone while none is paired (not while the
	/// addon's feed says the phone apps are not in the stores yet, `phones_offered`), the addon while the game is on
	/// this computer without QuestPump (`found`, as This computer's view holds it); a card its Not now put away waits
	/// `LATER_S`, and the next one shows meanwhile.
	fn next_card(&self, found: FoundGame, now: i64) -> Option<NextCard> {
		let desk = &self.settings.desk;
		let away = |until: Option<i64>| until.is_some_and(|t| now < t);
		if self.settings.pairing.phone.is_none() && self.phones_offered() && !away(desk.pair_card_until) {
			Some(NextCard::Pair)
		} else if found == FoundGame::Installed && !away(desk.addon_card_until) {
			Some(NextCard::Addon)
		} else {
			None
		}
	}

	/// Pump home's and Hero's topics now, the engine asked first when its answers are stale and it can answer.
	fn desk_views(&mut self) {
		self.home_ask();
		self.home_publish();
	}

	/// Pump home's and Hero's topics from the answers as they stand.
	pub fn home_publish(&mut self) {
		let now = self.p.now_ms().div_euclid(1_000);
		let sheet = self.session.sheet(now);
		let board = self.board(&sheet, now);
		self.publish_home(&sheet, &board, now);
		self.publish_hero(&sheet, &board, now);
		self.publish_streak(&sheet, &board, now);
		let game = !board.today && self.game_shown(&sheet).is_some();
		self.p.reminders_board(board, game, sheet.locked_until.is_some());
	}

	/// Asks the engine for its fold's summary, every move's record, the check-in, the pain flags, its Book rows beside
	/// the game's sheet the app shows, the titles earned, today's XP, the moves the unfinished ones ride as and, while
	/// the game's sheet the app shows needs it (`save_sheet_wanted`), the game's save folded as the game folds it, once
	/// the fold is done and while no ask is on its way; an answer that comes back while a fold runs (a merge failed and
	/// every row folds again) leaves the answers stale for the next look, and so does one that failed, asked again
	/// after a wait that doubles with each failure in a row (2 s at first, `RETRY_MAX_S` at most). While a pump runs on
	/// this computer the save is not asked (each row it writes drops the engine's fold of the save, a whole fold of
	/// every row): the sheet shown before it stays, and the pump's end asks again (`pump_publish`). A save that does
	/// not fold, and a Book that does not answer (Hero's ladder lines keep their last), are logged and left out, the
	/// rest of the answer kept, and so are the moves ridden as (the waiting card keeps its last names).
	fn home_ask(&mut self) {
		// stale, or never asked yet (a new player's stores bring no row)
		let stale = self.home.dirty || self.home.asked_day.is_none();
		let at = self.p.now_ms().div_euclid(1_000);
		if !stale || self.home.asking || !self.engine_ready() || at < self.home.retry_at {
			return;
		}
		let ids = facts().move_ids();
		let sheet = self.session.sheet(at);
		let shown = self.game_shown(&sheet);
		let game = shown.and_then(|g| serde_json::from_value::<GameLadders>(g.sheet).ok());
		let wanted = self.save_sheet_wanted(&sheet);
		let save = wanted && !self.desk_pump_running();
		let owed: Vec<String> = self.session.payable_kept().iter().map(ToString::to_string).collect();
		// the game's weeks run from the region's weekly reset the latest sheet it painted names
		let reset = history::reset(self.logged_sheets());
		let game_save = if save { self.game_save_cbor(&sheet) } else { None };
		(self.home.asking, self.home.dirty) = (true, false);
		self.home.asked_day = Some(self.home.days.day.index(at));
		self.engine_call(
			move |e| {
				// the save the game's sheet folds from, before its ask (the same save again keeps its fold)
				if save && let Err(err) = e.game_save(game_save.as_deref()) {
					log::warn!("desk: the game's save for its fold: {err}");
				}
				let summary = Summary::read(&e.summary(at)?);
				let book = e
					.book(game.as_ref(), at)
					.inspect_err(|err| log::warn!("desk: the engine's Book rows for Hero: {err}"))
					.ok();
				Ok(Asked {
					summary,
					records: e.records(&ids, at)?,
					checkin: e.checkin_due(at)?,
					pain: e.pain_flags()?,
					book: book.map(|b| (b.ladders, b.set_aside)),
					titles: e.titles()?,
					today_xp: e.today_xp(at)?,
					embers: e
						.embers(at)
						.inspect_err(|err| log::warn!("desk: the engine's embers for Pump home: {err}"))
						.ok(),
					rides: e
						.rides_as(owed, at)
						.inspect_err(|err| log::warn!("desk: the moves Pump home's unfinished moves ride as: {err}"))
						.ok(),
					game: if save {
						e.game_summary(at, reset).unwrap_or_else(|err| {
							log::warn!("desk: the game's save as the game folds it: {err}");
							None
						})
					} else {
						None
					},
				})
			},
			move |h, answer| {
				h.home.asking = false;
				match answer {
					Ok(asked) => {
						// the Saga reads today's XP, and the app's own summary's level, focus and earned steps, without a
						// game sheet
						if h.home.today_xp != asked.today_xp || h.home.summary != asked.summary {
							h.log_publish();
						}
						h.home.summary = asked.summary;
						h.home.records = asked.records.iter().map(|r| (r.r#move, r.record)).collect();
						(h.home.checkin, h.home.pain) = (asked.checkin, asked.pain);
						if let Some((ladders, set_aside)) = asked.book {
							(h.home.ladders, h.home.set_aside) = (ladders, set_aside);
						}
						(h.home.titles, h.home.today_xp) = (asked.titles, asked.today_xp);
						h.home.embers = asked.embers.or_else(|| h.home.embers.take());
						if let Some(rides) = asked.rides {
							h.home.rides = rides.into_iter().map(|r| (r.r#move, r.stand_in)).collect();
						}
						// the game's sheet the app shows changed: the Book's ladders and the Log's XP read it too; one
						// held back for a pump stays until the pump's end asks again
						if (save || !wanted) && h.home.game != asked.game {
							h.home.game = asked.game;
							h.book_refresh();
							h.log_refresh();
							// and the engine's pumps compose at the focus it names (`sheet_focus`)
							h.engine_rows(true);
						}
						(h.home.failed, h.home.retry_at) = (0, 0);
					}
					Err(e) => {
						log::warn!("desk: Pump home's ask of the engine: {e}");
						h.home.dirty = true;
						h.home.failed = h.home.failed.saturating_add(1);
						h.home.retry_at = at + (1_i64 << h.home.failed.min(9)).min(RETRY_MAX_S);
					}
				}
				if !h.engine_ready() {
					h.home.dirty = true;
				}
				h.desk_views();
			},
		);
	}

	/// The streak now, from the sheet Hero shows (`FreeMode.streak`): the game's with every source's days and the
	/// locked ones, the app's own with its own days alone (it folded every one of them, and never the game's).
	pub fn board(&self, sheet: &Sheet, now: i64) -> Board {
		self.board_from(sheet, now, i64::MAX)
	}

	/// The board now with the days a rest token kept marked back to game day `from` (the Saga's game week).
	pub fn board_from(&self, sheet: &Sheet, now: i64, from: i64) -> Board {
		let days = &self.home.days;
		let today = days.day.index(now);
		let day_of = |t| days.day.index(t);
		let none = BTreeSet::new();
		match self.game_shown(sheet) {
			Some(g) => Board::make(Some(&g.summary), &days.active, &days.locked, today, from, day_of),
			None => Board::make(self.home.summary.as_ref(), &days.own, &none, today, from, day_of),
		}
	}

	/// Today's embers on the game's sheet Hero shows: its summary's `em` while it counted them on `now`'s game day (0 on
	/// another or from an addon before slice 55); Pump home's row and the Result's (2026-10-29).
	pub(super) fn game_embers(&self, game: &GameSheet, now: i64) -> i64 {
		let day = &self.home.days.day;
		game.summary.embers.filter(|&(at, _)| day.index(at) == day.index(now)).map_or(0, |(_, em)| em)
	}

	/// Today's embers as Hero counts them now: the app's own fold's (the engine's last answer) or, where Hero shows the
	/// game's sheet and it counted more, the game's (`game_embers`); a pump's Result counts its own moves' on it
	/// (2026-10-29).
	pub(super) fn embers_today(&self, sheet: &Sheet, now: i64) -> i64 {
		let own = self.home.embers.as_ref().map_or(0, |e| e.today);
		own.max(self.game_shown(sheet).map_or(0, |g| self.game_embers(&g, now)))
	}

	/// Pump home's embers row on the player's cardio step (the Book's rung, its first step before one): today's embers
	/// on the sheet Hero shows, the game's summary's `em` while it counted them on `now`'s game day (0 on another or
	/// from an addon before slice 55) where Hero shows the game's sheet (`game_shown`, the save's fold with the app's
	/// rows once the strip's sheet lacks them), else the app's own fold's (the engine's last answer); none before the
	/// engine answered.
	fn embers_row(&self, sheet: &Sheet, now: i64) -> Option<EmbersRow> {
		let embers = self.home.embers.as_ref()?;
		let have = match self.game_shown(sheet) {
			Some(g) => self.game_embers(&g, now),
			None => embers.today,
		};
		let ladder = facts().library.cardio;
		let rung = self.home.ladders.iter().find(|l| l.ladder == ladder).map_or(1, |l| l.rung.max(1));
		Some(EmbersRow { have, need: embers.most, ladder, step: facts().step(ladder, rung)? })
	}

	fn publish_home(&mut self, sheet: &Sheet, board: &Board, now: i64) {
		let f = facts();
		let paired = self.settings.pairing.phone.is_some();
		let shell = self.p.home_shell(paired);
		let locked_until = sheet.locked_until.map(|t| t * 1_000);
		// the felt-unwell lock's start cancels a day reel being made (slice 45)
		self.p.reel_locked(locked_until.is_some());
		let game = self.wow_asks_pumps();
		// a client whose folder holds QuestPump keeps the moves owed for the game, its squares read or not
		let held_by_game = self.wow_with_addon();
		let pump = self.desk_pump_running();
		let engine = self.engine.state();
		let rested_by = |id: u16| f.move_joints(id).iter().any(|j| self.home.pain.contains(j));
		let payable = sheet.owed.iter().any(|o| !rested_by(o.move_id));
		let waiting = (!sheet.owed.is_empty()).then(|| Waiting {
			moves: sheet
				.owed
				.iter()
				.filter_map(|o| {
					// named as the move it rides as (`rides`), its Skip naming the owed move
					let shown = self.home.rides.get(&o.move_id).copied().unwrap_or(o.move_id);
					let name = f.name(shown).filter(|n| !n.is_empty())?.to_owned();
					Some(WaitingMove { name, shown, move_id: o.move_id, ts: o.ts })
				})
				.collect(),
			held: sheet.held.unwrap_or(0),
			rested: !payable,
			skip: !held_by_game,
		});
		let fresh: Vec<&OwedMove> = self.fresh_owed(sheet, now).collect();
		let dot = if held_by_game { 0 } else { fresh.iter().filter(|o| !rested_by(o.move_id)).count() };
		let game_flags = self.engine_folded().map(|folded| folded.game_flags.as_slice()).unwrap_or_default();
		let checkin_view = || Checkin {
			words: f.sheet.words.checkin.clone(),
			pain: f.sheet.words.pain.clone(),
			joints: f
				.sheet
				.joints
				.iter()
				.map(|j| Joint {
					id: j.id,
					name: j.name.clone(),
					flagged: self.home.pain.contains(&j.id),
					game: game_flags.contains(&j.id),
				})
				.collect(),
		};
		let checkin = self.home.checkin.then(checkin_view);
		let pain_failed = self.home.pain_failed.as_ref().map(|(seq, joints)| PainFailed {
			seq: *seq,
			joints: joints.clone(),
			checkin: checkin_view(),
		});
		// the foot: nothing while the lock holds or a pump runs; a step the player started; then the pumps, which a
		// running game with QuestPump asks itself
		let step = shell.step.map(|step| match step {
			Step::Grant { asked } => Primary::Grant { asked },
			Step::Firewall => Primary::Firewall,
		});
		// the foot offers pumps: no lock, no pump on screen, an engine that started
		let open = locked_until.is_none() && !pump && engine != EngineState::Failed;
		let offer = self.p.offer().filter(|_| open && !game).map(|o| Offer {
			id: o.id.clone(),
			name: o.name.clone(),
			kind: MATCH,
		});
		let primary = if !open {
			None
		} else if let Some(step) = step {
			Some(step)
		} else if game {
			None
		} else if !fresh.is_empty() && !held_by_game {
			Some(Primary::Catchup { payable })
		} else if offer.is_some() {
			Some(Primary::Offer)
		} else {
			Some(Primary::Quick)
		};
		let home = HomeView {
			game,
			fix: shell.fix,
			recent: self.recent_games(now),
			locked_until,
			pump,
			engine,
			offer,
			card: self.next_card(shell.found, now),
			streak: board.clone(),
			embers: self.embers_row(sheet, now).filter(|_| locked_until.is_none()),
			day: history::civil(self.home.days.day.index(now)),
			milestones: &f.constants().streak_milestones,
			held: if waiting.is_some() { 0 } else { sheet.held.unwrap_or(0) },
			dot,
			waiting,
			today: self.book_log.log.home.clone(),
			checkin,
			pain_failed,
			primary,
		};
		self.publisher.publish(HOME, &home);
	}

	/// The waiting moves that make Catch up the primary and put the dot on the Pump tab: those unfinished for at most
	/// `owed_fresh_days` game days (its owed row's game day to today's; one with no time, from an older helper, counts);
	/// an older one stays listed with its Skip and rides the next game pump (the phones' `CatchUp.fresh`).
	fn fresh_owed<'a>(&self, sheet: &'a Sheet, now: i64) -> impl Iterator<Item = &'a OwedMove> {
		let (day, most) = (&self.home.days.day, facts().constants().owed_fresh_days);
		let today = day.index(now);
		sheet.owed.iter().filter(move |o| o.ts.is_none_or(|ts| today - day.index(i64::from(ts)) <= most))
	}

	/// The page's intents on Pump home: the check-in's answer and the joints its last answer names.
	///
	/// # Errors
	/// Why the intent was refused.
	pub fn home_intent(&mut self, intent: &Intent) -> Result<(), String> {
		let at = self.p.now_ms().div_euclid(1_000);
		match intent.kind() {
			"home.checkin" => {
				let answer = intent
					.arg("answer")
					.and_then(Value::as_u64)
					.and_then(|a| u8::try_from(a).ok())
					.filter(|a| (1..=3).contains(a))
					.ok_or("home.checkin takes an answer of 1, 2 or 3")?;
				// asked once: a second answer before the topic hides the card writes nothing
				if !self.home.checkin {
					return Err("home.checkin: no check-in is due".to_owned());
				}
				self.engine_ready_or(intent)?;
				self.home.checkin = false;
				self.engine_call(
					move |e| e.checkin_row(answer, at),
					|h, row| {
						let failed = row.is_err();
						h.keep_own(CHECKIN_ROW, row.map(|r| vec![r]));
						// an answer the engine did not write leaves the check-in due: its card comes back
						if failed {
							h.home.checkin = true;
							h.home_publish();
						}
					},
				);
				// the card goes now; an ask of the engine comes after the row
				self.home_publish();
			}
			"home.pain" => {
				let joints: Vec<u8> = intent
					.arg("joints")
					.and_then(Value::as_array)
					.ok_or("home.pain takes joints")?
					.iter()
					.filter_map(|j| j.as_u64().and_then(|j| u8::try_from(j).ok()))
					.filter(|&j| facts().joint(j).is_some())
					.collect();
				if joints.is_empty() {
					// the window put away with no joint ticked: a Save that failed is answered
					if self.home.pain_failed.take().is_some() {
						self.home_publish();
					}
					return Ok(());
				}
				self.engine_ready_or(intent)?;
				let sent = joints.clone();
				self.engine_call(
					move |e| e.pain_rows(&joints, true, at),
					move |h, answer| match answer {
						Ok(rows) => {
							h.home.pain_failed = None;
							h.keep_own(PAIN_ROW, Ok(rows));
							h.desk_views();
						}
						Err(e) => {
							h.keep_own(PAIN_ROW, Err(e));
							// the page opens the joints window again with them, and says they were not saved
							h.home.pain_failures += 1;
							h.home.pain_failed = Some((h.home.pain_failures, sent));
							h.home_publish();
						}
					},
				);
			}
			"home.later" => {
				let desk = &mut self.settings.desk;
				let until = Some(at + LATER_S);
				match intent.arg("card").and_then(Value::as_str) {
					Some("pair") => desk.pair_card_until = until,
					Some("addon") => desk.addon_card_until = until,
					_ => return Err("home.later takes card: pair or addon".to_owned()),
				}
				self.save_settings();
				self.home_publish();
			}
			"home.skip" => {
				let move_id = intent
					.arg("move")
					.and_then(Value::as_u64)
					.and_then(|m| u16::try_from(m).ok())
					.ok_or("home.skip takes a move")?;
				let ts = intent.arg("ts").and_then(Value::as_u64).and_then(|t| u32::try_from(t).ok());
				self.pump_skip_owed(move_id, ts)?;
			}
			"home.span" => {
				let span: PlayedSpan = intent
					.arg("span")
					.and_then(|v| serde_json::from_value(v.clone()).ok())
					.ok_or("home.span takes span: week or all")?;
				self.settings.desk.played_span = span;
				self.save_settings();
				self.home_publish();
			}
			_ => return Err(intent.unknown()),
		}
		Ok(())
	}

	/// An intent that writes through the engine waits for its fold.
	///
	/// # Errors
	/// The engine folds every row or did not start.
	pub fn engine_ready_or(&self, intent: &Intent) -> Result<(), String> {
		if self.engine_ready() { Ok(()) } else { Err(format!("{}: the engine is folding", intent.kind())) }
	}
}
