//! Elite Dangerous' Player Journal (docs/plan.md slice 48; docs/computer-games.md §6.2; Frontier's Journal Manual: the
//! journals are written to `Saved Games\Frontier Developments\Elite Dangerous`, one `Journal.<time>.<part>.log` a
//! session): the newest journal followed while the game runs and is watched (slice 56), each line read by
//! questpump-core's
//! `feeds::elite` for a mission completed, by its `event` alone. Windows only, where the game runs.

#[cfg(windows)]
use std::path::PathBuf;

use questpump_core::feeds::elite;

use super::Word;
use super::tail::Follow;

/// The journals' folder.
#[cfg(windows)]
pub(super) fn journals() -> Option<PathBuf> {
	use windows::Win32::UI::Shell::FOLDERID_SavedGames;
	super::win::known_folder(&FOLDERID_SavedGames).map(|d| d.join("Frontier Developments").join("Elite Dangerous"))
}

/// The newest journal.
#[cfg(windows)]
fn journal() -> Option<std::path::PathBuf> {
	let log = |name: &str| std::path::Path::new(name).extension().is_some_and(|e| e.eq_ignore_ascii_case("log"));
	super::tail::newest(&journals()?, |name| name.starts_with("Journal.") && log(name))
}

/// No journal outside Windows.
#[cfg(not(windows))]
fn journal() -> Option<std::path::PathBuf> {
	None
}

/// The journal while the game runs and is watched.
#[derive(Default)]
pub(super) struct Elite {
	follow: Follow,
}

impl Elite {
	/// A look: the words of the lines written since the last; while the game runs, its newest journal looked for
	/// again.
	pub(super) fn look(&mut self, running: bool) -> Vec<Word> {
		let (left, lines) = self.follow.lines(running, journal);
		let lines = [left, lines].concat();
		let mut words = Vec::new();
		if !lines.is_empty() {
			words.push(Word::Up);
		}
		words.extend(lines.iter().filter_map(|l| elite::word(l)).map(Word::Match));
		words
	}
}
