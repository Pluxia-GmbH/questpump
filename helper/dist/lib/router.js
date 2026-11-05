// Where the page is (docs/ui.md, "The window model"): the five roots, Pump, Hero, Book, Log and Games (the desktop's
// fifth, slice 56), each with its own stack of pages (a tab keeps its stack; choosing the shown tab again takes it back
// to its root page, and that page back to its top, as the phones' TopOnTab does); the Menu's stack over the root it was
// opened from (its gear), which a tab closes; a pump's cover over everything, the ActionBar with it; and the pages its
// Ready view's Settings opens over the pump in the cover (since 2026-11-05: Your setup, and what it pushes), whose back
// returns to the pump as it stands, gone with the cover. A view is made
// by the factory its id registered (views/index.js) and is {el, primary?, onKey?, onFind?, findMove?, resume?,
// onShown?, onLeft?, secret?, stops}: its element, the control its keys go to first, its own keys (keys.js),
// Cmd/Ctrl+F's target, the Book's Moves search that Cmd/Ctrl+F reaches from a view with no search of its own (another
// root, a page on one, the Menu: `find`, slice 46, as on the phones), what takes the focus when a pump's cover goes from
// over it (the step a move started from), what it does each time it comes to the front (the Games tab says it was
// seen) and each time it leaves it (a code that opens the player's account held behind Show code again: views/page.js
// `heldCode`), whether it draws such a code now, which keeps the window out of screen captures while it is the view in
// front (`reveal`, `conceal`), and what it stops listening to when it goes. Rust takes the page somewhere through the
// `shell` topic's `go` ({to, seq, chosen}: a
// root's id or a Menu page's), taken once per seq; a root the player `chosen` in the macOS View menu is a press of its
// tab, as Cmd/Ctrl+1 to 5 are (lib/keys.js `takeFrame`; slice 46), its sound and its count with it, none of them under
// a pump's cover or a window, which the tabs stand behind; the menu's five tabs are greyed meanwhile (`view.tabs`,
// desk/menu.rs). The top view gets the keys and the focus (its primary, else its heading); a page popped gives the
// focus back to the control that opened it (the one its click pressed, else the one focused), and to the view's own
// when none did (Rust opened it); where the page under it drew again meanwhile (a sync, a sheet), to the opener's
// replacement by its key (lib/focus.js). A window that closes giving the focus back nowhere (its opener and its key
// drawn no more, or out of reach under a pump's cover) hands it to the view in front's own, never to the page's body.

import {call, intent, subscribe} from "./bridge.js";
import {t} from "./copy.js";
import {controlOf, focusView, giveBack, keyOf, topWindow} from "./focus.js";
import {mounted, takeFrame} from "./keys.js";
import {ROOTS} from "../kit/qp-root-host.js";

/** The Menu's own page, under every Menu page. */
export const MENU = "desk.menu";

const factories = new Map();
let host = null;
let root = "pump";
const stacks = Object.fromEntries(ROOTS.map((r) => [r, []]));
let menu = [];
let covering = null;
let above = []; // the pages over a pump's cover, in it (`over`)
let went = 0;
let pressed = null; // the control the click being dispatched pressed (a mouse click focuses no button in WebKit)
let tabsOn = null; // whether the tabs can act, as Rust was last told (`tellTabs`)
let told = null; // whether the window is kept out of screen captures, as Rust was last asked (`ask`)
let telling = Promise.resolve(); // the last of those asks, answered once it settles
let front = null; // the view in front as `show` last placed it
/** What waits for the frame's start (`whenStarted`). */
const starting = [];
const still = window.matchMedia("(prefers-reduced-motion: reduce)");

/** Runs `fn` once the frame has started (at once when it has): a module that follows a topic to cover the window or
 * open a window over any view waits for the frame its views go into. */
export function whenStarted(fn) {
	if (host) fn();
	else starting.push(fn);
}

/** Registers a view's factory by its id (a root's, or a screen's of contract/design/screens.json). */
export function register(id, factory) {
	factories.set(id, factory);
}

/** Whether a view's id is registered: a page that links another area's page shows its row only once it is. */
export function registered(id) {
	return factories.has(id);
}

function make(id, params) {
	const factory = factories.get(id);
	if (!factory) throw new Error(`router: no view ${id}`);
	const opener = pressed ?? document.activeElement;
	return {id, view: factory(params ?? {}), opener, openerKey: keyOf(opener)};
}

