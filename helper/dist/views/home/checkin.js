// The day-2 check-in on Pump home (Panel.lua RenderCheckin; iOS PumpTab's checkin and JointWindow.whereItHurts): one
// card with the game's question and its three answers as ChoiceChips, once, on the app's days two to four, disabled
// while the engine is not ready (Rust writes no answer while it folds or after it failed; an answer the engine did not
// write brings the card back); the last answer (very sore, or dark urine) asks where it hurts, in a window only its
// Save closes, once Rust took the joints: a Save refused keeps the window and the switches and says the joints were not
// saved, and so does one the engine did not write, whose window opens again with them ticked (the `home` topic's
// `pain_failed`). No safety note and no line for a joint flagged again: the Menu's Safety note page alone carries the
// note (David, 2026-10-24: only what is legally necessary). The words are the game's (the `home` topic's `checkin`,
// contract/sheet.json); Rust writes the rows (`home.checkin`, `home.pain`).

import {intent} from "../../lib/bridge.js";
import {t} from "../../lib/copy.js";
import {h} from "../../kit/element.js";
import {words} from "../page.js";

/** The check-in's card: the question and its three answers. */
export function checkinCard() {
	const question = h("p", {class: "qp-type-body-strong"});
	const answers = h("div", {class: "qp-chips"});
	const card = h("qp-card", {class: "qp-stack-small"}, question, answers);
	let held = null;
	// the last failed Save this page opened the window again for
	let failSeen = 0;
	const where = jointWindow();
	return {
		el: card,
		/** Opens the joints window again for a Save the engine did not write (the `home` topic's `pain_failed`), once
		 * per failure. */
		failed(pain) {
			if (!pain || pain.seq <= failSeen) return;
			failSeen = pain.seq;
			where.open(pain.checkin, pain.joints);
		},
		/** Draws the check-in of the `home` topic, or hides the card; its answers wait while the engine is not ready
		 * (`busy`). */
		draw(checkin, busy) {
			card.hidden = !checkin;
			for (const chip of answers.children) chip.disabled = busy;
			const text = JSON.stringify(checkin);
			if (!checkin || held === text) return;
			held = text;
			question.textContent = checkin.words.question;
			const answer = (code) => intent("home.checkin", {answer: code}).catch(console.error);
			answers.replaceChildren(
				h("qp-choice-chip", {label: checkin.words.fine, disabled: busy, "on:click": () => answer(1)}),
				h("qp-choice-chip", {label: checkin.words.sore, disabled: busy, "on:click": () => answer(2)}),
				h("qp-choice-chip", {label: checkin.words.dark, disabled: busy, "on:click": () => {
					answer(3);
					where.open(checkin);
				}}),
			);
		},
	};
}

/** Where it hurts, after the check-in's last answer: the joints as switches (those flagged already on and fixed, a
 * joint the game flagged named "(in the game)" as the phones' JointWindow), and Save, the one way out: busy until Rust
 * took the joints, and a Save refused stays open with its switches and says so; a Save with none ticked puts the window
 * away (and answers a failed Save the window was opened again for). */
function jointWindow() {
	const box = h("qp-window", {locked: true, placement: "bottom", primary: t("pump.joints.save")});
	const list = h("qp-card", {rows: true});
	const failed = words(t("desk.home.pain.failed"), "body");
	failed.setAttribute("role", "alert");
	box.content.append(list, failed);
	let ticked = new Set();
	let saving = false;
	box.addEventListener("qp-primary", () => {
		if (saving) return;
		const joints = [...ticked].sort((a, b) => a - b);
		saving = true;
		box.primaryState = "busy";
		failed.hidden = true;
		intent("home.pain", {joints}).then(() => box.close(), (e) => {
			console.error(e);
			failed.hidden = false;
		}).finally(() => {
			saving = false;
			box.primaryState = "normal";
		});
	});
	return {
		/** Opens it on the check-in's words with `again` ticked: the joints of a Save the engine did not write, which
		 * it says were not saved. */
		open(checkin, again = []) {
			ticked = new Set(checkin.joints.filter((j) => again.includes(j.id) && !j.flagged).map((j) => j.id));
			failed.hidden = !ticked.size;
			box.heading = checkin.pain.where;
			list.replaceChildren(...checkin.joints.map((joint) => {
				const name = joint.game ? t("menu.free.pain.ingame", {joint: joint.name}) : joint.name;
				const row = h("qp-row", {heading: name, trailing: "toggle", checked: joint.flagged || ticked.has(joint.id),
					disabled: joint.flagged});
				row.addEventListener("qp-change", (e) => {
					if (e.detail) ticked.add(joint.id);
					else ticked.delete(joint.id);
					row.checked = e.detail;
				});
				return row;
			}));
			box.open();
		},
	};
}
