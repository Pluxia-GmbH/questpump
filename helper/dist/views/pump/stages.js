// The pump's stages on the computer (docs/ui.md, "The pump on the computer"; the phones' PumpSession drawn once more):
// Ready with the move up next in full, the countdown, the running move, "Did you do the whole move?", the count set on
// the run view's dial (slice 66; a counted set's form step since slice 42) and a hold's "Could you have held longer?"
// after its Save (slice 52) and a pose's "How close to the goal pose?" over its greyed figure in Big view, every move's
// pour with its rewards (slice 65's hold's, every move's since slice 66), a steady move's "Did you keep that pace to the
// end?" (slice 42),
// and the end, its Result with its one reveal or a calm card. Each stage is {el (its head, body and foot), primary,
// space, onKey, frame, update}: the control its keys go to first (Space or Enter presses it; none where no key may
// answer), whether Space presses it with the focus on no control too (session.js `pressPrimary`: Start, Switch sides),
// its own keys (keys.js: Escape is Done on a running move and back to Ready from the countdown; the digits, Backspace,
// the arrows and the wheel edit the count), what it draws each frame from Rust's absolute instants (the clock, the
// digit's ring, an all-out window's part, the reveal), which decides nothing, and what it takes from the topic as it
// stands (the busy state, the read-aloud button's, the reveal's).

import {get, intent} from "../../lib/bridge.js";
import {t} from "../../lib/copy.js";
import {figureName} from "../../lib/figure.js";
import {announce} from "../../lib/focus.js";
import {time} from "../../lib/format.js";
import {guardMs} from "../../lib/keys.js";
import {openMenu, over} from "../../lib/router.js";
import {openShare, streakCard} from "../../lib/share.js";
import {h, qualityName} from "../../kit/element.js";
import {shifted, turned} from "../../kit/qp-count-bar.js";
import {countUnit} from "../../kit/qp-stepper.js";
import {rest, tripResult} from "./expedition.js";
import {howAndTips, stopLine} from "../../kit/move-how.js";
import {
	ANSWERS, amount, checkWords, clock, continueWords, goalOf, line, paceLine, phase, preview, question,
	recordWords, reward, sideName,
} from "./words.js";

/** How long "Go" shows over the move once it runs, as on the phones and the watches (seconds). */
const GO_SHOWN = 1;

/** An intent's failure goes to the console (Rust's handler logged why). */
function heard(promise) {
	promise.catch(console.error);
}

/** A stage's frame: its head, its body in one centred column (or, where the stage names its `layout`, a class pump.css
 * lays the stage out by, its parts as they are), its foot. */
function frame({head = [], body = [], foot = [], layout = null}) {
	const column = layout ? null : h("div", {class: "qp-column qp-pump-column"}, ...body);
	return {
		el: h("div", {class: ["qp-pump-stage", layout].filter(Boolean).join(" ")},
			h("header", {class: "qp-pump-head"}, ...head),
			h("div", {class: "qp-pump-body"}, ...(column ? [column] : body)),
			h("footer", {class: "qp-pump-foot"}, ...foot)),
		column,
	};
}

function button(tag, label, onClick, props = {}) {
	return h(tag, {label, ...props, "on:click": onClick});
}

/** What a stage drew takes no click or key for the keys' guard time from now (`guardMs`, the settle wait every app
 * takes): `settled(act)` is `act` once that time passed, nothing before, so the click or the key that ended what stood
 * there (a Done, a double click's second half, the swap row coming again as "Back to" under the pointer) never answers
 * what came in its place (David, 2026-10-31: "I had stretches that now are marked done in the desktop app even so I
 * never said I did them perfect etc"). */
function settles() {
	const drawn = performance.now();
	return (act) => (...args) => {
		if (performance.now() - drawn >= guardMs()) act(...args);
	};
}

/** The steps of the pump (a bar a move, a match pump's riders marked: slice 61) and, where the stage has them, the "···"
 * (every stage but Ready, whose bar holds its ways since 2026-11-05) and Big view's full screen. */
function steps(pump, ctx, {more = true, big = false} = {}) {
	const bar = h("qp-steps-bar", {total: pump.moves.length, now: pump.current + 1});
	bar.riders = pump.moves.map((m) => m.rider);
	const parts = [bar];
	if (big) parts.push(ctx.fullscreen());
	parts.push(more ? h("qp-round-button", {icon: "more", small: true, "on:click": () => ctx.open("more")})
		: h("span", {class: "qp-round-room", "aria-hidden": "true"}));
	return parts;
}

/** Back on a stage after Start: the move's Ready view as if Start never came. */
function back(pump) {
	return pump.back ? button("qp-quiet-button", t("common.back"), () => heard(intent("pump.back"))) : null;
}

/** The read-aloud button (`pump.guide`, the game's guide button; questpump-desk/src/pump/guide.rs): How and Tips read aloud, paused
 * and gone on with, its words and its glyph by the topic's `guide` (`show`, again at each `update`), hidden where no
 * guide reads; a quiet button, never the primary. */
function guideButton(pump) {
	const el = h("qp-quiet-button", {class: "qp-pump-guide", "on:click": () => heard(intent("pump.guide"))});
	const show = (state) => {
		el.hidden = state == null;
		el.label = t(state === "reading" ? "pump.guide.pause" : state === "paused" ? "pump.guide.resume"
			: "pump.guide.read");
		el.glyph = state === "reading" ? "mark_pause" : "mark_play";
	};
	show(pump.guide);
	return {el, show};
}

/** Big view's foot under its primary: the read-aloud button beside Back. */
function ways(guide, pump) {
	return h("div", {class: "qp-pump-ways"}, ...[guide.el, back(pump)].filter(Boolean));
}

// Ready -------------------------------------------------------------------------------------------------------------

/** The goal of the move on Ready, large, over its caps word, and at the row's end its record under "Record" (a
 * loaded move's weighted record; a side move's each side; none on a counted set, whose best is its line), as the
 * phones' GoalValue and RecordValue. */
function goalRow(m) {
	const goal = goalOf(m);
	const record = m.counted ? null : recordWords(m);
	return h("div", {class: "qp-ready-goal"},
		h("div", {class: "qp-ready-cell"},
			h("span", {class: goal.number ? "qp-type-num-large" : "qp-type-num-medium", text: goal.text}),
			goal.word ? h("span", {class: "qp-type-caps", text: goal.word}) : null),
		record ? h("div", {class: "qp-ready-cell"}, h("span", {class: "qp-type-caps", text: t("ladders.record.label")}),
			h("span", {class: "qp-type-num-medium", text: record})) : null);
}

/** The swap row under the figure (Panel.lua's swap row under its right column's figure, the one place to swap; on the
 * computer since 2026-10-26, David: "I cannot siwtch to an easier version", under the goal until 2026-11-05): the
 * first of Rust's offers in the game's order, its tile and its words with the move named, one click to swap it in;
 * with more offers the row's trailing segment, "More ways to swap", opens the Swap window with the rest. A click in the
 * keys' guard time after it is drawn does nothing, so a double click never swaps and swaps back (the row comes again as
 * "Back to" under the pointer), and while the engine answers it is disabled (`busy`). None until the options come,
 * without one, or while felt unwell locks pumps (Close in place of Start). */
function swapRow(pump, topic, ctx) {
	const [first, ...rest] = pump.swap?.options ?? [];
	if (!first || topic.locked_until) return {el: null, busy() {}};
	const settled = settles();
	const row = h("qp-row", {heading: first.line, move: first.key, quality: first.quality, trailing: "action",
		segment: rest.length ? t("pump.swap.more") : null, disabled: pump.busy,
		"on:click": settled(() => heard(intent("pump.swap", {take: first.take}))),
		"on:qp-segment": settled(() => ctx.open("swap"))});
	return {el: h("qp-card", {rows: true}, row), busy: (on) => (row.disabled = on)};
}

/** The Ready bar's ways (2026-11-05, the game's bar: Panel.lua's Bank all, Skip, Pause and Settings at its left), each
 * a secondary button: the way out, Bank all on a match's pump while a move of its own is left, Bank the rest on a
 * catch-up and once only a match pump's riders are left (Rust's `bank`, slice 58: one click, no confirm), Close on
 * every other pump, which ends it as End the pump here did (`pump.end`: a pump the player started owes nothing, call 4);
 * the game's Skip on a rider (one click, since 2026-10-09); Pause, opening its menu above it (`pauseMenu`); and Settings,
 * Your setup over the pump (lib/router.js `over`), whose back returns to the pump on its Ready view. */
