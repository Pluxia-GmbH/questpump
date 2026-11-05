// Pump home on the computer (docs/ui.md, "Pump home on the computer"; slice 32's Pump home, iOS PumpTab, Android's
// twin), from the `home` topic (helper/src-tauri/src/desk/home.rs): under the TopBar this computer's status Chip only
// while something needs fixing (the topic's `fix`: This computer's state words as the tray says them, the
// `menu.computer` topic's `state_line`, beside the tray icon's colour, `tone`; a save that failed; the firewall between
// this PC and the paired phone), opening This computer or Phones; the recent games (2026-10-27, `recent`,
// desk/recent.rs: since 2026-10-28 a Carousel of GameStat tiles, every game running now first, then every game with a
// move this week or played in the last week, the most moves first, in Rust's order, under Moves this week, or under
// Moves all time every game with a move by its moves in all, as the label's This week | All time says (`home.span`,
// kept on this computer); World of Warcraft's in its own words with the Chip's step as its dot, a click opening This
// computer for it and the game's window on the Games tab for the others; nothing before the first game played); the
// standing figure taking the room the blocks leave, the streak (from the least of `streak_milestones`, a 7-day streak,
// with its round Share, which opens the Share window on the streak card as it stands; slice 45; the whole board opens
// the streak page since slice 51), under Up next (2026-11-03) the embers row (slice 55: Cardio moves today 2 of 3 with
// the cardio step's move's tile and three marks, the phones' pump.tab's, starting that move at once, the step's card
// where Rust refuses the start; until 2026-11-03 it opened the card) and the step-up closest to done of all the ladders
// (2026-11-02: the `book` topic's `closest`, desk/book.rs, its key line over the ring's share, book.js `nextRow`,
// opening the ladder's page in the Book at that step; none while nothing climbs), none while pumps rest, then one card (the rest
// after feeling unwell, the banked moves (2026-11-03: their first three with their tiles, the whole card opening the
// window of every one with its Skip), or today's chapter, the Saga's own day, laid out as the
// phones' (board A-Home): Today's chapter with the chevron, its moves with the chapter's tag and its word over Moves
// today beside them, its strip at full height, its records (none at 0), the whole card opening the Saga; No moves yet
// today with the strip's empty axis before the day's first move, the busy pips before the Saga's first build), the
// day-2 check-in, the one card Rust chose that offers the next part (pairing a phone, the pairing code in a window, while the phone apps are in the stores; the
// addon while the game is here without it, Add QuestPump to the
// game, under the addon's line from the `menu.computer` topic's `addon`), and at the foot the one gold primary Rust chose
// with the quiet ways beside it: a setup step the player started, Catch up (while a move waits that waited at most
// owed_fresh_days game days; older ones alone leave Catch up a quiet way), the match's offer, or Quick pump (no length
// shown: since 2026-11-03 Rust starts the timed pump at the Menu's Quick pump length; the length row of slice 51 and
// its At Home went); a match's offer under another primary keeps its words above the foot and its Start as a quiet
// button. None while pumps rest; while a World of Warcraft client with QuestPump runs the line that the game asks the
// pumps (while QuestPump has not shown in it yet the pumps start here meanwhile, without Catch up); when the engine
// failed the line that pumps could not start, beside the Menu's report for help; while the engine folds the primary
// alone, busy. The TopBar's round place button (2026-11-03, Rapha: the place "via an icon ... top right") shows the
// place in use's own glyph from the `places` topic, which Rust changes at a start or an end of a time of the places'
// schedule, and opens the places window. The pumps are DESK-PUMP's intents: `pump.quick`, `pump.catchup` and
// `pump.start {kind, offer}` (an offer's Start: Ready at the offer's size since 2026-10-28, the Start a pump window
// preset to its kind until then), this window's own click, never the card's `outside`; a banked move's Skip is
// `home.skip`, the cardio row's start `pump.move` from `home`.

/** What the recent games count in each span (`home.span`): the label over them and what a screen reader hears after a
 * game's name, and the count a game's tile shows. */
const SPANS = {
	week: {label: "desk.home.games", count: "desk.home.games.count", of: (game) => game.moves},
	all: {label: "desk.home.games.all", count: "desk.home.games.count.all", of: (game) => game.all},
};

