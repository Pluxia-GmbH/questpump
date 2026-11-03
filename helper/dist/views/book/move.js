// A move's page (docs/ui.md, "Hero, the Book and the Log"; the phones' MoveJournal, slice 30), pushed from its window
// in Book or a refused Start's Include in pumps: the DetailBar with the move's name; the hero card with its figure
// (pausable; above the player's reach under "Above your step for now", David, 2026-10-14), its quality with the step's
// pips and its record (Max set for a reachable rep or hold move before the first), and its Start (slice 38: the move
// window's, ns.StartFor of the move, greyed with its reason by the pump's gate, its ladder's climb in the "To get here"
// row above the reach); How to (the setup line first, then How's numbered steps, the gear's own where the player's
// gear has them) and Tips (the move's own advice last), kit/move-how.js's, closed by a pose's goal pose, the stop line
// and the right-versus-wrong pair; the Helps row (slice 42: the pose that helps it, where the setup
// offers it, a row to that pose's page); Trains (the body map with its primary muscles lit and its secondary ones at
// half, the muscles in words, the stat it raises); You (how well the
// player knows it, its last count, its record (a loaded move's with the weight each was done with) with Correct a
// count under it (slice 44: the correction window, views/book/correct.js, on a move with a record or a set the window
// lists, which the Book carries while the page stands, `book.move`; disabled while a pump runs) and Clear this record
// under that, Favourite and Include in pumps). A redraw from `book` keeps the focus on the control it was on, and
// keeps the hero's figure and How to's pair playing (only what the Book moves is drawn again). A Start that goes
// nowhere leaves the page as it was, its Start ready again.

import {intent} from "../../lib/bridge.js";
import {t} from "../../lib/copy.js";
import {keepFocus} from "../../lib/focus.js";
import {LOCALE} from "../../lib/format.js";
import {push, register} from "../../lib/router.js";
import {h} from "../../kit/element.js";
import {howSteps, markLine, stopLine, tipList} from "../../kit/move-how.js";
import {page, watch} from "../page.js";
import {lifted} from "../pump/words.js";
import {loading, qualityLine} from "./book.js";
import {goLink, startBlock, startMove} from "./card.js";
import {openCorrectWindow} from "./correct.js";
import {
	MOVE, aboveReach, amount, confirmWindow, factsOf, familiarity, guideOf, lookup, moveName, moveValue, recordOf,
	recordWords, switches, valueCell,
} from "./common.js";
import {ladderOf} from "./steps.js";

const LIST = new Intl.ListFormat(LOCALE, {type: "conjunction"});

/** A sentence row: its words in body, under a glyph or a step's number. */
function sentence(text, lead) {
	return h("qp-row", {class: "qp-sentence", heading: text, trailing: "shown", ...lead});
}

/** The Helps row (slice 42, a soft need, never a gate): "Helps: {move}" where the move names a pose that helps it and
 * the setup offers that pose (the Book's `hint`), a row to that pose's page; none else. */
function helps(w, fact) {
	const pose = fact?.hint;
	if (pose == null) return [];
	return [h("qp-card", {rows: true}, h("qp-row", {heading: t("ladders.card.hint", {move: moveName(w, pose)}),
		glyph: "action_book", trailing: "nav", "on:click": () => push(MOVE, {move: pose})}))];
}

/** The hero card's parts round its figure (`before` it, `after` it): above the player's reach its line, then the
 * quality line, the record and the Start. */
function heroParts(w, book, pump, move, fact, start) {
	const m = w.moves.get(move);
	const above = aboveReach(w, book, fact, move);
	const quality = fact?.quality ?? (fact?.trained ? 1 : null);
	const value = moveValue(w, move, recordOf(book, factsOf(book), move), above, fact);
	const own = m?.ladder != null ? ladderOf(w, book, m.ladder) : null;
	const begin = book.moves.find((x) => x.move === move)?.start ?? null;
	return {
		before: above ? [h("p", {class: "qp-type-subtitle qp-centred", text: t("ladders.move.locked")})] : [],
		after: [h("div", {class: "qp-standing"}, quality ? qualityLine(w, quality, fact?.step ?? 0) : h("span"),
			value ? valueCell(value) : ""),
		startBlock({w, book, pump, start: (st, opens) => start(opens), go: goLink}, begin, own, move, true)],
	};
}

