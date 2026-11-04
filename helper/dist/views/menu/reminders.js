// Reminders (the Menu's page on the computer; docs/ui.md, "The reminders on the computer"), the phones' Reminders page
// drawn for the computer from the `menu.reminders` topic (helper/src-tauri/src/desk/reminders.rs): the streak
// reminder's switch with its time under it while it is on (a window with a time field, kept by Done; "In quiet hours"
// as its meta when its minute lies in them), the sitting reminder's switch, quiet hours (a window of From and Until in
// the system's own time fields, as the phones' Reminders page opens its pickers) and reminders a day (a window with
// the Stepper from 1 to its most), which the reminders alone keep since slice 48 (a match's offer keeps neither), each
// switch and row its name and its value alone (2026-10-22). Each choice asks Rust
// (`reminders.*`) and shows the answer the topic brings, never ahead of it. Also the offer, "Remind me once a day?",
// once Rust says it is due (`offer`), over whatever page shows once no other window stands open: Remind me turns the
// streak reminder on; Not now and the × leave it off. Rust is told it showed (`reminders.offered`), so it never comes
// again.

import {intent, subscribe} from "../../lib/bridge.js";
import {t} from "../../lib/copy.js";
import {time} from "../../lib/format.js";
import {register, whenStarted} from "../../lib/router.js";
import {h} from "../../kit/element.js";
import {page, watch} from "../page.js";

/** The minutes of a day. */
const DAY_MIN = 24 * 60;

/** A time of day (minutes after local midnight) in the computer's own format. */
export function clock(minutes) {
	const at = new Date();
	at.setHours(Math.floor(minutes / 60), minutes % 60, 0, 0);
	return time(at.getTime());
}

/** A time of day as a time field's value, "HH:MM" (the places' schedule's times too). */
export function field(minutes) {
	return `${String(Math.floor(minutes / 60)).padStart(2, "0")}:${String(minutes % 60).padStart(2, "0")}`;
}

/** A time field's value as minutes after midnight, or null when it holds no time (the places' schedule's too). */
export function minutes(value) {
	const at = /^(\d{2}):(\d{2})/.exec(value);
	if (!at) return null;
	const m = Number(at[1]) * 60 + Number(at[2]);
	return m < DAY_MIN ? m : null;
}

/** Quiet hours as a row's value: Off where their ends are equal. `r` holds `quiet_from` and `quiet_to`. */
function quietLine(r) {
	return r.quiet_from === r.quiet_to ? t("menu.reminders.off")
		: t("reminders.span", {from: clock(r.quiet_from), until: clock(r.quiet_to)});
}

/** Quiet hours' window: From and Until in the system's time fields, kept by Done (its × drops them); drawn as it
 * opens from the topic's `quiet_from` and `quiet_to`. */
function quietWindow() {
	const el = h("qp-window", {heading: t("reminders.quiet"), primary: t("common.done")});
	const from = h("qp-text-input", {mode: "time", label: t("reminders.quiet.from")});
	const until = h("qp-text-input", {mode: "time", label: t("reminders.quiet.until")});
	const caption = (key) => h("p", {class: "qp-type-caps", text: t(key)});
	el.content.append(caption("reminders.quiet.from"), from, caption("reminders.quiet.until"), until);
	const draw = (r) => {
		from.value = field(r.quiet_from);
		until.value = field(r.quiet_to);
	};
	el.addEventListener("qp-primary", () => {
		const [start, end] = [minutes(from.value), minutes(until.value)];
		if (start === null || end === null) return;
		intent("reminders.quiet", {from: start, to: end}).catch(console.error);
		el.close();
	});
	return {el, draw};
}

/** Reminders a day's window: the Stepper from 1 to its most, kept by Done (its × drops it); drawn as it opens from the
 * topic's `cap` and `cap_max`. */
