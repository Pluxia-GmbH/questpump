//! The match-over card (docs/ui.md, "The match-over card"; docs/plan.md slice 35, "The offer after a match"): a small
//! window (tauri.conf.json's `card`) that never takes the focus, above other windows and with no taskbar entry, showing
//! the offer that stands after a watched game's end. It is built hidden and unfocusable and ordered in without
//! activation (macOS `orderFrontRegardless`, Windows `SW_SHOWNOACTIVATE` over `HWND_TOPMOST`), never through `show()`
//! or `set_focus()`, at the lower right of the work area of a screen without the game's window (a second screen), or,
//! with one screen, of the game's screen once the game has left the front or closed (its screen as last seen, which
//! the look notes for each running game at most every `SCREEN_EVERY_S`, else the main one); with one screen and the
//! game in front the card waits (`CardState::waits`), and the offer goes to the phone and the watch meanwhile
//! (`runtime/matches.rs`, `out_look`), as it does once the card has shown to no input for `match_away_s` (the input
//! idle that long and the card on screen that long since it first showed, `CardState::shown_for`). One card at a
//! time: it goes after `match_offer_min`, on Later or ×, at any Start from its offer (`offer_answered`), when any device
//! takes the offer, when a move is done since, when a newer offer replaces it, and while a match, the felt-unwell
//! lock, a World of Warcraft client with QuestPump or a pump on this computer holds (`offers_held`); the offer itself
//! stands on Pump home and in the tray. Later or × on an offer that went out to the devices withdraws it there
//! (`offer_withdrawn`). It plays the offer's one soft sound as an offer's card first shows (`offer_sound`,
//! DESK-MEDIA's), never again as the same card is asked for again.
//!
//! The page (`card.html`) draws the `card` topic (`CardView`, or null) in the kit's `MatchCard`, its size chosen at
//! the offer's tier (slice 57: its match's minutes', `offers::Offer::tier`; the sitting clock's until then; with Moves
//! per match at Light no size shows, `one_size`, and Start still sends the tier), and sends `card.start` (`{offer,
//! size}`: `small`, `normal` or `big`) and `card.later` (`{offer}`, Later and × alike). The tray's "Pump for {game}"
//! sends `card.start` with neither; the main window sends neither (Pump home and the Games tab's Now card start the
//! offer with DESK-PUMP's own intents, the window's own click). Start asks DESK-PUMP from outside the main window
//! (`outside`, which opens it): `pump.start {kind, size?, offer}`, the pump at Ready for "A match" at the card's size,
//! else the offer's tier (`pump_start_intent`), after every game's offer, a found or added program's close too (since
//! 2026-10-28, Rapha: "it should directly open and start and not let me select stuff"; a program's close and the
//! tray's Start opened the Start a pump window, `pump.choose`, until then); no linked game rides along (a desktop
//! pump's rows carry none). The pump opens the main window, the card goes (DESK-PUMP's `offer_started`, for every Start from
//! the offer, Pump home's and the Now card's too, the first counted `started`: `offer_answered`), and the offer
//! stands until the pump starts (`offer_taken`). "Send a test offer" shows the card at
//! once on the main window's screen with the id `m-test` at no size chosen (Balanced; no `match_end`, no count, no
//! start of the clock again) and nothing while `offers_held`; a newer offer's card replaces it. "Open the pump after
//! each match" (on by default since 2026-10-27) presses Start for the player as the offer is made, at the offer's tier,
//! on the game's screen as last seen: at a match's end its feed told (Mechabellum's, CS2's), or at the game's close (a
//! League's or a program's offer comes at the close itself), in the place of an idle pump on screen, which ends as
//! Leave ends it, while a move that runs is never interrupted and the offer opens once its pump is idle (`open_offer`).
//! So with it on the card never shows for an offer (Rapha, 2026-10-29: "It should open the pump after a match ...
//! this pop up is not needed at all"; it opened only at the game's close until then, 2026-10-28), and with it off the
//! card is the offer's way in. A
//! click on the card may activate QuestPump on macOS (an `NSWindow` is no non-activating panel), so Later and × hide
//! QuestPump again where the click made it active (no main window open, or QuestPump not the active app as the card
//! came: a pump the card started leaves its window open since 2026-10-25), which gives the front back.
//!
//! The reminders show on the same window (`reminders.rs`, 2026-10-09): while no match offer stands (its card's Later
//! or time past too: slice 48's double-check), the reminder that stands, on the main screen, with the offer's soft
//! sound and never "Match over."; its Start opens the Quick pump at Ready, its Later and × end the reminder.

use std::collections::HashMap;

use questpump_core::games::{MATCH, Watched, computer_links};
use questpump_core::offers::{Offer, TEST};
use serde::Serialize;
use serde_json::{Value, json};
use tauri::utils::config::WindowConfig;
use tauri::{AppHandle, Manager as _, Monitor, WebviewWindow, WebviewWindowBuilder, WindowEvent};

