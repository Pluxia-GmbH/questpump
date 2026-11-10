//! The profile, the sheet and a feedback message, which travel in chunks (`Bridge.lua` `B.Chunks`): each chunk carries
//! the payload's total length, then 73 payload bytes; the payload is raw DEFLATE of CBOR, the addon's export codec
//! (`C_EncodingUtil.CompressString` after `SerializeCBOR`). The client writes every Lua string, a table's key too, as
//! a CBOR byte string (major type 2), never as a text string (warcraft.wiki.gg, `API_C_EncodingUtil.SerializeCBOR`):
//! `decode` hands each on as the text it is, which is what every reader here and of the kept values looks for.

use std::collections::HashMap;
use std::io::Read as _;

use ciborium::Value;

use crate::codes::StripEvent;
use crate::lua;
use crate::strip::Chunk;

/// A payload that arrived whole but does not decode.
#[derive(Debug, thiserror::Error)]
pub enum BlobError {
	#[error("the payload does not inflate: {0}")]
	Inflate(std::io::Error),
	#[error("the payload is not CBOR: {0}")]
	Cbor(String),
}

#[derive(Default)]
struct Partial {
	serial: u8,
	chunks: u8,
	length: u16,
	parts: Vec<Option<Vec<u8>>>,
}

/// Collects chunks per load and event until a copy is whole; a chunk of a new copy (another serial, count or length)
/// starts over.
#[derive(Default)]
pub struct Assembler {
	partial: HashMap<(u16, StripEvent), Partial>,
}

impl Assembler {
	/// Adds one chunk; returns the payload's bytes (still compressed) when its copy is complete.
	pub fn add(&mut self, load_id: u16, event: StripEvent, chunk: &Chunk) -> Option<Vec<u8>> {
		if chunk.chunks == 0 || chunk.chunk >= chunk.chunks {
			return None;
		}
		let p = self.partial.entry((load_id, event)).or_default();
		if p.parts.is_empty() || p.serial != chunk.serial || p.chunks != chunk.chunks || p.length != chunk.length {
			*p = Partial {
				serial: chunk.serial,
				chunks: chunk.chunks,
				length: chunk.length,
				parts: vec![None; usize::from(chunk.chunks)],
			};
		}
		p.parts[usize::from(chunk.chunk)] = Some(chunk.data.clone());
		if p.parts.iter().any(Option::is_none) {
			return None;
		}
		let mut bytes: Vec<u8> = p.parts.iter().flatten().flatten().copied().collect();
		bytes.truncate(usize::from(p.length));
		self.partial.remove(&(load_id, event));
		Some(bytes)
	}
}

/// Inflates a payload (raw DEFLATE; a zlib stream is read too) and decodes its CBOR, its strings as `strings` reads
/// them.
///
/// # Errors
/// When the bytes neither inflate nor decode.
pub fn decode(packed: &[u8]) -> Result<Value, BlobError> {
	let mut cbor = Vec::new();
	if flate2::read::DeflateDecoder::new(packed).read_to_end(&mut cbor).is_err() {
		cbor.clear();
		flate2::read::ZlibDecoder::new(packed).read_to_end(&mut cbor).map_err(BlobError::Inflate)?;
	}
	ciborium::from_reader(cbor.as_slice()).map(strings).map_err(|e| BlobError::Cbor(e.to_string()))
}

/// A decoded payload with every byte string, a map's key too, as the Lua string it is (`lua::string`: text when it is
/// UTF-8), the same value a saved file's reader makes of that string, so a profile the strip brought and one a file
/// held compare and are written alike.
fn strings(v: Value) -> Value {
	match v {
		Value::Bytes(b) => lua::string(b),
		Value::Array(a) => Value::Array(a.into_iter().map(strings).collect()),
		Value::Map(m) => Value::Map(m.into_iter().map(|(k, x)| (strings(k), strings(x))).collect()),
		Value::Tag(tag, inner) => Value::Tag(tag, Box::new(strings(*inner))),
		other => other,
	}
}

/// A CBOR value as JSON, for the lines and `GET /v1/sheet`: map keys become strings (an integer key its digits), a
/// map keyed 1 to n (how a Lua sequence may be written) an array, a byte string (`decode` leaves one only where a
/// string is no UTF-8) an array of numbers, a value JSON cannot hold (NaN, infinity, undefined) null. The phone apps'
/// decoders apply the same rules to an imported export.
#[must_use]
pub fn to_json(v: &Value) -> serde_json::Value {
	use serde_json::Value as J;
	match v {
		Value::Integer(i) => {
			let n = i128::from(*i);
			i64::try_from(n).map_or_else(|_| u64::try_from(n).map_or(J::Null, J::from), J::from)
		}
		Value::Float(f) => serde_json::Number::from_f64(*f).map_or(J::Null, J::Number),
		Value::Text(s) => J::String(s.clone()),
		Value::Bytes(b) => J::Array(b.iter().map(|&x| J::from(x)).collect()),
		Value::Bool(b) => J::Bool(*b),
		Value::Array(a) => J::Array(a.iter().map(to_json).collect()),
		Value::Map(m) => sequence(m).map_or_else(
			|| J::Object(m.iter().map(|(k, v)| (key_text(k), to_json(v))).collect()),
			|items| J::Array(items.into_iter().map(to_json).collect()),
		),
		Value::Tag(_, inner) => to_json(inner),
		_ => J::Null,
	}
}

/// A map's values in key order when its keys are exactly the integers 1 to n (a Lua sequence).
pub(crate) fn sequence(m: &[(Value, Value)]) -> Option<Vec<&Value>> {
	if m.is_empty() {
		return None;
	}
	let mut items = vec![None; m.len()];
	for (k, v) in m {
		let Value::Integer(i) = k else { return None };
		let slot = usize::try_from(i128::from(*i)).ok()?.checked_sub(1)?;
		*items.get_mut(slot)? = Some(v);
	}
	items.into_iter().collect()
}

/// A map key as text: a string as it is, an integer as its digits.
#[must_use]
pub fn key_text(k: &Value) -> String {
	match k {
		Value::Text(s) => s.clone(),
		Value::Integer(i) => i128::from(*i).to_string(),
		other => format!("{other:?}"),
	}
}

/// The value at a text key of a CBOR map.
#[must_use]
pub fn get<'a>(v: &'a Value, key: &str) -> Option<&'a Value> {
	match v {
		Value::Map(m) => m.iter().find(|(k, _)| matches!(k, Value::Text(t) if t == key)).map(|(_, v)| v),
		_ => None,
	}
}

/// A CBOR integer as a u64, if it is one; a float that holds a whole number counts too, in case the client's encoder
/// writes a Lua number (a double in Lua 5.1) as a float.
#[must_use]
pub fn as_u64(v: &Value) -> Option<u64> {
	match v {
		Value::Integer(i) => u64::try_from(i128::from(*i)).ok(),
		#[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "checked: whole, 0..2^53")]
		Value::Float(f) if f.fract() == 0.0 && (0.0..9.0e15).contains(f) => Some(*f as u64),
		_ => None,
	}
}

/// A CBOR text as a str, if it is one.
#[must_use]
pub fn as_text(v: &Value) -> Option<&str> {
	match v {
		Value::Text(s) => Some(s),
		_ => None,
	}
}
