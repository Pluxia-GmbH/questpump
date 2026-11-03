//! ELDEN RING's and ELDEN RING NIGHTREIGN's boss banners off the game's own window (2026-10-29; docs/computer-games.md
//! §6.2; questpump-core `feeds::banner`): while the game runs and is watched, a second Windows.Graphics.Capture session,
//! beside the strip's and never sharing it, on the game's largest window (`games::procs::main_window`), only where the
//! capture's border can be turned off, as the strip's WGC (rust-helper rule 6). At most `banner::RATE` frames a second
//! (the system's minimum update interval where it has one, and `wgc::Throttle` always), the cursor left out where the
//! system can, `Bgra8`, or `Rgba16F` on an HDR output brought back to its SDR white (`Scrgb`). Of each frame only the band
//! across the client area's middle (`banner::TOP` to `banner::BOTTOM` of its height, its full width) is copied, brought
//! down to `banner::WIDTH` pixels wide and tested (`banner::look`); a banner the gate passes is read by Windows' own OCR
//! (`Windows.Media.Ocr`, the player's profile languages); where the system has no OCR for them no session starts and
//! the game is heard at its close. Every frame, band and word read lives in memory for that test alone: none is kept, logged or sent. One thread,
//! "banner", looks at the window every 2 s, starts the session, moves its crop as the window moves or resizes and starts
//! it again for another window, display or colour format; it stops the session and ends once its handle is dropped.

use std::sync::Arc;
use std::sync::mpsc as std_mpsc;
use std::time::Duration;

use questpump_core::feeds::banner::{self, Banners, Step};
use questpump_core::strip::Scrgb;
use windows::Graphics::Imaging::{BitmapPixelFormat, SoftwareBitmap};
use windows::Media::Ocr::OcrEngine;
use windows::Win32::Foundation::HWND;
use windows_capture::capture::{CaptureControl, Context, GraphicsCaptureApiHandler};
use windows_capture::frame::Frame;
use windows_capture::graphics_capture_api::InternalCaptureControl;
use windows_capture::settings::{
	ColorFormat, CursorCaptureSettings, DirtyRegionSettings, DrawBorderSettings, MinimumUpdateIntervalSettings,
	SecondaryWindowSettings, Settings,
};
use windows_capture::window::Window;

use super::{Format, Look, Origin, clock, display, wgc, window};
use crate::games::procs;

/// What the capture tells: its session started (the game is heard), or a boss fell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Told {
	Up,
	Fell,
}

/// Whom the capture tells, from its own threads.
pub type Teller = Arc<dyn Fn(Told) + Send + Sync>;

/// The banners' capture while its game wants it: dropping the handle ends its thread at its next look, which stops the
/// session there, never on the thread that dropped it.
pub struct Watch {
	_stop: std_mpsc::Sender<()>,
}

impl Watch {
	/// Starts the thread on the game process `pid`'s window; none when no thread could start (logged).
	pub fn start(pid: u32, tell: Teller) -> Option<Self> {
		let (stop, stopped) = std_mpsc::channel();
		let spawned = std::thread::Builder::new().name("banner".into()).spawn(move || run(pid, &stopped, &tell));
		match spawned {
			Ok(_) => Some(Self { _stop: stop }),
			Err(e) => {
				log::warn!("banner: no thread: {e}");
				None
			}
		}
	}
}

/// What a session was started for; any change starts it again (a move or a resize only moves its crop).
#[derive(Debug, Clone, Copy, PartialEq)]
struct Target {
	window: usize,
	monitor: usize,
	hdr: bool,
	white: f32,
}

/// The running session, what it was started for, and its crop's cells.
struct Live {
	target: Target,
	origin: Origin,
	size: Origin,
	control: CaptureControl<Reader, anyhow::Error>,
}

