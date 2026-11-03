//! The match feeds of slice 48's games (docs/plan.md slice 48; docs/computer-games.md §6.2 and the formats' check of
//! 2026-10-19) and of those added on 2026-10-28 (Slay the Spire 2, Assetto Corsa, Guild Wars 2) and 2026-10-29 (the
//! close-only games' research, Rapha's): what each game's own
//! feed says, read into whether a match runs (`Fed`), the one thing that passes on. Pure: the desktop app's readers
//! (`src-tauri/src/games/`) poll the loopback, tail a log, listen on a loopback UDP port, copy a game's documented
//! shared memory, note a new file's time in its save folder or, for ELDEN RING and its NIGHTREIGN, bring down the
//! middle of the game's window (`banner`), and hand each answer, line, packet, page, time or band here. Each type below
//! holds only the fields it reads; every other field of an answer, a line or a packet (a player's name, a deck, a
//! score, a result, a car, a Steam id) is never looked at, kept or logged. Each machine tells an edge once: a reader
//! passes its words on as they come.

use serde::Deserialize;

use crate::games::Fed;

/// A little-endian `i32` at `at`, when the bytes hold one there.
fn i32_at(bytes: &[u8], at: usize) -> Option<i32> {
	bytes.get(at..at.checked_add(4)?).and_then(|b| b.try_into().ok()).map(i32::from_le_bytes)
}

/// StarCraft II's client API on `127.0.0.1:6119` (Blizzard's; StarCraft Casting Tool, sc2tools and sc2apiemulator read
/// it so): `/ui`'s `activeScreens` is empty during a match, or holds only a menu opened over it (`ScreenForeground…`,
/// sc2tools: "The match is still running underneath"); the loading screen comes before a match; any other screen (the
/// score screen, home, a lobby) is out of one. `/game`'s `isReplay` tells a replay from a match, and its `players[]`
/// whether one is loaded at all (the client answers an empty list in its menus): counted, each entry skipped unread, so
/// no name or result is ever looked at.
pub mod starcraft {
	use super::{Deserialize, Fed};

	/// The client's port, which no setting is known to change.
	pub const PORT: u16 = 6119;
	/// A menu or an alert over the running match.
	const OVER: &str = "ScreenForeground";
	const LOADING: &str = "ScreenLoading";

	/// What `/ui` shows.
	#[derive(Debug, Clone, Copy, PartialEq, Eq)]
	pub enum Screen {
		Match,
		Loading,
		Out,
	}

	#[derive(Deserialize)]
	struct Ui {
		#[serde(rename = "activeScreens")]
		active_screens: Vec<String>,
	}

	#[derive(Deserialize)]
	struct Game {
		#[serde(rename = "isReplay")]
		is_replay: bool,
		/// The players' entries, each skipped unread: their count alone.
		#[serde(default)]
		players: Vec<serde::de::IgnoredAny>,
	}

	/// `/ui`'s answer read; none when it is no such answer.
	#[must_use]
	pub fn screen(body: &[u8]) -> Option<Screen> {
		let ui: Ui = serde_json::from_slice(body).ok()?;
		let shown: Vec<&str> = ui.active_screens.iter().map(String::as_str).filter(|s| !s.starts_with(OVER)).collect();
		Some(match shown.as_slice() {
			[] => Screen::Match,
			[only] if only.starts_with(LOADING) => Screen::Loading,
			_ => Screen::Out,
		})
	}

	/// `/game` read: whether it is a match that plays (players listed, no replay); none when it is no such answer.
	#[must_use]
	pub fn plays(body: &[u8]) -> Option<bool> {
		serde_json::from_slice::<Game>(body).ok().map(|g| !g.is_replay && !g.players.is_empty())
	}

	/// A look's word: a match with players that is no replay plays, every screen but the loading one is out of a
	/// match; the loading screen, a replay and a game with no players say nothing.
	#[must_use]
	pub fn word(screen: Screen, plays: Option<bool>) -> Option<Fed> {
		match screen {
			Screen::Match => (plays == Some(true)).then_some(Fed::Playing),
			Screen::Out => Some(Fed::Ended),
			Screen::Loading => None,
		}
	}
}

/// Legends of Runeterra's Game Client API (Riot's docs/lor: "enabled locally on the player's computer using port
/// `21337` by default", which the game's settings may change or turn off): `/game-result`'s `GameID`, which "resets
/// every time the client restarts" and "is incremented after a game is completed". `LocalPlayerWon` is never read, and
/// no other endpoint is asked (`/positional-rectangles` holds the players' names).
pub mod runeterra {
	use super::{Deserialize, Fed};

	#[derive(Deserialize)]
	struct GameResult {
		#[serde(rename = "GameID")]
		game_id: i64,
	}

	/// `/game-result`'s `GameID`; none when it is no such answer.
	#[must_use]
	pub fn game_id(body: &[u8]) -> Option<i64> {
		serde_json::from_slice::<GameResult>(body).ok().map(|r| r.game_id)
	}

	/// The count of games since the client's start, as the polls saw it.
	#[derive(Debug, Default)]
	pub struct Games {
		last: Option<i64>,
	}

	impl Games {
		/// A poll's `GameID`: a rise is a game completed; the first count and a lower one (the client restarted) are
		/// only where the count stands.
		pub fn push(&mut self, id: i64) -> Option<Fed> {
			let before = self.last.replace(id);
			before.is_some_and(|b| id > b).then_some(Fed::Completed)
		}
	}
}

/// MTG Arena's `Player.log`, as the game writes it by default: the match door's state lines,
/// `STATE CHANGED {"old":"ConnectedToMatchDoor_ConnectingToGRE","new":"Playing"}` as a match starts and
/// `STATE CHANGED {"old":"Playing","new":"MatchCompleted"}` as it ends (once a match, a best-of-three's too), come with
/// Detailed Logs off (read in Rapha's log on 2026-11-03); with Detailed Logs on the game room's
/// `MatchGameRoomStateType_Playing` and `MatchGameRoomStateType_MatchCompleted` come too (17Lands' `mtga_follower.py`
/// and mtgatool's state names), plain substrings since that JSON has spaces in it, and a state told twice tells it
/// once (`Watcher::fed`). Its `DETAILED LOGS:` line near the top, written at each start whatever the setting, says the
/// log answered.
pub mod arena {
	/// What a line of the log says.
	#[derive(Debug, Clone, Copy, PartialEq, Eq)]
	pub enum Line {
		Playing,
		Completed,
		/// The log's header line: the game started and writes its log.
		Began,
	}

	/// A line's word; none for every other line.
	#[must_use]
	pub fn line(text: &str) -> Option<Line> {
		let state = changed_to(text);
		if state == Some("MatchCompleted") || text.contains("MatchGameRoomStateType_MatchCompleted") {
			Some(Line::Completed)
		} else if state == Some("Playing") || text.contains("MatchGameRoomStateType_Playing") {
			Some(Line::Playing)
		} else if text.contains("DETAILED LOGS:") {
			Some(Line::Began)
		} else {
			None
		}
	}

	/// The state a `STATE CHANGED` line's `"new"` names.
	fn changed_to(text: &str) -> Option<&str> {
		let (_, json) = text.split_once("STATE CHANGED")?;
		let (_, new) = json.split_once("\"new\"")?;
		let value = new.trim_start().strip_prefix(':')?.trim_start().strip_prefix('"')?;
		value.split_once('"').map(|(state, _)| state)
	}
}

/// Hearthstone's `Power.log` (hslog's tokens, HDT's and mello's readers): written once `log.config` holds a `[Power]`
/// section, which the game reads at its start. Only `GameState.` lines are read (`PowerTaskList.` repeats them later,
/// behind the animations): `CREATE_GAME` starts a game and the game entity's `tag=STATE value=COMPLETE` ends it; a game
/// watched as a spectator (from "Begin Spectating" or "Start Spectator" to "End Spectator") is no match.
pub mod hearthstone {
	use super::Fed;

	/// The section `log.config` needs, its header and its items as HDT and mello write them; the helper adds it only
	/// where no `[Power]` section stands (the file is the deck trackers' too).
	pub const HEADER: &str = "[Power]";
	pub const ITEMS: [&str; 5] =
		["LogLevel=1", "FilePrinting=True", "ConsolePrinting=False", "ScreenPrinting=False", "Verbose=True"];

	/// Whether a `log.config` holds a `[Power]` section, whatever its case and its spaces.
	#[must_use]
	pub fn has_section(text: &str) -> bool {
		text.lines().any(|l| l.trim().eq_ignore_ascii_case(HEADER))
	}

