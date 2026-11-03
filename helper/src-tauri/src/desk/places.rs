//! The places on the computer (docs/plan.md slice 51, "Places"; the phones' places): Home, the game's one setup, and
//! every other place the player keeps, each a name, a glyph, its equipment and its three switches
//! (`questpump_core::body::Place`). Home is the body's `home` (between the game, the computer and the phones), and
//! while no device and no game stamped it, this computer's own setup from before the places (`settings.place`, which
//! a change of Home made here follows); the others are the body's `places` (between the computer and the phones,
//! never the game), and while no device changed them, the templates of `contract/places.json` (`places::seeded`),
//! kept nowhere until the player changes one. Their plan (2026-11-03; Rapha: a default place and a weekly schedule) is
//! the body's `place_plan` beside them, Home the default and no time while no device changed it. The place in use is
//! this computer's own, never in the body: the last pick (`settings.place_in_use`, with when it was made) until the
//! schedule's next start or end or the plan's next change, else the time running now, else the default
//! (`questpump_core::places::in_use`, in this computer's local time). The engine's profile takes its equipment and
//! switches (`Helper::engine_profile`, again
//! at each pump's start and at each start or end of a time, `Helper::places_tick`), and Your setup's place section,
//! Start a pump's place row and Pump home's place button name it and change it. Each change of the body goes through
//! `Helper::body_changed`, stamped past the field's own (`setup::stamped`), so the latest change wins on every device.
//!
//! **The `places` topic**: `{list: [{id, template?, name?, glyph, gear, floor, space, quiet, home}], in_use, default,
//! times: [{days, from, to, place}], most, name_most, times_most, templates: [{id, template, glyph, gear, floor, space,
//! quiet}], empty: {glyph, gear, floor, space, quiet}, glyphs, words}`, Home first, then the others in the player's
//! order; `default` the default place in force (Home where the plan's has left the list), `times` the schedule's in
//! the player's order; `templates` Add a place's Start from (every template but Home), `words` the sheet's
//! (`menu_pages::Words`: the equipment's names and groups and the switches' names).
//!
//! **Intents**: `places.use {id}` (a pick: the place in use until the schedule's next start or end or the plan's next
//! change), `places.save {id?, template?, name?, glyph, gear, floor, space, quiet, default?}` (a place's window's Save;
//! no `id`: a new place, refused once the list holds `places_max`; Home keeps its name, its glyph and its place;
//! `default` its Default place switch turned on: the place is the default from then), `places.delete {id}` (never Home;
//! its times go, and the default goes to Home where it was), `places.order {ids}` (every place but Home in its new
//! order), `places.time.save {at?, days, from, to, place}` (a time's window's Save: `at` the time's place in the
//! schedule, none for a new one, refused once it holds `schedule_max`) and `places.time.delete {at}`.

use questpump_core::body::{Body, Place, PlacePlan, PlaceSetup, PlaceTime};
use questpump_core::places::{self, EmptyPlace};
use questpump_core::session::Zone;
use std::fmt::Write as _;

use serde::Serialize;
use serde_json::Value;

use tauri::Manager as _;

use super::facts::facts;
use super::menu_pages::WORDS;
use super::menu_pages::setup::stamped;
use super::{Desk, Intent};
use crate::lan;
use crate::runtime::{Helper, local_offset_at};
use crate::settings::{PlacePick, PumpPlace};

const TOPIC: &str = "places";

/// Home's id where `contract/places.json` does not read (its own `home`).
const HOME: &str = "home";

/// A place as the page draws it.
#[derive(Serialize)]
struct Shown<'a> {
	id: &'a str,
	#[serde(skip_serializing_if = "Option::is_none")]
	template: Option<&'a str>,
	#[serde(skip_serializing_if = "Option::is_none")]
	name: Option<&'a str>,
	glyph: &'a str,
	#[serde(flatten)]
	setup: &'a PlaceSetup,
	home: bool,
}

impl<'a> Shown<'a> {
	fn of(place: &'a Place) -> Self {
		Self {
			id: &place.id,
			template: place.template.as_deref(),
			name: place.name.as_deref(),
			glyph: &place.glyph,
			setup: &place.setup,
			home: false,
		}
	}
}

#[derive(Serialize)]
struct Topic<'a> {
	list: Vec<Shown<'a>>,
	in_use: &'a str,
	default: &'a str,
	times: &'a [PlaceTime],
	most: usize,
	name_most: usize,
	times_most: usize,
	templates: &'a [Place],
	empty: Option<&'a EmptyPlace>,
	glyphs: &'a [String],
	words: &'a Value,
}

/// Home's id.
fn home_id() -> &'static str {
	places::places().map_or(HOME, |p| p.home.as_str())
}

