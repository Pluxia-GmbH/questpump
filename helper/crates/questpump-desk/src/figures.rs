//! The move figures' table the desk reads (docs/plan.md slice 35, "The figures' weight"; `contract/figures/figures.json`,
//! built in): each library move's owner and its film's length, one loop at its real tempo (a move's beat, `beat`, which
//! the pump's rep count runs on and Big view's figure is phased to), the second each rep but the last ends where a loop
//! shows more than one (`rep_ends`, a dead bug's two sides), and whether a move has a film the day reel can show
//! (`has_film`, where the platform holds the whole set of films: `Platform::films`). The desktop app's `desk/figures.rs`
//! publishes the table as the `figures` topic and keeps the films' place.

use std::collections::BTreeMap;
use std::sync::LazyLock;

use serde::{Deserialize, Serialize};

use crate::facts::Facts;

/// figures.json, the part the desktop app reads.
#[derive(Deserialize)]
pub struct Table {
	pub card: String,
	pub crimson: String,
	pub film: Film,
	pub films_digest: String,
	/// Each file's sha256 by its path in contract/figures/.
	pub files: BTreeMap<String, String>,
	pub moves: BTreeMap<String, Move>,
	pub owners: BTreeMap<String, Owner>,
	/// The figures that are no library move, by name: a still each, no film (`streak_banner`).
	#[serde(default)]
	pub extras: BTreeMap<String, Extra>,
}

/// An extra figure's numbers: an owner's, read off its still.
#[derive(Deserialize, Serialize)]
pub struct Extra {
	pub r#box: [f64; 4],
	pub fade: [Option<f64>; 3],
	pub tile: [f64; 3],
}

/// The films' layout: the square's side and the tint's first row.
#[derive(Deserialize, Serialize)]
pub struct Film {
	pub side: u32,
	pub tint: u32,
}

#[derive(Deserialize, Serialize)]
pub struct Move {
	pub owner: String,
	pub key: f64,
	pub mirror: bool,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub fault: Option<String>,
	/// The move drawn with the implement each equipment key names (tools/figures' gear variants, 2026-10-09), which
	/// the pump shows a player whose load that implement carries (`lib/figure.js` `figureName`).
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub gear: Option<BTreeMap<String, Move>>,
	/// Where its loop shows more than one rep (a dead bug's two sides), the second each rep but the last ends.
	#[serde(default, skip_serializing)]
	pub counts: Vec<f64>,
}

/// An owner's numbers, shares of its square's side, unmirrored, and its film's length.
#[derive(Deserialize, Serialize)]
pub struct Owner {
	pub r#box: [f64; 4],
	pub fade: [Option<f64>; 3],
	pub tile: [f64; 3],
	/// The film's length in seconds, one loop at real tempo (`beat`); none in a table written before it.
	#[serde(default, rename = "loop", skip_serializing)]
	pub film_s: Option<f64>,
}

/// The table, built in; one that does not read leaves the app without figures (logged at the start).
pub static TABLE: LazyLock<Option<Table>> =
	LazyLock::new(|| serde_json::from_str(include_str!("../../../../contract/figures/figures.json")).ok());

/// A move's beat in seconds by its key: its owner's film's length, one loop at its real tempo; none for a move without
/// a figure, an owner whose length the table does not name, or a table that does not read.
pub fn beat(key: &str) -> Option<f64> {
	let table = TABLE.as_ref()?;
	table.owners.get(&table.moves.get(key)?.owner)?.film_s
}

/// The seconds in a move's loop at which each of its reps but the last ends (a dead bug's two sides), by its key; none
/// for a loop of one rep, a move without a figure or a table that does not read.
pub fn rep_ends(key: &str) -> Vec<f64> {
	TABLE.as_ref().and_then(|t| t.moves.get(key)).map(|m| m.counts.clone()).unwrap_or_default()
}

/// Whether a library move has a film the day reel can show, by its id (its key in `contract/moves.json`): its owner's
/// film with a length and the table's crimson, wherever the films are (`Platform::films` says whether they are here).
#[must_use]
pub fn has_film(f: &Facts, mv: u16) -> bool {
	let Some(table) = TABLE.as_ref() else { return false };
	let owner = f.text(mv).and_then(|t| table.moves.get(&t.key)).and_then(|entry| table.owners.get(&entry.owner));
	owner.is_some_and(|o| o.film_s.is_some()) && hex(&table.crimson).is_some()
}

/// "#RRGGBB" as its three bytes.
#[must_use]
pub fn hex(colour: &str) -> Option<[u8; 3]> {
	let digits = colour.strip_prefix('#').filter(|d| d.len() == 6)?;
	let byte = |i: usize| u8::from_str_radix(digits.get(i..i + 2)?, 16).ok();
	Some([byte(0)?, byte(2)?, byte(4)?])
}
