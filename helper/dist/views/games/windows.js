// The Games tab's windows (docs/plan.md slice 56, "The game's window"; docs/ui.md, "The Games tab"): the game's window
// (its icon and name as its heading; its switch, which says when it offers, "Offer a pump after each match" or "when
// you close" the game, with the game's line while it is on and a step is owed; for a game heard through a file of
// QuestPump's not set up yet, while it is on, Ask after each match as its primary, `games.setup`; Windows' one-time
// folder grant while it asks; its Port row; Rest your arms while it runs for an aim game while it is on; Not a game for
// a found game, Remove for one the player added), Add a game (the games QuestPump knows whose program has a window now,
// each with Turn on, the programs with a window now and, at its foot, More games), More games (what a switch does, then
// the named games of this system not found on this computer, each with its switch), Hidden (the found games put away
// with Not a game, each with Show again), Pause offers (for an hour, for 4 hours, until 04:00, while the game running
// now runs, the first running tile, as Rust pauses for it, until you resume, and Resume offers) and a game's Port. Each
// draws from the `games` topic as it opens and on every change while it stands, so a change made elsewhere meanwhile is
// what it shows; a choice asks Rust and shows the answer the topic brings.

import {intent} from "../../lib/bridge.js";
import {t} from "../../lib/copy.js";
import {time} from "../../lib/format.js";
import {h} from "../../kit/element.js";
import {gameIcon} from "../../kit/qp-game-tile.js";
import {words} from "../page.js";

/** Each named game's words, by its key in contract/links.json's computer list: a named game's tile carries no name of
 * its own. */
export const NAMES = {
	counterstrike: "links.name.counterstrike",
	dota: "links.name.dota",
	league: "links.name.league",
	rocketleague: "links.name.rocketleague",
	starcraft: "links.name.starcraft",
	mtgarena: "links.name.mtgarena",
	hearthstone: "links.name.hearthstone",
	runeterra: "links.name.runeterra",
	formulaone: "links.name.formulaone",
	forza: "links.name.forza",
	iracing: "links.name.iracing",
	assettocorsa: "links.name.assettocorsa",
	elite: "links.name.elite",
	slaythespire: "links.name.slaythespire",
	assettocorsaone: "links.name.assettocorsaone",
	guildwars: "links.name.guildwars",
	mechabellum: "links.name.mechabellum",
	pathofexile: "links.name.pathofexile",
	warframe: "links.name.warframe",
	warships: "links.name.warships",
	siege: "links.name.siege",
	tarkov: "links.name.tarkov",
	deadlock: "links.name.deadlock",
	fortnite: "links.name.fortnite",
	teamfortress: "links.name.teamfortress",
	naraka: "links.name.naraka",
	fallguys: "links.name.fallguys",
	deadbydaylight: "links.name.deadbydaylight",
	hunt: "links.name.hunt",
	valorant: "links.name.valorant",
	pubg: "links.name.pubg",
	halo: "links.name.halo",
	brawlhalla: "links.name.brawlhalla",
	rfactor: "links.name.rfactor",
	lemans: "links.name.lemans",
	automobilista: "links.name.automobilista",
	osu: "links.name.osu",
	stardew: "links.name.stardew",
	warcraft: "links.name.warcraft",
	eldenring: "links.name.eldenring",
	nightreign: "links.name.nightreign",
};

/** A game's name (a tile's, the Now card's game's, the pause's game's): its own (a found game's, a program the player
 * added), else a named game's words, so every place names a game alike. */
export function gameName(game) {
	return game.name ?? (NAMES[game.key] ? t(NAMES[game.key]) : game.key);
}

/** A game's line by the step it owes (its other states, off, not found, waiting for its start and ready, owe none and
 * say nothing); the lines that name the game take it as {game}, a racing game's port as {port}. */
