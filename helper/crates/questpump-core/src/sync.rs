//! The sync's keys, codes and batches (docs/plan.md slice 60; `contract/sync-api.yaml`; `contract/fixtures/seal.json`,
//! its `sync` and `rekey` blocks): the computer's side of the phones' sync cores, one owner for each. An account is a
//! random 16-byte id and a random 16-byte root secret R, which only the player's devices hold. From R, HKDF-SHA256 with
//! an empty salt gives the seal key (info `questpump sync seal v1`, 32 bytes), the auth key the server keeps the SHA-256
//! of (`questpump sync auth v1`, 32 bytes, base64url without padding), the key id every pushed batch names
//! (`questpump sync id v1`, 8 bytes, base64url without padding) and the key each device's own key has its MAC under
//! (`questpump sync device v1`, 32 bytes) (`SyncKeys`); the seal is the LAN's (`seal::Sealer`). A batch's plain text is
//! JSON, `{"v","account","device","seq","made_at","kind","body"}`, compressed with raw DEFLATE and sealed, at most
//! `MAX_SEALED` (`Batch`, `seal_batch`, `open_batch`); its body is a backup file's (`RowsBody`) or the device's own
//! (`DeviceBody`). R travels only sealed or in the player's hands: the restore code (`restore_code`, `read_restore`),
//! the link code a phone scans, and the link answer to the request code a computer shows (`LinkCode`), sealed to the
//! P-256 key the computer made for that request (`KeyPair`, `open_link_answer`), so the code on a screen and the answer
//! on the server tell nothing without that key; the check code both devices derive from the same ECDH (`Agreed::check`)
//! lets the player see that the answer came from their own phone. Each removal of a device makes a new R due ("The
//! account's new key"): every device registers a P-256 key of its own with its MAC under R (`SyncKeys::mac`), and the
//! device that makes the new R wraps it to each key whose MAC holds with the link answer's construction under its own
//! info (`wrap_root`, `open_wrap`) and seals the R it replaces under the new one, the chain's link (`open_prev`). The
//! cursor, the device token, this device's own key and what was sent are the account's state on this computer
//! (`Account`), which the app keeps in its settings, R, a rekey's R', the token and this device's own key in the
//! system's secret store with the pairing key.

use std::collections::BTreeMap;
use std::fmt;
use std::io::{Read as _, Write as _};

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use flate2::Compression;
use flate2::read::DeflateDecoder;
use flate2::write::DeflateEncoder;
use hkdf::hmac::{Hmac, KeyInit as _, Mac as _};
use p256::elliptic_curve::sec1::ToSec1Point as _;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::backup::GameRow;
use crate::body::Body;
use crate::link::SheetCharacter;
use crate::seal::{self, SealError, Sealer};

/// The batch's version this build writes, and the newest it reads.
pub const VERSION: u32 = 1;
/// The most a sealed batch may be (the server's 413 past it): a larger push goes as several batches.
pub const MAX_SEALED: usize = 256 << 10;
/// The most a batch's plain text may inflate to: past it the batch does not read (a sealed 256 KiB of DEFLATE could
/// inflate a thousandfold).
const PLAIN_MOST: u64 = 32 << 20;
/// R's length, and the account id's.
pub const ROOT: usize = 16;
/// A link request's id r.
pub const REQUEST_ID: usize = 16;
/// A device's id in the account, made by the server.
pub const DEVICE_ID: usize = 16;
/// A link request's public key P as its code carries it: P-256, SEC1 compressed.
pub const REQUEST_KEY: usize = 33;
/// A P-256 public key in SEC1's uncompressed form: the answerer's E at the head of a link answer, a rekey wrap's E and a
/// device's own key as it registers it.
pub const PUBLIC_KEY: usize = 65;
/// A device key's MAC.
pub const MAC: usize = 32;
/// A P-256 secret scalar's length.
pub const SECRET: usize = 32;
const SEAL_INFO: &[u8] = b"questpump sync seal v1";
const AUTH_INFO: &[u8] = b"questpump sync auth v1";
const ID_INFO: &[u8] = b"questpump sync id v1";
/// The key id's length.
const KID: usize = 8;
const LINK_INFO: &[u8] = b"questpump link v2";
const CHECK_INFO: &[u8] = b"questpump link check";
const DEVICE_INFO: &[u8] = b"questpump sync device v1";
const REKEY_INFO: &[u8] = b"questpump sync rekey v1";
/// The check code's numbers: six digits.
const CHECK_RANGE: u32 = 1_000_000;
/// What every link code and request code starts with.
const LINK_PREFIX: &str = "questpump-link:1:";
/// The request code's word after the prefix (an account id is 22 characters, never three).
const REQUEST_WORD: &str = "req";
/// The kind of this computer in the account (`Kind` of the contract).
pub const DESKTOP: &str = "desktop";

