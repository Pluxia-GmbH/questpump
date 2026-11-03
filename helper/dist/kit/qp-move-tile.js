// MoveTile: a move's key pose (`move`: a library move's key, or a pump move's `figureName`; its square of the move's
// bundled still, lib/figure.js `tileStill`, as the phones' MoveTile(move:) finds it; else the move glyph, the pump's,
// as theirs and the figure show for a move the build has no figure of) on the dark well in a 1.5-px edge of its
// quality's tone (`quality`) with a faint glow; `untrained` (no quality) the lineStrong edge and the pose at 75 %;
// `off` (switched off for pumps) the pose at 40 % with a small Off mark on its top corner; `favourite` a star on its
// other top corner; `mastered` (slice 42, a step mastered) a small gold seal with a check inside the tile at its
// bottom-trailing corner, at every size; `topped` (slice 42, a step card's group tile at its top once) a small check on
// a raised disc centred on its bottom-trailing corner, the favourite mark's twin; `face="ahead"` (slice 36, a step ahead or a move above the player's reach, in place of the locked
// tile: David, 2026-10-14, "it should not be locked") the pose greyed with no colour, its edge in text3 at 3:1 against
// the page, since it still opens a card or a window; `face="empty"` the well alone. `size` 44, 56 (the default) or
// 60. Named for a screen reader when it has a `name` ("Knee push-up, Uncommon 5",
// with its quality's `word` and its `step`), decoration otherwise; a button with `action`.

import {subscribe} from "../lib/bridge.js";
import {t} from "../lib/copy.js";
import {tileStill} from "../lib/figure.js";
import {KitElement, define, h, qualityName} from "./element.js";

class QpMoveTile extends KitElement {
	build() {
		this.redraw = () => this.update();
	}

	/** Drawn again as the figures come: a window opened before Rust published them (the tray's Open while the start
	 * still loads the engine, before `figures_start`) drew no still (lib/figure.js, imported first, takes the topic
	 * before this hears it). */
	connectedCallback() {
		super.connectedCallback();
		this.stopTable = subscribe("figures", this.redraw);
	}

	disconnectedCallback() {
		this.stopTable?.();
	}

	label() {
		if (!this.name) return null;
		let words = this.quality && this.word ? t("kit.quality.label", {name: this.name, quality: this.word,
			rung: this.step ?? 0}) : t("kit.quality.untrained", {name: this.name});
		if (this.favourite) words = t("kit.tile.favourite.label", {label: words});
		if (this.mastered) words = `${words}, ${t("ladders.tree.mastered")}`;
		if (this.topped) words = `${words}, ${t("ladders.card.topped")}`;
		if (this.off) words = t("kit.tile.off.label", {label: words});
		return words;
	}

	update() {
		const tag = this.action ? "button" : "div";
		if (this.control?.localName !== tag) {
			this.control = h(tag, {class: "qp-tile-well", ...(tag === "button" ? {type: "button"} : {})});
			this.replaceChildren(this.control);
		}
		const face = this.face ?? "pose";
		const name = qualityName(this.quality);
		if (name) this.style.setProperty("--qp-tone", `var(--qp-quality-${name})`);
		else this.style.removeProperty("--qp-tone");
		this.dataset.size = this.size ?? "56";
		this.toggleAttribute("data-untrained", !name);
		this.toggleAttribute("data-ahead", face === "ahead");
		const still = (face === "pose" || face === "ahead") && this.move ? tileStill(this.move) : null;
		const parts = [];
		if (still) {
			const [x, y, side] = still.crop ?? [0, 0, 1];
			const img = h("img", {src: still.src, alt: "", crossorigin: "anonymous", draggable: "false"});
			img.style.setProperty("--qp-crop-x", String(x));
			img.style.setProperty("--qp-crop-y", String(y));
			img.style.setProperty("--qp-crop-side", String(side));
			img.classList.toggle("qp-mirrored", Boolean(still.mirror));
			parts.push(img);
		} else if (face !== "empty") parts.push(h("qp-art", {name: "action_pump", class: "qp-tile-glyph"}));
		if (this.mastered) {
			parts.push(h("span", {class: "qp-tile-mastered", "aria-hidden": "true"},
				h("qp-art", {name: "mark_check"})));
		}
		this.control.replaceChildren(...parts);
		// the Off, favourite and topped marks stand half outside the tile, so each is the tile's own child, never
		// clipped by the well (the Off and favourite marks were the well's until slice 42's fix round, 2026-10-17, which
		// showed a quarter of each)
		this.offEl ??= h("span", {class: "qp-tile-off", "aria-hidden": "true"});
		this.offEl.textContent = t("kit.tile.off");
		this.favouriteEl ??= h("span", {class: "qp-tile-favourite", "aria-hidden": "true"}, h("qp-art", {name: "star"}));
		this.toppedEl ??= h("span", {class: "qp-tile-topped", "aria-hidden": "true"},
			h("qp-art", {name: "mark_check"}));
		for (const [el, on] of [[this.offEl, this.off], [this.favouriteEl, this.favourite], [this.toppedEl, this.topped]]) {
			if (on) this.append(el);
			else el.remove();
		}
		this.classList.toggle("qp-off", this.off);
		const label = this.label();
		if (label) {
			this.control.setAttribute("aria-label", label);
			if (tag === "div") this.control.setAttribute("role", "img");
		} else {
			this.control.removeAttribute("aria-label");
			this.control.removeAttribute("role");
		}
	}
}

define("qp-move-tile", QpMoveTile, {
	move: String, quality: Number, size: String, face: String, off: Boolean, favourite: Boolean, name: String,
	word: String, step: Number, action: Boolean, mastered: Boolean, topped: Boolean,
});
