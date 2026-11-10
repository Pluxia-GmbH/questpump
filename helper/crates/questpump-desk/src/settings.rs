//! The helper's own settings in its data folder (`settings.json`, written by the runtime's writer thread, and at once
//! by the start's crash sweep when it moved a mark): the pairing key, the install's id, the pairing record the mirror's
//! `pairing` block states, the paired phone's name, the display lag, the game folder it found or the player picked
//! last, whether the heart-rate strap and the walking pad are on, the games on the computer (slice 35: the picked
//! programs, the listener's port and token, the files written into the games' folders, whether a game's close opens
//! the pump, since slice 48 the aim games whose arms rest the player turned off, the fed games' ports the player
//! changed and the games the detector found, since slice 56 the games asked for, the games turned off and when the
//! Games tab was last opened), the computer's match offers (their pause and the latest game offer
//! on any device), its reminders (the streak reminder, the sitting cue, and the quiet hours and daily cap they alone
//! keep since slice 48, stored with the offers' as before), the sitting clock as the last quit kept it (slice 48) and
//! the games' runs and matches with it (2026-10-29, taken once by a start within `match_relaunch_s`), the main window's
//! place (with the mark that a place at the old first size went once), its size and Big view's full
//! screen, this computer's place for a pump, whether it was asked on it and whether it took Room to move's new default,
//! its sound and voice, the first five minutes and the last backup file (slice 35), the games played here with their
//! pumps (Pump home's recent games, 2026-10-27), Send crash reports with the newest of the system's crash reports and
//! of the panics handled (contract/crash-api.yaml), and Usage statistics (contract/telemetry-api.yaml), each switch's
//! answer with its moment and the words' version, and the sync's account (slice 60: Sync's switch, the account id and
//! R, this computer's token in it and how far it synced). The pairing key, R, the token, a rekey's R' and this
//! computer's own sync key live in the system's secret store, never in the file once it holds them (`secrets::Keeper`,
//! which every save goes through). A value that does not read takes its default alone (`Settings::read`), so only a
//! pairing key that neither the file nor the store holds starts every pairing again. Whether it starts at login is the
//! system's own record (`autostart.rs`), never a copy here: the settings keep only which program it was turned on for
//! and whether the player turned it off in the window.

use std::collections::BTreeMap;
use std::path::PathBuf;

use questpump_core::body::PlaceSetup;
use questpump_core::crash::Handled;
use questpump_core::mirror::Pairing;
use questpump_core::offers::{Pause, Tunables};
use questpump_core::telemetry::COPY_VERSION;
use serde::{Deserialize, Serialize};

use crate::menu_pages::welcome::Step;
use crate::seams::Came;

/// The settings' file in the data folder.
pub const FILE: &str = "settings.json";

