//! The feedback button's outbox on the loop (docs/plan.md slice 24): a message the game handed over on the strip and
//! one from the helper's window go into the outbox (questpump-core `outbox::Outbox`) first, so none is lost to a quit
//! or a missing connection, and a game message the outbox took in the last day is never taken again; one the outbox
//! cannot keep (the disk full, the folder not writable) is held in memory and gets one try, and the note or the form
//! says when it is lost. The outbox keeps a message `feedback::KEEP_DAYS` (30) days and `feedback::KEEP_MAX` (20)
//! messages at most, the oldest dropped first. Its batches go as every outbox's do (`runtime/outbox.rs`: at the start,
//! after a new message, every `OUTBOX_EVERY` while one waits, `OUTBOX_SOON` after a batch the route slowed). 204
//! deletes a message and the note (the tray's and the window's) says "Feedback sent"; 400, 401 and 413 drop it with a
//! log line and the note says "Feedback not accepted", as for a game message that did not check out; anything else
//! keeps it. After a batch the note says a message waits only while one does. The note is a state the tray and the page
//! word themselves (`ui::Note`). The log names a message by its id, never its text.

use std::collections::HashMap;
use std::path::Path;

use questpump_core::feedback::{self, Diagnostics, Message};
use questpump_core::outbox::{Answer, Kept};
use tauri::Manager as _;
use tokio::sync::mpsc::UnboundedSender;

use super::outbox::{Lane, News, Sender};
use super::{Helper, OUTBOX_EVERY, now};
use crate::lan::now_ms;
use crate::ui::{self, FeedbackReply, FeedbackState, Note, Shared};

/// The outbox and its sending.
pub(crate) struct Feedback {
	/// The outbox (none when its folder could not be made: every message is then held unkept) and its batches.
	sender: Sender,
	/// Messages the outbox could not keep, by id with their body: posted at the head of the next batch (again in the
	/// round after a 429), and lost when that try does not go.
	unkept: Vec<(String, Vec<u8>)>,
	/// The unkept messages the batch out carries.
	unkept_out: HashMap<String, Vec<u8>>,
	/// The window's sends waiting for their outcome, by id.
	waiters: HashMap<String, FeedbackReply>,
	/// How the latest message that left the outbox went, the note while none waits; none before the first.
	settled: Option<Note>,
	/// How many messages wait, as the note last said.
	waiting: usize,
}

impl Feedback {
	/// The outbox in the data folder, the client, and where the batches report.
	pub(super) fn new(dir: &Path, client: Option<reqwest::Client>, news: UnboundedSender<News>) -> Self {
		Self {
			sender: Sender::new(dir, Lane::Feedback, client, news),
			unkept: Vec::new(),
			unkept_out: HashMap::new(),
			waiters: HashMap::new(),
			settled: None,
			waiting: 0,
		}
	}
}

impl Helper {
	/// A message the game handed over: kept, then sent, once.
	pub(super) fn game_feedback(&mut self, message: &Message) {
		// copy-lint: not shown (a log line)
		if self.keep_feedback(message, "the game") {
			self.send_feedback();
		}
	}

	/// A message from the game's window that did not decode or check out (`Effects::feedback_dropped`, its log line
	/// written): dropped as the route's 400 drops one, and the note says so as it does for a 400.
	pub(super) fn game_feedback_dropped(&mut self) {
		self.feedback.settled = Some(Note::Refused { at: now_ms() });
		self.note_feedback(self.feedback.waiting);
	}

	/// The window's Send: the message with the diagnostics the form shows when its tick is on, kept, then sent; the form
	/// hears how it went.
	pub(super) fn window_feedback(&mut self, text: &str, diagnostics: bool, done: FeedbackReply) {
		let shown = self.app.state::<Shared>().view.lock().map(|v| v.feedback.clone()).unwrap_or_default();
		let diagnostics = diagnostics.then(|| Diagnostics::new(&shown.os, &shown.capture));
		let version = env!("CARGO_PKG_VERSION");
		let message = match Message::helper(crate::lan::now_ms(), text, version, diagnostics.as_ref()) {
			Ok(message) => message,
			Err(e) => {
				log::warn!("feedback: the window's message: {e}");
				let _ = done.send(Ok(FeedbackState::Refused));
				return;
			}
		};
		self.keep_feedback(&message, "the window"); // copy-lint: not shown (a log line)
		self.feedback.waiters.insert(message.id, done);
		self.send_feedback();
	}

