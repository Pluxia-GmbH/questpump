//! The link's keys and seal (contract/companion-api.yaml; rust-helper `protocol-and-mirror.md`, "The LAN API"): the
//! pairing key never travels. HKDF-SHA256 (salt empty) derives the bearer every request carries (info
//! `questpump bearer v1`, 32 bytes, base64url without padding), the key that seals every body with AES-256-GCM (info
//! `questpump seal v1`, 32 bytes) as nonce (12 bytes), ciphertext, tag (16 bytes), its associated data the body's
//! route (`Route`, since the release security review of 2026-10-31, L1), and the hello key (info
//! `questpump hello v1`, 32 bytes), which keys the hellos' proofs of the key (`link::HelloResponse`, `crate::ble`'s
//! hello): one-way, so a device tells the key it holds from a new pairing code before its bearer goes out, and nobody
//! learns the key or follows the computer by its hello. `contract/fixtures/seal.json` pins all three; the fourth, the
//! key that seals the Bluetooth link's envelopes (info `questpump ble v1`, slice 43), is `crate::ble`'s and
//! `contract/fixtures/ble.json`'s.

use std::collections::{HashSet, VecDeque};

use aes_gcm::aead::{Aead, Generate, KeyInit, Payload};
use aes_gcm::{Aes256Gcm, Key, aead};
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use hkdf::Hkdf;
use hkdf::hmac::{Hmac, Mac as _};
use sha2::Sha256;
use subtle::ConstantTimeEq as _;

use crate::ble::BleKeys;

pub const NONCE: usize = 12;
pub const TAG: usize = 16;
const BEARER_INFO: &[u8] = b"questpump bearer v1";
const SEAL_INFO: &[u8] = b"questpump seal v1";
const HELLO_INFO: &[u8] = b"questpump hello v1";
/// What a route's associated data starts with (`Route::aad`).
const ROUTE: &str = "questpump route v1";
/// The bytes of HMAC-SHA256 a hello's proof keeps (`proof`).
pub const PROOF_BYTES: usize = 8;

/// Why a body did not open.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum SealError {
	#[error("shorter than a nonce and a tag")]
	Short,
	/// A changed byte, or sealed under another key.
	#[error("the tag does not match")]
	Tag,
}

/// HKDF-SHA256 with an empty salt: `N` bytes of `key` (a pairing key, the sync's R, a link answer's ECDH secret) for
/// `info`.
pub(crate) fn derive<const N: usize>(key: &[u8], info: &[u8]) -> [u8; N] {
	let mut out = [0u8; N];
	// 8 and 32 bytes are far below HKDF-SHA256's limit of 255 × 32, so expand cannot fail
	let _ = Hkdf::<Sha256>::new(None, key).expand(info, &mut out);
	out
}

/// The hello key of a pairing key, which keys both hellos' proofs of the key (`Keys::hello`).
#[must_use]
pub fn hello_key(key: &[u8; 32]) -> [u8; 32] {
	derive(key, HELLO_INFO)
}

/// A hello's proof: the first `PROOF_BYTES` of HMAC-SHA256 under `key` over the parts (the LAN's
/// `link::HelloResponse`, the Bluetooth hello's `k`).
pub(crate) fn proof(key: &[u8], parts: &[&[u8]]) -> [u8; PROOF_BYTES] {
	let mut out = [0; PROOF_BYTES];
	// HMAC takes a key of any length, so a hello key or an install's id always makes one
	let Ok(mut mac) = <Hmac<Sha256> as hkdf::hmac::KeyInit>::new_from_slice(key) else { return out };
	for part in parts {
		mac.update(part);
	}
	out.copy_from_slice(&mac.finalize().into_bytes()[..PROOF_BYTES]);
	out
}

/// What a pairing key gives the link: the bearer, the route-bound seal, the hello key and the Bluetooth link's
/// envelopes.
#[derive(Clone)]
pub struct Keys {
	/// What `Authorization: Bearer` carries.
	pub bearer: String,
	pub seal: RouteSeal,
	/// What the hellos' proofs of the key are keyed with ([`hello_key`]).
	pub hello: [u8; 32],
	/// The Bluetooth link's envelopes (slice 43, `crate::ble::key`).
	pub ble: BleKeys,
}

impl Keys {
	#[must_use]
	pub fn derive(key: &[u8; 32]) -> Self {
		Self {
			bearer: URL_SAFE_NO_PAD.encode(derive::<32>(key, BEARER_INFO)),
			seal: RouteSeal(Sealer::new(&derive(key, SEAL_INFO))),
			hello: hello_key(key),
			ble: BleKeys::new(&crate::ble::key(key)),
		}
	}

	/// Whether an `Authorization` header carries this bearer, compared in constant time.
	#[must_use]
	pub fn authorizes(&self, header: Option<&str>) -> bool {
		let given = header.and_then(|h| h.strip_prefix("Bearer ")).unwrap_or_default();
		given.len() == self.bearer.len() && bool::from(given.as_bytes().ct_eq(self.bearer.as_bytes()))
	}
}

/// Which way a LAN body goes, the first word of its route's associated data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Way {
	/// A body a device sends.
	Request,
	/// A body the helper sends back, a sealed refusal included.
	Answer,
}

