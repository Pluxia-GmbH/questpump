//! The offer after a game on the computer (docs/plan.md slice 35, "The offer after a match", and slices 48, 57 and 59;
//! docs/ui.md, "The match-over card"; docs/computer-games.md §3-§4): one offer per finish, id `m-<uuid v4>`. `decide`
//! lets a finish offer only while no felt-unwell lock holds, no World of Warcraft client with QuestPump runs, the
//! computer's pause does not hold and the finish is at most `link_offer_max_min` old (a pump on the computer held one
//! back until 2026-10-28: its card now waits for that pump, and its auto-open takes an idle pump's place): since
//! slice 57 every match asks, as every turn-in in the game does, and a match's floor spaces the offers (the watcher's,
//! `games::Watcher`: `match_min_s` of a run before its close, and of a feed's match from its start); a match's offer
//! keeps no quiet hours, no daily cap and no gap (the reminders keep their quiet hours and cap, `remind_quiet_*` and
//! `remind_cap`, which the numbers here carry for them). The offer carries the size the match's own minutes give
//! (`sitting::tier`, one step per `match_step_min`), those minutes (from the match's start, League of Legends' since
//! slice 59 from `match_lead_league_min` before its game process started, for its champion select, though never from
//! before League's previous close or the latest move, `Tunables::league_lead_s`; and from the player's latest return to
//! the computer when that came later, `Gate::back_at`), whether the player had sat `match_sat_min` as it was
//! made (the pump's warm first move, `Tunables::warm`, on the sitting clock without the offers, `Gate::sat_s`; the
//! clock gates and sizes nothing) and the arms rest. `Standing` keeps the offer that stands: its card for
//! `match_offer_min`, Pump home's, the tray's and the Games tab's Now card's for `link_offer_max_min`, unless taken or
//! replaced; one made at a game's close (every watched game's, slice 56) is held on the computer for `match_relaunch_s`
//! after the close, so a relaunch in that time (a reconnect) takes it back unheard; one at a match's end a game's feed
//! told shows its card and opens its pump only `LOOK_S` later, the player still on the game's last screen (Rapha,
//! 2026-11-03). The computer asks first (slice 48's
//! double-check): an offer goes out to the devices only when the computer cannot ask, at a look after its hold while
//! its card's time holds and nothing answered it, where the loop finds the card could not show or the input idle
//! `match_away_s` while it shows (`Standing::due_out`, the rest of the rule in `runtime/matches.rs`), and never once
//! its card's time passed or the player answered it here. Each offer that went out and was taken, put off with the
//! card's Later or ×, or ended by a move done since keeps its id for the devices to take theirs back
//! (`offer_withdrawn`) until its `match_end` would have expired. The numbers come from contract/library.json's
//! constants, which `tools/gen-engine.lua` writes from `Constants.lua`.

use std::fmt::Write as _;
use std::sync::LazyLock;

use serde::{Deserialize, Serialize};

use crate::games::{Finish, Watched};
use crate::{library, sitting};

/// The test offer's id: "Send a test offer" shows the card with it, never queues a `match_end` and never counts.
pub const TEST: &str = "m-test";

/// The longest name an offer shows and its `match_end` carries, in characters (Unicode scalars, as
/// companion-api.yaml's `maxLength` counts them): a picked program's own name, on Windows its window's title, can be
/// longer.
pub const NAME_CHARS: usize = 40;

/// The seconds a match's end a game's feed told leaves the player on the game's last screen (the board, the
/// scoreboard) before its card shows and its pump opens (Rapha, 2026-11-03, of Mechabellum: "i still need like a sec to
/// look at the board"); an offer at a game's close has nothing left to look at and shows at once.
pub const LOOK_S: i64 = 3;

