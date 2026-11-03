//! GDI, the last backend (rust-helper `windows-capture-and-install.md`): `BitBlt` from the screen at the client area's
//! top-left into a top-down 32-bit DIB section of the band's size, at most 10 times a second, each read stamped when
//! it was made (GDI keeps no frame time). Every GDI object is released on every path: the screen's DC after each
//! copy, the section and its DC when the stream ends.

use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use questpump_core::strip;
use tokio::sync::mpsc::UnboundedSender;
use windows::Win32::Graphics::Gdi::{
	BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BitBlt, CreateCompatibleDC, CreateDIBSection, DIB_RGB_COLORS, DeleteDC,
	GdiFlush, GetDC, HBITMAP, HDC, HGDIOBJ, ReleaseDC, SRCCOPY, SelectObject,
};
use windows::core::Owned;

use super::{BAND, Event, Heard, Origin, Worker, byte_size, clock, send};

/// The section's row and size in bytes: 32-bit rows of the band's width, which are DWORD-aligned as DIB rows must be.
const PITCH: usize = strip::WIDTH * 4;
const BYTES: usize = PITCH * strip::CAPTURE_HEIGHT;

/// Starts copying the band from the screen on a thread of its own, `rate` times a second at most.
pub fn start(rate: u32, origin: Origin, events: &UnboundedSender<Event>) -> Result<Worker, String> {
	let step = Duration::from_secs(1) / rate.max(1);
	let events = events.clone();
	Worker::spawn("capture-gdi", Section::new, move |section, halt, heard| {
		run(&section, halt, heard, step, &origin, &events);
		Ok(())
	})
}

/// Copies and reads the band until halted. A screen that cannot be read (the secure desktop, GDI out of handles) is
/// the strip out of sight, sent and logged once for each run of failed copies; the next good copy reads again. Each
/// round counts as heard, failed or not: GDI is the last backend, and nothing would take over from it.
fn run(
	section: &Section,
	halt: &AtomicBool,
	heard: &Heard,
	step: Duration,
	origin: &Origin,
	events: &UnboundedSender<Event>,
) {
	let mut failing = false;
	while !halt.load(Ordering::Relaxed) {
		heard.now();
		match section.copy(origin.get()) {
			Ok(()) => {
				failing = false;
				let read = strip::read_band(section.pixels(), strip::WIDTH, strip::CAPTURE_HEIGHT, PITCH);
				send(events, read, clock::unix_now);
			}
			Err(e) if !failing => {
				failing = true;
				log::warn!("capture: GDI could not read the screen: {e}");
				let _ = events.send(Event::NoStrip);
			}
			Err(_) => {}
		}
		thread::sleep(step);
	}
}

/// A top-down 32-bit DIB section of the band's size, selected into a memory DC. Its fields drop in order: the DC's
/// first bitmap goes back in, then the section and the DC are deleted.
struct Section {
	_selected: Selected,
	_bitmap: Owned<HBITMAP>,
	dc: MemoryDc,
	bits: *mut u8,
}

