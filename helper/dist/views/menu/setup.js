// Your setup and Your moves on the computer (docs/ui.md, "The Menu on the computer"; the phones' Your setup and Your
// moves, slice 34), in the addon's words (contract/sheet.json, through the topic): the card that a pump is on screen;
// the Focus row (the focus in force, opening the focus window); the place in use (slice 51: its Row opens the places
// window, views/places/places.js; while work mode runs Work mode with its equipment, opening the Work mode page, and
// what follows is work mode's own equipment, slice 67) and where the player trains there (floor moves, room to move,
// quiet) and its equipment as chips in the game's order, in its five groups (2026-11-03: each a head that opens and
// closes its chips, with how many of its pieces are chosen while one is, a group with a chosen piece open at first),
// with what the next pieces would open under them, each chip's Tooltip on hover and on keyboard focus (its name, what
// it is, and while it is not chosen what it would open now, as the game's equipment tooltip; David, 2026-10-13: "Add a
// hover explanation to the equpiments to give more details"); the Body switches and Your moves, the same on the
// player's phones through the body's exchange (over them a card while they started empty because their file did not
// read and no device filled them since); the pain flags, a switch per joint, a joint the game flagged named so; the
// experience question as a Segmented, fixed once a ladder holds a rung (and then saying so); Moves per match as the
// Games tab shows it (slice 48: the body's match_pace, `games.pace`, from the `games` topic). Your moves lists the
// groups and the moves the player left out of pumps, each with its switch (what it does read after its name), a row
// switched staying on the page until the player leaves it, so a slip is undone in place, and at its foot the Book row,
// which opens the Book's Moves. Every switch and the Segmented move when the topic says the change holds, never before.
// From the `menu.setup` topic (helper/crates/questpump-desk/src/menu_pages/setup.rs); the engine's part (the pain
// flags, the focus, the start rungs, the equipment lines, what each piece would open) comes once it answers, and Rust
// asks it again at every fold, merge of rows and change of profile.

import {get, intent} from "../../lib/bridge.js";
import {t} from "../../lib/copy.js";
import {covered, find, push, register} from "../../lib/router.js";
import {h} from "../../kit/element.js";
import {focusGlyph, focusWindow} from "../hero/focus.js";
import {page, watch, words} from "../page.js";
import {openPlaces} from "../places/places.js";
import {gearLine, placeName, placeShown} from "../places/words.js";
import {paceRows} from "../games/games.js";

/** Each switch's glyph, as the phones' Your setup leads its rows. */
export const GLYPHS = {
	floor: "mark_dash", space: "rating_expertise", quiet: "reward_familiarity", no_jump: "stat_agility",
	no_strain: "stat_stamina", no_head_down: "action_character",
};
/** Where the player trains: a place's three switches. */
export const PLACE = ["floor", "space", "quiet"];
const BODY = ["no_jump", "no_strain", "no_head_down"];

/** What a piece would open (`{count, quality}`: a step of `look.steps` or one of `look.opens`) in `key`'s words with
 * `args` and the best quality among the moves, or in `plain`'s without it where none of them has a quality. */
function opensWords(open, vocab, key, plain, args = {}) {
	const quality = open.quality ? vocab.qualities[open.quality - 1] : null;
	return quality ? t(key, {...args, count: open.count, quality}) : t(plain, {...args, count: open.count});
}

/** An equipment line in the player's words (the phones' `FreeSettingsView.opens`): the piece, how many more moves it
 * would open and the best quality among them, without one where none of them has a quality. */
function opensLine(step, vocab) {
	const gear = vocab.gear.find((g) => g.key === step.key)?.name ?? step.key;
	return opensWords(step, vocab, "menu.free.opens", "menu.free.opens.plain", {gear});
}

/** What a piece of equipment not chosen would open now on its own (`look.opens`, the game's ns.SetupOpens) as its
 * Tooltip's last line says it, with the best quality among the moves where one of them has one; none while it is
 * chosen, before the engine's look, or where it opens nothing. */
