//! Rainbow Six Siege's match replays (2026-10-29; docs/computer-games.md §6.2): its install in a Steam library, else
//! the Ubisoft Connect install that holds `RainbowSix.exe` (`detect::ubisoft_installs`, the launcher's own records).
//! While the game runs and is watched, and once more at its close, each look lists `MatchReplay`: the newest
//! `Match-…` folder's creation time from the listing, and its newest `.rec` file's modified time from the file's own
//! attributes (`fs::metadata`: a listing's entry can keep the time a file had when the game opened it until the game
//! closes it, which would read as a quiet round), handed to questpump-core's `feeds::siege` with the look's time. No
//! file's contents are read, and no folder's or file's name (it names the match's date) is kept or logged. Windows
//! only, where the game runs.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use questpump_core::feeds::siege::{self, Matches};
use questpump_core::games::Watched;

use super::{Word, steam};

/// The game's install: its Steam library's, else Ubisoft Connect's that holds its program.
pub(super) fn install() -> Option<PathBuf> {
	steam::game_dir(&Watched::Siege, false).or_else(ubisoft)
}

/// The Ubisoft Connect install that holds the game's program.
#[cfg(windows)]
fn ubisoft() -> Option<PathBuf> {
	let image = Watched::Siege.windows_images().first()?;
	super::detect::ubisoft_installs().into_iter().find(|d| d.join(image).is_file())
}

/// No Ubisoft Connect outside Windows.
#[cfg(not(windows))]
fn ubisoft() -> Option<PathBuf> {
	None
}

/// A time as unix ms.
fn ms(at: SystemTime) -> Option<i64> {
	at.duration_since(UNIX_EPOCH).ok().and_then(|d| i64::try_from(d.as_millis()).ok())
}

/// The newest match folder's creation time (unix ms), from the listing, and its newest round file's modified time, from
/// the file's attributes.
fn listing(dir: &Path) -> (Option<i64>, Option<i64>) {
	let Ok(entries) = std::fs::read_dir(dir) else { return (None, None) };
	let newest = entries
		.flatten()
		.filter(|e| e.file_name().to_str().is_some_and(|n| n.starts_with(siege::MATCH)))
		.filter_map(|e| {
			let meta = e.metadata().ok().filter(std::fs::Metadata::is_dir)?;
			Some((ms(meta.created().ok()?)?, e.path()))
		})
		.max_by_key(|(at, _)| *at);
	let Some((created, folder)) = newest else { return (None, None) };
	let round = std::fs::read_dir(folder).ok().and_then(|entries| {
		entries
			.flatten()
			.filter(|e| Path::new(&e.file_name()).extension().is_some_and(|x| x.eq_ignore_ascii_case(siege::EXTENSION)))
			.filter_map(|e| ms(std::fs::metadata(e.path()).ok()?.modified().ok()?))
			.max()
	});
	(Some(created), round)
}

/// The match replays while the game runs and is watched.
pub(super) struct Siege {
	dir: Option<PathBuf>,
	matches: Matches,
}

impl Siege {
	pub(super) fn new() -> Self {
		Self { dir: install().map(|d| d.join("MatchReplay")), matches: Matches::default() }
	}

	/// A look: heard while the folder is there, and a match's start or end from what its listings show.
	pub(super) fn look(&mut self) -> Vec<Word> {
		let Some(dir) = self.dir.as_deref().filter(|d| d.is_dir()) else { return Vec::new() };
		let (folder, round) = listing(dir);
		let now = ms(SystemTime::now()).unwrap_or_default();
		let mut words = vec![Word::Up];
		words.extend(self.matches.push(now, folder, round).into_iter().map(Word::Match));
		words
	}
}
