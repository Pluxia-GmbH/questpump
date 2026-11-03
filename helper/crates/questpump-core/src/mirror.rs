//! The `QuestPump_Mirror` addon folder the helper writes into the game's AddOns (rust-helper `protocol-and-mirror.md`,
//! "The mirror writer"; docs/plan.md slice 5): a TOC that depends on QuestPump, one `State.lua` setting
//! `QuestPumpMirror = {savedAt, pairing, rows, chars, profiles, profileKeys, proof, body}`, which the addon's
//! `ns.BindDB` merges when it is newer than what the client loaded (`proof`, the pumps' heart-rate check marks, is read
//! by the sheet at every load and never merged: slice 8; `body`, the body every device shares, is taken field by field
//! at every load, each field that changed later than the game's own change of it: 2026-10-09), each install's in its
//! own numbering of the characters with its own profiles, the app's rows beside the game's (slice 26), and
//! `Alive.lua`, the helper's sign of life (`QuestPumpMirrorAlive`,
//! its clock while the game runs, which tells the addon at every load that the helper runs). Rows are Lua strings,
//! never tables, and the files hold nothing but literals, so they cannot run anything. Every write of the TOC and of
//! `State.lua` goes through a temporary file, an fsync and a rename; `Alive.lua`'s skips the fsync. `parse` reads
//! a written file back and `parse_save` the game's own save of the addon (its baseline too), for the helper to take in
//! (`Session::adopt`); `SavesRead` says which saves the client wrote since the helper read them.

use std::collections::{BTreeMap, HashMap};
use std::fmt::Write as _;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use ciborium::Value;
use serde::{Deserialize, Serialize};

use crate::body::Body;
use crate::codes::kind;
use crate::lua::{self, LuaError};
use crate::proof::{HrSource, Proof};
use crate::row::Row;
use crate::verdict::Verdict;

/// The mirror folder's name beside `QuestPump` in `Interface/AddOns`.
pub const FOLDER: &str = "QuestPump_Mirror";
/// The helper's sign of life in that folder (`alive_lua`), the TOC's second file.
pub const ALIVE: &str = "Alive.lua";
/// The addon's TOC `## Interface` line when the helper was built: the mirror's TOC repeats the installed addon's own
/// line (`interface`), and this one only when that TOC cannot be read. `tools/lint.lua` keeps it and `ICON` equal to
/// `QuestPump.toc`'s.
pub const INTERFACE: &str = "16001, 120105";
/// The addon's TOC `## IconTexture`, its own icon (`Media/Icon.tga`, tools/helper-icons.py): the AddOns list groups the
/// mirror under QuestPump and draws a question mark for a row without an icon.
pub const ICON: &str = r"Interface\AddOns\QuestPump\Media\Icon";

/// The pairing block the addon's setup screen reads (`ns.Pairing`), written on pairing and in every mirror write.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct Pairing {
	/// The helper's version.
	pub helper: String,
	/// `ios` or `android` once a phone paired.
	pub phone: Option<String>,
	pub phone_since: Option<i64>,
	/// `watchos` or `wearos` from a watch's first sync; absent without one.
	pub watch: Option<String>,
	/// When a phone or a watch last held the link (unix seconds, kept to the hour), so the game can tell a pairing whose
	/// app has not connected for days; absent until one has.
	pub linked_at: Option<i64>,
	/// The newest addon version in the helper's verified feed (`addon::Feed`); absent before one was verified.
	pub addon_latest: Option<String>,
	/// `helper` while the helper manages the install the mirror goes into (its folders hold what the helper put there),
	/// `other` otherwise; absent before the helper looked.
	pub addon_by: Option<String>,
	/// When that version came out (unix seconds, the feed's `pub_date`).
	pub addon_out: Option<i64>,
	/// Whether the phone apps can be installed from the stores, from the same feed; absent means unknown, and the
	/// addon asks for the phone as it always did.
	pub phone_apps: Option<bool>,
}

impl Pairing {
	/// The block with no phone or watch: what the feed and the install said stays.
	#[must_use]
	pub fn without_devices(&self) -> Self {
		Self {
			helper: self.helper.clone(),
			addon_latest: self.addon_latest.clone(),
			addon_by: self.addon_by.clone(),
			addon_out: self.addon_out,
			phone_apps: self.phone_apps,
			..Self::default()
		}
	}
}

