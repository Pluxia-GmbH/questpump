//! The helper's loop: capture events and socket commands go through questpump-core's `Session`; its effects become
//! socket lines, the saved rows and people, the tray's state and the mirror (debounced 2 s after strip rows, at once
//! after a reset, a catch-up, a pairing, the game's start or exit, and held while the addon leaves the world, at most
//! 10 s; a write due at once is never pushed back by a later debounce) with the helper's sign of life beside it
//! (`Alive.lua`: after every mirror write and every 10 s while the game runs, so the addon knows at each load that the
//! helper runs). The mirror, the sign of life, the settings, the people, the proof book and the body every app device
//! shares (`body.json`, slice 35) are written on one writer thread, in order, so an fsync or a rename the game blocks
//! never stalls the loop; `rows.jsonl` and `sheets.jsonl` take their appends on the loop (a line each, no fsync), and
//! so does a reset's compaction of both (an fsync, and on Windows a refused rename tried again for up to 5 s), which
//! comes from `/qp reset all` or a save taken in, never while a pump counts down. The game's saves and a mirror already
//! in a folder are read on the loop too. Before the first write into a folder, the `State.lua` already there and the
//! game's saves beside it are taken in, and a save again whenever the client writes it (a `/reload` or a logout seen by
//! its time, a new load on the strip, the game's exit: a lost data folder, a session played without the helper or out
//! of its sight, or a reset made then must not cost or keep the wrong rows). The install the session follows is the
//! mirror's folder (`Helper::addons`: the running game's when it holds QuestPump), and each install's mirror is written
//! in its own numbering of the characters with its own profiles (slice 26). Once the first mirror job took that install
//! in, or found none, the stores hold what this computer has (`stores_read`): the first five minutes decide from then
//! on whether the player is new (`desk::seams::Found`). A phone's app rows come with its syncs into their own store
//! (`app.jsonl`) and reach every mirror beside the game's, and every paired phone reads them back (`GET /v1/app_rows`,
//! slice 28). The LAN API (`lan.rs`) serves the phone and the watch on every address of the computer, bound once; the
//! computer's name, LAN address and `.local` name are read again every minute for the pairing code, every response's
//! helper block and, on Windows, the Bonjour record (`dnssd.rs`). The Bluetooth link serves the phones the same routes
//! while its switch is on and a phone is paired or the pairing code shows (`runtime/ble_link.rs`, `blelink`, slice 43).
//! The commands the strip's events make are queued for the device that holds the link, and the routes ask the loop for
//! what the session knows. The heart rate comes from a watch's syncs and, once the player turns it on, a Bluetooth
//! strap; every second the session gives out the heart-rate verdicts that are due. A walking pad the player turns on is
//! read on the same Bluetooth task (`bluetooth.rs`): its distance goes into each stretch row (`Session::pad`), its
//! readings and events onto the socket, which may set its speed or stop it; it never changes a pump. On Windows the LAN
//! API listens only once the firewall lets the phone in (`runtime/firewall.rs`), and a game folder Windows will not let
//! the helper write into gets the window's card (`runtime/folder.rs`, which also shows the folder, takes the one the
//! player picks and reads the move figures' state). A downloaded update installs once the game is closed
//! (`runtime/updates.rs`). A feedback message from the game or the window goes into its outbox and out to questpump.com
//! (`runtime/feedback.rs`), and so do the desktop app's crash reports while the player's switch lets them
//! (`runtime/crash.rs`, whose sweep runs first in the start, `sweep`) and its anonymous counts while Usage statistics
//! reads on (`runtime/counts.rs`; the switch and the privacy page, `runtime/usage.rs`), the outboxes sent alike
//! (`runtime/outbox.rs`). The games on the computer share the game's process snapshot each look, and their finishes
//! become the computer's match offers (`runtime/matches.rs`, slice 35). A paired phone's Pro proof rides its sync, and
//! the grants it gives go into `pro.json` on the writer thread with the account's token from the sync server
//! (`runtime/pro.rs`, slices 35 and 64). The addon's own engine
//! folds every row once at the start, off the loop, and takes the rows from elsewhere after every change the session
//! makes, behind one ordered door whose answers come back to the loop (`questpump-desk/src/engine.rs`, slice 35); a pump on the
//! computer keeps its clock in the loop too, its next instant one more arm of the select (`questpump-desk/src/pump.rs`), and a pump
//! the game offers on the strip ends it. Every failure stops only its own part: the mirror, the pairing and the LAN API
//! keep running without a capture, a strap, a pad, a game's feed or the engine.

use std::collections::HashSet;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use questpump_core::app_rows::{self, AppRows};
use questpump_core::body::Kept;
use questpump_core::codes::STRIP_VERSION;
use questpump_core::far::{self, FarRows};
use questpump_core::gatt::{MachineStatus, Treadmill};
use questpump_core::link::{self, Device, Sheet, Step, SyncRequest, Transport, Wow, WristEvent};
use questpump_core::mirror::{self, Sight, State};
use questpump_core::offers::{self, Standing};
use questpump_core::pair;
use questpump_core::proof::{Book, HrSource};
use questpump_core::protocol::{ErrorCode, Line, PadEventKind, SocketCommand};
use questpump_core::session::{
	BankPump, CatchupError, CatchupPump, Effects, MirrorDue, Now, PadDay, People, Session, Zone,
};
use questpump_core::sheets::{self, SheetLog};
use questpump_core::store::Store;
use questpump_core::strip::{DecodeError, SetupWord};
use questpump_desk::engine::Door;
use questpump_desk::platform::FoundGame;
use questpump_engine::Clock;
use tauri::{AppHandle, Manager as _};
use tokio::sync::{broadcast, mpsc, oneshot, watch};

use crate::blelink::Heard;
use crate::bluetooth::{self, Ble, Kind, PadCommand, Want};
use crate::capture::{Capture, Event};
use crate::desk::audio::Audio;
use crate::desk::launch::Launch;
use crate::desk::offers::Offering;
use crate::desk::reminders::Reminding;
use crate::desk::shell::{Out, Shell};
use crate::desk::windows::{self as desk_windows, Landing};
use crate::desk::{self, menu_pages};
use crate::elevate::Outcome;
use crate::game;
use crate::games::procs::Snapshot;
use crate::games::{self, Games};
use crate::lan::{self, Ask, Lan, Links, SyncAnswer};
use crate::pairing::Address;
use crate::secrets::{Keeper, RunSecrets};
use crate::settings::{self, Loaded, Region, Settings};
use crate::ui::{
	self, AddressLine, Belt, DeviceLine, Dot, Holder, Mirror, PhoneLine, Quit, Screen, Shared, Walk, WindowAsk,
};
use crate::{autostart, copy, dnssd, feeds, install, pairing, socket, units, update};

mod addon;
mod ble_link;
mod counts;
mod crash;
mod feedback;
mod firewall;
mod folder;
mod matches;
mod outbox;
mod pro;
mod sitting;
pub(crate) mod sync;
mod updates;
mod usage;

use addon::AddonState;
use ble_link::BleLinkState;
use counts::CountsState;
use crash::Crash;
use feedback::Feedback;
use firewall::Firewall;
use folder::Access;
pub(crate) use matches::arms_game;
pub(crate) use pro::{Pro, ProState};
use sync::SyncLink;

const DEBOUNCE: Duration = Duration::from_secs(2);
/// A batch that never pauses (a long hub) is written at the latest this long after its first row.
const DEBOUNCE_MAX: Duration = Duration::from_secs(10);
const LEAVING_MAX: Duration = Duration::from_secs(10);
/// While the game runs, the helper's sign of life (`Alive.lua`) is written again this often, so whenever the addon
/// loads (a login, a `/reload`, another character) it reads one written shortly before and knows the helper runs. The
/// check runs on the 2-s game tick, so the writes come 10 to 12 s apart; the addon's `C.helper_alive_slack_s`, 30 s,
/// covers that. (Until 2026-10-03 the pairing block's `seen_at` said it, measured against the client's session clock,
/// which starts at a point no source names: a helper that ran read as not running twice.)
const ALIVE_EVERY_S: i64 = 10;
/// How old the sign of life may be when a load reads it and still say QuestPump runs: the addon's
/// `C.helper_alive_slack_s`.
const ALIVE_SLACK: Duration = Duration::from_secs(30);
/// How long anchored frames may fail to read before the window says why: a capture in the middle of a repaint fails
/// now and then, a strip of another version or with colours the decoder cannot tell apart fails every time.
const UNREADABLE_AFTER: Duration = Duration::from_secs(5);
/// A gap between two frames that did not read after which the count of `UNREADABLE_AFTER` starts again.
const UNREADABLE_GAP: Duration = Duration::from_secs(2);
/// How long after the last frame that read the screen stays on Reading while the strip is out of view: a strip in view
/// reads at least each second (the addon's heartbeat), so a corner covered for a moment (a menu, a window passing over
/// it) never turns the state over, and the state leaves Reading only for a strip gone this long (2026-10-10, after
/// helper.log flipped between Out and Reading several times a minute).
const OUT_AFTER: Duration = Duration::from_secs(5);
/// How long the strip may stay out of sight in a running game before the window asks whether QuestPump is turned on.
const UNSEEN_AFTER: Duration = Duration::from_mins(1);
/// How often the pairing block's `linked_at` moves on while a device holds the link or posts (the game reads it in
/// days).
const LINKED_EVERY_S: i64 = 3_600;
/// A paired phone that has not linked for this long (`linked_at`, else `phone_since`): the Phones page shows the code
/// again, since its app may be gone or unpaired out of the helper's reach (since 2026-10-09).
const QUIET_AFTER_S: i64 = 14 * 86_400;
/// The mirror's error while no game folder is known, so the log and the socket say it once.
const NO_FOLDER: &str = "no folder"; // copy-lint: not shown (an error's key)
/// How often the game is looked for while it is closed: the install followed checked again (deleted, moved, QuestPump
/// removed) and, while none is known, the Agent's and the default folders searched. The game folder known without
/// QuestPump is looked into at every 2-s look instead (one `stat`), so QuestPump installed there gets the mirror and
/// the sign of life within seconds, before the client's first start.
const LOOK_FOR_GAME: Duration = Duration::from_secs(30);
/// How long after an Unpair a watch's sync names no watch (`Helper::unpaired_at`): past a held sync's 5 s and its
/// client's 20-s read, with room.
const WATCH_LATE_S: i64 = 60;
/// How often a feedback message, a crash report or a counts' batch that waits is sent again (and the first try, at the
/// start).
const OUTBOX_EVERY: Duration = Duration::from_mins(10);
/// An outbox's next round after the route slowed a batch (a 429, or a 204 while more waited): each route lets one entry
/// a minute through per client address.
const OUTBOX_SOON: Duration = Duration::from_secs(61);

/// The loop, supervised (`crate::keep`: its panic restarts QuestPump once); a start that fails says so on the tray and
/// This computer, and in the log.
pub fn spawn(app: AppHandle, launch: Launch, swept: anyhow::Result<Swept>) {
	let handle = app.clone();
	let looping = tauri::async_runtime::spawn(async move {
		let started = match swept {
			Ok(swept) => Box::pin(run(app.clone(), launch, swept)).await,
			Err(e) => Err(e),
		};
		if let Err(e) = started {
			log::error!("the helper could not start: {e:#}");
			ui::update(&app, |v| {
				v.shell.dot = Dot::Stopped;
				v.computer.screen = Screen::Failed { error: e.to_string() };
			});
		}
	});
	crate::keep::watch(&handle, crate::keep::Part::Loop, looping);
}

/// What the start takes first, before anything else it sets up (`lib.rs`'s `run()`, right after the panic hook and the
/// start's log; contract/crash-api.yaml, "First in the start"): the data folder, the settings, the region while a
/// switch is not answered, the outboxes' client and the news their batches report on, and the crash reports that
/// waited, already on their way while Send crash reports reads on (`Crash::sweep`); whether the run before crashed.
pub(crate) struct Swept {
	dir: PathBuf,
	settings: Settings,
	loaded: Loaded,
	/// The secrets' keeper the writer thread takes over (`secrets`).
	keeper: Keeper,
	region: Region,
	client: Option<reqwest::Client>,
	posts: (mpsc::UnboundedSender<outbox::News>, mpsc::UnboundedReceiver<outbox::News>),
	crash: Crash,
	crashed: bool,
}

/// The crash sweep, first in the start: the data folder made, the settings read with their secrets, the region read
/// while a switch is not answered, and the crash reports that wait swept (`Swept`), the marks it moved, and secrets the
/// file still held (`Keeper::moving`), saved at once. Its batch, when one goes, runs on the async runtime before the
/// app is built; the loop takes over its news.
///
/// # Errors
/// No data folder.
pub(crate) fn sweep() -> anyhow::Result<Swept> {
	let dir = data_dir()?;
	let (mut settings, loaded, mut keeper) = settings::load(&dir);
	let asks = settings.crash.choice.send.is_none() || settings.usage.send.is_none();
	let region = if asks && crate::crash::opt_in() { Region::OptIn } else { Region::Other };
	let client = crate::feedback::client()
		.inspect_err(|e| log::warn!("outbox: no HTTPS client ({e}); feedback, crash reports and counts wait"))
		.ok();
	let (posts_tx, posts_rx) = mpsc::unbounded_channel::<outbox::News>();
	let mut crash = Crash::new(&dir, client.clone(), posts_tx.clone());
	let marks = settings.crash.clone();
	let crashed = crash.sweep(&dir, &mut settings.crash, region);
	// the marks the sweep moved, kept before the rest of the start: a start that crashes before its own save never
	// sends the same reports again; and the secrets out of the file into their store
	if settings.crash != marks || keeper.moving() {
		settings::save_now(&settings, &dir, &mut keeper);
	}
	Ok(Swept { dir, settings, loaded, keeper, region, client, posts: (posts_tx, posts_rx), crash, crashed })
}

/// The data folder: `~/Library/Application Support/QuestPump` on macOS, `%LOCALAPPDATA%\QuestPump` on Windows.
pub fn data_dir() -> anyhow::Result<PathBuf> {
	let dir = data_home().ok_or_else(|| anyhow::anyhow!("no data folder"))?;
	std::fs::create_dir_all(&dir)?;
	Ok(dir)
}

/// Where the data folder is, made or not (the panic hook's, which runs before the loop made it).
pub fn data_home() -> Option<PathBuf> {
	Some(dirs::data_local_dir()?.join("QuestPump"))
}

/// An install's key in the people (slice 26): its AddOns folder as the file system names it (the running client's path
/// and a discovered or picked one may spell the same folder apart), on Windows without the verbatim prefix
/// `canonicalize` adds (`\\?\C:\…`, `\\?\UNC\server\share\…`), so one folder has one key. None while the folder does
/// not resolve: another spelling would be a second install of the same folder, so the helper follows, takes in and
/// writes no install there until it does.
fn install_key(addons: &Path) -> Option<String> {
	let key = std::fs::canonicalize(addons).ok()?.to_string_lossy().into_owned();
	if cfg!(windows) {
		if let Some(share) = key.strip_prefix(r"\\?\UNC\") {
			return Some(format!(r"\\{share}"));
		}
		if let Some(local) = key.strip_prefix(r"\\?\") {
			return Some(local.to_owned());
		}
	}
	Some(key)
}

pub(crate) fn now() -> Now {
	let t = time::OffsetDateTime::now_utc();
	let format =
		time::macros::format_description!("[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:3]Z");
	Now { unix: t.unix_timestamp(), rfc3339: t.format(&format).unwrap_or_default() }
}

/// The computer's offset from UTC at a unix second, in seconds, as its zone gives it for that instant (summer time
/// included); UTC when the system cannot tell. The session's `Zone`, here and in the stand-in helper.
#[must_use]
pub fn local_offset_at(t: i64) -> i64 {
	time::OffsetDateTime::from_unix_timestamp(t)
		.ok()
		.and_then(|at| time::UtcOffset::local_offset_at(at).ok())
		.map_or(0, |o| i64::from(o.whole_seconds()))
}

/// The local minute of the day at unix second `t` (quiet hours').
pub(crate) fn minute_of_day(t: i64) -> u16 {
	u16::try_from((t + local_offset_at(t)).rem_euclid(86_400) / 60).unwrap_or(0)
}

