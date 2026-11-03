// A step's words (docs/ui.md, "Hero, the Book and the Log" and "The talent trees", "The words"; the phones'
// LadderDetail and StepCard): when a step's rung became the player's, what its climb takes (the game's next lines in
// the goal's grammar), the stages of its path (slice 42: each a stage row's words, its state spoken, the "Then …"
// line once the climb's days are in) and the other ladder's step it waits for, a stretch check's row (slice 42),
// Mastered, what earns an orange or gold step, and what reaching a step of the Path takes. The ladder's page, the trees and the
// step's card all word a step through these, and from slice 38 the trees' own: a step's place on its ladder, the
// Tooltip's four lines, its spoken name, a Start's label and the card's lines beside it, a refused Start's lines and
// links, and a tree's Next line, each from the Book's fields as the engine gave them (ns.StartFor, ns.StepRing,
// ns.TreeEntry, ns.TreeGlow, ns.Stages, ns.MoveOpen, ns.NeedDone), never a rule of its own.

import {t} from "../../lib/copy.js";
import {clock, LOCALE, time} from "../../lib/format.js";
import {poseShort} from "../pump/words.js";
import {
	amount, dayCount, factsOf, gapWords, goal, moveName, number, opensWords, qualityWord,
} from "./common.js";

const DAY_MS = 86_400_000;
const LIST = new Intl.ListFormat(LOCALE, {type: "conjunction", style: "narrow"});
/** "How close to the goal pose?"'s Perfect (Core.lua ns.Enum.mark): a pose step's `ask` where only it counts a day. */
const PERFECT = 4;
const STEP_DATE = new Intl.DateTimeFormat(LOCALE, {day: "numeric", month: "short", timeZone: "UTC"});
const DATE = new Intl.DateTimeFormat(LOCALE, {day: "numeric", month: "short"});

/** A moment's game day as a date (a step mastered, a check passed: unix seconds), the day turning at the Book's
 * `dayOffsetH` (04:00) on the computer's clock, as the game's ns.DayMid and the phones' GameDay read it: a check passed
 * at 01:00 shows the day before, the same date on every surface (slice 42's last review, 2026-10-18). */
function gameDate(w, at) {
	return DATE.format(new Date((at - w.words.dayOffsetH * 3600) * 1000));
}

/** When a step's rung became the player's. */
export function reachedWords(reached) {
	if (!reached || reached.kind === "start") return t("ladders.step.start");
	if (reached.kind === "kept") return t("ladders.step.kept");
	return t("ladders.step.earned", {date: STEP_DATE.format(new Date(reached.day * DAY_MS))});
}

/** What a step's row says under its move (the Talents tab's step lines, Tree.lua `Tree.KeyLines`): when it became the
 * player's, its line, its stars or day pips, and once its climb's days are in the stage of its path it waits for
 * (`then`, slice 42); the next step of a counted cardio climb its ladder's proving window (`proving`, the Book row's,
 * slice 53). */
export function stepWords(w, step, book, proving = null) {
	const since = step.reached ? reachedWords(step.reached) : null;
	const state = step.state;
	switch (state.kind) {
		case "held": return {reached: reachedWords(step.reached ?? {kind: "start"})};
		// an orange or gold step its climb only opened (2026-10-09): what earns it, one fresh set at its floor
		case "now": return {reached: since, stars: state.stars ?? null, line: state.earn ? earnWords(w, state.earn) : null};
		case "next": return {...climbWords(w, state.climb, book, proving), reached: since};
		case "shut": return {reached: since, line: state.gap ? gapWords(w, state.gap) : t("ladders.step.shut")};
		case "off": return {reached: since, line: t("ladders.step.off", {move: moveName(w, state.move)})};
		case "waits":
		case "waitsStep": return {reached: since, line: waitsWords(w, state)};
		case "pain": return {reached: since, line: t("ladders.step.pain")};
		default: return {};
	}
}

/** A climb that waits for a step of another ladder: that step's move and the record that does it, or on a pose or
 * cardio step (`waitsStep`, 2026-10-10) that step's move on its ladder. */
function waitsWords(w, state) {
	if (state.kind === "waitsStep") {
		return t("ladders.step.waits.step", {move: moveName(w, state.move),
			ladder: w.ladders.get(state.ladder)?.name ?? ""});
	}
	return t("ladders.step.waits", {move: moveName(w, state.move),
		count: state.seconds ? clock(state.count) : String(state.count)});
}

// The steps' order (slice 42) ---------------------------------------------------------------------------------------

/** What a stage's move is by its family (the words' ladder's dose): `pose`, `cardio` (a countable form, counted in
 * clean days), else a strength or hold move. */
function stageKind(w, move) {
	const ladder = w.moves.get(move)?.ladder;
	return ladder != null ? w.ladders.get(ladder)?.dose ?? null : null;
}

/** The stages of a path still left (open or waiting): the "and {count} more moves" of a next step's key line. */
function stagesLeft(stages) {
	return (stages ?? []).filter((s) => s.state === "open" || s.state === "waiting").length;
}

/** Lines or names joined as a stage's alternatives ("{first} or {second}", `ladders.card.stage.or`). */
function either(parts) {
	return parts.reduce((first, second) => t("ladders.card.stage.or", {first, second}));
}

/** The moves of a stage its row names: the ones the setup offers (the Book's `offered`, slice 42's fix round,
 * 2026-10-17: never an alternative the setup leaves out), else the move its row names. */
function namedIds(stage) {
	return stage.offered?.length ? stage.offered : [stage.move];
}

/** A stage's moves named as its row names them: one, or the alternatives the setup offers joined ("Squat thrusts or
 * sprawls", `ladders.card.stage.or`). */
function stageNames(w, stage) {
	const names = namedIds(stage).map((id) => moveName(w, id)).filter(Boolean);
	return names.length ? either(names) : moveName(w, stage.move);
}

/** The top of each alternative of a stage the setup offers (the Book's `tops`, by move), where they differ (slice 42's
 * fix round, 2026-10-17: the sliding one-arm push-up's 18 beside the hand-on-the-bed one's 12, each its own); none
 * where one top serves them all. */
function ownTops(stage) {
	const tops = namedIds(stage).map((id) => ({move: id, top: stage.tops?.find((x) => x.move === id)?.top ?? stage.top}));
	return tops.length > 1 && tops.some((x) => x.top !== tops[0].top) ? tops : null;
}

/** What a stage takes, its row's line: a strength or hold move's top on its days (`ladders.path.need`), a pose's goal
 * pose close enough or Perfect on its days, a countable form's clean days so far; two alternatives whose tops differ
 * each with its own ("Sliding one-arm push-up 18+ on 2 days or One-arm push-up, hand on the bed 12+ on 2 days"); a done
 * countable stage its moves alone under its check, counting nothing more (slice 42's fix round, 2026-10-17: ns.Stages
 * gives a done stage no days). */
export function stageLine(w, stage) {
	const own = ownTops(stage);
	switch (stageKind(w, stage.move)) {
		case "pose": {
			const line = (move, top) => t(top >= PERFECT ? "ladders.card.stage.mark.perfect" : "ladders.card.stage.mark",
				{move, days: stage.days});
			return own ? either(own.map((x) => line(moveName(w, x.move), x.top))) : line(stageNames(w, stage), stage.top);
		}
		case "cardio":
			if (stage.state === "done") return stageNames(w, stage);
			return t("ladders.card.stage.clean", {move: stageNames(w, stage), ...dayCount(stage.have ?? 0, stage.days)});
		default: {
			const line = (id, move, top) => (top != null && top !== true
				? t("ladders.path.need", {move, goal: goal(top, holds(w, id)), days: stage.days}) : move);
			return own ? either(own.map((x) => line(x.move, moveName(w, x.move), x.top)))
				: line(stage.move, stageNames(w, stage), stage.top);
		}
	}
}

