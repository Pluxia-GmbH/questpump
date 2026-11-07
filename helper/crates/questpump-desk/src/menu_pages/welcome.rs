//! The first five minutes on the computer (docs/ui.md, "The first five minutes"; the phones' first start, slice 29),
//! for a new player, with no account and no sign-up. The desktop app is the helper: it sees the game's folder, its
//! saves, its mirror, a running client and the paired phone, so it never asks what it can see and never treats a
//! player with progress as new. What it found (`Desk::found`: whether the stores were read, where their progress
//! came from, the game, the phone) is read again whenever one of those changes (`welcome_found`), and nothing is
//! decided before the stores are read: no welcome shows before the take-in.
//!
//! **A returning player** (the stores hold progress) never sees the welcome. Whether the player is new is decided
//! once, at the first look after the stores were read, whatever step settings.json kept: any progress then ends the
//! flow, and none marks the player new (`found_new`). From then on only progress from the game or a phone ends it,
//! never this computer's own rows (the flow's own first pump's, or a pump from the tray meanwhile): at once while the
//! welcome stands at its first step, else at the next step change; never while a pump runs. The window then shows Pump
//! home, and the page's Toast says once where the progress came from and where QuestPump waits (`toast`, kept in
//! settings.json until the page says it showed it).
//!
//! **A new player**: the welcome (the figure, the wordmark, one line, the card On this computer with the game, found
//! or running with a check and the line that the game asks the pumps while it runs, else the game here without
//! QuestPump with "Add QuestPump to the game" (QuestPump puts the addon into the game folder, `runtime/addon.rs`), else
//! "No game found yet" (the tray's words) with "Choose its folder…" (This computer's picker, whose take-in may find
//! progress, its note under the card); and the phone, paired with a check and the Phones page's line, else "Pair
//! your phone", the Phones page's pairing code in a window over the welcome, which closes once the phone pairs; then
//! the gold Start) → the experience question (none when the body every app device shares holds one already, a paired
//! phone's, or a ladder holds a rung; once asked, Back from the place goes back to it) → where the player pumps (beside
//! the desk, or room to lie down, which sets the floor moves and room to move together; Quiet mode, which sets this
//! computer's quiet mode, never a Body switch) → the first Quick pump at once (`Start::Quick`, the engine's one pump,
//! never the timed pump every other Quick pump starts since 2026-11-03, then its Result; its own
//! step, so a restart before the pump shows the place's Start again) → Hero lighting the
//! muscles it trained, once the pump's Result or calm card was continued (`welcome.pumped`) → done: QuestPump waits in
//! the menu bar (Windows: the tray), the game asks the pumps while it runs when one was found, and a quiet "Pair your
//! phone" while none is paired; the games found are watched from the day they are found, on the Games tab (slice 56),
//! so no step asks for them. While a gate holds the pump back (the game with QuestPump runs, felt unwell locks pumps,
//! the engine failed), the place step says so and its Continue goes on to done, never a pump certain to be refused.
//! Each step is kept in settings.json, so a closed window or a restart picks the flow up where it stood.
//!
//! **The place before the first own pump**: a player who never answered where they pump on this computer (a returning
//! player, the welcome left early) is asked it once, before the first pump from any route (`settings.place_asked`; the
//! pump's start waits for it, `pump` topic's `place`): the welcome's place rows in a window, whose answer
//! (`welcome.where`) lets the waiting start go on (`Desk::place_answered`).
//!
//! **The `welcome` topic**: `{step: "welcome"|"experience"|"place"|"pump"|"hero"|"done"|null, found: {ready,
//! progress: "game"|"phone"|"both"|"here"|null, game: "running"|"found"|"installed"|"none", phone: name|null}, toast:
//! "game"|"phone"|"both"|"here"|null, experience, place: {floor, quiet}, trained: {tones: {muscle id: quality},
//! names: {muscle id: name}} | null, phones, words}`: `step` none once the first five minutes are done and before the
//! stores are read; `toast` where the progress came from that ended them, until the Toast said it; `trained` the Hero
//! step's, from the engine's own sheet; `phones` false while the addon's verified feed says the phone apps are not in
//! the stores yet, which hides the phone's part (the Phones page in the Menu keeps the pairing code for testers).
//!
//! **Intents**: `welcome.start`, `welcome.back`, `welcome.experience {answer}`, `welcome.place {floor, quiet, pump}`
//! (`pump`: Start my first pump, else the Continue a gate shows), `welcome.pumped` (the first pump left the screen),
//! `welcome.look` (the Hero step's muscles: none lit skips the step), `welcome.next` (on from Hero),
//! `welcome.done`; and the two taken once the flow is done too: `welcome.toasted` (the Toast said the progress found)
//! and `welcome.where {floor, quiet}` (the place window's answer).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{WELCOME, WORDS};
use crate::Intent;
use crate::facts::facts;
use crate::pump::Start;
use crate::seams::{Came, Found};
use crate::{Desk, Platform};

