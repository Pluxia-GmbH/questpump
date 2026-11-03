//! The sheet log: every sheet summary the helper read from the strip's `sheet` event that differs from that
//! character's last one, in the order it read them, kept in `sheets.jsonl` in the data folder and served page by page
//! as `GET /v1/sheets` (rust-helper `protocol-and-mirror.md`, "The sheet log"), so the phone's charts plot the sheet as
//! the addon folded it and never fold rows. The addon paints its sheet at every login and after every pump, so a
//! sheet that repeats the last one apart from when it was folded (`at`) and saved (`saved`) adds nothing. A reset
//! forgets the sheets folded before it, as the row store forgets the rows.

use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;
use serde_json::Value;
use serde_json::value::RawValue;

use crate::jsonl::JsonLines;
use crate::link::{self, SheetsPage};
use crate::protocol::SheetRecord;

pub const FILE: &str = "sheets.jsonl";

/// The sheets read, in their order, and each character's last one.
#[derive(Default)]
pub struct SheetLog {
	records: Vec<SheetRecord>,
	/// Each character's last logged sheet without `at` and `saved`, by the helper's own numbering across every install
	/// (`session::People`), the one `SheetRecord::char` holds: a sheet equal to it is not logged again.
	last: HashMap<u8, Value>,
	/// The latest reset's time: only a sheet folded at or after it is logged (the row store's floor).
	floor: Option<u32>,
	file: JsonLines,
}

/// A sheet without the fields that change at every paint of the same fold.
fn content(sheet: &RawValue) -> Value {
	let mut v = serde_json::from_str(sheet.get()).unwrap_or(Value::Null);
	if let Some(fields) = v.as_object_mut() {
		fields.remove("at");
		fields.remove("saved");
	}
	v
}

/// The one field of a sheet a reset reads.
#[derive(Deserialize)]
struct Folded {
	at: Option<u64>,
}

/// Whether a sheet was folded at or after `ts` (a reset's time): one without an `at` belongs to no save.
fn folded_since(r: &SheetRecord, ts: u32) -> bool {
	serde_json::from_str::<Folded>(r.sheet.get()).ok().and_then(|f| f.at).is_some_and(|at| at >= u64::from(ts))
}

impl SheetLog {
	/// Loads `sheets.jsonl` from the data folder (an absent file is an empty log). A line that does not read is skipped
	/// and counted, and the row store's latest reset (`floor`) applies as the file is read, so the sheets before it stay
	/// out even when the compaction after it failed.
	///
	/// # Errors
	/// A file that exists but cannot be read.
	pub fn open(dir: &Path, floor: Option<u32>) -> std::io::Result<(Self, usize)> {
		let (file, mut records, damaged) = JsonLines::open::<SheetRecord>(dir, FILE)?;
		if let Some(ts) = floor {
			records.retain(|r| folded_since(r, ts));
		}
		let mut log = Self { records, floor, file, ..Self::default() };
		log.reindex();
		Ok((log, damaged))
	}

	/// A log that keeps its sheets in memory only: a new session's, and the run's when the file cannot be read.
	#[must_use]
	pub fn in_memory() -> Self {
		Self::default()
	}

	fn reindex(&mut self) {
		self.last = self.records.iter().map(|r| (r.char, content(&r.sheet))).collect();
	}

	/// Whether a sheet belongs to the save that holds now: folded at or after the latest reset, as `open` and `reset`
	/// keep them (a copy of the deleted save's sheet can finish painting after the reset row; the sibling of the row
	/// store's `accepts`).
	#[must_use]
	pub fn accepts(&self, r: &SheetRecord) -> bool {
		self.floor.is_none_or(|ts| folded_since(r, ts))
	}

	/// Logs a sheet read from the strip, unless it repeats its character's last one apart from `at` and `saved` or the
	/// save it was folded in was reset since (`accepts`), so the log never shrinks when it is next opened.
	///
	/// # Errors
	/// The file cannot be written; the sheet stays in memory and is served all the same.
	pub fn add(&mut self, r: SheetRecord) -> std::io::Result<()> {
		if !self.accepts(&r) {
			return Ok(());
		}
		let seen = content(&r.sheet);
		if self.last.get(&r.char) == Some(&seen) {
			return Ok(());
		}
		let written = self.file.append(&r);
		self.last.insert(r.char, seen);
		self.records.push(r);
		written
	}

	/// A reset at `ts`: only the sheets folded at or after it stay, and the file is rewritten atomically.
	///
	/// # Errors
	/// The file cannot be rewritten; the log in memory is compacted all the same, and `open` compacts the file's
	/// sheets again.
	pub fn reset(&mut self, ts: u32) -> std::io::Result<()> {
		self.floor = Some(self.floor.map_or(ts, |f| f.max(ts)));
		self.records.retain(|r| folded_since(r, ts));
		self.reindex();
		self.file.rewrite(&self.records)
	}

	/// Files the sheets of one character under another (the row store's `rechar`), in memory and in `sheets.jsonl`,
	/// rewritten atomically even when none moves in memory.
	///
	/// # Errors
	/// The file cannot be rewritten; the log in memory has them filed all the same.
	pub fn rechar(&mut self, from: u8, to: u8) -> std::io::Result<()> {
		for r in self.records.iter_mut().filter(|r| r.char == from) {
			r.char = to;
		}
		self.reindex();
		self.file.rewrite(&self.records)
	}

	#[must_use]
	pub fn records(&self) -> &[SheetRecord] {
		&self.records
	}

	/// `GET /v1/sheets`: the page after the `after` sheets a device holds, at most `size` (`link::page_size`:
	/// `SHEETS_PAGE`, or the request's limit). Its `store` is the first sheet's `ts_helper`, empty while the log is.
	#[must_use]
	pub fn page(&self, after: u64, size: usize) -> SheetsPage {
		let (sheets, next, more) = link::page(&self.records, after, size);
		let store = self.records.first().map(|r| r.ts_helper.clone()).unwrap_or_default();
		SheetsPage { store, next, more, sheets }
	}
}
