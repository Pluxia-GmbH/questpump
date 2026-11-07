//! The browser as the desk's platform (docs/plan.md slice 63, "One desk core"): [`Web`] implements
//! `questpump_desk::Platform` with the Web Worker's imports (`crate::imports`), and [`Page`] is the page's
//! `questpump_desk::publisher::Sink`.
//!
//! - **The clock and the zone**: the Worker's own, as the engine's.
//! - **Persisting**: the data folder is [`DATA`], an Emscripten file system the Worker mounts on the browser's
//!   `IndexedDB` (`IDBFS`) and reads in before the desk opens, so the desk keeps its files there, the row stores' among
//!   them, as it keeps them in the desktop app's data folder; after a write the Worker is told (`imports::persist`) and
//!   copies the folder's changed files into `IndexedDB` once the call is over. A write that fails is logged and the desk
//!   goes on, as the desktop app's does on a full disk.
//! - **The audio**: every sound, take, stop and hold goes to the page as a message, which plays it on its
//!   `AudioContext` (a Worker has none) from the media folder the `shell` topic names.
//! - **The frame**: the browser's own `shell` topic (the desktop app's `ui::Shell`, its fields by the same names): the
//!   platform `web`, the engine's state, the media folder, and the frame's constants (one page size, no full screen of
//!   its own, the focus the page's, no screen reader it can see, no output to pick).
//! - **The sync** (slice 60's, the desk's `questpump_desk::sync`): the account kind `web`, each call and the live socket
//!   a message to the Worker (`imports::net`), which sends it with `fetch` and `WebSocket` from the browser's own
//!   origin, play.questpump.com, to the routes on questpump.com (cross-origin calls, which the server's `Origin` check
//!   takes and answers with CORS) and hands each answer back through the door (`qp_answer`, `qp_live`, here
//!   [`Web::answered`] and [`Web::heard`]). The account's secrets (R, the device token and secret, the keys) stay in
//!   `settings.json` in the data folder, which is the browser's `IndexedDB` for play.questpump.com alone, never in
//!   `localStorage` or a cookie and never in a request but as the routes' own auth key and bearer (docs/plan.md slice
//!   63, "Where the browser keeps its secrets"). A link request's code is the link a phone's camera opens in the phone
//!   app, shown only while the phone apps are in the stores. Sign out wipes the browser's data (`{"wipe": "out"}`), and
//!   so does a removal from the account (`{"wipe": "removed"}`, with its state screen).
//! - **The rest** is a browser's: no game, no devices, the films fetched by move as they show.

use std::collections::HashMap;
use std::path::Path;

use questpump_core::protocol::{Line, Source};
use questpump_core::session::Effects;
use questpump_core::sync::LinkCode;
use questpump_desk::Platform;
use questpump_desk::audio::Channel;
use questpump_desk::platform::{EngineState, Fold};
use questpump_desk::publisher::{Msg, Publisher, Sink};
use questpump_desk::settings::{Levels, SoundSettings};
use questpump_desk::sync::{ANSWER_MOST, Answer, Call, LiveHeard, News, Then, live_hello};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::imports;
use crate::play::Play;

/// The data folder, mounted on `IndexedDB` by the Worker (`web/desk-worker.js`) before the desk opens.
pub const DATA: &str = "/data";

/// The `settings.json` of the data folder, as the desktop app names its own.
const SETTINGS: &str = "settings.json";

/// The `shell` topic, the page's frame.
const SHELL: &str = "shell";

/// The `figures` topic, the move figures' table and where their files are.
const FIGURES: &str = "figures";

/// The Sound and voice page's topic (the desktop app's `desk/audio.rs` publishes its own, with the outputs).
const SOUND: &str = "menu.sound";

/// The desktop app's version, which the browser runs (it deploys from the desktop app's release commit): the About
/// page's and the report's (`build.rs` reads it from `helper/src-tauri/Cargo.toml`).
pub const VERSION: &str = env!("QP_APP_VERSION");

/// The Sound and voice page's test, as the desktop app plays it (`desk/audio.rs`): the go's struck bell, then after
/// `TEST_GAP_MS` a line of the game's voice.
const TEST_SOUND: &str = "go";
const TEST_LINE: &str = "cadence.count";
const TEST_GAP_MS: u32 = 450;

/// The share cards' scenes (`contract/design/scenes/scenes.json`), built in as the desktop app builds them in.
const SCENES: &str = include_str!("../../../../contract/design/scenes/scenes.json");

/// Sends one message to the page (a topic's change, a sound, a stop).
fn post(message: &Value) {
	imports::post(&message.to_string());
}

/// The page's half of publishing: each topic's change goes to it as `{"msg": {topic, rev, data}}`, the browser's own
/// part seeing it on the way (`Play::seen`). The Worker lives as long as its page, so a send never fails.
pub struct Page;

