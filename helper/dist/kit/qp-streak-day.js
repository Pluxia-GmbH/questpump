// StreakDay: the streak's day on the Result (slice 72; the phones' pump.done), its moment before the reward rows, where
// the pump's first move saved today made the streak a day longer: the streak's count over its caps label, this week's
// WeekDots and, under them, the day against the next milestone ("Day 3 of 7"; past the last milestone the count
// alone). `day` is Rust's {from, days, week, today_index, goal}. Until `arrive()` it stands as the pump found it, the
// count at `from` and today's mark open; `arrive()` rolls the count to today's over `carry`, the digits popping over
// the roll's last half, and lights today's mark at the roll's end (the phones' StreakMoment), at once with `still` (a
// landed reveal, one landed while the count rolls too, a calm end, Reduce Motion). Read as one: `spoken()`, which the Result says with its rewards. Never a
// word about a streak that ended: one that starts again rolls from 0. Since slice 74 the day stands against the
// player's own goal while one is ahead (`goal`, Rust's `Board::against`), and on a day with no goal ahead of it
// (`steps`, Rust's `Board::goal_steps`: never once the player said None) the goals to choose from stand in the line's
// place, a ChoiceGroup of which none is chosen as it shows, under a caps label that names them (under the line on the
// day that reaches a goal, "Day 14 of 14"); a choice fires qp-goal with its days, and its owner sets `chosen` once it
// holds and `offer` as Rust sends the day's number and steps again.

import {roll, settle} from "../lib/carry.js";
import {t} from "../lib/copy.js";
import {number} from "../lib/saga.js";
import {KitElement, define, emit, h} from "./element.js";
import {weekDays} from "./qp-streak-board.js";

/** The moments built so far: each one's label takes an id of its own. */
let labels = 0;

class QpStreakDay extends KitElement {
	build() {
		const hidden = {"aria-hidden": "true"};
		this.count = h("p", {class: "qp-type-num-large", ...hidden});
		this.dots = h("qp-week-dots", hidden);
		this.line = h("p", {class: "qp-type-meta", ...hidden});
		// read as one, and beside it the choices, which are the player's to reach
		this.moment = h("div", {class: "qp-streak-moment", role: "img"}, this.count,
			h("p", {class: "qp-type-caps", text: t("pump.home.streak.label"), ...hidden}), this.dots, this.line);
		// the choices under their own caps label, which names the group
		labels += 1;
		const id = `qp-streak-goal-${labels}`; // copy-lint: not shown (an element's id)
		this.goalLabel = h("p", {class: "qp-type-caps", id, text: t("pump.streak.goal.pick"), hidden: true});
		this.goals = h("qp-choice-group", {named: id, hidden: true,
			"on:qp-change": (e) => emit(this, "qp-goal", Number(e.detail))});
		this.append(this.moment, this.goalLabel, this.goals);
	}

	/** The day's number and the goals on offer as they stand now (Rust reads them live: a goal that arrives from another
	 * device takes the choices away), the moment itself as it stands. */
	set offer(value) {
		if (!this.data) return;
		this.data = {...this.data, goal: value?.goal ?? null, steps: value?.steps ?? []};
		if (this.built) this.update();
	}

	/** The goal picked among the day's steps (its days), none while none was. */
	set chosen(value) {
		this.picked = value ?? null;
		if (this.built) this.update();
	}

	get chosen() {
		const steps = this.day?.steps ?? [];
		return steps.includes(this.picked) ? this.picked : null;
	}

	set day(value) {
		this.data = value ?? null;
		this.arrived = false;
		this.lit = false;
		this.picked = null;
		if (this.built) this.update();
	}

	get day() {
		return this.data ?? null;
	}

	/** The day against the next milestone, none past the last. */
	goalWords() {
		const day = this.day;
		return day?.goal ? t("pump.streak.day.of", {day: day.days, goal: day.goal}) : "";
	}

	/** The line under the week: the day against its number, but none where the goals to choose from stand in its
	 * place; on the day that reaches a goal it says so over them. */
	lineWords() {
		const day = this.day;
		return !day?.steps?.length || day.goal === day.days ? this.goalWords() : "";
	}

	/** What a screen reader hears of it: the streak as it stands ("3-day streak"), then the day against the next
	 * milestone. */
	spoken() {
		const day = this.day;
		if (!day) return "";
		return [t("kit.fanfare.streak", {days: day.days}), this.lineWords()].filter(Boolean).join(", ");
	}

	/** The day arrives, once: the count rolls from yesterday's number to today's, and at the roll's end today's mark
	 * lights; `still` stands both at their end, a roll on its way too. */
	arrive(still) {
		const day = this.day;
		if (!day || this.lit || (this.arrived && !still)) return;
		this.arrived = true;
		const land = () => {
			this.lit = true;
			this.update();
		};
		if (still) {
			settle(this.count);
			return land();
		}
		roll(this.count, day.from, day.days, "carry", {
			landing: true,
			done: () => {
				land();
				this.dots.light([day.today_index]);
			},
		});
	}

	update() {
		const day = this.day;
		if (!day) return;
		// while it rolls, the count is the roll's
		if (!this.arrived || this.lit) this.count.textContent = number(this.lit ? day.days : day.from);
		const week = this.lit ? day.week : day.week.map((mark, i) => (i === day.today_index ? "open" : mark));
		this.dots.days = weekDays(week, day.today_index);
		const steps = day.steps ?? [];
		this.goals.hidden = !steps.length;
		this.goalLabel.hidden = !steps.length;
		this.goals.options = steps.map((days) => ({value: String(days), label: t("pump.streak.goal.days", {days})}));
		const chosen = this.chosen;
		this.goals.value = chosen === null ? null : String(chosen);
		this.line.textContent = this.lineWords();
		this.line.hidden = !this.line.textContent;
		this.moment.setAttribute("aria-label", this.spoken());
	}
}

define("qp-streak-day", QpStreakDay);
