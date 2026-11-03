// MoveWindow: a move shown in a Window from the bottom (the pump's How to do it, a tile in the Book): its name as the
// title (`heading`), its figure on the plinth (`stage`, a FigureStage lib/figure.js draws into), its QualityWord
// (`quality`, `quality-word`) with its record beside it (`record`, as the move keeps it) or Max set, none for a move
// that keeps no count (`bare`), its setup (the one safety fact) with How's numbered steps and Tips (`setup`, and the
// lists `how` and `tips`: kit/move-how.js), its stop line (`stop`), what the caller adds under them (`extra`: the
// Book's Include in pumps) and, where the caller has one, its Start pinned at the foot (`foot`, which the caller fills
// with a StartBlock: slice 36's rules, ns.StartFor). A move above the player's reach (`ahead`, its line: "Above your
// step for now") shows its figure and How to under that line, with no Max set (David, 2026-10-14: "you can still watch
// future excercises"; docs/safety.md, hazard 8: it never starts on its own, and its Start, the ladder's climb, stands
// in the "To get here" row). Opened from a tapped tile (`open`'s `origin`, the Moves grid's) it grows out of it and
// shrinks back into it as the player closes it (slice 46).

import {t} from "../lib/copy.js";
import {KitElement, define, h} from "./element.js";
import {howAndTips, stopLine} from "./move-how.js";

class QpMoveWindow extends KitElement {
	build() {
		this.window = h("qp-window", {placement: "bottom"});
		this.aheadLine = h("p", {class: "qp-type-subtitle qp-move-window-ahead"});
		this.figure = h("qp-figure-stage");
		this.wordEl = h("qp-quality-word");
		this.recordEl = h("span", {class: "qp-move-record"});
		this.lines = h("div", {class: "qp-move-window-lines"});
		this.more = h("div", {class: "qp-move-window-extra"});
		this.window.content.append(this.aheadLine, this.figure,
			h("div", {class: "qp-move-window-quality"}, this.wordEl, this.recordEl),
			this.lines, this.more);
		this.append(this.window);
		this.howLines ??= [];
		this.tipLines ??= [];
	}

	get stage() {
		this.ensure();
		return this.figure;
	}

	/** How's steps (one a line). */
	set how(value) {
		this.howLines = value ?? [];
		if (this.built) this.update();
	}

	get how() {
		return this.howLines ?? [];
	}

	/** Tips' lines, the move's own advice last. */
	set tips(value) {
		this.tipLines = value ?? [];
		if (this.built) this.update();
	}

	get tips() {
		return this.tipLines ?? [];
	}

	get extra() {
		this.ensure();
		return this.more;
	}

	/** The window's pinned foot, which a caller with a Start fills (none shows while it is empty). */
	get foot() {
		this.ensure();
		return this.window.foot;
	}

	/** Opens it, growing out of the tile `origin` returns (a Moves tile: qp-window's `open`). */
	open(origin = null) {
		this.ensure();
		this.update();
		this.window.open(origin);
	}

	close() {
		this.window.close();
	}

	update() {
		this.window.heading = this.heading ?? "";
		this.wordEl.hidden = !this.quality;
		if (this.quality) {
			this.wordEl.quality = this.quality;
			this.wordEl.word = this.qualityWord ?? "";
		}
		this.recordEl.hidden = this.bare;
		this.recordEl.replaceChildren(...(this.record
			? [h("small", {class: "qp-type-caps", text: t("ladders.record.label")}),
				h("span", {class: "qp-type-num-small", text: this.record})]
			: [h("small", {class: "qp-type-caps", text: t("kit.maxset")})]));
		this.aheadLine.textContent = this.ahead ?? "";
		this.aheadLine.hidden = !this.ahead;
		this.lines.replaceChildren(...howAndTips({setup: this.setup, how: this.how, tips: this.tips}),
			...[stopLine(this.stop)].filter(Boolean));
	}
}

define("qp-move-window", QpMoveWindow, {
	heading: String, quality: Number, "quality-word": String, record: String, bare: Boolean, ahead: String,
	setup: String, stop: String,
});
