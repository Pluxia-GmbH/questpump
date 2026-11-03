// HeroColumn: a page's column whose one hero (the child marked data-hero: a pump's TimerRing, Hero's body map) gives up
// height before a block under it or the column's foot leaves the first view: the hero takes the room the other blocks
// leave, between `least` and `most` (px), and past its least the column scrolls. kit.css lays it out.

import {KitElement, define} from "./element.js";

class QpHeroColumn extends KitElement {
	update() {
		for (const [attr, prop] of [["least", "--qp-hero-least"], ["most", "--qp-hero-most"]]) {
			const value = this.getAttribute(attr);
			if (value === null) this.style.removeProperty(prop);
			else this.style.setProperty(prop, `${Number(value)}px`);
		}
	}
}

define("qp-hero-column", QpHeroColumn, {least: Number, most: Number});
