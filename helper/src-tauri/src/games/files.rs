//! The files the helper writes into the games' folders (docs/plan.md slice 35): Valve's cfg files and Rocket League's
//! `TAStatsAPI.ini`, each written whole through a temporary file and a rename, and listed in the settings with its game,
//! so turning a game off removes what was written for it and nothing else. On Windows the list is mirrored to
//! `game-files` in the data folder (UTF-16LE with a BOM, one path a line), which the uninstaller's hook reads to remove
//! them (`windows/hooks.nsh`); macOS has no uninstaller, and a file left behind points at a closed port of the loopback.
//! Since slice 48 a file other programs keep too gets the helper's part added at its end instead (Hearthstone's
//! `log.config`, which the deck trackers share): never written whole, listed as shared, and never removed, by turning
//! the game off or by the uninstaller; it stays listed after the game is off, since it still holds the helper's part.

use std::io;
use std::path::{Path, PathBuf};

use questpump_core::mirror;

use crate::settings::{GameFile, GamesSettings};

/// The uninstaller's copy of the list, in the data folder.
#[cfg(windows)]
pub const LIST: &str = "game-files";

/// Whether the helper wrote this file for this game.
pub fn listed(games: &GamesSettings, game: &str, path: &Path) -> bool {
	games.written.iter().any(|f| f.game == game && f.path == path)
}

/// Writes `text` as the file `path` for `game`, its folder created first, and lists it (`list`).
///
/// # Errors
/// The folder or the file could not be written, or is on no disk of this computer's (`local`): nothing is listed then.
pub fn write(games: &mut GamesSettings, game: &str, path: &Path, text: &str) -> io::Result<()> {
	let (Some(dir), Some(name)) = (path.parent(), path.file_name().and_then(|n| n.to_str())) else {
		return Err(io::Error::new(io::ErrorKind::InvalidInput, "no folder or no file name"));
	};
	local(dir)?;
	std::fs::create_dir_all(dir)?;
	mirror::write_atomic(dir, name, text.as_bytes())?;
	list(games, game, path);
	Ok(())
}

/// A game's folder from records other accounts may write (the Steam, Epic and Battle.net lists): only one on this
/// computer's own disks is written in (`install::on_this_computer`), never a network share.
fn local(path: &Path) -> io::Result<()> {
	if crate::install::on_this_computer(path) {
		Ok(())
	} else {
		Err(io::Error::new(io::ErrorKind::InvalidInput, "a folder on a network share"))
	}
}

/// Lists `path` as the one file written for `game`; another listed for it (its folder moved, its install moved) goes.
pub fn list(games: &mut GamesSettings, game: &str, path: &Path) {
	listed_as(games, game, path, false);
}

/// Lists `path` for `game` as written whole (`shared` false) or as a shared file the helper added to; another file
/// written whole for it goes (a shared one stays listed: it keeps the helper's part).
fn listed_as(games: &mut GamesSettings, game: &str, path: &Path, shared: bool) {
	for old in take(games, game).into_iter().filter(|old| old != path) {
		remove(game, &old);
	}
	games.written.retain(|f| f.game != game || f.path != path);
	games.written.push(GameFile { game: game.to_owned(), path: path.to_path_buf(), shared });
}

/// Adds `text` at the end of the shared file `path` for `game` (made when it is missing, never written whole: other
/// programs keep their parts of it), and lists it as shared.
///
/// # Errors
/// The file could not be opened or written, or is on no disk of this computer's (`local`): nothing is listed then.
pub fn add_section(games: &mut GamesSettings, game: &str, path: &Path, text: &str) -> io::Result<()> {
	use std::io::Write as _;
	local(path)?;
	let mut file = std::fs::OpenOptions::new().append(true).create(true).open(path)?;
	file.write_all(text.as_bytes())?;
	file.sync_all()?;
	listed_as(games, game, path, true);
	Ok(())
}

/// Unlists every file written whole for `game` and gives their paths, which the helper may remove; a shared file stays
/// as it is and stays listed (the Menu's What it does names it, the uninstaller's list never does).
pub fn take(games: &mut GamesSettings, game: &str) -> Vec<PathBuf> {
	let (gone, kept): (Vec<GameFile>, Vec<GameFile>) =
		games.written.drain(..).partition(|f| f.game == game && !f.shared);
	games.written = kept;
	gone.into_iter().map(|f| f.path).collect()
}

/// Removes a file the helper wrote for `game`; one already gone is no error. The log names the game and the file's
/// name, never the folder (a Steam library's path may hold the player's name).
pub fn remove(game: &str, path: &Path) {
	let name = path.file_name().map_or_else(String::new, |n| n.to_string_lossy().into_owned());
	match std::fs::remove_file(path) {
		Ok(()) => log::info!("games: {game}: {name} removed"),
		Err(e) if e.kind() == io::ErrorKind::NotFound => {}
		Err(e) => log::warn!("games: {game}: {name} could not be removed: {e}"),
	}
}

/// The uninstaller's list: UTF-16LE with a BOM (NSIS reads it with `FileReadUTF16LE`), one path a line, CRLF; a shared
/// file is never on it.
#[cfg(windows)]
pub fn list_bytes(games: &GamesSettings) -> Vec<u8> {
	use std::os::windows::ffi::OsStrExt as _;
	let mut units: Vec<u16> = vec![0xfeff];
	for file in games.written.iter().filter(|f| !f.shared) {
		units.extend(file.path.as_os_str().encode_wide());
		units.extend([u16::from(b'\r'), u16::from(b'\n')]);
	}
	units.iter().flat_map(|u| u.to_le_bytes()).collect()
}
