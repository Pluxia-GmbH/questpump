//! The JSON lines the helper writes on its local socket (rust-helper `protocol-and-mirror.md`, "Event lines"), the
//! commands it takes there, and the device commands the strip's events become (queued for the phone or the watch by
//! `link::Link`, and written as `command` lines too). One object per line, `ts` first (the helper's clock, RFC 3339
//! with milliseconds), then `type`, then the fields in declaration order, so golden files stay diffable.

use serde::{Deserialize, Serialize};

use crate::codes::{PumpEnd, StripEvent, StripState, flags};
use crate::link::{Device, Queued, Summary, WristEvent};
use crate::proof::HrSource;
use crate::verdict::Verdict;

/// Where a row came from: the game, a catch-up the helper wrote for the watch, the phone or the computer (it pays owed
/// moves; the computer's since docs/plan.md slice 35, run in the desktop app), or an app (slice 26: an app pump, a pain
/// flag or a check-in a phone, a watch or the desktop app wrote, kept in `app.jsonl`).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Source {
	Game,
	Watch,
	Phone,
	App,
	Computer,
}

impl Source {
	/// The flag bit a row from this source carries (none for the game's own rows; an app row carries its device's bit
	/// beside it, which the phone stamps, or `Session::keep_computer` on the desktop app's).
	#[must_use]
	pub fn bit(self) -> u16 {
		match self {
			Self::Game => 0,
			Self::Watch => flags::WATCH,
			Self::Phone => flags::PHONE,
			Self::App => flags::APP,
			Self::Computer => flags::COMPUTER,
		}
	}

	/// The source a row's flag bits name: the app bit first, since an app row carries its device's bit too.
	#[must_use]
	pub fn from_flags(row_flags: u16) -> Self {
		if row_flags & flags::APP != 0 {
			Self::App
		} else if row_flags & flags::PHONE != 0 {
			Self::Phone
		} else if row_flags & flags::WATCH != 0 {
			Self::Watch
		} else if row_flags & flags::COMPUTER != 0 {
			Self::Computer
		} else {
			Self::Game
		}
	}
}

/// One row as the helper keeps it (`rows.jsonl`, and an app's in `app.jsonl`) and writes it on the socket: the row
/// grammar's fields, the row's own time as `row_ts`, and where the helper got it. `char` is the character's place in
/// the helper's own list, across every game install (slice 26; `session::People`), 0 on an app row.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct RowRecord {
	pub ts_helper: String,
	pub row_ts: u32,
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
	pub source: Source,
	/// The strip event that carried it; none for a row the helper wrote itself (a logout's owed moves, a catch-up).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub load_id: Option<u16>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub event_no: Option<u16>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub ts_client_ms: Option<u64>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub yards: Option<Yards>,
	/// The metres the walking pad counted over the stretch a stretch row closes (docs/plan.md slice 25): the helper's own
	/// reading of the pad, which the addon never sees; none when it counted none (no pad, a pad standing still) and on
	/// every other row.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub pad_metres: Option<u32>,
	/// The pump a row belongs to: a catch-up's (`w-<uuid>`, `p-<uuid>`, the desktop app's `d-<uuid>`), so a resent pump
	/// pays nothing twice, and a pump the desktop app ran (`d-<uuid>`, on its app rows: `Session::keep_computer`). None
	/// on every other row.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub pump: Option<String>,
	/// What a pump the desktop app ran was, on its offered pump row (`Session::keep_computer`), where its kind names it
	/// in the desktop's Saga ([`PumpKind`]); none on every other row (a phone keeps its own pump's kind in its own
	/// history).
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub pump_kind: Option<PumpKind>,
	/// An app row's copy number: the n-th copy of its text among its device's rows (`link::AppRow::n`), which with the
	/// text is the row's identity, so a row sent again is kept once and two rows alike stay two. None on every other
	/// row.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub copy: Option<u16>,
	/// The device that sent an app row; none on one taken in from a file, on the desktop app's own (the computer is no
	/// device of the link: `link::Device`), and on every other row.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub device: Option<Device>,
	/// A pump row's heart-rate proof, in `GET /v1/history` (the store and the `row` line never carry it: the verdict
	/// comes a minute after the go, as its own `verdict` line, and a later request's `proofs` bring it to a row already
	/// read).
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub hr_verdict: Option<Verdict>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub hr_peak: Option<u16>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub hr_delta: Option<i32>,
}

