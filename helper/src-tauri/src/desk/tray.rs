//! The tray icon (macOS's menu bar, Windows' notification area): the mark's silhouette in the capture's colour
//! (`Tone`, from `ui::Dot`: grey, amber, green, red; the 44-px set in macOS's menu bar, the 32-px set in Windows' tray,
//! icons/tray-*.png), its tooltip `QuestPump: <state>`, and its menu: this computer's state, which only shows, and the
//! phone's line, which while no phone is paired reads "Pair your phone…" and opens the Menu's Phones page, the feedback
//! note under them once there is one, the streak's line, which only shows (`reminders.rs`), the pump's items (Quick pump
//! and Catch up where they can start, or a line while a pump is on: `pump.rs`), then Open QuestPump, the offer's items
//! while they are due (Pump for {game}, `games.pump`, and Pause offers: `offers.rs`) and Quit QuestPump. Its words come
//! from `copy.rs`. This computer's state (`State`, from the view: a step left, the game read, waited for, out of view,
//! without QuestPump, found, no game known but the games it watches, or none known) and the phone's line are worded
//! here once, by the tray's rule, and published in their words beside their state (`ui::update`), so Pump home's Chip
//! and This computer say what the tray says; the icon's colour likewise (`Tone`), which Pump home's Chip shows as its
//! dot.
//! An area's items join the menu through `TrayLines::show` at their place, as the note does, and their events through
//! `on_menu`'s match (one line per area). The one tray icon is built here, never by a `trayIcon` key in tauri.conf.json,
//! which would make a second.

use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;
use tauri::image::Image;
use tauri::menu::{IsMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager as _};

use super::offers::{self, TrayItems};
use super::seams::FoundGame;
use super::windows::{self, Landing};
use crate::copy;
use crate::ui::{self, Access, Dot, Holder, Note, PhoneLine, View};

const TRAY: &str = "main";
const OPEN: &str = "tray.open";
const QUIT: &str = "tray.quit";
/// The phone's line, which opens the Phones page while no phone is paired.
const PHONE: &str = "tray.phone";
/// The note's place: under the state and the phone's lines.
const NOTE_AT: usize = 2;

/// What the tray draws, from the view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TrayState {
	pub(crate) dot: Dot,
	/// This computer's state in words (`state_line`).
	pub(crate) state: String,
	pub(crate) phone: PhoneLine,
	pub(crate) note: Option<Note>,
}

/// This computer's state (one owner: the tray's first line and its tooltip, Pump home's Chip, This computer's line): a
/// step left first, then the game as QuestPump reads it and finds it. The phone has its own line (`phone_line`).
#[derive(Serialize, Debug, Clone, Copy, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum State {
	/// A step is left on This computer: the grant a device needs, the folder's rights, the move to Applications, or
	/// the screen reading stopped.
	Step,
	/// The game's squares are read.
	Reading,
	/// The game with QuestPump runs and its squares are not read yet in this run of it.
	Waiting,
	/// The game with QuestPump runs, its squares were read in this run of it, and they have been out of view for the
	/// runtime's `OUT_AFTER` (a loading screen, a movie, the interface hidden, the game minimised or on another Space).
	Out,
	/// A game runs without QuestPump in its AddOns folder, or the game here has no QuestPump yet.
	Lacks,
	/// The game with QuestPump is found: not running, or running while nothing asks to read it (the grant waits for a
	/// device); its progress is kept for it.
	Found,
	/// No game is known, and QuestPump watches other games for their matches' end (`Computer::watching`).
	Watching,
	/// No game with QuestPump is known, and no other game is watched.
	#[default]
	#[serde(rename = "none")]
	Unfound,
}

