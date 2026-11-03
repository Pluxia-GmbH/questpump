// MoveFigurePair (iOS Kit/Figure.swift): a move's still right-versus-wrong pair (a pump's Ready view on a move's first
// appearance since 2026-10-24, the Book's move page): the key pose and the fault pose side by side, each in its half at its own shape,
// never taller than the half is wide, in the figure's colour (`colour`, #RRGGBB; the crimson without one, as the game
// tints its pair as it tints the figure) and mirrored as the pack mirrors the move (`side` "other" mirrors it again),
// drawn lighten with their cut edges faded (lib/figure.js `PairStill`); under the key pose the check and Right, under
// the fault pose the cross and Wrong (a shape and a word, never colour alone) over the fault's words. Stills only: a
// fault never moves. `move` is the library move's key and `name` its words, which a screen reader says over the key
// pose ("Right: <the move>"); the fault's ("Wrong: <the fault>") come from the figures' table. Without a pair (no
// fault, no table) nothing is drawn.

import {subscribe} from "../lib/bridge.js";
import {t} from "../lib/copy.js";
import {PairStill, pairOf} from "../lib/figure.js";
import {KitElement, define, h} from "./element.js";

class QpMoveFigurePair extends KitElement {
	build() {
		this.right = new PairStill();
		this.wrong = new PairStill();
		this.faultWords = h("span", {class: "qp-type-meta"});
		const side = (still, glyph, word, words) => h("figure", {role: "img"},
			h("div", {class: "qp-pair-cell"}, still.canvas),
			h("figcaption", {"aria-hidden": "true"}, h("span", {class: "qp-pair-word"}, h("qp-art", {name: glyph}),
				h("span", {text: word})), words ?? ""));
		this.rightEl = side(this.right, "mark_check", t("pump.figure.right"));
		this.wrongEl = side(this.wrong, "mark_close", t("pump.figure.wrong"), this.faultWords);
		this.append(this.rightEl, this.wrongEl);
		this.redraw = () => this.update();
	}

	connectedCallback() {
		super.connectedCallback();
		this.stopTable = subscribe("figures", this.redraw);
	}

	disconnectedCallback() {
		this.stopTable?.();
	}

	update() {
		const pair = pairOf(this.move, this.side);
		this.hidden = !pair;
		if (!pair) return;
		this.right.show(pair, "key", this.colour);
		this.wrong.show(pair, "fault", this.colour);
		this.faultWords.textContent = pair.fault;
		this.rightEl.setAttribute("aria-label", t("pump.figure.right.label", {move: this.name ?? ""}));
		this.wrongEl.setAttribute("aria-label", t("pump.figure.wrong.label", {fault: pair.fault}));
	}
}

define("qp-move-figure-pair", QpMoveFigurePair, {move: String, name: String, colour: String, side: String});
