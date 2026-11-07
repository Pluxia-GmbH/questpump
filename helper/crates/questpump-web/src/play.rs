//! The browser's own part of the desk (docs/plan.md slice 63, "Save your progress"; David, 2026-11-06: save right
//! after the first quick pump): the `play` topic, which only the browser publishes, so the shared pages in
//! `helper/dist` show its parts only where it stands and the desktop app, which never publishes it, looks and behaves
//! as before. The account comes when the player asks for it, never in silence: the first pump's Result carries "Save
//! your progress" as its main action with "Not now" beside it (each puts the offer away, `play.later`); until the
//! progress is saved Pump home shows "Not saved" from that Result on, which opens it; and Pump home's card asks once
//! more after the first record beaten or on the third game day, so Save is offered at most twice. What the browser
//! kept is `play.json` in the data folder.
//!
//! Each Result comes from the desk's own `pump` topic as the page's sink sends it (`Play::seen`, queued and taken
//! after the door's call, `Play::take`), so the desk needs no hook of the browser's: a pump whose topic shows its
//! Result ended done (its `ended_done`, the desk's `Run::ended_done`: a move done and no felt-unwell stop), never one
//! that ended before a move ran or because the player felt unwell.
//!
//! **The topic**: `{"saved": bool, "unsaved": bool, "offer": "result"|"home"|null, "key_due": bool, "phones": bool}`:
//! `saved` the player holds a way back into this browser's account (an email linked, a join by a restore or link
//! code, or Keep your key's Done; review of 2026-11-06), never merely an account Save's Send made; `unsaved` Pump
//! home's "Not saved" shows (a Result came and the progress is not saved); `offer` where Save is offered now (the
//! Result's main action, Pump home's card); `key_due` the player holds a way back and has not seen Keep your key, which
//! the browser's pages then open (`web/views/index.js`), so every way in ends there: Save, Sign in and join, I played
//! before; `phones` the phone apps are in the stores (the build's `PHONES`), so the Menu offers QuestPump for the phone
//! beside the computer's.
//!
//! **Intents** (`play.*`, which the door takes before the desk): `play.later` (Not now): the offer goes; `play.done`
//! (Keep your key's Done): the offer goes and the key counts as seen; `play.wipe` (Delete progress in this browser):
//! a guest's page deletes this browser's data and starts again (`{"wipe": "out"}`), and in an account the door takes
//! it as Sign out anyway (`email.signout {anyway: true}`: the account forgotten, `DELETE /session`, then the wipe).
//! Whether the browser is in an account and holds a way back comes from the desk's sync after each call
//! (`Play::account`).

use std::cell::RefCell;

use questpump_core::session::Zone;
use questpump_desk::Intent;
use questpump_desk::publisher::Publisher;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{imports, web};

/// The topic's name.
pub const TOPIC: &str = "play";

/// `play.json`: what the browser kept of the offers.
pub const FILE: &str = "play.json";

/// The desk's topic a Result shows on.
const PUMP: &str = "pump";

/// The game day, counted from the first Result's as 1, on which Pump home's card asks again.
const AGAIN_DAY: i64 = 3;

const DAY_S: i64 = 86_400;

/// What the browser keeps between its visits.
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
#[expect(clippy::struct_excessive_bools, reason = "what the browser kept, one yes or no a fact")]
struct Kept {
	/// The start of the game day of the first Result (unix seconds): the first offered Save on it.
	first_day: Option<i64>,
	/// The pump whose Result came last, so a Result the topic shows again counts once.
	last: Option<String>,
	/// Pump home's later card was offered.
	asked_again: bool,
	/// In an account the player holds a way back into.
	saved: bool,
	/// Keep your key's Done was pressed.
	#[serde(default)]
	key: bool,
	/// The player held a way back into the account this browser holds (`Play::account`'s `back`): kept, since a join
	/// by a restore or link code is the desk's to know in its run alone, so a reload before Keep your key's Done still
	/// opens it.
	#[serde(default)]
	back: bool,
}

/// Where Save is offered now.
#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum Offer {
	Result,
	Home,
}

#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[expect(clippy::struct_excessive_bools, reason = "the page's facts, one yes or no a field, as the topic names them")]
struct Topic {
	saved: bool,
	unsaved: bool,
	offer: Option<Offer>,
	key_due: bool,
	phones: bool,
}

/// A Result the `pump` topic showed: its pump and whether a record was beaten in it.
struct Shown {
	pump: String,
	record: bool,
}

thread_local! {
	/// The Results the sink saw during a door's call, for `Play::take` after it.
	static SHOWN: RefCell<Vec<Shown>> = const { RefCell::new(Vec::new()) };
}

/// The browser's offers and the `play` topic.
pub struct Play {
	kept: Kept,
	offer: Option<Offer>,
	/// The phone apps are in the stores (the build's `PHONES`).
	phones: bool,
	publisher: Publisher,
	zone: Zone,
}