/// A file the writer thread writes.
pub(crate) enum Job {
	Mirror {
		addons: PathBuf,
		text: String,
	},
	File {
		dir: PathBuf,
		name: &'static str,
		bytes: Vec<u8>,
	},
	/// settings.json from the settings' JSON, through the secrets' keeper (`secrets::Keeper::bytes`).
	Settings {
		dir: PathBuf,
		file: serde_json::Value,
	},
	/// The helper's sign of life beside the mirror in `addons`, at the helper's clock `unix`.
	Alive {
		addons: PathBuf,
		unix: i64,
	},
	/// Answered once every job asked before it is done.
	Flush(oneshot::Sender<()>),
}

/// What a write came to, back on the loop.
enum Written {
	Mirror(PathBuf, std::io::Result<()>),
	File(&'static str, std::io::Result<()>),
	Alive(PathBuf, std::io::Result<()>),
}

/// One thread writes every file, in the order asked, and holds the secrets' keeper, so a secret is in its store before
/// the settings.json that leaves it out; it runs supervised (`crate::keep`: its panic restarts QuestPump once).
fn writer(
	app: &AppHandle,
	results: mpsc::UnboundedSender<Written>,
	mut keeper: Keeper,
) -> std::sync::mpsc::Sender<Job> {
	let (tx, rx) = std::sync::mpsc::channel::<Job>();
	let spawned = std::thread::Builder::new().name("writer".into()).spawn(move || {
		for job in rx {
			let result = match job {
				Job::Mirror { addons, text } => Written::Mirror(addons.clone(), mirror::write(&addons, &text)),
				Job::File { dir, name, bytes } => Written::File(name, mirror::write_atomic(&dir, name, &bytes)),
				Job::Settings { dir, file } => Written::File(
					settings::FILE,
					keeper
						.bytes(file)
						.map_err(std::io::Error::from)
						.and_then(|bytes| mirror::write_atomic(&dir, settings::FILE, &bytes)),
				),
				Job::Alive { addons, unix } => {
					let result = mirror::write_alive(&addons, unix);
					Written::Alive(addons, result)
				}
				Job::Flush(done) => {
					let _ = done.send(());
					continue;
				}
			};
			let _ = results.send(result);
		}
	});
	match spawned {
		Ok(thread) => crate::keep::watch_thread(app, crate::keep::Part::Writer, thread),
		Err(e) => log::error!("writer: no thread: {e}"),
	}
	tx
}

/// What a one-time elevated step came to (Windows), back on the loop.
pub(crate) enum Elevated {
	Network(Outcome),
	Folder(Outcome),
	/// A game's folder for its setup file (the Games on this computer page), by the game's key.
	Game(String, Outcome),
}

/// What the screen reading's line knows beside the tray's dot.
#[derive(Default)]
pub(crate) struct Sighting {
	/// The line, so a new line on the same dot (the HDR hint) is shown and a repeat is not.
	pub(crate) line: Screen,
	/// The capture's display is an HDR output: a strip not in view says how to turn HDR off.
	hdr: bool,
	/// Anchored frames that did not read since the last that did: when the first and the latest came.
	unreadable: Option<(Instant, Instant)>,
	/// The capture backend of the stream that runs (`capture::Event::Display`'s, the counts' `capture_day`).
	pub(crate) backend: &'static str,
	/// When the strip last read, or the game started: a strip out of sight for a minute may be QuestPump turned off.
	read_at: Option<Instant>,
	/// A frame of the running client read since it started: QuestPump is on in it, so its strip out of sight later is
	/// its window covered or a menu, never QuestPump turned off (`Desk::wow_unseen`), and this computer's state says
	/// the game is out of view, never that QuestPump is waited for (`ui::RunningGame::read`). False again at a game
	/// exit and for another client process (`Helper::on_game_tick`).
	pub(crate) read: bool,
}

impl Sighting {
	/// The strip read within `OUT_AFTER`: a strip out of sight for less keeps the screen on Reading.
	fn read_lately(&self) -> bool {
		self.read_at.is_some_and(|t| t.elapsed() < OUT_AFTER)
	}

	/// Anchored frames that do not read are still coming (`UNREADABLE_GAP`): the strip is in view, and the unreadable
	/// rule says what the screen shows.
	fn unreadable_now(&self) -> bool {
		self.unreadable.is_some_and(|(_, last)| last.elapsed() <= UNREADABLE_GAP)
	}
}

/// The walking pad while the Bluetooth task reads it.
pub(crate) struct PadRead {
	/// Its name, as the task gives it; none for a pad that gives none.
	name: Option<String>,
	/// The page's line now, so a redraw that reads the same draws nothing (`Helper::show_pad`).
	shown: DeviceLine,
	/// The belt's speed in the last record that had one, in km/h.
	kmh: Option<f64>,
	/// The belt is stopped: the last record's speed was 0, or the pad's status said since that it stopped, paused or reset.
	standing: bool,
	/// The connection's records so far, and whether one carried the distance: FTMS makes Total Distance optional, so a
	/// pad whose first `NO_DISTANCE_AFTER` records carry none is read for its speed alone.
	records: u32,
	distance: bool,
}

/// The records after which a pad that sent no distance is said to share none (a record split in two at the connect
/// may come without it).
const NO_DISTANCE_AFTER: u32 = 5;

/// The LAN API's listeners, on every address.
pub(crate) struct Served {
	stop: watch::Sender<bool>,
}

/// The loop's state; the desktop app's areas add their handlers as `impl Helper` blocks in `desk/`.
pub(crate) struct Helper {
	pub(crate) app: AppHandle,
	pub(crate) dir: PathBuf,
	/// The desk: its areas' state, the stores and the settings, on the desktop app as its platform (`desk::shell`).
	pub(crate) desk: questpump_desk::Desk<Shell>,
	/// Whether this run's secrets reach the next start (`secrets::Keeper::run`): a run whose store could not be read
	/// while settings.json held none (`RunSecrets::Unread`) shows no pairing code, answers no phone (no LAN listener, no
	/// Bluetooth link) and refuses Sync and every way into an account (`Helper::secrets_unread`).
	pub(crate) secrets: RunSecrets,
	pub(crate) queue: Links,
	/// The routes' state.
	pub(crate) lan: Arc<Lan>,
	pub(crate) served: Option<Served>,
	pub(crate) lan_error: Option<String>,
	/// The device that held the link at the last look, for the window.
	pub(crate) linked: Option<Device>,
	/// The device, client tag and transport that held the link at the last look, for the log's holder line
	/// (`on_link_tick`).
	pub(crate) holding: Option<(Device, String, Transport)>,
	/// The computer's name, LAN address and `.local` name at the last look.
	pub(crate) address: Address,
	/// The Bonjour record (Windows) while the LAN API listens.
	pub(crate) advert: dnssd::Advert,
	pub(crate) lines: broadcast::Sender<String>,
	pub(crate) writes: std::sync::mpsc::Sender<Job>,
	pub(crate) due: Option<Instant>,
	/// When the helper's sign of life (`Alive.lua`) was last asked to be written, on the computer's clock: an `Instant`
	/// stands still while a Mac sleeps, and the sign of life must be fresh at the first load after it wakes.
	pub(crate) alive_at: Option<i64>,
	/// Why its last write failed, so the log says it once.
	pub(crate) alive_error: Option<String>,
	/// The first row of the batch the debounce is waiting out.
	pub(crate) batch_since: Option<Instant>,
	pub(crate) held_until: Option<Instant>,
	/// The capture stopped for want of the Screen Recording grant; the window asks for it once a device is paired.
	pub(crate) grant_missing: bool,
	/// `without_addon` is a folder the player picked (`runtime/folder.rs` `picked`): it stays while it is a game, and a
	/// first install goes there first (`first_target`); one found is ranked again at each look.
	pub(crate) without_addon_picked: bool,
	/// When the game was last looked for while it is closed (`look_for_game`, every `LOOK_FOR_GAME`).
	pub(crate) looked_for_game: Option<Instant>,
	/// The folder the last mirror went into.
	pub(crate) written: Option<PathBuf>,
	/// The folders whose `State.lua` was taken in.
	pub(crate) taken_in: HashSet<PathBuf>,
	/// When each of the game's saves was read, so one the client writes again comes in at the next look.
	pub(crate) saves_read: mirror::SavesRead,
	/// The setup word the strip carried last, with its load (since 2026-10-13): the pairing's step reads it while that
	/// load is the one the session follows (`setup_word`).
	pub(crate) setup: Option<(u16, SetupWord)>,
	/// The running client (its process) whose addon load may have come before the mirror's pairing block or sign of
	/// life: it ran when this run started while the sign of life in its folder was stale (`alive_fresh`), or a phone
	/// paired while it ran. Until its strip carries a setup word the pairing's step is `either`; it goes when the strip
	/// shows a later load, that client writes its save (a /reload, a logout) or another client or none runs, since a
	/// load that starts later reads both. Kept in memory alone: a run that starts within the addon's slack of the last
	/// run's sign of life (a reinstall, an update or the grant's restart) marks no client, one the last run had marked
	/// included (its load came before that run, or a phone paired while it ran), whose step is then `none` until its
	/// strip carries a setup word.
	pub(crate) loaded_before: Option<u32>,
	/// The pairing's step as the Phones page, This computer and the welcome show it (`show_next`), published on a
	/// change.
	pub(crate) next: Option<Step>,
	/// The app installs (the sync's `client`) that unpaired in this run, until a phone pairs again: a request of theirs
	/// still on its way names no phone, links nothing and brings no Pro proof (`device_synced`).
	pub(crate) unpaired: HashSet<String>,
	/// When a phone last unpaired (unix seconds): for `WATCH_LATE_S` after it a watch's sync names no watch, since its
	/// watch syncs under an install id of its own that `unpaired` never holds (a request of the old pairing still on
	/// its way); a watch of a phone paired since is named at a later sync.
	pub(crate) unpaired_at: Option<i64>,
	/// A folder's `State.lua` could not be read: no write replaces it unread; the write is tried again shortly.
	pub(crate) read_retry: bool,
	/// The files and folders that did not read at the last try, so the log says each once until it reads.
	pub(crate) unread: HashSet<PathBuf>,
	pub(crate) mirror_error: Option<String>,
	/// The Bluetooth task while the strap or the walking pad is on, and where it reports.
	pub(crate) ble: Option<bluetooth::Task>,
	pub(crate) bles: mpsc::UnboundedSender<Ble>,
	/// The walking pad the task reads now, for the window's line; none while it is not connected.
	pub(crate) pad_read: Option<PadRead>,
	/// The Bluetooth link to the phones (slice 43).
	pub(crate) ble_link: BleLinkState,
	/// Whether the LAN API may listen (Windows: once the firewall lets the phone in).
	pub(crate) firewall: Firewall,
	/// Why Windows refused the last mirror write, for the window's card.
	pub(crate) access: Access,
	/// The folder the window shows.
	pub(crate) shown_folder: Option<PathBuf>,
	/// A downloaded update, installed once nothing holds it back (`runtime/updates.rs`).
	pub(crate) update: Option<updates::Pending>,
	/// Wakes the update checks (`update::spawn`): the game closed.
	pub(crate) update_checks: feeds::Checks,
	/// A quit asked on the socket, handed to the loop's quit at the first look nothing holds it (`runtime/updates.rs`
	/// `socket_quit_due`); `true` for a restart.
	pub(crate) socket_quit: Option<bool>,
	/// Where the elevated steps report.
	pub(crate) elevated: mpsc::UnboundedSender<Elevated>,
	/// The feedback outbox and its sending.
	pub(crate) feedback: Feedback,
	/// The crash reports: what waits for the player's answer, and their outbox.
	pub(crate) crash: Crash,
	/// Where the computer is as the switches' default reads it (contract/crash-api.yaml): read at the start while a
	/// switch is not answered, never stored.
	pub(crate) region: Region,
	/// The anonymous counts: what is kept between batches, and their outbox.
	pub(crate) counts: CountsState,
	/// The desktop's side of the offers: the card, the Games page's picker, the tray's offer items (slice 35).
	pub(crate) offering: Offering,
	/// The desktop's reminders: the one whose card stands, the offer, the tray's streak line.
	pub(crate) reminding: Reminding,
	/// The addon in the game: its feed, the installs QuestPump made and what it does to them (`runtime/addon.rs`).
	pub(crate) addon: AddonState,
	/// The Backup and sync page's file part (`desk/menu_pages/backup.rs`).
	pub(crate) backup: menu_pages::backup::State,
}

impl Helper {
	/// A DHCP change or a rename: what the QR code, every response and the Bonjour record carry; and the Pro grants
	/// that ended.
	async fn on_pair_tick(&mut self) {
		self.address = find_address().await;
		self.desk.p.sync.name.clone_from(&self.address.name);
		self.serve_lan().await;
		self.on_pro_tick();
		self.desk.log_day();
		// the game day's turn sends the day before
		self.counts_tick();
		// a phone quiet for two weeks: the Phones page shows the code again
		self.show_phone();
	}

	fn emit(&self, line: &Line) {
		match line.to_json(&now().rfc3339) {
			Ok(text) => {
				let _ = self.lines.send(text);
			}
			Err(e) => log::warn!("a line did not serialise: {e}"),
		}
	}

	/// The mirror is due at `at`, or sooner if it is due already.
	fn due_by(&mut self, at: Instant) {
		self.due = Some(self.due.map_or(at, |d| d.min(at)));
	}

	/// The helper's sign of life goes into the folder the mirror was last written into, after each mirror write there.
	fn beat(&mut self) {
		if let Some(addons) = self.written.clone() {
			let unix = now().unix;
			self.alive_at = Some(unix);
			let _ = self.writes.send(Job::Alive { addons, unix });
		}
	}

	/// A game runs whose folder holds no QuestPump: no strip can show there.
	fn game_lacks_addon(&self) -> bool {
		self.desk.p.game.is_some() && self.desk.p.game_addons.is_none()
	}

	/// The sign of life again while the game runs, into the game's own folder: one the mirror has not reached yet (its
	/// write there failed, or none was made) gets the mirror again instead, whose write brings the sign of life.
	fn beat_for_game(&mut self) {
		let Some(addons) = self.desk.p.game_addons.clone() else { return };
		if self.written.as_ref() == Some(&addons) {
			self.beat();
		} else {
			self.alive_at = Some(now().unix);
			self.due_by(Instant::now());
		}
	}

	/// The debounce: each row of a batch pushes the write back to 2 s after it (10 s after the batch's first at most),
	/// and never delays a write that is due at once.
	fn due_soon(&mut self) {
		let now = Instant::now();
		let since = *self.batch_since.get_or_insert(now);
		let at = (now + DEBOUNCE).min(since + DEBOUNCE_MAX);
		self.due = Some(match self.due {
			Some(d) if d <= now => d,
			_ => at,
		});
	}

	fn save_file(&self, name: &'static str, bytes: Vec<u8>) {
		let _ = self.writes.send(Job::File { dir: self.dir.clone(), name, bytes });
	}

	/// The effects of rows the session took (`Effects`), wherever they came from: the desk takes them
	/// (`questpump_desk::Desk::apply`), and what it asked of the loop runs after (`drain`).
	pub(crate) fn apply(&mut self, fx: Effects) {
		self.desk.apply(fx);
		self.drain();
	}

	/// What the desk asked that only the loop does (`desk::shell::Out`), in the order it was asked, until none is left
	/// (what one of them asks the desk in turn included).
	pub(crate) fn drain(&mut self) {
		loop {
			let out = std::mem::take(&mut self.desk.p.out);
			if out.is_empty() {
				return;
			}
			for item in out {
				match item {
					Out::Effects(fx) => self.apply_shell(*fx),
					Out::MirrorSoon => self.due_soon(),
					Out::Backup => self.publish_backup(),
					Out::OffersPaused => self.offers_repaint(),
					Out::OfferStarted(offer) => self.offer_answered(&offer),
					Out::OfferTaken(offer) => self.offer_taken(&offer),
					Out::Board { board, game, resting } => self.reminders_board(board, game, resting),
					Out::ReminderOfferDue => self.reminder_offer_due(),
					Out::TreeCard(refused) => self.counts_tree_card(refused.as_deref()),
					Out::TreeStart(kind) => self.counts_tree_start(&kind),
					Out::Then(then) => then(self),
				}
			}
		}
	}

