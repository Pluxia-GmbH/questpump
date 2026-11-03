//! Finding a Steam game's folder (docs/plan.md slice 35; Valve's Game State Integration page and steamlocate-rs 2.1.1's
//! readers, read 2026-10-08): Steam's own folder (Windows: `HKCU\Software\Valve\Steam` `SteamPath`, then the machine's
//! `InstallPath` under `WOW6432Node` and without it; macOS: `~/Library/Application Support/Steam`), every library its
//! `steamapps/libraryfolders.vdf` lists, and the library whose `appmanifest_<appid>.acf` names the game's folder under
//! `steamapps/common`. Read when a game is set up and while it is not found, never on every look. Since slice 48 the
//! detector (`detect.rs`) reads them too: every manifest's game (its app, folder and name, `apps`), the libraries'
//! `steamapps/common` folders by their paths alone (macOS) and Steam's own `logs` folder (Windows). On macOS a library
//! on a volume under `/Volumes` (a removable or network volume, whose read asks the player's consent) is read only for
//! a game the player asked for (`volumes`); the startup volume's libraries alone tell what is installed. Since slice 56
//! the detector reads the type Steam gives its apps from Steam's own `appcache/appinfo.vdf` (`app_types`). On Windows
//! the icons (`icons.rs`) read a Steam game's own icon from Steam's local copies of it (`app_holding`, `icon_files`):
//! the `.ico` Steam's shortcuts to the game name, else the small icon in Steam's library cache, since a game's program
//! may carry its engine's icon (Mechabellum's Unity cube, Rapha's PC, 2026-10-28). Only small text files are read for
//! them, never Steam's binary records.

use std::collections::HashMap;
#[cfg(windows)]
use std::io::Read as _;
use std::path::{Path, PathBuf};

use questpump_core::games::Watched;
use questpump_core::gsi::Gsi;
use questpump_core::{appinfo, vdf};

/// Steam's own folder, when Steam is installed.
pub(super) fn root() -> Option<PathBuf> {
	#[cfg(windows)]
	{
		steam_path().map(PathBuf::from).filter(|p| p.is_dir())
	}
	#[cfg(not(windows))]
	{
		dirs::data_dir().map(|d| d.join("Steam")).filter(|p| p.is_dir())
	}
}

/// Every library once: Steam's own folder first, then those `libraryfolders.vdf` lists.
fn libraries(root: &Path) -> Vec<PathBuf> {
	let listed = std::fs::read_to_string(root.join("steamapps").join("libraryfolders.vdf"))
		.map(|text| vdf::library_paths(&text))
		.unwrap_or_default();
	let mut out = vec![root.to_path_buf()];
	for library in listed {
		if !out.contains(&library) {
			out.push(library);
		}
	}
	out
}

/// Whether a path is in a library whose manifests the detector reads (`apps`): on macOS one on the startup volume.
#[cfg(target_os = "macos")]
pub(super) fn read_here(path: &Path) -> bool {
	super::readable(path, false)
}

/// The libraries that may be read (`games::readable`: on macOS none in a folder that asks the player's consent, and one
/// under `/Volumes` only with `volumes`, for a game the player asked for).
fn readable_libraries(volumes: bool) -> Vec<PathBuf> {
	let libraries = root().map(|root| libraries(&root)).unwrap_or_default();
	libraries.into_iter().filter(|l| super::readable(l, volumes)).collect()
}

/// The manifests of a Steam game in each library that holds one and may be read, with the library's `steamapps`
/// folder.
fn manifests(app: u32, volumes: bool) -> impl Iterator<Item = (String, PathBuf)> {
	readable_libraries(volumes).into_iter().filter_map(move |library| {
		let steamapps = library.join("steamapps");
		let acf = std::fs::read_to_string(steamapps.join(format!("appmanifest_{app}.acf"))).ok()?;
		Some((acf, steamapps))
	})
}

/// A Steam game's install folder, when a library that may be read holds it (`volumes`: also those under `/Volumes`
/// on macOS, for a game the player asked for).
pub fn app_dir(app: u32, volumes: bool) -> Option<PathBuf> {
	manifests(app, volumes).find_map(|(acf, steamapps)| {
		let dir = steamapps.join("common").join(vdf::install_dir(&acf)?);
		dir.is_dir().then_some(dir)
	})
}

/// A named game's install folder in a Steam library: its first app a library holds (`Watched::steam_apps`: Path of
/// Exile's own before Path of Exile 2's); `volumes` as `app_dir`'s.
pub fn game_dir(watched: &Watched, volumes: bool) -> Option<PathBuf> {
	watched.steam_apps().iter().find_map(|&app| app_dir(app, volumes))
}

