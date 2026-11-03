//! The Bluetooth link (docs/plan.md slice 43, with its fixes after the review of 2026-10-17;
//! `contract/companion-api.yaml`, "The Bluetooth link"; rust-helper `bluetooth.md`, "The Bluetooth link's peripheral"): a
//! second transport for the LAN's routes, the computer a GATT peripheral and the phones centrals. Each call is the HTTP
//! request the phone would send, sealed again whole: one envelope, AES-256-GCM under the ble key (HKDF-SHA256 of the
//! pairing key, salt empty, info `questpump ble v1`), its associated data the direction's label, the message's id and
//! the connection's challenge; a request's plain text its method, path, agent and body, a response's its status and
//! body. An envelope travels in chunks, each a 4-byte header (the id, then the chunk's index in 3 bytes, big-endian)
//! and its part, the first part starting with the envelope's length; a receiver puts them together in a `Reassembly`
//! within the limits. The service's UUIDs, the hello the `hello` characteristic reads (made for the central that reads
//! it: an hourly proof of the pairing key over the connection's challenge, and the challenge) and the advertisement
//! live here too.
//! No clock, socket or radio: the app passes the time and the challenge in and drives the platform's peripheral.
//! `contract/fixtures/ble.json` pins every value and case here; the phones' cores (`BleFrame`) replay the same file.

use aes_gcm::aead::{Aead, Generate, KeyInit, Payload};
use aes_gcm::{Aes256Gcm, Key, aead};
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde::Serialize;

use crate::seal::{NONCE, SealError, TAG};

/// The HKDF info of the ble key, the pairing key's fourth derivation.
pub const INFO: &[u8] = b"questpump ble v1";
/// The service, drawn once (2026-10-16) and never changed.
pub const SERVICE: u128 = 0xdbd1_d980_bd49_46d6_a353_d4d1_ea3a_6672;
/// The plain hello a phone reads before anything sealed, made for the central that reads it: `{v, k, c}` (`hello`).
pub const HELLO: u128 = 0xdbd1_d981_bd49_46d6_a353_d4d1_ea3a_6672;
/// Phone to helper: write without response, or write.
pub const RX: u128 = 0xdbd1_d982_bd49_46d6_a353_d4d1_ea3a_6672;
/// Helper to phone: notify only, each message to the central that asked for it alone.
pub const TX: u128 = 0xdbd1_d983_bd49_46d6_a353_d4d1_ea3a_6672;
/// The name a Mac advertises (`CBAdvertisementDataLocalNameKey`).
pub const LOCAL_NAME: &str = "QuestPump";
/// The service data a Windows PC advertises on the service's UUID: "QP" and the layout's version (a Windows peripheral
/// advertises no local name of its own); no id of the computer, so a phone's standing scan filter matches every hour.
pub const SERVICE_DATA: [u8; 3] = [0x51, 0x50, VERSION];
/// The Bluetooth layout's version, the hello's `v`: 3 since the release security review of 2026-10-31 (L3); 2 keyed its
/// hashes with the install's id, which the pair link and every answer carry, so anyone who once saw it could follow
/// the computer; 1 carried the install's id and the key's id in plain.
pub const VERSION: u8 = 3;
/// The connection's challenge: 16 random bytes the helper draws for a central at its read of the hello from the start,
/// which every envelope on that connection carries in its associated data.
pub const CHALLENGE_BYTES: usize = 16;
/// A connection's challenge.
pub type Challenge = [u8; CHALLENGE_BYTES];
/// The hello's proof is of the UTC hour: the epoch is floor(unix seconds / this).
pub const EPOCH_S: i64 = 3_600;
/// The label of the hello's `k`.
const HELLO_KEY: &[u8] = b"questpump ble hello key";

