//! Work mode on the desk (docs/plan.md slice 67), shared by the desktop app and the browser: the switch and the
//! choices as the player's shared state holds them (`questpump_core::work::Work`, in the settings, merged with every
//! device's through the sync and the link), this device's work clock (`questpump_core::work::Clock`, at the loop's
//! look: `Desk::work_look`; a due block kept in the settings across a restart or a reload, `Settings::work_due`, which
//! asks again once the account's first pull answered), what holds it here, the ask and its two answers, Pump now (the
//! work pump, which banks as a match's does: `pump::Start::Work`) and Later (that pump composed with no screen and
//! banked: `Desk::pump_work_later`), and the anchor every pump moves.
//!
//! Work mode runs until the player stops it, every work pump alike (David, 2026-11-11, of the set of four pumps whose
//! dots the card filled: "Why are there 4 dots that get filled. Makes no sense. it should just run as long as I put
//! it").
//!
//! The clock always runs down, whatever the typing (David, 2026-11-10, whose block never came due while he watched his
//! work without touching the keys: "why is the clock stuck in work mode right now at 22? fix", then "it should just
//! always run down. no matter if there is typing?!"; a pause of `sit_away_s` in the input started the count over
//! until then): only a hold freezes it and only a move of the anchor starts it again. Whether somebody is there
//! (`Platform::idle_s` against `sit_away_s`) decides only when a due pump opens (`Desk::work_open`); a platform that
//! reads no input says none, and its due pump waits for no pause.
//!
//! The ask opens its pump itself (David, 2026-11-10: "It should work the same way as after a lol match ... go to the
//! foreground and start the exercise. No chosing wether to pump now or later"): while the ask stands a work pump comes
//! at its Ready view from outside the window, which opens in front for it (`Desk::work_open`), once for a due block,
//! as "Open the pump after each match" opens a match's once; its Bank all puts it off. It never opens while nobody is
//! there, waits for a pause in the player's keys, and one that opened itself and stands untouched once the player went
//! away closes until the return (`Desk::work_unattended`).
//!
//! The next block counts from the moment a work pump came on screen, whatever is then done with that pump (David,
//! 2026-11-10: "be sure that the new 25 minutes are counted right from the popup and not only after the excercises are
//! done"; 2026-11-11, who left one open past the next block and saw nothing come: "It does not matter that i dont do
//! the excercise right away. it should always continue with the 25 min"): its coming moves the anchor
//! (`Desk::work_pump_shown`) and it holds no clock (`Desk::work_hold`), so the ring counts on while it stands. When the
//! next block comes due and that pump still stands, it makes way once no move runs in it, as a pump makes way for a
//! game's (`Desk::pump_make_way`: its own moves left banked from a Ready view, its riders done paid), and the next
//! block's pump opens in its place. A Later and every other pump done move the anchor to now.
//!
//! The writes (contract/companion-api.yaml `Work`): a work pump's coming, a Later banked and any other pump done here
//! with a move saved each move the anchor; a work pump's end writes nothing more, but one the felt-unwell lock ended,
//! or one that closed untouched because the player went away, leaves its block due again (`Desk::work_due_again`), so
//! its pump comes again once the lock lifts or the player is back. A Later or a Pump now that answers an ask that
//! stands no more (`ask`: a page left open, an old notification) writes nothing; Later acts only while the block is
//! due.
//!
//! Work mode's own equipment (the gear and Floor moves, Room to move and Quiet mode, the office template's at first;
//! David, 2026-11-09: "just have an extra equipment setting for work") is no place: while work mode is on, every pump
//! composed here takes it in place of the place in use's (`Desk::work_setup`, read by `Desk::setup_in_force`), the
//! place in use and its pick staying as they are, and Your setup's switches and chips change it (`Desk::work_equip`).
//!
//! The topics: `work` (`WorkView`: Pump home's work card, the first Start's sheet and the tray's line, which the
//! platform also hears, `Platform::work_changed`) and `menu.work` (`MenuWork`: the Work mode page and the
//! How page's numbers). The intents: `work.start` (`{confirm?, every?, pace?, floor?, space?, quiet?, outside?}`: the
//! first opens the sheet, whose Start confirms), `work.stop`, `work.later` (`{ask?}`), `work.now` (`{ask?,
//! outside?}`), `work.set` (`{every?, pace?, floor?, space?, quiet?}`), `work.gear` (`{key, on}`: a piece of work
//! mode's equipment) and `work.close` (the sheet put away); `ask` is the ask's id the surface showed (`WorkView::ask`).

