//! The LAN API end to end over HTTP on the loopback (docs/plan.md slice 7; `contract/companion-api.yaml`): the routes
//! as the helper serves them, a stand-in for its loop answering what they ask, and a client that seals every body for
//! its route and opens every answer as its route's, the way the phone and the watch do. The hold is 300 ms here, 5 s
//! in the helper.

#![expect(clippy::unwrap_used, reason = "a request that fails fails the test loudly")]

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use questpump_core::body::{Body, Changed, Kept};
use questpump_core::link::{
	self, AppRowsResponse, BodyRequest, BodyResponse, Device, EventType, HelloResponse, HistoryResponse, OwedMove,
	PairRequest, PairResponse, PingResponse, ProProof, Received, RefusedResponse, Sample, Sheet, SheetCharacter,
	SheetResponse, SheetsResponse, SyncRequest, SyncResponse, WristEvent,
};
use questpump_core::mirror::Held;
use questpump_core::protocol::{Command, RowRecord, SheetRecord, Source};
use questpump_core::row::Row;
use questpump_core::seal::{Keys, Route, Way};
use questpump_core::session::{CatchupMove, Now, People, Session, Zone};
use questpump_core::sheets::SheetLog;
use questpump_core::store::Store;
use questpump_helper_lib::lan::{self, Ask, Lan, Links, OfferFacts, SyncAnswer};
use reqwest::{Method, StatusCode};
use serde::de::DeserializeOwned;
use tokio::sync::{mpsc, watch};

const HOLD: Duration = Duration::from_millis(300);
/// The stand-in helper's install id, base64url of bytes 0-15.
const HID: &str = "AAECAwQFBgcICQoLDA0ODw";
const LOCKED_UNTIL: i64 = 1_790_049_600;

/// What the stand-in loop was asked.
#[derive(Default)]
struct Asked {
	paired: Vec<(Device, String)>,
	samples: Vec<(Device, Sample)>,
	events: Vec<(Device, WristEvent)>,
	/// Each sync's Pro proof as it reached the loop.
	proofs: Vec<Option<ProProof>>,
}

struct Server {
	url: String,
	keys: Keys,
	lan: Arc<Lan>,
	queue: Links,
	asked: Arc<Mutex<Asked>>,
	/// The session the stand-in loop pages the history and the sheets from.
	session: Arc<Mutex<Session>>,
	http: reqwest::Client,
	_stop: watch::Sender<bool>,
}

fn key() -> [u8; 32] {
	std::array::from_fn(|i| u8::try_from(i).unwrap())
}

/// The route a body at `path` is sealed for, its query left out.
fn route<'a>(way: Way, method: &'a Method, path: &'a str) -> Route<'a> {
	Route { way, method: method.as_str(), path: path.split_once('?').map_or(path, |(p, _)| p) }
}

/// A request's body sealed under `keys` for its route, as the phones seal it.
fn sealed(keys: &Keys, method: &Method, path: &str, plain: &[u8]) -> Vec<u8> {
	keys.seal.seal(&route(Way::Request, method, path), plain)
}

/// An answer opened under `keys` as its route's answer; none when it does not open so.
fn opened(keys: &Keys, method: &Method, path: &str, body: &[u8]) -> Option<Vec<u8>> {
	keys.seal.open(&route(Way::Answer, method, path), body).ok().map(|(plain, _)| plain)
}

/// A pump row, as the strip stores it.
fn pump(ts: u32) -> RowRecord {
	RowRecord::new(&Row { ts, char: 1, kind: 6, reps: 1, outcome: 7, ..Row::default() }, "t0", Source::Game)
}

/// A session whose store holds these rows.
fn session_of(rows: impl IntoIterator<Item = RowRecord>) -> Session {
	let mut store = Store::in_memory();
	for r in rows {
		store.add(r).1.unwrap();
	}
	Session::new(store, People::default(), Zone::fixed(0))
}

/// The routes on a free loopback port, and a loop that answers: the lock while `locked`, one character, one row.
async fn serve(locked: bool) -> Server {
	serve_with(locked, true).await
}

/// The same, with the loop answering or already gone (the helper quitting).
async fn serve_with(locked: bool, looping: bool) -> Server {
	serve_on(vec![tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap()], locked, looping)
}

/// The unfinished list's one move the sheet serves: an all-out window, as a helper listed it before it named the owed
/// row's time, ladder and rung.
fn owed_window() -> OwedMove {
	OwedMove { move_id: 7, kind: 3, seconds: Some(40), ladder: None, rung: None, allout: true, ts: None }
}

