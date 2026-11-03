//! The desktop app's crash reports (contract/crash-api.yaml, "The desktop app"): the opt-in regions, the report the
//! panic hook writes and the one built at the next start from the system's own report of a crash of QuestPump (macOS's
//! `.ips`, Windows Error Reporting's `Report.wer`), each a text the player can read whole, and the body posted to
//! `ROUTE` through the outbox (`OUTBOX`). A report holds the build and its system, when and how it crashed and where:
//! it never reads the identifiers the system's reports carry (macOS's `crashReporterKey`, `userID`, `bootSessionUUID`,
//! `incident`, `procPath` and `parentProc`; Windows' `ReportIdentifier`, `IntegratorReportIdentifier`, `MachineName`
//! and the user's fields), so nothing names the player, the computer or the game, and the home folder's path becomes
//! `~` everywhere. No clock, file or platform call here: the app reads the files, the region and the time zone and
//! passes them in.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::feedback::{self, Invalid, invalid};
use crate::outbox::Rules;

/// The route the desktop app's crash reports go to (the phones' go through Crashlytics).
pub const ROUTE: &str = "https://questpump.com/api/crash";
/// A report's cap in Unicode scalar values (contract/crash-api.yaml `crash_report_max_chars`).
pub const REPORT_MAX_CHARS: usize = 12_000;
/// The largest body the route reads, in bytes (`crash_body_max_bytes`).
const BODY_MAX: usize = 16_384;
/// The most the body holds beside the report's text, in bytes: its keys and the version, the time, the kind and the
/// system (64 characters, six bytes each at worst once escaped).
const BODY_RESERVE: usize = 512;
/// The outbox keeps at most this many reports, each at most this many days from its `at` (`crash_outbox_max`,
/// `crash_outbox_days`).
pub const OUTBOX_MAX: usize = 5;
pub const OUTBOX_DAYS: i64 = 30;
/// The crash reports' outbox (`<data dir>/crash-outbox/`): 400 and 413 drop a report; every other answer but 204
/// keeps it.
pub static OUTBOX: Rules =
	Rules { route: ROUTE, folder: "crash-outbox", keep_days: OUTBOX_DAYS, keep_max: OUTBOX_MAX, refused: &[400, 413] };
/// The panic hook's reports in the data folder, one file a panic (`p-<unix ms>-<n>.json`, a `Report`), kept until the
/// switch or the player decides at the next start.
pub const FOLDER: &str = "crashes";
/// The `at` the route takes, unix seconds: 2024-01-01 to 2100-01-01.
const AT_MIN: i64 = 1_704_067_200;
const AT_MAX: i64 = 4_102_444_800;
/// The longest `os` the route takes, in characters.
const OS_MAX: usize = 64;
/// How long after a panic the system's report of QuestPump is the same crash: a panic where Rust cannot unwind (a
/// callback from the system's code) aborts the process, which the system records too; the panic's report says more.
const ABORT_AFTER_S: i64 = 10;
/// The room a cut list of frames keeps for the line that says so ("  ... 123 more cut" and its line break).
const CUT_RESERVE: usize = 32;
/// The longest signature the route takes, in ASCII characters.
const SIGNATURE_MAX: usize = 120;

