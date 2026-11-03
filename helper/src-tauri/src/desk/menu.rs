//! The macOS app menu, which shows while the main window is open (QuestPump is an ordinary app then), built explicitly
//! in place of Tauri's default one: its Quit sends `terminate:`, which ends the process without the loop's quit (no
//! pending mirror written, no "Leave the pump?"), and without an Edit menu Cmd-C and Cmd-V do nothing in the page's
//! text boxes. QuestPump: About, Hide, Hide Others, Show All, and Quit QuestPump (Cmd-Q) through `ui::quit`; Edit: the
//! system's items; View: Pump, Hero, Book, Saga and Games (Cmd-1 to Cmd-5, slices 46 and 56), which the page takes as
//! a press of that root's tab (`windows::go`, `chosen`), greyed while a pump's cover or a window stands over the page
//! (`tabs_on`), then Enter Full Screen and the zoom (Cmd-+, Cmd--, Cmd-0) through the loop, which keeps it in
//! settings.json (the web view's own zoom keys stay off: `zoomHotkeysEnabled` false); Window: Minimize and Close
//! Window, whose close is the window's own and asks first during a pump. A key with Cmd reaches the page first (`WebKit`'s
//! `WebViewImpl::performKeyEquivalent` hands it to the web content while the web view is the first responder and
//! resends it to the menu only where the page left it), and the page's `lib/keys.js` takes the zoom's and the tabs'
//! keys, so a key acts once; a menu item chosen with the mouse comes here. Windows has no app menu (Alt-F4 is the same
//! close; the tabs' keys are the page's alone). Its words come from `copy.rs` (the tabs' are the `ActionBar`'s own, the
//! `bar.*` keys, which name the screen `desk.app_menu`).
//!
//! The quit Apple event is QuestPump's own too (`catch_quit`): the Dock's Quit and Cmd-Tab's Q while the window makes
//! QuestPump an ordinary app, Activity Monitor's Quit, a logout, a restart or a shutdown, and System Settings' "Quit &
//! Reopen". `AppKit` would answer it with `terminate:` (tao's delegate has no `applicationShouldTerminate:`), which
//! ends the process without the loop's quit. The player's quit goes through `ui::quit` ("Leave the pump?" first while a
//! pump holds the window); the system's (the event names its reason) goes through the loop's quit at once, since nobody
//! waits on a page then. System Settings offers "Quit & Reopen" only once its toggle was turned on, so a quit is taken
//! for it only after the player pressed "Allow screen reading" (This computer's `asked`) while the grant is still
//! missing to this process: it starts QuestPump again (`update::reopen_after_quit`) and quits through the loop at once;
//! while a pump holds the window it is the player's quit, "Leave the pump?" first. Every other quit (the Dock's,
//! Cmd-Tab's Q on a computer that never asked for the grant) is the player's.

use std::sync::atomic::Ordering;

