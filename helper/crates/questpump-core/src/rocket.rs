//! Psyonix's Stats API for Rocket League (docs/plan.md slice 35; rocketleague.com/en/developer/stats-api, read
//! 2026-10-08): the game, once `TAStatsAPI.ini` in its `TAGame/Config` folder (or, without it, `DefaultStatsAPI.ini`
//! there) turns the feed on before its start, listens on a TCP port of the loopback and sends every connected program
//! its events as JSON envelopes, `{"Event": …, "Data": …}`. The helper reads the envelope's `Event` alone (`Data`,
//! with every player's name and platform id, is never read, kept or logged) and never writes to the socket, whose
//! commands would steer the game. The page names no delimiter between envelopes, so `Frames` cuts the stream at each
//! top-level JSON object's end.

use serde::Deserialize;

/// The file that turns the feed on, in `<install>/TAGame/Config`.
pub const INI: &str = "TAStatsAPI.ini";
/// The game's own file beside it, which the game reads when `INI` is missing (Psyonix: "or DefaultStatsAPI.ini if it
/// doesn't exist"), in the same section.
pub const DEFAULT_INI: &str = "DefaultStatsAPI.ini";
/// The file's section for the feed.
const SECTION: &str = "TAGame.MatchStatsExporter_TA";
/// The game's default port, which the helper's own file names.
pub const PORT: u16 = 49_123;
/// The longest envelope kept: an `UpdateState` of eight players is a few kilobytes; a longer one is skipped whole.
const FRAME_MAX: usize = 1 << 20;

/// The text of the helper's own `TAStatsAPI.ini`: one `UpdateState` a second (`PacketSendRate`, whose default 0
/// "disables the feature entirely", events included) on `port`, in the line endings of the game's own ini files.
#[must_use]
pub fn ini(port: u16) -> String {
	format!("[{SECTION}]\r\nPacketSendRate=1\r\nPort={port}\r\n")
}

/// The port a player's own `TAStatsAPI.ini` (or `DefaultStatsAPI.ini`) opens the feed on (its section's `Port`, the
/// default when it names none); none when the file turns the feed off (`PacketSendRate` missing or 0, or `Port` 0).
/// Keys and the section are read in any case, as the game's ini reader does.
#[must_use]
pub fn ini_port(text: &str) -> Option<u16> {
	let (mut rate, mut port, mut inside) = (None::<f64>, None::<u16>, false);
	for line in text.trim_start_matches('\u{feff}').lines() {
		let line = line.trim();
		if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
			inside = name.trim().eq_ignore_ascii_case(SECTION);
			continue;
		}
		let Some((key, value)) = line.split_once('=').filter(|_| inside) else { continue };
		let (key, value) = (key.trim(), value.trim());
		if key.eq_ignore_ascii_case("PacketSendRate") {
			rate = value.parse().ok();
		} else if key.eq_ignore_ascii_case("Port") {
			port = value.parse().ok();
		}
	}
	let port = port.unwrap_or(PORT);
	(rate.is_some_and(|r| r > 0.0) && port != 0).then_some(port)
}

/// The envelope's only field read.
#[derive(Deserialize)]
struct Envelope {
	#[serde(rename = "Event")]
	event: String,
}

/// The stream cut into envelopes: bytes go in as they arrive, each complete top-level object's `Event` comes out. A
/// string's braces are no structure; bytes between objects (whitespace, a separator) are skipped; an object that does
/// not read as an envelope gives nothing.
#[derive(Default)]
pub struct Frames {
	buf: Vec<u8>,
	depth: u32,
	string: bool,
	escape: bool,
	/// The object now arriving outgrew `FRAME_MAX`: its bytes are dropped until it ends.
	skipping: bool,
}

impl Frames {
	/// The events of the objects these bytes complete, in order.
	pub fn push(&mut self, bytes: &[u8]) -> Vec<String> {
		let mut events = Vec::new();
		for &b in bytes {
			if self.depth == 0 {
				if b == b'{' {
					self.depth = 1;
					self.buf.clear();
					self.buf.push(b);
					self.skipping = false;
				}
				continue;
			}
			if !self.skipping {
				self.buf.push(b);
				if self.buf.len() > FRAME_MAX {
					self.skipping = true;
					self.buf = Vec::new();
				}
			}
			if self.string {
				if self.escape {
					self.escape = false;
				} else if b == b'\\' {
					self.escape = true;
				} else if b == b'"' {
					self.string = false;
				}
				continue;
			}
			match b {
				b'"' => self.string = true,
				b'{' => self.depth += 1,
				b'}' => {
					self.depth -= 1;
					if self.depth == 0 && !self.skipping {
						if let Ok(envelope) = serde_json::from_slice::<Envelope>(&self.buf) {
							events.push(envelope.event);
						}
						self.buf.clear();
					}
				}
				_ => {}
			}
		}
		events
	}
}