/// The thread: a look every 2 s (`capture::TICK`) until the handle is dropped.
fn run(pid: u32, stopped: &std_mpsc::Receiver<()>, tell: &Teller) {
	super::dpi_aware();
	let probes = wgc::probe();
	if !probes.supported || !probes.border {
		log::info!("banner: no borderless window capture on this system; the game is heard at its close");
		return;
	}
	if OcrEngine::TryCreateFromUserProfileLanguages().is_err() {
		log::info!("banner: no OCR for this profile's languages; the game is heard at its close");
		return;
	}
	wgc::ask_borderless();
	let mut live: Option<Live> = None;
	let mut look: Option<Look> = None;
	let mut failed: Option<String> = None;
	loop {
		if wgc::border_denied() {
			log::info!("banner: borderless capture was denied; the game is heard at its close");
			break;
		}
		watch(pid, probes, &mut live, &mut look, &mut failed, tell);
		match stopped.recv_timeout(crate::capture::TICK) {
			Err(std_mpsc::RecvTimeoutError::Timeout) => {}
			Ok(()) | Err(std_mpsc::RecvTimeoutError::Disconnected) => break,
		}
	}
	end(&mut live);
}

/// One look at the game's window: the session keeps running and follows the window, starts again for another window,
/// display or colour format or after it stopped, and stops while the window does not show.
fn watch(
	pid: u32,
	probes: wgc::Probes,
	live: &mut Option<Live>,
	look: &mut Option<Look>,
	failed: &mut Option<String>,
	tell: &Teller,
) {
	let hwnd = procs::main_window(pid).filter(|&hwnd| window::shows(hwnd));
	let place = hwnd.and_then(|hwnd| Some((hwnd, window::client_in_frame(hwnd)?, window::client_size(hwnd)?)));
	let monitor = hwnd.and_then(display::of);
	let (Some((hwnd, origin, size)), Some(monitor)) = (place, monitor) else {
		end(live);
		return;
	};
	let look = match look.take_if(|l| !l.stale(&monitor)) {
		Some(fresh) => look.insert(fresh),
		None => look.insert(Look::read(&monitor)),
	};
	let target = Target { window: hwnd.0.addr(), monitor: monitor.handle.0.addr(), hdr: look.hdr, white: look.white };
	if let Some(l) = live.as_ref()
		&& l.target == target
		&& !l.control.is_finished()
	{
		l.origin.set(origin);
		l.size.set(size);
		return;
	}
	end(live);
	let (origin, size) = (Origin::new(origin), Origin::new(size));
	match start(hwnd, probes, target, origin.clone(), size.clone(), tell) {
		Ok(control) => {
			*failed = None;
			*live = Some(Live { target, origin, size, control });
			tell(Told::Up);
		}
		Err(e) => {
			// once for each new reason: the next look tries again
			if failed.as_ref() != Some(&e) {
				log::warn!("banner: the capture did not start: {e}");
			}
			*failed = Some(e);
		}
	}
}

/// Stops the session, if one runs, and joins its thread.
fn end(live: &mut Option<Live>) {
	if let Some(l) = live.take()
		&& let Err(e) = l.control.stop()
	{
		log::warn!("banner: the capture stopped: {e}");
	}
}

/// Starts a session on the game's window; the crate returns once `StartCapture` succeeded on its own thread.
fn start(
	hwnd: HWND,
	probes: wgc::Probes,
	target: Target,
	origin: Origin,
	size: Origin,
	tell: &Teller,
) -> Result<CaptureControl<Reader, anyhow::Error>, String> {
	let settings = Settings::new(
		Window::from_raw_hwnd(hwnd.0),
		if probes.cursor { CursorCaptureSettings::WithoutCursor } else { CursorCaptureSettings::Default },
		DrawBorderSettings::WithoutBorder,
		SecondaryWindowSettings::Default,
		if probes.interval {
			MinimumUpdateIntervalSettings::Custom(Duration::from_secs(1) / banner::RATE)
		} else {
			MinimumUpdateIntervalSettings::Default
		},
		DirtyRegionSettings::Default,
		if target.hdr { ColorFormat::Rgba16F } else { ColorFormat::Bgra8 },
		Flags { origin, size, scrgb: target.hdr.then(|| Scrgb::new(target.white)), tell: Arc::clone(tell) },
	);
	Reader::start_free_threaded(settings).map_err(|e| e.to_string())
}

pub struct Flags {
	origin: Origin,
	size: Origin,
	scrgb: Option<Scrgb>,
	tell: Teller,
}

