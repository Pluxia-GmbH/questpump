// The Saga's words and shapes (the Saga, 2026-10-15; contract/fixtures/history.json `saga`, questpump-desk/src/saga.rs), shared by
// the Saga's pages (views/log), Pump home's today's chapter (views/home) and the share cards (lib/share.js), so a day
// reads alike on every one: its chapter and date, its word in its tone, a set as the Log writes it, the parts of the
// day, the units drawn small beside a large number, the strip's bars placed on the 24 hours, and what a screen reader
// hears for a day and for its strip. Every key is named by its literal, so the copy's lint finds it.

import {t} from "./copy.js";
import {amount, lifted} from "../views/pump/words.js";
import {LOCALE} from "./format.js";

/** The kinds of moves in the strip's order from its axis (library.json's blocks). */
export const KINDS = ["strength", "cardio", "mobility", "yoga"];

/** The kinds' words, by their place in KINDS. */
export const KIND_WORDS = [t("saga.kind.strength"), t("saga.kind.cardio"), t("saga.kind.mobility"), t("saga.kind.yoga")];

/** The parts of the day by the hour a pump started (rule 9), in the strip's order. */
export const PARTS = ["morning", "afternoon", "evening", "night"];

const PART_WORDS = {
	morning: () => t("saga.time.morning"),
	afternoon: () => t("saga.time.afternoon"),
	evening: () => t("saga.time.evening"),
	night: () => t("saga.time.night"),
};

/** A part of the day's word. */
export function partWord(part) {
	return PART_WORDS[part]();
}

const QUALITY_WORDS = {
	uncommon: () => t("saga.quality.uncommon"),
	rare: () => t("saga.quality.rare"),
	epic: () => t("saga.quality.epic"),
	legendary: () => t("saga.best"),
};

/** A day's word: Best day ever for a best day (legendary), else Epic day, Rare day or Uncommon day; none before the
 * gate or on a Common day (2026-10-29: an ordinary day needs no tag). */
export function dayWord(quality) {
	return QUALITY_WORDS[quality]?.() ?? null;
}

/** A game day's key ("2026-10-14") at noon of that date in this computer's zone. */
export function noon(day) {
	const [y, m, d] = day.split("-").map(Number);
	return new Date(y, m - 1, d, 12);
}

const SHORT = {weekday: "short", day: "numeric", month: "short"};
const LONG = {weekday: "long", day: "numeric", month: "long"};
const DAY_MONTH = {day: "numeric", month: "short"};
const INITIAL = new Intl.DateTimeFormat(LOCALE, {weekday: "narrow"});
const WEEKDAY = new Intl.DateTimeFormat(LOCALE, {weekday: "long"});
const NUMBER = new Intl.NumberFormat(LOCALE);
/** The words are English: a unit takes English's form for its count, whatever the computer's locale (copy.js'). */
const ENGLISH = new Intl.PluralRules("en");

/** Each date's formats, this year's and another's, by its parts. */
const DATES = new Map();

/** A day's date in `parts`, with its year where it falls outside this one (December's chapters in January). */
function dated(day, parts) {
	const date = noon(day);
	if (!DATES.has(parts)) {
		DATES.set(parts, [new Intl.DateTimeFormat(LOCALE, parts),
			new Intl.DateTimeFormat(LOCALE, {...parts, year: "numeric"})]);
	}
	return DATES.get(parts)[date.getFullYear() === new Date().getFullYear() ? 0 : 1].format(date);
}

/** A day's date as the Saga shows it: the platform's short weekday, day and short month ("Tue 14 Oct"), its year
 * outside this one. */
export function shortDate(day) {
	return dated(day, SHORT);
}

/** A day's date in full ("Tuesday 14 October"), its year outside this one: a no-move day's page head, what a screen
 * reader hears. */
export function longDate(day) {
	return dated(day, LONG);
}

/** A day's day and short month ("13 Oct"), its year outside this one. */
export function dayMonth(day) {
	return dated(day, DAY_MONTH);
}

/** A week's first day as "Week of {date}" names it ("13 Oct"), its year outside this one. */
export function weekOf(day) {
	return t("saga.week.of", {date: dayMonth(day)});
}

/** A day's weekday initial (the week strip's). */
export function initial(day) {
	return INITIAL.format(noon(day));
}

/** A number with this computer's digit grouping. */
export function number(n) {
	return NUMBER.format(n);
}

/** A chapter's tag words ("Chapter 12"). */
export function chapterWords(n) {
	return t("saga.chapter", {n});
}

/** What a screen reader hears for a day's hero: "Chapter 12, Tuesday 14 October: 59 moves, Best day ever" (without
 * the word before the gate; "No moves yet today" before today's first move). */
