// StepPips: a ladder's 14 pips (tokens' surface.StepPips.count), the first `step` lit in the step's quality tone
// (`quality`), the rest dark; read as "Step 5 of 14".

import {t} from "../lib/copy.js";
import {KitElement, define, h, qualityName, token} from "./element.js";

class QpStepPips extends KitElement {
	update() {
		// out of the page there is no style to read the count from: drawn when it comes back
		if (!this.isConnected) return;
		const total = token(this, "--qp-surface-step-pips-count");
		const step = Math.max(0, Math.min(total, this.step ?? 0));
		const name = qualityName(this.quality);
		if (name) this.style.setProperty("--qp-tone", `var(--qp-quality-${name})`);
		this.setAttribute("role", "img");
		this.setAttribute("aria-label", t("kit.pips.label", {step, total}));
		this.replaceChildren(...Array.from({length: total}, (_, i) => h("i", {class: i < step ? "qp-lit" : null})));
	}
}

define("qp-step-pips", QpStepPips, {step: Number, quality: Number});
