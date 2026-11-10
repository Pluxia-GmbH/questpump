// Keep your key (docs/plan.md slice 63, "Save your progress"): the browser's last step of Save your progress, once the
// email is linked. The restore code held behind Show code (`heldCode`) with Copy (Copied for three seconds) and the one
// line that anyone with it can open the progress; Save in your password manager, a real form with the address as its
// `username` and the code as its `new-password` (WCAG 2.2 SC 3.3.8), whose submit hands both to the browser's
// credential store (`PasswordCredential`, Chrome's and Edge's), so it shows only where that store is and the address
// is known: Firefox offers to keep no field a script filled (its LoginManagerChild), WebKit has no such store (real
// Safari's own offer unobserved), and a join by a restore code knows no address, each a button that did nothing
// (2026-11-07); then Done (`play.done`, which asks the browser to keep this page's storage, and Pump home) over Get
// QuestPump for your computer (questpump.com/from/play in a new tab, the click counts' source `play`,
// contract/get-api.yaml). From the `menu.restore` and `menu.email` topics. Only the browser imports it
// (web/views/index.js); the desktop app never registers it.

import {intent} from "../../lib/bridge.js";
import {t} from "../../lib/copy.js";
import {goTop, register} from "../../lib/router.js";
import {h} from "../../kit/element.js";
import {GET_COMPUTER, page, restoreCode, watch} from "../page.js";

register("desk.play.key", () => {
	const view = page({heading: t("play.key.title")});
	const shown = restoreCode(view);
	// the password manager's form: the fields it reads, kept out of sight, and its one button
	const user = h("input", {type: "email", name: "username", autocomplete: "username", readonly: true,
		tabindex: "-1", "aria-hidden": "true"});
	const secret = h("input", {type: "password", name: "password", autocomplete: "new-password", readonly: true,
		tabindex: "-1", "aria-hidden": "true"});
	const keep = h("qp-secondary-button", {label: t("play.passwords"), hidden: true});
	const storable = typeof PasswordCredential === "function";
	const offer = () => {
		keep.hidden = !(storable && user.value && secret.value);
	};
	const form = h("form", {class: "qp-unseen", method: "post", action: "#"}, user, secret);
	form.addEventListener("submit", (e) => {
		e.preventDefault();
		if (storable && user.value && secret.value) {
			const credential = new PasswordCredential({id: user.value, password: secret.value, name: t("app.name")});
			navigator.credentials.store(credential).catch(console.error);
		}
	});
	keep.addEventListener("click", () => form.requestSubmit());
	const done = h("qp-primary-button", {label: t("common.done"),
		"on:click": () => intent("play.done").then(() => goTop("pump")).catch(console.error)});
	const get = h("qp-quiet-button", {label: t("setup.add.get"),
		"on:click": () => window.open(GET_COMPUTER, "_blank", "noopener")});
	view.body.append(shown.card, form, h("div", {class: "qp-foot"}, shown.copy, keep, done, get));
	watch(view, "menu.restore", (r) => {
		secret.value = r.code ?? "";
		offer();
		shown.set(r.code);
	});
	watch(view, "menu.email", (e) => {
		user.value = e.address ?? "";
		offer();
	});
	intent("restore.look").catch(console.error);
	view.primary = () => done;
	return view;
});
