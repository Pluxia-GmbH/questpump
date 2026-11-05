// CountBar (slice 66, every move's; the hold bar of slice 65; the phones' CountBar, the game's result view): a move's
// count against its step, filled in the step's quality tone. `bar` is {ladder, step, quality, goal, best, top, range,
// seconds}: the label `{ladder} · step {n}` (`step {n}` without a ladder), the range from 0 to the step's top (`range`)
// over 86 % of the track and a spill past it for the count beyond, and posts with their words at the goal, at the
// record before this set (none once it stands at the top or past it) and at the top, each lit once the count passes
// it, in the move's unit (`seconds`: m:ss, else the number of reps); `held` the count in it. `slot(n)` is where the nth
// unit lands (the window's px, for its grain), `flash()` the track's flash with its sparks at the top, and `roll(next)`
// turns it over to the next step's bar after a step-up. Decoration: the pour's words and the rows say it.

import {t} from "../lib/copy.js";
import {clock, LOCALE} from "../lib/format.js";
import {KitElement, define, h, qualityName, styled, token} from "./element.js";

/** The share of the track the range takes; the rest is the spill's (the mockup's `.tr`). */
const SPAN = 0.86;
const NUMBER = new Intl.NumberFormat(LOCALE);
const motion = window.matchMedia("(prefers-reduced-motion: reduce)");

/** A CSS transform to (x, y) px, then `then` (a scale, a rotation: `turned`). */
export function shifted(x, y, then = "") {
	// copy-lint: not shown (a CSS transform)
	return `translate(${x}px, ${y}px) ${then}`.trim();
}

/** A CSS rotation by `deg` degrees. */
export function turned(deg) {
	// copy-lint: not shown (a CSS transform)
	return `rotate(${deg}deg)`;
}

/** A grain's flight (`motion.pourFly`), by which the bar's own moments (its flash, its sparks, its roll) are timed. */
export function flyMs() {
	return token(document.documentElement, "--qp-motion-pour-fly-duration") || 0;
}

/** Sparks out of (x, y) in `layer`'s own px: `n` of them up to `reach` px, all round or (by default) fanned upward, in
 * the colour of `layer`'s `--qp-tone`; none under Reduce Motion. */
export function sparks(layer, x, y, n, reach, all = false) {
	if (motion.matches) return;
	for (let k = 0; k < n; k += 1) {
		const spark = h("i", {class: "qp-spark", "aria-hidden": "true"});
		layer.append(spark);
		const deg = all ? 360 * k / n + Math.random() * 20
			: -170 + 160 * (k / Math.max(1, n - 1)) + (Math.random() - 0.5) * 14;
		const a = deg * Math.PI / 180;
		const d = reach * (0.55 + Math.random() * 0.6);
		spark.animate([{transform: shifted(x, y, "scale(1)"), opacity: 1},
			{transform: shifted(x + Math.cos(a) * d, y + Math.sin(a) * d, "scale(0.4)"), opacity: 0}],
		{duration: flyMs() * (1.55 + Math.random() * 0.6), easing: "cubic-bezier(0.15, 0.7, 0.3, 1)", fill: "forwards"})
			.onfinish = () => spark.remove();
	}
}

class QpCountBar extends KitElement {
	build() {
		this.label = h("span", {class: "qp-count-label qp-type-caps"});
		this.fill = h("i", {class: "qp-count-fill"});
		this.spill = h("i", {class: "qp-count-spill"});
		this.track = h("div", {class: "qp-count-track"}, this.fill, this.spill);
		this.posts = h("div", {class: "qp-count-posts"});
		this.body = h("div", {class: "qp-count-body"}, h("div", {class: "qp-count-head"}, h("span", {class: "qp-gem"}),
			this.label), this.track, this.posts);
		this.setAttribute("aria-hidden", "true");
		this.append(this.body);
		this.heldValue ??= 0;
	}

	set bar(value) {
		this.barData = value;
		this.drawn = null;
		if (this.built) this.update();
	}

	get bar() {
		return this.barData;
	}

	set held(value) {
		this.heldValue = value;
		if (this.built) this.update();
	}

	get held() {
		return this.heldValue;
	}

	/** A share of the range as a share of the track. */
	at(count) {
		return count / Math.max(1, this.barData?.range ?? 1) * SPAN;
	}

	update() {
		const b = this.barData;
		if (!b) return;
		if (this.drawn !== b) {
			this.drawn = b;
			const name = qualityName(b.quality);
			if (name) this.style.setProperty("--qp-tone", `var(--qp-quality-${name})`);
			else this.style.removeProperty("--qp-tone");
			this.label.textContent = b.ladder ? t("kit.reward.step", {ladder: b.ladder, step: b.step ?? 0})
				: t("pump.saved.bar.step", {n: b.step ?? 0});
			this.label.hidden = b.step == null;
			const value = (n) => (b.seconds ? clock(n) : NUMBER.format(n));
			const post = (n, key, side) => n == null ? null : styled(h("i", {class: "qp-count-post", "data-value": n,
				"data-side": side}, h("span", {class: "qp-type-meta", text: t(key, {value: value(n)})})),
			{"--qp-at": this.at(n)});
			this.posts.replaceChildren(...[post(b.goal, "pump.saved.bar.goal", "down"),
				post(b.best, "pump.saved.bar.best", "up"), post(b.top, "pump.saved.bar.top", "down")].filter(Boolean));
		}
		const held = Math.max(0, this.heldValue);
		const range = Math.max(1, b.range);
		styled(this.fill, {"--qp-at": this.at(Math.min(held, range))});
		styled(this.spill, {"--qp-at": Math.min(1 - SPAN, this.at(Math.max(0, held - range)))});
		for (const p of this.posts.children) {
			const n = Number(p.dataset.value);
			p.toggleAttribute("data-hit", p.dataset.side === "up" ? held > n : held >= n);
		}
		this.toggleAttribute("data-full", b.top != null && held >= b.top);
	}

	/** The window's px of the middle of unit `n`'s place on the track (past the range, in the spill). */
	slot(n) {
		const r = this.track.getBoundingClientRect();
		return {x: r.left + Math.min(1, this.at(n - 0.5)) * r.width, y: r.top + r.height / 2};
	}

	/** The track's flash at the top, with its sparks. */
	flash() {
		this.track.animate([{filter: "brightness(1)"}, {filter: "brightness(1.9)", offset: 0.3}, {filter: "brightness(1)"}],
			{duration: flyMs() * 1.55, easing: "ease-out"});
		sparks(this.body, this.at(this.barData?.top ?? 0) * this.body.clientWidth, this.track.offsetTop
			+ this.track.offsetHeight / 2, 14, 46);
	}

	/** Turns over to bar `next` (the next step's, after a step-up): the old one out upward, the new one in from below. */
	roll(next) {
		if (motion.matches) {
			this.held = 0;
			this.bar = next;
			return;
		}
		this.body.animate([{transform: "none", opacity: 1}, {transform: "translateY(-26%)", opacity: 0}],
			{duration: flyMs(), easing: "ease-in", fill: "forwards"}).onfinish = () => {
			this.held = 0;
			this.bar = next;
			this.body.animate([{transform: "translateY(26%)", opacity: 0}, {transform: "none", opacity: 1}],
				{duration: flyMs(), easing: "cubic-bezier(0.2, 0.8, 0.2, 1)", fill: "forwards"});
		};
	}
}

define("qp-count-bar", QpCountBar);
