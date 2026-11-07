//! The crash reports (contract/crash-api.yaml, "The desktop app"; `crate::crash` captures and finds them,
//! questpump-core's `crash.rs` builds them). The Menu's switch Send crash reports is the app's own setting
//! (`settings.json` `crash.send`, with the answer's moment and the words' version, `crash.at` and `crash.copy`) with
//! three states: on, off, and not answered yet, which reads as on outside an opt-in region and as off inside one (the
//! EEA, India and Canada, guessed from the system's region and time zone at each start until the player answers;
//! `crate::crash::opt_in`). First in the start (`Crash::sweep`, from `runtime::sweep`, which `lib.rs`'s
//! `run()` calls right after the panic hook, before the window, the engine or any device, so an app that crashes in its
//! start still sends), with reports waiting (the panic hook's files and the system's reports newer than the last one
//! handled, `settings.json` `crash.handled`): on, they go into the outbox and out at once, and no page opens; off by
//! the player's answer, they are deleted and so is every one in the outbox; not answered, they wait in memory and the
//! next main window built opens on the page `desk.crash` (`Desk::crash_page`, once a run: at a player's launch that is
//! the start's own window, after a start at login or an update the window the player opens next), whose Send keeps them
//! in the outbox marked as the player's (`ASKED`; the switch still not answered), Always send turns the switch on and
//! does the same, and Don't send deletes them (the switch still not answered); closed any other way, the page comes
//! again at the next start. Turning the switch off deletes the reports that wait and empties the outbox; turning it on
//! sends what waits. The outbox (`crash::OUTBOX`: 5 reports, 30 days from each one's `at`) goes out as the feedback's
//! does (`runtime/outbox.rs`), but a round sends a report only while the switch is on or when the player's Send put it
//! there: one kept while the switch was on waits, unsent, while the switch reads not answered (a player who moved into
//! an opt-in region). 204 deletes a report, 400 and 413 drop it with a log line naming the code, anything else keeps
//! it. The `crash` topic carries the switch, the waiting reports' texts exactly as they would go, and how the page's
//! Send went. The page's intents: `{"type":"crash.switch","on":bool}`, `{"type":"crash.send"}`,
//! `{"type":"crash.always"}`, `{"type":"crash.skip"}`. The log names a report by its id, never its text.

use std::path::Path;
use std::sync::atomic::Ordering;

use questpump_core::crash::Handled;
use questpump_core::outbox::Answer;
use serde::Serialize;
use serde_json::Value;
use tauri::Manager as _;
use tokio::sync::mpsc::UnboundedSender;

use super::outbox::{Lane, News, Sender};
use super::{Helper, now};
use crate::crash::{Found, Source};
use crate::desk::{Desk, Intent};
use crate::settings::{CrashSettings, Region};

/// The topic the Menu's switch and the page follow.
pub(crate) const TOPIC: &str = "crash";

/// The start of the outbox's id of a report the player's answer on the page put there (Send, Always send, or the
/// switch turned on while the page asked): it goes whatever the switch reads.
const ASKED: &str = "asked-";

/// How the page's Send went.
#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum Outcome {
	/// The reports are on their way.
	Sending,
	/// The route took every one.
	Sent,
	/// One waits in the outbox (no connection, the route busy or down) and goes by itself.
	Kept,
	/// The route refused one, or it could not be kept: it never goes.
	Refused,
}

/// The `crash` topic: the switch as the Menu shows it (not answered shows its default), the texts of the reports that
/// wait for the player's answer exactly as they would go, and how the page's last Send went.
#[derive(Serialize)]
struct Page<'a> {
	on: bool,
	reports: Vec<&'a str>,
	outcome: Option<Outcome>,
}