/// Home's glyph (its template's).
fn home_glyph() -> &'static str {
	places::places()
		.and_then(|p| p.templates.iter().find(|t| t.id == p.home))
		.map_or("place_home", |t| t.glyph.as_str())
}

/// The templates Add a place starts from, every one but Home, as places (the addon's defaults where one names none).
fn templates() -> &'static [Place] {
	static SEEDED: std::sync::LazyLock<Vec<Place>> = std::sync::LazyLock::new(places::seeded);
	&SEEDED
}

/// A new place's id: `p-` and 8 lower-case hex digits, none of `taken`.
fn new_id(taken: &[Place]) -> String {
	loop {
		let mut random = [0u8; 4];
		rand::fill(&mut random);
		let id = random.iter().fold(String::from("p-"), |mut id, b| {
			let _ = write!(id, "{b:02x}");
			id
		});
		if taken.iter().all(|p| p.id != id) {
			return id;
		}
	}
}

/// An intent's text field.
fn text<'a>(intent: &'a Intent, name: &str) -> Option<&'a str> {
	intent.arg(name).and_then(Value::as_str)
}

/// A place's equipment with the piece `key` on or off: the sheet's pieces in `gear_order`, then those this build does
/// not name (a newer device's), kept as they were.
pub(crate) fn with_gear(gear: &[String], key: &str, on: bool) -> Vec<String> {
	let order = &facts().sheet.gear_order;
	let mut out: Vec<String> =
		order.iter().filter(|k| if k.as_str() == key { on } else { gear.contains(k) }).cloned().collect();
	out.extend(gear.iter().filter(|k| !order.contains(k)).cloned());
	out
}

/// The setup a place's window saves: its equipment (each a key of the sheet's `gear`, kept in `gear_order`, then the
/// pieces this build does not name that the place had, `kept`) and its three switches.
fn setup_of(intent: &Intent, kept: &[String]) -> Result<PlaceSetup, String> {
	let flag = |name: &str| intent.arg(name).and_then(Value::as_bool).ok_or(format!("places.save takes {name}"));
	let keys: Vec<&str> = intent
		.arg("gear")
		.and_then(Value::as_array)
		.ok_or("places.save takes gear")?
		.iter()
		.filter_map(Value::as_str)
		.collect();
	let order = &facts().sheet.gear_order;
	if let Some(unknown) = keys.iter().find(|k| !order.iter().any(|o| o == *k)) {
		return Err(format!("places.save: no equipment {unknown}"));
	}
	let mut gear: Vec<String> = order.iter().filter(|k| keys.contains(&k.as_str())).cloned().collect();
	gear.extend(kept.iter().filter(|k| !order.contains(k)).cloned());
	Ok(PlaceSetup { gear, floor: flag("floor")?, space: flag("space")?, quiet: flag("quiet")? })
}

impl Helper {
	/// Home's equipment and switches: the body's, else this computer's own from before the places.
	pub(crate) fn home_setup(&self) -> PlaceSetup {
		self.body.body.home.as_ref().map_or_else(|| PlaceSetup::from(&self.settings.place), |h| h.value.clone())
	}

	/// Every place but Home in the player's order: the body's, else the templates as seeded.
	fn other_places(&self) -> Vec<Place> {
		self.body.body.places.as_ref().map_or_else(|| templates().to_vec(), |p| p.value.clone())
	}

	/// The places' plan: the body's, else Home the default and no time.
	fn place_plan(&self) -> PlacePlan {
		self.body.body.place_plan.as_ref().map_or_else(PlacePlan::default, |p| p.value.clone())
	}

	/// The place in use on this computer now (`places::in_use`): its id and its setup.
	pub(crate) fn place_in_use(&self) -> (String, PlaceSetup) {
		let others = self.other_places();
		let home = home_id();
		let plan = self.place_plan();
		let plan_at = self.body.body.place_plan.as_ref().map(|p| p.changed_at.div_euclid(1_000));
		let pick = self.settings.place_in_use.as_ref().map(|p| (p.id.as_str(), p.at.div_euclid(1_000)));
		let listed = |id: &str| id == home || others.iter().any(|p| p.id == id);
		let now = lan::now_ms().div_euclid(1_000);
		let id = places::in_use(&plan, plan_at, pick, listed, now, &Zone::new(local_offset_at)).to_owned();
		match others.into_iter().find(|p| p.id == id) {
			Some(p) => (p.id, p.setup),
			None => (home.to_owned(), self.home_setup()),
		}
	}

	/// The loop's look (every few seconds): at a start or an end of a time the place in use changes on its own, and the
	/// engine takes its profile and the pages (Pump home's place button among them) show it.
	pub(crate) fn places_tick(&mut self) {
		let (id, _) = self.place_in_use();
		if self.home.place.as_deref() != Some(id.as_str()) {
			self.home.place = Some(id);
			self.profile_changed();
			self.publish_setup();
		}
	}

