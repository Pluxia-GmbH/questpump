//! The page's state as topics (docs/plan.md slice 35, "The UI stack"): each topic one whole value, replaced on every
//! change and pushed to every page that subscribed (the desktop app's webviews through a `tauri::ipc::Channel`, which
//! an app command's channel gets with no capability file; the browser's page through its Worker). A message is
//! `{topic, rev, data}`; `rev` counts every change of every topic from 1, so a page that holds a topic's value at one
//! rev takes a message only when it is newer, whichever of the subscription's answer (`Boot`, every topic's value) and
//! a message sent meanwhile arrives first. A topic whose new value equals the one held is not sent again. A state that
//! publishes several topics from one change stages them under its own lock and sends them after it (`stage`, `send`),
//! so two changes made on two threads get their revs in the order they were made. Each page holds one sink (a page that
//! loads again replaces its own), and a sink whose page is gone is dropped at the first send that fails or when its
//! window is destroyed (`forget`).

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use serde::Serialize;
use serde_json::Value;

/// One topic's change, as the page's `lib/bridge.js` receives it.
#[derive(Serialize, Debug, Clone)]
pub struct Msg {
	topic: String,
	rev: u64,
	data: Value,
}

/// A topic's value at its rev.
#[derive(Serialize, Debug, Clone)]
pub struct Held {
	rev: u64,
	data: Value,
}

/// `desk_subscribe`'s answer: every topic's value, so the first paint needs no second round trip.
#[derive(Serialize, Debug)]
pub struct Boot {
	topics: BTreeMap<String, Held>,
}

/// Where a page takes its messages: the platform's half of publishing (the desktop app's webview channel, the
/// browser's Worker).
pub trait Sink: Send + Sync {
	/// Sends one message to the page; false once the page is gone, which drops the sink.
	fn send(&self, msg: Msg) -> bool;
}

/// A subscribed page: its label, its sink's id (one label's sink replaced while a send runs is told apart from the
/// one that failed) and its sink.
type Page = (String, u32, Arc<dyn Sink>);

#[derive(Default)]
struct Inner {
	topics: BTreeMap<String, Held>,
	rev: u64,
	/// Each subscribed page's sink, by its label.
	pages: Vec<Page>,
}

/// Topics staged under the lock of the state they come from, sent once it is released (`Publisher::stage`,
/// `Publisher::send`).
#[derive(Default)]
pub struct Staged(Vec<Msg>);

/// The topics and their subscribers; a cheap handle, shared by the commands, the loop and the window's events.
#[derive(Clone, Default)]
pub struct Publisher {
	inner: Arc<Mutex<Inner>>,
}

impl Publisher {
	/// Subscribes a page (replacing a sink it held before) and answers every topic's value, both under one lock, so
	/// every later change comes as a message with a higher rev.
	pub fn subscribe(&self, label: &str, id: u32, sink: Arc<dyn Sink>) -> Boot {
		let Ok(mut inner) = self.inner.lock() else { return Boot { topics: BTreeMap::new() } };
		inner.pages.retain(|(held, _, _)| held != label);
		inner.pages.push((label.to_owned(), id, sink));
		Boot { topics: inner.topics.clone() }
	}

	/// Replaces a topic's value and sends it to every subscriber, unless it equals the value held.
	pub fn publish(&self, topic: &str, data: &impl Serialize) {
		let mut staged = Staged::default();
		self.stage(&mut staged, topic, data);
		self.send(&staged);
	}

	/// Replaces a topic's value and gives it its rev, unless it equals the value held, for `send` to send. Called under
	/// the lock of the state the value comes from, so the revs follow the order of that state's changes: a value staged
	/// later has the higher rev whichever send arrives first, and the page keeps it.
	pub fn stage(&self, staged: &mut Staged, topic: &str, data: &impl Serialize) {
		let data = match serde_json::to_value(data) {
			Ok(data) => data,
			Err(e) => {
				log::warn!("desk: topic {topic} did not serialise: {e}");
				return;
			}
		};
		let Ok(mut inner) = self.inner.lock() else { return };
		if inner.topics.get(topic).is_some_and(|held| held.data == data) {
			return;
		}
		inner.rev += 1;
		let rev = inner.rev;
		inner.topics.insert(topic.to_owned(), Held { rev, data: data.clone() });
		staged.0.push(Msg { topic: topic.to_owned(), rev, data });
	}

	/// Sends what was staged to every subscriber, outside every lock: a send evaluates a script in the webview, which
	/// may wait for the main thread.
	pub fn send(&self, staged: &Staged) {
		if staged.0.is_empty() {
			return;
		}
		let Ok(pages) = self.inner.lock().map(|inner| inner.pages.clone()) else { return };
		let gone: Vec<u32> = pages
			.iter()
			.filter(|(_, _, sink)| staged.0.iter().any(|msg| !sink.send(msg.clone())))
			.map(|(_, id, _)| *id)
			.collect();
		if !gone.is_empty()
			&& let Ok(mut inner) = self.inner.lock()
		{
			inner.pages.retain(|(_, id, _)| !gone.contains(id));
		}
	}

	/// Drops a page's sink: its window was destroyed.
	pub fn forget(&self, label: &str) {
		if let Ok(mut inner) = self.inner.lock() {
			inner.pages.retain(|(held, _, _)| held != label);
		}
	}
}
