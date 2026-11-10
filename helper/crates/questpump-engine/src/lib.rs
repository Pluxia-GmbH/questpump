//! The addon's own engine in the desktop app (docs/plan.md slice 35, "The engine is the addon's own Lua"; rust-helper
//! `SKILL.md`, rule 2): the addon's pure files (`tools/host.lua`'s `H.FILES`: the engine, its rules, the row grammar
//! and the fold, in `QuestPump.toc`'s order), `tools/host.lua`, the game client's surface they touch, and
//! `tools/apppump.lua`, the app pump's driver that `tools/gen-engine.lua` runs to write
//! `contract/fixtures/engine.json`, all embedded at build time and run on PUC Lua 5.1.5 (mlua, `vendored`). So the
//! desktop composes, judges and folds with the game's own code at its build's commit, the pumps engine.json pins for
//! the phones' ports, with no third port to drift.
//!
//! One thread, `questpump-engine`, owns the Lua state, the player's engine and the pump on screen; an [`Engine`] (cheap
//! to clone) sends it one request at a time and waits for the answer, so an async caller calls it through
//! `tokio::task::spawn_blocking`. In the browser (`wasm32-unknown-emscripten`, docs/plan.md slice 63, "The engine"),
//! which runs it in a Web Worker of its own with no threads, [`Engine::load`] loads it on the caller's thread instead
//! and [`Engine::call`] answers there: the same Lua on the same PUC Lua 5.1.5, so the browser composes what the desktop
//! app composes. A fold of the whole history takes its time once at start (about 1.8 s for 54,000 rows
//! on David's Mac, 2026-10-08); later rows merge in ([`Engine::merge`]: in turn, or the ring folded again from the
//! baseline, in milliseconds).
//!
//! The door is [`Engine::call`]: an op's name and its arguments as JSON, the answer as JSON (the shapes below, keys in
//! the contract's camelCase, a pump's moves counted from 0). The typed helpers call it:
//!
//! | op | arguments | answer |
//! |---|---|---|
//! | `init` | `profile`, `stamp` (the device bits of every row written: the app's and the computer's here) | null |
//! | `profile` | `profile` | null |
//! | `fold` | `own`, `game` (row texts), `sheetFocus`? | [`Folded`] |
//! | `merge` | `own`, `game` | [`Merged`] |
//! | `gameFlags` | | joint ids |
//! | `ridesAs` | `rows` (owed row texts), `at` | a [`RidesAs`] for each row that rides as another move, in their order |
//! | `owedAside` | `rows` (owed row texts), `at` | the places in `rows` (from 0) of those the setup in force leaves out with no stand-in |
//! | `startRows` | `at` | row texts |
//! | `painRows` | `joints`, `on`, `at` | row texts |
//! | `checkinRow`, `focusRow` | `code`, `at` | a row text |
//! | `clearRow` | `scope` ([`Clear`]), `id`, `at` | a row text |
//! | `checkinDue` | `at` | bool |
//! | `todayXp` | `at` | the XP the app's own sheet counted on `at`'s game day, its rested bonus in it |
//! | `levels` | `xps` (lifetime XP) | a [`LevelAt`] each, in their order |
//! | `locked` | `at` | [`Locked`] |
//! | `painFlags` | | joint ids |
//! | `records` | `moves`, `at` | [`Record`]s |
//! | `rungs`, `open` | `ladders` | [`Rung`]s, [`Open`]s |
//! | `allowed` | `moves` | [`Allowed`]s |
//! | `setupSteps` | `count` | [`SetupStep`]s |
//! | `setupOpens` | | [`SetupStep`]s |
//! | `ladderOpens` | `ladders` | for each ladder, what one more piece or switch of the setup opens ([`SetupStep`]s) |
//! | `summary` | `at` | the app's own sheet summary (Stats.lua `ns.SheetSummary`, v 6) |
//! | `gameSave` | `save`? (Base64 of its CBOR) | null (the save `gameSummary` folds from, kept across `init`) |
//! | `gameSummary` | `at`, `reset`? | [`GameSummary`] (the game's own sheet as its save folds at its next load), or null |
//! | `feedbackNumbers` | `at`, `reset`? | [`FeedbackNumbers`] (Stats.lua `ns.FeedbackNumbers` of that sheet), or null |
//! | `book` | `game`? ([`GameLadders`]), `at` | [`Book`] |
//! | `titles` | | the title codes earned (`ns.Enum.title`), in the list's order |
//! | `nextGoal` | `move`, `value` | the least the next fresh set asks after a first record `value` (`ns.FreshGoal`) |
//! | `nextLoad` | `move`, `at` | a loaded move's next fresh set ([`NextLoad`], `ns.NextLoad`), or null |
//! | `compose` | `id`, `kind`, `family`?, `tier`, `at`, `match`? ([`Match`]), `work`? ([`Work`]), `owed`?, `queue`? ([`Unfinished`]) | [`Composed`] |
//! | `quick` | `id`, `at`, `owed`?, `queue`? ([`Unfinished`]) | [`Composed`] |
//! | `expedition` | `id`, `stage` ([`ExpeditionStage`]), `regions`, `at` | [`Composed`] (an expedition's round, slice 51) |
//! | `expeditionNext` | `budget`, `elapsed` (seconds), `last`? ([`ExpeditionStage`]), `regions`, `at` | the next round's [`ExpeditionStage`], or null once the expedition is done |
//! | `expeditionPreview` | `minutes`, `at` | [`ExpeditionPreview`] |
//! | `pumpTime` | | the pump on screen's estimate in seconds (`ns.PumpTime`) |
//! | `pumpRegions` | `regions` | the regions an expedition's rounds took once the pump on screen is done (`ns.RoundRegions`) |
//! | `move` | `id`, `target` ([`Target`]), `at` | [`Picked`]: the start read now and, unless it refused, its pump |
//! | `starts` | `moves`, `at` | a [`Start`] for each move's window, in their order |
//! | `start` | `target` ([`Target`]), `at` | the [`Start`] that target runs now, composing nothing |
//! | `needs` | `needs` ([`NeedEntry`]s), `at` | a [`NeedState`] each (`ns.NeedDone`, filtered `ns.NeedAt`, a check's ask and `ns.CheckPassed`), in order |
//! | `catchup` | `id`, `rows` (row texts), `pays`, `at` | [`Dosed`]s |
//! | `sweep` | `at` | row texts (the mastered rows the state lacked, `ns.MasterySweep`) |
//! | `pump` | | [`Pump`] |
//! | `swapOptions` | `index`, `at` | [`SwapOptions`] or null |
//! | `swap` | `index`, `take` | bool |
//! | `dosed` | `index` | [`Dosed`] (the pump's move at `index` as it stands, whether Swap put it there), or null (a move the pump lacks, or a catch-up's that runs by its time) |
//! | `skip` | `index` | bool (a rider at `index` left the pump and its unfinished list: a catch-up's move, or a match pump's, whose later moves move up) |
//! | `afterPain` | `from`, `recorded`, `at` | [`Slot`]s (a match pump's riders as `pump` reads them) |
//! | `validate` | `index`, `value`, `load`? | [`Check`] or null |
//! | `finish` | `index`, `result`, `at` | row texts (refused for a match pump's rider) |
//! | `judge` | `index`, `result`, `at` | [`Judgement`] (a rider's; refused for a match pump's own move) |
//! | `lights` | `index`, `result`, `at` | bool (the move's row would light an ember, `ns.LightsEmber`) |
//! | `emptyDone` | `index`, `at` | the step down its easier pick makes ([`JudgedStep`]), or null (refused as `judge`) |
//! | `settle` | | bool |
//! | `correctable` | `move`, `at`, `horizon`? | [`CorrectSet`]s (Correct a count's window, `ns.Correctable`) |
//! | `correctCheck` | `set` ([`SetKey`]), `value`, `load`?, `at`, `horizon`? | [`CorrectionCheck`] |
//! | `correctRows` | `set` ([`SetKey`]), `value`?, `load`?, `answer`?, `at`, `horizon`? | [`Corrected`] |
//! | `stop` | `at` | row texts |
//! | `moves` | | [`PlannedMove`]s |
//! | `toward` | `index` | [`Toward`], or null for a move that asks nothing or the pump lacks |
//! | `checks` | `index`, `started`? ([`StartedCheck`]) | how many checks the move's pose can pass |
//! | `checked` | `index` | a [`CheckOutcome`] each of the checks `checks` read last for that move |
//! | `preview` | `at` | a [`Preview`] for each move of the pump on screen |
//! | `callout` | `index`, `count`?, `place`?, `at`, `before`, `after`? | the move's callout's take family (`record`), or null |

