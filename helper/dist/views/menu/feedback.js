// Send feedback on the computer (docs/plan.md slice 24; docs/ui.md, "The Menu on the computer"): the text in a TextInput
// held to its cap in Unicode scalar values (`feedback.max_chars`, what does not fit where it went in is refused, never
// the end already written; an input method's composition waits for its end), the count said to a screen reader once
// typing stops near the cap; the tick that adds the two system details, shown exactly as they go (the screen
// reading's code with its meaning); Send, and how it went: sent, kept to send later, refused, lost because QuestPump
// could not keep it, or no answer; the messages that wait, and how the last one went (the tray's note). The unsent
// draft stays in this page's own storage, so closing the window or an update's restart does not lose it.

import {call} from "../../lib/bridge.js";
import {t} from "../../lib/copy.js";
import {register} from "../../lib/router.js";
import {h} from "../../kit/element.js";
import {page, watch, words} from "../page.js";

/** The unsent draft's key in this page's storage. */
const DRAFT = "questpump.feedback.draft";
/** Where an answer comes from when a player wants one. */
const ADDRESS = "contact@questpump.com";
/** The text box's id, which its label names, and its count's, which describes it (one feedback page stands at a
 * time). */
const BOX_ID = "qp-feedback-text";
const COUNT_ID = "qp-feedback-count";
/** How near the cap the count is said to a screen reader, and how long after the last key. */
const NEAR = 50;
const SAID_MS = 1000;

const CAPTURE = {
	live: "desk.feedback.capture.live",
	no_game: "desk.feedback.capture.nogame",
	no_strip: "desk.feedback.capture.nostrip",
	stopped: "desk.feedback.capture.stopped",
	screen_permission: "desk.feedback.capture.permission",
};

function saveDraft(value) {
	try {
		if (value) localStorage.setItem(DRAFT, value);
		else localStorage.removeItem(DRAFT);
	} catch {
		// no storage: the draft lives only in the box
	}
}

function loadDraft() {
	try {
		return localStorage.getItem(DRAFT) ?? "";
	} catch {
		return "";
	}
}