/// One sheet as the helper keeps it (`sheets.jsonl`) and serves it (`GET /v1/sheets`): when the helper read it, the
/// character (the helper's own numbering across every install, as a history row's `char`; the sheet's own `char` is
/// its install's index), and the `sheet` strip event as the addon folded it, as JSON (its own `at` is when the addon
/// folded it, the time a chart plots). The sheet stays the JSON text it was written as: the helper holds every logged
/// sheet in memory for the pages, and a parsed tree takes several times the text's size.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SheetRecord {
	pub ts_helper: String,
	pub char: u8,
	pub sheet: Box<serde_json::value::RawValue>,
}

impl SheetRecord {
	/// # Errors
	/// A sheet that does not write as JSON text (a non-finite number).
	pub fn new(ts_helper: String, char: u8, sheet: &serde_json::Value) -> serde_json::Result<Self> {
		Ok(Self { ts_helper, char, sheet: serde_json::value::to_raw_value(sheet)? })
	}
}

impl PartialEq for SheetRecord {
	fn eq(&self, other: &Self) -> bool {
		self.ts_helper == other.ts_helper && self.char == other.char && self.sheet.get() == other.sheet.get()
	}
}

/// The stretch's yards so far when the row was painted.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Yards {
	pub foot: u32,
	pub mount: u32,
	pub swim: u32,
	pub taxi: u32,
}

impl RowRecord {
	#[must_use]
	pub fn row(&self) -> crate::row::Row {
		crate::row::Row {
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

	#[must_use]
	pub fn new(row: &crate::row::Row, ts_helper: &str, source: Source) -> Self {
		Self {
			ts_helper: ts_helper.to_owned(),
			row_ts: row.ts,
			char: row.char,
			quest_id: row.quest_id,
			tier: row.tier,
			slot: row.slot,
			ladder: row.ladder,
			rung: row.rung,
			move_id: row.move_id,
			kind: row.kind,
			reps: row.reps,
			seconds: row.seconds,
			outcome: row.outcome,
			flags: row.flags,
			source,
			load_id: None,
			event_no: None,
			ts_client_ms: None,
			yards: None,
			pad_metres: None,
			pump: None,
			pump_kind: None,
			copy: None,
			device: None,
			hr_verdict: None,
			hr_peak: None,
			hr_delta: None,
		}
	}
}

/// What a pump the desktop app ran was, where its kind changes its title: a move the player started from a step's card
/// or a move's window (`move`, docs/plan.md slice 38), which the desktop's Saga titles "A pump you asked for" as both
/// phones title theirs (`pump.move`), and since slice 51 an expedition's round (`expedition`: its stage, and a round's
/// number from 1), which the Saga titles "Expedition · warm-up", "Expedition · round 2" or "Expedition · cool-down"
/// (`expedition.saga.*`), as the phones title theirs.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PumpKind {
	Move,
	Expedition {
		stage: ExpeditionStage,
		#[serde(default, skip_serializing_if = "Option::is_none")]
		round: Option<u8>,
	},
}

/// An expedition round's stage as the Saga titles it (slice 51): the warm-up, a round (the engine's inner rounds and
/// its last), the cool-down.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExpeditionStage {
	Warmup,
	Round,
	Cooldown,
}

/// One decoded strip frame, as the `strip` line carries it.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct StripLine {
	pub seq: u8,
	pub load_id: u16,
	pub event_no: u16,
	pub event: StripEvent,
	pub state: StripState,
	pub quest_id: u32,
	pub level: u8,
	pub kind: u8,
	pub tier: u8,
	pub slot: u8,
	pub ladder: u8,
	pub rung: u8,
	pub row_flags: u16,
	pub move_id: u16,
	pub char_index: u8,
	pub outcome: u8,
	/// The row's own time on a row frame, the pump's offered time on a pump's frames; none when zero.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub row_ts: Option<u32>,
	/// The game clock in milliseconds, unwrapped from its 24-bit cell.
	pub ts_client_ms: u64,
	/// The countdown's start in the same units; none when no move counts down or runs.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub start_at_ms: Option<u64>,
	pub yards_foot: u32,
	pub yards_mount: u32,
	pub yards_swim: u32,
	pub yards_taxi: u32,
	pub reps: u16,
	pub seconds: u16,
	pub heartbeat: bool,
	pub reemit: bool,
	/// When the window server showed the capture, on the helper's wall clock, in seconds.
	pub display_time: f64,
}

