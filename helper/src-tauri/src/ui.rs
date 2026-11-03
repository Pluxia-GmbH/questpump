//! What the desktop app shows, kept in one view the runtime writes and every surface reads (docs/plan.md slice 35): the
//! window's topics (`shell`, `menu.computer`, `menu.phones`, `menu.devices`, `feedback`) and the tray's lines. The view
//! holds state, never words: each line is a value the page words through `helper/dist/lib/copy.js` and the tray
//! through `copy.rs`, so one change reaches the window and the tray alike; a line both show (this computer's state,
//! the phone's, the feedback note) is worded once, by the tray's rule, and published in its words beside its state
//! (`desk::tray::state`, `phone_line`, `note_line`), and the tray's colour beside it for Pump home's Chip
//! (`desk::tray::tone`). The tray's dot mirrors the capture (grey: no game, or the Screen Recording grant missing
//! before a device is paired or a strap or a walking pad turned on; amber: no strip, which after a read means none
//! read for the runtime's `OUT_AFTER`; green: live; red: the grant missing while one is, the stream stopped, or
//! QuestPump could not start).
//! A change is made under the view's lock, which also stages its topics in the desk's publisher (which keeps only those
//! that changed, each with its rev, so the revs follow the order of the changes whichever threads make them); the
//! topics are sent and the tray is drawn on the main thread once the lock is released (a send evaluates a script in the
//! webview and the tray's setters wait for the main thread, which may itself be waiting for the lock). Quit goes
//! through the loop, after "Leave the pump?" while a pump holds the window (`desk::Desk::guard`), at once for the
//! system's own quit.

use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;
use tauri::{AppHandle, Manager};
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot;

use crate::desk::seams::FoundGame;
use crate::desk::state::Staged;
use crate::desk::tray::{State, Tone};
use crate::desk::{self, Desk, DeskAsk};

/// The capture's state, the tray's dot and the `shell` topic's `dot`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Dot {
	#[default]
	NoGame,
	NoStrip,
	Live,
	Stopped,
	/// The capture waits for the Screen Recording grant, which This computer asks for once a game with QuestPump is
	/// known and a device is paired (or a strap or a walking pad is on).
	Off,
}

/// The engine's state, the `shell` topic's `engine` (`desk::engine`): a fold of every row runs (at the start, after a
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

/// Everything the window and the tray show, by topic.
#[derive(Debug, Clone, Default)]
pub struct View {
	pub shell: Shell,
	pub computer: Computer,
	pub phones: Phones,
	pub devices: Devices,
	pub feedback: FeedbackView,
}

impl View {
	/// What the tray draws.
	fn tray(&self) -> desk::tray::TrayState {
		desk::tray::TrayState {
			dot: self.shell.dot,
			state: self.computer.state_line.clone(),
			phone: self.phones.phone.clone(),
			note: self.feedback.note.clone(),
		}
	}
}

