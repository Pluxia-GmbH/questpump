// Email on the computer (docs/plan.md slice 60, "The screens"; the phones' Email page), from the `menu.email` topic
// (helper/crates/questpump-desk/src/sync.rs), each step as Rust's flow names it, in the account this computer holds a
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
// heard as it comes. In the browser (docs/plan.md slice 63) the device is this browser, and Save your progress opens
// the page on its Save flow (`save`): its heading, no Sign in or Enter restore code, Send making the account first
// (Keep your key follows any way back the player gets: web/views/index.js); its sign-in (I played before) offers Link
// a device, whose request code a phone answers, while the phone apps are in the stores (`requests`). There Sign out
// stands whenever the browser is in an account, an email linked or not, and goes once the rows it made reached the
// account; where they could not, the line that says so with Sign out anyway and Stay signed in (`unsent`) in the
// page's place.

import {get, intent} from "../../lib/bridge.js";
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

register("desk.menu.email", (params) => {
	const saving = Boolean(params?.save);
	const view = page({heading: t(saving ? "menu.sync.save.title" : "menu.email.title")});
	const web = get("shell")?.platform === "web";
	const computer = t(web ? "desk.menu.email.browser" : "desk.menu.email.computer");
	const enterRestore = () => push("desk.menu.restore", {enter: true});
	const openRestore = () => push("desk.menu.restore");
	/** Restore code's row: in the linked view's card, and in a card of its own in the not-linked view. */
	const restoreRow = () => h("qp-row", {heading: t("menu.restore.title"), glyph: "mark_lock",
		"on:click": openRestore});

	// not linked, or a new address
	const signin = h("qp-section-label", {label: t("menu.email.signin.title")});
	const address = h("qp-text-input", {label: t("menu.email.field"), inputmode: "email"});
	const send = h("qp-primary-button", {label: t("menu.email.send")});
	const sendCode = () => intent("email.send", {email: address.value.trim()}).catch(console.error);
	send.addEventListener("click", sendCode);
	address.addEventListener("qp-submit", sendCode);
	const restoreInstead = h("qp-quiet-button", {label: t("menu.email.needkey.restore"), "on:click": enterRestore});
	// a browser's sign-in while a phone can answer its request code: Link a device, which shows it
	const phoneInstead = h("qp-quiet-button", {label: t("menu.sync.linkdevice"), hidden: true,
		"on:click": () => push("desk.menu.linkdevice")});
	const privacy = h("qp-card", {rows: true}, h("qp-row", {heading: t("menu.privacy"), glyph: "mark_lock",
		trailing: "external", "on:click": () => intent("about.open", {page: "sync"}).catch(console.error)}));
	const field = part(signin, address, send, restoreInstead, phoneInstead, privacy);

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
	/** The linked address, null where this device does not know it (another device linked it, or this one joined by
	 * a code). */
	let linkedAddress = null;
	/** Sign out's row: in the linked view's card, and in the browser in the not-linked one's too. */
	const signOutRow = (key) => h("qp-row", {heading: t("menu.email.signout"), glyph: "mark_close", trailing: "action",
		"data-focus-key": key,
		"on:click": () => confirmWindow(t("menu.email.signout.title", {device: computer}),
			t(web ? "play.signout.body" : "menu.email.signout.body"), t("menu.email.signout"),
			() => intent("email.signout").catch(console.error))});
	const signOut = signOutRow("signout");
	const signOutUnlinked = signOutRow("signout:unlinked");
	const removeEmail = h("qp-row", {heading: t("menu.email.remove"), glyph: "mark_dash", trailing: "action",
		"data-focus-key": "remove",
		"on:click": () => confirmWindow(linkedAddress ? t("menu.email.remove.title", {address: linkedAddress})
			: t("menu.email.remove.unknown"), null, t("menu.sync.remove"),
			() => intent("email.remove").catch(console.error))});
	const linked = part(h("qp-card", {rows: true}, shown,
		h("qp-row", {heading: t("menu.email.change"), glyph: "mark_share", trailing: "action",
			"data-focus-key": "change:linked",
			"on:click": () => intent("email.change").catch(console.error)}),
		restoreRow(), signOut, removeEmail));

	// in an account no email holds: Restore code (and in the browser Sign out), over Delete account
	const unlinkedRestore = h("qp-card", {rows: true, hidden: true}, restoreRow(), web ? signOutUnlinked : null);

	// in the browser, a Sign out whose rows could not reach the account: sign out anyway, or stay
	const unsent = part(h("p", {class: "qp-type-body-strong", text: t("play.signout.unsent")}),
		h("div", {class: "qp-foot"},
			h("qp-primary-button", {label: t("play.signout.stay"), "data-focus-key": "stay",
				"on:click": () => intent("email.stay").catch(console.error)}),
			h("qp-secondary-button", {label: t("play.signout.anyway"),
				"on:click": () => intent("email.signout", {anyway: true}).catch(console.error)})));

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
		glyph: "mark_close", trailing: "action", destructive: true, "data-focus-key": "delete",
		"on:click": () => confirmWindow(t("menu.email.delete.title"), t("menu.email.delete.body"),
			t("menu.email.delete"), () => intent("email.delete").catch(console.error))}));
	const said = h("p", {class: "qp-type-body qp-quiet-line", hidden: true});

	view.body.append(said, unsent, field, codePart, taken, linked, needKey, unlinkedRestore, outLine, remove);

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
		unsent.hidden = !e.unsent;
		field.hidden = e.step !== "field" || linkedView || e.unsent;
		codePart.hidden = e.step !== "code";
		taken.hidden = e.step !== "taken";
		linked.hidden = !linkedView || e.unsent;
		const unsaved = e.save && !e.linked;
		unlinkedRestore.hidden = !e.account || e.linked || e.step !== "field" || e.purpose === "signin" || unsaved
			|| e.unsent;
		needKey.hidden = e.step !== "need_key";
		remove.hidden = !e.account || e.purpose === "signin" || e.step === "need_key" || unsaved || e.unsent;
		want(e.step === "need_key");
		busy = e.busy;

		signin.hidden = e.account || e.save;
		restoreInstead.hidden = e.account || e.save;
		phoneInstead.hidden = !web || !e.requests || e.account || e.save;
		const line = problem(e);
		address.error = e.step === "field" ? line ?? "" : "";
		address.disabled = e.busy;
		send.state = e.busy && e.step === "field" ? "busy" : "normal";

		sentTo.textContent = e.email ? t("menu.email.code.sent", {email: e.email}) : "";
		code.disabled = e.busy;
		code.error = e.step === "code" && e.error === "wrong" ? line : "";
		say(codeLine, e.step === "code" && e.error !== "wrong" ? line ?? "" : "", "error");
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

		linkedAddress = e.address ?? null;
		shown.heading = linkedAddress ?? t("menu.email.linked");
		signOut.disabled = e.busy;
		signOutUnlinked.disabled = e.busy;
		removeEmail.disabled = e.busy;

		held.set(needKey.hidden ? null : e.request ?? null);
		pair.hidden = !e.request;
		say(linkLine, e.linking === "linking" ? t("menu.linkdevice.linking")
			: e.linking === "linked" ? t("menu.linkdevice.linked")
				: e.linking === "failed" ? t("menu.sync.unanswered")
					: e.linking === "other" ? t("desk.menu.link.other")
						: e.linking === "rekey" ? t("menu.sync.rekey") : "",
			e.linking === "linked" ? "commit" : e.linking === "linking" ? null : "error");
		say(outLine, e.out_failed ? t("menu.sync.unanswered") : "", "error");

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
	intent("email.start", saving ? {save: true} : {}).catch(console.error);
	view.stops.push(() => {
		clearInterval(ticker);
		want(false);
	});
	view.primary = () => (!unsent.hidden ? unsent.querySelector("qp-primary-button") : field.hidden ? null : address);
	return view;
});