#[derive(Serialize, Deserialize, Debug, Clone)]
#[expect(clippy::struct_excessive_bools, reason = "the settings file's switches, read as they are")]
pub struct Settings {
	/// The 32-byte pairing key, base64url: shown in the pair link's QR code, never sent anywhere, kept in the secret
	/// store (`secrets`; in the file only while the store refuses it).
	#[serde(with = "bytes_text")]
	pub key: [u8; 32],
	/// This install's id, 16 random bytes, base64url in the file: in every pair link, so a phone tells its own helper
	/// from another one. Made at the first start (or read from a file of an earlier build, which has none: saved at
	/// that start) and never changed by a new pairing code.
	#[serde(default = "new_hid", with = "bytes_text")]
	pub hid: [u8; 16],
	pub pairing: Pairing,
	/// The paired phone's name, for the window.
	#[serde(default)]
	pub phone_name: Option<String>,
	/// The camera test's correction, in milliseconds (`{"cmd":"display_lag","ms":…}` on the socket, how much later the
	/// wrist's "go" came than the screen's); 0 until it is learnt.
	#[serde(default)]
	pub display_lag_ms: i32,
	/// The game's `Interface/AddOns` folder the mirror goes into, once found or picked.
	pub addons: Option<PathBuf>,
	/// The heart-rate strap is turned on (the window's button): the helper reads it from its start on.
	#[serde(default)]
	pub strap: bool,
	/// The walking pad is turned on (the window's button): the helper reads it from its start on.
	#[serde(default)]
	pub pad: bool,
	/// The Phones page's Bluetooth link switch (slice 43), on by default: while it is on and a phone is paired or the
	/// Phones page shows the code, the computer serves the phones over Bluetooth too (`blelink`).
	#[serde(default = "on")]
	pub ble_link: bool,
	/// The program start at login was last turned on for: a new one (a new build, the helper moved to Applications)
	/// is turned on again at its start.
	#[serde(default)]
	pub autostart_for: Option<PathBuf>,
	/// The player turned start at login off in the window: the helper leaves it off until they turn it on there.
	#[serde(default)]
	pub autostart_off: bool,
	/// The games on the computer.
	#[serde(default)]
	pub games: GamesSettings,
	/// The computer's match offers.
	#[serde(default)]
	pub offers: OfferSettings,
	/// The computer's reminders (`desk::reminders`).
	#[serde(default)]
	pub reminders: ReminderSettings,
	/// The sitting clock as the last quit kept it (`runtime/sitting.rs`, slice 48); none before the first quit, or when
	/// the clock had not started.
	#[serde(default)]
	pub sitting: Option<KeptClock>,
	/// The games' runs and matches as the last quit kept them (`games::Watcher::keep`), taken by the next start within
	/// `match_relaunch_s` (`Games::resume_runs`, 2026-10-29); none before the first quit, and none once a start took
	/// them.
	#[serde(default)]
	pub runs: Option<questpump_core::games::KeptRuns>,
	/// The main window's place when it last settled or closed (`desk::place`).
	#[serde(default)]
	pub window: Option<Place>,
	/// The place has been through the first window's growth (2026-10-15, David: "how to start the app bigger and not in
	/// phone size"): a file written before lacks this mark, and a place kept there at exactly the old first size,
	/// 520 × 760, was never resized (`place::never_resized`), so `Settings::load` drops it once and the window opens at
	/// the new first size; a player who picks 520 × 760 afterwards keeps it. Off in `Settings::default`, as `space_on`.
	#[serde(default)]
	pub window_grown: bool,
	/// The page's size, the player's step of it (Cmd/Ctrl + and -, the macOS View menu, the Menu's Size), 1 for 100 %,
	/// the default: the window shows the page at `desk::windows::BASE` times it. A file of before 2026-10-15 kept the
	/// zoom factor itself, read as the step, so the step a player chose stays theirs.
	#[serde(default = "own_size")]
	pub zoom: f64,
	/// Big view fills the screen (the window's own full screen), as the player last chose.
	#[serde(default)]
	pub fullscreen: bool,
	/// This computer's own setup from before the places (slice 51): Home's equipment and switches until a device or the
	/// game stamps the body's `home` (`desk::places`), a change of Home made here written into both.
	#[serde(default)]
	pub place: PumpPlace,
	/// The place the player last picked on this computer and when (2026-11-03, `desk::places`): it is the place in use until
	/// the schedule's next start or end or the plan's next change (`questpump_core::places::in_use`); none before the first
	/// pick. A file of before kept the place in use's id alone, read as a pick made as it loads (`places.json`: an older
	/// device's place in use becomes a pick made at that moment). Never in the body.
	#[serde(default, deserialize_with = "pick_or_id")]
	pub place_in_use: Option<PlacePick>,
	/// The Quick pump's length and the longest timed pump (slice 51, `desk::pump::expedition`).
	#[serde(default)]
	pub expedition: ExpeditionSettings,
	/// The desktop app's sound and voice and the output they play on (`desk::audio`).
	#[serde(default)]
	pub sound: SoundSettings,
	/// The first five minutes and the backup file (`desk::menu_pages`).
	#[serde(default)]
	pub desk: DeskSettings,
	/// The moves the player marked favourites in the Book on this computer before the favourites were shared
	/// (2026-10-09): the engine's profile takes them while no device changed the body's (`Desk::favourites`), and the
	/// Book's next Favourite moves them into the body and clears them here.
	#[serde(default)]
	pub favourites: Vec<u16>,
	/// The title the player chose on Hero for the app's own sheet on this computer before the title was shared
	/// (2026-10-09; contract/sheet.json's `titles`' key): the engine's profile takes it while no device chose the
	/// body's (`Desk::title`), and Hero's next choice moves it into the body and clears it here. None: no title.
	#[serde(default)]
	pub title: Option<String>,
	/// The style the share card was last drawn in on this computer (`desk::book`), drawn as Parchment without Pro.
	#[serde(default)]
	pub share_style: ShareStyle,
	/// The day's card and the format the Share window last drew it in on this computer (`desk::book`; the Saga).
	#[serde(default)]
	pub share_card: ShareCard,
	#[serde(default)]
	pub share_format: ShareFormat,
	/// The day reel's Sound as the Share window last set it on this computer (slice 45; `desk::book`): on by default.
	#[serde(default = "on")]
	pub share_sound: bool,
	/// Where this computer's pumps happen was asked on it (the first five minutes, the place window before its first
	/// own pump, or Your setup's place): the window shows no more.
	#[serde(default)]
	pub place_asked: bool,
	/// The place has been through Room to move's switch-on (on by default since 2026-10-09, David: "make sure room to
	/// move is default on"; the iPhone's `free.spaceOn`): a file written before lacks this mark and holds the old
	/// default, off, so `Settings::load` switches Room to move on once with the floor moves (room to lie down gets it,
	/// beside the desk stays without it, a Room to move the player turned on stays on); a player who turns it off
	/// afterwards keeps that. Off in `Settings::default`, so a file that reads only value by value (`Settings::read`,
	/// which lays it over the defaults) is still switched on.
	#[serde(default)]
	pub space_on: bool,
	/// Send crash reports, the Menu's switch, and the newest system crash report handled (`runtime/crash.rs`).
	#[serde(default)]
	pub crash: CrashSettings,
	/// Usage statistics, the Menu's switch (`runtime/usage.rs`).
	#[serde(default)]
	pub usage: Choice,
	/// The sync's account (slice 60, `crate::sync`): Sync's switch (on by default), the account id and R, this
	/// computer's device id and token, the cursor, the seq and what was sent; R, the token, a rekey's R' and this
	/// computer's own key in the secret store with the pairing key (`secrets`).
	#[serde(default)]
	pub sync: questpump_core::sync::Account,
	/// Work mode's state as this computer holds it after its last merge (slice 67, `questpump_core::work::Work`: the
	/// switch, Every, Moves per block, work mode's own equipment, the set and the clock's anchor), shared with every
	/// device.
	#[serde(default)]
	pub work: questpump_core::work::Work,
	/// Work mode was started here once: the first Start's sheet is shown no more (`work.sheet`).
	#[serde(default)]
	pub work_started: bool,
	/// The block due on this device, kept so a restart or a reload starts its work clock due (`Clock::due_from`): the
	/// anchor it came due after (unix ms) and the moment it came due (unix seconds); none while the block counts.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub work_due: Option<WorkDue>,
}