	/// The loop's part of the effects of rows the session took, after the desk took its own (the game's body, the
	/// people's and the proof book's files, the engine, the pages, the sync's look): the lines to the socket, the
	/// devices' commands, the logs, the game's feedback, a new load's saves, `/qp ping`, a suspend, the mirror's due
	/// time, the counts.
	fn apply_shell(&mut self, fx: Effects) {
		for line in &fx.lines {
			self.emit(line);
		}
		for command in fx.commands {
			if let Some(queued) = self.queue.queue(command, lan::now_ms()) {
				self.emit(&Line::Command { command: queued });
			}
		}
		for e in &fx.store_errors {
			log::warn!("{e}");
		}
		if fx.untimely > 0 {
			log::warn!("{} rows dated before QuestPump or too far past this clock left out", fx.untimely);
		}
		for e in &fx.sheet_errors {
			log::warn!("{}: {e}", sheets::FILE);
		}
		for e in &fx.blob_errors {
			log::warn!("a strip payload did not decode or check out: {e}");
		}
		// where the pump's commands went: the device that held the link as they were queued and its transport (since
		// 2026-10-14; the transport since slice 43), `→ ios (ble)`
		if let Some(line) = &fx.offered {
			let to = self.queue.holder_via(lan::now_ms()).map_or_else(
				|| "no device holds the link".to_owned(), // copy-lint: not shown (a log line)
				|(device, transport)| format!("{} ({})", device.name(), transport.name()),
			);
			log::info!("{line} → {to}");
			// a pump runs: the Bluetooth link's centrals at low latency
			self.ble_busy();
		}
		if let Some(message) = &fx.feedback {
			self.game_feedback(message);
		}
		if fx.feedback_dropped {
			self.game_feedback_dropped();
		}
		if fx.load_ended {
			// a new load read the pairing block and the sign of life the mirror carried at its start
			self.loaded_before = None;
			// the client wrote its save between the two loads: the rows that never reached the strip come in (an
			// import, a restore, a row still queued at the /reload), and reach the mirror of the load after
			if let Some(addons) = self.desk.p.game.as_ref().map(|g| g.addons.clone()) {
				self.take_in_saves(&addons);
			}
		}
		if fx.pinged {
			// `/qp ping`: answered out loud (its take) and on the screen reading's line, which keeps it until the reading
			// changes
			self.play_ping();
			let at = lan::now_ms();
			ui::update(&self.app, |v| v.computer.ping_at = Some(at));
		}
		match fx.suspend {
			Some(true) => self.held_until = Some(Instant::now() + LEAVING_MAX),
			Some(false) => self.held_until = None,
			None => {}
		}
		match fx.mirror {
			Some(MirrorDue::Now) => self.due_by(Instant::now()),
			Some(MirrorDue::Soon) => self.due_soon(),
			None => {}
		}
		// the counts look at the rows the stores took
		self.counts_rows();
	}

	/// The capture's state on the tray and the screen reading's line, and as a `status` line whenever its word changes.
	fn screen(&mut self, dot: Dot, line: Screen) {
		if self.desk.p.screen == dot && self.desk.p.sighting.line == line {
			return; // a strip out of sight arrives with every frame the capture reads
		}
		if status(dot) != status(self.desk.p.screen) {
			self.emit(&Line::Status { screen: Some(status(dot).into()), hint: None, hr: None, pad: None });
		}
		// the pairing block says whether the helper reads the game (`sight`): the game reads it at its next load
		if (dot == Dot::Stopped) != (self.desk.p.screen == Dot::Stopped) && self.grant_wanted() {
			self.due_soon();
		}
		// each change of what the screen reading shows goes into the log: a strip never read is found from it
		log::info!("screen: {line:?}");
		self.counts_capture(capture_state(dot, &line));
		self.desk.p.screen = dot;
		self.desk.p.sighting.line = line.clone();
		let capture = self.capture_word();
		let read = self.desk.p.sighting.read;
		ui::update(&self.app, |v| {
			v.shell.dot = dot;
			v.computer.screen = line;
			if let Some(game) = &mut v.computer.game {
				game.read = read;
			}
			v.computer.ping_at = None;
			capture.clone_into(&mut v.feedback.capture);
		});
	}

	/// The capture's state as the feedback form's diagnostics name it: the `status` line's word, or the `error` line's
	/// `screen_permission` while the Screen Recording grant is missing.
	fn capture_word(&self) -> &'static str {
		if self.grant_missing { "screen_permission" } else { status(self.desk.p.screen) }
	}

	fn on_capture(&mut self, event: Event) {
		match event {
			Event::Frame(frame, shown) => {
				if self.grant_missing {
					self.grant_missing = false;
					self.app.state::<Shared>().screen_missing.store(false, Ordering::Relaxed);
					ui::update(&self.app, |v| v.computer.permission = false);
				}
				// the load's setup word, which the pairing's step follows (`step`); a frame without one leaves the
				// last, since one load is one addon's (an older addon's never carries one)
				if let Some(word) = frame.setup() {
					self.setup = Some((frame.load_id, word));
				}
				self.desk.p.sighting.unreadable = None;
				self.desk.p.sighting.read_at = Some(Instant::now());
				if !self.desk.p.sighting.read {
					// the game read in this run: the game and the app in use today
					self.counts_active(questpump_core::telemetry::Source::Game);
					self.counts_active(questpump_core::telemetry::Source::Desktop);
				}
				self.desk.p.sighting.read = true;
				self.screen(Dot::Live, Screen::Reading);
				let fx = self.desk.session.on_frame(&frame, shown, &now());
				self.apply(fx);
			}
			Event::NoStrip => {
				self.desk.session.lost_sight();
				// a strip read within `OUT_AFTER` keeps Reading; the 2-s look ends it for a stream that sent this one
				// frame and stopped (`on_game_tick`)
				let reading = self.desk.p.screen == Dot::Live && self.desk.p.sighting.read_lately();
				if !reading && !matches!(self.desk.p.screen, Dot::Stopped | Dot::Off) && self.desk.p.game.is_some() {
					let line = self.out_of_sight();
					self.screen(Dot::NoStrip, line);
				}
			}
			Event::Unreadable(why) => {
				// the strip repaints every second, so a strip that does not read fails at least that often; a gap
				// longer than two seconds after a lone failed capture starts the count again
				let now = Instant::now();
				let since = match self.desk.p.sighting.unreadable {
					Some((first, last)) if now.duration_since(last) <= UNREADABLE_GAP => first,
					_ => now,
				};
				self.desk.p.sighting.unreadable = Some((since, now));
				if now.duration_since(since) >= UNREADABLE_AFTER && self.desk.p.game.is_some() {
					self.unreadable(&why);
				}
			}
			Event::Display { name, scale, hdr, backend } => {
				log::info!("capture: {name}, {scale}x{}", if hdr { ", HDR" } else { "" });
				self.desk.p.sighting.hdr = hdr;
				self.desk.p.sighting.backend = backend;
				self.desk.session.lost_sight(); // a stream starting: its first frame shows a paint of any age
				// a stream that starts again after a stop, or a strip that did not read: the new one is looked at afresh
				self.desk.p.sighting.unreadable = None;
				if self.desk.p.screen == Dot::Stopped && !self.grant_missing && self.desk.p.game.is_some() {
					// a game read in this run is out of view until its next frame reads, one not read yet waited for
					let line = if self.desk.p.sighting.read { self.out_of_sight() } else { Screen::Waiting };
					self.screen(Dot::NoStrip, line);
				}
			}
			Event::Error(code, detail) => {
				self.desk.session.lost_sight();
				log::warn!("capture stopped ({code:?}): {detail}");
				self.emit(&Line::Error { code, detail });
				if code == ErrorCode::ScreenPermission {
					self.counts_capture("denied");
					self.grant_missing = true;
					self.app.state::<Shared>().screen_missing.store(true, Ordering::Relaxed);
					self.show_grant();
				} else {
					// the macOS capture starts its stream again by itself; the Windows one tries its backends again when
					// the game starts again: the page's line says which
					self.screen(Dot::Stopped, Screen::Stopped);
				}
			}
		}
	}

	/// The screen line while the strip is out of sight in a running game: QuestPump missing from that game's folder, a
	/// strip unseen for a minute (QuestPump may be turned off), else a passing reason (on an HDR display, how to turn it
	/// off for the game).
	fn out_of_sight(&self) -> Screen {
		if self.game_lacks_addon() {
			return Screen::NoAddon;
		}
		// a minute out of sight from a start that never read may be QuestPump turned off in the game; a game read in this
		// run of the helper is out of view, however long (`Sighting::read`), so the window never asks after the AddOns
		// list beside "The game is out of view"
		if !self.desk.p.sighting.read
			&& !self.desk.p.sighting.hdr
			&& self.desk.p.sighting.read_at.is_some_and(|t| t.elapsed() >= UNSEEN_AFTER)
		{
			return Screen::Unseen;
		}
		Screen::Out { hdr: self.desk.p.sighting.hdr }
	}

	/// Anchored frames have not read for `UNREADABLE_AFTER`: the window says why and what to do, the tray turns red,
	/// and a `status` line carries the decoder's reason as its hint.
	fn unreadable(&mut self, why: &DecodeError) {
		let line = match why {
			DecodeError::Version(v) if *v > STRIP_VERSION => Screen::Newer,
			DecodeError::Version(_) => Screen::Older,
			_ => Screen::Colours { hdr: self.desk.p.sighting.hdr },
		};
		if self.desk.p.sighting.line != line {
			self.emit(&Line::Status { screen: None, hint: Some(why.to_string()), hr: None, pad: None });
		}
		self.screen(Dot::Stopped, line);
	}

	fn on_command(&mut self, cmd: SocketCommand) {
		match cmd {
			SocketCommand::Quit => self.socket_quit(false),
			// a restart that could not start QuestPump again would only quit (`update::starts_again`)
			SocketCommand::Restart if !update::starts_again() => {
				log::warn!("restart: QuestPump starts itself again only from its macOS app bundle");
			}
			SocketCommand::Restart => self.socket_quit(true),
			SocketCommand::DisplayLag { ms } => {
				log::info!("display lag: {ms} ms");
				self.desk.settings.display_lag_ms = ms;
				self.desk.session.set_display_lag(ms);
				self.desk.save_settings();
			}
			// 0 or less is no speed to walk at (fitted into the pad's range it would start the belt at its slowest, and
			// `pad.stop` is the stop): refused here, never sent; JSON carries no NaN or infinity
			SocketCommand::PadSpeed { kmh } if kmh <= 0.0 => {
				self.pad_refused("speed: not a speed".into()); // copy-lint: not shown (a socket line's detail)
			}
			SocketCommand::PadSpeed { kmh } => self.pad_command(PadCommand::Speed(kmh)),
			SocketCommand::PadStop => self.pad_command(PadCommand::Stop),
		}
	}

	/// A sync's events in order, each a socket line: a catch-up pump paid (`Session::catch_up`) or a skip
	/// (`device_skip`), held for the game while a client with QuestPump runs (`game_asks`), or a bank (`device_bank`),
	/// never held. Answers the ids of what it held, the response's `game_held`.
	fn device_events(&mut self, device: Device, events: Vec<WristEvent>, game_asks: bool, t: &Now) -> Vec<String> {
		let mut game_held = Vec::new();
		for event in events {
			self.emit(&Line::Wrist { device, event: event.clone() });
			if let Some(skip) = event.skip(device) {
				game_held.extend(self.device_skip(skip, game_asks, t));
				continue;
			}
			if let Some(bank) = event.bank(device) {
				self.device_bank(&bank, t);
				continue;
			}
			let Some(mut pump) = event.catch_up(device) else { continue };
			if game_asks {
				log::info!(
					"lan: catch-up {}: the game runs and asks its {} moves",
					link::pump_tag(&pump.id),
					pump.moves.len()
				);
				if !pump.moves.is_empty() {
					game_held.push(pump.id.clone());
				}
				pump.moves.clear();
				if pump.unwell_at.is_none() {
					continue;
				}
			}
			let (paid, fx) = self.desk.session.catch_up(&pump, t);
			self.apply(fx);
			match paid {
				Ok(_) | Err(CatchupError::AlreadyPaid(_)) => {} // sent again to a helper that restarted meanwhile
				Err(e) => log::warn!("lan: catch-up {}: {e}", link::pump_tag(&pump.id)),
			}
		}
		game_held
	}

	/// A device's Skip of unfinished moves (`WristEvent::skip`, since 2026-10-09): the game's skipped paid rows
	/// (`Session::skip`; one sent again is taken once), or, while a client with QuestPump runs, which asks the moves
	/// itself and reads paid rows only at its next load, none: held for the game as a catch-up is, its id answered for
	/// the response's `game_held`.
	fn device_skip(&mut self, skip: CatchupPump, game_asks: bool, t: &Now) -> Option<String> {
		if game_asks {
			log::info!("lan: skip {}: the game runs and asks its moves", link::pump_tag(&skip.id));
			return Some(skip.id);
		}
		let (skipped, fx) = self.desk.session.skip(&skip, t);
		self.apply(fx);
		match skipped {
			Ok(rows) => log::info!("lan: skip {}: {} unfinished moves skipped", link::pump_tag(&skip.id), rows.len()),
			Err(CatchupError::AlreadyPaid(_)) => {}
			Err(e) => log::warn!("lan: skip {}: {e}", link::pump_tag(&skip.id)),
		}
		None
	}

	/// A device's Bank all on a match pump (`WristEvent::bank`, docs/plan.md slice 58): the game's
	/// banked rows (`Session::bank`; one sent again is written once), written while a client with QuestPump runs too:
	/// its moves are in no queue the game asks, and reach it at its next load through the mirror as every row here does.
	fn device_bank(&mut self, bank: &BankPump, t: &Now) {
		let (banked, fx) = self.desk.session.bank(bank, t);
		self.apply(fx);
		match banked {
			Ok(rows) => log::info!("lan: bank {}: {} moves banked", link::pump_tag(&bank.id), rows.len()),
			Err(CatchupError::AlreadyPaid(_)) => {}
			Err(e) => log::warn!("lan: bank {}: {e}", link::pump_tag(&bank.id)),
		}
	}

