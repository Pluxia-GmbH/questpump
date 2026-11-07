// Book (docs/ui.md, "Hero, the Book and the Log" and "The Book's ladders and paths"; the phones' BookTab, design v3
// §4): the root's TopBar with the search button beside the gear, over the ladders. Ladders: the talent trees' Segmented
// (Strength, Endurance, Flexibility: the first until the player picks another), then the tree's ladders in its order, a
// card a branch, on the higher rung of the game's sheet and the app's own: the move standing for the player's step named
// over its tile with that step's QualityWord under it, the next step's move named over its tile in its ahead face, and
// between them the ladder's name in caps over the ring's share as a round bar (the one progress mark) with what the
// climb still takes under it; a row opens the ladder's page with its path, a family not open yet's its card with what
// opens it. Moves, behind the search button
// (which closes it again, as Escape does): the search (Cmd/Ctrl+F, from any other root too; each move's terms: its name,
// muscles, block, group and equipment), the filters (All, Favourites, Not trained yet, the regions, the blocks, "Off: 3"
// while a move is switched off), then every move the search and the filter leave by family, the grid as wide as the
// window (three to six tiles a row); the pointer resting on a tile shows its tooltip, a click opens its window with its
// Start, grown out of the tile and shrinking back into it as the player closes it (slice 46). After the words or the
// filter rest, a screen reader hears how many moves are left. A redraw keeps the focus on the row, the chip or the tile
// it was on (its window, once closed, gives it back to the tile). A move started from a card or a window returns there
// once its pump has gone (`resume`, the router's at the cover's end). From `book` and `book.words` (questpump-desk/src/book.rs) and the
// `pump` topic.
import {t} from "../../lib/copy.js";
import {announce, keepFocus} from "../../lib/focus.js";
import {push, register} from "../../lib/router.js";
import {h, styled, wordsWhole} from "../../kit/element.js";
import {page, watch} from "../page.js";
import {goLink, openCardWindow, openMoveWindow, startMove} from "./card.js";
import {
	LADDER, factsOf, fold, knownWords, lookup, moveName, opensWords, qualityWord, recordWords, tile, tileQuality,
} from "./common.js";
import {kindOf, ladderOf, stepAt, stepTip, waitingNote} from "./steps.js";

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

/** The one progress mark of a climb, the ring's share (Progress.lua `ns.StepRing`: the record toward the top and the
 * days there in one share; `fill` per mille) as a round bar whose fill ends in a glowing tip, breathing on the tiles'
 * one clock (motion.trackTip, held under reduced motion): the Book's ladder rows and Pump home's closest step-up.
 * Decoration: the rows read their words. */
export function progressBar(fill) {
	const share = Math.max(0, Math.min(1000, fill ?? 0)) / 1000;
	return styled(h("span", {class: "qp-fill-bar", "aria-hidden": "true"}, h("span", {class: "qp-fill-bar-fill"}),
		h("i", {class: "qp-fill-bar-tip"})), {"--qp-fill": share, "--qp-tip-now": `${Math.round(performance.now())}ms`});
}

/** A move's name over its tile in a ladder's row: a hyphenated word (push-up, One-arm) never breaks at its hyphen. */
function overName(text, at) {
	return h("span", {class: "qp-type-body-strong qp-duo-name", "data-at": at}, wordsWhole(text));
}

/** A ladder's row (Rapha, 2026-11-04, the exploration Ladder Row; surface.LadderRow): a column at each end, the
 * player's step's move named over its tile in its quality and a soft light, its QualityWord under it, and the next
 * step's move named over its tile in the ahead face; between them the ladder's name in caps over the one progress
 * mark (`progressBar`), what the climb still takes under it (the player's step's key line, steps.js `stepTip`); at the
 * top step no bar and no next, the key line under the ladder's name. The whole row opens its page. A ladder that waits
 * whole (a family not open yet, or one the setup or the player's own list leaves no move, whatever its rung) shows its
 * first move in the ahead face and what opens it, and opens its card, which says why and what to change. */
function ladderRow(w, book, row, openCard) {
	const lad = ladderOf(w, book, row.ladder);
	if (!lad) return null;
	const name = moveName(w, row.move);
	const ladder = h("span", {class: "qp-type-ladder-name qp-duo-ladder", text: lad.words.name});
	if (row.waits || row.opens || row.rung === 0) {
		const line = row.opens ? opensWords(w, row.opens) : waitingNote(lad);
		return h("qp-card-row", {class: "qp-ladder-row"},
			h("button", {type: "button", class: "qp-ladder-body qp-ladder-duo", "data-focus-key": `ladder:${row.ladder}`,
				"aria-label": [lad.words.name, line].filter(Boolean).join(", "), "on:click": () => openCard(row.ladder)},
			overName(name, "now"), ladder,
			h("span", {class: "qp-duo-tile", "data-at": "now"}, tile(w, row.move, {size: 64, face: "ahead"})),
			line ? h("span", {class: "qp-type-meta qp-duo-key", "data-at": "top", text: line}) : ""));
	}
	const quality = qualityWord(w, row.quality);
	const st = lad.now;
	const kind = st ? kindOf(lad, st) : null;
	const key = st ? stepTip(w, lad, st, kind).key : "";
	const next = kind === "now" ? stepAt(lad, lad.cur + 1) : null;
	const nextName = next?.move ? moveName(w, next.move) : "";
	const owned = kind === "top" || (kind === "now" && !st.state.earn);
	return h("qp-card-row", {class: "qp-ladder-row"},
		h("button", {type: "button", class: "qp-ladder-body qp-ladder-duo", "data-focus-key": `ladder:${row.ladder}`,
			"aria-label": [lad.words.name, name, quality, nextName, key].filter(Boolean).join(", "),
			"on:click": () => push(LADDER, {ladder: row.ladder})},
		overName(name, "now"), ladder, nextName ? overName(nextName, "next") : "",
		h("span", {class: "qp-duo-tile qp-duo-spot", "data-at": "now"},
			tile(w, st?.move ?? row.move, {size: 64, face: "pose", quality: owned ? st.quality : null, moving: true,
				...(st?.mastered ? {mastered: ""} : {})})),
		nextName ? h("span", {class: "qp-duo-bar"}, progressBar(lad.row.ring?.fill),
			key ? h("span", {class: "qp-type-meta qp-duo-key", text: key}) : "") : "",
		nextName ? h("span", {class: "qp-duo-tile", "data-at": "next"}, tile(w, next.move, {size: 42, face: "ahead"})) : "",
		!nextName && key ? h("span", {class: "qp-type-meta qp-duo-key", "data-at": "top", text: key}) : "",
		h("qp-quality-word", {class: "qp-duo-word", quality: row.quality, word: quality})));
}