use super::reminders::RemindView;
use super::windows::MAIN;
use super::{Desk, Intent};
use crate::games::Games;
use crate::games::procs::{self, Area, Snapshot};
use crate::lan;
use crate::runtime::Helper;

/// The card window's label (tauri.conf.json) and the topic it draws.
pub(crate) const CARD: &str = "card";
/// The card's distance from the work area's right and bottom edges, in CSS px (the layout's `space16`).
const EDGE: f64 = 16.0;
/// The sizes the card's Segmented offers (the phones' Start a pump's), each with the tier `pump.start` takes.
const SIZES: [(&str, u8); 3] = [("small", 1), ("normal", 2), ("big", 3)];
/// A card asked for whose window is missing this long after was never built (its label still taken by the one that
/// went): it is asked for again.
const REBUILD_S: i64 = 4;
/// The least time between two looks at a running game's screen (seconds): a window's screen asks the window server.
const SCREEN_EVERY_S: i64 = 10;

/// What the card shows (the `card` topic, null while no card shows): the offer's id (`m-test` for the test), the name
/// the offer carries, the size its Segmented starts at (the offer's tier; none for the test, Balanced), whether every
/// size takes the same moves (Moves per match at Light, `Helper::match_one_size`: no Segmented) and whether it is
/// "Send a test offer"'s; or a reminder's (`remind`: its id `r-…`, no name or size).
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub(crate) struct CardView {
	pub(crate) id: String,
	pub(crate) name: String,
	pub(crate) tier: Option<u8>,
	pub(crate) one_size: bool,
	pub(crate) test: bool,
	pub(crate) remind: Option<RemindView>,
}

impl CardView {
	pub(crate) fn of(offer: &Offer, one_size: bool) -> Self {
		Self {
			id: offer.id.clone(),
			name: offer.name.clone(),
			tier: Some(offer.tier),
			one_size,
			test: false,
			remind: None,
		}
	}
}

/// A running game's screen as the look last found it (`screens_look`): its window's screen, the one before where the
/// window was not found then (hidden, on another Space), and when it looked (unix seconds).
#[derive(Debug, Clone, Copy)]
struct Seen {
	area: Option<Area>,
	at: i64,
}

/// The card's state on the loop.
#[derive(Debug, Default)]
pub(crate) struct CardState {
	/// The standing offer's id and its game (`Watched::id`): the card looks for the game's window.
	game: Option<(String, String)>,
	/// Each watched game's screen as last seen with its window on it (`Watched::id`): its offer's card's screen, and the
	/// pump's, once it has closed.
	seen: HashMap<String, Seen>,
	/// The offer whose card could not show at this look: one screen, and its game in front (the offer goes to the
	/// devices then).
	waits: Option<String>,
	/// A watched game was in a match at the last look or post: a new match's start is its edge (`match_began`).
	in_match: bool,
	/// The card the window shows and the unix second it was asked for; none while it is closed.
	shown: Option<(CardView, i64)>,
	/// The unix second the standing offer's card first showed since it last opened (a reminder's and the test's never
	/// count): the away rule counts the card's time on screen from it (`out_look`, slice 48's second fix round). None
	/// while the card is closed, and a new offer's starts afresh.
	first_shown: Option<i64>,
	/// "Send a test offer"'s card while it lasts.
	test: Option<Test>,
	/// The offer whose card last played the offer's sound: a card asked for again stays silent.
	sounded: Option<String>,
	/// The offer whose Start was counted (`game_offer_desk` started): the Start a pump window opened from it and that
	/// window's own Start, or a Start of it again, count no second time.
	started: Option<String>,
	/// The offer "Open the pump after each match" opened, so it opens once.
	opened: Option<String>,
	/// The player started that offer's pump since from the card or the tray: a recall leaves it (`opened_from`).
	by_hand: bool,
	/// The offer made, or whose game closed, while a move ran in the pump on this computer (`desk_pump_mid_move`), and
	/// the unix second that pump went idle: none while the move runs, no move answers the offer (`answered_from`), and
	/// once it is idle the look opens it (`Helper::offer_after_pump`); a new offer ends it.
	behind: Option<(String, Option<i64>)>,
}

/// "Send a test offer"'s card: what it shows, the unix second it goes (`match_offer_min` after it was sent), and the
/// offer that stood as it was sent, whose card waits behind it (a newer offer's replaces it).
#[derive(Debug)]
struct Test {
	view: CardView,
	until: i64,
	over: Option<String>,
}

impl CardState {
	/// A new offer stands, from the game `game` (`Watched::id`).
	pub(crate) fn stands(&mut self, offer: &str, game: &str) {
		self.game = Some((offer.to_owned(), game.to_owned()));
		self.first_shown = None;
		self.behind = None;
	}

