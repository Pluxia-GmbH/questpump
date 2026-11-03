// A step's card on the computer (slice 36's design, built in slice 38; docs/ui.md, "The step's card"; the phones'
// StepCard, ladders.step_card): its header the step's move in its quality's colour with its QualityWord (a ladder that
// waits whole too, its note under it: "Not open yet", the setup's or the list's); on a step above the player's whose
// own move its Start does not run (the next step, a step ahead, a family not open yet) the ahead line ("Above your
// step for now") with the move's How to and
// its stop line; the figure on every step, pausable; on an owned step how it was reached; on a step the player has its
// mastery after its groups (slice 42: when it was mastered, or under Master this step how many of its moves reached
// their top with its pips, 2026-10-29); the step's measure, on every step: a strength or hold step's record and range
// and the player's step's stars as the cells of a still StatStrip (2026-10-29: "Record 22 · Range 20-30" as one line
// until then), a counted cardio step's clean days of 30 s all out with the shown form's best clean count, a timed one's
// Pace with its full moves under it off the player's step and the ladder's top, slice 52, each once a card, and the goal
// pose a climb line names; on a ladder of the Stamina Ledger's holds the player's step's Now row; a ladder back from a
// break its placing line; on a step above the player's what reaching it takes (the climb or the Path's need, with its
// count and DayPips and its path's stages as rows) or what holds it (the setup, the list, a pain flag); then the step's
// moves in three groups (slice 42, "The steps' order"): Your main move, To step up on the player's step (what opens the
// next step, its path's stages, the needs and the checks the climb waits for, 2026-10-29), Then, for the next step on a
// step above it (its path's stages in order, each in its state: done, open, waiting with what opens it, skipped with
// why), For variety (the other moves, folded past six under one row with no label over it); each step of another
// ladder the climb waits for with a check or a dash and, while it waits,
// its waits line (the engine's ns.NeedDone and ns.NeedAt, `needs`; no mark where the engine did not answer), an `any`
// group as one row, each such step a button to its card, which takes this one's place; each stretch check the climb or a stage waits for as a check row with "Do the check" (slice 42);
// the Helps row where the step's move names a pose that helps it; the one Start (a StartBlock: its label and lines by
// ns.StartFor, greyed with its reason by the pump's gate, in its block with the tile of the move it trains wherever
// that is not the move the card shows, "Train with" on the player's own step and "To get here" elsewhere; a refused
// one's lines and their link rows), and beside it on the player's own cardio step the Long Haul (slice 55); a timed
// cardio climb from the player's step reads as its Full moves row with its DayPips and, with Pro, its date at the row's
// end, as a hold climb's line ends in it (slice 52); then, quieter, the steps it opens on other ladders as buttons, the
// ladder's top (but on its top step), the weight line, what one more piece of the setup opens there, and the Ladder
// page row. Every group's move shows in place in the card with its own Start and Back to the step, never a modal
// window, which would take the focus the card closes on. Every word comes from views/book/steps.js over the Book's own
// fields. A ladder's page and the list show it in the kit's Window from the foot, which a click on its dim closes
// (beside its step on a talent tree's board until 2026-10-29, when the ladders' paths took the trees' place). Opened
// from a tapped step (a ladder page's path step, an Up next or a list row) or a Moves tile, the card's Window and the move's window grow out of it and shrink back into it as the
// player closes them (`origin`, slice 46; lib/grow.js).

import {get, intent, subscribe} from "../../lib/bridge.js";
import {t} from "../../lib/copy.js";
import {keepFocus} from "../../lib/focus.js";
import {openMenu, push} from "../../lib/router.js";
import {h, qualityName} from "../../kit/element.js";
import {howAndTips, markLine, stopLine} from "../../kit/move-how.js";
import {lifted} from "../pump/words.js";
import {
	MOVE, SETUP, aboveReach, amount, factsOf, follow, guideOf, moveName, moveValue, qualityWord, recordOf, spanWords,
	switches, tileQuality, valueWords,
} from "./common.js";
import {
	checkWords, chipWords, climbWords, closedLine, heldBy, kindOf, ladderOf, linksOf, masteryLine, needLine, needOf,
	needWords, opensLine, placingLine, reachedWords, refusalOf, showsCheck, stageLine, stageOpens, stageSkip, stageSpoken,
	startLabel, startLines, stepAt, unplaced, waitingNote, waitingStart, windowOf,
} from "./steps.js";

/** The For variety group's moves shown at once; past them it folds under its label row (slice 42), which opens it. */
const SAME_SHOWN = 6;
/** The Long Haul's button's focus key (slice 55). */
const HAUL_KEY = "card:haul";

/** A line of the card's body text. */
function body(text) {
	return h("p", {class: "qp-type-body", text});
}

/** A row of the card's measure or climb (slice 52, the game's Tree.CardPair): its label, its value at the row's end
 * (Pace 38 of 45 s, Full moves 1 of 2, Now 2:44-3:45) and after it what else ends the row (the DayPips, Pro's date). */
function pair(label, value, ...more) {
	return h("p", {class: "qp-type-body qp-card-pair"}, h("span", {text: label}),
		h("span", {class: "qp-card-pair-value"}, value, ...more));
}

/** Pro's date for the climb from the player's step (slice 52, the Book row's `forecast`: ns.HoldTopIn's on a hold
 * ladder, ns.SteadyForecast's on the cardio ladder, a counted step's once its path is in, as the Path says a span),
 * "In 4–16 days" at the end of its row or line; none without Pro or while the engine names none. */
function forecastOf(book, lad) {
	if (!book.pro || !lad.row.forecast) return null;
	return h("span", {class: "qp-type-meta qp-card-forecast", text: t("ladders.card.forecast",
		{span: spanWords(lad.row.forecast)})});
}

/** A still StatStrip of the card's measure (2026-10-29) holding `cells` ({value, name} or {stars, name}; a null one
 * left out), none where none stands. */
function statStrip(cells) {
	const shown = cells.filter(Boolean);
	if (!shown.length) return [];
	const strip = h("qp-stat-strip", {still: true});
	strip.cells = shown;
	return [strip];
}

/** The step's own measure under its figure, on every step: a strength or hold step's record and its window's range as
 * the cells of a still StatStrip (its range alone before the first), the player's step's stars its third cell (alone on
 * another step's measure); a counted cardio step's clean days of 30 s all out with the best clean count of the form it shows
 * (the ring's on the player's step, else the best of its forms), a timed cardio step's Pace (slice 52, the Book's
 * `pace` of the full move's `full`, ns.SteadyPace; the ladder's top's too, which is timed) with its full moves under it
 * on a step that is not the player's nor the ladder's top, which no climb leaves (a counted rung's too where the setup
 * shuts every form, which climbs by them: the Book's `clean`); a pose step's goal pose, each once a card (`said`, what
 * the climb's or the Path's line on the card says already: the clean days or full moves, the goal pose); on a ladder
 * of the Stamina Ledger's holds the player's step adds its Now row (slice 52, the Book row's `now`, ns.HoldNow: a
 * hold's as a clock, power's in reps, none before that move's first record); a ladder back from a break says what finds
 * the step again (ns.Placing, `placingLine`). No record or range on the player's step of a ladder not placed yet, whose
 * climb's line alone says its first max set places it (as the game's card and both phones'). */
