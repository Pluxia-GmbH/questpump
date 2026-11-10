//! The Bluetooth link (docs/plan.md slice 43, with its fixes after the review of 2026-10-17) end to end through
//! questpump-core's public API, against `contract/fixtures/ble.json`, which the phones' cores replay too: the ble key
//! from the pairing key, the service's UUIDs, every hello the helper makes (the hour's hashes and the challenge) and the
//! advertisement, the associated data with the connection's challenge, every sealed case built, sealed, cut and read
//! back, every refused envelope refused, every frame case through the receiver's `Reassembly`, the limits, and the
//! link's hold and holder over Bluetooth. The hello cases without `made` are the phones' judgements alone.

#![expect(clippy::unwrap_used, reason = "a fixture that does not read fails the test loudly")]

use std::fmt::Write as _;
use std::path::PathBuf;

use base64::Engine as _;
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use questpump_core::ble::{self, Direction, Method, Reassembly, Request, Response, Step};
use questpump_core::link::{self, Device, Link, SyncRequest, Transport, Via};
use questpump_core::seal::{self, Keys, Route, Way};
use serde_json::{Value as J, json};

fn fixture(name: &str) -> J {
	let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../contract/fixtures").join(name);
	serde_json::from_str(&std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))).unwrap()
}

fn unhex(s: &str) -> Vec<u8> {
	(0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap()).collect()
}

fn hex(bytes: &[u8]) -> String {
	bytes.iter().fold(String::new(), |mut out, b| {
		let _ = write!(out, "{b:02x}");
		out
	})
}

fn text(v: &J) -> &str {
	v.as_str().unwrap()
}

fn key(f: &J) -> [u8; 32] {
	unhex(text(&f["key_hex"])).try_into().unwrap()
}

fn direction(v: &J) -> Direction {
	match text(v) {
		"rx" => Direction::Rx,
		"tx" => Direction::Tx,
		other => panic!("a direction this test does not know: {other}"),
	}
}

fn id(v: &J) -> u8 {
	u8::try_from(v.as_u64().unwrap()).unwrap()
}

fn challenge(v: &J) -> ble::Challenge {
	unhex(text(v)).try_into().unwrap()
}

fn b64(v: &J) -> Vec<u8> {
	URL_SAFE_NO_PAD.decode(text(v)).unwrap()
}

#[test]
fn the_ble_key_the_service_the_hello_and_the_advertisement_are_the_contracts() {
	let f = fixture("ble.json");
	assert_eq!(ble::INFO, text(&f["info"]).as_bytes());
	assert_eq!(hex(&ble::key(&key(&f))), text(&f["ble_key_hex"]), "the fourth derivation of the pairing key");
	assert_eq!(text(&f["key_hex"]), text(&fixture("seal.json")["key_hex"]), "seal.json's test key");
	let uuids = &f["uuids"];
	for (name, uuid) in [("service", ble::SERVICE), ("hello", ble::HELLO), ("rx", ble::RX), ("tx", ble::TX)] {
		assert_eq!(ble::uuid_text(uuid), text(&uuids[name]), "{name}");
	}
	let gatt = &f["gatt"];
	assert_eq!(gatt["hello"]["v"], json!(ble::VERSION));
	assert_eq!(gatt["epoch_s"], json!(ble::EPOCH_S));
	assert_eq!(gatt["challenge_bytes"], json!(ble::CHALLENGE_BYTES));
	assert_eq!(gatt["labels"]["k"], json!("questpump ble hello key"));
	let mut made = 0;
	for case in gatt["cases"].as_array().unwrap() {
		let about = text(&case["about"]);
		let Some(m) = case.get("made") else { continue }; // a phone's judgement alone
		let hello_key: [u8; 32] = unhex(text(&m["hello_key_hex"])).try_into().unwrap();
		let at = m["at_s"].as_i64().unwrap();
		assert_eq!(json!(ble::epoch(at)), m["epoch"], "{about}: the hour's epoch");
		let bytes = ble::hello(&hello_key, at, &challenge(&m["challenge_hex"]));
		assert_eq!(String::from_utf8(bytes.clone()).unwrap(), text(&case["hello_text"]), "{about}: v, k, c");
		let hello: J = serde_json::from_slice(&bytes).unwrap();
		assert_eq!(hex(&b64(&hello["k"])), text(&m["k_hex"]), "{about}: k");
		made += 1;
	}
	assert!(made > 0, "the fixture has hellos the helper makes");
	let first = &gatt["cases"][0];
	assert_eq!(text(&first["hello_text"]), text(&gatt["hello_text"]), "the example is the first case's");
	let hello_key = unhex(text(&first["made"]["hello_key_hex"]));
	assert_eq!(hello_key, seal::hello_key(&key(&f)), "the hello proves the pairing key's hello key");
	assert_eq!(hello_key, Keys::derive(&key(&f)).hello);
	let advertising = &f["advertising"];
	assert_eq!(ble::LOCAL_NAME, text(&advertising["local_name"]));
	assert_eq!(hex(&ble::SERVICE_DATA), text(&advertising["service_data_hex"]));
	assert_eq!(text(&advertising["service_data_mask_hex"]), "ffffff", "the data matched whole");
	let aad = &f["aad"];
	let c = challenge(&aad["challenge_hex"]);
	assert_eq!(ble::aad(Direction::Rx, 0, &c)[..19], *text(&aad["rx"]).as_bytes());
	assert_eq!(ble::aad(Direction::Tx, 0, &c)[..19], *text(&aad["tx"]).as_bytes());
	assert_eq!(hex(&ble::aad(Direction::Rx, 1, &c)), text(&aad["rx_id_1_hex"]));
	assert_eq!(hex(&ble::aad(Direction::Tx, 255, &c)), text(&aad["tx_id_255_hex"]));
}

