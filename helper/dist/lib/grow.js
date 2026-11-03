// The tapped tile growing into what it opens (docs/plan.md slice 46; docs/ui.md, "Tablets and wide windows"): a tree's
// node into its pinned card or its card's Window, a ladder page's step tile into the card's Window, a Moves tile into
// its move's window. Material's container transform, its content fading in and fit to the width: one Web Animation
// (Element.animate) over the element at its own box, a box going from the tile's to the element's, the element scaled
// to the box's width from its top-left corner and clipped to the box's height (clip-path inset), its corners going
// from the tile's radius to its own and its outer shadow coming out over the last step, faded in over the first quarter
// of the way (MaterialContainerTransform's DEFAULT_ENTER_THRESHOLDS), in `motion.grow` (300 ms, standard). As the
// player closes it, it shrinks back into the tile in `motion.growBack` (250 ms) while that tile is at least half in
// view (the box reaching the tile at 0.9 of the way, the corners from 0.3 to 0.9 and the fade from 0.6 to 0.9: its
// DEFAULT_RETURN_THRESHOLDS); else its owner's own leave runs. The tile's box is its well's (the rounded square a
// MoveTile draws; a waiting ladder's column its own) read at the tap from the grid's cell where it stands now, the
// page's scroll included, and again at the close, never the element's layout read back; the element's own box is its
// layout box with its CSS entrance held off (`GROWN`: the Window's qp-window-in, kit.css, and the pinned card's
// @starting-style, book.css). Neither under Reduce Motion (prefers-reduced-motion: macOS's Reduce motion, Windows'
// Animation effects), which keeps the element's own fade.

import {token} from "../kit/element.js";

/** The class a grown element keeps while it stands, added before it shows so its own CSS entrance never starts (a
 * Window's dim fading in with the grow instead); its owner takes it off as it hides the element. */
export const GROWN = "qp-grown";
/** The class an element shrinking back into its tile holds: its own CSS leave held off, a Window's dim fading out with
 * it. */
const SHRINKING = "qp-shrinking";
/** The keyframes a morph lays its box on: a clip's height is the box's over the scale, which no interpolation between
 * two keyframes keeps, so each keyframe holds the exact box at its share of the way (the effect's easing picks the
 * share); 20 puts every threshold below on a keyframe. */
const STEPS = 20;
/** MaterialContainerTransform's DEFAULT_ENTER_THRESHOLDS and DEFAULT_RETURN_THRESHOLDS (material-components-android,
 * read 2026-10-18): the share of the eased way the box (`scaleMask`), the corners (`shapeMask`) and the fade take. */
const ENTER = {box: [0, 1], shape: [0, 0.75], fade: [0, 0.25]};
const RETURN = {box: [0, 0.9], shape: [0.3, 0.9], fade: [0.6, 0.9]};

const still = window.matchMedia("(prefers-reduced-motion: reduce)");
/** The morph under way on an element: its animation and what its end does (a shrink's owner hides the element). */
const running = new WeakMap();

/** Where `p` stands in `[from, to]`, from 0 to 1. */
function share(p, [from, to]) {
	return Math.min(1, Math.max(0, (p - from) / (to - from)));
}

function mix(a, b, p) {
	return a + (b - a) * p;
}

/** A tile's visible box: the MoveTile's well in it (a tree's node, a ladder page's row and the Moves grid hand theirs),
 * else the element itself (a waiting ladder's column). */
function wellOf(tile) {
	return tile.querySelector(".qp-tile-well") ?? tile;
}

/** The share of `el`'s box the window shows: inside the viewport and every box that clips it (its page's scroll); 0
 * for one out of the page or drawn nowhere (a hidden side's, a redrawn grid's old tile). */
function inView(el) {
	if (!el.isConnected) return 0;
	const box = el.getBoundingClientRect();
	if (!box.width || !box.height) return 0;
	let [left, top, right, bottom] = [Math.max(box.left, 0), Math.max(box.top, 0), Math.min(box.right, innerWidth),
		Math.min(box.bottom, innerHeight)];
	for (let at = el.parentElement; at; at = at.parentElement) {
		const style = getComputedStyle(at);
		if (style.overflowX === "visible" && style.overflowY === "visible") continue;
		const clip = at.getBoundingClientRect();
		[left, top, right, bottom] = [Math.max(left, clip.left), Math.max(top, clip.top), Math.min(right, clip.right),
			Math.min(bottom, clip.bottom)];
	}
	return (Math.max(0, right - left) * Math.max(0, bottom - top)) / (box.width * box.height);
}

/** How far an element's outer shadows reach past its edge (each one's offset, blur and spread), in px. */
function shadowReach(el) {
	let reach = 0;
	// the computed list: each shadow's colour, then its lengths, `inset` last; no comma inside a colour's parentheses
	for (const shadow of getComputedStyle(el).boxShadow.split(/,(?![^(]*\))/)) {
		if (shadow.includes("inset")) continue;
		const [x = 0, y = 0, blur = 0, spread = 0] = (shadow.match(/-?[\d.]+px/g) ?? []).map(parseFloat);
		reach = Math.max(reach, Math.max(Math.abs(x), Math.abs(y)) + blur + Math.max(0, spread));
	}
	return reach;
}

