// QuestPump's pages on the computer (docs/ui.md, "The Menu on the computer"): Safety and about (the rows of the safety
// note and How the heart-rate check works; Pluxia GmbH with the imprint's address, the not-a-medical-device line, the
// contact for questions and injuries and Riot's legal line; the privacy, imprint and security pages in the browser; the
// version), the safety note (the addon's own words, verbatim), How the heart-rate check works (the verdict rule in
// words, App Store guideline 1.4.1; also pushed from a pump's page in the Log), what QuestPump does and does not do on
// this computer (what it watches, keeps, listens to and sends, with this computer's own folders, games and ports; since
// slice 48 the games it asks on the loopback, whose logs and shared memory it reads and whose telemetry it hears, the
// shared files it added its part to, how it notices a game while "Notice games I play" is on, from this system's own
// sources, and while it is off how a Steam game turned on is still told, and the input's idle time the sitting clock
// reads), and
// Pro's page (slice 64: its state, Pro from questpump.com when bought there; what Pro adds on this computer: Pro's
// weeks, the charts, the Path's pace on a ladder's page and the share-card styles; while Pro is off Get Pro, which opens
// questpump.com's Pro page in the browser while it sells Pro and first asks to turn Sync on while it is off, else that
// Pro comes with QuestPump Pro in the app on the phone, and Pair your phone while none is; Manage Pro, Paddle's portal,
// for Pro bought on questpump.com; no price on the computer, from the `menu.pro` topic, runtime/pro.rs). From the
// `menu.about` topic (questpump-desk `menu_pages::about`, with the desktop app's own parts from
// helper/src-tauri/src/desk/menu_pages/about.rs), asked again as a page opens. In the browser (docs/plan.md slice 63)
// Safety and about without How the heart-rate check works (no heart rate there) and Riot's line (no Riot game).

import {get, intent} from "../../lib/bridge.js";
import {t} from "../../lib/copy.js";
import {LOCALE} from "../../lib/format.js";
import {push, register} from "../../lib/router.js";
import {feedback} from "../../lib/sound.js";
import {h} from "../../kit/element.js";
import {page, resultLine, say, watch, words} from "../page.js";
import {pairWindow} from "./phones.js";

/** How the heart-rate check works: a row here and a quiet button on a pump's page in the Log (views/log/pump.js). */
export const HEART = "desk.menu.heart";

/** A line of body words in the second text colour, as the phones draw the not-a-medical-device line. */
function secondary(text) {
	return h("p", {class: "qp-type-body qp-text2 qp-words-block", text});
}

/** A Row that opens one of the site's pages in the browser. */
function external(heading, glyph, name) {
	return h("qp-row", {heading, glyph, trailing: "external",
		"on:click": () => intent("about.open", {page: name}).catch(console.error)});
}

register("desk.menu.about", () => {
	const view = page({heading: t("menu.about.row")});
	const web = get("shell")?.platform === "web";
	const safety = h("qp-row", {heading: t("menu.safety.title"), glyph: "mark_heart",
		"on:click": () => push("desk.menu.safety")});
	const heart = web ? null
		: h("qp-row", {heading: t("menu.heart.row"), glyph: "mark_heart", "on:click": () => push(HEART)});
	const contact = words("", "body");
	const version = h("p", {class: "qp-type-meta qp-centre"});
	view.body.append(h("qp-card", {rows: true}, safety, heart),
		h("qp-section-label", {label: t("menu.about.header")}),
		h("qp-card", {class: "qp-stack-small"}, words(t("menu.about.publisher"), "body"),
			secondary(t("menu.about.medical")), contact, web ? null : words(t("about.riot"), "meta")),
		h("qp-card", {rows: true}, external(t("menu.privacy"), "mark_lock", "privacy"),
			external(t("menu.about.imprint"), "action_book", "imprint"),
			external(t("menu.about.security"), "rating_defense", "security")),
		version);
	watch(view, "menu.about", (a) => {
		contact.textContent = t("menu.about.contact", {address: a.contact});
		version.textContent = t("menu.version", {version: a.version});
	});
	intent("about.look").catch(console.error);
	view.primary = () => safety;
	return view;
});

