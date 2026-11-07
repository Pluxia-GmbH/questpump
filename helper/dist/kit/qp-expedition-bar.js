// ExpeditionBar (slice 51; contract/design/screens.json's component, Tokens.Surface.ExpeditionBar): an expedition's
// progress over each round's pump screens and the breather. One row of segments, a short one for the warm-up and the
// cool-down (shortFraction of a round's), one for each round planned: done goldLo, the one in play gold with StepsBar's
// glow, the rest the track. Over it the stage in caps (Warm-up, Round 3 of 5,
// Cool-down) and the clock mark with the minutes left (12:40 left: the length less the time since it began, ticking
// each second while it shows; nothing once none is left). `trip` is the `pump` topic's `expedition` (questpump-desk/src/pump/
// expedition.rs `TripView`). Read as one element: "Round 3 of 5, 12 minutes left".

import {t} from "../lib/copy.js";
import {clock} from "../lib/format.js";
import {KitElement, define, h} from "./element.js";

/** The stage's words in caps: the warm-up, a round of the rounds planned, the cool-down. */
export function stageWords(trip) {
	switch (trip.stage) {
		case "warmup": return t("expedition.stage.warmup");
		case "cooldown": return t("expedition.stage.cooldown");
		default: return t("expedition.bar.round", {round: trip.round ?? 1, count: Math.max(trip.count, trip.round ?? 1)});
	}
}

class QpExpeditionBar extends KitElement {
	build() {
		this.stage = h("span", {class: "qp-type-caps"});
		this.left = h("span", {class: "qp-type-num-row"});
		this.clockMark = h("qp-art", {name: "mark_clock"});
		this.leftBox = h("span", {class: "qp-expedition-left"}, this.clockMark, this.left);
		this.segments = h("div", {class: "qp-expedition-segments"});
		this.setAttribute("role", "img");
		this.append(h("div", {class: "qp-expedition-line"}, this.stage, this.leftBox), this.segments);
		this.tripData ??= null;
	}

	connectedCallback() {
		super.connectedCallback();
		this.timer = setInterval(() => this.tick(), 1000);
	}

	disconnectedCallback() {
		clearInterval(this.timer);
	}

	set trip(value) {
		this.tripData = value;
		if (this.built) this.update();
	}

	get trip() {
		return this.tripData;
	}

	/** The minutes left, each second. */
	tick() {
		const trip = this.tripData;
		if (!trip) return;
		const left = Math.max(0, (trip.ends - Date.now()) / 1000);
		this.leftBox.hidden = left < 1;
		const words = t("expedition.bar.left", {time: clock(left)});
		if (this.left.textContent !== words) this.left.textContent = words;
		this.setAttribute("aria-label", [this.stage.textContent,
			left >= 60 ? t("expedition.bar.spoken", {minutes: Math.floor(left / 60)}) : null].filter(Boolean).join(", "));
	}

	update() {
		const trip = this.tripData;
		if (!trip) return;
		this.stage.textContent = stageWords(trip);
		this.segments.replaceChildren(...trip.legs.map((leg) => h("i", {class: [leg.short ? "qp-short" : null,
			leg.state === "next" ? null : `qp-${leg.state}`].filter(Boolean).join(" ") || null})));
		this.tick();
	}
}

define("qp-expedition-bar", QpExpeditionBar);
