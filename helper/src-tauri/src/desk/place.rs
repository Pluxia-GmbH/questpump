//! The main window's place (docs/ui.md, "The window model"): its size, its position and its screen kept in
//! settings.json (`window`: written a second after each move or resize ends, `windows::SETTLE`, and at its close),
//! restored only onto a screen that is still there (by its name and its position), else opened on the screen under the
//! pointer (else the main one) at a share of that screen's work area (`FIRST_WIDTH`, `FIRST_HEIGHT`) and centred in it;
//! never smaller than 480 × 600 or the screen's work area, if that is smaller, and kept inside the work area with its
//! frame (the title bar and the borders, measured on the window once it is built and before it shows). Positions are
//! in the desktop's own units: points on macOS, whose desktop is one space in points while each screen's pixels are its
//! points times its own scale, and physical pixels on Windows, whose virtual screen is one space in pixels. A minimized
//! window, or one in its own full screen, keeps the place it had before. No window-state plugin: it would show and
//! focus a restored window, write only at the app's exit and keep its file in a second folder.

use questpump_desk::settings::Place;
use tauri::{AppHandle, LogicalPosition, Monitor, PhysicalPosition, Position, Runtime, WebviewWindow};

/// A share of one side of a screen's work area, held between two sizes, in logical pixels.
struct Share {
	part: f64,
	least: f64,
	most: f64,
}

impl Share {
	fn of(&self, side: f64) -> f64 {
		(side * self.part).clamp(self.least, self.most)
	}
}

/// The first window's inner size, a share of its screen's work area (David, 2026-10-15: "how to start the app bigger
/// and not in phone size"): two thirds of the width between 960 and 1490 and nine tenths of the height between 640
/// and 1000 (a 13-inch laptop's about 980 × 790, a 1080p screen's at 100 % about 1280 × 930), so the rail stands
/// beside the page from the first opening (720 CSS px at the page's 125 %, `windows::BASE`: 900 logical pixels) and
/// the Book's widest grid on a large screen (1490 since 2026-10-17, when it held the three talent trees side by side,
/// which the ladders' paths replaced on 2026-10-29: 1192 CSS px); `Area::fit` then holds it inside the work area with
/// its frame.
const FIRST_WIDTH: Share = Share { part: 2.0 / 3.0, least: 960.0, most: 1490.0 };
const FIRST_HEIGHT: Share = Share { part: 0.9, least: 640.0, most: 1000.0 };
/// The window's size where no screen is listed, tauri.conf.json's `main` too (1280 × 860: the size a window built from
/// the config has until this module sizes it, before it shows), in logical pixels.
const WIDTH: f64 = 1280.0;
const HEIGHT: f64 = 860.0;
/// The first window's size until 2026-10-15 (520 × 760, a phone's shape): a place saved at exactly that size was never
/// resized, and `Settings::load` drops it once (`never_resized`, `Settings::window_grown`).
const OLD: (f64, f64) = (520.0, 760.0);
/// The least size, where the work area allows it.
const MIN_WIDTH: f64 = 480.0;
const MIN_HEIGHT: f64 = 600.0;

/// How a new window stands: its inner size, its outer position (none without any screen listed: the config's `center`)
/// and whether it opens maximized.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Frame {
	pub(crate) width: f64,
	pub(crate) height: f64,
	pub(crate) position: Option<Position>,
	pub(crate) maximized: bool,
}

/// The window's frame around its page (the title bar and the borders) in logical pixels: its width, its height.
pub(crate) type Chrome = (f64, f64);

/// A position in physical pixels on a screen or a window of this scale, in the desktop's units.
fn units(physical: i32, scale: f64) -> i32 {
	if cfg!(target_os = "macos") {
		#[expect(clippy::cast_possible_truncation, reason = "a screen's points fit an i32")]
		let points = (f64::from(physical) / scale).round() as i32;
		points
	} else {
		physical
	}
}