function drop(entry) {
	for (const stop of entry.view.stops ?? []) stop();
	entry.view.el.remove();
}

function top() {
	return above.at(-1) ?? covering ?? menu.at(-1) ?? stacks[root].at(-1);
}

/** Whether a pump covers the window (a moment that waits for the pump's end, the best day's, asks). */
export function covered() {
	return Boolean(covering);
}

/** Whether `view` is the view in front: the last page over a pump's cover, else the pump, else the top page of the
 * Menu or of the root shown (the Games tab, in front as a game is found, has seen it). */
export function isFront(view) {
	return front === view;
}

/** Shows the top page of what is in front, hands it the keys and the focus (`focus`: the element to focus instead, or
 * its replacement by `key`) and, when it was not in front, says so to it (`onShown`); a cover going says so to the
 * document (`qp-uncovered`) once the focus is placed, so a window that opens then (the best day's moment) gives the
 * focus back there as it closes. */
function show(focus, key) {
	const uncovered = !covering && !host.cover.hidden;
	host.cover.hidden = !covering;
	host.bar.inert = Boolean(covering);
	host.stage.inert = Boolean(covering);
	host.menu.hidden = !menu.length;
	for (const r of ROOTS) host.stacks[r].hidden = r !== root || menu.length > 0;
	for (const list of [...Object.values(stacks), menu, above]) {
		list.forEach((entry, i) => {
			entry.view.el.hidden = i !== list.length - 1;
		});
	}
	if (covering) covering.view.el.hidden = above.length > 0;
	host.bar.selected = root;
	const entry = top();
	if (entry) {
		mounted(entry.view);
		// the page's body is no control (a page Rust opened had the focus nowhere): the view's own focus instead
		if (!giveBack(focus, key, entry.view.el)) focusView(entry.view);
	}
	if (entry?.view !== front) {
		front?.onLeft?.();
		front = entry?.view ?? null;
		front?.onShown?.();
	}
	if (uncovered) document.dispatchEvent(new CustomEvent("qp-uncovered"));
	tellTabs();
	tellSecret();
}

/** Asks Rust to keep the window out of screen captures, recordings and shares (`on`) or to let them see it again
 * (`secret_shown`, desk/commands.rs), once the ask before it was answered, so they arrive in order; the promise settles
 * once Rust answered (a refusal is logged: the window stays as it was). */
function ask(on) {
	told = on;
	telling = telling.then(() => call("secret_shown", {on})).catch(console.error);
	return telling;
}

/** Keeps the window out of screen captures while the view in front draws a code that opens the player's account (its
 * `secret`), never for one under a pump's cover or another page: asked at once as one comes, the promise `reveal`
 * awaits before the code draws, and let go only two frames after the code went, so the frame on screen no longer
 * holds it (a hidden window draws no frame and stays out meanwhile). The page's first `show` says none draws, so a page
 * loaded anew lets the window be seen again. */
function tellSecret() {
	if (front?.secret) return told === true ? telling : ask(true);
	if (told !== false) {
		requestAnimationFrame(() => requestAnimationFrame(() => {
			if (!front?.secret && told !== false) ask(false);
		}));
	}
	return telling;
}

/** The player pressed Show code on `view`: true once the window is kept out of screen captures and the view may draw
 * its code, false when it left the front meanwhile (its `onLeft` held the code again). */
export async function reveal(view) {
	if (front !== view) return false;
	view.secret = true;
	await tellSecret();
	return front === view && view.secret;
}

/** `view` draws its code no more (held behind Show code again, or the code gone): the window is seen by screen captures
 * again once the frame on screen no longer holds it (`tellSecret`). */
export function conceal(view) {
	view.secret = false;
	tellSecret();
}

/** Whether the tabs can act: no pump covers the window and no window stands over the page, the ActionBar out of reach
 * behind either. */
function tabsFree() {
	return !covering && !topWindow();
}

/** Tells Rust whether the tabs can act as that changes (`view.tabs`): the macOS View menu greys its five tabs while
 * they cannot (desk/menu.rs), as the iPad greys its commands. */
function tellTabs() {
	const on = tabsFree();
	if (on === tabsOn) return;
	tabsOn = on;
	intent("view.tabs", {on}).catch(console.error);
}

