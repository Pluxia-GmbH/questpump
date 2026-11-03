//! The feedback button's message (docs/plan.md slice 24): the body the helper posts to `ROUTE` for the game (the
//! strip's `feedback` event) and for its own window. The body, version 1, holds exactly `source`, `version`, `text`,
//! `at` (the unix second Send was pressed) and, never both, the game's counted `numbers` (when its tick is on) or the
//! helper window's `diagnostics` (when its tick is on); the route refuses any other key, and nothing about the player's
//! identity goes. The outbox that keeps a message until the route takes it is `outbox.rs`'s. No clock or network here:
//! the app posts the bytes and passes the time in.

use ciborium::Value;
use serde::Serialize;

use crate::blob;
use crate::outbox::Rules;

/// The route the game's, the helper window's and both phones' messages go to.
pub const ROUTE: &str = "https://questpump.com/api/feedback";
/// The text's cap in Unicode scalar values, after trimming (the addon's `ns.C.feedback_max_chars`); the helper's window
/// holds its text box to it.
pub const MAX_CHARS: usize = 500;
/// The most moves the numbers list (the addon's `ns.C.feedback_moves_max`).
const MOVES_MAX: usize = 40;
/// The largest body the route takes, in bytes.
const BODY_MAX: usize = 8192;
/// The `at` the route takes, unix seconds: 2025-01-01 to 2100-01-01, so only a clock far off is refused.
const AT_MIN: i64 = 1_735_689_600;
const AT_MAX: i64 = 4_102_444_800;
/// The outbox keeps a message at most this many days and at most `KEEP_MAX` messages, the oldest dropped first (every
/// outbox alike, the phones' too).
pub const KEEP_DAYS: i64 = 30;
pub const KEEP_MAX: usize = 20;
/// The outbox of the game's and the helper window's messages (`<data dir>/feedback/`): 400, 401 and 413 drop a message,
/// since the route never takes it.
pub static OUTBOX: Rules =
	Rules { route: ROUTE, folder: "feedback", keep_days: KEEP_DAYS, keep_max: KEEP_MAX, refused: &[400, 401, 413] };
const VERSION_MAX: usize = 32;
const MOVE_MAX: usize = 48;
const DIAGNOSTIC_MAX: usize = 64;

/// Where a message was sent from (the helper sends the game's and its own window's; the phones post theirs).
#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum Source {
	Game,
	Helper,
}

/// The body as the route reads it, in its declaration order.
#[derive(Serialize, Debug)]
struct Body<'a> {
	source: Source,
	version: &'a str,
	text: &'a str,
	at: i64,
	#[serde(skip_serializing_if = "Option::is_none")]
	numbers: Option<Numbers>,
	#[serde(skip_serializing_if = "Option::is_none")]
	diagnostics: Option<&'a Diagnostics>,
}

/// The game's counted totals as its window showed them, exactly these keys (contract/feedback-api.yaml, `Numbers`): no
/// health data goes.
#[derive(Serialize, Debug)]
struct Numbers {
	days: u32,
	finished: u32,
	banked: u32,
	owed: u32,
	max_sets: f64,
	moves: Vec<MoveCount>,
}

/// A move's swaps to the easier version on its first appearance and its early stops.
#[derive(Serialize, Debug)]
struct MoveCount {
	#[serde(rename = "move")]
	move_id: String,
	easier: u32,
	stopped: u32,
}

/// What the helper window's tick adds: the system and the capture's state, as the window shows them.
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Diagnostics {
	os: String,
	capture: String,
}

impl Diagnostics {
	/// Each value held to the route's 64 characters.
	#[must_use]
	pub fn new(os: &str, capture: &str) -> Self {
		let cap = |s: &str| s.chars().take(DIAGNOSTIC_MAX).collect();
		Self { os: cap(os), capture: cap(capture) }
	}
}