/// A due block as `Settings::work_due` keeps it.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkDue {
	pub anchor_ms: i64,
	pub at: i64,
}

/// The page's size at 100 %, its default step.
fn own_size() -> f64 {
	1.0
}

/// A switch on by default.
fn on() -> bool {
	true
}

impl Default for Settings {
	fn default() -> Self {
		Self {
			key: [0; 32],
			hid: [0; 16],
			pairing: Pairing::default(),
			phone_name: None,
			display_lag_ms: 0,
			addons: None,
			strap: false,
			pad: false,
			ble_link: on(),
			autostart_for: None,
			autostart_off: false,
			games: GamesSettings::default(),
			offers: OfferSettings::default(),
			reminders: ReminderSettings::default(),
			sitting: None,
			runs: None,
			window: None,
			window_grown: false,
			zoom: own_size(),
			fullscreen: false,
			place: PumpPlace::default(),
			place_in_use: None,
			expedition: ExpeditionSettings::default(),
			sound: SoundSettings::default(),
			desk: DeskSettings::default(),
			favourites: Vec::new(),
			title: None,
			share_style: ShareStyle::default(),
			share_card: ShareCard::default(),
			share_format: ShareFormat::default(),
			share_sound: on(),
			place_asked: false,
			space_on: false,
			crash: CrashSettings::default(),
			usage: Choice::default(),
			sync: questpump_core::sync::Account::default(),
			work: questpump_core::work::Work::default(),
			work_started: false,
			work_due: None,
		}
	}
}

/// A switch of the Menu and the privacy page (contract/telemetry-api.yaml, "The switches and the consent"): the player's
/// answer, on or off, with its moment (unix seconds) and the version of the words shown with it
/// (`telemetry::COPY_VERSION`, GDPR Art. 7(1)); none while not answered, which reads as on outside an opt-in region and
/// as off inside one (the region read again at each start, never stored, so a player who moves is read again).
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
#[serde(default)]
pub struct Choice {
	pub send: Option<bool>,
	pub at: Option<i64>,
	pub copy: Option<String>,
}

impl Choice {
	/// The player's answer at `now` (unix seconds), kept with the words' version.
	pub fn answer(&mut self, on: bool, now: i64) {
		*self = Self { send: Some(on), at: Some(now), copy: Some(COPY_VERSION.to_owned()) };
	}

