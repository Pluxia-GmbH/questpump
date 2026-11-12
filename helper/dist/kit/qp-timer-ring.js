// TimerRing, the pump's dial (iOS Kit/Figure.swift; 2026-10-23): an 8-px track and a gold arc with its glow from 12
// o'clock around a circular well, its face, which holds what the moment shows, each number and word a share of the
// ring's side (surface.TimerRing's dial fractions, kit.css), so the dial reads alike at any size its owner gives it from
// `least` up; under that, in a room named qp-dial, kit.css draws the clock alone, no arc (2026-10-25). The move's
// figure stands beside it on the PumpStage. `mode` is {kind: countdown {total, left, digit} (the digit, "Go" at
// 0) | timed {total, left} (the time left, m:ss) | up {total, elapsed} (a timed move run on after Keep going, slice 52:
// the seconds run, m:ss, counting up to the full move's total, the arc the time left to it) | hold {elapsed, goal?,
// record?} (the time held, its goal or record under it) | count {reps, goal?, word?} (a set of reps the voice counts:
// the reps counted, "goal 16+" or else its word under them, the arc filling toward the goal; 2026-10-13) | reps {goal}
// ("16+" over goal) | word {word} | entry {text, unit, share, chip} (the count set on the dial at Done, slice 66: the
// Stepper's value over its unit, the arc the count's share of the goal, the gold New record chip past the record) |
// pour {clock, seconds, share, caption?, chip} (a move's saved count pouring out of it, slices 65 and 66: the count
// landed, m:ss where it is `seconds`, the arc the share still to pour, the gold New record chip once past the
// record)}, its seconds from Rust's absolute instants each frame; `go` shows "Go" alone for a move's first second. Read
// as one: the digit, the seconds left, run or held with the goal or the record, the reps counted with the goal or the
// word, the count set, or the goal. Since slice 72 a countdown's digit pops as it comes (digitPop, as the phones'),
// and a goal met (a hold's seconds held, a counted set of reps' reps) turns its line gold behind a check until the
// side ends, as the game's goal line does, never by the colour alone.

import {t} from "../lib/copy.js";
import {pop} from "../lib/carry.js";
import {clock, LOCALE} from "../lib/format.js";
import {KitElement, define, h, token} from "./element.js";

const SVG = "http://www.w3.org/2000/svg";
const NUMBER = new Intl.NumberFormat(LOCALE);

class QpTimerRing extends KitElement {
	build() {
		// the ring in the token's own pixels, so its track keeps the token's width as the ring scales with the window
		const size = token(document.documentElement, "--qp-surface-timer-ring-size");
		const track = token(document.documentElement, "--qp-surface-timer-ring-track-width");
		const svg = document.createElementNS(SVG, "svg");
		svg.setAttribute("viewBox", `0 0 ${size} ${size}`);
		svg.setAttribute("aria-hidden", "true");
		const ring = (cls) => {
			const c = document.createElementNS(SVG, "circle");
			const at = String(size / 2);
			for (const [k, v] of Object.entries({cx: at, cy: at, r: String((size - track) / 2), pathLength: "100",
				class: cls})) {
				c.setAttribute(k, v);
			}
			return c;
		};
		this.arc = ring("qp-ring-arc");
		svg.append(ring("qp-ring-track"), this.arc);
		this.wordEl = h("span", {class: "qp-ring-digit qp-type-num-ring", "aria-hidden": "true"});
		this.clockEl = h("p", {class: "qp-ring-clock qp-type-num-hero", "aria-hidden": "true"});
		this.met = h("qp-art", {name: "mark_check", hidden: true});
		this.captionWords = h("span");
		this.caption = h("p", {class: "qp-ring-caption qp-type-caps", "aria-hidden": "true"}, this.met,
			this.captionWords);
		this.chip = h("span", {class: "qp-ring-chip qp-type-caps", "aria-hidden": "true", hidden: true,
			text: t("kit.reward.tag.record")});
		this.setAttribute("role", "img");
		this.append(h("div", {class: "qp-ring"}, svg, h("div", {class: "qp-ring-well"}),
			h("div", {class: "qp-ring-face"}, this.wordEl, this.clockEl, this.caption, this.chip)));
		this.modeData ??= {kind: "word", word: ""};
	}

	set mode(value) {
		this.modeData = value ?? {kind: "word", word: ""};
		if (this.built) this.update();
	}

	get mode() {
		return this.modeData;
	}

