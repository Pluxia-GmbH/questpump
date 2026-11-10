//! The browser's door to JavaScript (the exports `main.rs` lists): one desk on the Web Worker's thread, with the
//! addon's own engine behind the desk's door (`questpump_desk::engine::Door`, which runs each call on this thread), its
//! calls as JSON in and out, and the host's clock, zone, report, page and keeping as the module's imports
//! (`crate::imports`). After every call the door runs the engine's answers the call queued (and those they queue in
//! turn), builds the Log once the turn made it due and answers the pump's next wake, which the Worker sleeps until
//! (`qp_due`), as the desktop app's loop does around its `select!`.

use std::cell::RefCell;
use std::ffi::{CStr, CString, c_char};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::Path;
use std::sync::Arc;

use questpump_core::app_rows::{self, AppRows};
use questpump_core::body::Kept;
use questpump_core::far::{self, FarRows};
use questpump_core::proof::Book;
use questpump_core::session::{People, Session, Zone};
use questpump_core::sheets::{self, SheetLog};
use questpump_core::store::Store;
use questpump_desk::engine::Door;
use questpump_desk::menu_pages::about::{ABOUT, About, page_url};
use questpump_desk::publisher::Publisher;
use questpump_desk::settings::{self, Settings};
use questpump_desk::{Desk, Intent, Platform as _};
use questpump_engine::Clock;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::imports;
use crate::play::{self, Play};
use crate::web::{self, DATA, NetAnswer, NetHeard, Page, VERSION, Web};

/// The page's label in the publisher: the browser has one page.
const PAGE: &str = "play";
/// The page's presence for work mode's clock (slice 67, `lib/bridge.js`): `{present, since_s?}`, the platform's own.
const PRESENCE: &str = "presence.seen";

/// The browser's desk and its own part.
struct Open {
	desk: Desk<Web>,
	play: Play,
}

thread_local! {
	/// The desk, once `qp_open` opened it.
	static OPEN: RefCell<Option<Open>> = const { RefCell::new(None) };
}

#[expect(clippy::cast_possible_truncation, reason = "a unix time in whole seconds, as the host's clock counts them")]
fn now() -> i64 {
	imports::now_s().floor() as i64
}

/// A refusal as the page reads it: its kind (`shape`, `refused`, `panicked`) and its words.
struct Refusal {
	kind: &'static str,
	message: String,
}

impl Refusal {
	fn shape(message: impl Into<String>) -> Self {
		Self { kind: "shape", message: message.into() }
	}

	fn refused(message: impl Into<String>) -> Self {
		Self { kind: "refused", message: message.into() }
	}

	fn closed() -> Self {
		Self::refused("the desk is not open: qp_open comes first")
	}
}

/// An answer as the door writes it: `{"ok": value}`, or `{"error": {kind, message}}`.
fn reply(answer: Result<Value, Refusal>) -> *mut c_char {
	let body = match answer {
		Ok(value) => json!({ "ok": value }),
		Err(Refusal { kind, message }) => json!({ "error": { "kind": kind, "message": message } }),
	};
	// serde_json writes U+0000 as \u0000, so a JSON text holds no NUL and the CString always takes it whole
	CString::new(body.to_string()).unwrap_or_default().into_raw()
}

/// Runs `f`, a panic in it answered as a refusal of kind `panicked` and logged, so the calls after it are still served.
fn guarded(f: impl FnOnce() -> Result<Value, Refusal>) -> Result<Value, Refusal> {
	catch_unwind(AssertUnwindSafe(f)).unwrap_or_else(|payload| {
		let message = questpump_engine::panic_text(&*payload);
		log::error!("web: a call panicked: {message}");
		Err(Refusal { kind: "panicked", message })
	})
}

/// How the desk's door runs an engine call that may panic (`questpump_desk::engine::Caught`).
fn caught(f: Box<dyn FnOnce() + '_>) -> std::thread::Result<()> {
	catch_unwind(AssertUnwindSafe(f))
}

/// A NUL-terminated argument read as UTF-8 JSON.
///
/// # Safety
/// `text` is a NUL-terminated string the caller keeps until the call returns.
unsafe fn json_arg(what: &str, text: *const c_char) -> Result<Value, Refusal> {
	// SAFETY: the caller passes a NUL-terminated string it keeps until this returns (the function's contract)
	let bytes = unsafe { CStr::from_ptr(text) }.to_bytes();
	serde_json::from_slice(bytes).map_err(|e| Refusal::shape(format!("{what}: {e}")))
}