function readyWays(pump, ctx) {
	const secondary = (label, act, props = {}) => h("qp-secondary-button", {label, ...props, "on:click": act});
	const out = pump.bank
		? secondary(t(pump.bank === "it" ? "pump.ready.bank" : "pump.ready.bank.rest"), () => heard(intent("pump.bank")))
		: secondary(t("common.close"), () => heard(intent("pump.end")));
	const skip = pump.skip ? secondary(t("pump.unfinished.skip"), () => heard(intent("pump.skip")),
		{hint: t("pump.unfinished.skip.hint")}) : null;
	const pause = secondary(t("pump.ready.pause"), () => ctx.open("pause", pause), {menu: true});
	// its focus key brings the focus back to it as Your setup's back returns to a pump drawn again meanwhile
	const settings = secondary(t("pump.ready.settings"), () => over("desk.menu.setup"),
		{"data-focus-key": "pump:settings"});
	return h("div", {class: "qp-ready-ways"}, ...[out, skip, pause, settings].filter(Boolean));
}

/** Ready (2026-10-24, David: "It should be like the addon. Just go right away in and show whats up next."; the
 * game's Ready view; a quest page of every move's line until then), in the game's shape since 2026-11-05 (David: "I
 * would like to improve the desktop app to be more like the wow addon. See how it has a better UI with a right wrong
 * example. An easy way to switch to an alternative exercise. bank pause settings buttons etc."): × (leave) and the
 * steps; the move up next in full, its words (a stretch check's "Stretch check" over its name, its name with NEW beside
 * it on its first appearance and its quality, its goal with its record, the lines under the goal in the phones' order:
 * a pose's goal pose, a first max set's how far to go, Rust's `lines`; the arms rest's line while the pump rests the
 * arms for an aim game, slice 48's double-check: `arms`, the game's name; the move's rewards' pills, its stop line, none
 * on a timed move without one, How with the read-aloud button at its heading's end and Tips) beside the game's right
 * column (pump.css; on a narrow window the figure over the words and the rest under them, the page's order): its
 * figure playing large and pausable (Big view's one figure, `ctx.figure`, so the same film goes on into the countdown),
 * on its first appearance the right-versus-wrong pair under it, and the swap row while Swap offers a move (`swapRow`);
 * then the game's bar, its ways at the left (`readyWays`)
 * and Start at the right, the one red primary; while felt unwell locks pumps (a lock that came while the pump waited),
 * the rest's line and Close alone. */
export function ready(pump, topic, ctx) {
	const m = pump.moves[pump.current];
	const figure = h("qp-figure-stage", {class: "qp-ready-figure", fill: true, pausable: true, move: figureName(m)});
	figure.hold(ctx.figure);
	const guide = guideButton(pump);
	const said = (text) => (text ? h("p", {class: "qp-type-subtitle", text}) : null);
	const name = h("div", {class: "qp-ready-name"}, pump.check ? said(t("pump.source.check")) : null,
		h("div", {class: "qp-ready-title"}, h("h1", {class: "qp-type-title", text: m.name}),
			m.first ? h("qp-slant-tag", {label: t("pump.ready.new")}) : null),
		m.quality ? h("qp-quality-word", {quality: m.quality, word: m.word}) : null);
	const pills = pump.rewards.length ? h("div", {class: "qp-ready-pills"}, ...pump.rewards.map((p) =>
		h("qp-reward-pill", p.kind === "stat"
			? {kind: "stat", amount: p.amount, stat: p.stat, record: amount(p.beat, p.hold), glyph: `stat_${p.key}`}
			: p))) : null;
	const swapping = swapRow(pump, topic, ctx);
	const words = h("div", {class: "qp-ready-words"}, ...[name, goalRow(m), said(m.mark), said(m.note),
		...pump.lines.map((l) => said(line(l))), pump.arms ? said(t("pump.rest.arms", {game: pump.arms})) : null, pills,
		stopLine(m.stop), ...howAndTips({setup: pump.setup, how: pump.how, tips: pump.tips, action: guide.el})]
		.filter(Boolean));
	const pair = m.first ? h("qp-move-figure-pair", {move: figureName(m), name: m.name}) : null;
	const below = pair || swapping.el
		? h("div", {class: "qp-ready-below"}, ...[pair, swapping.el].filter(Boolean)) : null;
	let primary;
	let foot;
	if (topic.locked_until) {
		primary = button("qp-primary-button", t("common.close"), () => heard(intent("pump.end")));
		foot = [h("p", {class: "qp-type-subtitle qp-pump-centre",
			text: t("pump.ready.locked", {time: time(topic.locked_until)})}), primary];
	} else {
		primary = button("qp-primary-button", t("pump.ready.start"), () => heard(intent("pump.ready.start")),
			{class: "qp-ready-start", state: pump.busy ? "busy" : "normal"});
		foot = [h("div", {class: "qp-ready-bar"}, readyWays(pump, ctx), primary)];
	}
	const {el} = frame({
		head: [h("qp-round-button", {icon: "close", small: true, "on:click": () => ctx.open("leave")}),
			...steps(pump, ctx, {more: false})],
		body: [figure, words, below].filter(Boolean),
		foot,
		layout: "qp-ready",
	});
	return {
		el,
		primary: () => primary,
		space: !topic.locked_until,
		update(next) {
			if (!topic.locked_until) primary.state = next.busy ? "busy" : "normal";
			swapping.busy(next.busy);
			guide.show(next.guide);
		},
	};
}

// An expedition's breather (slice 51) ---------------------------------------------------------------------------------

/** The breather between two rounds of an expedition (expedition.js `rest`) in a stage's frame: Start now is its
 * primary (Space or Enter), Escape answers nothing. */
export function resting(pump) {
	const r = rest(pump);
	const {el} = frame({body: r.body, foot: r.foot});
	return {el, primary: r.primary, frame: r.frame};
}

// Big view: the countdown, the running move, the timed ask ----------------------------------------------------------

/** Big view's middle, the PumpStage (2026-10-23, David: "Please make the excercise way better and visible during the
 * countdown"): the move's figure whole on its stage, never dimmed, mirrored on a side move's other side (Big view's one
 * figure, `ctx.figure`, held from the countdown through the run and gone when no stage holds it), taking the height
 * (David: "why sprawl so much smaller than push up"), with the move's name, the TimerRing (its dial, which holds the
 * digit, "Go", the clock or the count) and the words (the line; `words` takes a pace line before it) placed about it
 * by pump.css. A set of reps' loop is phased to its go from the countdown on (its `beat` and the go, Rust's), so the
 * voice's count, and the count in the dial where the voice counts, come as the figure finishes each rep. The name
 * carries the side a side move is on, and fits its room (`fitName`). */
function bigMiddle(pump, ctx) {
	const m = pump.moves[pump.current];
	const name = h("h1", {class: "qp-big-name", text: sideName(m, pump.side)});
	const figure = h("qp-figure-stage", {class: "qp-big-figure", fill: true, move: figureName(m), side: pump.side,
		beat: m.beat, from: pump.go ?? pump.start});
	figure.hold(ctx.figure);
	const ring = h("qp-timer-ring");
	const sub = h("p", {class: "qp-big-line"});
	const words = h("div", {class: "qp-big-words"}, sub);
	const dial = h("div", {class: "qp-big-dial"}, ring);
	const stage = h("div", {class: "qp-big-stage"}, name, figure, dial, words);
	const keep = [dial];
	fitName(name, dial, keep);
	return {name, ring, dial, sub, words, keep, parts: [stage]};
}

/** Big view's name (a move's, or the count's question that names it) at its size in pump.css, or under it the largest
 * at which no word of it is wider than its line, the whole of each control in `keep` (the dial, and a count's weight
 * box or a question's answers) stays in the body's first view and the dial keeps TimerRing's least room
 * (`surface.TimerRing.least`, under which its count turns compact), at every size of the window (David, 2026-11-06, on
 * a count's question that broke "Copenhage-n" and hid the seconds under the foot: "they should always be visible no
 * matter the window size", and on a weighted count's keys pushed to the column's edges: "ui issues here with
 * overlapping"). Where no size keeps the dial that room, the least, the words' own size (pump.css
 * `--qp-big-line-size`), which gives the dial all the room there is; there a word too long for its line breaks again.
 * It fits when the stage is first drawn and whenever the body's or a kept control's size changes, and stops once the
 * stage has left the page. */
