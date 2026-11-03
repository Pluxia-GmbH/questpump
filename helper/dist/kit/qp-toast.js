// Toast: a small card sliding in from the top (Reduce Motion: a fade) with its glyph (an unlock's move's tile) and two
// lines, the news over its line, held, then gone; three at most wait, a further one drops the oldest waiting so the
// newest news wins; each said to a screen reader politely, and its feedback event fired as it shows (qp-toast, which
// lib/sound.js sounds, as the phones' ToastHost does). While `paused` (a pump stands: the phones' holdsToasts) none
// shows and the queue waits: one showing goes back to its head and comes again with a new hold, said and sounded only
// the first time. One host stands in the window; `show(toast)` posts one: {kind: record {move, quality, count, next} |
// title {title} | level {level} | paragon {rank} | unlock {move, key, quality, word, rekindled, way} | imported |
// embers {week, day} | found {from: game|phone|both|here, platform: macos|windows, pair}} (embers: the game day's last
// ember a desktop pump lit in the app's own fold, "Cardio done today" over the week's, never fewer than Hero's vault
// card says, slice 55, as the game's, led by the day's pips, every one lit, as the phones' toast; found: the computer's
// first open found the player's progress there; a third line says
// where QuestPump waits, as the first five minutes' last page does, and with `pair`, no phone paired yet, a fourth
// points to Pump home's card that pairs one). `dropCelebrations()` drops every celebrating one after a felt-unwell
// stop.

import {t} from "../lib/copy.js";
import {announce} from "../lib/focus.js";
import {KitElement, animationMs, define, emit, h, qualityName, token} from "./element.js";

/** A found toast's line, by where the progress came from. */
const FOUND = {
	game: "desk.welcome.found.game", phone: "desk.welcome.found.phone", both: "desk.welcome.found.both",
	here: "desk.welcome.found.here",
};

/** The found toast's lines under its first: where QuestPump waits (the first five minutes' last page's words), and,
 * while no phone is paired, the way to pair one; none for any other toast. */
function after(toast) {
	if (toast.kind !== "found") return [];
	const waits = t(toast.platform === "windows" ? "desk.done.tray" : "desk.done.menubar");
	return toast.pair ? [waits, t("desk.welcome.found.pair")] : [waits];
}

function heading(toast) {
	switch (toast.kind) {
		case "record": return t("kit.toast.record");
		case "title": return t("kit.toast.title");
		case "level": return t("kit.toast.level");
		case "paragon": return t("kit.toast.paragon");
		case "unlock": return t(toast.rekindled ? "kit.toast.rekindled" : toast.way === "earned" ? "kit.toast.won"
			: "kit.toast.unlock");
		case "found": return t("desk.welcome.found");
		case "embers": return t("kit.toast.embers");
		default: return t("kit.toast.imported");
	}
}

function line(toast) {
	switch (toast.kind) {
		case "record": return t("kit.toast.record.line", {move: toast.move, count: toast.count, next: toast.next});
		case "title": return toast.title;
		case "level": return t("kit.level", {level: toast.level});
		case "paragon": return t("kit.fanfare.paragon", {rank: toast.rank});
		case "unlock": return toast.move;
		case "found": return t(FOUND[toast.from]);
		case "embers": return t("kit.toast.embers.line", {count: toast.week});
		default: return t("kit.toast.imported.line");
	}
}

function lead(toast) {
	switch (toast.kind) {
		case "record": {
			const art = h("qp-art", {name: "reward_record"});
			const tone = qualityName(toast.quality);
			if (tone) art.style.setProperty("color", `var(--qp-quality-${tone})`);
			return art;
		}
		case "title": return h("qp-art", {name: "reward_title", class: "qp-text"});
		case "level":
		case "paragon": return h("qp-art", {name: "reward_xp", class: "qp-gold"});
		case "unlock": return h("qp-move-tile", {size: 44, move: toast.key, quality: toast.quality});
		case "found": return h("qp-art", {name: "mark_check", class: "qp-text2"});
		// the day's embers, every one lit, as Pump home's row draws them: the toast's lines say them
		case "embers": return h("qp-day-pips", {have: toast.day, need: toast.day, "aria-hidden": "true"});
		default: return h("qp-art", {name: "action_character", class: "qp-text2"});
	}
}

/** The feedback event its showing fires (contract/design/tokens.json `feedback`; the phones' ToastData.event): an
 * unlock's by its quality (Uncommon, Rare, Epic, and Legendary with Artifact each their own; Common and below,
 * rekindled, or an orange or gold step its climb only opened, `way` "opened", the plain one; "earned" is headed Earned,
 * 2026-10-09). */
function feedback(toast) {
	switch (toast.kind) {
		case "record": return "record";
		case "title": return "title";
		case "embers": return "win";
		case "level":
		case "paragon": return "levelUp";
		case "unlock": {
			const tone = qualityName(toast.quality);
			if (toast.rekindled || toast.way?.startsWith("opened") || tone === "starter" || tone === "common") return "unlock";
			if (tone === "uncommon") return "unlockUncommon";
			if (tone === "rare") return "unlockRare";
			return tone === "legendary" || tone === "artifact" ? "unlockLegendary" : "unlockEpic";
		}
		default: return "open";
	}
}

class QpToast extends KitElement {
	build() {
		this.queue = [];
		this.card = h("div", {class: "qp-toast", hidden: true});
		this.append(this.card);
	}

	show(toast) {
		this.ensure();
		this.queue.push(toast);
		// how many may wait behind the one shown (tokens' surface.Toast.queueCount)
		if (this.queue.length > token(this, "--qp-surface-toast-queue-count")) this.queue.shift();
		if (this.card.hidden) this.next();
	}

	dropCelebrations() {
		this.ensure();
		this.queue = this.queue.filter((toast) => toast.kind === "imported");
		if (!this.card.hidden && this.shown?.kind !== "imported") this.next();
	}

	next() {
		clearTimeout(this.timer);
		this.card.hidden = true;
		this.card.classList.remove("qp-out");
		this.shown = null;
		if (this.paused) return;
		const toast = this.queue.shift();
		if (!toast) return;
		this.shown = toast;
		const words = [line(toast), ...after(toast)];
		const news = h("div", {}, h("p", {class: "qp-type-body-strong", text: heading(toast)}),
			...words.map((text) => h("p", {class: "qp-type-meta", text})));
		if (toast.kind === "unlock") news.append(h("qp-quality-word", {quality: toast.quality, word: toast.word}));
		this.card.replaceChildren(lead(toast), news);
		this.card.hidden = false;
		// one back after a pause says nothing again (the phones' ToastHost)
		if (!toast.told) {
			toast.told = true;
			announce(`${heading(toast)}. ${words.join(" ")}`);
			emit(this, "qp-toast", feedback(toast));
		}
		const hold = token(this, "--qp-motion-toast-hold-duration");
		this.timer = setTimeout(() => {
			this.card.classList.add("qp-out");
			// its leaving motion's time, then the next one (Reduce Motion: a fade)
			this.timer = setTimeout(() => this.next(), animationMs(this.card));
		}, hold);
	}

	update() {
		if (this.paused && this.shown) {
			this.queue.unshift(this.shown);
			this.next();
			return;
		}
		if (!this.paused && this.card.hidden && this.queue.length) this.next();
	}
}

define("qp-toast", QpToast, {paused: Boolean});
