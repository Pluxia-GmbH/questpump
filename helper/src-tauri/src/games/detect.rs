//! Noticing the games the player plays (docs/plan.md slices 48 and 56; docs/computer-games.md §6.1): every `EVERY_MS`,
//! while the Games tab's "Notice games I play" is on (it is by default), the programs with a window that the picker
//! would list (`Games::picks`: World of Warcraft's clients, every program of a named game and the refused games left
//! out, no process opened) are held against the records that say a program is a game. On Windows: Steam's own
//! `logs/gameprocess_log.txt` (the processes Steam tracks for a game, from its last `LOOK_BACK` bytes, its lines
//! stamped before this start of Windows dropped and the whole log ignored while no `steam.exe` runs; the game named by
//! its `appmanifest_<app>.acf` and kept as `steam:<app>`, one row for its launcher and its game, running while Steam
//! tracks any process of it), Windows' Known Games List (`HKCU\System\GameConfigStore\Children\*` `MatchedExeFullPath`,
//! which Game Bar fills), the Epic Games Launcher's manifests (each launch executable and the `.exe` files of its
//! install), GOG Galaxy's `Games` keys (each game's `exe` and its folder), and the `.exe` files in the install folders
//! the Battle.net Agent's `product.db` names (`install::battlenet_games`: World of Warcraft's, the Agent's and the
//! Battle.net app's left out) and Ubisoft Connect's (`HKLM\SOFTWARE\WOW6432Node\Ubisoft\Launcher\Installs\<id>`
//! `InstallDir`, as Playnite's Ubisoft library reads it), each install's folder and `EXE_DEPTH` folders below it: a
//! program is seen by its image name alone (its path would take a handle), so a game whose image those folders hold is
//! one. On macOS: an image under a Steam library's `steamapps/common`, by its path alone (a game a manifest on the
//! startup volume names kept as `steam:<app>`; a library under `/Volumes` is never read, so a game there keeps its
//! program's path), or an app in `/Applications` or `~/Applications` whose `Info.plist` says it is a game
//! (`LSApplicationCategoryType`, `LSSupportsGameMode`), read once an app; an app anywhere else is never looked into,
//! since a read on a removable or network volume, in the Desktop, Documents or Downloads folder, iCloud Drive or a
//! cloud storage folder asks the player's consent, and the detector asks for no grant. The records that change only
//! with an install (`Records`) are read again every `REFRESH_MS` on a thread of their own, the last ones kept
//! meanwhile. A found game's name comes from a record, never a window's title: its Steam manifest's, Epic's
//! `DisplayName`, GOG's `gameName`, its program's version resource (`ProductName`, read from the file), its app's
//! `CFBundleDisplayName` or `CFBundleName`, else its program's file name. Launchers, shells, browsers, runtimes and
//! anti-cheat services are never games, nor are the Steam apps that run all day (desktop pets, focus rooms, idle games;
//! `detect::denied_app`), Roblox is refused (questpump-core's `detect`), and the programs the player picked by hand
//! stay theirs. A process Steam tracks under its test app (Spacewar, 480, which many games borrow) is a game kept and
//! named by its program, never as that app. A Steam app is noticed only when Steam's own record calls it a game, a demo
//! or a mod (`appcache/appinfo.vdf`, questpump-core's `appinfo`, read on the records' thread for the apps not asked
//! yet, the answers kept for the run: Wallpaper Engine and Steam's Tools and Applications never ask), an app the record
//! gives no type staying a game; one seen before its answer waits for the next read, which starts at once, and so does
//! a game in a Steam library on the startup volume (macOS) until the first read of its manifests is in. A found Steam
//! game the record calls something else leaves the found ones (`no_games`: one found before slice 56, when no type was
//! read, or kept by its program's key until `migrate` gave it its app's). A game noticed joins
//! `settings.games.detected` with its name and its program's key alone, watched from then on until the player turns it
//! off (slice 56: the Games tab's tiles); at most `MOST`, a new one taking the place of the one not running that closed
//! longest ago (`Detected::closed`, its latest close; the player's off list is apart from it, so a game turned off
//! stays off when it is noticed again). "Not a game" moves it to `settings.games.dismissed`, which is never noticed
//! again until the player puts it back. The log says only that one was noticed, never its name. A found or picked game
//! kept by its program's key that turns out to be a Steam game takes `steam:<app>` (`migrate`), its run and its off
//! going on under the new key; one a later build names (Mechabellum, 2026-10-28) leaves its list for the named game,
//! its off with it (`drop_named`). With noticing off no new game is found (the games found keep offering), and the Steam
//! games watched are still told by Steam's own records alone (`steam_records`: the manifests, and on Windows Steam's
//! process log), and nothing else is read.

#[cfg(target_os = "macos")]
use std::collections::HashMap as BundleMap;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::thread::JoinHandle;

use questpump_core::games::Watched;
use questpump_core::offers::offer_name;
use questpump_core::{appinfo, detect};

use super::icons::{Source, bundle_source};
use super::procs::{Pick, Proc, Snapshot};
use super::{Games, programs, steam};
use crate::settings::{Detected, GamesSettings};

