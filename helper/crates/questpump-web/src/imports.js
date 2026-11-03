// The browser's imports of the engine (an Emscripten JS library, linked by build.rs): the clock, the time zone and the
// report the engine's host asks for (questpump-engine `Clock` and `report`), each the function of the same name the
// module's host passed in its options when it made the module (Module.qpNow, Module.qpOffsetAt, Module.qpReport), so the
// Web Worker that holds the engine decides them, else the browser's own (its clock, its zone, the console). None
// throws: an exception thrown here would unwind through the engine's Lua mid-call, which no Rust catch sees, so a
// host's function that throws is logged and the browser's own answers.

addToLibrary({
	$qpAsk: (name, own, ...args) => {
		const host = Module[name];
		if (typeof host === "function") {
			try {
				return host(...args);
			} catch (error) {
				console.error(`questpump-web: ${name} threw`, error);
			}
		}
		return own(...args);
	},
	// the unix time now, in seconds
	qp_now__deps: ["$qpAsk"],
	qp_now: () => qpAsk("qpNow", () => Date.now() / 1000),
	// the seconds east of UTC at the unix time t (the player's zone; game days begin at 04:00 local)
	qp_offset_at__deps: ["$qpAsk"],
	qp_offset_at: (t) => qpAsk("qpOffsetAt", (at) => -new Date(at * 1000).getTimezoneOffset() * 60, t),
	// an error the engine's files reported and went on from, as UTF-8 at ptr, len bytes
	qp_report__deps: ["$qpAsk", "$UTF8ToString"],
	qp_report: (ptr, len) => {
		qpAsk("qpReport", (text) => console.error(`questpump-web: ${text}`), UTF8ToString(ptr, len));
	},
});
