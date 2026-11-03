//! The link to the phone and the watch (docs/plan.md slice 7; `contract/companion-api.yaml`, the source of truth for
//! the routes and bodies): the plaintext bodies, the command queue that answers the held `POST /v1/sync`, which device
//! holds the link, the acknowledgements, the delivery statistics the latency harness prints per session, and what
//! became of each pump's commands (`Link::unreached`). Since slice 43 a request comes on the LAN or over the Bluetooth
//! link (`Via`): a client whose held request came through a central counts alive past `ALIVE_MS` while that request is
//! pending at the helper (until its answer went to the radio, `Link::answered`) and for `ALIVE_MS` after, its hold may
//! run to `BLE_WAIT_MAX_MS`, and its central's disconnect or unsubscribe hands the link on at once (`Link::gone`).
//! Since the fix review of 2026-10-17 a phone that steps back for its watch keeps a standing request
//! (`Exchange::Standing`), which holds nothing and answers at once while no device holds the link (`Link::unheld`,
//! `Link::lapse_at`).
//! No clock, socket or lock here: the app passes the time in (milliseconds on the helper's wall clock) and serves the
//! routes.

use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use std::net::IpAddr;

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize};

use crate::body::Body;
use crate::codes::kind;
use crate::mirror::Sight;
use crate::protocol::{Command, RowRecord, SheetRecord, Source};
use crate::seal::NONCE;
use crate::session::{BankMove, BankPump, CatchupMove, CatchupPump};
use crate::verdict::{Outcome, Verdict};

/// The most rows one `GET /v1/history` or `GET /v1/app_rows` page carries.
pub const HISTORY_PAGE: usize = 5_000;
/// The most sheets one `GET /v1/sheets` page carries.
pub const SHEETS_PAGE: usize = 1_000;

/// A paged route's size for the request's `limit` (since slice 43: a phone asks 500 rows and 50 sheets over the
/// Bluetooth link, so a page's envelope stays far under 2 MiB, and none on the LAN): `most` without one, else the limit
/// held to 1-`most`.
#[must_use]
pub fn page_size(limit: Option<u32>, most: usize) -> usize {
	limit.map_or(most, |l| usize::try_from(l).unwrap_or(most).clamp(1, most))
}

/// How long the helper holds a sync request that finds no command, unless the request asks for longer
/// (`SyncRequest::wait`, at most `WAIT_MAX_MS`, `Link::hold_ms`).
pub const HOLD_MS: i64 = 5_000;
/// A request's `sent_at` (the device's estimate of the helper's clock) further than this from the helper's clock is
/// refused: a recorded request replayed later never passes, and the replay guard needs to remember nonces only this
/// long each way.
pub const FRESH_MS: i64 = 60_000;
/// How long the replay guard remembers a nonce: a body whose `sent_at` passed the `FRESH_MS` check either way.
pub const REPLAY_MS: i64 = 2 * FRESH_MS;
/// A device holds the link while its last held request came within this. A held request without `wait` is read for
/// 20 s on the device (`LinkClient.timeout(wait:)`, Android's `HelperLink.held`) and sent again when that ends, so with
/// the 5-s hold one exchange lost keeps the link; a request with `wait` is read for its wait and 10 s more, 25 s, so the
/// request lost after a 15-s one lets the link lapse for up to 15 s (`WAIT_MAX_MS`).
pub const ALIVE_MS: i64 = 25_000;
/// The longest a held request waits while no command waits for its device, when it asks for longer than `HOLD_MS`
/// (`SyncRequest::wait`, since 2026-10-13) and its client holds the link (`Link::hold_ms`): a phone that follows the
/// game in the background asks a third as often. The device reads such a request for 10 s past its wait, 25 s, and a
/// device whose held requests follow each other at once sends one every 15 s at most, inside `ALIVE_MS`, so it keeps
/// the link.
pub const WAIT_MAX_MS: i64 = 15_000;
/// The same over the Bluetooth link (slice 43, `ble_wait_max_ms`): a phone that follows the game with its screen locked
/// asks once in ten minutes, each answer waking it. Its client counts alive while such a request is pending at the
/// helper, whatever its age, and for `ALIVE_MS` after its answer went to the radio (`Link::answered`; `Link::gone` ends
/// it at once), so the hold may outlast `ALIVE_MS`.
pub const BLE_WAIT_MAX_MS: i64 = 600_000;
/// The longest a held or standing request waits on `transport` while no command waits for its device
/// (`SyncRequest::wait`).
fn wait_max(transport: Transport) -> i64 {
	match transport {
		Transport::Lan => WAIT_MAX_MS,
		Transport::Ble => BLE_WAIT_MAX_MS,
	}
}

/// A device silent this long ended its session: its delivery statistics are printed, and its next request starts
/// another (`Link::starts`).
pub const QUIET_MS: i64 = 60_000;
/// A session that goes on has its statistics printed at least this often (since the review of 2026-10-17): a phone
/// that stays linked by Bluetooth all evening never goes quiet, and the `link` line is what the acceptance reads.
pub const LINE_EVERY_MS: i64 = 3_600_000;
/// A countdown stays deliverable this long after its go: a device told later runs the move from the go.
pub const COUNTDOWN_AFTER_GO_MS: i64 = 30_000;
/// Wrong bearers from one address (since slice 43 also failures of one Bluetooth central) within `THROTTLE_MS` that
/// make the helper refuse it for `THROTTLE_MS` after the last.
pub const THROTTLE_AFTER: usize = 10;
pub const THROTTLE_MS: i64 = 60_000;
const MINUTE_MS: i64 = 60_000;
/// Sent commands whose send time a later receipt can still find (each found once: a receipt sent again after a lost
/// response counts once).
const SENT_KEPT: usize = 256;
/// The devices' latest pump events, each taken once: a device sends an event again until a response confirms it.
const TAKEN_KEPT: usize = 256;
/// The most pumps whose commands' fate the link keeps (`Link::unreached`): a caller that never asks
/// (`sim_helper`) loses the oldest past it.
const REACH_KEPT: usize = 256;
/// The most of a pump's id a log line shows (`pump_tag`): a catch-up's `p-<uuid>` or `w-<uuid>` is 38 characters.
const PUMP_TAG_CHARS: usize = 40;

/// The furthest past the helper's clock an instant on a device's own clock may lie (a body field's `changed_at`, a
/// request's `game_offer_at`; slice 35): a later one comes from a clock set wrong and is dropped, since it would win
/// every later change, or gate every device's game offers, until that time.
pub const AHEAD_MS: i64 = 86_400_000;

/// Whether `at`, unix milliseconds on a device's own clock, lies no more than `AHEAD_MS` past the helper's `now`.
#[must_use]
pub fn not_ahead(at: i64, now: i64) -> bool {
	at <= now.saturating_add(AHEAD_MS)
}

/// Reads a field leniently: one that does not read as `T` (a newer device's shape, a number out of range) is none, so
/// it never fails the body it rides on (a sync's `pro_proof`, each field of a `Body`).
///
/// # Errors
/// Only the input's own: a value that is no JSON at all.
pub fn lenient<'de, D: Deserializer<'de>, T: DeserializeOwned>(d: D) -> Result<Option<T>, D::Error> {
	Ok(serde_json::from_value(serde_json::Value::deserialize(d)?).ok())
}

/// The device a request names.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Device {
	Watchos,
	Wearos,
	Ios,
	Android,
}

impl Device {
	#[must_use]
	pub fn is_watch(self) -> bool {
		matches!(self, Self::Watchos | Self::Wearos)
	}

	/// The name the mirror's `pairing` block and the socket lines use.
	#[must_use]
	pub fn name(self) -> &'static str {
		match self {
			Self::Watchos => "watchos",
			Self::Wearos => "wearos",
			Self::Ios => "ios",
			Self::Android => "android",
		}
	}
}

/// The helper's name and current address, in every response.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Helper {
	pub name: String,
	pub host: String,
	pub port: u16,
	pub version: String,
	pub t: i64,
	/// The computer's own `.local` host name when it has a valid one (`pair::is_local_name`): a device keeps it in its
	/// pair link, so it follows a renamed computer while it reaches it. An older helper sends none.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub mdns: Option<String>,
	/// The helper install's id as the pair link carries it, base64url of 16 bytes. An older helper sends none.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub hid: Option<String>,
	/// What the computer knows of World of Warcraft (since 2026-10-09): the phones' pairing window, chip and card and
	/// the watches' session line say it. An older helper sends none, and one that does not read is none.
	#[serde(default, skip_serializing_if = "Option::is_none", deserialize_with = "lenient")]
	pub wow: Option<Wow>,
	/// What the helper reads of the game, only while `wow` is `addon` (since 2026-10-09): the pairing block's `sight`.
	#[serde(default, skip_serializing_if = "Option::is_none", deserialize_with = "lenient")]
	pub sight: Option<Sight>,
	/// The one step the pairing still asks of the player, only while `wow` is `addon` (since 2026-10-13). An older
	/// helper sends none, and one that does not read is none: a device reads either as `either`.
	#[serde(default, skip_serializing_if = "Option::is_none", deserialize_with = "lenient")]
	pub step: Option<Step>,
	/// How the computer's Bluetooth link stands (since slice 43), in every response. An older helper sends none, and one
	/// that does not read is none: a device then offers no Bluetooth step and gives no Bluetooth reason.
	#[serde(default, skip_serializing_if = "Option::is_none", deserialize_with = "lenient")]
	pub ble: Option<Ble>,
}

