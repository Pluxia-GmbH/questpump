//! Your setup and Your moves on the computer (docs/ui.md, "The Menu on the computer"; the phones' Your setup and Your
//! moves, slice 34): the place in use (the equipment, floor moves, room to move, quiet; `desk::places`, its change a
//! change of Home's or another place's in the body since slice 51); the body every app device shares (the Body
//! switches, the experience, the moves left out), each change made here stamped now, past the field's own stamp, in
//! `body.json` and given to every paired phone through the body's exchange; and the pain flags and the focus, rows on
//! every device already, written through the engine (once it has folded) and kept as the computer's own rows
//! (`Desk::keep_own`). The engine's profile follows every change (`Desk::profile_changed`), and the engine's look
//! follows the profile and every fold and merge of rows (`Desk::setup_look`, asked by the engine's door).
//!
//! **The `menu.setup` topic**: `{place_id, place: {gear: [key], floor, space, quiet}, switches: {no_jump, no_strain,
//! no_head_down}, body_reset, experience: "none"|"some"|"training", look: {pain: [joint id], focus: id|null, started,
//! steps: [{key, count, quality?}], opens: [{key, count, quality?}]} | null, in_game: [joint id],
//! moves: {groups: [{key, name}], out: [{move, name}], count}, pump_on, words}`, `body_reset` while the body started
//! empty because `body.json` did not read at this start and no device filled it since (`Kept::unread`), `look` the
//! engine's (none before its first look, or while it does not answer), `in_game` the joints the game's own rows flag
//! (the fold's answer, `Desk::engine_folded`), `pump_on` while a pump runs that a change here waits for (a pump's
//! Settings opens this page over its Ready view, where a change reaches an app pump at once: `Desk::setup_refilter`;
//! a catch-up takes only the pain flags at once, resting the moves they load), `words` the sheet's (`menu_pages::Words`).
//!
//! **Intents**: `setup.look` (the engine's look again: the page's mount), `setup.place {key: "floor"|"space"|"quiet",
//! on}`, `setup.gear {key, on}`, `setup.switch {key: "no_jump"|"no_strain"|"no_head_down", on}`, `setup.experience
//! {answer}`, `setup.include {move, on}` (a move left out of pumps, or taken back in), `setup.group {group, on}` (a
//! group the exclusion list holds whole), `setup.pain {joint, on}` and `setup.focus {focus}` (both refused while the
//! engine folds, as Hero's and Pump home's writes are).

use questpump_core::body::{Body, Changed, DEFAULT_EXPERIENCE, Moves, PlaceSetup, is_experience};
use questpump_engine::SetupStep;
use serde::Serialize;
use serde_json::Value;

use super::{SETUP, WORDS};
use crate::Intent;
use crate::facts::{Named, facts};
use crate::{Desk, Platform};

/// How many equipment lines Your setup shows (the phones' `FreeMode.setupLineCount`).
const SETUP_LINES: u8 = 2;
/// The rows Your setup writes, as the log names them (`Desk::keep_own`).
const PAIN_ROW: &str = "Your setup's pain flag"; // copy-lint: not shown (a log line's words)
const FOCUS_ROW: &str = "Your setup's focus"; // copy-lint: not shown (a log line's words)

/// What the engine says of the setup: the joints flagged now (the game's and every app's rows), the focus in force (the
/// latest focus row's; none before one, Balanced), whether a ladder holds a rung yet (the start rungs are set), what
/// the next pieces of equipment would open (`steps`, the equipment lines) and what each piece not added would open on
/// its own (`opens`, a gear chip's tooltip; `questpump_engine::SetupStep`).
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Look {
	pain: Vec<u8>,
	focus: Option<u8>,
	started: bool,
	steps: Vec<SetupStep>,
	opens: Vec<SetupStep>,
}

impl Look {
	/// Whether the start rungs are set: the welcome then asks no experience.
	#[must_use]
	pub fn started(&self) -> bool {
		self.started
	}
}

/// The Body switches as the body holds them (off while no device changed one).
#[derive(Serialize)]
struct Switches {
	no_jump: bool,
	no_strain: bool,
	no_head_down: bool,
}

/// A move the player left out of pumps.
#[derive(Serialize)]
struct Out {
	#[serde(rename = "move")]
	move_id: u16,
	name: &'static str,
}

