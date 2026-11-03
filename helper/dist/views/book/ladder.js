// A ladder's page (docs/ui.md, "Hero, the Book and the Log"; the phones' LadderDetail, design v3 §4), pushed from its
// row in Book: the DetailBar with the family's name; the hero card with the figure of the move standing for the rung
// (pausable), the move's name, its quality with the step's pips, the record with the goal ("Goal" over "16+"; nothing
// before the first), a pose family's hold, mark and days that counted; Share the record at the foot once a rep or hold
// ladder has one; the ladder's path (docs/ui.md "The Book's ladders and paths", 2026-10-29): every step from its first
// up in the trees' words, the tiers named where the colour turns, the player's step ringed with its goal, a step ahead
// dimmed with what opens it, the steps of other ladders each step leads to, each row opening the step's card (card.js,
// the kit's Window from the foot: its figure, what it takes, its Start; slice 38), which grows out of the row's tile and
// shrinks back into it (slice 46), the step an Up next row asked brought into view; the Path under it (the road to the
// ladder's orange and gold, 2026-10-09: the engine's `path`), with Pro its pace, without Pro one row that offers the
// pace and opens Pro's page in the Menu; and with Pro, the rung as a stair and the record over time (`book.chart`, which
// `book.ladder` asks for while the page stands). Without Pro no chart and no word of one. Last, on a strength or pose
// ladder, Start this ladder over behind its confirmation. A ladder no longer open shows its bar alone, to go back. A
// Book that leaves the hero as it was keeps it, its figure playing on, and draws the rest again.

import {intent} from "../../lib/bridge.js";
import {t} from "../../lib/copy.js";
import {keepFocus} from "../../lib/focus.js";
import {clock, LOCALE} from "../../lib/format.js";
import {openMenu, push, register} from "../../lib/router.js";
import {openShare} from "../../lib/share.js";
import {h, styled, token} from "../../kit/element.js";
import {page, watch} from "../page.js";
import {loading, qualityLine, stepsUp} from "./book.js";
import {goLink, openCardWindow, startMove} from "./card.js";
import {chartCard, recordChart, stairChart} from "./charts.js";
import {
	LADDER, confirmWindow, goal, ladderValue, lookup, moveName, number, qualityWord, spanWords, tile, valueCell,
} from "./common.js";
import {beadsOf, kindOf, ladderOf, linksOf, stepTip} from "./steps.js";

const SVG = "svg:";

/** Pro's page in the Menu, which the Path's row opens without Pro. */
const PRO = "desk.menu.pro"; // copy-lint: not shown (Pro's page's view id, the Menu's Pro row)
/** "How close to the goal pose?"'s Perfect (Core.lua ns.Enum.mark): a pose climb's `ask` where only it counts a day. */
const PERFECT = 4;
const LIST = new Intl.ListFormat(LOCALE, {type: "conjunction", style: "narrow"});

/** The share card's meta line: the character and the family, or the family. */
function footer(book, ladder) {
	return book.character ? t("ladders.card.footer", {character: book.character, ladder: ladder.name}) : ladder.name;
}

/** The Path (the road to the ladder's orange and gold steps) under its label, one card: the goal (the steps to the
 * step it leads to, under that step's move; every step the player's on the top one), with Pro the pace once the engine
 * has one (the steps themselves are the path above it, `pathCard`); without Pro, once the engine has a pace, one row
 * that offers it and opens Pro's page in the Menu. None for a family not open yet. */
function roadCard(w, row, pro) {
	const path = row.path;
	if (!path) return [];
	const items = [];
	const target = path.steps.find((st) => st.rung === path.target);
	if (target) {
		const count = path.steps.filter((st) => st.rung <= target.rung).length;
		const heading = t("ladders.path.goal", {count, quality: qualityWord(w, target.quality)});
		const name = moveName(w, target.move);
		items.push(h("div", {class: "qp-step-row", role: "group", "aria-label": LIST.format([heading, name])},
			tile(w, target.move, {size: 44, face: "ahead", quality: null}),
			h("div", {class: "qp-step-words"}, h("span", {class: "qp-type-body-strong", text: heading}),
				h("span", {class: "qp-type-meta", text: name}))));
	} else {
		items.push(h("qp-row", {heading: t("ladders.path.top"), glyph: "mark_check", trailing: "shown"}));
	}
	if (pro && path.pace) {
		items.push(h("qp-row", {heading: t("ladders.path.pace", {span: spanWords(path.pace)}), glyph: "mark_clock",
			trailing: "shown"}));
	}
	if (!pro && target && path.pace) {
		items.push(h("qp-row", {heading: t("ladders.path.pro", {quality: qualityWord(w, target.quality)}), glyph: "helm",
			trailing: "nav", "data-focus-key": "path:pro", "on:click": () => openMenu(PRO)}));
	}
	return [h("qp-section-label", {label: t("ladders.path")}), h("qp-card", {rows: true, class: "qp-steps"}, ...items)];
}