register("desk.menu.safety", () => {
	const view = page({heading: t("menu.safety.title")});
	const note = words("", "body");
	view.body.append(h("qp-card", {}, note));
	watch(view, "menu.about", (a) => {
		note.textContent = a.safety;
	});
	return view;
});

register(HEART, () => {
	const view = page({heading: t("log.proven")});
	const section = (label, ...lines) => [h("qp-section-label", {label}),
		h("qp-card", {class: "qp-stack-small"}, ...lines)];
	const baseline = words("", "body");
	const rise = words("", "body");
	const computer = words("", "body");
	view.body.append(...section(t("menu.heart.reads"), words(t("desk.menu.heart.reads.line"), "body")),
		...section(t("menu.heart.decides"), baseline, rise, words(t("menu.heart.twice"), "body"), computer),
		...section(t("menu.heart.without"), words(t("menu.heart.without.line"), "body")),
		...section(t("menu.heart.isnot"), words(t("menu.heart.isnot.line"), "body"),
			secondary(t("menu.about.medical"))),
		...section(t("menu.heart.stays"), words(t("desk.menu.heart.stays.line"), "body")));
	watch(view, "menu.about", ({heart}) => {
		baseline.textContent = t("menu.heart.baseline", {before: heart.before});
		rise.textContent = t("menu.heart.rise", {window: heart.window, reps: heart.reps, hold: heart.hold});
		computer.textContent = t("desk.menu.heart.computer", {after: heart.after});
	});
	intent("about.look").catch(console.error);
	return view;
});

register("desk.menu.pro", () => {
	const view = page({heading: t("pro.title")});
	const state = h("p", {class: "qp-type-body-strong", role: "status"});
	const failed = resultLine();
	let latest = null;
	// Get Pro was pressed on this page: Pro coming on after it is the purchase completed (slice 72)
	let buying = false;
	const getPro = () => {
		buying = true;
		if (latest?.sync) {
			intent("pro.get").catch(console.error);
			return;
		}
		// Pro goes with the account: Sync turned on first, then questpump.com's Pro page opens
		const ask = h("qp-window", {heading: t("desk.menu.pro.sync.title"), primary: t("desk.menu.pro.sync.on"),
			secondary: t("menu.sync.notnow")});
		ask.addEventListener("qp-primary", () => {
			intent("pro.get", {sync: true}).catch(console.error);
			ask.close();
		});
		ask.addEventListener("qp-secondary", () => ask.close());
		ask.addEventListener("qp-closed", () => ask.remove(), {once: true});
		ask.open();
	};
	const get = h("qp-primary-button", {label: t("desk.menu.pro.get"), hidden: true, "on:click": getPro});
	const manage = h("qp-secondary-button", {label: t("desk.menu.pro.manage"), hidden: true,
		"on:click": () => intent("pro.manage").catch(console.error)});
	const pair = h("qp-secondary-button", {label: t("desk.menu.phones.pair"), "on:click": pairWindow});
	const comes = words(t("desk.menu.pro.comes"), "body");
	view.body.append(state,
		h("qp-section-label", {label: t("desk.menu.pro.adds")}),
		h("qp-card", {class: "qp-stack-small"}, words(t("desk.menu.pro.weeks"), "body"),
			words(t("pro.has.charts"), "body"), words(t("pro.has.path"), "body"), words(t("pro.has.cards"), "body")),
		comes, failed, get, manage, pair);
	watch(view, "menu.pro", (p) => {
		latest = p;
		const on = p.state === "on";
		if (on && buying) feedback("unlockUncommon");
		if (on) buying = false;
		// On (from questpump.com, or through a phone's store), or the paired phone's Pro not confirmed here
		state.hidden = !on && p.state !== "refused";
		state.textContent = !on ? t("desk.menu.pro.refused") :
			p.from === "web" ? t("menu.pro.from.web") : t("desk.menu.pro.on");
		get.hidden = on || !p.get;
		get.state = p.busy ? "busy" : "normal";
		manage.hidden = p.from !== "web";
		manage.busy = p.busy;
		pair.hidden = p.state !== null;
		// where Pro comes from while the computer offers no Get Pro
		comes.hidden = on || p.get;
		say(failed, p.failed ? t("menu.sync.unanswered") : null, "error");
	});
	intent("pro.look").catch(console.error);
	view.primary = () => [get, manage, pair].find((b) => !b.hidden) ?? null;
	return view;
});

