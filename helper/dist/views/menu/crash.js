// The crash report page on the computer (contract/crash-api.yaml, "The desktop app"; helper/src-tauri/src/runtime/
// crash.rs): Rust opens the window on it at the start after QuestPump crashed while a report waits and the Menu's Send
// crash reports is not answered yet (an opt-in region). What a report holds and never holds; every waiting report's
// text exactly as it would go, behind Show the report; Send (this once), Always send (the switch on, then the same) and
// Don't send (deleted; the page closes); after Send the thanks, the line that it waits to go while offline, or that it
// was refused. Everything follows the `crash` topic: its reports, and the outcome of the page's Send.

import {intent} from "../../lib/bridge.js";
import {t} from "../../lib/copy.js";
import {back, register} from "../../lib/router.js";
import {h} from "../../kit/element.js";
import {page, watch, words} from "../page.js";

/** How the page's Send went, in the page's words. */
const OUTCOME = {sent: "crash.sent", kept: "desk.crash.kept", refused: "desk.crash.refused"};

register("desk.crash", () => {
	const view = page({heading: t("crash.title")});
	const show = h("qp-quiet-button", {label: t("desk.crash.show")});
	const texts = h("div", {class: "qp-part", hidden: true});
	const send = h("qp-primary-button", {label: t("crash.send")});
	const always = h("qp-secondary-button", {label: t("crash.always")});
	const skip = h("qp-quiet-button", {label: t("crash.skip")});
	const foot = h("div", {class: "qp-foot"}, send, always, skip);
	const state = h("p", {class: "qp-type-body-strong", role: "status"});
	view.body.append(words(t("desk.crash.body"), "body"), show, texts, foot, state);

	let shown = false;
	let topic = {reports: [], outcome: null};
	const render = () => {
		const asking = topic.reports.length > 0;
		const sending = topic.outcome === "sending";
		show.hidden = !asking;
		texts.hidden = !asking || !shown;
		show.label = t(shown ? "desk.crash.hide" : "desk.crash.show");
		foot.hidden = !asking && !sending;
		send.state = sending ? "busy" : "normal";
		always.hidden = sending;
		skip.hidden = sending;
		state.textContent = OUTCOME[topic.outcome] ? t(OUTCOME[topic.outcome]) : "";
	};
	show.addEventListener("click", () => {
		shown = !shown;
		render();
	});
	// each answer waits for the topic, which brings the reports gone and the outcome
	send.addEventListener("click", () => {
		if (send.state !== "busy") intent("crash.send").catch(console.error);
	});
	always.addEventListener("click", () => intent("crash.always").catch(console.error));
	skip.addEventListener("click", () => intent("crash.skip").then(back, console.error));
	render();
	watch(view, "crash", (c) => {
		topic = c;
		texts.replaceChildren(...c.reports.map((text) => h("qp-card", {}, h("pre", {class: "qp-report", text}))));
		render();
	});
	view.primary = () => send;
	return view;
});
