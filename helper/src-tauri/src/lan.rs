//! The LAN API the phone and the watch call (docs/plan.md slice 7; `contract/companion-api.yaml` is the source of
//! truth; rust-helper `protocol-and-mirror.md`, "The LAN API", "Serving" and "The held sync exchange"): plain HTTP on
//! every address of the computer, port 8787, every request with the derived bearer and every body sealed for its route
//! (the guard seals every answer, `Sealed` opens every request), except `GET /v1/hello`, which proves to a device
//! looking for its helper who answers before its bearer goes out, and every request's `Host` naming the computer
//! (`host`). The rules live in questpump-core (`seal`, `link`, `body`); this module serves them and asks the helper's
//! loop, which owns the session, the settings and the body every app device shares, for what only it knows (`Ask`). A
//! refused connection or request, a pairing request and a hello are logged at most once a minute per peer. The same
//! routes serve the Bluetooth link (slice 43): `router` is what both listeners call, and the bridge (`blelink`) hands
//! it each request it opened with the bearer added and its central as the request's `Peer`, which the guard, the
//! throttle, the refusal log and the sync's transport read in place of an address. It knows nothing of Tauri, so its
//! tests serve it on the loopback.

use std::collections::HashMap;
use std::fmt;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::task::{Context, Poll};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use axum::Router;
use axum::body::Bytes;
use axum::extract::DefaultBodyLimit;
use axum::extract::connect_info::Connected;
use axum::extract::rejection::QueryRejection;
use axum::extract::{ConnectInfo, FromRequest, FromRequestParts, Query, Request, State};
use axum::http::request::Parts;
use axum::http::uri::Authority;
use axum::http::{HeaderMap, HeaderValue, Method, StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use hyper_util::rt::{TokioExecutor, TokioIo, TokioTimer};
use hyper_util::server::conn::auto;
use hyper_util::service::TowerToHyperService;
use questpump_core::ble;
use questpump_core::body::Body;
use questpump_core::link::{
	self, AccountRequest, AccountResponse, AppRowsPage, AppRowsResponse, Ble, BodyRequest, BodyResponse, Device,
	Exchange, HelloResponse, HistoryPage, HistoryResponse, Link, PairRequest, PairResponse, PingResponse, Queued,
	RefusedResponse, Sheet, SheetResponse, SheetsPage, SheetsResponse, Step, Summary, SyncRequest, SyncResponse,
	Throttle, Transport, UnpairRequest, UnpairResponse, Unreached, Via, Wow,
};
use questpump_core::mirror::Sight;
use questpump_core::pair::is_private;
use questpump_core::protocol::Command;
use questpump_core::seal::{Keys, Replay, Route, Way};
use questpump_core::sync;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use socket2::{Domain, Protocol, Socket, Type};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{OwnedSemaphorePermit, Semaphore, mpsc, oneshot, watch};
use tower::Service as _;

/// A phone name longer than this is cut (it is only shown in the helper's window).
const NAME_CHARS: usize = 64;
/// An Unpair's `client` longer than this is no app install's id (a UUID): refused, so no request grows what the loop
/// keeps of it.
const CLIENT_CHARS: usize = 64;
/// The largest body: a sync carrying an hour of kept windows is about 100 KB, and one carrying a phone's 500 app rows
/// (the most it sends at once) about 30 KB.
const BODY_BYTES: usize = 256 * 1024;
/// A body that takes longer to arrive is refused (hyper times only the head).
const BODY_TIME: Duration = Duration::from_secs(10);
/// Open connections at a time, in all and from one address: a device holds one, a phone that proxies its watch two,
/// so a LAN device without the key cannot use up the helper's file descriptors (macOS starts an app with 256;
/// `raise_open_files` lifts the soft limit to 10240).
const CONNECTIONS: usize = 32;
const CONNECTIONS_PER_PEER: usize = 8;
/// A peer's refusals, pairing requests and hellos are each logged at most once this often.
const NOTE_EVERY_MS: i64 = 60_000;
/// The most peers the refusal log keeps a minute for: past it a new peer's line is skipped, so many source addresses
/// never grow the map or the log.
const NOTED_PEERS: usize = 256;
/// The pairing route, whose every attempt is logged.
const PAIR: &str = "/v1/pair";
/// The paged routes (`link::page_size`): over the Bluetooth link their answers go after every other (`is_page`).
const HISTORY: &str = "/v1/history";
const SHEETS: &str = "/v1/sheets";
const APP_ROWS: &str = "/v1/app_rows";
/// The most of a device's `User-Agent` a link's first line shows.
const AGENT_CHARS: usize = 120;

/// The helper's wall clock in milliseconds.
#[must_use]
pub fn now_ms() -> i64 {
	SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX))
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
	m.lock().unwrap_or_else(PoisonError::into_inner)
}

/// The command queue, shared by the loop (which queues what the strip shows) and the routes (which deliver it); every
/// change wakes the held requests.
#[derive(Clone)]
pub struct Links {
	link: Arc<Mutex<Link>>,
	wake: watch::Sender<u64>,
}

impl Links {
	#[must_use]
	pub fn new(now: i64) -> Self {
		Self { link: Arc::new(Mutex::new(Link::new(now))), wake: watch::Sender::new(0) }
	}

	fn bump(&self) {
		self.wake.send_modify(|n| *n = n.wrapping_add(1));
	}

	/// Queues a command for the device that holds the link; none when it expired already.
	#[must_use]
	pub fn queue(&self, command: Command, now: i64) -> Option<Queued> {
		let queued = lock(&self.link).queue(command, now);
		if queued.is_some() {
			self.bump();
		}
		queued
	}

	/// Queues the pump on offer again, ahead of the waiting commands, when the device that holds the link never heard
	/// it (`Link::reoffer`).
	#[must_use]
	pub fn reoffer(&self, offer: Command, now: i64) -> Option<Queued> {
		let queued = lock(&self.link).reoffer(offer, now);
		if queued.is_some() {
			self.bump();
		}
		queued
	}

	/// Takes a match's offer off the link (a device or the computer started its pump, the computer card's Later, a move
	/// done since): whether it still waited there (`Link::withdraw`). One taken off wakes the held requests, as one
	/// queued does (slice 48's second fix round).
	#[must_use]
	pub fn withdraw(&self, offer: &str) -> bool {
		let withdrawn = lock(&self.link).withdraw(offer);
		if withdrawn {
			self.bump();
		}
		withdrawn
	}

	#[must_use]
	pub fn holder(&self, now: i64) -> Option<Device> {
		lock(&self.link).holder(now)
	}

	/// The device that holds the link with its client's tag and transport, for the log (`Link::holder_tag`).
	#[must_use]
	pub fn holder_tag(&self, now: i64) -> Option<(Device, String, Transport)> {
		lock(&self.link).holder_tag(now)
	}

	/// The device that holds the link and its transport, for a pump's log line (`Link::holder_via`).
	#[must_use]
	pub fn holder_via(&self, now: i64) -> Option<(Device, Transport)> {
		lock(&self.link).holder_via(now)
	}

	/// Whether a client is linked over Bluetooth now (`Link::ble_linked`).
	#[must_use]
	pub fn ble_linked(&self, now: i64) -> bool {
		lock(&self.link).ble_linked(now)
	}

	/// A held request over Bluetooth is no longer pending (`Link::answered`): `Held`'s drop. Its client's last
	/// `ALIVE_MS` begins, so the held requests look again: a standing request waits for the moment no device holds the
	/// link (`Link::lapse_at`).
	fn answered(&self, device: Device, client: &str, generation: u64) {
		lock(&self.link).answered(device, client, generation, now_ms());
		self.bump();
	}

	/// A Bluetooth central's subscription ended (slice 43): its clients hold the link no more, and the held requests
	/// look again at once, so the link passes to the next device (`Link::gone`).
	pub fn gone(&self, central: &str) {
		if lock(&self.link).gone(central) {
			self.bump();
		}
	}

	/// The statistics of the devices whose sessions ended.
	#[must_use]
	pub fn quiet(&self, now: i64) -> Vec<Summary> {
		lock(&self.link).quiet(now)
	}

	/// Every device's statistics so far.
	#[must_use]
	pub fn summaries(&self) -> Vec<Summary> {
		lock(&self.link).summaries()
	}

	/// The pumps settled since the last look some of whose commands reached no device (`Link::unreached`).
	#[must_use]
	pub fn unreached(&self, now: i64) -> Vec<Unreached> {
		lock(&self.link).unreached(now)
	}
}

/// A held sync over the Bluetooth link pending at the helper (since the review of 2026-10-17): its client holds the link
/// past `link::ALIVE_MS` while one is. The route puts it on its response, and the bridge (`blelink`) keeps it with the
/// answer until its last chunk went to the radio; its last copy's drop, there or wherever the call or the answer ends
/// (a call let go, a central gone), tells the link the request is answered (`Link::answered`).
#[derive(Clone)]
pub struct Held {
	_pending: Arc<Pending>,
}

