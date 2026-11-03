// The Saga on the computer (the Log's place under its players' name since 2026-10-15; docs/ui.md, "The Saga"; board
// A-Day; the phones' LogTab), from `log` (desk/log.rs, desk/saga.rs) and `book.words` (the moves' words, the qualities,
// the muscles, the felt-unwell word) with the body's figure from `book`: the root's TopBar with the compact gold Share
// today before the gear (the Share window on today's Saga card; none while today has no move), today's chapter
// (views/log/chapter.js: the hero, its tiles and its pumps, each pump opening its page), with Pro "Your weeks" (each
// game week's days in the moves' steps, newest first, with the steps' legend; the moves per week as stacked columns by
// where they were done; none before a game day with a pump), then Earlier chapters, newest first: a row an earlier day
// with a move on one line (its chapter's tag, its date, its word with its gem or Best day ever with the star, and its
// moves at the row's end, the columns lined up down the list; stacked on a narrow page) and a quiet row a game day with
// pumps and no move done (its date and No moves), each opening its page (desk.log.day), Show more chapters at the foot
// while older ones wait. Before the first pump the EmptyState with Start a pump; before today's first move the hero's No
// moves yet today and Start a pump, the phones' one primary, which follows the pump's gate as Hero's does; before the
// first build the busy pips. On the first open each game day on this computer the chapter plays its reveal once (the
// strip grows, the number counts up, the muscles light; under Reduce Motion none of it) with the reveal sound, once the
// page is seen (the window in front and shown, no cover nor a window over it, and no best-day moment showing or due: it
// plays as that closes), then says so (`log.revealed`). Scrolled away from its top the page holds the book it shows
// (views/log/held.js): a newer one waits behind the "N new moves" pill and applies at the top, and Show more's answer
// applies where the reader is. Every redraw keeps the keyboard focus on the control it was on, the bar's included, or
// where that control went (Show more's last press, a pump row gone) on the one now in its place; where Start a pump
// gave way to the hero, on Share today, and where Share today went (a new game day's first book), on Start a pump, else
// the page's heading. No Apple Health or Health Connect minutes: those stay on the phones.

import {intent} from "../../lib/bridge.js";
import {t} from "../../lib/copy.js";
import {focusView, keepFocus, topWindow} from "../../lib/focus.js";
import {clock, LOCALE} from "../../lib/format.js";
import {covered, push, register} from "../../lib/router.js";
import {chapterWords, dayMonth, dayWord, heroSpoken, longDate, noon, number, shortDate, unit} from "../../lib/saga.js";
import {feedback} from "../../lib/sound.js";
import {openShare} from "../../lib/share.js";
import {h, qualityNumber} from "../../kit/element.js";
import {page, watch} from "../page.js";
import {loading} from "../book/book.js";
import {WEEK_SLOTS, legend, weekPlot} from "../book/charts.js";
import {startPump} from "../hero/hero.js";
import {momentDue} from "./best.js";
import {chapter, context} from "./chapter.js";
import {holdBook} from "./held.js";

export const PUMP = "desk.log.pump";
export const DAY = "desk.log.day";

/** How long the reveal waits before it looks again while a pump's cover or a window is over the Saga. */
const REVEAL_WAIT_MS = 500;

const LIST = new Intl.ListFormat(LOCALE, {type: "conjunction", style: "narrow"});
const TIME = new Intl.DateTimeFormat(LOCALE, {timeStyle: "short"});
const WEEKDAY = new Intl.DateTimeFormat(LOCALE, {weekday: "narrow"});

/** A game day's name over a pump's page: today's and yesterday's in words, any other its weekday and date, with its
 * year outside this one (`key` "2026-10-06", `ago` the game days before today's). */
export function dayName(key, ago) {
	if (ago === 0) return t("log.today");
	if (ago === 1) return t("log.yesterday");
	const date = noon(key);
	const thisYear = date.getFullYear() === new Date().getFullYear();
	return new Intl.DateTimeFormat(LOCALE, {weekday: "long", day: "numeric", month: "long",
		year: thisYear ? undefined : "numeric"}).format(date);
}

/** How many moves, in a pump's title. */
function moves(n) {
	return n === 0 ? t("pump.moves.none") : t("pump.moves.count", {count: n});
}