/// The window's frame: the system, the dot, where the window goes when it opens, the page's size and its steps, the
/// full-screen choice and the window's own full screen, its focus, the leave a close or a quit asks for while a pump
/// holds the window, and the engine's state.
#[derive(Serialize, Debug, Clone, PartialEq)]
#[expect(
	clippy::struct_excessive_bools,
	// copy-lint: not shown (a lint's reason)
	reason = "the `shell` topic's independent flags, which the page reads by name"
)]
pub struct Shell {
	/// `macos` or `windows`: the page picks the words of a step that differs.
	pub platform: &'static str,
	pub version: &'static str,
	pub dot: Dot,
	pub go: Go,
	/// The page's size, the player's step (1 is 100 %, the default; the window shows the page at
	/// `desk::windows::BASE` times it), kept in settings.json: the Menu's Size shows it.
	pub zoom: f64,
	/// The steps the size moves through, `desk::windows::ZOOMS`: the Menu's Size reads its ends from them.
	pub zooms: &'static [f64],
	/// The player's choice that Big view fills the screen (the window's own full screen), kept in settings.json: Big
	/// view applies it as it opens.
	pub fullscreen: bool,
	/// The window is in its own full screen now (Big view, the View menu, the green button), read from the window at
	/// each resize: what a full-screen toggle shows.
	pub fullscreen_now: bool,
	/// The main window has the keyboard's focus (Rust's `WindowEvent::Focused`, never the page's blur).
	pub focused: bool,
	/// A screen reader runs (`desk::audio::screen_reader`: macOS's `VoiceOver`; on Windows the system's flag that one
	/// does, which Narrator may leave unset), read at each change of the window's focus: the Book's Tree | List switch
	/// defaults to List under it (slice 38).
	pub reader: bool,
	/// A close or a quit asked while a pump holds the window: the pump asks "Leave the pump?".
	pub leave: Option<Leave>,
	/// The bundle's resources folder (`resources/` in it: the figures' stills and pairs, the voice, the sounds), which
	/// the page reaches through the asset protocol (`convertFileSrc`); none where the system gives no resource folder.
	pub resources: Option<String>,
	/// `busy` until the engine's fold lands: no view waits on it, and one that needs the engine says it is busy.
	pub engine: EngineState,
	/// The last fold, named in the report for help's header.
	#[serde(skip)]
	pub fold: Option<Fold>,
	/// The player chose where the sound plays (`desk::audio`): until then the first pump asks "Speakers or headset?".
	pub output_picked: bool,
	/// A line of the pump's voice while a screen reader runs and the main window has the focus, which the page says in
	/// its one polite live region instead of the voice (`desk::audio`), taken each time `seq` moves on.
	pub announce: Option<Announce>,
}

impl Default for Shell {
	fn default() -> Self {
		Self {
			platform: if cfg!(windows) { "windows" } else { "macos" },
			version: env!("CARGO_PKG_VERSION"),
			dot: Dot::NoGame,
			go: Go { to: desk::windows::ROOT, seq: 0, chosen: false },
			zoom: 1.0,
			zooms: &desk::windows::ZOOMS,
			fullscreen: false,
			fullscreen_now: false,
			focused: false,
			reader: false,
			leave: None,
			resources: None,
			engine: EngineState::Busy,
			fold: None,
			output_picked: false,
			announce: None,
		}
	}
}

/// A line for the page's live region: its words, taken each time `seq` moves on.
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Announce {
	pub text: String,
	pub seq: u64,
}

/// Where the page goes: a root's id or a Menu page's (`desk.menu.phones`), taken each time `seq` moves on.
#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct Go {
	pub to: &'static str,
	pub seq: u64,
	/// The player chose this root in the macOS View menu (slice 46): the page presses its tab, as Cmd/Ctrl+1 to 5 do,
	/// its sound and its count with it, unless a pump's cover or a window stands over the tabs.
	pub chosen: bool,
}

/// A leave asked for: what follows once the player leaves the pump, taken each time `seq` moves on.
#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct Leave {
	pub then: Then,
	pub seq: u64,
	/// The copy moved into Applications that a second launch asked to take over (`desk::launch::second`): started
	/// once this process has gone, only when the player leaves the pump (`ui::quit_for`); their Stay drops it with the
	/// ask.
	#[serde(skip)]
	pub start: Option<PathBuf>,
}

impl Leave {
	/// The leave asked after `standing` (its `seq` moved on), keeping a start the standing ask carried, which a quit
	/// or a close asked again while its window stands must not lose.
	pub fn after(standing: Option<&Self>, then: Then, start: Option<PathBuf>) -> Self {
		Self {
			then,
			seq: standing.map_or(1, |l| l.seq + 1),
			start: start.or_else(|| standing.and_then(|l| l.start.clone())),
		}
	}
}

#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Then {
	/// The window closes.
	Close,
	/// QuestPump quits.
	Quit,
}

