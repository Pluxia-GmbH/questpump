// The places' windows on the computer (docs/plan.md slice 51, "Places"; the phones' expedition.places, places.edit
// and places.place), from the `places` topic (helper/crates/questpump-desk/src/places.rs), each following it while it is open
// and leaving the page once closed:
// - the places window (desk.expedition.places; Pump home's place button opens it too): a Row for each place, its glyph
//   in a KindTile, its name, its equipment line, the Default tag on the default place (2026-11-03) and the check on the
//   place in use; a click (or Enter, the arrow keys moving between the rows) makes it the place in use until the
//   schedule's next start or end or the plan's next change and closes the window, another place than the one in use
//   with the `toggle` sound in the tap's place (slice 72); under them the Schedule row (the
//   one time there is, or how many); Add a place (the list's most named instead once it holds them) and Edit places;
// - Edit places (desk.places.edit): Home first (Your setup in the game, never moved or deleted), then each other place
//   with its drag handle, which drags it to another place in the list, Move up and Move down on Alt with the arrow keys
//   while its Row has the focus, and Delete with its confirm Window; a click on a Row opens its place's window; Done;
// - a place's window (desk.places.place): Start from for a new place (Basic first and chosen, then the templates, each
//   filling what follows), its name and its icon (never Home's, whose words and glyph are the game's), the Default
//   place switch (2026-11-03: on and inert at the default place; turned on elsewhere, Save makes that place the
//   default), where you train and the equipment chips in their groups with their Tooltips, Save (waiting for a name),
//   and Delete place with its confirm Window (never Home's, never a new place's); Home's says that it is the game's
//   setup;
// - the schedule (desk.places.schedule, 2026-11-03): a Row for each time, ordered by its first day (Monday first) and
//   its start, its place's glyph, its days and its span and place, opening its window; Add a time while the schedule
//   holds fewer than its most;
// - a time's window (desk.places.time): Days as seven round chips, Monday first, From and To in the system's time
//   fields (as Reminders' quiet hours), the place as checked Rows, Save held while no day is chosen or From equals To
//   (a To before From runs past midnight), and Delete this time for a kept one, at one click.
// Every change is an intent (`places.use`, `places.save`, `places.delete`, `places.order`, `places.time.save`,
// `places.time.delete`), and the topic then draws what holds.

import {get, intent, subscribe} from "../../lib/bridge.js";
import {t} from "../../lib/copy.js";
import {h} from "../../kit/element.js";
import {confirmWindow} from "../book/common.js";
import {field, minutes} from "../menu/reminders.js";
import {setupParts, toggle} from "../menu/setup.js";
import {WEEK, daysLine, gearLine, glyphName, placeName, templateName, timeLine, weekday} from "./words.js";

/** An intent's failure goes to the console (Rust's handler logged why). */
function heard(promise) {
	promise.catch(console.error);
}

/** A window following the `places` topic while it is open (`draw(places)` at once and at every change), leaving the
 * page once closed. */
function placesWindow(props, draw) {
	const win = h("qp-window", props);
	win.ensure();
	const stop = subscribe("places", (p) => {
		if (p) draw(p);
	});
	win.addEventListener("qp-closed", () => {
		stop();
		win.remove();
	}, {once: true});
	win.open();
	return win;
}

/** A place's Row: its glyph in a KindTile, its name, and its equipment line. */
function placeRow(place, props = {}) {
	return h("qp-row", {glyph: place.glyph, tile: true, heading: placeName(place), meta: gearLine(place), ...props});
}

/** The arrow keys move the focus between the rows of `card` (each a qp-row's button). */
function arrows(card) {
	card.addEventListener("keydown", (e) => {
		if (e.altKey || (e.key !== "ArrowDown" && e.key !== "ArrowUp")) return;
		const rows = [...card.querySelectorAll("qp-row .qp-row-body")];
		const at = rows.indexOf(document.activeElement);
		if (at < 0) return;
		e.preventDefault();
		rows[Math.max(0, Math.min(rows.length - 1, at + (e.key === "ArrowDown" ? 1 : -1)))].focus();
	});
}

