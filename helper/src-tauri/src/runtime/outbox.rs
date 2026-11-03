//! The outboxes' sending on the loop (docs/plan.md slice 24; contract/feedback-api.yaml, "The client's outbox", which
//! contract/crash-api.yaml's crash reports and contract/telemetry-api.yaml's anonymous counts follow with their own
//! outboxes): each outbox (questpump-core
//! `outbox::Outbox`, under its route's `Rules`) posts what waits one batch at a time, oldest first, in a task off the
//! loop, through the one HTTPS client (`crate::feedback::post`): at the start, right after a new entry (once the batch
//! out finishes), every `OUTBOX_EVERY` while one waits, and `OUTBOX_SOON` after a batch the route slowed, a 429 or a
//! 204 while more waited (each route lets one entry a minute through per client address, so the next would be a 429).
//! An answer the route's rules refuse drops the entry and the next goes at once (the route refuses before it counts);
//! any other answer but 204 keeps it and ends the batch, so a busy route or no connection is asked once a try. What
//! each answer means for the tray's note, the form and the crash page is the owner's (`runtime/feedback.rs`,
//! `runtime/crash.rs`, `runtime/counts.rs`). The log names an entry by its id, never its text.

use std::path::Path;
use std::time::Instant;

use questpump_core::outbox::{Answer, Outbox, Rules, Waiting};
use questpump_core::{crash, feedback, telemetry};
use tokio::sync::mpsc::UnboundedSender;

use super::{Helper, OUTBOX_SOON, now};

/// Which outbox: the feedback's, the crash reports' or the anonymous counts'.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Lane {
	Feedback,
	Crash,
	Counts,
}

impl Lane {
	fn rules(self) -> &'static Rules {
		match self {
			Self::Feedback => &feedback::OUTBOX,
			Self::Crash => &crash::OUTBOX,
			Self::Counts => &telemetry::OUTBOX,
		}
	}

	/// The log's name for it.
	fn name(self) -> &'static str {
		match self {
			Self::Feedback => "feedback",
			Self::Crash => "crash",
			Self::Counts => "counts",
		}
	}
}

/// What a batch task reports to the loop.
pub(crate) enum News {
	/// An entry's post: the route's status, or why no answer came.
	Posted(Lane, String, Result<u16, String>),
	/// The batch is over (every entry went, or one was kept and the rest waits); true when the route slowed it (a 429,
	/// or a 204 with more to go), so the next round goes `OUTBOX_SOON` later.
	Done(Lane, bool),
}

impl News {
	fn lane(&self) -> Lane {
		match self {
			Self::Posted(lane, ..) | Self::Done(lane, _) => *lane,
		}
	}
}

impl Helper {
	/// What an outbox's batch reported, to that outbox's owner.
	pub(super) fn on_posted(&mut self, news: News) {
		match news.lane() {
			Lane::Feedback => self.on_feedback(news),
			Lane::Crash => self.on_crash(news),
			Lane::Counts => self.on_counts(news),
		}
	}
}

/// One outbox and its sending.
pub(crate) struct Sender {
	lane: Lane,
	/// None when the folder could not be made.
	pub(super) outbox: Option<Outbox>,
	/// None when the system's TLS could not be set up: what the outbox keeps waits for a start that has it.
	client: Option<reqwest::Client>,
	/// The ids of the batch being posted.
	pub(super) batch: Option<Vec<String>>,
	/// An entry came while a batch was out: another follows it.
	pub(super) again: bool,
	/// When the round after a batch the route slowed goes.
	pub(super) soon: Option<Instant>,
	news: UnboundedSender<News>,
}

impl Sender {
	/// The lane's outbox in the data folder, the client, and where its batches report.
	pub(super) fn new(dir: &Path, lane: Lane, client: Option<reqwest::Client>, news: UnboundedSender<News>) -> Self {
		let outbox = Outbox::open(dir, lane.rules())
			.inspect_err(|e| log::warn!("{}: no outbox ({e}); nothing is kept to send later", lane.name()))
			.ok();
		Self { lane, outbox, client, batch: None, again: false, soon: None, news }
	}

