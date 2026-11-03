//! The desktop app QuestPump on this computer (docs/plan.md slice 35; docs/ui.md, "The desktop app"): its main window,
//! the macOS app menu, the tray, the window's place and the launch's reason, and the bridge between the page and the
//! loop. Rust owns state and time; the page (`helper/dist/`) draws and sends intents. State goes out as topics through
//! the publisher (`state.rs`): `shell` (the frame), `menu.computer`, `menu.phones`, `menu.devices` and `feedback` from
//! the view (`ui.rs`), and each area's own (`home`, `pump`, `hero`, `book`, `log`, `card`, its Menu pages). An intent
//! (`{"type": "<area>.<what>", ...}`) reaches the loop as a `DeskAsk` and its area's handler there, an `impl Helper`
//! block in the area's own file (the runtime's `Helper` and its fields are `pub(crate)` for them). The addon's own
//! engine answers through one ordered door (`engine.rs`), and what one area asks of another goes through its seam
//! (`seams.rs`), whose owner gives it its body; the contract's files every area reads (`contract/sheet.json`,
//! `library.json`, `moves.json`) are built in and parsed once in `facts.rs`. Sound and voice play from `audio.rs`
//! (topic `menu.sound`), and the move figures' table and files come from `figures.rs` (topic `figures`); the
//! day reel the Share window makes from `reel/` (topic `reel`, slice 45), the share cards' rarity line from
//! `standing.rs` (topic `standing`, slice 51); the reminders, on the card's window, from
//! `reminders.rs` (topic `menu.reminders`). The Menu's own
//! pages beside the shell's (Your setup, Backup, QuestPump's group) and the first five minutes are `menu_pages.rs`'s;
//! the crash reports' switch and page (topic `crash`, intents `crash.*`) are `runtime/crash.rs`'s, beside their outbox;
//! Usage statistics and the privacy page (topic `usage`, intents `usage.*` and `privacy.done`) `runtime/usage.rs`'s,
//! beside the anonymous counts; the sync's Sync card, Email, Restore code and Link a device (topics `menu.sync`,
//! `menu.email`, `menu.restore`, `menu.linkdevice`, intents `sync.*`, `email.*`, `restore.*`) `runtime/sync.rs`'s,
//! beside the sync itself (slice 60).

pub(crate) mod audio;
pub(crate) mod awake;
pub(crate) mod book;
pub(crate) mod card;
pub(crate) mod commands;
pub(crate) mod engine;
pub(crate) mod facts;
pub(crate) mod figures;
pub(crate) mod hero;
pub(crate) mod history;
pub(crate) mod home;
pub(crate) mod launch;
pub(crate) mod log;
#[cfg(target_os = "macos")]
pub(crate) mod menu;
pub(crate) mod menu_pages;
pub(crate) mod offers;
pub(crate) mod place;
pub(crate) mod places;
pub(crate) mod pump;
pub(crate) mod recent;
pub(crate) mod reel;
pub(crate) mod reminders;
pub(crate) mod saga;
pub(crate) mod seams;
pub(crate) mod standing;
pub(crate) mod state;
pub(crate) mod streak;
pub(crate) mod tray;
pub(crate) mod windows;

use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::time::Instant;

use serde_json::Value;
use tokio::sync::oneshot;

use crate::runtime::Helper;
use place::Place;
use state::Publisher;

/// The topics the view publishes (`ui::update`).
pub(crate) const SHELL: &str = "shell";
pub(crate) const COMPUTER: &str = "menu.computer";
pub(crate) const PHONES: &str = "menu.phones";
pub(crate) const DEVICES: &str = "menu.devices";
pub(crate) const FEEDBACK: &str = "feedback";

/// The desktop app's state the window's events, the commands and the loop share (managed by the app).
#[derive(Default)]
pub(crate) struct Desk {
	pub(crate) publisher: Publisher,
	/// A pump holds the main window (the pump sets it while one runs): its close and QuestPump's quit ask "Leave the
	/// pump?" first (`windows.rs`, `ui::quit`).
	pub(crate) guard: AtomicBool,
	/// The main window's place as it moves and resizes, kept in settings.json once it settles, when it closes and when
	/// QuestPump quits.
	pub(crate) place: Mutex<Option<Place>>,
	/// When the place last changed while not in settings.json yet: the loop writes it `windows::SETTLE` after
	/// (`Helper::place_settled`).
	pub(crate) moved: Mutex<Option<Instant>>,
	/// Crash reports wait for the player's answer (`runtime/crash.rs`): the next main window built opens on their page,
	/// once (`windows::open_main`).
	pub(crate) crash_page: AtomicBool,
	/// A switch of the privacy page is not answered (`runtime/usage.rs`): the next main window built opens on the page,
	/// once a run (after the crash reports' page when both wait).
	pub(crate) privacy_page: AtomicBool,
	/// The day reel shown and the job making it (`reel/`): the loop, the commands and the job share it.
	pub(crate) reels: reel::Reels,
}

impl Desk {
	/// The main window's place as it stands now.
	pub(crate) fn place(&self) -> Option<Place> {
		self.place.lock().ok().and_then(|p| p.clone())
	}
}