	/// The plan changed here now (the default, a time saved or deleted): into the body, stamped.
	fn plan_changed(&mut self, plan: PlacePlan) {
		let changed = stamped(plan, self.body.body.place_plan.as_ref());
		self.body_changed(&Body { place_plan: Some(changed), ..Body::default() });
	}

	/// Place `id` (listed already) is the default place from now.
	fn default_place(&mut self, id: &str) {
		let plan = PlacePlan { default: id.to_owned(), ..self.place_plan() };
		self.plan_changed(plan);
	}

	/// Whether `id` names Home or a place of the list.
	fn listed(&self, id: &str) -> bool {
		id == home_id() || self.other_places().iter().any(|p| p.id == id)
	}

	/// The place in use as a place, with Home's glyph and its template for Home (whose words name it).
	pub(crate) fn place_shown(&self) -> Place {
		let (id, setup) = self.place_in_use();
		if id == home_id() {
			return Place { id: id.clone(), template: Some(id), name: None, glyph: home_glyph().to_owned(), setup };
		}
		self.other_places().into_iter().find(|p| p.id == id).unwrap_or_else(|| Place {
			id,
			template: None,
			name: None,
			glyph: home_glyph().to_owned(),
			setup,
		})
	}

	/// The place in use changed as `change` does (Your setup's switches and equipment chips, the welcome's place):
	/// Home's change goes into the body's `home` and this computer's own setup, another place's into the body's
	/// `places`; nothing when it stays as it was. Asked no more where the player pumps (`Helper::place_changed`).
	pub(crate) fn change_place(&mut self, change: impl FnOnce(&mut PlaceSetup)) {
		let (id, before) = self.place_in_use();
		let mut setup = before.clone();
		change(&mut setup);
		if setup != before {
			self.set_place(&id, setup);
		}
		self.place_changed();
	}

	/// Home's setup set to `setup` (a backup file's place), a change of the body made now.
	pub(crate) fn set_home(&mut self, setup: PlaceSetup) {
		self.set_place(home_id(), setup);
		self.place_changed();
	}

	/// Place `id`'s setup set to `setup`, a change of the body made now.
	fn set_place(&mut self, id: &str, setup: PlaceSetup) {
		if id == home_id() {
			self.settings.place = PumpPlace::from(&setup);
			let changed = stamped(setup, self.body.body.home.as_ref());
			self.body_changed(&Body { home: Some(changed), ..Body::default() });
			return;
		}
		let mut list = self.other_places();
		if let Some(place) = list.iter_mut().find(|p| p.id == id) {
			place.setup = setup;
			self.places_changed(list);
		}
	}

	/// Every place but Home, changed here now (a save, a delete, an order): into the body, stamped.
	fn places_changed(&mut self, list: Vec<Place>) {
		let changed = stamped(list, self.body.body.places.as_ref());
		self.body_changed(&Body { places: Some(changed), ..Body::default() });
	}