impl Sink for Page {
	fn send(&self, msg: Msg) -> bool {
		let msg = serde_json::to_value(msg).unwrap_or_default();
		if let (Some(topic), Some(data)) = (msg.get("topic").and_then(Value::as_str), msg.get("data")) {
			Play::seen(topic, data);
		}
		post(&json!({ "msg": msg }));
		true
	}
}

#[derive(Serialize, Debug, Clone, PartialEq)]
#[expect(
	clippy::struct_excessive_bools,
	reason = "the `shell` topic's independent flags, which the page reads by name"
)]
struct Shell {
	platform: &'static str,
	version: &'static str,
	dot: &'static str,
	go: Go,
	zoom: f64,
	zooms: [f64; 1],
	fullscreen: bool,
	fullscreen_now: bool,
	focused: bool,
	reader: bool,
	leave: Option<Value>,
	resources: String,
	engine: EngineState,
	output_picked: bool,
	announce: Option<Value>,
}

/// Where the desk sends the page (`shell`'s `go`), taken once per `seq`.
#[derive(Serialize, Debug, Clone, PartialEq)]
struct Go {
	to: &'static str,
	seq: u64,
	chosen: bool,
}

/// The `figures` topic as the desktop app's `desk/figures.rs` publishes it, its folders in the media folder: the stills,
/// the fault pairs, the films and the scenes' pages, each a prefix a file's name is appended to.
#[derive(Serialize)]
struct Figures<'a> {
	card: &'a str,
	crimson: &'a str,
	film: &'a questpump_desk::figures::Film,
	moves: &'a std::collections::BTreeMap<String, questpump_desk::figures::Move>,
	owners: &'a std::collections::BTreeMap<String, questpump_desk::figures::Owner>,
	extras: &'a std::collections::BTreeMap<String, questpump_desk::figures::Extra>,
	faults: &'a std::collections::BTreeMap<String, String>,
	stills: String,
	pairs: String,
	films: String,
	scenes: Option<Value>,
}

/// Publishes the `figures` topic with the media folder `media`'s folders; a table that does not read publishes none,
/// so no figure shows (logged).
fn publish_figures(publisher: &Publisher, media: &str) {
	let Some(table) = questpump_desk::figures::TABLE.as_ref() else {
		log::error!("figures: the built-in figures.json does not read: no figure shows");
		return;
	};
	let scenes = if let Ok(Value::Object(mut scenes)) = serde_json::from_str::<Value>(SCENES) {
		scenes.insert("files".to_owned(), Value::String(format!("{media}scenes/")));
		Some(Value::Object(scenes))
	} else {
		log::error!("figures: the built-in scenes.json does not read: the share cards draw no scene");
		None
	};
	let topic = Figures {
		card: &table.card,
		crimson: &table.crimson,
		film: &table.film,
		moves: &table.moves,
		owners: &table.owners,
		extras: &table.extras,
		faults: &questpump_desk::facts::facts().moves.faults,
		stills: format!("{media}figures/stills/"),
		pairs: format!("{media}figures/pairs/"),
		films: format!("{media}figures/films/"),
		scenes,
	};
	publisher.publish(FIGURES, &topic);
}

/// The sync's transport in the browser: the routes' base (`https://questpump.com/api/sync/v1`, called cross-origin
/// from play.questpump.com with CORS; the page's own origin in a local run served from one address), the calls out by
/// id with what each answer becomes, and the live socket while it stands (its token's generation and what it heard).
struct Net {
	base: String,
	next: u32,
	pending: HashMap<u32, Then>,
	live: Option<(u64, LiveHeard)>,
}

/// A call's answer as the Worker hands it back (`qp_answer`): its status, `Retry-After` and body, or why none came.
#[derive(Deserialize)]
pub struct NetAnswer {
	id: u32,
	status: Option<u16>,
	retry: Option<u64>,
	#[serde(default)]
	body: String,
	error: Option<String>,
}

/// What the live socket heard (`qp_live`): a text message, or its end (its close code, whether it had connected).
#[derive(Deserialize)]
pub struct NetHeard {
	#[serde(rename = "gen")]
	generation: u64,
	text: Option<String>,
	end: Option<NetEnd>,
}

#[derive(Deserialize)]
struct NetEnd {
	code: Option<u16>,
	#[serde(default)]
	connected: bool,
}

/// The browser as the desk's platform.
pub struct Web {
	publisher: Publisher,
	shell: Shell,
	net: Net,
	name: String,
	phones: bool,
}

