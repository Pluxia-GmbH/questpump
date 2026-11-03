//! The outboxes of what the helper posts to questpump.com (contract/feedback-api.yaml, "The client's outbox", which
//! contract/crash-api.yaml's crash reports and contract/telemetry-api.yaml's anonymous counts follow with their own
//! folders and limits): each route's rules, what its answer means for an entry, and the entries kept on disk until the
//! route takes them, one file each (the feedback button's messages, `feedback::Message`, the desktop app's crash
//! reports, `crash::Report`, and its counts' batches, `telemetry::bodies`). No clock or network here: the app posts the
//! bytes and passes the time in.

use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use serde::Deserialize;

use crate::mirror;

/// How long the id of an entry kept once (`Outbox::keep`'s `once`: a game's feedback message) stays remembered, so one
/// the strip shows again, or a helper restarted during the same game session, never posts it twice.
const SEEN_FOR_S: i64 = 86_400;

/// One route's outbox: where it posts, the folder in the data folder that keeps its entries, its bounds (an entry kept
/// at most `keep_days` days from its body's `at`, at most `keep_max` entries, the oldest dropped first) and the answers
/// that drop an entry because the route never takes it.
#[derive(Debug)]
pub struct Rules {
	pub route: &'static str,
	pub folder: &'static str,
	pub keep_days: i64,
	pub keep_max: usize,
	pub refused: &'static [u16],
}

impl Rules {
	/// What the route's answer means for an entry.
	#[must_use]
	pub fn answer(&self, status: u16) -> Answer {
		match status {
			204 => Answer::Sent,
			s if self.refused.contains(&s) => Answer::Refused,
			429 => Answer::Busy,
			_ => Answer::Later,
		}
	}
}

/// What the route's answer means for an entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Answer {
	/// 204: it went, and the outbox deletes it.
	Sent,
	/// One of the route's `refused` answers (400 and 413, and the feedback's 401): the route never takes it, so the
	/// outbox drops it (the app logs it). The route answers these before it counts an entry against its limits, so the
	/// next one may go at once.
	Refused,
	/// 429: one entry a minute per client address (or all the route's 20) went already: kept and sent again in about a
	/// minute.
	Busy,
	/// Anything else (a 5xx, a challenge page, a route not there yet): kept and sent again later.
	Later,
}

/// How `Outbox::keep` took an entry.
#[derive(Debug)]
pub enum Kept {
	/// Written, to be sent.
	New,
	/// Written, but the id of an entry kept once could not be remembered on disk (`seen.json`): it goes once unless a
	/// restarted helper is shown it again.
	Unremembered(std::io::Error),
	/// An entry kept once that the outbox holds already or took in the last day: the strip showed the game's message
	/// again, or the helper restarted during the same game session.
	Again,
}

/// What waits in the outbox, as `Outbox::waiting` found it.
#[derive(Debug, Default)]
pub struct Waiting {
	/// The entries to post, oldest first, with their bytes.
	pub messages: Vec<(String, Vec<u8>)>,
	/// The ids of the entries deleted unsent this time: kept over the rules' days, or older than its newest `keep_max`.
	pub dropped: Vec<String>,
	/// For the log, by id and never a text: why each entry was dropped, and each one skipped this time because it did
	/// not read or a delete did not take (tried again the next time).
	pub notes: Vec<String>,
}

/// The `at` every outbox's body carries (unix seconds), which its age counts from.
#[derive(Deserialize)]
struct At {
	at: i64,
}

/// The entries not sent yet, one file each in `<data dir>/<rules' folder>/` (`<id>.json`, the body as it is posted,
/// written atomically), and the ids of the entries kept once that it took in the last day (`seen.json`).
pub struct Outbox {
	dir: PathBuf,
	rules: &'static Rules,
	seen: BTreeMap<String, i64>,
	/// Entries that went or were refused but whose file could not be deleted (another program holds it open): never
	/// listed as waiting again, their delete tried again at each `waiting`.
	gone: HashSet<String>,
}

impl Outbox {
	const SEEN: &str = "seen.json";

	/// The route's outbox in the data folder, its folder made when missing; a `seen.json` that does not read is an
	/// empty one.
	///
	/// # Errors
	/// The folder cannot be made.
	pub fn open(data_dir: &Path, rules: &'static Rules) -> std::io::Result<Self> {
		let dir = data_dir.join(rules.folder);
		std::fs::create_dir_all(&dir)?;
		let seen = std::fs::read(dir.join(Self::SEEN))
			.ok()
			.and_then(|bytes| serde_json::from_slice(&bytes).ok())
			.unwrap_or_default();
		Ok(Self { dir, rules, seen, gone: HashSet::new() })
	}

	/// Keeps an entry until it is sent, at `now` (unix seconds), its folder made again when it went missing. One kept
	/// `once` (a game's feedback message) has its id remembered before its file is written, so one the app then posts
	/// unkept is not taken twice either.
	///
	/// # Errors
	/// The entry could not be written: the app posts it once, unkept.
	pub fn keep(&mut self, id: &str, body: &[u8], once: bool, now: i64) -> std::io::Result<Kept> {
		let file = format!("{id}.json");
		self.seen.retain(|_, at| now - *at < SEEN_FOR_S);
		if once && (self.seen.contains_key(id) || self.dir.join(&file).exists()) {
			return Ok(Kept::Again);
		}
		if once {
			self.seen.insert(id.to_owned(), now);
		}
		std::fs::create_dir_all(&self.dir)?;
		mirror::write_atomic(&self.dir, &file, body)?;
		if !once {
			return Ok(Kept::New);
		}
		let remembered = serde_json::to_vec(&self.seen)
			.map_err(std::io::Error::from)
			.and_then(|seen| mirror::write_atomic(&self.dir, Self::SEEN, &seen));
		Ok(remembered.map_or_else(Kept::Unremembered, |()| Kept::New))
	}

