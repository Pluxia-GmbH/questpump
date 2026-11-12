// Hero on the computer (docs/ui.md, "Hero, the Book and the Log"; slice 30's Hero, iOS SheetTab, Android's twin), from
// the `hero` topic (helper/crates/questpump-desk/src/hero.rs): the header beside the Portrait (the name with its title in the
// game's grammar, on the app's own sheet with a title earned a button that opens the title window; the level line that
// opens the level's page; the beast), the chips row (whose sheet it is, the game's with its age or the app's own pumps,
// which opens This computer; the focus, which opens the focus window; the streak while one runs, which opens the
// streak page since slice 51), the body map taking
// the room the blocks leave, each trained muscle in its quality's tone (hover or focus shows its Tooltip, a click or
// Enter opens its page), the stat strip, each stat opening its page, and the weekly vault's card (2026-10-09: the
// summary's va, the game's or the app's own). Before the first pump: QP 1, "Your hero", the dark map, whose muscles
// still open their pages, and Start a pump in the strip's place, which follows the pump's gate as Pump home's foot does
// (the `pump` topic's `gate` and the `shell` topic's engine): busy while the engine folds, else disabled with the
// gate's words (the lock's rest line, the game asking the pumps, or pumps that could not start beside the Menu's report
// for help).

import {intent} from "../../lib/bridge.js";
import {still} from "../../lib/carry.js";
import {t} from "../../lib/copy.js";
import {LOCALE, time} from "../../lib/format.js";
import {isFront, openMenu, push, register, registered} from "../../lib/router.js";
import {h} from "../../kit/element.js";
import {reportButton} from "../menu/menu.js";
import {page, watch, words} from "../page.js";
import {focusGlyph, focusWindow} from "./focus.js";
import {ago, muscleName, nameParts, recordText} from "./words.js";

/** The body map's least height: the figure still large enough to point at a muscle (iOS HeroMap.least); above it the
 * map takes the room the blocks leave, its figure drawn to its own proportions. */
const MAP_LEAST = 280;

/** A level's point on the XP bar. */
function pointOf(level) {
	return {level: level.level, into: level.into, need: level.need, paragon: level.paragon};
}

/** Whether the XP at `now` is past the XP at `was` (a level, a rank or the XP into the same one): a bar is carried
 * home only forward. */
function grew(was, now) {
	if (now.paragon !== was.paragon) return now.paragon > was.paragon;
	if (now.level !== was.level) return now.level > was.level;
	return now.into > was.into;
}

/** The level line: the gold badge, the Paragon rank past the cap, the hero's XP bar and the XP into the level. */
function levelLine() {
	const badge = h("span", {class: "qp-level-badge", "aria-hidden": "true"});
	const paragon = h("span", {class: "qp-type-label", "aria-hidden": "true"});
	const bar = h("qp-xp-bar", {bare: true, hero: true});
	const numbers = h("span", {class: "qp-type-meta", "aria-hidden": "true"});
	const control = h("button", {type: "button", class: "qp-level-line"}, badge, paragon, bar, numbers);
	control.addEventListener("click", () => push("desk.hero.stat", {level: true}));
	/** The level beside the bar, as the point `at` names it: the badge, the Paragon rank, the XP into it. */
	const name = (at) => {
		badge.textContent = new Intl.NumberFormat(LOCALE).format(at.level);
		paragon.hidden = !at.paragon;
		paragon.textContent = at.paragon ? t("kit.fanfare.paragon", {rank: at.paragon}) : "";
		numbers.textContent = t("kit.xp.progress", {into: at.into, need: at.need});
	};
	let drawn = null;
	// a bar carried through a level or a rank began the new one: its name turns with it
	bar.addEventListener("qp-restart", () => drawn && name(drawn));
	return {
		el: control,
		draw(level, open) {
			drawn = pointOf(level);
			name(drawn);
			bar.rested = level.rested;
			bar.land(drawn);
			// the level's page reads the summary: nothing to open before the first pump
			control.disabled = !open;
		},
		/** The bar fills once from `from`, the point Hero last showed, to the level drawn (a reward carried home),
		 * through a level or a rank by its flash and its start from empty; until that start the badge, the rank and
		 * the numbers stay the old level's, as the phones hold them. */
		carry(from, level) {
			const to = pointOf(level);
			bar.carry(from, to);
			if (!still.matches && (to.level > from.level || to.paragon > from.paragon)) name(from);
		},
	};
}

/** Quick pump as the pump's gate stands, as Pump home's foot follows it (Hero's before the first pump, and the empty
 * Log's primary; slice 51, Rapha, 2026-10-21: Start a pump left them): none, busy while the engine folds, else disabled with the gate's words under it (the lock's rest line,
 * the game asking the pumps, or pumps that could not start beside the Menu's report for help). Its `el`, and `start`,
 * the button a view's keys press. */
