//! Slice 48's fed games in the helper's loop (docs/plan.md slices 48 and 56; docs/computer-games.md §6.2): StarCraft
//! II and Legends of Runeterra asked on the loopback while they run (`sc2.rs`, `runeterra.rs`), F1 25 and Forza
//! Motorsport heard on a loopback UDP port while they run (`udp.rs`), MTG Arena's, Elite Dangerous', Mechabellum's
//! (2026-10-28) and, once it is set up, Hearthstone's own logs followed while they run and read once more at their
//! close (`arena.rs`, `elite.rs`, `mechabellum.rs`, `hearthstone.rs`), iRacing's, Assetto Corsa Competizione's and,
//! since 2026-10-28, Assetto Corsa's and Guild Wars 2's documented shared memory read at each look while they run
//! (`shm.rs`, Windows), and since 2026-10-28 Slay the Spire 2's run history looked at while it runs and once more at
//! its close (`spire.rs`, Windows); since 2026-10-29 Path of Exile's, Warframe's, Fortnite's, Deadlock's and Team
//! Fortress 2's logs (`logs.rs`) and Escape from Tarkov's (`tarkov.rs`), followed as the other logs are, and World of
//! Warships' replays folder and Rainbow Six Siege's match replays listed as Slay the Spire 2's history is
//! (`warships.rs`, `siege.rs`), Windows; later that day NARAKA: BLADEPOINT's, Fall Guys', Dead by Daylight's, Hunt:
//! Showdown 1896's and VALORANT's logs (`logs.rs`), PUBG's replay folders (`pubg.rs`), the files Halo: The Master Chief
//! Collection, Brawlhalla, rFactor 2, Le Mans Ultimate, osu!, Stardew Valley and Warcraft III save at an end
//! (`saves.rs`; Stardew Valley's and Warcraft III's on macOS too) and Automobilista 2's documented shared memory
//! (`shm.rs`); and ELDEN RING's and ELDEN RING NIGHTREIGN's boss banners, read off the game's own window while it runs
//! (`banner.rs`, Windows). Each feed tells
//! only `Word`s: that it answered (the game is heard), whether a match runs (questpump-core's `Fed`, which the watcher
//! turns into a finish that offers at once, as Rocket League's does), and a step the player takes first (Detailed
//! Logs, a port another program holds, an F1 UDP format to change). A game that ran `SILENT_MS` with no word in this
//! run asks for its step too (`check_fed`): the loopback games, MTG Arena's and Elite Dangerous' logs and the racing
//! games' shared memory (Automobilista 2's for its Shared Memory setting), and Deadlock and Team Fortress 2 for their
//! launch option `-condebug`, never the UDP games
//! (silent in their menus), Hearthstone, Mechabellum, Path of Exile, Warframe, Escape from Tarkov and Fortnite (whose
//! logs, on by default, have no step to ask, nor have the log games added later that day), Guild Wars 2 (its link
//! waits for a character), Slay the Spire 2, World of Warships, Rainbow Six Siege nor the other file games (their
//! files wait for a run, a battle, a match or a day), nor the ELDEN RING games (no setting of theirs turns a banner
//! off). A reader goes with its game's close (a task stops, a log read its last lines, a view is no longer read, a
//! capture ends): a match it armed then ends at the close, held as
//! every close is (questpump-core's `Watcher`, `Finish::at_close`). The tasks' words come on the games' channel
//! (`News::Fed`, `Games::on_feed`), the looks' within `Games::tick`. Hearthstone's `log.config` gets its `[Power]`
//! section here when it has none (`files::add_section`), its folder made when the game is installed and has not made
//! it yet.

use std::path::PathBuf;

use questpump_core::feeds::{acc, automobilista, guildwars, hearthstone as hs, iracing, starcraft};
use questpump_core::games::{Fed, Finish, Signal, Watched};

use super::arena::{self, Arena};
use super::banner::Banner;
use super::elite::Elite;
use super::hearthstone::{self, Hearthstone};
use super::logs::{self, Logs};
use super::mechabellum::{self, Mechabellum};
use super::procs::Snapshot;
use super::pubg::{self, Pubg};
use super::saves::{self, Saves};
use super::siege::{self, Siege};
use super::spire::{self, Spire};
use super::tarkov::{self, Tarkov};
use super::task::{self, Task, Teller};
use super::udp::{self, Racing};
use super::warships::{self, Warships};
use super::{Games, SILENT_MS, Status, files, heard, hears, riot, rocket, runeterra, sc2, steam};
use crate::install;
use crate::settings::GamesSettings;

