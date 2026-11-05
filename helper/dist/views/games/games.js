// The Games tab (docs/plan.md slices 56 and 57; docs/ui.md, "The Games tab"), the desktop app's fifth root, the one
// place for the games on this computer and the offers after a match, in one column from the `games` topic: Now, one
// card of the state (paused: the pause in words with Resume offers; a match's offer standing that nothing holds: the
// game's icon, Pump for <game> by the name the offer carries, as the tray and Pump home name it, and Start, the view's
// primary, this window's own click for that offer, with no Later; a watched game running: its icon and Playing <game>;
// else Watching <game> or Watching N games), or, with no game found and none added, the EmptyState No games yet with
// Add a game; Your games, a grid of GameTiles in the topic's order (running, on, off), each opening its game's window,
// last the Add a game tile; After a match: Moves per match
// (the body's match_pace, shared with Your setup and the phones, `games.pace`), Open the pump after each match
// (`games.open_on_close`), Pause offers (its state as the value, opening its window) and Send a test offer
// (`games.test`; why it waits while it is held); a last card: Notice games I play (`games.notice`), More games while
// one is listed and Hidden while a found game is put away (its count), each opening its window; and Riot's legal line
// while a Riot game is listed. Each time the tab comes to the front, and while it is in front as its dot comes, it says
// so (`games.seen`: the bar's dot goes). A switch asks Rust and shows the answer the topic brings. Rows and tiles stand
// from one change to the next and the tab draws again keeping the focus (`keepFocus`), so the keyboard's place holds
// while a line changes or the tiles' order does; a tile gone gives it to the one after it, else the one before, else
// Add a game.

import {intent} from "../../lib/bridge.js";
import {t} from "../../lib/copy.js";
import {keepFocus} from "../../lib/focus.js";
import {LOCALE} from "../../lib/format.js";
import {isFront, register} from "../../lib/router.js";
import {h} from "../../kit/element.js";
import {gameIcon} from "../../kit/qp-game-tile.js";
import {page, watch, words} from "../page.js";
import {gameName, gameWindow, hiddenWindow, moreWindow, pauseLine, pauseWindow, pickerWindow, place, portWindow,
	statusLine} from "./windows.js";

/** Moves per match's paces, each with its words. */
const PACES = [
	{value: "light", key: "games.pace.light"},
	{value: "balanced", key: "games.pace.balanced"},
	{value: "extreme", key: "games.pace.extreme"},
];

/** Moves per match: its Segmented with the pace chosen (Balanced while no device chose one), and `nodes`, the section
 * label over it, as Your setup shows it; a choice asks Rust (`games.pace`) and shows the answer the `games` topic
 * brings to `draw`. The Games tab and Your setup show it. */
export function paceRows() {
	const control = h("qp-segmented", {label: t("games.pace"), value: "balanced"});
	control.options = PACES.map((p) => ({value: p.value, label: t(p.key)}));
	control.addEventListener("qp-change", (e) => intent("games.pace", {pace: e.detail}).catch(console.error));
	return {
		control,
		nodes: [h("qp-section-label", {label: t("games.pace")}), control],
		draw(topic) {
			control.value = (PACES.find((p) => p.value === topic.pace) ?? PACES[1]).value;
		},
	};
}

/** A tile's state for its GameTile: running, on, or off. */
function tileState(tile) {
	if (tile.running) return "playing";
	return tile.on ? "on" : "off";
}

/** The Now card: its state's words, the game's icon (the offer's game's while a match's offer stands, else the running
 * game's) and its ways under them: Start while the offer stands, this window's own click for it as Pump home's Start
 * is, never the card's `outside` (DESK-PUMP's `pump.start {kind, offer}`, the pump at Ready at the size the match gave,
 * every game's offer alike since 2026-10-28; no Later, which would leave the offer standing here, on Pump home and in
 * the tray), and Resume offers while paused. The ways stand in
 * the card only in their state, so a way the state takes away leaves the page at once (a hidden one would keep the
 * focus until the next rendering update) and the tab's redraw gives its focus to the control now in its place
 * (`keepFocus`). */
