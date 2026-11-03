// RungStars: a rung's stars (a ladder's page's now step): `total` of them, the first `stars` earned in gold, the rest
// text3 (3:1 on the card, so every star shows); read as "2 of 3 stars".

import {t} from "../lib/copy.js";
import {KitElement, define, h} from "./element.js";

class QpRungStars extends KitElement {
	update() {
		const total = Math.max(0, this.total ?? 3);
		const stars = Math.max(0, Math.min(total, this.stars ?? 0));
		this.setAttribute("role", "img");
		this.setAttribute("aria-label", t("kit.stars.label", {stars, total}));
		this.replaceChildren(...Array.from({length: total}, (_, i) =>
			h("qp-art", {name: "star", class: i < stars ? "qp-earned" : null})));
	}
}

define("qp-rung-stars", QpRungStars, {stars: Number, total: Number});
