// SlantTag (the Saga, 2026-10-15; Tokens.Surface.SlantTag): a slanted tag of caps words, its slant the token's share of
// its height. `tone`: "gold" (today's chapter), "dim" (an earlier chapter's, in text3) or a quality's name (a day's
// word, outlined in its tone); `filled` fills it (Best day ever, gold with goldInk words) and `star` sets the star
// before its words. Its words are its `label`; the words around it say it to a screen reader where the tag is part of
// a larger reading (the hero's), else it reads as its words.

import {KitElement, define, h} from "./element.js";

class QpSlantTag extends KitElement {
	build() {
		this.words = h("span", {class: "qp-slant-words"});
		this.append(this.words);
	}

	update() {
		const star = this.star ? h("qp-art", {name: "star"}) : "";
		this.words.replaceChildren(star, this.label ?? "");
		const tone = this.tone ?? "gold";
		this.dataset.tone = ["gold", "dim"].includes(tone) ? tone : "quality";
		if (this.dataset.tone === "quality") this.style.setProperty("--qp-tag", `var(--qp-quality-${tone})`);
		else this.style.removeProperty("--qp-tag");
	}
}

define("qp-slant-tag", QpSlantTag, {label: String, tone: String, filled: Boolean, star: Boolean});
