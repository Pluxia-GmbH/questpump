//! QuestPump kept alive (contract/crash-api.yaml, "The desktop app", "Kept alive"): the main loop, the LAN server, the
//! socket and the mirror's writer run supervised. One that ends in a panic (its report written by the panic hook, sent
//! by the next start's crash sweep) restarts QuestPump once, in the background with no window (`--restarted`, or
//! `--restarted-quick` when it ended within `QUICK` of its start), unless this start was itself such a quick restart
//! and ended within `QUICK` too: then QuestPump stays up showing its Failed state (the tray red, This computer's line),
//! so a part that panics at every start never loops. The restart is Tauri's (`tauri::process::restart`: through
//! LaunchServices on macOS, so the grants hold), after the single-instance lock is let go, so the new process never
//! hands itself to this one. A part that ends without a panic (the loop's Quit, the LAN server's stop) is left as it is.

use std::ffi::OsString;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use tauri::{AppHandle, Manager as _};

use crate::desk::launch;
use crate::ui::{self, Screen};

/// How soon after its start a part's panic makes a quick end: two in a row and QuestPump stops restarting.
const QUICK: Duration = Duration::from_mins(1);

/// When this process started (`mark_start`, first in `lib.rs`'s `run()`).
static STARTED: LazyLock<Instant> = LazyLock::new(Instant::now);
/// A part's end is being handled: a second part that ends meanwhile (the loop's end closes the writer's channel) adds
/// nothing.
static ENDING: AtomicBool = AtomicBool::new(false);

/// A supervised part.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Part {
	Loop,
	Lan,
	Socket,
	Writer,
}

impl Part {
	/// Its name in the log and in the Failed state.
	fn name(self) -> &'static str {
		match self {
			Self::Loop => "the main loop", // copy-lint: not shown (a log line's and an error's words)
			Self::Lan => "the LAN server", // copy-lint: not shown (a log line's and an error's words)
			Self::Socket => "the socket",  // copy-lint: not shown (a log line's and an error's words)
			Self::Writer => "the mirror's writer", // copy-lint: not shown (a log line's and an error's words)
		}
	}
}

/// Marks this process's start (first thing in `run()`): a part's panic is quick when it comes within `QUICK` of it.
pub(crate) fn mark_start() {
	LazyLock::force(&STARTED);
}

/// Supervises a task on Tauri's async runtime (the loop): its panic restarts QuestPump once.
pub(crate) fn watch(app: &AppHandle, part: Part, task: tauri::async_runtime::JoinHandle<()>) {
	let app = app.clone();
	tauri::async_runtime::spawn(async move {
		if let Err(tauri::Error::JoinError(e)) = task.await
			&& e.is_panic()
		{
			ended(&app, part);
		}
	});
}

/// Supervises a Tokio task (the LAN server, the socket): its panic restarts QuestPump once.
pub(crate) fn watch_tokio(app: &AppHandle, part: Part, task: tokio::task::JoinHandle<()>) {
	let app = app.clone();
	tauri::async_runtime::spawn(async move {
		if let Err(e) = task.await
			&& e.is_panic()
		{
			ended(&app, part);
		}
	});
}

/// Supervises a thread (the mirror's writer): its panic restarts QuestPump once. The watching thread waits for it.
pub(crate) fn watch_thread(app: &AppHandle, part: Part, thread: std::thread::JoinHandle<()>) {
	let app = app.clone();
	let watching = std::thread::Builder::new().name("keep".into()).spawn(move || {
		if thread.join().is_err() {
			ended(&app, part);
		}
	});
	if let Err(e) = watching {
		log::warn!("keep: {} runs unsupervised ({e})", part.name());
	}
}

/// A part ended in a panic: QuestPump restarts in the background, or, after a quick restart that ended quickly again,
/// stays up showing its Failed state.
fn ended(app: &AppHandle, part: Part) {
	if ENDING.swap(true, Ordering::SeqCst) {
		log::error!("keep: {} ended in a panic too", part.name());
		return;
	}
	let quick = STARTED.elapsed() < QUICK;
	let after_quick = std::env::args_os().any(|a| a == launch::RESTARTED_QUICK);
	if quick && after_quick {
		log::error!("keep: {} ended in a panic, as the start before did: QuestPump stays up and stopped", part.name());
		// copy-lint: not shown (an error's detail, as a start's failure shows it)
		let error = format!("{} stopped after a crash", part.name());
		ui::update(app, |v| {
			v.shell.dot = ui::Dot::Stopped;
			v.computer.screen = Screen::Failed { error };
		});
		return;
	}
	let flag = if quick { launch::RESTARTED_QUICK } else { launch::RESTARTED };
	log::error!("keep: {} ended in a panic: QuestPump restarts ({flag})", part.name());
	let mut env = app.env();
	env.args_os.retain(|a| a != launch::RESTARTED && a != launch::RESTARTED_QUICK);
	env.args_os.push(OsString::from(flag));
	// on the main thread, as Tauri's own restart there: the tray's icon goes with AppKit's main thread alone (removed
	// from another thread, AppKit traps; the probe of 2026-10-12). The new process must not hand itself to this one: the
	// lock goes first, then the tray, then the process
	let handle = app.clone();
	let restarting = app.run_on_main_thread(move || {
		tauri_plugin_single_instance::destroy(&handle);
		handle.cleanup_before_exit();
		log::logger().flush();
		tauri::process::restart(&env);
	});
	if let Err(e) = restarting {
		log::error!("keep: the restart could not reach the main thread ({e}): QuestPump stays up and stopped");
	}
}
