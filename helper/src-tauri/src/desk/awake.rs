//! The computer kept awake while a pump is on screen (docs/plan.md slice 35, "The UI stack"): on macOS an
//! `NSProcessInfo` activity (user initiated, latency critical, the display's idle sleep held off), so App Nap never
//! slows the pump's clock or its voice and the screen stays lit through a long hold; on Windows
//! `SetThreadExecutionState` with the display required. Both are held and released on the main thread, which lives as
//! long as QuestPump: Windows ties the state to the thread that set it, and the macOS token stays there.

use tauri::AppHandle;

/// Holds the computer awake (`on`) or lets it sleep again.
pub(crate) fn hold(app: &AppHandle, on: bool) {
	if let Err(e) = app.run_on_main_thread(move || platform::hold(on)) {
		log::warn!("awake: {e}");
	}
}

#[cfg(target_os = "macos")]
mod platform {
	use std::cell::RefCell;

	use objc2::rc::Retained;
	use objc2::runtime::{NSObjectProtocol, ProtocolObject};
	use objc2_foundation::{NSActivityOptions, NSProcessInfo, NSString};

	thread_local! {
		/// The activity's token while it is held (the main thread's own).
		static ACTIVITY: RefCell<Option<Retained<ProtocolObject<dyn NSObjectProtocol>>>> = const { RefCell::new(None) };
	}

	pub(super) fn hold(on: bool) {
		ACTIVITY.with(|held| {
			let mut held = held.borrow_mut();
			let info = NSProcessInfo::processInfo();
			if on && held.is_none() {
				let options = NSActivityOptions::UserInitiated
					| NSActivityOptions::LatencyCritical
					| NSActivityOptions::IdleDisplaySleepDisabled;
				// copy-lint: not shown (the system's activity reason)
				let reason = NSString::from_str("A QuestPump pump runs");
				*held = Some(info.beginActivityWithOptions_reason(options, &reason));
			} else if !on && let Some(activity) = held.take() {
				// SAFETY: `activity` is the token beginActivityWithOptions:reason: returned, which endActivity: takes.
				unsafe { info.endActivity(&activity) };
			}
		});
	}
}

#[cfg(windows)]
mod platform {
	use windows::Win32::System::Power::{ES_CONTINUOUS, ES_DISPLAY_REQUIRED, SetThreadExecutionState};

	pub(super) fn hold(on: bool) {
		let state = if on { ES_CONTINUOUS | ES_DISPLAY_REQUIRED } else { ES_CONTINUOUS };
		// SAFETY: SetThreadExecutionState takes a set of flags and returns the thread's previous state; no memory of
		// ours is passed.
		let _ = unsafe { SetThreadExecutionState(state) };
	}
}

#[cfg(not(any(target_os = "macos", windows)))]
mod platform {
	pub(super) fn hold(_on: bool) {}
}
