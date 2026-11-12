//! What the desk asks of the platform it runs on (docs/plan.md slice 63, "One desk core"): the desktop app
//! (`helper/src-tauri`, its `desk::shell::Shell`) and the browser (`questpump-web`) each implement [`Platform`], and the
//! desk holds its platform (`Desk::p`) and calls it, never the other way round, so the desk never reaches a window, a
//! file, a socket, a speaker or a game itself.
//!
//! The groups: the device the desk's rows are; the clock and the time zone; persisting (the settings, a file of the
//! data folder, the effects of rows the session took beyond the desk's own); the audio's play and stop, the screen
//! reader's live region and the guide's takes; the figures' place; whether a game runs and asks the pumps; the sync's
//! calls, its live socket and what it hands back (`crate::sync`); and the asks only the desktop app answers (the match
//! offers, the phones, the reminders, the window), each with a default that answers as a platform without them does, so
//! the browser implements what it has and nothing else. The pages' topics go out through the desk's
//! [`crate::publisher::Publisher`], whose [`crate::publisher::Sink`] is the platform's half of publishing; the pump's
//! next wake is the desk's own answer (`Desk::wake`), which the platform asks after each call and sleeps until.

use std::path::PathBuf;

use questpump_core::offers::{Offer, Pause, Tunables};
use questpump_core::protocol::Source;
use questpump_core::session::Effects;
use questpump_core::sync::LinkCode;
use questpump_core::work::Hold;
use serde::Serialize;
use serde_json::Value;

use crate::audio::Channel;
use crate::home::Board;
use crate::saga::ReelTune;
use crate::settings::{GamesSettings, SoundSettings};
use crate::sync::{Call, Then};
use crate::work::WorkView;

/// The engine's state, the `shell` topic's `engine` (`engine.rs`): a fold of every row runs (at the start, after a
/// reset), the engine answers, or it did not start or fold (the log says why).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EngineState {
	#[default]
	Busy,
	Ready,
	Failed,
}

/// The engine's last fold, for the report for help: its rows, how long it took and when it landed (unix ms).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fold {
	pub rows: usize,
	pub ms: u64,
	pub at: i64,
}

/// The game QuestPump knows on this computer.
#[derive(Serialize, Debug, Clone, Copy, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FoundGame {
	/// A client whose AddOns hold QuestPump runs (`Platform::game`'s `with_addon`), its squares read or not yet.
	Running,
	/// The install the desktop app follows holds QuestPump.
	Found,
	/// The game is on this computer without QuestPump yet (or a client without it runs): the addon comes from its page
	/// first.
	Installed,
	/// No game is known on this computer (and none ever in a browser).
	#[default]
	#[serde(rename = "none")]
	Unfound,
}

/// The game as the platform sees it now (`Platform::game`): a browser sees none.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[expect(
	clippy::struct_excessive_bools,
	reason = "the game's marks as the platform sees them, each a yes or no of its own"
)]
pub struct Game {
	/// A World of Warcraft client runs, with QuestPump or not (Pump home's recent games mark it Playing).
	pub client: bool,
	/// A World of Warcraft client runs whose AddOns hold QuestPump, by its folder alone: the game asks the moves owed,
	/// so no catch-up starts and no payment goes on a guess.
	pub with_addon: bool,
	/// That client's strip was never read and stayed out of sight for a minute while the screen is read: the app pumps
	/// start meanwhile (they neither pay nor owe).
	pub unseen: bool,
	/// An install with QuestPump is followed: its next load reads the mirror (a felt-unwell stop says a `/reload`
	/// brings the lock to the game).
	pub followed: bool,
	/// The client's squares are read now (the capture's dot is live): the game's sheet on the strip is the newest.
	pub live: bool,
	/// The game QuestPump knows.
	pub found: FoundGame,
}

impl Game {
	/// The game asks the pumps as far as the screen tells: no app pump and no match offer starts.
	#[must_use]
	pub fn asks_pumps(self) -> bool {
		self.with_addon && !self.unseen
	}
}

/// What Pump home's status Chip asks the player to fix, in this order; none while all is well, when no Chip shows
/// (`Platform::home_shell`).
#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Fix {
	/// This computer's state needs the player: the Chip says it in the tray's words with the tray's colour and opens
	/// This computer.
	Computer,
	/// The progress could not be saved into the game folder: it opens This computer, whose progress line says why.
	Save,
	/// Windows: the firewall keeps the paired phone out (the Phones page's firewall card): it opens Phones.
	Firewall,
}

