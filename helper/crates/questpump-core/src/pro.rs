//! Pro on the computer (docs/plan.md slice 35, "Pro on the computer", and slice 64, "Every device reads it";
//! `contract/companion-api.yaml`, `ProProof`; `contract/sync-api.yaml`, `ProState`): a paired phone's proof that Pro
//! holds on it and the account's token, each checked offline, and the grants the helper keeps, one per phone (the
//! sync's `client`) and the account's one, Pro on while any stands. An iPhone's proof is the App Store's signed
//! transaction and its renewal info, each a JWS (ES256) whose `x5c` chain ends at the pinned Apple Root CA - G3;
//! Android's is Play's purchase text and its RSA signature (PKCS #1 v1.5, SHA-1) under the app's licensing key,
//! `contract/store.json`'s `play.license_key`, whose public operation `crypto-bigint` does (no `rsa` crate:
//! RUSTSEC-2023-0071 has no fixed version). The account's token is the sync server's compact JWS (ES256) under one of
//! `web.pro_token_keys`, the JWKs `contract/store.json` holds (none: every token is ignored), naming the account, the
//! end of its Pro and where it was bought; the phones' cores check it the same way (Swift `AccountPro.swift`, Kotlin
//! `AccountPro.kt`). The bundle id, the package, the Pro product ids and the token keys come from
//! `contract/store.json`, built in. A release build takes Apple's `Production` environment only, a debug build
//! `Sandbox` too (`Rules`), never `Xcode` or `LocalTesting`. No clock or file here: the app passes the time in
//! (milliseconds on the helper's clock) and saves `Grants::to_json` as `FILE`, each phone's proof itself with, for Play,
//! when the helper received it, and the account's token; every start checks them again under the running build's
//! rules, so a debug build's Sandbox grant never unlocks a release build. Nothing here names a proof, a token or an id
//! it carries outside that file.

use std::collections::BTreeMap;
use std::sync::LazyLock;

use base64::Engine as _;
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use const_oid::ObjectIdentifier;
use crypto_bigint::modular::{FixedMontyForm, FixedMontyParams};
use crypto_bigint::{Odd, U64, U2048};
use der::asn1::UintRef;
use der::{Decode as _, Reader as _, SliceReader};
use p256::ecdsa::signature::Verifier as _;
use serde::{Deserialize, Serialize};
use sha1::Sha1;
use sha2::{Digest as _, Sha256};
use spki::SubjectPublicKeyInfoRef;
use x509_cert::Certificate;
use x509_cert::ext::pkix::BasicConstraints;

use crate::link::{Device, ProProof, ProStore};

/// The file the app keeps the grants in, in its data folder.
pub const FILE: &str = "pro.json";

const DAY_MS: i64 = 86_400_000;
/// A grant outlasts what its proof says by a week: a phone that stops linking keeps the computer's extras that long.
const GRACE_MS: i64 = 7 * DAY_MS;
/// Play's signed purchase carries no expiry and never changes across renewals, so a Play grant holds this long after
/// the helper last received the proof (the phones send it again every 24 hours while they link), plus `GRACE_MS`.
const PLAY_FRESH_MS: i64 = 30 * DAY_MS;
/// The skew a certificate's validity is read with at the JWS's `signedDate` (Apple's own libraries' `MAX_SKEW`).
const SKEW_MS: i64 = 60_000;
/// The longest text of a proof (`ProProof`'s `maxLength`): a JWS or Play's purchase text.
const TEXT_MAX: usize = 16_384;
/// The longest Play signature, in base64.
const SIGNATURE_MAX: usize = 1_024;
/// The longest account token (`ProState`'s `token`).
const TOKEN_MAX: usize = 4_096;
/// A P-256 coordinate's length in bytes, and an ES256 signature's (r and s).
const COORDINATE_BYTES: usize = 32;
const ES256_BYTES: usize = 64;

/// Apple Root CA - G3 as <https://www.apple.com/certificateauthority/AppleRootCA-G3.cer> serves it (583 bytes, read
/// 2026-10-08): the one certificate pinned. The intermediate and the leaf come with each proof and are never pinned
/// (Apple rotated the leaf in October 2025).
const APPLE_ROOT: &[u8] = include_bytes!("../certs/AppleRootCA-G3.cer");
/// The root's SHA-256 as Apple publishes it (<https://support.apple.com/en-us/126047>, "Apple Root CA - G3"), checked
/// before the first use: a file that differs pins nothing, and every Apple proof is refused.
const APPLE_ROOT_SHA256: [u8; 32] = [
	0x63, 0x34, 0x3a, 0xbf, 0xb8, 0x9a, 0x6a, 0x03, 0xeb, 0xb5, 0x7e, 0x9b, 0x3f, 0x5f, 0xa7, 0xbe, 0x7c, 0x4f, 0x5c,
	0x75, 0x6f, 0x30, 0x17, 0xb3, 0xa8, 0xc4, 0x88, 0xc3, 0x65, 0x3e, 0x91, 0x79,
];
static PINNED_ROOT: LazyLock<Option<&'static [u8]>> =
	LazyLock::new(|| (Sha256::digest(APPLE_ROOT)[..] == APPLE_ROOT_SHA256).then_some(APPLE_ROOT));

