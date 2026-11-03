// StartBlock: a step's card's and a move window's one Start (slice 36's design; the Window's states primary_note and
// primary_block in contract/design/screens.json): the gold PrimaryButton with its label (`label`, verb, move and goal;
// `state` the button's: normal, disabled while a surface's own state holds it, with its reason in words beside it,
// `note`, or busy while the engine folds), the card's lines that go with the start under it (`lines`), and, where
// nothing starts, the refusal's lines with their link rows instead of a button (`links`: [{heading, glyph, act}], a row
// each that opens Your setup, a move's Include in pumps or the pain switch). `block` (wherever the Start trains another
// move than the card or the window shows: a step ahead, a kept step, a family not open yet, a move above the reach, the
// player's own step's path or the easier move stepped down to, a stretch check or what opens a waiting move, slice 42)
// stands it in the "To get here" row, the tile of the move it trains (`tile-move`, `tile-quality`) beside the button,
// the caps label over it ("To get here"; `own`, the player's own step whose Start trains a stage of its path, an easier
// move or a place start: "Train with") and the reason under
// it when greyed, in the caps label's place, so the clicked step's figure never stands over another move's button.
// `aside` (the player's own cardio step's Long Haul, slice 55: a SecondaryButton as high as the Start) stands in the
// Start's row after it where both fit with their words, else under it. Fires qp-start when the Start is pressed.

import {t} from "../lib/copy.js";
import {KitElement, define, emit, h} from "./element.js";

class QpStartBlock extends KitElement {
	build() {
		this.caps = h("span", {class: "qp-type-caps qp-start-caps"});
		this.tile = h("qp-move-tile", {size: "44", "aria-hidden": "true"});
		this.button = h("qp-primary-button", {"on:click": () => {
			if ((this.state ?? "normal") === "normal") emit(this, "qp-start");
		}});
		this.row = h("div", {class: "qp-start-row"}, this.tile, this.button);
		// the focus a redraw keeps (lib/focus.js) goes to the Start itself
		this.control = this.button;
		this.noteEl = h("p", {class: "qp-type-meta qp-start-note"});
		this.linesEl = h("div", {class: "qp-start-lines"});
		this.linksEl = h("div", {class: "qp-start-links"});
		this.append(this.caps, this.row, this.noteEl, this.linesEl, this.linksEl);
		this.lineList = this.lineList ?? [];
		this.linkList = this.linkList ?? [];
	}

	/** The card's lines that go with the start ("3 fresh earns it.", "Today's day is in."). */
	set lines(value) {
		this.lineList = value ?? [];
		if (this.built) this.update();
	}

	get lines() {
		return this.lineList ?? [];
	}

	/** A control beside the Start in its row (the Long Haul's SecondaryButton), or none. */
	set aside(value) {
		this.asideEl = value ?? null;
		if (this.built) this.update();
	}

	/** The refusal's link rows, each {heading, glyph, act}. */
	set links(value) {
		this.linkList = value ?? [];
		if (this.built) this.update();
	}

	/** The Start's native button, for a view's primary and the focus. */
	get primary() {
		this.ensure();
		return this.label ? this.button : null;
	}

	update() {
		const shown = Boolean(this.label);
		const block = this.block && shown;
		const greyed = (this.state ?? "normal") !== "normal";
		this.button.hidden = !shown;
		this.button.label = this.label ?? "";
		this.button.state = this.state ?? "normal";
		this.row.classList.toggle("qp-start-block", block);
		// moved only when it changed, so a focused control keeps the focus through an update
		if (this.shownAside !== this.asideEl) {
			this.shownAside?.remove();
			if (this.asideEl) {
				this.asideEl.classList.add("qp-start-aside");
				this.asideEl.tall = true;
				this.row.append(this.asideEl);
			}
			this.shownAside = this.asideEl;
		}
		this.tile.hidden = !block;
		if (block) Object.assign(this.tile, {move: this.tileMove ?? null, quality: this.tileQuality ?? null});
		this.caps.textContent = t(this.own ? "ladders.card.foot.own" : "ladders.card.foot");
		this.caps.hidden = !block || (greyed && Boolean(this.note));
		this.noteEl.textContent = this.note ?? "";
		this.noteEl.hidden = !this.note;
		this.linesEl.replaceChildren(...this.lineList.map((line) => h("p", {class: "qp-type-meta", text: line})));
		this.linksEl.replaceChildren(...this.linkList.map((link) => h("qp-row", {heading: link.heading,
			glyph: link.glyph, trailing: "nav", "on:click": link.act})));
		this.linksEl.hidden = !this.linkList.length;
		this.hidden = !shown && !this.asideEl && !this.note && !this.lineList.length && !this.linkList.length;
	}
}

define("qp-start-block", QpStartBlock, {label: String, state: String, note: String, block: Boolean, own: Boolean,
	"tile-move": String, "tile-quality": Number});
