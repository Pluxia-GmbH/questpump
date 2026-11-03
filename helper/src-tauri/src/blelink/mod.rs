//! The Bluetooth link's bridge (docs/plan.md slice 43, with its fixes after the review of 2026-10-17; rust-helper
//! `bluetooth.md`, "The Bluetooth link's peripheral"): the computer a GATT peripheral that serves the LAN's routes to the
//! phones over Bluetooth, so a phone follows the game with its screen locked and links where the LAN cannot reach it.
//! The platform's peripheral (`mac.rs`: `CBPeripheralManager`; `win.rs`: `GattServiceProvider`) publishes the service,
//! advertises it, answers each central's read of the hello (`Hellos`: made for that central with a challenge drawn for
//! its connection) and passes on what the centrals do; this task puts each subscribed central's chunks together
//! (`ble::Reassembly`), opens each envelope under the current pairing key's ble key and that central's challenge, reads
//! its head and calls the routes the LAN serves (`lan::router`) with the helper's own bearer and the central as the
//! request's `Peer`, so the held sync, its holder rule, the re-offer, the receipts, freshness and replay are the LAN's.
//! Each answer goes back sealed under the same key, id and challenge, cut to the central's notification size, to that
//! central alone, through its `Queue` at the platform: a page's answer after every other, the centrals in turn. A call
//! keeps its place among a central's four from its envelope's opening until its answer's last chunk went to the radio;
//! one past them is answered `503`. An envelope that does not open is answered nothing and counts toward the central's
//! throttle, as does a message its reassembly dropped while no envelope of it had opened (`Lan::failed`: ten a minute
//! and its writes are ignored for a minute); a head that does not read is `400`, an answer past 2 MiB `500`, both with
//! no body. A central's unsubscribe, its read of the hello from the start, its silence past the longest hold and a new
//! pairing code end its calls and hand the link on (`Links::gone`); a new code also forgives every central and publishes
//! the service again. The latency asked of each central is medium while idle and low while a pump runs. Let go, the task
//! answers its calls in flight (an unpair's among them) for at most `STOP_WAIT`, then lets every central go and stops the
//! peripheral. The runtime starts it while the link's switch is on and a phone is paired or the pairing code can show
//! (`runtime/ble_link.rs`).

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use axum::Router;
use axum::body::Body;
use axum::http::{HeaderValue, Method as HttpMethod, Request as HttpRequest, StatusCode, Uri, header};
use questpump_core::ble::{self, Challenge, Direction, Method, Reassembly, Request, Response, Step};
use questpump_core::link::{self, Ble};
use questpump_core::seal::Keys;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::Instant;
use tower::ServiceExt as _;

use crate::lan::{self, Held, Lan, Links, Peer};

#[cfg(target_os = "macos")]
mod mac;
#[cfg(target_os = "macos")]
use mac as platform;
#[cfg(windows)]
mod win;
#[cfg(windows)]
use win as platform;
#[cfg(windows)]
pub(crate) use win::buffer;
#[cfg(not(any(target_os = "macos", windows)))]
mod platform {
	//! No peripheral on another system: the helper ships for macOS and Windows alone.
	use std::sync::Arc;

	use questpump_core::link::Ble;

	use super::{Events, Hellos, Latency, Outgoing, Radio};

	pub(super) struct Peripheral;

	impl Peripheral {
		pub(super) fn start(events: Events, _hellos: Arc<Hellos>) -> Self {
			events.send(Radio::State(Ble::Missing));
			Self
		}
		pub(super) fn send(&self, _central: &str, _message: Outgoing) {}
		pub(super) fn cancel(&self, _central: &str, _id: u8) {}
		pub(super) fn forget(&self, _central: &str) {}
		pub(super) fn latency(&self, _central: &str, _latency: Latency) {}
		pub(super) fn republish(&self) {}
		pub(super) fn check(&self) {}
		pub(super) fn quiet(&self) {}
		pub(super) fn close(&mut self) -> std::future::Ready<()> {
			std::future::ready(())
		}
	}

	pub(super) fn may_ask() -> bool {
		true
	}
}