/// How often the programs with a window are held against the records.
const EVERY_MS: i64 = 10_000;
/// How often the records that change only with an install are read again.
const REFRESH_MS: i64 = 10 * 60_000;
/// How much of Steam's process log's end is read: the lines of the games that run now.
#[cfg(windows)]
const LOOK_BACK: u64 = 64 * 1024;
/// How many folders below a launcher's install folder its `.exe` files are looked for in (Anno 1800's `Bin\Win64`,
/// Warcraft III's `_retail_\x86_64`, an Unreal game's `<Game>\Binaries\Win64`), and the most folders read for one
/// install.
#[cfg(windows)]
const EXE_DEPTH: usize = 3;
#[cfg(windows)]
const EXE_FOLDERS: usize = 256;
/// The most games kept noticed: the one not running that closed longest ago makes room for a new one, which waits
/// while every one runs.
const MOST: usize = 64;
/// A Steam game's key, before its app id.
const STEAM: &str = "steam:";

/// A program a launcher's record names on Windows: the game's name when the record gives one, and the program's file
/// when the record says where it is.
#[cfg(windows)]
#[derive(Debug, Clone, Default)]
struct Known {
	name: Option<String>,
	path: Option<PathBuf>,
}

/// The records that change only with an install, read off the loop.
#[derive(Default)]
struct Records {
	/// The games the Steam manifests name (on macOS the startup volume's alone).
	steam: Vec<steam::App>,
	/// The image names Windows' Known Games List and the launchers' records name, in lower case (none in a read for
	/// the watched Steam games alone, `steam_records`).
	#[cfg(windows)]
	known: HashMap<String, Known>,
	/// The type Steam's own record gives each app this read asked (`steam::app_types`), none where it gives none.
	kinds: HashMap<u32, Option<String>>,
}

/// What a read of the records asks of Steam's own record: the type of every app its manifests name and of those
/// `wanted` (seen running before an answer, and the found Steam games', a game found before slice 56 among them, whose
/// type was never asked), but the apps `known` (answered earlier in the run).
#[derive(Default)]
struct Ask {
	known: HashSet<u32>,
	wanted: HashSet<u32>,
}

impl Ask {
	/// The types of the apps the manifests `apps` name and of those wanted, those known left out.
	fn types(self, apps: &[steam::App]) -> HashMap<u32, Option<String>> {
		let mut ask: Vec<u32> =
			apps.iter().map(|a| a.id).chain(self.wanted).filter(|a| !self.known.contains(a)).collect();
		ask.sort_unstable();
		ask.dedup();
		if ask.is_empty() { HashMap::new() } else { steam::app_types(&ask) }
	}
}

/// What an app's `Info.plist` says, read once an app.
#[cfg(target_os = "macos")]
#[derive(Debug, Clone, Default)]
struct Bundle {
	game: bool,
	name: Option<String>,
}

/// What the detector keeps between looks.
#[derive(Default)]
pub(super) struct Detector {
	/// When it last looked, unix ms.
	looked: Option<i64>,
	records: Records,
	/// When the records were last asked for, whether that read took the launchers' records too or the Steam manifests
	/// alone (noticing off: `steam_records`), and the read under way.
	records_at: Option<i64>,
	records_whole: bool,
	reading: Option<JoinHandle<Records>>,
	/// Whether a read was taken in during this run (macOS: a game in a startup volume's Steam library waits for it,
	/// since its manifest may name it a Steam app of another type).
	#[cfg(target_os = "macos")]
	records_in: bool,
	/// The type of every Steam app asked in this run (`Records::kinds`), and the apps seen running before their answer,
	/// which the next read asks.
	kinds: HashMap<u32, Option<String>>,
	wanted: HashSet<u32>,
	/// The processes Steam tracks now, by process id, with their app (Windows; empty while no `steam.exe` runs).
	tracked: HashMap<u32, u32>,
	/// This start of Windows as Steam's log stamps its lines (local time), once known.
	#[cfg(windows)]
	boot: Option<String>,
	/// What each app bundle seen says.
	#[cfg(target_os = "macos")]
	bundles: BundleMap<PathBuf, Bundle>,
	/// The found games that ran at the last look, by key: one gone since has closed.
	running: HashSet<String>,
}

/// A Steam game's key.
fn steam_key(app: u32) -> String {
	format!("{STEAM}{app}")
}

/// The app id of a Steam game's key; none for another key.
fn steam_app(key: &str) -> Option<u32> {
	key.strip_prefix(STEAM)?.parse().ok()
}

/// This start of Windows in local time, as Steam's process log stamps its lines (`YYYY-MM-DD HH:MM:SS`).
#[cfg(windows)]
fn boot_stamp() -> Option<String> {
	let now = crate::lan::now_ms().div_euclid(1_000);
	let boot = super::win::boot_at(now);
	let local = time::OffsetDateTime::from_unix_timestamp(boot + crate::runtime::local_offset_at(boot)).ok()?;
	local.format(time::macros::format_description!("[year]-[month]-[day] [hour]:[minute]:[second]")).ok()
}

