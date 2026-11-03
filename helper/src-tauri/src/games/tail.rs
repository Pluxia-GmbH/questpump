//! A game's log read as it grows (docs/plan.md slice 48; rust-helper rule 10): opened read-only at each look and closed
//! after it, so no handle stays between looks (std's `File::open` on Windows shares reading, writing and deleting, so
//! the game goes on writing, renaming and deleting its log). A log already there when it is first followed is read from
//! its end (its lines are old); a log made anew under the same name (shorter than what was read, or another file: MTG
//! Arena's `Player.log` at each start) from its beginning, and so is a log found later in a folder of logs (`Follow`)
//! when it changed since the follow began, while one that did not (an old log found late) is read from its end. Only
//! complete lines are read, at most `READ_BYTES` a look; a line longer than `LINE_BYTES` is skipped whole. The lines go
//! to questpump-core's readers (`feeds`) and are dropped there; none is kept or logged.

use std::fs::{File, Metadata};
use std::io::{Read as _, Seek as _, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

/// The most a look reads of a log: a game's busiest seconds write far less.
const READ_BYTES: u64 = 4 * 1024 * 1024;
/// The longest line read: MTG Arena's longest messages are a few hundred kilobytes.
const LINE_BYTES: usize = 1024 * 1024;
/// A folder of logs is looked into for its newest at most this often (`Follow`).
const RESOLVE: Duration = Duration::from_secs(10);

/// Which file a path names now: its inode on macOS; on Windows its creation time, which NTFS's tunnelling may carry
/// over to a file made again under the same name within 15 s (the shrink check covers that case).
fn identity(meta: &Metadata) -> u64 {
	#[cfg(unix)]
	{
		use std::os::unix::fs::MetadataExt as _;
		meta.ino()
	}
	#[cfg(windows)]
	{
		use std::os::windows::fs::MetadataExt as _;
		meta.creation_time()
	}
}

/// One log followed as it grows.
pub(super) struct Tail {
	path: PathBuf,
	/// The file read so far, and how far.
	file: Option<u64>,
	at: u64,
	/// Within a line longer than `LINE_BYTES`, which is skipped to its end.
	skipping: bool,
	/// The file began anew since `began` was last asked (or this tail is new).
	fresh: bool,
}

impl Tail {
	/// Follows `path` from its end: a log that was there before.
	pub(super) fn at_end(path: PathBuf) -> Self {
		let meta = std::fs::metadata(&path).ok();
		let file = meta.as_ref().map(identity);
		let at = meta.map_or(0, |m| m.len());
		Self { path, file, at, skipping: false, fresh: true }
	}

	/// Follows `path` from its beginning: a log made since.
	pub(super) fn at_start(path: PathBuf) -> Self {
		Self { path, file: None, at: 0, skipping: false, fresh: true }
	}

	pub(super) fn path(&self) -> &Path {
		&self.path
	}

	/// Whether the log began anew (another file, or this tail new) since the last ask.
	pub(super) fn began(&mut self) -> bool {
		std::mem::take(&mut self.fresh)
	}

	/// The complete lines written since the last look; none while the log cannot be read.
	pub(super) fn lines(&mut self) -> Vec<String> {
		let Ok(mut file) = File::open(&self.path) else { return Vec::new() };
		let Ok(meta) = file.metadata() else { return Vec::new() };
		let now = identity(&meta);
		if meta.len() < self.at || self.file.is_some_and(|file| file != now) {
			self.at = 0;
			self.skipping = false;
			self.fresh = true;
		}
		self.file = Some(now);
		if meta.len() == self.at || file.seek(SeekFrom::Start(self.at)).is_err() {
			return Vec::new();
		}
		let mut bytes = Vec::new();
		if file.take(READ_BYTES).read_to_end(&mut bytes).is_err() {
			return Vec::new();
		}
		let Some(last) = bytes.iter().rposition(|&b| b == b'\n') else {
			// no line ends in this read: a line longer than a whole read is skipped, a shorter one waits
			if bytes.len() as u64 == READ_BYTES {
				self.at += READ_BYTES;
				self.skipping = true;
			}
			return Vec::new();
		};
		self.at += last as u64 + 1;
		let mut out = Vec::new();
		for line in bytes.get(..last).unwrap_or_default().split(|&b| b == b'\n') {
			if std::mem::take(&mut self.skipping) || line.len() > LINE_BYTES {
				continue;
			}
			let text = String::from_utf8_lossy(line);
			out.push(text.trim_end_matches('\r').to_owned());
		}
		out
	}
}

/// A log a game writes anew in a folder each session (Hearthstone's `Power.log` in its session's folder, Elite
/// Dangerous' journals): the newest one found, followed as it grows.
#[derive(Default)]
pub(super) struct Follow {
	tail: Option<Tail>,
	/// When the folder was last looked into.
	looked: Option<Instant>,
	/// When the follow began (its first look), by the system's clock, which a log's last change is held against.
	since: Option<SystemTime>,
}

impl Follow {
	/// The new lines of the newest log `find` names, looked for at the first call and then every `RESOLVE` while the
	/// game runs: the first log found ran before and is read from its end; a later one from its beginning when it
	/// changed since the follow began (the game's new session), else from its end (an old log, its games long played).
	/// When a newer log takes the old one's place, the old one's last lines are read first: they come first, the new
	/// log's after them (an end the old log wrote just before the new one began still counts).
	pub(super) fn lines(
		&mut self,
		running: bool,
		find: impl FnOnce() -> Option<PathBuf>,
	) -> (Vec<String>, Vec<String>) {
		let first = self.looked.is_none();
		let mut left = Vec::new();
		if first || (running && self.looked.is_some_and(|at| at.elapsed() >= RESOLVE)) {
			self.looked = Some(Instant::now());
			let since = *self.since.get_or_insert_with(SystemTime::now);
			if let Some(path) = find().filter(|p| self.tail.as_ref().is_none_or(|t| t.path() != p)) {
				let new = !first && std::fs::metadata(&path).and_then(|m| m.modified()).is_ok_and(|at| at >= since);
				let next = if new { Tail::at_start(path) } else { Tail::at_end(path) };
				left = self.tail.replace(next).map(|mut old| old.lines()).unwrap_or_default();
			}
		}
		(left, self.tail.as_mut().map(Tail::lines).unwrap_or_default())
	}

	/// Whether the log followed began anew (another log found, or this one made anew under its name) since the last
	/// ask; true at the first log found.
	pub(super) fn began(&mut self) -> bool {
		self.tail.as_mut().is_some_and(Tail::began)
	}
}

/// The newest entry of `dir` whose name `wanted` accepts, by its last change.
pub(super) fn newest(dir: &Path, wanted: impl Fn(&str) -> bool) -> Option<PathBuf> {
	std::fs::read_dir(dir)
		.ok()?
		.flatten()
		.filter(|e| e.file_name().to_str().is_some_and(&wanted))
		.filter_map(|e| Some((e.metadata().ok()?.modified().ok()?, e.path())))
		.max_by_key(|(at, _)| *at)
		.map(|(_, path)| path)
}
