//! The Games tab (docs/plan.md slices 56 and 57; docs/ui.md, "The match-over card"): the one place for the games on the
//! computer and the offers after them, its topic `games` (which replaced the Games on this computer page's `menu.games`
//! and the Offers page's `menu.offers`), the tray's offer items, and the intents of the `games` and `card` areas (the
//! card's own in `card.rs`). The module keeps its name, `offers.rs`, for the offers it holds.
//!
//! `games` is `{now: {state: "paused" | "ready" | "playing" | "watching" | "none", game: {key, name, icon} | null,
//! count, offer: {id, name, kind} | null}, pause, night, tiles: [Tile], more: [Tile], hidden: [{key, name}], picks,
//! pick_games, pace, open_on_close, notice, held, riot, dot}`. `now`: the Now card's state (paused while a pause holds,
//! else ready while a game's offer stands and nothing holds it, the condition the tray's item shows on,
//! `Helper::offer_ready`, slice 57, else playing while a watched game runs, else watching while one is on, else none)
//! with the game it names (while ready the offer's, else the first running in the tab's order, `Lists::playing`, or the
//! one watched when only one is: its key, its name as a tile's, none for a named game, which the page names, and its
//! icon), how many are watched, and while ready the offer: its id, the name it carries (the tray's and Pump home's
//! "Pump for {game}") and the kind it offers (`match`, every game's, a program's close too since 2026-10-28), which the
//! Now card's Start asks of DESK-PUMP as the window's own click (`pump.start {kind, offer}`, Ready at the offer's
//! size); held (a match on, a desktop pump), the card is back to playing. `pause`: `{kind: "none" | "until"
//! (at, unix s) | "while_running" (key, name) | "until_resumed"}`, a game it holds for by its `Watched::id` and its
//! name as a tile's (none for a named game); `night` the next 04:00 (unix s). `tiles`, `more`, `hidden`, `riot` and
//! `dot`: `games::Lists` (`Tile` = `{key, kind: "named" | "found" | "added", name, icon, on, riot, running, ends:
//! "match" | "close", setup ("Ask after each match" stands), status, port, aim}`, in the tab's order). `picks` the
//! picker's programs while it is open (`{key, name, image}`: those with a window now, but a found or added game that is
//! on and a named game's own programs), `pick_games` the named games it shows by name in their place while one of their
//! programs has a window and they are not watched (`Games::picker`: League of Legends for its client or the Riot
//! Client), each a key a pick turns on (`games.set`), `pace` Moves per match (the body's `match_pace`, `light`,
//! `balanced` or `extreme`, Balanced while no device chose it), `open_on_close` "Open the pump after each match"
//! (on unless the player turned it off), `notice` "Notice games I play", `held` whether the offers are held now (Send a
//! test offer waits, and says why). A match's offer keeps no quiet hours and no daily cap since slice 48: the Reminders
//! page keeps theirs (`desk/reminders.rs`).
//!
//! Intents: `games.set {key, on}` (any tile: `Games::set`), `games.setup {key}` ("Ask after each match"),
//! `games.dismiss {key}` ("Not a game"), `games.restore {key}` (a hidden game found again), `games.notice {on}`,
//! `games.pick` (the picker's lists, fresh), `games.add {key}` (a program from it), `games.remove {key}`, `games.port
//! {key, port}` (1024 to 65535, refused while another fed game takes it), `games.aim {key, on}` (an aim game's arms
//! rest), `games.grant {key}` (Windows' one-time elevated grant on the folder a game's file goes into), `games.test`
//! ("Send a test offer"), `games.pace {pace}` (Moves per match, a change of the body every device shares),
//! `games.pause {kind: none | hour | four_hours | night | game | resumed}`, `games.open_on_close {on}` and
//! `games.seen` (the tab opened, or in front as its dot came: the dot goes until a game is found again).
//!
//! The tray, while an offer stands and nothing holds it, holds "Pump for {game}" (slice 57, `games.pump`, the name the
//! offer carries: Start a pump with "A match" chosen, `card.start` with no offer and no size), and while a game is
//! watched or a pause holds, the Pause offers submenu: the pause's state, an hour, four hours, until 04:00, while this
//! game runs (the first running in the tab's order), until resumed, and Resume offers (`games.pause`; its own menu ids
//! stay `offers.*`). Its words come from `copy.rs`; its clicks go to the loop as the page's intents do.

