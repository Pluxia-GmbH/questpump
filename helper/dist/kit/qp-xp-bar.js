// XPBar: the XP bar, 8 high (`hero`: 6, Hero's): the gold fill to into / need on its track with an inset shadow, the
// rested part past it in gold at 28 %, and its caption under it unless `bare`: "QP 12" (past the level cap the Paragon
// rank, "Paragon 3") and "775 / 1077" in meta. The fill grows over the xpFill motion (a jump under Reduce Motion). Read
// as one: "QP 12, 775 of 1077 XP" (or "Paragon 3, …"), with the rested XP while there is any.

import {t} from "../lib/copy.js";
import {KitElement, define, h} from "./element.js";

function label(level, into, need, rested, paragon) {
	if (paragon > 0) {
		return rested > 0 ? t("kit.xp.label.paragon.rested", {rank: paragon, into, need, rested})
			: t("kit.xp.label.paragon", {rank: paragon, into, need});
	}
	return rested > 0 ? t("kit.xp.label.rested", {level, into, need, rested}) : t("kit.xp.label", {level, into, need});
}

class QpXpBar extends KitElement {
	build() {
		this.rest = h("i", {class: "qp-xp-rested"});
		this.fill = h("i", {class: "qp-xp-fill"});
		this.levelWord = h("span", {class: "qp-type-meta"});
		this.numbers = h("span", {class: "qp-type-meta"});
		this.caption = h("div", {class: "qp-xp-caption", "aria-hidden": "true"}, this.levelWord, this.numbers);
		this.append(h("div", {class: "qp-xp-track", "aria-hidden": "true"}, this.rest, this.fill), this.caption);
		this.setAttribute("role", "img");
	}

	update() {
		const [level, into, need] = [this.level ?? 1, this.into ?? 0, this.need ?? 0];
		const [rested, paragon] = [this.rested ?? 0, this.paragon ?? 0];
		const share = need > 0 ? Math.min(1, Math.max(0, into / need)) : 1;
		const rest = need > 0 ? Math.min(1 - share, Math.max(0, rested / need)) : 0;
		this.style.setProperty("--qp-xp-share", String(share));
		this.style.setProperty("--qp-xp-rest", String(rest));
		this.levelWord.textContent = paragon > 0 ? t("kit.fanfare.paragon", {rank: paragon}) : t("kit.level", {level});
		this.numbers.textContent = t("kit.xp.progress", {into, need});
		this.caption.hidden = this.bare;
		this.setAttribute("aria-label", label(level, into, need, rested, paragon));
	}
}

define("qp-xp-bar", QpXpBar, {
	level: Number, into: Number, need: Number, rested: Number, paragon: Number, bare: Boolean, hero: Boolean,
});
