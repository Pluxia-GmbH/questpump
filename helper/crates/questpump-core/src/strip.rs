//! Reading the pixel strip: two rows of 65 cells of 6 × 6 physical pixels (390 × 12) at the game window's top-left
//! corner. The frame is 96 bytes (the anchor colour first, the version, the fields, an 8-bit checksum); the strip paints
//! it as 2-bit symbols, each channel of a data cell one of four levels, beside a column of anchor and four grey
//! calibration cells that show the four levels as the capture sees them. The game's brightness, contrast and gamma
//! (and an HDR output) move every painted value, so the reader measures the levels in each capture instead of trusting
//! a painted value. The table is wow-addon-dev's `pixel-strip.md`, the encoder the addon's `Bridge.lua`.

use serde::Serialize;

use crate::codes::{STRIP_VERSION, StripEvent, StripState};

pub const COLUMNS: usize = 65;
pub const ROWS: usize = 2;
pub const CELL_PX: usize = 6;
/// The strip's size in physical pixels: the helper crops `WIDTH` wide.
pub const WIDTH: usize = COLUMNS * CELL_PX;
pub const HEIGHT: usize = ROWS * CELL_PX;
pub const ANCHOR: [u8; 3] = [42, 0, 69];
/// How far each channel of an anchor cell may lie from the painted anchor before the strip counts as absent: the
/// game's brightness, contrast and gamma move it (David's client read 43, 0, 72 on 2026-09-30).
pub const ANCHOR_TOLERANCE: u8 = 32;
/// The level each 2-bit symbol paints in a channel; calibration cells 1-4 of row 0 paint them as greys, in this order.
pub const LEVELS: [u8; 4] = [32, 96, 160, 224];
/// The least a calibration level may lie above the one before it, in every channel, for the capture to read.
const MIN_STEP: u8 = 8;
/// The frame's bytes (the table's cells 1-32, three bytes each).
pub const BYTES: usize = 96;
/// The first data cell of each row: after the anchor and the calibration in row 0, after the anchor in row 1.
const DATA_FROM: [usize; ROWS] = [5, 1];

// Bytes 4-96 (the anchor is not painted as data) fill the data cells' channels exactly, four symbols a byte.
const _: () = assert!((COLUMNS - DATA_FROM[0] + COLUMNS - DATA_FROM[1]) * 3 == (BYTES - 3) * 4);

/// Why a frame was refused. A refused frame is dropped without a word: a mid-repaint capture fails its checksum.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DecodeError {
	#[error("no anchor")]
	Anchor,
	#[error("strip version {0}, this helper reads {STRIP_VERSION}")]
	Version(u8),
	#[error("checksum")]
	Checksum,
	/// The calibration cells' levels are not apart enough in some channel to tell the symbols apart.
	#[error("calibration levels too close to read")]
	Calibration,
	#[error("unknown event code {0}")]
	Event(u8),
	#[error("unknown state code {0}")]
	State(u8),
	#[error("byte {0} is not zero")]
	NotZero(usize),
}

/// A frame as its cells hold it, every value raw (the names are `Bridge.lua`'s).
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct RawFrame {
	pub version: u8,
	pub seq: u8,
	/// Bit 0 a heartbeat, bit 1 a frame shown again after a cinematic, bits 2-3 the tier, bits 4-7 the setup word
	/// ([`RawFrame::setup`]).
	pub flags: u8,
	pub event: StripEvent,
	pub level: u8,
	pub kind: u8,
	pub load_id: u16,
	/// Counts events, never frames: with the load id it identifies an event for good.
	pub event_no: u16,
	pub body: Body,
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum Body {
	Fields(Fields),
	Chunk(Chunk),
}

/// The cells of every frame but a profile or sheet chunk.
#[derive(Serialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct Fields {
	pub quest_id: u32,
	/// `GetTime()` in milliseconds modulo 2^24.
	pub ts_client: u32,
	/// The countdown's start in the same units, 0 when no move is counting down or running.
	pub start_at: u32,
	pub yards_foot: u32,
	pub yards_mount: u32,
	pub yards_taxi: u32,
	pub yards_swim: u32,
	pub reps: u16,
	pub state: StripState,
	pub seconds: u16,
	/// A row's outcome; on `pump_offered` the pump's move count, on `pump_done` how it ended ([`crate::codes::PumpEnd`]).
	pub outcome: u8,
	pub slot: u8,
	pub ladder: u8,
	pub rung: u8,
	pub row_flags: u16,
	pub move_id: u16,
	pub char_index: u8,
	/// A row's own time; the pump's offered time on `pump_offered`, `countdown_start`, heartbeat and `pump_done`
	/// frames.
	pub row_ts: u32,
	/// On `pump_offered`: the reps the owed row would carry (0, 1 an all-out window; an addon from before 2026-10-15
	/// paints 2 on a closing move, read as a window too).
	pub owed_reps: u8,
	/// On `pump_offered`: 1 when the move would be owed at a logout (not a rider, not a pump the player asked for).
	pub owes: u8,
}

