//! The backup file (docs/plan.md slices 17 and 35; `contract/fixtures/backup.json`), the computer's side of the phones'
//! `Backup.File`: the book of the device that made it, which the iPhone, Android and the desktop app all read. The
//! desktop app writes its own book as a file of device `computer`: no pumps (it keeps its pumps as rows, never as the
//! phones' pump records), every app row it keeps as a phone keeps the helper's copy of them (`unstamp`: the app's and
//! the phone's bits cleared, a watch's bit 4 and the computer's bit 12 kept), the game's rows as `GET /v1/history`
//! serves them without their proofs, the game's sheets as `GET /v1/sheets` does, and this computer's engine profile.
//! The body notes (pain flags, the check-in's answers, every row stopped or banked for pain or for feeling unwell) stay
//! out of the file, as the phones keep them out of theirs. A file comes back in as a union of its rows: each stamped
//! as a phone stamps its own on the way to the helper (`stamp`), then counted by copies, as `POST /v1/sync` carries
//! them; the game's rows and sheets come back from the game's own save, never from a file.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::codes::{flags, kind};
use crate::link::{AppRow, Device};
use crate::protocol::{RowRecord, SheetRecord};
use crate::row::Row;

/// The file's version this build writes, and the newest it reads (the phones' `Backup.version`).
pub const VERSION: u32 = 1;
/// A file past this is no backup of QuestPump's and is never read in (the phones' `Book.maxBytes`).
pub const MAX_BYTES: u64 = 64 << 20;
/// The device a file the desktop app made names (`Backup.File.device`, slice 35).
pub const COMPUTER: &str = "computer";

/// Row kinds the body notes are (`ns.Enum.kind`): a pain flag and the check-in's answer.
const PAIN: u8 = 9;
const CHECKIN: u8 = 12;
/// The reason "pain" in flag bits 0-2 (`ns.Enum.stopped.pain`, `ns.Enum.bank.pain`); "felt unwell" is
/// `flags::UNWELL`.
const PAIN_REASON: u16 = 3;

/// A backup file, its keys written in order so the same book is the same bytes.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct File {
	/// `ios`, `android`, or `computer`.
	pub device: String,
	pub game: Game,
	/// Unix seconds.
	pub made_at: i64,
	/// The device's own engine profile as it wrote it (the phones' `EngineProfile`), kept whole.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub profile: Option<Value>,
	/// The phones' pump records, kept as written: the desktop app writes none and takes none in.
	pub pumps: Vec<Value>,
	pub questpump_backup: u32,
	/// The app rows in the addon's grammar, as the device keeps them.
	pub rows: Vec<String>,
}

/// The game's history a file carries.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct Game {
	pub rows: Vec<GameRow>,
	pub sheets: Vec<KeptSheet>,
}

/// A game row as a phone keeps it (`HistoryRow`), without its heart-rate proof, which the book never carries.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameRow {
	pub char: u8,
	pub flags: u16,
	pub kind: u8,
	pub ladder: u8,
	#[serde(rename = "move")]
	pub move_id: u16,
	pub outcome: u8,
	pub quest_id: u32,
	pub reps: u16,
	pub row_ts: u32,
	pub rung: u8,
	pub seconds: u16,
	pub slot: u8,
	pub tier: u8,
}

impl GameRow {
	/// A stored game row as a book carries it.
	#[must_use]
	pub fn of(r: &RowRecord) -> Self {
		Self {
			char: r.char,
			flags: r.flags,
			kind: r.kind,
			ladder: r.ladder,
			move_id: r.move_id,
			outcome: r.outcome,
			quest_id: r.quest_id,
			reps: r.reps,
			row_ts: r.row_ts,
			rung: r.rung,
			seconds: r.seconds,
			slot: r.slot,
			tier: r.tier,
		}
	}

	/// The row in the addon's grammar.
	#[must_use]
	pub fn row(&self) -> Row {
		Row {
			ts: self.row_ts,
			char: self.char,
			quest_id: self.quest_id,
			tier: self.tier,
			slot: self.slot,
			ladder: self.ladder,
			rung: self.rung,
			move_id: self.move_id,
			kind: self.kind,
			reps: self.reps,
			seconds: self.seconds,
			outcome: self.outcome,
			flags: self.flags,
		}
	}
}

/// A character's sheet as a phone keeps it (`KeptSheet`): its character and the sheet's JSON object.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct KeptSheet {
	pub char: u8,
	pub sheet: Value,
}

/// Why a file is refused whole, never read in part (the phones' `Backup.Refusal`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Refusal {
	/// A newer QuestPump made it.
	#[error("a newer QuestPump made it")]
	Newer,
	/// Not a QuestPump backup.
	#[error("not a QuestPump backup")]
	NotBackup,
}

