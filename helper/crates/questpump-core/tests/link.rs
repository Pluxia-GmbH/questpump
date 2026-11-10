//! The link to the phone and the watch, end to end through questpump-core's public API: the keys, the route-bound
//! seal and the hello key against `contract/fixtures/seal.json`, the hello's proofs and every body against
//! `contract/fixtures/wire.json`, and the command queue, the holder and the delivery statistics as the helper's held
//! `POST /v1/sync` uses them.

#![expect(clippy::unwrap_used, reason = "a fixture that does not read fails the test loudly")]

use std::assert_matches;
use std::net::IpAddr;
use std::path::PathBuf;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use questpump_core::codes::{PumpEnd, StripEvent, StripState};
use questpump_core::link::{
	self, AccountRequest, AccountResponse, AppRowsResponse, BodyRequest, BodyResponse, Device, EventType,
	HelloResponse, HistoryResponse, Link, PairRequest, PairResponse, PingResponse, Received, RefusedResponse,
	SheetResponse, SheetsResponse, SyncRequest, SyncResponse, Throttle, Via, WristEvent,
};
use questpump_core::protocol::{Command, Source};
use questpump_core::seal::{Keys, Replay, Route, SealError, Way};
use questpump_core::session::{CatchupMove, Now, People, Session, Zone};
use questpump_core::store::Store;
use questpump_core::strip::{self, Body};
use questpump_core::sync::{self, KeyPair, LinkCode, Opened, SyncKeys};
use questpump_core::verdict::Verdict;
use serde_json::Value as J;

fn fixture(name: &str) -> J {
	let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../contract/fixtures").join(name);
	serde_json::from_str(&std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))).unwrap()
}

fn unhex(s: &str) -> Vec<u8> {
	(0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap()).collect()
}

fn keys() -> Keys {
	Keys::derive(&unhex(fixture("seal.json")["key_hex"].as_str().unwrap()).try_into().unwrap())
}

/// A seal.json case's route: its way, method and path.
fn route(case: &J) -> Route<'_> {
	let way = match case["way"].as_str().unwrap() {
		"request" => Way::Request,
		"answer" => Way::Answer,
		other => panic!("no way {other}"),
	};
	Route { way, method: case["method"].as_str().unwrap(), path: case["path"].as_str().unwrap() }
}

#[test]
fn the_keys_come_from_the_pairing_key_and_the_seal_opens_what_every_app_seals() {
	let f = fixture("seal.json");
	let keys = keys();
	assert_eq!(keys.bearer, f["bearer"].as_str().unwrap());
	assert_eq!(keys.hello.to_vec(), unhex(f["hello_key_hex"].as_str().unwrap()), "the hello key");
	assert_ne!(Keys::derive(&[7; 32]).hello, keys.hello, "a new pairing code proves a new key");
	assert!(keys.authorizes(Some(&format!("Bearer {}", keys.bearer))));
	for header in
		[None, Some("Bearer"), Some(keys.bearer.as_str()), Some("Bearer p52Eui3lCNZb-pa8A_Ebh7j8vJ2V3rlnLanU5B80h7")]
	{
		assert!(!keys.authorizes(header), "{header:?} is not this helper's bearer");
	}
	for case in f["sealed"].as_array().unwrap() {
		let (route, about) = (route(case), &case["about"]);
		assert_eq!(String::from_utf8(route.aad()).unwrap(), case["aad"].as_str().unwrap(), "{about}");
		let sealed = STANDARD.decode(case["sealed_b64"].as_str().unwrap()).unwrap();
		let nonce: [u8; 12] = unhex(case["nonce_hex"].as_str().unwrap()).try_into().unwrap();
		let (plain, seen) = keys.seal.open(&route, &sealed).unwrap();
		assert_eq!((String::from_utf8(plain).unwrap(), seen), (case["plain"].as_str().unwrap().to_owned(), nonce));
		assert_eq!(keys.seal.seal_with(nonce, &route, case["plain"].as_str().unwrap().as_bytes()), sealed, "{about}");
	}
	for case in f["refused"].as_array().unwrap() {
		let sealed = STANDARD.decode(case["sealed_b64"].as_str().unwrap()).unwrap();
		let expected = if sealed.len() < 28 { SealError::Short } else { SealError::Tag };
		assert_eq!(keys.seal.open(&route(case), &sealed).err(), Some(expected), "{}", case["about"]);
	}
	let (sync, ping) = (
		Route { way: Way::Request, method: "POST", path: "/v1/sync" },
		Route { way: Way::Answer, method: "POST", path: "/v1/sync" },
	);
	let one = keys.seal.seal(&sync, b"{}");
	let two = keys.seal.seal(&sync, b"{}");
	assert_ne!(one[..12], two[..12], "a fresh nonce every time");
	assert_eq!(keys.seal.open(&sync, &two).unwrap().0, b"{}");
	assert_eq!(keys.seal.open(&ping, &two).err(), Some(SealError::Tag), "a request never opens as an answer");
}

/// The sync's sealed 422 (seal.json's `refusals`): the helper seals the refused request's nonce, byte for byte as the
/// fixture's refusal, which the phones take as that request's refusal and no other.
#[test]
fn a_body_that_does_not_read_is_refused_under_the_seal_naming_its_request() {
	let f = &fixture("seal.json")["refusals"];
	let keys = keys();
	let request = STANDARD.decode(f["request_b64"].as_str().unwrap()).unwrap();
	let (_, nonce) = keys.seal.open(&Route { way: Way::Request, method: "POST", path: "/v1/sync" }, &request).unwrap();
	let refused = &f["refused"];
	let answer = Route { way: Way::Answer, method: "POST", path: "/v1/sync" };
	assert_eq!(String::from_utf8(answer.aad()).unwrap(), refused["aad"].as_str().unwrap());
	let plain = serde_json::to_vec(&RefusedResponse::of(nonce)).unwrap();
	assert_eq!(String::from_utf8(plain.clone()).unwrap(), refused["plain"].as_str().unwrap());
	let at: [u8; 12] = unhex(refused["nonce_hex"].as_str().unwrap()).try_into().unwrap();
	let sealed = STANDARD.decode(refused["sealed_b64"].as_str().unwrap()).unwrap();
	assert_eq!(keys.seal.seal_with(at, &answer, &plain), sealed);
}

