//! The helper's row store: every row it has seen, in the order it saw them, kept in `rows.jsonl` in the data folder
//! (rust-helper `protocol-and-mirror.md`, "The row store") and written whole into the mirror. A strip row seen again
//! (the same load id, event counter and row: a frame shown again, or the last painted frame after a restart) is stored
//! once; a row of another load that happens to share a load id and a counter is a different row and stays. It keeps
//! the owed queue as the addon's fold reads it, and on a reset row it forgets every row dated before it (a reset read
//! late from a file keeps the rows the helper had seen after it).

use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::codes::{kind, outcome};
use crate::jsonl::JsonLines;
use crate::protocol::RowRecord;
use crate::row::Row;

pub const FILE: &str = "rows.jsonl";

/// The rows, the strip rows seen, the owed queue and the catch-up pumps paid.
#[derive(Default)]
pub struct Store {
	records: Vec<RowRecord>,
	seen: HashSet<(u16, u16, Row)>,
	/// `ns.OwedKey` → the owed or banked row, until a paid row names it (`DB.lua` `applyRow`): a move's payment, or
	/// since 2026-10-04 the player's skip in the game (the paid row alone, flag [`crate::codes::flags::SKIPPED`]),
	/// which clears it the same way.
	owed: HashMap<(u32, u32, u8), Row>,
	/// A paid row seen before its owed row (the logs merged out of order): that owed row is then skipped.
	paid_early: HashSet<(u32, u32, u8)>,
	/// The catch-up pumps whose rows are stored (a device resends a pump until the helper confirms it).
	pumps: HashSet<String>,
	/// The latest reset row's time: a row dated before it belongs to the deleted save.
	floor: Option<u32>,
	file: JsonLines,
}

/// What `Store::add` did with a row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Added {
	/// Appended.
	Row,
	/// A reset row: every row before it is gone.
	Reset,
	/// Already stored (the same strip row seen again), or dated before the last reset.
	Skipped,
}

impl Store {
	/// Loads `rows.jsonl` from the data folder (an absent file is an empty store). A line that does not read (damaged
	/// bytes, a write cut short) is skipped and counted, so one bad line never loses the rest, and the reset rule applies
	/// as the file is read: rows before a reset row are dropped even when the compaction after it failed.
	///
	/// # Errors
	/// A file that exists but cannot be read.
	pub fn open(dir: &Path) -> std::io::Result<(Self, usize)> {
		let (file, records, damaged) = JsonLines::open::<RowRecord>(dir, FILE)?;
		let mut store = Self { file, ..Self::default() };
		for r in records {
			if r.row().kind == kind::RESET {
				store.reset_to(r);
			} else {
				store.records.push(r);
			}
		}
		store.reindex();
		Ok((store, damaged))
	}

	/// A store that keeps its rows in memory only (tests).
	#[must_use]
	pub fn in_memory() -> Self {
		Self::default()
	}

	/// The reset row first, then the rows dated at or after it in their order: a reset forgets the save before it, and
	/// the rows written since (seen before the reset arrived, from a file read late) stay.
	fn reset_to(&mut self, reset: RowRecord) {
		let ts = reset.row_ts;
		let kept = std::mem::take(&mut self.records).into_iter().filter(|r| r.row_ts >= ts);
		self.records = std::iter::once(reset).chain(kept).collect();
	}

	/// Builds the lookups again from the records (after a load or a reset).
	fn reindex(&mut self) {
		self.seen.clear();
		self.owed.clear();
		self.paid_early.clear();
		self.pumps.clear();
		self.floor = None;
		for r in std::mem::take(&mut self.records) {
			self.index(&r);
			self.records.push(r);
		}
	}

	fn index(&mut self, r: &RowRecord) {
		let row = r.row();
		if let (Some(load), Some(event)) = (r.load_id, r.event_no) {
			self.seen.insert((load, event, row));
		}
		if let Some(id) = &r.pump {
			self.pumps.insert(id.clone());
		}
		if row.kind == kind::RESET {
			self.floor = Some(self.floor.map_or(row.ts, |f| f.max(row.ts)));
			return;
		}
		if !kind::is_move(row.kind) {
			return;
		}
		let key = row.owed_key();
		match row.outcome {
			outcome::PAID => {
				if self.owed.remove(&key).is_none() {
					self.paid_early.insert(key);
				}
			}
			outcome::OWED | outcome::BANKED => {
				if self.paid_early.remove(&key) {
					return;
				}
				self.owed.insert(key, row);
			}
			_ => {}
		}
	}

