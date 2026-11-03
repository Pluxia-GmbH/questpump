//! Pump home's recent games (2026-10-27, Rapha: "maybe show here the last 3 games i played and the amount of quest
//! pumps i collected in each"; David, 2026-10-27: "Instead of pumps count moves please"; docs/ui.md, "Pump home on the
//! computer"): the games played on this computer, each with its moves done this game week (from Monday's turn, the
//! streak board's week) and in all (`PlayedSpan`, the player's This week | All time, settings.json's
//! `desk.played_span`; 2026-10-28), every game running now first (marked Playing, played before or not, its count shown
//! too), then the others ranked by the count shown, the most first (Rapha, 2026-10-28: "show it ranked with highest
//! left"; the latest played up to three in all, until then): every game with a move this week, every game running now
//! and every game played in the last `PLAYED_S`, and in All time every game with a move. A named game runs for the row
//! while any of its own programs does (`Games::open`: League's client too, never a store's launcher). World of Warcraft
//! counts as a game: its moves are those done in the game's own pumps (`history::Pump::of_game`, never a device's
//! catch-up nor an app pump the save took in), each in the week of its own time, which the Saga's build hands over
//! (`Recent::wow`); it was played while a client ran or when its latest such move was, and its icon is its client's
//! program's, made as a game's is (`Games::set_wow`). Another game was played while the games' look saw it running
//! (`Games::running`, the watched games), and its moves are every move done in the pumps started from its match offer,
//! on any device or here (`offer_taken`), each pump in the week it began, counted once a move of it is done
//! (2026-10-27: a pump banked, recalled or left before its first move counts for nothing, nor does an offer a pump done
//! another way answered). No row names its game, so a pump is known as the app pump (`history::app_pumps`, any
//! device's) that began when the offer was taken (`TAKEN_BEFORE_S` before to `TAKEN_AFTER_S` after, the nearest),
//! looked for at each Saga build until `TAKEN_S` passed (`Recent::taken`, this run's alone), its moves as the latest
//! Saga build counts them (`Recent::moves`), and this computer keeps its own record (`Played`, in settings.json's
//! `desk.played`): each game's key, when it was last seen running, when its pumps began, kept `KEEP_S`, and the moves
//! of its pumps past that (`Played::before`), `KEPT` games at most besides those with a move, never synced, logged,
//! counted or sent anywhere. World of Warcraft's moves in all are those of every pump of the game's own the Saga's
//! build holds. A game shows only while it is a tile on the Games tab (a game put away with Not a game, or a program
//! removed, does not), or none of them before the first one played.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::history::{Pump, WEEK_S};
use crate::games::{Games, Wow};
use crate::runtime::Helper;
use crate::settings::PlayedSpan;

/// World of Warcraft's key among the games played (never a contract/links.json key, nor a found game's).
pub(crate) const WOW: &str = "wow"; // copy-lint: not shown (a key)

/// How lately a game without a move this week was played for Pump home to show it: a week.
const PLAYED_S: i64 = WEEK_S;

/// How long a game played, a pump of its and a move done in World of Warcraft are kept: four weeks, past any game week
/// Pump home counts.
const KEEP_S: i64 = 28 * 24 * 60 * 60;

/// The most games kept without a move: the one played longest ago goes first (a game with a move stays, for All time).
const KEPT: usize = 16;

/// How long before and after an offer was taken the pump that answered it began: a device's sync names the offer it
/// took after its pump began (later still when it was out of reach), this computer's pump begins as it takes it.
const TAKEN_BEFORE_S: i64 = 10 * 60;
const TAKEN_AFTER_S: i64 = 2 * 60;

/// How long a taken offer waits for a move of its pump: past it, the pump counts for nothing.
const TAKEN_S: i64 = 2 * 60 * 60;

