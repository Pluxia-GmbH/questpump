// The Saga's held book (slice 44, finding 13 of slice 34; docs/ui.md, "The Saga"; screens.json desk.log and
// desk.log.day; the phones' HeldSaga and rememberHeldSaga): while the reader is scrolled away from a Saga page's top
// (the root's, a chapter's page), the page keeps drawing the `log` topic it shows. A newer one that would draw otherwise
// waits, so a pump that lands, a sync or a late row moves nothing on screen and the focus stays where it is; one that
// draws alike (the reveal's flag, another page's open day) is taken at once. While the waiting one holds moves done
// the shown one does not, the Chip's new_moves pill floats centred under the TopBar over the page, "N new moves" (the
// page's `fresh` counts them; a pump with no move done adds none), coming and going with the toast's motion (Reduce
// Motion: a fade), said once politely as it comes and as its count changes, never taking the focus; its click or
// Enter takes the page to its top. At the top (scrolled there, the pill, the Saga's tab chosen again or Pump home's
// chapter card: lib/router.js `toTop`) the newest book applies, as every book does while the page stands there. A book
// the reader asked for (Show more chapters) applies where the reader is, the first control in view kept where it
// stands.

import {t} from "../../lib/copy.js";
import {announce, focusView} from "../../lib/focus.js";
import {toTop} from "../../lib/router.js";
import {animationMs, h} from "../../kit/element.js";

/** A page scrolled less than this from its top is at its top (a zoomed page scrolls in fractions of a pixel). */
const TOP_PX = 1;

/** Holds `view`'s book: `same(shown, newer)` whether two books draw alike on the page, `fresh(shown, newer)` how many
 * moves done the newer holds that the shown one does not, `draw()` draws the page from `book`. The page hands every
 * book to `take(book, asked)` (`asked`: the answer to the reader's own Show more chapters) and draws from `book`
 * alone. */
export function holdBook(view, {same, fresh, draw}) {
	let book = null;
	let waiting = null;
	let away = false;
	let inView = false;
	let count = 0;
	let told = 0;
	let leaving = 0;
	const pill = h("qp-chip", {state: "new_moves", button: true, "on:click": () => toTop(view)});
	const host = h("div", {class: "qp-held", hidden: true}, pill);
	view.body.before(host);

	/** Says the pill's words once a count, only while the page shows (a page under another says it as it comes back). */
	const say = () => {
		if (!inView || count === 0 || count === told) return;
		told = count;
		announce(pill.label);
	};

	/** The pill for `n` new moves, gone at none; leaving with the focus on it, the focus goes to the page's start (its
	 * primary, else its heading: `focusView`). */
	const showPill = (n) => {
		count = n;
		if (n > 0) {
			clearTimeout(leaving);
			host.classList.remove("qp-out");
			host.hidden = false;
			pill.label = t("saga.new.moves", {count: n});
			say();
			return;
		}
		told = 0;
		if (host.hidden || host.classList.contains("qp-out")) return;
		const focused = host.contains(document.activeElement);
		host.classList.add("qp-out");
		if (focused) focusView(view);
		leaving = setTimeout(() => {
			host.hidden = true;
			host.classList.remove("qp-out");
		}, animationMs(pill));
	};

	// drawn before the pill goes, so a pill that held the focus hands it to the start of the page as it now stands (Share
	// today gone at a new game day's first book: Start a pump), never to a control the redraw then removes
	const apply = (next) => {
		book = next;
		waiting = null;
		draw();
		showPill(0);
	};

	/** Applies `next` where the reader is: the first keyed control in view stands where it stood after the redraw. */
	const applyHere = (next) => {
		const top = view.el.getBoundingClientRect().top;
		const anchor = [...view.body.querySelectorAll("[data-focus-key]")]
			.find((el) => el.getBoundingClientRect().bottom > top);
		const key = anchor?.getAttribute("data-focus-key");
		const offset = anchor ? anchor.getBoundingClientRect().top - top : 0;
		apply(next);
		const moved = key ? view.body.querySelector(`[data-focus-key="${CSS.escape(key)}"]`) : null;
		if (moved) view.el.scrollTop += moved.getBoundingClientRect().top - top - offset;
	};

	// where the reader is: from the page's scrolling, and again as the page shows; a hidden page (under another page, or
	// another root's) reads no place, its scroll then reading 0 (a scroll event can still come as it hides)
	const look = () => {
		if (!view.el.getClientRects().length) return;
		away = view.el.scrollTop >= TOP_PX;
		if (!away && waiting) apply(waiting);
	};
	view.el.addEventListener("scroll", look, {passive: true});
	const seen = new IntersectionObserver((entries) => {
		inView = entries.some((e) => e.isIntersecting);
		if (!inView) return;
		look();
		say();
	});
	seen.observe(view.body);
	view.stops.push(() => {
		seen.disconnect();
		clearTimeout(leaving);
	});

	return {
		get book() {
			return book;
		},
		take(next, asked = false) {
			if (!book || !away || same(book, next)) apply(next);
			else if (asked) applyHere(next);
			else {
				waiting = next;
				showPill(fresh(book, next));
			}
		},
	};
}