	/// The text that adds the section at the end of `text`: after a line end when its last line has none, in the
	/// file's own line ends (CRLF when it holds one).
	#[must_use]
	pub fn section(text: &str) -> String {
		let end = if text.contains("\r\n") { "\r\n" } else { "\n" };
		let mut out = String::new();
		if !text.is_empty() && !text.ends_with('\n') {
			out.push_str(end);
		}
		for line in std::iter::once(HEADER).chain(ITEMS) {
			out.push_str(line);
			out.push_str(end);
		}
		out
	}

	/// One log as its lines came: a new log starts a fresh one, so a spectate whose end the log before never wrote (the
	/// game crashed or quit) is over with it.
	#[derive(Debug, Default)]
	pub struct Power {
		spectating: bool,
	}

	impl Power {
		/// A line's word: a game created or completed while no game is watched as a spectator.
		pub fn push(&mut self, line: &str) -> Option<Fed> {
			if line.contains("Begin Spectating") || line.contains("Start Spectator") {
				self.spectating = true;
				return None;
			}
			if line.contains("End Spectator") {
				self.spectating = false;
				return None;
			}
			if self.spectating || !line.contains("GameState.") {
				return None;
			}
			if line.contains("CREATE_GAME") {
				Some(Fed::Playing)
			} else if line.contains("Entity=GameEntity") && line.contains("tag=STATE value=COMPLETE") {
				Some(Fed::Ended)
			} else {
				None
			}
		}
	}
}

/// Elite Dangerous' Player Journal (Frontier's Journal Manual v30: "a stream of information about gameplay events
/// which can be used by tools developers", "written in line-delimited JSON format"): a `MissionCompleted` event, the
/// closest thing to a quest's turn-in, ends a "match" that ran from the game's start or the mission before that
/// offered (slice 57, `games::Finish::began_at`): a mission completed under `match_min_s` after it offers nothing
/// (`games::Watcher::step`), so missions handed in together offer once. Each line is read for its `event` alone.
pub mod elite {
	use super::{Deserialize, Fed};

	#[derive(Deserialize)]
	struct Entry {
		event: String,
	}

	/// A journal line's word; none for every event but a mission completed.
	#[must_use]
	pub fn word(line: &str) -> Option<Fed> {
		if !line.contains("MissionCompleted") {
			return None;
		}
		let entry: Entry = serde_json::from_str(line).ok()?;
		(entry.event == "MissionCompleted").then_some(Fed::Completed)
	}
}

/// Mechabellum's Unity `Player.log` (Game River's game writes it by default, no setting asked; read on Rapha's PC on
/// 2026-10-28): the lobby's push of a match's start, `recv message [-5] - [PushMatchStart]`, starts a match, and the
/// server's push of its end, `recv message [-57] - [PushGameOver]`, ends it, once a match. The client's own
/// `match enter match over state` is not read: it also comes where no server match ran (a replay, a game against the
/// computer: two of eight in that check's earlier log, with no start or end pushed), and the stack traces under a push
/// name its handler (`OnPushMatchStart`), so a line counts only as a received message ending in the push's own
/// bracketed name. The log also holds other players' names, ids and records; every line but these two is passed over,
/// and of these nothing is kept.
pub mod mechabellum {
	use super::Fed;

	/// A line's word: a match's start or end pushed by the server; none for every other line.
	#[must_use]
	pub fn line(text: &str) -> Option<Fed> {
		if !text.contains("recv message") {
			return None;
		}
		if text.trim_end().ends_with("- [PushMatchStart]") {
			Some(Fed::Playing)
		} else if text.trim_end().ends_with("- [PushGameOver]") {
			Some(Fed::Ended)
		} else {
			None
		}
	}
}

/// A log read line by line: each line's word, a match's start or end; none for every other line. The log games added
/// on 2026-10-29 take their lines through it, each reader keeping what it must between lines (a map entered, a lobby
/// made) and nothing of a line itself.
pub trait Lines {
	fn push(&mut self, line: &str) -> Option<Fed>;
}

/// Path of Exile's and Path of Exile 2's `Client.txt` (`<install>\logs\Client.txt`, written by default; Grinding Gear
/// Games' developer docs: "Reading the game's log files is okay as long as the user is aware of what you are doing with
/// that data"; the trackers read the same line): only the client's debug line `Generating level <n> area "<id>" with
/// seed <n>` is read, for its area id alone (a chat line typing the same words is no debug line). A map's area (its id starts `Map`, in both games) arms; the next area generated that is no map (the
/// hideout, a town) completes it, a checkpoint as a mission handed in is (contract/links.json's `checkpoint`: it waits
/// `match_checkpoint_min` from the game's start or the last that offered, so the maps of a sitting offer about once
/// every 20 minutes). A map entered again through its portal generates nothing, so a trip back to the hideout and in
/// again is one map. The log also holds chat, whispers and other players' names; no other line is looked at.
pub mod pathofexile {
	use super::{Fed, Lines};

	const GENERATING: &str = "] Generating level ";
	/// The client's own debug lines, never a chat line ("[INFO Client …] #name: …"), which anyone may type.
	const DEBUG: &str = "[DEBUG Client ";
	const AREA: &str = " area \"";

	/// The area a line generates, by its id; none for every other line.
	fn area(line: &str) -> Option<&str> {
		let (before, after) = line.split_once(GENERATING)?;
		if !before.contains(DEBUG) {
			return None;
		}
		let rest = after.split_once(AREA)?.1;
		let (id, after) = rest.split_once('"')?;
		after.starts_with(" with seed ").then_some(id)
	}

	/// Whether an area id is a campaign zone's: Path of Exile's `<part>_<act>_<zone>` (`1_1_2`), Path of Exile 2's
	/// `G<act>_<zone>` (`G1_2`); never a town's (`…_town`).
	fn campaign(id: &str) -> bool {
		let mut parts = id.split('_');
		let first = parts.next().unwrap_or_default();
		let numbered = |part: &str| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit());
		let zone = numbered(first.strip_prefix('G').unwrap_or(first)) && parts.next().is_some();
		zone && !id.ends_with("_town")
	}

	/// Whether an area id is a place the player goes back to between zones: a town (`1_1_town`, `G1_town`) or a hideout
	/// (`HideoutTemplarLab`).
	fn rest(id: &str) -> bool {
		id.ends_with("_town") || id.starts_with("Hideout")
	}

	/// Whether a map's area was generated and no other area since, or a campaign zone's and no town or hideout since.
	#[derive(Debug, Default)]
	pub struct Areas {
		in_map: bool,
		in_campaign: bool,
	}

	impl Lines for Areas {
		fn push(&mut self, line: &str) -> Option<Fed> {
			let id = area(line)?;
			if id.starts_with("Map") {
				self.in_map = true;
				return None;
			}
			if campaign(id) {
				self.in_campaign = true;
				return None;
			}
			let map = std::mem::take(&mut self.in_map);
			let zones = rest(id) && std::mem::take(&mut self.in_campaign);
			(map || zones).then_some(Fed::Completed)
		}
	}
}

/// Warframe's `EE.log` (`%LOCALAPPDATA%\Warframe\EE.log`, written anew at each launch; Digital Extremes: "If you use
/// external software in conjunction with Warframe, then you do so at your own risk"; the mission trackers read the
/// same lines): a mission's start (`ThemedSquadOverlay.lua: Mission name:` or `MissionIntro.lua: MissionName:`) arms,
/// and its end, whichever comes first (`EndOfMatch.lua: GiveMissionRewards. success=true`, `EndOfMatch.lua: Mission
/// Failed`, `TopMenu.lua: Abort`, or the ship's layout generated again, `/Lotus/Levels/Proc/PlayerShip generating
/// layout`), completes it: a checkpoint, missions being minutes long (`checkpoint`, as Path of Exile's). Every line is
/// looked at only for those tokens, among the script lines (`Script [Info]:`) and the ship's layout line.
pub mod warframe {
	use super::{Fed, Lines};

	const SCRIPT: &str = "Script [Info]: ";
	const STARTS: [&str; 2] = ["ThemedSquadOverlay.lua: Mission name:", "MissionIntro.lua: MissionName:"];
	const ENDS: [&str; 3] =
		["EndOfMatch.lua: GiveMissionRewards. success=true", "EndOfMatch.lua: Mission Failed", "TopMenu.lua: Abort"];
	const SHIP: &str = "/Lotus/Levels/Proc/PlayerShip generating layout";

	/// Whether a mission started and has not ended.
	#[derive(Debug, Default)]
	pub struct Missions {
		in_mission: bool,
	}

