// A chapter as the Saga draws it (the Saga, 2026-10-15; docs/ui.md, "The Saga"; board A-Day; the phones' LogTab and
// LogDay), one builder for today's on the Saga's root and an earlier day's on its page (desk.log.day), from a day of
// the `log` topic (questpump-desk/src/saga.rs `SagaDay`): the hero (the chapter's SlantTag and the date, the day's moves in numSaga
// with Moves beside it, Move for 1, and the day's word under it, the knight of the move that stands for the day cut out
// beside the number's foot under those words and inside the hero, the line of its unlock or its record under the
// number, kept left of the knight), the tile Your day with the plan's legend (the DayStrip over the KindBar),
// the tile Muscles beside the tile N records · N unlocks (a half with nothing hides and the other takes the row), the
// XP tile (hidden without XP or a level known) beside the Week tile (Week, Old best and Share week in its head, the
// WeekStrip), and Pumps, the day's pump rows as the Log listed them, each opening its page. The hero reads as one line
// with its highlight ("Chapter 12, Tuesday 14 October: 59 moves, Best day ever, Unlocked today, Archer push-up, Epic";
// the phones'), in the reading order as text, the strip as its parts' totals, the week a day at a time with its unit;
// each tile is a named group.
// `reveal()` plays the first open's motion once: the bars grow left to right, the number counts up and pops, the
// muscles light in turn (none under Reduce Motion, whose tokens are zero).

import {roll} from "../../lib/carry.js";
import {t} from "../../lib/copy.js";
import {LOCALE} from "../../lib/format.js";
import {push} from "../../lib/router.js";
import {KIND_WORDS, chapterWords, dayWord, heroSpoken, movesLabel, noon, number, setWords, shortDate, stripSpoken, unit,
	weekdaySpoken} from "../../lib/saga.js";
import {h, qualityName, styled} from "../../kit/element.js";
import {PUMP, pumpRow} from "./log.js";

const SIGNED = new Intl.NumberFormat(LOCALE, {signDisplay: "always"});
const WEEKDAY = new Intl.DateTimeFormat(LOCALE, {weekday: "short"});
const DAYS = new Intl.ListFormat(LOCALE, {type: "conjunction"});
const motion = window.matchMedia("(prefers-reduced-motion: reduce)");

/** What a chapter's parts read beside the day (`chapter`'s `ctx`): from `book.words` the moves' words by id, the
 * qualities' words, the muscles' names by id and the felt-unwell word; from `book` the body's figure; the focus's
 * `plan` from `log`. */
export function context(words, book, plan) {
	return {
		names: new Map(words.moves.map((m) => [m.move, m])),
		qualities: words.qualities,
		muscles: new Map(words.muscles.map((m) => [m.id, m.name])),
		unwell: words.unwell,
		figure: book?.figure ?? null,
		plan,
	};
}

/** A move's words from `book.words` (its name, its figure's key, its weight's kind). */
function named(ctx, move) {
	return ctx.names.get(move) ?? {name: "", key: null};
}

/** A rung's quality's word (contract/sheet.json's: Starter … Artifact) in its tone. */
function qualityLine(ctx, quality) {
	const name = qualityName(quality);
	const word = h("span", {class: "qp-saga-quality", text: ctx.qualities[quality - 1] ?? ""});
	if (name) styled(word, {"--qp-tag": `var(--qp-quality-${name})`});
	return word;
}

/** A day's word as its SlantTag: Best day ever filled gold with the star, else the word outlined in its tone; none
 * where `dayWord` has none (before the gate, a Common day). */
export function dayTag(quality) {
	const label = dayWord(quality);
	if (!label) return null;
	const best = quality === "legendary";
	return h("qp-slant-tag", {label, tone: best ? "gold" : quality, filled: best, star: best});
}

/** A count drawn large before its unit word drawn small ("3 records", "2 unlocks"). */
export function counted(count, what) {
	return h("span", {class: "qp-saga-counted"}, h("b", {class: "qp-saga-count", text: number(count)}),
		h("span", {class: "qp-type-caps", text: unit(count, what)}));
}

/** The hero's line's words: its caps (Unlocked today, an earlier day's Unlocked; Record today, New record), the move's
 * name and its quality's word or its best; none for a day with neither. */
