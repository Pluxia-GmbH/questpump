// Tooltip: a blue-black box with a cool edge and a shadow: a name, its QualityWord and one meta line (none where it has
// none to say): `meta`, or for a move trained without a record its `familiarity` ({step, days, need}: "Tried (1 of 3)",
// "Familiar: 5 of 8 days to learned (2 of 3)"), or for an `untrained` muscle or move "Not trained yet"; under it
// `effect`, a last line in green as the game writes an item's effect (a gear chip's: what the piece would open now).
// Shown beside the
// element the pointer rests on or the focus is on (`showAt(anchor, side, clear)`, kept as `shownAt` while it shows:
// after it, else before it where it does not fit; `side` "left" prefers before it; `clear` a box it never covers, a
// pinned card, over or under the anchor where neither side is off it). WCAG 1.4.13: it takes the pointer, so the
// pointer can cross onto it and rest there; the anchor's leaving and its own (`leave()`) hide it only after a grace in
// which the pointer may come back onto either; it stays until then, until its anchor's focus goes (`hide()`), or until
// Escape through its view's `onKey` (`escape(e)`), the focus kept where it is. One inside a Window (a place's equipment
// chips) carries `popover="manual"`, so it stands in the top layer over the window, never clipped by it. For a screen
// reader the anchor carries the same words.

import {t} from "../lib/copy.js";
import {KitElement, define, h} from "./element.js";

/** How long the pointer may take to cross from the anchor onto the Tooltip, or back, before it goes (WCAG 1.4.13's
 * hoverable content). */
const GRACE_MS = 200;

/** How well the player knows a move (Stats.lua ns.Familiarity: new 0 to learned 3) as the game's tooltip writes it;
 * `days` the game days it was done, `need` those that make it learned. */
export function familiarityWords(step, days, need) {
	const words = ["kit.familiarity.new", "kit.familiarity.tried", "kit.familiarity.familiar", "kit.familiarity.learned"];
	const word = t(words[Math.max(0, Math.min(3, step))]);
	const total = 3;
	return step === 2 ? t("kit.familiarity.days", {word, days, need, step, total})
		: t("kit.familiarity.count", {word, step, total});
}

class QpTooltip extends KitElement {
	build() {
		this.setAttribute("role", "tooltip");
		this.hidden = true;
		this.head = h("p", {class: "qp-type-body-strong qp-tooltip-name"});
		this.word = h("qp-quality-word");
		this.line = h("p", {class: "qp-tooltip-meta"});
		this.after = h("p", {class: "qp-tooltip-meta qp-tooltip-effect"});
		this.append(this.head, this.word, this.line, this.after);
		this.timer = 0;
		this.addEventListener("pointerenter", () => clearTimeout(this.timer));
		this.addEventListener("pointerleave", () => this.leave());
	}

	set familiarity(value) {
		this.known = value ?? null;
		if (this.built) this.update();
	}

	update() {
		this.head.textContent = this.name ?? "";
		this.word.hidden = !this.quality;
		if (this.quality) {
			this.word.quality = this.quality;
			this.word.word = this.qualityWord ?? "";
		}
		const known = this.known;
		const meta = this.meta ?? (this.untrained ? t("character.muscle.untrained")
			: known ? familiarityWords(known.step, known.days, known.need) : "");
		this.line.textContent = meta;
		this.line.hidden = !meta;
		this.after.textContent = this.effect ?? "";
		this.after.hidden = !this.effect;
	}

	/** Shows it beside `anchor`, inside the window: after it (`side` "left": before it), on its other side where it does
	 * not fit. `clear` (a box in the window's pixels: a pinned card) is never covered: where neither side beside the
	 * anchor fits off it, it stands over the anchor or under it, moved aside off that box. */
	showAt(anchor, side = null, clear = null) {
		clearTimeout(this.timer);
		this.shownAt = anchor;
		this.hidden = false;
		if (this.popover && !this.matches(":popover-open")) this.showPopover();
		const at = anchor.getBoundingClientRect();
		const box = this.getBoundingClientRect();
		const gap = 8;
		const clamp = (v, low, high) => Math.min(Math.max(low, v), high);
		const off = (place) => !clear || place.left + box.width <= clear.left || place.left >= clear.right
			|| place.top + box.height <= clear.top || place.top >= clear.bottom;
		// over the anchor, centred on it and aside off `clear`, else under it
		const over = () => {
			const centred = at.left + at.width / 2 - box.width / 2;
			const aside = !clear ? centred : clear.left >= at.right ? Math.min(centred, clear.left - gap - box.width)
				: clear.right <= at.left ? Math.max(centred, clear.right + gap) : centred;
			return {left: clamp(aside, gap, innerWidth - gap - box.width),
				top: at.top - gap - box.height >= gap ? at.top - gap - box.height : at.bottom + gap};
		};
		const after = at.right + gap + box.width <= innerWidth;
		const before = at.left - gap - box.width >= gap;
		const toBefore = side === "left" ? before || !after : !after;
		const top = clamp(at.top + at.height / 2 - box.height / 2, gap, innerHeight - gap - box.height);
		const beside = (onBefore) => ({top,
			left: onBefore ? Math.max(gap, at.left - gap - box.width) : at.right + gap});
		let place = beside(toBefore);
		if (!off(place)) {
			const other = beside(!toBefore);
			place = (toBefore ? after : before) && off(other) ? other : over();
		}
		this.style.setProperty("--qp-tooltip-x", `${place.left}px`);
		this.style.setProperty("--qp-tooltip-y", `${place.top}px`);
	}

	/** The pointer left the anchor or the Tooltip: it goes after the grace, unless the pointer rests on either then. */
	leave() {
		clearTimeout(this.timer);
		if (!this.shownAt) return;
		this.timer = setTimeout(() => {
			if (!this.matches(":hover") && !this.shownAt?.matches(":hover")) this.hide();
		}, GRACE_MS);
	}

	/** Hides it at once (the anchor's focus gone, a click, a scroll, another root). */
	hide() {
		clearTimeout(this.timer);
		this.hidden = true;
		if (this.popover && this.matches(":popover-open")) this.hidePopover();
		this.shownAt = null;
	}

	/** A view's `onKey`: Escape hides it while it shows, the focus kept where it is; true when it took the key. */
	escape(e) {
		if (e.key !== "Escape" || !this.shownAt) return false;
		this.hide();
		return true;
	}
}

define("qp-tooltip", QpTooltip, {name: String, quality: Number, "quality-word": String, meta: String, effect: String,
	untrained: Boolean});
