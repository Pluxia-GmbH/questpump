// VaultCard: the weekly vault (2026-10-09; Stats.lua ns.Vault, the summary's va; iOS Kit/Card.swift VaultCard,
// Android's kit/Card.kt): a Card with the week's omen in caps ("Hare week", slice 54; Weekly vault until then) and when
// it resets beside it (the week's turn in the locale's short weekday and time; "Complete · resets …" once all three
// goals are reached), then its three goals in Stats.lua's order, each its label, its value over its goal (a goal of
// tiers, the embers' since slice 55, over its next tier, its tiers' pips under its name lit gold as each is reached)
// and a thin gold bar filled to it, the check glyph in the number's place once reached (its last tier); read as one
// element a goal ("Moves done, 12 of 40", Reached). `vault` is the hero topic's {omen (the omen's key; none where the
// engine did not answer), goals: [{key: moves | records | embers, value, need, tiers?}], reset (unix seconds)}.

import {t} from "../lib/copy.js";
import {LOCALE} from "../lib/format.js";
import {KitElement, define, h} from "./element.js";

const LABELS = {moves: "kit.vault.moves", records: "kit.vault.records", embers: "kit.vault.embers"};
/** The week's title by its omen (contract/library.json `omens`). */
const WEEKS = {hare: "kit.vault.week.hare", oak: "kit.vault.week.oak", fist: "kit.vault.week.fist",
	frog: "kit.vault.week.frog", bear: "kit.vault.week.bear"};
const WHEN = new Intl.DateTimeFormat(LOCALE, {weekday: "short", hour: "2-digit", minute: "2-digit"});

/** One goal: its label (a goal of tiers' pips under it), its value over its goal (the check once reached) and its
 * bar. */
function goalRow(goal) {
	const name = t(LABELS[goal.key] ?? "kit.vault.embers");
	const done = goal.value >= goal.need;
	const share = goal.need > 0 ? Math.min(1, goal.value / goal.need) : 0;
	const track = h("span", {class: "qp-vault-track"}, h("span", {class: "qp-vault-fill"}));
	track.style.setProperty("--qp-vault-share", String(share));
	const value = done ? h("qp-art", {name: "mark_check", class: "qp-gold"})
		: h("span", {class: "qp-type-meta", text: t("kit.vault.value", {value: goal.value, need: goal.need})});
	const row = h("div", {class: "qp-vault-goal", role: "group",
		"aria-label": t("kit.vault.label", {goal: name, value: goal.value, need: goal.need})
			+ (done ? `, ${t("kit.vault.reached")}` : "")},
	h("span", {class: "qp-vault-line", "aria-hidden": "true"}, h("span", {class: "qp-type-body", text: name}), value),
	goal.tiers ? h("qp-day-pips", {"aria-hidden": "true", have: goal.tiers.filter((n) => goal.value >= n).length,
		need: goal.tiers.length}) : null,
	track);
	return row;
}

class QpVaultCard extends KitElement {
	build() {
		this.week = h("span", {class: "qp-type-caps"});
		this.reset = h("span", {class: "qp-type-meta"});
		this.goals = h("div", {class: "qp-vault-goals"});
		this.append(h("qp-card", {}, h("div", {class: "qp-vault-head"}, this.week, this.reset), this.goals));
	}

	set vault(value) {
		this.data = value ?? null;
		if (this.built) this.update();
	}

	get vault() {
		return this.data ?? null;
	}

	update() {
		const vault = this.vault;
		this.hidden = !vault;
		if (!vault) return;
		this.week.textContent = WEEKS[vault.omen] ? t(WEEKS[vault.omen]) : "";
		const when = WHEN.format(new Date(vault.reset * 1000));
		const complete = vault.goals.every((g) => g.value >= g.need);
		this.reset.textContent = t(complete ? "kit.vault.complete" : "kit.vault.reset", {time: when});
		this.goals.replaceChildren(...vault.goals.map(goalRow));
	}
}

define("qp-vault-card", QpVaultCard, {});
