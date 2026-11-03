//! The engine's thread: the Lua state, `tools/host.lua`'s environment on the embedder's clock and zone, the addon's
//! pure files loaded into one namespace, `tools/apppump.lua`'s driver on them, the player's engine and the pump on
//! screen; and each op of the door ([`crate::Engine::call`]) as the driver's call.

#[cfg(not(target_os = "emscripten"))]
use std::sync::mpsc::Receiver;
use std::time::Instant;

use mlua::{Function, LuaOptions, LuaSerdeExt, ObjectLike, StdLib, Table, Value};
use serde::Serialize;
use serde::de::DeserializeOwned;

#[cfg(not(target_os = "emscripten"))]
use crate::Job;
use crate::api::{
	self, Allowed, Check, CheckOutcome, Composed, CorrectCheck, CorrectSet, Corrected, CorrectionCheck, Dosed, Folded,
	JudgedStep, Judgement, LevelAt, Locked, Merged, NeedState, NextLoad, Open, Picked, PlannedMove, Preview, Pump,
	Record, Rider, RidesAs, Rung, SetKey, SetupStep, Slot, Start, SwapOptions, Take, Target, Toward,
};
use crate::embedded::{APPPUMP, FILES, HOST};
use crate::{Clock, EngineError};

/// The most memory the engine's Lua may hold: a fold of 54,000 rows held about 55 MiB (David's Mac, 2026-10-08), so
/// reaching it is a fault the caller hears as an error, never the computer's memory gone.
const MEMORY_LIMIT: usize = 512 << 20;

/// The offset callback's cache: a 15-minute bucket whose two ends agree is read once (a zone's offset never changes
/// twice within 15 minutes); one whose ends differ is asked at each instant.
const BUCKET_S: i64 = 900;

/// The computer's time zone as the host reads it, `offset_at` asked once per 15-minute bucket whose ends agree.
struct Offsets {
	at: Box<dyn Fn(i64) -> i64 + Send>,
	/// The last bucket read and its offset (none: the offset changes inside it, asked at each instant).
	last: Option<(i64, Option<i64>)>,
}

impl Offsets {
	fn at(&mut self, t: i64) -> i64 {
		let bucket = t.div_euclid(BUCKET_S);
		let steady = match self.last {
			Some((b, steady)) if b == bucket => steady,
			_ => {
				let start = bucket * BUCKET_S;
				let (first, last) = ((self.at)(start), (self.at)(start + BUCKET_S - 1));
				let steady = (first == last).then_some(first);
				self.last = Some((bucket, steady));
				steady
			}
		};
		steady.unwrap_or_else(|| (self.at)(t))
	}
}

/// The Lua state and what lives in it.
pub(crate) struct Host {
	lua: mlua::Lua,
	/// `tools/apppump.lua`'s `A`.
	app: Table,
	/// The addon's namespace the pure files fill (`ns`), for a rule of theirs the driver does not wrap.
	ns: Table,
	/// The player's engine (`A.new`), after `init`.
	player: Option<Table>,
	/// The pump on screen (`A.compose`, `A.quick`, or a catch-up's, `A.catchup`).
	pump: Option<Table>,
	/// The Swap options shown last: the move's index (from 1) and its options, which a swap takes from.
	shown: Option<(usize, Table)>,
	/// The save `gameSummary` folds from (the `gameSave` op's table and its Base64), kept across `init`; none before one
	/// came.
	game_save: Option<(Table, String)>,
}

/// mlua's serializer with Rust's `None` as Lua's nil (the driver reads an absent key), lists marked as arrays.
fn to_lua_options() -> mlua::serde::SerializeOptions {
	mlua::serde::SerializeOptions::new().serialize_none_to_null(false).serialize_unit_to_null(false)
}

/// The game's save as a Lua table, as the client loads a save: every table a plain one (no array metatable), a map's
/// integer keys numbers and a CBOR null nil. None for a save that is not CBOR of a map.
fn save_table(lua: &mlua::Lua, base64_cbor: &str) -> Result<Table, EngineError> {
	use base64::Engine as _;
	let refused = |why: String| EngineError::Refused(format!("the game's save: {why}"));
	let cbor = base64::engine::general_purpose::STANDARD.decode(base64_cbor).map_err(|e| refused(e.to_string()))?;
	let save: ciborium::Value = ciborium::from_reader(cbor.as_slice()).map_err(|e| refused(e.to_string()))?;
	let options = to_lua_options().set_array_metatable(false);
	match lua.to_value_with(&save, options)? {
		Value::Table(t) => Ok(t),
		_ => Err(refused("no table".into())),
	}
}