	/// The offer `offer` was made, or its game closed, while a move ran in the pump here: it waits for that pump.
	pub(crate) fn wait_pump(&mut self, offer: &str) {
		self.behind = Some((offer.to_owned(), None));
	}

	/// The offer that waits for a move running in the pump here, while it does.
	pub(crate) fn waiting_pump(&self) -> Option<&str> {
		self.behind.as_ref().filter(|(_, idle)| idle.is_none()).map(|(id, _)| id.as_str())
	}

	/// The pump the waiting offer waited for is idle at unix second `now`: a move from then on answers it.
	pub(crate) fn pump_idle(&mut self, now: i64) {
		if let Some((_, idle)) = self.behind.as_mut().filter(|(_, idle)| idle.is_none()) {
			*idle = Some(now);
		}
	}

	/// The unix second after which a performed move answers the offer `offer` made at `at`: none while it waits for a
	/// move running here, whose own moves never answer it; once that pump went idle, its moment.
	pub(crate) fn answered_from(&self, offer: &str, at: i64) -> Option<i64> {
		match self.behind.as_ref().filter(|(id, _)| id == offer) {
			Some((_, idle)) => idle.map(|idle| idle.max(at)),
			None => Some(at),
		}
	}

	/// The screen the offer `offer`'s game was last seen on.
	fn area_of(&self, offer: &str) -> Option<Area> {
		self.game_of(offer).and_then(|game| self.seen.get(game)).and_then(|seen| seen.area)
	}

	/// Whether the offer `offer`'s card could not show at this look (the game in front on its one screen).
	pub(crate) fn waits(&self, offer: &str) -> bool {
		self.waits.as_deref() == Some(offer)
	}

	/// Whether a match began since the last look or post (`in_match` now, none then); the state is kept for the next.
	pub(crate) fn match_began(&mut self, in_match: bool) -> bool {
		!std::mem::replace(&mut self.in_match, in_match) && in_match
	}

	/// The game the offer `offer` came from.
	fn game_of(&self, offer: &str) -> Option<&str> {
		self.game.as_ref().filter(|(id, _)| id == offer).map(|(_, game)| game.as_str())
	}

	/// The offer "Open the pump after each match" opened from the game `game` (`Watched::id`), while it is the last
	/// offer that stood and the player has not started its pump by hand since.
	pub(crate) fn opened_from(&self, game: &str) -> Option<&str> {
		self.opened.as_deref().filter(|id| !self.by_hand && self.game_of(id) == Some(game))
	}

	/// Whether the window shows the offer `offer`'s card.
	pub(crate) fn shows(&self, offer: &str) -> bool {
		self.shown.as_ref().is_some_and(|(view, _)| view.id == offer)
	}

	/// The card the window shows.
	pub(crate) fn shown(&self) -> Option<&CardView> {
		self.shown.as_ref().map(|(view, _)| view)
	}

	/// How long the offer `offer`'s card has shown at `now` (seconds since it first showed), none while it does not.
	pub(crate) fn shown_for(&self, offer: &str, now: i64) -> Option<i64> {
		self.first_shown.filter(|_| self.shows(offer)).map(|at| now - at)
	}

	/// Whether this is the offer `id`'s card's first show, which plays the offer's sound; marked as sounded.
	fn first_show(&mut self, id: &str) -> bool {
		if self.sounded.as_deref() == Some(id) {
			return false;
		}
		self.sounded = Some(id.to_owned());
		true
	}
}

/// A screen in `procs::Area`'s units (Windows: physical pixels; macOS: points of the global display space): its whole
/// area, its work area (no menu bar, Dock or taskbar) and its scale.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Screen {
	whole: Area,
	work: Area,
	scale: f64,
}

#[expect(clippy::cast_possible_truncation, reason = "a screen's coordinates fit an i32")]
fn whole(v: f64) -> i32 {
	v.round() as i32
}

impl Screen {
	/// A monitor as Tauri gives it: macOS's points times the screen's scale, Windows' pixels.
	fn of(m: &Monitor) -> Self {
		let scale = m.scale_factor();
		let per = if cfg!(target_os = "macos") { scale } else { 1.0 };
		let area = |x: i32, y: i32, width: u32, height: u32| Area {
			x: whole(f64::from(x) / per),
			y: whole(f64::from(y) / per),
			width: whole(f64::from(width) / per),
			height: whole(f64::from(height) / per),
		};
		let (at, size, work) = (m.position(), m.size(), m.work_area());
		Self {
			whole: area(at.x, at.y, size.width, size.height),
			work: area(work.position.x, work.position.y, work.size.width, work.size.height),
			scale,
		}
	}

