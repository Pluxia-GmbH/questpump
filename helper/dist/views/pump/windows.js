// The pump's windows on the computer (the phones' PumpWindows and StartWindow drawn once more): those a stage opens
// (the "···" past the Ready view, the swap row's more ways to swap, How to do it, the Ready bar's Pause menu, leaving
// the pump) and those Rust asks for through the `pump` and `shell` topics, whichever view is in front (Start a pump with
// its preset, "Leave the pump?" when the window closes or QuestPump quits mid-pump, and "Speakers or headset?" before
// the first pump). Every window closes with Escape or its ×, which never ends, banks or skips a pump.

import {get, intent, subscribe} from "../../lib/bridge.js";
import {t} from "../../lib/copy.js";
import {figureName} from "../../lib/figure.js";
import {topWindow} from "../../lib/focus.js";
import {time} from "../../lib/format.js";
import {openMenu, whenStarted} from "../../lib/router.js";
import {h} from "../../kit/element.js";
import {gameName} from "../games/windows.js";
import {openPlaces} from "../places/places.js";
import {gearLine, placeName, placeShown} from "../places/words.js";
import {kindName, recordWords, sizes} from "./words.js";

/** An intent's failure goes to the console (Rust's handler logged why). */
function heard(promise) {
	promise.catch(console.error);
}

/** Opens a window (`props`: its heading, placement, locked, primary), filled by `fill(content, win)`; it leaves the page
 * once closed, after `closed()`. */
export function openWindow(props, fill, closed) {
	const win = h("qp-window", props);
	win.ensure();
	fill(win.content, win);
	win.addEventListener("qp-closed", () => {
		win.remove();
		closed?.();
	}, {once: true});
	win.open();
	return win;
}

/** A Row that closes its window, then acts. */
function row(win, props, act) {
	return h("qp-row", {...props, "on:click": () => {
		win.close();
		act();
	}});
}

/** Leave keeps a move (`pump_stop`, the Android phone's `keepsOnLeave`): one done; a pose, a steady move or a hold's
 * saved count waiting at its ask, which an end keeps with no answer; an app pump's own counted set at its count or its
 * whole-move ask, kept done with Form slipped (never a rider's); or a steady move Keep going ran past its plan, saved
 * as its Done would (`pump_kept`). */
function anyDone(pump) {
	const m = pump.moves[pump.current];
	const counted = !m?.rider && m?.counted && (pump.stage === "counting" || pump.stage === "asking");
	const kept = pump.stage === "running" && pump.kept && Date.now() - pump.start >= (m?.seconds ?? 0) * 1000;
	return ["marking", "pacing", "longer"].includes(pump.stage) || counted || kept
		|| pump.moves.some((x) => x.state === "done");
}

/** The "···" (from the bottom) on every stage past the Ready view, whose own bar holds its ways (since 2026-11-05): How
 * to do it, Done without a count on the count but a counted set's form step (its three answers save it Not counted,
 * as the game's and the phones' do; Rust refuses it there), End the pump here (and an expedition's End the timed
 * pump), which reads Bank the rest on a rider, a catch-up's move or a match pump's (slices 58 and 61; End's action: a
 * running timed move's seconds paid, the rest left unfinished), and last "I feel unwell", in the stop colour, what it
 * does read after its name. */
export function more(pump, open) {
	return openWindow({heading: t("kit.more"), placement: "bottom"}, (content, win) => {
		const rows = [row(win, {heading: t("pump.more.how"), glyph: "action_character"}, () => open("move"))];
		if (pump.stage === "counting" && !pump.moves[pump.current]?.counted) {
			rows.push(row(win, {heading: pump.count.empty, glyph: "mark_check", trailing: "action"},
				() => heard(intent("pump.count.none"))));
		}
		rows.push(row(win, {heading: t(pump.rest ? "pump.ready.bank.rest" : "pump.ready.end"), glyph: "mark_close",
			trailing: "action"}, () => heard(intent("pump.end"))));
		// an expedition's round: its end too, which ends on the Result of the rounds done (slice 51)
		if (pump.what === "expedition") {
			rows.push(row(win, {heading: t("expedition.stop"), glyph: "mark_close", trailing: "action"},
				() => heard(intent("expedition.stop"))));
		}
		rows.push(row(win, {heading: t("pump.unwell"), hint: t("pump.unwell.hint"), glyph: "mark_heart",
			trailing: "action", destructive: true}, () => heard(intent("pump.unwell"))));
		content.append(h("qp-card", {rows: true}, ...rows));
	});
}

