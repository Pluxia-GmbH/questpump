//! The games on the computer in the loop (docs/plan.md slices 35, 48 and 57, docs/computer-games.md §3-§4;
//! `crate::games`): each 2-s look takes one process snapshot, which `game.rs` read first, and gives it to the games;
//! the listener's posts and the feeds come on their own channels. Every finish is decided at once
//! (`questpump_core::offers::decide`: since slice 57 every match asks) and logged as `offer <key> end: offered (<id>)`
//! or `offer <key> end: dropped (<reason>)`, the game's key and the offer's id alone. An offer made stands
//! (`Standing`) with the size its match's own minutes gave (none from before the end of the input's latest pause),
//! whether the player had sat `match_sat_min` on the sitting clock without the offers (`runtime/sitting.rs`
//! `seated_s`: its pump's warm first move) and the arms rest, a standing sitting cue ends (no reminder while it
//! stands), and its card shows as the look finds it due (`desk::card`), a match's end a feed told `offers::LOOK_S`
//! after it, so the player sees the game's last screen first (Rapha, 2026-11-03). The computer asks first (slice 48's
//! double-check): the offer goes as a `match_end` to the device that holds the link (`match_offer_min` to live, a
//! newer one replacing it there) only at a look after the card's chance where the computer cannot ask (`out_look`:
//! the card could not show, the game in front on its one screen, or showed to no input for `match_away_s`; `offer
//! <id>: out to the devices`), never once its card's time passed or the player answered it here (`offer <id>: answered
//! on the computer, not sent to the devices`). An offer made at a game's close (any watched game's, slice 56) is held
//! `match_relaunch_s` first: a start again within that time takes it back unheard by any device (`offer <id>:
//! recalled, its game started again`), and ends what "Open the pump after each match" started from it
//! while nothing ran: the Start a pump window opened for it, a start waiting for the place, a pump being composed, or
//! the pump at Ready with nothing done (`offer <id>: <what> ends, its game started again`). A watched game's new run or
//! a new match ends whatever was opened from outside the window and stands untouched, nothing written but a match
//! pump's banked moves (`pump: <what> ends, a game started`). An offer a feed told starts the sitting clock again at
//! once, one at a game's close as its hold ends (`last_game_offer_at`: the sitting cue's, never the warm move's); no
//! quiet hours, cap or gap hold a match's offer back (slice 48), nor since slice 57 the sitting clock. A performed
//! move on any device newer than the
//! offer ends it (withdrawn on the devices when it went out). An offer made opens its pump when the player chose so
//! (on by default), at a game's close at once and at a match's end a feed told once the look ends (Rapha, 2026-10-29;
//! at the close alone until then),
//! in the place of an idle pump on this computer, and after one whose move runs, which the offer waits for (2026-10-28;
//! its moves never answer it). A pause whose time
//! passed or whose game stopped ends on the look. Every sync answers the offers' part: the ids taken back
//! (`offer_withdrawn`), a watched game in a match (`playing`), an aim game resting the arms (`arms`) and the latest
//! game offer on any device (`game_offer_at`), into which the device's own is folded first.

use questpump_core::games::{Finish, MATCH, Watched};
use questpump_core::offers::{self, Gate, Offer, Pause, Tunables};
use questpump_core::protocol::{Command, Line};

use super::Helper;
use crate::desk::seams::Recalled;
use crate::games::procs::Snapshot;
use crate::games::{Heard, News};
use crate::lan::{self, OfferFacts};

/// What a recall or a game's start ended (`Recalled`), logged for `who` (the offer, or the pump) and `why`.
fn ended_log(ended: Recalled, who: &str, why: &str) {
	match ended {
		Recalled::Choose => log::info!("{who}: the Start a pump window ends, {why}"),
		Recalled::Waiting => log::info!("{who}: the start waiting for the place ends, {why}"),
		Recalled::Composing => log::info!("{who}: the pump being composed ends, {why}"),
		Recalled::Ready => log::info!("{who}: the pump at Ready ends, {why}"),
	}
}