#[test]
fn the_limits_are_the_contracts() {
	let limits = &fixture("ble.json")["limits"];
	let n = |name: &str| usize::try_from(limits[name].as_u64().unwrap()).unwrap();
	let ms = |name: &str| limits[name].as_i64().unwrap();
	assert_eq!(n("header_bytes"), ble::HEADER_BYTES);
	assert_eq!(n("length_bytes"), ble::LENGTH_BYTES);
	assert_eq!(n("envelope_min_bytes"), ble::ENVELOPE_MIN_BYTES);
	assert_eq!(n("request_max_bytes"), ble::REQUEST_MAX_BYTES);
	assert_eq!(n("response_max_bytes"), ble::RESPONSE_MAX_BYTES);
	assert_eq!(ms("chunk_gap_ms"), ble::CHUNK_GAP_MS);
	assert_eq!(n("in_flight_max"), ble::IN_FLIGHT_MAX);
	assert_eq!(n("att_value_max"), ble::ATT_VALUE_MAX);
	assert_eq!(n("path_max_bytes"), ble::PATH_MAX_BYTES);
	assert_eq!(n("agent_max_bytes"), ble::AGENT_MAX_BYTES);
	assert_eq!(n("failed_max"), link::THROTTLE_AFTER, "the LAN's throttle, by central");
	assert_eq!(ms("failed_window_ms"), link::THROTTLE_MS);
	assert_eq!(ms("ble_wait_max_ms"), link::BLE_WAIT_MAX_MS);
	let pages = &limits["pages"];
	for (route, most) in
		[("history", link::HISTORY_PAGE), ("app_rows", link::HISTORY_PAGE), ("sheets", link::SHEETS_PAGE)]
	{
		let asked = u32::try_from(pages[route].as_u64().unwrap()).unwrap();
		assert_eq!(
			link::page_size(Some(asked), most),
			usize::try_from(asked).unwrap(),
			"{route}: a phone's limit holds"
		);
	}
	assert_eq!(link::page_size(None, link::HISTORY_PAGE), link::HISTORY_PAGE, "none on the LAN: the most");
	assert_eq!(link::page_size(Some(9_999), link::SHEETS_PAGE), link::SHEETS_PAGE, "never past the most");
}

