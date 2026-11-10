//! What this computer shows of the player at it, read for work mode alone and only while it is on (docs/plan.md slice
//! 67; the desk's `work.rs` holds the rules): the app in front other than QuestPump (`procs::front_other`), which Back
//! to work brings back; whether the microphone or the camera is in use (a call); whether the screen is presented or
//! mirrored; and whether a window in full screen is in front. Each is a state the system shows any app, never what a
//! device hears or sees, and none asks the system for a grant (rust-helper rule 3; `desk.menu.does`'s front, calls and
//! full-screen lines):
//!
//! - macOS: CoreAudio's process objects, each asked whether it runs input (`kAudioProcessPropertyIsRunningInput`, macOS
//!   14.2 on; asked per app, since a Bluetooth microphone in use reads as stopped at the device), the apps' own alone
//!   (a bundle id outside Apple's, or FaceTime's and WebKit's, which Safari's calls run in: Siri's and dictation's
//!   listening is no call); CoreMediaIO's devices, each asked whether any app runs it
//!   (`kCMIODevicePropertyDeviceIsRunningSomewhere`); CoreGraphics' active displays, one in a mirror set; the window
//!   server's list for the front window's bounds against its display's (`procs::Front::fills`). Whether macOS 15 to
//!   27 show no prompt for these reads is checked on the installed bundle (the plan's "To check on a device").
//! - Windows: the shell's notification state (`SHQueryUserNotificationState`: presentation mode, a full-screen or
//!   Direct3D full-screen app in front, never QuestPump's own window) and the registry's record of each app's use of
//!   the microphone and the webcam (`CapabilityAccessManager\ConsentStore`: an app whose `LastUsedTimeStop` is 0 uses
//!   it now; community knowledge, not documented by Microsoft).

use questpump_core::work::Hold;

use crate::games::procs::{self, Front};

/// One look while work mode is on: the app in front other than QuestPump, whether the microphone or the camera is in
/// use, and presenting, mirroring or a window in full screen in front (`Hold::Present`, `Hold::FullScreen`).
pub(crate) struct Seen {
	pub(crate) front: Option<Front>,
	pub(crate) call: bool,
	pub(crate) shown: Option<Hold>,
}

/// What this computer shows now.
pub(crate) fn look() -> Seen {
	let front = procs::front_other();
	Seen { front, call: platform::call(), shown: platform::shown(front.as_ref()) }
}

#[cfg(target_os = "macos")]
use mac as platform;
#[cfg(windows)]
use win as platform;

#[cfg(target_os = "macos")]
mod mac {
	use std::ffi::c_void;
	use std::ptr::NonNull;

	use objc2_core_foundation::{CFRetained, CFString};
	use objc2_core_graphics::{CGDisplayIsInMirrorSet, CGGetActiveDisplayList};
	use questpump_core::work::Hold;

	use crate::games::procs::Front;

	/// CoreAudio's and CoreMediaIO's property address (`AudioHardwareBase.h`, `CMIOHardwareObject.h`: the same three
	/// four-character codes).
	#[repr(C)]
	struct Address {
		selector: u32,
		scope: u32,
		element: u32,
	}

	/// A four-character code, as the headers write `'glob'`.
	const fn code(c: [u8; 4]) -> u32 {
		u32::from_be_bytes(c)
	}

	/// `kAudioObjectSystemObject`, `kCMIOObjectSystemObject`.
	const SYSTEM: u32 = 1;
	/// `kAudioObjectPropertyScopeGlobal`, `kCMIOObjectPropertyScopeGlobal`.
	const GLOBAL: u32 = code(*b"glob");
	/// `kAudioObjectPropertyElementMain`, `kCMIOObjectPropertyElementMain`.
	const MAIN: u32 = 0;
	/// `kAudioHardwarePropertyProcessObjectList` (macOS 14.2 on).
	const PROCESSES: u32 = code(*b"prs#");
	/// `kAudioProcessPropertyIsRunningInput`.
	const RUNS_INPUT: u32 = code(*b"piri");
	/// `kAudioProcessPropertyBundleID`: a `CFString` the caller releases.
	const BUNDLE: u32 = code(*b"pbid");
	/// `kCMIOHardwarePropertyDevices`.
	const DEVICES: u32 = code(*b"dev#");
	/// `kCMIODevicePropertyDeviceIsRunningSomewhere`.
	const RUNS_SOMEWHERE: u32 = code(*b"gone");
	/// The most displays a look asks for.
	const DISPLAYS: usize = 16;
	/// Apple's own apps a call runs in, whose input counts: FaceTime, and WebKit's processes (Safari's web calls).
	const APPLE_CALLS: [&str; 2] = ["com.apple.FaceTime", "com.apple.WebKit"];

