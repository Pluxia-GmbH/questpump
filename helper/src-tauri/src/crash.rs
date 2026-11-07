//! The desktop app's crash capture (contract/crash-api.yaml, "The desktop app"; questpump-core's `crash.rs` owns the
//! report, its text and the opt-in regions, `runtime/crash.rs` what the start, the switch and the page do with them).
//! The panic hook, installed first in `run()`, writes each panic's report into the data folder (`crashes/`, one file a
//! panic, at most `crash::OUTBOX_MAX` a run) with `std::fs` before anything else, then logs one line, then hands the
//! panic on to the hook it replaced (the standard library's); it calls nothing that can panic itself. A panic the
//! engine's door catches on purpose (`caught`, `questpump-desk/src/engine.rs`) is logged and never becomes a report, since the app
//! goes on, and a panic that cannot unwind right after a report on its thread (the abort a panic in a system callback
//! ends in) writes none. At the next start `found` reads those files and the system's own reports of a crash of this
//! program newer than the last one handled: on macOS each `<process name>-*.ips` in `~/Library/Logs/DiagnosticReports`,
//! on Windows each Windows Error Reporting report in the user's store (`%LOCALAPPDATA%\Microsoft\Windows\WER`) and the
//! machine's (`%ProgramData%\Microsoft\Windows\WER`), each `ReportArchive` and `ReportQueue` (`AppCrash_<the program's
//! name, cut short>_*\Report.wer`), never one older than the outbox keeps, and keeps the newest `crash::OUTBOX_MAX`.
//! One that does not read is skipped with a log line (one written in the last minute is tried again at the next start),
//! and the system's report of the abort a panic ended in, at this start or an earlier one, is left to the panic's own.
//! The route's `os` and the region and time zone the opt-in regions read come from here too.

use std::cell::Cell;
use std::panic::AssertUnwindSafe;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use questpump_core::crash::{self, Handled, Origin, Panic, Report};
use questpump_core::mirror;

/// How recent a system report that does not read may be and still be tried again at the next start, in seconds: the
/// system may still be writing it.
const FRESH_S: i64 = 60;

/// The texts the standard library gives a panic that cannot unwind (core's `panic_cannot_unwind`, a panic out of a
/// function that cannot unwind such as a system callback, and `panic_in_cleanup`, a panic in a destructor while
/// unwinding; rustc's `UnwindTerminateReason::as_str`), which aborts the process: `PanicHookInfo::can_unwind` is
/// unstable (`panic_can_unwind`, rust-lang/rust#92988), so the hook knows such a panic by its text.
const CANNOT_UNWIND: [&str; 2] = [
	"panic in a function that cannot unwind", // copy-lint: not shown (the standard library's text)
	"panic in a destructor during cleanup",   // copy-lint: not shown (the standard library's text)
];

thread_local! {
	/// The engine's door is running a call whose panic it catches on purpose (`caught`).
	static CAUGHT: Cell<bool> = const { Cell::new(false) };
	/// When the panic hook last wrote a report on this thread (unix seconds).
	static REPORTED: Cell<Option<i64>> = const { Cell::new(None) };
}

/// The route's `os`: the system as Send feedback's diagnostics name it and the architecture, "macOS 27.0, arm64".
pub(crate) fn os() -> String {
	let arch = match std::env::consts::ARCH {
		"aarch64" => "arm64",
		other => other,
	};
	format!("{}, {arch}", crate::feedback::os())
}

/// The home folder as a report's text names it `~`.
fn home() -> Option<String> {
	dirs::home_dir().map(|h| h.to_string_lossy().into_owned())
}

