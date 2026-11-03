//! The heart-rate proof of each pump (docs/plan.md slice 8): the heart rate the watch's syncs and a Bluetooth strap
//! bring, every game move and catch-up pump scored by the verdict rule once it has ended and its window has passed,
//! the pump's best verdict kept for the mirror's check mark and the apps, and the null windows that calibrate each
//! source's rise. Every move of a pump is measured against the minute before the pump's first go (the rule's
//! pre-pump median), on every source it has, and counts when either proves it (a strap reads through a push-up that a
//! wrist loses). No clock or file here: the session passes the time in (milliseconds on the helper's clock), and the
//! app saves the book as `Book::FILE` when it changes.

use std::collections::{BTreeMap, VecDeque};

use serde::{Deserialize, Serialize};

use crate::codes::kind;
use crate::verdict::{self, Class, Input, Outcome, Verdict};

/// How long heart samples stay: a catch-up pump's `pump_done` comes when the device finished it, and a null window's
/// baseline starts 11 minutes before a go.
const KEEP_MS: i64 = 60 * 60_000;
/// A move's verdict waits this long past its window for a watch's last samples (a 5-s batch and its delivery).
pub const SETTLE_AFTER_MS: i64 = 10_000;
/// A watch in use whose windows have not reached a move's window yet (a stalled sync, a relay through the phone) is
/// waited for this long past the window before the move settles without them.
const WATCH_LATE_MS: i64 = 60_000;
/// A source heard from this long before a move's go to the end of its window had a say in it: a watch sends a window
/// every 5 s through its play session, readings or none, and a strap reads once a second.
const IN_USE_MS: i64 = 5 * 60_000;
/// A move whose end never came (the strip went out of sight) settles this long after its go, and a wrist scores an
/// after-set move from its go then.
const SET_MAX_MS: i64 = 10 * 60_000;
/// A null window counts only when no move started from this long before its baseline to its end: the heart rate
/// takes a few minutes to settle after a pump.
const NULL_CLEAR_MS: i64 = 4 * 60_000;
/// The pumps whose proof the mirror carries: the newest, plenty for the sheet's Log of 20 per character.
pub const MIRROR_KEPT: usize = 200;
/// The pumps whose proof the book keeps.
const KEPT: usize = 5_000;
/// The latest pumps whose first go the book remembers.
const FIRSTS_KEPT: usize = 64;
const S: f64 = 1_000.0;

/// Where a heart rate came from.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum HrSource {
	/// A watch's 5-s samples, through its sync.
	Watch,
	/// A Bluetooth strap on the helper (the WHOOP's broadcast, a Polar H10).
	Strap,
}

/// A pump's heart-rate proof: the best verdict its moves got, as the mirror, the history and the socket carry it.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Proof {
	pub pump_id: String,
	pub verdict: Verdict,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub peak: Option<u16>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub delta: Option<i32>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub source: Option<HrSource>,
}

/// Which of two readings stands: the higher verdict, then the larger rise.
fn rank(verdict: Verdict, delta: Option<i32>) -> (Verdict, i32) {
	(verdict, delta.unwrap_or(i32::MIN))
}

/// The class the verdict scores a row kind as: a rep or cardio move is dynamic, a hold or a flow isometric or mobility.
#[must_use]
pub fn class_of(row_kind: u8) -> Class {
	if row_kind == kind::HOLD || row_kind == kind::FLOW { Class::Iso } else { Class::Dynamic }
}

/// A move (or a catch-up pump) waiting for its verdict.
#[derive(Debug, Clone, PartialEq)]
pub struct Move {
	pub pump_id: String,
	/// The pump's row time: its offered time (a game pump) or its start (a catch-up pump), the mirror's key.
	pub ts: u32,
	/// The move's slot; none for a catch-up pump.
	pub slot: Option<u8>,
	pub class: Class,
	/// A wrist scores it on the 30 s after its set (`moves::after_set`).
	pub after_set: bool,
	/// The go, in milliseconds on the helper's clock.
	pub go: i64,
	/// The set's end, once the game left the move (its count being typed, the pump gone on).
	pub end: Option<i64>,
	/// What the device measured itself (a catch-up pump's `pump_done` from a watch).
	pub measured: Option<Outcome>,
}

/// A move in the book: what the book knew of it at its go.
#[derive(Debug, Clone)]
struct Waiting {
	m: Move,
	/// The pump's first go: the minute before it is the baseline for every move of the pump.
	first_go: i64,
	/// The strap was on at the go: a source for this move, heard or not.
	strap: bool,
	/// Its set ended at its result view and its row is not written yet: the game's Back there still takes it back
	/// (`cancel`), so it settles only once `end` keeps it, or `SET_MAX_MS` after its go.
	unsaved: bool,
}