/// What the helper can read of the game, as the pairing block tells the addon's setup screen and, since 2026-10-09, every
/// response tells the phones and the watches (`link::Helper::sight`, its names on the wire).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Sight {
	/// It reads the game, or nothing is paired that needs it to: no field.
	Sees,
	/// The macOS Screen Recording grant is missing: `sees_game = false`.
	NoGrant,
	/// The capture stopped, or the strip it sees does not read: `capture = "stopped"` (the window says why).
	Stopped,
}

/// A character as the addon keeps it in `chars` (the index is its place in the list).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct Character {
	pub guid: String,
	pub name: Option<String>,
	pub realm: Option<String>,
	pub class: Option<String>,
}

/// Everything one `State.lua` holds, for the install whose folder it goes into (`session::Session::mirrored`): the
/// rows and the characters in that install's numbering, its profiles only.
#[derive(Debug, Clone)]
pub struct State<'a> {
	pub saved_at: i64,
	pub pairing: &'a Pairing,
	/// The time the pairing block states: a write that includes every row the helper has seen (the addon's stale-helper
	/// line compares it with its last row). Whether the helper runs now is `Alive.lua`'s to say, never this.
	pub seen_at: i64,
	/// Whether the helper can read the game, which the addon's setup screen asks about.
	pub sight: Sight,
	/// The game's rows (every install's, and the catch-ups) and the app's, merged by time.
	pub rows: &'a [Row],
	/// `chars[i]` is the install's character index `i + 1`: its own, then every other character the helper knows with
	/// a GUID; None for an index no profile event named yet, which the addon's merge skips.
	pub chars: &'a [Option<Character>],
	/// The install's profiles by name, as the addon's CBOR carried them (a move id keeps its integer key).
	pub profiles: &'a BTreeMap<String, Value>,
	/// The profile a character uses, by GUID, for the ones that chose one other than Default.
	pub profile_keys: &'a BTreeMap<String, String>,
	/// The newest pumps' heart-rate proofs by the pump's row time (`Book::mirror`).
	pub proofs: &'a [(u32, &'a Proof)],
	/// The body every device shares (`body::Kept::body`): its Body switches, its moves and since slice 51 Home's setup,
	/// each with the second it last changed, which the game takes field by field (DB.lua `ns.TakeBody`; the experience,
	/// the favourites, the title and the other places stay the apps').
	pub body: &'a Body,
}

/// What a `State.lua` holds, read back: the helper takes it in before its first write into that folder.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Held {
	pub saved_at: i64,
	pub rows: Vec<Row>,
	pub chars: Vec<Option<Character>>,
	pub profiles: BTreeMap<String, Value>,
	pub profile_keys: BTreeMap<String, String>,
	/// The pumps' heart-rate proofs by row time (a mirror's; a save has none).
	pub proof: BTreeMap<u32, Proof>,
	/// A save's baseline (DB.lua `ns.Trim`: what its ring let go, folded, with its horizon), as the addon wrote it; none
	/// for a mirror, for a save whose baseline is no table or an empty one, and for a save whose client never read it
	/// back (no `meta.readBackSeen`: the Forever beta, whose next load folds no baseline).
	pub baseline: Option<Value>,
}

/// The name serde gives a unit variant (`proven`, `watch`).
fn name<T: Serialize>(v: &T) -> Option<String> {
	serde_json::to_value(v).ok()?.as_str().map(str::to_owned)
}

/// The unit variant serde names so.
fn named<T: serde::de::DeserializeOwned>(s: &str) -> Option<T> {
	serde_json::from_value(serde_json::Value::String(s.to_owned())).ok()
}

fn field<'a>(map: &'a [(Value, Value)], key: &str) -> Option<&'a Value> {
	map.iter().find(|(k, _)| k.as_text() == Some(key)).map(|(_, v)| v)
}

/// A table's entries, whether it read as a list or a map (`{}` reads as an empty list).
fn entries(v: Option<&Value>) -> Vec<(Value, Value)> {
	match v {
		Some(Value::Map(m)) => m.clone(),
		Some(Value::Array(a)) => (1_u64..).zip(a.iter().cloned()).map(|(i, x)| (Value::Integer(i.into()), x)).collect(),
		_ => Vec::new(),
	}
}