/// A move of an offered pump, as the devices get it.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct OfferedMove {
	pub slot: u8,
	#[serde(rename = "move")]
	pub move_id: u16,
	/// Reps or seconds to reach; 0 for a max set.
	pub target: u16,
}

/// What the game did, for the device that holds the link (`contract/companion-api.yaml`, `Command`; the countdown's
/// times are on the helper's wall clock in milliseconds, converted from the game clock through the paint's display
/// time).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Command {
	Countdown {
		pump_id: String,
		slot: u8,
		start_at: i64,
		go_at: i64,
		ticks: u8,
	},
	/// Keep going pressed in the game on the move a device follows (slice 52's double-check, 2026-10-24): its clock
	/// runs on past the plan to the full move's `seconds`, counting up, and the game ends the move there by itself.
	Keep {
		pump_id: String,
		slot: u8,
		seconds: u16,
	},
	TurnIn {
		quest_id: u32,
		tier: u8,
	},
	QuestComplete {
		quest_id: u32,
	},
	PumpOffered {
		pump_id: String,
		/// The quest of the pump's first move, which the watch's summary names. The helper names it in every pump it
		/// offers; the contract's `Command` makes it optional, and a body without it reads (slice 43's `wire.json`).
		#[serde(default, skip_serializing_if = "Option::is_none")]
		quest_id: Option<u32>,
		moves: Vec<OfferedMove>,
		/// The character's streak in days, as its latest sheet says; none before a sheet. The contract and `wire.json`
		/// still carry it, though no device reads it since the coach's countdown line went (2026-10-09).
		#[serde(default, skip_serializing_if = "Option::is_none")]
		streak: Option<u16>,
	},
	PumpDone {
		pump_id: String,
		/// How the game ended it, from the frame: a device shows the result only for `done`, which an absent `ended`
		/// reads as (contract/companion-api.yaml).
		#[serde(default)]
		ended: PumpEnd,
	},
	/// The game's move ended: done (its count being typed), cancelled back to Ready, or cut by a `/reload` or a logout.
	/// A device drops a cancelled move, counting down, running, or ended already when the game's Back left its result
	/// view before its row (2026-10-05); otherwise it cancels its countdown or finishes its running move.
	MoveEnd {
		pump_id: String,
		slot: u8,
		/// The game went back to Ready (Escape, a popup, the game menu, combat, its Back, from the result view too):
		/// the move never counts.
		#[serde(default, skip_serializing_if = "std::ops::Not::not")]
		cancelled: bool,
	},
	/// The count the player typed in the game for one move of the game's pump (slice 51), sent the moment the move's own
	/// result row crosses the strip, after its `move_end`: the row's reps (0 on a hold), seconds and judged outcome (beat
	/// 1, miss 2, placement 3, censored 4, done 5, last beat 11), which a device shows on the move with the record mark
	/// when it beats the record the device knows.
	MoveCounted {
		pump_id: String,
		slot: u8,
		reps: u16,
		seconds: u16,
		outcome: u8,
	},
	/// The game quit.
	SessionEnd,
	/// A move's heart-rate verdict (a catch-up pump's, without a slot), a minute after its go or once a proven move
	/// ended, and for a game move only once the game kept it (its row, or the game going on): the wrist or the phone
	/// shows it on the move's card.
	Verdict {
		pump_id: String,
		#[serde(default, skip_serializing_if = "Option::is_none")]
		slot: Option<u8>,
		verdict: Verdict,
		#[serde(default, skip_serializing_if = "Option::is_none")]
		peak: Option<u16>,
		#[serde(default, skip_serializing_if = "Option::is_none")]
		delta: Option<i32>,
	},
	/// A match a game the player turned on in QuestPump on the computer ended, or a program the player picked closed
	/// (slice 35): the offer of a pump after it, for the device that holds the link, which raises it as its own notice
	/// and takes it back when a response lists the offer in `offer_withdrawn`. Nothing of the match but our own
	/// clock's: since slice 57 its minutes size it, and the computer's sitting clock says whether it opens warm.
	MatchEnd {
		/// The offer's id, `m-` and a version-4 UUID.
		offer: String,
		/// The game's key in `contract/links.json`'s `computer` list (`program` for a picked one).
		link: String,
		/// The game's own name as the computer shows it (a picked program's own), never translated: at most 40
		/// characters, Unicode scalars (`offers::NAME_CHARS`; `offers::offer_name` cuts a longer one).
		name: String,
		/// The kind of `contract/kinds.json` its pump opens on: `match` for every game's offer (none for a program's
		/// close until 2026-10-28: the phones still take one without it from an older desktop app).
		#[serde(default, skip_serializing_if = "Option::is_none")]
		kind: Option<String>,
		/// When it ended, on the helper's wall clock.
		ended_at: i64,
		/// The pump's size (slice 48: 1 Light, 2 Balanced, 3 Extreme; `offers::Offer::tier`), since slice 57 by the
		/// match's own minutes (the computer's sitting clock until then); none from an older desktop app (Balanced then).
		#[serde(default, skip_serializing_if = "Option::is_none")]
		tier: Option<u8>,
		/// The match's whole minutes, from its start to its end by the computer's own clock (slice 57,
		/// `offers::Offer::minutes`; League's from its champion select since slice 59, `match_lead_league_min`), which
		/// the pump's cardio takes; none from an older desktop app, whose offer a device reads as before (its minutes
		/// from `tier`, its warm move from them).
		#[serde(default, skip_serializing_if = "Option::is_none")]
		minutes: Option<u16>,
		/// The computer's sitting clock held `match_sat_min` as the offer was made (slice 57, `offers::Offer::warm`):
		/// the pump opens on its warm move; present only when true, and read only beside `minutes`.
		#[serde(default, skip_serializing_if = "std::ops::Not::not")]
		warm: bool,
		/// The arms rest (slice 48): the game is marked `aim` in contract/links.json, its switch is on and it still ran
		/// at the match's end; present only when true.
		#[serde(default, skip_serializing_if = "std::ops::Not::not")]
		arms: bool,
	},
}