/// Installs the panic hook (first in `run()`, on the main thread, before anything that may panic): each panic's report
/// goes into `<data dir>/crashes/` as `p-<unix ms>-<n>.json`, written with `std::fs` and renamed into place, then one
/// log line names it, then the hook it replaced runs. A run writes at most `crash::OUTBOX_MAX` reports, so a panic the
/// app survives again and again writes no more; a panic that cannot unwind within `crash::same_crash` of a report on
/// its thread is the abort that report's panic ends in and writes none. The hook never runs again on a thread inside it
/// (std aborts a panic in a hook first), and its log line cannot wait on the log file's lock: fern formats a record
/// before it takes that lock (`meta-logging-in-format`, src-tauri's Cargo.toml), so a panic while a log call formats
/// holds none.
pub(crate) fn catch_panics() {
	let folder = crate::runtime::data_home().map(|dir| dir.join(crash::FOLDER));
	let (os, home) = (os(), home());
	let count = AtomicUsize::new(0);
	#[cfg(windows)]
	main_thread::mark();
	let before = std::panic::take_hook();
	std::panic::set_hook(Box::new(move |info| {
		let location =
			info.location().map_or_else(String::new, |l| format!("{}:{}:{}", l.file(), l.line(), l.column()));
		let ms = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_millis());
		let at = i64::try_from(ms / 1000).unwrap_or(0);
		let said = if CAUGHT.get() {
			// copy-lint: not shown (a log line)
			format!("crash: a panic at {location}, which the engine's door catches: no report")
		} else if info.payload_as_str().is_some_and(|said| CANNOT_UNWIND.contains(&said))
			&& REPORTED.get().is_some_and(|last| crash::same_crash(last, at))
		{
			// copy-lint: not shown (a log line)
			format!("crash: a panic at {location} that cannot unwind, after this thread's report: no report")
		} else {
			let n = count.fetch_add(1, Ordering::Relaxed);
			if n < crash::OUTBOX_MAX {
				let report = Report::panic(
					&Origin { version: env!("CARGO_PKG_VERSION"), os: &os, home: home.as_deref() },
					&Panic {
						at,
						thread: &thread_name(),
						message: info.payload_as_str().unwrap_or_default(),
						location: &location,
						backtrace: &trace(),
					},
				);
				let name = format!("p-{ms}-{n}.json");
				match write(folder.as_deref(), &name, &report) {
					Ok(()) => {
						REPORTED.set(Some(at));
						// copy-lint: not shown (a log line)
						format!("crash: a panic at {location}, its report {name} written")
					}
					Err(e) => format!("crash: a panic at {location}, its report not written ({e})"),
				}
			} else {
				// copy-lint: not shown (a log line)
				format!("crash: a panic at {location}, its report not written: {} this run already", crash::OUTBOX_MAX)
			}
		};
		log::error!("{said}");
		before(info);
	}));
}

/// A panic's report into the folder: the JSON written beside its name, then renamed into place
/// (`mirror::write_quick`), so a file there always reads whole.
fn write(folder: Option<&Path>, name: &str, report: &Report) -> std::io::Result<()> {
	let folder = folder.ok_or_else(|| std::io::Error::other("no data folder"))?;
	std::fs::create_dir_all(folder)?;
	let bytes = serde_json::to_vec(report).map_err(std::io::Error::other)?;
	mirror::write_quick(folder, name, &bytes)
}

/// The panicking thread's name as the system keeps it (the standard library gives a thread it starts its name there),
/// "main" for the main thread, else "unnamed": `std::thread::current()` panics once the thread's own data is gone, so
/// the hook never calls it.
#[cfg(target_os = "macos")]
fn thread_name() -> String {
	// SAFETY: pthread_main_np only reads the calling thread's own state
	if unsafe { libc::pthread_main_np() } == 1 {
		return "main".to_owned(); // copy-lint: not shown (the standard library's name for the main thread)
	}
	let mut name = [0_u8; 64];
	// SAFETY: pthread_getname_np writes at most `name.len()` bytes, ending in a NUL, into this buffer on the stack, for
	// the calling thread's own handle
	let read = unsafe { libc::pthread_getname_np(libc::pthread_self(), name.as_mut_ptr().cast(), name.len()) };
	let name = std::ffi::CStr::from_bytes_until_nul(&name).map(|n| n.to_string_lossy().into_owned());
	match name {
		Ok(name) if read == 0 && !name.is_empty() => name,
		_ => "unnamed".to_owned(), // copy-lint: not shown (a report's thread line)
	}
}

