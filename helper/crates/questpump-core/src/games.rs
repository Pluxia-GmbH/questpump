//! The games on the computer whose end offers a pump (docs/plan.md slices 35 and 48; contract/links.json's
//! `computer` list): Counter-Strike 2 and Dota 2 through Valve's Game State Integration (`gsi.rs`), Rocket League
//! through Psyonix's Stats API (`rocket.rs`), slice 48's fed games through their own feeds (`feeds.rs`, `Fed`),
//! League of Legends and Teamfight Tactics and any program the player picks or the detector finds by their process
//! alone. `Watcher` turns their feeds into finishes, one per match: a match's end only after an in-match phase this
//! helper saw (CS2 `live` or `intermission`, Dota 2 `GAME_IN_PROGRESS`, Rocket League `RoundStarted`), and a process's
//! close at the first look it is gone once it ran its floor from its first start (League `match_min_league_s`, every
//! other game `match_min_s`; slice 48), unless its feed ended a match in that run (slice 56: every named game is heard
//! at its close too, so one whose feed tells nothing, not set up or missing a setting, still offers); a start again
//! within `match_relaunch_s` of that close (a reconnect, a crash started again) continues the run and its match and
//! takes the close's offer back (`take_resumed`), so its later close may offer again. A fed game whose process closes
//! while its match is armed ends that match at the close, whatever its floor (`Finish::at_close`, held as every close
//! is). Each finish carries where its match began by the helper's own clock (`Finish::began_at`, slice 57: the match's
//! minutes size its pump), and a feed's match that ended under `match_min_s` from there offers nothing, its run's match
//! going on (the floor a close has: Elite Dangerous' missions handed in together offer once); since slice 59 it also
//! carries the close of its game's run before (`Finish::previous_close`), from before which League's lead counts no
//! minutes (`offers::Tunables::league_lead_s`; League's floor still judges its game process alone). A quit keeps the
//! watcher's runs and matches (`Watcher::keep`), and a start within `match_relaunch_s` of it goes on with them
//! (`Watcher::restore`), so a match a reinstall, a restart or Quit and reopen split still offers from its first start
//! (an update waits while a watched game runs). Only the end passes on, and our own clock's times: never a score, a
//! death, a loss, a match id or a game's own clock.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;
use std::sync::LazyLock;

use serde::{Deserialize, Serialize};

use crate::gsi::Gsi;

/// The kind of contract/kinds.json every game's offer carries: a match's end and any game's close alike, a found or
/// added program's too (2026-10-28, Rapha: "it should directly open and start and not let me select stuff"; a
/// program's close offered none until then, and its Start opened the Start a pump window), so every Start from an
/// offer opens the pump at Ready at the offer's size.
pub const MATCH: &str = "match";
/// The longest `playing` lasts without a break: past it the Android phone's hold no longer counts the match as word
/// from the helper, so a game left open in a match all evening never keeps a foreground service up. The sitting clock
/// takes the same bound (`sitting::Sitting::look`): a watched game counts as input only while the system's own idle
/// time is under it, so a game left in front overnight ends as a pause.
pub const PLAYING_CAP_MS: i64 = 3 * 3_600_000;

/// Steam's app ids (the `appmanifest_<appid>.acf` that names each game's folder; each checked against Steam's store
/// `appdetails` on 2026-10-20).
const CS2_APP: u32 = 730;
const DOTA_APP: u32 = 570;
const ROCKET_APP: u32 = 252_950;
const ARENA_APP: u32 = 2_141_910;
const F1_APP: u32 = 3_059_520;
const FORZA_APP: u32 = 2_440_510;
const IRACING_APP: u32 = 266_410;
const ACC_APP: u32 = 805_550;
const ELITE_APP: u32 = 359_320;
/// The games added on 2026-10-28 (the games compatibility research), checked against the store's `appdetails` that
/// day: Slay the Spire 2, Assetto Corsa (the first), Guild Wars 2.
const SPIRE_APP: u32 = 2_868_840;
const AC_APP: u32 = 244_210;
const GUILD_WARS_APP: u32 = 1_284_210;
/// Mechabellum (Game River; checked against Steam's store `appdetails` on 2026-10-28).
const MECHABELLUM_APP: u32 = 669_330;
/// The games added on 2026-10-29 (the close-only games' research), checked against the store's `appdetails` that day:
/// Path of Exile and Path of Exile 2 (one named game, whose builds share their programs' names), Warframe, World of
/// Warships, Rainbow Six Siege, Escape from Tarkov, Deadlock, Team Fortress 2. Fortnite is the Epic Games Store's alone.
const POE_APPS: [u32; 2] = [238_960, 2_694_490];
const WARFRAME_APP: u32 = 230_410;
const WARSHIPS_APP: u32 = 552_990;
const SIEGE_APP: u32 = 359_550;
const TARKOV_APP: u32 = 3_932_890;
const DEADLOCK_APP: u32 = 1_422_450;
const TF2_APP: u32 = 440;
/// The games added later on 2026-10-29, each app's name and launch options read from Steam's app info
/// (api.steamcmd.net) that day: NARAKA: BLADEPOINT, Fall Guys, Dead by Daylight, Hunt: Showdown 1896, PUBG:
/// BATTLEGROUNDS, Halo: The Master Chief Collection, Brawlhalla, rFactor 2, Le Mans Ultimate, Automobilista 2, Stardew
/// Valley. VALORANT is the Riot Client's alone, osu! (stable) its own installer's, Warcraft III the Battle.net app's.
const NARAKA_APP: u32 = 1_203_220;
const FALL_GUYS_APP: u32 = 1_097_150;
const DBD_APP: u32 = 381_210;
const HUNT_APP: u32 = 594_650;
const PUBG_APP: u32 = 578_080;
const HALO_APP: u32 = 976_730;
const BRAWLHALLA_APP: u32 = 291_550;
const RFACTOR_APP: u32 = 365_960;
const LE_MANS_APP: u32 = 2_399_420;
const AMS2_APP: u32 = 1_066_890;
const STARDEW_APP: u32 = 413_150;
/// ELDEN RING and ELDEN RING NIGHTREIGN (2026-10-29, their boss banners read off the game's window), each app's name
/// and launch options read from Steam's app info that day (both start Easy Anti-Cheat's `start_protected_game.exe` in
/// their `Game` folder, which runs the game).
const ELDEN_RING_APP: u32 = 1_245_620;
const NIGHTREIGN_APP: u32 = 2_622_380;

/// The ports a fed game's loopback feed takes unless the player set another (its window's Port): Legends of Runeterra's
/// Game Client API (Riot: "port `21337` by default", which its settings may change), F1 25's UDP telemetry (the
/// default its `hardware_settings_config.xml` holds) and Forza Motorsport's Data Out, which names none (QuestPump's
/// own, outside the 5200-5300 Forza's support page keeps for the game's own socket).
const RUNETERRA_PORT: u16 = 21_337;
const F1_PORT: u16 = 20_777;
const FORZA_PORT: u16 = 5_555;

/// The Riot Client's Windows images (its services, its window and the window's renderer) and its Mac bundle: the
/// launcher League of Legends, Legends of Runeterra and VALORANT start from, one of each game's own programs.
const RIOT_CLIENT: [&str; 3] = ["RiotClientServices.exe", "RiotClientUx.exe", "RiotClientUxRender.exe"];
const RIOT_CLIENT_APP: &str = "Riot Client.app";

/// One game of contract/links.json's `computer` list: its key (the `link` a `match_end` carries), the name an offer
/// shows (none for a picked program: the program's own), the systems it runs on (`windows`, `macos`), how the helper
/// hears it (`gsi`, `stats`, `process`, and since slice 48 `client`, `log`, `udp`, `shared`, since 2026-10-28 `files`,
/// since 2026-10-29 `screen`), whether its matches load the hands and arms (`aim`, a shooter's: slice 48's arms rest
/// while it still runs), whether its feed tells a
/// checkpoint rather than a match (`checkpoint`, 2026-10-28: a mission handed in, whose finish waits
/// `match_checkpoint_min` from the last).
#[derive(Deserialize, Debug, Clone)]
pub struct ComputerLink {
	pub key: String,
	#[serde(default)]
	pub name: Option<String>,
	pub systems: Vec<String>,
	pub trigger: String,
	#[serde(default)]
	pub aim: bool,
	#[serde(default)]
	pub checkpoint: bool,
}