function nowCard() {
	const icon = h("span", {class: "qp-games-now-icon"});
	const head = h("h2", {class: "qp-type-heading"});
	let offer = null; // the offer the card shows: {id, name, kind}
	const start = h("qp-primary-button", {label: t("pump.questdone.start"), "data-focus-key": "now:start",
		"on:click": () => intent("pump.start", {kind: offer.kind, offer: offer.id}).catch(console.error)});
	const resume = h("qp-secondary-button", {label: t("games.resume"), "data-focus-key": "now:resume",
		"on:click": () => intent("games.pause", {kind: "none"}).catch(console.error)});
	const ways = h("div", {class: "qp-games-now-ways"});
	const el = h("qp-card", {class: "qp-games-now"}, h("div", {class: "qp-games-now-head"}, icon, head), ways);
	let drawnIcon = null;
	let drawnWays = [];
	/** Puts the state's ways in the card, only when they changed, so one that stays keeps its focus. */
	const show = (shown) => {
		if (shown.length === drawnWays.length && shown.every((way, i) => way === drawnWays[i])) return;
		drawnWays = shown;
		ways.replaceChildren(...shown);
	};
	return {
		el,
		/** The view's primary: Start while the offer stands, else none. */
		primary: () => (drawnWays.includes(start) ? start : null),
		/** Draws the topic's `now` (and its pause's words while paused); hidden with every game off. */
		draw(topic) {
			const now = topic.now;
			el.hidden = now.state === "none" || (now.state === "watching" && !now.count);
			const ready = !el.hidden && now.state === "ready";
			const paused = !el.hidden && now.state === "paused";
			const named = (ready || now.state === "playing") && now.game;
			offer = ready ? now.offer : null;
			show(ready ? [start] : paused ? [resume] : []);
			if (el.hidden) return;
			icon.hidden = !named;
			if (named && drawnIcon !== `${now.game.icon ?? ""}\n${gameName(now.game)}`) {
				drawnIcon = `${now.game.icon ?? ""}\n${gameName(now.game)}`;
				icon.replaceChildren(gameIcon(now.game.icon, gameName(now.game)));
			}
			if (paused) head.textContent = pauseLine(topic.pause);
			else if (ready) head.textContent = t("games.pump", {game: offer.name});
			else if (named) head.textContent = t("games.now.playing", {game: gameName(now.game)});
			else if (now.count === 1 && now.game) head.textContent = t("desk.tray.state.watching", {game: gameName(now.game)});
			else head.textContent = t("desk.tray.state.watching.games", {count: now.count});
		},
	};
}