/// A game a Steam manifest names: its app, its install folder and its name as the manifest writes it.
#[derive(Debug, Clone)]
pub(super) struct App {
	pub id: u32,
	pub dir: PathBuf,
	pub name: Option<String>,
}

/// The most of a manifest or a shortcut read for a game's icon (each is a few hundred bytes).
#[cfg(windows)]
const SMALL_BYTES: u64 = 64 * 1024;

/// A small text file's start, at most `SMALL_BYTES`, its bytes that are no UTF-8 replaced.
#[cfg(windows)]
fn small_text(path: &Path) -> Option<String> {
	let mut bytes = Vec::new();
	std::fs::File::open(path).ok()?.take(SMALL_BYTES).read_to_end(&mut bytes).ok()?;
	Some(String::from_utf8_lossy(&bytes).into_owned())
}

/// The Steam app whose install folder holds `path` (`<library>\steamapps\common\<installdir>\...`, the path made whole
/// first, so forward slashes read too), as the library's `appmanifest_<app>.acf` names that folder; none for a path
/// in no Steam library.
#[cfg(windows)]
pub(super) fn app_holding(path: &Path) -> Option<u32> {
	use std::path::Component;
	let path = std::path::absolute(path).ok()?;
	let parts: Vec<Component<'_>> = path.components().collect();
	let named = |part: &Component<'_>, name: &str| {
		let Component::Normal(part) = part else { return false };
		part.to_str().is_some_and(|part| part.eq_ignore_ascii_case(name))
	};
	let at = parts.windows(3).position(|w| named(&w[0], "steamapps") && named(&w[1], "common"))?;
	let Component::Normal(dir) = parts.get(at + 2)? else { return None };
	let dir = dir.to_str()?;
	let steamapps: PathBuf = parts[..=at].iter().collect();
	std::fs::read_dir(&steamapps).ok()?.flatten().find_map(|entry| {
		let file = entry.file_name();
		let app = file.to_str()?.strip_prefix("appmanifest_")?.strip_suffix(".acf")?.parse().ok()?;
		let named_dir = vdf::install_dir(&small_text(&entry.path())?)?;
		named_dir.eq_ignore_ascii_case(dir).then_some(app)
	})
}

/// The files on this computer that hold a Steam game's own icon, best first: the `.ico` each Steam shortcut to the
/// game names (an internet shortcut, `URL=steam://rungameid/<app>`, in the player's Start menu's `Steam` folder, then
/// on the Desktop; only an `IconFile` inside Steam's `steam\games` folder, `..` and links resolved), then the icon
/// Steam keeps in its library cache (`appcache\librarycache\<app>\<40 hex>.jpg`, 32 px; the folders beside it hold
/// the store's art, never read); none without Steam.
#[cfg(windows)]
pub(super) fn icon_files(app: u32) -> Vec<PathBuf> {
	use windows::Win32::UI::Shell::{FOLDERID_Desktop, FOLDERID_Programs};

	use super::win::known_folder;
	let Some(root) = root() else { return Vec::new() };
	let mut files = Vec::new();
	if let Ok(games) = std::fs::canonicalize(root.join("steam").join("games")) {
		let url = format!("steam://rungameid/{app}");
		let folders = [known_folder(&FOLDERID_Programs).map(|p| p.join("Steam")), known_folder(&FOLDERID_Desktop)];
		for folder in folders.into_iter().flatten() {
			let Ok(entries) = std::fs::read_dir(folder) else { continue };
			let mut shortcuts: Vec<PathBuf> = entries
				.flatten()
				.map(|entry| entry.path())
				.filter(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("url")))
				.collect();
			shortcuts.sort();
			for shortcut in shortcuts {
				let Some(icon) = small_text(&shortcut).and_then(|text| shortcut_icon(&text, &url)) else { continue };
				let Ok(icon) = std::fs::canonicalize(icon) else { continue };
				let ico = icon.extension().is_some_and(|e| e.eq_ignore_ascii_case("ico"));
				if ico && icon.parent() == Some(games.as_path()) && icon.is_file() && !files.contains(&icon) {
					files.push(icon);
				}
			}
		}
	}
	let cache = root.join("appcache").join("librarycache").join(app.to_string());
	if let Ok(entries) = std::fs::read_dir(cache) {
		let mut cached: Vec<PathBuf> = entries
			.flatten()
			.filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_file()))
			.filter(|entry| {
				let name = entry.file_name();
				name.to_str()
					.and_then(|n| n.strip_suffix(".jpg"))
					.is_some_and(|stem| stem.len() == 40 && stem.bytes().all(|b| b.is_ascii_hexdigit()))
			})
			.map(|entry| entry.path())
			.collect();
		cached.sort();
		files.extend(cached.into_iter().take(1));
	}
	files
}