	#[link(name = "CoreAudio", kind = "framework")]
	unsafe extern "C" {
		fn AudioObjectGetPropertyDataSize(
			object: u32,
			address: *const Address,
			qualifier_size: u32,
			qualifier: *const c_void,
			size: *mut u32,
		) -> i32;
		fn AudioObjectGetPropertyData(
			object: u32,
			address: *const Address,
			qualifier_size: u32,
			qualifier: *const c_void,
			size: *mut u32,
			data: *mut c_void,
		) -> i32;
	}

	#[link(name = "CoreMediaIO", kind = "framework")]
	unsafe extern "C" {
		fn CMIOObjectGetPropertyDataSize(
			object: u32,
			address: *const Address,
			qualifier_size: u32,
			qualifier: *const c_void,
			size: *mut u32,
		) -> i32;
		fn CMIOObjectGetPropertyData(
			object: u32,
			address: *const Address,
			qualifier_size: u32,
			qualifier: *const c_void,
			size: u32,
			used: *mut u32,
			data: *mut c_void,
		) -> i32;
	}

	const fn global(selector: u32) -> Address {
		Address { selector, scope: GLOBAL, element: MAIN }
	}

	/// The object ids a property of the audio system lists (none where it reads none).
	fn audio_ids(selector: u32) -> Vec<u32> {
		let address = global(selector);
		let mut size = 0u32;
		// SAFETY: a size query of the system object with no qualifier; `size` lives for the call.
		if unsafe { AudioObjectGetPropertyDataSize(SYSTEM, &raw const address, 0, std::ptr::null(), &raw mut size) }
			!= 0
		{
			return Vec::new();
		}
		let mut ids = vec![0u32; usize::try_from(size).unwrap_or(0) / size_of::<u32>()];
		let mut size = u32::try_from(ids.len() * size_of::<u32>()).unwrap_or(0);
		// SAFETY: `ids` holds `size` bytes of u32s and lives for the call, which writes at most that many and sets
		// `size` to what it wrote.
		let read = unsafe {
			AudioObjectGetPropertyData(
				SYSTEM,
				&raw const address,
				0,
				std::ptr::null(),
				&raw mut size,
				ids.as_mut_ptr().cast(),
			)
		};
		ids.truncate(if read == 0 { usize::try_from(size).unwrap_or(0) / size_of::<u32>() } else { 0 });
		ids
	}

	/// A u32 property of an audio object; none where it reads none.
	fn audio_u32(object: u32, selector: u32) -> Option<u32> {
		let address = global(selector);
		let (mut value, mut size) = (0u32, u32::try_from(size_of::<u32>()).ok()?);
		// SAFETY: `value` holds the 4 bytes `size` names and lives for the call, which writes at most that many.
		let read = unsafe {
			AudioObjectGetPropertyData(
				object,
				&raw const address,
				0,
				std::ptr::null(),
				&raw mut size,
				(&raw mut value).cast(),
			)
		};
		(read == 0).then_some(value)
	}

	/// An audio process object's bundle id; none where it has none.
	fn bundle(process: u32) -> Option<String> {
		let address = global(BUNDLE);
		let mut text: *const CFString = std::ptr::null();
		let mut size = u32::try_from(size_of::<*const CFString>()).ok()?;
		// SAFETY: `text` holds one CFStringRef, the size `size` names, and lives for the call, which writes at most that.
		let read = unsafe {
			AudioObjectGetPropertyData(
				process,
				&raw const address,
				0,
				std::ptr::null(),
				&raw mut size,
				(&raw mut text).cast(),
			)
		};
		let text = NonNull::new(text.cast_mut()).filter(|_| read == 0)?;
		// SAFETY: the property hands the caller a CFString it owns (AudioHardware.h: "The caller is responsible for
		// releasing the returned CFObject"); `CFRetained` releases it once on drop.
		let text = unsafe { CFRetained::from_raw(text) };
		Some(text.to_string()).filter(|id| !id.is_empty())
	}

	/// An app runs input: a process object of the audio system that runs input and is an app's own (`APPLE_CALLS`).
	fn microphone() -> bool {
		audio_ids(PROCESSES).into_iter().any(|process| {
			audio_u32(process, RUNS_INPUT).is_some_and(|on| on != 0)
				&& bundle(process).is_some_and(|id| {
					!id.starts_with("com.apple.") || APPLE_CALLS.iter().any(|call| id.starts_with(call))
				})
		})
	}

