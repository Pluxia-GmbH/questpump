//! A QR code's modules as the pages draw them in the kit's `QrWell` (ink on the light well, its quiet zone around): the
//! desktop app's pair link (slices 5 and 35) and the sync's link and request codes (slice 60), on the computer and in
//! the browser.

use serde::Serialize;

/// A QR code's modules for the page: `width` modules a side, `modules` row by row from the top, `1` dark and `0`
/// light, without the quiet zone (the `QrWell` draws four modules of it).
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Qr {
	pub width: usize,
	pub modules: String,
}

/// A QR code of `text` as the pages draw it (the pair link's; the sync's link and request codes, slice 60); none for
/// text past a QR code's room.
#[must_use]
pub fn qr(text: &str) -> Option<Qr> {
	qrcode::QrCode::new(text.as_bytes()).ok().map(|code| Qr {
		width: code.width(),
		modules: code.to_colors().iter().map(|c| if *c == qrcode::Color::Dark { '1' } else { '0' }).collect(),
	})
}