/// The rows, characters, profiles and profile keys of a mirror's or a save's table (`rows` names its list of rows).
fn held(m: &[(Value, Value)], rows: &str) -> Held {
	let text_of = |v: &Value| v.as_text().map(str::to_owned);
	let mut held = Held {
		saved_at: field(m, "savedAt").and_then(Value::as_integer).and_then(|i| i64::try_from(i).ok()).unwrap_or(0),
		..Held::default()
	};
	for (_, v) in entries(field(m, rows)) {
		if let Some(row) = v.as_text().and_then(|s| s.parse().ok()) {
			held.rows.push(row);
		}
	}
	for (_, v) in entries(field(m, "chars")) {
		let c = v.as_map().and_then(|c| {
			let get = |k| field(c, k).and_then(text_of);
			Some(Character { guid: get("guid")?, name: get("name"), realm: get("realm"), class: get("class") })
		});
		held.chars.push(c);
	}
	for (k, v) in entries(field(m, "profiles")) {
		if let Some(name) = text_of(&k) {
			held.profiles.insert(name, v);
		}
	}
	for (k, v) in entries(field(m, "profileKeys")) {
		if let (Some(guid), Some(name)) = (text_of(&k), text_of(&v)) {
			held.profile_keys.insert(guid, name);
		}
	}
	for (k, v) in entries(field(m, "proof")) {
		let ts = k.as_integer().and_then(|i| u32::try_from(i).ok());
		let proof = v.as_map().and_then(|p| {
			let int = |key| field(p, key).and_then(Value::as_integer);
			Some(Proof {
				pump_id: field(p, "pump").and_then(text_of).unwrap_or_default(),
				verdict: named::<Verdict>(field(p, "verdict")?.as_text()?)?,
				peak: int("peak").and_then(|i| u16::try_from(i).ok()),
				delta: int("delta").and_then(|i| i32::try_from(i).ok()),
				source: field(p, "source").and_then(Value::as_text).and_then(named::<HrSource>),
			})
		});
		if let (Some(ts), Some(proof)) = (ts, proof) {
			held.proof.insert(ts, proof);
		}
	}
	held
}

/// The table a file assigns to `name`.
fn table(text: &str, name: &str) -> Result<Vec<(Value, Value)>, LuaError> {
	let found = lua::globals(text)?.into_iter().find(|(n, _)| n == name).map(|(_, v)| v);
	match found {
		Some(Value::Map(m)) => Ok(m),
		Some(Value::Array(a)) if a.is_empty() => Ok(Vec::new()),
		_ => Err(LuaError { at: 0, what: "no table of that name" }),
	}
}

/// Reads the text of a `State.lua` back. A row off the grammar, a character without a GUID and a key or value of the
/// wrong type are left out, as the addon's merge leaves them out.
///
/// # Errors
/// Text that is not Lua literals, or no `QuestPumpMirror` table in it.
pub fn parse(text: &str) -> Result<Held, LuaError> {
	Ok(held(&table(text, "QuestPumpMirror")?, "rows"))
}

/// Reads the game's own save of the addon (`WTF/Account/<account>/SavedVariables/QuestPump.lua`, the client writes it
/// at a logout or a `/reload`, and the beta never reads it back): the log's rows, the characters, the profiles, the
/// profile keys and, once its client read it back (`meta.readBackSeen`), the baseline (the history the ring let go,
/// which the engine's fold of the save starts from: `Session::game_save`), never the `backup` a reset keeps. When
/// `meta.resetAt` names a reset whose row the log no longer holds (the ring trims its oldest rows), that reset row
/// leads the rows, as `/qp reset all` wrote it (`Settings.lua`: the current character is index 1 of the new save).
///
/// # Errors
/// Text that is not Lua literals, or no `QuestPumpDB` table in it.
pub fn parse_save(text: &str) -> Result<Held, LuaError> {
	let db = table(text, "QuestPumpDB")?;
	let mut held = held(&db, "log");
	let meta = field(&db, "meta").and_then(Value::as_map);
	// the baseline only of a save its client reads back (DB.lua sets meta.readBackSeen only once a load read one): the
	// Forever beta never does, so its next load merges every mirror row into an empty save and folds them with no
	// baseline, and so does the engine's fold of that save (tools/apppump.lua `wholeFrom` dates its first whole day)
	let read_back = meta.and_then(|m| field(m, "readBackSeen")).and_then(Value::as_bool) == Some(true);
	held.baseline =
		field(&db, "baseline").filter(|b| read_back && matches!(b, Value::Map(m) if !m.is_empty())).cloned();
	let reset_at =
		meta.and_then(|meta| field(meta, "resetAt")).and_then(Value::as_integer).and_then(|i| u32::try_from(i).ok());
	if let Some(ts) = reset_at
		&& !held.rows.iter().any(|r| r.kind == kind::RESET && r.ts == ts)
	{
		held.rows.insert(0, Row { ts, char: 1, kind: kind::RESET, ..Row::default() });
	}
	Ok(held)
}