#[derive(Deserialize)]
struct Links {
	computer: Vec<ComputerLink>,
}

/// The file is built in (`tools/copytext.py --check` holds its shape); its list builds only when it reads into these
/// types and names every named game with its name.
static LINKS: LazyLock<Option<Vec<ComputerLink>>> = LazyLock::new(|| {
	let links: Links = serde_json::from_str(include_str!("../../../../contract/links.json")).ok()?;
	let named = ALL.iter().all(|w| links.computer.iter().any(|l| l.key == w.link() && l.name.is_some()));
	named.then_some(links.computer)
});

/// contract/links.json's `computer` list, in its own order; none when it did not build in (`make check` keeps it
/// readable), and then no game is watched.
#[must_use]
pub fn computer_links() -> Option<&'static [ComputerLink]> {
	LINKS.as_deref()
}

/// A game the helper watches.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Watched {
	CounterStrike,
	Dota,
	League,
	RocketLeague,
	/// Slice 48's fed games, each heard by its own feed (`feeds.rs`).
	StarCraft,
	MtgArena,
	Hearthstone,
	Runeterra,
	FormulaOne,
	Forza,
	IRacing,
	AssettoCorsa,
	Elite,
	/// The games added on 2026-10-28, each heard by its own feed (`feeds.rs`): Slay the Spire 2's run history,
	/// Assetto Corsa's (the first) shared memory, Guild Wars 2's `MumbleLink` in a player-versus-player match.
	SlayTheSpire,
	AssettoCorsaOne,
	GuildWars,
	/// Mechabellum, heard by its own Unity `Player.log` (2026-10-28).
	Mechabellum,
	/// The games added on 2026-10-29, each heard by its own feed (`feeds.rs`): Path of Exile's and Path of Exile 2's
	/// `Client.txt` (one game: both builds run under the same programs' names), Warframe's `EE.log`, World of Warships'
	/// battle file and replays, Rainbow Six Siege's match replays, Escape from Tarkov's logs, Deadlock's and Team
	/// Fortress 2's console log (with `-condebug`) and Fortnite's own log.
	PathOfExile,
	Warframe,
	Warships,
	Siege,
	Tarkov,
	Deadlock,
	Fortnite,
	TeamFortress,
	/// The games added later on 2026-10-29, each heard by its own feed (`feeds.rs`): NARAKA: BLADEPOINT's and Fall
	/// Guys' Unity `Player.log`, Dead by Daylight's and VALORANT's Unreal logs, Hunt: Showdown 1896's `Game.log`, PUBG's
	/// replay folders, Halo: The Master Chief Collection's carnage reports, Brawlhalla's replays, rFactor 2's and Le Mans
	/// Ultimate's results files, Automobilista 2's documented shared memory, osu!'s replays, Stardew Valley's saves and
	/// Warcraft III's replays (the files by their times alone).
	Naraka,
	FallGuys,
	DeadByDaylight,
	Hunt,
	Valorant,
	Pubg,
	Halo,
	Brawlhalla,
	RFactor,
	LeMans,
	Automobilista,
	Osu,
	Stardew,
	Warcraft,
	/// ELDEN RING and ELDEN RING NIGHTREIGN (2026-10-29), heard by the boss banners read off the game's own window
	/// (`feeds::banner`, Windows): neither writes a log or a file at a boss's death.
	EldenRing,
	Nightreign,
	/// A program the player picked or the detector found: its key (on Windows its image name in lower case, on macOS
	/// its bundle's or its executable's path; `steam:<app>` for a Steam game) and the name it was picked or found
	/// under.
	Program {
		key: String,
		name: String,
	},
}

/// Every named game, in links.json's order.
static ALL: [Watched; 41] = [
	Watched::CounterStrike,
	Watched::Dota,
	Watched::League,
	Watched::RocketLeague,
	Watched::StarCraft,
	Watched::MtgArena,
	Watched::Hearthstone,
	Watched::Runeterra,
	Watched::FormulaOne,
	Watched::Forza,
	Watched::IRacing,
	Watched::AssettoCorsa,
	Watched::Elite,
	Watched::SlayTheSpire,
	Watched::AssettoCorsaOne,
	Watched::GuildWars,
	Watched::Mechabellum,
	Watched::PathOfExile,
	Watched::Warframe,
	Watched::Warships,
	Watched::Siege,
	Watched::Tarkov,
	Watched::Deadlock,
	Watched::Fortnite,
	Watched::TeamFortress,
	Watched::Naraka,
	Watched::FallGuys,
	Watched::DeadByDaylight,
	Watched::Hunt,
	Watched::Valorant,
	Watched::Pubg,
	Watched::Halo,
	Watched::Brawlhalla,
	Watched::RFactor,
	Watched::LeMans,
	Watched::Automobilista,
	Watched::Osu,
	Watched::Stardew,
	Watched::Warcraft,
	Watched::EldenRing,
	Watched::Nightreign,
];

