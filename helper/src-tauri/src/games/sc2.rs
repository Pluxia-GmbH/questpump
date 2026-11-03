//! StarCraft II's client API on 127.0.0.1 (docs/plan.md slice 48; docs/computer-games.md §6.2): while the game runs and
//! is watched (slice 56), a GET of `/ui` every `EVERY`, and of `/game` while its screens say a match runs, each answer
//! read by questpump-core's `feeds::starcraft` and dropped (`/game`'s `players[]` is counted, its names and results
//! never read: a match plays only with players and no replay). Nothing is ever sent to the game but those two GETs.
//! Each change of its word is told once.

use std::time::Duration;

use questpump_core::feeds::starcraft::{self, PORT, Screen};

use super::Word;
use super::task::{self, Task, Teller};

/// How often the client is asked.
const EVERY: Duration = Duration::from_secs(2);

/// Asks the client until the handle is dropped.
pub(super) fn start(teller: Teller, client: reqwest::Client) -> Task {
	Task::spawn(move |mut stop| async move {
		let (mut up, mut said) = (false, None);
		loop {
			if let Some(screen) = task::get(&client, PORT, "/ui").await.and_then(|b| starcraft::screen(&b)) {
				if !std::mem::replace(&mut up, true) {
					teller.tell(Word::Up);
				}
				let plays = if screen == Screen::Match {
					task::get(&client, PORT, "/game").await.and_then(|b| starcraft::plays(&b))
				} else {
					None
				};
				if let Some(word) = starcraft::word(screen, plays).filter(|w| said != Some(*w)) {
					said = Some(word);
					teller.tell(Word::Match(word));
				}
			}
			if !task::wait(EVERY, &mut stop).await {
				return;
			}
		}
	})
}
