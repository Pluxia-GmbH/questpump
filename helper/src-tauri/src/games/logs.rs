//! The games added on 2026-10-29 heard by one log each at a known place (docs/computer-games.md §6.2): Path of
//! Exile's `Client.txt` (in each install of Path of Exile or Path of Exile 2 found: Steam's libraries, the Epic Games
//! Launcher's manifests, Grinding Gear Games' own folders), Warframe's `EE.log`, Fortnite's `FortniteGame.log`, and
//! Deadlock's and Team Fortress 2's `console.log`, which the game writes only with the launch option `-condebug`. Each
//! is followed while the game runs and is watched, and read once more at its close, from its end at the first look
//! (`Tail::at_end`: a log made anew at the game's start is read from its beginning), each line handed to
//! questpump-core's reader for the game (`feeds::Lines`), which looks at its own tokens alone; no line is kept. Later
//! that day NARAKA: BLADEPOINT's and Fall Guys' Unity `Player.log` (in the user's `LocalLow`), Dead by Daylight's
//! `DeadByDaylight.log` and VALORANT's `ShooterGame.log` (in the local application data) and Hunt: Showdown 1896's
//! `USER\Game.log` in its Steam library. Windows for all of them; macOS for Path of Exile alone (the only one with a
//! Mac build, 2026-10-29), whose log goes into `~/Library/Caches/com.GGG.PathOfExile/Logs` there (one guide on the
//! game's forum, unverified until it runs on a Mac).

use std::path::PathBuf;

use questpump_core::feeds::{
	Lines, deadbydaylight, deadlock, fallguys, fortnite, hunt, naraka, pathofexile, teamfortress, valorant, warframe,
};
use questpump_core::games::Watched;

use super::tail::Tail;
use super::{Word, epic, steam};

/// Grinding Gear Games' own installers' folders (Windows).
#[cfg(windows)]
const POE: &str = r"C:\Program Files (x86)\Grinding Gear Games\Path of Exile"; // copy-lint: not shown (a path)
#[cfg(windows)]
const POE2: &str = r"C:\Program Files (x86)\Grinding Gear Games\Path of Exile 2"; // copy-lint: not shown (a path)
/// The folder Path of Exile's own Mac launcher downloads the game into, under Application Support (macOS).
#[cfg(not(windows))]
const POE_MAC: &str = "Path of Exile"; // copy-lint: not shown (a folder)
/// Path of Exile's log folder on macOS, under the user's Caches.
#[cfg(not(windows))]
const POE_MAC_LOGS: [&str; 2] = ["com.GGG.PathOfExile", "Logs"]; // copy-lint: not shown (a path)
/// Fortnite's launcher, which the Epic Games Launcher's manifest names as the game's launch executable.
const FORTNITE_LAUNCHER: &str = "FortniteLauncher.exe";

/// Every install of Path of Exile or Path of Exile 2 found: its Steam apps' folders, the Epic Games Launcher's installs
/// that launch one of its programs, and Grinding Gear Games' own folders, each once.
pub(super) fn pathofexile_installs() -> Vec<PathBuf> {
	let images = Watched::PathOfExile.windows_images();
	let steam = Watched::PathOfExile.steam_apps().iter().filter_map(|&app| steam::app_dir(app, false));
	let epic = epic::items()
		.into_iter()
		.filter(|item| images.iter().any(|image| item.launches(image)))
		.filter_map(|item| item.install_location.map(PathBuf::from));
	#[cfg(windows)]
	let own: Vec<PathBuf> = [POE, POE2].into_iter().map(PathBuf::from).collect();
	#[cfg(not(windows))]
	let own: Vec<PathBuf> = dirs::data_dir().map(|d| d.join(POE_MAC)).into_iter().collect();
	let mut out: Vec<PathBuf> = Vec::new();
	for dir in steam.chain(epic).chain(own).filter(|d| d.is_dir()) {
		if !out.contains(&dir) {
			out.push(dir);
		}
	}
	out
}

/// Path of Exile's `Client.txt`: in each install's `logs` (Windows); in the user's Caches on macOS, `LatestClient.txt`
/// there when no `Client.txt` is (Mac players name both).
fn pathofexile_logs() -> Vec<PathBuf> {
	#[cfg(windows)]
	{
		pathofexile_installs().into_iter().map(|d| d.join("logs").join("Client.txt")).collect()
	}
	#[cfg(not(windows))]
	{
		let Some(dir) = dirs::cache_dir().map(|d| POE_MAC_LOGS.iter().fold(d, |d, part| d.join(part))) else {
			return Vec::new();
		};
		let client = dir.join("Client.txt");
		let latest = dir.join("LatestClient.txt");
		vec![if !client.is_file() && latest.is_file() { latest } else { client }]
	}
}

/// Warframe's folder in the local application data, which holds its log and its own launcher's install
/// (`Downloaded\Public`).
pub(super) fn warframe_dir() -> Option<PathBuf> {
	dirs::data_local_dir().map(|d| d.join("Warframe"))
}