function capWindow() {
	const el = h("qp-window", {heading: t("reminders.cap"), primary: t("common.done")});
	const stepper = h("qp-stepper", {least: 1, label: t("reminders.cap"), "unit-one": t("reminders.cap.reminder"),
		"unit-other": t("reminders.cap.reminders"), "on:qp-change": (e) => {
			stepper.value = e.detail;
		}});
	el.content.append(stepper);
	const draw = (r) => {
		stepper.most = r.cap_max;
		stepper.value = r.cap;
	};
	el.addEventListener("qp-primary", () => {
		intent("reminders.cap", {cap: stepper.value}).catch(console.error);
		el.close();
	});
	return {el, draw};
}

/** The streak reminder's time window: one time field, kept by Done (its × drops it); drawn as it opens. */
function timeWindow() {
	const el = h("qp-window", {heading: t("reminders.time"), primary: t("common.done")});
	const at = h("qp-text-input", {mode: "time", label: t("reminders.time")});
	el.content.append(at);
	const draw = (r) => {
		at.value = field(r.streak_at);
	};
	el.addEventListener("qp-primary", () => {
		const m = minutes(at.value);
		if (m === null) return;
		intent("reminders.time", {at: m}).catch(console.error);
		el.close();
	});
	return {el, draw};
}

register("desk.reminders", () => {
	const view = page({heading: t("menu.reminders")});
	const boxes = {time: timeWindow(), quiet: quietWindow(), cap: capWindow()};
	let reminders = null; // the topic as it stands
	const opener = (box) => () => {
		if (!reminders) return;
		box.draw(reminders);
		box.el.open();
	};
	const toggle = (type) => (e) => intent(type, {on: e.detail}).catch(console.error);
	const streak = h("qp-row", {heading: t("reminders.streak"), glyph: "action_pump", trailing: "toggle",
		"on:qp-change": toggle("reminders.streak")});
	const time = h("qp-row", {heading: t("reminders.time"), glyph: "mark_clock", trailing: "value", hidden: true,
		"on:click": opener(boxes.time)});
	const sitting = h("qp-row", {heading: t("reminders.sitting"), glyph: "stat_agility", trailing: "toggle",
		"on:qp-change": toggle("reminders.sitting")});
	const quiet = h("qp-row", {heading: t("reminders.quiet"), glyph: "mark_pause", trailing: "value",
		"on:click": opener(boxes.quiet)});
	const cap = h("qp-row", {heading: t("reminders.cap"), glyph: "action_log", trailing: "value",
		"on:click": opener(boxes.cap)});
	view.body.append(h("qp-card", {rows: true}, streak, time), h("qp-card", {rows: true}, sitting),
		h("qp-card", {rows: true}, quiet, cap), ...Object.values(boxes).map((box) => box.el));
	watch(view, "menu.reminders", (r) => {
		reminders = r;
		streak.checked = r.streak_on;
		time.hidden = !r.streak_on;
		time.value = clock(r.streak_at);
		time.meta = r.streak_quiet ? t("reminders.quiet.inside") : null;
		sitting.checked = r.sitting_on;
		quiet.value = quietLine(r);
		cap.value = String(r.cap);
	});
	view.primary = () => streak;
	return view;
});

// The offer, once a Result here was put away (the phones' reminders.offer), after any window open then has closed
let offer = null; // the window while it shows
let waiting = false; // waiting for another window to close
let latest = null; // the topic as it last came

function ask() {
	if (!latest?.offer || offer) return;
	if (document.querySelector("dialog[open]")) {
		if (!waiting) {
			waiting = true;
			document.addEventListener("qp-closed", () => {
				waiting = false;
				ask();
			}, {once: true});
		}
		return;
	}
	offer = h("qp-window", {heading: t("reminders.offer.title"), primary: t("reminders.offer.yes"),
		secondary: t("reminders.offer.no")});
	offer.addEventListener("qp-primary", () => {
		intent("reminders.streak", {on: true}).catch(console.error);
		offer.close();
	});
	offer.addEventListener("qp-secondary", () => offer.close());
	offer.addEventListener("qp-closed", () => {
		offer.remove();
		offer = null;
	}, {once: true});
	intent("reminders.offered").catch(console.error);
	offer.open();
}

whenStarted(() => subscribe("menu.reminders", (r) => {
	latest = r;
	ask();
}));