/// The LAN hello (wire.json's first `hello_response`, find.json's rule): over the device's challenge and the answer's
/// own bytes, the proofs of the install's id and of the pairing key, byte for byte as every app checks them.
#[test]
fn the_hello_proves_the_install_and_the_key_over_the_devices_challenge() {
	let wire = fixture("wire.json");
	let case = wire["bodies"].as_array().unwrap().iter().find(|b| b["kind"] == "hello_response").unwrap();
	let hid = "AAECAwQFBgcICQoLDA0ODw";
	let run = |from: u8| -> [u8; 16] { std::array::from_fn(|i| from + u8::try_from(i).unwrap()) };
	let (c, n) = (run(0xc0), run(0xd0));
	let hello = HelloResponse::of(hid, &keys().hello, &c, &n).unwrap();
	assert_eq!(serde_json::to_value(&hello).unwrap(), case["json"], "{}", case["about"]);
	let other = HelloResponse::of(hid, &Keys::derive(&[7; 32]).hello, &c, &n).unwrap();
	assert_eq!((other.h.as_str(), other.n.as_str()), (hello.h.as_str(), hello.n.as_str()), "the same install");
	assert_ne!(other.k, hello.k, "another key proves nothing of this one");
	assert_ne!(HelloResponse::of(hid, &keys().hello, &n, &n).unwrap().k, hello.k, "another challenge");
	assert!(HelloResponse::of("not*base64", &keys().hello, &c, &n).is_none(), "no install id to prove");
}

/// A sealed request but the sync's names every field it takes: one it does not know does not read, so nothing rides a
/// route unseen; the sync's takes one, since a newer app adds fields and events to it.
#[test]
fn a_request_with_a_field_it_does_not_know_does_not_read_but_the_syncs_does() {
	let extra = |kind: &str| {
		let wire = fixture("wire.json");
		let mut json = wire["bodies"].as_array().unwrap().iter().find(|b| b["kind"] == kind).unwrap()["json"].clone();
		json["extra"] = J::Bool(true);
		json
	};
	assert!(serde_json::from_value::<PairRequest>(extra("pair_request")).is_err());
	assert!(serde_json::from_value::<BodyRequest>(extra("body_request")).is_err());
	assert!(serde_json::from_value::<AccountRequest>(extra("account_request")).is_err());
	let unpair = serde_json::json!({"device": "ios", "client": "c", "sent_at": 1, "extra": true});
	assert!(serde_json::from_value::<link::UnpairRequest>(unpair).is_err());
	assert!(serde_json::from_value::<SyncRequest>(extra("sync_request")).is_ok(), "a newer app's sync reads");
}

#[test]
fn a_body_sent_again_is_refused_for_as_long_as_its_sent_at_could_pass() {
	let mut replay = Replay::new(link::REPLAY_MS);
	assert!(replay.fresh([1; 12], 0));
	assert!(!replay.fresh([1; 12], 1_000), "the same request replayed on the LAN");
	assert!(replay.fresh([2; 12], link::REPLAY_MS));
	assert!(!replay.fresh([2; 12], link::REPLAY_MS + 1), "still in the window");
	assert!(replay.fresh([1; 12], link::REPLAY_MS + 1), "past the window, where its sent_at is refused anyway");
}