	/// What a LAN route asks the loop.
	fn on_ask(&mut self, ask: Ask) {
		match ask {
			Ask::Pair { device, name, done } => {
				self.phone_paired(device, name);
				// the answer says the game as the pairing left it: the grant a phone now wants
				self.tell_devices();
				let _ = done.send(());
			}
			Ask::Unpair { device, client, done } => {
				self.phone_unpaired(device, &client);
				let _ = done.send(());
			}
			Ask::Synced { request, transport, done } => {
				let SyncRequest {
					device, client, samples, events, rows, pro_proof, offer_taken, game_offer_at, ..
				} = *request;
				self.device_synced(device, &client);
				self.counts_transport(transport);
				// a watch's 5-s windows are a heart-rate source even without a reading (the stream dropped); a phone has
				// no heart rate of its own
				if device.is_watch() {
					for sample in samples {
						self.desk.session.heart(HrSource::Watch, sample.sample_end, sample.bpm);
					}
				}
				let t = now();
				// while a client with QuestPump runs it asks the owed moves itself on its next pump, and reads paid rows
				// only at its next load: a catch-up pays none of them then (the desktop's own: `questpump-desk/src/pump.rs`
				// `pump_paid`), its felt-unwell stop still written, and the answer names it (`game_held`), so the device
				// keeps them owed
				let game_asks = self.desk.wow_with_addon();
				let game_held = self.device_events(device, events, game_asks, &t);
				// the app's rows (slice 26), kept before the response confirms them; one kept already is confirmed again,
				// and none while one could not be written
				let mut unwritten = false;
				if !rows.is_empty() {
					let (taken, fx) = self.desk.session.take_app_rows(device, &rows, &t);
					log::info!(
						"lan: {}: {} app rows, {} new, {} dropped{}",
						device.name(),
						rows.len(),
						taken.kept,
						taken.dropped,
						// copy-lint: not shown (a log line)
						if taken.unwritten { ", not all written: none confirmed" } else { "" }
					);
					unwritten = taken.unwritten;
					self.apply(fx);
				}
				// a match's offer a device took: withdrawn before this sync is answered, which names it
				if let Some(offer) = &offer_taken {
					self.offer_taken(offer);
				}
				// a device that holds the link without having heard the pump on offer (it took the link after another
				// device, or started again mid-pump) gets it before this sync is answered, ahead of its countdown
				if let Some(offer) = self.desk.session.offered()
					&& let Some(queued) = self.queue.reoffer(offer, lan::now_ms())
				{
					log::info!("lan: the pump on offer again for the device that holds the link (seq {})", queued.seq);
					self.emit(&Line::Command { command: queued });
				}
				// a phone's Pro proof (slice 35): checked here, never a reason to fail the sync; none from a phone that
				// unpaired, whose grant went with it
				if let Some(proof) = pro_proof.filter(|_| !self.unpaired.contains(&client)) {
					self.take_pro(&client, device, &proof);
				}
				let rows_store = (!unwritten).then(|| self.desk.session.app_rows().id());
				// the offers' part: this one's latest game offer folded in, and the facts every response carries as it
				// leaves told to the routes before this sync is answered (the ids taken, a watched game in a match, the
				// arms, the lock)
				self.offer_heard(game_offer_at, t.unix);
				let _ = done.send(SyncAnswer { rows_store, game_held });
			}
			Ask::Body { body, done } => {
				if self.desk.body.take(&body) {
					log::info!("lan: body: a phone's later changes taken");
					self.desk.save_body();
					self.desk.profile_changed();
					self.desk.publish_setup();
					self.desk.publish_welcome();
				}
				let _ = done.send(self.desk.body.body.clone());
			}
			Ask::Sheet { done } => {
				let mut sheet = Sheet { game_asks: self.desk.wow_with_addon(), ..self.desk.session.sheet(now().unix) };
				self.desk.serve_shown(&mut sheet);
				let _ = done.send(sheet);
			}
			Ask::History { after, proofs_since, size, done } => {
				let _ = done.send(self.desk.session.history(after, proofs_since, size));
			}
			Ask::Sheets { after, size, done } => {
				let _ = done.send(self.desk.session.sheet_log().page(after, size));
			}
			Ask::AppRows { after, size, done } => {
				let _ = done.send(self.desk.session.app_rows().page(after, size));
			}
			Ask::Account { put: None, done } => {
				let _ = done.send(Some(self.desk.account_state()));
			}
			Ask::Account { put: Some((account, r)), done } => self.desk.account_put(account, r, done),
		}
	}

	/// A device's sync, a plain post too: a watch's first names it in the pairing block, a phone's names a phone paired
	/// where none is (another phone with the key, after one unpaired), and either links now (`linked_at`: a phone that
	/// sends its rows from the background counts). The phone that unpaired, whose last request may come after its
	/// Unpair, and a watch while no phone is paired (its phone's Unpair ended its pairing too) or in the minute after an
	/// Unpair (`unpaired_at`) name nothing and link nothing.
	fn device_synced(&mut self, device: Device, client: &str) {
		let p = &mut self.desk.settings.pairing;
		let late_watch = device.is_watch() && self.unpaired_at.is_some_and(|at| now().unix - at < WATCH_LATE_S);
		if self.unpaired.contains(client) || (device.is_watch() && p.phone.is_none()) || late_watch {
			return;
		}
		let named = if device.is_watch() {
			let first = p.watch.as_deref() != Some(device.name());
			p.watch = Some(device.name().to_owned());
			first
		} else if p.phone.is_none() {
			log::info!("lan: a {} phone links again with the pairing code", device.name());
			p.phone = Some(device.name().to_owned());
			p.phone_since = Some(now().unix);
			true
		} else {
			false
		};
		if named {
			self.paired();
		}
		self.note_linked(lan::now_ms().div_euclid(1000));
		// a device served: the app in use today
		self.counts_active(questpump_core::telemetry::Source::Desktop);
	}

	/// A phone paired with the code shown (`POST /v1/pair`): the pairing block names it, with its name. A phone that
	/// unpaired in this run and pairs again syncs as any other: the pair request names no client, so every one goes from
	/// `unpaired`.
	fn phone_paired(&mut self, device: Device, name: String) {
		log::info!("lan: {name} ({}) paired", device.name());
		self.unpaired.clear();
		// the running game's load came before the pairing: it reads the pairing block at its next load, and until its
		// strip says what it shows, the step is either (a game started later reads it at its first, from the mirror
		// written before it starts)
		if self.desk.settings.pairing.phone.is_none()
			&& let Some(game) = &self.desk.p.game
		{
			self.loaded_before = Some(game.pid);
		}
		let p = &mut self.desk.settings.pairing;
		p.phone = Some(device.name().to_owned());
		p.phone_since = Some(now().unix);
		self.desk.settings.phone_name = Some(name);
		self.counts_pairing("phone", None);
		// a pairing that ended unread is made again
		ui::update(&self.app, |v| v.phones.unread = false);
		self.paired();
	}

	/// A phone's Unpair (`POST /v1/unpair`, since 2026-10-09): every pairing ends on the computer, its watch's too (the
	/// watch forgets the helper with its phone), so the game's setup screen asks again at its next load and the Phones
	/// page shows the code; that phone's Pro grant goes. The pairing code stays (`renew_pairing` makes a new one).
	fn phone_unpaired(&mut self, device: Device, client: &str) {
		log::info!("lan: the {} phone unpaired", device.name());
		self.unpaired.insert(client.to_owned());
		self.unpaired_at = Some(now().unix);
		self.end_pairings();
		// its grant first, so the other phones' grants alone decide Pro's state as the pairing block changes
		self.forget_pro(client);
		self.paired();
	}

	/// The pairing block names no phone, watch or link any more.
	fn end_pairings(&mut self) {
		let p = &mut self.desk.settings.pairing;
		p.phone = None;
		p.phone_since = None;
		p.watch = None;
		p.linked_at = None;
		self.desk.settings.phone_name = None;
	}

	/// The pairing block changed: saved, shown, and in the game at its next `/reload`; Pro's state on the computer
	/// follows the paired phone, and the Bluetooth link runs while one is paired.
	fn paired(&mut self) {
		self.desk.save_settings();
		self.ble_link_sync();
		self.show_phone();
		self.show_grant();
		self.due_by(Instant::now());
		self.pro_changed();
		self.desk.welcome_found();
	}

	/// The one step the pairing still asks of the player (`link::Step`, in every response's helper block since
	/// 2026-10-13): the Screen Recording grant while it is missing and wanted (`sight`); nothing while no client with
	/// QuestPump runs, since the game reads the pairing block at its next start by itself and its pumps reach the
	/// phones through the strip with no reload; else what the running game's strip says (`setup_word`): its setup
	/// window's Check again, a /reload where the client does not read its own save back and its load ran without
	/// QuestPump here, or nothing; `either` while a client whose load may have come before the pairing
	/// (`loaded_before`) has carried no word.
	fn step(&self) -> Step {
		if self.sight() == Sight::NoGrant {
			return Step::Grant;
		}
		if !self.desk.wow_with_addon() {
			return Step::Done;
		}
		match self.setup_word() {
			Some(word) if word.check_shown => Step::CheckAgain,
			Some(word) if word.save_unread && !word.helper_seen => Step::Reload,
			None if self.desk.p.game.as_ref().is_some_and(|g| self.loaded_before == Some(g.pid)) => Step::Either,
			Some(_) | None => Step::Done,
		}
	}

	/// The setup word of the load the session follows (`Session::load`): none before that load's strip carried one,
	/// once it ended (its save written, the game left), or from an addon older than 2026-10-13.
	fn setup_word(&self) -> Option<SetupWord> {
		self.setup.filter(|&(load, _)| self.desk.session.load() == Some(load)).map(|(_, word)| word)
	}

	/// The pairing's step on the Phones page, This computer and the welcome's Game row (the `menu.phones` topic's
	/// `next`): while a phone is paired, or while the running game's load ran without QuestPump here (its setup screen
	/// and, on a client that does not read its save back, its saved progress wait for one), and none when nothing is left; published only when
	/// it changes.
	fn show_next(&mut self, step: Step) {
		let paired = self.desk.settings.pairing.phone.is_some();
		let unseen = self.setup_word().is_some_and(|word| !word.helper_seen);
		let next = (step != Step::Done && (paired || unseen)).then_some(step);
		if next != self.next {
			self.next = next;
			ui::update(&self.app, |v| v.phones.next = next);
		}
	}

	/// The Screen Recording grant is asked for once it gives the player something (`grant_ask`): a game install with
	/// QuestPump known (`followed`: without one no strip ever shows), and a phone or a watch paired, or the strap or the
	/// walking pad on, whose heart rate meets the pumps and whose metres meet the stretch rows only through the strip.
	fn grant_wanted(&self) -> bool {
		let p = &self.desk.settings.pairing;
		self.desk.p.followed.is_some()
			&& (p.phone.is_some() || p.watch.is_some() || self.desk.settings.strap || self.desk.settings.pad)
	}

	/// A missing Screen Recording grant, asked for in the window only while it is wanted; the pairing block says so
	/// then (`sight`), so the addon's setup screen asks for it too, and the pairing's step names it first (`step`).
	fn show_grant(&mut self) {
		if !self.grant_missing {
			return;
		}
		let wanted = self.grant_wanted();
		let (dot, line) = grant_ask(wanted);
		ui::update(&self.app, |v| v.computer.permission = dot == Dot::Stopped);
		self.screen(dot, line);
		if wanted {
			self.due_by(Instant::now());
		}
	}

	/// The window's strap button, and the start with the strap on: the Bluetooth task reads it while it is on (macOS asks
	/// for Bluetooth the first time), and the verdict counts the strap as a source. Off disconnects it.
	fn set_strap(&mut self, on: bool) {
		let read = self.ble.is_some() && self.desk.settings.strap;
		if self.desk.settings.strap != on {
			self.desk.settings.strap = on;
			self.desk.save_settings();
			self.counts_trying(Kind::Strap, on);
		}
		self.desk.session.set_strap(on, lan::now_ms());
		if read && !on {
			self.emit(&ble_status(Kind::Strap, "off", None));
		}
		// starting until the task says what it does
		ui::update(&self.app, |v| v.devices.strap = ui::Device { on, line: on.then_some(DeviceLine::Starting) });
		self.sync_ble();
		self.show_grant();
	}

	/// The window's walking pad button, and the start with the pad on: the Bluetooth task reads it while it is on (macOS
	/// asks for Bluetooth the first time), and the metres it counts go into the stretch rows. Off disconnects it.
	fn set_pad(&mut self, on: bool) {
		let read = self.ble.is_some() && self.desk.settings.pad;
		if self.desk.settings.pad != on {
			self.desk.settings.pad = on;
			self.desk.save_settings();
			self.counts_trying(Kind::Pad, on);
		}
		if read && !on {
			self.emit(&ble_status(Kind::Pad, "off", None));
		}
		self.pad_read = None;
		// starting until the task says what it does
		ui::update(&self.app, |v| v.devices.pad = ui::Device { on, line: on.then_some(DeviceLine::Starting) });
		self.sync_ble();
		self.show_grant();
	}

	/// Whether the player has the device turned on.
	fn is_on(&self, kind: Kind) -> bool {
		match kind {
			Kind::Strap => self.desk.settings.strap,
			Kind::Pad => self.desk.settings.pad,
		}
	}

	/// The Bluetooth task for the devices that are on: started with the first, told when they change (a device no longer
	/// wanted is let go) and stopped with the last.
	fn sync_ble(&mut self) {
		let want = Want { strap: self.desk.settings.strap, pad: self.desk.settings.pad };
		if !want.any() {
			self.ble = None; // dropping the task stops it
		} else if let Some(task) = &self.ble {
			task.want(want);
		} else {
			self.ble = Some(bluetooth::start(self.bles.clone(), want));
		}
	}

	/// What the Bluetooth task says, taken only for a device still on: the task, or a device it reads, may have spoken
	/// just before the player turned it off.
	fn on_ble(&mut self, event: Ble) {
		// the answer to a socket client's command, given even when the pad, or the whole task, was turned off since
		if let Ble::PadRefused(why) = event {
			self.pad_refused(why);
			return;
		}
		if self.ble.is_none() {
			return; // a task turned off a moment ago
		}
		match event {
			Ble::State(_, state @ (bluetooth::State::Refused | bluetooth::State::Failed(_))) => {
				self.ble_stopped(&state);
			}
			Ble::State(kind, state) if self.is_on(kind) => self.ble_state(kind, &state),
			Ble::Heart(m) if self.desk.settings.strap => {
				if let Some(bpm) = m.reading() {
					self.desk.session.heart(HrSource::Strap, lan::now_ms(), Some(bpm));
				}
				self.emit(&Line::Hr { bpm: m.bpm, rr: m.rr, contact: m.contact });
			}
			Ble::Pad(record) if self.desk.settings.pad => self.on_pad(&record),
			Ble::PadStatus(status) if self.desk.settings.pad => self.pad_status(status),
			Ble::State(..) | Ble::Heart(_) | Ble::Pad(_) | Ble::PadStatus(_) | Ble::PadRefused(_) => {}
		}
	}

	/// A device's state: a `status` line in its words and the page's line; the pad's name is kept while it is connected.
	fn ble_state(&mut self, kind: Kind, state: &bluetooth::State) {
		let (word, hint, line) = ble_words(kind, state);
		self.emit(&ble_status(kind, word, hint));
		self.counts_ble(kind, state);
		match kind {
			Kind::Strap => ui::update(&self.app, |v| v.devices.strap.line = Some(line)),
			Kind::Pad => {
				self.pad_read = match state {
					bluetooth::State::Connected(name) => Some(PadRead {
						name: name.clone(),
						shown: line.clone(),
						kmh: None,
						standing: false,
						records: 0,
						distance: false,
					}),
					_ => None,
				};
				ui::update(&self.app, |v| v.devices.pad.line = Some(line));
			}
		}
	}

	/// The Bluetooth task stopped by itself (refused, no adapter, a failure). Its run is over for every device, so each
	/// one that is on turns off with its own words, and what the run says of the other device next finds no task; refused
	/// Bluetooth is one `error` line however many were on.
	fn ble_stopped(&mut self, state: &bluetooth::State) {
		self.ble = None;
		if self.desk.settings.strap {
			self.ble_state(Kind::Strap, state);
			self.desk.settings.strap = false;
			self.desk.session.set_strap(false, lan::now_ms());
			ui::update(&self.app, |v| v.devices.strap.on = false);
		}
		if self.desk.settings.pad {
			self.ble_state(Kind::Pad, state);
			self.desk.settings.pad = false;
			ui::update(&self.app, |v| v.devices.pad.on = false);
		}
		match state {
			bluetooth::State::Refused => {
				log::warn!("bluetooth: not allowed for QuestPump");
				self.emit(&Line::Error {
					code: ErrorCode::BluetoothPermission,
					detail: "Bluetooth is not allowed for QuestPump".into(),
				});
			}
			bluetooth::State::Failed(e) => log::warn!("bluetooth: {e}"),
			_ => {}
		}
		self.desk.save_settings();
		self.show_grant();
	}

	/// A Treadmill Data record: a `pad` line, its distance for the stretch rows, and the window's line while the pad is
	/// connected.
	fn on_pad(&mut self, record: &Treadmill) {
		self.emit(&Line::Pad { kmh: record.kmh, distance_m: record.distance_m, elapsed_s: record.elapsed_s });
		if let Some(metres) = record.distance_m {
			self.desk.session.pad(metres);
		}
		if let Some(read) = &mut self.pad_read {
			read.records = read.records.saturating_add(1);
			read.distance |= record.distance_m.is_some();
			if let Some(kmh) = record.kmh {
				read.kmh = Some(kmh);
				read.standing = kmh <= 0.0;
			}
		}
		self.show_pad();
	}

	/// The pad's line while a pad is read, drawn again when a record or a status comes and on the game tick: a load that
	/// starts or ends and the game day's turn move the day's figures while a stopped belt may send nothing.
	fn show_pad(&mut self) {
		let today = self.desk.session.pad_today(now().unix);
		let Some(read) = &mut self.pad_read else { return };
		let line = pad_walking(read, today);
		if line != read.shown {
			read.shown.clone_from(&line);
			ui::update(&self.app, |v| v.devices.pad.line = Some(line));
		}
	}

