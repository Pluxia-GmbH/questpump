//! An expedition on the computer (docs/plan.md slice 51, "An expedition is rounds"; the phones' expedition): rounds
//! one after another, each an app pump the engine composes as every app pump (`A.expedition`, its kind key
//! `expedition`), the next one only once the round before ended, from the rows its counts wrote and the regions its
//! record moves took (`Pump:regions`), at the stage the engine decides from the real clock (`A.expeditionNext`: the
//! warm-up, the inner rounds, the last round, the cool-down; none once it is done). Between two rounds the breather
//! (`expedition_rest_s`, the ring): the next round is composed as it begins and waits under it; Start now or the
//! ring's end opens its first move's Ready view (never a countdown), and End the timed pump ends it. Each round's rows
//! are an app pump's rows, its offered row naming its stage and number (`PumpKind::Expedition`, the Saga's title); its
//! Result is held back, and the expedition's Result is every round's together (`view::join`): the Fanfare's
//! "Timed pump complete" on a whole one, a calm "N rounds done" on one ended part way (End the timed pump, End the
//! pump here, the game's own pump), the felt-unwell card alone on a felt-unwell stop. Every start of the Quick pump
//! starts it at the length the Menu's Quick pump length sets (Rapha, 2026-11-03: Pump home's gold Quick pump, the tray,
//! a reminder; the length row of 2026-10-21 and its three lengths gone), and its plan, the engine's dry run of that
//! length (`A.expeditionPreview`), the `ExpeditionBar`'s segments, decided again before each round.
//!
//! **The `expedition` topic**: `{lengths, length}`: the lengths the Menu's Quick pump length offers
//! (`expedition_lengths_min`, minutes) and the one set (kept in settings.json; the first while none is).
//!
//! **Intents**: `expedition.length {minutes}` (the Menu's Quick pump length, one of `lengths`), `expedition.next` (the
//! breather's Start now) and `expedition.stop` (End the expedition, on the breather or a round's More).

use questpump_core::body::Place;
use questpump_core::protocol::ExpeditionStage;
use questpump_engine::{Engine, EngineError, ExpeditionStage as Leg};
use serde::Serialize;
use serde_json::Value;

use super::{Asking, Composed, Cut, Gate, Run, Stage, Started, Stop, What, new_id, rows, unix_s, view};
use crate::facts::facts;
use crate::{Desk, Intent, Platform};

const TOPIC: &str = "expedition";

/// How an expedition ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum End {
	/// Its cool-down, or its last round where no cool-down came.
	Whole,
	/// End the timed pump, End the pump here or the game's own pump, part way.
	Early,
	/// I feel unwell.
	Unwell,
}

/// The expedition the pump on screen is a round of.
pub(super) struct Trip {
	/// The length chosen, minutes.
	minutes: u16,
	/// When it began (ms).
	began: i64,
	/// The place it runs at, as it began.
	place: Place,
	/// The stages planned, the `ExpeditionBar`'s segments: the dry run's at the start, decided again before each round.
	legs: Vec<Leg>,
	/// The stages begun so far.
	at: usize,
	/// The stage that ran last and the regions its rounds' record moves took.
	last: Option<Leg>,
	regions: Vec<String>,
	/// The rounds between the warm-up and the cool-down begun so far.
	round: u8,
	/// Each round's Result as the app's own sheet paid it, held back for the expedition's.
	earned: Vec<view::Earned>,
	/// The moves done in its rounds, and the rounds with a move done.
	done: Vec<u16>,
	rounds: u8,
	/// The breather's end (ms) while it shows.
	rest: Option<i64>,
	/// The next round is on the engine, and once composed it is the pump on screen, waiting under the breather.
	waiting: bool,
	next: bool,
	end: Option<End>,
	/// When it ended (ms).
	ended: Option<i64>,
	/// The highest colour the player had reached as it began (`book_top_colour`; none before the Book's first
	/// answer), against which the share card's headline finds a colour reached for the first time.
	top: Option<u8>,
	/// Once it ended, whether it ran longer than every timed pump before it on this computer (settings'
	/// `expedition.longest`): the share card's minutes headline.
	longest: bool,
	/// The round the bar names (its stage and number): the one in play, under the breather the one that ran.
	label: (&'static str, Option<u8>),
}

impl Trip {
	/// The breather shows.
	pub(super) fn resting(&self) -> bool {
		self.rest.is_some()
	}