/// The panicking thread's name as the system keeps it (the standard library gives a thread it starts its description
/// there), "main" for the thread `catch_panics` ran on, else "unnamed": `std::thread::current()` panics once the
/// thread's own data is gone, so the hook never calls it.
#[cfg(windows)]
fn thread_name() -> String {
	use windows::Win32::Foundation::{HLOCAL, LocalFree};
	use windows::Win32::System::Threading::{GetCurrentThread, GetThreadDescription};
	if main_thread::is_current() {
		return "main".to_owned(); // copy-lint: not shown (the standard library's name for the main thread)
	}
	// SAFETY: GetCurrentThread is the calling thread's own pseudo handle, which needs no closing
	let name = match unsafe { GetThreadDescription(GetCurrentThread()) } {
		Ok(description) if !description.is_null() => {
			// SAFETY: a description the call returned is a NUL-terminated string it allocated
			let name = unsafe { description.to_string() }.unwrap_or_default();
			// SAFETY: the description was allocated by GetThreadDescription with LocalAlloc and is freed here once
			unsafe { LocalFree(Some(HLOCAL(description.0.cast()))) };
			name
		}
		_ => String::new(),
	};
	if name.is_empty() { "unnamed".to_owned() } else { name } // copy-lint: not shown (a report's thread line)
}

/// The main thread on Windows, which has no description of its own: the thread `catch_panics` ran on.
#[cfg(windows)]
mod main_thread {
	use std::sync::atomic::{AtomicU32, Ordering};

	use windows::Win32::System::Threading::GetCurrentThreadId;

	static MAIN: AtomicU32 = AtomicU32::new(0);

	pub(super) fn mark() {
		// SAFETY: GetCurrentThreadId only reads the calling thread's own id
		MAIN.store(unsafe { GetCurrentThreadId() }, Ordering::Relaxed);
	}

	pub(super) fn is_current() -> bool {
		// SAFETY: GetCurrentThreadId only reads the calling thread's own id
		let id = unsafe { GetCurrentThreadId() };
		id != 0 && id == MAIN.load(Ordering::Relaxed)
	}
}

/// The panic's backtrace as the standard library prints it, a numbered line a frame with its function's name, which
/// questpump-core's `crash::Report::panic` reads.
#[cfg(target_os = "macos")]
fn trace() -> String {
	std::backtrace::Backtrace::force_capture().to_string()
}

/// The panic's backtrace, a numbered line a frame as questpump-core's `crash::Report::panic` reads it, from the
/// `backtrace` crate, whose frames keep their addresses: each frame as its module and the offset into it
/// (`questpump-helper.exe+0x1a2b3`), never an address. Unresolved: the installer ships no symbols, so the program's
/// frames would read no name anyway, and resolving runs dbghelp, which takes the loader's lock, inside the hook. The hook's
/// own frames come first (no name marks where the panic begins), as many as the standard library's machinery takes.
#[cfg(windows)]
fn trace() -> String {
	use std::fmt::Write as _;
	let mut out = String::new();
	for (n, frame) in backtrace::Backtrace::new_unresolved().frames().iter().enumerate() {
		let place = module_offset(frame.ip()).unwrap_or_else(|| "<unknown>".to_owned()); // copy-lint: not shown
		let _ = writeln!(out, "{n:4}: {place}");
	}
	out
}

/// An address as the module it lies in and the offset into it, `questpump-helper.exe+0x1a2b3`.
#[cfg(windows)]
fn module_offset(ip: *mut std::ffi::c_void) -> Option<String> {
	use windows::Win32::Foundation::HMODULE;
	use windows::Win32::System::LibraryLoader::{
		GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS, GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT, GetModuleFileNameW,
		GetModuleHandleExW,
	};
	use windows::core::PCWSTR;
	let mut module = HMODULE::default();
	let flags = GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT;
	// SAFETY: with FROM_ADDRESS the name argument is an address inside a module, never read as a string, and with
	// UNCHANGED_REFCOUNT no reference is taken that would need releasing; `module` is a valid place on the stack
	unsafe { GetModuleHandleExW(flags, PCWSTR(ip.cast_const().cast()), &raw mut module) }.ok()?;
	let mut path = [0_u16; 1024];
	// SAFETY: writes at most the buffer's length into it, for a module the process has loaded
	let len = usize::try_from(unsafe { GetModuleFileNameW(Some(module), &mut path) }).ok()?;
	let path = String::from_utf16_lossy(path.get(..len).filter(|p| !p.is_empty() && len < 1024)?);
	let name = path.rsplit(['\\', '/']).next()?;
	let offset = ip.addr().checked_sub(module.0.addr())?;
	Some(format!("{name}+{offset:#x}"))
}

/// Runs `call`, catching its panic as the engine's door does on purpose (the calls behind it still run): the hook logs
/// it and writes no report.
pub(crate) fn caught<R>(call: impl FnOnce() -> R) -> std::thread::Result<R> {
	let was = CAUGHT.replace(true);
	let result = std::panic::catch_unwind(AssertUnwindSafe(call));
	CAUGHT.set(was);
	result
}

