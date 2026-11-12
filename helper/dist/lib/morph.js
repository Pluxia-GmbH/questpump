// Parts of the page taking a new place in one move (2026-11-11): where a drawing changes its layout under the player's
// eyes (the dial giving way to a move's rows on its pour), the parts that carry a `view-transition-name`
// (views/pump/pump.css, each name on one drawn element at a time, or the web view skips the motion and only draws the
// change) travel from their place to the new one through the web view's own view transition
// (document.startViewTransition: WebView2, WKWebView from Safari 18, the browsers since October 2025), and the rest of
// the page stays as it is, live and clickable. Where the web view has none, and under Reduce Motion, the change is
// drawn at once. A stage's change into the next is lib/stage.js's fade, never this.

const still = window.matchMedia("(prefers-reduced-motion: reduce)");

/** The longest a change waits for the frame in which the web view captures the page before it (ms): a window the
 * system keeps from drawing gives none, and the change must not wait for one. */
const CAPTURE_WAIT = 120;

/** The class a move sets on the page's root while it runs (pump.css: the page itself takes no part in it). */
const MOVING = "qp-morph";

let latest = null;

/** Draws `change` as one move of the named parts; the promise it returns settles once the change is drawn (the motion
 * still running then) and rejects with what `change` threw. `change` runs once, in a later task where the web view
 * draws the move, and a move that starts while another runs ends that one where its change put it. */
export function morph(change) {
	if (still.matches || !document.startViewTransition) {
		return new Promise((resolve) => resolve(change()));
	}
	const root = document.documentElement;
	root.classList.add(MOVING);
	let drawn = false;
	const move = document.startViewTransition(() => {
		drawn = true;
		change();
	});
	latest = move;
	const late = setTimeout(() => {
		if (!drawn) move.skipTransition();
	}, CAPTURE_WAIT);
	// a move the web view skips (a hidden window, a newer move, a name on two elements) still draws its change, and
	// what the change threw comes with the promise returned: neither is said twice
	move.ready.catch(() => {});
	move.finished.catch(() => {}).then(() => {
		clearTimeout(late);
		if (latest !== move) return;
		latest = null;
		root.classList.remove(MOVING);
	});
	return move.updateCallbackDone;
}