	/// Whether a catch-up pump's rows are stored already.
	#[must_use]
	pub fn has_pump(&self, id: &str) -> bool {
		self.pumps.contains(id)
	}

	/// Whether a row belongs to the save the store keeps: a row dated before the latest reset belongs to the deleted
	/// one (a reset row older than the latest one too).
	#[must_use]
	pub fn accepts(&self, row: &Row) -> bool {
		self.floor.is_none_or(|f| row.ts >= f && (row.kind != kind::RESET || row.ts > f))
	}

	/// Adds a row: appended (and written to `rows.jsonl`), or on a reset row the store keeps it and the rows dated at
	/// or after it (the reset row is appended first, then the file is compacted; a failed compaction leaves a file
	/// `open` still reads right).
	///
	/// # Errors
	/// The file cannot be written; the row stays in memory and reaches the mirror all the same.
	pub fn add(&mut self, r: RowRecord) -> (Added, std::io::Result<()>) {
		let row = r.row();
		if let (Some(load), Some(event)) = (r.load_id, r.event_no)
			&& self.seen.contains(&(load, event, row))
		{
			return (Added::Skipped, Ok(()));
		}
		if !self.accepts(&row) {
			return (Added::Skipped, Ok(()));
		}
		let written = self.file.append(&r);
		if row.kind == kind::RESET {
			self.reset_to(r);
			self.reindex();
			return (Added::Reset, written.and_then(|()| self.file.rewrite(&self.records)));
		}
		self.index(&r);
		self.records.push(r);
		(Added::Row, written)
	}

	/// Files the rows of one character under another (a placeholder the helper learnt is a character it knew,
	/// `Session::settle_aliases`), in memory and in `rows.jsonl`, rewritten atomically even when none moves in memory:
	/// a rewrite that failed before left them under the old one in the file.
	///
	/// # Errors
	/// The file cannot be rewritten; the store in memory has them filed all the same.
	pub fn rechar(&mut self, from: u8, to: u8) -> std::io::Result<()> {
		for r in self.records.iter_mut().filter(|r| r.char == from) {
			r.char = to;
		}
		self.reindex();
		self.file.rewrite(&self.records)
	}

	/// The latest reset row's time: rows and profiles from before it belong to the deleted save.
	#[must_use]
	pub fn floor(&self) -> Option<u32> {
		self.floor
	}

	/// The records in the order the store saw them (a save taken in, a late catch-up or a paid row brings rows dated
	/// before newer ones).
	#[must_use]
	pub fn records(&self) -> &[RowRecord] {
		&self.records
	}

	/// Which order the records count in (`GET /v1/history`'s `store`): the first record's `ts_helper` and `row_ts`. A
	/// reset puts its own row first and a new data folder starts empty, so it changes whenever the order starts again;
	/// empty while the store is.
	#[must_use]
	pub fn id(&self) -> String {
		self.records.first().map(|r| format!("{}/{}", r.ts_helper, r.row_ts)).unwrap_or_default()
	}

	/// The rows in the order they were seen, as the mirror writes them.
	#[must_use]
	pub fn rows(&self) -> Vec<Row> {
		self.records.iter().map(RowRecord::row).collect()
	}

	/// Whether an owed or banked row with this key waits to be paid.
	#[must_use]
	pub fn is_owed(&self, key: (u32, u32, u8)) -> bool {
		self.owed.contains_key(&key)
	}

	/// The owed queue oldest first, as `ns.OwedList` sorts it: by time, then slot, then quest.
	#[must_use]
	pub fn owed(&self) -> Vec<Row> {
		let mut list: Vec<Row> = self.owed.values().copied().collect();
		list.sort_by_key(|r| (r.ts, r.slot, r.quest_id));
		list
	}
}