/// What a feed of the games tells the loop on the games' channel: Rocket League's connection, or a slice-48 feed's
/// word.
#[derive(Debug)]
pub enum News {
	Rocket(rocket::News),
	Fed(Watched, Word),
}

/// A fed game's word.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Word {
	/// The feed answered: the game is heard.
	Up,
	/// Whether a match runs, for the watcher (the feed answered too).
	Match(Fed),
	/// A step the player takes before the feed can answer (`PortTaken`, `UdpFormat`).
	Step(Status),
	/// The port is listened on again.
	Cleared,
}

/// How a fed game is heard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Feed {
	/// Asked on the loopback while it runs.
	Client,
	/// Its own log, followed while it runs and read once more at its close.
	Log,
	/// Its UDP telemetry, heard while it runs (the port stays free for another program otherwise).
	Udp(Racing),
	/// Its documented shared memory, read while it runs (a mapping another program keeps open after the game closed
	/// holds what the game last wrote, never read).
	Shared,
	/// A new file in its own save folder, noticed by its time while it runs and once more at its close, never opened.
	Files,
	/// Its boss banners, read off the middle of its own window while it runs (Windows): nothing of a frame is kept.
	Screen,
}

impl Feed {
	pub(super) fn of(watched: &Watched) -> Option<Self> {
		match watched {
			Watched::StarCraft | Watched::Runeterra => Some(Self::Client),
			Watched::MtgArena
			| Watched::Hearthstone
			| Watched::Elite
			| Watched::Mechabellum
			| Watched::PathOfExile
			| Watched::Warframe
			| Watched::Tarkov
			| Watched::Deadlock
			| Watched::Fortnite
			| Watched::TeamFortress
			| Watched::Naraka
			| Watched::FallGuys
			| Watched::DeadByDaylight
			| Watched::Hunt
			| Watched::Valorant => Some(Self::Log),
			Watched::FormulaOne => Some(Self::Udp(Racing::Formula)),
			Watched::Forza => Some(Self::Udp(Racing::Forza)),
			Watched::IRacing
			| Watched::AssettoCorsa
			| Watched::AssettoCorsaOne
			| Watched::GuildWars
			| Watched::Automobilista => Some(Self::Shared),
			Watched::SlayTheSpire
			| Watched::Warships
			| Watched::Siege
			| Watched::Pubg
			| Watched::Halo
			| Watched::Brawlhalla
			| Watched::RFactor
			| Watched::LeMans
			| Watched::Osu
			| Watched::Stardew
			| Watched::Warcraft => Some(Self::Files),
			Watched::EldenRing | Watched::Nightreign => Some(Self::Screen),
			Watched::CounterStrike
			| Watched::Dota
			| Watched::League
			| Watched::RocketLeague
			| Watched::Program { .. } => None,
		}
	}
}

/// The fed games' readers while their games want them.
#[derive(Default)]
pub(super) struct Feeds {
	/// The loopback polls' client, made at the first poll, and whether that was tried (no poll without it).
	client: Option<reqwest::Client>,
	client_tried: bool,
	/// The tasks that run, by key, with the port each took.
	tasks: std::collections::HashMap<&'static str, (Task, u16)>,
	arena: Option<Arena>,
	hearthstone: Option<Hearthstone>,
	elite: Option<Elite>,
	mechabellum: Option<Mechabellum>,
	iracing: Option<iracing::Session>,
	acc: Option<acc::Race>,
	/// The games added on 2026-10-28: Assetto Corsa's race (the same page as Assetto Corsa Competizione's, read for
	/// either game alone while it runs), Guild Wars 2's player-versus-player match, Slay the Spire 2's run history.
	acone: Option<acc::Race>,
	guildwars: Option<guildwars::Match>,
	spire: Option<Spire>,
	/// The games added on 2026-10-29: the ones heard by one log at a known place, by key (`logs.rs`), Escape from
	/// Tarkov's session logs, World of Warships' replays folder and Rainbow Six Siege's match replays.
	logs: std::collections::HashMap<&'static str, Option<Logs>>,
	tarkov: Option<Tarkov>,
	warships: Option<Warships>,
	siege: Option<Siege>,
	/// The games added later on 2026-10-29: PUBG's replay folders, the games heard by the files they save at an end, by
	/// key (`saves.rs`), and Automobilista 2's race.
	pubg: Option<Pubg>,
	saves: std::collections::HashMap<&'static str, Option<Saves>>,
	automobilista: Option<automobilista::Race>,
	/// The ELDEN RING games' boss banners' readers while they run, by key (`banner.rs`).
	banners: std::collections::HashMap<&'static str, Banner>,
}

