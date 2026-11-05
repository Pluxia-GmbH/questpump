//! The seams between the desktop app's areas (docs/plan.md slice 35): what one area asks of another's state or does
//! through another's part, each one `impl Helper` method its owner gives its body (and only its owner) and every
//! caller calls, never doing its work itself. A body here is the one right before its owner's part exists (no pump
//! runs, an offer taken changes nothing) or hands the call to its owner's file (the sound and voice: `audio.rs`).

use std::path::PathBuf;

use questpump_core::codes::flags;
use questpump_core::protocol::RowRecord;
use serde::{Deserialize, Serialize};

use crate::runtime::Helper;

/// What QuestPump found on this computer (`Helper::found`): whether the stores were read, where the progress they hold
/// came from, the game it knows and the phone paired. The first five minutes read it, so the app never asks what it
/// can see and never treats a player with progress as new (docs/ui.md, "The first five minutes"); progress is a move
/// the player did, never a character the game showed or the start rungs "Play without them" writes, so a new player
/// at the game's setup screen still gets the welcome.
#[derive(Serialize, Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Found {
	/// The stores hold what this computer has: the install the session follows was taken in, or none is known
	/// (`Helper::stores_read`). Nothing is decided before.
	pub(crate) ready: bool,
	/// Where the progress the stores hold came from; none while they hold none.
	pub(crate) progress: Option<Came>,
	pub(crate) game: FoundGame,
	/// The paired phone's name (`Helper::phone_name`); none while no phone is paired.
	pub(crate) phone: Option<String>,
}

/// Where the progress the stores hold came from, by the moves done (`Row::performed`): the game (its rows, taken in from
/// the mirror, the saves or the strip), a phone or its watch (their app rows), both, or this computer's own app rows
/// alone.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Came {
	Game,
	Phone,
	Both,
	Here,
}

/// The game QuestPump knows on this computer.
#[derive(Serialize, Debug, Clone, Copy, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum FoundGame {
	/// A client whose AddOns hold QuestPump runs (`Helper::wow_with_addon`), its squares read or not yet.
	Running,
	/// The install the session follows holds QuestPump (`Helper::followed`).
	Found,
	/// The game is on this computer without QuestPump yet (`Helper::without_addon`, or a client without it runs): the
	/// addon comes from its page first.
	Installed,
	/// No game is known on this computer.
	#[default]
	#[serde(rename = "none")]
	Unfound,
}

/// A move's guide as the install the app follows holds its takes (`Helper::guide_takes`): How's takes then Tips', in
/// the order the voice reads them, and the take Tips begin at, after `callout_gap_s` (none where Tips has no take or
/// How has none before it; Speech.lua `GuideTakes`'s `starts`). No take: no guide to read.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct GuideTakes {
	pub(crate) takes: Vec<GuideTake>,
	pub(crate) tips: Option<usize>,
}

/// One take of a move's guide: its file in the install and its length in seconds.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct GuideTake {
	pub(crate) file: PathBuf,
	pub(crate) seconds: f64,
}

/// What the helper's stores hold (`Helper::store_mark`): the game's records and the app rows by their counts since the
/// reset at `floor` (a reset compacts both stores, and the counts start again), the app's latest Start over
/// (`AppRows::start_over`: the app's own history starts there), and the other computers' rows the sync brought with
/// the characters their device batches named (slice 60: `far`, `far_named`). The stores took rows since an earlier
/// mark when the two differ.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct StoreMark {
	pub(crate) game: usize,
	pub(crate) own: usize,
	pub(crate) floor: Option<u32>,
	pub(crate) start_over: Option<u32>,
	pub(crate) far: usize,
	pub(crate) far_named: usize,
}

/// A cue of the pump's sound and voice (`Helper::cue`): what DESK-PUMP's schedule asks for and when, which DESK-MEDIA
/// plays on the picked output, or says in the one polite live region while a screen reader runs and QuestPump's
/// window has the focus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Cue {
	/// A feedback event of `contract/design/tokens.json` (`feedback`), by its name (`go`, `done`): the desktop column's
	/// sound and voice line.
	Event(&'static str),
	/// A line of `VoiceClips.lua` by its id (`count.10`): its take, silence where it has none.
	Line(String),
}