function measure(w, book, lad, st, kind, said) {
	const m = w.moves.get(st.move);
	const own = kind === "now" || kind === "top";
	const stars = own && st.state.stars != null
		? {name: t("ladders.card.stat.stars"), stars: {stars: st.state.stars, total: w.words.stars}} : null;
	if (own && lad.row.placing) return [...statStrip([stars]), body(placingLine(lad))];
	if (!m || (kind === "now" && unplaced(lad))) return statStrip([stars]);
	const facts = factsOf(book);
	const ring = own ? lad.row.ring : null;
	if (lad.words.dose === "pose") {
		return [...statStrip([stars]), ...(m.mark && m.mark !== said.mark
			? [h("span", {class: "qp-type-caps", text: t("ladders.mark")}), body(m.mark)] : [])];
	}
	if (lad.words.dose === "cardio") {
		if (st.clean != null) {
			const parts = said.clean ? [] : [body(t("ladders.card.clean", {count: st.clean,
				seconds: w.words.countSeconds}))];
			const best = ring?.move != null ? ring.record ?? 0
				: Math.max(0, ...formsOf(w, lad, st).map((x) => recordOf(book, facts, x.move) ?? 0));
			if (best > 0) parts.push(body(t("ladders.card.best", {count: best})));
			return [...statStrip([stars]), ...parts];
		}
		const parts = st.pace != null
			? [pair(t("ladders.card.pace"), t("ladders.card.pace.value", {seconds: st.pace, full: st.full}))] : [];
		if (!own && !said.full && st.rung < lad.top) {
			parts.push(body(t("ladders.path.need.full", {need: w.words.fullDays})));
		}
		return [...statStrip([stars]), ...parts];
	}
	const win = windowOf(w, lad, st.rung, st.move);
	const hold = m.dose === "hold";
	const record = (ring?.move === st.move ? ring.record : null) || recordOf(book, facts, st.move);
	const now = win && own && lad.row.now ? [pair(t("ladders.card.now"), t("ladders.card.now.value",
		{low: amount(lad.row.now[0], hold), high: amount(lad.row.now[2], hold)}))] : [];
	return [...statStrip([
		win && record ? {name: t("ladders.record.label"), value: amount(record, hold)} : null,
		win ? {name: t("ladders.card.stat.range"), value: t("ladders.card.now.value", {low: amount(win.lo, hold),
			high: amount(win.hi, hold)})} : null,
		stars,
	]), ...now];
}

/** The countable forms of a cardio ladder's step the setup allows, in the words' order (road order, slice 54): the
 * Book's `forms` (ns.CountForms with the player's filter, the forms its rung climbs on, as the game's card reads them);
 * none on a timed rung and none where the setup shuts every form, whose rung climbs by full moves. */
function formsOf(w, lad, st) {
	const allowed = new Set(st.forms ?? []);
	return w.words.moves.filter((x) => x.ladder === lad.id && x.rung === st.rung && x.count && allowed.has(x.move));
}

/** A line with a check where what it names is in, a dash where it is not, and that state in words for a screen
 * reader (`spoken`: done, not done by default), since the marks are pictures; neither where it is not known (`done`
 * null: the engine did not answer); `meta` a line under it (or a list of lines, each under the one before). With `act`
 * the line is a button (a step of another ladder: its card), its key `key`. */
function checked(text, done, meta = null, act = null, key = null, spoken = null) {
	const known = done != null;
	const parts = [known ? h("qp-art", {name: done ? "mark_check" : "mark_dash", "aria-hidden": "true"}) : "",
		h("div", {class: "qp-card-check-words"}, h("span", {class: "qp-type-meta", text}),
			...[meta].flat().filter(Boolean).map((line) => h("span", {class: "qp-type-meta", text: line})),
			known ? h("span", {class: "qp-unseen", text: spoken ?? t(done ? "kit.objective.state.done"
				: "kit.objective.state.missed")}) : "")];
	const cls = ["qp-card-check", ...(done ? ["qp-done"] : []), ...(act ? ["qp-card-link"] : [])].join(" ");
	if (!act) return h("div", {class: cls}, ...parts);
	return h("button", {type: "button", class: cls, "data-focus-key": key, "on:click": act}, ...parts);
}

/** A move's How and Tips on a step's card, as the phones' StepCard draws them (MoveGuide, 2026-10-24; the game's card,
 * Tree.lua `CardMore`, the steps and the advice): How's setup line and numbered steps for the gear that carries a
 * loaded move's load (`guideOf`), Tips with the move's own advice last (moves.json's `hint`), a pose's goal pose
 * (`mark`, where no measure shows it) and the stop line. */
function howTo(w, book, move, mark = true) {
	const m = w.moves.get(move);
	return h("div", {class: "qp-card-how"}, ...howAndTips({...guideOf(m, factsOf(book).get(move)), tips: m?.tips}),
		mark ? markLine(m?.mark) : null, stopLine(m?.stop));
}

/** The rung whose climb or Path a card on step `st` says (the step above the player's from the player's own, the step
 * itself from the next, a step ahead or a kept one); none on the others. */
function targetOf(lad, st, kind) {
	if (kind === "now") return lad.cur + 1;
	return kind === "next" || kind === "ahead" || kind === "kept" ? st.rung : null;
}

/** A group's caps label. */
function caps(text) {
	return h("span", {class: "qp-type-caps", text});
}

/** What opens the next step from the player's (its climb, with its pips), or what reaching a step above it takes (the
 * Path's need); on the player's step, and on a step further up, what holds a climb that cannot move (the setup, the
 * player's own list, a pain flag: the hover's words); the top's line; nothing more on a ladder back from a break, whose
 * measure says what finds the step again. Returns `reached` (an owned step's line), `climb` (the climb's or the Path's
 * line with its pips), `stages` (that climb's path, the card's own Then
 * group on the player's step, rows of their own on a step above it), the steps of other ladders that climb waits for
 * (`into`, linksOf's, none where the step's own Start is stopped), the step they stand into (`up`: the climb's target
 * step, which a check row opens) and what a line of them says already (`said`: the clean days or full moves, the goal
 * pose). The climb from the player's step: a counted cardio one's proving window once its days and path are in
 * (slice 53), a timed one's Full moves row, and with Pro its date at the end (slice 52). */
