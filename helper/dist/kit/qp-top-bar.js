// TopBar: a root's top (design v3 §3): its title (`heading`) in the title role at the leading edge and the round gear
// that opens the Menu at the trailing one (qp-menu), 52 high. `action` (with_primary, the Saga's Share today): a compact
// PrimaryButton before the gear.

import {KitElement, define, emit, h} from "./element.js";

class QpTopBar extends KitElement {
	build() {
		this.head = h("h1", {class: "qp-type-title"});
		this.gear = h("qp-round-button", {icon: "menu", "on:click": () => emit(this, "qp-menu")});
		this.append(this.head, this.gear);
	}

	update() {
		this.head.textContent = this.heading ?? "";
	}

	/** The compact primary before the gear (none: null). */
	set action(el) {
		this.ensure();
		if (el === this.actionEl) return;
		this.actionEl?.remove();
		this.actionEl = el ?? null;
		if (el) this.insertBefore(el, this.gear);
	}
}

define("qp-top-bar", QpTopBar, {heading: String});
