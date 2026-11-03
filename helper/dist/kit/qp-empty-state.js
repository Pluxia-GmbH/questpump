// EmptyState: an empty page's content, top-aligned, space32 under what stands above it: a glyph, a heading and one gold
// button (`action`, its words, firing qp-action), never a line under the heading (2026-10-22).

import {KitElement, define, emit, h} from "./element.js";

class QpEmptyState extends KitElement {
	build() {
		this.glyphEl = h("qp-art", {class: "qp-empty-glyph"});
		this.head = h("h2", {class: "qp-type-heading"});
		this.button = h("qp-primary-button", {"on:click": () => emit(this, "qp-action")});
		this.append(this.glyphEl, this.head, this.button);
	}

	update() {
		if (this.glyph && this.glyphEl.name !== this.glyph) this.glyphEl.name = this.glyph;
		this.glyphEl.hidden = !this.glyph;
		this.head.textContent = this.heading ?? "";
		this.button.label = this.action ?? "";
		this.button.hidden = !this.action;
	}
}

define("qp-empty-state", QpEmptyState, {glyph: String, heading: String, action: String});
