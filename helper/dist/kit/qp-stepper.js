// Stepper: the count (a pump's count view): − and + round 64 keys either side of the value in numCount over its unit in
// caps (`unit`: reps or seconds; or the words of a count of something else, `unit-one` and `unit-other`, as the
// phones' reminders a day), a key at the count's least (`least`) or most (`most`) refusing; the record chip under it
// shown while the value passes the `record` (still under Reduce Motion). The value is a spin button: the arrows step
// it, and the count view types digits and Backspace into it (`type`, `erase`). A change fires qp-change with the value.
// The kg mode (`decimal`: a loaded move's weight box beside the reps, its words `unit-other`) holds a weight in tenths
// of a kg, none while the box is empty (no weight), shown in kg with one decimal; its − and + step as Progress.lua's
// ns.LoadStep does on every surface: from empty either key offers `first` (the plan, else the start range's low end or
// a kettlebell's first size at or above it), a kettlebell's `sizes` (tenths) are stepped through and stayed inside, any
// other weight steps `stride` tenths (load_box_step), to the next step from an odd weight, never under one step nor
// over `most`. The count view also types "." or "," (`point`) and empties it (`clear`), a second decimal rounding to
// the nearest tenth; "0." or "." stays as typed until a digit follows. It also stands compact at a row's end
// (`compact`, in a `.qp-stepper-line`: the Menu's Size on This computer, a counted set's Clean reps), its keys at the
// RoundButton's size beside the value with no unit (the row names it), which steps through `steps` where it has them
// (a list, ascending: its ends are the least and the most) and shows as its page's words where it has them (`text`,
// read as the value too), the value and its keys named by the page (`label`, `less-label`, `more-label`). `none` is
// what a 0 is (a counted set's "Not counted", 2026-10-23): it shows a dash and reads as those words. On the dial
// (slice 66: the pump's count set on its TimerRing at Done, `ring`, given before it is built) the ring is its value,
// − and + at its sides: the ring's face shows the value over its unit, its arc the value's share of `against` (the
// goal, else the record; full for any count without one), and its gold chip takes the record chip's place; the wheel
// steps it too.

import {t} from "../lib/copy.js";
import {kg} from "../lib/format.js";
import {KitElement, define, emit, h} from "./element.js";

/** The computer's decimal separator, which the kg box shows a typed point as (half a kg, in kg, without its digits). */
const POINT = kg(5).replace(/\d/g, "");

/** A count's unit in words: its seconds (`seconds`) or its reps, one or more. */
export function countUnit(value, seconds) {
	return seconds ? t(value === 1 ? "kit.second" : "kit.seconds") : t(value === 1 ? "kit.rep" : "kit.reps");
}

/** The wheel's travel one step takes on the dial (px; a line's travel, `deltaMode` 1, counts as one step). */
const WHEEL_STEP = 40;

/** Typed kg ("12", "12.", "12.55") in tenths of a kg, to the nearest tenth (a second decimal of 5 or more rounds up);
 * none for nothing typed. */
function tenths(text) {
	if (!text || text === ".") return null;
	const [whole, decimals = ""] = text.split(".");
	return Number(whole || 0) * 10 + Number(decimals[0] ?? 0) + (Number(decimals[1] ?? 0) >= 5 ? 1 : 0);
}

class QpStepper extends KitElement {
	build() {
		this.less = h("button", {type: "button", class: "qp-step-key", text: "−", "on:click": () => this.step(-1)});
		this.more = h("button", {type: "button", class: "qp-step-key", text: "+", "on:click": () => this.step(1)});
		this.number = h("span", {class: "qp-type-num-count"});
		this.unitWord = h("span", {class: "qp-type-caps"});
		if (this.ring) this.ring.setAttribute("aria-hidden", "true");
		this.control = h("div", {class: "qp-step-value", role: "spinbutton", tabindex: "0",
			"on:keydown": (e) => this.key(e)}, ...(this.ring ? [this.ring] : [this.number, this.unitWord]));
		if (this.ring) {
			this.wheeled = 0;
			this.addEventListener("wheel", (e) => this.wheel(e), {passive: false});
		}
		this.chip = h("qp-chip", {state: "record", label: t("kit.toast.record")});
		this.append(h("div", {class: "qp-stepper-row"}, this.less, this.control, this.more), this.chip);
		/** The kg mode's characters as typed, shown until a step, a clamp or a new value replaces them. */
		this.typed = null;
		/** A kettlebell's sizes in tenths of a kg, ascending; none for half-kg steps. */
		this.sizes ??= [];
		/** The values − and + step through, ascending; none for steps of one. */
		this.steps ??= [];
	}

	bounds() {
		if (this.steps.length) return [this.steps[0], this.steps.at(-1)];
		return [this.least ?? 0, this.most ?? Number.MAX_SAFE_INTEGER];
	}

	set(value) {
		const [least, most] = this.bounds();
		if (this.decimal) {
			const next = value === null || value <= 0 ? null : Math.min(most, value);
			if (next === this.value) return;
			this.value = next;
			emit(this, "qp-change", next);
			return;
		}
		const next = Math.max(least, Math.min(most, value));
		if (next === (this.value ?? 0)) return;
		this.value = next;
		emit(this, "qp-change", next);
	}

	/** The kg mode's weight one step up (1) or down (-1), Progress.lua ns.LoadStep: from empty `first`; a kettlebell's
	 * next or previous size, staying at the smallest and the biggest; else `stride` up to the next step (at most `most`)
	 * or down to the step under it (at least one step). */
	weightAfter(by) {
		const value = this.value;
		if (value === null) return this.first ?? null;
		if (this.sizes.length) {
			return by > 0 ? this.sizes.find((s) => s > value) ?? value : this.sizes.findLast((s) => s < value) ?? value;
		}
		const step = this.stride;
		if (by > 0) return Math.min(this.bounds()[1], value - (value % step) + step);
		return Math.max(step, value % step === 0 ? value - step : value - (value % step));
	}

