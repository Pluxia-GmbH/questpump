//! The reminders on the computer (docs/ui.md, "The reminders on the computer"; docs/plan.md slice 35, "The reminders on
//! the computer", 2026-10-09): the phones' streak reminder and sitting cue (slice 32) in the desktop app, each shown on
//! the match-over card's window (`card.rs`), which never takes the focus, never as a system notification (no
//! permission to ask, and a game's full screen and Windows' Do Not Disturb hold toasts back).
//!
//! - **The streak reminder**, off until the player turns it on (the Reminders page, or Remind me in the offer below):
//!   once a game day at its time (`remind_streak_min`, 19:00, by default; a time already past when it is turned on or
//!   moved comes from the next game day), on a game day with no performed move yet on any device (the home `Board`'s),
//!   in the phones' words ("One pump today makes your streak 13 days.", in the game or here while Hero shows the game's
//!   sheet, and that one pump starts the streak where none counts). After `remind_taper_days` game days with no
//!   performed move it comes only every `remind_taper_every` game days, from `remind_weekly_days` once a week, and
//!   daily again from the next pump. Since 2026-10-27 it waits until QuestPump has run `remind_run_min` (an hour) in
//!   this run (Rapha: "shuold not appear immediately. i have it open to use it in a game and not to quick pump"): a
//!   start after its time no longer shows it at once, the first look past that hour does while today still has no
//!   pump, and its place under the cap is kept meanwhile. A quit starts the hour again, so a run shorter than an hour
//!   never shows it. Its card stands until the game day turns or the day is done.
//! - **The sitting cue**, off by default: once the sitting clock (`runtime/sitting.rs`, slice 48: the minutes since
//!   the latest of an input pause of `AWAY_S` at this computer, a performed move and a game offer on any device, and
//!   04:00 only where the input's idle time never reads) holds `sit_after_min`, an hour that starts outside quiet
//!   hours, at most one each `sit_after_min`, never in the place the streak reminder still to come today needs under
//!   the cap. Its card stands `sit_after_min`, or until a pump or a match's offer.
//! - Both wait while no felt-unwell lock holds, no World of Warcraft client with QuestPump asks the pumps, no pump runs
//!   here, no watched game is in a match (`offers_held`) and the offers are not paused, and keep their own quiet hours
//!   and daily cap (`OfferSettings::count`; the match offers shared them until slice 48); and the engine has folded, so
//!   the board is the player's. No reminder comes or shows while a match's offer stands (slice 48's double-check), held
//!   too and after its card's Later: a new offer ends a standing sitting cue (`reminder_sitting_ends`), and a standing
//!   streak reminder's card shows again once the offer no longer stands (`card.rs`, `card_due`). The card's Start opens
//!   the Quick pump at Ready (`pump.quick` from outside the window, which opens it), Later and × end the reminder for
//!   its game day.
//! - **The offer**: "Remind me once a day?" once, as the first Result here that ended with its reveal is put away
//!   (`pump_continue`), shown outside the first five minutes (at their end for the welcome's first pump), never while
//!   the felt-unwell lock holds or a pump is on screen or being made, nor once the streak reminder is on.
//! - **The tray's streak line**, shown only: the days one pump today makes in the streak reminder's words, Today: done
//!   with the streak once today has a pump, the resting line while felt unwell rests the pumps; from the board Pump home
//!   draws (`reminders_board`), just above the pump's items.
//!
//! The `menu.reminders` topic is `{streak_on, streak_at, streak_quiet, sitting_on, quiet_from, quiet_to, cap, cap_max,
//! offer}` (the times in minutes after local midnight; `streak_quiet` when the streak reminder's time lies in quiet
//! hours, so it never comes; `offer` while the offer is due). Intents: `reminders.streak {on}`, `reminders.time {at}`,
//! `reminders.sitting {on}`, `reminders.quiet {from, to}` (minutes after local midnight), `reminders.cap {cap}` (1 to
//! `remind_cap_max`) and `reminders.offered` (the offer showed, whatever the answer).

use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

