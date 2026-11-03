// The charts of the Book and the Log, Pro's (docs/ui.md, "Pro on the computer"; the phones' Charts and LogWeeks, one
// block with Android's: `--qp-surface-chart-*`): a ladder's rung as a stair in its rungs' quality tones, a move's
// record as a gold step line with each typed count a dot and each rise starred, and a value per game week as stacked
// columns or a gold line over the weeks' slots. Each is an inline SVG drawn at its box's width (again as the window
// resizes), its value axis in caps at the leading edge, the dated ones over at least the last 90 days and wider than
// the card past them (it scrolls, its newest end in view), the pointer showing the value under it; each carries its
// words for a screen reader and its legend under it, and says "Nothing to show yet." without a point. The frame, the
// axes, the step line, the stars and the pointer are the parts Hero's charts share (lib/charts.js); a muscle's best rung
// on Hero is this stair with its record days starred.

import {t} from "../../lib/copy.js";
import {h, qualityName} from "../../kit/element.js";
import {axisWords, dated, dateLabels, px, responsive, scrub, star, stepPlot, svg, ticks, timeAxis, WEEK_SLOTS}
	from "../../lib/charts.js";
import {LOCALE} from "../../lib/format.js";

// the Book's ladder page and the Log's weeks take the shared card, legend and slots from here with the plots
export {WEEK_SLOTS, chartCard, legend} from "../../lib/charts.js";

/** A ladder's rung over time as a stair: each rung held as a tread in its quality's tone until it changed, a riser and
 * a dot at each change, every rung on the axis with its words in its quality's tone (`levels`: {value, quality, word,
 * label}); `records`, the record days a muscle's best rung stars (the phones' QualityChart), each starred above the
 * plot. */
export function stairChart(points, levels, now, label, records = []) {
	if (!points.length) return null;
	const left = 104;
	return dated(label, left, (width) => {
		const row = px("rung-row");
		const top = Math.max(...levels.map((l) => l.value));
		const room = records.length ? px("star") + 8 : 0;
		const height = top * row;
		const axis = timeAxis(points[0].at, now, width);
		const total = axis.total;
		const root = svg("svg", {width: total, height: room + height + 40, class: "qp-chart-svg"});
		const y = (v) => room + height - (v - 0.5) * row + 4;
		const tone = (v) => {
			const name = qualityName(levels.find((l) => l.value === v)?.quality);
			return name ? `var(--qp-quality-${name})` : "var(--qp-color-text2)";
		};
		for (const level of levels) {
			root.append(svg("line", {x1: 0, x2: total, y1: y(level.value), y2: y(level.value), class: "qp-chart-grid"}));
		}
		const line = [...points, {at: Math.max(now, points.at(-1).at), value: points.at(-1).value}];
		for (let i = 0; i + 1 < line.length; i += 1) {
			const tread = svg("line", {x1: axis.x(line[i].at), x2: axis.x(line[i + 1].at), y1: y(line[i].value),
				y2: y(line[i].value), class: "qp-chart-tread"});
			tread.style.setProperty("stroke", tone(line[i].value));
			root.append(tread);
			if (i + 1 < points.length) {
				root.append(svg("line", {x1: axis.x(points[i + 1].at), x2: axis.x(points[i + 1].at), y1: y(line[i].value),
					y2: y(points[i + 1].value), class: "qp-chart-riser"}));
			}
		}
		for (const p of points) {
			const dot = svg("circle", {cx: axis.x(p.at), cy: y(p.value), r: px("dot") / 2, class: "qp-chart-dot"});
			dot.style.setProperty("fill", tone(p.value));
			root.append(dot);
		}
		for (const at of records) root.append(star(axis.x(at), 2));
		dateLabels(root, axis, now, room + height + 8);
		const frame = h("div", {class: "qp-chart-frame"}, root);
		frame.append(scrub(root, axis, points, (p) => levels.find((l) => l.value === p.value)?.word ?? "", room,
			room + height + 8));
		const words = axisWords(left, room + height + 40, levels.map((l) => ({y: y(l.value),
			text: l.label.toLocaleUpperCase(LOCALE), tone: tone(l.value)})));
		return {axis: words, plot: frame};
	});
}

/** A move's record over time as a gold step line, each typed count a dot in `text`, each rise starred above. */
export function recordChart(record, counts, rises, seconds, now, label, format) {
	return stepPlot({line: record, dots: counts, dotClass: "qp-chart-count", stars: rises, now, label, format,
		words: (p) => (seconds ? t("ladders.record.seconds", {count: p.value}) : t("ladders.record", {count: p.value}))});
}