function gearOpens(key, s, vocab) {
	const open = !s.place.gear.includes(key) && s.look?.opens.find((o) => o.key === key);
	return open ? opensWords(open, vocab, "menu.free.gear.opens", "menu.free.gear.opens.plain") : null;
}

/** The equipment groups' words and glyphs, by the sheet's group key: literal keys, so the copy lint sees each read. */
const GROUPS = {
	home: {words: () => t("places.gear.group.home"), glyph: "place_home"},
	bars: {words: () => t("places.gear.group.bars"), glyph: "place_bars"},
	weights: {words: () => t("places.gear.group.weights"), glyph: "place_dumbbell"},
	cardio: {words: () => t("places.gear.group.cardio"), glyph: "stat_stamina"},
	core: {words: () => t("places.gear.group.core"), glyph: "stat_strength"},
};

/** The equipment in its groups (2026-11-03; the sheet's words' `gear_groups`, Library.lua `ns.GearGroups`), Your
 * setup's and a place's window's: each group a head Row (its name, how many of its pieces are chosen while one is, a
 * chevron) that opens and closes its pieces' chips (`chip(g)` makes one) under it. `draw()` counts the chips chosen
 * now and, at its first call, opens each group with a chosen piece; `reveal(chip)` opens the group that holds it. */
export function gearGroups(w, chip) {
	const byKey = new Map(w.gear.map((g) => [g.key, g]));
	const groups = w.gear_groups.map((group) => {
		const chips = group.gear.map((key) => byKey.get(key)).filter(Boolean).map(chip);
		const box = h("div", {class: "qp-chips"}, ...chips);
		const kind = GROUPS[group.key];
		const head = h("qp-row", {heading: kind?.words() ?? "", glyph: kind?.glyph, trailing: "expander"});
		const open = (on) => {
			head.expanded = on;
			box.hidden = !on;
		};
		head.addEventListener("click", () => open(!head.expanded));
		open(false);
		return {el: h("div", {class: "qp-gear-group"}, h("qp-card", {rows: true}, head), box), head, chips, open};
	});
	let drawn = false;
	return {
		el: h("div", {class: "qp-gear-groups"}, ...groups.map((g) => g.el)),
		chips: groups.flatMap((g) => g.chips),
		draw() {
			for (const g of groups) {
				const chosen = g.chips.filter((c) => c.selected).length;
				g.head.value = chosen ? t("places.gear.group.count", {chosen, total: g.chips.length}) : null;
				if (!drawn && chosen) g.open(true);
			}
			drawn = true;
		},
		reveal(target) {
			groups.find((g) => g.chips.includes(target))?.open(true);
		},
	};
}

/** A switch's Row: a press asks for the value it would take (`ask(on)`), the topic then says what holds. */
export function toggle(heading, glyph, ask) {
	const row = h("qp-row", {heading, glyph, trailing: "toggle"});
	row.addEventListener("qp-change", (e) => ask(e.detail).catch(console.error));
	return row;
}

/** Where you train's three switches (`PLACE`) as Rows in a card, from the sheet's `words`, as a place's window and work
 * mode's pages show them: a press asks for the value it would take (`ask(key, on)`, a promise); `draw(setup)` checks
 * each as `setup` holds it. */
export function placeSwitches(words, ask) {
	const card = h("qp-card", {rows: true}, ...PLACE.flatMap((key) => {
		const word = words.switches[key];
		if (!word) return [];
		const row = toggle(word.name, GLYPHS[key], (on) => ask(key, on));
		row.dataset.key = key;
		return [row];
	}));
	return {
		card,
		draw(setup) {
			for (const row of card.children) row.checked = setup[row.dataset.key];
		},
	};
}

