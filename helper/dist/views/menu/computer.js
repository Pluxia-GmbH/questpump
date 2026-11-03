// This computer (the Menu's page; docs/ui.md, "The Menu on the computer"): this computer's state in the tray's words
// (`state_line`, which Pump home's Chip shows too), the step left in the game (the Phones page's `next`, `gameStep`);
// the steps a Mac or a Windows PC may need first (the move to Applications while QuestPump runs from its download, the
// Screen Recording grant while a device needs it, its one line naming what it reads, with the restart that applies it,
// the right to write into the game folder); the game (the running client by its
// install's label, as the folder line names it), the screen reading, your progress kept for the game, the QuestPump
// addon in the game (`addon`, runtime/addon.rs: its version and whether it is up to date, with Add QuestPump to the
// game while the game is here without it and Update now for a newer one QuestPump does not keep), the move figures in
// the game folder, the move figures in this app (desk/figures.rs: bundled, kept from their download or on their way)
// and the version;
// the game folder with Change…; Start with this computer where the system can say; and Size, the page's size with −
// and + (`sizeRow`). From the `menu.computer` topic (helper/src-tauri/src/ui.rs `Computer`), the `menu.phones` topic's
// `next` and the `shell` topic's platform and size.

import {call, get, intent} from "../../lib/bridge.js";
import {t} from "../../lib/copy.js";
import {time} from "../../lib/format.js";
import {register} from "../../lib/router.js";
import {h} from "../../kit/element.js";
import {page, watch, words} from "../page.js";

function windows() {
	return get("shell")?.platform === "windows";
}

function screenLine(screen) {
	switch (screen.state) {
		case "waiting": return t("desk.menu.computer.screen.waiting");
		case "reading": return t("desk.menu.computer.screen.reading");
		case "no_addon": return t("desk.menu.computer.screen.noaddon");
		case "out": return t(screen.hdr ? "desk.menu.computer.screen.out.hdr" : "desk.menu.computer.screen.out");
		case "unseen": return t("desk.menu.computer.screen.unseen");
		case "newer": return t("desk.menu.computer.screen.newer");
		case "older": return t("desk.menu.computer.screen.older");
		case "colours": return t(screen.hdr ? "desk.menu.computer.screen.colours.hdr" : "desk.menu.computer.screen.colours");
		case "stopped":
			return t(windows() ? "desk.menu.computer.screen.stopped.windows" : "desk.menu.computer.screen.stopped");
		case "grant": return t("desk.menu.computer.screen.grant");
		case "off": return t("desk.menu.computer.screen.off");
		case "failed": return t(windows() ? "desk.menu.computer.screen.failed.windows"
			: "desk.menu.computer.screen.failed", {error: screen.error});
		default: return t("desk.menu.computer.screen.notseen");
	}
}

function mirrorLine(mirror) {
	switch (mirror.state) {
		case "kept": return t("desk.menu.computer.mirror.kept", {time: time(mirror.at)});
		case "failed": return t("desk.menu.computer.mirror.failed", {error: mirror.error});
		case "unfound": return t("desk.menu.computer.mirror.unfound");
		case "here": return t("desk.menu.computer.mirror.here");
		default: return t("desk.menu.computer.mirror.waiting");
	}
}

function folderLine(folder) {
	switch (folder.state) {
		case "found": return folder.label ? t("desk.menu.computer.folder.labelled", folder) : folder.path;
		case "none": return t("desk.menu.computer.folder.none");
		default: return t("desk.menu.computer.folder.unknown");
	}
}

/** The game line: the running client by its install's label (This computer's and the welcome's), else not running. */
export function gameLine(game) {
	if (!game) return t("desk.menu.computer.game.none");
	return game.label ? t("desk.menu.computer.game.running", {label: game.label})
		: t("desk.menu.computer.game.running.plain");
}

/** The step left in the game in words (the `menu.phones` topic's `next`, Rust's `link::Step`), as This computer, the
 * welcome and the Phones page say it; null when none is left, or for the Screen Recording grant, which This computer
 * asks in its own card. */
export function gameStep(next) {
	switch (next) {
		case "check_again": return t("desk.menu.phones.next.check");
		case "reload": return t("desk.menu.phones.next.reload");
		case "either": return t("desk.menu.phones.next.either");
		default: return null;
	}
}

/** The addon's line (the `menu.computer` topic's `addon.line`, Rust's `ui::AddonLine`) as This computer, the welcome
 * and Pump home's card say it; null when there is nothing to say. */
export function addonLine(line) {
	switch (line?.state) {
		case "missing": return t("desk.addon.missing");
		case "adding": return t("desk.addon.adding");
		case "add_failed": return t("desk.addon.addfailed", {error: line.error});
		case "added": return t("desk.addon.added");
		case "version": return line.version;
		case "current": return t("desk.addon.current", {version: line.version});
		case "updating": return t("desk.addon.updating");
		case "ready": return t("desk.addon.ready", {version: line.version, new: line.new});
		case "newer": return t("desk.addon.newer", {version: line.version, new: line.new});
		case "failed": return t("desk.addon.failed", {error: line.error});
		default: return null;
	}
}

