//! The helper's store of app rows (docs/plan.md slice 26; rust-helper `protocol-and-mirror.md`, "The row store"):
//! every row a phone's app wrote and sent in `POST /v1/sync` (its own pumps, its pain flags and check-in answers, its
//! start rungs, its felt-unwell stops), kept in `app.jsonl` in the data folder apart from the game's rows, since an app
//! row never pays or owes a move, and written into every install's mirror beside them, so the game folds one record
//! per move whether the count was typed in the game or in the app. A row's identity is its text and its copy number
//! (the n-th copy of that text among its device's rows), the rule the addon's merge counts by: a row sent again is
//! kept once, and two rows alike stay two. The game store's latest reset is its floor too (one floor, `Store::floor`):
//! `/qp reset all` is the player's, everywhere. Every paired phone reads the store back page by page (`GET
//! /v1/app_rows`, slice 28), the helper's copy: a phone that lost its rows, or a second phone paired to this helper,
//! gets the app rows of every phone. A phone's "Start over in the app" (slice 28) comes as a reset row too since
//! 2026-10-09, kept beside the rest and served with them, never written into the mirror (the game keeps what it has):
//! the app's own history starts at the latest one (`start_over`, `after_start_over`), on every phone and on the
//! computer.

use std::collections::HashSet;
use std::path::Path;

use crate::codes::kind;
use crate::jsonl::JsonLines;
use crate::link::{self, AppRow, AppRowsPage};
use crate::protocol::RowRecord;
use crate::row::Row;

pub const FILE: &str = "app.jsonl";

/// Whether an app row counts in the app's own history after its latest Start over at `start_over` (unix seconds; the
/// phones' `Backup.Floor.drops`, the Android core's `Backup.KEPT_KINDS`, `tools/apppump.lua` `FLOOR_KEEPS`): one dated
/// at or after it, and before it the pain, focus and clear rows (the body's state, the account's focus, a record cleared
/// or a ladder started over, which the game keeps too) and, since slice 44 (contract/fixtures/backup.json `imports`
/// case 10), every correction and load row, its set's or not: a correction carries the time of the set it names, so one
/// written after a Start over of a game set from before it is the player's now; one whose app set the floor dropped
/// names nothing. Never a reset row itself. Every row counts before the first Start over.
#[must_use]
pub fn after_start_over(row: &Row, start_over: Option<u32>) -> bool {
	row.kind != kind::RESET
		&& start_over.is_none_or(|t| {
			row.ts >= t || matches!(row.kind, kind::PAIN | kind::FOCUS | kind::CLEAR | kind::CORRECTION | kind::LOAD)
		})
}

/// The app rows in the order the helper took them.
#[derive(Default)]
pub struct AppRows {
	records: Vec<RowRecord>,
	/// Each kept row's text and copy number.
	kept: HashSet<(Row, u16)>,
	/// The latest reset the store was opened or compacted with, part of its id (`id`).
	floor: Option<u32>,
	file: JsonLines,
	/// The file lacks a row the store keeps (one of the computer's own whose append failed, `add_own`): the next write
	/// rewrites it whole.
	behind: bool,
	/// The latest Start over among the records (`start_over`) and how many of them are Start overs, kept as rows come
	/// and go, so the readers that ask at every look (`Session::app_mark`, the desktop app's store mark) never walk the
	/// store.
	latest_start_over: Option<u32>,
	start_overs: usize,
}

impl AppRows {
	/// Loads `app.jsonl` from the data folder (an absent file is an empty store), keeping only the rows dated at or
	/// after the game store's latest reset (`floor`), so a compaction that failed never brings the old rows back. A
	/// line that does not read is skipped and counted.
	///
	/// # Errors
	/// A file that exists but cannot be read.
	pub fn open(dir: &Path, floor: Option<u32>) -> std::io::Result<(Self, usize)> {
		let (file, records, damaged) = JsonLines::open::<RowRecord>(dir, FILE)?;
		let mut store = Self { floor, file, ..Self::default() };
		for r in records.into_iter().filter(|r| floor.is_none_or(|f| r.row_ts >= f)) {
			if r.copy.is_some_and(|n| store.kept.insert((r.row(), n))) {
				store.count(&r);
				store.records.push(r);
			}
		}
		Ok((store, damaged))
	}

	/// A kept record into the Start over counts.
	fn count(&mut self, r: &RowRecord) {
		if r.kind == kind::RESET {
			self.start_overs += 1;
			self.latest_start_over = self.latest_start_over.max(Some(r.row_ts));
		}
	}

	/// A store that keeps its rows in memory only (tests, the stand-in helper).
	#[must_use]
	pub fn in_memory() -> Self {
		Self::default()
	}

	/// Keeps an app row a phone sent or a file held (its `copy` set): false when that copy is kept already. The session
	/// refuses one dated before the game store's latest reset first (`Session::keep_app`).
	///
	/// # Errors
	/// The file cannot be written: the row is not kept (false), unlike a game row (`Store::add`), since the phone holds
	/// it and sends it again until a sync confirms it (a file's comes again with the file), and the next try writes it.
	pub fn add(&mut self, r: RowRecord) -> (bool, std::io::Result<()>) {
		self.keep(r, false)
	}

