//! Start at login (on by default for a published build, for each new program and whenever the system holds no record
//! of it, unless the player turned it off in the window: `runtime.rs`, `Helper::start_at_login`). The system's record
//! reads as one of three (`Record`): missing (never registered, or deleted with an uninstall), off (turned off in the
//! system's own settings, which the helper leaves alone) or on.
//!
//! macOS: the app itself as a login item through `SMAppService.mainApp` (macOS 13 and later), which System Settings
//! lists under Open at Login by the app's name and starts through LaunchServices, so the grants stay with the signed
//! bundle; a bare binary in development has no bundle to register (decided 2026-09-30 over a hand-written launchd
//! agent, which macOS files under Allow in the Background and which breaks when the app moves). A program at a new
//! path registers anew (`renew`), since the system's record of the old one may still read Enabled and name the old
//! path (a moved bundle, or the rename of slice 35 from `QuestPump Helper.app` to `QuestPump.app`).
//!
//! Windows: the value `QuestPump` under the user's `Run` key, the productName Tauri's uninstaller removes, with the
//! program's path in quotes (the installed path may have spaces, and an unquoted one is resolved piece by piece;
//! `tauri-plugin-autostart` writes it unquoted) and `--login` after it (`LOGIN`). A start turned off in Task Manager
//! (the `StartupApproved` value) reads as off, and turning it on in the helper's window turns it on there too. The
//! values the helper wrote under its name before slice 35, `QuestPump Helper`, go at every start (`forget_old`), so an
//! old value never starts an old program at login.

#[cfg(windows)]
pub use imp::forget_old;
pub use imp::{record, renew, set};

/// The system's record of the start at login.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Record {
	/// None: never registered, or deleted (Windows' uninstaller deletes the Run value, and a reinstall at the same path
	/// leaves `settings.json` as it was).
	Missing,
	/// Turned off in the system's own settings (Task Manager's Startup apps, System Settings' Login Items).
	Off,
	On,
}

/// Whether the helper starts with the computer; `None` where the system cannot say (a bare binary in development, the
/// registry unreadable).
pub fn state() -> Option<bool> {
	record().map(|r| r == Record::On)
}

#[cfg(target_os = "macos")]
mod imp {
	use anyhow::{anyhow, bail};
	use objc2_service_management::{
		SMAppService, SMAppServiceStatus, kSMErrorAlreadyRegistered, kSMErrorJobNotFound, kSMErrorLaunchDeniedByUser,
	};

	use super::Record;

	/// An app bundle's executable: the only thing `SMAppService.mainApp` can register.
	fn bundled() -> bool {
		std::env::current_exe().is_ok_and(|p| p.to_string_lossy().contains(".app/Contents/MacOS/"))
	}

	/// Whether an `NSError`'s code is one of ServiceManagement's `kSMError…` numbers.
	fn is(code: isize, error: u32) -> bool {
		isize::try_from(error).is_ok_and(|e| e == code)
	}

	/// The system's record for this bundle: not registered (or not found) is missing, waiting for the player's approval
	/// in System Settings is off; `None` for a bare binary in development.
	pub fn record() -> Option<Record> {
		if !bundled() {
			return None;
		}
		// SAFETY: `mainAppService` and `status` are messages to ServiceManagement's SMAppService, which is not bound to
		// the main thread; both only read the system's record for this bundle.
		let status = unsafe { SMAppService::mainAppService().status() };
		Some(match status {
			SMAppServiceStatus::Enabled => Record::On,
			SMAppServiceStatus::RequiresApproval => Record::Off,
			_ => Record::Missing,
		})
	}

	/// Starts with the computer, or not.
	///
	/// # Errors
	/// The system refused the change, or there is no bundle to register.
	pub fn set(on: bool) -> anyhow::Result<()> {
		if !bundled() {
			bail!("a development build is no app bundle, so it cannot start at login");
		}
		// SAFETY: as in `state`.
		let service = unsafe { SMAppService::mainAppService() };
		// SAFETY: register and unregister change only this bundle's login item and return an NSError on refusal.
		let result = unsafe { if on { service.registerAndReturnError() } else { service.unregisterAndReturnError() } };
		match result {
			Ok(()) => Ok(()),
			Err(e) if on && is(e.code(), kSMErrorAlreadyRegistered) => Ok(()),
			Err(e) if !on && is(e.code(), kSMErrorJobNotFound) => Ok(()),
			Err(e) if is(e.code(), kSMErrorLaunchDeniedByUser) => {
				// the player turned it off in System Settings, which alone can turn it on again: show them where
				// SAFETY: a class message that opens the Login Items pane.
				unsafe { SMAppService::openSystemSettingsLoginItems() };
				Err(anyhow!("turned off in System Settings, Login Items: {e}"))
			}
			Err(e) => Err(anyhow!("{e}")),
		}
	}

	/// Registers this program's bundle anew: unregistered first (the app keeps running: Apple's `unregister()`), then
	/// registered, so a record that names the bundle's old path is replaced.
	///
	/// # Errors
	/// The system refused either step, or there is no bundle to register.
	pub fn renew() -> anyhow::Result<()> {
		set(false)?;
		set(true)
	}
}

