//! Slay the Spire 2's run history (2026-10-28, docs/computer-games.md §6.2; ststracker.app's About page names the
//! folder its uploader takes the `.run` files from, `%APPDATA%\SlayTheSpire2\steam\<SteamID>\profile1\saves\history`):
//! while the game runs and is watched, and once more at its close, each look lists the history folders of every
//! account and profile under `%APPDATA%\SlayTheSpire2` (`<store>\<account>\<profile>\saves\history`, at most
//! `FOLDERS` folders looked into) and hands the newest `.run` file's modified time to questpump-core's `feeds::Newer`,
//! whose first look sets where the history stands and whose newer one is a run completed. The times come from the
//! folders' own listings (std's `DirEntry::metadata`, which on Windows reads the listing's entry and opens no file):
//! no run file is opened, and no file's or folder's name (the Steam id) is kept or logged. Windows only, where its
//! folder is known.

#[cfg(windows)]
use std::path::{Path, PathBuf};

use questpump_core::feeds::Newer;
#[cfg(windows)]
use questpump_core::feeds::spire;

use super::Word;

/// The most folders a look lists: the stores, the accounts, the profiles and their history folders together.
#[cfg(windows)]
const FOLDERS: usize = 64;

/// The game's own folder in the roaming application data (`%APPDATA%\SlayTheSpire2`).
#[cfg(windows)]
pub(super) fn root() -> Option<PathBuf> {
	dirs::config_dir().map(|d| d.join("SlayTheSpire2"))
}

/// No folder outside Windows.
#[cfg(not(windows))]
pub(super) fn root() -> Option<std::path::PathBuf> {
	None
}

/// The folders listed in `dir`, each listing counted against `left`.
#[cfg(windows)]
fn folders(dir: &Path, left: &mut usize) -> Vec<PathBuf> {
	if *left == 0 {
		return Vec::new();
	}
	*left -= 1;
	let Ok(entries) = std::fs::read_dir(dir) else { return Vec::new() };
	entries.flatten().filter(|e| e.file_type().is_ok_and(|t| t.is_dir())).map(|e| e.path()).collect()
}

/// Whether a history folder is there, and the newest run file's modified time in the history folders (unix ms).
#[cfg(windows)]
fn newest() -> (bool, Option<i64>) {
	let Some(root) = root() else { return (false, None) };
	let mut left = FOLDERS;
	let mut histories = Vec::new();
	for store in folders(&root, &mut left) {
		for account in folders(&store, &mut left) {
			for profile in folders(&account, &mut left) {
				let history = profile.join("saves").join("history");
				if history.is_dir() {
					histories.push(history);
				}
			}
		}
	}
	let mut newest: Option<i64> = None;
	for history in &histories {
		if left == 0 {
			break;
		}
		left -= 1;
		let Ok(entries) = std::fs::read_dir(history) else { continue };
		for entry in entries.flatten() {
			let run =
				Path::new(&entry.file_name()).extension().is_some_and(|e| e.eq_ignore_ascii_case(spire::EXTENSION));
			let modified = run.then(|| entry.metadata().ok()?.modified().ok()).flatten();
			let ms = modified
				.and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
				.and_then(|d| i64::try_from(d.as_millis()).ok());
			newest = newest.max(ms);
		}
	}
	(!histories.is_empty(), newest)
}

/// Nothing outside Windows.
#[cfg(not(windows))]
fn newest() -> (bool, Option<i64>) {
	(false, None)
}

/// The history while the game runs and is watched.
#[derive(Default)]
pub(super) struct Spire {
	history: Newer,
}

impl Spire {
	/// A look: heard while a history folder is there, and a run completed once a newer run file came.
	pub(super) fn look(&mut self) -> Vec<Word> {
		let (found, newest) = newest();
		let mut words = Vec::new();
		if found {
			words.push(Word::Up);
		}
		words.extend(self.history.push(newest).map(Word::Match));
		words
	}
}