/// What a recall ended on this computer, opened from outside the window while nothing ran: what "Open the pump when
/// the game closes" started from a match offer taken back (`Helper::offer_recalled`), or, since slice 48's
/// double-check, whatever the tray, the card or the auto-open started as a watched game's new run or a new match began
/// (`Helper::outside_recalled`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Recalled {
	/// The Start a pump window opened for it (after a program's close, or from the tray).
	Choose,
	/// Its start waiting for the place window's answer, and that window.
	Waiting,
	/// The pump the engine composes for it, which shows nowhere as the answer lands.
	Composing,
	/// Its pump on its Ready view with nothing done.
	Ready,
}

impl Helper {
	/// The stores' mark now; each reader keeps the one it last read and reads on when they differ. Owner: the runtime
	/// (its session's stores). Callers: Pump home's game days (`home.rs`), the Book and the Log (`book.rs`), the pump's
	/// pain flags (`pump.rs`) and the engine's door (`engine.rs`).
	pub(crate) fn store_mark(&self) -> StoreMark {
		let store = self.session.store();
		let own = self.session.app_rows();
		StoreMark {
			game: store.records().len(),
			own: own.records().len(),
			floor: store.floor(),
			start_over: own.start_over(),
			far: self.session.far_rows().records().len(),
			far_named: self.session.people().far.values().map(std::collections::BTreeMap::len).sum(),
		}
	}

	/// What QuestPump found on this computer now. Owner: the runtime (its stores, the people, the game it follows, the
	/// pairing). Caller: the first five minutes (`menu_pages/welcome.rs`), whenever one of those changed.
	pub(crate) fn found(&self) -> Found {
		let done = |r: &RowRecord| r.row().performed();
		let game_rows = self.session.store().records().iter().any(done);
		let own = self.session.app_rows().records();
		// an app row carries exactly one device's bit: this computer's, or a phone's or a watch's
		let device = own.iter().any(|r| r.flags & flags::COMPUTER == 0 && done(r));
		let progress = match (game_rows, device) {
			(true, true) => Some(Came::Both),
			(true, false) => Some(Came::Game),
			(false, true) => Some(Came::Phone),
			(false, false) => own.iter().any(done).then_some(Came::Here),
		};
		Found { ready: self.stores_read, progress, game: self.found_game(), phone: self.phone_name() }
	}

	/// The game QuestPump knows on this computer now. Owner: the runtime (the game it follows and runs). Callers: what
	/// QuestPump found (`found`), what every response tells the phones and the watches (`Helper::tell_devices`).
	pub(crate) fn found_game(&self) -> FoundGame {
		if self.wow_with_addon() {
			FoundGame::Running
		} else if self.followed.is_some() {
			FoundGame::Found
		} else if self.without_addon.is_some() || self.game.is_some() {
			FoundGame::Installed
		} else {
			FoundGame::Unfound
		}
	}

	/// A pump runs on this computer (its Ready view to its Result): no match card meanwhile, and Pump home and the
	/// card say so. Owner: DESK-PUMP (`desk/pump.rs`'s machine). Callers: the card (DESK-GAMES), Pump home (DESK-HOME),
	/// the socket's quit and restart and the update's install (`runtime/updates.rs` `quit_held`).
	pub(crate) fn desk_pump_running(&self) -> bool {
		self.pump_running()
	}

	/// A move runs in the pump on this computer, or one is being made (anything but an idle pump: `pump_mid_move`): a
	/// game's offer made or its game closed meanwhile waits for it, never interrupting it. Owner: DESK-PUMP
	/// (`desk/pump.rs`). Callers: DESK-GAMES (`runtime/matches.rs` `offer` and `offer_after_pump`).
	pub(crate) fn desk_pump_mid_move(&self) -> bool {
		self.pump_mid_move()
	}

	/// "Open the pump after each match" opens a game's pump: the pump on screen ends while it is idle, as Leave ends
	/// it (`pump_make_way`); false, nothing changed, while a move runs. Owner: DESK-PUMP (`desk/pump.rs`). Caller:
	/// DESK-GAMES (`desk/card.rs` `open_offer`).
	pub(crate) fn desk_pump_make_way(&mut self) -> bool {
		self.pump_make_way()
	}

