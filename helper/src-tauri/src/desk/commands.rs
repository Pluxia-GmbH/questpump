//! The page's commands, app commands all (registered with `generate_handler!`, called through
//! `window.__TAURI__.core.invoke`; Tauri checks the ACL only for plugin commands, so the local page needs no capability
//! file): `desk_subscribe` (the topics' channel and their values now), `desk_intent` (an intent to the loop, answered
//! once its area took it), `save_image` (a PNG the page made, saved where the player picks), the day reel's
//! `reel_make` (the Share window's order, its id once its job started), `reel_cancel`, `save_video` (the made reel copied
//! where the player picks) and `copy_video` (its file on the clipboard), and the Menu pages' own,
//! each an ask to the loop: the grant's button and the restart that applies it, Open Applications, a new pairing code,
//! the Bluetooth link's switch and whether the Phones page and the pairing code show, the strap, the walking pad, the
//! game folder's picker, the two elevated steps on Windows, start with the computer, Copy a report for help (the report
//! for the clipboard) and Send feedback (how the message went); `secret_shown`, whether the page in front draws a code
//! that opens the player's account; and `page_error`, a page's uncaught error or unhandled rejection into `helper.log`
//! (never sent; contract/crash-api.yaml, "Kept alive").

use std::sync::Mutex;
use std::time::{Duration, Instant};

use tauri::ipc::{Channel, InvokeBody, Request};
use tauri::{AppHandle, Manager as _, State, Webview};
use tokio::sync::oneshot;

use super::state::{Boot, Msg};
use super::windows::MAIN;
use super::{Desk, DeskAsk, Intent, reel};
use crate::copy;
use crate::ui::{self, FeedbackState, Shared, WindowAsk};

/// The most page errors `helper.log` takes in `PAGE_ERRORS_EVERY`: a page that fails in a loop never floods it.
const PAGE_ERRORS_MAX: u32 = 20;
const PAGE_ERRORS_EVERY: Duration = Duration::from_mins(1);
/// The longest message and place of a page error the log keeps, in characters.
const PAGE_ERROR_CHARS: usize = 300;

/// The page errors logged in the current minute: when it began and how many.
static PAGE_ERRORS: Mutex<Option<(Instant, u32)>> = Mutex::new(None);

/// The end of the log a report for help holds: the newest 200 KB.
const LOG_TAIL: usize = 200_000;
/// A saved image's name when the page names none.
const IMAGE: &str = "QuestPump.png";
/// The header that names a saved image.
const IMAGE_NAME: &str = "x-qp-name";

/// The page's subscription: its channel for every change from now on, and every topic's value now.
#[tauri::command]
#[expect(clippy::needless_pass_by_value, reason = "Tauri hands a command its arguments by value")]
pub fn desk_subscribe(on: Channel<Msg>, webview: Webview, desk: State<'_, Desk>) -> Boot {
	desk.publisher.subscribe(webview.label(), on)
}

/// An intent (`{"type": "<area>.<what>", ...}`), answered once its area's handler took it.
///
/// # Errors
/// The intent does not read, no area handles it, its handler refused it, or the loop does not run.
#[tauri::command]
pub async fn desk_intent(intent: serde_json::Value, state: State<'_, Shared>) -> Result<(), String> {
	let intent = Intent::from_value(intent)?;
	let (done, outcome) = oneshot::channel();
	if !state.ask(WindowAsk::Desk(DeskAsk::Intent { intent, done })) {
		return Err("the loop does not run".to_owned());
	}
	outcome.await.map_err(|_| "the loop stopped".to_owned())?
}

/// A PNG the page made (a share card), its bytes the request's raw body: the system's save dialog on the main window,
/// then the file. True once saved, false when the player cancelled.
///
/// # Errors
/// The body is no PNG, or the file could not be written.
#[tauri::command]
pub async fn save_image(app: AppHandle, request: Request<'_>) -> Result<bool, String> {
	use tauri_plugin_dialog::DialogExt as _;
	let InvokeBody::Raw(bytes) = request.body() else { return Err("a PNG's bytes as the body".to_owned()) };
	if !bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
		return Err("not a PNG".to_owned());
	}
	let bytes = bytes.clone();
	let name = request
		.headers()
		.get(IMAGE_NAME)
		.and_then(|v| v.to_str().ok())
		.filter(|n| {
			std::path::Path::new(n).extension().is_some_and(|e| e.eq_ignore_ascii_case("png"))
				&& !n.contains(['/', '\\'])
		})
		.unwrap_or(IMAGE)
		.to_owned();
	let (picked, chosen) = oneshot::channel();
	let mut dialog =
		app.dialog().file().set_title(copy::desk_window_save_title()).add_filter("PNG", &["png"]).set_file_name(name);
	if let Some(window) = app.get_webview_window(MAIN) {
		dialog = dialog.set_parent(&window);
	}
	dialog.save_file(move |path| {
		let _ = picked.send(path.and_then(|p| p.into_path().ok()));
	});
	let Some(path) = chosen.await.map_err(|_| "the dialog closed".to_owned())? else { return Ok(false) };
	std::fs::write(&path, bytes).map_err(|e| format!("{}: {e}", path.display()))?;
	Ok(true)
}