/** Under "How to" a card of the setup line and How's numbered steps, for the gear that carries a loaded move's load
 * (`guideOf`); under "Tips" a card of the tips with the move's own advice last (moves.json's `hint`: under the Helps
 * row until 2026-10-22); then, closing the Tips card (How's where the move has no tips), a pose's goal pose (a twin's
 * and a path stage's too, 2026-10-14), the stop line and the right-versus-wrong pair, as the phones'
 * MoveGuide adds them. */
function howTo(w, move, fact) {
	const m = w.moves.get(move);
	const {setup, how} = guideOf(m, fact);
	const more = [markLine(m.mark), stopLine(m.stop), h("qp-move-figure-pair", {move: m.key, name: m.name})]
		.filter(Boolean);
	const steps = howSteps(setup, how);
	const tips = tipList(m.tips);
	const card = (parts) => h("qp-card", {class: "qp-how-card"}, ...parts);
	const parts = [];
	if (steps.length) {
		parts.push(h("qp-section-label", {label: t("ladders.journal.how")}),
			card(tips.length ? steps : [...steps, ...more]));
	}
	if (tips.length) parts.push(h("qp-section-label", {label: t("kit.move.tips")}), card([...tips, ...more]));
	else if (!steps.length) parts.push(card(more));
	return parts;
}

function trains(w, book, move, fact) {
	const m = w.moves.get(move);
	// in its quality's tone once trained (Starter for one no ladder places), else in Starter's grey
	const quality = fact?.trained ? fact.quality ?? 1 : 1;
	const map = h("qp-muscle-map", {figure: book.figure ?? "male", label: m.name});
	map.tones = Object.fromEntries(m.mp.map((id) => [id, quality]));
	map.half = Object.fromEntries(m.ms.map((id) => [id, quality]));
	const parts = [map];
	if (m.mp.length) parts.push(h("p", {class: "qp-type-body", text: LIST.format(m.mp.map((id) => w.muscles.get(id)))}));
	if (m.ms.length) {
		parts.push(h("p", {class: "qp-type-meta",
			text: t("ladders.journal.also.list", {muscles: LIST.format(m.ms.map((id) => w.muscles.get(id)))})}));
	}
	const stat = fact?.stat ? w.stats.get(fact.stat) : null;
	if (stat) {
		const counted = m.dose === "rep" || m.dose === "hold" || m.count != null;
		parts.push(h("div", {class: "qp-stat-line"}, h("qp-art", {name: `stat_${fact.stat}`}),
			h("span", {class: "qp-type-body", text: counted ? t("ladders.journal.stat", {stat})
				: t("ladders.journal.stat.learn", {stat})})));
	}
	return h("qp-card", {class: "qp-trains"}, ...parts);
}

/** The confirmation before `move`'s record is cleared on every device and in the game (Rust refuses it while a pump
 * runs): its page's row's, and Correct a count's for a record from an older set. */
function confirmClear(w, move) {
	confirmWindow(t("ladders.journal.clear.title", {move: moveName(w, move)}), t("ladders.journal.clear.body"),
		t("ladders.journal.clear.confirm"), () => intent("book.clear", {move}).catch(console.error));
}

/** Correct a count (slice 44): on a move with a record or a set the correction window lists (its `sets`, which the
 * Book carries while the page stands), opening that window (`correct`); disabled while a pump runs. None else. */
function correctRow(pump, fact, record, correct) {
	if (!record && !fact?.sets?.length) return [];
	return [h("qp-row", {heading: t("ladders.journal.correct"), glyph: "action_log", trailing: "action",
		disabled: Boolean(pump?.running), "data-focus-key": "correct", "on:click": correct})];
}

function you(w, book, pump, move, fact, correct) {
	const hold = w.moves.get(move)?.dose === "hold";
	const each = w.moves.get(move)?.load === "each";
	const rows = [];
	if (fact && fact.familiarity >= 1) {
		rows.push(sentence(familiarity(fact.familiarity, fact.days, w.words.learnedDays), {glyph: "reward_familiarity"}));
	} else if (!fact?.trained) {
		rows.push(sentence(t("character.muscle.untrained"), {glyph: "reward_familiarity"}));
	}
	if (fact?.last !== undefined && fact?.last !== null) {
		rows.push(h("qp-row", {heading: t("ladders.journal.last"), glyph: "action_log", trailing: "shown",
			value: fact.lastLoad != null ? lifted(fact.lastLoad, fact.last, each) : amount(fact.last, hold)}));
	}
	const record = recordWords(w, book, factsOf(book), move);
	if (record) {
		rows.push(h("qp-row", {heading: t("ladders.record.label"), glyph: "reward_record", trailing: "shown",
			value: record}));
	}
	rows.push(...correctRow(pump, fact, record, correct));
	if (record) {
		rows.push(h("qp-row", {heading: t("ladders.journal.clear"), glyph: "mark_close", trailing: "action",
			destructive: true, "data-focus-key": "clear", "on:click": () => confirmClear(w, move)}));
	}
	rows.push(...switches(w, book, factsOf(book), move, null));
	return h("qp-card", {rows: true}, ...rows);
}