	/// When the breather ends, once the next round waits under it.
	pub(super) fn rest_until(&self) -> Option<i64> {
		self.rest.filter(|_| self.next)
	}

	/// The stage `stage` begins: the plan keeps it where the engine decided otherwise (a warm-up skipped, a round more
	/// or fewer than the dry run's, a cool-down that came early).
	fn begin(&mut self, stage: Leg) {
		let i = self.at;
		while stage != Leg::Warmup && self.legs.get(i) == Some(&Leg::Warmup) {
			self.legs.remove(i);
		}
		if self.legs.get(i) != Some(&stage) {
			match stage {
				Leg::Warmup | Leg::Inner => self.legs.insert(i, stage),
				Leg::Last | Leg::Cooldown => {
					self.legs.truncate(i);
					self.legs.push(stage);
					if stage == Leg::Last {
						self.legs.push(Leg::Cooldown);
					}
				}
			}
		}
		self.at += 1;
		self.last = Some(stage);
		if matches!(stage, Leg::Inner | Leg::Last) {
			self.round += 1;
		}
	}

	/// The minutes its Result and card name, once it ended at `ended`: a whole expedition's chosen length (what was set
	/// out, as `docs/ui.md` shows it), one ended part way the whole minutes it ran, at least one.
	fn minutes_shown(&self, ended: i64) -> i64 {
		if self.end == Some(End::Whole) {
			i64::from(self.minutes)
		} else {
			(ended - self.began).div_euclid(60_000).max(1)
		}
	}

	/// The round's stage and number as its offered row and its title name them.
	fn what(&self, stage: Leg) -> What {
		what_of(stage, self.round)
	}

	/// The round shown is the one `what` is: the bar names it.
	fn show(&mut self, what: &What) {
		if let What::Expedition { stage, round } = *what {
			self.label = match stage {
				ExpeditionStage::Warmup => ("warmup", None),
				ExpeditionStage::Round => ("round", round),
				ExpeditionStage::Cooldown => ("cooldown", None),
			};
		}
	}

	/// The rounds planned between the warm-up and the cool-down.
	fn count(&self) -> usize {
		self.legs.iter().filter(|l| matches!(l, Leg::Inner | Leg::Last)).count()
	}
}

/// What the engine answered for an expedition's next round: the round composed (its stage, the pump, its start's
/// facts), or none more (the rows of a round that composed no move, the start rungs').
enum Next {
	Round(Leg, Box<(Composed, Started)>),
	Done(Leg, Composed),
	Over,
}

/// A round's stage and number as its offered row and its title name them, `round` the rounds begun with it.
fn what_of(stage: Leg, round: u8) -> What {
	match stage {
		Leg::Warmup => What::Expedition { stage: ExpeditionStage::Warmup, round: None },
		Leg::Inner | Leg::Last => What::Expedition { stage: ExpeditionStage::Round, round: Some(round) },
		Leg::Cooldown => What::Expedition { stage: ExpeditionStage::Cooldown, round: None },
	}
}

/// The same for a stage composed after `begun` rounds that never began (`Trip::begin`): its rows are kept under it
/// when the expedition left or ended while it was composed.
fn unbegun(stage: Leg, begun: u8) -> What {
	what_of(stage, begun + u8::from(matches!(stage, Leg::Inner | Leg::Last)))
}

/// The next round from the clock: none once the expedition is done, and none more when a round composes no move (the
/// place shuts every move it could ask: no round is forced).
fn next_round(
	e: &Engine,
	id: &str,
	budget: i64,
	elapsed: i64,
	last: Option<Leg>,
	regions: &[String],
	at: i64,
) -> Result<Next, EngineError> {
	let Some(stage) = e.expedition_next(budget, elapsed, last, regions, at)? else { return Ok(Next::Over) };
	let composed = e.expedition(id, stage, regions, at)?;
	if composed.pump.slots.is_empty() {
		return Ok(Next::Done(stage, composed));
	}
	Ok(Next::Round(stage, Box::new((composed, Started::read(e, at)?))))
}

#[derive(Serialize)]
struct Topic<'a> {
	lengths: &'a [u16],
	length: Option<u16>,
}