/// Apple's marker extensions: the intermediate's (WWDR) and the leaf's (the App Store's receipt signing).
const APPLE_INTERMEDIATE: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.2.840.113635.100.6.2.1");
const APPLE_LEAF: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.2.840.113635.100.6.11.1");
const EC_PUBLIC_KEY: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.2.840.10045.2.1");
const SECP256R1: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.2.840.10045.3.1.7");
const SECP384R1: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.3.132.0.34");
const ECDSA_WITH_SHA256: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.2.840.10045.4.3.2");
const ECDSA_WITH_SHA384: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.2.840.10045.4.3.3");
const RSA_ENCRYPTION: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.2.840.113549.1.1.1");

/// An RSA-2048 modulus's length in bytes.
const MODULUS_BYTES: usize = 256;
/// The public exponent Play's licensing keys have.
const EXPONENT: [u8; 3] = [0x01, 0x00, 0x01];
/// SHA-1's `DigestInfo` before the digest (RFC 8017 9.2, note 1).
const SHA1_INFO: [u8; 15] = [0x30, 0x21, 0x30, 0x09, 0x06, 0x05, 0x2b, 0x0e, 0x03, 0x02, 0x1a, 0x05, 0x00, 0x04, 0x14];
const SHA1_BYTES: usize = 20;

/// The facts `contract/store.json` holds that the check compares, built in. Each is required: a key renamed or moved
/// there makes the file not read (`STORE`), never a fact silently empty.
#[derive(Deserialize)]
struct Store {
	apple: AppleFacts,
	play: PlayFacts,
	pro: ProFacts,
	web: WebFacts,
}

#[derive(Deserialize)]
struct AppleFacts {
	bundle_id: String,
}

#[derive(Deserialize)]
struct PlayFacts {
	package: String,
	/// The app's licensing key, the base64 of its `SubjectPublicKeyInfo` (Play Console: Monetize with Play,
	/// Monetization setup, Licensing); absent until it is copied there.
	license_key: Option<String>,
}

#[derive(Deserialize)]
struct ProFacts {
	play_product: String,
	plans: Vec<Plan>,
}

#[derive(Deserialize)]
struct Plan {
	apple_product: String,
}

/// Pro on questpump.com (slice 64): Paddle's environment and prices, by plan (none until Pluxia's live Paddle account
/// makes them, which keeps every Get Pro hidden), and the sync server's token: its marks and its keys.
#[derive(Deserialize)]
struct WebFacts {
	paddle: Paddle,
	pro_token: TokenMarks,
	pro_token_keys: Vec<TokenKey>,
}

/// What marks a token as the sync server's Pro token for the devices (`contract/sync-api.yaml`, `ProTokenHeader` and
/// `ProTokenClaims`): the header's `typ` and the claims' `iss` and `aud`, each required exactly.
#[derive(Deserialize)]
struct TokenMarks {
	typ: String,
	iss: String,
	aud: String,
}

#[derive(Deserialize)]
struct Paddle {
	/// `sandbox` or `live`: questpump.com sells only in `live` (a sandbox's checkout charges nobody).
	environment: String,
	prices: BTreeMap<String, Option<String>>,
}

/// One token key as a JWK (RFC 7517) of an ES256 key: P-256, its point's coordinates in base64url.
#[derive(Deserialize)]
struct TokenKey {
	kid: String,
	kty: String,
	crv: String,
	x: String,
	y: String,
}

impl Store {
	/// Whether an App Store product is one of Pro's plans.
	fn apple_pro(&self, product: &str) -> bool {
		self.pro.plans.iter().any(|p| p.apple_product == product)
	}

	/// The first fact that is empty, by its key in the file (a list of no plan counts as an empty `pro.plans`).
	fn empty_fact(&self) -> Option<&'static str> {
		if self.pro.plans.is_empty() {
			return Some("pro.plans");
		}
		let facts = [
			("apple.bundle_id", &self.apple.bundle_id),
			("play.package", &self.play.package),
			("pro.play_product", &self.pro.play_product),
		];
		let plans = self.pro.plans.iter().map(|p| ("an apple_product of pro.plans", &p.apple_product));
		facts.into_iter().chain(plans).find(|(_, fact)| fact.is_empty()).map(|(name, _)| name)
	}
}

/// The built-in facts, or why they check no proof: the file does not read, or a fact is empty. Every proof is then
/// refused (`Refusal::Facts`), and the start's log says why (`facts_problem`).
static STORE: LazyLock<Result<Store, String>> = LazyLock::new(|| {
	let store: Store =
		serde_json::from_str(include_str!("../../../../contract/store.json")).map_err(|e| e.to_string())?;
	match store.empty_fact() {
		Some(name) => Err(format!("{name} is empty")),
		None => Ok(store),
	}
});

static PLAY_KEY: LazyLock<Option<PlayKey>> = LazyLock::new(|| {
	let der = STANDARD.decode(STORE.as_ref().ok()?.play.license_key.as_deref()?).ok()?;
	PlayKey::from_spki(&der)
});

/// Why the built-in facts check no proof, for the log at the start; none when every one is there.
#[must_use]
pub fn facts_problem() -> Option<&'static str> {
	STORE.as_ref().err().map(String::as_str)
}

/// Whether Play's licensing key is built in, for the log at the start: without it every Play proof is refused
/// (`Refusal::Key`).
#[must_use]
pub fn play_key_built_in() -> bool {
	PLAY_KEY.is_some()
}

/// The token keys built in, by their `kid`: each of `web.pro_token_keys` that is a P-256 point (one that is not is left
/// out).
static TOKEN_KEYS: LazyLock<Vec<(String, p256::ecdsa::VerifyingKey)>> = LazyLock::new(|| {
	let Ok(store) = STORE.as_ref() else { return Vec::new() };
	store.web.pro_token_keys.iter().filter_map(|k| Some((k.kid.clone(), k.verifying_key()?))).collect()
});

