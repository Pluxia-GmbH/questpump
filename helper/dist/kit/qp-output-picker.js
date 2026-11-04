// OutputPicker (desktop; docs/ui.md, "The pump on the computer": the output the player picks at the first pump): the
// outputs the sound and the voice can play on, as Rows, never a <select> (VoiceOver loses one beside a tray icon):
// this computer's sound first (the system's own choice, with the output it uses now), then each output the system
// lists, the picked one checked; picking one keeps it (`sound.output`, which plays the test on it, so the player hears
// where it goes) and fires qp-picked; Play a test sound plays it again; while the picked output is not connected, the
// note that this computer's sound plays instead. `ask` (the first pump's window) puts "Speakers or headset?" above the
// rows. It follows the `menu.sound` topic (helper/src-tauri/src/desk/audio.rs) and asks for the outputs
// again each time it shows (one plugged in since shows up).

import {intent, subscribe} from "../lib/bridge.js";
import {t} from "../lib/copy.js";
import {KitElement, define, emit, h} from "./element.js";

class QpOutputPicker extends KitElement {
	build() {
		this.head = h("h2", {class: "qp-type-heading", text: t("desk.sound.ask")});
		this.rows = h("qp-card", {rows: true});
		this.gone = h("p", {class: "qp-type-meta qp-words-block"});
		this.test = h("qp-row", {heading: t("desk.sound.test"), glyph: "mark_play", trailing: "action",
			"on:click": () => intent("sound.test").catch(console.error)});
		this.append(this.head, this.rows, this.gone, h("qp-card", {rows: true}, this.test));
		this.draw = (page) => this.render(page);
	}

	connectedCallback() {
		super.connectedCallback();
		this.stop = subscribe("menu.sound", this.draw);
		intent("sound.look").catch(console.error);
	}

	disconnectedCallback() {
		this.stop?.();
	}

	/** Keeps an output (null: this computer's sound) and says so. */
	pick(id, name) {
		intent("sound.output", {id, name}).then(() => emit(this, "qp-picked", id), console.error);
	}

	update() {
		this.head.hidden = !this.ask;
	}

	/** Draws the rows, made anew only when the outputs change (a row the focus is on keeps it as the topic moves). */
	render(page) {
		const shape = JSON.stringify([page.looked, page.outputs.map((o) => [o.id, o.name])]);
		if (shape !== this.shape) {
			this.shape = shape;
			this.system = h("qp-row", {heading: t("desk.sound.system"), glyph: "device_computer", trailing: "checked",
				"on:click": () => this.pick(null, "")});
			this.choices = page.outputs.map((output) => [output.id, h("qp-row", {heading: output.name,
				glyph: "rating_expertise", trailing: "checked", "on:click": () => this.pick(output.id, output.name)})]);
			const looking = page.looked ? null : h("qp-row", {heading: t("desk.sound.looking"),
				glyph: "rating_expertise", trailing: "shown"});
			this.rows.replaceChildren(...[this.system, looking, ...this.choices.map(([, row]) => row)].filter(Boolean));
		}
		const now = page.outputs.find((o) => o.default);
		this.system.meta = now ? t("desk.sound.system.now", {name: now.name}) : null;
		this.system.checked = page.output === null;
		for (const [id, row] of this.choices) row.checked = page.output === id;
		this.gone.hidden = !page.gone;
		this.gone.textContent = page.gone ? t("desk.sound.gone", {name: page.gone}) : "";
	}
}

define("qp-output-picker", QpOutputPicker, {ask: Boolean});
