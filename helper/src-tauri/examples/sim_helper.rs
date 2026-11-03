//! A stand-in for the QuestPump helper, to test the phone and watch apps in the Android emulator and the iOS simulator
//! with no game and no tray app running (docs/plan.md slice 7; `contract/companion-api.yaml`). It serves the helper's
//! own LAN routes (`src/lan.rs`: the bearer, the seal, the real held sync, 5 s or a request's `wait` up to 15 s) on
//! the loopback only, answers what they ask like a paired helper with one character (the sheet, the unfinished list,
//! the history, the sheets and the app rows of `contract/fixtures/wire.json`, paged as the helper pages them), and
//! takes the game's side from a script: commands, the felt-unwell lock, the unfinished list and the helper block's
//! game, on a second loopback port. It is an example of the helper crate, so it never ships.
//!
//! Run it from `helper/` (Ctrl-C stops it, as does `POST /quit`):
//!
//! ```text
//! cargo run --example sim_helper -- [--port 8787] [--control-port <port + 1>] [--key <base64url>]
//! ```
//!
//! Each run makes a new pairing key unless `--key` repeats one (the `ready` line prints it). stdout carries one JSON
//! line per thing that happened, `ts` and `type` first: `ready` (the ports, the key, the bearer and the two pair
//! links), `pair`, `sync` (a device's sample count, its last heart rate, its event count, the lock it was answered),
//! `wrist` and `command` (the helper socket's own lines: a device's pump event, a command queued), `catch_up` (the owed
//! moves a catch-up pump paid), `rows` (a phone's app rows: how many came, were new and were dropped), `received` (a
//! device's receipts since the last: how many commands and how fast, and apart how many came late, queued while it did
//! not hold the link; the helper's loop sees no per-command acknowledgement), `holder` (which device holds the link),
//! `offer_taken` (a device started the pump a `match_end` offered), `body` (a phone's body: whether it changed the one
//! every app device shares, and that body), `lock`, `owed`, `playing`, `game` and `stopped`.
//!
//! The pair links: `android_emulator` names 10.0.2.2, which an Android emulator reaches as the Mac's loopback
//! (developer.android.com, "Network addresses"). The phone (`adb -s <phone> shell "am start -a
//! android.intent.action.VIEW -d '<link>' ai.pluxia.questpump"`) or a debug watch (`adb -s <watch> shell "am broadcast
//! -n ai.pluxia.questpump/ai.pluxia.questpump.wear.DebugPair --es link '<link>'"`) takes it. `ios_simulator` names
//! 127.0.0.1, which the simulator shares with the Mac, but the apps' pair link takes only private addresses, and
//! loopback is none: the iPhone app and the watch's `-pairLink` refuse it until a debug build allows loopback.
//!
//! The control port, all JSON (`contract/companion-api.yaml` names every command field):
//!
//! ```text
//! curl -s 127.0.0.1:8788/state                                  # paired devices, last sync per device, events, ...
//! curl -s 127.0.0.1:8788/command -d '{"type":"turn_in","quest_id":26391,"tier":2}'
//! curl -s 127.0.0.1:8788/command \
//!   -d '{"type":"pump_offered","quest_id":26391,"moves":[{"slot":0,"move":12,"target":11}]}'
//! curl -s 127.0.0.1:8788/command -d '{"type":"countdown","slot":0,"go_in_ms":5000}'
//! curl -s 127.0.0.1:8788/command -d '{"type":"keep","slot":0,"seconds":45}'
//! curl -s 127.0.0.1:8788/command -d '{"type":"move_end","slot":0,"cancelled":true}'
//! curl -s 127.0.0.1:8788/command -d '{"type":"verdict","slot":0,"verdict":"proven","peak":152,"delta":27}'
//! curl -s 127.0.0.1:8788/command -d '{"type":"pump_done"}'
//! curl -s 127.0.0.1:8788/command -d '{"type":"pump_done","ended":"paused"}'   # done, banked, paused, unwell, skipped
//! curl -s 127.0.0.1:8788/command -d '{"type":"session_end"}'
//! curl -s 127.0.0.1:8788/command \
//!   -d '{"type":"match_end","link":"counterstrike","name":"Counter-Strike 2","kind":"match"}'
//! curl -s 127.0.0.1:8788/playing -d '{"playing":true}'           # a watched game in a match, in every response
//! curl -s 127.0.0.1:8788/playing -d '{"playing":true,"arms":true}'  # an aim game resting the arms runs too
//! curl -s 127.0.0.1:8788/game -d '{"game_asks":true}'           # WoW with QuestPump runs: no catch-up pays
//! curl -s 127.0.0.1:8788/game -d '{"game_asks":true,"step":"check_again"}'   # the pairing's step too (none,
//!                                                    # check_again, reload, grant, either; none when left out)
//! curl -s 127.0.0.1:8788/game -d '{"game_asks":false,"ble":"no_grant"}'  # the computer's Bluetooth link too (on,
//!                                                    # ask, no_grant, off, no_peripheral, none, stopped; slice 43)
//! curl -s 127.0.0.1:8788/lock -d '{"locked_until":1790049600}'  # unix seconds; {"locked_until":null} clears it
//! curl -s 127.0.0.1:8788/owed -d '[{"move":12,"kind":1},{"move":4,"kind":3,"seconds":40,"allout":true}]'
//! curl -s -X POST 127.0.0.1:8788/quit
//! ```
//!
//! A command may leave out its `pump_id` (the current pump's; a `pump_offered` without one starts a new pump,
//! `g-1-<unix s>`), a countdown its `slot` (the current one), `ticks` (the game's 5), `go_at` (`go_in_ms` from now,
//! by default the ticks' seconds, so the count starts at once) and `start_at` (the ticks before the go), a `keep` or a
//! `move_end` its `slot`, a `pump_done` its `ended` (`done`), a `match_end` its `offer` (a new `m-<uuid>`) and
//! `ended_at` (now). The answer is the command as queued; 410 when it had expired already (or, for a `match_end`, when
//! the built-in library lacks `match_offer_min`).
//!
//! The offer after a match (slice 35) as the helper answers it: a `match_end` queued here counts as the latest game
//! offer; a request's `offer_taken` takes its command off the link and lists its id in `offer_withdrawn` of every
//! response until the command would have expired; every response names the latest game offer any device or the stand-in
//! made (`game_offer_at`) and `playing` and `arms` as `/playing` set them, each as it stands when the response leaves,
//! and a waiting held or standing request answers at once when the ids taken back, `playing` or `arms` change, as the
//! helper's does (slice 48's second fix round). `POST /v1/body` merges a phone's body into one kept in memory, as the
//! helper keeps `body.json`.
//!
//! The stand-in keeps no game rows: a catch-up pump pays the first owed entry of each move it ran, none while `/game`
//! says the game asks the owed moves (every sync and sheet response says so then, `game_asks`, and the sync response
//! names each catch-up it held so in `game_held`), and a felt-unwell stop is only logged (set the lock with `/lock`). A
//! phone's app rows it keeps in memory as the helper keeps them (`Session::take_app_rows`): every response names their
//! store (`rows_store`), a stop among them locks as the helper's would, until the next game day in the computer's zone,
//! beside the lock `/lock` sets, and `GET /v1/app_rows` serves them back after the app rows of `wire.json`, which it
//! holds from the start as another phone's (slice 28: a phone takes them in as rows of the player's other devices).