/// The crash reports' outbox and what waits for the player.
pub(crate) struct Crash {
	sender: Sender,
	/// The reports found at the start that wait for the player's answer, and the newest of the system's reports looked
	/// at with them (the mark once they are answered).
	asking: Vec<Found>,
	asking_newest: Option<Handled>,
	/// The ids the page's Send put in the outbox that no answer settled yet, and whether one of them was kept or
	/// refused so far.
	pending: Vec<String>,
	kept: bool,
	refused: bool,
	outcome: Option<Outcome>,
	/// How many reports the outbox's last round held back while the switch is not on (the log says it when it changes).
	held: usize,
}

impl Crash {
	/// The outbox in the data folder, the client, and where its batches report.
	pub(super) fn new(dir: &Path, client: Option<reqwest::Client>, news: UnboundedSender<News>) -> Self {
		Self {
			sender: Sender::new(dir, Lane::Crash, client, news),
			asking: Vec::new(),
			asking_newest: None,
			pending: Vec::new(),
			kept: false,
			refused: false,
			outcome: None,
			held: 0,
		}
	}

	/// First in the start, before the window, the engine or any device (`runtime::sweep`): the reports that wait go
	/// into the outbox and out at once while the switch reads on (`region`: where the computer is, read while it is not
	/// answered), are deleted with every one in the outbox when the player's answer is off, or wait for the page; the
	/// marks they reach go into `marks`, which `runtime::sweep` saves at once when they moved. True when a report
	/// waited: the run before crashed.
	pub(super) fn sweep(&mut self, dir: &Path, marks: &mut CrashSettings, region: Region) -> bool {
		let answer = marks.choice.send;
		match answer {
			Some(on) => log::info!("crash: Send crash reports {}, the player's answer", if on { "on" } else { "off" }),
			None if region == Region::OptIn => log::info!("crash: Send crash reports not answered, an opt-in region"),
			None => log::info!("crash: Send crash reports not answered, on outside the opt-in regions"),
		}
		let waiting = crate::crash::found(dir, marks.handled.as_ref(), marks.handled_panic, now().unix);
		let crashed = !waiting.reports.is_empty();
		if answer == Some(false) {
			drop_reports(&waiting.reports, waiting.newest, "the switch is off", marks); // copy-lint: not shown
			self.clear();
		} else if waiting.reports.is_empty() {
			handled(marks, waiting.newest, None);
		} else if marks.choice.reads(region) {
			self.keep(waiting.reports, waiting.newest, false, marks);
			self.round(true);
		} else {
			log::info!("crash: {} reports wait for the player's answer", waiting.reports.len());
			self.asking = waiting.reports;
			self.asking_newest = waiting.newest;
		}
		crashed
	}

	/// Reports into the outbox, each panic file deleted once its report is kept, `asked` ones marked as the player's
	/// (`ASKED`), the marks they reach in `marks`: their ids in the outbox. A report the route would refuse is dropped;
	/// one the outbox cannot keep is lost, but a panic's, whose file waits for the next start.
	fn keep(
		&mut self,
		reports: Vec<Found>,
		newest: Option<Handled>,
		asked: bool,
		marks: &mut CrashSettings,
	) -> Vec<String> {
		let panic = newest_panic(&reports);
		let mut ids = Vec::new();
		for found in reports {
			let id = &found.id;
			let body = match found.report.body() {
				Ok(body) => body,
				Err(e) => {
					log::warn!("crash: {id} breaks the route's rules ({e}): dropped");
					crash_forget(&found);
					continue;
				}
			};
			let kept_as = if asked { format!("{ASKED}{id}") } else { id.clone() };
			let kept = match self.sender.outbox.as_mut() {
				Some(outbox) => outbox.keep(&kept_as, &body, false, now().unix).map_err(|e| e.to_string()),
				None => Err("no outbox".to_owned()),
			};
			match (kept, &found.source) {
				(Ok(_), _) => {
					log::info!("crash: {kept_as} kept to send");
					crash_forget(&found);
					ids.push(kept_as);
				}
				(Err(e), Source::Panic(_)) => {
					log::warn!("crash: {id} could not be kept ({e}): it waits for the next start");
				}
				(Err(e), Source::System) => log::warn!("crash: {id} could not be kept ({e}): it is lost"),
			}
		}
		handled(marks, newest, panic);
		ids
	}