/// This computer: its state in words, the game, the screen reading and its grant, the player's progress kept for the
/// game, the game folder and its figures, the version, starting with the computer, the steps a Mac or a Windows PC may
/// need, and the move figures' films in the app.
#[derive(Serialize, Debug, Clone, Default, PartialEq)]
pub struct Computer {
	/// This computer's state (`desk::tray::state`, from this view), kept by `update`: Pump home's Chip and the page's
	/// line say what the tray says.
	pub state: State,
	/// The state in words, the tray's (`desk::tray::state_line`), kept by `update`.
	pub state_line: String,
	/// The colour of this computer's dot, the tray icon's (`desk::tray::tone`, from the shell's `dot`), kept by
	/// `update`: Pump home's Chip shows it while this computer's state needs the player (`desk::home`'s `Fix`).
	pub tone: Tone,
	/// What QuestPump found of the game (`Helper::found`'s `game`), which the state reads.
	pub found: FoundGame,
	/// The other games QuestPump watches for their matches' end, by name (`Helper::show_games`): with no game found,
	/// the state names them.
	pub watching: Vec<String>,
	/// The World of Warcraft client running now, none while none runs.
	pub game: Option<RunningGame>,
	pub screen: Screen,
	/// When the game last sent `/qp ping` (unix ms), until the screen line changes.
	pub ping_at: Option<i64>,
	/// The Screen Recording grant is missing while a device needs it: the page explains it and asks.
	pub permission: bool,
	/// macOS asked for the grant before: the button opens System Settings, and a restart applies a grant.
	pub asked: bool,
	/// How many times the restart that applies a grant did not happen: each failure changes it, so the page puts its
	/// button back after every one.
	pub restart_failed: u32,
	pub mirror: Mirror,
	pub folder: Folder,
	/// Why a folder the player picked was not taken.
	pub folder_note: Option<FolderNote>,
	pub pack: Pack,
	pub version: Version,
	/// Starts with the computer; none where the system cannot say (a bare binary in development, a Mac app still in its
	/// download), which hides the switch.
	pub autostart: Option<bool>,
	/// Windows: why QuestPump cannot write into the game folder.
	pub access: Access,
	pub access_note: Option<AdminNote>,
	/// QuestPump's own program, which Controlled folder access must allow.
	pub program: String,
	/// macOS: QuestPump runs from its download (`update::unmoved`): the page asks to move it to Applications.
	pub unmoved: bool,
	/// The move figures' films in this app (`desk::figures`): bundled, kept from their download, or on their way.
	pub animations: Animations,
	/// The addon in the game: what QuestPump can do for it and its line (`runtime/addon.rs`).
	pub addon: AddonView,
}

/// The addon in the game as This computer, the welcome and Pump home's card show it (`Helper::addon_view`).
#[derive(Serialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct AddonView {
	/// What a click does: add QuestPump to the game found without it, or update an install another updater made.
	pub offer: Option<AddonOffer>,
	pub line: AddonLine,
}

#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AddonOffer {
	/// "Add QuestPump to the game".
	Add,
	/// "Update now": a newer version is out for an install QuestPump does not manage (one click, and it does).
	Update,
}

/// The addon's line.
#[derive(Serialize, Debug, Clone, Default, PartialEq, Eq)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum AddonLine {
	/// No game found: nothing to say.
	#[default]
	None,
	/// The game is here without QuestPump.
	Missing,
	Adding,
	AddFailed {
		error: String,
	},
	/// Added while the game ran: it loads QuestPump once started again.
	Added,
	/// The installed version, with no newer one known (no feed checked yet, or a version that does not read).
	Version {
		version: String,
	},
	/// No newer version in the feed.
	Current {
		version: String,
	},
	Updating,
	/// A newer version is staged for an install QuestPump manages: it goes in once the game is closed.
	Ready {
		version: String,
		new: String,
	},
	/// A newer version is out for an install QuestPump does not manage (with Update now).
	Newer {
		version: String,
		new: String,
	},
	Failed {
		error: String,
	},
}

