//! The helper's updates (rust-helper `tauri-shell.md`, "The updater"): `update.rs` checks and downloads; the loop
//! installs a downloaded update only while nothing would lose to it (no game session its strip, no player the window
//! they use or the pump in it, no match's offer its card or its stand, which live only in memory, no watched game the
//! installer's window over it), and winds the helper down first (the LAN API stopped, the engine's calls answered,
//! every pending write done). A quit or a restart asked on the socket, a reinstall's, waits the same way for the pumps
//! and the offers (`quit_held`).

use std::sync::atomic::Ordering;

use tauri::Manager as _;

use super::Helper;
use crate::desk::Desk;
use crate::desk::windows::MAIN;
use crate::lan;
use crate::ui::{self, Quit, Shared, Update};
use crate::update::{self, Found};

impl Helper {
	/// What the last check found. A release the feed no longer offers is dropped; a failed check leaves a pending
	/// update and its line as they are.
	pub(super) fn on_update(&mut self, found: Found) {
		let line = match found {
			Found::Ready(ready) => {
				let line = Update::Ready { new: ready.version().to_owned() };
				self.update = Some(ready);
				line
			}
			Found::Current => {
				self.update = None;
				Update::Current
			}
			Found::Failed(e) => {
				log::warn!("update: {e}");
				if self.update.is_some() {
					return;
				}
				Update::Failed
			}
		};
		ui::update(&self.app, |v| v.computer.version.update = Some(line));
	}

	/// What holds a quit asked on the socket back, and an update's install with it: a pump is being made, runs on this
	/// computer or holds the window; the game's pump is on (`Session::pump_on`, also while the strip is out of sight: a
	/// new start would hear it mid-pump, without its moves); a match's offer stands within its time or its card shows
	/// (both live only in memory, so the new QuestPump would have neither on the tray, Pump home or the screen).
	pub(super) fn quit_held(&self) -> bool {
		self.desk_pump_starting()
			|| self.desk_pump_running()
			|| self.app.state::<Desk>().guard.load(Ordering::Relaxed)
			|| (self.game.is_some() && self.session.pump_on())
			|| self.standing.current().is_some()
			|| self.offering.card.shown().is_some()
	}

	/// What holds a downloaded update back: the game runs; the addon's own update is under way (it goes first,
	/// `runtime/addon.rs`); the main window is open (the new QuestPump starts with `--updated`, which opens none, so the
	/// window the player uses would go); what holds a quit (`quit_held`); a quit
	/// asked on the socket waits (a reinstall's restart starts the bundle it swapped in, which an install would
	/// replace); a watched game runs or is in a match (Windows' installer would open its window over the game; a quit
	/// keeps the watcher's runs, `Watcher::keep`, but only for a start within `match_relaunch_s`, which an install may
	/// outlast).
	fn update_held(&self) -> bool {
		self.game.is_some()
			|| self.addon_busy()
			|| self.app.get_webview_window(MAIN).is_some()
			|| self.quit_held()
			|| self.socket_quit.is_some()
			|| self
				.games
				.as_ref()
				.is_some_and(|games| !games.running().is_empty() || games.watcher.playing(lan::now_ms()))
	}

	/// A quit or a restart asked on the socket (a reinstall's, rust-helper step 3; never the player's, whose tray Quit
	/// asks "Leave the pump?"): at once when nothing holds it, else at the first 2-s look nothing does (`quit_held`), so
	/// a swap never ends a pump, the game's or a match's offer; a later ask replaces an earlier one.
	pub(super) fn socket_quit(&mut self, again: bool) {
		self.socket_quit = Some(again);
		if self.quit_held() {
			log::info!("quit: asked on the socket; it waits until no pump runs and no match's offer stands");
		}
		self.socket_quit_due();
	}

	/// The quit asked on the socket, handed to the loop's quit once nothing holds it (`Quit`'s `again` for a restart).
	pub(super) fn socket_quit_due(&mut self) {
		let Some(again) = self.socket_quit else { return };
		if self.quit_held() {
			return;
		}
		self.socket_quit = None;
		log::info!("quit: asked on the socket{}", if again { ", then a start again" } else { "" });
		if let Some(tx) = self.app.state::<Shared>().quit.lock().ok().and_then(|q| q.clone()) {
			let _ = tx.send(Quit { start: None, again });
		}
	}

	/// Installs a downloaded update at the first look nothing holds it back (`update_held`): the helper winds down,
	/// then the new one replaces it and starts (Windows' installer ends this process itself). A macOS account that
	/// cannot replace the helper's app gets the download page instead of an administrator prompt nobody asked for; a
	/// failed install listens again and carries on.
	pub(super) async fn update_when_closed(&mut self) {
		if self.update_held() {
			return;
		}
		let Some(ready) = self.update.take() else { return };
		let new = ready.version().to_owned();
		if !update::replaceable() {
			log::info!("update: {new} is out, and this account cannot replace the helper's app");
			ui::update(&self.app, |v| v.computer.version.update = Some(Update::Unreplaceable { new }));
			return;
		}
		log::info!("update: installing {new}");
		ui::update(&self.app, |v| v.computer.version.update = Some(Update::Installing { new }));
		self.wind_down().await;
		if let Err(e) = update::install(&self.app, &ready) {
			log::warn!("update: {e:#}");
			ui::update(&self.app, |v| v.computer.version.update = Some(Update::NotInstalled));
			self.carry_on().await;
		}
	}
}