	impl Lines for Missions {
		fn push(&mut self, line: &str) -> Option<Fed> {
			let script = line.split_once(SCRIPT).map(|(_, rest)| rest);
			if script.is_some_and(|s| STARTS.iter().any(|t| s.starts_with(t))) {
				self.in_mission = true;
				return None;
			}
			let ended = script.is_some_and(|s| ENDS.iter().any(|t| s.contains(t))) || line.contains(SHIP);
			(ended && std::mem::take(&mut self.in_mission)).then_some(Fed::Completed)
		}
	}
}

/// Escape from Tarkov's logs (`<install>\Logs\log_<date>_<version>\`, a folder each session; `TarkovMonitor` reads the
/// same tokens): `application|GameStarted` in the session's `application` log, the raid begun (for a PMC at the end of
/// its countdown, for a scav at once), arms, and `Got notification | UserMatchOver` in its `notifications` log, the raid
/// left (extracted, killed or run through), ends it. The notification's JSON after the token (the map, the raid's id)
/// is never parsed; every other line (the profile and account ids among them) is passed over.
pub mod tarkov {
	use super::{Fed, Lines};

	/// A line's word, from either log.
	#[derive(Debug, Default)]
	pub struct Raid;

	impl Lines for Raid {
		fn push(&mut self, line: &str) -> Option<Fed> {
			if line.contains("application|GameStarted") {
				Some(Fed::Playing)
			} else if line.contains("Got notification | UserMatchOver") {
				Some(Fed::Ended)
			} else {
				None
			}
		}
	}
}

/// Deadlock's console log (`<library>\steamapps\common\Deadlock\game\citadel\console.log`, written only with the
/// launch option `-condebug`, which the player adds in Steam): a lobby made for a match (`Lobby <lobby> for Match
/// <match> created`) and then `OnGameStateChanged: GameInProgress (7)` arm a match; `OnGameStateChanged: PostGame (8)`
/// after that lobby, or the lobby destroyed (`Lobby … for Match … destroyed`), ends it. A local bot match (`Local bot
/// match: Init.`) makes no lobby and so arms nothing, and a game state changing with no lobby (a broadcast watched) is
/// passed over; `[Server]` lines (a server run on this computer) and network messages
/// (`NETWORK_DISCONNECT_LOOPDEACTIVATE` among them) end nothing. The lobby's and the match's ids are never kept.
pub mod deadlock {
	use super::{Fed, Lines};

	/// Whether a match's lobby stands.
	#[derive(Debug, Default)]
	pub struct Lobby {
		open: bool,
	}

	impl Lines for Lobby {
		fn push(&mut self, line: &str) -> Option<Fed> {
			if line.contains("[Server]") || line.contains("Local bot match: Init.") {
				return None;
			}
			let lobby = line.contains("Lobby ") && line.contains(" for Match ");
			if lobby && line.trim_end().ends_with(" created") {
				self.open = true;
				return None;
			}
			if lobby && line.trim_end().ends_with(" destroyed") {
				self.open = false;
				return Some(Fed::Ended);
			}
			if !self.open {
				return None;
			}
			if line.contains("OnGameStateChanged: GameInProgress (7)") {
				Some(Fed::Playing)
			} else if line.contains("OnGameStateChanged: PostGame (8)") {
				Some(Fed::Ended)
			} else {
				None
			}
		}
	}
}

/// Fortnite's own log (`%LOCALAPPDATA%\FortniteGame\Saved\Logs\FortniteGame.log`, written anew at each start, the last
/// one kept beside it under another name; Epic: "do not officially support the use of third-party software; use them at
/// your own risk"): `LogNet: Welcomed by server` (a match's server took the player, never the lobby's own map) arms,
/// and `LogNet: Browse: /Game/Maps/Frontend` (back in the lobby) ends it. Every other line (the account's name and ids
/// among them) is passed over.
pub mod fortnite {
	use super::{Fed, Lines};

	const FRONTEND: &str = "/Game/Maps/Frontend";

	/// A line's word.
	#[derive(Debug, Default)]
	pub struct Matches;

	impl Lines for Matches {
		fn push(&mut self, line: &str) -> Option<Fed> {
			if line.contains("LogNet: Welcomed by server") && !line.contains(FRONTEND) {
				Some(Fed::Playing)
			} else if line.contains("LogNet: Browse: ") && line.contains(FRONTEND) {
				Some(Fed::Ended)
			} else {
				None
			}
		}
	}
}

/// Team Fortress 2's console log (`<library>\steamapps\common\Team Fortress 2\tf\console.log`, written only with the
/// launch option `-condebug`, as Deadlock's): a matchmade match's lobby (`Lobby created`, after `[PartyClient] Entering
/// queue`) arms, and its end ends it, whichever comes first: `Lobby destroyed`, `Sending request to abandon current
/// match`, `GS marked assigned match as ended` or `Thanks for Playing Mann vs. Machine.`. A chat line (a player's name,
/// then ` :  `, then the message) is skipped whole, so nothing typed in the chat starts or ends a match; a community
/// server's game makes no lobby and is heard at the game's close.
pub mod teamfortress {
	use super::{Fed, Lines};

	const ENDS: [&str; 3] = [
		"Sending request to abandon current match",
		"GS marked assigned match as ended",
		"Thanks for Playing Mann vs. Machine.",
	];

	/// A line's word.
	#[derive(Debug, Default)]
	pub struct Lobby;

	impl Lines for Lobby {
		fn push(&mut self, line: &str) -> Option<Fed> {
			if line.contains(" :  ") {
				return None;
			}
			let line = line.trim_end();
			if line.ends_with("Lobby created") {
				Some(Fed::Playing)
			} else if line.ends_with("Lobby destroyed") || ENDS.iter().any(|t| line.contains(t)) {
				Some(Fed::Ended)
			} else {
				None
			}
		}
	}
}

/// NARAKA: BLADEPOINT's Unity `Player.log` (`%USERPROFILE%\AppData\LocalLow\24Entertainment\Naraka\Player.log`, written
/// by default; the battle states as github.com/ViewSuSu/BlackGoldAncientSword's `BattleStateMachine.cs` reads them,
/// 2026-10-23, its one source, unverified in a live match): a line holding `DoHideTeamOffLoadingPage` (the loading page
/// hidden as a battle begins) or `TeamBattle Init` arms a battle, and `TeamBattle Destroy` or `GridMapManager Destroy`
/// ends it. A start while a battle is armed says nothing. Every other line is passed over.
pub mod naraka {
	use super::{Fed, Lines};

	const STARTS: [&str; 2] = ["DoHideTeamOffLoadingPage", "TeamBattle Init"];
	const ENDS: [&str; 2] = ["TeamBattle Destroy", "GridMapManager Destroy"];

	/// Whether a battle began and has not ended.
	#[derive(Debug, Default)]
	pub struct Battles {
		in_battle: bool,
	}

	impl Lines for Battles {
		fn push(&mut self, line: &str) -> Option<Fed> {
			if STARTS.iter().any(|t| line.contains(t)) {
				return (!std::mem::replace(&mut self.in_battle, true)).then_some(Fed::Playing);
			}
			let ended = ENDS.iter().any(|t| line.contains(t));
			(ended && std::mem::take(&mut self.in_battle)).then_some(Fed::Ended)
		}
	}
}

/// Fall Guys' Unity `Player.log` (`%USERPROFILE%\AppData\LocalLow\Mediatonic\FallGuys_client\Player.log`, written by
/// default; the lines github.com/yuma-dev/setsu's `fall_guys.rs` and `ShootMe/FallGuysStats` read): a show's first round
/// going live (`[GameSession] Changing state from Countdown to Playing`) arms it, its later rounds saying nothing, and
/// the show's end (`== [CompletedEpisodeDto] ==`, its rewards) or the main menu loaded before it (`[StateMainMenu]
/// Loading scene MainMenu`, a show left early) ends it. Every other line (the player's and the others' names among
/// them) is passed over.
pub mod fallguys {
	use super::{Fed, Lines};

	const PLAYING: &str = "[GameSession] Changing state from Countdown to Playing";
	const ENDS: [&str; 2] = ["== [CompletedEpisodeDto] ==", "[StateMainMenu] Loading scene MainMenu"];

	/// Whether a show's round went live and the show has not ended.
	#[derive(Debug, Default)]
	pub struct Shows {
		in_show: bool,
	}

	impl Lines for Shows {
		fn push(&mut self, line: &str) -> Option<Fed> {
			if line.contains(PLAYING) {
				return (!std::mem::replace(&mut self.in_show, true)).then_some(Fed::Playing);
			}
			let ended = ENDS.iter().any(|t| line.contains(t));
			(ended && std::mem::take(&mut self.in_show)).then_some(Fed::Ended)
		}
	}
}