import {call, intent} from "../../lib/bridge.js";
import {t} from "../../lib/copy.js";
import {keepFocus} from "../../lib/focus.js";
import {time} from "../../lib/format.js";
import {goTop, openGame, openLadder, openMenu, openStep, push, register} from "../../lib/router.js";
import {chapterWords, heroSpoken, movesLabel, number, stripSpoken, unit} from "../../lib/saga.js";
import {openShare, streakCard} from "../../lib/share.js";
import {h} from "../../kit/element.js";
import {gameName, place} from "../games/windows.js";
import {addonLine} from "../menu/computer.js";
import {reportButton} from "../menu/menu.js";
import {pairWindow} from "../menu/phones.js";
import {page, watch, words} from "../page.js";
import {openPlaces} from "../places/places.js";
import {inUse, placeName} from "../places/words.js";
import {loading, nextRow, rowsCard} from "../book/book.js";
import {startMove} from "../book/card.js";
import {lookup} from "../book/common.js";
import {ladderOf, stepAt} from "../book/steps.js";
import {dayTag} from "../log/chapter.js";
import {checkinCard} from "./checkin.js";

/** The standing figure on the root: shadow boxing, its loop at the calm pace (still on the rest after feeling unwell,
 * as the phones' PumpTab, and on its key pose under Reduce Motion: the figure element's own rule). */
const STANDING = "shadow_boxing";
/** The figure's least and most heights (iOS PumpTab's `stageLeast` and `stageMost`). */
const STAGE_LEAST = 112;
const STAGE_MOST = 320;

/** The status Chip for what needs fixing (`home.fix`, Rust's `Fix`): its words, its dot and the Menu page it opens;
 * none while all is well. This computer's state takes the tray's words and the tray icon's colour at the same moment
 * (the `menu.computer` topic's `state_line` and `tone`, helper/src-tauri/src/desk/tray.rs `Tone`, named as the Chip
 * names its dots); a save that failed and the firewall take the stop's red dot. */
function chip(fix, computer) {
	switch (fix) {
		case "computer":
			return {label: computer.state_line, dot: computer.tone, to: "desk.menu.computer"};
		case "save":
			return {label: t("desk.home.fix.save"), dot: "stopped", to: "desk.menu.computer"};
		case "firewall":
			return {label: t("desk.home.fix.firewall"), dot: "stopped", to: "desk.menu.phones"};
		default:
			return null;
	}
}

/** A recent game's name: World of Warcraft's own words, else its tile's (`gameName`). */
function recentName(game) {
	return game.wow ? t("menu.yours.game") : gameName(game);
}

/** An offer's Start (the gold primary, or a quiet button under another primary), Pump for <game> after a match's end
 * and a program's close alike (slice 57), the game by the offer's own name, as the tray and the Games tab's Now card
 * name it (a named game's as contract/links.json's computer list writes it; a program's own): the pump at Ready for
 * the offer's kind at its size, as the card's Start and the Now card's (2026-10-28, Rapha: "it should directly open and
 * start and not let me select stuff"). */
function offerStart(offer) {
	return {label: t("games.pump", {game: offer.name}),
		go: () => intent("pump.start", {kind: offer.kind, offer: offer.id}).catch(console.error)};
}

/** A card that offers the next part with its heading (its question), its way there and Not now, which puts it away
 * for a week (`home.later`): the card, its heading and its way there. */
function laterCard(card, title, label, go) {
	const later = h("qp-quiet-button", {label: t("desk.home.later"),
		"on:click": () => intent("home.later", {card}).catch(console.error)});
	const head = h("h2", {class: "qp-type-heading", text: title});
	const way = h("qp-secondary-button", {label, "on:click": go});
	const el = h("qp-card", {class: "qp-stack-small"}, head, way, later);
	el.hidden = true;
	return {el, head, way};
}

/** The banked moves Pump home's card shows (the rest in its window). */
const BANKED_SHOWN = 3;

/** A banked move's tile key: the move it rides as (`shown`), none before the Book's words came. */
function bankedKey(w, m) {
	return w?.moves.get(m.shown)?.key ?? null;
}