/// A game played on this computer (settings.json's `desk.played`).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub(crate) struct Played {
	/// The game's key: `Watched::id` (a named game's contract/links.json key, a found or picked program's own), or
	/// `WOW`.
	pub(crate) key: String,
	/// When it was last seen running, unix seconds.
	pub(crate) last: i64,
	/// When each of its pumps with a move done began, unix seconds, oldest first (none for World of Warcraft, whose
	/// rows hold its moves); the field's name stays for the settings already written.
	#[serde(default)]
	pub(crate) pumps: Vec<i64>,
	/// The moves done in its pumps that went from `pumps` past `KEEP_S`, so All time keeps them (2026-10-28; none in a
	/// record written before, which kept none older).
	#[serde(default)]
	pub(crate) before: usize,
}

/// The recent games' state in the loop beside settings.json (a field of `Home`).
#[derive(Default)]
pub(crate) struct Recent {
	/// The games running at the last look (`Games::open`), `WOW` among them while a client runs: settings.json is
	/// written when they change, and at the quit (`recent_seen`).
	running: Vec<String>,
	/// World of Warcraft's moves done in the last `KEEP_S`, by their rows' own times, newest first: the Saga's build
	/// hands them over (`desk/log.rs`).
	pub(crate) wow: Vec<i64>,
	/// World of Warcraft's moves done in all the game's own pumps the Saga's build holds.
	wow_all: usize,
	/// The moves done in each kept pump of another game (`Played::pumps`), by its start, as the latest Saga build
	/// counts them (`history::app_pumps`), so a pump's later moves count as they land.
	moves: HashMap<i64, usize>,
	/// The match offers taken whose pump has no move done yet: the game's key and when it was taken (unix seconds).
	taken: Vec<(String, i64)>,
}

/// A game on Pump home's row, as its tile on the Games tab names it.
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub(crate) struct RecentGame {
	/// `Watched::id`, or `WOW`: a click opens its window on the Games tab.
	key: String,
	/// World of Warcraft, which the page names in its own words and whose click opens This computer.
	wow: bool,
	/// A found or picked game's own name; none for a named game (the page names it by `links.name.<key>`) and for
	/// World of Warcraft.
	name: Option<String>,
	/// Its icon's asset URL (the tile's, or World of Warcraft's, `games/icons.rs`), while its file stands.
	icon: Option<String>,
	/// It runs now: its game, or another of its own programs (`Games::open`).
	playing: bool,
	/// Its moves done this game week.
	moves: usize,
	/// Its moves done in all.
	all: usize,
}

/// Pump home's recent games as the player set This week | All time: the span the row counts and its games.
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub(crate) struct RecentGames {
	span: PlayedSpan,
	games: Vec<RecentGame>,
}

/// The list kept tidy at `now`: with the latest Saga build's `moves` (each kept pump's, by its start) the pumps older
/// than `KEEP_S` go, their moves added to `before` (a pump the build no longer holds had none); then the games neither
/// played nor pumped since and with no move go, and past `KEPT` those played longest ago without a move.
fn prune(played: &mut Vec<Played>, now: i64, moves: Option<&HashMap<i64, usize>>) {
	let since = now - KEEP_S;
	if let Some(moves) = moves {
		for p in played.iter_mut() {
			let old: usize =
				p.pumps.iter().filter(|&&at| at < since).map(|at| moves.get(at).copied().unwrap_or(0)).sum();
			p.before += old;
			p.pumps.retain(|&at| at >= since);
		}
	}
	played.retain(|p| p.last >= since || !p.pumps.is_empty() || p.before > 0);
	played.sort_by_key(|p| std::cmp::Reverse(p.last));
	let mut without = 0;
	played.retain(|p| {
		if !p.pumps.is_empty() || p.before > 0 {
			return true;
		}
		without += 1;
		without <= KEPT
	});
}

/// The entry of `key`, made when it is new (played at `now`).
fn entry<'a>(played: &'a mut Vec<Played>, key: &str, now: i64) -> &'a mut Played {
	let at = played.iter().position(|p| p.key == key).unwrap_or_else(|| {
		played.push(Played { key: key.to_owned(), last: now, pumps: Vec::new(), before: 0 });
		played.len() - 1
	});
	&mut played[at]
}