use questpump_core::body::PlaceSetup;
use questpump_core::offers;
use questpump_core::session::CatchupError;
use questpump_core::sitting;
use questpump_core::work::{self, Clock, Hold, Seen, State, Tunables, Work};
use serde::Serialize;
use serde_json::Value;

use crate::facts::facts;
use crate::places::with_gear;
use crate::platform::Platform;
use crate::pump::Start;
use crate::seams::StoreMark;
use crate::settings::{Settings, WorkDue};
use crate::{Desk, Intent};

/// The topics.
const TOPIC: &str = "work";
const MENU_TOPIC: &str = "menu.work";

/// The longest a due block kept across a restart waits for the account's first pull before it asks (`Desk::work_kept`),
/// the phones' wait for the account before a Later (`WorkMode.LATER_PULL_MS`).
const KEPT_WAIT_MS: i64 = 3_000;

/// The input pause after which a due pump opens itself (seconds, `Desk::work_open`): two of the loop's looks with no
/// key and no click, so the window that takes the front takes no key the player meant for another app.
const OPEN_PAUSE_S: i64 = 4;

/// Work mode's state on this device between looks.
#[derive(Debug, Default)]
pub struct WorkDesk {
	/// This device's work clock.
	clock: Clock,
	/// Its last look's state.
	state: Option<State>,
	/// The work pump on screen, from its Ready view to its end: it holds no clock, and the next block's pump takes its
	/// place (`work_open`).
	pump: Option<WorkPump>,
	/// The due block's pump opening itself (`work_open`).
	open: Opening,
	/// A Later's pump is being composed and banked, with the anchor of the ask it answers (it holds the clock as a pump
	/// does).
	later: Option<i64>,
	/// The last Later's refusal, by its words' key, until the next Start, Pump now, Later, work pump or new ask.
	refused: Option<&'static str>,
	/// The first Start's sheet is open.
	sheet: bool,
	/// The stores' mark the anchor was last looked for at, and the view last told the platform.
	mark: Option<StoreMark>,
	told: Option<WorkView>,
	/// The due block a restart or a reload cut off (`Settings::work_due`), until the clock takes it (`Desk::work_kept`),
	/// and the latest moment it waits to.
	kept: Option<WorkDue>,
	kept_until: Option<i64>,
	/// What held the clock at the last look: a change is said in the log.
	held: Option<Hold>,
	/// The last look while on (unix seconds): a gap of `sit_away_s` to the next is a time nobody was there (the computer
	/// slept), as the sitting clock reads one.
	looked: Option<i64>,
}

impl WorkDesk {
	/// Work mode on this device at a start: the due block the settings kept while on (`Settings::work_due`), which the
	/// clock takes once the account was heard (`Desk::work_kept`), so an ask a restart or a reload cut off comes again
	/// and its answers still stand.
	#[must_use]
	pub fn kept(settings: &Settings) -> Self {
		Self { kept: settings.work_due.filter(|_| settings.work.on), ..Self::default() }
	}
}

/// A due block's pump opening itself (`Desk::work_open`), by the block's anchor.
#[derive(Debug, Default)]
struct Opening {
	/// The block a work pump was started for (the one that opened itself, Pump home's or the tray's Pump now): none
	/// opens itself for it again.
	done: Option<i64>,
	/// The block whose opening waits for the player's keys to pause, and since when (unix seconds).
	waits: Option<(i64, i64)>,
	/// The block whose opening a gate refused, said once in the log.
	refused: Option<i64>,
	/// The next work pump to come on screen opened itself (`WorkPump::unasked`).
	unasked: bool,
}

/// The work pump on screen: its id, whether it opened itself (`Desk::work_open`: nobody pressed for it) and the
/// anchor its coming wrote (none for one that came while work mode was off): its block is due again only while that
/// anchor stands (`Desk::work_due_again`).
#[derive(Debug)]
struct WorkPump {
	id: String,
	unasked: bool,
	anchor: Option<i64>,
}

/// How a Later's bank came back (`Desk::pump_work_later`).
#[derive(Debug)]
pub(crate) enum LaterEnd {
	/// Its moves are banked.
	Banked,
	/// The session refused the bank (the felt-unwell lock, a reset since the block came due, no move to bank).
	Refused(CatchupError),
	/// The engine composed no work pump (its error logged).
	Failed,
	/// The ask it answered stands no more (the anchor moved while it composed): nothing banked.
	Gone,
}

/// The ask's id Pump home and a notification name (`WorkView::ask`): `w-` and the anchor it asks after.
fn ask_id(anchor: i64) -> String {
	format!("w-{anchor}")
}

/// Where the work clock stands, as the work card and the tray show it.
#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Phase {
	Off,
	Counting,
	Held,
	Due,
	Ask,
}

