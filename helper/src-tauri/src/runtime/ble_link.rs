//! The Bluetooth link on the loop (docs/plan.md slice 43, with its fixes after the review of 2026-10-17; the bridge is
//! `blelink`): it runs while the Phones page's switch is on (`settings.json` `ble_link`, on by default) and a phone is
//! paired or the pairing code shows or can show (the Phones page or the pairing window open, the computer on a home
//! network: on a Windows PC without its firewall rule the code shows beside the firewall card once the link serves); on
//! macOS it is made only once the Bluetooth grant is answered, or where the code shows, which asks it in context
//! (making the manager raises the prompt). What it says is every response's helper block `ble` (`Helper::tell_devices`;
//! `ask` before macOS's question shows and while it shows), the Phones page's Bluetooth row and the counts' `link_ble`.
//! The platform tries a failed publish or advertising again itself; a link that gave up stays stopped, the row saying
//! it could not start or stopped, until the switch is turned off and on. A link let go answers its calls in flight
//! first (an unpair's: the phone hears it), and another starts only once it ended. A new pairing code lets every
//! central go and publishes the service again. A pump on the strip asks the centrals for low latency, idle for medium.

use questpump_core::link::Ble;
use questpump_core::seal;
use tokio::sync::mpsc;

use super::Helper;
use crate::blelink::{self, BleLink, Heard, HelloKey};
use crate::ui::{self, BleLine, BleRow};

/// Whether the Phones page or the pairing window is open, and whether the pairing code shows (there or in the pairing
/// window).
#[derive(Default, Clone, Copy)]
struct Shown {
	page: bool,
	code: bool,
}

/// The link's state on the loop.
pub(crate) struct BleLinkState {
	link: Option<BleLink>,
	/// A link let go that still answers its calls in flight: none starts until it ended (`Heard::Stopped`).
	stopping: Option<BleLink>,
	heard: mpsc::UnboundedSender<Heard>,
	/// How it stands as the bridge last said; none before it said anything, and after it stopped.
	said: Option<Ble>,
	/// It said `on` since it started: a failure then is a link that stopped, not one that could not start.
	served: bool,
	/// It gave up, until the switch is turned off and on: whether it had served before (the row's words).
	failed: Option<bool>,
	/// A pump runs on the strip: the centrals' latency is low.
	busy: bool,
	/// What the window shows of the pairing.
	shown: Shown,
	/// The platform may start the link without asking the player (`blelink::may_start_unasked`: macOS's grant
	/// answered), read again at each start or stop and on the link tick, never on every turn of the loop.
	unasked: bool,
	/// The Phones page's row as last shown, with whether the link served then (the pairing code follows it).
	row: Option<(BleRow, bool)>,
}

impl BleLinkState {
	pub(super) fn new(heard: mpsc::UnboundedSender<Heard>) -> Self {
		Self {
			link: None,
			stopping: None,
			heard,
			said: None,
			served: false,
			failed: None,
			busy: false,
			shown: Shown::default(),
			unasked: blelink::may_start_unasked(),
			row: None,
		}
	}
}

impl Helper {
	/// What every response's helper block says of the link (`Helper.ble`): stopped while its switch is off, it gave
	/// up or the secrets did not read at this start, `ask` while macOS's grant is not answered (before the link runs
	/// and asks, and while its question waits, when the bridge has said nothing), else what the bridge last said (none
	/// before).
	pub(super) fn ble_word(&self) -> Option<Ble> {
		let b = &self.ble_link;
		if !self.settings.ble_link || b.failed.is_some() || self.secrets_unread() {
			Some(Ble::Stopped)
		} else if !b.unasked && (b.link.is_none() || b.said.is_none()) {
			Some(Ble::Ask)
		} else {
			b.said
		}
	}

	/// Whether the link serves phones now (`on`): the pairing code may show beside the Windows firewall card.
	pub(super) fn ble_serves(&self) -> bool {
		self.ble_word() == Some(Ble::On)
	}

	/// What the hellos' proofs are keyed with: the current pairing code's hello key.
	fn ble_hello_key(&self) -> HelloKey {
		seal::hello_key(&self.settings.key)
	}

	/// Starts or stops the bridge as the switch, the pairing, the window and the grant now say, and shows the row;
	/// never in a run whose secrets did not read, whose hellos (v 3) would prove a pairing code the next start drops. A
	/// link let go first answers its calls in flight; while it does, none starts (its end says `Heard::Stopped`).
	pub(super) fn ble_link_sync(&mut self) {
		self.ble_link.unasked = blelink::may_start_unasked();
		if self.ble_link.stopping.as_ref().is_some_and(BleLink::ended) {
			self.ble_link.stopping = None;
		}
		let b = &self.ble_link;
		let pairable = b.shown.code || (b.shown.page && self.address.host.is_some());
		let wanted = self.settings.ble_link
			&& !self.secrets_unread()
			&& b.failed.is_none()
			&& (self.settings.pairing.phone.is_some() || pairable)
			&& (b.shown.page || b.unasked);
		match (wanted, b.link.is_some()) {
			(true, false) if b.stopping.is_none() => {
				log::info!("ble: the Bluetooth link starts");
				let link = BleLink::start(self.lan.clone(), self.queue.clone(), self.ble_hello_key(), b.heard.clone());
				link.busy(b.busy);
				self.ble_link.link = Some(link);
				self.ble_link.said = None;
				self.ble_link.served = false;
			}
			(false, true) => {
				log::info!("ble: the Bluetooth link stops");
				self.ble_let_go();
			}
			_ => {}
		}
		self.ble_show();
	}