/** Add a place, or the list's most where it holds them, as a part of a window's foot. */
function addPart(onAdd) {
	const add = h("qp-secondary-button", {label: t("places.add"), "on:click": onAdd});
	const full = h("p", {class: "qp-type-meta qp-places-full"});
	return {
		el: h("div", {class: "qp-places-add"}, add, full),
		draw(p) {
			const room = p.list.length < p.most;
			add.hidden = !room;
			full.hidden = room;
			full.textContent = t("places.full", {count: p.most});
		},
	};
}

/** A place of the topic's list by its id (Home where the list does not hold it). */
function placeOf(p, id) {
	return p.list.find((place) => place.id === id) ?? p.list[0];
}

/** The places window (desk.expedition.places): the place in use picked here. */
export function openPlaces() {
	const card = h("qp-card", {rows: true, class: "qp-places-list"});
	arrows(card);
	const schedule = h("qp-row", {glyph: "mark_clock", heading: t("places.schedule"), "on:click": () => openSchedule()});
	let shown = "";
	let win = null;
	const add = addPart(() => openPlace(null));
	win = placesWindow({heading: t("places.title")}, (p) => {
		add.draw(p);
		// the one time there is, or how many; none with no time
		const [only] = p.times;
		schedule.meta = p.times.length === 1 ? `${daysLine(only.days)} · ${timeLine(only, placeOf(p, only.place))}`
			: p.times.length ? t("places.schedule.times", {count: p.times.length}) : null;
		// made again only when what it lists changed, so the focus stays on its row
		const listed = JSON.stringify([p.list, p.in_use, p.default]);
		if (listed === shown) return;
		shown = listed;
		const focused = card.contains(document.activeElement);
		card.replaceChildren(...p.list.map((place) => placeRow(place, {trailing: "checked",
			value: place.id === p.default ? t("places.default.tag") : null,
			checked: place.id === p.in_use, "data-id": place.id,
			"data-sound": place.id === p.in_use ? null : "toggle", "on:click": () => {
				heard(intent("places.use", {id: place.id}));
				win.close();
			}})));
		if (focused) card.querySelector(`qp-row[data-id="${CSS.escape(p.in_use)}"] .qp-row-body`)?.focus();
	});
	win.content.append(card, h("qp-card", {rows: true}, schedule));
	win.foot.append(add.el, h("qp-quiet-button", {label: t("places.edit"), "on:click": () => openEdit()}));
	// the focus starts on the place in use
	requestAnimationFrame(() => card.querySelector("qp-row[checked] .qp-row-body")?.focus());
	return win;
}

/** Every place but Home in the order the rows of `card` show them. */
function order(card) {
	return [...card.querySelectorAll(".qp-place-edit[data-id]")].map((el) => el.dataset.id);
}

/** Drags a row by its handle (`grip`) through `card`: the row follows the pointer among the others, and the order
 * goes to Rust as it lands. */
function draggable(grip, item, card) {
	grip.addEventListener("pointerdown", (e) => {
		if (e.button !== 0) return;
		e.preventDefault();
		grip.setPointerCapture(e.pointerId);
		const before = order(card).join();
		item.classList.add("qp-dragging");
		const move = (ev) => {
			const others = [...card.querySelectorAll(".qp-place-edit[data-id]")].filter((el) => el !== item);
			const next = others.find((el) => {
				const box = el.getBoundingClientRect();
				return ev.clientY < box.top + box.height / 2;
			});
			if (next) {
				if (item.nextElementSibling !== next) card.insertBefore(item, next);
			} else if (card.lastElementChild !== item) {
				card.append(item);
			}
		};
		const end = () => {
			grip.removeEventListener("pointermove", move);
			item.classList.remove("qp-dragging");
			const now = order(card);
			if (now.join() !== before) heard(intent("places.order", {ids: now}));
		};
		grip.addEventListener("pointermove", move);
		grip.addEventListener("pointerup", end, {once: true});
		grip.addEventListener("pointercancel", end, {once: true});
	});
}

/** Asks before place `place` goes, then deletes it (`then` after). */
function confirmDelete(place, then = null) {
	const name = placeName(place);
	confirmWindow(t("places.delete.title", {place: name}), t("places.delete.body"), t("places.delete"), () => {
		heard(intent("places.delete", {id: place.id}));
		then?.();
	}, t("places.delete.keep"));
}

