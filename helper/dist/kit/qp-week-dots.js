// WeekDots: the week's seven 30-px marks from the week's first day, equal columns. `days`: a list of seven {initial,
// done, rested, resting, today, future}: a pump day gold with a check over its weekday's initial, a rested day (a rest
// token kept it) with the rest mark in frost and a resting day (felt unwell) with the pause mark in text2 (2026-10-09:
// never a red miss), today ringed in gold; read as the caller's `label` says. `steps` (Pro's weeks): a list of seven {step 0-3, record, today, future}: a day by its moves in the
// body map's capped steps, a record day's star, today ringed, a day to come without an edge. `month` (slice 51, the
// streak page): the weekday `initials` over a game month's days in rows of seven from the week's first day, `days` a
// cell each, null for a cell before the month's first, each mark as the days' with its `date` (the day of the month)
// in a mark without a glyph, `monthGap` between the rows. `light(indices)` (slice 72) lights a week's marks one after
// another as their days arrive (the streak's day on the Result, a reward carried home: kit.css), so the marks are made
// again only when what they show changed.

import {KitElement, define, h} from "./element.js";

class QpWeekDots extends KitElement {
	set days(value) {
		this.week = value ?? [];
		if (this.built) this.update();
	}

	get days() {
		return this.week ?? [];
	}

	/** Lights the marks of the week's days `indices` (0 the week's first day), each the chip's stagger after the one
	 * before (motion.carry; at once under Reduce Motion, whose tokens give no time). */
	light(indices) {
		const marks = this.querySelectorAll(".qp-dot");
		indices.forEach((day, order) => {
			const mark = marks[day];
			if (!mark) return;
			mark.style.setProperty("--qp-lit-order", String(order));
			mark.classList.add("qp-lit-in");
		});
	}

	update() {
		this.setAttribute("role", "img");
		if (this.label) this.setAttribute("aria-label", this.label);
		// drawn again only when the marks changed, so a mark that lights is not cut off by a redraw of the same week
		const drawn = JSON.stringify([this.days, this.month, this.steps, this.initials ?? null]);
		if (drawn === this.drawn) return;
		this.drawn = drawn;
		if (this.month) {
			this.replaceChildren(...(this.initials ?? []).map((i) => h("span", {class: "qp-type-caps", text: i})),
				...this.days.map((day) => (day ? h("span", {class: "qp-day"}, this.mark(day)) : h("span"))));
			return;
		}
		this.replaceChildren(...this.days.map((day) => h("span", {class: "qp-day"}, this.mark(day),
			this.steps ? "" : h("span", {class: "qp-type-caps", text: day.initial ?? ""}))));
	}

	/** A day's mark. */
	mark(day) {
		const mark = h("span", {class: "qp-dot"});
		if (this.steps) mark.dataset.step = String(day.step ?? 0);
		mark.classList.toggle("qp-done", Boolean(day.done));
		mark.classList.toggle("qp-today", Boolean(day.today));
		mark.classList.toggle("qp-future", Boolean(day.future));
		mark.classList.toggle("qp-rested", Boolean(day.rested));
		mark.classList.toggle("qp-resting", Boolean(day.resting));
		if (day.done && !this.steps) mark.append(h("qp-art", {name: "mark_check"}));
		if (day.rested) mark.append(h("qp-art", {name: "mark_rest"}));
		if (day.resting) mark.append(h("qp-art", {name: "mark_pause"}));
		if (day.record) mark.append(h("qp-art", {name: "star"}));
		if (day.date && !mark.childElementCount) mark.append(h("span", {class: "qp-dot-date", text: day.date}));
		return mark;
	}
}

define("qp-week-dots", QpWeekDots, {label: String, steps: Boolean, month: Boolean});
