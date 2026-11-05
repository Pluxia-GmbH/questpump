// Menu: a list of choices that opens from its button (2026-11-05: a pump's Pause on its Ready view, as the game's
// ns.Menu, Kit.OpenMenu, opened above its button): a box with no dim above its owner (under it where the window has no
// room above), or beside the choice that opened it as a submenu; one role="menu" named by its `label`, its `items` in
// order: a group's title ({group}: caps words, a rule before every group but the first), and each choice ({label, act,
// hint?, destructive?}) a role="menuitem" whose click closes the menu (and the menu it opened from) and then acts, or
// one with a submenu ({label, submenu: {label, items}}) that opens that menu beside itself. The keys WAI-ARIA's menu
// pattern names (APG, "Menu Button" and "Menu and Menubar"): Up and Down move between the choices, round, Home and End
// to the first and the last, Enter or Space take one (its submenu's first choice on one that has a submenu), Right
// opens a choice's submenu on its first choice, Left or Escape closes a submenu back to its choice, Escape or a click
// outside the box closes the menu, Tab closes every menu; the focus goes back to the control that opened it. Its owner
// is read as a menu button (qp-secondary-button's `menu`), open or not (`aria-expanded`). A native <dialog> opened with
// showModal, so it stands over the page as a Window does (lib/focus.js `topWindow`): the tabs wait, the keys' half-second
// guard runs as it opens, and a pump's move hears it (`pump.window`). A click in that guard time after it opens does
// nothing, so a double click on its button neither closes it again nor takes the choice that came under the pointer
// (a pump's I feel unwell ends the pump and locks pumps until 04:00); it never covers its button, scrolling where the
// window has room for it on neither side. A click on its box outside a choice keeps its keys (the box takes the focus).
// Fires qp-closed once closed, then leaves the page.

import {guard, guardMs} from "../lib/keys.js";
import {KitElement, define, emit, h, token} from "./element.js";

/** The keys that move between the choices: where each goes from the choice at `i` of `n`. */
const MOVES = {
	ArrowDown: (i, n) => (i + 1) % n,
	ArrowUp: (i, n) => (i - 1 + n) % n,
	Home: () => 0,
	End: (_, n) => n - 1,
};

class QpMenu extends KitElement {
	build() {
		this.list = h("div", {class: "qp-menu-list", role: "menu"});
		this.dialog = h("dialog", {class: "qp-menu", tabindex: "-1"}, this.list);
		this.dialog.addEventListener("cancel", (e) => {
			e.preventDefault();
			this.close();
		});
		// a click outside the box lands on the dialog itself (a modal dialog's ::backdrop is its own)
		this.dialog.addEventListener("click", (e) => {
			if (e.target !== this.dialog || !this.settled()) return;
			const box = this.dialog.getBoundingClientRect();
			const out = e.clientX < box.left || e.clientX > box.right || e.clientY < box.top || e.clientY > box.bottom;
			if (out) this.close({all: true});
		});
		this.dialog.addEventListener("keydown", (e) => this.onKey(e));
		this.parentMenu = null; // the menu a submenu opened from
		this.sub = null; // the submenu open from this one
		this.append(this.dialog);
	}

	update() {
		this.list.setAttribute("aria-label", this.label ?? "");
		this.dialog.setAttribute("aria-label", this.label ?? "");
	}

	/** Its choices and its groups' titles, in order (the module's doc). */
	set items(items) {
		this.ensure();
		const parts = [];
		let group = null;
		for (const item of items) {
			if (item.group) {
				if (parts.length) parts.push(h("div", {class: "qp-menu-rule", role: "separator"}));
				group = h("div", {role: "group", "aria-label": item.group},
					h("div", {class: "qp-menu-title qp-type-caps", "aria-hidden": "true", text: item.group}));
				parts.push(group);
				continue;
			}
			const choice = this.choice(item);
			if (group) group.append(choice);
			else parts.push(choice);
		}
		this.list.replaceChildren(...parts);
	}

	/** One choice: its words, what it does read after them (`hint`), the stop colour where it stops (`destructive`),
	 * and the chevron of a choice that opens a submenu. */
	choice(item) {
		const el = h("button", {type: "button", class: "qp-menu-item", role: "menuitem", tabindex: "-1"},
			h("span", {class: "qp-menu-words", text: item.label}));
		el.classList.toggle("qp-destructive", Boolean(item.destructive));
		if (item.hint) el.setAttribute("aria-description", item.hint);
		if (item.submenu) {
			el.setAttribute("aria-haspopup", "menu");
			el.setAttribute("aria-expanded", "false");
			el.append(h("span", {class: "qp-end-nav", "aria-hidden": "true"}));
			el.addEventListener("click", () => this.settled() && this.openSub(el, item.submenu));
		} else {
			el.addEventListener("click", () => {
				if (!this.settled()) return;
				this.close({all: true});
				item.act();
			});
		}
		return el;
	}

