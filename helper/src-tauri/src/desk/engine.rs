//! The addon's own engine in the desktop app (docs/plan.md slice 35, "The engine is the addon's own Lua";
//! `questpump-engine`): it composes, judges and folds every pump this computer runs, from the one progression of every
//! row the helper keeps. Its thread starts with the helper, and the engine there is a new player's (`init`) with this
//! computer's profile (`Helper::engine_profile`: the body every app device shares and this computer's place) and the
//! computer's app rows' bits (`STAMP`, 6144), then folds once, off the loop, every device's app rows (`app.jsonl`)
//! beside the game's rows as `GET /v1/history` serves them, with the shown sheet's focus as the phones fold it. Rows
//! from elsewhere then merge in, in the order the helper took them (a phone's or a watch's app rows, the game's rows
//! the strip brings, rows taken in from a save, a catch-up's), never a row this engine wrote
//! (`Session::keep_computer`'s, the app rows with a pump id); a merge that fails folds every row again, and so do a
//! reset (both stores start again) and a new focus on the shown sheet. A fold that failed stays failed until a reset,
//! which starts the engine again (`init` with the profile then, and the fold). A changed body or place gives the
//! engine the profile again (`Helper::profile_changed`). The `shell` topic says `engine`: `busy` while a fold runs,
//! then `ready` or `failed` (with a log line); the report for help names the engine's `DIGEST` and the last fold's rows
//! and time.
//!
//! **The door.** Every engine call from the loop goes through `Helper::engine_call(call, then)`: `call` runs on the
//! door's thread with the engine, after every call asked before it, and `then` runs back on the loop with its answer,
//! as a message the loop's select takes in turn (`Door::answer`). So answers come back in the order the calls were
//! asked (a pump's compose before its finish, a merge before the compose after it), and the loop never waits on the
//! engine, but once: before this process goes (a quit, a restart, an update), `Helper::engine_settle` waits up to
//! `SETTLE` for every call asked so far and runs their answers, so the rows a pump's last calls wrote are kept first.
//! The calls answer in a few milliseconds (`summary`, `compose`, `quick`, `finish`, `records` and the other reads and
//! writes of `questpump_engine::Engine`; an in-order merge in about 1 ms, a ring merge in about 50 ms), except `fold`:
//! 1 to 2 s at 30,000 to 50,000 rows, which the door runs itself (at the start and for a refold, as `init` and
//! `merge`), never an area. While a fold runs (`!Helper::engine_ready()`), a call waits behind it: an area starts no
//! pump and answers no view from the engine until it is ready. A merge that has to fold every row itself (a game row at
//! or before the kept ring's horizon, a correction or an undo, the game's or since slice 44 a phone's; a save taken in
//! brings those) takes as long, the engine ready all the while: the calls behind it answer late, and the loop still
//! never waits. Rows the engine writes (`compose`, `quick`, `finish`, `stop`, `pain_rows`, `focus_row`, `checkin_row`,
//! `start_rows`, and Correct a count's `correct_rows`, which folds its ring again itself) go to
//! `Session::keep_computer` as written, already stamped; a desktop catch-up's rows are the session's
//! (`Session::catch_up`), never the engine's, and merge in as the game's: the engine doses its moves (`catch_up`) and
//! judges its counts (`judge`) writing nothing, and at its end folds the judged counts out of its state again from its
//! ring (`settle`, a merge's tens of milliseconds) before the session's rows merge in. A call that panics on the door's
//! thread is answered `EngineError::Panicked` (and logged, never a crash report: `crate::crash::caught`), and the calls
//! behind it still run.
//!
//! **The fold's answer.** `Helper::engine_folded()` is the engine's [`Folded`] as it stands: none before the first fold
//! lands and after a fold failed, else `game_flags` (the joints the game's own rows flag, ascending `ns.Enum.joint`
//! ids: the game's save's pain flags). Every fold sets it and every merge keeps it current: the same door job asks
//! `gameFlags` after the merge (`Engine::game_flags`). The engine's own rows reach it once the session keeps them
//! (`Session::keep_computer`'s effects through `Helper::apply`: a merge of no rows then answers what the engine's
//! writes changed). Its readers: Pump home's check-in window names a joint flagged in `game_flags` "flagged in the
//! game" (the phones' `JointPicker`), and DESK-MENU's Your setup does the same in its pain list (the phones'
//! `PainSection`); none runs a call for them (a pump's Something hurts lists only the joints not flagged). Your setup's experience lock is the summary's (any
//! ladder holds a rung, the phones' `FreeMode.started`). Each fold and merge that lands asks Your setup's look again
//! (`Helper::setup_look`), so its pain flags, focus and start rungs follow every row. A view that needs the flags at
//! the instant of a call asks `Engine::game_flags` through `Helper::engine_call`.