impl Helper {
	/// The games' settings changed: saved, and on Windows the uninstaller's list of the files written beside them.
	pub(crate) fn save_games(&self) {
		self.save_settings();
		#[cfg(windows)]
		self.save_file(crate::games::files::LIST, crate::games::files::list_bytes(&self.settings.games));
	}

	/// One look at the games, after the game's: a game started again recalls its close's offer, a new run or match
	/// ends what opened from outside the window untouched, the running games' screens are noted, their finishes
	/// offered, the standing offer and the pause expired, a held offer's hold ended, an offer a move answered since
	/// ended, an offer that waited for a move here taken up once the pump is idle, a match's end opened once the player's
	/// look at the game's last screen ended (`offers::LOOK_S`), a closed game's offer opened when the player chose so,
	/// the card shown or closed, the offer out to the devices where the computer cannot ask, the pages
	/// and the tray drawn.
	pub(super) fn on_games_tick(&mut self, snap: &Snapshot) {
		let Some(games) = self.games.as_mut() else { return };
		let ms = lan::now_ms();
		let unix = ms.div_euclid(1_000);
		let ticked = games.tick(snap, ms, &mut self.settings.games);
		let t = games.tunables();
		// a game kept by its Steam app from now on: a pause while it runs names its new key, before the look asks
		// whether it still runs (saved with the games' settings, which the new key changed)
		for (from, to) in &ticked.rekeyed {
			if let Pause::WhileRunning { game } = &mut self.settings.offers.pause
				&& game == from
			{
				game.clone_from(to);
			}
		}
		let pause_over = self.settings.offers.pause.over(unix, games.paused_runs(&self.settings.offers.pause));
		if ticked.changed {
			self.save_games();
		}
		// before the holds end, so a start again in the look its hold ends still takes the offer back
		for game in &ticked.resumed {
			self.recall(game);
		}
		// before this look's offers, whose "Open the pump after each match" it would end
		self.game_started(!ticked.started.is_empty());
		self.screens_look(snap, unix);
		for finish in &ticked.finishes {
			self.offer(finish, unix);
		}
		self.standing.expire(unix, t);
		if let Some(offer) = self.standing.hold_ends(unix).cloned() {
			log::info!("offer {}: its game did not start again, the sitting clock starts again from it", offer.id);
			self.clock_from(&offer);
		}
		self.offer_done_since(unix);
		self.offer_after_pump(unix);
		if let Some(game) = self.standing.look_ends(unix).map(|o| o.game.clone()) {
			self.open_offer(&game, unix);
		}
		if pause_over {
			log::info!("offers: the pause ended");
			self.settings.offers.pause = Pause::None;
			self.save_settings();
		}
		for game in &ticked.closed {
			self.open_offer(game, unix);
		}
		self.card_look(snap, unix);
		self.out_look(unix, t);
		self.show_games();
	}

	/// A post from CS2 or Dota 2.
	pub(super) fn on_gsi(&mut self, heard: Heard) {
		let Some(games) = self.games.as_mut() else { return };
		let ms = lan::now_ms();
		let (finish, changed) = games.on_gsi(heard, ms, &mut self.settings.games);
		if changed {
			self.save_games();
		}
		self.game_started(false);
		if let Some(finish) = finish {
			self.offer(&finish, ms.div_euclid(1_000));
		}
		self.show_games();
	}

	/// What a game's feed says: Rocket League's, or a fed game's task (slice 48); a match's end offers at once.
	pub(super) fn on_feed(&mut self, news: News) {
		let Some(games) = self.games.as_mut() else { return };
		let ms = lan::now_ms();
		let (finish, changed) = games.on_feed(news, ms, &mut self.settings.games);
		if changed {
			self.save_games();
		}
		self.game_started(false);
		if let Some(finish) = finish {
			self.offer(&finish, ms.div_euclid(1_000));
		}
		self.show_games();
	}