/// The `work` topic: the switch, whether the first Start's sheet is open and whether it was ever confirmed here
/// (`started`: a Start then turns work mode on at once), where the clock stands with the whole minutes left (rounded
/// up) and what holds it, the id of the ask while the block is due (`ask`, which its Pump now and Later send back),
/// whether a work pump stands on this device (`pump`: no other starts beside it, so the tray greys its Pump now, and
/// the browser's page shows its notification while the one that opened itself stands), Every, and the last Later's
/// refusal (`work.later.reset`, `work.later.unwell`).
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
#[expect(
	clippy::struct_excessive_bools,
	reason = "the `work` topic's flags, each a yes or no its readers take as it is"
)]
pub struct WorkView {
	pub on: bool,
	pub sheet: bool,
	pub started: bool,
	pub phase: Phase,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub left_min: Option<i64>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub hold: Option<Hold>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub ask: Option<String>,
	pub pump: bool,
	pub every: u16,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub refused: Option<&'static str>,
}

/// The `menu.work` topic: the Work mode page's choices (Every among its choices, Moves per block, work mode's own
/// equipment: `gear`, `floor`, `space`, `quiet`) and the How page's numbers (a work pump's moves at that pace and its
/// minutes by the game's estimate of a move's time, rounded up).
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct MenuWork {
	pub every: u16,
	pub choices: &'static [u16],
	pub pace: String,
	#[serde(flatten)]
	pub setup: PlaceSetup,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub moves: Option<u8>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub minutes: Option<i64>,
}

impl<P: Platform> Desk<P> {
	/// Work mode's intents.
	///
	/// # Errors
	/// No such intent, work mode's tunables do not read, or the intent does not apply now (the reason).
	pub fn work_intent(&mut self, intent: &Intent) -> Result<(), String> {
		let t = work::tunables().ok_or("work mode: the tunables do not read")?;
		let outside = intent.arg("outside").and_then(Value::as_bool).unwrap_or(false);
		let ask = intent.arg("ask").and_then(Value::as_str);
		match intent.kind() {
			"work.start" => self.work_start(intent, &t, outside),
			"work.stop" => {
				self.work_stop();
				Ok(())
			}
			"work.later" => self.work_later(ask),
			"work.now" => self.work_now(outside, ask),
			"work.set" => {
				self.work_choose(intent, &t);
				Ok(())
			}
			"work.gear" => {
				let key = intent.arg("key").and_then(Value::as_str).ok_or("work.gear takes key")?;
				let on = intent.arg("on").and_then(Value::as_bool).ok_or("work.gear takes on")?;
				if !facts().sheet.gear_order.iter().any(|k| k == key) {
					return Err(format!("work.gear: no equipment {key}"));
				}
				self.work_equip(|setup| setup.gear = with_gear(&setup.gear, key, on));
				Ok(())
			}
			"work.close" => {
				self.work.sheet = false;
				self.work_publish();
				Ok(())
			}
			_ => Err(intent.unknown()),
		}
	}

	/// Whether work mode is on (the shell's sitting reminder holds while it is: slice 67).
	#[must_use]
	pub fn work_on(&self) -> bool {
		self.settings.work.on
	}

	/// The anchor of the block due on this device (its ask standing or waiting for its moment, or held since): none
	/// while work mode is off, the block still counts, or the anchor moved since the clock's last look.
	fn work_due(&self) -> Option<i64> {
		let w = &self.settings.work;
		(w.on && self.work.clock.due() == Some(w.reset_at)).then_some(w.reset_at)
	}

	/// Whether the ask after `anchor` still stands in the state: work mode on, and the anchor as it was asked.
	fn work_stands(&self, anchor: i64) -> bool {
		let w = &self.settings.work;
		w.on && w.reset_at == anchor
	}

	/// Whether an answer naming the ask `ask` (none: the tray's, which names none) comes too late: the ask it names is
	/// not the one due now.
	fn work_stale(&self, ask: Option<&str>) -> bool {
		ask.is_some_and(|ask| self.work_due().map(ask_id).as_deref() != Some(ask))
	}

	/// The id this device writes as the anchor's mover (`Work::reset_by`): its install id.
	fn work_by(&self) -> String {
		self.settings.hid_text()
	}