impl Command {
	/// The pump a command tells of: a game pump's `g-` id, or a catch-up's own on its verdict; none for a turn-in, a
	/// quest, the game's quit or a match's offer.
	#[must_use]
	pub fn pump_id(&self) -> Option<&str> {
		match self {
			Self::Countdown { pump_id, .. }
			| Self::Keep { pump_id, .. }
			| Self::PumpOffered { pump_id, .. }
			| Self::PumpDone { pump_id, .. }
			| Self::MoveEnd { pump_id, .. }
			| Self::MoveCounted { pump_id, .. }
			| Self::Verdict { pump_id, .. } => Some(pump_id),
			Self::TurnIn { .. } | Self::QuestComplete { .. } | Self::SessionEnd | Self::MatchEnd { .. } => None,
		}
	}
}

/// Why a task stopped.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
	ScreenPermission,
	StreamStopped,
	NoDisplay,
	MirrorWrite,
	NoGameFolder,
	/// The LAN API could not listen (the port taken, the address gone): the phone and the watch cannot reach it.
	Lan,
	/// Bluetooth was refused (the grant, or a policy): the strap and the walking pad cannot be read until it is allowed.
	BluetoothPermission,
}

/// What a `pad_event` line says happened: the walking pad's own Fitness Machine Status (FTMS 0x2ADA), or a `pad.speed`
/// or `pad.stop` that did not go through.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PadEventKind {
	/// The belt started or resumed.
	Started,
	/// The belt stopped.
	Stopped,
	/// The belt paused.
	Paused,
	/// The belt stopped because its safety key came off.
	SafetyKey,
	/// The pad reset itself.
	Reset,
	/// The pad took a new target speed, the line's `kmh`.
	Speed,
	/// Another app took the pad's control: the next command asks for it again.
	ControlLost,
	/// A command the pad or the helper refused: the line's `detail` names the command and why.
	Refused,
}

