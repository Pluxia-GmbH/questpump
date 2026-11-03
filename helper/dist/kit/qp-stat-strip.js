// StatStrip: five cells in a card, each a stat's glyph, its number in numStat and its short name in caps, each a button
// firing qp-open with its index (`cells`: a list of {glyph, value, name}). `still` (a step's card, 2026-10-29: its
// record, its range and its stars): the stats' cells, none a button; a cell's `stars` ({stars, total}) draws its
// RungStars in the value's place. The cells are made again only when their number or the state changes; new values
// redraw them in place, so a focused cell keeps the focus.

import {KitElement, define, emit, h} from "./element.js";

/** A stat's cell, a button (none where the strip is `still`), and how it draws a cell's values. */
function statCell(strip, index, still) {
	const glyph = h("qp-art");
	const value = h("span", {class: "qp-type-num-stat"});
	const stars = h("qp-rung-stars");
	const name = h("span", {class: "qp-type-caps"});
	const el = still ? h("div", {class: "qp-stat-cell"}, glyph, value, stars, name)
		: h("button", {type: "button", class: "qp-stat-cell", "on:click": () => emit(strip, "qp-open", index)},
			glyph, value, stars, name);
	return {el, draw(cell) {
		glyph.hidden = !cell.glyph;
		if (cell.glyph) glyph.name = cell.glyph;
		value.hidden = Boolean(cell.stars);
		value.textContent = cell.value ?? "";
		stars.hidden = !cell.stars;
		if (cell.stars) Object.assign(stars, cell.stars);
		name.textContent = cell.name;
	}};
}

class QpStatStrip extends KitElement {
	set cells(value) {
		this.cellData = value ?? [];
		if (this.built) this.update();
	}

	get cells() {
		return this.cellData ?? [];
	}

	update() {
		const cells = this.cells;
		if (this.drawn?.still !== this.still || this.drawn.cells.length !== cells.length) {
			this.drawn = {still: this.still, cells: cells.map((_, index) => statCell(this, index, this.still))};
			this.replaceChildren(...this.drawn.cells.map((cell) => cell.el));
		}
		cells.forEach((cell, index) => this.drawn.cells[index].draw(cell));
	}
}

define("qp-stat-strip", QpStatStrip, {still: Boolean});
