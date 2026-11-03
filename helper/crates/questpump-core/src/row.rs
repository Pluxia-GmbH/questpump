//! The addon's row grammar (docs/plan.md slice 1; `DB.lua` `ns.FIELDS`, `ns.EncodeRow`):
//! `ts|char|questID|tier|slot|ladder|rung|move|kind|reps|seconds|outcome|flags`, non-negative integers only. The helper
//! writes a row into the mirror as this exact string, because the addon's merge unions rows by string: a row carried
//! with another time or another field comes back as a second row.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::codes::{flags, kind, outcome};

/// One row, every field of the grammar in its own number.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Row {
	pub ts: u32,
	pub char: u8,
	pub quest_id: u32,
	pub tier: u8,
	pub slot: u8,
	pub ladder: u8,
	pub rung: u8,
	#[serde(rename = "move")]
	pub move_id: u16,
	pub kind: u8,
	pub reps: u16,
	pub seconds: u16,
	pub outcome: u8,
	pub flags: u16,
}

/// A string that is not a row of the grammar.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("not a row of the grammar: {0:?}")]
pub struct RowError(pub String);

impl Row {
	/// The owed queue's key (`ns.OwedKey`): a paid row repeats its owed row's time, quest and slot.
	#[must_use]
	pub fn owed_key(&self) -> (u32, u32, u8) {
		(self.ts, self.quest_id, self.slot)
	}

	/// The reason in flag bits 0-2 (a stop's, or a bank's on a banked row).
	#[must_use]
	pub fn reason(&self) -> u16 {
		self.flags & flags::REASON_MASK
	}

	/// A move done (`RowOutcome.performed`): a typed, done or stopped move; owed, banked and paid rows are none, and so
	/// are the rows that are no move (a pump's, a rung's, a check-in's).
	#[must_use]
	pub fn performed(&self) -> bool {
		kind::is_move(self.kind) && outcome::is_performed(self.outcome)
	}

	/// Whether this row locks every pump until the next game day, as the addon's fold reads it (`DB.lua` `applyRow`):
	/// a move row other than a payment whose reason is "felt unwell", or a pump or level-up banked for it.
	#[must_use]
	pub fn locks(&self) -> bool {
		let unwell = self.reason() == flags::UNWELL;
		if kind::is_move(self.kind) {
			return unwell && self.outcome != outcome::PAID;
		}
		unwell && kind::is_pump(self.kind) && self.outcome == outcome::BANKED
	}
}

impl fmt::Display for Row {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
			self.ts,
			self.char,
			self.quest_id,
			self.tier,
			self.slot,
			self.ladder,
			self.rung,
			self.move_id,
			self.kind,
			self.reps,
			self.seconds,
			self.outcome,
			self.flags
		)
	}
}

impl std::str::FromStr for Row {
	type Err = RowError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let bad = || RowError(s.to_owned());
		let parts: Vec<&str> = s.split('|').collect();
		if parts.len() != 13 || parts.iter().any(|p| p.is_empty() || !p.bytes().all(|b| b.is_ascii_digit())) {
			return Err(bad());
		}
		let n = |i: usize| -> Result<u64, RowError> { parts[i].parse::<u64>().map_err(|_| bad()) };
		let fit = |v: u64| -> Result<u32, RowError> { u32::try_from(v).map_err(|_| bad()) };
		Ok(Self {
			ts: fit(n(0)?)?,
			char: u8::try_from(n(1)?).map_err(|_| bad())?,
			quest_id: fit(n(2)?)?,
			tier: u8::try_from(n(3)?).map_err(|_| bad())?,
			slot: u8::try_from(n(4)?).map_err(|_| bad())?,
			ladder: u8::try_from(n(5)?).map_err(|_| bad())?,
			rung: u8::try_from(n(6)?).map_err(|_| bad())?,
			move_id: u16::try_from(n(7)?).map_err(|_| bad())?,
			kind: u8::try_from(n(8)?).map_err(|_| bad())?,
			reps: u16::try_from(n(9)?).map_err(|_| bad())?,
			seconds: u16::try_from(n(10)?).map_err(|_| bad())?,
			outcome: u8::try_from(n(11)?).map_err(|_| bad())?,
			flags: u16::try_from(n(12)?).map_err(|_| bad())?,
		})
	}
}