/// The running client as This computer names it: its install's label (`install::label`, as the folder line names the
/// install), none where the install names none; and whether its squares were read since it started.
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct RunningGame {
	pub label: Option<String>,
	/// A frame of this client read since it started (the runtime's `Sighting::read`), kept by the runtime's `screen`:
	/// with its squares out of view the state is then "out of view", never "Waiting to see QuestPump"
	/// (`desk::tray::state`). Each client process starts unread.
	#[serde(skip)]
	pub read: bool,
}

/// The move figures' films on this computer (`desk::figures`): a move shows its still until they are here.
#[derive(Serialize, Debug, Clone, Default, PartialEq, Eq)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum Animations {
	/// Before the start has looked.
	#[default]
	Unknown,
	/// The app carries them.
	Bundled,
	/// Downloaded once and kept in the data folder.
	Kept,
	/// On their way: how much of the download has come, in percent.
	Downloading { percent: u8 },
	/// The download failed; QuestPump tries again later.
	Failed { error: String },
	/// Neither in the app nor to be fetched (a build without them and without a feed).
	Missing,
}

/// The screen reading's line.
#[derive(Serialize, Debug, Clone, Default, PartialEq, Eq)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum Screen {
	/// Before the game ever showed its squares.
	#[default]
	NotSeen,
	/// From the game's start until its squares show.
	Waiting,
	Reading,
	/// The running game has no QuestPump (the page names it as the game line does).
	NoAddon,
	/// The squares are out of sight for a moment (a movie, a loading screen, the menu); `hdr` on an HDR display.
	Out {
		hdr: bool,
	},
	/// Out of sight for a minute: QuestPump may be turned off in the game.
	Unseen,
	/// The game's strip is a newer or an older version than this program reads.
	Newer,
	Older,
	/// The squares show, and their colours do not read.
	Colours {
		hdr: bool,
	},
	/// The capture stopped.
	Stopped,
	/// The grant is missing and a device needs it: the card above asks.
	Grant,
	/// The grant is missing and nothing needs it yet.
	Off,
	/// QuestPump could not start.
	Failed {
		error: String,
	},
}

/// The progress line: the mirror in the game folder.
#[derive(Serialize, Debug, Clone, Default, PartialEq, Eq)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum Mirror {
	#[default]
	Waiting,
	/// Written at this instant (unix ms).
	Kept {
		at: i64,
	},
	Failed {
		error: String,
	},
	/// The game folder does not resolve.
	Unfound,
	/// No game folder with QuestPump is known: the progress is kept on this computer meanwhile.
	Here,
}

/// The game folder line.
#[derive(Serialize, Debug, Clone, Default, PartialEq, Eq)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum Folder {
	/// Before any look.
	#[default]
	Unknown,
	/// No game was found on this computer, with QuestPump or without (a game without it shows as `Found`, its pack line
	/// saying QuestPump is not in it): choose its folder where it is.
	None,
	/// The folder, and the game's version found there.
	Found { path: String, label: Option<String> },
}

/// Why a folder the player picked was not taken.
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum FolderNote {
	/// The running game uses this folder: close it first.
	Running { path: String },
	/// The picked folder holds no AddOns folder.
	NoAddons { path: String },
	/// The picked folder's game has no QuestPump yet: QuestPump follows it once the addon is there.
	NoQuestPump { path: String },
}

/// The move figures' pack beside QuestPump in the game folder.
#[derive(Serialize, Debug, Clone, Default, PartialEq, Eq)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum Pack {
	#[default]
	NotSeen,
	/// The folder has no QuestPump.
	Absent,
	Installed,
	Missing,
	/// From another download.
	Other {
		pack: String,
		addon: String,
	},
}

/// The version line: the build and what the updater knows.
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Version {
	pub number: &'static str,
	pub build: Build,
	pub update: Option<Update>,
}