impl Web {
	/// The browser's platform, publishing its `shell` and `figures` topics through `publisher` (the desk's): `media` is
	/// the digest-named folder of the figures, stills, sounds and takes (`web/build.mjs`), relative to the page; `sync`
	/// the sync routes' base, `name` this browser's name in the account's Devices lists and `phones` whether the phone
	/// apps are in the stores.
	#[must_use]
	pub fn new(publisher: Publisher, media: String, sync: String, name: String, phones: bool) -> Self {
		let shell = Shell {
			platform: "web",
			version: VERSION,
			dot: "no_game",
			go: Go { to: "pump", seq: 0, chosen: false },
			zoom: 1.0,
			zooms: [1.0],
			fullscreen: false,
			fullscreen_now: false,
			focused: true,
			reader: false,
			leave: None,
			resources: media,
			engine: EngineState::Busy,
			output_picked: true,
			announce: None,
		};
		publisher.publish(SHELL, &shell);
		publish_figures(&publisher, &shell.resources);
		let net = Net { base: sync, next: 0, pending: HashMap::new(), live: None };
		Self { publisher, shell, net, name, phones }
	}

	/// A call's answer the Worker handed back: what it becomes, for `Desk::on_sync` (none for a call this platform no
	/// longer waits on).
	pub fn answered(&mut self, answer: NetAnswer) -> Option<News> {
		let then = self.net.pending.remove(&answer.id)?;
		let called = match (answer.status, answer.error) {
			(Some(_), None) if answer.body.len() > ANSWER_MOST => {
				Err(format!("an answer past {ANSWER_MOST} bytes")) // copy-lint: not shown (a log line)
			}
			(Some(status), None) => Ok(Answer { status, retry: answer.retry, body: answer.body.into_bytes() }),
			(_, why) => Err(why.unwrap_or_else(|| "no answer".to_owned())), // copy-lint: not shown (a log line)
		};
		Some(then(called))
	}

	/// What the live socket heard: a message's nudge or its end, for `Desk::on_sync` (none for a socket the desk closed
	/// or a message that does not read).
	pub fn heard(&mut self, heard: NetHeard) -> Option<News> {
		let (generation, live) = self.net.live.as_mut().filter(|(g, _)| *g == heard.generation)?;
		let generation = *generation;
		if let Some(end) = heard.end {
			self.net.live = None;
			return Some(News::LiveEnded(generation, end.code, end.connected));
		}
		live.message(generation, heard.text.as_deref()?)
	}
}

/// Why the browser's data goes.
#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Wipe {
	/// Signed out, or Delete progress in this browser: the page starts again at once.
	Out,
	/// Removed from the account: the page says so, and starts again on the player's press.
	Removed,
}

/// Has the page delete this browser's data and start again as a guest's (`{"wipe": "out"|"removed"}`): the mark that
/// finishes it at the next start should the page close first (`questpump_desk::sync::LEAVING`), then the Worker
/// removes the settings file from `IndexedDB`, so the account's secrets go even where the rest does not.
pub fn wipe(why: Wipe) {
	keep(questpump_desk::sync::LEAVING, b"");
	post(&json!({ "wipe": why }));
}

/// Opens a site's page in a new tab (the page opened the tab inside the player's click, `bridge.js`).
pub fn open(url: &str) {
	post(&json!({ "open": url }));
}

/// Writes a file of the data folder and has the Worker keep it; a failure is logged, as a full disk is on the computer.
pub fn keep(name: &str, bytes: &[u8]) {
	match questpump_core::mirror::write_atomic(Path::new(DATA), name, bytes) {
		Ok(()) => imports::persist(),
		Err(e) => log::warn!("web: {name} was not kept: {e}"),
	}
}

/// The `menu.sound` topic as the desktop app's (`desk/audio.rs`): the switches and the levels, and no output to list or
/// pick (the browser plays on the system's own), looked already.
#[derive(Serialize)]
#[expect(
	clippy::struct_excessive_bools,
	reason = "the page's switches, one yes or no a field, as the topic names them"
)]
struct SoundTopic<'a> {
	sound: bool,
	voice: bool,
	callouts: bool,
	count: bool,
	voice_offers: bool,
	levels: &'a Levels,
	outputs: [(); 0],
	output: Option<()>,
	gone: Option<()>,
	looked: bool,
}

impl Platform for Web {
	/// A browser's rows are an app's, as a phone's: bit 7 beside the app bit (docs/plan.md slice 63, "The pumps").
	const DEVICE: Source = Source::Phone;
	const PUMP_IDS: &'static str = "b-";
	const BY: u8 = questpump_core::codes::by::BROWSER;
	const WIPES_ON_SIGN_OUT: bool = true;

	#[expect(clippy::cast_possible_truncation, reason = "a unix time in milliseconds, far below 2^63")]
	fn now_ms(&self) -> i64 {
		(imports::now_s() * 1000.0).round() as i64
	}