export function heroSpoken(day) {
	if (!day.chapter) return t("saga.none.title");
	const args = {n: number(day.chapter), date: longDate(day.day), count: day.moves};
	const quality = dayWord(day.quality);
	return quality ? t("saga.hero.spoken.quality", {...args, quality}) : t("saga.hero.spoken", args);
}

/** What a screen reader hears for a day's strip: each part with moves and its total. */
export function stripSpoken(buckets) {
	return PARTS.map((part, i) => [part, buckets[i]]).filter(([, n]) => n > 0)
		.map(([part, n]) => t("saga.strip.spoken", {part: partWord(part), count: n})).join(", ");
}

/** What a screen reader hears for a day of the week's bars: "Tuesday: 12 moves". */
export function weekdaySpoken(day, moves) {
	return t("saga.strip.spoken", {part: WEEKDAY.format(noon(day)), count: moves});
}

const BIG = {
	day: [() => t("saga.move"), () => t("saga.moves")],
	today: [() => t("saga.move.today"), () => t("saga.moves.today")],
	week: [() => t("saga.move.week"), () => t("saga.moves.week")],
};

/** The caps label beside a day's big number (`what`: "day" an earlier day's, "today", "week"), singular for 1: "Move",
 * "Moves today", "Move this week". */
export function movesLabel(count, what) {
	const [one, many] = BIG[what];
	return ENGLISH.select(count) === "one" ? one() : many();
}

const UNITS = {
	record: [() => t("saga.unit.record"), () => t("saga.unit.records")],
	unlock: [() => t("saga.unit.unlock"), () => t("saga.unit.unlocks")],
	move: [() => t("saga.unit.move"), () => t("saga.unit.moves")],
};

/** The unit word drawn small beside a count drawn large ("3" "records"; the kit.rep and kit.reps pattern). */
export function unit(count, what) {
	const [one, many] = UNITS[what];
	return ENGLISH.select(count) === "one" ? one() : many();
}

/** A set as the Log writes it: a hold's m:ss, a loaded set's "20 kg × 8" (each hand's with `each`), else its count. */
export function setWords(set, hold, each) {
	return set.load != null ? lifted(set.load, set.value, each) : amount(set.value, hold);
}

/** The strip's 24 hours from the day's turn, in minutes. */
const DAY_MINUTES = 1440;

/** The bars placed on a strip `width` wide from `left` (Tokens.Surface.DayStrip, rule 9), by their places in `bars`,
 * each `bar` wide (a number, or a function of the bar: the Quest Bill's barcode, as wide as its moves) and `gap` apart:
 * in minute order, then the bars' order (the night the clocks go back repeats an hour's minutes), at one pitch for the
 * day, the largest that lets every bar stand at or right of its own start (its minute's place on the 24 hours) and
 * inside the strip, never more than its width and `gap`: for each bar, the room from its start to the strip's end over
 * the bars from it on. Below that the gap closes first, then the bars narrow alike (never under `least`), so a crowded
 * hour reads as a block at its hour; each bar at its start or a pitch right of the one before, whichever is later, and
 * only what still passes the end at `least` and no gap moves back left from it. Returns each bar's left edge (`xs`)
 * and width (`widths`). */
export function placeBars(bars, left, width, bar, gap, least = 1) {
	const order = bars.map((b, i) => i).sort((a, b) => bars[a].minute - bars[b].minute || a - b);
	const wanted = bars.map((b) => (typeof bar === "function" ? bar(b) : bar));
	const end = left + width;
	const starts = bars.map((b, i) => left + (b.minute / DAY_MINUTES) * (width - wanted[i]));
	// each bar's room to the end, and the wanted widths of the bars from it on (in minute order)
	let tail = 0;
	let gapRoom = gap;
	let scale = 1;
	for (let k = order.length - 1; k >= 0; k -= 1) {
		const i = order[k];
		tail += wanted[i];
		const room = end - starts[i];
		const count = order.length - k;
		gapRoom = Math.min(gapRoom, (room - tail) / count);
		scale = Math.min(scale, room / tail);
	}
	// the gap closes first, then every bar narrows alike
	const between = Math.max(0, gapRoom);
	const widths = wanted.map((w) => (gapRoom >= 0 ? w : Math.max(least, w * scale)));
	const xs = new Array(bars.length);
	let next = -Infinity;
	for (const i of order) {
		xs[i] = Math.max(starts[i], next);
		next = xs[i] + widths[i] + between;
	}
	let last = end;
	for (let k = order.length - 1; k >= 0; k -= 1) {
		const i = order[k];
		xs[i] = Math.max(left, Math.min(xs[i], last - widths[i]));
		last = xs[i] - between;
	}
	return {xs, widths};
}

/** Each part with moves and the middle of its bars on the strip (where its total and its name stand), from the bars'
 * places (`placeBars`). */