/// The sync's keys, batch, link answer and codes (seal.json's `sync` block): every value derived from R as the phones
/// derive it, the batch opened, inflated and sealed again to the same bytes, the link answer the phones seal with their
/// fresh key opened with the request's (the same seal and check code from either side), and the restore, link and
/// request codes read and written as given, a request with an older or broken key read as none.
#[test]
fn the_sync_keys_come_from_r_and_its_batches_codes_and_link_answer_read_as_every_app_writes_them() {
	use sha2::Digest as _;
	let f = &fixture("seal.json")["sync"];
	let hex = |key: &str| unhex(f[key].as_str().unwrap());
	let text = |key: &str| f[key].as_str().unwrap().to_owned();
	let r: [u8; 16] = hex("R_hex").try_into().unwrap();
	let keys = SyncKeys::derive(&r);
	assert_eq!(keys.auth, text("auth"));
	assert_eq!(keys.kid, text("kid"));
	// the server keeps the SHA-256 of the auth key's 32 bytes
	let auth_hash = sha2::Sha256::digest(base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(&keys.auth).unwrap());
	assert_eq!(auth_hash.to_vec(), hex("auth_sha256_hex"));
	// the seal key: what it seals under a nonce is what the fixture's key seals
	let fixture_seal = questpump_core::seal::Sealer::new(&hex("seal_key_hex").try_into().unwrap());

	let batch = &f["batch"];
	let nonce: [u8; 12] = unhex(batch["nonce_hex"].as_str().unwrap()).try_into().unwrap();
	let sealed = STANDARD.decode(batch["sealed_b64"].as_str().unwrap()).unwrap();
	let deflated = STANDARD.decode(batch["deflated_b64"].as_str().unwrap()).unwrap();
	let plain = batch["plain"].as_str().unwrap();
	let (opened, seen) = keys.seal.open(&sealed).unwrap();
	assert_eq!((opened.as_slice(), seen), (deflated.as_slice(), nonce));
	assert_eq!(String::from_utf8(sync::inflate(&deflated).unwrap()).unwrap(), plain, "raw DEFLATE, no zlib header");
	assert_eq!(keys.seal.seal_with(nonce, &deflated), sealed);
	assert_eq!(fixture_seal.seal_with(nonce, &deflated), sealed, "the seal key");
	let read: J = serde_json::from_str(plain).unwrap();
	let (account, device, seq) =
		(read["account"].as_str().unwrap(), read["device"].as_str().unwrap(), read["seq"].as_u64().unwrap());
	assert_eq!(account, text("account"));
	let Opened::Read(back) = sync::open_batch(&keys, &sealed, account, device, seq).unwrap() else {
		panic!("a batch of this version reads")
	};
	assert_eq!(serde_json::to_value(&back).unwrap(), read, "the batch's fields as the contract lists them");
	assert!(sync::open_batch(&keys, &sealed, account, device, seq + 1).is_err(), "a seq the listing does not name");
	assert!(sync::open_batch(&keys, &sealed, account, "AAAAAAAAAAAAAAAAAAAAAA", seq).is_err(), "another device's");
	let ours = sync::seal_batch(&keys, &back).unwrap();
	assert_eq!(sync::open_batch(&keys, &ours, account, device, seq).unwrap(), Opened::Read(back));

	let request = KeyPair::from_bytes(&hex("request_secret_hex").try_into().unwrap()).unwrap();
	assert_eq!(request.public().to_vec(), hex("request_public_hex"), "P, compressed");
	let link = &f["link"];
	let link_hex = |key: &str| unhex(link[key].as_str().unwrap());
	let answerer = KeyPair::from_bytes(&link_hex("ephemeral_secret_hex").try_into().unwrap()).unwrap();
	let ephemeral = link_hex("ephemeral_public_hex");
	let nonce: [u8; 12] = link_hex("nonce_hex").try_into().unwrap();
	let sealed = STANDARD.decode(link["sealed_b64"].as_str().unwrap()).unwrap();
	let plain = link["plain"].as_str().unwrap();
	let (head, body) = sealed.split_at(sync::PUBLIC_KEY);
	assert_eq!(head, ephemeral.as_slice(), "E, uncompressed, first");
	let link_seal = questpump_core::seal::Sealer::new(&hex("link_key_hex").try_into().unwrap());
	assert_eq!(link_seal.seal_with(nonce, plain.as_bytes()), body, "the link key");
	// ECDH(e, P) is ECDH(d, E): the phone's seal is the one the computer opens, its check code the computer's
	let (phone, computer) = (answerer.agree(&request.public()).unwrap(), request.agree(&ephemeral).unwrap());
	assert_eq!(phone.seal.seal_with(nonce, plain.as_bytes()), body);
	assert_eq!(computer.seal.open(body).unwrap(), (plain.as_bytes().to_vec(), nonce));
	let check = link["check"].as_str().unwrap();
	assert_eq!(
		u32::from_be_bytes(link_hex("check_hex").try_into().unwrap()) % 1_000_000,
		check.parse::<u32>().unwrap()
	);
	assert_eq!((phone.check.as_str(), computer.check.as_str()), (check, check), "the check code, six digits");
	let account_id: [u8; 16] = sync::bytes(&text("account")).unwrap();
	let opened = sync::open_link_answer(&request, &sealed).unwrap();
	assert_eq!((opened.account, opened.r, opened.check.as_str()), (account_id, r, check));
	assert!(sync::open_link_answer(&answerer, &sealed).is_err(), "with another request's key");
	assert!(sync::open_link_answer(&request, body).is_err(), "without the answerer's key");
	assert!(KeyPair::from_bytes(&[0; 32]).is_none(), "zero is no secret");

	assert_eq!(sync::restore_code(&r), text("restore"));
	assert_eq!(sync::read_restore(&text("restore")), Ok(r));
	assert_eq!(sync::read_restore(&text("restore").to_lowercase().replace('-', " ")), Ok(r), "case-blind, spaces");
	assert_eq!(sync::read_restore(&text("restore").replace('0', "O").replace('1', "l")), Ok(r), "O as 0, L as 1");
	let mut off = text("restore");
	off.replace_range(0..1, if off.starts_with('A') { "B" } else { "A" });
	assert!(sync::read_restore(&off).is_err(), "a character off");
	assert!(sync::read_restore(&text("restore")[..20]).is_err(), "too short");

	let code = LinkCode::Account { account: account_id, r };
	assert_eq!(code.to_string(), text("link_code"));
	assert_eq!(LinkCode::read(&text("link_code")), Some(code));
	let (id, key_text) = text("request_code").rsplit_once(':').map(|(a, b)| (a.to_owned(), b.to_owned())).unwrap();
	let code = LinkCode::read(&text("request_code")).unwrap();
	let LinkCode::Request { id: request_id, key } = &code else { panic!("a request code") };
	assert_eq!(*key, request.public());
	assert_eq!(sync::text(key), key_text);
	assert!(id.ends_with(&sync::text(request_id)));
	assert_eq!(code.to_string(), text("request_code"));
	assert_eq!(LinkCode::read("questpump-link:1:nope"), None);
	for refused in f["request_refused"].as_array().unwrap() {
		assert_eq!(LinkCode::read(refused.as_str().unwrap()), None, "{refused}");
	}
}