/// One chunk of a profile or sheet payload.
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Chunk {
	pub chunk: u8,
	pub chunks: u8,
	/// Which copy of the payload the chunk belongs to (it wraps): a new copy replaces a partial old one.
	pub serial: u8,
	/// The payload's total length, repeated in every chunk.
	pub length: u16,
	pub data: Vec<u8>,
}

impl RawFrame {
	#[must_use]
	pub fn is_heartbeat(&self) -> bool {
		self.flags & 1 == 1
	}

	#[must_use]
	pub fn tier(&self) -> u8 {
		(self.flags >> 2) & 3
	}

	/// The setup word the addon paints in every frame since 2026-10-13 (cell 2's flags, bits 4-7): none while bit 4
	/// is clear, an addon from before it. No frame is refused for these bits.
	#[must_use]
	pub fn setup(&self) -> Option<SetupWord> {
		(self.flags & 16 != 0).then_some(SetupWord {
			check_shown: self.flags & 32 != 0,
			helper_seen: self.flags & 64 != 0,
			save_unread: self.flags & 128 != 0,
		})
	}
}

/// What the game's load says of its setup (`Bridge.lua`'s setup word): the pairing's step (`link::Step`) is read
/// from it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetupWord {
	/// The setup window asks for its Check again: the companions' page with a step open, on screen or held back by a
	/// fight, a pump or a cinematic (bit 5).
	pub check_shown: bool,
	/// This load saw QuestPump on this computer running, by its sign of life (bit 6).
	pub helper_seen: bool,
	/// This client does not read its own save back (the Forever beta), so its saved progress comes only from the
	/// mirror (bit 7).
	pub save_unread: bool,
}

fn uint(b: &[u8], at: usize, n: usize) -> u32 {
	b[at..at + n].iter().fold(0, |v, &x| (v << 8) | u32::from(x))
}

// Byte `i` of the table's 1-based numbering, as `Bridge.lua` and `pixel-strip.md` count.
fn at(b: &[u8; BYTES], i: usize) -> u8 {
	b[i - 1]
}

fn uint_at(b: &[u8; BYTES], i: usize, n: usize) -> u32 {
	uint(b, i - 1, n)
}

fn u16_at(b: &[u8; BYTES], i: usize) -> u16 {
	u16::from_be_bytes([b[i - 1], b[i]])
}