/// The game's saves of the addon beside `addons` (the game's `Interface/AddOns`): `WTF/Account/<account>/
/// SavedVariables/QuestPump.lua` for every account that has one, in name order.
#[must_use]
pub fn saves(addons: &Path) -> Vec<std::path::PathBuf> {
	let Some(accounts) = addons.parent().and_then(Path::parent).map(|game| game.join("WTF").join("Account")) else {
		return Vec::new();
	};
	let mut found: Vec<_> = std::fs::read_dir(accounts)
		.into_iter()
		.flatten()
		.flatten()
		.map(|e| e.path().join("SavedVariables").join("QuestPump.lua"))
		.filter(|p| p.is_file())
		.collect();
	found.sort();
	found
}

/// When the helper last read each of the game's saves (the file's modification time then), so a save the client
/// wrote since (a `/reload`, a logout, the game's exit) is taken in once, even when the strip never showed that load,
/// and an unchanged one is not read again.
#[derive(Debug, Default)]
pub struct SavesRead(HashMap<PathBuf, SystemTime>);

impl SavesRead {
	/// The saves beside `addons` (`saves`) the client wrote since the helper read them, each with its modification time
	/// now, taken before the read (a write during the read shows as written again). A save never read, or whose time
	/// the system cannot give, counts as written.
	#[must_use]
	pub fn written(&self, addons: &Path) -> Vec<(PathBuf, Option<SystemTime>)> {
		saves(addons)
			.into_iter()
			.filter_map(|path| {
				let at = std::fs::metadata(&path).and_then(|m| m.modified()).ok();
				(at.is_none() || self.0.get(&path) != at.as_ref()).then_some((path, at))
			})
			.collect()
	}

	/// A save read as it was at `at` (`written`'s time).
	pub fn read(&mut self, path: PathBuf, at: Option<SystemTime>) {
		if let Some(at) = at {
			self.0.insert(path, at);
		}
	}
}

/// The `State.lua` in the game's `Interface/AddOns`, whether or not it exists.
#[must_use]
pub fn state_path(addons: &Path) -> std::path::PathBuf {
	addons.join(FOLDER).join("State.lua")
}

/// A Lua string literal: `\\`, `"`, and every control character (below 0x20, and 0x7F) as a three-digit `\ddd`, so
/// the file holds one line per value and nothing can end the string early; the rest of the text as it is (UTF-8).
#[must_use]
pub fn lua_string(s: &str) -> String {
	let mut out = String::with_capacity(s.len() + 2);
	out.push('"');
	for c in s.chars() {
		match c {
			'\\' => out.push_str("\\\\"),
			'"' => out.push_str("\\\""),
			'\0'..='\u{1f}' | '\u{7f}' => {
				let _ = write!(out, "\\{:03}", u32::from(c));
			}
			_ => out.push(c),
		}
	}
	out.push('"');
	out
}

/// A CBOR byte string as a Lua string literal: every byte outside printable ASCII as `\ddd`, since it need not be
/// UTF-8.
fn lua_bytes(bytes: &[u8]) -> String {
	let mut out = String::with_capacity(bytes.len() + 2);
	out.push('"');
	for &b in bytes {
		match b {
			b'\\' => out.push_str("\\\\"),
			b'"' => out.push_str("\\\""),
			0x20..=0x7E => out.push(char::from(b)),
			_ => {
				let _ = write!(out, "\\{b:03}");
			}
		}
	}
	out.push('"');
	out
}

fn lua_number_f64(f: f64) -> Option<String> {
	if !f.is_finite() {
		return None;
	}
	let s = format!("{f:?}"); // the shortest text that reads back as the same double
	Some(s)
}