impl Helper {
	/// The games running now (the games' look's, any of a named game's own programs, `Games::open`; `WOW` while a World
	/// of Warcraft client runs) played at unix second `now`; whether they changed since the last look.
	pub(crate) fn recent_seen(&mut self, now: i64) -> bool {
		let mut running: Vec<String> = self.games.as_ref().map(|g| g.open().to_vec()).unwrap_or_default();
		if self.game.is_some() {
			running.push(WOW.to_owned());
		}
		let played = &mut self.settings.desk.played;
		for key in &running {
			entry(played, key, now).last = now;
		}
		let changed = running != self.home.recent.running;
		self.home.recent.running = running;
		changed
	}

	/// The loop's look (every 2 s, after the games'): the games running played now, kept in settings.json as one
	/// starts or closes (the quit keeps the last moment, `wind_down`), and World of Warcraft handed to the games' icons
	/// while Pump home may show it.
	pub(crate) fn recent_look(&mut self, now: i64) {
		if self.recent_seen(now) {
			prune(&mut self.settings.desk.played, now, None);
			self.save_settings();
		}
		let known = self.game.is_some()
			|| !self.home.recent.wow.is_empty()
			|| self.settings.desk.played.iter().any(|p| p.key == WOW);
		let addons = self.game.as_ref().map(|g| &g.addons).or(self.followed.as_ref());
		let wow = known.then(|| Wow {
			// the install's flavour folder, above its Interface/AddOns
			root: addons.and_then(|a| a.parent()?.parent()).map(std::path::Path::to_path_buf),
			pid: self.game.as_ref().map(|g| g.pid),
		});
		if let Some(games) = self.games.as_mut() {
			games.set_wow(wow);
		}
	}

	/// What Pump home's recent games take from the Saga's build at unix second `now`: World of Warcraft's moves
	/// (`game_pumps`, the game's rows' pumps: the moves of the game's own done in the last `KEEP_S`, and their number in
	/// all), each taken offer's pump with a move done among `app_pumps` (every device's), kept for its game once, a taken
	/// offer past `TAKEN_S` dropped, and the moves done in each kept pump, those of a pump past `KEEP_S` added to its
	/// game's `before` as it goes. Whether Pump home changed (it is published again then).
	pub(crate) fn recent_rows(&mut self, game_pumps: &[Pump], app_pumps: &[Pump], now: i64) -> bool {
		let since = now - KEEP_S;
		let mut wow: Vec<i64> = game_pumps
			.iter()
			.filter(|p| p.of_game())
			.flat_map(|p| p.moves.iter().map(|m| m.ts))
			.filter(|&at| at >= since)
			.collect();
		wow.sort_unstable_by_key(|&at| std::cmp::Reverse(at));
		let wow_all = game_pumps.iter().filter(|p| p.of_game()).map(Pump::moves_done).sum();
		let mut changed = self.home.recent.wow != wow || self.home.recent.wow_all != wow_all;
		self.home.recent.wow = wow;
		self.home.recent.wow_all = wow_all;
		let mut counted = false;
		for (game, at) in std::mem::take(&mut self.home.recent.taken) {
			let began = app_pumps
				.iter()
				.filter(|p| !p.moves.is_empty() && (at - TAKEN_BEFORE_S..=at + TAKEN_AFTER_S).contains(&p.id))
				.min_by_key(|p| (p.id - at).abs())
				.map(|p| p.id);
			match began {
				Some(began) => {
					let p = entry(&mut self.settings.desk.played, &game, began);
					p.last = p.last.max(began);
					p.pumps.push(began);
					p.pumps.sort_unstable();
					counted = true;
				}
				None if now - at <= TAKEN_S => self.home.recent.taken.push((game, at)),
				None => {}
			}
		}
		let kept: Vec<i64> = self.settings.desk.played.iter().flat_map(|p| p.pumps.iter().copied()).collect();
		let mut moves: HashMap<i64, usize> = HashMap::new();
		for p in app_pumps.iter().filter(|p| kept.contains(&p.id)) {
			moves.entry(p.id).or_insert_with(|| p.moves_done());
		}
		// a pump past KEEP_S goes into its game's `before` with the moves this build counts
		if counted || kept.iter().any(|&at| at < now - KEEP_S) {
			prune(&mut self.settings.desk.played, now, Some(&moves));
			self.save_settings();
			changed = true;
		}
		changed |= self.home.recent.moves != moves;
		self.home.recent.moves = moves;
		changed
	}