	/// Whether an area's centre lies on it.
	fn holds(&self, a: &Area) -> bool {
		let (x, y) = (i64::from(a.x) + i64::from(a.width) / 2, i64::from(a.y) + i64::from(a.height) / 2);
		let s = &self.whole;
		(i64::from(s.x)..i64::from(s.x) + i64::from(s.width)).contains(&x)
			&& (i64::from(s.y)..i64::from(s.y) + i64::from(s.height)).contains(&y)
	}
}

/// Every screen, the main one first.
fn screens(app: &AppHandle) -> Vec<Screen> {
	let main = app.primary_monitor().ok().flatten().map(|m| Screen::of(&m).whole);
	let mut all: Vec<Screen> = app.available_monitors().unwrap_or_default().iter().map(Screen::of).collect();
	all.sort_by_key(|s| Some(s.whole) != main);
	all
}

/// Where a card goes, in `Area`'s units: its top left and its size.
#[derive(Debug, Clone, Copy)]
struct Spot {
	x: f64,
	y: f64,
	width: f64,
	height: f64,
}

/// The lower right of a screen's work area for a window of `width` × `height` CSS px.
fn spot(screen: &Screen, width: f64, height: f64) -> Spot {
	// macOS places in points, Windows in the screen's own pixels
	let per = if cfg!(windows) { screen.scale } else { 1.0 };
	let (width, height, edge) = (width * per, height * per, EDGE * per);
	let w = &screen.work;
	let right = f64::from(w.x) + f64::from(w.width);
	let bottom = f64::from(w.y) + f64::from(w.height);
	Spot { x: right - width - edge, y: bottom - height - edge, width, height }
}

/// tauri.conf.json's card window.
fn card_config(app: &AppHandle) -> Option<WindowConfig> {
	app.config().app.windows.iter().find(|w| w.label == CARD).cloned()
}

/// Builds the card window hidden and unfocusable, then orders it in at `spot` without activating anything (on the main
/// thread). A card window already there stays as it is. It takes no zoom: the card keeps the page's own scale and its
/// 360 × 168 (tauri.conf.json), never the main window's `windows::BASE` or the player's step, since it is a fixed
/// corner over the game, not resizable, whose page clips what overflows it (desk.css `.qp-card-window`): a larger
/// scale would cut its words and its Start.
fn open(app: &AppHandle, config: &WindowConfig, spot: Spot) {
	if app.get_webview_window(CARD).is_some() {
		return;
	}
	// whether QuestPump was in front as the card came, which its Later or × gives back (`give_back`)
	#[cfg(target_os = "macos")]
	ACTIVE_AT_CARD.store(app_active(), std::sync::atomic::Ordering::Relaxed);
	let builder = match WebviewWindowBuilder::from_config(app, config) {
		Ok(builder) => builder.on_navigation(super::windows::own_pages(app, config)),
		Err(e) => {
			log::warn!("card: {e}");
			return;
		}
	};
	// macOS takes the place at creation, in points; Windows' comes in its own pixels as the card is ordered in
	#[cfg(target_os = "macos")]
	let builder = builder.position(spot.x, spot.y).inner_size(spot.width, spot.height);
	let window = match builder.build() {
		Ok(window) => window,
		Err(e) => {
			log::warn!("card: {e}");
			return;
		}
	};
	let handle = app.clone();
	window.on_window_event(move |event| {
		if matches!(event, WindowEvent::Destroyed) {
			handle.state::<Desk>().publisher.forget(CARD);
		}
	});
	if let Err(e) = order_in(&window, spot) {
		log::warn!("card: {e}");
	}
}

/// macOS: the window in front of the others on its screen, the app left inactive and the window never key.
#[cfg(target_os = "macos")]
fn order_in(window: &WebviewWindow, _spot: Spot) -> Result<(), String> {
	use objc2::msg_send;
	use objc2::runtime::AnyObject;
	let ns_window: *mut AnyObject = window.ns_window().map_err(|e| e.to_string())?.cast();
	// SAFETY: `ns_window` is the card's live NSWindow (Tauri hands it out while the window exists) and this runs on the
	// main thread; orderFrontRegardless takes nothing and returns nothing.
	unsafe {
		let () = msg_send![ns_window, orderFrontRegardless];
	}
	Ok(())
}

/// Windows: the window placed, sized and raised to the topmost band without activation, then shown without activation.
#[cfg(windows)]
fn order_in(window: &WebviewWindow, spot: Spot) -> Result<(), String> {
	use ::windows::Win32::UI::WindowsAndMessaging::{
		HWND_TOPMOST, SW_SHOWNOACTIVATE, SWP_NOACTIVATE, SetWindowPos, ShowWindow,
	};
	let hwnd = window.hwnd().map_err(|e| e.to_string())?;
	let (x, y, width, height) = (whole(spot.x), whole(spot.y), whole(spot.width), whole(spot.height));
	// SAFETY: `hwnd` is the card's live window (Tauri hands it out while the window exists) and this runs on the main
	// thread that owns it; SetWindowPos and ShowWindow only move, size, raise and show it.
	unsafe {
		SetWindowPos(hwnd, Some(HWND_TOPMOST), x, y, width, height, SWP_NOACTIVATE).map_err(|e| e.to_string())?;
		let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
	}
	Ok(())
}