/// Your moves (the phones' `YourMoves`, without the favourites the computer keeps none of): the groups the exclusion
/// list holds whole and the moves the player left out, by name, and how many moves those are.
#[derive(Serialize)]
struct YourMoves {
	groups: Vec<Named>,
	out: Vec<Out>,
	count: usize,
}

#[derive(Serialize)]
struct Topic<'a> {
	/// The place in use (`desk::places`): its id and its setup.
	place_id: &'a str,
	place: &'a PlaceSetup,
	switches: Switches,
	body_reset: bool,
	experience: &'a str,
	look: Option<&'a Look>,
	in_game: &'a [u8],
	moves: YourMoves,
	/// A pump runs that a change here waits for (`Desk::desk_pump_refits`): the page's card says so.
	pump_on: bool,
	words: &'a Value,
}

/// A move with a switch of its own: one of the library's, offered without being checked, never the floor move (the
/// phones' `YourMoves`).
fn switched(id: u16) -> bool {
	let library = &facts().library;
	id != library.floor_move && library.moves.get(&id).is_some_and(|m| !m.opt_in)
}

impl YourMoves {
	fn of(moves: &Moves) -> Self {
		let f = facts();
		let mut groups: Vec<Named> = moves
			.exclude
			.groups
			.iter()
			.map(|g| Named { key: g.clone(), name: f.sheet.groups.get(g).cloned().unwrap_or_else(|| g.clone()) })
			.collect();
		groups.sort_by(|a, b| a.name.cmp(&b.name));
		let mut out: Vec<Out> = moves
			.exclude
			.ids
			.iter()
			.filter(|&&id| switched(id))
			.filter_map(|&id| f.name(id).map(|name| Out { move_id: id, name }))
			.collect();
		out.sort_by(|a, b| a.name.cmp(b.name));
		let grouped = f
			.library
			.moves
			.iter()
			.filter(|(id, m)| switched(**id) && moves.exclude.groups.contains(&m.group))
			.map(|(id, _)| *id);
		let mut every: Vec<u16> = out.iter().map(|o| o.move_id).chain(grouped).collect();
		every.sort_unstable();
		every.dedup();
		Self { groups, out, count: every.len() }
	}
}

/// Takes a group the exclusion list holds whole back in, or leaves it out again (the phones' `checkGroup`).
fn check_group(moves: &mut Moves, group: &str, on: bool) {
	moves.exclude.groups.retain(|g| g != group);
	if !on {
		moves.exclude.groups.push(group.to_owned());
	}
}

/// An intent's boolean field.
fn flag(intent: &Intent, name: &str) -> Result<bool, String> {
	intent.arg(name).and_then(Value::as_bool).ok_or_else(|| format!("{} takes {name}", intent.kind()))
}

/// An intent's text field.
fn text<'a>(intent: &'a Intent, name: &str) -> Result<&'a str, String> {
	intent.arg(name).and_then(Value::as_str).ok_or_else(|| format!("{} takes {name}", intent.kind()))
}

/// An intent's number field, as the type it must fit.
fn number<T: TryFrom<u64>>(intent: &Intent, name: &str) -> Result<T, String> {
	intent
		.arg(name)
		.and_then(Value::as_u64)
		.and_then(|n| T::try_from(n).ok())
		.ok_or_else(|| format!("{} takes {name}", intent.kind()))
}

/// A change of a body field made on this computer now: stamped at this computer's clock, and past the field's own
/// stamp when a phone's clock runs ahead of it, so the player's newest change always wins (`Body::merge`: the latest
/// stamp wins).
pub fn stamped<T>(value: T, field: Option<&Changed<T>>, now_ms: i64) -> Changed<T> {
	let after = field.map_or(i64::MIN, |f| f.changed_at.saturating_add(1));
	Changed { value, changed_at: now_ms.max(after) }
}

