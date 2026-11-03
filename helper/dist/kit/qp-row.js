// Row: a list row (design v3 §1 rule 8), at least 56 high (64 with a meta line) and one control as a whole: a leading
// move's tile (`move`, its key, in its `quality`'s edge; the 44 MoveTile, which the row's words name, as the phones'
// RowLead.Tile: Swap's rows, 2026-10-26), glyph (`glyph`, an art piece; `tile` sets it in a KindTile, a place's) or a
// step's number (`step`), its main words in bodyStrong (`heading`) with its meta lines (`meta`, or `lines` for several:
// its own state or values, never what it does, 2026-10-22), what a screen reader hears after its name (`hint`, what it
// does: its aria-description), and what closes it (`trailing`): `nav` (a chevron: it opens a page), `value` (a value
// and the chevron; `value-spoken` the words a screen reader hears for it, "75 XP" for "+75"), `shown` (a value alone:
// the row is no control), `toggle` (a switch: the row is the switch, firing qp-change), `action` (nothing: the row does
// it), `external` (the way out: it opens a page outside the app), `checked` (a choice's check while `checked`),
// `expander` (a chevron turned down while `expanded`). `extra` stands a node of its own before the end (Pump home's
// embers marks, slice 55). `segment` stands a control of its own at the row's end, a chevron behind a hairline at the
// row's height, which a screen reader hears by these words and whose click fires qp-segment, never the row's click (the
// pump's swap row, "More ways to swap", 2026-10-26). `destructive` draws its words in stop, `disabled` in text3, inert,
// its segment too.

import {KitElement, define, emit, h} from "./element.js";

class QpRow extends KitElement {
	build() {
		this.tileEl = h("qp-move-tile", {size: 44, "aria-hidden": "true"});
		this.glyphEl = h("qp-art", {class: "qp-row-glyph"});
		this.stepEl = h("span", {class: "qp-row-step qp-type-num-small", "aria-hidden": "true"});
		this.head = h("span", {class: "qp-row-title"});
		this.metas = h("span", {class: "qp-row-meta"});
		this.valueEl = h("span", {class: "qp-row-value"});
		this.spokenEl = h("span", {class: "qp-unseen"});
		this.end = h("span", {class: "qp-row-end", "aria-hidden": "true"});
		this.words = h("span", {class: "qp-row-words"}, this.head, this.metas);
		this.segmentEl = h("button", {type: "button", class: "qp-row-segment"},
			h("span", {class: "qp-row-end qp-end-nav", "aria-hidden": "true"}));
		this.segmentEl.addEventListener("click", (e) => {
			e.stopPropagation();
			emit(this, "qp-segment");
		});
		this.metaLines ??= [];
	}

	/** A node before the row's end (its marks), or none. */
	set extra(value) {
		this.extraEl = value ?? null;
		if (this.built) this.update();
	}

	/** Several meta lines. */
	set lines(value) {
		this.metaLines = value ?? [];
		if (this.built) this.update();
	}

	get lines() {
		return this.metaLines ?? [];
	}

	update() {
		const trailing = this.trailing ?? "nav";
		const tag = trailing === "shown" ? "div" : "button";
		if (this.control?.localName !== tag) {
			this.control = h(tag, {class: "qp-row-body", ...(tag === "button" ? {type: "button"} : {})});
			if (tag === "button") {
				this.control.addEventListener("click", (e) => {
					if (this.trailing === "toggle") {
						e.stopPropagation();
						emit(this, "qp-change", !this.checked);
					}
				});
			}
			this.replaceChildren(this.control);
		}
		if (this.segment) {
			this.segmentEl.setAttribute("aria-label", this.segment);
			this.segmentEl.disabled = this.disabled;
			if (this.segmentEl.parentNode !== this) this.append(this.segmentEl);
		} else {
			this.segmentEl.remove();
		}
		this.classList.toggle("qp-has-segment", Boolean(this.segment));
		const leading = this.move ? this.tileEl : this.glyph ? this.glyphEl : this.step ? this.stepEl : null;
		if (this.move) Object.assign(this.tileEl, {move: this.move, quality: this.quality});
		if (this.glyph && this.glyphEl.name !== this.glyph) this.glyphEl.name = this.glyph;
		this.glyphEl.classList.toggle("qp-row-tile", this.tile);
		this.stepEl.textContent = this.step ?? "";
		this.head.textContent = this.heading ?? "";
		const lines = this.meta ? [this.meta, ...this.lines] : this.lines;
		this.metas.replaceChildren(...lines.filter(Boolean).map((line) => h("span", {text: line})));
		this.metas.hidden = !this.metas.childElementCount;
		this.valueEl.textContent = this.value ?? "";
		this.valueEl.toggleAttribute("aria-hidden", Boolean(this.valueSpoken));
		this.spokenEl.textContent = this.valueSpoken ?? "";
		this.end.className = `qp-row-end qp-end-${trailing}`;
		this.end.replaceChildren(trailing === "toggle" ? h("span", {class: "qp-switch-look"}, h("span", {class: "qp-knob"}))
			: trailing === "checked" && this.checked ? h("qp-art", {name: "mark_check"})
				: trailing === "external" ? h("qp-art", {name: "mark_arrow"}) : "");
		const spoken = this.value && this.valueSpoken ? this.spokenEl : null;
		const parts = [leading, this.words, this.value ? this.valueEl : null, spoken, this.extraEl, this.end]
			.filter(Boolean);
		this.control.replaceChildren(...parts);
		this.classList.toggle("qp-destructive", this.destructive);
		if (this.hint) this.control.setAttribute("aria-description", this.hint);
		else this.control.removeAttribute("aria-description");
		if (tag === "button") {
			this.control.disabled = this.disabled;
			if (trailing === "toggle") {
				this.control.setAttribute("role", "switch");
				this.control.setAttribute("aria-checked", String(this.checked));
			} else {
				this.control.removeAttribute("role");
				this.control.removeAttribute("aria-checked");
			}
			if (trailing === "checked") this.control.setAttribute("aria-pressed", String(this.checked));
			else this.control.removeAttribute("aria-pressed");
			if (trailing === "expander") this.control.setAttribute("aria-expanded", String(this.expanded));
			else this.control.removeAttribute("aria-expanded");
		}
	}
}

define("qp-row", QpRow, {
	heading: String, meta: String, hint: String, move: String, quality: Number, glyph: String, tile: Boolean,
	step: Number, value: String, "value-spoken": String, trailing: String, checked: Boolean, expanded: Boolean,
	destructive: Boolean, disabled: Boolean, segment: String,
});