	/// The switch is off: every report in the outbox is deleted unsent, and the page's Send waits for none.
	fn clear(&mut self) {
		if let Some(outbox) = self.sender.outbox.as_mut() {
			match outbox.clear() {
				Ok((deleted, notes)) => {
					for id in deleted {
						log::info!("crash: {id} deleted from the outbox: the switch is off");
					}
					for note in notes {
						log::warn!("crash: {note}");
					}
				}
				Err(e) => log::warn!("crash: the outbox could not be emptied ({e})"),
			}
		}
		self.pending.clear();
		self.outcome = None;
	}

	/// A round of the outbox: what waits goes, oldest first (while the switch is not `on`, only what the player's Send
	/// put there), while the system's TLS was set up. The ids it dropped unsent (over the outbox's days or count) and
	/// whether a batch went; none when the outbox could not be listed.
	fn round(&mut self, on: bool) -> Option<(Vec<String>, bool)> {
		let waiting = self.sender.waiting()?;
		let (messages, held): (Vec<_>, Vec<_>) =
			waiting.messages.into_iter().partition(|(id, _)| on || id.starts_with(ASKED));
		if std::mem::replace(&mut self.held, held.len()) != held.len() && !held.is_empty() {
			log::info!("crash: {} reports wait unsent: Send crash reports is not on", held.len());
		}
		let posted = !messages.is_empty() && self.sender.can_post();
		if posted {
			self.sender.post(messages);
		}
		Some((waiting.dropped, posted))
	}
}

/// Reports turned down: each panic file deleted, the system's marked handled in `marks`.
fn drop_reports(reports: &[Found], newest: Option<Handled>, why: &str, marks: &mut CrashSettings) {
	for found in reports {
		log::info!("crash: {} deleted unsent: {why}", found.id);
		crash_forget(found);
	}
	handled(marks, newest, newest_panic(reports));
}

/// Reports handled (kept or turned down): the newest system report looked at moves the mark, so none is offered twice,
/// and the newest panic's moment (`newest_panic`) is kept, so the system's report of the abort it ended in, found at a
/// later start, never goes either. True when a mark moved.
fn handled(marks: &mut CrashSettings, newest: Option<Handled>, panic: Option<i64>) -> bool {
	let mut moved = false;
	if let Some(newest) = newest
		&& marks.handled.as_ref().is_none_or(|h| *h < newest)
	{
		marks.handled = Some(newest);
		moved = true;
	}
	if let Some(at) = panic
		&& marks.handled_panic.is_none_or(|h| h < at)
	{
		marks.handled_panic = Some(at);
		moved = true;
	}
	moved
}

impl Helper {
	/// The switch as it reads: the player's answer, else on outside an opt-in region.
	pub(crate) fn crash_on(&self) -> bool {
		self.desk.settings.crash.choice.reads(self.region)
	}

	/// In the start, once a window can be asked for (the sweep ran first in `run()` and saved the marks it moved):
	/// reports that wait for the player's answer take the next main window to their page.
	pub(super) fn crash_start(&mut self) {
		if !self.crash.asking.is_empty() {
			self.app.state::<Desk>().crash_page.store(true, Ordering::Relaxed);
		}
		self.crash_publish();
	}

	/// The page's and the Menu's intents.
	pub(crate) fn crash_intent(&mut self, intent: &Intent) -> Result<(), String> {
		match intent.kind() {
			"crash.switch" => {
				let on =
					intent.arg("on").and_then(Value::as_bool).ok_or_else(|| format!("{} takes on", intent.kind()))?;
				self.crash_answer(on);
			}
			"crash.send" => self.crash_send_asked(),
			"crash.always" => {
				self.desk.settings.crash.choice.answer(true, now().unix);
				self.desk.save_settings();
				log::info!("crash: Send crash reports on, the player's answer (Always send)");
				self.privacy_settled();
				self.crash_send_asked();
			}
			"crash.skip" => {
				let (asking, newest) = self.crash_take_asked();
				self.crash_drop(&asking, newest, "the player said Don't send"); // copy-lint: not shown (a log line)
			}
			_ => return Err(intent.unknown()),
		}
		self.crash_publish();
		Ok(())
	}