use std::sync::atomic::AtomicBool;

use questpump_core::body::{Body, DEFAULT_PACE, is_pace};
use questpump_core::games::{MATCH, Watched};
use questpump_core::offers::{Offer, Pause, Tunables};
use questpump_core::session::Zone;
use serde::Serialize;
use serde_json::{Value, json};
use tauri::menu::{MenuItem, PredefinedMenuItem, Submenu};
use tauri::{AppHandle, Manager as _, Wry};
use tokio::sync::oneshot;

use super::card::CardState;
use super::menu_pages::setup::stamped;
use super::tray::{TrayLines, clock_at};
use super::{Desk, DeskAsk, Intent};
use crate::copy;
use crate::elevate::Outcome;
use crate::games::procs::{Pick, Snapshot};
use crate::games::{Games, Lists, Picker, Tile};
use crate::runtime::{Elevated, Helper, local_offset_at};
use crate::ui::{self, Shared, WindowAsk};
use crate::{elevate, lan};

/// The Games tab's topic.
pub(crate) const GAMES: &str = "games";

/// The tray's offer items, by their menu ids.
const PUMP: &str = "offers.pump";
const PAUSE: &str = "offers.pause";
const STATUS: &str = "offers.status";
const HOUR: &str = "offers.pause.hour";
const FOUR: &str = "offers.pause.four";
const NIGHT: &str = "offers.pause.night";
const GAME: &str = "offers.pause.game";
const RESUMED: &str = "offers.pause.resumed";
const RESUME: &str = "offers.resume";

/// The desktop's side of the offers, kept on the loop: the card, the picker's lists while it is open, and what the
/// tray's offer items show (none: drawn again at the next look).
#[derive(Debug, Default)]
pub(crate) struct Offering {
	pub(crate) card: CardState,
	picker: Option<Picker>,
	tray: Option<TrayOffers>,
}

/// The Games tab.
#[derive(Serialize)]
struct GamesTopic<'a> {
	now: Now,
	pause: PauseView,
	night: i64,
	#[serde(flatten)]
	lists: Lists,
	picks: Option<&'a [Pick]>,
	pick_games: Option<&'a [&'static str]>,
	/// Moves per match (the body's `match_pace`; `DEFAULT_PACE` while no device chose it).
	pace: &'a str,
	open_on_close: bool,
	notice: bool,
	/// The offers are held now (`Helper::offers_held`): "Send a test offer" waits, and says why.
	held: bool,
}

/// The Now card's state, in the order it is decided: a pause first, then an offer that stands and nothing holds, a
/// watched game running, a game watched, none.
#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum State {
	Paused,
	Ready,
	Playing,
	Watching,
	None,
}

/// The Now card.
#[derive(Serialize)]
struct Now {
	state: State,
	game: Option<NowGame>,
	/// How many games are watched.
	count: usize,
	/// While `ready`, the offer its Start starts (DESK-PUMP's `pump.start {kind, offer}`), else none.
	offer: Option<NowOffer>,
}

/// The game the Now card names: its key, its name as its tile's (none for a named game, which the page names from
/// `links.name.<key>`) and its icon.
#[derive(Serialize)]
struct NowGame {
	key: String,
	name: Option<String>,
	icon: Option<String>,
}

impl NowGame {
	fn of(tile: &Tile) -> Self {
		Self { key: tile.key.clone(), name: tile.name.clone(), icon: tile.icon.clone() }
	}