use std::sync::mpsc as sync_mpsc;
use std::time::{Duration, Instant};

use questpump_core::body::{Changed, DEFAULT_EXPERIENCE, DEFAULT_PACE};
use questpump_core::codes::flags;
use questpump_core::protocol::RowRecord;
use questpump_core::row::Row;
use questpump_engine::{Clock, Engine, EngineError, Exclusions, Folded, Profile};
use tokio::sync::{mpsc, oneshot};

use super::seams::StoreMark;
use crate::lan;
use crate::runtime::{Helper, local_offset_at};
use crate::ui::{self, EngineState, Fold};

/// The device bits of every row the engine writes: the app's and the computer's (`ns.Enum.flagBit`).
const STAMP: u16 = flags::APP | flags::COMPUTER;
/// The longest a quit, a restart or an update waits for the calls asked of the door before it (`engine_settle`): a
/// call answers in milliseconds, a fold in 1 to 2 s at 50,000 rows, so the calls behind a fold still answer in time.
const SETTLE: Duration = Duration::from_secs(3);

/// A call on the door's thread, with the engine (none when it did not start).
type Job = Box<dyn FnOnce(Option<&Engine>) + Send>;

/// An answer back on the loop.
pub(crate) type Back = Box<dyn FnOnce(&mut Helper) + Send>;

/// The engine's door: the thread that owns the order of its calls, where the answers go, and what the engine holds.
pub(crate) struct Door {
	/// The door's thread's inbox; none when the thread did not start (every call answers `Stopped`).
	jobs: Option<sync_mpsc::Sender<Job>>,
	back: mpsc::UnboundedSender<Back>,
	/// The answers, which the loop's select takes (`answer`).
	answers: mpsc::UnboundedReceiver<Back>,
	state: EngineState,
	/// A fold was asked and its answer has not come back (the state is `Busy` meanwhile).
	folding: bool,
	/// How far the engine has the stores (`Helper::store_mark` as its last fold or merge read them).
	seen: StoreMark,
	/// The shown sheet's focus the last fold took.
	focus: Option<u8>,
	/// The profile the engine was given last; none before its start.
	profile: Option<Profile>,
	/// The fold's answer as the last fold or merge left it (`Helper::engine_folded`).
	folded: Option<Folded>,
}

impl Door {
	/// The door's thread, which starts the engine (on the computer's clock and time zone, its reports in the log) and
	/// then runs each call in the order asked, with the channel its answers reach the loop by.
	pub(crate) fn start() -> Self {
		let (back, answers) = mpsc::unbounded_channel::<Back>();
		let (jobs, inbox) = sync_mpsc::channel::<Job>();
		let spawned = std::thread::Builder::new().name("engine-door".into()).spawn(move || {
			let clock =
				Clock { now: Box::new(|| lan::now_ms().div_euclid(1_000)), offset_at: Box::new(local_offset_at) };
			let engine = match Engine::spawn(clock, Box::new(|report| log::warn!("engine: {report}"))) {
				Ok(engine) => Some(engine),
				Err(e) => {
					log::error!("engine: it did not start: {e}");
					None
				}
			};
			for job in inbox {
				job(engine.as_ref());
			}
		});
		let jobs = match spawned {
			Ok(_) => Some(jobs),
			Err(e) => {
				log::error!("engine: no thread for its door: {e}");
				None
			}
		};
		Self {
			jobs,
			back,
			answers,
			state: EngineState::Busy,
			folding: false,
			seen: StoreMark::default(),
			focus: None,
			profile: None,
			folded: None,
		}
	}