function opensNext(ctx, lad, st, kind) {
	const {w, book} = ctx;
	const said = {clean: false, full: false, mark: null};
	const out = {reached: [], climb: [], stages: null, into: [], up: null, said};
	const cur = lad.cur;
	if (kind === "top") {
		out.climb.push(body(t("ladders.path.top")));
		return out;
	}
	if (kind === "now" && lad.row.placing) return out;
	if (kind === "yours") out.reached.push(h("p", {class: "qp-type-meta", text: reachedWords(st.reached)}));
	// the climb into the step above the player's, or the Path's need for a step ahead
	const target = targetOf(lad, st, kind);
	if (target == null) return out;
	let words = null;
	let shut = false;
	// Pro's date stands only at the end of an open climb from the player's step, never beside what holds it
	let open = false;
	// a step whose own Start the setup or the player's list refuses: its refusal names the rule, so the road to it
	// says nothing more (the game's `stopped`, Tree.CardStep)
	const stopped = kind !== "now" && (st.start?.refused === "setup" || st.start?.refused === "off");
	if (target === cur + 1) {
		const next = stepAt(lad, target);
		open = next?.state.kind === "next";
		if (open) words = climbWords(w, next.state.climb, book, lad.row.proving ?? null);
		// the setup's rule, the move left out or the pain flag that holds it, as the hover says it, on the player's
		// step's card and on that step's own (a step of another ladder it waits for says it in its own row below)
		else if (next && ["shut", "off", "pain"].includes(next.state.kind) && !stopped) {
			words = {line: closedLine(w, next)?.text ?? null};
		}
	} else {
		const path = stopped ? null : lad.row.path?.steps.find((p) => p.rung === target);
		// a climb the setup, the player's own list or another ladder's step shuts says what holds it as the hover does
		if (path?.shut) words = {line: closedLine(w, {state: path.shut, step: path.step})?.text ?? null};
		else if (path) words = needWords(w, path);
		shut = path?.shut?.kind === "waits" || path?.shut?.kind === "waitsStep";
	}
	// each step of another ladder the climb waits for, read from the links into that step, as the engine reads it
	// (on a far step whose way the setup or the player's list stops, none: its refusal says it, as the game's road)
	const up = stopped && target !== cur + 1 ? null : stepAt(lad, target);
	out.up = up;
	out.into = up ? linksOf(w, lad.id, up.step, book).into : [];
	// a climb shut by such a step says it in that step's own row, never in a line above them too
	if (words?.line && !(shut && out.into.some((need) => needLine(w, need)))) {
		const days = words.days;
		// its day pips at the end of the line they count (none on a Path step, whose days are a number)
		const pips = days && typeof days === "object" && days.need > 0
			? h("qp-day-pips", {have: days.have, need: days.need}) : null;
		const forecast = open ? forecastOf(book, lad) : null;
		// its days as a count with its pips at the line's end (2026-10-29: the pips alone until then), which the count
		// reads
		if (pips) {
			pips.setAttribute("aria-hidden", "true");
			// under To step up (the player's step) the climb's row words, `up`
			const label = words.full ? t("ladders.card.full") : (kind === "now" && words.up) || words.line;
			out.climb.push(pair(label, t("ladders.card.full.value",
				{have: Math.min(days.have, days.need), need: days.need}), pips, forecast));
		} else {
			out.climb.push(h("p", {class: "qp-type-body qp-card-climb"}, words.line, forecast));
		}
		// what a pose climb's days still lack (slice 47: the days on the step's main pose, the Perfect day)
		if (words.short) out.climb.push(h("p", {class: "qp-type-meta", text: words.short}));
		out.said.clean = Boolean(words.clean);
		out.said.full = Boolean(words.full);
		if (words.mark) out.said.mark = w.moves.get(stepAt(lad, cur + 1)?.state.climb?.move)?.mark ?? null;
	}
	out.stages = words?.stages?.length ? words.stages : null;
	return out;
}

/** A move of the card's step as its group shows it (slice 42): its tile (44: its key pose in its quality where
 * trained, the ahead face above the player's reach or while it waits, the off face where the setup or the player's
 * list leaves it out, its topped mark once it reached its top once), a button that shows the move in place in the card
 * (`ctx.show`). */
function groupTile(ctx, move, waiting = false) {
	const {w, book} = ctx;
	const m = w.moves.get(move);
	const fact = factsOf(book).get(move);
	const el = h("qp-move-tile", {move: m?.key ?? "", size: "44", action: true, name: m?.name ?? "",
		quality: fact?.trained ? fact.quality ?? null : null, word: fact?.quality ? qualityWord(w, fact.quality) : null,
		step: fact?.step ?? 0, face: waiting || aboveReach(w, book, fact, move) ? "ahead" : "pose",
		off: Boolean(fact?.off || fact?.gap), topped: Boolean(fact?.topped), "data-focus-key": `card:move:${move}`});
	el.addEventListener("click", () => ctx.show(move));
	return el;
}

/** A stretch check's row (slice 42; a rung's need or a move's check the engine reads as one, `state.check`): the
 * pose's tile, "{move}: Good or better, once" (Perfect where it asks it), a dash and, while undone, what one hold at
 * its goal pose opens (`opens`: the move waiting on it, or the climb to its step) and its button "Do the check" (none
 * where the card's own Start is that same check, which then reads "Do the check"), or done its check and how it
 * passed. The button takes the card's Start gate (2026-10-19, the program's double-check: `heldBy`, greyed with its
 * note while felt unwell locks the pumps or the game asks them, busy while the engine folds); where the check's own Start
 * refuses (the Book's `start` of its row, ns.StartFor of the check: the setup, a pain flag) its refusal's lines stand
 * in the button's place, named on the check's own ladder as the phones name it. `entry` the need as the library names
 * it (an `any` group's choice the row shows). */
function checkRow(ctx, entry, state, opens) {
	const {w} = ctx;
	const words = checkWords(w, entry, state, opens);
	const pose = state.move ?? entry.move;
	const own = ctx.ownStart?.as === "check" && ctx.ownStart.move === pose;
	const live = !state.done && !own && Boolean(ctx.check);
	const refused = live && state.start?.refused ? refusalOf(w, {id: entry.ladder}, state.start).lines : [];
	const held = live && !refused.length ? heldBy(ctx.pump) : null;
	const row = h("div", {class: "qp-card-check-row"}, pose != null ? groupTile(ctx, pose) : "",
		checked(words.need, Boolean(state.done), [words.line, ...refused, held?.note]));
	if (live && !refused.length) {
		const key = `card:check:${entry.ladder}:${entry.step}:${pose ?? ""}`;
		row.append(h("qp-secondary-button", {label: t("ladders.start.check"), "data-focus-key": key,
			disabled: held?.state === "disabled", busy: held?.state === "busy",
			"on:click": () => ctx.check(needOf(entry), opens)}));
	}
	return row;
}

/** A move's own checks as rows under the row that names it (slice 42: library `checks`, the Book's move `checks` as
 * the engine reads them): a stretch check its check row (an `any` group's by the choice the setup offers, the engine's
 * `choice`), a record its need row with its chip and waits line; none before the engine answered. */
function moveChecks(ctx, move) {
	const {w, book} = ctx;
	const states = factsOf(book).get(move)?.checks ?? [];
	const entries = w.moves.get(move)?.checks ?? [];
	return entries.map((entry, i) => {
		const state = states[i];
		if (!state) return null;
		if (state.check != null) return checkRow(ctx, entry.any?.[state.choice - 1] ?? entry, state, {move});
		const first = entry.any?.find((x) => x.ladder === w.moves.get(state.move)?.ladder) ?? entry.any?.[0] ?? entry;
		return needRow(ctx, [{ladder: first.ladder, step: first.step, need: state}], `card:need:${move}:${i}`);
	}).filter(Boolean);
}

