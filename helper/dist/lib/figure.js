// The move figures in the desktop app (docs/ui.md, "The pump on the computer": the figures; the phones' Figures.swift,
// Kit/Figure.swift and Kit/FigureLoop.swift, drawn once more): the armour figure from the `figures` topic
// (helper/src-tauri/src/desk/figures.rs: the phones' contract/figures/figures.json and where its files are). A move's
// film (H.264, stacked: the base in its top square, the tint in the square from row `film.tint`) plays muted in a
// <video crossorigin> drawn to a canvas each frame, the base and then the tint times the figure's colour added
// (player.js `addTint`, the game's ADD tint, as Figure.lua and the site's hero draw theirs), starting on its key pose
// and looping; its key-pose still stands in until the film shows that frame, and in its place under Reduce Motion
// (prefers-reduced-motion, which follows macOS's Reduce motion and Windows' Animation effects), off screen, with the
// window hidden and wherever the film does not play (not yet downloaded, a broken file). A still in a colour other than
// the crimson it is drawn in is recoloured once per colour (player.js `recolour`) into a kept ImageBitmap. The other
// side of a side move is the figure mirrored; the edges of the square that cut the film's floor light fade out over the
// FigureStage's fadeFraction, and the square is drawn `lighten` over what lies below (the plinth's glow on the
// stage's ground), so no box shows. A move the table lacks (a newer game's library) shows the move's glyph instead.
//
// `MoveFigure` is one figure, filling its square (qp-figure-stage's): `new MoveFigure()`, its element `el`, then
// `show({move, side, colour, still, calm, paused, beat, from})` as often as the moment changes (side "other" mirrors;
// colour "#RRGGBB", the crimson without one; still: the key pose only; calm: the loop at a stroll; beat and from: a set
// of reps' beat in ms and its go in unix ms, both Rust's, for Big view's figure phased to the go) and `stop()` when it
// goes.
// `figureOf(move)` gives a move's numbers, `drawnBox(move, side)` its figure's box as drawn, `tileStill(move)` the
// MoveTile's crop of its crimson still, and `pairOf`, `PairStill` the right-versus-wrong pair (qp-move-figure-pair);
// each takes a pump move's `figureName` too, the move drawn with the implement that carries its load (the table's
// `gear`).
//
// Phased to the go (the rep count, 2026-10-09): the film is one rep from its start pose back to it, so it plays at the
// rate that makes it last one beat and shows ((now - from) mod beat) of it: through the countdown its frame 0 comes
// with the go (no jump at the go), and the voice's count n (Rust's, at the loop's n-th rep end after the go) comes as
// the figure finishes rep n. Each frame it is put back in phase where it drifted more than `DRIFT` (a loop's wrap, a
// stall), aimed ahead by what the last seek took to land (`seekCost`: a seek decodes from the film's one key frame),
// and so it is as it plays again after anything stopped it (off screen, the window hidden), never starting over.
// Under Reduce Motion it stands on its key pose as every figure does; the count runs on the clock all the same.

import {get, subscribe} from "./bridge.js";
import {addTint, recolour, shares} from "./player.js";

/** The calm loop's rate (Kit/FigureLoop.swift `FigureMotion.calmRate`). */
const CALM_RATE = 0.6;
/** The recoloured stills and pairs kept (about 0.9 MB each at 480 px). */
const KEPT = 24;
/** How far a phased film may drift from the go's clock before it is put back (seconds: about two frames at 30 fps,
 * under what the eye takes for out of time with the voice). */
const DRIFT = 0.06;
/** The most a phase seek aims ahead of the clock for its own decoding (seconds). */
const SEEK_LEAD_MAX = 0.5;

let table = get("figures") ?? null;
const live = new Set();
subscribe("figures", (value) => {
	table = value;
	for (const figure of live) figure.refresh();
});

const motion = window.matchMedia("(prefers-reduced-motion: reduce)");
motion.addEventListener("change", () => {
	for (const figure of live) figure.refresh();
});
document.addEventListener("visibilitychange", () => {
	for (const figure of live) figure.refresh();
});