impl TokenKey {
	/// The key, when it is an EC key on P-256 with both coordinates.
	fn verifying_key(&self) -> Option<p256::ecdsa::VerifyingKey> {
		if self.kty != "EC" || self.crv != "P-256" {
			return None;
		}
		let (x, y) = (URL_SAFE_NO_PAD.decode(&self.x).ok()?, URL_SAFE_NO_PAD.decode(&self.y).ok()?);
		if x.len() != COORDINATE_BYTES || y.len() != COORDINATE_BYTES {
			return None;
		}
		// SEC 1's uncompressed point: 0x04, then x and y
		let point: Vec<u8> = [0x04].into_iter().chain(x).chain(y).collect();
		p256::ecdsa::VerifyingKey::from_sec1_bytes(&point).ok()
	}
}

/// How many account token keys are built in, for the log at the start: with none every account token is ignored
/// (`Refusal::Key`).
#[must_use]
pub fn token_keys_built_in() -> usize {
	TOKEN_KEYS.len()
}

/// Whether questpump.com sells Pro: `web.paddle.environment` is `live` and a price of `web.paddle.prices` is set (Get
/// Pro shows only then).
#[must_use]
pub fn web_prices() -> bool {
	STORE.as_ref().is_ok_and(|s| {
		let paddle = &s.web.paddle;
		paddle.environment == "live" && paddle.prices.values().any(|p| p.as_deref().is_some_and(|p| !p.is_empty()))
	})
}

/// Why a proof grants nothing. The names alone go into the log, never the proof.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
	/// It does not read: a part missing or too long, not a JWS, a claim of the wrong type or a date missing.
	Shape,
	/// Apple's chain does not hold: not three certificates, not the pinned root's, a marker or a date wrong, or no
	/// root pinned.
	Chain,
	/// The JWS's or Play's signature does not check.
	Signature,
	/// Another app's: the bundle id or the package.
	Bundle,
	/// Not a Pro product.
	Product,
	/// Apple's environment is not one this build takes.
	Environment,
	/// Not an auto-renewable subscription.
	Kind,
	/// Not bought by the account (Family Sharing, an organisation's).
	Owner,
	/// Refunded or revoked.
	Revoked,
	/// Upgraded to another subscription: that one's transaction stands for it.
	Upgraded,
	/// Its grant ended already.
	Expired,
	/// Play's purchase is not bought (pending).
	Pending,
	/// Play's subscription is on hold or paused.
	Suspended,
	/// No Play licensing key is built in, or no token key of the account token's `kid`.
	Key,
	/// The built-in store facts do not read or one is empty (`facts_problem`): no proof checks.
	Facts,
	/// The store is not the device's own (an iPhone's Play proof, any watch's proof).
	Device,
	/// The renewal info belongs to another subscription than the transaction.
	Subscription,
	/// The account token names another account than the one the computer is in.
	Account,
	/// The account token's `typ`, `iss` or `aud` is not the Pro token's (`contract/store.json`, `web.pro_token`):
	/// something else the key signed, or a token meant for another audience.
	Audience,
}

impl Refusal {
	/// The store says the subscription does not hold now (it ended, was refunded or upgraded, is pending or on hold),
	/// rather than the computer being unable to confirm it: the computer then says Pro comes with the phone, never that
	/// it could not be confirmed (the Menu's Pro row).
	#[must_use]
	pub fn lapsed(self) -> bool {
		matches!(self, Self::Expired | Self::Revoked | Self::Upgraded | Self::Pending | Self::Suspended)
	}
}

/// The environments a build takes from Apple: always `Production`, and `Sandbox` (App Review, sandbox accounts and
/// Apple's beta testing) only in a debug build; never `Xcode` or `LocalTesting`, which Apple does not sign.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rules {
	pub sandbox: bool,
}

/// A proof's grant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Grant {
	/// Until when Pro holds on the computer, on the helper's clock (ms), the grace week included.
	pub until: i64,
	/// Why an iPhone's renewal info did not count when it came and failed: the transaction alone grants then.
	pub renewal: Option<Refusal>,
}

/// What `Grants::take` came to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Taken {
	/// Checked and kept (or Play's proof received again, which holds 30 days from now): the file is due.
	Kept(Grant),
	/// Apple's proof the grant already rests on, sent again: nothing changed.
	Same,
	/// `none`: the store answered without Pro; whether the phone had a grant (then the file is due).
	Dropped(bool),
	/// Refused: a grant the phone had stays as it was.
	Refused(Refusal),
}

/// Where the account's Pro was bought, as its token names it.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Source {
	AppStore,
	Play,
	Web,
}

/// The account's Pro as its token grants it (slice 64): Swift's and Kotlin's `AccountPro.Grant`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountGrant {
	/// The account the token names.
	pub account: String,
	/// The end of the account's Pro the server knows (ms).
	pub until: i64,
	/// Until when the token holds (ms): `until` and a week of grace, as the server signed it.
	pub ends: i64,
	pub source: Source,
}

/// What `Grants::take_token` came to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenTaken {
	/// Checked and kept in place of the one before: the file is due.
	Kept(AccountGrant),
	/// The token kept already, or no token while the status stands: nothing changed.
	Same,
	/// No token and no status that stands: the account has no Pro; whether one was kept (then the file is due).
	Dropped(bool),
	/// Refused: the kept token stays as it was.
	Refused(Refusal),
}

/// The account's kept token and its grant.
#[derive(Debug, Clone)]
struct AccountHeld {
	token: String,
	grant: AccountGrant,
}