export function startPump(view) {
	const start = h("qp-primary-button", {label: t("pump.quick"), glyph: "action_pump",
		"on:click": () => intent("pump.quick").catch(console.error)});
	const held = words("", "meta");
	const report = reportButton();
	let gate = null;
	let lockedUntil = null;
	let engine = "busy";
	const draw = () => {
		const failed = gate === "engine" && engine === "failed";
		const folding = gate === "engine" && !failed;
		start.state = !gate ? "normal" : folding ? "busy" : "disabled";
		held.hidden = !gate || folding;
		held.textContent = gate === "game" ? t("pump.waiting.game")
			: gate === "lock" ? t("pump.rest.line", {time: time(lockedUntil)}) : failed ? t("desk.home.failed") : "";
		report.el.hidden = !failed;
	};
	watch(view, "pump", (pump) => {
		gate = pump.gate ?? null;
		lockedUntil = pump.locked_until ?? null;
		draw();
	});
	watch(view, "shell", (shell) => {
		engine = shell.engine;
		draw();
	});
	view.stops.push(report.stop);
	return {el: h("div", {class: "qp-stack-small"}, start, held, report.el), start};
}

/** The title window (iOS TitleWindow), on the app's own sheet alone: "Your title", then No title and every
 * title the app's own fold holds earned, in the deeds' order, each led by the title reward's glyph, the one shown
 * checked; a click keeps the choice on this computer (`hero.title`), whose summary then names it beside the hero's
 * name, and closes; the close keeps nothing. A game sheet's title is the game's choice. */
function titleWindow() {
	const box = h("qp-window", {heading: t("character.title.title")});
	const rows = h("qp-card", {rows: true});
	box.content.append(rows);
	const row = (name, key, checked) => {
		const choice = h("qp-row", {heading: name, glyph: "reward_title", trailing: "checked", checked});
		choice.addEventListener("click", () => {
			intent("hero.title", {key}).catch(console.error);
			box.close();
		});
		return choice;
	};
	return {
		open(hero) {
			rows.replaceChildren(row(t("character.title.none"), null, !hero.title_key),
				...hero.titles.map((title) => row(title.name, title.key, title.key === hero.title_key)));
			box.open();
		},
	};
}

