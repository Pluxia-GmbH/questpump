// Window: a modal card over the dim (docs/ui.md, "What stays native, and the kit"), centred or from the bottom
// (`placement`): a heading, a round close, its content (the element's own children), an optional primary button
// (`primary`, its words; `primary-state`, the PrimaryButton's) and under it an optional quiet one (`secondary`, its
// words: the state with_secondary, the reminder offer's Not now), modal to a screen reader and named by its heading
// (`label` names one whose content carries its own heading), entering with the window motion and leaving with windowOut
// (Reduce Motion: a fade each way). A native <dialog> opened with showModal: the focus goes in, the page behind is
// inert, and the focus goes back to its opener when it closes; its own polite live region says what lib/focus.js
// announces while it is open (the page's is inert behind it). Escape closes it as the × does, never `locked`
// (closedby="none": the place window, the check-in's joints), which has no × either and closes only by its own buttons
// (keys.js keeps Escape from it where the engine lacks closedby). `dim`, on a window that only shows (a step's card),
// closes it on a click on the dim around it as the × does; never on one that asks a choice, the safety, the pairing or
// a confirmation. Opened again while it leaves, it stays. Fires qp-primary, qp-secondary, qp-dismiss as the player
// closes it by its × or Escape (lib/sound.js sounds the close then, never at a close its owner makes, as the phones'
// Window fires `.close` only from its own close), and, once closed, qp-closed. Its heading and its buttons stay put
// while the content scrolls between them, with what the caller pins at its foot (`foot`: a step's card's and a move
// window's Start, a qp-start-block, so a greyed Start's reason and the "To get here" row never scroll away; the states
// primary_note and primary_block). Opened from a tapped tile (`open`'s `origin`: a tree's node, a ladder page's step
// row, a Moves tile; slice 46, the state grow) it grows out of that tile, its dim fading in with it, and the player's
// own close (its ×, Escape, the dim) shrinks it back into the tile while that is half in view (lib/grow.js); its other
// closes (a Start, a link, a layout) leave as any window does, and a swap of what it shows (a step's card another
// step's takes the place of) clears `origin`, so it shrinks into no tile.

import {holdFocus} from "../lib/focus.js";
import {GROWN, grow, settle, shrink} from "../lib/grow.js";
import {guard} from "../lib/keys.js";
import {KitElement, animationMs, define, emit, h} from "./element.js";

class QpWindow extends KitElement {
	build() {
		this.head = h("h2", {class: "qp-type-heading"});
		this.closer = h("qp-round-button", {icon: "close", "on:click": () => this.dismiss()});
		this.body = h("div", {class: "qp-window-body"}, ...this.childNodes);
		this.primaryButton = h("qp-primary-button", {"on:click": () => emit(this, "qp-primary")});
		this.secondaryButton = h("qp-quiet-button", {"on:click": () => emit(this, "qp-secondary")});
		this.footEl = h("div", {class: "qp-window-foot"});
		this.dialog = h("dialog", {class: "qp-window"},
			h("header", {class: "qp-window-head"}, this.head, this.closer), this.body, this.primaryButton,
			this.secondaryButton, this.footEl, h("div", {class: "qp-unseen", "aria-live": "polite", "data-live": ""}));
		this.dialog.addEventListener("cancel", (e) => {
			e.preventDefault();
			if (!this.locked) this.dismiss();
		});
		// a click on the dim lands on the dialog itself, outside its box (a modal dialog's ::backdrop is its own)
		this.dialog.addEventListener("click", (e) => {
			if (!this.dim || this.locked || e.target !== this.dialog) return;
			const box = this.dialog.getBoundingClientRect();
			const out = e.clientX < box.left || e.clientX > box.right || e.clientY < box.top || e.clientY > box.bottom;
			if (out) this.dismiss();
		});
		this.leaving = null; // the leave under way: its end's timer and listener
		this.append(this.dialog);
	}

	update() {
		this.head.textContent = this.heading ?? "";
		this.head.hidden = !this.heading;
		this.dialog.dataset.placement = this.placement ?? "center";
		this.closer.hidden = this.locked;
		if (this.locked) this.dialog.setAttribute("closedby", "none");
		else this.dialog.removeAttribute("closedby");
		this.dialog.setAttribute("aria-label", this.heading || this.label || "");
		this.primaryButton.hidden = !this.primary;
		this.primaryButton.label = this.primary ?? "";
		this.primaryButton.state = this.primaryState ?? "normal";
		this.secondaryButton.hidden = !this.secondary;
		this.secondaryButton.label = this.secondary ?? "";
	}

	/** The content's container, for a caller that fills it after making the window. */
	get content() {
		this.ensure();
		return this.body;
	}

	/** What stays pinned at the window's foot under its buttons, for a caller that fills it (none shows while empty). */
	get foot() {
		this.ensure();
		return this.footEl;
	}

	/** Opens it modally, growing out of the tile `origin` returns where one is given (a function, asked again as the
	 * player closes it, since a Book that came meanwhile may have drawn its grid again; null where that tile is gone);
	 * the keys' half-second guard starts. A window still leaving stays, entering again. */
	open(origin = null) {
		if (!this.isConnected) document.body.append(this);
		this.update();
		const stays = this.endLeave();
		if (this.dialog.open) {
			if (stays) guard();
			return;
		}
		this.origin = origin;
		grow(this.dialog, origin?.() ?? null, () => {
			this.giveBack = holdFocus();
			this.dialog.showModal();
		});
		guard();
	}

	/** Ends a leave under way without closing (its timer, its listener and its shrink gone); true when there was one. */
	endLeave() {
		const leave = this.leaving;
		if (!leave) return false;
		this.leaving = null;
		clearTimeout(leave.timer);
		this.dialog.removeEventListener("animationend", leave.done);
		settle(this.dialog);
		this.dialog.classList.remove("qp-leaving");
		return true;
	}

	/** Open and not leaving yet: a close would start its leave. */
	closable() {
		return this.dialog.open && !this.leaving;
	}

	/** The player's own close (the ×, Escape and the dim): fires qp-dismiss, then closes, back into its tile. */
	dismiss() {
		if (!this.closable()) return;
		emit(this, "qp-dismiss");
		this.close({back: true});
	}

	/** Closes it with its leaving motion, or with `back` shrinking into the tile it grew from where that is half in view,
	 * gives the focus back and fires qp-closed. */
	close({back = false} = {}) {
		if (!this.closable()) return;
		const leave = {};
		leave.done = () => {
			// a later open() kept the window: this leave is no longer the window's
			if (this.leaving !== leave) return;
			this.endLeave();
			this.dialog.close();
			this.dialog.classList.remove(GROWN);
			this.giveBack?.();
			emit(this, "qp-closed");
		};
		this.leaving = leave;
		if (back && shrink(this.dialog, this.origin?.() ?? null, leave.done)) return;
		this.dialog.classList.add("qp-leaving");
		const ms = animationMs(this.dialog);
		if (!ms) {
			leave.done();
			return;
		}
		this.dialog.addEventListener("animationend", leave.done, {once: true});
		leave.timer = setTimeout(leave.done, ms + 50);
	}
}

define("qp-window", QpWindow, {heading: String, label: String, placement: String, locked: Boolean, primary: String,
	"primary-state": String, secondary: String, dim: Boolean});