	/// A finish's offer, decided at unix second `now` against the lock, a World of Warcraft client with QuestPump, the
	/// computer's pause and the finish's age (a pump on this computer dropped it until 2026-10-28; one made while a move
	/// runs there waits for that pump, `CardState::wait_pump`), the sitting clock without the offers telling
	/// only whether its pump opens warm (`seated_s`, `match_sat_min`), no match counting minutes from before the end of
	/// the input's latest pause (`Sitting::kept`) and no League lead counting minutes from before the newest performed
	/// move on any device (`last_move_at`, slice 59); one made stands, counts, ends a standing sitting cue and its card
	/// shows; a match's end a feed told starts the sitting clock again at once (the sitting cue's, whose stand-up
	/// ask the offer is). Every one made opens its pump when the player chose so (`open_offer`: a game's close at once,
	/// a match's end a feed told once its look ends, `offers::LOOK_S`); one made at a game's close (`Finish::at_close`:
	/// any watched game's, a fed game's closed mid-match among them) is also held `match_relaunch_s`. None goes to the devices here (`out_look`).
	fn offer(&mut self, finish: &Finish, now: i64) {
		let Some(games) = self.games.as_ref() else { return };
		let t = games.tunables();
		let o = &self.settings.offers;
		let gate = Gate {
			now,
			wow: self.wow_asks_pumps(),
			locked_until: self.session.locked_until(now),
			pause: &o.pause,
			paused_runs: games.paused_runs(&o.pause),
			sat_s: self.seated_s(now),
			moved_at: self.last_move_at(),
			back_at: self.sitting.kept(),
			arms: self.arms_runs(&finish.watched),
		};
		let mut random = [0u8; 16];
		rand::fill(&mut random);
		let key = finish.watched.link();
		match offers::decide(finish, &gate, t, random) {
			Ok(offer) => {
				log::info!("offer {key} end: offered ({})", offer.id);
				let (link, from) = (offer.link, offer.game.clone());
				if offer.held_until.is_none() {
					self.clock_from(&offer);
				}
				self.offering.card.stands(&offer.id, &from);
				if self.desk_pump_mid_move() {
					log::info!("offer {}: a move runs here, the offer waits for the pump", offer.id);
					self.offering.card.wait_pump(&offer.id);
				}
				self.standing.replace(offer, t);
				self.reminder_sitting_ends();
				self.open_offer(&from, now);
				self.counts_offer(link, "shown");
			}
			Err(why) => log::info!("offer {key} end: dropped ({why})"),
		}
	}

	/// Whether the arms rest goes with the game `watched`'s offer: contract/links.json marks it `aim`, its switch in
	/// the game's window is on (`settings.games.aim_off`) and its process ran at the last look.
	fn arms_runs(&self, watched: &Watched) -> bool {
		watched.aim()
			&& self.settings.games.aim_on(watched.link())
			&& self.games.as_ref().is_some_and(|g| g.running().iter().any(|id| id == watched.id()))
	}

	/// The name of the first game marked `aim` with its switch on that runs now (contract/links.json's): a pump on this
	/// computer from an offer or a match picked by hand rests the arms while one runs, and its Ready view names it.
	pub(crate) fn arms_game(&self) -> Option<String> {
		let games = self.games.as_ref()?;
		games
			.running()
			.iter()
			.filter_map(|id| Watched::from_key(id))
			.find(|w| self.arms_runs(w))
			.map(|w| w.name().to_owned())
	}

	/// Whether a game marked `aim` with its switch on runs now (`arms_game`): every sync response says so (`arms`).
	pub(crate) fn arms_now(&self) -> bool {
		self.arms_game().is_some()
	}

	/// The sitting clock starts again from the offer's moment (the latest game offer, which every sync answers and the
	/// sitting cue counts from, never a match's warm move: `seated_s`): one a feed told as it is made, one at a game's
	/// close as its hold ends.
	fn clock_from(&mut self, offer: &Offer) {
		self.settings.offers.game_offered(offer.at);
		self.save_settings();
	}