register("hero", () => {
	const view = page({heading: t("bar.hero"), root: true});
	view.el.classList.add("qp-hero");
	const name = h("h2", {class: "qp-hero-name qp-type-display-small"});
	// the name line as a button, its chevron at its end: the title window on the app's own sheet with a title earned
	const nameButton = h("button", {type: "button", class: "qp-name-line", "on:click": () => hero && titles.open(hero)});
	const level = levelLine();
	const beast = words("", "meta");
	const header = h("div", {class: "qp-hero-head"}, h("qp-portrait"),
		h("div", {class: "qp-hero-lines"}, name, level.el, beast));
	// where the sheet comes from, opening This computer where it stands (a plain chip in the browser, which has none)
	const opens = registered("desk.menu.computer");
	const source = h("qp-chip", {button: opens, state: "source",
		"on:click": () => opens && openMenu("desk.menu.computer")});
	const focus = h("qp-chip", {button: true, state: "focus"});
	const streak = h("qp-chip", {button: true, state: "plain", glyph: "mark_check", hint: t("pump.streak.open"),
		"on:click": () => push("desk.streak")});
	const map = h("qp-muscle-map", {mode: "hero", label: t("character.map.label"), "data-hero": ""});
	const tooltip = h("qp-tooltip");
	const strip = h("qp-stat-strip");
	const vault = h("qp-vault-card");
	// Start a pump before the first pump, with why it waits in Pump home's words
	const begin = startPump(view);
	// what the map's shades say: each muscle in its best step's quality (2026-10-29)
	const column = h("qp-hero-column", {least: MAP_LEAST}, header,
		h("div", {class: "qp-chips"}, source, focus, streak), h("qp-section-label", {label: t("character.map.title")}), map,
		strip, vault, begin.el);
	view.body.append(column, tooltip);
	const choose = focusWindow((id) => intent("hero.focus", {focus: id}).catch(console.error));
	const titles = titleWindow();
	let hero = null;

	const drawSource = () => {
		if (!hero) return;
		source.glyph = hero.source.kind === "game" ? "link_game" : "action_pump";
		source.label = hero.source.kind === "game" ? t("desk.hero.source.game", {ago: ago(hero.source.at, Date.now())})
			: t("character.source.app");
	};
	// the sheet's age reads the clock at the minute, as the phones' header does
	const minute = setInterval(drawSource, 60_000);
	view.stops.push(() => clearInterval(minute));

	focus.addEventListener("click", () => hero && choose.open(hero.focuses,
		hero.focuses.find((f) => f.key === hero.focus)?.id));
	map.addEventListener("qp-muscle", (e) => push("desk.hero.muscle", {id: e.detail.id}));
	map.addEventListener("qp-muscle-hover", (e) => {
		const muscle = hero?.muscles.find((m) => m.id === e.detail.id);
		if (!muscle) return;
		Object.assign(tooltip, {name: muscleName(muscle), quality: muscle.quality, qualityWord: muscle.word ?? "",
			untrained: !muscle.quality});
		tooltip.meta = muscle.move ? (muscle.record > 0 ? t("character.tooltip.record",
			{move: muscle.move.name, record: recordText(muscle.record, muscle.seconds)}) : muscle.move.name)
			: muscle.quality && muscle.week !== null ? t("character.muscle.week", {count: muscle.week}) : null;
		tooltip.showAt(e.detail.el);
	});
	// the pointer may cross onto the Tooltip (WCAG 1.4.13); the focus leaving takes it at once
	map.addEventListener("pointerleave", () => tooltip.leave());
	map.addEventListener("focusout", () => tooltip.hide());
	// Escape hides it too while it shows, the focus kept on its muscle
	view.onKey = (e) => tooltip.escape(e);
	strip.addEventListener("qp-open", (e) => push("desk.hero.stat", {stat: e.detail}));

	// what Hero showed while it was last in front (slice 72, a reward carried home): the next time it shows, its XP bar
	// fills once from there and the muscles whose tone changed since fade up; the page's own memory, never replayed
	let seen = null;
	// the `hero` it last drew in front: while Hero is not in front (a pump's cover over it, whose floating Result leaves
	// it showing) the level and the muscles' tones stay as it last showed, so each moves once as Hero is in front
	// again and never before
	let fronted = null;
	const tonesOf = (from) => Object.fromEntries(from.muscles.filter((m) => m.quality > 0)
		.map((m) => [m.id, m.quality]));
	const drawCarried = () => {
		if (!fronted || isFront(view)) fronted = hero;
		level.draw(fronted.level, hero.summary);
		map.tones = tonesOf(fronted);
	};
	const carry = () => {
		if (!hero) return;
		const now = {level: pointOf(hero.level), tones: tonesOf(hero)};
		const was = seen;
		seen = now;
		if (!was) return;
		if (grew(was.level, now.level)) level.carry(was.level, hero.level);
		const lit = Object.keys(now.tones).map(Number).filter((id) => now.tones[id] !== was.tones[id]);
		if (lit.length) map.fade(lit);
	};
	view.onShown = () => {
		if (hero) drawCarried();
		carry();
	};
	// a bar still on its way as Hero leaves the front stands at its end
	view.onLeft = () => fronted && level.draw(fronted.level, hero.summary);

	watch(view, "hero", (value) => {
		hero = value;
		const parts = nameParts(hero.name, hero.title, hero.grammar);
		const lines = [h("span", {text: parts.first}), parts.second ? h("span", {text: parts.second}) : ""];
		if (hero.own && hero.summary && hero.titles.length) {
			nameButton.replaceChildren(...lines, h("span", {class: "qp-end-nav", "aria-hidden": "true"}));
			nameButton.setAttribute("aria-label", parts.whole);
			if (nameButton.parentNode !== name) name.replaceChildren(nameButton);
			name.removeAttribute("aria-label");
		} else {
			name.replaceChildren(...lines);
			name.setAttribute("aria-label", parts.whole);
		}
		drawCarried();
		beast.hidden = !hero.beast;
		beast.textContent = hero.beast ?? "";
		drawSource();
		const named = hero.focuses.find((f) => f.key === hero.focus);
		focus.glyph = focusGlyph(hero.focus);
		focus.label = t("character.meta.focus", {focus: named?.name ?? hero.focus});
		streak.hidden = !hero.streak;
		if (hero.streak) {
			streak.label = hero.streak.tokens > 0 ? t("character.streak.tokens", {count: hero.streak.tokens,
				days: new Intl.NumberFormat(LOCALE).format(hero.streak.days)}) : t("character.streak", {days: hero.streak.days});
		}
		map.figure = hero.figure;
		map.names = Object.fromEntries(hero.muscles.map((m) => [m.id, muscleName(m)]));
		// a tooltip of colours that changed (a sync, a pump) never waits there
		tooltip.hide();
		strip.hidden = !hero.summary;
		strip.cells = hero.stats.map((s) => ({glyph: `stat_${s.key}`, value: new Intl.NumberFormat(LOCALE).format(s.value),
			name: s.name}));
		vault.vault = hero.summary ? hero.vault ?? null : null;
		begin.el.hidden = hero.summary;
		// the first values are what the page starts from; later ones move only while it is in front
		if (!seen || isFront(view)) carry();
	});
	view.primary = () => (hero?.summary ? null : begin.start);
	view.stops.push(() => choose.remove());
	return view;
});
