//! The sync's transport on the computer (docs/plan.md slices 60 and 63; contract/sync-api.yaml): the sync itself is the
//! desk's (`questpump_desk::sync`: the account, the keys, the batches, the pages' topics and intents), and the desktop
//! app sends what it asks (`questpump_desk::Platform::sync_call`, `live_open`, `live_close`, here [`SyncLink`]) off the
//! loop through the one HTTPS client (`crate::feedback::client`: no `Origin`, the helper's User-Agent), each answer
//! back on the loop as `questpump_desk::sync::News`, which the loop hands to `Desk::on_sync`.
//!
//! The pump push's posts are paced here too (slice 76; contract/sync-api.yaml, "The pump push"; [`Telling`]): while
//! Sync is on, an iPhone of the account follows this computer and a game with QuestPump runs, the loop tells the
//! account each state of the game's pump (`Helper::tell`, every turn), one post at a time and at most one a second,
//! the newest in the place of one that waits, and only a state that differs from the last the account took; 4 minutes
//! after the last post the account answered it says the `alive` word, which keeps the following, and one the account
//! did not take goes again after the wait. The state is the desk's
//! (`questpump_desk::activity::GamePump`, fed each command the loop queues for the link) and the post the sync's
//! (`Desk::tell`); each answer gives the account's clock (`AccountClock`), on which a state's moments are written.
//! The log names each word by its event, never a state's content.

use std::time::{Duration, Instant};

use futures_util::{SinkExt as _, StreamExt as _};
use questpump_core::protocol::Command;
use questpump_core::sync::LivePump;
use questpump_desk::activity::{GamePump, Told, Word};
use questpump_desk::sync::{ANSWER_MOST, Answer, Call, Called, LiveHeard, Method, News, Then, live_hello};
use tokio::sync::mpsc;

use super::Helper;
use crate::lan;

/// The sync's routes on questpump.com (contract/sync-api.yaml).
const BASE: &str = "https://questpump.com/api/sync/v1";
const LIVE: &str = "wss://questpump.com/api/sync/v1/live";

/// The desktop app's side of the sync's transport: the HTTPS client (none when it did not build: every call answers
/// that), where the answers go back to the loop, the live socket's task while it stands, this computer's name for the
/// device batch (the pair link's, read again every minute) and whether this run's secrets did not read.
pub(crate) struct SyncLink {
	client: Option<reqwest::Client>,
	news: mpsc::UnboundedSender<News>,
	live: Option<tokio::task::JoinHandle<()>>,
	pub(crate) name: String,
	pub(crate) secrets_unread: bool,
}

impl SyncLink {
	/// The transport, and where the server's answers come back to the loop.
	pub(crate) fn new(
		client: Option<reqwest::Client>,
		name: String,
		secrets_unread: bool,
	) -> (Self, mpsc::UnboundedReceiver<News>) {
		let (news, answers) = mpsc::unbounded_channel();
		(Self { client, news, live: None, name, secrets_unread }, answers)
	}

	pub(crate) fn reachable(&self) -> bool {
		self.client.is_some()
	}

	/// A call to the server off the loop, its answer back as `then`.
	pub(crate) fn call(&self, call: Call, then: Then) {
		let Some(client) = self.client.clone() else {
			let _ = self.news.send(then(Err("no HTTPS client".to_owned()))); // copy-lint: not shown (a log line)
			return;
		};
		let tx = self.news.clone();
		tokio::spawn(async move {
			let answer = request(&client, call).await;
			let _ = tx.send(then(answer));
		});
	}

	/// The live socket with `token`, in place of one that stands.
	pub(crate) fn live_open(&mut self, token: String, generation: u64) {
		self.live_close();
		self.live = Some(tokio::spawn(live(token, generation, self.news.clone())));
	}

	pub(crate) fn live_close(&mut self) {
		if let Some(live) = self.live.take() {
			live.abort();
		}
	}
}

/// The least time between two posts of the pump push.
const TELL_LEAST: Duration = Duration::from_secs(1);
/// The `alive` word goes this long after the last post the account answered.
const ALIVE_EVERY: Duration = Duration::from_mins(4);
/// A ping goes on the live socket this often, and a socket that sent nothing since the ping before it is over
/// (`live`).
const LIVE_PING: Duration = Duration::from_secs(30);
/// The wait after a post the account did not take, when its answer names none.
const TELL_RETRY: Duration = Duration::from_mins(1);
/// How long an exchange stays a candidate for the account's clock, counted back from the newest (`ClockOffset`'s
/// window in each phone core), in ms.
const CLOCK_WINDOW_MS: i64 = 60_000;
/// A move of the offset beyond both exchanges' error bounds that counts as a clock step (`ClockOffset`'s), in ms.
const CLOCK_STEP_MS: i64 = 50;

/// One post's exchange with the account: how far the account's clock was ahead of the computer's (ms), the round trip
/// and when the answer arrived (ms on the computer's clock).
#[derive(Clone, Copy)]
struct Exchange {
	offset: i64,
	round_trip: i64,
	at: i64,
}

