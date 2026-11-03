// Pro's charts on Hero's pages (docs/ui.md, "Pro on the computer"; iOS Charts.swift's StepChart, QualityChart, WeekBars
// and StepWeeks, Android's Charts.kt), drawn from the `hero.charts` topic with the parts the Book's charts draw with
// (lib/charts.js): a value the sheets folded as a gold step line with a dot at each change, holding until it changed
// and the last one reaching today; a muscle's best rung as the Book's stair in the qualities' tones; a value per game
// week as gold bars at each week's middle, every week the rows count with its bar, a loss below zero (never in the stop
// red); each with its record days or weeks starred above it and the pointer's value; and a muscle's moves per game week
// as a strip of the body map's steps with the count in each. A screen reader reads each chart's values as a list (the
// phones' audio graph's values list). Only while Pro holds: without it Hero's pages draw no chart and no teaser.

import {t} from "../../lib/copy.js";
import {DAY, axisStart, axisWords, chartCard, dateLabels, dated, px, scrub, star, stepPlot, svg, ticks, timeAxis}
	from "../../lib/charts.js";
import {h} from "../../kit/element.js";
import {LOCALE} from "../../lib/format.js";
import {stairChart} from "../book/charts.js";

/** A game week, and where its bar, its star and a scrub's rule stand: its middle (the phones' `Week.middle`). */
const WEEK = 7 * DAY;
/** The weeks a strip of steps shows: a quarter (the phones' `StepWeeks.shown`). */
const WEEKS_SHOWN = 13;
const DATE = new Intl.DateTimeFormat(LOCALE, {day: "numeric", month: "short"});
const NUMBER = new Intl.NumberFormat(LOCALE);

function date(at) {
	return DATE.format(new Date(at * 1000));
}

/** The record days or weeks on a time axis from `first` to `end` (the phones' `stars`): those a chart stars and its
 * legend and values name. */
function onAxis(records, first, end) {
	return records.filter((at) => at >= axisStart(first, end) && at <= end);
}

/** A chart's values in words, each a day's (or a week's) and its value, then each starred record. */
function values(points, words, stars, weekly) {
	const when = (at) => (weekly ? t("charts.weekof", {date: date(at)}) : date(at));
	return [...points.map(([at, v]) => t("charts.row", {date: when(at), series: words(v)})),
		...stars.map((at) => t("charts.row", {date: when(at), series: t("charts.legend.star")}))];
}

/** A value the sheets folded ([at, value]) as a gold step line to `now`, a dot at each change, its record days starred
 * (`records`: each game day's turn). */
export function stepChart({title, label, points, records, now, format}) {
	const stars = onAxis(records, points[0]?.[0], now);
	const line = points.map(([at, value]) => ({at, value}));
	const plot = stepPlot({line, dots: line, dotClass: "qp-chart-dot qp-gold", stars, now, label,
		format: (v) => NUMBER.format(v), words: (p) => format(p.value)});
	return chartCard(title, plot, stars.length ? [{star: true}] : [], values(points, format, stars, false));
}

/** A muscle's best rung ([at, quality]) as the stair the Book draws a ladder's rung with, each quality a level in its
 * tone with its word and numeral ("Rare 4"), its record days starred; the time before a first quality is no tread. */
export function qualityChart({title, label, points, qualities, records, now}) {
	const levels = qualities.map((word, i) => {
		const named = t("character.quality", {quality: word, number: i + 1});
		return {value: i + 1, quality: i + 1, word: named, label: named};
	});
	const trained = points.filter(([, v]) => v > 0);
	const stars = onAxis(records, trained[0]?.[0], now);
	const plot = stairChart(trained.map(([at, value]) => ({at, value})), levels, now, label, stars);
	const words = (v) => levels[v - 1]?.word ?? t("character.muscle.untrained");
	return chartCard(title, plot, stars.length ? [{star: true}] : [], values(trained, words, stars, false));
}

/** A value per game week ([week start, value], every week the rows count) as bars at each week's middle on the time
 * axis, a loss below zero in the loss tone, its record weeks starred. */
