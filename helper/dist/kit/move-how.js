// How and Tips, the one way they are drawn (docs/ui.md, "The pump on the computer": Ready shows the move up next in
// full, 2026-10-24; the game's Ready view's How and Tips, Panel.lua `fillGuide`): the move's setup line under the
// character's glyph, How's steps numbered from 1 (each step one item read with its number), and Tips' lines, the move's
// own advice last (Rust's `Facts::tips`). Ready, a pump's move window and a move's page in the Book draw them through
// these, and those and a step's card the stop line, a line of its own with no glyph (2026-10-25, David: "remove the
// heart. it slooks too extreme").

import {t} from "../lib/copy.js";
import {h} from "./element.js";

/** How's rows: the setup line under its glyph (none without one), then the steps as a numbered list. */
export function howSteps(setup, steps = []) {
	const rows = [];
	if (setup) {
		rows.push(h("p", {class: "qp-how-setup"}, h("qp-art", {name: "action_character"}),
			h("span", {class: "qp-type-body", text: setup})));
	}
	if (steps.length) {
		// role="list": a list drawn without its markers keeps its list in WebKit's accessibility tree
		rows.push(h("ol", {class: "qp-how-steps", role: "list"}, ...steps.map((step, i) => h("li", {},
			h("span", {class: "qp-how-number qp-type-num-row", text: String(i + 1)}),
			h("span", {class: "qp-type-body", text: step})))));
	}
	return rows;
}

/** Tips' lines as a list; none without one. */
export function tipList(tips = []) {
	if (!tips.length) return [];
	return [h("ul", {class: "qp-how-tips", role: "list"},
		...tips.map((tip) => h("li", {class: "qp-type-body", text: tip})))];
}

/** How under its caps heading, `action` at the heading's end (Ready's read-aloud button), then Tips under its own; a
 * part with no lines shows nothing. */
export function howAndTips({setup = null, how = [], tips = [], action = null}) {
	const parts = [];
	const steps = howSteps(setup, how);
	if (steps.length) {
		const head = h("div", {class: "qp-how-head"}, h("qp-section-label", {label: t("kit.move.how")}), action);
		parts.push(head, ...steps);
	}
	const tipped = tipList(tips);
	if (tipped.length) parts.push(h("qp-section-label", {label: t("kit.move.tips")}), ...tipped);
	return parts;
}

/** The move's stop line; none without one. */
export function stopLine(stop) {
	return stop ? h("p", {class: "qp-type-body qp-move-stop", text: stop}) : null;
}

/** A pose's goal pose under the check (its own mark's sentence); none without one. */
export function markLine(mark) {
	return mark ? h("p", {class: "qp-how-setup"}, h("qp-art", {name: "mark_check"}),
		h("span", {class: "qp-type-body", text: mark})) : null;
}