struct Pending {
	links: Links,
	device: Device,
	client: String,
	generation: u64,
}

impl Held {
	fn new(links: &Links, req: &SyncRequest, generation: u64) -> Self {
		let pending = Pending { links: links.clone(), device: req.device, client: req.client.clone(), generation };
		Self { _pending: Arc::new(pending) }
	}
}

impl Drop for Pending {
	fn drop(&mut self) {
		self.links.answered(self.device, &self.client, self.generation);
	}
}

/// What a route asks the helper's loop.
pub enum Ask {
	/// A phone paired: the pairing block, the window.
	Pair { device: Device, name: String, done: oneshot::Sender<()> },
	/// A phone unpaired (since 2026-10-09): the pairing ends on the computer, and the Pro grant of its `client` goes.
	Unpair { device: Device, client: String, done: oneshot::Sender<()> },
	/// A sync arrived on a transport, the request as it came but for its `events`, which hold only the pump events not
	/// taken before (`Link::take_events`): the watch's pairing, its heart samples, a catch-up pump's rows, a device's app
	/// rows (each kept once, whatever the repeats), a match's offer taken, a device's latest game offer and a phone's
	/// Pro proof; the transport for the counts' `link_ble`.
	Synced { request: Box<SyncRequest>, transport: Transport, done: oneshot::Sender<SyncAnswer> },
	/// A phone's body, each field checked (`Body::validate`), into the body every app device shares (slice 35); the
	/// answer is that body after the merge.
	Body { body: Box<Body>, done: oneshot::Sender<Body> },
	/// The unfinished list and count, the held count, the lock and the characters with their sheets.
	Sheet { done: oneshot::Sender<Sheet> },
	/// A page of at most `size` rows the helper keeps, with the pumps' proofs from a unix second when asked.
	History { after: u64, proofs_since: Option<u64>, size: usize, done: oneshot::Sender<HistoryPage> },
	/// A page of at most `size` sheets the helper has read.
	Sheets { after: u64, size: usize, done: oneshot::Sender<SheetsPage> },
	/// A page of at most `size` app rows every phone sent, the helper's copy (slice 28).
	AppRows { after: u64, size: usize, done: oneshot::Sender<AppRowsPage> },
	/// The desktop app's sync account (slice 60): as it stands (`GET /v1/account`), or after it joined a phone's
	/// account and R on the player's Add (`PUT /v1/account`); none answers a join the server refused.
	Account { put: Option<(String, [u8; sync::ROOT])>, done: oneshot::Sender<Option<AccountState>> },
}

/// The desktop app's sync account as `/v1/account` answers it (slice 60), the desk's (`questpump_desk::sync`).
pub use questpump_desk::sync::AccountState;

/// What the loop answers a sync with, for every response of the exchange (plain ones too): what it took of this
/// request. The offer facts the response carries are the loop's as the response leaves (`OfferFacts`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SyncAnswer {
	/// The id of the store of app rows (`AppRows::id`), which confirms every row the request carried; None when one of
	/// them could not be written (`AppTaken::unwritten`), so the response confirms nothing and the phone sends them
	/// again.
	pub rows_store: Option<String>,
	/// The pump ids of the catch-ups among the request's new events whose moves the loop held for the game (while a
	/// World of Warcraft client with QuestPump ran as it took them, `Desk::wow_with_addon`), kept with each taken
	/// event (`Link::game_held`).
	pub game_held: Vec<String>,
}

/// The offer facts every sync response carries (slice 35), as the loop last told them (`Lan::set_offers`: at the end
/// of each of its turns and as it takes a sync) and read as the response leaves (slice 48's second fix round), so a
/// request held up to its wait answers with them as they are then, never as they were at its arrival.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OfferFacts {
	/// The felt-unwell lock's end while it lies ahead.
	pub locked_until: Option<i64>,
	/// The match offers taken back (a device or the computer started the pump, the card's Later or a move done since,
	/// on an offer that went out) whose `match_end` had not expired (slice 35).
	pub offer_withdrawn: Vec<String>,
	/// A game the player turned on in QuestPump on the computer is in a match (slice 35).
	pub playing: bool,
	/// An aim game with its arms rest on runs on the computer (slice 48's double-check, `Desk::arms_now`).
	pub arms: bool,
	/// The latest game offer on any device, unix seconds (slice 35).
	pub game_offer_at: Option<i64>,
}

impl OfferFacts {
	/// The part a waiting held or standing request answers at once on when it changes (slice 48's second fix round):
	/// the ids taken back, `playing` and `arms`. The lock's end and the latest game offer ride with the next response.
	fn early(&self) -> Early {
		(self.offer_withdrawn.clone(), self.playing, self.arms)
	}
}

/// The game as a sync response tells it: the helper block's `wow`, `sight` and `step`, `game_asks`, and since slice 43
/// the block's `ble`, the computer's Bluetooth link, which the phones' pairing window and follow card show too.
type GameShown = (Option<Wow>, Option<Sight>, Option<Step>, bool, Option<Ble>);

/// The offer facts a waiting request answers at once on (`OfferFacts::early`).
type Early = (Vec<String>, bool, bool);

/// What a held or standing request answers at once on while it differs from what its client last heard: the game as a
/// response tells it and the offer facts' early part.
type Told = (GameShown, Early);

/// What the responses tell: the helper block, `game_asks` beside it, and what each client's last sync response told
/// of the game.
struct Shown {
	helper: link::Helper,
	/// A World of Warcraft client with QuestPump runs (`Desk::wow_with_addon`, through `Lan::set_game`): every sync
	/// response's `game_asks` as it leaves (slice 35), while which the devices offer no catch-up and no Skip.
	asks: bool,
	/// The offer facts as the loop last told them (`Lan::set_offers`).
	offers: OfferFacts,
	/// The game and the offer facts' early part as each client's last sync response told them (the game since
	/// 2026-10-13, the offer facts since slice 48's second fix round), with the `next` that response carried, one entry
	/// per app install that synced in this run, as `Link` keeps its clients: a held request answers at once while
	/// either differs from it, so a change made between a response and that client's next request reaches the client
	/// too (`Lan::told`). It holds what was sent, not what arrived.
	told: HashMap<(Device, String), (Told, u64)>,
}

impl Shown {
	fn game(&self) -> GameShown {
		(self.helper.wow, self.helper.sight, self.helper.step, self.asks, self.helper.ble)
	}

	/// What a response tells now that a waiting request answers at once on.
	fn now_told(&self) -> Told {
		(self.game(), self.offers.early())
	}

	/// The helper block as a response carries it, stamped with the helper's clock.
	fn block(&self) -> link::Helper {
		let mut h = self.helper.clone();
		h.t = now_ms();
		h
	}
}

/// The routes' shared state.
pub struct Lan {
	/// The current pairing key's bearer and seal, read by every request: a new pairing code replaces them for the
	/// open connections too.
	keys: Mutex<Arc<Keys>>,
	/// When the routes started, on the helper's clock: a body sent before it was sent to an earlier run, whose replay
	/// guard is gone, and is refused.
	started: i64,
	replay: Mutex<Replay>,
	throttle: Mutex<Throttle<Peer>>,
	noted: Mutex<Noted>,
	links: Links,
	shown: Mutex<Shown>,
	asks: mpsc::UnboundedSender<Ask>,
	hold: Duration,
}

impl Lan {
	/// The routes for a pairing key; `hold` is `link::HOLD_MS` in the helper (a test holds shorter), the least a held
	/// request waits for a command, and the holder's `wait` up to `link::WAIT_MAX_MS` the most (`Link::hold_ms`).
	#[must_use]
	pub fn new(
		key: &[u8; 32],
		links: Links,
		helper: link::Helper,
		asks: mpsc::UnboundedSender<Ask>,
		hold: Duration,
	) -> Arc<Self> {
		Arc::new(Self {
			keys: Mutex::new(Arc::new(Keys::derive(key))),
			started: now_ms(),
			replay: Mutex::new(Replay::new(link::REPLAY_MS)),
			throttle: Mutex::new(Throttle::default()),
			noted: Mutex::new(Noted::default()),
			links,
			shown: Mutex::new(Shown { helper, asks: false, offers: OfferFacts::default(), told: HashMap::new() }),
			asks,
			hold,
		})
	}

	/// A new pairing key: from the next request on, on every connection already open as well, only a device that
	/// scanned the new code is let in, and a response only it can open.
	pub fn rekey(&self, key: &[u8; 32]) {
		*lock(&self.keys) = Arc::new(Keys::derive(key));
	}

	/// The current pairing key's bearer, seal, key id and Bluetooth envelopes.
	#[must_use]
	pub fn keys(&self) -> Arc<Keys> {
		lock(&self.keys).clone()
	}