	/// A camera runs for any app: a CoreMediaIO device running somewhere.
	fn camera() -> bool {
		let devices = global(DEVICES);
		let mut size = 0u32;
		// SAFETY: a size query of the system object with no qualifier; `size` lives for the call.
		if unsafe { CMIOObjectGetPropertyDataSize(SYSTEM, &raw const devices, 0, std::ptr::null(), &raw mut size) } != 0
		{
			return false;
		}
		let mut ids = vec![0u32; usize::try_from(size).unwrap_or(0) / size_of::<u32>()];
		let (size, mut used) = (u32::try_from(ids.len() * size_of::<u32>()).unwrap_or(0), 0u32);
		// SAFETY: `ids` holds `size` bytes of u32s and lives for the call, which writes at most that many and sets `used`.
		let read = unsafe {
			CMIOObjectGetPropertyData(
				SYSTEM,
				&raw const devices,
				0,
				std::ptr::null(),
				size,
				&raw mut used,
				ids.as_mut_ptr().cast(),
			)
		};
		if read != 0 {
			return false;
		}
		ids.truncate(usize::try_from(used).unwrap_or(0) / size_of::<u32>());
		let runs = global(RUNS_SOMEWHERE);
		ids.into_iter().any(|device| {
			let (mut on, mut used) = (0u32, 0u32);
			// SAFETY: `on` holds the 4 bytes passed and lives for the call, which writes at most that many.
			let read = unsafe {
				CMIOObjectGetPropertyData(
					device,
					&raw const runs,
					0,
					std::ptr::null(),
					4,
					&raw mut used,
					(&raw mut on).cast(),
				)
			};
			read == 0 && on != 0
		})
	}

	/// The microphone or the camera in use.
	pub(super) fn call() -> bool {
		microphone() || camera()
	}

	/// A display in a mirror set (presenting on a projector, a screen mirrored to a TV), else the window in front
	/// filling its screen.
	pub(super) fn shown(front: Option<&Front>) -> Option<Hold> {
		let mut displays = [0u32; DISPLAYS];
		let mut count = 0u32;
		// SAFETY: `displays` has room for the `DISPLAYS` ids asked and `count` receives how many were written; both live
		// for the call.
		let read = unsafe {
			CGGetActiveDisplayList(u32::try_from(DISPLAYS).unwrap_or(0), displays.as_mut_ptr(), &raw mut count)
		};
		let listed = displays.get(..usize::try_from(count).unwrap_or(0)).unwrap_or_default();
		if read.0 == 0 && listed.iter().any(|&display| CGDisplayIsInMirrorSet(display)) {
			return Some(Hold::Present);
		}
		front.is_some_and(|f| f.fills).then_some(Hold::FullScreen)
	}
}

#[cfg(windows)]
mod win {
	use questpump_core::work::Hold;
	use windows::Win32::System::Registry::HKEY_CURRENT_USER;
	use windows::Win32::UI::Shell::{
		QUNS_BUSY, QUNS_PRESENTATION_MODE, QUNS_RUNNING_D3D_FULL_SCREEN, SHQueryUserNotificationState,
	};

	use crate::games::procs::{self, Front};
	use crate::games::win::{qword, subkeys};

	/// Where Windows records each app's use of a device, by the device's name (`microphone`, `webcam`): a key per
	/// packaged app, and one per program under `NonPackaged`.
	const STORE: &str = r"Software\Microsoft\Windows\CurrentVersion\CapabilityAccessManager\ConsentStore";
	const NOT_PACKAGED: &str = "NonPackaged";

	/// An app's key records a use that has not stopped: a start and a stop of 0.
	fn using(key: &str) -> bool {
		qword(HKEY_CURRENT_USER, key, "LastUsedTimeStart").is_some_and(|start| start > 0)
			&& qword(HKEY_CURRENT_USER, key, "LastUsedTimeStop") == Some(0)
	}

	/// An app uses the device `device` now.
	fn in_use(device: &str) -> bool {
		let base = format!(r"{STORE}\{device}");
		subkeys(HKEY_CURRENT_USER, &base).into_iter().any(|app| {
			if app == NOT_PACKAGED {
				let programs = format!(r"{base}\{NOT_PACKAGED}");
				subkeys(HKEY_CURRENT_USER, &programs)
					.into_iter()
					.any(|program| using(&format!(r"{programs}\{program}")))
			} else {
				using(&format!(r"{base}\{app}"))
			}
		})
	}

	/// The microphone or the camera in use.
	pub(super) fn call() -> bool {
		in_use("microphone") || in_use("webcam")
	}

	/// Presentation mode, else a full-screen app in front, as the shell tells it (QuestPump's own window in front,
	/// Big view's full screen, is none: `front` is another app's).
	pub(super) fn shown(front: Option<&Front>) -> Option<Hold> {
		let ours = procs::foreground_pid() == Some(std::process::id());
		// SAFETY: SHQueryUserNotificationState only writes the state it returns.
		match unsafe { SHQueryUserNotificationState() } {
			Ok(QUNS_PRESENTATION_MODE) => Some(Hold::Present),
			Ok(QUNS_BUSY | QUNS_RUNNING_D3D_FULL_SCREEN) if !ours && front.is_some() => Some(Hold::FullScreen),
			_ => None,
		}
	}
}
