// The first five minutes on the computer (docs/ui.md, "The first five minutes"; the phones' first start): a new player,
// no account and no sign-up. Rust decides from what QuestPump found on the computer (its stores, the game, the paired
// phone; helper/crates/questpump-desk/src/menu_pages/welcome.rs): the `welcome` topic names a step only for a new player and
// only once the stores are read, and a returning player's first open ends the flow with `toast`, where the progress
// came from, which the kit's Toast says once over the window as it stands (never over a pump), with where QuestPump
// waits, then `welcome.toasted`. While a step stands, the welcome covers the window as the phones' first start replaces
// their root, one step at a time, each kept by Rust: the welcome (the figure, the wordmark, the card On this computer
// with the game and the phone as QuestPump found them, in This computer's and the Phones page's words, a phone not
// paired yet pairing through the Phones page's own code in a window over the welcome, and the gold Start), the
// experience question, where the player pumps (the line that a gate holds the first pump back, and Continue, while one
// does), the first Quick pump at once (DESK-PUMP's own cover, then its Result; `pump`, its own step, shows the place
// again until a pump came and went), Hero lighting the muscles it trained, and done: where QuestPump waits (the games
// it watches are the Games tab's, slice 56, never a step). A pump's cover goes over the welcome and
// the welcome comes back after it; while QuestPump still runs from its download (macOS), This computer's move to
// Applications comes first. The page also holds the Toast while a pump stands (the phones' holdsToasts), and asks where
// the player pumps in a window before the first own pump on a computer that never asked it (the `pump` topic's
// `place`).

import {call, get, intent, subscribe} from "../../lib/bridge.js";
import {t} from "../../lib/copy.js";
import {focusView} from "../../lib/focus.js";
import {time} from "../../lib/format.js";
import {mounted} from "../../lib/keys.js";
import {cover, over, register, uncover} from "../../lib/router.js";
import {h} from "../../kit/element.js";
import {addonLine, folderNote, gameLine, gameStep} from "../menu/computer.js";
import {pairWindow} from "../menu/phones.js";
import {words} from "../page.js";
import {openWindow} from "../pump/windows.js";

/** The figure the welcome shows (the phones' WelcomePage's): shadow boxing's guard stance, its loop at the calm pace
 * (the figure element's `calm`). */
const FIGURE = "shadow_boxing";
/** How long the first pump may take to show before Start my first pump stops waiting for it and offers itself again. */
const PUMP_WAIT_MS = 3000;

let host = null;
let toaster = null;
let topic = null;
// the found progress was said in this page (Rust forgets it once `welcome.toasted` reaches it)
let toasted = false;
let platform = "macos";
let engine = "busy";
let unmoved = false;
// Start my first pump was pressed: busy until its pump shows, or the wait is over
let asked = false;
let pumpWait = null;
// at the pump step, a pump's cover was seen; once it is gone, `welcome.pumped`, and the step stays busy until Hero
let pumpSeen = false;
let pumped = false;
// the `pump` topic as it stands: its gate, its place window, its pump
let pumpTopic = null;
// the welcome's view in the cover now, whose step redraws when the pump or the engine moves
let shown = null;
// the close of the pairing window the welcome opened
let pairing = null;
let place = null;
let placeHeld = false;
let toastTimer = null;

/** The welcome's own view stands in the cover. */
function ours() {
	return Boolean(host.cover.querySelector(":scope > [data-welcome]"));
}

/** Another view (a pump) stands in the cover. */
function foreign() {
	return !host.cover.hidden && !ours();
}

/** The welcome over the window while a step stands and nothing else covers it; gone once the flow is done, and then
 * the progress found said. At the pump step, a pump that came and went moves the flow on to Hero. */
function sync() {
	if (!host) return;
	const step = topic?.step;
	if (step !== "pump") pumpSeen = pumped = false;
	if (step === "pump" && foreign()) pumpSeen = true;
	if (step === "pump" && pumpSeen && host.cover.hidden) {
		pumpSeen = false;
		pumped = true;
		intent("welcome.pumped").catch((e) => {
			console.error(e);
			pumped = false;
			refresh();
		});
	}
	if (!step) {
		if (ours()) uncover();
		sayFound();
		return;
	}
	if (unmoved || foreign()) return;
	if (!ours()) cover("desk.welcome");
}

/** The first pump is on its way: DESK-PUMP holds its start for the place window, or its pump stands. */
function onItsWay() {
	return Boolean(pumpTopic?.place || pumpTopic?.pump);
}