	/// The `places` area's intents (the module's doc lists them).
	pub(crate) fn places_intent(&mut self, intent: &Intent) -> Result<(), String> {
		let home = home_id();
		match intent.kind() {
			"places.use" => {
				let id = text(intent, "id").ok_or("places.use takes an id")?;
				if !self.listed(id) {
					return Err(format!("places.use: no place {id}"));
				}
				self.settings.place_in_use = Some(PlacePick { id: id.to_owned(), at: lan::now_ms() });
				self.place_changed();
			}
			"places.save" => {
				let id = text(intent, "id");
				let default = intent.arg("default").and_then(Value::as_bool).unwrap_or(false);
				if id == Some(home) {
					let setup = setup_of(intent, &self.home_setup().gear)?;
					self.set_place(home, setup);
					if default {
						self.default_place(home);
					}
					self.place_changed();
					return Ok(());
				}
				let kept = id
					.and_then(|id| self.other_places().into_iter().find(|p| p.id == id))
					.map(|p| p.setup.gear)
					.unwrap_or_default();
				let setup = setup_of(intent, &kept)?;
				let name = text(intent, "name").map(str::trim).filter(|n| !n.is_empty()).map(str::to_owned);
				let template = text(intent, "template").map(str::to_owned);
				let glyph = text(intent, "glyph").ok_or("places.save takes a glyph")?.to_owned();
				let mut list = self.other_places();
				let id = if let Some(id) = id {
					if !list.iter().any(|p| p.id == id) {
						return Err(format!("places.save: no place {id}"));
					}
					id.to_owned()
				} else {
					let most = places::places().map_or(12, |p| p.places_max);
					if list.len() + 1 >= most {
						return Err("places.save: the list holds its most".to_owned());
					}
					new_id(&list)
				};
				let place = Place { id: id.clone(), template, name, glyph, setup };
				if !place.checks_out() {
					return Err(format!("places.save: {id} does not read"));
				}
				match list.iter_mut().find(|p| p.id == id) {
					Some(kept) => *kept = place,
					None => list.push(place),
				}
				self.places_changed(list);
				if default {
					self.default_place(&id);
				}
			}
			"places.delete" => {
				let id = text(intent, "id").ok_or("places.delete takes an id")?;
				let mut list = self.other_places();
				let before = list.len();
				list.retain(|p| p.id != id);
				if list.len() == before {
					return Err(format!("places.delete: no place {id} (Home is never deleted)"));
				}
				if self.settings.place_in_use.as_ref().is_some_and(|p| p.id == id) {
					self.settings.place_in_use = None;
					self.save_settings();
				}
				self.places_changed(list);
				let plan = self.place_plan();
				if plan.default == id || plan.times.iter().any(|t| t.place == id) {
					let default = if plan.default == id { home.to_owned() } else { plan.default };
					let times = plan.times.into_iter().filter(|t| t.place != id).collect();
					self.plan_changed(PlacePlan { default, times });
				}
			}
			"places.order" => {
				let ids: Vec<&str> = intent
					.arg("ids")
					.and_then(Value::as_array)
					.ok_or("places.order takes ids")?
					.iter()
					.filter_map(Value::as_str)
					.collect();
				let list = self.other_places();
				let mut sorted: Vec<&str> = ids.clone();
				sorted.sort_unstable();
				let mut have: Vec<&str> = list.iter().map(|p| p.id.as_str()).collect();
				have.sort_unstable();
				if sorted != have {
					return Err("places.order takes every place but Home, once each".to_owned());
				}
				let ordered = ids.iter().filter_map(|id| list.iter().find(|p| p.id == *id).cloned()).collect();
				self.places_changed(ordered);
			}
			"places.time.save" | "places.time.delete" => self.schedule_intent(intent)?,
			_ => return Err(intent.unknown()),
		}
		Ok(())
	}

	/// The schedule's intents, `places.time.save` and `places.time.delete` (the module's doc).
	fn schedule_intent(&mut self, intent: &Intent) -> Result<(), String> {
		match intent.kind() {
			"places.time.save" => {
				let at = intent.arg("at").and_then(Value::as_u64).and_then(|at| usize::try_from(at).ok());
				let time = serde_json::from_value::<PlaceTime>(serde_json::json!({
					"days": intent.arg("days"),
					"from": intent.arg("from"),
					"to": intent.arg("to"),
					"place": intent.arg("place"),
				}))
				.map_err(|e| format!("places.time.save: {e}"))?;
				if !time.checks_out() || !self.listed(&time.place) {
					return Err("places.time.save: the time does not read".to_owned());
				}
				let mut plan = self.place_plan();
				match at {
					Some(at) => *plan.times.get_mut(at).ok_or(format!("places.time.save: no time {at}"))? = time,
					None if plan.times.len() >= places::places().map_or(21, |p| p.schedule_max) => {
						return Err("places.time.save: the schedule holds its most".to_owned());
					}
					None => plan.times.push(time),
				}
				self.plan_changed(plan);
			}
			"places.time.delete" => {
				let at = intent
					.arg("at")
					.and_then(Value::as_u64)
					.and_then(|at| usize::try_from(at).ok())
					.ok_or("places.time.delete takes at")?;
				let mut plan = self.place_plan();
				if at >= plan.times.len() {
					return Err(format!("places.time.delete: no time {at}"));
				}
				plan.times.remove(at);
				self.plan_changed(plan);
			}
			_ => return Err(intent.unknown()),
		}
		Ok(())
	}

	/// `places` as it stands.
	pub(crate) fn publish_places(&self) {
		let home = Place {
			id: home_id().to_owned(),
			template: Some(home_id().to_owned()),
			name: None,
			glyph: home_glyph().to_owned(),
			setup: self.home_setup(),
		};
		let others = self.other_places();
		let (in_use, _) = self.place_in_use();
		let plan = self.place_plan();
		let default = if self.listed(&plan.default) { plan.default.as_str() } else { home_id() };
		let list =
			std::iter::once(Shown { home: true, ..Shown::of(&home) }).chain(others.iter().map(Shown::of)).collect();
		let p = places::places();
		let topic = Topic {
			list,
			in_use: &in_use,
			default,
			times: &plan.times,
			most: p.map_or(12, |p| p.places_max),
			name_most: p.map_or(24, |p| p.place_name_max),
			times_most: p.map_or(21, |p| p.schedule_max),
			templates: templates(),
			empty: p.map(|p| &p.empty),
			glyphs: p.map_or(&[][..], |p| p.glyphs.as_slice()),
			words: &WORDS,
		};
		self.app.state::<Desk>().publisher.publish(TOPIC, &topic);
	}
}