register("desk.menu.does", () => {
	const view = page({heading: t("desk.menu.does.title")});
	const games = words("", "body");
	const kept = h("qp-card", {rows: true});
	const gsi = words("", "body");
	const lan = words("", "body");
	const windows = get("shell")?.platform === "windows";
	// the turned-on games by how each is heard (slice 48), each line while one is on; while Notice games I play is off,
	// the Steam games turned on, which Steam's own records still tell (by this system's own words)
	const heard = {
		client: {key: "desk.menu.does.client", line: words("", "body")},
		logs: {key: "desk.menu.does.logs", line: words("", "body")},
		shared: {key: "desk.menu.does.shared", line: words("", "body")},
		saves: {key: "desk.menu.does.saves", line: words("", "body")},
		screen: {key: "desk.menu.does.screen", line: words("", "body")},
		udp: {key: "desk.menu.does.udp", line: words("", "body")},
		steam: {key: windows ? "desk.menu.does.steam.windows" : "desk.menu.does.steam", line: words("", "body")},
	};
	const drops = words(t("desk.menu.does.drops"), "body");
	// how it notices a game, from this system's own sources (slice 48), while Notice games I play is on
	const detect = words(t(windows ? "desk.menu.does.detect.windows" : "desk.menu.does.detect"), "body");
	const shown = (heading, path) => h("qp-row", {heading, meta: path, glyph: "mark_list", trailing: "shown"});
	view.body.append(
		h("qp-section-label", {label: t("desk.menu.does.watches")}),
		h("qp-card", {class: "qp-stack-small"}, words(t("desk.menu.does.game"), "body"), games, heard.client.line,
			heard.logs.line, heard.shared.line, heard.saves.line, heard.screen.line, detect, heard.steam.line,
			words(t("desk.menu.does.idle"), "body"),
			// work mode's reads (slice 67, helper/src-tauri/src/presence.rs), each only while it runs
			words(t("desk.menu.does.front"), "body"), words(t("desk.menu.does.calls"), "body"),
			words(t("desk.menu.does.fullscreen"), "body"),
			words(t("desk.menu.does.never"), "body")),
		h("qp-section-label", {label: t("desk.menu.does.keeps")}), kept,
		words(t("desk.menu.does.removes"), "meta"),
		h("qp-section-label", {label: t("desk.menu.does.listens")}),
		h("qp-card", {class: "qp-stack-small"}, lan, gsi, heard.udp.line, drops),
		h("qp-section-label", {label: t("desk.menu.does.sends")}),
		h("qp-card", {class: "qp-stack-small"}, words(t("desk.menu.does.feedback"), "body"),
			words(t("desk.menu.does.updates"), "body"), words(t("desk.menu.does.crash"), "body"),
			words(t("desk.menu.does.usage"), "body"), words(t("desk.menu.does.nothing"), "body")));
	watch(view, "menu.about", (a) => {
		const d = a.does;
		const list = (names) => new Intl.ListFormat(LOCALE, {type: "conjunction"}).format(names);
		games.textContent = d.games.length
			? t("desk.menu.does.games", {games: list(d.games)})
			: t("desk.menu.does.games.none");
		for (const [by, {key, line}] of Object.entries(heard)) {
			line.hidden = !d[by].length;
			if (d[by].length) line.textContent = t(key, {games: list(d[by])});
		}
		kept.replaceChildren(shown(t("desk.menu.does.data"), d.data),
			...(d.mirror ? [shown(t("desk.menu.does.mirror"), d.mirror)] : []),
			...d.files.map((path) => shown(t("desk.menu.does.file"), path)),
			...d.added.map((path) => shown(t("desk.menu.does.file.shared"), path)));
		// a port as it is written, never grouped by the locale's digits
		lan.textContent = t("desk.menu.does.lan", {port: String(d.lan)});
		gsi.hidden = d.gsi === null;
		if (d.gsi !== null) gsi.textContent = t("desk.menu.does.gsi", {port: String(d.gsi)});
		drops.hidden = !d.games.length;
		detect.hidden = !d.detect;
	});
	intent("about.look").catch(console.error);
	return view;
});