	fn zone(&self) -> fn(i64) -> i64 {
		imports::offset_at
	}

	fn save_settings(&self, file: Value) {
		match serde_json::to_vec_pretty(&file) {
			Ok(bytes) => keep(SETTINGS, &bytes),
			Err(e) => log::warn!("web: the settings did not serialise: {e}"),
		}
	}

	fn save_file(&self, name: &'static str, bytes: Vec<u8>) {
		keep(name, &bytes);
	}

	/// A browser takes rows from no game and no device, so only the stores' own failures and keeping them are its: the
	/// lines, the devices' commands, the mirror and the game's feedback belong to a computer with the game. The Worker
	/// copies the data folder out only when a store wrote (a row kept, or a reset, the far rows, the people or the
	/// proof book: each makes the mirror due or says so), never on the 2-s look's empty take.
	fn effects(&mut self, fx: Effects) {
		for error in fx.store_errors.iter().chain(&fx.sheet_errors) {
			log::warn!("web: {error}");
		}
		let rows = fx.lines.iter().any(|l| matches!(l, Line::Row(_)));
		if rows || fx.mirror.is_some() || fx.people_changed || fx.proof_changed {
			imports::persist();
		}
	}

	const SYNC_KIND: &'static str = questpump_core::sync::WEB;

	fn device_name(&self) -> String {
		self.name.clone()
	}

	fn sync_call(&mut self, call: Call, then: Then) {
		self.net.next = self.net.next.wrapping_add(1);
		let id = self.net.next;
		self.net.pending.insert(id, then);
		let body = call.body.map(|b| String::from_utf8_lossy(&b).into_owned());
		let url = format!("{}{}", self.net.base, call.path);
		let fetch = json!({ "id": id, "method": call.method.name(), "url": url, "token": call.token, "body": body });
		imports::net(&json!({ "fetch": fetch }).to_string());
	}

	fn live_open(&mut self, token: String, generation: u64) {
		self.net.live = Some((generation, LiveHeard::default()));
		let url = format!("{}/live", self.net.base.replacen("http", "ws", 1));
		let live = json!({ "gen": generation, "url": url, "hello": live_hello(&token) });
		imports::net(&json!({ "live": live }).to_string());
	}

	fn live_close(&mut self) {
		if self.net.live.take().is_some() {
			imports::net(&json!({ "live": null }).to_string());
		}
	}

	/// The link a phone's camera opens in the phone app (`https://questpump.com/link#req.…`), which answers it.
	fn request_code(&self, code: &LinkCode) -> String {
		code.to_link()
	}

	/// Only a phone app answers a browser's request, so it shows one only while they are in the stores.
	fn link_requests(&self) -> bool {
		self.phones
	}

	/// Out of the account, which the desk forgot and saved first: the page deletes the data folder and starts again
	/// as a guest's, saying first why when the browser was removed.
	fn signed_out(&mut self, removed: bool) {
		wipe(if removed { Wipe::Removed } else { Wipe::Out });
	}

	fn play_sound(&self, id: &str) {
		post(&json!({ "audio": { "sound": id } }));
	}

	fn play_take(&self, id: &str) {
		post(&json!({ "audio": { "take": id } }));
	}

	fn hush(&self) {
		post(&json!({ "audio": { "hush": true } }));
	}

	fn hold_output(&self, on: bool) {
		post(&json!({ "audio": { "hold": on } }));
	}

	/// The page plays at the levels (each on the desktop app's scale, `bridge.js`), and the page shows the switches.
	fn sound_changed(&mut self, sound: &SoundSettings) {
		post(&json!({ "audio": { "levels": sound.levels } }));
		let topic = SoundTopic {
			sound: sound.on,
			voice: sound.voice,
			callouts: sound.callouts,
			count: sound.count,
			voice_offers: sound.voice_offers,
			levels: &sound.levels,
			outputs: [],
			output: None,
			gone: None,
			looked: true,
		};
		self.publisher.publish(SOUND, &topic);
	}

	fn sound_test(&self, channel: Channel) {
		if channel != Channel::Voice {
			post(&json!({ "audio": { "sound": TEST_SOUND } }));
		}
		if channel != Channel::Sounds {
			let after = if channel == Channel::Master { TEST_GAP_MS } else { 0 };
			post(&json!({ "audio": { "take": TEST_LINE, "after": after } }));
		}
	}

	/// The browser fetches each move's film from the media folder as it shows, so the whole set is there.
	fn films(&self) -> bool {
		true
	}

	fn engine_state(&mut self, state: EngineState, _fold: Option<Fold>) {
		self.shell.engine = state;
		self.publisher.publish(SHELL, &self.shell);
	}
}