/** Edit places (desk.places.edit). */
export function openEdit() {
	const card = h("qp-card", {rows: true, class: "qp-places-list"});
	arrows(card);
	const add = addPart(() => openPlace(null));
	let shown = "";
	const win = placesWindow({heading: t("places.edit"), primary: t("common.done")}, (p) => {
		add.draw(p);
		const listed = JSON.stringify(p.list);
		if (listed === shown) return;
		shown = listed;
		const focused = document.activeElement?.closest?.(".qp-place-edit")?.dataset.key;
		card.replaceChildren(...p.list.map((place) => {
			if (place.home) {
				return h("div", {class: "qp-place-edit", "data-key": place.id},
					placeRow(place, {meta: t("places.home.meta"), "on:click": () => openPlace(place)}));
			}
			const name = placeName(place);
			const row = placeRow(place, {"on:click": () => openPlace(place)});
			const grip = h("span", {class: "qp-place-grip", title: t("places.drag.hint"), "aria-hidden": "true"},
				h("i"), h("i"), h("i"));
			const remove = h("button", {type: "button", class: "qp-place-delete", text: t("places.delete"),
				"aria-label": t("places.delete.spoken", {place: name}), "on:click": () => confirmDelete(place)});
			const item = h("div", {class: "qp-place-edit", "data-id": place.id, "data-key": place.id}, grip, row,
				remove);
			draggable(grip, item, card);
			return item;
		}));
		for (const control of card.querySelectorAll(".qp-place-edit[data-id] .qp-row-body")) {
			// copy-lint: not shown (ARIA's key names, which the screen reader words itself)
			control.setAttribute("aria-keyshortcuts", "Alt+ArrowUp Alt+ArrowDown");
			control.setAttribute("aria-description", `${t("places.move.up")}, ${t("places.move.down")}`);
		}
		if (focused) card.querySelector(`.qp-place-edit[data-key="${CSS.escape(focused)}"] .qp-row-body`)?.focus();
	});
	// Move up and Move down: Alt with the arrow keys on a row's focus
	card.addEventListener("keydown", (e) => {
		if (!e.altKey || (e.key !== "ArrowUp" && e.key !== "ArrowDown")) return;
		const item = document.activeElement?.closest?.(".qp-place-edit[data-id]");
		if (!item) return;
		e.preventDefault();
		const ids = order(card);
		const at = ids.indexOf(item.dataset.id);
		const to = at + (e.key === "ArrowUp" ? -1 : 1);
		if (to < 0 || to >= ids.length) return;
		[ids[at], ids[to]] = [ids[to], ids[at]];
		heard(intent("places.order", {ids}));
	});
	win.content.append(h("p", {class: "qp-type-caps", text: t("places.drag.hint")}), card, add.el);
	win.addEventListener("qp-primary", () => win.close());
	return win;
}

/** The setup a template or Empty starts a new place with. */
function startOf(p, template) {
	const from = template ? p.templates.find((x) => x.template === template || x.id === template) : null;
	const base = from ?? p.empty ?? {glyph: p.glyphs[0], gear: [], floor: true, space: true, quiet: false};
	return {glyph: base.glyph, gear: [...base.gear], floor: base.floor, space: base.space, quiet: base.quiet};
}

