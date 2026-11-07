//! The main window (docs/ui.md, "The window model"): built from `tauri.conf.json`'s `main` when the player opens it
//! (the tray's Open QuestPump, a launch from the Dock, Applications or the Start menu, a second launch) and destroyed
//! when it closes, so its web view is released and QuestPump idles in the menu bar or the tray. While it is open
//! QuestPump is an ordinary app on macOS (a Dock icon and Cmd-Tab: the activation policy switched to Regular before it
//! opens and back to Accessory once it is gone). Its place (a second after each move or resize ends, and at its
//! close), its size (the page at `BASE` times the player's step) and Big view's full screen are kept in
//! settings.json; the window's own full screen, whatever turned it on, goes to the page as the `shell` topic's
//! `fullscreen_now`. A close while a pump holds the window (`Desk::guard`) asks "Leave the pump?" instead; its focus
//! goes to the page as the `shell` topic's `focused` (Rust's `WindowEvent::Focused`, never the page's blur, which a
//! click inside `WebView2` can fake). A pump's end leaves the window open on the page under it, whoever started the
//! pump (the card's and the tray's handed the focus back and closed it until 2026-10-25). On Windows the web view's own
//! browser keys (find, print, reload, zoom, the keyboard's browser keys) are off in every window of the page
//! (`browser_keys_off`).

use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use questpump_desk::settings::Place;
use tauri::utils::config::WindowConfig;
use tauri::{AppHandle, LogicalSize, Manager as _, Url, WebviewWindow, WebviewWindowBuilder, WindowEvent};

use super::{Desk, DeskAsk, Intent};
use crate::runtime::Helper;
use crate::ui::{self, Leave, Shared, Then, WindowAsk};

/// The main window's label (tauri.conf.json).
pub(crate) const MAIN: &str = "main";
/// The page a fresh window shows: the Pump root.
pub(crate) const ROOT: &str = "pump";
/// The page a fresh window shows instead while crash reports wait for the player's answer (`Desk::crash_page`).
const CRASH: &str = "desk.crash";
/// The page a fresh window shows instead while a switch of it is not answered (`Desk::privacy_page`), once a run.
const PRIVACY: &str = "desk.privacy";
/// The page's own scale on the computer, which every step of `ZOOMS` multiplies (David, 2026-10-15: "Also in the
/// desktop app everything is just too small"): the kit's sizes are the phones' points as CSS px, which at a desk's
/// distance read at 74-87 % of their size on a phone in the hand, and 125 % brings them back (docs/ui.md, "The window
/// model"), on the 4-px grid's whole pixels. 100 % is this scale; the card over the game keeps the page's own
/// (`card.rs`).
pub(crate) const BASE: f64 = 1.25;
/// The player's steps of the page's size, of `BASE` (Cmd/Ctrl + and -, the macOS View menu, the Menu's Size on This
/// computer): 1 is 100 %, the default; 2, 200 %, the most the layout is held to (a 480-px window then 192 CSS px wide).
pub(crate) const ZOOMS: [f64; 8] = [0.8, 0.9, 1.0, 1.1, 1.25, 1.5, 1.75, 2.0];
/// How long the window's place stands still after a move or a resize before it goes into settings.json (the loop's
/// 250-ms tick looks, `Helper::place_settled`): a drag writes it once, and a crash after it keeps it.
const SETTLE: Duration = Duration::from_secs(1);

/// Where an opening takes the page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Landing {
	/// Where it is: an open window comes forward as it stands, a new one shows the Pump root (or, once, the crash
	/// reports' page while they wait for the player's answer, else, once a run, the privacy page while a switch of it is
	/// not answered).
	Here,
	/// The Menu's This computer page: the grant, the move to Applications.
	Computer,
	/// The Menu's Phones page: the tray's "Pair your phone…".
	Phones,
}

impl Landing {
	fn page(self) -> Option<&'static str> {
		match self {
			Self::Here => None,
			Self::Computer => Some("desk.menu.computer"),
			Self::Phones => Some("desk.menu.phones"),
		}
	}
}