/// A CBOR value as a Lua literal: a map as `{ ["key"] = value, [n] = value }` with its keys sorted, an array as a list,
/// numbers, booleans and strings as themselves. None for a value Lua cannot hold (null, NaN), which is left out.
#[must_use]
pub fn lua_value(v: &Value, indent: &str) -> Option<String> {
	Some(match v {
		Value::Integer(i) => i128::from(*i).to_string(),
		Value::Float(f) => lua_number_f64(*f)?,
		Value::Text(s) => lua_string(s),
		Value::Bytes(b) => lua_bytes(b),
		Value::Bool(b) => b.to_string(),
		Value::Tag(_, inner) => lua_value(inner, indent)?,
		Value::Array(a) => {
			let inner = format!("{indent}\t");
			// a value Lua cannot hold stays a nil in its place, so every later item keeps its index
			let items: Vec<String> = a.iter().map(|x| lua_value(x, &inner).unwrap_or_else(|| "nil".into())).collect();
			if items.is_empty() {
				"{}".to_owned()
			} else {
				format!("{{\n{inner}{}\n{indent}}}", items.join(&format!(",\n{inner}")))
			}
		}
		Value::Map(m) => {
			let inner = format!("{indent}\t");
			let mut entries: Vec<(String, String)> =
				m.iter().filter_map(|(k, x)| Some((lua_key(k)?, lua_value(x, &inner)?))).collect();
			entries.sort();
			if entries.is_empty() {
				"{}".to_owned()
			} else {
				let lines: Vec<String> = entries.into_iter().map(|(k, val)| format!("{inner}{k} = {val}")).collect();
				format!("{{\n{}\n{indent}}}", lines.join(",\n"))
			}
		}
		_ => return None,
	})
}

/// A map key as Lua writes it: an integer (and a float that holds one, as a CBOR encoder may write a Lua number) as
/// `[n]`, another number as itself, a string quoted, a boolean as itself; any other key is left out.
fn lua_key(k: &Value) -> Option<String> {
	Some(match k {
		Value::Integer(i) => format!("[{}]", i128::from(*i)),
		Value::Float(f) if f.is_finite() && f.fract() == 0.0 && f.abs() < 9.0e15 => format!("[{f:.0}]"),
		Value::Float(f) => format!("[{}]", lua_number_f64(*f)?),
		Value::Text(s) => format!("[{}]", lua_string(s)),
		Value::Bool(b) => format!("[{b}]"),
		_ => return None,
	})
}

fn optional(out: &mut String, key: &str, value: Option<String>) {
	if let Some(v) = value {
		let _ = writeln!(out, "\t\t{key} = {v},");
	}
}

