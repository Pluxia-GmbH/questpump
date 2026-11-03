// The charts' shared parts (docs/ui.md, "Pro on the computer"; the phones' one Charts module, one block with
// Android's: `--qp-surface-chart-*`), which the Book's and the Log's charts (views/book/charts.js) and Hero's
// (views/hero/charts.js) draw with: a chart's card with its title, its legend and its values for a screen reader; a
// dated plot's frame (its value axis's words in a column that stays, the plot beside it at its box's width, again as
// the window resizes, over at least the last 90 days and wider than the card past them, scrolled to its newest end),
// its time axis and dates, a gold step line, a record's star, and the pointer's value.

import {t} from "./copy.js";
import {h} from "../kit/element.js";
import {artSvg} from "../kit/qp-art.js";
import {LOCALE} from "./format.js";

export const DAY = 86_400;
/** The days a dated chart shows in its box; an older first point widens it. */
const WINDOW_DAYS = 90;
/** A date on the time axis every this many days, none this close to the axis's end. */
const LABEL_DAYS = 14;
const LABEL_ROOM_DAYS = 10;
/** The px a dated plot keeps clear at each end of its time axis: half a date's width. */
const EDGE = 14;
/** The weeks Pro's weeks lay their slots out for. */
export const WEEK_SLOTS = 13;

/** A token's number in px (the chart block's measures). */
export function px(name) {
	return parseFloat(getComputedStyle(document.documentElement).getPropertyValue(`--qp-surface-chart-${name}`)) || 0;
}

export function svg(tag, attrs = {}) {
	return h(`svg:${tag}`, attrs);
}

/** Draws `draw(width)` into `box` now and again whenever its width changes. */
export function responsive(box, draw) {
	let shown = -1;
	const redraw = () => {
		const width = Math.round(box.clientWidth || 0);
		if (width && width !== shown) {
			shown = width;
			draw(width);
		}
	};
	new ResizeObserver(redraw).observe(box);
	requestAnimationFrame(redraw);
}

/** Ticks of a value axis from `lo` to `hi`: steps of 1, 2 or 5 times a power of ten, at most four. */
export function ticks(lo, hi) {
	for (let power = 1; ; power *= 10) {
		for (const m of [1, 2, 5]) {
			const step = m * power;
			const from = Math.floor(lo / step) * step;
			const count = Math.max(1, Math.ceil((hi - from) / step));
			if (count <= 4) return Array.from({length: count + 1}, (_, i) => from + i * step);
		}
	}
}

/** A value line's span with a little room, never below zero (the phones' `StepChart.domain`). */
function span(values) {
	const lo = Math.min(...values);
	const hi = Math.max(...values);
	const pad = Math.max(1, (hi - lo) * 0.15);
	return [Math.max(0, lo - pad), hi + pad];
}

const DATE = new Intl.DateTimeFormat(LOCALE, {day: "numeric"});
const MONTH = new Intl.DateTimeFormat(LOCALE, {month: "short"});
const WHEN = new Intl.DateTimeFormat(LOCALE, {day: "numeric", month: "short", year: "numeric"});

/** A dated point's day in words, as a scrub and a screen reader say it. */
function when(at) {
	return WHEN.format(new Date(at * 1000));
}

/** The chart's legend: each series' swatch in its colour (a CSS value) with its words, then a record's star. */
export function legend(items) {
	return h("div", {class: "qp-chart-legend qp-type-meta"}, ...items.map((item) => item.star
		? h("span", {class: "qp-legend-item"}, h("qp-art", {name: "star", class: "qp-chart-star"}),
			t("charts.legend.star"))
		: h("span", {class: "qp-legend-item"}, swatch(item.colour), item.label)));
}

function swatch(colour) {
	const s = h("span", {class: "qp-swatch", "aria-hidden": "true"});
	s.style.setProperty("--qp-swatch", colour);
	return s;
}

/** A chart in its card's grammar: its title, the plot (or "Nothing to show yet." without one), its legend, and its
 * values as a list a screen reader reads (`values`, words a value; the phones' audio graph's values list). */
export function chartCard(title, plot, items = [], values = []) {
	return h("div", {class: "qp-chart"}, h("h3", {class: "qp-type-body-strong", text: title}),
		plot ?? h("p", {class: "qp-type-body qp-dim", text: t("charts.empty")}), plot && items.length ? legend(items) : "",
		plot && values.length ? h("ul", {class: "qp-unseen"}, ...values.map((text) => h("li", {text}))) : "");
}

/** A dated plot's frame: the value axis's words in a column of `left` px that stays, and beside it the plot in a box
 * that scrolls, its newest end in view. `draw(width)` gives {axis, plot}: the axis's SVG and the plot's frame for a
 * box `width` px wide. */
export function dated(label, left, draw) {
	const box = h("div", {class: "qp-chart-box qp-dated", role: "img", "aria-label": label});
	const axis = h("div", {class: "qp-chart-axis-column"});
	const scroll = h("div", {class: "qp-chart-scroll"});
	box.append(axis, scroll);
	responsive(box, (width) => {
		const drawn = draw(Math.max(width - left, 40));
		axis.replaceChildren(drawn.axis);
		scroll.replaceChildren(drawn.plot);
		scroll.scrollLeft = scroll.scrollWidth;
	});
	return box;
}

/** Where a dated plot's time axis starts: its first point, 90 days before now at the latest. A record's star shows on
 * the axis from there (the phones' `TimeAxis.start`). */
export function axisStart(first, now) {
	return Math.min(first ?? now, now - WINDOW_DAYS * DAY);
}