/// The offers', the watcher's and the reminders' numbers, in seconds unless named in minutes.
#[derive(Debug, Clone)]
pub struct Tunables {
	/// A finish older than this offers nothing, and an offer stands this long (`link_offer_max_min`).
	pub max_age_s: i64,
	/// The card's life and the `match_end` command's (`match_offer_min`).
	pub offer_s: i64,
	/// The least any game but League ran, from its first start, before its close offers (`match_min_s`).
	pub min_run_s: i64,
	/// The least League of Legends' game process ran before its close offers (`match_min_league_s`: a remake and the
	/// loading screen stay under it), its own minutes alone, never `league_lead_s`.
	pub league_min_s: i64,
	/// The least a checkpoint game's feed finish waits from where its match began, the game's start or the last that
	/// offered (`match_checkpoint_min`, 2026-10-28: a mission or a map is no match), in seconds.
	pub checkpoint_s: i64,
	/// The lead by which a League of Legends offer at its game process's close counts the match from before the process
	/// started (`match_lead_league_min`, slice 59): champion select, the ready check and the hand-off to the loading
	/// screen, which run in the client the app never hears. Never from before League's previous close
	/// (`Finish::previous_close`), the latest move (`Gate::moved_at`) or the latest return (`Gate::back_at`), and never
	/// into the process's own minutes; no other game has a lead (a fed game's match holds its pick phase).
	pub league_lead_s: i64,
	/// A relaunch within this time continues a game's run and takes its close's held offer back (`match_relaunch_s`).
	pub relaunch_s: i64,
	/// The input idle time while a card shows after which its offer goes out to the devices (`match_away_s`).
	pub away_s: i64,
	/// The minutes sat at the computer from which a match's pump opens on its warm move (`match_sat_min`), and the
	/// minutes of a match one size step takes (`match_step_min`).
	pub sat_min: i64,
	pub step_min: i64,
	/// The reminders' own a game day by default, and the most a player sets (`remind_cap`, `remind_cap_max`).
	pub cap: u32,
	pub cap_max: u32,
	/// The reminders' quiet hours by default, minutes after local midnight (`remind_quiet_from_min`,
	/// `remind_quiet_to_min`).
	pub quiet: (u16, u16),
	/// The pause's "an hour" and "four hours" (`pause_hour_s`, `pause_four_hours_s`).
	pub hour_s: i64,
	pub four_hours_s: i64,
}

static TUNABLES: LazyLock<Option<Tunables>> = LazyLock::new(|| {
	let c = library::constants()?;
	Some(Tunables {
		max_age_s: c.link_offer_max_min * 60,
		offer_s: c.match_offer_min * 60,
		min_run_s: c.match_min_s,
		league_min_s: c.match_min_league_s,
		checkpoint_s: c.match_checkpoint_min * 60,
		league_lead_s: c.match_lead_league_min * 60,
		relaunch_s: c.match_relaunch_s,
		away_s: c.match_away_s,
		sat_min: c.match_sat_min,
		step_min: c.match_step_min,
		cap: c.remind_cap,
		cap_max: c.remind_cap_max,
		quiet: (c.remind_quiet_from_min, c.remind_quiet_to_min),
		hour_s: c.pause_hour_s,
		four_hours_s: c.pause_four_hours_s,
	})
});

/// The numbers, built in from contract/library.json; none when its constants do not read (`make check` keeps them
/// readable), and then no game offers.
#[must_use]
pub fn tunables() -> Option<&'static Tunables> {
	TUNABLES.as_ref()
}

impl Tunables {
	/// Whether a match's pump after `sat_s` seconds seated (the sitting clock without the offers, which stand nobody up:
	/// `Gate::sat_s`) opens on its warm move: they reach `match_sat_min` (hazard 2, sudden exertion after a long sit;
	/// slice 57: the clock's one say in a match's pump).
	#[must_use]
	pub fn warm(&self, sat_s: i64) -> bool {
		i64::from(sitting::minutes(sat_s)) >= self.sat_min
	}
}

/// The computer's pause of its offers (the tray and the Games tab's Pause offers).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Pause {
	#[default]
	None,
	/// An hour, four hours, until 04:00: until this unix second.
	Until {
		at: i64,
	},
	/// While this game runs (`Watched::id`): its own ends too, and every other game's while it runs.
	WhileRunning {
		game: String,
	},
	UntilResumed,
}

impl Pause {
	/// Whether it holds back a finish of `finishing` (`Watched::id`) at `now`, `paused_runs` saying whether the game a
	/// `WhileRunning` names runs.
	#[must_use]
	pub fn holds(&self, now: i64, finishing: &str, paused_runs: bool) -> bool {
		match self {
			Self::None => false,
			Self::Until { at } => now < *at,
			Self::WhileRunning { game } => paused_runs || game == finishing,
			Self::UntilResumed => true,
		}
	}

	/// Whether it is over: its time passed, or its game no longer runs.
	#[must_use]
	pub fn over(&self, now: i64, paused_runs: bool) -> bool {
		match self {
			Self::Until { at } => now >= *at,
			Self::WhileRunning { .. } => !paused_runs,
			Self::None | Self::UntilResumed => false,
		}
	}
}