function fitName(name, dial, keep) {
	const roomy = (body) => {
		if (name.scrollWidth > name.clientWidth + 1) return false;
		const top = body.getBoundingClientRect().top - body.scrollTop;
		if (keep.some((el) => el.getBoundingClientRect().bottom - top > body.clientHeight + 1)) return false;
		const css = getComputedStyle(dial);
		const ring = parseFloat(css.getPropertyValue("--qp-surface-timer-ring-least"));
		return dial.clientHeight - parseFloat(css.paddingTop) - parseFloat(css.paddingBottom) >= ring - 0.5;
	};
	/** The largest size from `lo`, where the name is roomy, to `hi`, where it is not, at which it is roomy. */
	const largest = (body, lo, hi) => {
		for (let i = 0; i < 8; i++) {
			const size = (lo + hi) / 2;
			name.style.fontSize = `${size}px`;
			if (roomy(body)) lo = size;
			else hi = size;
		}
		return lo;
	};
	const fit = (body) => {
		name.style.fontSize = "";
		name.style.overflowWrap = "normal";
		if (!roomy(body)) {
			const css = getComputedStyle(name);
			const most = parseFloat(css.fontSize);
			const floor = Math.min(most, parseFloat(css.getPropertyValue("--qp-big-line-size")));
			name.style.fontSize = `${floor}px`;
			if (roomy(body)) name.style.fontSize = `${largest(body, floor, most)}px`;
		}
		name.style.overflowWrap = "";
	};
	const body = () => (name.isConnected ? name.closest(".qp-pump-body") : null);
	const again = () => {
		const room = body();
		if (room) return fit(room);
		sized.disconnect();
		document.fonts.removeEventListener("loadingdone", again);
	};
	const sized = new ResizeObserver(again);
	// once the stage is on the page (session.js places it in the same task): fitted before its first paint, even in a
	// window the game hides, where neither a frame nor a size change comes until it shows; again at each size of the
	// body and when a font arrives (Cinzel, the name's, after a launch), which changes the name and not the body
	queueMicrotask(() => {
		const room = body();
		if (!room) return;
		fit(room);
		sized.observe(room);
		// a control that shows or hides later (the weight box's "Don't know the weight") changes what must stay in view
		for (const el of keep) if (el !== dial) sized.observe(el);
		document.fonts.addEventListener("loadingdone", again);
	});
}

/** The countdown (design v3 §4): the move's name and its side, the move's figure playing where it will run, the digit
 * in its dial (the first until it comes), Back where Start stood with the read-aloud button beside it; Escape is
 * Back. */
export function countdown(pump, topic, ctx) {
	const m = pump.moves[pump.current];
	const mid = bigMiddle(pump, ctx);
	// a timed move with no stop line leaves the line empty, which hides it
	mid.sub.textContent = pump.side === "first" ? t("pump.countdown.first")
		: pump.side === "other" ? t("pump.countdown.switch") : m.stop ?? "";
	const guide = guideButton(pump);
	const {el} = frame({head: steps(pump, ctx, {more: false, big: true}), body: mid.parts, foot: [ways(guide, pump)]});
	let digit = pump.digit;
	return {
		el,
		big: true,
		primary: () => null,
		update: (next) => {
			guide.show(next.guide);
			digit = next.digit;
		},
		onKey(e) {
			if (e.key !== "Escape") return false;
			heard(intent("pump.back"));
			return true;
		},
		frame(now) {
			const left = Math.max(0, (pump.go - now) / 1000);
			mid.ring.mode = {kind: "countdown", total: 5, left, digit: digit ?? 5};
			return true;
		},
	};
}

/** The reps a set's figure has ended `ms` into its side's run, at most one set's most, by its count's rule (`rule`, the
 * move's `rep_count`: `beat` ms a loop, `ends` the ms in the loop each rep ends, the last the loop's, and `most`), a
 * rep ending at `ms` itself counted: the twin of Rust's `Beats::ended` (questpump-desk/src/pump/run.rs; Speech.lua countsBy), by
 * which the voice counts and the count step starts, kept here so no rep changes the `pump` topic. */
function ended(rule, ms) {
	const elapsed = Math.max(0, ms);
	const loops = Math.floor(elapsed / rule.beat);
	const into = elapsed - loops * rule.beat;
	return Math.min(rule.most, loops * rule.ends.length + rule.ends.filter((e) => e <= into).length);
}

/** The running move (mockup pump-run): its name (its side on a side move), its figure on the stage (a set of reps' at
 * the pace the voice counts), the dial (a timed side's time left, after Keep going its seconds run counting up to the
 * full move, a hold's time held against its goal or record; a set of reps the voice counts its reps counted so far,
 * from 0 at each side's go, over its goal or else Max set or One set (2026-10-13); any other set of reps its goal, else
 * Max set or One set) with "Go" alone for its first second, a steady move's pace in words (slice 42's fix round,
 * 2026-10-17: the talk test the voice's cadence sets), its line (an all-out window's part, else the stop line through
 * every set, none on a timed move without one), Done (Switch sides on a side move's first side) with Keep going over it
 * in a steady move's last keep_going_s (slice 52; Space, which once pressed presses no Done), and Back with the
 * read-aloud button beside it; Escape and Enter are Done. */
export function running(pump, topic, ctx) {
	const m = pump.moves[pump.current];
	const mid = bigMiddle(pump, ctx);
	const done = button("qp-primary-button", pump.side === "first" ? t("pump.running.switch") : t("pump.running.done"),
		() => heard(intent("pump.done")));
	if (m.pace != null) mid.words.prepend(h("p", {class: "qp-big-line", text: paceLine(m.pace)}));
	// a steady move's last keep_going_s while its time is under the full move's (slice 52): the clock runs on to it;
	// over Done, so Done keeps its place at the window's foot
	const keep = pump.keep_going ? button("qp-secondary-button", t("pump.running.keepgoing"),
		() => heard(intent("pump.keep"))) : null;
	const guide = guideButton(pump);
	const {el} = frame({head: steps(pump, ctx, {big: true}), body: mid.parts, foot: [keep, done, ways(guide, pump)]});
	const goal = m.target.kind === "goal" ? m.target.value : null;
	const word = () => (m.target.kind === "set" ? t("kit.set") : t("kit.maxset"));
	const mode = (ms) => {
		const elapsed = ms / 1000;
		if (pump.limit != null) {
			// after Keep going the seconds run, counting up to the full move (slice 52)
			return pump.kept ? {kind: "up", total: pump.limit, elapsed}
				: {kind: "timed", total: pump.limit, left: Math.max(0, pump.limit - elapsed)};
		}
		if (m.hold) {
			return {kind: "hold", elapsed, goal: m.target.kind === "time" ? m.target.value : null, record: m.record};
		}
		if (m.rep_count) return {kind: "count", reps: ended(m.rep_count, ms), goal, word: goal == null ? word() : null};
		if (goal != null) return {kind: "reps", goal};
		return {kind: "word", word: word()};
	};
	return {
		el,
		big: true,
		primary: () => done,
		space: pump.side === "first",
		update: (next) => guide.show(next.guide),
		onKey(e) {
			// Keep going is Space, the game's bar's secondary key, while Enter stays Done; once pressed Space presses no
			// Done for the rest of the move, as the game's Space never ends one (Panel.HandleKey)
			if (keep && e.key === " ") {
				heard(intent("pump.keep"));
				return true;
			}
			if (pump.kept && e.key === " " && done.contains(e.target)) return true;
			if (e.key !== "Escape") return false;
			heard(intent("pump.done"));
			return true;
		},
		frame(now) {
			const ms = Math.max(0, now - pump.start);
			const elapsed = ms / 1000;
			mid.ring.go = elapsed < GO_SHOWN;
			mid.ring.mode = mode(ms);
			const words = phase(m, elapsed);
			if (mid.sub.textContent !== words) mid.sub.textContent = words;
			return true;
		},
	};
}

/** A timed move Done before the ask's share of its time (2026-10-05): "Did you do the whole move?" and the time it ran
 * said plainly; no answer has a key (Enter and Escape answer nothing), and a click answers once the ask has settled
 * (`settles`), so the Done that ended the move, pressed twice, never answers it. */
