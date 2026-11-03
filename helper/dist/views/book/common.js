// What the Book's views share (docs/ui.md, "Hero, the Book and the Log"; the phones' BookTab, LadderDetail and
// MoveJournal): the words of `book.words` looked up by id, a move's facts from `book`, the record and goal in the
// phones' one grammar, a ladder's row and the rule that opens a family, a move's switches (Favourite and Include in
// pumps, which move only when `book` says so) and their following while a window stands, the confirmation before a
// record is cleared or a ladder starts over, and the Moves search's fold. Rust sends the facts; these only put them in
// words. A move's window and a step's card are card.js's.

import {intent, subscribe} from "../../lib/bridge.js";
import {t} from "../../lib/copy.js";
import {clock, LOCALE} from "../../lib/format.js";
import {openMenu} from "../../lib/router.js";
import {h} from "../../kit/element.js";
import {lifted} from "../pump/words.js";

/** The ids of the Book's pages and of Your setup (DESK-MENU's Menu page). */
export const LADDER = "desk.book.ladder";
export const MOVE = "desk.book.move";
export const SETUP = "desk.menu.setup"; // copy-lint: not shown (Your setup's view id, DESK-MENU's Menu page)

/** `book.words` looked up by id. */
export function lookup(words) {
	return {
		words,
		moves: new Map(words.moves.map((m) => [m.move, m])),
		ladders: new Map(words.ladders.map((l) => [l.id, l])),
		muscles: new Map(words.muscles.map((m) => [m.id, m.name])),
		stats: new Map(words.stats.map((s) => [s.key, s.name])),
	};
}

/** `book`'s moves' facts by id. */
export function factsOf(book) {
	return new Map((book?.moves ?? []).map((m) => [m.move, m]));
}

/** A move's setup line and How's steps for the gear that carries its load in the player's setup (its fact's
 * `implement`, ns.Implement), else its own (Rust's `Facts::setup` and `Facts::how`), as the phones' Book picks them. */
export function guideOf(m, fact) {
	const gear = fact?.implement;
	return {setup: (gear && m?.setups?.[gear]) ?? m?.setup ?? null, how: (gear && m?.hows?.[gear]) ?? m?.how ?? []};
}

/** A quality's word (1 Starter … 7 Artifact). */
export function qualityWord(w, quality) {
	return w.words.qualities[quality - 1] ?? "";
}

/** A move's name, its key's when the words lack it. */
export function moveName(w, move) {
	return w.moves.get(move)?.name ?? "";
}

/** A tile of `move` with its key pose. */
export function tile(w, move, attrs) {
	return h("qp-move-tile", {...attrs, move: w.moves.get(move)?.key});
}

const NUMBER = new Intl.NumberFormat(LOCALE);

/** A count in the player's digits. */
export function number(n) {
	return NUMBER.format(n);
}

/** Days counted toward `need` as the day lines take them (`kit.days.label`, `ladders.measure.counteddays`,
 * `ladders.measure.perfect`): their plural on `need`, the days in (at most `need`) as words, since the phones' catalogs
 * cannot number a plural with two integers. */
export function dayCount(have, need) {
	return {need, have: number(Math.min(have, need))};
}

/** The Path's pace in each unit: one number, and a range. */
const SPANS = {
	days: ["ladders.path.span.days", "ladders.path.span.days.range"],
	weeks: ["ladders.path.span.weeks", "ladders.path.span.weeks.range"],
	months: ["ladders.path.span.months", "ladders.path.span.months.range"],
	years: ["ladders.path.span.years", "ladders.path.span.years.range"],
};

/** A span as the Path says it (the engine's {unit, low, high}: the Path's pace, Pro's forecast on a step's card since
 * slice 52) in words: "3–5 months", or "4 months" where both ends round alike. */
export function spanWords(pace) {
	const [one, range] = SPANS[pace.unit];
	return pace.low === pace.high ? t(one, {count: pace.high}) : t(range, {low: pace.low, high: pace.high});
}

/** A record or a count as its move keeps it: a number, a hold's time as a clock. */
export function amount(value, seconds) {
	return seconds ? clock(value) : number(value);
}

/** A goal in the phones' one grammar: "25+", a hold's "0:45+". */
export function goal(value, seconds) {
	return seconds ? t("ladders.next.time", {time: clock(value)}) : t("kit.goal.value", {goal: value});
}

/** The record a move window and a move's page show (`BookLadder.value`): its count or a hold's time (a loaded move's
 * weighted record in its words, `fact`'s); Max set for a rep or hold move without one that stands within the player's
 * reach; none for a move that keeps no count. */