/// Dead by Daylight's Unreal log (`%LOCALAPPDATA%\DeadByDaylight\Saved\Logs\DeadByDaylight.log`, written by default and
/// anew at each start; the lines github.com/nickyramone/stabd's `MainLogProcessor.java` reads, unverified on the current
/// build): the trial's map loaded (`GameFlow: ACollectable::BeginPlay`) arms a trial, and its end (`GameFlow:
/// ADBDGameState::SetGameLevelEnded`) ends it; an end with no trial armed (a build whose start line changed) completes
/// one, sized from the game's start or its last finish. The end line's reason and every other line (the server's
/// address and the other players among them) are passed over.
pub mod deadbydaylight {
	use super::{Fed, Lines};

	const START: &str = "GameFlow: ACollectable::BeginPlay";
	const END: &str = "GameFlow: ADBDGameState::SetGameLevelEnded";

	/// Whether a trial's map loaded and the trial has not ended.
	#[derive(Debug, Default)]
	pub struct Trials {
		in_trial: bool,
	}

	impl Lines for Trials {
		fn push(&mut self, line: &str) -> Option<Fed> {
			if line.contains(START) {
				return (!std::mem::replace(&mut self.in_trial, true)).then_some(Fed::Playing);
			}
			if !line.contains(END) {
				return None;
			}
			Some(if std::mem::take(&mut self.in_trial) { Fed::Ended } else { Fed::Completed })
		}
	}
}

/// Hunt: Showdown 1896's `Game.log` (`<library>\steamapps\common\Hunt Showdown 1896\USER\Game.log`, written by default;
/// the mission bag's lines as github.com/diegomm27/HuntLogger's `evidence.py` reads them, its one source, unverified in a
/// live match): a line naming `CMetaMissionBag` with `MissionStarted` arms a mission, and one naming it with
/// `ContentsDumped` (its bag emptied as the mission ends) ends it, both in any case. Every other line is passed over.
pub mod hunt {
	use super::{Fed, Lines};

	const BAG: &str = "cmetamissionbag";
	const STARTED: &str = "missionstarted";
	const DUMPED: &str = "contentsdumped";

	/// Whether a mission started and has not ended.
	#[derive(Debug, Default)]
	pub struct Missions {
		in_mission: bool,
	}

	impl Lines for Missions {
		fn push(&mut self, line: &str) -> Option<Fed> {
			let line = line.to_ascii_lowercase();
			if !line.contains(BAG) {
				return None;
			}
			if line.contains(STARTED) {
				return (!std::mem::replace(&mut self.in_mission, true)).then_some(Fed::Playing);
			}
			(line.contains(DUMPED) && std::mem::take(&mut self.in_mission)).then_some(Fed::Ended)
		}
	}
}

/// VALORANT's Unreal log (`%LOCALAPPDATA%\VALORANT\Saved\Logs\ShooterGame.log`, written by default and anew at each
/// start, the last one kept beside it as `ShooterGame-backup-…`): only `LogGameFlowStateManager` lines are read, for the
/// state they move to (`Transitioning from <state> to <state>`, `Reconcile called with state: <state> and new state:
/// <state>` or `Broadcasting state changed to <state>`), in any case, as github.com/kirmitboi1123/ValSpot's
/// `GameLog.cs` and github.com/Tanveer-exe/VibePause read them (their casing unverified against a live log). `InGame`
/// arms a match, but not in the Range (the last `/Game/GameModes/` path any line named holds `ShootingRange`, kept as
/// that one bool); `PostMatch`, `WaitingToPostMatch`, `TransitionToPostMatch`, `TransitionToMainMenu` or `MainMenu` ends
/// an armed one. The log also holds Riot's servers' addresses with the player's id; no other line is looked at, and of
/// these nothing is kept.
pub mod valorant {
	use super::{Fed, Lines};

	const FLOW: &str = "LogGameFlowStateManager";
	/// The words before the state a flow line moves to, in lower case.
	const TO: [&str; 2] = [" to ", " and new state: "];
	const MODES: &str = "/game/gamemodes/";
	const RANGE: &str = "shootingrange";
	const IN_GAME: &str = "ingame";
	const ENDS: [&str; 5] =
		["postmatch", "waitingtopostmatch", "transitiontopostmatch", "transitiontomainmenu", "mainmenu"];

	/// The state a flow line moves to, in lower case; none for every other line.
	fn destination(line: &str) -> Option<String> {
		if !line.contains(FLOW) {
			return None;
		}
		let lower = line.to_ascii_lowercase();
		let at = TO.iter().filter_map(|w| lower.find(w).map(|i| i + w.len())).max()?;
		let state: String = lower.get(at..)?.chars().take_while(char::is_ascii_alphanumeric).collect();
		(!state.is_empty()).then_some(state)
	}

	/// Whether a match is armed, and whether the last game mode named was the Range.
	#[derive(Debug, Default)]
	pub struct Flow {
		armed: bool,
		range: bool,
	}

	impl Lines for Flow {
		fn push(&mut self, line: &str) -> Option<Fed> {
			let lower = line.to_ascii_lowercase();
			if let Some((_, mode)) = lower.split_once(MODES) {
				self.range = mode.split_whitespace().next().is_some_and(|m| m.contains(RANGE));
			}
			let state = destination(line)?;
			if state == IN_GAME {
				return (!self.range && !std::mem::replace(&mut self.armed, true)).then_some(Fed::Playing);
			}
			(ENDS.contains(&state.as_str()) && std::mem::take(&mut self.armed)).then_some(Fed::Ended)
		}
	}
}

/// World of Warships' battles as its own `replays` folder shows them (replays are recorded by default): the game writes
/// `tempArenaInfo.json` there as a battle begins and deletes it as the battle ends, leaving a new `*.wowsreplay` file
/// (its `temp.wowsreplay` while the battle runs). Neither is ever opened: each look hands over whether the battle file
/// is there and the newest finished replay's modified time, from the folder's listing. The battle file appearing after
/// the first look arms (one found at the first look may be a battle already running or one a Steam build left behind,
/// and arms nothing); its going ends the armed battle; a new replay ends an armed battle too, or, where none was armed
/// (that stale file, a battle running as the reader began), completes one; the replay that follows a battle's end is
/// that battle's and says nothing.
pub mod warships {
	use super::Fed;

	/// The battle file's name and the replays' extension in the folder.
	pub const ARENA: &str = "tempArenaInfo.json";
	pub const EXTENSION: &str = "wowsreplay";
	/// The replay the game writes while a battle runs, never a finished one.
	pub const RUNNING: &str = "temp.wowsreplay";

	/// Where the battles stand.
	#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
	enum Battle {
		/// No look yet.
		#[default]
		Unseen,
		/// No battle armed.
		Out,
		/// A battle whose file appeared runs.
		Armed,
		/// A battle ended by its file's going: its replay is still to come.
		Owed,
	}

	/// The folder as the looks saw it.
	#[derive(Debug, Default)]
	pub struct Battles {
		battle: Battle,
		arena: bool,
		replay: Option<i64>,
	}

	impl Battles {
		/// A look: whether the battle file is there, and the newest finished replay's modified time (unix ms).
		pub fn push(&mut self, arena: bool, replay: Option<i64>) -> Vec<Fed> {
			let mut words = Vec::new();
			let (appeared, gone) = (arena && !self.arena, !arena && self.arena);
			self.arena = arena;
			if self.battle == Battle::Unseen {
				self.battle = Battle::Out;
				self.replay = replay;
				return words;
			}
			if appeared {
				self.battle = Battle::Armed;
				words.push(Fed::Playing);
			} else if gone && self.battle == Battle::Armed {
				self.battle = Battle::Owed;
				words.push(Fed::Ended);
			}
			if replay > self.replay {
				self.replay = replay;
				match std::mem::replace(&mut self.battle, Battle::Out) {
					Battle::Armed => words.push(Fed::Ended),
					Battle::Owed => {}
					Battle::Out | Battle::Unseen => words.push(Fed::Completed),
				}
			}
			words
		}
	}
}

/// Rainbow Six Siege's match replays (`<install>\MatchReplay\Match-<date>_<time>-<n>\`, a folder each match with a
/// `.rec` file each round, written while the match is played): never opened, only the folders' and files' times from
/// their listings. A match folder newer than any at the first look arms (its minutes from its appearance); the match
/// ends once no round file under it changed for `QUIET_MS` while the game runs, or a newer match folder appears (which
/// arms the next), or the game closes (the watcher ends an armed match there).
pub mod siege {
	use super::Fed;