/// How the computer's Bluetooth link stands, as every response's helper block says it (slice 43;
/// `contract/companion-api.yaml`, `Helper.ble`): the phones' pairing window, follow card and Helper page, and the
/// computer's Phones page, say it.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Ble {
	/// It serves the link (advertising while a phone is paired or the Phones page shows the code).
	On,
	/// macOS has yet to answer QuestPump's Bluetooth grant, which the Phones page asks in context.
	Ask,
	/// The grant is refused (macOS's Privacy & Security, Bluetooth; Windows' privacy settings).
	NoGrant,
	/// The computer's Bluetooth is turned off.
	Off,
	/// Its adapter cannot serve phones (a Windows PC without the peripheral role): phones link on Wi-Fi alone.
	NoPeripheral,
	/// The computer has no Bluetooth.
	#[serde(rename = "none")]
	Missing,
	/// The player turned the link off on the Phones page, or it could not start (the Phones page says why).
	Stopped,
}

impl Ble {
	/// The word the wire, the log and the counts use.
	#[must_use]
	pub fn word(self) -> &'static str {
		match self {
			Self::On => "on",
			Self::Ask => "ask",
			Self::NoGrant => "no_grant",
			Self::Off => "off",
			Self::NoPeripheral => "no_peripheral",
			Self::Missing => "none",
			Self::Stopped => "stopped",
		}
	}
}

/// How a request reached the helper (slice 43): the LAN, or the Bluetooth link through one central (the Mac's
/// `CBCentral` identifier, Windows' device id).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Via {
	Lan,
	Ble(String),
}

impl Via {
	#[must_use]
	pub fn transport(&self) -> Transport {
		match self {
			Self::Lan => Transport::Lan,
			Self::Ble(_) => Transport::Ble,
		}
	}
}

/// The transport a request came on.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum Transport {
	#[default]
	Lan,
	Ble,
}

impl Transport {
	/// The word the log uses.
	#[must_use]
	pub fn name(self) -> &'static str {
		match self {
			Self::Lan => "lan",
			Self::Ble => "ble",
		}
	}
}

/// The transports a client's session came on (`Summary::transport`; the counts' `link_ble`).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Transports {
	Lan,
	Ble,
	Both,
}

impl Transports {
	/// These and one more.
	#[must_use]
	pub fn with(this: Option<Self>, t: Transport) -> Self {
		match (this, t) {
			(None | Some(Self::Lan), Transport::Lan) => Self::Lan,
			(None | Some(Self::Ble), Transport::Ble) => Self::Ble,
			_ => Self::Both,
		}
	}

	/// The word the log and the counts use.
	#[must_use]
	pub fn name(self) -> &'static str {
		match self {
			Self::Lan => "lan",
			Self::Ble => "ble",
			Self::Both => "both",
		}
	}
}

/// What the computer knows of World of Warcraft, as every response's helper block says it (since 2026-10-09; the
/// desktop's `FoundGame` in the wire's words).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Wow {
	/// An install with QuestPump, the one the helper follows.
	Addon,
	/// The game without QuestPump yet: an install found, or a client running without it.
	NoAddon,
	/// No game known on this computer.
	#[serde(rename = "none")]
	Unfound,
}

/// The one step a pairing still asks of the player, as every response's helper block says it while `wow` is `addon`
/// (since 2026-10-13; the desktop's `Helper::step` reads it from the strip's setup word, `strip::SetupWord`): the
/// phones' pairing window and the desktop's pages name it and nothing else.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Step {
	/// Nothing is left: the game's pumps reach the phone with no reload, and the game reads the pairing at its next
	/// start by itself.
	#[serde(rename = "none")]
	Done,
	/// The running game's setup window asks for its Check again, on screen or held back by a fight, a pump or a
	/// cinematic.
	CheckAgain,
	/// The running game does not read its own save back (the Forever beta) and loaded without the helper: one
	/// `/reload` brings its saved progress back.
	Reload,
	/// macOS's Screen Recording grant is missing (`sight` is `no_grant`).
	Grant,
	/// The helper cannot tell: a running game whose load may have come before the pairing, and whose strip has carried
	/// no setup word since (an addon from before 2026-10-13, or no square read yet).
	Either,
}

/// `GET /v1/hello`'s body, the one answer without a bearer and without a seal: who answers on this address, so a
/// device looking for its helper checks a candidate before its bearer goes out. Since the release security review of
/// 2026-10-31 (L3) it names nothing of the computer, no id, name or port: over the device's challenge `c` and the
/// answer's own random `n`, `h` proves the install's id and `k` the pairing key (`seal::hello_key`), which only a
/// device holding them can check (`contract/fixtures/find.json`). The pairing's install with another key is the
/// device's helper with a new code, and answers to one device or to many link nothing; the names reach a paired device
/// in the sealed answers' `helper` block.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct HelloResponse {
	/// The body's version, [`HELLO_VERSION`].
	pub v: u8,
	pub n: String,
	pub h: String,
	pub k: String,
}

/// The hello body's version: 2 since 2026-11-01 (1 showed the install's id and the key's id in plain).
pub const HELLO_VERSION: u8 = 2;
/// The bytes of a hello's challenge, the device's `c`, and of the answer's own `n`.
pub const HELLO_BYTES: usize = 16;
/// The labels of the hello's proofs: `h` of the install's id, `k` of the pairing key.
const HELLO_HID: &[u8] = b"questpump lan hello hid";
const HELLO_KEY: &[u8] = b"questpump lan hello key";

impl HelloResponse {
	/// The hello for the install's id `hid` (base64url, the helper block's), the current hello key, the device's
	/// challenge `c` and the answer's random `n`: `h` the proof under the install's id, `k` under the hello key, each over
	/// its label, `c` and `n`. None for an id that is not base64url.
	#[must_use]
	pub fn of(hid: &str, hello_key: &[u8; 32], c: &[u8; HELLO_BYTES], n: &[u8; HELLO_BYTES]) -> Option<Self> {
		let hid = URL_SAFE_NO_PAD.decode(hid).ok()?;
		let text = |bytes: &[u8]| URL_SAFE_NO_PAD.encode(bytes);
		Some(Self {
			v: HELLO_VERSION,
			n: text(n),
			h: text(&crate::seal::proof(&hid, &[HELLO_HID, c, n])),
			k: text(&crate::seal::proof(hello_key, &[HELLO_KEY, c, n])),
		})
	}
}

/// `POST /v1/pair`'s request. It and the unpair, body and account requests refuse a field they do not know (since
/// 2026-11-01): a newer app adds nothing to them alone, unlike the sync's, which takes one (`SyncRequest`).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PairRequest {
	pub device: Device,
	pub name: String,
	/// When the phone sent it, on the helper's clock (its ping gave the offset).
	pub sent_at: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PairResponse {
	pub helper: Helper,
}

/// `POST /v1/unpair`'s request (since 2026-10-09): a phone's Unpair, before it forgets the key; `client` names the Pro
/// grant that goes with it.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct UnpairRequest {
	pub device: Device,
	/// The app install's own id, the sync's `client`.
	pub client: String,
	/// When the phone sent it, on the helper's clock.
	pub sent_at: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct UnpairResponse {
	pub helper: Helper,
}

/// `POST /v1/sync`'s 422 for a body that opened but does not read (since the release security review of 2026-10-31;
/// every other route's is bare): the refused request's nonce, base64url without padding, sealed as every body, so only
/// the helper holding the pairing key refuses a request for good, and a device drops what that request carried on this
/// answer alone (contract/companion-api.yaml `SyncUnreadable`; seal.json `refusals`).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct RefusedResponse {
	pub nonce: String,
}

impl RefusedResponse {
	/// The refusal of the request sealed under `nonce`.
	#[must_use]
	pub fn of(nonce: [u8; NONCE]) -> Self {
		Self { nonce: URL_SAFE_NO_PAD.encode(nonce) }
	}
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PingResponse {
	pub helper: Helper,
	/// The request's `t0`, echoed: a device drops a response that is not its request's.
	pub t0: i64,
	pub t1: i64,
	pub t2: i64,
}

/// One 5-s window, its times on the helper's clock.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Sample {
	pub sample_end: i64,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub sent_at: Option<i64>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub bpm: Option<u16>,
	pub steps: u32,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub metres: Option<f64>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub cadence: Option<f64>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub pump_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
	PumpStart,
	PumpDone,
	/// The player's one-click Skip of an unfinished move on the device (since 2026-10-09): `WristEvent::skip`.
	Skip,
	/// The player's Bank it on a match pump's Ready view (docs/plan.md slice 58): `WristEvent::bank`.
	Bank,
	/// A newer app's event this helper does not know: taken and ignored, so the sync still confirms it.
	#[serde(other)]
	Other,
}

