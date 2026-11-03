//! Steam's own record of every app it knows, `appcache/appinfo.vdf` (docs/plan.md slice 56, design 7; the format as
//! the `SteamDatabase/SteamAppInfo` README and parser and `ValveKeyValue`'s binary reader read it, 2026-10-25), read for
//! the type Steam gives the apps asked (`common` → `type`: `game`, `demo`, `mod`, `tool`, `application`, `dlc`, `music`,
//! …), so the detector notices a game and never a tool Steam launches (Wallpaper Engine, Steam's Tools and
//! Applications). The file: a header (its magic, `07 56 44` above a version byte of 39, 40 or 41; the universe; from 41
//! the string table's offset, an int64 from the file's start), then each app (its id; the size of the rest of its entry
//! up to its key-values' end; its info state, its last change, its PICS token, its text's SHA-1 and its change number;
//! from 40 its key-values' SHA-1; then its binary key-values) until an app id of 0. From 41 a string table at that
//! offset (a count, then that many NUL-ended strings) holds every key: a key is an int32 index into it, where 39 and 40
//! write the key itself NUL-ended. A pair is a type byte (`OBJECT` and its pairs up to an end byte, `STRING` NUL-ended
//! UTF-8, `WIDE` an int32 count of UTF-16 units with its NUL, `BLOB` a uint32 length, the numbers 4 or 8 bytes), its key
//! and its value. An app not asked is skipped by its size, unread. A file of another version, or cut short before an
//! app asked, gives no type for it: the detector then holds the app a game, as it did before it read Steam's types.

use std::collections::{HashMap, HashSet};

/// The magic's three bytes above its version byte.
const MAGIC: u32 = 0x07_56_44;
/// The versions read: 39 (2017), 40 (December 2022, the key-values' SHA-1), 41 (June 2024, the string table).
const VERSIONS: std::ops::RangeInclusive<u32> = 39..=41;
/// An entry's fields between its size and its key-values: info state, last change, PICS token, SHA-1, change number.
const ENTRY_HEAD: usize = 4 + 4 + 8 + 20 + 4;
/// The key-values' SHA-1, from version 40.
const BINARY_SHA: usize = 20;
/// The deepest an app's key-values nest: Steam's nest a handful of levels; a deeper one is no Steam entry.
const MAX_DEPTH: usize = 32;

/// The binary key-values' type bytes (`ValveKeyValue`'s `KV1BinaryNodeType`).
const OBJECT: u8 = 0;
const STRING: u8 = 1;
const INT32: u8 = 2;
const FLOAT32: u8 = 3;
const POINTER: u8 = 4;
const WIDE: u8 = 5;
const COLOR: u8 = 6;
const UINT64: u8 = 7;
const END: u8 = 8;
const BLOB: u8 = 9;
const INT64: u8 = 10;
const ALTERNATE_END: u8 = 11;

/// The types Steam gives an app the detector may notice: a game, a game's demo, a mod of a game.
const NOTICED: [&str; 3] = ["game", "demo", "mod"];

/// Whether an app of the type `kind` (as `types` gives it) is one the detector notices: a game, a demo or a mod.
#[must_use]
pub fn noticed(kind: &str) -> bool {
	NOTICED.contains(&kind)
}

/// The type Steam's `appinfo.vdf` (its bytes) gives each of the apps `asked`, in lower case, by app id; an app the
/// file does not hold, or whose entry does not read, is left out.
#[must_use]
pub fn types(bytes: &[u8], asked: &[u32]) -> HashMap<u32, String> {
	let asked: HashSet<u32> = asked.iter().copied().collect();
	let mut out = HashMap::new();
	let mut c = Cursor { bytes, at: 0 };
	let Some(magic) = c.u32() else { return out };
	let version = magic & 0xFF;
	if magic >> 8 != MAGIC || !VERSIONS.contains(&version) || c.u32().is_none() {
		return out;
	}
	let names = if version >= 41 {
		let table = c.i64().and_then(|at| usize::try_from(at).ok()).and_then(|at| bytes.get(at..));
		match table.and_then(Names::table) {
			Some(names) => names,
			None => return out,
		}
	} else {
		Names::Inline
	};
	while out.len() < asked.len() {
		let (Some(app), Some(size)) = (c.u32(), c.u32()) else { break };
		let end = usize::try_from(size).ok().and_then(|size| c.at.checked_add(size)).filter(|&end| end <= bytes.len());
		let Some(end) = end.filter(|_| app != 0) else { break };
		if asked.contains(&app) {
			let head = ENTRY_HEAD + if version >= 40 { BINARY_SHA } else { 0 };
			let mut entry = Cursor { bytes: &bytes[..end], at: c.at };
			if let Some(kind) = entry.skip(head).and_then(|()| app_type(&mut entry, &names)) {
				out.insert(app, String::from_utf8_lossy(kind).to_ascii_lowercase());
			}
		}
		c.at = end;
	}
	out
}

/// The key a type is looked for under: `common`, then `type` in it.
#[derive(Clone, Copy)]
enum Name {
	Common,
	Type,
}

impl Name {
	fn text(self) -> &'static [u8] {
		match self {
			Self::Common => b"common",
			Self::Type => b"type",
		}
	}
}