	/// Start: the first opens the first Start's sheet (from outside the window, the window first), whose Start comes back
	/// with `confirm` and its choices; every later Start is one tap with the choices held.
	fn work_start(&mut self, intent: &Intent, t: &Tunables, outside: bool) -> Result<(), String> {
		if self.settings.work.on {
			return Err("work mode is on already".to_owned());
		}
		let confirm = intent.arg("confirm").and_then(Value::as_bool).unwrap_or(false);
		if !self.settings.work_started && !confirm {
			if outside {
				self.p.open_window()?;
			}
			self.work.sheet = true;
			self.work_publish();
			return Ok(());
		}
		self.work_choose(intent, t);
		let (now, by) = (self.p.now_ms(), self.work_by());
		self.settings.work.start(now, &by);
		self.settings.work_started = true;
		(self.work.sheet, self.work.refused, self.work.kept) = (false, None, None);
		(self.work.clock, self.work.open) = (Clock::default(), Opening::default());
		::log::info!("work mode: on, every {} min", self.settings.work.every_min);
		self.work_changed();
		Ok(())
	}

	/// Stop: off at once with no confirm; a work pump on screen runs on as a pump of its own, still known as work
	/// mode's (`WorkDesk::pump`), so it holds no clock where work mode starts again under it.
	fn work_stop(&mut self) {
		if !self.settings.work.on {
			return;
		}
		let (now, by) = (self.p.now_ms(), self.work_by());
		self.settings.work.stop(now, &by);
		self.work_off();
		::log::info!("work mode: off");
		self.work_changed();
	}

	/// Work mode went off (Stop, another device's): this device's clock, ask and opening go, the stores' mark and the
	/// work pump on screen stay.
	fn work_off(&mut self) {
		self.work = WorkDesk { mark: self.work.mark, pump: self.work.pump.take(), ..WorkDesk::default() };
	}

	/// Pump now (the ask's, Pump home's, the tray's): a work pump, whose coming moves the anchor (`work_pump_shown`),
	/// on the ask while the block is due and from the tray while it still counts alike. One naming an ask that stands
	/// no more starts nothing and writes nothing.
	fn work_now(&mut self, outside: bool, ask: Option<&str>) -> Result<(), String> {
		if !self.settings.work.on {
			return Err("work mode is off".to_owned());
		}
		if self.work.later.is_some() {
			return Err("a Later is being banked".to_owned());
		}
		if self.work_stale(ask) {
			::log::info!("work mode: Pump now on an ask that stands no more: nothing written");
			return Ok(());
		}
		let due = self.work_due();
		self.work.refused = None;
		let started = self.pump_ask(Start::Work, outside);
		// a start refused (a pump on screen or being made already, a gate) leaves the ask standing
		if started.is_ok() {
			// the ask is answered, and a pump started for a due block is that block's one opening (`work_open`)
			self.work.clock.answered();
			if due.is_some() {
				self.work.open.done = due;
			}
		}
		self.work_publish();
		started
	}

	/// The ask's pump opens itself (each look while the ask stands): a work pump at its Ready view from outside the
	/// window, which comes to the front for it, once for a due block (`Opening::done`). Never while nobody is there (no
	/// input for `sit_away_s`: it opens at the return), and not between two keystrokes: it waits for the input to pause
	/// `OPEN_PAUSE_S`, at most `work_late_s`, so the window takes no key meant for another app. The pump still on
	/// screen (the last block's work pump, left open past this block, or any pump's Result) makes way first, once no
	/// move runs in it and no count or answer is waited for (`Desk::pump_make_way`: tried again at each look while
	/// one is) and only for a start no gate refuses; a Result stands `work_moment_s` before it does, so a pump finished
	/// after this block came due still shows what it earned. A start a gate refuses (the engine not folded yet) is
	/// tried again at the next look, said once in the log; one accepted whose pump never comes (the place window
	/// first, a compose that makes no move) leaves the block due with nothing opened again.
	fn work_open(&mut self, now: i64, idle_s: Option<u32>, t: &Tunables) {
		let Some(due) = self.work_due().filter(|&due| self.work.open.done != Some(due)) else { return };
		let idle = idle_s.map(i64::from);
		if idle.is_some_and(|s| s >= sitting::away_s()) {
			return;
		}
		if idle.is_some_and(|s| s < OPEN_PAUSE_S) {
			let since = match self.work.open.waits {
				Some((anchor, since)) if anchor == due => since,
				_ => now,
			};
			self.work.open.waits = Some((due, since));
			if now - since < t.late_s {
				return;
			}
		}
		if self.pump_ended_at().is_some_and(|at| now - at.div_euclid(1_000) < t.moment_s) {
			return;
		}
		if self.start_gate(&Start::Work).is_none() && !self.pump_make_way() {
			return;
		}
		let ask = ask_id(due);
		match self.work_now(true, Some(&ask)) {
			Ok(()) => {
				self.work.open.unasked = true;
				self.pump_unasked();
				::log::info!("work mode: the pump opens itself ({ask})");
				self.p.work_opened(&ask);
			}
			Err(e) if self.work.open.refused != Some(due) => {
				self.work.open.refused = Some(due);
				::log::info!("work mode: the pump does not open yet: {e}");
			}
			Err(_) => {}
		}
	}

