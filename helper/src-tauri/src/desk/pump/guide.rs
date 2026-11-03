//! A move's How and Tips read aloud on the computer (docs/ui.md, "The pump on the computer": Ready's read-aloud button;
//! the game's guide, Speech.lua `GuideTakes`, `ToggleGuide` and `Say`): the takes DESK-MEDIA finds in the install the
//! app follows (`Helper::guide_takes`), read one after another, How's then Tips' with `callout_gap_s` before Tips, each
//! as the take before ends, on the loop's wake (`Helper::pump_wake`), never a page timer. The button (`pump.guide`)
//! begins it on the move's Ready view, its countdown or its run, pauses it while it reads (held too) and goes on from
//! the take it paused in, once a line said meanwhile and the gap after it have passed. A line the pump says over it
//! holds it (`Helper::guide_hold`): its take stops, and it goes on from that take `callout_gap_s` after the line ends;
//! a count plays over it. It reads for one move's run, the pump, the move's index and the move: another stage, another
//! move or the pump's end stops it (`Helper::pump_keep_guide`, at each publish), and Back keeps it.

use super::{Run, Stage, run};
use crate::desk::facts::facts;
use crate::desk::seams::GuideTakes;
use crate::lan;
use crate::runtime::Helper;

/// A move's run in a pump, the one a guide reads for: the pump, the move's index and the move (Speech.lua `runId`).
type Read = (String, usize, u16);

/// Where a guide stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
	/// A take plays, or the gap after it: the next one is due.
	Reading,
	/// A line said over it plays: it goes on from its take once that line and the gap after it have passed.
	Held,
	/// The player paused it: it goes on from its take at the button.
	Paused,
}

/// A move's guide read aloud (Speech.lua's `guide`).
pub(super) struct Guide {
	run: Read,
	takes: GuideTakes,
	/// The take on: playing (or just played) while it reads, the one it goes on from while it stands.
	at: usize,
	/// When the take on ends (ms).
	ends: i64,
	/// When it goes on (ms): to the next take while it reads, to its take again once held; none while paused.
	due: Option<i64>,
	state: State,
	/// When the last line said over it ends (ms): a resume waits for it and the gap after it.
	quiet: i64,
}

