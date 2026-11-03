//! Legends of Runeterra's Game Client API on 127.0.0.1 (docs/plan.md slice 48; docs/computer-games.md §6.2; Riot's
//! docs/lor): while the game runs and the player has not turned it off, a GET of `/game-result` every `EVERY` at the
//! port its settings name (21337 unless the player set another in the game's window), read by questpump-core's
//! `feeds::runeterra` for its `GameID` alone, which rises once a game completed; `LocalPlayerWon` and every other
//! endpoint are never read.

use std::time::Duration;

use questpump_core::feeds::runeterra::{self, Games};

use super::Word;
use super::task::{self, Task, Teller};

/// How often the client is asked.
const EVERY: Duration = Duration::from_secs(2);

/// Asks the client on `port` until the handle is dropped.
pub(super) fn start(teller: Teller, client: reqwest::Client, port: u16) -> Task {
	Task::spawn(move |mut stop| async move {
		let (mut up, mut games) = (false, Games::default());
		loop {
			if let Some(id) = task::get(&client, port, "/game-result").await.and_then(|b| runeterra::game_id(&b)) {
				if !std::mem::replace(&mut up, true) {
					teller.tell(Word::Up);
				}
				if let Some(word) = games.push(id) {
					teller.tell(Word::Match(word));
				}
			}
			if !task::wait(EVERY, &mut stop).await {
				return;
			}
		}
	})
}