/// Crockford's base32 symbols, and its check symbols after them (the restore code).
const CROCKFORD: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";
const CHECK: &[u8; 37] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ*~$=U";
/// The restore code's symbols before its check symbol: 128 bits and two zero bits.
const SYMBOLS: usize = 26;

/// What R gives the sync: the seal, the auth key, the key id and the key each device's own key has its MAC under. No
/// `Debug`: it holds keys.
#[derive(Clone)]
pub struct SyncKeys {
	pub seal: Sealer,
	/// What `POST /api/sync/v1/accounts` and `POST /api/sync/v1/session` carry (base64url without padding).
	pub auth: String,
	/// What every pushed batch names (base64url without padding).
	pub kid: String,
	/// HKDF-SHA256 of R, info `questpump sync device v1`: what a device key's MAC is made under (`mac`).
	device: [u8; 32],
}

impl SyncKeys {
	#[must_use]
	pub fn derive(r: &[u8; ROOT]) -> Self {
		Self {
			seal: Sealer::new(&seal::derive(r, SEAL_INFO)),
			auth: URL_SAFE_NO_PAD.encode(seal::derive::<32>(r, AUTH_INFO)),
			kid: URL_SAFE_NO_PAD.encode(seal::derive::<KID>(r, ID_INFO)),
			device: seal::derive(r, DEVICE_INFO),
		}
	}

	fn hmac(&self, device: &[u8; DEVICE_ID], key: &[u8; PUBLIC_KEY]) -> Hmac<sha2::Sha256> {
		// HMAC takes a key of any length
		let Ok(mut mac) = Hmac::<sha2::Sha256>::new_from_slice(&self.device) else {
			unreachable!("HMAC takes a key of any length")
		};
		mac.update(device);
		mac.update(key);
		mac
	}

	/// A device key's MAC under this R (contract/sync-api.yaml, "Each device's key"): HMAC-SHA256 over the device's
	/// id (16 bytes) then its key (65), with no length or separator, which only a holder of R can make.
	#[must_use]
	pub fn mac(&self, device: &[u8; DEVICE_ID], key: &[u8; PUBLIC_KEY]) -> [u8; MAC] {
		self.hmac(device, key).finalize().into_bytes().into()
	}

	/// Whether `mac` is that device key's MAC under this R, compared in constant time.
	#[must_use]
	pub fn mac_holds(&self, device: &[u8; DEVICE_ID], key: &[u8; PUBLIC_KEY], mac: &[u8]) -> bool {
		self.hmac(device, key).verify_slice(mac).is_ok()
	}
}

/// A P-256 key pair whose secret stays on this computer (contract/sync-api.yaml): one for each link request, its
/// public key P in the request code and the only key that opens the answer ("Linking a device by a code"); this
/// device's own, made once and registered with its MAC under each R, which opens the wraps of a new R ("Each device's
/// key"); and a fresh one for each wrap this computer makes (`wrap_root`). No `Debug`: it holds the secret.
pub struct KeyPair(p256::SecretKey);

impl KeyPair {
	/// The key of a secret scalar (32 random bytes); none for bytes that are no P-256 secret (zero, or not below the
	/// group's order).
	#[must_use]
	pub fn from_bytes(secret: &[u8; SECRET]) -> Option<Self> {
		p256::SecretKey::from_bytes(&(*secret).into()).ok().map(Self)
	}

	/// The public key as a request code carries it: SEC1 compressed.
	#[must_use]
	pub fn public(&self) -> [u8; REQUEST_KEY] {
		p256::CompressedPoint::from(&self.0.public_key()).into()
	}

	/// The public key as a device registers it and a wrap starts with it: SEC1 uncompressed.
	#[must_use]
	pub fn public_uncompressed(&self) -> [u8; PUBLIC_KEY] {
		self.0.public_key().to_uncompressed_point().into()
	}

