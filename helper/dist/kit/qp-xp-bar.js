// XPBar: the XP bar, 8 high (`hero`: 6, Hero's): the gold fill to into / need on its track with an inset shadow, the
// rested part past it in gold at 28 %, and its caption under it unless `bare`: "QP 12" (past the level cap the Paragon
// rank, "Paragon 3") and "775 / 1077" in meta. The fill grows over the xpFill motion (a jump under Reduce Motion). Read
// as one: "QP 12, 775 of 1077 XP" (or "Paragon 3, …"), with the rested XP while there is any.
// At a level or a Paragon rank (slice 72, motion.levelRoll) the bar never jumps and never runs backwards: it fills to
// its end (`fillTo`, a leg its caller times), the full bar flashes once (`flash`) and it starts the new level from
// empty (`restart`, which says so: `qp-restart`), each at the Result's instants, or the two by themselves (`turn`);
// `carry` runs the same from the
// point a surface last showed (Hero's bar carried home, motion.carry), and `land` stands it at its end. A point is
// {level, into, need, paragon}. Reduce Motion: the new fill at once.
// With the flash the level rises (`rise`, motion.levelRise, surface.XPBar's rise; Rapha, 2026-11-11: "a nice level up
// with epic arrow animation going up while leveling"): gold chevrons rise from along the bar in waves, the middle
// ones first, and sparks leave its end, while the new level fills under them and its word pops (digitPop); half as
// far on a bar with little room over it (`hero`, and `compact`: the floating Result's). It changes none of the roll's
// instants.

import {pop, still} from "../lib/carry.js";
import {t} from "../lib/copy.js";
import {KitElement, define, h, token} from "./element.js";
import {shifted, sparks, turned} from "./qp-count-bar.js";

/** What a roll leaves on the element while it runs (kit.css): a timed leg, the carry's timing, the flash, no motion. */
const ROLLING = ["data-leg", "data-carry", "data-flash", "data-still"];

function label(level, into, need, rested, paragon) {
	if (paragon > 0) {
		return rested > 0 ? t("kit.xp.label.paragon.rested", {rank: paragon, into, need, rested})
			: t("kit.xp.label.paragon", {rank: paragon, into, need});
	}
	return rested > 0 ? t("kit.xp.label.rested", {level, into, need, rested}) : t("kit.xp.label", {level, into, need});
}

class QpXpBar extends KitElement {
	build() {
		this.rest = h("i", {class: "qp-xp-rested"});
		this.fill = h("i", {class: "qp-xp-fill"});
		this.levelWord = h("span", {class: "qp-type-meta"});
		this.numbers = h("span", {class: "qp-type-meta"});
		this.caption = h("div", {class: "qp-xp-caption", "aria-hidden": "true"}, this.levelWord, this.numbers);
		this.append(h("div", {class: "qp-xp-track", "aria-hidden": "true"}, this.rest, this.fill), this.caption);
		this.setAttribute("role", "img");
	}

	disconnectedCallback() {
		for (const part of this.querySelectorAll(".qp-xp-rise, .qp-spark")) part.remove();
	}

	/** A new roll's number, which an older roll's late callback no longer matches. */
	begin() {
		this.rolls = (this.rolls ?? 0) + 1;
		return this.rolls;
	}

	/** The fill runs to `point` over `ms` (Rust's instants give it: the Result's bar reaching its end as the banner
	 * turns). */
	fillTo(point, ms) {
		// under Reduce Motion the bar waits where it stands for the reveal's landing, which stands it at its end
		if (still.matches) return;
		this.style.setProperty("--qp-xp-leg", `${Math.max(0, ms)}ms`);
		this.toggleAttribute("data-leg", true);
		Object.assign(this, point);
	}

	/** The bar, full, flashes once over levelRoll (a level's or a rank's end). */
	flash() {
		if (still.matches) return;
		for (const mark of ROLLING) this.removeAttribute(mark);
		this.into = this.need ?? 0;
		this.toggleAttribute("data-flash", true);
		this.rise();
	}

