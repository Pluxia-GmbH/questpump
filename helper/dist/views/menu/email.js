// Email on the computer (docs/plan.md slice 60, "The screens"; the phones' Email page), from the `menu.email` topic
// (helper/src-tauri/src/runtime/sync.rs), each step as Rust's flow names it, in the account this computer holds a
// token and the key R for whatever Sync's switch says (`account`, `Account::held`): not linked, the address field,
// Send code and the Privacy link (the privacy page's sync part), then Restore code while this computer is in an
// account, or on a sign-in (this computer in no account) Enter restore code; the code: where it went, six boxes over
// one field the system's one-time code fills, which goes at its sixth digit or at Enter, Send again 30 s after the
// last code with its countdown, Change email, the wrong, expired and too-many lines, and that the account holds every
// join until another device makes its new key; an address another account
// holds: Sign in and join, Change email; linked: the address (else Email linked, where another device linked it),
// Change email, Restore code, Sign out and Remove email (each asked once); after a sign-in on a computer without the
// account's key: the request code as a QR, which a phone in the account scans and answers (a phone of other progress
// is refused, its line said and a new code shown), and Enter restore code, the request code held behind Show code
// (`heldCode`). So Restore code is there whenever this computer is in an account, an email linked or not, and opens
// only from its row. While this computer is in an account and not signing in, Delete account (asked once) is last,
// with the line of a Sign out, Remove email or Delete account the server did not answer over it. Every result line is
// heard as it comes.

import {intent} from "../../lib/bridge.js";
import {t} from "../../lib/copy.js";
import {announce} from "../../lib/focus.js";
import {push, register} from "../../lib/router.js";
import {h} from "../../kit/element.js";
import {confirmWindow} from "../book/common.js";
import {heldCode, page, resultLine, say, watch, words} from "../page.js";

/** How long after a code Send again waits. */
const AGAIN_MS = 30 * 1000;
/** A code's digits. */
const DIGITS = 6;

/** The line of a step's problem, none without one. */
function problem(e) {
	switch (e.error) {
		case "invalid": return t("menu.email.invalid");
		case "wrong": return t("menu.email.code.wrong", {count: e.tries_left ?? 0});
		case "expired": return t("menu.email.code.expired");
		case "limit": return t("menu.email.code.limit");
		case "failed": return t("menu.sync.unanswered");
		case "rekey": return t("menu.sync.rekey");
		default: return null;
	}
}

/** A step's own part of the page, shown alone. */
function part(...children) {
	return h("div", {class: "qp-part", hidden: true}, ...children);
}

