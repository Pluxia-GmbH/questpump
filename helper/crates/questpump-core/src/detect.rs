//! Noticing a game the player could turn on (docs/plan.md slice 48; docs/computer-games.md §6.1): pure readers of what
//! the desktop app's detector looks at (`src-tauri/src/games/detect.rs`): Steam's own `logs/gameprocess_log.txt`, the
//! paths Windows' Known Games List and the launchers' records hold, a Mac app's `Info.plist` category, the deny list
//! of launchers, shells, editors, browsers, runtimes, crash reporters and anti-cheat services, which are no game, the
//! Steam apps that run all day (desktop pets, focus rooms, idle games) and Steam's test app that games borrow, and
//! the games QuestPump refuses (Roblox, docs/computer-games.md §6.5: most of its players are children), which neither
//! the detector nor the hand picker lists. Nothing of a game is kept but its name and its program's key.

use std::collections::HashMap;

/// The programs that are no game though a game's records name them (Windows image names, compared in any case): the
/// launchers and their helpers (Steam's, Epic's, Blizzard's, Riot's, EA's, Ubisoft's, GOG's, Rockstar's, Paradox's and
/// 2K's), the shells and Notepad a game's folder or a player's script opens, the browsers, Java (Minecraft's and every
/// other Java program's), the engines' crash reporters and the anti-cheat services and bootstraps a game starts beside
/// itself (Easy Anti-Cheat's `start_protected_game.exe`; `DENIED_SUFFIXES` for the rest).
const DENIED_IMAGES: [&str; 49] = [
	"steam.exe",
	"steamwebhelper.exe",
	"steamservice.exe",
	"epicgameslauncher.exe",
	"epicwebhelper.exe",
	"battle.net.exe",
	"agent.exe",
	"blizzarderror.exe",
	"riotclientservices.exe",
	"riotclientux.exe",
	"riotclientuxrender.exe",
	"eadesktop.exe",
	"eabackgroundservice.exe",
	"upc.exe",
	"ubisoftconnect.exe",
	"ubisoftgamelauncher.exe",
	"galaxyclient.exe",
	"launcher.exe",
	"launcherpatcher.exe",
	"rockstarservice.exe",
	"socialclubhelper.exe",
	"playgtav.exe",
	"paradox launcher.exe",
	"dowser.exe",
	"2klauncher.exe",
	"cmd.exe",
	"conhost.exe",
	"powershell.exe",
	"pwsh.exe",
	"windowsterminal.exe",
	"notepad.exe",
	"chrome.exe",
	"msedge.exe",
	"firefox.exe",
	"opera.exe",
	"brave.exe",
	"vivaldi.exe",
	"java.exe",
	"javaw.exe",
	"unitycrashhandler64.exe",
	"unitycrashhandler32.exe",
	"crashreportclient.exe",
	"easyanticheat.exe",
	"easyanticheat_eos.exe",
	"beservice.exe",
	"beservice_x64.exe",
	"eaanticheat.gameservicelauncher.exe",
	"gamelaunchhelper.exe",
	"start_protected_game.exe",
];

/// The image names' ends of the anti-cheat bootstraps that start a game and leave it running (`ArmA3_BE.exe`,
/// `BattlEye`'s; `RocketLeague_EAC.exe`, Easy Anti-Cheat's): the game they start is found in their place.
const DENIED_SUFFIXES: [&str; 2] = ["_be.exe", "_eac.exe"];