/// The Share window's order for a day reel (`lib/reel.js`): its body the request's raw bytes (the head's JSON and the
/// layers' PNGs, `desk::reel`), handed to the loop, which checks it and starts its job. The reel's id once it started;
/// its progress comes on the `reel` topic.
///
/// # Errors
/// The body is no bytes, the loop does not run, or it refused the order (a pump runs, felt unwell rests the pumps, the
/// plan or a layer does not hold).
#[tauri::command]
pub async fn reel_make(request: Request<'_>, state: State<'_, Shared>) -> Result<u64, String> {
	let InvokeBody::Raw(bytes) = request.body() else { return Err("a reel's order as the body".to_owned()) };
	let (done, outcome) = oneshot::channel();
	if !state.ask(WindowAsk::Desk(DeskAsk::Reel { body: bytes.clone(), done })) {
		return Err("the loop does not run".to_owned());
	}
	outcome.await.map_err(|_| "the loop stopped".to_owned())?
}

/// The Share window closed or waits no more for the reel `id` (none: whichever): the reel being made, where it is that
/// one, stops and leaves no file.
#[tauri::command]
#[expect(clippy::needless_pass_by_value, reason = "Tauri hands a command its app handle by value")]
pub fn reel_cancel(app: AppHandle, id: Option<u64>) {
	reel::cancel_reel(&app, id);
}

/// Save video: the made reel `id` copied where the player picks in the system's save dialog on the main window, under
/// its own name. True once saved, false when the player cancelled.
///
/// # Errors
/// The reel is not made (or another took its place), or the file could not be written.
#[tauri::command]
pub async fn save_video(app: AppHandle, id: u64) -> Result<bool, String> {
	use tauri_plugin_dialog::DialogExt as _;
	let file = reel::made(&app, id).ok_or("that reel is not made")?;
	let name = file.file_name().and_then(|n| n.to_str()).unwrap_or_default().to_owned();
	let (picked, chosen) = oneshot::channel();
	let mut dialog = app
		.dialog()
		.file()
		.set_title(copy::desk_window_save_video_title())
		.add_filter("MP4", &["mp4"])
		.set_file_name(name);
	if let Some(window) = app.get_webview_window(MAIN) {
		dialog = dialog.set_parent(&window);
	}
	dialog.save_file(move |path| {
		let _ = picked.send(path.and_then(|p| p.into_path().ok()));
	});
	let Some(path) = chosen.await.map_err(|_| "the dialog closed".to_owned())? else { return Ok(false) };
	std::fs::copy(&file, &path).map_err(|e| format!("{}: {e}", path.display()))?;
	Ok(true)
}

/// Copy video: the made reel `id` on the clipboard as a file, to paste into a chat.
///
/// # Errors
/// The reel is not made, or the system refused it.
#[tauri::command]
#[expect(clippy::needless_pass_by_value, reason = "Tauri hands a command its app handle by value")]
pub fn copy_video(app: AppHandle, id: u64) -> Result<(), String> {
	reel::copy(&app, id).map_err(|e| format!("{e:#}"))
}

/// This computer's restart that applies a Screen Recording grant.
#[tauri::command]
#[expect(clippy::needless_pass_by_value, reason = "Tauri hands a command its state by value")]
pub fn restart(state: State<'_, Shared>) {
	state.ask(WindowAsk::Restart);
}

/// Phones' New pairing code, pressed twice.
#[tauri::command]
#[expect(clippy::needless_pass_by_value, reason = "Tauri hands a command its state by value")]
pub fn new_pairing(state: State<'_, Shared>) {
	state.ask(WindowAsk::NewPairing);
}

/// Devices' strap switch: on reads it on the Bluetooth task (macOS asks for Bluetooth the first time), off lets it go.
#[tauri::command]
#[expect(clippy::needless_pass_by_value, reason = "Tauri hands a command its state by value")]
pub fn set_strap(on: bool, state: State<'_, Shared>) {
	state.ask(WindowAsk::Strap(on));
}

/// Devices' walking pad switch: on reads it on the Bluetooth task (macOS asks for Bluetooth the first time), off lets it
/// go.
#[tauri::command]
#[expect(clippy::needless_pass_by_value, reason = "Tauri hands a command its state by value")]
pub fn set_pad(on: bool, state: State<'_, Shared>) {
	state.ask(WindowAsk::Pad(on));
}

