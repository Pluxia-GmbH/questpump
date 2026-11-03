// Correct a count's window on the computer (docs/plan.md slice 44, "One correction, from any device"; screens.json
// desk.book.correct, the phones' correct.window drawn once more in their words and states), opened by Correct a count on
// a move's page. Its list is the sets the Book carries for the move while its page stands (`book`'s move `sets`: the
// engine's correctable, the move's sets of the last correct_days game days after the window's reach, a margin of rows
// past the game's ring horizon, the newest ten and the record's set where it is older), newest first, each its value as
// the fold reads it over its game day's date and its time, Record beside the record's set; where the record's set is not
// listed, "Your record is from an older set…", or with no set "No set of the last 14 days to correct.", each with Clear
// this record while the move has a record (the move page's confirmation, the window closed first). A set opens its
// correction in the same window: what was saved and when ("You saved no count on …" for a set with no number), the
// count step's control for its kind at the count step's limits (the stepper of reps or seconds, a counted set's Not
// counted at 0, a loaded set's weight box with what Save keeps, a pose's four answers; Done without a count beside a rep
// or hold set, the 0 a correction keeps: slice 44's review, 2026-10-18), Save, disabled until the value differs and
// while a pump runs (the `pump` topic's `running`, Rust's own refusal: its moves under way, never its Result; "A pump
// is on…" pinned under it), and Pick another set; on a set of this computer's whose winning correction came from
// another device while this side still reads it otherwise (its `ownSide`, 2026-10-19), "Corrected to 15 on another
// device…" over a Save ready at the value it reads now, while the input still holds that value. Save sends
// `book.correct` with its own number, and Rust answers on the `book.correct` topic: "Is that right?" once in the count
// step's words (its Yes sends the count again with `confirm`, Change it goes back to the value), one set's most, the
// set gone (back on the list with its line), nothing written (the view as it stood, its Save or Yes ready again), or
// done: "Corrected to …" (the set as the fold reads it now, as the list says it next; "Corrected to no count." with no
// number), what the rows changed on the steps (a ladder back to or up to a step, a step no longer or now Mastered:
// 2026-10-19), why the step stays and Done. The digits, Backspace and the arrows edit the value in focus, Enter on it
// saves; Escape closes the window as any window, and the focus goes back to Correct a count.

import {get, intent, subscribe} from "../../lib/bridge.js";
import {t} from "../../lib/copy.js";
import {announce} from "../../lib/focus.js";
import {kg, LOCALE} from "../../lib/format.js";
import {h} from "../../kit/element.js";
import {ANSWERS, amount, checkWords, lifted, preview, question} from "../pump/words.js";
import {factsOf, moveName, recordWords} from "./common.js";

/** Each Save's number, which its outcome on the `book.correct` topic carries back: the window shows its latest alone. */
let saves = Date.now();

/** A set's game day's date and its time, as the computer writes them ("Tue 14 Oct, 18:04"). */
const WHEN = new Intl.DateTimeFormat(LOCALE, {weekday: "short", day: "numeric", month: "short", hour: "numeric",
	minute: "2-digit"});

/** A set's moment: its game day's date (the day turns at `dayOffsetH`, 04:00, as the game's ns.DayMid: a set at 01:00
 * is the day before's) at its own time on the clock. */
function when(w, ts) {
	const at = new Date(ts * 1000);
	const day = new Date((ts - w.words.dayOffsetH * 3600) * 1000);
	at.setFullYear(day.getFullYear(), day.getMonth(), day.getDate());
	return WHEN.format(at);
}

/** A pose's answer in the ask's words (Perfect … Not yet). */
function answerWord(code) {
	const answer = ANSWERS.find(([c]) => c === code);
	return answer ? t(answer[1]) : "";
}

/** What a set's kind and its move make of its control: a hold's seconds, a counted set (its 0 Not counted), a loaded
 * rep set's weight in each hand or in all, done on each side. */
function shapeOf(w, set) {
	const m = w.moves.get(set.move);
	const hold = set.kind === "seconds";
	const counted = set.kind === "count" && m?.dose === "cardio";
	return {hold, counted, loaded: set.kind === "count" && !counted && m?.load != null, each: m?.load === "each",
		side: m?.side === true};
}

/** A set's reading as the window says it (`value`, `load` and `done` as the fold reads them, a listed set's or a
 * correction's outcome's): a pose's answer; at 0 a counted set's Not counted, and a rep or hold set's Done without a
 * count where the fold reads it done (saved with an empty box, or corrected to 0), else Could not do one (Could not hold
 * it), a stored 0 (slice 44's review, 2026-10-18); a hold's time, a set with a weight "11 kg × 16", else "22 reps". */
