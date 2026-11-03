//! The games added later on 2026-10-29 heard by a file they save as something ends (docs/computer-games.md §6.2), each
//! by the newest such file's modified time alone: Halo: The Master Chief Collection's multiplayer carnage reports
//! (`LocalLow\MCC\Temporary\mpcarnagereport*.xml`, a game's end; the campaign's are left out), Brawlhalla's replays
//! (`%USERPROFILE%\BrawlhallaReplays\*.replay`, a match's end), rFactor 2's and Le Mans Ultimate's results
//! (`<install>\UserData\Log\Results\*.xml`, a session's end), osu!'s replays (`<install>\Data\r\*.osr`, a play passed),
//! Stardew Valley's saves (`Saves\<farm>\…`, written as the player sleeps: a day's end; on Windows and macOS) and
//! Warcraft III's replays (`BattleNet\<account>\Replays\…\*.w3g`, `LastReplay.w3g` rewritten and a new one in
//! `Autosaved` at a game's end; on Windows and macOS). While the game runs and is watched, and once more at its close,
//! each look lists its folders, at most `FOLDERS` of them, and hands the newest wanted file's modified
//! time to questpump-core's `feeds::Newer`, whose first look sets where the folder stands and whose newer one is
//! something completed. The times come from the listings (std's `DirEntry::metadata`, which on Windows reads the
//! listing's entry and opens no file): no file is opened, and no file's or folder's name (a farm's, an account's, a
//! replay's) is kept or logged.

use std::path::{Path, PathBuf};

use questpump_core::feeds::Newer;
use questpump_core::games::Watched;

use super::{Word, logs, steam};

/// The most folders a look lists.
const FOLDERS: usize = 64;

/// What a game's look lists: its folders, how deep under them, and the files it counts by their names.
struct Spec {
	roots: Vec<PathBuf>,
	depth: usize,
	wanted: fn(&str) -> bool,
}

/// Whether a file's name ends with `.<extension>`, in any case.
fn has_extension(name: &str, extension: &str) -> bool {
	Path::new(name).extension().is_some_and(|e| e.eq_ignore_ascii_case(extension))
}

/// osu!'s install where its own installer puts it (`%LOCALAPPDATA%\osu!`).
pub(super) fn osu() -> Option<PathBuf> {
	dirs::data_local_dir().map(|d| d.join("osu!"))
}

/// Stardew Valley's saves: `%APPDATA%\StardewValley\Saves` on Windows, `~/.config/StardewValley/Saves` on macOS.
pub(super) fn stardew_saves() -> Option<PathBuf> {
	#[cfg(windows)]
	let base = dirs::config_dir();
	#[cfg(not(windows))]
	let base = dirs::home_dir().map(|h| h.join(".config"));
	base.map(|d| d.join("StardewValley").join("Saves"))
}

/// Warcraft III's own folder of the player's files: `Documents\Warcraft III` on Windows (unverified, by analogy with the
/// Mac's), `~/Library/Application Support/Warcraft III` on macOS (Blizzard's forum, 2024-07-15).
pub(super) fn warcraft_files() -> Option<PathBuf> {
	#[cfg(windows)]
	let base = dirs::document_dir();
	#[cfg(not(windows))]
	let base = dirs::data_dir();
	base.map(|d| d.join("Warcraft III"))
}

/// Each Battle.net account's `Replays` folder under Warcraft III's own folder.
fn warcraft_replays() -> Vec<PathBuf> {
	let Some(Ok(accounts)) = warcraft_files().map(|d| std::fs::read_dir(d.join("BattleNet"))) else {
		return Vec::new();
	};
	accounts.flatten().map(|e| e.path().join("Replays")).filter(|d| d.is_dir()).take(FOLDERS).collect()
}

/// Halo: The Master Chief Collection's folder of reports in the user's `LocalLow`.
pub(super) fn halo_reports() -> Option<PathBuf> {
	logs::local_low().map(|d| d.join("MCC").join("Temporary"))
}

/// Brawlhalla's folder of replays in the user's home.
pub(super) fn brawlhalla_replays() -> Option<PathBuf> {
	dirs::home_dir().map(|h| h.join("BrawlhallaReplays"))
}

/// What a game heard by its saved files lists; none for another game.
fn spec(watched: &Watched) -> Option<Spec> {
	let results = || steam::game_dir(watched, false).map(|d| d.join("UserData").join("Log").join("Results"));
	let (roots, depth, wanted): (Vec<PathBuf>, usize, fn(&str) -> bool) = match watched {
		Watched::Halo => (halo_reports().into_iter().collect(), 0, |name| {
			let name = name.to_ascii_lowercase();
			name.starts_with("mpcarnagereport") && has_extension(&name, "xml")
		}),
		Watched::Brawlhalla => (brawlhalla_replays().into_iter().collect(), 0, |name| has_extension(name, "replay")),
		Watched::RFactor | Watched::LeMans => (results().into_iter().collect(), 0, |name| has_extension(name, "xml")),
		Watched::Osu => {
			(osu().map(|d| d.join("Data").join("r")).into_iter().collect(), 0, |name| has_extension(name, "osr"))
		}
		// a farm's files, never Steam Cloud's own record beside the farms
		Watched::Stardew => (stardew_saves().into_iter().collect(), 1, |name| !has_extension(name, "vdf")),
		// each account's `Replays`: its `LastReplay.w3g` and `Autosaved\Multiplayer\…\<game>.w3g`
		Watched::Warcraft => (warcraft_replays(), 3, |name| has_extension(name, "w3g")),
		_ => return None,
	};
	Some(Spec { roots, depth, wanted })
}

/// The newest wanted file's modified time (unix ms) in `dir` and the folders under it to `depth`, each listing counted
/// against `left`.
fn newest_in(dir: &Path, depth: usize, wanted: fn(&str) -> bool, left: &mut usize) -> Option<i64> {
	if *left == 0 {
		return None;
	}
	*left -= 1;
	let Ok(entries) = std::fs::read_dir(dir) else { return None };
	let mut newest: Option<i64> = None;
	let mut folders = Vec::new();
	for entry in entries.flatten() {
		let Ok(meta) = entry.metadata() else { continue };
		if meta.is_dir() {
			folders.push(entry.path());
			continue;
		}
		if !entry.file_name().to_str().is_some_and(wanted) {
			continue;
		}
		let ms = meta
			.modified()
			.ok()
			.and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
			.and_then(|d| i64::try_from(d.as_millis()).ok());
		newest = newest.max(ms);
	}
	if depth > 0 {
		for folder in folders {
			newest = newest.max(newest_in(&folder, depth - 1, wanted, left));
		}
	}
	newest
}

/// A game's saved files while it runs and is watched.
pub(super) struct Saves {
	spec: Option<Spec>,
	newer: Newer,
}

impl Saves {
	pub(super) fn of(watched: &Watched) -> Self {
		Self { spec: spec(watched), newer: Newer::default() }
	}

	/// A look: heard while one of its folders is there, and something completed once a newer file came.
	pub(super) fn look(&mut self) -> Vec<Word> {
		let Some(spec) = self.spec.as_ref() else { return Vec::new() };
		let roots: Vec<&PathBuf> = spec.roots.iter().filter(|d| d.is_dir()).collect();
		if roots.is_empty() {
			return Vec::new();
		}
		let mut left = FOLDERS;
		let newest = roots.iter().filter_map(|d| newest_in(d, spec.depth, spec.wanted, &mut left)).max();
		let mut words = vec![Word::Up];
		words.extend(self.newer.push(newest).map(Word::Match));
		words
	}
}
