// An expedition's parts of the pump on the computer (docs/plan.md slice 51; the phones' expedition.round and
// expedition.done): the breather between two rounds (desk.expedition.round: the ExpeditionBar over it, Rest over the
// TimerRing's 30 s, the next round's moves on their tiles once it is composed, Start now and End the timed pump), and
// the expedition's Result's own parts around the pump's Result (desk.expedition.done: the standard-bearer, the line
// under the Fanfare or the calm title of one ended part way, the MuscleMap's day variant, Share and Done, and the
// Share window opening by itself on the expedition card as the reveal ends, its close landing on Pump home). The
// `pump` topic's `expedition` (questpump-desk/src/pump/expedition.rs `TripView`) holds what they draw.

import {get, intent} from "../../lib/bridge.js";
import {t} from "../../lib/copy.js";
import {announce} from "../../lib/focus.js";
import {expeditionCard, openShare} from "../../lib/share.js";
import {goTop} from "../../lib/router.js";
import {h} from "../../kit/element.js";
import {placeName} from "../places/words.js";

/** The standard-bearer (contract/figures/figures.json `extras`). */
const BANNER = "streak_banner"; // copy-lint: not shown (a figure's name in figures.json)

/** The expeditions whose Share window opened by itself, once each (a stage drawn again never opens it twice). */
const shared = new Set();

/** An intent's failure goes to the console (Rust's handler logged why). */
function heard(promise) {
	promise.catch(console.error);
}

/** The breather (desk.expedition.round): the bar (the session's), Rest over the ring counting down the 30 s, then once
 * the next round is composed its caps (Round 4 next, Cool-down next) over its moves, each a MoveTile with its name
 * under it (Tokens.Surface.ExpeditionRest), Start now (Space or Enter) and End the timed pump. Its ring draws from
 * Rust's instant; Rust ends it. */
export function rest(pump) {
	const r = pump.expedition.rest;
	const ring = h("qp-timer-ring", {class: "qp-expedition-ring"});
	const next = r.next === "cooldown" ? t("expedition.next.cooldown")
		: r.next === "round" ? t("expedition.next.round", {round: r.round ?? 1}) : null;
	const tiles = r.next ? pump.moves.map((m) => h("div", {class: "qp-expedition-tile"},
		h("qp-move-tile", {size: 52, move: m.key, name: m.name, quality: m.quality}),
		h("span", {class: "qp-type-meta", "aria-hidden": "true", text: m.name}))) : [];
	const start = h("qp-secondary-button", {label: t("expedition.round.start"),
		"on:click": () => heard(intent("expedition.next"))});
	const stop = h("qp-quiet-button", {label: t("expedition.stop"), "on:click": () => heard(intent("expedition.stop"))});
	const body = [h("h1", {class: "qp-type-title qp-pump-centre", text: t("expedition.rest")}), ring,
		next ? h("p", {class: "qp-type-caps qp-pump-centre", text: next}) : null,
		tiles.length ? h("div", {class: "qp-expedition-tiles"}, ...tiles) : null].filter(Boolean);
	announce(t("expedition.rest.spoken", {seconds: r.seconds}));
	return {
		body,
		foot: [start, stop],
		primary: () => start,
		frame(now) {
			ring.mode = {kind: "timed", total: r.seconds, left: Math.max(0, (r.until - now) / 1000)};
			return now < r.until;
		},
	};
}

/** The muscles' names, for the map's label: the Saga's words. */
function musclesSaid(done) {
	const names = new Map((get("menu.setup")?.words?.muscles ?? []).map((m) => [m.id, m.name]));
	const list = (ids) => ids.map((id) => names.get(id)).filter(Boolean).join(", ");
	return [done.most.length ? `${t("saga.muscles.most")}: ${list(done.most)}` : "",
		done.some.length ? `${t("saga.muscles.some")}: ${list(done.some)}` : ""].filter(Boolean).join(". ");
}

/** The timed pump card of the Result's expedition: its place's words and template (its scene's key; Home's id is its
 * template's), and the headline's facts. */
function card(done) {
	return expeditionCard({minutes: done.minutes, place: placeName(done.place), scene: done.place.template ?? done.place.id,
		moves: done.moves, records: done.records, colour: done.colour, record: done.record, longest: done.longest});
}

/** The expedition's own parts of its Result (`pump.expedition.done`): over the Result the standard-bearer and, on one
 * ended part way, its calm title (N rounds done); under the Fanfare the line (20 min · Hotel room); after the reward
 * rows the MuscleMap's day variant with its legend; Share (the primary) and Done. `opened()` opens the Share window on
 * the expedition card once (as the reveal ends, or at once on one ended part way), its close landing on Pump home. */
export function tripResult(pump) {
	const done = pump.expedition.done;
	const figure = h("qp-figure-stage", {move: BANNER, still: true, colour: get("book")?.tint ?? null,
		class: "qp-expedition-figure"});
	const early = done.early ? h("h1", {class: "qp-type-title qp-pump-centre",
		text: t("expedition.done.early", {count: done.rounds})}) : null;
	const line = h("p", {class: "qp-type-meta qp-pump-centre",
		text: t("expedition.done.line", {minutes: done.minutes, place: placeName(done.place)})});
	const map = h("qp-muscle-map", {mode: "day", figure: get("book")?.figure ?? "male",
		label: musclesSaid(done) || t("saga.muscles")});
	map.most = done.most;
	map.some = done.some;
	const legend = h("div", {class: "qp-saga-legend qp-type-meta", "aria-hidden": "true"},
		h("span", {}, h("i", {class: "qp-swatch-most"}), t("saga.muscles.most")),
		h("span", {}, h("i", {class: "qp-swatch-some"}), t("saga.muscles.some")));
	const muscles = done.most.length || done.some.length
		? h("qp-card", {class: "qp-expedition-muscles"}, h("span", {class: "qp-type-caps", text: t("saga.muscles")}), map,
			legend) : null;
	const share = () => {
		shared.add(pump.id);
		const win = openShare(card(done));
		// closing it is one tap and lands on Pump home
		win.addEventListener("qp-closed", () => {
			heard(intent("pump.continue").then(() => goTop("pump")));
		}, {once: true});
	};
	const shareButton = h("qp-primary-button", {label: t("kit.share"), glyph: "mark_share", "on:click": share});
	// Done lands on Pump home too, past the setup page the expedition started from
	const doneButton = h("qp-quiet-button", {label: t("common.done"),
		"on:click": () => heard(intent("pump.continue").then(() => goTop("pump")))});
	return {
		head: [figure, early].filter(Boolean),
		line: early ? null : line,
		tail: [early ? line : null, muscles].filter(Boolean),
		foot: [shareButton, doneButton],
		primary: shareButton,
		/** The Share window by itself, once. */
		opened() {
			if (shared.has(pump.id) || get("pump")?.pump?.id !== pump.id) return;
			map.light?.();
			share();
		},
	};
}