use questpump_core::link::Sheet;
use questpump_core::offers::{self, Tunables};
use questpump_core::session::Zone;
use serde::Serialize;
use serde_json::{Value, json};
use tauri::menu::MenuItem;
use tauri::{AppHandle, Manager as _, Wry};

use super::card::CardView;
use super::facts::facts;
use super::home::Board;
use super::tray::TrayLines;
use super::{Desk, Intent};
use crate::copy;
use crate::games::Games;
use crate::lan;
use crate::runtime::{Helper, local_offset_at, minute_of_day};

/// The Reminders page's topic.
pub(crate) const REMINDERS: &str = "menu.reminders";
/// The tray's streak line's menu id.
const TRAY_STREAK: &str = "reminders.streak";
const DAY_S: i64 = 86_400;
const WEEK_DAYS: i64 = 7;

/// What a reminder's card shows (`CardView::remind`).
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum RemindView {
	/// The streak reminder: the days one pump today makes (1: it starts the streak), and whether Hero shows the game's
	/// sheet (in the game or here).
	Streak { days: i64, game: bool },
	/// The sitting cue.
	Sitting,
}

/// A reminder whose card stands: what it shows, the game day it came on (`GameDay.index`), when it came (unix
/// seconds), and for the sitting cue until when it stands.
#[derive(Debug, Clone)]
pub(crate) struct Remind {
	view: CardView,
	day: i64,
	at: i64,
	until: Option<i64>,
}

/// The board Pump home last drew, whether Hero shows the game's sheet, and whether felt unwell rests the pumps now.
#[derive(Debug, Clone)]
struct Seen {
	board: Board,
	game: bool,
	resting: bool,
}

/// The reminders' state on the loop (a field of the runtime's `Helper`).
#[derive(Debug)]
pub(crate) struct Reminding {
	/// When this run began (`Reminding::new`, as the helper is made): the streak reminder waits `remind_run_min`.
	began: Instant,
	/// The reminder whose card stands.
	card: Option<Remind>,
	seen: Option<Seen>,
	/// A Result here was put away: the offer is due.
	offer_due: bool,
	/// The tray's streak line as last drawn.
	tray: Option<String>,
}

impl Reminding {
	/// This run's reminders, nothing standing yet, its hour counted from now.
	pub(crate) fn new() -> Self {
		Self { began: Instant::now(), card: None, seen: None, offer_due: false, tray: None }
	}

	/// QuestPump has run `remind_run_min` in this run: the streak reminder may show.
	fn run_long_enough(&self) -> bool {
		self.began.elapsed() >= Duration::from_secs(facts().constants().remind_run_min * 60)
	}
}

/// The Reminders page.
#[derive(Serialize)]
#[expect(clippy::struct_excessive_bools, reason = "the page's switches and marks, each a yes or no of its own")]
struct RemindersTopic {
	streak_on: bool,
	streak_at: u16,
	streak_quiet: bool,
	sitting_on: bool,
	quiet_from: u16,
	quiet_to: u16,
	cap: u32,
	cap_max: u32,
	offer: bool,
}

/// The tray's streak line, made on the main thread the first time it shows.
struct TrayStreak {
	item: MenuItem<Wry>,
	shown: AtomicBool,
}

/// The tray's streak line now: the resting line while felt unwell rests the pumps on a day with none, today's done
/// line, else the days one pump today makes in the streak reminder's words.
fn tray_line(seen: &Seen) -> String {
	let b = &seen.board;
	if seen.resting && !b.today {
		let line = if b.days > 0 { copy::pump_home_streak_resting() } else { copy::pump_home_streak_resting_none() };
		return line.to_owned();
	}
	if b.today {
		return copy::desk_tray_streak_done(b.days);
	}
	match (b.after_pump > 1, seen.game) {
		(false, false) => copy::reminders_notify_streak_start().to_owned(),
		(false, true) => copy::reminders_notify_streak_start_game().to_owned(),
		(true, false) => copy::reminders_notify_streak_body(b.after_pump),
		(true, true) => copy::reminders_notify_streak_body_game(b.after_pump),
	}
}