	/// The next answer, for the loop's select to run on the helper (the door keeps a sender, so one always comes).
	pub(crate) async fn answer(&mut self) -> Option<Back> {
		self.answers.recv().await
	}

	/// A call onto the door's thread; one the thread can no longer take is answered at once without the engine.
	fn send(&self, job: Job) {
		match &self.jobs {
			Some(jobs) => {
				if let Err(sync_mpsc::SendError(job)) = jobs.send(job) {
					job(None);
				}
			}
			None => job(None),
		}
	}
}

/// Lua's traceback in the log, after the line that names an error the engine's Lua raised.
fn traceback(e: &EngineError) {
	if let EngineError::Lua { traceback, .. } = e
		&& !traceback.is_empty()
	{
		log::warn!("engine: {traceback}");
	}
}

/// Rows as the engine reads them, the addon's grammar.
fn texts(rows: &[Row]) -> Vec<String> {
	rows.iter().map(ToString::to_string).collect()
}

impl Helper {
	/// The door: `call` runs on the door's thread with the engine, after every call asked before it, and `then` runs on
	/// the loop with its answer (`EngineError::Stopped` when the engine did not start or has stopped,
	/// `EngineError::Panicked` when `call` panicked). Never the engine's `init`, `fold` or `merge`, which the door runs
	/// itself.
	pub(crate) fn engine_call<T, C, F>(&self, call: C, then: F)
	where
		T: Send + 'static,
		C: FnOnce(&Engine) -> Result<T, EngineError> + Send + 'static,
		F: FnOnce(&mut Helper, Result<T, EngineError>) + Send + 'static,
	{
		let back = self.engine.back.clone();
		self.engine.send(Box::new(move |engine: Option<&Engine>| {
			let answer = match engine {
				// the calls behind a panicking one still run, and its `then` hears it; the panic is logged, never a
				// crash report (`crate::crash::caught`)
				Some(engine) => crate::crash::caught(move || call(engine)).unwrap_or_else(|payload| {
					let message = questpump_engine::panic_text(&*payload);
					log::error!("engine: a call panicked: {message}");
					Err(EngineError::Panicked(message))
				}),
				None => Err(EngineError::Stopped),
			};
			let _ = back.send(Box::new(move |helper: &mut Helper| then(helper, answer)));
		}));
	}

	/// Before this process goes (a quit, a restart, an update): every call asked of the door so far answered and its
	/// `then` run here in its order, as the loop would have run it, so the rows a pump's last calls wrote (a move's
	/// result, the felt-unwell stop) are kept before the writes are flushed; the calls those answers ask in turn too,
	/// all within `SETTLE`, after which what is still under way is lost (logged).
	pub(crate) async fn engine_settle(&mut self) {
		let until = tokio::time::Instant::now() + SETTLE;
		loop {
			let (done, landed) = oneshot::channel();
			self.engine.send(Box::new(move |_| {
				let _ = done.send(());
			}));
			if tokio::time::timeout_at(until, landed).await.is_err() {
				log::warn!("engine: calls still under way after {} s, their answers lost", SETTLE.as_secs());
				return;
			}
			let mut ran = false;
			while let Ok(back) = self.engine.answers.try_recv() {
				back(self);
				ran = true;
			}
			if !ran {
				return;
			}
		}
	}

	/// The engine has folded every row and no fold runs: a call answers in milliseconds.
	pub(crate) fn engine_ready(&self) -> bool {
		self.engine.state == EngineState::Ready
	}

	/// The fold's answer as it stands (the module's "The fold's answer"): the game's own pain flags and whether an app
	/// device set the rungs; none before the first fold lands and after a fold failed.
	pub(crate) fn engine_folded(&self) -> Option<&Folded> {
		self.engine.folded.as_ref()
	}

	/// The favourites every device shares (the body's since 2026-10-09: the Book's Favourite here and on the phones);
	/// before any device changed them, the ones this computer's Book marked before they were shared
	/// (`settings.favourites`).
	pub(crate) fn favourites(&self) -> Vec<u16> {
		self.body.body.favourites.as_ref().map_or_else(|| self.settings.favourites.clone(), |f| f.value.clone())
	}

