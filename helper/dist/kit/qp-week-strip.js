// WeekStrip (the Saga, 2026-10-15; Tokens.Surface.WeekStrip; contract/fixtures/history.json `saga` rule 17): the game
// week holding a day as seven bars of its moves over each weekday's initial, the day `ringed` (its bar gold and ringed,
// its initial gold), a day without a move no bar but a short rest bar in WeekDots' rest where a rest token kept it
// (`rested`, the days' places; today's week's alone, from the streak's board), and the dashed Old best line at `best`
// moves where the day had one; the bars scale to the larger of the week's most and the old best. `days`
// [{day, moves}] are its data; it reads as its `label`.

import {initial} from "../lib/saga.js";
import {KitElement, define, h, styled} from "./element.js";

class QpWeekStrip extends KitElement {
	build() {
		this.dayData ??= [];
		this.restedDays ??= [];
		this.setAttribute("role", "img");
	}

	set days(value) {
		this.dayData = value ?? [];
		if (this.built) this.update();
	}

	set rested(value) {
		this.restedDays = value ?? [];
		if (this.built) this.update();
	}

	update() {
		if (this.label) this.setAttribute("aria-label", this.label);
		const most = Math.max(1, this.best ?? 0, ...this.dayData.map((d) => d.moves));
		const bars = this.dayData.map((d, i) => {
			const bar = d.moves > 0 ? styled(h("i", {class: "qp-week-bar"}), {"--qp-week-share": d.moves / most})
				: this.restedDays.includes(i) ? h("i", {class: "qp-week-bar qp-rest"}) : "";
			if (bar) bar.classList.toggle("qp-ringed", d.moves > 0 && i === this.ringed);
			return h("span", {class: "qp-week-col"}, h("span", {class: "qp-week-well"}, bar),
				h("span", {class: "qp-week-initial", text: initial(d.day)}));
		});
		const line = this.best ? styled(h("i", {class: "qp-week-best"}), {"--qp-week-share": this.best / most}) : "";
		this.replaceChildren(h("div", {class: "qp-week-strip", "aria-hidden": "true"}, line, ...bars));
		this.classList.toggle("qp-has-ring", this.ringed !== null && this.ringed !== undefined);
	}
}

define("qp-week-strip", QpWeekStrip, {label: String, ringed: Number, best: Number});
