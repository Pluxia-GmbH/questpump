// WorkCard: work mode's card on Pump home (docs/plan.md slice 67; screens.json WorkCard, design B: Start and Stop;
// Android's ui/WorkCard.kt is its twin), drawn from the `work` topic (helper/crates/questpump-desk/src/work.rs
// `WorkView`, its `work` property), one layout in every state: off, a compact row of Work mode and a trailing Start;
// on, the ring with the whole minutes left in it beside Work mode over the set's dots (the done ones gold, the next
// outlined, the last bigger, the long pump's) and Stop under them in stop; held, the ring dimmed and the hold's words
// in the dots' line ("On a call"), the title and Stop as they were; due, the ring full with a check, and under the
// block the gold Pump now and Later side by side; a Later refused says why under them. Start fires qp-start, Pump now
// qp-now, Later qp-later and Stop qp-stop; a focused button a state change hides hands the focus to the card's title,
// never to another button (a second Enter on Stop would end work mode). Also the WorkStrip, the set's blocks drawn for
// the first Start's sheet and the How page: each block's bar with a dot after it, the last dot bigger, Every's minutes
// under each (`every`, `blocks`); it says nothing to a screen reader, whose words are the How page's lines and the
// sheet's Every.

import {t} from "../lib/copy.js";
import {KitElement, define, emit, h, token} from "./element.js";

const SVG = "http://www.w3.org/2000/svg";

/** A hold's words, by its name on the topic (questpump-core `work::Hold`; the tray's `hold_words`). */
const HOLDS = {
	call: () => t("work.hold.call"),
	present: () => t("work.hold.present"),
	fullscreen: () => t("work.hold.fullscreen"),
	game: () => t("work.hold.game"),
	pump: () => t("work.hold.pump"),
	quiet: () => t("work.hold.quiet"),
	unwell: () => t("work.hold.unwell"),
};

/** A Later's refusal, by its key (`WorkView.refused`). */
const REFUSED = {
	"work.later.reset": () => t("work.later.reset"),
	"work.later.unwell": () => t("work.later.unwell"),
};

/** A Later's refusal's words by its key (`WorkView.refused`), none for another. */
function refusedWords(key) {
	return REFUSED[key]?.() ?? "";
}

/** Whether a work pump is due on `work` (the topic): its ask stands or waits for its moment. */
export function workDue(work) {
	return Boolean(work?.on) && (work.phase === "due" || work.phase === "ask");
}

class QpWorkCard extends KitElement {
	build() {
		const size = token(document.documentElement, "--qp-layout-tile");
		const width = token(document.documentElement, "--qp-layout-space4");
		const circle = (cls) => {
			const c = document.createElementNS(SVG, "circle");
			for (const [k, v] of Object.entries({cx: size / 2, cy: size / 2, r: (size - width) / 2, pathLength: 100,
				class: cls})) {
				c.setAttribute(k, String(v));
			}
			return c;
		};
		const svg = document.createElementNS(SVG, "svg");
		svg.setAttribute("viewBox", `0 0 ${size} ${size}`);
		svg.setAttribute("aria-hidden", "true");
		this.arc = circle("qp-work-arc");
		svg.append(circle("qp-work-track"), this.arc);
		this.minutes = h("span", {class: "qp-type-num-small", "aria-hidden": "true"});
		this.check = h("qp-art", {name: "mark_check", "aria-hidden": "true"});
		this.ring = h("div", {class: "qp-work-ring", role: "img"}, svg,
			h("div", {class: "qp-work-face"}, this.minutes, this.check));
		// the focus a hidden button held lands here, out of the Tab order
		this.head = h("h2", {class: "qp-type-heading", text: t("work.title"), tabindex: "-1"});
		this.start = h("qp-secondary-button", {label: t("work.start"), compact: true,
			"on:click": () => emit(this, "qp-start")});
		this.dots = h("div", {class: "qp-work-dots", role: "img"});
		this.holdLine = h("p", {class: "qp-type-body-strong qp-work-hold"});
		this.stop = h("qp-quiet-button", {class: "qp-destructive qp-work-stop", label: t("work.stop"),
			"on:click": () => emit(this, "qp-stop")});
		this.now = h("qp-primary-button", {label: t("work.pumpnow"), glyph: "action_pump",
			"on:click": () => emit(this, "qp-now")});
		this.later = h("qp-secondary-button", {label: t("work.later"), tall: true,
			"on:click": () => emit(this, "qp-later")});
		this.buttons = h("div", {class: "qp-work-buttons"}, this.now, this.later);
		this.refused = h("p", {class: "qp-type-meta qp-work-refused"});
		this.append(h("qp-card", {},
			h("div", {class: "qp-work-top"}, this.ring,
				h("div", {class: "qp-work-words"}, h("div", {class: "qp-work-title"}, this.head, this.start), this.dots,
					this.holdLine, this.stop)),
			this.buttons, this.refused));
		this.shown ??= null;
	}