	/// Whether a batch can go: the system's TLS was set up.
	pub(super) fn can_post(&self) -> bool {
		self.client.is_some()
	}

	/// What the route's answer means for an entry.
	pub(super) fn answer(&self, status: u16) -> Answer {
		self.lane.rules().answer(status)
	}

	/// What waits in the outbox, oldest first, with what it dropped or skipped logged; nothing without an outbox, None
	/// when it could not be listed (its entries wait for the next try).
	pub(super) fn waiting(&mut self) -> Option<Waiting> {
		let name = self.lane.name();
		let Some(outbox) = self.outbox.as_mut() else { return Some(Waiting::default()) };
		match outbox.waiting(now().unix) {
			Ok(waiting) => {
				for note in &waiting.notes {
					log::warn!("{name}: {note}");
				}
				Some(waiting)
			}
			Err(e) => {
				log::warn!("{name}: the outbox could not be listed ({e}): what it holds waits for the next try");
				None
			}
		}
	}

	/// Posts `entries` in order in a task off the loop, which reports each answer and the batch's end; nothing without
	/// a client.
	pub(super) fn post(&mut self, entries: Vec<(String, Vec<u8>)>) {
		let Some(client) = self.client.clone() else { return };
		self.batch = Some(entries.iter().map(|(id, _)| id.clone()).collect());
		let (lane, news) = (self.lane, self.news.clone());
		let files: Vec<Option<std::path::PathBuf>> =
			entries.iter().map(|(id, _)| self.outbox.as_ref().map(|o| o.file(id))).collect();
		tauri::async_runtime::spawn(async move {
			let (count, mut slowed) = (entries.len(), false);
			for (i, ((id, body), file)) in entries.into_iter().zip(files).enumerate() {
				let posted = crate::feedback::post(&client, lane.rules().route, body).await;
				let answer = posted.as_ref().ok().map(|&status| lane.rules().answer(status));
				// an entry the route took or refused leaves the outbox here at once, so it never goes twice though the
				// loop never hears it (its start crashed while the crash sweep's batch was out); the loop's `remove`
				// tries again a delete that did not take
				if let (Some(Answer::Sent | Answer::Refused), Some(file)) = (answer, &file) {
					let _ = std::fs::remove_file(file);
				}
				if news.send(News::Posted(lane, id, posted)).is_err() {
					return; // the loop has gone: the helper quits
				}
				match answer {
					// the route refuses before it counts an entry against its limits: the next goes now
					Some(Answer::Refused) => {}
					Some(Answer::Sent) if i + 1 == count => {}
					// one entry a minute per address: the next would be a 429
					Some(Answer::Sent | Answer::Busy) => {
						slowed = true;
						break;
					}
					// no connection, the route down: the rest waits for the next try
					Some(Answer::Later) | None => break,
				}
			}
			let _ = news.send(News::Done(lane, slowed));
		});
	}

	/// The batch out is over: its ids, and whether another follows at once (an entry came meanwhile and the route did
	/// not slow it; a slowed batch's round `OUTBOX_SOON` on carries what came).
	pub(super) fn ended(&mut self, slowed: bool) -> (Vec<String>, bool) {
		self.soon = slowed.then(|| Instant::now() + OUTBOX_SOON);
		let again = std::mem::take(&mut self.again) && !slowed;
		(self.batch.take().unwrap_or_default(), again)
	}

	/// The round `OUTBOX_SOON` after a batch the route slowed is due.
	pub(super) fn due(&self) -> bool {
		self.soon.is_some_and(|at| Instant::now() >= at)
	}

	/// The round `OUTBOX_SOON` after a batch the route slowed is still to come: a post before it would be a 429.
	pub(super) fn held(&self) -> bool {
		self.soon.is_some_and(|at| Instant::now() < at)
	}

	/// An entry that went or that the route refused leaves the outbox (one whose file stays is never posted again).
	pub(super) fn remove(&mut self, id: &str) {
		if let Some(Err(e)) = self.outbox.as_mut().map(|o| o.remove(id)) {
			log::warn!("{}: {id} could not be deleted from the outbox ({e}): it is not sent again", self.lane.name());
		}
	}
}