	/// The name, the `.local` name and the address every response and the hello carry from now on (the computer was
	/// renamed, the helper moved to another network); without an address the last one stays.
	pub fn set_address(&self, name: &str, host: Option<IpAddr>, mdns: Option<&str>) {
		let mut shown = lock(&self.shown);
		let h = &mut shown.helper;
		name.clone_into(&mut h.name);
		h.mdns = mdns.map(str::to_owned);
		if let Some(host) = host {
			h.host = host.to_string();
		}
	}

	/// What the computer knows of World of Warcraft, what the helper reads of it and the pairing's step (since
	/// 2026-10-09, the step since 2026-10-13), in every response's helper block from now on, the sight and the step
	/// only with QuestPump in the game, whether a client with QuestPump runs, in every sync response's `game_asks`, and
	/// how the computer's Bluetooth link stands (`ble`, since slice 43; none before it is known). A change answers the
	/// held requests at once (`held`), so a phone's open pairing window shows it and an idle phone hears a game start or
	/// quit; whether it changed.
	#[must_use]
	pub fn set_game(&self, wow: Wow, sight: Sight, step: Step, asks: bool, ble: Option<Ble>) -> bool {
		let addon = wow == Wow::Addon;
		let game = (Some(wow), addon.then_some(sight), addon.then_some(step), asks, ble);
		let changed = {
			let mut s = lock(&self.shown);
			let changed = s.game() != game;
			(s.helper.wow, s.helper.sight, s.helper.step, s.asks, s.helper.ble) = game;
			changed
		};
		if changed {
			self.links.bump();
		}
		changed
	}

	/// The offer facts every sync response carries from now on (`OfferFacts`); a change of their early part (the ids
	/// taken back, `playing`, `arms`) answers the waiting held and standing requests at once (`held`, `stand`).
	pub fn set_offers(&self, offers: OfferFacts) {
		let early = {
			let mut s = lock(&self.shown);
			let early = s.offers.early() != offers.early();
			s.offers = offers;
			early
		};
		if early {
			self.links.bump();
		}
	}

	/// What every sync response tells now that a waiting request answers at once on: the game and the offer facts'
	/// early part.
	fn now_told(&self) -> Told {
		lock(&self.shown).now_told()
	}

	/// The game and the offer facts' early part as this client last heard them: what its last sync response told, while
	/// this request continues from that response (its `since` is the `next` the response carried). None, which `held`
	/// answers at once, for a client not answered in this run (the helper started again) and for a request that does
	/// not continue from its last response: the app or its link started again with a `since` of 0, or that response,
	/// whose `next` was new, never reached it. Best-effort, since the record is what was sent: a response lost with the
	/// `next` the device held already (no command in it, as when a kick cut it on its way or the connection dropped)
	/// counts as heard, so a change of the game it carried reaches the device at the end of its next request's hold,
	/// and a start of a client that never held the link in this run (its `since` of 0 given back) reads as continuing.
	fn told(&self, req: &SyncRequest) -> Option<Told> {
		let s = lock(&self.shown);
		s.told
			.get(&(req.device, req.client.clone()))
			.filter(|(_, next)| *next == req.since)
			.map(|(told, _)| told.clone())
	}

	/// The helper block, `game_asks` and the offer facts of a sync response to this client, as they stand when it
	/// leaves, kept with the response's `next` as what it was told last (`told`).
	fn tell(&self, req: &SyncRequest, next: u64) -> (link::Helper, bool, OfferFacts) {
		let mut s = lock(&self.shown);
		let told = s.now_told();
		s.told.insert((req.device, req.client.clone()), (told, next));
		(s.block(), s.asks, s.offers.clone())
	}

	/// Logs a line about a peer unless one of the same topic went out within the last minute: a phone that retries an
	/// old code, a device's sweep or a stranger's scan (or writes over Bluetooth) writes one line a minute, never one a
	/// request.
	fn note(&self, peer: &Peer, topic: Topic, line: impl FnOnce() -> String) {
		if !lock(&self.noted).first(peer, topic, now_ms()) {
			return;
		}
		match topic {
			Topic::Refused | Topic::Request => log::warn!("{}", line()),
			Topic::Pair | Topic::Hello | Topic::Central => log::info!("{}", line()),
		}
	}

	/// Whether a peer is refused for its failures (`429` on the LAN; a central's writes ignored), which is noted: an
	/// address's wrong bearers, a central's dropped messages and envelopes that do not open, `link::THROTTLE_AFTER`
	/// within a minute.
	#[must_use]
	pub fn throttled(&self, peer: &Peer) -> bool {
		let refused = lock(&self.throttle).refused(peer, now_ms());
		if refused {
			self.note(peer, topic(peer), || match peer {
				// copy-lint: not shown (a log line)
				Peer::Lan(_) => format!("lan: refused {peer}: ten wrong bearers within a minute (429)"),
				// copy-lint: not shown (a log line)
				Peer::Ble(_) => format!("lan: refused {peer}: ten failed messages within a minute, its writes ignored"),
			});
		}
		refused
	}

	/// A central's message dropped or envelope that did not open (slice 43, `blelink`): counted toward its throttle and
	/// noted, with its subscriptions, once a minute (`note_central`).
	pub fn failed(&self, peer: &Peer, why: &str) {
		lock(&self.throttle).failed(peer.clone(), now_ms());
		self.note(peer, topic(peer), || format!("lan: refused {peer}: {why}"));
	}

	/// A line about a Bluetooth central (its subscription came or went), in the one line a minute each central gets
	/// with its refusals: a stranger that subscribes and unsubscribes in a loop writes no more.
	pub fn note_central(&self, peer: &Peer, line: impl FnOnce() -> String) {
		self.note(peer, topic(peer), line);
	}

	/// A new pairing code (since the review of 2026-10-17): every Bluetooth central's failures are forgiven, as their
	/// calls and challenges end.
	pub fn forgive_centrals(&self) {
		lock(&self.throttle).retain(|peer| !matches!(peer, Peer::Ble(_)));
	}

	fn helper(&self) -> link::Helper {
		lock(&self.shown).block()
	}

	/// Asks the loop; none when it has stopped.
	async fn ask<T>(&self, ask: impl FnOnce(oneshot::Sender<T>) -> Ask) -> Option<T> {
		let (tx, rx) = oneshot::channel();
		self.asks.send(ask(tx)).ok()?;
		rx.await.ok()
	}
}

/// Who sent a request: an address on the LAN, or a Bluetooth central (slice 43), whose id the bridge puts on each
/// request it hands the routes. The guard's throttle, the refusal log and the sync's transport read it.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Peer {
	Lan(IpAddr),
	/// The Mac's `CBCentral` identifier, Windows' device id.
	Ble(String),
}

impl Peer {
	/// How the request came, for the link (`Link::arrive`).
	#[must_use]
	pub fn via(&self) -> Via {
		match self {
			Self::Lan(_) => Via::Lan,
			Self::Ble(central) => Via::Ble(central.clone()),
		}
	}
}

impl fmt::Display for Peer {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Lan(ip) => write!(f, "{ip}"),
			Self::Ble(central) => write!(f, "ble {}", ble::central_tag(central)),
		}
	}
}

/// The request's peer: the bridge's `Peer` on a request it hands the routes, else the LAN connection's address
/// (`ConnectInfo`, IPv4-mapped ones canonicalised). A request with neither is a router served some other way: 500.
impl<S: Send + Sync> FromRequestParts<S> for Peer {
	type Rejection = StatusCode;

	fn from_request_parts(parts: &mut Parts, _: &S) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
		let peer = parts.extensions.get::<Self>().cloned().or_else(|| {
			let ConnectInfo(conn) = parts.extensions.get::<ConnectInfo<Conn>>()?;
			Some(Self::Lan(conn.peer.ip().to_canonical()))
		});
		std::future::ready(peer.ok_or(StatusCode::INTERNAL_SERVER_ERROR))
	}
}

/// An answer's plain JSON, which the guard seals for its request's route on the way out (`Way::Answer`), so no
/// route seals an answer of its own and none goes out for another route.
#[derive(Clone)]
struct Unsealed(Bytes);

/// A route's answer: 200 and `body`'s JSON for the guard to seal.
fn reply<T: serde::Serialize>(body: &T) -> Response {
	match serde_json::to_vec(body) {
		Ok(json) => {
			let mut response = StatusCode::OK.into_response();
			response.extensions_mut().insert(Unsealed(json.into()));
			response
		}
		Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
	}
}

/// A request body that says when it was sent, on the helper's clock.
trait Stamped {
	/// Whether a body of this route that opened but does not read is refused sealed for its request
	/// (`RefusedResponse`): the sync's alone, the one 422 a device drops what it sent on. Every other route answers it
	/// bare, which no device acts on.
	const SEALS_REFUSAL: bool = false;