	/// The player went away (no input for `sit_away_s`) from a work pump that opened itself and still waits untouched
	/// on its Ready view: it ends with nothing written, so a pump nobody saw keeps no screen awake and holds no
	/// restart, update or match offer back, and its block is due again from `now` (unix seconds), so the pump opens
	/// again at the return.
	fn work_unattended(&mut self, now: i64) {
		let Some((id, anchor)) = self.work.pump.as_ref().filter(|p| p.unasked).map(|p| (p.id.clone(), p.anchor)) else {
			return;
		};
		if self.pump_end_untouched(&id) {
			::log::info!("work mode: nobody at the pump that opened itself; it closes until the return");
			self.work_due_again(now, anchor);
		}
	}

	/// The block whose work pump left with nothing for it (the felt-unwell lock ended it, or it closed untouched with
	/// nobody there) is due again from `now` (unix seconds): the clock stands due after the anchor that pump's coming
	/// wrote (`anchor`) and no opening is marked, so its pump comes again once nothing holds the clock and the player
	/// is there. Nothing where the anchor moved since (a pump done on another device or in the game): that move starts
	/// the count again, as it drops any block that is due.
	fn work_due_again(&mut self, now: i64, anchor: Option<i64>) {
		if !self.settings.work.on || anchor != Some(self.settings.work.reset_at) {
			return;
		}
		self.work.clock = Clock::due_from(self.settings.work.reset_at, now);
		self.work.open = Opening::default();
		self.work_keep_due();
	}

	/// Later on the ask while the block is due: a work pump banked with no screen (`pump_work_later`), the anchor moved
	/// once it is (`work_later_ended`). One naming an ask that stands no more banks nothing and writes nothing.
	fn work_later(&mut self, ask: Option<&str>) -> Result<(), String> {
		if !self.settings.work.on {
			return Err("work mode is off".to_owned());
		}
		if self.work.later.is_some() {
			return Err("a Later is being banked".to_owned());
		}
		if self.work_stale(ask) {
			::log::info!("work mode: Later on an ask that stands no more: nothing written");
			return Ok(());
		}
		let anchor = self.work_due().ok_or("work mode: no work pump is due")?;
		let at = self.p.now_ms().div_euclid(1_000);
		let facts = self.work_facts(at).ok_or("work mode: no work pump now")?;
		// this Later's own end says what it banked, never an earlier one's refusal
		self.work.refused = None;
		self.work.later = Some(anchor);
		if let Err(e) = self.pump_work_later(facts) {
			self.work.later = None;
			return Err(e);
		}
		::log::info!("work mode: Later");
		// the ask goes while it banks (`work_later_ended` puts it back where nothing could be banked)
		self.work.clock.answered();
		self.work_publish();
		Ok(())
	}

	/// Whether the ask the Later being banked answers still stands (`pump_work_later` banks only then).
	pub(crate) fn work_later_stands(&self) -> bool {
		self.work.later.is_some_and(|anchor| self.work_stands(anchor))
	}

	/// Whether a reset (a Start over, `Store::floor`) came since the block the Later answers came due (now where the
	/// clock holds no due moment): the Later banks nothing then (`work.later.reset`), as the phones compare the floor
	/// with the due moment; the bank's own start stays now, so the felt-unwell lock refuses it as it refuses any.
	pub(crate) fn work_later_reset(&self) -> bool {
		let due_at = self.work.clock.due_since().map_or_else(|| self.p.now_ms().div_euclid(1_000), |(_, at)| at);
		self.session.store().floor().is_some_and(|floor| i64::from(floor) >= due_at)
	}

	/// A Later's bank came back, and Later never claims a bank it could not make: banked, the anchor moved to now;
	/// refused by the felt-unwell lock, its words shown and nothing moved (the lock holds the clock); refused by a reset
	/// since the block came due, its words shown and the count started over; an engine that composed nothing or a
	/// pump with no move to bank, the ask standing still with its Pump now and Later (logged, no words, no move of the
	/// clock); the ask gone while it composed, nothing.
	pub(crate) fn work_later_ended(&mut self, end: LaterEnd) {
		let asked = self.work.later.take();
		match end {
			LaterEnd::Banked => match asked.filter(|&anchor| self.work_stands(anchor)) {
				Some(_) => self.work_anchor(),
				None => ::log::info!("work mode: Later banked as its ask went: the anchor stays"),
			},
			LaterEnd::Refused(CatchupError::Locked(_)) => {
				self.work.refused = Some("work.later.unwell");
				self.work.clock.answered();
			}
			LaterEnd::Refused(CatchupError::BeforeReset) => {
				self.work.refused = Some("work.later.reset");
				self.work_anchor();
			}
			LaterEnd::Refused(other) => {
				::log::warn!("work mode: Later banked nothing: {other}; the ask stands");
				self.work.clock.stand();
			}
			LaterEnd::Failed => self.work.clock.stand(),
			LaterEnd::Gone => ::log::info!("work mode: Later's ask stands no more: nothing banked"),
		}
		self.work_publish();
	}

