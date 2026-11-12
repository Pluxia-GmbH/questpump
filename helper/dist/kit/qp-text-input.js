// TextInput: a text field in a card well with a line edge (lineStrong while it has the keyboard), body words, the
// prompt in text3. `mode`: `line` (one line that scrolls sideways; Enter fires qp-submit), `prose` (the player's own
// sentences: three lines high from the first, growing to `lines`, with capitals and corrections), `search` (one line
// with a clear control that empties it, named `clear-label`, else as the phones' Book names it: Clear the search),
// `time` (a time of day in the system's own field and the computer's own format, its value "HH:MM" on the 24-hour
// clock: the desktop's stand-in for the phones' time pickers), `code` (a code of `digits` digits, 6 by default: a box
// for each over one field that keeps digits alone, which the system's one-time-code fill fills (`autocomplete
// one-time-code`), the next box edged while the field has the keyboard: the phones' code boxes). `masked` shows a
// `line` field's text as dots, a password field's, while it is set (a typed restore code: views/page.js `heldCode`). Named `label` for a screen reader; `value` reads and
// sets its text; `max` the most characters it takes (a place's name); `disabled` while what it holds is on its way;
// `inputmode` the keyboard a touch screen shows (`numeric` for a number, a code's own); `error` a line under the field
// in stop, which marks the field invalid and describes it to a screen reader, until it is set empty, and which a
// screen reader hears as it is set (lib/focus.js `announce`: the open window's live region, else the page's), again at
// each set, so a Done
// pressed twice on the same wrong value says it twice.

import {t} from "../lib/copy.js";
import {announce} from "../lib/focus.js";
import {feedback} from "../lib/sound.js";
import {KitElement, define, emit, h, styled} from "./element.js";

/** A code's digits where the field names none. */
const DIGITS = 6;

/** How many fields were built: each error line's id. */
let fields = 0;

class QpTextInput extends KitElement {
	build() {
		const mode = this.mode ?? "line";
		const code = mode === "code";
		this.control = mode === "prose"
			? h("textarea", {rows: String(Math.max(3, this.lines ?? 3)), spellcheck: "true", autocapitalize: "sentences"})
			: h("input", {type: mode === "search" || mode === "time" ? mode : "text", spellcheck: "false",
				autocapitalize: "off", autocomplete: code ? "one-time-code" : "off", "on:keydown": (e) => {
					if (e.key === "Enter" && !e.isComposing) emit(this, "qp-submit");
				}});
		this.boxes = [];
		if (code) {
			const digits = this.digits ?? DIGITS;
			// digits alone, before any listener of the page reads the field (no maxlength, which would cut a pasted
			// "123 456" before its digits are read)
			this.control.addEventListener("input", () => {
				const kept = this.control.value.replace(/\D/g, "").slice(0, digits);
				if (kept !== this.control.value) this.control.value = kept;
				this.drawBoxes();
			});
			this.control.addEventListener("focus", () => this.drawBoxes());
			this.control.addEventListener("blur", () => this.drawBoxes());
			this.boxes = Array.from({length: digits}, () => h("span", {class: "qp-code-box"}));
		}
		this.clear = h("button", {type: "button", class: "qp-clear", hidden: true, "on:click": () => {
			this.control.value = "";
			this.control.dispatchEvent(new Event("input", {bubbles: true}));
			this.control.focus();
		}}, h("qp-art", {name: "mark_close"}));
		this.control.addEventListener("input", () => {
			this.clear.hidden = !this.control.value || this.mode !== "search";
		});
		fields += 1;
		this.errorEl = h("span", {class: "qp-input-error", id: `qp-input-error-${fields}`, hidden: true});
		const well = code
			? styled(h("div", {class: "qp-input-well qp-code-well"},
				h("div", {class: "qp-code-boxes", "aria-hidden": "true"}, ...this.boxes), this.control),
			{"--qp-code-digits": this.boxes.length})
			: h("div", {class: "qp-input-well"}, this.control, this.clear);
		this.append(well, this.errorEl);
	}

	/** A code's boxes: a digit each as typed, the next one (the last once full) edged while the field has the keyboard. */
	drawBoxes() {
		const typed = this.control.value;
		const next = Math.min(typed.length, this.boxes.length - 1);
		const focused = document.activeElement === this.control;
		this.boxes.forEach((box, i) => {
			box.textContent = typed[i] ?? "";
			box.classList.toggle("qp-code-next", focused && i === next);
		});
	}

	attributeChangedCallback(name, old, value) {
		super.attributeChangedCallback();
		if (name === "error" && value && this.built) {
			announce(value);
			// what the field took failed and its error shows (slice 72: a wrong code, an address refused), once as it
			// comes
			if (value !== old) feedback("error");
		}
	}

	get value() {
		return this.control?.value ?? "";
	}

	set value(text) {
		if (!this.control) return;
		this.control.value = text;
		if (this.boxes.length) this.drawBoxes();
	}

	update() {
		if ((this.mode ?? "line") === "line") this.control.type = this.masked ? "password" : "text";
		this.control.disabled = this.disabled;
		this.control.placeholder = this.prompt ?? "";
		if (this.max) this.control.maxLength = this.max;
		else this.control.removeAttribute("maxlength");
		if (this.label) this.control.setAttribute("aria-label", this.label);
		const inputmode = this.inputmode ?? (this.boxes.length ? "numeric" : null);
		if (inputmode) this.control.setAttribute("inputmode", inputmode);
		else this.control.removeAttribute("inputmode");
		this.errorEl.textContent = this.error ?? "";
		this.errorEl.hidden = !this.error;
		if (this.error) {
			this.control.setAttribute("aria-invalid", "true");
			this.control.setAttribute("aria-describedby", this.errorEl.id);
		} else {
			this.control.removeAttribute("aria-invalid");
			this.control.removeAttribute("aria-describedby");
		}
		this.clear.setAttribute("aria-label", this.clearLabel ?? t("ladders.search.clear"));
		this.clear.hidden = !this.control.value || this.mode !== "search";
	}
}

define("qp-text-input", QpTextInput, {
	mode: String, lines: Number, digits: Number, label: String, prompt: String, "clear-label": String, max: Number,
	disabled: Boolean, inputmode: String, error: String, masked: Boolean,
});