/** The FigureStage token's fade: the share of the side a cut edge fades over (read as a figure is made and as it
 * resizes, never per frame: a computed style may recalculate the page's). */
function fadeFraction() {
	return parseFloat(getComputedStyle(document.documentElement).getPropertyValue(
		"--qp-surface-figure-stage-fade-fraction")) || 0;
}

/** A file's URL under a folder's asset prefix (figures.rs: convertFileSrc's spelling). */
function url(prefix, name) {
	return prefix ? prefix + encodeURIComponent(name) : null;
}

/** "#RRGGBB" as the [r, g, b] shares player.js takes. */
function colourOf(hex) {
	return shares(/^#[0-9A-Fa-f]{6}$/.test(hex ?? "") ? hex : table.crimson);
}

/** A move's figure: its entry and its owner's numbers, mirrored as the pack mirrors it (`mirror` true); null without.
 * `move` is a library move's key, or "<key>.<gear>" (`figureName`): the move drawn with the implement that gear key
 * names, its own where the table has no figure of that gear; or an extra's name (the standard-bearer
 * `streak_banner`, slice 51), its still alone (`extra`). */
export function figureOf(move) {
	const [key, gear] = (move ?? "").split(".");
	const own = table?.moves?.[key];
	const entry = (gear && own?.gear?.[gear]) || own;
	const owner = entry && table.owners[entry.owner];
	if (owner) return {...entry, box: owner.box, fade: owner.fade, tile: owner.tile};
	const extra = table?.extras?.[key];
	return extra ? {owner: key, key: 0, box: extra.box, fade: extra.fade, tile: extra.tile, extra: true} : null;
}

/** A pump move's figure name (`figureOf`): its key, and the gear whose implement carries its load (the engine's
 * `implement`, the player's equipment) where it has one, so a dumbbell row shows the dumbbell (2026-10-09). */
export function figureName(m) {
	return m.implement ? `${m.key}.${m.implement}` : m.key;
}

/** A move's tile (what MoveTile draws for its `move`, and a share card's): the square `tile` of its crimson still,
 * mirrored as the pack mirrors it. */
export function tileStill(move) {
	const figure = figureOf(move);
	const src = figure && url(table.stills, `${figure.owner}.png`);
	return src ? {src, crop: figure.tile, mirror: figure.mirror} : null;
}

/** An extra figure's still (figures.json `extras`: a figure that is no library move, the streak card's
 * standard-bearer `streak_banner`) recoloured in `hex` (the crimson without one), with its tile's square, never
 * mirrored; null where the table or its still lacks it. */
export function extraStill(name, hex) {
	const extra = table?.extras?.[name];
	if (!extra || !table.stills) return Promise.resolve(null);
	return stillIn(table.stills, name, hex).then((img) => ({img, crop: extra.tile, mirror: false}), () => null);
}

/** A box [left, top, right, bottom] drawn mirrored. */
function mirroredBox([l, t, r, b]) {
	return [1 - r, t, 1 - l, b];
}

/** Whether a figure is drawn mirrored on `side`: the pack's own mirror, turned again on a side move's other side. */
function mirrored(figure, side) {
	return figure.mirror !== (side === "other");
}

/** A move's figure box [left, top, right, bottom] in its square as it is drawn on `side` (the PumpStage's fit); null
 * without its numbers. */
export function drawnBox(move, side) {
	const figure = figureOf(move);
	if (!figure) return null;
	return mirrored(figure, side) ? mirroredBox(figure.box) : figure.box;
}

/** Cut edges [left, right, bottom] (null where the light fades by itself) drawn mirrored. */
function mirroredFade([l, r, b]) {
	return [r === null ? null : 1 - r, l === null ? null : 1 - l, b];
}

/** Fades the cut edges [left, right, bottom] of what `ctx` holds in the rect [x, y, w, h] (Kit/FigureLoop.swift
 * `FadeMask`): clear at the edge, whole a share `ff` of the side further in. */
function fadeEdges(ctx, [l, r, b], [x, y, w, h], ff) {
	ctx.save();
	ctx.globalAlpha = 1;
	ctx.globalCompositeOperation = "destination-in";
	const ramp = (x0, y0, x1, y1, edge) => {
		const g = ctx.createLinearGradient(x0, y0, x1, y1);
		g.addColorStop(0, "rgb(0 0 0 / 0)");
		g.addColorStop(Math.min(1, edge), "rgb(0 0 0 / 0)");
		g.addColorStop(Math.min(1, edge + ff), "rgb(0 0 0)");
		g.addColorStop(1, "rgb(0 0 0)");
		ctx.fillStyle = g;
		ctx.fillRect(0, 0, ctx.canvas.width, ctx.canvas.height);
	};
	if (l !== null) ramp(x, 0, x + w, 0, l);
	if (r !== null) ramp(x + w, 0, x, 0, 1 - r);
	if (b !== null) ramp(0, y + h, 0, y, 1 - b);
	ctx.restore();
}

/** An image loaded through the asset protocol, CORS-clean for the canvas. */
function load(src) {
	return new Promise((done, failed) => {
		const img = new Image();
		img.crossOrigin = "anonymous";
		img.decoding = "async";
		img.onload = () => done(img);
		img.onerror = () => failed(new Error(`${src} did not load`));
		img.src = src;
	});
}

/** The recoloured stills, by their URL and colour, most recent last. */
const stills = new Map();

/** A still (stills/<owner> or pairs/<owner>.key|fault, without .png, under `prefix`) over the card in `hex`: its tint
 * taken out in the crimson and put back in the colour (player.js `recolour`), opaque, once per colour, kept. */
function stillIn(prefix, name, hex) {
	const src = url(prefix, `${name}.png`);
	if (!src) return Promise.reject(new Error(`no folder for ${name}`));
	const key = `${src}#${hex}`;
	const held = stills.get(key);
	if (held) {
		stills.delete(key);
		stills.set(key, held);
		return held;
	}
	const made = Promise.all([load(src), load(url(prefix, `${name}.t.png`))]).then(([still, tint]) => {
		const [w, h] = [still.naturalWidth, still.naturalHeight];
		const canvas = document.createElement("canvas");
		[canvas.width, canvas.height] = [w, h];
		const ctx = canvas.getContext("2d", {willReadFrequently: true});
		ctx.drawImage(tint, 0, 0, w, h);
		const light = ctx.getImageData(0, 0, w, h);
		ctx.clearRect(0, 0, w, h);
		ctx.drawImage(still, 0, 0, w, h);
		const pixels = recolour(ctx.getImageData(0, 0, w, h), light, shares(table.card), shares(table.crimson),
			colourOf(hex));
		return createImageBitmap(pixels);
	});
	made.catch(() => stills.delete(key));
	stills.set(key, made);
	while (stills.size > KEPT) stills.delete(stills.keys().next().value);
	return made;
}

/** A canvas's backing store at its CSS size in the screen's pixels; false while it has no size. */
function fit(canvas) {
	const scale = window.devicePixelRatio || 1;
	const w = Math.round(canvas.clientWidth * scale), h = Math.round(canvas.clientHeight * scale);
	if (!w || !h) return false;
	if (canvas.width !== w || canvas.height !== h) [canvas.width, canvas.height] = [w, h];
	return true;
}

/** One move's figure on a canvas (the module header). */
export class MoveFigure {
	constructor() {
		this.canvas = document.createElement("canvas");
		this.canvas.className = "qp-figure-canvas";
		this.canvas.setAttribute("aria-hidden", "true");
		this.glyph = document.createElement("qp-art");
		this.glyph.setAttribute("name", "action_pump");
		this.glyph.className = "qp-figure-glyph";
		this.el = document.createElement("div");
		this.el.className = "qp-figure";
		this.el.append(this.canvas, this.glyph);
		this.ctx = this.canvas.getContext("2d");
		// the tint's scratch square, sized as each film loads
		this.off = document.createElement("canvas");
		this.wanted = {};
		this.shown = null;
		this.inView = true;
		this.fraction = fadeFraction();
		this.resize = new ResizeObserver(() => {
			this.fraction = fadeFraction();
			this.paint();
		});
		this.view = new IntersectionObserver((seen) => {
			this.inView = seen.some((s) => s.isIntersecting);
			this.refresh();
		});
	}

	/** Shows a move (the module header); the same options again change nothing. */
	show(options) {
		this.wanted = {...options};
		if (!live.has(this)) {
			live.add(this);
			this.resize.observe(this.canvas);
			this.view.observe(this.el);
		}
		this.refresh();
	}

	/** Plays the film's rep once from its start and back to its key pose (a standing loop's tap under Reduce Motion);
	 * nothing without a film. */
	playOnce() {
		if (!this.standing() || !this.video || !this.ready) return;
		this.once = true;
		this.video.currentTime = 0;
		this.lastTime = 0;
		this.refresh();
	}

	/** Lets the figure go: its film stops and frees its decoder. */
	stop() {
		live.delete(this);
		this.resize.disconnect();
		this.view.disconnect();
		this.dropFilm();
		this.shown = null;
	}

	/** The loop stands on its key pose: Reduce Motion, unless a tap plays its rep. */
	standing() {
		return motion.matches && !this.once;
	}

	/** Brings what shows in line with what is wanted and what the table has: the film (kept while it may play, paused
	 * where it is off screen, with the window hidden or paused by the player, held on its key pose while it stands),
	 * else the still. */
	refresh() {
		const w = this.wanted;
		const figure = live.has(this) && table ? figureOf(w.move) : null;
		this.el.toggleAttribute("data-none", !figure && live.has(this) && Boolean(table));
		if (!figure) {
			this.dropFilm();
			this.shown = null;
			this.clear();
			return;
		}
		const mirror = mirrored(figure, w.side);
		const hex = /^#[0-9A-Fa-f]{6}$/.test(w.colour ?? "") ? w.colour.toUpperCase() : table.crimson;
		this.shown = {figure, mirror, hex, fade: mirror ? mirroredFade(figure.fade) : figure.fade};
		this.loadStill(figure.owner, hex);
		const film = w.still || figure.extra ? null : url(table.films, `${figure.owner}.mp4`);
		if (film) this.film(film, figure.key);
		else this.dropFilm();
		const video = this.video;
		if (video && this.ready) {
			const phased = this.phaseAt(Date.now()) !== null;
			// a phased film lasts one beat (its own length, but for the beat's whole milliseconds)
			video.playbackRate = phased ? video.duration / (w.beat / 1000) : w.calm ? CALM_RATE : 1;
			const plays = !this.standing() && !w.paused && this.inView && document.visibilityState === "visible";
			// back in view, the window shown again, a new go: in phase with the clock before it plays on
			if (plays) this.phase();
			if (plays && video.paused) video.play().catch((e) => console.error("figure:", e));
			else if (!plays && !video.paused) video.pause();
			if (this.standing() && Math.abs(video.currentTime - figure.key) > 0.01) video.currentTime = figure.key;
		}
		this.paint();
	}

	/** The key-pose still in the colour, recoloured once and kept; drawn while no film frame shows. */
	loadStill(owner, hex) {
		const key = `${owner}#${hex}`;
		if (this.stillKey === key) return;
		this.stillKey = key;
		this.still = null;
		stillIn(table.stills, owner, hex).then((bitmap) => {
			if (this.stillKey !== key) return;
			this.still = bitmap;
			this.paint();
		}, (e) => console.error("figure:", e));
	}

	/** Where a phased film stands at `now` (seconds into it): the share of the beat since the last one counted from the
	 * go, the countdown's before it included; null unless a beat and a go are wanted and the film knows its length. */
	phaseAt(now) {
		const {beat, from} = this.wanted;
		const duration = this.video?.duration;
		if (!(beat > 0) || typeof from !== "number" || !(duration > 0)) return null;
		return ((((now - from) % beat) + beat) % beat) / beat * duration;
	}

	/** Puts a phased film back in phase where it is more than `DRIFT` from the clock, either way round the loop's wrap,
	 * aimed where the clock will be once the seek lands (`seekCost`, the last seek's); nothing while it seeks or is
	 * not phased. */
	phase() {
		const video = this.video;
		const now = Date.now();
		const at = this.phaseAt(now);
		if (at === null || video.seeking) return;
		const length = video.duration;
		const off = (((video.currentTime - at) % length) + length * 1.5) % length - length / 2;
		if (Math.abs(off) > DRIFT) this.seek(this.phaseAt(now + this.seekCost * 1000));
	}

	/** Seeks the film to `at` seconds, timed until it lands (`seekCost`). */
	seek(at) {
		this.seekStart = performance.now();
		this.video.currentTime = at;
	}

	/** The film for `src`, started on its key pose (`key` seconds in), or in phase with the go where it is phased. */
	film(src, key) {
		if (this.video?.dataset.src === src) return;
		this.dropFilm();
		const video = document.createElement("video");
		video.className = "qp-figure-video";
		video.dataset.src = src;
		video.crossOrigin = "anonymous";
		video.muted = true;
		video.loop = true;
		video.playsInline = true;
		video.preload = "auto";
		video.setAttribute("aria-hidden", "true");
		[this.off.width, this.off.height] = [table.film.side, table.film.side];
		video.addEventListener("loadedmetadata", () => {
			const phased = this.phaseAt(Date.now() + this.seekCost * 1000);
			this.seek(phased ?? Math.min(key, Math.max(0, video.duration - 0.05)));
		}, {once: true});
		video.addEventListener("seeked", () => {
			if (this.video !== video) return;
			if (this.seekStart !== null) {
				this.seekCost = Math.min(SEEK_LEAD_MAX, (performance.now() - this.seekStart) / 1000);
				this.seekStart = null;
			}
			if (this.ready) return;
			this.ready = true;
			this.frames();
			this.refresh();
		});
		video.addEventListener("timeupdate", () => {
			// a tap's rep under Reduce Motion ends where the looping film wraps to its start
			if (this.once && video.currentTime < this.lastTime) this.endOnce();
			this.lastTime = video.currentTime;
		});
		video.addEventListener("error", () => {
			console.error("figure:", src, video.error?.message);
			this.dropFilm();
			this.paint();
		}, {once: true});
		this.video = video;
		this.ready = false;
		this.framed = false;
		this.lastTime = 0;
		this.seekStart = null;
		this.seekCost = 0;
		this.el.append(video);
		video.src = src;
	}

	/** A tap's one rep is over: back to the key pose. */
	endOnce() {
		if (!this.once) return;
		this.once = false;
		const key = this.shown?.figure.key ?? 0;
		if (this.video) {
			this.video.pause();
			this.video.currentTime = key;
		}
		this.refresh();
	}

	/** Draws each new film frame while the film plays, a phased one kept in phase with the go's clock, from the first
	 * frame the film presents (`framed`): until then WebKit draws nothing of a film the asset protocol serves, even
	 * after `seeked`, and the tint's multiply turned that empty frame into a crimson square (seen 2026-10-23). */
	frames() {
		const video = this.video;
		if (!video) return;
		const next = () => {
			if (this.video !== video) return;
			this.framed = true;
			if (!video.paused) this.phase();
			this.paint();
			if (video.requestVideoFrameCallback) video.requestVideoFrameCallback(next);
			else requestAnimationFrame(next);
		};
		if (video.requestVideoFrameCallback) video.requestVideoFrameCallback(next);
		else next();
	}

	dropFilm() {
		const video = this.video;
		this.video = null;
		this.ready = false;
		this.framed = false;
		this.filmSize = null;
		if (!video) return;
		video.pause();
		video.removeAttribute("src");
		video.load();
		video.remove();
	}

	clear() {
		this.ctx.setTransform(1, 0, 0, 1, 0, 0);
		this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);
	}

	/** Draws the film's frame where it shows its key pose or plays, else the still, then fades the cut edges. */
	paint() {
		const shown = this.shown;
		if (!shown || !fit(this.canvas)) return;
		const ctx = this.ctx, W = this.canvas.width, H = this.canvas.height;
		const filmShows = this.video && this.framed && this.video.readyState >= 2;
		// mid-seek (a re-phase past DRIFT, the loop's wrap) the film's last frame stays on the canvas: the key-pose
		// still drawn for that one paint flashed the move's key pose, a push-up's lowest point (Rapha, 2026-10-26)
		if (!filmShows && this.filmSize === `${W}x${H}`) return;
		if (!filmShows && !this.still) return;
		this.clear();
		ctx.save();
		if (shown.mirror) ctx.setTransform(-1, 0, 0, 1, W, 0);
		ctx.globalAlpha = 1;
		ctx.globalCompositeOperation = "source-over";
		if (filmShows) {
			const {side, tint} = table.film;
			ctx.drawImage(this.video, 0, 0, side, side, 0, 0, W, H);
			addTint(ctx, this.off, this.video, [0, tint, side, side], [0, 0, W, H], colourOf(shown.hex));
			this.filmSize = `${W}x${H}`;
		} else {
			ctx.drawImage(this.still, 0, 0, W, H);
			this.filmSize = null;
		}
		ctx.restore();
		fadeEdges(ctx, shown.fade, [0, 0, W, H], this.fraction);
	}
}

