//! The game window's state and place as the capture reads them: whether it shows, its client area's top-left on the
//! screen and inside the window's frame, and its scale. Physical pixels throughout: the process is per-monitor-v2 DPI
//! aware (`dpi_aware` in `mod.rs`).

use windows::Win32::Foundation::{HWND, POINT, RECT};
use windows::Win32::Graphics::Dwm::{DWMWA_CLOAKED, DWMWA_EXTENDED_FRAME_BOUNDS, DwmGetWindowAttribute};
use windows::Win32::Graphics::Gdi::ClientToScreen;
use windows::Win32::System::StationsAndDesktops::{DESKTOP_CONTROL_FLAGS, DESKTOP_READOBJECTS, OpenInputDesktop};
use windows::Win32::UI::HiDpi::GetDpiForWindow;
use windows::Win32::UI::WindowsAndMessaging::{GetClientRect, IsIconic, IsWindowVisible};
use windows::core::Owned;

use super::byte_size;

/// Whether the window shows on a screen: visible, not minimised and not cloaked (on another virtual desktop, or kept
/// off screen by DWM while it reports visible).
pub fn shows(hwnd: HWND) -> bool {
	// SAFETY: IsWindowVisible and IsIconic only read the window's state; a stale handle reads as not visible.
	let open = unsafe { IsWindowVisible(hwnd).as_bool() && !IsIconic(hwnd).as_bool() };
	let mut cloaked = 0u32;
	// SAFETY: DWMWA_CLOAKED is a u32, written into `cloaked`, whose size the call is given.
	let read = unsafe { DwmGetWindowAttribute(hwnd, DWMWA_CLOAKED, (&raw mut cloaked).cast(), byte_size::<u32>()) };
	open && !(read.is_ok() && cloaked != 0)
}

/// The client area's top-left on the screen: where DXGI and GDI read the band.
pub fn client_origin(hwnd: HWND) -> Option<(i32, i32)> {
	let mut at = POINT::default();
	// SAFETY: ClientToScreen reads and writes one POINT through a pointer to `at`, which lives for the call.
	unsafe { ClientToScreen(hwnd, &raw mut at) }.as_bool().then_some((at.x, at.y))
}

/// The client area's top-left inside the window's frame, which is what Windows.Graphics.Capture captures: its screen
/// top-left minus the DWM extended frame bounds' (OBS's `get_client_box`). Never windows-capture's
/// `Window::title_bar_height`, which scales a physical height by the DPI a second time.
pub fn client_in_frame(hwnd: HWND) -> Option<(i32, i32)> {
	let mut frame = RECT::default();
	// SAFETY: DWMWA_EXTENDED_FRAME_BOUNDS is a RECT, written into `frame`, whose size the call is given.
	unsafe { DwmGetWindowAttribute(hwnd, DWMWA_EXTENDED_FRAME_BOUNDS, (&raw mut frame).cast(), byte_size::<RECT>()) }
		.ok()?;
	let (x, y) = client_origin(hwnd)?;
	Some((x - frame.left, y - frame.top))
}

/// The client area's width and height (ELDEN RING's boss banners, `banner.rs`: the band across its middle).
pub fn client_size(hwnd: HWND) -> Option<(i32, i32)> {
	let mut rect = RECT::default();
	// SAFETY: GetClientRect writes one RECT through a pointer to `rect`, which lives for the call.
	unsafe { GetClientRect(hwnd, &raw mut rect) }.ok()?;
	Some((rect.right - rect.left, rect.bottom - rect.top))
}

/// The window's scale: its DPI over 96.
pub fn scale(hwnd: HWND) -> f64 {
	// SAFETY: GetDpiForWindow only reads the window's DPI; 0 for a stale handle.
	let dpi = unsafe { GetDpiForWindow(hwnd) };
	if dpi == 0 { 1.0 } else { f64::from(dpi) / 96.0 }
}

/// Whether the player's own desktop takes the input: not while the PC is locked or a UAC prompt holds the secure
/// desktop, where no backend sees the game (a quiet WGC session there is no stall, and DXGI refuses the duplication),
/// and the helper cannot open the input desktop.
pub fn desktop_active() -> bool {
	// SAFETY: opens the input desktop for reading only; the handle is closed below.
	let Ok(desktop) = (unsafe { OpenInputDesktop(DESKTOP_CONTROL_FLAGS(0), false, DESKTOP_READOBJECTS) }) else {
		return false;
	};
	// SAFETY: `desktop` was just opened; `Owned` closes it once, here.
	drop(unsafe { Owned::new(desktop) });
	true
}