	/// The x coordinate of ECDH with the holder of `theirs` (a P-256 public key in either SEC1 form), the same from
	/// either side; none for bytes that are no point of the curve.
	fn shared(&self, theirs: &[u8]) -> Option<p256::ecdh::SharedSecret> {
		let theirs = p256::PublicKey::from_sec1_bytes(theirs).ok()?;
		Some(p256::ecdh::diffie_hellman(self.0.to_nonzero_scalar(), theirs.as_affine()))
	}

	/// What a link request's key and an answer's key share; none for bytes that are no point of the curve.
	#[must_use]
	pub fn agree(&self, theirs: &[u8]) -> Option<Agreed> {
		let shared = self.shared(theirs)?;
		let x = shared.raw_secret_bytes();
		Some(Agreed { seal: Sealer::new(&seal::derive(x, LINK_INFO)), check: check_code(x) })
	}

	/// The seal of a rekey's wrap between this key and `theirs`: HKDF-SHA256 of the x coordinate of their ECDH, salt
	/// empty, info `questpump sync rekey v1`, 32 bytes; none for bytes that are no point of the curve.
	#[must_use]
	pub fn wrap_seal(&self, theirs: &[u8]) -> Option<Sealer> {
		Some(Sealer::new(&seal::derive(self.shared(theirs)?.raw_secret_bytes(), REKEY_INFO)))
	}
}

/// What a link request's key and an answer's key share, both from the x coordinate of their ECDH.
pub struct Agreed {
	/// The answer's seal: HKDF-SHA256, salt empty, info `questpump link v2`, 32 bytes.
	pub seal: Sealer,
	/// The check code the answering phone and the requesting device both show (`check_code`).
	pub check: String,
}

/// The check code of ECDH's x coordinate: the first 4 bytes of HKDF-SHA256 (salt empty, info `questpump link check`)
/// read as a big-endian unsigned 32-bit number, its remainder by 1,000,000 written as six digits with leading zeros.
fn check_code(x: &[u8]) -> String {
	format!("{:06}", u32::from_be_bytes(seal::derive(x, CHECK_INFO)) % CHECK_RANGE)
}

/// Base64url without padding, as every id and key travels.
#[must_use]
pub fn text<const N: usize>(bytes: &[u8; N]) -> String {
	URL_SAFE_NO_PAD.encode(bytes)
}

/// An id or a key from its base64url text; none of another length.
#[must_use]
pub fn bytes<const N: usize>(text: &str) -> Option<[u8; N]> {
	URL_SAFE_NO_PAD.decode(text).ok()?.try_into().ok()
}

/// Why a restore code did not read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum RestoreError {
	#[error("not 27 symbols")]
	Length,
	#[error("a symbol Crockford's base32 lacks")]
	Symbol,
	/// The check symbol does not match, or the two bits after R are not zero.
	#[error("the check symbol does not match")]
	Check,
}

/// R as the restore code shows it: Crockford's base32 of its 128 bits and two zero bits, then the check symbol (the 26
/// symbols' values summed mod 37), in groups of four (`XXXX-XXXX-XXXX-XXXX-XXXX-XXXX-XXC`).
#[must_use]
pub fn restore_code(r: &[u8; ROOT]) -> String {
	let n = u128::from_be_bytes(*r);
	let mut symbols = Vec::with_capacity(SYMBOLS + 1);
	let mut sum = 0usize;
	for i in 0..SYMBOLS {
		// symbol i holds bits 5i to 5i + 4 of the 130 (R's 128, then two zero bits), from the top
		let shift = 125 - 5 * i;
		let value = if shift >= 2 { (n >> (shift - 2)) & 31 } else { (n << 2) & 31 };
		let value = usize::try_from(value).unwrap_or_default();
		sum += value;
		symbols.push(CROCKFORD[value]);
	}
	symbols.push(CHECK[sum % CHECK.len()]);
	let groups: Vec<String> = symbols.chunks(4).map(|g| String::from_utf8_lossy(g).into_owned()).collect();
	groups.join("-")
}

/// A symbol's value as the restore code reads it: case-blind, `O` as `0`, `I` and `L` as `1`; the check symbols too
/// where `check`.
fn symbol(c: u8, check: bool) -> Option<usize> {
	let c = match c.to_ascii_uppercase() {
		b'O' => b'0',
		b'I' | b'L' => b'1',
		c => c,
	};
	let table: &[u8] = if check { CHECK } else { CROCKFORD };
	table.iter().position(|&s| s == c)
}

