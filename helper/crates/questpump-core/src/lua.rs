//! A reader for Lua literals, the subset `State.lua` is written in (`mirror.rs`): `Name = value` statements whose values
//! are strings, numbers, booleans, nil and tables, with `--` line comments. The helper reads a mirror it wrote before,
//! so a lost data folder never costs the rows the game reads back; nothing in the file runs.

use ciborium::Value;

/// Tables nested deeper than this are refused (a profile nests three or four deep).
const DEPTH: usize = 64;

/// Text that is not a Lua literal of the subset, and where it stops reading.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("not a Lua literal at byte {at}: {what}")]
pub struct LuaError {
	pub at: usize,
	pub what: &'static str,
}

/// A Lua string as the helper keeps it, whether a saved file's literal or the client's CBOR brought it: text when its
/// bytes are UTF-8, a byte string when they are not, so every reader finds a key or a name as text.
pub(crate) fn string(bytes: Vec<u8>) -> Value {
	String::from_utf8(bytes).map_or_else(|e| Value::Bytes(e.into_bytes()), Value::Text)
}

struct Reader<'a> {
	b: &'a [u8],
	at: usize,
}

impl Reader<'_> {
	fn fail<T>(&self, what: &'static str) -> Result<T, LuaError> {
		Err(LuaError { at: self.at, what })
	}

	fn peek(&self) -> Option<u8> {
		self.b.get(self.at).copied()
	}

	fn rest(&self) -> &[u8] {
		&self.b[self.at.min(self.b.len())..]
	}

	/// Skips white space and line comments.
	fn skip(&mut self) -> Result<(), LuaError> {
		loop {
			while self.peek().is_some_and(|c| c.is_ascii_whitespace()) {
				self.at += 1;
			}
			if !self.rest().starts_with(b"--") {
				return Ok(());
			}
			if self.rest()[2..].starts_with(b"[") {
				return self.fail("a block comment");
			}
			while self.peek().is_some_and(|c| c != b'\n') {
				self.at += 1;
			}
		}
	}

	fn eat(&mut self, c: u8) -> Result<bool, LuaError> {
		self.skip()?;
		let found = self.peek() == Some(c);
		if found {
			self.at += 1;
		}
		Ok(found)
	}

	fn name(&mut self) -> Option<String> {
		let start = self.at;
		if !self.peek().is_some_and(|c| c.is_ascii_alphabetic() || c == b'_') {
			return None;
		}
		while self.peek().is_some_and(|c| c.is_ascii_alphanumeric() || c == b'_') {
			self.at += 1;
		}
		Some(String::from_utf8_lossy(&self.b[start..self.at]).into_owned())
	}

	/// A `Name =` that starts a table field, or nothing (the position is kept).
	fn field_name(&mut self) -> Result<Option<String>, LuaError> {
		self.skip()?;
		let start = self.at;
		if let Some(name) = self.name() {
			self.skip()?;
			if self.peek() == Some(b'=') && self.b.get(self.at + 1) != Some(&b'=') {
				self.at += 1;
				return Ok(Some(name));
			}
		}
		self.at = start;
		Ok(None)
	}

	fn value(&mut self, depth: usize) -> Result<Value, LuaError> {
		self.skip()?;
		match self.peek() {
			Some(b'"' | b'\'') => self.string(),
			Some(b'{') => self.table(depth),
			Some(b'-' | b'.' | b'0'..=b'9') => self.number(),
			Some(_) => match self.name().as_deref() {
				Some("true") => Ok(Value::Bool(true)),
				Some("false") => Ok(Value::Bool(false)),
				Some("nil") => Ok(Value::Null),
				_ => self.fail("a value"),
			},
			None => self.fail("a value"),
		}
	}

	fn string(&mut self) -> Result<Value, LuaError> {
		let quote = self.b[self.at];
		self.at += 1;
		let mut out = Vec::new();
		loop {
			let Some(c) = self.peek() else { return self.fail("an unfinished string") };
			self.at += 1;
			match c {
				_ if c == quote => break,
				b'\n' => return self.fail("a line break in a string"),
				b'\\' => {
					let Some(e) = self.peek() else { return self.fail("an unfinished string") };
					self.at += 1;
					out.push(match e {
						b'n' => b'\n',
						b't' => b'\t',
						b'r' => b'\r',
						b'a' => 7,
						b'b' => 8,
						b'f' => 12,
						b'v' => 11,
						b'\\' | b'"' | b'\'' | b'\n' => e,
						b'0'..=b'9' => {
							let mut n = u32::from(e - b'0');
							for _ in 0..2 {
								let Some(d @ b'0'..=b'9') = self.peek() else { break };
								n = n * 10 + u32::from(d - b'0');
								self.at += 1;
							}
							let Ok(byte) = u8::try_from(n) else { return self.fail("an escape past 255") };
							byte
						}
						_ => return self.fail("an unknown escape"),
					});
				}
				_ => out.push(c),
			}
		}
		Ok(string(out))
	}

	fn number(&mut self) -> Result<Value, LuaError> {
		let start = self.at;
		if self.peek() == Some(b'-') {
			self.at += 1;
		}
		while self.peek().is_some_and(|c| c.is_ascii_digit() || c == b'.') {
			self.at += 1;
		}
		if matches!(self.peek(), Some(b'e' | b'E')) {
			self.at += 1;
			if matches!(self.peek(), Some(b'+' | b'-')) {
				self.at += 1;
			}
			while self.peek().is_some_and(|c| c.is_ascii_digit()) {
				self.at += 1;
			}
		}
		let text = std::str::from_utf8(&self.b[start..self.at]).unwrap_or_default();
		if !text.contains(['.', 'e', 'E']) {
			if let Ok(i) = text.parse::<i64>() {
				return Ok(Value::Integer(i.into()));
			}
			if let Ok(u) = text.parse::<u64>() {
				return Ok(Value::Integer(u.into()));
			}
		}
		match text.parse::<f64>() {
			Ok(f) if f.is_finite() => Ok(Value::Float(f)),
			_ => self.fail("a number"),
		}
	}

	/// A table: a list (`Value::Array`, a nil kept in its place) when it has only positional items, else a map in which
	/// the positional items take the keys 1, 2, … as Lua gives them.
	fn table(&mut self, depth: usize) -> Result<Value, LuaError> {
		if depth == 0 {
			return self.fail("tables nested too deep");
		}
		self.at += 1;
		let mut list = Vec::new();
		let mut map = Vec::new();
		loop {
			if self.eat(b'}')? {
				break;
			}
			if self.eat(b'[')? {
				let key = self.value(depth - 1)?;
				if !self.eat(b']')? || !self.eat(b'=')? {
					return self.fail("a key without `] =`");
				}
				map.push((key, self.value(depth - 1)?));
			} else if let Some(name) = self.field_name()? {
				map.push((Value::Text(name), self.value(depth - 1)?));
			} else {
				list.push(self.value(depth - 1)?);
			}
			if !self.eat(b',')? && !self.eat(b';')? {
				if self.eat(b'}')? {
					break;
				}
				return self.fail("a `,` or `}` in a table");
			}
		}
		if map.is_empty() {
			return Ok(Value::Array(list));
		}
		let mut all: Vec<(Value, Value)> = (1_u64..).zip(list).map(|(i, v)| (Value::Integer(i.into()), v)).collect();
		all.extend(map);
		Ok(Value::Map(all))
	}
}

/// The `Name = value` statements of a file, in order.
///
/// # Errors
/// Text outside the subset: a statement that is not an assignment of a literal, a block comment, an unfinished string.
pub fn globals(text: &str) -> Result<Vec<(String, Value)>, LuaError> {
	let mut r = Reader { b: text.as_bytes(), at: 0 };
	let mut out = Vec::new();
	loop {
		r.skip()?;
		if r.peek().is_none() {
			return Ok(out);
		}
		let Some(name) = r.name() else { return r.fail("a statement") };
		if !r.eat(b'=')? {
			return r.fail("an `=` after a name");
		}
		out.push((name, r.value(DEPTH)?));
		r.eat(b';')?;
	}
}
