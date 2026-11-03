//! Why QuestPump started (docs/plan.md slice 35, decision 2): a player's launch from the Dock, Applications or the Start
//! menu, and a second launch while it runs, open the main window; a start at login, after an update, after the system
//! quit it or after a crash of a supervised part (`--login`, `--updated`, `--reopened`, `--restarted` and
//! `--restarted-quick`, `crate::keep`) never does, nor does one whose launch event says it is a login item.
//! Windows' Run value carries `--login` (`autostart.rs`) and the updater's relaunch `--updated` on both systems
//! (`update.rs`). macOS starts its login item (`SMAppService.mainApp`) with no arguments: a start is a login's when the
//! launch Apple event says so (`keyAELaunchedAsLogInItem`) or, where it does not, when the login item is on and the
//! session's login was less than a minute ago. The restart that applies a Screen Recording grant opens the window on
//! This computer (`--window`). While a Mac's QuestPump runs from its download (`update::unmoved`), opening it again (a
//! second launch, the Dock) opens This computer, whose card asks for the move to Applications, and a launch of the
//! copy the player moved there takes over: this one quits through the loop ("Leave the pump?" first while a pump holds
//! the window) and that one starts once it has gone, only on the path that quits (the player's Stay keeps this one
//! and starts nothing).

use std::ffi::OsString;

use tauri::AppHandle;

use super::windows::{self, Landing};

/// The Run value's argument: a start at login.
pub(crate) const LOGIN: &str = "--login";
/// The updater's relaunch.
pub(crate) const UPDATED: &str = "--updated";
/// The start after the system quit QuestPump while the Screen Recording grant was missing (`update::reopen_after_quit`):
/// it goes on only when the grant now holds (`lib.rs`).
pub(crate) const REOPENED: &str = "--reopened";
/// The restart from This computer's grant card: the window opens on This computer.
pub(crate) const WINDOW: &str = "--window";
/// The restart after a supervised part ended in a panic (`crate::keep`): no window; `RESTARTED_QUICK` when it ended
/// within a minute of its start, so a second quick end stays up instead of restarting again.
pub(crate) const RESTARTED: &str = "--restarted";
pub(crate) const RESTARTED_QUICK: &str = "--restarted-quick";

/// Why this process started.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Launch {
	/// The player opened it: the window opens.
	Player,
	/// The grant card's restart: the window opens on This computer.
	Restart,
	/// At login, after an update or after the system quit it: no window.
	Background,
}

/// The reason the arguments give.
fn of_args(args: &[OsString]) -> Option<Launch> {
	if args.iter().any(|a| a == LOGIN || a == UPDATED || a == REOPENED || a == RESTARTED || a == RESTARTED_QUICK) {
		Some(Launch::Background)
	} else if args.iter().any(|a| a == WINDOW) {
		Some(Launch::Restart)
	} else {
		None
	}
}

/// This process's reason, read once in the app's setup (on macOS while the launch event is the current Apple event).
pub(crate) fn at_start() -> Launch {
	let args: Vec<OsString> = std::env::args_os().skip(1).collect();
	if let Some(launch) = of_args(&args) {
		return launch;
	}
	#[cfg(target_os = "macos")]
	if let Some(how) = mac::login_start() {
		log::info!("launch: at login ({how:?}): no window");
		return Launch::Background;
	}
	Launch::Player
}

/// A second launch while QuestPump runs (the single-instance plugin hands over its arguments, its program first): the
/// window opens (`reopen`), unless the second start was a background one; on a Mac running from its download, a launch
/// of the copy moved into Applications takes over (`update::moved_copy`).
pub(crate) fn second(app: &AppHandle, args: &[String]) {
	let rest: Vec<OsString> = args.iter().skip(1).map(OsString::from).collect();
	match of_args(&rest) {
		Some(Launch::Background) => {}
		Some(Launch::Restart) => windows::open_main(app, Landing::Computer),
		Some(Launch::Player) | None => {
			#[cfg(target_os = "macos")]
			if let Some(moved) =
				args.first().and_then(|program| crate::update::moved_copy(std::path::Path::new(program)))
			{
				log::info!("launch: {} opened, so this copy quits for it", moved.display());
				return crate::ui::quit_for(app, Some(moved));
			}
			reopen(app);
		}
	}
}

