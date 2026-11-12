// The pump on the computer (docs/ui.md, "The pump on the computer"; the phones' PumpSession): while the `pump` topic
// holds a pump it covers the whole window, the rail with it, one stage at a time (stages.js), drawn from Rust's state
// and instants; the stage a change brings takes the keys and the focus (its primary), with the keys' half-second guard
// again, Enter (and Space where the game's presses it) pressing its primary also where a click left the focus on no
// control (`pressPrimary`), and a window a stage opened closes as the stage moves on. Big view (the countdown, a
// running move, the timed ask, the count on its dial, a hold's longer ask, a pose's ask, the pour) has the window's own full screen
// beside the steps, its toggle showing the window as it is now (`shell`'s `fullscreen_now`) and sent through the pump
// (`pump.fullscreen`), so the pump's end leaves a full screen it put on.
// The move's one figure plays from its Ready view through the countdown, the run and its count (Big view's PumpStage),
// one for every stage of a move (a set of reps' loop phased to its go from the countdown on), stopped once no stage
// holds it. One stage changes into the next with the stage's cross-fade (lib/stage.js, slice 72): the stage that leaves
// fades out behind the one that comes, around the figure, which stays where it stands, and an expedition's bar. Where
// the next stage gives the figure another room (Ready's into Big view's, on the same side of the window) it glides
// there in the fade's time (`glide`, 2026-11-11), and the stage says what it came from (`data-from`), by which
// pump.css plays what only that change brings (the dial's sweep at Start and at Go, the dial's keys coming out at
// Done, the figure dimming at Save).
// While the window has lost the focus mid-pump (the `shell` topic's `focused`, Rust's), a veil says to click to control
// the pump, since the keys would reach the game; the clock and the voice run on under it.

import {get, intent, subscribe} from "../../lib/bridge.js";
import {still} from "../../lib/carry.js";
import {t} from "../../lib/copy.js";
import {MoveFigure} from "../../lib/figure.js";
import {focusView, pressesNothing} from "../../lib/focus.js";
import {mounted} from "../../lib/keys.js";
import {cover, isFront, register, toCover, uncover, whenStarted} from "../../lib/router.js";
import {change} from "../../lib/stage.js";
import {h} from "../../kit/element.js";
import {asking, countdown, counting, done, longer, marking, pacing, ready, resting, running, saved} from "./stages.js";
import {leave, more, move, pauseMenu, swap} from "./windows.js";

const STAGES = {ready, countdown, running, asking, counting, longer, saved, marking, pacing, done, rest: resting};

/** The parts of a pump no stage's fade cuts in two: the countdown and the run, whose figure is one film, and the count
 * and its pour, which drains the dial the count was set on. */
const PARTS = {countdown: "run", running: "run", counting: "count", saved: "count"};

/** A stage as its fade reads it: the pump, the part of it the stage belongs to and the move it is on. */
function shotOf(pump) {
	return {id: pump.id, part: PARTS[pump.stage] ?? pump.stage, move: pump.current};
}

/** Where a stage's figure stands on the window: its square's box, none for a stage with no figure. */
function standsAt(stageEl) {
	const square = stageEl?.querySelector("qp-figure-stage")?.square;
	return square?.isConnected ? square.getBoundingClientRect() : null;
}

/** The figure in `holder` comes from where it stood (`from`, the leaving stage's `standsAt`) to its room here over the
 * stage's time on its curve; at once where it stands as it stood and under Reduce Motion. */
function glide(holder, from) {
	if (!holder || !from?.width || still.matches) return;
	const square = holder.square;
	const to = square.getBoundingClientRect();
	if (!to.width) return;
	const [dx, dy] = [from.left + from.width / 2 - to.left - to.width / 2,
		from.top + from.height / 2 - to.top - to.height / 2];
	const scale = from.width / to.width;
	if (Math.abs(dx) < 1 && Math.abs(dy) < 1 && Math.abs(scale - 1) < 0.01) return;
	const css = getComputedStyle(holder);
	const stood = `translate(${dx}px, ${dy}px) scale(${scale})`; // copy-lint: not shown (a transform)
	square.animate([{transform: stood}, {transform: "none"}],
		{duration: parseFloat(css.getPropertyValue("--qp-motion-stage-duration")) || 0,
			easing: css.getPropertyValue("--qp-motion-stage-easing").trim() || "ease"});
}
// copy-lint: not shown (the cover's view id)
const COVER = "desk.pump";