use objc2::rc::Retained;
use objc2::runtime::NSObject;
use objc2::{DefinedClass as _, MainThreadMarker, MainThreadOnly, define_class, msg_send, sel};
use objc2_foundation::{NSAppleEventDescriptor, NSAppleEventManager};
use tauri::menu::{AboutMetadata, IsMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{AppHandle, Manager as _};

use super::{Desk, DeskAsk};
use crate::ui::{self, Shared, WindowAsk};
use crate::{copy, update};

const QUIT: &str = "app.quit";
/// View's id, by which `tabs_on` finds its tabs.
const VIEW: &str = "app.view";
const ZOOM_IN: &str = "app.zoom_in";
const ZOOM_OUT: &str = "app.zoom_out";
const ZOOM_RESET: &str = "app.zoom_reset";
/// Apple's four-character codes (Core Services' `AppleEvents.h` and `AERegistry.h`): the quit event, and its reason,
/// which loginwindow sets for a logout, a restart, a shutdown or its quit of every app (`kAEReallyLogOut`,
/// `kAERestart`, `kAEShutDown`, `kAEQuitAll`).
const CORE_EVENT_CLASS: u32 = u32::from_be_bytes(*b"aevt");
const QUIT_APPLICATION: u32 = u32::from_be_bytes(*b"quit");
const QUIT_REASON: u32 = u32::from_be_bytes(*b"why?");

/// One of View's tabs (slice 46): its item's id, the root the page presses (its id in `helper/dist/kit/qp-root-host.js`
/// `ROOTS`) and its name (the tab's own words).
struct Tab {
	id: &'static str,
	root: &'static str,
	name: fn() -> &'static str,
}

/// View's tabs in the `ActionBar`'s order, on Cmd-1 to Cmd-5.
const TABS: [Tab; 5] = [
	Tab { id: "app.tab.pump", root: super::windows::ROOT, name: copy::bar_pump },
	Tab { id: "app.tab.hero", root: "hero", name: copy::bar_hero },
	Tab { id: "app.tab.book", root: "book", name: copy::bar_book },
	Tab { id: "app.tab.log", root: "log", name: copy::bar_log },
	Tab { id: "app.tab.games", root: "games", name: copy::bar_games },
];

define_class!(
	// SAFETY: NSObject has no subclassing requirements, and `QuitHandler` implements no `Drop`.
	#[unsafe(super(NSObject))]
	#[thread_kind = MainThreadOnly]
	#[name = "QuestPumpQuitHandler"]
	#[ivars = AppHandle]
	struct QuitHandler;

	impl QuitHandler {
		// SAFETY: the signature NSAppleEventManager calls a handler's selector with: the event and its reply, both
		// descriptors, and no result.
		#[unsafe(method(handleQuit:withReplyEvent:))]
		fn handle_quit(&self, event: &NSAppleEventDescriptor, _reply: &NSAppleEventDescriptor) {
			on_quit(self.ivars(), event);
		}
	}
);

/// The menu bar's menus.
pub(crate) fn build(app: &AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
	let about = AboutMetadata { name: Some(copy::desk_window_menu_app().to_owned()), ..AboutMetadata::default() };
	let quest_pump = Submenu::with_items(
		app,
		copy::desk_window_menu_app(),
		true,
		&[
			&PredefinedMenuItem::about(app, Some(copy::desk_window_menu_about()), Some(about))?,
			&PredefinedMenuItem::separator(app)?,
			&PredefinedMenuItem::hide(app, Some(copy::desk_window_menu_hide()))?,
			&PredefinedMenuItem::hide_others(app, Some(copy::desk_window_menu_hideothers()))?,
			&PredefinedMenuItem::show_all(app, Some(copy::desk_window_menu_showall()))?,
			&PredefinedMenuItem::separator(app)?,
			&MenuItem::with_id(app, QUIT, copy::desk_window_menu_quit(), true, Some("CmdOrCtrl+Q"))?,
		],
	)?;
	let edit = Submenu::with_items(
		app,
		copy::desk_window_menu_edit(),
		true,
		&[
			&PredefinedMenuItem::undo(app, Some(copy::desk_window_menu_undo()))?,
			&PredefinedMenuItem::redo(app, Some(copy::desk_window_menu_redo()))?,
			&PredefinedMenuItem::separator(app)?,
			&PredefinedMenuItem::cut(app, Some(copy::desk_window_menu_cut()))?,
			&PredefinedMenuItem::copy(app, Some(copy::desk_window_menu_copy()))?,
			&PredefinedMenuItem::paste(app, Some(copy::desk_window_menu_paste()))?,
			&PredefinedMenuItem::select_all(app, Some(copy::desk_window_menu_selectall()))?,
		],
	)?;
	let view = Submenu::with_id_and_items(
		app,
		VIEW,
		copy::desk_window_menu_view(),
		true,
		&[
			&PredefinedMenuItem::fullscreen(app, Some(copy::desk_window_menu_fullscreen()))?,
			&PredefinedMenuItem::separator(app)?,
			&MenuItem::with_id(app, ZOOM_IN, copy::desk_zoom_in(), true, Some("CmdOrCtrl+="))?,
			&MenuItem::with_id(app, ZOOM_OUT, copy::desk_zoom_out(), true, Some("CmdOrCtrl+-"))?,
			&MenuItem::with_id(app, ZOOM_RESET, copy::desk_zoom_reset(), true, Some("CmdOrCtrl+0"))?,
		],
	)?;
	// the five roots first, over a separator
	let tabs = TABS
		.iter()
		.enumerate()
		.map(|(at, tab)| MenuItem::with_id(app, tab.id, (tab.name)(), true, Some(format!("CmdOrCtrl+{}", at + 1))))
		.collect::<tauri::Result<Vec<_>>>()?;
	let separator = PredefinedMenuItem::separator(app)?;
	let mut first: Vec<&dyn IsMenuItem<tauri::Wry>> = tabs.iter().map(|tab| tab as &dyn IsMenuItem<_>).collect();
	first.push(&separator);
	view.prepend_items(&first)?;
	let window = Submenu::with_items(
		app,
		copy::desk_window_menu_window(),
		true,
		&[
			&PredefinedMenuItem::minimize(app, Some(copy::desk_window_menu_minimize()))?,
			&PredefinedMenuItem::close_window(app, Some(copy::desk_window_menu_close()))?,
		],
	)?;
	Menu::with_items(app, &[&quest_pump, &edit, &view, &window])
}

/// Greys View's five tabs while they cannot act (`on` false: a pump's cover or a window stands over the page, the
/// `ActionBar` out of reach behind it; the page's `view.tabs`, `lib/router.js` `tellTabs`), as the iPad greys its
/// commands; the page refuses a tab then too (`choose`), whose key never reaches the menu. On the main thread, where
/// an item's `set_enabled` runs at once.
pub(crate) fn tabs_on(app: &AppHandle, on: bool) {
	let Some(view) = app.menu().and_then(|menu| menu.get(VIEW)).and_then(|item| item.as_submenu().cloned()) else {
		return log::warn!("menu: no View menu to grey its tabs in");
	};
	for tab in &TABS {
		if let Some(item) = view.get(tab.id).and_then(|item| item.as_menuitem().cloned())
			&& let Err(e) = item.set_enabled(on)
		{
			log::warn!("menu: {e}");
		}
	}
}

/// Makes the quit Apple event QuestPump's own (in the app's setup, on the main thread). The app has finished launching
/// by then, so this replaces the handler `AppKit` installs before `applicationWillFinishLaunching:`.
pub(crate) fn catch_quit(app: &AppHandle) {
	let Some(mtm) = MainThreadMarker::new() else {
		log::error!("quit: not on the main thread, so the system's quit ends QuestPump without the loop");
		return;
	};
	let handler: Retained<QuitHandler> = {
		let this = QuitHandler::alloc(mtm).set_ivars(app.clone());
		// SAFETY: NSObject's `init` on the allocated object whose ivars are set.
		unsafe { msg_send![super(this), init] }
	};
	let manager = NSAppleEventManager::sharedAppleEventManager();
	// SAFETY: setEventHandler:andSelector:forEventClass:andEventID: takes an object that answers the selector with the
	// (event, reply) signature above and two FourCharCodes; it keeps no strong reference to the handler, which
	// therefore stays alive for the process (`forget` below).
	unsafe {
		let () = msg_send![
			&*manager,
			setEventHandler: &*handler,
			andSelector: sel!(handleQuit:withReplyEvent:),
			forEventClass: CORE_EVENT_CLASS,
			andEventID: QUIT_APPLICATION
		];
	}
	std::mem::forget(handler);
}

/// Whether a quit event is the system's: it names its reason (Apple's header calls it a parameter, the event carries it
/// as an attribute: either is read).
fn system_quit(event: &NSAppleEventDescriptor) -> bool {
	// SAFETY: attributeDescriptorForKeyword: and paramDescriptorForKeyword: take a FourCharCode and return a descriptor
	// or nil.
	let reason: Option<Retained<NSAppleEventDescriptor>> = unsafe {
		let attribute: Option<Retained<NSAppleEventDescriptor>> =
			msg_send![event, attributeDescriptorForKeyword: QUIT_REASON];
		attribute.or_else(|| msg_send![event, paramDescriptorForKeyword: QUIT_REASON])
	};
	reason.is_some_and(|r| r.typeCodeValue() != 0)
}

/// A quit Apple event came.
fn on_quit(app: &AppHandle, event: &NSAppleEventDescriptor) {
	if system_quit(event) {
		log::info!("quit: the system's (a logout, a restart or a shutdown)");
		return ui::quit_now(app, None);
	}
	let shared = app.state::<Shared>();
	let reopen = shared.screen_missing.load(Ordering::Relaxed) && shared.view.lock().is_ok_and(|v| v.computer.asked);
	if reopen && !app.state::<Desk>().guard.load(Ordering::Relaxed) {
		log::info!("quit: asked after the player asked for screen reading, so QuestPump starts again");
		if let Err(e) = update::reopen_after_quit() {
			log::warn!("starting again: {e:#}");
		}
		return ui::quit_now(app, None);
	}
	ui::quit(app);
}

/// The app menu's own items (the system's run themselves).
pub(crate) fn on_menu(app: &AppHandle, event: &MenuEvent) {
	if let Some(tab) = TABS.iter().find(|tab| event.id.as_ref() == tab.id) {
		return super::windows::go(app, tab.root, true);
	}
	let step = match event.id.as_ref() {
		QUIT => return ui::quit(app),
		ZOOM_IN => 1,
		ZOOM_OUT => -1,
		ZOOM_RESET => 0,
		_ => return,
	};
	app.state::<Shared>().ask(WindowAsk::Desk(DeskAsk::Zoom(step)));
}