	/// A setup change reaches the pump on screen at once (`setup_refilter`): an app pump waiting on its Ready view,
	/// whose Settings opens Your setup over it; Your setup says a change waits for the next pump only while a pump runs
	/// that it does not reach whole (a catch-up's, which takes the pain flags alone). Owner: DESK-PUMP (`desk/pump.rs`). Caller: Your setup
	/// (`menu_pages/setup.rs`).
	pub(crate) fn desk_pump_refits(&self) -> bool {
		self.pump_refits()
	}

	/// What makes the pump on its Ready view changed (the game's refilter, Events.lua `filterKey`): the profile's or the
	/// focus (`profile`: the place's gear, floor moves, room and quiet, the Body switches, the moves left out or checked,
	/// the experience, the focus) or the pain flags Your setup set: an app pump waiting on its Ready view is composed
	/// again while no move of it ended, else refit from its move on, as its Pain does; a catch-up's on its Ready view
	/// rests the moves the flags load (`pump_setup_refit`). Owner: DESK-PUMP (`desk/pump.rs`). Callers: the engine's
	/// profile (`desk/engine.rs` `profile_changed`), Your setup's pain and focus (`menu_pages/setup.rs`).
	pub(crate) fn setup_refilter(&mut self, profile: bool) {
		self.pump_setup_refit(profile);
	}

	/// A pump's Pause menu paused the offers (`pump.pause`, the game's Pause: how long): the pause `kind` names set and
	/// the Games tab and the tray drawn again, or why none was, nothing changed (`offers_paused`). Owner: DESK-GAMES
	/// (`desk/offers.rs`). Caller: DESK-PUMP (`desk/pump.rs` `pump_pause`).
	pub(crate) fn pause_offers(&mut self, kind: &str) -> Result<(), String> {
		self.offers_paused(kind)?;
		self.show_games();
		Ok(())
	}

	/// A pump is being made on this computer (the engine composes or doses it, a match pump's riders among them): the
	/// reminder offer waits as it waits while one runs. Owner: DESK-PUMP (`desk/pump.rs`). Callers: the reminders
	/// (`desk/reminders.rs` `publish_reminders`), the socket's quit and restart and the update's install
	/// (`runtime/updates.rs` `quit_held`).
	pub(crate) fn desk_pump_starting(&self) -> bool {
		self.pump_starting()
	}

	/// This computer's place was answered (`settings.place_asked`): a pump's start that waited for it goes on. Owner:
	/// DESK-PUMP (`desk/pump.rs`). Caller: the place's owner (`menu_pages/setup.rs` `place_changed`: Your setup, the
	/// first five minutes, the place window).
	pub(crate) fn place_answered(&mut self) {
		self.pump_waited();
	}

	/// A World of Warcraft client runs whose AddOns hold QuestPump (docs/plan.md slice 35; a client without it holds
	/// nothing back), by its folder alone: the game asks the moves owed then, so the computer starts no catch-up and a
	/// phone's or a watch's catch-up pays none of its moves (the sync answers' and the sheet's `game_asks`), never
	/// released on a guess. Callers: the payments (the sync handler, the sheet, the computer's catch-up), what QuestPump
	/// found (`found`), `wow_asks_pumps`.
	pub(crate) fn wow_with_addon(&self) -> bool {
		self.game.is_some() && self.game_addons.is_some()
	}

	/// The game asks the pumps (`wow_with_addon`) as far as the screen tells: the computer starts no app pump and no
	/// match offer, unless the client's strip was never read and stayed out of sight for a minute while the screen is
	/// read (`wow_unseen`), when the app pumps start here meanwhile (they neither pay nor owe). Callers: the app pumps'
	/// gate (`desk/pump.rs`), the offers (`runtime/matches.rs`, `desk/card.rs`), Pump home.
	pub(crate) fn wow_asks_pumps(&self) -> bool {
		self.wow_with_addon() && !self.wow_unseen()
	}

