//! The game's pump as an iPhone's Live Activity shows it (docs/plan.md slice 76; contract/sync-api.yaml, "The pump
//! push"): a locked iPhone runs no code of the app, so the desktop app tells the account each state of the game's
//! pump, sealed (`questpump_core::sync::LivePump`), and the server pushes it to the activity. [`GamePump`] makes that
//! state from the commands the desktop app queues for the link (`pump_offered`, `countdown`, `keep`, `move_end`,
//! `pump_done`), whichever device holds the link, or none, by the rule the iPhone has for a game pump of its own
//! (`apple/iOS` `PumpRunner.receive`, `LiveActivities.phone`): the move on screen by its kind, the moves done, a
//! timed move's clock from its go to its end, a hold's and a kept move's counting up, a rep move's goal alone. One
//! state stands for a move's countdown and its run together (`started_at` the go, which may lie ahead), where the
//! phone's own rule writes two. A game's pump never ends in a state: its end is the following between pumps.
//!
//! The names are the move book's (`facts`), cut as the contract says (`sync::live_name`); every other word is made on
//! the phone. No clock here: the caller passes the computer's clock in, and the offset to the account's
//! (`GamePump::state`). The desktop app's loop feeds it and paces the posts (`runtime/sync.rs`); the browser has no
//! game and makes none.

use std::collections::BTreeSet;
use std::time::Duration;

use questpump_core::protocol::{Command, OfferedMove};
use questpump_core::sync::{self, LiveMoves, LivePump};

use crate::facts::facts;

/// What a command changed, as the word the account hears of it (contract/sync-api.yaml `ActivityState.event`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Change {
	/// A pump's offer, the pump's first.
	Offer,
	/// A move's Start (a side move's second one too) or its Keep going.
	Start,
	/// A move's end or its way back to its Ready, a pump composed again, a pump's end.
	Rest,
}

/// A word the desktop app posts to the account (`ActivityState.event`): a state of the game's pump, or the word that
/// keeps a following while nothing changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Word {
	/// A pump's offer; `alert` while no device holds the computer's link as the pump is offered, so the push alerts.
	Offer {
		alert: bool,
	},
	Start,
	/// Every other state.
	Rest,
	/// No state and no push.
	Alive,
}

impl Word {
	/// The word of a change; an offer's with whether a device holds the link.
	#[must_use]
	pub fn of(change: Change, held: bool) -> Self {
		match change {
			Change::Offer => Self::Offer { alert: !held },
			Change::Start => Self::Start,
			Change::Rest => Self::Rest,
		}
	}

	/// The contract's word for it, which the log names too.
	#[must_use]
	pub fn event(self) -> &'static str {
		match self {
			Self::Offer { .. } => "offer",
			Self::Start => "start",
			Self::Rest => "rest",
			Self::Alive => "alive",
		}
	}
}

/// What became of a post (`Desk::tell`), for the platform that paces them (`Platform::told`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Told {
	/// The server took it: its clock as it did (unix ms, `ActivityAnswer.t`; none from an answer that does not read).
	Taken(Option<i64>),
	/// The server refused what it said (400, 413): the same state is never sent again.
	Refused,
	/// Not taken (no answer, a 429, a state under another key than the account's, a token gone since): the state as
	/// it then stands goes after the answer's `Retry-After`, or the platform's own wait.
	Later(Option<Duration>),
}

/// How the activity shows a move, by the library's dose.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
	/// Reps: its goal, or a max set; no clock.
	Rep,
	/// A hold: its clock counts up from the go.
	Hold,
	/// Cardio or flow: its clock runs from the go to its end.
	Timed,
}

/// A move's kind; one the library lacks (a newer addon's) shows as a rep move, as on the phone.
fn kind(id: u16) -> Kind {
	let f = facts();
	if f.hold(id) {
		Kind::Hold
	} else if f.rep(id) || !f.knows(id) {
		Kind::Rep
	} else {
		Kind::Timed
	}
}

/// A side move: the game starts it once for each side.
fn sided(id: u16) -> bool {
	facts().text(id).is_some_and(|t| t.side)
}