/// A setup step the player has started, Pump home's foot's primary before any pump (`Platform::home_shell`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
	/// macOS: allow screen reading (`asked`: macOS asked before, the button opens System Settings).
	Grant { asked: bool },
	/// Windows: the firewall's one-time step that lets the phone in.
	Firewall,
}

/// What the platform's own state gives Pump home (`Platform::home_shell`): its Chip's fix, a setup step started and
/// the game QuestPump found as This computer shows it. A browser has none of them.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HomeShell {
	pub fix: Option<Fix>,
	pub step: Option<Step>,
	pub found: FoundGame,
}

/// A game the Games tab shows as a tile (Pump home's recent games show only those): its key (a named game's
/// contract/links.json key, a found or picked program's own), a found or picked game's own name (none for a named
/// game, which the page names) and its icon's asset URL while its file stands.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GameTile {
	pub key: String,
	pub name: Option<String>,
	pub icon: Option<String>,
}

/// A move's guide as the install the app follows holds its takes (`Platform::guide_takes`): How's takes then Tips', in
/// the order the voice reads them, and the take Tips begin at, after `callout_gap_s` (none where Tips has no take or
/// How has none before it; Speech.lua `GuideTakes`'s `starts`). No take: no guide to read.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct GuideTakes {
	pub takes: Vec<GuideTake>,
	pub tips: Option<usize>,
}

/// One take of a move's guide: its file and its length in seconds.
#[derive(Debug, Clone, PartialEq)]
pub struct GuideTake {
	pub file: PathBuf,
	pub seconds: f64,
}

