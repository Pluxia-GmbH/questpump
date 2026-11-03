// StepReveal (slice 65; the phones' StepReveal, the game's result view): the step a hold's save opened, played over the
// figure's room once the pour lands. `reveal` is Rust's {at, opens, folds, tiles, from, to, step, quality, word, hero,
// colour, landed}: the ladder's tiles come in, the held ones muted and the ones ahead dark behind padlocks, the
// current one (`from`) never highlighted; a light runs along the rail from its edge to the next tile (`to`) and fills
// it gold; that tile's lock shakes and flies off in the tone and the tile bursts into its colour at `opens` (a flash, a
// pop that settles, a shockwave, sparks); then the new move's figure (`hero`) rises over the ladder with "Step {n}
// unlocked" in gold caps, its name in Cinzel and its QualityWord; a new colour (`colour`) adds turning rays and a
// second burst at the gem. `play()` runs it from Rust's instants (its parts timed as shares of `motion.stepReveal`'s
// revealFromMs, the moment the next step opens, as the mockup timed them); `fold(rect, then)` flies the hero into the
// unlock row's tile (`rect`, the window's px) and fades the rest. Said once as a status: its caption, name and quality.

import {t} from "../lib/copy.js";
import {KitElement, define, h, qualityName, styled} from "./element.js";
import {flyMs, shifted, sparks} from "./qp-hold-bar.js";

/** Each part's start and length as shares of the opening (`revealFromMs`; the mockup's 1240 ms). */
const AT = {
	ladder: [0, 0.27],
	rail: [0.45, 0.355],
	shake: [0.74, 0.24],
	fly: [1.05, 0.37],
	open: [1, 0.56],
	glow: [1.06, 0.56],
	rays: [1.11, 0.73],
	hero: [1.13, 0.45],
	cap: [1.24, 0.24],
	name: [1.31, 0.37],
	word: [1.42, 0.34],
	gem: [1.45, 0],
};
const EASE = "cubic-bezier(0.2, 0.8, 0.2, 1)";
const POP = "cubic-bezier(0.2, 1.35, 0.4, 1)";

class QpStepReveal extends KitElement {
	build() {
		this.setAttribute("role", "status");
		this.timers = [];
	}

	disconnectedCallback() {
		for (const timer of this.timers) clearTimeout(timer);
		this.timers = [];
	}

	set reveal(value) {
		this.data = value;
		this.ensure();
		this.draw();
	}

	get reveal() {
		return this.data;
	}

	draw() {
		const r = this.data;
		const name = qualityName(r.quality);
		if (name) this.style.setProperty("--qp-tone", `var(--qp-quality-${name})`);
		this.toggleAttribute("data-colour", r.colour);
		this.tiles = r.tiles.map((tile, i) => {
			const ahead = i > r.from;
			const tone = qualityName(tile.quality);
			const el = h("span", {class: "qp-reveal-tile", "data-state": ahead ? "ahead" : "held"},
				h("qp-move-tile", {move: tile.key, quality: tile.quality, size: "60"}),
				h("i", {class: "qp-reveal-flash"}), h("i", {class: "qp-reveal-shock"}),
				ahead ? h("qp-art", {name: "mark_lock", class: "qp-reveal-lock"}) : null);
			if (tone) el.style.setProperty("--qp-tone", `var(--qp-quality-${tone})`);
			return el;
		});
		this.rail = h("i", {class: "qp-reveal-rail"});
		this.fill = h("i", {class: "qp-reveal-fill"});
		this.run = h("i", {class: "qp-reveal-run"});
		this.ladder = h("div", {class: "qp-reveal-ladder"}, this.rail, this.fill, this.run, ...this.tiles);
		this.glow = h("div", {class: "qp-reveal-glow"});
		this.rays = r.colour ? h("div", {class: "qp-reveal-rays"}) : null;
		this.hero = h("div", {class: "qp-reveal-hero"}, h("qp-figure-stage", {move: r.hero.key, still: true}));
		this.cap = h("p", {class: "qp-reveal-cap qp-type-caps", text: t("pump.saved.unlocked", {n: r.step})});
		this.nameEl = h("h2", {class: "qp-reveal-name qp-type-title", text: r.hero.name});
		this.word = h("qp-quality-word", {class: "qp-reveal-word", quality: r.quality, word: r.word});
		this.words = h("div", {class: "qp-reveal-words"}, this.cap, this.nameEl, this.word);
		this.replaceChildren(...[this.glow, this.rays, this.hero, this.words, this.ladder].filter(Boolean));
		this.setAttribute("aria-label", [this.cap.textContent, r.hero.name, r.word].join(", "));
	}

	/** The tile the save opened in its colour. */
	open() {
		const tile = this.tiles[this.data.to];
		if (!tile) return;
		tile.dataset.state = "open";
		tile.querySelector(".qp-reveal-lock")?.remove();
	}

