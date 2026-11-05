//! The pump's clock rules on the computer, ported from the phones' cores (`QuestPumpCore`'s `Countdown`, `MoveRun`,
//! `Marks`, `CountStep` and `CatchUp`; the Android core's twins) and `Speech.lua`'s marks, with the rep count's rule,
//! so a move on the computer counts, switches, marks and asks as it does on a phone and in the game. Pure: times are
//! seconds as `f64` on a side's own clock, milliseconds on it (the rep count's beats) or unix milliseconds, and nothing
//! here reads a clock.

use questpump_core::codes::kind;
use questpump_core::link::OwedMove;
use questpump_engine::Check;
use serde::Serialize;

/// The countdown's digits before the go (`Countdown.digits`: 5, as on every surface).
pub(super) const DIGITS: i64 = 5;
/// The beat before the first digit ("Ready…", `Countdown.lead`), in ms.
const LEAD_MS: i64 = 500;

/// Seconds as `f64` from milliseconds (a pump's spans stay far inside `f64`'s exact integers).
#[expect(clippy::cast_precision_loss, reason = "a span of a pump in ms, far below 2^52")]
pub(super) fn secs(ms: i64) -> f64 {
	ms as f64 / 1000.0
}

/// Milliseconds from seconds, rounded to the nearest.
#[expect(clippy::cast_possible_truncation, reason = "a span of a pump in seconds, far inside i64")]
pub(super) fn millis(s: f64) -> i64 {
	(s * 1000.0).round() as i64
}

/// Whole seconds, down (a hold is never credited a second it did not reach).
#[expect(clippy::cast_possible_truncation, reason = "a span of a pump in seconds, far inside i64")]
pub(super) fn whole(s: f64) -> i64 {
	s.max(0.0).floor() as i64
}

/// A count as `f64`.
#[expect(clippy::cast_precision_loss, reason = "a count of reps or seconds, far below 2^52")]
pub(super) fn real(n: i64) -> f64 {
	n as f64
}

/// The go of a countdown started at `press` (ms): the lead, then the digits.
pub(super) fn go_after(press: i64) -> i64 {
	press + LEAD_MS + DIGITS * 1000
}

/// The digit that has come by `now` before the go `go` (5 to 1, each a second before the go), none in the beat
/// before the first (whose place shows the first digit, as the phones' ring does) and none at or past the go.
pub(super) fn digit(go: i64, now: i64) -> Option<i64> {
	(now >= go - DIGITS * 1000 && now < go).then(|| ((go - now + 999) / 1000).clamp(1, DIGITS))
}

/// When the countdown next changes after `now`: the next digit's moment, or the go.
pub(super) fn next_digit(go: i64, now: i64) -> i64 {
	(1..=DIGITS).map(|n| go - n * 1000).filter(|&at| at > now).min().unwrap_or(go)
}

/// A side of a side move.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Side {
	First,
	Other,
}

/// How a running side ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum End {
	/// The first side ended: the countdown again, then the other side.
	SwitchSides,
	/// The move ended, counting `seconds`; `full`, a timed move's whole seconds when it ended under the ask's share of
	/// them, which asks "Did you do the whole move?" before anything is saved.
	Done { seconds: i64, full: Option<i64> },
}

/// One move from its first go to its end (`MoveRun`): which side runs, and what the move counts when it ends. A side
/// move (a hold, a flow move, a move of reps, one done on the floor or lying down too since 2026-10-13; David:
/// "single leg bridge should have switch side button") runs one side, the countdown again, then the other side on its
/// own clock from zero: a timed move half its seconds a side, counting both; a hold or reps count up on each side and
/// count the weaker.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct MoveRun {
	timed: bool,
	seconds: Option<i64>,
	side: Option<Side>,
	first_side: Option<f64>,
	/// Keep going pressed (slice 52): the seconds the clock now runs to, the full move's (`Slot::keep`).
	kept: Option<i64>,
}

impl MoveRun {
	/// A move of row kind `kind`, on its first side when it is a side move (`side`).
	pub(super) fn new(kind: u8, seconds: Option<i64>, side: bool) -> Self {
		let timed = kind == kind::CARDIO || kind == kind::FLOW;
		Self { timed, seconds, side: side.then_some(Side::First), first_side: None, kept: None }
	}

	pub(super) fn side(&self) -> Option<Side> {
		self.side
	}