/// The text of `State.lua`.
#[must_use]
pub fn state_lua(s: &State<'_>) -> String {
	let mut out = String::new();
	out.push_str("-- Written by QuestPump on this computer, rewritten on every change: never edit it.\n");
	out.push_str("QuestPumpMirror = {\n");
	let _ = writeln!(out, "\tsavedAt = {},", s.saved_at);
	out.push_str("\tpairing = {\n");
	let _ = writeln!(out, "\t\thelper = {},", lua_string(&s.pairing.helper));
	optional(&mut out, "phone", s.pairing.phone.as_deref().map(lua_string));
	optional(&mut out, "phone_since", s.pairing.phone_since.map(|t| t.to_string()));
	optional(&mut out, "watch", s.pairing.watch.as_deref().map(lua_string));
	optional(&mut out, "linked_at", s.pairing.linked_at.map(|t| t.to_string()));
	optional(&mut out, "addon_latest", s.pairing.addon_latest.as_deref().map(lua_string));
	optional(&mut out, "addon_by", s.pairing.addon_by.as_deref().map(lua_string));
	optional(&mut out, "addon_out", s.pairing.addon_out.map(|t| t.to_string()));
	optional(&mut out, "phone_apps", s.pairing.phone_apps.map(|b| b.to_string()));
	match s.sight {
		Sight::Sees => {}
		Sight::NoGrant => out.push_str("\t\tsees_game = false,\n"),
		Sight::Stopped => out.push_str("\t\tcapture = \"stopped\",\n"),
	}
	let _ = writeln!(out, "\t\tseen_at = {},", s.seen_at);
	out.push_str("\t},\n");
	out.push_str("\trows = {\n");
	for row in s.rows {
		let _ = writeln!(out, "\t\t\"{row}\",");
	}
	out.push_str("\t},\n");
	out.push_str("\tchars = {\n");
	for c in s.chars {
		match c {
			Some(c) => {
				let mut fields = vec![format!("guid = {}", lua_string(&c.guid))];
				for (k, v) in [("name", &c.name), ("realm", &c.realm), ("class", &c.class)] {
					if let Some(v) = v {
						fields.push(format!("{k} = {}", lua_string(v)));
					}
				}
				let _ = writeln!(out, "\t\t{{ {} }},", fields.join(", "));
			}
			None => out.push_str("\t\t{},\n"), // an index no profile event named: the addon keeps the row's own index
		}
	}
	out.push_str("\t},\n");
	out.push_str("\tprofiles = {\n");
	for (name, profile) in s.profiles {
		if let Some(v) = lua_value(profile, "\t\t") {
			let _ = writeln!(out, "\t\t[{}] = {v},", lua_string(name));
		}
	}
	out.push_str("\t},\n");
	out.push_str("\tprofileKeys = {\n");
	for (guid, name) in s.profile_keys {
		let _ = writeln!(out, "\t\t[{}] = {},", lua_string(guid), lua_string(name));
	}
	out.push_str("\t},\n");
	out.push_str("\tproof = {\n");
	for (ts, p) in s.proofs {
		let Some(verdict) = name(&p.verdict) else { continue };
		let mut fields = vec![format!("verdict = {}", lua_string(&verdict))];
		if let Some(peak) = p.peak {
			fields.push(format!("peak = {peak}"));
		}
		if let Some(delta) = p.delta {
			fields.push(format!("delta = {delta}"));
		}
		if let Some(source) = p.source.as_ref().and_then(name) {
			fields.push(format!("source = {}", lua_string(&source)));
		}
		fields.push(format!("pump = {}", lua_string(&p.pump_id)));
		let _ = writeln!(out, "\t\t[{ts}] = {{ {} }},", fields.join(", "));
	}
	out.push_str("\t},\n");
	let body = lua_value(&body_value(s.body), "\t").unwrap_or_else(|| "{}".to_owned());
	let _ = writeln!(out, "\tbody = {body},");
	out.push_str("}\n");
	out
}

/// The mirror's `body` as the game takes it at its next load (DB.lua `ns.TakeBody`): each Body switch a device changed
/// as `{ value, at }`, the moves as `{ at, ids, groups, include }` and Home's setup (slice 51) as
/// `{ at, gear, floor, space, quiet }`, each with the second it changed (`at`, the
/// change's milliseconds rounded down, as the addon stamps its own: DB.lua `ns.BodyStamp`). `state_lua` writes it into
/// `State.lua`, and the engine's fold of the game's save takes it as that load does (`session::GameSave`).
#[must_use]
pub fn body_value(body: &Body) -> Value {
	let text = |s: &str| Value::Text(s.to_owned());
	let at = |ms: i64| (text("at"), Value::Integer(ms.div_euclid(1_000).into()));
	let ids = |ids: &[u16]| Value::Array(ids.iter().map(|&id| Value::Integer(id.into())).collect());
	let mut out = Vec::new();
	for (key, field) in [("noJump", &body.no_jump), ("noStrain", &body.no_strain), ("noHeadDown", &body.no_head_down)] {
		if let Some(f) = field {
			out.push((text(key), Value::Map(vec![(text("value"), Value::Bool(f.value)), at(f.changed_at)])));
		}
	}
	if let Some(m) = &body.moves {
		let v = &m.value;
		let moves = vec![
			at(m.changed_at),
			(text("ids"), ids(&v.exclude.ids)),
			(text("groups"), Value::Array(v.exclude.groups.iter().map(|g| text(g)).collect())),
			(text("include"), ids(&v.include)),
		];
		out.push((text("moves"), Value::Map(moves)));
	}
	// Home's setup (slice 51), which DB.lua `ns.TakeBody` takes when it changed later than the profile's `bodyAt.home`;
	// the other places never reach the game
	if let Some(h) = &body.home {
		let v = &h.value;
		let home = vec![
			at(h.changed_at),
			(text("gear"), Value::Array(v.gear.iter().map(|g| text(g)).collect())),
			(text("floor"), Value::Bool(v.floor)),
			(text("space"), Value::Bool(v.space)),
			(text("quiet"), Value::Bool(v.quiet)),
		];
		out.push((text("home"), Value::Map(home)));
	}
	Value::Map(out)
}

