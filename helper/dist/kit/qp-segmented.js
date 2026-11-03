// Segmented: two or three segments in a card well, the chosen one raised, read as tabs (a tab list whose chosen tab is
// selected, named `label`; the arrow keys move between them, Home and End to the ends); `disabled` (an answer that no
// longer changes): every word in text3; `small` (desktop, 2026-10-28: Pump home's This week | All time beside a
// label): the Chip's height, each segment as wide as its word. `options` is a list of {value, label}; a choice fires
// qp-change with its value (the owner sets `value` once it holds).

import {KitElement, define, emit, h} from "./element.js";

class QpSegmented extends KitElement {
	build() {
		this.list = h("div", {role: "tablist", class: "qp-segments", "on:keydown": (e) => this.key(e)});
		this.append(this.list);
		this.choices = this.choices ?? [];
	}

	set options(value) {
		this.choices = value ?? [];
		if (this.built) this.update();
	}

	get options() {
		return this.choices;
	}

	choose(value) {
		if (!this.disabled && value !== this.value) emit(this, "qp-change", value);
	}

	key(e) {
		const values = this.choices.map((o) => o.value);
		const at = values.indexOf(this.value);
		const to = e.key === "ArrowRight" || e.key === "ArrowDown" ? Math.min(at + 1, values.length - 1)
			: e.key === "ArrowLeft" || e.key === "ArrowUp" ? Math.max(at - 1, 0)
				: e.key === "Home" ? 0 : e.key === "End" ? values.length - 1 : null;
		if (to === null) return;
		e.preventDefault();
		this.choose(values[to]);
		this.list.children[to]?.focus();
	}

	update() {
		if (this.label) this.list.setAttribute("aria-label", this.label);
		else this.list.removeAttribute("aria-label");
		// the segments are made again only when the options change, so a value a topic brings back keeps the focus on
		// its segment
		const segments = [...this.list.children];
		const same = segments.length === this.choices.length && this.choices.every((option, i) =>
			segments[i].segment === option.value && segments[i].textContent === option.label);
		if (!same) {
			this.list.replaceChildren(...this.choices.map((option) => {
				const button = h("button", {type: "button", role: "tab", class: "qp-segment", text: option.label,
					"on:click": () => this.choose(option.value)});
				button.segment = option.value;
				return button;
			}));
		}
		for (const button of this.list.children) {
			const chosen = button.segment === this.value;
			button.setAttribute("aria-selected", String(chosen));
			button.tabIndex = chosen ? 0 : -1;
			button.disabled = this.disabled;
		}
		this.control = this.list.querySelector("[aria-selected='true']") ?? this.list.firstElementChild;
	}
}

define("qp-segmented", QpSegmented, {value: String, disabled: Boolean, label: String, small: Boolean});
