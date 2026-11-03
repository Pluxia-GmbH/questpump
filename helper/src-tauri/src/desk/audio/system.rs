//! What the desktop app's sound asks of the system (`desk::audio`): whether a screen reader runs. No line is said in
//! the system's voice: every line is a recorded take (David, 2026-10-16).

/// A screen reader runs: macOS's own (`isVoiceOverEnabled`).
#[cfg(target_os = "macos")]
pub(crate) fn screen_reader() -> bool {
	objc2_app_kit::NSWorkspace::sharedWorkspace().isVoiceOverEnabled()
}

/// A screen reader runs: Windows' flag that one is (`SPI_GETSCREENREADER`, which Narrator, NVDA and JAWS set).
#[cfg(windows)]
pub(crate) fn screen_reader() -> bool {
	use windows::Win32::UI::WindowsAndMessaging::{
		SPI_GETSCREENREADER, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, SystemParametersInfoW,
	};
	use windows::core::BOOL;
	let mut on = BOOL(0);
	// SAFETY: SPI_GETSCREENREADER writes one BOOL through the pointer, which points at `on` for the call.
	let read = unsafe {
		SystemParametersInfoW(
			SPI_GETSCREENREADER,
			0,
			Some((&raw mut on).cast()),
			SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
		)
	};
	read.is_ok() && on.as_bool()
}