	/// The pad's own Fitness Machine Status as a `pad_event` line; a code the helper has no word for goes only to the log.
	fn pad_status(&mut self, status: MachineStatus) {
		if let Some(read) = &mut self.pad_read {
			match status {
				MachineStatus::Started => read.standing = false,
				MachineStatus::Stopped | MachineStatus::Paused | MachineStatus::SafetyKey | MachineStatus::Reset => {
					read.standing = true;
				}
				MachineStatus::TargetSpeed(_) | MachineStatus::ControlLost | MachineStatus::Other(_) => {}
			}
		}
		self.show_pad();
		let (event, kmh) = match status {
			MachineStatus::Started => (PadEventKind::Started, None),
			MachineStatus::Stopped => (PadEventKind::Stopped, None),
			MachineStatus::Paused => (PadEventKind::Paused, None),
			MachineStatus::SafetyKey => (PadEventKind::SafetyKey, None),
			MachineStatus::Reset => (PadEventKind::Reset, None),
			MachineStatus::TargetSpeed(kmh) => (PadEventKind::Speed, Some(kmh)),
			MachineStatus::ControlLost => (PadEventKind::ControlLost, None),
			MachineStatus::Other(code) => {
				log::debug!("pad: machine status {code:#04x}");
				return;
			}
		};
		self.emit(&Line::PadEvent { event, kmh, detail: None });
	}

	/// A socket client's command for the walking pad: the Bluetooth task's while the pad is on, refused at once while it
	/// is off.
	fn pad_command(&self, command: PadCommand) {
		match &self.ble {
			Some(task) if self.desk.settings.pad => task.pad(command),
			// copy-lint: not shown (a socket line's detail)
			_ => self.pad_refused(format!("{}: the walking pad is off", command.word())),
		}
	}

	/// A pad command that did not go through: a `pad_event` `refused` line whose `detail` names the command and why.
	fn pad_refused(&self, why: String) {
		log::warn!("pad: {why}");
		self.emit(&Line::PadEvent { event: PadEventKind::Refused, kmh: None, detail: Some(why) });
	}

	/// The heart-rate verdicts whose window has passed.
	fn on_proof_tick(&mut self) {
		let fx = self.desk.session.tick(lan::now_ms());
		self.apply(fx);
	}

	/// The paired phone, its watch and the device that gets the countdowns now (Phones and the tray), and whether the
	/// phone has been quiet for `QUIET_AFTER_S` (the Phones page shows the code again).
	fn show_phone(&self) {
		let p = &self.desk.settings.pairing;
		let line = PhoneLine {
			name: self.desk.phone_name(),
			watch: p.watch.is_some(),
			holder: self.linked.map(|d| if d.is_watch() { Holder::Watch } else { Holder::Phone }),
		};
		let last = p.linked_at.or(p.phone_since);
		let quiet = p.phone.is_some() && last.is_some_and(|t| now().unix - t >= QUIET_AFTER_S);
		ui::update(&self.app, |v| {
			v.phones.phone = line;
			v.phones.quiet = quiet;
		});
	}

	/// What the computer knows of World of Warcraft and the helper reads of it, in every response to the phones and the
	/// watches (since 2026-10-09; `Lan::set_game`): the game QuestPump found (`found_game`), the pairing block's sight
	/// and, since 2026-10-13, the pairing's step (`step`), which the desktop's pages name too (`show_next`), and whether
	/// a client with QuestPump runs (`wow_with_addon`), every sync response's `game_asks`, then the offer facts every
	/// sync response carries (`tell_offers`, slice 48's second fix round). After every turn of the loop, and before a
	/// pairing's answer.
	fn tell_devices(&mut self) {
		let wow = match self.found_game() {
			FoundGame::Running | FoundGame::Found => Wow::Addon,
			FoundGame::Installed => Wow::NoAddon,
			FoundGame::Unfound => Wow::Unfound,
		};
		let (sight, step, asks, ble) = (self.sight(), self.step(), self.desk.wow_with_addon(), self.ble_word());
		if self.lan.set_game(wow, sight, step, asks, ble) {
			let read = if wow == Wow::Addon { format!(", {sight:?}, step {step:?}") } else { String::new() };
			let runs = if asks { ", running" } else { "" };
			let ble = ble.map_or("unknown", link::Ble::word);
			log::info!("lan: the devices hear the game as {wow:?}{read}{runs}, Bluetooth {ble}");
		}
		self.show_next(step);
		// the offer facts as this turn left them, in the next response (a change of the ids taken back, `playing` or
		// `arms` answers the waiting requests at once)
		self.tell_offers(lan::now_ms().div_euclid(1_000));
	}

	/// The LAN API for the computer as the last look found it: its names and address in every response and the hello,
	/// the window's pairing code, and, once the firewall lets the phone in (Windows), listeners on every address, bound
	/// once (an address that moves needs no new bind) and tried again at the next look when the port is taken, with the
	/// Bonjour record (Windows) while they listen; none in a run whose secrets did not read (`secrets_unread`), whose
	/// hello (v 2, its proof of this run's key) would tell a paired phone of a pairing code the next start drops.
	async fn serve_lan(&mut self) {
		let a = &self.address;
		self.lan.set_address(&a.name, a.host, a.mdns.as_deref()); // a renamed computer shows in the next response
		let open = self.may_listen().await;
		self.show_pair();
		if open && self.served.is_none() && !self.secrets_unread() {
			self.listen();
		}
		self.advertise();
	}

	/// Listeners on every address (`lan::bind`), serving the routes until the helper winds down.
	fn listen(&mut self) {
		match lan::bind(Ipv4Addr::UNSPECIFIED, Ipv6Addr::UNSPECIFIED, pair::PORT) {
			Ok(listeners) => {
				let (stop, stopped) = watch::channel(false);
				// supervised: its panic restarts QuestPump once (`crate::keep`)
				let serving = tokio::spawn(lan::serve(listeners, self.lan.clone(), stopped));
				crate::keep::watch_tokio(&self.app, crate::keep::Part::Lan, serving);
				self.served = Some(Served { stop });
				self.lan_error = None;
				log::info!("lan: serving on every address, port {}", pair::PORT);
			}
			Err(e) => {
				let detail = format!("port {}: {e}", pair::PORT);
				let a = &self.address;
				let line = AddressLine::Lan {
					name: a.name.clone(),
					host: a.host.map(|h| h.to_string()),
					error: e.to_string(),
				};
				ui::update(&self.app, |v| v.phones.address = Some(line));
				if self.lan_error.as_deref() != Some(detail.as_str()) {
					log::warn!("lan: {detail}");
					self.lan_error = Some(detail.clone());
					self.emit(&Line::Error { code: ErrorCode::Lan, detail });
				}
			}
		}
	}

	/// The Bonjour record (Windows) of the computer as the last look found it, while the LAN API listens.
	fn advertise(&mut self) {
		let a = &self.address;
		let record = self.served.as_ref().and_then(|_| dnssd::Record::new(a.mdns.as_deref(), a.host));
		self.advert.show(record);
	}

	/// The window's "New pairing code": a new key, every pairing ended (the game's setup screen asks again at the next
	/// `/reload`) and every Pro grant with them, the LAN API's keys replaced on the listener it has (every open
	/// connection refuses the old bearer from its next request on), and the new code in the window.
	fn renew_pairing(&mut self) {
		if self.secrets_unread() {
			log::info!("lan: no new pairing code while the secrets did not read at this start");
			return;
		}
		log::info!("lan: a new pairing code; every pairing ends");
		self.desk.settings.new_key();
		self.end_pairings();
		// the grants first, so Pro's state never reads on for a moment with the old code's grants
		self.clear_pro();
		self.paired();
		self.lan.rekey(&self.desk.settings.key);
		self.ble_rekeyed(); // every central goes, and the service is published again with the new code's hellos
		self.show_pair();
	}

	/// This run's secrets will not reach the next start (`RunSecrets::Unread`).
	pub(crate) fn secrets_unread(&self) -> bool {
		self.secrets == RunSecrets::Unread
	}

	/// The pair link's address and QR code: none without a home network, nor while the firewall keeps the phone out
	/// unless the Bluetooth link serves (a phone pairs by it then, slice 43), nor in a run whose secrets did not read.
	fn show_pair(&self) {
		let Address { name, host, mdns } = &self.address;
		let name = name.clone();
		let open = self.firewall == Firewall::Open;
		let (address, qr) = match *host {
			_ if self.secrets_unread() => (AddressLine::Unread { name }, None),
			Some(host) if open || self.ble_serves() => {
				let (_, qr) =
					pairing::link_and_qr(host, self.desk.settings.key, &name, mdns.as_deref(), self.desk.settings.hid);
				let host = host.to_string();
				(if open { AddressLine::Ready { name, host } } else { AddressLine::Ble { name, host } }, qr)
			}
			Some(host) => (AddressLine::Firewall { name, host: host.to_string() }, None),
			None => (AddressLine::None { name }, None),
		};
		ui::update(&self.app, |v| {
			v.phones.address = Some(address);
			v.phones.qr = qr;
		});
	}

	/// What the window asks.
	async fn on_window(&mut self, ask: WindowAsk) {
		match ask {
			WindowAsk::Restart => self.restart().await,
			WindowAsk::NewPairing => self.renew_pairing(),
			WindowAsk::Strap(on) => self.set_strap(on),
			WindowAsk::Pad(on) => self.set_pad(on),
			WindowAsk::BleLink(on) => self.set_ble_link(on),
			WindowAsk::PhonesShown { page, code } => self.phones_shown(page, code),
			WindowAsk::Folder(path) => self.picked(&path),
			WindowAsk::AllowNetwork => self.allow_network(),
			WindowAsk::AllowFolder => self.allow_folder(),
			WindowAsk::Addon => self.addon_install(),
			WindowAsk::Update => self.update_now().await,
			WindowAsk::Feedback { text, diagnostics, done } => self.window_feedback(&text, diagnostics, done),
			WindowAsk::Desk(ask) => self.on_desk(ask),
			WindowAsk::Autostart(on) => {
				match autostart::set(on) {
					// the player's own choice, kept: off stays off at the next starts, on is on for this program
					Ok(()) => {
						self.desk.settings.autostart_off = !on;
						self.desk.settings.autostart_for = if on { std::env::current_exe().ok() } else { None };
						self.desk.save_settings();
					}
					Err(e) => log::warn!("start at login: {e:#}"),
				}
				let now = autostart::state();
				ui::update(&self.app, |v| v.computer.autostart = now);
			}
		}
	}

	/// The Screen Recording card's "Restart QuestPump" (macOS applies a grant only to a process started after it):
	/// QuestPump winds down, then starts again through LaunchServices with its window open on This computer; if it
	/// cannot, it listens again, carries on and says how to restart it by hand.
	async fn restart(&mut self) {
		log::info!("restart: the player asked, for the Screen Recording grant");
		self.wind_down().await;
		if let Err(e) = update::restart(&self.app) {
			log::warn!("restart: {e:#}");
			ui::update(&self.app, |v| v.computer.restart_failed = v.computer.restart_failed.saturating_add(1));
			self.carry_on().await;
		}
	}

	/// A one-time elevated step came back.
	async fn on_elevated(&mut self, done: Elevated) {
		match done {
			Elevated::Network(outcome) => self.network_allowed(outcome).await,
			Elevated::Folder(outcome) => self.folder_allowed(outcome),
			Elevated::Game(key, outcome) => self.game_allowed(&key, &outcome),
		}
	}

	/// What the helper does once at its start: the crash reports that wait first (`runtime/crash.rs`), the counts' kept
	/// facts before anything takes rows (`runtime/counts.rs`), the engine's fold of every row asked (it lands on the
	/// loop later), the settings saved (a new pairing key, the helper's version), the window's place, zoom and full
	/// screen handed to the desk, the strap and the walking pad read from the start when they were on, the LAN API,
	/// start at login on the very first start, the update checks, the page's first view (with the Phones page's line
	/// that the pairing must be scanned again when the settings did not read), and the main window at a player's launch
	/// (`desk::launch`; never at login, after an update or after the system quit it): on This computer while a Mac
	/// helper runs from its download (`update::unmoved`: the page asks for the move to Applications), after the grant
	/// card's restart and after the update This computer's Restart to update installed, else where the page stands
	/// (the first five minutes cover it for a new player, `desk::menu_pages::welcome`).
	async fn start(&mut self, started: Started, updates: mpsc::UnboundedSender<update::Found>) {
		let Started { loaded, launch, crashed, far_lost } = started;
		// before the engine, the devices and the window (the sweep ran first in `run()`): the crash reports that wait
		// take the next window to their page, and so does the privacy page while a switch is not answered
		self.crash_start();
		self.usage_start();
		// the counts' kept facts before anything takes rows: every `apply` looks at each new row once (`counts_rows`)
		self.counts_load(loaded);
		// the install a strip seen before the first mirror write belongs to: the folder remembered (`addons` follows the
		// running game's from then on)
		if let Some(install) = self.desk.settings.addons.as_deref().and_then(install_key) {
			let fx = self.desk.session.set_install(&install);
			self.apply(fx);
		}
		self.desk.engine_start();
		self.desk.book_log_start();
		self.drain();
		let unmoved = update::unmoved();
		self.desk.save_settings();
		if let Ok(mut place) = self.app.state::<crate::desk::Desk>().place.lock() {
			place.clone_from(&self.desk.settings.window);
		}
		let program = std::env::current_exe().map(|p| p.display().to_string()).unwrap_or_default();
		let asked = crate::capture::permission_asked(&self.dir);
		let (os, capture) = (crate::feedback::os(), self.capture_word());
		let (zoom, fullscreen) = (self.desk.settings.zoom, self.desk.settings.fullscreen);
		let resources = self.app.path().resource_dir().ok().map(|dir| dir.join("resources").display().to_string());
		ui::update(&self.app, |v| {
			v.shell.zoom = zoom;
			v.shell.fullscreen = fullscreen;
			v.shell.resources = resources;
			v.computer.asked = asked;
			v.computer.unmoved = unmoved;
			v.computer.version.build = update::build();
			v.computer.program = program;
			v.feedback.os = os;
			capture.clone_into(&mut v.feedback.capture);
		});
		self.show_phone();
		questpump_desk::Platform::show_found(&mut self.desk.p);
		if loaded == Loaded::Unread {
			ui::update(&self.app, |v| v.phones.unread = true);
		}
		self.sound_start();
		self.figures_start();
		self.reel_start();
		self.start_pro(loaded != Loaded::Read);
		self.menu_start();
		self.desk.sync_start(far_lost);
		self.set_strap(self.desk.settings.strap);
		self.set_pad(self.desk.settings.pad);
		self.serve_lan().await;
		self.ble_link_sync();
		self.start_at_login(unmoved);
		// the counts after the stores are read and the switches answered: the finished game days go
		self.counts_start(crashed);
		self.update_checks = update::spawn(self.app.clone(), &self.dir, updates);
		self.addon_start();
		if unmoved {
			log::info!("the helper runs from its download: the page asks to move it to Applications");
		}
		// the start of the update the player's Restart to update installed opens This computer, whose version line the
		// button was on (`update::restarted`, on Windows too, whose installer starts it with `--updated`)
		let restarted = update::restarted(&self.dir, &self.app.package_info().version.to_string());
		// a first start shows the first five minutes over the window (`desk::menu_pages::welcome`), never the Phones page
		let landing = match launch {
			Launch::Background if restarted => Some(Landing::Computer),
			Launch::Background => None,
			Launch::Restart => Some(Landing::Computer),
			Launch::Player if unmoved => Some(Landing::Computer),
			Launch::Player => Some(Landing::Here),
		};
		self.drain();
		log::info!("launch: {launch:?} {loaded:?}, the window {}", landing.map_or("stays closed", |_| "opens"));
		if let Some(landing) = landing {
			let handle = self.app.clone();
			let _ = self.app.run_on_main_thread(move || desk_windows::open_main(&handle, landing));
		}
	}