	/// The Work mode page's or the first Start's choices (`every`, `pace` and the switches `floor`, `space` and
	/// `quiet` of work mode's equipment), each kept where it reads.
	fn work_choose(&mut self, intent: &Intent, t: &Tunables) {
		let w = &self.settings.work;
		let every =
			intent.arg("every").and_then(Value::as_u64).and_then(|v| u16::try_from(v).ok()).unwrap_or(w.every_min);
		let moves_pace = intent.arg("pace").and_then(Value::as_str).unwrap_or(&w.pace).to_owned();
		let switch = |name: &str, held: bool| intent.arg(name).and_then(Value::as_bool).unwrap_or(held);
		let setup = PlaceSetup {
			floor: switch("floor", w.floor),
			space: switch("space", w.space),
			quiet: switch("quiet", w.quiet),
			..w.setup()
		};
		let now = self.p.now_ms();
		if self.settings.work.choose(every, &moves_pace, setup, now, t) {
			self.work_changed();
		}
	}

	/// Work mode's own equipment changed as `change` does (a chip of the Work mode page; Your setup's switches and
	/// chips while work mode is on, `Desk::change_setup`), kept at once with its `settings_at`.
	pub(crate) fn work_equip(&mut self, change: impl FnOnce(&mut PlaceSetup)) {
		let Some(t) = work::tunables() else { return };
		let w = &self.settings.work;
		let (every, moves_pace, mut setup) = (w.every_min, w.pace.clone(), w.setup());
		change(&mut setup);
		let now = self.p.now_ms();
		if self.settings.work.choose(every, &moves_pace, setup, now, &t) {
			self.work_changed();
		}
	}

	/// What a work pump composes with at unix second `at` (the engine's `work`): its warm move once this device's
	/// sitting clock held `work_warm_min`, counted from the later of the last move done (any device's rows this
	/// computer holds) and this device's latest time away or walk (`Platform::seated_s`); Start and Later start nothing
	/// for it (slice 67's R6).
	pub(crate) fn work_facts(&self, at: i64) -> Option<questpump_engine::Work> {
		let t = work::tunables()?;
		Some(questpump_engine::Work { warm: self.p.seated_s(at, self.last_move_at()) >= t.warm_min.saturating_mul(60) })
	}

	/// A work pump came on screen: the anchor moves to this moment, so the next block counts from its coming whatever
	/// is then done with it, and it stands as the pump the next block's takes the place of (`work_open`). One that
	/// comes after a Stop is a pump of its own and moves nothing.
	pub(crate) fn work_pump_shown(&mut self, id: &str) {
		let unasked = std::mem::take(&mut self.work.open.unasked);
		let on = self.settings.work.on;
		if on {
			self.work.refused = None;
			self.work_anchor();
		}
		let anchor = on.then_some(self.settings.work.reset_at);
		self.work.pump = Some(WorkPump { id: id.to_owned(), unasked, anchor });
		self.work_publish();
	}

	/// A pump ended on this device (`pump_finish`), with a move saved (`saved`) or none, felt unwell ending it or not,
	/// while work mode is on: a work pump's end writes nothing, its coming having moved the anchor, but one felt unwell
	/// ended leaves its block due again, so its pump opens again once the lock lifts and the player is there; any other
	/// pump with a move saved moves the anchor to now.
	pub(crate) fn work_pump_ended(&mut self, id: &str, saved: bool, unwell: bool) {
		let pump = self.work.pump.take_if(|p| p.id == id);
		if !self.settings.work.on {
			return;
		}
		match pump {
			Some(pump) if unwell => {
				::log::info!("work mode: the work pump ended for felt unwell: its block is due again");
				let now = self.p.now_ms().div_euclid(1_000);
				self.work_due_again(now, pump.anchor);
			}
			None if saved => self.work_anchor(),
			Some(_) | None => {}
		}
	}