/** What opens a waiting stage (`after` at `afterTop` with a loaded opener's weight, `opensAfter`); none for a stage
 * not waiting. */
export function stageOpens(w, stage) {
	if (stage.state !== "waiting" || stage.after == null) return null;
	return opensAfter(w, {after: stage.after, top: stage.afterTop, kg: stage.afterKg, each: stage.afterEach});
}

/** What a waiting stage or move opens after (`opens`: the move `after` at its top `top`, a path wait or a stage's opener;
 * the game's Tree.OpensAfter): "Opens after Bulgarian split squat at 25 a side" (its top in its own unit: a count, a
 * hold's time, a loaded move's weight, `kg` and `each`, as the game's Sheet.ClimbTop adds it, "a side" on a move done on
 * each side since slice 42's fix round, 2026-10-17), a pose's first Good or Perfect day, a form's first clean window. With
 * `sentence` a held line's wait (`heldWords`), a whole sentence with its own full stop ("Opens after Bulgarian split
 * squat at 25 a side.", `ladders.held.opens` and its `.mark` and `.perfect`; 2026-10-19, the double-check's fix
 * review: no code adds or strips one); none there for a form's clean window, which has no sentence and no held line
 * names (every stage of the cardio ladder's paths opens with its step). */
function opensAfter(w, {after, top, kg, each}, sentence = false) {
	const move = moveName(w, after);
	if (stageKind(w, after) === "pose") {
		const perfect = top >= PERFECT;
		const key = sentence ? (perfect ? "ladders.held.opens.perfect" : "ladders.held.opens.mark")
			: perfect ? "ladders.card.stage.opens.perfect" : "ladders.card.stage.opens.mark";
		return t(key, {move});
	}
	if (top == null || top === true) return sentence ? null : t("ladders.measure.ahead", {move});
	const count = amount(top, holds(w, after));
	const at = kg ? t(each ? "ladders.goal.load.each" : "ladders.goal.load", {goal: count, kg}) : count;
	return t(sentence ? "ladders.held.opens" : "ladders.card.stage.opens", {move,
		goal: w.moves.get(after)?.side ? t("ladders.card.stage.side", {goal: at}) : at});
}

/** Why a skipped stage leaves the climb (its `skip`): the player's own list leaves its move out ("You left it out"),
 * or the setup does; none for a stage not skipped. */
export function stageSkip(stage) {
	if (stage.state !== "skipped") return null;
	return t(stage.skip === "off" ? "ladders.card.stage.skip.off" : "ladders.card.stage.skip.setup");
}

/** A stage row's state for a screen reader: done, open with its days, waits, skipped. */
export function stageSpoken(stage) {
	switch (stage.state) {
		case "done": return t("ladders.card.stage.done");
		case "open": return t("ladders.card.stage.open", dayCount(stage.have ?? 0, stage.days));
		case "waiting": return t("ladders.card.stage.waits");
		default: return t("ladders.card.stage.skipped");
	}
}

/** The stage's moves as one line names them with one top: the alternatives the setup offers joined, or where their
 * tops differ the move the stage names (nearest its days) alone, with its own top (slice 42's fix round). */
function proofNames(w, stage) {
	return ownTops(stage) ? moveName(w, stage.move) : stageNames(w, stage);
}

/** The name a line gives the path's `next` stage (the engine's, ns.PathNext; the iPhone's StepLines.proofName,
 * Android's nextName): the stage's moves as its row names them (`proofNames`) where it names the move the climb's turn
 * runs and no wait holds it, else that move alone (the game's Tree.PathKey names the move; 2026-10-19, the
 * double-check's fix review: a stage's own move may be one its check holds while the turn runs another of it). */
function nextName(w, path, next) {
	const stage = path?.[next.stage - 1];
	return stage && stage.move === next.move && !next.waits ? proofNames(w, stage) : moveName(w, next.move);
}

/** The path's `next` stage once the climb's own days are in, by the keys of the line that names it (`keys`: the
 * hover's key line and a ladder page's next row, `ladders.measure.proof*`, the game's `tree.key.path*`, or a tree's
 * Next line, `ladders.tree.next.proof*`): "Then Pistol, hands on the desk 8+" (its top with a loaded move's weight, as
 * the game's Sheet.ClimbTop), a pose's close enough or Perfect on its days, a form's clean days so far; in the move's
 * own top, days and days so far, as the game's and the phones' lines read them. */
function thenWords(w, path, next, keys) {
	const proof = nextName(w, path, next);
	switch (stageKind(w, next.move)) {
		case "pose": return t(next.top >= PERFECT ? keys.perfect : keys.mark, {proof, days: next.days});
		case "cardio": return t(keys.clean, {proof, ...dayCount(next.have, next.days)});
		default:
			return t(keys.count, {proof, goal: next.top != null && next.top !== true
				? climbGoal(next.top, holds(w, next.move), next) : ""}).trim();
	}
}

/** The keys of a key line's and a tree's Next line's next stage (`thenWords`). */
const THEN_KEYS = {
	key: {count: "ladders.measure.proof", mark: "ladders.measure.proof.mark",
		perfect: "ladders.measure.proof.mark.perfect", clean: "ladders.measure.proof.clean"},
	tree: {count: "ladders.tree.next.proof", mark: "ladders.tree.next.proof.mark",
		perfect: "ladders.tree.next.proof.perfect", clean: "ladders.tree.next.proof.clean"},
};

/** What holds the path's next stage where its move is not open (the engine's `next.waits`, ns.PathNext's: the game's
 * Tree.PathHeldWords and Tree.WaitWords with `sentence`), as the {wait} of a held line, always a whole sentence with
 * its own full stop that the line adds none after and no code adds or strips (2026-10-19, the double-check's fix
 * review): the move whose top opens its stage ("Opens after Bulgarian split squat at 25 a side.", `opensAfter`), or
 * the move's own check as the Book reads it (`book`'s move `checks`): a stretch check's ("Opens after your first Good
 * hold of Deep squat.", `checkNeed`), else its need row's line ("Waits for a record of 12 on Feet-up push-up.",
 * `needLine`); none where none names it (the Book not read yet). */
function heldWords(w, book, next) {
	const waits = next.waits;
	if (!waits) return null;
	if (waits.kind === "path") return waits.after != null ? opensAfter(w, waits, true) : null;
	const need = book ? factsOf(book).get(next.move)?.checks?.[waits.check - 1] : null;
	if (need?.move == null) return null;
	return need.check != null ? checkNeed(w, need.move, need.check, true) : needLine(w, {need, ladder: null});
}

/** The climb's stage still to do once its days are in, as the engine's `next` names it (ns.PathNext, the one rule of
 * every Next and key line since 2026-10-19): the move the climb's turn runs at its top (`thenWords`), or where the
 * turn has none the move that holds the climb and what holds it ("Then Pistol, hands on the desk. Opens after
 * Bulgarian split squat at 25 a side.", `heldWords`, the Book `book` its moves' checks); none before its days are in
 * or where the path is in. */
