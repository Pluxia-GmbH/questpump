//! Finding the running game (docs/companion.md, "Finding the install"): the client's image path, never its process
//! name, and its flavour read from the path's `_<flavour>_` folder, never hard-coded; the client is whatever program
//! sits right in a flavour folder (`in_flavour`), whatever its name (the Forever launch's client and app are named
//! only once it ships). On macOS the processes' image paths, from the look's process snapshot (`games/procs.rs`, which
//! the games on the computer share); on Windows the game's window (class `GxWindowClass`) and its process's image path,
//! read with `PROCESS_QUERY_LIMITED_INFORMATION` only. Of several clients the helper follows one process (`running`):
//! the capture reads that process's window and the mirror goes into its folder. The game folder when the game is not
//! running: `install.rs`.

use std::path::{Path, PathBuf};

use crate::games::procs::Snapshot;
use crate::install;

#[cfg(windows)]
pub use win::window;

/// The running client, when there is one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Game {
	/// The flavour folder's name, such as `_classic_beta_`.
	pub flavour: String,
	pub addons: PathBuf,
	/// The client's process, whose window the capture reads: a client started again is another game.
	pub pid: u32,
}

/// A running client and when its process started, in the system's own units (only compared).
struct Client {
	game: Game,
	started: u64,
}

/// A flavour folder's name: `_retail_`, `_classic_beta_` and the like.
pub fn is_flavour(name: &str) -> bool {
	name.len() > 2 && name.starts_with('_') && name.ends_with('_')
}

/// The client process `pid` runs from an executable path, with the `Interface/AddOns` folder of its install: the path's
/// last `_<flavour>_` component, the one nearest the executable (a folder above the install may be named like one, such
/// as `D:\_Games_`).
fn from_executable(path: &Path, pid: u32) -> Option<Game> {
	let (at, flavour) = path
		.components()
		.enumerate()
		.filter_map(|(i, part)| Some((i, part.as_os_str().to_str().filter(|n| is_flavour(n))?.to_owned())))
		.last()?;
	let root: PathBuf = path.components().take(at + 1).collect();
	Some(Game { flavour, addons: root.join("Interface").join("AddOns"), pid })
}

/// A program right inside a flavour folder: a Windows client's executable (`_retail_\Wow.exe`), a Mac client's app
/// bundle (`_classic_beta_/World of Warcraft Beta.app`).
fn in_flavour(program: &Path) -> bool {
	program.parent().and_then(Path::file_name).and_then(|n| n.to_str()).is_some_and(is_flavour)
}

/// A Windows client's executable name: `Wow.exe`, `WowB.exe`, `WowT.exe`, `WowClassic*.exe` and their `-arm64`
/// variants (`WowUp`'s list), in any case: the games' picker leaves them out by the snapshot's names (`games/mod.rs`),
/// which carry no path; the game itself is found by its window and its folder (`win::clients`).
#[cfg_attr(not(windows), expect(dead_code, reason = "only Windows names processes by their image file"))]
pub fn is_client_name(name: &str) -> bool {
	let head = name.get(..3).is_some_and(|h| h.eq_ignore_ascii_case("wow"));
	let tail = name.len() >= 7 && name.get(name.len() - 4..).is_some_and(|t| t.eq_ignore_ascii_case(".exe"));
	head && tail
}

/// The client the helper follows, one rule on both systems: every client whose flavour folder is there counts, with an
/// AddOns folder or without (a new install has none until an addon is put there, and its running client is the game on
/// this computer that Add QuestPump to the game prefers: `runtime/addon.rs` `first_target`); of several, one whose
/// folder holds QuestPump (no strip shows in another, and it gets no mirror: `Helper::game_addons`), then the one
/// followed now while it runs (leaving it would end its session, and its logout rule would owe the moves still open in
/// it), then the one started last. The capture reads the chosen process's window, so the strip and the mirror come
/// from the same client. A choice among several is logged when it changes.
pub fn running(followed: Option<&Game>, snap: &Snapshot) -> Option<Game> {
	let clients: Vec<Client> = clients(snap).into_iter().filter(|c| install::is_game(&c.game.addons)).collect();
	let chosen = clients
		.iter()
		.max_by_key(|c| (install::holds_questpump(&c.game.addons), followed == Some(&c.game), c.started))?
		.game
		.clone();
	if clients.len() > 1 && followed != Some(&chosen) {
		let all: Vec<String> = clients.iter().map(|c| format!("{} (pid {})", c.game.flavour, c.game.pid)).collect();
		log::info!(
			"game: {} clients run, {}; following {} (pid {})",
			all.len(),
			all.join(", "),
			chosen.flavour,
			chosen.pid
		);
	}
	Some(chosen)
}