/// How the file writes its keys: each key itself (39, 40), or an index into its string table (41), where the two
/// names looked for stand at these places (none: the table lacks it, so no key is it).
enum Names {
	Inline,
	Table { common: Option<u32>, kind: Option<u32> },
}

/// A key as an entry writes it.
enum Key<'a> {
	Text(&'a [u8]),
	Index(u32),
}

impl Names {
	/// Version 41's string table (the file from its offset on): a count, then that many NUL-ended strings, of which
	/// only the places of `common` and `type` are kept; none for a table cut short.
	fn table(bytes: &[u8]) -> Option<Self> {
		let mut c = Cursor { bytes, at: 0 };
		let count = c.u32()?;
		let (mut common, mut kind) = (None, None);
		for at in 0..count {
			let text = c.text()?;
			if common.is_none() && text.eq_ignore_ascii_case(Name::Common.text()) {
				common = Some(at);
			} else if kind.is_none() && text.eq_ignore_ascii_case(Name::Type.text()) {
				kind = Some(at);
			}
		}
		Some(Self::Table { common, kind })
	}

	/// The next key.
	fn key<'a>(&self, c: &mut Cursor<'a>) -> Option<Key<'a>> {
		match self {
			Self::Inline => c.text().map(Key::Text),
			Self::Table { .. } => c.u32().map(Key::Index),
		}
	}

	/// Whether a key is the name `name`, in any case.
	fn is(&self, key: &Key<'_>, name: Name) -> bool {
		match (self, key) {
			(Self::Inline, Key::Text(text)) => text.eq_ignore_ascii_case(name.text()),
			(Self::Table { common, kind }, Key::Index(at)) => {
				let place = match name {
					Name::Common => common,
					Name::Type => kind,
				};
				*place == Some(*at)
			}
			_ => false,
		}
	}
}

/// An app's key-values: its root object's `common` → `type`, its other pairs skipped.
fn app_type<'a>(c: &mut Cursor<'a>, names: &Names) -> Option<&'a [u8]> {
	if c.u8()? != OBJECT {
		return None;
	}
	names.key(c)?;
	find(c, names, &[Name::Common, Name::Type], 1)
}

/// The text at `path` (each a key one object further in) in the object at the cursor, its other pairs skipped; none
/// when the object ends without it or does not read.
fn find<'a>(c: &mut Cursor<'a>, names: &Names, path: &[Name], depth: usize) -> Option<&'a [u8]> {
	let (&want, rest) = path.split_first()?;
	if depth > MAX_DEPTH {
		return None;
	}
	loop {
		let kind = c.u8()?;
		if matches!(kind, END | ALTERNATE_END) {
			return None;
		}
		let key = names.key(c)?;
		let hit = names.is(&key, want);
		match kind {
			OBJECT if hit && !rest.is_empty() => return find(c, names, rest, depth + 1),
			STRING if hit && rest.is_empty() => return c.text(),
			_ => skip(c, names, kind, depth)?,
		}
	}
}

/// Skips a value of the type `kind`.
fn skip(c: &mut Cursor<'_>, names: &Names, kind: u8, depth: usize) -> Option<()> {
	match kind {
		OBJECT => {
			if depth >= MAX_DEPTH {
				return None;
			}
			loop {
				let inner = c.u8()?;
				if matches!(inner, END | ALTERNATE_END) {
					return Some(());
				}
				names.key(c)?;
				skip(c, names, inner, depth + 1)?;
			}
		}
		STRING => c.text().map(|_| ()),
		INT32 | FLOAT32 | POINTER | COLOR => c.skip(4),
		UINT64 | INT64 => c.skip(8),
		WIDE => {
			let units = usize::try_from(c.u32()?).ok()?;
			c.skip(units.checked_mul(2)?)
		}
		BLOB => {
			let len = usize::try_from(c.u32()?).ok()?;
			c.skip(len)
		}
		_ => None,
	}
}

/// A place in the bytes, read little-endian; every read stays within them.
struct Cursor<'a> {
	bytes: &'a [u8],
	at: usize,
}

impl<'a> Cursor<'a> {
	fn take(&mut self, n: usize) -> Option<&'a [u8]> {
		let end = self.at.checked_add(n)?;
		let taken = self.bytes.get(self.at..end)?;
		self.at = end;
		Some(taken)
	}

	fn skip(&mut self, n: usize) -> Option<()> {
		self.take(n).map(|_| ())
	}

	fn u8(&mut self) -> Option<u8> {
		self.take(1).map(|b| b[0])
	}

	fn u32(&mut self) -> Option<u32> {
		self.take(4).and_then(|b| b.try_into().ok()).map(u32::from_le_bytes)
	}

	fn i64(&mut self) -> Option<i64> {
		self.take(8).and_then(|b| b.try_into().ok()).map(i64::from_le_bytes)
	}

	/// A NUL-ended string, without its NUL.
	fn text(&mut self) -> Option<&'a [u8]> {
		let rest = self.bytes.get(self.at..)?;
		let len = rest.iter().position(|&b| b == 0)?;
		let text = &rest[..len];
		self.at += len + 1;
		Some(text)
	}
}