/// A TOC's `## <field>:` value, trimmed, when the line is there and holds one.
#[must_use]
pub fn toc_field<'a>(text: &'a str, field: &str) -> Option<&'a str> {
	text.lines()
		.find_map(|l| l.strip_prefix("## ")?.strip_prefix(field)?.strip_prefix(':'))
		.map(str::trim)
		.filter(|v| !v.is_empty())
}

/// The installed addon's `## Interface` line (`Interface/AddOns/QuestPump/QuestPump.toc`), so a client that moves on
/// to a new interface number loads the mirror as it loads the addon, whatever the helper's own age.
#[must_use]
pub fn interface(addons: &Path) -> String {
	std::fs::read_to_string(addons.join("QuestPump").join("QuestPump.toc"))
		.ok()
		.and_then(|text| toc_field(&text, "Interface").map(str::to_owned))
		.unwrap_or_else(|| INTERFACE.to_owned())
}

/// The mirror's TOC: written once, and again only when its text changes (the Interface line, a file the helper adds).
#[must_use]
pub fn toc(interface: &str) -> String {
	format!(
		"## Interface: {interface}\n## Title: QuestPump Mirror\n## Notes: Written by QuestPump on this computer: your \
		 progress, kept safe for the game.\n## Author: Pluxia GmbH\n## IconTexture: {ICON}\n## Dependencies: \
		 QuestPump\n## X-QuestPump-Generated: true\n\nState.lua\n{ALIVE}\n"
	)
}

/// The text of `Alive.lua`: the helper's clock at its last sign of life. The addon reads it at every load
/// (`ns.HelperState`): a helper that runs wrote it shortly before, whatever the kind of load.
#[must_use]
pub fn alive_lua(unix: i64) -> String {
	format!(
		"-- Written by QuestPump on this computer while the game runs: never edit it.\nQuestPumpMirrorAlive = {unix}\n"
	)
}

/// Writes `name` in `dir` through a file of its own (`temp`), an fsync and a rename (atomic on POSIX, POSIX semantics
/// on Windows 10 1607+ NTFS). A rename the game blocks on Windows (access denied 5, a sharing violation 32) is tried
/// again every 200 ms for 5 s; the same numbers elsewhere (EIO, EPIPE on macOS) fail at once.
///
/// # Errors
/// The write or the last rename's error.
pub fn write_atomic(dir: &Path, name: &str, bytes: &[u8]) -> std::io::Result<()> {
	replace(dir, name, bytes, true)
}

/// Writes `name` in `dir` as `write_atomic` does, without the fsync and the tries again: a file written again seconds
/// later, or one written as the program dies (a panic's report).
///
/// # Errors
/// The write or the rename's error.
pub fn write_quick(dir: &Path, name: &str, bytes: &[u8]) -> std::io::Result<()> {
	replace(dir, name, bytes, false)
}

/// A data file `name` in `dir` as `parse` reads its bytes: none while it is absent. One that is there but does not read
/// (a disk error, a hand edit, an earlier build's shape) is kept aside (`keep_aside`) before anything replaces it, and
/// comes back as why, for the caller to log and to start from its default.
///
/// # Errors
/// Why the file did not read, and where it was kept aside.
pub fn read_data<T>(
	dir: &Path,
	name: &str,
	parse: impl FnOnce(&[u8]) -> Result<T, String>,
) -> Result<Option<T>, String> {
	let why = match std::fs::read(dir.join(name)) {
		Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
		Err(e) => e.to_string(),
		Ok(bytes) => match parse(&bytes) {
			Ok(value) => return Ok(Some(value)),
			Err(e) => e,
		},
	};
	Err(match keep_aside(dir, name) {
		Ok(()) => format!("{why}; kept aside as {name}.unread"),
		Err(e) => format!("{why}; {name}.unread: {e}"),
	})
}

/// Copies the data file `name` in `dir` aside as `<name>.unread` (an earlier copy replaced): a file that did not read,
/// or read only in part, is never replaced unread by the next save.
///
/// # Errors
/// The copy's.
pub fn keep_aside(dir: &Path, name: &str) -> std::io::Result<()> {
	std::fs::copy(dir.join(name), dir.join(format!("{name}.unread"))).map(drop)
}

