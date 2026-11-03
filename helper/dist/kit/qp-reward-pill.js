// RewardPill: what the move up next earns on its Ready view, the kit's Chip with its glyph before its words, as the
// phones' RewardPill draws theirs (2026-10-24): the move's XP (`kind="xp"`, `amount`), a record, a stat a record would
// raise and its condition (`kind="stat"`: "+1 Strength if you beat 12", `amount`, `stat` its name, `record` the count,
// `glyph` its art).

import {t} from "../lib/copy.js";
import {KitElement, define, h} from "./element.js";

const GLYPHS = {xp: "reward_xp", record: "reward_record"};

class QpRewardPill extends KitElement {
	build() {
		this.chip = h("qp-chip", {state: "plain"});
		this.append(this.chip);
	}

	update() {
		const kind = this.kind ?? "xp";
		this.chip.glyph = kind === "stat" ? this.glyph : GLYPHS[kind];
		this.chip.label = kind === "xp" ? t("kit.pill.xp", {amount: this.amount ?? 0})
			: kind === "record" ? t("kit.pill.record")
				: t("kit.reward.stat.beat", {amount: this.amount ?? 0, stat: this.stat ?? "", record: this.record ?? ""});
	}
}

define("qp-reward-pill", QpRewardPill, {kind: String, amount: Number, stat: String, record: String, glyph: String});