const STEPS = {
	restart: "games.status.restart",
	launch: "games.status.launch",
	permission: "games.status.permission",
	asking: "games.status.asking",
	inioff: "games.status.inioff",
	failed: "games.status.failed",
	apioff: "games.status.api",
	udp: "games.status.udp",
	porttaken: "games.status.porttaken",
	udpformat: "games.status.udpformat",
	nojournal: "games.status.nojournal",
	noshared: "games.status.noshared",
};

/** A game whose words for a step are its own (Forza Motorsport's Data Out is not F1's UDP Telemetry; Deadlock's and Team
 * Fortress 2's launch option, `-condebug`, is not Dota 2's; Automobilista 2 shares its race data only once its Shared
 * Memory is set to Project CARS 2). */
const OWN_STATUS = {
	forza: {udp: "games.status.udp.forza"},
	deadlock: {launch: "games.status.launch.condebug"},
	teamfortress: {launch: "games.status.launch.condebug"},
	automobilista: {noshared: "games.status.noshared.ams2"},
};

/** The line of the step a game owes while it is on (its tile's dot, its window's line), or null. */
export function statusLine(game) {
	const key = OWN_STATUS[game.key]?.[game.status] ?? STEPS[game.status];
	// a port as it is written, never grouped by the locale's digits
	return game.on && key ? t(key, {game: gameName(game), port: String(game.port ?? "")}) : null;
}

/** The pause's state in words. */
export function pauseLine(pause) {
	switch (pause.kind) {
		case "until": return t("games.paused.until", {time: time(pause.at * 1000)});
		case "while_running": return t("desk.offers.paused.game", {game: gameName(pause)});
		case "until_resumed": return t("games.paused.resumed");
		default: return t("desk.offers.paused.none");
	}
}

/** Every game the topic lists, the tiles and More games. */
function listed(topic) {
	return [...topic.tiles, ...topic.more];
}

/** Puts `nodes` into `parent` in their order, unless they stand so already: a node taken out loses the focus. */
export function place(parent, nodes) {
	const now = [...parent.children];
	if (now.length !== nodes.length || now.some((node, i) => node !== nodes[i])) parent.replaceChildren(...nodes);
}

/** The lowest and highest port the Port window takes (the system keeps the ports below). */
const PORTS = [1024, 65535];

/** A game's Port window: one field, kept by Done (its × drops it); drawn for a game as it opens. A port outside PORTS,
 * or one another game's feed takes (the desktop app refuses it too), keeps nothing and says why under the field. */
export function portWindow() {
	const el = h("qp-window", {heading: t("games.port"), primary: t("common.done")});
	const field = h("qp-text-input", {mode: "line", label: t("games.port"), inputmode: "numeric"});
	el.content.append(field);
	let key = null;
	let games = [];
	const keep = () => {
		const typed = field.value.trim();
		const port = Number(typed);
		if (!key) return;
		if (!/^\d+$/.test(typed) || port < PORTS[0] || port > PORTS[1]) {
			field.error = t("games.port.invalid");
			return;
		}
		const other = games.find((game) => game.key !== key && game.port === port);
		if (other) {
			field.error = t("games.port.taken", {game: gameName(other)});
			return;
		}
		intent("games.port", {key, port}).catch(console.error);
		el.close();
	};
	el.addEventListener("qp-primary", keep);
	field.addEventListener("qp-submit", keep);
	field.addEventListener("input", () => {
		field.error = null;
	});
	const open = (game, topic) => {
		key = game.key;
		games = listed(topic);
		field.value = String(game.port);
		field.error = null;
		el.open();
	};
	return {el, open};
}

/** A game's window, opened by its key (`open`), drawn from the topic until it closes, closed once the topic lists the
 * game no more (shown again, removed). Its Port row opens `port` over it. After Not a game or Remove the focus goes to
 * the tile `nearOf(key)` names (beside the game's own) once the window and the game's tile have both gone; a tile the
 * tab still draws as the window closes takes the focus back, and the tab's redraw moves it on. */
