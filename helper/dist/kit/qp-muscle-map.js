// MuscleMap: the figure's body and its sixteen muscles in their tones (Hero's map, a move page's Trains card), from
// helper/dist/art/body.js (tools/bodymap.py's): `figure` female or male (the sheet's; male when it names none); `tones`
// {muscle id: quality} lights a trained muscle in its quality's tone at the map's fill over the dark figure, `half`
// {muscle id: quality} a move's secondary muscle at half of it; the `selected` muscle at full tone with the selected
// edge. `mode="hero"`: each named muscle (`names` {id: name}) is a control: hover or focus fires qp-muscle-hover
// ({id, el}) for the Tooltip, a click or Enter fires qp-muscle ({id}). `mode="day"` (the Saga's Muscles): `most` and
// `some` (muscle ids) light the day's muscles, Most in gold and Some in gold at the token's share; `light()` lights them
// one after another once (muscleLight, staggered; at once under Reduce Motion). `fade(ids)` (slice 72, a reward carried
// home) fades Hero's muscles `ids` up from the untrained dark into their tones, once.

import {BODY} from "../art/body.js";
import {KitElement, define, emit, qualityName, token} from "./element.js";

// play.questpump.com asks Trusted Types for every HTML sink (web/worker.js's CSP, docs/plan.md slice 63), and this is
// one of the page's two that write markup: the body map's own, tools/bodymap.py's, which this policy alone passes on.
// A browser or webview without Trusted Types takes the markup as it is.
const POLICY = globalThis.trustedTypes?.createPolicy("qp-muscle-map", {createHTML: (markup) => markup});

class QpMuscleMap extends KitElement {
	build() {
		const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
		svg.setAttribute("viewBox", `0 0 ${BODY.width} ${BODY.height}`);
		svg.innerHTML = POLICY ? POLICY.createHTML(BODY.markup) : BODY.markup;
		this.svg = svg;
		this.append(svg);
		// the listeners on the element, never on the svg: Chromium makes an svg with focus listeners a Tab stop of its
		// own, which the day's map (no control in it) must not be
		this.addEventListener("click", (e) => this.pick(e));
		this.addEventListener("keydown", (e) => {
			if (e.key === "Enter" || e.key === " ") this.pick(e);
		});
		for (const kind of ["pointerover", "focusin"]) {
			this.addEventListener(kind, (e) => {
				const muscle = e.target.closest?.("[data-muscle]");
				if (muscle && this.mode === "hero") emit(this, "qp-muscle-hover", {id: Number(muscle.dataset.muscle), el: muscle});
			});
		}
		this.toneData ??= {};
		this.halfData ??= {};
		this.nameData ??= {};
	}

	pick(e) {
		const muscle = e.target.closest?.("[data-muscle]");
		if (!muscle || this.mode !== "hero") return;
		e.preventDefault();
		emit(this, "qp-muscle", {id: Number(muscle.dataset.muscle)});
	}

	set tones(value) {
		this.toneData = value ?? {};
		if (this.built) this.update();
	}

	set half(value) {
		this.halfData = value ?? {};
		if (this.built) this.update();
	}

	set names(value) {
		this.nameData = value ?? {};
		if (this.built) this.update();
	}

	set most(value) {
		this.mostData = value ?? [];
		if (this.built) this.update();
	}

	set some(value) {
		this.someData = value ?? [];
		if (this.built) this.update();
	}

	/** Lights the day's muscles one after another, from dark (the Saga's first open each game day). */
	light() {
		const lit = [...this.svg.querySelectorAll(".qp-day-most, .qp-day-some")];
		const step = token(this, "--qp-motion-saga-reveal-stagger") || 0;
		for (const path of lit) path.classList.add("qp-day-dark");
		requestAnimationFrame(() => requestAnimationFrame(() => lit.forEach((path, i) => {
			path.style.transitionDelay = `${i * step}ms`;
			path.classList.remove("qp-day-dark");
		})));
	}

	/** Fades the lit muscles `ids` up from the untrained dark into their tones, once (muscleLight: the muscles trained
	 * since Hero last showed; at once under Reduce Motion, whose token gives the fade no time). */
	fade(ids) {
		const lit = [...this.svg.querySelectorAll(".qp-lit")]
			.filter((path) => ids.includes(Number(path.dataset.muscle)));
		for (const path of lit) path.classList.add("qp-dark");
		requestAnimationFrame(() => requestAnimationFrame(() => {
			for (const path of lit) path.classList.remove("qp-dark");
		}));
	}

	update() {
		const figure = this.figure === "female" ? "female" : "male";
		for (const group of this.svg.querySelectorAll("[data-body]")) {
			if (group.dataset.body === figure) group.removeAttribute("display");
			else group.setAttribute("display", "none");
		}
		const hero = this.mode === "hero";
		const day = this.mode === "day";
		this.svg.setAttribute("role", hero ? "group" : "img");
		for (const path of this.svg.querySelectorAll("[data-muscle]")) {
			const id = Number(path.dataset.muscle);
			const full = qualityName(this.toneData[id]);
			const half = qualityName(this.halfData[id]);
			const tone = this.mode === "dark" || day ? null : full ?? half;
			path.classList.toggle("qp-day-most", day && (this.mostData ?? []).includes(id));
			path.classList.toggle("qp-day-some", day && !(this.mostData ?? []).includes(id)
				&& (this.someData ?? []).includes(id));
			if (tone) path.style.setProperty("--qp-muscle", `var(--qp-quality-${tone})`);
			else path.style.removeProperty("--qp-muscle");
			path.classList.toggle("qp-lit", Boolean(tone));
			path.classList.toggle("qp-half", Boolean(tone) && !full);
			path.classList.toggle("qp-selected", id === this.selected);
			const name = this.nameData[id];
			if (hero && name && path.closest("[data-body]").dataset.body === figure) {
				path.setAttribute("tabindex", "0");
				path.setAttribute("role", "button");
				path.setAttribute("aria-label", name);
			} else {
				path.removeAttribute("tabindex");
				path.removeAttribute("role");
				path.removeAttribute("aria-label");
			}
		}
		if (this.label) this.svg.setAttribute("aria-label", this.label);
	}
}

define("qp-muscle-map", QpMuscleMap, {figure: String, mode: String, selected: Number, label: String});