use std::collections::BTreeMap;
use std::fmt::Display;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use anyhow::{Context as _, bail};
use axum::Router;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use questpump_core::body::Kept;
use questpump_core::link::{
	self, AppRow, AppRowsResponse, Ble, Device, Helper, HistoryPage, HistoryResponse, OwedMove, Queued, Sheet,
	SheetResponse, SheetsPage, SheetsResponse, Step, Summary, SyncRequest, Wow, WristEvent,
};
use questpump_core::mirror::Sight;
use questpump_core::pair::{self, PairLink};
use questpump_core::protocol::{Command, Line};
use questpump_core::seal::Keys;
use questpump_core::session::{COUNTDOWN_TICKS, CatchupPump, Now, People, Session, Zone, pump_id};
use questpump_core::store::Store;
use questpump_helper_lib::lan::{self, Ask, Lan, Links, OfferFacts, SyncAnswer, now_ms};
use questpump_helper_lib::local_offset_at;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use tokio::net::TcpListener;
use tokio::sync::{mpsc, watch};

/// The computer's name in the pair link and in every response, which the apps show.
const NAME: &str = "QuestPump stand-in";
/// The addon load the stand-in's game pumps come from (`g-<load>-<offered unix s>`).
const LOAD: u16 = 1;
/// The Android emulator's alias for the Mac's loopback: a private IPv4, so the apps' pair link takes it.
const ANDROID_HOST: Ipv4Addr = Ipv4Addr::new(10, 0, 2, 2);
/// The contract's example bodies: the stand-in's character, its unfinished list, its history, its sheets and its app
/// rows.
const WIRE: &str = include_str!("../../../contract/fixtures/wire.json");
const USAGE: &str = "usage: sim_helper [--port 8787] [--control-port <port + 1>] [--key <base64url of 32 bytes>]";

