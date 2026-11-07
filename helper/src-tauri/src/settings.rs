//! The desktop app's settings file (`settings.json` in its data folder): its shape is the desk's
//! (`questpump_desk::settings`, re-exported here, whose doc lists what it keeps); reading it, saving it at once for the
//! start's sweep and the secrets' keeper every save goes through are the app's, since a browser keeps its settings
//! elsewhere. A value that does not read takes its default alone (`read`), so only a pairing key that neither the file
//! nor the store holds starts every pairing again.

use std::path::Path;

use questpump_core::mirror;
pub(crate) use questpump_desk::settings::*;
use serde_json::{Map, Value};

use crate::secrets::Keeper;

/// Saved at once on this thread through the secrets' keeper, for the start's sweep (`runtime::sweep`), which runs
/// before the writer thread exists: a start that crashes before its own save then never finds the reports it swept
/// again, and secrets the file still held are in the store before anything else runs. A failure is a log line, and
/// the start's own save tries again.
pub(crate) fn save_now(settings: &Settings, dir: &Path, keeper: &mut Keeper) {
	let saved = serde_json::to_value(settings)
		.and_then(|file| keeper.bytes(file))
		.map_err(std::io::Error::from)
		.and_then(|bytes| mirror::write_atomic(dir, FILE, &bytes));
	if let Err(e) = saved {
		log::warn!("{FILE}: {e}");
	}
}

/// Reads the settings, making a pairing key and an install id when there are none, switching Room to move on once
/// in a place kept before it was on by default (`space_on`), dropping once a window's place kept at the old first
/// size (`window_grown`) and turning a game found before this build named it into the named game
/// (`games::drop_named`); the helper's version is always its own. The secrets come from their store
/// (`Keeper::open`, which every later save goes through), into a file that is gone or does not read as well, so
/// neither ends the pairing or the account the store still holds. A file that does not read whole is kept aside and
/// logged, never silently replaced.
pub(crate) fn load(dir: &Path) -> (Settings, Loaded, Keeper) {
	let file =
		mirror::read_data(dir, FILE, |b| serde_json::from_slice::<Map<String, Value>>(b).map_err(|e| e.to_string()));
	let (mut file, gone) = match file {
		Ok(Some(file)) => (Value::Object(file), None),
		Ok(None) => (Value::Object(Map::new()), Some(Loaded::First)),
		Err(why) => {
			log::warn!("{FILE}: {why}");
			(Value::Object(Map::new()), Some(Loaded::Unread))
		}
	};
	let keeper = Keeper::open(dir, &mut file);
	let (mut s, loaded) = match gone {
		None => read(dir, &file),
		Some(gone) => kept(&file, gone),
	};
	if loaded != Loaded::Read {
		s.new_key();
		s.hid = new_hid();
	}
	if !s.space_on {
		// the start's own save (`Helper::start`) keeps the mark
		if s.place.floor && !s.place.space {
			s.place.space = true;
			log::info!("{FILE}: Room to move switched on with the floor moves, its default since 2026-10-09");
		}
		s.space_on = true;
	}
	let found = s.games.drop_found_programs();
	if found > 0 {
		// the start's own save keeps it
		log::info!("{FILE}: {found} found games turned on left the picked programs: found games are watched now");
	}
	// a game found before this build named it is the named game now (the start's own save keeps it)
	crate::games::drop_named(&mut s.games);
	if !s.window_grown {
		// the start's own save keeps the mark, as `space_on`'s
		if s.window.as_ref().is_some_and(crate::desk::place::never_resized) {
			s.window = None;
			log::info!("{FILE}: the window's place at the old first size, 520 × 760, went: it opens at the new one");
		}
		s.window_grown = true;
	}
	env!("CARGO_PKG_VERSION").clone_into(&mut s.pairing.helper);
	(s, loaded, keeper)
}