/// Whether `minute` (after local midnight) lies in quiet hours `from`-`to`: none where both are equal, and a span across
/// midnight wraps (the phones' `Reminders.quiet`). The reminders' alone since slice 48.
#[must_use]
pub fn quiet(minute: u16, (from, to): (u16, u16)) -> bool {
	if from == to {
		return false;
	}
	if from < to {
		return from <= minute && minute < to;
	}
	minute >= from || minute < to
}

/// Everything a finish's offer depends on at the moment it is decided.
#[derive(Debug, Clone)]
pub struct Gate<'a> {
	/// Unix seconds.
	pub now: i64,
	/// A World of Warcraft client whose AddOns hold QuestPump runs: the game asks the pumps then.
	pub wow: bool,
	/// The felt-unwell lock's end (`Session::locked_until`).
	pub locked_until: Option<i64>,
	pub pause: &'a Pause,
	/// Whether the game a `Pause::WhileRunning` names runs.
	pub paused_runs: bool,
	/// The seconds the player has sat at `now`, on the sitting clock without the offers (`sitting::Sitting::sat_s`
	/// with no offer: since the newest performed move on any device or the end of the latest pause in this computer's
	/// input), so an offer let pass or put off stands nobody up: the offer's warm move, never whether it comes or its
	/// size.
	pub sat_s: i64,
	/// The newest performed move on any device, unix seconds (the app's `Helper::last_move_at`): the time a pump
	/// already paid, from before which League's lead counts no minutes (`Tunables::league_lead_s`). None before any.
	pub moved_at: Option<i64>,
	/// The end of the latest pause in this computer's input, unix seconds (`sitting::Sitting::kept`: a sleep, an input
	/// pause of `sitting::AWAY_S`, the helper's first look in this run): no match counts minutes from before it, so a
	/// game left open overnight or through a day away offers at its close the minutes since the player came back (a
	/// match played counts as input, `sitting::Sitting::look`, and is never cut). None before the first look.
	pub back_at: Option<i64>,
	/// The arms rest holds for the finishing game: contract/links.json marks it `aim`, its switch in the game's window
	/// is on, and its process still runs.
	pub arms: bool,
}

/// Why a finish offered nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Dropped {
	#[error("the felt-unwell lock")]
	Locked,
	#[error("a WoW client runs")]
	Wow,
	#[error("offers paused")]
	Paused,
	#[error("older than link_offer_max_min")]
	Old,
}

/// A finish's offer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Offer {
	/// `m-<uuid v4>`.
	pub id: String,
	/// links.json's key and the name the offer shows; every offer's kind is a match (`games::MATCH`).
	pub link: &'static str,
	pub name: String,
	/// What tells its game from every other (`Watched::id`: a named game's key, a found or picked program's own).
	pub game: String,
	/// When the game ended, unix ms.
	pub ended_at: i64,
	/// When it was offered, unix seconds.
	pub at: i64,
	/// The pump's size by the match's own minutes (`sitting::tier`: 1 Light, 2 Balanced, 3 Extreme), which the card
	/// shows chosen, and those whole minutes, from the match's start (`Finish::began_at`; League's up to its lead before it,
	/// `Tunables::league_lead_s`; or the player's return to the computer when later, `Gate::back_at`) to its end
	/// (`Finish::ended_at`), which the pump's cardio takes as its active minutes; the `match_end` carries both, never a
	/// log line or a count.
	pub tier: u8,
	pub minutes: u16,
	/// The player had sat `match_sat_min` as the offer was made, on the sitting clock without the offers
	/// (`Tunables::warm` of `Gate::sat_s`): its pump opens on its warm move.
	pub warm: bool,
	/// The arms rest (`Gate::arms`), for the whole life of the offer on a device; the computer's own pump also asks
	/// whether the game still runs as it starts.
	pub arms: bool,
	/// While an offer made at a game's close is held on the computer (any watched game's, a fed game's whose match ran
	/// as it closed among them: `Finish::at_close`): until this unix second (`match_relaunch_s` after the close), in
	/// which a start again takes it back (`Standing::recall`) and after which the sitting clock starts again from it
	/// (`Standing::hold_ends`); none for a match's end the game's feed told, whose clock starts again at the decision,
	/// and once the hold ended.
	pub held_until: Option<i64>,
	/// While the player still looks at the game's last screen after a match's end its feed told (`LOOK_S` after the
	/// decision, unix seconds): no card shows and "Open the pump after each match" waits until then
	/// (`Standing::look_ends`); none for an offer at a game's close, and once the look ended.
	pub look_until: Option<i64>,
	/// When it went out to the devices as a `match_end` (unix seconds), whose life on a device runs `match_offer_min`
	/// from then: at the first look after its hold where the computer cannot ask (`Standing::due_out`); none until then,
	/// and for good once its card's time passed or the player answered it on the computer.
	pub out_at: Option<i64>,
}