impl Watched {
	/// The named games, in links.json's order.
	#[must_use]
	pub fn named() -> &'static [Self] {
		&ALL
	}

	/// A named game by its links.json key.
	#[must_use]
	pub fn from_key(key: &str) -> Option<Self> {
		Self::named().iter().find(|w| w.link() == key).cloned()
	}

	/// Its links.json key, the `link` of its offer.
	#[must_use]
	pub fn link(&self) -> &'static str {
		match self {
			Self::CounterStrike => "counterstrike",
			Self::Dota => "dota",
			Self::League => "league",
			Self::RocketLeague => "rocketleague",
			Self::StarCraft => "starcraft",
			Self::MtgArena => "mtgarena",
			Self::Hearthstone => "hearthstone",
			Self::Runeterra => "runeterra",
			Self::FormulaOne => "formulaone",
			Self::Forza => "forza",
			Self::IRacing => "iracing",
			Self::AssettoCorsa => "assettocorsa",
			Self::Elite => "elite",
			Self::SlayTheSpire => "slaythespire",
			Self::AssettoCorsaOne => "assettocorsaone",
			Self::GuildWars => "guildwars",
			Self::Mechabellum => "mechabellum",
			Self::PathOfExile => "pathofexile",
			Self::Warframe => "warframe",
			Self::Warships => "warships",
			Self::Siege => "siege",
			Self::Tarkov => "tarkov",
			Self::Deadlock => "deadlock",
			Self::Fortnite => "fortnite",
			Self::TeamFortress => "teamfortress",
			Self::Naraka => "naraka",
			Self::FallGuys => "fallguys",
			Self::DeadByDaylight => "deadbydaylight",
			Self::Hunt => "hunt",
			Self::Valorant => "valorant",
			Self::Pubg => "pubg",
			Self::Halo => "halo",
			Self::Brawlhalla => "brawlhalla",
			Self::RFactor => "rfactor",
			Self::LeMans => "lemans",
			Self::Automobilista => "automobilista",
			Self::Osu => "osu",
			Self::Stardew => "stardew",
			Self::Warcraft => "warcraft",
			Self::EldenRing => "eldenring",
			Self::Nightreign => "nightreign",
			Self::Program { .. } => "program",
		}
	}

	/// What tells it from every other watched game: a named game's key, a program's own key.
	#[must_use]
	pub fn id(&self) -> &str {
		match self {
			Self::Program { key, .. } => key,
			named => named.link(),
		}
	}

	/// The name its offer shows: links.json's for a named game, the program's own. The list builds in only with every
	/// named game's name, and without it no game is watched, so the key in its place never shows.
	#[must_use]
	pub fn name(&self) -> &str {
		match self {
			Self::Program { name, .. } => name,
			named => computer_links()
				.and_then(|links| links.iter().find(|l| l.key == named.link()))
				.and_then(|l| l.name.as_deref())
				.unwrap_or_else(|| named.link()),
		}
	}

	/// Whether contract/links.json marks it `aim` (a shooter: a pump while it runs rests the arms); never a program.
	#[must_use]
	pub fn aim(&self) -> bool {
		computer_links().is_some_and(|links| links.iter().any(|l| l.key == self.link() && l.aim))
	}

	/// Whether contract/links.json marks it `checkpoint` (2026-10-28: its feed tells a mission handed in, whose finish
	/// waits `match_checkpoint_min` from the last; never a program).
	#[must_use]
	pub fn checkpoint(&self) -> bool {
		computer_links().is_some_and(|links| links.iter().any(|l| l.key == self.link() && l.checkpoint))
	}

	/// Its Steam apps, for the games the helper finds through Steam's libraries: one app a game, but Path of Exile's
	/// two (Path of Exile and Path of Exile 2, 2026-10-29), its own first; none for a game Steam does not sell.
	#[must_use]
	pub fn steam_apps(&self) -> &'static [u32] {
		match self {
			Self::CounterStrike => &[CS2_APP],
			Self::Dota => &[DOTA_APP],
			Self::RocketLeague => &[ROCKET_APP],
			Self::MtgArena => &[ARENA_APP],
			Self::FormulaOne => &[F1_APP],
			Self::Forza => &[FORZA_APP],
			Self::IRacing => &[IRACING_APP],
			Self::AssettoCorsa => &[ACC_APP],
			Self::Elite => &[ELITE_APP],
			Self::SlayTheSpire => &[SPIRE_APP],
			Self::AssettoCorsaOne => &[AC_APP],
			Self::GuildWars => &[GUILD_WARS_APP],
			Self::Mechabellum => &[MECHABELLUM_APP],
			Self::PathOfExile => &POE_APPS,
			Self::Warframe => &[WARFRAME_APP],
			Self::Warships => &[WARSHIPS_APP],
			Self::Siege => &[SIEGE_APP],
			Self::Tarkov => &[TARKOV_APP],
			Self::Deadlock => &[DEADLOCK_APP],
			Self::TeamFortress => &[TF2_APP],
			Self::Naraka => &[NARAKA_APP],
			Self::FallGuys => &[FALL_GUYS_APP],
			Self::DeadByDaylight => &[DBD_APP],
			Self::Hunt => &[HUNT_APP],
			Self::Pubg => &[PUBG_APP],
			Self::Halo => &[HALO_APP],
			Self::Brawlhalla => &[BRAWLHALLA_APP],
			Self::RFactor => &[RFACTOR_APP],
			Self::LeMans => &[LE_MANS_APP],
			Self::Automobilista => &[AMS2_APP],
			Self::Stardew => &[STARDEW_APP],
			Self::EldenRing => &[ELDEN_RING_APP],
			Self::Nightreign => &[NIGHTREIGN_APP],
			Self::League
			| Self::StarCraft
			| Self::Hearthstone
			| Self::Runeterra
			| Self::Fortnite
			| Self::Valorant
			| Self::Osu
			| Self::Warcraft
			| Self::Program { .. } => &[],
		}
	}

	/// Its game process's image names on Windows, compared in any case (Rocket League's Steam launch starts
	/// `RocketLeague_EAC.exe` first, which then runs the game itself; Forza Motorsport's Steam and Microsoft Store
	/// builds name theirs apart; Assetto Corsa's sim, which its launcher starts for each session, in its 64-bit and
	/// 32-bit builds; Guild Wars 2's 64-bit client, the GW2 wiki's Command line page; since 2026-10-29 Path of Exile's
	/// standalone, Steam and Epic builds, both games' alike, and Team Fortress 2's 64-bit and 32-bit builds, as Steam's
	/// launch options name them; Deadlock's `deadlock.exe`, `project8.exe` before Valve renamed it); later that day the
	/// game each launcher or anti-cheat bootstrap runs (Dead by Daylight's, PUBG's, Halo: The Master Chief Collection's,
	/// rFactor 2's, VALORANT's, Fall Guys'), Automobilista 2's own and its AVX build, which it switches to where the
	/// processor has AVX, and Stardew Valley's with SMAPI's, which runs the game in its own process; NARAKA's, Hunt's and
	/// Le Mans Ultimate's are unverified (Steam hides or names only a launcher), and ELDEN RING's and NIGHTREIGN's,
	/// which Easy Anti-Cheat's `start_protected_game.exe` runs from the game's `Game` folder, come from players' guides
	/// alone; none for a program, whose key is its image.
	#[must_use]
	pub fn windows_images(&self) -> &'static [&'static str] {
		match self {
			Self::CounterStrike => &["cs2.exe"],
			Self::Dota => &["dota2.exe"],
			Self::League => &["League of Legends.exe"],
			Self::RocketLeague => &["RocketLeague.exe"],
			Self::StarCraft => &["SC2_x64.exe"],
			Self::MtgArena => &["MTGA.exe"],
			Self::Hearthstone => &["Hearthstone.exe"],
			Self::Runeterra => &["LoR.exe"],
			Self::FormulaOne => &["F1_25.exe"],
			Self::Forza => &["forza_steamworks_release_final.exe", "forza_gaming.desktop.x64_release_final.exe"],
			Self::IRacing => &["iRacingSim64DX11.exe"],
			Self::AssettoCorsa => &["AC2-Win64-Shipping.exe"],
			Self::Elite => &["EliteDangerous64.exe"],
			Self::SlayTheSpire => &["SlayTheSpire2.exe"],
			Self::AssettoCorsaOne => &["acs.exe", "acs_x86.exe"],
			Self::GuildWars => &["Gw2-64.exe"],
			Self::Mechabellum => &["Mechabellum.exe"],
			Self::PathOfExile => &[
				"PathOfExile.exe",
				"PathOfExile_x64.exe",
				"PathOfExileSteam.exe",
				"PathOfExile_x64Steam.exe",
				"PathOfExileEGS.exe",
				"PathOfExile_x64EGS.exe",
			],
			Self::Warframe => &["Warframe.x64.exe"],
			Self::Warships => &["WorldOfWarships64.exe", "WorldOfWarships32.exe"],
			Self::Siege => &["RainbowSix.exe", "RainbowSix_Vulkan.exe"],
			Self::Tarkov => &["EscapeFromTarkov.exe"],
			Self::Deadlock => &["deadlock.exe", "project8.exe"],
			Self::Fortnite => &["FortniteClient-Win64-Shipping.exe"],
			Self::TeamFortress => &["tf_win64.exe", "tf.exe"],
			Self::Naraka => &["NarakaBladepoint.exe"],
			Self::FallGuys => &["FallGuys_client_game.exe"],
			Self::DeadByDaylight => &["DeadByDaylight-Win64-Shipping.exe"],
			Self::Hunt => &["HuntGame.exe"],
			Self::Valorant => &["VALORANT-Win64-Shipping.exe"],
			Self::Pubg => &["TslGame.exe"],
			Self::Halo => &["MCC-Win64-Shipping.exe"],
			Self::Brawlhalla => &["Brawlhalla.exe"],
			Self::RFactor => &["rFactor2.exe"],
			Self::LeMans => &["Le Mans Ultimate.exe"],
			Self::Automobilista => &["AMS2AVX.exe", "AMS2.exe"],
			Self::Osu => &["osu!.exe"],
			Self::Stardew => &["Stardew Valley.exe", "StardewModdingAPI.exe"],
			Self::Warcraft => &["Warcraft III.exe"],
			Self::EldenRing => &["eldenring.exe"],
			Self::Nightreign => &["nightreign.exe"],
			Self::Program { .. } => &[],
		}
	}

	/// Whether a Windows image name is one of this game's own programs that is not its game process, compared in any
	/// case: its launcher, its client or the anti-cheat bootstrap that starts it (an `_EAC` beside one of its images),
	/// which a detector never takes for another game and the picker shows as the game itself. League's client
	/// (`LeagueClient*`), the Riot Client that starts the Riot games (`RIOT_CLIENT`), iRacing's UI and service, Elite
	/// Dangerous' launcher, StarCraft II's switcher, MTG Arena's launcher, Assetto Corsa
	/// Competizione's starter and Assetto Corsa's launcher (`AssettoCorsa.exe`, whose sessions run in `acs.exe`); since
	/// 2026-10-29 World of Warships' starter (`WorldOfWarships.exe`, Steam's launch option, which runs the client under
	/// `bin`), Escape from Tarkov's launcher and Fortnite's launcher and its Easy Anti-Cheat bootstrap; later that day the
	/// programs Steam's launch options start (Fall Guys' `FallGuys_client.exe`, Dead by Daylight's `DeadByDaylight.exe`,
	/// Hunt's `hunt.exe`, PUBG's `ExecPubg.exe`, Halo: The Master Chief Collection's `mcclauncher.exe`, rFactor 2's
	/// `Launch rFactor.exe`), VALORANT's `VALORANT.exe`, which starts its game, and the Riot Client, and Warcraft III's
	/// launcher.
	#[must_use]
	pub fn is_family_image(&self, image: &str) -> bool {
		let lower = image.to_ascii_lowercase();
		let own = |names: &[&str]| names.iter().any(|n| n.eq_ignore_ascii_case(image));
		let bootstrap = self.windows_images().iter().any(|i| {
			let stem = i.strip_suffix(".exe").unwrap_or(i);
			lower == format!("{}_eac.exe", stem.to_ascii_lowercase())
		});
		bootstrap
			|| match self {
				Self::League => {
					own(&RIOT_CLIENT)
						|| (lower.starts_with("leagueclient")
							&& Path::new(&lower).extension().is_some_and(|e| e == "exe"))
				}
				Self::Runeterra => own(&RIOT_CLIENT),
				Self::IRacing => own(&["iRacingUI.exe"]) || own(&IRACING_SERVICES),
				Self::Elite => own(&["EDLaunch.exe"]),
				Self::StarCraft => own(&["SC2Switcher.exe", "SC2Switcher_x64.exe"]),
				Self::MtgArena => own(&["MTGALauncher.exe"]),
				Self::AssettoCorsa => own(&["AC2.exe"]),
				Self::AssettoCorsaOne => own(&["AssettoCorsa.exe"]),
				Self::Warships => own(&["WorldOfWarships.exe"]),
				Self::Tarkov => own(&["BsgLauncher.exe"]),
				Self::Fortnite => own(&["FortniteLauncher.exe", "FortniteClient-Win64-Shipping_EAC_EOS.exe"]),
				Self::FallGuys => own(&["FallGuys_client.exe"]),
				Self::DeadByDaylight => own(&["DeadByDaylight.exe"]),
				Self::Hunt => own(&["hunt.exe"]),
				Self::Valorant => own(&["VALORANT.exe"]) || own(&RIOT_CLIENT),
				Self::Pubg => own(&["ExecPubg.exe"]),
				Self::Halo => own(&["mcclauncher.exe"]),
				Self::RFactor => own(&["Launch rFactor.exe"]),
				Self::Warcraft => own(&["Warcraft III Launcher.exe"]),
				_ => false,
			}
	}

	/// Whether a macOS image path is one of this game's own programs, its game or its launcher, by its path alone:
	/// anything inside `League of Legends.app` (the client and the game) or `Riot Client.app`, inside StarCraft II's
	/// folder, MTG Arena's `MTGA.app`, Hearthstone's folder, Path of Exile's `Path of Exile.app` and
	/// `PathOfExileClient.app` or Warcraft III's folder (2026-10-29), which a detector never takes for another game and
	/// the picker shows as the game itself. A Steam game's folder is the desktop app's to tell (its manifest).
	#[must_use]
	pub fn is_mac_family(&self, path: &Path) -> bool {
		let within = |part: &str| {
			path.components().any(|c| c.as_os_str().to_str().is_some_and(|n| n.eq_ignore_ascii_case(part)))
		};
		match self {
			Self::League => within("League of Legends.app") || within(RIOT_CLIENT_APP),
			Self::StarCraft => within("StarCraft II"),
			Self::MtgArena => within("MTGA.app"),
			Self::Hearthstone => within("Hearthstone"),
			Self::PathOfExile => within("Path of Exile.app") || within("PathOfExileClient.app"),
			Self::Warcraft => within("Warcraft III"),
			_ => false,
		}
	}

	/// Whether a macOS image path is this game's process, by its path alone: League's game bundle (`is_league_path`),
	/// StarCraft II's `SC2.app` under its `Versions`, MTG Arena's `MTGA.app`, Hearthstone's `Hearthstone.app`, Dota 2's
	/// binary under its Steam folder (`is_dota_path`), Mechabellum's app under its Steam folder (`is_mechabellum_path`,
	/// unverified on a Mac), Path of Exile's `Path of Exile.app` (its own launcher's copy in Application Support and
	/// Steam's, unverified on a Mac; Path of Exile 2 has no Mac build yet), since 2026-10-29 Stardew Valley's under its
	/// Steam folder (`in_steam_bundle`) and Warcraft III's `Warcraft III.app` (both unverified on a Mac); the others run
	/// on Windows alone.
	#[must_use]
	pub fn is_mac_image(&self, path: &Path) -> bool {
		match self {
			Self::League => is_league_path(path),
			Self::Dota => is_dota_path(path),
			Self::StarCraft => {
				bundle_image(path, "SC2.app", "SC2") && path.components().any(|c| c.as_os_str() == "StarCraft II")
			}
			Self::MtgArena => bundle_image(path, "MTGA.app", "MTGA"),
			Self::Hearthstone => bundle_image(path, "Hearthstone.app", "Hearthstone"),
			Self::Mechabellum => is_mechabellum_path(path),
			Self::PathOfExile => bundle_image(path, "Path of Exile.app", "Path of Exile"),
			Self::Stardew => in_steam_bundle(path, "Stardew Valley"),
			Self::Warcraft => bundle_image(path, "Warcraft III.app", "Warcraft III"),
			_ => false,
		}
	}

	/// The loopback port its feed takes unless the player set another: Legends of Runeterra's API, F1 25's and Forza
	/// Motorsport's telemetry; none for a game whose feed has no port the player sets.
	#[must_use]
	pub fn default_port(&self) -> Option<u16> {
		match self {
			Self::Runeterra => Some(RUNETERRA_PORT),
			Self::FormulaOne => Some(F1_PORT),
			Self::Forza => Some(FORZA_PORT),
			_ => None,
		}
	}

	/// Whether it is a Riot game (League of Legends and TFT, Legends of Runeterra, since 2026-10-29 VALORANT): Riot's
	/// line shows wherever one is listed (slice 56). Since 2026-10-28 it is watched once found, as every named game is.
	#[must_use]
	pub fn riot(&self) -> bool {
		matches!(self, Self::League | Self::Runeterra | Self::Valorant)
	}

	/// Whether its feed needs a file of QuestPump's in the game's own files (CS2's and Dota 2's Game State Integration
	/// cfg, Rocket League's `TAStatsAPI.ini`, Hearthstone's `log.config` section), which goes in only once the player
	/// asks for it ("Ask after each match", slice 56); until then it is heard at its close alone.
	#[must_use]
	pub fn writes_file(&self) -> bool {
		matches!(self, Self::CounterStrike | Self::Dota | Self::RocketLeague | Self::Hearthstone)
	}
}

