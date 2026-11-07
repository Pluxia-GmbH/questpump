//! The sync's transport on the computer (docs/plan.md slices 60 and 63; contract/sync-api.yaml): the sync itself is the
//! desk's (`questpump_desk::sync`: the account, the keys, the batches, the pages' topics and intents), and the desktop
//! app sends what it asks (`questpump_desk::Platform::sync_call`, `live_open`, `live_close`, here [`SyncLink`]) off the
//! loop through the one HTTPS client (`crate::feedback::client`: no `Origin`, the helper's User-Agent), each answer
//! back on the loop as `questpump_desk::sync::News`, which the loop hands to `Desk::on_sync`.

use std::time::Duration;

use futures_util::{SinkExt as _, StreamExt as _};
use questpump_desk::sync::{ANSWER_MOST, Answer, Call, Called, LiveHeard, Method, News, Then, live_hello};
use tokio::sync::mpsc;

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
/// (`LiveHeard::message`), until it closes (its close code: 4401 for a token it did not take).
async fn live(token: String, generation: u64, news: mpsc::UnboundedSender<News>) {
	use tokio_tungstenite::tungstenite::Message;
	use tokio_tungstenite::tungstenite::client::IntoClientRequest as _;
	use tokio_tungstenite::tungstenite::http::{HeaderValue, header};

	let mut connected = false;
	let mut heard = LiveHeard::default();
	let code = async {
		let mut request = LIVE.into_client_request().ok()?;
		request.headers_mut().insert(header::USER_AGENT, HeaderValue::from_static(crate::feedback::USER_AGENT));
		let dialled = tokio::time::timeout(Duration::from_secs(20), tokio_tungstenite::connect_async(request));
		let (mut ws, _) = dialled.await.ok()?.ok()?;
		ws.send(Message::text(live_hello(&token))).await.ok()?;
		connected = true;
		while let Some(message) = ws.next().await {
			match message.ok()? {
				Message::Text(text) => {
					if let Some(nudge) = heard.message(generation, text.as_str()) {
						let _ = news.send(nudge);
					}
				}
				Message::Close(frame) => return frame.map(|f| u16::from(f.code)),
				_ => {}
			}
		}
		None
	}
	.await;
	let _ = news.send(News::LiveEnded(generation, code, connected));
}