impl Default for Version {
	fn default() -> Self {
		Self { number: env!("CARGO_PKG_VERSION"), build: Build::Development, update: None }
	}
}

/// What kind of build runs (`update::installed`, `update::unmoved`).
#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Build {
	/// A published build in its installed bundle: it updates itself.
	Installed,
	/// macOS: a published build still in its download.
	Unmoved,
	/// A build made by hand: it never updates itself.
	Development,
}

/// What the last update check came to.
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum Update {
	/// Downloaded: it installs once the game and the window are closed, no pump runs and no watched game is in a match
	/// (`runtime/updates.rs`); `now` while nothing but the open window holds it, so This computer's Restart to update
	/// installs it at once.
	Ready {
		new: String,
		now: bool,
	},
	Current,
	Failed,
	/// macOS: this account cannot replace the app.
	Unreplaceable {
		new: String,
	},
	Installing {
		new: String,
	},
	NotInstalled,
}

/// Windows: why QuestPump cannot write into the game folder.
#[derive(Serialize, Debug, Clone, Copy, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Access {
	#[default]
	Fine,
	/// The folder's permissions: the page offers the one-time elevated grant.
	Acl,
	/// Controlled folder access: the page says where the player allows QuestPump.
	Cfa,
}

/// How a one-time elevated step went (Windows).
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum AdminNote {
	/// Its prompt is up.
	Waiting,
	Declined,
	Failed {
		error: String,
	},
	/// The firewall rule was made, and another rule still keeps the phone out.
	Another,
}

/// Phones: the pairing code, the address under it, the paired phone, the step left in the game, on Windows the
/// firewall's card, whether the pairing ended because the settings did not read, and the Bluetooth link's row.
#[derive(Serialize, Debug, Clone, Default, PartialEq)]
pub struct Phones {
	/// The pair link's QR code; none without a home network, before the firewall lets the phone in or in a run whose
	/// secrets did not read (`AddressLine::Unread`).
	pub qr: Option<crate::pairing::Qr>,
	pub address: Option<AddressLine>,
	pub phone: PhoneLine,
	/// The phone's line in words, the tray's (`desk::tray::phone_line`), kept by `update`.
	pub phone_line: String,
	/// The pairing's step (`Helper::show_next`, since 2026-10-13 the devices' `link::Step`): none when nothing is left.
	pub next: Option<questpump_core::link::Step>,
	/// Windows: whether the firewall lets the phone in.
	pub firewall: FirewallCard,
	pub network_note: Option<AdminNote>,
	/// The settings did not read at this start (`settings::Loaded::Unread`), so a new pairing code ended the pairing
	/// (no phone is named anywhere): the page asks to scan the code again until a phone pairs.
	pub unread: bool,
	/// The paired phone has not linked for two weeks (since 2026-10-09): the page shows the code again, with why.
	pub quiet: bool,
	/// The Bluetooth link's switch and its line (slice 43).
	pub ble: BleRow,
}

/// The Phones page's Bluetooth row (slice 43): the link's switch, on by default, and its line.
#[derive(Serialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct BleRow {
	pub on: bool,
	/// None while the link has not said how it stands, serves with no phone linked by it or the switch is off (since
	/// 2026-10-22: the switch says those alone).
	pub line: Option<BleLine>,
}

/// The Bluetooth row's line: the helper block's `ble` in the page's words, a linked phone's name, or that it could not
/// start or stopped on an error.
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum BleLine {
	/// A phone links by it now: the paired phone's name.
	Linked { phone: String },
	/// macOS has yet to answer the Bluetooth grant, which this page asks.
	Ask,
	/// The grant is refused.
	Refused,
	/// The computer's Bluetooth is off.
	Off,
	/// The adapter cannot serve phones (Windows).
	NoPeripheral,
	/// The computer has no Bluetooth.
	#[serde(rename = "none")]
	Missing,
	/// It could not start (the system's why stays in `helper.log`): the switch off and on tries once more.
	Failed,
	/// It served, then stopped on an error: the switch off and on starts it once more.
	Lost,
}