function highlightWords(day, ctx, today) {
	const hl = day.highlight;
	if (!hl) return null;
	const move = named(ctx, hl.move);
	if (hl.kind === "unlock") {
		return {caps: t(today ? "saga.unlocked.today" : "saga.unlocked"), move: move.name,
			what: ctx.qualities[hl.quality - 1] ?? ""};
	}
	return {caps: t(today ? "saga.record.today" : "saga.record"), move: move.name,
		what: setWords(hl.best, hl.hold, move.load === "each")};
}

/** The hero's line under the number: its caps (an unlock's in gold, a record's in text2) over the move with its
 * quality's word in its tone or its best. */
function highlight(day, ctx, today) {
	const words = highlightWords(day, ctx, today);
	if (!words) return null;
	const hl = day.highlight;
	const unlock = hl.kind === "unlock";
	const what = unlock ? qualityLine(ctx, hl.quality) : h("span", {class: "qp-saga-best", text: words.what});
	return h("div", {class: "qp-saga-highlight"},
		h("div", {class: unlock ? "qp-saga-rule-line" : "qp-saga-rule-line qp-saga-rule-record"},
			h("i", {class: "qp-saga-rule"}), h("span", {class: "qp-type-caps", text: words.caps})),
		h("div", {class: "qp-saga-what"}, h("span", {class: "qp-saga-move", text: words.move}), what));
}

/** What a screen reader hears for the hero: the day's line, then its highlight's caps, move and quality or best. */
function heroLine(day, ctx, today) {
	const words = highlightWords(day, ctx, today);
	return [heroSpoken(day), ...(words ? [words.caps, words.move, words.what] : [])].filter(Boolean).join(", ");
}

/** The knight of the move that stands for the day: its key pose, cut out on the dark page. */
function knight(day, ctx) {
	const key = day.hero ? named(ctx, day.hero.move).key : null;
	if (!key) return null;
	return h("div", {class: "qp-saga-knight", "aria-hidden": "true"}, h("qp-figure-stage", {move: key, still: true}));
}

/** The hero: the chapter's tag and date, the number with Moves and the day's word, the knight beside the number's foot
 * and the line under the number; its words in the reading order as one line of text (a screen reader's browse mode
 * reads text, never a group's label alone), the drawing hidden from it. */
function hero(day, ctx, today) {
	const number_ = h("span", {class: "qp-saga-number", "aria-hidden": "true", text: number(day.moves)});
	const side = h("div", {class: "qp-saga-side"}, h("span", {class: "qp-saga-unit", text: movesLabel(day.moves, "day")}),
		dayTag(day.quality));
	const line = highlight(day, ctx, today);
	if (line) line.setAttribute("aria-hidden", "true");
	const el = styled(h("div", {class: "qp-saga-hero"},
		h("p", {class: "qp-unseen", text: heroLine(day, ctx, today)}),
		h("div", {class: "qp-saga-head", "aria-hidden": "true"}, h("qp-slant-tag", {label: chapterWords(day.chapter)}),
			h("span", {class: "qp-type-caps qp-saga-date", text: shortDate(day.day)})),
		number_, h("div", {class: "qp-saga-aside", "aria-hidden": "true"}, side, knight(day, ctx)), line ?? ""),
	{"--qp-digits": Math.max(2, String(day.moves).length)});
	return {el, number: number_};
}

/** A tile: a card of the Saga's tiles, a group named by its aria label (never a landmark of its own). */
function tile(label, cls, ...children) {
	return h("div", {class: ["qp-saga-tile", cls].join(" "), role: "group", "aria-label": label}, ...children);
}