function place(entry, section) {
	section.append(entry.view.el);
	return entry;
}

/** The player chose a root's tab or the Menu: said to the counts (`feature_day`'s hero, book, log, games and menu). */
function seen(feature) {
	if (["hero", "book", "log", "games", "menu"].includes(feature)) intent("usage.seen", {feature}).catch(console.error);
}

/** Takes a view's page back to its top (a root's tab chosen again, the Saga's new-pumps pill): smoothly, at once under
 * Reduce Motion (the phones' TopOnTab). */
export function toTop(view) {
	view.el.scrollTo({top: 0, behavior: still.matches ? "auto" : "smooth"});
}

/** Chooses a root: the Menu closes; the shown root's tab again takes its stack back to its root page, or, on that page,
 * the page back to its top (the Saga's held book then applies there). */
export function go(to) {
	const again = to === root && !menu.length && stacks[root].length === 1;
	closeMenu(false);
	if (to === root) {
		while (stacks[root].length > 1) drop(stacks[root].pop());
	}
	root = to;
	show();
	if (again) toTop(stacks[root][0].view);
}

/** The player chose a root by its key or the View menu: its tab pressed (kit/qp-action-bar.js `press`: its sound, its
 * qp-tab, which `go` and the views' Tooltips hear); none while a pump covers the window or a window stands over the
 * page, the tabs out of reach behind it. */
function choose(to) {
	if (!tabsFree()) return;
	host.bar.press(to);
}

/** Cmd/Ctrl+F in a view with no search of its own (another root, a page on one, the Menu), and Your moves' Book row:
 * the Book on its own page with its Moves search's keyboard (its view's `findMove`), as the phones' Find a move from
 * anywhere; none while a pump covers the window or a window stands over the page. */
export function find() {
	if (!tabsFree()) return;
	goTop("book");
	stacks.book[0].view.findMove?.();
}

/** Pump home's embers row (slice 55): the Book on its own page with step `step` of ladder `ladder` (the cardio step)
 * selected and its card open (its view's `openStep`); none while a pump covers the window or a window stands over the
 * page. */
export function openStep(ladder, step) {
	if (!tabsFree()) return;
	goTop("book");
	stacks.book[0].view.openStep?.(ladder, step);
}

/** Pump home's closest step-up (2026-11-02): the Book on its own page with ladder `ladder`'s page pushed, step `step`
 * in view (its view's `openLadder`); none while a pump covers the window or a window
 * stands over the page. */
export function openLadder(ladder, step) {
	if (!tabsFree()) return;
	goTop("book");
	stacks.book[0].view.openLadder?.(ladder, step);
}

/** Pump home's recent games (2026-10-27): the Games tab on its own page with the window of the game `key` open (its
 * view's `openGame`); none while a pump covers the window or a window stands over the page. */
export function openGame(key) {
	if (!tabsFree()) return;
	goTop("games");
	stacks.games[0].view.openGame?.(key);
}

/** Opens a root on its own page at its top whatever its stack held (Pump home's chapter card opens the Saga on today),
 * said to the counts as a tab is. */
export function goTop(to) {
	closeMenu(false);
	while (stacks[to].length > 1) drop(stacks[to].pop());
	root = to;
	show();
	toTop(stacks[to][0].view);
	seen(to);
}

/** Pushes a page onto the stack in front (the pages over a pump's cover while one stands, else the Menu's while it is
 * open). */
export function push(id, params) {
	if (above.length) {
		above.push(place(make(id, params), host.cover));
	} else {
		const list = menu.length ? menu : stacks[root];
		list.push(place(make(id, params), menu.length ? host.menu : host.stacks[root]));
	}
	show();
}

/** Opens a page over a pump's cover (its Ready view's Settings: Your setup, 2026-11-05), the pump under it out of
 * sight and its keys with the page, the ActionBar still out of reach; Back from it returns to the pump as it stands
 * then. None without a cover. */
export function over(id, params) {
	if (!covering) return;
	above.push(place(make(id, params), host.cover));
	show();
}

/** Back one page; the Menu's own page closes the Menu, and the last page over a pump's cover returns to the pump. */
export function back() {
	if (above.length) {
		const gone = above.pop();
		drop(gone);
		show(gone.opener, gone.openerKey);
		return;
	}
	const list = menu.length ? menu : stacks[root];
	if (list.length <= 1) {
		if (menu.length) closeMenu(true);
		return;
	}
	const gone = list.pop();
	drop(gone);
	show(gone.opener, gone.openerKey);
}