/// StarCraft II's default install, for a copy the Battle.net Agent does not name.
#[cfg(windows)]
const STARCRAFT: &str = r"C:\Program Files (x86)\StarCraft II"; // copy-lint: not shown (a path)
#[cfg(not(windows))]
const STARCRAFT: &str = "/Applications/StarCraft II"; // copy-lint: not shown (a path)
/// The folders F1 25's, Forza Motorsport's and iRacing's launchers use by default (Windows).
const FORMULA_ONE: &str = r"C:\Program Files\EA Games\F1 25"; // copy-lint: not shown (a path)
const FORZA: &str = r"C:\XboxGames\Forza Motorsport"; // copy-lint: not shown (a path)
const IRACING: &str = r"C:\Program Files (x86)\iRacing"; // copy-lint: not shown (a path)
/// Guild Wars 2's own installer's folder (Windows).
const GUILD_WARS: &str = r"C:\Program Files\Guild Wars 2"; // copy-lint: not shown (a path)
/// Warcraft III's default install, for a copy the Battle.net Agent does not name.
#[cfg(windows)]
const WARCRAFT: &str = r"C:\Program Files (x86)\Warcraft III"; // copy-lint: not shown (a path)
#[cfg(not(windows))]
const WARCRAFT: &str = "/Applications/Warcraft III"; // copy-lint: not shown (a path)

/// The port a fed game's feed takes: the player's, else its own default; none for a game whose feed has no port.
pub(super) fn port_of(settings: &GamesSettings, watched: &Watched) -> Option<u16> {
	let default = watched.default_port()?;
	Some(settings.ports.get(watched.link()).copied().unwrap_or(default))
}

/// Whether a fed game is installed on this computer, as far as its records tell (the Games tab lists those): Steam's
/// libraries (one under `/Volumes` on macOS only for a game the player asked for, `volumes`), the Battle.net Agent's
/// database, the Riot Client's record (`riot::install`), the folders the games and their launchers use by default, or
/// the folder its log goes into once it ran.
pub(super) fn installed(watched: &Watched, volumes: bool) -> bool {
	install_dir(watched, volumes).is_some()
		|| match watched {
			Watched::MtgArena => arena::log_dir().is_some_and(|d| d.is_dir()),
			Watched::Elite => journals(),
			Watched::SlayTheSpire => spire::root().is_some_and(|d| d.is_dir()),
			Watched::Mechabellum => mechabellum::log_dir().is_some_and(|d| d.is_dir()),
			Watched::Warframe => logs::warframe_dir().is_some_and(|d| d.is_dir()),
			Watched::Fortnite => logs::fortnite_logs().is_some_and(|d| d.is_dir()),
			Watched::Naraka | Watched::FallGuys | Watched::DeadByDaylight | Watched::Valorant => {
				logs::log_dir(watched).is_some_and(|d| d.is_dir())
			}
			Watched::Pubg => pubg::demos().is_some_and(|d| d.is_dir()),
			Watched::Halo => saves::halo_reports().is_some_and(|d| d.is_dir()),
			Watched::Brawlhalla => saves::brawlhalla_replays().is_some_and(|d| d.is_dir()),
			Watched::Stardew => saves::stardew_saves().is_some_and(|d| d.is_dir()),
			Watched::Warcraft => saves::warcraft_files().is_some_and(|d| d.is_dir()),
			_ => false,
		}
}