export function gameWindow(port, nearOf) {
	const el = h("qp-window", {class: "qp-game-window"});
	const icon = h("span", {class: "qp-game-window-icon"});
	const head = h("h2", {class: "qp-type-heading"});
	let key = null;
	let shown = null; // the game as the window last drew it
	let topic = null;
	const offer = h("qp-row", {glyph: "action_pump", trailing: "toggle",
		"on:qp-change": (e) => intent("games.set", {key, on: e.detail}).catch(console.error)});
	const grant = h("qp-row", {heading: t("games.grant"), glyph: "mark_lock", trailing: "action", hidden: true,
		"on:click": () => intent("games.grant", {key}).catch(console.error)});
	const portRow = h("qp-row", {heading: t("games.port"), glyph: "mark_link", trailing: "value", hidden: true,
		"on:click": () => shown && port.open(shown, topic)});
	const aim = h("qp-row", {heading: t("games.aim"), glyph: "mark_rest", trailing: "toggle", hidden: true,
		"on:qp-change": (e) => intent("games.aim", {key, on: e.detail}).catch(console.error)});
	// Not a game for a found game, Remove for an added one: either takes the game off the tab and closes the window
	const awayRow = h("qp-row", {glyph: "mark_close", trailing: "action", "on:click": () => {
		if (!shown) return;
		const gone = key;
		const near = nearOf(gone);
		if (shown.kind === "found") intent("games.dismiss", {key}).catch(console.error);
		else intent("games.remove", {key}).catch(console.error);
		el.addEventListener("qp-closed", () => {
			if (topic && !listed(topic).some((game) => game.key === gone)) near.focus();
		}, {once: true});
		el.close();
	}});
	const away = h("qp-card", {rows: true}, awayRow);
	el.content.append(h("div", {class: "qp-game-window-head"}, icon, head),
		h("qp-card", {rows: true}, offer, grant, portRow, aim), away);
	el.addEventListener("qp-primary", () => intent("games.setup", {key}).catch(console.error));
	let drawnIcon = null;
	const draw = (value) => {
		topic = value;
		if (!key) return;
		const game = listed(value).find((g) => g.key === key);
		if (!game) {
			el.close();
			return;
		}
		shown = game;
		const name = gameName(game);
		el.label = name;
		head.textContent = name;
		if (drawnIcon !== `${game.icon ?? ""}\n${name}`) {
			drawnIcon = `${game.icon ?? ""}\n${name}`;
			icon.replaceChildren(gameIcon(game.icon, name));
		}
		// the switch says when the game offers: each match's end, or its program closing (Rust's `Tile::ends`)
		offer.heading = game.ends === "match" ? t("games.offer.match") : t("games.offer.close", {game: name});
		offer.checked = game.on;
		offer.meta = statusLine(game);
		// the four games heard through a file of QuestPump's, while the player has not asked for it yet (`setup`: Ask
		// after each match stands) and the game is on
		el.primary = game.on && game.setup ? t("games.setup") : null;
		grant.hidden = !game.on || !["permission", "asking"].includes(game.status);
		grant.disabled = game.status === "asking";
		portRow.hidden = !game.on || game.port === null;
		// a port as it is written, never grouped by the locale's digits
		portRow.value = game.port === null ? "" : String(game.port);
		aim.hidden = !game.on || game.aim === null;
		aim.checked = Boolean(game.aim);
		away.hidden = game.kind === "named";
		awayRow.heading = t(game.kind === "found" ? "games.found.dismiss" : "games.program.remove");
		awayRow.destructive = game.kind === "added";
	};
	el.addEventListener("qp-closed", () => {
		key = null;
		shown = null;
	});
	const open = (gameKey) => {
		key = gameKey;
		if (topic) draw(topic);
		el.open();
	};
	return {el, open, draw};
}