register(MOVE, ({move}) => {
	const view = page({heading: ""});
	let w = null;
	let book = null;
	let pump = null;
	let started = false; // its Start ran, until its pump has gone
	// with what a stretch check its Start runs opens (slice 42: the move waiting on it)
	const start = (opens) => {
		started = true;
		startMove({move}, {place: "move", ladder: w.moves.get(move)?.ladder ?? null, step: null,
			...(opens ? {opens} : {})}, () => {
			started = false;
		});
	};
	// what no Book moves, kept while the page stands so their figures play on: the hero's figure in its card, and How
	// and Tips with the pair (made again only for new words or another implement)
	let fixed = null;
	const draw = () => {
		if (!w || !book) return;
		const m = w.moves.get(move);
		view.bar.heading = m?.name ?? "";
		if (!book.ready || !m) {
			view.body.replaceChildren(book.ready ? "" : loading());
			return;
		}
		const fact = factsOf(book).get(move);
		const gear = fact?.implement ?? null;
		if (fixed?.w !== w) {
			const figure = h("qp-figure-stage", {move: m.key, pausable: true, label: m.name});
			fixed = {w, figure, hero: h("qp-card", {class: "qp-book-hero"}, figure)};
		}
		// How for the gear the setup holds now (a setup change picks another implement's steps)
		if (fixed.how == null || fixed.gear !== gear) Object.assign(fixed, {gear, how: howTo(w, move, fact)});
		const {figure, hero, how} = fixed;
		const parts = heroParts(w, book, pump, move, fact, start);
		for (const el of [...hero.children]) if (el !== figure) el.remove();
		figure.before(...parts.before);
		figure.after(...parts.after);
		const rest = [...helps(w, fact), h("qp-section-label", {label: t("ladders.journal.trains")}),
			trains(w, book, move, fact),
			h("qp-section-label", {label: t("ladders.journal.you")}), you(w, book, pump, move, fact, correct)];
		const last = how.at(-1);
		if (view.body.firstElementChild === hero && last.parentNode === view.body) {
			while (last.nextSibling) last.nextSibling.remove();
			view.body.append(...rest);
		} else {
			view.body.replaceChildren(hero, ...how, ...rest);
		}
	};
	// the Book carries this move's sets for Correct a count while the page stands; its window goes with the page
	let correcting = null;
	const correct = () => {
		correcting = openCorrectWindow(w, move, () => confirmClear(w, move));
	};
	intent("book.move", {move, shown: true}).catch(console.error);
	view.stops.push(() => {
		correcting?.close();
		intent("book.move", {move, shown: false}).catch(console.error);
	});
	watch(view, "book.words", (words) => {
		w = lookup(words);
		keepFocus(view.body, draw);
	});
	watch(view, "book", (b) => {
		book = b;
		keepFocus(view.body, draw);
	});
	// a pump publish redraws the page (restarting its figure) only where the Start's gate moved or a pump began or
	// ended (Correct a count waits for its end)
	let gate = null;
	watch(view, "pump", (p) => {
		pump = p;
		// a pump on screen that is no move started from a move's page or window keeps no return
		if (p.pump && !(p.pump.what === "move" && p.pump.from?.place === "move")) started = false;
		const now = JSON.stringify([p.gate, p.locked_until, Boolean(p.pump), p.running]);
		if (now === gate) return;
		gate = now;
		keepFocus(view.body, draw);
	});
	// the pump's cover has gone: a move started here comes back to its page, the focus on its Start
	view.resume = () => {
		const back = started;
		started = false;
		return back ? view.body.querySelector("qp-start-block")?.primary ?? null : null;
	};
	return view;
});