/** Enter, or Space where the game's Space presses it (the stage's `space`: Start, and Switch sides on a side move's
 * first side; the game's Space never ends a move), on nothing a key presses (a click on the page's ground, its words or
 * its figure leaves the focus there, since a mouse click focuses no button in WebKit): the stage's primary, as with the
 * focus on it, as the game's keys need no focus (Panel.HandleKey; 2026-10-25, David: "also make start button like in
 * the addon startable (space or enter dont remember)"); nothing with Cmd, Ctrl or Alt, nor where the stage's primary is
 * its heading, which no key answers. Whether it took the key. */
function pressPrimary(stage, e) {
	if (e.metaKey || e.ctrlKey || e.altKey || !pressesNothing(e.target)) return false;
	if (e.key !== "Enter" && !(e.key === " " && stage?.space)) return false;
	const control = stage?.primary?.()?.control;
	if (!(control instanceof HTMLButtonElement)) return false;
	control.click();
	return true;
}

/** What makes a stage new: everything Rust sent of the pump but its busy state, the read-aloud button's state, the
 * countdown's digit, whether its reveal landed and a move's pour as it goes on (its landing, its rewards, its Back),
 * which the stage on screen takes as it stands (so the countdown's Back and read-aloud button keep the focus through
 * its digits, and the pour plays on through its write). */
function keyOf(topic) {
	const pump = topic.pump;
	// a move's saved stage is one pour from its plan to Next: its own row's write changes the moves under it, its
	// earned summary, landing and Back follow through `update`
	if (pump.stage === "saved") {
		return JSON.stringify({pump: {id: pump.id, stage: pump.stage, current: pump.current, at: pump.saved?.at},
			locked: topic.locked_until});
	}
	return JSON.stringify({pump: {...pump, busy: false, guide: null, digit: null,
		reveal: pump.reveal && {...pump.reveal, landed: false}}, locked: topic.locked_until});
}

