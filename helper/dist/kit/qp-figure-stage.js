// FigureStage (iOS Kit/Figure.swift): the armour figure (lib/figure.js) in its square centred on the stage, on its gold
// plinth glow, the glow's middle under the figure's feet (the bottom of its box; at the plinth's own height from the
// foot under a move's glyph, where the table has no figure). `move` (a library move's key, or an extra's name: the
// standard-bearer `streak_banner`, its key pose alone), `side` ("other" mirrors a side move's other side), `colour`
// (#RRGGBB, the crimson without one), `still` (the key pose only: nothing invites a pump) and `calm` (the loop at a
// stroll: a figure standing by) say what it shows. `pausable` (a move's page, a ladder's page) adds the round mark at
// its bottom-trailing corner while the loop plays, the pause mark while it plays and the play mark while `paused`,
// firing qp-pause; under Reduce Motion a looping figure stands on its key pose and the mark plays its rep once ("Play
// the move once"). The figure itself answers a click as its mark does; the mark is the keyboard's way. `square` is the
// figure's container. `beat` and `from` phase a set of reps' loop to its go (lib/figure.js). `fill` (the PumpStage, Big
// view's) fits the figure's own box, as drawn on its side, as large as the stage holds it in its middle (kit.css
// `qp-figure-stage[fill]`, the box's numbers set here), its square clipped by the stage, whose edges fade;
// `hold(figure)` shows a MoveFigure its owner keeps from page to page, and the stage makes its own only when it holds
// none.

import {t} from "../lib/copy.js";
import {MoveFigure, drawnBox, figureOf} from "../lib/figure.js";
import {subscribe} from "../lib/bridge.js";
import {KitElement, define, emit, h} from "./element.js";

const motion = window.matchMedia("(prefers-reduced-motion: reduce)");

class QpFigureStage extends KitElement {
	build() {
		this.plinth = h("div", {class: "qp-plinth", "aria-hidden": "true"});
		this.box = h("div", {class: "qp-figure-square", "on:click": () => this.press()}, this.plinth);
		this.mark = h("button", {type: "button", class: "qp-figure-mark", "on:click": () => this.press()});
		this.append(this.box, this.mark);
		this.redraw = () => this.update();
	}

	connectedCallback() {
		super.connectedCallback();
		this.stopTable = subscribe("figures", this.redraw);
		motion.addEventListener("change", this.redraw);
	}

	disconnectedCallback() {
		this.stopTable?.();
		motion.removeEventListener("change", this.redraw);
		if (!this.held) this.figure?.stop();
	}

	/** Shows `figure`, a MoveFigure its owner keeps from page to page (Big view's: one film in phase from the countdown
	 * through the run), in place of its own; the stage going never stops it. */
	hold(figure) {
		this.ensure();
		if (this.figure && !this.held) this.figure.stop();
		this.figure?.el.remove();
		this.box.append(figure.el);
		this.figure = figure;
		this.held = true;
		this.update();
	}

	get square() {
		this.ensure();
		return this.box;
	}

	/** What the mark does now: pause (or play on) a playing loop, play a standing loop's rep once, or nothing. */
	action() {
		// an extra (the standard-bearer) has its still alone
		if (this.still || !figureOf(this.move) || figureOf(this.move).extra) return null;
		if (motion.matches) return "once";
		return this.pausable ? "pause" : null;
	}

	press() {
		switch (this.action()) {
			case "once":
				this.figure?.playOnce();
				break;
			case "pause":
				this.paused = !this.paused;
				emit(this, "qp-pause", this.paused);
				break;
			default:
		}
	}

	update() {
		if (this.isConnected) {
			if (!this.figure) {
				this.figure = new MoveFigure();
				this.box.append(this.figure.el);
			}
			this.figure.show({move: this.move, side: this.side, colour: this.colour, still: this.still,
				calm: this.calm, paused: this.paused, beat: this.beat, from: this.from});
		}
		const figure = figureOf(this.move);
		if (figure) this.style.setProperty("--qp-figure-feet", String(figure.box[3]));
		else this.style.removeProperty("--qp-figure-feet");
		const [left, top, right, bottom] = (this.fill && drawnBox(this.move, this.side)) || [0, 0, 1, 1];
		for (const [name, value] of [["l", left], ["t", top], ["w", right - left], ["h", bottom - top]]) {
			this.style.setProperty(`--qp-figure-${name}`, String(value));
		}
		this.toggleAttribute("data-feet", Boolean(figure));
		const action = this.action();
		this.mark.hidden = !action;
		const icon = action === "pause" && !this.paused ? "mark_pause" : "mark_play";
		this.mark.replaceChildren(h("qp-art", {name: icon}));
		const label = action === "once" ? "pump.figure.action"
			: this.paused ? "kit.figure.play.action" : "kit.figure.pause.action";
		this.mark.setAttribute("aria-label", t(label));
		this.box.classList.toggle("qp-pressable", Boolean(action));
		this.box.setAttribute("aria-label", this.label ?? t("pump.figure.label"));
		this.box.setAttribute("role", "img");
	}
}

define("qp-figure-stage", QpFigureStage, {
	move: String, side: String, colour: String, still: Boolean, calm: Boolean, pausable: Boolean, paused: Boolean,
	label: String, beat: Number, from: Number, fill: Boolean,
});