/// A body note: a pain flag or the check-in's answer, or a move, level-up or pump row stopped or banked for pain or
/// for feeling unwell (the phones' `Backup.isBodyNote`).
#[must_use]
pub fn is_body_note(row: &Row) -> bool {
	if row.kind == PAIN || row.kind == CHECKIN {
		return true;
	}
	let reason = row.reason();
	(kind::REP..=kind::PUMP).contains(&row.kind) && (reason == PAIN_REASON || reason == flags::UNWELL)
}

/// An app row as a phone keeps the helper's copy of it (the phones' `AppRows.unstamp`): the app's and the phone's bits
/// cleared, a watch's bit 4 and the computer's bit 12 kept; a phone's Start over as its reset row (the app's floor,
/// since 2026-10-09).
#[must_use]
pub fn unstamp(row: &Row) -> Row {
	Row { flags: row.flags & !(flags::APP | flags::PHONE), ..*row }
}

/// A row of a file as a phone sends it to the helper (the phones' `AppRows.stamp`): character 0 and the app bit, with
/// the phone's bit unless the row carries the watch's or the computer's; a reset row too since 2026-10-09 (a phone's
/// Start over, the app's floor, which the store of app rows keeps and the mirror never carries: the game keeps what it
/// has).
#[must_use]
pub fn stamp(row: &Row) -> Row {
	let device = row.flags & (flags::WATCH | flags::COMPUTER) != 0;
	let bits = if device { flags::APP } else { flags::APP | flags::PHONE };
	Row { char: 0, flags: row.flags | bits, ..*row }
}

/// The desktop app's book as a file made at `made_at` (unix seconds): every app row it keeps, unstamped and in time
/// order (a stable sort, as a phone holds the helper's copy), and the game's rows (their characters as `GET
/// /v1/history` serves them) and sheets, the body notes left out of both; the profile as given.
#[must_use]
pub fn book(made_at: i64, app: &[RowRecord], game: &[RowRecord], sheets: &[SheetRecord], profile: Value) -> File {
	let mut rows: Vec<Row> = app.iter().map(RowRecord::row).filter(|r| !is_body_note(r)).map(|r| unstamp(&r)).collect();
	rows.sort_by_key(|r| r.ts);
	File {
		device: COMPUTER.to_owned(),
		game: Game {
			rows: game.iter().filter(|r| !is_body_note(&r.row())).map(GameRow::of).collect(),
			sheets: sheets
				.iter()
				.filter_map(|s| serde_json::from_str(s.sheet.get()).ok().map(|sheet| KeptSheet { char: s.char, sheet }))
				.collect(),
		},
		made_at,
		profile: Some(profile),
		pumps: Vec::new(),
		questpump_backup: VERSION,
		rows: rows.iter().map(ToString::to_string).collect(),
	}
}

/// A file's bytes: its keys sorted at every depth (the phones' encoder sorts them too), so the same book is the same
/// file.
///
/// # Errors
/// A value that does not write as JSON (none of a file's types fail).
pub fn encode(file: &File) -> serde_json::Result<Vec<u8>> {
	serde_json::to_vec(&serde_json::to_value(file)?)
}

/// A file's bytes as a backup, or why it is refused whole: a newer version, or no backup at all (the phones'
/// `Backup.decode`).
///
/// # Errors
/// `Refusal::Newer` for a version past `VERSION`, `Refusal::NotBackup` for anything that does not read as a backup.
pub fn decode(bytes: &[u8]) -> Result<File, Refusal> {
	#[derive(Deserialize)]
	struct Head {
		questpump_backup: Option<u32>,
	}
	let head: Head = serde_json::from_slice(bytes).map_err(|_| Refusal::NotBackup)?;
	match head.questpump_backup {
		Some(v) if v > VERSION => Err(Refusal::Newer),
		Some(v) if v >= 1 => serde_json::from_slice(bytes).map_err(|_| Refusal::NotBackup),
		_ => Err(Refusal::NotBackup),
	}
}

impl File {
	/// The device that sent the file's own rows, as the helper's store of app rows records a sender: a phone's file its
	/// phone, the computer's none (the computer is no client of the link).
	#[must_use]
	pub fn sender(&self) -> Option<Device> {
		match self.device.as_str() {
			"ios" => Some(Device::Ios),
			"android" => Some(Device::Android),
			_ => None,
		}
	}

	/// The file's rows as a phone would send them (`app_rows`).
	#[must_use]
	pub fn app_rows(&self) -> Vec<AppRow> {
		app_rows(&self.rows)
	}
}

/// A book's rows (a file's, a sync batch's) as a phone would send them: each that reads as the grammar writes it,
/// stamped (`stamp`), with its copy number among the book's rows of the same stamped text.
#[must_use]
pub fn app_rows(rows: &[String]) -> Vec<AppRow> {
	let mut copies: HashMap<String, u16> = HashMap::new();
	rows.iter()
		.filter_map(|text| text.parse::<Row>().ok().filter(|r| r.to_string() == *text))
		.map(|r| stamp(&r))
		.map(|r| {
			let row = r.to_string();
			let n = copies.entry(row.clone()).or_default();
			*n = n.saturating_add(1);
			AppRow { row, n: *n }
		})
		.collect()
}