	/// Start at login is on by default for a published build (the mirror is the beta's only save and the wrist
	/// countdown needs the helper whenever the game runs; decided 2026-09-30, and David the same day: "can we make this
	/// the default"): registered anew at the start of a program at a path it was not registered for yet (the first
	/// start, the helper moved to Applications, the renamed app of slice 35), since the system's record may still name
	/// the old path, and whenever the system holds no record of it (`autostart::Record::Missing`: Windows' uninstaller
	/// deleted the Run value of a program reinstalled at the same path), unless the player turned it off in the window,
	/// which the helper then leaves off; one the player turned off in the system's own settings (`Record::Off`) stays
	/// off for that program. A development build is never registered by itself, and a Mac helper still in its download
	/// (`unmoved`) is neither registered nor offered the setting: its path is gone once the disk image is. On Windows
	/// the values of the helper's old name go first, at every start.
	fn start_at_login(&mut self, unmoved: bool) {
		#[cfg(windows)]
		match autostart::forget_old() {
			Ok(true) => log::info!("start at login: the values of the old name \"QuestPump Helper\" removed"),
			Ok(false) => {}
			Err(e) => log::warn!("start at login: {e:#}"),
		}
		if unmoved {
			ui::update(&self.app, |v| v.computer.autostart = None);
			return;
		}
		let program = std::env::current_exe().ok();
		if !cfg!(debug_assertions)
			&& !self.desk.settings.autostart_off
			&& program.is_some()
			&& (self.desk.settings.autostart_for != program || autostart::record() == Some(autostart::Record::Missing))
		{
			match autostart::renew() {
				Ok(()) => {
					self.desk.settings.autostart_for = program;
					self.desk.save_settings();
				}
				Err(e) => log::warn!("start at login: {e:#}"),
			}
		}
		let on = autostart::state();
		ui::update(&self.app, |v| v.computer.autostart = on);
	}

	/// The devices whose sessions ended print their delivery statistics and, while a phone is paired, each settled pump
	/// some of whose commands reached no device a line (since 2026-10-14); the log and the window show who holds the
	/// link, and the pairing block when one last did (`linked_at`, saved at most once an hour, in the game from the
	/// next mirror write); and the place in use, which a time of the places' schedule changes (`places_tick`).
	fn on_link_tick(&mut self) {
		let now = lan::now_ms();
		for summary in self.queue.quiet(now) {
			self.summary(summary);
		}
		let paired = self.desk.settings.pairing.phone.is_some();
		for u in self.queue.unreached(now).into_iter().filter(|_| paired) {
			log::info!("pump {}: {} of {} commands reached no device", link::pump_tag(&u.pump), u.unreached, u.queued);
		}
		// the log names the client that holds the link, the window only its kind
		let holding = self.queue.holder_tag(now);
		let linked = holding.as_ref().map(|&(device, _, _)| device);
		if linked.is_some() {
			self.note_linked(now.div_euclid(1000));
		}
		if holding != self.holding {
			match &holding {
				Some((device, tag, via)) => {
					log::info!("lan: {} holds the link (client {tag}, {})", device.name(), via.name());
				}
				None => log::info!("lan: no device holds the link"),
			}
			self.holding = holding;
		}
		if linked != self.linked {
			self.linked = linked;
			self.show_phone();
		}
		self.counts_link();
		self.ble_tick();
		self.desk.places_tick();
	}

	/// A device linked at `unix`: the pairing block's `linked_at`, saved at most once an hour (`LINKED_EVERY_S`).
	fn note_linked(&mut self, unix: i64) {
		let p = &mut self.desk.settings.pairing;
		if p.linked_at.is_none_or(|t| unix - t >= LINKED_EVERY_S) {
			p.linked_at = Some(unix);
			self.desk.save_settings();
		}
	}

	/// A device's session ended: its delivery time over the commands queued while it held the link, and apart the ones
	/// it got late, after a wait for a device to hold it (since 2026-10-14).
	fn summary(&self, s: link::Summary) {
		let ms = |v: Option<i64>| v.map_or_else(|| "none".to_owned(), |ms| format!("{ms} ms"));
		let rate = |b: Option<f64>| b.map_or_else(|| "none".to_owned(), |b| format!("{b:.1} %/h"));
		log::info!(
			"lan: {} session (client {}): {} commands, delivery p50 {}, p95 {}, late {}, transport {}, battery {} (lan {}, \
			 ble {})",
			s.device.name(),
			link::client_tag(&s.client),
			s.commands,
			ms(s.p50_ms),
			ms(s.p95_ms),
			s.late,
			s.transport.map_or("none", link::Transports::name),
			rate(s.battery_per_hour),
			rate(s.battery_lan_per_hour),
			rate(s.battery_ble_per_hour)
		);
		self.emit(&Line::Link(s));
	}

	/// The game folder the mirror goes into, the install the session follows (`followed`; slice 26: the strip's rows and
	/// characters are that install's, numbered as it numbers them): the running client's when it holds QuestPump
	/// (another flavour without it, such as retail beside the Forever install, gets no mirror and is never remembered),
	/// else the one found or picked before while it still holds QuestPump (one deleted, moved or emptied of QuestPump is
	/// looked for again), else a folder the player picked before QuestPump was in it, once it is, else the preferred
	/// install the Battle.net Agent or the default folders know that has QuestPump (`install::discover`). A new one is
	/// remembered and shown; while none is known, the game on this computer without QuestPump is shown instead
	/// (`show_without_addon`). Whether one is followed decides whether the Screen Recording grant is wanted
	/// (`grant_wanted`), so a change shows the grant again.
	fn addons(&mut self) -> Option<PathBuf> {
		let holds = |a: &PathBuf| install::holds_questpump(a);
		let found = self
			.desk
			.p
			.game_addons
			.clone()
			.or_else(|| self.desk.settings.addons.clone().filter(holds))
			.or_else(|| self.desk.p.without_addon.clone().filter(holds))
			.or_else(install::discover);
		let followed_before = self.desk.p.followed.is_some();
		self.desk.p.followed.clone_from(&found);
		match &found {
			Some(found) => self.follow(found),
			None => self.show_without_addon(),
		}
		if followed_before != self.desk.p.followed.is_some() {
			self.show_grant();
		}
		found
	}

	/// The install with QuestPump the session follows from now: its numbering, remembered and shown.
	fn follow(&mut self, found: &PathBuf) {
		self.desk.p.without_addon = None;
		self.without_addon_picked = false;
		if let Some(install) = install_key(found) {
			let fx = self.desk.session.set_install(&install);
			self.apply(fx);
		}
		if self.desk.settings.addons.as_ref() != Some(found) {
			self.desk.settings.addons = Some(found.clone());
			self.desk.save_settings();
		}
		if self.shown_folder.as_ref() != Some(found) {
			self.show_folder(found);
		}
	}

	/// While no install with QuestPump is known: the game on this computer without it (`without_addon`: the running
	/// client's, an AddOns folder or not; the folder the player picked while it is a game; else the preferred of the
	/// one remembered while it is a game and the installs the Agent or the default folders know, `install::preferred`
	/// over `install::discover_game`, ranked again at each look, so a beta found before the live install came gives way
	/// to it, as the first install does, `first_target`), shown on This computer with its folder, which says QuestPump is
	/// not in it; none shows that no game folder is known. The progress is kept on this computer meanwhile.
	fn show_without_addon(&mut self) {
		let running = self.desk.p.game.as_ref().filter(|_| self.desk.p.game_addons.is_none()).map(|g| g.addons.clone());
		let picked = self.desk.p.without_addon.take().filter(|a| self.without_addon_picked && install::is_game(a));
		self.without_addon_picked = running.is_none() && picked.is_some();
		let game = running.or(picked).or_else(|| {
			let remembered = self.desk.settings.addons.clone().filter(|a| install::is_game(a));
			install::preferred(remembered.into_iter().chain(install::discover_game()))
		});
		self.desk.p.without_addon.clone_from(&game);
		if game.is_some() {
			self.addon_game_found();
		}
		match game {
			Some(path) if self.shown_folder.as_ref() != Some(&path) => self.show_folder(&path),
			Some(_) => {}
			None => {
				self.shown_folder = None;
				ui::update(&self.app, |v| {
					v.computer.folder = ui::Folder::None;
					v.computer.pack = ui::Pack::NotSeen;
				});
			}
		}
		ui::update(&self.app, |v| v.computer.mirror = Mirror::Here);
	}

	/// The 2-s look, every `LOOK_FOR_GAME` while the game is closed (the game's start looks at once), and at once when
	/// QuestPump is now in the game folder known without it (`without_addon`, one `stat` a look): the install followed
	/// checked again, and one found since (QuestPump installed while QuestPump runs) gets the mirror at once, so the
	/// client's first start already finds the pairing block, the sign of life and the rows.
	fn look_for_game(&mut self) {
		if self.desk.p.game.is_some() {
			return;
		}
		let installed = self.desk.p.followed.is_none()
			&& self.desk.p.without_addon.as_ref().is_some_and(|a| install::holds_questpump(a));
		if !installed && self.looked_for_game.is_some_and(|t| t.elapsed() < LOOK_FOR_GAME) {
			return;
		}
		self.looked_for_game = Some(Instant::now());
		let before = self.desk.p.followed.clone();
		if let Some(found) = self.addons()
			&& before.as_ref() != Some(&found)
		{
			log::info!("game: QuestPump found in {}", found.display());
			self.due_by(Instant::now());
		}
	}

	/// The 2-s look: one process snapshot for the game and the games on the computer, then an update waiting for its
	/// moment and an outbox's round due.
	async fn on_look(&mut self, capture: &Capture) {
		let snap = Snapshot::take();
		self.on_game_tick(capture, &snap);
		self.sitting_look(now().unix, &snap);
		self.reminders_look();
		self.on_games_tick(&snap);
		self.desk.recent_look(now().unix);
		self.look_for_game();
		// the addon's update before QuestPump's own (`addon_busy` holds that one)
		self.addon_look();
		self.update_when_closed().await;
		self.socket_quit_due();
		self.feedback_due();
		self.crash_due();
		self.counts_due();
		self.desk.home_look();
		// a game started or quit, the engine folded: the pump's gates and the tray
		self.desk.pump_refresh();
		// the game found or running, and a pump that held the first five minutes' end gone
		self.desk.welcome_found();
	}

	fn on_game_tick(&mut self, capture: &Capture, snap: &Snapshot) {
		self.show_pad();
		// a save the client wrote while it runs (a /reload, a logout to the character list) comes in now, so a load the
		// strip never showed (no grant, the game out of view) is in the mirror from the load after it on; the load after
		// it reads the pairing block and the sign of life, so its load no longer comes before them
		if let Some(addons) = self.desk.p.game.as_ref().map(|g| g.addons.clone()).filter(|a| self.taken_in.contains(a))
			&& self.take_in_saves(&addons)
		{
			self.loaded_before = None;
		}
		if alive_due(self.desk.p.game.is_some(), self.alive_at, now().unix) {
			self.beat_for_game();
		}
		let running = game::running(self.desk.p.game.as_ref(), snap);
		// QuestPump installed or removed while the game runs (a /reload loads it): the mirror follows from its next write
		let lacked = self.game_lacks_addon();
		let had = self.desk.p.game_addons.take();
		self.desk.p.game_addons = running.as_ref().map(|g| g.addons.clone()).filter(|a| install::holds_questpump(a));
		let lacks = running.is_some() && self.desk.p.game_addons.is_none();
		if running == self.desk.p.game {
			if self.desk.p.game_addons != had {
				// QuestPump installed in the running game (its rows are that install's), or removed from it
				let _ = self.addons();
			}
			// a still corner sends no frame: the line for a strip out of sight that long, or in a game without QuestPump,
			// comes on the tick, and so does the end of Reading once no frame read for `OUT_AFTER` (a stream that
			// stopped as the window left the screen sent one frame without the strip; frames that do not read are the
			// unreadable rule's)
			let unseen = self.desk.p.sighting.read_at.is_some_and(|t| t.elapsed() >= UNSEEN_AFTER);
			let gone = self.desk.p.screen == Dot::Live
				&& !self.desk.p.sighting.read_lately()
				&& !self.desk.p.sighting.unreadable_now();
			if self.desk.p.game.is_some()
				&& (gone || (self.desk.p.screen == Dot::NoStrip && (lacks || lacked || unseen)))
			{
				let line = self.out_of_sight();
				self.screen(Dot::NoStrip, line);
			}
			return;
		}
		let was = self.desk.p.game.take();
		self.desk.p.game = running;
		self.desk.p.sighting.read = false; // another client, or none: none of its frames read yet
		// the client whose load came before the mirror is left once another or none runs: a client that starts now reads
		// the pairing block and the sign of life at its load
		if self.desk.p.game.as_ref().map(|g| g.pid) != self.loaded_before {
			self.loaded_before = None;
		}
		// the capture reads the window of the client the mirror follows, never another client's
		capture.set_game(self.desk.p.game.as_ref().map(|g| g.pid));
		// the client by its install's label, and what QuestPump found of the game with it, in one change of the view
		let client =
			self.desk.p.game.as_ref().map(|g| ui::RunningGame { label: install::label(&g.addons), read: false });
		let found = self.desk.found().game;
		ui::update(&self.app, |v| {
			v.computer.game = client;
			v.computer.found = found;
			v.computer.folder_note = None; // a pick refused while the game ran can be made now
		});
		self.shown_folder = None; // the folder's state again: an addon updated while the game was closed, another install
		let _ = self.addons();
		// a capture that stopped tries again with the game (a missing grant waits for the grant and a restart)
		let again = self.desk.p.screen == Dot::Stopped && !self.grant_missing;
		if again || !matches!(self.desk.p.screen, Dot::Stopped | Dot::Off) {
			self.desk.p.sighting.unreadable = None;
			if self.desk.p.game.is_some() {
				self.desk.p.sighting.read_at = Some(Instant::now());
				let line = if lacks { self.out_of_sight() } else { Screen::Waiting };
				self.screen(Dot::NoStrip, line);
			} else {
				self.screen(Dot::NoGame, Screen::NotSeen);
			}
		}
		// every game start: a game in another folder than the last write gets the pairing block and the rows before
		// it logs in, and the write's sign of life tells the addon at its first load that the helper runs
		// (`ns.HelperState`)
		if self.desk.p.game.is_some() {
			self.due_by(Instant::now());
		}
		// the client followed until now is left: its logout rule and the devices' `session_end`, whether none runs now
		// or another took its place within the tick. `game::running` leaves a running client only for one with
		// QuestPump where it had none, so a client left this way quit or never painted a strip.
		if let Some(left) = was {
			let fx = self.desk.session.end_game(&now());
			self.apply(fx);
			// the client wrote its save as it quit: rows the strip never showed (the helper started late, the capture
			// stopped) come in here
			self.take_in_saves(&left.addons);
			// the game closed: what the release feeds bring may go in now
			if self.desk.p.game.is_none() {
				self.feeds_wake();
			}
		}
	}

	/// What a file of the install in `addons` holds comes into the store and the people, in that install's numbering
	/// (`Session::adopt`). A mirror that does not read is copied aside as `State.lua.unread` before the helper's write
	/// replaces it; a save that does not read is only reported, since the helper never writes it.
	/// False when the file exists but could not be read (a permission, a sharing violation while the client reads it)
	/// or its folder does not resolve (`install_key`).
	fn take_in_file(&mut self, path: &Path, addons: &Path, save: bool) -> bool {
		let bytes = match std::fs::read(path) {
			Ok(b) => b,
			Err(e) if e.kind() == std::io::ErrorKind::NotFound => return true,
			Err(e) => {
				if self.unread.insert(path.to_path_buf()) {
					log::warn!("{}: {e}; read again until it reads", path.display());
				}
				return false;
			}
		};
		self.unread.remove(path);
		let parse = if save { mirror::parse_save } else { mirror::parse };
		let held =
			std::str::from_utf8(&bytes).map_err(|e| e.to_string()).and_then(|t| parse(t).map_err(|e| e.to_string()));
		match held {
			Ok(held) => {
				let Some(install) = install_key(addons) else {
					if self.unread.insert(addons.to_path_buf()) {
						log::warn!("{}: the folder does not resolve; read again until it does", addons.display());
					}
					return false;
				};
				self.unread.remove(addons);
				let (taken, fx) = self.desk.session.adopt(&held, &install, &now());
				if taken > 0 || fx.people_changed {
					log::info!("{}: took in {taken} rows the helper did not have", path.display());
				}
				self.apply(fx);
			}
			Err(e) if save => log::warn!("{}: {e}", path.display()),
			Err(e) => {
				log::warn!("{}: {e}; kept aside as State.lua.unread", path.display());
				if let Err(e) = std::fs::copy(path, path.with_file_name("State.lua.unread")) {
					log::warn!("State.lua.unread: {e}");
				}
			}
		}
		true
	}