/// A pump on the device: a game pump's start and settled card, or a catch-up pump's end with its moves.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct WristEvent {
	#[serde(rename = "type")]
	pub kind: EventType,
	pub pump_id: String,
	/// A `bank`'s (docs/plan.md slice 58): the pump's size, 1 Light, 2 Balanced, 3 Extreme, which its banked rows carry.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub tier: Option<u8>,
	pub start_at: i64,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub seconds: Option<u32>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub steps: Option<u32>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub metres: Option<f64>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub peak_bpm: Option<u16>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub delta_bpm: Option<i32>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub verdict: Option<Verdict>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub slot: Option<u8>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub moves: Option<Vec<CatchupMove>>,
	/// A catch-up pump, or the watch's own pump, stopped for felt unwell: its rows lock every pump until the next game
	/// day.
	#[serde(default, skip_serializing_if = "std::ops::Not::not")]
	pub unwell: bool,
	/// With `unwell`, the moment of the stop on the helper's clock: the lock runs to the next game day after the stop,
	/// so a catch-up that runs across 04:00 locks the new day. An app that sends none has its stop dated `start_at`.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub stopped_at: Option<i64>,
}

impl WristEvent {
	/// The catch-up pump this event is, when it is one: a `pump_done` with its moves, whose id is the device's own
	/// (`w-` from a watch, `p-` from a phone); its start and its felt-unwell stop floored to the helper's unix second (a
	/// stop never before the start), and the verdict a watch measured itself. The watch's own pump (`w-`, no moves) is
	/// taken as its felt-unwell stop alone. An app pump on a phone (`f-`) is none since slice 26: its stop reaches the
	/// helper among the app's rows (`SyncRequest::rows`).
	#[must_use]
	pub fn catch_up(&self, device: Device) -> Option<CatchupPump> {
		let (prefix, source) = if device.is_watch() { ("w-", Source::Watch) } else { ("p-", Source::Phone) };
		let moves = self.moves.clone().unwrap_or_default();
		// a pump stopped for felt unwell before its first move still carries the stop
		if self.kind != EventType::PumpDone || !self.pump_id.starts_with(prefix) || (moves.is_empty() && !self.unwell) {
			return None;
		}
		let unix_s = |ms: i64| u32::try_from(ms.div_euclid(1000)).ok();
		let start = unix_s(self.start_at)?;
		let unwell_at = self.unwell.then(|| self.stopped_at.and_then(unix_s).map_or(start, |at| at.max(start)));
		let measured = self.verdict.filter(|_| device.is_watch()).map(|verdict| Outcome {
			verdict,
			peak: self.peak_bpm,
			delta: self.delta_bpm,
		});
		Some(CatchupPump { source, id: self.pump_id.clone(), start, moves, measured, unwell_at })
	}

	/// The skip this event is, when it is one (since 2026-10-09): a `skip` under the device's own id (`w-` from a
	/// watch, `p-` from a phone) naming the moves skipped, each with the owed row's `ts` the list gave it, at the click
	/// floored to the helper's unix second; `Session::skip` writes it.
	#[must_use]
	pub fn skip(&self, device: Device) -> Option<CatchupPump> {
		let (prefix, source) = if device.is_watch() { ("w-", Source::Watch) } else { ("p-", Source::Phone) };
		let moves = self.moves.clone().unwrap_or_default();
		if self.kind != EventType::Skip || !self.pump_id.starts_with(prefix) || moves.is_empty() {
			return None;
		}
		let start = u32::try_from(self.start_at.div_euclid(1000)).ok()?;
		Some(CatchupPump { source, id: self.pump_id.clone(), start, moves, measured: None, unwell_at: None })
	}

	/// The bank this event is, when it is one (docs/plan.md slice 58): a `bank` under the device's own pump id (`w-`
	/// from a watch, `f-` from a phone: its app pump's, contract/companion-api.yaml `WristEvent`) naming the moves
	/// banked, each a move of the grammar (`kind` 1 to 4, a move id), at the pump's start floored to the helper's unix
	/// second, with its tier (a missing or out-of-range one is Balanced's); a move without a kind is left out, and one
	/// with none left is no bank. `Session::bank` writes it.
	#[must_use]
	pub fn bank(&self, device: Device) -> Option<BankPump> {
		let (prefix, source) = if device.is_watch() { ("w-", Source::Watch) } else { ("f-", Source::Phone) };
		if self.kind != EventType::Bank || !self.pump_id.starts_with(prefix) {
			return None;
		}
		let moves: Vec<BankMove> = self
			.moves
			.iter()
			.flatten()
			.filter_map(|m| {
				let row_kind = m.kind.filter(|&k| kind::is_move(k))?;
				Some(BankMove {
					move_id: m.move_id,
					kind: row_kind,
					ladder: m.ladder.unwrap_or(0),
					rung: m.rung.unwrap_or(0),
					seconds: m.seconds,
					allout: m.allout,
				})
			})
			.collect();
		if moves.is_empty() {
			return None;
		}
		let start = u32::try_from(self.start_at.div_euclid(1000)).ok()?;
		let tier = self.tier.filter(|t| (1..=3).contains(t)).unwrap_or(2);
		Some(BankPump { source, id: self.pump_id.clone(), start, tier, moves })
	}
}

/// One row an app wrote, as a phone sends it in `SyncRequest::rows` (docs/plan.md slice 26) and as `GET /v1/app_rows`
/// serves it back (slice 28): the row grammar's text (`char` 0, the app bit and the device's bit in the flags) and its
/// copy number, the n-th copy of that text among the device's rows, from 1. The text and the number are the row's
/// identity.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct AppRow {
	pub row: String,
	pub n: u16,
}

/// When the device received a command.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Received {
	pub seq: u64,
	pub at: i64,
}

/// `POST /v1/sync`'s request. It takes a field it does not know, as its events take a type they do not know: a newer
/// app adds them, and a 422 would make the phone drop what it sent for good.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SyncRequest {
	pub device: Device,
	/// The app install's own id (a UUID it keeps): two devices of one kind are two clients.
	pub client: String,
	/// The device's clock, echoed in the response.
	pub t0: i64,
	/// When the device sent it, on the helper's clock (`t0` plus its offset).
	pub sent_at: i64,
	pub since: u64,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub samples: Vec<Sample>,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub events: Vec<WristEvent>,
	/// A phone's app rows the helper has not confirmed yet (slice 26): every one is taken, whatever their number, and
	/// the response's `rows_store` confirms them.
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub rows: Vec<AppRow>,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub received: Vec<Received>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub battery: Option<f64>,
	/// False for a request that holds nothing (`SyncRequest::exchange`): a plain post without `wait`, a standing request
	/// with it.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub hold: Option<bool>,
	/// The `offer` of a match's offer (`Command::MatchEnd`) whose pump the device started (slice 35), sent again until
	/// a response arrives: the helper takes the command back and lists the id in `SyncResponse::offer_withdrawn`.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub offer_taken: Option<String>,
	/// When the device last posted a game offer, unix seconds (slice 35): the helper keeps the latest of any device's
	/// and its own, which starts the computer's sitting clock again (slice 48, `sitting`) and every response answers,
	/// for the phones' `link_gap_min` between their offers.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub game_offer_at: Option<i64>,
	/// A phone's Pro proof (slice 35): one that does not read is none, so a proof never fails the request it rides on.
	#[serde(default, skip_serializing_if = "Option::is_none", deserialize_with = "lenient")]
	pub pro_proof: Option<ProProof>,
	/// How long, in milliseconds, the helper may hold this held request while no command waits for the device (since
	/// 2026-10-13): a phone sends 15,000 while no pump runs on it and it has nothing to send, 600,000 over the Bluetooth
	/// link (slice 43). One that does not read is none, `HOLD_MS`; `Link::hold_ms` keeps it between `HOLD_MS` and the
	/// transport's most (`WAIT_MAX_MS`, `BLE_WAIT_MAX_MS`), and honours it only for the client that holds the link and
	/// for a standing request (`Exchange::Standing`), which it makes of a request with `hold: false`.
	#[serde(default, skip_serializing_if = "Option::is_none", deserialize_with = "lenient")]
	pub wait: Option<i64>,
}

/// How the helper answers a sync request, by its `hold` and `wait`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Exchange {
	/// `hold` absent or true: held until a command waits for its device, a newer request of its client, a change of the
	/// game or its hold's end; it makes its client a candidate to hold the link and acknowledges what it saw when its
	/// client holds it.
	Held,
	/// `hold: false` with `wait` (since the review of 2026-10-17): a phone that steps back for its watch keeps one over
	/// Bluetooth, so an answer wakes it. It never counts toward holding the link and, as a plain post, carries no
	/// command and acknowledges none; it waits as a held request does, its `wait` honoured whoever holds the link, and
	/// answers at once while no device holds the link (`Link::unheld`, its answer's `SyncResponse::unheld`), so the
	/// phone takes the link the moment its watch's link goes quiet.
	Standing,
	/// `hold: false` without `wait`: answered at once, no commands, no acknowledgement, and the client's held or
	/// standing request stays in place.
	Plain,
}