	/// An offer's game: its tile's where the tab holds one, else its key and the name the offer carries (none for a
	/// named game), with no icon (a program removed since).
	fn of_offer(offer: &Offer, lists: &Lists) -> Self {
		lists.tiles.iter().find(|t| t.key == offer.game).map_or_else(
			|| Self {
				key: offer.game.clone(),
				name: Watched::from_key(&offer.game).is_none().then(|| offer.name.clone()),
				icon: None,
			},
			Self::of,
		)
	}
}

/// The offer the Now card shows: its id, the name it carries ("Pump for {game}", as the tray and Pump home name it) and
/// the kind its Start opens the pump at Ready for (`match`, every game's offer's).
#[derive(Serialize)]
struct NowOffer {
	id: String,
	name: String,
	kind: &'static str,
}

impl NowOffer {
	fn of(offer: &Offer) -> Self {
		Self { id: offer.id.clone(), name: offer.name.clone(), kind: MATCH }
	}
}

/// The pause as the tab and the tray word it; while a game runs, that game by its `Watched::id` and its name as its
/// tile's (none for a named game, which the page names).
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum PauseView {
	None,
	Until { at: i64 },
	WhileRunning { key: String, name: Option<String> },
	UntilResumed,
}

/// A watched game running: its id (`Watched::id`) and the name its offers show.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Running {
	id: String,
	name: String,
}

/// What the tray's offer items show.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TrayOffers {
	/// "Pump for {game}", while an offer stands and nothing holds it (`Helper::offer_ready`).
	pump: Option<String>,
	/// The submenu shows: a game is watched, or a pause holds.
	pause: bool,
	/// The pause's state, its first line.
	status: String,
	/// "Until 04:00", with the next game day's start.
	night: String,
	/// "While <game> runs", with the first game running in the tab's order; none while no watched game runs.
	game: Option<String>,
	/// Resume offers is enabled.
	paused: bool,
}

/// The tray's offer items, built once with the tray (`tray::build`).
pub(crate) struct TrayItems {
	pump: MenuItem<Wry>,
	pump_shown: AtomicBool,
	pause: Submenu<Wry>,
	pause_shown: AtomicBool,
	status: MenuItem<Wry>,
	night: MenuItem<Wry>,
	game: MenuItem<Wry>,
	resume: MenuItem<Wry>,
}

/// Builds the tray's offer items, out of the menu until they are due.
pub(crate) fn tray_items(app: &tauri::App) -> tauri::Result<TrayItems> {
	let item = |id: &str, text: &str, enabled: bool| MenuItem::with_id(app, id, text, enabled, None::<&str>);
	// its words, the game's name, come as an offer stands (`redraw`)
	let pump = item(PUMP, "", true)?;
	let status = item(STATUS, copy::desk_offers_paused_none(), false)?;
	let hour = item(HOUR, copy::desk_offers_pause_hour(), true)?;
	let four = item(FOUR, copy::desk_offers_pause_four(), true)?;
	let night = item(NIGHT, "", true)?;
	let game = item(GAME, copy::desk_offers_pause_game_none(), false)?;
	let resumed = item(RESUMED, copy::desk_offers_pause_resumed(), true)?;
	let resume = item(RESUME, copy::desk_offers_resume(), false)?;
	let (line, line2) = (PredefinedMenuItem::separator(app)?, PredefinedMenuItem::separator(app)?);
	let pause = Submenu::with_id_and_items(
		app,
		PAUSE,
		copy::desk_offers_pause(),
		true,
		&[&status, &line, &hour, &four, &night, &game, &resumed, &line2, &resume],
	)?;
	Ok(TrayItems {
		pump,
		pump_shown: AtomicBool::new(false),
		pause,
		pause_shown: AtomicBool::new(false),
		status,
		night,
		game,
		resume,
	})
}

/// Whether a tray menu id is an offer item's.
pub(crate) fn is_tray(id: &str) -> bool {
	[PUMP, HOUR, FOUR, NIGHT, GAME, RESUMED, RESUME].contains(&id)
}