/// The `ExpeditionBar` and the breather and the Result's facts, for the pump on screen.
#[derive(Serialize, PartialEq)]
pub(super) struct TripView {
	/// When the length chosen runs out (ms): the bar's minutes left.
	ends: i64,
	legs: Vec<LegView>,
	/// The stage in play, or the last that ran (`warmup`, `round`, `cooldown`), its round's number and the rounds
	/// planned.
	stage: &'static str,
	round: Option<u8>,
	count: usize,
	rest: Option<RestView>,
	done: Option<DoneView>,
}

/// A segment of the bar: short for the warm-up and the cool-down; `done`, `now` or `next`.
#[derive(Serialize, PartialEq)]
struct LegView {
	short: bool,
	state: &'static str,
}

/// The breather: its end (ms), its length (s), and once the next round waits under it, its stage and number.
#[derive(Serialize, PartialEq)]
struct RestView {
	until: i64,
	seconds: i64,
	next: Option<&'static str>,
	round: Option<u8>,
}

/// The expedition's Result: the minutes it took, the rounds with a move done, whether it ended part way, its place,
/// the muscles its moves worked, the moves done, the records and the XP of every round; and the share card's headline
/// facts (docs/plan.md slice 51, "The share cards tell one true thing"): a colour reached for the first time, the
/// record with the biggest step, and whether it was the longest timed pump yet.
#[derive(Serialize, PartialEq)]
struct DoneView {
	minutes: i64,
	rounds: u8,
	early: bool,
	place: Place,
	most: Vec<u8>,
	some: Vec<u8>,
	moves: usize,
	records: usize,
	xp: i64,
	#[serde(flatten)]
	brag: view::Brag,
	longest: bool,
}

impl<P: Platform> Desk<P> {
	/// The `expedition` area's intents (the module's doc lists them).
	///
	/// # Errors
	/// Why the intent was refused.
	pub fn expedition_intent(&mut self, intent: &Intent) -> Result<(), String> {
		match intent.kind() {
			"expedition.length" => {
				let minutes = intent
					.arg("minutes")
					.and_then(Value::as_u64)
					.and_then(|m| u16::try_from(m).ok())
					.ok_or("expedition.length takes minutes")?;
				if !facts().constants().expedition_lengths_min.contains(&minutes) {
					return Err(format!("expedition.length: {minutes} minutes is no length"));
				}
				self.settings.expedition.quick = Some(minutes);
				self.save_settings();
				self.publish_expedition();
				Ok(())
			}
			"expedition.next" => self.expedition_rest_end(),
			"expedition.stop" => self.expedition_stop(),
			_ => Err(intent.unknown()),
		}
	}

	/// The Quick pump's length (minutes): the one the Menu set while `expedition_lengths_min` lists it, else that list's
	/// first; none while the list does not read.
	pub(super) fn quick_length(&self) -> Option<u16> {
		let lengths = &facts().constants().expedition_lengths_min;
		self.settings.expedition.quick.filter(|m| lengths.contains(m)).or_else(|| lengths.first().copied())
	}

	/// `expedition` as it stands.
	pub fn publish_expedition(&self) {
		let topic = Topic { lengths: &facts().constants().expedition_lengths_min, length: self.quick_length() };
		self.publisher.publish(TOPIC, &topic);
	}

