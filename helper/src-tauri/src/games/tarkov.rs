//! Escape from Tarkov's logs (2026-10-29; docs/computer-games.md §6.2; `TarkovMonitor` reads the same files): the
//! install where its uninstall record names it (`EscapeFromTarkov` from Battlestate Games' launcher, `Steam App 3932890`
//! from Steam, `InstallLocation`, in the user's and the machine's registry), else Steam's library; its logs in
//! `<install>\Logs` (`<install>\build\Logs` in Steam's build), a folder `log_<date>_<version>` each session, of which
//! the newest one's `application` and `notifications` logs are followed while the game runs and is watched, and read
//! once more at its close (`Follow`: a session's new folder found within 10 s). Each line goes to questpump-core's
//! `feeds::tarkov` for a raid begun or left, by its token alone; no line is kept. Windows only, where the game runs.

use std::path::PathBuf;

use questpump_core::feeds::Lines;
use questpump_core::feeds::tarkov::Raid;
use questpump_core::games::Watched;

use super::tail::{Follow, newest};
use super::{Word, steam};

/// The game's uninstall records under `Software\Microsoft\Windows\CurrentVersion\Uninstall`: Battlestate Games'
/// launcher's and Steam's.
#[cfg(windows)]
const RECORDS: [&str; 2] = ["EscapeFromTarkov", "Steam App 3932890"]; // copy-lint: not shown (registry keys)

/// The installs the game's uninstall records name (`InstallLocation`), in the user's registry and then the machine's,
/// in its 64-bit and its 32-bit views.
#[cfg(windows)]
fn recorded() -> Vec<PathBuf> {
	use windows::Win32::System::Registry::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
	use windows::core::{HSTRING, PCWSTR, w};
	const UNINSTALL: [&str; 2] = [
		r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
		r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
	];
	let mut out = Vec::new();
	for root in [HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE] {
		for uninstall in UNINSTALL {
			for record in RECORDS {
				let key = HSTRING::from(format!(r"{uninstall}\{record}"));
				if let Some(dir) = super::win::text(root, PCWSTR(key.as_ptr()), w!("InstallLocation")) {
					out.push(PathBuf::from(dir));
				}
			}
		}
	}
	out
}

/// No records outside Windows.
#[cfg(not(windows))]
fn recorded() -> Vec<PathBuf> {
	Vec::new()
}

/// The game's install: the first its records name that is there, else Steam's library's.
pub(super) fn install() -> Option<PathBuf> {
	recorded().into_iter().find(|d| d.is_dir()).or_else(|| steam::game_dir(&Watched::Tarkov, false))
}

/// The game's folder of session logs: `Logs` in its install, or `build\Logs` in Steam's build.
fn logs() -> Option<PathBuf> {
	let install = install()?;
	[install.join("Logs"), install.join("build").join("Logs")].into_iter().find(|d| d.is_dir())
}

/// The newest session's log whose name ends `<kind>_000.log` (or `<kind>.log`), in any case.
fn session_log(kind: &str) -> Option<PathBuf> {
	let session = newest(&logs()?, |name| name.starts_with("log_"))?;
	let ends = [format!("{kind}_000.log"), format!("{kind}.log")];
	newest(&session, |name| {
		let name = name.to_ascii_lowercase();
		ends.iter().any(|end| name.ends_with(end.as_str()))
	})
}

/// The two logs while the game runs and is watched.
#[derive(Default)]
pub(super) struct Tarkov {
	application: Follow,
	notifications: Follow,
	raid: Raid,
}

impl Tarkov {
	/// A look: the words of both logs' lines written since the last (the old session's last ones first, when a newer
	/// session took its place); while the game runs, the newest session looked for again.
	pub(super) fn look(&mut self, running: bool) -> Vec<Word> {
		let (a_left, a_lines) = self.application.lines(running, || session_log("application"));
		let (n_left, n_lines) = self.notifications.lines(running, || session_log("notifications"));
		let lines = [a_left, n_left, a_lines, n_lines].concat();
		let mut words = Vec::new();
		if !lines.is_empty() {
			words.push(Word::Up);
		}
		words.extend(lines.iter().filter_map(|l| self.raid.push(l)).map(Word::Match));
		words
	}
}