impl SyncRequest {
	/// How the helper answers it.
	#[must_use]
	pub fn exchange(&self) -> Exchange {
		match (self.hold, self.wait) {
			(Some(false), None) => Exchange::Plain,
			(Some(false), Some(_)) => Exchange::Standing,
			_ => Exchange::Held,
		}
	}
}

/// The store a phone's Pro proof comes from (slice 35), or `none` on the wire: the store answered without Pro after a
/// proof went (lapsed, refunded, revoked), which drops that phone's grant at once.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProStore {
	AppStore,
	Play,
	#[serde(rename = "none")]
	Lapsed,
}

impl ProStore {
	/// The name the wire and the log lines use.
	#[must_use]
	pub fn name(self) -> &'static str {
		match self {
			Self::AppStore => "app_store",
			Self::Play => "play",
			Self::Lapsed => "none",
		}
	}
}

/// A phone's proof that Pro holds on it (slice 35; `contract/companion-api.yaml`, `ProProof`): the store's own signed
/// record, which the helper checks offline. An iPhone sends its Pro entitlement's signed transaction (`transaction`)
/// and its renewal info (`renewal`), both the App Store's JWS; Android sends Play's purchase JSON as the exact text it
/// signed (`purchase`) and the signature in standard base64 (`signature`).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ProProof {
	pub store: ProStore,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub transaction: Option<String>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub renewal: Option<String>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub purchase: Option<String>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub signature: Option<String>,
}

/// A command as the device gets it: its sequence, what the game did, when the helper queued it and until when it
/// holds.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Queued {
	pub seq: u64,
	#[serde(flatten)]
	pub command: Command,
	pub sent_at: i64,
	pub expires_at: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[expect(clippy::struct_excessive_bools, reason = "the response's facts on the wire, each a yes or no of its own")]
pub struct SyncResponse {
	pub helper: Helper,
	pub t0: i64,
	pub t1: i64,
	pub t2: i64,
	/// The since to send next: the holder's the helper's current sequence (`Link::respond`), any other request's its own
	/// since, except a standing answer that carries `mirror` (slice 51), whose `next` is the highest `seq` among them.
	pub next: u64,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub locked_until: Option<i64>,
	/// The id of the helper's store of app rows (`AppRows::id`, slice 26), in every response from this helper, empty
	/// while the store is: it confirms every row the request carried, and another id than the one a phone kept tells it
	/// to send its rows again. An older helper sends none, which confirms no row.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub rows_store: Option<String>,
	/// The match offers whose notice every device takes back (`match_end`s that had not expired): taken by a device or
	/// the computer (slice 35, `SyncRequest::offer_taken`), put off with the computer card's Later or ×, or ended by a
	/// pump done another way since (slice 48's double-check), in every response to every device.
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub offer_withdrawn: Vec<String>,
	/// A game the player turned on in QuestPump on the computer is in a match (slice 35): the Android phone's
	/// off-screen hold counts the response as word from the helper.
	#[serde(default, skip_serializing_if = "std::ops::Not::not")]
	pub playing: bool,
	/// A game marked `aim` in contract/links.json whose "Rest your arms while it runs" is on runs on the computer now
	/// (slice 48's double-check, `Helper::arms_now`): a device's pump from a computer match's offer rests the arms while
	/// a response within the last minute says so (or, with none that recent, as its `match_end`'s `arms` says).
	#[serde(default, skip_serializing_if = "std::ops::Not::not")]
	pub arms: bool,
	/// The latest game offer the helper knows on any device, unix seconds (slice 35).
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub game_offer_at: Option<i64>,
	/// A World of Warcraft client with QuestPump runs on the computer (slice 35): the game asks the owed moves itself
	/// on its next pump, so a catch-up that reaches the helper meanwhile pays none of them (its felt-unwell stop still
	/// locks), and the devices offer none.
	#[serde(default, skip_serializing_if = "std::ops::Not::not")]
	pub game_asks: bool,
	/// The request's catch-ups whose moves the helper held for the game as it took them (slice 35, `game_asks` then), by
	/// pump id, an event sent again answered as it was taken (`Link::game_held`): the device's end line follows it.
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub game_held: Vec<String>,
	/// No device holds the link as this answer to a standing request leaves (since the review of 2026-10-17,
	/// `Exchange::Standing`): the phone that steps back for its watch takes the link at once. Written only when true,
	/// and only on a standing request's answer.
	#[serde(default, skip_serializing_if = "std::ops::Not::not")]
	pub unheld: bool,
	/// Since slice 51, only on a standing request's answer to a paired phone: the game pump's commands queued for the
	/// device that holds the link whose `seq` is above the request's `since`, in `seq` order (`Link::mirror`), which the
	/// phone shows and never acknowledges or answers; left out when there is none.
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub mirror: Vec<Queued>,
	pub commands: Vec<Queued>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SheetCharacter {
	pub index: u8,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub guid: Option<String>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub name: Option<String>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub realm: Option<String>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub class: Option<String>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub sheet: Option<serde_json::Value>,
}

/// An owed move as the unfinished list shows it, in the order a catch-up pump pays them: the move, its kind, a cardio
/// or flow move's planned seconds, the ladder and rung it was owed at, and whether a cardio move is an all-out window of
/// its own move (a finisher's, or a closing move's since 2026-10-15).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct OwedMove {
	#[serde(rename = "move")]
	pub move_id: u16,
	pub kind: u8,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub seconds: Option<u16>,
	/// The owed row's ladder (0: a move on none) and rung (since 2026-10-12), at which a catch-up doses the move as the
	/// game doses an owed one (Engine.lua `riderSlot`: an owed anchor at its ladder's rung now is the rung's attempt)
	/// and asks its count. Absent from an older helper: a catch-up then asks no count.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub ladder: Option<u8>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub rung: Option<u8>,
	#[serde(default, skip_serializing_if = "std::ops::Not::not")]
	pub allout: bool,
	/// The owed row's time (since 2026-10-09): a skip names it back, and the apps count the move's game days unfinished
	/// from it (`owed_fresh_days`). Absent from an older helper.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub ts: Option<u32>,
}

/// What `GET /v1/sheet` serves beside the helper: the unfinished count and list, the held count, the lock, whether
/// the game asks the owed moves now and the characters.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Sheet {
	pub unfinished: u32,
	/// The owed moves the game holds back now, left out of `owed` (none is written as absent).
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub held: Option<u32>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub locked_until: Option<i64>,
	/// A World of Warcraft client with QuestPump runs on the computer (`SyncResponse::game_asks`): the unfinished list
	/// is the game's next pump's, and no catch-up pays it meanwhile. The runtime's to say: `Session::sheet` leaves it
	/// false.
	#[serde(default, skip_serializing_if = "std::ops::Not::not")]
	pub game_asks: bool,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub owed: Vec<OwedMove>,
	pub characters: Vec<SheetCharacter>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SheetResponse {
	pub helper: Helper,
	#[serde(flatten)]
	pub sheet: Sheet,
}

/// The page of a kept list after the `after` items a device holds: at most `size` items, the `next` to ask from and
/// whether more follow. The list only grows until its store changes, so a count is a place in it; an `after` past the
/// end is an empty page at the end.
#[must_use]
pub fn page<T: Clone>(items: &[T], after: u64, size: usize) -> (Vec<T>, u64, bool) {
	let (at, next, more) = page_bounds(items.len(), after, size);
	(items[at].to_vec(), next, more)
}

/// `page`'s places in a list of `len` items: the items the page takes, the `next` to ask from and whether more follow.
#[must_use]
pub fn page_bounds(len: usize, after: u64, size: usize) -> (std::ops::Range<usize>, u64, bool) {
	let start = usize::try_from(after).map_or(len, |a| a.min(len));
	let end = start.saturating_add(size).min(len);
	(start..end, u64::try_from(end).unwrap_or(u64::MAX), end < len)
}

/// A pump's heart-rate proof as a history page lists it when asked (`proofs_since`): a phone that read the pump row
/// before its verdict was out gets it here.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct HistoryProof {
	pub row_ts: u32,
	pub hr_verdict: Verdict,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub hr_peak: Option<u16>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub hr_delta: Option<i32>,
}

/// One page of the rows the helper keeps, in the store's order (`GET /v1/history`).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct HistoryPage {
	/// Which order `next` counts in (`Store::id`): a device that kept another one asks again from 0.
	pub store: String,
	pub next: u64,
	pub more: bool,
	/// Present when the request named `proofs_since`, oldest first.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub proofs: Option<Vec<HistoryProof>>,
	pub rows: Vec<RowRecord>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct HistoryResponse {
	pub helper: Helper,
	#[serde(flatten)]
	pub page: HistoryPage,
}