#![forbid(unsafe_code)]

mod api;
mod host;

mod embedded {
	include!(concat!(env!("OUT_DIR"), "/embedded.rs"));
}

#[cfg(target_os = "emscripten")]
use std::cell::RefCell;
#[cfg(target_os = "emscripten")]
use std::rc::Rc;
#[cfg(not(target_os = "emscripten"))]
use std::sync::mpsc;

use serde::Serialize;
use serde::de::DeserializeOwned;

pub use api::{
	Allowed, Book, BookLadder, BookMove, BookTree, CardioDay, Check, CheckOpens, CheckOutcome, CheckWait, Clear, Climb,
	Composed, CorrectCheck, CorrectKind, CorrectSet, Corrected, CorrectionCheck, Dosed, Earn, Embers, Exclusions,
	ExpeditionPreview, ExpeditionStage, FeedbackNumbers, Finished, Folded, GameLadders, GameSummary, GapKind, Goal,
	Haul, Judged, JudgedRung, JudgedStep, Judgement, LadderPath, LadderStep, LevelAt, Locked, MasteredChange, Mastery,
	Match, MergeHow, Merged, MoveResult, MoveWaits, NeedEntry, NeedState, NextLoad, NextStep, Open, Opening, OwnSide,
	Pace, PaceUnit, PathNeed, PathNext, PathStep, Picked, PlannedMove, Preview, PreviewRound, Profile, Proving, Pump,
	Reached, Record, Rider, RidesAs, Ring, Rung, RungChange, SetAside, SetKey, SetReading, SetupGap, SetupStep, Slot,
	Stage, StageState, StageTop, Start, StartedCheck, Stays, StepChanges, StepNeed, StepState, SwapOptions, Take,
	Target, Top, Toward, Unfinished, Work,
};

/// The notices of what the engine brings into the app (Lua 5.1.5, mlua and mlua-sys, lua-src), each with its MIT
/// text: the desktop app's Acknowledgements lists them.
pub const NOTICES: &str = include_str!("../NOTICES.txt");

/// The SHA-256 of the embedded sources (each file's path, a NUL, its text with CRLF read as LF, a NUL, in load order):
/// the engine a build runs, named in the diagnostics.
pub const DIGEST: &str = embedded::DIGEST;

/// A work pump's size code (slice 67): the Normal pump, whose tier its offered row and its moves carry; its moves come
/// from its place in the set, never from the tier (`tools/apppump.lua` `A.compose` refuses any other).
pub const WORK_TIER: u8 = 2;

/// The engine's clock and time zone: the unix time now, and the seconds east of UTC at an instant (the computer's
/// zone; game days begin at 04:00 local). The engine asks for the offset once per 15 minutes of the rows it reads.
pub struct Clock {
	pub now: Box<dyn Fn() -> i64 + Send>,
	pub offset_at: Box<dyn Fn(i64) -> i64 + Send>,
}

/// Why the engine did not answer.
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
	/// The engine's Lua raised (or ran out of its memory): where (`DB.lua:1056`; empty when the error names no place),
	/// the message and Lua's traceback.
	#[error("{}", placed(.file_line, .message))]
	Lua { file_line: String, message: String, traceback: String },
	/// A request the engine cannot serve: an unknown op, no player before `init`, no pump on screen, a move the pump
	/// lacks, a swap with no options shown.
	#[error("{0}")]
	Refused(String),
	/// Arguments or an answer that do not have the op's shape.
	#[error("{0}")]
	Shape(String),
	/// The engine's thread could not start.
	#[error("the engine's thread did not start: {0}")]
	Spawn(std::io::Error),
	/// The engine's thread has ended.
	#[error("the engine has stopped")]
	Stopped,
	/// A call panicked on its caller's thread, which answered it with what the panic said and went on with the calls
	/// behind it (the desktop's door).
	#[error("the call panicked: {0}")]
	Panicked(String),
}

/// What a panic said (its message, when it gave one as text): the [`EngineError::Panicked`] of a call that panicked, on
/// the desktop's door and the browser's.
#[must_use]
pub fn panic_text(payload: &(dyn std::any::Any + Send)) -> String {
	payload
		.downcast_ref::<&str>()
		.map(|s| (*s).to_owned())
		.or_else(|| payload.downcast_ref::<String>().cloned())
		.unwrap_or_default()
}

/// A Lua error's message after its place, as Lua prints one.
fn placed(file_line: &str, message: &str) -> String {
	if file_line.is_empty() { message.to_owned() } else { format!("{file_line}: {message}") }
}

