// DetailBar: a pushed page's top (design v3 §3): a round back button (qp-back), the title (`heading`) in the
// titleDetail role (none on a page whose heading stands in its body) and an optional trailing round button (its
// `trailing` icon, firing qp-trailing), 52 high. `action` (with_primary, a chapter's Share day): a compact PrimaryButton
// at the trailing edge.

import {KitElement, define, emit, h} from "./element.js";

class QpDetailBar extends KitElement {
	build() {
		this.back = h("qp-round-button", {icon: "back", "on:click": () => emit(this, "qp-back")});
		this.head = h("h1", {class: "qp-type-title-detail"});
		this.end = h("qp-round-button", {"on:click": () => emit(this, "qp-trailing")});
		this.append(this.back, this.head, this.end);
	}

	update() {
		this.head.textContent = this.heading ?? "";
		this.head.hidden = !this.heading;
		this.end.hidden = !this.trailing;
		if (this.trailing) this.end.icon = this.trailing;
	}

	/** The compact primary at the trailing edge (none: null). */
	set action(el) {
		this.ensure();
		if (el === this.actionEl) return;
		this.actionEl?.remove();
		this.actionEl = el ?? null;
		if (el) this.append(el);
	}
}

define("qp-detail-bar", QpDetailBar, {heading: String, trailing: String});