/** A value a week over Pro's weeks' slots, the newest at the right: each week's `values` stacked bottom up in
 * `colours` (`kind` "stack"), or one gold line with a dot a week ("line"); a week holding a record starred above its
 * column; the newest week's date under the slots and the oldest's where it ends clear of it; a click on a week shows
 * its words (`words(week)`) until the next. `weeks` oldest first: {start, date, label, values, star}. */
export function weekPlot(weeks, kind, colours, words, label) {
	const box = h("div", {class: "qp-chart-box", role: "img", "aria-label": label});
	let chosen = null;
	responsive(box, (width) => {
		const room = weeks.some((w) => w.star) ? px("star") + 8 : 0;
		const height = px("height");
		const highest = Math.max(0, ...weeks.map((w) => (w.values ?? []).reduce((a, b) => a + b, 0)));
		const marks = ticks(0, highest);
		const top = marks.at(-1) || 1;
		const left = 32;
		const plot = width - left;
		const slot = plot / WEEK_SLOTS;
		const offset = WEEK_SLOTS - weeks.length;
		const x = (i) => left + slot * (offset + i + 0.5);
		const y = (v) => room + height * (1 - v / top);
		const root = svg("svg", {width, height: room + height + 22, class: "qp-chart-svg"});
		for (const tick of marks) {
			const w = svg("text", {x: left - 6, y: y(tick) + 4, class: "qp-chart-axis qp-end"});
			w.textContent = String(tick);
			root.append(w, svg("line", {x1: left, x2: width, y1: y(tick), y2: y(tick), class: "qp-chart-grid"}));
		}
		const bar = Math.min(px("bar-width"), slot * 0.6);
		const points = [];
		weeks.forEach((week, i) => {
			if (!week.values) return;
			if (kind === "stack") {
				let base = 0;
				week.values.forEach((value, p) => {
					if (value <= 0) return;
					const foot = y(base) - (base > 0 ? px("stack-gap") : 0);
					const rect = svg("rect", {x: x(i) - bar / 2, y: y(base + value), width: bar,
						height: Math.max(0, foot - y(base + value))});
					rect.style.setProperty("fill", colours[p]);
					root.append(rect);
					base += value;
				});
			} else {
				points.push([x(i), y(week.values[0] ?? 0)]);
			}
			if (week.star) root.append(star(x(i), 2));
		});
		if (points.length > 1) root.append(svg("path", {d: `M${points.map((p) => p.join(",")).join(" L")}`,
			class: "qp-chart-line"}));
		for (const [cx, cy] of points) root.append(svg("circle", {cx, cy, r: px("dot") / 2, class: "qp-chart-dot qp-gold"}));
		const dates = svg("g");
		const newest = weeks.at(-1);
		if (newest) {
			const n = svg("text", {x: width, y: room + height + 16, class: "qp-chart-axis qp-end"});
			n.textContent = newest.date.toLocaleUpperCase(LOCALE);
			dates.append(n);
		}
		const oldest = weeks[0];
		if (weeks.length > 1 && x(0) - slot / 2 + 60 < width - 60) {
			const o = svg("text", {x: left + slot * offset, y: room + height + 16, class: "qp-chart-axis"});
			o.textContent = oldest.date.toLocaleUpperCase(LOCALE);
			dates.append(o);
		}
		root.append(dates);
		const rule = svg("line", {y1: room, y2: room + height, class: "qp-chart-rule", visibility: "hidden"});
		const scrubLabel = h("div", {class: "qp-scrub", hidden: true});
		root.append(rule);
		const show = () => {
			const i = weeks.findIndex((w) => w.start === chosen);
			if (i < 0 || !weeks[i].values) {
				rule.setAttribute("visibility", "hidden");
				scrubLabel.hidden = true;
				return;
			}
			rule.setAttribute("x1", x(i));
			rule.setAttribute("x2", x(i));
			rule.setAttribute("visibility", "visible");
			scrubLabel.replaceChildren(h("span", {class: "qp-type-num-row", text: words(weeks[i])}),
				h("span", {class: "qp-type-meta", text: weeks[i].label}));
			scrubLabel.style.setProperty("--qp-scrub-x", `${x(i)}px`);
			scrubLabel.hidden = false;
		};
		root.addEventListener("click", (e) => {
			const at = Math.floor((e.clientX - root.getBoundingClientRect().left - left) / slot) - offset;
			const week = weeks[at];
			chosen = week?.values && week.start !== chosen ? week.start : null;
			show();
		});
		show();
		box.replaceChildren(h("div", {class: "qp-chart-frame"}, root, scrubLabel));
	});
	return box;
}
