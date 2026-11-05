// RewardRow: a result's row (the Result's reveal, iOS Kit/Result.swift RewardRow): the move's tile (44) in its
// quality's edge, the reward's words in bodyStrong, under them the move's name in its tone, or an unlock's ladder and
// step with its QualityWord, and a record's glyph in the move's tone; a row with no move (a star, a stat, a title, a
// day's line, a streak milestone) leads with its glyph in the tile's place. `reward` is {kind, ...}: record {value},
// first_record {value, next}, enough {value}, placed {value, record}, star {count}, stat {amount, stat, glyph}, points
// {count}, title {title}, unlock {ladder, step, quality, word, rekindled, way}, day {line: welcome {xp} | streak {days}
// | streak_new | rested {count} | token}, milestone {days}, vault {goal: moves | records | embers, count}, and since
// slice 42 mastered {ladder, step} (the step's tile with its seal), check {move} (a stretch check passed, the move it
// opens, open after the set), check_climb {ladder, step} (a climb's check passed, the climb open; slice 42's fix
// round), check_held {line} (a check passed whose move or climb another wait still holds: that wait in its own words,
// filled in by the pump's words), check_placed {ladder, step} (its stretch ladder lifted), check_waits, best_new
// {count} (a counted set's new best clean count, a record), best {count} (the best that stands), slipped, clean_day
// {have, need, step} (the cardio climb's day it brought), since slice 52 full_move {have, need, step} (its full move),
// since slice 53 not_all_out, return {back, step} (a measure back from a break: the step it holds or goes back to) and
// climb_waits (a proving window that did not pass), since slice 55 haul_day {time, have, need, next} (a Long Haul's day
// toward its next length, the times in words), since 2026-10-29 moved {amount, tag, was, lines, steps} (a move the
// pump did, on its tile: its name, what it did in numbers, its "New record" or "First record" tag in gold with the
// record glyph and the record it passed, its own lines in meta, and each step it counts toward with its words and its
// days as pips, {words, have, need, brought}, whose last pip lights as the row arrives where this pump `brought` it;
// said quietly with no shine and no tag), cardio {have, need} (Cardio today with its pips) and stats {stats} (the stats
// raised, one chip each), since slice 65 xp {xp} (the XP one move's save earned, on its pour), the quiet ones in
// meta; `move` is {name, key, quality}, its tile the key's pose. An unlock's `way` (2026-10-09): "climb",
// "opened" (an orange or gold step its climb only opened: "Opened: …" said quietly in
// meta, its ladder and step under it with no QualityWord, since a later set earns the colour; 2026-10-22) or "earned"
// ("Earned: …", in its tone). A Rare or higher unlock that is not Rekindled nor only opened with `share` carries the
// round Share button (qp-share), and so does a streak milestone's with `share` (slice 45: the Share window on the
// streak card). Read as one, as earned where it is a reward; the reveal says `spoken()`. The caller adds `shown` as
// the reveal brings it in (kit.css: its tile shines once).

import {t} from "../lib/copy.js";
import {KitElement, define, emit, h, qualityName} from "./element.js";
import {milestoneWords} from "./qp-fanfare.js";

/** The rewards a screen reader hears as earned: never a day's line nor a placement under the record that stays. */
const EARNED = new Set(["record", "first_record", "enough", "star", "stat", "points", "title", "unlock", "milestone",
	"vault", "mastered", "check", "check_climb", "check_placed", "best_new", "stats", "xp"]);
const RECORDS = new Set(["record", "first_record", "enough", "placed", "best_new"]);
/** The rows said quietly, in meta (slice 42): a check that waits or whose opening still waits, a best that stands, a
 * slipped form, a day it brought. */
const QUIET = new Set(["check_waits", "check_held", "best", "slipped", "not_all_out", "clean_day", "full_move",
	"haul_day", "return", "climb_waits"]);

function dayWords(day) {
	switch (day.line) {
		case "welcome": return t("pump.reward.welcome", {xp: day.xp});
		case "streak": return t("pump.reward.streak", {days: day.days});
		case "streak_new": return t("pump.reward.streak.new");
		case "rested": return t("pump.reward.rested", {count: day.count});
		default: return t("pump.reward.token");
	}
}

/** Said quietly in meta, with no shine: a QUIET row, a Rekindled or only opened step, a move's row with no tag. */
function quietRow(reward) {
	return QUIET.has(reward.kind) || (reward.kind === "unlock" && (reward.rekindled || opened(reward)))
		|| (reward.kind === "moved" && !reward.tag);
}

/** A moved row's tag in words. */
function tagWords(tag) {
	return t(tag === "record" ? "kit.reward.tag.record" : "kit.reward.tag.first");
}

/** An orange or gold step its climb only opened (2026-10-09). */
function opened(reward) {
	return reward.way === "opened";
}

