// Hero's words that the page puts together from the `hero` topic (docs/plan.md slice 35: the page formats only): the
// hero's name with its title in the game's grammar (the cores' `titledParts`), a record as a count or a hold's clock,
// how long ago the game folded its sheet, a muscle's name, and what opens a family not open yet.

import {t} from "../../lib/copy.js";
import {clock, LOCALE} from "../../lib/format.js";

const NUMBER = new Intl.NumberFormat(LOCALE);
const AGO = new Intl.RelativeTimeFormat(LOCALE, {numeric: "auto", style: "short"});

/** A form of the game's grammar ("%s, %s") filled in turn. */
export function fill(form, ...parts) {
	let i = 0;
	return form.replace(/%s/g, () => parts[i++] ?? "");
}

/** The hero's name with its title (`SheetVocab.Words.titled` and `titledParts`): "Mirelle, Hand of Iron", but "Mirelle
 * the Relentless" for a title in lower case; cut at the grammar's seam into `first` and `second`. The sheet's name, else
 * "Your hero". */
export function nameParts(name, title, grammar) {
	const who = name ?? t("character.name.fallback");
	const form = title ? (title[0] === title[0].toLowerCase() ? grammar.suffix : grammar.titled) : null;
	if (!form) return {whole: who, first: who, second: null};
	const seam = form.indexOf("%s", form.indexOf("%s") + 2);
	if (seam < 0) {
		const whole = fill(form, who, title);
		return {whole, first: whole, second: null};
	}
	return {whole: fill(form, who, title), first: fill(form.slice(0, seam), who).trim(),
		second: fill(form.slice(seam), title).trim()};
}

/** A record as the tooltip and the ladder lines say it: the count, or a hold's seconds as a clock (m:ss). */
export function recordText(value, seconds) {
	return seconds ? clock(value) : NUMBER.format(value);
}

/** How long ago a unix second was, in the system's words ("5 min. ago"; "now" under a minute). */
export function ago(at, nowMs) {
	const s = Math.max(0, Math.round(nowMs / 1000 - at));
	if (s < 60) return AGO.format(0, "second");
	if (s < 3600) return AGO.format(-Math.floor(s / 60), "minute");
	if (s < 86400) return AGO.format(-Math.floor(s / 3600), "hour");
	return AGO.format(-Math.floor(s / 86400), "day");
}

/** A muscle's name, or its number where the sheet names none. */
export function muscleName(muscle) {
	return muscle.name ?? t("character.muscle.fallback", {number: muscle.id});
}

/** What opens a family not open yet, as Book and Hero's pages say it: a record, a pose's goal pose in the check's words
 * ("Opens at a Perfect Wrist lean", slice 42's fix round, 2026-10-17), or a step of another ladder. */
export function openingWords(opens) {
	switch (opens.kind) {
		case "record": return t("ladders.opens.record", {move: opens.name, count: recordText(opens.count, opens.seconds)});
		case "pose": return t(opens.perfect ? "ladders.opens.pose.perfect" : "ladders.opens.pose", {move: opens.name});
		default: return t("ladders.opens", {ladder: opens.ladder, step: opens.step});
	}
}