	/// What a timed side counts down from: its seconds, half of them on each side of a side move; none for a hold or a
	/// set of reps, whose clock counts up until Done.
	pub(super) fn limit(&self) -> Option<f64> {
		let seconds = real(self.kept.or(self.seconds).filter(|_| self.timed)?);
		Some(if self.side.is_none() { seconds } else { seconds / 2.0 })
	}

	/// Keep going (slice 52, Events.lua `Lc.KeepGoing`): the clock runs on to `to` seconds, the full move's, where it
	/// stops by itself; false where it was kept already or the move is no timed move of one side.
	pub(super) fn keep(&mut self, to: i64) -> bool {
		if !self.timed || self.side.is_some() || self.kept.is_some() {
			return false;
		}
		self.kept = Some(to);
		true
	}

	/// Keep going was pressed on it.
	pub(super) fn kept(&self) -> bool {
		self.kept.is_some()
	}

	/// The running side ended `elapsed` seconds after its go (Done, or exactly its limit): a side move's first side
	/// switches; any other side ends the move, a timed one asking whether it was done whole when the time it ran is
	/// under `ask_share` of its seconds (library.json `timed_ask_share`).
	pub(super) fn end(&mut self, elapsed: f64, ask_share: f64) -> End {
		if self.side == Some(Side::First) {
			self.side = Some(Side::Other);
			self.first_side = Some(elapsed.max(0.0));
			return End::SwitchSides;
		}
		let asks = self.timed && self.seconds.is_some_and(|s| self.ran(elapsed) < ask_share * real(s));
		End::Done { seconds: self.value(elapsed), full: if asks { self.seconds } else { None } }
	}

	/// The whole seconds the move counts when it ends `elapsed` seconds into the running side.
	pub(super) fn value(&self, elapsed: f64) -> i64 {
		whole(self.ran(elapsed))
	}

	/// The time the move counts when it ends `elapsed` seconds into the running side, in ms and not floored to a whole
	/// second (`ran`): a set of reps' counted reps are read at it (`Beats::ended`), which a second rounded down could
	/// cut by a rep.
	pub(super) fn ran_ms(&self, elapsed: f64) -> i64 {
		millis(self.ran(elapsed))
	}

	/// Both sides' time on a timed side move, the weaker side's on any other side move (a hold or a move of reps), the
	/// side's own otherwise.
	fn ran(&self, elapsed: f64) -> f64 {
		let elapsed = elapsed.max(0.0);
		match (self.side, self.first_side) {
			(Some(Side::Other), Some(first)) if self.timed => first + elapsed,
			(Some(Side::Other), Some(first)) => first.min(elapsed),
			_ => elapsed,
		}
	}
}

/// A timer's mark: the number said, or a soft tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Mark {
	Say(i64),
	Tick,
}

/// Every timer's marks (`Speech.lua`'s scheduleMarks, the phones' `Marks`), `step` apart on the running side's own
/// clock: a hold says its elapsed seconds up to `to`; a timed move marks its seconds left down to 10 (the last five tick
/// each second, `last_five`), a flow move saying each, a cardio move ticking while more than `voice` are left and saying
/// them from there, a counted set only ticking; a set of reps has no timer to mark, and its figure's beat counts its
/// reps instead (`Beats`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Marks {
	pub(super) step: f64,
	pub(super) to: f64,
	pub(super) voice: f64,
}

impl Marks {
	/// The next mark after `elapsed` seconds on the side's clock (`limit`: a timed side's) and after the one made last,
	/// due at `last`: a timer that wakes a hair early never marks a second twice. None once the side has no mark left.
	pub(super) fn next(
		&self,
		row_kind: u8,
		limit: Option<f64>,
		elapsed: f64,
		last: Option<f64>,
		counted: bool,
	) -> Option<(f64, Mark)> {
		let elapsed = elapsed.max(0.0);
		match row_kind {
			kind::HOLD => {
				let at = (((elapsed / self.step).floor() + 1.0) * self.step).max(last.unwrap_or(0.0) + self.step);
				(at <= self.to).then(|| (at, Mark::Say(whole(at))))
			}
			kind::CARDIO | kind::FLOW => {
				let limit = limit?;
				let mut left = (((limit - elapsed) / self.step).ceil() - 1.0) * self.step;
				if let Some(last) = last {
					left = left.min(limit - last - self.step);
				}
				if left < 2.0 * self.step {
					return None;
				}
				let quiet = row_kind == kind::CARDIO && (counted || left > self.voice);
				Some((limit - left, if quiet { Mark::Tick } else { Mark::Say(whole(left)) }))
			}
			_ => None,
		}
	}
}