export function partTotals(bars, placed, buckets) {
	return PARTS.map((part, i) => {
		const own = bars.map((b, j) => j).filter((j) => bars[j].bucket === part);
		if (!buckets[i] || !own.length) return null;
		const from = Math.min(...own.map((j) => placed.xs[j]));
		const to = Math.max(...own.map((j) => placed.xs[j] + placed.widths[j]));
		return {part, total: buckets[i], x: (from + to) / 2};
	}).filter(Boolean);
}

/** The strip's words on its `width` (Tokens.Surface.DayStrip): each part's total over the middle of its bars and its
 * name under them, each row moved apart `gap` (`spread`), the names between the turn's time at both ends (`ends`, its
 * width): where they do not fit, the right end's time goes, then the left's too, then the names (the totals alone).
 * `parts` are partTotals' with each one's `totalWidth` and `nameWidth`; returns the totals' and the names' middles
 * (`names` null where they go) and which ends show ([left, right]). */
export function stripWords(parts, ends, width, gap) {
	const totals = spread(parts.map((p) => ({x: p.x, width: p.totalWidth})), 0, width, gap);
	const names = (from, to) => spread(parts.map((p) => ({x: p.x, width: p.nameWidth})), from, to, gap);
	const fits = (xs, from) => !parts.length || xs[0] - parts[0].nameWidth / 2 >= from - 0.5;
	const inner = ends + gap;
	for (const [shown, from, to] of [[[true, true], inner, width - inner], [[true, false], inner, width],
		[[false, false], 0, width]]) {
		const xs = names(from, to);
		if (fits(xs, from)) return {totals, names: xs, ends: shown};
	}
	return {totals, names: null, ends: [true, true]};
}

/** The middles of words on one line (`items` {x, width}: each one's wanted middle and its width), moved apart so
 * none comes within `gap` of the next and all stay between `left` and `right` (the strip's part names between its
 * ends' times). */
export function spread(items, left, right, gap) {
	const xs = items.map((i) => i.x);
	for (let i = 0; i < items.length; i += 1) {
		const least = i === 0 ? left + items[i].width / 2 : xs[i - 1] + (items[i - 1].width + items[i].width) / 2 + gap;
		xs[i] = Math.max(xs[i], least);
	}
	for (let i = items.length - 1; i >= 0; i -= 1) {
		const last = i === items.length - 1;
		const most = last ? right - items[i].width / 2 : xs[i + 1] - (items[i + 1].width + items[i].width) / 2 - gap;
		xs[i] = Math.min(xs[i], most);
	}
	return xs;
}

/** A bar's blocks from its axis up: one a move, by kind, each run of one kind a block of its own height (rule 9). */
export function barRuns(kinds) {
	return KINDS.map((kind, k) => ({kind, count: kinds[k]})).filter((r) => r.count > 0);
}

/** A block's height and the gap between blocks for every bar of the day (Tokens.Surface.DayStrip): `rows` blocks of
 * `height` fill the plot `plot` high, and past them every block shrinks alike, so the day's tallest pump fills it and
 * the others keep their share of it; the gaps go where they would eat the bars, and a block is never under a sliver. */
export function blockSize(bars, plot, rows, gap) {
	const tallest = Math.max(0, ...bars.map((b) => b.kinds.reduce((a, n) => a + n, 0)));
	if (tallest <= rows) return {unit: (plot - (rows - 1) * gap) / rows, gap};
	const between = plot / tallest < 2 * gap ? 0 : gap;
	return {unit: Math.max(0.5, (plot - (tallest - 1) * between) / tallest), gap: between};
}

/** The streak card's day grid (rule 19; surface.StreakCard): a cell a day of a streak of `days` days in a box `width` ×
 * `height` points, each column count tried, its rows the days over it and its cell the most `cellMost`, the width and
 * the height leave (a gap a `gapFraction` of a cell between cells); the largest cell wins (alike within a millionth of
 * a point), then the fewest rows, then the fewest columns. `check`: the cell is large enough for its check mark
 * (`checkFrom`). Cells fill from the top left, a row left to right, the oldest day first. */
export function placeGrid(days, width, height, gapFraction, cellMost, checkFrom) {
	let best = {cols: 0, rows: 0, cell: 0};
	for (let cols = 1; cols <= days; cols += 1) {
		const rows = Math.ceil(days / cols);
		const cell = Math.min(cellMost, width / (cols + (cols - 1) * gapFraction),
			height / (rows + (rows - 1) * gapFraction));
		const larger = cell > best.cell + 1e-6;
		const alike = Math.abs(cell - best.cell) <= 1e-6;
		if (larger || (alike && (rows < best.rows || (rows === best.rows && cols < best.cols)))) {
			best = {cols, rows, cell};
		}
	}
	return {...best, gap: best.cell * gapFraction, check: best.cell >= checkFrom};
}