/// The session's frame handler: throttle, crop, bring down, test, read.
pub struct Reader {
	origin: Origin,
	size: Origin,
	scrgb: Option<Scrgb>,
	tell: Teller,
	throttle: wgc::Throttle,
	banners: Banners,
	/// Windows' OCR for the player's profile languages, made on the session's thread; none without one.
	ocr: Option<OcrEngine>,
	/// The OCR failed in this session (logged once).
	ocr_failed: bool,
}

impl GraphicsCaptureApiHandler for Reader {
	type Flags = Flags;
	type Error = anyhow::Error;

	fn new(ctx: Context<Self::Flags>) -> Result<Self, Self::Error> {
		let Flags { origin, size, scrgb, tell } = ctx.flags;
		let ocr = OcrEngine::TryCreateFromUserProfileLanguages().ok();
		Ok(Self {
			origin,
			size,
			scrgb,
			tell,
			throttle: wgc::Throttle::new(banner::RATE, clock::HUNDRED_NS),
			banners: Banners::default(),
			ocr,
			ocr_failed: false,
		})
	}

	/// Only a failed GPU readback ends the session (the next look starts it again); a frame without room for the band
	/// is skipped.
	fn on_frame_arrived(&mut self, frame: &mut Frame, _control: InternalCaptureControl) -> Result<(), Self::Error> {
		let Ok(at) = frame.timestamp() else { return Ok(()) };
		if !self.throttle.due(at.Duration) {
			return Ok(());
		}
		let format = match frame.color_format() {
			ColorFormat::Bgra8 => Format::Bgra8,
			ColorFormat::Rgba16F => Format::Rgba16F,
			ColorFormat::Rgba8 => return Ok(()), // never asked for
		};
		let Some((left, top, wide, high)) = crop(self.origin.get(), self.size.get(), (frame.width(), frame.height()))
		else {
			return Ok(());
		};
		let mut pixels = frame.buffer_crop(left, top, left + wide, top + high)?;
		let pitch = usize::try_from(pixels.row_pitch())?;
		let band = (usize::try_from(wide)?, usize::try_from(high)?);
		let Some((rgb, width, height)) = shrink(pixels.as_raw_buffer(), band, pitch, format, self.scrgb.as_ref())
		else {
			return Ok(());
		};
		let ms = at.Duration / (clock::HUNDRED_NS / 1_000);
		let fell = match self.banners.push(banner::look(&rgb, width, height), ms) {
			Step::Wait => false,
			Step::Fell => true,
			Step::Read => self.read(&rgb, width, height).is_some_and(|text| self.banners.read(&text, ms)),
		};
		if fell {
			(self.tell)(Told::Fell);
		}
		Ok(())
	}
}

impl Reader {
	/// The band's words as the OCR reads them (light words on the dark strip turned dark on light, and doubled in size
	/// while that stays within the OCR's largest image); none without an OCR or when it failed (logged once a session).
	fn read(&mut self, rgb: &[u8], width: usize, height: usize) -> Option<String> {
		let ocr = self.ocr.as_ref()?;
		match recognise(ocr, rgb, width, height) {
			Ok(text) => Some(text),
			Err(e) => {
				if !std::mem::replace(&mut self.ocr_failed, true) {
					log::warn!("banner: the OCR failed: {e}");
				}
				None
			}
		}
	}
}

/// The OCR's text of the band: each pixel's luma inverted, as BGRA, scaled up `2` where the OCR's largest image allows.
fn recognise(ocr: &OcrEngine, rgb: &[u8], width: usize, height: usize) -> windows::core::Result<String> {
	let largest = usize::try_from(OcrEngine::MaxImageDimension()?).unwrap_or(0);
	let scale = if width * 2 <= largest && height * 2 <= largest { 2 } else { 1 };
	let (wide, high) = (width * scale, height * scale);
	let mut bgra = vec![0u8; wide * high * 4];
	for (at, out) in bgra.as_chunks_mut::<4>().0.iter_mut().enumerate() {
		let from = ((at / wide / scale) * width + (at % wide) / scale) * 3;
		let [red, green, blue] = [rgb[from], rgb[from + 1], rgb[from + 2]].map(u32::from);
		let ink = u8::try_from(255 - (299 * red + 587 * green + 114 * blue) / 1000).unwrap_or(0);
		*out = [ink, ink, ink, 255];
	}
	let (wide, high) = (i32::try_from(wide).unwrap_or(0), i32::try_from(high).unwrap_or(0));
	let buffer = crate::blelink::buffer(&bgra)?;
	let bitmap = SoftwareBitmap::CreateCopyFromBuffer(&buffer, BitmapPixelFormat::Bgra8, wide, high)?;
	let text = ocr.RecognizeAsync(&bitmap)?.join()?.Text()?;
	Ok(text.to_string_lossy())
}