	/// The title chosen on Hero for the app's own sheet, every device's (the body's since 2026-10-09; empty there for
	/// none); before any device chose one, the one this computer chose before titles were shared (`settings.title`).
	pub(crate) fn title(&self) -> Option<String> {
		match &self.body.body.title {
			Some(t) => Some(t.value.clone()).filter(|key| !key.is_empty()),
			None => self.settings.title.clone(),
		}
	}

	/// This computer's engine profile: the body every device shares (`body.json`: the Body switches, the experience, the
	/// moves left out and the opt-in moves checked, "Moves per match"; off, `none`, none and Balanced while no device
	/// changed them, the phones' defaults; `favourites`, `title`) and the place in use on this computer
	/// (`Helper::place_in_use`). "Moves per match" goes in only when it is not Balanced (`matchPace`, slice 48), and sizes
	/// only a pump composed with a game's end's facts (`tools/apppump.lua`); every other app pump is Balanced.
	pub(crate) fn engine_profile(&self) -> Profile {
		let ((_, place), body) = (self.place_in_use(), &self.body.body);
		let on = |switch: &Option<Changed<bool>>| switch.as_ref().is_some_and(|s| s.value);
		let moves = body.moves.as_ref().map(|m| &m.value);
		Profile {
			gear: place.gear.clone(),
			floor: place.floor,
			space: place.space,
			quiet: place.quiet,
			no_jump: on(&body.no_jump),
			no_strain: on(&body.no_strain),
			no_head_down: on(&body.no_head_down),
			experience: body.experience.as_ref().map_or(DEFAULT_EXPERIENCE, |e| e.value.as_str()).to_owned(),
			exclude: moves.map_or_else(Exclusions::default, |m| Exclusions {
				ids: m.exclude.ids.clone(),
				groups: m.exclude.groups.clone(),
			}),
			include: moves.map_or_else(Vec::new, |m| m.include.clone()),
			favourites: self.favourites(),
			title: self.title(),
			match_pace: body.match_pace.as_ref().map(|p| p.value.clone()).filter(|p| p != DEFAULT_PACE),
		}
	}

	/// The body, this computer's place, its favourites or its title changed: the engine takes the profile they make now
	/// (nothing when it is the one it has; before the engine's start its `init` takes the profile as it stands then),
	/// the Book, Your setup, Pump home and Hero read it again, and where what the pumps' filters read changed
	/// (`filtered`) the pump on its Ready view is composed again or refit after it (`setup_refilter`).
	pub(crate) fn profile_changed(&mut self) {
		if self.engine.profile.is_none() || self.engine.state == EngineState::Failed {
			return;
		}
		let profile = self.engine_profile();
		if self.engine.profile.as_ref() == Some(&profile) {
			return;
		}
		let refilter = self.engine.profile.as_ref().is_some_and(|before| filtered(before) != filtered(&profile));
		self.engine.profile = Some(profile.clone());
		self.engine_call(
			move |engine| engine.set_profile(&profile),
			|_, answer| {
				if let Err(e) = answer {
					log::warn!("engine: the profile: {e}");
					traceback(&e);
				}
			},
		);
		self.book_refresh();
		self.setup_look();
		self.home_stale();
		if refilter {
			self.setup_refilter(true);
		}
	}

	/// The engine's start, once the session is open: a new player's engine with this computer's profile and `STAMP`,
	/// then the fold of every row.
	pub(crate) fn engine_start(&mut self) {
		let profile = self.engine_profile();
		self.engine.profile = Some(profile.clone());
		self.engine_fold(Some(profile));
	}

	/// The focus of the game's sheet the app shows (`Helper::game_shown`: the newest sheet the strip brought, or the
	/// engine's fold of the game's save while none is as new as the game's rows), which the fold takes as the phones do
	/// (`sheetFocus`: the summary's `fc`, folded as the oldest focus row of all), so the pumps compose at the focus Hero
	/// names; none before a sheet names one. The save's fold merges the app's own focus rows already, so one refold
	/// leaves it as it was.
	fn sheet_focus(&self) -> Option<u8> {
		let sheet = self.session.sheet(lan::now_ms().div_euclid(1_000));
		self.game_shown(&sheet)?.summary.focus
	}

