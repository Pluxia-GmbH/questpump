// Book (docs/ui.md, "Hero, the Book and the Log" and "The Book's ladders and paths"; the phones' BookTab, design v3
// §4): the root's TopBar over the Ladders | Moves switch. Ladders: the talent trees' Segmented (Strength, Endurance,
// Flexibility: the first until the player picks another), the tree's Up next (the climbs closest to their next step, the
// `book` topic's `next`, desk/book.rs `up_next`: each its next step's tile, move and ladder, what the climb still takes
// with its days as pips or its ring's share as a bar, opening the ladder's page at that step; with none, the tree's Next
// line: why nothing climbs now), then one card of the tree's ladders in the tree's order on the higher rung of the game's
// sheet and the app's own (its move's tile, the family in caps, the move, the record (nothing before the first), a pose's
// hold, a chevron on cardio; its QualityWord with "Step 5 of 22" and its steps as one meter in their tiers' colours), a
// row opening the ladder's page with its path, a family not open yet's its card with what opens it. Moves: the search
// (Cmd/Ctrl+F, from any other root too; each move's terms: its name, muscles, block, group and equipment), the filters
// (All, Favourites, Not trained yet, the regions, the blocks, "Off: 3" while a move is switched off), then every move the
// search and the filter leave by family, the grid as wide as the window (three to six tiles a row); the pointer resting
// on a tile shows its tooltip, a click opens its window with its Start, grown out of the tile and shrinking back into it
// as the player closes it (slice 46). After the words or the filter rest, a screen reader hears how many moves are left.
// A redraw keeps the focus on the row, the chip or the tile it was on (its window, once closed, gives it back to the
// tile). A move started from a card or a window returns there once its pump has gone (`resume`, the router's at the
// cover's end). From `book` and `book.words` (desk/book.rs) and the `pump` topic's gate.

import {get} from "../../lib/bridge.js";
import {t} from "../../lib/copy.js";
import {announce, keepFocus} from "../../lib/focus.js";
import {push, register} from "../../lib/router.js";
import {h, qualityName, styled} from "../../kit/element.js";
import {page, watch} from "../page.js";
import {goLink, openCardWindow, openMoveWindow, startMove} from "./card.js";
import {
	LADDER, factsOf, fold, knownWords, ladderValue, lookup, moveName, opensWords, qualityWord, recordWords, tile,
	tileQuality, valueCell,
} from "./common.js";
import {ladderOf, stepAt, stepTip, treeNext, waitingNote} from "./steps.js";

/** How long the words and the filter rest before a screen reader hears the count. */
const SETTLE_MS = 800;

/** The pips a page shows while the engine folds. */
export function loading() {
	return h("div", {class: "qp-book-loading", role: "img", "aria-label": t("kit.button.busy")},
		h("span", {class: "qp-busy"}, h("i"), h("i"), h("i")));
}

/** A step's quality as its word with the step's pips beside it. */
export function qualityLine(w, quality, step) {
	return h("div", {class: "qp-quality-line"}, h("qp-quality-word", {quality, word: qualityWord(w, quality)}),
		h("qp-step-pips", {step, quality}));
}

/** A ladder's steps that hold a move, from its first up (the Book's steps stand from the top down). */
export function stepsUp(row) {
	return row.steps.filter((s) => s.rung && s.move).reverse();
}

/** A ladder's steps as one meter: a segment a step that holds a move, from its first up, the steps the player has (on
 * the row's rung or under it) in their tier's colour, the rest the track, so the colour turns where the tier does.
 * Decoration: the row reads its step. */
function stepMeter(row) {
	return h("span", {class: "qp-step-meter", "aria-hidden": "true"}, ...stepsUp(row).map((s) => {
		const tone = s.rung <= row.rung ? qualityName(s.quality) : null;
		return tone ? styled(h("i", {"data-on": ""}), {"--qp-tone": `var(--qp-quality-${tone})`}) : h("i");
	}));
}

/** A ladder's row: the whole row opens its page; a ladder that waits whole (a family not open yet, or one the setup or
 * the player's own list leaves no move, whatever its rung) opens its card, which says why and what to change. */