/// Whether a macOS image path is the executable `exe` of an app bundle named `bundle`
/// (`<bundle>/Contents/MacOS/<exe>`).
fn bundle_image(path: &Path, bundle: &str, exe: &str) -> bool {
	let names: Vec<&str> =
		path.ancestors().take(4).map(|p| p.file_name().and_then(|n| n.to_str()).unwrap_or_default()).collect();
	matches!(names.as_slice(), [file, "MacOS", "Contents", app] if *file == exe && app.eq_ignore_ascii_case(bundle))
}

/// iRacing's Windows services, its own programs that run whether or not the player opened the game: they tell no
/// running game on Pump home (the desktop app's `Games::open`).
pub const IRACING_SERVICES: [&str; 2] = ["iRacingService.exe", "iRacingService64.exe"];

/// Whether a process is the Riot Client, by its Windows image (`RIOT_CLIENT`) or its macOS path (anything inside
/// `Riot Client.app`): the launcher the Riot games start from, which tells none apart, so it shows none of them open
/// (the desktop app's `Games::opens`).
#[must_use]
pub fn is_riot_client(image: &str, path: Option<&Path>) -> bool {
	RIOT_CLIENT.iter().any(|name| name.eq_ignore_ascii_case(image))
		|| path.is_some_and(|path| {
			path.components().any(|c| c.as_os_str().to_str().is_some_and(|n| n.eq_ignore_ascii_case(RIOT_CLIENT_APP)))
		})
}

