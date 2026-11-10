// A place's words on the computer (docs/plan.md slice 51, "Places"; contract/places.json): its name (the player's own,
// else its template's words, places.template.<id>), its glyph's name for a screen reader (places.glyph.<the glyph less
// place_>) and its meta line, the equipment's first three names and how many more (the phones' PlaceRow), from the
// `places` topic (helper/crates/questpump-desk/src/places.rs); what the place button and the place rows name (slice
// 67: Work mode while it runs, with work mode's own equipment); and a time of the places' schedule (2026-11-03): its
// days as its title in this computer's weekday names (format.js' LOCALE, Monday first, three or more in a row as a
// span), and its span and place as its meta line.

import {get} from "../../lib/bridge.js";
import {t} from "../../lib/copy.js";
import {LOCALE} from "../../lib/format.js";
import {clock} from "../menu/reminders.js";

/** The templates' words, by template id: literal keys, so the copy lint sees each read. */
const TEMPLATES = {
	home: () => t("places.template.home"),
	hotel: () => t("places.template.hotel"),
	gym: () => t("places.template.gym"),
	park: () => t("places.template.park"),
	office: () => t("places.template.office"),
	outdoors: () => t("places.template.outdoors"),
};

/** The glyphs' names, by art piece. */
const GLYPHS = {
	place_home: () => t("places.glyph.home"),
	place_bed: () => t("places.glyph.bed"),
	place_dumbbell: () => t("places.glyph.dumbbell"),
	place_bars: () => t("places.glyph.bars"),
	place_desk: () => t("places.glyph.desk"),
	place_peak: () => t("places.glyph.peak"),
};

/** How many pieces of equipment the meta line names before "+ N more". */
const NAMED = 3;

/** Work mode's glyph: its Menu row's, and the place button's while it runs. */
export const WORK_GLYPH = "place_desk";

/** A place's name: the player's own, else its template's words (Home's always); Work mode for work mode's equipment
 * (`placeShown`, and a timed pump's place that began while work mode ran). */
export function placeName(place) {
	if (!place) return "";
	if (place.work) return t("work.title");
	if (place.name) return place.name;
	return TEMPLATES[place.template ?? place.id]?.() ?? "";
}

/** A template's words, by its id. */
export function templateName(id) {
	return TEMPLATES[id]?.() ?? "";
}

/** A glyph's name, for a screen reader. */
export function glyphName(glyph) {
	return GLYPHS[glyph]?.() ?? "";
}

/** A place's meta line: its equipment's first names in the game's order and how many more, or No equipment. */
export function gearLine(place) {
	const names = new Map((get("places")?.words?.gear ?? []).map((g) => [g.key, g.name]));
	const gear = (place?.gear ?? []).map((key) => names.get(key)).filter(Boolean);
	if (!gear.length) return t("menu.setup.nogear");
	const shown = gear.slice(0, NAMED).join(", ");
	return gear.length > NAMED ? `${shown} ${t("places.gear.more", {count: gear.length - NAMED})}` : shown;
}

/** The place in use from the `places` topic (Home while the list does not hold it). */
function inUse(places) {
	return places?.list.find((p) => p.id === places.in_use) ?? places?.list[0] ?? null;
}

/** What the place button and the place rows name from the `places` topic (slice 67): while work mode runs, work mode's
 * own equipment as a place (`work`, Work mode's words and glyph), which every pump then takes, else the place in use;
 * a click on one opens the Work mode page while `work` holds, else the places window. */
export function placeShown(places) {
	return places?.work ? {...places.work, work: true, glyph: WORK_GLYPH} : inUse(places);
}

/** The ISO weekdays, Monday first (1 Monday to 7 Sunday). */
export const WEEK = [1, 2, 3, 4, 5, 6, 7];

/** The weekday names' formats: the short name (a time's title), the narrow letter (a day's chip) and the full name. */
const WEEKDAYS = {
	short: new Intl.DateTimeFormat(LOCALE, {weekday: "short"}),
	narrow: new Intl.DateTimeFormat(LOCALE, {weekday: "narrow"}),
	long: new Intl.DateTimeFormat(LOCALE, {weekday: "long"}),
};

/** ISO weekday `day`'s name in `style` (short, narrow or long): 2024-01-01 was a Monday. */
export function weekday(day, style = "short") {
	return WEEKDAYS[style].format(new Date(2024, 0, day, 12));
}

/** A time's days as its title: the short names in the week's order, three or more in a row written as a span. */
export function daysLine(days) {
	const sorted = WEEK.filter((d) => days.includes(d));
	const parts = [];
	for (let i = 0; i < sorted.length;) {
		let j = i;
		while (j + 1 < sorted.length && sorted[j + 1] === sorted[j] + 1) j++;
		if (j - i >= 2) parts.push(`${weekday(sorted[i])}–${weekday(sorted[j])}`);
		else parts.push(...sorted.slice(i, j + 1).map((d) => weekday(d)));
		i = j + 1;
	}
	return parts.join(", ");
}

/** A time's meta line: its span in this computer's clock and its place's name (`place`, one of the topic's list). */
export function timeLine(time, place) {
	return t("places.time.meta", {from: clock(time.from), to: clock(time.to), place: placeName(place)});
}