function ladderRow(w, book, row, openCard) {
	const words = w.ladders.get(row.ladder);
	const name = w.moves.get(row.move)?.name ?? "";
	if (row.waits || row.opens || row.rung === 0) {
		const lad = ladderOf(w, book, row.ladder);
		const line = row.opens ? opensWords(w, row.opens) : lad ? waitingNote(lad) : "";
		return h("qp-card-row", {class: "qp-ladder-row"},
			h("button", {type: "button", class: "qp-ladder-body", "data-focus-key": `ladder:${row.ladder}`,
				"on:click": () => openCard(row.ladder)},
			tile(w, row.move, {size: 60, name, face: "ahead"}),
			h("div", {class: "qp-ladder-words"},
				h("span", {class: "qp-type-caps", text: words.name}),
				h("span", {class: "qp-type-subtitle", text: line})),
			h("span", {class: "qp-ladder-chevron", "aria-hidden": "true"})));
	}
	const value = ladderValue(row, words);
	const top = stepsUp(row).at(-1);
	return h("qp-card-row", {class: "qp-ladder-row"},
		h("button", {type: "button", class: "qp-ladder-body", "data-focus-key": `ladder:${row.ladder}`,
			"on:click": () => push(LADDER, {ladder: row.ladder})},
			tile(w, row.move, {size: 60, quality: row.quality, name, word: qualityWord(w, row.quality), step: row.step}),
			h("div", {class: "qp-ladder-words"},
				h("span", {class: "qp-type-caps", text: words.name}),
				h("span", {class: "qp-type-body-strong", text: name}),
				h("div", {class: "qp-quality-line"}, h("qp-quality-word", {quality: row.quality,
					word: qualityWord(w, row.quality)}),
				top ? h("span", {class: "qp-type-meta", text: t("kit.pips.label", {step: row.step, total: top.step})}) : ""),
				stepMeter(row)),
			value ? valueCell(value) : words.dose === "cardio" ? h("span", {class: "qp-ladder-chevron", "aria-hidden": "true"}) : null));
}

/** A climb of a tree's Up next: its next step's tile (its pose greyed, a step ahead), its move over its ladder, what the
 * climb still takes (the player's step's key line, steps.js `stepTip`) with its days as pips where it counts days,
 * else its ring's share as a bar; read as one, it opens the ladder's page at that step (`open`: pushed on the Book's
 * stack; Pump home's row, the router's `openLadder`). */
export function nextRow(w, book, id, open = (ladder, step) => push(LADDER, {ladder, step})) {
	const lad = ladderOf(w, book, id);
	const next = lad && stepAt(lad, lad.cur + 1);
	if (!lad?.now || !next?.move) return null;
	const tip = stepTip(w, lad, lad.now, "now");
	const name = moveName(w, next.move);
	const under = [];
	if (tip.key) under.push(h("span", {class: "qp-type-meta", text: tip.key}));
	if (tip.pips?.need > 0) {
		under.push(h("qp-day-pips", {have: Math.min(tip.pips.have, tip.pips.need), need: tip.pips.need, "aria-hidden": "true"}));
	} else {
		const fill = Math.max(0, Math.min(1000, lad.row.ring?.fill ?? 0));
		under.push(styled(h("span", {class: "qp-fill-bar", "aria-hidden": "true"}), {"--qp-fill": fill / 1000}));
	}
	return h("qp-card-row", {class: "qp-ladder-row"},
		h("button", {type: "button", class: "qp-ladder-body", "data-focus-key": `next:${id}`,
			"aria-label": [name, lad.words.name, tip.key].filter(Boolean).join(", "),
			"on:click": () => open(id, next.step)},
		tile(w, next.move, {size: 44, face: "ahead"}),
		h("div", {class: "qp-ladder-words"},
			h("span", {class: "qp-type-body-strong", text: name}),
			h("span", {class: "qp-type-meta", text: lad.words.name}), ...under),
		h("span", {class: "qp-ladder-chevron", "aria-hidden": "true"})));
}

/** A card of rows drawn on their own, each its place in it. */
export function rowsCard(rows) {
	const card = h("div", {class: "qp-book-rows"}, ...rows);
	card.querySelectorAll("qp-card-row").forEach((row, i, all) => {
		row.place = all.length === 1 ? "alone" : i === 0 ? "first" : i === all.length - 1 ? "last" : "middle";
	});
	return card;
}

/** A tree's Up next under its label: its climbs' rows, else the tree's Next line (why nothing climbs now), else
 * nothing. `pump` the pump topic (the felt-unwell lock's end). */
