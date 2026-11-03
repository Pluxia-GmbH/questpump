// FocusVeil: the veil over a pump whose window lost the focus mid-pump (Rust's WindowEvent::Focused, the `shell`
// topic's `focused`): the pump dimmed under one line (`message`: "Click to control the pump", since Space would reach the
// game). A click anywhere gives the window the focus back, and its owner lifts the veil when `focused` says so; the
// clock and the voice run on under it.

import {KitElement, define, h} from "./element.js";

class QpFocusVeil extends KitElement {
	build() {
		this.line = h("p", {class: "qp-type-heading"});
		this.append(this.line);
	}

	update() {
		this.line.textContent = this.message ?? "";
	}
}

define("qp-focus-veil", QpFocusVeil, {message: String});