fn parse<T: DeserializeOwned>(args: serde_json::Value) -> Result<T, EngineError> {
	serde_json::from_value(args).map_err(|e| EngineError::Shape(e.to_string()))
}

fn json<T: Serialize>(t: &T) -> Result<serde_json::Value, EngineError> {
	serde_json::to_value(t).map_err(|e| EngineError::Shape(e.to_string()))
}

/// A pump's move by the door's index (from 0) as the driver counts it (from 1); an index no pump can hold is refused.
fn from_one(index: usize) -> Result<usize, EngineError> {
	index.checked_add(1).ok_or_else(|| EngineError::Refused(format!("the pump has no move {index}")))
}

/// A judge asked of a move of a match pump's own (slice 61), whose result `finish` writes as an app row.
fn own_move(index: usize) -> EngineError {
	EngineError::Refused(format!("move {index} is the pump's own: finish writes it"))
}

impl Host {
	/// A Lua 5.1 state with the base, math, string and table libraries (mlua always opens the base library), no file
	/// access (`dofile` and `loadfile` gone) and nothing printed; `tools/host.lua`'s environment on `clock` with
	/// `report` as the files' error handler; the pure files loaded in their order, each chunk named by its path, as the
	/// client names an addon's; `tools/apppump.lua` on them.
	pub(crate) fn start(clock: Clock, report: Box<dyn Fn(String) + Send>) -> Result<Self, EngineError> {
		let started = Instant::now();
		let lua = mlua::Lua::new_with(
			StdLib::MATH | StdLib::STRING | StdLib::TABLE,
			LuaOptions::new().catch_rust_panics(false),
		)?;
		lua.set_memory_limit(MEMORY_LIMIT)?;
		let globals = lua.globals();
		for name in ["dofile", "loadfile", "print"] {
			globals.raw_set(name, Value::Nil)?;
		}
		let host: Table = lua.load(HOST).set_name("@tools/host.lua").eval()?;
		let Clock { now, offset_at } = clock;
		let mut offsets = Offsets { at: offset_at, last: None };
		let opts = lua.create_table()?;
		opts.set("now", lua.create_function(move |_, ()| Ok(now()))?)?;
		opts.set("offset", lua.create_function_mut(move |_, t: i64| Ok(offsets.at(t)))?)?;
		opts.set(
			"report",
			lua.create_function(move |_, err: Value| {
				report(err.to_string()?);
				Ok(())
			})?,
		)?;
		let env: Table = host.call_function("env", opts)?;
		let read = lua.create_function(|_, path: String| {
			FILES
				.iter()
				.find(|(name, _)| *name == path)
				.map(|(_, source)| *source)
				.ok_or_else(|| mlua::Error::runtime(format!("{path} is not embedded")))
		})?;
		let ns: Table = host.call_function("load", (env.clone(), read, lua.create_table()?))?;
		let driver: Function = lua.load(APPPUMP).set_name("@tools/apppump.lua").set_environment(env).eval()?;
		let mark = lua.create_function(|lua, list: Table| {
			list.set_metatable(Some(lua.array_metatable()))?;
			Ok(list)
		})?;
		let marks = lua.create_table()?;
		marks.set("array", mark)?;
		let app: Table = driver.call((ns.clone(), marks))?;
		log::info!(
			"engine: {} files and the app pump's driver loaded in {} ms, {} KiB, digest {}",
			FILES.len(),
			started.elapsed().as_millis(),
			lua.used_memory() / 1024,
			crate::DIGEST
		);
		Ok(Self { lua, app, ns, player: None, pump: None, shown: None, game_save: None })
	}

	/// Answers each job until every [`crate::Engine`] is dropped.
	#[cfg(not(target_os = "emscripten"))]
	pub(crate) fn serve(mut self, jobs: &Receiver<Job>) {
		while let Ok(Job { op, args, reply }) = jobs.recv() {
			let answer = self.answer(&op, args);
			if reply.send(answer).is_err() {
				log::debug!("engine: {op}'s caller left before its answer");
			}
		}
	}

	fn encode<T: Serialize>(&self, t: &T) -> Result<Value, EngineError> {
		Ok(self.lua.to_value_with(t, to_lua_options())?)
	}

	fn decode<T: DeserializeOwned>(&self, v: Value) -> Result<T, EngineError> {
		Ok(self.lua.from_value(v)?)
	}

	fn player(&self) -> Result<&Table, EngineError> {
		self.player.as_ref().ok_or_else(|| EngineError::Refused("no player yet: init comes first".into()))
	}

	fn pump(&self) -> Result<&Table, EngineError> {
		self.pump.as_ref().ok_or_else(|| EngineError::Refused("no pump on screen".into()))
	}