/// The processes Steam tracks for its games now, each with its app: the end of its process log, its lines from before
/// `boot` dropped.
#[cfg(windows)]
fn steam_tracked(boot: Option<&str>) -> HashMap<u32, u32> {
	use std::io::{Read as _, Seek as _, SeekFrom};
	let Some(path) = steam::root().map(|r| r.join("logs").join("gameprocess_log.txt")) else {
		return HashMap::new();
	};
	let Ok(mut file) = std::fs::File::open(path) else { return HashMap::new() };
	let len = file.metadata().map_or(0, |m| m.len());
	let from = len.saturating_sub(LOOK_BACK);
	let mut bytes = Vec::new();
	if file.seek(SeekFrom::Start(from)).is_err() || file.take(LOOK_BACK).read_to_end(&mut bytes).is_err() {
		return HashMap::new();
	}
	let text = String::from_utf8_lossy(&bytes);
	// a read that starts inside a line skips to the next one
	let text = if from > 0 { text.split_once('\n').map_or("", |(_, rest)| rest) } else { &text };
	detect::steam_tracked(text, boot)
}

/// Ubisoft Connect's installs: each game's `InstallDir` under the launcher's `Installs` key, in the registry's 32-bit
/// view (where the launcher writes it), then its 64-bit one; written with `/`, read as Windows' paths.
#[cfg(windows)]
pub(super) fn ubisoft_installs() -> Vec<PathBuf> {
	use windows::Win32::System::Registry::HKEY_LOCAL_MACHINE;
	use windows::core::{HSTRING, PCWSTR, w};
	const INSTALLS: [&str; 2] =
		[r"SOFTWARE\WOW6432Node\Ubisoft\Launcher\Installs", r"SOFTWARE\Ubisoft\Launcher\Installs"];
	INSTALLS
		.iter()
		.flat_map(|installs| {
			super::win::subkeys(HKEY_LOCAL_MACHINE, installs).into_iter().filter_map(move |id| {
				let key = HSTRING::from(format!(r"{installs}\{id}"));
				super::win::text(HKEY_LOCAL_MACHINE, PCWSTR(key.as_ptr()), w!("InstallDir"))
			})
		})
		.map(|dir| PathBuf::from(dir.replace('/', "\\")))
		.filter(|dir| dir.is_dir())
		.collect()
}

/// A GOG Galaxy game as its `Games\<id>` key holds it: its `gameName`, its program (`exe`, else its `launchCommand`)
/// and its folder (`path`).
#[cfg(windows)]
struct Gog {
	name: Option<String>,
	exe: Option<String>,
	folder: Option<PathBuf>,
}

/// GOG Galaxy's games, in the registry's 32-bit view (where GOG writes them, as `GameFinder` reads them), then its
/// 64-bit one.
#[cfg(windows)]
fn gog_games() -> Vec<Gog> {
	use windows::Win32::System::Registry::HKEY_LOCAL_MACHINE;
	use windows::core::{HSTRING, PCWSTR, w};
	const GAMES: [&str; 2] = [r"SOFTWARE\WOW6432Node\GOG.com\Games", r"SOFTWARE\GOG.com\Games"];
	GAMES
		.iter()
		.flat_map(|games| {
			super::win::subkeys(HKEY_LOCAL_MACHINE, games).into_iter().map(move |id| {
				let key = HSTRING::from(format!(r"{games}\{id}"));
				let value = |name| super::win::text(HKEY_LOCAL_MACHINE, PCWSTR(key.as_ptr()), name);
				Gog {
					name: value(w!("gameName")),
					exe: value(w!("exe")).or_else(|| value(w!("launchCommand"))),
					folder: value(w!("path")).map(PathBuf::from).filter(|d| d.is_dir()),
				}
			})
		})
		.collect()
}

/// A program a record names, under the name the record gives: a record named first keeps its name and its file.
#[cfg(windows)]
fn name_known(path: &str, name: Option<&str>, into: &mut HashMap<String, Known>) {
	let Some(image) = detect::image_of(path) else { return };
	let entry = into.entry(image).or_default();
	if entry.name.is_none() {
		entry.name = name.map(str::to_owned);
	}
	if entry.path.is_none() {
		entry.path = Some(PathBuf::from(path.trim().trim_matches('"'))).filter(|p| p.is_absolute());
	}
}

/// The `.exe` files in an install's folder and the folders up to `EXE_DEPTH` below it, at most `EXE_FOLDERS` folders
/// read, nearest first, each under the install's `name`; a link to another folder is never followed, and an image a
/// record named first keeps its record.
#[cfg(windows)]
fn exe_images(install: &Path, name: Option<&str>, into: &mut HashMap<String, Known>) {
	let mut folders = std::collections::VecDeque::from([(install.to_path_buf(), 0)]);
	for _ in 0..EXE_FOLDERS {
		let Some((folder, depth)) = folders.pop_front() else { break };
		let Ok(entries) = std::fs::read_dir(&folder) else { continue };
		for entry in entries.flatten() {
			let Ok(kind) = entry.file_type() else { continue };
			let path = entry.path();
			if kind.is_dir() && depth < EXE_DEPTH {
				folders.push_back((path, depth + 1));
			} else if kind.is_file() && path.extension().is_some_and(|e| e.eq_ignore_ascii_case("exe")) {
				name_known(&path.to_string_lossy(), name, into);
			}
		}
	}
}