/// The end of every call on the open desk, as the desktop app's loop ends its turn: the engine's answers the call
/// queued run on the desk (and those they queue in turn), the Log is built once the turn made it due, the browser's own
/// part takes a Result it saw; the answer is the pump's next wake (unix ms, none while nothing waits).
fn settle(open: &mut Open) -> Value {
	while let Some(back) = open.desk.engine.try_answer() {
		back(&mut open.desk);
	}
	open.desk.log_flush();
	open.play.take(now());
	open.play.account(open.desk.settings.sync.held(), open.desk.sync_way_back());
	json!({ "wake": open.desk.pump_wake() })
}

/// The open desk, `f` run on it and the call settled, or a refusal before `qp_open`.
fn with_open(f: impl FnOnce(&mut Open) -> Result<(), Refusal>) -> Result<Value, Refusal> {
	OPEN.with_borrow_mut(|open| {
		let open = open.as_mut().ok_or_else(Refusal::closed)?;
		let done = f(open);
		let wake = settle(open);
		done.map(|()| wake)
	})
}

/// `qp_open`'s arguments: the media folder the page reaches the figures, stills, sounds and takes in (relative to the
/// page: `web/build.mjs`'s digest-named folder), whether the phone apps are in the stores, the sync routes' base (the
/// page's origin and `/api/sync/v1`) and this browser's name in the account's Devices lists.
#[derive(Deserialize)]
struct OpenArgs {
	media: String,
	#[serde(default)]
	phones: bool,
	sync: String,
	name: String,
}

/// Safety and about's topic as the browser has it: the desk's, with no Pro row and nothing done on a computer.
fn publish_about(publisher: &Publisher) {
	publisher.publish(ABOUT, &About::<(), ()>::new(VERSION, None, None));
}

/// The `about` area's intents: the topic again, and a site's page opened in a new tab (`{"open": url}`).
fn about_intent(open: &Open, intent: &Intent) -> Result<(), String> {
	match intent.kind() {
		"about.look" => publish_about(&open.desk.publisher),
		"about.open" => web::open(page_url(intent)?),
		_ => return Err(intent.unknown()),
	}
	Ok(())
}

/// Logs a store's lines that did not read.
fn damaged(file: &str, lines: usize) {
	if lines > 0 {
		log::warn!("{file}: {lines} lines did not read and were skipped");
	}
}

/// The session from the data folder, as the desktop app opens its own: the rows, the app's rows, the sheets, the
/// people, the heart-rate proofs and the other devices' rows; a store that cannot be read starts empty in memory for
/// this run (logged), so the page still opens. With it whether `far.jsonl` lost rows, which the sync takes in again
/// (`Desk::sync_start`).
fn open_session(dir: &Path, settings: &Settings) -> (Session, bool) {
	let store = match Store::open(dir) {
		Ok((store, lines)) => {
			damaged("rows.jsonl", lines);
			store
		}
		Err(e) => {
			log::error!("rows.jsonl could not be read ({e}); the rows of this run stay in memory");
			Store::in_memory()
		}
	};
	let floor = store.floor();
	let app = match AppRows::open(dir, floor) {
		Ok((app, lines)) => {
			damaged(app_rows::FILE, lines);
			app
		}
		Err(e) => {
			log::error!("{} could not be read ({e}); the app rows of this run stay in memory", app_rows::FILE);
			AppRows::in_memory()
		}
	};
	let sheet_log = match SheetLog::open(dir, floor) {
		Ok((log, lines)) => {
			damaged(sheets::FILE, lines);
			log
		}
		Err(e) => {
			log::warn!("{} could not be read ({e}); the sheets of this run stay in memory", sheets::FILE);
			SheetLog::in_memory()
		}
	};
	let (people, unread) = People::load(dir);
	if let Some(why) = unread {
		log::warn!("{}: {why}; the characters' bindings start empty", People::FILE);
	}
	let mut session = Session::new(store, people, Zone::new(imports::offset_at));
	session.set_app_rows(app);
	session.set_sheet_log(sheet_log);
	session.set_display_lag(settings.display_lag_ms);
	session.set_book(Book::load(dir));
	let far_lost = match FarRows::open(dir) {
		Ok((rows, lines)) => {
			damaged(far::FILE, lines);
			session.set_far_rows(rows);
			lines > 0
		}
		Err(e) => {
			log::warn!("{} could not be read ({e}); the other devices' rows of this run stay in memory", far::FILE);
			true
		}
	};
	(session, far_lost)
}