	/// The pump's slot at `index` (from 0).
	fn slot(&self, index: usize) -> Result<Table, EngineError> {
		let slots: Table = self.pump()?.get("slots")?;
		slots
			.get::<Option<Table>>(from_one(index)?)?
			.ok_or_else(|| EngineError::Refused(format!("the pump has no move {index}")))
	}

	/// The pump's move at `index` (from 0) as it stands, a catch-up's after Swap too (whose slots stand at their places):
	/// its slot, the move as a device runs it and whether Swap put it there; none for a move the pump lacks and a
	/// catch-up's move that runs by its time, which holds no slot.
	fn dosed(&self, index: usize) -> Result<Option<Dosed>, EngineError> {
		let i = from_one(index)?;
		let slots: Table = self.pump()?.get("slots")?;
		let Some(slot) = slots.get::<Option<Table>>(i)? else {
			return Ok(None);
		};
		let slot: Slot = self.decode(self.app.call_function("slotJSON", slot)?)?;
		let mut planned: PlannedMove = self.decode(self.pump()?.call_method("move", (self.player()?, i))?)?;
		planned.slot = index;
		let swapped = self.pump()?.get::<Table>("swapped")?.contains_key(i)?;
		Ok(Some(Dosed { index, slot, planned, swapped }))
	}

	/// A list of slots in the contract's shape (`A.slotsJSON`).
	fn slots(&self, slots: Table) -> Result<Vec<Slot>, EngineError> {
		self.decode(self.app.call_function("slotsJSON", slots)?)
	}

	/// Row texts as the driver takes them: a sequence, nil for none (a pump no match sized carries no rider).
	fn texts(&self, rows: Vec<String>) -> Result<Value, EngineError> {
		if rows.is_empty() { Ok(Value::Nil) } else { Ok(Value::Table(self.lua.create_sequence_from(rows)?)) }
	}

	/// The pump on screen as the door shows it.
	fn shown_pump(&self) -> Result<Pump, EngineError> {
		let pump = self.pump()?;
		// a match pump's riders (A.ridersJSON), their places and their rows' places from 1 in the driver
		let mut riders: Vec<Rider> = self.decode(self.app.call_function("ridersJSON", pump)?)?;
		for r in &mut riders {
			let shape = || EngineError::Shape("ridersJSON: a place at 0".into());
			(r.index, r.row) = (r.index.checked_sub(1).ok_or_else(shape)?, r.row.checked_sub(1).ok_or_else(shape)?);
		}
		// each swapped move's slot it replaced, which its way back puts back
		let mut swapped: Vec<usize> = Vec::new();
		pump.get::<Table>("swapped")?.for_each(|i: usize, _: Table| {
			swapped.push(i - 1);
			Ok(())
		})?;
		swapped.sort_unstable();
		Ok(Pump {
			id: pump.get("id")?,
			kind: pump.get("kind")?,
			family: pump.get("family")?,
			tier: pump.get("tier")?,
			slots: self.slots(pump.get("slots")?)?,
			swapped,
			riders,
		})
	}

	/// A pump composed by `A.compose`, `A.quick` or `A.pick`, now the one on screen.
	fn composed(&mut self, made: (Table, Value)) -> Result<Composed, EngineError> {
		let (pump, rows) = made;
		self.pump = Some(pump);
		self.shown = None;
		Ok(Composed { pump: self.shown_pump()?, rows: self.decode(rows)? })
	}

	/// The entry of Correct a count's window for the set `key` names at `at`, its sets after `horizon`
	/// (`Engine:correctSet`, as the driver takes it for `correctCheck` and `correctRows`); none once the set has left it.
	fn correct_set(&self, key: &SetKey, at: i64, horizon: Option<i64>) -> Result<Option<Table>, EngineError> {
		Ok(self.player()?.call_method("correctSet", (self.encode(key)?, at, horizon))?)
	}

	/// What a Start of `target` starts at `at` (`A.start`), as the driver answers it and in the door's shape.
	fn start_of(&self, target: Target, at: i64) -> Result<(Value, Start), EngineError> {
		let start: Value = self.app.call_function("start", (self.player()?, self.encode(&target)?, at))?;
		let read = self.decode(start.clone())?;
		Ok((start, read))
	}