/// `pro.json`: the phones' grants by the sync's `client`, and the account's token (slice 64). A file of slice 35's
/// shape, the phones' map alone, reads as its phones.
#[derive(Serialize, Deserialize)]
struct File<P> {
	phones: BTreeMap<String, P>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	account: Option<String>,
}

/// One phone's grant.
#[derive(Debug, Clone)]
struct Held {
	device: Device,
	proof: ProProof,
	/// When the helper received the proof (ms): Play's grant runs from it.
	taken_at: i64,
	until: i64,
}

/// A grant as `pro.json` keeps it: the proof itself and when it came, never its end, which every start computes again.
#[derive(Serialize, Deserialize)]
struct Saved {
	device: Device,
	proof: ProProof,
	taken_at: i64,
}

/// The phones' grants, by the sync's `client`, and the account's.
#[derive(Debug, Default)]
pub struct Grants {
	held: BTreeMap<String, Held>,
	account: Option<AccountHeld>,
}

impl Grants {
	/// A phone's proof at `now`: `none` drops its grant; one equal to the proof its grant rests on is not checked
	/// again (Play's then holds from now); any other is checked and, when it grants, replaces the phone's grant.
	pub fn take(&mut self, client: &str, device: Device, proof: &ProProof, now: i64, rules: Rules) -> Taken {
		if !store_of(device, proof.store) {
			return Taken::Refused(Refusal::Device);
		}
		if proof.store == ProStore::Lapsed {
			return Taken::Dropped(self.held.remove(client).is_some());
		}
		if let Some(held) = self.held.get_mut(client)
			&& held.device == device
			&& held.proof == *proof
		{
			if proof.store != ProStore::Play {
				return Taken::Same;
			}
			held.taken_at = now;
			held.until = play_until(now);
			return Taken::Kept(Grant { until: held.until, renewal: None });
		}
		match check(device, proof, now, now, rules) {
			Ok(grant) => {
				let held = Held { device, proof: proof.clone(), taken_at: now, until: grant.until };
				self.held.insert(client.to_owned(), held);
				Taken::Kept(grant)
			}
			Err(why) => Taken::Refused(why),
		}
	}

	/// The account's token from the sync server (`GET pro`, with the `status` it answered) at `now`, for `account`, the
	/// one the computer is in: none drops the kept one unless the status stands (`active`, `trialing`, `past_due`),
	/// when the kept one runs out at its own `exp`; one that checks replaces it; one that does not changes nothing.
	pub fn take_token(&mut self, token: Option<&str>, status: Option<&str>, account: &str, now: i64) -> TokenTaken {
		let Some(token) = token else {
			// a grant that stands with no token (the server signs none, say): the kept one runs out at its `exp`
			if matches!(status, Some("active" | "trialing" | "past_due")) {
				return TokenTaken::Same;
			}
			return TokenTaken::Dropped(self.account.take().is_some());
		};
		if self.account.as_ref().is_some_and(|a| a.token == token && a.grant.account == account) {
			return TokenTaken::Same;
		}
		match check_token(token, account, now) {
			Ok(grant) => {
				self.account = Some(AccountHeld { token: token.to_owned(), grant: grant.clone() });
				TokenTaken::Kept(grant)
			}
			Err(why) => TokenTaken::Refused(why),
		}
	}

	/// The account's grant while it stands at `now` for `account`, the one the computer is in (none outside one).
	#[must_use]
	pub fn account_pro(&self, now: i64, account: Option<&str>) -> Option<&AccountGrant> {
		let a = &self.account.as_ref()?.grant;
		(Some(a.account.as_str()) == account && a.ends > now).then_some(a)
	}

	/// Whether Pro holds on the computer at `now`: any phone's grant stands, or the account's for `account`.
	#[must_use]
	pub fn on(&self, now: i64, account: Option<&str>) -> bool {
		self.held.values().any(|h| h.until > now) || self.account_pro(now, account).is_some()
	}

	/// A phone unpaired (`POST /v1/unpair`): its grant goes; whether it had one (then the file is due).
	pub fn forget(&mut self, client: &str) -> bool {
		self.held.remove(client).is_some()
	}

	/// A new pairing code: every phone's grant goes with the pairings (the account's stays); whether any did (then the
	/// file is due).
	pub fn clear(&mut self) -> bool {
		let any = !self.held.is_empty();
		self.held.clear();
		any
	}

	/// Drops the grants that ended by `now`, and the account's token once it ended or names another account than
	/// `account`, the one the computer is in; whether any went (the file is due).
	pub fn prune(&mut self, now: i64, account: Option<&str>) -> bool {
		let before = self.held.len();
		self.held.retain(|_, h| h.until > now);
		let token = self.account.is_some() && self.account_pro(now, account).is_none();
		if token {
			self.account = None;
		}
		token || self.held.len() != before
	}