/// Why a message cannot go: the rule it breaks (a crash report's too, or why the system's report does not read).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{0}")]
pub struct Invalid(String);

pub(crate) fn invalid(why: impl Into<String>) -> Invalid {
	Invalid(why.into())
}

/// A message checked against the route's rules: the name the outbox keeps it under (`g-<load id>-<at>` for the game's,
/// `h-<unix ms>` for the helper window's) and the body's JSON, as it is posted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
	pub id: String,
	bytes: Vec<u8>,
}

impl Message {
	/// The game's message from its `feedback` strip event: a CBOR map of exactly `text`, `version`, `at` (unix seconds
	/// when Send was pressed, which goes in the body and with the load id names the message) and, when the tick was on,
	/// `numbers`.
	///
	/// # Errors
	/// The first rule the payload breaks: a message the route would refuse is never posted.
	pub fn game(load_id: u16, v: &Value) -> Result<Self, Invalid> {
		keys(v, &["text", "version", "at"], &["numbers"])?;
		let text = blob::get(v, "text")
			.and_then(blob::as_text)
			.and_then(text)
			.ok_or_else(|| invalid(format!("the text is empty or over {MAX_CHARS} characters")))?;
		let version =
			blob::get(v, "version").and_then(blob::as_text).filter(|s| version(s)).ok_or_else(|| invalid("version"))?;
		let at = blob::get(v, "at")
			.and_then(blob::as_u64)
			.and_then(|t| i64::try_from(t).ok())
			.filter(|t| (AT_MIN..=AT_MAX).contains(t))
			.ok_or_else(|| invalid("at"))?;
		let numbers = blob::get(v, "numbers").map(numbers).transpose()?;
		let body = Body { source: Source::Game, version, text: &text, at, numbers, diagnostics: None };
		Ok(Self { id: format!("g-{load_id}-{at}"), bytes: bytes(&body)? })
	}

	/// The helper window's message, named by `at_ms` (unix milliseconds, when Send was pressed; its second goes in the
	/// body): the player's text, the helper's version and, when the tick is on, its diagnostics.
	///
	/// # Errors
	/// The text is empty or over `MAX_CHARS`, the version breaks the route's pattern, or the clock is far off.
	pub fn helper(
		at_ms: i64,
		raw: &str,
		helper_version: &str,
		diagnostics: Option<&Diagnostics>,
	) -> Result<Self, Invalid> {
		let text = text(raw).ok_or_else(|| invalid(format!("the text is empty or over {MAX_CHARS} characters")))?;
		if !version(helper_version) {
			return Err(invalid("version"));
		}
		let at = at_ms.div_euclid(1000);
		if !(AT_MIN..=AT_MAX).contains(&at) {
			return Err(invalid("at: the computer's clock is far off"));
		}
		let body =
			Body { source: Source::Helper, version: helper_version, text: &text, at, numbers: None, diagnostics };
		Ok(Self { id: format!("h-{at_ms}"), bytes: bytes(&body)? })
	}

	/// The body's JSON, as it is posted.
	#[must_use]
	pub fn body(&self) -> &[u8] {
		&self.bytes
	}
}

/// Whether a message's id names one from the game (`g-<load id>-<at>`), not the helper's window (`h-<unix ms>`): the
/// outbox takes a game's message once, since the strip may show it again.
#[must_use]
pub fn from_game(id: &str) -> bool {
	id.starts_with("g-")
}

/// The player's text as it goes: trimmed, 1 to `MAX_CHARS` Unicode scalar values; None when it is empty or longer.
fn text(raw: &str) -> Option<String> {
	let t = raw.trim();
	(!t.is_empty() && t.chars().count() <= MAX_CHARS).then(|| t.to_owned())
}

/// A version the route takes: 1 to 32 of `[A-Za-z0-9._+-]` (the crash route's too).
pub(crate) fn version(v: &str) -> bool {
	(1..=VERSION_MAX).contains(&v.len()) && v.bytes().all(|b| b.is_ascii_alphanumeric() || b"._+-".contains(&b))
}