/// The `IconFile` an internet shortcut's `[InternetShortcut]` section names, when its `URL` is `url`.
#[cfg(windows)]
fn shortcut_icon(text: &str, url: &str) -> Option<PathBuf> {
	let (mut section, mut target, mut icon) = (false, None, None);
	for line in text.lines().map(str::trim) {
		if line.starts_with('[') {
			section = line.eq_ignore_ascii_case("[InternetShortcut]");
		} else if let Some((key, value)) = line.split_once('=').filter(|_| section) {
			let (key, value) = (key.trim(), value.trim());
			if key.eq_ignore_ascii_case("URL") {
				target = Some(value);
			} else if key.eq_ignore_ascii_case("IconFile") && !value.is_empty() {
				icon = Some(value);
			}
		}
	}
	icon.filter(|_| target.is_some_and(|t| t.eq_ignore_ascii_case(url))).map(PathBuf::from)
}

/// Every game the manifests of the libraries on the startup volume name (every library on Windows), read for the
/// detector at its refresh, off the loop.
pub(super) fn apps() -> Vec<App> {
	let mut apps = Vec::new();
	for library in readable_libraries(false) {
		let steamapps = library.join("steamapps");
		let Ok(entries) = std::fs::read_dir(&steamapps) else { continue };
		for entry in entries.flatten() {
			let file = entry.file_name();
			let id = file.to_str().and_then(|f| f.strip_prefix("appmanifest_")?.strip_suffix(".acf")?.parse().ok());
			let Some(id) = id else { continue };
			let Ok(acf) = std::fs::read_to_string(entry.path()) else { continue };
			let Some(dir) = vdf::install_dir(&acf).map(|d| steamapps.join("common").join(d)) else { continue };
			apps.push(App { id, dir, name: vdf::app_name(&acf) });
		}
	}
	apps
}

/// Every library's `steamapps/common`, where Steam puts its games; none without Steam.
#[cfg(target_os = "macos")]
pub(super) fn commons() -> Vec<PathBuf> {
	root()
		.map(|root| libraries(&root).into_iter().map(|l| l.join("steamapps").join("common")).collect())
		.unwrap_or_default()
}

/// The folder a game's cfg goes into, under its install: CS2's `game/csgo/cfg` (Valve's page), Dota 2's
/// `game/dota/cfg/gamestate_integration` (the community libraries' folder, which the writer creates); none when the
/// install lacks the `cfg` folder above it (an install Steam has not finished).
pub fn cfg_dir(game: Gsi, app: &Path) -> Option<PathBuf> {
	match game {
		Gsi::CounterStrike => Some(app.join("game").join("csgo").join("cfg")).filter(|d| d.is_dir()),
		Gsi::Dota => {
			let cfg = app.join("game").join("dota").join("cfg");
			cfg.is_dir().then(|| cfg.join("gamestate_integration"))
		}
	}
}

/// Steam's folder as the player's Steam wrote it, else as its installer wrote it for the machine.
#[cfg(windows)]
fn steam_path() -> Option<String> {
	use windows::Win32::System::Registry::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
	use windows::core::w;

	use super::win::text;
	text(HKEY_CURRENT_USER, w!(r"Software\Valve\Steam"), w!("SteamPath"))
		.or_else(|| text(HKEY_LOCAL_MACHINE, w!(r"SOFTWARE\WOW6432Node\Valve\Steam"), w!("InstallPath")))
		.or_else(|| text(HKEY_LOCAL_MACHINE, w!(r"SOFTWARE\Valve\Steam"), w!("InstallPath")))
}

/// The type Steam's own record gives each app asked (`appcache/appinfo.vdf`, questpump-core's `appinfo`), its file
/// read whole on the records' thread: every app asked has its answer, none where the record gives none (no Steam, the
/// file missing or of a version not read, the app not in it).
pub(super) fn app_types(asked: &[u32]) -> HashMap<u32, Option<String>> {
	let bytes = root().and_then(|root| std::fs::read(root.join("appcache").join("appinfo.vdf")).ok());
	let mut types = bytes.map(|bytes| appinfo::types(&bytes, asked)).unwrap_or_default();
	asked.iter().map(|&app| (app, types.remove(&app))).collect()
}