/// One page of the store of app rows, in the order the helper took them (`GET /v1/app_rows`, slice 28): the helper's
/// copy of every paired phone's app rows.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct AppRowsPage {
	/// Which order `next` counts in (`AppRows::id`, the sync's `rows_store`): a device that kept another one asks again
	/// from 0.
	pub store: String,
	pub next: u64,
	pub more: bool,
	pub rows: Vec<AppRow>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct AppRowsResponse {
	pub helper: Helper,
	#[serde(flatten)]
	pub page: AppRowsPage,
}

/// `POST /v1/body`'s request (slice 35): a phone's body, each field it changed with when. Its own fields refuse one they
/// do not know (`PairRequest`); its `body`'s take one, as the shared body's version only adds fields.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BodyRequest {
	pub device: Device,
	/// When the phone sent it, on the helper's clock (its ping gave the offset).
	pub sent_at: i64,
	pub body: Body,
}

/// `POST /v1/body`'s answer: the body every app device shares, as the helper keeps it after the merge.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct BodyResponse {
	pub helper: Helper,
	pub body: Body,
}

/// `GET /v1/account`'s and `PUT /v1/account`'s answer (slice 60; contract/sync-api.yaml): the desktop app's sync account
/// with its root secret R, sealed under the pairing key as every body, so only a paired phone reads it; `account` and
/// `r` null while it has none (its Sync off, no account made yet).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct AccountResponse {
	pub helper: Helper,
	pub account: Option<String>,
	pub r: Option<String>,
	/// The account's devices as the server last listed them.
	pub devices: u32,
	/// Whether an email is linked to the account.
	pub email: bool,
}

/// `PUT /v1/account` (slice 60): a phone's sync account, handed to the desktop app on the player's Add.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AccountRequest {
	pub device: Device,
	/// When the phone sent it, on the helper's clock.
	pub sent_at: i64,
	pub account: String,
	pub r: String,
}

/// One page of the sheets the helper has read, in the order it read them (`GET /v1/sheets`).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SheetsPage {
	/// Which order `next` counts in (`SheetLog::page`), as `HistoryPage::store`.
	pub store: String,
	pub next: u64,
	pub more: bool,
	pub sheets: Vec<SheetRecord>,
}

/// `GET /v1/sheets`'s body. Its fields are written out, not a flattened `SheetsPage` as `HistoryResponse` has: a kept
/// sheet is JSON text (`RawValue`), which serde reads only from the body's text, never through a flatten's buffer.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SheetsResponse {
	pub helper: Helper,
	pub store: String,
	pub next: u64,
	pub more: bool,
	pub sheets: Vec<SheetRecord>,
}

impl SheetsResponse {
	#[must_use]
	pub fn new(helper: Helper, page: SheetsPage) -> Self {
		let SheetsPage { store, next, more, sheets } = page;
		Self { helper, store, next, more, sheets }
	}
}

/// A device's delivery statistics for one session, printed when it goes quiet and every `LINE_EVERY_MS` of a session
/// that goes on (the latency harness).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Summary {
	pub device: Device,
	pub client: String,
	/// Commands the device reported receiving that were queued while it held the link, which the delivery times cover.
	pub commands: usize,
	/// Delivery time (receipt minus send, on the helper's clock) of those, median and 95th percentile: the link's own
	/// latency, with no wait for a device to hold it.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub p50_ms: Option<i64>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub p95_ms: Option<i64>,
	/// Commands it reported receiving that were queued while it did not hold the link (no device held it, or another
	/// did), counted apart since 2026-10-14: their time was mostly the wait for a device.
	#[serde(default)]
	pub late: usize,
	/// Battery spent per hour of the session, in percent.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub battery_per_hour: Option<f64>,
	/// The transports the session's requests came on (since slice 43).
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub transport: Option<Transports>,
	/// Battery spent per hour while its requests came on the LAN and over Bluetooth, each over the stretches between
	/// two readings whose later one came on it, at least `BATTERY_HOURS_MIN` of them (since slice 43): the Bluetooth
	/// link's cost beside the LAN's play session.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub battery_lan_per_hour: Option<f64>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub battery_ble_per_hour: Option<f64>,
}

/// Whether a character of an id a device sent may go into a log line: an ASCII letter, digit or hyphen, so no id
/// breaks a line or forges another.
fn tag_char(c: char) -> bool {
	c.is_ascii_alphanumeric() || c == '-'
}

/// The first eight characters of a client's id (an app install's UUID) that may go into a log line (`tag_char`), which
/// name it in the log.
#[must_use]
pub fn client_tag(client: &str) -> String {
	client.chars().filter(|&c| tag_char(c)).take(8).collect()
}

/// A pump's id as a log line shows it: the characters that may go into one (`tag_char`), at most `PUMP_TAG_CHARS`, with
/// `…` after them when any was left out. A game pump's `g-` id comes from the strip, a catch-up's from a device's body,
/// checked only by its prefix (`WristEvent::catch_up`).
#[must_use]
pub fn pump_tag(pump: &str) -> String {
	let mut tag: String = pump.chars().filter(|&c| tag_char(c)).take(PUMP_TAG_CHARS).collect();
	// every character kept is one byte: a shorter tag left one out
	if tag.len() < pump.len() {
		tag.push('…');
	}
	tag
}

/// The least hours of readings a battery rate is given for.
const BATTERY_HOURS_MIN: f64 = 0.1;

/// Battery spent over stretches of a session: the percent and the milliseconds.
#[derive(Default, Clone, Copy)]
struct Spent {
	level: f64,
	ms: i64,
}

impl Spent {
	/// Percent per hour, once the stretches add up to `BATTERY_HOURS_MIN`.
	fn per_hour(self) -> Option<f64> {
		#[expect(clippy::cast_precision_loss, reason = "a session's milliseconds stay far below 2^52")]
		let hours = self.ms as f64 / 3_600_000.0;
		(hours >= BATTERY_HOURS_MIN).then(|| self.level * 100.0 / hours)
	}
}

/// What the helper knows of one client.
#[derive(Default)]
struct Seen {
	/// The last held request's arrival; none after a restart until it syncs.
	held_at: Option<i64>,
	/// The last held request's `since`, as this run reads it: the sequence the device has seen up to.
	since: u64,
	/// The last request of any kind.
	last_at: i64,
	/// Bumped by every held request: an older one still waiting answers at once.
	generation: u64,
	delivery: Vec<i64>,
	/// The receipts of commands queued while this client did not hold the link (`Summary::late`).
	late: usize,
	/// A look at the quiet clients (`Link::quiet`) ended its session: its next request starts another (`Link::starts`).
	ended: bool,
	/// The first and the latest battery reading of the session: (time, level).
	battery: Option<((i64, f64), (i64, f64))>,
	/// The battery spent between readings, by the transport the later one came on (LAN, Bluetooth).
	spent: [Spent; 2],
	/// The transport its last held request came on (slice 43), which the holder's log line names (`Link::holder_tag`).
	transport: Transport,
	/// The central its last held request came through, until that central's subscription ends (slice 43, `Link::gone`).
	central: Option<String>,
	/// The generations of its held requests through `central` pending at the helper: from their arrival until their
	/// answer went to the radio, or was dropped (`Link::answered`). While one is, it counts alive whatever its age, and
	/// its session goes on.
	pending: Vec<u64>,
	/// When the answer of its last held request through a central went to the radio: it counts alive `ALIVE_MS` after.
	answered_at: Option<i64>,
	/// The end of its standing request's wait (`Exchange::Standing`, since the fix review of 2026-10-17): its session
	/// goes on until then, as while a held request of it waits over Bluetooth; a held request of it ends it, and so does
	/// the going of the central it came through (`standing_central`, `Link::gone`); never its answer, since its arrival
	/// (`last_at`) may be 600 s old then and the next one comes within a second.
	standing_until: Option<i64>,
	/// The central its standing request came through; none on the LAN.
	standing_central: Option<String>,
	/// The transports its session's requests came on.
	carried: Option<Transports>,
	/// When the statistics its next `link` line prints began: the session's start, or its last line (`LINE_EVERY_MS`).
	line_from: Option<i64>,
}

impl Seen {
	/// Whether a held request of it waits at the helper over Bluetooth now.
	fn waits(&self) -> bool {
		self.central.is_some() && !self.pending.is_empty()
	}

	/// Whether its session has ended at `now`: no request for `QUIET_MS`, no held request of it pending over Bluetooth
	/// and no standing request of it waiting (either may wait `BLE_WAIT_MAX_MS`).
	fn quiet(&self, now: i64) -> bool {
		!self.waits() && self.standing_until.is_none_or(|until| now > until) && now - self.last_at >= QUIET_MS
	}

	/// Whether it holds a request that counts at `now`: a held request pending over Bluetooth whatever its age, else
	/// its last held request within `ALIVE_MS` of its arrival or of its answer over Bluetooth; with that request's
	/// arrival.
	fn alive(&self, now: i64) -> Option<i64> {
		self.held_at.filter(|_| self.alive_until().is_some_and(|until| now <= until))
	}

