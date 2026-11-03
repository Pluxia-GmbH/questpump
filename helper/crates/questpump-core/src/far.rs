//! Another computer's game rows (docs/plan.md slice 60, "The game"): the rows a computer of the same account sent in
//! its `rows` batches (`sync::RowsBody`'s `game.rows`, that computer's `rows.jsonl` in its own numbering of the
//! characters), kept in `far.jsonl` with the computer's install id (`hid`), the GUID the batch's `chars` named for the
//! row's character, and its copy number among the rows of the same text and character. A row is known by that text with
//! its character as its GUID (`FarRecord::key`), whichever install id brought it, so a batch taken in twice, or again
//! from a computer that came back with a new install id, keeps each row once, and two computers' rows never mix. The
//! session writes them into every install's mirror as rows of another source (`as_app`: the app's bit and the
//! computer's), each character found by its GUID (`FarRecord::guid`, else, for the batch of an older desktop app
//! without `chars`, the one that computer's `device` batch named for its index: `People::far`), so the game folds them
//! for its records, rungs, the sheet's gains, the day plan and the clocks and never for its owed queue: each computer
//! keeps its own. The rows that are a computer's own queue or its own state stay out (`carried`): its pauses, its owed,
//! banked and paid rows, and every body note (which its batches never carry either). Its reset is the player's: the
//! session takes it as this computer's own (`Session::take_far`) and keeps it here as one that came in (`came_in`), so
//! it is never sent again; the rows dated before the latest reset go (`FarRows::floor`).

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::backup::{GameRow, is_body_note};
use crate::codes::{flags, kind, outcome};
use crate::jsonl::JsonLines;
use crate::row::Row;

pub const FILE: &str = "far.jsonl";
/// A character's pause (`ns.Enum.kind.pause`): the computer's own, as its queue is.
const PAUSE: u8 = 15;
/// Every device's bit and the app's: what another computer's row loses and gains on its way into this one's game.
pub const DEVICE_BITS: u16 = flags::APP | flags::PHONE | flags::WATCH | flags::COMPUTER;

/// One kept row of another computer.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct FarRecord {
	/// The computer's install id (base64url).
	pub hid: String,
	/// The row in the addon's grammar, its character in that computer's numbering.
	pub row: String,
	/// Its copy number among the rows of the same text and character (`key`).
	pub n: u16,
	/// The GUID of the row's character, as the batch's `chars` named it; none for a row of no character and for one of
	/// an older desktop app's batch, whose character that computer's `device` batch names.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub guid: Option<String>,
}

/// Whose a row is, as `FarRecord::key` reads it: no character's, a character's by its GUID, or a character's that an
/// older batch named by its computer's index alone.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Who {
	Nobody,
	Guid(String),
	Index(String, u8),
}

impl FarRecord {
	/// The row it keeps; none for a line that does not read.
	#[must_use]
	pub fn parsed(&self) -> Option<Row> {
		self.row.parse().ok()
	}

	/// Whose row it is.
	#[must_use]
	pub fn who(&self, row: &Row) -> Who {
		who(&self.hid, row, self.guid.as_deref())
	}

	/// What the row is known by, whichever install id brought it: whose it is, its text with no character and its copy
	/// number.
	#[must_use]
	pub fn key(&self) -> Option<(Who, String, u16)> {
		let row = self.parsed()?;
		Some((self.who(&row), Row { char: 0, ..row }.to_string(), self.n))
	}
}

fn who(hid: &str, row: &Row, guid: Option<&str>) -> Who {
	match (row.char, guid) {
		(0, _) => Who::Nobody,
		(_, Some(guid)) => Who::Guid(guid.to_owned()),
		(index, None) => Who::Index(hid.to_owned(), index),
	}
}

/// Whether another computer's game row joins this computer's game: none of its pauses, owed, banked or paid rows, and no
/// body note (its reset joins as this computer's own: `Session::take_far`).
#[must_use]
pub fn carried(row: &Row) -> bool {
	row.kind != PAUSE && !matches!(row.outcome, outcome::OWED | outcome::BANKED | outcome::PAID) && !is_body_note(row)
}

/// Another computer's row as this computer's game takes it: the app's bit and the computer's in place of every device
/// bit (an app row never pays or owes: DB.lua `applyRow`), its character `char` in the numbering it is written in.
#[must_use]
pub fn as_app(row: &Row, char: u8) -> Row {
	Row { char, flags: (row.flags & !DEVICE_BITS) | flags::APP | flags::COMPUTER, ..*row }
}

