// Focus and what a screen reader hears (docs/ui.md, "The pump on the computer" and "What stays native, and the kit"):
// one polite live region for the page (index.html's #qp-live), which says a moment once (a copied line, the Result's
// one announcement, a pump's milestones while VoiceOver or NVDA runs instead of the voice), and one in each window
// (qp-window's), which says it instead while that window is the modal one on top, since a modal window makes the page
// behind it inert and a screen reader hears nothing there; a view's focus on its primary when it mounts; a window's
// return of the focus to the control that opened it; and the focus kept on a control a view draws again (each such
// control named by its key, `data-focus-key`: "move:12", "filter:all", "expander:above"), or, where the redraw left none
// of its key, on the control now standing in its place, so a redraw from a topic or a click never drops it to the page.

/** How long after the region empties a line comes in it. */
const ANNOUNCE_MS = 50;

let region = null;

/** The attribute that names a control across its view's redraws. */
const KEY = "data-focus-key";

/** What a press can press: a control that takes the focus or a click (a kit element's native control inside it). */
const CONTROL = "button, input, textarea, [tabindex]";

/** The control a press on `target` presses, or none: a press on a scrollbar, the page's ground or its text presses
 * none (no scroller of the page is a control). */
export function controlOf(target) {
	return target instanceof Element ? target.closest(CONTROL) : null;
}

/** Whether Space or Enter on `target` presses nothing of its own: the page's body or text, or a heading a view focused
 * (`focusOn`'s tabindex -1), never a control (a mouse click focuses no button in WebKit, so a click leaves the focus
 * there). */
export function pressesNothing(target) {
	const control = controlOf(target);
	return !control || (!control.matches("button, input, textarea") && control.getAttribute("tabindex") === "-1");
}

/** Finds the page's live region. */
export function startFocus() {
	region = document.getElementById("qp-live");
}

/** The modal window on top, if one is open. */
export function topWindow() {
	const open = document.querySelectorAll("dialog[open]");
	return open.length ? open[open.length - 1] : null;
}

/** Says a line once to a screen reader, politely (after what it is reading): in the modal window on top while one is
 * open, else in the page's region. */
export function announce(text) {
	const live = topWindow()?.querySelector(":scope > [data-live]") ?? region;
	if (!live) return;
	live.textContent = "";
	// the line a moment later, so the same line said twice is heard twice (a timer, never a frame: a hidden window
	// draws none)
	setTimeout(() => {
		live.textContent = text;
	}, ANNOUNCE_MS);
}

/** The key of the control holding `el` (the focus by default), or null. */
export function keyOf(el = document.activeElement) {
	return el?.closest?.(`[${KEY}]`)?.getAttribute(KEY) ?? null;
}

/** Focuses the first control under `root` keyed `key` that takes the focus (one in a view out of sight takes none);
 * whether one took it. */
function focusKeyed(root, key) {
	for (const el of root.querySelectorAll(`[${KEY}="${CSS.escape(key)}"]`)) {
		el.focus({preventScroll: true});
		if (el.contains(document.activeElement)) return true;
	}
	return false;
}

/** Gives the focus back to `el` (a page's or a window's opener), or, when a redraw replaced it meanwhile, to the
 * control of its `key` under `root`; whether one took it (the page's body never does). */
export function giveBack(el, key, root = document) {
	if (el && el !== document.body && el.isConnected && !el.closest("[hidden]") && typeof el.focus === "function") {
		el.focus({preventScroll: true});
		if (el.contains(document.activeElement)) return true;
	}
	return Boolean(key) && focusKeyed(root, key);
}

/** Remembers the focus now; the function it returns gives the focus back there (a closing window's opener), or, when a
 * redraw replaced that control meanwhile, to its replacement by its key. */
export function holdFocus() {
	const held = document.activeElement;
	const key = keyOf(held);
	return () => giveBack(held, key);
}

/** The keys of the keyed controls under `root` that stand before `el` in the page's order, the nearest last (`el`'s
 * own keyed control is none of them). */
function keysBefore(root, el) {
	const keys = [];
	for (const keyed of root.querySelectorAll(`[${KEY}]`)) {
		if (keyed.contains(el) || !(keyed.compareDocumentPosition(el) & Node.DOCUMENT_POSITION_FOLLOWING)) break;
		keys.push(keyed.getAttribute(KEY));
	}
	return keys;
}

/** Focuses the keyed control under `root` now standing where a lost one stood, `before` the keys that stood before it
 * (nearest last): the first control after the nearest of them still drawn (a pump row gone gives it to the row after;
 * Show more's last press to the first chapter it brought), else that one and those above it, nearest first; with none
 * of them drawn, the first control under `root`. The element that took the focus (one out of sight takes none), or
 * null. */
function focusNear(root, before) {
	const keyed = [...root.querySelectorAll(`[${KEY}]`)];
	const keys = keyed.map((el) => el.getAttribute(KEY));
	const at = before.findLast((k) => keys.includes(k));
	const i = at === undefined ? -1 : keys.indexOf(at);
	for (const el of [...keyed.slice(i + 1), ...keyed.slice(0, i + 1).reverse()]) {
		el.focus({preventScroll: true});
		if (el.contains(document.activeElement)) return document.activeElement;
	}
	return null;
}

/** Draws `root` again (`draw`) and puts the focus back where it was in it: on the same control where the redraw drew
 * it again (a part the view keeps across its redraws), on the new control of its key where the redraw replaced it, and
 * where none of that key is left (or the control had no key), on the control `fallback(key, near)` names, so the focus
 * never drops to the page. `near()` focuses the control now standing in the lost one's place (`focusNear`) and returns
 * it, or null; it is the fallback where a view names none, and a view names its own where that is not the place (Start
 * a pump giving way to Share today, the bar's Share today or Share day gone: `focusView`, the page's start). A view
 * whose redraw also changes its bar keeps the focus over its whole page (`view.el`). */
export function keepFocus(root, draw, fallback = (key, near) => near()) {
	const held = root.contains(document.activeElement) ? document.activeElement : null;
	const key = held ? keyOf(held) : null;
	const before = held ? keysBefore(root, held) : [];
	draw();
	if (!held || root.contains(document.activeElement)) return;
	if (root.contains(held)) {
		held.focus({preventScroll: true});
		if (held.contains(document.activeElement)) return;
	}
	if (key && focusKeyed(root, key)) return;
	fallback(key, () => focusNear(root, before))?.focus({preventScroll: true});
}

/** Focuses `target`, one that is no control (a heading, a code drawn: views/page.js `heldCode`) taking the focus without
 * joining the Tab order; whether it took it (a disabled control or one out of sight takes none). */
export function focusOn(target) {
	// a kit element focuses its own native control
	if (!target.control && !target.matches(CONTROL)) target.setAttribute("tabindex", "-1");
	target.focus({preventScroll: true});
	return target.contains(document.activeElement);
}

/** Focuses a view's primary control, else, where it has none or its primary takes no focus (Start a pump disabled while
 * the pump's gate holds), its first heading that takes it, so the keyboard and a screen reader start there; the element
 * that took the focus, or null. */
export function focusView(view) {
	const primary = view.primary?.();
	if (primary && focusOn(primary)) return document.activeElement;
	for (const heading of view.el.querySelectorAll("h1, h2, [data-focus]")) {
		if (focusOn(heading)) return document.activeElement;
	}
	return null;
}