function thenOf(w, climb, book) {
	const next = climb.need > 0 && climb.have >= climb.need ? climb.next : null;
	if (!next) return null;
	const held = heldWords(w, book, next);
	if (held) return t("ladders.measure.proof.held", {proof: moveName(w, next.move), wait: held});
	return thenWords(w, climb.path, next, THEN_KEYS.key);
}

// Stretch checks and Mastered (slice 42) -------------------------------------------------------------------------

/** A stretch check's row (a need or a move's check the engine reads as one: its `check`, the least answer it counts;
 * `entry` the choice the row shows): "{move}: Good or better, once" (Perfect where it asks it); while undone what one
 * hold at its goal pose opens (the move waiting on it, `opens.move`, or the climb to `opens.step`); done, as the
 * engine's ns.CheckPassed says it (the game's Tree.CheckDone, the phones' checkPassed; 2026-10-18): the day it was
 * first passed (`at`, the earliest over an either-or group's checks), else, where no answer did it, "Passed: your
 * {ladder} is at step {n}." (`ladder`, the stretch ladder of the choice that is done, at its step now, `step`). */
export function checkWords(w, entry, state, opens) {
	const perfect = state.check >= PERFECT;
	const need = checkNeed(w, state.move ?? entry.move, state.check);
	if (!state.done) {
		const line = opens.step != null
			? t(perfect ? "ladders.card.check.line.step.perfect" : "ladders.card.check.line.step", {n: opens.step})
			: t(perfect ? "ladders.card.check.line.perfect" : "ladders.card.check.line", {move: moveName(w, opens.move)});
		return {need, line};
	}
	if (state.at) return {need, line: t("ladders.card.check.done", {date: gameDate(w, state.at)})};
	if (state.ladder != null) {
		return {need, line: t("ladders.card.check.done.step", {ladder: w.ladders.get(state.ladder)?.name ?? "",
			n: state.step})};
	}
	return {need, line: null};
}

/** A stretch check's need, its row's words: "{move}: Good or better, once" (Perfect where `check`, the least answer it
 * counts, asks it); with `sentence` a held line's wait ("Opens after your first Good hold of {move}.", `heldWords`;
 * the game's Tree.CheckNeed). */
function checkNeed(w, move, check, sentence = false) {
	const perfect = check >= PERFECT;
	const key = sentence ? (perfect ? "ladders.held.check.perfect" : "ladders.held.check")
		: perfect ? "ladders.card.check.need.perfect" : "ladders.card.check.need";
	return t(key, {move: moveName(w, move)});
}

/** Whether `x`, a step a link into a ladder's step comes from (`linksOf`'s `into`), is the one its link's check row
 * shows: a stretch check the engine reads as one (its `check`), the link's own step or, in an `any` group (slice 42),
 * the choice the setup offers (the engine's `choice`, from 1: ns.EntryFor with the setup alone, the game's
 * Tree.Alternative). */
export function showsCheck(x) {
	return x.need?.check != null && x.entry === (x.link.any?.[x.need.choice - 1] ?? x.link);
}

/** A step's mastery on a step the player has (slice 42): "Mastered {date}" (its game day, gameDate), or before it its
 * progress row's words under Master this step, "{have} of {need} moves at their top" (2026-10-29; "Toward Mastered: …"
 * until then); none on a step ahead or where the setup offers none. */
export function masteryLine(w, st) {
	if (st.mastered) return t("ladders.step.mastered", {date: gameDate(w, st.mastered)});
	if (st.mastery) return t("ladders.card.master.count", dayCount(st.mastery.have, st.mastery.need));
	return null;
}

/** A strength climb's top as a goal, with the weight a loaded move's climb asks (`kg`, each hand's where `each`;
 * ns.ClimbKg, 2026-10-12: "20+ with 15 kg or more"). */
function climbGoal(top, seconds, need) {
	const plain = goal(top, seconds);
	if (!need.kg) return plain;
	return t(need.each ? "ladders.goal.load.each" : "ladders.goal.load", {goal: plain, kg: need.kg});
}

/** What earns an orange or gold step its climb only opened: one fresh set of its move at its floor, a pose answered
 * Perfect. */
function earnWords(w, earn) {
	if (earn.mark) return t("ladders.step.toearn.mark");
	return t("ladders.step.toearn", {move: moveName(w, earn.move), goal: goal(earn.count, earn.seconds)});
}

/** What the climb to a step takes, the game's next lines in the goal's grammar, with its day pips (`days`), its path's
 * stages (`stages`, the card's rows) and once its days are in the stage still to do (`then`, slice 42; `thenOf` with
 * the Book `book`); a counted cardio climb's clean days (slice 42) and a timed one's full moves (slice 52; its ready
 * days until then, the climb's kind keeping their name), each marked (`clean`, `full`) so the card's measure says them
 * once, and a counted climb whose days and path are in its proving window (`proving`, the Book row's, slice 53,
 * Tree.lua Tree.ClimbLine: its form's one clean window with no pips, or after a try that did not pass this session
 * that the climb waits); a
 * pose climb's days with what they still lack (`short`, slice 47: the card's and the ladder page's line under the
 * climb, Tree.lua Tree.ClimbHead), naming the climb's `mainPose` (the main pose the pumps bring for the step, else the
 * step's anchor, as Tree.PoseShort; 2026-10-20); the climb's row under the card's To step up (`up`, 2026-10-29: a
 * strength climb's line with no "Next:" and no full stop, a counted cardio climb's "Clean days"), its count and pips at
 * its end. */
export function climbWords(w, climb, book, proving = null) {
	const hold = (move) => holds(w, move);
	const days = {have: climb.have ?? 0, need: climb.need ?? 0};
	switch (climb.kind) {
		case "unplaced": return {line: t("ladders.step.notplaced")};
		case "strength": {
			const words = {days, stages: climb.path ?? null, then: thenOf(w, climb, book)};
			if (climb.move && climb.top) {
				const said = {move: moveName(w, climb.move), goal: climbGoal(climb.top, hold(climb.move), climb),
					days: climb.need};
				words.line = t("ladders.step.next", said);
				words.up = t("ladders.card.up.next", said);
			}
			return words;
		}
		case "mark": {
			// any goal pose of the step the ladder stands on (the step's below) counts the family's day (slice 47),
			// answered close enough, or Perfect where only Perfect counts (the climb's `ask`, ns.MarkNeeds); the line
			// names the goal pose of the step's main pose (ns.MainPose), and under it what the days still lack: the days
			// on the main pose the pumps bring (`mainPose`, ns.PoseMain, else the step's anchor) and the Perfect day
			// (ns.PoseNeeds' mainLeft and perfectLeft)
			const mark = climb.move ? w.moves.get(climb.move)?.mark : null;
			const key = climb.ask === PERFECT ? "ladders.step.next.mark.perfect" : "ladders.step.next.mark";
			return {line: mark ? t(key, {days: climb.need, mark}) : null, mark: Boolean(mark), days,
				short: poseShort(climb, climb.mainPose != null ? moveName(w, climb.mainPose) : null),
				stages: climb.path ?? null, then: thenOf(w, climb, book)};
		}
		case "clean":
			if (proving) return {line: provingWords(w, proving), clean: true, stages: climb.path ?? null};
			return {line: t("ladders.step.next.clean", dayCount(climb.have, climb.need)), days, clean: true,
				up: t("ladders.card.up.clean"), stages: climb.path ?? null, then: thenOf(w, climb, book)};
		case "ready":
			return {line: t("ladders.step.next.full", dayCount(climb.have, climb.need)), days, full: true};
		default: return {};
	}
}

