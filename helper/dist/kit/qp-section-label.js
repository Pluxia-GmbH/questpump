// SectionLabel: a caps label 20 above its card and 8 over it, a heading for a screen reader's headings list.

import {KitElement, define, h} from "./element.js";

class QpSectionLabel extends KitElement {
	build() {
		this.head = h("h2", {class: "qp-type-caps"});
		this.append(this.head);
	}

	update() {
		this.head.textContent = this.label ?? "";
	}
}

define("qp-section-label", QpSectionLabel, {label: String});