/// The Steam apps Steam types as games that run all day beside the player's work and play (2026-10-28, the games
/// compatibility research): the desktop pets and taskbar companions, the focus and lo-fi rooms, and the idle games
/// meant to be left running, whose close after hours of running says nothing of a match. Each id and its store name
/// checked against Steam's `appdetails` that day: Bongo Cat, TBH: Task Bar Hero, Deskrawl: Idle ARPG, War of
/// Genesis: Idle Loot, Character House -Desktop Companion-, Revolution Idle, Cookie Clicker, VPet-Simulator, Banana,
/// Desktop Mate, Mini Cozy Room: Lo-Fi, Chill with You : Lo-Fi Story, Spirit City: Lofi Sessions, gogh: Focus with Your
/// Avatar, Melvor Idle, NGU IDLE, Tap Ninja - Idle game, Unnamed Space Idle, Cell to Singularity, Clicker Heroes, Idle
/// Champions of the Forgotten Realms, Firestone – Idle Clicker Online RPG, Typing Farmer, Tap Tap Loot, Fish to Dish:
/// Idle Sushi, Farmer Against Potatoes Idle, Duelist: Idle Expedition, Idle Hero TD, Pibcremental ("idle-incremental
/// clicker"). An incremental game played in sessions (Magic Research 2, Tower Wizard) stays a game. The player may
/// still add any of them by hand.
const DENIED_APPS: [u32; 29] = [
	3_419_430, 3_678_970, 4_623_570, 4_891_320, 4_868_310, 2_763_740, 1_454_400, 1_920_960, 2_923_300, 3_301_060,
	3_511_030, 3_548_580, 2_113_850, 3_213_850, 1_267_910, 1_147_690, 1_891_700, 2_471_100, 977_400, 363_970, 627_690,
	1_013_320, 4_216_670, 3_959_890, 3_399_960, 1_535_560, 4_282_780, 2_897_580, 4_573_090,
];

/// Spacewar, Steam's test app (Steamworks' example game), whose id many games and mods borrow to run through Steam:
/// a process Steam tracks under it is no Steam game of that name, and keeps its program's own key and name.
const BORROWED_APP: u32 = 480;

/// The games QuestPump refuses (docs/computer-games.md §6.5: Roblox, most of whose players are children, and whose
/// terms ask a parent's consent below the age of majority): its player's, launcher's and studio's Windows images and
/// its Mac bundles, which neither the detector nor the hand picker lists.
const REFUSED_IMAGES: [&str; 4] =
	["robloxplayerbeta.exe", "robloxplayerlauncher.exe", "robloxstudiobeta.exe", "robloxstudiolauncherbeta.exe"];
const REFUSED_BUNDLES: [&str; 3] = ["Roblox.app", "RobloxPlayer.app", "RobloxStudio.app"];

/// The Mac launchers whose bundles sit in a Steam library or call themselves games (bundle names, in any case).
const DENIED_BUNDLES: [&str; 7] = [
	"Steam.app",
	"Battle.net.app",
	"Epic Games Launcher.app",
	"Riot Client.app",
	"GOG Galaxy.app",
	"Heroic.app",
	"Ubisoft Connect.app",
];

/// The category every Mac game declares, and the prefix and suffix of its subcategories
/// (`public.app-category.action-games` and the others: Apple's `LSApplicationCategoryType`).
const GAMES: &str = "public.app-category.games";
const CATEGORY: &str = "public.app-category.";
const GAMES_SUFFIX: &str = "-games";

/// The stamp each line of Steam's process log starts with, `[YYYY-MM-DD HH:MM:SS]` in the computer's local time.
const STAMP: usize = 21;

/// Steam's process log's words: an app's process tracked, no longer tracked, and the app gone.
const ADDING: &str = " adding PID ";
const TRACKED: &str = " as a tracked process";
const NO_LONGER: &str = " no longer tracking PID ";

/// Whether a Windows image name is a launcher, a shell, a browser, a runtime or a service, never a game.
#[must_use]
pub fn denied_image(image: &str) -> bool {
	let lower = image.to_ascii_lowercase();
	DENIED_IMAGES.iter().any(|d| *d == lower) || DENIED_SUFFIXES.iter().any(|end| lower.ends_with(end))
}

/// Whether a Steam app is one that runs all day, never noticed as a game (`DENIED_APPS`).
#[must_use]
pub fn denied_app(app: u32) -> bool {
	DENIED_APPS.contains(&app)
}

/// Whether a Steam app id is the test app many games borrow (`BORROWED_APP`): it names no game.
#[must_use]
pub fn borrowed_app(app: u32) -> bool {
	app == BORROWED_APP
}

/// Whether a Windows image name is a refused game's.
#[must_use]
pub fn refused_image(image: &str) -> bool {
	REFUSED_IMAGES.iter().any(|r| r.eq_ignore_ascii_case(image))
}