/** Why a folder the player picked was not taken (This computer's and the welcome's line under the folder). */
export function folderNote(note) {
	if (!note) return "";
	const key = {running: "desk.menu.computer.folder.running", no_quest_pump: "desk.menu.computer.folder.noquestpump"};
	return t(key[note.state] ?? "desk.menu.computer.folder.noaddons", {path: note.path});
}

function packLine(pack) {
	switch (pack.state) {
		case "installed": return t("desk.menu.computer.pack.installed");
		case "missing": return t("desk.menu.computer.pack.missing");
		case "other": return t("desk.menu.computer.pack.other", {pack: pack.pack, addon: pack.addon});
		default: return t("desk.menu.computer.pack.notseen");
	}
}

function animationsLine(animations) {
	switch (animations.state) {
		case "bundled": return t("desk.figures.bundled");
		case "kept": return t("desk.figures.kept");
		case "downloading": return t("desk.figures.downloading", {percent: animations.percent});
		case "failed": return t("desk.figures.failed", {error: animations.error});
		case "missing": return t("desk.figures.missing");
		default: return t("desk.menu.computer.pack.notseen");
	}
}

function versionLine(version) {
	const v = {version: version.number};
	const update = version.update;
	if (update) {
		switch (update.state) {
			case "ready": return t("desk.menu.computer.version.ready", {...v, new: update.new});
			case "current": return t("desk.menu.computer.version.current", v);
			case "failed": return t("desk.menu.computer.version.failed", v);
			case "unreplaceable": return t("desk.menu.computer.version.unreplaceable", {...v, new: update.new});
			case "installing": return t("desk.menu.computer.version.installing", {...v, new: update.new});
			default: return t("desk.menu.computer.version.notinstalled", v);
		}
	}
	if (version.build === "unmoved") return t("desk.menu.computer.version.unmoved", v);
	if (version.build === "development") return t("desk.menu.computer.version.development", v);
	return version.number;
}

/** How a one-time administrator step went (Windows); `failed` is its card's key for a failure. */
export function adminNote(note, failed) {
	if (!note) return "";
	switch (note.state) {
		case "waiting": return t("desk.menu.computer.admin.waiting");
		case "declined": return t("desk.menu.computer.admin.declined");
		case "another": return t("desk.menu.phones.firewall.another");
		default: return t(failed, {error: note.error});
	}
}

/** Size (2026-10-15, for Windows above all, where the keys were the only way): the page's size as a percentage of its
 * default (100 %, the page at 125 %: desk/windows.rs `BASE`) between − and + (the kit's Stepper, compact), each step
 * through Rust's `view.zoom` as the keys and the macOS View menu go, and drawn from the `shell` topic's `zoom` and
 * `zooms`, so a key or the menu moves it too. Returns the row and its drawing from `shell`. */
function sizeRow() {
	const heading = t("desk.menu.computer.size");
	const stepper = h("qp-stepper", {compact: true, label: heading, "less-label": t("desk.menu.computer.size.smaller"),
		"more-label": t("desk.menu.computer.size.larger")});
	let at = null; // the percentage stepped from: the topic's, or the Stepper's own last step before the topic's answer
	stepper.addEventListener("qp-change", (e) => {
		// its words and its spoken value at once, with the value: the topic's answer only confirms them
		stepper.text = t("desk.menu.computer.size.value", {percent: e.detail});
		if (at !== null) intent("view.zoom", {step: Math.sign(e.detail - at)}).catch(console.error);
		at = e.detail;
	});
	const el = h("div", {class: "qp-stepper-line"},
		h("span", {class: "qp-row-words"}, h("span", {class: "qp-row-title", text: heading})), stepper);
	const draw = (shell) => {
		at = Math.round(shell.zoom * 100);
		stepper.steps = shell.zooms.map((z) => Math.round(z * 100));
		stepper.text = t("desk.menu.computer.size.value", {percent: at});
		stepper.value = at;
	};
	return {el, draw};
}

/** A step's card: its heading, its words and its controls. */
export function stepCard(heading, ...parts) {
	return h("qp-card", {class: "qp-step"}, h("h2", {class: "qp-type-heading", text: heading}), ...parts);
}