/** A move in the banked window: its tile, its name, and with `skip` the game's Skip beside it (one click, no confirm;
 * David, 2026-10-04: "the skip should be per unfinished move without a confirm"), which `home.skip` writes. */
function bankedMove(w, m, skip) {
	const lead = h("span", {class: "qp-banked-name"}, h("qp-move-tile", {size: 44, move: bankedKey(w, m),
		"aria-hidden": "true"}), h("span", {class: "qp-type-body-strong", text: m.name}));
	if (!skip) return h("div", {class: "qp-waiting-move"}, lead);
	return h("div", {class: "qp-waiting-move"}, lead, h("qp-quiet-button", {label: t("pump.unfinished.skip"),
		spoken: t("pump.unfinished.skip.named", {move: m.name}), hint: t("pump.unfinished.skip.hint"),
		"on:click": () => intent("home.skip", {move: m.move, ...(m.ts == null ? {} : {ts: m.ts})}).catch(console.error)}));
}

/** Every banked move in a Window from the bottom (2026-11-03, Rapha: the Skip left Pump home's card), each with its
 * Skip while no game client holds them, and Catch up while one can be paid here; it closes once none is left. */
function bankedWindow() {
	const box = h("qp-window", {placement: "bottom"});
	const list = h("div", {class: "qp-waiting-moves"});
	const rested = words(t("pump.unfinished.held"), "body");
	box.content.append(list, rested);
	let shown = false;
	let drawn = "";
	box.addEventListener("qp-closed", () => {
		shown = false;
	});
	box.addEventListener("qp-primary", () => {
		box.close();
		intent("pump.catchup").catch(console.error);
	});
	const win = {
		/** Draws it on `home`'s banked moves (`w` the Book's words for the tiles); closes it once none is left or
		 * felt unwell rests the pumps. */
		draw(home, w) {
			const wait = home.waiting;
			if (!wait?.moves.length || home.locked_until != null) {
				if (shown) box.close();
				return;
			}
			box.heading = t("pump.waiting.title", {count: wait.moves.length});
			box.primary = catchUp(home) ? t("pump.waiting.catchup") : null;
			rested.hidden = !wait.rested;
			// made again only when the moves, their Skip or the tiles' words changed, so a focused Skip stays put
			const listed = JSON.stringify([wait.moves, wait.skip, Boolean(w)]);
			if (listed === drawn) return;
			drawn = listed;
			keepFocus(list, () => list.replaceChildren(...wait.moves.map((m) => bankedMove(w, m, wait.skip))));
		},
		open(home, w) {
			shown = true;
			drawn = "";
			win.draw(home, w);
			if (shown) box.open();
		},
		get shown() {
			return shown;
		},
	};
	return win;
}

/** Today's chapter (`home`'s `today`, the Saga's day), the whole card a button that opens the Saga on today, as the
 * phones lay it out (board A-Home): Today's chapter with the chevron at its end; the moves with the chapter's tag and
 * the day's word over Moves today (Move today for 1) beside them; the strip at full height; its records (the line gone
 * at none); read as it shows. Before the day's first move, No moves yet today over the strip's empty axis; before
 * Rust's first build of it, the busy pips. */