/// The image names Windows' Known Games List, the Epic Games Launcher's manifests and installs, GOG Galaxy's games
/// and the Battle.net and Ubisoft installs name, each with the name and the file its record gives.
#[cfg(windows)]
fn known_games() -> HashMap<String, Known> {
	use windows::Win32::System::Registry::HKEY_CURRENT_USER;
	use windows::core::{HSTRING, PCWSTR, w};
	const CHILDREN: &str = r"System\GameConfigStore\Children";
	let mut known = HashMap::new();
	for child in super::win::subkeys(HKEY_CURRENT_USER, CHILDREN) {
		let key = HSTRING::from(format!(r"{CHILDREN}\{child}"));
		if let Some(path) = super::win::text(HKEY_CURRENT_USER, PCWSTR(key.as_ptr()), w!("MatchedExeFullPath")) {
			name_known(&path, None, &mut known);
		}
	}
	for item in super::epic::items() {
		let install = item.install_dir();
		if let Some(launch) = item.launch_executable.as_deref() {
			let full = install.as_ref().map_or_else(|| PathBuf::from(launch), |d| d.join(launch));
			name_known(&full.to_string_lossy(), item.display_name.as_deref(), &mut known);
		}
		if let Some(install) = install {
			exe_images(&install, item.display_name.as_deref(), &mut known);
		}
	}
	for gog in gog_games() {
		if let Some(exe) = gog.exe.as_deref() {
			name_known(exe, gog.name.as_deref(), &mut known);
		}
		if let Some(folder) = gog.folder.as_deref() {
			exe_images(folder, gog.name.as_deref(), &mut known);
		}
	}
	for install in crate::install::battlenet_games().into_iter().chain(ubisoft_installs()) {
		exe_images(&install, None, &mut known);
	}
	known
}

/// The records, read on the refresh's own thread, with the Steam apps' types `ask` asks.
fn records(ask: Ask) -> Records {
	let steam = steam::apps();
	let kinds = ask.types(&steam);
	Records {
		steam,
		#[cfg(windows)]
		known: known_games(),
		kinds,
	}
}

/// The records the Steam games watched need while "Notice games I play" is off, read on the refresh's own thread: the
/// Steam manifests alone, never Windows' Known Games List, the launchers' records nor Steam's own record of types
/// (nothing is noticed).
fn steam_records() -> Records {
	Records {
		steam: steam::apps(),
		#[cfg(windows)]
		known: HashMap::new(),
		kinds: HashMap::new(),
	}
}

/// The folders whose apps' `Info.plist` may be read: `/Applications` and `~/Applications`, which no privacy consent
/// guards (a removable or network volume, the Desktop, Documents and Downloads folders, iCloud Drive and the cloud
/// storage folders each ask the player at a read, so an app there is never looked into).
#[cfg(target_os = "macos")]
pub(super) fn app_folders() -> Vec<PathBuf> {
	let mut folders = vec![PathBuf::from("/Applications")];
	folders.extend(dirs::home_dir().map(|h| h.join("Applications")));
	folders
}

/// What an app bundle's `Info.plist` says: whether it is a game (a games category, or Game Mode), and its name.
#[cfg(target_os = "macos")]
fn read_bundle(bundle: &Path) -> Bundle {
	let Ok(info) = plist::Value::from_file(bundle.join("Contents").join("Info.plist")) else {
		return Bundle::default();
	};
	let Some(info) = info.as_dictionary() else { return Bundle::default() };
	let text = |key: &str| info.get(key).and_then(plist::Value::as_string).filter(|t| !t.trim().is_empty());
	let game = text("LSApplicationCategoryType").is_some_and(detect::is_game_category)
		|| info.get("LSSupportsGameMode").and_then(plist::Value::as_boolean) == Some(true);
	Bundle { game, name: text("CFBundleDisplayName").or_else(|| text("CFBundleName")).map(str::to_owned) }
}

/// A program file's name without its extension: the last name a found game may take.
#[cfg(windows)]
fn stem(name: &str) -> String {
	Path::new(name).file_stem().map_or_else(|| name.to_owned(), |s| s.to_string_lossy().into_owned())
}

impl Detector {
	/// The records again when they are due (`REFRESH_MS`), or at once while a Steam app seen running waits for its
	/// type, read on a thread of their own while the last ones stay; a finished read taken in, its types kept for the
	/// run. `whole`: every record (noticing on), else the Steam manifests alone (`steam_records`, for the Steam games
	/// watched while noticing is off); noticing turned on again after such a read reads them all at once.
	fn refresh(&mut self, now: i64, whole: bool, found: &[Detected]) {
		if self.reading.as_ref().is_some_and(JoinHandle::is_finished) {
			match self.reading.take().map(JoinHandle::join) {
				Some(Ok(mut records)) => {
					self.kinds.extend(std::mem::take(&mut records.kinds));
					self.records = records;
					#[cfg(target_os = "macos")]
					{
						self.records_in = true;
					}
					// an app seen running while this read was under way that it answered needs no read again
					self.wanted.retain(|app| !self.kinds.contains_key(app));
				}
				Some(Err(_)) => log::warn!("games: the records' read ended in a panic"),
				None => {}
			}
		}
		let wider = whole && !self.records_whole;
		let wanted = whole && !self.wanted.is_empty();
		if self.reading.is_some() || (!wider && !wanted && self.records_at.is_some_and(|at| now - at < REFRESH_MS)) {
			return;
		}
		self.records_at = Some(now);
		self.records_whole = whole;
		let found = found.iter().filter_map(|d| steam_app(&d.key));
		let wanted = std::mem::take(&mut self.wanted).into_iter().chain(found).collect();
		let ask = Ask { known: self.kinds.keys().copied().collect(), wanted };
		let read = move || if whole { records(ask) } else { steam_records() };
		match std::thread::Builder::new().name("games-records".to_owned()).spawn(read) {
			Ok(reading) => self.reading = Some(reading),
			Err(e) => log::warn!("games: the records could not be read: {e}"),
		}
	}