/// Whether a macOS image path is League of Legends' game process, which runs one match: the bundle inside the
/// launcher's `League of Legends.app/Contents/LoL/Game/` (the client beside it, `LeagueClient`, is no match).
#[must_use]
pub fn is_league_path(path: &Path) -> bool {
	let parts: Vec<&str> = path.components().filter_map(|c| c.as_os_str().to_str()).collect();
	let game = parts.windows(2).any(|w| w[0] == "LoL" && w[1] == "Game");
	let file = path.file_name().and_then(|f| f.to_str()).unwrap_or_default();
	game && file.replace(' ', "").eq_ignore_ascii_case("LeagueofLegends")
}

/// Whether a macOS image path is Dota 2's, by its path alone (slice 56: no Steam library file read, so a library on an
/// external disk asks nothing): a binary under `game/bin` of its folder in a Steam library,
/// `steamapps/common/dota 2 beta/game/bin/…`.
#[must_use]
pub fn is_dota_path(path: &Path) -> bool {
	const FOLDERS: [&str; 5] = ["steamapps", "common", "dota 2 beta", "game", "bin"];
	let parts: Vec<&str> = path.components().filter_map(|c| c.as_os_str().to_str()).collect();
	parts
		.windows(FOLDERS.len() + 1)
		.any(|w| w.iter().zip(FOLDERS).all(|(part, folder)| part.eq_ignore_ascii_case(folder)))
}

/// Whether a macOS image path is Mechabellum's, by its path alone, as `is_dota_path` tells Dota 2's: an executable in
/// an app bundle's `Contents/MacOS` under its folder in a Steam library, `steamapps/common/Mechabellum/…`. Unity's
/// layout for a Mac build, unverified until the game runs on a Mac (2026-10-28).
#[must_use]
pub fn is_mechabellum_path(path: &Path) -> bool {
	in_steam_bundle(path, "Mechabellum")
}

/// Whether a macOS image path is an executable in a `Contents/MacOS` under the Steam library folder `folder`,
/// `steamapps/common/<folder>/…`: Mechabellum's (`is_mechabellum_path`) and Stardew Valley's, whose folder is itself
/// the bundle (Steam's launch option `Contents/MacOS/StardewValley`, 2026-10-29).
fn in_steam_bundle(path: &Path, folder: &str) -> bool {
	let folders = ["steamapps", "common", folder];
	let parts: Vec<&str> = path.components().filter_map(|c| c.as_os_str().to_str()).collect();
	let under =
		parts.windows(folders.len()).any(|w| w.iter().zip(folders).all(|(part, f)| part.eq_ignore_ascii_case(f)));
	let names: Vec<&str> =
		path.ancestors().skip(1).take(2).map(|p| p.file_name().and_then(|n| n.to_str()).unwrap_or_default()).collect();
	under && names == ["MacOS", "Contents"]
}

/// What the watcher hears, each at the helper's clock in unix milliseconds.
#[derive(Debug, Clone)]
pub enum Signal {
	/// An authenticated post's phase (`gsi::read`); none while no map is loaded.
	Gsi { game: Gsi, phase: Option<String>, at: i64 },
	/// Whether a watched game's process runs, every look (each 2 s).
	Process { watched: Watched, running: bool, at: i64 },
	/// A Stats API envelope's event.
	Rocket { event: String, at: i64 },
	/// The Stats API connection ended.
	RocketGone { at: i64 },
	/// A fed game's word (`feeds.rs`, slice 48).
	Fed { watched: Watched, state: Fed, at: i64 },
	/// A game's feed answered with no word of a match (a fed game's client API, log, journal, telemetry or shared
	/// memory read, Rocket League's Stats API connected): its run was told, so its process running is no longer its
	/// match (slice 56).
	Answered { watched: Watched, at: i64 },
}

/// Whether a fed game's match runs, the one thing its reader tells (slice 48: each game's answer, line, packet or page
/// read into this alone, never a name, a score or a result).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fed {
	/// A match runs that this helper may see end: it arms it, the first since an end its start (slice 57).
	Playing,
	/// The match ended: an armed one finishes, once.
	Ended,
	/// A match ended whose start no feed tells (Legends of Runeterra's count of games, an Elite Dangerous mission): it
	/// finishes at once.
	Completed,
}

/// A game's end that may offer a pump.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finish {
	pub watched: Watched,
	/// When its match began, unix ms, by the helper's own clock (slice 57: the match's minutes size its pump): the
	/// match's start its feed told in this run, else the later of the run's first start and its last finish in the run
	/// (`Run::from`: how long the game was open, Rapha, 2026-10-28: "in case it just cant know the match or play
	/// duration, then just take the time the game was open"), the run's first start being the first look that saw it
	/// running, so a game already running as the helper started counts from then, unless a quit within
	/// `match_relaunch_s` before kept its run (`Watcher::restore`), which goes on from its own; where no run is open (a
	/// post before its game's first look, a game whose process the looks do not find), the game's first word since its
	/// last finish (`Watcher::heard`), the first the helper knew of it, else its end, a match nothing measured, which
	/// offers at no minutes.
	pub began_at: i64,
	/// When it ended, unix ms: the phase's edge, or the process's last close.
	pub ended_at: i64,
	/// Whether it came at the game's close (any watched game's, a fed game's whose match ran as it closed among them):
	/// its offer is held for `match_relaunch_s`, which a start again takes back.
	pub at_close: bool,
	/// The close of its game's run before this one, unix ms (slice 59: League's lead counts no minutes from before it,
	/// `offers::Tunables::league_lead_s`), or the watcher's first look where it saw no run of the game end (the watcher
	/// keeps both in memory, `Watcher::last_close` and `Watcher::first_look`, and a quit hands them to a start within
	/// `match_relaunch_s`, `Watcher::keep`), so a restart between two matches never lets the lead reach back into the
	/// match before.
	pub previous_close: Option<i64>,
}

/// A fed game's match: whether one is on, and whether an in-match phase that can end it was seen.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
struct Feed {
	in_match: bool,
	armed: bool,
}

/// Where a run stands after its last close (`Run::close`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Close {
	/// No close of it offered, or what a start again continued has ended.
	#[default]
	Quiet,
	/// Its last close offered: a start again within `match_relaunch_s` takes that offer back.
	Offered,
	/// Started again within `match_relaunch_s` of a close that offered, that offer taken back: the first match start
	/// its feed tells after it is the match the close broke off (a crash and its reconnect, iRacing's test drive and
	/// its race), which goes on from `Run::from`; that start or a finish ends it.
	Rejoined,
}

/// A game's run: from its first start to its last close, while a relaunch may still continue it, where its current
/// match began (`from`), whether its last close offered or it was started again after one that did (`close`), whether
/// its feed told anything in it (a GSI post, a Stats API event or connection, a fed game's word or answer: its process
/// running is no longer its match then) and whether its feed ended a match in it (its close then offers nothing more).
/// It ends at a look `match_relaunch_s` after its last close, or at a start no look caught closed that long, and its
/// close is then its game's last (`Watcher::last_close`, slice 59): never a close a start again may still continue.
#[derive(Debug, Clone, Copy)]
struct Run {
	since: i64,
	/// Where the run's current match began, unix ms (slice 57): the run's first start, then each match start its feed
	/// tells (CS2's `warmup`, Dota 2's first state before `GAME_IN_PROGRESS`, Rocket League's `MatchInitialized`, a
	/// fed game's `Playing`) but the first after a start again (`Close::Rejoined`), and each finish its feed tells that
	/// offers (one under `match_min_s` moves it not). A close's finish moves it not: it ends the run, or a start again
	/// within `match_relaunch_s` takes its offer back and the run goes on as one match.
	from: i64,
	closed: Option<i64>,
	close: Close,
	told: bool,
	ended: bool,
}

impl Run {
	fn new(since: i64) -> Self {
		Self { since, from: since, closed: None, close: Close::Quiet, told: false, ended: false }
	}
}