/** One step of another ladder the climb waits for (an `any` group's steps joined, "{first}, or {second}": done when one
 * is), named by its chip, with a check or a dash and its waits line while undone (`needLine`), a button to that step's
 * card in this one's place where it stands on a rung (`ctx.select`; an `any` group's the step the setup offers). */
function needRow(ctx, group, key) {
	const {w, book} = ctx;
	const words = group.map((x) => chipWords(w, book, x.ladder, x.step))
		.reduce((first, second) => t("ladders.card.need.or", {first, second}));
	const need = group[0].need;
	const pick = group.find((x) => x.ladder === w.moves.get(need?.move)?.ladder) ?? group[0];
	const onRung = w.ladders.get(pick.ladder)?.steps.includes(pick.step);
	const act = onRung ? () => ctx.select(pick.ladder, pick.step) : null;
	return checked(words, need?.done ?? null, needLine(w, pick), act, key);
}

/** What a climb's stretch check opens (`up`, the step of ladder `ladder` it climbs into): that step's move and its step
 * on the one scale, which its row's line names, and the climb's ladder and rung, by which the engine puts that check
 * first among the pose's (slice 42's fix round, 2026-10-17). */
function climbOpens(ladder, up) {
	return up ? {move: up.move, step: up.step, ladder, rung: up.rung} : null;
}

/** The steps of other ladders a climb of ladder `ladder` waits for (`into`, linksOf's), as rows: each stretch check a
 * check row under the caps label "Stretch check" (opening the climb to `up`; an `any` group's by the choice the setup
 * offers, `showsCheck`, as the game's Tree.CardWaits), each other step its need row, an `any` group one row. */
function climbNeeds(ctx, ladder, into, up) {
	const groups = new Map();
	for (const x of into) groups.set(x.group, [...(groups.get(x.group) ?? []), x]);
	const needs = [];
	const checks = [];
	for (const group of groups.values()) {
		const [x] = group;
		const shown = group.find(showsCheck);
		if (shown) {
			checks.push(checkRow(ctx, shown.entry, shown.need, climbOpens(ladder, up)));
		} else {
			needs.push(needRow(ctx, group, `card:need:${x.ladder}:${x.step}`));
		}
	}
	return {needs, checks: checks.length ? [caps(t("ladders.card.check")), ...checks] : []};
}

/** A family not open yet's stretch checks (slice 42's fix round, 2026-10-17: the planche's wrist lean, as the game's
 * card and the phones' `familyChecks` show them): each check of its opening (the links of its words with `opens`,
 * into its first step, as the engine reads them; an `any` group's by the choice the setup offers, `showsCheck`) a check
 * row under "Stretch check", its "Do the check" opening the family, whose first step's move its line names; none where
 * its opening names no check. */
function familyChecks(ctx, lad) {
	const first = cardStep(lad, null);
	if (!first) return [];
	const opens = {move: first.move, family: lad.id};
	const rows = linksOf(ctx.w, lad.id, first.step, ctx.book).into
		.filter((x) => x.link.opens && showsCheck(x))
		.map((x) => checkRow(ctx, x.entry, x.need, opens));
	return rows.length ? [h("div", {class: "qp-card-next"}, caps(t("ladders.card.check")), ...rows)] : [];
}

/** A stage of a step's path as its row in the card's Then group (slice 42): its moves' tiles (both where the setup
 * offers two), what it takes (`stageLine`, which joins them) and its state: done (a check), open (its DayPips),
 * waiting (the tiles greyed and what opens it), skipped (dimmed with a dash and why, with a row to Your setup or to the
 * move's Include in pumps); its state in words for a screen reader; under it the checks its moves wait for. */
function stageRow(ctx, stage, i) {
	const {w, book} = ctx;
	const facts = factsOf(book);
	const waiting = stage.state === "waiting";
	const tiles = h("div", {class: "qp-card-stage-tiles"}, ...stage.ids.map((id) => groupTile(ctx, id, waiting)));
	const meta = stage.state === "waiting" ? stageOpens(w, stage) : stage.state === "skipped" ? stageSkip(stage)
		: null;
	const done = stage.state === "done" ? true : stage.state === "skipped" ? null : false;
	const words = h("div", {class: "qp-card-stage-words"},
		h("span", {class: "qp-type-body", text: stageLine(w, stage)}),
		meta ? h("span", {class: "qp-type-meta", text: meta}) : "",
		stage.state === "open" && stage.days > 0 ? h("qp-day-pips", {have: stage.have ?? 0, need: stage.days}) : "",
		h("span", {class: "qp-unseen", text: stageSpoken(stage)}));
	const mark = stage.state === "done" ? h("qp-art", {name: "mark_check", "aria-hidden": "true"})
		: stage.state === "skipped" ? h("qp-art", {name: "mark_dash", "aria-hidden": "true"}) : "";
	const row = h("div", {class: "qp-card-stage", role: "group", "data-state": stage.state,
		"data-focus-key": `card:stage:${i}`}, tiles, words, mark);
	const parts = [row];
	// a stage skipped by the setup or the player's list opens what to change
	if (stage.state === "skipped" && ctx.go) {
		const name = moveName(w, stage.move);
		const link = stage.skip === "off"
			? {heading: t("ladders.refuse.include", {move: name}), glyph: "action_pump", kind: "include",
				move: stage.move}
			: {heading: t("menu.setup"), glyph: "action_character", kind: "setup",
				focus: facts.get(stage.move)?.gap?.key};
		parts.push(h("qp-row", {heading: link.heading, glyph: link.glyph, trailing: "nav",
			"on:click": () => ctx.go(link)}));
	}
	for (const id of stage.ids) if (done === false) parts.push(...moveChecks(ctx, id));
	return parts;
}

/** A stage of the climb that leads into a step above the player's, as one line with its mark (slice 42; the climb's
 * proof row until then): what it takes, and what opens it or why it is skipped. */
function roadRow(w, stage) {
	const meta = stage.state === "waiting" ? stageOpens(w, stage) : stageSkip(stage);
	return checked(stageLine(w, stage), stage.state === "skipped" ? null : stage.state === "done", meta, null, null,
		stageSpoken(stage));
}

/** The step's mastery (slice 42, a reward, never a gate): "Mastered 16 Oct", or under the caps label Master this step
 * its progress row, "1 of 2 moves at their top" with its pips (2026-10-29); none where the Book gives neither. */
function masteryOf(w, st) {
	const line = masteryLine(w, st);
	if (!line) return [];
	if (st.mastered) return [h("p", {class: "qp-type-meta", text: line})];
	const {have, need} = st.mastery;
	return [h("div", {class: "qp-card-group"}, caps(t("ladders.card.group.master")),
		pair(line, h("qp-day-pips", {have: Math.min(have, need), need, "aria-hidden": "true"})))];
}

