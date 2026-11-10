// Work mode's pages on the computer and in the browser (docs/plan.md slice 67; screens.json desk.work.sheet,
// desk.work.how and desk.menu.work; Android's ui/WorkCard.kt is their twin), from the `work` and `menu.work` topics
// (helper/crates/questpump-desk/src/work.rs) and the sheet's words in the `places` topic:
// - the first Start's sheet (desk.work.sheet), over whatever page shows while the `work` topic's `sheet` holds (the
//   first Start on Pump home's work card or in the tray asks for it): Work mode in its banner with its (i) before the
//   close, the strip at the Every chosen, Every, Where you train's three switches (work mode's own equipment, the
//   office template's at first; David, 2026-11-09: "just have an extra equipment setting for work") and the gold
//   Start, which turns work mode on with them (`work.start {confirm, every, floor, space, quiet}`); its × and Escape
//   put it away (`work.close`), and it closes whenever the topic says it is put away;
// - How work mode works (desk.work.how), from the (i): the strip and five numbered lines at the player's choices, the
//   short pump's minutes by the game's estimate of a move (`menu.work`'s `minutes`), the fifth what this computer
//   waits for (the browser's: quiet hours alone);
// - the Work mode page (desk.menu.work), the Menu's Work mode row in Your pumps, which the place button and the place
//   rows open while work mode runs: the (i) on its bar, Every and Moves per block (each a `work.set`), then work mode's
//   own equipment as a place's window draws a setup (Where you train's switches, each a `work.set`, and Equipment's
//   chips in their groups with their Tooltips, each a `work.gear`), every answer the topic brings; no switch and no
//   hours (Start and Stop are the work card's and the tray's).

import {get, intent, subscribe} from "../../lib/bridge.js";
import {t} from "../../lib/copy.js";
import {register, whenStarted} from "../../lib/router.js";
import {h} from "../../kit/element.js";
import {PLACE, pageTooltip, placeSwitches, setupParts} from "../menu/setup.js";
import {page, watch} from "../page.js";

/** Moves per block's paces, each with its words (Moves per match's, views/games/games.js). */
const PACES = [
	{value: "light", label: () => t("games.pace.light")},
	{value: "balanced", label: () => t("games.pace.balanced")},
	{value: "extreme", label: () => t("games.pace.extreme")},
];

/** An intent's failure goes to the console (Rust's handler logged why). */
function heard(promise) {
	promise.catch(console.error);
}

/** Every's lengths as a Segmented's options, from `menu.work`'s choices. */
function everyOptions(m) {
	return m.choices.map((minutes) => ({value: String(minutes), label: t("work.every.value", {minutes})}));
}

/** How work mode works (desk.work.how) at Every's `every` minutes (the saved Every without one). */
export function openHow(every) {
	const m = get("menu.work");
	if (!m) return;
	const minutes = every ?? m.every;
	const web = get("shell")?.platform === "web";
	const lines = [
		t("work.how.clock", {minutes}),
		m.short != null && m.minutes != null ? t("work.how.pump", {moves: m.short, minutes: String(m.minutes)}) : null,
		m.long != null ? t("work.how.long", {moves: m.long}) : null,
		t("work.how.bank"),
		web ? t("work.how.waits.web") : t("work.how.waits"),
	].filter(Boolean);
	const win = h("qp-window", {heading: t("work.how.title")});
	win.content.append(h("qp-work-strip", {every: minutes, blocks: m.blocks}),
		h("qp-card", {rows: true}, ...lines.map((line, i) => h("qp-row", {step: i + 1, heading: line, trailing: "shown"}))));
	win.addEventListener("qp-closed", () => win.remove(), {once: true});
	win.open();
}

/** The first Start's sheet (desk.work.sheet), its choices from `menu.work` until the player changes them; Where you
 * train's switches once the sheet's words have come (the `places` topic's). */
function openSheet() {
	const chosen = {every: null, floor: null, space: null, quiet: null};
	const win = h("qp-window", {heading: t("work.title"), primary: t("work.start")});
	win.aside.append(h("qp-round-button", {icon: "how", "on:click": () => openHow(chosen.every)}));
	const strip = h("qp-work-strip");
	const every = h("qp-segmented", {label: t("work.every"), "on:qp-change": (e) => {
		chosen.every = Number(e.detail);
		draw();
	}});
	let switches = null;
	win.content.append(strip, h("qp-section-label", {label: t("work.every")}), every);
	function draw() {
		const m = get("menu.work");
		if (!m) return;
		chosen.every ??= m.every;
		for (const key of PLACE) chosen[key] ??= m[key];
		strip.every = chosen.every;
		strip.blocks = m.blocks;
		every.options = everyOptions(m);
		every.value = String(chosen.every);
		const words = get("places")?.words;
		if (!switches && words) {
			switches = placeSwitches(words, (key, on) => {
				chosen[key] = on;
				draw();
				return Promise.resolve();
			});
			win.content.append(h("qp-section-label", {label: t("menu.free.place")}), switches.card);
		}
		switches?.draw(chosen);
	}
	const stops = [subscribe("menu.work", draw), subscribe("places", draw)];
	win.addEventListener("qp-primary", () => {
		heard(intent("work.start", {confirm: true, every: chosen.every, floor: chosen.floor, space: chosen.space,
			quiet: chosen.quiet}));
		win.close();
	});
	win.addEventListener("qp-dismiss", () => heard(intent("work.close")));
	win.addEventListener("qp-closed", () => {
		for (const stop of stops) stop();
		win.remove();
		sheet = null;
	}, {once: true});
	win.open();
	return win;
}

let sheet = null; // the first Start's sheet while it shows

whenStarted(() => subscribe("work", (work) => {
	if (work?.sheet && !sheet) sheet = openSheet();
	else if (!work?.sheet && sheet) sheet.close();
}));

register("desk.menu.work", () => {
	const view = page({heading: t("work.title")});
	view.bar.trailing = "how";
	view.bar.addEventListener("qp-trailing", () => openHow());
	const set = (args) => intent("work.set", args);
	const every = h("qp-segmented", {label: t("work.every"),
		"on:qp-change": (e) => heard(set({every: Number(e.detail)}))});
	const pace = h("qp-segmented", {label: t("work.pace"), "on:qp-change": (e) => heard(set({pace: e.detail}))});
	pace.options = PACES.map((p) => ({value: p.value, label: p.label()}));
	view.body.append(h("qp-section-label", {label: t("work.every")}), every,
		h("qp-section-label", {label: t("work.pace")}), pace);
	const tooltip = pageTooltip(view);
	let equipment = null;
	let shown = null;
	const draw = () => {
		if (!shown) return;
		every.options = everyOptions(shown);
		every.value = String(shown.every);
		pace.value = (PACES.find((p) => p.value === shown.pace) ?? PACES[1]).value;
		// work mode's own equipment once the sheet's words have come, each change kept at once
		const words = get("places")?.words;
		if (!equipment && words) {
			equipment = setupParts(words, tooltip, (key, on) => set({[key]: on}),
				(key, on) => heard(intent("work.gear", {key, on})));
			view.body.append(...equipment.nodes);
		}
		equipment?.draw(shown);
	};
	watch(view, "menu.work", (m) => {
		shown = m;
		draw();
	});
	watch(view, "places", draw);
	view.primary = () => every;
	return view;
});