/// How often the messages under way are looked at for one waiting past `ble::CHUNK_GAP_MS`, the silent centrals let go
/// and the platform asked whether it still advertises.
const TICK: Duration = Duration::from_secs(1);
/// A subscribed central with no call in flight that the bridge heard nothing of for this long is let go: the longest a
/// held sync waits on Bluetooth and a minute more (one whose going the system never told, as across a Mac's sleep).
const IDLE_MS: i64 = link::BLE_WAIT_MAX_MS + 60_000;
/// A central that read the hello and did not subscribe within this is forgotten (a phone subscribes at once after).
const UNSUBSCRIBED_MS: i64 = 60_000;
/// The most centrals the bridge, the hellos and the Mac's subscriptions keep: a new one past it lets the longest silent
/// one without a call go, so a stranger reading the hello from many addresses grows nothing.
pub(crate) const CENTRALS_MAX: usize = 32;
/// The most messages waiting for one central at the platform: its calls' answers and as many with no call behind them
/// (`400`, `503`); one past it is dropped unsent.
const QUEUE_MAX: usize = 2 * ble::IN_FLIGHT_MAX;
/// How long a bridge that was let go waits for its calls' answers to reach the radio (an unpair's: the phone hears it
/// before the link stops) before it lets every central go.
const STOP_WAIT: Duration = Duration::from_secs(5);
/// A publish or an advertising that failed is tried again after this, doubled at each failure in a row (`backoff`).
const RETRY_FIRST: Duration = Duration::from_secs(2);
/// The failures in a row the platform tries again after; one more, and it gives up (`Radio::Failed`).
const RETRIES: u32 = 5;

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
	m.lock().unwrap_or_else(PoisonError::into_inner)
}

/// How long after the `failures`-th failure in a row of a publish or an advertising the platform tries again: 2, 4, 8,
/// 16 and 32 s, then none, and it gives up. A radio turned off is no failure: the platform says `off` and publishes at
/// the next on.
pub(crate) fn backoff(failures: u32) -> Option<Duration> {
	(1..=RETRIES).contains(&failures).then(|| RETRY_FIRST * 2_u32.pow(failures - 1))
}

/// What the platform's peripheral tells the bridge.
pub(crate) enum Radio {
	/// How the computer's Bluetooth stands now: `on` once the service is published and advertised.
	State(Ble),
	/// It gave up: a publish or an advertising that failed `RETRIES` times in a row, or the system's refusal of the
	/// service, with the system's why for `helper.log` (the Phones page says only that it could not start or stopped).
	Failed(String),
	/// A central read the hello from its start: the challenge its connection's envelopes carry (`Hellos::read`).
	Hello { central: String, challenge: Challenge },
	/// A central subscribed to `tx`; the most one notification carries to it (`CBCentral.maximumUpdateValueLength`,
	/// `GattSubscribedClient.MaxNotificationSize`).
	Subscribed { central: String, size: usize },
	/// A central's subscription ended: it unsubscribed or disconnected, or the radio went.
	Unsubscribed { central: String },
	/// A central wrote `value` to `rx`, with its notification size while its `tx` subscription stands as the platform
	/// knows it at the write (the Mac's own list, Windows' `SubscribedClients`); none without one.
	Written { central: String, value: Vec<u8>, size: Option<usize> },
}

/// What the bridge hears, in the order it happened: the platform's words, the calls' answers and the messages that
/// went to the radio.
enum Event {
	Radio(Radio),
	Answered(Answered),
	/// A message handed to the platform went to the radio whole, or was dropped (`Done`): its call's place frees.
	Sent {
		central: String,
		call: Option<u64>,
	},
}

/// Where the platform tells the bridge.
#[derive(Clone)]
pub(crate) struct Events(mpsc::UnboundedSender<Event>);

impl Events {
	pub(crate) fn send(&self, radio: Radio) {
		let _ = self.0.send(Event::Radio(radio));
	}
}

/// The connection latency asked of a central: medium while idle, low while a pump runs (a wish the central may not
/// grant).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Latency {
	Medium,
	Low,
}

/// Whether the platform may start the peripheral now without asking the player out of context: on macOS once the
/// Bluetooth grant is answered (`CBManager.authorization`), since a `CBPeripheralManager` made before raises the
/// prompt; the Phones page and the pairing window ask it in context. Windows asks nothing.
pub(crate) fn may_start_unasked() -> bool {
	platform::may_ask()
}

/// What the bridge tells the runtime.
pub(crate) enum Heard {
	/// The link's state, for every response's helper block, the Phones page and the counts.
	State(Ble),
	/// It gave up (`Radio::Failed`): the runtime stops it, and the Phones page says it could not start or stopped.
	Failed(String),
	/// A bridge let go ended: the peripheral is gone, so another may start.
	Stopped,
}