/** The proving window's line (slice 53, the Book row's `proving`, ns.Proving): "Next: Tuck burpee, a clean 30-s all-out
 * set."
 * while it is due or next, "No step up yet." after a try that did not pass this session. */
export function provingWords(w, proving) {
	return proving.kind === "waits" ? t("pump.climb.again")
		: t("ladders.step.next.prove", {move: moveName(w, proving.move)});
}

/** What reaching a step of the Path takes, in the goal's grammar, with its days and its path's stages (the climb from
 * the step under it: a card's rows on a step past the next, slice 42); a step the setup shuts, or one that waits for a
 * step of another ladder, says what holds it instead. */
export function needWords(w, st) {
	if (st.shut) {
		const state = st.shut;
		if (state.kind === "off") return {line: t("ladders.step.off", {move: moveName(w, state.move)})};
		if (state.kind === "waits" || state.kind === "waitsStep") return {line: waitsWords(w, state)};
		return {line: state.gap ? gapWords(w, state.gap) : t("ladders.step.shut")};
	}
	const n = st.need;
	const hold = (move) => holds(w, move);
	switch (n.kind) {
		case "unplaced": return {line: t("ladders.step.notplaced")};
		case "strength": {
			const words = {days: n.days, stages: n.path ?? null};
			if (n.move && n.top) {
				words.line = t("ladders.path.need", {move: moveName(w, n.move), goal: climbGoal(n.top, hold(n.move), n),
					days: n.days});
			}
			return words;
		}
		case "mark": {
			// any goal pose of the step under it answered close enough (Perfect alone below a risky step; slice 47), the
			// line naming the goal pose of that step's main pose
			const mark = n.move ? w.moves.get(n.move)?.mark : null;
			const key = n.ask === PERFECT ? "ladders.path.need.mark.perfect" : "ladders.path.need.mark";
			return {line: mark ? t(key, {days: n.days, mark}) : null, days: n.days, stages: n.path ?? null};
		}
		case "clean":
			return {line: t("ladders.path.need.clean", {need: n.days}), clean: true, stages: n.path ?? null};
		case "ready": return {line: t("ladders.path.need.full", {need: n.days}), full: true};
		default: return {};
	}
}

// The talent trees (slice 38) ---------------------------------------------------------------------------------------

/** A ladder as the trees read it: its words (`book.words`), its Book row, its steps by their step on the one scale,
 * the player's step on it (the step the Book marks now), its rung and its top rung (the highest that holds a move), and
 * the Book it was read from (its moves' checks, which a held next stage's words read); none for a ladder the words or
 * the Book lack. */
export function ladderOf(w, book, id) {
	const words = w.ladders.get(id);
	const row = book.ladders.find((l) => l.ladder === id);
	if (!words || !row) return null;
	const byStep = new Map(row.steps.map((s) => [s.step, s]));
	const now = row.steps.find((s) => s.state.kind === "now") ?? null;
	const top = Math.max(0, ...row.steps.filter((s) => s.rung && s.move).map((s) => s.rung));
	return {id, words, row, byStep, now, cur: now?.rung ?? row.rung, top, book};
}

/** The Book's step on ladder `lad` at rung `rung`, none past its rungs. */
export function stepAt(lad, rung) {
	return lad.row.steps.find((s) => s.rung === rung) ?? null;
}

/** A step's place on its ladder as the board draws it: `waiting` (the ladder waits whole), `now` (the player's step),
 * `top` (the player's step is the ladder's top), `yours` (a step the player has below it), `kept` (one they reached
 * before a step down, above it), `next` (the step above the player's) or `ahead`; none for a step no rung stands on. */
export function kindOf(lad, st) {
	if (!st?.rung || !st.move) return null;
	if (lad.row.waits) return "waiting";
	if (st.state.kind === "now") return lad.cur >= lad.top ? "top" : "now";
	if (st.rung < lad.cur) return "yours";
	if (st.reached) return "kept";
	return st.rung === lad.cur + 1 ? "next" : "ahead";
}

/** A strength ladder the player has not placed yet (its climb waits for the first max set); never one back from a
 * break, whose first set finds the step again (`placing`). */
export function unplaced(lad) {
	if (lad.row.placing) return false;
	const next = stepAt(lad, lad.cur + 1);
	return next?.state.climb?.kind === "unplaced" || lad.now?.start?.as === "place";
}

/** What finds the step of a ladder back from a break again (Tree.lua `Tree.PlacingLine`): a strength ladder's next
 * set, the cardio ladder's next all-out window or steady move, whichever measures its step (the Book's `placingBy`). */
export function placingLine(lad) {
	if (lad.row.placingBy === "window") return t("ladders.measure.placing.window");
	if (lad.row.placingBy === "steady") return t("ladders.measure.placing.steady");
	return t("ladders.measure.placing");
}

/** Where the player stands on a step, as its spoken name says it. */
function standingWords(lad, kind) {
	switch (kind) {
		case "now":
		case "top": return t(unplaced(lad) ? "ladders.standing.notplaced" : "ladders.standing.now");
		case "yours": return t("ladders.standing.yours");
		case "kept": return t("ladders.standing.kept");
		case "next": return t("ladders.standing.next");
		default: return t("ladders.standing.ahead");
	}
}

/** Whether move `move` keeps its count in seconds (a hold). */
function holds(w, move) {
	return w.moves.get(move)?.dose === "hold";
}

/** The window of `move` at rung `rung` of the ladder, {lo, hi}: its own, else the rung's anchor's where the move counts
 * in the anchor's unit (Progress.lua ns.OwnUnit: reps beside reps, seconds beside seconds); none where the rung has
 * none (cardio, a pose family) or the move counts in another unit with no window of its own there. */
export function windowOf(w, lad, rung, move) {
	const win = lad.words.windows[rung - 1];
	if (!win) return null;
	if (win.alt?.[move]) return win.alt[move];
	if (win.lo == null || win.hi == null) return null;
	const unit = (id) => w.moves.get(id)?.dose;
	return win.anchor == null || unit(win.anchor) === unit(move) ? {lo: win.lo, hi: win.hi} : null;
}

/** The move standing for rung `rung` of the ladder, as its steps name it. */
function rungMove(lad, rung) {
	return stepAt(lad, rung)?.move ?? null;
}

/** A line with its spoken form and its pips, as a step's key line, a card line or a refusal carries it. */
function said(text, more = {}) {
	return {text, spoken: more.spoken ?? text, pips: more.pips ?? null};
}

/** A record of `record` in its window, "Record 22 · Range 20-30", and in words for a step's spoken name. */
function recordLine(record, win, hold) {
	return said(t("ladders.measure.record", {record: amount(record, hold), low: amount(win.lo, hold),
		high: amount(win.hi, hold)}), {spoken: t("ladders.tree.spoken.record", {record: amount(record, hold),
		high: amount(win.hi, hold)})});
}

/** What holds a climb that cannot move now, by its step's state (the step above the player's): the setup's rule that
 * shuts it, the move the player left out, the step of another ladder it waits for, a pain flag; none for a climb
 * open. The hover's key line and the player's step's card say it alike. */