/** The swap row's more ways to swap (Panel.lua's Swap menu, under the game's "Swap this move", Rust's `title`): every
 * offer but the row's own, in the game's order, each its tile, its name and the kind of swap in the game's words (Back
 * to, Same level or Closest your setup allows, Easier version, Another move); one click swaps it in. */
export function swap(pump) {
	return openWindow({heading: pump.swap.title, placement: "bottom"}, (content, win) => {
		content.append(h("qp-card", {rows: true}, ...pump.swap.options.slice(1).map((o) => row(win,
			{heading: o.name, meta: o.kind, move: o.key, quality: o.quality, trailing: "action"},
			() => heard(intent("pump.swap", {take: o.take}))))));
	});
}

/** How to do it for the move on: its figure (the other way round on a side move's other side), its quality and record
 * (a loaded move's weighted record, as its Ready view names it), How and Tips as its Ready view shows them (Rust's
 * `setup`, `how` and `tips`) and its stop line. */
export function move(pump) {
	const m = pump.moves[pump.current];
	const shown = h("qp-move-window", {heading: m.name, quality: m.quality, "quality-word": m.word,
		record: recordWords(m), setup: pump.setup, stop: m.stop});
	shown.how = pump.how;
	shown.tips = pump.tips;
	shown.ensure();
	shown.stage.setAttribute("move", figureName(m));
	if (pump.side === "other") shown.stage.setAttribute("side", "other");
	shown.addEventListener("qp-closed", () => shown.remove(), {once: true});
	document.body.append(shown);
	shown.open();
	return shown;
}

/** The Ready bar's Pause (2026-11-05; the game's Panel.PauseMenu, opened above its button `from`): how long, the
 * Games tab's Pause offers' choices under its name (an hour, four hours, until the next game day, while the watched
 * game running now runs, while one runs, until resumed; none where this computer holds no offers or on a work pump,
 * `topic.work`), each pausing them and ending the pump as its bar's Close does (`pump.pause`); then Your body:
 * Something hurts while a joint is left unflagged, its joints in a menu beside it under the game's question, one click
 * flagging one (`pump.pain`), and I feel unwell last, in the stop colour, what it does read after its name
 * (`pump.unwell`). */
export function pauseMenu(topic, from) {
	// a work pump pauses no offer (Stop ends work mode, Later puts a pump off): its menu is Your body alone (slice 67)
	const games = topic.work ? null : get("games");
	const running = games?.tiles?.find((tile) => tile.running);
	const pause = (kind) => () => heard(intent("pump.pause", {kind}));
	const items = games ? [
		{group: t("games.pause")},
		{label: t("games.pause.hour"), act: pause("hour")},
		{label: t("games.pause.four"), act: pause("four_hours")},
		{label: t("games.pause.night", {time: time(games.night * 1000)}), act: pause("night")},
		running ? {label: t("desk.offers.pause.game", {game: gameName(running)}), act: pause("game")} : null,
		{label: t("games.pause.resumed"), act: pause("resumed")},
	] : [];
	const joints = topic.pain.filter((j) => !j.flagged);
	items.push({group: t("pump.ready.body")}, joints.length ? {label: t("pump.more.pain"), submenu: {
		label: topic.pain_ask,
		items: joints.map((j) => ({label: j.name, act: () => heard(intent("pump.pain", {joints: [j.id]}))})),
	}} : null, {label: t("pump.unwell"), hint: t("pump.unwell.hint"), destructive: true,
		act: () => heard(intent("pump.unwell"))});
	const menu = h("qp-menu", {label: t("pump.ready.pause")});
	menu.items = items.filter(Boolean);
	menu.open(from);
	return menu;
}