	/// The switch as it reads: the player's answer, else on outside an opt-in region.
	#[must_use]
	pub fn reads(&self, region: Region) -> bool {
		self.send.unwrap_or(region == Region::Other)
	}
}

/// Where the computer is as the switches' default reads it (contract/crash-api.yaml, "The opt-in regions": its region
/// setting and its time zone, `crash::opt_in`), read at each start while a switch is not answered and never stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Region {
	/// The EEA, India or Canada, or neither the region nor the zone read: a switch stays off until answered, and the
	/// privacy page offers Accept all and Reject all.
	OptIn,
	/// Elsewhere: a switch not answered reads on.
	Other,
}

/// The crash reports (contract/crash-api.yaml, "The desktop app"; `runtime/crash.rs`): the Menu's switch Send crash
/// reports as the player answered it (`Choice`, its fields beside these in the file), the newest of the system's crash
/// reports of QuestPump handled (taken, deleted by the player's answer or skipped), so none at or before it is offered
/// again, and the moment of the newest panic report handled (unix seconds), so the system's report of the abort it
/// ended in, found at a later start, never goes either (`crash::same_crash`).
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
#[serde(default)]
pub struct CrashSettings {
	#[serde(flatten)]
	pub choice: Choice,
	pub handled: Option<Handled>,
	pub handled_panic: Option<i64>,
}

/// The desktop app's sound and voice (the Menu's Sound and voice page; `desk::audio`): the sounds (on by default, never
/// the only signal), the voice's lines in a pump (on by default, as the game's own voice is), its callouts after a
/// move's line (on by default, as the game's General page has them; heard only with the voice on), the rep count, each
/// rep's number as a running set of reps' figure finishes it ("Count reps aloud", on by default, as the game and the
/// phones have it, and set on the welcome's place step too; heard only with the sounds and the voice on; 2026-10-09),
/// the voice's "Match over." on the match card (off by default), the volume (`Levels`), the output the player picked
/// (none: the system's own), and whether the player chose one yet (the first pump asks "Speakers or headset?" until
/// then). The struct's own `serde(default)` reads a file written before a switch was added with that switch's default.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(default)]
#[expect(clippy::struct_excessive_bools, reason = "the Sound and voice page's switches, read as they are")]
pub struct SoundSettings {
	pub on: bool,
	pub voice: bool,
	pub callouts: bool,
	pub count: bool,
	pub voice_offers: bool,
	pub levels: Levels,
	pub output: Option<Output>,
	pub output_picked: bool,
}

impl Default for SoundSettings {
	fn default() -> Self {
		Self {
			on: true,
			voice: true,
			callouts: true,
			count: true,
			voice_offers: false,
			levels: Levels::default(),
			output: None,
			output_picked: false,
		}
	}
}

/// The Sound and voice page's volume, each a percent from 0 (silent) to 100 (as recorded, the default, so a file
/// written before 2026-10-23 plays as it did): the master level over everything, the sounds' and the voice's (every
/// take: a pump's lines, the rep count, the match card's "Match over." and `/qp ping`'s answer). A level sets how loud,
/// never whether: the switches alone decide what plays (`desk::audio`).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(default)]
pub struct Levels {
	pub master: u8,
	pub sounds: u8,
	pub voice: u8,
}

impl Default for Levels {
	fn default() -> Self {
		Self { master: 100, sounds: 100, voice: 100 }
	}
}

/// An audio output the player picked: its system id (cpal's `DeviceId`, `<host>:<id>`) and its name as it was then,
/// for the note while it is not connected.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Output {
	pub id: String,
	pub name: String,
}

/// A share card's style (docs/plan.md slice 34; the phones' `ShareStyle`): Parchment, the quest page as the pump shows
/// it, free; Night and Gilded, Pro's.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ShareStyle {
	#[default]
	Parchment,
	Night,
	Gilded,
}

/// The Book's Ladders side as the player set its Tree | List switch on this computer (slice 38).
/// The span Pump home's recent games count as the player set their This week | All time switch on this computer
/// (2026-10-28): the moves done this game week, the default, or in all.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum PlayedSpan {
	#[default]
	Week,
	All,
}

/// A day's share card (the Saga, 2026-10-15; the phones' `ShareCard` kinds): the Saga card or the Quest Bill.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ShareCard {
	#[default]
	Saga,
	Bill,
}

