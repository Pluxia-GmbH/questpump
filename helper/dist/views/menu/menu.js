// The Menu on the computer (docs/ui.md, "The Menu on the computer"): one page of Rows, every setting within two clicks,
// in its groups: This computer, Phones (its line the paired phone's) and Devices (the games and the offers after them
// are the Games tab's, slice 56); your pumps (Reminders, Sound and voice, Your setup with its equipment's count, Quick
// pump length with its Segmented of the `expedition` topic's lengths inline (2026-11-03: the timed pump every Quick
// pump starts, `expedition.length`, its value moving with Rust's answer), Backup and sync with the day of its last
// backup file or sync, whichever came later, then Email with the linked address, Email linked where another device
// linked it, or Not linked, slice 60); and QuestPump (Safety and about, what
// QuestPump does on this computer, Send feedback, the switch Send crash reports and Usage statistics, each with its
// disclosure line, Copy a report for help, the Acknowledgements, and Pro's row: On, or that the paired phone's Pro
// could not be confirmed here, opening Pro's page; no price). The switches show the `crash` and `usage`
// topics' `on` and move with Rust's answer to `crash.switch` and `usage.switch` (contract/crash-api.yaml,
// contract/telemetry-api.yaml). A row that opens another area's page shows once that page is registered (the router's
// lookup), a group without a row not at all. Copy a report for help puts QuestPump's own report (the `diagnostics`
// command's) on the clipboard, written inside the click with the text still coming (WebKit asks for that), and says for
// three seconds whether it copied; Pump home and Hero offer the same report as a quiet button when pumps could not
// start. In the browser (its `play` topic, docs/plan.md slice 63) the Menu starts with Get QuestPump for your computer
// (the site's /from/play in a new tab) and, while the phone apps are in the stores (the topic's `phones`), Get
// QuestPump for your phone (the site's /get/phone in a new tab), and a Menu not yet saved ends on Delete progress in
// this browser (asked once), which deletes this browser's data and starts again ("Leaving"); the desktop app publishes
// no `play`.

import {call, get, intent} from "../../lib/bridge.js";
import {t} from "../../lib/copy.js";
import {announce} from "../../lib/focus.js";
import {MENU, push, register, registered} from "../../lib/router.js";
import {h} from "../../kit/element.js";
import {confirmWindow} from "../book/common.js";
import {GET_COMPUTER, GET_PHONE, page, watch} from "../page.js";
import {backupDay} from "./backup.js";
import {clock} from "./reminders.js";

/** How long the report row's line says how it went. */
const SAID_MS = 3000;

/** Copy a report for help: QuestPump's own report (the `diagnostics` command's) on the clipboard, written inside the
 * click with the text still coming (WebKit asks for that); resolves with the key of how it went, which a screen reader
 * hears as it is said. */
function copyReport() {
	const text = call("diagnostics").then((report) => new Blob([report], {type: "text/plain"}));
	return navigator.clipboard.write([new ClipboardItem({"text/plain": text})]).then(
		() => "desk.feedback.diagnostics.copied", () => "desk.feedback.diagnostics.failed").then((key) => {
		announce(t(key));
		return key;
	});
}

/** Copy a report for help as a quiet button (Pump home's and Hero's beside "Pumps on this computer could not start"):
 * its words say for three seconds how the copy went. Returns the button and the way to stop its timer. */
export function reportButton() {
	let reset = null;
	const said = (key) => {
		button.label = t(key);
		clearTimeout(reset);
		reset = setTimeout(() => {
			button.label = t("desk.feedback.diagnostics");
		}, SAID_MS);
	};
	const button = h("qp-quiet-button", {label: t("desk.feedback.diagnostics"),
		"on:click": () => copyReport().then(said)});
	return {el: button, stop: () => clearTimeout(reset)};
}

/** The Row that opens the page `to`, none while that page is not registered. */
function open(to, heading, glyph) {
	return registered(to) ? h("qp-row", {heading, glyph, "on:click": () => push(to)}) : null;
}

/** A group: its caps label (none for the first) over a card of its rows; none when no row stands. */
function group(label, ...rows) {
	const kept = rows.filter(Boolean);
	if (!kept.length) return [];
	const card = h("qp-card", {rows: true}, ...kept);
	return label ? [h("qp-section-label", {label}), card] : [card];
}