/// The band's place in a frame of `frame` size: the client area's (at `origin` inside the frame, `size` big) full width,
/// from `banner::TOP` to `banner::BOTTOM` of its height, when it lies inside the frame (`buffer_crop` takes end
/// coordinates and checks no bounds, as `wgc::crop` says).
fn crop(origin: (i32, i32), size: (i32, i32), frame: (u32, u32)) -> Option<(u32, u32, u32, u32)> {
	let (x, y) = (u32::try_from(origin.0).ok()?, u32::try_from(origin.1).ok()?);
	let (width, height) = (u32::try_from(size.0).ok()?, u32::try_from(size.1).ok()?);
	let at = |share: f64| {
		#[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "a share of a window's height")]
		let row = (f64::from(height) * share).round() as u32;
		row
	};
	let (top, bottom) = (y.checked_add(at(banner::TOP))?, y.checked_add(at(banner::BOTTOM))?);
	let right = x.checked_add(width)?;
	(width > 0 && bottom > top && right <= frame.0 && bottom <= frame.1).then_some((x, top, width, bottom - top))
}

/// The band brought down to `banner::WIDTH` pixels wide (each pixel the mean of the block it stands for; never brought
/// up) as R, G, B bytes, with its width and height: BGRA as captured, or scRGB at the display's SDR white.
fn shrink(
	pixels: &[u8],
	(cols, rows): (usize, usize),
	pitch: usize,
	format: Format,
	scrgb: Option<&Scrgb>,
) -> Option<(Vec<u8>, usize, usize)> {
	// an HDR frame's table comes with its session (`Flags`): none means a format never asked for
	let (bytes, scrgb) = if format == Format::Bgra8 { (4, None) } else { (8, Some(scrgb?)) };
	if cols == 0 || rows == 0 || pitch < cols * bytes || pixels.len() < pitch * (rows - 1) + cols * bytes {
		return None;
	}
	let width = cols.min(banner::WIDTH);
	let height = (rows * width / cols).max(1);
	// a pixel's R, G and B
	let colour = |col: usize, row: usize| -> [u32; 3] {
		let at = row * pitch + col * bytes;
		match scrgb {
			None => [pixels[at + 2], pixels[at + 1], pixels[at]].map(u32::from),
			Some(table) => {
				[at, at + 2, at + 4].map(|ch| u32::from(table.byte(u16::from_le_bytes([pixels[ch], pixels[ch + 1]]))))
			}
		}
	};
	// the source's span of the output's `i`th of `n` steps over `len`: at least one
	let span = |i: usize, n: usize, len: usize| (i * len / n)..((i + 1) * len / n).max(i * len / n + 1);
	let mut out = vec![0u8; width * height * 3];
	for (out_row, line) in out.chunks_exact_mut(width * 3).enumerate() {
		let down = span(out_row, height, rows);
		for (out_col, px) in line.as_chunks_mut::<3>().0.iter_mut().enumerate() {
			let across = span(out_col, width, cols);
			let mut sum = [0u32; 3];
			for row in down.clone() {
				for col in across.clone() {
					let [red, green, blue] = colour(col, row);
					sum = [sum[0] + red, sum[1] + green, sum[2] + blue];
				}
			}
			let count = u32::try_from(down.len() * across.len()).unwrap_or(1).max(1);
			for (byte, total) in px.iter_mut().zip(sum) {
				*byte = u8::try_from(total / count).unwrap_or(u8::MAX);
			}
		}
	}
	Some((out, width, height))
}
