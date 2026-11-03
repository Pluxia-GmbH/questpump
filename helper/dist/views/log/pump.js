// A pump's page in the Log (the phones' LogPumpPage), pushed from its row: its title in the DetailBar; a card with when
// it was and where it came from ("Today · 14:32 · on a computer"), how a game pump with no move done ended, how long it
// took and the XP it paid, its marks as chips (the heart-rate check, a record, the game's felt-unwell word on an app
// pump stopped for feeling unwell); How the heart-rate check works under it while it shows the check; then each move
// with its tile and what was done: the typed count (with the weight a loaded move's set was done with, "11 kg × 16"),
// a counted set's count or Not counted with "form slipped" or "not all out" after either where it was answered so (as
// the game's Log words it), else the seconds as a clock (a hold's time held), read with its unit. The page shows the
// Saga's newest version of the pump by its id (today's chapter's pumps, or the open chapter's), and the last one it
// showed once neither holds it (the game day turned at 04:00 under it), until Back.

import {t} from "../../lib/copy.js";
import {clock, LOCALE} from "../../lib/format.js";
import {push, register} from "../../lib/router.js";
import {h} from "../../kit/element.js";
import {page, watch} from "../page.js";
import {HEART} from "../menu/about.js";
import {lifted} from "../pump/words.js";
import {PUMP, dayName, length, source, title, undone} from "./log.js";

const TIME = new Intl.DateTimeFormat(LOCALE, {timeStyle: "short"});

/** A counted set's answer's words beside its count (`answer`: 1 Form slipped, 2026-10-19; 2 Not all out, slice 53). */
const SAID = {1: "log.result.slipped", 2: "log.result.notallout"};

/** What a move did, shown and spoken with its unit: the typed count (with its weight, a loaded move's in each hand
 * where `each`), a counted set's count or Not counted (with "form slipped" or "not all out" where it was saved so),
 * else its seconds as a clock; none for a rep move done without a count. */
function done(m, each) {
	if (m.counted) {
		const has = m.reps !== undefined && m.reps !== null;
		const shown = has ? new Intl.NumberFormat(LOCALE).format(m.reps) : t("pump.count.none");
		const spoken = has ? t("log.reps", {count: m.reps}) : shown;
		const said = SAID[m.answer];
		if (!said) return {shown, spoken};
		return {shown: t(said, {value: shown}), spoken: t(said, {value: spoken})};
	}
	if (m.reps !== undefined && m.reps !== null) {
		if (m.load != null) {
			const weighed = lifted(m.load, m.reps, each);
			return {shown: weighed, spoken: weighed};
		}
		return {shown: new Intl.NumberFormat(LOCALE).format(m.reps), spoken: t("log.reps", {count: m.reps})};
	}
	if (m.seconds <= 0) return null;
	const time = t("kit.objective.spoken.time", {seconds: m.seconds});
	return {shown: clock(m.seconds), spoken: m.hold ? t("log.held", {time}) : time};
}

register(PUMP, ({id}) => {
	const view = page({heading: ""});
	let names = new Map();
	let unwell = "";
	let log = null;
	let last = null;
	const draw = () => {
		const e = [log?.today, log?.open].filter(Boolean).flatMap((d) => d.entries).find((x) => x.id === id) ?? last;
		if (!e) return;
		last = e;
		view.bar.heading = title(e);
		// the game day it began on: a pump across the day's turn is listed under each day it holds a move on
		const when = [dayName(e.day, e.ago), TIME.format(new Date(e.at * 1000)), source(e).join(" · ")].filter(Boolean)
			.join(" · ");
		const facts = [undone(e), e.seconds > 0 ? length(e.seconds) : null,
			e.xp ? t("pump.reward.xp", {xp: e.xp}) : null].filter(Boolean).join(" · ");
		const chips = [];
		if (e.proven) chips.push(h("qp-chip", {label: t("log.proven"), state: "ok"}));
		if (e.moves.some((m) => m.record)) chips.push(h("qp-chip", {label: t("log.pump.record"), state: "record"}));
		if (e.unwell) chips.push(h("qp-chip", {label: unwell, state: "off"}));
		const card = h("qp-card", {class: "qp-pump-card"}, h("p", {class: "qp-type-body-strong", text: when}),
			facts ? h("p", {class: "qp-type-meta", text: facts}) : "",
			chips.length ? h("div", {class: "qp-chips"}, ...chips) : "");
		const parts = [card];
		if (e.proven) {
			parts.push(h("qp-quiet-button", {label: t("menu.heart.row"), "data-focus-key": "heart",
				"on:click": () => push(HEART)}));
		}
		if (e.moves.length) {
			parts.push(h("qp-section-label", {label: t("log.pump.moves")}), h("qp-card", {rows: true},
				...e.moves.map((m) => {
					const move = names.get(m.move);
					const what = done(m, move?.load === "each");
					const tile = h("qp-move-tile", {size: 44, move: move?.key});
					return h("div", {class: "qp-pump-move", role: "group",
						"aria-label": [move?.name, m.record ? t("log.record") : null, what?.spoken].filter(Boolean).join(", ")},
					tile, h("div", {class: "qp-step-words", "aria-hidden": "true"},
						h("span", {class: "qp-type-body-strong", text: move?.name ?? ""}),
						m.record ? h("span", {class: "qp-type-meta", text: t("log.record")}) : ""),
					what ? h("span", {class: "qp-type-num-row", text: what.shown, "aria-hidden": "true"}) : "");
				})));
		}
		view.body.replaceChildren(...parts);
	};
	watch(view, "book.words", (words) => {
		names = new Map(words.moves.map((m) => [m.move, m]));
		unwell = words.unwell;
		draw();
	});
	watch(view, "log", (l) => {
		log = l;
		draw();
	});
	return view;
});