fn bytes(body: &Body) -> Result<Vec<u8>, Invalid> {
	let bytes = serde_json::to_vec(body).map_err(|e| invalid(e.to_string()))?;
	if bytes.len() > BODY_MAX {
		return Err(invalid(format!("{} bytes, over the route's {BODY_MAX}", bytes.len())));
	}
	Ok(bytes)
}

/// A CBOR map whose keys are text, each one of `required` or `optional`, once, with every one of `required` there.
fn keys(v: &Value, required: &[&str], optional: &[&str]) -> Result<(), Invalid> {
	let Value::Map(m) = v else { return Err(invalid("not a map")) };
	let mut seen: Vec<&str> = Vec::with_capacity(m.len());
	for (k, _) in m {
		let Value::Text(k) = k else { return Err(invalid(format!("the key {k:?}"))) };
		if !(required.contains(&k.as_str()) || optional.contains(&k.as_str())) || seen.contains(&k.as_str()) {
			return Err(invalid(format!("the key {k:?}")));
		}
		seen.push(k);
	}
	match required.iter().find(|r| !seen.contains(r)) {
		Some(missing) => Err(invalid(format!("no {missing}"))),
		None => Ok(()),
	}
}

fn numbers(v: &Value) -> Result<Numbers, Invalid> {
	keys(v, &["days", "finished", "banked", "owed", "max_sets", "moves"], &[])?;
	let count = |k: &str| blob::get(v, k).and_then(count).ok_or_else(|| invalid(format!("numbers.{k}")));
	let max_sets = blob::get(v, "max_sets").and_then(decimal).ok_or_else(|| invalid("numbers.max_sets"))?;
	let moves = blob::get(v, "moves")
		.and_then(list)
		.filter(|m| m.len() <= MOVES_MAX)
		.ok_or_else(|| invalid(format!("numbers.moves is no list of at most {MOVES_MAX}")))?;
	Ok(Numbers {
		days: count("days")?,
		finished: count("finished")?,
		banked: count("banked")?,
		owed: count("owed")?,
		max_sets,
		moves: moves.into_iter().map(move_count).collect::<Result<_, _>>()?,
	})
}

fn move_count(v: &Value) -> Result<MoveCount, Invalid> {
	keys(v, &["move", "easier", "stopped"], &[])?;
	let move_id = blob::get(v, "move")
		.and_then(blob::as_text)
		.filter(|m| {
			(1..=MOVE_MAX).contains(&m.len())
				&& m.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
		})
		.ok_or_else(|| invalid("numbers.moves: a move id"))?;
	let count = |k: &str| blob::get(v, k).and_then(count).ok_or_else(|| invalid(format!("numbers.moves: {k}")));
	Ok(MoveCount { move_id: move_id.to_owned(), easier: count("easier")?, stopped: count("stopped")? })
}

/// A count: a whole number from 0 (a Lua number may arrive as a float).
fn count(v: &Value) -> Option<u32> {
	blob::as_u64(v).and_then(|n| u32::try_from(n).ok())
}

/// A number from 0 to one decimal (a median of max sets).
fn decimal(v: &Value) -> Option<f64> {
	let x = match v {
		Value::Float(f) => *f,
		other => f64::from(count(other)?),
	};
	(x.is_finite() && (0.0..=f64::from(u32::MAX)).contains(&x)).then(|| (x * 10.0).round() / 10.0)
}

/// A Lua sequence as the client's CBOR writes it: an array, a map keyed 1 to n, or an empty map (an empty table).
fn list(v: &Value) -> Option<Vec<&Value>> {
	match v {
		Value::Array(a) => Some(a.iter().collect()),
		Value::Map(m) if m.is_empty() => Some(Vec::new()),
		Value::Map(m) => blob::sequence(m),
		_ => None,
	}
}