/// The seconds left from which a timed side ticks each second (Panel.lua updateRunning's `lastFive`).
const LAST_FIVE: i64 = 5;

/// A timed side's last ticks (Panel.lua updateRunning's `lastFive`, after the marks, which stop at 10 left): the next
/// after `elapsed` seconds into a side of `limit` seconds and after the one made last (`last`, the whole seconds left
/// it ticked), as the whole seconds left come to 5, 4, 3, 2 and 1, each once, due at `limit` less them; after a late
/// wake the one of the second it woke in, never the ones it slept through. None once the side has none left.
pub(super) fn last_five(limit: f64, elapsed: f64, last: Option<i64>) -> Option<(f64, i64)> {
	#[expect(clippy::cast_possible_truncation, reason = "a span of a pump in seconds, far inside i64")]
	let left = (limit - elapsed.max(0.0)).ceil() as i64;
	let whole = last.map_or(LAST_FIVE, |l| l - 1).min(LAST_FIVE).min(left);
	(whole >= 1).then(|| (limit - real(whole), whole))
}

/// A count as entered: the reps (a hold's seconds, a counted set's clean reps) and, on a loaded move, the weight in the
/// box beside them in tenths of a kg (none: the box empty, no weight); on a counted set the answer to "All out and
/// clean?" it was saved with (`answer`, `ns.Enum.answer`: 0 Yes, to the end, 1 Form slipped, 2 Not all out; slice 42,
/// the third since slice 53).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Typed {
	pub(crate) value: i64,
	pub(crate) load: Option<u32>,
	pub(crate) answer: u8,
}

/// The rep count (2026-10-09; David: "the voice counts for you in the exact speed of the animation so you can follow
/// along"; the rule the game's voice and the phones count by, `Speech.lua`'s repCount and the phones' `RepCount`): a
/// running set of reps says its number as its figure ends a rep, from 1 to `most` (`max_reps`), on past the goal
/// (the least, never the most). The figure's loop at real tempo is `beat` ms, one rep, or two where it shows a dead
/// bug's two sides (`ends`, the ms in the loop each rep ends, the last the loop's end). A side move counts each side's
/// run on its own from 1, past its goal until the player presses Switch sides (David, 2026-10-13: "dont just stop
/// counting and switching at the goal"). It follows the side's clock, not the figure's frames. Where the player's
/// switches let the voice count (2026-10-13; David: "better to show the rep count increasing instead of the time",
/// "and then prefill the rep count after done press"), Big view shows the reps ended so far (`ended`, whose twin on
/// the page reads this rule's fields from the `pump` topic's `rep_count`) and the count step starts from the reps
/// ended by Done (a side move's weaker side's), which the player saves or changes as a typed count.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(super) struct Beats {
	beat: i64,
	ends: Vec<i64>,
	/// How many reps a loop holds (`ends`' length).
	#[serde(skip)]
	per: i64,
	most: i64,
}

/// A count due on a running set of reps: its ms on the side's clock and its number (`n`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Due {
	pub(super) at: i64,
	pub(super) n: i64,
}

impl Beats {
	/// A set's count at `beat` ms a loop, its earlier rep ends `inner` (ms in the loop).
	pub(super) fn new(beat: i64, inner: &[i64], most: i64) -> Self {
		let mut ends: Vec<i64> = inner.iter().copied().filter(|&e| e > 0 && e < beat).collect();
		ends.sort_unstable();
		ends.push(beat);
		let per = i64::try_from(ends.len()).unwrap_or(1);
		Self { beat, ends, per, most }
	}

	/// Count `n`'s ms on the side's clock from the go (Speech.lua countAt): the loop's rep ends, every beat.
	pub(super) fn at(&self, n: i64) -> i64 {
		let end = usize::try_from((n - 1).rem_euclid(self.per)).ok().and_then(|i| self.ends.get(i));
		(n - 1).div_euclid(self.per) * self.beat + end.copied().unwrap_or(self.beat)
	}

	/// The count due next after `elapsed` ms and after the one made last (`last`, none before the first): no number
	/// twice, and after a late wake (a sleep, a stall) the next rep's end past `elapsed`, never a burst of the ones it
	/// missed. None past `most`.
	pub(super) fn next(&self, elapsed: i64, last: Option<Due>) -> Option<Due> {
		let n = (self.ended(elapsed) + 1).max(last.map_or(0, |d| d.n) + 1);
		(n <= self.most).then(|| Due { at: self.at(n), n })
	}

