// StepsBar: one 4-high bar per move of a pump: done goldLo, now gold with a glow, the rest the track; an unfinished
// move riding a match's pump (slice 61) in the Legendary orange, done, now (its glow) or not yet, under the word Banked
// centred above the riders' bars (Rapha, 2026-11-02); read as "Move 2 of 4", or where riders ride
// beside the pump's own moves as the game's line reads it (Panel.lua `stepsText`): "Move 2 of 6, then 3 unfinished" on
// a move of its own, "Unfinished 1 of 3, move 4 of 6" on a rider. `now` counts from 1; `riders` (a property: a
// boolean a move, in the pump's order) marks the riders, and a pump whose every move is one (a catch-up) reads and
// draws as before.

import {t} from "../lib/copy.js";
import {KitElement, define, h, styled} from "./element.js";

class QpStepsBar extends KitElement {
	/** The riders, a boolean a move in the pump's order. */
	set riders(value) {
		this._riders = value;
		if (this.built) this.update();
	}

	get riders() {
		return this._riders ?? [];
	}

	update() {
		const total = Math.max(1, this.total ?? 1);
		const now = Math.max(1, Math.min(total, this.now ?? 1));
		const marks = this.riders.slice(0, total);
		const riders = marks.filter(Boolean).length;
		const mixed = riders > 0 && riders < total;
		const rider = (i) => mixed && marks[i] === true;
		this.setAttribute("role", "img");
		this.setAttribute("aria-label", !mixed ? t("kit.steps.move", {now, total})
			: rider(now - 1) ? t("kit.steps.rider", {at: marks.slice(0, now).filter(Boolean).length, riders, now, total})
				: t("kit.steps.riders", {now, total, riders}));
		const bars = Array.from({length: total}, (_, i) => {
			const state = i + 1 < now ? "qp-done" : i + 1 === now ? "qp-now" : null;
			return h("i", {class: rider(i) ? ["qp-rider", state].filter(Boolean).join(" ") : state});
		});
		if (!mixed) {
			this.replaceChildren(...bars);
			return;
		}
		// the riders ride together after the pump's own moves (Engine.lua ns.Compose), the pump's own all-out window
		// after them at most: the word spans the bars from the first rider to the last
		const from = marks.indexOf(true);
		const span = marks.lastIndexOf(true) - from + 1;
		const label = styled(h("b", {class: "qp-steps-banked", text: t("kit.steps.banked"), "aria-hidden": "true"}),
			{"--qp-steps-total": total, "--qp-steps-from": from, "--qp-steps-span": span});
		this.replaceChildren(label, ...bars);
	}
}

define("qp-steps-bar", QpStepsBar, {total: Number, now: Number});
