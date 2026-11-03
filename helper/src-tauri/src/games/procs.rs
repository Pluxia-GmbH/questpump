//! How the helper sees the other programs on the computer without opening them (docs/plan.md slice 35; rust-helper rule
//! 10): one snapshot of the processes a look, which the games on the computer and `game.rs`'s World of Warcraft clients
//! share. Windows: a Toolhelp snapshot, image names only (a path would take a handle; World of Warcraft keeps its
//! one limited-query handle in `game.rs`); macOS: each process's image path (`libproc`), as `game.rs` always read it. The
//! window list gives the program in front, the screen a game's window is on and the picker's programs, and on macOS the
//! capture's look at the followed client's window (`capture/macos.rs`): on Windows the top-level windows, on macOS the
//! window server's list, whose owner fields need no Screen Recording grant.

use std::path::PathBuf;

/// A running process, seen without a handle.
#[derive(Debug, Clone)]
pub struct Proc {
	pub pid: u32,
	/// Its image file's name, such as `cs2.exe` or `dota2`.
	pub name: String,
	/// Its image's full path on macOS; none on Windows, where reading it takes a handle.
	pub path: Option<PathBuf>,
}

/// Every process at one look.
#[derive(Debug, Default)]
pub struct Snapshot {
	pub procs: Vec<Proc>,
}

impl Snapshot {
	/// A process's image name.
	#[cfg_attr(not(windows), expect(dead_code, reason = "Windows tells processes by their image name"))]
	pub fn name_of(&self, pid: u32) -> Option<&str> {
		self.procs.iter().find(|p| p.pid == pid).map(|p| p.name.as_str())
	}
}

/// A screen's area: on Windows a monitor's rectangle in physical pixels, on macOS a display's bounds in points of the
/// global display space (its origin the main display's top left), as Tauri's monitors give them on each.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Area {
	pub x: i32,
	pub y: i32,
	pub width: i32,
	pub height: i32,
}

/// A program the picker offers: its key (`Watched::Program`'s: on Windows the image name in lower case, on macOS the
/// app bundle's path, or the executable's outside a bundle), the name it shows (a window's title on Windows, the app's
/// name on macOS) and, on Windows, its image name beside it; its process, for the helper alone.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Pick {
	pub key: String,
	pub name: String,
	pub image: Option<String>,
	#[serde(skip)]
	pub pid: u32,
}

#[cfg(target_os = "macos")]
use mac as platform;
#[cfg(target_os = "macos")]
pub(crate) use mac::{display_at, largest, windows};
#[cfg(windows)]
use win as platform;
#[cfg(windows)]
pub(crate) use win::{area, main_window, process_id, top_level};

/// The process whose window is in front: the card shows once the game has left the front.
pub fn foreground_pid() -> Option<u32> {
	platform::foreground_pid()
}

/// The screen a process's largest window is on: the card goes to another screen when there is one.
pub fn window_monitor(pid: u32) -> Option<Area> {
	platform::window_monitor(pid)
}

/// The programs the picker offers, from the windows on screen.
pub fn pick_list(snap: &Snapshot) -> Vec<Pick> {
	platform::pick_list(snap)
}

impl Snapshot {
	/// Every process now; none when the system will not list them.
	#[cfg(target_os = "macos")]
	pub fn take() -> Self {
		let Ok(pids) = libproc::processes::pids_by_type(libproc::processes::ProcFilter::All) else {
			return Self::default();
		};
		let procs = pids
			.into_iter()
			.filter_map(|pid| {
				let path = PathBuf::from(libproc::proc_pid::pidpath(i32::try_from(pid).ok()?).ok()?);
				let name = path.file_name()?.to_str()?.to_owned();
				Some(Proc { pid, name, path: Some(path) })
			})
			.collect();
		Self { procs }
	}

	/// Every process now; none when the system will not list them.
	#[cfg(windows)]
	pub fn take() -> Self {
		Self { procs: win::processes() }
	}
}

#[cfg(target_os = "macos")]
mod mac {
	use std::path::{Path, PathBuf};

	use objc2_core_foundation::{CFArray, CFDictionary, CFNumber, CFRetained, CFString, CFType, CGPoint, CGRect};
	use objc2_core_graphics::{
		CGDisplayBounds, CGGetDisplaysWithPoint, CGRectMakeWithDictionaryRepresentation, CGWindowListCopyWindowInfo,
		CGWindowListOption, kCGNullWindowID, kCGWindowBounds, kCGWindowLayer, kCGWindowNumber, kCGWindowOwnerName,
		kCGWindowOwnerPID,
	};

	use super::{Area, Pick};

	/// One entry of the window server's list.
	pub(crate) struct Window {
		pid: u32,
		/// Its window number, ScreenCaptureKit's window id.
		id: u32,
		owner: Option<String>,
		bounds: Option<CGRect>,
	}

