// Numbers and times in the player's own conventions (Intl, the system's region) in the copy's English: the words around
// them come from lib/copy.js, which groups an int's digits itself.

/** The locale every date, weekday, relative time, list and number the page writes is in (the phones' `copyLocale`):
 * English, the copy's one language, with the computer's region (where its language names none the likely one) and its
 * language's hour cycle, so a computer set to German reads "Tue 16:00" and "29 Oct" beside the English words, never
 * "Di., 16:00" (the Pixel 8a's test, 2026-10-29); the computer's own tag where it is in English already. */
export const LOCALE = copyLocale(navigator.language);

function copyLocale(tag) {
	try {
		const device = new Intl.Locale(tag);
		if (device.language === "en") return device.toString();
		const {hourCycle} = new Intl.DateTimeFormat(tag, {hour: "numeric"}).resolvedOptions();
		return new Intl.Locale("en", {region: device.maximize().region, hourCycle}).toString();
	} catch {
		// no tag, or one Intl refuses: English as it stands
		return "en";
	}
}

const TIME = new Intl.DateTimeFormat(LOCALE, {timeStyle: "short"});

/** A moment's time of day (unix ms), such as 14:02 or 2:02 PM. */
export function time(ms) {
	return TIME.format(new Date(ms));
}

/** Seconds as a clock shows them, m:ss, to the nearest second (a hold's time, how long a move took). */
export function clock(seconds) {
	const s = Math.max(0, Math.round(seconds));
	return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, "0")}`;
}

const KG = new Intl.NumberFormat(LOCALE, {maximumFractionDigits: 1});

/** A weight kept in tenths of a kg, in kg with its one decimal, the decimal dropped when 0: 125 is 12.5, 120 is 12. */
export function kg(tenths) {
	return KG.format(tenths / 10);
}
