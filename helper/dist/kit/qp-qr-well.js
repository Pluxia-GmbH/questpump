// QrWell: the pairing QR code on a light well in `text` (#F2EBDC) with its quiet zone of four modules and `ink`
// modules (docs/ui.md, "The Menu on the computer": a camera reads dark modules on light), named for VoiceOver and NVDA
// (`label`). `code` is {width, modules}: its modules row by row, 1 dark (helper/src-tauri/src/pairing.rs); none draws
// no code.

import {KitElement, define} from "./element.js";

const SVG = "http://www.w3.org/2000/svg";
/** The quiet zone, in modules. */
const QUIET = 4;

class QpQrWell extends KitElement {
	set code(value) {
		this.modules = value ?? null;
		if (this.built) this.update();
	}

	get code() {
		return this.modules ?? null;
	}

	update() {
		this.setAttribute("role", "img");
		this.setAttribute("aria-label", this.label ?? "");
		const code = this.code;
		this.hidden = !code;
		if (!code || code === this.drawn) return;
		this.drawn = code;
		const side = code.width + QUIET * 2;
		const svg = document.createElementNS(SVG, "svg");
		svg.setAttribute("viewBox", `0 0 ${side} ${side}`);
		svg.setAttribute("shape-rendering", "crispEdges");
		svg.setAttribute("aria-hidden", "true");
		const ground = document.createElementNS(SVG, "rect");
		ground.setAttribute("width", String(side));
		ground.setAttribute("height", String(side));
		ground.setAttribute("class", "qp-qr-ground");
		let d = "";
		for (let i = 0; i < code.modules.length; i += 1) {
			if (code.modules[i] !== "1") continue;
			d += `M${(i % code.width) + QUIET} ${Math.floor(i / code.width) + QUIET}h1v1h-1z`;
		}
		const ink = document.createElementNS(SVG, "path");
		ink.setAttribute("d", d);
		ink.setAttribute("class", "qp-qr-ink");
		svg.append(ground, ink);
		this.replaceChildren(svg);
	}
}

define("qp-qr-well", QpQrWell, {label: String});