/// The user's `LocalLow` folder, where Unity's games write their `Player.log` (Windows); none elsewhere.
pub(super) fn local_low() -> Option<PathBuf> {
	#[cfg(windows)]
	{
		use windows::Win32::UI::Shell::FOLDERID_LocalAppDataLow;
		super::win::known_folder(&FOLDERID_LocalAppDataLow)
	}
	#[cfg(not(windows))]
	{
		None
	}
}

/// The folder each log game of the local application data or `LocalLow` writes its log into (NARAKA: BLADEPOINT's,
/// Fall Guys', Dead by Daylight's, VALORANT's), there once the game ran; none for another game.
pub(super) fn log_dir(watched: &Watched) -> Option<PathBuf> {
	let local = |parts: &[&str]| dirs::data_local_dir().map(|d| parts.iter().fold(d, |d, part| d.join(part)));
	match watched {
		Watched::Naraka => local_low().map(|d| d.join("24Entertainment").join("Naraka")),
		Watched::FallGuys => local_low().map(|d| d.join("Mediatonic").join("FallGuys_client")),
		Watched::DeadByDaylight => local(&["DeadByDaylight", "Saved", "Logs"]), // copy-lint: not shown (a path)
		Watched::Valorant => local(&["VALORANT", "Saved", "Logs"]),             // copy-lint: not shown (a path)
		_ => None,
	}
}

/// Fortnite's folder of logs in the local application data.
pub(super) fn fortnite_logs() -> Option<PathBuf> {
	dirs::data_local_dir().map(|d| d.join("FortniteGame").join("Saved").join("Logs"))
}

/// Fortnite's install as the Epic Games Launcher's manifest names it (the one that launches its launcher or its game).
pub(super) fn fortnite_install() -> Option<PathBuf> {
	let game = Watched::Fortnite.windows_images().first()?;
	epic::items()
		.into_iter()
		.filter(|item| item.launches(FORTNITE_LAUNCHER) || item.launches(game))
		.find_map(|item| Some(PathBuf::from(item.install_location?)).filter(|p| p.is_dir()))
}

/// The logs a game heard by one log is read from; none for another game.
fn paths(watched: &Watched) -> Vec<PathBuf> {
	let console = |inside: &[&str]| {
		steam::game_dir(watched, false).map(|dir| inside.iter().fold(dir, |d, part| d.join(part)).join("console.log"))
	};
	match watched {
		Watched::PathOfExile => pathofexile_logs(),
		Watched::Warframe => warframe_dir().map(|d| d.join("EE.log")).into_iter().collect(),
		Watched::Fortnite => fortnite_logs().map(|d| d.join("FortniteGame.log")).into_iter().collect(),
		Watched::Deadlock => console(&["game", "citadel"]).into_iter().collect(),
		Watched::TeamFortress => console(&["tf"]).into_iter().collect(),
		Watched::Naraka | Watched::FallGuys => log_dir(watched).map(|d| d.join("Player.log")).into_iter().collect(),
		Watched::DeadByDaylight => log_dir(watched).map(|d| d.join("DeadByDaylight.log")).into_iter().collect(),
		Watched::Valorant => log_dir(watched).map(|d| d.join("ShooterGame.log")).into_iter().collect(),
		Watched::Hunt => steam::game_dir(watched, false).map(|d| d.join("USER").join("Game.log")).into_iter().collect(),
		_ => Vec::new(),
	}
}

/// A game's reader of its lines; none for another game.
fn reader(watched: &Watched) -> Option<Box<dyn Lines + Send>> {
	Some(match watched {
		Watched::PathOfExile => Box::new(pathofexile::Areas::default()),
		Watched::Warframe => Box::new(warframe::Missions::default()),
		Watched::Fortnite => Box::new(fortnite::Matches),
		Watched::Deadlock => Box::new(deadlock::Lobby::default()),
		Watched::TeamFortress => Box::new(teamfortress::Lobby),
		Watched::Naraka => Box::new(naraka::Battles::default()),
		Watched::FallGuys => Box::new(fallguys::Shows::default()),
		Watched::DeadByDaylight => Box::new(deadbydaylight::Trials::default()),
		Watched::Hunt => Box::new(hunt::Missions::default()),
		Watched::Valorant => Box::new(valorant::Flow::default()),
		_ => return None,
	})
}

/// A game's log (or, for Path of Exile, the log of each install: only the one that runs writes) while the game runs
/// and is watched.
pub(super) struct Logs {
	tails: Vec<Tail>,
	lines: Option<Box<dyn Lines + Send>>,
}

impl Logs {
	pub(super) fn of(watched: &Watched) -> Self {
		Self { tails: paths(watched).into_iter().map(Tail::at_end).collect(), lines: reader(watched) }
	}

	/// A look: the words of the lines written since the last; any line read tells that the game is heard.
	pub(super) fn look(&mut self) -> Vec<Word> {
		let Some(reader) = self.lines.as_mut() else { return Vec::new() };
		let mut words = Vec::new();
		for tail in &mut self.tails {
			let lines = tail.lines();
			if !lines.is_empty() && words.is_empty() {
				words.push(Word::Up);
			}
			words.extend(lines.iter().filter_map(|l| reader.push(l)).map(Word::Match));
		}
		words
	}
}