export function asking(pump, topic, ctx) {
	const settled = settles();
	const heading = h("h1", {class: "qp-big-name", tabindex: "-1", text: t("pump.timed.ask.title")});
	const said = h("qp-card", {class: "qp-big-card"}, h("p", {class: "qp-type-body",
		text: t("pump.timed.ask.clock", {clock: clock(pump.asked.seconds), full: clock(pump.asked.full)})}));
	const yes = button("qp-primary-button", t("pump.timed.ask.yes"),
		settled(() => heard(intent("pump.timed", {whole: true}))));
	const no = button("qp-secondary-button", t("pump.timed.ask.no"),
		settled(() => heard(intent("pump.timed", {whole: false}))));
	const {el} = frame({head: steps(pump, ctx, {big: true}), body: [heading, said], foot: [yes, no, back(pump)]});
	return {el, big: true, primary: () => heading};
}

// How close to the goal pose? ---------------------------------------------------------------------------------------

/** The goal pose's sentence, which the question's heading names as its description. */
const GOAL_ID = "qp-pump-goal";

/** A pose whose move asks ran its whole time (2026-10-09, David: "Reached the mark was very unclear. Think through a
 * better ask. Maybe 4 options or so"; 2026-10-14: "whats to achieve, the "mark" (please rename for clarity), is not
 * very clear"): on the run's own Big view, so its pour plays where the ask stood (Rapha, 2026-11-09: "over the animation
 * of the pose (like pose grayed out behind) and not full screen so you can see collect animation after clicking"), the
 * figure dimmed behind a card over it and the dial at the pose's time as the pour opens: in the card "Goal pose" over its
 * sentence, the question, the four answers from Perfect down as rows (each its key and its word, the key its code: 4
 * Perfect to 1 Not yet, so a stray 1 is the lowest answer; Enter and Escape answer nothing), which answer once the ask has
 * settled (`settles`), and Back; nothing is written before an answer. */
export function marking(pump, topic, ctx) {
	const m = pump.moves[pump.current];
	const ask = pump.marking;
	const settled = settles();
	const mid = bigMiddle(pump, ctx);
	const [stage] = mid.parts;
	stage.classList.add("qp-marking");
	const seconds = m.seconds ?? 0;
	mid.ring.mode = {kind: "pour", clock: seconds, seconds: true, share: 1,
		caption: t("kit.sweep.goal", {time: clock(seconds)})};
	const heading = h("h2", {class: "qp-type-heading qp-pump-centre", tabindex: "-1", text: t("pump.mark.ask.title"),
		"aria-describedby": ask.mark ? GOAL_ID : null});
	const answer = (code) => settled(() => heard(intent("pump.mark", {answer: code})));
	const rows = ANSWERS.map(([code, word]) => h("qp-row", {step: code, heading: t(word), trailing: "action",
		"on:click": answer(code)}));
	const goal = ask.mark ? h("div", {class: "qp-mark qp-pump-centre", role: "group"},
		h("span", {class: "qp-type-caps", text: t("ladders.mark")}),
		h("p", {class: "qp-type-subtitle", id: GOAL_ID, text: ask.mark})) : null;
	stage.append(h("div", {class: "qp-mark-ask"}, ...[goal, heading, h("qp-card", {rows: true}, ...rows)].filter(Boolean)));
	const {el} = frame({head: steps(pump, ctx, {big: true}), body: mid.parts, foot: [back(pump)]});
	return {
		el,
		big: true,
		primary: () => heading,
		onKey(e) {
			const code = ANSWERS.find(([c]) => String(c) === e.key)?.[0];
			if (code == null) return false;
			answer(code)();
			return true;
		},
	};
}

// Did you keep that pace to the end? ---------------------------------------------------------------------------------

/** The pace ask's answers in their order, each its code (Core.lua ns.Enum.pace) and its words: keys 1-3 answer them
 * (slice 52: Yes, easily, Yes, just, I eased off). */
const PACES = [[3, "pump.pace.easily"], [1, "pump.pace.held"], [2, "pump.pace.eased"]];

/** A steady move whose slot asks its pace ran its whole time (slice 42; since slice 52 where the steady ledger asks,
 * once a pump, and after every Long Haul; the cadence lines set the pace by the talk test): its key pose held still,
 * the question, its three answers as rows (keys 1-3; Enter and Escape answer nothing), which answer once the ask has
 * settled (`settles`), and Back; nothing is written before an answer, which goes in the move's row. */
export function pacing(pump, topic, ctx) {
	const m = pump.moves[pump.current];
	const settled = settles();
	const heading = h("h1", {class: "qp-type-heading qp-pump-centre", tabindex: "-1", text: t("pump.pace.title")});
	const answer = (code) => settled(() => heard(intent("pump.pace", {answer: code})));
	const rows = PACES.map(([code, words], i) => h("qp-row", {step: i + 1, heading: t(words), trailing: "action",
		"on:click": answer(code)}));
	const body = [h("qp-figure-stage", {move: figureName(m), still: true, class: "qp-pump-figure"}), heading,
		h("qp-card", {rows: true}, ...rows)];
	const {el} = frame({head: steps(pump, ctx), body, foot: [back(pump)]});
	return {
		el,
		primary: () => heading,
		onKey(e) {
			const n = /^[1-3]$/.test(e.key) ? Number(e.key) : 0;
			if (!n) return false;
			answer(PACES[n - 1][0])();
			return true;
		},
	};
}

// Could you have held longer? ----------------------------------------------------------------------------------------

/** The hold ask's answers in their order, each its code (Core.lua ns.Enum.longer) and its words: keys 1-3. */
const LONGER = [[1, "pump.hold.ask.no"], [2, "pump.hold.ask.little"], [3, "pump.hold.ask.lot"]];

/** A fresh hold of a Stamina Ledger ladder whose Save the ledger asks about (slice 52, Progress.lua ns.HoldAsk; the
 * phones' pump.count longer), on the run view it was counted on (slice 66): the dial still at the seconds saved, the
 * question in the name's place, its three answers as rows under the dial (keys 1-3; Enter and Escape answer nothing),
 * which answer once the ask has settled (`settles`), and Back; the hold is saved with its answer, nothing before it. */
export function longer(pump, topic, ctx) {
	const m = pump.moves[pump.current];
	const c = pump.count;
	const settled = settles();
	const mid = bigMiddle(pump, ctx);
	mid.name.textContent = t("pump.hold.ask.title");
	mid.name.tabIndex = -1;
	const whole = against(m, c);
	const share = whole ? c.from / whole : 1;
	mid.ring.mode = {kind: "entry", text: String(c.from), unit: countUnit(c.from, true), share,
		chip: c.chip != null && c.from > c.chip};
	const answer = (code) => settled(() => heard(intent("pump.longer", {answer: code})));
	const rows = LONGER.map(([code, words], i) => h("qp-row", {step: i + 1, heading: t(words), trailing: "action",
		"on:click": answer(code)}));
	const answers = h("qp-card", {rows: true, class: "qp-big-card"}, ...rows);
	mid.keep.push(answers);
	mid.words.replaceChildren(answers);
	const guide = guideButton(pump);
	const {el} = frame({head: steps(pump, ctx, {big: true}), body: mid.parts, foot: [ways(guide, pump)]});
	return {
		el,
		big: true,
		primary: () => mid.name,
		update: (next) => guide.show(next.guide),
		onKey(e) {
			const n = /^[1-3]$/.test(e.key) ? Number(e.key) : 0;
			if (!n) return false;
			answer(LONGER[n - 1][0])();
			return true;
		},
	};
}

// A move saved: its pour ---------------------------------------------------------------------------------------------

/** How far down a grain's path sags under the straight line between the dial and the bar (px), as the mockup flies
 * them. */
const GRAIN_SAG = 30;

/** A grain from the dial's tip (`from`) into its unit's place on the bar (`to`), flying `fly` ms from `delay` ms
 * from now (a negative delay starts it part way, as a late frame finds it). */
function grain(layer, from, to, fly, delay) {
	const el = h("i", {class: "qp-pour-grain", "aria-hidden": "true"});
	layer.append(el);
	const mid = {x: (from.x + to.x) / 2, y: Math.max(from.y, to.y) + GRAIN_SAG};
	el.animate([{transform: shifted(from.x, from.y, turned(from.angle + 90)), opacity: 1},
		{transform: shifted(mid.x, mid.y, turned((from.angle + 90) / 2)), offset: 0.45},
		{transform: shifted(to.x, to.y, turned(0)), opacity: 0.3}],
	{duration: Math.max(1, fly), delay, easing: "cubic-bezier(0.45, 0, 0.7, 1)", fill: "both"})
		.onfinish = () => el.remove();
}