/** The step's moves in its groups (slice 42, "The steps' order"; every move of the step's rung in exactly one): Your
 * main move (the move standing for the step, "{move} stands in for {anchor} with your setup." where that is not the
 * rung's anchor, and on the top the top's line, `climb`); on the player's step (`kind` now) To step up (2026-10-29: what
 * opens the next step, `climb`, its path's stages as rows and the steps of other ladders and the stretch checks the
 * climb waits for, `needs`; none where none of them stands), else Then, for the next step (the step's path's stages as
 * rows, none for a step whose climb has no path); its mastery (`masteryOf`); For variety (the rest, folded past six
 * under one row, "For variety · 9 moves", with no caps label over it, which the row names already, 2026-10-29). */
function groups(ctx, lad, st, kind, climb, needs) {
	const {w} = ctx;
	const parts = [];
	const main = [h("div", {class: "qp-card-main"}, groupTile(ctx, st.move),
		h("span", {class: "qp-type-body-strong", text: moveName(w, st.move)}))];
	if (st.anchor != null && st.anchor !== st.move) {
		main.push(h("p", {class: "qp-type-meta", text: t("ladders.card.main.standin", {move: moveName(w, st.move),
			anchor: moveName(w, st.anchor)})}));
	}
	const up = kind === "now";
	parts.push(h("div", {class: "qp-card-group"}, caps(t("ladders.card.group.main")), ...main, ...(up ? [] : climb),
		...moveChecks(ctx, st.move)));
	const stages = (st.stages ?? []).flatMap((stage, i) => stageRow(ctx, stage, i));
	if (up) {
		const rows = [...climb, ...stages, ...needs.needs, ...needs.checks];
		if (rows.length) parts.push(h("div", {class: "qp-card-group"}, caps(t("ladders.card.group.up")), ...rows));
	} else if (stages.length) {
		parts.push(h("div", {class: "qp-card-group"}, caps(t("ladders.card.group.then")), ...stages));
	}
	parts.push(...masteryOf(w, st));
	const inPath = new Set((st.stages ?? []).flatMap((s) => s.ids));
	const same = w.words.moves.filter((m) => m.ladder === lad.id && m.rung === st.rung && m.move !== st.move
		&& !inPath.has(m.move)).map((m) => m.move);
	if (same.length) {
		const tiles = h("div", {class: "qp-card-moves"}, ...same.map((move) => groupTile(ctx, move)));
		const label = caps(t("ladders.card.group.same"));
		const group = h("div", {class: "qp-card-group"}, label, tiles);
		// a crowded step's group folds under one row naming it, which opens it in place (its label back, its first tile
		// then focused)
		if (same.length > SAME_SHOWN) {
			tiles.hidden = true;
			label.hidden = true;
			const more = h("qp-quiet-button", {label: t("ladders.card.group.same.more", {count: same.length}),
				"data-focus-key": "card:same", "on:click": () => {
					tiles.hidden = false;
					label.hidden = false;
					more.remove();
					tiles.querySelector("qp-move-tile")?.focus({preventScroll: true});
				}});
			group.insertBefore(more, tiles);
		}
		parts.push(group);
	}
	return parts;
}

/** The Helps row (slice 42, a soft need, never a gate): "Helps: {move}" where the step's move names a pose that helps
 * it and the setup offers that pose (the Book's move `hint`), the pose's tile a button showing it in place; the move's
 * own advice stands in its How to (2026-10-22). */
function hintOf(ctx, move) {
	const {w, book} = ctx;
	const pose = factsOf(book).get(move)?.hint;
	if (pose == null) return [];
	return [h("div", {class: "qp-card-check-row"}, groupTile(ctx, pose),
		h("div", {class: "qp-card-check-words"}, h("span", {class: "qp-type-meta",
			text: t("ladders.card.hint", {move: moveName(w, pose)})})))];
}

/** What a stretch check a Start runs opens (slice 42: its lines and the Result name it): the move the
 * window or the card shows where that move waits on the check (its `waits`), else the climb to the step above the
 * player's on the card's ladder (`lad`); none for another Start. */
function checkOpens(book, lad, start, shown = null) {
	if (start?.as !== "check") return null;
	if (shown != null && factsOf(book).get(shown)?.waits?.kind === "check") return {move: shown};
	const up = lad ? stepAt(lad, lad.cur + 1) : null;
	return up?.move != null ? climbOpens(lad.id, up) : null;
}

/** The Start of a start (`start`) in a StartBlock: its label, its lines, greyed with its reason by the pump's gate,
 * or a refusal's lines and link rows; `lad` the ladder the
 * card is for, `shown` the move the card or the window shows, and `move` where that is a move's (its window, its page,
 * a card's move shown in place: a pain refusal and a reach line name the move). A Start that trains another move than
 * `shown` (a step ahead's climb, a kept step's climb back, a stage of the path, a stretch check or an easier move on
 * the player's own step, a family's opening) stands in its block with the tile of the move it trains, under "To get
 * here", or "Train with" on the player's own step (`own`). `ctx.go(link)` opens a refusal's link; `ctx.start(start,
 * opens)` starts, with what a stretch check opens (`checkOpens`). */
export function startBlock(ctx, start, lad, shown, move = false, own = false) {
	const {w, book, pump} = ctx;
	const block = h("qp-start-block", {"data-focus-key": "card:start"});
	if (!start?.move || start.refused) {
		const refused = start ? refusalOf(w, lad, start, move) : {lines: [], links: []};
		block.lines = refused.lines;
		block.links = refused.links.map((link) => ({...link, act: () => ctx.go(link)}));
		return block;
	}
	const held = heldBy(pump);
	const elsewhere = shown != null && start.move !== shown;
	const opens = checkOpens(book, lad, start, move ? shown : null);
	Object.assign(block, {label: startLabel(w, start), state: held?.state ?? "normal",
		note: held?.note ?? null, block: elsewhere, own: elsewhere && own,
		tileMove: elsewhere ? w.moves.get(start.move)?.key ?? null : null,
		tileQuality: elsewhere ? tileQuality(factsOf(book).get(start.move)) : null});
	block.lines = startLines(w, book, lad, start, move ? shown : null, opens);
	block.addEventListener("qp-start", () => ctx.start(start, opens));
	return block;
}

/** The card's header: the move in its quality's colour, with its QualityWord. */
function header(w, move, quality, id) {
	const title = h("h3", {class: "qp-type-heading qp-card-title", id, text: moveName(w, move)});
	const tone = qualityName(quality);
	if (tone) title.style.setProperty("--qp-tone", `var(--qp-quality-${tone})`);
	return h("div", {class: "qp-card-head"}, title,
		quality ? h("qp-quality-word", {quality, word: qualityWord(w, quality)}) : "");
}

/** The figure of a move, pausable as any loop. */
function figure(w, move) {
	const m = w.moves.get(move);
	return h("qp-figure-stage", {move: m?.key ?? "", pausable: true, label: m?.name ?? ""});
}

/** The ahead line over a step above the player's or a move above their reach. */
function aheadLine() {
	return h("p", {class: "qp-type-subtitle", text: t("ladders.move.locked")});
}

/** Another move of the step shown in place in the card: its header, figure, How and Tips (its advice last in Tips
 * since 2026-10-24, after How's steps from 2026-10-22, when its hint line went), its checks (slice 42), its own Start
 * (its move window's, ns.StartFor of the move: a move that waits for what opens it runs that, "Do the check" or the
 * move whose top opens its stage) and Back to the step. */