	/// Keeps a message in the outbox, or, when the outbox cannot keep it, holds it for one try; false for a game
	/// message taken already, which goes once.
	fn keep_feedback(&mut self, message: &Message, from: &str) -> bool {
		let id = &message.id;
		let once = feedback::from_game(id);
		let kept = match self.feedback.sender.outbox.as_mut() {
			Some(outbox) => outbox.keep(id, message.body(), once, now().unix).map_err(|e| e.to_string()),
			None => Err("no outbox".to_owned()),
		};
		match kept {
			Ok(Kept::New) => log::info!("feedback: {id} from {from}, kept to send"),
			Ok(Kept::Unremembered(e)) => {
				log::warn!("feedback: {id} from {from}, kept to send; its name could not be remembered ({e})");
			}
			Ok(Kept::Again) => {
				log::info!("feedback: {id} from {from} again: it goes once");
				return false;
			}
			Err(e) => {
				let f = &mut self.feedback;
				if f.unkept.iter().any(|(held, _)| held == id) || f.unkept_out.contains_key(id) {
					return false;
				}
				log::warn!("feedback: {id} from {from} could not be kept ({e}): it gets one try, unkept");
				f.unkept.push((id.clone(), message.body().to_vec()));
			}
		}
		true
	}

	/// Posts what waits, unkept messages first, unless a batch is out (then another follows it).
	pub(super) fn send_feedback(&mut self) {
		self.feedback.sender.soon = None;
		if self.feedback.sender.batch.is_some() {
			self.feedback.sender.again = true;
			return;
		}
		let kept = self.feedback_waiting().unwrap_or_default();
		let unkept = std::mem::take(&mut self.feedback.unkept);
		if !self.feedback.sender.can_post() {
			// no TLS on this system: what the outbox keeps waits for a start that has it, and the unkept are lost
			for (id, _) in unkept {
				self.lose_feedback(&id, "no HTTPS client"); // copy-lint: not shown (a log line)
			}
			self.answer_unsent(None);
			self.note_feedback(kept.len());
			return;
		}
		if kept.is_empty() && unkept.is_empty() {
			self.answer_unsent(None);
			// a message the note said waits left the outbox some other way (its file deleted)
			if self.feedback.waiting > 0 {
				self.note_feedback(0);
			}
			return;
		}
		self.feedback.unkept_out = unkept.iter().cloned().collect();
		let messages: Vec<(String, Vec<u8>)> = unkept.into_iter().chain(kept).collect();
		self.feedback.sender.post(messages);
		// a form whose message this batch does not carry (its entry did not read this time) hears it waits
		self.answer_unsent(None);
	}

	/// The round `OUTBOX_SOON` after a batch the route slowed, once its time came (checked on the 2-s game tick).
	pub(super) fn feedback_due(&mut self) {
		if self.feedback.sender.due() {
			self.send_feedback();
		}
	}