/// What a quit keeps of the watcher for the next start (`Watcher::keep`; settings.json's `runs`), unix ms, each game by
/// `Watched::id`: every run, open or closed within `match_relaunch_s`, the feeds' matches (Counter-Strike 2's, Dota 2's
/// and Rocket League's, slice 48's fed games in a match, `Watcher::fed`, and the first words of those with no run,
/// `Watcher::heard`), each game's last ended run's close, the first look and since when a match ran. A start within
/// `match_relaunch_s` of the quit goes on from it (`Watcher::restore`; 2026-10-29: a League match a reinstall split two
/// seconds before its close offered nothing, the new process's run two seconds long).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeptRuns {
	/// When the quit came, unix ms.
	quit_at: i64,
	#[serde(default)]
	runs: Vec<KeptRun>,
	#[serde(default)]
	cs2: Feed,
	#[serde(default)]
	dota: Feed,
	#[serde(default)]
	rocket: Feed,
	#[serde(default)]
	replay: bool,
	#[serde(default)]
	fed: Vec<String>,
	#[serde(default)]
	heard: BTreeMap<String, i64>,
	#[serde(default)]
	last_close: BTreeMap<String, i64>,
	#[serde(default)]
	first_look: Option<i64>,
	#[serde(default)]
	playing_since: Option<i64>,
}

/// A run as the quit came (`Run`): its game, its first start, where its match began, its last close while a start
/// again may still continue it and where that close left it, whether its feed told anything in it and whether its
/// feed ended a match in it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct KeptRun {
	game: String,
	since: i64,
	from: i64,
	#[serde(default)]
	closed: Option<i64>,
	#[serde(default)]
	close: Close,
	#[serde(default)]
	told: bool,
	#[serde(default)]
	ended: bool,
}

/// The games' state machine, owned by the helper's loop.
pub struct Watcher {
	min_ms: i64,
	/// The least a checkpoint game's feed finish waits from where its match began (`match_checkpoint_min`).
	checkpoint_ms: i64,
	league_ms: i64,
	relaunch_ms: i64,
	cs2: Feed,
	dota: Feed,
	rocket: Feed,
	/// Rocket League showed a replay: its events say nothing of a match until it is destroyed.
	replay: bool,
	/// Slice 48's fed games in a match now, armed by their `Fed::Playing`.
	fed: HashSet<Watched>,
	/// Every watched game's run, from the first look its process ran (or as a quit kept it, `restore`).
	runs: HashMap<Watched, Run>,
	/// Each game's last ended run's close, unix ms, in this helper process and what a quit handed it (slice 59;
	/// `restore`): every finish of the game carries it (`Finish::previous_close`), the earliest League's lead counts
	/// from. A game turned off keeps it.
	last_close: HashMap<Watched, i64>,
	/// The first signal's time, unix ms (or the one a quit kept, `restore`): the earliest League's lead counts from
	/// where no run of it ended yet.
	first_look: Option<i64>,
	/// Each game's first feed word since its last finish while it has no run open, unix ms, a match start its feed told
	/// then in its place: what a finish with no run counts its match from (`Finish::began_at`). A run of the game
	/// starting drops it, and a finish moves it to its end.
	heard: HashMap<Watched, i64>,
	/// Since when a watched game is in a match without a break.
	playing_since: Option<i64>,
	/// The games started again after a close that offered, since `take_resumed`.
	resumed: Vec<Watched>,
}

impl Watcher {
	/// A watcher whose games must run `min_s` and League `league_min_s` before a close offers, whose checkpoint games'
	/// feed finishes wait `checkpoint_s` from the last, and whose games continue their run when started again within
	/// `relaunch_s` of a close (`match_min_s`, `match_min_league_s`, `match_checkpoint_min`, `match_relaunch_s`).
	#[must_use]
	pub fn new(min_s: i64, league_min_s: i64, checkpoint_s: i64, relaunch_s: i64) -> Self {
		Self {
			min_ms: min_s * 1_000,
			checkpoint_ms: checkpoint_s * 1_000,
			league_ms: league_min_s * 1_000,
			relaunch_ms: relaunch_s * 1_000,
			cs2: Feed::default(),
			dota: Feed::default(),
			rocket: Feed::default(),
			replay: false,
			fed: HashSet::new(),
			runs: HashMap::new(),
			last_close: HashMap::new(),
			first_look: None,
			heard: HashMap::new(),
			playing_since: None,
			resumed: Vec::new(),
		}
	}

	/// The least a game runs, from its first start to a close, for that close to offer (ms).
	fn floor_ms(&self, watched: &Watched) -> i64 {
		if *watched == Watched::League { self.league_ms } else { self.min_ms }
	}

	/// The least a feed's match runs, from where it began to its end, for that end to offer (ms): a checkpoint game's
	/// waits `checkpoint_ms` (a mission handed in counts from the game's start or the last that offered), every other
	/// game's `min_ms`.
	fn feed_floor_ms(&self, watched: &Watched) -> i64 {
		if watched.checkpoint() { self.checkpoint_ms } else { self.min_ms }
	}

	/// The games started again within `match_relaunch_s` of a close that offered, since the last call: their offer is
	/// taken back, and their run goes on from its first start, its match with it (`Close::Rejoined`). The desktop app
	/// takes them once every process of the look was stepped (`Games::tick`), so a resume is never also a new start.
	pub fn take_resumed(&mut self) -> Vec<Watched> {
		std::mem::take(&mut self.resumed)
	}

	/// What a quit at `quit_at` (unix ms) keeps for the next start (`restore`): every run, open or closed within
	/// `match_relaunch_s`, the feeds' matches, each game's last ended run's close, the first look and since when a
	/// match runs.
	#[must_use]
	pub fn keep(&self, quit_at: i64) -> KeptRuns {
		let id = |watched: &Watched| watched.id().to_owned();
		let mut runs: Vec<KeptRun> = self
			.runs
			.iter()
			.map(|(watched, run)| KeptRun {
				game: id(watched),
				since: run.since,
				from: run.from,
				closed: run.closed,
				close: run.close,
				told: run.told,
				ended: run.ended,
			})
			.collect();
		runs.sort_by(|a, b| a.game.cmp(&b.game));
		let mut fed: Vec<String> = self.fed.iter().map(id).collect();
		fed.sort();
		KeptRuns {
			quit_at,
			runs,
			cs2: self.cs2,
			dota: self.dota,
			rocket: self.rocket,
			replay: self.replay,
			fed,
			heard: self.heard.iter().map(|(watched, at)| (id(watched), *at)).collect(),
			last_close: self.last_close.iter().map(|(watched, at)| (id(watched), *at)).collect(),
			first_look: self.first_look,
			playing_since: self.playing_since,
		}
	}

	/// Takes on what a quit kept (`keep`) at a start at `now` (unix ms) within `match_relaunch_s` of it, before the
	/// first look, for the games in `watched`: the watcher goes on as if its looks had missed the time between, as a
	/// start again within `match_relaunch_s` of a close continues a run (a longer gap could hide a close and a new
	/// start, which are two runs). A run open at the quit goes on while its process runs and closes at the first look
	/// otherwise, dated there and offering by the floors; one closed at the quit goes on if its game started again,
	/// its offer then recalled (none stands in this process); a feed's match goes on, so its next in-match word is no
	/// new match (an end a game's log wrote between the quit and the start is never read: its reader starts at the
	/// log's end, and the match ends at its next end or at the close); each game's last close and the first look come
	/// back too (a program no longer watched leaves its close behind). A later start, or a watcher that looked
	/// already, takes nothing. How many runs went on.
	pub fn restore(&mut self, kept: &KeptRuns, now: i64, watched: &[Watched]) -> usize {
		if self.first_look.is_some() || !(0..self.relaunch_ms).contains(&(now - kept.quit_at)) {
			return 0;
		}
		let of = |id: &str| watched.iter().find(|w| w.id() == id);
		let mut taken = 0;
		for run in kept.runs.iter().filter(|r| {
			r.since <= r.from && r.from <= kept.quit_at && r.closed.is_none_or(|c| r.since <= c && c <= kept.quit_at)
		}) {
			let Some(game) = of(&run.game) else { continue };
			let KeptRun { since, from, closed, close, told, ended, .. } = *run;
			self.runs.insert(game.clone(), Run { since, from, closed, close, told, ended });
			taken += 1;
		}
		let named = |w: Watched| watched.contains(&w);
		if named(Watched::CounterStrike) {
			self.cs2 = kept.cs2;
		}
		if named(Watched::Dota) {
			self.dota = kept.dota;
		}
		if named(Watched::RocketLeague) {
			self.rocket = kept.rocket;
			self.replay = kept.replay;
		}
		self.fed.extend(kept.fed.iter().filter_map(|id| of(id)).cloned());
		for (id, &at) in kept.heard.iter().filter(|&(_, &at)| at <= kept.quit_at) {
			if let Some(game) = of(id).filter(|game| !self.runs.contains_key(game)) {
				self.heard.insert(game.clone(), at);
			}
		}
		for (id, &at) in kept.last_close.iter().filter(|&(_, &at)| at <= kept.quit_at) {
			if let Some(game) = of(id).cloned().or_else(|| Watched::from_key(id)) {
				self.last_close.insert(game, at);
			}
		}
		self.first_look = kept.first_look.filter(|&at| at <= kept.quit_at);
		self.playing_since = kept.playing_since.filter(|&at| at <= kept.quit_at && self.in_match());
		taken
	}