struct Args {
	port: u16,
	control: u16,
	key: Option<[u8; 32]>,
}

impl Args {
	fn parse() -> anyhow::Result<Self> {
		let (mut port, mut control, mut key) = (pair::PORT, None, None);
		let mut args = std::env::args().skip(1);
		while let Some(flag) = args.next() {
			let value = args.next().with_context(|| format!("{flag} takes a value; {USAGE}"))?;
			match flag.as_str() {
				"--port" => port = listen_port(&value)?,
				"--control-port" => control = Some(listen_port(&value)?),
				"--key" => {
					let bytes = URL_SAFE_NO_PAD.decode(&value).ok().and_then(|k| <[u8; 32]>::try_from(k).ok());
					key = Some(bytes.context("--key takes the 32-byte key in base64url, as `ready` prints it")?);
				}
				_ => bail!("unknown flag {flag}; {USAGE}"),
			}
		}
		let control = match control {
			Some(c) => c,
			None => port.checked_add(1).context("--port 65535 leaves no control port; pass --control-port")?,
		};
		Ok(Self { port, control, key })
	}
}

/// A port the apps' pair link accepts (1024-65535).
fn listen_port(value: &str) -> anyhow::Result<u16> {
	value.parse().ok().filter(|&p| p >= 1024).with_context(|| format!("{value}: a port in 1024-65535"))
}

/// The helper's clock as its socket's lines write it (`runtime.rs` `now`): RFC 3339 in UTC with milliseconds.
fn ts() -> String {
	let format =
		time::macros::format_description!("[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:3]Z");
	time::OffsetDateTime::now_utc().format(&format).unwrap_or_default()
}

/// A line the helper's socket writes too, as it writes it.
fn print_line(line: &Line) {
	match line.to_json(&ts()) {
		Ok(text) => println!("{text}"),
		Err(e) => eprintln!("sim_helper: a line did not serialise: {e}"),
	}
}

/// A line of the stand-in's own: the time, the kind, then the fields of an object.
fn print_event(kind: &str, fields: Value) {
	#[derive(Serialize)]
	struct Out<'a> {
		ts: String,
		#[serde(rename = "type")]
		kind: &'a str,
		#[serde(flatten)]
		fields: Value,
	}
	match serde_json::to_string(&Out { ts: ts(), kind, fields }) {
		Ok(text) => println!("{text}"),
		Err(e) => eprintln!("sim_helper: a {kind} line did not serialise: {e}"),
	}
}

#[derive(Serialize)]
struct Paired {
	at: i64,
	device: Device,
	name: String,
}

#[derive(Serialize, Default)]
struct LastSync {
	at: i64,
	syncs: u64,
	samples: usize,
	/// The newest heart rate a window carried.
	bpm: Option<u16>,
	events: usize,
	/// The app rows its syncs carried, repeats included.
	rows: usize,
}

#[derive(Serialize)]
struct Taken {
	at: i64,
	device: Device,
	event: WristEvent,
}

#[derive(Serialize)]
struct Received {
	at: i64,
	#[serde(flatten)]
	summary: Summary,
}

/// What the stand-in's loop knows: the state dump's fields.
#[derive(Serialize)]
struct World {
	paired: Vec<Paired>,
	holder: Option<Device>,
	devices: BTreeMap<Device, LastSync>,
	events: Vec<Taken>,
	queued: Vec<Queued>,
	received: Vec<Received>,
	/// The unfinished list, the lock as set (the routes send it only while it lies ahead) and the character.
	sheet: Sheet,
	/// Every row and proof of the history bodies, as one store, and the sheets.
	#[serde(skip)]
	history: HistoryPage,
	#[serde(skip)]
	sheets: SheetsPage,
	/// The pump and the slot a command without them is for.
	pump_id: String,
	slot: u8,
	/// The phones' app rows, kept as the helper keeps them.
	#[serde(skip)]
	app: Session,
	/// The match offers a device took, each until its `match_end` would have expired.
	withdrawn: Vec<(String, i64)>,
	/// The latest game offer any device named or a queued `match_end` made, unix seconds.
	game_offer_at: Option<i64>,
	/// A watched game in a match, as `/playing` set it.
	playing: bool,
	/// An aim game with its arms rest on runs (slice 48's double-check), as `/playing` set it.
	arms: bool,
	/// The body every app device shares, as the helper keeps it.
	body: Kept,
	/// The sync account a phone's Add handed over (`PUT /v1/account`), which `GET /v1/account` answers (slice 60): the
	/// sim joins no server.
	#[serde(skip)]
	account: Option<(String, String)>,
}