/// R from a restore code as the player typed or pasted it: case-blind, `O` as `0`, `I` and `L` as `1`, hyphens and
/// spaces skipped.
///
/// # Errors
/// Not 27 symbols, a symbol the alphabet lacks, or a check that does not hold.
pub fn read_restore(code: &str) -> Result<[u8; ROOT], RestoreError> {
	let kept: Vec<u8> = code.bytes().filter(|&b| b != b'-' && !b.is_ascii_whitespace()).collect();
	let [body @ .., check] = kept.as_slice() else { return Err(RestoreError::Length) };
	if body.len() != SYMBOLS {
		return Err(RestoreError::Length);
	}
	let values: Vec<usize> =
		body.iter().map(|&c| symbol(c, false)).collect::<Option<_>>().ok_or(RestoreError::Symbol)?;
	let check = symbol(*check, true).ok_or(RestoreError::Symbol)?;
	if values.iter().sum::<usize>() % CHECK.len() != check {
		return Err(RestoreError::Check);
	}
	let wide = |v: usize| u128::try_from(v).unwrap_or_default();
	let mut n = 0u128;
	for &v in &values[..SYMBOLS - 1] {
		n = (n << 5) | wide(v);
	}
	let last = wide(values[SYMBOLS - 1]);
	if last & 3 != 0 {
		return Err(RestoreError::Check);
	}
	n = (n << 3) | (last >> 2);
	Ok(n.to_be_bytes())
}

/// A code a QR carries between the player's devices (contract/sync-api.yaml, "Linking a device by a code").
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkCode {
	/// `questpump-link:1:<account>:<R>`: a device with R shows it, a phone scans it and joins.
	Account { account: [u8; ROOT], r: [u8; ROOT] },
	/// `questpump-link:1:req:<r>:<P>`: a computer without R shows it, P its request's public key (`KeyPair`); a
	/// phone with R answers through the server, sealed to P.
	Request { id: [u8; REQUEST_ID], key: [u8; REQUEST_KEY] },
}

impl LinkCode {
	/// A scanned or pasted code; none that is no link code, a request's among them whose key is no compressed point of
	/// the curve (an older request's 32-byte T).
	#[must_use]
	pub fn read(code: &str) -> Option<Self> {
		let rest = code.trim().strip_prefix(LINK_PREFIX)?;
		let (first, second) = rest.split_once(':')?;
		if first == REQUEST_WORD {
			let (id, key) = second.split_once(':')?;
			let key: [u8; REQUEST_KEY] = bytes(key)?;
			p256::PublicKey::from_sec1_bytes(&key).ok()?;
			return Some(Self::Request { id: bytes(id)?, key });
		}
		Some(Self::Account { account: bytes(first)?, r: bytes(second)? })
	}
}

impl fmt::Display for LinkCode {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Account { account, r } => write!(f, "{LINK_PREFIX}{}:{}", text(account), text(r)),
			Self::Request { id, key } => write!(f, "{LINK_PREFIX}{REQUEST_WORD}:{}:{}", text(id), text(key)),
		}
	}
}

/// A link answer's plain text: the account and R, both base64url.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct LinkAnswer {
	pub account: String,
	pub r: String,
}

/// Why a batch or a link answer did not read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum OpenError {
	#[error("did not open: {0}")]
	Seal(SealError),
	#[error("did not inflate")]
	Inflate,
	#[error("did not read")]
	Read,
	/// Its account, device or seq is not the one the server listed it under.
	#[error("its account, device or seq differs from the listing")]
	Differs,
}

/// A link answer opened: the account and R it brings, and the check code it shares with the phone that sealed it. No
/// `Debug`: it holds R.
pub struct LinkOpened {
	pub account: [u8; ROOT],
	pub r: [u8; ROOT],
	pub check: String,
}

/// What a sealed message to one of this computer's keys brings (a link answer, a rekey's wrap): the sender's fresh public
/// key E (`PUBLIC_KEY` bytes), then the body sealed under the seal `seal_of` makes of E, opened, with what else
/// `seal_of` gives.
fn open_sent<T>(sealed: &[u8], seal_of: impl FnOnce(&[u8]) -> Option<(Sealer, T)>) -> Result<(Vec<u8>, T), OpenError> {
	let (theirs, body) = sealed.split_at_checked(PUBLIC_KEY).ok_or(OpenError::Seal(SealError::Short))?;
	let (seal, also) = seal_of(theirs).ok_or(OpenError::Read)?;
	let (plain, _) = seal.open(body).map_err(OpenError::Seal)?;
	Ok((plain, also))
}