impl From<mlua::Error> for EngineError {
	fn from(e: mlua::Error) -> Self {
		match e {
			mlua::Error::CallbackError { cause, .. } => Self::from((*cause).clone()),
			mlua::Error::RuntimeError(text) => {
				let (first, traceback) = text.split_once('\n').unwrap_or((text.as_str(), ""));
				// "Core.lua:12: message", as Lua names a chunk loaded with '@' and its path
				let (file_line, message) = match first.split_once(": ") {
					Some((at, message)) if at.rsplit_once(':').is_some_and(|(_, line)| line.parse::<u32>().is_ok()) => {
						(at.to_owned(), message.to_owned())
					}
					_ => (String::new(), first.to_owned()),
				};
				Self::Lua { file_line, message, traceback: traceback.to_owned() }
			}
			mlua::Error::SerializeError(text) | mlua::Error::DeserializeError(text) => Self::Shape(text),
			mlua::Error::FromLuaConversionError { .. } => Self::Shape(e.to_string()),
			other => Self::Lua { file_line: String::new(), message: other.to_string(), traceback: String::new() },
		}
	}
}

/// One request to the engine's thread and where its answer goes.
#[cfg(not(target_os = "emscripten"))]
struct Job {
	op: String,
	args: serde_json::Value,
	reply: mpsc::SyncSender<Result<serde_json::Value, EngineError>>,
}

/// A handle to the engine's thread; every clone sends to the same engine, which ends when the last one is dropped. In the
/// browser the engine itself, on the one thread of the Web Worker that holds it.
#[derive(Clone)]
pub struct Engine {
	#[cfg(not(target_os = "emscripten"))]
	jobs: mpsc::Sender<Job>,
	#[cfg(target_os = "emscripten")]
	host: Rc<RefCell<host::Host>>,
}

impl Engine {
	/// Starts the engine's thread (`questpump-engine`, an 8 MiB stack: Lua's recursion through `pcall` and
	/// `table.sort` runs on it) and loads the engine on it, on `clock`; `report` hears each error the files report and
	/// go on from (a row the fold skips). Returns once the engine is loaded.
	///
	/// # Errors
	/// The thread did not start, or the engine did not load.
	#[cfg(not(target_os = "emscripten"))]
	pub fn spawn(clock: Clock, report: Box<dyn Fn(String) + Send>) -> Result<Self, EngineError> {
		let (jobs, inbox) = mpsc::channel::<Job>();
		let (ready, loaded) = mpsc::sync_channel(1);
		std::thread::Builder::new()
			.name("questpump-engine".into())
			.stack_size(8 << 20)
			.spawn(move || match host::Host::start(clock, report) {
				Ok(host) => {
					if ready.send(Ok(())).is_ok() {
						host.serve(&inbox);
					}
				}
				Err(e) => {
					let _ = ready.send(Err(e));
				}
			})
			.map_err(EngineError::Spawn)?;
		loaded.recv().map_err(|_| EngineError::Stopped)??;
		Ok(Self { jobs })
	}

	/// The browser's entry (docs/plan.md slice 63): loads the engine on the caller's thread, the Web Worker's one, on
	/// `clock`, with `report` as in [`Engine::spawn`]; every call then answers on that thread. Its stack is the one the
	/// module was linked with (`questpump-web`'s build: as large as the desktop's thread's).
	///
	/// # Errors
	/// The engine did not load.
	#[cfg(target_os = "emscripten")]
	pub fn load(clock: Clock, report: Box<dyn Fn(String) + Send>) -> Result<Self, EngineError> {
		Ok(Self { host: Rc::new(RefCell::new(host::Host::start(clock, report)?)) })
	}

	/// The door: `op` with its arguments (the table above), the answer as JSON.
	///
	/// # Errors
	/// [`EngineError`]: the Lua raised, the request was refused or misshapen, or the engine has stopped.
	#[cfg(not(target_os = "emscripten"))]
	pub fn call(&self, op: &str, args: serde_json::Value) -> Result<serde_json::Value, EngineError> {
		let (reply, answer) = mpsc::sync_channel(1);
		self.jobs.send(Job { op: op.to_owned(), args, reply }).map_err(|_| EngineError::Stopped)?;
		answer.recv().map_err(|_| EngineError::Stopped)?
	}

	/// The door: `op` with its arguments (the table above), the answer as JSON, on the caller's thread.
	///
	/// # Errors
	/// [`EngineError`]: the Lua raised, the request was refused or misshapen, or a call came from inside the engine's
	/// own call (a `report` or clock callback that called the engine again: refused, where the desktop's thread would
	/// wait on itself).
	#[cfg(target_os = "emscripten")]
	pub fn call(&self, op: &str, args: serde_json::Value) -> Result<serde_json::Value, EngineError> {
		let mut host = self
			.host
			.try_borrow_mut()
			.map_err(|_| EngineError::Refused(format!("{op} called from inside the engine's own call")))?;
		host.answer(op, args)
	}

	fn ask<T: DeserializeOwned>(&self, op: &str, args: &impl Serialize) -> Result<T, EngineError> {
		let args = serde_json::to_value(args).map_err(|e| EngineError::Shape(e.to_string()))?;
		serde_json::from_value(self.call(op, args)?).map_err(|e| EngineError::Shape(e.to_string()))
	}

	/// A new player's engine (no rows yet; [`Engine::fold`] takes theirs): `profile`, and the flags every row it writes
	/// carries (`ns.Enum.flagBit`'s app and computer bits on the desktop, 6144). The pump on screen goes.
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn init(&self, profile: &Profile, stamp: u16) -> Result<(), EngineError> {
		self.ask("init", &api::Init { profile: profile.clone(), stamp })
	}

	/// The player's profile changed (a Body switch, the experience, the moves, the place, a favourite, the title).
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn set_profile(&self, profile: &Profile) -> Result<(), EngineError> {
		self.ask("profile", &api::SetProfile { profile: profile.clone() })
	}

	/// The player's one progression from every row: the app's own (`own`, every device's, as the helper keeps them)
	/// and the game's (`game`, as `/v1/history` serves them), and the shown game sheet's focus (`sheet_focus`, its
	/// summary's `fc`). Takes its time with a long history: once at start, then [`Engine::merge`].
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn fold(&self, own: Vec<String>, game: Vec<String>, sheet_focus: Option<u8>) -> Result<Folded, EngineError> {
		self.ask("fold", &api::Fold { own, game, sheet_focus })
	}

	/// Rows from elsewhere since the fold: the app's own of other devices and the game's, each in the order the helper
	/// keeps them; never a row this engine wrote, which is in already. Answers how they went in.
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn merge(&self, own: Vec<String>, game: Vec<String>) -> Result<Merged, EngineError> {
		self.ask("merge", &api::Merge { own, game })
	}

	/// The joints the game's own rows flag now, ascending (`ns.Enum.joint` ids: the game's save's pain flags, as
	/// [`Folded::game_flags`] after a fold).
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn game_flags(&self) -> Result<Vec<u8>, EngineError> {
		self.ask("gameFlags", &serde_json::Value::Null)
	}