/// Where the first five minutes stand.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Step {
	/// The welcome; `addon`, the page for a player who played with the addon (until 2026-10-08), reads as it.
	#[default]
	#[serde(alias = "addon")]
	Welcome,
	Experience,
	Place,
	/// The first Quick pump, asked from the place step: Hero once it left the screen.
	Pump,
	Hero,
	/// Done; `games`, the games to watch before done until slice 56, reads as it.
	#[serde(alias = "games")]
	Done,
}

/// The muscles the first pump trained, as Hero lights them: each one's quality (1 Starter to 7 Artifact, the engine's
/// `mq`), and every muscle's name for the map.
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Trained {
	tones: BTreeMap<u8, u8>,
	names: BTreeMap<u8, String>,
}

#[derive(Serialize)]
struct Place {
	floor: bool,
	quiet: bool,
}

#[derive(Serialize)]
struct Topic<'a> {
	step: Option<Step>,
	found: &'a Found,
	toast: Option<Came>,
	experience: &'a str,
	place: Place,
	trained: Option<&'a Trained>,
	/// The phone's part shows (the phone row, Done's Pair your phone, the Toast's pairing card): not while the addon's
	/// verified feed says the phone apps are not in the stores yet (`Desk::phones_offered`).
	phones: bool,
	words: &'a Value,
}

impl<P: Platform> Desk<P> {
	/// The `welcome` area's intents.
	///
	/// # Errors
	/// Why the intent was refused.
	pub fn welcome_intent(&mut self, intent: &Intent) -> Result<(), String> {
		match intent.kind() {
			"welcome.toasted" => {
				if self.settings.desk.toast.take().is_some() {
					self.save_settings();
					self.publish_welcome();
				}
				return Ok(());
			}
			"welcome.where" => return self.answer_place(intent),
			_ => {}
		}
		if self.settings.desk.onboarded {
			return Err(format!("{}: the first five minutes are done", intent.kind()));
		}
		let step = self.settings.desk.welcome;
		match intent.kind() {
			"welcome.start" => {
				let asked = self.settings.desk.asked_experience || self.asks_experience();
				self.settings.desk.asked_experience = asked;
				self.to(if asked { Step::Experience } else { Step::Place });
			}
			"welcome.back" => match step {
				Step::Experience => self.to(Step::Welcome),
				Step::Place | Step::Pump => {
					self.to(if self.settings.desk.asked_experience { Step::Experience } else { Step::Welcome });
				}
				_ => return Err("welcome.back: no step back from here".to_owned()),
			},
			"welcome.experience" => {
				let answer = intent.arg("answer").and_then(Value::as_str).ok_or("welcome.experience takes answer")?;
				self.set_experience(answer)?;
				self.to(Step::Place);
			}
			"welcome.place" if matches!(step, Step::Place | Step::Pump) => self.first_pump(intent)?,
			"welcome.pumped" if step == Step::Pump => {
				if self.desk_pump_running() {
					return Err("welcome.pumped: the pump still runs".to_owned());
				}
				self.to(Step::Hero);
			}
			"welcome.look" => self.welcome_look(),
			"welcome.next" if step == Step::Hero => self.to(Step::Done),
			"welcome.next" => return Err("welcome.next: no next step from here".to_owned()),
			"welcome.done" => self.onboarded(None),
			"welcome.place" | "welcome.pumped" => return Err(format!("{}: not at {step:?}", intent.kind())),
			_ => return Err(intent.unknown()),
		}
		Ok(())
	}

	/// The place step's answer: the place kept, then the first Quick pump (`pump`, Start my first pump; its own step
	/// first, so a restart before it shows the place's Start again), refused while a gate holds it back, which the step
	/// then says; or on to done (the Continue the step shows while a gate holds: never a pump certain to be refused,
	/// never a silent skip).
	fn first_pump(&mut self, intent: &Intent) -> Result<(), String> {
		let pump = intent.arg("pump").and_then(Value::as_bool).ok_or("welcome.place takes pump")?;
		self.answer_place(intent)?;
		self.menu.trained = None;
		if !pump {
			self.to(Step::Done);
			return Ok(());
		}
		if let Some(gate) = self.pump_gate() {
			return Err(format!("welcome.place: no pump while {gate:?} holds")); // copy-lint: not shown (an intent's error)
		}
		self.to(Step::Pump);
		if self.settings.desk.onboarded {
			return Ok(());
		}
		// the engine's one Quick pump, never the timed pump every other Quick pump starts (2026-11-03)
		self.pump_ask(Start::Quick, false)
	}

	/// Where the player pumps on this computer, from the place step or the place window before the first own pump:
	/// beside the desk (no floor moves and no room to move, as the addon's Office preset sets them) or with room to
	/// lie down (both on; 2026-10-09), and quiet, asked no more (`Desk::place_changed`, which lets a start that
	/// waited for it go on).
	fn answer_place(&mut self, intent: &Intent) -> Result<(), String> {
		let on = |name| intent.arg(name).and_then(Value::as_bool).ok_or(format!("{} takes {name}", intent.kind()));
		let (floor, quiet) = (on("floor")?, on("quiet")?);
		self.change_place(|place| (place.floor, place.space, place.quiet) = (floor, floor, quiet));
		Ok(())
	}