/** The dial's tip at arc share `share` in the window's px, with its angle (deg, from 12 o'clock as the arc runs). */
function tipOf(ring, share) {
	const r = ring.getBoundingClientRect();
	const angle = -90 + 360 * share;
	const radius = r.width * 0.47;
	const a = angle * Math.PI / 180;
	return {x: r.left + r.width / 2 + radius * Math.cos(a), y: r.top + r.height / 2 + radius * Math.sin(a), angle};
}

/** A move saved (slice 65's hold, every move since slice 66, `desk.pump.saved`; the phones' pump.saved): the figure
 * dimmed, what it saved (a set of reps' reps, a hold's seconds, a counted set's clean reps, a timed move's seconds)
 * pouring one grain a unit from the dial's tip into their places on the CountBar beside it, the dial emptying as they
 * leave, its number counting each landing unit (m:ss for seconds), the record chip once past the record and the top's
 * flash and sparks; then, where the save opened a step, the StepReveal over the figure's room folding into its unlock
 * row while the bar rolls over to the next step's; then the move's own rows, in place of the dial on a wide window and
 * of the figure on a tall one, the next move coming by itself the game's record_hold_s after them, or after the last
 * window over the stage closed, never under one (Rust's `Pour::goes_on_at`; Rapha, 2026-11-04). `Next: {move}` (Finish
 * on the pump's last move) is the primary from the first frame (Enter); a click on the stage lands one stage (the pour, then the
 * reveal); Back and Escape take the move back to its Ready view, nothing saved, while the pour runs. Rust plans every
 * instant and plays the sounds; under Reduce Motion the end state at once. */
export function saved(pump, topic, ctx) {
	const m = pump.moves[pump.current];
	let s = pump.saved;
	const settled = settles();
	const mid = bigMiddle(pump, ctx);
	const [stage] = mid.parts;
	stage.classList.add("qp-saved");
	const bar = h("qp-count-bar", {class: "qp-saved-bar"});
	bar.bar = s.bar;
	const rows = h("div", {class: "qp-saved-rows", hidden: true});
	const room = h("div", {class: "qp-saved-room"});
	const layer = h("div", {class: "qp-pour-layer", "aria-hidden": "true"});
	stage.append(bar, rows, room, layer);
	const next = button("qp-primary-button", s.next ? t("pump.saved.next", {move: s.next}) : t("pump.saved.finish"),
		settled(() => heard(intent("pump.saved.next"))));
	const backButton = button("qp-quiet-button", t("common.back"), settled(() => heard(intent("pump.back"))));
	backButton.hidden = !pump.back;
	const guide = guideButton(pump);
	const {el} = frame({head: steps(pump, ctx, {big: true}), body: mid.parts,
		foot: [next, h("div", {class: "qp-pump-ways"}, guide.el, backButton)]});
	stage.addEventListener("click", settled(() => heard(intent("pump.saved.land"))));
	const {goal, top, seconds} = s.bar;
	const caption = goal == null ? "" : seconds ? t("kit.sweep.goal", {time: clock(goal)})
		: t("kit.sweep.goal.reps", {count: goal});
	const record = s.record ?? null;
	const f0 = Math.min(1, s.value / Math.max(1, s.against));
	let poured = 0;
	let flashed = false;
	let reveal = null;
	let folded = false;
	let shownRows = 0;
	let rowEls = [];
	let earnedFor = null;
	if (matchMedia("(prefers-reduced-motion: reduce)").matches && !s.landed) {
		heard(intent("pump.saved.land", {still: true}));
	}
	const landedCount = (now) => (s.landed ? s.value : s.starts.filter((at) => at + s.fly <= now).length);
	const showRows = (now, e) => {
		if (earnedFor !== e) {
			earnedFor = e;
			rowEls = e.rows.map((row) => {
				const r = h("qp-reward-row");
				r.reward = reward(row.reward);
				r.move = row.move;
				return r;
			});
		}
		while (shownRows < rowEls.length && now >= e.rows_at[shownRows]) {
			rowEls[shownRows].shown = true;
			rows.append(rowEls[shownRows]);
			shownRows += 1;
		}
		rows.hidden = shownRows === 0;
		stage.classList.toggle("qp-saved-rows-in", shownRows > 0);
		if (shownRows === 1 && !rows.dataset.said) {
			rows.dataset.said = "";
			announce(rowEls.flatMap((row) => row.spoken()).filter(Boolean).join(". "));
		}
	};
	/** The reveal's own unlock row, whose tile its hero folds into (the first unlock where none names its move). */
	const unlockRow = () => {
		const unlocks = rowEls.filter((row) => row.reward?.kind === "unlock");
		const hero = s.earned?.reveal?.hero?.key;
		return unlocks.find((row) => hero != null && row.move?.key === hero) ?? unlocks[0] ?? null;
	};
	return {
		el,
		big: true,
		primary: () => next,
		update(nextPump) {
			s = nextPump.saved ?? s;
			backButton.hidden = !nextPump.back;
			guide.show(nextPump.guide);
		},
		onKey(e) {
			if (e.key !== "Escape" || s.landed) return false;
			// as Back's click: an Escape right behind the Save never drops the pour
			settled(() => heard(intent("pump.back")))();
			return true;
		},
		frame(now) {
			const n = s.value;
			const started = s.landed ? n : s.starts.filter((at) => at <= now).length;
			// the grains that have left and not yet flown, each once
			if (s.landed) layer.replaceChildren();
			else {
				for (; poured < started; poured += 1) {
					const from = tipOf(mid.ring, f0 * (n - poured) / n);
					grain(layer, from, bar.slot(poured + 1), s.fly, s.starts[poured] - now);
				}
			}
			const held = landedCount(now);
			mid.ring.mode = {kind: "pour", clock: held, seconds, share: f0 * (n - started) / Math.max(1, n), caption,
				chip: record != null && held > record};
			if (!folded) bar.held = held;
			if (!flashed && top != null && held >= top) {
				flashed = true;
				if (!s.landed) bar.flash();
			}
			const e = s.earned;
			if (!e) return !s.landed;
			const r = e.reveal;
			if (r && !reveal && !folded && !r.landed) {
				reveal = h("qp-step-reveal");
				room.append(reveal);
				reveal.reveal = r;
				reveal.play();
			}
			const folds = r && (r.landed || now >= r.folds);
			if (!r || folds) showRows(now, e);
			// the hero folds into its row's tile once that row is in (at once where the reveal landed)
			const row = unlockRow();
			if (r && folds && !folded && (r.landed || !row || row.isConnected)) {
				folded = true;
				const tile = row?.isConnected ? row.querySelector("qp-move-tile") : null;
				reveal?.fold(tile?.getBoundingClientRect() ?? null);
				if (e.next_bar) {
					if (r.landed) {
						bar.held = 0;
						bar.bar = e.next_bar;
					} else bar.roll(e.next_bar);
				}
			}
			const last = e.rows_at[e.rows_at.length - 1] ?? 0;
			return shownRows < rowEls.length || now < last || !s.landed;
		},
	};
}

// The count, set on the dial (slice 66) -------------------------------------------------------------------------------

/** What the dial's arc stands the count against: the move's goal (its count or its hold's seconds), else its record (a
 * counted set's best clean count); none for a move with neither, whose arc is full for any count. */
function against(m, c) {
	const goal = m.target.kind === "goal" || m.target.kind === "time" ? m.target.value : null;
	return goal ?? c.record ?? null;
}

/** A loaded move's weight box under the dial (the kg mode's stepper: `load_box_step` steps or a kettlebell's sizes,
 * typing "." or ","), at the weight typed before a check, else the plan or the start range's low end (`first`), empty
 * once the player clears it for no weight; none on any other move. */
function weightBox(c) {
	if (!c.kg) return null;
	const words = t(c.kg.each ? "pump.count.kg.each" : "pump.count.kg");
	const box = h("qp-stepper", {decimal: true, value: c.kg.from, first: c.kg.first, most: c.kg.most,
		stride: c.kg.step, "unit-one": words, "unit-other": words, label: words});
	box.sizes = c.kg.sizes;
	return box;
}

/** The answers of a counted set's form step in their order, each its code (Core.lua ns.Enum.answer: 0 Yes, to the end)
 * and its words: keys 1-3 (Not all out since slice 53). */
