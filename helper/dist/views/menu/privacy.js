// The privacy page on the computer (contract/telemetry-api.yaml, "The switches and the consent"; helper/src-tauri/src/
// runtime/usage.rs): Rust opens the first main window the player opens in a run on it while Usage statistics or Send
// crash reports is not answered. The title, one line, the two switches with their lines, set as the region reads them
// (the `usage` and `crash` topics' `on`: off in an opt-in region, never pre-ticked there; on elsewhere), how each is
// used (questpump.com/privacy#telemetry in the browser), and Done, which keeps both as the player's answers and closes
// the page. In an opt-in region (the `usage` topic's `opt_in`, read once a start) learnclash's buttons instead, each
// through the same intent: Accept all, the primary, which answers both on, or Save choices in its place once the
// player moved a switch here, which keeps both as set; and Reject all, a full secondary button under it, which answers
// both off, so a yes and a no take one click each. Closed any other way, nothing is answered and the page comes again
// at the next start. The switches here are the page's own until an answer (each moves at once, as a Toggle row's
// state); the Menu's rows answer one at a time.

import {get, intent} from "../../lib/bridge.js";
import {t} from "../../lib/copy.js";
import {back, register} from "../../lib/router.js";
import {h} from "../../kit/element.js";
import {page, watch, words} from "../page.js";

register("desk.privacy", () => {
	const view = page({heading: t("privacy.title")});
	const usage = h("qp-row", {heading: t("menu.usage"), meta: t("menu.usage.note"), trailing: "toggle"});
	const crash = h("qp-row", {heading: t("menu.crash"), meta: t("menu.crash.note"), trailing: "toggle"});
	const link = h("qp-row", {heading: t("privacy.link"), glyph: "mark_lock", trailing: "external",
		"on:click": () => intent("about.open", {page: "telemetry"}).catch(console.error)});
	const optIn = Boolean(get("usage")?.opt_in);
	const primary = h("qp-primary-button", {label: t(optIn ? "privacy.accept" : "privacy.done")});
	const reject = optIn ? h("qp-secondary-button", {label: t("privacy.reject"), tall: true}) : null;
	view.body.append(words(t("privacy.body"), "body"), h("qp-card", {rows: true}, usage, crash),
		h("qp-card", {rows: true}, link), h("div", {class: "qp-foot"}, primary, reject));

	// the region's reading until the player moves a switch here; in an opt-in region the first move, even one moved
	// back, makes Accept all Save choices
	const moved = {usage: false, crash: false};
	for (const [row, name] of [[usage, "usage"], [crash, "crash"]]) {
		row.addEventListener("qp-change", (e) => {
			moved[name] = true;
			row.checked = e.detail;
			if (optIn) primary.label = t("privacy.save");
		});
		watch(view, name, (topic) => {
			if (!moved[name]) row.checked = topic.on;
		});
	}
	// one answer at a time; the Primary works busy, Reject all (no busy state) stands disabled meanwhile
	let answering = false;
	const answer = (button, choice) => {
		if (answering) return;
		answering = true;
		const busy = button === primary;
		if (busy) button.state = "busy";
		else button.disabled = true;
		intent("privacy.done", choice).then(back, (e) => {
			console.error(e);
			if (busy) button.state = "normal";
			else button.disabled = false;
			answering = false;
		});
	};
	const accepting = () => optIn && !moved.usage && !moved.crash;
	primary.addEventListener("click", () => answer(primary, accepting() ? {usage: true, crash: true}
		: {usage: Boolean(usage.checked), crash: Boolean(crash.checked)}));
	reject?.addEventListener("click", () => answer(reject, {usage: false, crash: false}));
	// in an opt-in region the focus starts on the heading, as learnclash's dialog has no default button, so a stray
	// Enter answers nothing
	view.primary = () => optIn ? null : primary;
	return view;
});