/// The line under the pairing code.
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum AddressLine {
	Ready {
		name: String,
		host: String,
	},
	/// Windows: the code shows once the firewall rule lets the phone in.
	Firewall {
		name: String,
		host: String,
	},
	/// Windows without the firewall rule while the Bluetooth link serves (since the review of 2026-10-17): the code
	/// shows beside the firewall card, and a phone pairs by Bluetooth now and on Wi-Fi once it can reach the PC.
	Ble {
		name: String,
		host: String,
	},
	/// No home network.
	None {
		name: String,
	},
	/// The LAN API could not listen.
	Lan {
		name: String,
		host: Option<String>,
		error: String,
	},
	/// The secrets' store could not be read at this start (`Helper::secrets_unread`): no code until the next start.
	Unread {
		name: String,
	},
}

/// The paired phone, its watch and the device that holds the link: the Phones page's line and the tray's.
#[derive(Serialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct PhoneLine {
	/// The paired phone's name; none before a phone pairs.
	pub name: Option<String>,
	pub watch: bool,
	pub holder: Option<Holder>,
}

/// The device that holds the link and gets the countdowns now.
#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Holder {
	Phone,
	Watch,
}

/// Windows: whether the firewall lets the phone in.
#[derive(Serialize, Debug, Clone, Copy, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FirewallCard {
	#[default]
	Open,
	/// No rule lets the phone in: the page offers the one-time elevated step.
	Rule,
	/// "Block all incoming connections" is on: the page says where to turn it off.
	Blocked,
}

/// Devices: the heart-rate strap and the walking pad.
#[derive(Serialize, Debug, Clone, Default, PartialEq)]
pub struct Devices {
	pub strap: Device,
	pub pad: Device,
}

/// A Bluetooth device: turned on, and what it is doing.
#[derive(Serialize, Debug, Clone, Default, PartialEq)]
pub struct Device {
	pub on: bool,
	pub line: Option<DeviceLine>,
}

/// A device's line.
#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum DeviceLine {
	/// Turned on, until the Bluetooth task says what it does.
	Starting,
	Searching,
	NotFound,
	/// Read now; `name` none for a device that gives none.
	Connected {
		name: Option<String>,
	},
	/// Bluetooth neither on nor off yet.
	Waiting,
	/// Bluetooth is off.
	Off,
	/// Bluetooth was refused (the switch turned off).
	Refused,
	Failed {
		error: String,
	},
	/// The walking pad while it is read: its belt and the day's walk.
	Walking {
		name: Option<String>,
		belt: Belt,
		walk: Walk,
	},
}

/// The pad's belt.
#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum Belt {
	/// No record has said its speed yet.
	Unknown,
	Stopped,
	/// Walking at this speed, in the player's road units ("3.2 km/h").
	Moving {
		speed: String,
	},
}

/// The day's walk with the quests, in the player's road units.
#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum Walk {
	/// No record carried a distance yet.
	Pending,
	/// The pad shares no distance.
	NoDistance,
	/// No walk today.
	None,
	Open {
		open: String,
	},
	Kept {
		kept: String,
	},
	Both {
		kept: String,
		open: String,
	},
	/// No game runs: the walk counts only while it does.
	Game,
	KeptGame {
		kept: String,
	},
}

/// What the feedback form shows (slice 24): the text's cap, what goes with a message, and how the last one went.
#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct FeedbackView {
	/// The text's cap in Unicode scalar values (questpump-core `feedback::MAX_CHARS`), which the text box holds to.
	pub max_chars: usize,
	/// QuestPump's version, which goes with every message.
	pub version: &'static str,
	/// The diagnostics its tick adds, exactly as they go: the system, and the screen reading's code word.
	pub os: String,
	pub capture: String,
	/// How the last message went or that one waits, also the tray menu's note; none before any.
	#[serde(skip)]
	pub note: Option<Note>,
	/// The note in words, the tray's (`desk::tray::note_line`), kept by `update`.
	pub note_line: Option<String>,
	/// How many messages wait to go (the outbox's and any held unkept), as of the last batch.
	pub waiting: usize,
}

