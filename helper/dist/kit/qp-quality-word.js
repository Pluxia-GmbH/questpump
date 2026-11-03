// QualityWord: a quality as its 9-px diamond gem with a soft glow and its word in its tone (Barlow 600 13), never the
// colour alone. `quality` indexes contract/sheet.json's qualities from 1 (Starter) to 7 (Artifact); `word` is its word
// in the player's language, with its `rung` after it where it names one ("Uncommon 5"). Read as the words.

import {t} from "../lib/copy.js";
import {KitElement, define, h, qualityName} from "./element.js";

class QpQualityWord extends KitElement {
	build() {
		this.words = h("span");
		this.append(h("span", {class: "qp-gem", "aria-hidden": "true"}), this.words);
	}

	update() {
		const name = qualityName(this.quality);
		if (name) this.style.setProperty("--qp-tone", `var(--qp-quality-${name})`);
		else this.style.removeProperty("--qp-tone");
		const word = this.word ?? "";
		this.words.textContent = this.rung === null ? word : t("character.quality", {quality: word, number: this.rung});
	}
}

define("qp-quality-word", QpQualityWord, {quality: Number, word: String, rung: Number});
