//! The page's state as topics (docs/plan.md slice 35, "The UI stack"): each topic one whole value, replaced on every
//! change and pushed to every webview that subscribed through a `tauri::ipc::Channel` (an app command's channel needs
//! no capability file). A message is `{topic, rev, data}`; `rev` counts every change of every topic from 1, so a page
//! that holds a topic's value at one rev takes a message only when it is newer, whichever of the subscription's answer
//! (`Boot`, every topic's value) and a message sent meanwhile arrives first. A topic whose new value equals the one
//! held is not sent again. A state that publishes several topics from one change stages them under its own lock and
//! sends them after it (`stage`, `send`), so two changes made on two threads get their revs in the order they were
//! made. Each webview holds one channel (a page that loads again replaces its own), and a channel whose webview is gone
//! is dropped at the first send that fails or when its window is destroyed.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use serde::Serialize;
use serde_json::Value;
use tauri::ipc::Channel;

/// One topic's change, as the page's `lib/bridge.js` receives it.
#[derive(Serialize, Debug, Clone)]
pub(crate) struct Msg {
	topic: String,
	rev: u64,
	data: Value,
}

/// A topic's value at its rev.
#[derive(Serialize, Debug, Clone)]
pub(crate) struct Held {
	rev: u64,
	data: Value,
}

/// `desk_subscribe`'s answer: every topic's value, so the first paint needs no second round trip.
#[derive(Serialize, Debug)]
pub(crate) struct Boot {
	topics: BTreeMap<String, Held>,
}

#[derive(Default)]
struct Inner {
	topics: BTreeMap<String, Held>,
	rev: u64,
	/// Each subscribed webview's channel, by its label.
	channels: Vec<(String, Channel<Msg>)>,
}

/// Topics staged under the lock of the state they come from, sent once it is released (`Publisher::stage`,
/// `Publisher::send`).
#[derive(Default)]
pub(crate) struct Staged(Vec<Msg>);

/// The topics and their subscribers; a cheap handle, shared by the commands, the loop and the window's events.
#[derive(Clone, Default)]
pub(crate) struct Publisher {
	inner: Arc<Mutex<Inner>>,
}

impl Publisher {
	/// Subscribes a webview (replacing a channel it held before) and answers every topic's value, both under one lock,
	/// so every later change comes as a message with a higher rev.
	pub(crate) fn subscribe(&self, label: &str, channel: Channel<Msg>) -> Boot {
		let Ok(mut inner) = self.inner.lock() else { return Boot { topics: BTreeMap::new() } };
		inner.channels.retain(|(held, _)| held != label);
		inner.channels.push((label.to_owned(), channel));
		Boot { topics: inner.topics.clone() }
	}

	/// Replaces a topic's value and sends it to every subscriber, unless it equals the value held.
	pub(crate) fn publish(&self, topic: &str, data: &impl Serialize) {
		let mut staged = Staged::default();
		self.stage(&mut staged, topic, data);
		self.send(&staged);
	}

	/// Replaces a topic's value and gives it its rev, unless it equals the value held, for `send` to send. Called under
	/// the lock of the state the value comes from, so the revs follow the order of that state's changes: a value staged
	/// later has the higher rev whichever send arrives first, and the page keeps it.
	pub(crate) fn stage(&self, staged: &mut Staged, topic: &str, data: &impl Serialize) {
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
	pub(crate) fn send(&self, staged: &Staged) {
		if staged.0.is_empty() {
			return;
		}
		let Ok(channels) = self.inner.lock().map(|inner| inner.channels.clone()) else { return };
		let gone: Vec<u32> = channels
			.iter()
			.filter(|(_, channel)| staged.0.iter().any(|msg| channel.send(msg.clone()).is_err()))
			.map(|(_, channel)| channel.id())
			.collect();
		if !gone.is_empty()
			&& let Ok(mut inner) = self.inner.lock()
		{
			inner.channels.retain(|(_, channel)| !gone.contains(&channel.id()));
		}
	}

	/// Drops a webview's channel: its window was destroyed.
	pub(crate) fn forget(&self, label: &str) {
		if let Ok(mut inner) = self.inner.lock() {
			inner.channels.retain(|(held, _)| held != label);
		}
	}
}