	/** The choices that take the focus, in order. */
	choices() {
		return [...this.list.querySelectorAll("[role=menuitem]")];
	}

	/** Opens it over the page from `owner` (its button, or the choice a submenu opens from, `parent` its menu): above
	 * the owner, else under it, its leading edge on the owner's, or beside the choice for a submenu; the focus on its
	 * first choice. */
	open(owner, parent = null) {
		this.ensure();
		if (!this.isConnected) document.body.append(this);
		this.update();
		this.owner = owner;
		this.parentMenu = parent;
		this.openedAt = performance.now();
		this.dialog.showModal();
		this.place();
		this.expanded(true);
		guard();
		this.choices()[0]?.focus();
	}

	/** Whether its guard time since it opened passed (`guardMs`): a click before it does nothing. */
	settled() {
		return performance.now() - this.openedAt >= guardMs();
	}

	/** Its box placed by its owner's, inside the window by the margin: a menu above its button, else under it, else on
	 * the side with more room at that room's height, scrolling, so it never covers its button. */
	place() {
		const at = (this.owner.control ?? this.owner).getBoundingClientRect();
		const margin = token(this, "--qp-layout-space8");
		this.dialog.style.removeProperty("max-height");
		let box = this.dialog.getBoundingClientRect();
		const room = (size, most) => Math.max(margin, Math.min(size, most - margin));
		let left;
		let top;
		if (this.parentMenu) {
			const right = at.right + margin / 2;
			left = right + box.width <= innerWidth - margin ? right : at.left - margin / 2 - box.width;
			top = at.top;
		} else {
			left = at.left;
			const above = at.top - margin / 2 - margin;
			const below = innerHeight - margin - at.bottom - margin / 2;
			const up = box.height <= above || (box.height > below && above >= below);
			if (box.height > (up ? above : below)) {
				this.dialog.style.setProperty("max-height", `${Math.max(0, up ? above : below)}px`);
				box = this.dialog.getBoundingClientRect();
			}
			top = up ? at.top - margin / 2 - box.height : at.bottom + margin / 2;
		}
		this.dialog.style.setProperty("--qp-menu-x", `${room(left, innerWidth - box.width)}px`);
		this.dialog.style.setProperty("--qp-menu-y", `${room(top, innerHeight - box.height)}px`);
	}

	/** Its owner says whether the menu is open. */
	expanded(open) {
		(this.owner?.control ?? this.owner)?.setAttribute("aria-expanded", String(open));
	}

	/** Opens the submenu `spec` beside the choice `el`. */
	openSub(el, spec) {
		this.sub?.close({focus: false});
		const sub = h("qp-menu", {label: spec.label});
		sub.items = spec.items;
		sub.addEventListener("qp-closed", () => {
			if (this.sub === sub) this.sub = null;
		}, {once: true});
		this.sub = sub;
		sub.open(el, this);
	}

	onKey(e) {
		if (e.isComposing) return;
		const choices = this.choices();
		const at = choices.indexOf(document.activeElement);
		const move = MOVES[e.key];
		if (move && choices.length) {
			e.preventDefault();
			// from the box itself (no choice has the focus) Down goes to the first and Up to the last
			const from = at >= 0 ? at : e.key === "ArrowUp" ? 0 : -1;
			choices[move(from, choices.length)]?.focus();
		} else if (e.key === "ArrowRight" && choices[at]?.getAttribute("aria-haspopup") === "menu") {
			e.preventDefault();
			choices[at].click();
		} else if (e.key === "ArrowLeft" && this.parentMenu) {
			e.preventDefault();
			this.close();
		} else if (e.key === "Tab") {
			e.preventDefault();
			this.close({all: true});
		}
	}

	/** Closes it (with `all` every menu it opened from too), its submenu first; the focus back on what opened it
	 * (`focus`), then qp-closed, and it leaves the page. */
	close({all = false, focus = true} = {}) {
		if (!this.dialog.open) return;
		this.sub?.close({focus: false});
		this.dialog.close();
		this.expanded(false);
		const parent = this.parentMenu;
		if (all && parent) {
			parent.close({all: true, focus});
		} else if (focus) {
			(this.owner?.control ?? this.owner)?.focus?.({preventScroll: true});
		}
		emit(this, "qp-closed");
		this.remove();
	}
}

define("qp-menu", QpMenu, {label: String});
