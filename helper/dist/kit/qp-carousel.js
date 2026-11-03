// Carousel (desktop; Pump home's recent games, 2026-10-28, Rapha: "show thus like in a carousel if i have multiple
// games which i then can click or scroll to the right"): its tiles in one row from the leading edge, never stretched,
// which scrolls sideways once they do not fit, each tile snapping to the row's start. A trackpad, a sideways wheel and
// Shift with the wheel scroll it as the system does; an upright wheel stays the page's. While more lies to one side,
// that edge fades and a small RoundButton with a chevron stands over it, scrolling the row by most of its width (at once
// under Reduce Motion); the buttons are the mouse's way and stay out of the Tab order, named by `previous` and `next`
// for a screen reader. Left and Right move the focus from tile to tile, Home and End to the first and the last, and a
// focused tile scrolls into view. The tiles are the caller's, in `track`, a group named by `label`.

import {KitElement, define, h} from "./element.js";

const still = window.matchMedia("(prefers-reduced-motion: reduce)");

/** The share of the row's width a button scrolls: less than a whole view, so a tile cut at the edge stays in sight. */
const PAGE = 0.8;

/** How far from an end, in px, still counts as at it (a zoomed page's scroll stops on a fraction). */
const NEAR = 1;

class QpCarousel extends KitElement {
	build() {
		this.track = h("div", {class: "qp-carousel-track", role: "group", "on:scroll": () => this.edges(),
			"on:keydown": (e) => this.key(e)});
		this.before = this.arrow("previous", -1);
		this.after = this.arrow("next", 1);
		this.append(this.track, this.before, this.after);
		// the row's ends move with its width and with its tiles (each tile watched as it comes)
		this.resize = new ResizeObserver(() => this.edges());
		this.tiles = new MutationObserver((changes) => {
			for (const change of changes) {
				for (const node of change.addedNodes) if (node instanceof Element) this.resize.observe(node);
			}
			this.edges();
		});
	}

	connectedCallback() {
		super.connectedCallback();
		this.resize.observe(this.track);
		for (const tile of this.track.children) this.resize.observe(tile);
		this.tiles.observe(this.track, {childList: true});
		this.edges();
	}

	disconnectedCallback() {
		this.resize.disconnect();
		this.tiles.disconnect();
	}

	/** A side's button: the mouse's way, out of the Tab order (the keys move along the tiles). */
	arrow(icon, step) {
		const el = h("qp-round-button", {icon, small: true, class: `qp-carousel-${icon}`, hidden: true,
			"on:click": () => this.track.scrollBy({left: step * this.track.clientWidth * PAGE,
				behavior: still.matches ? "auto" : "smooth"})});
		el.ensure();
		el.control.tabIndex = -1;
		return el;
	}

	update() {
		if (this.label) this.track.setAttribute("aria-label", this.label);
		else this.track.removeAttribute("aria-label");
		this.before.spoken = this.previous ?? "";
		this.after.spoken = this.next ?? "";
	}

	/** Fades and shows a side's button while more lies past it; a button pressed to the end hands the focus to the tile
	 * at that end, so it never drops to the page. */
	edges() {
		const left = this.track.scrollLeft;
		const most = this.track.scrollWidth - this.track.clientWidth;
		const sides = [[this.before, left > NEAR, () => this.track.firstElementChild],
			[this.after, left < most - NEAR, () => this.track.lastElementChild]];
		for (const [button, more, end] of sides) {
			if (!more && button.contains(document.activeElement)) end()?.focus({preventScroll: true});
			button.hidden = !more;
		}
		this.toggleAttribute("data-more-start", left > NEAR);
		this.toggleAttribute("data-more-end", left < most - NEAR);
	}

	/** Left and Right, Home and End: the focus to the tile beside, first or last. */
	key(e) {
		if (e.altKey || e.ctrlKey || e.metaKey || e.shiftKey) return;
		const tiles = [...this.track.children].filter((tile) => !tile.hidden);
		const at = tiles.findIndex((tile) => tile.contains(e.target));
		if (at < 0) return;
		const to = {ArrowLeft: at - 1, ArrowRight: at + 1, Home: 0, End: tiles.length - 1}[e.key];
		if (to === undefined) return;
		e.preventDefault();
		tiles[Math.max(0, Math.min(to, tiles.length - 1))].focus();
	}
}

define("qp-carousel", QpCarousel, {label: String, previous: String, next: String});