/// What a phone's link answer brings, opened with the request's key (`open_sent`, the link's seal).
///
/// # Errors
/// An answer whose head is no point of the curve, whose body does not open under the shared seal or does not read as an
/// answer.
pub fn open_link_answer(key: &KeyPair, sealed: &[u8]) -> Result<LinkOpened, OpenError> {
	let (plain, check) = open_sent(sealed, |theirs| key.agree(theirs).map(|a| (a.seal, a.check)))?;
	let answer: LinkAnswer = serde_json::from_slice(&plain).map_err(|_| OpenError::Read)?;
	Ok(LinkOpened {
		account: bytes(&answer.account).ok_or(OpenError::Read)?,
		r: bytes(&answer.r).ok_or(OpenError::Read)?,
		check,
	})
}

/// A new R wrapped to a device's key (contract/sync-api.yaml, "Making the new R"): the public key of `e`, a fresh key
/// made for this wrap alone, then R sealed under their wrap seal (`KeyPair::wrap_seal`), 109 bytes; none for a device
/// key that is no point of the curve.
#[must_use]
pub fn wrap_root(e: &KeyPair, theirs: &[u8], r: &[u8; ROOT]) -> Option<Vec<u8>> {
	let mut wrap = e.public_uncompressed().to_vec();
	wrap.extend(e.wrap_seal(theirs)?.seal(r));
	Some(wrap)
}

/// The new R a wrap brings, opened with this device's own key (`open_sent`, the wrap's seal).
///
/// # Errors
/// A wrap whose head is no point of the curve, or whose body does not open under the shared seal or is no R.
pub fn open_wrap(key: &KeyPair, sealed: &[u8]) -> Result<[u8; ROOT], OpenError> {
	let (plain, ()) = open_sent(sealed, |theirs| key.wrap_seal(theirs).map(|seal| (seal, ())))?;
	plain.try_into().map_err(|_| OpenError::Read)
}

/// The earlier R a link of the account's chain gives (contract/sync-api.yaml, "Taking the wrap"): `prev`, the R of
/// `prev_kid` sealed under the seal key of `kid`'s R, opened with `newer`, the R of `kid`. None unless `newer` is
/// `kid`'s R, `prev` opens under it and the R it holds is `prev_kid`'s, so a chain checked link by link from a new R
/// back to the R a device holds proves the new R came from a holder of that one.
#[must_use]
pub fn open_prev(newer: &[u8; ROOT], kid: &str, prev_kid: &str, prev: &[u8]) -> Option<[u8; ROOT]> {
	let keys = SyncKeys::derive(newer);
	if keys.kid != kid {
		return None;
	}
	let r: [u8; ROOT] = keys.seal.open(prev).ok()?.0.try_into().ok()?;
	(SyncKeys::derive(&r).kid == prev_kid).then_some(r)
}

/// A batch's kind.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
	/// A backup file's body: what the device made since its last batch.
	Rows,
	/// The device's own name, kind, profile, body stamps and characters' sheets; the server keeps a device's newest.
	Device,
}

impl Kind {
	#[must_use]
	pub fn name(self) -> &'static str {
		match self {
			Self::Rows => "rows",
			Self::Device => "device",
		}
	}
}

/// A batch's plain text, its keys in the contract's order.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Batch {
	pub v: u32,
	pub account: String,
	pub device: String,
	pub seq: u64,
	/// Unix milliseconds.
	pub made_at: i64,
	pub kind: Kind,
	pub body: Value,
}

/// Raw DEFLATE (no zlib header) of a batch's plain text.
#[must_use]
pub fn deflate(plain: &[u8]) -> Vec<u8> {
	let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
	// writing into a Vec cannot fail
	let _ = encoder.write_all(plain);
	encoder.finish().unwrap_or_default()
}

/// A batch's plain text from its raw DEFLATE, at most `PLAIN_MOST`.
///
/// # Errors
/// Bytes that are no raw DEFLATE, or that inflate past `PLAIN_MOST`.
pub fn inflate(deflated: &[u8]) -> Result<Vec<u8>, OpenError> {
	let mut plain = Vec::new();
	DeflateDecoder::new(deflated).take(PLAIN_MOST + 1).read_to_end(&mut plain).map_err(|_| OpenError::Inflate)?;
	if u64::try_from(plain.len()).unwrap_or(u64::MAX) > PLAIN_MOST {
		return Err(OpenError::Inflate);
	}
	Ok(plain)
}

