// The move figure's player on a canvas: the JavaScript twin of Figure.lua's core (Resolve, NewState and Step line for
// line, as tools/figures/steps.py is the Python one, so it steps to contract/fixtures/figure-steps.json, and
// CellCoords' cell in pixels) and of View:Apply's four layers: base A at the cell, tint A over it, and during a
// cross-fade base B at the next cell at alpha a with tint B at a and tint A at 1 - a; each tint is the tint sheet's
// cell times the colour, added (the game's ADD blend, `addTint`: `multiply` by the colour, then `lighter`), and a
// mirrored move draws its cells flipped; and of the phones' still in a colour (`recolour`, FigureLibrary.recolour):
// over the card, the still's own tint taken out in the crimson and put back in the colour. One module for three pages
// (docs/plan.md slice 35): the review page (tools/figures/review.py writes this file's text into its own module script,
// since a page opened from disk cannot import one), questpump.com's hero (`publish.py --site <move>` copies it to
// site/public/figure.js, the move's two sheets and its pack entry to site/public/figure/) and the desktop app, whose
// films and stills (helper/dist/lib/figure.js) take their colour through `addTint` and `recolour` from the copy
// `publish.py` keeps at helper/dist/lib/player.js.
//
// On a page that loads it, it plays every <canvas data-figure="<the move's JSON>">: the JSON is the pack's shape for
// one move (hz, box, bg, moves) with `move`, the key it shows, and `colour`, the tint's #RRGGBB, its sheet names
// relative to it. A <button aria-controls="<the canvas id>"> pauses and resumes it (aria-pressed true while paused;
// WCAG 2.2.2, as a click on the game's figure does), and so does a click on the canvas; under prefers-reduced-motion
// the key pose holds still and the button hides. The canvas keeps its CSS size and draws at the screen's pixels.

/** The move's entry and the entry whose sheets it draws on (a variant names its reference move in `ref`). */
export function resolve(fig, key) {
	const m = fig.moves[key];
	if (!m) return [null, null];
	return [m, m.ref ? fig.moves[m.ref] : m];
}

/** Cell c's rectangle in its sheet, in pixels: [x, y, w, h] (0-based cells, row-major). */
export function cellRect(base, c) {
	const cols = base.cols;
	const col = c % cols;
	const row = Math.floor(c / cols);
	const x = col * base.cell[0] + base.pad;
	const y = row * base.cell[1] + base.pad;
	return [x, y, base.size[0], base.size[1]];
}

export function newState() {
	return { t: 0, run: 1, runStart: 0 };
}

/**
 * Advances by `elapsed` seconds; returns [the cell, the cell blended over it or null, the blend's alpha]. Runs are
 * 1-based as in Lua, and the loop wraps with an explicit floor, so every player computes the same doubles.
 */
export function step(st, m, hz, elapsed) {
	const seq = m.seq;
	const nruns = seq.length / 2;
	const loop = m.ticks / hz;
	let t = st.t + elapsed;
	if (t >= loop) {
		t = t - Math.floor(t / loop) * loop;
		st.run = 1;
		st.runStart = 0;
	}
	st.t = t;
	const tick = t * hz;
	while (st.run < nruns && tick >= st.runStart + seq[2 * st.run - 1]) {
		st.runStart = st.runStart + seq[2 * st.run - 1];
		st.run = st.run + 1;
	}
	const cell = seq[2 * st.run - 2];
	if (m.fade.includes(st.run)) {
		const nxt = (st.run % nruns) + 1;
		return [cell, seq[2 * nxt - 2], (tick - st.runStart) / seq[2 * st.run - 1]];
	}
	return [cell, null, 0];
}

/** "#RRGGBB" as the [r, g, b] shares Figure.lua's SetVertexColor takes. */
export function shares(hex) {
	return [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16) / 255);
}

function rgb(v) {
	return `rgb(${v.map((c) => Math.round(c * 255)).join(",")})`;
}