/// The main window, just opened, onto `screen` when it opened on another, centred in its work area.
fn to_screen(app: &AppHandle, screen: Screen) {
	let Some(window) = app.get_webview_window(MAIN) else { return };
	if window.current_monitor().ok().flatten().is_some_and(|m| Screen::of(&m).whole == screen.whole) {
		return;
	}
	let (Ok(size), Ok(scale)) = (window.outer_size(), window.scale_factor()) else { return };
	let per = if cfg!(target_os = "macos") { scale } else { 1.0 };
	let w = &screen.work;
	let x = f64::from(w.x) + (f64::from(w.width) - f64::from(size.width) / per).max(0.0) / 2.0;
	let y = f64::from(w.y) + (f64::from(w.height) - f64::from(size.height) / per).max(0.0) / 2.0;
	#[cfg(target_os = "macos")]
	let at = tauri::Position::Logical(tauri::LogicalPosition::new(x, y));
	#[cfg(windows)]
	let at = tauri::Position::Physical(tauri::PhysicalPosition::new(whole(x), whole(y)));
	if let Err(e) = window.set_position(at) {
		log::warn!("window: {e}");
	}
}

/// macOS: whether QuestPump was the active app as the card window came (`open`), so its Later or × hides it again only
/// where the click made it so.
#[cfg(target_os = "macos")]
static ACTIVE_AT_CARD: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// macOS: QuestPump is the active app (a click on the card may have made it so; an opening asks for it, `windows::forward`).
#[cfg(target_os = "macos")]
pub(super) fn app_active() -> bool {
	use objc2::runtime::AnyObject;
	use objc2::{class, msg_send};
	// SAFETY: NSApplication's sharedApplication and isActive take nothing; this runs on the main thread, where AppKit's
	// application object lives, and a null answer is checked before it is messaged.
	unsafe {
		let app: *mut AnyObject = msg_send![class!(NSApplication), sharedApplication];
		!app.is_null() && msg_send![app, isActive]
	}
}

impl Helper {
	/// Whether no card shows now, no offer goes out to the devices and the tray and the Now card offer none: what holds
	/// the offers whatever runs here (`offers_held_apart`), or a pump runs on this computer (`desk_pump_running`).
	pub(crate) fn offers_held(&self, now: i64) -> bool {
		self.offers_held_apart(now) || self.desk_pump_running()
	}

	/// What holds the offers back whatever runs on this computer, "Open the pump after each match" too: the
	/// felt-unwell lock holds, the game asks the pumps (`wow_asks_pumps`), or a watched game is in a match.
	fn offers_held_apart(&self, now: i64) -> bool {
		self.session.locked_until(now).is_some()
			|| self.wow_asks_pumps()
			|| self.games.as_ref().is_some_and(|g| g.watcher.in_match())
	}

	/// The card due at `now`, none while something holds the offers back: the standing offer's while its card's time
	/// holds, nothing answered it and the player's look at the game's last screen ended (`Offer::look_until`); else,
	/// while no offer stands at all (its card's Later or time past keeps the reminders away too), the reminder's that
	/// stands.
	fn card_due(&self, now: i64) -> Option<CardView> {
		if self.offers_held(now) {
			return None;
		}
		let Some(offer) = self.standing.current() else { return self.reminder_view() };
		(self.standing.card(now) && offer.look_until.is_none())
			.then(|| CardView::of(offer, self.offer_one_size(Some(&offer.id))))
	}