/// The account's new key (seal.json's `rekey` block): a device's key and its MAC under R and under the new R, the three
/// MACs every app refuses, the new R's keys and restore code, the wrap the rekeying device seals with its fresh key and
/// nonce and the device opens with its own, and the old R sealed under the new one, the chain's link, which only the
/// new R opens to the R its kid names.
#[test]
fn the_account_s_new_key_is_wrapped_to_each_device_key_its_mac_holds_and_chained_to_the_r_it_replaces() {
	use sha2::Digest as _;
	let f = &fixture("seal.json")["rekey"];
	let hex = |v: &J| unhex(v.as_str().unwrap());
	let text = |v: &J| v.as_str().unwrap().to_owned();
	let (next, wrap, prev) = (&f["next"], &f["wrap"], &f["prev"]);
	let r: [u8; 16] = hex(&f["R_hex"]).try_into().unwrap();
	let next_r: [u8; 16] = hex(&next["R_hex"]).try_into().unwrap();
	let (keys, next_keys) = (SyncKeys::derive(&r), SyncKeys::derive(&next_r));
	assert_eq!(keys.kid, text(&f["kid"]));
	assert_eq!(keys.kid, text(&fixture("seal.json")["sync"]["kid"]), "the sync block's R");

	// the device's key and its MAC under R and under the new R
	let device_key = KeyPair::from_bytes(&hex(&f["device_secret_hex"]).try_into().unwrap()).unwrap();
	let public = device_key.public_uncompressed();
	assert_eq!(public.to_vec(), hex(&f["device_public_hex"]), "SEC1 uncompressed");
	assert_eq!(sync::text(&public), text(&f["device_key"]));
	let device: [u8; 16] = sync::bytes(&text(&f["device"])).unwrap();
	assert_eq!(sync::text(&keys.mac(&device, &public)), text(&f["mac"]));
	assert_eq!(sync::text(&next_keys.mac(&device, &public)), text(&next["mac"]));
	assert!(keys.mac_holds(&device, &public, &sync::bytes::<32>(&text(&f["mac"])).unwrap()));
	assert!(next_keys.mac_holds(&device, &public, &sync::bytes::<32>(&text(&next["mac"])).unwrap()));
	for refused in f["mac_refused"].as_array().unwrap() {
		// each checked under the R its kid names: R's
		assert_eq!(text(&refused["kid"]), keys.kid);
		let (device, key, mac) = (&refused["device"], &refused["key"], &refused["mac"]);
		let (device, key): ([u8; 16], [u8; 65]) =
			(sync::bytes(&text(device)).unwrap(), sync::bytes(&text(key)).unwrap());
		assert!(!keys.mac_holds(&device, &key, &sync::bytes::<32>(&text(mac)).unwrap()), "{refused}");
	}

	// the new R's keys and restore code, as R's
	assert_eq!(next_keys.auth, text(&next["auth"]));
	let auth_hash =
		sha2::Sha256::digest(base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(&next_keys.auth).unwrap());
	assert_eq!(auth_hash.to_vec(), hex(&next["auth_sha256_hex"]));
	assert_eq!(next_keys.kid, text(&next["kid"]));
	assert_eq!(sync::restore_code(&next_r), text(&next["restore"]));
	let next_seal = questpump_core::seal::Sealer::new(&hex(&next["seal_key_hex"]).try_into().unwrap());

	// the wrap: the rekeying device's fresh E, then the new R sealed under their ECDH's wrap seal
	let e = KeyPair::from_bytes(&hex(&wrap["ephemeral_secret_hex"]).try_into().unwrap()).unwrap();
	assert_eq!(e.public_uncompressed().to_vec(), hex(&wrap["ephemeral_public_hex"]), "E, uncompressed");
	let nonce: [u8; 12] = hex(&wrap["nonce_hex"]).try_into().unwrap();
	let sealed = STANDARD.decode(wrap["sealed_b64"].as_str().unwrap()).unwrap();
	assert_eq!(sealed.len(), 109);
	let (head, body) = sealed.split_at(sync::PUBLIC_KEY);
	assert_eq!(head, e.public_uncompressed().as_slice(), "E first");
	let wrap_key = questpump_core::seal::Sealer::new(&hex(&wrap["wrap_key_hex"]).try_into().unwrap());
	assert_eq!(wrap_key.seal_with(nonce, &next_r), body, "the wrap key");
	// ECDH(e, the device's key) is ECDH(the device's secret, E)
	assert_eq!(e.wrap_seal(&public).unwrap().seal_with(nonce, &next_r), body, "made from the fixed e and nonce");
	assert_eq!(device_key.wrap_seal(head).unwrap().open(body).unwrap(), (next_r.to_vec(), nonce));
	assert_eq!(sync::open_wrap(&device_key, &sealed).unwrap(), next_r, "opened with the device's secret");
	assert!(sync::open_wrap(&e, &sealed).is_err(), "with another key");
	assert!(sync::open_wrap(&device_key, body).is_err(), "without E");
	let ours = sync::wrap_root(&e, &public, &next_r).unwrap();
	assert_eq!((ours.len(), &ours[..sync::PUBLIC_KEY]), (109, head), "a wrap this computer makes, under a fresh nonce");
	assert_eq!(sync::open_wrap(&device_key, &ours).unwrap(), next_r);
	assert!(sync::wrap_root(&e, &[4; 65], &next_r).is_none(), "a device key that is no point");
	assert_eq!(SyncKeys::derive(&sync::open_wrap(&device_key, &sealed).unwrap()).kid, text(&next["kid"]));

	// prev: R sealed under the new R's seal key, the chain's link back to the kid the device holds
	let prev_nonce: [u8; 12] = hex(&prev["nonce_hex"]).try_into().unwrap();
	let prev_sealed = STANDARD.decode(prev["sealed_b64"].as_str().unwrap()).unwrap();
	assert_eq!(prev_sealed.len(), 44);
	assert_eq!(next_keys.seal.seal_with(prev_nonce, &r), prev_sealed);
	assert_eq!(next_seal.seal_with(prev_nonce, &r), prev_sealed, "the new R's seal key");
	assert_eq!(sync::open_prev(&next_r, &next_keys.kid, &keys.kid, &prev_sealed), Some(r));
	assert_eq!(sync::open_prev(&next_r, &next_keys.kid, &next_keys.kid, &prev_sealed), None, "another prev_kid");
	assert_eq!(sync::open_prev(&next_r, &keys.kid, &keys.kid, &prev_sealed), None, "a link of another kid");
	assert_eq!(sync::open_prev(&r, &keys.kid, &keys.kid, &prev_sealed), None, "opened under the old R");
}

fn roundtrip<T: serde::de::DeserializeOwned + serde::Serialize>(json: &J) -> J {
	// through the text, as a body arrives: a kept sheet (`RawValue`) reads only from JSON text
	let value: T = serde_json::from_str(&json.to_string()).unwrap_or_else(|e| panic!("{json}: {e}"));
	serde_json::to_value(&value).unwrap()
}

#[test]
fn every_body_reads_and_writes_back_as_the_contract_shows_it() {
	for body in fixture("wire.json")["bodies"].as_array().unwrap() {
		let json = &body["json"];
		let back = match body["kind"].as_str().unwrap() {
			"hello_response" => roundtrip::<HelloResponse>(json),
			"pair_request" => roundtrip::<PairRequest>(json),
			"pair_response" => roundtrip::<PairResponse>(json),
			"ping_response" => roundtrip::<PingResponse>(json),
			"sync_request" => roundtrip::<SyncRequest>(json),
			"sync_response" => roundtrip::<SyncResponse>(json),
			"sheet_response" => roundtrip::<SheetResponse>(json),
			"history_response" => roundtrip::<HistoryResponse>(json),
			"sheets_response" => roundtrip::<SheetsResponse>(json),
			"app_rows_response" => roundtrip::<AppRowsResponse>(json),
			"body_request" => roundtrip::<BodyRequest>(json),
			"body_response" => roundtrip::<BodyResponse>(json),
			"account_response" => roundtrip::<AccountResponse>(json),
			"account_request" => roundtrip::<AccountRequest>(json),
			"refused_response" => roundtrip::<RefusedResponse>(json),
			other => panic!("a body this test does not know: {other}"),
		};
		assert_eq!(&back, json, "{}", body["about"]);
	}
}