	/// Whether Steam's own record lets the Steam app `app` be noticed, once its type was asked in this run: it calls it
	/// a game, a demo or a mod, or gives it no type (it stays a game).
	fn typed_game(&self, app: u32) -> Option<bool> {
		self.kinds.get(&app).map(|kind| kind.as_deref().is_none_or(appinfo::noticed))
	}

	/// Whether the Steam app `app` may be noticed: never one that runs all day (questpump-core's `detect::denied_app`),
	/// else by its type (`typed_game`); one not answered yet waits, and the next read asks it.
	fn steam_game(&mut self, app: u32) -> bool {
		if detect::denied_app(app) {
			return false;
		}
		if let Some(game) = self.typed_game(app) {
			return game;
		}
		self.wanted.insert(app);
		false
	}

	/// The found Steam games that are no game leave the found ones: those Steam's own record calls no game, demo or mod
	/// (a game found before slice 56, when no type was read, and one kept by its program's key until `migrate` gave it
	/// its app's), those that run all day (`detect::denied_app`, 2026-10-28) and one kept by Steam's test app
	/// (`detect::borrowed_app`), which is noticed again by its program: whether any did, logged by their count alone. A
	/// type not read keeps the game.
	fn no_games(&self, settings: &mut GamesSettings) -> bool {
		let before = settings.detected.len();
		settings.detected.retain(|d| {
			steam_app(&d.key).is_none_or(|app| {
				!detect::denied_app(app) && !detect::borrowed_app(app) && self.typed_game(app) != Some(false)
			})
		});
		let gone = before - settings.detected.len();
		if gone > 0 {
			log::info!("games: {gone} found games left the found ones, no game by Steam's record or our own list");
		}
		gone > 0
	}

	/// Where a running process of the game `key` gives its icon: macOS the app bundle around it; Windows a Steam
	/// game's folder with this process's image (no other path is read: a process's would take a handle).
	pub(super) fn icon_seen(&self, key: &str, p: &Proc) -> Option<Source> {
		if cfg!(windows) {
			let dir = steam_app(key).and_then(|app| self.steam_dir(app))?;
			return Some(Source::Folder { dir: dir.to_path_buf(), images: vec![p.name.clone()] });
		}
		p.path.as_deref().and_then(bundle_source)
	}

	/// Where the records give a found or picked game's icon: a Steam game's folder (macOS: the first app bundle in
	/// it), on macOS the app bundle its key names, on Windows the program a launcher's record names for its image.
	pub(super) fn icon_source(&self, key: &str) -> Option<Source> {
		if let Some(app) = steam_app(key) {
			let dir = self.steam_dir(app)?.to_path_buf();
			return Some(Source::Folder { dir, images: Vec::new() });
		}
		#[cfg(windows)]
		{
			// a Windows key is its image name in lower case, as the records are kept
			self.records.known.get(key).and_then(|known| known.path.clone()).map(Source::File)
		}
		#[cfg(not(windows))]
		{
			bundle_source(Path::new(key))
		}
	}

	/// The processes Steam tracks now (Windows): none while no `steam.exe` runs, whose log then says nothing of now.
	#[cfg(windows)]
	fn track(&mut self, snap: &Snapshot) {
		if !snap.procs.iter().any(|p| p.name.eq_ignore_ascii_case("steam.exe")) {
			self.tracked.clear();
			return;
		}
		if self.boot.is_none() {
			self.boot = boot_stamp();
		}
		self.tracked = steam_tracked(self.boot.as_deref());
	}

	/// The folder of the Steam game `app`, as the records name it.
	fn steam_dir(&self, app: u32) -> Option<&Path> {
		self.records.steam.iter().find(|a| a.id == app).map(|a| a.dir.as_path())
	}

	/// The name the Steam game `app`'s manifest gives it.
	fn steam_name(&self, app: u32) -> Option<String> {
		self.records.steam.iter().find(|a| a.id == app).and_then(|a| a.name.clone())
	}

	/// The Steam game a process belongs to: Steam tracks it (Windows), or it runs from a game's folder a manifest
	/// names (macOS); never Steam's test app, which names no game (`detect::borrowed_app`).
	fn steam_of(&self, p: &Proc) -> Option<u32> {
		let app = if cfg!(windows) {
			self.tracked.get(&p.pid).copied()
		} else {
			let path = p.path.as_deref()?;
			self.records.steam.iter().find(|a| path.starts_with(&a.dir)).map(|a| a.id)
		};
		app.filter(|app| !detect::borrowed_app(*app))
	}