	set work(value) {
		this.shown = value ?? null;
		if (this.built) this.update();
	}

	get work() {
		return this.shown ?? null;
	}

	update() {
		const focused = [this.start, this.now, this.later, this.stop].find((b) => b.contains(document.activeElement));
		const w = this.shown;
		const on = Boolean(w?.on);
		const due = workDue(w);
		const held = on && !due && Boolean(w.hold);
		this.ring.hidden = !on;
		this.ring.classList.toggle("qp-held", held);
		this.dots.hidden = !on || held;
		this.holdLine.hidden = !held;
		this.holdLine.textContent = held ? HOLDS[w.hold]?.() ?? "" : "";
		if (on) {
			const left = w.left_min ?? w.every;
			// a block held after it came due keeps its full ring (its minutes left are none)
			const full = due || left <= 0;
			const share = full ? 1 : w.every > 0 ? Math.min(1, Math.max(0, 1 - left / w.every)) : 0;
			this.arc.setAttribute("stroke-dasharray", `${share * 100} 100`);
			this.check.hidden = !full;
			this.minutes.hidden = full;
			this.minutes.textContent = full ? "" : String(left);
			this.ring.setAttribute("aria-label", full ? t("work.due") : t("work.next", {minutes: left}));
			this.drawDots(w.block, w.blocks);
		}
		this.start.hidden = on;
		this.stop.hidden = !on;
		this.buttons.hidden = !due;
		const refused = refusedWords(w?.refused);
		this.refused.hidden = !refused;
		this.refused.textContent = refused;
		// the focus a button this state hides held goes to the card's title, never to the page's body or another button
		const shows = (b) => !b.hidden && !(this.buttons.hidden && this.buttons.contains(b));
		if (focused && !shows(focused)) this.head.focus();
	}

	/** The set's dots: `done` of `blocks` gold, the next outlined, the last one bigger; read as Pump 2 of 4. */
	drawDots(done, blocks) {
		this.dots.setAttribute("aria-label", t("work.set", {block: Math.min(done + 1, blocks), blocks}));
		const kind = (i) => [i < done ? "qp-lit" : i === done ? "qp-next" : null, i === blocks - 1 ? "qp-work-long" : null]
			.filter(Boolean).join(" ") || null;
		const drawn = Array.from({length: blocks}, (_, i) => kind(i) ?? "").join(",");
		if (this.dots.dataset.drawn === drawn) return;
		this.dots.dataset.drawn = drawn;
		this.dots.replaceChildren(...Array.from({length: blocks}, (_, i) => h("i", {class: kind(i)})));
	}
}

class QpWorkStrip extends KitElement {
	update() {
		const blocks = Math.max(0, this.blocks ?? 0);
		const every = this.every ?? 0;
		this.setAttribute("aria-hidden", "true");
		this.replaceChildren(...Array.from({length: blocks}, (_, i) => h("div", {class: "qp-work-block"},
			h("div", {class: "qp-work-bar-row"}, h("i", {class: "qp-work-bar"}),
				h("i", {class: i === blocks - 1 ? "qp-work-dot qp-work-long" : "qp-work-dot"})),
			h("span", {class: "qp-type-meta", text: t("work.every.value", {minutes: every})}))));
	}
}

define("qp-work-card", QpWorkCard);
define("qp-work-strip", QpWorkStrip, {every: Number, blocks: Number});
