// MatchCard: the match-over card (docs/ui.md, "The match-over card"), the whole of its small window: the offer's title
// ("Good game!") with a quiet Later and the × beside it, its one line the game's name alone (every game's offer, a
// program's close too since 2026-10-28), the size as a Segmented (Light · Balanced · Extreme, as the phones' Start a
// pump; none while every size takes the same moves, `one_size`, Moves per match at Light) and the gold Start. A mouse's
// way: its window never takes the focus, so no key reaches it, and the same offer stands on Pump home and the Games
// tab's Now card for the keyboard and a screen reader. `offer` is the `card` topic's {id, name, tier, one_size, test,
// remind}; each new offer starts at its tier (1-3: the size the match gave by its minutes since slice 57,
// helper/src-tauri/src/desk/card.rs), Balanced for one without. Start fires qp-start with the size chosen (the tier
// where none shows); Later and the × fire qp-later. A reminder's card (`remind`: the streak reminder's {kind:
// "streak", days, game} or the sitting cue's {kind: "sitting"}; helper/src-tauri/src/desk/reminders.rs) shows its
// title and line in the phones' words and the gold Quick pump, no size.

import {t} from "../lib/copy.js";
import {KitElement, define, emit, h} from "./element.js";

/** The sizes, as the phones' Start a pump names them. */
const SIZES = [
	{value: "small", key: "pump.size.small"},
	{value: "normal", key: "pump.size.normal"},
	{value: "big", key: "pump.size.big"},
];
/** The size an offer's Segmented starts at: its tier's (1 Light, 2 Balanced, 3 Extreme), Balanced without one. */
const sizeOf = (offer) => SIZES[(offer?.tier ?? 2) - 1]?.value ?? SIZES[1].value;

/** A reminder's line: the sitting cue's, or the days one pump today makes (one pump starts a streak where none counts),
 * in the game or here while Hero shows the game's sheet. */
function remindLine(remind) {
	if (remind.kind !== "streak") return t("reminders.notify.sitting.body");
	if (remind.days > 1) {
		return t(remind.game ? "reminders.notify.streak.body.game" : "reminders.notify.streak.body", {days: remind.days});
	}
	return t(remind.game ? "reminders.notify.streak.start.game" : "reminders.notify.streak.start");
}

class QpMatchCard extends KitElement {
	build() {
		this.head = h("h1", {class: "qp-type-heading qp-match-title"});
		this.later = h("qp-quiet-button", {"on:click": () => emit(this, "qp-later")});
		this.closer = h("qp-round-button", {icon: "close", small: true, "on:click": () => emit(this, "qp-later")});
		this.line = h("p", {class: "qp-type-subtitle qp-match-line"});
		this.size = h("qp-segmented", {value: sizeOf(this.shown), "on:qp-change": (e) => {
			this.size.value = e.detail;
		}});
		this.start = h("qp-primary-button", {"on:click": () => emit(this, "qp-start", this.size.value)});
		this.append(h("div", {class: "qp-match-head"}, this.head, this.later, this.closer), this.line,
			h("div", {class: "qp-match-foot"}, this.size, this.start));
		this.shown ??= null;
	}

	set offer(value) {
		if (value?.id !== this.shown?.id && this.size) this.size.value = sizeOf(value);
		this.shown = value ?? null;
		if (this.built) this.update();
	}

	get offer() {
		return this.shown ?? null;
	}

	update() {
		const offer = this.shown;
		const remind = offer?.remind ?? null;
		this.head.textContent = !remind ? t("reminders.notify.game.title.match")
			: t(remind.kind === "streak" ? "reminders.notify.streak.title" : "reminders.notify.sitting.title");
		this.later.label = t("desk.card.later");
		this.size.label = t("pump.questdone.size");
		this.size.options = SIZES.map((s) => ({value: s.value, label: t(s.key)}));
		this.size.hidden = Boolean(remind) || Boolean(offer?.one_size);
		this.start.label = remind ? t("pump.quick") : t("pump.questdone.start");
		this.line.textContent = !offer ? "" : remind ? remindLine(remind) : offer.name;
	}
}

define("qp-match-card", QpMatchCard);