#[test]
fn every_sealed_case_builds_seals_cuts_and_reads_back_as_the_contract_shows_it() {
	let f = fixture("ble.json");
	let keys = Keys::derive(&key(&f));
	for case in f["sealed"].as_array().unwrap() {
		let about = text(&case["about"]);
		let (direction, id) = (direction(&case["direction"]), id(&case["id"]));
		let c = challenge(&case["challenge_hex"]);
		let body = STANDARD.decode(text(&case["body_b64"])).unwrap();
		let plain = match direction {
			Direction::Rx => {
				let method = match text(&case["method"]) {
					"GET" => Method::Get,
					"POST" => Method::Post,
					other => panic!("{about}: a method this test does not know: {other}"),
				};
				let request =
					Request { method, path: text(&case["path"]).into(), agent: text(&case["agent"]).into(), body };
				let plain = request.to_plain();
				assert_eq!(Request::read(&plain), Ok(request), "{about}: the head reads back");
				plain
			}
			Direction::Tx => {
				let response = Response { status: u16::try_from(case["status"].as_u64().unwrap()).unwrap(), body };
				let plain = response.to_plain();
				assert_eq!(Response::read(&plain), Ok(response), "{about}: the head reads back");
				plain
			}
		};
		assert_eq!(hex(&plain), text(&case["plain_hex"]), "{about}: the plain text");
		let nonce: [u8; 12] = unhex(text(&case["nonce_hex"])).try_into().unwrap();
		let envelope = keys.ble.seal_with(nonce, direction, id, &c, &plain);
		assert_eq!(hex(&envelope), text(&case["envelope_hex"]), "{about}: the envelope");
		assert_eq!(keys.ble.open(direction, id, &c, &envelope), Ok(plain), "{about}: it opens");
		for (mtu, chunks) in case["chunks"].as_object().unwrap() {
			let size = ble::chunk_size(mtu.parse().unwrap());
			let cut: Vec<String> = ble::chunks(id, &envelope, size).iter().map(|c| hex(c)).collect();
			let expected: Vec<&str> = chunks.as_array().unwrap().iter().map(text).collect();
			assert_eq!(cut, expected, "{about}: cut at MTU {mtu}");
			let mut whole = Reassembly::new(ble::RESPONSE_MAX_BYTES);
			let steps: Vec<Step> = ble::chunks(id, &envelope, size).iter().map(|c| whole.push(c, 0)).collect();
			assert_eq!(steps.last(), Some(&Step::Message { id, envelope: envelope.clone() }), "{about}: put together");
		}
		if let Some(inner) = case.get("body_plain") {
			let r = &case["body_route"];
			let way = if text(&r["way"]) == "request" { Way::Request } else { Way::Answer };
			let route = Route { way, method: text(&r["method"]), path: text(&r["path"]) };
			let (opened, seen) = keys.seal.open(&route, &STANDARD.decode(text(&case["body_b64"])).unwrap()).unwrap();
			assert_eq!(
				String::from_utf8(opened).unwrap(),
				text(inner),
				"{about}: the LAN body inside, sealed for its route"
			);
			assert_eq!(hex(&seen), text(&case["body_nonce_hex"]));
		}
	}
	let c = [7; ble::CHALLENGE_BYTES];
	let one = keys.ble.seal(Direction::Tx, 7, &c, b"\x00\xc8");
	let two = keys.ble.seal(Direction::Tx, 7, &c, b"\x00\xc8");
	assert_ne!(one[..12], two[..12], "a fresh nonce every time");
	assert_eq!(keys.ble.open(Direction::Tx, 7, &c, &two).unwrap(), b"\x00\xc8");
}

#[test]
fn an_envelope_that_does_not_open_or_whose_head_does_not_read_is_refused() {
	let f = fixture("ble.json");
	let keys = Keys::derive(&key(&f));
	for case in f["refused"].as_array().unwrap() {
		let about = text(&case["about"]);
		let (direction, id) = (direction(&case["direction"]), id(&case["id"]));
		let opened =
			keys.ble.open(direction, id, &challenge(&case["challenge_hex"]), &unhex(text(&case["envelope_hex"])));
		match text(&case["why"]) {
			"seal" => assert!(opened.is_err(), "{about}: it must not open"),
			"head" => {
				let plain = opened.unwrap_or_else(|e| panic!("{about}: it opens ({e})"));
				assert_eq!(hex(&plain), text(&case["plain_hex"]), "{about}");
				let read = match direction {
					Direction::Rx => Request::read(&plain).err(),
					Direction::Tx => Response::read(&plain).err(),
				};
				assert!(read.is_some(), "{about}: its head must not read");
			}
			other => panic!("{about}: a why this test does not know: {other}"),
		}
	}
}

/// A step's outcome in the fixture's words.
fn outcome(step: Step) -> J {
	match step {
		Step::Partial => json!("partial"),
		Step::Ignored => json!("ignored"),
		Step::Message { id, envelope } => json!({"message": {"id": id, "envelope_hex": hex(&envelope)}}),
		Step::Dropped { id, why } => json!({"dropped": {"id": id, "why": why.word()}}),
	}
}