/// A position in the desktop's units, as the window takes it.
fn position(x: i32, y: i32) -> Position {
	if cfg!(target_os = "macos") {
		LogicalPosition::new(f64::from(x), f64::from(y)).into()
	} else {
		PhysicalPosition::new(x, y).into()
	}
}

/// A screen's work area: its origin in the desktop's units, its size in logical pixels, and how many of the desktop's
/// units a logical pixel spans there.
struct Area {
	x: i32,
	y: i32,
	width: f64,
	height: f64,
	per: f64,
}

impl Area {
	fn of(monitor: &Monitor) -> Self {
		let (wa, scale) = (monitor.work_area(), monitor.scale_factor());
		Self {
			x: units(wa.position.x, scale),
			y: units(wa.position.y, scale),
			width: f64::from(wa.size.width) / scale,
			height: f64::from(wa.size.height) / scale,
			per: if cfg!(target_os = "macos") { 1.0 } else { scale },
		}
	}

	/// An inner size held between the least size and the work area, the frame counted.
	fn fit(&self, width: f64, height: f64, chrome: Chrome) -> (f64, f64) {
		let (room_w, room_h) = (self.width - chrome.0, self.height - chrome.1);
		(width.clamp(MIN_WIDTH.min(room_w), room_w), height.clamp(MIN_HEIGHT.min(room_h), room_h))
	}

	/// The first window's inner size on this area (`FIRST_WIDTH`, `FIRST_HEIGHT`), held inside it with its frame.
	fn first(&self, chrome: Chrome) -> (f64, f64) {
		self.fit(FIRST_WIDTH.of(self.width), FIRST_HEIGHT.of(self.height), chrome)
	}

	/// Where an edge of a window `size` logical pixels long may stand along one axis: from the area's start to where
	/// the window's far edge meets the area's.
	fn span(&self, origin: i32, extent: f64, size: f64) -> (i32, i32) {
		#[expect(clippy::cast_possible_truncation, reason = "a screen's pixels fit an i32")]
		let room = ((extent - size) * self.per).max(0.0).round() as i32;
		(origin, origin + room)
	}

	/// The outer position nearest (x, y) at which a window of this outer size stays inside the area.
	fn inside(&self, (x, y): (i32, i32), (width, height): (f64, f64)) -> (i32, i32) {
		let ((x0, x1), (y0, y1)) = (self.span(self.x, self.width, width), self.span(self.y, self.height, height));
		(x.clamp(x0, x1), y.clamp(y0, y1))
	}

	/// The outer position that centres a window of this outer size in the area.
	fn centre(&self, (width, height): (f64, f64)) -> (i32, i32) {
		let ((x0, x1), (y0, y1)) = (self.span(self.x, self.width, width), self.span(self.y, self.height, height));
		(x0 + (x1 - x0) / 2, y0 + (y1 - y0) / 2)
	}
}

/// The frame of a window to open from its saved place, its own frame (`chrome`) counted in its size and its position.
pub(crate) fn restore<R: Runtime>(app: &AppHandle<R>, saved: Option<&Place>, chrome: Chrome) -> Frame {
	let monitors = app.available_monitors().unwrap_or_default();
	let screen = saved.and_then(|place| {
		monitors.iter().find(|m| {
			let (at, scale) = (m.position(), m.scale_factor());
			m.name() == place.monitor.as_ref()
				&& units(at.x, scale) == place.monitor_x
				&& units(at.y, scale) == place.monitor_y
		})
	});
	if let (Some(place), Some(monitor)) = (saved, screen) {
		let area = Area::of(monitor);
		let (width, height) = area.fit(place.width, place.height, chrome);
		let (x, y) = area.inside((place.x, place.y), (width + chrome.0, height + chrome.1));
		return Frame { width, height, position: Some(position(x, y)), maximized: place.maximized };
	}
	let main = app.primary_monitor().ok().flatten();
	let here = under_pointer(app, main.as_ref()).or(main).or_else(|| monitors.into_iter().next());
	let Some(area) = here.as_ref().map(Area::of) else {
		return Frame { width: WIDTH, height: HEIGHT, position: None, maximized: false };
	};
	let (width, height) = area.first(chrome);
	let (x, y) = area.centre((width + chrome.0, height + chrome.1));
	Frame { width, height, position: Some(position(x, y)), maximized: false }
}