/// Opens the main window, or brings it forward, and takes the page where `landing` says (on the main thread: the tray,
/// the app menu, a second launch and the loop's start call it).
pub(crate) fn open_main(app: &AppHandle, landing: Landing) {
	let open = app.get_webview_window(MAIN);
	let desk = app.state::<Desk>();
	let fresh = open.is_none() && landing == Landing::Here;
	let page = landing
		.page()
		.or_else(|| (fresh && desk.crash_page.swap(false, Ordering::Relaxed)).then_some(CRASH))
		.or_else(|| (fresh && desk.privacy_page.swap(false, Ordering::Relaxed)).then_some(PRIVACY));
	if open.is_none() || page.is_some() {
		go(app, page.unwrap_or(ROOT), false);
	}
	if let Some(window) = open {
		let _ = window.unminimize();
		let _ = window.show();
		forward(app, &window);
		return;
	}
	let Some(config) = app.config().app.windows.iter().find(|w| w.label == MAIN).cloned() else { return };
	// macOS: an ordinary app while the window is open, so the game's player finds it again with Cmd-Tab
	#[cfg(target_os = "macos")]
	let _ = app.set_activation_policy(tauri::ActivationPolicy::Regular);
	let built = WebviewWindowBuilder::from_config(app, &config)
		.map(|b| b.visible(false).on_navigation(own_pages(app, &config)).build());
	let window = match built {
		Ok(Ok(window)) => window,
		Ok(Err(e)) | Err(e) => {
			log::error!("window: {e}");
			#[cfg(target_os = "macos")]
			let _ = app.set_activation_policy(tauri::ActivationPolicy::Accessory);
			return;
		}
	};
	browser_keys_off(&window);
	// the place once the window's frame is known, before it shows: the size first, since macOS keeps a window's bottom
	// edge as its size changes and its top-left corner as it moves
	let chrome = super::place::chrome(&window).unwrap_or_default();
	let frame = super::place::restore(app, desk.place().as_ref(), chrome);
	let _ = window.set_size(LogicalSize::new(frame.width, frame.height));
	if let Some(position) = frame.position {
		let _ = window.set_position(position);
	}
	set_minimum(&window);
	// at every opening, the default step too: the web view starts at its own 100 %
	if let Err(e) = window.set_zoom(BASE * ui::view(app).shell.zoom) {
		log::warn!("zoom: {e}");
	}
	let handle = app.clone();
	window.on_window_event(move |event| on_event(&handle, event));
	let _ = window.show();
	if frame.maximized {
		let _ = window.maximize();
	}
	forward(app, &window);
}

/// Where a window of the page may go: QuestPump's own pages alone, at Tauri's origin for them (`tauri://localhost` on
/// macOS, `http://tauri.localhost` on Windows, `https` with the window's `useHttpsScheme`) or, under `tauri dev`, at the
/// dev server's, so nothing takes a window elsewhere (a link dropped on it, a page's bug); another page opens in the
/// browser (`about::open_url`). The main window and the card take it (`card::open`); Tauri refuses every new window a
/// page asks for already.
pub(crate) fn own_pages(app: &AppHandle, config: &WindowConfig) -> impl Fn(&Url) -> bool + Send + 'static {
	let own = match (cfg!(windows), config.use_https_scheme) {
		(true, true) => "https://tauri.localhost",
		(true, false) => "http://tauri.localhost",
		(false, _) => "tauri://localhost",
	};
	let dev = app.config().build.dev_url.clone().filter(|_| tauri::is_dev());
	let origins: Vec<Url> = Url::parse(own).ok().into_iter().chain(dev).collect();
	move |url| {
		let ours = origins.iter().any(|o| {
			o.scheme() == url.scheme()
				&& o.host_str() == url.host_str()
				&& o.port_or_known_default() == url.port_or_known_default()
		});
		if !ours {
			log::warn!("window: a page went for {}://{}, refused", url.scheme(), url.host_str().unwrap_or_default());
		}
		ours
	}
}

/// macOS: how long after an opening QuestPump asks again to be the active app while it is not (`forward`), each wait
/// after the one before.
#[cfg(target_os = "macos")]
const FORWARD_AGAIN: [Duration; 3] =
	[Duration::from_millis(100), Duration::from_millis(200), Duration::from_millis(400)];

