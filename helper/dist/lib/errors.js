// A page's own errors into QuestPump's log (contract/crash-api.yaml, "The desktop app", "Kept alive"): an uncaught
// error or an unhandled rejection in this window goes to Rust's `page_error`, which writes one line to helper.log (its
// window, its kind, its message and where), never sent anywhere. At most ERRORS_MAX a minute leave the page (Rust
// holds its own limit too), so a page that fails in a loop never floods the log. The listeners go on as this module
// evaluates: each page's entry module imports it first (app.js, card.js), so they listen before the kit, the views or
// anything else the page imports evaluates (only ./bridge.js, which this module calls through, comes before it), and
// a module that throws as it evaluates leaves its line in the log.

import {call} from "./bridge.js";

/** The most errors the page hands over in a minute. */
const ERRORS_MAX = 10;
const MINUTE_MS = 60_000;

let since = 0;
let count = 0;

/** Hands one error to Rust while the minute's count allows. */
function report(kind, message, place) {
	const now = Date.now();
	if (now - since >= MINUTE_MS) {
		since = now;
		count = 0;
	}
	count += 1;
	if (count > ERRORS_MAX) return;
	// a failed report has nowhere left to go
	call("page_error", {kind, message: String(message ?? ""), place: String(place ?? "")}).catch(() => {});
}

window.addEventListener("error", (e) => {
	const place = e.filename ? `${e.filename}:${e.lineno}:${e.colno}` : "";
	report("error", e.message || e.error, place);
});
window.addEventListener("unhandledrejection", (e) => {
	const reason = e.reason;
	report("rejection", reason instanceof Error ? `${reason.name}: ${reason.message}` : reason, reason?.stack ?? "");
});