/// A move's name as a state carries it; empty for a move the book lacks.
fn name(id: u16) -> String {
	sync::live_name(facts().name(id).unwrap_or_default())
}

/// A moment on the computer's clock (ms) as a state carries it: unix seconds on the account's clock, `offset` ms
/// ahead of the computer's, rounded to the nearest second.
fn account_s(ms: i64, offset: i64) -> i64 {
	(ms + offset + 500).div_euclid(1_000)
}

/// The move that counts down or runs: the one on screen (`Pump::current`).
#[derive(Clone, Copy)]
struct Run {
	/// The go of the side that runs, ms on the computer's clock.
	go: i64,
	/// A side move's other side runs.
	other: bool,
	/// The game's Keep going: its clock counts up.
	kept: bool,
}

/// The game's pump from its offer to its end.
struct Pump {
	id: String,
	/// Its moves in the offer's order, a slot its place from 0.
	moves: Vec<OfferedMove>,
	/// The slots of the moves that ran.
	done: BTreeSet<u8>,
	/// The move on screen, its place in `moves`.
	current: usize,
	/// The last move ended: none is on screen until the game ends the pump, composes it again or takes the move back.
	ended: bool,
	run: Option<Run>,
}

impl Pump {
	/// The move on screen; none once the last move ended.
	fn on_screen(&self) -> Option<&OfferedMove> {
		self.moves.get(self.current).filter(|_| !self.ended)
	}

	/// The game's Start for the move at `index`, going at `go`: a side move's Start again while its first side runs is
	/// its other side; any other while a move runs ends that move as far as it ran (its end was lost).
	fn start(&mut self, index: usize, go: i64, now: i64) {
		let other = index == self.current
			&& self.run.is_some_and(|r| !r.other)
			&& self.moves.get(index).is_some_and(|m| sided(m.move_id));
		if !other {
			let ran = self.on_screen().filter(|_| self.run.is_some_and(|r| now >= r.go)).map(|m| m.slot);
			self.done.extend(ran);
			(self.current, self.ended) = (index, false);
		}
		self.run = Some(Run { go, other, kept: false });
	}

	/// The game ended the move that runs: as far as it ran, the next move's Ready on screen (none after the last); one
	/// that never reached its go, a side move's other side too, goes back to its Ready. False while none runs.
	fn end(&mut self, now: i64) -> bool {
		let Some(run) = self.run.take() else { return false };
		if now >= run.go
			&& let Some(slot) = self.on_screen().map(|m| m.slot)
		{
			self.done.insert(slot);
			if self.current + 1 < self.moves.len() {
				self.current += 1;
			} else {
				self.ended = true;
			}
		}
		true
	}

	/// The game took the move of `slot` back to its Ready: the move that counts down or runs, as if its Start never
	/// came (a cancel for a move that ended before it changes nothing), or one that ended while the game still waits
	/// on it (the next move's Ready on screen, or none after the last). Whether anything changed.
	fn back(&mut self, slot: u8) -> bool {
		let index = self.moves.iter().position(|m| m.slot == slot);
		let ended = self.done.contains(&slot);
		if self.run.is_some() {
			if ended && index != Some(self.current) {
				return false;
			}
			self.run = None;
			return true;
		}
		let Some(index) = index.filter(|_| ended) else { return false };
		let waits = if self.ended { index == self.current } else { index + 1 == self.current };
		if waits {
			self.done.remove(&slot);
			(self.current, self.ended) = (index, false);
		}
		waits
	}

	/// The game's Keep going on the move of `slot` while it runs: a timed move's clock counts up from its go.
	fn keep(&mut self, slot: u8, now: i64) -> bool {
		let timed = self.on_screen().is_some_and(|m| m.slot == slot && kind(m.move_id) == Kind::Timed);
		match self.run.as_mut().filter(|r| timed && now >= r.go) {
			Some(run) => {
				run.kept = true;
				true
			}
			None => false,
		}
	}