/// Phones' Bluetooth link switch (slice 43): on serves the phones over Bluetooth too (macOS asks for Bluetooth on this
/// page the first time), off stops it.
#[tauri::command]
#[expect(clippy::needless_pass_by_value, reason = "Tauri hands a command its state by value")]
pub fn set_ble_link(on: bool, state: State<'_, Shared>) {
	state.ask(WindowAsk::BleLink(on));
}

/// Whether the page in front draws a code that opens the player's account (the restore code, the one typed to
/// restore, Link a device's code, Email's request code), each held behind Show code until the player presses it
/// (views/page.js `heldCode`; lib/router.js `reveal` awaits this answer before the code draws, and asks it off only
/// once the frame on screen no longer holds the code): its window is kept out of screen captures, recordings and
/// shares while one draws, and seen again once none does. Windows leaves it out of every capture
/// (`WDA_EXCLUDEFROMCAPTURE`; before Windows 10 2004 it shows black); macOS's `NSWindowSharingNone` keeps it from the
/// window list and the screenshots that read it, while an app that captures through ScreenCaptureKit still sees it
/// (Electron's `setContentProtection` says the same), so on a Mac the press is what keeps the code out of a stream. The
/// pairing code is no such code: it opens this computer's own network and Bluetooth link alone, and Pump home shows it
/// to every player without a phone.
///
/// # Errors
/// The window refused the change.
#[tauri::command]
#[expect(clippy::needless_pass_by_value, reason = "Tauri hands a command its webview by value")]
pub fn secret_shown(on: bool, webview: Webview) -> Result<(), String> {
	webview.window().set_content_protected(on).map_err(|e| {
		log::warn!("window: {} stays as it was for screen captures: {e}", webview.label());
		e.to_string()
	})?;
	if on {
		log::info!("window: {} kept out of screen captures", webview.label());
	} else {
		log::info!("window: {} seen by screen captures again", webview.label());
	}
	Ok(())
}

/// Whether the Phones page is open and the pairing code shows (there or in the pairing window): the Bluetooth link
/// advertises while the code shows, and asks for macOS's Bluetooth grant on the Phones page alone (slice 43).
#[tauri::command]
#[expect(clippy::needless_pass_by_value, reason = "Tauri hands a command its state by value")]
pub fn phones_shown(page: bool, code: bool, state: State<'_, Shared>) {
	state.ask(WindowAsk::PhonesShown { page, code });
}

/// This computer's Change… beside the game folder: the system's folder picker on the main window (macOS would
/// otherwise hang its sheet on a hidden window), which returns at once and hands the choice to the loop when the player
/// makes it.
#[tauri::command]
#[expect(clippy::needless_pass_by_value, reason = "Tauri hands a command its app handle by value")]
pub fn pick_folder(app: AppHandle) {
	use tauri_plugin_dialog::DialogExt as _;
	let handle = app.clone();
	let mut dialog = app.dialog().file().set_title(copy::desk_menu_computer_folder_pick());
	if let Some(window) = app.get_webview_window(MAIN) {
		dialog = dialog.set_parent(&window);
	}
	dialog.pick_folder(move |picked| {
		if let Some(path) = picked.and_then(|p| p.into_path().ok()) {
			handle.state::<Shared>().ask(WindowAsk::Folder(path));
		}
	});
}

/// Phones' Allow on my network (Windows): Windows asks for an administrator once.
#[tauri::command]
#[expect(clippy::needless_pass_by_value, reason = "Tauri hands a command its state by value")]
pub fn allow_network(state: State<'_, Shared>) {
	state.ask(WindowAsk::AllowNetwork);
}

/// This computer's Allow writing to the game folder (Windows): Windows asks for an administrator once.
#[tauri::command]
#[expect(clippy::needless_pass_by_value, reason = "Tauri hands a command its state by value")]
pub fn allow_folder(state: State<'_, Shared>) {
	state.ask(WindowAsk::AllowFolder);
}

/// Add QuestPump to the game (the welcome's game row, Pump home's card, This computer) and This computer's Update now.
#[tauri::command]
#[expect(clippy::needless_pass_by_value, reason = "Tauri hands a command its state by value")]
pub fn addon_install(state: State<'_, Shared>) {
	state.ask(WindowAsk::Addon);
}

/// This computer's Open Applications (macOS): the Finder shows the folder to drag QuestPump into.
#[tauri::command]
pub fn open_applications() {
	if cfg!(target_os = "macos")
		&& let Err(e) = super::menu_pages::about::open_url("/Applications")
	{
		log::warn!("open Applications: {e}");
	}
}