/// A fed game's install folder, where its records name one (`installed`'s: a Steam library's, the Battle.net Agent's,
/// the Riot Client's, the folder its launcher uses by default); none for the others.
pub(super) fn install_dir(watched: &Watched, volumes: bool) -> Option<PathBuf> {
	let steam = || steam::game_dir(watched, volumes);
	let default = |dir: &str| Some(PathBuf::from(dir)).filter(|d| d.is_dir());
	match watched {
		Watched::StarCraft => install::battlenet_install("s2").or_else(|| default(STARCRAFT)),
		Watched::Hearthstone => hearthstone::install(),
		Watched::Runeterra | Watched::Valorant => riot::install(watched),
		Watched::Warcraft => install::battlenet_install("w3").or_else(|| default(WARCRAFT)),
		Watched::Osu => saves::osu().filter(|d| d.is_dir()),
		Watched::FormulaOne => steam().or_else(|| default(FORMULA_ONE)),
		Watched::Forza => steam().or_else(|| default(FORZA)),
		Watched::IRacing => steam().or_else(|| default(IRACING)),
		Watched::MtgArena
		| Watched::AssettoCorsa
		| Watched::Elite
		| Watched::Mechabellum
		| Watched::SlayTheSpire
		| Watched::AssettoCorsaOne
		| Watched::Deadlock
		| Watched::TeamFortress
		| Watched::Naraka
		| Watched::FallGuys
		| Watched::DeadByDaylight
		| Watched::Hunt
		| Watched::Pubg
		| Watched::Halo
		| Watched::Brawlhalla
		| Watched::RFactor
		| Watched::LeMans
		| Watched::Automobilista
		| Watched::Stardew
		| Watched::EldenRing
		| Watched::Nightreign => steam(),
		Watched::GuildWars => steam().or_else(|| default(GUILD_WARS)),
		Watched::PathOfExile => logs::pathofexile_installs().into_iter().next(),
		Watched::Warframe => {
			steam().or_else(|| logs::warframe_dir().map(|d| d.join("Downloaded").join("Public")).filter(|d| d.is_dir()))
		}
		Watched::Fortnite => logs::fortnite_install(),
		Watched::Tarkov => tarkov::install(),
		Watched::Warships => warships::install(),
		Watched::Siege => siege::install(),
		Watched::CounterStrike | Watched::Dota | Watched::League | Watched::RocketLeague | Watched::Program { .. } => {
			None
		}
	}
}

/// Whether Elite Dangerous' journals' folder is there (the game ran on this computer).
fn journals() -> bool {
	#[cfg(windows)]
	{
		super::elite::journals().is_some_and(|d| d.is_dir())
	}
	#[cfg(not(windows))]
	{
		false
	}
}

/// iRacing's sim as its shared memory says it; none outside Windows or while the mapping is gone.
fn iracing_sim() -> Option<iracing::Sim> {
	#[cfg(windows)]
	{
		super::shm::read(super::shm::IRACING, |bytes| iracing::read(bytes))
	}
	#[cfg(not(windows))]
	{
		None
	}
}

/// The Kunos games' `status` and `session` as their graphics page says them (Assetto Corsa Competizione's, or Assetto
/// Corsa's: the same page); none outside Windows or while the mapping is gone.
fn acc_page() -> Option<(i32, i32)> {
	#[cfg(windows)]
	{
		super::shm::read(super::shm::ACC, |bytes| acc::graphics(&bytes(0, acc::READ)?))
	}
	#[cfg(not(windows))]
	{
		None
	}
}

/// Automobilista 2's game, session and race states as its shared memory says them; none outside Windows or while the
/// mapping is gone (its Shared Memory setting off, or the game closed).
fn automobilista_states() -> Option<(i32, i32, i32)> {
	#[cfg(windows)]
	{
		super::shm::read(super::shm::AUTOMOBILISTA, |bytes| automobilista::states(&bytes(0, automobilista::READ)?))
	}
	#[cfg(not(windows))]
	{
		None
	}
}

/// Guild Wars 2's `mapType` as its link says it; none outside Windows, while the mapping is gone or before the game
/// filled it.
fn guildwars_map() -> Option<i32> {
	#[cfg(windows)]
	{
		super::shm::read(super::shm::GUILD_WARS, |bytes| guildwars::map_type(bytes))
	}
	#[cfg(not(windows))]
	{
		None
	}
}