/** The rounded square a ring strokes round a step's tile, from 12 o'clock clockwise, in a box of `size` px: its half
 * side `a` and corner radius `r` about the centre (the phones' ringPath, so a fill's share means the same point). */
function ringPath(size, a, r) {
	const c = size / 2;
	const [x0, x1, y0, y1] = [c - a, c + a, c - a, c + a];
	return `M ${c} ${y0} H ${x1 - r} A ${r} ${r} 0 0 1 ${x1} ${y0 + r} V ${y1 - r} A ${r} ${r} 0 0 1 ${x1 - r} ${y1} `
		+ `H ${x0 + r} A ${r} ${r} 0 0 1 ${x0} ${y1 - r} V ${y0 + r} A ${r} ${r} 0 0 1 ${x0 + r} ${y0} Z`;
}

/** The ticks' dashes along a ring of 1000 (their per mille, a dash of `width` each): a dash array starts with a dash,
 * so an empty one first, then each gap before a tick and the tick. */
function tickDashes(ticks, width) {
	const out = [0];
	let at = 0;
	for (const tick of ticks) {
		const from = Math.max(at, tick - width / 2);
		out.push(from - at, width);
		at = from + width;
	}
	out.push(Math.max(0, 1000 - at));
	return out.join(" ");
}

/** The ring round the player's step's tile (ns.StepRing): its track, its fill in per mille from 12 o'clock clockwise,
 * its star ticks cut across it, `ringGap` out from the 44 tile's edge and `ringWidth` wide. Decoration. */
function ringOf(ring) {
	const root = document.documentElement;
	const side = token(root, "--qp-layout-tile-result");
	const gap = token(root, "--qp-layout-ring-gap");
	const width = token(root, "--qp-layout-ring-width");
	const radius = token(root, "--qp-layout-tile-radius-small");
	const size = side + 2 * (gap + width);
	const d = ringPath(size, side / 2 + gap + width / 2, radius + gap + width / 2);
	const tick = token(root, "--qp-surface-ladder-tree-tick-fraction") * 1000;
	const ticks = ring.ticks ?? [];
	const fill = Math.max(0, Math.min(1000, ring.fill ?? 0));
	const done = styled(h(`${SVG}path`, {class: "qp-path-ring-fill", d, pathLength: "1000"}),
		{"stroke-dasharray": [fill, 1000].join(" ")});
	const cuts = styled(h(`${SVG}path`, {class: "qp-path-ring-ticks", d, pathLength: "1000"}),
		{"stroke-dasharray": ticks.length ? tickDashes(ticks, tick) : [0, 1000].join(" ")});
	return h(`${SVG}svg`, {class: "qp-path-ring", viewBox: `0 0 ${size} ${size}`, "aria-hidden": "true",
		focusable: "false"}, h(`${SVG}path`, {class: "qp-path-ring-track", d, pathLength: "1000"}), done, cuts);
}

/** A step's tile on the path: its move's tile in its quality where the player has it (an orange or gold step its climb
 * only opened once a set earns it), the ahead face on the next step and above it, a mastered step's seal, a step the
 * setup shuts dashed; on the player's step the ring round it and, where its climb has a path, its beads (slice 42: one
 * a stage, done filled, open hollow, waiting a dim dot). */
function pathTile(w, lad, st, kind) {
	const owned = kind === "yours" || kind === "kept" || kind === "top" || (kind === "now" && !st.state.earn);
	const box = h("span", {class: "qp-path-tile"}, tile(w, st.move, {size: 44,
		face: kind === "next" || kind === "ahead" ? "ahead" : "pose", quality: owned ? st.quality : null,
		...(st.mastered ? {mastered: ""} : {})}));
	box.toggleAttribute("data-shut", st.state.kind === "shut" || (st.start?.refused === "setup"
		&& st.start.ladder === lad.id && st.start.rung === st.rung));
	if ((kind === "now" || kind === "top") && lad.row.ring) box.append(ringOf(lad.row.ring));
	const stages = beadsOf(lad, st, kind);
	if (stages) {
		box.append(h("span", {class: "qp-path-beads", "aria-hidden": "true"}, ...stages.map((s) => h("i", {
			"data-state": s.state === "skipped" ? "done" : s.state}))));
	}
	return box;
}

