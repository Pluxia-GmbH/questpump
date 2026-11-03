//! A reel's frame from its parts (docs/ui.md, "The share card": the Reel), in BGRA, the order both encoders take: the
//! page's layers under the knight (the card's ground, a parchment card's dark well), the knight, the layers over it
//! (the words, the number and the strip at the reveal, a shot's slot), each layer a PNG the page drew on its canvas
//! (`lib/reel.js`) at its place for its frames. The knight is the move's film frame as `lib/figure.js` draws a film: the
//! base square with the tint square times the crimson added (`clamp(base + tint × colour / 255)`, the game's ADD),
//! cut to its tile and mirrored as the pack mirrors it, then drawn as `lib/share.js` `knight` draws the card's still: on
//! Night through a round mask (opaque to `mask` of its radius, clear at its edge) and lightened over the iron, on
//! Parchment and Gilded lightened into its rounded well (`radius`); "lighten" over an opaque ground is
//! `dst + max(0, src - dst) × coverage`, as a canvas composites it.

use std::io::Cursor;

use anyhow::{Context as _, bail};

/// A frame: BGRA rows, opaque, `width` × `height`.
pub(super) struct Canvas {
	pub(super) width: usize,
	pub(super) height: usize,
	pub(super) px: Vec<u8>,
}

impl Canvas {
	pub(super) fn new(width: usize, height: usize) -> Self {
		Self { width, height, px: vec![0; width * height * 4] }
	}

	/// Black, opaque: the ground before the layers.
	pub(super) fn clear(&mut self) {
		for p in self.px.as_chunks_mut::<4>().0 {
			*p = [0, 0, 0, 255];
		}
	}
}

/// A layer the page drew: its pixels (BGRA, straight alpha) at its place (the frames it shows on and whether it lies
/// over the knight are its order's, `desk::reel`). `opaque` covers the whole frame with no clear pixel (the ground),
/// drawn as a copy.
pub(super) struct Layer {
	x: usize,
	y: usize,
	width: usize,
	height: usize,
	px: Vec<u8>,
	opaque: bool,
}

impl Layer {
	/// A layer from its PNG, placed at (`x`, `y`) on a `canvas_w` × `canvas_h` frame, which it must not leave.
	///
	/// # Errors
	/// The PNG does not decode, or it reaches past the frame.
	pub(super) fn read(png: &[u8], left: usize, top: usize, canvas_w: usize, canvas_h: usize) -> anyhow::Result<Self> {
		let mut decoder = png::Decoder::new(Cursor::new(png));
		decoder.set_transformations(png::Transformations::normalize_to_color8());
		let mut reader = decoder.read_info().context("a layer's PNG")?;
		let size = reader.output_buffer_size().context("a layer's PNG is too large")?;
		let mut raw = vec![0; size];
		let info = reader.next_frame(&mut raw).context("a layer's PNG")?;
		let (width, height) = (usize::try_from(info.width)?, usize::try_from(info.height)?);
		let inside = |at: usize, size: usize, side: usize| at.checked_add(size).is_some_and(|reach| reach <= side);
		if !inside(left, width, canvas_w) || !inside(top, height, canvas_h) || width == 0 || height == 0 {
			bail!("a layer of {width} × {height} at {left}, {top} leaves the {canvas_w} × {canvas_h} frame");
		}
		let channels = match info.color_type {
			png::ColorType::Grayscale => 1,
			png::ColorType::GrayscaleAlpha => 2,
			png::ColorType::Rgb => 3,
			png::ColorType::Rgba => 4,
			png::ColorType::Indexed => bail!("a layer's PNG kept its palette"),
		};
		let mut px = Vec::with_capacity(width * height * 4);
		for row in raw.chunks_exact(info.line_size).take(height) {
			for p in row.chunks_exact(channels).take(width) {
				let bgra = match *p {
					[grey] => [grey, grey, grey, 255],
					[grey, alpha] => [grey, grey, grey, alpha],
					[red, green, blue] => [blue, green, red, 255],
					[red, green, blue, alpha] => [blue, green, red, alpha],
					_ => [0; 4],
				};
				px.extend_from_slice(&bgra);
			}
		}
		let whole = left == 0 && top == 0 && width == canvas_w && height == canvas_h;
		let opaque = whole && px.as_chunks::<4>().0.iter().all(|p| p[3] == 255);
		let layer = Self { x: left, y: top, width, height, px, opaque };
		Ok(if opaque { layer } else { layer.trimmed() })
	}