/// The routes on these listeners, reached through 127.0.0.1 at the first one's port.
#[expect(clippy::too_many_lines, reason = "one arm per ask of the loop, each the stand-in's answer")]
fn serve_on(listeners: Vec<tokio::net::TcpListener>, locked: bool, looping: bool) -> Server {
	let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, listeners[0].local_addr().unwrap().port()));
	let queue = Links::new(lan::now_ms());
	let (asks_tx, mut asks) = mpsc::unbounded_channel::<Ask>();
	let helper = link::Helper {
		name: "Test Mac".into(),
		host: "127.0.0.1".into(),
		port: addr.port(),
		version: "0.1.0".into(),
		t: 0,
		mdns: Some("Test-Mac.local".into()),
		hid: Some(HID.into()),
		wow: None,
		sight: None,
		step: None,
		ble: None,
	};
	let lan = Lan::new(&key(), queue.clone(), helper, asks_tx, HOLD);
	let asked = Arc::new(Mutex::new(Asked::default()));
	let seen = asked.clone();
	let session = Arc::new(Mutex::new(session_of([pump(1_790_000_000)])));
	let kept = session.clone();
	let mut shared = Kept::default();
	if looping {
		let told = lan.clone();
		tokio::spawn(async move {
			while let Some(ask) = asks.recv().await {
				match ask {
					Ask::Pair { device, name, done } => {
						seen.lock().unwrap().paired.push((device, name));
						let _ = done.send(());
					}
					Ask::Unpair { done, .. } => {
						let _ = done.send(());
					}
					Ask::Synced { request, done, .. } => {
						let SyncRequest { device, samples, events, offer_taken, game_offer_at, pro_proof, .. } =
							*request;
						let mut seen = seen.lock().unwrap();
						seen.samples.extend(samples.into_iter().map(|x| (device, x)));
						seen.events.extend(events.into_iter().map(|e| (device, e)));
						seen.proofs.push(pro_proof);
						// the helper's own answers come with the desktop's offers (slice 35), told to the routes
						// before the sync is answered and read as its response leaves: this loop withdraws the
						// offer taken and names the device's game offer as the latest
						told.set_offers(OfferFacts {
							locked_until: locked.then_some(LOCKED_UNTIL),
							offer_withdrawn: offer_taken.into_iter().collect(),
							playing: false,
							arms: false,
							game_offer_at,
							work: None,
						});
						let _ = done.send(SyncAnswer { rows_store: Some(String::new()), game_held: Vec::new() });
					}
					Ask::Body { body, done } => {
						shared.take(&body);
						let _ = done.send(shared.body.clone());
					}
					Ask::Sheet { done } => {
						let thrall = SheetCharacter {
							index: 1,
							guid: Some("Player-1-0A".into()),
							name: Some("Thrall".into()),
							realm: None,
							class: None,
							sheet: Some(serde_json::json!({"level": 12})),
						};
						let sheet = Sheet {
							unfinished: 1,
							held: None,
							locked_until: None,
							game_asks: false,
							owed: vec![owed_window()],
							characters: vec![thrall],
						};
						let _ = done.send(sheet);
					}
					Ask::History { after, proofs_since, size, done } => {
						let _ = done.send(kept.lock().unwrap().history(after, proofs_since, size));
					}
					Ask::Sheets { after, size, done } => {
						let _ = done.send(kept.lock().unwrap().sheet_log().page(after, size));
					}
					Ask::AppRows { after, size, done } => {
						let _ = done.send(kept.lock().unwrap().app_rows().page(after, size));
					}
					Ask::Account { done, .. } => {
						let _ = done.send(None);
					}
				}
			}
		});
	} else {
		drop(asks); // the loop is gone: every ask fails
	}
	let (stop, stopped) = watch::channel(false);
	tokio::spawn(lan::serve(listeners, lan.clone(), stopped));
	Server {
		url: format!("http://{addr}"),
		keys: Keys::derive(&key()),
		lan,
		queue,
		asked,
		session,
		http: reqwest::Client::new(),
		_stop: stop,
	}
}

impl Server {
	async fn post(&self, path: &str, body: &impl serde::Serialize) -> (StatusCode, Vec<u8>) {
		let plain = serde_json::to_vec(body).unwrap();
		self.post_raw(path, sealed(&self.keys, &Method::POST, path, &plain), &self.keys.bearer).await
	}

	async fn post_raw(&self, path: &str, body: Vec<u8>, bearer: &str) -> (StatusCode, Vec<u8>) {
		self.send_raw(Method::POST, path, body, bearer).await
	}

	async fn send_raw(&self, method: Method, path: &str, body: Vec<u8>, bearer: &str) -> (StatusCode, Vec<u8>) {
		let r = self
			.http
			.request(method, format!("{}{path}", self.url))
			.header("Authorization", format!("Bearer {bearer}"))
			.body(body)
			.send()
			.await
			.unwrap();
		(r.status(), r.bytes().await.unwrap().to_vec())
	}

	async fn get(&self, path: &str) -> (StatusCode, Vec<u8>) {
		let r = self
			.http
			.get(format!("{}{path}", self.url))
			.header("Authorization", format!("Bearer {}", self.keys.bearer))
			.send()
			.await
			.unwrap();
		(r.status(), r.bytes().await.unwrap().to_vec())
	}

	/// The answer to a request at `path`, opened as that route's.
	fn open<T: DeserializeOwned>(&self, method: &Method, path: &str, (status, body): (StatusCode, Vec<u8>)) -> T {
		assert_eq!(status, StatusCode::OK, "{method} {path}");
		serde_json::from_slice(&opened(&self.keys, method, path, &body).unwrap()).unwrap()
	}