	/** The arc's share of the circle. */
	fraction() {
		const m = this.mode;
		switch (m.kind) {
			case "countdown":
			case "timed": return m.total > 0 ? Math.min(1, Math.max(0, m.left / m.total)) : 0;
			case "up": return m.total > 0 ? Math.min(1, Math.max(0, (m.total - m.elapsed) / m.total)) : 0;
			case "hold": {
				const against = m.goal ?? m.record;
				return against ? Math.min(1, m.elapsed / against) : 1;
			}
			case "count": return m.goal ? Math.min(1, m.reps / m.goal) : 1;
			case "entry":
			case "pour": return Math.min(1, Math.max(0, m.share));
			default: return 1;
		}
	}

	/** The goal is met: a hold held its goal's seconds, a counted set of reps counted its goal's reps. */
	goalMet() {
		const m = this.mode;
		if (!m.goal) return false;
		return (m.kind === "hold" && m.elapsed >= m.goal) || (m.kind === "count" && m.reps >= m.goal);
	}

	/** The line under a count: its goal ("goal 16+"), else its word. */
	countUnder() {
		const m = this.mode;
		return m.goal ? t("kit.sweep.goal.reps", {count: m.goal}) : m.word ?? "";
	}

	/** What a screen reader hears. */
	spoken() {
		const m = this.mode;
		const seconds = (n) => t("kit.objective.spoken.time", {seconds: n});
		switch (m.kind) {
			case "countdown": return m.digit > 0 ? String(m.digit) : t("kit.sweep.go");
			case "timed": return seconds(Math.ceil(m.left));
			case "up": return seconds(Math.floor(Math.min(m.total, m.elapsed)));
			case "pour": {
				const landed = m.seconds ? seconds(m.clock) : t("log.reps", {count: m.clock});
				return m.chip ? `${landed}, ${t("kit.reward.tag.record")}` : landed;
			}
			case "entry": return m.chip ? `${m.text} ${m.unit}, ${t("kit.reward.tag.record")}` : `${m.text} ${m.unit}`;
			case "hold": {
				const held = seconds(Math.floor(m.elapsed));
				const against = m.goal ? t("kit.sweep.goal.spoken", {count: m.goal})
					: m.record ? t("kit.sweep.record.spoken", {count: m.record}) : null;
				return against ? `${held}, ${against}` : held;
			}
			case "count": {
				const reps = t("log.reps", {count: m.reps});
				const under = this.countUnder();
				return under ? `${reps}, ${under}` : reps;
			}
			case "reps": return `${t("kit.goal.value", {goal: m.goal})} ${t("kit.goal")}`;
			default: return m.word ?? "";
		}
	}

	update() {
		const m = this.mode;
		this.arc.setAttribute("stroke-dasharray", `${this.fraction() * 100} 100`);
		this.dataset.state = m.kind;
		const word = m.kind === "countdown" ? (m.digit > 0 ? String(m.digit) : t("kit.sweep.go"))
			: this.go ? t("kit.sweep.go") : "";
		if (this.wordEl.textContent !== word) {
			this.wordEl.textContent = word;
			// each digit of the countdown, and its Go, pops as it comes
			if (word && m.kind === "countdown") pop(this.wordEl);
		}
		this.wordEl.hidden = !word;
		this.wordEl.classList.toggle("qp-ring-go", word === t("kit.sweep.go"));
		let big = "";
		let small = "";
		if (m.kind === "timed") big = clock(Math.ceil(m.left));
		else if (m.kind === "up") big = clock(Math.floor(Math.min(m.total, m.elapsed)));
		else if (m.kind === "hold") {
			big = clock(Math.floor(m.elapsed));
			small = m.goal ? t("kit.sweep.goal", {time: clock(m.goal)})
				: m.record ? t("kit.sweep.record", {count: m.record}) : "";
		} else if (m.kind === "pour") {
			big = m.seconds ? clock(m.clock) : NUMBER.format(m.clock);
			small = m.caption ?? "";
		} else if (m.kind === "entry") {
			big = m.text;
			small = m.unit;
		} else if (m.kind === "count") {
			big = NUMBER.format(m.reps);
			small = this.countUnder();
		} else if (m.kind === "reps") {
			big = t("kit.goal.value", {goal: m.goal});
			small = t("kit.goal");
		} else if (m.kind === "word") big = m.word ?? "";
		// "Go" stands alone for its second, the clock and its caption under it after
		if (this.clockEl.textContent !== big) this.clockEl.textContent = big;
		this.clockEl.hidden = !big || Boolean(word);
		if (this.captionWords.textContent !== small) this.captionWords.textContent = small;
		this.caption.hidden = !small || Boolean(word);
		const met = this.goalMet();
		this.met.hidden = !met;
		this.toggleAttribute("data-met", met);
		this.chip.hidden = !((m.kind === "pour" || m.kind === "entry") && m.chip);
		const said = this.spoken();
		if (this.getAttribute("aria-label") !== said) this.setAttribute("aria-label", said);
	}
}

define("qp-timer-ring", QpTimerRing, {go: Boolean});