	/// `welcome` as it stands.
	pub fn publish_welcome(&self) {
		let desk = &self.settings.desk;
		let (_, place) = self.place_in_use();
		let topic = Topic {
			step: (!desk.onboarded && self.menu.found.ready).then_some(desk.welcome),
			found: &self.menu.found,
			toast: desk.toast,
			experience: self
				.body
				.body
				.experience
				.as_ref()
				.map_or(questpump_core::body::DEFAULT_EXPERIENCE, |e| &e.value),
			place: Place { floor: place.floor, quiet: place.quiet },
			trained: self.menu.trained.as_ref(),
			phones: self.phones_offered(),
			words: &WORDS,
		};
		self.publish(WELCOME, &topic);
	}

	/// What QuestPump found, read again while the first five minutes stand (the stores took rows, the stores were read,
	/// the game or the pairing changed, the 2-s look): shown, and the flow done for a returning player at the first
	/// look after the stores were read, at whatever step it stood, and whenever later it stands at the welcome.
	pub fn welcome_found(&mut self) {
		if self.settings.desk.onboarded {
			return;
		}
		let read_before = self.menu.found.ready;
		self.take_found();
		if (!read_before && self.menu.found.ready) || self.settings.desk.welcome == Step::Welcome {
			self.welcome_back();
		}
	}

	/// What QuestPump found now, kept and shown when it changed.
	fn take_found(&mut self) {
		let found = self.found();
		if found != self.menu.found {
			self.menu.found = found;
			self.publish_welcome();
		}
	}

	/// The first five minutes end for a returning player, once the stores were read and while no pump runs: at the
	/// first look, any progress (none marks the player new, `found_new`, kept); after it, only the game's or a phone's,
	/// never this computer's own rows (the flow's own first pump's). True when they ended.
	fn welcome_back(&mut self) -> bool {
		let found = &self.menu.found;
		if !found.ready {
			return false;
		}
		let first = !self.settings.desk.found_new;
		let Some(came) = found.progress.filter(|&came| first || came != Came::Here) else {
			if first {
				self.settings.desk.found_new = true;
				self.save_settings();
			}
			return false;
		};
		if self.desk_pump_running() {
			return false;
		}
		self.onboarded(Some(came));
		true
	}

	/// The experience question has an answer to give: no device set one in the body yet, and no ladder holds a rung
	/// (the start rungs were set from an earlier answer, on any device).
	fn asks_experience(&self) -> bool {
		self.body.body.experience.is_none() && !self.menu.setup.as_ref().is_some_and(super::setup::Look::started)
	}

	/// The flow on at `step`, kept; or done instead, when the stores hold progress that ends it by now.
	fn to(&mut self, step: Step) {
		self.take_found();
		if self.welcome_back() {
			return;
		}
		self.settings.desk.welcome = step;
		self.save_settings();
		self.publish_welcome();
	}

	/// The first five minutes done: the main window opens on Pump from now on. `found`: QuestPump found progress that
	/// came from there, which the page's Toast says once.
	fn onboarded(&mut self, found: Option<Came>) {
		let at = self.settings.desk.welcome;
		match found {
			Some(came) => log::info!("welcome: found progress ({came:?}) at {at:?}, done"),
			None => log::info!("welcome: done at {at:?}"),
		}
		self.settings.desk.onboarded = true;
		self.settings.desk.toast = found;
		self.save_settings();
		self.publish_welcome();
	}

	/// The Hero step's muscles: the engine's own sheet now, after the first pump; none lit (the pump did not run, or
	/// trained none) goes on to done.
	fn welcome_look(&mut self) {
		let at = self.p.now_ms().div_euclid(1_000);
		self.engine_call(
			move |engine| engine.summary(at),
			|helper, answer| {
				let tones: BTreeMap<u8, u8> = match answer {
					Ok(summary) => summary
						.get("mq")
						.and_then(Value::as_array)
						.map(|mq| {
							mq.iter()
								.enumerate()
								.filter_map(|(i, q)| {
									let q = u8::try_from(q.as_u64()?).ok().filter(|&q| q > 0)?;
									Some((u8::try_from(i + 1).ok()?, q))
								})
								.collect()
						})
						.unwrap_or_default(),
					Err(e) => {
						log::warn!("welcome: the engine's sheet: {e}");
						BTreeMap::new()
					}
				};
				if helper.settings.desk.onboarded || helper.settings.desk.welcome != Step::Hero {
					return;
				}
				if tones.is_empty() {
					helper.to(Step::Done);
					return;
				}
				let names = facts().sheet.muscles.iter().map(|m| (m.id, m.name.clone())).collect();
				helper.menu.trained = Some(Trained { tones, names });
				helper.publish_welcome();
			},
		);
	}
}