/// The screen under the pointer, where the player just clicked the tray, the Dock or the Start menu. tao 0.37.2 gives
/// the pointer in physical pixels, on macOS the desktop's points times the main screen's scale
/// (`util::cursor_position`), and finds a screen by a point in the desktop's units (`monitor::from_point`:
/// `CGDisplayBounds`, in points, on macOS; `MonitorFromPoint`, in pixels, on Windows).
fn under_pointer<R: Runtime>(app: &AppHandle<R>, main: Option<&Monitor>) -> Option<Monitor> {
	let at = app.cursor_position().ok()?;
	let scale = if cfg!(target_os = "macos") { main?.scale_factor() } else { 1.0 };
	app.monitor_from_point(at.x / scale, at.y / scale).ok().flatten()
}

/// A place saved at the first window's old size (`OLD`): 520 wide to the logical pixel and 760 high, or shorter where
/// a short work area held the old window's height (a 1080p laptop at 150 %): the player never resized it, so it stands
/// for no place at all (`Settings::load`, once); a maximized one was the player's choice and stays.
pub(crate) fn never_resized(place: &Place) -> bool {
	!place.maximized && (place.width - OLD.0).abs() < 0.5 && place.height < OLD.1 + 0.5
}

/// The window's frame around its page: its outer size less its inner size, in logical pixels.
pub(crate) fn chrome<R: Runtime>(window: &WebviewWindow<R>) -> Option<Chrome> {
	let scale = window.scale_factor().ok()?;
	let (outer, inner) = (window.outer_size().ok()?, window.inner_size().ok()?);
	Some((
		f64::from(outer.width.saturating_sub(inner.width)) / scale,
		f64::from(outer.height.saturating_sub(inner.height)) / scale,
	))
}

/// The least inner size on the window's screen now: 480 × 600, or the work area less the window's frame, if smaller.
pub(crate) fn minimum<R: Runtime>(window: &WebviewWindow<R>) -> Option<(f64, f64)> {
	let area = Area::of(&window.current_monitor().ok().flatten()?);
	let (width, height) = chrome(window)?;
	Some((MIN_WIDTH.min(area.width - width), MIN_HEIGHT.min(area.height - height)))
}

/// Where the window stands now; while it is maximized its size and position stay the ones `before` kept, and while it
/// is minimized (Windows parks it at -32000 with no size) or in its own full screen nothing changes (none).
pub(crate) fn capture<R: Runtime>(window: &WebviewWindow<R>, before: Option<&Place>) -> Option<Place> {
	if window.is_minimized().unwrap_or(false) || window.is_fullscreen().unwrap_or(false) {
		return None;
	}
	let maximized = window.is_maximized().unwrap_or(false);
	let monitor = window.current_monitor().ok().flatten()?;
	let (at, monitor_scale) = (monitor.position(), monitor.scale_factor());
	let (monitor_x, monitor_y) = (units(at.x, monitor_scale), units(at.y, monitor_scale));
	let name = monitor.name().cloned();
	if maximized && let Some(before) = before {
		return Some(Place { monitor: name, monitor_x, monitor_y, maximized, ..before.clone() });
	}
	let scale = window.scale_factor().ok()?;
	let (position, size) = (window.outer_position().ok()?, window.inner_size().ok()?);
	Some(Place {
		x: units(position.x, scale),
		y: units(position.y, scale),
		width: f64::from(size.width) / scale,
		height: f64::from(size.height) / scale,
		monitor: name,
		monitor_x,
		monitor_y,
		maximized,
	})
}
