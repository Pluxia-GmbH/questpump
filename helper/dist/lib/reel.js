// The day reel on the computer (docs/plan.md slice 45; docs/ui.md, "The share card": the Reel; the kit's ReelPreview):
// the Share window's maker, its preview and its two buttons' commands. Rust makes the reel (helper/src-tauri/src/desk/
// reel: the films, the sound and the system's encoder) from what the page drew (lib/share.js `reelLayers`: the day's
// Saga card's layers and the knight's square), sent as one body, `bundle`'s: a 32-bit little-endian length, the head's
// JSON, then each layer's PNG in the head's order. `ReelMaker` sends it (`reel_make`, which answers the reel's id once
// its job started), follows the `reel` topic until its reel is made, failed or was cancelled, and makes it again where
// Save video or Copy video finds none made (`made`); a later reel or its window's close (`close`, `reel_cancel`) stops
// the one being made, which leaves no file. A made reel is kept for its order's `key` while another format is chosen
// (`cancel`), so Reel chosen again with the same day, plan, style and Sound shows it at once, as Android's maker does
// (2026-10-19, the program's double-check); a new order, the window's close or Rust's next reel lets it go, and its
// file stays for the clipboard until Rust's sweep. `reelPreview` is the ReelPreview of the made file, following Reduce
// Motion as it changes; `saveVideo` and `copyVideo` hand it to Rust's save dialog and to the clipboard.

import {call, callBytes, get, subscribe} from "./bridge.js";
import {t} from "./copy.js";
import {h} from "../kit/element.js";

/** A layer's canvas as its PNG's bytes. */
function png(canvas) {
	return new Promise((resolve, reject) => canvas.toBlob((blob) => (blob ? resolve(blob) : reject(new Error("no PNG"))),
		"image/png")).then((blob) => blob.arrayBuffer()).then((buffer) => new Uint8Array(buffer));
}

/** The reel's order as Rust reads it (`desk::reel` `read`): the head ({plan, sound, name, knight}, with each layer's
 * place, frames, depth and PNG's length) and the layers' PNGs after it. */
export async function bundle(head, layers) {
	// each canvas encoded as the others are (the encoders run beside the page)
	const pngs = await Promise.all(layers.map((layer) => png(layer.canvas)));
	const json = new TextEncoder().encode(JSON.stringify({...head, layers: layers.map((l, i) => ({x: l.x, y: l.y,
		from: l.from, to: l.to, over: l.over, bytes: pngs[i].length}))}));
	const body = new Uint8Array(4 + json.length + pngs.reduce((a, p) => a + p.length, 0));
	new DataView(body.buffer).setUint32(0, json.length, true);
	body.set(json, 4);
	let at = 4 + json.length;
	for (const p of pngs) {
		body.set(p, at);
		at += p.length;
	}
	return body;
}

/** A reel that could not be made (the window says `share.reel.failed` or `share.reel.unsupported`), or that a later
 * reel or a cancel took the place of. */
export class ReelGone extends Error {}

/** The topic's states as the window shows them. */
const PHASES = new Set(["making", "ready", "failed", "unsupported", "cancelled"]);

/** One Share window's reels: `phase` idle (none asked yet), making, ready (`src` its file's asset URL), failed,
 * unsupported (no H.264 encoder here) or cancelled (a pump starting, the felt-unwell lock as it starts); `onChange`
 * hears each change. `kept` the last reel made ({key, id, src}), while Rust still shows it made. */
export class ReelMaker {
	constructor(onChange) {
		this.onChange = onChange;
		this.phase = "idle";
		this.src = null;
		this.id = null;
		this.order = null;
		this.kept = null;
		this.round = 0;
		this.waiting = [];
		this.topic = get("reel") ?? null;
		this.stop = subscribe("reel", (value) => {
			this.topic = value;
			this.follow();
		});
	}

	/** Makes the reel of `order` ({key, plan, sound, name, draw}: what the reel shows in one string, rule 18's plan of the
	 * day, the Sound choice, the file's name, and `draw(timing)`, the layers and the knight in the topic's timing): the
	 * one being made goes, and the reel made last of the same key, while Rust still shows it made, comes back at once. */
	async make(order) {
		const round = this.drop();
		this.order = order;
		const kept = this.kept;
		if (kept?.key === order.key && this.topic?.id === kept.id && this.topic?.state === "ready") {
			this.id = kept.id;
			this.src = kept.src;
			this.set("ready");
			return;
		}
		this.kept = null;
		this.set("making");
		try {
			const timing = this.topic?.timing;
			if (!timing) throw new Error("the reel's timing did not come");
			const {knight, layers} = await order.draw(timing);
			if (round !== this.round) return;
			const body = await bundle({plan: order.plan, sound: order.sound, name: order.name, knight}, layers);
			if (round !== this.round) return;
			const id = await callBytes("reel_make", body);
			if (round !== this.round) {
				// a later reel or the window's close came while Rust started this one: it goes
				call("reel_cancel", {id}).catch(console.error);
				return;
			}
			this.id = id;
			this.follow();
		} catch (e) {
			console.error(e);
			if (round === this.round) this.set("failed");
		}
	}