/// Whether an image path is a client's own executable: the bundle right around it (`<name>.app/Contents/MacOS/`) sits
/// right in a flavour folder, whatever its name ("World of Warcraft Beta.app" in `_classic_beta_`, and the Forever
/// launch's app, unnamed before it ships). A helper the client carries in its bundle is none: the voice chat's
/// `Contents/Helpers/WowVoiceProxy.app`, which starts with a login and, as the process started last, was followed
/// instead of the client on 2026-10-04 (the strip went unread until the helper started again).
#[cfg(target_os = "macos")]
pub fn is_client_path(path: &Path) -> bool {
	let mut up = path.ancestors().skip(1);
	let (Some(macos), Some(contents), Some(app)) = (up.next(), up.next(), up.next()) else { return false };
	macos.file_name().is_some_and(|n| n == "MacOS")
		&& contents.file_name().is_some_and(|n| n == "Contents")
		&& app.extension().is_some_and(|e| e.eq_ignore_ascii_case("app"))
		&& in_flavour(app)
}

/// Every running World of Warcraft client, found by its image path (never by the process name), with its start.
#[cfg(target_os = "macos")]
fn clients(snap: &Snapshot) -> Vec<Client> {
	snap.procs
		.iter()
		.filter_map(|p| {
			let path = p.path.as_deref().filter(|path| is_client_path(path))?;
			let game = from_executable(path, p.pid)?;
			let started = libproc::proc_pid::pidinfo::<libproc::bsd_info::BSDInfo>(i32::try_from(p.pid).ok()?, 0)
				.map_or(0, |i| i.pbi_start_tvsec.saturating_mul(1_000_000).saturating_add(i.pbi_start_tvusec));
			Some(Client { game, started })
		})
		.collect()
}

/// Every running World of Warcraft client, found by its window.
#[cfg(windows)]
fn clients(snap: &Snapshot) -> Vec<Client> {
	win::clients(snap)
}

#[cfg(windows)]
mod win {
	use std::ffi::OsString;
	use std::os::windows::ffi::OsStringExt;
	use std::path::PathBuf;

	use windows::Win32::Foundation::{FILETIME, HANDLE, HWND};
	use windows::Win32::System::Threading::{
		GetProcessTimes, OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
	};
	use windows::Win32::UI::WindowsAndMessaging::{GetClassNameW, IsWindowVisible};
	use windows::core::{Owned, PWSTR};

	use super::{Client, from_executable, in_flavour};
	use crate::games::procs::{Snapshot, area, process_id, top_level};

	/// The client's main window class, whatever the locale's title (rust-helper `windows-capture-and-install.md`).
	const CLASS: &str = "GxWindowClass";
	/// The longest path `QueryFullProcessImageNameW` can return, in UTF-16 units (the NT path limit).
	const PATH_UNITS: usize = 32_768;

	/// The game's main window: its handle as a number, since a window handle is not `Send` (`hwnd` rebuilds it on the
	/// thread that uses it).
	#[derive(Debug, Clone, PartialEq, Eq)]
	pub struct GameWindow {
		hwnd: usize,
	}

	impl GameWindow {
		pub fn hwnd(&self) -> HWND {
			HWND(std::ptr::without_provenance_mut(self.hwnd))
		}
	}