	/// A sealed route's GET, its answer opened.
	async fn got<T: DeserializeOwned>(&self, path: &str) -> T {
		self.open(&Method::GET, path, self.get(path).await)
	}

	/// A sealed route's POST, its answer opened.
	async fn posted<T: DeserializeOwned>(&self, path: &str, body: &impl serde::Serialize) -> T {
		self.open(&Method::POST, path, self.post(path, body).await)
	}

	async fn sync(&self, req: &SyncRequest) -> SyncResponse {
		self.posted("/v1/sync", req).await
	}

	/// A device's first held request, which the helper answers at once since nothing says what it heard: the `since`
	/// its next held request continues from, held then.
	async fn first(&self, device: Device) -> u64 {
		let started = Instant::now();
		let answered = self.sync(&held(device, 0, 0)).await;
		assert!(started.elapsed() < HOLD, "a client the helper told nothing yet is answered at once");
		answered.next
	}
}

fn held(device: Device, t0: i64, since: u64) -> SyncRequest {
	SyncRequest {
		device,
		client: format!("{}-test", device.name()),
		t0,
		sent_at: lan::now_ms(),
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

#[tokio::test]
async fn a_phone_pairs_pings_and_reads_the_sheet_the_history_and_the_app_rows_through_sealed_bodies() {
	let s = serve(false).await;
	let pairing = PairRequest { device: Device::Ios, name: " David's iPhone ".into(), sent_at: lan::now_ms() };
	let paired: PairResponse = s.posted("/v1/pair", &pairing).await;
	assert_eq!((paired.helper.name.as_str(), paired.helper.port > 0), ("Test Mac", true));
	assert_eq!(s.asked.lock().unwrap().paired, [(Device::Ios, "David's iPhone".to_owned())], "the name, trimmed");
	let before = lan::now_ms();
	let ping: PingResponse = s.got("/v1/ping?t0=123").await;
	assert!(ping.t0 == 123 && before <= ping.t1 && ping.t1 <= ping.t2, "the t0 echoed, then the helper's two times");
	let sheet: SheetResponse = s.got("/v1/sheet").await;
	assert_eq!((sheet.sheet.unfinished, sheet.sheet.characters[0].name.as_deref()), (1, Some("Thrall")));
	let owed = &sheet.sheet.owed[0];
	assert_eq!((owed.move_id, owed.seconds, owed.allout), (7, Some(40), true), "the unfinished list, flattened in");
	let history: HistoryResponse = s.got("/v1/history").await;
	assert_eq!((history.page.rows.len(), history.page.store.as_str(), history.page.more), (1, "t0/1790000000", false));
	let app_rows: AppRowsResponse = s.got("/v1/app_rows").await;
	assert_eq!((app_rows.page.rows.len(), app_rows.page.store.as_str()), (0, ""), "no phone sent a row yet");
}

#[tokio::test]
async fn the_history_comes_a_page_at_a_time_and_a_reset_starts_a_new_store() {
	let s = serve(false).await;
	*s.session.lock().unwrap() = session_of((1_790_000_000..).take(link::HISTORY_PAGE + 1).map(pump));
	let first: HistoryResponse = s.got("/v1/history").await;
	let page = &first.page;
	assert_eq!((page.rows.len(), page.next, page.more, page.proofs.as_ref()), (5_000, 5_000, true, None));
	let last: HistoryResponse = s.got("/v1/history?after=5000&proofs_since=1790000000").await;
	assert_eq!((last.page.rows[0].row_ts, last.page.next, last.page.more), (1_790_005_000, 5_001, false));
	assert_eq!(
		(&last.page.store, last.page.proofs.as_deref()),
		(&page.store, Some(&[][..])),
		"asked for: none out yet"
	);
	// a save taken in that holds a reset: the store starts again with the reset row
	let reset = Row { ts: 1_790_000_100, char: 1, kind: 14, ..Row::default() };
	let held = Held { saved_at: 1_790_000_100, rows: vec![reset], ..Held::default() };
	s.session.lock().unwrap().adopt(&held, "", &Now { unix: 1_790_009_000, rfc3339: "t1".into() });
	let after: HistoryResponse = s.got("/v1/history?after=5001").await;
	assert_ne!(after.page.store, page.store, "the phone sees a new store and asks again from 0");
	assert_eq!((after.page.rows.len(), after.page.next), (0, 4_902), "the reset row and the 4,901 rows from it on");
}

#[tokio::test]
async fn the_sheets_come_a_page_at_a_time_in_the_order_the_helper_read_them() {
	let s = serve(false).await;
	let mut log = SheetLog::in_memory();
	for at in (1_790_000_000..).take(link::SHEETS_PAGE + 1) {
		let sheet = serde_json::json!({"char": 1, "at": at, "lvl": at}); // a new level each: none repeats the last
		log.add(SheetRecord::new(format!("t{at}"), 1, &sheet).unwrap()).unwrap();
	}
	s.session.lock().unwrap().set_sheet_log(log);
	let first: SheetsResponse = s.got("/v1/sheets").await;
	assert_eq!((first.store.as_str(), first.next, first.more), ("t1790000000", 1_000, true));
	let last: SheetsResponse = s.got("/v1/sheets?after=1000").await;
	let read = last.sheets.iter().map(|r| r.ts_helper.as_str()).collect::<Vec<_>>();
	assert_eq!((read, last.next, last.more), (vec!["t1790001000"], 1_001, false));
}

#[tokio::test]
async fn a_held_sync_answers_the_moment_a_command_is_queued_and_the_next_request_acknowledges_it() {
	let s = Arc::new(serve(false).await);
	let beat = Sample {
		sample_end: lan::now_ms(),
		sent_at: None,
		bpm: Some(72),
		steps: 3,
		metres: None,
		cadence: None,
		pump_id: None,
	};
	let since = s.first(Device::Watchos).await;
	let first = SyncRequest { samples: vec![beat.clone()], ..held(Device::Watchos, 1, since) };
	let started = Instant::now();
	let waiting = tokio::spawn({
		let s = s.clone();
		async move { s.sync(&first).await }
	});
	tokio::time::sleep(Duration::from_millis(60)).await;
	let queued = s.queue.queue(Command::TurnIn { quest_id: 26391, tier: 2 }, lan::now_ms()).unwrap();
	let answered = waiting.await.unwrap();
	assert!(started.elapsed() < HOLD, "answered at the queueing, not at the hold's end");
	assert_eq!((answered.t0, answered.next), (1, queued.seq));
	assert_eq!(answered.commands, std::slice::from_ref(&queued));
	assert_eq!(s.asked.lock().unwrap().samples, [(Device::Watchos, beat)], "the watch's heart rate reached the loop");
	let receipt = Received { seq: queued.seq, at: lan::now_ms() };
	let again = SyncRequest { received: vec![receipt], ..held(Device::Watchos, 2, answered.next) };
	let started = Instant::now();
	let quiet = s.sync(&again).await;
	assert!(quiet.commands.is_empty() && started.elapsed() >= HOLD, "acknowledged: the next request holds for nothing");
	assert_eq!(quiet.next, queued.seq);
}

#[tokio::test]
async fn a_second_held_request_from_the_same_device_answers_the_first_at_once() {
	let s = Arc::new(serve(false).await);
	let since = s.first(Device::Watchos).await;
	let started = Instant::now();
	let older = tokio::spawn({
		let s = s.clone();
		async move { s.sync(&held(Device::Watchos, 1, since)).await }
	});
	tokio::time::sleep(Duration::from_millis(60)).await;
	let newer = tokio::spawn({
		let s = s.clone();
		async move { s.sync(&held(Device::Watchos, 2, since)).await }
	});
	let older = older.await.unwrap();
	assert!(started.elapsed() < HOLD && older.commands.is_empty(), "the older one answered without commands");
	let queued = s.queue.queue(Command::QuestComplete { quest_id: 7 }, lan::now_ms()).unwrap();
	assert_eq!(newer.await.unwrap().commands, [queued], "the newer one is the watch's link");
}

#[tokio::test]
async fn a_phones_plain_post_beside_the_watch_answers_at_once_and_its_catch_up_pump_reaches_the_loop() {
	let s = serve(true).await;
	let watch = tokio::spawn({
		let url = s.url.clone();
		let body =
			sealed(&s.keys, &Method::POST, "/v1/sync", &serde_json::to_vec(&held(Device::Watchos, 1, 0)).unwrap());
		let bearer = s.keys.bearer.clone();
		async move {
			reqwest::Client::new()
				.post(format!("{url}/v1/sync"))
				.header("Authorization", format!("Bearer {bearer}"))
				.body(body)
				.send()
				.await
		}
	});
	tokio::time::sleep(Duration::from_millis(60)).await;
	let _ = s.queue.queue(Command::TurnIn { quest_id: 1, tier: 1 }, lan::now_ms());
	let pump = WristEvent {
		kind: EventType::PumpDone,
		pump_id: "p-1b4e".into(),
		tier: None,
		start_at: 1_790_000_040_000,
		seconds: None,
		steps: None,
		metres: None,
		peak_bpm: None,
		delta_bpm: None,
		verdict: None,
		slot: None,
		moves: Some(vec![CatchupMove {
			seconds: 40,
			move_id: 12,
			ts: None,
			set: None,
			ran: None,
			kind: None,
			ladder: None,
			rung: None,
			allout: false,
		}]),
		unwell: false,
		stopped_at: None,
	};
	let plain = SyncRequest { events: vec![pump.clone()], hold: Some(false), ..held(Device::Ios, 5, 0) };
	let started = Instant::now();
	let answered = s.sync(&plain).await;
	assert!(started.elapsed() < HOLD, "a plain post is answered at once");
	assert!(answered.commands.is_empty() && answered.next == 0, "no commands, and its own since back");
	assert_eq!(answered.locked_until, Some(LOCKED_UNTIL), "the felt-unwell lock rides in every response");
	assert!(s.asked.lock().unwrap().events.contains(&(Device::Ios, pump)));
	assert_eq!(watch.await.unwrap().unwrap().status(), StatusCode::OK);
}

#[tokio::test]
async fn an_offer_taken_comes_back_withdrawn_and_a_pro_proof_that_does_not_read_never_fails_the_request() {
	let s = serve(false).await;
	let offer = "m-3f2b8c1e-7a4d-4e5f-9b6a-0c1d2e3f4a5b";
	let taken = SyncRequest {
		offer_taken: Some(offer.into()),
		game_offer_at: Some(1_790_000_701),
		hold: Some(false),
		..held(Device::Android, 1, 0)
	};
	let answered = s.sync(&taken).await;
	assert_eq!(
		(answered.offer_withdrawn.as_slice(), answered.game_offer_at),
		(&[offer.to_owned()][..], Some(1_790_000_701)),
		"the loop's answer rides in the plain post's response"
	);
	// a newer phone's proof of a store this helper does not know: the request reads, without the proof
	let mut body = serde_json::to_value(SyncRequest { hold: Some(false), ..held(Device::Ios, 2, 0) }).unwrap();
	body["pro_proof"] = serde_json::json!({"store": "kiosk", "receipt": 7});
	assert_eq!(s.post("/v1/sync", &body).await.0, StatusCode::OK, "never a 422 that drops what the request carried");
	assert_eq!(s.asked.lock().unwrap().proofs, [None, None]);
}

#[tokio::test]
async fn a_phones_body_merges_field_by_field_through_the_sealed_route_and_a_watch_is_refused() {
	let s = serve(false).await;
	let switch = |value, changed_at| Some(Changed { value, changed_at });
	let first = Body {
		no_jump: switch(true, 1_790_000_300_000),
		experience: Some(Changed { value: "training".into(), changed_at: 1_790_000_200_000 }),
		..Body::default()
	};
	let request = |device, body| BodyRequest { device, sent_at: lan::now_ms(), body };
	let answer: BodyResponse = s.posted("/v1/body", &request(Device::Android, first.clone())).await;
	assert_eq!(answer.body, first, "the helper had none: the phone's fields are the body");
	let later = Body {
		no_jump: switch(false, 1_790_000_100_000),
		no_strain: switch(true, 1_790_000_400_000),
		experience: Some(Changed { value: "expert".into(), changed_at: 1_790_000_500_000 }),
		..Body::default()
	};
	let answer: BodyResponse = s.posted("/v1/body", &request(Device::Ios, later)).await;
	let merged = Body { no_strain: switch(true, 1_790_000_400_000), ..first };
	assert_eq!(answer.body, merged, "an older change loses, a later one wins, an unknown experience is dropped alone");
	let mut unread = serde_json::to_value(request(Device::Ios, Body::default())).unwrap();
	unread["body"]["moves"] =
		serde_json::json!({"value": {"exclude": {"ids": [70_000], "groups": []}}, "changed_at": 1});
	let answer: BodyResponse = s.posted("/v1/body", &unread).await;
	assert!(answer.body.moves.is_none(), "a move id out of range drops its field, never the request");
	let watch = request(Device::Watchos, Body::default());
	assert_eq!(s.post("/v1/body", &watch).await.0, StatusCode::UNPROCESSABLE_ENTITY, "its phone relays its profile");
}

#[tokio::test]
async fn a_sync_the_helpers_loop_cannot_take_is_refused_so_the_device_keeps_its_events() {
	let s = serve_with(false, false).await;
	let plain = SyncRequest { hold: Some(false), ..held(Device::Watchos, 1, 0) };
	assert_eq!(s.post("/v1/sync", &plain).await.0, StatusCode::SERVICE_UNAVAILABLE, "never a 200 that confirms them");
}

#[tokio::test]
async fn the_helper_refuses_a_wrong_bearer_a_replayed_body_and_a_body_that_does_not_read() {
	let s = serve(false).await;
	let post = Method::POST;
	let body = sealed(&s.keys, &post, "/v1/sync", &serde_json::to_vec(&held(Device::Ios, 1, 0)).unwrap());
	let plain_request = SyncRequest { hold: Some(false), ..held(Device::Ios, 1, 0) };
	let replayed = sealed(&s.keys, &post, "/v1/sync", &serde_json::to_vec(&plain_request).unwrap());
	assert_eq!(s.post_raw("/v1/sync", replayed.clone(), &s.keys.bearer).await.0, StatusCode::OK);
	assert_eq!(
		s.post_raw("/v1/sync", replayed, &s.keys.bearer).await.0,
		StatusCode::BAD_REQUEST,
		"the same body again"
	);
	let other = sealed(&Keys::derive(&[9; 32]), &post, "/v1/sync", b"{}");
	assert_eq!(
		s.post_raw("/v1/sync", other, &s.keys.bearer).await.0,
		StatusCode::BAD_REQUEST,
		"sealed under another key"
	);
	let unknown = sealed(&s.keys, &post, "/v1/sync", br#"{"device":"toaster","t0":1,"since":0}"#);
	let (status, refusal) = s.post_raw("/v1/sync", unknown.clone(), &s.keys.bearer).await;
	assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
	// a body that does not read is refused sealed as the sync's answer
	let refused = opened(&s.keys, &post, "/v1/sync", &refusal).unwrap();
	assert_eq!(
		serde_json::from_slice::<RefusedResponse>(&refused).unwrap(),
		RefusedResponse::of(unknown[..12].try_into().unwrap()),
		"naming that request alone, the one 422 a device drops what it sent on"
	);
	// a sync's request taken to another sealed route, or sent back as the sync's answer, does not open: no body of one
	// route or way reads as another's
	let request = serde_json::to_vec(&plain_request).unwrap();
	let moved = sealed(&s.keys, &post, "/v1/sync", &request);
	for (method, path) in [(Method::POST, "/v1/pair"), (Method::POST, "/v1/body"), (Method::PUT, "/v1/account")] {
		assert_eq!(
			s.send_raw(method, path, moved.clone(), &s.keys.bearer).await,
			(StatusCode::BAD_REQUEST, Vec::new()),
			"{path}: refused unopened"
		);
	}
	let turned = s.keys.seal.seal(&route(Way::Answer, &post, "/v1/sync"), &request);
	assert_eq!(
		s.post_raw("/v1/sync", turned, &s.keys.bearer).await.0,
		StatusCode::BAD_REQUEST,
		"an answer as a request"
	);
	let mut extra = serde_json::to_value(PairRequest { device: Device::Ios, name: "P".into(), sent_at: lan::now_ms() });
	extra.as_mut().unwrap()["admin"] = serde_json::Value::Bool(true);
	assert_eq!(
		s.post("/v1/pair", &extra.unwrap()).await,
		(StatusCode::UNPROCESSABLE_ENTITY, Vec::new()),
		"a field the pairing does not know: refused bare"
	);
	let watch_pairing = PairRequest { device: Device::Watchos, name: "Watch".into(), sent_at: lan::now_ms() };
	let stale = SyncRequest { sent_at: lan::now_ms() - link::FRESH_MS - 1_000, ..plain_request.clone() };
	assert_eq!(s.post("/v1/sync", &stale).await.0, StatusCode::BAD_REQUEST, "a recorded request replayed later");
	let earlier_run = SyncRequest { sent_at: lan::now_ms() - 5_000, ..plain_request.clone() };
	assert_eq!(
		s.post("/v1/sync", &earlier_run).await.0,
		StatusCode::BAD_REQUEST,
		"sent before this helper started: replayed after a restart, whose replay guard began empty"
	);
	assert_eq!(
		s.post("/v1/pair", &watch_pairing).await.0,
		StatusCode::UNPROCESSABLE_ENTITY,
		"a watch pairs through its phone"
	);
	assert_eq!(s.get("/v1/ping").await.0, StatusCode::UNPROCESSABLE_ENTITY, "no t0");
	for page in
		["/v1/history?after=-1", "/v1/history?proofs_since=today", "/v1/sheets?after=-1", "/v1/app_rows?after=x"]
	{
		assert_eq!(s.get(page).await.0, StatusCode::UNPROCESSABLE_ENTITY, "{page}: a place that does not read");
	}
	for _ in 0..link::THROTTLE_AFTER {
		assert_eq!(s.post_raw("/v1/sync", body.clone(), "wrong").await.0, StatusCode::UNAUTHORIZED);
	}
	let (status, _) = s.post_raw("/v1/sync", body, &s.keys.bearer).await;
	assert_eq!(status, StatusCode::TOO_MANY_REQUESTS, "ten wrong bearers: refused for a minute, the right one too");
}

#[tokio::test]
async fn the_holders_plain_post_carries_no_commands_and_leaves_its_held_request_to_deliver_them() {
	let s = Arc::new(serve(false).await);
	let since = s.first(Device::Watchos).await;
	let waiting = tokio::spawn({
		let s = s.clone();
		async move { s.sync(&held(Device::Watchos, 1, since)).await }
	});
	tokio::time::sleep(Duration::from_millis(60)).await;
	let queued = s.queue.queue(Command::QuestComplete { quest_id: 7 }, lan::now_ms()).unwrap();
	let last_word = SyncRequest { hold: Some(false), ..held(Device::Watchos, 2, 0) };
	let answered = s.sync(&last_word).await;
	assert!(answered.commands.is_empty() && answered.next == 0, "a plain post acknowledges nothing, even the holder's");
	assert_eq!(waiting.await.unwrap().commands, [queued], "the held request still delivers the command");
}

#[tokio::test]
async fn a_new_pairing_code_refuses_the_old_key_on_a_connection_already_open() {
	let s = serve(false).await;
	let plain = |t0| SyncRequest { hold: Some(false), ..held(Device::Watchos, t0, 0) };
	assert_eq!(s.post("/v1/sync", &plain(1)).await.0, StatusCode::OK, "the connection is open and kept alive");
	let new_key = [7; 32];
	s.lan.rekey(&new_key);
	assert_eq!(s.post("/v1/sync", &plain(2)).await.0, StatusCode::UNAUTHORIZED, "the old bearer, on the same client");
	let keys = Keys::derive(&new_key);
	let body = sealed(&keys, &Method::POST, "/v1/sync", &serde_json::to_vec(&plain(3)).unwrap());
	let (status, answer) = s.post_raw("/v1/sync", body, &keys.bearer).await;
	assert_eq!(status, StatusCode::OK, "a device that scanned the new code");
	let (new, old) =
		(opened(&keys, &Method::POST, "/v1/sync", &answer), opened(&s.keys, &Method::POST, "/v1/sync", &answer));
	assert!(new.is_some() && old.is_none(), "only the new key opens the answer");
}

#[tokio::test]
async fn a_body_over_the_limit_is_refused_as_too_large() {
	let s = serve(false).await;
	let (status, _) = s.post_raw("/v1/sync", vec![0; 257 * 1024], &s.keys.bearer).await;
	assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
}

#[tokio::test]
async fn one_address_holds_at_most_eight_connections_and_a_closed_one_frees_its_place() {
	use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
	let s = serve(false).await;
	let addr = s.url.trim_start_matches("http://").to_owned();
	let mut open = Vec::new();
	for _ in 0..8 {
		open.push(tokio::net::TcpStream::connect(&addr).await.unwrap());
	}
	let ping = format!("GET /v1/ping?t0=1 HTTP/1.1\r\nHost: {addr}\r\nAuthorization: Bearer {}\r\n\r\n", s.keys.bearer);
	let answer = async |stream: &mut tokio::net::TcpStream| {
		let _ = stream.write_all(ping.as_bytes()).await;
		let mut head = [0; 12];
		let n = tokio::time::timeout(Duration::from_secs(1), stream.read(&mut head)).await.unwrap().unwrap_or(0);
		String::from_utf8_lossy(&head[..n]).into_owned()
	};
	let mut ninth = tokio::net::TcpStream::connect(&addr).await.unwrap();
	assert_eq!(answer(&mut ninth).await, "", "the ninth is closed at accept");
	drop(open.pop());
	tokio::time::sleep(Duration::from_millis(50)).await; // the server sees the close and frees the place
	let mut again = tokio::net::TcpStream::connect(&addr).await.unwrap();
	assert_eq!(answer(&mut again).await, "HTTP/1.1 200");
}

/// The challenge the client draws for its hellos here: bytes c0-cf.
const CHALLENGE: [u8; link::HELLO_BYTES] =
	[0xc0, 0xc1, 0xc2, 0xc3, 0xc4, 0xc5, 0xc6, 0xc7, 0xc8, 0xc9, 0xca, 0xcb, 0xcc, 0xcd, 0xce, 0xcf];

impl Server {
	/// `GET /v1/hello` at `base` with `CHALLENGE`, with no bearer.
	async fn hello_at(&self, base: &str) -> (StatusCode, Option<HelloResponse>, String) {
		let c = URL_SAFE_NO_PAD.encode(CHALLENGE);
		let r = self.http.get(format!("{base}/v1/hello?c={c}")).send().await.unwrap();
		let (status, kind) = (r.status(), r.headers().get("content-type").map(|v| v.to_str().unwrap().to_owned()));
		(status, serde_json::from_slice(&r.bytes().await.unwrap()).ok(), kind.unwrap_or_default())
	}

	/// A request's status line at the server's port with this `Host` line (none for `None`), on a connection of its own.
	async fn status_with_host(&self, host: Option<&str>, path: &str) -> String {
		use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
		let addr = self.url.trim_start_matches("http://");
		let mut stream = tokio::net::TcpStream::connect(addr).await.unwrap();
		let host = host.map(|h| format!("Host: {h}\r\n")).unwrap_or_default();
		let head = format!("GET {path} HTTP/1.1\r\n{host}Authorization: Bearer {}\r\n\r\n", self.keys.bearer);
		stream.write_all(head.as_bytes()).await.unwrap();
		let mut line = [0; 12];
		let n = tokio::time::timeout(Duration::from_secs(1), stream.read(&mut line)).await.unwrap().unwrap_or(0);
		String::from_utf8_lossy(&line[..n]).into_owned()
	}
}

/// The hello the test's install gives to `CHALLENGE` under `key`, its own bytes `n`.
fn hello_of(key: &[u8; 32], n: &str) -> Option<HelloResponse> {
	let n: [u8; link::HELLO_BYTES] = URL_SAFE_NO_PAD.decode(n).unwrap().try_into().unwrap();
	HelloResponse::of(HID, &Keys::derive(key).hello, &CHALLENGE, &n)
}

#[tokio::test]
async fn hello_proves_the_install_and_the_key_without_a_bearer_and_names_nothing_of_the_computer() {
	let s = serve(false).await;
	let (status, hello, kind) = s.hello_at(&s.url).await;
	assert_eq!((status, kind.as_str()), (StatusCode::OK, "application/json"), "plain JSON, never sealed");
	let first = hello.unwrap();
	assert_eq!(Some(&first), hello_of(&key(), &first.n).as_ref());
	let again = s.hello_at(&s.url).await.1.unwrap();
	assert!(again.n != first.n && again.h != first.h && again.k != first.k, "no two hellos alike");
	let r = s.http.get(format!("{}/v1/hello?c={}", s.url, URL_SAFE_NO_PAD.encode(CHALLENGE))).send().await.unwrap();
	let body: serde_json::Value = r.json().await.unwrap();
	let mut fields: Vec<&str> = body.as_object().unwrap().keys().map(String::as_str).collect();
	fields.sort_unstable();
	assert_eq!(fields, ["h", "k", "n", "v"], "the proofs alone: no id, name, .local name or port to follow");
	for query in ["", "?c=", "?c=AAEC", "?c=not*base64*at*all*"] {
		let r = s.http.get(format!("{}/v1/hello{query}", s.url)).send().await.unwrap();
		assert_eq!(r.status(), StatusCode::UNPROCESSABLE_ENTITY, "{query}: no challenge of 16 bytes");
	}
	// the computer renamed and moved: the sealed helper block says so, to the paired phone alone
	s.lan.set_address("Büro", Some(IpAddr::from([10, 0, 0, 7])), None);
	let ping: PingResponse = s.got("/v1/ping?t0=1").await;
	let block = (ping.helper.name.as_str(), ping.helper.host.as_str(), ping.helper.mdns, ping.helper.hid);
	assert_eq!(block, ("Büro", "10.0.0.7", None, Some(HID.to_owned())));
	let new_key = [7; 32];
	s.lan.rekey(&new_key);
	let rekeyed = s.hello_at(&s.url).await.1.unwrap();
	assert_eq!(Some(&rekeyed), hello_of(&new_key, &rekeyed.n).as_ref(), "the new code's key");
	assert_ne!(Some(&rekeyed), hello_of(&key(), &rekeyed.n).as_ref(), "never the old key's");
	let sealed = ["/v1/ping?t0=2", "/v1/sheet", "/v1/history", "/v1/sheets", "/v1/app_rows"];
	for path in sealed.iter().cycle().take(link::THROTTLE_AFTER) {
		assert_eq!(s.get(path).await.0, StatusCode::UNAUTHORIZED, "{path}: the old bearer");
	}
	assert_eq!(s.hello_at(&s.url).await.0, StatusCode::TOO_MANY_REQUESTS, "under the same throttle as every route");
}

#[tokio::test]
async fn a_request_whose_host_names_another_computer_is_misdirected() {
	let s = serve(false).await;
	let port = s.url.rsplit(':').next().unwrap();
	let c = URL_SAFE_NO_PAD.encode(CHALLENGE);
	for host in [format!("127.0.0.1:{port}"), format!("localhost:{port}"), format!("Test-Mac.local:{port}")] {
		assert_eq!(s.status_with_host(Some(&host), "/v1/ping?t0=1").await, "HTTP/1.1 200", "{host}");
	}
	assert_eq!(s.status_with_host(Some(&format!("TEST-MAC.LOCAL.:{port}")), "/v1/sheet").await, "HTTP/1.1 200");
	assert_eq!(s.status_with_host(Some(&format!("[::1]:{port}")), "/v1/sheet").await, "HTTP/1.1 200");
	for host in [
		Some(format!("attacker.example:{port}")),
		Some(format!("127.0.0.1.attacker.example:{port}")),
		Some(format!("Other-Mac.local:{port}")),
		Some("127.0.0.1:1".to_owned()),
		Some("127.0.0.1".to_owned()),
		Some("[::1".to_owned()),
		None,
	] {
		for path in ["/v1/ping?t0=1".to_owned(), format!("/v1/hello?c={c}")] {
			assert_eq!(s.status_with_host(host.as_deref(), &path).await, "HTTP/1.1 421", "{host:?} {path}");
		}
	}
}

#[tokio::test]
async fn the_routes_answer_on_ipv4_and_ipv6_at_one_port() {
	let listeners = lan::bind(Ipv4Addr::LOCALHOST, Ipv6Addr::LOCALHOST, 0).unwrap();
	let ports: Vec<u16> = listeners.iter().map(|l| l.local_addr().unwrap().port()).collect();
	assert!(ports.len() == 2 && ports[0] == ports[1], "{ports:?}");
	let s = serve_on(listeners, false, true);
	for base in [format!("http://127.0.0.1:{}", ports[0]), format!("http://[::1]:{}", ports[0])] {
		let (status, hello, _) = s.hello_at(&base).await;
		let hello = hello.unwrap();
		let mine = hello_of(&key(), &hello.n);
		assert_eq!((status, Some(&hello)), (StatusCode::OK, mine.as_ref()), "{base}");
	}
	let ping: PingResponse = s.got("/v1/ping?t0=1").await;
	assert_eq!(ping.t0, 1, "the sealed routes answer as before");
}