/// Decodes the 96 bytes of one frame: the anchor, the version, the checksum, the codes, and zeros where the table
/// keeps zeros.
///
/// # Errors
/// The first check the bytes fail.
pub fn decode_bytes(b: &[u8; BYTES]) -> Result<RawFrame, DecodeError> {
	if b[..3] != ANCHOR {
		return Err(DecodeError::Anchor);
	}
	if at(b, 4) != STRIP_VERSION {
		return Err(DecodeError::Version(at(b, 4)));
	}
	let sum = b[3..93].iter().fold(0u8, |s, &x| s.wrapping_add(x));
	if sum != at(b, 94) || at(b, 95) != 0 || at(b, 96) != 0 {
		return Err(DecodeError::Checksum);
	}
	let event = StripEvent::from_code(at(b, 7)).ok_or(DecodeError::Event(at(b, 7)))?;
	let body = if event.is_blob() {
		let mut data: Vec<u8> = b[15..39].to_vec();
		data.extend_from_slice(&b[42..93]);
		let length = u16::from_be_bytes([data[0], data[1]]);
		Body::Chunk(Chunk { chunk: at(b, 10), chunks: at(b, 11), serial: at(b, 12), length, data: data.split_off(2) })
	} else {
		// bytes 58-59 (cell 20, the jog that led a closing move until 2026-10-15) are neither read nor checked, so an
		// older addon's frames still decode
		for i in [53, 54, 60].into_iter().chain(61..=93) {
			if at(b, i) != 0 {
				return Err(DecodeError::NotZero(i));
			}
		}
		Body::Fields(Fields {
			quest_id: uint_at(b, 10, 4),
			ts_client: uint_at(b, 16, 3),
			start_at: uint_at(b, 19, 3),
			yards_foot: uint_at(b, 22, 3),
			yards_mount: uint_at(b, 25, 3),
			yards_taxi: uint_at(b, 28, 3),
			reps: u16_at(b, 31),
			state: StripState::from_code(at(b, 33)).ok_or(DecodeError::State(at(b, 33)))?,
			seconds: u16_at(b, 34),
			outcome: at(b, 36),
			yards_swim: uint_at(b, 37, 3),
			slot: at(b, 42),
			ladder: at(b, 43),
			rung: at(b, 44),
			row_flags: u16::from(at(b, 45)) | (u16::from(at(b, 55)) << 8),
			move_id: u16_at(b, 46),
			char_index: at(b, 48),
			row_ts: uint_at(b, 49, 4),
			owed_reps: at(b, 56),
			owes: at(b, 57),
		})
	};
	Ok(RawFrame {
		version: at(b, 4),
		seq: at(b, 5),
		flags: at(b, 6),
		event,
		level: at(b, 8),
		kind: at(b, 9),
		load_id: u16_at(b, 14),
		event_no: u16_at(b, 40),
		body,
	})
}

/// What one capture shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Read {
	/// No anchor in the corner: a movie, a cinematic, a hidden interface, the game not showing the addon.
	NoStrip,
	/// The strip, decoded.
	Frame(RawFrame),
	/// The anchor is there but the frame fails a check (a capture in the middle of a repaint): dropped.
	Rejected(DecodeError),
}

/// Where each data symbol is painted, in order: (cell, channel), the cells numbered row 0 then row 1, left to right, red,
/// green and blue in each.
fn slots() -> impl Iterator<Item = (usize, usize)> {
	(0..ROWS)
		.flat_map(|row| (DATA_FROM[row]..COLUMNS).map(move |column| row * COLUMNS + column))
		.flat_map(|cell| (0..3).map(move |channel| (cell, channel)))
}

fn is_anchor(px: [u8; 3]) -> bool {
	px.iter().zip(ANCHOR).all(|(&v, a)| v.abs_diff(a) <= ANCHOR_TOLERANCE)
}

/// Whether `bgra` holds `height` BGRA rows of `width` pixels, `bytes_per_row` apart.
fn holds(bgra: &[u8], width: usize, height: usize, bytes_per_row: usize) -> bool {
	width > 0 && bgra.len() >= bytes_per_row.saturating_mul(height) && bytes_per_row >= width * 4
}

/// The pixel at `x`, `y` of BGRA rows `bytes_per_row` apart, as red, green and blue.
fn rgb(bgra: &[u8], bytes_per_row: usize, x: usize, y: usize) -> [u8; 3] {
	let p = y * bytes_per_row + x * 4;
	[bgra[p + 2], bgra[p + 1], bgra[p]]
}

/// The anchor column's centre in a capture `width` pixels wide.
fn anchor_x(width: usize) -> usize {
	(CELL_PX / 2 * width / WIDTH).min(width - 1)
}

/// The symbol whose calibration value lies nearest `v`; a tie goes to the lower symbol.
fn nearest(v: u8, levels: [u8; 4]) -> u8 {
	let mut best = 0;
	for (s, &level) in (0u8..).zip(&levels) {
		if v.abs_diff(level) < v.abs_diff(levels[usize::from(best)]) {
			best = s;
		}
	}
	best
}