	/** The level rises over the full bar: the chevrons of surface.XPBar's rise from along it, each over levelRise, the
	 * middle ones first as a wave spreading to its ends and each wave two staggers behind the one before, so every
	 * column rises as chevrons one over the other; and sparks from the bar's end. */
	rise() {
		const root = document.documentElement;
		const track = this.fill.parentElement;
		const [left, top] = [track.offsetLeft, track.offsetTop + track.offsetHeight / 2];
		const scale = this.compact || this.hero ? token(this, "--qp-surface-xp-bar-rise-compact-fraction") : 1;
		this.style.setProperty("--qp-xp-rise", String(scale));
		const height = token(this, "--qp-surface-xp-bar-rise-height") * scale;
		sparks(this, left + track.offsetWidth, top, token(this, "--qp-surface-xp-bar-rise-spark-count"), height);
		const count = token(this, "--qp-surface-xp-bar-rise-count");
		const waves = token(this, "--qp-surface-xp-bar-rise-wave-count");
		const duration = token(root, "--qp-motion-level-rise-duration") || 0;
		const stagger = token(root, "--qp-motion-level-rise-stagger") || 0;
		const easing = getComputedStyle(root).getPropertyValue("--qp-motion-level-rise-easing").trim() || "ease";
		for (let i = 0; i < count * waves; i += 1) {
			const [column, wave] = [i % count, Math.floor(i / count)];
			const x = left + track.offsetWidth * (column + 0.5) / count;
			const chevron = h("i", {class: "qp-xp-rise", "aria-hidden": "true"});
			this.append(chevron);
			chevron.animate([{transform: shifted(x, top, turned(45)), opacity: 0},
				{opacity: 1, offset: 0.25},
				{transform: shifted(x, top - height, turned(45)), opacity: 0}],
			{duration, delay: (Math.abs(column - (count - 1) / 2) + wave * 2) * stagger, easing, fill: "both"})
				.onfinish = () => chevron.remove();
		}
	}

	/** After its flash the bar stands empty on `point`'s level and fills to its share over xpFill. */
	restart(point) {
		if (still.matches) return this.land(point);
		this.removeAttribute("data-flash");
		this.place({...point, into: 0});
		Object.assign(this, point);
		pop(this.levelWord);
		// the new level began: what names the level beside the bar turns with it (Hero's badge)
		this.dispatchEvent(new CustomEvent("qp-restart"));
	}

	/** A level's or a rank's turn by itself: the flash, then the new level from empty as the flash ends. */
	turn(point) {
		if (still.matches) return this.land(point);
		const roll = this.begin();
		this.flash();
		this.fill.parentElement.addEventListener("animationend", () => {
			if (roll === this.rolls) this.restart(point);
		}, {once: true});
	}

	/** Stands the bar at `point` with no motion, from where its next change moves. */
	place(point) {
		this.toggleAttribute("data-still", true);
		Object.assign(this, point);
		// the fill takes its place before the motion comes back
		void this.fill.offsetWidth;
		this.removeAttribute("data-still");
	}

	/** Moves once from `from`, the point a surface last showed, to `point` over the carry's time, through a level or a
	 * rank by its turn. */
	carry(from, point) {
		if (still.matches) return this.land(point);
		const roll = this.begin();
		const turns = point.level > from.level || point.paragon > from.paragon;
		this.place(from);
		this.toggleAttribute("data-carry", true);
		const end = turns ? {...from, into: from.need} : point;
		if (end.into === from.into && end.need === from.need) return turns ? this.turn(point) : this.land(point);
		this.fill.addEventListener("transitionend", () => {
			if (roll !== this.rolls) return;
			this.removeAttribute("data-carry");
			if (turns) this.turn(point);
		}, {once: true});
		Object.assign(this, end);
	}

	/** Every part of a roll at its end: the bar at `point`. */
	land(point) {
		this.begin();
		for (const mark of ROLLING) this.removeAttribute(mark);
		Object.assign(this, point);
	}

	update() {
		const [level, into, need] = [this.level ?? 1, this.into ?? 0, this.need ?? 0];
		const [rested, paragon] = [this.rested ?? 0, this.paragon ?? 0];
		const share = need > 0 ? Math.min(1, Math.max(0, into / need)) : 1;
		const rest = need > 0 ? Math.min(1 - share, Math.max(0, rested / need)) : 0;
		this.style.setProperty("--qp-xp-share", String(share));
		this.style.setProperty("--qp-xp-rest", String(rest));
		this.levelWord.textContent = paragon > 0 ? t("kit.fanfare.paragon", {rank: paragon}) : t("kit.level", {level});
		this.numbers.textContent = t("kit.xp.progress", {into, need});
		this.caption.hidden = this.bare;
		this.setAttribute("aria-label", label(level, into, need, rested, paragon));
	}
}

define("qp-xp-bar", QpXpBar, {
	level: Number, into: Number, need: Number, rested: Number, paragon: Number, bare: Boolean, hero: Boolean,
	compact: Boolean,
});