/// What a hello's proof is keyed with: the current pairing key's hello key (`seal::hello_key`), so only a phone that
/// holds the key tells this computer's hello from another's (since the release security review of 2026-10-31, L3).
pub(crate) type HelloKey = [u8; 32];

/// The hello each central reads (`ble::hello`): at a read from its start made with a challenge drawn for that central,
/// which goes to the bridge (`Radio::Hello`) before the read is answered, so the bridge has it before the central's
/// first write; kept for that central's reads at a later offset (a long read: 54 bytes pass ATT MTU 23's 22). A new
/// pairing code makes them anew. Shared by the bridge and the platform's read handler.
pub(crate) struct Hellos(Mutex<Made>);

struct Made {
	key: HelloKey,
	/// The latest hello of each central, `CENTRALS_MAX` at most.
	kept: VecDeque<Kept>,
}

/// A central's latest hello and the challenge it was made with.
struct Kept {
	central: String,
	challenge: Challenge,
	bytes: Vec<u8>,
}

impl Hellos {
	fn new(key: HelloKey) -> Self {
		Self(Mutex::new(Made { key, kept: VecDeque::new() }))
	}

	/// A read of `central`'s hello at `offset`: from the start a new challenge and hello, told to `events`; at a later
	/// offset the kept bytes from there; none past their end, or with none kept (`InvalidOffset`).
	pub(crate) fn read(&self, events: &Events, central: &str, offset: usize) -> Option<Vec<u8>> {
		let mut made = lock(&self.0);
		if offset > 0 {
			let kept = made.kept.iter().find(|k| k.central == central);
			return kept.and_then(|k| k.bytes.get(offset..)).map(<[u8]>::to_vec);
		}
		let mut challenge = [0; ble::CHALLENGE_BYTES];
		rand::fill(&mut challenge);
		let bytes = ble::hello(&made.key, lan::now_ms().div_euclid(1_000), &challenge);
		made.kept.retain(|k| k.central != central);
		if made.kept.len() >= CENTRALS_MAX {
			made.kept.pop_front();
		}
		made.kept.push_back(Kept { central: central.to_owned(), challenge, bytes: bytes.clone() });
		events.send(Radio::Hello { central: central.to_owned(), challenge });
		Some(bytes)
	}

	/// A new pairing code: the hellos from now on show its key, and none kept reads on.
	fn rekey(&self, key: HelloKey) {
		let mut made = lock(&self.0);
		made.key = key;
		made.kept.clear();
	}

	/// A central went: the hello it read with `challenge` goes, never one it read since. A read reaches `read` at once
	/// and the bridge hears of a going later, so a phone that turns `tx` off and reads the hello again at once (an
	/// iPhone's quiesce and wake) keeps the hello its long read still needs.
	fn forget(&self, central: &str, challenge: Option<Challenge>) {
		lock(&self.0).kept.retain(|k| k.central != central || Some(k.challenge) != challenge);
	}
}

/// One message for a central: its id, its chunks in order, whether it goes ahead of the page answers, and what it tells
/// when it went.
pub(crate) struct Outgoing {
	id: u8,
	chunks: VecDeque<Vec<u8>>,
	urgent: bool,
	_done: Done,
}

/// Told when its message went to the radio whole, or was dropped (its central went, the queue was full, the bridge
/// stopped): the call's place at the bridge frees, and a held sync's answer is no longer pending (`Held`).
struct Done {
	events: mpsc::UnboundedSender<Event>,
	central: String,
	call: Option<u64>,
	_held: Option<Held>,
}

impl Drop for Done {
	fn drop(&mut self) {
		let _ = self.events.send(Event::Sent { central: std::mem::take(&mut self.central), call: self.call });
	}
}

/// A central's messages waiting for the radio, which the platform sends a chunk at a time (`next`, then `sent` once the
/// radio took it): an urgent one goes ahead of every page answer, even one partly sent (a phone puts four messages
/// together at once, each by its id), at most `QUEUE_MAX`.
#[derive(Default)]
pub(crate) struct Queue {
	messages: VecDeque<Outgoing>,
}

impl Queue {
	pub(crate) fn push(&mut self, message: Outgoing) {
		if self.messages.len() >= QUEUE_MAX {
			return; // dropped unsent: its `Done` tells
		}
		let at = if message.urgent {
			self.messages.iter().position(|m| !m.urgent).unwrap_or(self.messages.len())
		} else {
			self.messages.len()
		};
		self.messages.insert(at, message);
	}

