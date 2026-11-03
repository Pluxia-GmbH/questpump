//! PUBG: BATTLEGROUNDS' replays (2026-10-29; docs/computer-games.md §6.2): `%LOCALAPPDATA%\TslGame\Saved\Demos`, a
//! folder each match the game records. While the game runs and is watched, and once more at its close, each look lists
//! it: the newest folder's creation time, and whether `PUBG.replayinfo` is in that folder (the file's attributes alone),
//! handed to questpump-core's `feeds::pubg`. No file is opened, and no folder's or file's name (a folder's names the
//! match) is kept or logged. Windows only, where the game runs.

use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use questpump_core::feeds::pubg::{self, Matches};

use super::Word;

/// The game's folder of replays in the local application data, there once the game recorded one.
pub(super) fn demos() -> Option<PathBuf> {
	dirs::data_local_dir().map(|d| d.join("TslGame").join("Saved").join("Demos"))
}

/// The newest match folder's creation time (unix ms), from the listing, and whether its end file is in it.
fn listing(dir: &Path) -> (Option<i64>, bool) {
	let Ok(entries) = std::fs::read_dir(dir) else { return (None, false) };
	let newest = entries
		.flatten()
		.filter_map(|e| {
			let meta = e.metadata().ok().filter(std::fs::Metadata::is_dir)?;
			let at = meta.created().ok()?.duration_since(UNIX_EPOCH).ok()?;
			Some((i64::try_from(at.as_millis()).ok()?, e.path()))
		})
		.max_by_key(|(at, _)| *at);
	let Some((created, folder)) = newest else { return (None, false) };
	(Some(created), folder.join(pubg::INFO).is_file())
}

/// The replays while the game runs and is watched.
pub(super) struct Pubg {
	dir: Option<PathBuf>,
	matches: Matches,
}

impl Pubg {
	pub(super) fn new() -> Self {
		Self { dir: demos(), matches: Matches::default() }
	}

	/// A look: heard while the folder is there, and a match's start or end from what its listings show.
	pub(super) fn look(&mut self) -> Vec<Word> {
		let Some(dir) = self.dir.as_deref().filter(|d| d.is_dir()) else { return Vec::new() };
		let (folder, done) = listing(dir);
		let mut words = vec![Word::Up];
		words.extend(self.matches.push(folder, done).into_iter().map(Word::Match));
		words
	}
}