	/// The round files' extension and the match folders' names' start.
	pub const EXTENSION: &str = "rec";
	pub const MATCH: &str = "Match-";
	/// How long no round file changes, once a round was written, before the match is over: a round's end, its score
	/// and the next round's operator pick take less. Before its first round file a match never ends by quiet (its
	/// loading, bans and first pick can take longer).
	pub const QUIET_MS: i64 = 90_000;

	/// The match folders as the looks saw them.
	#[derive(Debug, Default)]
	pub struct Matches {
		looked: bool,
		/// The newest match folder's creation time (unix ms).
		newest: Option<i64>,
		/// Whether a match is armed, and its newest round file's time (unix ms) once one was written.
		armed: bool,
		last: Option<i64>,
	}

	impl Matches {
		/// A look at `now` (unix ms): the newest match folder's creation time and its newest round file's modified
		/// time.
		pub fn push(&mut self, now: i64, folder: Option<i64>, round: Option<i64>) -> Vec<Fed> {
			let mut words = Vec::new();
			if !std::mem::replace(&mut self.looked, true) {
				self.newest = folder;
				return words;
			}
			if folder > self.newest {
				self.newest = folder;
				if std::mem::replace(&mut self.armed, true) {
					words.push(Fed::Ended);
				}
				self.last = round;
				words.push(Fed::Playing);
				return words;
			}
			if !self.armed {
				return words;
			}
			self.last = self.last.max(round);
			if self.last.is_some_and(|last| now - last >= QUIET_MS) {
				(self.armed, self.last) = (false, None);
				words.push(Fed::Ended);
			}
			words
		}
	}
}

/// A folder of files a game saves, as the looks saw it: the newest file's modified time alone, handed over by the
/// desktop app from the folders' listings, never a file opened, nor a file's or a folder's name kept. The first look
/// sets where the folder stands, and a newer file after it is something completed (`Completed`), once however many came
/// in the look, which ends a "match" from the game's start or the last that offered (slice 57, `games::Finish::began_at`;
/// Rapha, 2026-10-28: a match whose length its feed does not tell counts the game's time open since its last end). Slay
/// the Spire 2's run history (`spire`) and since 2026-10-29 Halo: The Master Chief Collection's multiplayer carnage
/// reports, Brawlhalla's replays, rFactor 2's and Le Mans Ultimate's results, osu!'s replays, Stardew Valley's saves and
/// Warcraft III's replays (the desktop app's `saves.rs`).
#[derive(Debug, Default)]
pub struct Newer {
	/// The newest file's modified time seen (unix ms), and whether the first look set it.
	newest: Option<i64>,
	began: bool,
}

impl Newer {
	/// A look's newest file's modified time (unix ms; none while the folder holds none): the first look sets where the
	/// folder stands, and a newer one after it is something completed, once however many came in the look.
	pub fn push(&mut self, newest: Option<i64>) -> Option<Fed> {
		if !std::mem::replace(&mut self.began, true) {
			self.newest = newest;
			return None;
		}
		let newer = newest.filter(|n| self.newest.is_none_or(|before| *n > before))?;
		self.newest = Some(newer);
		Some(Fed::Completed)
	}
}

/// PUBG: BATTLEGROUNDS' replays (`%LOCALAPPDATA%\TslGame\Saved\Demos\`, a folder each match the game records, its
/// `PUBG.replayinfo` written as the match ends; github.com/hoangvu12/Hako's `pubg/watch.rs` and github.com/Segergren/
/// Segra's `PubgIntegration.cs` watch the same; whether replays are recorded by default is unverified): never opened,
/// only the newest folder's creation time and whether that file is in it, from the listings. A folder newer than any at
/// the first look arms a match (an armed one before it ends first), and its `PUBG.replayinfo` appearing ends it.
pub mod pubg {
	use super::Fed;

	/// The file a match's folder gets at its end.
	pub const INFO: &str = "PUBG.replayinfo";

	/// The match folders as the looks saw them.
	#[derive(Debug, Default)]
	pub struct Matches {
		looked: bool,
		/// The newest match folder's creation time (unix ms).
		newest: Option<i64>,
		armed: bool,
	}

	impl Matches {
		/// A look: the newest match folder's creation time, and whether its end file is there.
		pub fn push(&mut self, folder: Option<i64>, done: bool) -> Vec<Fed> {
			let mut words = Vec::new();
			if !std::mem::replace(&mut self.looked, true) {
				self.newest = folder;
				return words;
			}
			if folder > self.newest {
				self.newest = folder;
				if std::mem::replace(&mut self.armed, true) {
					words.push(Fed::Ended);
				}
				words.push(Fed::Playing);
			}
			if done && std::mem::take(&mut self.armed) {
				words.push(Fed::Ended);
			}
			words
		}
	}
}

/// F1 25's UDP telemetry (EA's "F1 25 Telemetry" specification: "all values are encoded using Little Endian format.
/// All data is packed"), which the player points at 127.0.0.1 and a port with UDP Telemetry on and UDP Broadcast Mode
/// off, in one of the UDP formats read here: 2025, 2024 (EA's "F1 24 UDP Specification") and 2026 (the 2026 Season
/// Pack's, as github.com/MacManley/f1-26-udp reads it: "F1 26 will only support the previous 2 UDP formats"). The three
/// keep alike what is read: the 29-byte header's `m_packetFormat`, `m_packetId` and `m_sessionUID`; a Session packet's
/// `m_sessionType` at byte 35 (15-17 a race; 18 a time trial); an Event packet's code right after the header (`SEND`,
/// Session Ended); a Final Classification packet ("Once at the end of a race"), packet ids 1, 3 and 8. F1 23's format
/// (2023) numbers its sessions otherwise (10 a race) and is left out, with every older one: a datagram whose header
/// names such a format (`other_format`) is the game heard in a format to change. A race starts at its session's first
/// packet and ends at its classification or its session's end, once a session (the `m_sessionUID`, kept in memory
/// alone); a practice, a qualifying or a time trial is no match. The Participants packet (names, "the Steam Id on
/// PC") and every other field are never read.
pub mod formula {
	use super::Fed;

	/// The UDP formats read here, and the years an older or a newer F1 format names (F1 2018's on).
	const FORMATS: [u16; 3] = [2024, 2025, 2026];
	const YEARS: std::ops::RangeInclusive<u16> = 2018..=2035;
	/// The header of F1 22's format and older ones, the shortest an F1 datagram starts with.
	const OLD_HEADER: usize = 24;
	const HEADER: usize = 29;
	const SESSION: u8 = 1;
	const EVENT: u8 = 3;
	const CLASSIFICATION: u8 = 8;
	/// The Session packet's `m_sessionType`, after the header's 29 bytes, the weather, both temperatures, the laps and
	/// the track's length.
	const SESSION_TYPE_AT: usize = 35;
	/// The Event packet's four-letter code, right after the header.
	const CODE_AT: usize = HEADER;
	const RACES: std::ops::RangeInclusive<u8> = 15..=17;

	/// A datagram's `m_packetFormat`, its first two bytes.
	fn format(packet: &[u8]) -> Option<u16> {
		packet.get(..2).and_then(|b| b.try_into().ok()).map(u16::from_le_bytes)
	}

	/// Whether a datagram is F1's telemetry in a format read here: the game is heard.
	#[must_use]
	pub fn ours(packet: &[u8]) -> bool {
		packet.len() >= HEADER && format(packet).is_some_and(|f| FORMATS.contains(&f))
	}

	/// Whether a datagram looks like F1's telemetry in another UDP format (F1 23's 2023, an older one): its header's
	/// format names an F1 year that is not read here. The game is heard, set to a format to change.
	#[must_use]
	pub fn other_format(packet: &[u8]) -> bool {
		packet.len() >= OLD_HEADER && format(packet).is_some_and(|f| YEARS.contains(&f) && !FORMATS.contains(&f))
	}

	/// The race sessions as their packets came.
	#[derive(Debug, Default)]
	pub struct Race {
		/// The race session that plays, and the last that ended.
		race: Option<u64>,
		ended: Option<u64>,
	}

	impl Race {
		/// A datagram's word; none for one of another format or another kind.
		pub fn push(&mut self, packet: &[u8]) -> Option<Fed> {
			if !ours(packet) {
				return None;
			}
			let uid = u64::from_le_bytes(packet.get(7..15)?.try_into().ok()?);
			match *packet.get(6)? {
				SESSION => {
					let race = RACES.contains(packet.get(SESSION_TYPE_AT)?);
					let new = race && self.race != Some(uid) && self.ended != Some(uid);
					new.then(|| {
						self.race = Some(uid);
						Fed::Playing
					})
				}
				EVENT if packet.get(CODE_AT..CODE_AT + 4) == Some(b"SEND".as_slice()) => self.end(uid),
				CLASSIFICATION => self.end(uid),
				_ => None,
			}
		}