/// A chunk's header: the message's id and the chunk's index.
pub const HEADER_BYTES: usize = 4;
/// The envelope's length at the start of the first chunk's part.
pub const LENGTH_BYTES: usize = 4;
/// The least an envelope may be: a nonce and a tag.
pub const ENVELOPE_MIN_BYTES: usize = NONCE + TAG;
/// The most an envelope toward the helper may be (288 KiB: a LAN body of at most 256 KiB, its head and the seal).
pub const REQUEST_MAX_BYTES: usize = 288 * 1024;
/// The most an envelope toward the phone may be (2 MiB): the helper answers 500 past it.
pub const RESPONSE_MAX_BYTES: usize = 2 * 1024 * 1024;
/// A message whose next chunk comes more than this after the one before goes (exactly this still counts).
pub const CHUNK_GAP_MS: i64 = 10_000;
/// The messages a receiver puts together at once (a whole message in its first chunk takes no place among them), the
/// calls one central has at the helper, each from its envelope's opening until its answer's last chunk went to the radio
/// (a request that opens past them is answered `503`), and a phone's own calls.
pub const IN_FLIGHT_MAX: usize = 4;
/// An ATT value's most: a chunk is at most min(ATT MTU - 3, this) bytes.
pub const ATT_VALUE_MAX: usize = 512;
/// The most bytes of a request's path with its query.
pub const PATH_MAX_BYTES: usize = 1024;
/// The most bytes of a request's agent.
pub const AGENT_MAX_BYTES: usize = 255;
/// The least ATT MTU (Bluetooth Core, Vol 3 Part F 3.2.8: LE's `ATT_MTU` default), so the least a chunk carries.
const ATT_MTU_MIN: usize = 23;

/// The ble key of a pairing key (`Keys::ble` holds its cipher).
#[must_use]
pub fn key(pairing: &[u8; 32]) -> [u8; 32] {
	crate::seal::derive(pairing, INFO)
}

/// A UUID as its text, lower case, 8-4-4-4-12 (`CBUUID(string:)`, the fixture's).
#[must_use]
pub fn uuid_text(uuid: u128) -> String {
	let h = format!("{uuid:032x}");
	format!("{}-{}-{}-{}-{}", &h[..8], &h[8..12], &h[12..16], &h[16..20], &h[20..])
}

/// The hello characteristic's value: the Bluetooth layout's version, the hour's proof of the pairing key over the
/// connection's challenge and the challenge, as compact JSON in that order (54 bytes). A phone judges it against its
/// pairing at the epoch before, its own and the one after; nothing in it names the computer, and no two hellos link.
#[derive(Serialize)]
struct Hello {
	v: u8,
	k: String,
	c: String,
}

/// The epoch of a unix second: its UTC hour.
#[must_use]
pub fn epoch(unix_s: i64) -> i64 {
	unix_s.div_euclid(EPOCH_S)
}

/// The hello's bytes for the current pairing key's hello key (`seal::hello_key`), the helper's clock (unix seconds)
/// and the challenge drawn for the central that reads it: `k` the proof under the hello key over `questpump ble hello
/// key`, the hour's epoch (8 bytes, big-endian) and the challenge, `c` the challenge, each base64url without padding.
#[must_use]
pub fn hello(hello_key: &[u8; 32], unix_s: i64, challenge: &Challenge) -> Vec<u8> {
	let k = crate::seal::proof(hello_key, &[HELLO_KEY, &epoch(unix_s).to_be_bytes(), challenge]);
	let text = |bytes: &[u8]| URL_SAFE_NO_PAD.encode(bytes);
	serde_json::to_vec(&Hello { v: VERSION, k: text(&k), c: text(challenge) }).unwrap_or_default()
}

/// Which way a message goes: its associated data's label.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
	/// Phone to helper, on `rx`.
	Rx,
	/// Helper to phone, on `tx`.
	Tx,
}

impl Direction {
	fn label(self) -> &'static [u8] {
		match self {
			Self::Rx => b"questpump ble v1 rx",
			Self::Tx => b"questpump ble v1 tx",
		}
	}
}

/// A message's associated data: its direction's label, its id as one byte, then the challenge of the connection it
/// travels on (36 bytes). An envelope opens only in the direction, under the id and on the connection it was sealed
/// for, so a reflected message, one moved to another call, and one captured and written again from another central or
/// on a later connection never open.
#[must_use]
pub fn aad(direction: Direction, id: u8, challenge: &Challenge) -> Vec<u8> {
	let mut out = direction.label().to_vec();
	out.push(id);
	out.extend(challenge);
	out
}

/// Seals and opens envelopes under the ble key.
#[derive(Clone)]
pub struct BleKeys {
	cipher: Aes256Gcm,
}

impl BleKeys {
	/// The cipher of a ble key (`key`).
	#[must_use]
	pub fn new(key: &[u8; 32]) -> Self {
		Self { cipher: Aes256Gcm::new(&Key::<Aes256Gcm>::from(*key)) }
	}