function inPlace(ctx, lad, move, back) {
	const {w, book} = ctx;
	const fact = factsOf(book).get(move);
	const quality = fact?.quality ?? null;
	const above = aboveReach(w, book, fact, move);
	const start = book.moves.find((x) => x.move === move)?.start ?? null;
	const backButton = h("qp-quiet-button", {label: t("ladders.card.step"), "data-focus-key": "card:back",
		"on:click": back});
	const own = {...ctx, ownStart: start};
	return {
		body: [header(w, move, quality, ctx.titleId), above ? aheadLine() : "", figure(w, move),
			howTo(w, book, move), ...moveChecks(own, move), ...hintOf(own, move)],
		start: startBlock(ctx, start, lad, move, true),
		quiet: [backButton],
	};
}

/** A step's card: `ctx` {w, book, pump, titleId, start(start, opens), check(need, opens), haul(start), go(link),
 * select(ladder, step), page(ladder), show(move)}, the ladder (`lad`, steps.js `ladderOf`) and the
 * step (`st`; a ladder that waits whole: its lowest step). Returns its parts: `body` (the header down to the groups,
 * the needs, the checks and the hint), `start` (its StartBlock) and `quiet` (the links out, the top, the weight, the
 * gear and the Ladder page row). A ladder that waits whole has the header and the quiet part of any step's card, its
 * note ("Not open yet", the setup's or the list's) under its header. */
function cardParts(ctx, lad, st, showing = null) {
	const {w, book} = ctx;
	if (showing) return inPlace(ctx, lad, showing, () => ctx.show(null));
	const kind = kindOf(lad, st);
	const closed = kind === "waiting" && lad.row.waits === "closed";
	// a step above the player's whose own move its Start does not run: the ahead line, its How to and its stop line
	const ahead = kind === "next" || kind === "ahead" || closed;
	const start = st.start ?? null;
	const card = {...ctx, ownStart: start};
	const parts = [header(w, st.move, st.quality, ctx.titleId)];
	if (kind === "waiting") parts.push(h("p", {class: "qp-type-subtitle", text: waitingNote(lad)}));
	if (ahead) parts.push(aheadLine());
	// a family not open yet says what opens it, beside its Start where that runs the climb that opens it
	if (closed && st.start?.via !== "opens") {
		const line = opensLine(w, lad);
		if (line) parts.push(h("p", {class: "qp-type-subtitle", text: line}));
	}
	parts.push(figure(w, st.move));
	// a pose's goal pose is its measure, so its How to leaves it out
	if (ahead) parts.push(howTo(w, book, st.move, lad.words.dose !== "pose"));
	const next = opensNext(card, lad, st, kind);
	parts.push(...next.reached);
	parts.push(h("div", {class: "qp-card-measure"}, ...measure(w, book, lad, st, kind, next.said)));
	// on the player's step the climb stands under To step up, on the top its line in its main move's group; above it,
	// what reaching the step takes stands before the step's own groups, its path's stages each a line
	const own = kind === "now" || kind === "top";
	if (!own && (next.climb.length || next.stages)) {
		parts.push(h("div", {class: "qp-card-next"}, ...next.climb,
			...(next.stages ?? []).map((stage) => roadRow(w, stage))));
	}
	const needs = climbNeeds(card, lad.id, next.into, next.up);
	parts.push(...groups(card, lad, st, kind, own ? next.climb : [], needs));
	if (kind !== "now" && (needs.needs.length || needs.checks.length)) {
		parts.push(h("div", {class: "qp-card-next"}, ...needs.needs, ...needs.checks));
	}
	if (closed) parts.push(...familyChecks(card, lad));
	parts.push(...hintOf(card, st.move));
	const quiet = [];
	const out = linksOf(w, lad.id, st.step).out;
	if (out.length) {
		quiet.push(caps(t("ladders.card.opens")),
			h("div", {class: "qp-card-opens"}, ...out.map((link) => {
				const words = chipWords(w, book, link.ladder, link.step);
				// a button that swaps that step's card in
				return h("qp-secondary-button", {label: words, "data-focus-key": `card:opens:${link.ladder}:${link.step}`,
					"on:click": () => ctx.select(link.ladder, link.step)});
			})));
	}
	// the ladder's top, but on its top step, whose header names it
	const topStep = stepAt(lad, lad.top);
	if (topStep && kind !== "top") {
		quiet.push(h("p", {class: "qp-type-meta", text: t("ladders.card.top", {move: moveName(w, topStep.move),
			quality: qualityWord(w, topStep.quality)})}));
	}
	const fact = factsOf(book).get(st.move);
	if (fact?.topLoad != null) {
		const each = w.moves.get(st.move)?.load === "each";
		quiet.push(h("p", {class: "qp-type-meta", text: t("ladders.card.weight", {best: lifted(fact.topLoad, fact.topReps,
			each), last: fact.lastLoad != null ? lifted(fact.lastLoad, fact.last ?? fact.topReps, each) : "–"})}));
	}
	const gear = lad.row.gear?.find((g) => g.quality);
	if (gear) {
		quiet.push(h("p", {class: "qp-type-meta", text: t("ladders.card.gear", {count: gear.count,
			gear: w.words.gear[gear.key] ?? gear.key, quality: qualityWord(w, gear.quality)})}));
	}
	if (!lad.row.opens && ctx.page) {
		quiet.push(h("qp-row", {heading: t("ladders.card.page"), glyph: "action_book", trailing: "nav",
			"data-focus-key": "card:page", "on:click": () => ctx.page(lad.id)}));
	}
	return {body: parts, start: withHaul(ctx, lad, kind, start, startBlock(ctx, start, lad, st.move, false, own)), quiet};
}

/** The Start (`start`, its `block`) of the player's own cardio step's card, its top step's too (Tree.lua's `ownStep`),
 * with the Long Haul beside it where both fit with their words, else under it (slice 55, the Book row's `haul`: "Long
 * Haul 1:30", its length open now, ns.Haul), a SecondaryButton that starts it as the Start starts (`ctx.haul`:
 * `pump.move` with the target {haul: true}, which the engine reads again, ns.HaulStart), greyed where the pump's
 * gate holds the card's Start; the Start alone on any
 * other card and beside a Start that starts nothing, whose refusal stands alone (Tree.lua Tree.CardStep). */
function withHaul(ctx, lad, kind, start, block) {
	const haul = lad.row.haul;
	if ((kind !== "now" && kind !== "top") || !haul || !start?.move || start.refused) return block;
	block.aside = h("qp-secondary-button", {label: t("ladders.card.haul", {time: amount(haul.seconds, true)}),
		disabled: Boolean(heldBy(ctx.pump)), "data-focus-key": HAUL_KEY, "on:click": () => ctx.haul(haul.start)});
	return block;
}

/** The moves a step's groups, checks and hint name (slice 42): its rung's moves, its path's (a stage's move may stand
 * on the rung below), each one's checks' poses and the hint's pose. */
