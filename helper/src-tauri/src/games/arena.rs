//! MTG Arena's `Player.log` (docs/plan.md slice 48; docs/computer-games.md §6.2): where the game writes it (Windows:
//! `AppData\LocalLow\Wizards Of The Coast\MTGA`, as Wizards and the trackers name it; macOS: Unity's and the trackers'
//! `~/Library/Logs/Wizards Of The Coast/MTGA`, unchecked against Wizards' own page), followed as it grows while the
//! game runs and is watched (slice 56), each line read by questpump-core's `feeds::arena` for whether a match plays or
//! completed and whether the log began; every other line is passed over, and no line is kept. The game writes the log
//! anew at each start, its header line near the top, which is looked for once in each new log (a log that ran before
//! the helper's start too), so the game is heard before its first match.

use std::fs::File;
use std::io::Read as _;
use std::path::PathBuf;

use questpump_core::feeds::arena::{self, Line};
use questpump_core::games::Fed;

use super::Word;
use super::tail::Tail;

/// How much of a new log's top is read for its header line.
const HEAD_BYTES: u64 = 64 * 1024;

/// The folder MTG Arena writes its log into; none where the system names no such folder.
pub(super) fn log_dir() -> Option<PathBuf> {
	#[cfg(windows)]
	{
		use windows::Win32::UI::Shell::FOLDERID_LocalAppDataLow;
		super::win::known_folder(&FOLDERID_LocalAppDataLow).map(|d| d.join("Wizards Of The Coast").join("MTGA"))
	}
	#[cfg(not(windows))]
	{
		dirs::home_dir().map(|h| h.join("Library").join("Logs").join("Wizards Of The Coast").join("MTGA"))
	}
}

/// What a line says, as the loop takes it.
fn word(line: Line) -> Vec<Word> {
	match line {
		Line::Began => vec![Word::Up],
		Line::Playing => vec![Word::Up, Word::Match(Fed::Playing)],
		Line::Completed => vec![Word::Up, Word::Match(Fed::Ended)],
	}
}

/// The log while the game runs and is watched.
pub(super) struct Arena {
	tail: Option<Tail>,
}

impl Arena {
	pub(super) fn new() -> Self {
		Self { tail: log_dir().map(|d| Tail::at_end(d.join("Player.log"))) }
	}

	/// A look: the words of the lines written since the last.
	pub(super) fn look(&mut self) -> Vec<Word> {
		let Some(tail) = self.tail.as_mut() else { return Vec::new() };
		let mut words = Vec::new();
		if tail.began() {
			// the header line of a log that began before this tail read it
			let head = File::open(tail.path()).ok().and_then(|f| {
				let mut bytes = Vec::new();
				f.take(HEAD_BYTES).read_to_end(&mut bytes).ok().map(|_| bytes)
			});
			let head = head.map(|b| String::from_utf8_lossy(&b).into_owned()).unwrap_or_default();
			let began = head.lines().filter_map(arena::line).find(|l| *l == Line::Began);
			words.extend(began.into_iter().flat_map(word));
		}
		words.extend(tail.lines().iter().filter_map(|l| arena::line(l)).flat_map(word));
		words
	}
}