	/// A client whose folder holds QuestPump runs, and its strip has not shown for a minute while the screen is read
	/// (This computer's `Screen::Unseen`) and never since it started (a strip read once and later out of sight is the
	/// window covered or a menu, and still asks the pumps): QuestPump has not shown in that game yet (the login screen,
	/// a realm queue, a cinematic, or QuestPump off in it), so the app pumps start here meanwhile (Pump home says so),
	/// while every payment still waits for the game. Callers: `wow_asks_pumps`, Pump home.
	pub(crate) fn wow_unseen(&self) -> bool {
		self.wow_with_addon() && !self.sighting.read && self.sighting.line == crate::ui::Screen::Unseen
	}

	/// A device or this computer took the match offer `offer` (`m-<uuid>`: a phone's or a watch's sync carries it as
	/// `offer_taken` until a response comes, so it may come again; a desktop pump started from the offer): its
	/// `match_end` withdrawn, the standing offer taken, the card closed, its game's pump waiting for a move on Pump
	/// home's recent games (`recent_taken`), and the id answered in `offer_withdrawn` until its `match_end` would have
	/// expired. The test's id (`m-test`) only closes the test's card. Owner: DESK-GAMES. Callers: the sync handler
	/// (`runtime.rs`, `Ask::Synced`), DESK-PUMP (a pump started from an offer).
	pub(crate) fn offer_taken(&mut self, offer: &str) {
		if offer == questpump_core::offers::TEST {
			self.close_test();
			return;
		}
		if self.games.is_none() {
			return;
		}
		let now = crate::lan::now_ms().div_euclid(1_000);
		let known = self.standing.withdrawn().any(|id| id == offer);
		// the standing offer's game, once an offer: its pump counts on Pump home's recent games once a move is done
		if let Some(game) = self.standing.current().filter(|o| o.id == offer).map(|o| o.game.clone()) {
			self.recent_taken(game, now);
		}
		let queued = self.offer_ends(offer, now);
		if !known {
			log::info!("offer {offer}: taken{}", if queued { ", its match_end withdrawn" } else { "" });
		}
		self.show_games();
	}

	/// The match offer `offer` "Open the pump after each match" opened was taken back (its game started again
	/// within `match_relaunch_s` of its close; slice 48): what that started from it ends while nothing ran, nothing
	/// written, before its pump took the offer (the Start a pump window opened for it, its start waiting for the place
	/// with the place window, the pump the engine composes for it) or after (its pump waiting on its Ready view with
	/// nothing done); one the player took further stays. What ended, if anything. Owner: DESK-PUMP (`desk/pump.rs`).
	/// Caller: DESK-GAMES (`runtime/matches.rs`).
	pub(crate) fn offer_recalled(&mut self, offer: &str) -> Option<Recalled> {
		self.pump_recalled(Some(offer))
	}

	/// A watched game's new run or a new match began (slice 48's double-check): whatever the tray, the card or "Open the
	/// pump after each match" opened from outside the window ends while it stands untouched, nothing written (the
	/// Start a pump window, a start waiting for the place with the place window, a pump being composed, the pump at
	/// Ready with nothing done) but a match pump's banked moves (since 2026-10-27, `Helper::pump_recalled`); one the
	/// player started from the page or took further stays. What ended, if anything.
	/// Owner: DESK-PUMP (`desk/pump.rs`). Caller: DESK-GAMES (`runtime/matches.rs`).
	pub(crate) fn outside_recalled(&mut self) -> Option<Recalled> {
		self.pump_recalled(None)
	}

	/// A start of a pump from the match offer `offer` was asked and went through (the card's or the tray's Start,
	/// Pump home's, the Games tab's Now card's, Start a pump opened from it, "Open the pump after each match"): the
	/// player answered it on this computer, so its card goes, it no longer goes out to the devices and its first Start
	/// counts (`game_offer_desk` started); it stands until its pump starts (`offer_taken`). Owner: DESK-GAMES
	/// (`desk/card.rs`). Caller: DESK-PUMP (`pump_start_intent`).
	pub(crate) fn offer_started(&mut self, offer: &str) {
		self.offer_answered(offer);
	}

	/// The card's one soft sound as it shows, and, for a match's (`line`), "Match over." with the voice on for offers.
	/// Owner: DESK-MEDIA (`audio.rs`). Caller: DESK-GAMES (the card).
	pub(crate) fn offer_sound(&mut self, line: bool) {
		self.play_offer(line);
	}