	/// The entries waiting at `now` (unix seconds), oldest first by their body's `at` (the file's time for one whose
	/// body names none, a counts batch), with the bytes to post. First it deletes what it no longer keeps: an entry past the rules'
	/// days, and every one older than its newest `keep_max`. An entry that does not read is skipped this time, never
	/// the whole outbox.
	///
	/// # Errors
	/// The folder is there and cannot be listed. A missing folder holds nothing (`keep` makes it again).
	pub fn waiting(&mut self, now: i64) -> std::io::Result<Waiting> {
		let mut out = Waiting::default();
		let mut found = Vec::new();
		for (id, path) in self.entries(&mut out.notes)? {
			if self.gone.contains(&id) {
				if self.remove(&id).is_err() {
					out.notes.push(format!("{id} went already, and its file still could not be deleted"));
				}
				continue;
			}
			let read = std::fs::metadata(&path)
				.and_then(|m| m.modified())
				.and_then(|kept| std::fs::read(&path).map(|bytes| (kept, bytes)));
			match read {
				Ok((kept, bytes)) => {
					let at = serde_json::from_slice::<At>(&bytes).map_or_else(
						|_| kept.duration_since(UNIX_EPOCH).map_or(0, |d| i64::try_from(d.as_secs()).unwrap_or(0)),
						|body| body.at,
					);
					found.push((at, id, bytes));
				}
				Err(e) => out.notes.push(format!("{id} did not read ({e}): skipped this time")),
			}
		}
		found.sort_unstable_by(|a, b| (a.0, &a.1).cmp(&(b.0, &b.1)));
		let (days, max) = (self.rules.keep_days, self.rules.keep_max);
		let over = found.len().saturating_sub(max);
		for (i, (at, id, bytes)) in found.into_iter().enumerate() {
			let why = if now - at > days * 86_400 {
				format!("kept over {days} days")
			} else if i < over {
				format!("older than the newest {max}")
			} else {
				out.messages.push((id, bytes));
				continue;
			};
			match std::fs::remove_file(self.dir.join(format!("{id}.json"))) {
				Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
					out.notes
						.push(format!("{id} is {why}, and its file could not be deleted ({e}): skipped this time"));
				}
				_ => {
					out.notes.push(format!("{id} dropped unsent: {why}"));
					out.dropped.push(id);
				}
			}
		}
		Ok(out)
	}

	/// Deletes every entry unsent (its switch turned off: Send crash reports, Usage statistics), each one whose file
	/// stays never listed as waiting again: the ids deleted, and the notes for the log.
	///
	/// # Errors
	/// The folder is there and cannot be listed.
	pub fn clear(&mut self) -> std::io::Result<(Vec<String>, Vec<String>)> {
		let (mut deleted, mut notes) = (Vec::new(), Vec::new());
		for (id, _) in self.entries(&mut notes)? {
			match self.remove(&id) {
				Ok(()) => deleted.push(id),
				Err(e) => notes.push(format!("{id} could not be deleted ({e}): it is never sent")),
			}
		}
		Ok((deleted, notes))
	}

	/// An entry's file, which the batch that posted it deletes as soon as the route took or refused it (the crash sweep's
	/// batch, first in the start, may finish before the loop that would hear it runs).
	#[must_use]
	pub fn file(&self, id: &str) -> PathBuf {
		self.dir.join(format!("{id}.json"))
	}

	/// Deletes an entry that went or that the route refused; one whose file stays is never listed as waiting again.
	///
	/// # Errors
	/// The file is there and cannot be deleted (`waiting` tries again).
	pub fn remove(&mut self, id: &str) -> std::io::Result<()> {
		match std::fs::remove_file(self.dir.join(format!("{id}.json"))) {
			Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
				self.gone.insert(id.to_owned());
				Err(e)
			}
			_ => {
				self.gone.remove(id);
				Ok(())
			}
		}
	}

	/// Each entry's id and file in the folder, never `seen.json` nor a write cut short (`mirror::temp`'s); one that
	/// does not list gets a note. A missing folder holds none.
	fn entries(&self, notes: &mut Vec<String>) -> std::io::Result<Vec<(String, PathBuf)>> {
		let listed = match std::fs::read_dir(&self.dir) {
			Ok(listed) => listed,
			Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
			Err(e) => return Err(e),
		};
		let mut out = Vec::new();
		for entry in listed {
			let path = match entry {
				Ok(entry) => entry.path(),
				Err(e) => {
					notes.push(format!("an entry did not read ({e}): skipped this time"));
					continue;
				}
			};
			let name = path.file_name().and_then(|n| n.to_str()).unwrap_or_default();
			if let Some(id) = name.strip_suffix(".json").filter(|_| name != Self::SEEN) {
				out.push((id.to_owned(), path.clone()));
			}
		}
		Ok(out)
	}
}
