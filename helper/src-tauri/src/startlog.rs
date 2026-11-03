//! The log before the log plugin's (contract/crash-api.yaml, "The desktop app": the crash sweep runs first in the
//! start, and a start that fails before the window writes its error to `helper.log`). From the first lines of `run()`
//! every record goes through `START`: it keeps them (the start's own, the sweep's, the settings' read) until the app's
//! setup attaches the log plugin's logger (`attach`, which takes them first, in their order) and from then on hands
//! every record straight to it, taking no lock (a panic hook's line never waits on one the panicking thread holds). A
//! start that fails before its setup (`fail`) writes what it kept and its error into `helper.log` itself, in the
//! plugin's own format and folder.

use std::io::Write as _;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use log::{Level, LevelFilter, Log, Metadata, Record};

/// The most records kept before the plugin's logger is attached: a start says a few dozen lines.
const KEPT_MAX: usize = 500;

/// A record kept: its level, its target and its message, and when it came (unix seconds).
struct Kept {
	level: Level,
	target: String,
	message: String,
	at: i64,
}

/// The process's logger.
struct Start {
	inner: OnceLock<Box<dyn Log>>,
	kept: Mutex<Vec<Kept>>,
}

static START: Start = Start { inner: OnceLock::new(), kept: Mutex::new(Vec::new()) };

impl Log for Start {
	fn enabled(&self, metadata: &Metadata) -> bool {
		self.inner.get().map_or(metadata.level() <= Level::Info, |inner| inner.enabled(metadata))
	}

	fn log(&self, record: &Record) {
		if let Some(inner) = self.inner.get() {
			inner.log(record);
			return;
		}
		if record.level() > Level::Info {
			return;
		}
		// never waits: a record that meets the lock taken (another thread keeping one) is lost, never the start
		if let Ok(mut kept) = self.kept.try_lock()
			&& kept.len() < KEPT_MAX
		{
			kept.push(Kept {
				level: record.level(),
				target: record.target().to_owned(),
				message: record.args().to_string(),
				at: time::OffsetDateTime::now_utc().unix_timestamp(),
			});
		}
	}

	fn flush(&self) {
		if let Some(inner) = self.inner.get() {
			inner.flush();
		}
	}
}

/// The process's logger from here on (first in `run()`, after the panic hook): it keeps the records until `attach`.
pub(crate) fn install() {
	if log::set_logger(&START).is_ok() {
		log::set_max_level(LevelFilter::Info);
	}
}

/// The log plugin's logger, from the app's setup: the records kept so far go to it first, in their order, then every
/// record.
pub(crate) fn attach(logger: Box<dyn Log>, level: LevelFilter) {
	let kept = START.kept.lock().map(|mut k| std::mem::take(&mut *k)).unwrap_or_default();
	for k in &kept {
		logger.log(&Record::builder().level(k.level).target(&k.target).args(format_args!("{}", k.message)).build());
	}
	if START.inner.set(logger).is_ok() {
		log::set_max_level(level);
	}
}

/// The log's file as the log plugin names it: `helper.log` in the app's log folder (`~/Library/Logs/<identifier>` on
/// macOS, `%LOCALAPPDATA%\<identifier>\logs` on Windows, as Tauri's `app_log_dir`).
fn file(identifier: &str) -> Option<PathBuf> {
	#[cfg(target_os = "macos")]
	let dir = dirs::home_dir()?.join("Library/Logs").join(identifier);
	#[cfg(not(target_os = "macos"))]
	let dir = dirs::data_local_dir()?.join(identifier).join("logs");
	Some(dir.join("helper.log"))
}

/// A line as the plugin writes one: `[2026-10-12][17:03:01][target][LEVEL] message`, in UTC.
fn line(at: i64, level: Level, target: &str, message: &str) -> String {
	let stamp = time::OffsetDateTime::from_unix_timestamp(at)
		.ok()
		.and_then(|t| {
			t.format(time::macros::format_description!("[[[year]-[month]-[day]][[[hour]:[minute]:[second]]")).ok()
		})
		.unwrap_or_default();
	format!("{stamp}[{target}][{level}] {message}\n")
}

/// The start failed before its setup attached the plugin's logger: what was kept and the failure go into
/// `helper.log` (`identifier`: the app's, tauri.conf.json's), and to the standard error.
pub(crate) fn fail(identifier: &str, error: &dyn std::fmt::Display) {
	eprintln!("QuestPump could not start: {error}"); // copy-lint: not shown (the standard error's line)
	let at = time::OffsetDateTime::now_utc().unix_timestamp();
	let mut text: String = START
		.kept
		.lock()
		.map(|mut k| std::mem::take(&mut *k))
		.unwrap_or_default()
		.iter()
		.map(|k| line(k.at, k.level, &k.target, &k.message))
		.collect();
	// copy-lint: not shown (a log line)
	text.push_str(&line(at, Level::Error, module_path!(), &format!("QuestPump could not start: {error}")));
	let Some(path) = file(identifier) else { return };
	let written = path
		.parent()
		.map_or(Ok(()), std::fs::create_dir_all)
		.and_then(|()| std::fs::OpenOptions::new().create(true).append(true).open(&path))
		.and_then(|mut f| f.write_all(text.as_bytes()));
	if let Err(e) = written {
		eprintln!("{}: {e}", path.display()); // copy-lint: not shown (the standard error's line)
	}
}
