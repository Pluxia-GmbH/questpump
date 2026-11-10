//! The sitting clock in the loop (docs/computer-games.md §3; docs/plan.md slices 48 and 57; questpump-core
//! `sitting.rs`, which holds its rules): every 2-s look reads this computer's input idle time (Quartz's HID state on
//! macOS, `GetLastInputInfo` on Windows, neither behind a permission) and whether a watched game counts as input (its
//! window in front, or a match running: slice 48's double-check, so a game left open behind other windows lets a pause
//! count; only while the system's idle time is under `PLAYING_CAP_MS`'s 3 hours, so a game left in front overnight
//! ends as a pause), whether the sitting cue is on or not, so the one clock serves the sitting cue
//! (`desk/reminders.rs`) and the match offers (`runtime/matches.rs`: since slice 57 whether a pump after a match opens
//! on its warm move, never whether it comes or its size; the end of the input's latest pause also bounds a match's
//! minutes, and the raw idle time tells a card's offer when the player left the desk) alike. Its start is the latest
//! of the end of the input's latest pause, the newest performed move on any device among the rows the helper holds
//! and, for the sitting cue alone (`Helper::sat_s`: an offer is a stand-up ask itself), the latest game offer on any
//! device (`settings.offers.last_game_offer_at`); a match's warm move reads it without the offers (`Desk::seated_s`),
//! so an offer let pass or put off stands nobody up. The game day's 04:00 bounds it only where the idle time never
//! read. A quit keeps the pause's end and the last look a watched game counted at in settings.json
//! (`settings.sitting`), and a start within `sitting::away_s` of that quit (an update, Quit and reopen) takes them on.
//! While work mode is on (slice 67) a call, presenting, mirroring and full screen count as input too, as a game in
//! front does, so a long call is no time away (`runtime/work.rs`, which reads them then alone).

use questpump_core::session::Zone;
use questpump_core::sitting::Sitting;

use super::{Helper, local_offset_at};
use crate::games::procs::{self, Snapshot};
use crate::settings::KeptClock;

/// Whole seconds since this computer's last keyboard, mouse or tablet input (Quartz's HID state, which needs no Input
/// Monitoring grant: it reads a time, never an event); none where the system gives no number.
#[cfg(target_os = "macos")]
fn input_idle_s() -> Option<u32> {
	use objc2_core_graphics::{CGEventSource, CGEventSourceStateID, CGEventType};
	// kCGAnyInputEventType, `(CGEventType)(~0)` in CGEventTypes.h, which the bindings leave out
	let any = CGEventType(!0);
	let idle = CGEventSource::seconds_since_last_event_type(CGEventSourceStateID::HIDSystemState, any);
	#[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "a finite time clamped into a u32")]
	idle.is_finite().then(|| idle.clamp(0.0, f64::from(u32::MAX)) as u32)
}

/// Whole seconds since this session's last input (`GetLastInputInfo`'s milliseconds); a last input the tick count has
/// not reached yet (Microsoft: the value "is not guaranteed to be incremental") is now.
#[cfg(windows)]
fn input_idle_s() -> Option<u32> {
	use ::windows::Win32::System::SystemInformation::GetTickCount;
	use ::windows::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};
	let mut info = LASTINPUTINFO { cbSize: u32::try_from(size_of::<LASTINPUTINFO>()).ok()?, dwTime: 0 };
	// SAFETY: `info` is a LASTINPUTINFO with its cbSize set, which GetLastInputInfo only writes; GetTickCount takes
	// nothing.
	let (read, now) = unsafe { (GetLastInputInfo(&raw mut info).as_bool(), GetTickCount()) };
	if !read {
		return None;
	}
	let idle = now.wrapping_sub(info.dwTime);
	Some(if idle > u32::MAX / 2 { 0 } else { idle / 1_000 })
}

/// The clock a quit kept (`Helper::sitting_keep`), at a start at unix second `now`.
pub(super) fn restored(kept: Option<KeptClock>, now: i64) -> Sitting {
	kept.map_or_else(Sitting::default, |k| Sitting::restored(k.since, k.game_seen, k.quit_at, now))
}

impl Helper {
	/// One look at this computer's input (the loop's look, before the reminders' and the games'): a watched game that
	/// ran at the games' last look keeps the idle time fresh while a match runs (`Watcher::match_runs`) or its window is
	/// in front in this look's snapshot, and so do work mode's call, presenting and full screen while it is on
	/// (`work_look_here`, which this look runs first), as long as the system's idle time is under `PLAYING_CAP_MS`
	/// (`Sitting::look`).
	pub(super) fn sitting_look(&mut self, now: i64, snap: &Snapshot) {
		let game = self.desk.p.games.as_ref().is_some_and(|g| {
			g.watcher.match_runs()
				|| (!g.running().is_empty()
					&& procs::foreground_pid()
						.is_some_and(|front| g.running().iter().any(|id| g.pids(id, snap).contains(&front))))
		});
		let present = self.work_look_here(game);
		self.desk.p.sitting.look(now, input_idle_s(), game || present);
	}

	/// The seconds the sitting clock holds at unix second `now`, the sitting cue's: the latest game offer on any device
	/// starts it again too.
	pub(crate) fn sat_s(&self, now: i64) -> i64 {
		self.sat_since(now, self.desk.settings.offers.last_game_offer_at)
	}

	/// The sitting clock at `now`, the latest game offer `last_offer` counted in.
	fn sat_since(&self, now: i64, last_offer: Option<i64>) -> i64 {
		let day = Zone::new(local_offset_at).game_day_start(now);
		self.desk.p.sitting.sat_s(now, self.desk.last_move_at(), last_offer, day)
	}

	/// The clock kept for the next start, as this process goes at unix second `now` (`wind_down`).
	pub(super) fn sitting_keep(&mut self, now: i64) {
		let game_seen = self.desk.p.sitting.game_seen();
		self.desk.settings.sitting =
			self.desk.p.sitting.kept().map(|since| KeptClock { since, game_seen, quit_at: now });
		self.desk.save_settings();
	}
}