		fn end(&mut self, uid: u64) -> Option<Fed> {
			(self.race == Some(uid)).then(|| {
				self.race = None;
				self.ended = Some(uid);
				Fed::Ended
			})
		}
	}
}

/// Forza Motorsport's Data Out (Forza's support page: "`S32 IsRaceOn; // = 1 when race is on. = 0 when in
/// menus/race stopped`", the Sled format's 232 bytes and the 2023 Dash format's 331, as forza-data-web reads them):
/// `IsRaceOn`, the first four bytes, alone. A race plays once `IsRaceOn` held 1 for `ARM_MS` and ends once it held 0
/// for `END_MS`, or once no datagram came for `END_MS` after it (the game stops sending in some menus and when Data Out
/// goes off: a short pause in the race or a restart's load is no end); a datagram of another size (Forza Horizon's
/// 324, another program's) is not read.
pub mod forza {
	use super::{Fed, i32_at};

	const SIZES: [usize; 2] = [232, 331];
	/// How long `IsRaceOn` holds 1 before a race plays, and 0 before it ended.
	pub const ARM_MS: i64 = 10_000;
	pub const END_MS: i64 = 30_000;

	/// Whether a datagram is Forza Motorsport's Data Out in a format read here: the game is heard.
	#[must_use]
	pub fn ours(packet: &[u8]) -> bool {
		SIZES.contains(&packet.len())
	}

	/// The race as the datagrams told it.
	#[derive(Debug, Default)]
	pub struct Race {
		on_since: Option<i64>,
		off_since: Option<i64>,
		/// When the last datagram read came.
		last: Option<i64>,
		playing: bool,
	}

	impl Race {
		/// A datagram at `at` (unix ms): its word, once a hold passed.
		pub fn push(&mut self, packet: &[u8], at: i64) -> Option<Fed> {
			if !ours(packet) {
				return None;
			}
			self.last = Some(at);
			if i32_at(packet, 0)? == 1 {
				self.off_since = None;
				let since = *self.on_since.get_or_insert(at);
				if !self.playing && at - since >= ARM_MS {
					self.playing = true;
					return Some(Fed::Playing);
				}
				None
			} else {
				self.on_since = None;
				self.off_since.get_or_insert(at);
				self.idle(at)
			}
		}

		/// The clock at `at` with no datagram: a race whose `IsRaceOn` went 0, or whose datagrams stopped, ends once
		/// `END_MS` passed.
		pub fn idle(&mut self, at: i64) -> Option<Fed> {
			let quiet = |since: Option<i64>| since.is_some_and(|since| at - since >= END_MS);
			let ended = self.playing && (quiet(self.off_since) || quiet(self.last));
			ended.then(|| {
				// a race that goes on after the silence arms again from its next datagram
				self.playing = false;
				self.on_since = None;
				Fed::Ended
			})
		}
	}
}

/// iRacing's SDK in its shared memory `Local\IRSDKMemMapFileName` (iRacing.com's `irsdk_defines.h`, as iRon and
/// pyirsdk read it): the 112-byte header's version (2), `status` (`irsdk_stConnected`), variable count and offset and
/// its rotating buffers (the newest by `tickCount`); the variables' headers (144 bytes each: type, offset, name),
/// looked through for two by name and type, the `SessionState` integer (Racing 4) and the `IsReplayPlaying` bool; and
/// of the newest buffer only those two values. `read` asks for each of those byte ranges alone (the desktop app copies
/// each from the view), never the session's YAML with the drivers' names. A race plays from Racing while no replay
/// plays and ends when the sim leaves its session: `status` without connected, or the mapping gone, and as a rule the
/// sim's close, since iRacing runs its sim one session at a time and its process exits as the player leaves (the close
/// ends an armed match: `games::Watcher`). Checkered and `CoolDown` end nothing: a practice, a qualifying and the race
/// of one event are one match, sized by its own minutes from its first Racing state to the sim leaving the session
/// (slice 57, `games::Finish::began_at`; a test drive and the race the sim starts again for within `match_relaunch_s`
/// go on as one, `games::Watcher`'s rejoined run).
pub mod iracing {
	use super::{Fed, i32_at};

	const VERSION: i32 = 2;
	const CONNECTED: i32 = 1;
	/// `irsdk_bool` and `irsdk_int`, the variables' types read.
	const BOOL: i32 = 1;
	const INT: i32 = 2;
	/// The header's length: ten integers, two of padding, and four buffers' 16 bytes each.
	const HEADER: usize = 112;
	const VAR_HEADER: usize = 144;
	const NAME_AT: usize = 16;
	const NAME_LEN: usize = 32;
	const BUFS_AT: usize = 48;
	const BUF: usize = 16;
	const MAX_BUFS: usize = 4;
	/// The most variables' headers read (the sim lists a few hundred).
	const MAX_VARS: usize = 2048;
	const RACING: i32 = 4;

	/// What the view says of the sim.
	#[derive(Debug, Clone, Copy, PartialEq, Eq)]
	pub enum Sim {
		/// Not in a session (`status` without `irsdk_stConnected`).
		Off,
		/// In a session: its `SessionState`, and whether a replay plays.
		State { session: i32, replay: bool },
	}

	/// The view read through `bytes(at, len)`, a copy of that range of it (none past its end); none when it is no SDK
	/// view this reader knows.
	#[must_use]
	pub fn read(mut bytes: impl FnMut(usize, usize) -> Option<Vec<u8>>) -> Option<Sim> {
		let header = bytes(0, HEADER)?;
		let at = |n: usize| i32_at(&header, n);
		if at(0)? != VERSION {
			return None;
		}
		if at(4)? & CONNECTED == 0 {
			return Some(Sim::Off);
		}
		let vars = usize::try_from(at(24)?).ok()?.min(MAX_VARS);
		let heads = bytes(usize::try_from(at(28)?).ok()?, vars.checked_mul(VAR_HEADER)?)?;
		let offset_of = |wanted: &[u8], kind: i32| {
			(0..vars).map(|i| i * VAR_HEADER).find_map(|h| {
				let name = heads.get(h + NAME_AT..h + NAME_AT + NAME_LEN)?;
				let name = name.split(|&b| b == 0).next().unwrap_or(name);
				let offset = i32_at(&heads, h + 4).and_then(|o| usize::try_from(o).ok());
				(name == wanted && i32_at(&heads, h) == Some(kind)).then_some(offset).flatten()
			})
		};
		let state_at = offset_of(b"SessionState", INT)?;
		let replay_at = offset_of(b"IsReplayPlaying", BOOL);
		let bufs = usize::try_from(at(32)?).ok()?.min(MAX_BUFS);
		let newest = (0..bufs).map(|b| BUFS_AT + b * BUF).max_by_key(|&b| at(b).unwrap_or(i32::MIN))?;
		let buf = usize::try_from(at(newest + 4)?).ok()?;
		let session = i32_at(&bytes(buf.checked_add(state_at)?, 4)?, 0)?;
		let replay = replay_at
			.and_then(|r| bytes(buf.checked_add(r)?, 1))
			.and_then(|b| b.first().copied())
			.is_some_and(|b| b != 0);
		Some(Sim::State { session, replay })
	}

	/// The session as the looks saw it.
	#[derive(Debug, Default)]
	pub struct Session {
		racing: bool,
	}

	impl Session {
		/// A look's word.
		pub fn push(&mut self, sim: Sim) -> Option<Fed> {
			match sim {
				Sim::State { session: RACING, replay: false } if !self.racing => {
					self.racing = true;
					Some(Fed::Playing)
				}
				Sim::Off if self.racing => {
					self.racing = false;
					Some(Fed::Ended)
				}
				Sim::State { .. } | Sim::Off => None,
			}
		}
	}
}

/// Slay the Spire 2's run history (ststracker.app's About page: the `.run` files of
/// `%APPDATA%\SlayTheSpire2\steam\<SteamID>\profile1\saves\history`, which its uploader takes, one a finished run):
/// a run file newer than any the history held when the reader began is a run completed, read by `Newer`. The desktop
/// app hands over each look's newest modified time alone: no file is opened, and neither a file's name nor a folder's
/// (the Steam id) is kept.
pub mod spire {
	/// A run file's extension.
	pub const EXTENSION: &str = "run";
}