/** A move's right-versus-wrong pair (`FigureLibrary.pair`): its owner's pair stills' name, whether it is drawn
 * mirrored and the fault's words; null without a fault, its words or the table. */
export function pairOf(move, side) {
	const figure = figureOf(move);
	const words = figure?.fault && table.faults?.[figure.fault];
	if (!words || !table.pairs) return null;
	const mirror = mirrored(figure, side);
	// a pair's cell is the loop's square without its margins: its own edges are the ones the loop's numbers name
	const [l, r, b] = figure.fade;
	const fade = [l === null ? null : 0, r === null ? null : 1, b === null ? null : 1];
	return {owner: figure.owner, mirror, fault: words, fade: mirror ? mirroredFade(fade) : fade};
}

/** One still of a pair (`which`: key or fault) on a canvas in the colour, faded at its cut edges, drawn lighten: the
 * canvas fills its half's square and the still is drawn inside it at its own shape, centred (a push-up's wide still
 * drawn to the square's whole size looked squashed, Rapha, 2026-10-26). */
export class PairStill {
	constructor() {
		this.canvas = document.createElement("canvas");
		this.canvas.className = "qp-pair-canvas";
		this.canvas.setAttribute("aria-hidden", "true");
		this.ctx = this.canvas.getContext("2d");
		this.fraction = fadeFraction();
		this.resize = new ResizeObserver(() => {
			this.fraction = fadeFraction();
			this.paint();
		});
		this.resize.observe(this.canvas);
	}