function chapterCard() {
	const counts = h("span", {class: "qp-chapter-counts"});
	const tag = h("qp-slant-tag", {});
	const count = h("span", {class: "qp-chapter-number"});
	const word = h("span");
	const label = h("span", {class: "qp-saga-unit"});
	const none = h("span", {class: "qp-type-heading", text: t("saga.none.title")});
	const big = h("span", {class: "qp-chapter-big"}, count, h("span", {class: "qp-chapter-side"},
		h("span", {class: "qp-chapter-tags"}, tag, word), label));
	const strip = h("qp-day-strip");
	const busy = loading();
	const button = h("button", {type: "button", class: "qp-chapter-card", "on:click": () => goTop("log")},
		h("span", {class: "qp-chapter-top", "aria-hidden": "true"}, h("span", {class: "qp-type-caps", text: t("saga.home")}),
			h("i", {class: "qp-chevron"})),
		h("span", {class: "qp-chapter-body", "aria-hidden": "true"}, big, none, busy, strip, counts));
	return {
		el: h("qp-card", {class: "qp-chapter"}, button),
		/** Draws today's chapter; the busy pips before Rust's first build of it (`today` none). */
		draw(today) {
			const moves = today?.moves ?? 0;
			busy.hidden = Boolean(today);
			big.hidden = !today || moves === 0;
			none.hidden = !today || moves > 0;
			strip.hidden = !today;
			counts.replaceChildren();
			const said = [];
			const records = moves > 0 ? today.records : 0;
			if (records > 0) {
				counts.append(h("b", {text: number(records)}), ` ${unit(records, "record")}`);
				said.push(`${number(records)} ${unit(records, "record")}`);
			}
			if (moves > 0) {
				tag.label = chapterWords(today.chapter);
				count.textContent = number(moves);
				label.textContent = movesLabel(moves, "today");
				word.replaceChildren(dayTag(today.quality) ?? "");
			}
			counts.hidden = records === 0;
			strip.bars = today?.bars ?? [];
			strip.buckets = today?.buckets ?? [0, 0, 0, 0];
			strip.label = moves > 0 ? stripSpoken(today.buckets) : t("saga.none.title");
			const spoken = !today ? t("kit.button.busy") : moves > 0 ? [heroSpoken(today), ...said].join(", ")
				: t("saga.none.title");
			button.setAttribute("aria-label", `${t("saga.home")}: ${spoken}`);
		},
	};
}

/** A catch-up starts here: moves wait, no game client holds them (their Skip shows) and one no pain flag rests. */
function catchUp(home) {
	return Boolean(home.waiting?.skip && !home.waiting.rested);
}

/** The Quick pump's button words and its start (Rust's gates hold it). */
const QUICK = {label: () => t("pump.quick"), go: () => intent("pump.quick").catch(console.error)};

/** The foot's buttons for the primary Rust chose: the gold one first, then the quiet ways. */
function foot(home) {
	const p = home.primary;
	if (!p) return [];
	const busy = home.engine === "busy";
	const pumping = p.kind === "catchup" || p.kind === "offer" || p.kind === "quick";
	const state = pumping && busy ? "busy" : p.kind === "catchup" && !p.payable ? "disabled" : "normal";
	let primary;
	switch (p.kind) {
		case "grant":
			primary = h("qp-primary-button", {label: t(p.asked ? "desk.menu.computer.screen.settings"
				: "desk.menu.computer.screen.allow"), "on:click": () => call("allow_screen").catch(console.error)});
			break;
		case "firewall":
			primary = h("qp-primary-button", {label: t("desk.menu.phones.firewall.allow"),
				"on:click": () => call("allow_network").catch(console.error)});
			break;
		case "catchup":
			primary = h("qp-primary-button", {label: t("pump.waiting.catchup"), state,
				"on:click": () => intent("pump.catchup").catch(console.error)});
			break;
		case "offer": {
			const start = offerStart(home.offer);
			primary = h("qp-primary-button", {label: start.label, state, "on:click": start.go});
			break;
		}
		default:
			primary = h("qp-primary-button", {label: QUICK.label(), glyph: "action_pump", state, "on:click": QUICK.go});
	}
	const quiet = [];
	// Quick pump beside a primary of another kind, as the phones' foot keeps it under Catch up; under another primary
	// a standing offer keeps its Start; none while the engine folds, when the busy primary shows alone
	const asks = !home.game && !busy;
	if (asks && p.kind !== "quick") quiet.push(h("qp-quiet-button", {label: QUICK.label(), "on:click": QUICK.go}));
	// moves that waited longer than owed_fresh_days game days leave Quick pump the primary: Catch up stays a quiet way
	// while one of them can be paid here (none a pain flag rests, no game client holding them)
	if (asks && catchUp(home) && p.kind !== "catchup") {
		quiet.push(h("qp-quiet-button", {label: t("pump.waiting.catchup"),
			"on:click": () => intent("pump.catchup").catch(console.error)}));
	}
	if (asks && home.offer && p.kind !== "offer") {
		const start = offerStart(home.offer);
		quiet.push(h("qp-quiet-button", {label: start.label, "on:click": start.go}));
	}
	return [primary, ...quiet];
}