/// Whether a Mac path is inside a refused game's bundle.
#[must_use]
pub fn refused_path(path: &std::path::Path) -> bool {
	path.components()
		.filter_map(|c| c.as_os_str().to_str())
		.any(|c| REFUSED_BUNDLES.iter().any(|r| r.eq_ignore_ascii_case(c)))
}

/// Whether a Mac app bundle's name is a launcher's.
#[must_use]
pub fn denied_bundle(bundle: &str) -> bool {
	DENIED_BUNDLES.iter().any(|d| d.eq_ignore_ascii_case(bundle))
}

/// Whether an `LSApplicationCategoryType` is the games category or one of its subcategories.
#[must_use]
pub fn is_game_category(category: &str) -> bool {
	category == GAMES || (category.starts_with(CATEGORY) && category.ends_with(GAMES_SUFFIX))
}

/// A Windows path's file name in lower case (Known Games List's `MatchedExeFullPath`, a launcher's launch executable),
/// whichever separator it uses.
#[must_use]
pub fn image_of(path: &str) -> Option<String> {
	let name = path.trim().trim_matches('"').rsplit(['\\', '/']).next()?;
	(!name.is_empty()).then(|| name.to_lowercase())
}

/// The decimal number at the start of `text`, and the text after it.
fn number(text: &str) -> Option<(u32, &str)> {
	let end = text.find(|c: char| !c.is_ascii_digit()).unwrap_or(text.len());
	let (digits, rest) = text.split_at(end);
	Some((digits.parse().ok()?, rest))
}

/// The number right after the first `word` in `line`, and the text after it.
fn after<'a>(line: &'a str, word: &str) -> Option<(u32, &'a str)> {
	let at = line.find(word)?;
	number(line.get(at + word.len()..)?)
}

/// "`AppID` 400 adding PID 1014178 as a tracked process": the app and the process. The line goes on with the game's
/// command line, which holds the user's folder and is never read past "as a tracked process".
fn adding(line: &str) -> Option<(u32, u32)> {
	let (app, rest) = after(line, "AppID ")?;
	let (pid, rest) = number(rest.strip_prefix(ADDING)?)?;
	rest.starts_with(TRACKED).then_some((app, pid))
}

/// "`AppID` 400 no longer tracking PID 1014178, exit code 0": the process.
fn no_longer(line: &str) -> Option<u32> {
	let (_, rest) = after(line, "AppID ")?;
	number(rest.strip_prefix(NO_LONGER)?).map(|(pid, _)| pid)
}

/// "Game 400 going away" or "Remove 400 from running list": the app.
fn gone(line: &str) -> Option<u32> {
	let going = after(line, "Game ").filter(|(_, rest)| rest.starts_with(" going away")).map(|(app, _)| app);
	going.or_else(|| {
		after(line, "Remove ").filter(|(_, rest)| rest.starts_with(" from running list")).map(|(app, _)| app)
	})
}

/// Whether a line of Steam's process log was stamped before `boot` (`YYYY-MM-DD HH:MM:SS`, local time): a process
/// id it names belonged to a run of Windows before this one. A line without a stamp is kept.
fn before(line: &str, boot: &str) -> bool {
	let stamp = line.get(1..STAMP - 1).filter(|_| line.starts_with('[') && line.get(STAMP - 1..STAMP) == Some("]"));
	stamp.is_some_and(|stamp| stamp < boot)
}

/// The processes Steam tracks now, by process id, each with its app id, as the log's lines in their order tell it;
/// the lines stamped before `boot` (this start of the computer, `YYYY-MM-DD HH:MM:SS` in local time, when known) are
/// dropped, since a process id they name may now be another program's.
#[must_use]
pub fn steam_tracked(log: &str, boot: Option<&str>) -> HashMap<u32, u32> {
	let mut tracked = HashMap::new();
	for line in log.lines().filter(|l| boot.is_none_or(|boot| !before(l, boot))) {
		if let Some((app, pid)) = adding(line) {
			tracked.insert(pid, app);
		} else if let Some(pid) = no_longer(line) {
			tracked.remove(&pid);
		} else if let Some(app) = gone(line) {
			tracked.retain(|_, a| *a != app);
		}
	}
	tracked
}