/// What the desktop app asks the loop, through `ui::WindowAsk::Desk`.
#[derive(Debug)]
pub(crate) enum DeskAsk {
	/// The page's intent, answered once its area's handler took it.
	Intent { intent: Intent, done: oneshot::Sender<Result<(), String>> },
	/// The macOS app menu's Zoom In (1), Zoom Out (-1) or Actual Size (0).
	#[cfg(target_os = "macos")]
	Zoom(i8),
	/// The main window closes: its place goes into settings.json at once.
	Place(Place),
	/// A backup file's dialog answered: the file written, or read for the loop to take in (`menu_pages::backup`).
	Backup(menu_pages::backup::Picked),
	/// The move figures' films landed after a download: the Saga's reels can show their moves now (`figures.rs`).
	Films,
	/// The Share window's order for a day reel (the `reel_make` command's body), answered with the reel's id once its
	/// job started (`reel/`).
	Reel { body: Vec<u8>, done: oneshot::Sender<Result<u64, String>> },
	/// The main window is gone: what its pages held on the loop goes with them (the Book's standing move pages, whose
	/// sets it carries for Correct a count, slice 44).
	Gone,
}

/// An intent from the page: its type (`<area>.<what>`) and its other fields.
#[derive(Debug)]
pub(crate) struct Intent {
	kind: String,
	args: serde_json::Map<String, Value>,
}

impl Intent {
	/// Reads `{"type": "<area>.<what>", ...}`.
	///
	/// # Errors
	/// The value is no object, or its type is no `<area>.<what>`.
	pub(crate) fn from_value(value: Value) -> Result<Self, String> {
		let Value::Object(mut args) = value else { return Err("an intent is an object".to_owned()) };
		let Some(Value::String(kind)) = args.remove("type") else { return Err("an intent names its type".to_owned()) };
		if !kind.split_once('.').is_some_and(|(area, what)| !area.is_empty() && !what.is_empty()) {
			return Err(format!("{kind}: an intent's type is <area>.<what>"));
		}
		Ok(Self { kind, args })
	}

	/// The area that handles it (`pump` of `pump.quick`).
	pub(crate) fn area(&self) -> &str {
		self.kind.split_once('.').map_or("", |(area, _)| area)
	}

	pub(crate) fn kind(&self) -> &str {
		&self.kind
	}

	/// A field's value.
	pub(crate) fn arg(&self, name: &str) -> Option<&Value> {
		self.args.get(name)
	}

	/// An intent this area has no handler for.
	pub(crate) fn unknown(&self) -> String {
		format!("{}: no such intent", self.kind) // copy-lint: not shown (an intent's error)
	}
}

impl Helper {
	/// The desktop app's asks: each intent to its area's handler.
	pub(crate) fn on_desk(&mut self, ask: DeskAsk) {
		match ask {
			DeskAsk::Intent { intent, done } => {
				let outcome = self.intent(&intent);
				if let Err(e) = &outcome {
					// the log crate's (this module's `log` is the Log's page)
					::log::warn!("desk: {e}");
				}
				let _ = done.send(outcome);
			}
			#[cfg(target_os = "macos")]
			DeskAsk::Zoom(step) => self.zoom(step),
			DeskAsk::Place(place) => self.keep_place(Some(place)),
			DeskAsk::Backup(picked) => self.on_backup(picked),
			DeskAsk::Films => self.log_publish(),
			DeskAsk::Gone => {
				self.book_pages_gone();
				self.sync_pages_gone();
			}
			DeskAsk::Reel { body, done } => {
				let outcome = self.reel_make(&body);
				if let Err(e) = &outcome {
					::log::warn!("reel: {e}");
				}
				let _ = done.send(outcome);
			}
		}
	}

	/// An intent to its area's handler (one line per area): the page's, the tray's, and one area's asking another's (the
	/// card's Start asks the pump's `pump.start`).
	pub(crate) fn intent(&mut self, intent: &Intent) -> Result<(), String> {
		match intent.area() {
			"view" => self.view_intent(intent),
			"card" | "games" => self.offers_intent(intent),
			"reminders" => self.reminders_intent(intent),
			"sound" => self.sound_intent(intent),
			"home" => self.home_intent(intent),
			"hero" => self.hero_intent(intent),
			"setup" => self.setup_intent(intent),
			"places" => self.places_intent(intent),
			"expedition" => self.expedition_intent(intent),
			"backup" => self.backup_intent(intent),
			"about" => self.about_intent(intent),
			"pro" => self.pro_intent(intent),
			"welcome" => self.welcome_intent(intent),
			"book" | "share" => self.book_intent(intent),
			"standing" => self.standing_intent(intent),
			"log" => self.log_intent(intent),
			"pump" => self.pump_intent(intent),
			"crash" => self.crash_intent(intent),
			"usage" | "privacy" => self.usage_intent(intent),
			"sync" => self.sync_intent(intent),
			"email" => self.email_intent(intent),
			"restore" => self.restore_intent(intent),
			_ => Err(intent.unknown()),
		}
	}
}