	/// How many reps the loop has ended by `elapsed` ms on the side's clock, at most `most` (Speech.lua countsBy): a rep
	/// ending at `elapsed` itself has ended, as its count is due then (`at`). What Big view shows and the count step
	/// starts from; `helper/dist/views/pump/stages.js` `ended` is its twin on the page.
	pub(super) fn ended(&self, elapsed: i64) -> i64 {
		let elapsed = elapsed.max(0);
		let loops = elapsed / self.beat;
		let into = elapsed - loops * self.beat;
		let n = loops * self.per + i64::try_from(self.ends.iter().filter(|&&e| e <= into).count()).unwrap_or(0);
		n.min(self.most)
	}
}

/// The count step of the move being counted (`CountStep`, the game's result view): the check it shows about the count
/// entered (refused past one set, or asked about once far above or far below the record), the count "Is that right?"
/// asked about, and the count last entered, held until it is recorded; each with its weight, so a weight changed is
/// asked about again. Correct a count's window checks a corrected count by it too (slice 44, `desk/book.rs`).
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct CountStep {
	pub(super) shown: Option<(Check, Typed)>,
	pub(super) really: Option<Typed>,
	pub(super) typed: Option<Typed>,
}

/// What a count entered asks before the engine's check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Entered {
	/// Save does not take it.
	Stays,
	/// A counted set's 0 (Not counted): the set saved with its answer and no count.
	WithoutCount,
	/// The engine checks it against the game's limits (`validate`).
	Check(Typed),
}

/// What a checked count does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Outcome {
	/// Refused, or asked about: nothing recorded.
	Stays,
	/// Recorded as typed, `confirmed` when "Is that right?" asked and its Yes kept it.
	Record { typed: Typed, confirmed: bool },
}

/// What set a count's most, which the line under the dial names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CapBy {
	/// What the app counted (`pump.count.cap`'s {counted}).
	Counted(i64),
	/// The number a set of reps has to beat, past what the app counted (`pump.count.cap.goal`'s {goal}).
	Goal(i64),
}

/// The most a count takes short of one set's most, and what set it: none where the floor of what counts did, which no
/// line names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Cap {
	pub(super) most: i64,
	pub(super) by: Option<CapBy>,
}

impl CountStep {
	/// Save takes `n`: never 0 on a max set, whose 0 is "Could not do one".
	pub(super) fn saves(n: i64, max_set: bool) -> bool {
		n > 0 || !max_set
	}

	/// A count entered on a move that is a max set or a counted set.
	pub(super) fn enter(&mut self, typed: Typed, max_set: bool, counted: bool) -> Entered {
		if !Self::saves(typed.value, max_set) {
			return Entered::Stays;
		}
		if counted && typed.value <= 0 {
			self.clear();
			return Entered::WithoutCount;
		}
		self.typed = Some(typed);
		Entered::Check(typed)
	}

	/// The engine's check of a count as the step asks it: a counted set asks "Is that right?" only about a count far
	/// above its best, never a low one (slice 42's fix round, 2026-10-17: the game's Lc.Counted; "a count under the best
	/// is a normal all-out window").
	pub(crate) fn asks(check: Option<Check>, counted: bool) -> Option<Check> {
		check.filter(|c| !(counted && *c == Check::Low))
	}

	/// The engine's check of a count (Progress.lua `ns.Validate`): a refusal shows; a count or a weight far from the
	/// record or the plan is asked about once (the same count with the same weight again keeps it); the rest is recorded.
	pub(crate) fn checked(&mut self, typed: Typed, check: Option<Check>) -> Outcome {
		match check {
			Some(Check::Max) => {
				self.shown = Some((Check::Max, typed));
				Outcome::Stays
			}
			Some(asks) if self.really != Some(typed) => {
				self.really = Some(typed);
				self.shown = Some((asks, typed));
				Outcome::Stays
			}
			_ => {
				self.clear();
				Outcome::Record { typed, confirmed: check.is_some() }
			}
		}
	}

	/// A count recorded: the step starts again.
	pub(crate) fn clear(&mut self) {
		*self = Self::default();
	}

	/// The check the step shows and the count it is about: a refusal (`max`) or "Is that right?".
	pub(crate) fn asked(&self) -> Option<(Check, Typed)> {
		self.shown
	}

	/// "Change it" (Events.lua `Lc.ClearReally`): the step starts again, but after the weight's own question (`heavy`)
	/// the count stays and the weight box takes the weight planned back (Panel.lua `Panel.ChangeCount`).
	pub(super) fn change(&mut self, plan: Option<u32>) {
		let kept = match self.shown {
			Some((Check::Heavy, typed)) => Some(Typed { load: plan, ..typed }),
			_ => None,
		};
		self.clear();
		self.typed = kept;
	}