	/// A cue of the pump's sound and voice, now. Owner: DESK-MEDIA (`audio.rs`: the sounds, the takes, the picked
	/// output, the live region while a screen reader runs and the window has the focus). Callers: DESK-PUMP (its cue
	/// schedule), the page's kit (`sound.event`).
	pub(crate) fn cue(&mut self, cue: Cue) {
		self.play_cue(cue);
	}

	/// A voice line's take lasts this long (seconds), none for a line with no take: a callout waits for the move's line.
	/// Owner: DESK-MEDIA (`audio.rs`, the takes' receipts). Caller: DESK-PUMP.
	pub(crate) fn line_seconds(id: &str) -> Option<f64> {
		Self::take_length(id)
	}

	/// A move's callout follows its line on the Ready view: the Sound and voice page's Callouts switch. Owner: DESK-MEDIA
	/// (`audio.rs`). Caller: DESK-PUMP.
	pub(crate) fn callouts_on(&self) -> bool {
		self.callouts_wanted()
	}

	/// A set of reps' count goes to the voice now (heard at its level): the sounds, the voice and the Sound and voice
	/// page's "Count reps aloud" on, and no screen reader taking the lines (a count is skipped, never announced). Owner:
	/// DESK-MEDIA (`audio.rs`). Caller: DESK-PUMP, at each count.
	pub(crate) fn counts_on(&self) -> bool {
		self.counts_wanted()
	}

	/// The player's switches let the voice count a set of reps: the sounds, the voice and "Count reps aloud" on,
	/// whether or not a count is heard now (a screen reader, a skipped count), so Big view shows the reps counted and
	/// the count step starts from them. Owner: DESK-MEDIA (`audio.rs`). Caller: DESK-PUMP, at each publish and at Done.
	pub(crate) fn counts_chosen(&self) -> bool {
		self.count_switched()
	}

	/// The voice stops at once: a pump stopped (pain, felt unwell, the player's or the game's end), went back to Ready
	/// or left the screen (Speech.lua's `StopSound`). Owner: DESK-MEDIA (`audio.rs`). Caller: DESK-PUMP.
	pub(crate) fn hush(&self) {
		self.hush_voice();
	}

	/// The output stays open while a pump runs (`on` at its start, off at its end), so a tick every few seconds never
	/// waits for an output to wake. Owner: DESK-MEDIA (`audio.rs`). Caller: DESK-PUMP.
	pub(crate) fn hold_sound(&self, on: bool) {
		self.hold_output(on);
	}

	/// The takes of move `key`'s guide, the implement's (`gear`) where it has its own, from the install the app
	/// follows, each only where its words are the app's. Owner: DESK-MEDIA (`audio.rs`). Caller: DESK-PUMP (Ready's
	/// read-aloud button, `pump/guide.rs`).
	pub(crate) fn guide_takes(&mut self, key: &str, gear: Option<&str>) -> GuideTakes {
		self.guide_read(key, gear)
	}

	/// A take of a move's guide plays now on a player of its own, its take before stopping, at the voice's level
	/// whatever the switches say and never to the live region; no line stops it. Owner: DESK-MEDIA (`audio.rs`).
	/// Caller: DESK-PUMP.
	pub(crate) fn guide_play(&self, take: &GuideTake) {
		self.play_guide(take.file.clone());
	}

	/// The guide's take stops (paused, held, or its move's run over). Owner: DESK-MEDIA (`audio.rs`). Caller:
	/// DESK-PUMP.
	pub(crate) fn guide_stop(&self) {
		self.stop_guide();
	}

	/// A voice line `seconds` long plays now (`id`): a move's guide reading stops at its take and goes on
	/// `callout_gap_s` after the line ends, a count aside, which plays over it (Speech.lua `Say`). Owner: DESK-PUMP
	/// (`pump/guide.rs`). Caller: DESK-MEDIA (`audio.rs` `voice_line`, the one way a pump's line reaches the voice).
	pub(crate) fn guide_hold(&mut self, id: &str, seconds: f64) {
		self.pump_guide_hold(id, seconds);
	}
}