/// The settings in the file's JSON: as saved, or else value by value (an earlier or later build's shape, a hand
/// edit), each value that does not read at its default alone, logged, and the file kept aside. Only a pairing key
/// that does not read starts every pairing again, so no change of these settings' shape ever costs a pairing
/// (2026-10-08, after a dropped welcome step would have made every settings file of that step unreadable).
fn read(dir: &Path, file: &Value) -> (Settings, Loaded) {
	if let Ok(s) = serde_json::from_value::<Settings>(file.clone()) {
		return (s, Loaded::Read);
	}
	let Ok(mut kept) = serde_json::to_value(Settings::default()) else { return (Settings::default(), Loaded::Unread) };
	let mut dropped = Vec::new();
	for (name, value) in file.as_object().into_iter().flatten() {
		overlay(&mut kept, "", name, value, &mut dropped);
	}
	let Ok(mut s) = serde_json::from_value::<Settings>(kept) else { return (Settings::default(), Loaded::Unread) };
	let aside = match mirror::keep_aside(dir, FILE) {
		// copy-lint: not shown (a log line's words)
		Ok(()) => format!("kept aside as {FILE}.unread"),
		Err(e) => format!("{FILE}.unread: {e}"),
	};
	let read = |field: &str| file.get(field).is_some() && !dropped.iter().any(|d| d == &format!("/{field}"));
	if !read("key") {
		log::warn!("{FILE}: the pairing key did not read ({aside}), and every pairing starts again");
		// the new key ends every pairing: no phone, watch or name stays for the mirror, the tray or the pages
		s.pairing = s.pairing.without_devices();
		s.phone_name = None;
		return (s, Loaded::Unread);
	}
	if dropped.is_empty() {
		log::warn!("{FILE}: a value this build needs was missing, at its default ({aside})");
	} else {
		log::warn!("{FILE}: {} did not read, each at its default ({aside})", dropped.join(", "));
	}
	if !read("hid") {
		s.hid = new_hid();
	}
	(s, Loaded::Read)
}

/// The settings of a file that is gone (`First`) or did not read (`Unread`): the defaults, with what the secrets'
/// keeper laid in (`Keeper::open`: the pairing key, the install's id, the sync's R and a rekey's R'), read back when
/// the key came.
fn kept(file: &Value, gone: Loaded) -> (Settings, Loaded) {
	let Ok(mut kept) = serde_json::to_value(Settings::default()) else { return (Settings::default(), gone) };
	let mut dropped = Vec::new();
	for (name, value) in file.as_object().into_iter().flatten() {
		overlay(&mut kept, "", name, value, &mut dropped);
	}
	let mut s = serde_json::from_value::<Settings>(kept).unwrap_or_default();
	let came = |field: &str| file.get(field).is_some() && !dropped.iter().any(|d| d == &format!("/{field}"));
	if !came("key") {
		if gone == Loaded::Unread {
			log::warn!("{FILE}: every pairing starts again");
		}
		return (s, gone);
	}
	log::info!("{FILE}: none that read, the pairing key from the secrets' keeper");
	if !came("hid") {
		s.hid = new_hid();
	}
	(s, Loaded::Read)
}

/// Lays the file's `value` of the field `name` of the object at `parent` (a JSON pointer) over `kept`, the settings as
/// JSON: an object the defaults hold too field by field, any other value whole, kept only while `kept` still reads as
/// settings; one that does not is named in `dropped` by its pointer and stays at its default.
fn overlay(kept: &mut Value, parent: &str, name: &str, value: &Value, dropped: &mut Vec<String>) {
	let at = format!("{parent}/{}", name.replace('~', "~0").replace('/', "~1"));
	if let (Some(Value::Object(_)), Value::Object(fields)) = (kept.pointer(&at), value) {
		for (field, inner) in fields {
			overlay(kept, &at, field, inner, dropped);
		}
		return;
	}
	let Some(object) = kept.pointer_mut(parent).and_then(Value::as_object_mut) else { return };
	let before = object.insert(name.to_owned(), value.clone());
	if serde_json::from_value::<Settings>(kept.clone()).is_ok() {
		return;
	}
	if let Some(object) = kept.pointer_mut(parent).and_then(Value::as_object_mut) {
		match before {
			Some(before) => object.insert(name.to_owned(), before),
			None => object.remove(name),
		};
	}
	dropped.push(at);
}
