// The keyboard (docs/plan.md slice 35, decision 10; docs/ui.md, "The pump on the computer"), the game's grammar: each
// view focuses its one primary, so Space or Enter presses it (on a pump's stages Enter, and Space where the game's
// presses it, also where a click left the focus on no control: views/pump/session.js), and may take keys of its own
// (`onKey`: the count's digits, Escape as Done on a running move or back to Ready from the countdown; true when it
// took the key). A held key
// repeats nothing, and for its first half second a view (or a window) takes no Enter or Space, so a key held from the
// screen before never presses its primary. Escape closes a window, never one marked closedby="none" (the place window,
// the check-in's joints), which a capture-phase guard keeps open where the engine lacks closedby (WebKit), and never
// banks, skips or ends a pump. Cmd/Ctrl + and - and 0 zoom the page through Rust (`view.zoom`, kept in settings.json),
// Cmd/Ctrl+1 to 5 choose the five roots' tabs in the ActionBar's order as the macOS View menu's items do (slice 46; the
// frame's `tab`, lib/router.js, which chooses none under a pump's cover or a window), Cmd/Ctrl+F goes to the view's own
// search (`onFind`), else, from any other root, a page on one or the Menu, to the Book's Moves search as on the phones
// (the frame's `find`; none under a pump's cover or a window either), the web view's own accelerators (reload, print,
// find on the page, history) and a keyboard's browser keys do nothing, and nor does its context menu outside a text
// box. No single-letter shortcut. On macOS the page sees a key with Cmd before the app menu does (WebKit's WebViewImpl
// performKeyEquivalent: a key the page takes, its default prevented, never reaches the menu), so a key and its menu
// item act once.

import {intent} from "./bridge.js";
import {topWindow} from "./focus.js";
import {token} from "../kit/element.js";
import {ROOTS} from "../kit/qp-root-host.js";

/** How long a view's primary takes no key after it mounts (and the pump's swap row and an ask's answers no click or key
 * after they are drawn, views/pump/stages.js), in ms: the settle wait every app takes (tokens.css'
 * --qp-motion-settle-duration, contract/design/tokens.json `motion.settle`; the game's `answer_settle_s`). */
export function guardMs() {
	return token(document.documentElement, "--qp-motion-settle-duration");
}

let current = null; // the view the keys go to
let mountedAt = 0;
let frame = null; // the main window's frame (the card's page has none): Cmd/Ctrl+1 to 5's `tab`, Cmd/Ctrl+F's `find`

/** A view (or a window's content) took the screen: keys go to it, and its first half second takes no Enter or Space. */
export function mounted(view) {
	current = view;
	mountedAt = performance.now();
}

/** A window opened over the view: the same half-second guard. */
export function guard() {
	mountedAt = performance.now();
}

/** The frame's keys from now on: Cmd/Ctrl+1 to 5 choose its roots through `tab(root)`, and Cmd/Ctrl+F where the view
 * in front has no search of its own goes to `find()`. */
export function takeFrame(keys) {
	frame = keys;
}

function typing(target) {
	return target instanceof HTMLTextAreaElement
		|| (target instanceof HTMLInputElement && !["checkbox", "radio", "button", "range"].includes(target.type));
}

function zoom(e) {
	if (!(e.metaKey || e.ctrlKey) || e.altKey) return false;
	const step = e.key === "=" || e.key === "+" ? 1 : e.key === "-" ? -1 : e.key === "0" ? 0 : null;
	if (step === null) return false;
	intent("view.zoom", {step}).catch(console.error);
	return true;
}

/** The root Cmd/Ctrl+1 to 5 names, in the ActionBar's order: by the key's digit, else by its place on the keyboard (a
 * layout whose digits take Shift: AZERTY's 1 key gives "&"); none for another key. */
function tabRoot(e) {
	if (!(e.metaKey || e.ctrlKey) || e.altKey) return null;
	const digit = /^[1-9]$/.test(e.key) ? e.key : /^Digit[1-9]$/.test(e.code) ? e.code.slice(-1) : null;
	return digit ? ROOTS[Number(digit) - 1] ?? null : null;
}

/** A keyboard's browser keys (the "Browser control keys" of KeyboardEvent.key's values). */
const BROWSER_KEYS = new Set(["BrowserBack", "BrowserFavorites", "BrowserForward", "BrowserHome", "BrowserRefresh",
	"BrowserSearch", "BrowserStop"]);

/** The web view's own keys that would reload, print, find on the page (F3, Shift+F3, Cmd/Ctrl+G and Shift with it)
 * or move in its history, and a keyboard's browser keys: QuestPump has none of those. */
function browserKey(e) {
	const mod = e.metaKey || e.ctrlKey;
	const key = e.key.toLowerCase();
	return e.key === "F5" || e.key === "F3" || BROWSER_KEYS.has(e.key)
		|| (mod && (key === "r" || key === "p" || key === "g"))
		|| (e.altKey && (e.key === "ArrowLeft" || e.key === "ArrowRight") && !typing(e.target));
}

function stop(e) {
	e.preventDefault();
	e.stopPropagation();
}

function onKeyDown(e) {
	if (e.isComposing) return;
	if (zoom(e) || browserKey(e)) {
		stop(e);
		return;
	}
	const root = frame ? tabRoot(e) : null;
	if (root) {
		stop(e);
		if (!e.repeat) frame.tab(root);
		return;
	}
	const over = topWindow();
	if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "f") {
		stop(e);
		// a held key finds once; a window stands over the view's search, out of reach
		if (e.repeat || over) return;
		if (current?.onFind) current.onFind();
		else frame?.find();
		return;
	}
	if (e.key === "Escape") {
		// a closable window closes itself on its cancel event; one that must stay keeps the key, and a held Escape closes
		// no second window and steps back no further
		if (e.repeat || over?.getAttribute("closedby") === "none") stop(e);
		else if (!over && current?.onKey?.(e)) stop(e);
		return;
	}
	if (e.key === "Enter" || e.key === " ") {
		const text = typing(e.target) && (e.key === " " || e.target instanceof HTMLTextAreaElement);
		if (!text && (e.repeat || performance.now() - mountedAt < guardMs())) {
			stop(e);
			return;
		}
	}
	// a view's own keys take no repeat: a held digit types one digit
	if (!over && !e.repeat && current?.onKey?.(e)) stop(e);
}

function onKeyUp(e) {
	// Space presses a button on its key up: the guard keeps that one too
	if (e.key === " " && !typing(e.target) && performance.now() - mountedAt < guardMs()) stop(e);
}

/** Starts listening, before any other listener sees a key. */
export function startKeys() {
	window.addEventListener("keydown", onKeyDown, true);
	window.addEventListener("keyup", onKeyUp, true);
	window.addEventListener("contextmenu", (e) => {
		if (!typing(e.target)) e.preventDefault();
	}, true);
}