	/// The main window of the client process `pid` (`super::running`'s choice): its top-level `GxWindowClass` window;
	/// of several, the largest visible one.
	pub fn window(pid: u32) -> Option<GameWindow> {
		let mut best: Option<((bool, i64), GameWindow)> = None;
		top_level(|hwnd| {
			if !has_class(hwnd) || process_id(hwnd) != Some(pid) {
				return;
			}
			// SAFETY: IsWindowVisible only reads the state of a window handle EnumWindows just passed.
			let rank = (unsafe { IsWindowVisible(hwnd) }.as_bool(), area(hwnd));
			if best.as_ref().is_none_or(|(r, _)| rank > *r) {
				best = Some((rank, GameWindow { hwnd: hwnd.0.addr() }));
			}
		});
		best.map(|(_, w)| w)
	}

	/// Every running client: a process with a top-level `GxWindowClass` window whose executable sits right in a
	/// `_<flavour>_` folder, whatever its name. Only a process the snapshot lists is opened (one started after the
	/// snapshot is found at the next look).
	pub(super) fn clients(snap: &Snapshot) -> Vec<Client> {
		let mut pids: Vec<u32> = Vec::new();
		top_level(|hwnd| {
			if has_class(hwnd)
				&& let Some(pid) = process_id(hwnd)
				&& snap.name_of(pid).is_some()
				&& !pids.contains(&pid)
			{
				pids.push(pid);
			}
		});
		pids.into_iter().filter_map(client).collect()
	}

	/// A client's process, read with `PROCESS_QUERY_LIMITED_INFORMATION` only (never a handle that could read the
	/// game's memory, rust-helper rule 10): its image path and its start.
	fn client(pid: u32) -> Option<Client> {
		// SAFETY: OpenProcess returns a fresh handle on success; `Owned` is its only owner and closes it once on drop.
		let process = unsafe { Owned::new(OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?) };
		let path = image_path(*process)?;
		let game = from_executable(&path, pid).filter(|_| in_flavour(&path))?;
		Some(Client { game, started: started(*process) })
	}

	fn has_class(hwnd: HWND) -> bool {
		let mut name = [0u16; 64];
		// SAFETY: GetClassNameW writes at most the slice's length of UTF-16 units into `name`, which lives for the
		// call.
		let len = unsafe { GetClassNameW(hwnd, &mut name) };
		let len = usize::try_from(len).unwrap_or(0);
		name.get(..len).is_some_and(|n| CLASS.encode_utf16().eq(n.iter().copied()))
	}

	/// The image path of a process opened with `PROCESS_QUERY_LIMITED_INFORMATION`.
	fn image_path(process: HANDLE) -> Option<PathBuf> {
		let mut path = vec![0u16; PATH_UNITS];
		let mut len = u32::try_from(path.len()).ok()?;
		// SAFETY: `path` holds `len` writable UTF-16 units; the call writes at most that many and sets `len` to the
		// units written, without the terminator.
		unsafe { QueryFullProcessImageNameW(process, PROCESS_NAME_WIN32, PWSTR(path.as_mut_ptr()), &raw mut len) }
			.ok()?;
		// as UTF-16 units: an NTFS name may hold an unpaired surrogate, which a String cannot
		Some(PathBuf::from(OsString::from_wide(path.get(..usize::try_from(len).ok()?)?)))
	}

	/// When a process opened with `PROCESS_QUERY_LIMITED_INFORMATION` started, in 100-ns units since 1601; 0 when it
	/// cannot be read.
	fn started(process: HANDLE) -> u64 {
		let [mut created, mut exited, mut kernel, mut user] = [FILETIME::default(); 4];
		// SAFETY: GetProcessTimes writes one FILETIME through each pointer, all four living for the call; the handle
		// carries the access right it needs.
		if unsafe { GetProcessTimes(process, &raw mut created, &raw mut exited, &raw mut kernel, &raw mut user) }
			.is_err()
		{
			return 0;
		}
		(u64::from(created.dwHighDateTime) << 32) | u64::from(created.dwLowDateTime)
	}
}