	fn sent_at(&self) -> i64;
}

impl Stamped for PairRequest {
	fn sent_at(&self) -> i64 {
		self.sent_at
	}
}

impl Stamped for SyncRequest {
	const SEALS_REFUSAL: bool = true;

	fn sent_at(&self) -> i64 {
		self.sent_at
	}
}

impl Stamped for UnpairRequest {
	fn sent_at(&self) -> i64 {
		self.sent_at
	}
}

impl Stamped for BodyRequest {
	fn sent_at(&self) -> i64 {
		self.sent_at
	}
}

impl Stamped for AccountRequest {
	fn sent_at(&self) -> i64 {
		self.sent_at
	}
}

/// A body opened for its route, read and found fresh: the extractor every sealed route takes last. A body sealed for
/// another route or way does not open; one that opens but holds a field the route's request does not know does not
/// read (all but the sync's refuse one: `link::PairRequest`); one sent more than `FRESH_MS` from the helper's clock,
/// before these routes started, or again within the window, is refused: a recorded request replayed on the LAN, after
/// a restart as well.
struct Sealed<T>(T);

impl<T: DeserializeOwned + Stamped> FromRequest<Arc<Lan>> for Sealed<T> {
	type Rejection = Refusal;

	async fn from_request(req: Request, lan: &Arc<Lan>) -> Result<Self, Self::Rejection> {
		let (method, path) = (req.method().clone(), req.uri().path().to_owned());
		let body = tokio::time::timeout(BODY_TIME, Bytes::from_request(req, lan))
			.await
			.map_err(|_| Refusal(StatusCode::REQUEST_TIMEOUT, Why::Slow, None))?
			.map_err(|rejection| Refusal(rejection.status(), Why::Cut, None))?; // 413 past the body limit
		let route = Route { way: Way::Request, method: method.as_str(), path: &path };
		let (plain, nonce) =
			lan.keys().seal.open(&route, &body).map_err(|_| Refusal(StatusCode::BAD_REQUEST, Why::Seal, None))?;
		let body: T = serde_json::from_slice(&plain).map_err(|_| {
			// the sync's refusal names this request, and the guard seals it for the sync's answer, so no other answer at
			// the address, nor another route's, makes a device drop what it sent (one string always serialises); every
			// other route's is bare
			let refused = T::SEALS_REFUSAL.then(|| serde_json::to_vec(&RefusedResponse::of(nonce)).unwrap_or_default());
			Refusal(StatusCode::UNPROCESSABLE_ENTITY, Why::Unread, refused)
		})?;
		let now = now_ms();
		// the replay guard keeps a nonce only of a body sent in time
		let why = if body.sent_at().abs_diff(now) > link::FRESH_MS.unsigned_abs() {
			Some(Why::Off(body.sent_at().saturating_sub(now)))
		} else if body.sent_at() < lan.started {
			Some(Why::Early)
		} else if !lock(&lan.replay).fresh(nonce, now) {
			Some(Why::Replayed)
		} else {
			None
		};
		match why {
			Some(why) => Err(Refusal(StatusCode::BAD_REQUEST, why, None)),
			None => Ok(Sealed(body)),
		}
	}
}

/// Why a sealed body was refused, carried on the refusal's response for the guard's note (since 2026-10-14).
#[derive(Clone, Copy)]
enum Why {
	/// It took longer than `BODY_TIME` to arrive.
	Slow,
	/// It did not arrive whole, or it is over `BODY_BYTES`.
	Cut,
	/// Its seal does not open under the current key and its route.
	Seal,
	/// It opened but does not read as the route's body.
	Unread,
	/// Its `sent_at` lies this many milliseconds from the helper's clock, more than `FRESH_MS`: the device's estimate
	/// of the helper's clock is off.
	Off(i64),
	/// Its `sent_at` lies before these routes started.
	Early,
	/// Its nonce came within the replay window.
	Replayed,
}

impl Why {
	/// The words of the guard's log line.
	fn words(self) -> String {
		match self {
			Self::Slow => format!("a body slower than {} s", BODY_TIME.as_secs()), // copy-lint: not shown (a log line)
			Self::Cut => "a body too large or cut off".to_owned(),                 // copy-lint: not shown (a log line)
			Self::Seal => "a seal that does not open".to_owned(),                  // copy-lint: not shown (a log line)
			Self::Unread => "a body that does not read".to_owned(),                // copy-lint: not shown (a log line)
			// copy-lint: not shown (a log line)
			Self::Off(ms) => format!("sent {:+} s off the helper's clock", ms / 1_000),
			Self::Early => "sent before this run of the helper started".to_owned(), // copy-lint: not shown (a log line)
			Self::Replayed => "a nonce seen before (a replay)".to_owned(),          // copy-lint: not shown (a log line)
		}
	}
}

/// A sealed body refused: its status, with why on the response for the guard's note, and its body: for a sync body
/// that opened but does not read, that request's refusal (`RefusedResponse`, since the security review of
/// 2026-10-31; `Stamped::SEALS_REFUSAL`), which the guard seals as the route's answer, the one answer a device drops
/// what the request carried on; every other refusal has none.
struct Refusal(StatusCode, Why, Option<Vec<u8>>);

impl IntoResponse for Refusal {
	fn into_response(self) -> Response {
		let Self(status, why, refused) = self;
		let mut response = status.into_response();
		if let Some(plain) = refused {
			response.extensions_mut().insert(Unsealed(plain.into()));
		}
		response.extensions_mut().insert(why);
		response
	}
}

/// What the log notes about a peer, each at most once a minute.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Topic {
	/// A connection dropped at accept, a wrong bearer, a throttled peer.
	Refused,
	/// A Bluetooth central's subscription, its dropped messages, envelopes that did not open and its throttle (slice
	/// 43): one line a minute for each central.
	Central,
	/// A request with this helper's bearer refused all the same (since 2026-10-14): its body (the seal, its time, a
	/// replay, its size), its query, or the loop gone.
	Request,
	/// A request on the pairing route, whatever becomes of it.
	Pair,
	Hello,
}

/// The topic of a peer's refusals: a central's share one line a minute with its subscriptions.
fn topic(peer: &Peer) -> Topic {
	match peer {
		Peer::Lan(_) => Topic::Refused,
		Peer::Ble(_) => Topic::Central,
	}
}

/// Whether a request's path (with its query) is a paged route's, whose answer over the Bluetooth link may run to many
/// chunks: it goes after every other answer to its central, so a held sync's countdown never waits behind a page.
#[must_use]
pub fn is_page(path: &str) -> bool {
	let path = path.split_once('?').map_or(path, |(path, _)| path);
	[HISTORY, SHEETS, APP_ROWS].contains(&path)
}

/// When each peer's last line of each topic went out, kept for a minute.
#[derive(Default)]
struct Noted {
	at: HashMap<(Peer, Topic), i64>,
}

impl Noted {
	/// Whether a line about `peer` on `topic` may go out at `now`: none did within the last minute. It is then
	/// counted from `now`.
	fn first(&mut self, peer: &Peer, topic: Topic, now: i64) -> bool {
		self.at.retain(|_, &mut at| now - at < NOTE_EVERY_MS);
		let key = (peer.clone(), topic);
		if self.at.contains_key(&key) || self.at.len() >= NOTED_PEERS {
			return false;
		}
		self.at.insert(key, now);
		true
	}
}

/// Every sealed route: a peer not refused for its failures, with this helper's bearer (a peer outside the home network
/// never got a connection, `admit`; the Bluetooth bridge's request carries the bearer the bridge added for an envelope
/// that opened, and its central as its `Peer`), its answer sealed for its route on the way out (`Unsealed`; a refusal
/// without one goes bare). A pairing request is noted before its bearer is checked, so an attempt the bearer or the
/// seal then refuses shows in the log too, and a request the route then refuses (a body that does not open, is stale,
/// replayed, slow, too large or does not read, a query that does not read, the loop gone) is noted with its status and
/// why (since 2026-10-14).
async fn guard(State(lan): State<Arc<Lan>>, peer: Peer, req: Request, next: Next) -> Response {
	if lan.throttled(&peer) {
		return StatusCode::TOO_MANY_REQUESTS.into_response();
	}
	let (method, path): (Method, String) = (req.method().clone(), req.uri().path().to_owned());
	if path == PAIR {
		lan.note(&peer, Topic::Pair, || format!("lan: a pairing request from {peer}"));
	}
	let bearer = req.headers().get(header::AUTHORIZATION).and_then(|v| v.to_str().ok());
	if !lan.keys().authorizes(bearer) {
		lock(&lan.throttle).failed(peer.clone(), now_ms());
		lan.note(&peer, Topic::Refused, || format!("lan: refused {peer} on {path}: not this helper's bearer (401)"));
		return StatusCode::UNAUTHORIZED.into_response();
	}
	let mut response = next.run(req).await;
	if let Some(Unsealed(plain)) = response.extensions_mut().remove::<Unsealed>() {
		let route = Route { way: Way::Answer, method: method.as_str(), path: &path };
		*response.body_mut() = axum::body::Body::from(lan.keys().seal.seal(&route, &plain));
		response.headers_mut().insert(header::CONTENT_TYPE, HeaderValue::from_static("application/octet-stream"));
	}
	let status = response.status();
	if status.is_client_error() || status.is_server_error() {
		lan.note(&peer, Topic::Request, || {
			let why = response
				.extensions()
				.get::<Why>()
				.map_or_else(|| status.canonical_reason().unwrap_or("refused").to_lowercase(), |why| why.words());
			format!("lan: refused {peer} on {path}: {why} ({})", status.as_u16())
		});
	}
	response
}

