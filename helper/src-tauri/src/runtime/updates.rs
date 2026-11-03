//! The helper's updates (rust-helper `tauri-shell.md`, "The updater"): `update.rs` checks, keeps the download in the
//! data folder and reports; the loop holds the update (`Pending`) and installs it only while nothing would lose to it
//! (no game session its strip, no player the window they use or the pump in it, no match's offer its card or its stand,
//! which live only in memory, no watched game the installer's window over it), or at once at This computer's Restart to
//! update while only the open window holds it, and winds the helper down first (the LAN API stopped, the engine's calls
//! answered, every pending write done). A failed install keeps the update and tries again `AGAIN_FIRST` later, then
//! every `AGAIN`. A quit or a restart asked on the socket, a reinstall's, waits the same way for the pumps and the
//! offers (`quit_held`).

use std::sync::atomic::Ordering;
use std::time::Duration;

use tauri::Manager as _;

use super::Helper;
use crate::desk::Desk;
use crate::desk::windows::MAIN;
use crate::feeds::Due;
use crate::lan;
use crate::ui::{self, Quit, Shared, Update};
use crate::update::{self, Found, Ready};

/// After a failed install, the first try again, and the next ones.
const AGAIN_FIRST: Duration = Duration::from_mins(10);
const AGAIN: Duration = Duration::from_hours(1);

/// A downloaded update the loop holds until it installs.
pub(crate) struct Pending {
	ready: Box<Ready>,
	/// After a failed install, when it tries again.
	again: Option<Due>,
}

impl Pending {
	/// Its install may go: none failed, or the try again is due.
	fn due(&self) -> bool {
		self.again.is_none_or(|again| again.passed())
	}

	/// An install failed: it tries again `AGAIN_FIRST` later, after one that failed before `AGAIN` later.
	fn failed(&mut self) {
		self.again = Some(Due::after(if self.again.is_some() { AGAIN } else { AGAIN_FIRST }));
	}
}

impl Helper {
	/// What the last check found. The same release found again refreshes the one held (its manifest, its tries kept);
	/// a release the feed no longer offers is dropped; a failed check leaves a pending update and its line as they are.
	pub(super) fn on_update(&mut self, found: Found) {
		let line = match found {
			Found::Ready(ready) => {
				match &mut self.update {
					Some(pending) if pending.ready.version() == ready.version() => pending.ready = ready,
					_ => {
						log::info!("update: {} is ready", ready.version());
						self.update = Some(Pending { ready, again: None });
					}
				}
				self.show_pending();
				return;
			}
			Found::Current => {
				self.update = None;
				Update::Current
			}
			Found::Unreplaceable(new) => {
				log::info!("update: {new} is out, and this account cannot replace the helper's app");
				self.update = None;
				Update::Unreplaceable { new }
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

	/// The version line of the update held: ready, with whether nothing but the open window holds it (This computer's
	/// Restart to update), or not installed while a failed install waits to try again.
	fn show_pending(&self) {
		let Some(pending) = &self.update else { return };
		let line = if pending.due() {
			Update::Ready { new: pending.ready.version().to_owned(), now: !self.install_held() }
		} else {
			Update::NotInstalled
		};
		ui::update(&self.app, |v| v.computer.version.update = Some(line));
	}

	/// Both release feeds checked again (the game closed, so what they bring can go in now), unless one was checked
	/// lately (`feeds::Checks::now`).
	pub(super) fn feeds_wake(&self) {
		self.update_checks.now();
		self.check_feed();
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

	/// What holds a downloaded update back even when the player asks for it (This computer's Restart to update): the
	/// game runs; the addon's own update is under way (it goes first, `runtime/addon.rs`); what holds a quit
	/// (`quit_held`); a quit asked on the socket waits (a reinstall's restart starts the bundle it swapped in, which an
	/// install would replace); a watched game runs or is in a match (Windows' installer would open its window over the
	/// game; a quit keeps the watcher's runs, `Watcher::keep`, but only for a start within `match_relaunch_s`, which an
	/// install may outlast).
	fn install_held(&self) -> bool {
		self.game.is_some()
			|| self.addon_busy()
			|| self.quit_held()
			|| self.socket_quit.is_some()
			|| self
				.games
				.as_ref()
				.is_some_and(|games| !games.running().is_empty() || games.watcher.playing(lan::now_ms()))
	}

	/// What holds a downloaded update back by itself: what holds it when asked (`install_held`), and the main window
	/// open (the new QuestPump starts with `--updated`, which opens none, so the window the player uses would go).
	fn update_held(&self) -> bool {
		self.app.get_webview_window(MAIN).is_some() || self.install_held()
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

	/// The 2-s look: the version line of the update held, which installs once nothing holds it back (`update_held`) and
	/// a try again after a failed install is due.
	pub(super) async fn update_when_closed(&mut self) {
		self.show_pending();
		if self.update_held() || !self.update.as_ref().is_some_and(Pending::due) {
			return;
		}
		self.install_update(false).await;
	}

	/// This computer's Restart to update: the update installs now while nothing but the open window holds it (the
	/// button shows only then; a click that comes once something else holds it does nothing), and the new QuestPump
	/// opens This computer (`update::restarted`).
	pub(super) async fn update_now(&mut self) {
		if self.install_held() {
			return;
		}
		self.install_update(true).await;
	}

	/// The update held installed: the helper winds down, then the new one replaces it and starts (Windows' installer
	/// ends this process itself), brought forward when the player `asked`. A macOS account that cannot replace the
	/// helper's app gets the download page instead of an administrator prompt nobody asked for. A failed install
	/// listens again, carries on and keeps the update to try again (`Pending::failed`), unless its file went (it no
	/// longer matched its signature: the next check downloads it again).
	async fn install_update(&mut self, asked: bool) {
		let Some(mut pending) = self.update.take() else { return };
		let new = pending.ready.version().to_owned();
		if !update::replaceable() {
			log::info!("update: {new} is out, and this account cannot replace the helper's app");
			ui::update(&self.app, |v| v.computer.version.update = Some(Update::Unreplaceable { new }));
			return;
		}
		log::info!("update: installing {new}{}", if asked { ", as the player asked" } else { "" });
		ui::update(&self.app, |v| v.computer.version.update = Some(Update::Installing { new: new.clone() }));
		self.wind_down().await;
		if asked {
			update::note_restart(&self.dir, &new);
		}
		if let Err(e) = update::install(&self.app, &pending.ready, asked) {
			log::warn!("update: {e:#}");
			if asked {
				update::drop_restart(&self.dir);
			}
			if pending.ready.kept() {
				pending.failed();
				self.update = Some(pending);
			}
			ui::update(&self.app, |v| v.computer.version.update = Some(Update::NotInstalled));
			self.carry_on().await;
		}
	}
}
