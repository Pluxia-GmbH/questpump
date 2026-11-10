//! The seams between the desk's areas (docs/plan.md slice 35): what one area asks of another's state or does through
//! another's part, each one `impl Desk` method its owner gives its body (and only its owner) and every caller calls,
//! never doing its work itself. A body here hands the call to its owner's file (the pump's, `pump.rs`; the sound and
//! voice, `audio.rs`) or to the platform (`Platform`: the game, the offers, the audio's output). The desktop app's own
//! seams (the game it follows, the match offers) are its `desk/seams.rs`.

use questpump_core::protocol::{RowRecord, Source};
use serde::{Deserialize, Serialize};

use crate::platform::{FoundGame, GuideTake, GuideTakes};
use crate::{Desk, Platform};

/// What QuestPump found on this computer (`Desk::found`): whether the stores were read, where the progress they hold
/// came from, the game it knows and the phone paired. The first five minutes read it, so the app never asks what it
/// can see and never treats a player with progress as new (docs/ui.md, "The first five minutes"); progress is a move
/// the player did, never a character the game showed or the start rungs "Play without them" writes, so a new player
/// at the game's setup screen still gets the welcome.
#[derive(Serialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct Found {
	/// The stores hold what this computer has: the install the session follows was taken in, or none is known
	/// (`Platform::stores_read`). Nothing is decided before.
	pub ready: bool,
	/// Where the progress the stores hold came from; none while they hold none.
	pub progress: Option<Came>,
	pub game: FoundGame,
	/// The paired phone's name (`Desk::phone_name`); none while no phone is paired.
	pub phone: Option<String>,
}

/// Where the progress the stores hold came from, by the moves done (`Row::performed`): the game (its rows, taken in from
/// the mirror, the saves or the strip), a phone or its watch (their app rows), both, or this computer's own app rows
/// alone.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Came {
	Game,
	Phone,
	Both,
	Here,
}

/// What the helper's stores hold (`Desk::store_mark`): the game's records and the app rows by their counts since the
/// reset at `floor` (a reset compacts both stores, and the counts start again), the app's latest Start over
/// (`AppRows::start_over`: the app's own history starts there), and the other computers' rows the sync brought with
/// the characters their device batches named (slice 60: `far`, `far_named`). The stores took rows since an earlier
/// mark when the two differ.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StoreMark {
	pub game: usize,
	pub own: usize,
	pub floor: Option<u32>,
	pub start_over: Option<u32>,
	pub far: usize,
	pub far_named: usize,
}

/// A cue of the pump's sound and voice (`Desk::cue`): what DESK-PUMP's schedule asks for and when, which DESK-MEDIA
/// plays on the picked output, or says in the one polite live region while a screen reader runs and QuestPump's
/// window has the focus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Cue {
	/// A feedback event of `contract/design/tokens.json` (`feedback`), by its name (`go`, `done`): the desktop column's
	/// sound and voice line.
	Event(&'static str),
	/// A line of `VoiceClips.lua` by its id (`count.10`): its take, silence where it has none.
	Line(String),
}

/// What a recall ended on this computer, opened from outside the window while nothing ran: what "Open the pump when
/// the game closes" started from a match offer taken back (`Desk::offer_recalled`), or, since slice 48's
/// double-check, whatever the tray, the card or the auto-open started as a watched game's new run or a new match began
/// (`Desk::outside_recalled`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Recalled {
	/// The Start a pump window opened for it (after a program's close, or from the tray).
	Choose,
	/// Its start waiting for the place window's answer, and that window.
	Waiting,
	/// The pump the engine composes for it, which shows nowhere as the answer lands.
	Composing,
	/// Its pump on its Ready view with nothing done.
	Ready,
}