/// What the stand-in serves from the start, out of `wire.json`.
struct Seed {
	sheet: Sheet,
	history: HistoryPage,
	sheets: SheetsPage,
	/// Another phone's app rows, which no paired device sent.
	app_rows: Vec<AppRow>,
}

impl World {
	fn new(seed: Seed, now: i64) -> anyhow::Result<Self> {
		let Seed { mut sheet, history, sheets, app_rows } = seed;
		sheet.characters.truncate(1);
		sheet.unfinished = u32::try_from(sheet.owed.len()).unwrap_or(u32::MAX);
		let mut app = Session::new(Store::in_memory(), People::default(), Zone::new(local_offset_at));
		// through the helper's own take-in, as an iPhone's (a page never names the device)
		let at = Now { unix: now.div_euclid(1000), rfc3339: format!("sim-{now}") };
		let (taken, _) = app.take_app_rows(Device::Ios, &app_rows, &at);
		anyhow::ensure!(taken.dropped == 0, "wire.json: {} app rows the helper would drop", taken.dropped);
		Ok(Self {
			paired: Vec::new(),
			holder: None,
			devices: BTreeMap::new(),
			events: Vec::new(),
			queued: Vec::new(),
			received: Vec::new(),
			sheet,
			history,
			sheets,
			pump_id: pump_id(LOAD, unix_s(now)),
			slot: 0,
			app,
			withdrawn: Vec::new(),
			game_offer_at: None,
			playing: false,
			arms: false,
			body: Kept::default(),
			account: None,
		})
	}

	/// The felt-unwell lock's end while it lies ahead, as the helper answers it: the one `/lock` set, or an app stop's.
	fn locked(&self, now: i64) -> Option<i64> {
		let at = now.div_euclid(1000);
		self.sheet.locked_until.filter(|&t| t > at).max(self.app.locked_until(at))
	}

	fn sheet(&self, now: i64) -> Sheet {
		Sheet { locked_until: self.locked(now), ..self.sheet.clone() }
	}

	fn set_owed(&mut self, owed: Vec<OwedMove>) {
		self.sheet.unfinished = u32::try_from(owed.len()).unwrap_or(u32::MAX);
		self.sheet.owed = owed;
	}

	/// A sync's samples, its events not taken before, its app rows, the offer it took and its latest game offer, logged
	/// and kept; the answer is the app rows' store and the catch-ups held (the offer facts ride as the routes read them
	/// when the response leaves: `offers`, `Sim::tell_offers`).
	fn synced(&mut self, request: &SyncRequest, now: i64) -> SyncAnswer {
		let SyncRequest { device, samples, events, rows, .. } = request;
		let device = *device;
		let bpm = samples.iter().rev().find_map(|s| s.bpm);
		let seen = self.devices.entry(device).or_default();
		seen.at = now;
		seen.syncs += 1;
		seen.samples += samples.len();
		seen.events += events.len();
		seen.rows += rows.len();
		seen.bpm = bpm.or(seen.bpm);
		if !rows.is_empty() {
			let at = Now { unix: now.div_euclid(1000), rfc3339: format!("sim-{now}") };
			let (taken, _) = self.app.take_app_rows(device, rows, &at);
			print_event(
				"rows",
				json!({"device": device, "rows": rows.len(), "kept": taken.kept, "dropped": taken.dropped}),
			);
		}
		let locked_until = self.locked(now);
		print_event(
			"sync",
			json!({"device": device, "samples": samples.len(), "bpm": bpm, "events": events.len(),
				"locked_until": locked_until}),
		);
		let mut game_held = Vec::new();
		for event in events {
			print_line(&Line::Wrist { device, event: event.clone() });
			self.events.push(Taken { at: now, device, event: event.clone() });
			if let Some(pump) = event.catch_up(device)
				&& self.pay(device, &pump)
			{
				game_held.push(pump.id);
			}
		}
		if let Some(id) = &request.offer_taken {
			let until = self.queued.iter().find_map(|q| match &q.command {
				Command::MatchEnd { offer, .. } if offer == id => Some(q.expires_at),
				_ => None,
			});
			print_event("offer_taken", json!({"device": device, "offer": id, "known": until.is_some()}));
			if let Some(until) = until
				&& !self.withdrawn.iter().any(|(taken, _)| taken == id)
			{
				self.withdrawn.push((id.clone(), until));
			}
		}
		self.game_offer_at = self.game_offer_at.max(request.game_offer_at);
		self.withdrawn.retain(|&(_, until)| until > now);
		SyncAnswer { rows_store: Some(self.app.app_rows().id()), game_held }
	}

