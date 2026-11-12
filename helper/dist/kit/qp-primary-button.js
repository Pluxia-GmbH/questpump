// PrimaryButton: the one gold button of a screen, 56 high, full width: the gold gradient with its highlight, shade, lip
// and glow, goldInk words and an optional leading glyph. Pressed: no lip, 2 lower, darker. `state="disabled"`: raised,
// text3 words, no glow. `state="busy"`: three pips lit in turn after the words (still under Reduce Motion), a click does
// nothing, and a screen reader hears "Working" after the words. `compact` (a bar's with_primary): 44 high, as wide as
// its words. `progress` (0 to 1, none: no fill): a lighter fill grows from the leading edge to that share under the
// words, a light sweeping over it (goesOnShimmer; the fill alone under Reduce Motion), for a button its screen presses
// by itself once full (the saved move's Next, 2026-11-11).

import {t} from "../lib/copy.js";
import {KitElement, define, h} from "./element.js";

class QpPrimaryButton extends KitElement {
	build() {
		this.words = h("span", {class: "qp-words"});
		this.busy = h("span", {class: "qp-busy", "aria-hidden": "true"}, h("i"), h("i"), h("i"));
		this.said = h("span", {class: "qp-unseen"});
		this.goes = h("i", {class: "qp-goes", "aria-hidden": "true", hidden: true});
		this.control = h("button", {type: "button", class: "qp-primary"}, this.goes, this.words, this.busy, this.said);
		this.control.addEventListener("click", (e) => {
			if (this.state === "busy") e.stopImmediatePropagation();
		});
		this.append(this.control);
	}

	update() {
		this.words.textContent = this.label ?? "";
		const glyph = this.control.querySelector(":scope > qp-art");
		if (this.glyph && glyph?.name !== this.glyph) {
			glyph?.remove();
			this.control.prepend(h("qp-art", {name: this.glyph}));
		} else if (!this.glyph) {
			glyph?.remove();
		}
		this.control.disabled = this.state === "disabled";
		const busy = this.state === "busy";
		this.busy.hidden = !busy;
		this.control.setAttribute("aria-busy", String(busy));
		this.said.textContent = busy ? t("kit.button.busy") : "";
		const progress = this.progress;
		this.goes.hidden = progress == null;
		if (progress != null) this.goes.style.setProperty("--qp-goes", String(Math.min(1, Math.max(0, progress))));
	}
}

define("qp-primary-button", QpPrimaryButton, {label: String, glyph: String, state: String, compact: Boolean,
	progress: Number});
