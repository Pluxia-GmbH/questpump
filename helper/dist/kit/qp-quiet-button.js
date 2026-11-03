// QuietButton: a third choice, words alone, 44 high, text2 words in buttonQuiet; hover lightens them, a press darkens
// them. `glyph` leads the words with a piece of the art where the button plays or pauses (the pump's read-aloud
// button: mark_play, mark_pause). `spoken` names it to a screen reader where its words alone say too little (one Skip
// beside each unfinished move: "Skip Push-up"), and `hint` is what it does, read after its name.

import {KitElement, define, h} from "./element.js";

class QpQuietButton extends KitElement {
	build() {
		this.control = h("button", {type: "button", class: "qp-quiet"});
		this.append(this.control);
	}

	update() {
		const words = this.label ?? "";
		if (this.glyph) this.control.replaceChildren(h("qp-art", {name: this.glyph}), h("span", {text: words}));
		else this.control.textContent = words;
		this.control.classList.toggle("qp-quiet-glyph", Boolean(this.glyph));
		this.control.disabled = this.disabled;
		for (const [attr, value] of [["aria-label", this.spoken], ["aria-description", this.hint]]) {
			if (value) this.control.setAttribute(attr, value);
			else this.control.removeAttribute(attr);
		}
	}
}

define("qp-quiet-button", QpQuietButton, {
	label: String, disabled: Boolean, spoken: String, hint: String, glyph: String,
});