	/** The reveal from Rust's instants, or at its end where it landed. */
	play() {
		const r = this.data;
		if (r.landed) {
			this.open();
			this.dataset.landed = "";
			return;
		}
		const unit = r.opens - r.at;
		const base = r.at - Date.now();
		const run = (el, frames, [start, length], easing = EASE, fill = "both") => el?.animate(frames,
			{duration: Math.max(1, unit * length), delay: base + unit * start, easing, fill});
		const from = this.tiles[r.from];
		const to = this.tiles[r.to];
		run(this, [{opacity: 0}, {opacity: 1}], [0, 0.18]);
		run(this.ladder, [{opacity: 0, transform: "translateY(30px)"}, {opacity: 1, transform: "none"}], AT.ladder);
		if (from && to) {
			// the light leaves the current step's edge and fills the rail up to the next one; the current step stays quiet
			const x0 = from.offsetLeft + from.offsetWidth;
			const x1 = to.offsetLeft;
			styled(this.fill, {left: `${x0}px`, width: `${Math.max(0, x1 - x0)}px`});
			run(this.fill, [{transform: "scaleX(0)", opacity: 1}, {transform: "scaleX(1)", opacity: 1}], AT.rail,
				"cubic-bezier(0.5, 0, 0.3, 1)");
			run(this.run, [{opacity: 0, transform: shifted(x0, 0, "scale(0.6)")},
				{opacity: 1, transform: shifted(x0, 0, "scale(1)"), offset: 0.15},
				{opacity: 1, transform: shifted(x1, 0, "scale(1.2)"), offset: 0.9},
				{opacity: 0, transform: shifted(x1, 0, "scale(1.8)")}], AT.rail, "cubic-bezier(0.5, 0, 0.3, 1)");
			const lock = to.querySelector(".qp-reveal-lock");
			run(lock, [{transform: "rotate(0)"}, {transform: "rotate(-14deg)", offset: 0.2},
				{transform: "rotate(12deg)", offset: 0.45}, {transform: "rotate(-8deg)", offset: 0.7},
				{transform: "rotate(0)"}], AT.shake, "ease-in-out", "none");
			run(lock, [{opacity: 1, transform: "none", color: "var(--qp-color-text3)"},
				{opacity: 1, transform: "translateY(-8px) scale(1.25)", color: "var(--qp-tone)", offset: 0.35},
				{opacity: 0, transform: "translateY(-34px) scale(1.5) rotate(18deg)", color: "var(--qp-tone)"}], AT.fly);
		}
		this.later(r.opens, () => {
			this.open();
			if (!to) return;
			run(to.querySelector("img"), [{filter: "grayscale(1) brightness(0.35)", opacity: 0.4},
				{filter: "grayscale(0) brightness(1.9)", opacity: 1, offset: 0.35},
				{filter: "grayscale(0) brightness(1)", opacity: 1}], [0, AT.open[1]], "ease-out", "none");
			run(to.querySelector(".qp-reveal-flash"), [{opacity: 0}, {opacity: 0.65, offset: 0.3}, {opacity: 0}],
				[0, AT.open[1]], "linear");
			run(to, [{transform: "scale(1)"}, {transform: "scale(1.28)", offset: 0.35}, {transform: "scale(1.1)"}],
				[0, AT.open[1]], POP);
			run(to.querySelector(".qp-reveal-shock"), [{opacity: 0.95, transform: "scale(1)"},
				{opacity: 0, transform: "scale(2.1)"}], [0.03, AT.open[1]], "cubic-bezier(0.1, 0.6, 0.3, 1)");
			sparks(this.ladder, to.offsetLeft + to.offsetWidth / 2, to.offsetTop + to.offsetHeight / 2,
				r.colour ? 24 : 14, r.colour ? 95 : 65, true);
		});
		run(this.glow, [{opacity: 0, transform: "translate(-50%, -50%) scale(0.6)"},
			{opacity: 1, transform: "translate(-50%, -50%) scale(1)"}], AT.glow);
		run(this.rays, [{opacity: 0}, {opacity: 1}], AT.rays);
		run(this.hero, [{opacity: 0, transform: "translateY(26px) scale(0.84)"}, {opacity: 1, transform: "none"}],
			AT.hero, POP);
		run(this.cap, [{opacity: 0, transform: "translateY(8px)"}, {opacity: 1, transform: "none"}], AT.cap);
		run(this.nameEl, [{opacity: 0, transform: "translateY(10px) scale(1.04)"}, {opacity: 1, transform: "none"}],
			AT.name);
		run(this.word, [{opacity: 0, transform: "scale(0.6)"}, {opacity: 1, transform: "scale(1.15)", offset: 0.6},
			{opacity: 1, transform: "none"}], AT.word, POP);
		if (r.colour) {
			this.later(r.at + unit * AT.gem[0], () => {
				const gem = this.word.querySelector(".qp-gem");
				if (!gem) return;
				const g = gem.getBoundingClientRect();
				const box = this.getBoundingClientRect();
				sparks(this, g.left - box.left + g.width / 2, g.top - box.top + g.height / 2, 10, 40, true);
			});
		}
	}

	/** `act` at unix ms `at` (at once where that passed), while the reveal is on the page. */
	later(at, act) {
		this.timers.push(setTimeout(() => {
			if (this.isConnected) act();
		}, Math.max(0, at - Date.now())));
	}

	/** The fold: the hero flies into `rect` (the unlock row's tile, the window's px) while the rest fades; `then` once
	 * it is over. */
	fold(rect, then) {
		const fly = flyMs();
		const from = this.hero.getBoundingClientRect();
		const done = () => {
			this.remove();
			then?.();
		};
		for (const el of [this.ladder, this.words, this.glow, this.rays]) {
			el?.animate([{opacity: 1}, {opacity: 0}], {duration: fly, fill: "forwards"});
		}
		if (!rect || !fly || from.width === 0) {
			setTimeout(done, fly);
			return;
		}
		const k = Math.min(rect.width / from.width, rect.height / from.height);
		const dx = rect.left + rect.width / 2 - (from.left + from.width / 2);
		const dy = rect.top + rect.height / 2 - (from.top + from.height / 2);
		this.hero.animate([{transform: "none"}, {transform: shifted(dx, dy, `scale(${k})`)}],
			{duration: fly, easing: "cubic-bezier(0.5, 0, 0.2, 1)", fill: "forwards"}).onfinish = done;
	}
}

define("qp-step-reveal", QpStepReveal);