impl<P: Platform> Desk<P> {
	/// The `setup` area's intents.
	///
	/// # Errors
	/// Why the intent was refused.
	pub fn setup_intent(&mut self, intent: &Intent) -> Result<(), String> {
		match intent.kind() {
			"setup.look" => self.setup_look(),
			"setup.place" => {
				let on = flag(intent, "on")?;
				let key = text(intent, "key")?;
				if !matches!(key, "floor" | "space" | "quiet") {
					return Err(format!("setup.place: no switch {key}"));
				}
				self.change_place(|place| match key {
					"floor" => place.floor = on,
					"space" => place.space = on,
					_ => place.quiet = on,
				});
			}
			"setup.gear" => {
				let (key, on) = (text(intent, "key")?, flag(intent, "on")?);
				let order = &facts().sheet.gear_order;
				if !order.iter().any(|k| k == key) {
					return Err(format!("setup.gear: no equipment {key}"));
				}
				self.change_place(|place| place.gear = super::super::places::with_gear(&place.gear, key, on));
			}
			"setup.switch" => {
				let (key, on) = (text(intent, "key")?, flag(intent, "on")?);
				let body = &self.body.body;
				let change = match key {
					"no_jump" => {
						Body { no_jump: Some(stamped(on, body.no_jump.as_ref(), self.p.now_ms())), ..Body::default() }
					}
					"no_strain" => Body {
						no_strain: Some(stamped(on, body.no_strain.as_ref(), self.p.now_ms())),
						..Body::default()
					},
					"no_head_down" => Body {
						no_head_down: Some(stamped(on, body.no_head_down.as_ref(), self.p.now_ms())),
						..Body::default()
					},
					other => return Err(format!("setup.switch: no switch {other}")),
				};
				self.body_changed(&change);
			}
			"setup.experience" => {
				let answer = text(intent, "answer")?;
				self.set_experience(answer)?;
			}
			"setup.include" => {
				let (id, on) = (number::<u16>(intent, "move")?, flag(intent, "on")?);
				let (group, opt_in, members) =
					facts().group(id).ok_or_else(|| format!("setup.include: no move {id}"))?;
				self.change_moves(|moves| moves.check(id, on, group, opt_in, &members));
			}
			"setup.group" => {
				let (group, on) = (text(intent, "group")?, flag(intent, "on")?);
				self.change_moves(|moves| check_group(moves, group, on));
			}
			"setup.pain" => {
				let (joint, on) = (number::<u8>(intent, "joint")?, flag(intent, "on")?);
				if facts().joint(joint).is_none() {
					return Err(format!("setup.pain: no joint {joint}"));
				}
				self.engine_ready_or(intent)?;
				self.set_pain(joint, on);
				// after its rows on the engine's ordered door: the pump on its Ready view follows the flags
				self.setup_refilter(false);
			}
			"setup.focus" => {
				let focus = number::<u8>(intent, "focus")?;
				if facts().focus(focus).is_none() {
					return Err(format!("setup.focus: no focus {focus}"));
				}
				self.engine_ready_or(intent)?;
				self.set_focus(focus);
				// after its row on the engine's ordered door: the pump on its Ready view is composed with the new focus
				self.setup_refilter(true);
			}
			_ => return Err(intent.unknown()),
		}
		Ok(())
	}

	/// `menu.setup` as it stands.
	pub fn publish_setup(&self) {
		let body = &self.body.body;
		let on = |switch: &Option<Changed<bool>>| switch.as_ref().is_some_and(|s| s.value);
		let moves = body.moves.as_ref().map(|m| m.value.clone()).unwrap_or_default();
		let (place_id, place) = self.place_in_use();
		let topic = Topic {
			place_id: &place_id,
			place: &place,
			switches: Switches {
				no_jump: on(&body.no_jump),
				no_strain: on(&body.no_strain),
				no_head_down: on(&body.no_head_down),
			},
			body_reset: self.body.unread.is_some() && *body == Body::default(),
			experience: body.experience.as_ref().map_or(DEFAULT_EXPERIENCE, |e| e.value.as_str()),
			look: self.menu.setup.as_ref(),
			in_game: self.engine_folded().map_or(&[], |f| f.game_flags.as_slice()),
			moves: YourMoves::of(&moves),
			pump_on: self.desk_pump_running() && !self.desk_pump_refits(),
			words: &WORDS,
		};
		self.publish(SETUP, &topic);
		self.publish_places();
		self.publish_expedition();
	}