	/// The layer cut to the box of its pixels that show (a page's layer is drawn on a canvas of the whole frame, its
	/// words in a corner of it), so a frame blends only those; a layer with none shows nothing.
	fn trimmed(self) -> Self {
		let shows = |x: usize, y: usize| self.px[(y * self.width + x) * 4 + 3] != 0;
		let rows: Vec<usize> = (0..self.height).filter(|&y| (0..self.width).any(|x| shows(x, y))).collect();
		let (Some(&top), Some(&bottom)) = (rows.first(), rows.last()) else {
			return Self { width: 0, height: 0, px: Vec::new(), ..self };
		};
		let columns = |x: usize| (top..=bottom).any(|y| shows(x, y));
		let left = (0..self.width).find(|&x| columns(x)).unwrap_or(0);
		let right = (0..self.width).rev().find(|&x| columns(x)).unwrap_or(self.width - 1);
		let (width, height) = (right + 1 - left, bottom + 1 - top);
		let mut px = Vec::with_capacity(width * height * 4);
		for y in top..=bottom {
			let at = (y * self.width + left) * 4;
			px.extend_from_slice(&self.px[at..at + width * 4]);
		}
		Self { x: self.x + left, y: self.y + top, width, height, px, ..self }
	}

	/// Draws it over the frame: a copy where it is opaque and whole, else source-over with its straight alpha.
	pub(super) fn draw(&self, canvas: &mut Canvas) {
		if self.opaque {
			canvas.px.copy_from_slice(&self.px);
			return;
		}
		for row in 0..self.height {
			let src = &self.px[row * self.width * 4..(row + 1) * self.width * 4];
			let at = ((self.y + row) * canvas.width + self.x) * 4;
			let dst = &mut canvas.px[at..at + self.width * 4];
			for (d, s) in dst.as_chunks_mut::<4>().0.iter_mut().zip(src.as_chunks::<4>().0) {
				let a = u32::from(s[3]);
				if a == 0 {
					continue;
				}
				for (dc, sc) in d.iter_mut().zip(s).take(3) {
					*dc = mix(*dc, *sc, a);
				}
			}
		}
	}
}

/// `dst` toward `src` by `a` of 255, rounded.
fn mix(dst: u8, src: u8, a: u32) -> u8 {
	let v = (u32::from(src) * a + u32::from(dst) * (255 - a) + 127) / 255;
	u8::try_from(v).unwrap_or(u8::MAX)
}

/// How the knight stands on the card: cut out through a round mask on Night (opaque to `mask` of its radius), or in
/// the rounded well of `radius` pixels on Parchment and Gilded; either way lightened over what lies below.
#[derive(Debug, Clone, Copy)]
pub(super) enum Look {
	Night { mask: f64 },
	Well { radius: f64 },
}

/// The knight's square on the frame: its corner and side in pixels, and its look.
#[derive(Debug, Clone, Copy)]
pub(super) struct Knight {
	pub(super) x: usize,
	pub(super) y: usize,
	pub(super) size: usize,
	pub(super) look: Look,
}

/// A film's frame as the knight shows it: its tile's square, tinted, `side` pixels a side, BGRA, opaque.
pub(super) struct Square {
	side: usize,
	px: Vec<u8>,
}

/// Where a film's frame keeps its parts (figures.json `film`): the square's side, the tint's first row, the tile
/// (shares of the square: x, y, side) and the colour the tint is added in.
#[derive(Debug, Clone, Copy)]
pub(super) struct FilmCut {
	pub(super) side: usize,
	pub(super) tint: usize,
	pub(super) tile: [f64; 3],
	pub(super) colour: [u8; 3],
}

impl FilmCut {
	/// The tile's square of a decoded film frame (BGRA rows `stride` bytes apart, `width` × `height`), its tint added:
	/// none where the frame is smaller than its layout says.
	pub(super) fn square(&self, frame: &[u8], width: usize, height: usize, stride: usize) -> Option<Square> {
		let side = self.side;
		if width < side || height < self.tint + side {
			return None;
		}
		let at = |share: f64| -> usize {
			#[expect(
				clippy::cast_possible_truncation,
				clippy::cast_sign_loss,
				clippy::cast_precision_loss,
				// copy-lint: not shown (a lint's reason)
				reason = "a share of a side of a few hundred pixels, rounded"
			)]
			let px = (share.clamp(0.0, 1.0) * side as f64).round() as usize;
			px.min(side)
		};
		let (x0, y0) = (at(self.tile[0]), at(self.tile[1]));
		let cut = at(self.tile[2]).min(side - x0).min(side - y0);
		if cut == 0 {
			return None;
		}
		// the colour in the frame's channel order
		let [red, green, blue] = self.colour.map(u32::from);
		let tone = [blue, green, red];
		let mut px = Vec::with_capacity(cut * cut * 4);
		for row in 0..cut {
			let base_at = (y0 + row) * stride + x0 * 4;
			let tint_at = (self.tint + y0 + row) * stride + x0 * 4;
			let base = frame.get(base_at..base_at + cut * 4)?;
			let tint = frame.get(tint_at..tint_at + cut * 4)?;
			for (p, light) in base.as_chunks::<4>().0.iter().zip(tint.as_chunks::<4>().0) {
				for ((channel, lit), colour) in p.iter().zip(light).zip(tone) {
					let sum = u32::from(*channel) + (u32::from(*lit) * colour + 127) / 255;
					px.push(u8::try_from(sum.min(255)).unwrap_or(u8::MAX));
				}
				px.push(255);
			}
		}
		Some(Square { side: cut, px })
	}
}