export function closedLine(w, st) {
	const s = st?.state;
	switch (s?.kind) {
		case "shut": return said(s.gap ? setupLine(w, s.gap, st.step) : t("ladders.step.shut"));
		case "off": return said(t("ladders.refuse.off", {move: moveName(w, s.move)}));
		case "waits":
		case "waitsStep": return said(waitsWords(w, s));
		case "pain": return said(t("ladders.step.pain"));
		default: return null;
	}
}

/** A setup refusal's words by its gap's kind, on the card's own ladder and on another (led by that ladder's name). */
const SETUP_LINES = {
	gear: {own: "ladders.refuse.setup.gear", other: "ladders.refuse.other.gear"},
	off: {own: "ladders.refuse.setup.off", other: "ladders.refuse.other.off"},
	on: {own: "ladders.refuse.setup.on", other: "ladders.refuse.other.on"},
};

/** Your setup's rule that shuts step `n` (of another ladder, `ladder`, where one is named), before its link. */
function setupLine(w, gap, n, ladder = null) {
	const words = w.words.gear[gap.key] ?? gap.key;
	const keys = SETUP_LINES[gap.kind] ?? SETUP_LINES.on;
	const what = gap.kind === "gear" ? {gear: words} : {setting: words};
	return ladder ? t(keys.other, {ladder, n, ...what}) : t(keys.own, {n, ...what});
}

/** What a step's own refused Start says on its Tooltip (Tree.lua `Tree.HeldLine`'s first part): a pain flag's line,
 * else its refusal's line by the level that stops it; none for a Start that runs. */
function refusedLine(w, lad, start) {
	if (!start?.refused) return null;
	if (start.refused === "pain") return said(t("ladders.step.pain"));
	const line = refusalOf(w, lad, start).lines[0];
	return line ? said(line) : null;
}

/** What holds a step that is not the player's, before its own line (Tree.lua `Tree.HeldLine` and `Tree.KeyLines`' shut
 * climb): its own Start refused, a pain flag on the ladder (the Book's `pain`, ns.TreeEntry's) on a step above the
 * player's, or on the step right above it what holds its climb (`closedLine`); none where nothing holds it. */
function heldLine(w, lad, st) {
	const refused = refusedLine(w, lad, st.start);
	if (refused) return refused;
	if (lad.row.pain && st.rung > lad.cur) return said(t("ladders.step.pain"));
	return st.rung === lad.cur + 1 ? closedLine(w, st) : null;
}

/** The key line of the player's step, in its own measure and in the small-area framing (Tree.lua `Tree.NowKey`): the
 * placement, the break, the top, its own Start's refusal (`Tree.HeldLine`), a pain flag and what earns an orange or
 * gold step its climb only opened (the card's words, `earnWords`); while its climb is shut below the top its record on
 * its range and what holds the climb (`Tree.ClosedKey`); then a strength or hold step's record up its window while
 * under half of it, what is left past it, then its days at the top and the stage of its path still to do; a counted
 * cardio step's clean days (slice 42; `Tree.NowCardio`), with its days in the stage still to do, and with its path in
 * too its proving window or the climb that waits (slice 53); a timed one's full moves (slice 52); a pose step's days,
 * with its days in the stage still to do. */
function nowLine(w, lad) {
	const next = stepAt(lad, lad.cur + 1);
	const climb = next?.state.kind === "next" ? next.state.climb : null;
	const ring = lad.row.ring;
	if (lad.row.placing) return said(placingLine(lad));
	if (unplaced(lad)) return said(t("ladders.measure.notplaced"));
	if (lad.cur >= lad.top) return said(t("ladders.path.top"));
	const refused = refusedLine(w, lad, lad.now?.start);
	if (refused) return refused;
	if (lad.row.pain || next?.state.kind === "pain") return said(t("ladders.step.pain"));
	const earn = lad.now?.state.earn;
	if (earn) return said(earnWords(w, earn));
	const held = climb ? null : closedLine(w, next);
	if (held) {
		const record = recordKey(w, lad);
		return record ? said(t("ladders.measure.held", {record: record.text, holds: held.text}),
			{spoken: t("ladders.tree.spoken.held", {record: record.spoken, holds: lowerFirst(held.spoken)})}) : held;
	}
	const pips = (have, need) => ({have: Math.min(have, need), need});
	switch (lad.words.dose) {
		case "pose": {
			if (!climb) return closedLine(w, next) ?? said("");
			const then = thenOf(w, climb, lad.book);
			if (then) return said(then);
			const key = climb.ask === PERFECT ? "ladders.measure.perfect" : "kit.days.label";
			return said(t(key, dayCount(climb.have, climb.need)), {pips: pips(climb.have, climb.need)});
		}
		case "cardio": {
			// the days as words but the days it takes: a plural of two numbers is one the phones' catalogs cannot number
			if (climb?.kind === "ready") {
				return said(t("ladders.measure.full", dayCount(climb.have, climb.need)), {pips: pips(climb.have, climb.need)});
			}
			if (climb?.kind === "clean") {
				// its proving window once its days and path are in (slice 53, Tree.lua Tree.NowCardio)
				if (lad.row.proving) return said(provingWords(w, lad.row.proving));
				const then = thenOf(w, climb, lad.book);
				if (then) return said(then);
				return said(t("ladders.measure.clean", dayCount(climb.have, climb.need)), {pips: pips(climb.have, climb.need)});
			}
			return closedLine(w, next) ?? said("");
		}
		default: {
			if (ring?.fill == null) return said(t("ladders.measure.first"));
			const hold = holds(w, ring.move);
			const win = windowOf(w, lad, lad.cur, ring.move);
			const record = ring.record ?? 0;
			const left = climb?.kind === "strength" ? climb.need - climb.have : 0;
			if (win && record < win.hi) {
				if (record * 2 >= win.lo + win.hi && left > 0) {
					return said(t("ladders.measure.left", {left: amount(win.hi - record, hold), days: left}));
				}
				return recordLine(record, win, hold);
			}
			if (climb?.kind === "strength" && left > 0) {
				return said(t("ladders.measure.counteddays", dayCount(climb.have, climb.need)),
					{pips: pips(climb.have, climb.need)});
			}
			const then = climb?.kind === "strength" ? thenOf(w, climb, lad.book) : null;
			if (then) return said(then);
			return closedLine(w, next) ?? (win ? recordLine(record, win, hold) : said(""));
		}
	}
}

/** The player's step's record on its range (Tree.lua `Tree.RecordKey`): a strength or hold step's in the window of the
 * move standing for it; none with no record or no range (a pose, a cardio step, whose counted forms' counts have no
 * window since slice 42). */
function recordKey(w, lad) {
	const ring = lad.row.ring;
	if (!ring?.move || !(ring.record > 0)) return null;
	if (lad.words.dose === "pose" || lad.words.dose === "cardio") return null;
	const win = windowOf(w, lad, lad.cur, ring.move);
	return win ? recordLine(ring.record, win, holds(w, ring.move)) : null;
}

/** The keys of what opens the step above the player's (`next`) and of what brings a kept one back there (`back`), by
 * the climb's measure (Tree.lua `Tree.OpensLine`'s "tree.key.next" and "tree.key.back" families; a timed cardio
 * climb's, kind `ready`, in full moves since slice 52). */
const OPENS_KEYS = {
	next: {strength: "ladders.measure.next", mark: "ladders.measure.next.mark", perfect: "ladders.measure.next.perfect",
		ready: "ladders.measure.next.full", clean: "ladders.measure.next.clean"},
	back: {strength: "ladders.measure.kept", mark: "ladders.measure.kept.mark", perfect: "ladders.measure.kept.perfect",
		ready: "ladders.measure.kept.full", clean: "ladders.measure.kept.clean"},
};