/// contract/crash-api.yaml `x-opt-in-regions` (learnclash's `GdprRegion` of 2026-10-12 without the United Kingdom and
/// Switzerland, where both switches start on since 2026-10-13): the EU's 27 and the EEA's three others, then the EU's
/// outermost regions and Åland, where the GDPR applies under their own codes, then India and Canada (2026-10-12:
/// India's DPDP Act, and Quebec's Law 25, which a computer cannot tell apart from the rest of Canada), by ISO 3166-1
/// alpha-2 country code.
const COUNTRIES: &[&str] = &[
	"AT", "BE", "BG", "HR", "CY", "CZ", "DK", "EE", "FI", "FR", "DE", "GR", "HU", "IE", "IT", "LV", "LT", "LU", "MT",
	"NL", "PL", "PT", "RO", "SK", "SI", "ES", "SE", "IS", "LI", "NO", "AX", "GF", "GP", "MQ", "RE", "YT", "MF", "IN",
	"CA",
];
/// contract/crash-api.yaml `x-opt-in-regions`: the same regions by IANA time zone, with the Crown Dependencies' and
/// Gibraltar's (their own laws, the safe side) and Europe/Kiev and Europe/Kyiv, the zone Windows gives Finland,
/// Estonia, Latvia, Lithuania and Bulgaria through "FLE Standard Time"; India's and Canada's with the older names
/// tzdata keeps for them, Asia/Calcutta the one Windows gives India.
const TIME_ZONES: &[&str] = &[
	"Europe/Vienna",
	"Europe/Brussels",
	"Europe/Sofia",
	"Europe/Zagreb",
	"Asia/Nicosia",
	"Asia/Famagusta",
	"Europe/Prague",
	"Europe/Copenhagen",
	"Europe/Tallinn",
	"Europe/Helsinki",
	"Europe/Mariehamn",
	"Europe/Paris",
	"Europe/Berlin",
	"Europe/Busingen",
	"Europe/Athens",
	"Europe/Budapest",
	"Europe/Dublin",
	"Europe/Rome",
	"Europe/Riga",
	"Europe/Vilnius",
	"Europe/Luxembourg",
	"Europe/Malta",
	"Europe/Amsterdam",
	"Europe/Warsaw",
	"Europe/Lisbon",
	"Atlantic/Madeira",
	"Atlantic/Azores",
	"Europe/Bucharest",
	"Europe/Bratislava",
	"Europe/Ljubljana",
	"Europe/Madrid",
	"Africa/Ceuta",
	"Atlantic/Canary",
	"Europe/Stockholm",
	"Atlantic/Reykjavik",
	"Europe/Vaduz",
	"Europe/Oslo",
	"Arctic/Longyearbyen",
	"Europe/Guernsey",
	"Europe/Jersey",
	"Europe/Isle_of_Man",
	"Europe/Gibraltar",
	"America/Cayenne",
	"America/Guadeloupe",
	"America/Martinique",
	"Indian/Reunion",
	"Indian/Mayotte",
	"America/Marigot",
	"Europe/Kiev",
	"Europe/Kyiv",
	"Asia/Kolkata",
	"Asia/Calcutta",
	"America/St_Johns",
	"America/Halifax",
	"America/Glace_Bay",
	"America/Moncton",
	"America/Goose_Bay",
	"America/Blanc-Sablon",
	"America/Toronto",
	"America/Iqaluit",
	"America/Atikokan",
	"America/Winnipeg",
	"America/Resolute",
	"America/Rankin_Inlet",
	"America/Regina",
	"America/Swift_Current",
	"America/Edmonton",
	"America/Cambridge_Bay",
	"America/Inuvik",
	"America/Vancouver",
	"America/Creston",
	"America/Dawson_Creek",
	"America/Fort_Nelson",
	"America/Whitehorse",
	"America/Dawson",
	"Canada/Atlantic",
	"Canada/Central",
	"Canada/Eastern",
	"Canada/Mountain",
	"Canada/Newfoundland",
	"Canada/Pacific",
	"Canada/Saskatchewan",
	"Canada/Yukon",
	"America/Coral_Harbour",
	"America/Montreal",
	"America/Nipigon",
	"America/Pangnirtung",
	"America/Rainy_River",
	"America/Thunder_Bay",
	"America/Yellowknife",
];

/// Whether this computer is in an opt-in region, where Send crash reports starts not answered (off until the player
/// says yes): its region's country is one of the list's, or its time zone is, or the time zone names no place
/// (`names_place`; none or blank too: the safe side, since the zone is the sign of where the computer is). A country
/// that is not two ASCII letters (a region such as "150", Europe) names none; both lose the spaces, tabs and line ends
/// around them first.
#[must_use]
pub fn opt_in(country: Option<&str>, zone: Option<&str>) -> bool {
	let country = country
		.map(trimmed)
		.filter(|c| c.len() == 2 && c.bytes().all(|b| b.is_ascii_alphabetic()))
		.map(str::to_ascii_uppercase);
	let zone = zone.map_or("", trimmed);
	country.as_deref().is_some_and(|c| COUNTRIES.contains(&c)) || TIME_ZONES.contains(&zone) || !names_place(zone)
}

/// Whether an IANA time zone names a place, `Area/Location`: one without a slash (`UTC`, `GMT`, `CET`, an offset) or
/// under `Etc/` (`Etc/UTC`, `Etc/GMT+1`) says nothing of where the computer is (crash-api.yaml, "The opt-in regions").
fn names_place(zone: &str) -> bool {
	zone.contains('/') && !zone.starts_with("Etc/")
}

/// The text without the spaces, tabs and line ends around it, the same four characters on every platform.
fn trimmed(text: &str) -> &str {
	text.trim_matches([' ', '\t', '\n', '\r'])
}

/// How QuestPump crashed: a Rust panic the hook wrote, or a crash the system recorded.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
	Panic,
	Crash,
}

/// A crash report as it goes: the build that crashed (its version as About shows it), its system as "macOS 27.0, arm64"
/// names it, the crash's moment (unix seconds), its kind, the text the page shows whole, and where it crashed for the
/// mail's subject (`signature`: a panic's file and line, a crash's faulting module and offset; none from a panic file of
/// a build before it, or a report that names no place). The panic hook's file in `FOLDER` holds it as JSON.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Report {
	pub version: String,
	pub at: i64,
	pub kind: Kind,
	pub os: String,
	pub report: String,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub signature: Option<String>,
}