/// A share card's format (the Saga, 2026-10-15): the story (1080 × 1920), the post (1080 × 1350), the square
/// (1080 × 1080), the sticker (transparent, 1080 wide, the Saga card's alone) or, since slice 45, the reel (a day's
/// Saga card in motion, 1080 × 1920, `desk::reel`; a card that has none draws the story in its place). Every one free.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ShareFormat {
	#[default]
	Story,
	Post,
	Square,
	Sticker,
	Reel,
}

/// Where this computer's pumps happen (slice 35): the equipment at hand (Library.lua's `needs` keys), floor moves,
/// room to move and quiet. Each device keeps its own (the body every app device shares leaves it out), from the
/// phones' defaults (`EngineProfile`) and the addon's: no equipment, floor moves, room to move (since 2026-10-09; a
/// place kept before with the floor moves is switched on once, `Settings::space_on`), not quiet.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(default)]
pub struct PumpPlace {
	pub gear: Vec<String>,
	pub floor: bool,
	pub space: bool,
	pub quiet: bool,
}

impl Default for PumpPlace {
	fn default() -> Self {
		Self { gear: Vec::new(), floor: true, space: true, quiet: false }
	}
}

impl From<&PumpPlace> for PlaceSetup {
	fn from(p: &PumpPlace) -> Self {
		Self { gear: p.gear.clone(), floor: p.floor, space: p.space, quiet: p.quiet }
	}
}

impl From<&PlaceSetup> for PumpPlace {
	fn from(p: &PlaceSetup) -> Self {
		Self { gear: p.gear.clone(), floor: p.floor, space: p.space, quiet: p.quiet }
	}
}

/// The timed pumps on this computer (slice 51; kept on this device alone): the Quick pump's length in minutes, the
/// Menu's Quick pump length (2026-11-03: one of `expedition_lengths_min`; none, or one no longer listed, its first),
/// and the minutes of the longest timed pump that ended here (none before the first), which the share card's "My
/// longest timed pump yet" beats. A field of an earlier build is ignored as it reads (`last`, the setup page's last
/// used length, until 2026-10-21; `lengths`, the length row's three, until 2026-11-03).
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
#[serde(default)]
pub struct ExpeditionSettings {
	pub quick: Option<u16>,
	pub longest: Option<u16>,
}

/// A place picked on this computer: its id among the places and when (unix ms).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PlacePick {
	pub id: String,
	pub at: i64,
}

/// `place_in_use` as kept: a pick, or a file of before 2026-11-03's id alone, a pick made now.
fn pick_or_id<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<PlacePick>, D::Error> {
	#[derive(Deserialize)]
	#[serde(untagged)]
	enum Kept {
		Pick(PlacePick),
		Id(String),
	}
	Ok(Option::<Kept>::deserialize(d)?.map(|kept| match kept {
		Kept::Pick(pick) => pick,
		Kept::Id(id) => PlacePick { id, at: wall_ms() },
	}))
}

/// The desktop app's own state of its Menu (slice 35; `desk::menu_pages`): the first five minutes, which a computer
/// whose settings lack `desk.onboarded` shows a new player at the main window's first open (a player with progress
/// never sees them), when the player last saved a backup file, Pump home's cards answered, the game days the Saga's
/// best-day moment and reveal last showed, and the games played on this
/// computer with their pumps (Pump home's recent games, 2026-10-27; `desk::recent`) and the span they count
/// (2026-10-28). A field of an earlier build is ignored as it reads (`addon`, the welcome's "I play with the addon"
/// until 2026-10-08; `tree_hinted`, the trees' first-time callout until 2026-10-22; `games_card`, Pump home's card that
/// offered the games found, until slice 56; `book_view`, the Book's Tree | List switch, until 2026-10-29).
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
#[serde(default)]
pub struct DeskSettings {
	/// The first five minutes are done, or QuestPump found progress.
	pub onboarded: bool,
	/// Where the first five minutes stand: a closed window or a restart picks them up there.
	pub welcome: Step,
	/// The first look after the stores were read found no progress: from then on only the game's or a phone's ends the
	/// first five minutes, never this computer's own rows (the flow's own first pump's).
	pub found_new: bool,
	/// The first five minutes asked the experience question, so Back from where you pump goes back to it.
	pub asked_experience: bool,
	/// The first five minutes ended because QuestPump found the player's progress, and where it came from, until the
	/// window's Toast has said so once (`welcome.toasted`).
	pub toast: Option<Came>,
	/// When the player last saved a backup file, unix milliseconds.
	pub backup_at: Option<i64>,
	/// Pump home's card that pairs a phone was put away by its Not now until then, unix seconds.
	pub pair_card_until: Option<i64>,
	/// Pump home's card that offers the addon was put away by its Not now until then, unix seconds.
	pub addon_card_until: Option<i64>,
	/// The game day ("yyyy-MM-dd") the best-day moment last showed on this computer (`desk::log`): it shows once a game
	/// day, never synced.
	pub best_day: Option<String>,
	/// The game day ("yyyy-MM-dd") the Saga's reveal last played on this computer (`desk::log`): it plays once a game
	/// day, a restart included, never synced.
	pub revealed: Option<String>,
	/// The games played on this computer, when each was last seen running, when its pumps were and their moves past
	/// four weeks (`desk::recent`): Pump home's recent games, never synced or sent anywhere.
	pub played: Vec<Played>,
	/// What Pump home's recent games count, as the player set This week | All time (2026-10-28).
	pub played_span: PlayedSpan,
}