	/// An expedition of `minutes` begins (`pump_begin`, past the gates and the place): the dry run's plan for its bar,
	/// then its first round.
	pub(super) fn expedition_begin(&mut self, minutes: u16, outside: bool) {
		let (now, id) = (self.p.now_ms(), new_id::<P>());
		let at = unix_s(now);
		self.pumps.starting = true;
		self.pumps.choose = None;
		self.pumps.trip = Some(Trip {
			minutes,
			began: now,
			place: self.place_shown(),
			legs: Vec::new(),
			at: 0,
			last: None,
			regions: Vec::new(),
			round: 0,
			earned: Vec::new(),
			done: Vec::new(),
			rounds: 0,
			rest: None,
			waiting: true,
			next: false,
			end: None,
			ended: None,
			top: self.book_top_colour(),
			longest: false,
			label: ("warmup", None),
		});
		let budget = i64::from(minutes) * 60;
		let pump_id = id.clone();
		self.engine_call(
			move |e| {
				let plan = e.expedition_preview(minutes, at)?;
				Ok((
					plan.rounds.into_iter().map(|r| r.stage).collect::<Vec<_>>(),
					next_round(e, &pump_id, budget, 0, None, &[], at)?,
				))
			},
			move |h, answer| {
				h.pumps.starting = false;
				match answer {
					Ok((legs, Next::Round(stage, made))) => {
						let Some(trip) = h.pumps.trip.as_mut() else {
							// left meanwhile: the rows the engine wrote are kept all the same
							h.pump_keep(&id, unbegun(stage, 0).kind(), &rows(&made.0.rows));
							return;
						};
						trip.legs = legs;
						trip.waiting = false;
						trip.begin(stage);
						let what = trip.what(stage);
						trip.show(&what);
						let (composed, started) = *made;
						log::info!("expedition {id}: {minutes} min, {} rounds planned", trip.count());
						h.pump_composed(
							composed,
							Some(started),
							Asking { what, id, offer: None, outside, arms: None, riding: Vec::new() },
						);
					}
					Ok((_, next)) => {
						if let Next::Done(stage, composed) = next {
							h.pump_keep(&id, unbegun(stage, 0).kind(), &rows(&composed.rows));
						}
						log::info!("expedition {id}: the engine composed no round");
						h.pumps.trip = None;
						h.pump_nowhere();
					}
					Err(e) => {
						log::warn!("expedition: the engine composed no round: {e}");
						h.pumps.trip = None;
						h.pump_nowhere();
					}
				}
			},
		);
	}

	/// The breather shows over an expedition's round (`pump_answered`'s check that keeps a late answer off another
	/// pump).
	pub(super) fn expedition_resting(&self) -> bool {
		self.pumps.trip.as_ref().is_some_and(Trip::resting)
	}