/// A batch sealed as it is pushed: its JSON, raw DEFLATE, then the seal under a fresh nonce.
///
/// # Errors
/// A body that does not write as JSON (none of the bodies here fail).
pub fn seal_batch(keys: &SyncKeys, batch: &Batch) -> serde_json::Result<Vec<u8>> {
	Ok(keys.seal.seal(&deflate(&serde_json::to_vec(batch)?)))
}

/// What a fetched batch is.
#[derive(Debug, Clone, PartialEq)]
pub enum Opened {
	Read(Batch),
	/// Made by a newer QuestPump (its `v`): kept unread until an update reads it.
	Newer(u32),
}

/// A fetched batch opened, inflated and read, checked against what the server listed it under (its account, device
/// and seq).
///
/// # Errors
/// One that does not open, inflate or read, or that differs from its listing.
pub fn open_batch(keys: &SyncKeys, sealed: &[u8], account: &str, device: &str, seq: u64) -> Result<Opened, OpenError> {
	#[derive(Deserialize)]
	struct Head {
		v: u32,
	}
	let (deflated, _) = keys.seal.open(sealed).map_err(OpenError::Seal)?;
	let plain = inflate(&deflated)?;
	let head: Head = serde_json::from_slice(&plain).map_err(|_| OpenError::Read)?;
	if head.v > VERSION {
		return Ok(Opened::Newer(head.v));
	}
	let batch: Batch = serde_json::from_slice(&plain).map_err(|_| OpenError::Read)?;
	if batch.account != account || batch.device != device || batch.seq != seq {
		return Err(OpenError::Differs);
	}
	Ok(Opened::Read(batch))
}

/// A `rows` batch's body: a backup file's (`contract/fixtures/backup.json`: `pumps`, `rows`, `game.rows`) holding only
/// what the device made since its last batch. The computer sends no pumps, its own app rows as a phone keeps them
/// (`backup::unstamp`) and the game's rows (`GameRow`), each text with all its copies, so the receiver's union counts
/// them as the device holds them; its install id (`hid`), so the game rows keep their computer however the device's id
/// in the account changes (a phone's batch has none); and `chars`, each character index its game rows use to the
/// character's GUID, by which every reader renumbers them into its own numbering. A field the device has none of is
/// left out (contract/sync-api.yaml, "The batch"). Unknown fields (a phone's `game.sheets` and `game.characters`) are
/// skipped.
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct RowsBody {
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub pumps: Vec<Value>,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub rows: Vec<String>,
	#[serde(default, skip_serializing_if = "GameRows::is_empty")]
	pub game: GameRows,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub hid: Option<String>,
	/// The character index of each game row that names one (as its text: `"1"`) to that character's GUID.
	#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
	pub chars: BTreeMap<String, String>,
}

impl RowsBody {
	/// `chars` as indexes: each that reads as one (1 to 255), with its GUID.
	#[must_use]
	pub fn guids(&self) -> BTreeMap<u8, String> {
		self.chars
			.iter()
			.filter_map(|(index, guid)| Some((index.parse::<u8>().ok().filter(|&i| i > 0)?, guid.clone())))
			.filter(|(_, guid)| !guid.is_empty())
			.collect()
	}
}

/// A `rows` batch's game rows.
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct GameRows {
	#[serde(default)]
	pub rows: Vec<GameRow>,
}

impl GameRows {
	#[must_use]
	pub fn is_empty(&self) -> bool {
		self.rows.is_empty()
	}
}

/// A `device` batch's body: the device's name and kind, its engine profile, the body every device shares with each
/// field's change time (`Body`, as `POST /v1/body` merges it: the latest change wins), each character's newest sheet
/// as a book carries it (`carried`), and a computer's install id (`hid`), which names the computer its game rows'
/// characters belong to. A field the device has none of is left out, and one that reads `null` is none.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct DeviceBody {
	pub name: String,
	pub kind: String,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub profile: Option<Value>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub stamps: Option<Body>,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub sheets: Vec<SheetCharacter>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub hid: Option<String>,
}

