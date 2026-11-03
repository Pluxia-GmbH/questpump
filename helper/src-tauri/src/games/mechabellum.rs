//! Mechabellum's `Player.log` (2026-10-28; docs/computer-games.md §6.2): Unity writes it for every player with no
//! setting asked, into `AppData\LocalLow\GameRiver\Mechabellum` on Windows (read on Rapha's PC) and, by Unity's rule
//! for a Mac build, `~/Library/Logs/GameRiver/Mechabellum` on macOS (unverified until the game runs on a Mac). The game
//! moves the log to `Player-prev.log` at each start and writes it anew, which the tail reads from its beginning (another
//! file, or a shorter one). It is followed while the game runs and is watched, each line read by questpump-core's
//! `feeds::mechabellum` for a match's start or end alone; every other line (other players' names, ids and records among
//! them) is passed over, and no line is kept.

use std::path::PathBuf;

use questpump_core::feeds::mechabellum;

use super::Word;
use super::tail::Tail;

/// The folder Mechabellum writes its log into; none where the system names no such folder.
pub(super) fn log_dir() -> Option<PathBuf> {
	#[cfg(windows)]
	{
		super::logs::local_low().map(|d| d.join("GameRiver").join("Mechabellum"))
	}
	#[cfg(not(windows))]
	{
		dirs::home_dir().map(|h| h.join("Library").join("Logs").join("GameRiver").join("Mechabellum"))
	}
}

/// The log while the game runs and is watched.
pub(super) struct Mechabellum {
	tail: Option<Tail>,
}

impl Mechabellum {
	pub(super) fn new() -> Self {
		Self { tail: log_dir().map(|d| Tail::at_end(d.join("Player.log"))) }
	}

	/// A look: the words of the lines written since the last; any line read tells that the game is heard.
	pub(super) fn look(&mut self) -> Vec<Word> {
		let Some(tail) = self.tail.as_mut() else { return Vec::new() };
		let lines = tail.lines();
		let mut words = Vec::new();
		if !lines.is_empty() {
			words.push(Word::Up);
		}
		words.extend(lines.iter().filter_map(|l| mechabellum::line(l)).map(Word::Match));
		words
	}
}