/** Your day with the plan's legend: the strip over the kinds bar (the day's moves stand in the hero above it). */
function dayTile(day, ctx) {
	const head = h("div", {class: "qp-saga-tile-head"},
		h("span", {class: "qp-type-caps", text: t("saga.day")}),
		h("span", {class: "qp-saga-plan qp-type-meta"}, h("i", {class: "qp-kind-tick-mark"}),
			t("saga.plan", {focus: ctx.plan.name})));
	const strip = h("qp-day-strip", {label: stripSpoken(day.buckets)});
	strip.bars = day.bars;
	strip.buckets = day.buckets;
	const kinds = h("qp-kind-bar", {label: [t("saga.plan", {focus: ctx.plan.name}),
		...KIND_WORDS.map((w, k) => `${w} ${number(day.kinds[k])}`)].join(", ")});
	kinds.kinds = day.kinds;
	kinds.shares = ctx.plan.shares;
	return {el: tile(t("saga.day"), "qp-saga-day", head, strip, kinds), strip};
}

/** The muscles the day trained: Most in gold, Some at its share, with their legend. */
function musclesTile(day, ctx) {
	const words = (ids) => ids.map((id) => ctx.muscles.get(id)).filter(Boolean).join(", ");
	const said = [day.most.length ? `${t("saga.muscles.most")}: ${words(day.most)}` : "",
		day.some.length ? `${t("saga.muscles.some")}: ${words(day.some)}` : ""].filter(Boolean).join(". ");
	const map = h("qp-muscle-map", {mode: "day", figure: ctx.figure ?? "male", label: said || t("saga.muscles")});
	map.most = day.most;
	map.some = day.some;
	const legend = h("div", {class: "qp-saga-legend qp-type-meta", "aria-hidden": "true"},
		h("span", {}, h("i", {class: "qp-swatch-most"}), t("saga.muscles.most")),
		h("span", {}, h("i", {class: "qp-swatch-some"}), t("saga.muscles.some")));
	return {el: tile(t("saga.muscles"), "qp-saga-muscles", h("span", {class: "qp-type-caps", text: t("saga.muscles")}), map,
		legend), map};
}

/** The day's records (the move, its best in gold, was {value}) and unlocks (the move, its quality's word in its tone);
 * none without either. */
function recordsTile(day, ctx) {
	if (!day.records.length && !day.unlocks.length) return null;
	const head = h("div", {class: "qp-saga-tile-head"});
	if (day.records.length) head.append(h("span", {class: "qp-saga-pair"}, h("qp-art", {name: "reward_record"}),
		counted(day.records.length, "record")));
	if (day.unlocks.length) head.append(counted(day.unlocks.length, "unlock"));
	const rows = [
		...day.records.map((r) => {
			const move = named(ctx, r.move);
			const each = move.load === "each";
			const was = r.was ? h("span", {class: "qp-saga-was", text: ` ${t("saga.was", {value: setWords(r.was, r.hold, each)})}`})
				: "";
			return h("div", {class: "qp-saga-line"}, h("span", {class: "qp-saga-move", text: move.name}),
				h("span", {class: "qp-saga-value"}, h("b", {class: "qp-saga-best", text: setWords(r.best, r.hold, each)}), was));
		}),
		...day.unlocks.map((u) => h("div", {class: "qp-saga-line"}, h("span", {class: "qp-saga-move",
			text: named(ctx, u.move).name}), qualityLine(ctx, u.quality))),
	];
	return tile([day.records.length ? `${number(day.records.length)} ${unit(day.records.length, "record")}` : "",
		day.unlocks.length ? `${number(day.unlocks.length)} ${unit(day.unlocks.length, "unlock")}` : ""]
		.filter(Boolean).join(", "), "qp-saga-records", head, ...rows);
}

/** The XP the day gained and its level (Level 23 → 24, or Level 24) with the level's bar; none where no sheet knows. */
function xpTile(day) {
	if (!day.xp && !day.level) return null;
	const parts = [];
	if (day.xp) {
		parts.push(h("div", {class: "qp-saga-xp"}, h("qp-art", {name: "reward_xp"}),
			h("b", {class: "qp-saga-xp-number", text: SIGNED.format(day.xp)}),
			h("span", {class: "qp-type-caps", text: t("pump.today.xp")})));
	}
	const l = day.level;
	if (l) {
		const up = l.from && l.from !== l.to;
		const bar = styled(h("span", {class: "qp-saga-level-bar"}, h("i")),
			{"--qp-share": l.need > 0 ? Math.min(1, l.into / l.need) : 1});
		// Level 23 → 24: the arrow sized with the numbers beside it
		parts.push(h("div", {class: "qp-saga-level"}, h("span", {class: "qp-type-caps qp-gold", text: t("saga.level")}),
			up ? h("b", {class: "qp-saga-level-from", text: number(l.from)}) : "",
			up ? h("qp-art", {name: "mark_arrow", class: "qp-saga-level-arrow"}) : "",
			h("b", {class: "qp-saga-level-to", text: number(l.to)}), bar));
	}
	const said = [day.xp ? t("saga.xp", {xp: day.xp}) : "",
		l ? (l.from && l.from !== l.to ? t("saga.level.up", {from: l.from, to: l.to}) : `${t("saga.level")} ${l.to}`)
			: ""].filter(Boolean).join(", ");
	return tile(said, "qp-saga-xp-tile", ...parts);
}