#[derive(Deserialize)]
struct HelloQuery {
	c: String,
}

/// Who answers on this address, with no bearer and no seal (since the release security review of 2026-10-31, L3): over
/// the device's challenge `c` and 16 random bytes of its own, the proofs of the install's id and of the current key
/// alone (`HelloResponse::of`), so a device looking for its helper checks a candidate before its bearer goes out and
/// nobody else learns who answered: no id, name or port, and no two answers alike. A `c` that is not 16 bytes of
/// base64url gets 422. Under the same throttle as every route; noted once a minute per peer.
async fn hello(State(lan): State<Arc<Lan>>, peer: Peer, query: Result<Query<HelloQuery>, QueryRejection>) -> Response {
	if lan.throttled(&peer) {
		return StatusCode::TOO_MANY_REQUESTS.into_response();
	}
	lan.note(&peer, Topic::Hello, || format!("lan: hello from {peer}"));
	let challenge = query.ok().and_then(|Query(q)| URL_SAFE_NO_PAD.decode(q.c).ok());
	let Some(c) = challenge.and_then(|c| <[u8; link::HELLO_BYTES]>::try_from(c).ok()) else {
		return StatusCode::UNPROCESSABLE_ENTITY.into_response();
	};
	let mut n = [0; link::HELLO_BYTES];
	rand::fill(&mut n);
	let hello = lan.helper().hid.and_then(|hid| HelloResponse::of(&hid, &lan.keys().hello, &c, &n));
	match hello.and_then(|h| serde_json::to_vec(&h).ok()) {
		Some(json) => ([(header::CONTENT_TYPE, "application/json")], json).into_response(),
		// a helper block without an install id has none to show: a looking device skips this address
		None => StatusCode::NOT_FOUND.into_response(),
	}
}

async fn pair(State(lan): State<Arc<Lan>>, Sealed(req): Sealed<PairRequest>) -> Response {
	let name: String = req.name.trim().chars().take(NAME_CHARS).collect(); // a phone's name, for the window
	if req.device.is_watch() || name.is_empty() {
		return StatusCode::UNPROCESSABLE_ENTITY.into_response(); // a watch pairs through its phone
	}
	match lan.ask(|done| Ask::Pair { device: req.device, name, done }).await {
		Some(()) => reply(&PairResponse { helper: lan.helper() }),
		None => StatusCode::SERVICE_UNAVAILABLE.into_response(),
	}
}

/// A phone's Unpair (since 2026-10-09): the loop ends the pairing on the computer; a watch's pairing is its phone's.
async fn unpair(State(lan): State<Arc<Lan>>, Sealed(req): Sealed<UnpairRequest>) -> Response {
	if req.device.is_watch() || req.client.chars().count() > CLIENT_CHARS {
		return StatusCode::UNPROCESSABLE_ENTITY.into_response();
	}
	match lan.ask(|done| Ask::Unpair { device: req.device, client: req.client, done }).await {
		Some(()) => reply(&UnpairResponse { helper: lan.helper() }),
		None => StatusCode::SERVICE_UNAVAILABLE.into_response(),
	}
}

#[derive(Deserialize)]
struct PingQuery {
	t0: i64,
}

async fn ping(State(lan): State<Arc<Lan>>, query: Result<Query<PingQuery>, QueryRejection>) -> Response {
	let t1 = now_ms();
	let Ok(Query(PingQuery { t0 })) = query else {
		return StatusCode::UNPROCESSABLE_ENTITY.into_response();
	};
	reply(&PingResponse { helper: lan.helper(), t0, t1, t2: now_ms() })
}

/// The held exchange: the request's receipts and battery go into the statistics (the first request of a device's
/// session logged with its `User-Agent`: `session_starts`), the rest of it to the loop (its events only the ones not
/// taken before), and every response names the store that keeps its app rows (`rows_store`) unless one could not be
/// written, with the offer facts after a match as they stand when it leaves (`OfferFacts`: the lock's end, the ids
/// taken back, `playing`, `arms` and the latest game offer), whether the game asks the owed moves as the response
/// leaves (`game_asks`) and which of the request's catch-ups it held for the game (`game_held`, an event sent again
/// answered as it was taken); a plain post answers at once; a held one answers the moment a command waits for its
/// device, when a newer request from the same device arrives, while the game a sync response tells or the offer facts'
/// early part (an offer taken back, `playing`, `arms`: slice 48's second fix round) differs from what this client last
/// heard or nothing says what it heard (`Lan::told`), or after the hold from its arrival (`HOLD_MS`, or the holder's
/// `wait` up to its transport's most: `Link::hold_ms`). Over the Bluetooth link (slice 43) its `wait` may run to
/// `BLE_WAIT_MAX_MS`, and a held one keeps its client the link's holder while it is pending at the helper: its response
/// carries a `Held` the bridge drops once the answer went to the radio. A standing request (since the review of
/// 2026-10-17, `Exchange::Standing`) waits as a held one does but holds nothing, carries no command and answers at once
/// while no device holds the link, which its answer says (`unheld`: `stand`), and since slice 51 a phone's carries the
/// game pump's commands queued for the holder as `mirror`, answering at once when one it has not shown waits.
async fn sync(
	State(lan): State<Arc<Lan>>,
	peer: Peer,
	headers: HeaderMap,
	Sealed(req): Sealed<SyncRequest>,
) -> Response {
	let t1 = now_ms();
	let arrived = tokio::time::Instant::now();
	// the game and the offer facts' early part as this client last heard them: a change since, made before this request
	// arrived or during the loop's turn for it, answers it at once, as does none (the app or the helper started again)
	let told = lan.told(&req);
	let floor = i64::try_from(lan.hold.as_millis()).unwrap_or(link::HOLD_MS);
	let via = peer.via();
	let exchange = req.exchange();
	let (starts, generation, events, hold) = {
		let mut link = lock(&lan.links.link);
		let starts = link.starts(&req);
		let generation = link.arrive(&req, &via, t1);
		(starts, generation, link.take_events(&req), link.hold_ms(&req, via.transport(), floor, t1))
	};
	// a held request over Bluetooth stays pending, its client the link's holder, until its answer went to the radio
	let pending =
		(exchange == Exchange::Held && matches!(via, Via::Ble(_))).then(|| Held::new(&lan.links, &req, generation));
	if starts {
		session_starts(&req, &peer, &headers);
	}
	if exchange != Exchange::Plain {
		lan.links.bump(); // an older held or standing request of this client answers now
	}
	// the loop must take the events and the app rows before the response confirms them: without it, no answer but 503;
	// a game offer dated more than a day past the helper's clock (a device clock set ahead) is left out
	let game_offer_at = req.game_offer_at.filter(|&at| link::not_ahead(at.saturating_mul(1_000), t1));
	let request = Box::new(SyncRequest { events, game_offer_at, ..req.clone() });
	let transport = via.transport();
	let Some(answer) = lan.ask(|done| Ask::Synced { request, transport, done }).await else {
		return StatusCode::SERVICE_UNAVAILABLE.into_response();
	};
	// the catch-ups held for the game, those just taken and those taken before that came again
	let game_held = lock(&lan.links.link).game_held(&req, &answer.game_held);
	// the hold counts from the arrival, so the loop's turn for this request is inside it
	let deadline = arrived + Duration::from_millis(hold.unsigned_abs());
	let (commands, mirror, next, unheld) = match exchange {
		Exchange::Held => {
			let (commands, next) = held(&lan, &req, generation, told, deadline).await;
			(commands, Vec::new(), next, false)
		}
		Exchange::Standing => {
			let unheld = stand(&lan, &req, generation, told, deadline).await;
			// the game pump's commands for the holder since this phone's last standing answer (slice 51)
			let (mirror, next) = lock(&lan.links.link).mirror(&req, now_ms());
			(Vec::new(), mirror, next, unheld)
		}
		Exchange::Plain => (Vec::new(), Vec::new(), lock(&lan.links.link).since(req.since), false),
	};
	// the offer facts as the response leaves, after the hold
	let (helper, game_asks, offers) = lan.tell(&req, next);
	let mut response = reply(&SyncResponse {
		helper,
		t0: req.t0,
		t1,
		t2: now_ms(),
		next,
		locked_until: offers.locked_until,
		rows_store: answer.rows_store,
		offer_withdrawn: offers.offer_withdrawn,
		playing: offers.playing,
		arms: offers.arms,
		game_offer_at: offers.game_offer_at,
		game_asks,
		game_held,
		unheld,
		mirror,
		commands,
	});
	if let Some(held) = pending {
		response.extensions_mut().insert(held);
	}
	response
}