	/// The grants `pro.json` keeps, each proof checked again at `now` under this build's rules and the account's token
	/// for `account`, the one the computer is in, and how many were dropped (refused or ended; then the file is due).
	/// A file that does not read keeps nothing, and an entry that does not read is dropped alone.
	#[must_use]
	pub fn from_file(bytes: &[u8], now: i64, rules: Rules, account: Option<&str>) -> (Self, usize) {
		let value: serde_json::Value = serde_json::from_slice(bytes).unwrap_or_default();
		let file: File<serde_json::Value> = if value.get("phones").is_some() {
			serde_json::from_value(value).unwrap_or_else(|_| File { phones: BTreeMap::new(), account: None })
		} else {
			File { phones: serde_json::from_value(value).unwrap_or_default(), account: None }
		};
		let mut grants = Self::default();
		let mut dropped = 0;
		if let Some(token) = file.account {
			match account.map(|a| check_token(&token, a, now)) {
				Some(Ok(grant)) => grants.account = Some(AccountHeld { token, grant }),
				_ => dropped += 1,
			}
		}
		for (client, entry) in file.phones {
			let Ok(Saved { device, proof, taken_at }) = serde_json::from_value(entry) else {
				dropped += 1;
				continue;
			};
			match check(device, &proof, taken_at, now, rules) {
				Ok(grant) => {
					grants.held.insert(client, Held { device, proof, taken_at, until: grant.until });
				}
				Err(_) => dropped += 1,
			}
		}
		(grants, dropped)
	}

	/// The file's bytes.
	///
	/// # Errors
	/// Never for these types; serde's error otherwise.
	pub fn to_json(&self) -> serde_json::Result<Vec<u8>> {
		let phones: BTreeMap<String, Saved> = self
			.held
			.iter()
			.map(|(client, h)| {
				(client.clone(), Saved { device: h.device, proof: h.proof.clone(), taken_at: h.taken_at })
			})
			.collect();
		serde_json::to_vec(&File { phones, account: self.account.as_ref().map(|a| a.token.clone()) })
	}
}

/// Whether a proof's store can come from the device: the iPhone's App Store, Android's Play, `none` from either phone.
fn store_of(device: Device, store: ProStore) -> bool {
	matches!(
		(device, store),
		(Device::Ios, ProStore::AppStore | ProStore::Lapsed) | (Device::Android, ProStore::Play | ProStore::Lapsed)
	)
}

fn play_until(taken_at: i64) -> i64 {
	taken_at.saturating_add(PLAY_FRESH_MS + GRACE_MS)
}

/// A phone's proof (not `none`) checked at `now` against the built-in facts and the pinned root or the built-in key, a
/// Play proof received at `taken_at`.
fn check(device: Device, proof: &ProProof, taken_at: i64, now: i64, rules: Rules) -> Result<Grant, Refusal> {
	let store = STORE.as_ref().map_err(|_| Refusal::Facts)?;
	match (device, proof.store) {
		(Device::Ios, ProStore::AppStore) => {
			let root = (*PINNED_ROOT).ok_or(Refusal::Chain)?;
			let transaction = proof.transaction.as_deref().ok_or(Refusal::Shape)?;
			verify_app_store(root, store, transaction, proof.renewal.as_deref(), now, rules)
		}
		(Device::Android, ProStore::Play) => {
			let (Some(purchase), Some(signature)) = (proof.purchase.as_deref(), proof.signature.as_deref()) else {
				return Err(Refusal::Shape);
			};
			let key = PLAY_KEY.as_ref().ok_or(Refusal::Key)?;
			verify_play(key, store, purchase, signature, taken_at, now)
		}
		_ => Err(Refusal::Device),
	}
}

/// An iPhone's signed transaction, and its renewal info when it came, checked against `root` and `store` at `now`: the
/// chain and the signature, then the environment and the claims. The grant lasts until the later of the transaction's
/// `expiresDate` and a checked renewal's `gracePeriodExpiresDate` (Apple's billing grace), plus `GRACE_MS`.
fn verify_app_store(
	root: &[u8],
	store: &Store,
	transaction: &str,
	renewal: Option<&str>,
	now: i64,
	rules: Rules,
) -> Result<Grant, Refusal> {
	let t = verify_jws(root, transaction)?;
	if !takes(t.environment.as_deref(), rules) {
		return Err(Refusal::Environment);
	}
	if t.bundle_id.as_deref() != Some(store.apple.bundle_id.as_str()) {
		return Err(Refusal::Bundle);
	}
	if !t.product_id.as_deref().is_some_and(|p| store.apple_pro(p)) {
		return Err(Refusal::Product);
	}
	if t.kind.as_deref() != Some("Auto-Renewable Subscription") {
		return Err(Refusal::Kind);
	}
	if t.in_app_ownership_type.as_deref() != Some("PURCHASED") {
		return Err(Refusal::Owner);
	}
	if t.revocation_date.is_some() {
		return Err(Refusal::Revoked);
	}
	if t.is_upgraded == Some(true) {
		return Err(Refusal::Upgraded);
	}
	let expires = t.expires_date.ok_or(Refusal::Shape)?;
	let (grace, renewal) = match renewal.map(|r| grace_end(root, store, r, &t)) {
		Some(Ok(grace)) => (grace, None),
		Some(Err(why)) => (None, Some(why)),
		None => (None, None),
	};
	let until = expires.max(grace.unwrap_or(expires)).saturating_add(GRACE_MS);
	if until <= now {
		return Err(Refusal::Expired);
	}
	Ok(Grant { until, renewal })
}

/// The end of the billing grace a renewal info names (none outside a grace), once it checks as the same subscription
/// as the transaction `t`: its chain and signature, the same environment and original transaction, a Pro product.
fn grace_end(root: &[u8], store: &Store, renewal: &str, t: &Claims) -> Result<Option<i64>, Refusal> {
	let r = verify_jws(root, renewal)?;
	if r.environment != t.environment {
		return Err(Refusal::Environment);
	}
	if r.original_transaction_id.is_none() || r.original_transaction_id != t.original_transaction_id {
		return Err(Refusal::Subscription);
	}
	if !r.product_id.as_deref().is_some_and(|p| store.apple_pro(p)) {
		return Err(Refusal::Product);
	}
	Ok(r.grace_period_expires_date)
}