/// Brings the main window, open and shown, to the front and QuestPump with it, at every opening (`open_main`: the
/// game's close with "Open the pump after each match", the card's and the tray's Start, the tray, a second launch).
/// macOS: tao's `set_focus` (`makeKeyAndOrderFront:`, then `activateIgnoringOtherApps:`), asked again after each of
/// `FORWARD_AGAIN` while QuestPump is not active, since the window server refuses an app whose new window is not on
/// screen yet ("presents 0 windows ... Denying the request", seen on 2026-10-26 when a League close built the window).
/// Windows: never tao's `set_focus`, whose fallback sends a synthetic Alt key through `SendInput` (no input to a game,
/// not even a key: `docs/companion.md`), but `SetForegroundWindow`; where Windows refuses it (the last input went to
/// another app), the window comes above the others without activation and its taskbar button flashes until the player
/// comes to it.
fn forward(app: &AppHandle, window: &WebviewWindow) {
	#[cfg(target_os = "macos")]
	{
		let _ = window.set_focus();
		let handle = app.clone();
		std::thread::spawn(move || {
			for wait in FORWARD_AGAIN {
				std::thread::sleep(wait);
				let (sent, got) = std::sync::mpsc::channel();
				let on_main = handle.clone();
				let asked = handle.run_on_main_thread(move || {
					let again = !super::card::app_active()
						&& on_main.get_webview_window(MAIN).is_some_and(|w| w.set_focus().is_ok());
					let _ = sent.send(again);
				});
				if asked.is_err() || !got.recv().unwrap_or(false) {
					return;
				}
			}
		});
	}
	#[cfg(windows)]
	{
		let _ = app;
		if let Err(e) = to_front(window) {
			log::warn!("window: {e}");
		}
	}
}

/// Windows: the window made the foreground window, or, where Windows refuses that, raised above the other windows
/// without activation (into the topmost band and out of it again) with its taskbar button flashing until it is.
#[cfg(windows)]
fn to_front(window: &WebviewWindow) -> Result<(), String> {
	use ::windows::Win32::UI::WindowsAndMessaging::{
		FLASHW_TIMERNOFG, FLASHW_TRAY, FLASHWINFO, FlashWindowEx, HWND_NOTOPMOST, HWND_TOPMOST, SWP_NOACTIVATE,
		SWP_NOMOVE, SWP_NOSIZE, SetForegroundWindow, SetWindowPos,
	};
	let hwnd = window.hwnd().map_err(|e| e.to_string())?;
	let flash = FLASHWINFO {
		cbSize: u32::try_from(size_of::<FLASHWINFO>()).map_err(|e| e.to_string())?,
		hwnd,
		dwFlags: FLASHW_TRAY | FLASHW_TIMERNOFG,
		uCount: 0,
		dwTimeout: 0,
	};
	let keep = SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE;
	// SAFETY: `hwnd` is the main window's live window (Tauri hands it out while the window exists) and this runs on the
	// main thread that owns it; SetForegroundWindow asks for the foreground, SetWindowPos only reorders the window,
	// moving and sizing nothing, and FlashWindowEx reads `flash` while the call lasts.
	unsafe {
		if SetForegroundWindow(hwnd).as_bool() {
			return Ok(());
		}
		SetWindowPos(hwnd, Some(HWND_TOPMOST), 0, 0, 0, 0, keep).map_err(|e| e.to_string())?;
		SetWindowPos(hwnd, Some(HWND_NOTOPMOST), 0, 0, 0, 0, keep).map_err(|e| e.to_string())?;
		let _ = FlashWindowEx(&raw const flash);
	}
	Ok(())
}

/// Takes the page to `to` (a root's id or a Menu page's) once, `chosen` where the player chose that root in the View
/// menu: the `shell` topic's `go`, which the page takes once per `seq` (lib/router.js).
pub(crate) fn go(app: &AppHandle, to: &'static str, chosen: bool) {
	ui::update(app, |v| v.shell.go = ui::Go { to, seq: v.shell.go.seq + 1, chosen });
}

/// Closes the main window as its own close button does (the guard asks first while a pump holds it).
pub(crate) fn close_main(app: &AppHandle) {
	if let Some(window) = app.get_webview_window(MAIN) {
		let _ = window.close();
	}
}

/// The window's least size on its screen now.
fn set_minimum(window: &WebviewWindow) {
	if let Some((width, height)) = super::place::minimum(window) {
		let _ = window.set_min_size(Some(LogicalSize::new(width, height)));
	}
}

