// A view's page: the Page under a TopBar (a root's) or a DetailBar (a pushed page's), and one centred column of at most
// 600 px under it (desk.css). `watch` follows a topic for as long as the view stands; `say` sets a step's result line,
// which a screen reader hears as it changes; `heldCode` holds a code that opens the player's account behind Show code.

import {subscribe} from "../lib/bridge.js";
import {t} from "../lib/copy.js";
import {announce, focusOn} from "../lib/focus.js";
import {conceal, reveal} from "../lib/router.js";
import {h} from "../kit/element.js";

/** A pushed page with its DetailBar `heading` (a root page's TopBar with `root`), its column `body`. */
export function page({heading, root = false}) {
	const bar = h(root ? "qp-top-bar" : "qp-detail-bar", {heading});
	const body = h("div", {class: "qp-column"});
	const el = h("qp-page", {}, bar, body);
	return {el, body, bar, stops: []};
}

/** Follows a topic while the view stands: `fn` gets its value now and on every change. */
export function watch(view, topic, fn) {
	view.stops.push(subscribe(topic, fn));
}

/** A paragraph of words, in the type role the caller names (no default, so no line under a heading comes by itself,
 * 2026-10-22). */
export function words(text, role) {
	return h("p", {class: `qp-type-${role} qp-words-block`, text});
}

/** A step's result line (wrong, expired, linking, linked, unanswered…), hidden while it has no words. */
export function resultLine() {
	return h("p", {class: "qp-type-body qp-quiet-line", hidden: true});
}

/** Sets a result line's words (none hides it), which a screen reader hears whenever they change (lib/focus.js
 * `announce`). */
export function say(line, text) {
	if (text && text !== line.textContent) announce(text);
	line.textContent = text;
	line.hidden = !text;
}

/** A code that opens the player's account (the restore code, the one typed to restore, Link a device's code, Email's
 * request code; the security review of 2026-10-31) held behind Show code, on macOS and Windows alike: the button
 * (`show`) stands where the code goes while the view has one; its press keeps the window out of screen captures first
 * (lib/router.js `reveal`, awaited) and only then draws the code (`draw(code)`, the focus on the element it returns),
 * which the view leaving the front (`onLeft`) or its code going takes away again (`draw(null)`, the window seen again:
 * `conceal`). `set` gives it the view's code as it comes and changes, null for none. */
export function heldCode(view, draw) {
	let code = null;
	let drawn = false;
	const show = h("qp-secondary-button", {label: t("menu.linkdevice.reveal"), hidden: true});
	const hold = () => {
		drawn = false;
		show.hidden = code === null;
		draw(null);
		conceal(view);
	};
	show.addEventListener("click", () => {
		reveal(view).then((may) => {
			if (!may) return;
			drawn = true;
			const shown = draw(code);
			show.hidden = true;
			focusOn(shown);
		}, console.error);
	});
	view.onLeft = hold;
	return {
		show,
		set(value) {
			code = value ?? null;
			if (code === null) hold();
			else if (drawn) draw(code);
			else show.hidden = false;
		},
	};
}