#[cfg(windows)]
mod imp {
	use anyhow::bail;
	use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, WIN32_ERROR};
	use windows::Win32::System::Registry::{
		HKEY_CURRENT_USER, REG_BINARY, REG_SZ, RRF_RT_REG_BINARY, RRF_RT_REG_SZ, RegDeleteKeyValueW, RegGetValueW,
		RegSetKeyValueW,
	};
	use windows::core::{HSTRING, PCWSTR, w};

	use super::Record;

	const RUN: PCWSTR = w!(r"Software\Microsoft\Windows\CurrentVersion\Run");
	const APPROVED: PCWSTR = w!(r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run");
	/// The productName: the value Tauri's NSIS uninstaller deletes from the Run key.
	const NAME: PCWSTR = w!("QuestPump");
	/// The productName before slice 35 (2026-10-08), whose values `forget_old` removes.
	const OLD_NAME: PCWSTR = w!("QuestPump Helper");
	/// Task Manager's record of a start that is on (an even first byte; an odd one marks it off, followed by when).
	const ENABLED: [u8; 12] = [2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

	fn check(e: WIN32_ERROR, what: &str) -> anyhow::Result<()> {
		if e.is_ok() { Ok(()) } else { bail!("{what}: {}", windows::core::Error::from(e.to_hresult())) }
	}

	/// The Run value and Task Manager's record: off when Task Manager turned it off (kept for its name even once the
	/// Run value is gone, so a reinstall respects it), else missing without the Run value, else on; `None` when the Run
	/// key cannot be read.
	pub fn record() -> Option<Record> {
		let mut size = 0u32;
		// SAFETY: a size query: no buffer, `size` receives the value's byte count.
		let run = unsafe { RegGetValueW(HKEY_CURRENT_USER, RUN, NAME, RRF_RT_REG_SZ, None, None, Some(&raw mut size)) };
		let present = run.is_ok();
		if !present && run != ERROR_FILE_NOT_FOUND {
			return None;
		}
		let on = approved();
		Some(match (present, on) {
			(_, false) => Record::Off,
			(false, true) => Record::Missing,
			(true, true) => Record::On,
		})
	}

	/// Task Manager's record does not say off: none, one in another shape, or an even first byte, as Windows reads it.
	fn approved() -> bool {
		let mut approved = [0u8; 12];
		let mut len = u32::try_from(approved.len()).unwrap_or(0);
		// SAFETY: `approved` holds `len` bytes; the call writes at most that many and sets `len` to what it wrote.
		let read = unsafe {
			RegGetValueW(
				HKEY_CURRENT_USER,
				APPROVED,
				NAME,
				RRF_RT_REG_BINARY,
				None,
				Some(approved.as_mut_ptr().cast()),
				Some(&raw mut len),
			)
		};
		!read.is_ok() || len == 0 || approved[0] % 2 == 0
	}

	/// Starts with the computer, or not.
	///
	/// # Errors
	/// The registry refused the change.
	pub fn set(on: bool) -> anyhow::Result<()> {
		if !on {
			// SAFETY: deletes one named value under the user's own key.
			let e = unsafe { RegDeleteKeyValueW(HKEY_CURRENT_USER, RUN, NAME) };
			return if e == ERROR_FILE_NOT_FOUND { Ok(()) } else { check(e, "the Run value") };
		}
		let program = std::env::current_exe()?;
		let command = HSTRING::from(format!("\"{}\" {}", program.display(), crate::desk::launch::LOGIN));
		let bytes = u32::try_from((command.len() + 1) * 2)?; // UTF-16 with its terminating NUL
		// SAFETY: `command` is a NUL-terminated UTF-16 string of `bytes` bytes, alive for the call.
		let e =
			unsafe { RegSetKeyValueW(HKEY_CURRENT_USER, RUN, NAME, REG_SZ.0, Some(command.as_ptr().cast()), bytes) };
		check(e, "the Run value")?;
		// SAFETY: `ENABLED` is 12 bytes, alive for the call.
		let e = unsafe {
			RegSetKeyValueW(HKEY_CURRENT_USER, APPROVED, NAME, REG_BINARY.0, Some(ENABLED.as_ptr().cast()), 12)
		};
		check(e, "Task Manager's startup record")
	}

	/// Writes the Run value anew for this program, whatever program it named before.
	///
	/// # Errors
	/// The registry refused the change.
	pub fn renew() -> anyhow::Result<()> {
		set(true)
	}

	/// Removes the Run value and Task Manager's record the helper wrote under its old name; whether one was there.
	///
	/// # Errors
	/// The registry refused a removal.
	pub fn forget_old() -> anyhow::Result<bool> {
		let mut removed = false;
		// copy-lint: not shown (the error's context in the log)
		for (key, what) in [(RUN, "the old Run value"), (APPROVED, "the old startup record")] {
			// SAFETY: deletes one named value under the user's own key.
			let e = unsafe { RegDeleteKeyValueW(HKEY_CURRENT_USER, key, OLD_NAME) };
			if e != ERROR_FILE_NOT_FOUND {
				check(e, what)?;
				removed = true;
			}
		}
		Ok(removed)
	}
}
