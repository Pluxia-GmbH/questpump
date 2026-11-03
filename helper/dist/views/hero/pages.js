// Hero's pushed pages (docs/ui.md, "Hero, the Book and the Log"; iOS HeroPages, Android's twin): a muscle's (from the
// body map) and a stat's or the level's (from the stat strip and the level line), each the DetailBar and one hero card
// (the quality or the value, the ladders that train or raise it with the move standing for each, a muscle's next goal),
// then with Pro its history under the Pro label (`hero.charts`); without Pro nothing more, no teaser. From the `hero`
// topic. On a muscle's page an open line whose move the library holds opens the move's page in the Book (the phones'
// LadderLines `pushes`, its chevron at its end); a locked line and a stat's page's lines are words.

import {get} from "../../lib/bridge.js";
import {t} from "../../lib/copy.js";
import {LOCALE} from "../../lib/format.js";
import {push, register} from "../../lib/router.js";
import {h} from "../../kit/element.js";
import {MOVE} from "../book/common.js";
import {page, watch, words} from "../page.js";
import {qualityChart, stepChart, stepWeeks, weekBars} from "./charts.js";
import {muscleName, openingWords, recordText} from "./words.js";

const NUMBER = new Intl.NumberFormat(LOCALE);
const GAIN = new Intl.NumberFormat(LOCALE, {signDisplay: "exceptZero"});

/** The ladders a hero card lists under its caps label: each the tile of the move standing for it (its quality's edge),
 * the move's name, the ladder's name with its quality, and the move's record; a family not open yet dimmed, with its
 * opening rule and its lock. Where it `pushes` (a muscle's page), an open line naming a move is a button to the move's
 * page. */
function ladderLines(title, lines, pushes = false) {
	if (!lines.length) return null;
	return h("div", {class: "qp-stack-small"}, h("h3", {class: "qp-type-caps", text: title}), ...lines.map((line) => {
		const move = line.move?.name ?? line.ladder_name;
		const meta = h("span", {class: "qp-ladder-meta"}, h("span", {class: "qp-type-meta", text: line.ladder_name}),
			line.opens ? h("span", {class: "qp-type-meta", text: openingWords(line.opens)})
				: line.word ? h("qp-quality-word", {quality: line.quality, word: line.word}) : "");
		const end = line.opens ? h("qp-art", {name: "mark_lock", class: "qp-ladder-lock", "aria-hidden": "true"})
			: line.record > 0 ? h("span", {class: "qp-type-num-row", text: recordText(line.record, line.seconds)}) : "";
		const parts = [h("qp-move-tile", {move: line.move?.key, quality: line.quality || null, size: 44}),
			h("span", {class: "qp-ladder-words"}, h("span", {class: "qp-type-body-strong", text: move}), meta), end];
		if (pushes && !line.opens && line.move) {
			return h("button", {type: "button", class: "qp-ladder-line qp-opens",
				"on:click": () => push(MOVE, {move: line.move.id})}, ...parts,
			h("span", {class: "qp-end-nav", "aria-hidden": "true"}));
		}
		return h("div", {class: line.opens ? "qp-ladder-line qp-closed" : "qp-ladder-line"}, ...parts);
	}));
}

/** The Pro label and the history card, while Pro holds and its charts came; nothing otherwise. */
function history(hero, charts, draw) {
	if (!hero.pro || !charts) return [];
	return [h("qp-section-label", {label: t("pro.title")}), h("qp-card", {class: "qp-charts"}, ...draw(charts))];
}

/** A muscle's next goal, in words: the step that raises its quality, or the top quality held; none on a muscle not
 * trained yet, whose Trained by lines open their moves. */
function raiseWords(raise) {
	switch (raise?.kind) {
		case "step": return t("character.muscle.raise", {step: raise.step, ladder: raise.ladder, quality: raise.quality});
		case "top": return t("character.muscle.raise.top", {quality: raise.quality});
		default: return null;
	}
}

