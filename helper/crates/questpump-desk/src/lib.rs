//! The desk of QuestPump's desktop app and of its browser version (docs/plan.md slice 63, "One desk core"): the logic
//! behind the pages `helper/dist` draws, out of the desktop app's `impl Helper` into a crate of its own, so the desktop
//! app (`helper/src-tauri`) and the browser (`questpump-web`, `wasm32-unknown-emscripten`) run one desk on one engine.
//! The desk holds its state and its [`Platform`] (`Desk::p`), and asks the platform for everything outside it: the
//! clock and the time zone, persisting, the audio, the figures' place, whether a game runs, the sync's send and the
//! desktop's own asks. The pages' topics go out through the [`publisher::Publisher`] into the platform's sinks.
//!
//! The areas, each an `impl Desk` block in its own file with its state a field of [`Desk`]: the pump and its deadlines
//! (`pump.rs`, `pump/`: topic `pump`), Pump home (`home.rs`, `streak.rs`: `home`, `streak`), Hero (`hero.rs`: `hero`,
//! `hero.charts`), the Book (`book.rs`: `book`, `book.words`, `book.correct`), the Saga and the Log (`log.rs`,
//! `saga.rs`, `history.rs`: `log`), Your setup and its places (`menu_pages/setup.rs`, `places.rs`: `menu.setup`,
//! `places`), the first five minutes (`menu_pages/welcome.rs`: `welcome`), the pump's cue policy with the Sound and
//! voice page's switches (`audio.rs`: `sound.*` intents) and the sync with Email, Restore code and Link a device
//! (`sync.rs`: `menu.sync`, `menu.email`, `menu.restore`, `menu.linkdevice`; `sync.*`, `email.*`, `restore.*`). The addon's own engine answers through one ordered door
//! (`engine.rs`); what one area asks of another goes through its seam (`seams.rs`); the contract's files every area
//! reads are built in once (`facts.rs`); the settings every area keeps are `settings.rs`'s. An intent (`{"type":
//! "<area>.<what>", ...}`) reaches its area through [`Desk::intent`].

#![forbid(unsafe_code)]

pub mod audio;
pub mod book;
pub mod engine;
pub mod facts;
pub mod figures;
pub mod hero;
pub mod history;
pub mod home;
pub mod log;
pub mod menu_pages;
pub mod places;
pub mod platform;
pub mod publisher;
pub mod pump;
pub mod qr;
pub mod recent;
pub mod saga;
pub mod seams;
pub mod settings;
pub mod streak;
pub mod sync;

use questpump_core::body::Kept;
use questpump_core::proof::Book;
use questpump_core::protocol::{Command, Line};
use questpump_core::session::{Effects, Now, People, Session};
use serde_json::Value;

pub use platform::Platform;
use publisher::Publisher;

/// The desk on its platform: the platform, the pages' publisher, the stores and the settings every area reads, the
/// engine's door and each area's state between the platform's calls.
pub struct Desk<P: Platform> {
	/// The platform the desk runs on.
	pub p: P,
	/// The pages' topics, shared with the platform's own topics (the desktop app's frame and tray).
	pub publisher: Publisher,
	/// The rows every device keeps and their effects (`questpump_core::session`).
	pub session: Session,
	/// The settings (`settings.json` on the computer).
	pub settings: settings::Settings,
	/// The body every app device shares (`body.json`).
	pub body: Kept,
	/// The addon's own engine behind its ordered door.
	pub engine: engine::Door<P>,
	/// Pump home's and Hero's state between looks.
	pub home: home::Home,
	/// Your setup's and the first five minutes' state.
	pub menu: menu_pages::MenuPages,
	/// The Book and the Log.
	pub book_log: book::BookLog,
	/// The pump on this computer.
	pub pumps: pump::Pumps,
	/// The sync between the player's devices, beside the account in the settings (slice 60).
	pub sync: sync::SyncState,
}

impl<P: Platform> Desk<P> {
	/// The desk on platform `p`, publishing through `publisher`, with the stores, the settings and the body read and the
	/// engine's door started.
	pub fn new(
		p: P,
		publisher: Publisher,
		session: Session,
		settings: settings::Settings,
		body: Kept,
		engine: engine::Door<P>,
	) -> Self {
		let zone = p.zone();
		Self {
			p,
			publisher,
			session,
			settings,
			body,
			engine,
			home: home::Home::new(zone),
			menu: menu_pages::MenuPages::default(),
			book_log: book::BookLog::default(),
			pumps: pump::Pumps::default(),
			sync: sync::SyncState::default(),
		}
	}
}

impl<P: Platform> Desk<P> {
	/// Now on the platform's clock, as the session dates what it keeps (unix seconds and RFC 3339 to the millisecond).
	pub fn now(&self) -> Now {
		let ms = self.p.now_ms();
		let format =
			time::macros::format_description!("[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:3]Z");
		let rfc3339 = time::OffsetDateTime::from_unix_timestamp_nanos(i128::from(ms) * 1_000_000)
			.ok()
			.and_then(|t| t.format(&format).ok())
			.unwrap_or_default();
		Now { unix: ms.div_euclid(1_000), rfc3339 }
	}

	/// The settings changed: the platform keeps them.
	pub fn save_settings(&self) {
		match serde_json::to_value(&self.settings) {
			Ok(file) => self.p.save_settings(file),
			Err(e) => ::log::warn!("{}: {e}", settings::FILE),
		}
	}