impl Games {
	/// The status a fed game starts at, or comes back to: heard, a racing game's telemetry to turn on, or waiting for
	/// the game's first word.
	pub(super) fn fed_initial(watched: &Watched, settings: &GamesSettings) -> Status {
		if settings.heard.iter().any(|k| k == watched.link()) {
			Status::Ready
		} else if matches!(Feed::of(watched), Some(Feed::Udp(_))) {
			Status::Udp
		} else {
			Status::Waiting
		}
	}

	/// A heard fed game's place: Hearthstone's section goes into its `log.config` (once it is set up); the others write
	/// nothing and take their first status, from a new track's `Off` too (found means watched: no switch gives one its
	/// first status), so a first-run step shows.
	pub(super) fn place_fed(&mut self, watched: &Watched, settings: &mut GamesSettings) -> bool {
		if *watched == Watched::Hearthstone {
			return self.place_section(settings);
		}
		let track = self.tracks.entry(watched.link()).or_default();
		if matches!(track.status, Status::Off | Status::Waiting | Status::Udp) {
			track.status = Self::fed_initial(watched, settings);
		}
		false
	}

	/// Hearthstone's `log.config` with a `[Power]` section: one that has a section is left alone (the trackers' or
	/// the helper's); else the section is added at its end, never written whole, and listed as a shared file, which
	/// turning the game off leaves. A preferences folder the game has not made yet is made once its install is known,
	/// so its first start writes `Power.log`; without an install the game is not found.
	fn place_section(&mut self, settings: &mut GamesSettings) -> bool {
		let key = Watched::Hearthstone.link();
		let track = self.tracks.entry(key).or_default();
		let Some((path, dir)) = hearthstone::config().and_then(|p| Some((p.clone(), p.parent()?.to_path_buf()))) else {
			track.status = Status::NotFound;
			return false;
		};
		if !dir.is_dir() {
			if hearthstone::install().is_none() {
				track.status = Status::NotFound;
				return false;
			}
			if let Err(e) = std::fs::create_dir_all(&dir) {
				Self::written(track, key, dir, Err(e));
				return false;
			}
		}
		let text = match std::fs::read_to_string(&path) {
			Ok(text) => text,
			Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
			Err(e) => {
				log::warn!("games: {key}: log.config could not be read: {e}");
				track.status = Status::Failed;
				return false;
			}
		};
		if hs::has_section(&text) {
			track.status = Self::in_place(track, settings, key);
			track.dir = Some(dir);
			return false;
		}
		let outcome = files::add_section(settings, key, &path, &hs::section(&text));
		let changed = outcome.is_ok();
		if changed {
			settings.heard.retain(|k| k != key);
		}
		Self::written(track, key, dir, outcome);
		changed
	}

	/// A fed game that ran `SILENT_MS` without a word in this run asks for its step, a Ready kept from an earlier run
	/// included: a client game for its API where the game's settings can turn it off or move it (Legends of Runeterra),
	/// else for a new start (StarCraft II, whose client API no setting is known to change); Elite Dangerous for its
	/// journal, iRacing and the Kunos games for their shared memory (a game run as administrator keeps it from
	/// QuestPump; Automobilista 2's until its Shared Memory is set to Project CARS 2, the page's own words for it). The
	/// UDP games are silent in their menus, Hearthstone's `Power.log` before its first game, Guild Wars 2's link before
	/// a character is in the world and Slay the Spire 2's history before its first run, so none of them is asked; nor
	/// are Mechabellum and MTG Arena, whose logs need no setting and so have no step to name (MTG Arena's Detailed Logs
	/// step went on 2026-11-03: its match lines come without them). Since 2026-10-29 Deadlock and Team Fortress 2 ask
	/// for their launch option `-condebug`, without which they write no console log; Path of Exile's, Warframe's,
	/// Escape from Tarkov's and Fortnite's logs are written by default, and World of Warships' and Rainbow Six Siege's
	/// files wait for a battle or a match, so none of those is asked; nor are the log and file games added later that
	/// day, for the same reasons.
	pub(super) fn check_fed(&mut self, watched: &Watched, now: i64) {
		let step = match (Feed::of(watched), watched) {
			(Some(Feed::Client), Watched::Runeterra) => Status::ApiOff,
			(Some(Feed::Client), _) => Status::Restart,
			(Some(Feed::Log), Watched::Elite) => Status::NoJournal,
			(Some(Feed::Log), Watched::Deadlock | Watched::TeamFortress) => Status::Launch,
			(Some(Feed::Shared), Watched::GuildWars) => return,
			(Some(Feed::Shared), _) => Status::NoShared,
			_ => return,
		};
		let key = watched.link();
		let Some(track) = self.tracks.get_mut(key) else { return };
		let Some(since) = track.since else { return };
		let silent = now - since >= SILENT_MS && track.answered.is_none_or(|at| at < since);
		if silent && matches!(track.status, Status::Waiting | Status::Ready) {
			track.status = step;
			log::info!("games: {key}: no answer {} s after its start", SILENT_MS / 1_000);
		}
	}