/// A click on an offer item, to the loop as the page's intent.
pub(crate) fn on_tray(app: &AppHandle, id: &str) {
	let pause = |kind: &str| json!({"type": "games.pause", "kind": kind});
	let asked = match id {
		PUMP => json!({"type": "card.start"}),
		HOUR => pause("hour"),
		FOUR => pause("four_hours"),
		NIGHT => pause("night"),
		GAME => pause("game"),
		RESUMED => pause("resumed"),
		RESUME => pause("none"),
		_ => return,
	};
	let intent = match Intent::from_value(asked) {
		Ok(intent) => intent,
		Err(e) => {
			log::warn!("tray: {e}");
			return;
		}
	};
	// the loop answers the page; a tray click waits for no answer
	let (done, _) = oneshot::channel();
	app.state::<Shared>().ask(WindowAsk::Desk(DeskAsk::Intent { intent, done }));
}

/// Draws the offer items as `s` says (on the main thread).
fn redraw(app: &AppHandle, s: &TrayOffers) {
	let Some(lines) = app.try_state::<TrayLines>() else { return };
	let items = &lines.offers;
	if let Some(text) = &s.pump {
		let _ = items.pump.set_text(text);
	}
	lines.show(&items.pump, &items.pump_shown, lines.after_open(), s.pump.is_some());
	let _ = items.status.set_text(&s.status);
	let _ = items.night.set_text(&s.night);
	let _ = items.game.set_text(s.game.as_deref().unwrap_or(copy::desk_offers_pause_game_none()));
	let _ = items.game.set_enabled(s.game.is_some());
	let _ = items.resume.set_enabled(s.paused);
	lines.show(&items.pause, &items.pause_shown, lines.before_quit(), s.pause);
}

/// A unix second as the local clock time, as the tray words it.
fn clock(at: i64) -> String {
	clock_at(at.saturating_mul(1_000))
}

