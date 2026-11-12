// Link a device on the computer (docs/plan.md slice 60, "The screens"; the phones' Link a device page), from the
// `menu.linkdevice` topic (helper/crates/questpump-desk/src/sync.rs): this computer's code as a QR, the link code while
// it is in its account (a phone scans it and joins: "Scan this with QuestPump on your other device"), else the request
// code, which a phone in an account scans and answers with the account's key, sealed ("Scan this with QuestPump on your
// phone"), and how that answer went (Linking…, Linked, that the server did not answer, or, after an email sign-in, that
// a phone of other progress was refused, or that the account holds every join until another device makes its new key),
// heard as it comes. An answer no email sign-in binds shows its check code in the code's place, "Does your phone show
// this number?" with Yes, join (`sync.link.join`) and No, which makes a new request (`sync.link.refuse`): whoever saw
// the code could have answered first, and only the player's own phone shows the same number. The page asks Rust for the
// request code while it stands (`sync.link.start`, `sync.link.stop`), and holds either code behind Show code
// (`heldCode`). The link code also goes by Copy (Copied for three seconds; shown or held, as the Restore code's), to
// paste where a camera cannot scan it: a browser at play.questpump.com (docs/plan.md slice 63).

import {intent} from "../../lib/bridge.js";
import {t} from "../../lib/copy.js";
import {announce, focusOn} from "../../lib/focus.js";
import {register} from "../../lib/router.js";
import {h} from "../../kit/element.js";
import {copyButton, heldCode, page, resultLine, say, watch, words} from "../page.js";

register("desk.menu.linkdevice", () => {
	const view = page({heading: t("menu.sync.linkdevice")});
	const well = h("qp-qr-well", {label: t("menu.sync.linkdevice"), tabindex: "-1"});
	const scan = words("", "body");
	scan.hidden = true;
	const held = heldCode(view, (code) => {
		well.code = code;
		scan.hidden = code === null;
		return well;
	});
	const pair = h("qp-card", {class: "qp-pair"}, well, scan, held.show);
	// the link code's text, which a request code has none of
	const copy = copyButton(view);
	const copied = h("div", {class: "qp-foot", hidden: true}, copy.button);
	const line = resultLine();
	// an answer no email sign-in binds: its check code, joined only on the player's Yes
	const asked = t("desk.menu.link.check.title");
	const digits = h("p", {class: "qp-type-num-medium qp-centre"});
	const yes = h("qp-primary-button", {label: t("desk.menu.link.check.yes"),
		"on:click": () => intent("sync.link.join").catch(console.error)});
	const no = h("qp-secondary-button", {label: t("desk.menu.link.check.no"),
		"on:click": () => intent("sync.link.refuse").catch(console.error)});
	const check = h("div", {class: "qp-part", hidden: true},
		h("qp-card", {class: "qp-stack-small"}, h("p", {class: "qp-type-body-strong", text: asked}), digits),
		h("div", {class: "qp-foot"}, yes, no));
	view.body.append(pair, copied, check, line);
	watch(view, "menu.linkdevice", (l) => {
		held.set(l.code ?? null);
		pair.hidden = !l.code;
		copy.set(l.text);
		copied.hidden = !l.code || !l.text;
		const number = l.check ?? "";
		if (number && number !== digits.textContent) {
			// heard digit by digit
			announce(`${asked} ${[...number].join(" ")}`);
			if (check.hidden) queueMicrotask(() => focusOn(yes));
		}
		digits.textContent = number;
		check.hidden = !number;
		// the link code goes to any device of the player's, the request code to a phone, which alone answers it
		scan.textContent = t(l.kind === "request" ? "desk.menu.link.request" : "menu.linkdevice.show");
		say(line, l.state === "linking" ? t("menu.linkdevice.linking")
			: l.state === "linked" ? t("menu.linkdevice.linked")
				: l.state === "failed" ? t("menu.sync.unanswered")
					: l.state === "other" ? t("desk.menu.link.other")
						: l.state === "rekey" ? t("menu.sync.rekey") : "",
		// a device linked is a commitment that succeeded; a link that did not come about, an error (slice 72)
		l.state === "linked" ? "commit" : l.state === "linking" ? null : "error");
	});
	intent("sync.link.start").catch(console.error);
	view.stops.push(() => intent("sync.link.stop").catch(console.error));
	view.primary = () => (!check.hidden ? yes : well.hidden ? held.show : well);
	return view;
});