/// The body as the route reads it (contract/crash-api.yaml `CrashReport`), in its declaration order.
#[derive(Serialize)]
struct Body<'a> {
	source: &'static str,
	version: &'a str,
	at: i64,
	kind: Kind,
	os: &'a str,
	report: &'a str,
	#[serde(skip_serializing_if = "Option::is_none")]
	signature: Option<&'a str>,
}

/// Who writes a report: the build, its system (the route's `os`) and the home folder, whose path becomes `~`.
pub struct Origin<'a> {
	pub version: &'a str,
	pub os: &'a str,
	pub home: Option<&'a str>,
}

/// A Rust panic as the hook sees it: when, on which thread, what it said, where, and the backtrace as the standard
/// library prints it (`std::backtrace::Backtrace`'s `Display`).
pub struct Panic<'a> {
	pub at: i64,
	pub thread: &'a str,
	pub message: &'a str,
	pub location: &'a str,
	pub backtrace: &'a str,
}

/// The newest of the system's crash reports of QuestPump handled at a start (taken, deleted by the player's answer or
/// skipped), by its time and its file's or folder's name: none at or before its time is offered again, whatever its name
/// (Windows keeps a crash's report under a new folder's name once it moves it from the queue into the archive).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Handled {
	pub at: i64,
	pub name: String,
}

/// Whether the system's report of a crash at `system_at` is the abort a panic at `panic_at` ended in.
#[must_use]
pub fn same_crash(panic_at: i64, system_at: i64) -> bool {
	(panic_at - 1..=panic_at + ABORT_AFTER_S).contains(&system_at)
}

impl Report {
	/// A panic's report: the thread's name, the message, the file and line, then the backtrace's frames (function names
	/// alone, no addresses), from the panic itself: the hook's own frames before it are left out where the frames name
	/// their functions (macOS; on Windows, whose frames are modules and offsets, they stay).
	#[must_use]
	pub fn panic(origin: &Origin, p: &Panic) -> Self {
		let fixed =
			vec![format!("Thread: {}", p.thread), format!("Message: {}", p.message), format!("At: {}", p.location)];
		let frames = panic_frames(p.backtrace);
		let report = text(origin, p.at, Kind::Panic, fixed, vec![("Backtrace:".to_owned(), frames)]);
		// the file and line, without the column
		let place = match p.location.rsplit_once(':') {
			Some((place, column)) if !column.is_empty() && column.bytes().all(|b| b.is_ascii_digit()) => place,
			_ => p.location,
		};
		Self {
			version: origin.version.to_owned(),
			at: p.at,
			kind: Kind::Panic,
			os: cap(origin.os, OS_MAX),
			report,
			signature: subject_place(place, origin.home),
		}
	}

	/// The report of a crash macOS recorded in `bytes`, an `.ips` file (its first line a JSON header, then the body's
	/// JSON; Apple's "Interpreting the JSON format of a crash report"): the exception, its signal and codes, the
	/// termination's reason, the application specific information, the faulting module and offset, then the faulting
	/// thread's frames (each image's name, its symbol when the report names one, the offset) and the last exception's
	/// backtrace when there is one. None for another process's report (`process` is QuestPump's).
	///
	/// # Errors
	/// The file does not read as a crash report (`bug_type` 309) with a time.
	pub fn ips(origin: &Origin, process: &str, bytes: &[u8]) -> Result<Option<Self>, Invalid> {
		let text_of = String::from_utf8_lossy(bytes);
		let (first, rest) = text_of.split_once('\n').ok_or_else(|| invalid("no header line"))?;
		let head: IpsHead = serde_json::from_str(first).map_err(|e| invalid(format!("the header: {e}")))?;
		let body: IpsBody = serde_json::from_str(rest).map_err(|e| invalid(format!("the body: {e}")))?;
		let name = head.name.as_deref().or(body.proc_name.as_deref());
		if name != Some(process) {
			return Ok(None);
		}
		if head.bug_type.as_deref() != Some("309") {
			return Err(invalid(format!("not a crash report (bug_type {})", head.bug_type.unwrap_or_default())));
		}
		let at = body
			.capture_time
			.as_deref()
			.or(head.timestamp.as_deref())
			.and_then(apple_time)
			.ok_or_else(|| invalid("no time"))?;
		let version = head.app_version.as_deref().filter(|v| feedback::version(v)).unwrap_or(origin.version);
		let os = match (body.os_version.as_ref().and_then(|o| o.train.as_deref()), body.cpu_type.as_deref()) {
			(Some(train), Some(cpu)) => format!("{train}, {}", apple_arch(cpu)),
			(Some(train), None) => train.to_owned(),
			_ => origin.os.to_owned(),
		};
		let (fixed, tails) = body.lines();
		let report = text(&Origin { version, os: &os, home: origin.home }, at, Kind::Crash, fixed, tails);
		let place = body.faulting().map(|(image, offset)| match offset {
			Some(offset) => format!("{image}+{offset:#x}"),
			None => image.to_owned(),
		});
		Ok(Some(Self {
			version: version.to_owned(),
			at,
			kind: Kind::Crash,
			os: cap(&os, OS_MAX),
			report,
			signature: place.and_then(|p| subject_place(&p, origin.home)),
		}))
	}