	/// The next chunk to send.
	pub(crate) fn next(&self) -> Option<&[u8]> {
		self.messages.front()?.chunks.front().map(Vec::as_slice)
	}

	/// The chunk `next` gave went to the radio; its message's last tells (`Done`).
	pub(crate) fn sent(&mut self) {
		let Some(message) = self.messages.front_mut() else { return };
		message.chunks.pop_front();
		if message.chunks.is_empty() {
			self.messages.pop_front();
		}
	}

	/// The messages under an id dropped: its call was replaced, and the phone would read them as the new one's.
	pub(crate) fn cancel(&mut self, id: u8) {
		self.messages.retain(|m| m.id != id);
	}
}

/// What the runtime tells a running bridge.
enum Order {
	/// A new pairing code: every central goes and is forgiven, the hellos show the new key, the service is published
	/// again.
	Rekey(HelloKey),
	/// A pump runs (low latency) or not (medium).
	Busy(bool),
	/// The runtime let it go.
	Stop,
}

/// The running bridge. `stop` (or dropping it) lets it go: it answers its calls in flight for at most `STOP_WAIT`, then
/// lets every central go, stops the peripheral and says `Heard::Stopped`.
pub(crate) struct BleLink {
	orders: mpsc::UnboundedSender<Order>,
	task: JoinHandle<()>,
}

impl BleLink {
	/// Starts the peripheral with the hellos of the current pairing code's hello key, serving `lan`'s routes; what it
	/// hears goes to `heard`.
	pub(crate) fn start(lan: Arc<Lan>, links: Links, key: HelloKey, heard: mpsc::UnboundedSender<Heard>) -> Self {
		let (orders, rx) = mpsc::unbounded_channel();
		let task = tokio::spawn(run(lan, links, key, heard, rx));
		Self { orders, task }
	}

	/// Whether the bridge ended (let go, or a panic, which the crash report names).
	pub(crate) fn ended(&self) -> bool {
		self.task.is_finished()
	}

	/// A new pairing code: its hello key.
	pub(crate) fn rekey(&self, key: HelloKey) {
		let _ = self.orders.send(Order::Rekey(key));
	}

	/// A pump runs, or none does: each central's latency.
	pub(crate) fn busy(&self, busy: bool) {
		let _ = self.orders.send(Order::Busy(busy));
	}

	/// Lets it go (dropping it does too), after the answers of its calls in flight.
	pub(crate) fn stop(&self) {
		let _ = self.orders.send(Order::Stop);
	}
}

/// A call at the helper: its number (`Bridge::calls`), and its task while the routes answer it; none once its answer
/// waits for the radio, where it keeps its place until it went (`Event::Sent`).
struct Call {
	number: u64,
	task: Option<JoinHandle<()>>,
}

/// One central the bridge hears from.
struct Central {
	/// The most a notification carries to it, while its `tx` subscription stands; none before and after.
	size: Option<usize>,
	/// Its connection's challenge, from its last read of the hello from the start; none before one.
	challenge: Option<Challenge>,
	/// An envelope of it opened on this connection: its dropped messages count for nothing since (the radio loses
	/// chunks).
	opened: bool,
	reassembly: Reassembly,
	/// Its calls, by message id: an answer whose call a newer one under the same id replaced is never sent, since the
	/// phone would take it for the newer call's.
	calls: HashMap<u8, Call>,
	/// When the bridge last heard of it (a read, a subscription, a write, a message that went to it).
	seen: i64,
}

impl Central {
	fn new(now: i64) -> Self {
		Self {
			size: None,
			challenge: None,
			opened: false,
			reassembly: Reassembly::new(ble::REQUEST_MAX_BYTES),
			calls: HashMap::new(),
			seen: now,
		}
	}
}

/// An answer for a central: its id, its call (none for an answer no call stands behind), the keys and challenge its
/// envelope opened under, whether it goes ahead of the page answers, the answer, and a held sync's pending mark.
struct Answered {
	central: String,
	id: u8,
	call: Option<u64>,
	keys: Arc<Keys>,
	challenge: Challenge,
	urgent: bool,
	answer: Response,
	held: Option<Held>,
}