/// Guild Wars 2's `MumbleLink` shared memory (ArenaNet's, for Mumble's positional audio and the overlays: the GW2
/// wiki's API:MumbleLink, read on 2026-10-28 as Gw2Sharp's `Gw2LinkedMem` and `MapType` lay it out, the wiki itself
/// being down for maintenance that day): the `LinkedMem`'s game name (`wchar_t name[256]` at byte 44, "Guild Wars 2")
/// and its context's `mapType` (a `uint32` at byte 1140: the context at 1108, its 28-byte server address, `mapId`,
/// then `mapType`), alone. A competitive player-versus-player map (2) plays; leaving it for any other map (Heart of
/// the Mists, the lobby, is a public map, 5, as the official API's `/v2/maps/350` says) ends the match; a redirect (0,
/// a login into a running match) says nothing. The identity (its JSON holds the character's name), the positions and
/// every other field are never read.
pub mod guildwars {
	use super::{Fed, i32_at};

	/// Where the game's name lies, and its length in UTF-16 units.
	pub const NAME_AT: usize = 44;
	const NAME: &str = "Guild Wars 2";
	/// Where the context's `mapType` lies.
	pub const MAP_TYPE_AT: usize = 1140;
	const REDIRECT: i32 = 0;
	const PVP: i32 = 2;

	/// The link read through `bytes(at, len)`, a copy of that range of it: its `mapType`; none when it is not Guild
	/// Wars 2's (another program's link, or the game before it filled it).
	#[must_use]
	pub fn map_type(mut bytes: impl FnMut(usize, usize) -> Option<Vec<u8>>) -> Option<i32> {
		let units = NAME.encode_utf16().count();
		let name = bytes(NAME_AT, units * 2)?;
		let ours = name.as_chunks::<2>().0.iter().map(|u| u16::from_le_bytes(*u)).eq(NAME.encode_utf16());
		if !ours {
			return None;
		}
		i32_at(&bytes(MAP_TYPE_AT, 4)?, 0)
	}

	/// The match as the looks saw it.
	#[derive(Debug, Default)]
	pub struct Match {
		pvp: bool,
	}

	impl Match {
		/// A look's `mapType`: its word.
		pub fn push(&mut self, map_type: i32) -> Option<Fed> {
			match map_type {
				PVP if !self.pvp => {
					self.pvp = true;
					Some(Fed::Playing)
				}
				REDIRECT | PVP => None,
				_ if self.pvp => {
					self.pvp = false;
					Some(Fed::Ended)
				}
				_ => None,
			}
		}
	}
}

/// Assetto Corsa Competizione's shared memory `Local\acpmf_graphics` (Kunos' `SPageFileGraphic`, as PyAccSharedMemory
/// and pyacc read it): its `status` (OFF 0, REPLAY 1, LIVE 2, PAUSE 3) and `session` (RACE 2), the page's second and
/// third integers, alone (`READ`, the bytes the desktop app copies). A race plays once it is LIVE and ends at OFF or
/// REPLAY, or once a LIVE session is no longer a race (the next session of the weekend); a pause never ends it, and a
/// practice, a qualifying or a hotlap is no match. Assetto Corsa (the first, 2026-10-28) writes the same page under the
/// same name, its first three integers alike: `AC_STATUS` (OFF 0, REPLAY 1, LIVE 2, PAUSE 3) and `AC_SESSION_TYPE`
/// (RACE 2; its hotlap, time attack, drift and drag sessions are no match), as mdjarv's assettocorsasharedmemory and
/// Crew Chief's `ACSData.cs` read them, so one reader serves both; its sim (`acs.exe`) runs one session and closes as
/// the player leaves it, which ends an armed race at the close (`games::Watcher`).
pub mod acc {
	use super::{Fed, i32_at};

	/// The page's first three integers, `packetId`, `status` and `session`.
	pub const READ: usize = 12;
	const OFF: i32 = 0;
	const REPLAY: i32 = 1;
	const LIVE: i32 = 2;
	const RACE: i32 = 2;

	/// A copy of the page's start read: its `status` and `session`.
	#[must_use]
	pub fn graphics(page: &[u8]) -> Option<(i32, i32)> {
		Some((i32_at(page, 4)?, i32_at(page, 8)?))
	}

	/// The race as the looks saw it.
	#[derive(Debug, Default)]
	pub struct Race {
		racing: bool,
	}

	impl Race {
		/// A look's word.
		pub fn push(&mut self, (status, session): (i32, i32)) -> Option<Fed> {
			if status == LIVE && session == RACE && !self.racing {
				self.racing = true;
				return Some(Fed::Playing);
			}
			let left = matches!(status, OFF | REPLAY) || (status == LIVE && session != RACE);
			if left && self.racing {
				self.racing = false;
				return Some(Fed::Ended);
			}
			None
		}
	}
}

/// Automobilista 2's shared memory (`$pcars2$`, the Project CARS 2 layout Reiza's game publishes once the player sets
/// Options, System, Shared Memory to "Project CARS 2"; its `SharedMemory.h`, as github.com/viper4gh/CREST2-AMS2 (version
/// 14) and `SeriousOldMan/Simulator-Controller` (version 13) carry it, read on 2026-10-29): the struct's first five
/// `unsigned int`s, `mVersion`, `mBuildVersionNumber`, `mGameState`, `mSessionState` and `mRaceState`, of which the
/// last three alone are read (`READ`, the bytes the desktop app copies). A race plays once the game is
/// `GAME_INGAME_PLAYING` (2) in a `SESSION_RACE` (5) whose `mRaceState` is `RACESTATE_RACING` (2); it ends at
/// `RACESTATE_FINISHED` (3), `RACESTATE_DISQUALIFIED` (4), `RACESTATE_RETIRED` (5) or `RACESTATE_DNF` (6), at the front
/// end (`GAME_EXITED` 0, `GAME_FRONT_END` 1, `GAME_FRONT_END_REPLAY` 7), or once the session is no longer a race; a pause,
/// a menu with the clock running and a restart end nothing, and a practice or a qualifying is no match. The
/// participants, the car and every other field are never read.
pub mod automobilista {
	use super::{Fed, i32_at};

	/// The struct's first five integers.
	pub const READ: usize = 20;
	const FRONT_END: [i32; 3] = [0, 1, 7];
	const PLAYING: i32 = 2;
	const RACE: i32 = 5;
	const RACING: i32 = 2;
	const OVER: [i32; 4] = [3, 4, 5, 6];

	/// A copy of the struct's start read: its `mGameState`, `mSessionState` and `mRaceState`.
	#[must_use]
	pub fn states(page: &[u8]) -> Option<(i32, i32, i32)> {
		Some((i32_at(page, 8)?, i32_at(page, 12)?, i32_at(page, 16)?))
	}

	/// The race as the looks saw it.
	#[derive(Debug, Default)]
	pub struct Race {
		racing: bool,
	}

	impl Race {
		/// A look's word.
		pub fn push(&mut self, (game, session, race): (i32, i32, i32)) -> Option<Fed> {
			if game == PLAYING && session == RACE && race == RACING && !self.racing {
				self.racing = true;
				return Some(Fed::Playing);
			}
			let left = FRONT_END.contains(&game) || OVER.contains(&race) || (game == PLAYING && session != RACE);
			if left && self.racing {
				self.racing = false;
				return Some(Fed::Ended);
			}
			None
		}
	}
}