	/// The anchor moved here now (a work pump's coming, a Later banked or one a reset refused, another pump done): the
	/// clock counts again from it.
	fn work_anchor(&mut self) {
		let (now, by) = (self.p.now_ms(), self.work_by());
		self.settings.work.anchor(now, &by);
		self.work.clock.answered();
		self.work_changed();
	}

	/// Work mode's own equipment while it is on, which every pump composed here takes in place of the place in use's
	/// (slice 67, contract/places.json); none while it is off.
	pub(crate) fn work_setup(&self) -> Option<PlaceSetup> {
		let w = &self.settings.work;
		w.on.then(|| w.setup())
	}

	/// Another device's work state heard (a sync batch, a linked phone's request): merged by the rule, and on a change
	/// kept and every view told; whether it changed.
	pub fn work_heard(&mut self, heard: &Work) -> bool {
		let Some(t) = work::tunables() else { return false };
		let was_on = self.settings.work.on;
		if !self.settings.work.merge(heard, self.p.now_ms(), &t) {
			return false;
		}
		::log::info!("work mode: another device's state taken (on: {})", self.settings.work.on);
		if was_on && !self.settings.work.on {
			self.work_off();
		}
		self.work_changed();
		true
	}

	/// The work state changed here: kept (with the due block as the clock holds it now), sent to the account's devices,
	/// the engine's profile with Moves per block (`profile.workPace`, which a Later and a work pump compose at, and a
	/// heard state may change too) and, while on, work mode's equipment (`Desk::setup_in_force`), and the views told:
	/// Your setup's, the welcome's place and the places' (the place button and the place rows name Work mode while it
	/// is on) and work mode's.
	fn work_changed(&mut self) {
		self.settings.work_due = self.work_due_kept();
		self.save_settings();
		self.sync_changed();
		self.profile_changed();
		self.publish_setup();
		self.publish_welcome();
		self.work_publish();
	}

	/// The loop's look (every 2 s, from `home_look`): the anchor moved by a pump done on another device or in the game,
	/// then the work clock over this device's presence and holds, and the views.
	pub fn work_look(&mut self) {
		let Some(t) = work::tunables() else { return };
		let now_ms = self.p.now_ms();
		let now = now_ms.div_euclid(1_000);
		self.work_moved(now_ms);
		if !self.settings.work.on {
			self.work.state = None;
			self.work_keep_due();
			self.work_publish();
			return;
		}
		if self.work_kept(now_ms) {
			self.work_publish();
			return;
		}
		let idle_s = self.p.idle_s();
		// nobody was there: no input for `sit_away_s`, or no look for as long (the computer slept through it)
		let slept = self.work.looked.replace(now).is_some_and(|at| now - at >= sitting::away_s());
		if slept || idle_s.is_some_and(|s| i64::from(s) >= sitting::away_s()) {
			self.work_unattended(now);
		}
		let hold = self.work_hold(now);
		if hold != self.work.held {
			match hold {
				Some(hold) => ::log::info!("work mode: held ({hold:?})"),
				None => ::log::info!("work mode: nothing holds it"),
			}
			self.work.held = hold;
		}
		let seen = Seen { now, anchor_ms: self.settings.work.reset_at, idle_s, hold };
		let state = self.work.clock.look(&seen, self.settings.work.every_min, &t);
		if state == (State::Ask { new: true }) {
			::log::info!("work mode: a pump is due and asks");
			// a new ask lets a refused Later's words go
			self.work.refused = None;
		}
		self.work.state = Some(state);
		self.work_keep_due();
		if matches!(state, State::Ask { .. }) {
			self.work_open(now, idle_s, &t);
		}
		self.work_publish();
	}

	/// Whether the due block kept across a restart (`WorkDesk::kept`) waits still: until the account's first pull
	/// answered (none waits where Sync is off or out of reach), at most `KEPT_WAIT_MS` from its first look, so an ask
	/// another device answered while this one was off never shows (R2); then the clock takes it due where its anchor
	/// still stands, and it goes where the anchor moved. Nothing is due while it waits, so no Pump now or Later acts.
	fn work_kept(&mut self, now_ms: i64) -> bool {
		let Some(due) = self.work.kept else { return false };
		let until = *self.work.kept_until.get_or_insert(now_ms.saturating_add(KEPT_WAIT_MS));
		if now_ms < until && self.sync_first_pull_pending() {
			return true;
		}
		self.work.kept = None;
		if due.anchor_ms == self.settings.work.reset_at {
			self.work.clock = Clock::due_from(due.anchor_ms, due.at);
		}
		false
	}

