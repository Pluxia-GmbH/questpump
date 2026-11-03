//! The games on the computer (docs/plan.md slices 35, 48 and 56; questpump-core `games.rs`): found means watched (slice
//! 56). Every named game this system runs is watched until the player turns it off, the Riot games (League of Legends
//! and Teamfight Tactics, Legends of Runeterra) too since 2026-10-28 (until then once the player turned them on); so
//! are the games the detector finds (`detect.rs`) and the programs the player picks, each until the player turns it off
//! (`off`). A watched game is heard at its close (its process, seen without a handle, `procs.rs`: one snapshot a look
//! shared with `game.rs`), and at each match's end where it tells QuestPump: League's game process, the fed games'
//! feeds (`fed.rs`), and once the player asks for it ("Ask after each match", `asked`) the four games whose feed needs
//! a file of QuestPump's in their folders: Counter-Strike 2 (Windows) and Dota 2 (Windows and macOS) a Game State
//! Integration cfg, found through Steam's libraries, posting to the helper's listener on 127.0.0.1; Rocket League
//! (Windows) the helper's `TAStatsAPI.ini` when it has none of the player's (nor a `DefaultStatsAPI.ini` that turns the
//! feed on), its feed read while it runs; Hearthstone a `[Power]` section added to its `log.config`, a shared file
//! turning the game off leaves. Each look (`Games::tick`, every 2 s) finds what runs, puts a set-up game's file in
//! place when it is due, tells the watcher, and asks a game that ran `SILENT_MS` without a word for its setup step; the
//! listener's posts (`on_gsi`) and the feeds' news (`on_feed`) come on their own channels. Turning a game off removes
//! its file. Which named games are installed on this computer (Steam's libraries, Epic's for Rocket League, the Riot
//! Client's records for League of Legends and Legends of Runeterra (`riot.rs`), the other fed games' records) is looked
//! for at the first look and every `FIND_EVERY_MS` (`Games::found`): those are the Games tab's tiles (`tiles.rs`), with
//! the games found and picked, each with its icon (`icons.rs`). Nothing of a game passes on but its end: no Steam ID,
//! player name, score or match id is read, logged or kept.

mod arena;
mod banner;
mod detect;
mod elite;
mod epic;
mod fed;
pub mod files;
mod gsi;
mod hearthstone;
mod icons;
mod logs;
mod mechabellum;
pub mod procs;
mod pubg;
mod riot;
mod rocket;
mod runeterra;
mod saves;
mod sc2;
#[cfg(windows)]
mod shm;
mod siege;
mod spire;
mod steam;
mod tail;
mod tarkov;
mod task;
mod tiles;
mod udp;
mod warships;
#[cfg(windows)]
mod win;

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use base64::Engine as _;
use questpump_core::games::{ComputerLink, Finish, KeptRuns, Signal, Watched, Watcher};
use questpump_core::gsi::{self as gsi_cfg, Gsi};
use questpump_core::offers::{Pause, Tunables, offer_name};
use questpump_core::rocket as rocket_ini;
use serde::Serialize;
use tokio::sync::mpsc;

pub(crate) use detect::drop_named;
pub use fed::{News, Word};
pub use gsi::Heard;
pub use tiles::{Lists, Tile};

use crate::game;
use crate::settings::{GamesSettings, Program};
use procs::{Pick, Proc, Snapshot};

/// A game that ran this long with its file in place and sent no word asks for its setup step (the cfg is read at the
/// game's start only): twice the cfg's heartbeat, since a game idle in its menus may post first at the heartbeat after
/// its load; Rocket League's feed, heard at its connection, waits as long.
const SILENT_MS: i64 = 2 * gsi_cfg::HEARTBEAT_S * 1_000;
/// A game not found, or whose file could not go in, is looked for again this often.
const LOOK_AGAIN_MS: i64 = 60_000;
/// The cfg files' token: 24 random bytes.
const TOKEN_BYTES: usize = 24;
/// The games installed on this computer are looked for again this often, never on every look.
const FIND_EVERY_MS: i64 = 10 * 60_000;

/// A game's line in its window on the Games tab (`games.status.<status>` in contract/copy/en.json); a tile carries
/// only a step owed (`Status::owed`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
	/// No feed asked for yet: turned off, or a file game not set up.
	#[default]
	Off,
	/// Not installed on this computer, as far as the helper can find.
	NotFound,
	/// Its file is in place; the game's first word comes at its next start.
	Waiting,
	/// The game ran when its file went in, or ran `SILENT_MS` without a word (StarCraft II's client API among them, slice
	/// 48): start it again.
	Restart,
	/// Dota 2 ran `SILENT_MS` without a word: its launch option is missing.
	Launch,
	/// Windows keeps the game's folder closed: the one-time elevated grant opens it.
	Permission,
	/// The grant's prompt is up.
	Asking,
	/// The player's own `TAStatsAPI.ini` turns the feed off.
	IniOff,
	/// Its file could not go in for another reason (the log says why).
	Failed,
	/// Heard from, or watched where its process is all there is to hear (League of Legends): its ends offer pumps.
	Ready,
	/// Legends of Runeterra ran `SILENT_MS` without an answer: its API is off in its settings or on another port (slice
	/// 48).
	ApiOff,
	/// A racing game whose telemetry was not heard yet: the player sends it to 127.0.0.1 at its port (slice 48).
	Udp,
	/// Another program holds a racing game's port (slice 48).
	PortTaken,
	/// F1's telemetry came in a UDP format not read here (slice 48): the player sets 2026, 2025 or 2024.
	UdpFormat,
	/// Elite Dangerous ran `SILENT_MS` and no journal of it came (slice 48).
	NoJournal,
	/// iRacing or Assetto Corsa Competizione ran `SILENT_MS` and its shared memory could not be read (slice 48).
	NoShared,
}