impl Helper {
	/// The offers' numbers; none when library.json's constants did not build in (no game offers then).
	fn tunables(&self) -> Option<&'static Tunables> {
		self.games.as_ref().map(Games::tunables)
	}

	/// The watched game running now that "while this game runs" names: the first running in the tab's order
	/// (`Lists::playing`), as the tab's Now card and its Pause offers name it.
	fn running_game(&self, lists: &Lists) -> Option<Running> {
		let id = lists.playing()?.key.clone();
		let name = Games::name_of(&id, &self.settings.games)?;
		Some(Running { id, name })
	}

	/// The pause as the tab and the tray show it (the game it holds for named as its tile is: a program by its own name,
	/// a named game by the page).
	fn pause_view(&self) -> PauseView {
		match &self.settings.offers.pause {
			Pause::None => PauseView::None,
			Pause::Until { at } => PauseView::Until { at: *at },
			Pause::WhileRunning { game } => PauseView::WhileRunning {
				key: game.clone(),
				name: Watched::from_key(game)
					.is_none()
					.then(|| Games::name_of(game, &self.settings.games).unwrap_or_default()),
			},
			Pause::UntilResumed => PauseView::UntilResumed,
		}
	}

	/// The standing offer while nothing holds the offers back (`offers_held`): the one the tray's item starts and the
	/// Now card shows (slice 57).
	fn offer_ready(&self, now: i64) -> Option<&Offer> {
		self.standing.current().filter(|_| !self.offers_held(now))
	}

	/// The Now card: paused while a pause holds, else ready while an offer stands and nothing holds it
	/// (`offer_ready`), else playing while a watched game runs, else watching while one is on, else none; the game it
	/// names is the offer's while ready, else the first running in the tab's order, or the only one watched.
	fn now_card(&self, lists: &Lists, now: i64) -> Now {
		let on: Vec<&Tile> = lists.tiles.iter().filter(|t| t.on).collect();
		let running = lists.playing();
		let paused = self.settings.offers.pause != Pause::None;
		let ready = if paused { None } else { self.offer_ready(now) };
		let state = if paused {
			State::Paused
		} else if ready.is_some() {
			State::Ready
		} else if running.is_some() {
			State::Playing
		} else if on.is_empty() {
			State::None
		} else {
			State::Watching
		};
		let only = if let [one] = on.as_slice() { Some(one) } else { None };
		let game = match ready {
			Some(offer) => Some(NowGame::of_offer(offer, lists)),
			None => running.or(only.copied()).map(NowGame::of),
		};
		Now { state, game, count: on.len(), offer: ready.map(NowOffer::of) }
	}

	/// The Games tab's topic, the tray's offer items and the games watched by name (This computer's state names them
	/// while no World of Warcraft is found), as they stand (each sent only when it changed).
	pub(crate) fn show_games(&mut self) {
		let Some(games) = self.games.as_ref() else { return };
		let settings = &self.settings.games;
		let lists = games.lists(settings);
		let watching: Vec<String> = lists.watched().iter().map(|w| w.name().to_owned()).collect();
		let watched = !watching.is_empty();
		ui::update(&self.app, |v| v.computer.watching = watching);
		let now = lan::now_ms().div_euclid(1_000);
		let night = Zone::new(local_offset_at).next_game_day(now);
		let tray = self.tray_offers(night, watched, &lists);
		let topic = GamesTopic {
			now: self.now_card(&lists, now),
			pause: self.pause_view(),
			night,
			lists,
			picks: self.offering.picker.as_ref().map(|p| p.programs.as_slice()),
			pick_games: self.offering.picker.as_ref().map(|p| p.games.as_slice()),
			pace: self.body.body.match_pace.as_ref().map_or(DEFAULT_PACE, |p| p.value.as_str()),
			open_on_close: !settings.open_on_close_off,
			notice: !settings.notice_off,
			held: self.offers_held(now),
		};
		self.app.state::<Desk>().publisher.publish(GAMES, &topic);
		if self.offering.tray.as_ref() != Some(&tray) {
			self.offering.tray = Some(tray.clone());
			let handle = self.app.clone();
			if let Err(e) = self.app.run_on_main_thread(move || redraw(&handle, &tray)) {
				log::warn!("tray: {e}");
			}
		}
	}

	/// What the tray's offer items show now (`watched`: a game is watched; `lists`, the Games tab's).
	fn tray_offers(&self, night: i64, watched: bool, lists: &Lists) -> TrayOffers {
		let now = lan::now_ms().div_euclid(1_000);
		let pump = self.offer_ready(now).map(|offer| copy::games_pump(&offer.name));
		let status = match self.pause_view() {
			PauseView::None => copy::desk_offers_paused_none().to_owned(),
			PauseView::Until { at } => copy::desk_offers_paused_until(&clock(at)),
			PauseView::WhileRunning { key, .. } => {
				copy::desk_offers_paused_game(&Games::name_of(&key, &self.settings.games).unwrap_or_default())
			}
			PauseView::UntilResumed => copy::desk_offers_paused_resumed().to_owned(),
		};
		let paused = self.settings.offers.pause != Pause::None;
		TrayOffers {
			pump,
			pause: paused || watched,
			status,
			night: copy::desk_offers_pause_night(&clock(night)),
			game: self.running_game(lists).map(|r| copy::desk_offers_pause_game(&r.name)),
			paused,
		}
	}

	/// The `games` and `card` areas, the Games tab drawn again after each.
	pub(crate) fn offers_intent(&mut self, intent: &Intent) -> Result<(), String> {
		let outcome = match intent.area() {
			"card" => self.card_intent(intent),
			_ => self.games_intent(intent),
		};
		self.show_games();
		outcome
	}

	/// The Games tab's intents for its games: a tile turned on or off or set up, a found game put away or found again,
	/// noticing, the picker, a port, Windows' grant (the offers' own: `offer_intent`).
	fn games_intent(&mut self, intent: &Intent) -> Result<(), String> {
		let key = || intent.arg("key").and_then(Value::as_str).ok_or_else(|| format!("{} takes a key", intent.kind()));
		let on = || intent.arg("on").and_then(Value::as_bool).ok_or_else(|| format!("{} takes on", intent.kind()));
		match intent.kind() {
			"games.set" => {
				let (key, on) = (key()?, on()?);
				let games = self.games.as_mut().ok_or("no games on this computer")?;
				if !games.set(key, on, &mut self.settings.games) {
					return Err(format!("games.set: no game {key}"));
				}
				// a found or picked game's key may be a path in the player's folders: the log names a named game alone
				// copy-lint: not shown (a log line)
				let named = if Watched::from_key(key).is_some() { key } else { "a found or picked game" };
				log::info!("games: {named} turned {}", if on { "on" } else { "off" });
				self.save_games();
			}
			"games.setup" => {
				let key = key()?;
				let games = self.games.as_mut().ok_or("no games on this computer")?;
				if games.setup(key, &mut self.settings.games)? {
					log::info!("games: {key}: asked after each match");
					self.save_games();
				}
			}
			"games.dismiss" => {
				let games = self.games.as_mut().ok_or("no games on this computer")?;
				if games.dismiss(key()?, &mut self.settings.games) {
					log::info!("games: a found game put away");
					self.save_games();
				}
			}
			"games.restore" => {
				if Games::restore(key()?, &mut self.settings.games) {
					log::info!("games: a hidden game found again");
					self.save_games();
				}
			}
			"games.notice" => {
				let on = on()?;
				let settings = &mut self.settings.games;
				if settings.notice_off == on {
					settings.notice_off = !on;
					log::info!("games: noticing games turned {}", if on { "on" } else { "off" });
					self.save_games();
				}
			}
			"games.pick" => {
				let games = self.games.as_ref().ok_or("no games on this computer")?;
				let settings = &self.settings.games;
				// a found or added game that is on is a tile already; one turned off stays, which a pick turns on again
				let on = |key: &str| {
					!settings.is_off(key)
						&& (settings.detected.iter().any(|d| d.key == key)
							|| settings.programs.iter().any(|p| p.key == key))
				};
				let mut picker = games.picker(&Snapshot::take(), settings);
				picker.programs.retain(|p| !on(&p.key));
				self.offering.picker = Some(picker);
			}
			"games.add" => {
				let key = key()?;
				let pick =
					self.offering.picker.as_ref().and_then(|picker| picker.programs.iter().find(|p| p.key == key));
				let pick = pick.ok_or("games.add: no such program in the picker")?.clone();
				if Games::add_program(&pick, &mut self.settings.games) {
					log::info!("games: a program picked");
					self.save_games();
				}
				self.offering.picker = None;
			}
			"games.remove" => {
				let key = key()?;
				let games = self.games.as_mut().ok_or("no games on this computer")?;
				if games.remove_program(key, &mut self.settings.games) {
					log::info!("games: a program removed");
					self.save_games();
				}
			}
			"games.grant" => self.grant_game(key()?)?,
			"games.port" => {
				let key = key()?;
				let port = intent.arg("port").and_then(Value::as_u64).and_then(|p| u16::try_from(p).ok());
				let port = port.ok_or("games.port takes a port up to 65535")?;
				let games = self.games.as_mut().ok_or("no games on this computer")?;
				if games.set_port(key, port, &mut self.settings.games)? {
					log::info!("games: {key}: its port set");
					self.save_games();
				}
			}
			_ => return self.offer_intent(intent),
		}
		Ok(())
	}

	/// The Games tab's intents for the offers themselves: the test, Moves per match, the arms' rest, the pause, Open
	/// the pump after each match, and the tab seen.
	fn offer_intent(&mut self, intent: &Intent) -> Result<(), String> {
		let key = || intent.arg("key").and_then(Value::as_str).ok_or_else(|| format!("{} takes a key", intent.kind()));
		let on = || intent.arg("on").and_then(Value::as_bool).ok_or_else(|| format!("{} takes on", intent.kind()));
		match intent.kind() {
			"games.test" => self.test_offer()?,
			"games.pace" => {
				let pace = intent.arg("pace").and_then(Value::as_str).filter(|p| is_pace(p));
				let pace = pace.ok_or("games.pace takes light, balanced or extreme")?;
				let changed = stamped(pace.to_owned(), self.body.body.match_pace.as_ref());
				log::info!("games: Moves per match {pace}");
				self.body_changed(&Body { match_pace: Some(changed), ..Body::default() });
			}
			"games.aim" => {
				let (key, on) = (key()?, on()?);
				if !Watched::from_key(key).is_some_and(|w| w.aim()) {
					return Err(format!("games.aim: {key} is no aim game"));
				}
				let off = &mut self.settings.games.aim_off;
				off.retain(|k| k != key);
				if !on {
					off.push(key.to_owned());
				}
				log::info!("games: {key}: the arms rest {}", if on { "on" } else { "off" });
				self.save_settings();
			}
			"games.pause" => self.pause(intent)?,
			"games.open_on_close" => {
				self.settings.games.open_on_close_off = !on()?;
				self.save_settings();
			}
			"games.seen" => {
				self.settings.games.seen_at = Some(lan::now_ms().div_euclid(1_000));
				self.save_settings();
			}
			_ => return Err(intent.unknown()),
		}
		Ok(())
	}

	/// Windows' one-time elevated grant on the folder a game's file goes into, off the loop; a second click while its
	/// prompt is up asks nothing.
	fn grant_game(&mut self, key: &str) -> Result<(), String> {
		let games = self.games.as_mut().ok_or("no games on this computer")?;
		let Some(folder) = games.grant_target(key) else { return Ok(()) };
		let (tx, key) = (self.elevated.clone(), key.to_owned());
		tokio::spawn(async move {
			let outcome = tokio::task::spawn_blocking(move || elevate::allow_folder(&folder))
				.await
				.unwrap_or_else(|e| Outcome::Failed(e.to_string()));
			let _ = tx.send(Elevated::Game(key, outcome));
		});
		Ok(())
	}

	/// The grant came back: the file goes in at the next look; a declined or failed grant asks again. The log names the
	/// game alone: a failure's text may hold the folder, which is inside the player's own game library.
	pub(crate) fn game_allowed(&mut self, key: &str, outcome: &Outcome) {
		match outcome {
			Outcome::Done => log::info!("games: {key}: its folder may be written"),
			Outcome::Declined => log::info!("games: {key}: the administrator step was declined"),
			Outcome::Failed(_) => log::warn!("games: {key}: the grant could not be made"),
		}
		if let Some(games) = self.games.as_mut() {
			games.granted(key, matches!(outcome, Outcome::Done));
		}
		self.show_games();
	}

	/// The pause the tab or the tray chose (`games.pause`).
	fn pause(&mut self, intent: &Intent) -> Result<(), String> {
		let t = self.tunables().ok_or("no offers on this computer")?;
		let now = lan::now_ms().div_euclid(1_000);
		let kind = intent.arg("kind").and_then(Value::as_str).ok_or("games.pause takes a kind")?;
		let pause = match kind {
			"none" => Pause::None,
			"hour" => Pause::Until { at: now + t.hour_s },
			"four_hours" => Pause::Until { at: now + t.four_hours_s },
			"night" => Pause::Until { at: Zone::new(local_offset_at).next_game_day(now) },
			"game" => {
				let lists = self.games.as_ref().ok_or("no games on this computer")?.lists(&self.settings.games);
				Pause::WhileRunning { game: self.running_game(&lists).ok_or("no watched game runs")?.id }
			}
			"resumed" => Pause::UntilResumed,
			_ => return Err(format!("games.pause: no pause {kind}")),
		};
		log::info!("offers: the pause {kind}");
		self.settings.offers.pause = pause;
		// the tray's items are drawn again, whatever a click on them showed
		self.offering.tray = None;
		self.save_settings();
		Ok(())
	}
}
