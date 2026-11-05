// RoundButton: a 40 circle (36 in a pump, `small`) of card with a line edge and its glyph in text2, named for a screen
// reader: the gear that opens the Menu, a page's way back, a pump's other choices ("···"), a window's close, Share and
// the coach's flag, the streak page's earlier and later month (slice 51), a Carousel's previous and next (a chevron
// each way), the Book's search (Moves, 2026-11-04) and Pump home's place button (2026-11-03: the place in use's own
// glyph, contract/places.json `glyphs`), which `spoken` alone names; `spoken` names the others where their glyph's name
// says too little (the Saga's Week tile's Share week). Hover lifts it, a press darkens it.

import {t} from "../lib/copy.js";
import {KitElement, define, h} from "./element.js";

const ICONS = {
	menu: {glyph: "action_menu", label: () => t("bar.menu")},
	back: {label: () => t("kit.back")},
	more: {label: () => t("kit.more")},
	close: {glyph: "mark_close", label: () => t("common.close")},
	share: {glyph: "mark_share", label: () => t("kit.share")},
	search: {glyph: "mark_search", label: () => t("ladders.search")},
	report: {glyph: "mark_report", label: () => t("coach.report.label")},
	earlier: {label: () => t("pump.streak.month.earlier")},
	later: {label: () => t("pump.streak.month.later")},
	previous: {},
	next: {},
	place_home: {glyph: "place_home"},
	place_bed: {glyph: "place_bed"},
	place_dumbbell: {glyph: "place_dumbbell"},
	place_bars: {glyph: "place_bars"},
	place_desk: {glyph: "place_desk"},
	place_peak: {glyph: "place_peak"},
};

/** The glyphs drawn in CSS: the chevron (its way by the button's icon) and the three dots. */
const DRAWN = {
	back: "qp-chevron", earlier: "qp-chevron", later: "qp-chevron", previous: "qp-chevron", next: "qp-chevron",
	more: "qp-dots",
};

class QpRoundButton extends KitElement {
	build() {
		this.control = h("button", {type: "button", class: "qp-round"});
		this.append(this.control);
	}

	update() {
		const icon = ICONS[this.icon];
		if (!icon) return;
		const name = this.spoken || icon.label?.();
		if (name) this.control.setAttribute("aria-label", name);
		else this.control.removeAttribute("aria-label");
		if (this.shown === this.icon) return;
		this.shown = this.icon;
		this.control.dataset.icon = this.icon;
		this.control.replaceChildren(icon.glyph ? h("qp-art", {name: icon.glyph})
			: h("span", {class: DRAWN[this.icon], "aria-hidden": "true"}));
	}
}

define("qp-round-button", QpRoundButton, {icon: String, small: Boolean, spoken: String});
