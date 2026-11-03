//! The pair link, end to end through questpump-core's public API: every link in `contract/fixtures/pair.json` reads
//! into its fields and is built back from them, and every refused one fails with the error the apps show.

#![expect(clippy::unwrap_used, reason = "a fixture that does not read fails the test loudly")]

use std::fmt::Write as _;
use std::path::PathBuf;

use questpump_core::pair::{PairError, PairLink};
use serde_json::Value as J;

fn fixture(name: &str) -> J {
	let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../contract/fixtures").join(name);
	serde_json::from_str(&std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))).unwrap()
}

fn hex(bytes: &[u8]) -> String {
	bytes.iter().fold(String::new(), |mut s, b| {
		let _ = write!(s, "{b:02x}");
		s
	})
}

#[test]
fn a_pair_link_carries_its_fields_and_refuses_one_out_of_bounds() {
	let pair = fixture("pair.json");
	for l in pair["links"].as_array().unwrap() {
		let link = PairLink::parse(l["link"].as_str().unwrap()).unwrap_or_else(|e| panic!("{}: {e}", l["about"]));
		assert_eq!(link.host.to_string(), l["host"].as_str().unwrap());
		assert_eq!(u64::from(link.port), l["port"].as_u64().unwrap());
		assert_eq!(hex(&link.key), l["key_hex"].as_str().unwrap());
		assert_eq!(link.name, l["name"].as_str().unwrap());
		assert_eq!(link.mdns.as_deref(), l["mdns"].as_str(), "{}: absent when the link has none", l["about"]);
		assert_eq!(link.hid.map(|h| hex(&h)).as_deref(), l["hid_hex"].as_str(), "{}", l["about"]);
		assert_eq!(link.to_url(), l["link"].as_str().unwrap(), "built back from its fields");
	}
	for r in pair["refused"].as_array().unwrap() {
		let err = PairLink::parse(r["link"].as_str().unwrap()).expect_err(r["about"].as_str().unwrap());
		assert_eq!(serde_json::to_value(err).unwrap(), r["error"], "{}", r["about"]);
		let _: PairError = err;
	}
}