	/// A racing game whose port another program held goes back to its first line once the game closed (its task
	/// stopped): its next start listens again.
	fn port_free(&mut self, watched: &Watched, settings: &GamesSettings) {
		if let Some(track) = self.tracks.get_mut(watched.link()).filter(|t| t.status == Status::PortTaken) {
			track.status = Self::fed_initial(watched, settings);
		}
	}

	/// The fed games at a look: each heard game's task or banners' reader started or stopped, its log or shared memory
	/// read, and the words of those reads taken; with the finishes they brought and whether the settings changed.
	pub(super) fn feeds_look(
		&mut self,
		snap: &Snapshot,
		now: i64,
		settings: &mut GamesSettings,
	) -> (Vec<Finish>, bool) {
		let mut words: Vec<(Watched, Word)> = Vec::new();
		for watched in Watched::named() {
			let Some(feed) = Feed::of(watched) else { continue };
			let key = watched.link();
			let on = hears(settings, watched);
			let running = self.running.iter().any(|r| r == key);
			let follows = on && running;
			let mut said = |word: Word| words.push((watched.clone(), word));
			match (feed, watched) {
				(Feed::Client, _) => {
					let port =
						if *watched == Watched::StarCraft { Some(starcraft::PORT) } else { port_of(settings, watched) };
					self.want(watched, port.filter(|_| on && running));
				}
				(Feed::Udp(_), _) => {
					let port = port_of(settings, watched).filter(|_| on && running);
					if port.is_none() {
						self.port_free(watched, settings);
					}
					self.want(watched, port);
				}
				(Feed::Log | Feed::Files, _) => self.feeds.read(watched, follows).into_iter().for_each(&mut said),
				(Feed::Screen, _) => {
					let pid = follows.then(|| self.pids(key, snap).first().copied()).flatten();
					self.banner(watched, pid);
				}
				(Feed::Shared, Watched::IRacing) => {
					if let Some(session) = keep(&mut self.feeds.iracing, on && running, iracing::Session::default) {
						let sim = iracing_sim();
						if sim.is_some() {
							said(Word::Up);
						}
						// a mapping gone is the sim out of its session
						session.push(sim.unwrap_or(iracing::Sim::Off)).map(Word::Match).into_iter().for_each(&mut said);
					}
				}
				(Feed::Shared, Watched::Automobilista) => {
					let slot = &mut self.feeds.automobilista;
					if let Some(race) = keep(slot, on && running, automobilista::Race::default) {
						// a mapping gone says nothing: the close ends a race it armed
						if let Some(states) = automobilista_states() {
							said(Word::Up);
							race.push(states).map(Word::Match).into_iter().for_each(&mut said);
						}
					}
				}
				(Feed::Shared, Watched::GuildWars) => {
					if let Some(pvp) = keep(&mut self.feeds.guildwars, on && running, guildwars::Match::default) {
						// a link gone or not the game's says nothing: the close ends a match it armed
						if let Some(map) = guildwars_map() {
							said(Word::Up);
							pvp.push(map).map(Word::Match).into_iter().for_each(&mut said);
						}
					}
				}
				(Feed::Shared, _) => {
					// the Kunos games write one page: each game's race read only while its own process runs
					let slot =
						if *watched == Watched::AssettoCorsaOne { &mut self.feeds.acone } else { &mut self.feeds.acc };
					if let Some(race) = keep(slot, on && running, acc::Race::default) {
						let page = acc_page();
						if page.is_some() {
							said(Word::Up);
						}
						race.push(page.unwrap_or_default()).map(Word::Match).into_iter().for_each(&mut said);
					}
				}
			}
		}
		let (mut finishes, mut changed) = (Vec::new(), false);
		for (watched, word) in words {
			let (finish, news) = self.word(&watched, word, now, settings);
			finishes.extend(finish);
			changed |= news;
		}
		(finishes, changed)
	}