	/// Character creation's start rungs, written (a pump writes them itself when no ladder has a rung).
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn start_rows(&self, at: i64) -> Result<Vec<String>, EngineError> {
		self.ask("startRows", &api::At { at })
	}

	/// Pain flags set (`on`) or cleared for `joints` (`ns.Enum.joint` ids), written: a row per joint whose flag
	/// changes, by ascending id.
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn pain_rows(&self, joints: &[u8], on: bool, at: i64) -> Result<Vec<String>, EngineError> {
		self.ask("painRows", &api::Pain { joints: joints.to_vec(), on, at })
	}

	/// The check-in's answer (`ns.Enum.checkin`), written.
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn checkin_row(&self, answer: u8, at: i64) -> Result<String, EngineError> {
		self.ask("checkinRow", &api::Code { code: answer, at })
	}

	/// The focus chosen (`ns.Enum.focus`), written.
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn focus_row(&self, focus: u8, at: i64) -> Result<String, EngineError> {
		self.ask("focusRow", &api::Code { code: focus, at })
	}

	/// A clear row, written: the record of move `id` cleared, or ladder `id` started over (`scope`). The engine refuses a
	/// move its library lacks and a ladder that is no strength or pose ladder open now.
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn clear_row(&self, scope: Clear, id: u16, at: i64) -> Result<String, EngineError> {
		self.ask("clearRow", &api::ClearRow { scope, id, at })
	}

	/// Whether the one-tap check-in is due.
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn checkin_due(&self, at: i64) -> Result<bool, EngineError> {
		self.ask("checkinDue", &api::At { at })
	}

	/// The XP the app's own sheet counted on the game day holding `at`, its rested bonus in it (Stats.lua `ns.Today`'s
	/// `xp`, the phones' `todayXP`): Pump home's Today while Hero shows the app's own sheet.
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn today_xp(&self, at: i64) -> Result<i64, EngineError> {
		self.ask("todayXp", &api::At { at })
	}

	/// The level each lifetime XP of `xps` reaches, in their order (Stats.lua `levelOf`): the Saga's level at a day's
	/// start and end on the desktop app (contract/fixtures/history.json `saga` rule 15), never a Rust copy of the curve.
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn levels(&self, xps: &[i64]) -> Result<Vec<LevelAt>, EngineError> {
		self.ask("levels", &api::Xps { xps: xps.to_vec() })
	}

	/// Whether every pump is locked for feeling unwell, and since when.
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn locked(&self, at: i64) -> Result<Locked, EngineError> {
		self.ask("locked", &api::At { at })
	}

	/// The joints flagged now, ascending (`ns.Enum.joint` ids).
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn pain_flags(&self) -> Result<Vec<u8>, EngineError> {
		self.ask("painFlags", &serde_json::Value::Null)
	}

	/// The move each owed row of `rows` rides as now where it is another (Engine.lua `ns.RidesAs`, 2026-10-29: an owed
	/// backup of its rung the setup leaves unneeded rides as the move that stands for that rung, as
	/// [`Engine::catch_up`] doses it): `rows` as the helper's store keeps them (`Session::payable_kept`), each read as
	/// the fold reads an owed row; one [`RidesAs`] for each row that rides as another move, in their order, which Pump
	/// home's unfinished list names as the game's HUD does.
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn rides_as(&self, rows: Vec<String>, at: i64) -> Result<Vec<RidesAs>, EngineError> {
		self.ask("ridesAs", &api::RowsAt { rows, at })
	}

	/// The places in `rows` (from 0) of the owed rows that the setup in force leaves out with no move of their level to
	/// stand in (Engine.lua `ns.OwedAside`, 2026-11-09), read as [`Engine::rides_as`] reads them: Pump home's unfinished
	/// list and a catch-up leave them out until the setup lets them in again.
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn owed_aside(&self, rows: Vec<String>, at: i64) -> Result<Vec<usize>, EngineError> {
		self.ask("owedAside", &api::RowsAt { rows, at })
	}

	/// The records of `moves` with the goal an app pump asks of each; moves without one left out.
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn records(&self, moves: &[u16], at: i64) -> Result<Vec<Record>, EngineError> {
		self.ask("records", &api::MovesAt { moves: moves.to_vec(), at })
	}

	/// Each ladder's rung now, its best and the days its top was reached at it.
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn rungs(&self, ladders: &[u8]) -> Result<Vec<Rung>, EngineError> {
		self.ask("rungs", &api::Ladders { ladders: ladders.to_vec() })
	}

	/// Whether each family is open now.
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn open(&self, ladders: &[u8]) -> Result<Vec<Open>, EngineError> {
		self.ask("open", &api::Ladders { ladders: ladders.to_vec() })
	}

	/// Whether each move passes the player's filters now.
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn allowed(&self, moves: &[u16]) -> Result<Vec<Allowed>, EngineError> {
		self.ask("allowed", &api::Moves { moves: moves.to_vec() })
	}

	/// Your setup's next `count` pieces of equipment, each the one that opens the most with the ones before it added
	/// (the phones' `AgnosticEngine.setupSteps`).
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn setup_steps(&self, count: u8) -> Result<Vec<SetupStep>, EngineError> {
		self.ask("setupSteps", &api::Count { count })
	}

	/// What each piece of equipment not added would open now on its own (the phones' `AgnosticEngine.setupOpens`, the
	/// game's equipment tooltip): a piece that opens nothing is left out.
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn setup_opens(&self) -> Result<Vec<SetupStep>, EngineError> {
		self.ask("setupOpens", &serde_json::Value::Null)
	}

	/// The Book at `at` from the one progression beside the game's sheet the app shows (`game`; none without one): every
	/// ladder that holds a move with its fourteen steps, every move's facts and what of the game's
	/// sheet it set aside after a clear row or a correction ([`SetAside`]).
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn book(&self, game: Option<&GameLadders>, at: i64) -> Result<Book, EngineError> {
		self.ask("book", &api::BookAt { game: game.cloned(), at })
	}

	/// The app's own sheet summary at `at` (Stats.lua `ns.SheetSummary`, v 6: the shape of the game's sheet, which the
	/// helper keeps as JSON too; the week from Monday's game day, the game's rows only marking it).
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn summary(&self, at: i64) -> Result<serde_json::Value, EngineError> {
		self.call("summary", serde_json::json!({ "at": at }))
	}

	/// The save [`Engine::game_summary`] folds from (`save`, the CBOR of `questpump_core::session::GameSave`: the addon's
	/// profile of the character it is folded for with the body every device shares, which the game takes into it at its
	/// next load, and its install's baseline with the characters each side numbers; none: the app's profile without
	/// its title and no baseline), kept until the next one, a new player's `init` included. A save other than the one
	/// it holds folds again at the next ask; the same save again keeps its fold, so the caller sends it before each ask.
	///
	/// # Errors
	/// As [`Engine::call`]; the save not CBOR of a map is refused.
	pub fn game_save(&self, save: Option<&[u8]>) -> Result<(), EngineError> {
		use base64::Engine as _;
		let save = save.map(|cbor| base64::engine::general_purpose::STANDARD.encode(cbor));
		self.ask("gameSave", &api::GameSaveAt { save })
	}