/// The account's clock as the posts' answers give it, by the rule each phone core keeps the helper's clock by
/// (contract/companion-api.yaml, `ClockOffset`): the offset of the smallest round trip among the exchanges of the
/// last minute, a fresh window after a clock step; the computer's own clock until the first answer.
#[derive(Default)]
struct AccountClock {
	exchanges: Vec<Exchange>,
}

impl AccountClock {
	fn best(&self) -> Option<Exchange> {
		self.exchanges.iter().copied().min_by_key(|e| e.round_trip)
	}

	/// How far the account's clock is ahead of the computer's, in ms.
	fn offset(&self) -> i64 {
		self.best().map_or(0, |e| e.offset)
	}

	/// A post sent at `sent` and answered at `back` (ms on the computer's clock), which the account took at `t` on
	/// its own: `t` against the middle of the two. An answer that arrived before its post went changes nothing.
	fn add(&mut self, sent: i64, t: i64, back: i64) {
		let round_trip = back - sent;
		if round_trip < 0 {
			return;
		}
		let exchange = Exchange { offset: t - i64::midpoint(sent, back), round_trip, at: back };
		let stepped = self.best().is_some_and(|b| {
			(exchange.offset - b.offset).abs() > CLOCK_STEP_MS + i64::midpoint(round_trip, b.round_trip)
		});
		if stepped {
			self.exchanges.clear();
		}
		self.exchanges.push(exchange);
		self.exchanges.retain(|e| e.at >= back - CLOCK_WINDOW_MS);
	}
}

/// A post of the pump push that is out: its word, the state it told as the computer's own clock dates it (none for
/// the `alive` word), the following it was told for (`Desk::followed`) and when it went (ms on the computer's clock).
struct Post {
	word: Word,
	state: Option<LivePump>,
	since: i64,
	sent: i64,
}

/// The pump push on the computer (slice 76): the game's pump as the activity shows it, and what the account heard of
/// it.
#[derive(Default)]
pub(crate) struct Telling {
	pump: GamePump,
	/// The newest change no post told yet, as its word: the state as it then stands goes with it.
	due: Option<Word>,
	/// The post that is out, until its answer (one at a time).
	out: Option<Post>,
	/// The state the account took last, as the computer's own clock dates it: only one that differs goes. None while
	/// nothing is told (nobody follows, Sync off, no game with QuestPump), so the state as it stands goes first.
	told: Option<LivePump>,
	/// The following that state was told for: a later one is a phone that began to follow, which hears the state as
	/// it stands.
	told_since: Option<i64>,
	/// When the last post went.
	posted: Option<Instant>,
	/// When the account last answered a post, taken or refused: the `alive` word counts from here, so one that did
	/// not go through is said again after the wait, never 4 minutes later (the account ends a following after 10
	/// minutes without a word).
	answered: Option<Instant>,
	/// No post before this: the wait after one the account did not take.
	wait_until: Option<Instant>,
	clock: AccountClock,
}

impl Telling {
	/// A command the loop queued for the link at `now` (ms on the computer's clock), `held` while a device holds the
	/// link: what it changes of the game's pump is due as its word (an offer's alert only while no device holds the
	/// link, which alerts itself).
	pub(crate) fn command(&mut self, command: &Command, now: i64, held: bool) {
		if let Some(change) = self.pump.command(command, now) {
			self.due = Some(Word::of(change, held));
		}
	}
}

impl Helper {
	/// The pump push's next post, looked at every turn of the loop: none while one is out, within a second of the
	/// last or in the wait after one the account did not take; then the state as it stands where it differs from the
	/// last the account took or a phone began to follow, with the word of the newest change (`rest` without one), else
	/// the `alive` word 4 minutes after the last post the account answered. A computer nobody follows, one with Sync
	/// off and one where no game with QuestPump runs post nothing and forget what was told.
	pub(crate) fn tell(&mut self) {
		let t = &mut self.telling;
		let Some(since) = self.desk.followed().filter(|_| self.desk.wow_with_addon()) else {
			(t.due, t.told, t.told_since) = (None, None, None);
			return;
		};
		let now = Instant::now();
		if t.out.is_some()
			|| t.wait_until.is_some_and(|at| now < at)
			|| t.posted.is_some_and(|at| now < at + TELL_LEAST)
		{
			return;
		}
		let name = &self.address.name;
		let own = t.pump.state(name, 0);
		let began = t.told_since.is_none_or(|told| since > told);
		let (word, state) = if began || t.told.as_ref() != Some(&own) {
			(t.due.unwrap_or(Word::Rest), Some(own))
		} else {
			// the newest change left the state as the account has it
			t.due = None;
			if t.answered.is_some_and(|at| now < at + ALIVE_EVERY) {
				return;
			}
			(Word::Alive, None)
		};
		let dated = state.as_ref().map(|_| t.pump.state(name, t.clock.offset()));
		if self.desk.tell(word, dated.as_ref()) {
			(t.due, t.posted) = (None, Some(now));
			t.out = Some(Post { word, state, since, sent: lan::now_ms() });
		}
	}