/// A move whose verdict is out.
#[derive(Debug, Clone, PartialEq)]
pub struct Settled {
	pub pump_id: String,
	pub ts: u32,
	pub slot: Option<u8>,
	pub outcome: Outcome,
	pub source: Option<HrSource>,
	/// The rise the source needed (its calibration); none when the device's own verdict or no source decided it.
	pub rise: Option<u16>,
}

/// One source's reading of a move.
enum Scored {
	/// A wrist waits for the set's end.
	Wait,
	/// The outcome, when its window ends, and the rise it needed.
	Done(Outcome, i64, u16),
}

#[derive(Serialize, Deserialize, Default)]
struct Saved {
	proofs: BTreeMap<u32, Proof>,
	nulls: BTreeMap<HrSource, VecDeque<i32>>,
}

/// The proofs, the null windows, the heart samples of the last hour and the moves waiting for a verdict.
#[derive(Default)]
pub struct Book {
	proofs: BTreeMap<u32, Proof>,
	nulls: BTreeMap<HrSource, VecDeque<i32>>,
	/// Heart samples by source and time (ms); a watch window without a reading is None (the watch was there).
	heart: BTreeMap<HrSource, BTreeMap<i64, Option<u16>>>,
	waiting: Vec<Waiting>,
	/// The goes of the last hour: a null window near another pump is no ordinary play.
	goes: VecDeque<i64>,
	/// The latest pumps' first goes, by row time: one baseline and one null window per pump, whatever its moves and
	/// Starts.
	firsts: VecDeque<(u32, i64)>,
	/// Since when the strap is on (ms), while it is.
	strap_since: Option<i64>,
}

impl Book {
	pub const FILE: &str = "proof.json";

	/// Reads the saved book from the data folder; an absent or damaged file is an empty one.
	#[must_use]
	pub fn load(dir: &std::path::Path) -> Self {
		let saved: Saved =
			std::fs::read(dir.join(Self::FILE)).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default();
		Self { proofs: saved.proofs, nulls: saved.nulls, ..Self::default() }
	}

	/// The proofs and the null windows as the app saves them.
	///
	/// # Errors
	/// Never for these types; serde's error otherwise.
	pub fn to_json(&self) -> serde_json::Result<Vec<u8>> {
		serde_json::to_vec(&Saved { proofs: self.proofs.clone(), nulls: self.nulls.clone() })
	}

	/// The strap was turned on or off at `at` (ms): a move whose go came while it was on has it as a source, heard or
	/// not.
	pub fn set_strap(&mut self, on: bool, at: i64) {
		if !on {
			self.strap_since = None;
		} else if self.strap_since.is_none() {
			self.strap_since = Some(at);
		}
	}

	/// A heart sample, a reading outside `verdict::BPM_RANGE` kept as none (the source was there); one sent again (a
	/// device resends a window until a response confirms it, with the time it was first sent with) counts once.
	pub fn heart(&mut self, source: HrSource, at: i64, bpm: Option<u16>) {
		let bpm = bpm.filter(|b| verdict::BPM_RANGE.contains(b));
		self.heart.entry(source).or_default().entry(at).or_insert(bpm);
	}

	/// A move's go: it waits for its verdict (a Start again replaces the one it cancelled). A pump's first go is kept
	/// as its baseline's end, and a game pump's first go adds each source's null window while the ten minutes before it
	/// are still kept (a catch-up pump, run away from the game, has no ordinary play before it). Returns whether a null
	/// window came in, which the app saves.
	pub fn start(&mut self, m: Move) -> bool {
		let mut added = false;
		let first_go = if let Some(&(_, go)) = self.firsts.iter().find(|&&(ts, _)| ts == m.ts) {
			go
		} else {
			if self.firsts.len() == FIRSTS_KEPT {
				self.firsts.pop_front();
			}
			self.firsts.push_back((m.ts, m.go));
			if m.slot.is_some() {
				added = self.add_nulls(m.go);
			}
			m.go
		};
		let strap = self.strap_since.is_some_and(|t| t <= m.go);
		self.waiting.retain(|w| !(w.m.pump_id == m.pump_id && w.m.slot == m.slot));
		self.goes.push_back(m.go);
		self.waiting.push(Waiting { m, first_go, strap, unsaved: false });
		added
	}

	/// The game kept a move (its row came, or it went on) at `at`: its set ended then, or at its result view when
	/// `end_unsaved` dated it there, and it settles from now on.
	pub fn end(&mut self, pump_id: &str, slot: u8, at: i64) {
		self.set_end(pump_id, slot, at, false);
	}

	/// The game shows a move's result view at `at`: its set ended, but until its row the game's Back takes it back
	/// (`cancel`), so it waits for `end` before it settles.
	pub fn end_unsaved(&mut self, pump_id: &str, slot: u8, at: i64) {
		self.set_end(pump_id, slot, at, true);
	}