function upNext(w, book, tree, pump) {
	const rows = (tree.next ?? []).map((id) => nextRow(w, book, id)).filter(Boolean);
	if (rows.length) return [h("qp-section-label", {label: t("ladders.next")}), rowsCard(rows)];
	const line = treeNext(w, book, tree, pump);
	if (!line) return [];
	return [h("qp-section-label", {label: t("ladders.next")}),
		h("qp-card", {}, h("p", {class: "qp-type-body", text: line}))];
}

/** The Moves side's filters in their order, Off with its count while a move is off. */
function filters(w, offCount) {
	return [
		{id: "all", label: t("ladders.filter.all")},
		{id: "favourites", label: t("ladders.filter.favourites")},
		{id: "untrained", label: t("character.muscle.untrained")},
		...w.words.regions.map((r) => ({id: `region:${r.key}`, label: r.name})),
		...w.words.blocks.map((b) => ({id: `block:${b.key}`, label: b.name})),
		...(offCount ? [{id: "off", label: t("ladders.moves.off", {count: offCount})}] : []),
	];
}

register("book", () => {
	const view = page({heading: t("bar.book"), root: true});
	// the wide column, its TopBar as wide
	view.el.classList.add("qp-wide");
	view.body.classList.add("qp-wide");
	const segment = h("qp-segmented", {value: "ladders"});
	segment.options = [{value: "ladders", label: t("ladders.segment.ladders")},
		{value: "moves", label: t("ladders.segment.moves")}];
	const ladders = h("div", {class: "qp-book-side qp-ladders"});
	const moves = h("div", {class: "qp-book-side qp-book-moves", hidden: true});
	const search = h("qp-text-input", {mode: "search", label: t("ladders.search"), prompt: t("ladders.search"),
		"clear-label": t("ladders.search.clear")});
	const chips = h("div", {class: "qp-filter-row", role: "group", "aria-label": t("ladders.segment.moves")});
	const grid = h("div", {class: "qp-book-families"});
	const tooltip = h("qp-tooltip");
	// the Ladders side: the trees' Segmented, the tree's Up next, its ladders
	const trees = h("qp-segmented", {label: t("ladders.segment.ladders")});
	const next = h("div", {class: "qp-book-next"});
	const list = h("div", {class: "qp-book-rows"});
	ladders.append(trees, next, list);
	moves.append(search, chips, grid);
	view.body.append(segment, ladders, moves);
	document.body.append(tooltip);
	view.stops.push(() => tooltip.remove());

	let w = null;
	let book = null;
	let pump = get("pump");
	let tree = null; // the tree the player picked, by its key (the first until then)
	let filter = "all";
	let terms = null; // each move's terms, folded
	let settle = null;
	let returning = null; // a move started from the list or a move's window, until its pump has gone

	/** A ladder's card from the list (at `step`, else the ladder's own), its Start returning to its row; a start that
	 * goes nowhere opens it again. */
	const listCard = (ladder, step = null, counted = true) => {
		openCardWindow({w, place: "list", ladder, step, go: goLink, counted,
			start: (target, from) => {
				returning = from;
				startMove(target, from, () => {
					if (returning !== from) return;
					returning = null;
					listCard(ladder, step, false);
				});
			}});
	};
	/** A move's window from Moves, its Start returning to it; a start that goes nowhere opens it again. Opened by a tap
	 * on its tile (`tapped`), it grows out of that tile (never on a return or a start that went nowhere). */
	const moveWindow = (move, tapped = false) => {
		const origin = tapped ? () => grid.querySelector(`[data-focus-key="move:${move}"]`) : null;
		openMoveWindow(w, book, move, {origin, start: (target, from) => {
			const back = {...from, move};
			returning = back;
			startMove(target, from, () => {
				if (returning !== back) return;
				returning = null;
				if (w && book?.ready) moveWindow(move);
			});
		}});
	};

	const choose = (side) => {
		segment.value = side;
		ladders.hidden = side !== "ladders";
		moves.hidden = side !== "moves";
		tooltip.hide();
	};
	segment.addEventListener("qp-change", (e) => choose(e.detail));
	trees.addEventListener("qp-change", (e) => {
		tree = e.detail;
		drawLadders();
	});

	const drawLadders = () => keepFocus(ladders, () => {
		if (!book.ready) {
			trees.hidden = true;
			next.replaceChildren();
			list.replaceChildren(loading());
			return;
		}
		const sheet = w.words.trees;
		const shown = sheet.find((tr) => tr.key === tree) ?? sheet[0] ?? null;
		trees.hidden = sheet.length < 2;
		trees.options = sheet.map((tr) => ({value: tr.key, label: tr.name}));
		trees.value = shown?.key ?? null;
		const order = shown ? shown.branches.flatMap((b) => b.ladders) : book.ladders.map((l) => l.ladder);
		const rows = order.map((id) => book.ladders.find((l) => l.ladder === id)).filter(Boolean);
		const state = shown && book.trees.find((tr) => tr.tree === shown.key);
		next.replaceChildren(...(state ? upNext(w, book, state, pump) : []));
		list.replaceChildren(rowsCard(rows.map((row) => ladderRow(w, book, row, (ladder) => listCard(ladder)))));
	});

	/** Whether the search and the filter leave a move. */
	const keeps = (move, facts, query) => {
		const m = w.moves.get(move);
		if (query && !terms.get(move)?.some((term) => term.includes(query))) return false;
		const fact = facts.get(move);
		switch (true) {
			case filter === "all": return true;
			case filter === "favourites": return book.favourites.includes(move);
			case filter === "untrained": return !fact?.trained;
			case filter === "off": return Boolean(fact?.off);
			case filter.startsWith("region:"): return m?.region === filter.slice(7);
			case filter.startsWith("block:"): return m?.block === filter.slice(6);
			default: return true;
		}
	};

	/** A move's tooltip line: its record (a loaded move's with its weight), else how well the player knows it, else Not
	 * trained yet. */
	const tipLine = (move, facts) => {
		const fact = facts.get(move);
		const record = recordWords(w, book, facts, move);
		if (record) return t("kit.record.yours", {record});
		if (fact?.trained) return knownWords(fact, w.words.learnedDays);
		return t("character.muscle.untrained");
	};

	const cell = (move, facts) => {
		const m = w.moves.get(move);
		const fact = facts.get(move);
		const quality = tileQuality(fact);
		const record = recordWords(w, book, facts, move);
		const known = knownWords(fact, w.words.learnedDays);
		const el = tile(w, move, {size: 60, quality, off: Boolean(fact?.off),
			favourite: book.favourites.includes(move), action: true, name: m.name,
			word: quality ? qualityWord(w, quality) : null, step: fact?.step ?? 0, "data-focus-key": `move:${move}`});
		el.addEventListener("click", () => {
			tooltip.hide();
			moveWindow(move, true);
		});
		const show = () => {
			const shownQuality = fact?.quality ?? (fact?.trained ? 1 : null);
			Object.assign(tooltip, {name: m.name,
				quality: shownQuality, qualityWord: shownQuality ? qualityWord(w, shownQuality) : null,
				meta: tipLine(move, facts) ?? ""});
			tooltip.showAt(el);
		};
		el.addEventListener("pointerenter", show);
		el.addEventListener("pointerleave", () => tooltip.leave());
		const lines = [h("span", {class: `qp-type-meta${quality ? "" : " qp-dim"}`, text: m.name})];
		if (quality) lines.push(h("qp-quality-word", {quality, word: qualityWord(w, quality)}));
		// what the tooltip says, for a screen reader after the tile's own words
		const more = record ? t("kit.record.yours", {record}) : known;
		if (more) lines.push(h("span", {class: "qp-unseen", text: more}));
		return h("div", {class: "qp-move-cell"}, el, ...lines);
	};

	const drawMoves = () => keepFocus(moves, () => {
		if (!book.ready) {
			grid.replaceChildren(loading());
			return;
		}
		const facts = factsOf(book);
		const offCount = book.moves.filter((m) => m.off).length;
		if (filter === "off" && !offCount) filter = "all";
		chips.replaceChildren(...filters(w, offCount).map((f) => h("qp-choice-chip", {label: f.label,
			selected: f.id === filter, "data-focus-key": `filter:${f.id}`, "on:click": () => {
				filter = f.id;
				tooltip.hide();
				drawMoves();
				said();
			}})));
		const query = fold(search.value);
		const shown = w.words.families.map((f) => ({ladder: f.ladder, moves: f.moves.filter((m) => keeps(m, facts, query))}))
			.filter((f) => f.moves.length);
		if (!shown.length) {
			const words = empty(query);
			const state = h("qp-empty-state", {glyph: "action_book", heading: words.title, "on:qp-action": () => {
				if (query) {
					search.value = "";
					search.control.dispatchEvent(new Event("input", {bubbles: true}));
				} else {
					filter = "all";
					drawMoves();
				}
			}});
			state.action = words.button;
			grid.replaceChildren(state);
			return;
		}
		grid.replaceChildren(...shown.flatMap((f) => [
			h("qp-section-label", {label: f.ladder ? w.ladders.get(f.ladder)?.name ?? "" : t("ladders.moves.other")}),
			h("div", {class: "qp-move-grid"}, ...f.moves.map((m) => cell(m, facts))),
		]));
	});

	/** The empty grid's heading and its one button by its cause: words typed, Favourites with none, Not trained yet
	 * with none. */
	const empty = (query) => {
		if (query) return {title: t("ladders.search.none"), button: t("ladders.search.clear")};
		if (filter === "favourites") return {title: t("ladders.favourites.none"), button: t("ladders.filter.showall")};
		if (filter === "untrained") return {title: t("ladders.untrained.none"), button: t("ladders.filter.showall")};
		return {title: t("ladders.search.none"), button: t("ladders.filter.showall")};
	};

	/** Once the words and the filter rest, a screen reader hears how many moves are left, or the empty state. */
	const said = () => {
		clearTimeout(settle);
		settle = setTimeout(() => {
			const count = grid.querySelectorAll(".qp-move-cell").length;
			announce(count ? t("ladders.moves.count", {count}) : empty(fold(search.value)).title);
		}, SETTLE_MS);
	};
	view.stops.push(() => clearTimeout(settle));

	search.addEventListener("input", () => {
		tooltip.hide();
		drawMoves();
		said();
	});
	moves.addEventListener("scroll", () => tooltip.hide(), true);
	view.el.addEventListener("scroll", () => tooltip.hide(), true);

	let wanted = null; // the step Pump home's embers row asked for, until the Book is built to show it
	/** The step `wanted` names (the router's `openStep`): its card over the Ladders. */
	const openWanted = () => {
		if (!wanted || !w || !book?.ready) return;
		const {ladder, step} = wanted;
		wanted = null;
		choose("ladders");
		listCard(ladder, step);
	};
	const draw = () => {
		if (!w || !book) return;
		drawLadders();
		drawMoves();
		openWanted();
	};
	watch(view, "book.words", (words) => {
		w = lookup(words);
		terms = new Map(words.moves.map((m) => [m.move, m.terms.map(fold).filter(Boolean)]));
		draw();
	});
	watch(view, "book", (b) => {
		book = b;
		draw();
	});
	watch(view, "pump", (p) => {
		pump = p;
		// a pump on screen keeps only its own return: none for a pump that is no move started here (one the engine
		// refused composed none), the list's or the move window's else
		if (p.pump) {
			const place = p.pump.what === "move" ? p.pump.from?.place : null;
			if (place !== "list" && place !== "move") returning = null;
		}
		if (w && book?.ready) drawLadders();
	});
	view.primary = () => segment;
	view.openStep = (ladder, step) => {
		wanted = {ladder, step};
		openWanted();
	};
	// a ladder's page at a step (Pump home's closest step-up, the router's `openLadder`), on the Ladders side
	view.openLadder = (ladder, step) => {
		choose("ladders");
		push(LADDER, {ladder, step});
	};
	// Moves with its search's keyboard: Cmd/Ctrl+F here, from another root, a page on one or the Menu (the router's
	// `find`)
	view.findMove = () => {
		choose("moves");
		search.focus();
	};
	view.onFind = () => view.findMove();
	// Escape: the Tooltip
	view.onKey = (e) => tooltip.escape(e);
	/** The pump's cover has gone (the router's `uncover`): a move started here comes back where it started, the list's
	 * row or the move's window again. */
	view.resume = () => {
		const from = returning;
		returning = null;
		if (from?.place === "move" && from.move != null && w && book?.ready) {
			moveWindow(from.move);
			return null;
		}
		if (from?.place === "list") return list.querySelector(`[data-focus-key="ladder:${from.ladder}"]`);
		return null;
	};
	// another root or the Menu takes the tooltip with the Book
	const hide = () => tooltip.hide();
	for (const type of ["qp-tab", "qp-menu"]) {
		document.addEventListener(type, hide, true);
		view.stops.push(() => document.removeEventListener(type, hide, true));
	}
	return view;
});
