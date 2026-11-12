// A view's page: the Page under a TopBar (a root's) or a DetailBar (a pushed page's), and one centred column of at most
// 600 px under it (desk.css). `watch` follows a topic for as long as the view stands; `say` sets a step's result line,
// which a screen reader hears as it changes, with the feedback event of a step that succeeded or failed (slice 72:
// `commit`, `error`); `heldCode` holds a code that opens the player's account behind Show code.

import {subscribe} from "../lib/bridge.js";
import {t} from "../lib/copy.js";
import {announce, focusOn} from "../lib/focus.js";
import {conceal, reveal} from "../lib/router.js";
import {feedback} from "../lib/sound.js";
import {h} from "../kit/element.js";

/** QuestPump on your computer from the browser (docs/plan.md slice 63): the site's click count (`/from/play`, the
 * source `play`, contract/get-api.yaml), which goes on to the download page. */
export const GET_COMPUTER = "https://questpump.com/from/play";

/** QuestPump on your phone from the browser while the phone apps are in the stores (the `play` topic's `phones`): the
 * site's `/get/phone`, which sends the visitor to their phone's store, or to the start page where it cannot tell
 * (contract/get-api.yaml). */
export const GET_PHONE = "https://questpump.com/get/phone";

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
 * `announce`); with `event`, the words say a step that succeeded (`commit`) or failed (`error`), which sounds as they
 * come. */
export function say(line, text, event = null) {
	if (text && text !== line.textContent) {
		announce(text);
		if (event) feedback(event);
	}
	line.textContent = text;
	line.hidden = !text;
}

/** How long Copy says Copied. */
const COPIED_MS = 3000;

/** Copy for a code the view holds (the restore code, Link a device's link code): the code on the clipboard, then
 * Copied for three seconds, heard as it is said; disabled while `set` gives it none. Its timer stops with the view. */
export function copyButton(view) {
	let text = "";
	let reset = null;
	const button = h("qp-secondary-button", {label: t("menu.restore.copy")});
	button.addEventListener("click", () => {
		navigator.clipboard.writeText(text).then(() => {
			button.label = t("menu.restore.copied");
			announce(button.label);
			clearTimeout(reset);
			reset = setTimeout(() => {
				button.label = t("menu.restore.copy");
			}, COPIED_MS);
		}, console.error);
	});
	view.stops.push(() => clearTimeout(reset));
	return {
		button,
		set(value) {
			text = value ?? "";
			button.disabled = !value;
		},
	};
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

/** The restore code held behind Show code, its one line under it, and its Copy (Restore code's page and Keep your
 * key's): `set` takes the code, none while there is none. */
export function restoreCode(view) {
	const code = h("p", {class: "qp-type-num-medium qp-words-block qp-centre", hidden: true});
	const held = heldCode(view, (shown) => {
		code.textContent = shown ?? "";
		code.hidden = shown === null;
		return code;
	});
	const copy = copyButton(view);
	return {
		card: h("qp-card", {class: "qp-stack-small"}, code, held.show, words(t("menu.restore.line"), "body")),
		copy: copy.button,
		set(value) {
			copy.set(value);
			held.set(value || null);
		},
	};
}