/** Leaving the pump before its end (its ×, the window's close, Quit QuestPump): what stays of it, in a catch-up's
 * words where only riders are left (`rest`, slice 61); Leave ends it (`then`: the window closes or QuestPump quits
 * after), its × stays. */
export function leave(pump, then) {
	const done = anyDone(pump);
	const words = pump.rest ? (done ? t("pump.leave.owed") : t("pump.leave.owed.none"))
		: done ? t("pump.leave.free") : t("pump.leave.free.none");
	let left = false;
	return openWindow({heading: t("pump.leave.title")}, (content, win) => {
		content.append(h("p", {class: "qp-type-body qp-text2", text: words}), h("qp-secondary-button",
			{label: t("pump.leave.confirm"), "on:click": () => {
				left = true;
				if (then) heard(intent("pump.leave", {then}));
				else heard(intent("pump.end"));
				win.close();
			}}));
	}, () => {
		if (then && !left) heard(intent("pump.stay"));
	});
}

// The windows Rust asks for ----------------------------------------------------------------------------------------

const asked = {choose: null, leave: null, output: null};
/** An ask whose window was shown, held until the topic clears it, so a window closed before Rust's answer arrives is
 * never opened twice. */
const held = {};
let leaveSeq = 0;
let pumpId = null;

/** Start a pump (the phones' StartWindow): what the player finished as the six plain kinds in ChoiceChips, the size as
 * a Segmented (the size Rust preset: an offer's tier, else Balanced), the place in use (slice 51: its Row opens the
 * places window; while work mode runs Work mode with its equipment, slice 67, which closes the window and opens the
 * Work mode page) and Start; a match's offer chooses A match, a program's close none. A pump from an offer, or of A
 * match, is a match's pump (slice 48's double-check): while every size takes the same moves (`one_size`: Moves per
 * match at Light) no size shows, the preset size still sent, and while an aim game rests the arms (`arms`, its name)
 * the window says so. */
function choose(c) {
	let kind = c.kinds.includes(c.kind) ? c.kind : null;
	let size = String(c.size ?? 2);
	let shownWin = null;
	const placeRow = h("qp-row", {tile: true, "on:click": () => {
		if (!placeShown(get("places"))?.work) return openPlaces();
		shownWin?.close();
		openMenu("desk.menu.work");
	}});
	const stop = subscribe("places", (p) => {
		const shown = placeShown(p);
		placeRow.heading = placeName(shown);
		placeRow.meta = gearLine(shown);
		placeRow.glyph = shown?.glyph ?? null;
	});
	const matched = () => Boolean(c.offer) || kind === "match";
	shownWin = openWindow({heading: t("pump.questdone.title"), placement: "bottom", primary: t("pump.questdone.start"),
		"primary-state": kind ? "normal" : "disabled"}, (content, win) => {
		const sizeHead = h("p", {class: "qp-type-caps", text: t("pump.questdone.size")});
		const segmented = h("qp-segmented", {value: size});
		segmented.options = sizes();
		segmented.addEventListener("qp-change", (e) => {
			size = e.detail;
			segmented.value = size;
		});
		const rest = c.arms ? h("p", {class: "qp-type-body", text: t("pump.rest.arms", {game: c.arms})}) : null;
		const draw = () => {
			const one = c.one_size && matched();
			sizeHead.hidden = one;
			segmented.hidden = one;
			if (rest) rest.hidden = !matched();
		};
		const chips = c.kinds.map((key) => {
			const chip = h("qp-choice-chip", {label: kindName(key), selected: key === kind});
			chip.addEventListener("click", () => {
				kind = key;
				for (const other of chips) other.selected = other === chip;
				win.primaryState = "normal";
				draw();
			});
			return chip;
		});
		draw();
		content.append(...[h("p", {class: "qp-type-caps", text: t("pump.questdone.kind")}),
			h("div", {class: "qp-pump-chips"}, ...chips), sizeHead, segmented, rest,
			h("p", {class: "qp-type-caps", text: t("places.row")}), h("qp-card", {rows: true}, placeRow),
		].filter(Boolean));
		win.addEventListener("qp-primary", () => {
			if (kind) heard(intent("pump.start", {kind, size: Number(size), offer: c.offer}));
		});
	}, () => {
		stop();
		heard(intent("pump.dismiss", {window: "choose"}));
	});
	return shownWin;
}