	/// Takes one signal; a match's end gives its finish, once. A feed's word marks its game's run told, and a match's
	/// end it gave marks the run ended, the run's next match beginning there; one that ended under `match_min_s` from
	/// where it began gives no finish and leaves the run as it was (its match goes on: Elite Dangerous' missions handed
	/// in together finish once, the first, and a forfeit in a match's first minutes not at all), unless its game has
	/// no run to measure it by (`Finish::began_at`). A finish ends a start again's continued match (`Close::Rejoined`).
	pub fn step(&mut self, signal: Signal) -> Option<Finish> {
		// (the finish, the signal's time, the game whose feed spoke)
		let (finish, at, word) = match signal {
			Signal::Gsi { game, phase, at } => {
				self.told(&game.watched());
				(self.gsi(game, phase.as_deref(), at), at, Some(game.watched()))
			}
			Signal::Process { watched, running, at } => (self.process(watched, running, at), at, None),
			Signal::Rocket { event, at } => {
				self.told(&Watched::RocketLeague);
				(self.rocket_event(&event, at), at, Some(Watched::RocketLeague))
			}
			Signal::RocketGone { at } => {
				self.forget_match(&Watched::RocketLeague);
				(None, at, None)
			}
			Signal::Fed { watched, state, at } => {
				self.told(&watched);
				(self.fed(watched.clone(), state, at), at, Some(watched))
			}
			Signal::Answered { watched, at } => {
				self.told(&watched);
				(None, at, Some(watched))
			}
		};
		self.first_look.get_or_insert(at);
		if let Some(watched) = word.filter(|w| !self.runs.contains_key(w)) {
			self.heard.entry(watched).or_insert(at);
		}
		// the floor a close has spaces a feed's finishes too, a checkpoint game's by its own
		let finish = finish.filter(|f| {
			f.at_close
				|| !self.runs.contains_key(&f.watched)
				|| f.ended_at - f.began_at >= self.feed_floor_ms(&f.watched)
		});
		if let Some(f) = &finish
			&& let Some(run) = self.runs.get_mut(&f.watched)
		{
			if run.close == Close::Rejoined {
				run.close = Close::Quiet;
			}
			if !f.at_close {
				run.ended = true;
				run.from = f.ended_at;
			}
		} else if let Some(f) = &finish {
			self.heard.insert(f.watched.clone(), f.ended_at);
		}
		if self.in_match() {
			self.playing_since.get_or_insert(at);
		} else {
			self.playing_since = None;
		}
		finish
	}

	/// A CS2 phase or a Dota 2 game state. CS2: `warmup` starts a match, `live` and `intermission` arm it, `gameover`
	/// ends an armed one. Dota 2: the states before `GAME_IN_PROGRESS` start one, `GAME_IN_PROGRESS` arms it,
	/// `POST_GAME` ends an armed one; a disconnect leaves it armed for the reconnect. No map: the menus. A start is
	/// the match's beginning only where the feed knew no match before it (the posts repeat a phase), and a match first
	/// heard armed began where its run's last one ended (`Run::from`).
	fn gsi(&mut self, game: Gsi, phase: Option<&str>, at: i64) -> Option<Finish> {
		let feed = match game {
			Gsi::CounterStrike => &mut self.cs2,
			Gsi::Dota => &mut self.dota,
		};
		let Some(phase) = phase else {
			*feed = Feed::default();
			return None;
		};
		let new = !feed.in_match && !feed.armed;
		// (whether this word starts a match, whether it ends one)
		let (began, ended) = match game {
			Gsi::CounterStrike => match phase {
				"warmup" => {
					feed.in_match = true;
					(new, false)
				}
				"live" | "intermission" => {
					*feed = Feed { in_match: true, armed: true };
					(false, false)
				}
				"gameover" => (false, true),
				_ => (false, false),
			},
			Gsi::Dota => match phase.strip_prefix("DOTA_GAMERULES_STATE_").unwrap_or(phase) {
				"GAME_IN_PROGRESS" => {
					*feed = Feed { in_match: true, armed: true };
					(false, false)
				}
				"POST_GAME" => (false, true),
				"DISCONNECT" | "LAST" => {
					feed.in_match = false;
					(false, false)
				}
				_ => {
					feed.in_match = true;
					(new, false)
				}
			},
		};
		if ended {
			let armed = std::mem::take(feed).armed;
			return armed.then(|| self.finish(game.watched(), at, false));
		}
		if began {
			self.began(&game.watched(), at);
		}
		None
	}

	/// A Stats API event: `MatchInitialized` starts a match (its beginning, `Run::from`), `RoundStarted` arms it,
	/// `MatchEnded` ends an armed one; from `ReplayCreated` to `MatchDestroyed` every event is a replay's.
	fn rocket_event(&mut self, event: &str, at: i64) -> Option<Finish> {
		if event == "MatchDestroyed" {
			self.rocket = Feed::default();
			self.replay = false;
			return None;
		}
		if self.replay {
			return None;
		}
		match event {
			"ReplayCreated" => {
				self.replay = true;
				self.rocket = Feed::default();
				None
			}
			"MatchInitialized" => {
				self.rocket.in_match = true;
				self.began(&Watched::RocketLeague, at);
				None
			}
			"RoundStarted" => {
				self.rocket = Feed { in_match: true, armed: true };
				None
			}
			"MatchEnded" => {
				let armed = self.rocket.armed;
				self.rocket = Feed::default();
				armed.then(|| self.finish(Watched::RocketLeague, at, false))
			}
			_ => None,
		}
	}

	/// A fed game's word: `Playing` arms its match (its first word since the last end is the match's beginning,
	/// `Run::from`), `Ended` ends an armed one, `Completed` ends one whatever came before; each finish once, on the
	/// edge.
	fn fed(&mut self, watched: Watched, state: Fed, at: i64) -> Option<Finish> {
		match state {
			Fed::Playing => {
				if self.fed.insert(watched.clone()) {
					self.began(&watched, at);
				}
				None
			}
			Fed::Ended => self.fed.remove(&watched).then(|| self.finish(watched, at, false)),
			Fed::Completed => {
				self.fed.remove(&watched);
				Some(self.finish(watched, at, false))
			}
		}
	}

	/// A match of `watched` began at `at` (unix ms), as its feed told: its open run's current match from now on, but
	/// the first after a start again (`Close::Rejoined`), which is the match the close broke off going on.
	fn began(&mut self, watched: &Watched, at: i64) {
		if !self.runs.contains_key(watched) {
			self.heard.insert(watched.clone(), at);
		}
		let Some(run) = self.runs.get_mut(watched).filter(|r| r.closed.is_none()) else { return };
		if run.close == Close::Rejoined {
			run.close = Close::Quiet;
		} else {
			run.from = at;
		}
	}