register(COVER, () => {
	const page = h("qp-page", {class: "qp-pump"});
	const veil = h("qp-focus-veil", {message: t("desk.pump.focus"), hidden: true});
	// an expedition's bar over each round's stages and the breather (slice 51), kept across them so its clock ticks on
	const bar = h("qp-expedition-bar");
	const barRow = h("div", {class: "qp-expedition-bar-row"}, bar);
	let stage = null;
	let key = null;
	let shot = null;
	// the stage the one on the page came from, by its name (`data-from`)
	let from = null;
	let win = null;
	let raf = 0;
	let topic = get("pump");
	const told = new Set();
	const fullscreens = new Set();
	const figure = new MoveFigure();
	const view = {
		el: page,
		stops: [],
		primary: () => stage?.primary?.() ?? null,
		onKey: (e) => stage?.onKey?.(e) || pressPrimary(stage, e),
	};

	const fullscreenWords = (on) => (on ? t("desk.pump.fullscreen.off") : t("desk.pump.fullscreen.on"));
	const ctx = {
		/** The move's one figure, which Ready and Big view hold from Ready through the run (qp-figure-stage `hold`). */
		figure,
		/** Opens a stage's window: the "···", Swap's options, How to do it, Pause's menu from its button `from`,
		 * leaving the pump. */
		open(kind, from) {
			win?.close?.();
			const pump = topic.pump;
			if (kind === "more") win = more(pump, ctx.open);
			else if (kind === "swap") win = swap(pump);
			else if (kind === "move") win = move(pump);
			else if (kind === "pause") win = pauseMenu(topic, from);
			else win = leave(pump, null);
		},
		/** Big view's full-screen toggle: the window's own as it is now, the choice remembered (`pump.fullscreen`). */
		fullscreen() {
			const on = Boolean(get("shell")?.fullscreen_now);
			const toggle = h("qp-quiet-button", {class: "qp-pump-full", label: fullscreenWords(on), "on:click": () => {
				intent("pump.fullscreen", {on: !get("shell")?.fullscreen_now}).catch(console.error);
			}});
			fullscreens.add(toggle);
			return toggle;
		},
		/** Whether the Result of pump `id` was said already (once each). */
		told(id) {
			if (told.has(id)) return true;
			told.add(id);
			return false;
		},
	};

	const draw = () => {
		raf = 0;
		if (stage?.frame?.(Date.now())) raf = requestAnimationFrame(draw);
	};

	const veiled = () => {
		const shell = get("shell");
		veil.hidden = !(topic?.pump && topic.pump.stage !== "done" && shell && !shell.focused);
	};

	const render = (next) => {
		topic = next;
		if (!topic.pump) return;
		const now = keyOf(topic);
		const trip = topic.pump.expedition;
		const barred = Boolean(trip) && topic.pump.stage !== "done";
		if (trip) bar.trip = trip;
		if (now === key && stage) {
			stage.update?.(topic.pump);
			if (!raf) draw();
			return;
		}
		key = now;
		win?.close?.();
		win = null;
		// Your setup over the pump stands only over its Ready view
		if (topic.pump.stage !== "ready") toCover();
		fullscreens.clear();
		const stood = standsAt(stage?.el);
		page.dataset.from = from ?? "";
		from = topic.pump.stage;
		stage = STAGES[topic.pump.stage](topic.pump, topic, ctx);
		if (!stage.el.contains(figure.el)) figure.stop();
		page.classList.toggle("qp-big", Boolean(stage.big));
		// the floating Result: the cover see-through, the page under the pump showing round its card
		page.classList.toggle("qp-pump-floating", Boolean(stage.floats));
		page.light = stage.light ? "result" : null;
		stage.el.classList.toggle("qp-expedition-stage", barred);
		if (barred) stage.el.prepend(barRow);
		// the stage's change (slice 72, the phones' StageFade): within one part of a move (the countdown into its run,
		// the count into its pour) the next moment comes at once; another stage fades in, over the one that leaves
		// only where that one's words stay true, the same move's Ready view or its countdown and run, around the
		// figure in place and an expedition's bar; any other stage leaves at once. The floating Result comes by its
		// card's own motion (resultFloatIn), with no fade of the stage over it
		const was = shot;
		shot = shotOf(topic.pump);
		const moved = !was || was.id !== shot.id || was.move !== shot.move;
		if (stage.floats || (was && !moved && was.part === shot.part)) {
			page.replaceChildren(stage.el, veil);
		} else {
			const under = Boolean(was) && !moved && (was.part === "ready" || was.part === "run");
			const holder = under && stage.el.contains(figure.el) ? figure.el.closest("qp-figure-stage") : null;
			change(page, [stage.el, veil], [holder, barred ? barRow : null, veil], {under});
			glide(holder, stood);
		}
		veiled();
		cancelAnimationFrame(raf);
		draw();
		// the keys and the focus only while the pump is in front: not before its cover placed it, nor under Your setup
		// its Settings opened over it (lib/router.js `over`), which keeps them
		if (isFront(view)) {
			mounted(view);
			focusView(view);
		}
	};

	view.stops.push(subscribe("pump", render), subscribe("shell", (shell) => {
		for (const toggle of fullscreens) toggle.label = fullscreenWords(shell.fullscreen_now);
		veiled();
	}), () => {
		cancelAnimationFrame(raf);
		win?.close?.();
		figure.stop();
	});
	return view;
});

whenStarted(() => {
	let covered = false;
	subscribe("pump", (topic) => {
		if (topic.pump && !covered) {
			covered = true;
			cover(COVER);
		} else if (!topic.pump && covered) {
			covered = false;
			uncover();
		}
	});
});