/// `name` in `dir` through a file of its own (`temp`) and a rename, the file gone when either fails. `durable` adds the
/// fsync and, on Windows, the tries again; a file written again seconds later goes without both.
fn replace(dir: &Path, name: &str, bytes: &[u8], durable: bool) -> std::io::Result<()> {
	let dst = dir.join(name);
	let (tmp, mut f) = temp(dir, name)?;
	let written = f.write_all(bytes).and_then(|()| if durable { f.sync_all() } else { Ok(()) });
	drop(f);
	let placed = written.and_then(|()| if durable { rename_retried(&tmp, &dst) } else { std::fs::rename(&tmp, &dst) });
	if placed.is_err() {
		let _ = std::fs::remove_file(&tmp);
	}
	placed
}

/// A new file beside `name` in `dir` to write it through, `<name>.<pid>-<random>.tmp`, created exclusively: never a
/// file or a link that is there already (a game's folder may be another account's to write in; on Windows
/// `create_new` opens no reparse point either), and never another writer's. A name that is there is tried again with
/// a new one, twice.
///
/// # Errors
/// The create's.
pub fn temp(dir: &Path, name: &str) -> std::io::Result<(PathBuf, std::fs::File)> {
	use std::hash::BuildHasher as _;
	let mut tries = 0;
	loop {
		// a SipHash of the clock under keys of this process's own: no other program can guess it
		let random = std::collections::hash_map::RandomState::new().hash_one(SystemTime::now());
		let path = dir.join(format!("{name}.{}-{random:016x}.tmp", std::process::id()));
		match std::fs::File::create_new(&path) {
			Ok(file) => return Ok((path, file)),
			Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists && tries < 2 => tries += 1,
			Err(e) => return Err(e),
		}
	}
}

/// Whether the file named `file` is a write of `name` that never reached its rename (a crash or a power cut between
/// them): `temp`'s `<name>.<pid>-<random>.tmp`, or the `<name>.tmp` of builds before 2026-11-01.
#[must_use]
pub fn is_temp_of(file: &str, name: &str) -> bool {
	file.strip_prefix(name)
		.and_then(|rest| rest.strip_prefix('.'))
		.is_some_and(|rest| rest == "tmp" || rest.rsplit_once('.').is_some_and(|(_, last)| last == "tmp"))
}

/// A rename tried again every 200 ms for 5 s on Windows while another program holds the file or a file inside the
/// folder (os errors 5 and 32: the game reading a file, a virus scanner looking at one just written); elsewhere once.
///
/// # Errors
/// The rename's, once the tries are over.
pub fn rename_retried(from: &Path, to: &Path) -> std::io::Result<()> {
	let mut waited = 0;
	loop {
		match std::fs::rename(from, to) {
			Ok(()) => return Ok(()),
			Err(e) if cfg!(windows) && matches!(e.raw_os_error(), Some(5 | 32)) && waited < 5_000 => {
				std::thread::sleep(Duration::from_millis(200));
				waited += 200;
			}
			Err(e) => return Err(e),
		}
	}
}

/// Writes the mirror folder into `addons` (the game's `Interface/AddOns`): the TOC when it is missing or stale, then
/// `State.lua` (the text `state_lua` made, so the caller can build it and leave the slow write to another thread).
///
/// # Errors
/// A folder or file that cannot be written.
pub fn write(addons: &Path, state_lua: &str) -> std::io::Result<()> {
	let dir = addons.join(FOLDER);
	std::fs::create_dir_all(&dir)?;
	let toc_path = dir.join(format!("{FOLDER}.toc"));
	let toc = toc(&interface(addons));
	if std::fs::read_to_string(&toc_path).map_or(true, |t| t != toc) {
		write_atomic(&dir, &format!("{FOLDER}.toc"), toc.as_bytes())?;
	}
	write_atomic(&dir, "State.lua", state_lua.as_bytes())
}

/// Writes `Alive.lua` into the mirror folder `write` made in `addons`, the helper's clock at `unix`: no fsync and one
/// rename, since the next one comes seconds later and a lost one only reads as a helper that is not running.
///
/// # Errors
/// No mirror folder there yet, or a file that cannot be written.
pub fn write_alive(addons: &Path, unix: i64) -> std::io::Result<()> {
	write_quick(&addons.join(FOLDER), ALIVE, alive_lua(unix).as_bytes())
}