	/// What the batch out reported: a message's answer (the outbox, the note and a waiting form follow it), or its end.
	pub(super) fn on_feedback(&mut self, news: News) {
		match news {
			News::Posted(_, id, posted) => {
				let unkept = self.feedback.unkept_out.remove(&id);
				let state = match posted.map(|status| (status, self.feedback.sender.answer(status))) {
					Ok((_, Answer::Sent)) => {
						log::info!("feedback: {id} sent");
						if unkept.is_none() {
							self.feedback.sender.remove(&id);
						}
						self.feedback.settled = Some(Note::Sent { at: now_ms() });
						Some(FeedbackState::Sent)
					}
					Ok((status, Answer::Refused)) => {
						log::warn!("feedback: {id} refused by the route ({status}): dropped");
						if unkept.is_none() {
							self.feedback.sender.remove(&id);
						}
						self.feedback.settled = Some(Note::Refused { at: now_ms() });
						Some(FeedbackState::Refused)
					}
					Ok((status, Answer::Busy)) => {
						log::info!("feedback: {id} not sent yet (the route answered {status}); it goes in a minute");
						// an unkept one keeps its try for the round a minute on, its form waiting for it
						unkept.map_or(Some(FeedbackState::Queued), |body| {
							self.feedback.unkept.push((id.clone(), body));
							None
						})
					}
					Ok((status, Answer::Later)) if unkept.is_some() => {
						// copy-lint: not shown (a log line)
						self.lose_feedback(&id, &format!("the route answered {status}"));
						None
					}
					Err(e) if unkept.is_some() => {
						self.lose_feedback(&id, &e);
						None
					}
					Ok((status, Answer::Later)) => {
						log::info!("feedback: {id} not sent yet (the route answered {status}); it goes later");
						Some(FeedbackState::Queued)
					}
					Err(e) => {
						log::info!("feedback: {id} not sent yet ({e}); it goes later");
						Some(FeedbackState::Queued)
					}
				};
				if let Some(state) = state {
					self.answer_feedback(&id, state);
				}
			}
			News::Done(_, slowed) => {
				let (batch, again) = self.feedback.sender.ended(slowed);
				// unkept messages the batch did not reach: kept for the round a minute on, else lost with the rest
				for (id, body) in std::mem::take(&mut self.feedback.unkept_out) {
					if slowed {
						self.feedback.unkept.push((id, body));
					} else {
						self.lose_feedback(&id, "the batch ended before it"); // copy-lint: not shown (a log line)
					}
				}
				// the round a minute on carries every message, the ones that came meanwhile too, so their forms hear
				// now; otherwise the ones the batch did not reach wait for the next try, and a batch for the ones that
				// came meanwhile follows
				self.answer_unsent(again.then_some(batch.as_slice()));
				let waiting = match self.feedback_waiting() {
					Some(kept) => kept.len() + self.feedback.unkept.len(),
					None => self.feedback.waiting,
				};
				self.note_feedback(waiting);
				if again {
					self.send_feedback();
				}
			}
		}
	}

	/// The outbox's messages that wait, oldest first, with what it dropped or skipped logged (the form of a message it
	/// dropped hears it is lost); empty without an outbox, None when the outbox could not be listed.
	fn feedback_waiting(&mut self) -> Option<Vec<(String, Vec<u8>)>> {
		let waiting = self.feedback.sender.waiting()?;
		// only a clock set back makes a message just sent the oldest
		for id in &waiting.dropped {
			self.answer_feedback(id, FeedbackState::Unsaved);
		}
		Some(waiting.messages)
	}

	/// Answers that its message waits in the outbox to each form whose message no batch carries and none holds unkept;
	/// with `only`, just the forms of those ids.
	fn answer_unsent(&mut self, only: Option<&[String]>) {
		let f = &self.feedback;
		let carried = |id: &String| {
			f.sender.batch.as_ref().is_some_and(|b| b.contains(id)) || f.unkept.iter().any(|(held, _)| held == id)
		};
		let unsent: Vec<String> =
			f.waiters.keys().filter(|&id| !carried(id) && only.is_none_or(|only| only.contains(id))).cloned().collect();
		for id in unsent {
			self.answer_feedback(&id, FeedbackState::Queued);
		}
	}

	/// The form that waits for this message hears how it went.
	fn answer_feedback(&mut self, id: &str, state: FeedbackState) {
		if let Some(done) = self.feedback.waiters.remove(id) {
			let _ = done.send(Ok(state));
		}
	}

	/// An unkept message whose one try did not go: lost, and the note (and its form) says so.
	fn lose_feedback(&mut self, id: &str, why: &str) {
		log::warn!("feedback: {id} was not kept and did not go ({why}): it is lost");
		self.feedback.settled = Some(Note::Lost { at: now_ms(), game: feedback::from_game(id) });
		self.answer_feedback(id, FeedbackState::Unsaved);
	}

	/// The note, the tray's and the form's, and the form's count of the messages that wait: that one waits while one
	/// does (and when the next try comes), else how the latest one that left the outbox went (no note before the
	/// first), so it never says one waits when none does.
	fn note_feedback(&mut self, waiting: usize) {
		self.feedback.waiting = waiting;
		let note = if waiting == 0 {
			self.feedback.settled.clone()
		} else if self.feedback.sender.soon.is_some() {
			Some(Note::Soon)
		} else {
			Some(Note::Later { minutes: OUTBOX_EVERY.as_secs() / 60 })
		};
		ui::update(&self.app, |v| {
			v.feedback.note = note;
			v.feedback.waiting = waiting;
		});
	}
}