/// Whether this computer is in an opt-in region (questpump-core `crash::opt_in`): its region's country as the
/// system's settings name it (`units::region`) and its IANA time zone (the `iana-time-zone` crate: macOS's
/// `CFTimeZoneCopySystem`, Windows' `Calendar.GetTimeZone`).
pub(crate) fn opt_in() -> bool {
	let zone =
		iana_time_zone::get_timezone().inspect_err(|e| log::warn!("crash: the time zone does not read ({e})")).ok();
	crash::opt_in(crate::units::region().as_deref(), zone.as_deref())
}

/// Where a report found at the start came from.
#[derive(Debug, Clone)]
pub(crate) enum Source {
	/// The panic hook's file, deleted once the report is kept or turned down.
	Panic(PathBuf),
	/// The system's report, never deleted: once it is handled the mark moves past it (`Waiting::newest`).
	System,
}

/// A report found at the start: its id, where it came from, and the report.
#[derive(Debug, Clone)]
pub(crate) struct Found {
	pub(crate) id: String,
	pub(crate) source: Source,
	pub(crate) report: Report,
}

/// The reports that wait at a start, oldest first, and the newest of the system's reports looked at (the mark once
/// they are handled).
#[derive(Debug, Default)]
pub(crate) struct Waiting {
	pub(crate) reports: Vec<Found>,
	pub(crate) newest: Option<Handled>,
}

/// The reports that wait at a start, at `now` (unix seconds): the panic hook's files, and the system's reports of this
/// program newer than `handled`, but the one a panic's abort left (`crash::same_crash`: a panic found now, or the
/// newest panic handled at an earlier start, `handled_panic`), none written more than `crash::OUTBOX_DAYS` ago, the
/// newest `crash::OUTBOX_MAX` of them (as many as the outbox keeps): an older panic file is deleted and an older system
/// report skipped, each with a log line.
pub(crate) fn found(data_dir: &Path, handled: Option<&Handled>, handled_panic: Option<i64>, now: i64) -> Waiting {
	let since = now - crash::OUTBOX_DAYS * 86_400;
	let (os, home) = (os(), home());
	let origin = Origin { version: env!("CARGO_PKG_VERSION"), os: &os, home: home.as_deref() };
	let mut reports = panics(data_dir, since);
	let (system, newest) = system(&origin, handled, since, now);
	for one in system {
		let abort = reports
			.iter()
			.find(|p| matches!(p.source, Source::Panic(_)) && crash::same_crash(p.report.at, one.report.at));
		if let Some(panic) = abort {
			log::info!("crash: {} is the abort the panic {} ended in: skipped", one.id, panic.id);
		} else if let Some(at) = handled_panic.filter(|at| crash::same_crash(*at, one.report.at)) {
			log::info!(
				"crash: {} is the abort the panic at {at}, handled at an earlier start, ended in: skipped",
				one.id
			);
		} else {
			reports.push(one);
		}
	}
	reports.sort_by(|a, b| (a.report.at, &a.id).cmp(&(b.report.at, &b.id)));
	let over = reports.len().saturating_sub(crash::OUTBOX_MAX);
	for old in reports.drain(..over) {
		let why = format!("older than the newest {}", crash::OUTBOX_MAX); // copy-lint: not shown (a log line)
		match &old.source {
			Source::Panic(path) => match std::fs::remove_file(path) {
				Ok(()) => log::warn!("crash: {} deleted: {why}", old.id),
				Err(e) => log::warn!("crash: {} is {why}, and could not be deleted ({e})", old.id),
			},
			Source::System => log::warn!("crash: {} skipped: {why}", old.id),
		}
	}
	Waiting { reports, newest }
}

