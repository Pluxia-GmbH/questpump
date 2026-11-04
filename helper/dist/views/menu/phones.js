// Phones (the Menu's page; docs/ui.md, "The Menu on the computer"): on Windows first the firewall's card while it keeps
// the phone out; then the pairing code in its QrWell (ink on the light well, its quiet zone around: a camera reads dark
// modules on light), the address under it and New pairing code, pressed twice within five seconds since it ends every
// pairing; once a phone is paired, its line under the heading (`phone_line`, the tray's words, as the Menu's row says
// it), and the code behind Pair another phone, or in sight with why while the paired phone has not linked for two weeks
// (`quiet`); the step left in the game whenever Rust names one (`next`: This computer's `gameStep`, or the Screen
// Recording grant); when QuestPump's settings did not read at its start, a line that the pairing ended and the code
// must be scanned again; when its secrets did not read at this start, no code and the address line saying pairing waits
// for the next start; and the Bluetooth link's row (slice 43): its switch, on by default, and its line by the helper's
// `ble` (`ble.line`: a link, an ask or a problem; none while it serves or stands switched off). From the `menu.phones`
// topic (helper/src-tauri/src/ui.rs `Phones`). The pairing code with its firewall card and address (`pairingCode`; on
// Windows without the firewall rule the code shows beside the card) is also the pairing window (`pairWindow`) of the
// first five minutes, Pump home's card and Pro's page, which counts as this page and shows the Bluetooth row's line
// under the code (since the review of 2026-10-17). Rust hears whether this page or the window is open and whether a
// code shows anywhere (`phones_shown`): the Bluetooth link advertises while the code shows or can show, and asks
// macOS's Bluetooth grant there alone.

import {call, get, subscribe} from "../../lib/bridge.js";
import {t} from "../../lib/copy.js";
import {register} from "../../lib/router.js";
import {h} from "../../kit/element.js";
import {page, watch, words} from "../page.js";
import {openWindow} from "../pump/windows.js";
import {adminNote, gameStep, stepCard} from "./computer.js";

/** How long New pairing code waits for its second press. */
const ARMED_MS = 5000;

/** The Phones page and the pairing window open (one of each at most) and the pairing codes in sight, each by its
 * `pairingCode()`, as Rust last heard them (`phones_shown`). */
let pagesOpen = 0;
const codesShown = new Set();
let told = null;

/** Tells Rust whether the Phones page or the pairing window is open and a pairing code shows, when that changed. */
function tellShown() {
	const now = {page: pagesOpen > 0, code: codesShown.size > 0};
	if (told && told.page === now.page && told.code === now.code) return;
	told = now;
	call("phones_shown", now).catch(console.error);
}

// a page loaded anew (the window's web content restarted) shows no code until one is drawn
tellShown();

/** The Bluetooth row's line by the helper's state (`ui.rs` `BleLine`): a link, an ask or a problem; none while it
 * serves with no phone linked by it or stands switched off, which its switch shows. */
function bleLines(line) {
	if (!line) return [];
	const windows = get("shell")?.platform === "windows";
	switch (line.state) {
		case "linked": return [t("desk.menu.phones.ble.linked", {phone: line.phone})];
		case "ask": return [t("desk.menu.phones.ble.ask")];
		case "refused": return [t(windows ? "desk.menu.phones.ble.refused.windows" : "desk.menu.phones.ble.refused")];
		case "off": return [t("desk.menu.phones.ble.off")];
		case "no_peripheral": return [t("desk.menu.phones.ble.noperipheral")];
		case "none": return [t("desk.menu.phones.ble.none")];
		case "failed": return [t("desk.menu.phones.ble.failed")];
		case "lost": return [t("desk.menu.phones.ble.lost")];
		default: return [];
	}
}

function addressLine(address) {
	if (!address) return "";
	switch (address.state) {
		case "ready": return t("desk.menu.phones.address", address);
		case "firewall": return t("desk.menu.phones.address.firewall", address);
		case "ble": return t("desk.menu.phones.address.ble", address);
		case "none": return t("desk.menu.phones.address.none", address);
		case "unread": return t("desk.menu.phones.address.unread", address);
		default: return address.host ? t("desk.menu.phones.address.lan.host", address)
			: t("desk.menu.phones.address.lan", address);
	}
}

/** The pairing code as the Phones page shows it, for that page and the pairing window (`pairWindow`): on Windows
 * first the firewall's card while it keeps the phone out, then the code in its QrWell (named for a screen reader) and
 * New pairing code (pressed twice within five seconds, since it ends every pairing), and the address under it.
 * `draw(p, shown)` from the `menu.phones` topic (`shown`: the code shows, none while there is none), `stop()` at its
 * page's end. */
