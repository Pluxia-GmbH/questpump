// Chip: a 32-high pill of card with a line edge, an optional dot or glyph and its words. `state`: `ok` (the green dot
// with its glow), `reaching` (a pulsing amber dot, still under Reduce Motion), `off` (a grey dot), `stopped` (a red
// dot, the status chip's while the tray icon is red: helper/src-tauri/src/desk/tray.rs `Tone`), `record` (36 high in
// the uncommon tone, the record glyph), `plain` (words and an optional glyph: Hero's streak), `focus` and `source`
// (Hero's: a glyph and words, a button), `new_moves` (the Saga's held book, slice 44: gold words over card after the
// upright arrow in gold, floating over the page; views/log/held.js places it, moves it in and out and says it).
// `button` makes the chip a button (the status chip opens This computer; since slice 51 Hero's streak the streak
// page), at least 44 high for the pointer, dimmed while pressed, `hint` what it does, read after its words.

import {KitElement, define, h} from "./element.js";

class QpChip extends KitElement {
	build() {
		this.dot = h("span", {class: "qp-chip-dot", "aria-hidden": "true"});
		this.glyphEl = h("qp-art", {class: "qp-chip-glyph"});
		this.words = h("span", {class: "qp-chip-words"});
	}

	update() {
		const tag = this.button ? "button" : "span";
		if (this.control?.localName !== tag) {
			this.control = h(tag, {class: "qp-chip", ...(tag === "button" ? {type: "button"} : {})});
			this.replaceChildren(this.control);
		}
		const state = this.state ?? "plain";
		const glyph = state === "record" ? "reward_record" : state === "new_moves" ? "mark_arrow" : this.glyph;
		if (glyph && this.glyphEl.name !== glyph) this.glyphEl.name = glyph;
		const dotted = state === "ok" || state === "reaching" || state === "off" || state === "stopped";
		this.control.dataset.state = state;
		if (this.button && this.hint) this.control.setAttribute("aria-description", this.hint);
		else this.control.removeAttribute("aria-description");
		this.words.textContent = this.label ?? "";
		this.control.replaceChildren(...[dotted ? this.dot : null, glyph ? this.glyphEl : null, this.words].filter(Boolean));
	}
}

define("qp-chip", QpChip, {label: String, state: String, glyph: String, button: Boolean, hint: String});