	/** One step up (1) or down (-1). */
	step(by) {
		if (this.decimal) {
			this.typed = null;
			this.set(this.weightAfter(by));
			this.update();
			return;
		}
		const value = this.value ?? 0;
		if (!this.steps.length) {
			this.set(value + by);
			return;
		}
		this.set(by > 0 ? this.steps.find((s) => s > value) ?? value : this.steps.findLast((s) => s < value) ?? value);
	}

	/** A typed digit: appended to the value. */
	type(digit) {
		if (this.decimal) {
			this.typing(`${this.typed ?? ""}${digit}`);
			return;
		}
		this.set((this.value ?? 0) * 10 + digit);
	}

	/** The kg mode's "." or ",": the decimal point, once. */
	point() {
		if (this.decimal && !this.typed?.includes(".")) this.typing(`${this.typed || "0"}.`);
	}

	/** Backspace: the value's last digit gone (the kg mode's last character). */
	erase() {
		if (this.decimal) {
			this.typing((this.typed ?? (this.value === null ? "" : String(this.value / 10))).slice(0, -1));
			return;
		}
		this.set(Math.floor((this.value ?? 0) / 10));
	}

	/** The kg mode's box emptied: no weight. */
	clear() {
		this.typing("");
	}

	/** The kg mode's characters typed so far and the weight they make; past the most, the most in kg instead. "0." and
	 * "." make no weight yet and stay as typed for the digit after them. */
	typing(text) {
		const value = tenths(text);
		this.typed = text;
		this.set(value);
		if (value !== null && value > this.bounds()[1]) this.typed = null;
		this.update();
	}

	/** The wheel on the dial: up one step per `WHEEL_STEP` of travel up, down per one down. */
	wheel(e) {
		e.preventDefault();
		this.wheeled += e.deltaMode === 0 ? e.deltaY : Math.sign(e.deltaY) * WHEEL_STEP;
		while (Math.abs(this.wheeled) >= WHEEL_STEP) {
			const by = this.wheeled > 0 ? -1 : 1;
			this.wheeled += by * WHEEL_STEP;
			this.step(by);
		}
	}

	key(e) {
		if (e.key === "ArrowUp" || e.key === "ArrowRight") this.step(1);
		else if (e.key === "ArrowDown" || e.key === "ArrowLeft") this.step(-1);
		else return;
		e.preventDefault();
	}

	update() {
		this.less.setAttribute("aria-label", this.lessLabel ?? t("kit.stepper.less"));
		this.more.setAttribute("aria-label", this.moreLabel ?? t("kit.stepper.more"));
		if (this.label) this.control.setAttribute("aria-label", this.label);
		else this.control.removeAttribute("aria-label");
		this.number.className = this.compact ? "qp-type-body-strong" : "qp-type-num-count";
		if (this.decimal) {
			this.updateWeight();
			return;
		}
		const value = this.value ?? 0;
		const [least, most] = this.bounds();
		const blank = value === 0 && Boolean(this.none);
		const unit = this.text || this.compact || blank ? ""
			: this.unitOne && this.unitOther ? (value === 1 ? this.unitOne : this.unitOther)
				: countUnit(value, this.unit === "seconds");
		this.number.textContent = this.text ?? (blank ? "–" : String(value));
		this.unitWord.textContent = unit;
		this.unitWord.hidden = !unit;
		this.keys(value <= least, value >= most);
		this.control.setAttribute("aria-valuenow", String(value));
		this.control.setAttribute("aria-valuemin", String(least));
		if (this.most !== null || this.steps.length) this.control.setAttribute("aria-valuemax", String(most));
		this.control.setAttribute("aria-valuetext", this.text ?? (blank ? this.none : `${value} ${unit}`.trim()));
		const passed = this.record !== null && value > this.record;
		this.chip.hidden = !passed || Boolean(this.ring);
		if (this.ring) {
			const share = this.against ? value / this.against : value > 0 ? 1 : 0;
			this.ring.mode = {kind: "entry", text: this.number.textContent, unit, share, chip: passed};
		}
	}

	/** The minus and the plus disabled at their ends; a key that disables while it holds the focus hands it to the
	 * value first, so a keyboard player stepping to an end stays on the control rather than on the page's top. */
	keys(lessOff, moreOff) {
		const at = document.activeElement;
		if ((lessOff && at === this.less) || (moreOff && at === this.more)) this.control.focus();
		this.less.disabled = lessOff;
		this.more.disabled = moreOff;
	}

	/** The kg mode: the weight as typed or in kg (a dash while empty) over its words. */
	updateWeight() {
		const value = this.value;
		const unit = this.unitOther ?? "";
		this.number.textContent = this.typed ? this.typed.replace(".", POINT) : value === null ? "–" : kg(value);
		this.unitWord.textContent = unit;
		this.keys(this.weightAfter(-1) === value, this.weightAfter(1) === value);
		this.control.removeAttribute("aria-valuemin");
		this.control.removeAttribute("aria-valuemax");
		if (value === null) this.control.removeAttribute("aria-valuenow");
		else this.control.setAttribute("aria-valuenow", String(value / 10));
		this.control.setAttribute("aria-valuetext", value === null ? unit : `${kg(value)} ${unit}`);
		this.chip.hidden = true;
	}
}

define("qp-stepper", QpStepper, {value: Number, least: Number, most: Number, record: Number, unit: String,
	"unit-one": String, "unit-other": String, decimal: Boolean, first: Number, stride: Number, compact: Boolean,
	text: String, label: String, "less-label": String, "more-label": String, none: String, against: Number});
