// KindBar (the Saga, 2026-10-15; Tokens.Surface.Kinds; contract/fixtures/history.json `saga` rule 10): the day's moves
// by kind as one 100 % bar (strength, cardio, mobility, yoga in their tones, a kind with none left out), white ticks at
// the plan's shares (`shares`, percent, the focus's), and each kind's word with its count in one row under the bar,
// each at its part's start where the row holds them, moved apart so none touches (lib/saga.js `spread`), a whole word
// on a second row only where the row cannot hold them all (as the phones lay their labels). `kinds` are the counts; it
// reads as its `label`.

import {KIND_WORDS, KINDS, number, spread} from "../lib/saga.js";
import {KitElement, define, h, styled, token} from "./element.js";

class QpKindBar extends KitElement {
	build() {
		this.kindData ??= [0, 0, 0, 0];
		this.shareData ??= [0, 0, 0, 0];
		this.setAttribute("role", "img");
		// a new width lays the words out again
		new ResizeObserver(() => {
			if (Math.round(this.clientWidth) !== this.laidWidth) this.place();
		}).observe(this);
	}

	set kinds(value) {
		this.kindData = value ?? [0, 0, 0, 0];
		if (this.built) this.update();
	}

	set shares(value) {
		this.shareData = value ?? [0, 0, 0, 0];
		if (this.built) this.update();
	}

	update() {
		if (this.label) this.setAttribute("aria-label", this.label);
		const parts = KINDS.map((kind, k) => ({kind, count: this.kindData[k], word: KIND_WORDS[k]}))
			.filter((p) => p.count > 0);
		const track = h("div", {class: "qp-kind-track", "aria-hidden": "true"},
			...parts.map((p) => styled(h("i", {class: `qp-kind-part qp-kind-${p.kind}`}), {"flex-grow": p.count})));
		// the plan's ticks at the shares' running sums, the last (100 %) left out
		let sum = 0;
		const ticks = this.shareData.slice(0, -1).map((s) => {
			sum += s;
			return styled(h("i", {class: "qp-kind-tick"}), {left: `${sum}%`});
		});
		const bar = h("div", {class: "qp-kind-bar", "aria-hidden": "true"}, track, ...ticks);
		this.words = h("div", {class: "qp-kind-words", "aria-hidden": "true"}, ...parts.map((p) => h("span",
			{class: "qp-kind-word"}, p.word, " ", h("b", {class: "qp-kind-count", text: number(p.count)}))));
		// each word's part starts at the share of the moves before it
		const total = parts.reduce((a, p) => a + p.count, 0) || 1;
		let before = 0;
		this.starts = parts.map((p) => {
			const at = before / total;
			before += p.count;
			return at;
		});
		this.replaceChildren(bar, this.words);
		this.place();
	}

	/** Lays the words out on the row, measured as drawn: each at its part's start, moved apart; wrapped where the row
	 * cannot hold them all. */
	place() {
		const width = Math.round(this.clientWidth);
		if (!width || !this.words) return;
		this.laidWidth = width;
		const words = [...this.words.children];
		this.words.classList.remove("qp-kind-wrap");
		const gap = token(this, "--qp-layout-space12");
		const widths = words.map((w) => w.getBoundingClientRect().width);
		if (widths.reduce((a, w) => a + w, 0) + gap * (words.length - 1) > width) {
			this.words.classList.add("qp-kind-wrap");
			for (const w of words) w.style.removeProperty("left");
			return;
		}
		const middles = spread(words.map((w, i) => ({x: this.starts[i] * width + widths[i] / 2, width: widths[i]})), 0,
			width, gap);
		words.forEach((w, i) => {
			w.style.left = `${middles[i] - widths[i] / 2}px`;
		});
	}
}

define("qp-kind-bar", QpKindBar, {label: String});