/// A held request from the device's one app install.
fn held(device: Device, since: u64) -> SyncRequest {
	SyncRequest {
		device,
		client: format!("{}-1", device.name()),
		t0: 0,
		sent_at: 0,
		since,
		samples: vec![],
		events: vec![],
		rows: vec![],
		received: vec![],
		battery: None,
		hold: None,
		offer_taken: None,
		game_offer_at: None,
		work: None,
		pro_proof: None,
		wait: None,
	}
}

fn turn_in(quest: u32) -> Command {
	Command::TurnIn { quest_id: quest, tier: 1 }
}

#[test]
fn the_holder_gets_each_command_until_it_acknowledges_it() {
	let mut link = Link::new(1_000);
	link.arrive(&held(Device::Watchos, 0), &Via::Lan, 1_000);
	let first = link.queue(turn_in(1), 1_010).unwrap();
	let second = link.queue(turn_in(2), 1_020).unwrap();
	assert_eq!(
		(first.seq, second.seq, link.next()),
		(1_001, 1_002, 1_002),
		"the sequence starts at the helper's start"
	);
	assert_eq!(link.respond(&held(Device::Watchos, 0), 1_030).0.len(), 2);
	link.arrive(&held(Device::Watchos, first.seq), &Via::Lan, 1_040);
	let left: Vec<u64> = link.respond(&held(Device::Watchos, first.seq), 1_040).0.iter().map(|q| q.seq).collect();
	assert_eq!(left, [second.seq], "the first is acknowledged, the second not yet");
	link.arrive(&held(Device::Watchos, second.seq), &Via::Lan, 1_050);
	assert!(link.respond(&held(Device::Watchos, 0), 1_050).0.is_empty(), "acknowledged commands never come back");
}

#[test]
fn a_watch_holds_the_link_over_a_phone_and_the_phone_takes_it_when_the_watch_goes_quiet() {
	let mut link = Link::new(0);
	link.arrive(&held(Device::Ios, 0), &Via::Lan, 1_000);
	link.arrive(&held(Device::Watchos, 0), &Via::Lan, 2_000);
	let queued = link.queue(turn_in(7), 2_100).unwrap();
	assert_eq!(link.holder(2_100), Some(Device::Watchos));
	assert!(
		link.respond(&held(Device::Ios, 0), 2_100).0.is_empty(),
		"the phone gets nothing while the watch holds the link"
	);
	link.arrive(&held(Device::Ios, 0), &Via::Lan, 20_000);
	let later = 2_000 + link::ALIVE_MS + 1;
	assert_eq!(link.holder(later), Some(Device::Ios), "the watch's last request is too old");
	assert_eq!(
		link.respond(&held(Device::Ios, 0), later).0.first().map(|q| q.seq),
		Some(queued.seq),
		"what the watch never saw"
	);
}

#[test]
fn a_response_never_lets_a_device_acknowledge_a_command_it_did_not_get() {
	let mut link = Link::new(0);
	link.arrive(&held(Device::Watchos, 0), &Via::Lan, 1_000);
	link.arrive(&held(Device::Ios, 0), &Via::Lan, 1_000);
	let queued = link.queue(turn_in(5), 1_100).unwrap();
	assert_eq!(link.respond(&held(Device::Ios, 0), 1_100), (vec![], 0), "the phone's since comes back as it was");
	let (commands, next) = link.respond(&held(Device::Watchos, 0), 1_100);
	assert_eq!((commands.len(), next), (1, queued.seq), "the holder's next covers exactly what it got");
}

#[test]
fn a_command_holds_until_it_expires_and_a_countdown_seen_after_its_window_is_never_queued() {
	let mut link = Link::new(0);
	link.arrive(&held(Device::Watchos, 0), &Via::Lan, 0);
	let countdown =
		|go_at| Command::Countdown { pump_id: "g-1-2".into(), slot: 0, start_at: go_at - 5_000, go_at, ticks: 5 };
	let queued = link.queue(countdown(10_000), 1_000).unwrap();
	assert_eq!(queued.expires_at, 10_000 + link::COUNTDOWN_AFTER_GO_MS);
	link.arrive(&held(Device::Watchos, 0), &Via::Lan, queued.expires_at - 2); // the watch keeps syncing
	assert_eq!(
		link.respond(&held(Device::Watchos, 0), queued.expires_at - 1).0.len(),
		1,
		"a late device runs the move from its go"
	);
	assert!(link.respond(&held(Device::Watchos, 0), queued.expires_at).0.is_empty(), "dropped at its expiry");
	assert!(link.queue(countdown(10_000), 10_000 + link::COUNTDOWN_AFTER_GO_MS).is_none(), "born expired");
}

#[test]
fn a_second_held_request_from_the_same_device_makes_the_first_answer() {
	let mut link = Link::new(0);
	let first = link.arrive(&held(Device::Watchos, 0), &Via::Lan, 1_000);
	assert!(link.is_current(&held(Device::Watchos, 0), first));
	let second = link.arrive(&held(Device::Watchos, 0), &Via::Lan, 1_500);
	assert!(!link.is_current(&held(Device::Watchos, 0), first), "the older request answers at once, without commands");
	assert!(link.is_current(&held(Device::Watchos, 0), second));
	let phone = link.arrive(&held(Device::Ios, 0), &Via::Lan, 1_600);
	assert!(
		link.is_current(&held(Device::Watchos, 0), second) && link.is_current(&held(Device::Ios, 0), phone),
		"one request per device"
	);
}

#[test]
fn two_watches_hold_one_at_a_time_without_answering_each_others_requests() {
	let mut link = Link::new(0);
	let real = held(Device::Watchos, 0);
	let dev = SyncRequest { client: "watchos-2".into(), ..held(Device::Watchos, 0) };
	let first = link.arrive(&real, &Via::Lan, 1_000);
	let second = link.arrive(&dev, &Via::Lan, 1_100);
	assert!(link.is_current(&real, first) && link.is_current(&dev, second), "each keeps its own hold");
	let queued = link.queue(turn_in(9), 1_200).unwrap();
	assert!(link.respond(&real, 1_200).0.is_empty(), "the latest watch holds the link");
	assert_eq!(link.respond(&dev, 1_200), (vec![queued.clone()], queued.seq));
}

