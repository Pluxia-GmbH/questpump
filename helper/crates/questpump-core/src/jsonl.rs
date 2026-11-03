//! A JSON Lines file in the data folder, as the row store (`rows.jsonl`) and the sheet log (`sheets.jsonl`) keep
//! theirs: one JSON object per line, appended and flushed, rewritten whole and atomically when it is compacted. A line
//! that does not read (damaged bytes, a write cut short) is skipped and counted, so one bad line never loses the rest.

use std::io::Write as _;
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde::de::DeserializeOwned;

/// Where a list's lines go; the default keeps them in memory only (`SheetLog::in_memory`).
#[derive(Default)]
pub struct JsonLines {
	/// The data folder and the file's name.
	file: Option<(PathBuf, &'static str)>,
	/// The file's last line has no newline (a write cut short): the next append starts a new line first.
	torn: bool,
}

impl JsonLines {
	/// Reads `name` in `dir` (an absent file holds nothing): the items that read, in the file's order, and how many
	/// lines did not.
	///
	/// # Errors
	/// A file that exists but cannot be read.
	pub fn open<T: DeserializeOwned>(dir: &Path, name: &'static str) -> std::io::Result<(Self, Vec<T>, usize)> {
		let bytes = match std::fs::read(dir.join(name)) {
			Ok(b) => b,
			Err(e) if e.kind() == std::io::ErrorKind::NotFound => Vec::new(),
			Err(e) => return Err(e),
		};
		let torn = bytes.last().is_some_and(|&b| b != b'\n');
		let (mut items, mut damaged) = (Vec::new(), 0);
		for line in bytes.split(|&b| b == b'\n').filter(|l| !l.is_empty()) {
			match serde_json::from_slice(line) {
				Ok(item) => items.push(item),
				Err(_) => damaged += 1,
			}
		}
		Ok((Self { file: Some((dir.to_path_buf(), name)), torn }, items, damaged))
	}

	/// Appends one item as a line and flushes it.
	///
	/// # Errors
	/// The file cannot be opened or written.
	pub fn append<T: Serialize>(&mut self, item: &T) -> std::io::Result<()> {
		let Some((dir, name)) = &self.file else { return Ok(()) };
		let mut f = std::fs::OpenOptions::new().create(true).append(true).open(dir.join(name))?;
		let mut line = Vec::new();
		if self.torn {
			line.push(b'\n');
		}
		line.extend(serde_json::to_vec(item)?);
		line.push(b'\n');
		let written = f.write_all(&line).and_then(|()| f.flush());
		// a write cut short (a full disk) may leave part of the line: the next append starts a new one, as after a torn
		// file, so the line after it still reads
		self.torn = written.is_err();
		written
	}

	/// Replaces the file with these items (a compaction), through a temporary file and a rename, so a failed write
	/// leaves the old file whole.
	///
	/// # Errors
	/// The temporary file cannot be written or renamed.
	pub fn rewrite<'a, T: Serialize + 'a>(&self, items: impl IntoIterator<Item = &'a T>) -> std::io::Result<()> {
		let Some((dir, name)) = &self.file else { return Ok(()) };
		let mut text = Vec::new();
		for item in items {
			text.extend(serde_json::to_vec(item)?);
			text.push(b'\n');
		}
		crate::mirror::write_atomic(dir, name, &text)
	}
}
