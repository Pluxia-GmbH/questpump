//! Where the Riot Client installed League of Legends (with Teamfight Tactics), Legends of Runeterra and, since
//! 2026-10-29, VALORANT (docs/computer-games.md §6.1 and §6.5): on Windows its own record of each product,
//! `%ProgramData%\Riot Games\Metadata\<id>.live\<id>.live.product_settings.yaml` (`league_of_legends`, `bacon`,
//! `valorant`, this one unverified), whose
//! `product_install_full_path` line names the install's folder wherever the player put it (read on Rapha's PC on
//! 2026-10-26, whose League sits outside the default folder: `product_install_full_path: "C:/Games/Riot Games/..."`);
//! else the folder the Riot Client uses by default. On macOS no such record has been read on a Mac yet, so League is
//! looked for at its default place alone. A read of the record, bounded to `SETTINGS_BYTES`, never a write, and nothing
//! of it is kept but the folder.

use std::io::Read as _;
use std::path::PathBuf;

use questpump_core::games::Watched;

/// League of Legends' install where the Riot Client puts it by default.
#[cfg(windows)]
const LEAGUE: &str = r"C:\Riot Games\League of Legends"; // copy-lint: not shown (a path)
#[cfg(not(windows))]
const LEAGUE: &str = "/Applications/League of Legends.app"; // copy-lint: not shown (a path)
/// Legends of Runeterra's (a Windows game).
#[cfg(windows)]
const RUNETERRA: Option<&str> = Some(r"C:\Riot Games\LoR"); // copy-lint: not shown (a path)
#[cfg(not(windows))]
const RUNETERRA: Option<&str> = None;
/// VALORANT's (a Windows game).
#[cfg(windows)]
const VALORANT: Option<&str> = Some(r"C:\Riot Games\VALORANT"); // copy-lint: not shown (a path)
#[cfg(not(windows))]
const VALORANT: Option<&str> = None;

/// The most of a product's settings file read (the League file is about 1 KB).
const SETTINGS_BYTES: u64 = 64 * 1024;
/// The settings file's key that names the install's folder, a top-level line.
const FULL_PATH: &str = "product_install_full_path:";

/// A Riot game's install folder, when it is there: as the Riot Client's record names it, else at its default place;
/// none for any other game.
pub(super) fn install(watched: &Watched) -> Option<PathBuf> {
	let (id, default) = match watched {
		Watched::League => ("league_of_legends", Some(LEAGUE)),
		Watched::Runeterra => ("bacon", RUNETERRA),
		Watched::Valorant => ("valorant", VALORANT),
		_ => return None,
	};
	let there = |d: &PathBuf| d.is_dir();
	recorded(id).filter(there).or_else(|| default.map(PathBuf::from).filter(there))
}

/// The install folder the Riot Client's record of the product `id`'s live patchline names; none outside Windows, where
/// the record is unverified, or when it is missing or names none.
fn recorded(id: &str) -> Option<PathBuf> {
	if !cfg!(windows) {
		return None;
	}
	let product = format!("{id}.live");
	let file = PathBuf::from(std::env::var_os("ProgramData")?)
		.join("Riot Games")
		.join("Metadata")
		.join(&product)
		.join(format!("{product}.product_settings.yaml"));
	let mut bytes = Vec::new();
	std::fs::File::open(file).ok()?.take(SETTINGS_BYTES).read_to_end(&mut bytes).ok()?;
	// the record is in `ProgramData`, which other accounts may write: a folder of this computer's own disks alone
	full_path(&String::from_utf8_lossy(&bytes)).map(PathBuf::from).filter(|p| crate::install::on_this_computer(p))
}

/// The `product_install_full_path` line's value, its quotes taken off; a plain read of that one top-level line, not
/// YAML (the Riot Client writes it as one double-quoted path with forward slashes).
fn full_path(settings: &str) -> Option<&str> {
	let value = settings.lines().find_map(|line| line.strip_prefix(FULL_PATH))?.trim();
	let value = value
		.strip_prefix('"')
		.and_then(|v| v.strip_suffix('"'))
		.or_else(|| value.strip_prefix('\'').and_then(|v| v.strip_suffix('\'')))
		.unwrap_or(value);
	(!value.is_empty()).then_some(value)
}