	/// The post that is out answered (`Platform::told`): taken, its state is the last the account took and its answer
	/// the account's clock; refused for what it said, the same state never goes again; not taken, a state's word stays
	/// due unless a newer change took its place, and the state as it then stands, or the `alive` word, goes after the
	/// wait.
	pub(crate) fn on_told(&mut self, told: Told) {
		let t = &mut self.telling;
		let Some(Post { word, state, since, sent }) = t.out.take() else { return };
		match told {
			Told::Taken(at) => {
				if let Some(at) = at {
					t.clock.add(sent, at, lan::now_ms());
				}
				if word != Word::Alive {
					let alerts = if word == (Word::Offer { alert: true }) { ", alerting" } else { "" };
					log::info!("pump push: {} told{alerts}", word.event());
				}
			}
			Told::Refused => {}
			Told::Later(wait) => {
				// `answered` stays: an `alive` that did not go through is due again after the wait
				t.wait_until = Some(Instant::now() + wait.unwrap_or(TELL_RETRY));
				if word != Word::Alive {
					t.due.get_or_insert(word);
				}
				return;
			}
		}
		t.answered = Some(Instant::now());
		if let Some(state) = state {
			(t.told, t.told_since) = (Some(state), Some(since));
		}
	}
}

/// One request to the sync's routes: JSON in and out, the token as its bearer, no `Origin` (the client's).
async fn request(client: &reqwest::Client, call: Call) -> Called {
	let method = match call.method {
		Method::Get => reqwest::Method::GET,
		Method::Post => reqwest::Method::POST,
		Method::Put => reqwest::Method::PUT,
		Method::Delete => reqwest::Method::DELETE,
	};
	let mut req = client.request(method, format!("{BASE}{}", call.path));
	if let Some(token) = call.token {
		req = req.bearer_auth(token);
	}
	if let Some(body) = call.body {
		req = req.header(reqwest::header::CONTENT_TYPE, "application/json").body(body);
	}
	let why = |e: reqwest::Error| format!("{:#}", anyhow::Error::new(e.without_url()));
	let mut res = req.send().await.map_err(why)?;
	let status = res.status().as_u16();
	let retry = res
		.headers()
		.get(reqwest::header::RETRY_AFTER)
		.and_then(|v| v.to_str().ok())
		.and_then(|v| v.trim().parse::<u64>().ok());
	let mut body = Vec::new();
	while let Some(chunk) = res.chunk().await.map_err(why)? {
		if body.len() + chunk.len() > ANSWER_MOST {
			return Err(format!("an answer past {ANSWER_MOST} bytes")); // copy-lint: not shown (a log line)
		}
		body.extend_from_slice(&chunk);
	}
	Ok(Answer { status, retry, body })
}

/// The live socket: the token as its first message, then each message the server sends as a nudge
/// (`LiveHeard::message`), until it closes (its close code: 4401 for a token it did not take). It counts as connected
/// from the server's first message on, never from the token sent: the desk starts its waits again after a socket
/// that connected (`News::LiveEnded`), so one that ends unanswered would open the next every 5 s for good. A ping
/// goes every `LIVE_PING`, and a socket that sent nothing since the ping before it ends here with no code: the
/// server's platform answers each ping, and a connection that died without a close (the computer slept, its network
/// changed) would otherwise stay open and silent for good, with it the word of who follows this computer, which comes
/// on the socket alone.
async fn live(token: String, generation: u64, news: mpsc::UnboundedSender<News>) {
	use tokio_tungstenite::tungstenite::client::IntoClientRequest as _;
	use tokio_tungstenite::tungstenite::http::{HeaderValue, header};
	use tokio_tungstenite::tungstenite::{Bytes, Message};

	let mut connected = false;
	let mut heard = LiveHeard::default();
	let code = async {
		let mut request = LIVE.into_client_request().ok()?;
		request.headers_mut().insert(header::USER_AGENT, HeaderValue::from_static(crate::feedback::USER_AGENT));
		let dialled = tokio::time::timeout(Duration::from_secs(20), tokio_tungstenite::connect_async(request));
		let (mut ws, _) = dialled.await.ok()?.ok()?;
		ws.send(Message::text(live_hello(&token))).await.ok()?;
		let mut ping = tokio::time::interval_at(tokio::time::Instant::now() + LIVE_PING, LIVE_PING);
		ping.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
		let mut quiet = false;
		loop {
			tokio::select! {
				message = ws.next() => {
					quiet = false;
					match message?.ok()? {
						Message::Text(text) => {
							connected = true;
							if let Some(nudge) = heard.message(generation, text.as_str()) {
								let _ = news.send(nudge);
							}
						}
						Message::Close(frame) => return frame.map(|f| u16::from(f.code)),
						_ => {}
					}
				}
				_ = ping.tick() => {
					if quiet {
						return None;
					}
					ws.send(Message::Ping(Bytes::new())).await.ok()?;
					quiet = true;
				}
			}
		}
	}
	.await;
	let _ = news.send(News::LiveEnded(generation, code, connected));
}