	/// The pump as a state holds it, its moments on the account's clock (`account_s`).
	fn state(&self, offset: i64) -> LiveMoves {
		let on = self.on_screen();
		let (mut goal, mut max_set, mut started_at, mut ends_at) = (None, false, None, None);
		if let Some(m) = on {
			let kind = kind(m.move_id);
			max_set = m.target == 0 && kind != Kind::Timed;
			goal = (kind == Kind::Rep && m.target > 0).then_some(m.target);
			if let Some(run) = self.run {
				let timed = kind == Kind::Timed && m.target > 0 && !run.kept;
				if timed || kind == Kind::Hold || run.kept {
					started_at = Some(account_s(run.go, offset));
				}
				if timed {
					// its seconds from the go, half of them on each side of a side move
					let ms = i64::from(m.target) * if sided(m.move_id) { 500 } else { 1_000 };
					ends_at = Some(account_s(run.go + ms, offset));
				}
			}
		}
		let count = |n: usize| u16::try_from(n).unwrap_or(u16::MAX);
		LiveMoves {
			pump_id: self.id.clone(),
			moves: self.moves.iter().take(sync::LIVE_NAMES).map(|m| name(m.move_id)).collect(),
			total: count(self.moves.len()),
			done: count(self.moves.iter().filter(|m| self.done.contains(&m.slot)).count()),
			current: on.and_then(|_| i16::try_from(self.current).ok()).unwrap_or(-1),
			name: on.map_or_default(|m| name(m.move_id)),
			goal,
			max_set,
			started_at,
			ends_at,
			next: on.and_then(|_| self.moves.get(self.current + 1)).map(|m| name(m.move_id)),
		}
	}
}

/// The game's pump as the activity shows it, from the commands the desktop app queues for the link.
#[derive(Default)]
pub struct GamePump {
	pump: Option<Pump>,
}

impl GamePump {
	/// A command the desktop app queued at `now` (ms on its clock): what it changed, as the word the account hears of
	/// it; none for a command that changes nothing the activity shows (a `move_counted`, a `verdict`, a command of
	/// another pump than the one on offer, the game's quit, which says no word and leaves no pump).
	pub fn command(&mut self, command: &Command, now: i64) -> Option<Change> {
		match command {
			Command::PumpOffered { pump_id, moves, .. } if !moves.is_empty() => match self.pump.as_mut() {
				// the same pump composed again keeps what ran
				Some(p) if p.id == *pump_id => {
					p.moves.clone_from(moves);
					(p.current, p.ended) = (p.current.min(moves.len().saturating_sub(1)), false);
					Some(Change::Rest)
				}
				// a new one takes the place of the game's last, whose end was lost
				_ => {
					self.pump = Some(Pump {
						id: pump_id.clone(),
						moves: moves.clone(),
						done: BTreeSet::new(),
						current: 0,
						ended: false,
						run: None,
					});
					Some(Change::Offer)
				}
			},
			Command::Countdown { pump_id, slot, go_at, .. } => {
				let p = self.pump.as_mut().filter(|p| p.id == *pump_id)?;
				let index = p.moves.iter().position(|m| m.slot == *slot)?;
				p.start(index, *go_at, now);
				Some(Change::Start)
			}
			Command::Keep { pump_id, slot, .. } => {
				let p = self.pump.as_mut().filter(|p| p.id == *pump_id)?;
				p.keep(*slot, now).then_some(Change::Start)
			}
			Command::MoveEnd { pump_id, slot, cancelled } => {
				let p = self.pump.as_mut().filter(|p| p.id == *pump_id)?;
				let changed = if *cancelled { p.back(*slot) } else { p.end(now) };
				changed.then_some(Change::Rest)
			}
			Command::PumpDone { pump_id, .. } => self.pump.take_if(|p| p.id == *pump_id).map(|_| Change::Rest),
			Command::SessionEnd => {
				self.pump = None;
				None
			}
			_ => None,
		}
	}

	/// The state as it stands: the pump with the move on screen, or between pumps the following itself with the
	/// computer's name (`following`, `link::Helper::name`); a clock's moments on the account's clock, `offset` ms ahead
	/// of the computer's.
	#[must_use]
	pub fn state(&self, following: &str, offset: i64) -> LivePump {
		match &self.pump {
			Some(pump) => LivePump::Pump(pump.state(offset)),
			None => LivePump::Following(sync::live_name(following)),
		}
	}
}