/// Whether a build takes Apple's environment.
fn takes(environment: Option<&str>, rules: Rules) -> bool {
	match environment {
		Some("Production") => true,
		Some("Sandbox") => rules.sandbox,
		_ => false,
	}
}

/// The claims of an App Store transaction and of a renewal info this check reads (Apple's
/// `JWSTransactionDecodedPayload` and `JWSRenewalInfoDecodedPayload`); the dates are unix milliseconds.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Claims {
	signed_date: Option<i64>,
	environment: Option<String>,
	bundle_id: Option<String>,
	product_id: Option<String>,
	#[serde(rename = "type")]
	kind: Option<String>,
	in_app_ownership_type: Option<String>,
	revocation_date: Option<i64>,
	is_upgraded: Option<bool>,
	expires_date: Option<i64>,
	original_transaction_id: Option<String>,
	grace_period_expires_date: Option<i64>,
}

#[derive(Deserialize)]
struct Header {
	alg: String,
	x5c: Vec<String>,
}

/// A compact JWS's claims once its chain holds at its own `signedDate` and its ES256 signature covers its header and
/// payload.
fn verify_jws(root: &[u8], jws: &str) -> Result<Claims, Refusal> {
	if jws.len() > TEXT_MAX {
		return Err(Refusal::Shape);
	}
	let mut parts = jws.split('.');
	let (Some(header), Some(payload), Some(signature), None) = (parts.next(), parts.next(), parts.next(), parts.next())
	else {
		return Err(Refusal::Shape);
	};
	let decode = |part: &str| URL_SAFE_NO_PAD.decode(part).map_err(|_| Refusal::Shape);
	let head: Header = serde_json::from_slice(&decode(header)?).map_err(|_| Refusal::Shape)?;
	if head.alg != "ES256" {
		return Err(Refusal::Signature);
	}
	let claims: Claims = serde_json::from_slice(&decode(payload)?).map_err(|_| Refusal::Shape)?;
	let signed = claims.signed_date.ok_or(Refusal::Shape)?;
	let key = leaf_key(root, &head.x5c, signed)?;
	let signature = p256::ecdsa::Signature::from_slice(&decode(signature)?).map_err(|_| Refusal::Signature)?;
	let input = format!("{header}.{payload}");
	key.verify(input.as_bytes(), &signature).map_err(|_| Refusal::Signature)?;
	Ok(claims)
}

/// One certificate, read, and the exact bytes its issuer signed (its `tbsCertificate` as it came, never encoded
/// again).
struct Cert<'a> {
	cert: Certificate,
	tbs: &'a [u8],
}

impl<'a> Cert<'a> {
	fn read(der: &'a [u8]) -> Result<Self, Refusal> {
		let cert = Certificate::from_der(der).map_err(|_| Refusal::Chain)?;
		let mut reader = SliceReader::new(der).map_err(|_| Refusal::Chain)?;
		let tbs = reader
			.sequence(|r| {
				let tbs = r.tlv_bytes()?;
				r.tlv_bytes()?; // signatureAlgorithm
				r.tlv_bytes()?; // signatureValue
				Ok::<_, der::Error>(tbs)
			})
			.map_err(|_| Refusal::Chain)?;
		reader.finish().map_err(|_| Refusal::Chain)?;
		Ok(Self { cert, tbs })
	}

	/// The certificate's key, on P-256 or P-384.
	fn key(&self) -> Result<Key, Refusal> {
		let spki = self.cert.tbs_certificate().subject_public_key_info();
		if spki.algorithm.oid != EC_PUBLIC_KEY {
			return Err(Refusal::Chain);
		}
		let curve = spki.algorithm.parameters.as_ref().and_then(|p| p.decode_as::<ObjectIdentifier>().ok());
		let point = spki.subject_public_key.as_bytes().ok_or(Refusal::Chain)?;
		let key = match curve {
			Some(SECP256R1) => p256::ecdsa::VerifyingKey::from_sec1_bytes(point).ok().map(Key::P256),
			Some(SECP384R1) => p384::ecdsa::VerifyingKey::from_sec1_bytes(point).ok().map(Key::P384),
			_ => None,
		};
		key.ok_or(Refusal::Chain)
	}

	/// Whether `issuer` (whose key is `key`) issued this certificate: the issuer's name, and its signature over the
	/// exact TBS bytes, ecdsa-with-SHA384 under a P-384 key or ecdsa-with-SHA256 under a P-256 key, no other pair.
	fn issued_by(&self, issuer: &Cert, key: &Key) -> bool {
		let tbs = self.cert.tbs_certificate();
		let algorithm = self.cert.signature_algorithm();
		if tbs.issuer() != issuer.cert.tbs_certificate().subject()
			|| algorithm.parameters.is_some()
			|| tbs.signature() != algorithm
		{
			return false;
		}
		let Some(signature) = self.cert.signature().as_bytes() else { return false };
		match key {
			Key::P384(key) if algorithm.oid == ECDSA_WITH_SHA384 => {
				p384::ecdsa::DerSignature::from_bytes(signature).is_ok_and(|s| key.verify(self.tbs, &s).is_ok())
			}
			Key::P256(key) if algorithm.oid == ECDSA_WITH_SHA256 => {
				p256::ecdsa::DerSignature::from_bytes(signature).is_ok_and(|s| key.verify(self.tbs, &s).is_ok())
			}
			_ => false,
		}
	}

