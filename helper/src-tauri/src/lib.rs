//! QuestPump on this computer, the helper (docs/plan.md slices 5, 7, 8, 16, 24, 25 and 35): the shell around
//! questpump-core with the platform backends (the capture on macOS and Windows), the heart-rate strap and the walking
//! pad over Bluetooth, the Bluetooth link that serves the phones the LAN's routes (`blelink`), the local socket, the
//! LAN API the phone and the watch call, the desktop app's window (`desk`), start at login, signed updates of itself
//! and of the addon in the game (`addon`), the feedback button's post, the crash reports (`crash`, `keep`) and the
//! anonymous counts (`runtime/counts.rs`). It lives in the menu bar or the tray; the main window opens at a player's
//! launch, never at login or after an update (`desk::launch`), and while it is open QuestPump is an ordinary app; only
//! Quit exits (and an update, which starts the new QuestPump, and a crash of a supervised part, which restarts it once:
//! `keep`).
mod addon;
mod autostart;
mod blelink;
mod bluetooth;
mod capture;
#[rustfmt::skip]
mod copy;
mod crash;
mod desk;
mod dnssd;
mod elevate;
mod feedback;
mod feeds;
mod game;
mod games;
mod install;
mod keep;
pub mod lan;
mod pairing;
mod presence;
mod runtime;
mod secrets;
mod settings;
mod socket;
mod startlog;
mod ui;
mod units;
mod update;

pub use runtime::local_offset_at;

use tauri::{Manager as _, RunEvent};
use tauri_plugin_log::{RotationStrategy, Target, TargetKind};

#[cfg_attr(not(target_os = "macos"), expect(unused_variables, reason = "only macOS reopens a running app"))]
fn on_event(app: &tauri::AppHandle, event: RunEvent) {
	match event {
		// no window is a normal state: stay alive; the tray's Quit exits with a code
		RunEvent::ExitRequested { code: None, api, .. } => api.prevent_exit(),
		// opening the running QuestPump again (Finder, Launchpad, the Dock, `open`) shows its window
		#[cfg(target_os = "macos")]
		RunEvent::Reopen { .. } => desk::launch::reopen(app),
		_ => {}
	}
}

/// The most files macOS lets one process keep open (`<sys/syslimits.h>`'s `OPEN_MAX`; setrlimit(2): `RLIMIT_NOFILE`'s
/// soft limit may not exceed it).
#[cfg(target_os = "macos")]
const OPEN_MAX: libc::rlim_t = 10240;

/// What became of the open files' limit at the start (`raise_open_files`), logged once the log runs.
#[cfg(target_os = "macos")]
enum OpenFiles {
	Raised { from: libc::rlim_t, to: libc::rlim_t },
	Enough(libc::rlim_t),
	Kept(std::io::Error),
}

/// macOS starts an app with 256 open files at most (`launchctl limit maxfiles`), which a burst can pass: the window's
/// pages asking for many move figures through the asset protocol at once beside the stores' writes (2026-10-08, "Too
/// many open files" on a `rows.jsonl` append and a `settings.json` save, 69 files open once the burst was over). The
/// soft limit goes up to what the system lets one process have; a refusal keeps the old one.
#[cfg(target_os = "macos")]
fn raise_open_files() -> OpenFiles {
	let mut limit = libc::rlimit { rlim_cur: 0, rlim_max: 0 };
	// SAFETY: getrlimit writes into `limit`, a valid rlimit on this stack, and reads nothing else
	if unsafe { libc::getrlimit(libc::RLIMIT_NOFILE, &raw mut limit) } != 0 {
		return OpenFiles::Kept(std::io::Error::last_os_error());
	}
	let want = limit.rlim_max.min(OPEN_MAX);
	if limit.rlim_cur >= want {
		return OpenFiles::Enough(limit.rlim_cur);
	}
	let raised = libc::rlimit { rlim_cur: want, rlim_max: limit.rlim_max };
	// SAFETY: setrlimit reads `raised`, a valid rlimit on this stack, and changes only this process's own limit
	if unsafe { libc::setrlimit(libc::RLIMIT_NOFILE, &raw const raised) } != 0 {
		return OpenFiles::Kept(std::io::Error::last_os_error());
	}
	OpenFiles::Raised { from: limit.rlim_cur, to: want }
}

/// The log plugin's logger as `helper.log` writes it: rotated at 5 MB, five kept, at Info. Its logger is attached
/// behind the start's log in the setup (`startlog::attach`), so the lines before it (the crash sweep's) reach the file.
fn log_plugin() -> tauri_plugin_log::Builder {
	tauri_plugin_log::Builder::new()
		.clear_targets()
		.target(Target::new(TargetKind::LogDir { file_name: Some("helper".into()) }))
		.level(log::LevelFilter::Info)
		.rotation_strategy(RotationStrategy::KeepSome(5))
		.max_file_size(5_000_000)
}