/** A pump's title: a level-up's, a catch-up's, a game pump's with its quest's size (with no move done, none of its
 * moves), an app pump's, a move started from this computer's Book as asked ("A pump you asked for · 1 move", as
 * both phones title theirs), and an expedition's round by its stage ("Expedition · round 2", slice 51). */
export function title(e) {
	const n = e.count;
	const counted = e.noMove && n > 0 ? t("log.moves.none.of", {count: n}) : moves(n);
	switch (e.kind) {
		case "levelUp": return t("log.pump.levelup", {moves: counted});
		case "catchUp": return t("pump.title.catchup", {moves: moves(n)});
		case "app": return t("log.pump.app", {moves: counted});
		case "move": return t("pump.title.free", {kind: t("pump.move"), moves: counted});
		case "expedition": return expeditionTitle(e);
		default:
			if (e.tier === 1) return t("log.pump.game.small", {moves: counted});
			if (e.tier === 2) return t("log.pump.game.normal", {moves: counted});
			if (e.tier >= 3) return t("log.pump.game.big", {moves: counted});
			return t("pump.title.game", {moves: counted});
	}
}

/** An expedition round's title by its stage: the warm-up, a round by its number, the cool-down. */
function expeditionTitle(e) {
	if (e.stage === "warmup") return t("expedition.saga.warmup");
	if (e.stage === "cooldown") return t("expedition.saga.cooldown");
	return t("expedition.saga.round", {round: e.round ?? 1});
}

const SOURCE = {
	game: () => t("log.source.game"),
	catchUp: () => t("log.source.catchup"),
	phone: () => t("log.source.phone"),
	watch: () => t("log.source.watch"),
	computer: () => t("log.source.computer"),
};

/** Where a pump came from, in the words' order: its source, then the watch or a computer where a catch-up ran there. */
export function source(e) {
	const words = [];
	if (e.source !== "watch") words.push(SOURCE[e.source]());
	if (e.source === "watch" || e.watch) words.push(SOURCE.watch());
	if (e.computer) words.push(SOURCE.computer());
	return words;
}

/** How long a pump took: seconds under a minute, else whole minutes. */
export function length(seconds) {
	return seconds < 60 ? t("common.seconds.short", {count: seconds})
		: t("log.minutes.short", {count: Math.round(seconds / 60)});
}

/** How a game pump with no move done ended. */
export function undone(e) {
	if (!e.noMove) return null;
	return e.noMove === "banked" ? t("log.banked") : t("log.nomove");
}

/** A pump's line: when, where it came from, how it ended without a move, how long, its reps and holds, a record, and
 * the game's felt-unwell word (`unwell`) on one stopped for feeling unwell. */
function meta(e, unwell) {
	const parts = [TIME.format(new Date(e.at * 1000)), ...source(e)];
	const end = undone(e);
	if (end) parts.push(end);
	if (e.seconds > 0) parts.push(length(e.seconds));
	const counted = e.moves.filter((m) => m.reps !== undefined && m.reps !== null);
	if (counted.length) parts.push(t("log.reps", {count: counted.reduce((a, m) => a + m.reps, 0)}));
	const held = e.moves.filter((m) => m.hold).reduce((a, m) => a + m.seconds, 0);
	if (held > 0) parts.push(t("log.held", {time: clock(held)}));
	if (e.moves.some((m) => m.record)) parts.push(t("log.record"));
	if (e.unwell) parts.push(unwell);
	return parts.join(" · ");
}

/** The weekdays' initials from a week's first day. */
function initials(week) {
	return week.days.map((d) => WEEKDAY.format(noon(d.day)));
}

/** A week's words: its first date (its year outside this one), its label, its days with a move, moves and records,
 * its moves by where they were done. */
function weekWords(week) {
	const date = dayMonth(week.days[0].day);
	const active = week.days.filter((d) => d.step > 0).length;
	const facts = [t("log.weeks.days.count", {count: active}), t("common.moves.count", {count: week.moves})];
	if (week.records > 0) facts.push(t("log.calendar.records", {count: week.records}));
	return {date, label: t("charts.weekof", {date}), facts: LIST.format(facts),
		split: t("log.weeks.split", {game: week.game, catchUp: week.catchUp, app: week.app})};
}