export function weekBars({title, label, points, records, now, format}) {
	const end = Math.max(now, (points.at(-1)?.[0] ?? now) + WEEK);
	const stars = onAxis(records, points[0]?.[0], end);
	const plot = points.length ? dated(label, 40, (width) => {
		const room = stars.length ? px("star") + 8 : 0;
		const height = px("height");
		const all = points.map(([, v]) => v);
		const marks = ticks(Math.min(0, ...all), Math.max(0, ...all));
		const top = Math.max(0, ...all, marks.at(-1));
		const bottom = Math.min(0, ...all, marks[0]);
		const axis = timeAxis(points[0][0], end, width);
		const root = svg("svg", {width: axis.total, height: room + height + 40, class: "qp-chart-svg"});
		const y = (v) => room + height * (1 - (v - bottom) / (top - bottom || 1));
		for (const tick of marks) {
			root.append(svg("line", {x1: 0, x2: axis.total, y1: y(tick), y2: y(tick), class: "qp-chart-grid"}));
		}
		const bar = px("bar-width");
		for (const [at, v] of points) {
			const [from, to] = v >= 0 ? [y(v), y(0)] : [y(0), y(v)];
			root.append(svg("rect", {x: axis.x(at + WEEK / 2) - bar / 2, y: from, width: bar, height: Math.max(1, to - from),
				class: v < 0 ? "qp-chart-bar qp-loss" : "qp-chart-bar"}));
		}
		for (const at of stars) root.append(star(axis.x(at + WEEK / 2), 2));
		dateLabels(root, axis, end, room + height + 8);
		const frame = h("div", {class: "qp-chart-frame"}, root);
		frame.append(scrub(root, axis, points.map(([at, value]) => ({at, value})), (p) => format(p.value), room,
			room + height));
		const valueAxis = axisWords(40, room + height + 40, marks.map((tick) => ({y: y(tick), text: NUMBER.format(tick)})));
		return {axis: valueAxis, plot: frame};
	}) : null;
	return chartCard(title, plot, stars.length ? [{star: true}] : [], values(points, format, stars, true));
}

/** A step of moves as the weeks' squares and the body map shade them (`HistorySeries.step`): 0 none, 1 under the first
 * map step, 2 under the second, 3 from it. */
function moveStep(moves, mapSteps) {
	if (moves <= 0) return 0;
	if (moves < (mapSteps[0] ?? Infinity)) return 1;
	return moves < (mapSteps[1] ?? Infinity) ? 2 : 3;
}

/** A muscle's moves per game week as a strip of squares, the latest at the end, each with its count and its step's
 * shade, the steps' words under it. */
export function stepWeeks({title, name, points, mapSteps}) {
	const shown = points.slice(-WEEKS_SHOWN);
	if (!shown.length) return chartCard(title, null);
	const strip = h("div", {class: "qp-chart-weeks", role: "list", "aria-label": name},
		...shown.map(([at, v]) => h("span", {class: `qp-week-step qp-step-${moveStep(v, mapSteps)}`, role: "listitem",
			"aria-label": t("charts.row", {date: t("charts.weekof", {date: date(at)}), series: NUMBER.format(v)}),
			text: NUMBER.format(v)})));
	const [low, high] = mapSteps;
	const legend = [t("charts.steps.none"), t("charts.steps.range", {low: 1, high: low - 1}),
		t("charts.steps.range", {low, high: high - 1}), t("charts.steps.from", {low: high})];
	return h("section", {class: "qp-chart"}, h("h3", {class: "qp-type-body-strong", text: title}), strip,
		h("div", {class: "qp-chart-dates qp-type-caps", "aria-hidden": "true"}, h("span", {text: date(shown[0][0])}),
			h("span", {text: date(shown.at(-1)[0])})),
		h("p", {class: "qp-chart-steps qp-type-meta"}, ...legend.map((word, i) =>
			h("span", {class: `qp-week-step qp-step-${i}`, text: word}))));
}
