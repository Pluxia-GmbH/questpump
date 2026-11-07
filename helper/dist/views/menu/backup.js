// Backup and sync on the computer (docs/ui.md, "The Menu on the computer"; the phones' Backup and sync page; slice
// 60): the Sync card first, its switch "Sync your progress" with its state line (syncing, the last sync, waiting for a
// connection, failed, a newer QuestPump, full, the account deleted, this computer removed, a join held until another
// device makes the account's new key or this computer's key replaced by it, or the switch greyed while the secrets did
// not read at this start, `unread`), Devices (a Row per device of the account: its glyph, its name, else This computer
// or its kind's word, and its last sync; another one's Row asks once to remove it), Link a device and the legal line
// on what syncs (`menu.sync.footer`), from the `menu.sync` topic
// (helper/crates/questpump-desk/src/sync.rs); the switch moves when the topic brings Rust's answer (Toggle's contract).
// Then the backup file: its last date, the last outcome (what a file brought, or why nothing came), then Save a backup
// file (the system's save dialog, which Rust shows) and Restore from a file (its open dialog), both waiting while one
// is under way, Restore while a pump runs, from the `menu.backup` topic
// (helper/src-tauri/src/desk/menu_pages/backup.rs). In the browser (docs/plan.md slice 63) the Sync card alone, this
// browser's Row its own, and while it is in no account Save your progress in the switch's place, which opens the Email
// page's Save flow, since a browser makes no account in silence.

import {get, intent} from "../../lib/bridge.js";
import {t} from "../../lib/copy.js";
import {announce} from "../../lib/focus.js";
import {LOCALE} from "../../lib/format.js";
import {push, register} from "../../lib/router.js";
import {h} from "../../kit/element.js";
import {confirmWindow} from "../book/common.js";
import {page, watch, words} from "../page.js";

const DAY = 24 * 60 * 60 * 1000;
const SHORT = new Intl.DateTimeFormat(LOCALE, {day: "numeric", month: "short"});
const SHORT_YEAR = new Intl.DateTimeFormat(LOCALE, {day: "numeric", month: "short", year: "numeric"});
const WHEN = new Intl.DateTimeFormat(LOCALE, {dateStyle: "medium", timeStyle: "short"});

/** The last backup's day as the Menu's row names it (the phones' `backupDay`): today, yesterday, its short date with
 * its year outside this one, or never. */
export function backupDay(ms) {
	if (!ms) return t("menu.backup.never.short");
	const day = new Date(ms);
	const today = new Date();
	today.setHours(0, 0, 0, 0);
	if (day >= today) return t("log.today");
	if (day >= new Date(today.getTime() - DAY)) return t("log.yesterday");
	return (day.getFullYear() === today.getFullYear() ? SHORT : SHORT_YEAR).format(day);
}

/** Why nothing, or not all, came of a file, in the phones' words and the computer's own (a store that did not take
 * every row, a pump that started while the open dialog stood). */
function refused(why) {
	switch (why) {
		case "newer": return t("menu.backup.newer");
		case "not_backup": return t("menu.backup.notbackup");
		case "too_large": return t("menu.backup.toolarge");
		case "unreadable": return t("menu.backup.unreadable");
		case "not_kept": return t("desk.menu.backup.notkept");
		case "pump_on": return t("desk.menu.backup.pumpon");
		default: return t("menu.backup.notsaved");
	}
}

/** What a file brought: its log entries, else this computer's setup, else nothing new; and that this computer kept its
 * own setup over the file's. */
function restored(o) {
	const line = o.rows ? t("menu.backup.restored", {items: t("menu.backup.entries", {count: o.rows})})
		: o.settings ? t("desk.menu.backup.place") : t("menu.backup.nothing");
	return o.kept ? [line, t("desk.menu.backup.kept")].join(" ") : line;
}

/** The Sync card's state line: what the sync does or why it stopped, else its last sync; none before the first. */
function syncLine(s) {
	switch (s.line) {
		case "syncing": return t("menu.sync.syncing");
		case "waiting": return t("menu.sync.waiting");
		case "failed": return t("menu.sync.failed");
		case "newer": return t("menu.sync.newer");
		case "full": return t("menu.sync.full");
		case "deleted": return t("menu.sync.deleted");
		case "removed": return t("menu.sync.removed");
		case "rekey": return t("menu.sync.rekey");
		case "unread": return t("desk.menu.sync.unread");
		default: return s.on && s.last ? t("menu.sync.last", {date: WHEN.format(new Date(s.last))}) : null;
	}
}

/**
 * A device's words where it sent no name yet: its kind's (contract/sync-api.yaml `Kind`), a browser's (`web`) too, and
 * a kind this build does not know (a newer QuestPump's) the word for any device.
 */
function kindWord(kind) {
	switch (kind) {
		case "ios": return t("menu.sync.kind.ios");
		case "android": return t("menu.sync.kind.android");
		case "desktop": return t("menu.sync.kind.desktop");
		case "web": return t("menu.sync.kind.web");
		default: return t("menu.sync.kind.unknown");
	}
}