/** The week holding the day: Week with its Old best and its Share week in the head, its seven bars, the day ringed and
 * the Old best line; each day read with its moves ("Tuesday: 12 moves"). */
function weekTile(day, ctx) {
	const w = day.week;
	const share = w.moves > 0 ? h("qp-round-button", {icon: "share", small: true, spoken: t("saga.share.week"),
		"data-focus-key": "share-week", "on:click": () => ctx.share("week")}) : "";
	const head = h("div", {class: "qp-saga-tile-head"}, h("span", {class: "qp-type-caps", text: t("saga.week")}),
		h("span", {class: "qp-saga-week-end"}, w.oldBest ? h("span", {class: "qp-saga-old qp-type-meta"},
			h("i", {class: "qp-saga-dash"}), t("saga.week.best"), h("b", {class: "qp-saga-count", text: number(w.oldBest)}))
			: "", share));
	// the days a rest token kept, as the week's dots read them
	const kept = w.days.filter((d, i) => d.moves === 0 && w.rested.includes(i)).map((d) => WEEKDAY.format(noon(d.day)));
	const said = [...w.days.map((d) => weekdaySpoken(d.day, d.moves)),
		w.oldBest ? `${t("saga.week.best")} ${number(w.oldBest)}` : "",
		kept.length ? t("kit.weekdots.rested", {days: DAYS.format(kept)}) : ""].filter(Boolean).join(", ");
	const strip = h("qp-week-strip", {label: said, ringed: w.ringed, best: w.oldBest ?? null});
	strip.rested = w.rested;
	strip.days = w.days;
	return tile(t("saga.week"), "qp-saga-week", head, strip);
}

/** The day's pump rows, as the Log listed them, each opening its page (a no-move day's page lists them alone). */
export function pumps(day, ctx) {
	if (!day.entries.length) return [];
	const n = day.entries.length;
	return [h("qp-section-label", {label: t("saga.pumps")}), h("div", {class: "qp-log-day"}, ...day.entries.map((e, i) =>
		pumpRow(e, n === 1 ? "alone" : i === 0 ? "first" : i === n - 1 ? "last" : "middle", ctx.unwell,
			() => push(PUMP, {id: e.id}))))];
}

/** A chapter's parts: its hero, tiles and pumps, and `reveal()`, its first open's motion. `ctx`: the move words
 * (`names`, a Map by id), the quality words (`qualities`), the muscles' names (`muscles`, a Map by id), the body's
 * `figure`, the focus's `plan`, the felt-unwell word (`unwell`) and `share(kind)` ("day" or "week"); `today`: today's
 * chapter (its caps say today). */
export function chapter(day, ctx, today) {
	const top = hero(day, ctx, today);
	const your = dayTile(day, ctx);
	const muscles = musclesTile(day, ctx);
	const records = recordsTile(day, ctx);
	const xp = xpTile(day);
	const parts = [top.el, your.el, h("div", {class: "qp-saga-pair-row"}, muscles.el, records),
		h("div", {class: "qp-saga-pair-row"}, xp, weekTile(day, ctx)), ...pumps(day, ctx)];
	return {
		parts,
		reveal() {
			if (motion.matches) return;
			your.strip.grow();
			muscles.map.light();
			// the number counts up from 0 over the reveal's time and pops at its end (lib/carry.js)
			if (day.moves > 0) roll(top.number, 0, day.moves, "saga-reveal");
		},
	};
}

