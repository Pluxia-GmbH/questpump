//! The seams of the desktop app's own areas (docs/plan.md slice 35; the desk's are `questpump_desk::seams`): what the
//! loop's areas ask of the games on the computer and the card, each one `impl Helper` method its owner gives its body
//! and every caller calls, the desk's asks among them (`desk::shell::Out`).

use questpump_desk::platform::FoundGame;

use crate::runtime::Helper;

impl Helper {
	/// The game QuestPump knows on this computer now (`Shell::found_game`). Callers: what every response tells the
	/// phones and the watches (`Helper::tell_devices`), the addon's feed (`runtime/addon.rs`).
	pub(crate) fn found_game(&self) -> FoundGame {
		self.desk.p.found_game()
	}

	/// A device or this computer took the match offer `offer` (`m-<uuid>`: a phone's or a watch's sync carries it as
	/// `offer_taken` until a response comes, so it may come again; a desktop pump started from the offer): its
	/// `match_end` withdrawn, the standing offer taken, the card closed, its game's pump waiting for a move on Pump
	/// home's recent games (`recent_taken`), and the id answered in `offer_withdrawn` until its `match_end` would have
	/// expired. The test's id (`m-test`) only closes the test's card. Owner: DESK-GAMES. Callers: the sync handler
	/// (`runtime.rs`, `Ask::Synced`), DESK-PUMP (a pump started from an offer, `Platform::offer_taken`).
	pub(crate) fn offer_taken(&mut self, offer: &str) {
		if offer == questpump_core::offers::TEST {
			self.close_test();
			return;
		}
		if self.desk.p.games.is_none() {
			return;
		}
		let now = crate::lan::now_ms().div_euclid(1_000);
		let known = self.desk.p.standing.withdrawn().any(|id| id == offer);
		// the standing offer's game, once an offer: its pump counts on Pump home's recent games once a move is done
		if let Some(game) = self.desk.p.standing.current().filter(|o| o.id == offer).map(|o| o.game.clone()) {
			self.desk.recent_taken(game, now);
		}
		let queued = self.offer_ends(offer, now);
		if !known {
			log::info!("offer {offer}: taken{}", if queued { ", its match_end withdrawn" } else { "" });
		}
		self.show_games();
	}

	/// The card's one soft sound as it shows, and, for a match's (`line`), "Match over." with the voice on for offers.
	/// Owner: DESK-MEDIA (`audio.rs`). Caller: DESK-GAMES (the card).
	pub(crate) fn offer_sound(&mut self, line: bool) {
		self.play_offer(line);
	}
}