	/// Seals a message's plain text under a fresh random nonce, for the connection of `challenge`.
	#[must_use]
	pub fn seal(&self, direction: Direction, id: u8, challenge: &Challenge, plain: &[u8]) -> Vec<u8> {
		self.seal_with(aead::Nonce::<Aes256Gcm>::generate().into(), direction, id, challenge, plain)
	}

	/// Seals under a given nonce (the fixtures'; a live message always takes `seal`): nonce, ciphertext, tag.
	#[must_use]
	pub fn seal_with(
		&self,
		nonce: [u8; NONCE],
		direction: Direction,
		id: u8,
		challenge: &Challenge,
		plain: &[u8],
	) -> Vec<u8> {
		let aad = aad(direction, id, challenge);
		let mut out = nonce.to_vec();
		// encryption fails only past AES-GCM's 64 GiB message limit
		let sealed = self.cipher.encrypt(&aead::Nonce::<Aes256Gcm>::from(nonce), Payload { msg: plain, aad: &aad });
		out.extend(sealed.unwrap_or_default());
		out
	}

	/// Opens an envelope sealed in `direction` under `id` on the connection of `challenge`.
	///
	/// # Errors
	/// An envelope shorter than a nonce and a tag, or one whose tag does not match (a changed byte, another key, the
	/// other direction, another id or another connection's challenge).
	pub fn open(
		&self,
		direction: Direction,
		id: u8,
		challenge: &Challenge,
		envelope: &[u8],
	) -> Result<Vec<u8>, SealError> {
		if envelope.len() < ENVELOPE_MIN_BYTES {
			return Err(SealError::Short);
		}
		let (nonce, body) = envelope.split_at(NONCE);
		let nonce: [u8; NONCE] = nonce.try_into().map_err(|_| SealError::Short)?;
		let aad = aad(direction, id, challenge);
		self.cipher
			.decrypt(&aead::Nonce::<Aes256Gcm>::from(nonce), Payload { msg: body, aad: &aad })
			.map_err(|_| SealError::Tag)
	}
}

/// Why an envelope that opened did not read as its head.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum HeadError {
	#[error("shorter than its head says")]
	Short,
	#[error("a method that is no GET or POST")]
	Method,
	#[error("a path that is not 1-1024 bytes of visible ASCII under /v1/")]
	Path,
	#[error("an agent that is not printable ASCII")]
	Agent,
	#[error("a GET that carries a body")]
	Body,
}

/// A request's method, as its head carries it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
	Get,
	Post,
}

impl Method {
	fn code(self) -> u8 {
		match self {
			Self::Get => 1,
			Self::Post => 2,
		}
	}

	/// The HTTP method's name.
	#[must_use]
	pub fn name(self) -> &'static str {
		match self {
			Self::Get => "GET",
			Self::Post => "POST",
		}
	}
}

/// A request as an envelope toward the helper carries it: the HTTP request the phone would send on the LAN, less the
/// bearer, which the helper adds itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
	pub method: Method,
	/// The path with its query (`/v1/…`).
	pub path: String,
	/// The User-Agent the LAN request would carry; empty for none.
	pub agent: String,
	/// The LAN body as sent: sealed under the seal key for a POST, empty for a GET.
	pub body: Vec<u8>,
}

impl Request {
	/// The plain text: the method (1 byte: 1 GET, 2 POST), the path's length (2 bytes, big-endian) and the path, the
	/// agent's length (1 byte) and the agent, then the body.
	#[must_use]
	pub fn to_plain(&self) -> Vec<u8> {
		let path = self.path.as_bytes();
		let agent = self.agent.as_bytes();
		let mut out = Vec::with_capacity(4 + path.len() + agent.len() + self.body.len());
		out.push(self.method.code());
		out.extend(u16::try_from(path.len()).unwrap_or(u16::MAX).to_be_bytes());
		out.extend(path);
		out.push(u8::try_from(agent.len()).unwrap_or(u8::MAX));
		out.extend(agent);
		out.extend(&self.body);
		out
	}