/** Where you train and Equipment as a place's window and the Work mode page draw a setup (`nodes`: each section label
 * over its part): the three switches (`placeSwitches`, each asking `ask(key, on)`) and the equipment chips in their
 * groups (`gearGroups`, each asking `press(key, on)`, its Tooltip `tooltip`); `draw(setup)` shows `setup` ({gear,
 * floor, space, quiet}). */
export function setupParts(words, tooltip, ask, press) {
	const switches = placeSwitches(words, ask);
	const groups = gearGroups(words, (g) => {
		const chip = gearChip(g, tooltip, () => press(g.key, !chip.selected));
		return chip;
	});
	return {
		nodes: [h("qp-section-label", {label: t("menu.free.place")}), switches.card,
			h("qp-section-label", {label: t("menu.free.equipment")}), groups.el],
		draw(setup) {
			switches.draw(setup);
			for (const chip of groups.chips) chip.selected = setup.gear.includes(chip.dataset.key);
			groups.draw();
		},
	};
}

/** A page's chip Tooltip (Your setup's, the Work mode page's), in the document's body while the page stands: a scroll,
 * another root or the Menu put it away, and so does Escape while it shows. */
export function pageTooltip(view) {
	const tooltip = h("qp-tooltip");
	document.body.append(tooltip);
	const untip = () => tooltip.hide();
	view.el.addEventListener("scroll", untip, true);
	for (const type of ["qp-tab", "qp-menu"]) {
		document.addEventListener(type, untip, true);
		view.stops.push(() => document.removeEventListener(type, untip, true));
	}
	view.onKey = (e) => tooltip.escape(e);
	view.stops.push(() => tooltip.remove());
	return tooltip;
}

/** An equipment chip (Your setup's, a place's window's) of piece `g` from the sheet's words: a click puts its Tooltip
 * away and does `press()`; its Tooltip (`tooltip`) shows on hover and keyboard focus with its name, what it is and
 * `effect(key)`'s last line (what it would open, in Your setup), and the pointer may cross onto it (WCAG 1.4.13).
 * `chip.tip()` draws it again where it shows. */
export function gearChip(g, tooltip, press, effect = () => null) {
	const chip = h("qp-choice-chip", {label: g.name, "data-key": g.key, "data-gap": g.key});
	chip.tip = () => {
		Object.assign(tooltip, {name: g.name, meta: g.tip ?? "", effect: effect(g.key)});
		tooltip.showAt(chip);
	};
	chip.addEventListener("click", () => {
		tooltip.hide();
		press();
	});
	for (const kind of ["pointerenter", "focusin"]) chip.addEventListener(kind, () => chip.tip());
	// the pointer may cross onto the Tooltip; the focus leaving takes it at once
	chip.addEventListener("pointerleave", () => tooltip.leave());
	chip.addEventListener("focusout", () => tooltip.hide());
	return chip;
}