	/// The offer facts every response carries as it leaves, as the helper's `Helper::tell_offers` tells them: the lock,
	/// the offers taken back whose command has not expired, `playing` and `arms` as `/playing` set them, and the latest
	/// game offer.
	fn offers(&self, now: i64) -> OfferFacts {
		OfferFacts {
			locked_until: self.locked(now),
			offer_withdrawn: self
				.withdrawn
				.iter()
				.filter(|&&(_, until)| until > now)
				.map(|(id, _)| id.clone())
				.collect(),
			playing: self.playing,
			arms: self.arms,
			game_offer_at: self.game_offer_at,
		}
	}

	/// A catch-up pump pays the first owed entry of each move it ran: the stand-in's short form of the helper's rule
	/// (`Session::catch_up` pays the oldest owed row of that move), so the next sheet shows the list shorter; none while
	/// the game asks them (`/game`). Whether it held moves for the game (`game_held`).
	fn pay(&mut self, device: Device, pump: &CatchupPump) -> bool {
		let mut owed = self.sheet.owed.clone();
		let mut paid = Vec::new();
		let ran = if self.sheet.game_asks { &[][..] } else { &pump.moves[..] };
		for m in ran {
			if let Some(i) = owed.iter().position(|o| o.move_id == m.move_id) {
				owed.remove(i);
				paid.push(m.move_id);
			}
		}
		self.set_owed(owed);
		print_event(
			"catch_up",
			json!({"device": device, "pump_id": pump.id, "paid": paid, "unwell": pump.unwell_at.is_some(),
				"game_asks": self.sheet.game_asks, "unfinished": self.sheet.unfinished}),
		);
		self.sheet.game_asks && !pump.moves.is_empty()
	}

	/// Fills what a script may leave out of a command (the header comment lists it).
	fn fill(&self, fields: &mut Map<String, Value>, now: i64) {
		let kind = fields.get("type").and_then(Value::as_str).unwrap_or_default().to_owned();
		let of_a_pump =
			matches!(kind.as_str(), "countdown" | "pump_offered" | "keep" | "move_end" | "pump_done" | "verdict");
		if of_a_pump && !fields.contains_key("pump_id") {
			let id = if kind == "pump_offered" { pump_id(LOAD, unix_s(now)) } else { self.pump_id.clone() };
			fields.insert("pump_id".into(), id.into());
		}
		if matches!(kind.as_str(), "countdown" | "keep" | "move_end") {
			fields.entry("slot").or_insert(self.slot.into());
		}
		if kind == "countdown" {
			let ticks = fields.entry("ticks").or_insert(COUNTDOWN_TICKS.into()).as_i64().unwrap_or_default();
			let go_in = fields.remove("go_in_ms").and_then(|v| v.as_i64()).unwrap_or(ticks * 1000);
			let go_at = fields.entry("go_at").or_insert((now + go_in).into()).as_i64().unwrap_or_default();
			fields.entry("start_at").or_insert((go_at - ticks * 1000).into());
		}
		if kind == "match_end" {
			fields.entry("offer").or_insert_with(|| offer_id().into());
			fields.entry("ended_at").or_insert(now.into());
		}
	}

	/// A queued command: kept, and a pump offered or counted down becomes the current one.
	fn adopt(&mut self, queued: &Queued) {
		match &queued.command {
			Command::PumpOffered { pump_id, .. } => self.pump_id.clone_from(pump_id),
			Command::Countdown { pump_id, slot, .. } => {
				self.pump_id.clone_from(pump_id);
				self.slot = *slot;
			}
			// the stand-in's own match offer is the latest game offer, as the helper's is
			Command::MatchEnd { .. } => {
				self.game_offer_at = self.game_offer_at.max(Some(queued.sent_at.div_euclid(1000)));
			}
			_ => {}
		}
		self.queued.push(queued.clone());
	}
}

fn unix_s(now: i64) -> u32 {
	u32::try_from(now.div_euclid(1000)).unwrap_or_default()
}

/// A match offer's id: `m-` and a version-4 UUID, lower-case and hyphenated.
fn offer_id() -> String {
	let mut bytes = [0u8; 16];
	rand::fill(&mut bytes);
	format!("m-{}", uuid::Builder::from_random_bytes(bytes).into_uuid())
}

/// The stand-in's state, shared by its loop and its control routes.
struct Sim {
	links: Links,
	/// The routes' state, whose helper block `/game` sets.
	lan: Arc<Lan>,
	/// The `ready` line's fields, which the state dump repeats.
	ready: Value,
	world: Mutex<World>,
	quit: watch::Sender<bool>,
}

