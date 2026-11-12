// Restore code on the computer (docs/plan.md slice 60, "The screens"; the phones' Restore code page), from the
// `menu.restore` topic (helper/crates/questpump-desk/src/sync.rs): this computer's code in its groups, selectable, held
// behind Show code (`heldCode`), with Copy (Copied for three seconds; it copies the code, shown or held), the one line
// that anyone with it can open the progress, and Done; opened to enter one (`enter`, Email's Enter restore code), the
// field, what it holds masked until Show code, and Continue, the wrong and not-found lines under the field (and that
// the account holds every join until another device makes its new key), and back
// to Email once this computer joined the account the code opens.

import {intent} from "../../lib/bridge.js";
import {t} from "../../lib/copy.js";
import {back, register} from "../../lib/router.js";
import {feedback} from "../../lib/sound.js";
import {h} from "../../kit/element.js";
import {heldCode, page, restoreCode, watch} from "../page.js";

/** The problem's line under the field, none without one. */
function problem(error) {
	switch (error) {
		case "wrong": return t("menu.restore.wrong");
		case "not_found": return t("menu.restore.notfound");
		case "failed": return t("menu.sync.unanswered");
		case "rekey": return t("menu.sync.rekey");
		default: return "";
	}
}

register("desk.menu.restore", (params) => {
	const view = page({heading: t("menu.restore.title")});
	if (params?.enter) {
		const field = h("qp-text-input", {label: t("menu.restore.title"), masked: true});
		const go = h("qp-primary-button", {label: t("menu.restore.continue")});
		const enter = () => intent("restore.enter", {code: field.value}).catch(console.error);
		go.addEventListener("click", enter);
		field.addEventListener("qp-submit", enter);
		// what the player typed or pasted, masked until Show code, and masked again once the field is emptied
		const held = heldCode(view, (typed) => {
			field.masked = typed === null;
			return field;
		});
		field.addEventListener("input", () => held.set(field.value || null));
		view.body.append(field, h("div", {class: "qp-foot"}, held.show, go));
		let done; // the joins at the first look: the next one is this code's
		watch(view, "menu.restore", (r) => {
			if (done !== undefined && r.done > done) {
				// the code restored the account (slice 72: a commitment that succeeded)
				feedback("commit");
				back();
				return;
			}
			done ??= r.done;
			field.disabled = r.busy;
			field.error = problem(r.error);
			go.state = r.busy ? "busy" : "normal";
		});
		intent("restore.look").catch(console.error);
		view.primary = () => field;
		return view;
	}
	const shown = restoreCode(view);
	const done = h("qp-primary-button", {label: t("common.done"), "on:click": () => back()});
	view.body.append(shown.card, h("div", {class: "qp-foot"}, shown.copy, done));
	watch(view, "menu.restore", (r) => shown.set(r.code));
	intent("restore.look").catch(console.error);
	view.primary = () => done;
	return view;
});