/** One step of the path: its tile, its move, its key line (how it was reached, the player's step's goal, what opens a
 * step ahead: steps.js `stepTip`, the trees' words) with its days as pips; a check at the end of a step the player has
 * below theirs, or Share its unlock card from Rare up where it was unlocked on a day or kept; a step ahead dimmed. Read
 * as the trees read a step ("Push-up, Uncommon, step 5, your step, record 27 of 30"); the row opens the step's card
 * (`openStep`). */
function pathStep(w, book, ladder, lad, st, openStep) {
	const kind = kindOf(lad, st);
	const tip = stepTip(w, lad, st, kind);
	const under = [];
	if (tip.key) under.push(h("span", {class: "qp-type-meta", text: tip.key}));
	if (tip.pips?.need > 0) {
		under.push(h("qp-day-pips", {have: Math.min(tip.pips.have, tip.pips.need), need: tip.pips.need,
			"aria-hidden": "true"}));
	}
	const shares = st.quality >= 4 && (st.reached?.kind === "day" || st.reached?.kind === "kept")
		&& (kind === "yours" || kind === "now" || kind === "top");
	const end = kind === "yours" && !shares ? h("qp-art", {class: "qp-path-check", name: "mark_check", "aria-hidden": "true"})
		: h("span", {class: "qp-ladder-chevron", "aria-hidden": "true"});
	const shell = h("button", {type: "button", class: `qp-step-row${kind === "ahead" ? " qp-dim" : ""}`,
		"aria-label": tip.spoken, ...(kind === "now" || kind === "top" ? {"aria-current": "step"} : {}),
		"on:click": () => openStep(st.step, `step:${st.step}`), "data-focus-key": `step:${st.step}`},
	pathTile(w, lad, st, kind),
	h("div", {class: "qp-step-words"}, h("span", {class: "qp-type-body-strong", text: tip.name}), ...under), end);
	if (!shares) return shell;
	const share = h("qp-round-button", {icon: "share", small: true, class: "qp-step-share",
		"data-focus-key": `share:${st.step}`, "on:click": () => openShare({
			kind: "unlock", key: w.moves.get(st.move)?.key, name: tip.name, title: t("ladders.card.unlocked"),
			meta: footer(book, ladder), quality: st.quality, qualityWord: qualityWord(w, st.quality), step: st.step,
		})});
	return h("div", {class: "qp-step-shared"}, shell, share);
}

/** The ladder's path under the Steps label, one card (docs/ui.md "The Book's ladders and paths"): every step from its
 * first up, a QualityWord over each whose tier differs from the step before it (Starter … Artifact, a heading), each
 * step's row ([pathStep]) and under it the steps of other ladders it leads to (steps.js `linksOf`'s `out`: "Leads to
 * Handstand, step 1"), each opening that ladder's page at its step where it is open, else that step's card
 * (`openStep`). A ladder that waits whole: its one row of what opens it, its card. */
function pathCard(w, book, ladder, row, openStep) {
	const lad = ladderOf(w, book, row.ladder);
	if (!lad) return [];
	const items = [];
	if (row.waits) {
		const tip = stepTip(w, lad, null, "waiting");
		items.push(h("qp-row", {heading: tip.key, glyph: "mark_link", trailing: "nav", "aria-label": tip.spoken,
			"data-focus-key": "waits", "on:click": () => openStep(null, "waits")}));
	}
	let last = null;
	for (const st of row.waits ? [] : stepsUp(row)) {
		if (st.quality !== last) {
			items.push(h("div", {class: "qp-step-head", role: "heading", "aria-level": "3"},
				h("qp-quality-word", {quality: st.quality, word: qualityWord(w, st.quality)})));
		}
		items.push(pathStep(w, book, ladder, lad, st, openStep));
		for (const o of linksOf(w, row.ladder, st.step).out) {
			const into = book.ladders.find((l) => l.ladder === o.ladder);
			const open = into && !into.waits && !into.opens && into.rung > 0;
			items.push(h("qp-row", {heading: t("ladders.path.branch", {ladder: w.ladders.get(o.ladder)?.name ?? "",
				step: o.step}), glyph: "mark_arrow", trailing: "nav", "data-focus-key": `branch:${o.ladder}:${o.step}`,
			"on:click": () => (open ? push(LADDER, {ladder: o.ladder, step: o.step})
				: openStep(o.step, `branch:${o.ladder}:${o.step}`, true, o.ladder))}));
		}
		last = st.quality;
	}
	return [h("qp-section-label", {label: t("ladders.steps")}), h("qp-card", {rows: true, class: "qp-steps"}, ...items)];
}