/** A place's window (desk.places.place): `place` one of the topic's list, or null for a new place. */
export function openPlace(place) {
	const p = get("places");
	if (!p) return null;
	const isNew = !place;
	const home = Boolean(place?.home);
	// what Save sends: the place as it stands, changed here
	let template = place?.template ?? null;
	const now = isNew ? startOf(p, null) : {glyph: place.glyph, gear: [...place.gear], floor: place.floor,
		space: place.space, quiet: place.quiet};
	const win = h("qp-window", {heading: isNew ? t("places.place.new") : placeName(place), primary: t("pump.count.save")});
	win.ensure();
	// the chips' Tooltip inside the window, in the top layer over it
	const tooltip = h("qp-tooltip", {popover: "manual"});
	win.dialog.append(tooltip);
	const parts = [];

	// Start from: Basic (the floor and room to move, no equipment), chosen at first, then the templates, for a new place
	const from = [{key: null, label: t("places.place.empty")},
		...p.templates.map((x) => ({key: x.template ?? x.id, label: templateName(x.template ?? x.id)}))];
	const fromChips = from.map((f) => {
		const chip = h("qp-choice-chip", {label: f.label, selected: f.key === null});
		chip.addEventListener("click", () => {
			const before = template ? templateName(template) : "";
			template = f.key;
			Object.assign(now, startOf(p, f.key));
			// the name follows the template while the player has not typed one of their own
			if (!name.value.trim() || name.value.trim() === before) name.value = f.key ? templateName(f.key) : "";
			for (const other of fromChips) other.selected = other === chip;
			draw();
		});
		return chip;
	});
	if (isNew) {
		parts.push(h("qp-section-label", {label: t("places.place.from")}),
			h("div", {class: "qp-chips"}, ...fromChips));
	}

	// the name and the icon, never Home's
	const name = h("qp-text-input", {label: t("places.place.name"), max: p.name_most});
	const nameNote = h("p", {class: "qp-type-meta", text: t("places.place.name.empty")});
	const icons = p.glyphs.map((glyph) => h("button", {type: "button", class: "qp-place-icon",
		"aria-label": glyphName(glyph), "on:click": () => {
			now.glyph = glyph;
			draw();
		}}, h("qp-art", {name: glyph})));
	if (!home) {
		name.ensure();
		name.value = isNew ? "" : placeName(place);
		name.addEventListener("input", () => draw());
		name.addEventListener("qp-submit", () => save());
		parts.push(h("qp-section-label", {label: t("places.place.name")}), name, nameNote,
			h("qp-section-label", {label: t("places.place.icon")}),
			h("div", {class: "qp-place-icons", role: "group", "aria-label": t("places.place.icon")}, ...icons));
	} else {
		parts.push(h("p", {class: "qp-type-meta qp-words-block", text: t("places.home.note")}));
	}

	// the Default place switch: on and inert at the default place, Save making another the default once turned on
	const isDefault = !isNew && place.id === p.default;
	now.default = isDefault;
	const defaultRow = toggle(t("places.place.default"), "star", (on) => {
		now.default = on;
		draw();
		return Promise.resolve();
	});
	defaultRow.disabled = isDefault;
	parts.push(h("qp-card", {rows: true}, defaultRow));

	// where you train and the equipment
	const setup = setupParts(p.words, tooltip, (key, on) => {
		now[key] = on;
		draw();
		return Promise.resolve();
	}, (key, on) => {
		now.gear = on ? [...now.gear, key] : now.gear.filter((k) => k !== key);
		draw();
	});
	parts.push(...setup.nodes);
	win.content.append(...parts);

	// Delete place, never Home's or a new place's
	if (!isNew && !home) {
		win.foot.append(h("qp-quiet-button", {label: t("places.place.delete"), class: "qp-destructive",
			"on:click": () => confirmDelete(place, () => win.close())}));
	}

	/** The name Save sends: none while it is the template's own words (shown in the player's language). */
	const named = () => {
		const text = name.value.trim();
		return text && !(template && text === templateName(template)) ? text : null;
	};
	const ready = () => home || Boolean(name.value.trim());
	function draw() {
		setup.draw(now);
		defaultRow.checked = now.default;
		for (const [i, icon] of icons.entries()) icon.setAttribute("aria-pressed", String(p.glyphs[i] === now.glyph));
		nameNote.hidden = home || ready();
		win.primaryState = ready() ? "normal" : "disabled";
	}
	function save() {
		if (!ready()) return;
		const gearOrder = p.words.gear.map((g) => g.key);
		const gear = gearOrder.filter((k) => now.gear.includes(k));
		const args = {gear, floor: now.floor, space: now.space, quiet: now.quiet, glyph: now.glyph};
		if (!isNew) args.id = place.id;
		if (now.default && !isDefault) args.default = true;
		if (!home) {
			if (template) args.template = template;
			const own = named();
			if (own) args.name = own;
		}
		heard(intent("places.save", args));
		win.close();
	}
	win.addEventListener("qp-primary", save);
	win.addEventListener("qp-closed", () => win.remove(), {once: true});
	win.addEventListener("scroll", () => tooltip.hide(), true);
	// Escape puts the Tooltip away first, the window staying
	win.addEventListener("keydown", (e) => {
		if (tooltip.escape(e)) e.preventDefault();
	}, true);
	draw();
	win.open();
	return win;
}

/** The minutes of a day. */
const DAY_MIN = 24 * 60;

/** The schedule's times in its order (by the first day, Monday first, then the start), each with its place in the
 * topic's list (`at`, which Save and Delete name). */