	/// Whether a process is one of a named game's own programs that is not its game process: its launcher or client
	/// (`Watched::is_family_image`, `Watched::is_mac_family`), or a process of its Steam app (Windows: Steam tracks it
	/// under the game's app; macOS: it runs from the game's Steam folder).
	fn of_named(&self, w: &Watched, p: &Proc) -> bool {
		let steam = self.steam_of(p).is_some_and(|app| w.steam_apps().contains(&app));
		let family = if cfg!(windows) {
			w.is_family_image(&p.name)
		} else {
			p.path.as_deref().is_some_and(|path| w.is_mac_family(path))
		};
		steam || family
	}

	/// Whether a process is the found Steam game `app`'s (`steam:<app>`): Steam tracks it under that app (Windows), or
	/// it runs from the game's folder (macOS).
	fn runs_steam(&self, app: u32, p: &Proc) -> bool {
		if cfg!(windows) {
			return self.tracked.get(&p.pid) == Some(&app);
		}
		self.steam_dir(app).zip(p.path.as_deref()).is_some_and(|(dir, path)| path.starts_with(dir))
	}

	/// The picks that are games, each with its key and the name it is kept under, a Steam game once.
	#[cfg(windows)]
	fn games(&mut self, picks: &[Pick], snap: &Snapshot) -> Vec<(String, String)> {
		let mut out: Vec<(String, String)> = Vec::new();
		for pick in picks {
			let Some(image) = pick.image.as_deref() else { continue };
			if detect::denied_image(image) {
				continue;
			}
			// any process of the pick's program that Steam tracks makes it that Steam game, launcher or game; one Steam
			// tracks under its test app (Spacewar, which many games borrow) is a game kept by its program and named by
			// its own records, never by that app
			let procs = || snap.procs.iter().filter(|p| p.name.eq_ignore_ascii_case(image));
			let app = procs().find_map(|p| self.steam_of(p));
			let borrowed = procs().any(|p| self.tracked.get(&p.pid).is_some_and(|a| detect::borrowed_app(*a)));
			let found = if let Some(app) = app {
				if !self.steam_game(app) {
					continue;
				}
				Some((steam_key(app), self.steam_name(app).unwrap_or_else(|| stem(image))))
			} else {
				let known = self.records.known.get(&image.to_lowercase());
				(known.is_some() || borrowed).then(|| {
					let name = known.and_then(|known| {
						known.name.clone().or_else(|| known.path.as_deref().and_then(super::win::product_name))
					});
					(pick.key.clone(), name.unwrap_or_else(|| stem(image)))
				})
			};
			if let Some(found) = found.filter(|(key, _)| out.iter().all(|(k, _)| k != key)) {
				out.push(found);
			}
		}
		out
	}

	/// The picks that are games, each with its key and the name it is kept under, a Steam game once.
	#[cfg(target_os = "macos")]
	fn games(&mut self, picks: &[Pick], snap: &Snapshot) -> Vec<(String, String)> {
		let commons = steam::commons();
		let apps = app_folders();
		let mut out: Vec<(String, String)> = Vec::new();
		for pick in picks {
			let Some(proc) = snap.procs.iter().find(|p| p.pid == pick.pid) else { continue };
			let Some(path) = proc.path.as_deref() else { continue };
			let app = super::icons::bundle_of(path);
			if app.and_then(Path::file_name).and_then(|n| n.to_str()).is_some_and(detect::denied_bundle) {
				continue;
			}
			// a Steam library's game by its path alone (its manifest on the startup volume names it, and until the
			// first read of the manifests is in such a game waits, as its type does); an app's Info.plist only in the
			// apps' folders
			let found = if let Some(id) = self.steam_of(proc) {
				if !self.steam_game(id) {
					continue;
				}
				Some((steam_key(id), self.steam_name(id).unwrap_or_else(|| pick.name.clone())))
			} else if commons.iter().any(|c| path.starts_with(c)) {
				if !self.records_in && steam::read_here(path) {
					continue;
				}
				Some((pick.key.clone(), pick.name.clone()))
			} else {
				app.filter(|b| apps.iter().any(|a| b.starts_with(a))).and_then(|b| {
					let read = self.bundles.entry(b.to_path_buf()).or_insert_with(|| read_bundle(b));
					read.game.then(|| (pick.key.clone(), read.name.clone().unwrap_or_else(|| pick.name.clone())))
				})
			};
			if let Some(found) = found.filter(|(key, _)| out.iter().all(|(k, _)| k != key)) {
				out.push(found);
			}
		}
		out
	}
}

/// Whether a game's key is among `list`'s.
fn listed(list: &[Detected], key: &str) -> bool {
	list.iter().any(|d| d.key == key)
}

/// A found or picked game's key `from` is `to` from now on (a Steam game found before it was kept by its app, by its
/// program's key): every list that holds it takes the new key, the player's off list too, and one that holds both
/// keeps the row it had for `to`.
fn rekey(settings: &mut GamesSettings, from: &str, to: &str) -> bool {
	let mut changed = false;
	if let Some(at) = settings.off.iter().position(|k| k == from) {
		if settings.is_off(to) {
			settings.off.remove(at);
		} else {
			to.clone_into(&mut settings.off[at]);
		}
		changed = true;
	}
	for list in [&mut settings.detected, &mut settings.dismissed] {
		if let Some(at) = list.iter().position(|d| d.key == from) {
			if listed(list, to) {
				list.remove(at);
			} else {
				to.clone_into(&mut list[at].key);
			}
			changed = true;
		}
	}
	if let Some(at) = settings.programs.iter().position(|p| p.key == from) {
		if settings.programs.iter().any(|p| p.key == to) {
			settings.programs.remove(at);
		} else {
			to.clone_into(&mut settings.programs[at].key);
		}
		// a program the player added stays theirs: it is no hidden game too (`Games::add_program`)
		settings.dismissed.retain(|d| d.key != to);
		changed = true;
	}
	changed
}