/// The route a LAN body is sealed for: its associated data is `questpump route v1 <way> <METHOD> <path>`, the method
/// in capitals and the path without its query (`contract/fixtures/seal.json`), so a body opens only as the route and
/// the way it was sealed for: one recorded on a route and posted at another, or an answer sent back as a request, opens
/// nowhere.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Route<'a> {
	pub way: Way,
	pub method: &'a str,
	pub path: &'a str,
}

impl Route<'_> {
	/// The associated data's bytes.
	#[must_use]
	pub fn aad(&self) -> Vec<u8> {
		let way = match self.way {
			Way::Request => "request",
			Way::Answer => "answer",
		};
		format!("{ROUTE} {way} {} {}", self.method, self.path).into_bytes()
	}
}

/// The LAN's seal: the seal key's, every body bound to its route (`Route`).
#[derive(Clone)]
pub struct RouteSeal(Sealer);

impl RouteSeal {
	/// Seals a body for `route` under a fresh random nonce.
	#[must_use]
	pub fn seal(&self, route: &Route<'_>, plain: &[u8]) -> Vec<u8> {
		self.0.seal_in(aead::Nonce::<Aes256Gcm>::generate().into(), &route.aad(), plain)
	}

	/// Seals a body for `route` under a given nonce (the fixtures'; a live body always takes `seal`).
	#[must_use]
	pub fn seal_with(&self, nonce: [u8; NONCE], route: &Route<'_>, plain: &[u8]) -> Vec<u8> {
		self.0.seal_in(nonce, &route.aad(), plain)
	}

	/// Opens a body sealed for `route` and returns its plain text and its nonce.
	///
	/// # Errors
	/// A body shorter than a nonce and a tag, or one whose tag does not match under the key and the route.
	pub fn open(&self, route: &Route<'_>, sealed: &[u8]) -> Result<(Vec<u8>, [u8; NONCE]), SealError> {
		self.0.open_in(&route.aad(), sealed)
	}
}

/// Seals and opens bodies under a seal key, with no associated data: the sync's (`crate::sync`); the LAN's
/// is `RouteSeal`.
#[derive(Clone)]
pub struct Sealer {
	cipher: Aes256Gcm,
}

impl Sealer {
	#[must_use]
	pub fn new(key: &[u8; 32]) -> Self {
		Self { cipher: Aes256Gcm::new(&Key::<Aes256Gcm>::from(*key)) }
	}

	/// Seals under a fresh random nonce.
	#[must_use]
	pub fn seal(&self, plain: &[u8]) -> Vec<u8> {
		self.seal_with(aead::Nonce::<Aes256Gcm>::generate().into(), plain)
	}

	/// Seals under a given nonce (the fixtures'; a live body always takes `seal`).
	#[must_use]
	pub fn seal_with(&self, nonce: [u8; NONCE], plain: &[u8]) -> Vec<u8> {
		self.seal_in(nonce, &[], plain)
	}

	/// Opens a body and returns its plain text and its nonce.
	///
	/// # Errors
	/// A body shorter than a nonce and a tag, or one whose tag does not match.
	pub fn open(&self, sealed: &[u8]) -> Result<(Vec<u8>, [u8; NONCE]), SealError> {
		self.open_in(&[], sealed)
	}

	fn seal_in(&self, nonce: [u8; NONCE], aad: &[u8], plain: &[u8]) -> Vec<u8> {
		let mut out = nonce.to_vec();
		// encryption fails only past AES-GCM's 64 GiB message limit
		let payload = Payload { msg: plain, aad };
		out.extend(self.cipher.encrypt(&aead::Nonce::<Aes256Gcm>::from(nonce), payload).unwrap_or_default());
		out
	}

	fn open_in(&self, aad: &[u8], sealed: &[u8]) -> Result<(Vec<u8>, [u8; NONCE]), SealError> {
		if sealed.len() < NONCE + TAG {
			return Err(SealError::Short);
		}
		let (nonce, body) = sealed.split_at(NONCE);
		let nonce: [u8; NONCE] = nonce.try_into().map_err(|_| SealError::Short)?;
		let payload = Payload { msg: body, aad };
		let plain = self.cipher.decrypt(&aead::Nonce::<Aes256Gcm>::from(nonce), payload).map_err(|_| SealError::Tag)?;
		Ok((plain, nonce))
	}
}

/// The nonces of the requests of the last few minutes: a body sent again (a replay on the LAN) is refused. A request
/// older than the window is refused by its `sent_at` (`link::FRESH_MS`), so the window never has to outlive it and
/// nothing has to survive a restart.
pub struct Replay {
	seen: HashSet<[u8; NONCE]>,
	order: VecDeque<(i64, [u8; NONCE])>,
	window_ms: i64,
}

impl Replay {
	/// Remembers nonces for `window_ms`.
	#[must_use]
	pub fn new(window_ms: i64) -> Self {
		Self { seen: HashSet::new(), order: VecDeque::new(), window_ms }
	}

	/// Whether a nonce is new at `now`; a new one is remembered.
	pub fn fresh(&mut self, nonce: [u8; NONCE], now: i64) -> bool {
		while self.order.front().is_some_and(|&(at, _)| at < now - self.window_ms) {
			if let Some((_, old)) = self.order.pop_front() {
				self.seen.remove(&old);
			}
		}
		if !self.seen.insert(nonce) {
			return false;
		}
		self.order.push_back((now, nonce));
		true
	}
}