/** The moves' steps in words ("None", "1–3", "4–9", "10+" for map steps [4, 10]). */
function stepWords(steps) {
	const [first = 1, second = first] = steps;
	const range = (lo, hi) => (hi > lo ? t("charts.steps.range", {low: lo, high: hi}) : String(lo));
	return [t("charts.steps.none"), range(1, first - 1), range(first, second - 1), t("charts.steps.from", {low: second})];
}

/** Pro's weeks (at least one): the days a week as marks, newest first, with the steps' legend; the moves per week as
 * columns, stacked by where they were done. */
function proWeeks(log, mapSteps) {
	const head = h("div", {class: "qp-week-row qp-week-initials", "aria-hidden": "true"}, h("span"),
		h("div", {class: "qp-initials"}, ...initials(log.weeks[0]).map((i) => h("span", {class: "qp-type-tab", text: i}))));
	const rows = log.weeks.map((week) => {
		const words = weekWords(week);
		const dots = h("qp-week-dots", {steps: true, label: `${words.label}: ${words.facts}`});
		dots.days = week.days;
		return h("div", {class: "qp-week-row"}, h("span", {class: "qp-type-caps qp-chart-date", text: words.date,
			"aria-hidden": "true"}), dots);
	});
	const steps = stepWords(mapSteps);
	const stepLegend = h("div", {class: "qp-chart-legend qp-type-meta", "aria-hidden": "true"},
		h("span", {text: t("log.calendar.legend")}),
		...steps.map((words, step) => h("span", {class: "qp-legend-item"}, h("span", {class: "qp-dot qp-legend-dot",
			"data-step": String(step)}), words)),
		h("span", {class: "qp-legend-item"}, h("qp-art", {name: "star", class: "qp-chart-star"}), t("log.calendar.record")));
	const shown = [...log.weeks].reverse().slice(-WEEK_SLOTS).map((week) => {
		const words = weekWords(week);
		return {start: week.start, date: words.date, label: words.label, values: [week.game, week.catchUp, week.app],
			star: week.records > 0, split: words.split};
	});
	const colours = ["var(--qp-surface-chart-game)", "var(--qp-surface-chart-catch-up)", "var(--qp-surface-chart-app)"];
	const kinds = [t("log.weeks.game"), t("log.weeks.catchup"), t("log.weeks.app")];
	const columns = weekPlot(shown, "stack", colours, (w) => w.split, t("log.weeks.moves"));
	return h("qp-card", {class: "qp-pro-weeks"},
		h("div", {class: "qp-chart"}, h("h3", {class: "qp-type-body-strong", text: t("log.weeks.days")}), head, ...rows,
			stepLegend),
		h("div", {class: "qp-chart"}, h("h3", {class: "qp-type-body-strong", text: t("log.weeks.moves")}), columns,
			legend([...kinds.map((label, i) => ({label, colour: colours[i]})),
				...(shown.some((w) => w.star) ? [{star: true}] : [])])));
}

/** A pump's row in its day's card, opening its page (`open`): its XP shown "+75" and read "75 XP". */
export function pumpRow(e, place, unwell, open) {
	const row = h("qp-row", {heading: title(e), meta: meta(e, unwell), glyph: glyphOf(e),
		trailing: e.xp ? "value" : "nav", value: e.xp ? t("log.xp", {xp: e.xp}) : null,
		"value-spoken": e.xp ? t("kit.pill.xp", {amount: e.xp}) : null, "data-focus-key": `pump:${e.id}`,
		"on:click": open});
	return h("qp-card-row", {place}, row);
}

/** A pump's kind tile: a pump of the game's or of an app is a pump; a level-up's the XP; a catch-up's the arrow. */
function glyphOf(e) {
	if (e.kind === "levelUp") return "reward_xp";
	if (e.kind === "catchUp") return "mark_arrow";
	return "action_pump";
}

/** A day's word as the kit's QualityWord, its gem and its word in its tone (Rare day), or Best day ever in gold after
 * the star; none before the gate. */
