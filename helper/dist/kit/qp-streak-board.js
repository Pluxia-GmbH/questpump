// StreakBoard: Pump home's streak block (slice 32): the streak's `days` as the screen's hero number over its caps
// label, this week's WeekDots (`week`: seven marks from Monday, the game week's first day, each "done", "rested",
// "resting" or "open"; `today-index` 0-6; read as the phones' WeekDots read it: the days pumped, then the rest token's
// days and the resting ones, or "No moves yet this week"), and one line by the rule the phones' widgets share: while
// felt unwell rests the pumps and today has no pump (`resting`), "Resting today keeps your streak." with a streak or
// "Pumps rest today." without; else Today: done or Today: open (`today`), at 0 days too, with the rest `tokens` held
// on a streak. Never words of loss. Read as one element. With `share` (slice 45: from a 7-day streak) a round Share
// beside the count, read as "Share your streak", which fires qp-share (the Share window on the streak card as it
// stands). Since slice 51 the whole board is a button that fires qp-open (the streak page), read as the board's words
// with its hint, Open your streak; the round Share stays its own button over it. `carry(from)` (slice 72, a reward
// carried home) moves the board once from what it last showed, {days, week}: the count rolls to its new number and the
// week's new marks light, the days a rest token kept one after another before the day done.

import {roll, settle} from "../lib/carry.js";
import {t} from "../lib/copy.js";
import {LOCALE} from "../lib/format.js";
import {KitElement, define, emit, h} from "./element.js";

/** Monday to Sunday's initials and names in the copy's English (LOCALE; 2026-10-05 is a Monday). */
const weekdays = (weekday) => Array.from({length: 7}, (_, i) =>
	new Intl.DateTimeFormat(LOCALE, {weekday, timeZone: "UTC"}).format(Date.UTC(2026, 9, 5 + i)));
const INITIALS = weekdays("narrow");
const NAMES = weekdays("long");
const DAYS = new Intl.ListFormat(LOCALE, {type: "conjunction"});

/** A game week's marks (`week`: seven of "done", "rested", "resting" or "open" from Monday; `todayIndex` 0-6) as
 * WeekDots draws its days. */
export function weekDays(week, todayIndex) {
	return week.map((mark, i) => ({initial: INITIALS[i], done: mark === "done", rested: mark === "rested",
		resting: mark === "resting", today: i === todayIndex, future: todayIndex !== null && i > todayIndex}));
}

/** The days of `week` that came since `before` (the week a surface last showed), in the order they light: the days a
 * rest token kept, then the days done. */
export function newMarks(before, week) {
	const came = (mark) => week.flatMap((now, i) => (now === mark && before[i] !== mark ? [i] : []));
	return [...came("rested"), ...came("done")];
}

class QpStreakBoard extends KitElement {
	build() {
		// the words the board's button reads, heard once through it
		const hidden = {"aria-hidden": "true"};
		this.count = h("p", {class: "qp-type-num-hero", ...hidden});
		this.shareButton = h("qp-round-button", {icon: "share", spoken: t("pump.home.streak.share"),
			"on:click": () => emit(this, "qp-share")});
		this.dots = h("qp-week-dots", hidden);
		this.line = h("p", {class: "qp-type-meta", ...hidden});
		this.open = h("button", {type: "button", class: "qp-streak-open", "aria-description": t("pump.streak.open"),
			"on:click": () => emit(this, "qp-open")});
		this.append(this.open, h("div", {class: "qp-streak-count"}, this.count, this.shareButton),
			h("p", {class: "qp-type-caps", text: t("pump.home.streak.label"), ...hidden}), this.dots, this.line);
		this.weekData ??= [];
	}

	set week(value) {
		this.weekData = value ?? [];
		if (this.built) this.update();
	}

	get week() {
		return this.weekData ?? [];
	}

	/** Moves the board once from what it last showed (`from`: {days, week}) to what it is now. */
	carry(from) {
		roll(this.count, from.days, this.days ?? 0, "carry");
		this.dots.light(newMarks(from.week, this.week));
	}

	/** A count still on its way stands at its end (the page left, a pump's cover over it). */
	stand() {
		settle(this.count);
	}

	words() {
		const days = this.days ?? 0;
		const line = this.resting && !this.today ? t(days > 0 ? "pump.home.streak.resting" : "pump.home.streak.resting.none")
			: t(this.today ? "pump.home.streak.today.done" : "pump.home.streak.today.open");
		const tokens = this.tokens ?? 0;
		const held = days > 0 && !(this.resting && !this.today) && tokens > 0;
		return held ? `${line} · ${t("pump.home.streak.tokens", {count: tokens})}` : line;
	}

	update() {
		this.count.textContent = new Intl.NumberFormat(LOCALE).format(this.days ?? 0);
		this.shareButton.hidden = !this.share;
		const marked = (mark) => NAMES.filter((_, i) => this.week[i] === mark);
		const [pumped, rested, resting] = ["done", "rested", "resting"].map(marked);
		const label = [pumped.length ? DAYS.format(pumped) : t("log.week.none")];
		if (rested.length) label.push(t("kit.weekdots.rested", {days: DAYS.format(rested)}));
		if (resting.length) label.push(t("kit.weekdots.resting", {days: DAYS.format(resting)}));
		this.dots.label = label.join(". ");
		this.dots.days = weekDays(this.week, this.todayIndex);
		this.line.textContent = this.words();
		this.open.setAttribute("aria-label", `${t("pump.home.streak.line")} ${this.count.textContent} ${
			t("pump.home.streak.label")}. ${this.dots.label}. ${this.line.textContent}`);
	}
}

define("qp-streak-board", QpStreakBoard, {days: Number, tokens: Number, today: Boolean, resting: Boolean,
	"today-index": Number, share: Boolean});