/// QuestPump opened again while it runs (a second launch, the Dock, Finder): its window, on This computer while it runs
/// from its download, whose card asks for the move to Applications.
pub(crate) fn reopen(app: &AppHandle) {
	let landing = if crate::update::unmoved() { Landing::Computer } else { Landing::Here };
	windows::open_main(app, landing);
}

#[cfg(target_os = "macos")]
mod mac {
	use objc2::msg_send;
	use objc2::rc::Retained;
	use objc2_foundation::{NSAppleEventDescriptor, NSAppleEventManager};

	/// Apple's four-character codes (Core Services' `AERegistry.h`).
	const CORE_EVENT_CLASS: u32 = u32::from_be_bytes(*b"aevt");
	const OPEN_APPLICATION: u32 = u32::from_be_bytes(*b"oapp");
	const PROP_DATA: u32 = u32::from_be_bytes(*b"prdt");
	const LAUNCHED_AS_LOGIN_ITEM: u32 = u32::from_be_bytes(*b"lgit");
	/// How long after the session's login a start of the login item counts as the login's.
	const LOGIN_WINDOW_S: i64 = 60;

	/// How a start shows it is a login's.
	#[derive(Debug)]
	pub(super) enum LoginStart {
		/// The launch Apple event says so.
		LaunchEvent,
		/// The login item is on and the session logged in less than `LOGIN_WINDOW_S` ago.
		JustAfterLogin,
	}

	/// How this start shows it is a login's, if it is one.
	pub(super) fn login_start() -> Option<LoginStart> {
		if launched_as_login_item() {
			return Some(LoginStart::LaunchEvent);
		}
		let age = session_age()?;
		(crate::autostart::state() == Some(true) && age < LOGIN_WINDOW_S).then_some(LoginStart::JustAfterLogin)
	}

	/// The launch Apple event says the app was opened as a login item.
	fn launched_as_login_item() -> bool {
		objc2::rc::autoreleasepool(|_| {
			let Some(event) = NSAppleEventManager::sharedAppleEventManager().currentAppleEvent() else { return false };
			// SAFETY: eventClass and eventID are NSAppleEventDescriptor's getters, each returning a FourCharCode.
			let (class, id): (u32, u32) = unsafe { (msg_send![&*event, eventClass], msg_send![&*event, eventID]) };
			if class != CORE_EVENT_CLASS || id != OPEN_APPLICATION {
				return false;
			}
			// SAFETY: paramDescriptorForKeyword: takes a FourCharCode and returns a descriptor or nil.
			let data: Option<Retained<NSAppleEventDescriptor>> =
				unsafe { msg_send![&*event, paramDescriptorForKeyword: PROP_DATA] };
			data.is_some_and(|d| d.enumCodeValue() == LAUNCHED_AS_LOGIN_ITEM)
		})
	}

	/// Seconds since this user's console session logged in (utmpx, which `who` reads), none when no record says.
	fn session_age() -> Option<i64> {
		let user = std::env::var("USER").ok()?;
		let mut login: Option<i64> = None;
		// SAFETY: setutxent, getutxent and endutxent walk the system's utmpx records on this thread; each record read is
		// copied out before the next call, and a null ends the walk.
		unsafe {
			libc::setutxent();
			loop {
				let entry = libc::getutxent();
				if entry.is_null() {
					break;
				}
				let entry = &*entry;
				let text = |field: &[libc::c_char]| {
					let bytes: Vec<u8> = field.iter().take_while(|&&c| c != 0).map(|&c| c.to_ne_bytes()[0]).collect();
					String::from_utf8_lossy(&bytes).into_owned()
				};
				if entry.ut_type == libc::USER_PROCESS
					&& text(&entry.ut_line) == "console"
					&& text(&entry.ut_user) == user
				{
					login = Some(login.map_or(entry.ut_tv.tv_sec, |l| l.max(entry.ut_tv.tv_sec)));
				}
			}
			libc::endutxent();
		}
		let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).ok()?.as_secs();
		Some(i64::try_from(now).ok()? - login?)
	}
}