/// The most the characters a device batch carries may take, as JSON (the phones' `Backup.charactersRoom`).
pub const CHARACTERS_ROOM: usize = 512_000;

/// A sheet's `at` (unix seconds), none counting before every other.
fn sheet_at(c: &SheetCharacter) -> i64 {
	c.sheet.as_ref().and_then(|s| s.get("at")).and_then(Value::as_i64).unwrap_or(i64::MIN)
}

/// The characters as a book carries them (the phones' `Backup.carried`): each with a sheet, without its sheet's
/// felt-unwell days (`lk`) and the owed moves its rider gates hold back (`held`), so no trace of a body note goes; the
/// oldest left out until the rest fit `CHARACTERS_ROOM`; by their place in the list.
#[must_use]
pub fn carried(characters: &[SheetCharacter]) -> Vec<SheetCharacter> {
	let mut kept: Vec<SheetCharacter> = characters
		.iter()
		.filter_map(|c| {
			let Some(Value::Object(sheet)) = &c.sheet else { return None };
			let mut sheet = sheet.clone();
			sheet.remove("lk");
			sheet.remove("held");
			Some(SheetCharacter { sheet: Some(Value::Object(sheet)), ..c.clone() })
		})
		.collect();
	kept.sort_by(|a, b| sheet_at(b).cmp(&sheet_at(a)).then(a.index.cmp(&b.index)));
	while !kept.is_empty() && serde_json::to_vec(&kept).map_or(usize::MAX, |b| b.len()) > CHARACTERS_ROOM {
		kept.pop();
	}
	kept.sort_by_key(|c| c.index);
	kept
}

/// What this computer sent of a store: the store's id when it was read (`Store::id`, `AppRows::id`) and how many of its
/// records; another id sends the store again from its first record (the union keeps each row once).
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
#[serde(default)]
pub struct Sent {
	pub store: String,
	pub upto: usize,
}

impl Sent {
	/// The first record to send of a store with this id.
	#[must_use]
	pub fn from(&self, store: &str) -> usize {
		if self.store == store { self.upto } else { 0 }
	}
}

/// The account on this computer, kept in settings.json, R, a rekey's R', the token and this computer's own key in the
/// system's secret store with the pairing key (the desktop app's `secrets`): Sync's switch (on by default), the
/// account id and R (base64url), this computer's device id and token in it, the cursor taken in up to, the next
/// batch's seq, what was sent of each store, the device batch last sent (its digest), the linked email's address as
/// this computer typed it (the server keeps only its hash), each device's name as its device batch gave it, the
/// newest version another device's batch was made by while this build reads none of it, when it last synced, and the
/// account's new key's state here: this computer's own key's secret, the kid of the R it was last registered under,
/// and the new R a rekey of this computer's waits with for its answer.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(default)]
pub struct Account {
	pub on: bool,
	pub account: Option<String>,
	pub r: Option<String>,
	pub device: Option<String>,
	pub token: Option<String>,
	pub cursor: u64,
	pub seq: u64,
	pub game: Sent,
	pub app: Sent,
	pub device_digest: Option<String>,
	/// Whether an email holds the account (the session's `email`), and its address when this computer linked it.
	pub email: bool,
	pub address: Option<String>,
	pub names: BTreeMap<String, String>,
	/// The first cursor of a batch kept unread for its newer version, and that version: a build that reads it takes
	/// the account in again from before it.
	pub newer: Option<(u64, u32)>,
	/// When the last pull reached the account's end (unix ms): the Sync card's last sync.
	pub synced_at: Option<i64>,
	/// This computer's own P-256 key's secret (base64url, `SECRET` bytes; `KeyPair`), made once and kept through every
	/// account it joins and leaves: a secret as R is, never synced, never in a backup, the mirror or a log.
	pub device_secret: Option<String>,
	/// The kid of the R this computer's key was last registered under in this account (`PUT /devices/self/key`
	/// answered 204, or a rekey of its own taken): another kid, or none, registers it again.
	pub key_kid: Option<String>,
	/// The new R of a rekey this computer posted (base64url), kept beside R until the account settles it: the server's
	/// 204, a read that names its kid or a join it opens makes it R, and only a read that names another kid at a
	/// generation past the one it was made from drops it, so an answer lost on the way never loses the account. It
	/// stays through a removal and a join of the same account with the same R, never through another R or account.
	pub next_r: Option<String>,
}