register("desk.menu.setup", ({focus: wanted = null}) => {
	const view = page({heading: t("menu.setup")});
	const pumpOn = h("qp-card", {hidden: true}, words(t("menu.setup.pumpon"), "body"));
	const focus = h("qp-row", {heading: t("menu.free.focus"), trailing: "value"});
	const focusCard = h("qp-card", {rows: true, hidden: true}, focus);
	// the place in use, opening the places window (Work mode while it runs, opening its page)
	const placeRow = h("qp-row", {tile: true,
		"on:click": () => (placeShown(get("places"))?.work ? push("desk.menu.work") : openPlaces())});
	const place = h("qp-card", {rows: true});
	const gear = h("div");
	let groups = null;
	const opens = h("div", {class: "qp-part"});
	const body = h("qp-card", {rows: true});
	const bodyReset = h("qp-card", {hidden: true}, words(t("desk.menu.setup.body.reset"), "body"));
	const moves = h("qp-row", {heading: t("menu.moves"), glyph: "star", trailing: "value",
		"on:click": () => push("desk.menu.moves")});
	const painPart = h("div", {class: "qp-part", hidden: true});
	const pain = h("qp-card", {rows: true});
	const experienceLabel = h("qp-section-label");
	const experience = h("qp-segmented");
	const experienceNote = words("", "meta");
	const pace = paceRows();
	watch(view, "games", pace.draw);
	painPart.append(h("qp-section-label", {label: t("menu.free.pain")}), pain);
	let vocab = null;
	let chooser = null;
	let current = null;
	focus.addEventListener("click", () => chooser?.open(vocab.focus, current?.look?.focus ?? 1));
	experience.addEventListener("qp-change", (e) => intent("setup.experience", {answer: e.detail})
		.catch(console.error));
	view.body.append(pumpOn, focusCard,
		h("qp-section-label", {label: t("places.row")}), h("qp-card", {rows: true}, placeRow),
		h("qp-section-label", {label: t("menu.free.place")}), place,
		h("qp-section-label", {label: t("menu.free.equipment")}), gear, opens,
		h("qp-section-label", {label: t("menu.free.body")}), bodyReset, body, h("qp-card", {rows: true}, moves),
		painPart, experienceLabel, experience, experienceNote, ...pace.nodes);
	const tooltip = pageTooltip(view);

	/** The rows the sheet's words make, once: the place's and the body's switches, the chips, the joints. */
	function build(w) {
		vocab = w;
		const switchRow = (key) => {
			const word = w.switches[key];
			if (!word) return null;
			const ask = BODY.includes(key) ? (on) => intent("setup.switch", {key, on})
				: (on) => intent("setup.place", {key, on});
			const row = toggle(word.name, GLYPHS[key], ask);
			// its profile key, and the key the game's rules name it by (a refused Start's gap)
			Object.assign(row.dataset, {key, gap: word.gap});
			return row;
		};
		place.replaceChildren(...PLACE.map(switchRow).filter(Boolean));
		body.replaceChildren(...BODY.map(switchRow).filter(Boolean));
		groups = gearGroups(w, (g) => {
			const chip = gearChip(g, tooltip, () => intent("setup.gear", {key: g.key, on: !chip.selected})
				.catch(console.error), (key) => gearOpens(key, current, vocab));
			return chip;
		});
		gear.replaceChildren(groups.el);
		pain.replaceChildren(...w.joints.map((j) => {
			const row = toggle(j.name, "mark_report", (on) => intent("setup.pain", {joint: j.id, on}));
			row.dataset.joint = String(j.id);
			return row;
		}));
		experienceLabel.label = experience.label = w.experience.title;
		experience.options = w.experience.answers.map((a) => ({value: a.key, label: a.name}));
		chooser = focusWindow((id) => intent("setup.focus", {focus: id}).catch(console.error));
	}

	watch(view, "menu.setup", (s) => {
		if (!vocab) build(s.words);
		current = s;
		pumpOn.hidden = !s.pump_on;
		bodyReset.hidden = !s.body_reset;
		for (const row of place.children) row.checked = s.place[row.dataset.key];
		for (const row of body.children) row.checked = s.switches[row.dataset.key];
		for (const chip of groups.chips) {
			const g = vocab.gear.find((piece) => piece.key === chip.dataset.key);
			chip.selected = s.place.gear.includes(g.key);
			// what its Tooltip says, for a screen reader after the chip's name
			chip.hint = [g.tip, gearOpens(g.key, s, vocab)].filter(Boolean).join(" ") || null;
		}
		groups.draw();
		tooltip.shownAt?.tip?.();
		moves.value = s.moves.count ? t("menu.moves.count", {count: s.moves.count}) : t("menu.moves.none");
		experience.value = s.experience;
		const look = s.look;
		focusCard.hidden = painPart.hidden = !look;
		experience.disabled = Boolean(look?.started);
		experienceNote.textContent = t("menu.free.experience.locked");
		experienceNote.hidden = !look?.started;
		if (look) {
			const chosen = vocab.focus.find((f) => f.id === (look.focus ?? 1));
			focus.value = chosen?.name ?? null;
			focus.glyph = focusGlyph(chosen?.key);
			for (const row of pain.children) {
				const joint = vocab.joints.find((j) => j.id === Number(row.dataset.joint));
				row.checked = look.pain.includes(joint.id);
				// a flag the game's own rows set says so (the phones' PainSection)
				row.heading = row.checked && s.in_game.includes(joint.id)
					? t("menu.free.pain.ingame", {joint: joint.name}) : joint.name;
			}
		}
		opens.replaceChildren(...(look?.steps ?? []).map((step) => words(opensLine(step, vocab), "meta")));
	});

	watch(view, "places", (p) => {
		const shown = placeShown(p);
		placeRow.heading = placeName(shown);
		placeRow.meta = shown?.work ? gearLine(shown) : t("expedition.place.hint");
		placeRow.glyph = shown?.glyph ?? null;
	});

	// the engine's part now (Rust asks it again whenever the engine's state moves)
	intent("setup.look").catch(console.error);
	view.stops.push(() => chooser?.remove());
	// opened from a refused Start (slice 38): at the switch or the piece its gap names (by the key the game's rules
	// name it by, the words' `gap`), or at the pain switches, once
	let at = wanted;
	const named = () => {
		const key = at;
		at = null;
		if (key === "pain") return painPart.hidden ? null : pain.firstElementChild;
		const target = key ? [...place.children, ...body.children, ...(groups?.chips ?? [])]
			.find((el) => el.dataset.gap === key) : null;
		// a piece's chip shows in its group, opened for it
		if (target) groups?.reveal(target);
		return target;
	};
	view.primary = () => {
		const target = named();
		target?.scrollIntoView({block: "center"});
		return target ?? (current?.look ? focus : place.firstElementChild);
	};
	return view;
});

