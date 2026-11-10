// The pump's words on the computer (docs/plan.md slice 35): the phones' keys, named here by their literals so the copy's
// lint finds each one, and the values the `pump` topic carries put into them (a hold's count as a clock, a count with
// the computer's digit grouping, a weight in kg from its tenths). The game's own words come from Rust already filled
// in, but for a count typed with a weight, whose amounts these fill in.

import {t} from "../../lib/copy.js";
import {kg} from "../../lib/format.js";
import {rewardWords} from "../../kit/qp-reward-row.js";
import {fill} from "../hero/words.js";

/** The answers to "How close to the goal pose?" in the ask's order, each its code (Core.lua ns.Enum.mark) and its
 * word alone (2026-10-22): the pump's ask (each on the key of its code, 4 Perfect to 1 Not yet, as the game's) and
 * Correct a count's pose (slice 44). */
export const ANSWERS = [
	[4, "pump.mark.ask.perfect"],
	[3, "pump.mark.ask.good"],
	[2, "pump.mark.ask.okay"],
	[1, "pump.mark.ask.notyet"],
];

/** Seconds as a clock, m:ss (every hold's time and a timed move's). */
export function clock(seconds) {
	const s = Math.max(0, Math.floor(seconds));
	return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, "0")}`;
}

/** A count as the move keeps it: a hold's seconds as a clock, else the number in the computer's digits. */
export function amount(value, hold) {
	return hold ? clock(value) : new Intl.NumberFormat().format(value);
}

/** A set with a weight (tenths of a kg), as every record and set place says it: "11 kg × 16", "6 kg each × 12". */
export function lifted(load, reps, each) {
	return t(each ? "kit.record.load.each" : "kit.record.load", {kg: kg(load), reps});
}

/** The record a pump's move names (its Ready view, its window): a loaded move's weighted record ("10 kg × 20"), else
 * its count (a hold's as a clock, a counted set's best clean count); none without one. */
export function recordWords(m) {
	if (m.load?.top) return lifted(m.load.top.load, m.load.top.reps, m.load.each);
	return m.record == null ? null : amount(m.record, m.hold);
}

/** What the player finished (a key of contract/kinds.json) as Start a pump names it, or the Quick pump; a key this
 * build has no words for shows as it is. */
export function kindName(key) {
	switch (key) {
		case "quest": return t("pump.kind.quest");
		case "match": return t("pump.kind.match");
		case "level": return t("pump.kind.level");
		case "boss": return t("pump.kind.boss");
		case "stream": return t("pump.kind.stream");
		case "work": return t("pump.kind.work");
		case "quick": return t("pump.quick");
		default: return key;
	}
}

/** The sizes of an app pump, Light · Balanced · Extreme, as the Segmented takes them (tiers 1 to 3). */
export function sizes() {
	return [
		{value: "1", label: t("pump.size.small")},
		{value: "2", label: t("pump.size.normal")},
		{value: "3", label: t("pump.size.big")},
	];
}

/** The Result's Continue: a work pump's Back to work, which brings the app that was in front back (`pump.back`, slice
 * 67: on the computer alone); a move pump returns where it started (the ladder's page, the move's window or page, the
 * list); any other pump's Continue. */
export function continueWords(pump) {
	if (pump.back) return t("desk.pump.done.work");
	switch (pump.what === "move" ? pump.from?.place : null) {
		case "ladder": return t("pump.done.back.ladder");
		case "move": return t("pump.done.back.move");
		case "list": return t("pump.done.back.list");
		default: return t("pump.done.continue");
	}
}

/** A move's goal on its Ready view, the game's goalOf in the phones' words: its value (`text`), the caps word under it
 * (`word`: Each side on a side move, else Goal for a count or a hold's time to reach, "Goal · set 2 today" where its
 * ladder had sets this game day (`target.set`, 2026-10-28), a counted set's time; none where the value says it all) and
 * whether the value is a number (`number`: "16+", a time) or words (a loaded move's "16+ reps with 11 kg", All out,
 * Max set, One set). */
export function goalOf(m) {
	const value = m.target.value ?? 0;
	const each = m.sides ? t("kit.eachside") : null;
	const goal = m.target.set ? t("kit.goal.set", {set: m.target.set}) : t("kit.goal");
	switch (m.target.kind) {
		case "goal":
			if (m.load?.plan != null) {
				const load = {target: value, kg: kg(m.load.plan)};
				const text = t(m.load.each ? "pump.ready.load.each" : "pump.ready.load", load);
				return {text, word: each ?? goal};
			}
			return {text: t("kit.goal.value", {goal: value}), word: each ?? goal, number: true};
		case "time": return {text: clock(value), word: each ?? (m.hold ? goal : null), number: true};
		case "allout": return {text: t("pump.ready.allout"), word: clock(value)};
		case "max": return {text: t("kit.maxset"), word: each};
		default: return {text: t("kit.set"), word: each};
	}
}

/** A steady move's pace in words by its cadence rung (`pump.pace.rung<n>`, rungs 1 to 6, Rust's `cadence_rung`: from
 * step 7 step 6's since slice 52, a Long Haul step 1's since slice 55; slice 42's fix round, 2026-10-17: the talk
 * test the cadence lines set, so the pace ask's "that pace" is stated without the voice), rung 0 a match pump's warm
 * move's easy cadence (`pump.pace.easy`, the stand-up's: the Rust's `EASY_CADENCE`, slice 48's double-check). */
const PACES = ["pump.pace.rung1", "pump.pace.rung2", "pump.pace.rung3", "pump.pace.rung4", "pump.pace.rung5",
	"pump.pace.rung6"];

export function paceLine(rung) {
	return t(rung === 0 ? "pump.pace.easy" : PACES[Math.min(Math.max(rung, 1), PACES.length) - 1]);
}

/** What a pose family's days toward its step still lack (slice 47, 2026-10-19; Tree.lua Tree.PoseShort, the game's
 * ladders.step.shortMain and .shortPerfect): the days still to come on the step's main pose (`main`), by its name
 * (`pose`), and those still to come answered Perfect (`perfect`), each a sentence; null where neither is short. The
 * Book's pose climb (its `main` and `perfect`) under the step card's climb line, and the Ready view's day note after
 * its days (Rust's `Short`). */
export function poseShort({main, perfect}, pose) {
	const parts = [];
	if (main > 0 && pose) parts.push(t("ladders.step.short.main", {days: main, move: pose}));
	if (perfect > 0) parts.push(t("ladders.step.short.perfect", {days: perfect}));
	return parts.length ? parts.join(" ") : null;
}

/** A pose's day note under its goal pose (slice 47, Panel.lua poseDaysLine): any goal pose of the step counts the
 * family's day (held Perfect where only Perfect counts), the days so far of those the step takes, then what they
 * still lack, one line as the game joins it. */
function poseDays(l) {
	const days = t(l.perfect ? "pump.ready.mark.days.perfect" : "pump.ready.mark.days", {days: l.days, total: l.total});
	const short = poseShort(l.short, l.short.pose);
	return short ? `${days} ${short}` : days;
}

/** A line under the goal of the move on Ready, its values alone (2026-10-22): since slice 42 a counted set's goal with
 * no number ("All out, clean reps") and its best clean count (a record, never a goal), since its fix round a steady
 * move's pace in words, and since slice 47 a pose's family's days; none for the lines that only explained (a first
 * clean count's, a stretch check's, a lighter goal's reason). A side move's record and goal each side stand in its
 * goal and record (2026-10-24, as the phones'). */
export function line(l) {
	switch (l.kind) {
		case "words": return l.text;
		case "carry": return t("pump.ready.carry", {seconds: l.seconds});
		case "carry_side": return t("pump.ready.carry.side", {seconds: l.seconds});
		case "flow_side": return t("pump.ready.flow.side", {seconds: l.seconds});
		case "form": return t("pump.ready.goal.form",
			{form: l.load == null ? amount(l.form, l.hold) : lifted(l.load, l.form, l.each)});
		case "load_start":
			return t(l.each ? "pump.ready.load.start.each" : "pump.ready.load.start", {lo: l.lo, hi: l.hi});
		case "load_heavier": return t("pump.ready.load.heavier", {reps: l.reps, kg: kg(l.load)});
		case "allout_clean": return t("pump.ready.allout.clean");
		case "best": return t("pump.ready.best", {count: l.count});
		case "pose_days": return poseDays(l);
		case "pace": return paceLine(l.rung);
		default: return null;
	}
}

/** The count step's question, naming the move: a counted set's form step's "did your form hold?" (slice 42). */
export function question(count) {
	switch (count.question) {
		case "form": return t("pump.count.form", {move: count.name});
		case "hold_side": return t("pump.count.hold.side", {move: count.name});
		case "hold": return t("pump.count.hold", {move: count.name});
		case "reps_side": return t("pump.count.reps.side", {move: count.name});
		default: return t("pump.count.reps", {move: count.name});
	}
}

/** "Is that right?" in the game's words: filled in by Rust, or for a count with a weight filled here with the count at
 * its weight and the record it was checked against (the weighted record, else the count's), or the weight's own
 * question with the weight typed and the one planned ("18 kg is far above the 8 kg planned"). */
export function checkWords(check) {
	if (check.words === "filled") return {line: check.line, yes: check.yes};
	const value = lifted(check.load, check.value, check.each);
	if (check.words === "heavy") {
		return {line: fill(check.line, kg(check.load), kg(check.plan)), yes: fill(check.yes, value)};
	}
	const record = check.record_load == null ? amount(check.record, false)
		: lifted(check.record_load, check.record, check.each);
	return {line: fill(check.line, value, record), yes: fill(check.yes, value)};
}

/** What Save will keep on a loaded move with a weight in its box: "16 reps with 11 kg", "12 reps with 6 kg each". */
export function preview(count, load, each) {
	return t(each ? "pump.count.preview.load.each" : "pump.count.preview.load", {count, kg: kg(load)});
}

/** A move done with a weight's next fresh set: "Next time: 11 kg, 15+ reps". */
export function nextTime(n) {
	return t(n.each ? "pump.done.load.next.each" : "pump.done.load.next", {kg: kg(n.load), target: n.target});
}

/** The running move's line in Big view at `elapsed` seconds: an all-out window's part from its go (its build, then all
 * out; a counted set has no build, all out from its go), else the move's stop line, empty for a timed move that has
 * none, which the empty line hides (David, 2026-10-24: only what is legally necessary). */
export function phase(move, elapsed) {
	if (move.allout) {
		if (elapsed < (move.build ?? 0)) return t("pump.phase.build");
		return move.counted ? t("pump.phase.count") : t("pump.phase.fast");
	}
	return move.stop ?? "";
}

/** The move's name in Big view, with its side on a side move. */
export function sideName(move, side) {
	if (side === "first") return t("pump.side.first", {move: move.name});
	if (side === "other") return t("pump.side.other", {move: move.name});
	return move.name;
}

/** What still holds what a passed stretch check opens, in the wait's own words, as the card says each (slice 42's fix
 * round, 2026-10-17; Rust's `WaitWords`): another check's pose answered Good or better (Perfect) once, a record of
 * another ladder, a step another ladder's climb does, or the stage of a step's path (the move whose top opens it with a
 * loaded move's weight, "20 with 15 kg or more", "a side" for a move done on each side, a pose's first Good or Perfect
 * day). */
function waitWords(w) {
	switch (w.kind) {
		case "check": return t(w.perfect ? "ladders.card.check.need.perfect" : "ladders.card.check.need", {move: w.pose});
		case "record": return t("ladders.step.waits", {move: w.move, count: w.seconds ? clock(w.count) : String(w.count)});
		case "step": return t("ladders.step.waits.step", {move: w.move, ladder: w.ladder});
		default: {
			if (w.pose) {
				return t(w.perfect ? "ladders.card.stage.opens.perfect" : "ladders.card.stage.opens.mark", {move: w.move});
			}
			if (w.top == null) return t("ladders.measure.ahead", {move: w.move});
			// its top in its own unit with a loaded move's weight, as the card's line says it (the game's Sheet.ClimbTop)
			const at = amount(w.top, w.seconds);
			const goal = w.kg ? t(w.each ? "ladders.goal.load.each" : "ladders.goal.load", {goal: at, kg: w.kg}) : at;
			return t("ladders.card.stage.opens", {move: w.move, goal: w.side ? t("ladders.card.stage.side", {goal}) : goal});
		}
	}
}

/** A move's row (Rust's `Reward::Moved`, 2026-10-29) in words: what it did (a set with a weight as "11 kg × 16", a
 * hold's count or a timed move's seconds as a clock; none for one done with neither), its tag with the record it passed
 * ("was 35"), its notes (a first record's next goal, each other note in its row's words), its next weight, and its
 * steps with their days (`brought`: this pump brought the last one in; the days at the top are counted so far). */
function moved(r) {
	const did = r.count == null ? null : r.load != null ? lifted(r.load, r.count, r.each) : amount(r.count, r.clock);
	const was = r.was == null ? null
		: t("kit.reward.was", {value: r.was.load != null ? lifted(r.was.load, r.was.value, r.was.each)
			: amount(r.was.value, r.was.hold)});
	const lines = r.notes.map((note) => {
		const n = reward(note);
		return n.kind === "first_record" ? (n.next ? t("kit.reward.next", {goal: n.next}) : null) : rewardWords(n);
	}).filter(Boolean);
	if (r.again) lines.push(nextTime({...r.again, each: r.each}));
	const steps = r.steps.map((step) => ({words: step.kind === "top_day"
		? t("kit.reward.topday", {have: step.have, need: step.need, n: step.step}) : rewardWords(reward(step)),
	have: step.have, need: step.need, brought: step.kind !== "top_day"}));
	return {kind: r.kind, amount: did ?? (r.time != null ? clock(r.time) : null), tag: r.tag, was, lines, steps};
}

/** A reward row as RewardRow takes it: its values put in words (a hold's as a clock, a set with a weight as "11 kg ×
 * 16", a placement's record that stays with its weight where it has one), a first record's next goal, a stat's glyph,
 * a day's line, a passed check whose opening still waits, what holds it (slice 42's fix round), a move's row
 * (`moved`) and the stats raised together. */
export function reward(r) {
	const value = () => (r.load != null ? lifted(r.load, r.value, r.each) : amount(r.value, r.hold));
	switch (r.kind) {
		case "moved": return moved(r);
		case "stats": return {kind: r.kind, stats: r.stats.map(reward)};
		case "record": return {kind: r.kind, value: value()};
		case "enough": return {kind: r.kind, value: amount(r.value, r.hold)};
		case "placed": return {kind: r.kind, value: value(),
			record: r.record_load != null ? lifted(r.record_load, r.record, r.each) : amount(r.record, r.hold)};
		case "first_record": return {kind: r.kind, value: value(),
			next: r.next == null ? null : r.hold ? t("ladders.next.time", {time: clock(r.next)})
				: t("kit.goal.value", {goal: r.next})};
		case "stat": return {kind: r.kind, amount: r.amount, stat: r.stat, glyph: `stat_${r.key}`};
		case "day": return {kind: r.kind, line: r.line, xp: r.value, days: r.value, count: r.value};
		case "check_held": return {kind: r.kind, line: waitWords(r.wait)};
		case "haul_day": return {kind: r.kind, time: clock(r.time), have: r.have, need: r.need, next: clock(r.next)};
		default: return r;
	}
}