	/// The due block as the settings keep it (`Settings::work_due`): the clock's, or the one kept across a restart while
	/// it waits (`work_kept`), while work mode is on and its anchor is the state's (none once a work pump's coming or a heard
	/// state moved it).
	fn work_due_kept(&self) -> Option<WorkDue> {
		let w = &self.settings.work;
		let due = self.work.clock.due_since().map(|(anchor_ms, at)| WorkDue { anchor_ms, at }).or(self.work.kept);
		due.filter(|d| w.on && d.anchor_ms == w.reset_at)
	}

	/// The due block kept in the settings as the clock holds it, saved as it changes (it came due, its anchor moved),
	/// so a restart or a reload starts the clock due again (`WorkDesk::kept`).
	fn work_keep_due(&mut self) {
		let due = self.work_due_kept();
		if self.settings.work_due != due {
			self.settings.work_due = due;
			self.save_settings();
		}
	}

	/// A move done in the game since the anchor (the stores took rows since the last look): the anchor moves to it, as
	/// any pump done on any device moves it (this device's own pumps move it at their end, `work_pump_ended`, and every
	/// other device's at theirs, heard in `work`, so a work pump's own moves never move the ask it answers).
	fn work_moved(&mut self, now_ms: i64) {
		let mark = self.store_mark();
		if self.work.mark == Some(mark) {
			return;
		}
		self.work.mark = Some(mark);
		let w = &self.settings.work;
		let moved =
			self.last_move_in_game().map(|s| s.saturating_mul(1_000)).filter(|&at| at > w.reset_at && at <= now_ms);
		if let Some(at) = moved.filter(|_| w.on) {
			let by = self.work_by();
			self.settings.work.anchor(at, &by);
			self.work.clock.answered();
			self.work_changed();
		}
	}

	/// What holds the work clock on this device now: a pump of another kind open, any pump being made or a Later being
	/// banked, the felt-unwell lock, what the platform reads (a call, presenting, full screen, a game in front, World of
	/// Warcraft's window among them), a match's offer standing or the game's quest pump open, the player's quiet hours.
	/// A work pump on screen holds nothing: its coming moved the anchor, and the next block counts on while it stands.
	fn work_hold(&self, now: i64) -> Option<Hold> {
		let other = self.pump_running() && self.work.pump.is_none();
		if other || self.pump_starting() || self.work.later.is_some() {
			return Some(Hold::Pump);
		}
		if self.session.locked_until(now).is_some() {
			return Some(Hold::Unwell);
		}
		if let Some(hold) = self.p.work_hold() {
			return Some(hold);
		}
		// World of Warcraft holds it only while its quest pump is open, as the platform's look holds it while its
		// window is in front (slice 67: a game left open behind the work app holds nothing)
		if self.p.offer().is_some() || self.session.pump_on() {
			return Some(Hold::Game);
		}
		let quiet = offers::tunables().is_some_and(|t| {
			let local = now + (self.p.zone())(now);
			let minute = u16::try_from(local.rem_euclid(86_400) / 60).unwrap_or(0);
			offers::quiet(minute, self.settings.offers.quiet(t))
		});
		quiet.then_some(Hold::Quiet)
	}

	/// The `work` and `menu.work` topics now, and the platform told when the view changed.
	pub fn work_publish(&mut self) {
		let Some(t) = work::tunables() else { return };
		let w = &self.settings.work;
		let (phase, left_s, hold) = match self.work.state.filter(|_| w.on) {
			None if w.on => (Phase::Counting, None, None),
			None => (Phase::Off, None, None),
			Some(State::Counting { left_s }) => (Phase::Counting, Some(left_s), None),
			Some(State::Held { hold, left_s }) => (Phase::Held, Some(left_s), Some(hold)),
			Some(State::Due) => (Phase::Due, None, None),
			Some(State::Ask { .. }) => (Phase::Ask, None, None),
		};
		let view = WorkView {
			on: w.on,
			sheet: self.work.sheet,
			started: self.settings.work_started,
			phase,
			left_min: left_s.map(|s| (s + 59) / 60),
			hold,
			ask: self.work_due().map(ask_id),
			pump: self.work.pump.is_some(),
			every: w.every_min,
			refused: self.work.refused,
		};
		let moves = work::pump_moves(&w.pace);
		let menu = MenuWork {
			every: w.every_min,
			choices: t.every_choices,
			pace: w.pace.clone(),
			setup: w.setup(),
			moves,
			minutes: moves
				.map(|m| (i64::from(m) * facts().constants().move_estimate_s + 59) / 60)
				.filter(|&min| min > 0),
		};
		self.publisher.publish(TOPIC, &view);
		self.publisher.publish(MENU_TOPIC, &menu);
		if self.work.told.as_ref() != Some(&view) {
			self.p.work_changed(&view);
			self.work.told = Some(view);
		}
	}
}