	/// The game's own sheet at `at` as the game folds its save at its next load (the save [`Engine::game_save`] gave
	/// last: its baseline, then its rows from the latest reset on merged with the app's own as its mirror merges, every
	/// row through the sheet, read with the character's profile in the game as its next load binds it, in the game's
	/// weeks from the region's weekly reset `reset`, a reset instant a sheet of the game named, the US reset without
	/// one): the sheet the desktop app shows while no sheet the strip brought is as new as the game's rows. None while
	/// the game's rows hold none. Folded once after rows came, the engine wrote one or a save came (or for another
	/// reset), so the first ask after them takes about as long as a fold.
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn game_summary(&self, at: i64, reset: Option<i64>) -> Result<Option<GameSummary>, EngineError> {
		self.ask("gameSummary", &api::GameAt { at, reset })
	}

	/// The game's feedback numbers at `at` (Stats.lua `ns.FeedbackNumbers`, contract/feedback-api.yaml's counting rules
	/// as the addon counts them) over the game's own sheet as [`Engine::game_summary`] folds it, in the same weeks
	/// (`reset`, so the sheet folded for Hero is the one counted): the desktop app's weekly `review_numbers` of the game
	/// (contract/telemetry-api.yaml). None while the game's rows hold none.
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn feedback_numbers(&self, at: i64, reset: Option<i64>) -> Result<Option<FeedbackNumbers>, EngineError> {
		self.ask("feedbackNumbers", &api::GameAt { at, reset })
	}

	/// The pump a plain kind (`kind`, contract/kinds.json's key, and `family`, its family) and a size (`tier`) ask for,
	/// with a game's end's facts (`r#match`: the profile's "Moves per match" sizes it, its minutes give its cardio's
	/// length, a long sit warms it up first, the arms rest keeps it off the arms; none for every other pump, Balanced):
	/// the start rungs first while no ladder has one, then its moves and its offered row. A match pump carries the
	/// `unfinished` moves its device may pay (slice 61; empty for every other pump): composed on the queue as the game's
	/// quest pump, those that fit ride after its own moves (its [`Pump`]'s `riders`), each judged with
	/// [`Engine::judge`] and paid as a catch-up's, never written by [`Engine::finish`]. It becomes the pump on screen.
	///
	/// # Errors
	/// As [`Engine::call`].
	#[expect(clippy::too_many_arguments, reason = "the op's arguments, each a fact the pump is composed on")]
	pub fn compose(
		&self,
		id: &str,
		kind: &str,
		family: &str,
		tier: u8,
		at: i64,
		r#match: Option<Match>,
		unfinished: Unfinished,
	) -> Result<Composed, EngineError> {
		let args = api::Compose {
			id: id.to_owned(),
			kind: kind.to_owned(),
			family: Some(family.to_owned()),
			tier,
			at,
			r#match,
			work: None,
			unfinished,
		};
		self.ask("compose", &args)
	}

	/// A work pump (slice 67): an app pump of the kind key `kind` (contract/kinds.json's `work`) and its `family`, a
	/// Normal pump ([`WORK_TIER`]) sized by its place in the set at "Moves per block" (`work`, the profile's `workPace`),
	/// with no all-out window of its own or owed, its warm move first where `work.warm`, carrying the `unfinished` moves
	/// as [`Engine::compose`] carries a match pump's: the start rungs first while no ladder has one, then its moves and
	/// its offered row. It becomes the pump on screen.
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn work(
		&self,
		id: &str,
		kind: &str,
		family: &str,
		at: i64,
		work: Work,
		unfinished: Unfinished,
	) -> Result<Composed, EngineError> {
		let args = api::Compose {
			id: id.to_owned(),
			kind: kind.to_owned(),
			family: Some(family.to_owned()),
			tier: WORK_TIER,
			at,
			r#match: None,
			work: Some(work),
			unfinished,
		};
		self.ask("compose", &args)
	}

	/// The titles earned now, by their codes (`ns.Enum.title`) in the list's order: a pump's Result names those earned
	/// after it and not before it.
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn titles(&self) -> Result<Vec<u8>, EngineError> {
		self.ask("titles", &serde_json::Value::Null)
	}

