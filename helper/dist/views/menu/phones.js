// Phones (the Menu's page; docs/ui.md, "The Menu on the computer"): on Windows first the firewall's card while it keeps
// the phone out; then the pairing code in its QrWell (ink on the light well, its quiet zone around: a camera reads dark
// modules on light), the address under it and New pairing code, pressed twice within five seconds since it ends every
// pairing; once a phone is paired, its line under the heading (`phone_line`, the tray's words, as the Menu's row says
// it), and the code behind Pair another phone, or in sight with why while the paired phone has not linked for two weeks
// (`quiet`); the step left in the game whenever Rust names one (`next`: This computer's `gameStep`, or the Screen
// Recording grant); when QuestPump's settings did not read at its start, a line that the pairing ended and the code
// must be scanned again; when its secrets did not read at this start, no code and the address line saying pairing waits
// for the next start. From the `menu.phones` topic (helper/src-tauri/src/ui.rs `Phones`). The pairing code with its
// firewall card and address (`pairingCode`) is also the pairing window (`pairWindow`) of the first five minutes, Pump
// home's card and Pro's page.

import {call, subscribe} from "../../lib/bridge.js";
import {t} from "../../lib/copy.js";
import {register} from "../../lib/router.js";
import {h} from "../../kit/element.js";
import {page, watch, words} from "../page.js";
import {openWindow} from "../pump/windows.js";
import {adminNote, gameStep, stepCard} from "./computer.js";

/** How long New pairing code waits for its second press. */
const ARMED_MS = 5000;

function addressLine(address) {
	if (!address) return "";
	switch (address.state) {
		case "ready": return t("desk.menu.phones.address", address);
		case "firewall": return t("desk.menu.phones.address.firewall", address);
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
		},
		stop: () => clearTimeout(armed),
	};
}

/** The close of the pairing window while it stands. */
let pairing = null;

/** The pairing code in a window over the page that asks for it (the first five minutes, Pump home's card, Pro's page):
 * this page's own code with its firewall card and address, closed once a phone pairs; a second ask while it stands
 * opens no second one. Returns its close. */
export function pairWindow() {
	if (pairing) return pairing;
	const code = pairingCode();
	let win = null;
	const stop = subscribe("menu.phones", (p) => {
		code.draw(p, true);
		if (p.phone.name) win?.close();
	});
	win = openWindow({heading: t("desk.menu.phones.pair")},
		(content) => content.append(code.firewall, code.code, code.address), () => {
			stop();
			code.stop();
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

	view.body.append(code.firewall, heading, paired, unread, quiet, next, code.code, code.address, another);

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