/** The progress QuestPump found at the first open, said once by the Toast over the window as it stands (Pump home, or
 * This computer while the move to Applications comes first), never over a pump or its place window, with where
 * QuestPump waits and, while no phone is paired, Pump home's card that pairs one. */
function sayFound() {
	if (!topic?.toast || topic.step || toasted || foreign() || onItsWay()) return;
	toasted = true;
	toaster.show({kind: "found", from: topic.toast, platform, pair: topic.phones && !topic.found.phone});
	intent("welcome.toasted").catch(console.error);
}

/** A pump holds the toasts (the phones' PumpRunner.holdsToasts): one on screen, but a Result that celebrates once its
 * reveal is over; the hold ends by itself at the reveal's end. */
function holdToasts() {
	clearTimeout(toastTimer);
	const pump = pumpTopic?.pump;
	const reveal = pump?.reveal;
	const celebrates = Boolean(pump?.result) && !pump.calm;
	const revealing = Boolean(reveal) && !reveal.landed && Date.now() < reveal.end;
	toaster.paused = Boolean(pump) && (pump.stage !== "done" || !celebrates || revealing);
	if (pump?.stage === "done" && celebrates && revealing) toastTimer = setTimeout(holdToasts, reveal.end - Date.now());
}

/** The step on screen draws again (the gate, the engine, the first pump on its way). */
function refresh() {
	shown?.refresh();
}

/** Waits for the first pump only while nothing says it is on its way: a refused or vanished start offers itself again
 * once the wait is over. */
function waitForPump() {
	clearTimeout(pumpWait);
	pumpWait = null;
	if (asked && !onItsWay() && !foreign()) pumpWait = setTimeout(pumpDone, PUMP_WAIT_MS);
}

function pumpDone() {
	clearTimeout(pumpWait);
	pumpWait = null;
	asked = false;
	refresh();
}

/** The cover changed: the first pump's came, or went. */
function onCover() {
	if (asked && foreign()) pumpDone();
	sync();
}

/** The first Quick pump at once (`welcome.place` with `pump`: Rust keeps the place, moves the flow to its pump step and
 * asks DESK-PUMP's Quick pump); a pump refused shows the place step again with why. */
function firstPump(answer) {
	asked = true;
	refresh();
	intent("welcome.place", {...answer, pump: true}).then(waitForPump, (e) => {
		console.error(e);
		pumpDone();
	});
}

/** The pairing code over the welcome (the Phones page's own, `pairWindow`), the flow standing at its step; it closes
 * once a phone pairs, and the phone's rows coming in make the player a returning one. */
function pairPhone() {
	pairing = pairWindow();
}

/** Where the player pumps, as the place step and the place window show it: Beside the desk or Room to lie down (Rust
 * sets Your setup's Floor moves and Room to move from `floor`), Quiet mode in Your setup's word (the sheet's, through
 * the topic), none without its word, and under it the Sound and voice page's "Count reps aloud" (2026-10-09, David:
 * "also put into onboarding"), which asks Rust at once (`sound.count`) and moves with
 * the `menu.sound` topic, as on that page; `answer()` the choice, `stop()` the switch's watch. */
function placeRows(w) {
	let floor = w.place.floor;
	let quiet = w.place.quiet;
	const desk = h("qp-row", {heading: t("desk.place.desk"), glyph: "device_computer", trailing: "checked"});
	const room = h("qp-row", {heading: t("desk.place.floor"), glyph: "mark_dash", trailing: "checked"});
	const word = w.words?.switches?.quiet?.name;
	const hush = word ? h("qp-row", {heading: word, glyph: "reward_familiarity", trailing: "toggle"}) : null;
	// shown once the Sound and voice page's topic stands, whose area answers `sound.count`
	const count = h("qp-row", {heading: t("menu.repcount"), glyph: "mark_clock", trailing: "toggle", hidden: true});
	count.addEventListener("qp-change", (e) => intent("sound.count", {on: e.detail}).catch(console.error));
	const watching = subscribe("menu.sound", (s) => {
		count.hidden = false;
		count.checked = s.count;
	});
	const draw = () => {
		desk.checked = !floor;
		room.checked = floor;
		if (hush) hush.checked = quiet;
	};
	desk.addEventListener("click", () => {
		floor = false;
		draw();
	});
	room.addEventListener("click", () => {
		floor = true;
		draw();
	});
	hush?.addEventListener("qp-change", (e) => {
		quiet = e.detail;
		draw();
	});
	draw();
	const cards = [h("qp-card", {rows: true}, desk, room), h("qp-card", {rows: true}, ...[hush, count].filter(Boolean))];
	return {cards, answer: () => ({floor, quiet}), stop: watching};
}