	/// The on-screen windows of the normal layer, front to back (the window server's order), desktop elements left
	/// out: their owner, number, owner's name and bounds, which need no Screen Recording grant (only a window's title
	/// does).
	pub(crate) fn windows() -> Vec<Window> {
		let option = CGWindowListOption::OptionOnScreenOnly | CGWindowListOption::ExcludeDesktopElements;
		let Some(list) = CGWindowListCopyWindowInfo(option, kCGNullWindowID) else { return Vec::new() };
		// SAFETY: CGWindowListCopyWindowInfo returns an array of window dictionaries with string keys (Apple's
		// "Required Window List Keys"); each value is read below only after a checked downcast.
		let list = unsafe { CFRetained::cast_unchecked::<CFArray<CFDictionary<CFString, CFType>>>(list) };
		// SAFETY: the keys are the framework's own constant strings, which live as long as the process.
		let (layer_key, pid_key, id_key, owner_key, bounds_key) =
			unsafe { (kCGWindowLayer, kCGWindowOwnerPID, kCGWindowNumber, kCGWindowOwnerName, kCGWindowBounds) };
		let number = |entry: &CFDictionary<CFString, CFType>, key: &CFString| {
			entry.get(key).and_then(|v| v.downcast::<CFNumber>().ok()).and_then(|n| n.as_i64())
		};
		list.iter()
			.filter(|entry| number(entry, layer_key) == Some(0))
			.filter_map(|entry| {
				let pid = u32::try_from(number(&entry, pid_key)?).ok()?;
				let id = u32::try_from(number(&entry, id_key)?).ok()?;
				let owner = entry.get(owner_key).and_then(|v| v.downcast::<CFString>().ok()).map(|s| s.to_string());
				let bounds = entry.get(bounds_key).and_then(|v| v.downcast::<CFDictionary>().ok()).and_then(|d| {
					let mut rect = CGRect::default();
					// SAFETY: writes one CGRect through a pointer to `rect`, which lives for the call.
					unsafe { CGRectMakeWithDictionaryRepresentation(Some(&d), &raw mut rect) }.then_some(rect)
				});
				Some(Window { pid, id, owner, bounds })
			})
			.collect()
	}

	/// The process whose window is in front: the first window of the normal layer.
	pub fn foreground_pid() -> Option<u32> {
		windows().first().map(|w| w.pid)
	}

	/// The display that holds the centre of the process's largest window, as its bounds in points.
	pub fn window_monitor(pid: u32) -> Option<Area> {
		let (_, largest) = largest(&windows(), pid)?;
		let centre =
			CGPoint { x: largest.origin.x + largest.size.width / 2.0, y: largest.origin.y + largest.size.height / 2.0 };
		let b = display_at(centre)?;
		#[expect(clippy::cast_possible_truncation, reason = "a display's bounds in points fit an i32")]
		Some(Area {
			x: b.origin.x.round() as i32,
			y: b.origin.y.round() as i32,
			width: b.size.width.round() as i32,
			height: b.size.height.round() as i32,
		})
	}

	/// The process's largest window among `windows` by area, of two as large the lower number (as the capture's
	/// `locate` breaks the tie, so both name one window): its number and bounds.
	pub(crate) fn largest(windows: &[Window], pid: u32) -> Option<(u32, CGRect)> {
		windows.iter().filter(|w| w.pid == pid).filter_map(|w| Some((w.id, w.bounds?))).max_by(|(ia, a), (ib, b)| {
			(a.size.width * a.size.height).total_cmp(&(b.size.width * b.size.height)).then(ib.cmp(ia))
		})
	}

	/// The bounds in points of the display that holds `point` in the global display space.
	pub(crate) fn display_at(point: CGPoint) -> Option<CGRect> {
		let (mut display, mut found) = (0u32, 0u32);
		// SAFETY: room for one display id and the count, both living for the call.
		unsafe { CGGetDisplaysWithPoint(point, 1, &raw mut display, &raw mut found) };
		(found > 0).then(|| CGDisplayBounds(display))
	}

	/// A process's key for the picker: the outermost app bundle around its executable (an app's helpers run inside
	/// it), else the executable itself.
	fn program_key(path: &Path) -> PathBuf {
		path.ancestors()
			.filter(|a| a.extension().is_some_and(|e| e.eq_ignore_ascii_case("app")))
			.last()
			.map_or_else(|| path.to_path_buf(), Path::to_path_buf)
	}