function ordered(times) {
	const first = (time) => Math.min(...time.days);
	return times.map((time, at) => ({time, at}))
		.sort((a, b) => first(a.time) - first(b.time) || a.time.from - b.time.from || a.at - b.at);
}

/** The schedule (desk.places.schedule): a Row per time, Add a time while it holds fewer than its most. */
export function openSchedule() {
	const card = h("qp-card", {rows: true, class: "qp-places-list"});
	arrows(card);
	const add = h("qp-quiet-button", {label: t("places.schedule.add"), "on:click": () => openTime(null)});
	let shown = "";
	const win = placesWindow({heading: t("places.schedule")}, (p) => {
		add.hidden = p.times.length >= p.times_most;
		const listed = JSON.stringify([p.times, p.list]);
		if (listed === shown) return;
		shown = listed;
		card.hidden = !p.times.length;
		card.replaceChildren(...ordered(p.times).map(({time, at}) => {
			const place = placeOf(p, time.place);
			return h("qp-row", {glyph: place.glyph, tile: true, heading: daysLine(time.days),
				meta: timeLine(time, place), "on:click": () => openTime({time, at})});
		}));
	});
	win.content.append(card);
	win.foot.append(add);
	return win;
}

/** A time's window (desk.places.time): `kept` a time of the schedule with its place in it ({time, at}), or null for a
 * new one (no day yet, from this hour to the next, at the place in use). */
export function openTime(kept) {
	const p = get("places");
	if (!p) return null;
	const hour = new Date().getHours() * 60;
	const now = kept ? {...kept.time, days: [...kept.time.days]}
		: {days: [], from: hour, to: (hour + 60) % DAY_MIN, place: p.in_use};
	const win = h("qp-window", {heading: t("places.schedule"), primary: t("pump.count.save")});
	win.ensure();
	const caps = (key) => h("p", {class: "qp-type-caps", text: t(key)});
	// the days: seven round chips, Monday first, each its narrow letter, read by its full name
	const days = WEEK.map((day) => h("button", {type: "button", class: "qp-day-chip", "aria-label": weekday(day, "long"),
		text: weekday(day, "narrow"), "on:click": () => {
			now.days = now.days.includes(day) ? now.days.filter((d) => d !== day) : [...now.days, day];
			draw();
		}}));
	const from = h("qp-text-input", {mode: "time", label: t("places.time.from")});
	const to = h("qp-text-input", {mode: "time", label: t("places.time.to")});
	for (const input of [from, to]) {
		input.ensure();
		input.addEventListener("input", () => draw());
	}
	from.value = field(now.from);
	to.value = field(now.to);
	const places = h("qp-card", {rows: true, class: "qp-places-list"}, ...p.list.map((place) => placeRow(place, {
		trailing: "checked", meta: null, "data-id": place.id, "on:click": () => {
			now.place = place.id;
			draw();
		}})));
	arrows(places);
	win.content.append(caps("places.time.days"),
		h("div", {class: "qp-day-chips", role: "group", "aria-label": t("places.time.days")}, ...days),
		caps("places.time.from"), from, caps("places.time.to"), to, caps("places.time.place"), places);
	if (kept) {
		win.foot.append(h("qp-quiet-button", {label: t("places.time.delete"), class: "qp-destructive", "on:click": () => {
			heard(intent("places.time.delete", {at: kept.at}));
			win.close();
		}}));
	}
	/** The span the fields hold, or null while either holds no time. */
	const span = () => {
		const [start, end] = [minutes(from.value), minutes(to.value)];
		return start === null || end === null ? null : {from: start, to: end};
	};
	const ready = () => {
		const s = span();
		return now.days.length > 0 && s !== null && s.from !== s.to;
	};
	function draw() {
		for (const [i, chip] of days.entries()) chip.setAttribute("aria-pressed", String(now.days.includes(WEEK[i])));
		for (const row of places.children) row.checked = row.dataset.id === now.place;
		win.primaryState = ready() ? "normal" : "disabled";
	}
	win.addEventListener("qp-primary", () => {
		if (!ready()) return;
		const args = {days: WEEK.filter((d) => now.days.includes(d)), ...span(), place: now.place};
		if (kept) args.at = kept.at;
		heard(intent("places.time.save", args));
		win.close();
	});
	win.addEventListener("qp-closed", () => win.remove(), {once: true});
	draw();
	win.open();
	return win;
}