	/// The stores as the fold reads them: the game's rows with their characters as `GET /v1/history` serves them, and
	/// every app row the helper keeps, the computer's own among them, with the other computers' game rows after them in
	/// the history's numbering (slice 60: rows of another source with the app's bit, which the engine folds on the app
	/// rows' side as every mirror carries them to the game; its game side leaves an app row out).
	fn every_row(&self) -> (Vec<Row>, Vec<Row>, StoreMark) {
		let rows = self.session.history_rows(0).map(|r| r.row()).collect();
		let mut own: Vec<Row> = self.session.app_rows().records().iter().map(RowRecord::row).collect();
		own.extend(self.session.far_history(0));
		(rows, own, self.store_mark())
	}

	/// Folds every row again, off the loop (after `init` with `profile` at the start); none while one runs, whose rows
	/// were read when it was asked and whose landing takes the rows since.
	fn engine_fold(&mut self, init: Option<Profile>) {
		if self.engine.folding {
			return;
		}
		let (game, own, seen) = self.every_row();
		let focus = self.sheet_focus();
		let rows = game.len() + own.len();
		(self.engine.folding, self.engine.focus) = (true, focus);
		if self.engine.state != EngineState::Busy {
			self.engine.state = EngineState::Busy;
			ui::update(&self.app, |v| v.shell.engine = EngineState::Busy);
		}
		self.engine_call(
			move |engine| {
				if let Some(profile) = &init {
					engine.init(profile, STAMP)?;
				}
				let (own, game) = (texts(&own), texts(&game));
				let started = Instant::now();
				let folded = engine.fold(own, game, focus)?;
				Ok((folded, started.elapsed()))
			},
			move |helper, answer| helper.folded(answer, seen, rows),
		);
	}

	/// A fold came back: the engine is ready with its answer, and the rows that came while it ran merge in (the
	/// engine's own among them: a call asked before the fold wrote them into the state the fold replaced); or it failed,
	/// until a reset (its rows kept as `seen`, so only a reset after them starts it again).
	fn folded(&mut self, answer: Result<(Folded, Duration), EngineError>, seen: StoreMark, rows: usize) {
		self.engine.folding = false;
		self.engine.seen = seen;
		match answer {
			Ok((folded, took)) => {
				let ms = u64::try_from(took.as_millis()).unwrap_or(u64::MAX);
				log::info!("engine {}: {rows} rows folded in {ms} ms", questpump_engine::DIGEST);
				(self.engine.state, self.engine.folded) = (EngineState::Ready, Some(folded));
				let fold = Fold { rows, ms, at: lan::now_ms() };
				ui::update(&self.app, |v| {
					v.shell.engine = EngineState::Ready;
					v.shell.fold = Some(fold);
				});
				self.engine_merge(true, true);
				// the steps whose moves all reached their top with no mastered row yet (an old save's, a catch-up's, a
				// move left out) master now, quietly, after the merge
				self.engine_sweep();
				// the Book, the Log and Your setup read the engine again, after the merge of the rows that came
				// meanwhile
				self.book_refresh();
				self.log_refresh();
				self.setup_look();
			}
			Err(e) => {
				log::error!("engine: the fold of {rows} rows failed, no pump on this computer until a reset: {e}");
				traceback(&e);
				(self.engine.state, self.engine.folded) = (EngineState::Failed, None);
				ui::update(&self.app, |v| v.shell.engine = EngineState::Failed);
			}
		}
	}