/// The run the move on's guide reads for: the move on while the pump shows its Ready view, its countdown or its run
/// (Speech.lua `guideRun`); else none.
fn read_of(run: &Run) -> Option<Read> {
	let m = run.shown()?;
	matches!(run.stage, Stage::Ready | Stage::Countdown { .. } | Stage::Running { .. })
		.then(|| (run.id.clone(), run.current, m.r#move))
}

/// The gap before Tips and after a line said over the guide (ms).
fn gap() -> i64 {
	run::millis(facts().constants().callout_gap_s)
}

/// The guide stands where it reads: on its take, or on the next where its take had ended (it stood in the gap after
/// it); a guide held already stays on its take (Speech.lua `haltGuide`).
fn halt(guide: &mut Guide, now: i64) {
	if guide.state == State::Reading && now >= guide.ends {
		guide.at += 1;
	}
	guide.due = None;
}

impl Helper {
	/// The read-aloud button's word on the `pump` topic: `reading` (held too), `paused`, or `idle` while none reads;
	/// none where no guide can read now (`Pumps::offered`).
	pub(super) fn pump_guide_state(&self) -> Option<&'static str> {
		self.pumps.offered.then(|| match self.pumps.guide.as_ref().map(|g| g.state) {
			None => "idle",
			Some(State::Paused) => "paused",
			Some(State::Reading | State::Held) => "reading",
		})
	}

	/// Before each publish: a guide whose run is over stops (another stage, another move, the pump's end), and whether
	/// the button shows is read again (the move on's takes in the install the app follows).
	pub(super) fn pump_keep_guide(&mut self) {
		let read = self.pumps.run.as_ref().and_then(read_of);
		if self.pumps.guide.as_ref().is_some_and(|g| Some(&g.run) != read.as_ref()) {
			self.pumps.guide = None;
			self.guide_stop();
		}
		self.pumps.offered = read.is_some() && (self.pumps.guide.is_some() || !self.shown_guide().takes.is_empty());
	}

	/// The move on's guide's takes (`Helper::guide_takes`), its implement's own where the voice has them.
	fn shown_guide(&mut self) -> GuideTakes {
		let Some(m) = self.pumps.run.as_ref().and_then(Run::shown) else { return GuideTakes::default() };
		let gear = m.implement.clone();
		let Some(text) = facts().text(m.r#move) else { return GuideTakes::default() };
		self.guide_takes(&text.key, gear.as_deref())
	}

	/// The read-aloud button (`pump.guide`; Speech.lua `ToggleGuide`, the player asked, so the voice's switches decide
	/// nothing): begins the move on's guide, pauses it while it reads (held too), and goes on from the take it paused
	/// in, once a line said meanwhile and the gap after it have passed.
	pub(super) fn pump_guide(&mut self) -> Result<(), String> {
		let now = lan::now_ms();
		match self.pumps.guide.as_mut() {
			Some(guide) if guide.state == State::Paused => {
				if guide.quiet > now {
					guide.state = State::Held;
					guide.due = Some(guide.quiet + gap());
				} else {
					let at = guide.at;
					self.guide_read_take(at, now);
				}
			}
			Some(guide) => {
				let playing = guide.state == State::Reading;
				halt(guide, now);
				guide.state = State::Paused;
				if playing {
					self.guide_stop();
				}
			}
			None => self.guide_begin(now)?,
		}
		self.pump_publish();
		Ok(())
	}

	/// Begins the move on's guide at its first take (Speech.lua `beginGuide`): the line speaking stops, and a callout
	/// still to come on its Ready view stays unsaid.
	fn guide_begin(&mut self, now: i64) -> Result<(), String> {
		let read = self.pumps.run.as_ref().and_then(read_of).ok_or("no guide reads now")?;
		let takes = self.shown_guide();
		if takes.takes.is_empty() {
			return Err("the move has no guide in the install".to_owned());
		}
		if let Some(run) = self.pumps.run.as_mut() {
			run.callout = None;
			run.callout_ends = None;
		}
		self.hush();
		self.pumps.guide =
			Some(Guide { run: read, takes, at: 0, ends: now, due: None, state: State::Reading, quiet: now });
		self.guide_read_take(0, now);
		Ok(())
	}

	/// Reads take `at` of the guide, the next one due as it ends (`callout_gap_s` after it where Tips begin); past the
	/// last the guide ends (Speech.lua `readTake`).
	fn guide_read_take(&mut self, at: usize, now: i64) {
		let Some(guide) = self.pumps.guide.as_mut() else { return };
		let Some(take) = guide.takes.takes.get(at).cloned() else {
			self.pumps.guide = None;
			return;
		};
		guide.at = at;
		guide.ends = now + run::millis(take.seconds);
		guide.due = Some(guide.ends + if guide.takes.tips == Some(at + 1) { gap() } else { 0 });
		guide.state = State::Reading;
		self.guide_play(&take);
	}

	/// A voice line `seconds` long plays now (`id`; Speech.lua `Say`): a guide reading, or held already, stops at its
	/// take and goes on from it `callout_gap_s` after this line ends (a line stops the one before, so its end is the
	/// voice's); a paused one keeps when the line ends, which its resume waits for; a count plays over the guide.
	pub(crate) fn pump_guide_hold(&mut self, id: &str, seconds: f64) {
		if id.starts_with("count.") {
			return;
		}
		let now = lan::now_ms();
		let Some(guide) = self.pumps.guide.as_mut() else { return };
		guide.quiet = now + run::millis(seconds);
		if guide.state == State::Paused {
			return;
		}
		let playing = guide.state == State::Reading;
		halt(guide, now);
		guide.state = State::Held;
		guide.due = Some(guide.quiet + gap());
		if playing {
			self.guide_stop();
		}
	}

	/// When the guide goes on next (ms, `Helper::pump_wake`): its next take, or its take again once held; none while
	/// paused or with none reading.
	pub(super) fn guide_wake(&self) -> Option<i64> {
		self.pumps.guide.as_ref()?.due
	}

	/// The guide goes on where it is due: to its next take, or once held to the take it stood on; one past its last
	/// ends, and the button says so.
	pub(super) fn guide_due(&mut self, now: i64) {
		let Some(guide) = self.pumps.guide.as_ref().filter(|g| g.due.is_some_and(|due| now >= due)) else { return };
		let at = if guide.state == State::Reading { guide.at + 1 } else { guide.at };
		self.guide_read_take(at, now);
		if self.pumps.guide.is_none() {
			self.pump_publish();
		}
	}
}
