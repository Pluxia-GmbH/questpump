// GameTile (desktop; the Games tab's grid, docs/plan.md slice 56): a game's own icon (`icon`, the asset URL of the icon
// Rust read from its program; while it has none, or the file does not load, the first letter of its `name` on the kit's
// raised surface) on a 60 well over its name, and one thing more at most, as its `state` says: `playing` (the word
// Playing), `off` (the tile dimmed, with the word Off) or `on` (nothing more). `owed`, the words of a step the game
// still owes (start it again, a launch option, Windows' grant), puts the pulsing amber dot of the Chip's `reaching` on
// the icon's corner and is what a screen reader hears after the tile's name. The tile is one button firing qp-open (the
// game's window), heard as its name and its word ("Dota 2, Off"). `add` makes it the grid's last tile, Add a game: a
// plus in the well and `name` its words.

import {t} from "../lib/copy.js";
import {LOCALE} from "../lib/format.js";
import {KitElement, define, emit, h} from "./element.js";

const GRAPHEMES = new Intl.Segmenter(undefined, {granularity: "grapheme"});

/** A name's first letter, upper case (one grapheme: an accented letter stays whole). */
function firstLetter(name) {
	const first = GRAPHEMES.segment((name ?? "").trim())[Symbol.iterator]().next().value;
	return first ? first.segment.toLocaleUpperCase(LOCALE) : "";
}

/** A game's icon on its well, decoration: the image at `icon`, else (none, or one that does not load) the first letter
 * of `name` on the kit's raised surface. The Now card and the game's window show it too. */
export function gameIcon(icon, name) {
	const well = h("span", {class: "qp-game-icon", "aria-hidden": "true"});
	const letter = () => well.replaceChildren(h("span", {class: "qp-game-letter", text: firstLetter(name)}));
	if (icon) {
		well.append(h("img", {src: icon, alt: "", draggable: "false", "on:error": letter}));
	} else {
		letter();
	}
	return well;
}

/** Each state's word on the tile (none while it is on). */
const WORDS = {playing: "games.tile.playing", off: "games.tile.off"};

class QpGameTile extends KitElement {
	build() {
		this.face = h("span", {class: "qp-game-face"});
		this.nameEl = h("span", {class: "qp-game-name", "aria-hidden": "true"});
		this.wordEl = h("span", {class: "qp-game-word", "aria-hidden": "true"});
		this.control = h("button", {type: "button", class: "qp-game-tile", "on:click": () => emit(this, "qp-open")},
			this.face, this.nameEl, this.wordEl);
		this.dot = h("span", {class: "qp-game-dot", "aria-hidden": "true"});
		this.append(this.control);
	}

	update() {
		const state = this.add ? "on" : this.state ?? "on";
		const name = this.name ?? "";
		// the icon drawn again only when it or the name changed, so a redraw never reloads the image
		const drawn = this.add ? "+" : `${this.icon ?? ""}\n${name}`;
		if (drawn !== this.drawnIcon) {
			this.drawnIcon = drawn;
			const well = this.add ? h("span", {class: "qp-game-icon qp-game-add", "aria-hidden": "true"})
				: gameIcon(this.icon, name);
			this.face.replaceChildren(well);
		}
		if (this.owed && !this.add) this.face.append(this.dot);
		else this.dot.remove();
		this.nameEl.textContent = name;
		const word = WORDS[state] ? t(WORDS[state]) : "";
		this.wordEl.textContent = word;
		this.wordEl.hidden = !word;
		this.dataset.state = state;
		this.control.setAttribute("aria-label", word ? `${name}, ${word}` : name);
		if (this.owed && !this.add) this.control.setAttribute("aria-description", this.owed);
		else this.control.removeAttribute("aria-description");
	}
}

define("qp-game-tile", QpGameTile, {name: String, icon: String, state: String, owed: String, add: Boolean});