register("desk.menu.moves", () => {
	const view = page({heading: t("menu.moves")});
	const list = h("qp-card", {rows: true});
	const none = h("qp-card", {}, words(t("menu.moves.out.none"), "body"));
	// the rows the page shows: those it opened with and every one added while it stands, a group's by its key
	const groups = new Map();
	const moves = new Map();
	// the Book's row while it can open (none over a pump, whose cover holds the tabs: its Settings opens this page)
	const book = covered() ? null
		: h("qp-card", {rows: true},
			h("qp-row", {heading: t("bar.book"), glyph: "action_book", trailing: "nav", "on:click": find}));
	view.body.append(...[h("qp-section-label", {label: t("menu.moves.out")}), list, none, book].filter(Boolean));
	watch(view, "menu.setup", (s) => {
		for (const g of s.moves.groups) {
			if (groups.has(g.key)) continue;
			const row = toggle(g.name, "action_book", (on) => intent("setup.group", {group: g.key, on}));
			row.hint = t("menu.moves.group");
			groups.set(g.key, row);
		}
		for (const m of s.moves.out) {
			if (moves.has(m.move)) continue;
			const row = toggle(m.name, "action_pump", (on) => intent("setup.include", {move: m.move, on}));
			row.hint = t("ladders.move.include");
			moves.set(m.move, row);
		}
		const out = new Set(s.moves.groups.map((g) => g.key));
		const left = new Set(s.moves.out.map((m) => m.move));
		for (const [key, row] of groups) row.checked = !out.has(key);
		for (const [move, row] of moves) row.checked = !left.has(move);
		const byName = (a, b) => a.heading.localeCompare(b.heading);
		const rows = [...[...groups.values()].sort(byName), ...[...moves.values()].sort(byName)];
		list.replaceChildren(...rows);
		list.hidden = !rows.length;
		none.hidden = rows.length > 0;
	});
	view.primary = () => list.firstElementChild;
	return view;
});
