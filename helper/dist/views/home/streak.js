// The streak page on the computer (slice 51; docs/ui.md, "Away Mode", the streak page; the phones' pump.streak), from
// the StreakBoard and Hero's streak chip, drawn from the `streak` topic (helper/src-tauri/src/desk/streak.rs): the
// days as the hero number (text, never gold) over Day streak and A quest a day. (the board's resting words while felt
// unwell rests today), the standard-bearer at its key pose beside them in the colour
// the game draws the character in (`book`'s tint); the month in WeekDots' month state under its name, an earlier and
// a later month a step each on their RoundButtons and the arrow keys (back to the month the streak began in, none past
// this one); the rest tokens, `token_max` slots with the held ones' rest mark, how many are held, the active days to
// the next one and how one is earned; the next milestone in words over a bar from the milestone before it with the
// days to go (Every milestone reached past the last); the best streak; and Share streak (the Share window on the
// streak card as it stands; none at 0 days). Its words keep `words.json`'s `care` rule: nothing is lost, owed or due.

import {get} from "../../lib/bridge.js";
import {t} from "../../lib/copy.js";
import {register} from "../../lib/router.js";
import {openShare, streakCard} from "../../lib/share.js";
import {h, styled} from "../../kit/element.js";
import {milestoneWords} from "../../kit/qp-fanfare.js";
import {page, watch} from "../page.js";

/** The standard-bearer (contract/figures/figures.json `extras`). */
const BANNER = "streak_banner"; // copy-lint: not shown (a figure's name in figures.json)
/** Monday to Sunday's initials in the player's language (2026-10-05 is a Monday). */
const INITIALS = Array.from({length: 7}, (_, i) =>
	new Intl.DateTimeFormat(undefined, {weekday: "narrow", timeZone: "UTC"}).format(Date.UTC(2026, 9, 5 + i)));
const DAYS = new Intl.ListFormat(undefined, {type: "conjunction"});
const count = (n) => new Intl.NumberFormat().format(n);

/** A game month's name ("October", "October 2025" in another year). */
function monthName(m) {
	const year = m.year === new Date().getFullYear() ? undefined : "numeric";
	return new Intl.DateTimeFormat(undefined, {month: "long", year, timeZone: "UTC"})
		.format(Date.UTC(m.year, m.month - 1, 1));
}

/** A month's WeekDots cells: blank before its first, then each day's mark with its date. */
function cells(m) {
	const days = m.days.map((mark, i) => ({date: count(i + 1), done: mark === "done", rested: mark === "rested",
		resting: mark === "resting", today: i + 1 === m.today, future: m.today !== null && i + 1 > m.today}));
	return [...Array.from({length: m.lead}, () => null), ...days];
}

/** A month as a screen reader hears it: its name, the days pumped, then those a rest token kept and the resting. */
function monthSaid(m) {
	const date = (i) => new Intl.DateTimeFormat(undefined, {day: "numeric", month: "long", timeZone: "UTC"})
		.format(Date.UTC(m.year, m.month - 1, i + 1));
	const marked = (mark) => m.days.flatMap((d, i) => (d === mark ? [date(i)] : []));
	const [pumped, rested, resting] = ["done", "rested", "resting"].map(marked);
	const said = [monthName(m)];
	if (pumped.length) said.push(DAYS.format(pumped));
	if (rested.length) said.push(t("kit.weekdots.rested", {days: DAYS.format(rested)}));
	if (resting.length) said.push(t("kit.weekdots.resting", {days: DAYS.format(resting)}));
	return said.join(". ");
}

