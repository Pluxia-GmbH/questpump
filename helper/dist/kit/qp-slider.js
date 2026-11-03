// Slider (desktop): a level from 0 to 100 % as a row: its name (`heading`, which also names the control), the track
// with the gold fill to the level under a light knob, and the level in words ("80%"). A native range input, so the
// pointer drags it, the arrows step it by STRIDE, Page Up and Page Down by 10 and Home and End go to its ends, and
// VoiceOver and NVDA read it as a slider; it moves at once and fires qp-change with the level as the drag ends or a key
// steps. `value` is the owner's level, which the control takes only while the player is not moving it (dragging or
// holding the focus), so an answer that comes back late never pulls the knob from under the pointer or the keys.

import {t} from "../lib/copy.js";
import {KitElement, define, emit, h} from "./element.js";

/** A step of the arrow keys, 5 %: 2 dB on the sound's scale (desk/audio.rs `gain`), as WoW's own volume steps. */
const STRIDE = 5;

class QpSlider extends KitElement {
	build() {
		this.head = h("span", {class: "qp-row-title"});
		this.shown = h("span", {class: "qp-slider-value", "aria-hidden": "true"});
		this.control = h("input", {type: "range", class: "qp-range", min: "0", max: "100", step: String(STRIDE),
			"on:input": () => this.draw(), "on:change": () => this.moved(), "on:pointerdown": () => this.hold(),
			"on:blur": () => this.update()});
		this.append(h("span", {class: "qp-row-words"}, this.head),
			h("span", {class: "qp-slider-control"}, this.control, this.shown));
	}

	/** A drag holds the control until the pointer lets go anywhere. */
	hold() {
		this.dragging = true;
		const end = () => {
			this.dragging = false;
		};
		window.addEventListener("pointerup", end, {once: true, capture: true});
		window.addEventListener("pointercancel", end, {once: true, capture: true});
	}

	/** The level the player set is the value until the owner's answer comes, so the focus leaving keeps it. */
	moved() {
		this.dragging = false;
		this.value = Number(this.control.value);
		emit(this, "qp-change", this.value);
	}

	/** The fill and the words at the control's level. */
	draw() {
		const level = Number(this.control.value);
		const words = t("desk.slider.value", {percent: level});
		this.style.setProperty("--qp-slider-share", String(level / 100));
		this.shown.textContent = words;
		this.control.setAttribute("aria-valuetext", words);
	}

	update() {
		this.head.textContent = this.heading ?? "";
		this.control.setAttribute("aria-label", this.heading ?? "");
		if (!this.dragging && document.activeElement !== this.control) this.control.value = String(this.value ?? 100);
		this.draw();
	}
}

define("qp-slider", QpSlider, {value: Number, heading: String});