	/// The running link is let go: it answers its calls in flight, then stops; what it said stands no more.
	fn ble_let_go(&mut self) {
		if let Some(link) = self.ble_link.link.take() {
			link.stop();
			self.ble_link.stopping = Some(link);
		}
		self.ble_link.said = None;
	}

	/// What the bridge said; nothing from a bridge let go but that it ended.
	pub(super) fn on_ble_heard(&mut self, heard: Heard) {
		match heard {
			Heard::Stopped => {
				self.ble_link.stopping = None;
				self.ble_link_sync();
				return;
			}
			_ if self.ble_link.link.is_none() => return,
			Heard::State(state) => {
				if self.ble_link.said != Some(state) {
					log::info!("ble: the Bluetooth link is {}", state.word());
				}
				self.ble_link.said = Some(state);
				self.ble_link.served |= state == Ble::On;
			}
			Heard::Failed(why) => {
				let lost = self.ble_link.served;
				// the system's own error stays here; the Phones page says only that it stopped or could not start
				log::warn!("ble: the Bluetooth link {}: {why}", if lost { "stopped" } else { "could not start" });
				self.ble_link.failed = Some(lost);
				self.ble_let_go();
			}
		}
		self.ble_show();
	}

	/// The Phones page's switch: on starts the link where it is wanted (and tries once more after it gave up), off
	/// stops it.
	pub(super) fn set_ble_link(&mut self, on: bool) {
		if self.settings.ble_link != on {
			self.settings.ble_link = on;
			self.save_settings();
		}
		self.ble_link.failed = None;
		self.ble_link_sync();
	}

	/// Whether the Phones page or the pairing window is open and the pairing code shows.
	pub(super) fn phones_shown(&mut self, page: bool, code: bool) {
		self.ble_link.shown = Shown { page, code };
		self.ble_link_sync();
	}

	/// A new pairing code: every central goes, the hellos show the new key, and the service is published again (the
	/// bridge opens every envelope under the new keys from now on).
	pub(super) fn ble_rekeyed(&self) {
		if let Some(link) = &self.ble_link.link {
			link.rekey(self.ble_hello_key());
		}
	}

	/// A pump runs on the strip, or none does: the centrals' latency.
	pub(super) fn ble_busy(&mut self) {
		let busy = self.session.offered().is_some();
		if busy != self.ble_link.busy {
			self.ble_link.busy = busy;
			if let Some(link) = &self.ble_link.link {
				link.busy(busy);
			}
		}
	}

	/// The link tick: a bridge that ended by itself (a panic) gave up, the row follows whether a phone links by it,
	/// and the counts take the day's word.
	pub(super) fn ble_tick(&mut self) {
		if self.ble_link.link.as_ref().is_some_and(BleLink::ended) {
			// copy-lint: not shown (an error's detail in helper.log, as the bridge's own)
			let error = "the bridge ended by itself".to_owned();
			self.on_ble_heard(Heard::Failed(error));
		}
		self.ble_busy();
		// the grant answered meanwhile (in System Settings, or at the strap's prompt), or a link let go ended: the link
		// may start
		self.ble_link_sync();
		self.counts_link_ble(self.ble_word());
	}

	/// The Phones page's Bluetooth row, published when it changes: its switch, and its line where it has one (none
	/// while the switch is off or the link serves with no phone linked by it, which the switch says alone); the pairing
	/// code follows whether the link serves (`show_pair`: on Windows beside the firewall card).
	fn ble_show(&mut self) {
		let line = if !self.settings.ble_link {
			None
		} else if let Some(lost) = self.ble_link.failed {
			Some(if lost { BleLine::Lost } else { BleLine::Failed })
		} else {
			self.ble_word().and_then(|word| self.ble_line(word))
		};
		let row = BleRow { on: self.settings.ble_link, line };
		let serves = self.ble_serves();
		let (shown, then) = self.ble_link.row.take().unzip();
		self.ble_link.row = Some((row.clone(), serves));
		if shown.as_ref() != Some(&row) {
			ui::update(&self.app, |v| v.phones.ble = row);
		}
		if then != Some(serves) {
			self.show_pair();
		}
	}

	/// The row's line by the link's word: a phone linked by it now, an ask or a problem; none while it serves with no
	/// phone linked or stands stopped.
	fn ble_line(&self, word: Ble) -> Option<BleLine> {
		Some(match word {
			Ble::On => {
				let phone = self.phone_name().filter(|_| self.queue.ble_linked(crate::lan::now_ms()))?;
				BleLine::Linked { phone }
			}
			Ble::Ask => BleLine::Ask,
			Ble::NoGrant => BleLine::Refused,
			Ble::Off => BleLine::Off,
			Ble::NoPeripheral => BleLine::NoPeripheral,
			Ble::Missing => BleLine::Missing,
			Ble::Stopped => return None,
		})
	}

	/// QuestPump winds down: the bridge stops, letting every central go.
	pub(super) fn ble_stop(&mut self) {
		self.ble_link.link = None;
		self.ble_link.stopping = None;
	}
}