impl Status {
	/// Whether it is a step the player owes (a restart, a launch option, a setting, a grant, a port), which the game's
	/// tile marks with a dot; none for a game off, not found, waiting for its first word or heard.
	fn owed(self) -> bool {
		!matches!(self, Self::Off | Self::NotFound | Self::Waiting | Self::Ready)
	}
}

/// What the helper knows of a watched named game in this run.
#[derive(Debug, Default)]
struct Track {
	status: Status,
	/// The folder its file goes into, once found; while `status` asks for a grant, the nearest folder above the file
	/// that exists, which the grant opens (a folder the writer then creates in it inherits the grant).
	dir: Option<PathBuf>,
	/// When the helper last looked for its install, unix ms.
	looked: Option<i64>,
	/// Since when its process runs, unix ms.
	since: Option<i64>,
	/// Rocket League's feed port: the helper's own ini's or the player's.
	port: Option<u16>,
	/// When a fed game's feed last answered, unix ms (slice 48): a client game silent since its start asks for its API.
	answered: Option<i64>,
}

impl Track {
	/// Whether its install is due a look: never looked at, or not found or not written a while ago.
	fn due(&self, now: i64) -> bool {
		let retry = matches!(self.status, Status::NotFound | Status::Failed | Status::Permission);
		self.looked.is_none_or(|at| retry && now - at >= LOOK_AGAIN_MS)
	}
}

/// Add a game's lists (`Games::picker`): the programs it offers, and the named games it shows by name in place of
/// their programs (`Watched::link`).
#[derive(Debug, Default)]
pub struct Picker {
	pub programs: Vec<Pick>,
	pub games: Vec<&'static str>,
}

/// What a look brought: the finishes, the watched games whose process closed since the last look (`Watched::id`;
/// "Open the pump after each match"), the games started again within `match_relaunch_s` of a close that offered
/// (`Watched::id`: that offer is recalled), the watched games whose
/// process started since the last look and were not started again so (`Watched::id`: a new run, which ends what opened
/// from outside the window untouched; slice 48's double-check), the found and picked games kept by their Steam app
/// from now on, each `(from, to)` (`detect.rs`'s `migrate`: a pause while one runs names the new key), and whether the
/// games' settings changed (save them and the uninstaller's list).
#[derive(Debug, Default)]
pub struct Ticked {
	pub finishes: Vec<Finish>,
	pub closed: Vec<String>,
	pub resumed: Vec<String>,
	pub started: Vec<String>,
	pub rekeyed: Vec<(String, String)>,
	pub changed: bool,
}

/// The games on the computer, owned by the helper's loop.
pub struct Games {
	t: &'static Tunables,
	/// contract/links.json's `computer` list, as built in.
	links: &'static [ComputerLink],
	/// The state machine every finish comes from; `watcher.playing` and `watcher.in_match` answer the sync response
	/// and the card.
	pub(crate) watcher: Watcher,
	tracks: HashMap<&'static str, Track>,
	/// What of the watched games ran at the last look, by `Watched::id`.
	running: Vec<String>,
	/// What of the watched games was open at the last look, by `Watched::id`: those running, and the named games one of
	/// whose other own programs ran (`Games::opens`), for Pump home's recent games alone.
	open: Vec<String>,
	/// The listener while CS2 or Dota 2 is set up, and when it last failed to start.
	server: Option<gsi::Server>,
	server_failed: Option<i64>,
	heard: mpsc::UnboundedSender<Heard>,
	/// Rocket League's feed while the game runs, with the port it reads.
	client: Option<(rocket::Client, u16)>,
	news: mpsc::UnboundedSender<News>,
	/// The named games installed on this computer, in contract/links.json's order, the install folders of those whose
	/// records name one (their icons' programs are under them), and when they were looked for (unix ms).
	found: Vec<&'static str>,
	dirs: HashMap<&'static str, PathBuf>,
	found_at: Option<i64>,
	/// The fed games' readers (`fed.rs`) and the detector (`detect.rs`), slice 48.
	feeds: fed::Feeds,
	detector: detect::Detector,
	/// The games' icon files (`icons.rs`, slice 56).
	icons: icons::Icons,
	/// World of Warcraft while Pump home's recent games may show it (`set_wow`): its icon is made as a game's is.
	wow: Option<Wow>,
}

/// World of Warcraft's program as Pump home's recent games show its icon (`desk::recent`, 2026-10-27): its install's
/// flavour folder (`_classic_beta_` and the like, the client's executable or app bundle in it) and its client's process
/// while one runs, whose program comes first.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Wow {
	pub(crate) root: Option<PathBuf>,
	pub(crate) pid: Option<u32>,
}

/// Whether a write failed for the folder's permissions, which only Windows' elevated grant opens.
fn denied(e: &std::io::Error) -> bool {
	cfg!(windows) && e.kind() == std::io::ErrorKind::PermissionDenied
}

/// Whether a named game is installed on this computer: CS2 and Dota 2 in a Steam library, Rocket League's folder
/// for its ini (Steam or Epic), League of Legends where the Riot Client's record names it, else at its default place
/// (`riot::install`), a fed game as its records tell (`fed::installed`). A Steam library on a volume under `/Volumes`
/// (macOS) is read only for a game the player asked for (`volumes`: `asked`).
fn installed(watched: &Watched, volumes: bool) -> bool {
	match watched {
		Watched::CounterStrike | Watched::Dota => steam::game_dir(watched, volumes).is_some(),
		Watched::RocketLeague => rocket::config_dir().is_some(),
		Watched::League => riot::install(watched).is_some(),
		Watched::Program { .. } => false,
		fed => fed::installed(fed, volumes),
	}
}