/// A device's link session starts (since 2026-10-14: `Link::starts`): its kind, the start of its install's id, whether
/// it holds a request or posts, the transport (since slice 43: `lan`, or `ble` with its central's tag) and what its app
/// says it is (`QuestPump/<version> (<build>; <platform>)`; one that is not visible ASCII is not shown), so a device
/// test reads from the log which app came, how and when.
fn session_starts(req: &SyncRequest, peer: &Peer, headers: &HeaderMap) {
	let agent: String = match headers.get(header::USER_AGENT).map(HeaderValue::to_str) {
		None => "no agent".to_owned(),                    // copy-lint: not shown (a log line)
		Some(Err(_)) => "an unreadable agent".to_owned(), // copy-lint: not shown (a log line)
		Some(Ok(agent)) => agent.chars().take(AGENT_CHARS).collect(),
	};
	let via = match peer {
		Peer::Lan(_) => "lan".to_owned(), // copy-lint: not shown (a log line)
		Peer::Ble(_) => peer.to_string(),
	};
	let exchange = match req.exchange() {
		Exchange::Held => "held",
		Exchange::Standing => "standing",
		Exchange::Plain => "plain",
	};
	log::info!(
		"lan: {} links (client {}, {exchange}, {via}, {agent})",
		req.device.name(),
		link::client_tag(&req.client)
	);
}

async fn held(
	lan: &Lan,
	req: &SyncRequest,
	generation: u64,
	told: Option<Told>,
	deadline: tokio::time::Instant,
) -> (Vec<Queued>, u64) {
	let mut wake = lan.links.wake.subscribe();
	let mut last = false;
	loop {
		wake.borrow_and_update(); // seen before the look, so a command queued or a game changed after it wakes the wait
		// the game or the offer facts' early part changed, or nothing says what this client heard: answer at once
		let changed = told.as_ref() != Some(&lan.now_told());
		{
			let mut link = lock(&lan.links.link);
			if !link.is_current(req, generation) {
				return (Vec::new(), link.since(req.since)); // a newer request from this client took over
			}
			let (commands, next) = link.respond(req, now_ms());
			if !commands.is_empty() || changed || last {
				return (commands, next);
			}
		}
		last = !matches!(tokio::time::timeout_at(deadline, wake.changed()).await, Ok(Ok(())));
	}
}

/// A standing request's wait (since the review of 2026-10-17): it answers at once while no device holds the link, when
/// a newer held or standing request from its client arrives, while the game or the offer facts' early part differs from
/// what this client last heard (`Lan::told`), when a game pump's command it has not shown waits for the holder (slice
/// 51, `Link::mirror`, which its answer carries as `mirror`), or at its hold's end, with no command whoever holds the
/// link. The holder going quiet with nothing arriving wakes no request, so it also looks again the moment the next
/// client stops counting alive (`Link::lapse_at`). Whether no device holds the link as it answers
/// (`SyncResponse::unheld`).
async fn stand(
	lan: &Lan,
	req: &SyncRequest,
	generation: u64,
	told: Option<Told>,
	deadline: tokio::time::Instant,
) -> bool {
	let mut wake = lan.links.wake.subscribe();
	let mut last = false;
	loop {
		wake.borrow_and_update(); // seen before the look, as in `held`
		let changed = told.as_ref() != Some(&lan.now_told());
		let lapse = {
			let mut link = lock(&lan.links.link);
			let now = now_ms();
			let unheld = link.unheld(now);
			// a game pump's command queued for the holder that this phone has not shown yet answers it at once (slice
			// 51), so it shows the pump as a mirror
			let mirrored = !link.mirror(req, now).0.is_empty();
			if unheld || changed || last || mirrored || !link.is_current(req, generation) {
				return unheld;
			}
			link.lapse_at(now).map(|at| tokio::time::Instant::now() + Duration::from_millis((at - now).unsigned_abs()))
		};
		let look = lapse.map_or(deadline, |at| at.min(deadline));
		last = match tokio::time::timeout_at(look, wake.changed()).await {
			Ok(Ok(())) => false,
			Ok(Err(_)) => true,
			Err(_) => tokio::time::Instant::now() >= deadline,
		};
	}
}

async fn sheet(State(lan): State<Arc<Lan>>) -> Response {
	match lan.ask(|done| Ask::Sheet { done }).await {
		Some(sheet) => reply(&SheetResponse { helper: lan.helper(), sheet }),
		None => StatusCode::SERVICE_UNAVAILABLE.into_response(),
	}
}

/// A place, a second or a limit that does not read (a negative number, text) is 422, as every query.
#[derive(Deserialize)]
struct HistoryQuery {
	after: Option<u64>,
	proofs_since: Option<u64>,
	/// The most rows the page holds (since slice 43: 500 over the Bluetooth link), `link::page_size`.
	limit: Option<u32>,
}

async fn history(State(lan): State<Arc<Lan>>, query: Result<Query<HistoryQuery>, QueryRejection>) -> Response {
	let Ok(Query(HistoryQuery { after, proofs_since, limit })) = query else {
		return StatusCode::UNPROCESSABLE_ENTITY.into_response();
	};
	let size = link::page_size(limit, link::HISTORY_PAGE);
	match lan.ask(|done| Ask::History { after: after.unwrap_or(0), proofs_since, size, done }).await {
		Some(page) => reply(&HistoryResponse { helper: lan.helper(), page }),
		None => StatusCode::SERVICE_UNAVAILABLE.into_response(),
	}
}

/// A page's place, the items of the store the device holds already, and its most (since slice 43, `link::page_size`).
#[derive(Deserialize)]
struct PageQuery {
	after: Option<u64>,
	limit: Option<u32>,
}

async fn sheets(State(lan): State<Arc<Lan>>, query: Result<Query<PageQuery>, QueryRejection>) -> Response {
	let Ok(Query(PageQuery { after, limit })) = query else {
		return StatusCode::UNPROCESSABLE_ENTITY.into_response();
	};
	let size = link::page_size(limit, link::SHEETS_PAGE);
	match lan.ask(|done| Ask::Sheets { after: after.unwrap_or(0), size, done }).await {
		Some(page) => reply(&SheetsResponse::new(lan.helper(), page)),
		None => StatusCode::SERVICE_UNAVAILABLE.into_response(),
	}
}

/// The helper's copy of every phone's app rows (slice 28), paged as the history is.
async fn app_rows(State(lan): State<Arc<Lan>>, query: Result<Query<PageQuery>, QueryRejection>) -> Response {
	let Ok(Query(PageQuery { after, limit })) = query else {
		return StatusCode::UNPROCESSABLE_ENTITY.into_response();
	};
	let size = link::page_size(limit, link::HISTORY_PAGE);
	match lan.ask(|done| Ask::AppRows { after: after.unwrap_or(0), size, done }).await {
		Some(page) => reply(&AppRowsResponse { helper: lan.helper(), page }),
		None => StatusCode::SERVICE_UNAVAILABLE.into_response(),
	}
}

/// The body every app device shares (slice 35): a phone's fields, each dropped alone when it does not check out (one
/// dated more than a day past the helper's clock among them), merged by the loop field by field (the later change wins,
/// a tie keeps the helper's), and the merged body back. A watch's body is its phone's: it never calls the route.
async fn body(State(lan): State<Arc<Lan>>, Sealed(req): Sealed<BodyRequest>) -> Response {
	if req.device.is_watch() {
		return StatusCode::UNPROCESSABLE_ENTITY.into_response();
	}
	let mut theirs = req.body;
	theirs.validate(now_ms());
	match lan.ask(|done| Ask::Body { body: Box::new(theirs), done }).await {
		Some(body) => reply(&BodyResponse { helper: lan.helper(), body }),
		None => StatusCode::SERVICE_UNAVAILABLE.into_response(),
	}
}

/// The desktop app's sync account (slice 60), for a paired phone's account question after pairing.
async fn account(State(lan): State<Arc<Lan>>) -> Response {
	match lan.ask(|done| Ask::Account { put: None, done }).await.flatten() {
		Some(state) => reply(&account_response(&lan, state)),
		None => StatusCode::SERVICE_UNAVAILABLE.into_response(),
	}
}