/// The panic hook's reports in the data folder; one that does not read, or older than `since`, is deleted with a log
/// line.
fn panics(data_dir: &Path, since: i64) -> Vec<Found> {
	let dir = data_dir.join(crash::FOLDER);
	let listed = match std::fs::read_dir(&dir) {
		Ok(listed) => listed,
		Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Vec::new(),
		Err(e) => {
			log::warn!("crash: {} could not be listed ({e})", dir.display());
			return Vec::new();
		}
	};
	let mut out = Vec::new();
	for path in listed.flatten().map(|entry| entry.path()) {
		let Some(id) = path.file_name().and_then(|n| n.to_str()).and_then(|n| n.strip_suffix(".json")) else {
			continue;
		};
		let id = id.to_owned();
		let read = std::fs::read(&path)
			.map_err(|e| e.to_string())
			.and_then(|bytes| serde_json::from_slice::<Report>(&bytes).map_err(|e| e.to_string()));
		let why = match read {
			Ok(report) if report.at >= since => {
				out.push(Found { id, source: Source::Panic(path), report });
				continue;
			}
			Ok(_) => format!("older than {} days", crash::OUTBOX_DAYS), // copy-lint: not shown (a log line)
			Err(e) => format!("it does not read ({e})"),
		};
		match std::fs::remove_file(&path) {
			Ok(()) => log::warn!("crash: {id} deleted: {why}"),
			Err(e) => log::warn!("crash: {id} is {why}, and could not be deleted ({e})"),
		}
	}
	out
}

/// The time a file was last written, unix seconds.
#[cfg(target_os = "macos")]
fn written_at(path: &Path) -> Option<i64> {
	let modified = std::fs::metadata(path).and_then(|m| m.modified()).ok()?;
	i64::try_from(modified.duration_since(UNIX_EPOCH).ok()?.as_secs()).ok()
}

/// The system's reports looked at in one start: each one's mark with its report when it reads, and the moment of the
/// oldest one that does not read yet (`take`), before which the mark stops.
#[derive(Default)]
struct Looked {
	taken: Vec<(Handled, Option<Found>)>,
	held: Option<i64>,
}

impl Looked {
	/// A system report as read: its report when it is this program's and reads, else a log line once it is newer than
	/// the mark. The mark compares moments alone: one crash's report at the same moment under another name (Windows
	/// moves a queued report into the archive) is the one handled. One that does not read is taken past, but when it
	/// was written in the last `FRESH_S` (the system may still be writing it): that one, and every one after it, waits
	/// for the next start (`settle`). `name` is its file's or its folder's.
	fn take(
		&mut self,
		read: Result<Option<Report>, String>,
		name: &str,
		id: String,
		file_at: i64,
		now: i64,
		handled: Option<&Handled>,
	) {
		let at = match &read {
			Ok(Some(report)) => report.at,
			Ok(None) => return,
			Err(_) => file_at,
		};
		if handled.is_some_and(|h| h.at >= at) {
			return;
		}
		let mark = Handled { at, name: name.to_owned() };
		let found = match read {
			Ok(Some(report)) => Some(Found { id, source: Source::System, report }),
			Ok(None) => return,
			Err(e) if file_at > now - FRESH_S => {
				log::warn!("crash: the system's report {name} does not read yet ({e}): tried again at the next start");
				self.held = Some(self.held.map_or(at, |held| held.min(at)));
				return;
			}
			Err(e) => {
				log::warn!("crash: the system's report {name} skipped: {e}");
				None
			}
		};
		self.taken.push((mark, found));
	}

	/// The reports taken and the newest mark, none at or after a report that does not read yet: those wait for the
	/// next start with it, so the mark never passes one still being written.
	fn settle(mut self) -> (Vec<Found>, Option<Handled>) {
		if let Some(held) = self.held {
			let before = self.taken.len();
			self.taken.retain(|(mark, _)| mark.at < held);
			let waiting = before - self.taken.len();
			if waiting > 0 {
				log::info!("crash: {waiting} of the system's reports wait for the next start, after one still written");
			}
		}
		let newest = self.taken.iter().map(|(mark, _)| mark).max().cloned();
		(self.taken.into_iter().filter_map(|(_, found)| found).collect(), newest)
	}
}