	/// The match offer of the game `game` (`Watched::id`) was taken at unix second `now` (a pump started from it on a
	/// device or here): its pump's moves count once a move of it is done (`recent_rows`, at the next Saga build). Caller:
	/// `offer_taken`, once an offer.
	pub(crate) fn recent_taken(&mut self, game: String, now: i64) {
		self.home.recent.taken.push((game, now));
		self.log_publish();
	}

	/// Pump home's recent games at unix second `now` (`RecentGames`), each a tile on the Games tab or World of Warcraft:
	/// every one with a move this game week, every one running now, played before or not (Rapha, 2026-10-27: a game
	/// running always shows), every one played within `PLAYED_S`, and in All time every one with a move, however many
	/// (Pump home's row scrolls); every game running first (Rapha, 2026-10-28: "running games leftmost"), then the most
	/// moves in the span the player set, then the latest played; none before the first one played. The loop's look
	/// publishes Pump home every 2 s, after the games' look that finds them running, so a start or a close shows at
	/// once.
	pub(crate) fn recent_games(&self, now: i64) -> RecentGames {
		let span = self.settings.desk.played_span;
		let week = self.home.days.day.monday(now);
		let since = now - PLAYED_S;
		let tiles = self.games.as_ref().map(|g| g.lists(&self.settings.games).tiles).unwrap_or_default();
		let open: &[String] = self.games.as_ref().map_or(&[], Games::open);
		let played = &self.settings.desk.played;
		let recent = &self.home.recent;
		let wow_seen = played.iter().find(|p| p.key == WOW).map(|p| p.last);
		// a game with moves in all from before any kept look counts as played longest ago
		let wow_last = wow_seen
			.max(recent.wow.first().copied())
			.or(self.game.is_some().then_some(now))
			.or((recent.wow_all > 0).then_some(0));
		let wow = wow_last.map(|last| {
			let game = RecentGame {
				key: WOW.to_owned(),
				wow: true,
				name: None,
				icon: self.games.as_ref().and_then(Games::wow_icon),
				playing: self.game.is_some(),
				moves: recent.wow.iter().filter(|&&at| at >= week).count(),
				all: recent.wow_all,
			};
			(last, game)
		});
		// a tile running now counts as played now, before the look's record of it lands
		let others = tiles.iter().filter_map(|tile| {
			let p = played.iter().find(|p| p.key == tile.key);
			let playing = open.contains(&tile.key);
			let last = if playing { Some(now) } else { p.map(|p| p.last) }?;
			let moves = |from: i64| -> usize {
				p.map_or(0, |p| p.pumps.iter().filter(|&&at| at >= from).filter_map(|at| recent.moves.get(at)).sum())
			};
			let game = RecentGame {
				key: tile.key.clone(),
				wow: false,
				name: tile.name.clone(),
				icon: tile.icon.clone(),
				playing,
				moves: moves(week),
				all: p.map_or(0, |p| p.before) + moves(i64::MIN),
			};
			Some((last, game))
		});
		let count = |g: &RecentGame| match span {
			PlayedSpan::Week => g.moves,
			PlayedSpan::All => g.all,
		};
		let mut games: Vec<(i64, RecentGame)> = wow
			.into_iter()
			.chain(others)
			.filter(|(last, g)| g.playing || g.moves > 0 || *last >= since || (span == PlayedSpan::All && g.all > 0))
			.collect();
		games.sort_by_key(|(last, g)| (!g.playing, std::cmp::Reverse(count(g)), std::cmp::Reverse(*last)));
		RecentGames { span, games: games.into_iter().map(|(_, g)| g).collect() }
	}
}
