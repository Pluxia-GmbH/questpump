// The focus window (slice 30; iOS FocusWindow, Android's twin; the game's Sheet.lua focus window, docs/ui.md "The focus
// window"), one module wherever it opens: Hero's focus Chip, Your setup's Focus row, and the QP 10 ask, once a launch,
// over whatever page shows (the `hero` topic's `ask_focus`, which Rust raises as the phones' HomeView.askFocus does).
// "Your focus" (the ask's "QP {level}: pick your focus"), then the focuses in contract/sheet.json's order, each led by
// its glyph with its name, Balanced as "Balanced (recommended)", the one in force checked where the player opened it
// and none in the ask, whose primary, Keep Balanced, chooses Balanced so the ask never comes back. A click chooses the
// focus (a focus row, written by the page's own intent) and closes; the close chooses nothing.

import {intent, subscribe} from "../../lib/bridge.js";
import {t} from "../../lib/copy.js";
import {whenStarted} from "../../lib/router.js";
import {h} from "../../kit/element.js";

/** A focus's glyph by its key (`Focus.glyph`), wherever the focus shows: Balanced the pump's, Strength the Strength
 * stat's, Endurance the Stamina stat's, Flexibility the Spirit stat's; the pump's for a key this build draws none for. */
export function focusGlyph(key) {
	return {strength: "stat_strength", endurance: "stat_stamina", flexibility: "stat_spirit"}[key] ?? "action_pump";
}

/** The focus window, its choice sent through `choose(id)`: `open(focuses, id)` where the player opened it, the focus in
 * force (`id`) checked; `ask(focuses, level)` for the QP 10 ask; `el` the window, `remove()` takes it out. */
export function focusWindow(choose) {
	const box = h("qp-window");
	const rows = h("qp-card", {rows: true});
	box.content.append(rows);
	let shown = [];
	const pick = (id) => {
		choose(id);
		box.close();
	};
	// Keep Balanced, the ask's answer that keeps the mix as it is: contract/sheet.json lists Balanced first
	box.addEventListener("qp-primary", () => pick(shown[0].id));
	const show = (focuses, checked, level) => {
		shown = focuses;
		box.heading = level === null ? t("character.focus.title") : t("character.focus.ask.title", {level});
		box.primary = level === null ? null : t("character.focus.keep");
		rows.replaceChildren(...focuses.map((focus, i) => {
			// Balanced, the focus before any is chosen, is the one the game recommends
			const name = i === 0 ? t("character.focus.recommended", {focus: focus.name}) : focus.name;
			const row = h("qp-row", {heading: name, glyph: focusGlyph(focus.key), trailing: "checked",
				checked: focus.id === checked});
			row.addEventListener("click", () => pick(focus.id));
			return row;
		}));
		box.open();
	};
	return {
		el: box,
		open: (focuses, id) => show(focuses, id, null),
		ask: (focuses, level) => show(focuses, null, level),
		remove: () => box.remove(),
	};
}

// the QP 10 ask, once Rust says it is due; told it showed, Rust keeps it away for the rest of the launch
let asking = null;
whenStarted(() => subscribe("hero", (hero) => {
	if (!hero?.ask_focus || asking) return;
	asking = focusWindow((id) => intent("hero.focus", {focus: id}).catch(console.error));
	asking.el.addEventListener("qp-closed", () => {
		asking.remove();
		asking = null;
	}, {once: true});
	intent("hero.focus.asked").catch(console.error);
	asking.ask(hero.focuses, hero.ask_focus);
}));