/// ELDEN RING's and ELDEN RING NIGHTREIGN's boss banners, read off the game's own window (2026-10-29, Rapha: "what would
/// be highest roi first and have almost no risk? do that then only"; docs/computer-games.md §6.2): neither game writes a
/// log, a file or a feed at a boss's death, so the desktop app captures the middle of the game's window while it runs
/// (Windows alone), at most `RATE` frames a second, and hands each frame's band, brought down to `WIDTH` pixels wide, to
/// `look`. A boss's death shows a gold banner across a dark strip at the window's middle ("ENEMY FELLED", "GREAT ENEMY
/// FELLED", "DEMIGOD FELLED", "LEGEND FELLED", "GOD SLAIN"; Nightreign's "NIGHTLORD FELLED"), a death a red one ("YOU
/// DIED"), never a kill, and a site of grace a gold one too ("LOST GRACE DISCOVERED"), which the text tells apart. The
/// colour test is github.com/MaximilianRau04/elden-ring-bosstracker's (`death_detector`: a dark share and a gold share
/// gate a frame before its OCR), with the strip's shape added: the gold rows lie in one strip `STRIP` high near the
/// window's middle, and the gold columns centre on it. A frame that passes on `FRAMES` frames in a row is read by the
/// system's OCR (`Banners`), whose words must hold a kill word (`KILL_WORDS`); without OCR nothing is read (the desktop
/// app starts no capture, and the game is heard at its close). A kill is a checkpoint (`Fed::Completed`), and nothing tells
/// for `COOLDOWN_MS` after one. Pure: no frame, band or word is kept, and the text passes only into `kill`.
pub mod banner {
	/// The band's top and bottom, as shares of the client area's height: the banner's dark strip runs across the
	/// window's middle.
	pub const TOP: f64 = 0.38;
	pub const BOTTOM: f64 = 0.62;
	/// The band's width once brought down, in pixels (its height follows the window's shape).
	pub const WIDTH: usize = 640;
	/// The frames a second the capture reads at most.
	pub const RATE: u32 = 4;
	/// A pixel is dark below this luma (Rec. 601, 0-255), and the band's dark share must reach `DARK_SHARE`: the
	/// banner's strip behind the words.
	const DARK_LUMA: i32 = 90;
	const DARK_SHARE: f64 = 0.30;
	/// A pixel is gold where its red and green both stand this far above its blue, and the band's gold share must pass
	/// `GOLD_SHARE`.
	const GOLD: i32 = 60;
	const GOLD_SHARE: f64 = 0.0008;
	/// A pixel is red where its red stands this far above the mean of its green and blue; red that outnumbers gold is
	/// "YOU DIED".
	const RED: i32 = 45;
	/// The gold rows' strip (from its gold's 5th to its 95th percentile) is this high, as shares of the window's height
	/// (the words are about 4-8 % of it, with room for their glow), and its middle this near the window's.
	const STRIP: (f64, f64) = (0.03, 0.10);
	const MIDDLE: f64 = 0.05;
	/// The gold columns' centre lies this near the band's middle, as a share of its width.
	const SIDE: f64 = 0.10;
	/// The frames in a row a banner shows before the OCR reads it.
	const FRAMES: u32 = 2;
	/// How often the OCR reads a banner that still shows, and how long nothing tells after a kill (ms).
	const READ_EVERY_MS: i64 = 500;
	pub const COOLDOWN_MS: i64 = 10_000;
	/// The kill banners' last word, in the languages a source names them in: English "FELLED" and "SLAIN" (the
	/// bosstracker's `felled_phrases`, "ENEMY FELLED" and "GREAT ENEMY FELLED"; the Fextralife wiki's Bosses page),
	/// German "GEFALLEN" (the bosstracker's "GEGNER GEFALLEN" and "GROSSER GEGNER GEFALLEN"). No source found names
	/// the French, Spanish, Italian, Portuguese, Polish or Russian banners (2026-10-29): a kill shown in those is heard
	/// at the game's close alone.
	pub const KILL_WORDS: [&str; 3] = ["FELLED", "SLAIN", "GEFALLEN"];
	/// The least a read word matches a kill word: 1 less their Levenshtein distance over the longer word's length.
	const WORD_MATCH: f64 = 0.8;

	/// What one frame's band shows.
	#[derive(Debug, Clone, Copy, PartialEq, Eq)]
	pub enum Look {
		Nothing,
		/// A gold banner across a dark strip at the window's middle: a kill, or a grace found.
		Banner,
		/// A red banner: "YOU DIED".
		Died,
	}

	/// A band's look: `rgb` holds `width` × `height` pixels, three bytes each in R, G, B order, row after row, the band
	/// from `TOP` to `BOTTOM` of the window's height.
	#[must_use]
	pub fn look(rgb: &[u8], width: usize, height: usize) -> Look {
		let pixels = width * height;
		if pixels == 0 || rgb.len() < pixels * 3 {
			return Look::Nothing;
		}
		let (mut dark, mut red) = (0usize, 0usize);
		// the gold pixels by row, and the sum of their columns
		let mut rows = vec![0usize; height];
		let mut columns = 0usize;
		for (at, px) in rgb.as_chunks::<3>().0.iter().take(pixels).enumerate() {
			let [rd, gr, bl] = px.map(i32::from);
			if (299 * rd + 587 * gr + 114 * bl) / 1000 < DARK_LUMA {
				dark += 1;
			}
			if rd.min(gr) - bl > GOLD {
				rows[at / width] += 1;
				columns += at % width;
			} else if rd - i32::midpoint(gr, bl) > RED {
				red += 1;
			}
		}
		let gold: usize = rows.iter().sum();
		if ratio(dark, pixels) < DARK_SHARE {
			return Look::Nothing;
		}
		if red > gold && ratio(red, pixels) > GOLD_SHARE {
			return Look::Died;
		}
		if ratio(gold, pixels) <= GOLD_SHARE {
			return Look::Nothing;
		}
		// the rows holding the gold's 5th and 95th percentiles
		let at = |part: usize| {
			let mut seen = 0;
			rows.iter().position(|n| {
				seen += n;
				seen * 100 > gold * part
			})
		};
		let (Some(first), Some(last)) = (at(5), at(95)) else { return Look::Nothing };
		// a band row's place in the window's height
		let row = |r: usize| TOP + (BOTTOM - TOP) * ratio(r, height);
		let (top, bottom) = (row(first), row(last + 1));
		let strip = (STRIP.0..=STRIP.1).contains(&(bottom - top)) && (f64::midpoint(top, bottom) - 0.5).abs() <= MIDDLE;
		let centred = (ratio(columns, gold * width) - 0.5).abs() <= SIDE;
		if strip && centred { Look::Banner } else { Look::Nothing }
	}

	/// `n` over `of`, 0 where `of` is.
	#[expect(clippy::cast_precision_loss, reason = "pixel counts far below 2^52")]
	fn ratio(n: usize, of: usize) -> f64 {
		if of == 0 { 0.0 } else { n as f64 / of as f64 }
	}

	/// Whether the OCR's text holds a kill word: each word in capitals matched against `KILL_WORDS` (`WORD_MATCH`); a
	/// word of fewer than four letters never is.
	#[must_use]
	pub fn kill(text: &str) -> bool {
		text.split(|c: char| !c.is_alphabetic()).filter(|w| w.chars().count() >= 4).any(|word| {
			let word: Vec<char> = word.to_uppercase().chars().collect();
			KILL_WORDS.iter().any(|kill| {
				let kill: Vec<char> = kill.chars().collect();
				1.0 - ratio(distance(&word, &kill), word.len().max(kill.len())) >= WORD_MATCH
			})
		})
	}

	/// The Levenshtein distance between two words.
	fn distance(from: &[char], to: &[char]) -> usize {
		let mut row: Vec<usize> = (0..=to.len()).collect();
		for (i, letter) in from.iter().enumerate() {
			let mut diagonal = row[0];
			row[0] = i + 1;
			for (j, other) in to.iter().enumerate() {
				let above = row[j + 1];
				row[j + 1] = (above + 1).min(row[j] + 1).min(diagonal + usize::from(letter != other));
				diagonal = above;
			}
		}
		row[to.len()]
	}

	/// What the capture does with a frame's look.
	#[derive(Debug, Clone, Copy, PartialEq, Eq)]
	pub enum Step {
		Wait,
		/// Read the band with the OCR and hand its text to `Banners::read`.
		Read,
		/// A boss fell, by the gate alone (no OCR).
		Fell,
	}

	/// The banners as the frames showed them: how many frames in a row showed one, when the OCR last read one and when
	/// the last kill was told, each at its frame's time (ms on a steady clock).
	#[derive(Debug, Default)]
	pub struct Banners {
		streak: u32,
		read_at: Option<i64>,
		fell_at: Option<i64>,
	}

	impl Banners {
		/// A frame's look at `at`: a banner on `FRAMES` frames in a row is read (at most every `READ_EVERY_MS` while it
		/// shows); nothing within `COOLDOWN_MS` of a kill.
		pub fn push(&mut self, look: Look, at: i64) -> Step {
			if look != Look::Banner || self.fell_at.is_some_and(|fell| at - fell < COOLDOWN_MS) {
				self.streak = 0;
				return Step::Wait;
			}
			self.streak += 1;
			if self.streak < FRAMES || self.read_at.is_some_and(|read| at - read < READ_EVERY_MS) {
				return Step::Wait;
			}
			self.read_at = Some(at);
			Step::Read
		}

		/// The OCR's text of the band read at `at`: whether it is a kill (`kill`), which starts the cooldown.
		pub fn read(&mut self, text: &str, at: i64) -> bool {
			let fell = kill(text);
			if fell {
				self.told(at);
			}
			fell
		}

		fn told(&mut self, at: i64) {
			self.fell_at = Some(at);
			self.streak = 0;
		}
	}
}