register("desk.menu.computer", () => {
	const view = page({heading: t("desk.menu.computer.title")});
	const state = h("p", {class: "qp-type-body-strong", role: "status"});
	const next = h("p", {class: "qp-type-body-strong qp-next", role: "status"});

	const move = stepCard(t("desk.menu.computer.move.title"), words(t("desk.menu.computer.move.line"), "body"),
		h("qp-secondary-button", {label: t("desk.menu.computer.move.open"),
			"on:click": () => call("open_applications").catch(console.error)}));

	const allow = h("qp-secondary-button", {"on:click": () => call("allow_screen").catch(console.error)});
	const restart = h("qp-secondary-button", {label: t("desk.menu.computer.screen.restart")});
	let failedRestarts = 0; // the topic counts each restart that failed: a new count puts the button's words back
	restart.addEventListener("click", () => {
		restart.label = t("desk.menu.computer.screen.restarting");
		call("restart").catch(console.error);
	});
	const restartNote = words(t("desk.menu.computer.screen.restart.failed"), "meta");
	const restartBlock = h("div", {class: "qp-stack-small"}, restart, restartNote);
	const screen = stepCard(t("desk.menu.computer.screen.title"), words(t("desk.menu.computer.screen.how"), "body"),
		allow, restartBlock);

	const accessAllow = h("qp-secondary-button", {label: t("desk.menu.computer.access.allow"),
		"on:click": () => call("allow_folder").catch(console.error)});
	const accessNote = words("", "meta");
	const acl = h("div", {class: "qp-stack-small"}, words(t("desk.menu.computer.access.acl"), "body"), accessAllow,
		accessNote);
	const cfa = words("", "body");
	const access = stepCard(t("desk.menu.computer.access.title"), acl, cfa);

	const status = (heading) => h("qp-row", {heading, trailing: "shown"});
	const game = status(t("desk.menu.computer.game"));
	const reading = status(t("desk.menu.computer.screen"));
	const mirror = status(t("desk.menu.computer.mirror"));
	const pack = status(t("desk.menu.computer.pack"));
	// Add QuestPump to the game or Update now while the topic offers one, else its line alone
	const addon = h("qp-row", {heading: t("desk.addon.row"), trailing: "shown"});
	addon.addEventListener("click", () => {
		if (addon.trailing === "action") call("addon_install").catch(console.error);
	});
	const animations = status(t("desk.figures.row"));
	const version = status(t("desk.menu.computer.version"));
	const folder = h("qp-row", {heading: t("desk.menu.computer.folder"), trailing: "action",
		value: t("desk.menu.computer.folder.change"), "on:click": () => call("pick_folder").catch(console.error)});
	const note = words("", "meta");
	// the switch moves when the topic says the system took the change (Toggle's contract), never before
	const autostart = h("qp-row", {heading: t("desk.menu.computer.autostart"), trailing: "toggle",
		"on:qp-change": (e) => call("set_autostart", {on: e.detail}).catch(console.error)});
	const autostartCard = h("qp-card", {rows: true}, autostart);
	const size = sizeRow();

	view.body.append(state, next, move, screen, access,
		h("qp-card", {rows: true}, game, reading, mirror, addon, pack, animations, version),
		h("qp-card", {rows: true}, folder), note, autostartCard, h("qp-card", {rows: true}, size.el));

	watch(view, "menu.phones", (p) => {
		const step = gameStep(p.next);
		next.textContent = step ?? "";
		next.hidden = !step;
	});
	watch(view, "shell", (shell) => size.draw(shell));
	watch(view, "menu.computer", (c) => {
		state.textContent = c.state_line;
		move.hidden = !c.unmoved;
		screen.hidden = !c.permission;
		allow.label = t(c.asked ? "desk.menu.computer.screen.settings" : "desk.menu.computer.screen.allow");
		restartBlock.hidden = !c.asked;
		restartNote.hidden = !c.restart_failed;
		if (c.restart_failed !== failedRestarts) {
			failedRestarts = c.restart_failed;
			restart.label = t("desk.menu.computer.screen.restart");
		}
		access.hidden = c.access === "fine";
		acl.hidden = c.access !== "acl";
		cfa.hidden = c.access !== "cfa";
		cfa.textContent = t("desk.menu.computer.access.cfa", {program: c.program});
		accessNote.textContent = adminNote(c.access_note, "desk.menu.computer.access.failed");
		accessNote.hidden = !c.access_note;
		game.meta = gameLine(c.game);
		const line = screenLine(c.screen);
		reading.meta = c.ping_at ? t("desk.menu.computer.screen.ping", {line, time: time(c.ping_at)}) : line;
		mirror.meta = mirrorLine(c.mirror);
		folder.meta = folderLine(c.folder);
		const offer = c.addon.offer;
		addon.hidden = c.addon.line.state === "none";
		addon.meta = addonLine(c.addon.line);
		addon.trailing = offer ? "action" : "shown";
		addon.value = offer ? t(offer === "add" ? "desk.addon.add" : "desk.addon.update") : null;
		pack.hidden = c.pack.state === "absent";
		pack.meta = packLine(c.pack);
		animations.meta = animationsLine(c.animations);
		version.meta = versionLine(c.version);
		note.textContent = folderNote(c.folder_note);
		note.hidden = !c.folder_note;
		autostartCard.hidden = c.autostart === null;
		autostart.checked = c.autostart === true;
	});
	view.primary = () => (screen.hidden ? null : allow);
	return view;
});