impl<P: Platform> Desk<P> {
	/// The stores' mark now; each reader keeps the one it last read and reads on when they differ. Owner: the runtime
	/// (its session's stores). Callers: Pump home's game days (`home.rs`), the Book and the Log (`book.rs`), the pump's
	/// pain flags (`pump.rs`) and the engine's door (`engine.rs`).
	pub fn store_mark(&self) -> StoreMark {
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
	pub fn found(&self) -> Found {
		let done = |r: &RowRecord| r.row().performed();
		let game_rows = self.session.store().records().iter().any(done);
		let own = self.session.app_rows().records();
		// an app row carries exactly one device's bit: this platform's, or a phone's or a watch's
		let device = own.iter().any(|r| r.flags & P::DEVICE.bit() == 0 && done(r));
		let progress = match (game_rows, device) {
			(true, true) => Some(Came::Both),
			(true, false) => Some(Came::Game),
			(false, true) => Some(Came::Phone),
			(false, false) => own.iter().any(done).then_some(Came::Here),
		};
		Found { ready: self.p.stores_read(), progress, game: self.p.game().found, phone: self.phone_name() }
	}

	/// A pump runs on this computer (its Ready view to its Result): no match card meanwhile, and Pump home and the
	/// card say so. Owner: DESK-PUMP (`pump.rs`'s machine). Callers: the card (DESK-GAMES), Pump home (DESK-HOME),
	/// the socket's quit and restart and the update's install (`runtime/updates.rs` `quit_held`).
	pub fn desk_pump_running(&self) -> bool {
		self.pump_running()
	}

	/// A move runs in the pump on this computer, or one is being made (anything but an idle pump: `pump_mid_move`): a
	/// game's offer made or its game closed meanwhile waits for it, never interrupting it. Owner: DESK-PUMP
	/// (`pump.rs`). Callers: DESK-GAMES (`runtime/matches.rs` `offer` and `offer_after_pump`).
	pub fn desk_pump_mid_move(&self) -> bool {
		self.pump_mid_move()
	}

	/// "Open the pump after each match" opens a game's pump: the pump on screen ends while it is idle, as Leave ends
	/// it (`pump_make_way`); false, nothing changed, while a move runs. Owner: DESK-PUMP (`pump.rs`). Caller:
	/// DESK-GAMES (`desk/card.rs` `open_offer`).
	pub fn desk_pump_make_way(&mut self) -> bool {
		self.pump_make_way()
	}

	/// A setup change reaches the pump on screen at once (`setup_refilter`): an app pump waiting on its Ready view,
	/// whose Settings opens Your setup over it; Your setup says a change waits for the next pump only while a pump runs
	/// that it does not reach whole (a catch-up's, which takes the pain flags alone). Owner: DESK-PUMP (`pump.rs`). Caller: Your setup
	/// (`menu_pages/setup.rs`).
	pub fn desk_pump_refits(&self) -> bool {
		self.pump_refits()
	}

	/// What makes the pump on its Ready view changed (the game's refilter, Events.lua `filterKey`): the profile's or the
	/// focus (`profile`: the place's gear, floor moves, room and quiet, the Body switches, the moves left out or checked,
	/// the experience, the focus) or the pain flags Your setup set: an app pump waiting on its Ready view is composed
	/// again while no move of it ended, else refit from its move on, as its Pain does; a catch-up's on its Ready view
	/// rests the moves the flags load (`pump_setup_refit`). Owner: DESK-PUMP (`pump.rs`). Callers: the engine's
	/// profile (`engine.rs` `profile_changed`), Your setup's pain and focus (`menu_pages/setup.rs`).
	pub fn setup_refilter(&mut self, profile: bool) {
		self.pump_setup_refit(profile);
	}

	/// A pump's Pause menu paused the offers (`pump.pause`, the game's Pause: how long): the pause `kind` names set and
	/// the Games tab and the tray drawn again, or why none was, nothing changed (`offers_paused`). Owner: DESK-GAMES
	/// (`desk/offers.rs`). Caller: DESK-PUMP (`pump.rs` `pump_pause`).
	///
	/// # Errors
	/// Why no pause started (`Platform::offers_pause`).
	pub fn pause_offers(&mut self, kind: &str) -> Result<(), String> {
		let pause = self.p.offers_pause(kind, &self.settings.games)?;
		log::info!("offers: the pause {kind}");
		self.settings.offers.pause = pause;
		self.save_settings();
		self.p.offers_paused();
		Ok(())
	}

	/// A pump is being made on this computer (the engine composes or doses it, a match pump's riders among them): the
	/// reminder offer waits as it waits while one runs. Owner: DESK-PUMP (`pump.rs`). Callers: the reminders
	/// (`desk/reminders.rs` `publish_reminders`), the socket's quit and restart and the update's install
	/// (`runtime/updates.rs` `quit_held`).
	pub fn desk_pump_starting(&self) -> bool {
		self.pump_starting()
	}

	/// This computer's place was answered (`settings.place_asked`): a pump's start that waited for it goes on. Owner:
	/// DESK-PUMP (`pump.rs`). Caller: the place's owner (`menu_pages/setup.rs` `place_changed`: Your setup, the
	/// first five minutes, the place window).
	pub fn place_answered(&mut self) {
		self.pump_waited();
	}

	/// A World of Warcraft client runs whose AddOns hold QuestPump (docs/plan.md slice 35; a client without it holds
	/// nothing back), by its folder alone: the game asks the moves owed then, so the computer starts no catch-up and a
	/// phone's or a watch's catch-up pays none of its moves (the sync answers' and the sheet's `game_asks`), never
	/// released on a guess. Callers: the payments (the sync handler, the sheet, the computer's catch-up), what QuestPump
	/// found (`found`), `wow_asks_pumps`.
	pub fn wow_with_addon(&self) -> bool {
		self.p.game().with_addon
	}

	/// The game asks the pumps (`wow_with_addon`) as far as the screen tells: the computer starts no app pump and no
	/// match offer, unless the client's strip was never read and stayed out of sight for a minute while the screen is
	/// read (`wow_unseen`), when the app pumps start here meanwhile (they neither pay nor owe). Callers: the app pumps'
	/// gate (`pump.rs`), the offers (`runtime/matches.rs`, `desk/card.rs`), Pump home.
	pub fn wow_asks_pumps(&self) -> bool {
		self.p.game().asks_pumps()
	}

	/// A client whose folder holds QuestPump runs, and its strip has not shown for a minute while the screen is read
	/// (This computer's `Screen::Unseen`) and never since it started (a strip read once and later out of sight is the
	/// window covered or a menu, and still asks the pumps): QuestPump has not shown in that game yet (the login screen,
	/// a realm queue, a cinematic, or QuestPump off in it), so the app pumps start here meanwhile (Pump home says so),
	/// while every payment still waits for the game. Callers: `wow_asks_pumps`, Pump home.
	pub fn wow_unseen(&self) -> bool {
		self.p.game().unseen
	}

	/// The match offer `offer` "Open the pump after each match" opened was taken back (its game started again
	/// within `match_relaunch_s` of its close; slice 48): what that started from it ends while nothing ran, nothing
	/// written, before its pump took the offer (the Start a pump window opened for it, its start waiting for the place
	/// with the place window, the pump the engine composes for it) or after (its pump waiting on its Ready view with
	/// nothing done); one the player took further stays. What ended, if anything. Owner: DESK-PUMP (`pump.rs`).
	/// Caller: DESK-GAMES (`runtime/matches.rs`).
	pub fn offer_recalled(&mut self, offer: &str) -> Option<Recalled> {
		self.pump_recalled(Some(offer))
	}

	/// A watched game's new run or a new match began (slice 48's double-check): whatever the tray, the card or "Open the
	/// pump after each match" opened from outside the window ends while it stands untouched, nothing written (the
	/// Start a pump window, a start waiting for the place with the place window, a pump being composed, the pump at
	/// Ready with nothing done) but a match pump's banked moves (since 2026-10-27, `Desk::pump_recalled`); one the
	/// player started from the page or took further stays. What ended, if anything.
	/// Owner: DESK-PUMP (`pump.rs`). Caller: DESK-GAMES (`runtime/matches.rs`).
	pub fn outside_recalled(&mut self) -> Option<Recalled> {
		self.pump_recalled(None)
	}

	/// A start of a pump from the match offer `offer` was asked and went through (the card's or the tray's Start,
	/// Pump home's, the Games tab's Now card's, Start a pump opened from it, "Open the pump after each match"): the
	/// player answered it on this computer, so its card goes, it no longer goes out to the devices and its first Start
	/// counts (`game_offer_desk` started); it stands until its pump starts (`offer_taken`). Owner: DESK-GAMES
	/// (`desk/card.rs`). Caller: DESK-PUMP (`pump_start_intent`).
	pub fn offer_started(&mut self, offer: &str) {
		self.p.offer_started(offer);
	}

	/// A cue of the pump's sound and voice, now. Owner: DESK-MEDIA (`audio.rs`: the sounds, the takes, the picked
	/// output, the live region while a screen reader runs and the window has the focus). Callers: DESK-PUMP (its cue
	/// schedule), the page's kit (`sound.event`).
	pub fn cue(&mut self, cue: Cue) {
		self.play_cue(cue);
	}

	/// A voice line's take lasts this long (seconds), none for a line with no take: a callout waits for the move's line.
	/// Owner: DESK-MEDIA (`audio.rs`, the takes' receipts). Caller: DESK-PUMP.
	#[must_use]
	pub fn line_seconds(id: &str) -> Option<f64> {
		Self::take_length(id)
	}

	/// A move's callout follows its line on the Ready view: the Sound and voice page's Callouts switch. Owner: DESK-MEDIA
	/// (`audio.rs`). Caller: DESK-PUMP.
	pub fn callouts_on(&self) -> bool {
		self.callouts_wanted()
	}

	/// A set of reps' count goes to the voice now (heard at its level): the sounds, the voice and the Sound and voice
	/// page's "Count reps aloud" on, and no screen reader taking the lines (a count is skipped, never announced). Owner:
	/// DESK-MEDIA (`audio.rs`). Caller: DESK-PUMP, at each count.
	pub fn counts_on(&self) -> bool {
		self.counts_wanted()
	}

	/// The player's switches let the voice count a set of reps: the sounds, the voice and "Count reps aloud" on,
	/// whether or not a count is heard now (a screen reader, a skipped count), so Big view shows the reps counted and
	/// the count step starts from them. Owner: DESK-MEDIA (`audio.rs`). Caller: DESK-PUMP, at each publish and at Done.
	pub fn counts_chosen(&self) -> bool {
		self.count_switched()
	}

	/// The voice stops at once: a pump stopped (pain, felt unwell, the player's or the game's end), went back to Ready
	/// or left the screen (Speech.lua's `StopSound`). Owner: DESK-MEDIA (`audio.rs`). Caller: DESK-PUMP.
	pub fn hush(&self) {
		self.p.hush();
	}

	/// The output stays open while a pump runs (`on` at its start, off at its end), so a tick every few seconds never
	/// waits for an output to wake. Owner: DESK-MEDIA (`audio.rs`). Caller: DESK-PUMP.
	pub fn hold_sound(&self, on: bool) {
		self.p.hold_output(on);
	}

	/// The takes of move `key`'s guide, the implement's (`gear`) where it has its own, from the install the app
	/// follows, each only where its words are the app's. Owner: DESK-MEDIA (`audio.rs`). Caller: DESK-PUMP (Ready's
	/// read-aloud button, `pump/guide.rs`).
	pub fn guide_takes(&mut self, key: &str, gear: Option<&str>) -> GuideTakes {
		self.p.guide_takes(key, gear)
	}

	/// A take of a move's guide plays now on a player of its own, its take before stopping, at the voice's level
	/// whatever the switches say and never to the live region; no line stops it. Owner: DESK-MEDIA (`audio.rs`).
	/// Caller: DESK-PUMP.
	pub fn guide_play(&self, take: &GuideTake) {
		self.p.play_guide(take);
	}

	/// The guide's take stops (paused, held, or its move's run over). Owner: DESK-MEDIA (`audio.rs`). Caller:
	/// DESK-PUMP.
	pub fn guide_stop(&self) {
		self.p.stop_guide();
	}

	/// A voice line `seconds` long plays now (`id`): a move's guide reading stops at its take and goes on
	/// `callout_gap_s` after the line ends, a count aside, which plays over it (Speech.lua `Say`). Owner: DESK-PUMP
	/// (`pump/guide.rs`). Caller: DESK-MEDIA (`audio.rs` `voice_line`, the one way a pump's line reaches the voice).
	pub fn guide_hold(&mut self, id: &str, seconds: f64) {
		self.pump_guide_hold(id, seconds);
	}

	/// The paired phone's name for the window: the name it gave at pairing, else its kind; none while no phone is
	/// paired. Owner: the settings' pairing record. Callers: what QuestPump found (`found`), the Phones page, the tray.
	pub fn phone_name(&self) -> Option<String> {
		let phone = self.settings.pairing.phone.as_ref()?;
		Some(self.settings.phone_name.clone().unwrap_or_else(|| phone.clone()))
	}

	/// The phone's step shows in the desk's own screens: the addon's verified feed does not say the phone apps are still
	/// out of the stores (none yet: as before). Callers: Pump home's card, the first five minutes, the desktop app's.
	pub fn phones_offered(&self) -> bool {
		self.settings.pairing.phone_apps != Some(false)
	}

	/// Whether Pro's extras show: on while any phone's verified grant or the account's stands now
	/// (`Platform::pro_grants`), whether or not the pairing block names a phone, so one phone's Unpair never hides the
	/// grants of the others that hold the code; off while a phone is paired without one; none while neither (the Menu's
	/// Pro page then offers to pair a phone, and Get Pro).
	pub fn pro_on(&self) -> Option<bool> {
		let on = self.p.pro_grants(self.settings.sync.account.as_deref());
		(on || self.settings.pairing.phone.is_some()).then_some(on)
	}

	/// The newest performed move any device made, among the rows the stores hold (unix seconds): the sitting clock's
	/// start.
	pub fn last_move_at(&self) -> Option<i64> {
		self.last_move(|_| true)
	}

	/// The newest performed move made in the game this computer reads (unix seconds; `Source::Game`: its strip, mirror
	/// and saves): work mode's anchor follows it (`Desk::work_moved`), the game being the one device that moves no
	/// anchor itself; this device's own pumps move the anchor at their end, and every other device's at theirs, heard
	/// in `work` (a phone's moves moving it here would end the ask its own work pump answers before that pump ends).
	pub fn last_move_in_game(&self) -> Option<i64> {
		self.last_move(|r| r.source == Source::Game)
	}

	/// The newest performed move among the stores' records `of` keeps (unix seconds).
	fn last_move(&self, of: impl Fn(&RowRecord) -> bool) -> Option<i64> {
		let f = crate::facts::facts();
		self.session
			.store()
			.records()
			.iter()
			.chain(self.session.app_rows().records())
			.filter(|r| of(r))
			.map(RowRecord::row)
			.filter(|row| {
				questpump_core::codes::kind::is_move(row.kind)
					&& questpump_core::codes::outcome::is_performed(row.outcome)
					&& f.knows(row.move_id)
			})
			.map(|row| i64::from(row.ts))
			.max()
	}

	/// The name of the first game marked `aim` with its switch on that runs now (contract/links.json's): a pump from an
	/// offer or a match picked by hand rests the arms while one runs, and its Ready view names it.
	pub fn arms_game(&self) -> Option<String> {
		self.p.arms_game(&self.settings.games)
	}

	/// Whether a game marked `aim` with its switch on runs now (`arms_game`).
	pub fn arms_now(&self) -> bool {
		self.arms_game().is_some()
	}

	/// The sitting clock at unix second `now`, without the offers (the minutes sat a match pump's warm move reads).
	pub fn seated_s(&self, now: i64) -> i64 {
		self.p.seated_s(now, self.last_move_at())
	}
}
