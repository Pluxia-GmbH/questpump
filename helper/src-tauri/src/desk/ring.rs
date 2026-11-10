//! The tray mark with work mode's ring (docs/plan.md slice 67; docs/ui.md, "The tray"): the mark's own picture (the
//! tone's, `tray.rs`) drawn smaller in the middle of its canvas, and a ring around it in the mark's own colour, its
//! track faint and its arc, from the top clockwise, the share of the block counted (full once a pump is due). Drawn
//! at run time from the pictures the tray already ships, so no picture joins the bundle, and drawn again only when the
//! share's whole minute or the mark's colour changes.

use std::f64::consts::TAU;

use tauri::image::Image;

/// The mark's size inside the ring, a share of the canvas: its widest part clears the ring's inner edge.
const INNER: f64 = 0.6;
/// The ring's thickness, a share of the canvas (4 px of the 44-px menu bar picture, 3 of the 32-px tray one).
const THICK: f64 = 0.09;
/// The track's opacity, a share of the arc's.
const TRACK: f64 = 0.3;

/// One pixel in premultiplied light (red, green, blue and alpha, 0 to 1).
type Light = [f64; 4];

/// The mark `mark` (RGBA) with a ring whose arc covers `fill` (0 to 1) of it.
pub(crate) fn ringed(mark: &Image<'_>, fill: f64) -> Image<'static> {
	let (w, h) = (mark.width(), mark.height());
	let rgba = mark.rgba();
	let colour = brightest(rgba);
	let (cx, cy) = (f64::from(w) / 2.0, f64::from(h) / 2.0);
	let side = f64::from(w.min(h));
	let thick = side * THICK;
	let mid = side / 2.0 - thick / 2.0 - 0.5;
	let fill = fill.clamp(0.0, 1.0);
	let mut out = Vec::with_capacity(rgba.len());
	for y in 0..h {
		for x in 0..w {
			let (px, py) = (f64::from(x) + 0.5 - cx, f64::from(y) + 0.5 - cy);
			// the mark, its footprint in the picture as large as the pixel over the mark's scale
			let at = |v: f64, centre: f64| (v - 0.5) / INNER + centre;
			let shrunk = sample(rgba, w, h, [at(px, cx), at(py, cy), at(px + 1.0, cx), at(py + 1.0, cy)]);
			let distance = px.hypot(py);
			let cover = (thick / 2.0 + 0.5 - (distance - mid).abs()).clamp(0.0, 1.0);
			let turn = px.atan2(-py).rem_euclid(TAU) / TAU;
			let alpha = cover * if turn < fill { 1.0 } else { TRACK };
			let ring = colour.map(|c| c * alpha);
			let a = shrunk[3];
			let light: Light =
				std::array::from_fn(|i| if i == 3 { a + alpha * (1.0 - a) } else { shrunk[i] + ring[i] * (1.0 - a) });
			out.extend_from_slice(&straight(light));
		}
	}
	Image::new_owned(out, w, h)
}

/// The mark's own colour: its brightest opaque pixel's, as straight red, green and blue (0 to 1) with an alpha of 1.
fn brightest(rgba: &[u8]) -> Light {
	let best = rgba
		.as_chunks::<4>()
		.0
		.iter()
		.filter(|p| p[3] > 200)
		.max_by_key(|p| p.iter().take(3).map(|&c| u32::from(c)).sum::<u32>());
	best.map_or([1.0; 4], |&[r, g, b, _]| [f64::from(r) / 255.0, f64::from(g) / 255.0, f64::from(b) / 255.0, 1.0])
}

/// The picture's light over the box `[left, top, right, bottom]` of its own pixels, averaged by each pixel's share of
/// it; outside the picture is clear.
fn sample(rgba: &[u8], width: u32, height: u32, [left, top, right, bottom]: [f64; 4]) -> Light {
	let mut sum = [0.0; 4];
	let area = (right - left) * (bottom - top);
	if area <= 0.0 {
		return sum;
	}
	let span = |from: f64, to: f64, size: u32| {
		let first = from.floor().max(0.0);
		let last = to.ceil().min(f64::from(size));
		#[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "clamped into the picture")]
		(first as u32..last.max(first) as u32)
			.map(move |i| (i, (to.min(f64::from(i) + 1.0) - from.max(f64::from(i))).max(0.0)))
	};
	let pixels = rgba.as_chunks::<4>().0;
	for (row, share_y) in span(top, bottom, height) {
		for (column, share_x) in span(left, right, width) {
			let at = usize::try_from(u64::from(row) * u64::from(width) + u64::from(column)).unwrap_or(usize::MAX);
			let Some(&[red, green, blue, opacity]) = pixels.get(at) else { continue };
			let alpha = f64::from(opacity) / 255.0;
			let weight = share_x * share_y;
			for (total, channel) in sum.iter_mut().zip([red, green, blue]) {
				*total += f64::from(channel) / 255.0 * alpha * weight;
			}
			sum[3] += alpha * weight;
		}
	}
	sum.map(|s| s / area)
}

/// Premultiplied light as straight RGBA bytes.
fn straight(light: Light) -> [u8; 4] {
	let a = light[3].clamp(0.0, 1.0);
	#[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "a channel clamped into 0..=255")]
	let byte = |v: f64| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
	if a <= 0.0 {
		return [0; 4];
	}
	[byte(light[0] / a), byte(light[1] / a), byte(light[2] / a), byte(a)]
}