	/// One op of the door.
	#[expect(clippy::too_many_lines, reason = "one arm per op of the door, each a call of the driver")]
	pub(crate) fn answer(&mut self, op: &str, args: serde_json::Value) -> Result<serde_json::Value, EngineError> {
		let lua = &self.lua;
		match op {
			"init" => {
				let a: api::Init = parse(args)?;
				let profile: Table = self.app.call_function("profile", self.encode(&a.profile)?)?;
				let o = lua.create_table()?;
				o.set("stamp", a.stamp)?;
				self.player = Some(self.app.call_function("new", (profile, o))?);
				self.pump = None;
				self.shown = None;
				Ok(serde_json::Value::Null)
			}
			"profile" => {
				let a: api::SetProfile = parse(args)?;
				let profile: Table = self.app.call_function("profile", self.encode(&a.profile)?)?;
				self.player()?.set("profile", profile)?;
				Ok(serde_json::Value::Null)
			}
			"fold" => {
				let a: api::Fold = parse(args)?;
				let started = Instant::now();
				let (own, game) = (a.own.len(), a.game.len());
				let folded: Value = self.player()?.call_method(
					"fold",
					(lua.create_sequence_from(a.own)?, lua.create_sequence_from(a.game)?, a.sheet_focus),
				)?;
				let folded: Folded = self.decode(folded)?;
				log::info!(
					"engine: folded {own} own and {game} game rows in {} ms, {} KiB",
					started.elapsed().as_millis(),
					lua.used_memory() / 1024
				);
				json(&folded)
			}
			"merge" => {
				let a: api::Merge = parse(args)?;
				let started = Instant::now();
				let (own, game) = (a.own.len(), a.game.len());
				let how: Merged = self.decode(
					self.player()?
						.call_method("merge", (lua.create_sequence_from(a.own)?, lua.create_sequence_from(a.game)?))?,
				)?;
				log::debug!(
					"engine: merged {own} own and {game} game rows ({how:?}) in {} ms",
					started.elapsed().as_millis()
				);
				json(&how)
			}
			"gameFlags" => {
				let flags: Vec<u8> = self.decode(self.player()?.call_method("gameFlags", ())?)?;
				json(&flags)
			}
			"ridesAs" => {
				let a: api::RowsAt = parse(args)?;
				let rides: Vec<RidesAs> =
					self.decode(self.player()?.call_method("ridesAs", (lua.create_sequence_from(a.rows)?, a.at))?)?;
				json(&rides)
			}
			"startRows" => {
				let a: api::At = parse(args)?;
				let rows: Vec<String> = self.decode(self.player()?.call_method("startRows", a.at)?)?;
				json(&rows)
			}
			"painRows" => {
				let a: api::Pain = parse(args)?;
				let rows: Vec<String> = self.decode(
					self.player()?.call_method("painRows", (lua.create_sequence_from(a.joints)?, a.on, a.at))?,
				)?;
				json(&rows)
			}
			"checkinRow" => {
				let a: api::Code = parse(args)?;
				let row: String = self.player()?.call_method("checkinRow", (a.code, a.at))?;
				json(&row)
			}
			"focusRow" => {
				let a: api::Code = parse(args)?;
				let row: String = self.player()?.call_method("focusRow", (a.code, a.at))?;
				json(&row)
			}
			"clearRow" => {
				let a: api::ClearRow = parse(args)?;
				let row: String = self.player()?.call_method("clearRow", (self.encode(&a.scope)?, a.id, a.at))?;
				json(&row)
			}
			"checkinDue" => {
				let a: api::At = parse(args)?;
				let due: bool = self.player()?.call_method("checkinDue", a.at)?;
				json(&due)
			}
			"todayXp" => {
				let a: api::At = parse(args)?;
				let xp: i64 = self.player()?.call_method("todayXp", a.at)?;
				json(&xp)
			}
			"levels" => {
				let a: api::Xps = parse(args)?;
				let levels: Vec<LevelAt> =
					self.decode(self.player()?.call_method("levels", lua.create_sequence_from(a.xps)?)?)?;
				json(&levels)
			}
			"locked" => {
				let a: api::At = parse(args)?;
				let (locked, unwell): (bool, Option<i64>) = self.player()?.call_method("locked", a.at)?;
				json(&Locked { locked, unwell })
			}
			"painFlags" => {
				let mut joints: Vec<u8> = Vec::new();
				self.player()?.get::<Table>("state")?.get::<Table>("pain")?.for_each(|joint: u8, _: Value| {
					joints.push(joint);
					Ok(())
				})?;
				joints.sort_unstable();
				json(&joints)
			}
			"records" => {
				let a: api::MovesAt = parse(args)?;
				let records: Vec<Record> =
					self.decode(self.player()?.call_method("records", (lua.create_sequence_from(a.moves)?, a.at))?)?;
				json(&records)
			}
			"rungs" => {
				let a: api::Ladders = parse(args)?;
				let rungs: Vec<Rung> =
					self.decode(self.player()?.call_method("rungs", lua.create_sequence_from(a.ladders)?)?)?;
				json(&rungs)
			}
			"open" => {
				let a: api::Ladders = parse(args)?;
				let open: Vec<Open> =
					self.decode(self.player()?.call_method("open", lua.create_sequence_from(a.ladders)?)?)?;
				json(&open)
			}
			"allowed" => {
				let a: api::Moves = parse(args)?;
				let allowed: Vec<Allowed> =
					self.decode(self.player()?.call_method("allowed", lua.create_sequence_from(a.moves)?)?)?;
				json(&allowed)
			}
			"setupSteps" => {
				let a: api::Count = parse(args)?;
				let steps: Vec<SetupStep> = self.decode(self.player()?.call_method("setupSteps", a.count)?)?;
				json(&steps)
			}
			"setupOpens" => {
				let opens: Vec<SetupStep> = self.decode(self.player()?.call_method("setupOpens", ())?)?;
				json(&opens)
			}
			"ladderOpens" => {
				// what one more piece of the setup, a switch among them, opens on each ladder (Engine:ladderOpens over
				// Progress.lua ns.SetupOpens with the ladder, its quality on the screens' scale there, as Your setup's)
				let a: api::Ladders = parse(args)?;
				let opens: Vec<Vec<SetupStep>> =
					self.decode(self.player()?.call_method("ladderOpens", lua.create_sequence_from(a.ladders)?)?)?;
				json(&opens)
			}
			"summary" => {
				let a: api::At = parse(args)?;
				self.decode(self.player()?.call_method("summary", a.at)?)
			}
			"gameSave" => {
				let a: api::GameSaveAt = parse(args)?;
				// the same save again keeps its table, and with it the fold gameSummary holds
				if self.game_save.as_ref().map(|(_, text)| text) != a.save.as_ref() {
					self.game_save = match a.save {
						Some(text) => Some((save_table(lua, &text)?, text)),
						None => None,
					};
				}
				Ok(serde_json::Value::Null)
			}
			"gameSummary" => {
				let a: api::GameAt = parse(args)?;
				let save = self.game_save.as_ref().map(|(t, _)| t.clone());
				let sheet: Value = self.player()?.call_method("gameSummary", (a.at, a.reset, save))?;
				match sheet {
					Value::Nil => Ok(serde_json::Value::Null),
					sheet => json(&api::GameSummary { sheet: self.decode(sheet)? }),
				}
			}
			"feedbackNumbers" => {
				let a: api::GameAt = parse(args)?;
				// the game's own sheet as gameSummary folds it (once after rows came or a save, else the one it holds)
				let save = self.game_save.as_ref().map(|(t, _)| t.clone());
				let sheet: Value = self.player()?.call_method("gameSummary", (a.at, a.reset, save))?;
				if sheet.is_nil() {
					return Ok(serde_json::Value::Null);
				}
				let game: Table = self.player()?.get("gameSheet")?;
				let numbers: Value = self.ns.get::<Function>("FeedbackNumbers")?.call((game, a.at))?;
				json(&self.decode::<api::FeedbackNumbers>(numbers)?)
			}
			"book" => {
				let a: api::BookAt = parse(args)?;
				let game = a.game.as_ref().map(|g| self.encode(g)).transpose()?;
				let book: api::Book = self.decode(self.player()?.call_method("book", (game, a.at))?)?;
				json(&book)
			}
			"titles" => {
				let codes: Vec<u8> = self.decode(self.player()?.call_method("titles", ())?)?;
				json(&codes)
			}
			"nextGoal" => {
				let a: api::NextGoal = parse(args)?;
				let goal: i64 = self.player()?.call_method("nextGoal", (a.r#move, a.value))?;
				json(&goal)
			}
			"nextLoad" => {
				let a: api::NextLoadAt = parse(args)?;
				let next: Option<NextLoad> = self.decode(self.player()?.call_method("nextLoad", (a.r#move, a.at))?)?;
				json(&next)
			}
			"compose" => {
				let a: api::Compose = parse(args)?;
				// a plain kind's pump has no `chosen` move (A.pick's); its `match` facts, or nil; a match pump's
				// unfinished moves (slice 61), or nil
				let r#match = a.r#match.map(|m| self.encode(&m)).transpose()?.unwrap_or(Value::Nil);
				let (owed, queue) = (self.texts(a.unfinished.owed)?, self.texts(a.unfinished.queue)?);
				let made = self.app.call_function(
					"compose",
					(self.player()?, a.id, a.kind, a.family, a.tier, a.at, Value::Nil, r#match, owed, queue),
				)?;
				json(&self.composed(made)?)
			}
			"quick" => {
				let a: api::Quick = parse(args)?;
				let (owed, queue) = (self.texts(a.unfinished.owed)?, self.texts(a.unfinished.queue)?);
				let made = self.app.call_function("quick", (self.player()?, a.id, a.at, owed, queue))?;
				json(&self.composed(made)?)
			}
			"expedition" => {
				let a: api::ExpeditionAsk = parse(args)?;
				let made = self.app.call_function(
					"expedition",
					(self.player()?, a.id, self.encode(&a.stage)?, self.encode(&a.regions)?, a.at),
				)?;
				json(&self.composed(made)?)
			}
			"expeditionNext" => {
				let a: api::ExpeditionAt = parse(args)?;
				let exp = api::Exp { budget: a.budget, elapsed: a.elapsed, last: a.last, regions: a.regions };
				let stage: Value =
					self.app.call_function("expeditionNext", (self.player()?, self.encode(&exp)?, a.at))?;
				json(&self.decode::<Option<api::ExpeditionStage>>(stage)?)
			}
			"expeditionPreview" => {
				let a: api::Minutes = parse(args)?;
				let preview: Value = self.app.call_function("expeditionPreview", (self.player()?, a.minutes, a.at))?;
				json(&self.decode::<api::ExpeditionPreview>(preview)?)
			}
			"pumpTime" => {
				let seconds: i64 = self.pump()?.call_method("time", ())?;
				json(&seconds)
			}
			"pumpRegions" => {
				let a: api::Regions = parse(args)?;
				let regions: Value = self.pump()?.call_method("regions", self.encode(&a.regions)?)?;
				json(&self.decode::<Vec<String>>(regions)?)
			}
			"move" => {
				// the start read again as the player presses Start, so the pump is the one the engine gives now, and
				// composed only where it starts something (A.pick takes the start as it is)
				let a: api::Chosen = parse(args)?;
				let (raw, start) = self.start_of(a.target, a.at)?;
				if start.refused.is_some() || start.r#move.is_none() {
					return json(&Picked { start, pump: None });
				}
				let made = self.app.call_function("pick", (self.player()?, a.id, raw, a.at))?;
				let pump = Some(self.composed(made)?);
				json(&Picked { start, pump })
			}
			"start" => {
				let a: api::StartAt = parse(args)?;
				json(&self.start_of(a.target, a.at)?.1)
			}
			"starts" => {
				let a: api::Starts = parse(args)?;
				let starts = a
					.moves
					.into_iter()
					.map(|m| Ok(self.start_of(Target::Move { r#move: m }, a.at)?.1))
					.collect::<Result<Vec<Start>, EngineError>>()?;
				json(&starts)
			}
			"needs" => {
				// each step another ladder waits for as a card reads it (E:needs): whether it is done (Progress.lua
				// ns.NeedDone, the test ns.RungNeed, ns.MoveOpen and ns.Open walk) on the state a Start and the Book read
				// (E:startState), what it waits for (ns.NeedAt with the player's filter, ns.FilterContext, as the
				// Talents tab's need rows: the move the setup offers on that step and the record that does it), and on a
				// stretch check (an `any` group's by the choice the setup offers) its least answer and how it was passed
				// (slice 42, ns.CheckPassed)
				let a: api::NeedsAt = parse(args)?;
				let list = lua.create_sequence_from(
					a.needs.iter().map(|need| self.encode(need)).collect::<Result<Vec<Value>, EngineError>>()?,
				)?;
				let answers: Vec<NeedState> = self.decode(self.player()?.call_method("needs", (list, a.at))?)?;
				json(&answers)
			}
			"catchup" => {
				let a: api::Catchup = parse(args)?;
				let rows = lua.create_sequence_from(a.rows)?;
				let pays =
					lua.create_sequence_from(a.pays.into_iter().map(from_one).collect::<Result<Vec<_>, _>>()?)?;
				// a catch-up that fails leaves no pump on screen, never the one before it, whose slots no catch-up's
				// move may read
				(self.pump, self.shown) = (None, None);
				let (pump, asks): (Table, Value) =
					self.app.call_function("catchup", (self.player()?, a.id, rows, pays, a.at))?;
				let mut dosed: Vec<Dosed> = self.decode(asks)?;
				for d in &mut dosed {
					d.index =
						d.index.checked_sub(1).ok_or_else(|| EngineError::Shape("catchup: a move at 0".into()))?;
					d.planned.slot = d.index;
				}
				self.pump = Some(pump);
				json(&dosed)
			}
			"pump" => json(&self.shown_pump()?),
			"swapOptions" => {
				let a: api::IndexAt = parse(args)?;
				let i = from_one(a.index)?;
				let options: Option<Table> = self.pump()?.call_method("swapOptions", (self.player()?, i, a.at))?;
				let Some(options) = options else {
					return json(&None::<SwapOptions>);
				};
				let shown: SwapOptions = self.decode(self.app.call_function("optionsJSON", &options)?)?;
				self.shown = Some((i, options));
				json(&Some(shown))
			}
			"swap" => {
				let a: api::Swap = parse(args)?;
				let i = from_one(a.index)?;
				let option: Option<Table> = match &self.shown {
					Some((shown, options)) if *shown == i => options.get(match a.take {
						Take::Another => "another",
						Take::Easier => "easier",
						Take::Backup => "backup",
						Take::Back => "back",
					})?,
					_ => return Err(EngineError::Refused(format!("no Swap options shown for move {}", a.index))),
				};
				let option = option.ok_or_else(|| {
					EngineError::Refused(format!("move {}'s Swap options hold no {:?}", a.index, a.take))
				})?;
				let swapped: bool = self.pump()?.call_method("swap", (i, option))?;
				self.shown = None;
				json(&swapped)
			}
			"dosed" => {
				let a: api::Index = parse(args)?;
				json(&self.dosed(a.index)?)
			}
			"ready" => {
				// the move dosed again as its Ready view opens (Pump:ready), then as it stands
				let a: api::IndexAt = parse(args)?;
				let i = from_one(a.index)?;
				let slots: Table = self.pump()?.get("slots")?;
				if slots.contains_key(i)? {
					self.pump()?.call_method::<bool>("ready", (self.player()?, i, a.at))?;
				}
				json(&self.dosed(a.index)?)
			}
			"skip" => {
				let a: api::Index = parse(args)?;
				let skipped: bool = self.pump()?.call_method("skip", from_one(a.index)?)?;
				self.shown = None;
				json(&skipped)
			}
			"afterPain" => {
				let a: api::AfterPain = parse(args)?;
				self.pump()?.call_method::<()>("afterPain", (self.player()?, from_one(a.from)?, a.recorded, a.at))?;
				self.shown = None;
				json(&self.slots(self.pump()?.get("slots")?)?)
			}
			"validate" => {
				let a: api::Validate = parse(args)?;
				let slot = self.slot(a.index)?;
				let check: Option<Check> =
					self.decode(self.player()?.call_method("validate", (slot, a.value, a.load))?)?;
				json(&check)
			}
			"finish" => {
				let a: api::Finish = parse(args)?;
				let (i, result) = (from_one(a.index)?, self.encode(&a.result)?);
				let (rows, day, kindled, lit): (Value, Value, Option<i64>, Option<bool>) =
					self.pump()?.call_method("finish", (self.player()?, i, result, a.at))?;
				if rows == Value::Boolean(false) {
					return Err(EngineError::Refused(format!(
						"move {} rides: judge it, its payment writes it",
						a.index
					)));
				}
				json(&api::Finished {
					rows: self.decode(rows)?,
					day: self.decode(day)?,
					kindled,
					lit: lit.unwrap_or(false),
				})
			}
			"lights" => {
				let a: api::Finish = parse(args)?;
				let (i, result) = (from_one(a.index)?, self.encode(&a.result)?);
				let lights: bool = self.pump()?.call_method("lights", (self.player()?, i, result, a.at))?;
				json(&lights)
			}
			"holdAsk" => {
				let a: api::HoldAskAt = parse(args)?;
				let asks: bool =
					self.pump()?.call_method("holdAsk", (self.player()?, from_one(a.index)?, a.value, a.at))?;
				json(&asks)
			}
			"embers" => {
				let a: api::At = parse(args)?;
				let embers: api::Embers = self.decode(self.player()?.call_method("embers", a.at)?)?;
				json(&embers)
			}
			"judge" => {
				let a: api::Finish = parse(args)?;
				let (i, result) = (from_one(a.index)?, self.encode(&a.result)?);
				let (set, down): (Value, Value) =
					self.pump()?.call_method("judge", (self.player()?, i, result, a.at))?;
				if set == Value::Boolean(false) {
					return Err(own_move(a.index));
				}
				json(&Judgement { set: self.decode(set)?, down: self.decode(down)? })
			}
			"emptyDone" => {
				let a: api::IndexAt = parse(args)?;
				let down: Value = self.pump()?.call_method("emptyDone", (self.player()?, from_one(a.index)?, a.at))?;
				if down == Value::Boolean(false) {
					return Err(own_move(a.index));
				}
				json(&self.decode::<Option<JudgedStep>>(down)?)
			}
			"sweep" => {
				// the held climbs' undo rows and the mastered rows the state lacks (Engine:sweep), written
				let a: api::At = parse(args)?;
				let rows: Vec<String> = self.decode(self.player()?.call_method("sweep", a.at)?)?;
				json(&rows)
			}
			"settle" => {
				let folded: bool = self.player()?.call_method("settle", ())?;
				json(&folded)
			}
			"correctable" => {
				// Correct a count's window (Engine:correctable over Progress.lua ns.Correctable), as the contract
				// carries its sets (A.correctableJSON)
				let a: api::CorrectableAt = parse(args)?;
				let found: Table = self.player()?.call_method("correctable", (a.r#move, a.at, a.horizon))?;
				let sets: Vec<CorrectSet> = self.decode(self.app.call_function("correctableJSON", found)?)?;
				json(&sets)
			}
			"correctCheck" => {
				// the set read again in the window, then its check (Engine:correctCheck, ns.CorrectionCheck)
				let a: api::CorrectCheckAt = parse(args)?;
				let Some(entry) = self.correct_set(&a.set, a.at, a.horizon)? else {
					return json(&CorrectionCheck::Gone);
				};
				let asked: Option<CorrectCheck> =
					self.decode(self.player()?.call_method("correctCheck", (entry, a.value, a.load))?)?;
				json(&asked.map_or(CorrectionCheck::Clear, |check| CorrectionCheck::Asks { check }))
			}
			"correctRows" => {
				// the set read again in the window, then its rows written and the ring folded again
				// (Engine:correctRows, ns.CorrectionRows)
				let a: api::CorrectRowsAt = parse(args)?;
				let Some(entry) = self.correct_set(&a.set, a.at, a.horizon)? else {
					return json(&Corrected::Gone);
				};
				let (rows, stays, now, steps): (Value, Value, Value, Value) =
					self.player()?.call_method("correctRows", (entry, a.value, a.load, a.answer, a.at))?;
				if rows.is_nil() {
					return Err(EngineError::Refused(format!("the set {:?} takes no such correction", a.set)));
				}
				json(&Corrected::Written {
					rows: self.decode(rows)?,
					stays: self.decode(stays)?,
					now: self.decode(now)?,
					steps: self.decode(steps)?,
				})
			}
			"stop" => {
				let a: api::At = parse(args)?;
				let rows: Vec<String> = self.decode(self.pump()?.call_method("stop", (self.player()?, a.at))?)?;
				json(&rows)
			}
			"moves" => {
				let pump = self.pump()?;
				let mut moves: Vec<PlannedMove> = Vec::new();
				let slots: Table = pump.get("slots")?;
				for i in 1..=slots.raw_len() {
					let mut planned: PlannedMove = self.decode(pump.call_method("move", (self.player()?, i))?)?;
					planned.slot = i - 1;
					moves.push(planned);
				}
				json(&moves)
			}
			"toward" => {
				let a: api::Index = parse(args)?;
				let toward: Option<Toward> =
					self.decode(self.pump()?.call_method("toward", (self.player()?, from_one(a.index)?))?)?;
				json(&toward)
			}
			"checks" => {
				let a: api::ChecksAt = parse(args)?;
				let started = a.started.as_ref().map(|s| self.encode(s)).transpose()?;
				let checks: usize =
					self.decode(self.pump()?.call_method("checks", (self.player()?, from_one(a.index)?, started))?)?;
				json(&checks)
			}
			"checked" => {
				let a: api::Index = parse(args)?;
				let outcomes: Vec<CheckOutcome> =
					self.decode(self.pump()?.call_method("checked", (self.player()?, from_one(a.index)?))?)?;
				json(&outcomes)
			}
			"callout" => {
				let a: api::CalloutAt = parse(args)?;
				let i = from_one(a.index)?;
				let family: Option<String> = match a.count {
					Some(n) => {
						let place = a.place.map(from_one).transpose()?;
						let pump = self.pump.clone();
						self.app.call_function(
							"catchupCallout",
							(self.player()?, pump, place, i, n, a.at, a.before, a.after),
						)?
					}
					None => self.pump()?.call_method("callout", (self.player()?, i, a.at, a.before, a.after))?,
				};
				json(&family)
			}
			"preview" => {
				let a: api::At = parse(args)?;
				let previews: Vec<Preview> =
					self.decode(self.pump()?.call_method("preview", (self.player()?, a.at))?)?;
				json(&previews)
			}
			_ => Err(EngineError::Refused(format!("no op {op}"))),
		}
	}
}