export function moveValue(w, move, record, locked, fact) {
	const m = w.moves.get(move);
	const hold = m?.dose === "hold";
	if (record) return {kind: hold ? "time" : "count", value: record, words: loadedWords(w, move, fact)};
	return !locked && (m?.dose === "rep" || hold) ? {kind: "maxset"} : null;
}

/** A loaded move's weighted record ("10 kg × 20": the set with the best score, beside the reps record its rung reads);
 * none before a set with a weight. */
function loadedWords(w, move, fact) {
	return fact?.topLoad != null ? lifted(fact.topLoad, fact.topReps, w.moves.get(move)?.load === "each") : null;
}

/** A move's record in words (`recordOf`): a loaded move's weighted record, else its count or a hold's time; none
 * without one. */
export function recordWords(w, book, facts, move) {
	const record = recordOf(book, facts, move);
	if (!record) return null;
	return loadedWords(w, move, facts.get(move)) ?? amount(record, w.moves.get(move)?.dose === "hold");
}

/** A ladder row's end and its page's hero (`words`, its ladder's): its record, a pose's hold; none before the first
 * record (no Max set over a dash, 2026-11-02) and none on cardio. */
export function ladderValue(row, words) {
	if (row.opens) return null;
	const dose = words.dose;
	if (dose === "cardio") return null;
	if (dose === "pose") return row.hold ? {kind: "pose", value: row.hold} : null;
	return row.record > 0 ? {kind: row.seconds ? "time" : "count", value: row.record} : null;
}

/** A record value's caps label and its number, as RecordValue writes them. */
export function valueWords(value) {
	switch (value.kind) {
		case "count": return {label: t("ladders.record.label"), number: value.words ?? number(value.value)};
		case "time": return {label: t("ladders.record.label"), number: clock(value.value)};
		case "pose": return {label: t("ladders.hold"), number: clock(value.value)};
		// None yet before the first record (the 2026-10-21 review: Max set over a dash read as a broken number)
		default: return {label: t("ladders.record.label"), number: t("kit.record.none"), dim: true};
	}
}

/** A record value as the kit draws it: its label in caps over its number. */
export function valueCell(value, leading = false) {
	const words = valueWords(value);
	return h("div", {class: `qp-book-value${leading ? " qp-leading" : ""}`},
		h("span", {class: "qp-type-caps", text: words.label}),
		// None yet in body words, never as loud as a record's number
		h("span", {class: words.dim ? "qp-type-body qp-dim" : "qp-type-num-medium", text: words.number}));
}

/** A move's record as Book shows it: the record of the ladder that stands on it (the higher of the game's sheet and
 * the app's own), else the move's own. */
export function recordOf(book, facts, move) {
	return book.ladders.find((l) => l.move === move && l.record > 0)?.record ?? facts.get(move)?.record ?? null;
}

/** Whether a move stands above the player's reach (`fact.above`, ns.AboveReach) or in a family not open yet, whose
 * every move waits for its opening: no Max set, its figure under the ahead line, its tile in the ahead face. */
export function aboveReach(w, book, fact, move) {
	const ladder = w.moves.get(move)?.ladder;
	return Boolean(fact?.above) || (ladder != null && book.ladders.find((l) => l.ladder === ladder)?.waits === "closed");
}

/** A trained move's quality: its own rung's on its ladder, Starter for a trained move no ladder places; none
 * untrained. */
export function tileQuality(fact) {
	if (!fact?.trained) return null;
	return fact.quality ?? 1;
}

/** "How close to the goal pose?"'s Perfect (Core.lua ns.Enum.mark). */
const PERFECT = 4;

/** What opens a family not open yet, in the game's words: a record, a step of another ladder, or since slice 42 a
 * pose's goal pose (the planche's wrist lean), in the check's words since the fix round (2026-10-17): "Opens at a
 * Perfect {move}" ("… a Good …" where Good does it), never by the step that pose stands on, which every player has. */
export function opensWords(w, opens) {
	if (opens.kind === "record") {
		return t("ladders.opens.record", {move: moveName(w, opens.move), count: amount(opens.count, opens.seconds)});
	}
	if (opens.kind === "pose") {
		return t(opens.ask >= PERFECT ? "ladders.opens.pose.perfect" : "ladders.opens.pose", {move: moveName(w, opens.move)});
	}
	return t("ladders.opens", {ladder: w.ladders.get(opens.ladder)?.name ?? "", step: opens.step});
}

/** How well the player knows a move trained that keeps no record ("Tried (1 of 3)"); none for a move with a record,
 * not trained, or with no day counted ("New" would gainsay its trained edge). */
export function knownWords(fact, learned) {
	if (!fact || fact.record || !fact.trained || fact.familiarity < 1) return null;
	return familiarity(fact.familiarity, fact.days, learned);
}