	/// The loopback polls' client, made once.
	fn client(&mut self) -> Option<reqwest::Client> {
		if !std::mem::replace(&mut self.feeds.client_tried, true) {
			self.feeds.client = task::client();
		}
		self.feeds.client.clone()
	}

	/// A task for a client or UDP game while `port` is wanted: started, started again on another port, or stopped.
	fn want(&mut self, watched: &Watched, port: Option<u16>) {
		let key = watched.link();
		let Some(port) = port else {
			self.feeds.tasks.remove(key);
			return;
		};
		if self.feeds.tasks.get(key).is_some_and(|(_, at)| *at == port) {
			return;
		}
		let teller = Teller::new(watched.clone(), self.news.clone());
		let task = match Feed::of(watched) {
			Some(Feed::Udp(racing)) => Some(udp::start(teller, racing, port)),
			Some(Feed::Client) => self.client().map(|client| {
				if *watched == Watched::StarCraft {
					sc2::start(teller, client)
				} else {
					runeterra::start(teller, client, port)
				}
			}),
			_ => None,
		};
		match task {
			Some(task) => {
				self.feeds.tasks.insert(key, (task, port));
			}
			None => {
				self.feeds.tasks.remove(key);
			}
		}
	}

	/// An ELDEN RING game's banners' reader while its process `pid` runs heard: started, started again on another
	/// process of the game, or dropped (its capture stops on its own thread).
	fn banner(&mut self, watched: &Watched, pid: Option<u32>) {
		let key = watched.link();
		let Some(pid) = pid else {
			self.feeds.banners.remove(key);
			return;
		};
		if self.feeds.banners.get(key).is_some_and(|b| b.pid == pid) {
			return;
		}
		match Banner::start(pid, Teller::new(watched.clone(), self.news.clone()), key) {
			Some(banner) => {
				self.feeds.banners.insert(key, banner);
			}
			None => {
				self.feeds.banners.remove(key);
			}
		}
	}

	/// A fed game's word, from its task or a look: heard (its run told for the watcher, and a match's start or end), a
	/// step asked, or a port listened on again. A word for a game turned off a moment ago is dropped.
	pub(super) fn word(
		&mut self,
		watched: &Watched,
		word: Word,
		now: i64,
		settings: &mut GamesSettings,
	) -> (Option<Finish>, bool) {
		let key = watched.link();
		if !hears(settings, watched) {
			return (None, false);
		}
		let track = self.tracks.entry(key).or_default();
		match word {
			Word::Up | Word::Match(_) => {
				track.answered = Some(now);
				track.status = Status::Ready;
				let changed = heard(settings, key);
				// either tells the run: a game whose feed answered counts as in a match only between its feed's words
				let signal = match word {
					Word::Match(state) => Signal::Fed { watched: watched.clone(), state, at: now },
					_ => Signal::Answered { watched: watched.clone(), at: now },
				};
				(self.watcher.step(signal), changed)
			}
			Word::Step(status) => {
				if track.status != status {
					log::info!("games: {key}: {status:?}");
				}
				track.status = status;
				(None, false)
			}
			Word::Cleared => {
				if track.status == Status::PortTaken {
					track.status = Self::fed_initial(watched, settings);
				}
				(None, false)
			}
		}
	}

