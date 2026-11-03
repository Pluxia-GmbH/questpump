//! The engine's door to JavaScript (the exports `main.rs` lists): one engine on the Web Worker's thread, its calls as
//! JSON in and out, and the host's clock, zone and report as the module's imports (`src/imports.js`).

use std::cell::RefCell;
use std::ffi::{CStr, CString, c_char};
use std::panic::{AssertUnwindSafe, catch_unwind};

use questpump_engine::{Clock, Engine, EngineError};
use serde_json::{Value, json};

// SAFETY: src/imports.js defines the three with these signatures (numbers as f64; a pointer and a byte count).
unsafe extern "C" {
	safe fn qp_now() -> f64;
	safe fn qp_offset_at(t: f64) -> f64;
	fn qp_report(text: *const u8, len: usize);
}

thread_local! {
	/// The engine, once `qp_start` loaded it.
	static ENGINE: RefCell<Option<Engine>> = const { RefCell::new(None) };
}

#[expect(clippy::cast_possible_truncation, reason = "a unix time in whole seconds, as the host's clock counts them")]
fn now() -> i64 {
	qp_now().floor() as i64
}

#[expect(clippy::cast_possible_truncation, reason = "an offset in whole seconds, within a day")]
#[expect(clippy::cast_precision_loss, reason = "a unix time in seconds is exact in an f64 (below 2^53)")]
fn offset_at(t: i64) -> i64 {
	qp_offset_at(t as f64).round() as i64
}

fn report(text: &str) {
	// SAFETY: the pointer and length are the borrowed text's own bytes, read by the import before it returns
	unsafe { qp_report(text.as_ptr(), text.len()) }
}

/// An answer as the door writes it: `{"ok": value}`, or `{"error": {...}}` with the refusal's kind and words.
fn reply(answer: Result<Value, EngineError>) -> *mut c_char {
	let body = match answer {
		Ok(value) => json!({ "ok": value }),
		Err(EngineError::Lua { file_line, message, traceback }) => {
			json!({ "error": { "kind": "lua", "fileLine": file_line, "message": message, "traceback": traceback } })
		}
		Err(EngineError::Refused(message)) => json!({ "error": { "kind": "refused", "message": message } }),
		Err(EngineError::Shape(message)) => json!({ "error": { "kind": "shape", "message": message } }),
		Err(EngineError::Panicked(message)) => json!({ "error": { "kind": "panicked", "message": message } }),
		Err(other) => json!({ "error": { "kind": "stopped", "message": other.to_string() } }),
	};
	// serde_json writes U+0000 as \u0000, so a JSON text holds no NUL and the CString always takes it whole
	CString::new(body.to_string()).unwrap_or_default().into_raw()
}

/// Runs `f`, a panic in it answered as `EngineError::Panicked`, as the desktop's door answers one, and the calls after
/// it still served.
fn caught(f: impl FnOnce() -> Result<Value, EngineError>) -> Result<Value, EngineError> {
	catch_unwind(AssertUnwindSafe(f))
		.unwrap_or_else(|payload| Err(EngineError::Panicked(questpump_engine::panic_text(&*payload))))
}

/// Loads the engine (again: a new one in place of the old) on the host's clock, zone and report.
#[unsafe(no_mangle)]
pub extern "C" fn qp_start() -> *mut c_char {
	reply(caught(|| {
		let engine = Engine::load(
			Clock { now: Box::new(now), offset_at: Box::new(offset_at) },
			Box::new(|text: String| report(&text)),
		)?;
		ENGINE.with_borrow_mut(|held| *held = Some(engine));
		Ok(json!({ "digest": questpump_engine::DIGEST }))
	}))
}

/// An op of the engine's door with its arguments as JSON.
///
/// # Safety
/// `op` and `args` are NUL-terminated strings the caller keeps until the call returns.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn qp_call(op: *const c_char, args: *const c_char) -> *mut c_char {
	// SAFETY: the caller passes two NUL-terminated strings it keeps until this returns (the function's contract)
	let (op, args) = unsafe { (CStr::from_ptr(op), CStr::from_ptr(args)) };
	reply(caught(|| {
		let op = op.to_str().map_err(|e| EngineError::Shape(format!("the op's name: {e}")))?;
		let args: Value = serde_json::from_slice(args.to_bytes())
			.map_err(|e| EngineError::Shape(format!("{op}'s arguments: {e}")))?;
		// the handle cloned out, so the thread-local is free again while the engine answers
		let engine = ENGINE
			.with_borrow(Clone::clone)
			.ok_or_else(|| EngineError::Refused("the engine has not started: qp_start comes first".into()))?;
		engine.call(op, args)
	}))
}

/// Gives an answer's memory back.
///
/// # Safety
/// `answer` is an answer of `qp_start` or `qp_call` not given back yet, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn qp_free(answer: *mut c_char) {
	if !answer.is_null() {
		// SAFETY: the pointer came from CString::into_raw in `reply` and comes back once (the function's contract)
		drop(unsafe { CString::from_raw(answer) });
	}
}