	/// The programs with a window on screen, each once by its key under its owner's name, this one left out.
	pub fn pick_list(snap: &super::Snapshot) -> Vec<Pick> {
		let own = std::process::id();
		let mut out: Vec<Pick> = Vec::new();
		for w in windows() {
			if w.pid == own {
				continue;
			}
			let Some(path) = snap.procs.iter().find(|p| p.pid == w.pid).and_then(|p| p.path.as_deref()) else {
				continue;
			};
			let key = program_key(path).to_string_lossy().into_owned();
			let name = w.owner.filter(|o| !o.trim().is_empty()).unwrap_or_else(|| {
				Path::new(&key).file_stem().map_or_else(String::new, |s| s.to_string_lossy().into_owned())
			});
			if !name.is_empty() && !out.iter().any(|p| p.key == key) {
				out.push(Pick { key, name, image: None, pid: w.pid });
			}
		}
		out
	}
}

#[cfg(windows)]
mod win {
	use std::mem::size_of;

	use windows::Win32::Foundation::{HWND, LPARAM, RECT};
	use windows::Win32::Graphics::Dwm::{DWMWA_CLOAKED, DwmGetWindowAttribute};
	use windows::Win32::Graphics::Gdi::{GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromWindow};
	use windows::Win32::System::Diagnostics::ToolHelp::{
		CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS,
	};
	use windows::Win32::UI::WindowsAndMessaging::{
		EnumWindows, GW_OWNER, GWL_EXSTYLE, GetForegroundWindow, GetWindow, GetWindowLongPtrW, GetWindowRect,
		GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible, WS_EX_TOOLWINDOW,
	};
	use windows::core::{BOOL, Owned};

	use super::{Area, Pick, Proc, Snapshot};

	/// The longest window title the picker reads, in UTF-16 units.
	const TITLE_UNITS: usize = 256;

