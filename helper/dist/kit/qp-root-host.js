// RootHost: the window's frame (docs/ui.md, "The window model"): the five roots, Pump, Hero, Book, Log and Games (the
// desktop's fifth, slice 56; the phones keep four), each with its own stack of pages, the Menu's stack over the root it
// was opened from, and the ActionBar at the foot under 720 CSS px wide or as the left rail from there; a pump covers
// the whole window, the bar with it. lib/router.js decides what each stack holds; this element holds the parts. The
// stacks keep their pages' elements, the top one shown.

import {KitElement, define, h} from "./element.js";

/** Each root's face, in the ActionBar's order: its glyph (an art piece) and its name's key, which its tab and its root
 * page's heading show. */
export const ROOT_FACES = {
	pump: {glyph: "action_pump", name: "bar.pump"},
	hero: {glyph: "action_character", name: "bar.hero"},
	book: {glyph: "action_book", name: "bar.book"},
	log: {glyph: "action_log", name: "bar.log"},
	games: {glyph: "action_games", name: "bar.games"},
};

export const ROOTS = Object.keys(ROOT_FACES);

class QpRootHost extends KitElement {
	build() {
		this.bar = h("qp-action-bar");
		this.stacks = Object.fromEntries(ROOTS.map((root) => [root, h("section", {class: "qp-stack", "data-root": root})]));
		this.menu = h("section", {class: "qp-stack", "data-menu": "", hidden: true});
		this.cover = h("section", {class: "qp-cover", hidden: true});
		this.stage = h("main", {class: "qp-stage"}, ...Object.values(this.stacks), this.menu);
		this.append(this.bar, this.stage, this.cover);
	}
}

define("qp-root-host", QpRootHost);