	/// The game window's port of a fed game (Legends of Runeterra's API, F1 25's and Forza Motorsport's telemetry):
	/// kept when it differs from the game's default, and the game heard again at it; true when it changed. A port
	/// below 1024 is refused (the system keeps those), and so is one another such game takes (its own or its default).
	pub fn set_port(&mut self, key: &str, port: u16, settings: &mut GamesSettings) -> Result<bool, String> {
		let watched = Watched::from_key(key).filter(|w| w.default_port().is_some());
		let watched = watched.ok_or_else(|| format!("games.port: {key} takes no port"))?;
		if port < 1024 {
			return Err(format!("games.port: {port} is below 1024"));
		}
		let other = Watched::named().iter().find(|w| **w != watched && port_of(settings, w) == Some(port));
		if let Some(other) = other {
			return Err(format!("games.port: {port} is {}'s", other.link()));
		}
		if port_of(settings, &watched) == Some(port) {
			return Ok(false);
		}
		if watched.default_port() == Some(port) {
			settings.ports.remove(key);
		} else {
			settings.ports.insert(key.to_owned(), port);
		}
		settings.heard.retain(|k| k != key);
		if let Some(track) = self.tracks.get_mut(key) {
			track.answered = None;
			track.status = Self::fed_initial(&watched, settings);
		}
		Ok(true)
	}
}

impl Feeds {
	/// A log game's or a file game's words at a look: its reader followed while the game runs heard (`follows`), and
	/// read once more at the first look after (`follow`); none for another game.
	fn read(&mut self, watched: &Watched, follows: bool) -> Vec<Word> {
		match watched {
			Watched::MtgArena => follow(&mut self.arena, follows, Arena::new, Arena::look),
			Watched::Hearthstone => {
				let look = |r: &mut Hearthstone| r.look(follows);
				follow(&mut self.hearthstone, follows, Hearthstone::default, look)
			}
			Watched::Elite => {
				let look = |r: &mut Elite| r.look(follows);
				follow(&mut self.elite, follows, Elite::default, look)
			}
			Watched::Mechabellum => follow(&mut self.mechabellum, follows, Mechabellum::new, Mechabellum::look),
			Watched::Tarkov => {
				let look = |r: &mut Tarkov| r.look(follows);
				follow(&mut self.tarkov, follows, Tarkov::default, look)
			}
			Watched::PathOfExile
			| Watched::Warframe
			| Watched::Deadlock
			| Watched::Fortnite
			| Watched::TeamFortress
			| Watched::Naraka
			| Watched::FallGuys
			| Watched::DeadByDaylight
			| Watched::Hunt
			| Watched::Valorant => {
				let slot = self.logs.entry(watched.link()).or_default();
				follow(slot, follows, || Logs::of(watched), Logs::look)
			}
			Watched::Pubg => follow(&mut self.pubg, follows, Pubg::new, Pubg::look),
			Watched::Halo
			| Watched::Brawlhalla
			| Watched::RFactor
			| Watched::LeMans
			| Watched::Osu
			| Watched::Stardew
			| Watched::Warcraft => {
				let slot = self.saves.entry(watched.link()).or_default();
				follow(slot, follows, || Saves::of(watched), Saves::look)
			}
			Watched::SlayTheSpire => follow(&mut self.spire, follows, Spire::default, Spire::look),
			Watched::Warships => follow(&mut self.warships, follows, Warships::new, Warships::look),
			Watched::Siege => follow(&mut self.siege, follows, Siege::new, Siege::look),
			_ => Vec::new(),
		}
	}
}

/// A reader kept while its game wants it, made at the first look that does and dropped once it no longer does.
fn keep<T>(slot: &mut Option<T>, on: bool, make: impl FnOnce() -> T) -> Option<&mut T> {
	if !on {
		*slot = None;
		return None;
	}
	Some(slot.get_or_insert_with(make))
}

/// A log's words at a look: its reader kept while its game runs heard (`follows`), made at the first look it does;
/// at the first look after, the reader reads once more (the lines the game wrote as it closed: an end written then
/// still ends its match before the close forgets it) and goes.
fn follow<T>(
	slot: &mut Option<T>,
	follows: bool,
	make: impl FnOnce() -> T,
	look: impl FnOnce(&mut T) -> Vec<Word>,
) -> Vec<Word> {
	if follows {
		return look(slot.get_or_insert_with(make));
	}
	slot.take().map(|mut reader| look(&mut reader)).unwrap_or_default()
}
