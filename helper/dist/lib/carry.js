// Numbers that move (contract/design/tokens.json `motion`): a count that rolls from what it showed to what it is now
// and pops at its end (digitPop), for the Saga's reveal, the streak's day on the Result and a reward carried home
// (slice 72, `carry`: the first time Pump home or Hero shows after a pump, each value the pump changed moves once from
// what the surface last showed). Under Reduce Motion every number stands at its end at once, and so does a roll that
// is overtaken (`settle`: the reveal landed by a click, the page left, a pump's cover over it, a new roll).

import {token} from "../kit/element.js";
import {number} from "./saga.js";

/** Reduce Motion, as the system says it now. */
export const still = matchMedia("(prefers-reduced-motion: reduce)");

/** Pops `el`'s digits once (kit.css `.qp-pop`, digitPop: nothing to see under Reduce Motion), again at each call. */
export function pop(el) {
	el.classList.remove("qp-pop");
	// the class taken and given in one task would not start the animation again
	void el.offsetWidth;
	el.classList.add("qp-pop");
}

/** The rolls on their way, by their element: what stands each at its end. */
const rolling = new WeakMap();

/** Stands `el`'s number at its roll's end now, where one is on its way; the roll's `done` does not run, so its caller
 * stands what it would have. */
export function settle(el) {
	rolling.get(el)?.();
}

/** Rolls `el`'s number from `from` to `to` over the motion `name`'s time (tokens.css `--qp-motion-<name>-duration`),
 * easing out, and pops it as it arrives (at once where the token gives no time), or with `landing` over the roll's
 * end, the pop over with the roll (the streak's day, as the phones': digitPop's time is the last half of `carry`);
 * `done` runs once the number stands at `to`. Reduce Motion, and a number that does not change, show `to` still. */
export function roll(el, from, to, name, {landing = false, done = null} = {}) {
	settle(el);
	if (still.matches || from === to) {
		el.textContent = number(to);
		done?.();
		return;
	}
	const ms = token(el, `--qp-motion-${name}-duration`) || 0;
	if (!ms) {
		el.textContent = number(to);
		pop(el);
		done?.();
		return;
	}
	const popAt = landing ? Math.max(0, ms - (token(el, "--qp-motion-digit-pop-duration") || 0)) : ms;
	const began = performance.now();
	let popped = false;
	let live = true;
	rolling.set(el, () => {
		live = false;
		rolling.delete(el);
		el.textContent = number(to);
	});
	const step = (now) => {
		if (!live) return;
		const p = Math.min(1, (now - began) / ms);
		el.textContent = number(Math.round(from + (to - from) * (1 - (1 - p) ** 3)));
		if (!popped && now - began >= popAt) {
			popped = true;
			pop(el);
		}
		if (p < 1) return requestAnimationFrame(step);
		rolling.delete(el);
		done?.();
	};
	el.textContent = number(from);
	requestAnimationFrame(step);
}