	/// The least the next fresh set of `move` asks after a first record of `value` (`ns.FreshGoal`: one step past it,
	/// on a long hold of `ns.HOLD_GOAL`'s ladders about a tenth more in whole fives).
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn next_goal(&self, r#move: u16, value: i64) -> Result<i64, EngineError> {
		self.ask("nextGoal", &api::NextGoal { r#move, value })
	}

	/// A loaded move's next fresh set after the sets so far, at `at` (`ns.NextLoad`: its plan and goal
	/// `goal_fresh_h` later): the Result's "Next time" line; none for a move with no set with a weight.
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn next_load(&self, r#move: u16, at: i64) -> Result<Option<NextLoad>, EngineError> {
		self.ask("nextLoad", &api::NextLoadAt { r#move, at })
	}

	/// The Quick pump: a Small pump no quest flavours, its moves the player's state alone picks; picked from a match's
	/// offer it carries the `unfinished` moves as [`Engine::compose`] does (slice 61; empty otherwise). It becomes the
	/// pump on screen.
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn quick(&self, id: &str, at: i64, unfinished: Unfinished) -> Result<Composed, EngineError> {
		self.ask("quick", &api::Quick { id: id.to_owned(), at, unfinished })
	}

	/// An expedition's round (slice 51, `A.expedition`): an app pump of the kind key `expedition` at its `stage`'s tier,
	/// composed by the expedition's rules with the `regions` the rounds before it took (none before the first): the
	/// start rungs first while no ladder has one, then its moves and its offered row. It becomes the pump on screen.
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn expedition(
		&self,
		id: &str,
		stage: ExpeditionStage,
		regions: &[String],
		at: i64,
	) -> Result<Composed, EngineError> {
		self.ask("expedition", &api::ExpeditionAsk { id: id.to_owned(), stage, regions: regions.to_vec(), at })
	}

	/// The stage of an expedition's next round at `at` (`A.expeditionNext`, Engine.lua `ns.ExpeditionNext`): `budget`
	/// the length chosen and `elapsed` the time since it began, in seconds, `last` the stage that ran last (none before
	/// the first round), `regions` the regions its rounds took; none once it is done.
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn expedition_next(
		&self,
		budget: i64,
		elapsed: i64,
		last: Option<ExpeditionStage>,
		regions: &[String],
		at: i64,
	) -> Result<Option<ExpeditionStage>, EngineError> {
		self.ask("expeditionNext", &api::ExpeditionAt { budget, elapsed, last, regions: regions.to_vec(), at })
	}

	/// The plan of an expedition of `minutes` at `at` (`A.expeditionPreview`): its rules played on a copy of the engine,
	/// every move done at its goal; nothing is written.
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn expedition_preview(&self, minutes: u16, at: i64) -> Result<ExpeditionPreview, EngineError> {
		self.ask("expeditionPreview", &api::Minutes { minutes, at })
	}

	/// The pump on screen's time by the estimate (`Pump:time`, `ns.PumpTime`), in seconds.
	///
	/// # Errors
	/// As [`Engine::call`]; refused with no pump on screen.
	pub fn pump_time(&self) -> Result<i64, EngineError> {
		self.ask("pumpTime", &serde_json::Value::Null)
	}

	/// The regions an expedition's rounds took once the pump on screen is done, `regions` those before it
	/// (`Pump:regions`, `ns.RoundRegions`: its record moves' regions as they stand, a swap's in).
	///
	/// # Errors
	/// As [`Engine::call`]; refused with no pump on screen.
	pub fn pump_regions(&self, regions: &[String]) -> Result<Vec<String>, EngineError> {
		self.ask("pumpRegions", &api::Regions { regions: regions.to_vec() })
	}

	/// What one more piece of equipment or a switch would open on each of `ladders` now (`tools/apppump.lua`
	/// `Engine:ladderOpens` over Progress.lua `ns.SetupOpens` with the ladder, the line of the Talents tab's card,
	/// Tree.lua `Tree.CardMore`, its quality shifted as Your setup's `setupOpens`): for each ladder in their order,
	/// each piece with how many moves it opens there and their best quality, the best first; a step's card's quiet line
	/// (slice 38).
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn ladder_opens(&self, ladders: &[u8]) -> Result<Vec<Vec<SetupStep>>, EngineError> {
		self.ask("ladderOpens", &api::Ladders { ladders: ladders.to_vec() })
	}

	/// A move started from a step's card or a move's window (slice 36; David, 2026-10-13: "a use should be able to
	/// start a move from the tree anytime"): what a Start of `target` starts at `at`, read again now (Progress.lua
	/// `ns.StartFor` through `A.start`, its start rungs folded in before the first pump), and, unless that refuses, the
	/// pump of its one move (`A.pick`: a counted set alone from its go, dosed at the start's rung, a Small pump that
	/// pays and owes nothing; its kind `move`), which becomes the pump on screen. A refusal composes nothing and leaves
	/// the pump on screen as it was.
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn chosen(&self, id: &str, target: Target, at: i64) -> Result<Picked, EngineError> {
		self.ask("move", &api::Chosen { id: id.to_owned(), target, at })
	}

	/// What each move's window starts at `at` (`A.start` of each move, slice 36), in the order of `moves`: the move at
	/// its own goal at or under the player's rung, its ladder's climb above it, or why it starts nothing.
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn starts(&self, moves: &[u16], at: i64) -> Result<Vec<Start>, EngineError> {
		self.ask("starts", &api::Starts { moves: moves.to_vec(), at })
	}

	/// What a Start of `target` starts at `at` (`A.start`, as [`Engine::chosen`] reads it), composing nothing: a move
	/// pump on screen asks it again once another device started a ladder over under it before its first move (the
	/// game's `Lc.Cleared`), to end or compose again a pump whose start moved.
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn start(&self, target: Target, at: i64) -> Result<Start, EngineError> {
		self.ask("start", &api::StartAt { target, at })
	}

	/// Each step of another ladder in `needs` at `at`, in their order (`tools/apppump.lua` `Engine:needs`): whether the
	/// player has done it (Progress.lua `ns.NeedDone`, the test a climb's `needs`, a move's `checks` and a family's
	/// opening walk; an `any` group once one of its steps is) on the state the Book and a Start read, and what it waits
	/// for (`ns.NeedAt` with the player's filter, `ns.FilterContext`: the move the setup offers on that step and its
	/// record; an `any` group's first step the setup offers): the check and the waits line a step's card shows beside
	/// each, so no card marks one done while the climb still waits for it, nor names another record than the one that
	/// does it, or a move the setup leaves out (slice 38); on a stretch check its least answer and when it was passed
	/// (slice 42).
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn needs(&self, needs: &[NeedEntry], at: i64) -> Result<Vec<NeedState>, EngineError> {
		self.ask("needs", &api::NeedsAt { needs: needs.to_vec(), at })
	}

	/// A catch-up (since 2026-10-12) of the owed rows `pays` names (for each of its moves in its order, the place of its
	/// row in `rows`, from 0), over every owed row it may pay (`rows`, row texts as the store keeps them,
	/// `Session::payable_kept`), each read as the fold reads an owed row (DB.lua `ns.Translate`: a merged move's as the
	/// move that took it, on that move's own rung) and the whole list dosed as the game doses the owed moves that ride a
	/// pump (Engine.lua `riderSlot`, at each row's ladder and rung): its moves that ask a count (a rep move, a hold, or
	/// an owed all-out window of a countable form, on its own move, a counted set; since slice 42's fix round an owed
	/// steady move whose pace it asks), each by its place in the catch-up.
	/// It becomes the pump on screen ([`Engine::validate`], [`Engine::judge`] and a catch-up's [`Engine::callout`] take
	/// a move's place; one that fails leaves none); nothing is written.
	///
	/// # Errors
	/// As [`Engine::call`], a place `rows` lacks among them.
	pub fn catch_up(&self, id: &str, rows: Vec<String>, pays: Vec<usize>, at: i64) -> Result<Vec<Dosed>, EngineError> {
		self.ask("catchup", &api::Catchup { id: id.to_owned(), rows, pays, at })
	}

	/// The pump on screen.
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn pump(&self) -> Result<Pump, EngineError> {
		self.ask("pump", &serde_json::Value::Null)
	}

	/// Swap's options for move `index` on its Ready view (an owed slot's among them: a catch-up's move that asks its
	/// count since 2026-10-26, and since 2026-11-02 every move of a catch-up the engine placed, whatever it asks): once it
	/// was swapped its way back alone, while the filters pass the move it replaced; none for a move the pump lacks. A
	/// swap takes from the options shown last.
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn swap_options(&self, index: usize, at: i64) -> Result<Option<SwapOptions>, EngineError> {
		self.ask("swapOptions", &api::IndexAt { index, at })
	}

	/// Move `index` becomes the option the player took from the ones shown, [`Take::Back`] the slot a swap replaced:
	/// false, and nothing changed, where the pump refuses it.
	///
	/// # Errors
	/// As [`Engine::call`]; refused without options shown for that move, or without the one taken among them.
	pub fn swap(&self, index: usize, take: Take) -> Result<bool, EngineError> {
		self.ask("swap", &api::Swap { index, take })
	}

	/// The pump's move `index` as it stands (a catch-up's after Swap: its slot, the move as a device runs it and whether
	/// Swap put it in place of the owed move's), none for a move the pump lacks and for a catch-up's move that runs by
	/// its time (Swap's way back to one among them).
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn dosed(&self, index: usize) -> Result<Option<Dosed>, EngineError> {
		self.ask("dosed", &api::Index { index })
	}

	/// The pump's move `index` as its Ready view opens (`tools/apppump.lua` `Pump:ready`, Engine.lua `ns.ReadyGoal`;
	/// 2026-10-28): a rep or hold move's goal set again on the sets the pump wrote before it, kept only where it asks
	/// less (a second set of a ladder in the pump asks the repeat share), never a move pump's; then the move as it
	/// stands, as [`Engine::dosed`] answers it (its `today` the sets its ladder took this game day), none for a move
	/// the pump lacks.
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn ready(&self, index: usize, at: i64) -> Result<Option<Dosed>, EngineError> {
		self.ask("ready", &api::IndexAt { index, at })
	}

	/// Skip of a rider at `index` (the game's `Lc.Skip`; the helper writes its skipped row): the move leaves the riders
	/// and the unfinished list a later move's Swap reads. A catch-up's keeps its other moves' places; a match pump's
	/// (slice 61) moves every later move one place down, so the caller reads the pump again ([`Engine::pump`]). False
	/// for a move of the pump's own or a place it lacks.
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn skip(&self, index: usize) -> Result<bool, EngineError> {
		self.ask("skip", &api::Index { index })
	}

	/// After the Ready view's Pain flagged a joint ([`Engine::pain_rows`] first) on move `from`: the pump composed again
	/// while no move was `recorded`, else the rest kept where the flags allow it (a match pump's rider the flag refuses
	/// leaves it to wait, slice 61, so its riders change too: [`Engine::pump`]). Answers the pump's moves.
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn after_pain(&self, from: usize, recorded: bool, at: i64) -> Result<Vec<Slot>, EngineError> {
		self.ask("afterPain", &api::AfterPain { from, recorded, at })
	}

	/// A typed count for move `index` checked against the game's limits before it is written, with the weight typed
	/// beside it on a loaded move (tenths of a kg; none: no weight).
	///
	/// # Errors
	/// As [`Engine::call`]; refused for a move the pump lacks.
	pub fn validate(&self, index: usize, value: i64, load: Option<u32>) -> Result<Option<Check>, EngineError> {
		self.ask("validate", &api::Validate { index, value, load })
	}

	/// Move `index`'s result, judged as the game judges it, with the rung rows the rules add: the rows written (none
	/// for a move the pump lacks) and the cardio day the set brought ([`Finished`]).
	///
	/// # Errors
	/// As [`Engine::call`]; refused for a match pump's rider (slice 61), which [`Engine::judge`] judges and its payment
	/// writes.
	pub fn finish(&self, index: usize, result: MoveResult, at: i64) -> Result<Finished, EngineError> {
		self.ask("finish", &api::Finish { index, result, at })
	}

	/// Whether move `index`'s hold set of `value` seconds at `at` asks "Could you have held longer?" before it is
	/// written (slice 52, Progress.lua `ns.HoldAsk`: a fresh hold of a ledger ladder near its goal, once a ladder a game
	/// day); false for a move the pump lacks.
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn hold_ask(&self, index: usize, value: i64, at: i64) -> Result<bool, EngineError> {
		self.ask("holdAsk", &api::HoldAskAt { index, value, at })
	}

	/// Today's embers on the app's own sheet at `at` (slice 55, Stats.lua `ns.Embers`), Pump home's embers row where
	/// Hero shows no game sheet, and the week's omen by the app's week.
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn embers(&self, at: i64) -> Result<Embers, EngineError> {
		self.ask("embers", &api::At { at })
	}

	/// A rider's set for move `index` (a catch-up's move, or since slice 61 a match pump's rider), judged as the game's
	/// own catch-up judges it and written nowhere (the helper writes the catch-up's rows from it; an app row would pay
	/// its XP twice): its set, with every bit of its row's flags (the caller keeps the judge's own), the rung rows the
	/// rules add and the steps it mastered, which fold into the state for the pump's later moves, again after every fold
	/// or merge that folds the state anew, until [`Engine::settle`]; none, and nothing of it folded, for a move that asks
	/// nothing or a result that is no set (a typed count, and since slice 42's fix round a counted window saved with no
	/// count, Form slipped or (slice 53) Not all out and a steady move's answered pace are sets). Set or not, the step
	/// down its easier pick makes (since 2026-10-26, `ns.EasierDown`), folded as the set is.
	///
	/// # Errors
	/// As [`Engine::call`]; refused for a move of a match pump's own, which [`Engine::finish`] writes.
	pub fn judge(&self, index: usize, result: MoveResult, at: i64) -> Result<Judgement, EngineError> {
		self.ask("judge", &api::Finish { index, result, at })
	}

	/// Whether move `index` lights an ember with `result` (Stats.lua `ns.LightsEmber`, against the cardio step now),
	/// written nowhere: a rider's move (a catch-up's, a match pump's), whose rows the game folds at its next load, for
	/// its Result's Cardio today (2026-10-29), as the row its payment writes reads (`Session::catch_up`): a set as
	/// [`Engine::judge`] judges it, any other result done in the seconds it ran as the move run (its slot's, or its owed
	/// row's where it has none), stopped short or not; a move of a match pump's own as [`Engine::finish`] judges it.
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn lights(&self, index: usize, result: MoveResult, at: i64) -> Result<bool, EngineError> {
		self.ask("lights", &api::Finish { index, result, at })
	}

	/// A rider's move `index` done without a count (the game's `Lc.EmptyDone`; paid done with no set): the step down its
	/// easier pick makes (`ns.EasierDown`, since 2026-10-26), folded as [`Engine::judge`]'s, or none.
	///
	/// # Errors
	/// As [`Engine::judge`].
	pub fn empty_done(&self, index: usize, at: i64) -> Result<Option<JudgedStep>, EngineError> {
		self.ask("emptyDone", &api::IndexAt { index, at })
	}

	/// The apps' sweep at `at` (`tools/apppump.lua` `Engine:sweep`): an undo row for each app climb today's rules would
	/// not make (Progress.lua `ns.HeldClimb`, 2026-10-24), then the mastered rows the state lacks (`ns.MasterySweep`,
	/// slice 42's fix round, 2026-10-17: every step reached whose moves the setup offers all reached their top once and
	/// that holds none), written: at the apps' load and after a set or a catch-up, quietly (no Result row, no toast).
	/// The rows written.
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn sweep(&self, at: i64) -> Result<Vec<String>, EngineError> {
		self.ask("sweep", &api::At { at })
	}

	/// A catch-up's end: the counts [`Engine::judge`] folded in leave the state, which folds again from its ring (a
	/// merge's tens of milliseconds), asked just before the merge of the rows the helper wrote for them (or where nothing
	/// was paid), so they count once and the reads after it never see them twice. Whether it folded (nothing judged: no
	/// fold).
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn settle(&self) -> Result<bool, EngineError> {
		self.ask("settle", &serde_json::Value::Null)
	}

	/// Correct a count's window for `move` at `at` (slice 44; `tools/apppump.lua` `Engine:correctable`, Progress.lua
	/// `ns.Correctable` on the log the engine folds): the move's sets of the last `correct_days` game days dated after
	/// the window's reach (DB.lua `ns.CorrectReach`: 2 x `ring_batch` rows of that log and of the game's owed, banked
	/// and paid move rows, which the game's ring holds beside it, past `horizon`, the largest `rh` of the game's sheets
	/// the app knows, or past the engine's own baseline, whichever is later; none: the baseline's, and with neither the
	/// rows past `ring_rows - ring_batch`), so no set the game's ring let go or a trim before the game's next load
	/// could fold is offered, the newest `correct_sets` and the record's set where it is older, newest first, each with
	/// what a correction may change, its value, outcome and weight as the fold reads them now, its next round and
	/// whether it holds the record.
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn correctable(&self, r#move: u16, at: i64, horizon: Option<i64>) -> Result<Vec<CorrectSet>, EngineError> {
		self.ask("correctable", &api::CorrectableAt { r#move, at, horizon })
	}

	/// The check a correction of `set` to `value` at `load` (tenths of a kg; none keeps the set's own) meets
	/// (`Engine:correctCheck`, Progress.lua `ns.CorrectionCheck`: the count step's limits and "Is that right?" against
	/// the state just before the set, never about a low count on a counted set, as its count step asks none; a pose's
	/// answer asks nothing), the set read again in the window at `at` after the reach `horizon` sets first: gone once
	/// it has left it.
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn correct_check(
		&self,
		set: SetKey,
		value: i64,
		load: Option<u32>,
		at: i64,
		horizon: Option<i64>,
	) -> Result<CorrectionCheck, EngineError> {
		self.ask("correctCheck", &api::CorrectCheckAt { set, value, load, at, horizon })
	}

	/// `set` corrected at `at` to `value` (a count, a hold's seconds) at `load` (a loaded rep set's weight in tenths of
	/// a kg, 0 none; none keeps its own) or to a pose's `answer` (`ns.Enum.mark`), the set read again in the window
	/// after the reach `horizon` sets first (`Engine:correctRows`, Progress.lua `ns.CorrectionRows`): the rows written
	/// (stamped as the engine's own, folded in: the ring folds again from its baseline), why the steps stay where they
	/// do and the set as the fold reads it after them; gone, with nothing written, once the set has left the window.
	/// The mastered sweep after it is the caller's ([`Engine::sweep`]).
	///
	/// # Errors
	/// As [`Engine::call`]; refused for a value or an answer the set does not take.
	pub fn correct_rows(
		&self,
		set: SetKey,
		value: Option<i64>,
		load: Option<u32>,
		answer: Option<u8>,
		at: i64,
		horizon: Option<i64>,
	) -> Result<Corrected, EngineError> {
		self.ask("correctRows", &api::CorrectRowsAt { set, value, load, answer, at, horizon })
	}

	/// A felt-unwell stop while no timed move runs: the pump banked with the unwell reason, which locks every pump
	/// until the next game day. The rows written.
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn stop(&self, at: i64) -> Result<Vec<String>, EngineError> {
		self.ask("stop", &api::At { at })
	}

	/// The pump's moves as a device runs them.
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn moves(&self) -> Result<Vec<PlannedMove>, EngineError> {
		self.ask("moves", &serde_json::Value::Null)
	}

	/// What the answer to "How close to the goal pose?" of move `index` counts toward (Progress.lua `ns.MarkToward`),
	/// read from the state now on the move's Ready view (slice 47: the family's days its line counts, Panel.lua
	/// `poseDaysLine`). None for a move that asks nothing or one the pump lacks.
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn toward(&self, index: usize) -> Result<Option<Toward>, EngineError> {
		self.ask("toward", &api::Index { index })
	}

	/// How many stretch checks move `index`'s pose can pass, read as its ask opens (`tools/apppump.lua` `Pump:checks`,
	/// slice 42's fix round, 2026-10-17; Progress.lua `ns.PoseChecks`, the game's one list, since its last review,
	/// 2026-10-18): the check the pump runs from the one a card or a window started (`started`, `ns.RunsCheck`) first,
	/// then the one its stretch slot runs, then every other live one the pose answers (`ns.CheckLive`: not done, its
	/// opening not open already); none for a move that asks nothing. The engine keeps them for [`Engine::checked`],
	/// which marks the one the pump runs.
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn checks(&self, index: usize, started: Option<StartedCheck>) -> Result<usize, EngineError> {
		self.ask("checks", &api::ChecksAt { index, started })
	}

	/// What the checks [`Engine::checks`] read last for move `index` came to after its answer (`Pump:checked`): passed
	/// or not and, passed, whether what each opens is open now (`ns.OpensNow`) or what still holds it.
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn checked(&self, index: usize) -> Result<Vec<CheckOutcome>, EngineError> {
		self.ask("checked", &api::Index { index })
	}

	/// The callout of move `index` on its Ready view at `at` (Stats.lua `ns.PickCallout`): its take family (`record`,
	/// `day30`), or none. `count` names a catch-up's moves and `place` the move's place in it (from 0,
	/// [`Dosed::index`]), whose slot the catch-up on screen dosed it as the callout reads, as the game's reads a rider's
	/// (none for a move it did not dose: only the pump's own facts speak); no `count`: the pump on screen's own move.
	/// `before` is the streak the board shows and `after` the one the day's first performed move makes (none once today
	/// has one).
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn callout(
		&self,
		index: usize,
		count: Option<usize>,
		place: Option<usize>,
		at: i64,
		before: i64,
		after: Option<i64>,
	) -> Result<Option<String>, EngineError> {
		self.ask("callout", &api::CalloutAt { index, count, place, at, before, after })
	}

	/// What each move of the pump on screen can earn before its Start, at `at`.
	///
	/// # Errors
	/// As [`Engine::call`].
	pub fn preview(&self, at: i64) -> Result<Vec<Preview>, EngineError> {
		self.ask("preview", &api::At { at })
	}
}