/** Where the player pumps, asked once before the first own pump on a computer that never asked it (the `pump`
 * topic's `place`): only its Continue closes it, and the pump that waited goes on. */
function followPlace(p) {
	if (!p?.place) {
		placeHeld = false;
		place?.close();
		return;
	}
	if (placeHeld || !topic) return;
	placeHeld = true;
	place = openWindow({heading: t("desk.place.title"), locked: true, primary: t("setup.experience.continue")},
		(content, win) => {
			const rows = placeRows(topic);
			content.append(...rows.cards);
			win.addEventListener("qp-primary", () => {
				intent("welcome.where", rows.answer()).catch(console.error);
				win.close();
			});
			win.addEventListener("qp-closed", rows.stop, {once: true});
		}, () => {
			place = null;
		});
}

/** Starts the welcome's watch over the frame: after the router, with the root host it covers and the Toast that says
 * the progress found. The `welcome` topic comes last, so its first look already knows whether QuestPump runs from its
 * download and whether a pump or its place window stands. */
export function startWelcome(rootHost, toast) {
	host = rootHost;
	toaster = toast;
	subscribe("shell", (shell) => {
		platform = shell.platform;
		engine = shell.engine;
		refresh();
	});
	subscribe("menu.computer", (computer) => {
		unmoved = computer.unmoved;
		sync();
	});
	subscribe("pump", (pump) => {
		pumpTopic = pump;
		holdToasts();
		followPlace(pump);
		waitForPump();
		refresh();
		sayFound();
	});
	subscribe("welcome", (w) => {
		topic = w;
		if (!w.step && pairing) {
			pairing();
			pairing = null;
		}
		// the place window a page that started with the pump topic's `place` already asked waited for this topic's words
		followPlace(pumpTopic);
		sync();
	});
	new MutationObserver(onCover).observe(host.cover, {attributes: true, attributeFilter: ["hidden"], childList: true});
	sync();
}

/** A step's back bar: the DetailBar's back goes one step back, never through the router. */
function backBar() {
	return h("qp-detail-bar");
}

/** A kit button of `tag` with its words and what a press does. */
function button(tag, label, then) {
	return h(tag, {label, "on:click": then});
}

/** A press that sends the intent `type`. */
function ask(type, args) {
	return () => intent(type, args).catch(console.error);
}

/** The game and the phone as QuestPump found them (the topic's `found`), in This computer's, the tray's and the Phones
 * page's words (their topics; the game's with the step left in the game, the Phones page's `next`: Check again,
 * /reload or either, as the strip's setup word says, and the addon's line, This computer's `addon`): a check where it
 * found one, else the way to it: Add QuestPump to the game for a game without QuestPump (QuestPump puts the addon in
 * its folder; while the game runs, the line after says to start it again), the game's folder through This computer's
 * picker while none is found (its take-in may find the player's progress, and its note shows under the card), a phone
 * through the Phones page's pairing code over the welcome; no phone row while the phone apps are not in the stores yet
 * (the topic's `phones`). */
function foundCard() {
	const game = h("qp-row", {heading: t("desk.menu.computer.game"), glyph: "action_games"});
	const phone = h("qp-row", {heading: t("desk.menu.phones.phone"), glyph: "device_phone"});
	const note = words("", "meta");
	game.addEventListener("click", () => {
		if (game.trailing !== "action") return;
		if (found?.game === "installed") call("addon_install").catch(console.error);
		else call("pick_folder").catch(console.error);
	});
	phone.addEventListener("click", () => {
		if (phone.trailing === "action") pairPhone();
	});
	let found = null;
	let offered = true;
	let computer = null;
	let phones = null;
	const draw = () => {
		if (!found) return;
		const known = found.game === "running" || found.game === "found";
		const installed = found.game === "installed";
		const addon = computer?.addon;
		// no action while QuestPump adds the addon
		const busy = installed && !addon?.offer && addon?.line?.state === "adding";
		game.glyph = known ? "mark_check" : "link_game";
		game.trailing = known || busy ? "shown" : "action";
		game.value = known || busy ? null : t(installed ? "desk.addon.add" : "desk.welcome.computer.choose");
		const running = found.game === "running" && computer?.game ? gameLine(computer.game) : null;
		// the step left in the game, as This computer names it, and the restart a game running since the add needs
		const step = gameStep(phones?.next);
		const added = addon?.line?.state === "added" ? addonLine(addon.line) : null;
		game.lines = known ? [running, step, added]
			: [installed ? addonLine(addon?.line) ?? t("desk.addon.missing") : t("desk.tray.state.none")];
		phone.hidden = !offered;
		const paired = Boolean(found.phone);
		phone.glyph = paired ? "mark_check" : "device_phone";
		phone.trailing = paired ? "shown" : "action";
		phone.value = paired ? null : t("desk.menu.phones.pair");
		phone.meta = phones?.phone_line ?? null;
		note.textContent = folderNote(computer?.folder_note);
		note.hidden = !computer?.folder_note;
	};
	const stops = [
		subscribe("menu.computer", (c) => {
			computer = c;
			draw();
		}),
		subscribe("menu.phones", (p) => {
			phones = p;
			draw();
		}),
	];
	return {
		parts: [h("qp-section-label", {label: t("desk.welcome.computer")}), h("qp-card", {rows: true}, game, phone), note],
		update: (w) => {
			found = w.found;
			offered = w.phones;
			draw();
		},
		stop: () => stops.forEach((stop) => stop()),
	};
}

