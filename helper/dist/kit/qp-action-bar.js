// ActionBar: the five tabs (design v3 §3; Games the desktop's fifth, slice 56): a glyph over its caps label each, the
// chosen one gold with its 26 × 3 gold marker and its glow on the bar's gold-fading top edge, the others text3; a gold
// dot on a tab's glyph while its root has something waiting (`mark`: Pump's unfinished moves, the games found since the
// Games tab was last opened), which a screen reader hears after the tab's name. A navigation landmark of buttons, the
// chosen one the current page. In a window 720 CSS px wide or more it is the left rail (its rail state, desk.css): the
// same glyphs, labels and gold marker in a column. A tab fires qp-tab with its root, pressed by a click or by `press`
// (Cmd/Ctrl+1 to 5, the macOS View menu; lib/router.js).

import {t} from "../lib/copy.js";
import {KitElement, define, emit, h} from "./element.js";
import {ROOTS, ROOT_FACES} from "./qp-root-host.js";

class QpActionBar extends KitElement {
	build() {
		this.tabs = Object.fromEntries(ROOTS.map((root) => {
			const button = h("button", {type: "button", class: "qp-tab", "on:click": () => emit(this, "qp-tab", root)},
				h("qp-art", {name: ROOT_FACES[root].glyph}),
				h("span", {class: "qp-tab-label", text: t(ROOT_FACES[root].name)}));
			return [root, button];
		}));
		this.dots = Object.fromEntries(ROOTS.map((root) => [root, h("span", {class: "qp-tab-dot", "aria-hidden": "true"})]));
		this.marks = {};
		this.append(h("nav", {"aria-label": t("desk.shell.tabs")}, ...Object.values(this.tabs)));
	}

	/** Puts the dot on `root`'s tab with `said`, the words a screen reader hears after the tab's name ("Pump, 3
	 * unfinished"), or takes it away (`said` null). */
	mark(root, said) {
		this.ensure();
		this.marks[root] = said || null;
		this.update();
	}

	/** Presses `root`'s tab as a click does (lib/sound.js's tab sound on a new tab, qp-tab). */
	press(root) {
		this.ensure();
		this.tabs[root]?.click();
	}

	update() {
		for (const [root, button] of Object.entries(this.tabs)) {
			const chosen = root === this.selected;
			button.classList.toggle("qp-chosen", chosen);
			if (chosen) button.setAttribute("aria-current", "page");
			else button.removeAttribute("aria-current");
			const said = this.marks[root];
			if (said) {
				button.append(this.dots[root]);
				button.setAttribute("aria-label", `${t(ROOT_FACES[root].name)}, ${said}`);
			} else {
				this.dots[root].remove();
				button.removeAttribute("aria-label");
			}
		}
	}
}

define("qp-action-bar", QpActionBar, {selected: String});
