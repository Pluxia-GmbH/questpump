// Devices (the Menu's page; docs/ui.md, "The Menu on the computer"): one card of the heart-rate strap and the walking
// pad, each a Row with its switch and its state's lines (the pad's belt and the day's walk while it is read). A switch
// asks Rust (set_strap, set_pad: macOS asks for Bluetooth the first time) and shows the answer the topic brings. From
// the `menu.devices` topic (helper/src-tauri/src/ui.rs `Devices`).

import {call, get} from "../../lib/bridge.js";
import {t} from "../../lib/copy.js";
import {register} from "../../lib/router.js";
import {h} from "../../kit/element.js";
import {page, watch} from "../page.js";

/** Each device's words for its states. */
const WORDS = {
	strap: {
		unnamed: "desk.menu.devices.strap.unnamed", searching: "desk.menu.devices.strap.searching",
		notFound: "desk.menu.devices.strap.notfound", connected: "desk.menu.devices.strap.connected",
		off: "desk.menu.devices.strap.off", refused: "desk.menu.devices.strap.refused",
		refusedWindows: "desk.menu.devices.strap.refused.windows", failed: "desk.menu.devices.strap.failed",
	},
	pad: {
		unnamed: "desk.menu.devices.pad.unnamed", searching: "desk.menu.devices.pad.searching",
		notFound: "desk.menu.devices.pad.notfound", connected: "desk.menu.devices.pad.connected",
		off: "desk.menu.devices.pad.off", refused: "desk.menu.devices.pad.refused",
		refusedWindows: "desk.menu.devices.pad.refused.windows", failed: "desk.menu.devices.pad.failed",
	},
};

function beltLine(name, belt) {
	switch (belt.state) {
		case "stopped": return t("desk.menu.devices.pad.stopped", {name});
		case "moving": return t("desk.menu.devices.pad.walking", {name, speed: belt.speed});
		default: return t("desk.menu.devices.pad.connected", {name});
	}
}

function walkLine(walk) {
	switch (walk.state) {
		case "no_distance": return t("desk.menu.devices.pad.nodistance");
		case "none": return t("desk.menu.devices.pad.walk.none");
		case "open": return t("desk.menu.devices.pad.walk.open", walk);
		case "kept": return t("desk.menu.devices.pad.walk.kept", walk);
		case "both": return t("desk.menu.devices.pad.walk.both", walk);
		case "game": return t("desk.menu.devices.pad.walk.game");
		case "kept_game": return t("desk.menu.devices.pad.walk.keptgame", walk);
		default: return null;
	}
}

/** A device's lines: its state's words, and the pad's walk while it is read. */
function lines(kind, line) {
	if (!line) return [];
	const keys = WORDS[kind];
	const windows = get("shell")?.platform === "windows";
	const name = line.name ?? t(keys.unnamed);
	switch (line.state) {
		case "starting": return [t("desk.menu.devices.starting")];
		case "searching": return [t(keys.searching)];
		case "not_found": return [t(keys.notFound)];
		case "connected": return [t(keys.connected, {name})];
		case "waiting": return [t(windows ? "desk.menu.devices.waiting.windows" : "desk.menu.devices.waiting")];
		case "off": return [t(keys.off)];
		case "refused": return [t(windows ? keys.refusedWindows : keys.refused)];
		case "failed": return [t(keys.failed, {error: line.error})];
		case "walking": return [beltLine(name, line.belt), walkLine(line.walk)].filter(Boolean);
		default: return [];
	}
}

register("desk.menu.devices", () => {
	const view = page({heading: t("desk.menu.devices.title")});
	const toggle = (heading, glyph, command) => {
		// the switch moves when the topic brings the device's state (Toggle's contract), never before
		const row = h("qp-row", {heading, glyph, trailing: "toggle"});
		row.addEventListener("qp-change", (e) => call(command, {on: e.detail}).catch(console.error));
		return row;
	};
	const strap = toggle(t("desk.menu.devices.strap"), "mark_heart", "set_strap");
	const pad = toggle(t("desk.menu.devices.pad"), "stat_agility", "set_pad");
	view.body.append(h("qp-card", {rows: true}, strap, pad));
	watch(view, "menu.devices", (d) => {
		strap.checked = d.strap.on;
		strap.lines = lines("strap", d.strap.line);
		pad.checked = d.pad.on;
		pad.lines = lines("pad", d.pad.line);
	});
	view.primary = () => strap;
	return view;
});