impl Default for FeedbackView {
	fn default() -> Self {
		Self {
			max_chars: questpump_core::feedback::MAX_CHARS,
			version: env!("CARGO_PKG_VERSION"),
			os: String::new(),
			capture: String::new(),
			note: None,
			note_line: None,
			waiting: 0,
		}
	}
}

/// The feedback note, the tray's and the form's.
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum Note {
	/// The latest message went at this instant (unix ms).
	Sent { at: i64 },
	/// The latest was turned away.
	Refused { at: i64 },
	/// One that could not be kept could not go either; `game` for a game's.
	Lost { at: i64, game: bool },
	/// One waits for the round a minute on.
	Soon,
	/// One waits for the next try, every `minutes`.
	Later { minutes: u64 },
}

/// How the form's message went, the page's answer to `send_feedback`.
#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FeedbackState {
	/// The route took it.
	Sent,
	/// It waits in the outbox (no connection, the route busy) and goes by itself later.
	Queued,
	/// The route, or QuestPump's own check, refused it: it never goes.
	Refused,
	/// The outbox could not keep it (the disk full, the folder not writable) and its one try did not go: it is lost,
	/// and the form keeps the text.
	Unsaved,
}

/// The loop's answer to the form's Send: how it went, or why it never reached the loop.
pub type FeedbackReply = oneshot::Sender<Result<FeedbackState, String>>;

/// Asks the loop to quit: it writes a pending mirror first, then starts the copy moved into Applications that a second
/// launch asked to take over, if one did (`start`, `desk::launch::second`), or QuestPump again from its own bundle for
/// a restart asked on the socket (`again`, `update::start_updated`).
pub struct Quit {
	pub start: Option<PathBuf>,
	pub again: bool,
}

/// What the window asks the loop.
#[derive(Debug)]
pub enum WindowAsk {
	/// Start again, with the window open on This computer (macOS applies a Screen Recording grant only to a new
	/// process).
	Restart,
	/// A new pairing key: every pairing ends, and the page shows the new code.
	NewPairing,
	/// The heart-rate strap on or off.
	Strap(bool),
	/// The walking pad on or off.
	Pad(bool),
	/// The Phones page's Bluetooth link switch (slice 43).
	BleLink(bool),
	/// The Phones page is open (`page`), and the pairing code shows there or in the pairing window (`code`): the
	/// Bluetooth link advertises while the code shows, and on macOS asks for the Bluetooth grant on the page (slice
	/// 43).
	PhonesShown { page: bool, code: bool },
	/// The folder the player picked for the game.
	Folder(PathBuf),
	/// Windows: the one-time elevated firewall rule that lets the phone reach QuestPump.
	AllowNetwork,
	/// Windows: the one-time elevated grant that lets QuestPump write into the game folder.
	AllowFolder,
	/// Add QuestPump to the game, or Update now (`runtime/addon.rs`).
	Addon,
	/// This computer's Restart to update: QuestPump's own update installs now (`runtime/updates.rs`).
	Update,
	/// Start with the computer, or not.
	Autostart(bool),
	/// The feedback form's Send: the text as typed, whether its diagnostics go, and where the outcome goes.
	Feedback { text: String, diagnostics: bool, done: FeedbackReply },
	/// The desktop app's asks: an intent, the zoom from the app menu, the window's place.
	Desk(DeskAsk),
}