/** What the climb `c` from the player's step opens the step above at, or brings a kept one back at (`back`), in the
 * climb's own measure (Tree.lua `Tree.OpensLine`): a strength climb's move at its top on its days (opening, and the
 * stages of its path still left, slice 42), a pose's days close enough (Perfect days where only Perfect counts), a
 * timed cardio rung's full moves (slice 52), a counted one's clean days (slice 42); the opening line's day pips; none
 * where the climb names no move. */
function opensAt(w, lad, c, back) {
	const keys = OPENS_KEYS[back ? "back" : "next"];
	const pips = !back && c.need ? {have: Math.min(c.have ?? 0, c.need), need: c.need} : null;
	switch (c.kind) {
		case "strength": {
			if (!c.move || c.top == null) return null;
			const move = moveName(w, c.move);
			const g = climbGoal(c.top, holds(w, c.move), c);
			const left = back ? 0 : stagesLeft(c.path);
			if (left > 0) {
				return said(t("ladders.measure.next.path", {days: number(c.need), move, goal: g, count: left}), {pips});
			}
			return said(t(keys.strength, {days: c.need, move, goal: g}), {pips});
		}
		case "mark":
			return said(c.ask === PERFECT ? t(keys.perfect, {count: c.need}) : t(keys.mark, {days: c.need}), {pips});
		case "ready": return said(t(keys.ready, {need: c.need}), {pips});
		case "clean": return said(t(keys.clean, {need: c.need}), {pips});
		default: return null;
	}
}

/** What the step above the player's opens at (its climb's need, a pose's days, a cardio rung's full moves or clean
 * days), or what holds it. */
function nextLine(w, lad, st) {
	const s = st.state;
	if (s.kind !== "next") return closedLine(w, st) ?? said("");
	if (s.climb.kind === "unplaced") return said(t("ladders.measure.notplaced"));
	return opensAt(w, lad, s.climb, false)
		?? said(t("ladders.measure.ahead", {move: moveName(w, rungMove(lad, st.rung - 1))}));
}

/** A step's key line with its pips and its words for a screen reader (the Tooltip's line 3): by its place on its
 * ladder, from the Book's own fields; on every step but the player's what holds it first (`heldLine`). */
function keyLine(w, lad, st, kind) {
	if (kind === "now" || kind === "top") return nowLine(w, lad);
	if (kind === "waiting") return waitingSentence(w, lad);
	const held = heldLine(w, lad, st);
	if (held) return held;
	switch (kind) {
		case "yours": return said(reachedWords(st.reached));
		case "next": return nextLine(w, lad, st);
		case "kept": {
			// right above the player's step it comes back by the climb from there; higher up after the step before it
			const c = st.rung === lad.cur + 1 && st.state.kind === "next" ? st.state.climb : null;
			return (c && opensAt(w, lad, c, true))
				?? said(t("ladders.measure.kept.after", {move: moveName(w, rungMove(lad, st.rung - 1))}));
		}
		default: return said(t("ladders.measure.ahead", {move: moveName(w, rungMove(lad, st.rung - 1))}));
	}
}

/** A ladder that waits whole: its note under its name ("Not open yet", "Shut by your setup", "Left out of your
 * pumps"). */
export function waitingNote(lad) {
	const key = {closed: "ladders.tree.waits.closed", setup: "ladders.tree.waits.setup", off: "ladders.tree.waits.off"};
	return t(key[lad.row.waits] ?? key.closed);
}

/** The start a ladder that waits whole carries on its steps (its first step's): what its card's Start runs, a family
 * not open yet's climb that opens it, or the setup's or the player's own list's refusal. */
export function waitingStart(lad) {
	return lad.row.steps.filter((s) => s.rung).at(-1)?.start ?? null;
}

/** A ladder that waits whole: its sentence, what opens it ("Handstand opens at a Push-up record of 15.") or what of
 * the setup or the player's own list leaves it no move. */
function waitingSentence(w, lad) {
	if (lad.row.waits === "closed" || !lad.row.waits) return said(opensLine(w, lad) ?? "");
	const start = waitingStart(lad);
	const lines = start?.refused ? refusalOf(w, lad, start).lines : [];
	return said(lines[0] ?? t("ladders.step.shut"));
}

/** What opens a family not open yet, as the card says it: "{ladder} opens at a {move} record of {count}.", "{ladder}
 * opens at a Perfect {move}." for a pose's goal pose (slice 42's fix round, 2026-10-17: the planche's wrist lean, in the
 * check's words; "… a Good …" where Good does it) or "{ladder} opens at {opener} step {n}."; none for a family open from
 * the start. */
export function opensLine(w, lad) {
	const o = lad.row.opens;
	if (!o) return null;
	if (o.kind === "record") {
		return t("ladders.card.via.record", {ladder: lad.words.name, move: moveName(w, o.move),
			count: amount(o.count, o.seconds)});
	}
	if (o.kind === "pose") {
		return t(o.ask >= PERFECT ? "ladders.card.via.pose.perfect" : "ladders.card.via.pose",
			{ladder: lad.words.name, move: moveName(w, o.move)});
	}
	return t("ladders.card.via.step", {ladder: lad.words.name, opener: w.ladders.get(o.ladder)?.name ?? "",
		n: o.step});
}


/** Where the player stands on a step with its mastery (slice 42), as its spoken name says it: "mastered" after the
 * standing on a step mastered. */
function standingOf(lad, st, kind) {
	const standing = standingWords(lad, kind);
	if (!st.mastered) return standing;
	return `${standing}, ${t("ladders.tree.mastered")}`;
}

/** The beads of a ringed step in words, after its key line in its spoken name (slice 42): its path's stages done (or
 * skipped) of them all; none on a step whose climb has no path, or off the player's step. */
function beadsWords(lad, st, kind) {
	const stages = beadsOf(lad, st, kind);
	if (!stages) return null;
	const done = stages.filter((s) => s.state === "done" || s.state === "skipped").length;
	return t("ladders.tree.spoken.beads", {done: number(done), count: stages.length});
}

/** The stages a ringed step's beads show (slice 42): the player's step's own path, under its ring (none while the
 * ladder waits whole, is back from a break or has no ring, and none on the top step). */
export function beadsOf(lad, st, kind) {
	if (kind !== "now" || !lad.row.ring || lad.row.placing) return null;
	return st.stages?.length ? st.stages : null;
}

/** A step as a path's row and Up next show it: its move, its key line with its pips, and its spoken name (the beads'
 * words after its key line); a ladder that waits whole's by its name, its sentence as its key line. */
export function stepTip(w, lad, st, kind) {
	if (kind === "waiting") {
		const sentence = waitingSentence(w, lad).text;
		const note = waitingNote(lad);
		// spoken after its name: what opens it without the ladder's name again ("opens at a Push-up record of 15")
		const said = lad.row.opens ? opensWords(w, lad.row.opens) : sentence;
		return {name: lad.words.name, key: sentence, pips: null,
			spoken: t("ladders.tree.spoken.waiting", {ladder: lad.words.name, note: lowerFirst(note),
				sentence: lowerFirst(said)})};
	}
	const name = moveName(w, st.move);
	const quality = qualityWord(w, st.quality);
	const line = keyLine(w, lad, st, kind);
	const beads = beadsWords(lad, st, kind);
	const spoken = lowerFirst(line.spoken);
	// a key line that ends a sentence ("… on Feet-elevated push-up.") runs on into the beads' words
	const said = beads ? LIST.format([spoken.replace(/\.$/, ""), beads].filter(Boolean)) : spoken;
	return {name, key: line.text, pips: line.pips, spoken: t("ladders.tree.spoken",
		{move: name, quality, n: st.step, standing: standingOf(lad, st, kind), line: said})};
}