	/// The player's answer to Send crash reports (the Menu's switch, the privacy page's answer), kept with its moment and
	/// the words' version: on sends what waits, off deletes it and empties the outbox at once.
	pub(crate) fn crash_answer(&mut self, on: bool) {
		self.desk.settings.crash.choice.answer(on, now().unix);
		self.desk.save_settings();
		log::info!("crash: Send crash reports {}, the player's answer", if on { "on" } else { "off" });
		self.privacy_settled();
		if on {
			self.crash_send_asked();
		} else {
			self.crash_off();
		}
		self.crash_publish();
	}

	/// The reports that waited for the player's answer, and the mark they reach; the page opens no more this run.
	fn crash_take_asked(&mut self) -> (Vec<Found>, Option<Handled>) {
		self.app.state::<Desk>().crash_page.store(false, Ordering::Relaxed);
		(std::mem::take(&mut self.crash.asking), self.crash.asking_newest.take())
	}

	/// Send, Always send or the switch turned on: the reports that waited go into the outbox and out, and the page
	/// follows how they went; with none waiting for the page, the outbox's reports kept while the switch was on go now.
	fn crash_send_asked(&mut self) {
		let (asking, newest) = self.crash_take_asked();
		if asking.is_empty() {
			if handled(&mut self.desk.settings.crash, newest, None) {
				self.desk.save_settings();
			}
			self.send_crashes();
			return;
		}
		let ids = self.crash_keep(asking, newest, true);
		self.crash.kept = false;
		self.crash.refused = false;
		self.crash.outcome = Some(if ids.is_empty() { Outcome::Refused } else { Outcome::Sending });
		self.crash.pending = ids;
		self.send_crashes();
	}

	/// The switch turned off: the reports that wait are deleted, and so is every one in the outbox.
	fn crash_off(&mut self) {
		let (asking, newest) = self.crash_take_asked();
		self.crash_drop(&asking, newest, "the switch is off"); // copy-lint: not shown (a log line)
		self.crash.clear();
	}

	/// Reports into the outbox (`Crash::keep`), the settings saved when a mark moved.
	fn crash_keep(&mut self, reports: Vec<Found>, newest: Option<Handled>, asked: bool) -> Vec<String> {
		let before = self.desk.settings.crash.clone();
		let ids = self.crash.keep(reports, newest, asked, &mut self.desk.settings.crash);
		if self.desk.settings.crash != before {
			self.desk.save_settings();
		}
		ids
	}

	/// Reports turned down (`drop_reports`), the settings saved when a mark moved.
	fn crash_drop(&mut self, reports: &[Found], newest: Option<Handled>, why: &str) {
		let before = self.desk.settings.crash.clone();
		drop_reports(reports, newest, why, &mut self.desk.settings.crash);
		if self.desk.settings.crash != before {
			self.desk.save_settings();
		}
	}

	/// Posts what waits in the outbox, unless a batch is out (then another follows it).
	pub(super) fn send_crashes(&mut self) {
		self.crash.sender.soon = None;
		if self.crash.sender.batch.is_some() {
			self.crash.sender.again = true;
			return;
		}
		// while the switch is not on, only what the player's Send put there goes
		let Some((dropped, posted)) = self.crash.round(self.crash_on()) else {
			self.crash_kept_all();
			return;
		};
		for id in &dropped {
			self.crash_settle(id, Answer::Refused);
		}
		self.crash_conclude();
		if !posted {
			// nothing left that the page waits for goes this round
			self.crash_kept_all();
		}
	}

