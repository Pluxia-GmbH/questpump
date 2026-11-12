// StreakDay: the streak's day on the Result (slice 72; the phones' pump.done), its moment before the reward rows, where
// the pump's first move saved today made the streak a day longer: the streak's count over its caps label, this week's
// WeekDots and, under them, the day against the next milestone ("Day 3 of 7"; past the last milestone the count
// alone). `day` is Rust's {from, days, week, today_index, goal}. Until `arrive()` it stands as the pump found it, the
// count at `from` and today's mark open; `arrive()` rolls the count to today's over `carry`, the digits popping over
// the roll's last half, and lights today's mark at the roll's end (the phones' StreakMoment), at once with `still` (a
// landed reveal, one landed while the count rolls too, a calm end, Reduce Motion). Read as one: `spoken()`, which the Result says with its rewards. Never a
// word about a streak that ended: one that starts again rolls from 0.

import {roll, settle} from "../lib/carry.js";
import {t} from "../lib/copy.js";
import {number} from "../lib/saga.js";
import {KitElement, define, h} from "./element.js";
import {weekDays} from "./qp-streak-board.js";

class QpStreakDay extends KitElement {
	build() {
		const hidden = {"aria-hidden": "true"};
		this.count = h("p", {class: "qp-type-num-large", ...hidden});
		this.dots = h("qp-week-dots", hidden);
		this.line = h("p", {class: "qp-type-meta", ...hidden});
		this.append(this.count, h("p", {class: "qp-type-caps", text: t("pump.home.streak.label"), ...hidden}),
			this.dots, this.line);
		this.setAttribute("role", "img");
	}

	set day(value) {
		this.data = value ?? null;
		this.arrived = false;
		this.lit = false;
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

	/** What a screen reader hears of it: the streak as it stands ("3-day streak"), then the day against the next
	 * milestone. */
	spoken() {
		const day = this.day;
		if (!day) return "";
		return [t("kit.fanfare.streak", {days: day.days}), this.goalWords()].filter(Boolean).join(", ");
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
		this.line.textContent = this.goalWords();
		this.line.hidden = !this.line.textContent;
		this.setAttribute("aria-label", this.spoken());
	}
}

define("qp-streak-day", QpStreakDay);