register("desk.streak", () => {
	const view = page({heading: t("pump.streak.title")});
	const number = h("p", {class: "qp-type-num-hero"});
	const line = h("p", {class: "qp-type-subtitle"});
	const figure = h("qp-figure-stage", {move: BANNER, still: true, label: t("pump.home.streak.label")});
	const hero = h("div", {class: "qp-streak-hero"},
		h("div", {class: "qp-streak-hero-words"}, number,
			h("p", {class: "qp-type-caps", text: t("pump.home.streak.label")}), line),
		figure);

	// the months, the earliest first, and how many before the last (this month) the one shown is
	let months = [];
	let back = 0;
	const earlier = h("qp-round-button", {icon: "earlier", "on:click": () => step(-1)});
	const later = h("qp-round-button", {icon: "later", "on:click": () => step(1)});
	const name = h("p", {class: "qp-type-caps", "aria-live": "polite"});
	const dots = h("qp-week-dots", {month: true});
	dots.initials = INITIALS;
	const month = h("qp-card", {class: "qp-streak-month"},
		h("div", {class: "qp-streak-month-head"}, earlier, name, later), dots);
	const drawMonth = () => {
		const m = months.at(-1 - back);
		if (!m) return;
		name.textContent = monthName(m);
		dots.label = monthSaid(m);
		dots.days = cells(m);
		earlier.inert = back === months.length - 1;
		later.inert = back === 0;
	};
	const step = (by) => {
		const next = Math.max(0, Math.min(months.length - 1, back - by));
		if (next === back) return;
		back = next;
		drawMonth();
	};
	// the arrow keys step the month as its buttons do
	view.onKey = (e) => {
		if (e.key !== "ArrowLeft" && e.key !== "ArrowRight") return false;
		step(e.key === "ArrowLeft" ? -1 : 1);
		return true;
	};

	const slots = h("div", {class: "qp-streak-slots", "aria-hidden": "true"});
	const held = h("p", {class: "qp-type-body-strong"});
	const nextToken = h("p", {class: "qp-type-meta"});
	const tokens = h("qp-card", {class: "qp-streak-tokens"},
		h("p", {class: "qp-type-caps", text: t("pump.streak.tokens")}),
		h("div", {class: "qp-streak-tokens-row"}, slots, h("div", {class: "qp-streak-tokens-words"}, held, nextToken)),
		h("p", {class: "qp-type-meta qp-streak-earn"}));

	const goal = h("p", {class: "qp-type-body-strong"});
	const fill = h("i");
	const bar = h("div", {class: "qp-streak-bar", "aria-hidden": "true"}, fill);
	const togo = h("p", {class: "qp-type-meta"});
	const milestone = h("qp-card", {class: "qp-streak-next"},
		h("p", {class: "qp-type-caps", text: t("pump.streak.next")}), goal, bar, togo);

	const best = h("p", {class: "qp-type-body-strong"});
	const bestCard = h("qp-card", {class: "qp-streak-best"},
		h("p", {class: "qp-type-caps", text: t("pump.streak.best")}), best);

	const share = h("qp-primary-button", {label: t("pump.streak.share"), glyph: "mark_share",
		"on:click": () => openShare(streakCard(get("streak")?.days ?? 0))});
	view.body.append(hero, month, tokens, milestone, bestCard, share);

	watch(view, "streak", (s) => {
		number.textContent = count(s.days);
		line.textContent = s.resting && !s.today
			? t(s.days > 0 ? "pump.home.streak.resting" : "pump.home.streak.resting.none")
			: t("pump.home.streak.line");
		// a new month shows it; another draw keeps the month the player stepped to
		const latest = s.months.at(-1);
		if (latest?.year !== months.at(-1)?.year || latest?.month !== months.at(-1)?.month) back = 0;
		months = s.months;
		back = Math.min(back, Math.max(0, months.length - 1));
		drawMonth();

		slots.replaceChildren(...Array.from({length: s.token_max}, (_, i) => h("span",
			{class: i < s.tokens ? "qp-streak-slot qp-streak-held" : "qp-streak-slot"},
			i < s.tokens ? h("qp-art", {name: "mark_rest"}) : "")));
		held.textContent = t("pump.streak.tokens.held", {count: s.tokens, max: s.token_max});
		nextToken.textContent = s.next_token === null
			? (s.tokens >= s.token_max ? t("pump.streak.tokens.full", {count: s.token_max}) : "")
			: t("pump.streak.tokens.next", {count: s.next_token});
		nextToken.hidden = !nextToken.textContent;
		tokens.querySelector(".qp-streak-earn").textContent = t("pump.streak.tokens.earn",
			{days: s.token_days, max: s.token_max});

		const m = s.milestone;
		goal.textContent = m ? milestoneWords(m.next) : t("pump.streak.next.none");
		bar.hidden = !m;
		togo.hidden = !m;
		if (m) {
			styled(fill, {"--qp-share": Math.max(0, Math.min(1, (s.days - m.from) / (m.next - m.from)))});
			togo.textContent = t("pump.streak.next.togo", {count: m.next - s.days});
		}

		best.textContent = t("pump.streak.best.days", {count: s.best});
		share.hidden = s.days === 0;
	});
	watch(view, "book", (b) => {
		figure.colour = b?.tint ?? null;
	});
	view.primary = () => (share.hidden ? null : share);
	return view;
});