#[test]
fn every_frame_case_puts_together_drops_or_ignores_as_the_contract_shows_it() {
	for case in fixture("ble.json")["frames"].as_array().unwrap() {
		let about = text(&case["about"]);
		let mut receiver = Reassembly::new(usize::try_from(case["limit"].as_u64().unwrap()).unwrap());
		let expects = case["expect"].as_array().unwrap();
		for (n, (step, expect)) in case["steps"].as_array().unwrap().iter().zip(expects).enumerate() {
			let got = if let Some(at) = step.get("tick_ms") {
				let dropped = receiver.tick(at.as_i64().unwrap());
				json!({"dropped": dropped.iter().map(|(id, why)| json!({"id": id, "why": why.word()})).collect::<Vec<_>>()})
			} else {
				outcome(receiver.push(&unhex(text(&step["chunk_hex"])), step["at_ms"].as_i64().unwrap()))
			};
			assert_eq!(&got, expect, "{about}: step {n}");
		}
	}
}

/// A phone's held request that may wait `wait`.
fn held(client: &str, wait: Option<i64>) -> SyncRequest {
	SyncRequest {
		device: Device::Ios,
		client: client.into(),
		t0: 0,
		sent_at: 0,
		since: 0,
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
		wait,
	}
}

#[test]
fn a_phone_on_bluetooth_holds_the_link_while_its_held_request_is_pending_and_hands_it_on_at_its_disconnect() {
	const CENTRAL: &str = "1A2B3C4D-0000-4000-8000-00000000AA01";
	let wait = fixture("ble.json")["limits"]["ble_wait_max_ms"].as_i64().unwrap();
	let mut link = Link::new(0);
	let central = Via::Ble(CENTRAL.into());
	let locked = held("iphone", Some(wait + 1));
	let first = link.arrive(&locked, &central, 1_000);
	assert_eq!(
		link.hold_ms(&locked, Transport::Ble, link::HOLD_MS, 1_000),
		wait,
		"its wait clamped to the Bluetooth link's most"
	);
	let on_wifi = held("iphone", Some(wait));
	link.arrive(&on_wifi, &Via::Lan, 2_000);
	assert_eq!(
		link.hold_ms(&on_wifi, Transport::Lan, link::HOLD_MS, 2_000),
		link::WAIT_MAX_MS,
		"on the LAN, the LAN's most"
	);
	link.answered(Device::Ios, "iphone", first, 2_001); // the earlier request on Bluetooth, answered at once
	let pending = link.arrive(&locked, &central, 3_000);
	let later = 3_000 + wait - 1;
	assert_eq!(link.holder(later), Some(Device::Ios), "alive past ALIVE_MS while its held request is pending");
	assert!(link.ble_linked(later) && link.quiet(later).is_empty(), "its session goes on meanwhile");
	link.answered(Device::Ios, "iphone", pending, later);
	assert_eq!(link.holder(later + link::ALIVE_MS), Some(Device::Ios), "ALIVE_MS after its answer went to the radio");
	assert_eq!(link.holder(later + link::ALIVE_MS + 1), None, "and no longer, though its central stays subscribed");
	let again = later + link::ALIVE_MS + 2;
	link.arrive(&locked, &central, again);
	let other = held("pixel", None);
	link.arrive(&other, &Via::Lan, again + 1);
	assert_eq!(link.holder_tag(again + 1).map(|(_, tag, _)| tag), Some("pixel".to_owned()), "the latest held request");
	assert!(!link.gone("another central"), "a central none came through");
	assert!(link.gone(CENTRAL));
	assert!(!link.ble_linked(again + 1));
	assert_eq!(
		link.holder(again + 1 + link::ALIVE_MS + 1),
		None,
		"the other phone's request aged, and none came again"
	);
	let back = link.arrive(&locked, &central, again + 2);
	assert!(link.gone(CENTRAL));
	link.answered(Device::Ios, "iphone", back, again + 3); // its call ended with the central: nothing left to answer
	assert_eq!(link.holder(again + 3), Some(Device::Ios), "the LAN phone takes the link at once");
	assert_eq!(link.holder_tag(again + 3).map(|(_, tag, _)| tag), Some("pixel".to_owned()));
}