/// The named game that a found or picked game's key is one of: its Steam app's key (`steam:<app>`) or, on Windows,
/// one of its game's image names. A game found before a build named it (Mechabellum, named on 2026-10-28) is that
/// named game from then on (`drop_named`).
fn named_program(key: &str) -> Option<&'static Watched> {
	Watched::named().iter().find(|w| {
		steam_app(key).is_some_and(|app| w.steam_apps().contains(&app))
			|| w.windows_images().iter().any(|image| image.eq_ignore_ascii_case(key))
	})
}

/// The found, hidden and picked games that are a named game's (`named_program`) leave their lists, so the named game
/// shows alone, by its name, and is heard by its feed: a game the player turned off or called no game turns its named
/// game off (`off`), and its own off goes. How many left; each logged by its named game's key alone (a found game's
/// key may be a path in the player's folders). At each start (`Settings::load`) and after each look's `migrate`, which
/// may give a game found by its program's key its Steam app's.
pub(crate) fn drop_named(settings: &mut GamesSettings) -> usize {
	let keys: Vec<(String, bool)> = settings
		.detected
		.iter()
		.map(|d| (d.key.clone(), false))
		.chain(settings.dismissed.iter().map(|d| (d.key.clone(), true)))
		.chain(settings.programs.iter().map(|p| (p.key.clone(), false)))
		.collect();
	let mut dropped = 0;
	for (key, hidden) in keys {
		let Some(named) = named_program(&key) else { continue };
		if (hidden || settings.is_off(&key)) && !settings.is_off(named.link()) {
			settings.off.push(named.link().to_owned());
		}
		settings.off.retain(|k| *k != key);
		settings.detected.retain(|d| d.key != key);
		settings.dismissed.retain(|d| d.key != key);
		settings.programs.retain(|p| p.key != key);
		log::info!("games: a found game is {} now", named.link());
		dropped += 1;
	}
	dropped
}

impl Games {
	/// Whether a found or picked game's key is a Steam game's (`steam:<app>`), watched through Steam's own records.
	pub fn is_steam_key(key: &str) -> bool {
		steam_app(key).is_some()
	}

	/// A look at the programs that run (every `EVERY_MS`): the records refreshed, Steam's tracked processes read, the
	/// found and picked Steam games kept by their app, the found Steam games Steam calls no game dropped (`no_games`),
	/// the found games' closes kept, and, while noticing is on, each new game running that is neither found, put away
	/// nor picked joins the found ones: whether the settings changed, and the keys `migrate` changed
	/// (`Ticked::rekeyed`). With noticing off nothing is read but what the Steam games watched need (none of them:
	/// nothing at all): the Steam manifests (`steam_records`) and, on Windows, Steam's process log, never the Known
	/// Games List nor a launcher's records.
	pub(super) fn notice(
		&mut self,
		snap: &Snapshot,
		now: i64,
		settings: &mut GamesSettings,
	) -> (bool, Vec<(String, String)>) {
		if self.detector.looked.is_some_and(|at| now - at < EVERY_MS) {
			return (false, Vec::new());
		}
		self.detector.looked = Some(now);
		if settings.notice_off {
			// no found game's close is kept meanwhile, so noticing turned on again starts from what runs then
			self.detector.running.clear();
			self.detector.wanted.clear();
			if !programs(settings).any(|w| Self::is_steam_key(w.id())) {
				self.detector.tracked.clear();
				return (false, Vec::new());
			}
		}
		self.detector.refresh(now, !settings.notice_off, &settings.detected);
		// Steam's process log is Windows' alone
		#[cfg(windows)]
		self.detector.track(snap);
		let rekeyed = self.migrate(snap, settings);
		let mut changed = !rekeyed.is_empty();
		changed |= drop_named(settings) > 0;
		changed |= self.detector.no_games(settings);
		if settings.notice_off {
			return (changed, rekeyed);
		}
		let picks = self.picks(snap);
		let games = self.detector.games(&picks, snap);
		changed |= self.closes(&games, now, settings);
		for (key, name) in &games {
			let known = listed(&settings.detected, key)
				|| listed(&settings.dismissed, key)
				|| settings.programs.iter().any(|p| &p.key == key);
			if known {
				continue;
			}
			// at the most, the found game not running now (no window of it, no process of it at this look) that closed
			// longest ago makes room (one never seen closing by when it was found); with none, the new one waits
			// (nothing changes, nothing is logged), so two games never take a place from each other look by look
			while settings.detected.len() >= MOST {
				let room = settings
					.detected
					.iter()
					.enumerate()
					.filter(|(_, d)| {
						games.iter().all(|(running, _)| *running != d.key) && !self.running.contains(&d.key)
					})
					.min_by_key(|(_, d)| d.closed.unwrap_or(d.seen))
					.map(|(at, _)| at);
				let Some(at) = room else { break };
				settings.detected.remove(at);
				changed = true;
			}
			if settings.detected.len() >= MOST {
				continue;
			}
			log::info!("games: a game noticed");
			let found = Detected {
				key: key.clone(),
				name: offer_name(name).to_owned(),
				seen: now.div_euclid(1_000),
				closed: None,
			};
			settings.detected.insert(0, found);
			changed = true;
		}
		(changed, rekeyed)
	}