/** What a reward says in its first line. */
export function rewardWords(reward, move) {
	switch (reward.kind) {
		case "moved": return move?.name ?? "";
		case "cardio": return t("pump.home.embers", {have: reward.have, need: reward.need});
		case "stats": return reward.stats.map((s) => rewardWords(s)).join(", ");
		case "record": return t("kit.reward.record", {value: reward.value});
		case "first_record": return t("kit.reward.first", {value: reward.value});
		case "enough": return t("kit.reward.enough", {value: reward.value});
		case "placed": return t("kit.reward.placed", {value: reward.value, record: reward.record});
		case "star": return t("pump.reward.stars", {count: reward.count});
		case "xp": return t("pump.reward.xp", {xp: reward.xp});
		case "stat": return t("kit.reward.stat", {amount: reward.amount, stat: reward.stat});
		case "points": return t("pump.reward.points", {count: reward.count});
		case "title": return t("kit.reward.title", {title: reward.title});
		case "unlock": return t(reward.rekindled ? "kit.reward.rekindled" : reward.way === "earned" ? "kit.reward.won"
			: opened(reward) ? "kit.reward.opened" : "kit.reward.unlock", {move: move?.name ?? reward.ladder});
		case "day": return dayWords(reward);
		case "vault": return t("kit.reward.vault");
		case "mastered": return t("kit.reward.mastered", {ladder: reward.ladder, n: reward.step});
		case "check": return t("kit.reward.check", {move: reward.move});
		case "check_climb": return t("kit.reward.check.climb", {ladder: reward.ladder, n: reward.step});
		case "check_held": return reward.line;
		case "check_placed": return t("kit.reward.check.placed", {ladder: reward.ladder, n: reward.step});
		case "check_waits": return t("pump.check.again");
		case "best_new": return t("kit.reward.best.new", {count: reward.count});
		case "best": return t("kit.reward.best", {count: reward.count});
		case "slipped": return t("kit.reward.slipped");
		case "not_all_out": return t("kit.reward.notallout");
		case "clean_day": return t("kit.reward.cleanday", {have: reward.have, need: reward.need, n: reward.step});
		case "full_move": return t("kit.reward.fullmove", {have: reward.have, need: reward.need, n: reward.step});
		case "haul_day": return t("kit.reward.haulday",
			{time: reward.time, have: reward.have, need: reward.need, next: reward.next});
		case "return": return t(reward.back ? "kit.reward.return.back" : "kit.reward.return.holds", {n: reward.step});
		case "climb_waits": return t("pump.climb.again");
		default: return milestoneWords(reward.days);
	}
}

/** A weekly vault goal's line under Weekly goal reached ("40 moves", an embers tier's "9 embers"). */
function vaultGoal(reward) {
	switch (reward.goal) {
		case "moves": return t("kit.reward.vault.moves", {count: reward.count});
		case "records": return t("kit.reward.vault.records", {count: reward.count});
		default: return t("kit.reward.vault.embers", {count: reward.count});
	}
}

/** A moveless row's glyph and its colour's class. */
function glyphOf(reward) {
	switch (reward.kind) {
		case "star": return ["star", "qp-gold"];
		case "title": return ["reward_title", "qp-text"];
		case "milestone": return ["mark_check", "qp-text"];
		case "mastered": case "check_placed": case "check_climb": return ["mark_check", "qp-gold"];
		case "check_waits": case "check_held": case "climb_waits": return ["mark_dash", "qp-text2"];
		case "clean_day": case "full_move": case "haul_day": case "cardio": return ["mark_check", "qp-text2"];
		// a step that holds the pump day's check; one that went back a dash, not done, as the climb that waits
		case "return": return [reward.back ? "mark_dash" : "mark_check", "qp-text2"];
		case "vault": case "xp": return ["reward_xp", "qp-text2"];
		case "stat": return reward.glyph ? [reward.glyph, "qp-text2"] : null;
		case "points": return ["reward_familiarity", "qp-text2"];
		case "day": return [reward.line === "welcome" ? "reward_xp"
			: reward.line === "streak" || reward.line === "streak_new" ? "mark_check" : "rating_defense", "qp-text2"];
		default: return null;
	}
}

/** A move's row under its name: what it did with its tag and the record it passed, its lines, then each step it counts
 * toward with its days as pips (the last one lit as the row arrives where this pump brought it). */
function movedParts(reward) {
	const did = h("span", {class: "qp-reward-did"},
		reward.amount ? h("span", {class: "qp-type-num-small", text: reward.amount}) : null,
		reward.tag ? h("qp-chip", {class: "qp-reward-tag", state: "plain", glyph: "reward_record",
			label: tagWords(reward.tag)}) : null,
		reward.was ? h("span", {class: "qp-type-meta", text: reward.was}) : null);
	const steps = reward.steps.map((step) => h("span", {class: "qp-reward-step"},
		h("span", {class: "qp-type-meta", text: step.words}),
		h("qp-day-pips", {have: step.have, need: step.need, brought: step.brought, "aria-hidden": "true"})));
	const lines = reward.lines.map((line) => h("span", {class: "qp-type-meta", text: line}));
	return [did.childElementCount ? did : null, ...lines, ...steps].filter(Boolean);
}

