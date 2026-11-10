//! Work mode on this computer at the loop's look (docs/plan.md slice 67; the desk's `work.rs` holds its rules and its
//! clock): while it is on, each 2-s look reads what the computer shows of the player (`presence::look`, nothing while
//! it is off): the app in front other than QuestPump, kept as the latest such app (QuestPump's own window in front
//! keeps the one before), which a work pump's Back to work brings back (`Platform::back_note`); and what holds the
//! work clock here beside the desk's own holds (a pump, the felt-unwell lock, a match's offer or the game's quest pump
//! open, quiet hours), in this order: the microphone or the camera in use (a call), a watched game in front or in a
//! match or World of Warcraft's window in front (a client left open behind the work app holds nothing: slice 67),
//! presenting or mirroring, a window in full screen in front. A game day it is on counts `work` among
//! `fitness_feature_day`'s features (`counts_work_on`).

use questpump_core::work::Hold;

use super::Helper;
use crate::games::procs;
use crate::presence;

impl Helper {
	/// The look's part of work mode on this computer, before the sitting clock's (`sitting_look`, which hands it
	/// whether a watched game counts as input, `game`): the front app and the hold kept for the desk's look
	/// (`Shell::front`, `Shell::work_hold`); whether a call, presenting or full screen counts as input to the sitting
	/// clock. Off, both are cleared and nothing is read; on, the game day counts `work`.
	pub(super) fn work_look_here(&mut self, game: bool) -> bool {
		if !self.desk.work_on() {
			(self.desk.p.work_hold, self.desk.p.front) = (None, None);
			return false;
		}
		self.counts_work_on();
		let shell = &mut self.desk.p;
		let seen = presence::look();
		if seen.front.is_some() {
			shell.front = seen.front;
		}
		let wow_front = shell.game.as_ref().is_some_and(|g| procs::foreground_pid() == Some(g.pid));
		shell.work_hold = if seen.call {
			Some(Hold::Call)
		} else if game || wow_front {
			Some(Hold::Game)
		} else {
			seen.shown
		};
		seen.call || seen.shown.is_some()
	}
}