/// This computer's state in the view: a step left first (as This computer's cards ask it), then the squares read, then
/// what QuestPump found of the game (`Helper::found`'s `game`, kept in `computer.found`), a running client without
/// QuestPump before a found install, so "Waiting…" only ever means a game with QuestPump not read yet in this run of it
/// (`computer.game`'s `read`), and one read before and out of view now is `Out`.
pub(crate) fn state(view: &View) -> State {
	let c = &view.computer;
	let dot = view.shell.dot;
	if c.permission || c.access != Access::Fine || c.unmoved || dot == Dot::Stopped {
		return State::Step;
	}
	if dot == Dot::Live {
		return State::Reading;
	}
	let read = c.game.as_ref().is_some_and(|g| g.read);
	match c.found {
		FoundGame::Running if dot == Dot::NoStrip && read => State::Out,
		FoundGame::Running if dot == Dot::NoStrip => State::Waiting,
		FoundGame::Running => State::Found,
		// the client running has no QuestPump, whatever install holds it
		_ if c.game.is_some() => State::Lacks,
		FoundGame::Found => State::Found,
		FoundGame::Installed => State::Lacks,
		FoundGame::Unfound if !c.watching.is_empty() => State::Watching,
		FoundGame::Unfound => State::Unfound,
	}
}

/// This computer's state in words: the tray's first line, Pump home's Chip and This computer's line; `watching` the
/// games watched by name, the one named and several counted.
pub(crate) fn state_line(state: State, watching: &[String]) -> String {
	match state {
		State::Step => copy::desk_tray_state_step().to_owned(),
		State::Reading => copy::desk_tray_state_reading().to_owned(),
		State::Waiting => copy::desk_tray_state_waiting().to_owned(),
		State::Out => copy::desk_tray_state_out().to_owned(),
		State::Lacks => copy::desk_tray_state_lacks().to_owned(),
		State::Found => copy::desk_tray_state_found().to_owned(),
		State::Watching => match watching {
			[game] => copy::desk_tray_state_watching(game),
			games => copy::desk_tray_state_watching_games(i64::try_from(games.len()).unwrap_or(i64::MAX)),
		},
		State::Unfound => copy::desk_tray_state_none().to_owned(),
	}
}

/// The tray menu's lines that change, and the menu itself.
pub(crate) struct TrayLines {
	menu: Menu<tauri::Wry>,
	state: MenuItem<tauri::Wry>,
	phone: MenuItem<tauri::Wry>,
	note: MenuItem<tauri::Wry>,
	/// The note is in the menu (while it has a text).
	note_shown: AtomicBool,
	/// The offer's items (`offers.rs`).
	pub(crate) offers: TrayItems,
}

impl TrayLines {
	/// Where an area's item goes: just above Open QuestPump (the separator, the state's and the phone's lines and the
	/// note above it).
	pub(crate) fn above_open(&self) -> usize {
		self.menu.items().ok().and_then(|items| items.iter().position(|i| i.id() == OPEN)).unwrap_or(0)
	}

	/// Puts an item into the menu at `at`, or takes it out, once: `shown` keeps whether it is in.
	pub(crate) fn show(&self, item: &dyn IsMenuItem<tauri::Wry>, shown: &AtomicBool, at: usize, on: bool) {
		if on && !shown.swap(true, Ordering::Relaxed) {
			let _ = self.menu.insert(item, at);
		} else if !on && shown.swap(false, Ordering::Relaxed) {
			let _ = self.menu.remove(item);
		}
	}

	/// Where an item goes to stand above the pump's items (Quick pump, Catch up, or the line while a pump is on), else
	/// above Open QuestPump: the reminders' streak line.
	pub(crate) fn above_pumps(&self) -> usize {
		let first = |items: Vec<tauri::menu::MenuItemKind<tauri::Wry>>| {
			items.iter().position(|i| i.id().as_ref().starts_with("pump.") || i.id() == OPEN)
		};
		self.menu.items().ok().and_then(first).unwrap_or(0)
	}

	/// Where an item goes to stand right under Open QuestPump.
	pub(crate) fn after_open(&self) -> usize {
		self.place_of(OPEN).map_or(0, |at| at + 1)
	}

	/// Where an item goes to stand right over Quit QuestPump.
	pub(crate) fn before_quit(&self) -> usize {
		self.place_of(QUIT).unwrap_or(0)
	}

	fn place_of(&self, id: &str) -> Option<usize> {
		self.menu.items().ok()?.iter().position(|item| item.id() == id)
	}
}