/// A call's answer on its way to the bridge: a call that ends without one (a panic, or the bridge cancelled it) sends
/// the `500` it was made with, which the bridge drops for a call it no longer knows, so its place never stays taken.
struct Reply {
	events: mpsc::UnboundedSender<Event>,
	answered: Option<Answered>,
}

impl Reply {
	fn send(mut self, answer: Response, held: Option<Held>) {
		if let Some(mut answered) = self.answered.take() {
			answered.answer = answer;
			answered.held = held;
			let _ = self.events.send(Event::Answered(answered));
		}
	}
}

impl Drop for Reply {
	fn drop(&mut self) {
		if let Some(answered) = self.answered.take() {
			let _ = self.events.send(Event::Answered(answered));
		}
	}
}

/// The bridge's state on its task.
struct Bridge {
	lan: Arc<Lan>,
	links: Links,
	router: Router,
	peripheral: platform::Peripheral,
	hellos: Arc<Hellos>,
	centrals: HashMap<String, Central>,
	events: mpsc::UnboundedSender<Event>,
	heard: mpsc::UnboundedSender<Heard>,
	latency: Latency,
	/// The calls made so far, which numbers each.
	calls: u64,
	/// Let go: it takes no new call and tells the runtime nothing more.
	stopping: bool,
}

async fn run(
	lan: Arc<Lan>,
	links: Links,
	key: HelloKey,
	heard: mpsc::UnboundedSender<Heard>,
	mut orders: mpsc::UnboundedReceiver<Order>,
) {
	let (events_tx, mut events) = mpsc::unbounded_channel();
	let hellos = Arc::new(Hellos::new(key));
	let mut bridge = Bridge {
		router: lan::router(lan.clone()),
		lan,
		links,
		peripheral: platform::Peripheral::start(Events(events_tx.clone()), hellos.clone()),
		hellos,
		centrals: HashMap::new(),
		events: events_tx,
		heard: heard.clone(),
		latency: Latency::Medium,
		calls: 0,
		stopping: false,
	};
	let mut tick = tokio::time::interval(TICK);
	tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
	let mut deadline: Option<Instant> = None;
	loop {
		if deadline.is_some_and(|at| bridge.idle() || Instant::now() >= at) {
			break;
		}
		tokio::select! {
			order = orders.recv(), if deadline.is_none() => match order {
				Some(Order::Rekey(key)) => bridge.rekey(key).await,
				Some(Order::Busy(busy)) => bridge.busy(busy),
				Some(Order::Stop) | None => {
					deadline = Some(Instant::now() + STOP_WAIT);
					bridge.stopping();
				}
			},
			Some(event) = events.recv() => bridge.on_event(event).await,
			_ = tick.tick() => bridge.tick(),
		}
	}
	bridge.stop().await;
	drop(bridge);
	let _ = heard.send(Heard::Stopped);
}

impl Bridge {
	async fn on_event(&mut self, event: Event) {
		match event {
			Event::Radio(radio) => self.on_radio(radio).await,
			Event::Answered(answered) => self.answer(answered),
			Event::Sent { central, call } => self.sent(&central, call),
		}
	}

	async fn on_radio(&mut self, radio: Radio) {
		match radio {
			Radio::State(state) => {
				if state != Ble::On {
					// below powered on the system drops every connection and subscription
					self.let_all_go().await;
				}
				if !self.stopping {
					let _ = self.heard.send(Heard::State(state));
				}
			}
			Radio::Failed(why) => {
				self.let_all_go().await;
				if !self.stopping {
					let _ = self.heard.send(Heard::Failed(why));
				}
			}
			Radio::Hello { central, challenge } => self.hello(&central, challenge).await,
			Radio::Subscribed { central, size } => {
				let (peer, tag) = (Peer::Ble(central.clone()), ble::central_tag(&central));
				// copy-lint: not shown (a log line)
				self.lan.note_central(&peer, || format!("ble: central {tag} subscribed ({size} bytes a notification)"));
				self.peripheral.latency(&central, self.latency);
				if let Some(entry) = self.entry(&central) {
					entry.size = Some(size);
				}
			}
			Radio::Unsubscribed { central } => {
				let (peer, tag) = (Peer::Ble(central.clone()), ble::central_tag(&central));
				self.lan.note_central(&peer, || format!("ble: central {tag} went"));
				self.let_go(&central).await;
			}
			Radio::Written { central, value, size } => self.written(&central, &value, size),
		}
	}