/** The time axis: from the first point (90 days back at the latest) to now, `width` a 90 days' box, inset by EDGE at
 * both ends so a dot, a star or a date at either end shows whole; `total` the plot's width. */
export function timeAxis(first, now, width) {
	const start = axisStart(first, now);
	const days = (now - start) / DAY;
	const inner = Math.max(width - 2 * EDGE, 1) * Math.max(1, days / WINDOW_DAYS);
	return {start, total: inner + 2 * EDGE, x: (at) => EDGE + ((at - start) / (now - start)) * inner};
}

/** A value axis's words, each at its grid line's height, right-aligned to the plot beside it. */
export function axisWords(left, height, labels) {
	const root = svg("svg", {width: left, height, class: "qp-chart-svg"});
	for (const label of labels) {
		const words = svg("text", {x: left - 6, y: label.y + 4, class: "qp-chart-axis qp-end"});
		words.textContent = label.text;
		if (label.tone) words.style.setProperty("fill", label.tone);
		root.append(words);
	}
	return root;
}

/** The dates under a dated plot: every LABEL_DAYS days from the axis's start, the day over the month in caps. */
export function dateLabels(g, axis, now, top) {
	const first = Math.ceil(axis.start / DAY) * DAY;
	for (let at = first; at <= now - LABEL_ROOM_DAYS * DAY; at += LABEL_DAYS * DAY) {
		const x = axis.x(at);
		g.append(svg("line", {x1: x, x2: x, y1: top, y2: top + 4, class: "qp-chart-grid"}));
		const words = svg("text", {x, y: top + 16, class: "qp-chart-axis"});
		const d = new Date(at * 1000);
		words.append(svg("tspan", {x, dy: 0}), svg("tspan", {x, dy: 13}));
		words.children[0].textContent = DATE.format(d);
		words.children[1].textContent = MONTH.format(d).toLocaleUpperCase(LOCALE);
		g.append(words);
	}
}

/** A record's star at `x` above the plot's top. */
export function star(x, y) {
	const side = px("star");
	const art = artSvg("star");
	art.setAttribute("x", x - side / 2);
	art.setAttribute("y", y);
	art.setAttribute("width", side);
	art.setAttribute("height", side);
	art.classList.add("qp-chart-star");
	return art;
}

/** The pointer's value: a rule at the point it rests on and its words above the plot. */
export function scrub(root, axis, points, words, top, bottom) {
	const rule = svg("line", {y1: top, y2: bottom, class: "qp-chart-rule", visibility: "hidden"});
	const label = h("div", {class: "qp-scrub", hidden: true});
	root.append(rule);
	const move = (e) => {
		const box = root.getBoundingClientRect();
		const x = e.clientX - box.left;
		const at = points.filter((p) => axis.x(p.at) <= x).at(-1);
		if (!at) {
			rule.setAttribute("visibility", "hidden");
			label.hidden = true;
			return;
		}
		rule.setAttribute("x1", x);
		rule.setAttribute("x2", x);
		rule.setAttribute("visibility", "visible");
		label.replaceChildren(h("span", {class: "qp-type-num-row", text: words(at)}),
			h("span", {class: "qp-type-meta", text: when(at.at)}));
		label.hidden = false;
		label.style.setProperty("--qp-scrub-x", `${x}px`);
	};
	root.addEventListener("pointermove", move);
	root.addEventListener("pointerleave", () => {
		rule.setAttribute("visibility", "hidden");
		label.hidden = true;
	});
	return label;
}


/** A dated step line (the phones' StepChart and RecordChart): `line` ({at, value}, by time) as a gold step line, each
 * value held until the next and the last to now; `dots` ({at, value}) each a dot of `dotClass`; `stars` (times) each
 * starred above the plot; the value axis's words by `format`, the pointer's along the line by `words(point)`. */
export function stepPlot({line, dots = [], dotClass = "", stars = [], now, label, format, words}) {
	const all = [...line, ...dots];
	if (!all.length) return null;
	const left = 40;
	return dated(label, left, (width) => {
		const room = stars.length ? px("star") + 8 : 0;
		const height = px("height");
		const [lo, hi] = span(all.map((p) => p.value));
		const marks = ticks(lo, hi);
		const top = Math.max(hi, marks.at(-1));
		const bottom = Math.min(lo, marks[0]);
		const axis = timeAxis(Math.min(...all.map((p) => p.at)), now, width);
		const total = axis.total;
		const root = svg("svg", {width: total, height: room + height + 40, class: "qp-chart-svg"});
		const y = (v) => room + height * (1 - (v - bottom) / (top - bottom || 1));
		for (const tick of marks) {
			root.append(svg("line", {x1: 0, x2: total, y1: y(tick), y2: y(tick), class: "qp-chart-grid"}));
		}
		if (line.length) {
			const held = [...line, {at: Math.max(now, line.at(-1).at), value: line.at(-1).value}];
			let d = `M${axis.x(held[0].at)},${y(held[0].value)}`;
			for (let i = 1; i < held.length; i += 1) d += ` H${axis.x(held[i].at)} V${y(held[i].value)}`;
			root.append(svg("path", {d, class: "qp-chart-line"}));
		}
		for (const p of dots) root.append(svg("circle", {cx: axis.x(p.at), cy: y(p.value), r: px("dot") / 2,
			class: dotClass}));
		for (const at of stars) root.append(star(axis.x(at), 2));
		dateLabels(root, axis, now, room + height + 8);
		const frame = h("div", {class: "qp-chart-frame"}, root);
		frame.append(scrub(root, axis, line, words, room, room + height));
		const valueAxis = axisWords(left, room + height + 40, marks.map((tick) => ({y: y(tick), text: format(tick)})));
		return {axis: valueAxis, plot: frame};
	});
}