/// An offer's id from 16 random bytes: `m-` and a version-4 UUID (RFC 9562 §5.4: version 4 in the 7th byte's high
/// nibble, variant 10 in the 9th byte's top bits).
#[must_use]
pub fn offer_id(mut random: [u8; 16]) -> String {
	random[6] = (random[6] & 0x0f) | 0x40;
	random[8] = (random[8] & 0x3f) | 0x80;
	let hex = random.iter().fold(String::with_capacity(32), |mut hex, b| {
		let _ = write!(hex, "{b:02x}");
		hex
	});
	format!("m-{}-{}-{}-{}-{}", &hex[..8], &hex[8..12], &hex[12..16], &hex[16..20], &hex[20..])
}

/// A game's or a program's name as an offer shows it: cut to `NAME_CHARS` characters on a character boundary, the
/// spaces the cut leaves at its end dropped.
#[must_use]
pub fn offer_name(name: &str) -> &str {
	let end = name.char_indices().nth(NAME_CHARS).map_or(name.len(), |(at, _)| at);
	name[..end].trim_end()
}

/// The offer a finish makes, or why it makes none; the first rule that holds it back names it.
///
/// # Errors
/// The reason no offer is made.
pub fn decide(finish: &Finish, gate: &Gate<'_>, t: &Tunables, random: [u8; 16]) -> Result<Offer, Dropped> {
	let w = &finish.watched;
	let rules = [
		(gate.locked_until.is_some_and(|end| gate.now < end), Dropped::Locked),
		(gate.wow, Dropped::Wow),
		(gate.pause.holds(gate.now, w.id(), gate.paused_runs), Dropped::Paused),
		(gate.now * 1_000 - finish.ended_at > t.max_age_s * 1_000, Dropped::Old),
	];
	if let Some(&(_, why)) = rules.iter().find(|(holds, _)| *holds) {
		return Err(why);
	}
	// the hold ends no sooner than the watcher's relaunch window (`games::Watcher`), in whole seconds rounded up, so
	// an offer a relaunch could still take back never goes out first
	let held_until = finish.at_close.then(|| (finish.ended_at + t.relaunch_s * 1_000 + 999).div_euclid(1_000));
	// League's champion select before its game process (slice 59), from no sooner than its previous close and the
	// latest move, and never into the process's own minutes
	let start = if *w == Watched::League && finish.at_close {
		let bounds = [finish.previous_close, gate.moved_at.map(|at| at * 1_000)];
		bounds.into_iter().flatten().fold(finish.began_at - t.league_lead_s * 1_000, i64::max).min(finish.began_at)
	} else {
		finish.began_at
	};
	let began_at = gate.back_at.map_or(start, |back| start.max(back * 1_000));
	let minutes = sitting::minutes((finish.ended_at - began_at).div_euclid(1_000));
	Ok(Offer {
		id: offer_id(random),
		link: w.link(),
		name: offer_name(w.name()).to_owned(),
		game: w.id().to_owned(),
		ended_at: finish.ended_at,
		at: gate.now,
		tier: sitting::tier(i64::from(minutes), t.step_min),
		minutes,
		warm: t.warm(gate.sat_s),
		arms: gate.arms,
		held_until,
		look_until: (!finish.at_close).then_some(gate.now + LOOK_S),
		out_at: None,
	})
}

/// The offer that stands on the computer, and the ids taken from it.
#[derive(Debug, Default)]
pub struct Standing {
	offer: Option<Offer>,
	/// The card shows until this unix second; Later and × end it sooner.
	card_until: i64,
	/// Taken offers' ids with the unix second their `match_end` would have expired.
	withdrawn: Vec<(String, i64)>,
}

impl Standing {
	/// A new offer stands in the old one's place, its card on.
	pub fn replace(&mut self, offer: Offer, t: &Tunables) {
		self.card_until = offer.at + t.offer_s;
		self.offer = Some(offer);
	}