/// Draws the tray's streak line (on the main thread), just above the pump's items.
fn draw_tray(app: &AppHandle, line: &str) {
	let Some(lines) = app.try_state::<TrayLines>() else { return };
	if app.try_state::<TrayStreak>().is_none() {
		match MenuItem::with_id(app, TRAY_STREAK, line, false, None::<&str>) {
			Ok(item) => {
				app.manage(TrayStreak { item, shown: AtomicBool::new(false) });
			}
			Err(e) => {
				log::warn!("reminders: the tray's line: {e}");
				return;
			}
		}
	}
	let Some(tray) = app.try_state::<TrayStreak>() else { return };
	let _ = tray.item.set_text(line);
	lines.show(&tray.item, &tray.shown, lines.above_pumps(), true);
}

/// Whether the streak reminder comes on a game day `missed` full game days after the last performed move (the taper
/// after an absence): daily until `remind_taper_days`, then every `remind_taper_every`, from `remind_weekly_days` once a
/// week.
fn tapered_day(missed: i64) -> bool {
	let c = facts().constants();
	if missed < c.remind_taper_days {
		return true;
	}
	if missed < c.remind_weekly_days {
		return (missed - c.remind_taper_days) % c.remind_taper_every.max(1) == 0;
	}
	(missed - c.remind_weekly_days) % WEEK_DAYS == 0
}