register(MENU, () => {
	const view = page({heading: t("bar.menu")});
	const computer = open("desk.menu.computer", t("desk.menu.computer.title"), "device_computer");
	const phones = open("desk.menu.phones", t("desk.menu.phones.title"), "device_phone");
	const devices = open("desk.menu.devices", t("desk.menu.devices.title"), "mark_heart");
	const reminders = open("desk.reminders", t("menu.reminders"), "mark_clock");
	const sound = open("desk.menu.sound", t("desk.menu.sound"), "rating_expertise");
	const setup = open("desk.menu.setup", t("menu.setup"), "action_character");
	// Quick pump length: the lengths inline, the one set chosen once Rust's answer holds
	const lengths = h("qp-segmented", {small: true, label: t("menu.quick.length"),
		"on:qp-change": (e) => intent("expedition.length", {minutes: Number(e.detail)}).catch(console.error)});
	const quick = h("qp-row", {heading: t("menu.quick.length"), glyph: "mark_clock", trailing: "shown"});
	quick.extra = lengths;
	const backup = open("desk.menu.backup", t("menu.backup.header.sync"), "mark_share");
	const email = open("desk.menu.email", t("menu.email.title"), "mark_cloud");
	const about = open("desk.menu.about", t("menu.about.row"), "action_book");
	const does = open("desk.menu.does", t("desk.menu.does"), "mark_lock");
	const feedback = open("desk.feedback", t("menu.feedback.row"), "mark_report");
	const acknowledgements = open("desk.menu.acknowledgements", t("menu.acknowledgements.row"), "reward_title");
	const pro = open("desk.menu.pro", t("pro.title"), "helm");
	const getComputer = get("play") ? h("qp-row", {heading: t("setup.add.get"), glyph: "device_computer",
		trailing: "external", "on:click": () => window.open(GET_COMPUTER, "_blank", "noopener")}) : null;
	const getPhone = get("play") ? h("qp-row", {heading: t("play.phoneapp"), glyph: "device_phone", trailing: "external",
		hidden: !get("play").phones, "on:click": () => window.open(GET_PHONE, "_blank", "noopener")}) : null;
	const wipe = get("play") ? h("qp-row", {heading: t("play.wipe"), glyph: "mark_close", trailing: "action",
		destructive: true, hidden: true, "data-focus-key": "wipe",
		"on:click": () => confirmWindow(t("play.wipe.title"), null, t("places.delete"),
			() => intent("play.wipe").catch(console.error))}) : null;
	// the switch moves when the topic brings Rust's answer (Toggle's contract), never before; each shows once its topic
	// stands (the browser sends nothing, slice 63, and publishes neither)
	const crash = h("qp-row", {heading: t("menu.crash"), meta: t("menu.crash.note"), glyph: "mark_report",
		trailing: "toggle", hidden: true});
	crash.addEventListener("qp-change", (e) => intent("crash.switch", {on: e.detail}).catch(console.error));
	const usage = h("qp-row", {heading: t("menu.usage"), meta: t("menu.usage.note"), glyph: "action_log",
		trailing: "toggle", hidden: true});
	usage.addEventListener("qp-change", (e) => intent("usage.switch", {on: e.detail}).catch(console.error));
	const diagnostics = h("qp-row", {heading: t("desk.feedback.diagnostics"), glyph: "mark_list", trailing: "action"});
	let reset = null;
	diagnostics.addEventListener("click", () => copyReport().then((key) => {
		diagnostics.meta = t(key);
		clearTimeout(reset);
		reset = setTimeout(() => {
			diagnostics.meta = null;
		}, SAID_MS);
	}));
	view.body.append(
		...group(null, getComputer, getPhone, computer, phones, devices),
		...group(t("desk.menu.section.pumps"), reminders, sound, setup, quick, backup, email),
		...group(t("desk.feedback.section"), about, does, feedback, crash, usage, diagnostics, acknowledgements, pro,
			wipe));
	watch(view, "menu.phones", (phonesTopic) => {
		if (phones) phones.meta = phonesTopic.phone_line;
	});
	watch(view, "menu.setup", (s) => {
		if (!setup) return;
		const count = s.place.gear.length;
		setup.value = count ? t("menu.setup.gear", {count}) : t("menu.setup.nogear");
	});
	watch(view, "expedition", (e) => {
		lengths.options = e.lengths.map((m) => ({value: String(m), label: t("pump.length.minutes", {minutes: m})}));
		lengths.value = String(e.length);
	});
	watch(view, "menu.reminders", (r) => {
		// the phones' Reminders value: the streak reminder's time (In quiet hours where it lies there), else the sitting
		// cue where it alone is on, else Off
		if (!reminders) return;
		reminders.value = r.streak_on ? (r.streak_quiet ? t("reminders.quiet.inside") : clock(r.streak_at))
			: r.sitting_on ? t("reminders.sitting") : t("menu.reminders.off");
	});
	watch(view, "crash", (c) => {
		crash.hidden = false;
		crash.checked = c.on;
	});
	watch(view, "usage", (u) => {
		usage.hidden = false;
		usage.checked = u.on;
	});
	// the later of the last backup file and the last sync (none while Sync is off)
	const last = {backup: null, sync: null};
	const lastDay = () => {
		if (backup) backup.value = backupDay(Math.max(last.backup ?? 0, last.sync ?? 0) || null);
	};
	watch(view, "menu.backup", (b) => {
		last.backup = b.last;
		lastDay();
	});
	watch(view, "menu.sync", (s) => {
		last.sync = s.on ? s.last : null;
		lastDay();
	});
	watch(view, "menu.email", (e) => {
		if (email) email.value = e.linked ? e.address ?? t("menu.email.linked") : t("menu.email.notlinked");
	});
	watch(view, "menu.about", (a) => {
		if (!pro) return;
		// On while Pro holds (a paired phone's or the account's), or that the phone's could not be confirmed
		pro.value = a.pro === "on" ? t("menu.on") : null;
		pro.meta = a.pro === "refused" ? t("desk.menu.pro.refused") : null;
	});
	watch(view, "play", (p) => {
		if (wipe) wipe.hidden = p.saved;
		if (getPhone) getPhone.hidden = !p.phones;
	});
	view.stops.push(() => clearTimeout(reset));
	view.primary = () => getComputer ?? computer;
	return view;
});