const FORMS = [[0, "pump.count.clean.end"], [1, "pump.count.slipped"], [2, "pump.count.notallout"]];

/** The line under the dial while the count stands at its most: half again what the app counted (`pump.count.cap`,
 * slice 66; Rapha, 2026-11-04: "with explanation") or the number a set of reps has to beat (`pump.count.cap.goal`),
 * whichever set it, else one set's most in the game's words; none where the floor of what counts sets the most (Rust's
 * `count_view`), hidden otherwise. */
function mostLine(c) {
	const max = c.cap && amount(c.cap.max, c.hold);
	const capped = c.cap?.goal != null ? t("pump.count.cap.goal", {max, goal: amount(c.cap.goal, c.hold)})
		: c.cap && t("pump.count.cap", {max, counted: amount(c.cap.counted, c.hold)});
	const text = capped || c.max_line;
	const el = h("p", {class: "qp-big-line", hidden: true, text});
	return {el, show: (value) => (el.hidden = !text || value < c.most)};
}

/** The dial of the count: the Stepper whose value the run view's TimerRing shows (`ring`), − and + at its sides, from
 * where the count step starts (the reps the figure ended by Done, a hold's stopwatch, the weaker side's; none on a
 * counted set), at most `most` (half again what the app counted or a set of reps' number to beat where that binds,
 * Rust's `count_cap`), the arc against the goal or the record, the record chip past the record. */
function dialOf(m, c, mid, props) {
	const stepper = h("qp-stepper", {class: "qp-dial-count", value: c.from, least: 0, most: c.most,
		unit: c.hold ? "seconds" : "reps", record: c.chip, against: against(m, c), ...props});
	stepper.ring = mid.ring;
	mid.dial.replaceChildren(stepper);
	return stepper;
}

/** A counted set's form step on its run view (slice 42, 2026-10-16; David: "The count in a time is not accurate enough
 * and individual. It should not actually be used to advance"; the phones' pump.count counted, the game's counted
 * Result; on the dial since slice 66): "{move}: all out and clean?" (slice 53; "did your form hold?" until then) in the
 * name's place, its optional clean count on the dial from "–" (0: no count, read "Not counted"), and under it the three
 * answers as rows of equal weight, Yes, to the end, Form slipped and Not all out (each saves the set with the count the
 * dial holds, on a click once the step has settled, `settles`; the digits and Backspace type the count wherever the
 * focus stands, as the game's do: keys 1-3 took the answers until 2026-11-04, so a typed 24 saved Form slipped), and "Is that right?" for a count far from the best, its Yes sending the count
 * with the answer it was given and Change it, Enter pressing that Yes; Back. No primary while it does not ask: Enter
 * and Escape answer nothing. No "Done without a count" either (windows.js): only an answer saves a counted set. */
function formStep(pump, topic, ctx) {
	const m = pump.moves[pump.current];
	const c = pump.count;
	const mid = bigMiddle(pump, ctx);
	mid.name.textContent = question(c);
	mid.name.tabIndex = -1;
	const stepper = dialOf(m, c, mid, {label: t("pump.count.clean.label"), none: t("pump.count.none")});
	const check = c.check ? checkWords(c.check) : null;
	const asked = check ? h("qp-card", {class: "qp-big-card"}, h("p", {class: "qp-type-body", text: check.line})) : null;
	const send = (value, answer) => heard(intent("pump.count", {value, answer}));
	const settled = settles();
	const pick = (answer) => settled(() => send(stepper.value ?? 0, answer));
	const rows = FORMS.map(([answer, words]) => h("qp-row", {heading: t(words), trailing: "action", "on:click": pick(answer)}));
	const answers = h("qp-card", {rows: true, class: "qp-big-card"}, ...rows);
	mid.keep.push(answers);
	const most = mostLine(c);
	mid.words.replaceChildren(...[most.el, asked, answers].filter(Boolean));
	const yes = check ? button("qp-primary-button", check.yes, settled(() => send(c.check.value, c.check.answer))) : null;
	const change = c.check ? button("qp-quiet-button", c.change, () => heard(intent("pump.count.clear"))) : null;
	const guide = guideButton(pump);
	const {el} = frame({head: steps(pump, ctx, {big: true}), body: mid.parts,
		foot: [yes, change, ways(guide, pump)].filter(Boolean)});
	let fresh = true; // the count the step started from: the first digit typed replaces it
	let asking = false;
	const settle = () => {
		const value = stepper.value ?? 0;
		most.show(value);
		const was = asking;
		asking = Boolean(c.check) && value === c.check.value;
		// the focus on the answers moves to the Yes that takes their place, and back
		const held = [yes, change, ...rows].some((x) => x?.contains(document.activeElement));
		for (const x of [asked, yes, change]) if (x) x.hidden = !asking;
		answers.hidden = asking;
		if (asking !== was && held) (asking ? yes : rows[0]).focus({preventScroll: true});
	};
	stepper.addEventListener("qp-change", settle);
	settle();
	return {
		el,
		big: true,
		primary: () => (asking ? yes : mid.name),
		update(next) {
			guide.show(next.guide);
			settle();
		},
		onKey(e) {
			const typing = stepper.contains(document.activeElement);
			// while "Is that right?" asks, Enter on the dial's value presses its Yes, as the count's does (on a − or + it
			// steps)
			if (asking && e.key === "Enter" && stepper.control.contains(document.activeElement)) {
				yes.control.click();
				return true;
			}
			const types = typing || !asking;
			if (types && /^[0-9]$/.test(e.key)) {
				if (fresh) stepper.set(Number(e.key));
				else stepper.type(Number(e.key));
				fresh = false;
				return true;
			}
			if (types && e.key === "Backspace") stepper.erase();
			else if (e.key === "ArrowUp" || e.key === "ArrowRight") stepper.step(1);
			else if (e.key === "ArrowDown" || e.key === "ArrowLeft") stepper.step(-1);
			else return false;
			fresh = false;
			return true;
		},
	};
}

/** An app pump's count, and since 2026-10-12 a catch-up's for a move the engine dosed (the game's result view), set
 * on the run view's dial since slice 66 (Rapha, 2026-11-04: "maybe just in the circle adjust the times and reps etc.
 * instead of new screen (its bad ux anyways)"; a count view of its own until then): Done keeps the move's figure on its
 * stage and stops the clock, the question naming the move takes the name's place, and the dial holds the count, − and +
 * at its sides (the digits, Backspace, the arrows and the wheel step and type it; the first digit replaces the count it
 * started from); a counted set's form step (`formStep`, slice 42). Under the dial: the line while the count stands at
 * its most (`mostLine`), on a loaded move the weight box (the keys go to it while it has the focus; "." or "," move it
 * there), "Don't know the weight" under it while it holds one, and what Save keeps with a weight ("16 reps with
 * 11 kg"), and "Is that right?" for a count (or a weight) far from the record, its Yes in Save's place and Change it in
 * "Could not do one"'s while the dial and the box hold the count it asks about; a new count there takes Save back (the
 * phones' CountStep.changed()), which sends that count to be checked again. Save never takes 0 on a first max set;
 * Back. Enter is the primary. The record chip shows on a count past the reps record, but not with a weight on a move
 * with a weighted record, a score the reps alone cannot tell. */