/** In the browser (its `play` topic; docs/plan.md slice 63, "The first visit"): the welcome's I played before, which
 * opens the Email page's sign-in over the welcome (Back returns to its steps; a join ends the first five minutes on
 * Keep your key, web/views/index.js); How hard's safety note with the maker and the contact (the `menu.about`
 * topic's); and no Done step, the first visit ending on Pump home. The desktop app publishes no `play` topic. */
function browser() {
	return get("play") ?? null;
}

/** The welcome: the figure, the wordmark, what QuestPump found on this computer (in the browser I played before
 * instead), and Start. */
function welcomeStep() {
	const start = button("qp-primary-button", t("setup.welcome.start"), ask("welcome.start"));
	if (browser()) {
		const played = button("qp-quiet-button", t("setup.welcome.played"), () => over("desk.menu.email"));
		return {primary: start, parts: [
			h("qp-figure-stage", {class: "qp-welcome-figure", move: FIGURE, calm: true}),
			h("div", {class: "qp-welcome-words"}, h("h1", {class: "qp-type-wordmark", text: t("app.name")})),
			h("div", {class: "qp-foot"}, start, played),
		]};
	}
	const here = foundCard();
	return {primary: start, update: here.update, stop: here.stop, parts: [
		h("qp-figure-stage", {class: "qp-welcome-figure", move: FIGURE, calm: true}),
		h("div", {class: "qp-welcome-words"}, h("h1", {class: "qp-type-wordmark", text: t("app.name")})),
		...here.parts,
		h("div", {class: "qp-foot"}, start),
	]};
}

/** The experience question: the three answers as Rows, the one in the body checked, Continue sets it. */
function experienceStep(w) {
	let chosen = w.experience;
	const rows = w.words.experience.answers.map((a) => {
		const row = h("qp-row", {heading: a.name, glyph: "stat_strength", trailing: "checked",
			checked: a.key === chosen});
		row.addEventListener("click", () => {
			chosen = a.key;
			for (const [i, r] of rows.entries()) r.checked = w.words.experience.answers[i].key === chosen;
		});
		return row;
	});
	const go = button("qp-primary-button", t("setup.experience.continue"), () => intent("welcome.experience",
		{answer: chosen}).catch(console.error));
	const about = browser() ? get("menu.about") : null;
	const safety = about ? h("qp-card", {class: "qp-stack-small"}, words(about.safety, "body"),
		words(t("menu.about.publisher"), "meta"),
		words(t("menu.about.contact", {address: about.contact}), "meta")) : null;
	return {primary: go, bar: backBar(), parts: [
		h("qp-figure-stage", {class: "qp-welcome-figure qp-small", move: FIGURE, calm: true}),
		h("div", {class: "qp-welcome-words"}, h("h2", {class: "qp-type-heading", text: w.words.experience.title})),
		h("qp-card", {rows: true}, ...rows),
		...[safety].filter(Boolean),
		h("div", {class: "qp-foot"}, go),
	]};
}

/** What the place step offers now: busy while the first pump is on its way, has just left for Hero or the engine folds;
 * while a gate holds the pump back (the game with QuestPump runs, felt unwell rests the pumps, the engine failed), the
 * line that says so and Continue, on to Done; else Start my first pump. */
