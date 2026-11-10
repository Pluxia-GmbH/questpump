//! The QuestPump helper's rules, with no platform code (rust-helper `SKILL.md`, rule 2): the pixel strip's decoder, the
//! row store, the app's rows and the sheet log, the session that turns frames into rows, device commands and mirror
//! writes, the `QuestPump_Mirror` writer and its reader, the socket protocol, the pair link, the link's seal and command
//! queue, the Bluetooth link's envelopes and chunks, the body every app device shares, the heart-rate verdict, each
//! pump's proof, the Bluetooth parsers (the strap's heart rate, the walking pad's treadmill data and control point), the
//! feedback button's message, the desktop app's crash reports with their opt-in regions, its anonymous counts with the
//! catalog they keep to, the outbox all three go out through, the games on the computer (Steam's key-values files and
//! its record of each app's type, the games' state machine, Valve's Game State Integration, Psyonix's Stats API, the
//! fed games' feeds, the detector's deny lists, the offer after a match and the sitting clock that warms its pump), the
//! check of a phone's Pro proof with the grants it gives, the backup file the desktop app writes and reads, the
//! sync's keys, codes and batches with the other computers' game rows it brings (slice 60), and the addon's signed
//! release feed with its zip, its install record and the folders' swap (the zip read by `zip`), and the places (slice
//! 51) and work mode's shared state and clock (slice 67). The app crate is the shell around it.

#![forbid(unsafe_code)]

/// The nearest-rank percentile `p` (0-100) of sorted values: the link's delivery statistics and the verdict's
/// calibration.
pub(crate) fn percentile<T: Copy>(sorted: &[T], p: usize) -> Option<T> {
	if sorted.is_empty() {
		return None;
	}
	let rank = (p * sorted.len()).div_ceil(100).max(1);
	sorted.get(rank - 1).copied()
}

pub mod addon;
pub mod app_rows;
pub mod appinfo;
pub mod backup;
pub mod ble;
pub mod blob;
pub mod body;
pub mod codes;
pub mod crash;
pub mod detect;
pub mod far;
pub mod feedback;
pub mod feeds;
pub mod games;
pub mod gatt;
pub mod gsi;
mod jsonl;
mod library;
pub mod link;
pub mod lua;
pub mod mirror;
pub mod moves;
pub mod offers;
pub mod outbox;
pub mod pair;
pub mod places;
pub mod pro;
pub mod proof;
pub mod protocol;
pub mod rocket;
pub mod row;
pub mod seal;
pub mod session;
pub mod sheets;
pub mod sitting;
pub mod store;
pub mod strip;
pub mod sync;
pub mod telemetry;
#[rustfmt::skip]
pub mod telemetry_catalog;
pub mod vdf;
pub mod verdict;
pub mod work;
pub mod zip;