	/// The game's saves beside a folder that the client wrote since the helper read them (`mirror::SavesRead`): the rows
	/// of a session played while the helper was off or out of the strip's sight, and a reset made then. The load the
	/// strip showed before the write ends first (`Session::save_written`), so the save's owed rows meet the logout
	/// rule's. A save that could not be read is tried again at the next look. Whether the client wrote one since.
	fn take_in_saves(&mut self, addons: &Path) -> bool {
		let written = self.saves_read.written(addons);
		let came = !written.is_empty();
		for (path, at) in written {
			if let Some(written) = at.and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()) {
				let fx = self.desk.session.save_written(written.as_secs_f64(), &now());
				self.apply(fx);
			}
			if self.take_in_file(&path, addons, true) {
				self.saves_read.read(path, at);
			}
		}
		came
	}

	/// Before the first write into a folder: the mirror already there (a lost data folder costs no rows), then the
	/// game's saves. False while that mirror exists but cannot be read: it is never overwritten unread.
	fn take_in(&mut self, addons: &Path) -> bool {
		if self.taken_in.contains(addons) {
			return true;
		}
		if !self.take_in_file(&mirror::state_path(addons), addons, false) {
			return false;
		}
		self.take_in_saves(addons);
		self.taken_in.insert(addons.to_path_buf());
		true
	}

	/// The stores hold what this computer has, from the first mirror job on, whether the take-in read the install it
	/// follows or failed (a `State.lua` that does not read, a folder that does not resolve: tried again, and what it
	/// brings later may still end the first five minutes): the first five minutes may decide.
	fn stores_are_read(&mut self) {
		if !self.desk.p.stores_read {
			self.desk.p.stores_read = true;
			self.desk.welcome_found();
		}
	}

	/// What the pairing block says the helper reads of the game, while a device, the strap or the pad needs it to
	/// (`grant_wanted`): the grant missing, or the capture stopped or reading a strip that does not read (the tray's
	/// red), on either system.
	fn sight(&self) -> Sight {
		if !self.grant_wanted() {
			Sight::Sees
		} else if self.grant_missing {
			Sight::NoGrant
		} else if self.desk.p.screen == Dot::Stopped {
			Sight::Stopped
		} else {
			Sight::Sees
		}
	}

	/// The mirror's text for the folder, in its install's numbering with its install's profiles (`Session::mirrored`),
	/// or None when no folder is known yet (reported once).
	fn mirror_job(&mut self) -> Option<(PathBuf, String)> {
		let Some(addons) = self.addons() else {
			self.stores_are_read();
			if self.mirror_error.as_deref() != Some(NO_FOLDER) {
				self.mirror_error = Some(NO_FOLDER.into());
				self.emit(&Line::Error {
					code: ErrorCode::NoGameFolder,
					detail: "no World of Warcraft folder with QuestPump found yet".into(),
				});
			}
			return None;
		};
		let taken = self.take_in(&addons);
		self.stores_are_read();
		if !taken {
			self.read_retry = true;
			return None;
		}
		// a folder that does not resolve is numbered and written once it does (`install_key`), reported once
		let Some(install) = install_key(&addons) else {
			// copy-lint: not shown (a socket line's detail)
			let detail = format!("{}: the folder does not resolve", addons.display());
			if self.mirror_error.as_deref() != Some(detail.as_str()) {
				self.mirror_error = Some(detail.clone());
				self.emit(&Line::Error { code: ErrorCode::MirrorWrite, detail });
				ui::update(&self.app, |v| v.computer.mirror = Mirror::Unfound);
			}
			self.read_retry = true;
			return None;
		};
		// a character the install has no index for yet gets one here, kept from then on (the people saved)
		let (mirrored, fx) = self.desk.session.mirrored(&install);
		self.apply(fx);
		let t = now();
		let text = mirror::state_lua(&State {
			saved_at: t.unix,
			pairing: &self.desk.settings.pairing,
			seen_at: t.unix,
			sight: self.sight(),
			rows: &mirrored.rows,
			chars: &mirrored.chars,
			profiles: &mirrored.profiles,
			profile_keys: &mirrored.profile_keys,
			proofs: &self.desk.session.book().mirror(),
			body: &self.desk.body.body,
		});
		Some((addons, text))
	}

	fn on_mirror_tick(&mut self) {
		let now = Instant::now();
		if self.held_until.is_some_and(|t| now < t) || self.due.is_none_or(|t| now < t) {
			return;
		}
		self.held_until = None;
		let job = self.mirror_job(); // taking a folder in makes the write due again: this write carries it
		self.due = None;
		self.batch_since = None;
		if std::mem::take(&mut self.read_retry) {
			self.due = Some(now + Duration::from_secs(1));
		}
		if let Some((addons, text)) = job {
			let _ = self.writes.send(Job::Mirror { addons, text });
		}
	}

	fn on_written(&mut self, written: Written) {
		match written {
			Written::Mirror(addons, Ok(())) => {
				self.mirror_error = None;
				self.written_fine();
				self.written = Some(addons);
				self.beat(); // the folder and its TOC are there: the file exists before the next client starts
				let at = lan::now_ms();
				ui::update(&self.app, |v| v.computer.mirror = Mirror::Kept { at });
			}
			Written::Mirror(addons, Err(e)) => {
				let detail = format!("{}: {e}", addons.display());
				if self.mirror_error.as_deref() != Some(detail.as_str()) {
					self.mirror_error = Some(detail.clone());
					self.emit(&Line::Error { code: ErrorCode::MirrorWrite, detail });
					let error = e.to_string();
					ui::update(&self.app, |v| v.computer.mirror = Mirror::Failed { error });
				}
				self.refused(&addons, &e);
			}
			Written::File(name, Err(e)) => log::warn!("{name}: {e}"),
			Written::File(_, Ok(())) => {}
			Written::Alive(addons, result) => {
				let error =
					result.err().map(|e| format!("{}: {e}", addons.join(mirror::FOLDER).join(mirror::ALIVE).display()));
				if error != self.alive_error {
					if let Some(e) = &error {
						log::warn!("{e}; the game reads the helper as not running if this lasts 30 s");
					}
					self.alive_error = error;
				}
			}
		}
	}

	/// Quit: the main window's place kept, a pump on screen ended as Leave ends it, a match's pump banking its own moves
	/// left (`pump_quit`: the system's own quit asks nothing first), the LAN API stops taking requests (the loop no
	/// longer answers them), the devices' statistics so far, the engine's calls answered, a pending mirror written first
	/// (the hold for `leaving` included), then the copy moved into Applications that a second launch asked to take over
	/// (`start`), or QuestPump's own bundle for a restart asked on the socket (`again`), is set to start once this process
	/// has gone, and the app exits.
	async fn quit(&mut self, start: Option<PathBuf>, again: bool) {
		let place = self.app.state::<crate::desk::Desk>().place();
		self.keep_place(place);
		self.desk.pump_quit();
		self.drain();
		self.wind_down().await;
		if let Some(moved) = start {
			match update::start_moved(&moved) {
				Ok(()) => log::info!("quit: {} starts once this copy has gone", moved.display()),
				Err(e) => log::warn!("quit: {}: {e:#}", moved.display()),
			}
		}
		if again {
			match update::start_updated() {
				Ok(()) => log::info!("quit: QuestPump starts again once this copy has gone"),
				Err(e) => log::warn!("quit: QuestPump does not start again: {e:#}"),
			}
		}
		self.app.exit(0);
	}

	/// Ends the helper's work before this process goes (a quit, a restart, an update): the LAN API stops and its Bonjour
	/// record (Windows) is withdrawn, the games' runs and the sitting clock are kept for the next start
	/// (`Watcher::keep`, `sitting_keep`), each device's link summary goes out on the socket, the engine's calls asked so
	/// far answer (`engine_settle`: the rows they wrote are kept), and every pending write is done.
	async fn wind_down(&mut self) {
		if let Some(served) = self.served.take() {
			let _ = served.stop.send(true);
		}
		// the games running are played up to now and their runs kept, both written by the clock's write just after
		self.desk.recent_seen(now().unix);
		self.desk.settings.runs = self.desk.p.games.as_ref().map(|g| g.watcher.keep(lan::now_ms()));
		self.sitting_keep(now().unix);
		self.ble_stop();
		self.advert.stop().await;
		let summaries = self.queue.summaries();
		let said = !summaries.is_empty();
		for summary in summaries {
			self.summary(summary);
		}
		if said {
			tokio::time::sleep(Duration::from_millis(50)).await; // the socket's writers send the lines before the exit
		}
		self.desk.engine_settle().await;
		self.drain();
		self.flush().await;
	}

	/// The loop goes on after a wind-down whose exit failed (a restart or an update that did not start): the LAN API
	/// listens again, and what the wind-down kept for a start that never came goes (the games' runs and the sitting
	/// clock), so a later start never takes it, and never closes a run again.
	async fn carry_on(&mut self) {
		self.desk.settings.runs = None;
		self.desk.settings.sitting = None;
		self.desk.save_settings();
		self.serve_lan().await;
	}

	/// A due mirror written now, whatever holds it (`leaving` included), then every write asked so far (the settings,
	/// the people, the proof book) waited for, at most 6 s.
	async fn flush(&mut self) {
		self.counts_save();
		if self.due.is_some()
			&& let Some((addons, text)) = self.mirror_job()
		{
			let _ = self.writes.send(Job::Mirror { addons, text });
		}
		let (done, wait) = oneshot::channel();
		if self.writes.send(Job::Flush(done)).is_ok() {
			let _ = tokio::time::timeout(Duration::from_secs(6), wait).await;
		}
	}
}

/// The sign of life in a game folder is fresh: its file was written within the addon's `C.helper_alive_slack_s`, so a
/// client that loaded meanwhile saw QuestPump run.
fn alive_fresh(addons: &Path) -> bool {
	std::fs::metadata(addons.join(mirror::FOLDER).join(mirror::ALIVE))
		.and_then(|m| m.modified())
		.is_ok_and(|at| at.elapsed().is_ok_and(|age| age < ALIVE_SLACK))
}

/// The helper's sign of life is written again (`ALIVE_EVERY_S`): the game runs and the last one is 10 s old on the computer's clock.
fn alive_due(game: bool, alive_at: Option<i64>, now: i64) -> bool {
	game && alive_at.is_none_or(|t| now.saturating_sub(t) >= ALIVE_EVERY_S)
}

/// The capture's state as the counts' `capture_day` names it: the strip read, out of sight or not there, not
/// readable (another strip version, colours that do not read), the grant missing, or the capture stopped.
fn capture_state(dot: Dot, line: &Screen) -> &'static str {
	match (dot, line) {
		(Dot::Live, _) | (_, Screen::Reading) => "reading",
		(Dot::Off, _) | (_, Screen::Grant | Screen::Off) => "denied",
		(_, Screen::Newer | Screen::Older | Screen::Colours { .. }) => "unreadable",
		(Dot::Stopped, _) => "error",
		_ => "no_strip",
	}
}

/// The `status` line's word for the capture's state.
fn status(dot: Dot) -> &'static str {
	match dot {
		Dot::NoGame => "no_game",
		Dot::NoStrip => "no_strip",
		Dot::Live => "live",
		Dot::Stopped | Dot::Off => "stopped",
	}
}

/// How a missing Screen Recording grant shows (decided 2026-09-26, `docs/tasks.md`): until a phone or a watch is
/// paired or a strap or a walking pad turned on, reading the screen gives the player nothing they see (the mirror comes
/// from the game's saves), so the tray stays grey and the page asks nothing; once one is, the tray turns red and This
/// computer explains the grant and asks.
fn grant_ask(wanted: bool) -> (Dot, Screen) {
	if wanted { (Dot::Stopped, Screen::Grant) } else { (Dot::Off, Screen::Off) }
}

/// A Bluetooth device's state as the `status` line's word, its hint and the page's line.
fn ble_words(kind: Kind, state: &bluetooth::State) -> (&'static str, Option<&'static str>, DeviceLine) {
	use bluetooth::State;
	match state {
		State::Searching => ("searching", None, DeviceLine::Searching),
		State::NotFound => {
			let hint = match kind {
				Kind::Strap => "WHOOP Broadcast HR toggle", // copy-lint: not shown (a socket line's hint)
				Kind::Pad => "close the pad's own app",     // copy-lint: not shown (a socket line's hint)
			};
			("not_found", Some(hint), DeviceLine::NotFound)
		}
		State::Connected(name) => ("connected", None, DeviceLine::Connected { name: name.clone() }),
		// Windows reads a radio turned off by a switch or by its driver as neither on nor off
		State::Waiting => ("waiting", None, DeviceLine::Waiting),
		State::BluetoothOff => ("bluetooth_off", None, DeviceLine::Off),
		State::Refused => ("stopped", None, DeviceLine::Refused),
		State::Failed(e) => ("stopped", None, DeviceLine::Failed { error: e.clone() }),
	}
}

/// The `status` line of a Bluetooth device's word: `hr` for the strap, `pad` for the walking pad.
fn ble_status(kind: Kind, word: &str, hint: Option<&str>) -> Line {
	let word = Some(word.to_owned());
	let (hr, pad) = match kind {
		Kind::Strap => (word, None),
		Kind::Pad => (None, word),
	};
	Line::Status { screen: None, hint: hint.map(Into::into), hr, pad }
}

/// The pad's line, in the player's road units: the pad's name and its belt (walking at its speed, stopped, or not known
/// yet), then the day's walk (`Session::pad_today`): the metres kept with the day's quests and those the open stretch
/// holds for the next one, or that the walk counts only while the game runs and the helper sees it; for a pad that
/// shares no distance, that the helper cannot keep it.
fn pad_walking(pad: &PadRead, today: PadDay) -> DeviceLine {
	let road = units::Road::now();
	let belt = match pad.kmh {
		_ if pad.standing => Belt::Stopped,
		Some(kmh) => Belt::Moving { speed: road.speed(kmh) },
		None => Belt::Unknown,
	};
	let metres = |m: u32| road.distance(f64::from(m));
	let walk = if pad.distance {
		match (today.kept, today.open) {
			(0, Some(0)) => Walk::None,
			(0, Some(open)) => Walk::Open { open: metres(open) },
			(kept, Some(0)) => Walk::Kept { kept: metres(kept) },
			(kept, Some(open)) => Walk::Both { kept: metres(kept), open: metres(open) },
			(0, None) => Walk::Game,
			(kept, None) => Walk::KeptGame { kept: metres(kept) },
		}
	} else if pad.records >= NO_DISTANCE_AFTER {
		Walk::NoDistance
	} else {
		Walk::Pending
	};
	DeviceLine::Walking { name: pad.name.clone(), belt, walk }
}

/// The computer's name, LAN address and `.local` name (read off the loop: `route` and `scutil` are processes).
async fn find_address() -> Address {
	tokio::task::spawn_blocking(Address::read).await.unwrap_or_else(|_| Address {
		name: copy::desk_menu_phones_unnamed().to_owned(),
		host: None,
		mdns: None,
	})
}

/// The capture's display and scale, which the socket's `hello` names, as each stream starts.
fn keep_display(display: &Mutex<(String, f64)>, event: &Event) {
	if let Event::Display { name, scale, .. } = event
		&& let Ok(mut d) = display.lock()
	{
		*d = (name.clone(), *scale);
	}
}

fn hello(state: &Arc<Mutex<(String, f64)>>) -> String {
	let (display, scale) = state.lock().map(|s| s.clone()).unwrap_or_default();
	let line = Line::Hello { pid: std::process::id(), version: env!("CARGO_PKG_VERSION").into(), display, scale };
	line.to_json(&now().rfc3339).unwrap_or_default()
}