impl Sim {
	fn world(&self) -> MutexGuard<'_, World> {
		self.world.lock().unwrap_or_else(PoisonError::into_inner)
	}

	/// The offer facts as the world holds them now, for every response from now on; a change of the ids taken back,
	/// `playing` or `arms` answers the waiting requests at once, as the helper's does (`Lan::set_offers`).
	fn tell_offers(&self, now: i64) {
		let offers = self.world().offers(now);
		self.lan.set_offers(offers);
	}

	fn on_ask(&self, ask: Ask) {
		let now = now_ms();
		match ask {
			Ask::Pair { device, name, done } => {
				print_event("pair", json!({"device": device, "name": name}));
				self.world().paired.push(Paired { at: now, device, name });
				let _ = done.send(());
			}
			Ask::Unpair { device, done, .. } => {
				print_event("unpair", json!({"device": device}));
				let _ = done.send(());
			}
			Ask::Synced { request, done, .. } => {
				let answer = self.world().synced(&request, now);
				if let Some(id) = &request.offer_taken {
					let _ = self.links.withdraw(id);
				}
				self.tell_offers(now);
				let _ = done.send(answer);
				// the request's receipts are in the statistics already (the route counts them before it asks)
				for summary in self.links.summaries().into_iter().filter(|s| s.commands > 0 || s.late > 0) {
					print_event("received", serde_json::to_value(&summary).unwrap_or_default());
					self.world().received.push(Received { at: now, summary });
				}
				self.watch_holder(now);
			}
			Ask::Sheet { done } => {
				let _ = done.send(self.world().sheet(now));
			}
			Ask::History { after, proofs_since, size, done } => {
				let world = self.world();
				let (rows, next, more) = link::page(&world.history.rows, after, size);
				let proofs = proofs_since.map(|since| {
					let all = world.history.proofs.iter().flatten();
					all.filter(|p| u64::from(p.row_ts) >= since).cloned().collect()
				});
				let _ = done.send(HistoryPage { store: world.history.store.clone(), next, more, proofs, rows });
			}
			Ask::Sheets { after, size, done } => {
				let world = self.world();
				let (sheets, next, more) = link::page(&world.sheets.sheets, after, size);
				let _ = done.send(SheetsPage { store: world.sheets.store.clone(), next, more, sheets });
			}
			Ask::AppRows { after, size, done } => {
				let _ = done.send(self.world().app.app_rows().page(after, size));
			}
			Ask::Body { body, done } => {
				let mut world = self.world();
				let changed = world.body.take(&body);
				print_event("body", json!({"changed": changed, "body": world.body.body}));
				let _ = done.send(world.body.body.clone());
			}
			Ask::Account { put, done } => {
				let mut world = self.world();
				if let Some((account, r)) = put {
					print_event("account", json!({"account": account}));
					world.account = Some((account, questpump_core::sync::text(&r)));
				}
				let (account, r) = world.account.clone().unzip();
				let devices = if account.is_some() { 2 } else { 0 };
				let _ = done.send(Some(lan::AccountState { account, r, devices, email: false }));
			}
		}
	}

	/// The device that holds the link, logged when it changes (a held request, or none for `link::ALIVE_MS`).
	fn watch_holder(&self, now: i64) {
		let holder = self.links.holder(now);
		let mut world = self.world();
		if world.holder != holder {
			world.holder = holder;
			print_event("holder", json!({"device": holder}));
		}
	}
}

/// The helper's loop, as far as the routes ask it.
async fn answer(sim: Arc<Sim>, mut asks: mpsc::UnboundedReceiver<Ask>) {
	let mut tick = tokio::time::interval(Duration::from_secs(1));
	loop {
		tokio::select! {
			ask = asks.recv() => match ask {
				Some(ask) => sim.on_ask(ask),
				None => return,
			},
			_ = tick.tick() => {
				let now = now_ms();
				sim.watch_holder(now);
				// an offer taken back whose command expired leaves the list
				sim.tell_offers(now);
			}
		}
	}
}

fn reply(status: StatusCode, body: &impl Serialize) -> Response {
	match serde_json::to_string(body) {
		Ok(text) => (status, [(header::CONTENT_TYPE, "application/json")], text).into_response(),
		Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
	}
}

fn refuse(detail: impl Display) -> Response {
	reply(StatusCode::BAD_REQUEST, &json!({"error": detail.to_string()}))
}

async fn state(State(sim): State<Arc<Sim>>) -> Response {
	#[derive(Serialize)]
	struct Dump<'a> {
		helper: &'a Value,
		#[serde(flatten)]
		world: &'a World,
	}
	reply(StatusCode::OK, &Dump { helper: &sim.ready, world: &sim.world() })
}