/// The view, shared between the runtime (which writes it), the commands and the tray, and the loop's quit and window
/// channels once it runs.
#[derive(Default)]
pub struct Shared {
	pub view: Mutex<View>,
	pub quit: Mutex<Option<UnboundedSender<Quit>>>,
	pub asks: Mutex<Option<UnboundedSender<WindowAsk>>>,
	/// The capture found the Screen Recording grant missing (the runtime keeps it): a quit asked for then, once the
	/// player pressed "Allow screen reading" (`computer.asked`), is taken for System Settings' "Quit & Reopen", and
	/// QuestPump starts again (`desk::menu`).
	pub screen_missing: AtomicBool,
}

impl Shared {
	/// Hands an ask to the loop once it runs (an app command from the local page needs no capability); false when it
	/// does not run yet or any more.
	pub fn ask(&self, ask: WindowAsk) -> bool {
		self.asks.lock().ok().and_then(|a| a.clone()).is_some_and(|tx| tx.send(ask).is_ok())
	}
}

/// The player's quit (the tray's Quit, the app menu's, the Dock's): `quit_for` with nothing to start after it.
pub fn quit(app: &AppHandle) {
	quit_for(app, None);
}

/// The player's quit, then `start` once this process has gone (the copy moved into Applications that a second launch
/// asked to take over): while a pump holds the window the page asks "Leave the pump?" first, with the window brought
/// forward, and its Leave quits (`desk/pump.rs` `pump_leave`, the ask's start with it) while its Stay drops both;
/// else the loop's quit at once.
pub fn quit_for(app: &AppHandle, start: Option<PathBuf>) {
	if app.state::<Desk>().guard.load(Ordering::Relaxed) {
		desk::windows::open_main(app, desk::windows::Landing::Here);
		update(app, |v| v.shell.leave = Some(Leave::after(v.shell.leave.as_ref(), Then::Quit, start)));
		return;
	}
	quit_now(app, start);
}

/// Quits through the loop when it runs, so a pending mirror is written, then starts `start` once this process has
/// gone; at once before the loop runs. Nothing asks first: the system's own quit (a logout, a restart, a shutdown,
/// System Settings' "Quit & Reopen") waits on no page.
pub fn quit_now(app: &AppHandle, start: Option<PathBuf>) {
	let sender = app.state::<Shared>().quit.lock().ok().and_then(|q| q.clone());
	match sender {
		Some(tx) if tx.send(Quit { start, again: false }).is_ok() => {}
		_ => app.exit(0),
	}
}

/// Changes the view: the topics that changed are staged under its lock, so their revs keep the order of the changes,
/// and go to the page once it is released; the tray is redrawn on the main thread then.
pub fn update(app: &AppHandle, change: impl FnOnce(&mut View)) {
	let publisher = &app.state::<Desk>().publisher;
	let mut staged = Staged::default();
	let (before, after) = {
		let shared = app.state::<Shared>();
		let Ok(mut view) = shared.view.lock() else { return };
		let before = view.tray();
		change(&mut view);
		view.computer.state = desk::tray::state(&view);
		view.computer.state_line = desk::tray::state_line(view.computer.state, &view.computer.watching);
		view.computer.tone = desk::tray::tone(view.shell.dot);
		view.phones.phone_line = desk::tray::phone_line(&view.phones.phone);
		view.feedback.note_line = view.feedback.note.as_ref().map(desk::tray::note_line);
		publisher.stage(&mut staged, desk::SHELL, &view.shell);
		publisher.stage(&mut staged, desk::COMPUTER, &view.computer);
		publisher.stage(&mut staged, desk::PHONES, &view.phones);
		publisher.stage(&mut staged, desk::DEVICES, &view.devices);
		publisher.stage(&mut staged, desk::FEEDBACK, &view.feedback);
		(before, view.tray())
	};
	publisher.send(&staged);
	if before != after {
		let handle = app.clone();
		let _ = app.run_on_main_thread(move || desk::tray::redraw(&handle, &before, &after));
	}
}

/// A copy of the view as it is now.
pub fn view(app: &AppHandle) -> View {
	app.state::<Shared>().view.lock().map(|v| v.clone()).unwrap_or_default()
}
