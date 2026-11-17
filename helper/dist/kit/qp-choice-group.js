// ChoiceGroup: ChoiceChips of which one is chosen (the streak's goal on the Result's streak day and on the streak page,
// slice 74): a radio group named `label`, one stop in the focus order, where a click chooses and the arrow keys move
// the choice (Home and End to the ends), the chips in a row that wraps, centred. `options` is a list of {value, label};
// a choice fires qp-change with its value (the owner sets `value` once it holds), a click on the chosen chip too:
// every click counts for its owner (the floating Result's hold starts over at each). With no `value` among them none
// is chosen (the Result's goals as they show): the first chip is the group's stop in the focus order, and an arrow
// key chooses from the chip that holds the focus. `named` is the id of the words on the page that name it, where it
// has such a label, in `label`'s place.

import {KitElement, define, emit, h} from "./element.js";

class QpChoiceGroup extends KitElement {
	build() {
		this.list = h("div", {role: "radiogroup", class: "qp-choice-group", "on:keydown": (e) => this.key(e)});
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
		emit(this, "qp-change", value);
	}

	key(e) {
		const values = this.choices.map((o) => o.value);
		const chosen = values.indexOf(this.value);
		// none chosen yet: the keys move from the chip the focus is on
		const at = chosen >= 0 ? chosen
			: [...this.list.children].findIndex((chip) => chip.control === document.activeElement);
		const to = e.key === "ArrowRight" || e.key === "ArrowDown" ? Math.min(at + 1, values.length - 1)
			: e.key === "ArrowLeft" || e.key === "ArrowUp" ? Math.max(at - 1, 0)
				: e.key === "Home" ? 0 : e.key === "End" ? values.length - 1 : null;
		if (to === null) return;
		e.preventDefault();
		// a key at the row's end moves no choice
		if (to !== at) this.choose(values[to]);
		this.list.children[to]?.control.focus();
	}

	update() {
		if (this.named) this.list.setAttribute("aria-labelledby", this.named);
		else this.list.removeAttribute("aria-labelledby");
		if (this.label && !this.named) this.list.setAttribute("aria-label", this.label);
		else this.list.removeAttribute("aria-label");
		// the chips are made again only when the options change, so a value its owner brings back keeps the focus
		const chips = [...this.list.children];
		const same = chips.length === this.choices.length && this.choices.every((option, i) =>
			chips[i].choice === option.value && chips[i].label === option.label);
		if (!same) {
			this.list.replaceChildren(...this.choices.map((option) => {
				const chip = h("qp-choice-chip", {label: option.label, radio: true,
					"on:click": () => this.choose(option.value)});
				chip.choice = option.value;
				return chip;
			}));
		}
		const none = !this.choices.some((option) => option.value === this.value);
		[...this.list.children].forEach((chip, i) => {
			chip.selected = chip.choice === this.value;
			chip.stop = none && i === 0;
		});
	}
}

define("qp-choice-group", QpChoiceGroup, {value: String, label: String, named: String});