async fn command(State(sim): State<Arc<Sim>>, body: Bytes) -> Response {
	let Ok(Value::Object(mut fields)) = serde_json::from_slice(&body) else {
		return refuse("a JSON object: a command with its type and fields (contract/companion-api.yaml, Command)");
	};
	let now = now_ms();
	sim.world().fill(&mut fields, now);
	let command: Command = match serde_json::from_value(Value::Object(fields)) {
		Ok(command) => command,
		Err(e) => return refuse(e),
	};
	let Some(queued) = sim.links.queue(command, now) else {
		let why = "expired already: a countdown whose go lies over 30 s back, or a match_end the built-in library \
		           gives no life (match_offer_min)";
		return reply(StatusCode::GONE, &json!({"error": why}));
	};
	sim.world().adopt(&queued);
	sim.tell_offers(now);
	print_line(&Line::Command { command: queued.clone() });
	reply(StatusCode::OK, &queued)
}

#[derive(Deserialize)]
struct Lock {
	/// Unix seconds; none clears the lock.
	locked_until: Option<i64>,
}

async fn lock(State(sim): State<Arc<Sim>>, body: Bytes) -> Response {
	let Lock { locked_until } = match serde_json::from_slice(&body) {
		Ok(lock) => lock,
		Err(e) => return refuse(e),
	};
	sim.world().sheet.locked_until = locked_until;
	sim.tell_offers(now_ms());
	print_event("lock", json!({"locked_until": locked_until}));
	reply(StatusCode::OK, &json!({"locked_until": locked_until}))
}

async fn owed(State(sim): State<Arc<Sim>>, body: Bytes) -> Response {
	let owed: Vec<OwedMove> = match serde_json::from_slice(&body) {
		Ok(owed) => owed,
		Err(e) => return refuse(e),
	};
	print_event("owed", json!({"owed": owed}));
	let mut world = sim.world();
	world.set_owed(owed);
	reply(StatusCode::OK, &json!({"unfinished": world.sheet.unfinished, "owed": world.sheet.owed}))
}

#[derive(Deserialize)]
struct Playing {
	playing: bool,
	#[serde(default)]
	arms: bool,
}

async fn playing(State(sim): State<Arc<Sim>>, body: Bytes) -> Response {
	let Playing { playing, arms } = match serde_json::from_slice(&body) {
		Ok(p) => p,
		Err(e) => return refuse(e),
	};
	{
		let mut world = sim.world();
		world.playing = playing;
		world.arms = arms;
	}
	sim.tell_offers(now_ms());
	print_event("playing", json!({"playing": playing, "arms": arms}));
	reply(StatusCode::OK, &json!({"playing": playing, "arms": arms}))
}

/// The game as `/game` sets it: from then on every response's helper block names World of Warcraft with QuestPump
/// (running while `game_asks`, else found), its sight `no_grant` for `grant` and `sees` otherwise, and the pairing's
/// step (since 2026-10-13; `none` when left out), the computer's Bluetooth link (`ble`, since slice 43; absent when
/// left out), and every sync and sheet response says `game_asks`, as the helper's `Lan::set_game` and its sheet do; a
/// change answers the held requests at once.
#[derive(Deserialize)]
struct GameNow {
	game_asks: bool,
	#[serde(default)]
	step: Option<Step>,
	#[serde(default)]
	ble: Option<Ble>,
}

async fn game(State(sim): State<Arc<Sim>>, body: Bytes) -> Response {
	let GameNow { game_asks, step, ble } = match serde_json::from_slice(&body) {
		Ok(g) => g,
		Err(e) => return refuse(e),
	};
	sim.world().sheet.game_asks = game_asks;
	let shown = step.unwrap_or(Step::Done);
	let sight = if shown == Step::Grant { Sight::NoGrant } else { Sight::Sees };
	let _ = sim.lan.set_game(Wow::Addon, sight, shown, game_asks, ble);
	print_event("game", json!({"game_asks": game_asks, "step": step, "ble": ble}));
	reply(StatusCode::OK, &json!({"game_asks": game_asks, "step": step, "ble": ble}))
}

async fn quit(State(sim): State<Arc<Sim>>) -> Response {
	sim.quit.send_replace(true);
	reply(StatusCode::OK, &json!({"stopping": true}))
}

/// The bodies of one kind in `wire.json`, in the file's order.
fn bodies<'a>(wire: &'a Value, kind: &'a str) -> impl Iterator<Item = Value> + 'a {
	wire["bodies"].as_array().into_iter().flatten().filter(move |b| b["kind"] == kind).map(|b| b["json"].clone())
}

