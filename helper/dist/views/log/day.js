// An earlier day's page (the Saga, 2026-10-15; docs/ui.md, "The Saga"; the phones' LogDay), pushed from its row under
// the Saga's Earlier chapters: a chapter's under the DetailBar Chapter N with the compact gold Share day at its right
// (the Share window on that day's card), then that day's hero, tiles and pumps as the Saga draws today's
// (views/log/chapter.js), its word as it stood at its end, the Week tile its game week with the day ringed; a game day
// with pumps and no move done under the DetailBar its date, No moves and its pumps, each "banked" or "no move done" as
// the Log says it (no hero, no tiles, no Share). Its heading shows as it opens (the row says its chapter), so the focus
// lands on it and a screen reader says it. It asks Rust for the day as it opens (`log.open {day}`) and lets it go as it
// closes; the pips show until the `log` topic's `open` is that day. Scrolled away from its top the page holds the day
// it shows (views/log/held.js): a newer one (a late row, another device's pump of that day) waits behind the "N new
// moves" pill and applies at the top. Every redraw keeps the keyboard focus on the control it was on, the bar's
// included, or on the one now in its place; where Share day went (the day no chapter any more, or the book opening
// another), on the page's heading.

import {intent} from "../../lib/bridge.js";
import {t} from "../../lib/copy.js";
import {focusView, keepFocus} from "../../lib/focus.js";
import {register} from "../../lib/router.js";
import {chapterWords, longDate} from "../../lib/saga.js";
import {openShare} from "../../lib/share.js";
import {h} from "../../kit/element.js";
import {page, watch} from "../page.js";
import {loading} from "../book/book.js";
import {chapter, context, pumps} from "./chapter.js";
import {holdBook} from "./held.js";
import {DAY} from "./log.js";

register(DAY, ({day, chapter: n}) => {
	const view = page({heading: n ? chapterWords(n) : longDate(day)});
	view.body.classList.add("qp-saga");
	let wordsTopic = null;
	let book = null;
	let shown = "";
	intent("log.open", {day}).catch(console.error);
	view.stops.push(() => intent("log.open", {day: null}).catch(console.error));
	/** The page's day in a book, none while the book opens another. */
	const dayOf = (log) => (log?.open?.day === day ? log.open : null);
	/** What the page draws of a book: the day and the focus's plan, never the rest of the topic. */
	const drawnOf = (log) => JSON.stringify([dayOf(log), log.plan]);
	/** How many moves done the day holds in `newer` over `was` (the book shown); a pump with no move done adds none. */
	const newMoves = (was, newer) => Math.max(0, (dayOf(newer)?.moves ?? 0) - (dayOf(was)?.moves ?? 0));
	const open = () => dayOf(hold.book);
	const share = (kind) => {
		if (open()) openShare({kind, day: open(), ctx: context(wordsTopic, book, hold.book.plan)});
	};
	const shareDay = h("qp-primary-button", {label: t("saga.share.day"), glyph: "mark_share", compact: true,
		"data-focus-key": "share:day", "on:click": () => share("day")});
	// over the whole page, since the redraw sets the bar's Share day too; Share day gone gives the focus to the page's
	// start (its heading), any other control to the one now in its place
	const draw = () => keepFocus(view.el, () => {
		const d = open();
		if (!d || !wordsTopic) {
			view.bar.action = null;
			view.body.replaceChildren(loading());
			shown = "";
			return;
		}
		const drawn = JSON.stringify([drawnOf(hold.book), book?.figure]);
		if (drawn === shown) return;
		shown = drawn;
		const ctx = {...context(wordsTopic, book, hold.book.plan), share};
		if (!d.chapter) {
			view.bar.heading = longDate(d.day);
			view.bar.action = null;
			view.body.replaceChildren(h("h2", {class: "qp-type-heading", text: t("saga.nomove")}), ...pumps(d, ctx));
			return;
		}
		view.bar.heading = chapterWords(d.chapter);
		view.bar.action = shareDay;
		view.body.replaceChildren(...chapter(d, ctx, false).parts);
	}, (key, near) => (key === "share:day" ? focusView(view) : near()));
	const hold = holdBook(view, {same: (a, b) => drawnOf(a) === drawnOf(b), fresh: newMoves, draw});
	watch(view, "book.words", (w) => {
		wordsTopic = w;
		shown = "";
		draw();
	});
	watch(view, "book", (b) => {
		book = b;
		draw();
	});
	watch(view, "log", (l) => hold.take(l));
	view.primary = () => (shareDay.isConnected ? shareDay : null);
	return view;
});