/// Reads the strip's band: BGRA pixels, `bytes_per_row` apart (a row may carry padding), each cell at its centre
/// pixel, scaled when the band is not exactly 390 × 12. Both anchor cells must lie within `ANCHOR_TOLERANCE`; the four
/// calibration cells give each channel's levels as captured, which must rise by at least `MIN_STEP` each, and each data
/// channel reads as the symbol whose level lies nearest.
#[must_use]
pub fn read(bgra: &[u8], width: usize, height: usize, bytes_per_row: usize) -> Read {
	if height == 0 || !holds(bgra, width, height, bytes_per_row) {
		return Read::NoStrip;
	}
	let cell = |cell: usize| {
		let (row, column) = (cell / COLUMNS, cell % COLUMNS);
		let x = ((column * CELL_PX + CELL_PX / 2) * width / WIDTH).min(width - 1);
		let y = ((row * CELL_PX + CELL_PX / 2) * height / HEIGHT).min(height - 1);
		rgb(bgra, bytes_per_row, x, y)
	};
	if !(0..ROWS).all(|row| is_anchor(cell(row * COLUMNS))) {
		return Read::NoStrip;
	}
	let measured: [[u8; 3]; 4] = std::array::from_fn(|k| cell(1 + k));
	let levels: [[u8; 4]; 3] = std::array::from_fn(|channel| measured.map(|px| px[channel]));
	// four calibration cells of one colour are a flat area near the anchor's colour (a night sky with the interface
	// hidden), never a strip whose levels came too close
	if levels.iter().all(|l| l[3].abs_diff(l[0]) < MIN_STEP) {
		return Read::NoStrip;
	}
	if levels.iter().any(|l| l.windows(2).any(|w| u16::from(w[1]) < u16::from(w[0]) + u16::from(MIN_STEP))) {
		return Read::Rejected(DecodeError::Calibration);
	}
	let mut bytes = [0u8; BYTES];
	bytes[..3].copy_from_slice(&ANCHOR);
	for (n, (at, channel)) in slots().enumerate() {
		let byte = &mut bytes[3 + n / 4];
		*byte = (*byte << 2) | nearest(cell(at)[channel], levels[channel]);
	}
	decode_bytes(&bytes).map_or_else(Read::Rejected, Read::Frame)
}

/// The first `n` cells of the strip's first row as a capture shows them (the anchor, the four calibration cells, then
/// data), from the band `read_band` finds: the helper logs them for a capture that does not read, to show how the
/// game's colours reached it.
#[must_use]
pub fn first_cells(bgra: &[u8], width: usize, height: usize, bytes_per_row: usize, n: usize) -> Vec<[u8; 3]> {
	if !holds(bgra, width, height, bytes_per_row) {
		return Vec::new();
	}
	let x = anchor_x(width);
	let anchor = |y: usize| is_anchor(rgb(bgra, bytes_per_row, x, y));
	let band = (height * HEIGHT / CAPTURE_HEIGHT).max(1);
	let Some(top) = (0..height.saturating_sub(band - 1)).find(|&y| anchor(y) && anchor(y + band - 1)) else {
		return Vec::new();
	};
	let y = top + (CELL_PX / 2 * band / HEIGHT).min(band - 1);
	(0..n.min(COLUMNS))
		.map(|column| rgb(bgra, bytes_per_row, ((column * CELL_PX + CELL_PX / 2) * width / WIDTH).min(width - 1), y))
		.collect()
}

/// The anchor column of a capture (`CAPTURE_HEIGHT` rows from the game window's corner) as it shows it, a cell's
/// height apart from the top down: the helper logs it for a capture without the strip, to show what was in the corner
/// (a windowed game's title bar, the clear background of a window off screen, the game with its interface hidden, a
/// movie or a loading screen: the capture holds the game's window alone).
#[must_use]
pub fn anchor_column(bgra: &[u8], width: usize, height: usize, bytes_per_row: usize) -> Vec<[u8; 3]> {
	if !holds(bgra, width, height, bytes_per_row) {
		return Vec::new();
	}
	let x = anchor_x(width);
	let cell = (height * CELL_PX / CAPTURE_HEIGHT).max(1); // a cell's rows at this capture's scale
	(cell / 2..height).step_by(cell).map(|y| rgb(bgra, bytes_per_row, x, y)).collect()
}

/// The rows the capture takes at the game window's top-left corner: the strip's 12 plus room for a windowed game's title
/// bar (28 points on a 2x display is 56 pixels).
pub const CAPTURE_HEIGHT: usize = 96;

