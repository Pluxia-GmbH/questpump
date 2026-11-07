// The own art inline (docs/ui.md, "The art"): a piece of helper/dist/art/art.js (tools/art.py's, from
// contract/design/art) as an <svg> in the page, its paints the tokens' classes (tokens.css) and its tint layers the
// element's colour (currentColor), so a glyph takes the colour of the words beside it. Inline, since the page loads no
// file by fetch (its CSP), and an <img> would draw the tint black. A piece shown twice gets its own gradient ids.
// Decoration: hidden from screen readers; what it means is said by the control or the words around it.

import {ART} from "../art/art.js";
import {KitElement, define} from "./element.js";

let copies = 0;

// play.questpump.com asks Trusted Types for every HTML sink (web/worker.js's CSP, docs/plan.md slice 63), and this is
// one of the page's two that write markup: the art's own, tools/art.py's, which this policy alone passes on. A browser
// or webview without Trusted Types takes the markup as it is.
const POLICY = globalThis.trustedTypes?.createPolicy("qp-art", {createHTML: (markup) => markup});

/** The piece's SVG element; its gradients' ids made this copy's own. */
export function artSvg(name) {
	const piece = ART[name];
	if (!piece) throw new Error(`art: no piece ${name}`);
	copies += 1;
	const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
	svg.setAttribute("viewBox", `0 0 ${piece.width} ${piece.height}`);
	svg.setAttribute("aria-hidden", "true");
	svg.setAttribute("focusable", "false");
	const markup = piece.markup.replaceAll(`qp-${name}-`, `qp-${name}-${copies}-`);
	svg.innerHTML = POLICY ? POLICY.createHTML(markup) : markup;
	return svg;
}

/** `<qp-art name="action_pump">`: one piece, sized by its CSS (1em by default). */
class QpArt extends KitElement {
	update() {
		if (this.shown === this.name) return;
		this.shown = this.name;
		this.replaceChildren(this.name ? artSvg(this.name) : "");
	}
}

define("qp-art", QpArt, {name: String});