/// The games on the computer (`games/`, slice 56): every game found is watched until the player turns it off (the Riot
/// games too since 2026-10-28; until then once the player turned them on).
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GamesSettings {
	/// The named games the player asked for by their contract/links.json key: a file game set up ("Ask after each
	/// match": its file goes in), and until 2026-10-28 a Riot game turned on, whose key may stay here; `on` until
	/// slice 56, when it listed every named game turned on.
	#[serde(default, alias = "on")]
	pub asked: Vec<String>,
	/// Every game the player turned off, by its key: a named game's contract/links.json key, a found game's, a picked
	/// program's. It stays off when it is found again.
	#[serde(default)]
	pub off: Vec<String>,
	/// When the Games tab was last opened, unix seconds: a game found since lights its dot (none until its first
	/// open, when every game found lights it).
	#[serde(default)]
	pub seen_at: Option<i64>,
	/// The programs the player picked.
	#[serde(default)]
	pub programs: Vec<Program>,
	/// The Game State Integration listener's port on 127.0.0.1, kept so the games' cfg files stay right; a new one
	/// when another program took it.
	#[serde(default)]
	pub gsi_port: Option<u16>,
	/// The cfg files' token: 24 random bytes, base64url. Never logged, never in diagnostics or feedback.
	#[serde(default)]
	pub gsi_token: Option<String>,
	/// Every file the helper wrote into a game's folder, by game: turning the game off removes it, and on Windows the
	/// uninstaller does (`game-files`, `games/files.rs`); a shared file it added its part to stays, and stays listed
	/// (`GameFile::shared`).
	#[serde(default)]
	pub written: Vec<GameFile>,
	/// The games heard from since their file was last written: they need no new start.
	#[serde(default)]
	pub heard: Vec<String>,
	/// The loopback ports the player set in a fed game's window for its feed (Legends of Runeterra's API, F1 25's and
	/// Forza Motorsport's telemetry; slice 48), by key; a game not listed takes its default
	/// (`Watched::default_port`).
	#[serde(default)]
	pub ports: BTreeMap<String, u16>,
	/// The games noticed running (`games/detect.rs`, slice 48), newest first: each watched until the player turns it
	/// off (`off`).
	#[serde(default)]
	pub detected: Vec<Detected>,
	/// The found games the player put away with "Not a game", newest first: never noticed again until put back.
	#[serde(default)]
	pub dismissed: Vec<Detected>,
	/// The Games tab's "Notice games I play" turned off (on by default): no new game is found, since no program is
	/// looked at for one; the games found, the named games and the programs added keep offering (a game's own off and
	/// the pause stop its offers).
	#[serde(default)]
	pub notice_off: bool,
	/// The player turned "Open the pump after each match" off (the Games tab; on by default since 2026-10-27,
	/// Rapha: "make it default to start for everyone"): while it is on, a watched game's close, its offer standing,
	/// opens the main window at Ready on its screen (`desk::card`). Until then the file kept `open_on_close`, off by
	/// default and written as false whether or not the player chose it, so it is no longer read (serde passes over the
	/// key, and the next save drops it): every player starts on, and only an off chosen since counts.
	#[serde(default)]
	pub open_on_close_off: bool,
	/// The games marked `aim` in contract/links.json whose "Rest your arms while it runs" the player turned off in the
	/// game's window, by their key (slice 48): on by default for every aim game.
	#[serde(default)]
	pub aim_off: Vec<String>,
}