impl Play {
	/// The offers as the browser kept them (`kept`, `play.json`'s bytes, none on a first visit), published at once;
	/// `phones` the build's `PHONES`.
	#[must_use]
	pub fn new(publisher: Publisher, kept: Option<&[u8]>, phones: bool) -> Self {
		let kept: Kept = kept
			.and_then(|bytes| serde_json::from_slice(bytes).map_err(|e| log::warn!("web: {FILE}: {e}")).ok())
			.unwrap_or_default();
		// the first Result offers Save: the topic says so before it shows, since the Result draws from the pump's topic
		let offer = (kept.first_day.is_none() && !kept.saved).then_some(Offer::Result);
		let play = Self { kept, offer, phones, publisher, zone: Zone::new(imports::offset_at) };
		play.publish();
		play
	}

	fn publish(&self) {
		let topic = Topic {
			saved: self.kept.saved,
			unsaved: self.kept.first_day.is_some() && !self.kept.saved,
			offer: self.offer,
			key_due: self.kept.back && !self.kept.key,
			phones: self.phones,
		};
		self.publisher.publish(TOPIC, &topic);
	}

	fn keep(&self) {
		match serde_json::to_vec(&self.kept) {
			Ok(bytes) => web::keep(FILE, &bytes),
			Err(e) => log::warn!("web: {FILE} did not serialise: {e}"),
		}
	}

	/// A topic's change on its way to the page: a `pump` that shows a Result is noted for `take` (never acted on here,
	/// inside the desk's call).
	pub fn seen(topic: &str, data: &Value) {
		if topic != PUMP {
			return;
		}
		let Some(pump) = data.get("pump").filter(|pump| !pump.is_null()) else { return };
		// the desk's own word for a pump that ended with a move done and no felt-unwell stop, as the Save offer reads it
		let done = pump.get("ended_done").and_then(Value::as_bool) == Some(true);
		let Some(result) = pump.get("result").filter(|result| done && !result.is_null()) else { return };
		let Some(id) = pump.get("id").and_then(Value::as_str) else { return };
		let record = result.get("rows").and_then(Value::as_array).is_some_and(|rows| {
			rows.iter().any(|row| row.pointer("/reward/kind").and_then(Value::as_str) == Some("record"))
		});
		SHOWN.with_borrow_mut(|shown| shown.push(Shown { pump: id.to_owned(), record }));
	}

	/// Takes the Results `seen` noted at `now` (unix seconds): the first, which offered Save as it showed, starts "Not
	/// saved"; a later one, with a record beaten or on the third game day from the first's, offers Pump home's card
	/// once.
	pub fn take(&mut self, now: i64) {
		let shown = SHOWN.with_borrow_mut(std::mem::take);
		let mut changed = false;
		for Shown { pump, record } in shown {
			if self.kept.saved || self.kept.last.as_deref() == Some(pump.as_str()) {
				continue;
			}
			self.kept.last = Some(pump);
			changed = true;
			let today = self.zone.game_day_start(now);
			match self.kept.first_day {
				None => self.kept.first_day = Some(today),
				Some(first) if !self.kept.asked_again => {
					// a first Result left without its Save or Not now offers nothing more
					if self.offer == Some(Offer::Result) {
						self.offer = None;
					}
					let day = (today - first + DAY_S / 2).div_euclid(DAY_S) + 1;
					if record || day >= AGAIN_DAY {
						self.kept.asked_again = true;
						self.offer = Some(Offer::Home);
					}
				}
				Some(_) => {}
			}
		}
		if changed {
			self.keep();
			self.publish();
		}
	}

	/// Whether this browser is in an account now (`held`) and the player holds a way back into it (`back`: an email
	/// linked or a join by a code, the desk's `sync_way_back`), after each call: saved once a way back is held or Keep
	/// your key's Done was pressed in it, and so while the account is held; then Save is offered no more and "Not
	/// saved" goes.
	pub fn account(&mut self, held: bool, back: bool) {
		// a way back once held stays while the account does, a join by a code too after a reload
		let back = held && (back || self.kept.back);
		let saved = held && (back || self.kept.key || self.kept.saved);
		let key = self.kept.key && held;
		if (self.kept.saved, self.kept.key, self.kept.back) == (saved, key, back) {
			return;
		}
		(self.kept.saved, self.kept.key, self.kept.back) = (saved, key, back);
		if saved {
			self.offer = None;
		}
		self.keep();
		self.publish();
	}

	/// A `play.*` intent: `Ok` once taken, else why not.
	///
	/// # Errors
	/// An intent this area does not know, or one whose arguments do not read.
	pub fn intent(&mut self, intent: &Intent) -> Result<(), String> {
		match intent.kind() {
			"play.later" => self.offer = None,
			"play.done" => {
				self.offer = None;
				self.kept.key = true;
				self.keep();
			}
			"play.wipe" => {
				log::info!("web: Delete progress in this browser");
				web::wipe(web::Wipe::Out);
				return Ok(());
			}
			_ => return Err(intent.unknown()),
		}
		self.publish();
		Ok(())
	}
}