impl Default for Account {
	fn default() -> Self {
		Self {
			on: true,
			account: None,
			r: None,
			device: None,
			token: None,
			cursor: 0,
			seq: 1,
			game: Sent::default(),
			app: Sent::default(),
			device_digest: None,
			email: false,
			address: None,
			names: BTreeMap::new(),
			newer: None,
			synced_at: None,
			device_secret: None,
			key_kid: None,
			next_r: None,
		}
	}
}

impl Account {
	/// R, when this computer holds it.
	#[must_use]
	pub fn root(&self) -> Option<[u8; ROOT]> {
		bytes(self.r.as_deref()?)
	}

	/// The keys R gives, when this computer holds it.
	#[must_use]
	pub fn keys(&self) -> Option<SyncKeys> {
		self.root().map(|r| SyncKeys::derive(&r))
	}

	/// This computer's own key, when it made one.
	#[must_use]
	pub fn device_key(&self) -> Option<KeyPair> {
		KeyPair::from_bytes(&bytes(self.device_secret.as_deref()?)?)
	}

	/// The new R of this computer's rekey that waits for its answer.
	#[must_use]
	pub fn next_root(&self) -> Option<[u8; ROOT]> {
		bytes(self.next_r.as_deref()?)
	}

	/// A new R for this account (a wrap taken, or this computer's own rekey in): R replaced, the rekey's R dropped,
	/// and the kid this computer's key now stands registered under (a rekey of its own registers it under the new R;
	/// a wrap taken registers it next).
	pub fn new_root(&mut self, r: &[u8; ROOT], key_kid: Option<String>) {
		(self.r, self.key_kid, self.next_r) = (Some(text(r)), key_kid, None);
	}

	/// A new token in an account (a join, a new account): what it took in and sent starts again (the union makes a
	/// second take-in and send a no-op), and the next batch is seq 1 of the new device. The address this computer
	/// typed stays only for the account it was typed for, while an email holds it; this computer's own key stays,
	/// registered in none yet; a rekey's R' stays while the join is of the same account with the same R (a rekey of
	/// the old device that may still go in), and goes with any other, which the account moved on to.
	pub fn joined(&mut self, account: String, r: String, device: String, token: String, email: bool) {
		let same = self.account.as_deref() == Some(account.as_str());
		let same_r = same && self.r.as_deref() == Some(r.as_str());
		*self = Self {
			on: true,
			address: self.address.take().filter(|_| email && same),
			device_secret: self.device_secret.take(),
			next_r: self.next_r.take().filter(|_| same_r),
			..Self::default()
		};
		(self.account, self.r, self.device, self.token, self.email) =
			(Some(account), Some(r), Some(device), Some(token), email);
	}

	/// The account deleted from this computer (`DELETE /account` answered 204): Sync off, the account, R and the
	/// token gone, every row and this computer's own key kept; Sync turned on again starts a new account.
	pub fn left(&mut self) {
		*self = Self { on: false, device_secret: self.device_secret.take(), ..Self::default() };
	}

	/// Out of the account but still holding it: signed out here (`DELETE /session` answered 204), removed by another
	/// device (a 401 on any call with the token), a new R whose wrap did not open or check here, or its R replaced
	/// while it was out (a 403 on joining it again). Sync off and the token gone, the account, R and the address typed
	/// for it kept, so Sync turned on again asks to join the same account (taken while no new R replaced this one) and
	/// an email sign-in of another joins that; this computer's own key kept, and a rekey's R', which a join that R
	/// no longer opens tries next (its 204 may have been lost).
	pub fn removed(&mut self) {
		let (account, r, address) = (self.account.take(), self.r.take(), self.address.take());
		let (device_secret, next_r) = (self.device_secret.take(), self.next_r.take());
		*self = Self { on: false, account, r, address, device_secret, next_r, ..Self::default() };
	}

	/// The account is gone (`POST /session` answered 404 for the account this computer holds): forgotten with its R,
	/// Sync's switch and this computer's own key kept, so a switch that is on makes a new account at once.
	pub fn gone(&mut self) {
		*self = Self { on: self.on, device_secret: self.device_secret.take(), ..Self::default() };
	}

	/// Whether this computer is in an account: it holds one and a token for it, whatever Sync's switch says.
	#[must_use]
	pub fn held(&self) -> bool {
		self.account.is_some() && self.token.is_some() && self.root().is_some()
	}
}
