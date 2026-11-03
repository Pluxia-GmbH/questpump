//! The pair link the helper's window shows as a QR code (docs/plan.md slice 5; rust-helper `protocol-and-mirror.md`,
//! "The pair link"): `https://questpump.com/pair#v=1&host=<lan ip>&port=<port>&key=<base64url>&name=<computer>`, then
//! `&mdns=<name>.local&hid=<base64url>` when the helper knows them (older helpers write neither). The payload rides in
//! the URL fragment, which a browser never sends to the server; the phone app reads it, refuses any field out of
//! bounds, and pairs with the helper on the LAN. `contract/fixtures/pair.json` holds the cases every app replays.

use std::fmt::Write as _;
use std::net::IpAddr;

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;

pub const BASE: &str = "https://questpump.com/pair#";
pub const VERSION: u32 = 1;
/// The helper's LAN port (slice 7 listens on it).
pub const PORT: u16 = 8787;

/// What the pair link carries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PairLink {
	pub host: IpAddr,
	pub port: u16,
	/// 32 random bytes, in the QR code whenever the helper's window shows it (a new code ends every pairing); the bearer
	/// and the seal key are derived from it, and it never travels itself.
	pub key: [u8; 32],
	/// The computer's name, for the phone to show.
	pub name: String,
	/// The computer's own `.local` host name ([`is_local_name`]), for the phone to find the helper by when the address
	/// moves.
	pub mdns: Option<String>,
	/// The helper install's id, 16 random bytes made once and kept across new pairing codes, so the phone tells its own
	/// helper from another one.
	pub hid: Option<[u8; 16]>,
}

/// Why a pair link is refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PairError {
	#[error("not a QuestPump pair link")]
	NotALink,
	#[error("a version this app does not read")]
	Version,
	#[error("a host outside the home network's private ranges")]
	Host,
	#[error("a port outside 1024-65535")]
	Port,
	#[error("a key that is not 32 bytes of base64url")]
	Key,
	#[error("no computer name")]
	Name,
	#[error("a host name that is not a .local name")]
	Mdns,
	#[error("a helper id that is not 16 bytes of base64url")]
	Hid,
}

/// Whether a name is a `.local` host name as the pair link carries it: at most 253 characters, ending in `.local`,
/// every label 1-63 letters, digits and hyphens that neither starts nor ends with a hyphen.
#[must_use]
#[expect(clippy::case_sensitive_file_extension_comparisons, reason = "a host name, not a file extension")]
pub fn is_local_name(name: &str) -> bool {
	name.len() <= 253
		&& name.ends_with(".local")
		&& name.split('.').all(|label| {
			(1..=63).contains(&label.len())
				&& !label.starts_with('-')
				&& !label.ends_with('-')
				&& label.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
		})
}

/// Whether an address lies in a home network's private ranges: RFC 1918, IPv4 link-local, IPv6 unique local
/// (`fc00::/7`) and link-local (`fe80::/10`).
#[must_use]
pub fn is_private(ip: &IpAddr) -> bool {
	match ip {
		IpAddr::V4(v4) => v4.is_private() || v4.is_link_local(),
		IpAddr::V6(v6) => {
			let first = v6.segments()[0];
			(first & 0xfe00) == 0xfc00 || (first & 0xffc0) == 0xfe80
		}
	}
}

fn percent(s: &str) -> String {
	let mut out = String::new();
	for b in s.bytes() {
		if b.is_ascii_alphanumeric() || b"-._~".contains(&b) {
			out.push(char::from(b));
		} else {
			let _ = write!(out, "%{b:02X}");
		}
	}
	out
}

fn unpercent(s: &str) -> Option<String> {
	let mut out = Vec::new();
	let mut bytes = s.bytes();
	while let Some(b) = bytes.next() {
		if b == b'%' {
			let hi = char::from(bytes.next()?).to_digit(16)?;
			let lo = char::from(bytes.next()?).to_digit(16)?;
			out.push(u8::try_from(hi * 16 + lo).ok()?);
		} else {
			out.push(b);
		}
	}
	String::from_utf8(out).ok()
}

/// Exactly `N` bytes in base64url without padding.
fn bytes<const N: usize>(text: &str) -> Option<[u8; N]> {
	URL_SAFE_NO_PAD.decode(text).ok()?.try_into().ok()
}

impl PairLink {
	/// The link's text: the fields in their order, `mdns` and `hid` only when known.
	#[must_use]
	pub fn to_url(&self) -> String {
		let mut url = format!(
			"{BASE}v={VERSION}&host={}&port={}&key={}&name={}",
			percent(&self.host.to_string()),
			self.port,
			URL_SAFE_NO_PAD.encode(self.key),
			percent(&self.name)
		);
		if let Some(mdns) = &self.mdns {
			let _ = write!(url, "&mdns={}", percent(mdns));
		}
		if let Some(hid) = &self.hid {
			let _ = write!(url, "&hid={}", URL_SAFE_NO_PAD.encode(hid));
		}
		url
	}

	/// Reads a pair link and checks every field.
	///
	/// # Errors
	/// The first field out of bounds.
	pub fn parse(url: &str) -> Result<Self, PairError> {
		let fragment = url.strip_prefix(BASE).ok_or(PairError::NotALink)?;
		let mut v = None;
		let (mut host, mut port, mut key, mut name, mut mdns, mut hid) = (None, None, None, None, None, None);
		for pair in fragment.split('&') {
			let (k, val) = pair.split_once('=').ok_or(PairError::NotALink)?;
			let val = unpercent(val).ok_or(PairError::NotALink)?;
			match k {
				"v" => v = Some(val),
				"host" => host = Some(val),
				"port" => port = Some(val),
				"key" => key = Some(val),
				"name" => name = Some(val),
				"mdns" => mdns = Some(val),
				"hid" => hid = Some(val),
				_ => {} // a field a later version adds
			}
		}
		if v.as_deref() != Some("1") {
			return Err(PairError::Version);
		}
		let host: IpAddr = host.and_then(|h| h.parse().ok()).ok_or(PairError::Host)?;
		if !is_private(&host) {
			return Err(PairError::Host);
		}
		let port: u16 = port.and_then(|p| p.parse().ok()).filter(|&p| p >= 1024).ok_or(PairError::Port)?;
		let key = key.as_deref().and_then(bytes).ok_or(PairError::Key)?;
		let name = name.filter(|n| !n.trim().is_empty()).ok_or(PairError::Name)?;
		// absent is fine (an older helper's link); present and out of bounds refuses the link
		let mdns = mdns.map(|m| is_local_name(&m).then_some(m).ok_or(PairError::Mdns)).transpose()?;
		let hid = hid.map(|h| bytes(&h).ok_or(PairError::Hid)).transpose()?;
		Ok(Self { host, port, key, name, mdns, hid })
	}
}