register("desk.feedback", () => {
	const view = page({heading: t("menu.feedback.row")});
	const note = h("p", {class: "qp-type-meta", role: "status"});
	const box = h("qp-text-input", {mode: "prose", lines: 8, label: t("feedback.box.label"),
		prompt: t("feedback.placeholder")});
	const count = h("p", {class: "qp-type-meta qp-count", "aria-hidden": "true"});
	const countWords = h("span", {class: "qp-unseen", id: COUNT_ID});
	const countSaid = h("span", {class: "qp-unseen", role: "status"});
	const tick = h("qp-row", {heading: t("desk.feedback.tick"), glyph: "action_log", trailing: "toggle",
		checked: true});
	const what = words("", "meta");
	const from = words("", "body");
	const os = h("code");
	const capture = h("code");
	const captureWords = h("span");
	const details = h("qp-card", {class: "qp-details"},
		h("p", {class: "qp-type-meta", text: t("desk.feedback.system")}), os,
		h("p", {class: "qp-type-meta", text: t("desk.menu.computer.screen")}), h("p", {}, capture, captureWords));
	const send = h("qp-primary-button", {label: t("feedback.send"), state: "disabled"});
	const state = h("p", {class: "qp-type-body-strong", role: "status"});
	const waitingLine = words("", "meta");
	view.body.append(note, h("label", {class: "qp-type-label", for: BOX_ID, text: t("feedback.box.label")}), box, count,
		countWords, countSaid, h("qp-card", {rows: true}, tick), what, from, details,
		words(t("feedback.reply", {address: ADDRESS}), "meta"), send, state, waitingLine);

	let topic = {max_chars: 0, waiting: 0};
	let sending = false;
	let composing = false;
	let shown = ""; // what the line under Send says: "", "sending", an outcome, "error", or "settled"
	let queuedSeen = false; // the topic counted the message the line says waits, so a count of 0 later means it left
	let said = null;

	const text = () => box.control;
	const showState = (outcome, line) => {
		shown = outcome;
		const lines = {
			sending: t("feedback.sending"),
			sent: t("feedback.sent"),
			queued: t("desk.feedback.queued"),
			refused: t("feedback.refused", {address: ADDRESS}),
			unsaved: t("desk.feedback.unsaved"),
			error: t("desk.feedback.error"),
		};
		state.textContent = line ?? lines[outcome] ?? "";
		renderWaiting();
	};
	const renderWaiting = () => {
		waitingLine.hidden = topic.waiting === 0 || shown !== "";
		waitingLine.textContent = topic.waiting ? t("feedback.waiting", {count: topic.waiting}) : "";
	};

	/** The cap: what does not fit where it went in (just before the caret) is refused; true when something was cut. */
	const holdToCap = () => {
		const area = text();
		const chars = Array.from(area.value);
		const over = chars.length - topic.max_chars;
		if (over <= 0) return false;
		const caret = Array.from(area.value.slice(0, area.selectionEnd)).length;
		const keep = Math.max(0, caret - over);
		let kept = chars.slice(0, keep).concat(chars.slice(caret));
		if (kept.length > topic.max_chars) kept = kept.slice(0, topic.max_chars); // a draft longer than the cap
		area.value = kept.join("");
		const at = Math.min(chars.slice(0, keep).join("").length, area.value.length);
		area.setSelectionRange(at, at);
		return true;
	};

	const changed = (inComposition) => {
		const cap = topic.max_chars;
		const cut = cap > 0 && !composing && !inComposition && holdToCap();
		const n = Array.from(text().value).length;
		const counted = cap ? t("feedback.counter.label", {count: n, max: cap}) : "";
		count.textContent = !cap ? "" : cut ? t("desk.feedback.fits", {max: cap}) : t("feedback.counter", {count: n,
			max: cap});
		countWords.textContent = counted;
		clearTimeout(said);
		if (cut) countSaid.textContent = t("desk.feedback.fits", {max: cap});
		else if (cap && cap - n <= NEAR) {
			// said once typing stops, so a screen reader does not talk over each key
			said = setTimeout(() => {
				countSaid.textContent = counted;
			}, SAID_MS);
		}
		send.state = sending ? "busy" : text().value.trim() === "" ? "disabled" : "normal";
	};

	const render = () => {
		note.textContent = topic.note_line ?? "";
		note.hidden = !topic.note_line;
		from.textContent = t("desk.feedback.from", {version: topic.version ?? ""});
		os.textContent = topic.os ?? "";
		capture.textContent = topic.capture ?? "";
		captureWords.textContent = CAPTURE[topic.capture] ? `: ${t(CAPTURE[topic.capture])}` : "";
		details.hidden = !tick.checked;
		what.textContent = t(tick.checked ? "desk.feedback.note.ticked" : "feedback.numbers.note.off");
		if (shown === "queued" && topic.waiting > 0) queuedSeen = true;
		// the message the line said waits has left: the line says how the latest one went
		if (shown === "queued" && queuedSeen && topic.waiting === 0) showState("settled", topic.note_line ?? "");
		renderWaiting();
	};

	box.addEventListener("compositionstart", () => {
		composing = true;
	});
	box.addEventListener("compositionend", () => {
		composing = false;
		changed(false);
		saveDraft(text().value);
	});
	box.addEventListener("input", (e) => {
		// a new message starts: the last one's outcome no longer stands under Send
		if (shown !== "" && shown !== "sending") showState("");
		changed(e.isComposing === true);
		saveDraft(text().value);
	});
	tick.addEventListener("qp-change", (e) => {
		tick.checked = e.detail;
		render();
	});
	send.addEventListener("click", () => {
		if (send.state !== "normal") return;
		sending = true;
		changed(false);
		showState("sending");
		// QuestPump holds the message now; the draft comes back only when it did not go and was not kept
		saveDraft("");
		call("send_feedback", {text: text().value, diagnostics: tick.checked}).then((outcome) => {
			queuedSeen = topic.waiting > 0; // the topic's count may come before this answer or after it
			showState(outcome);
			// a message sent or kept needs no second Send; any other stays in the box to send again or copy
			if (outcome === "sent" || outcome === "queued") box.value = "";
			else saveDraft(text().value);
		}, (error) => {
			console.error(error);
			showState("error");
			saveDraft(text().value);
		}).finally(() => {
			sending = false;
			changed(false);
			box.focus();
		});
	});

	box.ensure();
	// the label names the box and a click on it reaches the box; the box's description is its count
	text().id = BOX_ID;
	text().setAttribute("aria-describedby", COUNT_ID);
	box.value = loadDraft();
	watch(view, "feedback", (f) => {
		const capChanged = f.max_chars !== topic.max_chars;
		topic = f;
		if (capChanged) changed(false);
		render();
	});
	view.stops.push(() => clearTimeout(said));
	view.primary = () => box;
	return view;
});