/// Send feedback's Send: hands the message to the loop and answers once the route answered it, the outbox kept it for
/// later, the check refused it, or a message the outbox could not keep failed its one try.
///
/// # Errors
/// The loop does not run.
#[tauri::command]
pub async fn send_feedback(text: String, diagnostics: bool, state: State<'_, Shared>) -> Result<FeedbackState, String> {
	let (done, outcome) = oneshot::channel();
	if !state.ask(WindowAsk::Feedback { text, diagnostics, done }) {
		return Err("the loop does not run".to_owned());
	}
	outcome.await.map_err(|_| "the loop stopped".to_owned())?
}

/// A page's uncaught error or unhandled rejection (`lib/errors.js`, in the main window and the card): one `helper.log`
/// line naming the window, the kind, the message and where (the page's own file and line), cut to
/// `PAGE_ERROR_CHARS` with its control characters made spaces; at most `PAGE_ERRORS_MAX` a minute, the minute's first
/// one past them saying so. Never sent anywhere.
#[tauri::command]
#[expect(clippy::needless_pass_by_value, reason = "Tauri hands a command its arguments by value")]
pub fn page_error(kind: String, message: String, place: String, webview: Webview) {
	let now = Instant::now();
	let count = {
		let Ok(mut errors) = PAGE_ERRORS.lock() else { return };
		let (since, count) = errors.get_or_insert((now, 0));
		if now.duration_since(*since) >= PAGE_ERRORS_EVERY {
			(*since, *count) = (now, 0);
		}
		*count += 1;
		*count
	};
	let clean = |text: &str| -> String {
		text.chars().take(PAGE_ERROR_CHARS).map(|c| if c.is_control() { ' ' } else { c }).collect()
	};
	let kind = if kind == "rejection" { "an unhandled rejection" } else { "an uncaught error" }; // copy-lint: not shown
	match count {
		..=PAGE_ERRORS_MAX => {
			log::warn!("page: {} {kind}: {} at {}", webview.label(), clean(&message), clean(&place));
		}
		n if n == PAGE_ERRORS_MAX + 1 => {
			log::warn!("page: more than {PAGE_ERRORS_MAX} errors this minute; the rest go unlogged until it ends");
		}
		_ => {}
	}
}

/// This computer's Start with this computer.
#[tauri::command]
#[expect(clippy::needless_pass_by_value, reason = "Tauri hands a command its state by value")]
pub fn set_autostart(on: bool, state: State<'_, Shared>) {
	state.ask(WindowAsk::Autostart(on));
}

/// Copy a report for help: what QuestPump can say about itself, for a player to paste into a message, since none of it
/// is sent home: its version and system, the engine it runs (its sources' digest) and its last fold, the Menu's topics
/// as they stand and the end of its log (`helper.log`), in English for whoever helps. Left out: the pairing code's
/// modules (they carry the pairing key) and QuestPump's own program and resources paths (on Windows they hold the
/// account's name).
#[tauri::command]
#[expect(clippy::needless_pass_by_value, reason = "Tauri hands a command its app handle by value")]
pub fn diagnostics(app: AppHandle) -> String {
	let mut view = ui::view(&app);
	view.phones.qr = None;
	view.computer.program.clear();
	view.shell.resources = None;
	let log = app
		.path()
		.app_log_dir()
		.ok()
		.and_then(|dir| std::fs::read(dir.join("helper.log")).ok())
		.map_or_else(String::new, |bytes| {
			String::from_utf8_lossy(&bytes[bytes.len().saturating_sub(LOG_TAIL)..]).into_owned()
		});
	let fold = view.shell.fold.map_or_else(
		|| ", no fold yet".to_owned(), // copy-lint: not shown (a support report)
		// copy-lint: not shown (a support report)
		|f| format!(", the last fold of {} rows in {} ms at {} (unix ms)", f.rows, f.ms, f.at),
	);
	format!(
		// copy-lint: not shown (a support report)
		"QuestPump {} on {} ({})\nengine {}{fold}\n{}: {}\n{}: {}\n{}: {}\n{}: {}\n\n{log}",
		env!("CARGO_PKG_VERSION"),
		std::env::consts::OS,
		std::env::consts::ARCH,
		questpump_engine::DIGEST,
		super::SHELL,
		json(&view.shell),
		super::COMPUTER,
		json(&view.computer),
		super::PHONES,
		json(&view.phones),
		super::DEVICES,
		json(&view.devices),
	)
}

/// A topic's JSON for the report.
fn json(value: &impl serde::Serialize) -> String {
	serde_json::to_string(value).unwrap_or_default()
}

/// This computer's button that asks for the Screen Recording grant; from then on the card offers the restart that
/// applies it.
#[tauri::command]
#[expect(clippy::needless_pass_by_value, reason = "Tauri hands a command its app handle by value")]
pub fn allow_screen(app: AppHandle) {
	match crate::runtime::data_dir() {
		Ok(dir) => {
			crate::capture::request_permission(&dir);
			ui::update(&app, |v| v.computer.asked = true);
		}
		Err(e) => log::warn!("allow_screen: {e}"),
	}
}