function groupMoves(w, book, lad, st) {
	const facts = factsOf(book);
	const moves = [...w.words.moves.filter((m) => m.ladder === lad.id && m.rung === st.rung).map((m) => m.move),
		...(st.stages ?? []).flatMap((s) => s.ids)];
	const poses = moves.flatMap((m) => (facts.get(m)?.checks ?? []).map((c) => c.move));
	return [...moves, ...poses, facts.get(st.move)?.hint];
}

/** What a card on step `st` of ladder `lad` prints from the Book beyond its ladder's row (`showing` a move shown in
 * place): the moves whose facts and records it names (the step's move, its counted forms, its groups' moves, their
 * checks' poses and its hint's, the move its Start trains; the shown move, its checks' poses, its hint's and its
 * Start's) and the other ladders whose rows its chips, its need and check rows and its Start's lines name. */
function cardReads(w, book, lad, st, showing) {
	const facts = factsOf(book);
	if (showing) {
		const fact = facts.get(showing);
		return {moves: [showing, fact?.start?.move, fact?.hint, ...(fact?.checks ?? []).map((c) => c.move)],
			ladders: [fact?.start?.ladder, ...(w.moves.get(showing)?.checks ?? []).flatMap((c) => c.any ?? [c])
				.map((c) => c.ladder)]};
	}
	const kind = kindOf(lad, st);
	const target = targetOf(lad, st, kind);
	const into = target != null ? stepAt(lad, target) : null;
	// a family not open yet: its opening's links into its first step (its check rows' poses, slice 42's fix round)
	const first = lad.row.waits === "closed" ? cardStep(lad, null) : null;
	const opening = first ? linksOf(w, lad.id, first.step, book).into.filter((x) => x.link.opens) : [];
	const links = [...linksOf(w, lad.id, st.step).out, ...(into ? linksOf(w, lad.id, into.step).into : []), ...opening];
	return {
		moves: [st.move, ...formsOf(w, lad, st).map((x) => x.move), ...groupMoves(w, book, lad, st), st.start?.move,
			...opening.map((x) => x.need?.move)],
		ladders: [st.start?.ladder, ...links.map((link) => link.ladder)],
	};
}

/** What a card on step `st` shows as the Book and the pump's gate stand (`lad` its ladder, `showing` a move shown in
 * place): its ladder's row, the gate, and what it prints from the rest of the Book (`cardReads`), so it is drawn again
 * when one of them moves and only then, and a sync that changed nothing keeps its figure playing. */
function cardSignature(w, book, lad, st, showing, pump) {
	const facts = factsOf(book);
	const reads = cardReads(w, book, lad, st, showing);
	const moves = [...new Set(reads.moves.filter((m) => m != null))];
	const ladders = [...new Set(reads.ladders.filter((l) => l != null && l !== lad.id))];
	return JSON.stringify([showing, lad.row, book.pro, pump?.gate, pump?.locked_until,
		moves.map((m) => [facts.get(m) ?? null, recordOf(book, facts, m)]),
		ladders.map((l) => book.ladders.find((row) => row.ladder === l) ?? null)]);
}

/** Puts the focus in a card's Window that opened or changed what it shows (`root`): on the tile
 * of the move it came back from (`from`), else its Start where it can run, else Back to the step, else its title (a
 * greyed Start takes no focus, and a screen reader then starts at the card's name). */
function focusCard(root, from = null) {
	const tile = from != null ? root.querySelector(`[data-focus-key="card:move:${from}"]`) : null;
	const start = root.querySelector("qp-start-block")?.primary;
	const target = tile ?? (start && !start.control?.disabled ? start : null)
		?? root.querySelector('[data-focus-key="card:back"]');
	if (target) {
		target.focus({preventScroll: true});
		return;
	}
	const title = root.querySelector(".qp-card-title");
	title?.setAttribute("tabindex", "-1");
	title?.focus({preventScroll: true});
}

/** Where a refused Start's link row goes: Your setup at the switch or the piece its gap names (or at its pain
 * switches), or that move's page with its Include in pumps. */
export function goLink(link) {
	if (link.kind === "include" && link.move != null) push(MOVE, {move: link.move});
	else openMenu(SETUP, {focus: link.focus ?? null});
}

/** A move started from a step's card or a move's window (`pump.move`: Rust reads its start again as it composes, and
 * the gates and the place window hold it as any pump), with where its Result returns (`from`). `nowhere` runs once
 * where it goes nowhere: the intent refused (a gate closed in between), or the `pump` topic's count of the starts that
 * went nowhere moving on with no pump on screen (the engine refused the start it read again, or composed no move), so
 * the card or the window it came from comes back, the Book built again saying why. A pump on screen ends the watch. */
export function startMove(target, from, nowhere = null) {
	const before = get("pump")?.refused ?? 0;
	let watching = Boolean(nowhere);
	let stop = () => {};
	const end = (gone) => {
		if (!watching) return;
		watching = false;
		stop();
		if (gone) nowhere();
	};
	if (nowhere) {
		stop = subscribe("pump", (p) => {
			if (p.pump) end(false);
			else if ((p.refused ?? 0) > before) end(true);
		});
		// an answer the topic gave at once (it stands as it was pressed, so none) ended the watch before its stop came
		if (!watching) stop();
	}
	intent("pump.move", {target, from}).catch((e) => {
		console.error(e);
		end(true);
	});
}

/** The step a card stands on: step `step` of the ladder, or a ladder that waits whole's lowest (`step` none). */
function cardStep(lad, step) {
	return step == null ? lad.row.steps.filter((s) => s.rung).at(-1) ?? null : lad.byStep.get(step) ?? null;
}

/** The reason a card's Start is refused or greyed, as `tree_day` counts it: setup, off, game (the game asks the pumps),
 * reading (the engine folds); none for a pain flag or felt unwell, which it counts under no name. */
function refusedReason(start, pump) {
	if (start?.refused === "setup" || start?.refused === "off") return start.refused;
	if (start?.refused) return null;
	return heldBy(pump)?.reason ?? null;
}

/** A card opened: the counts take it, with its Start's refused or greyed reason (`book.card`). */
function cardOpened(lad, st, pump) {
	const start = st?.start ?? (lad ? waitingStart(lad) : null);
	intent("book.card", {refused: refusedReason(start, pump)}).catch(console.error);
}

/** A step's card in the kit's Window from the foot (a ladder's page, the list), following the Book and the pump's
 * gate while it stands, closed by its ×, Escape or a click on the dim: `opts` {w, place (ladder or list, whose row the
 * player opened it from, where its Start's Result returns), ladder, step (none: a waiting ladder), start(target, from)
 * (a stretch check's start, slice 42, with what it opens in `from.opens`), go(link), page(ladder)?, onClose()?,
 * counted? (false: no card opened anew, so not counted), origin()? (the step the player tapped, which it grows out of
 * and shrinks back into: qp-window's `open`)}. A need row or a step it opens swaps that step's card in, which shrinks
 * into no tile then; the window keeps what it stands on (`card`: {ladder, step, showing}). Returns the window. */