	fn set_end(&mut self, pump_id: &str, slot: u8, at: i64, unsaved: bool) {
		if let Some(w) = self.waiting.iter_mut().find(|w| w.m.pump_id == pump_id && w.m.slot == Some(slot)) {
			w.m.end.get_or_insert(at);
			w.unsaved = unsaved;
		}
	}

	/// The move never counts (the game went back to its Ready, from its result view too before its row, or cut it
	/// short and owes it): its go was no pump.
	pub fn cancel(&mut self, pump_id: &str, slot: u8) {
		if let Some(i) = self.waiting.iter().position(|w| w.m.pump_id == pump_id && w.m.slot == Some(slot)) {
			let w = self.waiting.remove(i);
			if let Some(g) = self.goes.iter().position(|&g| g == w.m.go) {
				self.goes.remove(g);
			}
		}
	}

	/// A pump's proof, by its row time.
	#[must_use]
	pub fn proof(&self, ts: u32) -> Option<&Proof> {
		self.proofs.get(&ts)
	}

	/// The proofs of the pumps dated at or after `ts`, oldest first (a history page's `proofs`).
	pub fn proofs_from(&self, ts: u32) -> impl Iterator<Item = (u32, &Proof)> {
		self.proofs.range(ts..).map(|(&ts, p)| (ts, p))
	}

	/// The newest `MIRROR_KEPT` proofs, oldest first: the mirror's `proof` table.
	#[must_use]
	pub fn mirror(&self) -> Vec<(u32, &Proof)> {
		let skip = self.proofs.len().saturating_sub(MIRROR_KEPT);
		self.proofs.iter().skip(skip).map(|(&ts, p)| (ts, p)).collect()
	}

	/// Proofs a file holds (a mirror written before the data folder was lost): the ones the book lacks. Returns whether
	/// any came in.
	pub fn adopt(&mut self, held: &BTreeMap<u32, Proof>) -> bool {
		let mut taken = false;
		for (&ts, p) in held {
			if let std::collections::btree_map::Entry::Vacant(e) = self.proofs.entry(ts) {
				e.insert(p.clone());
				taken = true;
			}
		}
		self.trim();
		taken
	}

	/// A reset: the pumps before it went with their rows.
	pub fn forget_before(&mut self, ts: u32) {
		self.proofs.retain(|&t, _| t >= ts);
		self.waiting.retain(|w| w.m.ts >= ts);
	}

	fn trim(&mut self) {
		while self.proofs.len() > KEPT {
			self.proofs.pop_first();
		}
	}

	/// Whether a source had a say in a move: heard from `IN_USE_MS` before its go to the end of its minute, or the
	/// strap on at the go. A source heard only later (a strap turned on after the pump) had none.
	fn present(&self, source: HrSource, w: &Waiting) -> bool {
		let (from, to) = (w.m.go - IN_USE_MS, w.m.go + window_ms());
		let heard = self.heart.get(&source).is_some_and(|h| h.range(from..=to).next().is_some());
		heard || (source == HrSource::Strap && w.strap)
	}

	/// The newest time a watch's windows reached.
	fn watch_reached(&self) -> Option<i64> {
		self.heart.get(&HrSource::Watch).and_then(|h| h.last_key_value()).map(|(&at, _)| at)
	}

	/// A source's readings from `from` on (ms) as the verdict takes them: seconds from `go`.
	fn samples(&self, source: HrSource, go: i64, from: i64) -> Vec<verdict::Sample> {
		self.heart.get(&source).map_or_else(Vec::new, |h| {
			h.range(from..)
				.filter_map(|(&at, bpm)| bpm.map(|bpm| verdict::Sample { t: seconds(at - go), bpm }))
				.collect()
		})
	}

	fn score(&self, w: &Waiting, source: HrSource, now: i64) -> Scored {
		let m = &w.m;
		let after = m.after_set && source == HrSource::Watch;
		// a wrist that lost the set's end scores the move from its go once `SET_MAX_MS` has passed
		if after && m.end.is_none() && now - m.go < SET_MAX_MS {
			return Scored::Wait;
		}
		let set_end = m.end.filter(|_| after);
		let nulls: Vec<i32> = self.nulls.get(&source).map(|n| n.iter().copied().collect()).unwrap_or_default();
		let rise = m.class.rise(&nulls);
		let samples = self.samples(source, m.go, w.first_go - baseline_ms());
		let base = verdict::baseline(&samples, seconds(w.first_go - m.go));
		let input = Input {
			class: m.class,
			source: true,
			set_end: set_end.map(|e| seconds(e - m.go)),
			samples,
			rise: Some(rise),
		};
		let closes = set_end.map_or(m.go + window_ms(), |e| e + after_set_ms());
		Scored::Done(verdict::verdict_over(&input, base), closes, rise)
	}