/** Add a game: the games QuestPump knows by name one of whose programs has a window now and that it does not watch
 * (Rust's `pick_games`: League of Legends for its client or the Riot Client), each by its name with Turn on, which
 * turns it on (`games.set`) in place of watching its launcher; then the
 * programs with a window now, fresh from Rust as it opens (`games.pick`; the topic brings them), a pick watching that
 * program (`games.add`); either closes the window. At its foot, while a game is listed there, More games: the window
 * closes and More games opens (`more`), so a player looking for a game QuestPump knows finds it from here, the focus
 * going back to Add a game once More games closes. */
export function pickerWindow(more) {
	const el = h("qp-window", {heading: t("games.pick")});
	const named = h("qp-card", {rows: true});
	const list = h("qp-card", {rows: true});
	const none = words(t("games.pick.none"), "meta");
	const moreRow = h("qp-row", {heading: t("games.more"), meta: t("games.pick.more"), glyph: "mark_list",
		trailing: "nav", "on:click": () => {
			el.addEventListener("qp-closed", () => more.open(), {once: true});
			el.close();
		}});
	el.content.append(named, list, none);
	el.foot.append(h("qp-card", {rows: true}, moreRow));
	let drawn = undefined; // the picks' keys and the games' as the lists last drew them
	const draw = (topic) => {
		el.foot.hidden = !topic.more.length;
		const games = (topic.pick_games ?? []).map((key) => listed(topic).find((game) => game.key === key))
			.filter(Boolean);
		const picks = topic.picks ? [...games, ...topic.picks].map((p) => p.key).join("\n") : null;
		if (picks === drawn) return;
		drawn = picks;
		named.replaceChildren(...games.map((game) => h("qp-row", {heading: gameName(game), value: t("games.tile.on"),
			glyph: "action_games", trailing: "action", "on:click": () => {
				intent("games.set", {key: game.key, on: true}).catch(console.error);
				el.close();
			}})));
		named.hidden = !games.length;
		list.replaceChildren(...(topic.picks ?? []).map((pick) => h("qp-row", {heading: pick.name, meta: pick.image,
			glyph: "action_games", trailing: "action", "on:click": () => {
				intent("games.add", {key: pick.key}).catch(console.error);
				el.close();
			}})));
		list.hidden = !topic.picks?.length;
		none.hidden = !topic.picks || topic.picks.length > 0 || games.length > 0;
	};
	const open = () => {
		intent("games.pick").catch(console.error);
		el.open();
	};
	return {el, open, draw};
}

/** More games: a line on what a switch does (Rapha, 2026-10-27: "you dont expect that these games give you pumps"),
 * then the named games of this system not found on this computer, each with its switch (`games.set`), for an install
 * QuestPump does not find; a switch only says whether QuestPump watches for the game, never moves it to Your games
 * (Rust's `Games::listed`). It closes once none is left. The focus moves on from a row whose game leaves the list
 * (found on this computer meanwhile, a tile then) as Hidden's does: to the row after it, to the window's × after the
 * last row, and, when none is left and the window closes, to that game's tile (`tileOf(key)`). */
export function moreWindow(tileOf) {
	const el = h("qp-window", {heading: t("games.more")});
	const list = h("qp-card", {rows: true});
	el.content.append(words(t("games.more.intro"), "body"), list);
	const rows = new Map(); // a game's key -> its row
	const draw = (topic) => {
		const keys = topic.more.map((game) => game.key);
		// the row that holds the focus and the row after it, as the list stands before this draw
		const was = [...rows].find(([, row]) => row.contains(document.activeElement))?.[0];
		const shownRows = [...list.children];
		const next = was ? shownRows[shownRows.indexOf(rows.get(was)) + 1] ?? null : null;
		const nodes = topic.more.map((game) => {
			let row = rows.get(game.key);
			if (!row) {
				const key = game.key;
				row = h("qp-row", {glyph: "action_games", trailing: "toggle",
					"on:qp-change": (e) => intent("games.set", {key, on: e.detail}).catch(console.error)});
				rows.set(key, row);
			}
			row.heading = gameName(game);
			row.checked = game.on;
			return row;
		});
		for (const key of rows.keys()) {
			if (!keys.includes(key)) rows.delete(key);
		}
		place(list, nodes);
		if (was && !keys.includes(was)) {
			if (el.closable() && keys.length) (next?.isConnected ? next : el.closer).focus();
			else if (el.closable()) el.addEventListener("qp-closed", () => tileOf(was)?.focus(), {once: true});
		}
		if (!nodes.length) el.close();
	};
	return {el, open: () => el.open(), draw};
}