	/// Every process's id and image name, from a Toolhelp snapshot: no process is opened.
	pub(super) fn processes() -> Vec<Proc> {
		// SAFETY: a snapshot of the process list; the handle is checked and owned below.
		let snap = match unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) } {
			Ok(handle) => handle,
			Err(e) => {
				log::warn!("processes: CreateToolhelp32Snapshot: {e}");
				return Vec::new();
			}
		};
		// SAFETY: CreateToolhelp32Snapshot returned a fresh handle; `Owned` closes it once on drop.
		let snap = unsafe { Owned::new(snap) };
		let mut entry =
			PROCESSENTRY32W { dwSize: u32::try_from(size_of::<PROCESSENTRY32W>()).unwrap_or(0), ..Default::default() };
		let mut out = Vec::new();
		// SAFETY: `entry` is a PROCESSENTRY32W with its size set, living for the call.
		let mut more = unsafe { Process32FirstW(*snap, &raw mut entry) }.is_ok();
		while more {
			let len = entry.szExeFile.iter().position(|&u| u == 0).unwrap_or(entry.szExeFile.len());
			if let Some(units) = entry.szExeFile.get(..len) {
				out.push(Proc { pid: entry.th32ProcessID, name: String::from_utf16_lossy(units), path: None });
			}
			// SAFETY: as above, the same snapshot and entry.
			more = unsafe { Process32NextW(*snap, &raw mut entry) }.is_ok();
		}
		out
	}

	/// The process a window belongs to.
	pub(crate) fn process_id(hwnd: HWND) -> Option<u32> {
		let mut pid = 0u32;
		// SAFETY: GetWindowThreadProcessId writes one u32 through a pointer to `pid`, which lives for the call.
		unsafe { GetWindowThreadProcessId(hwnd, Some(&raw mut pid)) };
		(pid != 0).then_some(pid)
	}

	/// Calls `visit` with every top-level window, through `EnumWindows` and a closure passed in its `LPARAM`.
	pub(crate) fn top_level<F: FnMut(HWND)>(mut visit: F) {
		unsafe extern "system" fn each<F: FnMut(HWND)>(hwnd: HWND, lparam: LPARAM) -> BOOL {
			// SAFETY: `lparam` carries the address of `visit` below, exposed there; EnumWindows calls this only during
			// the call, while `visit` lives and nothing else borrows it.
			let visit = unsafe { &mut *std::ptr::with_exposed_provenance_mut::<F>(lparam.0.cast_unsigned()) };
			visit(hwnd);
			true.into()
		}
		let lparam = LPARAM((&raw mut visit).expose_provenance().cast_signed());
		// SAFETY: `each::<F>` matches WNDENUMPROC and reads `lparam` only as the `F` it was made from, which outlives
		// the call; the callback always continues, so an error here is a real failure and there is nothing to undo.
		if let Err(e) = unsafe { EnumWindows(Some(each::<F>), lparam) } {
			log::warn!("windows: EnumWindows: {e}");
		}
	}

	/// The process whose window is in front.
	pub fn foreground_pid() -> Option<u32> {
		// SAFETY: reads the foreground window's handle, which may be null (no window in front).
		let hwnd = unsafe { GetForegroundWindow() };
		(!hwnd.is_invalid()).then(|| process_id(hwnd)).flatten()
	}

	/// The window's area in physical pixels, 0 when it cannot be read.
	pub(crate) fn area(hwnd: HWND) -> i64 {
		let mut rect = RECT::default();
		// SAFETY: GetWindowRect writes one RECT through a pointer to `rect`, which lives for the call.
		if unsafe { GetWindowRect(hwnd, &raw mut rect) }.is_err() {
			return 0;
		}
		(i64::from(rect.right) - i64::from(rect.left)) * (i64::from(rect.bottom) - i64::from(rect.top))
	}

	/// The process's largest visible top-level window: the one the card's screen follows, and the one ELDEN RING's
	/// boss banners are read off (`capture::banner`).
	pub(crate) fn main_window(pid: u32) -> Option<HWND> {
		let mut best: Option<(i64, HWND)> = None;
		top_level(|hwnd| {
			// SAFETY: IsWindowVisible only reads the state of a window handle EnumWindows just passed.
			if process_id(hwnd) != Some(pid) || !unsafe { IsWindowVisible(hwnd) }.as_bool() {
				return;
			}
			let size = area(hwnd);
			if best.is_none_or(|(s, _)| size > s) {
				best = Some((size, hwnd));
			}
		});
		best.map(|(_, hwnd)| hwnd)
	}

	/// The monitor of the process's largest visible top-level window, as its rectangle in physical pixels.
	pub fn window_monitor(pid: u32) -> Option<Area> {
		let hwnd = main_window(pid)?;
		// SAFETY: a monitor handle for a window that existed a moment ago; the nearest monitor if it moved off-screen.
		let monitor = unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST) };
		let mut info = MONITORINFO { cbSize: u32::try_from(size_of::<MONITORINFO>()).ok()?, ..Default::default() };
		// SAFETY: `info` is a MONITORINFO with its size set, living for the call.
		if !unsafe { GetMonitorInfoW(monitor, &raw mut info) }.as_bool() {
			return None;
		}
		let r = info.rcMonitor;
		Some(Area { x: r.left, y: r.top, width: r.right - r.left, height: r.bottom - r.top })
	}

	/// Whether a top-level window is one Alt+Tab would list: visible, not cloaked (a suspended store app's frame),
	/// owned by no other window and no tool window.
	fn listed(hwnd: HWND) -> bool {
		// SAFETY: reads the state of a window handle EnumWindows just passed.
		if !unsafe { IsWindowVisible(hwnd) }.as_bool() {
			return false;
		}
		let mut cloaked = 0u32;
		// SAFETY: writes one u32 through a pointer to `cloaked`, which lives for the call; its size is passed.
		let read = unsafe {
			DwmGetWindowAttribute(
				hwnd,
				DWMWA_CLOAKED,
				(&raw mut cloaked).cast(),
				u32::try_from(size_of::<u32>()).unwrap_or(4),
			)
		};
		if read.is_ok() && cloaked != 0 {
			return false;
		}
		// SAFETY: as above; an error is "no owner".
		let owned = unsafe { GetWindow(hwnd, GW_OWNER) }.is_ok();
		// SAFETY: as above.
		let style = unsafe { GetWindowLongPtrW(hwnd, GWL_EXSTYLE) };
		!owned && style & isize::try_from(WS_EX_TOOLWINDOW.0).unwrap_or(0) == 0
	}

	/// A window's title, empty when it has none.
	fn title(hwnd: HWND) -> String {
		let mut buf = [0u16; TITLE_UNITS];
		// SAFETY: writes at most the slice's length of UTF-16 units into `buf`, which lives for the call.
		let len = usize::try_from(unsafe { GetWindowTextW(hwnd, &mut buf) }).unwrap_or(0);
		buf.get(..len).map(String::from_utf16_lossy).unwrap_or_default().trim().to_owned()
	}

	/// The programs with a window Alt+Tab lists, each once by its image name with its first such window's title, this
	/// one left out.
	pub fn pick_list(snap: &Snapshot) -> Vec<Pick> {
		let own = std::process::id();
		let mut out: Vec<Pick> = Vec::new();
		top_level(|hwnd| {
			let Some(pid) = process_id(hwnd).filter(|&p| p != own) else { return };
			let Some(image) = snap.name_of(pid) else { return };
			let key = image.to_lowercase();
			if out.iter().any(|p| p.key == key) || !listed(hwnd) {
				return;
			}
			let name = title(hwnd);
			if !name.is_empty() {
				out.push(Pick { key, name, image: Some(image.to_owned()), pid });
			}
		});
		out
	}
}