/// The four state pictures of one system's set (`tools/icons.py`, RASTERS "tray").
struct DotSet {
	grey: &'static [u8],
	amber: &'static [u8],
	green: &'static [u8],
	red: &'static [u8],
}

/// macOS's menu bar takes the 44-px set (drawn 18 pt tall).
#[cfg(target_os = "macos")]
const DOTS: DotSet = DotSet {
	grey: include_bytes!("../../icons/tray-grey.png"),
	amber: include_bytes!("../../icons/tray-amber.png"),
	green: include_bytes!("../../icons/tray-green.png"),
	red: include_bytes!("../../icons/tray-red.png"),
};

/// Windows' tray takes the 32-px set (whole at 200 %, scaled to its 16-px icon at 100 %).
#[cfg(not(target_os = "macos"))]
const DOTS: DotSet = DotSet {
	grey: include_bytes!("../../icons/tray-grey-32.png"),
	amber: include_bytes!("../../icons/tray-amber-32.png"),
	green: include_bytes!("../../icons/tray-green-32.png"),
	red: include_bytes!("../../icons/tray-red-32.png"),
};

/// The colour of this computer's dot (one owner: the tray icon's picture and Pump home's Chip while this computer's
/// state needs the player), named as the Chip names its dots (`helper/dist/kit/qp-chip.js`'s `state`).
#[derive(Serialize, Debug, Clone, Copy, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Tone {
	/// Grey: no game runs, or the Screen Recording grant is missing while nothing needs it.
	#[default]
	Off,
	/// Amber: the game runs and its squares are not in view.
	Reaching,
	/// Green: the squares are read.
	Ok,
	/// Red: the player is needed (the grant missing while a device needs it, the stream stopped, a strip that does
	/// not read, QuestPump could not start).
	Stopped,
}

/// The dot's colour for the capture's state (`ui::Dot`).
pub(crate) fn tone(dot: Dot) -> Tone {
	match dot {
		Dot::NoGame | Dot::Off => Tone::Off,
		Dot::NoStrip => Tone::Reaching,
		Dot::Live => Tone::Ok,
		Dot::Stopped => Tone::Stopped,
	}
}

fn dot_icon(dot: Dot) -> Option<Image<'static>> {
	let bytes = match tone(dot) {
		Tone::Off => DOTS.grey,
		Tone::Reaching => DOTS.amber,
		Tone::Ok => DOTS.green,
		Tone::Stopped => DOTS.red,
	};
	Image::from_bytes(bytes).ok()
}

/// The icon's tooltip: `QuestPump: <state>`.
fn tooltip(state: &str) -> String {
	copy::desk_tray_tooltip(state)
}

/// The phone's line: the paired phone, its watch and the device that gets the countdowns now (the tray's, the Menu's,
/// the Phones page's and the welcome's); the tray's own while none is paired is `tray_phone`'s.
pub(crate) fn phone_line(phone: &PhoneLine) -> String {
	let Some(name) = &phone.name else { return copy::desk_tray_phone_none().to_owned() };
	match (phone.watch, phone.holder) {
		(false, None) => copy::desk_tray_phone_paired(name),
		(false, Some(_)) => copy::desk_tray_phone_paired_linked(name),
		(true, None) => copy::desk_tray_phone_watch(name),
		(true, Some(Holder::Phone)) => copy::desk_tray_phone_watch_phone(name),
		(true, Some(Holder::Watch)) => copy::desk_tray_phone_watch_linked(name),
	}
}

/// The tray's phone item: the phone's line, or "Pair your phone…" while none is paired, when a click opens the Phones
/// page.
fn tray_phone(phone: &PhoneLine) -> String {
	if phone.name.is_some() { phone_line(phone) } else { copy::desk_tray_phone_pair().to_owned() }
}

/// A local time of day such as "14:02" (UTC where the computer's offset cannot be read).
pub(crate) fn clock_at(ms: i64) -> String {
	let at = time::OffsetDateTime::from_unix_timestamp(ms.div_euclid(1000)).unwrap_or(time::OffsetDateTime::UNIX_EPOCH);
	let at = at.to_offset(time::UtcOffset::local_offset_at(at).unwrap_or(time::UtcOffset::UTC));
	format!("{:02}:{:02}", at.hour(), at.minute())
}

