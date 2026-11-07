// The browser's imports of the engine and the desk (an Emscripten JS library, linked by build.rs): the clock, the time
// zone and the report the engine's host asks for (questpump-engine `Clock` and `report`), a message to the page, the
// ask to keep the data folder and the sync's calls to the network, each the function of the same name the module's host
// passed in its options when it made the module (Module.qpNow, Module.qpOffsetAt, Module.qpReport, Module.qpPost,
// Module.qpPersist, Module.qpNet), so the Web Worker that holds the engine decides them, else the browser's own (its
// clock, its zone, the console) or nothing. None throws: an exception thrown here would unwind through the engine's Lua
// mid-call, which no Rust catch sees, so a host's function that throws is logged and the browser's own answers.

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
	// an error the engine's files or the desk reported and went on from, as UTF-8 at ptr, len bytes
	qp_report__deps: ["$qpAsk", "$UTF8ToString"],
	qp_report: (ptr, len) => {
		qpAsk("qpReport", (text) => console.error(`questpump-web: ${text}`), UTF8ToString(ptr, len));
	},
	// a message to the page (JSON as UTF-8 at ptr, len bytes): a topic's change, a sound, a take, a stop
	qp_post__deps: ["$qpAsk", "$UTF8ToString"],
	qp_post: (ptr, len) => {
		qpAsk("qpPost", () => {}, UTF8ToString(ptr, len));
	},
	// the data folder changed: the host keeps it in IndexedDB once the call is over
	qp_persist__deps: ["$qpAsk"],
	qp_persist: () => {
		qpAsk("qpPersist", () => {});
	},
	// a call to the network (JSON as UTF-8 at ptr, len bytes): the sync's fetch or its live socket, whose answers come
	// back through the door (qp_answer, qp_live) once this call is over; without a host's, each answers no answer
	qp_net__deps: ["$qpAsk", "$UTF8ToString"],
	qp_net: (ptr, len) => {
		qpAsk("qpNet", () => {}, UTF8ToString(ptr, len));
	},
});