	/// Keeps one of the computer's own app rows (the desktop app's, `Session::keep_computer`) as `add` keeps a phone's,
	/// but nothing sends it again: like a game row (`Store::add`), it is kept when the file cannot take it.
	///
	/// # Errors
	/// The file cannot be written: the row stays in memory all the same, reaches the mirror and the pages, and the next
	/// write rewrites the file whole with it.
	pub fn add_own(&mut self, r: RowRecord) -> (bool, std::io::Result<()>) {
		self.keep(r, true)
	}

	fn keep(&mut self, r: RowRecord, own: bool) -> (bool, std::io::Result<()>) {
		let row = r.row();
		let Some(copy) = r.copy else { return (false, Ok(())) };
		if self.kept.contains(&(row, copy)) {
			return (false, Ok(()));
		}
		let written = if self.behind {
			let whole = self.file.rewrite(self.records.iter().chain([&r]));
			self.behind = whole.is_err();
			whole
		} else {
			self.file.append(&r)
		};
		if written.is_err() && !own {
			return (false, written);
		}
		self.behind |= written.is_err();
		self.kept.insert((row, copy));
		self.count(&r);
		self.records.push(r);
		(true, written)
	}

	/// The copy number the next copy of a row's text takes: the first one the store does not keep (the desktop app's
	/// rows, `Session::keep_computer`, which the helper numbers itself).
	#[must_use]
	pub fn next_copy(&self, row: &Row) -> u16 {
		(1..=u16::MAX).find(|&n| !self.kept.contains(&(*row, n))).unwrap_or(u16::MAX)
	}

	/// A reset at `ts` (`/qp reset all`, from any install): only the rows dated at or after it stay, and the file is
	/// rewritten atomically.
	///
	/// # Errors
	/// The file cannot be rewritten; the store in memory is compacted all the same, and `open` compacts the file's rows
	/// again.
	pub fn reset(&mut self, ts: u32) -> std::io::Result<()> {
		self.records.retain(|r| r.row_ts >= ts);
		self.kept = self.records.iter().filter_map(|r| Some((r.row(), r.copy?))).collect();
		let start_overs = || self.records.iter().filter(|r| r.kind == kind::RESET);
		(self.latest_start_over, self.start_overs) = (start_overs().map(|r| r.row_ts).max(), start_overs().count());
		self.floor = self.floor.max(Some(ts));
		let written = self.file.rewrite(&self.records);
		// a file rewritten whole holds every row the store keeps
		self.behind &= written.is_err();
		written
	}

	/// The records in the order the helper took them.
	#[must_use]
	pub fn records(&self) -> &[RowRecord] {
		&self.records
	}

	/// The latest "Start over in the app" a phone sent (its reset row's time, unix seconds): where the app's own
	/// history starts (`after_start_over`); none before the first.
	#[must_use]
	pub fn start_over(&self) -> Option<u32> {
		self.latest_start_over
	}

	/// How many records reach the game's mirror: every one but the Start overs (`Session::mirrored`).
	#[must_use]
	pub fn mirrored_len(&self) -> usize {
		self.records.len() - self.start_overs
	}

	/// The rows in the order the helper took them, as the mirror merges them with the game's.
	#[must_use]
	pub fn rows(&self) -> Vec<Row> {
		self.records.iter().map(RowRecord::row).collect()
	}

	/// The store's id (a sync response's `rows_store`, an app rows page's `store`): the first record's `ts_helper`,
	/// `row_ts` and copy number, then the latest reset once there is one, so it changes whenever the store starts again
	/// (a new data folder) and at every reset, even one that kept the first row: a row dated before the reset but
	/// taken after it (a second phone's older rows, a phone's restored backup) goes from the middle, which moves every
	/// later row's place in the pages. Empty while the store is. A phone that kept another id sends its rows again and
	/// reads the pages from 0.
	#[must_use]
	pub fn id(&self) -> String {
		let Some(first) = self.records.first() else { return String::new() };
		let id = format!("{}/{}/{}", first.ts_helper, first.row_ts, first.copy.unwrap_or_default());
		match self.floor {
			Some(floor) => format!("{id}/{floor}"),
			None => id,
		}
	}

	/// `GET /v1/app_rows` (slice 28): the page after the `after` rows a device holds, in the order the helper took
	/// them, each row's text as the store keeps it (stamped: `char` 0, the app bit and its device's) with its copy
	/// number; at most `size` rows (`link::page_size`: `HISTORY_PAGE`, or the request's limit).
	#[must_use]
	pub fn page(&self, after: u64, size: usize) -> AppRowsPage {
		let (records, next, more) = link::page(&self.records, after, size);
		// every kept record has its copy number (`add`, `open`)
		let rows = records.iter().filter_map(|r| Some(AppRow { row: r.row().to_string(), n: r.copy? })).collect();
		AppRowsPage { store: self.id(), next, more, rows }
	}
}
