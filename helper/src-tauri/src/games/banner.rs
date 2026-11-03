//! ELDEN RING's and ELDEN RING NIGHTREIGN's boss banners in the games' loop (2026-10-29; docs/computer-games.md §6.2):
//! while the game runs and is heard, the capture's banner reader (`capture::banner`, Windows) on its game process's
//! window, whose words come on the games' channel as a task's do (`Teller`): its session started (`Word::Up`, the game
//! heard) and each boss felled (`Fed::Completed`, a checkpoint: questpump-core's `feeds::banner`). No other system has
//! one. Dropping the reader ends its thread, which stops the capture there.

use super::task::Teller;

/// The banners' reader on one game process.
pub(super) struct Banner {
	/// The game process whose window it reads: another process of the game starts a reader of its own.
	pub(super) pid: u32,
	#[cfg(windows)]
	_watch: crate::capture::banner::Watch,
}

impl Banner {
	/// A reader on the game process `pid`'s window, telling `teller`; none where no thread could start, and none
	/// outside Windows.
	pub(super) fn start(pid: u32, teller: Teller, key: &'static str) -> Option<Self> {
		#[cfg(windows)]
		{
			use questpump_core::games::Fed;

			use super::Word;
			use crate::capture::banner::{Told, Watch};
			let tell = move |told: Told| match told {
				Told::Up => teller.tell(Word::Up),
				Told::Fell => {
					log::info!("games: {key}: a boss fell");
					teller.tell(Word::Match(Fed::Completed));
				}
			};
			Some(Self { pid, _watch: Watch::start(pid, std::sync::Arc::new(tell))? })
		}
		#[cfg(not(windows))]
		{
			let _ = (pid, teller, key);
			None
		}
	}
}