function valueWords(w, set, {value, load, done}) {
	if (set.kind === "answer") return answerWord(value);
	const shape = shapeOf(w, set);
	const words = w.words.correct;
	if (value === 0) {
		if (shape.counted) return t("pump.count.none");
		return done ? words.empty : shape.hold ? words.couldNotHold : words.couldNot;
	}
	if (shape.hold) return amount(value, true);
	if (shape.loaded && load) return lifted(load, value, shape.each);
	return t("log.reps", {count: value});
}

/** Opens the window for `move` (its page's `w`, the words looked up): `clear` opens the move page's confirmation to
 * clear its record. Returns the window. */
export function openCorrectWindow(w, move, clear) {
	const name = moveName(w, move);
	const limits = w.words.correct;
	const win = h("qp-window", {heading: t("correct.window.title", {move: name})});
	let book = get("book");
	let pump = get("pump");
	let view = "list"; // list, edit, really, done
	let set = null; // the set being corrected, as the list named it
	let value = null; // its new count (a hold's seconds) or a pose's answer
	let load = null; // a loaded set's new weight, tenths of a kg (none: no weight)
	let mine = 0; // the number of this window's latest Save
	let busy = false;
	let gone = false; // the list says the set picked has left it
	let outcome = null; // the latest Save's outcome
	let control = null; // the edit view's value in focus first: the stepper or the first answer
	let elsewhere = null; // the edit view's "Corrected to … on another device" line, on a set with `ownSide`
	let save = null; // what Save and Enter do now
	const sets = () => factsOf(book).get(move)?.sets ?? [];
	const running = () => Boolean(pump?.running);

	const line = (text, cls = "qp-type-body qp-text2") => h("p", {class: cls, text});
	const clearRow = () => h("qp-row", {heading: t("ladders.journal.clear"), glyph: "mark_close", trailing: "action",
		destructive: true, "on:click": () => {
			win.close();
			clear();
		}});

	/** The list of sets, with what holds the record where none of them does. */
	function list() {
		const all = sets();
		const record = recordWords(w, book, factsOf(book), move) != null;
		const parts = [];
		if (gone) parts.push(line(t("correct.window.gone"), "qp-type-body"));
		if (all.length) {
			parts.push(h("qp-section-label", {label: t("correct.window.sets", {days: limits.days})}),
				h("qp-card", {rows: true}, ...all.map((s, i) => h("qp-row", {heading: valueWords(w, s, s),
					meta: when(w, s.ts), trailing: s.record ? "value" : "nav",
					value: s.record ? t("ladders.record.label") : null, "data-focus-key": `set:${i}`,
					"on:click": () => edit(s)}))));
			if (record && !all.some((s) => s.record)) parts.push(line(t("correct.window.old")), clearRow());
		} else {
			parts.push(line(t("correct.window.empty", {days: limits.days})));
			if (record) parts.push(clearRow());
		}
		return parts;
	}

	/** The correction of the set picked: what was saved and when (a set with no number under its own heading), then its
	 * control. */
	function editParts() {
		const date = when(w, set.ts);
		const parts = [line(set.kind === "answer" ? t("correct.window.saved.pose", {answer: answerWord(set.value), date})
			: set.value === 0 ? t("correct.window.saved.none", {date})
			: t("correct.window.saved", {value: valueWords(w, set, set), date}))];
		// another device's correction won while this side reads the set otherwise: Save writes it here as its own (a set
		// with no number in the done view's words, correct.window.done.none's); shown while the input holds the set as
		// it reads now (`settle`)
		elsewhere = set.ownSide ? line(set.kind !== "answer" && set.value === 0 ? t("correct.window.elsewhere.none")
			: t("correct.window.elsewhere", {value: valueWords(w, set, set)}), "qp-type-body") : null;
		if (elsewhere) parts.push(elsewhere);
		if (set.kind === "answer") {
			const rows = ANSWERS.map(([code, word]) => h("qp-row", {heading: t(word), trailing: "checked",
				checked: value === code, "on:click": () => {
					value = code;
					for (const [i, row] of rows.entries()) row.checked = ANSWERS[i][0] === code;
					settle();
				}}));
			control = rows.find((r) => r.checked) ?? rows[0];
			parts.push(h("p", {class: "qp-type-heading", text: t("pump.mark.ask.title")}), h("qp-card", {rows: true},
				...rows));
			return parts;
		}
		const shape = shapeOf(w, set);
		const most = shape.counted ? limits.counted : shape.hold ? limits.seconds : limits.reps;
		const stepper = h("qp-stepper", {value, least: 0, most: most.most, unit: shape.hold ? "seconds" : "reps",
			none: shape.counted ? t("pump.count.none") : null});
		const box = shape.loaded ? h("qp-stepper", {decimal: true, value: load, first: set.load ?? limits.kgStep,
			most: limits.kgMost, stride: limits.kgStep,
			"unit-one": t(shape.each ? "pump.count.kg.each" : "pump.count.kg"),
			"unit-other": t(shape.each ? "pump.count.kg.each" : "pump.count.kg")}) : null;
		const kept = box ? h("p", {class: "qp-type-subtitle qp-pump-centre"}) : null;
		const max = h("p", {class: "qp-type-meta qp-pump-centre", text: most.line});
		const count = {question: shape.hold ? (shape.side ? "hold_side" : "hold") : shape.side ? "reps_side" : "reps",
			name};
		const show = () => {
			max.hidden = (stepper.value ?? 0) < most.most;
			if (kept) {
				kept.textContent = load != null && value > 0 ? preview(value, load, shape.each) : "";
				kept.hidden = !kept.textContent;
			}
		};
		stepper.addEventListener("qp-change", (e) => {
			value = e.detail;
			show();
			settle();
		});
		box?.addEventListener("qp-change", (e) => {
			load = e.detail;
			show();
			settle();
		});
		// built now, so their controls take the keys before the window shows them
		stepper.ensure();
		box?.ensure();
		keys(stepper, box);
		control = stepper.control;
		show();
		parts.push(h("p", {class: "qp-type-heading qp-pump-centre", text: question(count)}),
			box ? h("div", {class: "qp-pump-boxes"}, stepper, box) : stepper, kept, max);
		// a count corrected to 0 reads as typing 0 does, done without a count (slice 44's review), never Could not do one
		if (!shape.counted && set.value !== 0) {
			parts.push(h("qp-quiet-button", {label: limits.empty,
				"on:click": () => {
					stepper.set(0);
					value = 0;
					send(false);
				}}));
		}
		return parts.filter(Boolean);
	}

	/** The value's keys while it has the focus: the digits type (the first replaces the count saved), Backspace erases,
	 * "." or "," move to the weight box, Enter saves; the weight box takes its own. */
	function keys(stepper, box) {
		let fresh = true;
		let freshWeight = true;
		stepper.control.addEventListener("keydown", (e) => {
			if (/^[0-9]$/.test(e.key)) {
				if (fresh) stepper.set(Number(e.key));
				else stepper.type(Number(e.key));
				fresh = false;
			} else if (e.key === "Backspace") {
				stepper.erase();
				fresh = false;
			} else if (box && (e.key === "." || e.key === ",")) {
				box.control.focus({preventScroll: true});
			} else if (e.key === "Enter") {
				save?.();
			} else {
				return;
			}
			e.preventDefault();
		});
		box?.control.addEventListener("keydown", (e) => {
			const point = e.key === "." || e.key === ",";
			if (/^[0-9]$/.test(e.key) || point) {
				if (freshWeight) box.clear();
				if (point) box.point();
				else box.type(Number(e.key));
				freshWeight = false;
			} else if (e.key === "Backspace") {
				box.erase();
				freshWeight = false;
			} else if (e.key === "Enter") {
				save?.();
			} else {
				return;
			}
			e.preventDefault();
		});
	}

	/** Whether the value, the weight or the answer differs from the set's as the fold reads it (the game's
	 * correctDiffers, the iPhone's `changed`; no weight and 0 kg alike, as Save sends them). */
	function changed() {
		if (set.kind === "answer") return value !== set.value;
		return value !== set.value || (shapeOf(w, set).loaded && (load ?? 0) !== (set.load ?? 0));
	}

	/** Whether Save takes the input: one that differs from the set's (`changed`), and on a set another device's
	 * correction moved (`ownSide`) the set as it reads now too, which Save writes as this side's own. */
	function differs() {
		return Boolean(set.ownSide) || changed();
	}

	/** Save's state (and "Is that right?"'s Yes), the foot's line and the edit view's elsewhere line: Save busy while
	 * its answer comes, disabled until the value differs and while a pump runs, "A pump is on…" pinned under it then;
	 * the elsewhere line only while the value, the weight and the answer still hold the set as it reads now (the game's
	 * correctResave, the iPhone's `changed`: 2026-10-19, the double-check's fix review), never over a Save that writes
	 * another value. */
	function settle() {
		if (view !== "edit" && view !== "really") return;
		if (elsewhere) elsewhere.hidden = changed();
		const off = running() || (view === "edit" && !differs());
		win.primaryState = busy ? "busy" : off ? "disabled" : "normal";
		save = busy || off ? null : () => send(view === "really");
		win.foot.replaceChildren(...(running() ? [line(t("correct.window.pumpon"), "qp-type-meta qp-text2")] : []));
	}

	/** Sends the Save: the count with a loaded set's weight (0 for none), or the answer; `confirm` "Is that right?"'s
	 * Yes, the count it asked about. */
	function send(confirm) {
		if (busy || running()) return;
		busy = true;
		mine = ++saves;
		const key = {seq: mine, move: set.move, ts: set.ts, quest: set.quest, slot: set.slot, confirm};
		const change = set.kind === "answer" ? {answer: value}
			: {value, ...(shapeOf(w, set).loaded ? {load: load ?? 0} : {})};
		settle();
		intent("book.correct", {...key, ...change}).catch((error) => {
			// refused at once (a pump began, the engine folds): Save is ready again, the pump's line under it
			console.error(error);
			if (mine !== key.seq) return;
			busy = false;
			draw();
		});
	}

	/** What the correction's rows changed on the steps, a line each (Rust's `steps`, the engine's): a ladder back to or
	 * up to a step, then a step no longer or now Mastered. */
	function stepLines(steps) {
		const ladder = (id) => w.ladders.get(id)?.name ?? "";
		return [...(steps?.rungs ?? []).map((r) => t(r.to < r.from ? "correct.window.step.back" : "correct.window.step.up",
			{ladder: ladder(r.ladder), step: r.toStep})),
		...(steps?.mastered ?? []).map((m) => t(m.won ? "correct.window.mastered.won" : "correct.window.mastered.lost",
			{step: m.step, ladder: ladder(m.ladder)}))];
	}

	/** The window's content for its view, its buttons and the focus. */
	function draw() {
		// "Is that right?" stands only on the question it asks (a Save that wrote nothing keeps it; never one without)
		if (view === "really" && !outcome?.check) view = "edit";
		const parts = [];
		let primary = null;
		let secondary = null;
		let focus = null;
		control = null;
		elsewhere = null;
		win.foot.replaceChildren();
		if (view === "list") {
			parts.push(...list());
			focus = parts.find((p) => p.localName === "qp-card")?.querySelector("qp-row") ?? null;
			save = null;
		} else if (view === "edit") {
			parts.push(...editParts());
			if (outcome?.outcome === "max") parts.push(line(outcome.line, "qp-type-body"));
			primary = t("pump.count.save");
			secondary = t("correct.window.other");
			focus = control;
		} else if (view === "really") {
			const words = checkWords(outcome.check);
			parts.push(h("qp-card", {}, line(words.line, "qp-type-body")));
			primary = words.yes;
			secondary = outcome.change;
		} else {
			// the set as the fold reads it after the write (Rust's `now`), as the list says it next
			const said = set.kind !== "answer" && outcome.value === 0 ? t("correct.window.done.none")
				: t("correct.window.done", {value: valueWords(w, set, outcome)});
			parts.push(line(said, "qp-type-subtitle"));
			const steps = stepLines(outcome.steps);
			parts.push(...steps.map((text) => line(text, "qp-type-body")));
			const stays = outcome?.stays === "game" ? t("correct.window.step.game")
				: outcome?.stays === "later" ? t("correct.window.step.later") : null;
			if (stays) parts.push(line(stays));
			primary = t("common.done");
			save = () => win.close();
			announce([said, ...steps, stays].filter(Boolean).join(" "));
		}
		win.content.replaceChildren(...parts);
		win.primary = primary;
		win.secondary = secondary;
		win.primaryState = busy ? "busy" : "normal";
		settle();
		requestAnimationFrame(() => (focus ?? (primary ? win.primaryButton : win.closer)).focus?.({preventScroll: true}));
	}

	function edit(s) {
		set = s;
		value = s.value;
		load = s.load ?? null;
		view = "edit";
		outcome = null;
		gone = false;
		draw();
	}

	win.addEventListener("qp-primary", () => save?.());
	win.addEventListener("qp-secondary", () => {
		if (view === "edit") {
			view = "list";
			draw();
		} else if (view === "really") {
			view = "edit";
			outcome = null;
			draw();
		}
	});
	const stops = [
		subscribe("book", (b) => {
			book = b;
			if (view === "list") draw();
		}),
		subscribe("pump", (p) => {
			pump = p;
			settle();
		}),
		subscribe("book.correct", (o) => {
			if (!o || o.seq !== mine || !busy) return;
			busy = false;
			// nothing written (the engine did not answer, a pump began meanwhile): the view as it stood, "Is that
			// right?" with its question and its Yes ready again, or the value with its Save
			if (o.outcome === "failed") {
				draw();
				return;
			}
			outcome = o;
			if (o.outcome === "done") view = "done";
			else if (o.outcome === "asks") view = "really";
			else if (o.outcome === "gone") {
				view = "list";
				gone = true;
				announce(t("correct.window.gone"));
			} else if (o.outcome === "max") announce(o.line);
			draw();
		}),
	];
	win.addEventListener("qp-closed", () => {
		for (const stop of stops) stop();
		win.remove();
	});
	document.body.append(win);
	draw();
	win.open();
	return win;
}