/** An element's four corners' radii (top-left, top-right, bottom-right, bottom-left), in px. */
function radii(el) {
	const style = getComputedStyle(el);
	return ["borderTopLeftRadius", "borderTopRightRadius", "borderBottomRightRadius", "borderBottomLeftRadius"]
		.map((corner) => parseFloat(style[corner]) || 0);
}

/** The keyframes of a morph between the tile's box `tile` (its corners' radius `round`) and the element's own `own`
 * (its corners `corners`, its outer shadow reaching `reach`): out of the tile, or `back` into it. */
function frames(own, corners, reach, tile, round, back) {
	const way = back ? RETURN : ENTER;
	const out = [];
	for (let i = 0; i <= STEPS; i++) {
		const k = i / STEPS;
		// how far the box, the corners and the opacity stand from the tile toward the element's own
		const toward = (span) => (back ? 1 - share(k, span) : share(k, span));
		const p = toward(way.box);
		const box = {left: mix(tile.left, own.left, p), top: mix(tile.top, own.top, p), width: mix(tile.width, own.width, p),
			height: mix(tile.height, own.height, p)};
		const scale = box.width / own.width;
		// the foot the box's height leaves out, in the element's own pixels; its shadow only at its own box
		const cut = Math.max(0, own.height - box.height / scale);
		const shade = p === 1 ? reach : 0;
		const radius = corners.map((c) => `${(mix(round, c, toward(way.shape)) + shade) / scale}px`).join(" ");
		// copy-lint: not shown (a CSS transform)
		const transform = `translate(${box.left - own.left}px, ${box.top - own.top}px) scale(${scale})`;
		// copy-lint: not shown (a CSS clip-path)
		const clipPath = `inset(${-shade}px ${-shade}px ${cut - shade}px ${-shade}px round ${radius})`;
		out.push({offset: k, opacity: toward(way.fade), transformOrigin: "0 0", transform, clipPath});
	}
	return out;
}

/** Ends the morph under way on `el` at once (a shrink's end runs: its owner hides the element), if one is. */
export function settle(el) {
	const morph = running.get(el);
	if (!morph) return;
	running.delete(el);
	morph.done?.();
	morph.anim.cancel();
	el.classList.remove(SHRINKING);
}

function run(el, keyframes, motion, done) {
	const anim = el.animate(keyframes, {duration: token(el, `--qp-motion-${motion}-duration`),
		easing: getComputedStyle(el).getPropertyValue(`--qp-motion-${motion}-easing`).trim() || "ease",
		// a shrink holds the tile's box until its owner has hidden the element
		fill: done ? "forwards" : "none"});
	const morph = {anim, done};
	running.set(el, morph);
	// a cancel (`settle`) rejects `finished`: nothing more to do then
	anim.finished.then(() => {
		if (running.get(el) === morph) settle(el);
	}, () => {});
	return anim;
}

/** Shows `el` through `show` (a step's card unhidden and placed, a Window's dialog opened) and grows it out of `tile`
 * (a tree's node, a step's row, a Moves tile; none: it shows with its own entrance) where the tile is in view and
 * Reduce Motion is off. Returns the animation, or none where `el` showed with its own entrance. */
export function grow(el, tile, show) {
	settle(el);
	const well = tile && !still.matches ? wellOf(tile) : null;
	if (!well || !inView(well)) {
		show();
		return null;
	}
	el.classList.add(GROWN);
	show();
	// both boxes as they stand once it shows (its showing may scroll the page: its Start into view)
	const from = well.getBoundingClientRect();
	const own = el.getBoundingClientRect();
	if (!own.width || !own.height) {
		// drawn nowhere after all: its own entrance, as it shows
		el.classList.remove(GROWN);
		return null;
	}
	return run(el, frames(own, radii(el), shadowReach(el), from, radii(well)[0], false), "grow", null);
}

/** Shrinks `el` back into `tile` as the player closes it, where that tile is at least half in view and Reduce Motion is
 * off, its own CSS leave held off; `done` hides it at the end (or as `settle` lands it early). Returns the animation,
 * or none where its owner's own leave runs. */
export function shrink(el, tile, done) {
	settle(el);
	const well = tile && !still.matches ? wellOf(tile) : null;
	if (!well || inView(well) < 0.5) return null;
	const own = el.getBoundingClientRect();
	if (!own.width || !own.height) return null;
	const to = well.getBoundingClientRect();
	el.classList.add(SHRINKING);
	return run(el, frames(own, radii(el), shadowReach(el), to, radii(well)[0], true), "grow-back", done);
}