/// macOS's own reports of a crash of this process (`~/Library/Logs/DiagnosticReports/<process name>-*.ips`, the process
/// name the program's file's), newer than `handled` and written from `since` on, and the newest of them.
#[cfg(target_os = "macos")]
fn system(origin: &Origin, handled: Option<&Handled>, since: i64, now: i64) -> (Vec<Found>, Option<Handled>) {
	let process = std::env::current_exe()
		.ok()
		.and_then(|p| p.file_name().and_then(|n| n.to_str()).map(str::to_owned))
		.unwrap_or_else(|| "questpump-helper".to_owned()); // copy-lint: not shown (a program's name)
	let Some(dir) = dirs::home_dir().map(|h| h.join("Library/Logs/DiagnosticReports")) else {
		return (Vec::new(), None);
	};
	let listed = match std::fs::read_dir(&dir) {
		Ok(listed) => listed,
		Err(e) if e.kind() == std::io::ErrorKind::NotFound => return (Vec::new(), None),
		Err(e) => {
			log::warn!("crash: the system's reports could not be listed ({e})");
			return (Vec::new(), None);
		}
	};
	let mut looked = Looked::default();
	let prefix = format!("{process}-");
	for path in listed.flatten().map(|entry| entry.path()) {
		let Some(name) = path.file_name().and_then(|n| n.to_str()).map(str::to_owned) else { continue };
		let Some(stem) = name.strip_prefix(&prefix).and_then(|n| n.strip_suffix(".ips")) else { continue };
		let Some(file_at) = written_at(&path).filter(|t| *t >= since) else { continue };
		let read = std::fs::read(&path)
			.map_err(|e| e.to_string())
			.and_then(|bytes| Report::ips(origin, &process, &bytes).map_err(|e| e.to_string()));
		let id = format!("ips-{stem}");
		looked.take(read, &name, id, file_at, now, handled);
	}
	looked.settle()
}

/// Windows Error Reporting's reports of a crash of this program, in the user's store
/// (`%LOCALAPPDATA%\Microsoft\Windows\WER`) and the machine's (`%ProgramData%\Microsoft\Windows\WER`), each
/// `ReportArchive` and `ReportQueue` (`AppCrash_<the program's name, cut short>_<...>\Report.wer`, the name in the file
/// checked against the program's; a report in two of them, queued and then archived, is read once), newer than
/// `handled` and written from `since` on, and the newest of them. A folder the user may not read gets a log line.
#[cfg(windows)]
fn system(origin: &Origin, handled: Option<&Handled>, since: i64, now: i64) -> (Vec<Found>, Option<Handled>) {
	let exe = std::env::current_exe()
		.ok()
		.and_then(|p| p.file_name().and_then(|n| n.to_str()).map(str::to_owned))
		.unwrap_or_else(|| "questpump-helper.exe".to_owned()); // copy-lint: not shown (a program's name)
	// the same environment variable `install.rs` reads the Battle.net Agent's folder from
	let stores = [dirs::data_local_dir(), std::env::var_os("ProgramData").map(PathBuf::from)];
	let (mut looked, mut seen) = (Looked::default(), std::collections::HashSet::new());
	for base in stores.into_iter().flatten() {
		for kept in ["ReportArchive", "ReportQueue"] {
			let dir = base.join(r"Microsoft\Windows\WER").join(kept);
			let listed = match std::fs::read_dir(&dir) {
				Ok(listed) => listed,
				Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
				Err(e) => {
					log::warn!("crash: {} could not be listed ({e})", dir.display());
					continue;
				}
			};
			for folder in listed.flatten().map(|entry| entry.path()) {
				let Some(name) = folder.file_name().and_then(|n| n.to_str()).map(str::to_owned) else { continue };
				// the program's name, cut short, then an underscore: one this program's name starts with
				let Some(cut) = name.strip_prefix("AppCrash_").and_then(|rest| rest.split('_').next()) else {
					continue;
				};
				if cut.is_empty() || !exe.to_ascii_lowercase().starts_with(&cut.to_ascii_lowercase()) {
					continue;
				}
				let file = folder.join("Report.wer");
				let file_at = match std::fs::metadata(&file).and_then(|m| m.modified()) {
					Ok(modified) => {
						modified.duration_since(UNIX_EPOCH).ok().and_then(|d| i64::try_from(d.as_secs()).ok())
					}
					Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
					Err(e) => {
						log::warn!("crash: {} could not be read ({e})", folder.display());
						continue;
					}
				};
				let Some(file_at) = file_at.filter(|t| *t >= since) else { continue };
				if !seen.insert(name.clone()) {
					continue;
				}
				let read = std::fs::read(&file)
					.map_err(|e| e.to_string())
					.and_then(|bytes| Report::wer(origin, &exe, &bytes, Some(file_at)).map_err(|e| e.to_string()));
				let id = format!("wer-{name}");
				looked.take(read, &name, id, file_at, now, handled);
			}
		}
	}
	looked.settle()
}