	/// The body changed (a phone's change, this computer's, the game's): the platform keeps `body.json`, and the
	/// desktop app's mirror carries it soon, whose `body` the game takes at its next load (one body everywhere,
	/// 2026-10-09).
	pub fn save_body(&mut self) {
		match self.body.to_json() {
			Ok(bytes) => self.p.save_file(Kept::FILE, bytes),
			Err(e) => ::log::warn!("{}: {e}", Kept::FILE),
		}
		self.p.body_changed();
	}

	/// The effects of rows the session took (`Effects`), wherever they came from: a pump the game offered on the strip
	/// ends the pump here (one pump at a time); the platform takes the lines, the commands and the files it keeps; the
	/// game's body joins the body every device shares; the people and the proof book are kept; then the rows reach the
	/// engine, Pump home, the Book and the Log, the pump's gates and the first five minutes, and the sync looks.
	pub fn apply(&mut self, mut fx: Effects) {
		// rows or characters the stores took: the sync's next push looks (slice 60)
		let stores_changed = fx.people_changed || fx.lines.iter().any(|l| matches!(l, Line::Row(_)));
		// a pump the game offered on the strip ends the pump on this computer (one pump at a time)
		if fx.commands.iter().any(|c| matches!(c, Command::PumpOffered { .. })) {
			self.pump_game_offered();
		}
		let game_body = fx.game_body.take();
		let (people_changed, proof_changed) = (fx.people_changed, fx.proof_changed);
		self.p.effects(fx);
		// the game's body (its Body switches and its Exercises page's list), repainted every few seconds and in every
		// file taken in: a change the player made in the game, within the profile the strip paints or the file's newest
		// character uses, joins the body every device shares, at the game's own time of it (else the platform's, the
		// file's for a file)
		if let Some(game) = &game_body {
			let at = game.at.unwrap_or_else(|| self.p.now_ms());
			if self.body.game(&game.profile, &game.seen, at) {
				::log::info!("body: the game's body changed");
				self.save_body();
				self.profile_changed();
				self.publish_setup();
				self.publish_welcome();
			}
		}
		if people_changed {
			let mut bytes = Vec::new();
			match ciborium::into_writer(self.session.people(), &mut bytes) {
				Ok(()) => self.p.save_file(People::FILE, bytes),
				Err(e) => ::log::warn!("people.cbor: {e}"),
			}
		}
		if proof_changed {
			match self.session.book().to_json() {
				Ok(bytes) => self.p.save_file(Book::FILE, bytes),
				Err(e) => ::log::warn!("{}: {e}", Book::FILE),
			}
		}
		// the rows the stores took from elsewhere reach the engine, in the order they came
		self.engine_rows(people_changed);
		self.home_rows();
		self.book_log_rows(people_changed);
		// the lock and the owed moves the pump's gates read, and its pain flags asked again when the stores took rows
		self.pump_refresh();
		// what the stores now hold: a player with progress is never new
		self.welcome_found();
		if stores_changed {
			self.sync_changed();
		}
	}
}

/// An intent from the page: its type (`<area>.<what>`) and its other fields.
#[derive(Debug)]
pub struct Intent {
	kind: String,
	args: serde_json::Map<String, Value>,
}

impl Intent {
	/// Reads `{"type": "<area>.<what>", ...}`.
	///
	/// # Errors
	/// The value is no object, or its type is no `<area>.<what>`.
	pub fn from_value(value: Value) -> Result<Self, String> {
		let Value::Object(mut args) = value else { return Err("an intent is an object".to_owned()) };
		let Some(Value::String(kind)) = args.remove("type") else { return Err("an intent names its type".to_owned()) };
		if !kind.split_once('.').is_some_and(|(area, what)| !area.is_empty() && !what.is_empty()) {
			return Err(format!("{kind}: an intent's type is <area>.<what>"));
		}
		Ok(Self { kind, args })
	}

	/// The area that handles it (`pump` of `pump.quick`).
	#[must_use]
	pub fn area(&self) -> &str {
		self.kind.split_once('.').map_or("", |(area, _)| area)
	}

	#[must_use]
	pub fn kind(&self) -> &str {
		&self.kind
	}

	/// A field's value.
	#[must_use]
	pub fn arg(&self, name: &str) -> Option<&Value> {
		self.args.get(name)
	}

	/// An intent this area has no handler for.
	#[must_use]
	pub fn unknown(&self) -> String {
		format!("{}: no such intent", self.kind) // copy-lint: not shown (an intent's error)
	}
}

impl<P: Platform> Desk<P> {
	/// An intent of the desk's areas to its handler (one line per area); an area the desk does not hold answers as
	/// unknown, for the platform's own areas to take.
	///
	/// # Errors
	/// No area takes it, or its area refused it (the reason).
	pub fn intent(&mut self, intent: &Intent) -> Result<(), String> {
		match intent.area() {
			"home" => self.home_intent(intent),
			"hero" => self.hero_intent(intent),
			"setup" => self.setup_intent(intent),
			"places" => self.places_intent(intent),
			"expedition" => self.expedition_intent(intent),
			"welcome" => self.welcome_intent(intent),
			"book" | "share" => self.book_intent(intent),
			"log" => self.log_intent(intent),
			"pump" => self.pump_intent(intent),
			"sound" => self.sound_intent(intent),
			"sync" => self.sync_intent(intent),
			"email" => self.email_intent(intent),
			"restore" => self.restore_intent(intent),
			_ => Err(intent.unknown()),
		}
	}

	/// Whether an intent's area is one of the desk's (`Desk::intent`).
	#[must_use]
	pub fn takes(area: &str) -> bool {
		matches!(
			area,
			"home"
				| "hero" | "setup"
				| "places" | "expedition"
				| "welcome" | "book"
				| "share" | "log"
				| "pump" | "sound"
				| "sync" | "email"
				| "restore"
		)
	}
}