	/// `watched`'s finish at `at` (unix ms): its match from where its run's current one began (`Run::from`), where it
	/// has no run from its first word since its last finish (`heard`), else from `at` itself; with the close of its
	/// game's last ended run (`last_close`), else the first look.
	fn finish(&self, watched: Watched, at: i64, at_close: bool) -> Finish {
		let from = self.runs.get(&watched).map(|run| run.from).or_else(|| self.heard.get(&watched).copied());
		let began_at = from.map_or(at, |from| from.min(at));
		let previous_close = self.last_close.get(&watched).copied().or(self.first_look);
		Finish { watched, began_at, ended_at: at, at_close, previous_close }
	}

	/// A look at a game's process. Its close offers at the first look it is gone, once its run reached its floor
	/// (`floor_ms`) from its first start and its feed ended no match in that run; its feed's match ends with it, and a
	/// fed game's armed one finishes at the close whatever the floor (the player left iRacing's session, quit within
	/// Forza Motorsport's 30 s, or the game crashed). Both on the edge alone, so a process the helper does not tell
	/// never ends the match its feed tells; the close's match runs from where the run's current one began (`Run::from`:
	/// its first start, a match start or the last finish its feed told). A start again within `relaunch_ms` continues
	/// the run (a close that offered is `resumed` and its match goes on, `Close::Rejoined`, and the run's next close
	/// may offer again), and a run closed longer goes, its close the game's last (`last_close`).
	fn process(&mut self, watched: Watched, running: bool, at: i64) -> Option<Finish> {
		let floor = self.floor_ms(&watched);
		let closing = !running && self.runs.get(&watched).is_some_and(|r| r.closed.is_none());
		let armed = closing && self.fed.contains(&watched);
		if closing {
			self.forget_match(&watched);
		}
		match (self.runs.get_mut(&watched), running) {
			(None, true) => {
				// the first look that sees it running: a game already open as the helper started counts from here, but for
				// the run a quit kept (`restore`), which is open already
				self.heard.remove(&watched);
				self.runs.insert(watched, Run::new(at));
				None
			}
			(Some(run), true) => {
				match run.closed.take() {
					// a start no look caught closed long enough (the computer slept) is a new run
					Some(closed) if at - closed >= self.relaunch_ms => {
						*run = Run::new(at);
						self.last_close.insert(watched, closed);
					}
					Some(_) => {
						let offered = run.close == Close::Offered;
						run.close = if offered { Close::Rejoined } else { Close::Quiet };
						if offered {
							self.resumed.push(watched);
						}
					}
					None => {}
				}
				None
			}
			(None, false) => None,
			(Some(run), false) => match run.closed {
				None => {
					run.closed = Some(at);
					let offers = run.close != Close::Offered && (armed || (!run.ended && at - run.since >= floor));
					if offers {
						run.close = Close::Offered;
					}
					offers.then(|| self.finish(watched, at, true))
				}
				Some(closed) if at - closed >= self.relaunch_ms => {
					self.runs.remove(&watched);
					self.last_close.insert(watched, closed);
					None
				}
				Some(_) => None,
			},
		}
	}

	/// Whether a watched game is in a match now: a game heard by its feed between its match's start and its end, a game
	/// whose feed told nothing in this run (League, a program, a named game not set up) while its process runs.
	#[must_use]
	pub fn in_match(&self) -> bool {
		self.feeds_in_match() || self.runs.values().any(|r| r.closed.is_none() && !r.told)
	}

	/// Whether a match runs now, as the sitting clock counts a game as input whatever window is in front: a fed game
	/// between its start and its end, or League of Legends' game process, which runs one match; a found or picked
	/// program's run is a session, which counts only while its window is in front.
	#[must_use]
	pub fn match_runs(&self) -> bool {
		self.feeds_in_match() || self.runs.get(&Watched::League).is_some_and(|r| r.closed.is_none())
	}

	/// A game heard by its feed is between its match's start and its end.
	fn feeds_in_match(&self) -> bool {
		self.cs2.in_match || self.dota.in_match || self.rocket.in_match || !self.fed.is_empty()
	}

	/// Whether the game `id` (`Watched::id`) has a run open: its process runs, or closed less than
	/// `match_relaunch_s` ago.
	#[must_use]
	pub fn runs(&self, id: &str) -> bool {
		self.runs.keys().any(|w| w.id() == id)
	}

	/// A picked or found program's key changed (the desktop app keeps a Steam game's program by its app from now on,
	/// `steam:<app>`): its run goes on under `to` with its first start, its close and whether that close offered, so a
	/// run open as the key changed still ends at the program's real close and a start again within `match_relaunch_s`
	/// still continues it; where `to` has a run already, `from`'s goes. Its last ended run's close (`last_close`) moves
	/// the same way. Only a program's key changes: the fed games' matches (`fed`) are a named game's, whose key never
	/// does.
	pub fn rekey(&mut self, from: &Watched, to: Watched) {
		if let Some(run) = self.runs.remove(from) {
			self.runs.entry(to.clone()).or_insert(run);
		}
		if let Some(close) = self.last_close.remove(from) {
			self.last_close.entry(to.clone()).or_insert(close);
		}
		if self.resumed.contains(&to) {
			self.resumed.retain(|w| w != from);
		} else if let Some(resumed) = self.resumed.iter_mut().find(|w| *w == from) {
			*resumed = to;
		}
	}

	/// A feed's word for `watched`: its open run was told (its process running is no longer its match).
	fn told(&mut self, watched: &Watched) {
		if let Some(run) = self.runs.get_mut(watched).filter(|r| r.closed.is_none()) {
			run.told = true;
		}
	}

	/// Drops what is known of a game turned off or a program removed: its match and its run. Its last ended run's
	/// close stays (`last_close`), a time on this computer's clock that bounds League's lead if it is turned on again.
	pub fn forget(&mut self, watched: &Watched) {
		self.forget_match(watched);
		self.runs.remove(watched);
		self.heard.remove(watched);
		self.resumed.retain(|w| w != watched);
		if !self.in_match() {
			self.playing_since = None;
		}
	}

	/// Drops the runs of the programs no longer watched, as `forget` does (`watched` says which still are: a found game
	/// that left the found ones or took a new key, a program turned off or removed), so none stays open as a match
	/// until the helper's next start; a named game's run stays.
	pub fn forget_programs(&mut self, watched: impl Fn(&Watched) -> bool) {
		let gone: Vec<Watched> =
			self.runs.keys().filter(|w| matches!(w, Watched::Program { .. }) && !watched(w)).cloned().collect();
		for program in &gone {
			self.forget(program);
		}
	}

	/// Drops a game's feed's match: its game closed, or its feed's connection ended.
	fn forget_match(&mut self, watched: &Watched) {
		match watched {
			Watched::CounterStrike => self.cs2 = Feed::default(),
			Watched::Dota => self.dota = Feed::default(),
			Watched::RocketLeague => {
				self.rocket = Feed::default();
				self.replay = false;
			}
			Watched::StarCraft
			| Watched::MtgArena
			| Watched::Hearthstone
			| Watched::Runeterra
			| Watched::FormulaOne
			| Watched::Forza
			| Watched::IRacing
			| Watched::AssettoCorsa
			| Watched::Elite
			| Watched::SlayTheSpire
			| Watched::AssettoCorsaOne
			| Watched::GuildWars
			| Watched::Mechabellum
			| Watched::PathOfExile
			| Watched::Warframe
			| Watched::Warships
			| Watched::Siege
			| Watched::Tarkov
			| Watched::Deadlock
			| Watched::Fortnite
			| Watched::TeamFortress
			| Watched::Naraka
			| Watched::FallGuys
			| Watched::DeadByDaylight
			| Watched::Hunt
			| Watched::Valorant
			| Watched::Pubg
			| Watched::Halo
			| Watched::Brawlhalla
			| Watched::RFactor
			| Watched::LeMans
			| Watched::Automobilista
			| Watched::Osu
			| Watched::Stardew
			| Watched::Warcraft
			| Watched::EldenRing
			| Watched::Nightreign => {
				self.fed.remove(watched);
			}
			Watched::League | Watched::Program { .. } => {}
		}
	}

	/// The sync response's `playing`: a watched game in a match, for at most `PLAYING_CAP_MS` without a break.
	#[must_use]
	pub fn playing(&self, now: i64) -> bool {
		self.playing_since.is_some_and(|since| now - since < PLAYING_CAP_MS)
	}
}