/// The stand-in's character: the sheet, the history pages of `wire.json` joined into one store, the sheets, and the
/// rows of its app rows pages.
fn seed() -> anyhow::Result<Seed> {
	let wire: Value = serde_json::from_str(WIRE)?;
	let body = |kind: &str| bodies(&wire, kind).next().with_context(|| format!("wire.json has no {kind}"));
	let sheet: SheetResponse = serde_json::from_value(body("sheet_response")?)?;
	// through the text: a kept sheet (`RawValue`) reads only from JSON text
	let sheets: SheetsResponse = serde_json::from_str(&body("sheets_response")?.to_string())?;
	let mut pages = bodies(&wire, "history_response").map(serde_json::from_value::<HistoryResponse>);
	let mut history = pages.next().context("wire.json has no history_response")??.page;
	for page in pages {
		let page = page?.page;
		history.rows.extend(page.rows);
		history.proofs.get_or_insert_default().extend(page.proofs.into_iter().flatten());
	}
	let sheets = SheetsPage { store: sheets.store, next: sheets.next, more: sheets.more, sheets: sheets.sheets };
	let mut app_rows = Vec::new();
	for page in bodies(&wire, "app_rows_response") {
		app_rows.extend(serde_json::from_value::<AppRowsResponse>(page)?.page.rows);
	}
	Ok(Seed { sheet: sheet.sheet, history, sheets, app_rows })
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let args = Args::parse()?;
	let key = args.key.unwrap_or_else(|| {
		let mut key = [0; 32];
		rand::fill(&mut key);
		key
	});
	let mut hid = [0; 16]; // a stand-in install's id, so its hello answers as a helper's does
	rand::fill(&mut hid);
	let seed = seed()?;
	let served = TcpListener::bind((Ipv4Addr::LOCALHOST, args.port))
		.await
		.with_context(|| format!("the LAN routes on 127.0.0.1:{}", args.port))?;
	let controlled = TcpListener::bind((Ipv4Addr::LOCALHOST, args.control))
		.await
		.with_context(|| format!("the control routes on 127.0.0.1:{}", args.control))?;
	let links = Links::new(now_ms());
	let (asks_tx, asks) = mpsc::unbounded_channel();
	// the address every response carries: a device follows only a private one, so neither emulator moves off its link
	let helper = Helper {
		name: NAME.to_owned(),
		host: Ipv4Addr::LOCALHOST.to_string(),
		port: args.port,
		version: env!("CARGO_PKG_VERSION").to_owned(),
		t: 0,
		mdns: None,
		hid: Some(URL_SAFE_NO_PAD.encode(hid)),
		wow: None,
		sight: None,
		step: None,
		ble: None,
	};
	let lan = Lan::new(&key, links.clone(), helper, asks_tx, Duration::from_millis(link::HOLD_MS.unsigned_abs()));
	let pair_link = |host| {
		PairLink { host: IpAddr::V4(host), port: args.port, key, name: NAME.to_owned(), mdns: None, hid: Some(hid) }
			.to_url()
	};
	let ready = json!({
		"lan": format!("127.0.0.1:{}", args.port),
		"control": format!("127.0.0.1:{}", args.control),
		"hold_ms": link::HOLD_MS,
		"wait_max_ms": link::WAIT_MAX_MS,
		"key": URL_SAFE_NO_PAD.encode(key),
		"bearer": Keys::derive(&key).bearer,
		"pair_links": {"android_emulator": pair_link(ANDROID_HOST), "ios_simulator": pair_link(Ipv4Addr::LOCALHOST)},
	});
	let (quitting, stopped) = watch::channel(false);
	let world = Mutex::new(World::new(seed, now_ms())?);
	let sim = Arc::new(Sim { links, lan: lan.clone(), ready: ready.clone(), world, quit: quitting });
	tokio::spawn(lan::serve(vec![served], lan, stopped.clone()));
	tokio::spawn(answer(sim.clone(), asks));
	print_event("ready", ready);
	let routes = Router::new()
		.route("/state", get(state))
		.route("/command", post(command))
		.route("/lock", post(lock))
		.route("/owed", post(owed))
		.route("/playing", post(playing))
		.route("/game", post(game))
		.route("/quit", post(quit))
		.with_state(sim);
	let mut stop = stopped;
	axum::serve(controlled, routes)
		.with_graceful_shutdown(async move {
			let _ = stop.wait_for(|&stopping| stopping).await;
		})
		.await?;
	print_event("stopped", json!({}));
	Ok(())
}