	/// The sweep (slice 42's fix round, 2026-10-17; `questpump_engine::Engine::sweep`, tools/apppump.lua
	/// `Engine:sweep`): an undo row for each app climb today's rules would not make (Progress.lua `ns.HeldClimb`,
	/// 2026-10-24), then the mastered rows the engine's state lacks (`ns.MasterySweep`), written and kept as this
	/// computer's own (`keep_own`), quietly (no Result row, no toast: the step, the seal and the count follow the fold),
	/// as the apps sweep at their load and after a set or a catch-up; asked behind the calls asked before it, so it
	/// reads the state they leave.
	pub(crate) fn engine_sweep(&self) {
		let at = lan::now_ms().div_euclid(1000);
		self.engine_call(
			move |e| e.sweep(at),
			|h, rows| {
				if !matches!(&rows, Ok(rows) if rows.is_empty()) {
					h.keep_own("sweep", rows); // copy-lint: not shown (a log line's label)
				}
			},
		);
	}

	/// After every change the session made (`Helper::apply`): the rows from elsewhere the stores took since the engine's
	/// last look merge in; `sheets` when the game's sheet the app shows may have changed (a character's sheet came, or
	/// the engine's fold of the save answered another; its focus folds).
	pub(crate) fn engine_rows(&mut self, sheets: bool) {
		self.engine_merge(sheets, false);
	}

	/// The rows the engine lacks: a fold of every row after a reset (the stores start again; a failed engine starts
	/// again), a new focus on the shown sheet or another computer's characters named anew (slice 60: its rows kept
	/// before take their places); else the rows past `seen` merge in, in the order each store took them,
	/// the app rows this engine wrote left out (`Session::keep_computer` gives them their pump's id) unless a fold
	/// `landed` that never read them. A merge of none still runs when only those came, so the fold's answer takes what
	/// the engine's writes changed.
	fn engine_merge(&mut self, sheets: bool, landed: bool) {
		let floor = self.session.store().floor();
		match self.engine.state {
			EngineState::Ready => {}
			EngineState::Failed if floor != self.engine.seen.floor => {
				log::info!("engine: a reset since the fold that failed, the engine starts again");
				self.engine_start();
				return;
			}
			_ => return, // a fold that lands takes the rows since; a failed engine takes none until a reset
		}
		let now = self.store_mark();
		if floor != self.engine.seen.floor
			|| now.far_named != self.engine.seen.far_named
			|| ((sheets || landed) && self.sheet_focus() != self.engine.focus)
		{
			self.engine_fold(None);
			return;
		}
		let own = self.session.app_rows().records();
		let seen = self.engine.seen;
		if now.game == seen.game && now.own == seen.own && now.far == seen.far {
			return;
		}
		let game_rows: Vec<Row> = self.session.history_rows(seen.game).map(|r| r.row()).collect();
		let mut own_rows: Vec<Row> = own
			.get(seen.own..)
			.unwrap_or_default()
			.iter()
			.filter(|r| landed || r.pump.is_none())
			.map(RowRecord::row)
			.collect();
		// the other computers' on the app rows' side (`every_row`)
		own_rows.extend(self.session.far_history(seen.far));
		self.engine.seen = now;
		let n = game_rows.len() + own_rows.len();
		self.engine_call(
			move |engine| Ok((engine.merge(texts(&own_rows), texts(&game_rows))?, engine.game_flags()?)),
			move |helper, answer| match answer {
				Ok((merged, game_flags)) => {
					log::debug!("engine: {n} rows merged ({:?})", merged.how);
					helper.engine.folded = Some(Folded { game_flags });
					helper.setup_look();
				}
				Err(e) => {
					log::warn!("engine: a merge of {n} rows failed, every row folds again: {e}");
					traceback(&e);
					helper.engine_fold(None);
				}
			},
		);
	}
}

/// What of a profile makes the pump on its Ready view (the game's `filterKey`, Events.lua: the place's gear, floor
/// moves, room and quiet, the Body switches, the moves left out and the opt-in moves checked; and the experience, whose
/// start rungs a pump not started is composed from, as Your setup shows no wait for it over that pump); the favourites,
/// the title and "Moves per match" move no pump on screen.
fn filtered(p: &Profile) -> (&[String], [bool; 6], &Exclusions, &[u16], &str) {
	(
		&p.gear,
		[p.floor, p.space, p.quiet, p.no_jump, p.no_strain, p.no_head_down],
		&p.exclude,
		&p.include,
		&p.experience,
	)
}