#[test]
fn a_plain_post_neither_holds_the_link_nor_acknowledges() {
	let mut link = Link::new(0);
	let watch = link.arrive(&held(Device::Watchos, 0), &Via::Lan, 1_000);
	let queued = link.queue(turn_in(3), 1_100).unwrap();
	let last_word = SyncRequest { hold: Some(false), ..held(Device::Watchos, queued.seq) };
	link.arrive(&last_word, &Via::Lan, 1_200);
	assert!(link.is_current(&held(Device::Watchos, 0), watch), "the holder's held request still waits");
	assert_eq!(link.respond(&held(Device::Watchos, 0), 1_200).0, [queued], "its since acknowledged nothing");
	let mut alone = Link::new(0);
	alone.arrive(&SyncRequest { hold: Some(false), ..held(Device::Ios, 0) }, &Via::Lan, 1_000);
	assert_eq!(alone.holder(1_000), None, "a lone phone's plain post takes no link");
}

#[test]
fn each_command_expires_after_its_own_time() {
	let mut link = Link::new(0);
	let at = 1_000;
	let offered = Command::PumpOffered { pump_id: "g-1-2".into(), quest_id: Some(7), moves: vec![], streak: None };
	let cases = [
		(turn_in(1), 30_000),
		(offered, 120_000),
		(Command::QuestComplete { quest_id: 7 }, 60_000),
		(Command::PumpDone { pump_id: "g-1-2".into(), ended: PumpEnd::Done }, 60_000),
		(Command::MoveEnd { pump_id: "g-1-2".into(), slot: 0, cancelled: true }, 60_000),
		(Command::SessionEnd, 60_000),
	];
	for (command, life) in cases {
		let name = format!("{command:?}");
		assert_eq!(link.queue(command, at).unwrap().expires_at, at + life, "{name}");
	}
}

#[test]
fn a_pump_event_sent_again_after_a_lost_response_is_taken_once() {
	let mut link = Link::new(0);
	let start = WristEvent {
		kind: EventType::PumpStart,
		pump_id: "g-513-1790000000".into(),
		tier: None,
		start_at: 1_790_000_000_500,
		seconds: None,
		steps: None,
		metres: None,
		peak_bpm: None,
		delta_bpm: None,
		verdict: None,
		slot: Some(0),
		moves: None,
		unwell: false,
		stopped_at: None,
	};
	let restart = WristEvent { start_at: 1_790_000_009_000, ..start.clone() };
	let req = SyncRequest { events: vec![start.clone()], ..held(Device::Watchos, 0) };
	assert_eq!(link.take_events(&req), std::slice::from_ref(&start));
	let again = SyncRequest { events: vec![start, restart.clone()], ..req };
	assert_eq!(link.take_events(&again), [restart], "the same start once; the move started again is new");
}

#[test]
fn a_receipt_sent_again_after_a_lost_response_counts_once() {
	let mut link = Link::new(0);
	link.arrive(&held(Device::Watchos, 0), &Via::Lan, 0);
	let queued = link.queue(turn_in(1), 1_000).unwrap();
	let receipt = SyncRequest { received: vec![Received { seq: queued.seq, at: 1_040 }], ..held(Device::Watchos, 0) };
	link.arrive(&receipt, &Via::Lan, 1_050);
	link.arrive(&receipt, &Via::Lan, 1_300); // its response was lost: the same receipt again
	let summaries = link.quiet(1_300 + link::QUIET_MS);
	assert_eq!((summaries[0].commands, summaries[0].p50_ms), (1, Some(40)));
}

#[test]
fn a_since_from_an_earlier_run_acknowledges_nothing_and_every_command_goes_out() {
	let mut link = Link::new(5_000);
	link.arrive(&held(Device::Watchos, 0), &Via::Lan, 5_000);
	let queued = link.queue(turn_in(4), 5_010).unwrap();
	link.arrive(&held(Device::Watchos, 999_999), &Via::Lan, 5_020); // the watch's since from a run whose clock was ahead
	assert_eq!(link.respond(&held(Device::Watchos, 999_999), 5_020).0.first().map(|q| q.seq), Some(queued.seq));
}

#[test]
fn a_devices_delivery_times_and_battery_come_out_per_session_once_it_goes_quiet() {
	let mut link = Link::new(0);
	link.arrive(&held(Device::Watchos, 0), &Via::Lan, 0);
	let mut seqs = Vec::new();
	for i in 0..20 {
		seqs.push(link.queue(turn_in(i), i64::from(i) * 1_000).unwrap().seq);
	}
	let received: Vec<Received> =
		seqs.iter().zip(0..).map(|(&seq, i)| Received { seq, at: i * 1_000 + 40 + i }).collect();
	let first = SyncRequest { received, battery: Some(0.9), ..held(Device::Watchos, *seqs.last().unwrap()) };
	link.arrive(&first, &Via::Lan, 30_000);
	let last = SyncRequest { battery: Some(0.8), ..held(Device::Watchos, *seqs.last().unwrap()) };
	link.arrive(&last, &Via::Lan, 30_000 + 3_600_000 / 2);
	assert!(link.quiet(30_000 + 3_600_000 / 2 + 1_000).is_empty(), "still in its session");
	let summaries = link.quiet(30_000 + 3_600_000 / 2 + link::QUIET_MS);
	let s = &summaries[0];
	assert_eq!((s.device, s.commands, s.p50_ms, s.p95_ms), (Device::Watchos, 20, Some(49), Some(58)));
	assert!((s.battery_per_hour.unwrap() - 20.0).abs() < 1e-9, "10 % in half an hour");
	assert!(link.summaries().is_empty(), "printed once");
}

#[test]
fn ten_wrong_bearers_within_a_minute_refuse_the_address_for_a_minute() {
	let mut throttle = Throttle::default();
	let peer: IpAddr = "192.168.1.30".parse().unwrap();
	let other: IpAddr = "192.168.1.31".parse().unwrap();
	for i in 0..9 {
		throttle.failed(peer, i * 1_000);
	}
	assert!(!throttle.refused(&peer, 9_000));
	throttle.failed(peer, 9_000);
	assert!(throttle.refused(&peer, 9_001) && !throttle.refused(&other, 9_001));
	assert!(!throttle.refused(&peer, 9_000 + 60_000), "a minute after the last wrong bearer");
}