impl Section {
	fn new() -> Result<Self, String> {
		// SAFETY: CreateCompatibleDC(None) makes a memory DC like the screen's; `MemoryDc` is its only owner.
		let dc = MemoryDc(unsafe { CreateCompatibleDC(None) });
		if dc.0.is_invalid() {
			return Err("no memory DC".into());
		}
		#[expect(clippy::cast_possible_wrap, reason = "390 and 96 pixels")]
		let (width, height) = (BAND.0 as i32, BAND.1 as i32);
		let info = BITMAPINFO {
			bmiHeader: BITMAPINFOHEADER {
				biSize: byte_size::<BITMAPINFOHEADER>(),
				biWidth: width,
				biHeight: -height, // negative: top-down rows
				biPlanes: 1,
				biBitCount: 32,
				biCompression: BI_RGB.0,
				..Default::default()
			},
			..Default::default()
		};
		let mut bits = std::ptr::null_mut();
		// SAFETY: CreateDIBSection reads `info` and writes the section's pixel pointer into `bits`, both locals that
		// live for the call.
		let bitmap = unsafe { CreateDIBSection(Some(dc.0), &raw const info, DIB_RGB_COLORS, &raw mut bits, None, 0) }
			.map_err(|e| format!("no DIB section: {e}"))?;
		// SAFETY: the bitmap is fresh from CreateDIBSection; `Owned` is its only owner and deletes it once.
		let bitmap = unsafe { Owned::new(bitmap) };
		if bits.is_null() {
			return Err("a DIB section without pixels".into());
		}
		// SAFETY: selects this section into this DC, both alive; `Selected` puts the returned bitmap back before
		// either is deleted.
		let first = unsafe { SelectObject(dc.0, HGDIOBJ::from(*bitmap)) };
		if first.is_invalid() {
			return Err("the DIB section could not be selected".into());
		}
		Ok(Self { _selected: Selected { dc: dc.0, first }, _bitmap: bitmap, dc, bits: bits.cast() })
	}

	/// Copies the band from the screen at `(x, y)` into the section, which is cleared first: a part of the band off
	/// every screen then reads black, never as the last copy's pixels.
	fn copy(&self, (x, y): (i32, i32)) -> Result<(), String> {
		// SAFETY: `bits` is the section's pixel memory, `BYTES` long, alive while `self` holds the bitmap; the last
		// `copy` flushed GDI, so no GDI call writes to it meanwhile.
		unsafe { std::ptr::write_bytes(self.bits, 0, BYTES) };
		// SAFETY: GetDC(None) gives the screen's DC or null; `ScreenDc` releases it once, right after the copy.
		let screen = ScreenDc(unsafe { GetDC(None) });
		if screen.0.is_invalid() {
			return Err("no screen DC".into());
		}
		#[expect(clippy::cast_possible_wrap, reason = "390 and 96 pixels")]
		let (width, height) = (BAND.0 as i32, BAND.1 as i32);
		// SAFETY: copies width × height pixels from the live screen DC at (x, y) into the section selected in the live
		// memory DC at (0, 0).
		let copied = unsafe { BitBlt(self.dc.0, 0, 0, width, height, Some(screen.0), x, y, SRCCOPY) };
		drop(screen);
		copied.map_err(|e| format!("BitBlt: {e}"))?;
		// SAFETY: GdiFlush has no preconditions; it ends this thread's batched GDI calls before the pixels are read.
		if unsafe { GdiFlush() }.as_bool() { Ok(()) } else { Err("GdiFlush failed".into()) }
	}

	fn pixels(&self) -> &[u8] {
		// SAFETY: `bits` is the section's pixel memory, `BYTES` long (top-down 32-bit rows of the band's width), alive
		// while `self` holds the bitmap; `copy` flushed GDI before this read, and nothing writes to it until the next
		// `copy` on this thread.
		unsafe { std::slice::from_raw_parts(self.bits, BYTES) }
	}
}

struct MemoryDc(HDC);

impl Drop for MemoryDc {
	fn drop(&mut self) {
		// SAFETY: the DC came from CreateCompatibleDC and is deleted once here, after its first bitmap went back in.
		let _ = unsafe { DeleteDC(self.0) };
	}
}

/// Puts the DC's first bitmap back, so the section is deleted unselected.
struct Selected {
	dc: HDC,
	first: HGDIOBJ,
}

impl Drop for Selected {
	fn drop(&mut self) {
		// SAFETY: the DC is alive (it drops after this) and `first` is the bitmap it held before the section.
		let _ = unsafe { SelectObject(self.dc, self.first) };
	}
}

struct ScreenDc(HDC);

impl Drop for ScreenDc {
	fn drop(&mut self) {
		if !self.0.is_invalid() {
			// SAFETY: the DC came from GetDC(None) and is released once here.
			let _ = unsafe { ReleaseDC(None, self.0) };
		}
	}
}