/// The settings from the data folder; a file that does not read starts at the defaults (logged).
fn open_settings(dir: &Path) -> Settings {
	let Ok(bytes) = std::fs::read(dir.join(settings::FILE)) else { return Settings::default() };
	serde_json::from_slice(&bytes).unwrap_or_else(|e| {
		log::warn!("{}: {e}; the settings start at their defaults", settings::FILE);
		Settings::default()
	})
}

/// Opens the browser's desk (again: a new one in place of the old) on the data folder the Worker mounted and read in
/// (`web::DATA`): the stores, the settings and the body read, the engine started on the host's clock and zone, the
/// `shell`, `figures` and `play` topics and every page's topic published, and the engine's fold of every row asked;
/// answered with the pump's next wake.
///
/// # Safety
/// `args` is a NUL-terminated string the caller keeps until the call returns.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn qp_open(args: *const c_char) -> *mut c_char {
	imports::log_to_console();
	// SAFETY: the caller passes a NUL-terminated string it keeps until this returns (the function's contract)
	let args = unsafe { json_arg("qp_open's arguments", args) };
	reply(guarded(|| {
		let args: OpenArgs =
			serde_json::from_value(args?).map_err(|e| Refusal::shape(format!("qp_open's arguments: {e}")))?;
		let dir = Path::new(DATA);
		let publisher = Publisher::default();
		let kept = std::fs::read(dir.join(play::FILE)).ok();
		let play = Play::new(publisher.clone(), kept.as_deref(), args.phones);
		let mut settings = open_settings(dir);
		// a browser pairs no phone (no LAN API, no Bluetooth link): no card or step offers the pairing code
		settings.pairing.phone_apps = Some(false);
		// and makes no account in silence: Sync stays off until the player saves, signs in or links (slice 63)
		if settings.sync.account.is_none() {
			settings.sync.on = false;
		}
		let (session, far_lost) = open_session(dir, &settings);
		let body = Kept::load(dir, now() * 1_000);
		if let Some(why) = &body.unread {
			log::warn!("{}: {why}; the body starts empty until a device fills it", Kept::FILE);
		}
		let clock = Clock { now: Box::new(now), offset_at: Box::new(imports::offset_at) };
		let platform = Web::new(publisher.clone(), args.media, args.sync, args.name, args.phones);
		let mut desk = Desk::new(platform, publisher, session, settings, body, Door::start(clock, caught));
		desk.engine_start();
		desk.book_log_start();
		desk.publish_setup();
		desk.publish_places();
		desk.publish_expedition();
		desk.publish_welcome();
		desk.home_stale();
		desk.pump_refresh();
		desk.welcome_found();
		let sound = desk.settings.sound.clone();
		desk.p.sound_changed(&sound);
		desk.sync_start(far_lost);
		publish_about(&desk.publisher);
		OPEN.with_borrow_mut(|held| *held = Some(Open { desk, play }));
		with_open(|_| Ok(()))
	}))
}

/// Subscribes the page (in place of the one before: a page loaded again) and answers every topic's value
/// (`{"topics": {name: {rev, data}}}`); every later change comes as an `{"msg": …}` message.
#[unsafe(no_mangle)]
pub extern "C" fn qp_subscribe() -> *mut c_char {
	reply(guarded(|| {
		OPEN.with_borrow(|open| {
			let open = open.as_ref().ok_or_else(Refusal::closed)?;
			let boot = open.desk.publisher.subscribe(PAGE, 0, Arc::new(Page));
			serde_json::to_value(boot).map_err(|e| Refusal::shape(format!("the topics: {e}")))
		})
	}))
}