/// One line on the socket.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Line {
	Hello {
		pid: u32,
		version: String,
		display: String,
		scale: f64,
	},
	Strip(StripLine),
	Row(RowRecord),
	/// A command as the link queued it.
	Command {
		command: Queued,
	},
	/// A pump on the phone or the watch, as its sync carried it.
	Wrist {
		device: Device,
		event: WristEvent,
	},
	/// A device's delivery statistics when its session ended (the latency harness).
	Link(Summary),
	/// A strap's heart rate, as each notification carries it (about once a second).
	Hr {
		bpm: u16,
		/// The beat-to-beat intervals, in seconds.
		#[serde(skip_serializing_if = "Vec::is_empty")]
		rr: Vec<f64>,
		#[serde(skip_serializing_if = "Option::is_none")]
		contact: Option<bool>,
	},
	/// A walking pad's Treadmill Data (FTMS 0x2ACD), one line per record (about once a second): the belt's speed, the
	/// distance and the time the pad counts from its own session's start, each when the record carries it.
	Pad {
		#[serde(skip_serializing_if = "Option::is_none")]
		kmh: Option<f64>,
		#[serde(skip_serializing_if = "Option::is_none")]
		distance_m: Option<u32>,
		#[serde(skip_serializing_if = "Option::is_none")]
		elapsed_s: Option<u16>,
	},
	/// What the walking pad did or refused (`PadEventKind`).
	PadEvent {
		event: PadEventKind,
		/// The new target speed, on `speed`.
		#[serde(skip_serializing_if = "Option::is_none")]
		kmh: Option<f64>,
		/// The command refused and why, on `refused`: `speed: not_permitted`, `stop: no control point`.
		#[serde(skip_serializing_if = "Option::is_none")]
		detail: Option<String>,
	},
	/// A move's heart-rate verdict once it is out (docs/plan.md slice 8): the pump's row time and slot, what the
	/// rule found, the source it read and the rise that source needed.
	Verdict {
		pump_id: String,
		row_ts: u32,
		#[serde(skip_serializing_if = "Option::is_none")]
		slot: Option<u8>,
		verdict: Verdict,
		#[serde(skip_serializing_if = "Option::is_none")]
		peak: Option<u16>,
		#[serde(skip_serializing_if = "Option::is_none")]
		delta: Option<i32>,
		#[serde(skip_serializing_if = "Option::is_none")]
		source: Option<HrSource>,
		#[serde(skip_serializing_if = "Option::is_none")]
		rise: Option<u16>,
	},
	/// A state changed: the capture's (`screen`), the strap's (`hr`) or the walking pad's (`pad`, in the strap's words),
	/// with a hint for the player.
	Status {
		#[serde(skip_serializing_if = "Option::is_none")]
		screen: Option<String>,
		#[serde(skip_serializing_if = "Option::is_none")]
		hint: Option<String>,
		#[serde(skip_serializing_if = "Option::is_none")]
		hr: Option<String>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pad: Option<String>,
	},
	Error {
		code: ErrorCode,
		detail: String,
	},
}

#[derive(Serialize)]
struct Envelope<'a> {
	ts: &'a str,
	#[serde(flatten)]
	line: &'a Line,
}

impl Line {
	/// The line's JSON text with the helper's time first, without the newline.
	///
	/// # Errors
	/// When serde cannot write the line (a float that is not finite).
	pub fn to_json(&self, ts: &str) -> serde_json::Result<String> {
		serde_json::to_string(&Envelope { ts, line: self })
	}
}

/// A command a client sends on the socket; an unknown one is ignored with a warning.
#[derive(Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(tag = "cmd")]
pub enum SocketCommand {
	/// A normal quit, exit 0, once nothing a quit would cut holds it (a pump on the computer or in the game, a match's
	/// offer or its card: the helper's `quit_held`), so a reinstall's quit never ends a pump (2026-10-31).
	#[serde(rename = "quit")]
	Quit,
	/// The same quit, then QuestPump started again from its own bundle as an update's relaunch, with no window
	/// (`--updated`): a reinstall swaps the bundle first, then asks this (rust-helper step 3). The macOS helper alone.
	#[serde(rename = "restart")]
	Restart,
	/// The correction the latency harness's camera test learns once (slice 7): how much later the wrist's "go" came
	/// than the screen's, negative when it came earlier (an external monitor's own delay): `{"cmd":"display_lag",
	/// "ms":25}`. Kept in the settings; every countdown's start is dated that much earlier.
	#[serde(rename = "display_lag")]
	DisplayLag { ms: i32 },
	/// Set the walking pad's belt to a speed in km/h, `{"cmd":"pad.speed","kmh":3.5}`: fitted into the pad's own range
	/// and started when the belt is stopped. A pad that does not take it answers with a `pad_event` `refused`.
	#[serde(rename = "pad.speed")]
	PadSpeed { kmh: f64 },
	/// Stop the walking pad's belt, `{"cmd":"pad.stop"}`.
	#[serde(rename = "pad.stop")]
	PadStop,
}
