//! What slice 48's loopback feeds share (docs/plan.md slice 48; rust-helper rules 8 and 10): a task that runs while its
//! game wants it and stops when its handle is dropped, and the one HTTP client the loopback polls ask with: no proxy
//! (the system's would see the loopback's traffic), no redirect, short timeouts, an answer of at most `BODY_BYTES`
//! read.

use std::future::Future;
use std::time::Duration;

use questpump_core::games::Watched;
use tokio::sync::{mpsc, watch};

use super::{News, Word};

/// The longest answer a poll reads: the games' answers are well under a kilobyte.
const BODY_BYTES: usize = 64 * 1024;
/// How long a poll waits for the connection, and for the whole answer.
const CONNECT: Duration = Duration::from_secs(1);
const ANSWER: Duration = Duration::from_secs(2);

/// A feed's task while its game wants it; dropping it stops the task at its next wait.
pub(super) struct Task {
	_stop: watch::Sender<bool>,
}

impl Task {
	/// Runs `work` with the receiver that changes when the handle is dropped.
	pub(super) fn spawn<F, W>(work: W) -> Self
	where
		W: FnOnce(watch::Receiver<bool>) -> F,
		F: Future<Output = ()> + Send + 'static,
	{
		let (stop, stopped) = watch::channel(false);
		tokio::spawn(work(stopped));
		Self { _stop: stop }
	}
}

/// What a feed's task tells the loop, one word at a time; a word sent after the loop stopped listening is dropped.
#[derive(Clone)]
pub(super) struct Teller {
	watched: Watched,
	news: mpsc::UnboundedSender<News>,
}

impl Teller {
	pub(super) fn new(watched: Watched, news: mpsc::UnboundedSender<News>) -> Self {
		Self { watched, news }
	}

	pub(super) fn tell(&self, word: Word) {
		let _ = self.news.send(News::Fed(self.watched.clone(), word));
	}
}

/// The loopback polls' client; none when the system's TLS backend cannot start (no poll then, logged once).
pub(super) fn client() -> Option<reqwest::Client> {
	let built = reqwest::Client::builder()
		.no_proxy()
		.redirect(reqwest::redirect::Policy::none())
		.connect_timeout(CONNECT)
		.timeout(ANSWER)
		.build();
	match built {
		Ok(client) => Some(client),
		Err(e) => {
			log::warn!("games: no client for the loopback: {e}");
			None
		}
	}
}

/// A GET of `path` on 127.0.0.1 at `port`: the answer's body when it came whole with a success status and fits
/// `BODY_BYTES`, else none.
pub(super) async fn get(client: &reqwest::Client, port: u16, path: &str) -> Option<Vec<u8>> {
	let mut answer = client.get(format!("http://127.0.0.1:{port}{path}")).send().await.ok()?;
	if !answer.status().is_success() || answer.content_length().is_some_and(|n| n > BODY_BYTES as u64) {
		return None;
	}
	let mut body = Vec::new();
	while let Some(chunk) = answer.chunk().await.ok()? {
		if body.len() + chunk.len() > BODY_BYTES {
			return None;
		}
		body.extend_from_slice(&chunk);
	}
	Some(body)
}

/// Waits `every`, or until the handle is dropped: false then.
pub(super) async fn wait(every: Duration, stop: &mut watch::Receiver<bool>) -> bool {
	tokio::select! {
		() = tokio::time::sleep(every) => true,
		_ = stop.changed() => false,
	}
}