/** What the hero card shows (its move, quality, step, record, goal, a pose's mark and days and whether only Perfect
 * counts them): a Book that moves none of it keeps the card, its figure playing on. */
function heroSignature(row) {
	return JSON.stringify([row.move, row.quality, row.step, row.record, row.seconds, row.next, row.mark, row.stars,
		row.hold, row.opens ?? null, climbAsk(row)]);
}

/** The ask of the climb from the player's step (Perfect alone below a risky step, ns.MarkNeeds); none off a pose
 * climb. */
function climbAsk(row) {
	const now = row.steps.find((s) => s.state.kind === "now")?.rung ?? row.rung;
	return row.steps.find((s) => s.rung === now + 1)?.state.climb?.ask ?? null;
}

/** The hero card: the figure, the move's name, its quality line, the record and the goal, a pose's mark with its days
 * (Perfect ones where only Perfect counts: the climb's `ask`). */
function hero(w, ladder, row) {
	const m = w.moves.get(row.move);
	const value = ladderValue(row, ladder);
	const numbers = value ? h("div", {class: "qp-hero-numbers"}, valueCell(value, true)) : null;
	if (value && row.record > 0 && ladder.dose !== "pose" && row.next) {
		numbers.append(h("div", {class: "qp-book-value qp-leading"}, h("span", {class: "qp-type-caps", text: t("kit.goal")}),
			h("span", {class: "qp-type-num-medium", text: goal(row.next, row.seconds)})));
	}
	const parts = [h("qp-figure-stage", {move: m?.key ?? "", pausable: true, label: m?.name ?? ""}),
		h("h2", {class: "qp-type-display-small qp-centred", text: m?.name ?? ""}),
		qualityLine(w, row.quality, row.step), numbers];
	if (row.mark) {
		const perfect = climbAsk(row) === PERFECT;
		parts.push(h("div", {class: "qp-mark", role: "group"}, h("span", {class: "qp-type-caps", text: t("ladders.mark")}),
			h("p", {class: "qp-type-body", text: row.mark}),
			h("span", {class: "qp-type-meta", text: t(perfect ? "ladders.mark.days.perfect" : "ladders.mark.days",
				{days: row.stars, total: w.words.poseDays})})));
	}
	return h("qp-card", {class: "qp-book-hero"}, ...parts);
}

/** Pro's charts: the rung as a stair, and the record of the move standing for the rung with its counts. */
function charts(w, ladder, row, chart) {
	const points = chart.points;
	const rungs = points.filter((p, i) => i === 0 || points[i - 1].rung !== p.rung).map((p) => ({at: p.at,
		value: p.rung}));
	const levels = ladder.qualities.map((quality, i) => ({value: i + 1, quality,
		word: t("ladders.chart.rung.word", {rung: i + 1, quality: qualityWord(w, quality)}),
		label: t("ladders.chart.rung.label", {rung: i + 1, quality: qualityWord(w, quality)})}));
	const parts = [h("qp-section-label", {label: t("pro.title")}),
		h("qp-card", {class: "qp-charts"},
			chartCard(t("ladders.chart.rung.title"),
				stairChart(rungs, levels, chart.now, t("ladders.chart.rung", {ladder: ladder.name}))))];
	if (row.move && ladder.dose !== "cardio") {
		const record = points.filter((p) => p.move === row.move && p.record > 0).map((p) => ({at: p.at, value: p.record}));
		const counts = chart.counts.map(([at, value]) => ({at, value}));
		const title = t("ladders.chart.record.title", {move: moveName(w, row.move)});
		const plot = recordChart(record, counts, chart.rises, row.seconds, chart.now, title,
			(v) => (row.seconds ? clock(v) : number(v)));
		parts[1].append(chartCard(title, plot, [{label: t("ladders.record.label"), colour: "var(--qp-surface-chart-mark)"},
			{label: t("ladders.chart.counts"), colour: "var(--qp-color-text)"}, ...(chart.rises.length ? [{star: true}] : [])]));
	}
	return parts;
}

/** The last card of a strength or pose ladder's page: Start this ladder over, behind its confirmation (Rust refuses it
 * while a pump runs). */
function restartCard(ladder) {
	return h("qp-card", {rows: true}, h("qp-row", {heading: t("ladders.detail.restart"), glyph: "mark_close",
		trailing: "action", destructive: true, "data-focus-key": "restart", "on:click": () => confirmWindow(
			t("ladders.detail.restart.title", {ladder: ladder.name}), t("ladders.detail.restart.body"),
			t("ladders.detail.restart.confirm"), () => intent("book.restart", {ladder: ladder.id}).catch(console.error))}));
}