impl GamesSettings {
	/// Whether the arms rest is on for the game `key` (none for a game not marked `aim`: `Watched::aim`).
	#[must_use]
	pub fn aim_on(&self, key: &str) -> bool {
		!self.aim_off.iter().any(|k| k == key)
	}

	/// Whether the player asked for the named game `key` (`asked`).
	#[must_use]
	pub fn is_asked(&self, key: &str) -> bool {
		self.asked.iter().any(|k| k == key)
	}

	/// Whether the player turned the game `key` off (`off`).
	#[must_use]
	pub fn is_off(&self, key: &str) -> bool {
		self.off.iter().any(|k| k == key)
	}

	/// The picked programs whose key is a found game's leave `programs` (slice 56: a found game turned on was one of
	/// them; found means watched now); how many left.
	pub fn drop_found_programs(&mut self) -> usize {
		let before = self.programs.len();
		let detected = &self.detected;
		self.programs.retain(|p| detected.iter().all(|d| d.key != p.key));
		before - self.programs.len()
	}
}

/// The sitting clock a quit kept (`runtime/sitting.rs`): the end of the input's latest pause, the last look a watched
/// game counted as input at (none before 2026-10-29) and when the quit came, unix seconds.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeptClock {
	pub since: i64,
	#[serde(default)]
	pub game_seen: Option<i64>,
	pub quit_at: i64,
}

/// A program the player picked: `Watched::Program`'s key and name.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Program {
	pub key: String,
	pub name: String,
}

/// A game noticed running: its program's key (`Watched::Program`'s; `steam:<app>` for a Steam game), the name it
/// shows, when it was first found and when it last closed (unix seconds; a new one takes the place of the found game
/// whose close is the oldest, `detect.rs`); nothing else of it is kept. `asked`, Pump home's one question about it
/// until slice 56, is ignored as it reads.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Detected {
	pub key: String,
	pub name: String,
	pub seen: i64,
	#[serde(default)]
	pub closed: Option<i64>,
}

/// A file the helper wrote into a game's folder, and the game's key; `shared` when the helper only added its part to a
/// file other programs keep too (Hearthstone's `log.config`, slice 48), which is never removed.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct GameFile {
	pub game: String,
	pub path: PathBuf,
	#[serde(default)]
	pub shared: bool,
}

/// The computer's match offers (`offers.rs` in questpump-core): the pause, the latest game offer on any device, which
/// starts the sitting clock again, and the reminders' quiet hours, daily cap and this game day's count, kept here as
/// before slice 48, when the match offers shared them (the constants' defaults while unset).
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct OfferSettings {
	#[serde(default)]
	pub pause: Pause,
	#[serde(default)]
	pub quiet_from_min: Option<u16>,
	#[serde(default)]
	pub quiet_to_min: Option<u16>,
	#[serde(default)]
	pub cap: Option<u32>,
	/// The latest game offer on any device, unix seconds.
	#[serde(default)]
	pub last_game_offer_at: Option<i64>,
	/// The game day (its first unix second) `shown` counts, and the reminders shown on it.
	#[serde(default)]
	pub shown_day: Option<i64>,
	#[serde(default)]
	pub shown: u32,
}

impl OfferSettings {
	/// The quiet hours, minutes after local midnight.
	#[must_use]
	pub fn quiet(&self, t: &Tunables) -> (u16, u16) {
		(self.quiet_from_min.unwrap_or(t.quiet.0), self.quiet_to_min.unwrap_or(t.quiet.1))
	}

	/// The day's cap, from 1 to `remind_cap_max`.
	#[must_use]
	pub fn cap(&self, t: &Tunables) -> u32 {
		self.cap.unwrap_or(t.cap).clamp(1, t.cap_max.max(1))
	}

	/// The reminders shown on game day `day`.
	#[must_use]
	pub fn shown_on(&self, day: i64) -> u32 {
		if self.shown_day == Some(day) { self.shown } else { 0 }
	}

	/// A match offer made at unix second `at` went out to the devices: the latest game offer, which starts the sitting
	/// clock again (slice 48: it counts under no cap).
	pub fn game_offered(&mut self, at: i64) {
		self.last_game_offer_at = Some(self.last_game_offer_at.map_or(at, |t| t.max(at)));
	}

	/// One more reminder (`desk::reminders`) on game day `day` under the cap.
	pub fn count(&mut self, day: i64) {
		self.shown = self.shown_on(day) + 1;
		self.shown_day = Some(day);
	}
}