	/// A round's moves are over (`pump_finish`): the moves it did join the expedition's, and how it ended decides what
	/// comes: felt unwell ends the expedition on its card; an early end, part way; its cool-down, whole; any other the
	/// breather, under which the next round is composed once the round's Result is read (`expedition_round_earned`).
	pub(super) fn expedition_round_ends(&mut self, stopped: bool) {
		let now = self.p.now_ms();
		let rest = facts().constants().expedition_rest_s * 1000;
		let (Some(run), Some(trip)) = (self.pumps.run.as_ref(), self.pumps.trip.as_mut()) else { return };
		if !matches!(run.what, What::Expedition { .. }) {
			return;
		}
		let done: Vec<u16> = run
			.moves
			.iter()
			.filter(|m| run.ran.contains_key(&m.slot) || run.recorded.contains_key(&m.slot))
			.map(|m| m.r#move)
			.collect();
		if !done.is_empty() {
			trip.rounds += 1;
			trip.done.extend(done);
		}
		if run.unwell {
			trip.end = Some(End::Unwell);
			trip.ended = Some(now);
		} else if stopped || run.cut.is_some() {
			trip.end = Some(End::Early);
		} else if trip.last != Some(Leg::Cooldown) {
			trip.rest = Some(now + rest);
			trip.waiting = true;
			trip.next = false;
		}
	}

	/// The rounds done before the one on screen ended part way are kept on the expedition's Result (`pump_stop`).
	pub(super) fn expedition_kept(&self) -> bool {
		self.pumps.trip.as_ref().is_some_and(|t| t.rounds > 0)
	}

	/// A round's Result as the app's own sheet paid it (`pump_result`): held for the expedition's, then the next round
	/// (the regions its record moves took and the clock deciding its stage) or the expedition's end.
	pub(super) fn expedition_round_earned(&mut self, earned: view::Earned) {
		let Some(trip) = self.pumps.trip.as_mut() else { return };
		trip.earned.push(earned);
		if trip.end.is_some() || trip.rest.is_none() {
			if trip.end.is_none() {
				trip.end = Some(End::Whole);
			}
			self.expedition_finish();
			return;
		}
		let (id, now) = (new_id::<P>(), self.p.now_ms());
		let at = unix_s(now);
		let (budget, elapsed) = (i64::from(trip.minutes) * 60, (now - trip.began).div_euclid(1000));
		let (last, regions, begun) = (trip.last, trip.regions.clone(), trip.round);
		let pump_id = id.clone();
		self.engine_call(
			move |e| {
				let regions = e.pump_regions(&regions)?;
				let next = next_round(e, &pump_id, budget, elapsed, last, &regions, at)?;
				Ok((regions, next))
			},
			move |h, answer| {
				let Some(trip) = h.pumps.trip.as_mut() else {
					// left meanwhile: the rows the engine wrote are kept all the same
					match answer {
						Ok((_, Next::Round(stage, made))) => {
							h.pump_keep(&id, unbegun(stage, begun).kind(), &rows(&made.0.rows));
						}
						Ok((_, Next::Done(stage, composed))) => {
							h.pump_keep(&id, unbegun(stage, begun).kind(), &rows(&composed.rows));
						}
						Ok((_, Next::Over)) | Err(_) => {}
					}
					return;
				};
				trip.waiting = false;
				match answer {
					Ok((regions, Next::Round(stage, made))) if trip.end.is_none() => {
						trip.regions = regions;
						trip.begin(stage);
						let what = trip.what(stage);
						let (composed, started) = *made;
						trip.next = true;
						h.pump_composed(
							composed,
							Some(started),
							Asking { what, id, offer: None, outside: false, arms: None, riding: Vec::new() },
						);
						// a Start now pressed before it came, or a ring that ran out meanwhile
						if h.pumps.trip.as_ref().and_then(Trip::rest_until).is_some_and(|until| until <= h.p.now_ms()) {
							let _ = h.expedition_rest_end();
						}
					}
					Ok((_, Next::Round(stage, made))) => {
						// ended on the breather while it was composed: its rows kept, the pump never shown
						let what = unbegun(stage, trip.round);
						h.pump_keep(&id, what.kind(), &rows(&made.0.rows));
						h.expedition_finish();
					}
					Ok((_, next)) => {
						trip.end.get_or_insert(End::Whole);
						if let Next::Done(stage, composed) = next {
							h.pump_keep(&id, unbegun(stage, begun).kind(), &rows(&composed.rows));
						}
						h.expedition_finish();
					}
					Err(e) => {
						log::warn!("expedition: the next round: {e}");
						trip.end.get_or_insert(End::Early);
						h.expedition_finish();
					}
				}
			},
		);
	}

	/// The breather ends (Start now, or the ring at its end): the next round's first move on its Ready view, read aloud;
	/// pressed before the round came, it opens as soon as it comes.
	fn expedition_rest_end(&mut self) -> Result<(), String> {
		// felt unwell or the game asking pumps since the breather began: no round opens
		if self.expedition_resting() && matches!(self.pump_gate(), Some(Gate::Lock | Gate::Game)) {
			return self.expedition_stop();
		}
		let trip = self.pumps.trip.as_mut().filter(|t| t.resting()).ok_or("no breather")?;
		if !trip.next {
			trip.rest = Some(self.p.now_ms());
			return Ok(());
		}
		trip.rest = None;
		trip.next = false;
		if let Some(run) = self.pumps.run.as_ref() {
			trip.show(&run.what);
		}
		self.pump_ready();
		self.pump_publish();
		Ok(())
	}

	/// The loop woke for the breather's end (`pump_due`): whether it ended now.
	pub(super) fn expedition_rest_due(&mut self, now: i64) -> bool {
		let due = self.pumps.trip.as_ref().and_then(Trip::rest_until).is_some_and(|until| now >= until);
		due && self.expedition_rest_end().is_ok()
	}

	/// Felt unwell or the game asking pumps closed the gate (`pump_refresh`, `pump_game_offered`): an expedition on its
	/// breather ends there part way, as End the timed pump ends it, its next round never shown.
	pub(super) fn expedition_gate_closed(&mut self) {
		if self.expedition_resting() {
			log::info!("expedition: a gate closed on the breather; the expedition ends");
			let _ = self.expedition_stop();
		}
	}

	/// End the timed pump: on the breather it ends part way with the rounds done (the next round, composed or on its
	/// way, never shown); in a round it ends as End the pump here does.
	fn expedition_stop(&mut self) -> Result<(), String> {
		let trip = self.pumps.trip.as_mut().ok_or("no expedition")?;
		if !trip.resting() {
			return self.pump_stop(Stop::Left);
		}
		trip.rest = None;
		trip.end = Some(End::Early);
		if !trip.waiting {
			self.expedition_finish();
		}
		self.pump_publish();
		Ok(())
	}

	/// The expedition is over: every round's Result together on the pump on screen (a round composed for the breather
	/// that never showed is put down as a pump left on its Ready view), with the reveal on a whole one and still on one
	/// ended part way.
	fn expedition_finish(&mut self) {
		let now = self.p.now_ms();
		let before = self.settings.expedition.longest;
		let Some(trip) = self.pumps.trip.as_mut() else { return };
		trip.rest = None;
		trip.next = false;
		trip.ended = Some(now);
		let whole = trip.end == Some(End::Whole);
		let minutes = trip.minutes_shown(now);
		// the longest timed pump yet: longer than one before it (the first sets the mark and claims nothing)
		let ran = u16::try_from(minutes).unwrap_or(u16::MAX);
		trip.longest = trip.end != Some(End::Unwell) && before.is_some_and(|b| ran > b);
		if trip.end != Some(End::Unwell) && before.is_none_or(|b| ran > b) {
			self.settings.expedition.longest = Some(ran);
			self.save_settings();
		}
		let Some(trip) = self.pumps.trip.as_mut() else { return };
		let joined = view::join(&trip.earned, whole.then_some(minutes));
		let Some(run) = self.pumps.run.as_mut() else { return };
		if run.running() {
			run.stage = Stage::Done;
			run.done_at = Some(now);
			run.swaps = None;
		}
		if whole {
			run.reveal = Some(view::Reveal::plan(now, &joined, true));
		} else {
			run.cut = Some(Cut::Left);
			run.reveal = None;
		}
		run.result = Some(joined);
		log::info!("expedition: over ({:?}) after {minutes} min", trip.end);
		self.pump_publish();
	}

	/// The `ExpeditionBar`, the breather and the Result's facts for the pump on screen `run`.
	pub(super) fn trip_view(&self, run: &Run) -> Option<TripView> {
		let trip = self.pumps.trip.as_ref()?;
		let playing = trip.rest.is_none() && trip.end.is_none() && run.running();
		// the stages shown so far: under the breather, the round waiting under it is begun and not shown
		let shown = trip.at - usize::from(trip.next);
		let legs = trip
			.legs
			.iter()
			.enumerate()
			.map(|(i, &leg)| {
				let state = if i + 1 == shown && playing {
					"now"
				} else if i < shown {
					"done"
				} else {
					"next"
				};
				LegView { short: matches!(leg, Leg::Warmup | Leg::Cooldown), state }
			})
			.collect();
		let (stage, round) = trip.label;
		let rest = trip.rest.map(|until| {
			let next = match run.what {
				What::Expedition { stage: ExpeditionStage::Cooldown, .. } if trip.next => Some("cooldown"),
				What::Expedition { stage: ExpeditionStage::Round, .. } if trip.next => Some("round"),
				_ => None,
			};
			let round = match run.what {
				What::Expedition { round, .. } if trip.next => round,
				_ => None,
			};
			RestView { until, seconds: facts().constants().expedition_rest_s, next, round }
		});
		let done = trip.ended.filter(|_| trip.end != Some(End::Unwell)).map(|ended| {
			let (most, some) = facts().muscles(trip.done.iter().copied());
			let result = run.result.as_ref();
			DoneView {
				minutes: trip.minutes_shown(ended),
				rounds: trip.rounds,
				early: trip.end != Some(End::Whole),
				place: trip.place.clone(),
				most,
				some,
				moves: trip.done.len(),
				records: result.map_or(0, view::Earned::records),
				xp: result.and_then(view::Earned::xp).unwrap_or(0),
				brag: result.map(|r| r.brag(trip.top)).unwrap_or_default(),
				longest: trip.longest,
			}
		});
		Some(TripView {
			ends: trip.began + i64::from(trip.minutes) * 60_000,
			legs,
			stage,
			round,
			count: trip.count(),
			rest,
			done,
		})
	}
}
