// The best-day moment on the computer (the Saga, 2026-10-15; board A-Best; docs/ui.md, "The Saga"; the phones'
// log.best), from the `log` topic's `best` (questpump-desk/src/log.rs: today first became a best day, the moment has not shown on
// this computer this game day and no felt-unwell lock holds): as the window comes forward (its open, once the first
// five minutes' cover is laid, its focus, and its showing where that comes after the focus) or a pump on this computer
// ends (its cover goes, after its Result; the log built at its end too; one the tray or the card started as well since
// 2026-10-25, whose end handed the focus back and closed the window until then), with the window in front and shown,
// never over a pump's or the welcome's cover nor over another window (it waits for that one's `qp-closed`), and never
// into a page the player reads
// (a best day a later log brings waits for the window's next coming forward), a modal page on the iron ground with slow
// gold rays, sized to the window so its Close shows at the default size: Ding!, the banner bar, New best day, the
// banner bar, the day's moves with the knight (the jumping jacks' key pose, arms up) behind the number's foot, its
// helmet and arms clear of the digits, Moves in one day, Your old best: N · date, the gold Share it (the Share window
// on today's Saga card in Story, for that opening alone) and the quiet Close (Escape too); it opens at its top, and
// where the window is too short for it (a large zoom) it scrolls with a fade at its foot that says more is below. Rust
// hears it showed as it shows (`log.best {day}`,
// as the phones mark it), so a quit or a closed window never shows it twice that game day; a pump starting, the
// felt-unwell lock or a new game day closes it, and its close says `qp-closed` as a window's does (the reminder offer
// waits for it). While it shows or is due, the Saga's reveal waits (`momentDue`) and plays once it closes. The bars
// slide in, the number counts up and pops, the rays turn (sagaReveal, digitPop, fanfareBars, bestDayRays; Reduce
// Motion: the still page, the sound kept), with the milestone sound once. It shows the number and awards nothing: no
// XP, title, badge, countdown or dare.

import {get, intent, subscribe} from "../../lib/bridge.js";
import {roll} from "../../lib/carry.js";
import {t} from "../../lib/copy.js";
import {topWindow} from "../../lib/focus.js";
import {guard} from "../../lib/keys.js";
import {covered, whenStarted} from "../../lib/router.js";
import {number, shortDate} from "../../lib/saga.js";
import {feedback} from "../../lib/sound.js";
import {openShare} from "../../lib/share.js";
import {h} from "../../kit/element.js";
import {context} from "./chapter.js";

/** The knight behind the number: the jumping jacks' key pose, arms up. */
const KNIGHT = "jumping_jacks";
const motion = window.matchMedia("(prefers-reduced-motion: reduce)");

/** The moment for today's best (`best`: {day, moves, oldBest}), returned as its dialog; `done` when it closes. */
function moment(best, share, done) {
	const count = h("span", {class: "qp-best-number", text: number(best.moves)});
	const close = () => dialog.close();
	const dialog = h("dialog", {class: "qp-best", "aria-label": `${t("best.title")}: ${number(best.moves)}`},
		h("div", {class: "qp-best-rays", "aria-hidden": "true"}),
		h("div", {class: "qp-best-page"},
			h("p", {class: "qp-best-ding", text: t("best.ding")}),
			h("qp-art", {name: "banner_bar", class: "qp-best-bar"}),
			h("h1", {class: "qp-best-title", text: t("best.title")}),
			h("qp-art", {name: "banner_bar", class: "qp-best-bar"}),
			h("div", {class: "qp-best-stage", "aria-hidden": "true"},
				h("qp-figure-stage", {move: KNIGHT, still: true}), count),
			h("p", {class: "qp-best-label", text: t("best.label")}),
			h("p", {class: "qp-type-meta qp-best-old", text: t("best.old",
				{count: best.oldBest.moves, date: shortDate(best.oldBest.day)})}),
			h("div", {class: "qp-best-foot"},
				h("qp-primary-button", {label: t("best.share"), glyph: "mark_share", "on:click": () => {
					close();
					share();
				}}),
				h("qp-quiet-button", {label: t("common.close"), "on:click": close})),
			h("div", {class: "qp-best-more", "aria-hidden": "true"}, h("i", {class: "qp-chevron"}))),
		h("div", {class: "qp-unseen", "aria-live": "polite", "data-live": ""}));
	// the fade at the foot while more of the page is below (a window too short for it)
	const below = () => dialog.classList.toggle("qp-best-below",
		dialog.scrollTop + dialog.clientHeight < dialog.scrollHeight - 1);
	dialog.addEventListener("scroll", below, {passive: true});
	const sized = new ResizeObserver(below);
	dialog.addEventListener("close", () => {
		sized.disconnect();
		// as a window's close says it, for what waits on the windows (the reminder offer, the Saga's reveal)
		dialog.dispatchEvent(new CustomEvent("qp-closed", {bubbles: true}));
		dialog.remove();
		done();
	});
	document.body.append(dialog);
	dialog.showModal();
	// a key held from the screen before (the Result's Continue) presses nothing here
	guard();
	// it opens at its top, Ding! and the headline first, whatever the focus would scroll to
	dialog.scrollTop = 0;
	dialog.querySelector("qp-primary-button")?.focus({preventScroll: true});
	sized.observe(dialog);
	sized.observe(dialog.querySelector(".qp-best-page"));
	feedback("milestone");
	if (!motion.matches) {
		dialog.classList.add("qp-best-in");
		roll(count, 0, best.moves, "saga-reveal");
	}
	return dialog;
}