	/// The card at a look: none while something holds the offers back; the test's while it lasts and no newer card is
	/// due; the standing offer's while its time holds, on a screen without the game, else the game's once it has left
	/// the front or closed; else a reminder's on the main screen.
	pub(crate) fn card_look(&mut self, snap: &Snapshot, now: i64) {
		self.offering.card.waits = None;
		let held = self.offers_held(now);
		let due = self.card_due(now);
		if let Some(test) = &self.offering.card.test {
			let newer = due.as_ref().is_some_and(|due| test.over.as_ref() != Some(&due.id));
			if now < test.until && !held && !newer {
				return;
			}
			// its time is up, the offers are held or a newer offer is due: the test's card goes, and a due offer's
			// comes at the next look
			self.offering.card.test = None;
			self.close_card();
			return;
		}
		let Some(due) = due else {
			self.close_card();
			return;
		};
		if let Some((shown, asked)) = &self.offering.card.shown {
			if shown.id != due.id {
				// a newer offer: this window goes, and the new card comes at the next look
				self.close_card();
				return;
			}
			if self.app.get_webview_window(CARD).is_some() || now - asked < REBUILD_S {
				return;
			}
		}
		if due.remind.is_some() {
			if let Some(main) = screens(&self.app).first().copied() {
				self.show_card(due, main, now);
			}
			return;
		}
		let Some(games) = self.games.as_ref() else { return };
		let game = self.offering.card.game_of(&due.id).map(str::to_owned);
		let pids = game.as_deref().map(|game| games.pids(game, snap)).unwrap_or_default();
		let area = pids.iter().find_map(|&pid| procs::window_monitor(pid));
		if let (Some(game), Some(_)) = (game, area) {
			self.offering.card.seen.insert(game, Seen { area, at: now });
		}
		let screens = screens(&self.app);
		// the game's screen: where its window is, else where it was last seen (closed, hidden, on another Space)
		let seen = self.offering.card.area_of(&due.id).and_then(|a| screens.iter().find(|s| s.holds(&a)).copied());
		let screen = if let Some(other) = area.and_then(|a| screens.iter().find(|s| !s.holds(&a)).copied()) {
			Some(other)
		} else if procs::foreground_pid().is_some_and(|front| pids.contains(&front)) {
			// one screen and the game in front (its scoreboard would hide the card): the card waits
			None
		} else {
			seen.or_else(|| screens.first().copied())
		};
		match screen {
			Some(screen) => self.show_card(due, screen, now),
			None => self.offering.card.waits = Some(due.id),
		}
	}

	/// Each watched game that runs has its window's screen looked at, at most every `SCREEN_EVERY_S` (slice 48's
	/// double-check): its offer's card, and the pump "Open the pump after each match" opens, go onto that screen.
	pub(crate) fn screens_look(&mut self, snap: &Snapshot, now: i64) {
		let Some(games) = self.games.as_ref() else { return };
		for game in games.running() {
			let last = self.offering.card.seen.get(game).copied();
			if last.is_some_and(|seen| now - seen.at < SCREEN_EVERY_S) {
				continue;
			}
			let found = games.pids(game, snap).into_iter().find_map(procs::window_monitor);
			let area = found.or(last.and_then(|seen| seen.area));
			self.offering.card.seen.insert(game.clone(), Seen { area, at: now });
		}
	}

	/// The card on `screen`'s lower right (a card window already there shows it in its place), its topic first so the
	/// page draws it as it loads, and the offer's sound on its first show.
	fn show_card(&mut self, view: CardView, screen: Screen, now: i64) {
		let Some(config) = card_config(&self.app) else {
			log::error!("card: tauri.conf.json has no card window");
			return;
		};
		let spot = spot(&screen, config.width, config.height);
		self.app.state::<Desk>().publisher.publish(CARD, &Some(&view));
		let first = self.offering.card.first_show(&view.id);
		let line = view.remind.is_none();
		if first {
			log::info!("{} {}: the card shows", if line { "offer" } else { "reminder" }, view.id);
		}
		if line && !view.test {
			self.offering.card.first_shown.get_or_insert(now);
		}
		self.offering.card.shown = Some((view, now));
		let handle = self.app.clone();
		if let Err(e) = self.app.run_on_main_thread(move || open(&handle, &config, spot)) {
			log::warn!("card: {e}");
		}
		if first {
			self.offer_sound(line);
		}
	}

	/// The card window goes (its offer stands), and its topic is null.
	pub(crate) fn close_card(&mut self) {
		self.offering.card.first_shown = None;
		if self.offering.card.shown.take().is_none() {
			return;
		}
		self.app.state::<Desk>().publisher.publish(CARD, &None::<CardView>);
		if let Some(window) = self.app.get_webview_window(CARD)
			&& let Err(e) = window.destroy()
		{
			log::warn!("card: {e}");
		}
	}

	/// "Send a test offer": the card at once on the main window's screen (the main one without it), with the id
	/// `m-test` and no size chosen (no match measured it), for `match_offer_min`, sounding at each press; no
	/// `match_end`, no count, and the clock runs on. A card already showing shows the test in its place.
	///
	/// # Errors
	/// Why no test card shows: no offers on this computer, the offers held now (`offers_held`), nothing to name.
	pub(crate) fn test_offer(&mut self) -> Result<(), String> {
		let t = self.games.as_ref().map(Games::tunables).ok_or("no offers on this computer")?;
		let now = lan::now_ms().div_euclid(1_000);
		if self.offers_held(now) {
			return Err("offers are held now".to_owned());
		}
		let view = self.test_view().ok_or("no game to name")?;
		let here = self.app.get_webview_window(MAIN).and_then(|w| w.current_monitor().ok().flatten());
		let screen = here.map(|m| Screen::of(&m)).or_else(|| screens(&self.app).first().copied()).ok_or("no screen")?;
		let over = self.card_due(now).map(|card| card.id);
		self.offering.card.test = Some(Test { view: view.clone(), until: now + t.offer_s, over });
		// each press is a new test, whose card sounds as it shows
		self.offering.card.sounded.take_if(|id| id == TEST);
		log::info!("offer {TEST}: the test card");
		self.show_card(view, screen, now);
		Ok(())
	}