function dayWordMark(quality) {
	const word = dayWord(quality);
	if (!word) return "";
	if (quality !== "legendary") {
		return h("qp-quality-word", {class: "qp-chapter-word", quality: qualityNumber(quality), word});
	}
	return h("span", {class: "qp-chapter-word qp-chapter-best"}, h("qp-art", {name: "star"}), word);
}

/** An earlier day's row under Earlier chapters, opening its page: a chapter's tag, date, word and moves on one line
 * (the columns lined up down the list); a game day with pumps and no move done its date and No moves. */
function chapterRow(c, place) {
	const open = () => push(DAY, {day: c.day, chapter: c.chapter ?? null});
	const date = h("span", {class: "qp-type-caps qp-chapter-date", text: shortDate(c.day)});
	if (!c.chapter) {
		return h("qp-card-row", {place}, h("button", {type: "button", class: "qp-chapter-row qp-chapter-quiet",
			"data-focus-key": `chapter:${c.day}`, "aria-label": [longDate(c.day), t("saga.nomove")].join(", "),
			"on:click": open}, h("span", {class: "qp-chapter-tag", "aria-hidden": "true"}), date,
		h("span", {class: "qp-chapter-word", text: t("saga.nomove")})));
	}
	return h("qp-card-row", {place}, h("button", {type: "button", class: "qp-chapter-row",
		"data-focus-key": `chapter:${c.day}`, "aria-label": heroSpoken(c), "on:click": open},
	h("span", {class: "qp-chapter-tag"}, h("qp-slant-tag", {label: chapterWords(c.chapter), tone: "dim"})), date,
	dayWordMark(c.quality), h("i", {class: "qp-chapter-rule"}),
	h("span", {class: "qp-chapter-count qp-saga-counted"}, h("b", {class: "qp-chapter-moves", text: number(c.moves)}),
		h("span", {class: "qp-type-meta", text: unit(c.moves, "move")}))));
}

/** What the root draws of a book: none of the reveal's flag (so the motion it played stays), an earlier chapter's
 * page or the best-day moment (so the row that opened the page is there for the focus to come back to). */
function drawnOf(log) {
	return JSON.stringify({...log, reveal: false, open: null, best: null});
}

/** How many moves done `newer` holds that `shown` does not: each game day's moves over the ones shown for that day
 * (today's for the day that was today; none for a day the shown book did not list), leaving out a day older than the
 * shown chapters while more waited under them. A pump with no move done adds none. */
function newMoves(shown, newer) {
	const listed = new Map(shown.chapters.map((c) => [c.day, c.moves]));
	listed.set(shown.today.day, shown.today.moves);
	const oldest = shown.chapters.at(-1)?.day;
	let count = 0;
	for (const d of [newer.today, ...newer.chapters]) {
		if (d !== newer.today && shown.more && oldest && d.day < oldest) continue;
		count += Math.max(0, d.moves - (listed.get(d.day) ?? 0));
	}
	return count;
}

/** Before today's first move: No moves yet today, and Start a pump. */
function noMoves(day, begin) {
	return [h("div", {class: "qp-saga-hero qp-saga-none"},
		h("div", {class: "qp-saga-head"}, day.chapter ? h("qp-slant-tag", {label: chapterWords(day.chapter)}) : "",
			h("span", {class: "qp-type-caps qp-saga-date", text: shortDate(day.day)})),
		h("h2", {class: "qp-type-heading", text: t("saga.none.title")})), begin.el];
}

