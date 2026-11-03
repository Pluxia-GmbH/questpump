// Toggle: a 50 × 30 switch read as one: the gold gradient with a light knob when on, a dark track with a pale knob when
// off, the knob sliding (at once under Reduce Motion); disabled, raised with a text3 knob. A native button with the
// switch role, named by `label`; a click fires qp-change with the value asked for (the owner sets `checked` once it
// holds).

import {KitElement, define, emit, h} from "./element.js";

class QpToggle extends KitElement {
	build() {
		this.control = h("button", {type: "button", role: "switch", class: "qp-switch",
			"on:click": () => emit(this, "qp-change", !this.checked)}, h("span", {class: "qp-knob"}));
		this.append(this.control);
	}

	update() {
		this.control.setAttribute("aria-checked", String(this.checked));
		this.control.disabled = this.disabled;
		if (this.label) this.control.setAttribute("aria-label", this.label);
	}
}

define("qp-toggle", QpToggle, {checked: Boolean, disabled: Boolean, label: String});