/// What the desk asks of the platform it runs on. The items without a default every platform answers; those with one
/// answer as the desktop app does, which a browser keeps but where it says otherwise (its sign-out's wipe, its request
/// code's link, its link requests only while the phone apps are in the stores, its pumps' `BY`).
pub trait Platform: 'static {
	// The device the desk's rows are.

	/// The device whose bit the rows of the pumps the desk runs carry (`questpump_core::codes::flags`): beside the app
	/// bit on an app pump's rows and the engine's (`Session::keep_computer`, `engine.init`), alone on a catch-up's and a
	/// bank's (`Session::catch_up`, `Session::bank`). The desktop app's is `Source::Computer`.
	const DEVICE: Source;

	/// The start of the id of every pump the desk runs (`pump::new_id`, `pump::ran_here`): the desktop app's is `d-`.
	const PUMP_IDS: &'static str;

	/// Signing out, or being removed from the account, forgets the account here and every row with it
	/// (`signed_out`): a browser, which the next person at it may use. Its Sign out sends what waits first and asks
	/// before leaving rows unsent. The desktop app keeps its rows and the account's R.
	const WIPES_ON_SIGN_OUT: bool = false;

	/// The pumps the desk banks are the account's unfinished list (`questpump_core::session::bank_quest`, slice 67): a
	/// device without a computer of the player's, the browser, whose banks every device folds. The desktop app's banks
	/// are its own (questID 0), as before.
	const ACCOUNT_BANKS: bool = false;

	/// Who ran the desk's own pumps, in their offered pump rows' answer bits (`questpump_core::codes::by`, Core.lua
	/// `ns.Enum.by`; `Desk::pump_keep`): a browser's `by::BROWSER`, so every Saga says "in a browser" (docs/plan.md
	/// slice 63). The desktop app's rows carry none (0), as before.
	const BY: u8 = 0;

	// The clock and the time zone.

	/// Now, in unix milliseconds.
	fn now_ms(&self) -> i64;

	/// The time zone: its offset from UTC at a unix second, in seconds east, as the zone gives it for that instant
	/// (summer time included): the game day's turn, the Saga's days, the places' times (`questpump_core::Zone`).
	fn zone(&self) -> fn(i64) -> i64;

	// Persisting.

	/// The settings changed: keep them (the desktop app's `settings.json`, through its secrets' keeper).
	fn save_settings(&self, file: Value);

	/// A file of the data folder changed (`people.cbor`, the proof book, `body.json`): keep its bytes.
	fn save_file(&self, name: &'static str, bytes: Vec<u8>);

	/// The body every device shares changed (kept in `body.json` before): the desktop app's mirror carries it soon, for
	/// the game's next load.
	fn body_changed(&mut self) {}

	/// The effects of rows the session took (`questpump_core::session::Effects`) beyond the desk's own, which the desk
	/// handled already (the game's body, the people's and the proof book's files): the lines to the socket, the
	/// devices' commands, the mirror, the feedback, the counts.
	fn effects(&mut self, fx: Effects);

	// The sync (slice 60; `crate::sync`).

	/// The device's kind as the sync server names it (contract/sync-api.yaml `Kind`): the desktop app's `desktop`.
	const SYNC_KIND: &'static str;

	/// This device's name, the device batch's, which the other devices' Devices lists show.
	fn device_name(&self) -> String;

	/// One call to the sync's routes, sent off the desk: its answer, or why none came, goes back as `then(answer)`
	/// through the platform's loop to `Desk::on_sync`, never inside this call. The answer's body is read up to
	/// `crate::sync::ANSWER_MOST`.
	fn sync_call(&mut self, call: Call, then: Then);

	/// The live socket opened with the device `token` (`crate::sync::live_hello` its first message): each message's
	/// nudge goes back as `crate::sync::LiveHeard::message` reads it, and its end as `News::LiveEnded`, each with
	/// `generation`, through the platform's loop to `Desk::on_sync`.
	fn live_open(&mut self, token: String, generation: u64);

	/// The live socket closed by the desk: no end comes back for it.
	fn live_close(&mut self);

	/// Whether the sync's calls can go at all (the desktop app's HTTPS client built).
	fn sync_reachable(&self) -> bool {
		true
	}

	/// This run's secrets will not reach the next start (the desktop app's `RunSecrets::Unread`): no account is joined,
	/// made or changed, since its R and token would be gone at the next start.
	fn secrets_unread(&self) -> bool {
		false
	}

	/// The account's Pro token and status as the server answered (`GET pro`, slice 64).
	fn account_pro(&mut self, token: Option<String>, status: Option<String>) {
		let _ = (token, status);
	}

	/// Manage Pro's address as the server answered (none: no answer, or one that is no HTTPS address).
	fn portal_answered(&mut self, url: Option<String>) {
		let _ = url;
	}

	/// Sync's switch moved: Pro's page shows it.
	fn sync_switched(&mut self) {}

	/// Out of the account on a platform that wipes (`WIPES_ON_SIGN_OUT`), the account already forgotten and saved:
	/// signed out (`removed` false) or removed from it (`removed`: another device removed it, or a new key left it
	/// out), the browser deletes its data and starts again as a guest's, saying why when it was removed. The desktop
	/// app keeps every row and the account's R, and is never called.
	fn signed_out(&mut self, _removed: bool) {}

	/// A link request's code as this device shows it (a QR's text): the code itself, or the browser's link a phone's
	/// camera opens in the phone app.
	fn request_code(&self, code: &LinkCode) -> String {
		code.to_string()
	}

	/// Whether a device without R shows a link request's code at all (the browser's only while the phone apps are in
	/// the stores, which alone answer one).
	fn link_requests(&self) -> bool {
		true
	}

	// The audio.

	/// A sound by its `sounds.json` id plays now (the switches already said yes).
	fn play_sound(&self, id: &str);

	/// A voice take by its `VoiceClips.lua` id plays now, stopping the line before it (the switches already said yes).
	fn play_take(&self, id: &str);

	/// The line playing stops (Speech.lua's `StopSound`).
	fn hush(&self);

	/// The output stays open between sounds while a pump runs (`on`), or may fall idle again.
	fn hold_output(&self, on: bool);

	/// The Sound and voice page's switches, levels or pick changed (`Desk::sound_intent`, kept already): play at them,
	/// and the page's `menu.sound` topic shows them.
	fn sound_changed(&mut self, sound: &SoundSettings);

	/// The page's test plays: the bell, then the line (`Channel::Master`), or a level's part alone.
	fn sound_test(&self, channel: Channel);

	/// A screen reader runs and the desk's window has the focus: a pump's line goes to the live region (`announce`)
	/// instead of the voice.
	fn reader_hears(&self) -> bool {
		false
	}

	/// A line for the page's one polite live region, while the screen reader hears it (`reader_hears`).
	fn announce(&mut self, text: String) {
		let _ = text;
	}

	/// The takes of move `key`'s guide (the implement's, `gear`, where it has its own) from the install the app follows;
	/// none without one.
	fn guide_takes(&mut self, key: &str, gear: Option<&str>) -> GuideTakes {
		let _ = (key, gear);
		GuideTakes::default()
	}

	/// A take of a move's guide plays on a player of its own, its take before stopping.
	fn play_guide(&self, take: &GuideTake) {
		let _ = take;
	}

	/// The guide's take stops.
	fn stop_guide(&self) {}

	// The figures' place.

	/// The whole set of the move figures' films is here (the Saga's day reel leaves out the moves it is not).
	fn films(&self) -> bool {
		false
	}

	/// The day reel's timing (the Share window's maker): none where no reel is made.
	fn reel_tune(&self) -> Option<ReelTune> {
		None
	}

	// Whether a game runs.

	/// The game as the platform sees it now.
	fn game(&self) -> Game {
		Game::default()
	}

	/// The stores hold what this platform keeps: the install the desktop app follows was taken in, or none is known.
	/// Nothing is decided before (the first five minutes wait).
	fn stores_read(&self) -> bool {
		true
	}

	/// The other games running now (the games' look's keys, any of a named game's own programs): none in a browser.
	fn games_open(&self) -> Vec<String> {
		Vec::new()
	}

	/// The Games tab's tiles as the games' settings list them: none in a browser.
	fn game_tiles(&self, games: &GamesSettings) -> Vec<GameTile> {
		let _ = games;
		Vec::new()
	}

	/// World of Warcraft's icon's asset URL, made from its client's program: none in a browser.
	fn wow_icon(&self) -> Option<String> {
		None
	}

	/// Whether Pump home may show World of Warcraft among the recent games (a client ran, or a move of the game's own
	/// is kept): the games' icons make its icon then.
	fn recent_wow(&mut self, known: bool) {
		let _ = known;
	}

	// The match offers (the desktop app's games on the computer, slices 48 and 57): none in a browser.

	/// The match offer that stands.
	fn offer(&self) -> Option<&Offer> {
		None
	}

	/// The match offers' tunables (the games' table).
	fn tunables(&self) -> Option<&Tunables> {
		None
	}

	/// The name of the first game marked `aim` with its switch on (`games`) that runs now: a match's pump rests the
	/// arms.
	fn arms_game(&self, games: &GamesSettings) -> Option<String> {
		let _ = games;
		None
	}

	/// The seconds sat at unix second `at` on the sitting clock without the offers, the newest performed move at
	/// `last_move`.
	fn seated_s(&self, at: i64, last_move: Option<i64>) -> i64 {
		let _ = (at, last_move);
		0
	}

	/// The offers' pause a pump's Pause menu names (`kind`: `none`, `hour`, `four_hours`, `night`, `game`, `resumed`)
	/// as it would start now, or why none does (no offers here, no watched game running for `game`, no such pause).
	///
	/// # Errors
	/// Why no such pause starts.
	fn offers_pause(&self, kind: &str, games: &GamesSettings) -> Result<Pause, String> {
		let _ = (kind, games);
		Err("no offers on this computer".to_owned()) // copy-lint: not shown (an intent's error)
	}

	/// The offers' pause changed (kept in the settings): the tray and the Games tab show it.
	fn offers_paused(&mut self) {}

	/// A pump's start from the match offer `offer` went through: its card goes, and it no longer goes out to the
	/// devices.
	fn offer_started(&mut self, offer: &str) {
		let _ = offer;
	}

	/// The pump from the match offer `offer` started: the offer is taken.
	fn offer_taken(&mut self, offer: &str) {
		let _ = offer;
	}

	// The shell around the pages.

	/// The engine's state and its last fold (none: the last stays), for the `shell` topic and the report for help.
	fn engine_state(&mut self, state: EngineState, fold: Option<Fold>) {
		let _ = (state, fold);
	}

	/// What Pump home's Chip, its foot's setup step and its next card read of the platform's own state, a phone
	/// paired or not (`paired`).
	fn home_shell(&self, paired: bool) -> HomeShell {
		let _ = paired;
		HomeShell::default()
	}

	/// What QuestPump found of the game goes into the platform's own view (the desktop app's This computer, its tray's
	/// first line): at the loop's look.
	fn show_found(&mut self) {}

	/// The felt-unwell lock holds (`on`) or not: its start cancels a day reel being made (slice 45).
	fn reel_locked(&mut self, on: bool) {
		let _ = on;
	}

	/// Whether a pump is on screen, at every publish of the pump (the desktop app's close guard asks "Leave the
	/// pump?").
	fn guard(&mut self, running: bool) {
		let _ = running;
	}

	/// A pump came on screen (`running`) or left it: the computer held awake meanwhile, a day reel being made
	/// cancelled at its start (slice 45), Backup's word on it.
	fn pump_running(&mut self, running: bool) {
		let _ = running;
	}

	/// The tray's pump items changed: a pump on, Quick pump can start, Catch up can start.
	fn tray_pumps(&mut self, items: (bool, bool, bool)) {
		let _ = items;
	}

	/// The desk's window opens, for a pump asked from outside it (the tray, the card).
	///
	/// # Errors
	/// The window could not be asked to open.
	fn open_window(&mut self) -> Result<(), String> {
		Ok(())
	}

	/// The desk's window stands (a pump made for it shows only there).
	fn window_open(&self) -> bool {
		true
	}

	/// The window's own full screen on or off, as it is not already (Big view at a pump's start and end).
	fn full_screen(&mut self, on: bool) {
		let _ = on;
	}

	/// Big view's own toggle: the window's full screen, and the platform's word on it.
	///
	/// # Errors
	/// The window refused it.
	fn view_full(&mut self, on: bool) -> Result<(), String> {
		let _ = on;
		Ok(())
	}

	/// "Leave the pump?" was answered Leave once the pump left the screen: the window closes, or with `quit` the app
	/// quits as the ask said.
	///
	/// # Errors
	/// The window could not be asked to close.
	fn leave(&mut self, quit: bool) -> Result<(), String> {
		let _ = quit;
		Ok(())
	}

	/// "Leave the pump?" was answered Stay.
	fn stay(&mut self) {}

	/// The streak's board as Pump home drew it, whether the game's sheet stands for an open day (`game`) and whether
	/// the felt-unwell lock holds (`resting`): the reminders' tray line.
	fn reminders_board(&mut self, board: Board, game: bool, resting: bool) {
		let _ = (board, game, resting);
	}

	/// A pump's Result was put away: "Remind me once a day?" may be due.
	fn reminder_offer_due(&mut self) {}

	/// A step's card opened in the Book's trees, with its Start's refused reason: the anonymous counts.
	fn tree_card(&mut self, refused: Option<&str>) {
		let _ = refused;
	}

	/// A Start from a step's card or a move's window composed its pump, by its kind: the anonymous counts.
	fn tree_start(&mut self, kind: &str) {
		let _ = kind;
	}

	/// Any verified Pro grant stands now, a phone's or the account's (`account`).
	fn pro_grants(&self, account: Option<&str>) -> bool {
		let _ = account;
		false
	}

	// Work mode (slice 67, `work.rs`): the device's presence and holds, and where its state shows outside the pages.

	/// The device's seconds without input at its last look (the sitting clock's `Sitting::idle_s`), from which work
	/// mode tells a due pump's moment and whether somebody is there for it, never its count; none where the platform
	/// reads none.
	fn idle_s(&self) -> Option<u32> {
		None
	}

	/// What on this device holds the work clock beside the desk's own holds (a pump open, quiet hours, the lock, a match
	/// offer, the game asking pumps): a call, presenting, full screen, a game in front. None where it reads none.
	fn work_hold(&self) -> Option<Hold> {
		None
	}

	/// Work mode's state changed (`WorkView`, the `work` topic's): the tray's line and ring.
	fn work_changed(&mut self, view: &WorkView) {
		let _ = view;
	}

	/// The work pump of the ask `ask` (`WorkView::ask`) was started by itself (`Desk::work_open`: its start accepted and
	/// the window asked to open, `open_window`, which brings the desktop app's to the front): the browser, whose tab
	/// cannot come to the front, tells its page, which shows a notification while the tab is hidden and that pump
	/// stands (`WorkView::pump`).
	fn work_opened(&mut self, ask: &str) {
		let _ = ask;
	}

	/// A work pump came on screen: the app in front before it noted, which its Result's Back to work brings back
	/// (`back_to_work`); whether one was noted. None where no other app comes to the front (the browser, whose Result
	/// keeps Continue).
	fn back_note(&mut self) -> bool {
		false
	}

	/// A work pump's Back to work: the app noted as it came (`back_note`) in front again, QuestPump's window left open
	/// behind it.
	fn back_to_work(&mut self) {}
}
