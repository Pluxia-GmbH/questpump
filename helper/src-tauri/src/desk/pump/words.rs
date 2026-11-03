//! What only the pump's screens read of the contract's files (docs/plan.md slice 35: Rust owns the state, and the page
//! formats only): the plain kinds of an app pump with their families (`contract/kinds.json`), a few rules of the
//! library the pump alone asks (a counted form's top, a carry) and the game's `%d` and `%s` filled in. The game's words
//! (a pose's goal pose among them), the moves' words, the library's facts and the tunables are the built-in facts'
//! (`desk/facts.rs`), which every area reads. The files are the addon's at this build's commit (`make gen-check`); a
//! file that does not read leaves its part empty, and the screens show what is left.

use std::sync::LazyLock;

use serde::Deserialize;

use crate::desk::facts::facts;

/// A plain kind of an app pump and the family its pump is composed for.
#[derive(Deserialize, Default)]
#[serde(default)]
pub(super) struct Kind {
	pub(super) key: String,
	pub(super) family: String,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct Kinds {
	kinds: Vec<Kind>,
}

static KINDS: LazyLock<Kinds> = LazyLock::new(|| {
	serde_json::from_str(include_str!("../../../../../contract/kinds.json")).unwrap_or_else(|e| {
		log::error!("pump: the built-in kinds.json does not read: {e}");
		Kinds::default()
	})
});

/// The plain kinds, in the order every screen lists them.
pub(super) fn kinds() -> &'static [Kind] {
	&KINDS.kinds
}

/// The family a plain kind's pump is composed for; none for a key the file lacks.
pub(super) fn family(kind: &str) -> Option<&'static str> {
	KINDS.kinds.iter().find(|k| k.key == kind).map(|k| k.family.as_str())
}

/// A move is a carry (`pattern` "carry"): its line names the load.
pub(super) fn carry(move_id: u16) -> bool {
	facts().library.moves.get(&move_id).is_some_and(|m| m.pattern == "carry")
}

/// A word with its numbers and names in place of the addon's `%d` and `%s`, each taking the next value in order (as
/// Lua's `string.format`); one past the last value stays as written (`SheetVocab.Words.fill`).
pub(super) fn fill(text: &str, values: &[&str]) -> String {
	let mut filled = String::with_capacity(text.len());
	let mut rest = text;
	for value in values {
		let at = [rest.find("%d"), rest.find("%s")].into_iter().flatten().min();
		let Some(at) = at else { break };
		filled.push_str(&rest[..at]);
		filled.push_str(value);
		rest = &rest[at + 2..];
	}
	filled.push_str(rest);
	filled
}