	/// The last instant it counts alive while nothing arrives: none before a held request, `i64::MAX` while one waits
	/// over Bluetooth (its answer starts the last `ALIVE_MS`, `Link::answered`), else `ALIVE_MS` after its last held
	/// request's arrival or that request's answer over Bluetooth.
	fn alive_until(&self) -> Option<i64> {
		let held = self.held_at?;
		if self.waits() {
			return Some(i64::MAX);
		}
		Some(self.answered_at.map_or(held, |at| at.max(held)).saturating_add(ALIVE_MS))
	}

	/// Its statistics since the last line, taken: `goes_on` keeps the latest battery reading as the next stretch's
	/// first, for a session whose hourly line this is.
	fn summary(&mut self, (device, client): &Client, goes_on: bool) -> Option<Summary> {
		let transport = self.carried.take();
		let [lan, ble] = std::mem::take(&mut self.spent);
		let battery = self.battery.take();
		if goes_on {
			self.battery = battery.map(|(_, last)| (last, last));
		}
		if self.delivery.is_empty() && self.late == 0 && battery.is_none() {
			return None;
		}
		let mut d = std::mem::take(&mut self.delivery);
		d.sort_unstable();
		let battery_per_hour =
			battery.and_then(|((t0, b0), (t1, b1))| Spent { level: b0 - b1, ms: t1 - t0 }.per_hour());
		Some(Summary {
			device: *device,
			client: client.clone(),
			commands: d.len(),
			p50_ms: crate::percentile(&d, 50),
			p95_ms: crate::percentile(&d, 95),
			late: std::mem::take(&mut self.late),
			battery_per_hour,
			transport,
			battery_lan_per_hour: lan.per_hour(),
			battery_ble_per_hour: ble.per_hour(),
		})
	}
}

/// A client: its device's kind and the install's own id.
type Client = (Device, String);

/// A command waiting for the device that holds the link.
struct Waiting {
	queued: Queued,
	/// A response carried it to the device that held the link (`Link::respond`): it left the helper, whether or not
	/// that device acknowledged it before it expired.
	handed: bool,
}

/// What became of one pump's commands so far (since 2026-10-14): how many were queued, how many left the queue
/// without a response carrying them to a device, and how many still wait.
struct Reach {
	pump: String,
	queued: usize,
	unreached: usize,
	waiting: usize,
}

impl Reach {
	/// Whether its pump's commands are all settled: none waits, and the pump is not the one on offer that may queue
	/// more (`running`).
	fn settled(&self, running: Option<&str>) -> bool {
		self.waiting == 0 && running != Some(self.pump.as_str())
	}
}

/// The pump on offer that may queue more: the latest `pump_offered`'s pump while no `pump_done` of it and no
/// `session_end` was queued since (`Link::offered`, `Link::offer_runs`).
fn running(offered: Option<&(String, u64)>, runs: bool) -> Option<&str> {
	offered.filter(|_| runs).map(|(id, _)| id.as_str())
}

/// A pump some of whose commands left the queue with no response carrying them to a device (`Link::unreached`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unreached {
	pub pump: String,
	/// Its commands that reached no device.
	pub unreached: usize,
	/// Its commands queued.
	pub queued: usize,
}

/// A command left the queue: its pump's tally counts it, as reached when a response carried it or the device that
/// holds the link acknowledged it.
fn left(reach: &mut VecDeque<Reach>, command: &Command, reached: bool) {
	let Some(r) = command.pump_id().and_then(|pump| reach.iter_mut().find(|r| r.pump == pump)) else { return };
	r.waiting = r.waiting.saturating_sub(1);
	if !reached {
		r.unreached += 1;
	}
}

/// A pump event as its client sends it again: the same pump, kind, move and start.
type Taken = (Client, String, EventType, Option<u8>, i64);

fn client(req: &SyncRequest) -> Client {
	(req.device, req.client.clone())
}

fn taken_key(req: &SyncRequest, e: &WristEvent) -> Taken {
	(client(req), e.pump_id.clone(), e.kind, e.slot, e.start_at)
}

/// The command queue and the clients.
pub struct Link {
	seq: u64,
	queue: VecDeque<Waiting>,
	/// Send times of the latest commands not yet receipted, with the client that held the link as each was queued, for
	/// the receipts that come after their acknowledgement.
	sent: VecDeque<(u64, i64, Option<Client>)>,
	clients: HashMap<Client, Seen>,
	/// The pump events taken, each with whether the helper held its catch-up's moves for the game (`game_held`).
	taken: VecDeque<(Taken, bool)>,
	/// The pump the latest `pump_offered` named, and its sequence.
	offered: Option<(String, u64)>,
	/// Whether the pump on offer may queue more: no `pump_done` of it and no `session_end` was queued since.
	offer_runs: bool,
	/// The pumps whose commands are not all settled, or settled with some that reached no device and not told yet
	/// (`unreached`), in the order their first command came, `REACH_KEPT` at most.
	reach: VecDeque<Reach>,
	/// The game pump's commands (`mirrored`) as they were queued, until each expires: a phone's mirror reads them
	/// here (`Link::mirror`), since the holder's acknowledgement takes them off `queue` at once (slice 51).
	shown: VecDeque<Queued>,
}

/// A command of the game's pump, which a phone that stepped back for its watch shows as a mirror (slice 51).
fn mirrored(c: &Command) -> bool {
	matches!(
		c,
		Command::PumpOffered { .. }
			| Command::Countdown { .. }
			| Command::MoveEnd { .. }
			| Command::MoveCounted { .. }
			| Command::Verdict { .. }
			| Command::PumpDone { .. }
			| Command::SessionEnd
	)
}

impl Link {
	/// A link whose sequence starts at `now`: it only grows, across restarts too, so a device's `since` from an
	/// earlier run never acknowledges a command of this one.
	#[must_use]
	pub fn new(now: i64) -> Self {
		Self {
			seq: u64::try_from(now).unwrap_or_default(),
			queue: VecDeque::new(),
			sent: VecDeque::new(),
			clients: HashMap::new(),
			taken: VecDeque::new(),
			offered: None,
			offer_runs: false,
			reach: VecDeque::new(),
			shown: VecDeque::new(),
		}
	}

	/// Drops the commands expired at `now`, each counted for its pump (one no response carried reached no device), and
	/// the tallies settled with every command reached, which `unreached` would never tell: so the tallies stay few
	/// whether or not a caller asks.
	fn expire(&mut self, now: i64) {
		let Self { queue, reach, offered, offer_runs, .. } = self;
		queue.retain(|w| {
			let keep = w.queued.expires_at > now;
			if !keep {
				left(reach, &w.queued.command, w.handed);
			}
			keep
		});
		let running = running(offered.as_ref(), *offer_runs);
		reach.retain(|r| r.unreached > 0 || !r.settled(running));
		self.shown.retain(|q| q.expires_at > now);
	}

	/// What the link keeps of a command it queues under the current sequence: a `pump_offered` names the pump on offer,
	/// which a `pump_done` of it or the game's `session_end` ends, and a command of a pump joins that pump's tally.
	fn tally(&mut self, command: &Command) {
		match command {
			Command::PumpOffered { pump_id, .. } => {
				self.offered = Some((pump_id.clone(), self.seq));
				self.offer_runs = true;
			}
			Command::PumpDone { pump_id, .. } if self.offered.as_ref().is_some_and(|(id, _)| id == pump_id) => {
				self.offer_runs = false;
			}
			Command::SessionEnd => self.offer_runs = false,
			_ => {}
		}
		let Some(pump) = command.pump_id() else { return };
		let i = self.reach.iter().position(|r| r.pump == pump).unwrap_or_else(|| {
			if self.reach.len() == REACH_KEPT {
				self.reach.pop_front();
			}
			self.reach.push_back(Reach { pump: pump.to_owned(), queued: 0, unreached: 0, waiting: 0 });
			self.reach.len() - 1
		});
		self.reach[i].queued += 1;
		self.reach[i].waiting += 1;
	}

	/// The pumps settled at `now`, once the commands expired by then left the queue, some of whose commands reached no
	/// device, each once: none of their commands waits, and the pump is not the one on offer that may queue more (its
	/// `pump_done` or the game's `session_end` came, a newer pump was offered, or it was never offered: a catch-up's
	/// verdicts). A command of the pump queued after that starts a tally of its own (a verdict after the pump's end).
	pub fn unreached(&mut self, now: i64) -> Vec<Unreached> {
		self.expire(now);
		let running = running(self.offered.as_ref(), self.offer_runs);
		let mut out = Vec::new();
		self.reach.retain(|r| {
			let settled = r.settled(running);
			if settled && r.unreached > 0 {
				out.push(Unreached { pump: r.pump.clone(), unreached: r.unreached, queued: r.queued });
			}
			!settled
		});
		out
	}

	/// The last sequence given out, the next response's `next`.
	#[must_use]
	pub fn next(&self) -> u64 {
		self.seq
	}