class QpRewardRow extends KitElement {
	build() {
		this.tile = h("qp-move-tile", {size: 44});
		this.lead = h("span", {class: "qp-reward-lead"});
		this.first = h("span", {class: "qp-reward-first"});
		this.second = h("span", {class: "qp-reward-second"});
		this.next = h("span", {class: "qp-type-meta"});
		this.mark = h("qp-art", {name: "reward_record", class: "qp-reward-glyph"});
		this.shareButton = h("qp-round-button", {icon: "share", small: true, "on:click": () => emit(this, "qp-share")});
		this.earned = h("span", {class: "qp-unseen"});
		this.append(this.tile, this.lead, h("span", {class: "qp-reward-words"}, this.first, this.second, this.next),
			this.mark, this.shareButton, this.earned);
	}

	set reward(value) {
		this.data = value ?? null;
		if (this.built) this.update();
	}

	get reward() {
		return this.data ?? null;
	}

	set move(value) {
		this.moveData = value ?? null;
		if (this.built) this.update();
	}

	get move() {
		return this.moveData ?? null;
	}

	/** What the reveal says of it: the reward, then the move a record is on (a first record's next goal after it), or
	 * an unlock's ladder and step with its quality's word (an opened step's with none). */
	spoken() {
		const reward = this.reward;
		if (!reward) return [];
		const words = rewardWords(reward, this.move);
		if (reward.kind === "unlock") {
			const step = t("kit.reward.step", {ladder: reward.ladder, step: reward.step});
			return opened(reward) ? [words, step] : [words, step, reward.word];
		}
		if (reward.kind === "vault") return [words, vaultGoal(reward)];
		if (reward.kind === "moved") {
			return [words, reward.amount, reward.tag ? tagWords(reward.tag) : null, reward.was, ...reward.lines,
				...reward.steps.map((step) => step.words)].filter(Boolean);
		}
		if (!RECORDS.has(reward.kind)) return [words];
		const next = reward.kind === "first_record" && reward.next ? [t("kit.reward.next", {goal: reward.next})] : [];
		return [words, ...(this.move ? [this.move.name] : []), ...next];
	}

	update() {
		const reward = this.reward;
		if (!reward) return;
		const move = this.move;
		const unlock = reward.kind === "unlock";
		const quality = unlock ? reward.quality : move?.quality;
		const tone = qualityName(quality);
		if (tone) this.style.setProperty("--qp-tone", `var(--qp-quality-${tone})`);
		else this.style.removeProperty("--qp-tone");
		this.tile.hidden = !move;
		if (move) {
			if (quality) this.tile.quality = quality;
			this.tile.move = move.key ?? null;
		}
		this.tile.mastered = reward.kind === "mastered";
		const glyph = move ? null : glyphOf(reward);
		this.lead.hidden = !glyph;
		if (glyph) this.lead.replaceChildren(h("qp-art", {name: glyph[0], class: glyph[1]}));
		this.first.textContent = rewardWords(reward, move);
		const quiet = quietRow(reward) && reward.kind !== "moved";
		this.first.className = quiet ? "qp-type-meta"
			: unlock && quality >= 6 ? "qp-type-body-strong qp-toned" : "qp-type-body-strong";
		this.toggleAttribute("data-quiet", quietRow(reward));
		this.second.classList.toggle("qp-reward-moved", reward.kind === "moved");
		this.first.hidden = reward.kind === "stats";
		if (reward.kind === "moved") {
			this.second.replaceChildren(...movedParts(reward));
		} else if (reward.kind === "stats") {
			this.second.replaceChildren(...reward.stats.map((s) => h("qp-chip",
				{state: "plain", glyph: s.glyph, label: rewardWords(s)})));
		} else if (reward.kind === "cardio") {
			this.second.replaceChildren(h("qp-day-pips", {have: reward.have, need: reward.need, "aria-hidden": "true"}));
		} else if (unlock) {
			const step = h("span", {class: "qp-type-meta",
				text: t("kit.reward.step", {ladder: reward.ladder, step: reward.step})});
			this.second.replaceChildren(...(opened(reward) ? [step]
				: [step, h("qp-quality-word", {quality: reward.quality, word: reward.word})]));
		} else if (reward.kind === "vault") {
			this.second.replaceChildren(h("span", {class: "qp-type-meta", text: vaultGoal(reward)}));
		} else {
			// a mastered step's tile is its anchor's, whose name is no line of its own (2026-10-22)
			const named = move && reward.kind !== "mastered";
			this.second.replaceChildren(named ? h("span", {class: "qp-reward-name", text: move.name}) : "");
		}
		this.next.textContent = reward.kind === "first_record" && reward.next ? t("kit.reward.next", {goal: reward.next})
			: "";
		this.next.hidden = !this.next.textContent;
		this.mark.hidden = !RECORDS.has(reward.kind);
		this.shareButton.hidden = !((unlock || reward.kind === "milestone") && this.share);
		this.earned.textContent = EARNED.has(reward.kind) || (reward.kind === "moved" && reward.tag)
			? t("kit.reward.earned") : "";
		this.setAttribute("role", "group");
	}
}

define("qp-reward-row", QpRewardRow, {share: Boolean, shown: Boolean});