	/// The found and picked games kept by their program's key that are Steam games take the key of their app
	/// (`steam:<app>`): on Windows once Steam tracks a process of that program, on macOS once a manifest names the
	/// folder their program runs from. A watched game's run, a found game's or a picked program's, goes on under its
	/// new key (`Watcher::rekey`), and so does its place among the games that ran at the last look, so the change is
	/// neither a close nor a new start, and its real close still offers. The log names the new key alone (an old one
	/// may be a path in the player's folders). The keys changed, each `(from, to)`.
	fn migrate(&mut self, snap: &Snapshot, settings: &mut GamesSettings) -> Vec<(String, String)> {
		let keys: Vec<String> = settings
			.detected
			.iter()
			.chain(&settings.dismissed)
			.map(|d| d.key.clone())
			.chain(settings.programs.iter().map(|p| p.key.clone()))
			.filter(|k| steam_app(k).is_none())
			.collect();
		let mut rekeyed = Vec::new();
		for key in keys {
			let app = if cfg!(windows) {
				snap.procs.iter().filter(|p| p.name.eq_ignore_ascii_case(&key)).find_map(|p| self.detector.steam_of(p))
			} else {
				self.detector.records.steam.iter().find(|a| Path::new(&key).starts_with(&a.dir)).map(|a| a.id)
			};
			let Some(app) = app else { continue };
			let to = steam_key(app);
			let watched = programs(settings).find(|w| w.id() == key);
			if !rekey(settings, &key, &to) {
				continue;
			}
			// the game now watched under `to` (its own row, where both keys were listed) takes the run on
			if let Some(from) = watched
				&& let Some(kept) = programs(settings).find(|w| w.id() == to)
			{
				self.watcher.rekey(&from, kept);
			}
			if let Some(at) = self.running.iter().position(|id| *id == key) {
				if self.running.contains(&to) {
					self.running.remove(at);
				} else {
					to.clone_into(&mut self.running[at]);
				}
			}
			log::info!("games: a game kept by its Steam app from now on ({to})");
			rekeyed.push((key, to));
		}
		rekeyed
	}

	/// The found games that ran at the last look and are gone now have closed: each one's latest close is kept
	/// (`Detected::closed`), by which the found game that closed longest ago makes room for a new one.
	fn closes(&mut self, games: &[(String, String)], now: i64, settings: &mut GamesSettings) -> bool {
		let running: HashSet<String> = games.iter().map(|(key, _)| key.clone()).collect();
		let gone: Vec<String> = self.detector.running.difference(&running).cloned().collect();
		self.detector.running = running;
		let mut changed = false;
		for key in gone {
			if let Some(found) = settings.detected.iter_mut().find(|d| d.key == key) {
				found.closed = Some(now.div_euclid(1_000));
				changed = true;
			}
		}
		changed
	}

	/// Whether the found Steam game or the picked program `key` runs as the process `p`: a Steam game while Steam
	/// tracks it under its app (Windows) or it runs from its folder (macOS); a program on Windows by its image name, on
	/// macOS inside its bundle or its executable.
	pub(super) fn is_program(&self, key: &str, p: &Proc) -> bool {
		if let Some(app) = steam_app(key) {
			return self.detector.runs_steam(app, p);
		}
		if cfg!(windows) {
			return p.name.eq_ignore_ascii_case(key);
		}
		p.path.as_deref().is_some_and(|path| path.starts_with(Path::new(key)))
	}

	/// The named games a process is one of the own programs of: its game, its launcher or client (the Riot Client is
	/// League's and Legends of Runeterra's), its Steam app's.
	pub(super) fn named_games_of<'a>(&'a self, p: &'a Proc) -> impl Iterator<Item = &'static Watched> + 'a {
		Watched::named().iter().filter(move |w| Self::is_process_of(w, p) || self.detector.of_named(w, p))
	}

	/// "Not a game" on a found game: it leaves the found ones, and with them the watched ones, for the hidden ones,
	/// never noticed again until the player puts it back.
	pub fn dismiss(&mut self, key: &str, settings: &mut GamesSettings) -> bool {
		let watched = programs(settings).find(|w| w.id() == key);
		let Some(at) = settings.detected.iter().position(|d| d.key == key) else { return false };
		let found = settings.detected.remove(at);
		if let Some(watched) = watched {
			self.watcher.forget(&watched);
		}
		if !listed(&settings.dismissed, key) {
			settings.dismissed.insert(0, found);
		}
		true
	}

	/// A hidden game put back: found again, watched unless the player turned it off.
	pub fn restore(key: &str, settings: &mut GamesSettings) -> bool {
		let Some(at) = settings.dismissed.iter().position(|d| d.key == key) else { return false };
		let found = settings.dismissed.remove(at);
		if !listed(&settings.detected, key) {
			settings.detected.insert(0, found);
		}
		true
	}
}