	/// Queues a command; none when it has expired already (a move seen long after its go). A match's offer replaces one
	/// still waiting: one offer at a time.
	pub fn queue(&mut self, command: Command, now: i64) -> Option<Queued> {
		let expires_at = match &command {
			Command::Countdown { go_at, .. } => go_at + COUNTDOWN_AFTER_GO_MS,
			Command::TurnIn { .. } => now + 30_000,
			Command::PumpOffered { .. } => now + 120_000,
			Command::QuestComplete { .. }
			| Command::Keep { .. }
			| Command::PumpDone { .. }
			| Command::MoveEnd { .. }
			| Command::MoveCounted { .. }
			| Command::Verdict { .. }
			| Command::SessionEnd => now + MINUTE_MS,
			// `match_offer_min` from when it goes out (`offers::Offer::out_at`: once the computer could not ask); a
			// library without the constant queues no match offer
			Command::MatchEnd { .. } => now + crate::offers::tunables()?.offer_s * 1_000,
		};
		if expires_at <= now {
			return None;
		}
		// the helper's 5-s link tick prunes it too (`unreached`); with no device syncing, a caller that never asks
		// (sim_helper) has nothing else prune it
		self.expire(now);
		if matches!(command, Command::MatchEnd { .. }) {
			self.queue.retain(|w| !matches!(w.queued.command, Command::MatchEnd { .. }));
		}
		self.seq += 1;
		self.tally(&command);
		let queued = Queued { seq: self.seq, command, sent_at: now, expires_at };
		self.queue.push_back(Waiting { queued: queued.clone(), handed: false });
		if mirrored(&queued.command) {
			self.shown.push_back(queued.clone());
		}
		let holder = self.holding(now).cloned();
		self.sent.push_back((queued.seq, now, holder));
		if self.sent.len() > SENT_KEPT {
			self.sent.pop_front();
		}
		Some(queued)
	}

	/// Takes a match's offer (`Command::MatchEnd`) off the queue: a device started its pump (`offer_taken`). Whether it
	/// still waited there (the holder may have heard it, or it expired).
	pub fn withdraw(&mut self, offer: &str) -> bool {
		let before = self.queue.len();
		self.queue.retain(|w| !matches!(&w.queued.command, Command::MatchEnd { offer: id, .. } if id == offer));
		self.queue.len() != before
	}

	/// The pump on offer (`offer`, its `pump_offered` as the session builds it now) queued again for the device that
	/// holds the link when that device never heard it: its last held request had seen no sequence up to the pump's
	/// latest offer, and none waits for it (another device acknowledged it, it expired, or the device started again
	/// mid-pump). It goes ahead of every waiting command, so a countdown of the pump that still waits reaches the
	/// device after its moves (a side move's second side is told from a new move by them). None when the holder heard
	/// it, or no device holds the link.
	pub fn reoffer(&mut self, offer: Command, now: i64) -> Option<Queued> {
		let Command::PumpOffered { pump_id, .. } = &offer else { return None };
		self.expire(now);
		let since = self.holding(now).and_then(|c| self.clients.get(c))?.since;
		let heard = self.offered.as_ref().is_some_and(|(id, seq)| {
			id == pump_id && (*seq <= since || self.queue.iter().any(|w| w.queued.seq == *seq))
		});
		if heard {
			return None;
		}
		let queued = self.queue(offer, now)?;
		self.queue.rotate_right(1); // queued last, delivered first
		Some(queued)
	}

	/// The client the commands go to: a watch whose held request came within `ALIVE_MS`, else such a phone, a phone
	/// whose held request is pending over Bluetooth counting alive whatever its age, and for `ALIVE_MS` after its answer
	/// went to the radio (slice 43); the latest request wins.
	fn holding(&self, now: i64) -> Option<&Client> {
		self.clients.iter().filter_map(|(c, s)| s.alive(now).map(|t| (c.0.is_watch(), t, c))).max().map(|(_, _, c)| c)
	}

	/// The kind of device that holds the link (`holder_tag` names its client too).
	#[must_use]
	pub fn holder(&self, now: i64) -> Option<Device> {
		self.holding(now).map(|c| c.0)
	}

	/// The device that holds the link with its client's tag (`client_tag`) and the transport its last held request came
	/// on, for the log's holder line: two clients of one kind taking turns, an app installed again, or a phone moving
	/// between the LAN and Bluetooth change it where `holder` stays.
	#[must_use]
	pub fn holder_tag(&self, now: i64) -> Option<(Device, String, Transport)> {
		self.holding(now).map(|c @ (device, client)| {
			(*device, client_tag(client), self.clients.get(c).map_or(Transport::Lan, |s| s.transport))
		})
	}

	/// The device that holds the link and the transport its last held request came on (the pump's log line,
	/// `→ ios (ble)`).
	#[must_use]
	pub fn holder_via(&self, now: i64) -> Option<(Device, Transport)> {
		self.holder_tag(now).map(|(device, _, transport)| (device, transport))
	}

	/// Whether no device holds the link at `now` (since the review of 2026-10-17): a standing request answers at once
	/// then, its answer saying so (`SyncResponse::unheld`).
	#[must_use]
	pub fn unheld(&self, now: i64) -> bool {
		self.holding(now).is_none()
	}

	/// The first instant after `now` at which a client that counts alive then no longer does with nothing arriving: the
	/// holder going quiet wakes no held request, so a standing request looks again then. None while each one that counts
	/// waits over Bluetooth, whose answer wakes the held requests (`Link::answered`, through the app's wake).
	#[must_use]
	pub fn lapse_at(&self, now: i64) -> Option<i64> {
		let until =
			self.clients.values().filter_map(Seen::alive_until).filter(|&until| now <= until && until < i64::MAX);
		until.min().map(|until| until + 1)
	}

	/// Whether a client is linked over Bluetooth at `now`: its last held request came through a central whose
	/// subscription stands, and it counts alive (the Phones page's Bluetooth row).
	#[must_use]
	pub fn ble_linked(&self, now: i64) -> bool {
		self.clients.values().any(|s| s.central.is_some() && s.alive(now).is_some())
	}

	/// A central's subscription ended (it disconnected, unsubscribed, read the hello anew, or the Bluetooth link let it
	/// go): the clients whose held requests came through it hold the link no more, so it passes to the next device at
	/// once, and a standing request that came through it went with it, so its client's session ends after its silence
	/// (`Seen::quiet`). Whether a client held the link through it.
	pub fn gone(&mut self, central: &str) -> bool {
		let mut any = false;
		for s in self.clients.values_mut() {
			if s.standing_central.as_deref() == Some(central) {
				s.standing_until = None;
				s.standing_central = None;
			}
			if s.central.as_deref() == Some(central) {
				s.central = None;
				s.held_at = None;
				s.pending.clear();
				any = true;
			}
		}
		any
	}

	/// A held request over Bluetooth is no longer pending at `now`: its answer went to the radio, or it was dropped (its
	/// call ended, its central went). Its client counts alive `ALIVE_MS` from here, while no other of it is pending.
	pub fn answered(&mut self, device: Device, client: &str, generation: u64, now: i64) {
		let Some(s) = self.clients.get_mut(&(device, client.to_owned())) else { return };
		let before = s.pending.len();
		s.pending.retain(|&g| g != generation);
		if s.pending.len() != before {
			s.answered_at = Some(now);
		}
	}

	/// Whether a request starts a session of its client's link (since 2026-10-14): a client this run never heard, or
	/// one whose session a look ended (`quiet`, after its silence of `QUIET_MS`), which printed that session's
	/// statistics first. A client back before the look goes on with its session. Asked before `arrive`.
	#[must_use]
	pub fn starts(&self, req: &SyncRequest) -> bool {
		self.clients.get(&client(req)).is_none_or(|s| s.ended)
	}

	/// A request arrived `via` the LAN or a Bluetooth central: its receipts and battery go into the statistics (a
	/// receipt's delivery time only for a command queued while its client held the link, the rest counted as late; the
	/// battery spent since the last reading by the transport it came on), a held one makes its client a candidate to
	/// hold the link (through a central: pending until `answered`) and acknowledges what it saw when it holds it, and a
	/// held or a standing one is its client's latest, so an older one of either answers at once (`is_current`). Returns
	/// the client's generation.
	pub fn arrive(&mut self, req: &SyncRequest, via: &Via, now: i64) -> u64 {
		let key = client(req);
		// a since beyond this run's sequence belongs to another run and acknowledges nothing
		let since = self.since(req.since);
		let transport = via.transport();
		let seen = self.clients.entry(key.clone()).or_default();
		seen.last_at = now;
		seen.ended = false;
		seen.line_from.get_or_insert(now);
		seen.carried = Some(Transports::with(seen.carried, transport));
		for r in &req.received {
			if let Some(i) = self.sent.iter().position(|(seq, _, _)| *seq == r.seq)
				&& let Some((_, sent, holder)) = self.sent.remove(i)
			{
				if holder.as_ref() == Some(&key) {
					seen.delivery.push(r.at.saturating_sub(sent));
				} else {
					seen.late += 1;
				}
			}
		}
		if let Some(level) = req.battery.filter(|b| (0.0..=1.0).contains(b)) {
			if let Some((_, (at, last))) = seen.battery {
				let spent = &mut seen.spent[usize::from(transport == Transport::Ble)];
				spent.level += last - level;
				spent.ms += now - at;
			}
			let first = seen.battery.map_or((now, level), |(first, _)| first);
			seen.battery = Some((first, (now, level)));
		}
		match req.exchange() {
			Exchange::Plain => return seen.generation,
			Exchange::Standing => {
				let wait = req.wait.unwrap_or(HOLD_MS).clamp(HOLD_MS, wait_max(transport));
				seen.standing_until = Some(now.saturating_add(wait));
				seen.standing_central = match via {
					Via::Ble(central) => Some(central.clone()),
					Via::Lan => None,
				};
				seen.generation += 1;
				return seen.generation;
			}
			Exchange::Held => {
				seen.standing_until = None;
				seen.standing_central = None;
			}
		}
		seen.held_at = Some(now);
		seen.since = since;
		seen.generation += 1;
		seen.transport = transport;
		match via {
			Via::Ble(central) => {
				if seen.central.as_ref() != Some(central) {
					seen.pending.clear();
				}
				seen.central = Some(central.clone());
				seen.pending.push(seen.generation);
				// a central has at most that many calls at the helper: one more is a lost answer's, which ends nothing
				if seen.pending.len() > crate::ble::IN_FLIGHT_MAX {
					seen.pending.remove(0);
				}
			}
			Via::Lan => seen.central = None,
		}
		let generation = seen.generation;
		if self.holding(now) == Some(&key) {
			let Self { queue, reach, .. } = self;
			queue.retain(|w| {
				let keep = w.queued.seq > since;
				if !keep {
					left(reach, &w.queued.command, true);
				}
				keep
			});
		}
		generation
	}