/// The place kept as the window moves and resizes, written once it settles (`Helper::place_settled`), when it closes
/// and when QuestPump quits.
fn track(app: &AppHandle, window: &WebviewWindow) {
	let desk = app.state::<Desk>();
	let Ok(mut held) = desk.place.lock() else { return };
	if let Some(place) = super::place::capture(window, held.as_ref())
		&& held.as_ref() != Some(&place)
	{
		*held = Some(place);
		if let Ok(mut moved) = desk.moved.lock() {
			*moved = Some(Instant::now());
		}
	}
}

/// The main window's events.
fn on_event(app: &AppHandle, event: &WindowEvent) {
	match event {
		WindowEvent::CloseRequested { api, .. } => {
			if app.state::<Desk>().guard.load(Ordering::Relaxed) {
				api.prevent_close();
				ui::update(app, |v| v.shell.leave = Some(Leave::after(v.shell.leave.as_ref(), Then::Close, None)));
				return;
			}
			if let Some(window) = app.get_webview_window(MAIN) {
				track(app, &window);
			}
			if let Some(place) = app.state::<Desk>().place() {
				app.state::<Shared>().ask(WindowAsk::Desk(DeskAsk::Place(place)));
			}
		}
		WindowEvent::Destroyed => {
			app.state::<Desk>().publisher.forget(MAIN);
			// its pages went with it: the Phones page and the pairing code show nowhere (the Bluetooth link, slice 43)
			app.state::<Shared>().ask(WindowAsk::PhonesShown { page: false, code: false });
			// a day reel being made goes with the window that asked for it (slice 45)
			super::reel::cancel(app);
			// and the Book's move pages, whose sets the Book carried for Correct a count (slice 44)
			app.state::<Shared>().ask(WindowAsk::Desk(DeskAsk::Gone));
			ui::update(app, |v| {
				v.shell.focused = false;
				v.shell.fullscreen_now = false;
				v.shell.leave = None;
			});
			// back in the menu bar: no Dock icon, no Cmd-Tab
			#[cfg(target_os = "macos")]
			let _ = app.set_activation_policy(tauri::ActivationPolicy::Accessory);
		}
		WindowEvent::Focused(focused) => {
			let focused = *focused;
			// a screen reader turned on or off while the window was away shows at its return (the Book's Tree | List
			// default, the `shell` topic's `reader`)
			let reader = super::audio::screen_reader();
			ui::update(app, |v| {
				v.shell.focused = focused;
				v.shell.reader = reader;
			});
		}
		WindowEvent::Moved(_) => {
			if let Some(window) = app.get_webview_window(MAIN) {
				track(app, &window);
			}
		}
		// full screen comes and goes with a resize, whatever asked for it (Big view, the View menu, the green button)
		WindowEvent::Resized(_) => {
			if let Some(window) = app.get_webview_window(MAIN) {
				track(app, &window);
				let now = window.is_fullscreen().unwrap_or(false);
				ui::update(app, |v| v.shell.fullscreen_now = now);
			}
		}
		WindowEvent::ScaleFactorChanged { .. } => {
			if let Some(window) = app.get_webview_window(MAIN) {
				set_minimum(&window);
				track(app, &window);
			}
		}
		_ => {}
	}
}

/// Windows: the web view's own browser keys off (find with Ctrl-F and F3, print, reload, its zoom, the keyboard's
/// Back, Forward and Search keys; the editing keys stay), which `WebView2` leaves on and Tauri never turns off. Every
/// window of the page calls it once it is built, before it shows (the main window here, the card in `card.rs`).
/// Microsoft's sample says the change holds from the next navigation, so the page's own key guard (`lib/keys.js`)
/// blocks the same keys too.
#[cfg_attr(not(windows), expect(unused_variables, reason = "only WebView2 has browser keys of its own"))]
pub(crate) fn browser_keys_off(window: &WebviewWindow) {
	#[cfg(windows)]
	{
		use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Settings3;
		use windows::core::Interface as _;
		let done = window.with_webview(|webview| {
			// SAFETY: the web view's controller, its core and its settings, on the main thread that owns them;
			// ICoreWebView2Settings3 is WebView2 1.0.864 and later.
			let off = unsafe {
				webview.controller().CoreWebView2().and_then(|core| core.Settings()).and_then(|settings| {
					settings.cast::<ICoreWebView2Settings3>()?.SetAreBrowserAcceleratorKeysEnabled(false)
				})
			};
			if let Err(e) = off {
				log::warn!("window: the browser keys stay on: {e}");
			}
		});
		if let Err(e) = done {
			log::warn!("window: the browser keys stay on: {e}");
		}
	}
}