	/// Reads an opened request's head: the method, a path of 1-1024 bytes of visible ASCII (0x21-0x7E) under `/v1/`,
	/// an agent of 0-255 bytes of 0x20-0x7E, and the body, none for a GET.
	///
	/// # Errors
	/// The head does not read (`HeadError`).
	pub fn read(plain: &[u8]) -> Result<Self, HeadError> {
		let (&method, rest) = plain.split_first().ok_or(HeadError::Short)?;
		let method = match method {
			1 => Method::Get,
			2 => Method::Post,
			_ => return Err(HeadError::Method),
		};
		let (len, rest) = rest.split_first_chunk::<2>().ok_or(HeadError::Short)?;
		let (path, rest) = rest.split_at_checked(usize::from(u16::from_be_bytes(*len))).ok_or(HeadError::Short)?;
		if path.is_empty()
			|| path.len() > PATH_MAX_BYTES
			|| !path.starts_with(b"/v1/")
			|| !path.iter().all(|b| (0x21..=0x7e).contains(b))
		{
			return Err(HeadError::Path);
		}
		let (&len, rest) = rest.split_first().ok_or(HeadError::Short)?;
		let (agent, body) = rest.split_at_checked(usize::from(len)).ok_or(HeadError::Short)?;
		if !agent.iter().all(|b| (0x20..=0x7e).contains(b)) {
			return Err(HeadError::Agent);
		}
		if method == Method::Get && !body.is_empty() {
			return Err(HeadError::Body);
		}
		// every byte of both is ASCII, checked above
		let text = |bytes: &[u8]| String::from_utf8_lossy(bytes).into_owned();
		Ok(Self { method, path: text(path), agent: text(agent), body: body.to_vec() })
	}
}

/// A response as an envelope toward the phone carries it: the HTTP status and the body the LAN would answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
	pub status: u16,
	/// The LAN response's body: sealed under the seal key for a 200, empty otherwise.
	pub body: Vec<u8>,
}

impl Response {
	/// The plain text: the status (2 bytes, big-endian), then the body.
	#[must_use]
	pub fn to_plain(&self) -> Vec<u8> {
		let mut out = self.status.to_be_bytes().to_vec();
		out.extend(&self.body);
		out
	}

	/// Reads an opened response.
	///
	/// # Errors
	/// Shorter than its status.
	pub fn read(plain: &[u8]) -> Result<Self, HeadError> {
		let (status, body) = plain.split_first_chunk::<2>().ok_or(HeadError::Short)?;
		Ok(Self { status: u16::from_be_bytes(*status), body: body.to_vec() })
	}
}

/// The most bytes a chunk carries, header included, at an ATT MTU: min(MTU - 3, 512), and never below LE's least.
#[must_use]
pub fn chunk_size(att_mtu: usize) -> usize {
	att_mtu.max(ATT_MTU_MIN).saturating_sub(3).min(ATT_VALUE_MAX)
}

/// An envelope cut into chunks of at most `size` bytes (`chunk_size`; a central's most a notification carries, held to
/// `ATT_VALUE_MAX`): each the id and its index (3 bytes, big-endian) and its part, the first part starting with the
/// envelope's length (4 bytes, big-endian); every chunk but the last is full.
#[must_use]
pub fn chunks(id: u8, envelope: &[u8], size: usize) -> Vec<Vec<u8>> {
	let size = size.clamp(chunk_size(ATT_MTU_MIN), ATT_VALUE_MAX);
	let mut whole = u32::try_from(envelope.len()).unwrap_or(u32::MAX).to_be_bytes().to_vec();
	whole.extend(envelope);
	whole
		.chunks(size - HEADER_BYTES)
		.enumerate()
		.map(|(index, part)| {
			let mut chunk = vec![id];
			chunk.extend(&u32::try_from(index).unwrap_or(u32::MAX).to_be_bytes()[1..]);
			chunk.extend(part);
			chunk
		})
		.collect()
}

/// Why a receiver dropped a message it was putting together (`Reassembly`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Why {
	/// A first chunk shorter than its header and the length, or a later one with nothing after its header.
	Short,
	/// A length under an envelope's least.
	Small,
	/// A length past the receiver's limit.
	Large,
	/// A chunk out of order: one between went missing.
	Gap,
	/// More bytes than the length said.
	Over,
	/// `IN_FLIGHT_MAX` messages under way already.
	Busy,
	/// Its next chunk came more than `CHUNK_GAP_MS` after the one before, or a look at the clock found it waiting that
	/// long.
	Slow,
}

impl Why {
	/// The word the fixture and the log use.
	#[must_use]
	pub fn word(self) -> &'static str {
		match self {
			Self::Short => "short",
			Self::Small => "small",
			Self::Large => "large",
			Self::Gap => "gap",
			Self::Over => "over",
			Self::Busy => "busy",
			Self::Slow => "slow",
		}
	}
}