	/// The request's pump events not taken yet, which count as taken from now on: an event sent again because its
	/// response was lost reaches the helper's loop once.
	pub fn take_events(&mut self, req: &SyncRequest) -> Vec<WristEvent> {
		let mut new = Vec::new();
		for e in &req.events {
			let key = taken_key(req, e);
			if self.taken.iter().any(|(k, _)| *k == key) {
				continue;
			}
			if self.taken.len() == TAKEN_KEPT {
				self.taken.pop_front();
			}
			self.taken.push_back((key, false));
			new.push(e.clone());
		}
		new
	}

	/// The pump ids among the request's events whose catch-up the helper held for the game (`SyncResponse::game_held`):
	/// `held` names those of the events just taken (`take_events`) whose moves the loop held, kept with their taken
	/// event, so an event sent again (its response lost) is answered as it was taken, whatever the game does since.
	pub fn game_held(&mut self, req: &SyncRequest, held: &[String]) -> Vec<String> {
		let mut out = Vec::new();
		for e in &req.events {
			let key = taken_key(req, e);
			let Some((_, was)) = self.taken.iter_mut().find(|(k, _)| *k == key) else { continue };
			*was |= held.contains(&e.pump_id);
			if *was && !out.contains(&e.pump_id) {
				out.push(e.pump_id.clone());
			}
		}
		out
	}

	/// A device's `since` as this run reads it: one beyond this run's sequence belongs to another run and is 0.
	#[must_use]
	pub fn since(&self, since: u64) -> u64 {
		if since > self.seq { 0 } else { since }
	}

	/// How long a held or standing request that came on `transport` waits while no command waits for its device, after
	/// its `arrive`: `floor` (`HOLD_MS` in the helper), or the request's `wait` up to its transport's most
	/// (`WAIT_MAX_MS` on the LAN, `BLE_WAIT_MAX_MS` over Bluetooth since slice 43, `wait_max`) when that is longer and
	/// its client holds the link, or it is a standing request (`Exchange::Standing`), which holds nothing. Any other client's held request waits
	/// `floor` alone (since 2026-10-13): the link passes to it when the holder's last held request grows older than
	/// `ALIVE_MS` or its central goes, which wakes no held request, so it looks again within `floor` (a phone whose
	/// watch stopped syncing hears the next countdown within 5 s).
	#[must_use]
	pub fn hold_ms(&self, req: &SyncRequest, transport: Transport, floor: i64, now: i64) -> i64 {
		match req.wait {
			Some(wait) if req.exchange() == Exchange::Standing || self.holding(now) == Some(&client(req)) => {
				wait.min(wait_max(transport)).max(floor)
			}
			_ => floor,
		}
	}

	/// Whether a held or standing request is still its client's latest.
	#[must_use]
	pub fn is_current(&self, req: &SyncRequest, generation: u64) -> bool {
		self.clients.get(&client(req)).is_some_and(|s| s.generation == generation)
	}

	/// What a response carries for a device at `now`: the unexpired commands after `since` and the `next` to send,
	/// read under one borrow so a command queued meanwhile is never acknowledged unseen. The device that holds the
	/// link gets every waiting command, each marked as handed to a device, and the current sequence; any other gets
	/// none, and its `since` back, so it never acknowledges what went to the holder when it takes the link over.
	pub fn respond(&mut self, req: &SyncRequest, now: i64) -> (Vec<Queued>, u64) {
		self.expire(now);
		let since = self.since(req.since);
		if self.holding(now) != Some(&client(req)) {
			return (Vec::new(), since);
		}
		let mut commands = Vec::new();
		for w in self.queue.iter_mut().filter(|w| w.queued.seq > since) {
			w.handed = true;
			commands.push(w.queued.clone());
		}
		(commands, self.seq)
	}

	/// What a standing request from a phone (since slice 51: a phone that stepped back for its watch keeps the game pump
	/// on its screen) carries at `now`: the game pump's commands waiting for the holder (`pump_offered`, `countdown`,
	/// `move_end`, `move_counted`, `verdict`, `pump_done`, `session_end`) whose `seq` is above the request's `since`, in
	/// `seq` order, never marked handed and kept for it until each expires (`shown`), and the `next` to
	/// send: the highest `seq` among them, else the request's own `since`. None for another kind of request or a watch.
	pub fn mirror(&mut self, req: &SyncRequest, now: i64) -> (Vec<Queued>, u64) {
		let since = self.since(req.since);
		if req.exchange() != Exchange::Standing || req.device.is_watch() {
			return (Vec::new(), since);
		}
		self.expire(now);
		// in `seq` order as queued; kept until each expires, whatever the holder acknowledged
		let mirror: Vec<Queued> = self.shown.iter().filter(|q| q.seq > since).cloned().collect();
		let next = mirror.last().map_or(since, |q| q.seq);
		(mirror, next)
	}

	/// The statistics of the devices silent for `QUIET_MS`, which this look ends, so each one's next request starts
	/// another session (`starts`), and of the sessions that go on whose last line is `LINE_EVERY_MS` old.
	pub fn quiet(&mut self, now: i64) -> Vec<Summary> {
		let mut out = Vec::new();
		for (c, s) in &mut self.clients {
			if s.quiet(now) {
				if !s.ended {
					s.ended = true;
					s.line_from = None;
					out.extend(s.summary(c, false));
				}
			} else if s.line_from.is_some_and(|from| now - from >= LINE_EVERY_MS) {
				s.line_from = Some(now);
				out.extend(s.summary(c, true));
			}
		}
		out.sort_by(|a, b| (a.device, &a.client).cmp(&(b.device, &b.client)));
		out
	}

	/// Every client's statistics so far (the helper quits).
	pub fn summaries(&mut self) -> Vec<Summary> {
		let mut out: Vec<Summary> = self.clients.iter_mut().filter_map(|(c, s)| s.summary(c, false)).collect();
		out.sort_by(|a, b| (a.device, &a.client).cmp(&(b.device, &b.client)));
		out
	}
}

/// The peers that failed: an address that sent wrong bearers, and since slice 43 a Bluetooth central whose messages
/// were dropped or whose envelopes did not open (`ble.json`'s `failed_max` and `failed_window_ms`): `THROTTLE_AFTER`
/// within `THROTTLE_MS` are refused for `THROTTLE_MS` after the last.
pub struct Throttle<K = IpAddr> {
	peers: HashMap<K, VecDeque<i64>>,
}

impl<K> Default for Throttle<K> {
	fn default() -> Self {
		Self { peers: HashMap::new() }
	}
}

impl<K: Eq + Hash> Throttle<K> {
	/// Whether a peer is refused at `now`.
	#[must_use]
	pub fn refused(&self, peer: &K, now: i64) -> bool {
		self.peers
			.get(peer)
			.is_some_and(|f| f.len() >= THROTTLE_AFTER && f.back().is_some_and(|&t| now - t < THROTTLE_MS))
	}

	/// A failure of a peer's.
	pub fn failed(&mut self, peer: K, now: i64) {
		self.peers.retain(|_, f| f.back().is_some_and(|&t| now - t < THROTTLE_MS));
		let failures = self.peers.entry(peer).or_default();
		failures.push_back(now);
		while failures.len() > THROTTLE_AFTER || failures.front().is_some_and(|&t| now - t >= THROTTLE_MS) {
			failures.pop_front();
		}
	}

	/// Forgets the failures of every peer `keep` says no to (since the review of 2026-10-17: a new pairing code
	/// forgives every Bluetooth central).
	pub fn retain(&mut self, mut keep: impl FnMut(&K) -> bool) {
		self.peers.retain(|peer, _| keep(peer));
	}
}