/// A phone's sync account on the player's Add (slice 60): the desktop app joins it (its Sync on, what it made sent
/// there, its old account left) and answers its account after the join; 409 when the server refused the join. An id or
/// an R that does not read as 16 bytes is 422, and so is a watch's.
async fn account_put(State(lan): State<Arc<Lan>>, Sealed(req): Sealed<AccountRequest>) -> Response {
	let r = sync::bytes::<{ sync::ROOT }>(&req.r);
	if req.device.is_watch() || sync::bytes::<16>(&req.account).is_none() || r.is_none() {
		return StatusCode::UNPROCESSABLE_ENTITY.into_response();
	}
	let put = r.map(|r| (req.account, r));
	match lan.ask(|done| Ask::Account { put, done }).await {
		Some(Some(state)) => reply(&account_response(&lan, state)),
		Some(None) => StatusCode::CONFLICT.into_response(),
		None => StatusCode::SERVICE_UNAVAILABLE.into_response(),
	}
}

fn account_response(lan: &Lan, state: AccountState) -> AccountResponse {
	let AccountState { account, r, devices, email } = state;
	AccountResponse { helper: lan.helper(), account, r, devices, email }
}

/// A LAN connection: the peer's address and the computer's it reached (`serve`), which every request on it carries
/// as its `ConnectInfo`.
#[derive(Debug, Clone, Copy)]
struct Conn {
	peer: SocketAddr,
	local: SocketAddr,
}

impl Connected<Self> for Conn {
	fn connect_info(conn: Self) -> Self {
		conn
	}
}

/// Whether a request's `Host` (or its target's authority) names the computer at `port`, the one its connection
/// reached: an IPv4 literal, a bracketed IPv6 one (a zone allowed), `localhost` or the helper's own `.local` name
/// (`mdns`, in any case, a trailing dot allowed), each with the port. A web page's own name never does, whatever
/// address an attacker points it at.
fn names_this_computer(host: &str, port: u16, mdns: Option<&str>) -> bool {
	let Some((name, at)) = host.rsplit_once(':') else { return false };
	if at.parse::<u16>().ok() != Some(port) {
		return false;
	}
	if let Some(v6) = name.strip_prefix('[').and_then(|n| n.strip_suffix(']')) {
		return v6.split_once('%').map_or(v6, |(addr, _)| addr).parse::<Ipv6Addr>().is_ok();
	}
	let name = name.strip_suffix('.').unwrap_or(name);
	let own = |m: &str| name.eq_ignore_ascii_case(m.strip_suffix('.').unwrap_or(m));
	name.parse::<Ipv4Addr>().is_ok() || name.eq_ignore_ascii_case("localhost") || mdns.is_some_and(own)
}

/// Every request on the LAN's listeners (since the release security review of 2026-10-31): one whose `Host` does not
/// name the computer at the port its connection reached (`names_this_computer`) is refused 421 with no body before any
/// route sees it, the hello's included, so a web page whose name an attacker points at the computer's address (DNS
/// rebinding) reaches nothing; noted once a minute per peer. The Bluetooth bridge calls `router` without it: no `Host`
/// travels there.
async fn host(State(lan): State<Arc<Lan>>, ConnectInfo(conn): ConnectInfo<Conn>, req: Request, next: Next) -> Response {
	let mdns = lan.helper().mdns;
	let named = |value: &str| names_this_computer(value, conn.local.port(), mdns.as_deref());
	let header = req.headers().get(header::HOST).and_then(|v| v.to_str().ok());
	let authority = req.uri().authority().map(Authority::as_str);
	if !header.is_some_and(named) || !authority.is_none_or(named) {
		let peer = Peer::Lan(conn.peer.ip().to_canonical());
		lan.note(&peer, Topic::Refused, || format!("lan: refused {peer}: a Host that names another computer (421)"));
		return StatusCode::MISDIRECTED_REQUEST.into_response();
	}
	next.run(req).await
}

/// The sealed routes behind the guard, then the open hello, which `route_layer` leaves out: what the LAN's listeners
/// serve behind `host` and the Bluetooth bridge calls (`blelink`), each request with its `Peer`.
pub fn router(lan: Arc<Lan>) -> Router {
	Router::new()
		.route(PAIR, post(pair))
		.route("/v1/unpair", post(unpair))
		.route("/v1/ping", get(ping))
		.route("/v1/sync", post(sync))
		.route("/v1/sheet", get(sheet))
		.route(HISTORY, get(history))
		.route(SHEETS, get(sheets))
		.route(APP_ROWS, get(app_rows))
		.route("/v1/body", post(body))
		.route("/v1/account", get(account).put(account_put))
		.route_layer(middleware::from_fn_with_state(lan.clone(), guard))
		.route("/v1/hello", get(hello))
		.layer(DefaultBodyLimit::max(BODY_BYTES))
		.with_state(lan)
}

/// One open connection's place: it frees the permit and the address's count when the connection ends.
struct Slot {
	peers: Arc<Mutex<HashMap<IpAddr, usize>>>,
	ip: IpAddr,
	_permit: OwnedSemaphorePermit,
}

impl Drop for Slot {
	fn drop(&mut self) {
		let mut peers = lock(&self.peers);
		if let Some(n) = peers.get_mut(&self.ip) {
			*n -= 1;
			if *n == 0 {
				peers.remove(&self.ip);
			}
		}
	}
}

/// Whether `peer` is on the home network the connection came in on although its address is not a private one: an
/// IPv6 global address in the same /64 as the helper's address it reached (SLAAC gives a LAN one /64). The helper
/// listens on `[::]` too, and a phone that looks the computer up by its `.local` name may pick its global IPv6 address:
/// refusing it after the accept would fail the request where a closed port lets the phone fall back to IPv4.
fn on_this_network(peer: IpAddr, local: IpAddr) -> bool {
	match (peer, local) {
		(IpAddr::V6(p), IpAddr::V6(l)) => {
			let global = |a: &std::net::Ipv6Addr| a.segments()[0] & 0xe000 == 0x2000; // 2000::/3, global unicast
			global(&p) && global(&l) && p.segments()[..4] == l.segments()[..4]
		}
		_ => false,
	}
}

/// A place for a new connection from `ip` reaching the helper's `local` address, or why it gets none: an address
/// outside the home network, or the helper or the address has as many open as it takes.
fn admit(
	ip: IpAddr,
	local: IpAddr,
	permits: &Arc<Semaphore>,
	peers: &Arc<Mutex<HashMap<IpAddr, usize>>>,
) -> Result<Slot, &'static str> {
	if !is_private(&ip) && !ip.is_loopback() && !on_this_network(ip, local) {
		return Err("not a private address");
	}
	let permit = permits.clone().try_acquire_owned().map_err(|_| "the helper holds as many connections as it takes")?;
	let mut open = lock(peers);
	let n = open.entry(ip).or_default();
	if *n >= CONNECTIONS_PER_PEER {
		return Err("this address holds as many connections as it may");
	}
	*n += 1;
	Ok(Slot { peers: peers.clone(), ip, _permit: permit })
}

/// A listener on one address, as every TCP listener of the helper is made (the LAN API's here, the games' on the
/// loopback, `games::gsi`); an IPv6 one takes IPv6 only, so `[::]` never claims the port of `0.0.0.0`.
pub(crate) fn listen(addr: SocketAddr) -> std::io::Result<TcpListener> {
	let socket = Socket::new(Domain::for_address(addr), Type::STREAM, Some(Protocol::TCP))?;
	if addr.is_ipv6() {
		socket.set_only_v6(true)?;
	}
	// a restarted helper binds while its last run's connections wait out TIME_WAIT, as tokio's own bind (mio's) does;
	// on Windows the option would let another program take the port, so mio leaves it off there too
	#[cfg(not(windows))]
	socket.set_reuse_address(true)?;
	#[cfg(windows)]
	exclusive(&socket)?;
	socket.set_nonblocking(true)?;
	socket.bind(&addr.into())?;
	socket.listen(128)?; // the standard library's and mio's backlog
	TcpListener::from_std(socket.into())
}