export function counting(pump, topic, ctx) {
	if (pump.count.question === "form") return formStep(pump, topic, ctx);
	const m = pump.moves[pump.current];
	const c = pump.count;
	const mid = bigMiddle(pump, ctx);
	const ask = question(c);
	mid.name.textContent = ask;
	const stepper = dialOf(m, c, mid, {label: ask});
	const weight = weightBox(c);
	// "Don't know the weight" under the box while it holds a weight (David, 2026-11-06: "there should be a "dont know
	// weight" option here"): it empties the box, so Save keeps the reps alone; − or + brings the planned weight back
	const unknown = weight ? button("qp-quiet-button", t("pump.count.kg.unknown"), () => {
		weight.clear();
		weight.focus({preventScroll: true});
	}) : null;
	mid.keep.push(...[weight, unknown].filter(Boolean));
	const kept = weight ? h("p", {class: "qp-big-line"}) : null;
	const most = mostLine(c);
	const check = c.check ? checkWords(c.check) : null;
	const asked = check ? h("qp-card", {class: "qp-big-card"}, h("p", {class: "qp-type-body", text: check.line})) : null;
	mid.words.replaceChildren(...[most.el, weight, unknown, kept, asked].filter(Boolean));
	// the count comes on the press that ended the move (its Done, which Enter is while it runs) and starts at a count of
	// its own (a hold's stopwatch, the voice's reps), and "Is that right?" comes on the press of Save: Save, Yes and
	// Could not do one answer once it has settled, so that press pressed twice never answers them, as the phones' foot
	const settled = settles();
	const count = (value, load) => settled(() => heard(intent("pump.count", {value: value(), load: load() ?? null})));
	const save = button("qp-primary-button", t("pump.count.save"), count(() => stepper.value, () => weight?.value));
	const couldNot = c.could_not
		? button("qp-quiet-button", c.could_not, settled(() => heard(intent("pump.count.couldnot")))) : null;
	const yes = check ? button("qp-primary-button", check.yes, count(() => c.check.value, () => c.check.load)) : null;
	const change = c.check ? button("qp-quiet-button", c.change, () => heard(intent("pump.count.clear"))) : null;
	const guide = guideButton(pump);
	const foot = [yes, change, save, couldNot, ways(guide, pump)].filter(Boolean);
	let fresh = true; // the count the step started from: the first digit typed replaces it
	let freshWeight = true; // so the weight planned
	let busy = pump.busy;
	let asking = false;
	const settle = () => {
		const value = stepper.value ?? 0;
		const load = weight?.value ?? null;
		most.show(value);
		if (kept) {
			kept.textContent = load != null && value > 0 ? preview(value, load, c.kg.each) : "";
			kept.hidden = !kept.textContent;
		}
		if (unknown) unknown.hidden = load == null;
		stepper.record = load != null && c.kg?.top ? null : c.chip;
		const was = asking;
		asking = Boolean(c.check) && stepper.value === c.check.value && load === (c.check.load ?? null);
		// the focus on the foot's answers moves to the primary that takes their place, so Enter presses it
		const held = [yes, change, save, couldNot].some((el) => el?.contains(document.activeElement));
		for (const el of [asked, yes, change]) if (el) el.hidden = !asking;
		for (const el of [save, couldNot]) if (el) el.hidden = asking;
		// Save disabled at 0 on a max set would drop its focus to the page: the dial keeps it
		if (c.max_set && value === 0 && save.contains(document.activeElement)) stepper.control.focus({preventScroll: true});
		save.state = c.max_set && value === 0 ? "disabled" : busy ? "busy" : "normal";
		if (asking !== was && held) (asking ? yes : save).focus({preventScroll: true});
	};
	stepper.addEventListener("qp-change", settle);
	weight?.addEventListener("qp-change", settle);
	settle();
	/** A key in the weight box: digits and the point type (the first replaces the weight planned), Backspace erases,
	 * the arrows step. */
	const weightKey = (e) => {
		const point = e.key === "." || e.key === ",";
		if (/^[0-9]$/.test(e.key) || point) {
			if (freshWeight) weight.clear();
			if (point) weight.point();
			else weight.type(Number(e.key));
		} else if (e.key === "Backspace") weight.erase();
		else if (e.key === "ArrowUp" || e.key === "ArrowRight") weight.step(1);
		else if (e.key === "ArrowDown" || e.key === "ArrowLeft") weight.step(-1);
		else return false;
		freshWeight = false;
		return true;
	};
	const {el} = frame({head: steps(pump, ctx, {big: true}), body: mid.parts, foot});
	return {
		el,
		big: true,
		primary: () => (asking ? yes : save),
		update(next) {
			busy = next.busy;
			guide.show(next.guide);
			settle();
		},
		onKey(e) {
			// Enter in the dial's or the weight's value presses the primary, as it would from it (the game's Enter saves
			// the count in its box); on a − or + it steps, the button's own
			if (e.key === "Enter" && [stepper, weight].some((box) => box?.control.contains(document.activeElement))) {
				(asking ? yes : save).control.click();
				return true;
			}
			// the weight box takes the keys while it has the focus; a decimal point anywhere moves the focus there
			if (weight && (e.key === "." || e.key === ",") && !weight.contains(document.activeElement)) {
				weight.focus({preventScroll: true});
			}
			if (weight?.contains(document.activeElement)) return weightKey(e);
			if (/^[0-9]$/.test(e.key)) {
				if (fresh) stepper.set(Number(e.key));
				else stepper.type(Number(e.key));
				fresh = false;
				return true;
			}
			if (e.key === "Backspace") stepper.erase();
			else if (e.key === "ArrowUp" || e.key === "ArrowRight") stepper.step(1);
			else if (e.key === "ArrowDown" || e.key === "ArrowLeft") stepper.step(-1);
			else return false;
			fresh = false;
			return true;
		},
	};
}

// The end ------------------------------------------------------------------------------------------------------------

/** A calm card: its title, if any, over its lines. */
function calmCard(titleText, lines, glyph) {
	const head = titleText ? h("div", {class: "qp-pump-calm-head"}, glyph ? h("qp-art", {name: glyph}) : null,
		h("h2", {class: "qp-type-heading", text: titleText})) : null;
	return h("qp-card", {class: "qp-pump-calm"}, head, ...lines.filter(Boolean));
}

/** A catch-up's payment, in words. */
function paidLine(paid) {
	switch (paid) {
		case "paid": return t("desk.pump.done.paid");
		case "unpaid": return t("desk.pump.done.unpaid");
		case "reset": return t("desk.pump.done.reset");
		case "locked": return t("desk.pump.done.locked");
		case "game": return t("pump.done.gameruns");
		default: return null;
	}
}

/** The calm end (felt unwell, ended early, ended by the game, ended before a move ran): no fanfare, no sound, no reveal.
 * The game ends an app pump with its own pump's offer, and a catch-up, or a match pump on a rider (slice 61), as soon
 * as it runs, which asks those moves itself. */
function calm(pump) {
	const c = pump.calm;
	const words = (key, role = "body") => h("p", {class: `qp-type-${role}`, text: key});
	switch (c.why) {
		case "unwell":
			return calmCard(c.title, [
				words(c.until ? t("pump.done.rest.until", {time: time(c.until)}) : t("pump.done.rest"), "body-strong"),
				c.reload ? words(t("pump.done.reload"), "subtitle") : null,
				c.paid ? words(paidLine(c.paid), "subtitle") : null,
			], "mark_heart");
		case "left":
			// a catch-up left at Bank the rest leaves the screen at once (`pump_stop`)
			return calmCard(t("pump.done.early"), [words(t("pump.leave.free"))]);
		case "game": {
			// the game runs the riders: on one, or past riders done that it now asks unpaid (`paid`)
			const runs = pump.moves[pump.current]?.rider || pump.result?.paid === "game";
			return calmCard(t("pump.done.early"), [words(runs ? t("pump.done.gameruns") : t("pump.done.gameasked"))]);
		}
		default:
			return calmCard(null, [words(t("pump.done.ended"))]);
	}
}

/** The Legendary moment over the Result (2026-10-09; the phones' pump.legendary, the game's Sheet.Legend): a Window
 * headed by the step's quality word in its tone, the move's figure in the quality's light (its still under reduced
 * motion), the move's name in that tone, its ladder and step, Share, which opens the unlock card, and Continue, which
 * goes on to the next such step or closes the Result (Rust's `pump.continue`, which also has the voice name the move).
 * Its × or Escape puts it away and leaves the Result, whose Continue goes on from it. */
function legendWindow() {
	const box = h("qp-window", {class: "qp-legend", primary: t("pump.done.continue")});
	const figure = h("qp-figure-stage", {class: "qp-legend-figure"});
	const name = h("p", {class: "qp-type-heading qp-toned qp-pump-centre"});
	const step = h("p", {class: "qp-type-meta qp-pump-centre"});
	let card = null;
	const share = h("qp-secondary-button", {label: t("kit.share"), "on:click": () => card && openShare(card)});
	box.content.append(figure, name, step, share);
	box.addEventListener("qp-primary", () => heard(intent("pump.continue")));
	return {
		el: box,
		/** The moment of the Result's row `row` (an orange or gold step's unlock). */
		show(row) {
			const u = row.reward;
			const tone = qualityName(u.quality);
			const colour = getComputedStyle(document.documentElement).getPropertyValue(`--qp-quality-${tone}`).trim();
			box.style.setProperty("--qp-tone", `var(--qp-quality-${tone})`);
			box.heading = u.word;
			const moveName = row.move?.name ?? u.ladder;
			figure.move = row.move?.key ?? null;
			figure.colour = colour || null;
			name.textContent = moveName;
			step.textContent = t("kit.reward.step", {ladder: u.ladder, step: u.step});
			const character = get("book")?.character;
			card = {kind: "unlock", key: row.move?.key, name: moveName, title: t("ladders.card.unlocked"),
				meta: character ? t("ladders.card.footer", {character, ladder: u.ladder}) : u.ladder,
				quality: u.quality, qualityWord: u.word, step: u.step};
			box.open();
			announce(t("kit.legend.label", {quality: u.word, move: moveName}));
		},
	};
}