/// Reads a capture taller than the strip (`CAPTURE_HEIGHT` rows from the window's corner): the strip's band starts at
/// the first row where the anchor column shows the anchor both there and on the band's last row, below a title bar in a
/// windowed game and at the top in fullscreen.
#[must_use]
pub fn read_band(bgra: &[u8], width: usize, height: usize, bytes_per_row: usize) -> Read {
	if !holds(bgra, width, height, bytes_per_row) {
		return Read::NoStrip;
	}
	let x = anchor_x(width);
	let anchor = |y: usize| is_anchor(rgb(bgra, bytes_per_row, x, y));
	let band = (height * HEIGHT / CAPTURE_HEIGHT).max(1); // the strip's rows at this capture's scale
	let Some(top) = (0..height.saturating_sub(band - 1)).find(|&y| anchor(y) && anchor(y + band - 1)) else {
		return Read::NoStrip;
	};
	read(&bgra[top * bytes_per_row..], width, band, bytes_per_row)
}

/// Reading a capture of an HDR output. Windows composes an HDR desktop in scRGB (linear light, 1.0 = 80 nits) and a
/// capture of it arrives as `Rgba16F`: four IEEE half floats a pixel, little-endian, in R, G, B, A order. The game's
/// SDR content sits at the display's SDR white level, so each channel is divided by that white, clamped, brought back
/// through the sRGB transfer and read as `read_band` reads BGRA, whose calibration cells take up what is left (rust-helper
/// `windows-capture-and-install.md`, "HDR and the decoder rule"). A table of all 65,536 half values per white level
/// keeps a frame at a lookup per channel.
pub struct Scrgb {
	lut: Vec<u8>,
}

impl Scrgb {
	/// The table for an SDR white of `sdr_white` × 80 nits (Windows' `SDRWhiteLevel` / 1000); a white that is not a
	/// positive number reads as 1.0.
	#[must_use]
	pub fn new(sdr_white: f32) -> Self {
		let white = if sdr_white.is_finite() && sdr_white > 0.0 { sdr_white } else { 1.0 };
		Self { lut: (0..=u16::MAX).map(|bits| srgb_byte(half(bits) / white)).collect() }
	}

	/// One channel's half float (its bits) as an sRGB byte at the display's SDR white: the strip's band here, ELDEN
	/// RING's boss banners in the desktop app (`feeds::banner`).
	#[must_use]
	pub fn byte(&self, bits: u16) -> u8 {
		self.lut[usize::from(bits)]
	}

	/// Reads a capture of the corner: `Rgba16F` pixels, `bytes_per_row` apart, as `read_band` reads BGRA.
	#[must_use]
	pub fn read_band(&self, rgba16f: &[u8], width: usize, height: usize, bytes_per_row: usize) -> Read {
		if width == 0 || rgba16f.len() < bytes_per_row.saturating_mul(height) || bytes_per_row < width * 8 {
			return Read::NoStrip;
		}
		let byte = |p: usize| self.byte(u16::from_le_bytes([rgba16f[p], rgba16f[p + 1]]));
		let mut bgra = vec![0u8; width * height * 4];
		for y in 0..height {
			for x in 0..width {
				let p = y * bytes_per_row + x * 8;
				let q = (y * width + x) * 4;
				bgra[q..q + 4].copy_from_slice(&[byte(p + 4), byte(p + 2), byte(p), 255]);
			}
		}
		read_band(&bgra, width, height, width * 4)
	}
}

/// An IEEE 754 half float's value.
fn half(bits: u16) -> f32 {
	let sign = if bits & 0x8000 == 0 { 1.0 } else { -1.0 };
	let exponent = i32::from((bits >> 10) & 0x1f);
	let fraction = f32::from(bits & 0x3ff);
	match exponent {
		0 => sign * fraction * 2f32.powi(-24),
		31 if fraction == 0.0 => sign * f32::INFINITY,
		31 => f32::NAN,
		_ => sign * (1.0 + fraction / 1024.0) * 2f32.powi(exponent - 15),
	}
}

/// A linear value as an sRGB byte: clamped to 0..1 (NaN reads as black), then the sRGB transfer.
fn srgb_byte(linear: f32) -> u8 {
	let l = if linear.is_nan() { 0.0 } else { linear.clamp(0.0, 1.0) };
	let encoded = if l <= 0.003_130_8 { 12.92 * l } else { 1.055 * l.powf(1.0 / 2.4) - 0.055 };
	#[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "a value in 0..=255 after the clamp")]
	let byte = (encoded * 255.0).round().clamp(0.0, 255.0) as u8;
	byte
}