impl Helper {
	/// The offers' numbers, which hold the reminders' defaults (quiet hours, the cap); none when they did not build in,
	/// and then no reminder comes.
	fn remind_tunables(&self) -> Option<&'static Tunables> {
		self.games.as_ref().map(Games::tunables)
	}

	/// The board Pump home just drew (`home_publish`), for the reminders and the tray's streak line; whether Hero shows
	/// the game's sheet is asked only while today has no pump, the only time their words name it.
	pub(crate) fn reminders_board(&mut self, sheet: &Sheet, board: Board) {
		let game = !board.today && self.game_shown(sheet).is_some();
		let seen = Seen { board, game, resting: sheet.locked_until.is_some() };
		let line = tray_line(&seen);
		self.reminding.seen = Some(seen);
		if self.reminding.tray.as_ref() == Some(&line) {
			return;
		}
		self.reminding.tray = Some(line.clone());
		let handle = self.app.clone();
		if let Err(e) = self.app.run_on_main_thread(move || draw_tray(&handle, &line)) {
			log::warn!("reminders: the tray's line: {e}");
		}
	}

	/// The loop's look (every 2 s, after the sitting clock's and before the card's): a standing reminder's end, a
	/// reminder due, and the page's topic.
	pub(crate) fn reminders_look(&mut self) {
		let now = lan::now_ms().div_euclid(1_000);
		self.reminder_over(now);
		if self.reminding.card.is_none()
			&& let Some(t) = self.remind_tunables()
		{
			self.reminder_due(now, t);
		}
		self.publish_reminders();
	}

	/// The standing reminder's card ends: the streak reminder's when its game day turned or has a pump, the sitting cue's
	/// after its time or a performed move since it came.
	fn reminder_over(&mut self, now: i64) {
		let Some(card) = &self.reminding.card else { return };
		let over = match card.view.remind {
			Some(RemindView::Streak { .. }) => {
				self.home.days.day.index(now) != card.day || self.reminding.seen.as_ref().is_none_or(|s| s.board.today)
			}
			_ => card.until.is_none_or(|until| now >= until) || self.last_move_at().is_some_and(|at| at >= card.at),
		};
		if over {
			self.reminding.card = None;
		}
	}

	/// The streak reminder's time on the local date of `now`, else the day before where that is still to come (unix
	/// seconds): the one of the current game day when its game day is today's.
	fn streak_time(&self, now: i64) -> i64 {
		let at = self.settings.reminders.streak_at.unwrap_or(facts().constants().remind_streak_min);
		let today = now - (now + local_offset_at(now)).rem_euclid(DAY_S) + i64::from(at) * 60;
		if today > now { today - DAY_S } else { today }
	}

	/// The streak reminder's time today has passed already (it was turned on or moved after it): it comes from the next
	/// game day, as the phones plan only the times still to come.
	fn streak_armed(&mut self) {
		let now = lan::now_ms().div_euclid(1_000);
		let day = &self.home.days.day;
		let today = day.index(now);
		if day.index(self.streak_time(now)) == today {
			self.settings.reminders.streak_day = Some(today);
		}
	}

	/// A reminder due at `now` stands, counted under the cap: the streak reminder first, then the sitting cue.
	fn reminder_due(&mut self, now: i64, t: &Tunables) {
		let start = Zone::new(local_offset_at).game_day_start(now);
		let o = &self.settings.offers;
		let paused_runs = self.games.as_ref().is_some_and(|g| g.paused_runs(&o.pause));
		let quiet = |at: i64| offers::quiet(minute_of_day(at), o.quiet(t));
		let shown = o.shown_on(start);
		if !self.engine_ready()
			|| self.standing.current().is_some()
			|| self.offers_held(now)
			|| o.pause.holds(now, "", paused_runs)
			|| quiet(now)
			|| shown >= o.cap(t)
		{
			return;
		}
		let Some(seen) = self.reminding.seen.clone() else { return };
		let r = &self.settings.reminders;
		let day = &self.home.days.day;
		let today = day.index(now);
		let at = self.streak_time(now);
		let streak_open = r.streak_on && !seen.board.today && r.streak_day != Some(today);
		let tapered = self.home.days.last_active().is_none_or(|last| tapered_day(today - last - 1));
		let due = streak_open && tapered && day.index(at) == today && !quiet(at);
		let ran = self.reminding.run_long_enough();
		if due && ran {
			let view = RemindView::Streak { days: seen.board.after_pump, game: seen.game };
			self.reminder_show(view, today, now, None, start);
			return;
		}
		// the place under the cap the streak reminder still to come today needs: later today, or once this run's hour
		// is up
		let later = at + DAY_S;
		let waiting =
			(due && !ran) || (streak_open && tapered && day.index(later) == today && later > now && !quiet(later));
		let hour = facts().constants().sit_after_min * 60;
		if !r.sitting_on
			|| shown + u32::from(waiting) >= o.cap(t)
			|| r.sitting_at.is_some_and(|last| now - last < hour)
			|| self.sat_s(now) < hour
			|| quiet(now - hour)
		{
			return;
		}
		self.reminder_show(RemindView::Sitting, today, now, Some(now + hour), start);
	}

	/// A reminder's card stands from now, counted under the cap on the game day starting at `start`.
	fn reminder_show(&mut self, view: RemindView, day: i64, now: i64, until: Option<i64>, start: i64) {
		let id = match view {
			RemindView::Streak { .. } => {
				self.settings.reminders.streak_day = Some(day);
				format!("r-streak-{day}")
			}
			RemindView::Sitting => {
				self.settings.reminders.sitting_at = Some(now);
				format!("r-sitting-{now}")
			}
		};
		self.settings.offers.count(start);
		self.save_settings();
		log::info!("reminder {id}: stands");
		self.counts_feature("reminders");
		let view = CardView { id, name: String::new(), tier: None, one_size: false, test: false, remind: Some(view) };
		self.reminding.card = Some(Remind { view, day, at: now, until });
	}

	/// The reminder's card that stands (`card.rs`), whose window shows it while no match offer stands.
	pub(crate) fn reminder_view(&self) -> Option<CardView> {
		self.reminding.card.as_ref().map(|r| r.view.clone())
	}

	/// The reminder card's Start: the Quick pump at Ready, asked from outside the window; the reminder has done its part.
	pub(crate) fn reminder_start(&mut self) -> Result<(), String> {
		self.intent(&Intent::from_value(json!({"type": "pump.quick", "outside": true}))?)?;
		self.reminder_later();
		Ok(())
	}

	/// A match's offer stands now (`runtime/matches.rs`): a standing sitting cue ends, since the offer asks for the same
	/// pump; a standing streak reminder stays, its card back once the offer no longer stands.
	pub(crate) fn reminder_sitting_ends(&mut self) {
		if let Some(card) = self.reminding.card.take_if(|r| matches!(r.view.remind, Some(RemindView::Sitting))) {
			log::info!("reminder {}: ended by a match's offer", card.view.id);
		}
	}

	/// The reminder card's Later or ×: the reminder ends for its game day (its window goes with `close_card`).
	pub(crate) fn reminder_later(&mut self) {
		if let Some(card) = self.reminding.card.take() {
			log::info!("reminder {}: ended", card.view.id);
		}
	}

	/// The Result of a pump here was put away: "Remind me once a day?" is due, until it shows once.
	pub(crate) fn reminder_offer_due(&mut self) {
		let r = &self.settings.reminders;
		if !r.offered && !r.streak_on {
			self.reminding.offer_due = true;
			self.publish_reminders();
		}
	}

	/// The Reminders page's topic.
	pub(crate) fn publish_reminders(&self) {
		let Some(t) = self.remind_tunables() else { return };
		let (r, o) = (&self.settings.reminders, &self.settings.offers);
		let streak_at = r.streak_at.unwrap_or(facts().constants().remind_streak_min);
		let now = lan::now_ms().div_euclid(1_000);
		let offer = self.reminding.offer_due
			&& !r.offered
			&& !r.streak_on
			&& self.settings.desk.onboarded
			&& self.session.locked_until(now).is_none()
			&& !self.desk_pump_running()
			&& !self.desk_pump_starting();
		self.app.state::<Desk>().publisher.publish(
			REMINDERS,
			&RemindersTopic {
				streak_on: r.streak_on,
				streak_at,
				streak_quiet: offers::quiet(streak_at, o.quiet(t)),
				sitting_on: r.sitting_on,
				quiet_from: o.quiet(t).0,
				quiet_to: o.quiet(t).1,
				cap: o.cap(t),
				cap_max: t.cap_max,
				offer,
			},
		);
	}

	/// The Reminders page's intents and the offer's.
	pub(crate) fn reminders_intent(&mut self, intent: &Intent) -> Result<(), String> {
		let on = || intent.arg("on").and_then(Value::as_bool).ok_or_else(|| format!("{} takes on", intent.kind()));
		match intent.kind() {
			"reminders.streak" => {
				let on = on()?;
				self.settings.reminders.streak_on = on;
				if on {
					self.settings.reminders.offered = true;
					self.streak_armed();
				}
				log::info!("reminders: the streak reminder {}", if on { "on" } else { "off" });
			}
			"reminders.time" => {
				let at = intent
					.arg("at")
					.and_then(Value::as_u64)
					.and_then(|m| u16::try_from(m).ok())
					.filter(|&m| i64::from(m) < DAY_S / 60)
					.ok_or("reminders.time takes at, minutes after midnight")?;
				self.settings.reminders.streak_at = Some(at);
				self.streak_armed();
			}
			"reminders.sitting" => {
				let on = on()?;
				self.settings.reminders.sitting_on = on;
				log::info!("reminders: the sitting cue {}", if on { "on" } else { "off" });
			}
			"reminders.quiet" => {
				let minute = |name| {
					let m = intent.arg(name).and_then(Value::as_u64).and_then(|m| u16::try_from(m).ok());
					m.filter(|&m| i64::from(m) < DAY_S / 60)
				};
				let (Some(from), Some(to)) = (minute("from"), minute("to")) else {
					return Err("reminders.quiet takes from and to".to_owned());
				};
				self.settings.offers.quiet_from_min = Some(from);
				self.settings.offers.quiet_to_min = Some(to);
			}
			"reminders.cap" => {
				let most = self.remind_tunables().ok_or("no reminders on this computer")?.cap_max;
				let cap = intent.arg("cap").and_then(Value::as_u64).and_then(|c| u32::try_from(c).ok());
				let cap = cap.filter(|c| (1..=most).contains(c));
				self.settings.offers.cap = Some(cap.ok_or("reminders.cap takes a cap from 1 to remind_cap_max")?);
			}
			"reminders.offered" => {
				self.settings.reminders.offered = true;
				self.reminding.offer_due = false;
			}
			_ => return Err(intent.unknown()),
		}
		self.save_settings();
		self.publish_reminders();
		Ok(())
	}
}