fn socket_task(
	app: &AppHandle,
	dir: &Path,
	lines: &broadcast::Sender<String>,
	display: &Arc<Mutex<(String, f64)>>,
) -> mpsc::UnboundedReceiver<SocketCommand> {
	let (cmds_tx, cmds) = mpsc::unbounded_channel::<SocketCommand>();
	let (dir, lines, hello_state) = (dir.to_path_buf(), lines.clone(), display.clone());
	let serving = tokio::spawn(async move {
		if let Err(e) = socket::serve(&dir, lines, move || hello(&hello_state), cmds_tx).await {
			log::error!("the local socket stopped: {e}");
		}
	});
	// supervised: its panic restarts QuestPump once (`crate::keep`)
	crate::keep::watch_tokio(app, crate::keep::Part::Socket, serving);
	cmds
}

/// The LAN routes' state for the pairing key and the install's id; `serve_lan` gives them the names and the address
/// before they listen.
fn lan_state(settings: &Settings, queue: &Links, asks: mpsc::UnboundedSender<Ask>) -> Arc<Lan> {
	let helper = link::Helper {
		name: String::new(),
		host: String::new(),
		port: pair::PORT,
		version: env!("CARGO_PKG_VERSION").to_owned(),
		t: 0,
		mdns: None,
		hid: Some(settings.hid_text()),
		wow: None,
		sight: None,
		step: None,
		ble: None,
	};
	Lan::new(&settings.key, queue.clone(), helper, asks, Duration::from_millis(link::HOLD_MS.unsigned_abs()))
}

/// The channels the tray and the window's commands reach the loop by: quit, and the window's asks.
fn window_channels(app: &AppHandle) -> (mpsc::UnboundedReceiver<Quit>, mpsc::UnboundedReceiver<WindowAsk>) {
	let shared = app.state::<Shared>();
	let (quit_tx, quits) = mpsc::unbounded_channel::<Quit>();
	if let Ok(mut q) = shared.quit.lock() {
		*q = Some(quit_tx);
	}
	let (asks_tx, asks) = mpsc::unbounded_channel::<WindowAsk>();
	if let Ok(mut a) = shared.asks.lock() {
		*a = Some(asks_tx);
	}
	(quits, asks)
}

/// The games on the computer, with the channels the listener's posts and the games' feeds (Rocket League's, slice 48's
/// fed games' tasks) reach the loop by; none when the offers' numbers or the games' list did not build in.
fn games_channels() -> (Option<Games>, mpsc::UnboundedReceiver<games::Heard>, mpsc::UnboundedReceiver<games::News>) {
	let (heard_tx, heard) = mpsc::unbounded_channel::<games::Heard>();
	let (news_tx, news) = mpsc::unbounded_channel::<games::News>();
	let games = match (offers::tunables(), questpump_core::games::computer_links()) {
		(Some(t), Some(links)) => Some(Games::new(t, links, heard_tx, news_tx)),
		(None, _) => {
			log::error!("contract/library.json's constants did not build in: no game on the computer is watched");
			None
		}
		(Some(_), None) => {
			log::error!("contract/links.json's computer list did not build in: no game on the computer is watched");
			None
		}
	};
	(games, heard, news)
}

/// The body every app device shares from `body.json`, a file that did not read logged (Your setup says it started
/// empty).
/// How the engine's door runs a call that may panic (`questpump_desk::engine::Caught`): its panic caught, never a
/// crash report (`crash::caught`).
fn engine_caught(call: Box<dyn FnOnce() + '_>) -> std::thread::Result<()> {
	crate::crash::caught(call)
}

fn kept_body(dir: &Path) -> Kept {
	let kept = Kept::load(dir, lan::now_ms());
	if let Some(why) = &kept.unread {
		log::warn!("{}: {why}; the body starts empty until a device fills it", Kept::FILE);
	}
	kept
}

/// The session from the data folder: the rows, the app's rows, the sheets, the people, the heart-rate proofs and the
/// learnt display lag; and whether `far.jsonl` lost rows (it could not be read, or lines of it did not), which the sync
/// takes in again (`Desk::sync_start`).
fn open_session(dir: &Path, settings: &Settings) -> anyhow::Result<(Session, bool)> {
	let (store, damaged) = Store::open(dir)?;
	if damaged > 0 {
		log::warn!("rows.jsonl: {damaged} lines did not read and were skipped");
	}
	// the app's rows from the latest reset on, the game store's floor
	let (app, damaged) = AppRows::open(dir, store.floor())?;
	if damaged > 0 {
		log::warn!("{}: {damaged} lines did not read and were skipped", app_rows::FILE);
	}
	// the sheet log only feeds the phone's charts: a file that cannot be read leaves it in memory for this run (and the
	// file untouched), never the helper unstarted
	let sheet_log = match SheetLog::open(dir, store.floor()) {
		Ok((log, damaged)) => {
			if damaged > 0 {
				log::warn!("{}: {damaged} lines did not read and were skipped", sheets::FILE);
			}
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
	let mut session = Session::new(store, people, Zone::new(local_offset_at));
	session.set_app_rows(app);
	session.set_sheet_log(sheet_log);
	session.set_display_lag(settings.display_lag_ms);
	session.set_book(Book::load(dir));
	// the other computers' rows (slice 60): a file that cannot be read leaves them in memory for this run; either way a
	// row lost is taken in again, since the sync pulls the account from its start (`Desk::sync_start`)
	let far_lost = match FarRows::open(dir) {
		Ok((far, damaged)) => {
			if damaged > 0 {
				log::warn!("{}: {damaged} lines did not read and were skipped", far::FILE);
			}
			session.set_far_rows(far);
			damaged > 0
		}
		Err(e) => {
			log::warn!("{} could not be read ({e}); the other computers' rows of this run stay in memory", far::FILE);
			true
		}
	};
	Ok((session, far_lost))
}

/// The loop's timers: a tick missed while the loop was busy comes late, never in a burst.
struct Ticks {
	game: tokio::time::Interval,
	/// The mirror's debounce, and the main window's place once it settled.
	mirror: tokio::time::Interval,
	pair: tokio::time::Interval,
	link: tokio::time::Interval,
	proof: tokio::time::Interval,
	/// A feedback message, a crash report or a counts' batch that waits; its first tick is at once, so what waits goes
	/// at the start.
	outbox: tokio::time::Interval,
	/// The sync's due calls and a link request's answer (slice 60).
	sync: tokio::time::Interval,
}

impl Ticks {
	fn new() -> Self {
		let every = |period| {
			let mut tick = tokio::time::interval(period);
			tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
			tick
		};
		Self {
			game: every(Duration::from_secs(2)),
			mirror: every(Duration::from_millis(250)),
			pair: every(Duration::from_secs(60)),
			link: every(Duration::from_secs(5)),
			proof: every(Duration::from_secs(1)),
			outbox: every(OUTBOX_EVERY),
			sync: every(Duration::from_secs(2)),
		}
	}
}

#[expect(clippy::too_many_lines, reason = "the loop's state and its receivers, one line each, made in one place")]
async fn run(app: AppHandle, launch: Launch, swept: Swept) -> anyhow::Result<()> {
	let Swept { dir, mut settings, loaded, keeper, region, client, posts: (posts_tx, posts), crash, crashed } = swept;
	let (session, far_lost) = open_session(&dir, &settings)?;
	let (lines, _) = broadcast::channel::<String>(1024);
	let (events_tx, events) = mpsc::unbounded_channel::<Event>();
	let (written_tx, written) = mpsc::unbounded_channel::<Written>();
	let (quits, window) = window_channels(&app);
	let (elevated_tx, elevated) = mpsc::unbounded_channel::<Elevated>();
	let (updates_tx, updates) = mpsc::unbounded_channel::<update::Found>();
	let (bles_tx, bles) = mpsc::unbounded_channel::<Ble>();
	let (ble_tx, ble) = mpsc::unbounded_channel::<Heard>();
	let feedback = Feedback::new(&dir, client.clone(), posts_tx.clone());
	let counts = CountsState::new(&dir, client.clone(), posts_tx);
	let (addon_state, addon) = AddonState::new();
	let (mut games, heard, feeds) = games_channels();
	// the runs the last quit kept go on before the first look; taken, so the start's own save drops them and a later
	// start never takes them again
	if let (Some(games), Some(kept)) = (games.as_mut(), settings.runs.take()) {
		games.resume_runs(&kept, lan::now_ms(), &settings.games);
	}
	let clock = Clock { now: Box::new(|| lan::now_ms().div_euclid(1_000)), offset_at: Box::new(local_offset_at) };
	let engine = Door::start(clock, engine_caught);
	let display = Arc::new(Mutex::new((String::new(), 1.0)));
	let cmds = socket_task(&app, &dir, &lines, &display);
	let address = find_address().await;
	let (asks_tx, asks) = mpsc::unbounded_channel::<Ask>();
	let queue = Links::new(lan::now_ms());
	let lan = lan_state(&settings, &queue, asks_tx);
	let capture = Capture::start(events_tx);
	let secrets = keeper.run();
	let (sync_link, sync) = SyncLink::new(client, address.name.clone(), secrets == RunSecrets::Unread);
	let writes = writer(&app, written_tx, keeper);
	let sitting = sitting::restored(settings.sitting, now().unix);
	let audio = Audio::start(&app, &settings.sound);
	let body = kept_body(&dir);
	let shell = Shell {
		audio,
		app: app.clone(),
		dir: dir.clone(),
		writes: writes.clone(),
		game: None,
		game_addons: None,
		followed: None,
		without_addon: None,
		sighting: Sighting::default(),
		screen: Dot::NoGame,
		stores_read: false,
		games,
		standing: Standing::default(),
		sitting,
		pro: pro::Pro::load(&dir, settings.sync.account.as_deref()),
		sync: sync_link,
		out: Vec::new(),
	};
	let publisher = app.state::<desk::Desk>().publisher.clone();
	let helper = Helper {
		desk: questpump_desk::Desk::new(shell, publisher, session, settings, body, engine),
		app,
		dir,
		secrets,
		queue,
		lan,
		served: None,
		lan_error: None,
		linked: None,
		holding: None,
		address,
		advert: dnssd::Advert::start(),
		lines,
		writes,
		due: Some(Instant::now()), // the pairing block reaches the game even before it runs
		alive_at: None,
		alive_error: None,
		batch_since: None,
		read_retry: false,
		unread: HashSet::new(),
		held_until: None,
		grant_missing: false,
		without_addon_picked: false,
		looked_for_game: None,
		written: None,
		taken_in: HashSet::new(),
		saves_read: mirror::SavesRead::default(),
		setup: None,
		loaded_before: game::running(None, &Snapshot::take()).filter(|g| !alive_fresh(&g.addons)).map(|g| g.pid),
		next: None,
		unpaired: HashSet::new(),
		unpaired_at: None,
		mirror_error: None,
		ble: None,
		bles: bles_tx,
		pad_read: None,
		ble_link: BleLinkState::new(ble_tx),
		firewall: Firewall::at_start(),
		access: Access::Fine,
		shown_folder: None,
		update: None,
		update_checks: feeds::Checks::default(),
		socket_quit: None,
		elevated: elevated_tx,
		feedback,
		crash,
		region,
		counts,
		offering: Offering::default(),
		reminding: Reminding::new(),
		addon: addon_state,
		backup: menu_pages::backup::State::default(),
	};
	let inbox = Inbox {
		events,
		cmds,
		asks,
		written,
		quits,
		window,
		elevated,
		updates,
		bles,
		ble,
		posts,
		heard,
		feeds,
		sync,
		addon,
	};
	Box::pin(serve(helper, inbox, Started { loaded, launch, crashed, far_lost }, updates_tx, capture, display)).await
}

/// How this start came: the settings read, why it started, and whether the run before crashed (the sweep found its
/// report).
struct Started {
	loaded: Loaded,
	launch: Launch,
	crashed: bool,
	/// `far.jsonl` lost rows (`open_session`).
	far_lost: bool,
}

/// What the loop takes from each source besides the ticks: the receiving ends `run` made.
struct Inbox {
	events: mpsc::UnboundedReceiver<Event>,
	cmds: mpsc::UnboundedReceiver<SocketCommand>,
	asks: mpsc::UnboundedReceiver<Ask>,
	written: mpsc::UnboundedReceiver<Written>,
	quits: mpsc::UnboundedReceiver<Quit>,
	window: mpsc::UnboundedReceiver<WindowAsk>,
	elevated: mpsc::UnboundedReceiver<Elevated>,
	updates: mpsc::UnboundedReceiver<update::Found>,
	bles: mpsc::UnboundedReceiver<Ble>,
	/// What the Bluetooth link's bridge says (slice 43).
	ble: mpsc::UnboundedReceiver<Heard>,
	/// What the outboxes' batches report.
	posts: mpsc::UnboundedReceiver<outbox::News>,
	heard: mpsc::UnboundedReceiver<games::Heard>,
	/// What the games' feeds say: Rocket League's connection and the fed games' tasks (slice 48).
	feeds: mpsc::UnboundedReceiver<games::News>,
	/// What the sync server answered (slice 60).
	sync: mpsc::UnboundedReceiver<questpump_desk::sync::News>,
	/// What the addon's feed checks, downloads and swaps report.
	addon: mpsc::UnboundedReceiver<crate::addon::News>,
}

/// The helper's start, then the one loop: every source and tick to its handler, until Quit.
async fn serve(
	mut helper: Helper,
	mut inbox: Inbox,
	started: Started,
	updates_tx: mpsc::UnboundedSender<update::Found>,
	capture: Capture,
	display: Arc<Mutex<(String, f64)>>,
) -> anyhow::Result<()> {
	helper.start(started, updates_tx).await;
	let mut ticks = Ticks::new();
	loop {
		let wake = helper.desk.pump_wake();
		tokio::select! {
			Some(event) = inbox.events.recv() => {
				keep_display(&display, &event);
				helper.on_capture(event);
			}
			Some(cmd) = inbox.cmds.recv() => helper.on_command(cmd),
			Some(ask) = inbox.asks.recv() => helper.on_ask(ask),
			Some(done) = inbox.written.recv() => helper.on_written(done),
			Some(Quit { start, again }) = inbox.quits.recv() => {
				helper.quit(start, again).await;
				return Ok(());
			}
			Some(ask) = inbox.window.recv() => helper.on_window(ask).await,
			Some(done) = inbox.elevated.recv() => helper.on_elevated(done).await,
			Some(found) = inbox.updates.recv() => helper.on_update(found),
			Some(event) = inbox.bles.recv() => helper.on_ble(event),
			Some(heard) = inbox.ble.recv() => helper.on_ble_heard(heard),
			_ = ticks.game.tick() => helper.on_look(&capture).await,
			_ = ticks.mirror.tick() => {
				helper.on_mirror_tick();
				// the main window's place, once it stood still after a move or a resize (desk/windows.rs)
				helper.place_settled();
			}
			_ = ticks.pair.tick() => helper.on_pair_tick().await,
			_ = ticks.link.tick() => helper.on_link_tick(),
			_ = ticks.proof.tick() => helper.on_proof_tick(),
			Some(news) = inbox.posts.recv() => helper.on_posted(news),
			Some(post) = inbox.heard.recv() => helper.on_gsi(post),
			Some(news) = inbox.feeds.recv() => helper.on_feed(news),
			Some(news) = inbox.sync.recv() => helper.desk.on_sync(news),
			Some(news) = inbox.addon.recv() => helper.on_addon(news),
			_ = ticks.sync.tick() => helper.desk.sync_drive(),
			Some(back) = helper.desk.engine.answer() => back(&mut helper.desk),
			() = desk::wake(wake) => helper.desk.pump_due(),
			_ = ticks.outbox.tick() => {
				helper.send_feedback();
				helper.send_crashes();
				helper.send_counts();
			}
		}
		// what the desk asked of the loop in this turn (`Helper::drain`)
		helper.drain();
		// the Log, once a turn, from whatever the turn made due (questpump-desk/src/log.rs)
		helper.desk.log_flush();
		helper.drain();
		// the game as the computer knows it now, in the next response to a phone or a watch
		helper.tell_devices();
	}
}