/// A row with its device bits aside, by which another computer's row is found to be one of this computer's own (a
/// computer whose data folder came back from its own batches under a new install id).
#[must_use]
pub fn bare(row: &Row) -> Row {
	Row { flags: row.flags & !DEVICE_BITS, ..*row }
}

/// The other computers' rows in the order this computer took them.
#[derive(Default)]
pub struct FarRows {
	records: Vec<FarRecord>,
	kept: HashSet<(Who, String, u16)>,
	/// The times of the resets that came in (the latest kept: `floor`).
	resets: BTreeSet<u32>,
	file: JsonLines,
	/// The file lacks a row the store keeps (an append that failed) or holds lines that did not read: the next take
	/// rewrites it whole.
	behind: bool,
}

impl FarRows {
	/// Loads `far.jsonl` from the data folder (an absent file holds nothing); a line that does not read is skipped and
	/// counted, and the next take rewrites the file without it.
	///
	/// # Errors
	/// A file that exists but cannot be read.
	pub fn open(dir: &Path) -> std::io::Result<(Self, usize)> {
		let (file, records, damaged) = JsonLines::open::<FarRecord>(dir, FILE)?;
		let mut store = Self { file, behind: damaged > 0, ..Self::default() };
		for r in records {
			store.keep(r);
		}
		Ok((store, damaged))
	}

	/// A store that keeps its rows in memory only.
	#[must_use]
	pub fn in_memory() -> Self {
		Self::default()
	}

	#[must_use]
	pub fn records(&self) -> &[FarRecord] {
		&self.records
	}

	/// Whether the reset at `ts` came in from another computer: this computer keeps it as its own and never sends it.
	#[must_use]
	pub fn came_in(&self, ts: u32) -> bool {
		self.resets.contains(&ts)
	}

	/// A record kept unless one with its key is; a line that does not read is none.
	fn keep(&mut self, r: FarRecord) {
		let Some(row) = r.parsed() else { return };
		let Some(key) = r.key() else { return };
		if !self.kept.insert(key) {
			return;
		}
		if row.kind == kind::RESET {
			self.resets.insert(row.ts);
		}
		self.records.push(r);
	}

	/// A batch's game rows from the computer `hid`, with its characters' GUIDs by index (`chars`, the batch's): each row
	/// it carries (`carried`, its resets among them, which the session took as this computer's) dated at or after
	/// `floor` (the store's latest reset), counted by copies among the batch's rows of the same text and character (a
	/// batch holds every copy of a text it sends), and kept when this store lacks that copy. The number of rows kept.
	///
	/// # Errors
	/// The file cannot be written: the rows stay in memory and reach the mirror, the take-in counts as not saved (the
	/// cursor stays before the batch), and the next take rewrites the file whole.
	pub fn take(
		&mut self,
		hid: &str,
		rows: &[GameRow],
		chars: &BTreeMap<u8, String>,
		floor: Option<u32>,
	) -> (usize, std::io::Result<()>) {
		let mut copies: HashMap<(Who, String), u16> = HashMap::new();
		let before = self.records.len();
		for row in rows.iter().map(GameRow::row).filter(|r| carried(r) && floor.is_none_or(|f| r.ts >= f)) {
			let guid = chars.get(&row.char).filter(|_| row.char > 0).cloned();
			let n = copies.entry((who(hid, &row, guid.as_deref()), Row { char: 0, ..row }.to_string())).or_default();
			*n = n.saturating_add(1);
			self.keep(FarRecord { hid: hid.to_owned(), row: row.to_string(), n: *n, guid });
		}
		let kept = self.records.len() - before;
		let written = if self.behind {
			self.file.rewrite(&self.records)
		} else {
			self.records[before..].iter().try_for_each(|r| self.file.append(r))
		};
		self.behind = written.is_err();
		(kept, written)
	}

	/// A reset at `ts`, this computer's or another's: the rows dated before it go, in memory and from `far.jsonl`
	/// (rewritten whole; one that fails is made again at the next take).
	///
	/// # Errors
	/// The file cannot be rewritten; the store in memory has them gone all the same.
	pub fn floor(&mut self, ts: u32) -> std::io::Result<()> {
		let before = self.records.len();
		let records = std::mem::take(&mut self.records);
		(self.kept, self.resets) = (HashSet::new(), BTreeSet::new());
		for r in records.into_iter().filter(|r| r.parsed().is_some_and(|row| row.ts >= ts)) {
			self.keep(r);
		}
		if self.records.len() == before && !self.behind {
			return Ok(());
		}
		let written = self.file.rewrite(&self.records);
		self.behind = written.is_err();
		written
	}
}