	/** Takes the topic's state of this window's reel. */
	follow() {
		const value = this.topic;
		if (this.id === null || value?.id !== this.id || !PHASES.has(value.state)) return;
		this.src = value.state === "ready" ? value.src : null;
		if (value.state === "ready" && this.order) this.kept = {key: this.order.key, id: this.id, src: this.src};
		this.set(value.state);
	}

	set(phase) {
		const changed = phase !== this.phase;
		this.phase = phase;
		if (phase !== "making") {
			const waiting = this.waiting.splice(0);
			for (const w of waiting) {
				if (phase === "ready" && this.id !== null) w.resolve(this.id);
				else w.reject(new ReelGone(phase));
			}
		}
		if (changed) this.onChange(phase);
	}

	/** Stops waiting for the reel asked last (its job, once it has an id, is Rust's to cancel); the next round. */
	drop() {
		this.round += 1;
		if (this.phase === "making" && this.id !== null) call("reel_cancel", {id: this.id}).catch(console.error);
		this.id = null;
		this.src = null;
		for (const w of this.waiting.splice(0)) w.reject(new ReelGone("replaced"));
		return this.round;
	}

	/** The reel no longer asked for (another format chosen): the one being made stops, the one made is kept. */
	cancel() {
		this.drop();
		this.order = null;
		this.set("idle");
	}

	/** The window closed: the reel being made stops and leaves no file, the one made is let go, and the topic is heard
	 * no more. */
	close() {
		this.kept = null;
		this.cancel();
		this.stop();
	}

	/** The made reel's id: at once where it is made, else once the one being made lands, else once the last order is
	 * made again; a reel that fails or goes rejects with `ReelGone`. */
	made() {
		if (this.phase === "ready" && this.id !== null) return Promise.resolve(this.id);
		if (this.phase !== "making") {
			if (!this.order) return Promise.reject(new ReelGone(this.phase));
			// its first steps run now: it is making before the wait below begins
			this.make(this.order);
		}
		return new Promise((resolve, reject) => this.waiting.push({resolve, reject}));
	}
}

/** The ReelPreview (docs/ui.md, the kit's ReelPreview): the made file `src` in the card's place, muted and looping from
 * its first frame; a press anywhere on it pauses it and plays it again, its round mark at the bottom-trailing corner
 * showing the pause mark while it plays and the play mark while it is held; read as "Video, 10 seconds" (`seconds`, the
 * reel's length rounded) with "Pause the reel" or "Play the reel" as its action; under Reduce Motion it stands on the
 * reel's last frame (`fps`, the reel's rate) until pressed, a press plays it once from its start, and it stands on its
 * last frame again at its end (Android's ReelPreview alike). Reduce Motion turned on while it shows stands it on its
 * last frame, turned off loops it again from its start (2026-10-19, as the phones follow the setting). */
export function reelPreview(src, seconds, fps) {
	const motion = matchMedia("(prefers-reduced-motion: reduce)");
	const video = h("video", {class: "qp-reel-video", preload: "auto", "aria-hidden": "true"});
	video.muted = true;
	video.loop = !motion.matches;
	video.playsInline = true;
	// under Reduce Motion: whether it stands on its last frame, from which a press plays it from its start
	let standing = false;
	const glyph = h("qp-art", {name: "mark_pause"});
	const el = h("button", {type: "button", class: "qp-reel-preview", "aria-label": t("share.reel.spoken", {seconds})},
		video, h("span", {class: "qp-reel-mark", "aria-hidden": "true"}, glyph));
	const paint = () => {
		const playing = !video.paused;
		glyph.setAttribute("name", playing ? "mark_pause" : "mark_play");
		el.setAttribute("aria-description", t(playing ? "share.reel.pause" : "share.reel.play"));
	};
	video.addEventListener("play", paint);
	video.addEventListener("pause", paint);
	el.addEventListener("click", () => {
		if (!video.paused) {
			video.pause();
			return;
		}
		if (standing) video.currentTime = 0;
		standing = false;
		video.play().catch(console.error);
	});
	// its last frame: a frame before its end at the reel's rate, once its length is known (as it loads and after each
	// play to its end under Reduce Motion)
	const stand = () => {
		if (!Number.isFinite(video.duration)) return;
		standing = true;
		video.currentTime = Math.max(0, video.duration - 1 / fps);
	};
	video.addEventListener("loadedmetadata", () => motion.matches && stand(), {once: true});
	video.addEventListener("ended", () => motion.matches && stand());
	video.autoplay = !motion.matches;
	video.addEventListener("canplay", () => !motion.matches && video.play().catch(console.error), {once: true});
	// the setting as it changes, until the preview leaves the page (a new reel's, the window's close)
	const follow = () => {
		if (!el.isConnected) {
			motion.removeEventListener("change", follow);
			return;
		}
		video.loop = !motion.matches;
		video.autoplay = !motion.matches;
		if (motion.matches) {
			video.pause();
			stand();
			return;
		}
		if (standing) video.currentTime = 0;
		standing = false;
		video.play().catch(console.error);
	};
	motion.addEventListener("change", follow);
	video.src = src;
	paint();
	return el;
}

/** Save video: the made reel `id` copied where the player picks in the system's save dialog; true once saved, false
 * when the player cancelled. */
export function saveVideo(id) {
	return call("save_video", {id});
}

/** Copy video: the made reel `id` on the clipboard as a file, to paste into a chat. */
export function copyVideo(id) {
	return call("copy_video", {id});
}