	/// What the engine says of the setup now (the joints flagged, the focus, the start rungs, the equipment lines and
	/// what each piece would open), then `menu.setup` and `welcome`: asked by the page's mount, a changed profile and
	/// every fold and merge of rows the engine's door answers, one at a time (one asked while another runs asks again
	/// once it answers). A call waits behind a fold that runs, and an engine that does not answer leaves the look as it
	/// was.
	pub fn setup_look(&mut self) {
		if self.menu.looking {
			self.menu.look_again = true;
			return;
		}
		self.menu.looking = true;
		let at = self.p.now_ms().div_euclid(1_000);
		self.engine_call(
			move |engine| {
				Ok((engine.pain_flags()?, engine.summary(at)?, engine.setup_steps(SETUP_LINES)?, engine.setup_opens()?))
			},
			|helper, answer| {
				helper.menu.looking = false;
				match answer {
					Ok((pain, summary, steps, opens)) => {
						// the summary's focus code (`fc`) and each ladder's first-reached rungs (`ear`, `{0}` while
						// none)
						let focus = summary.get("fc").and_then(Value::as_u64).and_then(|f| u8::try_from(f).ok());
						let started = summary.get("ear").and_then(Value::as_array).is_some_and(|ladders| {
							ladders.iter().any(|l| l.get(0).and_then(Value::as_u64).is_some_and(|best| best > 0))
						});
						helper.menu.setup = Some(Look { pain, focus, started, steps, opens });
					}
					Err(e) => log::warn!("menu: the engine's look at the setup: {e}"),
				}
				if std::mem::take(&mut helper.menu.look_again) {
					helper.setup_look();
				}
				helper.publish_setup();
				helper.publish_welcome();
			},
		);
	}

	/// This computer's place changed (here, in the first five minutes or in the place window before the first own
	/// pump): asked no more (`settings.place_asked`), kept, given to the engine (whose look follows) and shown, and a
	/// start that waited for it goes on.
	pub fn place_changed(&mut self) {
		self.settings.place_asked = true;
		self.save_settings();
		self.profile_changed();
		self.publish_setup();
		self.place_answered();
	}

	/// A change of the body made on this computer, stamped past the field's own stamp (`stamped`): it wins over every
	/// earlier change on any device, goes into body.json, reaches the paired phones at their next exchange and gives
	/// the engine the profile (whose look follows).
	pub fn body_changed(&mut self, change: &Body) {
		if self.body.body.merge(change, true) {
			self.save_body();
			self.profile_changed();
		}
		self.publish_setup();
		self.publish_welcome();
	}

	/// The experience question's answer, a change of the body made now.
	///
	/// # Errors
	/// An answer the question does not offer.
	pub fn set_experience(&mut self, answer: &str) -> Result<(), String> {
		if !is_experience(answer) {
			return Err(format!("no experience {answer}"));
		}
		let changed = stamped(answer.to_owned(), self.body.body.experience.as_ref(), self.p.now_ms());
		self.body_changed(&Body { experience: Some(changed), ..Body::default() });
		Ok(())
	}

	/// The moves left out of pumps and the opt-in moves checked, changed as `change` does (Your moves' switches and the
	/// Book's Include in pumps, each through `Moves::check`), a change of the body made now; nothing when they stay as
	/// they are.
	pub fn change_moves(&mut self, change: impl FnOnce(&mut Moves)) {
		let before = self.body.body.moves.as_ref().map(|m| m.value.clone()).unwrap_or_default();
		let mut moves = before.clone();
		change(&mut moves);
		if moves == before {
			return;
		}
		let changed = stamped(moves, self.body.body.moves.as_ref(), self.p.now_ms());
		self.body_changed(&Body { moves: Some(changed), ..Body::default() });
	}

	/// A joint's pain flag set or cleared, a row the engine writes as the game does (kept as the computer's own, whose
	/// merge asks the look again).
	fn set_pain(&mut self, joint: u8, on: bool) {
		let at = self.p.now_ms().div_euclid(1_000);
		self.engine_call(
			move |engine| engine.pain_rows(&[joint], on, at),
			|helper, answer| helper.keep_own(PAIN_ROW, answer),
		);
	}

	/// The focus chosen, a row the engine writes as the game's focus window does (kept as the computer's own, whose
	/// merge asks the look again).
	fn set_focus(&mut self, focus: u8) {
		let at = self.p.now_ms().div_euclid(1_000);
		self.engine_call(
			move |engine| engine.focus_row(focus, at),
			|helper, answer| helper.keep_own(FOCUS_ROW, answer.map(|row| vec![row])),
		);
	}
}