/// Draws the knight's square on the frame at its place, mirrored where `mirror`, scaled to its size (bilinear), in its
/// look.
pub(super) fn knight(canvas: &mut Canvas, square: &Square, mirror: bool, k: Knight) {
	if k.size == 0 || square.side == 0 {
		return;
	}
	#[expect(clippy::cast_precision_loss, reason = "pixel counts far below 2^52")]
	let (size, scale) = (k.size as f64, square.side as f64 / k.size as f64);
	for y in 0..k.size {
		let cy = k.y + y;
		if cy >= canvas.height {
			break;
		}
		for x in 0..k.size {
			let cx = k.x + x;
			if cx >= canvas.width {
				break;
			}
			#[expect(clippy::cast_precision_loss, reason = "pixel counts far below 2^52")]
			let (fx, fy) = (x as f64 + 0.5, y as f64 + 0.5);
			let cover = coverage(k.look, fx, fy, size);
			if cover <= 0.0 {
				continue;
			}
			let sx = if mirror { size - fx } else { fx } * scale - 0.5;
			let sy = fy * scale - 0.5;
			let src = sample(square, sx, sy);
			let at = (cy * canvas.width + cx) * 4;
			for (dst, s) in canvas.px[at..at + 3].iter_mut().zip(src) {
				let d = f64::from(*dst);
				let v = d + (s - d).max(0.0) * cover;
				#[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "a channel within 0-255")]
				let byte = v.round().clamp(0.0, 255.0) as u8;
				*dst = byte;
			}
		}
	}
}

/// How much of the knight shows at a pixel's middle (`x`, `y` from its square's corner, `size` its side): Night's
/// round mask, whole to `mask` of the radius and fading straight to clear at it; the well's rounded corners, a pixel
/// wide at their edge.
fn coverage(look: Look, x: f64, y: f64, size: f64) -> f64 {
	match look {
		Look::Night { mask } => {
			let radius = size / 2.0;
			let r = (x - radius).hypot(y - radius) / radius;
			if r <= mask {
				1.0
			} else if r >= 1.0 || mask >= 1.0 {
				0.0
			} else {
				(1.0 - r) / (1.0 - mask)
			}
		}
		Look::Well { radius } => {
			let radius = radius.clamp(0.0, size / 2.0);
			let cx = x.clamp(radius, size - radius);
			let cy = y.clamp(radius, size - radius);
			let outside = (x - cx).hypot(y - cy) - radius;
			(0.5 - outside).clamp(0.0, 1.0)
		}
	}
}

/// The square's colour at (`x`, `y`) in its pixels, between its four nearest, the edges held.
fn sample(square: &Square, x: f64, y: f64) -> [f64; 3] {
	#[expect(clippy::cast_precision_loss, reason = "pixel counts far below 2^52")]
	let last = (square.side - 1) as f64;
	let (x, y) = (x.clamp(0.0, last), y.clamp(0.0, last));
	let (x0, y0) = (x.floor(), y.floor());
	let (tx, ty) = (x - x0, y - y0);
	#[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "a clamped pixel index")]
	let (x0, y0) = (x0 as usize, y0 as usize);
	let (x1, y1) = ((x0 + 1).min(square.side - 1), (y0 + 1).min(square.side - 1));
	let at = |px: usize, py: usize, c: usize| f64::from(square.px[(py * square.side + px) * 4 + c]);
	let mut out = [0.0; 3];
	for (c, v) in out.iter_mut().enumerate() {
		let top = at(x0, y0, c) * (1.0 - tx) + at(x1, y0, c) * tx;
		let bottom = at(x0, y1, c) * (1.0 - tx) + at(x1, y1, c) * tx;
		*v = top * (1.0 - ty) + bottom * ty;
	}
	out
}