	/// The seconds a hold's count starts from: its stopwatch's `held`, or 0 where they fall under the ask's share of
	/// its `aim` (its goal, else its record), since a clock started late or stopped early is no count to offer.
	pub(super) fn held_count(held: i64, aim: Option<i64>, ask_share: f64) -> i64 {
		match aim {
			Some(aim) if real(held) < ask_share * real(aim) => 0,
			_ => held,
		}
	}

	/// The most a count takes where the app counted the move (slice 66, Rapha, 2026-11-04: "dont allow it to do more than
	/// 50% more of what the app counted"; David, 2026-11-04, of a fast honest set the figure counts short: "you fix and
	/// decide on the best"): the higher of `counted` (a set of reps' reps the figure ended by Done, a hold's stopwatch, the
	/// weaker side's on a side move) and `beat` (a set of reps' number to beat, `Run::count_cap`) times 1 + `over`
	/// (`count_over_share`), rounded down, and never under `least` (`min_reps`, a hold's `min_seconds`), so a count the app
	/// caught short of what counts at all can still be saved as one that does, where that stays under one set's `most`;
	/// none where the app counted nothing (a counted set, no rep count, an empty stopwatch) or one set's most binds first,
	/// which keeps today's bounds. The line names what set it only where the share passes that floor. Never a lower bound.
	pub(super) fn cap(counted: Option<i64>, beat: Option<i64>, most: i64, over: f64, least: i64) -> Option<Cap> {
		let counted = counted.filter(|&n| n > 0)?;
		let share = Self::share(beat.map_or(counted, |n| n.max(counted)), over);
		let by = beat.filter(|&n| n > counted).map_or(CapBy::Counted(counted), CapBy::Goal);
		let cap = share.max(least);
		(cap < most).then_some(Cap { most: cap, by: (share > least).then_some(by) })
	}

	/// `from` times 1 + `over`, rounded down: the half again [`Self::cap`] allows before its floor.
	#[expect(clippy::cast_possible_truncation, reason = "a count of one set, far inside i64")]
	fn share(from: i64, over: f64) -> i64 {
		(real(from) * (1.0 + over)).floor() as i64
	}
}

/// An owed move as a catch-up runs it (`CatchUp.Move`), or the one a match pump's rider pays (slice 61).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct CatchMove {
	pub(super) move_id: u16,
	pub(super) kind: u8,
	pub(super) seconds: Option<i64>,
	pub(super) allout: bool,
	pub(super) build: Option<i64>,
	/// Its owed row's time, which its Skip names back.
	pub(super) ts: Option<u32>,
	/// Its place in the owed moves the plan was made from: the owed row the engine doses it from.
	pub(super) row: usize,
}

impl CatchMove {
	/// The owed move `o` at place `row` of the owed moves, run as a catch-up runs it: an all-out window its own move's,
	/// with the build and the all-out end from its go, a plain cardio move no longer than `cardio_max` seconds (a row
	/// owed before cardio went short and hard kept its old length).
	pub(super) fn of(row: usize, o: &OwedMove, build: i64, cardio_max: i64) -> Self {
		let cap = if o.kind == kind::CARDIO && !o.allout { cardio_max } else { i64::MAX };
		Self {
			move_id: o.move_id,
			kind: o.kind,
			seconds: o.seconds.map(|s| i64::from(s).min(cap)),
			allout: o.allout,
			build: o.allout.then_some(build),
			ts: o.ts,
			row,
		}
	}
}

/// A catch-up's plan (`CatchUp.plan`) over the owed moves as `GET /v1/sheet` lists them (`OwedMove::of`): the plain
/// moves in the list's order (`Session::payable_kept`'s: oldest first, spread after the player's latest set), then
/// every all-out window, oldest first, as its last efforts (David, 2026-10-14: "It should just be all
/// coming right after the pump and when im done I can always bank the rest"; the first window alone until then, the
/// others left for the next), each as `CatchMove::of` runs it (an all-out window at most the longest window, which
/// `OwedMove::of` holds it to already).
pub(super) fn catch_up_plan(owed: &[OwedMove], build: i64, cardio_max: i64) -> Vec<CatchMove> {
	let (plain, windows): (Vec<_>, Vec<_>) = owed.iter().enumerate().partition(|(_, o)| !o.allout);
	plain.into_iter().chain(windows).map(|(row, o)| CatchMove::of(row, o, build, cardio_max)).collect()
}
