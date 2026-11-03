// Restore code on the computer (docs/plan.md slice 60, "The screens"; the phones' Restore code page), from the
// `menu.restore` topic (helper/src-tauri/src/runtime/sync.rs): this computer's code in its groups, selectable, held
// behind Show code (`heldCode`), with Copy (Copied for three seconds; it copies the code, shown or held), the one line
// that anyone with it can open the progress, and Done; opened to enter one (`enter`, Email's Enter restore code), the
// field, what it holds masked until Show code, and Continue, the wrong and not-found lines under the field (and that
// the account holds every join until another device makes its new key), and back
// to Email once this computer joined the account the code opens.

import {intent} from "../../lib/bridge.js";
import {t} from "../../lib/copy.js";
import {announce} from "../../lib/focus.js";
import {back, register} from "../../lib/router.js";
import {h} from "../../kit/element.js";
import {heldCode, page, watch, words} from "../page.js";

/** How long Copy says Copied. */
const SAID_MS = 3000;

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
	const code = h("p", {class: "qp-type-num-medium qp-words-block qp-centre", hidden: true});
	const held = heldCode(view, (shown) => {
		code.textContent = shown ?? "";
		code.hidden = shown === null;
		return code;
	});
	let restoreCode = "";
	const copy = h("qp-secondary-button", {label: t("menu.restore.copy")});
	let reset = null;
	copy.addEventListener("click", () => {
		navigator.clipboard.writeText(restoreCode).then(() => {
			copy.label = t("menu.restore.copied");
			announce(copy.label);
			clearTimeout(reset);
			reset = setTimeout(() => {
				copy.label = t("menu.restore.copy");
			}, SAID_MS);
		}, console.error);
	});
	const done = h("qp-primary-button", {label: t("common.done"), "on:click": () => back()});
	view.body.append(h("qp-card", {class: "qp-stack-small"}, code, held.show, words(t("menu.restore.line"), "body")),
		h("div", {class: "qp-foot"}, copy, done));
	watch(view, "menu.restore", (r) => {
		restoreCode = r.code ?? "";
		copy.disabled = !r.code;
		held.set(r.code || null);
	});
	intent("restore.look").catch(console.error);
	view.stops.push(() => clearTimeout(reset));
	view.primary = () => done;
	return view;
});