/// What a chunk did to the messages under way.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step {
	/// Its message is not whole yet.
	Partial,
	/// Its message is whole: the envelope, to open under its id.
	Message { id: u8, envelope: Vec<u8> },
	/// Its message went (`Why`).
	Dropped { id: u8, why: Why },
	/// It belongs to no message: a later chunk with no first, or a scrap shorter than a header.
	Ignored,
}

/// One message under way.
struct Open {
	id: u8,
	length: usize,
	/// The index the next chunk must carry.
	next: u32,
	envelope: Vec<u8>,
	/// When its last chunk came.
	at: i64,
}

/// The messages one sender has under way, put together chunk by chunk: at most `IN_FLIGHT_MAX` at once, each within
/// `limit` bytes (`REQUEST_MAX_BYTES` toward the helper) and with its chunks at most `CHUNK_GAP_MS` apart.
pub struct Reassembly {
	limit: usize,
	open: Vec<Open>,
}

impl Reassembly {
	/// A receiver that takes envelopes of at most `limit` bytes.
	#[must_use]
	pub fn new(limit: usize) -> Self {
		Self { limit, open: Vec::new() }
	}

	/// Takes one chunk at `now`. A first chunk for an id under way begins that id anew, the earlier message going
	/// unreported (the sender gave it up); a whole message in its first chunk takes no place among the ones under way.
	pub fn push(&mut self, chunk: &[u8], now: i64) -> Step {
		let Some((header, part)) = chunk.split_first_chunk::<HEADER_BYTES>() else { return Step::Ignored };
		let id = header[0];
		let index = u32::from_be_bytes([0, header[1], header[2], header[3]]);
		if index == 0 {
			self.open.retain(|m| m.id != id);
			return self.first(id, part, now);
		}
		let Some(at) = self.open.iter().position(|m| m.id == id) else { return Step::Ignored };
		let message = &mut self.open[at];
		let why = if part.is_empty() {
			Some(Why::Short)
		} else if now - message.at > CHUNK_GAP_MS {
			Some(Why::Slow)
		} else if index != message.next {
			Some(Why::Gap)
		} else if message.envelope.len() + part.len() > message.length {
			Some(Why::Over)
		} else {
			None
		};
		if let Some(why) = why {
			self.open.remove(at);
			return Step::Dropped { id, why };
		}
		message.envelope.extend(part);
		message.next += 1;
		message.at = now;
		if message.envelope.len() < message.length {
			return Step::Partial;
		}
		let Open { envelope, .. } = self.open.remove(at);
		Step::Message { id, envelope }
	}

	/// A message's first chunk: its length, then the start of its envelope.
	fn first(&mut self, id: u8, part: &[u8], now: i64) -> Step {
		let Some((length, part)) = part.split_first_chunk::<LENGTH_BYTES>() else {
			return Step::Dropped { id, why: Why::Short };
		};
		let length = usize::try_from(u32::from_be_bytes(*length)).unwrap_or(usize::MAX);
		let why = if length < ENVELOPE_MIN_BYTES {
			Why::Small
		} else if length > self.limit {
			Why::Large
		} else if part.len() > length {
			Why::Over
		} else if part.len() == length {
			return Step::Message { id, envelope: part.to_vec() };
		} else if self.open.len() >= IN_FLIGHT_MAX {
			Why::Busy
		} else {
			self.open.push(Open { id, length, next: 1, envelope: part.to_vec(), at: now });
			return Step::Partial;
		};
		Step::Dropped { id, why }
	}

	/// A look at the clock: the messages whose last chunk came more than `CHUNK_GAP_MS` before `now` go, each with its
	/// id.
	pub fn tick(&mut self, now: i64) -> Vec<(u8, Why)> {
		let mut gone = Vec::new();
		self.open.retain(|m| {
			let waits = now - m.at > CHUNK_GAP_MS;
			if waits {
				gone.push((m.id, Why::Slow));
			}
			!waits
		});
		gone
	}

	/// Whether no message is under way.
	#[must_use]
	pub fn is_empty(&self) -> bool {
		self.open.is_empty()
	}
}

/// The last eight characters of a central's id that may go into a log line (an ASCII letter or digit): the Mac's
/// `CBCentral` identifier's UUID, Windows' device id's address.
#[must_use]
pub fn central_tag(central: &str) -> String {
	let tag: Vec<char> = central.chars().filter(char::is_ascii_alphanumeric).collect();
	tag[tag.len().saturating_sub(8)..].iter().collect()
}