register("pump", () => {
	const view = page({heading: t("bar.pump"), root: true});
	view.el.classList.add("qp-home");
	// the place in use's glyph left of the gear, opening the places window
	const placeButton = h("qp-round-button", {icon: "place_home", "on:click": () => openPlaces()});
	view.bar.action = placeButton;
	let chipTo = "desk.menu.computer";
	const status = h("qp-chip", {button: true, hidden: true, "on:click": () => openMenu(chipTo)});
	// the recent games, each a GameStat made once and kept while the topic lists it
	const stats = new Map();
	const carousel = h("qp-carousel", {previous: t("desk.home.games.previous"), next: t("desk.home.games.next")});
	carousel.ensure();
	const recentRow = carousel.track;
	const recentLabel = h("qp-section-label");
	// This week | All time: the choice shows at once, and Rust keeps it and orders the games by it
	const spanSwitch = h("qp-segmented", {class: "qp-recent-span", small: true, label: t("desk.home.games.span"),
		"on:qp-change": (e) => {
			span = e.detail;
			drawRecent();
			intent("home.span", {span}).catch(console.error);
		}});
	spanSwitch.options = [{value: "week", label: t("desk.home.games.span.week")},
		{value: "all", label: t("desk.home.games.span.all")}];
	const recent = h("div", {class: "qp-recent", hidden: true},
		h("div", {class: "qp-recent-head"}, recentLabel, spanSwitch), carousel);
	const figure = h("qp-figure-stage", {move: STANDING, calm: true, "data-hero": ""});
	const board = h("qp-streak-board", {"on:qp-share": () => openShare(streakCard(board.days)),
		"on:qp-open": () => push("desk.streak")});
	// today's embers (slice 55) under Up next: a click starts the cardio step's move (2026-11-03), its card where Rust
	// refuses the start (a gate, the game asking the pumps), which says why
	let embersTo = null;
	let w = null;
	let book = null;
	let locked = false;
	/** The player's cardio step on the Book (its first before one), none before the Book's words and topic came. */
	const cardioStep = () => {
		const lad = embersTo && w && book ? ladderOf(w, book, embersTo.ladder) : null;
		const st = lad ? lad.now ?? stepAt(lad, 1) : null;
		return st?.rung ? st : null;
	};
	const startCardio = () => {
		if (!embersTo) return;
		const {ladder, step} = embersTo;
		const card = () => openStep(ladder, step);
		const st = cardioStep();
		if (!st) return card();
		startMove({ladder, rung: st.rung}, {place: "home", ladder, step: st.step}, card);
	};
	const embersPips = h("qp-day-pips", {"aria-hidden": "true"});
	const embersRow = h("qp-row", {trailing: "nav", "on:click": startCardio});
	embersRow.extra = embersPips;
	const embers = h("qp-card", {rows: true, hidden: true}, embersRow);
	// the step-up closest to done (`book.closest`, book.js `nextRow`)
	const closest = h("div", {hidden: true});
	const upNext = h("div", {class: "qp-stack-small", hidden: true},
		h("qp-section-label", {label: t("ladders.next")}), embers, closest);
	const drawUp = () => {
		embers.hidden = locked || !embersTo;
		if (!embers.hidden) {
			embersRow.heading = t("pump.home.embers", {have: embersTo.have, need: embersTo.need});
			embersPips.have = embersTo.have;
			embersPips.need = embersTo.need;
			const st = cardioStep();
			embersRow.move = st ? w.moves.get(st.move)?.key ?? null : null;
		}
		keepFocus(closest, () => {
			const row = !locked && w && book?.closest != null ? nextRow(w, book, book.closest, openLadder) : null;
			closest.hidden = !row;
			closest.replaceChildren(...(row ? [rowsCard([row])] : []));
		});
		upNext.hidden = embers.hidden && closest.hidden;
	};

	const restHead = h("h2", {class: "qp-type-heading", text: t("pump.rest.title")});
	const restLine = words("", "subtitle");
	const rest = h("qp-card", {class: "qp-rest"}, h("qp-art", {name: "mark_lock"}),
		h("div", {class: "qp-stack-small"}, restHead, restLine));

	// the banked moves: the heading and their first moves with their tiles, the whole card opening their window
	const banked = bankedWindow();
	let homeShown = null;
	const openBanked = () => homeShown && banked.open(homeShown, w);
	const waitHead = h("qp-row", {trailing: "nav", "on:click": openBanked});
	const waitList = h("qp-card", {rows: true}, waitHead);
	const waitHeld = words("", "meta");
	const waitRested = words(t("pump.unfinished.held"), "body");
	const waiting = h("div", {class: "qp-stack-small"}, waitList, waitHeld, waitRested);
	let movesShown = "";
	/** The card's first moves from the last `home` with the Book's words for their tiles, and the window while open. */
	const drawBanked = () => {
		const wait = homeShown?.waiting;
		if (wait) {
			const first = wait.moves.slice(0, BANKED_SHOWN);
			const listed = JSON.stringify([first, Boolean(w)]);
			if (listed !== movesShown) {
				movesShown = listed;
				keepFocus(waitList, () => waitList.replaceChildren(waitHead, ...first.map((m) => h("qp-row", {
					heading: m.name, move: bankedKey(w, m), trailing: "action", "on:click": openBanked}))));
			}
		}
		if (homeShown && banked.shown) banked.draw(homeShown, w);
	};

	const today = chapterCard();
	const held = words("", "meta");
	const todayBlock = h("div", {class: "qp-stack-small"}, today.el, held);

	const checkin = checkinCard();
	const pair = laterCard("pair", t("desk.home.pair.title"), t("desk.menu.phones.pair"), pairWindow);
	const addon = laterCard("addon", t("desk.addon.missing"), t("desk.addon.add"),
		() => call("addon_install").catch(console.error));
	const column = h("qp-hero-column", {least: STAGE_LEAST, most: STAGE_MOST}, status, recent, figure, board, upNext,
		rest, waiting, todayBlock, checkin.el, pair.el, addon.el);
	view.body.append(column);

	const gameLine = words(t("pump.waiting.game"), "meta");
	// pumps that could not start on this computer, beside the Menu's report for help
	const report = reportButton();
	const failed = h("div", {class: "qp-stack-small"}, words(t("desk.home.failed"), "meta"), report.el);
	// the match's offer in its notification's words (reminders.notify.game.*): the title and its one line with the game's
	// own name
	const offerLine = words("", "subtitle");
	const offer = h("div", {class: "qp-offer"}, h("p", {class: "qp-type-heading",
		text: t("reminders.notify.game.title.match")}), offerLine);
	const buttons = h("div", {class: "qp-foot-buttons"});
	const footer = h("div", {class: "qp-foot"}, gameLine, failed, offer, buttons);
	view.el.append(footer);
	view.stops.push(report.stop);
	let shown = "";

	/** The foot's buttons from the last `home`. */
	function drawFoot() {
		const home = homeShown;
		if (!home) return;
		// the foot is made again only when what it offers changed, so a hover, a press or the focus stays
		const offers = JSON.stringify([home.primary, home.game, home.engine, home.offer, catchUp(home)]);
		if (offers !== shown) {
			shown = offers;
			const focused = buttons.contains(document.activeElement);
			buttons.replaceChildren(...foot(home));
			if (focused) buttons.querySelector("qp-primary-button")?.focus({preventScroll: true});
		}
	}

	// the Chip and World of Warcraft's dot from what needs fixing and This computer's state, as they change
	let computer = null;
	let fix = null;
	let games = [];
	let span = "week";
	const drawChip = () => {
		const c = computer && chip(fix, computer);
		status.hidden = !c;
		if (!c) return;
		status.label = c.label;
		status.state = c.dot;
		chipTo = c.to;
	};
	/** A recent game's GameStat, made once and kept while the topic lists it: a click opens This computer for World of
	 * Warcraft, else the game's window on the Games tab. */
	const statOf = (game) => {
		let el = stats.get(game.key);
		if (!el) {
			el = h("qp-game-stat", {"data-focus-key": `recent:${game.key}`,
				"on:qp-open": () => (game.wow ? openMenu("desk.menu.computer") : openGame(game.key))});
			stats.set(game.key, el);
		}
		const name = recentName(game);
		el.name = name;
		el.icon = game.icon;
		const count = SPANS[span].of(game);
		el.count = count;
		el.playing = game.playing;
		el.owed = game.wow && fix === "computer" && computer ? computer.state_line : null;
		el.spoken = [name, game.playing ? t("games.tile.playing") : null,
			t(SPANS[span].count, {count})].filter(Boolean).join(", ");
		return el;
	};
	// a game gone or moved keeps the focus: the same game, else the one now in its place, else the Chip while it shows
	const drawRecent = () => {
		recent.hidden = games.length === 0;
		recentLabel.label = t(SPANS[span].label);
		carousel.label = t(SPANS[span].label);
		spanSwitch.value = span;
		const listed = games.map(statOf);
		for (const key of stats.keys()) {
			if (!games.some((game) => game.key === key)) stats.delete(key);
		}
		keepFocus(recentRow, () => place(recentRow, listed),
			(key, near) => near() ?? (status.hidden ? null : status));
	};
	watch(view, "menu.computer", (value) => {
		computer = value;
		drawChip();
		drawRecent();
		// the addon's card says where adding QuestPump to the game stands, busy while it is added
		const line = value.addon.line;
		addon.head.textContent = line.state === "adding" || line.state === "add_failed" ? addonLine(line)
			: t("desk.addon.missing");
		addon.way.busy = line.state === "adding";
	});

	watch(view, "book.words", (value) => {
		w = lookup(value);
		drawUp();
		drawBanked();
	});
	watch(view, "book", (value) => {
		book = value;
		drawUp();
	});

	watch(view, "home", (home) => {
		fix = home.fix ?? null;
		games = home.recent.games;
		span = home.recent.span;
		drawChip();
		drawRecent();
		locked = home.locked_until !== null && home.locked_until !== undefined;
		figure.toggleAttribute("still", locked);
		board.days = home.streak.days;
		board.tokens = home.streak.tokens;
		board.today = home.streak.today;
		board.todayIndex = home.streak.today_index;
		board.resting = locked;
		board.week = home.streak.week;
		board.share = home.milestones.length > 0 && home.streak.days >= Math.min(...home.milestones);
		embersTo = home.embers ?? null;
		drawUp();
		rest.hidden = !locked;
		if (locked) restLine.textContent = t("pump.rest.line", {time: time(home.locked_until)});
		const wait = home.waiting;
		waiting.hidden = locked || !wait;
		todayBlock.hidden = locked || Boolean(wait);
		homeShown = home;
		drawBanked();
		if (wait) {
			waitHead.heading = t("pump.waiting.title", {count: wait.moves.length});
			waitHeld.hidden = !wait.held;
			if (wait.held) waitHeld.textContent = t("pump.waiting.held", {count: wait.held});
			waitRested.hidden = !wait.rested;
		}
		today.draw(home.today);
		held.hidden = !home.held;
		if (home.held) held.textContent = t("pump.held", {count: home.held});
		checkin.draw(home.pump ? null : home.checkin, home.engine !== "ready");
		checkin.failed(home.pump ? null : home.pain_failed);
		pair.el.hidden = home.card?.kind !== "pair" || locked;
		addon.el.hidden = home.card?.kind !== "addon" || locked;
		const stopped = home.engine === "failed" && !locked && !home.pump;
		failed.hidden = !stopped;
		// the game asks the pumps while it runs (while QuestPump has not shown in it yet they start here)
		gameLine.hidden = !home.game || locked || stopped;
		offer.hidden = !home.offer;
		if (home.offer) {
			offerLine.textContent = t("reminders.notify.game.body.computer", {link: home.offer.name});
		}
		drawFoot();
	});
	// the place in use on the TopBar's button: its glyph (Home's where the button has none for it) and its name
	watch(view, "places", (p) => {
		const place = inUse(p);
		placeButton.icon = p.glyphs.includes(place?.glyph) ? place.glyph : "place_home";
		placeButton.spoken = t("places.button", {place: placeName(place)});
	});
	view.primary = () => buttons.querySelector("qp-primary-button") ?? (status.hidden ? null : status);
	return view;
});