/**
 * Adds the part [sx, sy, sw, sh] of the tint image `img` times `colour` (its [r, g, b] shares) over what `ctx` holds
 * at [dx, dy, dw, dh], at `alpha`: the game's ADD blend of a tint (Figure.lua), the part drawn on the scratch canvas
 * `off` (at least sw x sh), `multiply`-ed by the colour there and drawn `lighter`, so each channel is clamp(below +
 * tint x colour) on the stored values.
 */
export function addTint(ctx, off, img, [sx, sy, sw, sh], [dx, dy, dw, dh], colour, alpha = 1) {
	const o = off.getContext("2d");
	o.globalCompositeOperation = "source-over";
	o.clearRect(0, 0, sw, sh);
	o.drawImage(img, sx, sy, sw, sh, 0, 0, sw, sh);
	o.globalCompositeOperation = "multiply";
	o.fillStyle = rgb(colour);
	o.fillRect(0, 0, sw, sh);
	ctx.globalAlpha = alpha;
	ctx.globalCompositeOperation = "lighter";
	ctx.drawImage(off, 0, 0, sw, sh, dx, dy, dw, dh);
}

/**
 * A still in a colour (the phones' FigureLibrary.recolour): `still` drawn over the card `card`, its tint `tint` (the
 * cloth's light on black, ImageData of the same size) taken out in `from` and put back in `to`, per channel
 * clamp(S - tint x from + tint x to) with every colour's [r, g, b] shares, opaque; the still's own ImageData changes in
 * place and comes back.
 */
export function recolour(still, tint, card, from, to) {
	const s = still.data, t = tint.data;
	const bg = card.map((c) => c * 255);
	const shift = to.map((c, i) => c - from[i]);
	for (let i = 0; i < s.length; i += 4) {
		const a = s[i + 3] / 255;
		for (let c = 0; c < 3; c++) {
			const over = s[i + c] * a + bg[c] * (1 - a);
			s[i + c] = Math.min(255, Math.max(0, Math.round(over + t[i + c] * shift[c])));
		}
		s[i + 3] = 255;
	}
	return still;
}

/**
 * One move on one canvas: Figure.lua's view in "loop" mode. `fig` is the pack (or its one move's part), `sheet` and
 * `tint` the move's two sheet images, `colour` the tint's [r, g, b] shares and `flip` the other side of a side move.
 */
export class Figure {
	constructor(canvas, fig, key, sheet, tint, colour, flip = false) {
		[this.m, this.base] = resolve(fig, key);
		if (!this.m) throw new Error(`no figure for ${key}`);
		this.canvas = canvas;
		this.ctx = canvas.getContext("2d");
		this.hz = fig.hz;
		this.box = fig.box;
		this.card = `rgb(${fig.bg.join(",")})`;
		this.sheet = sheet;
		this.tint = tint;
		this.colour = colour;
		this.mirror = (this.m.mirror === true) !== flip;
		this.state = newState();
		this.paused = false; // a click's pause (the loop leaves it where it stopped)
		this.held = false; // reduced motion: the key pose, still
		this.shown = [this.m.seq[0], null, 0];
		this.off = document.createElement("canvas");
		[this.off.width, this.off.height] = this.base.size;
	}

	/** Both sheets have loaded. */
	get ready() {
		return this.sheet.complete && this.sheet.naturalWidth > 0 && this.tint.complete && this.tint.naturalWidth > 0;
	}

	/** One frame: steps by `elapsed` seconds and draws. */
	advance(elapsed) {
		this.shown = step(this.state, this.m, this.hz, elapsed);
		this.draw();
	}

	/** Holds one cell, still (View:Still): the key pose under reduced motion. */
	still(cell) {
		this.shown = [cell, null, 0];
		this.draw();
	}