/** Hidden: the found games put away with Not a game, each with Show again (`games.restore`). The focus moves on from a
 * row whose game shows again: to the row after it, to the window's × after the last row, and, when none is left and the
 * window closes, to that game's tile (`tileOf(key)`). */
export function hiddenWindow(tileOf) {
	const el = h("qp-window", {heading: t("games.hidden")});
	const list = h("qp-card", {rows: true});
	el.content.append(list);
	const rows = new Map(); // a hidden game's key -> its row
	let after = null; // the game shown again and the row after it, once the topic took it off the list
	const restore = (key, row) => {
		const shownRows = [...list.children];
		after = {key, next: shownRows[shownRows.indexOf(row) + 1] ?? null};
		intent("games.restore", {key}).catch(console.error);
	};
	const draw = (topic) => {
		const keys = topic.hidden.map((game) => game.key);
		const nodes = topic.hidden.map((game) => {
			let row = rows.get(game.key);
			if (!row) {
				const key = game.key;
				row = h("qp-row", {value: t("games.hidden.restore"), glyph: "action_games", trailing: "action",
					"on:click": () => restore(key, row)});
				rows.set(key, row);
			}
			row.heading = game.name;
			return row;
		});
		for (const key of rows.keys()) {
			if (!keys.includes(key)) rows.delete(key);
		}
		place(list, nodes);
		if (after && !keys.includes(after.key)) {
			const gone = after;
			after = null;
			if (el.closable() && keys.length) (gone.next?.isConnected ? gone.next : el.closer).focus();
			else if (el.closable()) el.addEventListener("qp-closed", () => tileOf(gone.key)?.focus(), {once: true});
		}
		// the last hidden game shown again leaves nothing to show
		if (!keys.length) el.close();
	};
	return {el, open: () => el.open(), draw};
}

/** Pause offers: its state, each pause as a row that takes it and closes the window (`games.pause`), and Resume offers
 * while one stands; "while the game runs" only while a watched game runs, naming the first running tile, the game Rust
 * pauses for. */
export function pauseWindow() {
	const el = h("qp-window", {heading: t("games.pause")});
	const state = words("", "meta");
	const pick = (kind) => {
		intent("games.pause", {kind}).catch(console.error);
		el.close();
	};
	const row = (kind, glyph = "mark_pause") => h("qp-row", {glyph, trailing: "action", "on:click": () => pick(kind)});
	const hour = row("hour");
	const four = row("four_hours");
	const night = row("night");
	const game = row("game");
	const resumed = row("resumed");
	const resume = row("none", "mark_play");
	const resumeCard = h("qp-card", {rows: true}, resume);
	el.content.append(state, h("qp-card", {rows: true}, hour, four, night, game, resumed), resumeCard);
	const draw = (topic) => {
		const running = topic.tiles.find((tile) => tile.running);
		state.textContent = pauseLine(topic.pause);
		hour.heading = t("games.pause.hour");
		four.heading = t("games.pause.four");
		night.heading = t("games.pause.night", {time: time(topic.night * 1000)});
		game.heading = running ? t("desk.offers.pause.game", {game: gameName(running)})
			: t("desk.offers.pause.game.none");
		game.disabled = !running;
		resumed.heading = t("games.pause.resumed");
		resume.heading = t("games.resume");
		resumeCard.hidden = topic.pause.kind === "none";
	};
	return {el, open: () => el.open(), draw};
}