/// A named game's install folder, where its records name one: a Steam game's in a Steam library, League of Legends'
/// as the Riot Client's record names it (`riot::install`), a fed game's as its records tell (`fed::install_dir`);
/// `volumes` as `installed`'s.
fn install_dir(watched: &Watched, volumes: bool) -> Option<PathBuf> {
	match watched {
		Watched::League => riot::install(watched),
		Watched::CounterStrike | Watched::Dota | Watched::RocketLeague => steam::game_dir(watched, volumes),
		Watched::Program { .. } => None,
		fed => fed::install_dir(fed, volumes),
	}
}

/// A new token for the cfg files.
fn new_token() -> String {
	let mut bytes = [0u8; TOKEN_BYTES];
	rand::fill(&mut bytes);
	base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

/// Whether the games' readers may read under a path where it stands: on macOS never in a folder whose read asks the
/// player's consent (the Desktop, Documents and Downloads folders, iCloud Drive, the cloud storage folders), and on
/// another volume (`/Volumes`) only for a game the player asked for (`volumes`: `asked`); on Windows every path on this
/// computer's own disks, never a network share a Steam library list names (`install::on_this_computer`).
pub(crate) fn readable(path: &Path, volumes: bool) -> bool {
	const CONSENT: [&str; 5] =
		// copy-lint: not shown (the folders' paths)
		["Desktop", "Documents", "Downloads", "Library/Mobile Documents", "Library/CloudStorage"];
	if cfg!(windows) {
		return crate::install::on_this_computer(path);
	}
	if dirs::home_dir().is_some_and(|home| CONSENT.iter().any(|folder| path.starts_with(home.join(folder)))) {
		return false;
	}
	volumes || !path.starts_with("/Volumes")
}

/// Whether the named game `w` is watched (slice 56): not turned off.
pub(crate) fn watches(settings: &GamesSettings, w: &Watched) -> bool {
	!settings.is_off(w.link())
}

/// Whether the named game `w` is heard by its feed: watched and, a game whose feed needs a file of QuestPump's, set up
/// ("Ask after each match", `asked`).
pub(crate) fn hears(settings: &GamesSettings, w: &Watched) -> bool {
	watches(settings, w) && (!w.writes_file() || settings.is_asked(w.link()))
}

/// The programs watched (slice 56): the games found and the programs picked that the player did not turn off, as the
/// watcher keeps them (a picked program a found game's key holds already is the found one).
fn programs(settings: &GamesSettings) -> impl Iterator<Item = Watched> + '_ {
	let found = settings.detected.iter().map(|d| (&d.key, &d.name));
	let picked = settings.programs.iter().filter(|p| settings.detected.iter().all(|d| d.key != p.key));
	found
		.chain(picked.map(|p| (&p.key, &p.name)))
		.filter(|(key, _)| !settings.is_off(key))
		.map(|(key, name)| Watched::Program { key: key.clone(), name: name.clone() })
}

/// Whether the game `id` (`Watched::id`) is watched now.
fn is_watched(settings: &GamesSettings, id: &str) -> bool {
	match Watched::from_key(id) {
		Some(named) => watches(settings, &named),
		None => programs(settings).any(|w| w.id() == id),
	}
}

/// Marks a game heard with its current file; true when that is news for the settings.
fn heard(settings: &mut GamesSettings, key: &str) -> bool {
	if settings.heard.iter().any(|k| k == key) {
		return false;
	}
	settings.heard.push(key.to_owned());
	log::info!("games: {key}: heard");
	true
}

impl Games {
	/// The games with the offers' numbers and the built-in list, reporting on these channels.
	pub fn new(
		t: &'static Tunables,
		links: &'static [ComputerLink],
		heard: mpsc::UnboundedSender<Heard>,
		news: mpsc::UnboundedSender<News>,
	) -> Self {
		Self {
			t,
			links,
			watcher: Watcher::new(t.min_run_s, t.league_min_s, t.checkpoint_s, t.relaunch_s),
			tracks: HashMap::new(),
			running: Vec::new(),
			open: Vec::new(),
			server: None,
			server_failed: None,
			heard,
			client: None,
			news,
			found: Vec::new(),
			dirs: HashMap::new(),
			found_at: None,
			feeds: fed::Feeds::default(),
			detector: detect::Detector::default(),
			icons: icons::Icons::default(),
			wow: None,
		}
	}