/// macOS's `WebContent` process of a window ended (`WebKit`'s crash or a kill under memory pressure, which leaves the page
/// blank): said in the log and the window loaded again (contract/crash-api.yaml, "Kept alive"; never sent).
#[cfg(target_os = "macos")]
fn web_content_ended(webview: &tauri::Webview) {
	log::warn!("window: {}'s WebContent process ended; the page loads again", webview.label());
	if let Err(e) = webview.reload() {
		log::warn!("window: {} did not load again: {e}", webview.label());
	}
}

pub fn run() {
	// first: a panic from here on leaves its crash report (`crash`), and the start's log keeps what it says until the
	// log plugin is up (`startlog`)
	crash::catch_panics();
	keep::mark_start();
	startlog::install();
	// started again after a system quit while the grant was missing: worth it only when System Settings' "Quit &
	// Reopen" followed the player's grant (`update::reopen_after_quit`)
	if std::env::args_os().any(|a| a == desk::launch::REOPENED) && !capture::screen_granted() {
		return;
	}
	// before anything else it sets up: the crash reports that wait, on their way while the switch reads on
	let swept = runtime::sweep();
	#[cfg(target_os = "macos")]
	let open_files = raise_open_files();
	let builder = tauri::Builder::default();
	// started at login or beside the game (`open -g`): never take the focus from the game at launch; opening the window
	// (`desk::windows::open_main`, `forward`) still brings QuestPump forward. The app menu is QuestPump's own: Tauri's
	// default one quits without the loop (`desk::menu`)
	#[cfg(target_os = "macos")]
	let builder = builder
		.activate_ignoring_other_apps(false)
		.menu(desk::menu::build)
		.on_menu_event(|app, event| desk::menu::on_menu(app, &event))
		.on_web_content_process_terminate(web_content_ended);
	let context = tauri::generate_context!();
	let identifier = context.config().identifier.clone();
	let app = builder
		.plugin(tauri_plugin_single_instance::init(|app, args, _cwd| desk::launch::second(app, &args))) // the first plugin
		.plugin(tauri_plugin_dialog::init())
		.plugin(tauri_plugin_updater::Builder::new().build())
		.invoke_handler(tauri::generate_handler![
			desk::commands::desk_subscribe,
			desk::commands::desk_intent,
			desk::commands::save_image,
			desk::commands::reel_make,
			desk::commands::reel_cancel,
			desk::commands::save_video,
			desk::commands::copy_video,
			desk::commands::allow_screen,
			desk::commands::restart,
			desk::commands::new_pairing,
			desk::commands::set_strap,
			desk::commands::set_pad,
			desk::commands::set_ble_link,
			desk::commands::phones_shown,
			desk::commands::secret_shown,
			desk::commands::pick_folder,
			desk::commands::allow_network,
			desk::commands::allow_folder,
			desk::commands::addon_install,
			desk::commands::install_update,
			desk::commands::set_autostart,
			desk::commands::open_applications,
			desk::commands::diagnostics,
			desk::commands::send_feedback,
			desk::commands::page_error
		])
		.setup(move |app| {
			// the log plugin, whose logger takes what the start's log kept first
			let (plugin, level, logger) = log_plugin().split(app.handle())?;
			app.handle().plugin(plugin)?;
			startlog::attach(logger, level);
			#[cfg(target_os = "macos")]
			match open_files {
				OpenFiles::Raised { from, to } => log::info!("open files: {to} at most (from {from})"),
				OpenFiles::Enough(at) => log::info!("open files: {at} at most"),
				OpenFiles::Kept(e) => log::warn!("open files: the limit stays ({e})"),
			}
			// in the menu bar until the main window opens (`desk::windows`)
			#[cfg(target_os = "macos")]
			app.set_activation_policy(tauri::ActivationPolicy::Accessory);
			let launch = desk::launch::at_start();
			app.manage(ui::Shared::default());
			app.manage(desk::Desk::default());
			// the quit Apple event (the Dock's, a logout's) goes through the loop too (`desk::menu`)
			#[cfg(target_os = "macos")]
			desk::menu::catch_quit(app.handle());
			desk::tray::build(app)?;
			runtime::spawn(app.handle().clone(), launch, swept);
			Ok(())
		})
		.build(context);
	match app {
		Ok(app) => app.run(on_event),
		// before the window: the error into helper.log, with what the start's log kept
		Err(e) => startlog::fail(&identifier, &e),
	}
}