#[test]
fn a_catch_up_pump_is_its_own_devices_and_carries_the_seconds_of_its_moves() {
	let event = |id: &str, moves| WristEvent {
		kind: EventType::PumpDone,
		pump_id: id.to_owned(),
		tier: None,
		start_at: 1_790_000_040_999,
		seconds: None,
		steps: None,
		metres: None,
		peak_bpm: None,
		delta_bpm: None,
		verdict: None,
		slot: None,
		moves,
		unwell: false,
		stopped_at: None,
	};
	let moves = Some(vec![
		CatchupMove {
			seconds: 40,
			move_id: 2,
			ts: None,
			set: None,
			ran: None,
			kind: None,
			ladder: None,
			rung: None,
			allout: false,
		},
		CatchupMove {
			seconds: 35,
			move_id: 12,
			ts: None,
			set: None,
			ran: None,
			kind: None,
			ladder: None,
			rung: None,
			allout: false,
		},
	]);
	let pump = event("p-1b4e", moves.clone()).catch_up(Device::Ios).unwrap();
	assert_eq!((pump.source, pump.start, pump.moves.len()), (Source::Phone, 1_790_000_040, 2), "floored to the second");
	assert_eq!(event("w-9", moves.clone()).catch_up(Device::Watchos).unwrap().source, Source::Watch);
	assert!(event("w-9", moves.clone()).catch_up(Device::Ios).is_none(), "a phone never pays with a watch's pump");
	assert!(event("g-513-1790000000", moves).catch_up(Device::Watchos).is_none(), "a game pump pays nothing");
	assert!(event("w-9", None).catch_up(Device::Watchos).is_none(), "no moves, no catch-up");
	let stop = WristEvent { unwell: true, ..event("w-9", None) };
	assert!(
		stop.catch_up(Device::Watchos).is_some_and(|p| p.unwell_at.is_some()),
		"a stop before the first move still locks"
	);
	// an app pump (the app composed it) is never a catch-up, its felt-unwell stop neither: the stop is one of the
	// app's rows (slice 26)
	let ran = Some(vec![CatchupMove {
		seconds: 30,
		move_id: 12,
		ts: None,
		set: None,
		ran: None,
		kind: None,
		ladder: None,
		rung: None,
		allout: false,
	}]);
	assert!(event("f-3f0d", ran.clone()).catch_up(Device::Ios).is_none(), "an app pump is never a payment");
	for device in [Device::Ios, Device::Android, Device::Watchos, Device::Wearos] {
		let app = WristEvent { unwell: true, verdict: Some(Verdict::Proven), ..event("f-3f0d", ran.clone()) };
		assert!(app.catch_up(device).is_none(), "{device:?}: its stop comes among the app's rows");
	}
	let newer: SyncRequest = serde_json::from_value(serde_json::json!({
		"device": "watchos", "client": "c", "t0": 1, "sent_at": 1, "since": 0,
		"events": [{"type": "pump_paused", "pump_id": "w-9", "start_at": 1}]
	}))
	.unwrap();
	assert_eq!(newer.events[0].kind, EventType::Other, "a newer app's event is taken, and pays nothing");
	assert!(newer.events[0].catch_up(Device::Watchos).is_none());
}

fn now(unix: i64) -> Now {
	Now { unix, rfc3339: format!("t{unix}") }
}

/// A logout case's frames: a pump offered, counted down and run.
fn frames_of(case: usize) -> Vec<strip::RawFrame> {
	let rows = fixture("rows.json");
	rows["logout"][case]["frames"]
		.as_array()
		.unwrap()
		.iter()
		.map(|f| strip::decode_bytes(&bytes(&f["bytes"])).unwrap())
		.collect()
}

fn bytes(v: &J) -> [u8; strip::BYTES] {
	let list: Vec<u8> = v.as_array().unwrap().iter().map(|b| u8::try_from(b.as_u64().unwrap()).unwrap()).collect();
	list.try_into().unwrap()
}

fn frames() -> Vec<strip::RawFrame> {
	frames_of(0)
}

/// The frame's bytes with some changed (1-based positions, as the strip's table counts them) and the checksum again.
fn changed(f: &strip::RawFrame, changes: &[(usize, u8)]) -> strip::RawFrame {
	let rows = fixture("rows.json");
	let all = rows["logout"][0]["frames"].as_array().unwrap();
	let mut b =
		all.iter().map(|x| bytes(&x["bytes"])).find(|b| strip::decode_bytes(b).ok().as_ref() == Some(f)).unwrap();
	for &(i, v) in changes {
		b[i - 1] = v;
	}
	b[93] = b[3..93].iter().fold(0u8, |s, &x| s.wrapping_add(x));
	strip::decode_bytes(&b).unwrap()
}

/// The countdowns and the move ends a session makes, in order: (slot, what it was).
fn moves(session: &mut Session, frames: &[strip::RawFrame]) -> Vec<(u8, &'static str)> {
	let mut out = Vec::new();
	for (i, f) in frames.iter().enumerate() {
		let shown = 1_000.0 + f64::from(u32::try_from(i).unwrap()) / 10.0;
		for c in session.on_frame(f, shown, &now(1)).commands {
			match c {
				Command::Countdown { slot, .. } => out.push((slot, "countdown")),
				Command::MoveEnd { slot, cancelled, .. } => {
					out.push((slot, if cancelled { "cancelled" } else { "done" }));
				}
				_ => {}
			}
		}
	}
	out
}

#[test]
fn every_move_of_a_pump_gets_its_own_countdown_and_its_end() {
	let mut session = Session::new(Store::in_memory(), People::default(), Zone::fixed(0));
	let seen = moves(&mut session, &frames_of(1));
	assert_eq!(
		seen,
		[(0, "countdown"), (0, "done"), (1, "countdown"), (1, "done")],
		"move 1 counted down and ended (its count typed), then move 2's own Start and end"
	);
}

#[test]
fn a_move_whose_result_never_painted_is_done_when_the_pump_goes_on_its_row_comes_or_the_next_start_does() {
	// a timed move that ran its time goes straight on to the next move: its result view never paints (frames 24-25
	// of case 1 are the squat's result heartbeats, 26 the push-up's Ready, 27 the squat's own row, 29 the push-up's
	// Start)
	let frames = frames_of(1);
	let without = |skip: std::ops::Range<usize>| -> Vec<strip::RawFrame> {
		frames.iter().enumerate().filter(|(i, _)| !skip.contains(i)).map(|(_, f)| f.clone()).collect()
	};
	let done = [(0, "countdown"), (0, "done"), (1, "countdown"), (1, "done")];
	for (skip, what) in [(24..26, "the pump went on"), (24..27, "its own row came"), (24..29, "the next Start came")] {
		let mut session = Session::new(Store::in_memory(), People::default(), Zone::fixed(0));
		assert_eq!(moves(&mut session, &without(skip)), done, "done, never cancelled, once {what}");
	}
}

