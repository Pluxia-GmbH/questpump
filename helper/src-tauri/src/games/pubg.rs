//! PUBG: BATTLEGROUNDS' replays (2026-10-29; docs/computer-games.md §6.2): `%LOCALAPPDATA%\TslGame\Saved\Demos`, a
//! folder each match the game records, written as the match ends (github.com/Segergren/Segra's `PubgIntegration.cs`
//! waits at most 10 s for a new folder's `PUBG.replayinfo`; whether replays are recorded by default is unverified).
//! While the game runs and is watched, and once more at its close, each look lists it for the newest folder's creation
//! time, handed to questpump-core's `Newer`: a folder newer than any at the first look is a match completed, sized from
//! the game's start or the last match that offered (`games::Finish::began_at`), as a game's saved files are
//! (`saves.rs`). No file is opened, and no folder's or file's name (a folder's names the match) is kept or logged.
//! Windows only, where the game runs.

use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use questpump_core::feeds::Newer;

use super::Word;

/// The game's folder of replays in the local application data, there once the game recorded one.
pub(super) fn demos() -> Option<PathBuf> {
	dirs::data_local_dir().map(|d| d.join("TslGame").join("Saved").join("Demos"))
}

/// The newest match folder's creation time (unix ms), from the listing.
fn newest(dir: &Path) -> Option<i64> {
	std::fs::read_dir(dir)
		.ok()?
		.flatten()
		.filter_map(|e| {
			let meta = e.metadata().ok().filter(std::fs::Metadata::is_dir)?;
			let at = meta.created().ok()?.duration_since(UNIX_EPOCH).ok()?;
			i64::try_from(at.as_millis()).ok()
		})
		.max()
}

/// The replays while the game runs and is watched.
pub(super) struct Pubg {
	dir: Option<PathBuf>,
	newer: Newer,
}

impl Pubg {
	pub(super) fn new() -> Self {
		Self { dir: demos(), newer: Newer::default() }
	}

	/// A look: heard while the folder is there, and a match completed once a newer folder came.
	pub(super) fn look(&mut self) -> Vec<Word> {
		let Some(dir) = self.dir.as_deref().filter(|d| d.is_dir()) else { return Vec::new() };
		let mut words = vec![Word::Up];
		words.extend(self.newer.push(newest(dir)).map(Word::Match));
		words
	}
}