	/// The report of a crash Windows Error Reporting recorded in `bytes`, a `Report.wer` (UTF-16 `key=value` lines):
	/// the event's type, then its signature (`Sig[*]`: the application, the faulting module, the exception's code and
	/// offset), at its `EventTime` (a `FILETIME`), else at `file_at`, of the version its signature names (the
	/// application's version, its first three parts), else this build's. None for another program's report (`exe` is
	/// QuestPump's, as `questpump-helper.exe`).
	///
	/// # Errors
	/// The file names no application, or no time is known.
	pub fn wer(origin: &Origin, exe: &str, bytes: &[u8], file_at: Option<i64>) -> Result<Option<Self>, Invalid> {
		let fields = wer_fields(&wide_text(bytes));
		let field = |key: &str| fields.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str());
		let mut signature: BTreeMap<u32, (Option<&str>, Option<&str>)> = BTreeMap::new();
		for (key, value) in &fields {
			let Some((n, part)) = key.strip_prefix("Sig[").and_then(|k| k.split_once("].")) else { continue };
			let Ok(n) = n.parse::<u32>() else { continue };
			let entry = signature.entry(n).or_default();
			match part {
				"Name" => entry.0 = Some(value.as_str()),
				"Value" => entry.1 = Some(value.as_str()),
				_ => {}
			}
		}
		let signed =
			|label: &str| signature.values().find(|(name, _)| *name == Some(label)).and_then(|(_, value)| *value);
		let app =
			signed("Application Name").or_else(|| field("NsAppName")).ok_or_else(|| invalid("no application named"))?;
		if !app.eq_ignore_ascii_case(exe) {
			return Ok(None);
		}
		// the version resource's four parts ("0.35.0.0") as About shows the version, its first three
		let version = signed("Application Version")
			.map(|v| v.trim().split('.').take(3).collect::<Vec<_>>().join("."))
			.filter(|v| feedback::version(v));
		let version = version.as_deref().unwrap_or(origin.version);
		let at = field("EventTime")
			.and_then(|t| t.trim().parse::<i64>().ok())
			.map(|ticks| ticks / 10_000_000 - 11_644_473_600)
			.filter(|t| (AT_MIN..=AT_MAX).contains(t))
			.or(file_at)
			.ok_or_else(|| invalid("no time"))?;
		let mut fixed = Vec::new();
		if let Some(event) = field("EventType") {
			fixed.push(format!("Event: {event}"));
		}
		for (name, value) in signature.values() {
			if let (Some(name), Some(value)) = (name, value) {
				fixed.push(format!("{name}: {value}"));
			}
		}
		let report = text(&Origin { version, ..*origin }, at, Kind::Crash, fixed, Vec::new());
		// the faulting module and the exception's offset into it (WER writes it as bare hex digits)
		let place = signed("Fault Module Name").map(|module| match signed("Exception Offset") {
			Some(offset) => match u64::from_str_radix(offset.trim().trim_start_matches("0x"), 16) {
				Ok(n) => format!("{module}+{n:#x}"),
				Err(_) => format!("{module}+{offset}"),
			},
			None => module.to_owned(),
		});
		Ok(Some(Self {
			version: version.to_owned(),
			at,
			kind: Kind::Crash,
			os: cap(origin.os, OS_MAX),
			report,
			signature: place.and_then(|p| subject_place(&p, origin.home)),
		}))
	}

	/// The body's JSON as it is posted.
	///
	/// # Errors
	/// A field out of the route's bounds (a version it refuses, a clock far off, an empty system or report) or a body
	/// over its 16,384 bytes: a report the route would refuse is never kept.
	pub fn body(&self) -> Result<Vec<u8>, Invalid> {
		if !feedback::version(&self.version) {
			return Err(invalid("version"));
		}
		if !(AT_MIN..=AT_MAX).contains(&self.at) {
			return Err(invalid("at: the computer's clock is far off"));
		}
		if !(1..=OS_MAX).contains(&self.os.chars().count()) {
			return Err(invalid("os"));
		}
		if !(1..=REPORT_MAX_CHARS).contains(&self.report.chars().count()) {
			return Err(invalid("report"));
		}
		let body = Body {
			source: "helper",
			version: &self.version,
			at: self.at,
			kind: self.kind,
			os: &self.os,
			report: &self.report,
			signature: self.signature.as_deref(),
		};
		let bytes = serde_json::to_vec(&body).map_err(|e| invalid(e.to_string()))?;
		if bytes.len() > BODY_MAX {
			return Err(invalid(format!("{} bytes, over the route's {BODY_MAX}", bytes.len())));
		}
		Ok(bytes)
	}
}