function placeOffer() {
	const gate = pumpTopic?.gate ?? null;
	if (asked || pumped || onItsWay()) return {state: "busy"};
	if (gate === "game") return {line: t("pump.waiting.game")};
	if (gate === "lock") return {line: t("pump.rest.line", {time: time(pumpTopic.locked_until)})};
	if (engine === "failed") return {line: t("desk.home.failed")};
	if (gate === "engine" || engine === "busy") return {state: "busy"};
	return {state: "normal"};
}

/** Where the player pumps (the place step, and the pump step until a pump came and went): the rows, then Start my
 * first pump, or a gate's line and Continue. */
function placeStep(w) {
	const rows = placeRows(w);
	const line = words("", "subtitle");
	const go = h("qp-primary-button", {"on:click": () => {
		const offer = placeOffer();
		if (offer.line) ask("welcome.place", {...rows.answer(), pump: false})();
		else if (offer.state === "normal") firstPump(rows.answer());
	}});
	const draw = () => {
		const offer = placeOffer();
		go.label = t(offer.line ? "setup.experience.continue" : "desk.place.go");
		go.state = offer.state ?? "normal";
		line.textContent = offer.line ?? "";
		line.hidden = !offer.line;
	};
	draw();
	return {primary: go, bar: backBar(), refresh: draw, stop: rows.stop, parts: [
		h("div", {class: "qp-welcome-words"}, h("h2", {class: "qp-type-heading", text: t("desk.place.title")})),
		...rows.cards,
		h("div", {class: "qp-foot"}, line, go),
	]};
}

/** Hero lighting the muscles the first pump trained (asked of the engine as the step shows), then on. */
function heroStep() {
	const go = button("qp-primary-button", t("setup.experience.continue"), ask("welcome.next"));
	const map = h("qp-muscle-map", {class: "qp-welcome-map", label: t("desk.welcome.hero.title")});
	ask("welcome.look")();
	const update = (w) => {
		map.tones = w.trained?.tones ?? {};
		map.names = w.trained?.names ?? {};
	};
	return {primary: go, update, parts: [
		h("div", {class: "qp-welcome-words"}, h("h2", {class: "qp-type-heading", text: t("desk.welcome.hero.title")})),
		map, h("div", {class: "qp-foot"}, go),
	]};
}

/** Done: where QuestPump waits, Done, and a quiet way to pair a phone while none is paired and the phone apps are in
 * the stores (the Phones page's own words and code, over this page). */
function doneStep() {
	const done = button("qp-primary-button", t("common.done"), ask("welcome.done"));
	if (browser()) {
		ask("welcome.done")();
		return {primary: done, parts: [h("div", {class: "qp-foot"}, done)]};
	}
	const pair = button("qp-quiet-button", t("desk.menu.phones.pair"), pairPhone);
	return {primary: done, update: (w) => {
		pair.hidden = Boolean(w.found.phone) || !w.phones;
	}, parts: [
		h("div", {class: "qp-welcome-words"}, h("h2", {class: "qp-type-heading", text: t("desk.done.title")}),
			words(t(platform === "windows" ? "desk.done.tray" : "desk.done.menubar"), "subtitle")),
		h("div", {class: "qp-foot"}, done, pair),
	]};
}

const STEPS = {welcome: welcomeStep, experience: experienceStep, place: placeStep, pump: placeStep, hero: heroStep,
	done: doneStep};

register("desk.welcome", () => {
	const el = h("qp-page", {light: "welcome", "data-welcome": ""});
	const view = {el, stops: [], refresh: () => drawn?.refresh?.()};
	let step = null;
	let drawn = null;
	shown = view;
	// a back bar's back goes one step back here, and the router never sees it
	el.addEventListener("qp-back", (e) => {
		e.stopPropagation();
		ask("welcome.back")();
	});
	view.stops.push(() => drawn?.stop?.());
	view.stops.push(() => {
		if (shown === view) shown = null;
	});
	view.stops.push(subscribe("welcome", (w) => {
		if (!w.step) return;
		if (w.step !== step) {
			drawn?.stop?.();
			step = w.step;
			drawn = STEPS[step](w);
			const column = h("div", {class: "qp-column qp-welcome"}, ...drawn.parts);
			el.replaceChildren(...[drawn.bar, column].filter(Boolean));
			view.primary = () => drawn.primary;
			mounted(view);
			focusView(view);
		}
		// what QuestPump found into the welcome and done, the Hero step's muscles into the step that asked for them
		drawn.update?.(w);
	}));
	return view;
});