	show(pair, which, hex) {
		const colour = /^#[0-9A-Fa-f]{6}$/.test(hex ?? "") ? hex.toUpperCase() : table.crimson;
		const key = `${pair.owner}.${which}#${colour}`;
		this.pair = pair;
		if (this.key === key) return this.paint();
		this.key = key;
		this.bitmap = null;
		stillIn(table.pairs, `${pair.owner}.${which}`, colour).then((bitmap) => {
			if (this.key !== key) return;
			this.bitmap = bitmap;
			this.paint();
		}, (e) => console.error("figure:", e));
		return undefined;
	}

	paint() {
		if (!this.bitmap || !fit(this.canvas)) return;
		const ctx = this.ctx, W = this.canvas.width, H = this.canvas.height;
		ctx.setTransform(1, 0, 0, 1, 0, 0);
		ctx.clearRect(0, 0, W, H);
		ctx.save();
		if (this.pair.mirror) ctx.setTransform(-1, 0, 0, 1, W, 0);
		const scale = Math.min(W / this.bitmap.width, H / this.bitmap.height);
		const w = this.bitmap.width * scale, h = this.bitmap.height * scale;
		const at = [(W - w) / 2, (H - h) / 2, w, h];
		ctx.drawImage(this.bitmap, ...at);
		ctx.restore();
		fadeEdges(ctx, this.pair.fade, at, this.fraction);
	}

	stop() {
		this.resize.disconnect();
	}
}