/** Pump home's closest step-up (the router's `openLadder` opens it): its next step's tile (its pose greyed, a step
 * ahead), its move over its ladder, what the climb still takes (the player's step's key line, steps.js `stepTip`) over
 * the one progress mark the Book's rows fill (`progressBar`); read as one, it opens the ladder's page at that step
 * (`open`). */
export function nextRow(w, book, id, open) {
	const lad = ladderOf(w, book, id);
	const next = lad && stepAt(lad, lad.cur + 1);
	if (!lad?.now || !next?.move) return null;
	const tip = stepTip(w, lad, lad.now, "now");
	const name = moveName(w, next.move);
	return h("qp-card-row", {class: "qp-ladder-row"},
		h("button", {type: "button", class: "qp-ladder-body", "data-focus-key": `next:${id}`,
			"aria-label": [name, lad.words.name, tip.key].filter(Boolean).join(", "),
			"on:click": () => open(id, next.step)},
		tile(w, next.move, {size: 44, face: "ahead"}),
		h("div", {class: "qp-ladder-words"},
			h("span", {class: "qp-type-body-strong", text: name}),
			h("span", {class: "qp-type-meta", text: lad.words.name}),
			tip.key ? h("span", {class: "qp-type-meta", text: tip.key}) : "",
			progressBar(lad.row.ring?.fill)),
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
	// Moves behind the TopBar's search button, which closes it again while it shows
	const find = h("qp-round-button", {icon: "search"});
	view.bar.action = find;
	const ladders = h("div", {class: "qp-book-side qp-ladders"});
	const moves = h("div", {class: "qp-book-side qp-book-moves", hidden: true});
	const search = h("qp-text-input", {mode: "search", label: t("ladders.search"), prompt: t("ladders.search"),
		"clear-label": t("ladders.search.clear")});
	const chips = h("div", {class: "qp-filter-row", role: "group", "aria-label": t("ladders.segment.moves")});
	const grid = h("div", {class: "qp-book-families"});
	const tooltip = h("qp-tooltip");
	// the Ladders side: the trees' Segmented, then the tree's ladders, a card a branch
	const trees = h("qp-segmented");
	const list = h("div", {class: "qp-book-branches"});
	ladders.append(trees, list);
	moves.append(search, chips, grid);
	view.body.append(ladders, moves);
	document.body.append(tooltip);
	view.stops.push(() => tooltip.remove());

	let w = null;
	let book = null;
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
		ladders.hidden = side !== "ladders";
		moves.hidden = side !== "moves";
		find.icon = side === "moves" ? "close" : "search";
		tooltip.hide();
	};
	find.addEventListener("click", () => {
		if (moves.hidden) view.findMove();
		else choose("ladders");
	});
	trees.addEventListener("qp-change", (e) => {
		tree = e.detail;
		drawLadders();
	});

	const drawLadders = () => keepFocus(ladders, () => {
		if (!book.ready) {
			trees.hidden = true;
			list.replaceChildren(loading());
			return;
		}
		const sheet = w.words.trees;
		const shown = sheet.find((tr) => tr.key === tree) ?? sheet[0] ?? null;
		trees.hidden = sheet.length < 2;
		trees.options = sheet.map((tr) => ({value: tr.key, label: tr.name}));
		trees.value = shown?.key ?? null;
		const branches = shown ? shown.branches.map((b) => b.ladders) : [book.ladders.map((l) => l.ladder)];
		list.replaceChildren(...branches.map((ids) => ids.map((id) => book.ladders.find((l) => l.ladder === id))
			.filter(Boolean).map((row) => ladderRow(w, book, row, (ladder) => listCard(ladder))).filter(Boolean))
			.filter((rows) => rows.length).map(rowsCard));
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
		// a pump on screen keeps only its own return: none for a pump that is no move started here (one the engine
		// refused composed none), the list's or the move window's else
		if (p.pump) {
			const place = p.pump.what === "move" ? p.pump.from?.place : null;
			if (place !== "list" && place !== "move") returning = null;
		}
	});
	view.primary = () => (moves.hidden && !trees.hidden ? trees : find);
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
	// Escape: the Tooltip, else Moves back to the ladders
	view.onKey = (e) => {
		if (tooltip.escape(e)) return true;
		if (e.key !== "Escape" || moves.hidden) return false;
		choose("ladders");
		find.focus();
		return true;
	};
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
