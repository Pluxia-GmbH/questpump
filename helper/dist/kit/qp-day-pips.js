// DayPips: an unlock's days as round pips (a ladder's page's next step): `need` pips, the first `have` lit gold, the rest
// text3; read as "1 of 2 days" (one day's form "0 of 1 day"). `brought` (a Result's move row, 2026-10-29): the last lit
// pip is the day this pump brought, which lights as its row arrives (kit.css).

import {t} from "../lib/copy.js";
import {LOCALE} from "../lib/format.js";
import {KitElement, define, h} from "./element.js";

const NUMBER = new Intl.NumberFormat(LOCALE);

class QpDayPips extends KitElement {
	update() {
		const need = Math.max(0, this.need ?? 0);
		const have = Math.max(0, Math.min(need, this.have ?? 0));
		this.setAttribute("role", "img");
		// the plural on the day count, the days in as words: the phones' catalogs cannot number a plural with two integers
		this.setAttribute("aria-label", t("kit.days.label", {need, have: NUMBER.format(have)}));
		const lit = (i) => (this.brought && i === have - 1 ? "qp-lit qp-brought" : "qp-lit");
		this.replaceChildren(...Array.from({length: need}, (_, i) => h("i", {class: i < have ? lit(i) : null})));
	}
}

define("qp-day-pips", QpDayPips, {have: Number, need: Number, brought: Boolean});