/** A line's first letter in lower case, as it stands inside a spoken name. */
function lowerFirst(text) {
	return text ? text.charAt(0).toLocaleLowerCase(LOCALE) + text.slice(1) : "";
}

/** The Start labels by their verb: with the start's goal, a max set's, and a start that counts nothing with no
 * number. */
const LABELS = {
	start: {goal: "ladders.start", max: "ladders.start.max", plain: "ladders.start.plain"},
	climb: {goal: "ladders.start.climb", max: "ladders.start.climb.max", plain: "ladders.start.climb.plain"},
	back: {goal: "ladders.start.back", max: "ladders.start.back.max", plain: "ladders.start.back.plain"},
	earn: {goal: "ladders.start.earn", max: "ladders.start.earn.max", plain: "ladders.start.earn.plain"},
};

/** A label's middle dot with the spaces round it, and the same dot bound to its neighbours (no-break spaces), so a
 * label that wraps never leaves its goal or the dot alone on a line. */
const DOT = /\s·\s/g;
const BOUND_DOT = "\u00a0·\u00a0";

/** A Start's label from the start's own fields (ns.StartFor's `as`, `goal` and `allout`; Tree.lua Tree.StartLabel's
 * rule, every surface's): verb, move and goal ("Climb with Push-up · 28+"); with no goal Max set where the move keeps
 * a count (a rep move or a hold, which the engine doses a goal for once there is a record); a place start "Find my step"
 * alone, the move it runs beside it (2026-10-29); an
 * all-out window, counted or not, all out on its own or a held step's start (no number since slice 42) and its plain
 * label under a climb, back or earn verb; a stretch check "Do the check" (slice 42); else the move alone (a timed
 * move, a pose); none for a refused start. */
export function startLabel(w, start) {
	if (!start?.move || start.refused) return null;
	if (start.as === "check") return t("ladders.start.check");
	const m = w.moves.get(start.move);
	const move = m?.name ?? "";
	const verb = start.back ? "back" : start.as === "climb" || start.as === "path" ? "climb"
		: start.as === "earn" ? "earn" : "start";
	const keys = LABELS[verb];
	let label;
	if (start.as === "place") label = t("ladders.start.place");
	else if (start.goal != null) label = t(keys.goal, {move, goal: goal(start.goal, m?.dose === "hold")});
	else if (start.allout) label = verb === "start" ? t("ladders.start.allout", {move}) : t(keys.plain, {move});
	else if (m?.dose === "rep" || m?.dose === "hold") label = t(keys.max, {move});
	else label = t(keys.plain, {move});
	return label.replace(DOT, BOUND_DOT);
}

/** The card's lines beside a Start (docs/ui.md's "The Start label" and "The card's other lines"): a stretch check's
 * what one hold at its goal pose opens (slice 42: `opens`, the move waiting on it or the climb to its step, as the
 * card or window that shows it knows), what earns an orange or gold step and when, what opens the waiting move it
 * runs toward (slice 42), when its day counts again, when a set is fresh again, the tiredness, and where the game's
 * sheet stands ahead of this computer's own. `lad` the ladder of the step or move the card is for, `own` the move a
 * move's window shows (a `via` path line names its stage). */
export function startLines(w, book, lad, start, own = null, opens = null) {
	if (!start?.move || start.refused) return [];
	if (start.as === "check") {
		if (!opens) return [];
		const perfect = start.ask >= PERFECT;
		const lines = [opens.step != null
			? t(perfect ? "ladders.card.check.line.step.perfect" : "ladders.card.check.line.step", {n: opens.step})
			: t(perfect ? "ladders.card.check.line.perfect" : "ladders.card.check.line", {move: moveName(w, opens.move)})];
		// a move its stage of a step's path holds too says that beside the check's line (slice 42's fix round,
		// 2026-10-17: ns.PathWaits, as the game's move window)
		const held = own != null ? factsOf(book).get(own)?.pathWaits : null;
		const line = held?.after != null ? opensAfter(w, held) : null;
		if (line) lines.push(line);
		return lines;
	}
	const m = w.moves.get(start.move);
	// a pose family's step is earned by a Perfect hold, told by the ladder the start runs (no move's dose says pose)
	const pose = (start.ladder != null ? w.ladders.get(start.ladder) : lad?.words)?.dose === "pose";
	const lines = [];
	// an earn line that names the time says when it counts: no day line and no fresh line after it
	let timed = false;
	if (start.as === "earn") {
		lines.push(pose ? t("ladders.card.earn.mark")
			: t("ladders.card.earn", {earn: amount(start.earn, m?.dose === "hold")}));
	} else if (start.earn != null || (pose && lad?.now?.state.earn && start.ladder === lad.id)) {
		// a step to earn whose set cannot earn it yet: not fresh, or its pose held today
		if (pose && start.dayIn) {
			lines.push(t("ladders.card.earn.from", {time: time(start.dayIn * 1000)}));
			timed = true;
		} else if (!pose && start.freshFrom) {
			lines.push(t("ladders.card.earn.fresh", {time: time(start.freshFrom * 1000),
				earn: amount(start.earn, m?.dose === "hold")}));
			timed = true;
		}
	}
	if (start.via === "opens" && lad) {
		const opens = opensLine(w, lad);
		if (opens) lines.push(opens);
	} else if (start.via === "path" && own != null) {
		// a move that waits for its stage of a path (slice 42): the move whose top opens it, as its stage row says it
		const waits = factsOf(book).get(own)?.waits;
		if (waits?.kind === "path") {
			const line = waits.after != null ? opensAfter(w, waits) : null;
			if (line) lines.push(line);
		}
	} else if (start.via === "needs" && lad) {
		const waiting = stepAt(lad, lad.cur + 1);
		if (waiting && (waiting.state.kind === "waits" || waiting.state.kind === "waitsStep")) {
			lines.push(waitsWords(w, waiting.state));
		}
	}
	if (!timed && start.dayKind === "session") lines.push(t("ladders.card.day"));
	else if (!timed && start.dayKind === "day" && start.dayIn) {
		lines.push(t("ladders.card.day.next", {time: time(start.dayIn * 1000)}));
	}
	if (!timed && start.freshFrom && start.as !== "earn" && start.earn == null) {
		lines.push(t("ladders.card.fresh", {time: time(start.freshFrom * 1000)}));
	}
	if (start.tired) lines.push(t("ladders.card.tired"));
	const app = start.ladder != null ? book.ladders.find((l) => l.ladder === start.ladder) : null;
	if (app?.appRung && start.rung === app.appRung) {
		const step = w.ladders.get(app.ladder)?.steps[app.appRung - 1] ?? app.appRung;
		lines.push(t("ladders.card.behind.computer", {ladder: w.ladders.get(app.ladder)?.name ?? "", n: step}));
	}
	return lines;
}

/** A refused Start's lines by its subject (the step or level that stops it, on this ladder or another) and the rows
 * that open what to change: Your setup at its gap's switch or piece, that move's Include in pumps (its page), or Your
 * setup's pain switches. `move` a move's window or page, whose pain line names the move. Each link {heading, glyph,
 * kind, move, focus}: `focus` what Your setup opens at (the gap's key, or "pain"). */
