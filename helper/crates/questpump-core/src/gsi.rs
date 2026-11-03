//! Valve's Game State Integration for Counter-Strike 2 and Dota 2 (docs/plan.md slice 35; Valve's developer wiki,
//! "Counter-Strike: Global Offensive Game State Integration", read 2026-10-08): the cfg file the helper writes into the
//! game's folder, which the game reads at its start, and the one thing it reads of each post the game sends to
//! 127.0.0.1. The cfg asks for the `map` component alone; whatever a post carries besides `auth.token` and the phase
//! (`map.phase` in CS2, `map.game_state` in Dota 2) — scores, a match id, the winner, a `provider` block with a Steam
//! ID should one come unasked — is never read: the body is parsed into a type with those fields only and dropped.

use serde::Deserialize;
use subtle::ConstantTimeEq as _;

use crate::games::Watched;

/// The cfg file's name: the game reads every `gamestate_integration_*.cfg` in its folder at its start.
pub const FILE: &str = "gamestate_integration_questpump.cfg";
/// The cfg's `heartbeat`, seconds: the longest a game in its menus or a quiet match goes without a post.
pub const HEARTBEAT_S: i64 = 60;

/// A game that posts its state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Gsi {
	CounterStrike,
	Dota,
}

impl Gsi {
	pub const ALL: [Self; 2] = [Self::CounterStrike, Self::Dota];

	/// The path its cfg's `uri` names on the helper's listener, so a post says which game sent it without a field of
	/// the body.
	#[must_use]
	pub fn path(self) -> &'static str {
		match self {
			Self::CounterStrike => "/cs2",
			Self::Dota => "/dota",
		}
	}

	#[must_use]
	pub fn watched(self) -> Watched {
		match self {
			Self::CounterStrike => Watched::CounterStrike,
			Self::Dota => Watched::Dota,
		}
	}
}

/// The cfg file's text: posts to the helper's listener on the IPv4 loopback (never `localhost`, which may resolve to
/// `::1`, where nothing listens) at this game's path, with the token, the `map` component alone, no BOM (Valve: "or else
/// the configuration-file won't load"). `timeout` 5 s lets a busy loop answer; `heartbeat` 60 s keeps a quiet match
/// heard.
#[must_use]
pub fn cfg(game: Gsi, port: u16, token: &str) -> String {
	let lines = [
		"\"QuestPump\"".to_owned(),
		"{".to_owned(),
		format!(" \"uri\" \"http://127.0.0.1:{port}{}\"", game.path()),
		" \"timeout\" \"5.0\"".to_owned(),
		" \"buffer\" \"0.5\"".to_owned(),
		" \"throttle\" \"1.0\"".to_owned(),
		format!(" \"heartbeat\" \"{HEARTBEAT_S}.0\""),
		" \"auth\"".to_owned(),
		" {".to_owned(),
		format!("  \"token\" \"{token}\""),
		" }".to_owned(),
		" \"data\"".to_owned(),
		" {".to_owned(),
		"  \"map\" \"1\"".to_owned(),
		" }".to_owned(),
		"}".to_owned(),
	];
	lines.join("\n") + "\n"
}

/// The only fields read of a post; serde skips every other.
#[derive(Deserialize)]
struct Body {
	auth: Option<Auth>,
	map: Option<Map>,
}

#[derive(Deserialize)]
struct Auth {
	token: Option<String>,
}

#[derive(Deserialize)]
struct Map {
	/// CS2: `warmup`, `live`, `intermission`, `gameover`.
	phase: Option<String>,
	/// Dota 2: `DOTA_GAMERULES_STATE_*`.
	game_state: Option<String>,
}

/// Why a post was refused.
#[derive(Debug, thiserror::Error)]
pub enum Refused {
	#[error("no token or another token")]
	Token,
	#[error("the body does not read: {0}")]
	Body(String),
}

/// A post's phase: CS2's `map.phase` or Dota 2's `map.game_state`; none while no map is loaded (the menus). The token
/// is compared in constant time.
///
/// # Errors
/// A body that is no JSON object of the expected shape, or one without the cfg's token.
pub fn read(game: Gsi, body: &[u8], token: &str) -> Result<Option<String>, Refused> {
	let body: Body = serde_json::from_slice(body).map_err(|e| Refused::Body(e.to_string()))?;
	let given = body.auth.and_then(|a| a.token).ok_or(Refused::Token)?;
	if !bool::from(given.as_bytes().ct_eq(token.as_bytes())) {
		return Err(Refused::Token);
	}
	Ok(body.map.and_then(|m| match game {
		Gsi::CounterStrike => m.phase,
		Gsi::Dota => m.game_state,
	}))
}
