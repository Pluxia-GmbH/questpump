// The page's part of the sound and voice (docs/ui.md, "The pump on the computer": sound and voice carry the moment):
// Rust plays every sound (helper/src-tauri/src/desk/audio.rs), and the page tells it the kit's feedback events where the
// phones' kit fires them (iOS Kit/Buttons.swift, Card.swift, ActionBar.swift, Window.swift, Quality.swift,
// Progress.swift, TextInput.swift, Figure.swift, Toast.swift): a tap on a button, a row that acts, a chip, a tile, a
// stat, a text box's clear and the figure's mark (but the × of a window, whose close sounds), a new tab of the ActionBar
// or a Segmented, a switch flipped, a window opening, a window the player closes by its × or Escape (qp-window's
// qp-dismiss; a window its owner closes, a row's action or a stop, closes silently, as on the phones), and a Toast as
// it shows (its own event, qp-toast). A kit element whose press is a moment of its own names its event in place of the
// tap (`data-sound`: a place picked plays `toggle`, slice 72), and a phone paired while the page stands plays `commit`.
// Each goes as `sound.event`, never while the Sounds switch is off. And while a
// screen reader runs and QuestPump's window has the focus, a pump's line comes as the `shell` topic's `announce`, which
// the page's one polite live region says (lib/focus.js) instead of the voice.

import {get, intent, subscribe} from "./bridge.js";
import {announce} from "./focus.js";

/** The kit elements whose buttons tap, and the Saga's rows that act as a row does (an earlier chapter's row, Pump
 * home's Today's chapter). */
const TAPS = "qp-primary-button, qp-secondary-button, qp-quiet-button, qp-round-button, qp-choice-chip, qp-row, "
	+ "qp-move-tile, qp-stat-strip, qp-text-input, qp-figure-stage, .qp-chapter-row, .qp-chapter-card";

/** The Sounds switch, as the Sound and voice page's topic last said. */
let soundsOn = true;

/** Sends a feedback event of contract/design/tokens.json while the sounds are on: the kit's own below, and a view's own
 * moment (the Saga's reveal, the best day's milestone, the talent trees' pinned card opening and closing, which is no
 * window). */
export function feedback(event) {
	if (soundsOn) intent("sound.event", {event}).catch(console.error);
}

function clicked(e) {
	const button = e.target.closest?.("button");
	if (!button || button.disabled) return;
	if (button.matches(".qp-tab")) {
		if (!button.hasAttribute("aria-current")) feedback("tab");
		return;
	}
	const kit = button.closest(TAPS);
	if (!kit) return;
	const kind = kit.localName;
	// a switch sounds as it flips (qp-change), the window's × as the window closes, a text box only at its clear
	if (kind === "qp-row" && kit.getAttribute("trailing") === "toggle") return;
	if (kind === "qp-round-button" && kit.getAttribute("icon") === "close") return;
	if (kind === "qp-text-input" && !button.matches(".qp-clear")) return;
	feedback(kit.dataset.sound ?? "tap");
}

function changed(e) {
	const kind = e.target.localName;
	if (kind === "qp-segmented") feedback("tab");
	else if (kind === "qp-toggle" || (kind === "qp-row" && e.target.getAttribute("trailing") === "toggle")) {
		feedback("toggle");
	}
}

/** Starts the kit's sounds and the live region's lines (after the bridge has its topics). */
export function startSound() {
	document.addEventListener("click", clicked, true);
	document.addEventListener("qp-change", changed);
	document.addEventListener("qp-dismiss", () => feedback("close"));
	document.addEventListener("qp-toast", (e) => feedback(e.detail));
	new MutationObserver((records) => {
		for (const r of records) {
			if (r.target.matches("dialog.qp-window") && r.target.open && r.oldValue === null) feedback("open");
		}
	}).observe(document.body, {subtree: true, attributes: true, attributeFilter: ["open"], attributeOldValue: true});
	subscribe("menu.sound", (page) => {
		soundsOn = page.sound;
	});
	// a phone paired (slice 72: a commitment that succeeded), once as the pairing comes, never for one that stood as
	// the page started
	let paired = null;
	subscribe("menu.phones", (phones) => {
		const now = Boolean(phones?.phone?.name);
		if (paired === false && now) feedback("commit");
		paired = now;
	});
	let heard = get("shell")?.announce?.seq ?? 0;
	subscribe("shell", (shell) => {
		const line = shell.announce;
		if (line && line.seq > heard) {
			heard = line.seq;
			announce(line.text);
		}
	});
}