/** Opens the Menu over the root, on one of its pages when `page` names one (with its `params`: Your setup's `focus`,
 * the switch or the piece a refused Start names). */
export function openMenu(page, params) {
	closeMenu(false);
	menu.push(place(make(MENU), host.menu));
	if (page && page !== MENU) menu.push(place(make(page, params), host.menu));
	show();
}

function closeMenu(focusBack) {
	if (!menu.length) return;
	const opener = menu[0].opener;
	for (const entry of menu.splice(0)) drop(entry);
	if (focusBack) show(opener);
}

/** Covers the whole window with a view (a pump), the ActionBar and the pages under it out of reach; one that takes
 * another's place never leaves the window uncovered between them (no `qp-uncovered`). */
export function cover(id, params) {
	dropAbove();
	if (covering) drop(covering);
	covering = place(make(id, params), host.cover);
	show();
}

/** The pages over a pump's cover go with it. */
function dropAbove() {
	for (const entry of above.splice(0)) drop(entry);
}

/** Back to the pump from the pages over its cover (its stage left Ready meanwhile: a setup change that left it no
 * move ended it, and its Result must not run unseen under them); nothing with none. */
export function toCover() {
	if (!above.length) return;
	dropAbove();
	show();
}

/** Takes the cover away (`show` then says so, `qp-uncovered`); the view in front may take the focus back where the
 * pump started (`resume`, run with `covered()` false already: a move started from a step's card returns to its step,
 * slice 38), else its own focus. */
export function uncover() {
	if (!covering) return;
	dropAbove();
	drop(covering);
	covering = null;
	show(top()?.view.resume?.() ?? null);
}

/** Goes where Rust says: a root, or a Menu page. */
function arrive(to) {
	if (covering) return;
	if (ROOTS.includes(to)) go(to);
	else openMenu(to);
}

/** Starts the frame in `rootHost`: every root's page made, the ActionBar's, the gear's and the back buttons' events
 * taken, a closed window's lost focus placed, and Rust's `go` followed. */
export function startRouter(rootHost) {
	host = rootHost;
	host.ensure();
	for (const r of ROOTS) stacks[r].push(place(make(r), host.stacks[r]));
	// the control a click (or a key's) presses is the opener of the page it opens, kept until the click's handlers ran
	document.addEventListener("click", (e) => {
		pressed = controlOf(e.target);
		setTimeout(() => {
			pressed = null;
		});
	}, true);
	host.addEventListener("qp-tab", (e) => {
		go(e.detail);
		seen(e.detail);
	});
	host.addEventListener("qp-menu", () => {
		openMenu();
		seen("menu");
	});
	host.addEventListener("qp-back", () => back());
	takeFrame({tab: choose, find});
	// a window opening or closing over the page: the View menu's tabs follow (`tellTabs`)
	new MutationObserver((records) => {
		if (records.some((r) => r.target.localName === "dialog")) tellTabs();
	}).observe(document.body, {subtree: true, attributes: true, attributeFilter: ["open"]});
	// a window closed whose opener the page under it drew away meanwhile (the Saga's Share today gone with a new game
	// day while its Share window stood) or a pump's cover put out of reach (the Share window a pump closes, its opener
	// under that pump): the view in front's own focus (the pump's there), once no window is left open
	document.addEventListener("qp-closed", () => {
		const focused = document.activeElement;
		if (topWindow() || (focused && focused !== document.body)) return;
		const entry = top();
		if (entry) focusView(entry.view);
	});
	subscribe("shell", (shell) => {
		if (!shell.go || shell.go.seq <= went) return;
		went = shell.go.seq;
		if (shell.go.chosen) choose(shell.go.to);
		else arrive(shell.go.to);
	});
	// the tabs' dots: Pump's, the unfinished moves a catch-up here pays that waited at most owed_fresh_days game days;
	// Games', a game found since the tab was last opened
	subscribe("home", (home) => {
		const count = home.dot ?? 0;
		host.bar.mark("pump", count > 0 ? t("pump.unfinished.header", {count}) : null);
	});
	subscribe("games", (games) => host.bar.mark("games", games.dot ? t("games.new") : null));
	if (!went) show();
	for (const fn of starting.splice(0)) fn();
}