/// The `.ips` header's fields a report reads.
#[derive(Deserialize, Default)]
#[serde(default)]
struct IpsHead {
	bug_type: Option<String>,
	name: Option<String>,
	app_version: Option<String>,
	timestamp: Option<String>,
}

/// The `.ips` body's fields a report reads (never an identifier: those are left unread).
#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct IpsBody {
	proc_name: Option<String>,
	capture_time: Option<String>,
	cpu_type: Option<String>,
	os_version: Option<IpsOs>,
	exception: Option<IpsException>,
	termination: Option<IpsTermination>,
	asi: Option<BTreeMap<String, Value>>,
	asi_backtraces: Option<Vec<String>>,
	last_exception_backtrace: Option<Vec<IpsFrame>>,
	faulting_thread: Option<usize>,
	threads: Vec<IpsThread>,
	used_images: Vec<IpsImage>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct IpsOs {
	train: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct IpsException {
	#[serde(rename = "type")]
	kind: Option<String>,
	signal: Option<String>,
	codes: Option<String>,
	subtype: Option<String>,
	message: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct IpsTermination {
	namespace: Option<String>,
	code: Option<i64>,
	indicator: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct IpsThread {
	triggered: Option<bool>,
	queue: Option<String>,
	name: Option<String>,
	frames: Vec<IpsFrame>,
}

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct IpsFrame {
	image_index: Option<usize>,
	image_offset: Option<u64>,
	symbol: Option<String>,
	symbol_location: Option<u64>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct IpsImage {
	name: Option<String>,
}

impl IpsBody {
	/// The report's lines: the exception, the termination, the application specific information and the faulting
	/// module, then the faulting thread's frames and the last exception's backtrace, each list cut at its end.
	fn lines(&self) -> (Vec<String>, Vec<(String, Vec<String>)>) {
		let mut fixed = Vec::new();
		if let Some(e) = &self.exception {
			match (&e.kind, &e.signal) {
				(Some(kind), Some(signal)) => fixed.push(format!("Exception: {kind} ({signal})")),
				(Some(one), None) | (None, Some(one)) => fixed.push(format!("Exception: {one}")),
				(None, None) => {}
			}
			let more =
				[("Exception codes", &e.codes), ("Exception subtype", &e.subtype), ("Exception message", &e.message)];
			fixed.extend(more.iter().filter_map(|(label, v)| v.as_ref().map(|v| format!("{label}: {v}"))));
		}
		if let Some(t) = &self.termination {
			let code = t.code.map(|c| format!(" {c:#x}")).unwrap_or_default();
			let indicator = t.indicator.as_ref().map(|i| format!(", {i}")).unwrap_or_default();
			fixed.push(format!("Termination: {}{code}{indicator}", t.namespace.as_deref().unwrap_or("?")));
		}
		for (image, said) in self.asi.iter().flatten() {
			let lines: Vec<&str> = match said {
				Value::String(s) => vec![s.as_str()],
				Value::Array(a) => a.iter().filter_map(Value::as_str).collect(),
				_ => Vec::new(),
			};
			fixed.extend(lines.into_iter().map(|line| format!("Application specific: {image}: {line}")));
		}
		let (index, thread) = self.crashed();
		if let Some((image, offset)) = self.faulting() {
			let offset = offset.map(|o| format!(" + {o:#x}")).unwrap_or_default();
			fixed.push(format!("Faulting module: {image}{offset}"));
		}
		let mut tails = Vec::new();
		if let (Some(i), Some(t)) = (index, thread) {
			let on = t.queue.as_ref().or(t.name.as_ref()).map(|q| format!(" ({q})")).unwrap_or_default();
			tails.push((format!("Thread {i} crashed{on}:"), self.frames(&t.frames)));
		}
		let last = match (&self.last_exception_backtrace, &self.asi_backtraces) {
			(Some(frames), _) if !frames.is_empty() => self.frames(frames),
			(_, Some(lines)) => {
				lines.iter().flat_map(|l| l.lines()).map(no_addresses).filter(|l| !l.is_empty()).collect()
			}
			_ => Vec::new(),
		};
		if !last.is_empty() {
			tails.push(("Last exception backtrace:".to_owned(), last));
		}
		(fixed, tails)
	}

	/// The crashed thread's index and the thread, as the report names it (`faultingThread`, else the triggered one).
	fn crashed(&self) -> (Option<usize>, Option<&IpsThread>) {
		let index = self.faulting_thread.or_else(|| self.threads.iter().position(|t| t.triggered == Some(true)));
		(index, index.and_then(|i| self.threads.get(i)))
	}

	/// The faulting module and the offset into it: the crashed thread's top frame's.
	fn faulting(&self) -> Option<(&str, Option<u64>)> {
		let top = self.crashed().1?.frames.first()?;
		Some((self.image(top), top.image_offset))
	}

	/// A frame's image by its index, "???" when the report names none.
	fn image(&self, frame: &IpsFrame) -> &str {
		frame.image_index.and_then(|i| self.used_images.get(i)).and_then(|image| image.name.as_deref()).unwrap_or("???")
	}

	/// Frames as lines: the number, the image, the offset into it, and the symbol with its distance when named.
	fn frames(&self, frames: &[IpsFrame]) -> Vec<String> {
		frames
			.iter()
			.enumerate()
			.map(|(i, f)| {
				let offset = f.image_offset.map(|o| format!("  {o:#x}")).unwrap_or_default();
				let symbol = match (&f.symbol, f.symbol_location) {
					(Some(s), Some(at)) => format!("  {s} + {at}"),
					(Some(s), None) => format!("  {s}"),
					(None, _) => String::new(),
				};
				format!("{i:>3}  {}{offset}{symbol}", self.image(f))
			})
			.collect()
	}
}

/// An `.ips` time ("2026-10-09 14:51:08.5572 -0600") as unix seconds.
fn apple_time(t: &str) -> Option<i64> {
	let mut parts = t.split_whitespace();
	let (date, clock, offset) = (parts.next()?, parts.next()?, parts.next()?);
	let mut d = date.split('-').map(str::parse::<i32>);
	let (year, month, day) = (d.next()?.ok()?, d.next()?.ok()?, d.next()?.ok()?);
	let mut c = clock.split('.').next()?.split(':').map(str::parse::<u8>);
	let (hour, minute, second) = (c.next()?.ok()?, c.next()?.ok()?, c.next()?.ok()?);
	let sign: i8 = if offset.starts_with('-') { -1 } else { 1 };
	let digits = offset.trim_start_matches(['+', '-']);
	let (oh, om) = (digits.get(..2)?.parse::<i8>().ok()?, digits.get(2..4)?.parse::<i8>().ok()?);
	let month = time::Month::try_from(u8::try_from(month).ok()?).ok()?;
	let date = time::Date::from_calendar_date(year, month, u8::try_from(day).ok()?).ok()?;
	let clock = time::Time::from_hms(hour, minute, second).ok()?;
	let offset = time::UtcOffset::from_hms(sign * oh, sign * om, 0).ok()?;
	Some(time::PrimitiveDateTime::new(date, clock).assume_offset(offset).unix_timestamp())
}

/// An `.ips` `cpuType` as the route's `os` names an architecture.
fn apple_arch(cpu: &str) -> &str {
	match cpu {
		"ARM-64" => "arm64",
		"X86-64" => "x86_64",
		other => other,
	}
}

/// A `Report.wer`'s text: UTF-16 (little-endian, else big-endian by its mark), or UTF-8 for a file without one.
fn wide_text(bytes: &[u8]) -> String {
	let units = |big: bool, from: usize| -> Vec<u16> {
		bytes
			.get(from..)
			.unwrap_or_default()
			.as_chunks::<2>()
			.0
			.iter()
			.map(|&pair| if big { u16::from_be_bytes(pair) } else { u16::from_le_bytes(pair) })
			.collect()
	};
	match bytes {
		[0xFF, 0xFE, ..] => String::from_utf16_lossy(&units(false, 2)),
		[0xFE, 0xFF, ..] => String::from_utf16_lossy(&units(true, 2)),
		[_, 0, ..] => String::from_utf16_lossy(&units(false, 0)),
		_ => String::from_utf8_lossy(bytes).into_owned(),
	}
}

/// A `Report.wer`'s `key=value` lines, in their order.
fn wer_fields(text: &str) -> Vec<(String, String)> {
	text.lines()
		.filter_map(|line| line.split_once('='))
		.map(|(k, v)| (k.trim().to_owned(), v.trim().to_owned()))
		.collect()
}

/// The frames of a backtrace as `std::backtrace::Backtrace` prints it: each frame's function name, numbered from the
/// panic on (the frames up to the standard library's `__rust_end_short_backtrace` are the panic's own machinery), its
/// `at <file>:<line>` lines and any address left out.
fn panic_frames(backtrace: &str) -> Vec<String> {
	let names: Vec<String> = backtrace
		.lines()
		.filter_map(|line| {
			let (number, name) = line.trim().split_once(": ")?;
			number.parse::<u32>().ok().map(|_| no_addresses(name))
		})
		.collect();
	let from = names.iter().rposition(|n| n.contains("__rust_end_short_backtrace")).map_or(0, |i| i + 1);
	names.get(from..).unwrap_or_default().iter().enumerate().map(|(i, n)| format!("{i:>3}  {n}")).collect()
}

/// A line without the addresses in it (a `0x` number of six digits or more) and with its runs of spaces made one.
fn no_addresses(line: &str) -> String {
	line.split_whitespace()
		.filter(|word| {
			let hex = word.strip_prefix("0x").unwrap_or_default();
			hex.len() < 6 || !hex.bytes().all(|b| b.is_ascii_hexdigit())
		})
		.collect::<Vec<_>>()
		.join(" ")
}

/// A line with the home folder's path as `~`, in every spelling a path takes. A Windows home (one with a backslash, as
/// `C:\Users\Name`) matches in any case, its separators single backslashes, the doubled ones Rust's debug format
/// writes or forward slashes, with or without the verbatim prefix `\\?\` (`\\\\?\\` in the debug format, `//?/`);
/// any other home (macOS's `/Users/name`) matches exactly. A longer name that starts with the home's
/// (`C:\Users\Name2`) is cut too: a report says less, never more.
fn home_free(line: &str, home: &str) -> String {
	if !home.contains('\\') {
		return line.replace(home, "~");
	}
	let bare = VERBATIM.iter().find_map(|v| home.strip_prefix(v)).unwrap_or(home);
	let parts: Vec<&str> = bare.split(['\\', '/']).filter(|p| !p.is_empty()).collect();
	if parts.is_empty() {
		return line.to_owned();
	}
	let mut out = String::with_capacity(line.len());
	let mut rest = line;
	while let Some(c) = rest.chars().next() {
		let len = if let Some(len) = windows_home_at(rest, &parts) {
			out.push('~');
			len
		} else {
			out.push(c);
			c.len_utf8()
		};
		rest = rest.get(len..).unwrap_or_default();
	}
	out
}

/// The verbatim prefix of a Windows path, as written, in Rust's debug format and with forward slashes.
const VERBATIM: [&str; 3] = [r"\\?\", r"\\\\?\\", "//?/"];

/// The length in bytes of a Windows home's spelling at the start of `s` (`parts` its folders, as `C:`, `Users`,
/// `Name`), if one is there.
fn windows_home_at(s: &str, parts: &[&str]) -> Option<usize> {
	let mut rest = VERBATIM.iter().find_map(|v| s.strip_prefix(v)).unwrap_or(s);
	for (i, part) in parts.iter().enumerate() {
		if i > 0 {
			rest = rest.strip_prefix(r"\\").or_else(|| rest.strip_prefix(['\\', '/']))?;
		}
		rest = strip_any_case(rest, part)?;
	}
	Some(s.len() - rest.len())
}

/// `s` after `part`, compared in any case, if it starts with it.
fn strip_any_case<'a>(s: &'a str, part: &str) -> Option<&'a str> {
	let mut chars = s.chars();
	for want in part.chars() {
		if !chars.next()?.to_lowercase().eq(want.to_lowercase()) {
			return None;
		}
	}
	Some(chars.as_str())
}

/// A crash's place as the mail's subject carries it (contract/crash-api.yaml `signature`): the home folder's path as
/// `~`, each character outside printable ASCII as `?`, at most `SIGNATURE_MAX` characters, its end kept (a file's name
/// and line, a module's offset); none when nothing is left.
fn subject_place(place: &str, home: Option<&str>) -> Option<String> {
	let place = match home.filter(|h| h.len() > 1) {
		Some(h) => home_free(place, h),
		None => place.to_owned(),
	};
	let ascii: Vec<char> = place.trim().chars().map(|c| if (' '..='~').contains(&c) { c } else { '?' }).collect();
	let kept: String = ascii[ascii.len().saturating_sub(SIGNATURE_MAX)..].iter().collect();
	let kept = kept.trim();
	(!kept.is_empty()).then(|| kept.to_owned())
}

/// A text's first `max` Unicode scalar values.
fn cap(s: &str, max: usize) -> String {
	s.chars().take(max).collect()
}

/// A character's bytes in the body's JSON (`serde_json`'s escapes).
fn json_len(c: char) -> usize {
	match c {
		'"' | '\\' | '\n' | '\r' | '\t' | '\u{8}' | '\u{c}' => 2,
		c if c < ' ' => 6,
		c => c.len_utf8(),
	}
}

/// What is left of a report's room: Unicode scalar values (`REPORT_MAX_CHARS`) and the bytes the text may take in the
/// body's JSON.
struct Room {
	chars: usize,
	bytes: usize,
}

impl Room {
	/// Takes a line and the line break before it when both fit.
	fn take(&mut self, line: &str) -> bool {
		let chars = line.chars().count() + 1;
		let bytes = line.chars().map(json_len).sum::<usize>() + 2;
		if chars > self.chars || bytes > self.bytes {
			return false;
		}
		self.chars -= chars;
		self.bytes -= bytes;
		true
	}

	/// As much of a line as fits, after its line break.
	fn fit(&mut self, line: &str) -> String {
		let mut out = String::new();
		if self.chars < 1 || self.bytes < 2 {
			return out;
		}
		self.chars -= 1;
		self.bytes -= 2;
		for c in line.chars() {
			let bytes = json_len(c);
			if self.chars < 1 || self.bytes < bytes {
				break;
			}
			self.chars -= 1;
			self.bytes -= bytes;
			out.push(c);
		}
		out
	}
}

/// The report's text, in the contract's order: "QuestPump on this computer <version> (<os>)", "Crashed: <date> UTC",
/// "Kind: panic" or "Kind: crash", the `fixed` lines, then each list of frames under its title. The home folder's path
/// becomes `~` in every line (`home_free`). It holds at most `REPORT_MAX_CHARS` Unicode scalar values and fits the
/// route's body: the lists are cut at their ends first (each keeps its first frames, taken in turns, and says how many
/// were cut), then a line too long for what is left, with the lines after it.
fn text(origin: &Origin, at: i64, kind: Kind, fixed: Vec<String>, tails: Vec<(String, Vec<String>)>) -> String {
	let home = |line: String| match origin.home.filter(|h| h.len() > 1) {
		Some(h) => home_free(&line, h),
		None => line,
	};
	let crashed = time::OffsetDateTime::from_unix_timestamp(at)
		.ok()
		.and_then(|t| t.format(time::macros::format_description!("[year]-[month]-[day] [hour]:[minute]:[second]")).ok())
		.unwrap_or_else(|| at.to_string());
	let kind = match kind {
		Kind::Panic => "panic",
		Kind::Crash => "crash",
	};
	let head = [
		format!("QuestPump on this computer {} ({})", origin.version, origin.os),
		format!("Crashed: {crashed} UTC"),
		format!("Kind: {kind}"),
	];
	let tails: Vec<(String, Vec<String>)> = tails
		.into_iter()
		.filter(|(_, frames)| !frames.is_empty())
		.map(|(title, frames)| (home(title), frames.into_iter().map(home).collect()))
		.collect();
	// each list's blank line and title, and the line that says it was cut, are kept out of the room first (the first
	// line has no line break before it, hence the one more)
	let titles = tails.iter().map(|(t, _)| (t.chars().count() + 2, t.chars().map(json_len).sum::<usize>() + 4));
	let (title_chars, title_bytes) = titles.fold((0, 0), |(c, b), (tc, tb)| (c + tc, b + tb));
	let marks = tails.len() * CUT_RESERVE;
	let mut room = Room {
		chars: (REPORT_MAX_CHARS + 1).saturating_sub(title_chars + marks),
		bytes: (BODY_MAX - BODY_RESERVE + 2).saturating_sub(title_bytes + marks),
	};
	let mut lines: Vec<String> = Vec::new();
	for line in head.into_iter().chain(fixed).map(home) {
		if room.take(&line) {
			lines.push(line);
		} else {
			lines.push(room.fit(&line));
			break;
		}
	}
	// the frames in turns, the first of each list first, until a list's next one no longer fits
	let mut kept = vec![0; tails.len()];
	let mut open = vec![true; tails.len()];
	let deepest = tails.iter().map(|(_, frames)| frames.len()).max().unwrap_or(0);
	for depth in 0..deepest {
		for (i, (_, frames)) in tails.iter().enumerate() {
			let Some(frame) = frames.get(depth) else { continue };
			if open[i] && room.take(frame) {
				kept[i] += 1;
			} else {
				open[i] = false;
			}
		}
	}
	for ((title, frames), kept) in tails.iter().zip(kept) {
		lines.push(String::new());
		lines.push(title.clone());
		lines.extend(frames.iter().take(kept).cloned());
		if kept < frames.len() {
			lines.push(format!("  ... {} more cut", frames.len() - kept));
		}
	}
	lines.join("\n")
}
