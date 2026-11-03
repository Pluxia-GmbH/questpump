//! What the helper knows of the addon's moves: `contract/moves.json`, which `tools/gen-fixtures.lua` writes from
//! `Library.lua` (`make fixtures`), built into the helper. A move a newer addon adds is unknown here and scored from its
//! go, as most moves are.

use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

use serde::Deserialize;

#[derive(Deserialize, Default)]
struct Moves {
	/// The moves the verdict scores on the 30 s after the set when the heart rate comes from a wrist (the library's
	/// `hrAfter`: push-ups and burpees, whose hands bear the body while wrist optical heart rate drops out).
	after_set: HashSet<u16>,
	/// The moves merged into another on 2026-10-04 (Library.lua `ns.Merged`): an old id, never reused, to the move that
	/// took it.
	#[serde(default)]
	merged: HashMap<u16, u16>,
}

/// The file is built in; one that does not read leaves the set empty, and every move is then scored from its go.
static MOVES: LazyLock<Moves> =
	LazyLock::new(|| serde_json::from_str(include_str!("../../../../contract/moves.json")).unwrap_or_default());

/// The move an id stands for now: a move merged into another is that move (its sets, its record and its unfinished
/// moves are the merged-into move's, as the addon's fold reads them), any other id itself.
#[must_use]
pub fn current(move_id: u16) -> u16 {
	MOVES.merged.get(&move_id).copied().unwrap_or(move_id)
}

/// Whether a wrist's heart rate scores this move on the 30 s after its set.
#[must_use]
pub fn after_set(move_id: u16) -> bool {
	MOVES.after_set.contains(&move_id)
}