	/// The central's entry, made when it is new (past `CENTRALS_MAX` one without a call goes first: the longest silent
	/// of those that never subscribed, else of the rest; none when every one has a call), and marked heard of now.
	fn entry(&mut self, central: &str) -> Option<&mut Central> {
		let now = lan::now_ms();
		if !self.centrals.contains_key(central) {
			if self.centrals.len() >= CENTRALS_MAX {
				let without = self.centrals.iter().filter(|(_, c)| c.calls.is_empty());
				let silent = without.min_by_key(|(_, c)| (c.size.is_some(), c.seen));
				let silent = silent.map(|(id, _)| id.clone())?;
				self.forget(&silent);
			}
			self.centrals.insert(central.to_owned(), Central::new(now));
		}
		let entry = self.centrals.get_mut(central)?;
		entry.seen = now;
		Some(entry)
	}

	/// A central read the hello from its start: a new connection, or the phone found the service changed. Its calls end
	/// and its reassembly empties, as at an unsubscribe (its subscription stands as the platform says), and its
	/// envelopes open under the new challenge alone.
	async fn hello(&mut self, central: &str, challenge: Challenge) {
		if self.stopping {
			return;
		}
		if let Some(entry) = self.centrals.get_mut(central) {
			let calls = std::mem::take(&mut entry.calls);
			entry.reassembly = Reassembly::new(ble::REQUEST_MAX_BYTES);
			end(calls).await;
			self.peripheral.forget(central);
			self.links.gone(central);
		}
		if let Some(entry) = self.entry(central) {
			entry.challenge = Some(challenge);
			entry.opened = false;
		}
	}

	/// A chunk a central wrote: none called for a central without a standing `tx` subscription (uncounted), ignored
	/// while it is throttled, else put together; a whole message is called, a dropped one counted against the central
	/// while no envelope of it has opened on its connection.
	fn written(&mut self, central: &str, value: &[u8], size: Option<usize>) {
		let Some(size) = size else { return };
		let peer = Peer::Ble(central.to_owned());
		if self.stopping || self.lan.throttled(&peer) {
			return;
		}
		let Some(entry) = self.entry(central) else { return };
		entry.size = Some(size);
		let opened = entry.opened;
		match entry.reassembly.push(value, lan::now_ms()) {
			Step::Partial | Step::Ignored => {}
			Step::Dropped { id, why } => {
				if !opened {
					self.lan.failed(&peer, &format!("message {id} dropped ({})", why.word()));
				}
			}
			Step::Message { id, envelope } => self.message(central, id, &envelope),
		}
	}

	/// A whole message: opened under the current key and the central's challenge, then read and called; an envelope
	/// that does not open is answered nothing and counted against the central, a head that does not read is answered
	/// `400`, a call past `ble::IN_FLIGHT_MAX` `503` (counted for nothing: it proved the key).
	fn message(&mut self, central: &str, id: u8, envelope: &[u8]) {
		let peer = Peer::Ble(central.to_owned());
		let keys = self.lan.keys();
		let Some(entry) = self.centrals.get_mut(central) else { return };
		let opened = entry.challenge.and_then(|c| keys.ble.open(Direction::Rx, id, &c, envelope).ok().map(|p| (c, p)));
		let Some((challenge, plain)) = opened else {
			self.lan.failed(&peer, "an envelope that does not open"); // copy-lint: not shown (a log line)
			return;
		};
		entry.opened = true;
		// a call again under an id still answering: the phone gave that one up (it reads a call as long as on the LAN),
		// and the old answer, waiting or partly sent, would read as the new one's
		if let Some(old) = entry.calls.remove(&id) {
			match old.task {
				Some(task) => task.abort(),
				None => self.peripheral.cancel(central, id),
			}
		}
		let busy = entry.calls.len() >= ble::IN_FLIGHT_MAX;
		let answered = |status: StatusCode| Answered {
			central: central.to_owned(),
			id,
			call: None,
			keys: keys.clone(),
			challenge,
			urgent: true,
			answer: Response { status: status.as_u16(), body: Vec::new() },
			held: None,
		};
		let read = Request::read(&plain).map_err(|e| e.to_string());
		let (request, urgent) = match read.and_then(|r| {
			let urgent = !lan::is_page(&r.path);
			http_request(r, &keys, &peer).map(|request| (request, urgent))
		}) {
			Ok(it) => it,
			Err(why) => {
				log::warn!("ble: refused {peer}: a head that does not read ({why}, 400)");
				self.send(answered(StatusCode::BAD_REQUEST));
				return;
			}
		};
		if busy {
			let (tag, most) = (ble::central_tag(central), ble::IN_FLIGHT_MAX);
			// copy-lint: not shown (a log line)
			self.lan.note_central(&peer, || format!("ble: central {tag}: call {id} past {most} in flight (503)"));
			self.send(answered(StatusCode::SERVICE_UNAVAILABLE));
			return;
		}
		self.calls += 1;
		let number = self.calls;
		let reply = Reply {
			events: self.events.clone(),
			answered: Some(Answered { call: Some(number), urgent, ..answered(StatusCode::INTERNAL_SERVER_ERROR) }),
		};
		let router = self.router.clone();
		let task = tokio::spawn(async move {
			let (answer, held) = call(router, request).await;
			reply.send(answer, held);
		});
		if let Some(entry) = self.centrals.get_mut(central) {
			entry.calls.insert(id, Call { number, task: Some(task) });
		}
	}

