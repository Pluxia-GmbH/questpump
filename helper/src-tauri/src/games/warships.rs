//! World of Warships' `replays` folder (2026-10-29; docs/computer-games.md §6.2): its install in a Steam library, else
//! the Wargaming Game Center's default folders (no record of the Game Center's own was found that names its installs
//! without reading its files, so a game installed elsewhere through it is heard at its close alone). While the game runs
//! and is watched, and once more at its close, each look lists the folder: whether `tempArenaInfo.json` is there and the
//! newest finished `*.wowsreplay` file's modified time, both from the listing (std's `DirEntry`, which on Windows reads
//! the listing's entry and opens no file), handed to questpump-core's `feeds::warships`. No file is opened, and no
//! file's name (a replay's names the date, the ship and the map) is kept or logged. Windows only, where the game runs.

use std::path::{Path, PathBuf};

use questpump_core::feeds::warships::{self, Battles};
use questpump_core::games::Watched;

use super::{Word, steam};

/// The Wargaming Game Center's default folders for the game: the one it uses now and the regions' it used before.
const DEFAULTS: [&str; 4] = [
	r"C:\Games\World_of_Warships",      // copy-lint: not shown (a path)
	r"C:\Games\World_of_Warships_EU",   // copy-lint: not shown (a path)
	r"C:\Games\World_of_Warships_NA",   // copy-lint: not shown (a path)
	r"C:\Games\World_of_Warships_ASIA", // copy-lint: not shown (a path)
];

/// The game's install: its Steam library's, else the first default folder there.
pub(super) fn install() -> Option<PathBuf> {
	steam::game_dir(&Watched::Warships, false).or_else(|| DEFAULTS.into_iter().map(PathBuf::from).find(|d| d.is_dir()))
}

/// The folder as one listing shows it: whether the battle file is there, and the newest finished replay's modified
/// time (unix ms).
fn listing(dir: &Path) -> (bool, Option<i64>) {
	let Ok(entries) = std::fs::read_dir(dir) else { return (false, None) };
	let (mut arena, mut newest) = (false, None);
	for entry in entries.flatten() {
		let name = entry.file_name();
		let Some(name) = name.to_str() else { continue };
		if name.eq_ignore_ascii_case(warships::ARENA) {
			arena = true;
			continue;
		}
		let replay = Path::new(name).extension().is_some_and(|e| e.eq_ignore_ascii_case(warships::EXTENSION));
		if !replay || name.eq_ignore_ascii_case(warships::RUNNING) {
			continue;
		}
		let modified = entry.metadata().ok().and_then(|m| m.modified().ok());
		let ms = modified
			.and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
			.and_then(|d| i64::try_from(d.as_millis()).ok());
		newest = newest.max(ms);
	}
	(arena, newest)
}

/// The replays folder while the game runs and is watched.
pub(super) struct Warships {
	dir: Option<PathBuf>,
	battles: Battles,
}

impl Warships {
	pub(super) fn new() -> Self {
		Self { dir: install().map(|d| d.join("replays")), battles: Battles::default() }
	}

	/// A look: heard while the folder is there, and a battle's start or end from what its listing shows.
	pub(super) fn look(&mut self) -> Vec<Word> {
		let Some(dir) = self.dir.as_deref().filter(|d| d.is_dir()) else { return Vec::new() };
		let (arena, replay) = listing(dir);
		let mut words = vec![Word::Up];
		words.extend(self.battles.push(arena, replay).into_iter().map(Word::Match));
		words
	}
}