/** The moment as it stands: its dialog and its day while it shows, the day it last showed on this page. */
let showing = null;
let shown = null;

/** Whether the moment shows, or the log names one this page has not shown yet: the Saga's reveal waits for it. */
export function momentDue() {
	const best = get("log")?.best;
	return Boolean(showing) || Boolean(best && best.day !== shown);
}

/** Follows the `log` topic for the moment and shows it once a game day: as the window comes forward or a pump here
 * ends, with the window in front and no cover or window over it. */
function startBest() {
	// the next log answers a coming forward that found none yet: the window's first, or the one a pump's end builds
	let waiting = false;
	// another window stood over the page: its close asks again
	let blocked = false;
	const show = () => {
		const best = get("log")?.best;
		if (!best || showing || best.day === shown || covered() || !document.hasFocus()
			|| document.visibilityState !== "visible") {
			return false;
		}
		if (topWindow()) {
			blocked = true;
			return false;
		}
		shown = best.day;
		const share = () => {
			const log = get("log");
			const words = get("book.words");
			if (log && words) {
				openShare({kind: "day", day: log.today, ctx: context(words, get("book"), log.plan), card: "saga",
					format: "story"});
			}
		};
		showing = {day: best.day, dialog: moment(best, share, () => {
			showing = null;
		})};
		intent("log.best", {day: best.day}).catch(console.error);
		return true;
	};
	const ask = (next) => {
		waiting = !show() && next;
	};
	const close = () => showing?.dialog.close();
	subscribe("log", (log) => {
		// a new game day takes it away
		if (showing && log.today?.day !== showing.day) close();
		if (waiting) ask(false);
	});
	// the felt-unwell lock and a pump starting close it (it counted as shown as it showed)
	subscribe("home", (home) => {
		if (home.locked_until) close();
	});
	subscribe("pump", (pump) => {
		if (pump.pump) close();
	});
	window.addEventListener("focus", () => ask(false));
	// the window's focus can come before it shows (macOS hears of a Space or the game's full screen going after the
	// window becomes key; Windows of a restore after its focus): its showing asks again
	document.addEventListener("visibilitychange", () => {
		if (document.visibilityState === "visible") ask(false);
	});
	// the player is elsewhere, or reads: a later log waits for the window's next coming forward
	const away = () => {
		waiting = false;
	};
	window.addEventListener("blur", away);
	document.addEventListener("keydown", away, true);
	document.addEventListener("pointerdown", away, true);
	document.addEventListener("qp-uncovered", () => ask(true));
	document.addEventListener("qp-closed", () => {
		if (!blocked) return;
		blocked = false;
		ask(false);
	});
	// the window's open, once the frame and the first five minutes' cover (app.js, after the router) are laid
	queueMicrotask(() => ask(!get("log")));
}

whenStarted(startBest);