export function pairingCode() {
	const allow = h("qp-secondary-button", {label: t("desk.menu.phones.firewall.allow"),
		"on:click": () => call("allow_network").catch(console.error)});
	const allowNote = words("", "meta");
	const rule = h("div", {class: "qp-stack-small"}, words(t("desk.menu.phones.firewall.rule"), "body"), allow,
		allowNote);
	const blocked = words(t("desk.menu.phones.firewall.blocked"), "body");
	const firewall = stepCard(t("desk.menu.phones.firewall.title"), rule, blocked);

	const well = h("qp-qr-well", {label: t("desk.menu.phones.code"), tabindex: "-1"});
	const renew = h("qp-quiet-button", {label: t("desk.menu.phones.renew")});
	let armed = null;
	renew.addEventListener("click", () => {
		if (armed) {
			clearTimeout(armed);
			armed = null;
			renew.label = t("desk.menu.phones.renew");
			call("new_pairing").catch(console.error);
			return;
		}
		renew.label = t("desk.menu.phones.renew.armed");
		armed = setTimeout(() => {
			armed = null;
			renew.label = t("desk.menu.phones.renew");
		}, ARMED_MS);
	});
	const code = h("qp-card", {class: "qp-pair"}, well, renew);
	const address = words("", "meta");
	const token = Symbol("code");
	return {firewall, code, address, well,
		draw(p, shown) {
			firewall.hidden = p.firewall === "open";
			rule.hidden = p.firewall !== "rule";
			blocked.hidden = p.firewall !== "blocked";
			allowNote.textContent = adminNote(p.network_note, "desk.menu.phones.firewall.failed");
			allowNote.hidden = !p.network_note;
			well.code = p.qr;
			code.hidden = !p.qr || !shown;
			address.textContent = addressLine(p.address);
			if (code.hidden) codesShown.delete(token);
			else codesShown.add(token);
			tellShown();
		},
		stop: () => {
			clearTimeout(armed);
			codesShown.delete(token);
			tellShown();
		},
	};
}

/** The close of the pairing window while it stands. */
let pairing = null;

/** The pairing code in a window over the page that asks for it (the first five minutes, Pump home's card, Pro's page):
 * this page's own code with its firewall card and address and, under them, the Bluetooth row's line; it counts as this
 * page, so the Bluetooth link starts there and macOS's grant is asked in context. Closed once a phone pairs; a second
 * ask while it stands opens no second one. Returns its close. */
export function pairWindow() {
	if (pairing) return pairing;
	const code = pairingCode();
	const ble = words("", "meta");
	let win = null;
	pagesOpen += 1;
	tellShown();
	const stop = subscribe("menu.phones", (p) => {
		code.draw(p, true);
		const lines = bleLines(p.ble.line);
		ble.textContent = lines.join(" ");
		ble.hidden = lines.length === 0;
		if (p.phone.name) win?.close();
	});
	win = openWindow({heading: t("desk.menu.phones.pair")},
		(content) => content.append(code.firewall, code.code, code.address, ble), () => {
			stop();
			code.stop();
			pagesOpen -= 1;
			tellShown();
			pairing = null;
		});
	pairing = () => win.close();
	return pairing;
}

register("desk.menu.phones", () => {
	const view = page({heading: t("desk.menu.phones.title")});
	const code = pairingCode();
	const heading = h("qp-section-label");
	const paired = h("p", {class: "qp-type-body-strong", role: "status"});
	const unread = words(t("desk.menu.phones.unread"), "body");
	const quiet = words(t("desk.menu.phones.quiet"), "body");
	const next = h("p", {class: "qp-type-body-strong qp-next", role: "status"});
	let showCode = false;
	const another = h("qp-quiet-button", {label: t("desk.menu.phones.another")});
	// the Bluetooth link's switch moves when the topic brings its state (Toggle's contract), never before
	const ble = h("qp-row", {heading: t("desk.menu.phones.ble.title"), glyph: "mark_link", trailing: "toggle"});
	ble.addEventListener("qp-change", (e) => call("set_ble_link", {on: e.detail}).catch(console.error));

	view.body.append(code.firewall, heading, paired, unread, quiet, next, h("qp-card", {rows: true}, ble),
		code.code, code.address, another);
	pagesOpen += 1;
	tellShown();
	view.stops.push(() => {
		pagesOpen -= 1;
		tellShown();
	});

	const draw = (p) => {
		const name = p.phone.name;
		heading.label = t(name ? "desk.menu.phones.paired.title" : "desk.menu.phones.pair");
		paired.textContent = name ? p.phone_line : "";
		paired.hidden = !name;
		unread.hidden = !p.unread;
		quiet.hidden = !name || !p.quiet;
		const step = p.next === "grant" ? t("desk.menu.phones.next.grant") : gameStep(p.next);
		next.textContent = step ?? "";
		next.hidden = !step;
		code.draw(p, !name || showCode || p.quiet);
		another.hidden = !p.qr || !name || showCode || p.quiet;
		ble.checked = p.ble.on;
		ble.lines = bleLines(p.ble.line);
	};
	let shown = null;
	another.addEventListener("click", () => {
		showCode = true;
		if (shown) draw(shown);
		code.well.focus();
	});
	watch(view, "menu.phones", (p) => {
		shown = p;
		draw(p);
	});
	view.stops.push(code.stop);
	return view;
});
