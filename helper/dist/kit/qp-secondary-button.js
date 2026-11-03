// SecondaryButton: a quieter choice beside the primary, 48 high, or as high as the primary when `tall` (a choice that
// weighs as much as the primary's: the privacy page's Reject all, as learnclash sizes its Accept all and Reject all),
// full width, raised with a lineStrong edge and text words; `destructive` (a step that cannot be undone) in stop.
// Pressed: darker. Disabled: a line edge, text3 words. `busy` (a step card's "Do the check" pressed on the pinned card,
// waiting for its pump; 2026-10-19): three pips lit in turn after the words as the primary's (still under Reduce
// Motion), a click does nothing, and a screen reader hears "Working" after the words.

import {t} from "../lib/copy.js";
import {KitElement, define, h} from "./element.js";

class QpSecondaryButton extends KitElement {
	build() {
		this.words = h("span", {class: "qp-words"});
		this.pips = h("span", {class: "qp-busy", "aria-hidden": "true"}, h("i"), h("i"), h("i"));
		this.said = h("span", {class: "qp-unseen"});
		this.control = h("button", {type: "button", class: "qp-secondary"}, this.words, this.pips, this.said);
		this.control.addEventListener("click", (e) => {
			if (this.busy) e.stopImmediatePropagation();
		});
		this.append(this.control);
	}

	update() {
		this.words.textContent = this.label ?? "";
		this.control.disabled = this.disabled;
		this.control.classList.toggle("qp-destructive", this.destructive);
		this.control.classList.toggle("qp-tall", this.tall);
		this.pips.hidden = !this.busy;
		this.control.setAttribute("aria-busy", String(this.busy));
		this.said.textContent = this.busy ? t("kit.button.busy") : "";
	}
}

define("qp-secondary-button", QpSecondaryButton,
	{label: String, disabled: Boolean, destructive: Boolean, tall: Boolean, busy: Boolean});