	/// The moves whose verdict is out at `now`: a move settles once it has ended and the game kept it (or
	/// `SET_MAX_MS` passed), at once when proven, else once its window and the wait for late samples have passed, and a
	/// watch in use has sent its windows up to the window's end or a minute more went by (no source at all: "no HR").
	/// The device then shows it on the move's card, which exists from its end. Each pump keeps its best verdict.
	/// Returns the settled moves and whether a proof changed.
	pub fn tick(&mut self, now: i64) -> (Vec<Settled>, bool) {
		let mut settled = Vec::new();
		let mut changed = false;
		let mut i = 0;
		while i < self.waiting.len() {
			let w = &self.waiting[i];
			let mut best: Option<(Outcome, Option<HrSource>, Option<u16>)> =
				w.m.measured.map(|o| (o, Some(HrSource::Watch), None));
			// the latest window of the sources it has; a minute from the go with none
			let mut closes: Option<i64> = None;
			let mut waits = false;
			let watch = self.present(HrSource::Watch, w);
			for source in [HrSource::Strap, HrSource::Watch] {
				if !self.present(source, w) {
					continue;
				}
				match self.score(w, source, now) {
					Scored::Wait => waits = true,
					Scored::Done(outcome, end, rise) => {
						closes = Some(closes.map_or(end, |c| c.max(end)));
						if best.is_none_or(|(b, _, _)| rank(outcome.verdict, outcome.delta) > rank(b.verdict, b.delta))
						{
							best = Some((outcome, Some(source), Some(rise)));
						}
					}
				}
			}
			let proven = best.is_some_and(|(o, _, _)| o.verdict == Verdict::Proven);
			let closes = closes.unwrap_or(w.m.go + window_ms());
			let ended = (w.m.end.is_some() && !w.unsaved) || now - w.m.go >= SET_MAX_MS;
			let late = watch && self.watch_reached().is_none_or(|at| at < closes) && now < closes + WATCH_LATE_MS;
			let due = ended && (proven || (!waits && !late && now >= closes + SETTLE_AFTER_MS));
			if !due {
				i += 1;
				continue;
			}
			let m = self.waiting.remove(i).m;
			let (outcome, source, rise) =
				best.unwrap_or((Outcome { verdict: Verdict::NoHr, peak: None, delta: None }, None, None));
			let proof = Proof {
				pump_id: m.pump_id.clone(),
				verdict: outcome.verdict,
				peak: outcome.peak,
				delta: outcome.delta,
				source,
			};
			if self.proofs.get(&m.ts).is_none_or(|old| rank(proof.verdict, proof.delta) > rank(old.verdict, old.delta))
			{
				self.proofs.insert(m.ts, proof);
				self.trim();
				changed = true;
			}
			settled.push(Settled { pump_id: m.pump_id, ts: m.ts, slot: m.slot, outcome, source, rise });
		}
		self.prune(now);
		(settled, changed)
	}

	/// Each source's null window before a pump's go, when that stretch was ordinary play: no move started from
	/// `NULL_CLEAR_MS` before its baseline to its end. Returns whether one came in.
	fn add_nulls(&mut self, go: i64) -> bool {
		let from = go - null_before_ms() - baseline_ms() - NULL_CLEAR_MS;
		let to = go - null_before_ms() + window_ms();
		if self.goes.iter().any(|&g| g >= from && g <= to) {
			return false;
		}
		let mut added = false;
		for source in [HrSource::Strap, HrSource::Watch] {
			if let Some(rise) = verdict::null_rise(&self.samples(source, go, go - null_before_ms() - baseline_ms())) {
				let nulls = self.nulls.entry(source).or_default();
				nulls.push_back(rise);
				while nulls.len() > verdict::NULL_KEPT {
					nulls.pop_front();
				}
				added = true;
			}
		}
		added
	}

	fn prune(&mut self, now: i64) {
		for samples in self.heart.values_mut() {
			*samples = samples.split_off(&(now - KEEP_MS));
		}
		while self.goes.front().is_some_and(|&g| g < now - KEEP_MS) {
			self.goes.pop_front();
		}
	}
}

#[expect(clippy::cast_precision_loss, reason = "milliseconds within an hour of a go")]
fn seconds(ms: i64) -> f64 {
	ms as f64 / S
}

#[expect(clippy::cast_possible_truncation, reason = "the rule's windows are whole seconds")]
fn ms(s: f64) -> i64 {
	(s * S) as i64
}

fn window_ms() -> i64 {
	ms(verdict::WINDOW_S)
}

fn baseline_ms() -> i64 {
	ms(verdict::BASELINE_S)
}

fn after_set_ms() -> i64 {
	ms(verdict::AFTER_SET_S)
}

fn null_before_ms() -> i64 {
	ms(verdict::NULL_BEFORE_S)
}
