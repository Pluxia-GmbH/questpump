// GameStat (desktop; Pump home's recent games, 2026-10-27; a small tile in a Carousel since 2026-10-28, Rapha: "i feel
// this could be a more compact one"): one game as one button firing qp-open: a small card of one width holding its icon
// (the GameTile's `gameIcon`: the program's own, else its name's first letter) with its `count` (Pump home's: the game's
// moves done this game week) beside it in the numbers' type, and its `name` under the card, centred and apart from it as
// the GameTile's (two lines at most; Rapha, 2026-10-28: "below the title separate like in the other one"). While `playing` a green dot sits on the icon's lower corner and Playing in small caps over the count. `owed`,
// the words of a step the game needs, puts the GameTile's pulsing amber dot on the icon's upper corner and is what a
// screen reader hears after its name; the button is heard as `spoken` (the caller's words: the name, Playing, the
// count with what it counts).

import {t} from "../lib/copy.js";
import {LOCALE} from "../lib/format.js";
import {KitElement, define, emit, h} from "./element.js";
import {gameIcon} from "./qp-game-tile.js";

const NUMBER = new Intl.NumberFormat(LOCALE);

class QpGameStat extends KitElement {
	build() {
		this.face = h("span", {class: "qp-game-face"});
		this.dot = h("span", {class: "qp-game-dot", "aria-hidden": "true"});
		this.live = h("span", {class: "qp-game-live", "aria-hidden": "true"});
		this.nameEl = h("span", {class: "qp-game-name", "aria-hidden": "true"});
		this.wordEl = h("span", {class: "qp-game-word", "aria-hidden": "true", text: t("games.tile.playing")});
		this.countEl = h("span", {class: "qp-game-stat-count", "aria-hidden": "true"});
		this.card = h("span", {class: "qp-game-stat-card"}, this.face, this.wordEl, this.countEl);
		this.control = h("button", {type: "button", class: "qp-game-stat", "on:click": () => emit(this, "qp-open")},
			this.card, this.nameEl);
		this.append(this.control);
	}

	update() {
		const name = this.name ?? "";
		// the icon drawn again only when it or the name changed, so a redraw never reloads the image
		const drawn = `${this.icon ?? ""}\n${name}`;
		if (drawn !== this.drawnIcon) {
			this.drawnIcon = drawn;
			this.face.replaceChildren(gameIcon(this.icon, name));
		}
		if (this.owed) this.face.append(this.dot);
		else this.dot.remove();
		if (this.playing) this.face.append(this.live);
		else this.live.remove();
		this.nameEl.textContent = name;
		this.wordEl.hidden = !this.playing;
		this.countEl.textContent = NUMBER.format(this.count ?? 0);
		this.dataset.state = this.playing ? "playing" : "on";
		this.control.setAttribute("aria-label", this.spoken ?? name);
		if (this.owed) this.control.setAttribute("aria-description", this.owed);
		else this.control.removeAttribute("aria-description");
	}
}

define("qp-game-stat", QpGameStat, {name: String, icon: String, count: Number, playing: Boolean, owed: String,
	spoken: String});