/** "Speakers or headset?" before the first pump on this computer: the output picker asking it (its question heads the
 * window), closed once an output is picked; closed without one, the system's sound stays and nothing asks again
 * (`sound.picked`). */
function output() {
	let picked = false;
	return openWindow({label: t("desk.sound.ask")}, (content, win) => {
		const picker = h("qp-output-picker", {ask: true});
		picker.addEventListener("qp-picked", () => {
			picked = true;
			win.close();
		});
		content.append(picker);
	}, () => {
		if (!picked) heard(intent("sound.picked"));
	});
}

/** A window Rust asks for: open while `wanted`, once until the topic clears it, closed when it no longer is. */
function ask(name, wanted, make) {
	if (!wanted) {
		held[name] = false;
		asked[name]?.close();
		return;
	}
	if (asked[name] || held[name]) return;
	held[name] = true;
	asked[name] = make();
	asked[name].addEventListener("qp-closed", () => {
		asked[name] = null;
	}, {once: true});
}

function followPump(topic) {
	ask("choose", topic.choose, () => choose(topic.choose));
	const pump = topic.pump;
	const first = pump && pump.id !== pumpId && pump.stage === "ready" && get("shell")?.output_picked === false;
	if (first && !asked.output) {
		asked.output = output();
		asked.output.addEventListener("qp-closed", () => {
			asked.output = null;
		}, {once: true});
	}
	pumpId = pump?.id ?? null;
	if (asked.leave && !(pump && pump.stage !== "done")) asked.leave.close();
}

function followShell(shell) {
	const pump = get("pump")?.pump;
	if (!shell.leave || shell.leave.seq <= leaveSeq) return;
	leaveSeq = shell.leave.seq;
	if (!pump || pump.stage === "done" || asked.leave) return;
	asked.leave = leave(pump, shell.leave.then);
	asked.leave.addEventListener("qp-closed", () => {
		asked.leave = null;
	}, {once: true});
}

/** Tells Rust when a window comes over the page and when the last one leaves it (`pump.window`), for a move's pour to
 * wait on (`Pour::cover`; David, 2026-11-04, QA-8): any window, those a stage opened and those Rust asks for alike, one
 * that goes with its page or without closing too, which is why the tree is watched while one stands (a window a page
 * holds leaves with that page, its `open` unchanged). A window opened from another stays over the page through the
 * change, since the new one stands before the old has left. A page that starts has none, which frees what a page that
 * went left waiting. */
function watchWindows() {
	let over = false;
	const opened = new MutationObserver(() => look());
	const left = new MutationObserver(() => look());
	const look = () => {
		const now = topWindow() !== null;
		if (now === over) return;
		over = now;
		if (now) left.observe(document.body, {childList: true, subtree: true});
		else left.disconnect();
		heard(intent("pump.window", {open: now}));
	};
	opened.observe(document.documentElement, {subtree: true, attributes: true, attributeFilter: ["open"]});
	heard(intent("pump.window", {open: false}));
	look();
}

whenStarted(() => {
	subscribe("pump", followPump);
	subscribe("shell", followShell);
	watchWindows();
});