#[test]
fn a_countdown_cancelled_back_to_ready_ends_on_the_device_and_the_next_start_counts_again() {
	let frames = frames();
	let at = frames.iter().position(|f| f.event == StripEvent::CountdownStart).unwrap();
	let start = &frames[at];
	let Body::Fields(f) = &start.body else { unreachable!() };
	// Escape during the count: an urgent heartbeat of the same pump shows it back at Ready (state idle, byte 33)
	let beat = frames[at + 1..].iter().find(|x| x.is_heartbeat()).unwrap();
	let ready = changed(beat, &[(5, 250), (33, 0)]);
	// Start again: a new countdown_start, a later start (bytes 19-21) under a new event counter (bytes 40-41)
	let later = (f.start_at + 3_000) & 0x00ff_ffff;
	let [_, s0, s1, s2] = later.to_be_bytes();
	let again = changed(start, &[(5, 251), (19, s0), (20, s1), (21, s2), (40, 0), (41, 250)]);
	let mut session = Session::new(Store::in_memory(), People::default(), Zone::fixed(0));
	let seen = moves(&mut session, &[frames[at - 1].clone(), start.clone(), ready, again, beat.clone()]);
	assert_eq!(
		seen,
		[(f.slot, "countdown"), (f.slot, "cancelled"), (f.slot, "countdown")],
		"the device drops its count, then counts the new Start; the old start's heartbeat adds nothing"
	);
}

fn countdowns(session: &mut Session, frames: &[strip::RawFrame]) -> Vec<(usize, Command)> {
	let mut out = Vec::new();
	for (i, f) in frames.iter().enumerate() {
		let shown = 1_000.0 + f64::from(u32::try_from(i).unwrap()) / 10.0;
		for c in session.on_frame(f, shown, &now(1)).commands {
			if matches!(c, Command::Countdown { .. }) {
				out.push((i, c));
			}
		}
	}
	out
}

#[test]
fn a_helper_that_missed_the_countdowns_frame_joins_from_the_next_heartbeat_once() {
	let frames = frames();
	let start = frames.iter().position(|f| f.event == StripEvent::CountdownStart).unwrap();
	let seen = countdowns(&mut Session::new(Store::in_memory(), People::default(), Zone::fixed(0)), &frames);
	assert_eq!(seen.iter().map(|(i, _)| *i).collect::<Vec<_>>(), [start], "the heartbeats after it add none");
	let missed: Vec<strip::RawFrame> =
		frames.iter().enumerate().filter(|(i, _)| *i != start).map(|(_, f)| f.clone()).collect();
	let joined = countdowns(&mut Session::new(Store::in_memory(), People::default(), Zone::fixed(0)), &missed);
	assert_eq!(joined.len(), 1, "one countdown for the pump, from the heartbeats");
	let (i, Command::Countdown { start_at, go_at, pump_id, .. }) = &joined[0] else { unreachable!() };
	let Body::Fields(f) = &missed[*i].body else { unreachable!() };
	assert_matches!(f.state, StripState::Countdown | StripState::Running);
	let shown = 1_000_000 + i64::from(u32::try_from(*i).unwrap()) * 100;
	assert_eq!(*start_at, shown + i64::from(f.start_at) - i64::from(f.ts_client), "dated by its own paint");
	assert_eq!((*go_at - *start_at, pump_id.starts_with("g-")), (5_000, true));
}

#[test]
fn a_start_first_seen_after_a_break_is_dated_from_the_next_heartbeat() {
	let frames = frames();
	let start = frames.iter().position(|f| f.event == StripEvent::CountdownStart).unwrap();
	let mut session = Session::new(Store::in_memory(), People::default(), Zone::fixed(0));
	countdowns(&mut session, &frames[..start]);
	session.lost_sight(); // the stream started again: the countdown's paint may be a second old when first seen
	let seen = countdowns(&mut session, &frames[start..]);
	let beat = frames[start..].iter().position(strip::RawFrame::is_heartbeat).unwrap();
	assert_eq!(seen.iter().map(|(i, _)| *i).collect::<Vec<_>>(), [beat], "one countdown, from the next paint");
}

#[test]
fn the_countdown_is_dated_earlier_by_the_learnt_display_lag() {
	let frames = frames();
	let plain = countdowns(&mut Session::new(Store::in_memory(), People::default(), Zone::fixed(0)), &frames);
	let mut lagged = Session::new(Store::in_memory(), People::default(), Zone::fixed(0));
	lagged.set_display_lag(25);
	let late = countdowns(&mut lagged, &frames);
	let (Command::Countdown { start_at: a, .. }, Command::Countdown { start_at: b, .. }) = (&plain[0].1, &late[0].1)
	else {
		unreachable!()
	};
	assert_eq!(a - b, 25, "the paint reached the glass 25 ms after the addon made it");
}

#[test]
fn the_game_quitting_tells_the_device_the_session_is_over() {
	let mut session = Session::new(Store::in_memory(), People::default(), Zone::fixed(0));
	assert_eq!(session.end_game(&now(1)).commands, [Command::SessionEnd]);
}

#[test]
fn the_game_quitting_mid_move_ends_the_move_before_the_session() {
	let frames = frames();
	let start = frames.iter().position(|f| f.event == StripEvent::CountdownStart).unwrap();
	let mut session = Session::new(Store::in_memory(), People::default(), Zone::fixed(0));
	countdowns(&mut session, &frames[..=start]);
	let ends: Vec<Command> = session
		.end_game(&now(1))
		.commands
		.into_iter()
		.filter(|c| matches!(c, Command::MoveEnd { .. } | Command::SessionEnd))
		.collect();
	assert_matches!(
		ends.as_slice(),
		[Command::MoveEnd { cancelled: false, .. }, Command::SessionEnd],
		"the device finishes the move, then asks whether the player is still there"
	);
}