	/// The game `game` (`Watched::id`) started again within `match_relaunch_s` of a close that offered (a
	/// reconnect, a crash started again): its held offer no longer stands and its card closes, nothing withdrawn since
	/// no device heard of it, and what "Open the pump after each match" started from the offer ends while nothing
	/// ran, before its pump took the offer (the Start a pump window, a start waiting for the place, a pump being
	/// composed) or after (its pump at Ready with nothing done; DESK-PUMP's `offer_recalled`). Its run goes on, and its
	/// later close may offer again.
	fn recall(&mut self, game: &str) {
		if let Some(offer) = self.standing.recall(game) {
			log::info!("offer {}: recalled, its game started again", offer.id);
			if self.offering.card.shows(&offer.id) {
				self.close_card();
			}
		}
		let Some(opened) = self.offering.card.opened_from(game).map(str::to_owned) else { return };
		if let Some(ended) = self.offer_recalled(&opened) {
			ended_log(ended, &format!("offer {opened}"), "its game started again"); // copy-lint: not shown (a log line)
		}
	}

	/// A watched game's new run (`started`: a process start no relaunch continues) or a new match (a watched game in a
	/// match now and none at the last look or post, `CardState::match_began`): the player went back to the game, so
	/// whatever was opened from outside the window and stands untouched ends, nothing written (DESK-PUMP's
	/// `outside_recalled`: the Start a pump window, a start waiting for the place, a pump being composed, the pump at
	/// Ready with nothing done) but a match pump's moves, which bank (since 2026-10-27).
	fn game_started(&mut self, started: bool) {
		let in_match = self.games.as_ref().is_some_and(|g| g.watcher.in_match());
		let began = self.offering.card.match_began(in_match);
		if (started || began)
			&& let Some(ended) = self.outside_recalled()
		{
			ended_log(ended, "pump", "a game started"); // copy-lint: not shown (a log line)
		}
	}

	/// A performed move on any device newer than the standing offer (`last_move_at`): a pump done another way answered
	/// it, so it ends, withdrawn on the devices when it went out (silently while held or when it never did), and its
	/// card closes. An offer waiting for a move that runs in the pump on this computer counts moves only from when that
	/// pump went idle (`CardState::answered_from`), so the pump it waited for never answers it.
	fn offer_done_since(&mut self, now: i64) {
		let Some((id, at)) = self.standing.current().map(|o| (o.id.clone(), o.at)) else { return };
		let Some(from) = self.offering.card.answered_from(&id, at) else { return };
		if self.last_move_at().is_none_or(|done| done <= from) {
			return;
		}
		self.offer_ends(&id, now);
		log::info!("offer {id}: a move was done since, it ends");
	}

	/// The offer that waited for a move running in the pump on this computer (made, or its game closed, meanwhile:
	/// `CardState::wait_pump`), at the first look that finds that pump idle (`desk_pump_mid_move` no longer): moves
	/// answer it from now on, and while it still stands "Open the pump after each match" opens it in the idle pump's
	/// place (`open_offer`).
	fn offer_after_pump(&mut self, now: i64) {
		let Some(id) = self.offering.card.waiting_pump().map(str::to_owned) else { return };
		if self.desk_pump_mid_move() {
			return;
		}
		self.offering.card.pump_idle(now);
		let Some(game) = self.standing.current().filter(|o| o.id == id).map(|o| o.game.clone()) else { return };
		log::info!("offer {id}: the pump here is idle");
		self.open_offer(&game, now);
	}

	/// The offer `id` ends on the computer: its `match_end` taken off the link, the standing offer taken (its id withdrawn
	/// on the devices when it went out, `Standing::take`), its card closed. Whether a `match_end` still waited on the
	/// link. Callers: a device's or this computer's pump from it (`offer_taken`), a move done since (`offer_done_since`).
	pub(crate) fn offer_ends(&mut self, id: &str, now: i64) -> bool {
		let Some(t) = self.games.as_ref().map(crate::games::Games::tunables) else { return false };
		let queued = self.queue.withdraw(id);
		let standing = self.standing.take(id, now, t);
		if standing || self.offering.card.shows(id) {
			self.close_card();
		}
		queued
	}