register("desk.menu.email", () => {
	const view = page({heading: t("menu.email.title")});
	const computer = t("desk.menu.email.computer");
	const enterRestore = () => push("desk.menu.restore", {enter: true});
	const openRestore = () => push("desk.menu.restore");
	/** Restore code's row: in the linked view's card, and in a card of its own in the not-linked view. */
	const restoreRow = () => h("qp-row", {heading: t("menu.restore.title"), "on:click": openRestore});

	// not linked, or a new address
	const signin = h("qp-section-label", {label: t("menu.email.signin.title")});
	const address = h("qp-text-input", {label: t("menu.email.field"), inputmode: "email"});
	const send = h("qp-primary-button", {label: t("menu.email.send")});
	const sendCode = () => intent("email.send", {email: address.value.trim()}).catch(console.error);
	send.addEventListener("click", sendCode);
	address.addEventListener("qp-submit", sendCode);
	const restoreInstead = h("qp-quiet-button", {label: t("menu.email.needkey.restore"), "on:click": enterRestore});
	const privacy = h("qp-card", {rows: true}, h("qp-row", {heading: t("menu.privacy"), glyph: "mark_lock",
		trailing: "external", "on:click": () => intent("about.open", {page: "sync"}).catch(console.error)}));
	const field = part(signin, address, send, restoreInstead, privacy);

	// the code
	const sentTo = words("", "body");
	const code = h("qp-text-input", {label: t("menu.email.code.field"), mode: "code", digits: DIGITS});
	let typed = "";
	const sendAnswer = () => {
		if (code.value.length === DIGITS) intent("email.code", {code: code.value}).catch(console.error);
	};
	code.addEventListener("input", () => {
		// the field keeps digits alone; the sixth sends it, once for each code typed
		if (code.value.length === DIGITS && code.value !== typed) sendAnswer();
		typed = code.value;
	});
	code.addEventListener("qp-submit", sendAnswer);
	const again = h("qp-quiet-button", {label: t("menu.email.code.again"),
		"on:click": () => intent("email.again").catch(console.error)});
	const change = (key) => h("qp-quiet-button", {label: t("menu.email.change"), "data-focus-key": key,
		"on:click": () => intent("email.change").catch(console.error)});
	const codeLine = resultLine();
	const codePart = part(h("qp-section-label", {label: t("menu.email.code.title")}), sentTo, code, codeLine, again,
		change("change:code"));

	// an address another account holds
	const join = h("qp-primary-button", {label: t("menu.email.taken.join"),
		"on:click": () => intent("email.join").catch(console.error)});
	const takenLine = resultLine();
	const taken = part(h("p", {class: "qp-type-body-strong", text: t("menu.email.taken.title")}), takenLine, join,
		change("change:taken"));

	// linked
	const shown = h("qp-row", {glyph: "mark_check", trailing: "shown"});
	const signOut = h("qp-row", {heading: t("menu.email.signout"), trailing: "action", "data-focus-key": "signout",
		"on:click": () => confirmWindow(t("menu.email.signout.title", {device: computer}), t("menu.email.signout.body"),
			t("menu.email.signout"), () => intent("email.signout").catch(console.error))});
	const removeEmail = h("qp-row", {heading: t("menu.email.remove"), trailing: "action", "data-focus-key": "remove",
		"on:click": () => confirmWindow(t("menu.email.remove.title", {address: shown.heading}), null,
			t("menu.sync.remove"), () => intent("email.remove").catch(console.error))});
	const linked = part(h("qp-card", {rows: true}, shown,
		h("qp-row", {heading: t("menu.email.change"), trailing: "action", "data-focus-key": "change:linked",
			"on:click": () => intent("email.change").catch(console.error)}),
		restoreRow(), signOut, removeEmail));

	// in an account no email holds: Restore code, over Delete account
	const unlinkedRestore = h("qp-card", {rows: true, hidden: true}, restoreRow());

	// a sign-in on a computer without the account's key
	const needTitle = t("menu.email.needkey.title", {device: computer});
	const well = h("qp-qr-well", {label: needTitle});
	const scan = words(t("desk.menu.link.request"), "body");
	scan.hidden = true;
	const held = heldCode(view, (request) => {
		well.code = request;
		scan.hidden = request === null;
		return well;
	});
	const linkLine = resultLine();
	const restore = h("qp-secondary-button", {label: t("menu.email.needkey.restore"), "on:click": enterRestore});
	const pair = h("qp-card", {class: "qp-pair"}, well, scan, held.show);
	const needKey = part(h("p", {class: "qp-type-body-strong", text: needTitle}), pair, linkLine, restore);

	// in an account: a Sign out, Remove email or Delete account the server did not answer, then Delete account
	const outLine = resultLine();
	const remove = h("qp-card", {rows: true, hidden: true}, h("qp-row", {heading: t("menu.email.delete"),
		trailing: "action", destructive: true, "data-focus-key": "delete",
		"on:click": () => confirmWindow(t("menu.email.delete.title"), t("menu.email.delete.body"),
			t("menu.email.delete"), () => intent("email.delete").catch(console.error))}));
	const said = h("p", {class: "qp-type-body qp-quiet-line", hidden: true});

	view.body.append(said, field, codePart, taken, linked, needKey, unlinkedRestore, outLine, remove);

	let wanting = false; // whether this page asked Rust for the request code
	const want = (on) => {
		if (on !== wanting) intent(on ? "sync.link.start" : "sync.link.stop").catch(console.error);
		wanting = on;
	};
	let heard; // the said seq at the first look: one that comes while the page shows is said
	let sentAt = null;
	let busy = false;
	let ticker = null;
	const tick = () => {
		const left = sentAt ? Math.ceil((sentAt + AGAIN_MS - Date.now()) / 1000) : 0;
		again.disabled = left > 0 || busy;
		again.label = left > 0 ? t("menu.email.code.wait", {seconds: left}) : t("menu.email.code.again");
		if (left <= 0) {
			clearInterval(ticker);
			ticker = null;
		}
	};
	watch(view, "menu.email", (e) => {
		const linkedView = e.linked && e.step === "field" && e.purpose !== "change";
		field.hidden = e.step !== "field" || linkedView;
		codePart.hidden = e.step !== "code";
		taken.hidden = e.step !== "taken";
		linked.hidden = !linkedView;
		unlinkedRestore.hidden = !e.account || e.linked || e.step !== "field" || e.purpose === "signin";
		needKey.hidden = e.step !== "need_key";
		remove.hidden = !e.account || e.purpose === "signin" || e.step === "need_key";
		want(e.step === "need_key");
		busy = e.busy;

		signin.hidden = e.account;
		restoreInstead.hidden = e.account;
		const line = problem(e);
		address.error = e.step === "field" ? line ?? "" : "";
		address.disabled = e.busy;
		send.state = e.busy && e.step === "field" ? "busy" : "normal";

		sentTo.textContent = e.email ? t("menu.email.code.sent", {email: e.email}) : "";
		code.disabled = e.busy;
		code.error = e.step === "code" && e.error === "wrong" ? line : "";
		say(codeLine, e.step === "code" && e.error !== "wrong" ? line ?? "" : "");
		if (e.step !== "code") {
			code.value = "";
			typed = "";
		}
		if (e.sent_at !== sentAt) {
			sentAt = e.sent_at;
			clearInterval(ticker);
			ticker = setInterval(tick, 1000);
		}
		tick();

		say(takenLine, e.step === "taken" ? line ?? "" : "");
		join.state = e.busy && e.step === "taken" ? "busy" : "normal";

		shown.heading = e.address ?? t("menu.email.linked");
		signOut.disabled = e.busy;
		removeEmail.disabled = e.busy;

		held.set(needKey.hidden ? null : e.request ?? null);
		pair.hidden = !e.request;
		say(linkLine, e.linking === "linking" ? t("menu.linkdevice.linking")
			: e.linking === "linked" ? t("menu.linkdevice.linked")
				: e.linking === "failed" ? t("menu.sync.unanswered")
					: e.linking === "other" ? t("desk.menu.link.other")
						: e.linking === "rekey" ? t("menu.sync.rekey") : "");
		say(outLine, e.out_failed ? t("menu.sync.unanswered") : "");

		// "Email linked" or "Account deleted", once it comes while the page shows; a sign-out says nothing more than the
		// page it leaves
		const seq = e.said?.seq ?? 0;
		const what = e.said?.what;
		said.textContent = what === "linked" ? t("menu.email.linked") : what === "deleted" ? t("menu.email.deleted") : "";
		said.hidden = heard === undefined || seq <= heard || !said.textContent;
		const fresh = !said.hidden && String(seq) !== said.dataset.seq;
		said.dataset.seq = String(seq);
		heard ??= seq;
		if (fresh) announce(said.textContent);
	});
	intent("email.start").catch(console.error);
	view.stops.push(() => {
		clearInterval(ticker);
		want(false);
	});
	view.primary = () => (field.hidden ? null : address);
	return view;
});