register("log", () => {
	const view = page({heading: t("bar.log"), root: true});
	view.el.classList.add("qp-saga-page");
	view.body.classList.add("qp-saga");
	let mapSteps = [4, 10];
	let wordsTopic = null;
	let book = null;
	let shown = "";
	let reveal = null;
	let moreAsked = false;
	const begin = startPump(view);
	begin.start.setAttribute("data-focus-key", "start");
	const share = (kind) => {
		const log = hold.book;
		if (log) openShare({kind, day: log.today, ctx: context(wordsTopic, book, log.plan)});
	};
	const shareToday = h("qp-primary-button", {label: t("saga.share.today"), glyph: "mark_share", compact: true,
		"data-focus-key": "share:today", "on:click": () => share("day")});
	// where the control the focus was on went: Start a pump giving way to the hero, or the bar's Share today going, gives
	// it to the page's start (Share today, Start a pump, else the heading), any other to the control now in its place
	const nearest = (key, near) => (key === "start" || key === "share:today" ? focusView(view) : near());
	// over the whole page, since the redraw sets the bar's Share today too
	const draw = () => keepFocus(view.el, () => {
		const log = hold.book;
		if (!log || !wordsTopic) {
			view.body.replaceChildren(loading());
			shown = "";
			return;
		}
		// what the root does not show redraws nothing (`drawnOf`)
		const drawn = JSON.stringify([drawnOf(log), book?.figure]);
		if (drawn === shown) return;
		shown = drawn;
		const today = log.today;
		view.bar.action = today.moves > 0 ? shareToday : null;
		const ctx = {...context(wordsTopic, book, log.plan), share};
		const parts = [];
		reveal = null;
		if (!log.ever) {
			parts.push(h("qp-empty-state", {glyph: "action_log", heading: t("log.empty.title")}), begin.el);
		} else if (today.moves === 0) {
			parts.push(...noMoves(today, begin));
		} else {
			const c = chapter(today, ctx, true);
			parts.push(...c.parts);
			reveal = c.reveal;
		}
		// Pro's weeks once a game day has a pump: before it no block stands for them
		if (log.pro && log.weeks.length) {
			parts.push(h("qp-section-label", {label: t("log.weeks.title")}), proWeeks(log, mapSteps));
		}
		if (log.chapters.length) {
			const n = log.chapters.length;
			parts.push(h("qp-section-label", {label: t("saga.chapters")}), h("div", {class: "qp-log-day qp-chapters"},
				...log.chapters.map((c, i) => chapterRow(c, n === 1 ? "alone" : i === 0 ? "first"
					: i === n - 1 ? "last" : "middle"))));
		}
		if (log.more) {
			parts.push(h("qp-quiet-button", {label: t("saga.chapters.more"), "data-focus-key": "more",
				"on:click": () => {
					moreAsked = true;
					intent("log.more").catch(console.error);
				}}));
		}
		view.body.replaceChildren(...parts);
		play();
	}, nearest);
	const hold = holdBook(view, {same: (a, b) => drawnOf(a) === drawnOf(b), fresh: newMoves, draw});
	// the first open each game day: once the Saga shows with today's chapter on it in the window in front, no cover nor
	// a window over it and no best-day moment showing or due (the moment opens first, the reveal as it closes); the
	// window coming forward looks again
	let visible = false;
	let later = 0;
	const play = () => {
		const seen = visible && document.hasFocus() && document.visibilityState === "visible";
		if (!seen || !reveal || !hold.book?.reveal) return;
		if (covered() || topWindow() || momentDue()) {
			clearTimeout(later);
			later = setTimeout(play, REVEAL_WAIT_MS);
			return;
		}
		const run = reveal;
		reveal = null;
		run();
		feedback("reveal");
		intent("log.revealed").catch(console.error);
	};
	new IntersectionObserver((seen) => {
		visible = seen.some((s) => s.isIntersecting);
		play();
	}).observe(view.body);
	// the window coming forward, or showing after its focus came (best.js)
	window.addEventListener("focus", play);
	document.addEventListener("visibilitychange", play);
	view.stops.push(() => {
		window.removeEventListener("focus", play);
		document.removeEventListener("visibilitychange", play);
	});
	watch(view, "book.words", (w) => {
		wordsTopic = w;
		mapSteps = w.mapSteps;
		shown = "";
		draw();
	});
	watch(view, "book", (b) => {
		book = b;
		draw();
	});
	watch(view, "log", (l) => {
		// Show more's answer: the chapters it asked for, or the last of them
		const asked = moreAsked && (!l.more || l.chapters.length > (hold.book?.chapters.length ?? 0));
		if (asked) moreAsked = false;
		hold.take(l, asked);
	});
	// the page's one primary: Share today once today has a move, else Start a pump
	view.primary = () => (shareToday.isConnected ? shareToday : begin.el.isConnected ? begin.start
		: view.body.querySelector(".qp-chapter-row"));
	return view;
});