export function refusalOf(w, lad, start, move = false) {
	const lines = [];
	const links = [];
	const other = start.ladder != null && start.ladder !== lad?.id ? w.ladders.get(start.ladder) : null;
	const n = start.rung != null ? (w.ladders.get(start.ladder)?.steps[start.rung - 1] ?? start.rung) : null;
	const setupLink = {heading: t("menu.setup"), glyph: "action_character", kind: "setup",
		focus: start.refused === "pain" ? "pain" : start.gap?.key ?? null};
	// the move the player's own list leaves out, by its name: a move's window shows its own Include in pumps beside it
	const includeLink = {heading: t("ladders.refuse.include", {move: moveName(w, start.move)}), glyph: "action_pump",
		kind: "include", move: start.move};
	switch (start.refused) {
		case "setup":
			lines.push(start.gap && n != null ? setupLine(w, start.gap, n, other?.name ?? null) : t("ladders.step.shut"));
			links.push(setupLink);
			if (start.off) {
				lines.push(t("ladders.refuse.off.too", {move: moveName(w, start.move)}));
				links.push(includeLink);
			}
			break;
		case "off":
			lines.push(t("ladders.refuse.off", {move: moveName(w, start.move)}));
			links.push(includeLink);
			break;
		case "pain":
			lines.push(t(move ? "ladders.refuse.pain.move" : "ladders.refuse.pain"));
			links.push(setupLink);
			break;
		default:
			break;
	}
	return {lines, links};
}

/** The reason a Start that can run is greyed now, by the `pump` topic's gate: felt unwell locks every pump (until its
 * end, in the tree's words, as the game's greyed Start says it), the game asks the pumps while it runs, or the engine
 * is folding (the button's busy state, no words); none while nothing holds it. */
export function heldBy(pump) {
	switch (pump?.gate) {
		case "lock": return {state: "disabled", note: pump.locked_until != null
			? t("ladders.tree.rest", {time: time(pump.locked_until)}) : null, reason: null};
		case "game": return {state: "disabled", note: t("pump.waiting.game"), reason: "game"};
		case "engine": return {state: "busy", note: null, reason: "reading"};
		default: return null;
	}
}

/** A tree's Next line under its name (docs/ui.md's "The tree's lines"): the glowing ladder's next step as its ladder's
 * page words it (the stage of its path still to do once the days are in, slice 42; the fresh set that earns a step
 * still to earn), the Trial's first set, or why nothing glows. `pump` the pump topic (the lock's end). */
export function treeNext(w, book, tree, pump) {
	if (tree.glow) {
		const lad = ladderOf(w, book, tree.glow);
		if (!lad) return "";
		if (tree.kind === "trial") {
			return t("ladders.tree.next.trial", {move: moveName(w, lad.now?.move ?? lad.row.move)});
		}
		const earn = lad.now?.state.earn;
		if (earn?.mark) return t("ladders.tree.next.earn.mark", {move: moveName(w, lad.now.move)});
		if (earn) return t("ladders.tree.next.earn", {move: moveName(w, earn.move), earn: amount(earn.count, earn.seconds)});
		const next = stepAt(lad, lad.cur + 1);
		const c = next?.state.kind === "next" ? next.state.climb : null;
		if (!c) return "";
		// a counted cardio climb's proving window first (slice 53, Tree.lua Tree.NextCardio): its days and path are in
		if (c.kind === "clean" && lad.row.proving) return provingWords(w, lad.row.proving);
		// once the days are in, the path's next stage as the engine names it (ns.PathNext), or its move and what holds it
		const n = c.need > 0 && c.have >= c.need ? c.next : null;
		if (!n) return climbWords(w, c, book).line ?? "";
		const held = heldWords(w, book, n);
		if (held) return t("ladders.tree.next.proof.held", {proof: moveName(w, n.move), wait: held});
		return thenWords(w, c.path, n, THEN_KEYS.tree);
	}
	switch (tree.reason) {
		// the lock's end is the pump topic's (the desktop's own lock, which every row's felt-unwell stop sets)
		case "rest": return pump?.locked_until != null ? t("ladders.tree.rest", {time: time(pump.locked_until)}) : "";
		case "day": return t("ladders.tree.day", {time: time((tree.at ?? 0) * 1000)});
		case "pain": return t("ladders.tree.pain");
		case "break": return t("ladders.tree.break");
		default: return t("ladders.tree.none");
	}
}

/** The steps a link joins (`book.words` links: into a ladder's step from another ladder's), each way from step `n` of
 * ladder `id`: the steps it opens (out) and the steps it waits for (in), each {ladder, step, link}, an `any` group's
 * (slice 42) one for each of its steps (`entry`, the step as the library names it: a check's target), the group's place
 * among the ladder's links its `group`; a step it waits for
 * with the engine's reading of its link (`need`, `book`'s `needs` of ladder `id`: `done`, ns.NeedDone, the `move` and
 * `count` that do it, ns.NeedAt, and a stretch check's `check` and `at`; none where the engine did not answer, and
 * never a test or a lookup of the page's own). */
export function linksOf(w, id, n, book = null) {
	const out = [];
	const into = [];
	const needs = book?.ladders.find((l) => l.ladder === id)?.needs ?? null;
	for (const l of w.words.ladders) {
		l.links.forEach((link, i) => {
			const steps = link.any ?? [link];
			for (const s of steps) if (s.ladder === id && s.step === n) out.push({ladder: l.id, step: link.into, link});
			if (l.id === id && link.into === n) {
				for (const s of steps) {
					into.push({ladder: s.ladder, step: s.step, entry: s, link, group: i, need: needs?.[i] ?? null});
				}
			}
		});
	}
	return {out, into};
}

/** A need's own fields as `ns.StartFor` takes a check's target (`{check}`) and the engine names it: its ladder, step,
 * move, count and goal pose's mark, none of a link's own. */
export function needOf(entry) {
	const {ladder, step, count, move, mark} = entry;
	return {ladder, step, count, move, mark};
}

/** What a step of another ladder a climb waits for still waits for (a link into it, `linksOf`'s `into`), as its row's
 * meta says it in the game's words from the engine's ns.NeedAt: "Waits for a record of 8 on Pull-up." where a record
 * does it, "Waits until you do Twisted chair on your Twist ladder." on a pose or cardio step (an `any` group's step the
 * setup offers, by its move's ladder); none once it is done or where the engine did not answer. */
export function needLine(w, into) {
	const need = into.need;
	if (!need || need.done || need.move == null) return null;
	return waitsWords(w, need.count != null
		? {kind: "waits", move: need.move, count: need.count, seconds: holds(w, need.move)}
		: {kind: "waitsStep", move: need.move, ladder: w.moves.get(need.move)?.ladder ?? into.ladder});
}

/** A step of another ladder as a cross-tree chip and the card's button name it: "Push-up · step 6 · Strength". */
export function chipWords(w, book, ladder, step) {
	const lad = ladderOf(w, book, ladder);
	const st = lad?.byStep.get(step);
	const tree = w.words.trees.find((tr) => tr.branches.some((b) => b.ladders.includes(ladder)));
	return t("ladders.tree.chip", {move: moveName(w, st?.move ?? lad?.row.move), n: step, tree: tree?.name ?? ""});
}