	fn marked(&self, oid: ObjectIdentifier) -> bool {
		self.cert.tbs_certificate().extensions().is_some_and(|e| e.iter().any(|x| x.extn_id == oid))
	}

	fn is_ca(&self) -> bool {
		matches!(self.cert.tbs_certificate().get_extension::<BasicConstraints>(), Ok(Some((_, b))) if b.ca)
	}

	/// Whether the validity covers `at` (ms) within `SKEW_MS`.
	fn valid_at(&self, at: i64) -> bool {
		let validity = self.cert.tbs_certificate().validity();
		let ms = |t: x509_cert::time::Time| i64::try_from(t.to_unix_duration().as_millis()).unwrap_or(i64::MAX);
		ms(validity.not_before) <= at.saturating_add(SKEW_MS) && at.saturating_sub(SKEW_MS) <= ms(validity.not_after)
	}
}

enum Key {
	P256(p256::ecdsa::VerifyingKey),
	P384(p384::ecdsa::VerifyingKey),
}

/// The leaf's key of an `x5c` chain that holds at `at` under `root`: exactly three certificates (the leaf, Apple's
/// intermediate, a root, whose place the pinned one takes); the intermediate issued by the root, a CA with Apple's
/// intermediate marker; the leaf issued by the intermediate with Apple's leaf marker and a P-256 key; every one valid
/// at `at`.
fn leaf_key(root: &[u8], x5c: &[String], at: i64) -> Result<p256::ecdsa::VerifyingKey, Refusal> {
	let [leaf, intermediate, _] = x5c else { return Err(Refusal::Chain) };
	let der = |text: &str| STANDARD.decode(text).map_err(|_| Refusal::Chain);
	let (leaf_der, intermediate_der) = (der(leaf)?, der(intermediate)?);
	let (root, intermediate, leaf) = (Cert::read(root)?, Cert::read(&intermediate_der)?, Cert::read(&leaf_der)?);
	let intermediate_key = intermediate.key()?;
	let holds = intermediate.issued_by(&root, &root.key()?)
		&& intermediate.is_ca()
		&& intermediate.marked(APPLE_INTERMEDIATE)
		&& leaf.issued_by(&intermediate, &intermediate_key)
		&& leaf.marked(APPLE_LEAF)
		&& [&root, &intermediate, &leaf].iter().all(|c| c.valid_at(at));
	match leaf.key()? {
		Key::P256(key) if holds => Ok(key),
		_ => Err(Refusal::Chain),
	}
}

/// Play's licensing key: an RSA-2048 public key with the exponent 65537, as Play Console gives every app.
struct PlayKey {
	n: U2048,
	params: FixedMontyParams<{ U2048::LIMBS }>,
}

impl PlayKey {
	/// The key from its `SubjectPublicKeyInfo` (DER); none when that is no RSA-2048 key with e = 65537.
	fn from_spki(der: &[u8]) -> Option<Self> {
		let spki = SubjectPublicKeyInfoRef::from_der(der).ok()?;
		// RFC 3279 2.3.1: the parameters are NULL
		if spki.algorithm.oid != RSA_ENCRYPTION || !spki.algorithm.parameters.is_some_and(der::AnyRef::is_null) {
			return None;
		}
		let mut reader = SliceReader::new(spki.subject_public_key.as_bytes()?).ok()?;
		let (n, e) = reader.sequence(|r| Ok::<_, der::Error>((UintRef::decode(r)?, UintRef::decode(r)?))).ok()?;
		reader.finish().ok()?;
		let n = n.as_bytes();
		if e.as_bytes() != EXPONENT || n.len() != MODULUS_BYTES || n.first().is_none_or(|b| b & 0x80 == 0) {
			return None;
		}
		let n = U2048::from_be_slice(n);
		let params = FixedMontyParams::new_vartime(Odd::new(n).into_option()?);
		Some(Self { n, params })
	}

	/// Whether `signature` is this key's RSASSA-PKCS1-v1_5 signature with SHA-1 over `message` (RFC 8017 8.2.2): the
	/// public operation's result compared whole with the encoding of the message's digest, nothing parsed. Every value
	/// here is public, so the variable-time exponentiation leaks nothing.
	fn verifies(&self, message: &[u8], signature: &[u8]) -> bool {
		if signature.len() != MODULUS_BYTES {
			return false;
		}
		let s = U2048::from_be_slice(signature);
		if s >= self.n {
			return false;
		}
		let m = FixedMontyForm::new(&s, &self.params).pow_vartime(&U64::from_u32(65_537)).retrieve();
		// EM = 0x00 || 0x01 || 0xFF.. || 0x00 || DigestInfo || SHA-1(message)
		let mut expected = [0xff_u8; MODULUS_BYTES];
		let info = MODULUS_BYTES - SHA1_BYTES - SHA1_INFO.len();
		expected[0] = 0x00;
		expected[1] = 0x01;
		expected[info - 1] = 0x00;
		expected[info..MODULUS_BYTES - SHA1_BYTES].copy_from_slice(&SHA1_INFO);
		expected[MODULUS_BYTES - SHA1_BYTES..].copy_from_slice(&Sha1::digest(message));
		m.to_be_bytes().as_ref() == expected.as_slice()
	}
}