	/** Draws the step last shown, at the canvas's size: the box fitted into it, centred. */
	draw() {
		if (!this.ready) return;
		const [cell, blend, alpha] = this.shown;
		const ctx = this.ctx, W = this.canvas.width, H = this.canvas.height;
		const k = Math.min(W, H) / this.box;
		ctx.save();
		ctx.globalAlpha = 1;
		ctx.globalCompositeOperation = "source-over";
		ctx.fillStyle = this.card;
		ctx.fillRect(0, 0, W, H);
		if (this.mirror) {
			ctx.translate(W, 0);
			ctx.scale(-1, 1);
		}
		this.layer(this.sheet, cell, 1, false, k);
		if (blend !== null) this.layer(this.sheet, blend, alpha, false, k);
		this.layer(this.tint, cell, blend !== null ? 1 - alpha : 1, true, k);
		if (blend !== null) this.layer(this.tint, blend, alpha, true, k);
		ctx.restore();
	}

	layer(img, cell, alpha, add, k) {
		const [x, y, w, h] = cellRect(this.base, cell);
		const W = this.canvas.width, H = this.canvas.height;
		const dw = w * k, dh = h * k, dx = (W - dw) / 2, dy = (H - dh) / 2;
		const ctx = this.ctx;
		if (!add) {
			ctx.globalAlpha = alpha;
			ctx.globalCompositeOperation = "source-over";
			ctx.drawImage(img, x, y, w, h, dx, dy, dw, dh);
			return;
		}
		addTint(ctx, this.off, img, [x, y, w, h], [dx, dy, dw, dh], this.colour, alpha);
	}
}

/**
 * One animation loop for many figures: each moving one advances by the real time between frames; one paused, held or
 * not loaded yet waits.
 */
export function play(figures) {
	let last = performance.now();
	function frame(now) {
		const dt = (now - last) / 1000;
		last = now;
		for (const f of figures) if (f.ready && !f.paused && !f.held) f.advance(dt);
		requestAnimationFrame(frame);
	}
	requestAnimationFrame(frame);
}

function loaded(src) {
	return new Promise((done, failed) => {
		const img = new Image();
		img.onload = () => done(img);
		img.onerror = () => failed(new Error(`${src} did not load`));
		img.src = src;
	});
}

/** The canvas's backing store at its CSS size in the screen's pixels. */
function fit(canvas) {
	const scale = window.devicePixelRatio || 1;
	const w = Math.round(canvas.clientWidth * scale), h = Math.round(canvas.clientHeight * scale);
	if (w > 0 && h > 0 && (canvas.width !== w || canvas.height !== h)) {
		canvas.width = w;
		canvas.height = h;
	}
}

/** Plays every canvas[data-figure] under `root` (the module header). */
export async function boot(root = document) {
	const canvases = [...root.querySelectorAll("canvas[data-figure]")];
	if (!canvases.length) return;
	const still = window.matchMedia("(prefers-reduced-motion: reduce)");
	const figures = [];
	for (const canvas of canvases) {
		const button = canvas.id ? root.querySelector(`button[aria-controls="${canvas.id}"]`) : null;
		try {
			const url = new URL(canvas.dataset.figure, document.baseURI);
			const response = await fetch(url);
			if (!response.ok) throw new Error(`${url} answered ${response.status}`);
			const fig = await response.json();
			const [m, base] = resolve(fig, fig.move);
			if (!m) throw new Error(`${url} has no entry for ${fig.move}`);
			const [sheet, tint] = await Promise.all([loaded(new URL(base.file, url)), loaded(new URL(base.tint, url))]);
			const f = new Figure(canvas, fig, fig.move, sheet, tint, shares(fig.colour));
			const toggle = () => {
				if (f.held) return;
				f.paused = !f.paused;
				if (button) button.setAttribute("aria-pressed", String(f.paused));
			};
			canvas.addEventListener("click", toggle);
			if (button) button.addEventListener("click", toggle);
			const motion = () => {
				f.held = still.matches;
				if (button) button.hidden = f.held;
				if (f.held) f.still(m.key || 0);
			};
			still.addEventListener("change", motion);
			fit(canvas);
			new ResizeObserver(() => {
				fit(canvas);
				f.draw();
			}).observe(canvas);
			motion();
			f.draw();
			figures.push(f);
		} catch (err) {
			if (button) button.hidden = true;
			console.error("figure:", err);
		}
	}
	if (figures.length) play(figures);
}

boot();