register(LADDER, ({ladder: id, step: asked = null}) => {
	const view = page({heading: ""});
	const foot = h("div", {class: "qp-book-foot"});
	view.el.append(foot);
	let w = null;
	let book = null;
	let returning = null; // the row whose step's card started a move here (its key), until its pump has gone
	let heroCard = null; // the hero shown and what it shows, kept while a Book moves none of it
	let heroSig = null;
	let scrolled = asked == null; // the step asked (an Up next row) brought into view once
	/** A step's card over the page (of this ladder, or of `ladder` a branch row names), its Start returning to this
	 * page's row `row`; a start that goes nowhere opens the card again. A card the player opened by its row (`counted`)
	 * grows out of the row's tile and shrinks back into it (slice 46); one a start that went nowhere opens never grows. */
	const openStep = (step, row, counted = true, ladder = id) => {
		if (!w) return;
		const origin = counted ? () => view.body.querySelector(`[data-focus-key="${row}"]`) : null;
		openCardWindow({w, place: "ladder", ladder, step, go: goLink, counted, origin,
			start: (target, from) => {
				returning = row;
				startMove(target, from, () => {
					if (returning !== row) return;
					returning = null;
					openStep(step, row, false, ladder);
				});
			}});
	};
	intent("book.ladder", {ladder: id}).catch(console.error);
	// the trees in use today (`feature_day`'s `tree`), as a phone counts its paths
	intent("usage.seen", {feature: "tree"}).catch(console.error);
	view.stops.push(() => intent("book.ladder", {ladder: null}).catch(console.error));
	const draw = () => {
		if (!w || !book) return;
		const ladder = w.ladders.get(id);
		const row = book.ladders.find((l) => l.ladder === id && !l.opens);
		view.bar.heading = ladder?.name ?? "";
		if (!book.ready) {
			view.body.replaceChildren(loading());
			foot.replaceChildren();
			heroCard = null;
			return;
		}
		if (!ladder || !row) {
			view.body.replaceChildren();
			foot.replaceChildren();
			heroCard = null;
			return;
		}
		const sig = heroSignature(row);
		if (sig !== heroSig || !heroCard) {
			heroCard = hero(w, ladder, row);
			heroSig = sig;
		}
		const parts = [...pathCard(w, book, ladder, row, openStep), ...roadCard(w, row, book.pro)];
		if (book.pro && book.chart?.ladder === id) parts.push(...charts(w, ladder, row, book.chart));
		// a strength or hold ladder starts over (its dose is reps or seconds); never cardio or a pose family
		if (ladder.dose === "rep" || ladder.dose === "hold") parts.push(restartCard(ladder));
		// the hero kept in place (moving it would restart its figure), the rest after it drawn again
		if (view.body.firstElementChild === heroCard) {
			while (heroCard.nextSibling) heroCard.nextSibling.remove();
			view.body.append(...parts);
		} else {
			view.body.replaceChildren(heroCard, ...parts);
		}
		const shares = row.record > 0 && (ladder.dose === "rep" || ladder.dose === "hold");
		foot.replaceChildren(...(shares ? [h("qp-primary-button", {label: t("ladders.share"), glyph: "mark_share",
			"data-focus-key": "share:record",
			"on:click": () => openShare({kind: "record", key: w.moves.get(row.move)?.key, name: moveName(w, row.move),
				title: moveName(w, row.move), meta: footer(book, ladder), quality: row.quality,
				qualityWord: qualityWord(w, row.quality), step: row.step, record: row.record, seconds: row.seconds})})]
			: []));
	};
	watch(view, "book.words", (words) => {
		w = lookup(words);
		keepFocus(view.el, draw);
	});
	watch(view, "book", (b) => {
		book = b;
		keepFocus(view.el, draw);
		// the step asked, once its row stands
		if (scrolled) return;
		const row = view.body.querySelector(`[data-focus-key="step:${asked}"]`);
		if (!row) return;
		scrolled = true;
		row.scrollIntoView({block: "center"});
	});
	// a pump on screen that is no move started from this page (one the engine refused composed none) keeps no return
	watch(view, "pump", (p) => {
		const from = p.pump?.what === "move" ? p.pump.from : null;
		if (p.pump && (from?.place !== "ladder" || from.ladder !== id)) returning = null;
	});
	view.primary = () => foot.querySelector("qp-primary-button");
	// the pump's cover has gone: a move started from a step's card here comes back to the row it opened from
	view.resume = () => {
		const back = returning;
		returning = null;
		return back ? view.body.querySelector(`[data-focus-key="${back}"]`) : null;
	};
	return view;
});

