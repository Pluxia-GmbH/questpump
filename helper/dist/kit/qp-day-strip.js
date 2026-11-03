// DayStrip (the Saga, 2026-10-15; Tokens.Surface.DayStrip; contract/fixtures/history.json `saga` rule 9): the game day
// from its turn to the next as one line, each pump of the day a bar at its start (lib/saga.js `placeBars`: at one
// pitch for the day, at or right of its start, the gap closing and then every bar narrowing on a crowded hour, which
// reads as a block at its hour), its moves stacked by kind from the axis (strength, cardio, mobility, yoga; a run of
// one kind one block of its own height, every block shrinking alike past three rows), each part of the day with moves
// its total over its bars and its name under them, both rows moved apart, "04:00" at both ends where the names leave
// them room (`stripWords`), on dashed grid lines. `bars` [{minute, bucket, kinds}] and `buckets` (the parts' totals)
// are its data; `reveal` grows the bars from the axis left to right once (sagaReveal; none under Reduce Motion, the
// tokens' zero); it reads as its `label` (the parts' totals).

import {LOCALE} from "../lib/format.js";
import {barRuns, blockSize, number, partTotals, partWord, placeBars, stripWords} from "../lib/saga.js";
import {svg} from "../lib/charts.js";
import {KitElement, define, token} from "./element.js";

/** The strip's height in px above its axis words: three rows of blocks and the totals' line. */
const TOTALS = 18;
const WORDS = 16;

class QpDayStrip extends KitElement {
	build() {
		this.barsData ??= [];
		this.bucketData ??= [0, 0, 0, 0];
		this.setAttribute("role", "img");
		// a new width draws again; the observer's first report of the width drawn already changes nothing (a grow
		// under way keeps its bars)
		new ResizeObserver(() => {
			if (Math.round(this.clientWidth) !== this.drawnWidth) this.draw();
		}).observe(this);
	}

	set bars(value) {
		this.barsData = value ?? [];
		if (this.built) this.update();
	}

	set buckets(value) {
		this.bucketData = value ?? [0, 0, 0, 0];
		if (this.built) this.update();
	}

	update() {
		if (this.label) this.setAttribute("aria-label", this.label);
		this.draw();
	}

	/** Grows the bars from the axis, left to right (the Saga's first open each game day). */
	grow() {
		const step = token(this, "--qp-motion-saga-reveal-stagger") || 0;
		this.querySelectorAll(".qp-strip-bar").forEach((bar, i) => {
			bar.style.animationDelay = `${i * step}ms`;
			bar.classList.add("qp-growing");
		});
	}

	draw() {
		const width = Math.round(this.clientWidth);
		if (!width) return;
		this.drawnWidth = width;
		const n = (name) => token(this, `--qp-surface-day-strip-${name}`);
		const [barW, gap, blockH, blockGap, rows] = [n("bar-width"), n("bar-gap"), n("block-height"), n("block-gap"),
			n("row-count")];
		const plotH = rows * blockH + (rows - 1) * blockGap;
		const axis = TOTALS + plotH + 0.5;
		const height = axis + WORDS;
		const root = svg("svg", {viewBox: `0 0 ${width} ${height}`, width, height, "aria-hidden": "true"});
		for (let r = 1; r <= rows; r += 1) {
			const y = axis - 0.5 - r * (blockH + blockGap) + blockGap / 2;
			root.append(svg("line", {x1: 0, x2: width, y1: y, y2: y, class: "qp-strip-grid"}));
		}
		root.append(svg("line", {x1: 0, x2: width, y1: axis, y2: axis, class: "qp-strip-axis"}));
		const bars = this.barsData;
		const placed = placeBars(bars, 0, width, barW, gap);
		const {unit, gap: between} = blockSize(bars, plotH, rows, blockGap);
		bars.forEach((b, i) => {
			const g = svg("g", {class: "qp-strip-bar"});
			let y = axis - 0.5;
			for (const run of barRuns(b.kinds)) {
				const h = run.count * unit + (run.count - 1) * between;
				y -= h;
				g.append(svg("rect", {x: placed.xs[i], y, width: placed.widths[i], height: h, rx: n("block-radius"),
					class: `qp-kind-${run.kind}`}));
				y -= between;
			}
			root.append(g);
		});
		const parts = partTotals(bars, placed, this.bucketData).map((p) => {
			const total = svg("text", {x: p.x, y: TOTALS - 4, class: "qp-strip-total", "text-anchor": "middle"});
			const name = svg("text", {x: p.x, y: height - 2, class: "qp-strip-part", "text-anchor": "middle"});
			total.textContent = number(p.total);
			name.textContent = partWord(p.part).toLocaleUpperCase(LOCALE);
			root.append(total, name);
			return {...p, total, name};
		});
		const turn = `${String(this.turn ?? 4).padStart(2, "0")}:00`;
		const start = svg("text", {x: 0, y: height - 2, class: "qp-strip-end", "text-anchor": "start"});
		const end = svg("text", {x: width, y: height - 2, class: "qp-strip-end", "text-anchor": "end"});
		start.textContent = turn;
		end.textContent = turn;
		root.append(start, end);
		this.replaceChildren(root);
		// the totals and the part names apart, the names between the ends' times, measured as drawn
		const laid = stripWords(parts.map((p) => ({x: p.x, totalWidth: p.total.getComputedTextLength(),
			nameWidth: p.name.getComputedTextLength()})), start.getComputedTextLength(), width,
		token(this, "--qp-layout-space8"));
		parts.forEach((p, i) => {
			p.total.setAttribute("x", String(laid.totals[i]));
			if (laid.names) p.name.setAttribute("x", String(laid.names[i]));
			else p.name.remove();
		});
		if (!laid.ends[0]) start.remove();
		if (!laid.ends[1]) end.remove();
	}
}

define("qp-day-strip", QpDayStrip, {label: String, turn: Number});
