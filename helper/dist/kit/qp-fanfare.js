// Fanfare: the banner bar above and mirrored below the words in goldHi with its soft gold glow, the bars sweeping out
// and the words fading up (Reduce Motion: still); no sound of its own. `kind`: quest (Quest complete, Start a pump's A
// quest), pump (Pump complete, any other app pump; 2026-10-21), level (QP 13), paragon (Paragon 3), moves (a catch-up's
// 3 moves done), streak (a 30-day streak, from 730 days on a whole year a 2-year streak), expedition (Timed pump
// complete, its `number` the minutes it took; slice 51), with its `number`. The Result says `spoken()` once ("You
// reached QP 13").

import {t} from "../lib/copy.js";
import {KitElement, define, h} from "./element.js";

/**
 * The whole years a streak of `days` is, from two (730, 1,095 …: "2-year streak"; the cores' `StreakBoard.years`,
 * 2026-10-09; the streak card's count, slice 45); null for one said in days ("1,000-day streak", and 365's).
 */
export function streakYears(days) {
	return days >= 730 && days % 365 === 0 ? days / 365 : null;
}

/** A streak milestone's words: "30-day streak", "1,000-day streak", "2-year streak" (the banner's and its row's). */
export function milestoneWords(days) {
	const y = streakYears(days);
	return y === null ? t("kit.fanfare.streak", {days}) : t("kit.fanfare.streak.years", {years: y});
}

/** The banner's words for a kind and its number. */
export function fanfareWords(kind, number) {
	switch (kind) {
		case "level": return t("kit.level", {level: number});
		case "paragon": return t("kit.fanfare.paragon", {rank: number});
		case "moves": return t("kit.fanfare.moves", {count: number});
		case "streak": return milestoneWords(number);
		case "expedition": return t("expedition.done.title");
		case "pump": return t("kit.fanfare.pump");
		default: return t("kit.fanfare.quest");
	}
}

class QpFanfare extends KitElement {
	build() {
		this.words = h("p", {class: "qp-type-fanfare"});
		this.append(h("qp-art", {name: "banner_bar", class: "qp-bar-top"}), this.words,
			h("qp-art", {name: "banner_bar", class: "qp-bar-bottom"}));
		this.setAttribute("aria-hidden", "true");
	}

	/** What a screen reader hears of it. */
	spoken() {
		switch (this.kind) {
			case "level": return t("kit.fanfare.spoken", {level: this.number});
			case "paragon": return t("kit.fanfare.paragon.spoken", {rank: this.number});
			case "expedition": return t("expedition.done.spoken", {minutes: this.number});
			case "streak": {
				const y = streakYears(this.number);
				return y === null ? t("kit.fanfare.streak.spoken", {days: this.number})
					: t("kit.fanfare.streak.years.spoken", {years: y});
			}
			default: return fanfareWords(this.kind, this.number);
		}
	}

	update() {
		this.words.textContent = fanfareWords(this.kind, this.number ?? 0);
	}
}

define("qp-fanfare", QpFanfare, {kind: String, number: Number});