register("games", () => {
	const view = page({heading: t("bar.games"), root: true});
	view.el.classList.add("qp-games");
	const tiles = new Map(); // a tile's key -> its GameTile
	const grid = h("div", {class: "qp-move-grid qp-game-grid"});
	const add = h("qp-game-tile", {add: true, name: t("games.pick"), "on:qp-open": () => picker.open()});
	/** The tile the focus goes to once the tile of `key` is gone (Not a game, Remove): the one after it, else the one
	 * before, else Add a game. */
	const nearTile = (key) => {
		const shown = [...grid.children];
		const at = shown.indexOf(tiles.get(key));
		if (at < 0) return add;
		return (shown[at + 1] !== add && shown[at + 1]) || shown[at - 1] || add;
	};
	const port = portWindow();
	const game = gameWindow(port, nearTile);
	const more = moreWindow((key) => tiles.get(key));
	const picker = pickerWindow(more);
	const hidden = hiddenWindow((key) => tiles.get(key));
	const pause = pauseWindow();
	const windows = [game, port, picker, more, hidden, pause];

	const now = nowCard();
	const empty = h("qp-empty-state", {glyph: "action_games", heading: t("games.none"), action: t("games.pick"),
		"on:qp-action": () => picker.open()});
	const yoursLabel = h("qp-section-label", {label: t("games.yours")});

	const pace = paceRows();
	const open = h("qp-row", {heading: t("desk.offers.open"), glyph: "action_pump", trailing: "toggle",
		"on:qp-change": (e) => intent("games.open_on_close", {on: e.detail}).catch(console.error)});
	const pauseRow = h("qp-row", {heading: t("games.pause"), glyph: "mark_pause", trailing: "value",
		"on:click": () => pause.open()});
	const test = h("qp-row", {heading: t("games.test"), glyph: "mark_arrow", trailing: "action",
		"on:click": () => intent("games.test").catch(console.error)});

	const notice = h("qp-row", {heading: t("games.notice"), glyph: "action_games", trailing: "toggle",
		"on:qp-change": (e) => intent("games.notice", {on: e.detail}).catch(console.error)});
	const moreRow = h("qp-row", {heading: t("games.more"), glyph: "mark_list", trailing: "nav", hidden: true,
		"on:click": () => more.open()});
	const hiddenRow = h("qp-row", {heading: t("games.hidden"), glyph: "mark_dash", trailing: "value", hidden: true,
		"on:click": () => hidden.open()});
	const riot = words(t("about.riot"), "meta");
	riot.hidden = true;

	view.body.append(now.el, empty, yoursLabel, grid,
		h("qp-section-label", {label: t("games.after")}),
		h("qp-card", {class: "qp-games-pace"}, h("h2", {class: "qp-type-body-strong", text: t("games.pace")}),
			pace.control),
		h("qp-card", {rows: true}, open, pauseRow, test),
		h("qp-card", {rows: true}, notice, moreRow, hiddenRow),
		riot, ...windows.map((w) => w.el));

	/** A tile's GameTile, made once and kept while the topic lists it. */
	const tileOf = (tile) => {
		let el = tiles.get(tile.key);
		if (!el) {
			const key = tile.key;
			el = h("qp-game-tile", {"data-focus-key": `game:${key}`, "on:qp-open": () => game.open(key)});
			tiles.set(key, el);
		}
		el.name = gameName(tile);
		el.icon = tile.icon;
		el.state = tileState(tile);
		el.owed = statusLine(tile);
		return el;
	};

	/** Draws the tab from the topic. */
	const draw = (value) => {
		// with every game off nothing is watched: no Now card then, and the EmptyState only while no game is listed
		now.draw(value);
		empty.hidden = value.now.state !== "none" || value.tiles.length > 0;
		const shown = value.tiles.map(tileOf);
		for (const key of tiles.keys()) {
			if (!value.tiles.some((tile) => tile.key === key)) tiles.delete(key);
		}
		place(grid, [...shown, add]);
		// with no game at all the EmptyState's Add a game stands for the grid
		yoursLabel.hidden = !empty.hidden;
		grid.hidden = yoursLabel.hidden;
		pace.draw(value);
		open.checked = value.open_on_close;
		pauseRow.value = pauseLine(value.pause);
		test.disabled = value.held;
		test.meta = value.held ? t("games.test.held") : null;
		notice.checked = value.notice;
		moreRow.hidden = !value.more.length;
		hiddenRow.hidden = !value.hidden.length;
		hiddenRow.value = value.hidden.length ? new Intl.NumberFormat(LOCALE).format(value.hidden.length) : "";
		riot.hidden = !value.riot;
		for (const w of windows) w.draw?.(value);
	};
	watch(view, "games", (value) => {
		// a tile moved or gone keeps the focus: the same tile, else the one now in its place, else Add a game
		keepFocus(view.el, () => draw(value), (key, near) => near() ?? add);
		// a game found while the tab is in front is seen at once (the bar's dot goes)
		if (value.dot && isFront(view)) intent("games.seen").catch(console.error);
	});
	// the tab in front: what is new on it has been seen (the bar's dot goes)
	view.onShown = () => intent("games.seen").catch(console.error);
	// Pump home's recent games open a game's window here (the router's openGame)
	view.openGame = (key) => game.open(key);
	// a match's offer standing: its Start is the tab's one primary; with no game yet, Add a game
	view.primary = () => now.primary() ?? (empty.hidden ? null : empty.querySelector("qp-primary-button"));
	return view;
});