/** The Result's foot and its main action: Continue; in the browser, while its `play` topic offers Save here (its first
 * pump's Result; docs/plan.md slice 63, "Save your progress") and this is a Result the offer counts (the pump's
 * `ended_done`, the desk's: a move done, never felt unwell's calm card or a pump that ended before a move ran;
 * questpump-web `Play::seen` reads the same), Save your progress as the main action, which
 * puts the offer away, ends the first five minutes where they stand (their Hero step would cover the Email page),
 * continues and opens the Email page in the Menu on its Save flow (`save`), with Not now beside it, which puts it
 * away and continues. The desktop app publishes no `play` topic. */
function saveOffer(pump, continueButton) {
	if (get("play")?.offer !== "result" || !pump.ended_done) {
		return {foot: [continueButton], primary: continueButton};
	}
	const save = button("qp-primary-button", t("menu.sync.save.title"), () => {
		const first = () => (get("welcome")?.step ? intent("welcome.done") : null);
		heard(intent("play.later").then(first).then(() => intent("pump.continue"))
			.then(() => openMenu("desk.menu.email", {save: true})));
	});
	const later = button("qp-quiet-button", t("menu.sync.notnow"), () => {
		heard(intent("play.later").then(() => intent("pump.continue")));
	});
	return {foot: [save, later], primary: save};
}

/** The pump's end (mockup pump-done): the fanfare and what the pump earned, played by the Result's one reveal on the
 * instants Rust gave (the XP counting up as the bar fills, the banner turning at a level, a Paragon rank or a streak
 * milestone, the reward rows one by one, each move the pump did first with its record and its progress, 2026-10-29)
 * and a catch-up's payment (no day-one soreness line since David, 2026-10-24: only what is legally necessary); a click lands the reveal (Reduce Motion lands it at once). A
 * streak milestone's Share (slice 45), under the banner once it turned to the milestone and on the milestone's row,
 * opens the Share window on the streak card. A pump that ended short ends calm, never with the fanfare, the reveal or a
 * sound: felt unwell; left part way or ended by the game's own pump, with what it kept and what the app's own sheet
 * paid for it, shown still; ended before a move ran. Continue (Enter) closes it. */
export function done(pump, topic, ctx) {
	const r = pump.result;
	let reveal = pump.reveal;
	const body = [];
	const continueButton = button("qp-primary-button", continueWords(pump), () => heard(intent("pump.continue")));
	const celebrates = Boolean(r) && !pump.calm;
	const kept = Boolean(r) && pump.calm && (pump.calm.why === "left" || pump.calm.why === "game");
	// an expedition's Result (slice 51): its own parts around the pump's; felt unwell shows its card alone
	const trip = pump.expedition?.done && r ? tripResult(pump) : null;
	if (trip) body.push(...trip.head);
	if (pump.calm && !trip) body.push(calm(pump));
	let draw = () => false;
	if (celebrates || kept) {
		const banner = celebrates && r.banner ? h("qp-fanfare", {kind: r.banner.kind, number: r.banner.number}) : null;
		const bannerShare = banner ? h("qp-round-button", {icon: "share", class: "qp-pump-banner-share",
			"on:click": () => openShare(streakCard(banner.number, true))}) : null;
		const shareShown = () => {
			if (bannerShare) bannerShare.hidden = banner.kind !== "streak";
		};
		shareShown();
		const gain = h("p", {class: "qp-pump-gain"}, h("span", {class: "qp-gold", text: "+"}),
			h("span", {class: "qp-type-num-large", text: "0"}));
		const gainWord = h("p", {class: "qp-type-caps qp-pump-centre", text: t("pump.done.experience")});
		const from = r.bar?.from ?? r.bar;
		const bar = r.bar ? h("qp-xp-bar", {level: from.level, into: from.into, need: from.need, paragon: from.paragon,
			rested: r.bar.rested}) : null;
		const rows = r.rows.map((row) => {
			const el = h("qp-reward-row");
			el.reward = reward(row.reward);
			el.move = row.move;
			if (row.reward.kind === "milestone") {
				el.share = true;
				el.addEventListener("qp-share", () => openShare(streakCard(row.reward.days, true)));
			}
			return el;
		});
		const list = h("qp-card", {class: "qp-pump-rows", hidden: true});
		const xp = r.xp != null && (celebrates || r.xp > 0);
		body.push(...[banner, bannerShare, trip?.line, xp ? gain : null, xp ? gainWord : null, bar, list].filter(Boolean));
		const number = gain.lastChild;
		let turned = false;
		let filled = false;
		let shownRows = 0;
		draw = (now) => {
			// a calm end's rewards stand still at their end, as a landed reveal's
			const landed = !celebrates || !reveal || reveal.landed;
			const at = (instant) => landed || (instant != null && now >= instant);
			if (banner && r.turn && !turned && at(reveal?.turn)) {
				turned = true;
				banner.kind = r.turn.kind;
				banner.number = r.turn.number;
				shareShown();
			}
			if (xp) {
				const fill = landed ? 1 : Math.min(1, Math.max(0, (now - reveal.gain) / Math.max(1,
					(reveal.rows[0] ?? reveal.end) - reveal.gain)));
				number.textContent = new Intl.NumberFormat().format(Math.round(r.xp * fill));
			}
			if (bar && !filled && at(reveal?.gain)) {
				filled = true;
				Object.assign(bar, {level: r.bar.level, into: r.bar.into, need: r.bar.need, paragon: r.bar.paragon});
			}
			while (shownRows < rows.length && at(reveal?.rows[shownRows])) {
				rows[shownRows].shown = celebrates;
				list.append(rows[shownRows]);
				shownRows += 1;
			}
			list.hidden = shownRows === 0;
			return !landed && now < reveal.end;
		};
		if (!ctx.told(pump.id)) {
			if (celebrates) {
				announce([banner?.spoken() ?? "", xp ? t("pump.reward.xp", {xp: r.xp}) : "",
					...rows.flatMap((row) => row.spoken())].filter(Boolean).join(". "));
			}
			// the game day's last ember a move lit (slice 55), a kept end's too: the window's one Toast host shows it
			// once the reveal is over
			if (r.embers != null) document.querySelector("qp-toast")?.show({kind: "embers", ...r.embers});
		}
		if (celebrates && reveal && !reveal.landed && matchMedia("(prefers-reduced-motion: reduce)").matches) {
			heard(intent("pump.land", {still: true}));
		}
	}
	// a match pump's riders the game runs for are its calm card's words already
	if (r?.paid && !(pump.calm?.why === "game" && r.paid === "game")) {
		body.push(h("p", {class: "qp-type-subtitle qp-pump-centre", text: paidLine(r.paid)}));
	}
	if (trip) body.push(...trip.tail);
	const offer = trip ? null : saveOffer(pump, continueButton);
	const {el, column} = frame({body, foot: trip ? trip.foot : offer.foot});
	// the Share window by itself as the reveal ends (at once on one ended part way)
	let opened = false;
	const reveals = draw;
	draw = (now) => {
		const going = reveals(now);
		if (trip && !going && !opened) {
			opened = true;
			queueMicrotask(() => trip.opened());
		}
		return going;
	};
	if (celebrates && reveal && !reveal.landed) {
		column.addEventListener("click", () => heard(intent("pump.land")), {once: true});
	}
	// the Legendary moment of each orange or gold step, one a Continue (`pump.legend`, the row's index)
	const legend = legendWindow();
	el.append(legend.el);
	let legendShown = null;
	const showLegend = (index) => {
		if (index == null || index === legendShown || !r?.rows[index]) return;
		legendShown = index;
		legend.show(r.rows[index]);
	};
	showLegend(pump.legend);
	return {
		el,
		light: celebrates,
		primary: () => (trip ? trip.primary : offer.primary),
		frame: (now) => draw(now),
		update(next) {
			reveal = next.reveal;
			showLegend(next.legend);
		},
	};
}
