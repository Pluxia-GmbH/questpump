//! The LAN API and the Windows Defender Firewall (rust-helper `windows-capture-and-install.md`, "The firewall rule"):
//! Windows asks on a program's first listen, and a standard user's answer writes block rules whatever they click, so
//! on Windows the helper listens only once a rule lets the phone in. Until then the Phones page offers the one-time
//! elevated step (`elevate::allow_network`) and shows no pairing code, or, while "Block all incoming connections" is on
//! (no rule opens that), says where to turn it off; the rules are read again each minute, so a rule made by hand or a
//! firewall turned off counts too. Elsewhere the helper listens at once.

use super::{Elevated, Helper};
use crate::elevate::{self, Gate, Outcome};
use crate::ui::{self, AdminNote, FirewallCard};

/// Whether the LAN API may listen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Firewall {
	Open,
	/// No rule lets the phone in yet: the window offers the step.
	Closed,
	/// The elevated step is on the screen.
	Asking,
	/// "Block all incoming connections" is on: the window says where the player turns it off.
	BlocksAll,
}

impl Firewall {
	pub(super) fn at_start() -> Self {
		if cfg!(windows) { Self::Closed } else { Self::Open }
	}

	/// The Phones page's card.
	fn card(self) -> FirewallCard {
		match self {
			Self::Open => FirewallCard::Open,
			Self::Closed | Self::Asking => FirewallCard::Rule,
			Self::BlocksAll => FirewallCard::Blocked,
		}
	}
}

/// The marker the uninstaller reads (`windows/hooks.nsh`): the helper made the rule, so uninstalling removes it.
pub(super) const MARKER: &str = "firewall-rule";

impl Helper {
	/// Whether the LAN API may listen now: the rules read again while none lets the phone in. Rules that cannot be read
	/// (the firewall service stopped) do not keep the phone out: the helper listens, as it would without a firewall.
	pub(super) async fn may_listen(&mut self) -> bool {
		match self.firewall {
			Firewall::Open => return true,
			Firewall::Asking => return false,
			Firewall::Closed | Firewall::BlocksAll => {}
		}
		let gate = match tokio::task::spawn_blocking(elevate::firewall).await {
			Ok(Ok(gate)) => gate,
			Ok(Err(e)) => {
				log::warn!("firewall: the rules could not be read ({e:#}); listening");
				Gate::Open
			}
			Err(e) => {
				log::warn!("firewall: the rules could not be read ({e}); listening");
				Gate::Open
			}
		};
		self.firewall = match gate {
			Gate::Open => Firewall::Open,
			Gate::Rule => Firewall::Closed,
			Gate::BlocksAll => Firewall::BlocksAll,
		};
		let card = self.firewall.card();
		ui::update(&self.app, |v| v.phones.firewall = card);
		self.firewall == Firewall::Open
	}

	/// The Phones page's "Allow on my network": the elevated step runs off the loop (the UAC prompt waits for the
	/// player).
	pub(super) fn allow_network(&mut self) {
		if self.firewall != Firewall::Closed {
			return;
		}
		let program = match std::env::current_exe() {
			Ok(p) => p,
			Err(e) => {
				log::warn!("firewall: the helper's own path: {e}");
				return;
			}
		};
		self.firewall = Firewall::Asking;
		ui::update(&self.app, |v| v.phones.network_note = Some(AdminNote::Waiting));
		let tx = self.elevated.clone();
		tokio::spawn(async move {
			let outcome = tokio::task::spawn_blocking(move || elevate::allow_network(&program))
				.await
				.unwrap_or_else(|e| Outcome::Failed(e.to_string()));
			let _ = tx.send(Elevated::Network(outcome));
		});
	}

	/// The firewall step came back: once it is done the rules are read again and the helper listens when they let the
	/// phone in; a declined or failed step, and a rule that another still overrides, say so on the card, which offers
	/// the step again.
	pub(super) async fn network_allowed(&mut self, outcome: Outcome) {
		self.firewall = Firewall::Closed;
		let note = match outcome {
			Outcome::Done => {
				self.save_file(MARKER, Vec::new());
				self.serve_lan().await;
				if self.firewall == Firewall::Closed {
					log::warn!("firewall: the rule was made, and another still keeps the phone out");
					Some(AdminNote::Another)
				} else {
					log::info!("firewall: the rule is made");
					None
				}
			}
			Outcome::Declined => {
				log::info!("firewall: the administrator step was declined");
				Some(AdminNote::Declined)
			}
			Outcome::Failed(e) => {
				log::warn!("firewall: the rule could not be made: {e}");
				Some(AdminNote::Failed { error: e })
			}
		};
		ui::update(&self.app, |v| v.phones.network_note = note);
	}
}