	/// The test's card goes: a pump its Start opened has started (`offer_taken` with `m-test`).
	pub(crate) fn close_test(&mut self) {
		if self.offering.card.test.take().is_some() && self.offering.card.shown().is_some_and(|c| c.test) {
			self.close_card();
		}
	}

	/// The test's card names the first game watched, in the Games tab's order (`Games::watched`), else the first game
	/// this system runs, with no size chosen (slice 57: the sitting clock sizes nothing, and no match measured it).
	fn test_view(&self) -> Option<CardView> {
		let system = if cfg!(windows) { "windows" } else { "macos" };
		let first = self.games.as_ref().and_then(|g| g.watched(&self.settings.games).into_iter().next());
		let watched = first.or_else(|| {
			computer_links()?
				.iter()
				.filter(|l| l.systems.iter().any(|on| on == system))
				.find_map(|l| Watched::from_key(&l.key))
		})?;
		Some(CardView {
			id: TEST.to_owned(),
			name: watched.name().to_owned(),
			tier: None,
			one_size: self.match_one_size(),
			test: true,
			remind: None,
		})
	}

	/// The `card` area, from the card window and the tray alone: `card.start` (`{offer?, size?}`) and `card.later`
	/// (`{offer?}`); an offer's id names the card the page shows (none: the standing offer, as the tray sends it).
	pub(crate) fn card_intent(&mut self, intent: &Intent) -> Result<(), String> {
		let asked = intent.arg("offer").and_then(Value::as_str);
		match intent.kind() {
			"card.start" => {
				let size = match intent.arg("size") {
					None | Some(Value::Null) => None,
					Some(v) => {
						let named = v.as_str().and_then(|s| SIZES.iter().find(|(name, _)| *name == s));
						Some(named.ok_or("card.start's size is a size")?.1)
					}
				};
				let view = self.card_offer(asked).ok_or("card.start: the offer is gone")?;
				if view.remind.is_some() {
					self.close_card();
					return self.reminder_start();
				}
				self.start_offer(&view, size, None)?;
				self.offering.card.by_hand |= self.offering.card.opened.as_ref() == Some(&view.id);
				Ok(())
			}
			"card.later" => {
				if asked.is_none_or(|id| self.offering.card.shows(id)) {
					if self.offering.card.shown().is_some_and(|card| card.remind.is_some()) {
						self.reminder_later();
					} else if self.offering.card.test.take().is_none() {
						self.offer_later();
					}
					self.close_card();
					self.give_back();
				}
				Ok(())
			}
			_ => Err(intent.unknown()),
		}
	}

	/// The standing offer's card's Later or ×: counted, its card closed, and the offer withdrawn on the devices when it
	/// went out to them (`Standing::later`); it still stands on Pump home and in the tray.
	fn offer_later(&mut self) {
		let Some((id, link, out)) = self.standing.current().map(|o| (o.id.clone(), o.link, o.out_at.is_some())) else {
			return;
		};
		self.counts_offer(link, "dismissed");
		let Some(t) = self.games.as_ref().map(Games::tunables) else { return };
		if self.standing.later(t) {
			if self.queue.withdraw(&id) {
				log::info!("offer {id}: put off on the computer, withdrawn on the devices and off the link");
			} else {
				log::info!("offer {id}: put off on the computer, withdrawn on the devices");
			}
		} else if !out {
			log::info!("offer {id}: answered on the computer, not sent to the devices");
		}
	}

	/// A start from the offer `offer` went through (DESK-PUMP's `offer_started`, for every Start from it: the card, the
	/// tray, the firmer choice, Pump home, the Games tab's Now card, Start a pump opened from it): the test's card goes;
	/// the standing offer's card closes for good, so the offer no longer goes out to the devices (`offer <id>: answered
	/// on the computer, not sent to the devices` when none heard of it), its first Start counts `started` (never the
	/// test's), and it stands until its pump starts.
	pub(crate) fn offer_answered(&mut self, offer: &str) {
		let now = lan::now_ms().div_euclid(1_000);
		if offer == TEST {
			self.offering.card.test = None;
		} else if let Some(o) = self.standing.current().filter(|o| o.id == offer) {
			if o.out_at.is_none() && self.standing.card(now) {
				log::info!("offer {offer}: answered on the computer, not sent to the devices");
			}
			let link = o.link;
			self.standing.close_card();
			if self.offering.card.started.as_deref() != Some(offer) {
				self.offering.card.started = Some(offer.to_owned());
				self.counts_offer(link, "started");
			}
		}
		if self.offering.card.shows(offer) {
			self.close_card();
		}
	}