/// The player's full-screen choice (`view.fullscreen`, Big view's toggle): the main window's own full screen, and the
/// `shell` topic's `fullscreen`; the choice is kept in settings.json by the caller.
pub(crate) fn full_view(app: &AppHandle, on: bool) -> Result<(), String> {
	if let Some(window) = app.get_webview_window(MAIN) {
		window.set_fullscreen(on).map_err(|e| e.to_string())?;
	}
	ui::update(app, |v| v.shell.fullscreen = on);
	Ok(())
}

impl Helper {
	/// The `view` area: the zoom (`{"type":"view.zoom","step":1|-1|0}`), Big view's full screen
	/// (`{"type":"view.fullscreen","on":true}`) and whether the page's tabs can act (`{"type":"view.tabs","on":false}`
	/// under a pump's cover or a window; the macOS View menu greys its five tabs meanwhile, `menu::tabs_on`, and Windows
	/// has no app menu).
	pub(crate) fn view_intent(&mut self, intent: &Intent) -> Result<(), String> {
		match intent.kind() {
			"view.zoom" => {
				let step = intent.arg("step").and_then(serde_json::Value::as_i64).ok_or("view.zoom takes a step")?;
				self.zoom(i8::try_from(step.signum()).unwrap_or(0));
			}
			"view.fullscreen" => {
				let on = intent.arg("on").and_then(serde_json::Value::as_bool).ok_or("view.fullscreen takes on")?;
				full_view(&self.app, on)?;
				if self.desk.settings.fullscreen != on {
					self.desk.settings.fullscreen = on;
					self.desk.save_settings();
				}
			}
			"view.tabs" => {
				let on = intent.arg("on").and_then(serde_json::Value::as_bool).ok_or("view.tabs takes on")?;
				#[cfg(target_os = "macos")]
				{
					let handle = self.app.clone();
					self.app
						.run_on_main_thread(move || super::menu::tabs_on(&handle, on))
						.map_err(|e| e.to_string())?;
				}
				#[cfg(not(target_os = "macos"))]
				let _ = on;
			}
			_ => return Err(intent.unknown()),
		}
		Ok(())
	}

	/// One step of the page's size (1 larger, -1 smaller, 0 back to 100 %), applied to the open window at `BASE` times
	/// it and kept: the keys, the macOS View menu and the Menu's Size all come here.
	pub(crate) fn zoom(&mut self, step: i8) {
		let now = self.desk.settings.zoom;
		let at = ZOOMS.iter().position(|z| (z - now).abs() < 0.001).unwrap_or(2);
		let zoom = match step {
			0 => 1.0,
			1 => ZOOMS[(at + 1).min(ZOOMS.len() - 1)],
			_ => ZOOMS[at.saturating_sub(1)],
		};
		if let Some(window) = self.app.get_webview_window(MAIN)
			&& let Err(e) = window.set_zoom(BASE * zoom)
		{
			log::warn!("zoom: {e}");
		}
		if (self.desk.settings.zoom - zoom).abs() > f64::EPSILON {
			self.desk.settings.zoom = zoom;
			self.desk.save_settings();
		}
		ui::update(&self.app, |v| v.shell.zoom = zoom);
	}

	/// The main window's place into settings.json once it has stood still for `SETTLE` after a move or a resize (the
	/// loop's 250-ms tick asks), through the settings' one writer.
	pub(crate) fn place_settled(&mut self) {
		let place = {
			let desk = self.app.state::<Desk>();
			let settled = desk.moved.lock().ok().and_then(|mut m| m.take_if(|at| at.elapsed() >= SETTLE));
			if settled.is_none() {
				return;
			}
			desk.place()
		};
		self.keep_place(place);
	}

	/// The main window's place into settings.json (it settled, it closed, or QuestPump quits).
	pub(crate) fn keep_place(&mut self, place: Option<Place>) {
		if place.is_some() && self.desk.settings.window != place {
			self.desk.settings.window = place;
			self.desk.save_settings();
		}
	}
}