	/// A call's answer: sealed and handed to the platform, unless a newer call under its id replaced it or its central
	/// went; the call keeps its place until the answer went to the radio.
	fn answer(&mut self, answered: Answered) {
		let Some(entry) = self.centrals.get_mut(&answered.central) else { return };
		let Some(call) = entry.calls.get_mut(&answered.id).filter(|c| Some(c.number) == answered.call) else { return };
		if call.task.take().is_none() {
			return; // answered already
		}
		self.send(answered);
	}

	/// An answer sealed under the keys and challenge its message opened under and cut to the central's notification
	/// size, to its queue at the platform; dropped for a central no longer subscribed, which frees its call (`Done`).
	fn send(&self, answered: Answered) {
		let Answered { central, id, call, keys, challenge, urgent, answer, held } = answered;
		let done = Done { events: self.events.clone(), central, call, _held: held };
		let Some(size) = self.centrals.get(&done.central).and_then(|c| c.size) else { return };
		let envelope = keys.ble.seal(Direction::Tx, id, &challenge, &answer.to_plain());
		let central = done.central.clone();
		let chunks = ble::chunks(id, &envelope, size).into();
		self.peripheral.send(&central, Outgoing { id, chunks, urgent, _done: done });
	}

	/// A message went to the radio whole, or was dropped: its call's place frees.
	fn sent(&mut self, central: &str, call: Option<u64>) {
		let Some(entry) = self.centrals.get_mut(central) else { return };
		entry.seen = lan::now_ms();
		if let Some(number) = call {
			entry.calls.retain(|_, c| c.number != number);
		}
	}

	/// The messages under way too long go, each counted against its central while none of its envelopes opened; the
	/// centrals silent too long go (`IDLE_MS`, `UNSUBSCRIBED_MS`); the platform looks whether it still advertises.
	fn tick(&mut self) {
		let now = lan::now_ms();
		for (central, entry) in &mut self.centrals {
			for (id, why) in entry.reassembly.tick(now) {
				if !entry.opened {
					self.lan.failed(&Peer::Ble(central.clone()), &format!("message {id} dropped ({})", why.word()));
				}
			}
		}
		let silent: Vec<String> = self
			.centrals
			.iter()
			.filter(|(_, c)| {
				let most = if c.size.is_some() { IDLE_MS } else { UNSUBSCRIBED_MS };
				c.calls.is_empty() && now - c.seen > most
			})
			.map(|(id, _)| id.clone())
			.collect();
		for central in silent {
			let (peer, tag) = (Peer::Ble(central.clone()), ble::central_tag(&central));
			// copy-lint: not shown (a log line)
			self.lan.note_central(&peer, || format!("ble: central {tag} silent too long, let go"));
			self.forget(&central);
		}
		if !self.stopping {
			self.peripheral.check();
		}
	}

	/// A pump runs or none does: each subscribed central's latency.
	fn busy(&mut self, busy: bool) {
		let latency = if busy { Latency::Low } else { Latency::Medium };
		if latency == self.latency {
			return;
		}
		self.latency = latency;
		for (central, entry) in &self.centrals {
			if entry.size.is_some() {
				self.peripheral.latency(central, latency);
			}
		}
	}

