//! The Web Worker's imports (`src/imports.js`, linked by build.rs), each wrapped once: the clock and the zone the
//! engine and the desk read, the report the engine's files and the desk's log lines go to, a message to the page, the
//! ask to keep the data folder and a call to the network. None throws (`$qpAsk`): a JavaScript exception would unwind through the engine's
//! Lua mid-call, where no Rust catch sees it.

// SAFETY: src/imports.js defines the six with these signatures (numbers as f64; a pointer and a byte count; none).
unsafe extern "C" {
	safe fn qp_now() -> f64;
	safe fn qp_offset_at(t: f64) -> f64;
	fn qp_report(text: *const u8, len: usize);
	fn qp_post(text: *const u8, len: usize);
	safe fn qp_persist();
	fn qp_net(text: *const u8, len: usize);
}

/// Now, in unix seconds with their fraction, by the host's clock.
#[must_use]
pub fn now_s() -> f64 {
	qp_now()
}

/// The offset from UTC at unix second `t`, in whole seconds east, by the host's zone.
#[must_use]
#[expect(clippy::cast_possible_truncation, reason = "an offset in whole seconds, within a day")]
#[expect(clippy::cast_precision_loss, reason = "a unix time in seconds is exact in an f64 (below 2^53)")]
pub fn offset_at(t: i64) -> i64 {
	qp_offset_at(t as f64).round() as i64
}

/// An error the engine or the desk went on from, for the Worker's console.
pub fn report(text: &str) {
	// SAFETY: the pointer and length are the borrowed text's own bytes, read by the import before it returns
	unsafe { qp_report(text.as_ptr(), text.len()) }
}

/// One message to the page, as JSON text.
pub fn post(text: &str) {
	// SAFETY: the pointer and length are the borrowed text's own bytes, read by the import before it returns
	unsafe { qp_post(text.as_ptr(), text.len()) }
}

/// The data folder changed: the Worker copies its changed files into `IndexedDB` once the call is over.
pub fn persist() {
	qp_persist();
}

/// One call to the network, as JSON text: the Worker sends it and its answer comes back through the door.
pub fn net(text: &str) {
	// SAFETY: the pointer and length are the borrowed text's own bytes, read by the import before it returns
	unsafe { qp_net(text.as_ptr(), text.len()) }
}

/// The `log` crate's lines (the desk's and the core's `log::warn!`) to the Worker's console through `report`, as the
/// desktop app's go to `helper.log`.
struct Console;

impl log::Log for Console {
	fn enabled(&self, metadata: &log::Metadata) -> bool {
		metadata.level() <= log::Level::Info
	}

	fn log(&self, record: &log::Record) {
		if self.enabled(record.metadata()) {
			report(&format!("{} {}", record.level(), record.args()));
		}
	}

	fn flush(&self) {}
}

static CONSOLE: Console = Console;

/// Sends the `log` lines to the console from now on (once: a second call keeps the first).
pub fn log_to_console() {
	if log::set_logger(&CONSOLE).is_ok() {
		log::set_max_level(log::LevelFilter::Info);
	}
}