	/// The card an id names: the test's, the standing offer's or the standing reminder's (none: the standing offer).
	fn card_offer(&self, asked: Option<&str>) -> Option<CardView> {
		let of = |o: &Offer| CardView::of(o, self.offer_one_size(Some(&o.id)));
		match asked {
			Some(TEST) => self.offering.card.test.as_ref().map(|test| test.view.clone()),
			Some(id) => self
				.standing
				.current()
				.filter(|o| o.id == id)
				.map(of)
				.or_else(|| self.reminder_view().filter(|card| card.id == id)),
			None => self.standing.current().map(of),
		}
	}

	/// An offer's Start, asked from outside the main window (the card, the tray, the firmer choice), which the pump
	/// opens: DESK-PUMP's `pump.start` at Ready for "A match" at `size`, else the offer's tier (`pump_start_intent`),
	/// every game's offer alike, never the Start a pump window. The pump opens the main window, which goes onto
	/// `screen` when given; the card goes and the Start counts (`offer_started`), and the offer stands until the pump
	/// starts.
	pub(crate) fn start_offer(
		&mut self,
		view: &CardView,
		size: Option<u8>,
		screen: Option<Screen>,
	) -> Result<(), String> {
		let mut ask = json!({"type": "pump.start", "kind": MATCH, "offer": view.id, "outside": true});
		if let Some(size) = size {
			ask["size"] = Value::from(size);
		}
		self.intent(&Intent::from_value(ask)?)?;
		let Some(screen) = screen else { return Ok(()) };
		// after the pump's own opening of the main window, queued on the main thread before this
		let handle = self.app.clone();
		self.app.run_on_main_thread(move || to_screen(&handle, screen)).map_err(|e| e.to_string())
	}

	/// "Open the pump after each match": the offer of the game `game` (`Watched::id`) was just made (a match's end, its
	/// close), its game closed with it standing, or the pump a move ran in as it came went idle
	/// (`Helper::offer_after_pump`), or the player's look at the game's last screen ended (`Standing::look_ends`, which
	/// it waits for): the offer's Start at its tier, on the game's screen as last seen, once, while nothing holds the
	/// offers back (`offers_held_apart`). An idle pump on screen ends for it (`desk_pump_make_way`:
	/// a catch-up's list, a Ready view, a Result; Rapha, 2026-10-28: "So should close catchup and jump into the new
	/// pump no? for all games."); while a move runs the offer waits for it (`CardState::wait_pump`).
	pub(crate) fn open_offer(&mut self, game: &str, now: i64) {
		if self.settings.games.open_on_close_off || self.offers_held_apart(now) {
			return;
		}
		let Some(offer) = self.standing.current() else { return };
		if self.offering.card.game_of(&offer.id) != Some(game)
			|| self.offering.card.opened.as_ref() == Some(&offer.id)
			|| offer.look_until.is_some()
		{
			return;
		}
		let id = offer.id.clone();
		if !self.desk_pump_make_way() {
			log::info!("offer {id}: a move runs here; its pump opens once the pump is idle");
			self.offering.card.wait_pump(&id);
			return;
		}
		let Some(offer) = self.standing.current().filter(|o| o.id == id) else { return };
		let view = CardView::of(offer, self.offer_one_size(Some(&offer.id)));
		let screen =
			self.offering.card.area_of(&view.id).and_then(|a| screens(&self.app).into_iter().find(|s| s.holds(&a)));
		self.offering.card.opened = Some(view.id.clone());
		self.offering.card.by_hand = false;
		log::info!("offer {}: the pump opens", view.id);
		if let Err(e) = self.start_offer(&view, view.tier, screen) {
			log::warn!("offer {}: {e}", view.id);
		}
	}

	/// The card closed by the player's own Later or ×: macOS hides QuestPump again when the click made it the active
	/// app, so the app in front before comes back: with no main window open, or with QuestPump not the active app as the
	/// card came (the window a pump the card or the tray started stays open, since 2026-10-25).
	#[cfg_attr(not(target_os = "macos"), expect(clippy::unused_self, reason = "a click on the card activates nothing"))]
	fn give_back(&self) {
		#[cfg(target_os = "macos")]
		{
			let handle = self.app.clone();
			let hid = self.app.run_on_main_thread(move || {
				let away = !ACTIVE_AT_CARD.load(std::sync::atomic::Ordering::Relaxed);
				if (handle.get_webview_window(MAIN).is_none() || away)
					&& app_active() && let Err(e) = handle.hide()
				{
					log::warn!("card: {e}");
				}
			});
			if let Err(e) = hid {
				log::warn!("card: {e}");
			}
		}
	}
}