export function openCardWindow(opts) {
	const {w} = opts;
	let book = get("book");
	let pump = get("pump");
	const win = h("qp-window", {placement: "bottom", class: "qp-card-window", dim: true});
	win.card = {ladder: opts.ladder, step: opts.step, showing: null};
	/** Starts `target` from the card's step, the window closed first, with what a stretch check opens (`opens`). */
	const begin = (target, start, opens) => {
		const lad = ladderOf(w, book, win.card.ladder);
		const st = lad && cardStep(lad, win.card.step);
		if (!st) return;
		win.close();
		opts.start(target, {place: opts.place, ladder: win.card.ladder, step: st.step, ...(opens ? {opens} : {})});
	};
	const ctx = {w, titleId: `qp-card-title-${opts.ladder}`,
		start: (start, opens) => {
			const lad = ladderOf(w, book, win.card.ladder);
			const st = lad && cardStep(lad, win.card.step);
			if (!st) return;
			begin(win.card.showing ? {move: win.card.showing} : {ladder: win.card.ladder, rung: st.rung}, start, opens);
		},
		// a check row's "Do the check" (slice 42): the check alone, returning to this card's step
		check: (need, opens) => begin({check: need}, {}, opens),
		// the Long Haul beside the Start (slice 55), returning to this card's step
		haul: (start) => begin({haul: true}, start),
		go: (link) => {
			win.close();
			opts.go(link);
		},
		// another step's card in this one's place
		select: (ladder, step) => {
			const lad = book?.ready ? ladderOf(w, book, ladder) : null;
			if (!lad) return;
			win.card = {ladder, step: lad.row.waits ? null : step, showing: null};
			// a swap: the tile it grew from is another step's
			win.origin = null;
			ctx.titleId = `qp-card-title-${ladder}`;
			draw();
			win.content.scrollTop = 0;
			focusCard(win);
			cardOpened(lad, cardStep(lad, win.card.step), pump);
		},
		// the ladder's page once the window has gone, so it gives the focus back to the control that opened the card
		page: opts.page ? (ladder) => {
			win.addEventListener("qp-closed", () => opts.page(ladder), {once: true});
			win.close();
		} : null,
		show: (move) => {
			const from = win.card.showing;
			win.card.showing = move;
			draw();
			focusCard(win, move == null ? from : null);
		},
	};
	let sig = null;
	const draw = () => {
		const lad = book?.ready ? ladderOf(w, book, win.card.ladder) : null;
		const st = lad ? cardStep(lad, win.card.step) : null;
		if (!st) {
			win.close();
			return;
		}
		const now = JSON.stringify([win.card.ladder, win.card.step])
			+ cardSignature(w, book, lad, st, win.card.showing, pump);
		if (now === sig) return;
		sig = now;
		Object.assign(ctx, {book, pump});
		const parts = cardParts(ctx, lad, st, win.card.showing);
		win.label = parts.body[0]?.textContent ?? lad.words.name;
		keepFocus(win, () => {
			win.content.replaceChildren(...parts.body, ...(parts.quiet.length
				? [h("div", {class: "qp-card-quiet"}, ...parts.quiet)] : []));
			win.foot.replaceChildren(parts.start);
		});
	};
	const stops = [subscribe("book", (b) => {
		book = b;
		draw();
	}), subscribe("pump", (p) => {
		pump = p;
		draw();
	})];
	win.addEventListener("qp-closed", () => {
		for (const stop of stops) stop();
		win.remove();
		opts.onClose?.();
	}, {once: true});
	document.body.append(win);
	draw();
	win.open(opts.origin ?? null);
	// its Start where it can run, else its title (never the × that showModal would focus)
	focusCard(win);
	const lad = ladderOf(w, book, opts.ladder);
	if (lad && opts.counted !== false) cardOpened(lad, cardStep(lad, opts.step), pump);
	return win;
}

/** A move's window from Book (the Moves grid): its figure, a move above the player's reach's too under "Above your step
 * for now" with no Max set (David, 2026-10-14), its quality's word and record, its setup, How, Tips and stop lines,
 * its Helps row (slice 42: the pose that helps it, a row to that pose's page), its switches and Open its page,
 * and at its foot its Start (ns.StartFor of the move: at its own goal at or under the player's rung, its ladder's climb
 * above it in the "To get here" row with the tile of the move that climb trains, a move that waits for what opens it
 * that ("Do the check", or the move whose top opens its stage), greyed with its reason by the pump's gate, or what
 * refuses it with its link rows). `opts.start(target, from)` starts it, the window closed first; its Result returns to
 * the move's window. `opts.origin()` the Moves tile the player tapped, which it grows out of and shrinks back into
 * (qp-window's `open`). */
export function openMoveWindow(w, book, move, opts = {}) {
	const facts = factsOf(book);
	const fact = facts.get(move);
	const m = w.moves.get(move);
	const quality = fact?.quality ?? (fact?.trained ? 1 : null);
	const above = aboveReach(w, book, fact, move);
	const value = moveValue(w, move, recordOf(book, facts, move), above, fact);
	const guide = guideOf(m, fact);
	const win = h("qp-move-window", {heading: m?.name ?? "", quality: quality ?? null,
		"quality-word": quality ? qualityWord(w, quality) : null,
		record: value && value.kind !== "maxset" ? valueWords(value).number : null, bare: !value,
		ahead: above ? t("ladders.move.locked") : null, setup: guide.setup, stop: m?.stop ?? null});
	win.how = guide.how;
	win.tips = m?.tips ?? [];
	win.stage.setAttribute("move", m?.key ?? "");
	const close = () => win.close();
	const pose = fact?.hint;
	if (pose != null) {
		const helps = h("qp-row", {heading: t("ladders.card.hint", {move: moveName(w, pose)}), glyph: "action_book",
			trailing: "nav", "on:click": () => {
				close();
				push(MOVE, {move: pose});
			}});
		win.extra.append(h("qp-card", {rows: true}, helps));
	}
	win.extra.append(h("qp-card", {rows: true}, ...switches(w, book, facts, move, close),
		h("qp-row", {heading: t("ladders.move.open"), glyph: "action_book", trailing: "nav", "on:click": () => {
			close();
			push(MOVE, {move});
		}})));
	const stops = [follow(w, win.extra, move)];
	if (opts.start) {
		let topic = book;
		let pump = get("pump");
		// its own ladder's words name the step a refusal names; a move off the ladders names the start's ladder
		const own = m?.ladder ?? null;
		const draw = () => {
			const start = topic.moves.find((x) => x.move === move)?.start ?? null;
			const lad = own != null ? ladderOf(w, topic, own) : null;
			const ctx = {w, book: topic, pump, start: (st, opens) => {
				close();
				opts.start({move}, {place: "move", ladder: st.ladder ?? null, step: null, ...(opens ? {opens} : {})});
			}, go: (link) => {
				close();
				goLink(link);
			}};
			// above the player's reach its Start is its ladder's climb, in "To get here"
			keepFocus(win, () => win.foot.replaceChildren(startBlock(ctx, start, lad, move, true)));
		};
		stops.push(subscribe("book", (b) => {
			topic = b;
			draw();
		}), subscribe("pump", (p) => {
			pump = p;
			draw();
		}));
	}
	win.addEventListener("qp-closed", () => {
		for (const stop of stops) stop();
		win.remove();
	});
	document.body.append(win);
	win.open(opts.origin ?? null);
	return win;
}