/** A device's glyph: a computer's for a computer and a browser, a phone's for the rest. */
const kindGlyph = (kind) => kind === "desktop" || kind === "web" ? "device_computer" : "device_phone";

/** The Sync card: the switch with its state line (in a browser in no account, Save your progress), then Devices with
 * Link a device, then the footer. */
function syncCard(web) {
	const toggle = h("qp-row", {heading: t("menu.sync.toggle"), glyph: "mark_cloud", trailing: "toggle",
		"data-focus-key": "sync"});
	toggle.addEventListener("qp-change", (e) => intent("sync.switch", {on: e.detail}).catch(console.error));
	const save = h("qp-row", {heading: t("menu.sync.save.title"), glyph: "mark_cloud", hidden: true,
		"data-focus-key": "save", "on:click": () => push("desk.menu.email", {save: true})});
	const link = h("qp-row", {heading: t("menu.sync.linkdevice"), glyph: "mark_link", "data-focus-key": "linkdevice",
		"on:click": () => push("desk.menu.linkdevice")});
	const label = h("qp-section-label", {label: t("menu.sync.devices")});
	const devices = h("qp-card", {rows: true});
	const els = [h("qp-section-label", {label: t("menu.sync.header")}), h("qp-card", {rows: true}, toggle, save),
		label, devices, words(t("menu.sync.footer"), "meta")];
	const remove = (d, name) => confirmWindow(t("menu.sync.remove.title", {device: name}), null, t("menu.sync.remove"),
		() => intent("sync.remove", {device: d.device}).catch(console.error));
	return {els, toggle, save, account(held) {
		toggle.hidden = web && !held;
		save.hidden = !toggle.hidden;
	}, draw(s) {
		toggle.checked = s.on;
		toggle.disabled = s.line === "unread";
		toggle.meta = syncLine(s);
		label.hidden = !s.on;
		devices.hidden = !s.on;
		// this computer first, then the others as the server lists them
		const rows = [...s.devices].sort((a, b) => Number(b.this) - Number(a.this)).map((d) => {
			const name = d.this ? t(web ? "play.sync.this" : "desk.menu.sync.this") : d.name ?? kindWord(d.kind);
			return h("qp-row", {heading: name, glyph: kindGlyph(d.kind),
				value: backupDay(d.seen_at), trailing: d.this ? "shown" : "action", "data-focus-key": `device:${d.device}`,
				"on:click": d.this ? null : () => remove(d, name)});
		});
		devices.replaceChildren(...rows, link);
	}};
}

register("desk.menu.backup", () => {
	const view = page({heading: t("menu.backup.header.sync")});
	const web = get("shell")?.platform === "web";
	const sync = syncCard(web);
	const last = h("p", {class: "qp-type-body-strong"});
	const busy = h("p", {class: "qp-type-body qp-quiet-line", hidden: true, text: t("menu.backup.restoring")});
	const said = h("p", {class: "qp-type-body qp-quiet-line", hidden: true});
	const save = h("qp-primary-button", {label: t("menu.backup.save")});
	const open = h("qp-secondary-button", {label: t("menu.backup.restore")});
	let opening = false;
	let heard; // the outcome's seq at the last look: one that comes while the page shows is said once
	save.addEventListener("click", () => {
		opening = false;
		intent("backup.save").catch(console.error);
	});
	open.addEventListener("click", () => {
		opening = true;
		intent("backup.open").catch(console.error);
	});
	view.body.append(...sync.els);
	watch(view, "menu.sync", sync.draw);
	watch(view, "menu.email", (e) => sync.account(e.account));
	intent("sync.look").catch(console.error);
	// the backup file is a computer's (its save and open dialogs)
	if (web) {
		view.primary = () => (sync.toggle.hidden ? sync.save : sync.toggle);
		return view;
	}
	view.body.append(h("qp-section-label", {label: t("menu.backup.header")}),
		h("qp-card", {class: "qp-stack-small"}, last, busy, said), h("div", {class: "qp-foot"}, save, open));
	watch(view, "menu.backup", (b) => {
		last.textContent = b.last ? t("menu.backup.last", {date: WHEN.format(new Date(b.last))})
			: t("menu.backup.never");
		// a file read and taken in says so while it comes; a save dialog says nothing
		busy.hidden = !(b.busy && opening);
		const outcome = b.outcome;
		said.hidden = !outcome || b.busy;
		if (outcome) said.textContent = outcome.state === "restored" ? restored(outcome) : refused(outcome.why);
		const seq = outcome?.seq ?? 0;
		if (heard !== undefined && seq > heard) announce(said.textContent);
		heard = seq;
		save.state = b.busy ? "busy" : "normal";
		open.disabled = b.busy || b.pump_on;
	});
	intent("backup.look").catch(console.error);
	view.primary = () => sync.toggle;
	return view;
});