	/// The round `OUTBOX_SOON` after a batch the route slowed, once its time came (checked on the 2-s game tick).
	pub(super) fn crash_due(&mut self) {
		if self.crash.sender.due() {
			self.send_crashes();
		}
	}

	/// What the batch out reported: a report's answer (the outbox and the page follow it), or its end.
	pub(super) fn on_crash(&mut self, news: News) {
		match news {
			News::Posted(_, id, posted) => {
				let answer = match posted.map(|status| (status, self.crash.sender.answer(status))) {
					Ok((_, Answer::Sent)) => {
						log::info!("crash: {id} sent");
						self.crash.sender.remove(&id);
						Answer::Sent
					}
					Ok((status, Answer::Refused)) => {
						log::warn!("crash: {id} refused by the route ({status}): dropped");
						self.crash.sender.remove(&id);
						Answer::Refused
					}
					Ok((status, answer)) => {
						log::info!("crash: {id} not sent yet (the route answered {status}); it goes later");
						answer
					}
					Err(e) => {
						log::info!("crash: {id} not sent yet ({e}); it goes later");
						Answer::Later
					}
				};
				self.crash_settle(&id, answer);
			}
			News::Done(_, slowed) => {
				let (_, again) = self.crash.sender.ended(slowed);
				// one kept waits for a later round, as does one this batch never reached; one the round a minute on
				// or the batch that follows carries is still on its way
				if !self.crash_conclude() && (self.crash.kept || !(slowed || again)) {
					self.crash_kept_all();
				}
				if again {
					self.send_crashes();
				}
			}
		}
		self.crash_publish();
	}

	/// A report the page's Send carried was answered: gone (sent or refused) or kept for later.
	fn crash_settle(&mut self, id: &str, answer: Answer) {
		let Some(at) = self.crash.pending.iter().position(|p| p == id) else { return };
		match answer {
			Answer::Sent => {
				self.crash.pending.remove(at);
			}
			Answer::Refused => {
				self.crash.pending.remove(at);
				self.crash.refused = true;
			}
			Answer::Busy | Answer::Later => self.crash.kept = true,
		}
	}

	/// Whether the page's Send is over (or none is on its way): once no report it carried waits, it says Sent, or
	/// Refused when one was refused.
	fn crash_conclude(&mut self) -> bool {
		if self.crash.outcome != Some(Outcome::Sending) {
			return true;
		}
		if !self.crash.pending.is_empty() {
			return false;
		}
		self.crash.outcome = Some(if self.crash.refused { Outcome::Refused } else { Outcome::Sent });
		true
	}

	/// The page's Send is over though a report it carried has not gone: it waits in the outbox and goes by itself.
	fn crash_kept_all(&mut self) {
		if self.crash.outcome == Some(Outcome::Sending) && !self.crash.pending.is_empty() {
			self.crash.pending.clear();
			self.crash.outcome = Some(Outcome::Kept);
			self.crash_publish();
		}
	}

	/// The `crash` topic as it stands.
	pub(crate) fn crash_publish(&self) {
		let page = Page {
			on: self.crash_on(),
			reports: self.crash.asking.iter().map(|f| f.report.report.as_str()).collect(),
			outcome: self.crash.outcome,
		};
		self.app.state::<Desk>().publisher.publish(TOPIC, &page);
	}
}

/// The moment of the newest panic among reports (unix seconds).
fn newest_panic(reports: &[Found]) -> Option<i64> {
	reports.iter().filter(|f| matches!(f.source, Source::Panic(_))).map(|f| f.report.at).max()
}

/// A report kept or turned down: the panic hook's file goes; the system's own report stays where the system keeps it.
fn crash_forget(found: &Found) {
	if let Source::Panic(path) = &found.source
		&& let Err(e) = std::fs::remove_file(path)
		&& e.kind() != std::io::ErrorKind::NotFound
	{
		log::warn!("crash: {} could not be deleted ({e}): it is offered again at the next start", found.id);
	}
}