	/// The named games this system runs, in contract/links.json's order.
	fn named(&self) -> impl Iterator<Item = Watched> + '_ {
		let system = if cfg!(windows) { "windows" } else { "macos" };
		self.links
			.iter()
			.filter(move |l| l.systems.iter().any(|s| s == system))
			.filter_map(|l| Watched::from_key(&l.key))
	}

	/// The offers' numbers.
	pub fn tunables(&self) -> &'static Tunables {
		self.t
	}

	/// Goes on with the runs a quit kept (`Watcher::restore`) of the games `tick` steps now (the named games not turned
	/// off, the programs found or picked), before the first look; the log counts them, never a game's name.
	pub fn resume_runs(&mut self, kept: &KeptRuns, now: i64, settings: &GamesSettings) {
		let watched: Vec<Watched> =
			Watched::named().iter().filter(|w| watches(settings, w)).cloned().chain(programs(settings)).collect();
		let taken = self.watcher.restore(kept, now, &watched);
		if taken > 0 {
			log::info!("games: {taken} runs the last quit kept go on");
		}
	}

	/// Whether a process is the named game `w`'s: on Windows one of its image names; on macOS its image path
	/// (`Watched::is_mac_image`: League's game bundle, StarCraft II's, MTG Arena's and Hearthstone's apps, Dota 2's
	/// binary under its Steam folder).
	fn is_process_of(w: &Watched, p: &Proc) -> bool {
		if cfg!(windows) {
			return w.windows_images().iter().any(|image| p.name.eq_ignore_ascii_case(image));
		}
		p.path.as_deref().is_some_and(|path| w.is_mac_image(path))
	}

	/// One look: the listener and the files, which games run, the watcher's finishes, the setup steps, the feed, the
	/// detector's look and the icons.
	pub fn tick(&mut self, snap: &Snapshot, now: i64, settings: &mut GamesSettings) -> Ticked {
		if self.found_at.is_none_or(|at| now - at >= FIND_EVERY_MS) {
			let asked = |w: &Watched| settings.is_asked(w.link());
			self.found = self.named().filter(|w| installed(w, asked(w))).map(|w| w.link()).collect();
			self.dirs = self.named().filter_map(|w| Some((w.link(), install_dir(&w, asked(&w))?))).collect();
			self.found_at = Some(now);
		}
		let mut ticked = Ticked { changed: self.serve(now, settings), ..Ticked::default() };
		let before = std::mem::take(&mut self.running);
		// a fed game's close goes to the watcher after its feed's last lines (`feeds_look`): an end written as the game
		// closed still ends its match before the close forgets it; a running one goes before them, so a feed's first
		// word (MTG Arena's header line, read at the reader's first look) finds the run open and tells it
		let mut fed_closes = Vec::new();
		for watched in Watched::named() {
			let key = watched.link();
			if !watches(settings, watched) {
				continue;
			}
			let running = snap.procs.iter().any(|p| Self::is_process_of(watched, p));
			let track = self.tracks.entry(key).or_default();
			let started = running && track.since.is_none();
			track.since = if running { track.since.or(Some(now)) } else { None };
			if started && matches!(track.status, Status::Restart | Status::Launch | Status::ApiOff) {
				// a new start reads the file: the step asked was the run before's
				track.status = Status::Waiting;
			}
			if started && matches!(track.status, Status::NoJournal | Status::NoShared | Status::UdpFormat) {
				// a fed game's step its last run asked: this run is heard anew
				track.status = Self::fed_initial(watched, settings);
			}
			// a game seen starting (or running at the helper's start) gets its file checked: one moved or deleted since
			// goes back in, and one that goes in while the game runs asks for a new start; a file game not set up has
			// none
			if hears(settings, watched) && (track.due(now) || (started && track.status != Status::Ready)) {
				ticked.changed |= self.place(watched, now, settings);
			}
			if running {
				self.running.push(key.to_owned());
				self.check(watched, now, settings);
				self.check_fed(watched, now);
			}
			let process = Signal::Process { watched: watched.clone(), running, at: now };
			if fed::Feed::of(watched).is_some() && !running {
				fed_closes.push(process);
			} else {
				ticked.finishes.extend(self.watcher.step(process));
			}
		}
		for watched in programs(settings) {
			let running = snap.procs.iter().any(|p| self.is_program(watched.id(), p));
			if running {
				self.running.push(watched.id().to_owned());
			}
			ticked.finishes.extend(self.watcher.step(Signal::Process { watched, running, at: now }));
		}
		self.feed(settings);
		let (finishes, changed) = self.feeds_look(snap, now, settings);
		ticked.finishes.extend(finishes);
		for process in fed_closes {
			ticked.finishes.extend(self.watcher.step(process));
		}
		// once every process of this look was stepped, the fed games' too: a game started again within
		// `match_relaunch_s` of a close that offered is resumed in this look, never also a new run
		ticked.resumed = self.watcher.take_resumed().iter().map(|w| w.id().to_owned()).collect();
		self.open = self.running.clone();
		for watched in Watched::named() {
			let key = watched.link();
			if watches(settings, watched)
				&& !self.open.iter().any(|r| r == key)
				&& snap.procs.iter().any(|p| self.opens(watched, p))
			{
				self.open.push(key.to_owned());
			}
		}
		ticked.started =
			self.running.iter().filter(|id| !before.contains(id) && !ticked.resumed.contains(id)).cloned().collect();
		// a game turned off or a program removed meanwhile closed nothing the player still watches
		ticked.closed =
			before.into_iter().filter(|id| !self.running.contains(id)).filter(|id| is_watched(settings, id)).collect();
		let (noticed, rekeyed) = self.notice(snap, now, settings);
		ticked.changed |= changed | noticed;
		ticked.rekeyed = rekeyed;
		// a program no longer watched (a found game that made room or that Steam calls no game, one whose new key
		// another row held) leaves no run behind, which would count as a match until the next start
		let watched: Vec<Watched> = programs(settings).collect();
		self.watcher.forget_programs(|w| watched.contains(w));
		self.icons_look(snap, settings);
		ticked
	}

	/// What ran at the last look, by `Watched::id`: the pause's "while this game runs" names the first.
	pub fn running(&self) -> &[String] {
		&self.running
	}

	/// What was open at the last look, by `Watched::id`: what ran, and a named game while another of its own programs
	/// runs (League's client out of a match; Rapha, 2026-10-28). Pump home's recent games alone read it: a match's end
	/// and an offer follow `running`.
	pub fn open(&self) -> &[String] {
		&self.open
	}

	/// Whether a process shows the named game `w` open: one of its own programs (`named_games_of`: its game, its
	/// launcher or client, its Steam app's), never one that starts other games too (the Riot Client,
	/// `games::is_riot_client`; Battle.net, Steam and the other stores' launchers belong to no game's own programs) nor
	/// iRacing's service, which Windows keeps running whether or not the player opened the game.
	fn opens(&self, w: &Watched, p: &Proc) -> bool {
		let service =
			|image: &str| questpump_core::games::IRACING_SERVICES.iter().any(|s| s.eq_ignore_ascii_case(image));
		!questpump_core::games::is_riot_client(&p.name, p.path.as_deref())
			&& !service(&p.name)
			&& self.named_games_of(p).any(|g| g == w)
	}

	/// World of Warcraft while Pump home's recent games may show it, none while they cannot: its icon is kept and made
	/// at the looks meanwhile (`icons_look`), and its file goes with it.
	pub(crate) fn set_wow(&mut self, wow: Option<Wow>) {
		self.wow = wow;
	}

	/// The asset URL of World of Warcraft's icon, while its file stands.
	pub(crate) fn wow_icon(&self) -> Option<String> {
		self.icons.url(crate::desk::recent::WOW)
	}

	/// The processes of the watched game `id` (`Watched::id`: a named game's key, a found or picked program's own) in
	/// a snapshot: the card looks for its window, and whether it is in front.
	pub fn pids(&self, id: &str, snap: &Snapshot) -> Vec<u32> {
		let named = Watched::from_key(id);
		snap.procs
			.iter()
			.filter(|p| match &named {
				Some(w) => Self::is_process_of(w, p),
				None => self.is_program(id, p),
			})
			.map(|p| p.pid)
			.collect()
	}

	/// The name a watched game (`Watched::id`) shows: a named game's contract/links.json name, a found or picked
	/// program's own.
	pub fn name_of(id: &str, settings: &GamesSettings) -> Option<String> {
		match Watched::from_key(id) {
			Some(named) => Some(named.name().to_owned()),
			None => programs(settings).find(|w| w.id() == id).map(|w| w.name().to_owned()),
		}
	}

	/// The listener runs while CS2 or Dota 2 is set up, on the kept port and with the kept token, made at its first
	/// start; a new port puts both cfg files in again. A listener that cannot start is tried again a minute later.
	fn serve(&mut self, now: i64, settings: &mut GamesSettings) -> bool {
		if !Gsi::ALL.iter().any(|g| hears(settings, &g.watched())) {
			self.server = None;
			return false;
		}
		if self.server.is_some() || self.server_failed.is_some_and(|at| now - at < LOOK_AGAIN_MS) {
			return false;
		}
		let mut changed = settings.gsi_token.is_none();
		let token = settings.gsi_token.get_or_insert_with(new_token).clone();
		match gsi::start(settings.gsi_port, token, self.heard.clone()) {
			Ok(server) => {
				self.server_failed = None;
				if settings.gsi_port != Some(server.port) {
					settings.gsi_port = Some(server.port);
					changed = true;
				}
				self.server = Some(server);
				// the cfg files name the port: each is looked at again now
				for g in Gsi::ALL {
					self.tracks.entry(g.watched().link()).or_default().looked = None;
				}
			}
			Err(e) => {
				log::warn!("games: no listener on 127.0.0.1: {e}");
				self.server_failed = Some(now);
				for g in Gsi::ALL {
					self.tracks.entry(g.watched().link()).or_default().status = Status::Failed;
				}
			}
		}
		changed
	}

	/// Looks for a heard game's install and puts its file in place (none for a game whose feed needs none); true when
	/// the settings changed.
	fn place(&mut self, watched: &Watched, now: i64, settings: &mut GamesSettings) -> bool {
		self.tracks.entry(watched.link()).or_default().looked = Some(now);
		match watched {
			Watched::CounterStrike => self.place_cfg(Gsi::CounterStrike, settings),
			Watched::Dota => self.place_cfg(Gsi::Dota, settings),
			Watched::RocketLeague => self.place_ini(settings),
			Watched::League | Watched::Program { .. } => {
				self.tracks.entry(watched.link()).or_default().status = Status::Ready;
				false
			}
			Watched::StarCraft
			| Watched::MtgArena
			| Watched::Hearthstone
			| Watched::Runeterra
			| Watched::FormulaOne
			| Watched::Forza
			| Watched::IRacing
			| Watched::AssettoCorsa
			| Watched::Elite
			| Watched::Mechabellum
			| Watched::SlayTheSpire
			| Watched::AssettoCorsaOne
			| Watched::GuildWars
			| Watched::PathOfExile
			| Watched::Warframe
			| Watched::Warships
			| Watched::Siege
			| Watched::Tarkov
			| Watched::Deadlock
			| Watched::Fortnite
			| Watched::TeamFortress
			| Watched::Naraka
			| Watched::FallGuys
			| Watched::DeadByDaylight
			| Watched::Hunt
			| Watched::Valorant
			| Watched::Pubg
			| Watched::Halo
			| Watched::Brawlhalla
			| Watched::RFactor
			| Watched::LeMans
			| Watched::Automobilista
			| Watched::Osu
			| Watched::Stardew
			| Watched::Warcraft
			| Watched::EldenRing
			| Watched::Nightreign => self.place_fed(watched, settings),
		}
	}

	/// The status a game with its file in place has: heard, or waiting for its start (a step asked stays asked).
	fn in_place(track: &Track, settings: &GamesSettings, key: &str) -> Status {
		if settings.heard.iter().any(|k| k == key) {
			Status::Ready
		} else if matches!(track.status, Status::Restart | Status::Launch) && track.since.is_some() {
			track.status
		} else {
			Status::Waiting
		}
	}

	/// What a write's outcome makes of a game: a file in place waits for the game's start, or asks for a new one
	/// when the game ran as it went in; a refusal asks for the grant (Windows) or fails.
	fn written(track: &mut Track, key: &str, dir: PathBuf, outcome: std::io::Result<()>) {
		match outcome {
			Ok(()) => {
				log::info!("games: {key}: its file went in");
				track.status = if track.since.is_some() { Status::Restart } else { Status::Waiting };
				track.dir = Some(dir);
			}
			Err(e) => {
				log::warn!("games: {key}: its file could not go in: {e}");
				track.status = if denied(&e) { Status::Permission } else { Status::Failed };
				// never a folder still missing (Dota 2's `gamestate_integration`): the grant on `cfg` reaches it
				track.dir = dir.ancestors().find(|d| d.is_dir()).map(Path::to_path_buf);
			}
		}
	}

	/// CS2's or Dota 2's cfg, written when it is missing or names another port or token.
	fn place_cfg(&mut self, game: Gsi, settings: &mut GamesSettings) -> bool {
		let key = game.watched().link();
		let port = self.server.as_ref().map(|s| s.port);
		let app = steam::game_dir(&game.watched(), true);
		let track = self.tracks.entry(key).or_default();
		let (Some(port), Some(token)) = (port, settings.gsi_token.clone()) else {
			track.status = Status::Failed;
			return false;
		};
		let Some(dir) = app.as_deref().and_then(|app| steam::cfg_dir(game, app)) else {
			track.status = Status::NotFound;
			return false;
		};
		let path = dir.join(gsi_cfg::FILE);
		let text = gsi_cfg::cfg(game, port, &token);
		if files::listed(settings, key, &path) && std::fs::read_to_string(&path).is_ok_and(|t| t == text) {
			track.status = Self::in_place(track, settings, key);
			track.dir = Some(dir);
			return false;
		}
		let outcome = files::write(settings, key, &path, &text);
		let changed = outcome.is_ok();
		if changed {
			settings.heard.retain(|k| k != key);
		}
		Self::written(track, key, dir, outcome);
		changed
	}

	/// Rocket League's ini: the player's own is read, never written over (one that turns the feed off asks them to
	/// turn it on); without one, the game's `DefaultStatsAPI.ini` is read when it turns the feed on (the player's, and
	/// nothing goes in), else the helper's goes in.
	fn place_ini(&mut self, settings: &mut GamesSettings) -> bool {
		let key = Watched::RocketLeague.link();
		let track = self.tracks.entry(key).or_default();
		let Some(dir) = rocket::config_dir() else {
			track.status = Status::NotFound;
			return false;
		};
		let path = dir.join(rocket_ini::INI);
		let ours = rocket_ini::ini(rocket_ini::PORT);
		match std::fs::read_to_string(&path) {
			// the helper's own file, listed again if the list lost it (its text is the helper's to the byte)
			Ok(text) if text == ours => {
				let changed = !files::listed(settings, key, &path);
				if changed {
					files::list(settings, key, &path);
				}
				track.port = Some(rocket_ini::PORT);
				track.status = Self::in_place(track, settings, key);
				track.dir = Some(dir);
				changed
			}
			Ok(text) => {
				// the player's file, or the helper's as they changed it: theirs from now on, never removed
				let changed = !files::take(settings, key).is_empty();
				track.port = rocket_ini::ini_port(&text);
				track.status = if track.port.is_some() { Self::in_place(track, settings, key) } else { Status::IniOff };
				track.dir = Some(dir);
				changed
			}
			Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
				let default = std::fs::read_to_string(dir.join(rocket_ini::DEFAULT_INI)).ok();
				if let Some(port) = default.as_deref().and_then(rocket_ini::ini_port) {
					let changed = !files::take(settings, key).is_empty();
					track.port = Some(port);
					track.status = Self::in_place(track, settings, key);
					track.dir = Some(dir);
					return changed;
				}
				let outcome = files::write(settings, key, &path, &ours);
				let changed = outcome.is_ok();
				if changed {
					settings.heard.retain(|k| k != key);
					track.port = Some(rocket_ini::PORT);
				}
				Self::written(track, key, dir, outcome);
				changed
			}
			Err(e) => {
				log::warn!("games: {key}: {} could not be read: {e}", rocket_ini::INI);
				track.status = Status::Failed;
				false
			}
		}
	}

	/// A running game with its file in place that sent no word for `SILENT_MS` asks for its step: Dota 2 its launch
	/// option, CS2 and Rocket League a new start (they read the file at their start only).
	fn check(&mut self, watched: &Watched, now: i64, settings: &GamesSettings) {
		let key = watched.link();
		let Some(track) = self.tracks.get_mut(key) else { return };
		let silent = track.since.is_some_and(|since| now - since >= SILENT_MS);
		if track.status != Status::Waiting || !silent || settings.heard.iter().any(|k| k == key) {
			return;
		}
		track.status = match watched {
			Watched::Dota => Status::Launch,
			Watched::CounterStrike | Watched::RocketLeague => Status::Restart,
			// the fed games' steps: `check_fed`, and their feeds' own words
			Watched::League
			| Watched::Program { .. }
			| Watched::StarCraft
			| Watched::MtgArena
			| Watched::Hearthstone
			| Watched::Runeterra
			| Watched::FormulaOne
			| Watched::Forza
			| Watched::IRacing
			| Watched::AssettoCorsa
			| Watched::Elite
			| Watched::Mechabellum
			| Watched::SlayTheSpire
			| Watched::AssettoCorsaOne
			| Watched::GuildWars
			| Watched::PathOfExile
			| Watched::Warframe
			| Watched::Warships
			| Watched::Siege
			| Watched::Tarkov
			| Watched::Deadlock
			| Watched::Fortnite
			| Watched::TeamFortress
			| Watched::Naraka
			| Watched::FallGuys
			| Watched::DeadByDaylight
			| Watched::Hunt
			| Watched::Valorant
			| Watched::Pubg
			| Watched::Halo
			| Watched::Brawlhalla
			| Watched::RFactor
			| Watched::LeMans
			| Watched::Automobilista
			| Watched::Osu
			| Watched::Stardew
			| Watched::Warcraft
			| Watched::EldenRing
			| Watched::Nightreign => return,
		};
		log::info!("games: {key}: no word {} s after its start", SILENT_MS / 1_000);
	}

	/// Rocket League's feed: connected while the game runs set up with its feed on, at the port its ini names.
	fn feed(&mut self, settings: &GamesSettings) {
		let key = Watched::RocketLeague.link();
		let port = self.tracks.get(key).and_then(|t| t.port);
		let heard = hears(settings, &Watched::RocketLeague);
		let wanted = port.filter(|_| heard && self.running.iter().any(|r| r == key));
		match (wanted, &self.client) {
			(Some(port), Some((_, at))) if *at == port => {}
			(Some(port), _) => self.client = Some((rocket::start(port, self.news.clone()), port)),
			(None, _) => self.client = None,
		}
	}

	/// An authenticated post: the game is heard, and its phase goes to the watcher. A post from a game turned off
	/// a moment ago is dropped.
	pub fn on_gsi(&mut self, (game, phase): Heard, now: i64, settings: &mut GamesSettings) -> (Option<Finish>, bool) {
		let key = game.watched().link();
		if !hears(settings, &game.watched()) {
			return (None, false);
		}
		let changed = heard(settings, key);
		self.tracks.entry(key).or_default().status = Status::Ready;
		(self.watcher.step(Signal::Gsi { game, phase, at: now }), changed)
	}

	/// What a feed on the games' channel says: Rocket League's connection, or a fed game's word (`fed.rs`).
	pub fn on_feed(&mut self, news: News, now: i64, settings: &mut GamesSettings) -> (Option<Finish>, bool) {
		match news {
			News::Rocket(news) => self.on_rocket(news, now, settings),
			News::Fed(watched, word) => self.word(&watched, word, now, settings),
		}
	}

	/// What Rocket League's feed says: the connection heard (its run told for the watcher), an event for the watcher,
	/// the connection gone.
	fn on_rocket(&mut self, news: rocket::News, now: i64, settings: &mut GamesSettings) -> (Option<Finish>, bool) {
		let key = Watched::RocketLeague.link();
		if !hears(settings, &Watched::RocketLeague) {
			return (None, false);
		}
		match news {
			rocket::News::Up => {
				let changed = heard(settings, key);
				self.tracks.entry(key).or_default().status = Status::Ready;
				(self.watcher.step(Signal::Answered { watched: Watched::RocketLeague, at: now }), changed)
			}
			rocket::News::Event(event) => (self.watcher.step(Signal::Rocket { event, at: now }), false),
			rocket::News::Gone => (self.watcher.step(Signal::RocketGone { at: now }), false),
		}
	}

	/// Whether the game a pause holds for (`Watched::id`) runs: its process, or its run still open.
	pub fn paused_runs(&self, pause: &Pause) -> bool {
		match pause {
			Pause::WhileRunning { game } => self.running.iter().any(|r| r == game) || self.watcher.runs(game),
			Pause::None | Pause::Until { .. } | Pause::UntilResumed => false,
		}
	}

	/// Whether a named game is a tile on the Games tab: on this computer, installed here or running (Rapha, 2026-10-27:
	/// a Riot game turned on under More games jumped into the grid, the others' switches did not); the others of this
	/// system are under More games, whose switch only says whether QuestPump watches for it, whatever it says.
	fn listed(&self, w: &Watched) -> bool {
		let key = w.link();
		self.found.contains(&key) || self.running.iter().any(|r| r == key)
	}

	/// A game's switch, "Offer a pump after each match" or "when you close" it (`games.set`, any tile's key or one
	/// under More games): on takes it from `off` (a file game's file goes in only by "Ask after each match", `setup`);
	/// off puts it in `off`, takes it from `asked`, removes what the helper wrote for
	/// it (a Rocket League ini the player changed stays, and so does Hearthstone's shared `log.config`) and forgets its
	/// run. False for a key no game has.
	pub fn set(&mut self, key: &str, on: bool, settings: &mut GamesSettings) -> bool {
		let named = Watched::from_key(key);
		let program = programs(settings).find(|w| w.id() == key);
		let known = named.is_some()
			|| settings.detected.iter().any(|d| d.key == key)
			|| settings.programs.iter().any(|p| p.key == key);
		if !known {
			return false;
		}
		if on {
			settings.off.retain(|k| k != key);
			if let Some(w) = &named {
				self.tracks.remove(w.link());
			}
			return true;
		}
		if !settings.is_off(key) {
			settings.off.push(key.to_owned());
		}
		match named {
			Some(watched) => {
				settings.asked.retain(|k| k != key);
				self.take_files(&watched, settings);
				self.watcher.forget(&watched);
			}
			None => {
				if let Some(program) = program {
					self.watcher.forget(&program);
				}
			}
		}
		true
	}

	/// "Ask after each match" (`games.setup`) for a game whose feed needs a file of QuestPump's: it joins `asked` (and
	/// leaves `off`), and its file goes in at the next look (Windows' one-time grant where its folder asks for one);
	/// whether that changed anything. Refused for another key.
	pub fn setup(&mut self, key: &str, settings: &mut GamesSettings) -> Result<bool, String> {
		let watched = Watched::from_key(key).filter(Watched::writes_file);
		let watched = watched.ok_or_else(|| format!("games.setup: {key} needs no file"))?;
		let off = settings.is_off(key);
		settings.off.retain(|k| k != key);
		if settings.is_asked(key) {
			return Ok(off);
		}
		settings.asked.push(key.to_owned());
		self.tracks.remove(watched.link());
		Ok(true)
	}

	/// What the helper wrote for a game goes, as its feed does (a Rocket League ini the player changed stays, and so
	/// does Hearthstone's shared `log.config`).
	fn take_files(&mut self, watched: &Watched, settings: &mut GamesSettings) {
		let key = watched.link();
		settings.heard.retain(|k| k != key);
		for path in files::take(settings, key) {
			let ours = *watched != Watched::RocketLeague
				|| std::fs::read_to_string(&path).is_ok_and(|t| t == rocket_ini::ini(rocket_ini::PORT));
			if ours {
				files::remove(key, &path);
			}
		}
		self.tracks.remove(key);
	}

	/// A program the player picked joins the watched ones, under its name as an offer shows it (`offer_name`: a
	/// window's title on Windows can be longer), and leaves the hidden ones ("Not a game") where it was one, so a game
	/// is never both hidden and listed; one found or picked already is only turned on again (taken from `off`). Whether
	/// that changed anything.
	pub fn add_program(pick: &Pick, settings: &mut GamesSettings) -> bool {
		let off = settings.is_off(&pick.key);
		settings.off.retain(|k| *k != pick.key);
		let hidden = settings.dismissed.len();
		settings.dismissed.retain(|d| d.key != pick.key);
		let shown = settings.dismissed.len() != hidden;
		if settings.detected.iter().any(|d| d.key == pick.key) || settings.programs.iter().any(|p| p.key == pick.key) {
			return off || shown;
		}
		settings.programs.push(Program { key: pick.key.clone(), name: offer_name(&pick.name).to_owned() });
		true
	}

	/// A picked program leaves the watched ones ("Remove"), and its off with it.
	pub fn remove_program(&mut self, key: &str, settings: &mut GamesSettings) -> bool {
		let watched = programs(settings).find(|w| w.id() == key);
		let Some(at) = settings.programs.iter().position(|p| p.key == key) else { return false };
		settings.programs.remove(at);
		settings.off.retain(|k| k != key);
		if let Some(watched) = watched {
			self.watcher.forget(&watched);
		}
		true
	}

	/// The picker's programs: those with a window now, World of Warcraft's clients, every program of a named game
	/// (its game, launcher or client, its Steam app's: `named_games_of`) and the refused games (Roblox:
	/// questpump-core's `detect`) left out.
	pub fn picks(&self, snap: &Snapshot) -> Vec<Pick> {
		self.sort_picks(snap).0
	}

	/// Add a game's lists (`games.pick`): the picker's programs (`picks`), and the named games not watched, one of
	/// whose programs has a window now (its game, its launcher or client: the Riot Client for League and Legends of
	/// Runeterra), in their contract/links.json order, which the picker shows by name in place of those programs, so a
	/// pick turns the game on rather than watching its launcher.
	pub fn picker(&self, snap: &Snapshot, settings: &GamesSettings) -> Picker {
		let (programs, open) = self.sort_picks(snap);
		let games =
			self.named().filter(|w| open.contains(w.link()) && !watches(settings, w)).map(|w| w.link()).collect();
		Picker { programs, games }
	}

	/// The programs with a window now, split: those the picker offers, and the named games (`Watched::link`) of the
	/// ones it leaves out.
	fn sort_picks(&self, snap: &Snapshot) -> (Vec<Pick>, HashSet<&'static str>) {
		#[cfg(windows)]
		let client = |p: &Proc| game::is_client_name(&p.name) || questpump_core::detect::refused_image(&p.name);
		#[cfg(not(windows))]
		let client = |p: &Proc| {
			p.path
				.as_deref()
				.is_some_and(|path| game::is_client_path(path) || questpump_core::detect::refused_path(path))
		};
		let mut open = HashSet::new();
		let programs = procs::pick_list(snap)
			.into_iter()
			.filter(|pick| {
				let Some(p) = snap.procs.iter().find(|p| p.pid == pick.pid) else { return true };
				let mut named = false;
				for w in self.named_games_of(p) {
					open.insert(w.link());
					named = true;
				}
				!named && !client(p)
			})
			.collect();
		(programs, open)
	}

	/// The folder a game's grant would open, its status then asking.
	pub fn grant_target(&mut self, key: &str) -> Option<PathBuf> {
		let track = self.tracks.get_mut(key).filter(|t| t.status == Status::Permission)?;
		track.status = Status::Asking;
		track.dir.clone()
	}

	/// The grant came back: the file is tried again at the next look; a declined or failed grant asks again.
	pub fn granted(&mut self, key: &str, done: bool) {
		if let Some(track) = self.tracks.get_mut(key).filter(|t| t.status == Status::Asking) {
			if done {
				track.looked = None;
				track.status = Status::Waiting;
			} else {
				track.status = Status::Permission;
			}
		}
	}
}