/// An intent of the page (`{"type": "<area>.<what>", …}`): the browser's own `play.*` to its part, the desk's areas
/// to the desk; answered with the pump's next wake once its area took it, else why not.
///
/// # Safety
/// `intent` is a NUL-terminated string the caller keeps until the call returns.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn qp_intent(intent: *const c_char) -> *mut c_char {
	// SAFETY: the caller passes a NUL-terminated string it keeps until this returns (the function's contract)
	let intent = unsafe { json_arg("the intent", intent) };
	reply(guarded(|| {
		let intent = Intent::from_value(intent?).map_err(Refusal::shape)?;
		with_open(|open| {
			// a guest's Delete progress in this browser in an account it holds no way back into (Save stopped at the
			// code): it leaves the account as Sign out anyway does (`DELETE /session`, whose last device takes the
			// account with it), so nothing stays behind on the server, then wipes
			if intent.kind() == "play.wipe" && open.desk.settings.sync.held() {
				let leave =
					Intent::from_value(json!({ "type": "email.signout", "anyway": true })).map_err(Refusal::shape)?;
				return open.desk.intent(&leave).map_err(Refusal::refused);
			}
			if intent.kind() == PRESENCE {
				let present = intent.arg("present").and_then(Value::as_bool).unwrap_or(false);
				let since_s = intent.arg("since_s").and_then(Value::as_i64).unwrap_or(0);
				open.desk.p.presence_seen(present, since_s, now());
				Ok(())
			} else if intent.area() == play::TOPIC {
				open.play.intent(&intent).map_err(Refusal::refused)
			} else if intent.area() == "about" {
				about_intent(open, &intent).map_err(Refusal::refused)
			} else if Desk::<Web>::takes(intent.area()) {
				open.desk.intent(&intent).map_err(Refusal::refused)
			} else {
				Err(Refusal::refused(intent.unknown()))
			}
		})
	}))
}

/// The pump's wake came (the instant the call before answered): whatever is due happens.
#[unsafe(no_mangle)]
pub extern "C" fn qp_due() -> *mut c_char {
	reply(guarded(|| {
		with_open(|open| {
			open.desk.pump_due();
			Ok(())
		})
	}))
}

/// The look every 2 s, as the desktop app's loop looks: the session's verdicts due, the views again (the clock, the
/// lock, a new game day), the pump's gates, the first five minutes and the places' times.
#[unsafe(no_mangle)]
pub extern "C" fn qp_look() -> *mut c_char {
	reply(guarded(|| {
		with_open(|open| {
			let desk = &mut open.desk;
			let fx = desk.session.tick(now() * 1_000);
			desk.apply(fx);
			desk.recent_look(now());
			desk.p.presence_look(now());
			desk.home_look();
			desk.pump_refresh();
			desk.welcome_found();
			desk.places_tick();
			desk.sync_drive();
			Ok(())
		})
	}))
}

/// A sync call's answer as the Worker got it (`{"id", "status", "retry", "body"}`, or `{"id", "error"}` when none
/// came), to the desk's sync.
///
/// # Safety
/// `answer` is a NUL-terminated string the caller keeps until the call returns.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn qp_answer(answer: *const c_char) -> *mut c_char {
	// SAFETY: the caller passes a NUL-terminated string it keeps until this returns (the function's contract)
	let answer = unsafe { json_arg("the answer", answer) };
	reply(guarded(|| {
		let answer: NetAnswer =
			serde_json::from_value(answer?).map_err(|e| Refusal::shape(format!("the answer: {e}")))?;
		with_open(|open| {
			if let Some(news) = open.desk.p.answered(answer) {
				open.desk.on_sync(news);
			}
			Ok(())
		})
	}))
}

/// What the live socket heard (`{"gen", "text"}`, or `{"gen", "end": {"code", "connected"}}` once it closed), to the
/// desk's sync.
///
/// # Safety
/// `heard` is a NUL-terminated string the caller keeps until the call returns.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn qp_live(heard: *const c_char) -> *mut c_char {
	// SAFETY: the caller passes a NUL-terminated string it keeps until this returns (the function's contract)
	let heard = unsafe { json_arg("the live socket's", heard) };
	reply(guarded(|| {
		let heard: NetHeard =
			serde_json::from_value(heard?).map_err(|e| Refusal::shape(format!("the live socket's: {e}")))?;
		with_open(|open| {
			if let Some(news) = open.desk.p.heard(heard) {
				open.desk.on_sync(news);
			}
			Ok(())
		})
	}))
}

/// Gives an answer's memory back.
///
/// # Safety
/// `answer` is an answer of the door not given back yet, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn qp_free(answer: *mut c_char) {
	if !answer.is_null() {
		// SAFETY: the pointer came from CString::into_raw in `reply` and comes back once (the function's contract)
		drop(unsafe { CString::from_raw(answer) });
	}
}