/** A familiarity step in the game's words (Stats.lua ns.Familiarity). */
export function familiarity(step, days, need) {
	const word = t(["kit.familiarity.new", "kit.familiarity.tried", "kit.familiarity.familiar",
		"kit.familiarity.learned"][Math.max(0, Math.min(3, step))]);
	return step === 2 ? t("kit.familiarity.days", {word, days, need, step, total: 3})
		: t("kit.familiarity.count", {word, step, total: 3});
}

/** What of the setup leaves a move out, in Your setup's words. */
export function gapWords(w, gap) {
	const words = w.words.gear[gap.key] ?? gap.key;
	if (gap.kind === "gear") return t("ladders.move.setup.gear", {gear: words});
	return gap.kind === "off" ? t("ladders.move.setup.off", {setting: words})
		: t("ladders.move.setup.on", {setting: words});
}

/** The line under Include in pumps: a move set aside while a better one is in the player's pumps ("Set aside: Goblet
 * squat takes its place"); none else. */
function includeLine(w, fact) {
	return fact?.better != null ? t("ladders.move.better", {move: moveName(w, fact.better)}) : null;
}

/** A move's switches, one block on its window and its page: Favourite, and Include in pumps (the floor move always in,
 * on and disabled; a move the setup leaves out off and disabled with the setting's line and a row that opens Your
 * setup; a move set aside for a better one with the line that names it; an opt-in move off until switched on). A
 * switch sends its intent and moves when `book` says so, never before (a refused one stays as it was): its page draws
 * again from `book`, its window follows it (`follow`). */
export function switches(w, book, facts, move, close) {
	const fact = facts.get(move);
	const favourite = h("qp-row", {heading: t("ladders.move.favourite"), glyph: "star", trailing: "toggle",
		checked: book.favourites.includes(move), "data-focus-key": "switch:favourite"});
	favourite.addEventListener("qp-change", (e) => {
		intent("book.favourite", {move, on: e.detail}).catch(console.error);
	});
	const rows = [favourite];
	if (move === w.words.floorMove) {
		rows.push(h("qp-row", {heading: t("ladders.move.include"), glyph: "action_pump", trailing: "toggle",
			checked: true, disabled: true}));
	} else if (fact?.gap) {
		rows.push(h("qp-row", {heading: t("ladders.move.include"), meta: gapWords(w, fact.gap), glyph: "action_pump",
			trailing: "toggle", disabled: true}));
		rows.push(h("qp-row", {heading: t("menu.setup"), glyph: "action_character", trailing: "nav",
			"on:click": () => {
				close?.();
				openMenu(SETUP);
			}}));
	} else {
		const on = !fact?.off;
		const include = h("qp-row", {heading: t("ladders.move.include"), meta: includeLine(w, fact),
			glyph: "action_pump", trailing: "toggle", checked: on, "data-focus-key": "switch:include"});
		include.addEventListener("qp-change", (e) => {
			intent("book.include", {move, on: e.detail}).catch(console.error);
		});
		rows.push(include);
	}
	return rows;
}

/** Keeps the switches of `move` under `root` on what `book` says while its window stands; returns the way to stop. */
export function follow(w, root, move) {
	return subscribe("book", (book) => {
		const favourite = root.querySelector('[data-focus-key="switch:favourite"]');
		if (favourite) favourite.checked = book.favourites.includes(move);
		const include = root.querySelector('[data-focus-key="switch:include"]');
		if (include) {
			const fact = factsOf(book).get(move);
			include.checked = !fact?.off;
			include.meta = includeLine(w, fact);
		}
	});
}

/** The confirmation before a step that cannot be undone (a record cleared, a ladder started over, a place deleted, a
 * device removed from sync, the account deleted): its question (`heading`), what goes and what stays (`body`, none
 * where the question says it all), and the step's destructive button (`confirm`), which acts and closes it; its ×,
 * Escape and the quiet `keep` where one is named (a place's Keep it) keep everything. It leaves the page once closed. */
export function confirmWindow(heading, body, confirm, act, keep = null) {
	const win = h("qp-window", {heading, ...(keep ? {secondary: keep} : {})});
	win.addEventListener("qp-secondary", () => win.close());
	win.content.append(...(body ? [h("p", {class: "qp-type-body qp-text2", text: body})] : []),
		h("qp-secondary-button", {label: confirm, destructive: true, "on:click": () => {
			act();
			win.close();
		}}));
	win.addEventListener("qp-closed", () => win.remove(), {once: true});
	win.open();
	return win;
}

/** A text as the Moves search compares it (the phones' `MoveSearch.fold`): canonically decomposed, each code point
 * lower-cased alone, every character but a letter or a decimal digit dropped. */
export function fold(text) {
	let out = "";
	for (const c of text.normalize("NFD")) out += c.toLowerCase();
	return out.replace(/[^\p{L}\p{Nd}]/gu, "");
}
