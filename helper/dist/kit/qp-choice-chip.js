// ChoiceChip: a chip the player chooses with (a kind of quest, the check-in's answer, Book's filters): a 32-high pill of
// card with a line edge and text2 words in a 44-high target, chosen raised with a lineStrong edge, a check and text
// words; `locked` (a choice of Pro's without Pro: the lock glyph before its words, never chosen); `disabled` (a choice
// that cannot act now, text3 words: the check-in's answers while the engine folds). A native button pressed while
// `selected`; `hint` is what its Tooltip says (a gear chip's in Your setup), read after its name.

import {KitElement, define, h} from "./element.js";

class QpChoiceChip extends KitElement {
	build() {
		this.words = h("span");
		this.control = h("button", {type: "button", class: "qp-choice"});
		this.append(this.control);
	}

	update() {
		const mark = this.locked ? "mark_lock" : this.selected ? "mark_check" : null;
		this.words.textContent = this.label ?? "";
		this.control.replaceChildren(...[mark ? h("qp-art", {name: mark}) : null, this.words].filter(Boolean));
		this.control.setAttribute("aria-pressed", String(this.selected && !this.locked));
		this.control.classList.toggle("qp-locked", this.locked);
		this.control.disabled = this.disabled;
		if (this.hint) this.control.setAttribute("aria-description", this.hint);
		else this.control.removeAttribute("aria-description");
	}
}

define("qp-choice-chip", QpChoiceChip, {label: String, selected: Boolean, locked: Boolean, disabled: Boolean,
	hint: String});