register("desk.hero.muscle", ({id}) => {
	const named = get("hero")?.muscles.find((m) => m.id === id);
	const view = page({heading: named ? muscleName(named) : t("character.muscle.fallback", {number: id})});
	const draw = () => {
		const hero = get("hero");
		const muscle = hero?.muscles.find((m) => m.id === id);
		if (!muscle) return;
		const name = muscleName(muscle);
		const head = muscle.word ? h("qp-quality-word", {quality: muscle.quality, word: muscle.word, class: "qp-heading"})
			: h("h2", {class: "qp-type-heading qp-muted", text: t("character.muscle.untrained")});
		const raise = raiseWords(muscle.raise);
		const card = h("qp-card", {class: "qp-stack-small"}, head,
			ladderLines(t("character.trained.by"), muscle.ladders, true), raise ? words(raise, "subtitle") : null);
		const charts = history(hero, get("hero.charts"), (c) => [
			qualityChart({title: t("character.muscle.best.title"), label: t("character.muscle.best.chart", {muscle: name}),
				points: c.quality[id] ?? [], qualities: c.qualities, records: c.record_days, now: Date.now() / 1000}),
			stepWeeks({title: t("character.muscle.weeks.title"), name: t("character.muscle.weeks.chart", {muscle: name}),
				points: c.muscle_weeks[id] ?? [], mapSteps: c.map_steps}),
		]);
		view.body.replaceChildren(card, ...charts);
	};
	watch(view, "hero", draw);
	watch(view, "hero.charts", draw);
	return view;
});

/** A stat's or the level's value with its glyph and its name in caps under it. */
function value(number, glyph, name) {
	return h("div", {class: "qp-stat-value"}, glyph ? h("qp-art", {name: glyph}) : "",
		h("div", {}, h("p", {class: "qp-type-num-large", text: number}), h("p", {class: "qp-type-caps", text: name})));
}

register("desk.hero.stat", ({stat = null, level = false}) => {
	const name = () => {
		const s = get("hero")?.stats[stat];
		return level ? t("character.stat.level") : s?.name ?? t("character.stat.fallback", {number: stat + 1});
	};
	const view = page({heading: name()});
	const draw = () => {
		const hero = get("hero");
		if (!hero) return;
		const named = name();
		const parts = [];
		if (level) {
			const l = hero.level;
			parts.push(value(hero.summary ? NUMBER.format(l.level) : "–", null, named));
			if (hero.summary) {
				parts.push(h("qp-xp-bar", {bare: true, level: l.level, into: l.into, need: l.need, rested: l.rested,
					paragon: l.paragon}), h("p", {class: "qp-type-meta", text: t("kit.xp.progress", {into: l.into,
					need: l.need})}));
			}
		} else {
			const s = hero.stats[stat];
			parts.push(value(hero.summary && s ? NUMBER.format(s.value) : "–", s ? `stat_${s.key}` : null, named));
			if (s?.week) parts.push(h("p", {class: "qp-type-meta", text: t("character.stat.week", {gain: GAIN.format(s.week)})}));
			parts.push(ladderLines(t("character.raised.by"), s?.ladders ?? []));
		}
		const charts = history(hero, get("hero.charts"), (c) => {
			const levelWords = (v) => t("character.stat.level.value", {level: Math.floor(v),
				percent: Math.floor((v - Math.floor(v)) * 100 + 1e-9)});
			const now = Date.now() / 1000;
			const line = stepChart({title: t("character.stat.chart", {name: named}),
				label: t("character.stat.chart", {name: named}), points: level ? c.level : c.stats[stat] ?? [],
				records: c.record_days, now, format: level ? levelWords : (v) => NUMBER.format(v)});
			if (level) return [line];
			return [line, weekBars({title: t("character.stat.gain.title"), label: t("character.stat.gain.chart",
				{name: named}), points: c.gains[stat] ?? [], records: c.record_weeks, now,
				format: (v) => t("charts.points", {count: v})})];
		});
		view.body.replaceChildren(h("qp-card", {class: "qp-stack-small"}, ...parts.filter(Boolean)), ...charts);
	};
	watch(view, "hero", draw);
	watch(view, "hero.charts", draw);
	return view;
});