/// The feedback note's line (the tray's and the form's).
pub(crate) fn note_line(note: &Note) -> String {
	match *note {
		Note::Sent { at } => copy::desk_feedback_note_sent(&clock_at(at)),
		Note::Refused { at } => copy::desk_feedback_note_refused(&clock_at(at)),
		Note::Lost { at, game: false } => copy::desk_feedback_note_lost(&clock_at(at)),
		Note::Lost { at, game: true } => copy::desk_feedback_note_lost_game(&clock_at(at)),
		Note::Soon => copy::desk_feedback_note_soon().to_owned(),
		Note::Later { minutes } => copy::desk_feedback_note_later(i64::try_from(minutes).unwrap_or(i64::MAX)),
	}
}

/// The menu's events: one line per area.
fn on_menu(app: &AppHandle, event: &MenuEvent) {
	match event.id.as_ref() {
		OPEN => windows::open_main(app, Landing::Here),
		PHONE => windows::open_main(app, Landing::Phones),
		QUIT => ui::quit(app),
		id if offers::is_tray(id) => offers::on_tray(app, id),
		id if id.starts_with("pump.") => super::pump::on_tray(app, id),
		_ => {}
	}
}

/// The one tray icon, with its menu.
pub(crate) fn build(app: &tauri::App) -> tauri::Result<()> {
	let (dot, now) = (Dot::default(), state_line(State::default(), &[]));
	let state = MenuItem::with_id(app, "tray.state", &now, false, None::<&str>)?;
	// no phone is known before the settings are read: the item asks to pair one until a phone's line replaces it
	let phone = MenuItem::with_id(app, PHONE, tray_phone(&PhoneLine::default()), true, None::<&str>)?;
	let open = MenuItem::with_id(app, OPEN, copy::desk_tray_open(), true, None::<&str>)?;
	let quit = MenuItem::with_id(app, QUIT, copy::desk_tray_quit(), true, None::<&str>)?;
	let note = MenuItem::with_id(app, "tray.note", "", false, None::<&str>)?;
	let line = PredefinedMenuItem::separator(app)?;
	let menu = Menu::with_items(app, &[&state, &phone, &line, &open, &quit])?;
	let offers = offers::tray_items(app)?;
	app.manage(TrayLines { menu: menu.clone(), state, phone, note, note_shown: AtomicBool::new(false), offers });
	let mut tray = TrayIconBuilder::with_id(TRAY)
		.tooltip(tooltip(&now))
		.menu(&menu)
		.show_menu_on_left_click(true)
		.on_menu_event(|app, event| on_menu(app, &event));
	if let Some(icon) = dot_icon(dot) {
		tray = tray.icon(icon);
	}
	tray.build(app)?;
	Ok(())
}

/// Draws what changed between two states (on the main thread).
pub(crate) fn redraw(app: &AppHandle, before: &TrayState, now: &TrayState) {
	let lines = app.try_state::<TrayLines>();
	let tray = app.tray_by_id(TRAY);
	if before.dot != now.dot
		&& let Some(tray) = &tray
	{
		let _ = tray.set_icon(dot_icon(now.dot));
	}
	if before.state != now.state {
		if let Some(tray) = &tray {
			let _ = tray.set_tooltip(Some(tooltip(&now.state)));
		}
		if let Some(lines) = &lines {
			let _ = lines.state.set_text(&now.state);
		}
	}
	let Some(lines) = lines else { return };
	if before.phone != now.phone {
		let _ = lines.phone.set_text(tray_phone(&now.phone));
		let _ = lines.phone.set_enabled(now.phone.name.is_none());
	}
	if before.note != now.note {
		if let Some(note) = &now.note {
			let _ = lines.note.set_text(note_line(note));
		}
		lines.show(&lines.note, &lines.note_shown, NOTE_AT, now.note.is_some());
	}
}