/// Windows: the port is this socket's alone, set before its bind as Microsoft asks of every server socket
/// (`SO_EXCLUSIVEADDRUSE`; "Using `SO_REUSEADDR` and `SO_EXCLUSIVEADDRUSE`"): without it a program of the same account
/// binds one of the computer's addresses at the port, with `SO_REUSEADDR` or without, and takes its connections. A
/// restart binds again once the last run's connections are no longer active (`TIME_WAIT` is not), and a refused bind
/// is tried again at the next look (`Helper::serve_lan`).
#[cfg(windows)]
fn exclusive(socket: &Socket) -> std::io::Result<()> {
	use std::os::windows::io::AsRawSocket as _;
	use windows::Win32::Networking::WinSock::{
		SO_EXCLUSIVEADDRUSE, SOCKET, SOCKET_ERROR, SOL_SOCKET, WSAGetLastError, setsockopt,
	};
	let raw = SOCKET(usize::try_from(socket.as_raw_socket()).map_err(std::io::Error::other)?);
	// SAFETY: `raw` is the open socket `socket` owns for the call; the option's value is a BOOL the call reads while
	// it lasts, and WSAGetLastError reads this thread's last Winsock error right after the call that failed.
	unsafe {
		if setsockopt(raw, SOL_SOCKET, SO_EXCLUSIVEADDRUSE, Some(&1i32.to_ne_bytes())) == SOCKET_ERROR {
			return Err(std::io::Error::from_raw_os_error(WSAGetLastError().0));
		}
	}
	Ok(())
}

/// Listens on an IPv4 and an IPv6 address on one port (0: the port the IPv4 listener gets, for both). The helper
/// passes the unspecified ones, `0.0.0.0` and `[::]`, so it listens on every address of the computer: one that changes
/// networks keeps its listeners, and a device reaches it at whichever address it finds, its `.local` name's IPv6 ones
/// included; outside addresses are still refused at accept (`admit`). The tests pass the loopback ones.
///
/// # Errors
/// IPv4 cannot listen (another program holds the port); IPv6 failing is logged and leaves IPv4 alone.
pub fn bind(v4: Ipv4Addr, v6: Ipv6Addr, port: u16) -> std::io::Result<Vec<TcpListener>> {
	let first = listen(SocketAddr::from((v4, port)))?;
	let port = first.local_addr()?.port();
	let mut listeners = vec![first];
	match listen(SocketAddr::from((v6, port))) {
		Ok(second) => listeners.push(second),
		Err(e) => log::warn!("lan: no IPv6 listener on [{v6}]:{port} ({e}); IPv4 only"),
	}
	Ok(listeners)
}

/// The next connection on any of the listeners.
fn accept_any(listeners: &[TcpListener], cx: &mut Context<'_>) -> Poll<std::io::Result<(TcpStream, SocketAddr)>> {
	for listener in listeners {
		if let Poll::Ready(accepted) = listener.poll_accept(cx) {
			return Poll::Ready(accepted);
		}
	}
	Poll::Pending
}

/// Serves the routes behind `host` on bound listeners until `shutdown` changes. hyper's header timer (30 s) reaps a
/// vanished device's keep-alive connection and never touches the 5-s hold, which runs after the head is read; axum's
/// own `serve` installs no timer. A connection takes a place first (`admit`): an outside address never reaches hyper,
/// and the log notes the drop.
pub async fn serve(listeners: Vec<TcpListener>, lan: Arc<Lan>, mut shutdown: watch::Receiver<bool>) {
	let permits = Arc::new(Semaphore::new(CONNECTIONS));
	let peers = Arc::new(Mutex::new(HashMap::new()));
	let mut make = router(lan.clone())
		.layer(middleware::from_fn_with_state(lan.clone(), host))
		.into_make_service_with_connect_info::<Conn>();
	let mut http = auto::Builder::new(TokioExecutor::new()).http1_only();
	http.http1().timer(TokioTimer::new()).header_read_timeout(Duration::from_secs(30));
	loop {
		let accepted = tokio::select! {
			accepted = std::future::poll_fn(|cx| accept_any(&listeners, cx)) => accepted,
			_ = shutdown.changed() => return,
		};
		let (tcp, peer) = match accepted {
			Ok(accepted) => accepted,
			Err(e) => {
				log::warn!("lan: accept: {e}");
				tokio::time::sleep(Duration::from_millis(100)).await; // out of file descriptors: no hot loop
				continue;
			}
		};
		let ip = peer.ip().to_canonical();
		// with no address of its own the connection's requests name no port of it, so `host` refuses them
		let local = tcp.local_addr().unwrap_or(peer);
		let slot = match admit(ip, local.ip().to_canonical(), &permits, &peers) {
			Ok(slot) => slot,
			Err(why) => {
				lan.note(&Peer::Lan(ip), Topic::Refused, || format!("lan: dropped {ip}: {why}"));
				continue; // the connection closes
			}
		};
		let Ok(svc) = make.call(Conn { peer, local }).await;
		let http = http.clone();
		tokio::spawn(async move {
			let _slot = slot;
			let _ = http.serve_connection(TokioIo::new(tcp), TowerToHyperService::new(svc)).await;
		});
	}
}

#[cfg(test)]
mod tests {
	use std::collections::HashMap;
	use std::net::IpAddr;
	use std::sync::{Arc, Mutex};

	use tokio::sync::Semaphore;

	use super::{NOTE_EVERY_MS, NOTED_PEERS, Noted, Peer, Topic, admit, names_this_computer};

	#[test]
	fn only_a_host_that_names_this_computer_at_its_port_is_served() {
		let mdns = Some("Davids-MacBook-Pro.local");
		let cases: [(&str, bool); 18] = [
			("192.168.1.20:8787", true),
			("[fe80::1c2a:3f%en0]:8787", true),
			("[2a02:1210:5c3b:9b00::87]:8787", true),
			("localhost:8787", true),
			("LOCALHOST:8787", true),
			("Davids-MacBook-Pro.local:8787", true),
			("davids-macbook-pro.local.:8787", true),
			("192.168.1.20:8788", false),
			("192.168.1.20", false),
			("192.168.1.20:", false),
			("fe80::1:8787", false),
			("[not-an-address]:8787", false),
			("attacker.example:8787", false),
			("192.168.1.20.attacker.example:8787", false),
			("Other-Mac.local:8787", false),
			("localhost.attacker.example:8787", false),
			("", false),
			(":8787", false),
		];
		for (host, served) in cases {
			assert_eq!(names_this_computer(host, 8787, mdns), served, "{host}");
		}
		assert!(!names_this_computer("Davids-MacBook-Pro.local:8787", 8787, None), "no .local name: none of it");
	}

	#[test]
	fn only_the_home_network_gets_a_connection_whatever_address_the_helper_listens_on() {
		let (permits, peers) = (Arc::new(Semaphore::new(4)), Arc::new(Mutex::new(HashMap::new())));
		let v4 = IpAddr::from([192, 168, 1, 20]);
		let lan6: IpAddr = "2a02:1210:5c3b:9b00::87".parse().unwrap_or(v4);
		let cases: [(&str, IpAddr, bool); 5] = [
			("the phone on the Wi-Fi", IpAddr::from([192, 168, 1, 23]), true),
			("an address on the internet", IpAddr::from([8, 8, 8, 8]), false),
			("the phone's global IPv6 address on the same LAN", "2a02:1210:5c3b:9b00::23".parse().unwrap_or(v4), true),
			("a global IPv6 address elsewhere", "2a02:1210:5c3b:9c00::23".parse().unwrap_or(v4), false),
			("a link-local IPv6 address", "fe80::1c2a:3f".parse().unwrap_or(v4), true),
		];
		for (case, peer, admitted) in cases {
			let local = if peer.is_ipv4() { v4 } else { lan6 };
			assert_eq!(admit(peer, local, &permits, &peers).is_ok(), admitted, "{case}");
		}
	}

	#[test]
	fn many_peers_never_grow_the_refusal_log_past_its_cap() {
		let mut noted = Noted::default();
		for n in 0..u32::try_from(NOTED_PEERS + 10).unwrap_or(u32::MAX) {
			noted.first(&Peer::Lan(IpAddr::from((10_u32 << 24 | n).to_be_bytes())), Topic::Refused, 0);
		}
		assert_eq!(noted.at.len(), NOTED_PEERS);
	}

	#[test]
	fn a_peer_gets_one_line_a_minute_per_topic_and_another_peer_its_own() {
		let (phone, stranger) =
			(&Peer::Lan(IpAddr::from([192, 168, 1, 23])), &Peer::Lan(IpAddr::from([192, 168, 1, 31])));
		let mut noted = Noted::default();
		assert!(noted.first(phone, Topic::Refused, 0), "the first refusal is logged");
		assert!(!noted.first(phone, Topic::Refused, 1_000), "a retry within the minute is not");
		assert!(noted.first(phone, Topic::Hello, 1_000), "another topic has its own minute");
		assert!(noted.first(stranger, Topic::Refused, 1_000), "another peer has its own minute");
		assert!(!noted.first(phone, Topic::Refused, NOTE_EVERY_MS - 1));
		assert!(noted.first(phone, Topic::Refused, NOTE_EVERY_MS), "a minute later, a line again");
		assert!(noted.first(stranger, Topic::Pair, 2 * NOTE_EVERY_MS));
		assert_eq!(noted.at.len(), 1, "what is a minute old is forgotten: {:?}", noted.at.keys());
	}
}