	/// The standing offer goes out to the devices (its `match_end` queued for the device that holds the link) at the
	/// first look after the card's chance where the computer cannot ask (slice 48's double-check): it may go as far as
	/// it tells (`Standing::due_out`: no hold, not out yet, its card's time holding with no Start or Later on it),
	/// nothing holds the offers back (`offers_held`: the felt-unwell lock, a World of Warcraft client with QuestPump, a
	/// pump on this computer, a watched game in a match), the computer's pause does not hold it, and its card could not
	/// show this look (`CardState::waits`: the game in front on its one screen) or has shown to no input for
	/// `match_away_s`: the system's own idle time, whatever game runs, and the card's own time on screen since it first
	/// showed (`CardState::shown_for`) each at least that long, so an input idle before the card came counts only from
	/// its show (slice 48's second fix round). Once its card's time passed or the player answered it here, it never
	/// goes.
	fn out_look(&mut self, now: i64, t: &Tunables) {
		let Some(offer) = self.standing.due_out(now) else { return };
		let o = &self.settings.offers;
		let paused_runs = self.games.as_ref().is_some_and(|g| g.paused_runs(&o.pause));
		let card = &self.offering.card;
		let away = card.shown_for(&offer.id, now).is_some_and(|s| s >= t.away_s)
			&& self.sitting.idle_s().is_some_and(|s| i64::from(s) >= t.away_s);
		if !(card.waits(&offer.id) || away) || o.pause.holds(now, &offer.game, paused_runs) || self.offers_held(now) {
			return;
		}
		let Some(offer) = self.standing.send(now).cloned() else { return };
		log::info!("offer {}: out to the devices", offer.id);
		self.queue_match_end(&offer);
	}

	/// The offer's `match_end` for the device that holds the link, a newer one replacing one still waiting there; its
	/// line on the socket as every queued command's.
	fn queue_match_end(&self, offer: &Offer) {
		let command = Command::MatchEnd {
			offer: offer.id.clone(),
			link: offer.link.to_owned(),
			name: offer.name.clone(),
			kind: Some(MATCH.to_owned()),
			ended_at: offer.ended_at,
			tier: Some(offer.tier),
			minutes: Some(offer.minutes),
			warm: offer.warm,
			arms: offer.arms,
		};
		if let Some(queued) = self.queue.queue(command, lan::now_ms()) {
			self.emit(&Line::Command { command: queued });
		} else {
			log::warn!("offer {}: no match_end queued (contract/library.json names no match_offer_min)", offer.id);
		}
	}

	/// A sync's part of the offers: the device's latest game offer (unix seconds on its own clock, at most a day ahead:
	/// `lan.rs` drops one further) folded into the latest the helper knows as the device told it (one still ahead
	/// starts the sitting clock only once this computer's clock reaches it, `sitting::Sitting::sat_s`, and
	/// settings.json is written only when it moves), and the offers expired by `now`; then the offer facts every
	/// response carries as it leaves, told to the routes (`tell_offers`).
	pub(super) fn offer_heard(&mut self, heard: Option<i64>, now: i64) {
		if let Some(at) = heard.filter(|&at| at > 0)
			&& self.settings.offers.last_game_offer_at.is_none_or(|last| at > last)
		{
			self.settings.offers.last_game_offer_at = Some(at);
			self.save_settings();
		}
		if let Some(t) = self.games.as_ref().map(crate::games::Games::tunables) {
			self.standing.expire(now, t);
		}
		self.tell_offers(now);
	}

	/// The offer facts every sync response carries as it leaves (`lan::OfferFacts`; slice 48's second fix round), told
	/// to the routes at the end of each of the loop's turns and as it takes a sync: the felt-unwell lock's end while it
	/// lies ahead, the ids taken back whose `match_end` has not expired, whether a watched game is in a match (at most
	/// 3 hours of it), whether an aim game rests the arms now and the latest game offer. A change of the ids, `playing`
	/// or `arms` answers the waiting held and standing requests at once (`Lan::set_offers`).
	pub(super) fn tell_offers(&self, now: i64) {
		self.lan.set_offers(OfferFacts {
			locked_until: self.session.locked_until(now),
			offer_withdrawn: self.standing.withdrawn().map(str::to_owned).collect(),
			playing: self.games.as_ref().is_some_and(|g| g.watcher.playing(now * 1_000)),
			arms: self.arms_now(),
			game_offer_at: self.settings.offers.last_game_offer_at,
		});
	}
}