/// The computer's reminders (the Menu's Reminders page; `desk::reminders`), the phones' streak reminder and sitting cue
/// (slice 32) on the computer, each off until the player turns it on (the streak reminder also by the offer after the
/// first Result here); their quiet hours and daily cap stay stored with the offers (`OfferSettings`), which shared them
/// until slice 48.
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
#[serde(default)]
pub struct ReminderSettings {
	/// The streak reminder is on: once a game day, on a day with no pump yet.
	pub streak_on: bool,
	/// Its time, minutes after local midnight; none: `remind_streak_min`.
	pub streak_at: Option<u16>,
	/// The sitting cue is on: after `sit_after_min` at this computer with no pump, a short pump on offer.
	pub sitting_on: bool,
	/// "Remind me once a day?" was shown after a Result here, whichever the answer.
	pub offered: bool,
	/// The game day (`GameDay.index`) the streak reminder last showed on: once a game day.
	pub streak_day: Option<i64>,
	/// When the sitting cue last showed, unix seconds.
	pub sitting_at: Option<i64>,
}

/// How the settings came in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Loaded {
	/// Read back with the pairing key (a value that did not read at its default alone: `Settings::read`).
	Read,
	/// No file yet and no pairing key in the secrets' store: the first start.
	First,
	/// A file that did not read, or whose pairing key did not (a disk error, a hand edit), while the secrets' store
	/// held none that reads (`secrets`): kept aside as `settings.json.unread`, and the helper starts with a new pairing
	/// code and no pairing, every other value that read kept.
	Unread,
}

impl Settings {
	/// A new pairing key: every device paired with the old one must scan the new code.
	pub fn new_key(&mut self) {
		rand::fill(&mut self.key);
	}

	/// The install's id as every response's helper block, the hello and the Bonjour record carry it: base64url without
	/// padding, as in the pair link.
	#[must_use]
	pub fn hid_text(&self) -> String {
		use base64::Engine as _;
		base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(self.hid)
	}
}

/// Where the desktop app's main window stood (its `desk/place.rs` places it): its outer position and its screen's
/// position in the desktop's units, its inner size in logical pixels, its screen's name and whether it was maximized
/// (the size and position then are the ones it had before).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Place {
	pub x: i32,
	pub y: i32,
	pub width: f64,
	pub height: f64,
	/// The screen's name and position, which tell it apart from another of the same name.
	pub monitor: Option<String>,
	pub monitor_x: i32,
	pub monitor_y: i32,
	pub maximized: bool,
}

/// A game played on the computer (settings.json's `desk.played`; the desktop app's `recent.rs`, Pump home's
/// recent games).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Played {
	/// The game's key: `Watched::id` (a named game's contract/links.json key, a found or picked program's own), or
	/// `wow`.
	pub key: String,
	/// When it was last seen running, unix seconds.
	pub last: i64,
	/// When each of its pumps with a move done began, unix seconds, oldest first (none for World of Warcraft, whose
	/// rows hold its moves); the field's name stays for the settings already written.
	#[serde(default)]
	pub pumps: Vec<i64>,
	/// The moves done in its pumps that went from `pumps` past `KEEP_S`, so All time keeps them (2026-10-28; none in a
	/// record written before, which kept none older).
	#[serde(default)]
	pub before: usize,
}

/// The wall clock in milliseconds, for a file of before 2026-11-03 read without a platform (`pick_or_id`).
fn wall_ms() -> i64 {
	std::time::SystemTime::now()
		.duration_since(std::time::UNIX_EPOCH)
		.map_or(0, |d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX))
}

/// A new install id.
#[must_use]
pub fn new_hid() -> [u8; 16] {
	let mut hid = [0; 16];
	rand::fill(&mut hid);
	hid
}

/// The key and the install id as base64url text in the file; a value of another length does not read.
mod bytes_text {
	use base64::Engine as _;
	use base64::engine::general_purpose::URL_SAFE_NO_PAD;
	use serde::{Deserialize, Deserializer, Serializer};

	pub fn serialize<S: Serializer, const N: usize>(bytes: &[u8; N], s: S) -> Result<S::Ok, S::Error> {
		s.serialize_str(&URL_SAFE_NO_PAD.encode(bytes))
	}

	pub fn deserialize<'de, D: Deserializer<'de>, const N: usize>(d: D) -> Result<[u8; N], D::Error> {
		let text = String::deserialize(d)?;
		let bytes = URL_SAFE_NO_PAD.decode(text).map_err(serde::de::Error::custom)?;
		bytes.try_into().map_err(|_| serde::de::Error::custom(format!("not {N} bytes")))
	}
}