	/// A new pairing code: every central goes with its calls, its challenge and its throttle, the hellos show the new
	/// key, and the service is published again, so a connected phone hears its services changed and reads the hello.
	async fn rekey(&mut self, key: HelloKey) {
		self.hellos.rekey(key);
		self.let_all_go().await;
		self.lan.forgive_centrals();
		self.peripheral.republish();
	}

	/// The runtime let the bridge go: no new call, no word to the runtime, and no advertising.
	fn stopping(&mut self) {
		self.stopping = true;
		self.peripheral.quiet();
	}

	/// Whether no central has a call at the helper.
	fn idle(&self) -> bool {
		self.centrals.values().all(|c| c.calls.is_empty())
	}

	/// A central with no call in flight goes at once: its waiting messages, the hello of its connection's challenge,
	/// and the link it held.
	fn forget(&mut self, central: &str) {
		let challenge = self.centrals.remove(central).and_then(|entry| entry.challenge);
		self.peripheral.forget(central);
		self.hellos.forget(central, challenge);
		self.links.gone(central);
	}

	/// A central's subscription ended: its calls are cancelled (each awaited, so a sync that took the link has done so
	/// before), its waiting messages dropped, then the link passes on.
	async fn let_go(&mut self, central: &str) {
		if let Some(entry) = self.centrals.get_mut(central) {
			end(std::mem::take(&mut entry.calls)).await;
		}
		self.forget(central);
	}

	async fn let_all_go(&mut self) {
		let centrals: Vec<String> = self.centrals.keys().cloned().collect();
		for central in centrals {
			self.let_go(&central).await;
		}
	}

	/// The bridge stops: every central goes, then the peripheral, which is gone when this returns.
	async fn stop(&mut self) {
		self.let_all_go().await;
		self.peripheral.close().await;
	}
}

impl Drop for Bridge {
	/// A bridge that ends any way, a panic's unwinding too, lets every central it still knows go, so none stays the
	/// link's holder (`stop` let them go first, awaiting their calls).
	fn drop(&mut self) {
		for (central, entry) in self.centrals.drain() {
			for call in entry.calls.into_values() {
				if let Some(task) = call.task {
					task.abort();
				}
			}
			self.links.gone(&central);
		}
	}
}

/// A central's calls end: each running one cancelled and awaited; an answer waiting at the platform goes with its
/// central's messages (`Peripheral::forget`).
async fn end(calls: HashMap<u8, Call>) {
	for call in calls.into_values() {
		if let Some(task) = call.task {
			task.abort();
			let _ = task.await;
		}
	}
}

/// The HTTP request an opened envelope's head carries, with this helper's bearer and the central as its `Peer`.
fn http_request(r: Request, keys: &Keys, peer: &Peer) -> Result<HttpRequest<Body>, String> {
	let method = match r.method {
		Method::Get => HttpMethod::GET,
		Method::Post => HttpMethod::POST,
	};
	let uri = Uri::try_from(r.path.as_str()).map_err(|e| e.to_string())?;
	let bearer = HeaderValue::try_from(format!("Bearer {}", keys.bearer)).map_err(|e| e.to_string())?;
	let mut request = HttpRequest::builder().method(method).uri(uri).header(header::AUTHORIZATION, bearer);
	if !r.agent.is_empty() {
		request = request.header(header::USER_AGENT, HeaderValue::try_from(r.agent).map_err(|e| e.to_string())?);
	}
	if r.method == Method::Post {
		request = request.header(header::CONTENT_TYPE, "application/octet-stream");
	}
	request.extension(peer.clone()).body(Body::from(r.body)).map_err(|e| e.to_string())
}

/// The routes' answer to one call: its status and body as the LAN would answer them, or `500` with no body when its
/// envelope would pass `ble::RESPONSE_MAX_BYTES`; with a held sync's pending mark (`lan::Held`).
async fn call(router: Router, request: HttpRequest<Body>) -> (Response, Option<Held>) {
	let Ok(mut response) = router.oneshot(request).await;
	let held = response.extensions_mut().remove::<Held>();
	let status = response.status().as_u16();
	let room = ble::RESPONSE_MAX_BYTES - ble::ENVELOPE_MIN_BYTES - 2;
	let answer = match axum::body::to_bytes(response.into_body(), room).await {
		Ok(body) => Response { status, body: body.to_vec() },
		Err(_) => Response { status: StatusCode::INTERNAL_SERVER_ERROR.as_u16(), body: Vec::new() },
	};
	(answer, held)
}