/// The keys of Play's purchase JSON this check reads (Play Billing's `Purchase.getOriginalJson()`).
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Purchase {
	package_name: Option<String>,
	product_id: Option<String>,
	#[serde(default)]
	product_ids: Vec<String>,
	/// `purchaseState`: 0 for a bought purchase in the raw JSON (the library's `PURCHASED` constant is 1), 4 while
	/// pending.
	#[serde(rename = "purchaseState")]
	state: Option<i64>,
	/// `suspended`: true while the subscription is paused or on hold after a declined payment. Google's documentation
	/// names no key of this JSON (the `Purchase` reference, read 2026-10-31, documents only `isSuspended()`); the key is
	/// the one Play Billing 9.1.0's `Purchase.isSuspended()` reads (`optBoolean("suspended")`, the class read
	/// 2026-10-31), and Play lists a suspended subscription only to an app that asks with `includeSuspendedSubscriptions`
	/// (8.1.0's release notes), which the phone does not, so this is a second guard.
	suspended: Option<bool>,
}

/// An Android phone's purchase text and its signature (standard base64) under `key`, received at `taken_at` and
/// checked against `store` at `now`: the signature over the text's exact bytes first, then the package, the Pro
/// product, bought and not suspended. The grant lasts `PLAY_FRESH_MS` from the receipt, plus `GRACE_MS`.
fn verify_play(
	key: &PlayKey,
	store: &Store,
	purchase: &str,
	signature: &str,
	taken_at: i64,
	now: i64,
) -> Result<Grant, Refusal> {
	if purchase.len() > TEXT_MAX || signature.len() > SIGNATURE_MAX {
		return Err(Refusal::Shape);
	}
	let signature = STANDARD.decode(signature).map_err(|_| Refusal::Signature)?;
	if !key.verifies(purchase.as_bytes(), &signature) {
		return Err(Refusal::Signature);
	}
	let p: Purchase = serde_json::from_str(purchase).map_err(|_| Refusal::Shape)?;
	if p.package_name.as_deref() != Some(store.play.package.as_str()) {
		return Err(Refusal::Bundle);
	}
	let product = &store.pro.play_product;
	if p.product_id.as_ref() != Some(product) && !p.product_ids.contains(product) {
		return Err(Refusal::Product);
	}
	if p.state != Some(0) {
		return Err(Refusal::Pending);
	}
	if p.suspended == Some(true) {
		return Err(Refusal::Suspended);
	}
	let until = play_until(taken_at);
	if until <= now {
		return Err(Refusal::Expired);
	}
	Ok(Grant { until, renewal: None })
}

/// The header of an account token (contract/sync-api.yaml, `ProTokenHeader`).
#[derive(Deserialize)]
struct TokenHeader {
	alg: String,
	kid: String,
	typ: String,
}

/// The claims of an account token (contract/sync-api.yaml, `ProTokenClaims`); the dates are unix seconds.
#[derive(Deserialize)]
struct TokenClaims {
	iss: String,
	aud: String,
	account: String,
	until: i64,
	source: Source,
	/// When the server signed it: read so a token without it does not check, as the phones' cores read it.
	#[serde(rename = "iat")]
	_iat: i64,
	exp: i64,
}

/// The sync server's token for `account` checked at `now` against the built-in token keys (slice 64; Swift's and
/// Kotlin's `AccountPro.check`): three base64url parts; a header of ES256, the `kid` of a built-in key and a `typ`; an
/// ES256 signature (r and s, 64 bytes) over the header and the payload as they came; the claims `iss`, `aud`,
/// `account` (the computer's own), `until`, `source`, `iat` and `exp`, each of its type; `typ`, `iss` and `aud`
/// exactly `web.pro_token`'s; it holds until `exp`.
///
/// # Errors
/// Why it grants nothing: `Shape`, `Key` (no built-in key of its `kid`), `Signature`, `Audience`, `Account` or
/// `Expired`.
pub fn check_token(token: &str, account: &str, now: i64) -> Result<AccountGrant, Refusal> {
	if token.len() > TOKEN_MAX {
		return Err(Refusal::Shape);
	}
	let mut parts = token.split('.');
	let (Some(header), Some(payload), Some(signature), None) = (parts.next(), parts.next(), parts.next(), parts.next())
	else {
		return Err(Refusal::Shape);
	};
	let decode = |part: &str| URL_SAFE_NO_PAD.decode(part).map_err(|_| Refusal::Shape);
	let head: TokenHeader = serde_json::from_slice(&decode(header)?).map_err(|_| Refusal::Shape)?;
	if head.alg != "ES256" {
		return Err(Refusal::Signature);
	}
	let key = TOKEN_KEYS.iter().find(|(kid, _)| *kid == head.kid).map(|(_, key)| key).ok_or(Refusal::Key)?;
	let signature = decode(signature)?;
	if signature.len() != ES256_BYTES {
		return Err(Refusal::Signature);
	}
	let signature = p256::ecdsa::Signature::from_slice(&signature).map_err(|_| Refusal::Signature)?;
	key.verify(format!("{header}.{payload}").as_bytes(), &signature).map_err(|_| Refusal::Signature)?;
	let claims: TokenClaims = serde_json::from_slice(&decode(payload)?).map_err(|_| Refusal::Shape)?;
	let marks = &STORE.as_ref().map_err(|_| Refusal::Facts)?.web.pro_token;
	if head.typ != marks.typ || claims.iss != marks.iss || claims.aud != marks.aud {
		return Err(Refusal::Audience);
	}
	if claims.account != account {
		return Err(Refusal::Account);
	}
	let ms = |s: i64| s.saturating_mul(1_000);
	let ends = ms(claims.exp);
	if ends <= now {
		return Err(Refusal::Expired);
	}
	Ok(AccountGrant { account: claims.account, until: ms(claims.until), ends, source: claims.source })
}