	/// The offer that stands.
	#[must_use]
	pub fn current(&self) -> Option<&Offer> {
		self.offer.as_ref()
	}

	/// Whether its card's time holds at `now`.
	#[must_use]
	pub fn card(&self, now: i64) -> bool {
		self.offer.is_some() && now < self.card_until
	}

	/// The card closes (a Start); the offer still stands until its pump starts.
	pub fn close_card(&mut self) {
		self.card_until = 0;
	}

	/// The card's Later or ×: it closes, the offer still standing on Pump home and in the tray, and one that went out is
	/// withdrawn on every device (the player answered it here) until its `match_end` would have expired there. True
	/// when it was withdrawn now.
	pub fn later(&mut self, t: &Tunables) -> bool {
		self.card_until = 0;
		let Some((id, out)) = self.offer.as_ref().and_then(|o| Some((o.id.as_str(), o.out_at?))) else { return false };
		if self.withdrawn.iter().any(|(w, _)| w == id) {
			return false;
		}
		self.withdrawn.push((id.to_owned(), out + t.offer_s));
		true
	}

	/// A device or the computer started the pump offer `id` opened, or a move was done since it came: it no longer
	/// stands, and its id is withdrawn on every device until its `match_end` would have expired there (none while it was
	/// still held or when it never went out: no device heard of it). True when it was the offer that stands (the card
	/// closes).
	pub fn take(&mut self, id: &str, now: i64, t: &Tunables) -> bool {
		let taken = self.offer.take_if(|o| o.id == id);
		let standing = taken.is_some();
		// an id that no longer stands went out before (a device's own `offer_taken`)
		let out = taken.map_or(Some(now), |offer| offer.out_at);
		if let Some(out) = out
			&& id != TEST
			&& !self.withdrawn.iter().any(|(w, _)| w == id)
		{
			self.withdrawn.push((id.to_owned(), out + t.offer_s));
		}
		standing
	}

	/// The held offer whose hold ended by `now`, once: a start again no longer takes it back, and the sitting clock
	/// starts again from it.
	pub fn hold_ends(&mut self, now: i64) -> Option<&Offer> {
		let offer = self.offer.as_mut().filter(|o| o.held_until.is_some_and(|until| now >= until))?;
		offer.held_until = None;
		Some(offer)
	}

	/// The offer whose look at its game's last screen ended by `now`, once: its card may show and its pump open.
	pub fn look_ends(&mut self, now: i64) -> Option<&Offer> {
		let offer = self.offer.as_mut().filter(|o| o.look_until.is_some_and(|until| now >= until))?;
		offer.look_until = None;
		Some(offer)
	}

	/// The offer that may go out to the devices at `now`, as far as it alone tells: it stands, no hold or look keeps it,
	/// it has not gone out, and its card's time holds with no Start or Later on it (a device would show a match long
	/// over past it, and the player answered it here). The loop asks the rest of the rule: the gate again, no match, and
	/// the card that could not show or the input idle `match_away_s` while it shows.
	#[must_use]
	pub fn due_out(&self, now: i64) -> Option<&Offer> {
		self.offer
			.as_ref()
			.filter(|o| o.held_until.is_none() && o.look_until.is_none() && o.out_at.is_none() && now < self.card_until)
	}

	/// The offer goes out to the devices at `now` (its `out_at`), once.
	pub fn send(&mut self, now: i64) -> Option<&Offer> {
		let offer = self.offer.as_mut().filter(|o| o.out_at.is_none())?;
		offer.out_at = Some(now);
		Some(offer)
	}

	/// The game `game` (`Watched::id`) started again within `match_relaunch_s` of its close: its held offer no longer
	/// stands, unheard by any device, so nothing is withdrawn. The offer taken back, if one was held.
	pub fn recall(&mut self, game: &str) -> Option<Offer> {
		self.offer.take_if(|o| o.game == game && o.held_until.is_some())
	}

	/// What `now` ends: an offer `link_offer_max_min` old, withdrawn ids past their command's expiry.
	pub fn expire(&mut self, now: i64, t: &Tunables) {
		self.offer.take_if(|o| now >= o.at + t.max_age_s);
		self.withdrawn.retain(|&(_, until)| now < until);
	}

	/// The ids every sync response lists in `offer_withdrawn`.
	pub fn withdrawn(&self) -> impl Iterator<Item = &str> {
		self.withdrawn.iter().map(|(id, _)| id.as_str())
	}
}
