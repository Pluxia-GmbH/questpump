//! Hearthstone's own log (docs/plan.md slice 48; docs/computer-games.md §6.2): its `log.config` (Windows:
//! `%LOCALAPPDATA%\Blizzard\Hearthstone`; macOS: `~/Library/Preferences/Blizzard/Hearthstone`, as the deck trackers HDT
//! and HSTracker name them), which gets a `[Power]` section when it holds none (`files::add_section`: the file is the
//! trackers' too, so it is never rewritten whole or removed), and the `Power.log` the game then writes under its
//! install's `Logs` folder, in a folder of its own each session (`Logs/Hearthstone_<date>/Power.log`; `Logs/Power.log`
//! before that, read only while no session folder exists), followed while the game runs and is set up ("Ask after each
//! match", slice 56), each new log with a fresh reader. Each line goes to questpump-core's `feeds::hearthstone` for
//! whether a game was created or completed (and whether it is watched as a spectator); every other line is passed over,
//! and no line is kept. The install is where the Battle.net Agent's database names it (`hs_beta`), else the default
//! folder.

use std::path::PathBuf;

use questpump_core::feeds::hearthstone::Power;

use super::Word;
use super::tail::{Follow, newest};
use crate::install;

/// The Battle.net Agent's product uid for Hearthstone.
const UID: &str = "hs_beta";
/// A session's folder of logs, by its name's start.
const SESSION: &str = "Hearthstone_";

/// Where `log.config` is, whether it is there yet or not.
pub(super) fn config() -> Option<PathBuf> {
	#[cfg(windows)]
	let prefs = dirs::data_local_dir();
	#[cfg(not(windows))]
	let prefs = dirs::home_dir().map(|h| h.join("Library").join("Preferences"));
	prefs.map(|p| p.join("Blizzard").join("Hearthstone").join("log.config"))
}

/// Hearthstone's install: where the Agent names it, else the default folder.
pub(super) fn install() -> Option<PathBuf> {
	let default = || {
		let dir = if cfg!(windows) {
			PathBuf::from(r"C:\Program Files (x86)\Hearthstone")
		} else {
			PathBuf::from("/Applications/Hearthstone")
		};
		dir.is_dir().then_some(dir)
	};
	install::battlenet_install(UID).or_else(default)
}

/// The `Power.log` the game writes now: the newest session folder's once the game made it (none until then); the one
/// beside the folders only while there is no session folder (a game from before them), so an old log there never
/// replays its games.
fn power_log() -> Option<PathBuf> {
	let logs = install()?.join("Logs");
	let log = match newest(&logs, |name| name.starts_with(SESSION)) {
		Some(session) => session.join("Power.log"),
		None => logs.join("Power.log"),
	};
	log.is_file().then_some(log)
}

/// The log while the game runs and is set up.
#[derive(Default)]
pub(super) struct Hearthstone {
	follow: Follow,
	power: Power,
}

impl Hearthstone {
	/// A look: the words of the lines written since the last (the old log's last ones first, when a newer log took
	/// its place); while the game runs, its newest log looked for again.
	pub(super) fn look(&mut self, running: bool) -> Vec<Word> {
		let (left, lines) = self.follow.lines(running, power_log);
		let mut words = Vec::new();
		self.read(&left, &mut words);
		if self.follow.began() {
			// a new log watches no game: a spectate the last one never ended (a crash, a quit) is over
			self.power = Power::default();
		}
		self.read(&lines, &mut words);
		words
	}

	/// One log's lines into words: a game's lines say the section is read; a game created or completed is its word.
	fn read(&mut self, lines: &[String], words: &mut Vec<Word>) {
		for line in lines {
			if let Some(fed) = self.power.push(line) {
				words.extend([Word::Up, Word::Match(fed)]);
			} else if words.is_empty() {
				words.push(Word::Up);
			}
		}
	}
}
