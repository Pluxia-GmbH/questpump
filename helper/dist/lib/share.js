// The share card on the computer (docs/ui.md, "Hero, the Book and the Log": a web view has no share sheet, so the
// card offers Save image and Copy image; the phones' ShareCard and ShareWindow, Android's drawShareCard the model): a
// record or a step unlocked as the quest page on the iron page, drawn on a 1080-px canvas from the tokens, in the
// player's style (Parchment free; Night and Gilded with Pro, the Style row absent without it) with the move's key pose
// in its quality's well; "Save image" hands the PNG to Rust's save dialog (the shell's `save_image`), "Copy image"
// puts it on the clipboard (a ClipboardItem whose PNG is promised inside the click). The window opens over whatever
// asks for it, and a pump that starts while it is open closes it: `openShare(card)`, card {kind: "record" | "unlock",
// key (the move's figure key), name (the move's words), title, meta, quality (1 Starter … 7 Artifact), qualityWord,
// step, record and seconds (a record's)}; since the Saga (2026-10-15) a day's {kind: "day", day, ctx} (its Saga card
// and Quest Bill) and a week's {kind: "week", day, ctx}; since slice 45 the streak card's (`streakCard(days,
// milestone)`: {kind: "streak", days, milestone, best, tokens, day, character, tint}), and a day's Reel,
// its layers drawn here (`reelLayers`) and made in Rust (lib/reel.js), with Copy video and Save video in place of
// Copy image and Save image; since slice 51 the timed pump card (`expeditionCard(...)`: {kind: "expedition", minutes,
// place, scene, moves, records, colour, record, longest, streak, day, character, tint}), and the streak card's faces
// with it, drawn on a hand-painted scene with one true headline and, from questpump.com's standing (the `standing`
// topic, asked as the window opens), the rarity line (`drawScene`; the streak's sticker stays the Saga's).

import {ART} from "../art/art.js";
import {callBytes, get, intent, subscribe} from "./bridge.js";
import {t} from "./copy.js";
import {extraStill, tileStill} from "./figure.js";
import {announce, keepFocus} from "./focus.js";
import {clock, LOCALE} from "./format.js";
import {ReelGone, ReelMaker, copyVideo, reelPreview, saveVideo} from "./reel.js";
import {barRuns, blockSize, chapterWords, dayWord, heroSpoken, initial, movesLabel, number, partTotals, partWord,
	placeBars, setWords, shortDate, stripWords, unit, weekOf} from "./saga.js";
import {h, qualityName, styled} from "../kit/element.js";
import {milestoneWords, streakYears} from "../kit/qp-fanfare.js";

/** A card saved or copied: said to the counts (`feature_day`'s share). */
function shared() {
	intent("usage.seen", {feature: "share"}).catch(console.error);
}

/** The card's side in pixels, its pixels a point (the iPhone's 360-pt card at 3×), a rule's height. */
const SIDE = 1080;
const PX = 3;
const RULE = PX;
/** The figure's well and the room above and below its row in points, under a title of one line and of two; the pose
 * 6 % in from each side (the tile's), and caps' tracking in em. */
const ROOMY = {well: 120, around: 12};
const SNUG = {well: 104, around: 4};
const POSE_INSET = 0.06;
const CAPS_TRACKING = 0.12;
/** The styles, Parchment first; the others Pro's. */
const STYLES = ["parchment", "night", "gilded"];

function token(name) {
	return getComputedStyle(document.documentElement).getPropertyValue(name).trim();
}

/** A gradient's stops as [offset 0-1, colour] from a `-stops` token ("#F2EBDC 0%, #D8CBB0 100%"). */
function stops(value) {
	return splitTop(value).map((part) => {
		const at = part.lastIndexOf(" ");
		return [parseFloat(part.slice(at + 1)) / 100, part.slice(0, at).trim()];
	});
}

/** A list split at its top-level commas (not those inside a colour's parentheses). */
function splitTop(value) {
	const out = [];
	let depth = 0;
	let from = 0;
	for (let i = 0; i < value.length; i += 1) {
		if (value[i] === "(") depth += 1;
		else if (value[i] === ")") depth -= 1;
		else if (value[i] === "," && depth === 0) {
			out.push(value.slice(from, i).trim());
			from = i + 1;
		}
	}
	out.push(value.slice(from).trim());
	return out.filter(Boolean);
}

/** A radial gradient token ("radial-gradient(80% 80% at 50% 35%, #2C261F 0%, #16130F 100%)") over the rect. */
function radial(ctx, value, x, y, w, ht) {
	const inner = value.slice(value.indexOf("(") + 1, value.lastIndexOf(")"));
	const [shape, ...rest] = splitTop(inner);
	const [size, at] = shape.split(" at ");
	const [rw, rh] = size.trim().split(/\s+/).map((v) => parseFloat(v) / 100);
	const [cx, cy] = at.trim().split(/\s+/).map((v) => parseFloat(v) / 100);
	ctx.save();
	ctx.beginPath();
	ctx.rect(x, y, w, ht);
	ctx.clip();
	ctx.translate(x + cx * w, y + cy * ht);
	ctx.scale(rw * w, rh * ht);
	const g = ctx.createRadialGradient(0, 0, 0, 0, 0, 1);
	for (const [offset, colour] of stops(rest.join(", "))) g.addColorStop(Math.min(1, Math.max(0, offset)), colour);
	ctx.fillStyle = g;
	ctx.fillRect(-1 / (rw || 1), -1 / (rh || 1), 2 / (rw || 1), 2 / (rh || 1));
	ctx.restore();
}

/** A colour token ("rgb(232 184 74 / 0.6)", "rgb(232 184 74)") at alpha 0. */
function clear(colour) {
	return colour.replace(/\s*\/\s*[\d.]+%?\s*\)$|\)$/, " / 0)");
}

/** A shadow token ("0px 12px 30px rgb(0 0 0 / 0.45)") as x, y, blur and colour. */
function shadow(value) {
	const [x, y, blur, ...colour] = value.split(/\s+(?![^(]*\))/);
	return {x: parseFloat(x), y: parseFloat(y), blur: parseFloat(blur), colour: colour.join(" ")};
}

/** A style's page: its ground (top to bottom), its edge, its vignette, its words and rules (`ShareStyle`). */
function look(style) {
	const parchment = {
		ground: stops(token("--qp-surface-quest-page-fill-stops")),
		edge: {colour: token("--qp-surface-quest-page-edge-color"), width: parseFloat(token("--qp-surface-quest-page-edge-width"))},
		vignette: shadow(token("--qp-surface-quest-page-vignette")),
		words: token("--qp-color-ink"),
		words2: token("--qp-color-ink2"),
		rule: token("--qp-surface-quest-page-rule"),
	};
	if (style === "night") {
		return {
			ground: stops(token("--qp-surface-share-card-night-ground-stops")),
			edge: {colour: token("--qp-surface-share-card-night-edge"),
				width: parseFloat(token("--qp-surface-share-card-night-edge-width"))},
			vignette: null,
			words: token("--qp-surface-share-card-night-words"),
			words2: token("--qp-surface-share-card-night-words2"),
			rule: token("--qp-surface-share-card-night-rule"),
			night: true,
		};
	}
	if (style === "gilded") {
		return {...parchment, edge: {colour: token("--qp-surface-share-card-gilded-edge"),
			width: parseFloat(token("--qp-surface-share-card-gilded-edge-width"))},
		rule: token("--qp-surface-share-card-gilded-rule")};
	}
	return parchment;
}

/** Words in a quality: its tone on Night's iron, its ink twin on the parchment. */
function qualityWords(theme, quality) {
	const name = qualityName(quality);
	if (!name) return theme.words;
	return token(theme.night ? `--qp-quality-${name}` : `--qp-quality-${name}-ink`) || theme.words;
}

/** The fonts the card draws with (the tokens' faces at the phones' card sizes), loaded before it draws. */
const FONTS = {
	title: `700 ${22 * PX}px "Cinzel"`, // copy-lint: not shown (a canvas font)
	meta: `400 ${13 * PX}px "Barlow"`, // copy-lint: not shown (a canvas font)
	caps: `600 ${12 * PX}px "Barlow"`, // copy-lint: not shown (a canvas font)
	number: `700 ${72 * PX}px "Barlow Condensed"`, // copy-lint: not shown (a canvas font)
	line: `600 ${17 * PX}px "Barlow"`, // copy-lint: not shown (a canvas font)
};

async function fonts() {
	await Promise.all(Object.values(FONTS).map((f) => document.fonts.load(f)));
}

/** Words broken into lines within `width`, between words only (a hyphenated term stays whole). */
function wrap(ctx, text, width) {
	const lines = [];
	let line = "";
	for (const word of text.split(" ")) {
		const next = line ? `${line} ${word}` : word;
		if (line && ctx.measureText(next).width > width) {
			lines.push(line);
			line = word;
		} else {
			line = next;
		}
	}
	if (line) lines.push(line);
	return lines;
}

/** The move's key pose, loaded through the asset protocol for the canvas; null without one. */
function pose(key) {
	const still = key ? tileStill(key) : null;
	if (!still) return Promise.resolve(null);
	return new Promise((resolve) => {
		const img = new Image();
		img.crossOrigin = "anonymous";
		img.onload = () => resolve({img, crop: still.crop, mirror: still.mirror});
		img.onerror = () => resolve(null);
		img.src = still.src;
	});
}

function roundRect(ctx, x, y, w, ht, r) {
	ctx.beginPath();
	ctx.roundRect(x, y, w, ht, r);
}

/** The quest page's rule: its style's colour fading out at both ends, one point high. */
function rule(ctx, x, y, width, colour) {
	const g = ctx.createLinearGradient(x, 0, x + width, 0);
	g.addColorStop(0, "rgb(0 0 0 / 0)");
	g.addColorStop(0.5, colour);
	g.addColorStop(1, "rgb(0 0 0 / 0)");
	ctx.fillStyle = g;
	ctx.fillRect(x, y, width, RULE);
}

/** Draws a record's or an unlock's card in `style` on a 1080-px canvas: the quest page on the iron page. */
async function drawQuest(card, style) {
	await fonts();
	const still = await pose(card.key);
	const canvas = document.createElement("canvas");
	canvas.width = SIDE;
	canvas.height = SIDE;
	const ctx = canvas.getContext("2d");
	const theme = look(style);
	// the page and its light
	ctx.fillStyle = token("--qp-surface-page-fill");
	ctx.fillRect(0, 0, SIDE, SIDE);
	radial(ctx, token("--qp-surface-page-glow"), 0, 0, SIDE, SIDE);
	const margin = 20 * PX;
	const pad = parseFloat(token("--qp-surface-quest-page-padding")) * PX;
	const inner = SIDE - 2 * margin - 2 * pad;
	ctx.font = FONTS.title;
	const title = wrap(ctx, card.title, inner);
	const titleLine = 28 * PX;
	ctx.font = FONTS.meta;
	const meta = wrap(ctx, card.meta, inner);
	const metaLine = 18 * PX;
	const room = title.length > 1 ? SNUG : ROOMY;
	const well = room.well * PX;
	const around = room.around * PX;
	const record = card.kind === "record";
	const numberLine = 76 * PX;
	const column = record ? 16 * PX + numberLine + (card.seconds ? 0 : 16 * PX) : 0;
	const row = Math.max(well, column);
	ctx.font = record ? FONTS.meta : FONTS.line;
	const underText = record ? t("ladders.card.quality", {quality: card.qualityWord, step: card.step})
		: t("ladders.card.unlock", {move: card.name, quality: card.qualityWord});
	const under = wrap(ctx, underText, inner);
	const underLine = (record ? 18 : 22) * PX;
	const pageHeight = pad + title.length * titleLine + 2 * PX + meta.length * metaLine + 14 * PX + RULE + 4 * PX
		+ around + row + around + under.length * underLine + 8 * PX + 4 * PX + 2 * PX + RULE + pad;
	const brandHeight = 16 * PX;
	let y = (SIDE - (pageHeight + 12 * PX + brandHeight)) / 2;
	const page = {x: margin, y, w: SIDE - 2 * margin, h: pageHeight};
	const radius = parseFloat(token("--qp-surface-quest-page-radius")) * PX;
	// the ground over its drop shadow, the parchment's vignette and the edge
	const drop = shadow(token("--qp-surface-quest-page-shadow"));
	ctx.save();
	ctx.shadowColor = drop.colour;
	ctx.shadowBlur = drop.blur * PX;
	ctx.shadowOffsetX = drop.x * PX;
	ctx.shadowOffsetY = drop.y * PX;
	roundRect(ctx, page.x, page.y, page.w, page.h, radius);
	ctx.fillStyle = theme.ground.at(-1)[1];
	ctx.fill();
	ctx.restore();
	const ground = ctx.createLinearGradient(0, page.y, 0, page.y + page.h);
	for (const [offset, colour] of theme.ground) ground.addColorStop(offset, colour);
	roundRect(ctx, page.x, page.y, page.w, page.h, radius);
	ctx.fillStyle = ground;
	ctx.fill();
	if (theme.vignette) {
		ctx.save();
		roundRect(ctx, page.x, page.y, page.w, page.h, radius);
		ctx.clip();
		ctx.shadowColor = theme.vignette.colour;
		ctx.shadowBlur = theme.vignette.blur * PX;
		ctx.beginPath();
		ctx.rect(page.x - 4 * SIDE, page.y - 4 * SIDE, page.w + 8 * SIDE, page.h + 8 * SIDE);
		ctx.roundRect(page.x, page.y, page.w, page.h, radius);
		ctx.fillStyle = theme.vignette.colour;
		ctx.fill("evenodd");
		ctx.restore();
	}
	const inset = (theme.edge.width * PX) / 2;
	roundRect(ctx, page.x + inset, page.y + inset, page.w - 2 * inset, page.h - 2 * inset, radius - inset);
	ctx.lineWidth = theme.edge.width * PX;
	ctx.strokeStyle = theme.edge.colour;
	ctx.stroke();
	// the title, the meta line and the rule
	const x = page.x + pad;
	y = page.y + pad;
	ctx.textBaseline = "top";
	ctx.fillStyle = theme.words;
	ctx.font = FONTS.title;
	for (const line of title) {
		ctx.fillText(line, x, y);
		y += titleLine;
	}
	y += 2 * PX;
	ctx.fillStyle = theme.words2;
	ctx.font = FONTS.meta;
	for (const line of meta) {
		ctx.fillText(line, x, y);
		y += metaLine;
	}
	y += 14 * PX;
	rule(ctx, x, y, inner, theme.rule);
	y += RULE + 4 * PX + around;
	// the figure's well, edged in the quality's tone, on the row (an unlock's across the page too)
	const wx = record ? x : x + (inner - well) / 2;
	const wy = y + (row - well) / 2;
	const tileRadius = parseFloat(token("--qp-layout-tile-radius")) * PX;
	ctx.save();
	roundRect(ctx, wx, wy, well, well, tileRadius);
	ctx.clip();
	radial(ctx, token("--qp-surface-move-tile-well"), wx, wy, well, well);
	if (still) {
		const [cx, cy, cs] = still.crop ?? [0, 0, 1];
		const side = Math.min(still.img.naturalWidth, still.img.naturalHeight);
		const poseInset = well * POSE_INSET;
		ctx.translate(wx + well / 2, wy + well / 2);
		if (still.mirror) ctx.scale(-1, 1);
		ctx.drawImage(still.img, cx * side, cy * side, cs * side, cs * side, -well / 2 + poseInset, -well / 2 + poseInset,
			well - 2 * poseInset, well - 2 * poseInset);
	}
	ctx.restore();
	const edge = parseFloat(token("--qp-layout-tile-edge")) * PX;
	const tone = qualityName(card.quality) ? token(`--qp-quality-${qualityName(card.quality)}`)
		: token("--qp-color-line-strong");
	roundRect(ctx, wx + edge / 2, wy + edge / 2, well - edge, well - edge, tileRadius);
	ctx.lineWidth = edge;
	ctx.strokeStyle = tone;
	ctx.stroke();
	// beside it a record: its label, the number and its unit, centred on the row
	if (record) {
		let cy = y + (row - column) / 2;
		const cx = x + well + 20 * PX;
		ctx.fillStyle = theme.words2;
		ctx.font = FONTS.caps;
		ctx.letterSpacing = `${CAPS_TRACKING * 12 * PX}px`;
		ctx.fillText(t("ladders.record.label").toLocaleUpperCase(LOCALE), cx, cy);
		cy += 16 * PX;
		ctx.letterSpacing = "0px";
		ctx.fillStyle = theme.words;
		ctx.font = FONTS.number;
		const amount = card.seconds ? clock(card.record) : new Intl.NumberFormat(LOCALE).format(card.record);
		ctx.fillText(amount, cx, cy);
		cy += numberLine;
		if (!card.seconds) {
			ctx.fillStyle = theme.words2;
			ctx.font = FONTS.caps;
			ctx.letterSpacing = `${CAPS_TRACKING * 12 * PX}px`;
			ctx.fillText(t("ladders.card.reps").toLocaleUpperCase(LOCALE), cx, cy);
			ctx.letterSpacing = "0px";
		}
	}
	y += row + around;
	// under the row: a record's quality and step; an unlock's move and quality, centred, in its quality
	ctx.fillStyle = record ? theme.words2 : qualityWords(theme, card.quality);
	ctx.font = record ? FONTS.meta : FONTS.line;
	ctx.textAlign = record ? "left" : "center";
	for (const line of under) {
		ctx.fillText(line, record ? x : x + inner / 2, y);
		y += underLine;
	}
	ctx.textAlign = "left";
	y += 8 * PX + 4 * PX + 2 * PX;
	rule(ctx, x, y, inner, theme.rule);
	// QuestPump under the page
	ctx.fillStyle = token("--qp-color-gold-hi");
	ctx.font = FONTS.caps;
	ctx.letterSpacing = `${CAPS_TRACKING * 12 * PX}px`;
	ctx.textAlign = "center";
	ctx.fillText(t("app.name").toLocaleUpperCase(LOCALE), SIDE / 2, page.y + page.h + 12 * PX);
	return canvas;
}

/** A record's or an unlock's words: what a screen reader hears for its picture ("Push-up: 17", "Unlocked: Archer
 * push-up · Rare"). */
function questWords(card) {
	return card.kind === "record" ? t("ladders.share.text", {move: card.name, record: card.record})
		: t("ladders.share.unlock", {move: card.name, quality: card.qualityWord});
}

// The Saga's cards (the Saga, 2026-10-15; boards A-Story, A-Feed, B-Feed; Tokens.Surface.SagaCard, QuestBill) --------

/** The day's cards and their formats; the sticker the day's Saga card's and the streak card's alone, the reel the day's
 * Saga card's. */
const CARDS = ["saga", "bill"];
const FORMATS = ["story", "post", "square", "sticker", "reel"];
/** The Quest Bill's wax seal's turn, in turns: a 30th of a turn anticlockwise (QuestBill; the phones' -12°). */
const SEAL_TURN = -1 / 30;

/** The cards' family for Barlow Condensed 700 with its tabular figures (fonts.css's tabular face, since a canvas
 * reads no font-variant-numeric; the iPhone's numLarge and numSaga): every word of the Quest Bill and of a reel's bill
 * line, so a column's digits line up, and every numSaga word (the cards' number, its unit, the strip's total), as the
 * tokens' tabular role says. */
const FIGURES = "barlow-condensed-tabular";
/** Barlow's and Barlow Condensed's line as a share of the size (hhea ascent 1.0 em over a descent of 0.2 em;
 * tokens.json `family`) and its baseline's depth under the line's top: a line of words centred in a taller line, as
 * the phones centre a text's line. */
const LINE = 1.2;
const ASCENT = 1.0;
/** The Quest Bill's more line's sign in the counts' column, before N more lines (board B-Feed; a sign, as the
 * stepper's keys). */
const MORE_MARK = "+";

/** A token's number (a length in pt, a count). */
function num(name) {
	return parseFloat(token(name)) || 0;
}

/** The least share of its size a card's number shrinks to where it must fit (SagaCard numberLeastFraction). */
function numberLeast() {
	return num("--qp-surface-saga-card-number-least-fraction");
}

/** A canvas font of a token's family at `pt` points. */
function font(weight, pt, family) {
	return `${weight} ${pt * PX}px ${token(`--qp-type-${family}-family`)}`; // copy-lint: not shown (a canvas font)
}

/** The fonts the Saga's cards draw with, loaded before they draw. */
async function sagaFonts() {
	const faces = [font(700, 15, "wordmark"), font(400, 13, "body"), font(600, 13, "body"), font(700, 13, "body"),
		font(500, 13, "body"), font(700, 40, FIGURES)];
	await Promise.all(faces.map((f) => document.fonts.load(f)));
}

/** An image loaded for the canvas; null where it does not load. */
function image(src) {
	return new Promise((resolve) => {
		const img = new Image();
		img.crossOrigin = "anonymous";
		img.onload = () => resolve(img);
		img.onerror = () => resolve(null);
		img.src = src;
	});
}

/** A piece of the own art (helper/dist/art/art.js) as an image, its tint layers in `colour`. */
function art(name, colour) {
	const piece = ART[name];
	if (!piece) return Promise.resolve(null);
	// copy-lint: not shown (an SVG document's markup)
	const svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${piece.width} ${piece.height}" `
		// copy-lint: not shown (an SVG document's markup)
		+ `width="${piece.width * 8}" height="${piece.height * 8}" color="${colour}">${piece.markup}</svg>`;
	return image(`data:image/svg+xml;charset=utf-8,${encodeURIComponent(svg)}`);
}

/** Words drawn with caps' tracking (`em` of the size) and in upper case where `caps`, slanted where `italic`; with a
 * `halo` (the sticker's: {colour, width, glow}) over its outline that wide each side, the glow's shadow under it. */
function words(ctx, text, x, y, {size, weight = 400, family = "body", colour, align = "left", caps = false,
	tracking = 0, baseline = "alphabetic", halo = null, italic = false}) {
	const shown = caps ? text.toLocaleUpperCase(LOCALE) : text;
	ctx.save();
	ctx.font = `${italic ? "italic " : ""}${font(weight, size, family)}`; // copy-lint: not shown (a canvas font)
	ctx.textAlign = align;
	ctx.textBaseline = baseline;
	ctx.letterSpacing = `${tracking * size * PX}px`;
	if (halo) {
		ctx.save();
		glowing(ctx, halo.glow);
		ctx.lineJoin = "round";
		ctx.strokeStyle = halo.colour;
		ctx.lineWidth = 2 * halo.width;
		ctx.strokeText(shown, x, y);
		ctx.restore();
	}
	ctx.fillStyle = colour;
	ctx.fillText(shown, x, y);
	const width = ctx.measureText(shown).width;
	ctx.restore();
	return width;
}

/** A shadow token's shadow (`shadow`) under what draws next. */
function glowing(ctx, glow) {
	ctx.shadowColor = glow.colour;
	ctx.shadowBlur = glow.blur * PX;
	ctx.shadowOffsetX = glow.x * PX;
	ctx.shadowOffsetY = glow.y * PX;
}

/** Words cut to `width` (points) with an ellipsis where they are wider (a long move's name). */
function fitted(ctx, text, width, style) {
	if (measure(ctx, text, style) / PX <= width) return text;
	let cut = text;
	while (cut.length > 1 && measure(ctx, `${cut}…`, style) / PX > width) cut = cut.slice(0, -1);
	return `${cut.trimEnd()}…`;
}

/** A reel line's name on one line `width` points wide (docs/ui.md, "The share card"), so the slot keeps one height on
 * every shot: whole, then cut with an ellipsis after each shorter run of its whole words, then its first word cut a
 * letter at a time (one too wide alone), the first that fits (the iPhone's `ReelLine.cuts`, Android's `cutWhole`). */
function wholeFitted(ctx, text, width, style) {
	const fits = (shown) => measure(ctx, shown, style) / PX <= width;
	if (fits(text)) return text;
	const names = text.trim().split(/\s+/);
	const first = Array.from(names[0]);
	const cuts = [];
	for (let n = names.length - 1; n >= 1; n -= 1) cuts.push(`${names.slice(0, n).join(" ")}…`);
	for (let end = first.length - 1; end >= 1; end -= 1) cuts.push(`${first.slice(0, end).join("")}…`);
	return cuts.find(fits) ?? cuts.at(-1) ?? text;
}

/** The width words would take (as `words` draws them). */
function measure(ctx, text, {size, weight = 400, family = "body", caps = false, tracking = 0, italic = false}) {
	ctx.save();
	ctx.font = `${italic ? "italic " : ""}${font(weight, size, family)}`; // copy-lint: not shown (a canvas font)
	ctx.letterSpacing = `${tracking * size * PX}px`;
	const w = ctx.measureText(caps ? text.toLocaleUpperCase(LOCALE) : text).width;
	ctx.restore();
	return w;
}

/** A Saga card's look in a style: Night the iron page in gold, Parchment the parchment in ink with the number in gold
 * ink and the kinds in their ink twins, Gilded the parchment in its gold band. */
function sagaLook(style) {
	const night = style === "night";
	const ink = (dark, light) => token(night ? dark : light);
	return {
		night,
		gilded: style === "gilded",
		words: ink("--qp-color-text", "--qp-color-ink"),
		words2: ink("--qp-color-text2", "--qp-color-ink2"),
		words3: ink("--qp-color-text3", "--qp-color-ink2"),
		gold: ink("--qp-color-gold", "--qp-surface-saga-card-parchment-number"),
		axis: ink("--qp-surface-day-strip-axis", "--qp-surface-day-strip-axis-ink"),
		grid: ink("--qp-surface-day-strip-grid", "--qp-surface-day-strip-grid-ink"),
		total: ink("--qp-surface-day-strip-total", "--qp-surface-day-strip-total-ink"),
		oldBest: ink("--qp-surface-week-strip-old-best", "--qp-surface-week-strip-old-best-ink"),
		ringed: ink("--qp-color-gold", "--qp-surface-week-strip-today-ink"),
		ring: ink("--qp-color-gold-hi", "--qp-surface-week-strip-ring-ink"),
		kind: (k) => token(`--qp-surface-kinds-${k}${night ? "" : "-ink"}`),
		quality: (q) => token(night ? `--qp-quality-${q}` : `--qp-quality-${q}-ink`),
		rule: night ? token("--qp-surface-share-card-night-rule") : token("--qp-surface-quest-page-rule"),
	};
}

/** The card's ground: Night's iron with its light, stripes and gold edge lines; the parchment with its vignette, in
 * the gold band for Gilded. */
function sagaGround(ctx, look, w, ht, format) {
	if (look.night) {
		const g = ctx.createLinearGradient(0, 0, 0, ht);
		for (const [o, c] of stops(token("--qp-surface-saga-card-night-ground-stops"))) g.addColorStop(o, c);
		ctx.fillStyle = g;
		ctx.fillRect(0, 0, w, ht);
		const tall = format === "story";
		radial(ctx, token(tall ? "--qp-surface-saga-card-warm" : "--qp-surface-saga-card-warm-post"), 0, 0, w, ht);
		radial(ctx, token(tall ? "--qp-surface-saga-card-glow" : "--qp-surface-saga-card-glow-post"), 0, 0, w, ht);
		stripes(ctx, w, ht);
		// the gold lines along the top and the bottom, fading out to both ends (to their own colour, clear: a gradient
		// mixes its colours unpremultiplied, and a clear black would darken them on the way)
		const edge = num("--qp-surface-saga-card-night-edge-line-width") * PX;
		const colour = token("--qp-surface-saga-card-night-edge-line-color");
		const line = ctx.createLinearGradient(0, 0, w, 0);
		line.addColorStop(0, clear(colour));
		line.addColorStop(0.5, colour);
		line.addColorStop(1, clear(colour));
		ctx.fillStyle = line;
		ctx.fillRect(0, 0, w, edge);
		ctx.fillRect(0, ht - edge, w, edge);
		return;
	}
	const g = ctx.createLinearGradient(0, 0, 0, ht);
	for (const [o, c] of stops(token("--qp-surface-quest-page-fill-stops"))) g.addColorStop(o, c);
	ctx.fillStyle = g;
	ctx.fillRect(0, 0, w, ht);
	const vignette = shadow(token("--qp-surface-quest-page-vignette"));
	ctx.save();
	ctx.shadowColor = vignette.colour;
	ctx.shadowBlur = vignette.blur * PX;
	ctx.beginPath();
	ctx.rect(-4 * w, -4 * ht, 9 * w, 9 * ht);
	ctx.rect(0, 0, w, ht);
	ctx.fillStyle = vignette.colour;
	ctx.fill("evenodd");
	ctx.restore();
	if (look.gilded) gildedBand(ctx, w, ht);
}

/** ShareCard's gilded band around a whole card (Gilded's Saga card and Quest Bill). */
function gildedBand(ctx, w, ht) {
	const band = num("--qp-surface-share-card-gilded-edge-width") * PX;
	ctx.strokeStyle = token("--qp-surface-share-card-gilded-edge");
	ctx.lineWidth = band;
	ctx.strokeRect(band / 2, band / 2, w - band, ht - band);
}

/** A quality's gem: a diamond `side` points across in its tone, centred on (cx, cy) in points (Android's drawGem). */
function diamond(ctx, tone, cx, cy, side) {
	const r = side / 2;
	ctx.save();
	ctx.fillStyle = tone;
	ctx.beginPath();
	ctx.moveTo(cx * PX, (cy - r) * PX);
	ctx.lineTo((cx + r) * PX, cy * PX);
	ctx.lineTo(cx * PX, (cy + r) * PX);
	ctx.lineTo((cx - r) * PX, cy * PX);
	ctx.closePath();
	ctx.fill();
	ctx.restore();
}

/** The Saga hero's faint stripes over a ground. */
function stripes(ctx, w, ht) {
	const pitch = num("--qp-surface-saga-hero-stripe-pitch") * PX;
	const width = num("--qp-surface-saga-hero-stripe-width") * PX;
	ctx.save();
	ctx.fillStyle = token("--qp-surface-saga-hero-stripe-color");
	ctx.translate(w / 2, ht / 2);
	ctx.rotate((28 * Math.PI) / 180);
	const reach = Math.hypot(w, ht);
	for (let x = -reach; x < reach; x += pitch) ctx.fillRect(x, -reach, width, 2 * reach);
	ctx.restore();
}

/** A slanted tag's width in points, its slant's lean included. */
function tagWidth(ctx, label, star) {
	const text = measure(ctx, label, {size: 12, weight: 700, caps: true, tracking: 0.12}) / PX;
	const inner = (star ? num("--qp-surface-slant-tag-star-size") + 6 : 0) + text;
	const ht = 16 + 2 * num("--qp-surface-slant-tag-pad-v");
	return inner + 2 * num("--qp-surface-slant-tag-pad-h") + num("--qp-surface-slant-tag-slant-fraction") * ht;
}

/** A day's word as a slanted tag at (x, y) its top-left: Best day ever filled gold with the star, else the word
 * outlined in its tone; with a `halo` (the sticker's, outlined alone) its edge, star and words over their outline and
 * shadow. Returns its width. */
async function slantTag(ctx, x, y, label, tone, filled, star, halo = null) {
	const size = 12;
	const padH = num("--qp-surface-slant-tag-pad-h");
	const padV = num("--qp-surface-slant-tag-pad-v");
	const starSize = num("--qp-surface-slant-tag-star-size");
	const text = measure(ctx, label, {size, weight: 700, caps: true, tracking: 0.12}) / PX;
	const inner = (star ? starSize + 6 : 0) + text;
	const w = inner + 2 * padH;
	const ht = 16 + 2 * padV;
	const slant = num("--qp-surface-slant-tag-slant-fraction");
	ctx.save();
	ctx.beginPath();
	ctx.moveTo((x + slant * ht) * PX, y * PX);
	ctx.lineTo((x + w + slant * ht) * PX, y * PX);
	ctx.lineTo((x + w) * PX, (y + ht) * PX);
	ctx.lineTo(x * PX, (y + ht) * PX);
	ctx.closePath();
	const edge = num("--qp-surface-slant-tag-edge-width") * PX;
	if (filled) {
		ctx.fillStyle = tone;
		ctx.fill();
	} else {
		if (halo) {
			ctx.save();
			glowing(ctx, halo.glow);
			ctx.lineJoin = "round";
			ctx.strokeStyle = halo.colour;
			ctx.lineWidth = edge + 2 * halo.width;
			ctx.stroke();
			ctx.restore();
		}
		ctx.strokeStyle = tone;
		ctx.lineWidth = edge;
		ctx.stroke();
	}
	ctx.restore();
	let at = x + padH + (slant * ht) / 2;
	const colour = filled ? token("--qp-color-gold-ink") : tone;
	if (star) {
		const glyph = await art("star", colour);
		if (glyph) {
			ctx.save();
			if (halo) glowing(ctx, halo.glow);
			ctx.drawImage(glyph, at * PX, (y + (ht - starSize) / 2) * PX, starSize * PX, starSize * PX);
			ctx.restore();
		}
		at += starSize + 6;
	}
	words(ctx, label, at * PX, (y + ht / 2) * PX, {size, weight: 700, colour, caps: true, tracking: 0.12,
		baseline: "middle", halo});
	return w + slant * ht;
}

/** The parchment's dark rounded well a card's knight stands in, `size` points square at (x, y); none on Night's iron.
 * The day reel draws it under the knight's film. */
function well(ctx, look, x, y, size) {
	if (look.night) return;
	ctx.save();
	roundRect(ctx, x * PX, y * PX, size * PX, size * PX, num("--qp-surface-saga-card-well-radius") * PX);
	ctx.clip();
	radial(ctx, token("--qp-surface-move-tile-well"), x * PX, y * PX, size * PX, size * PX);
	ctx.restore();
}

/** The knight that stands for the card (`still`: the figure's still as `pose` gives it, the move's key pose): cut out on Night's iron (a soft round mask, drawn lighten so the still's own ground
 * leaves no box), in its dark rounded well on the parchment. */
async function knight(ctx, look, still, x, y, size) {
	if (!still) return;
	const px = size * PX;
	const off = document.createElement("canvas");
	off.width = px;
	off.height = px;
	const o = off.getContext("2d");
	const [cx, cy, cs] = still.crop ?? [0, 0, 1];
	// an image's or a recoloured still's (an ImageBitmap's) pixels
	const side = Math.min(still.img.naturalWidth ?? still.img.width, still.img.naturalHeight ?? still.img.height);
	o.save();
	if (still.mirror) {
		o.translate(px, 0);
		o.scale(-1, 1);
	}
	o.drawImage(still.img, cx * side, cy * side, cs * side, cs * side, 0, 0, px, px);
	o.restore();
	if (look.night) {
		const fade = num("--qp-surface-saga-hero-mask-fraction");
		const g = o.createRadialGradient(px / 2, px / 2, 0, px / 2, px / 2, px / 2);
		g.addColorStop(0, "rgb(0 0 0)");
		g.addColorStop(fade, "rgb(0 0 0)");
		g.addColorStop(1, "rgb(0 0 0 / 0)");
		o.globalCompositeOperation = "destination-in";
		o.fillStyle = g;
		o.fillRect(0, 0, px, px);
		ctx.save();
		ctx.globalCompositeOperation = "lighten";
		ctx.drawImage(off, x * PX, y * PX);
		ctx.restore();
		return;
	}
	well(ctx, look, x, y, size);
	ctx.save();
	roundRect(ctx, x * PX, y * PX, px, px, num("--qp-surface-saga-card-well-radius") * PX);
	ctx.clip();
	ctx.globalCompositeOperation = "lighten";
	ctx.drawImage(off, x * PX, y * PX);
	ctx.restore();
}

/** The strip of a day on a card (rule 9): its bars at their pumps' starts at one pitch (a crowded hour a block),
 * stacked by kind, the parts' totals over them and their names under them, both rows moved apart, "04:00" at both ends
 * where the names leave them room (lib/saga.js `placeBars`, `stripWords`), the plot `ht` points high from `y`. `grow`
 * (the day reel's reveal) gives each bar's share of its height by its place, grown from the axis as the Saga's strip
 * grows (none: whole). */
function cardStrip(ctx, look, bars, buckets, x, y, width, ht, grow = null) {
	const barW = num("--qp-surface-day-strip-bar-width");
	const gap = num("--qp-surface-day-strip-bar-gap");
	const rows = num("--qp-surface-day-strip-row-count");
	const blockGap = num("--qp-surface-day-strip-block-gap");
	const totals = 18;
	const plot = Math.max(12, ht - totals - 16);
	const axis = y + totals + plot;
	const unit = (plot - (rows - 1) * blockGap) / rows;
	ctx.save();
	ctx.strokeStyle = look.grid;
	ctx.lineWidth = PX;
	ctx.setLineDash([num("--qp-surface-day-strip-grid-dash") * PX, num("--qp-surface-day-strip-grid-gap") * PX]);
	for (let r = 1; r <= rows; r += 1) {
		const ly = (axis - r * (unit + blockGap)) * PX;
		ctx.beginPath();
		ctx.moveTo(x * PX, ly);
		ctx.lineTo((x + width) * PX, ly);
		ctx.stroke();
	}
	ctx.setLineDash([]);
	ctx.strokeStyle = look.axis;
	ctx.beginPath();
	ctx.moveTo(x * PX, axis * PX);
	ctx.lineTo((x + width) * PX, axis * PX);
	ctx.stroke();
	ctx.restore();
	const placed = placeBars(bars, x, width, barW, gap, 1 / PX);
	const {unit: block, gap: gapOf} = blockSize(bars, plot, rows, blockGap);
	bars.forEach((b, i) => {
		const share = grow ? grow(i) : 1;
		if (share <= 0) return;
		ctx.save();
		ctx.translate(0, axis * PX);
		ctx.scale(1, share);
		ctx.translate(0, -axis * PX);
		let top = axis;
		for (const run of barRuns(b.kinds)) {
			const h_ = run.count * block + (run.count - 1) * gapOf;
			top -= h_;
			ctx.fillStyle = look.kind(run.kind);
			roundRect(ctx, placed.xs[i] * PX, top * PX, placed.widths[i] * PX, h_ * PX,
				num("--qp-surface-day-strip-block-radius") * PX);
			ctx.fill();
			top -= gapOf;
		}
		ctx.restore();
	});
	const label = {size: num("--qp-surface-day-strip-label-size"), weight: 600, caps: true,
		tracking: num("--qp-surface-day-strip-label-tracking-fraction")};
	const end = num("--qp-surface-day-strip-end-size");
	const total = {size: num("--qp-surface-day-strip-total-size"), weight: 700, family: FIGURES};
	const parts = partTotals(bars, placed, buckets);
	const ends = measure(ctx, TURN, {size: end, weight: 500}) / PX;
	const laid = stripWords(parts.map((p) => ({x: p.x - x, totalWidth: measure(ctx, number(p.total), total) / PX,
		nameWidth: measure(ctx, partWord(p.part), label) / PX})), ends, width, num("--qp-layout-space8"));
	parts.forEach((p, i) => {
		words(ctx, number(p.total), (x + laid.totals[i]) * PX, (y + totals - 4) * PX, {...total, colour: look.total,
			align: "center"});
		if (laid.names) {
			words(ctx, partWord(p.part), (x + laid.names[i]) * PX, (axis + 13) * PX, {...label, colour: look.words2,
				align: "center"});
		}
	});
	const turn = {size: end, weight: 500, colour: look.words3};
	if (laid.ends[0]) words(ctx, TURN, x * PX, (axis + 13) * PX, turn);
	if (laid.ends[1]) words(ctx, TURN, (x + width) * PX, (axis + 13) * PX, {...turn, align: "right"});
}

/** The game day's turn as the strip's ends write it. */
const TURN = `${String(4).padStart(2, "0")}:00`;

/** The week's seven bars on a card (rule 17; WeekStrip): a bar a day with a move in `bar` (at least barLeast high,
 * scaled to the week's most or the old best where that is more), none for a day without one but a short rest bar in
 * WeekDots' rest where a rest token kept it (today's week's), the day ringed, the old
 * best a dashed line across the bars with its legend over them at the strip's end as the Week tile writes it (three
 * dashes, Old best in meta's Barlow and the number in numRow's Barlow Condensed), each weekday's initial under its
 * bar. */
function cardWeek(ctx, look, week, x, y, width, ht) {
	const {days, ringed, rested, oldBest} = week;
	const most = Math.max(1, oldBest ?? 0, ...days.map((d) => d.moves));
	const col = width / 7;
	const barW = Math.min(col * 0.55, 30);
	const least = num("--qp-surface-week-strip-bar-least");
	const radius = num("--qp-surface-week-strip-radius");
	const dash = num("--qp-surface-week-strip-dash") * PX;
	// the legend over the bars and the initials under them keep inside the strip's room
	const legend = oldBest ? 20 : 0;
	const plot = ht - 18 - legend;
	const bottom = y + legend + plot;
	days.forEach((d, i) => {
		const cx = x + col * i + col / 2;
		if (d.moves > 0) {
			const bh = Math.max(least, (d.moves / most) * plot);
			ctx.fillStyle = i === ringed ? look.ringed : token("--qp-surface-week-strip-bar");
			roundRect(ctx, (cx - barW / 2) * PX, (bottom - bh) * PX, barW * PX, bh * PX, radius * PX);
			ctx.fill();
			if (i === ringed) {
				ctx.strokeStyle = look.ring;
				ctx.lineWidth = num("--qp-surface-week-strip-ring-width") * PX;
				const g = num("--qp-surface-week-strip-ring-gap") + num("--qp-surface-week-strip-ring-width") / 2;
				roundRect(ctx, (cx - barW / 2 - g) * PX, (bottom - bh - g) * PX, (barW + 2 * g) * PX, (bh + 2 * g) * PX,
					(radius + g) * PX);
				ctx.stroke();
			}
		} else if (rested.includes(i)) {
			const rest = num("--qp-surface-week-strip-rest-height");
			ctx.save();
			ctx.globalAlpha = num("--qp-surface-week-strip-rest-opacity");
			ctx.fillStyle = token("--qp-surface-week-dots-rest");
			roundRect(ctx, (cx - barW / 2) * PX, (bottom - rest) * PX, barW * PX, rest * PX, radius * PX);
			ctx.fill();
			ctx.restore();
		}
		words(ctx, initial(d.day), cx * PX, (bottom + 18) * PX, {size: 11, weight: 600, colour: i === ringed ? look.total
			: look.words3, align: "center"});
	});
	if (!oldBest) return;
	const dashed = (from, to, ly) => {
		ctx.save();
		ctx.strokeStyle = look.oldBest;
		ctx.lineWidth = PX;
		ctx.setLineDash([dash, dash]);
		ctx.beginPath();
		ctx.moveTo(from * PX, ly * PX);
		ctx.lineTo(to * PX, ly * PX);
		ctx.stroke();
		ctx.restore();
	};
	dashed(x, x + width, bottom - (oldBest / most) * plot);
	// the legend right-aligned over the bars: the number at the strip's end, Old best and three dashes before it
	const ly = y + 12;
	const size = 12;
	const gap = num("--qp-layout-space4");
	const best = {size: size * (num("--qp-type-num-row-size") / num("--qp-type-meta-size")), weight: 700,
		family: "num-row", colour: look.words, align: "right"};
	const bestW = words(ctx, number(oldBest), (x + width) * PX, ly * PX, best) / PX;
	const label = {size, family: "meta", colour: look.words2, align: "right"};
	const labelW = words(ctx, t("saga.week.best"), (x + width - bestW - gap) * PX, ly * PX, label) / PX;
	const right = x + width - bestW - labelW - 2 * gap;
	dashed(right - 5 * num("--qp-surface-week-strip-dash"), right, ly - size * 0.3);
}

/** A card's stats (records, XP, day streak; a week's days, records, XP), each left out at 0 or unknown: its number
 * over its glyph and its caps word, divided by hairlines. */
async function cardStats(ctx, look, stats, x, y, width, size) {
	const shown = stats.filter((s) => s.value);
	if (!shown.length) return;
	const col = width / shown.length;
	for (const [i, s] of shown.entries()) {
		const cx = x + col * i + col / 2;
		words(ctx, s.text, cx * PX, (y + size * 0.86) * PX, {size, weight: 700, family: FIGURES, colour: look.words,
			align: "center"});
		const label = measure(ctx, s.label, {size: 12, weight: 600, caps: true, tracking: 0.12}) / PX;
		const glyph = await art(s.glyph, look.gold);
		const lineX = cx - (label + 18) / 2;
		if (glyph) ctx.drawImage(glyph, lineX * PX, (y + size + 2) * PX, 14 * PX, 14 * PX);
		words(ctx, s.label, (lineX + 18) * PX, (y + size + 13) * PX, {size: 12, weight: 600, colour: look.words2,
			caps: true, tracking: 0.12});
		if (i > 0) {
			ctx.fillStyle = look.rule;
			ctx.fillRect((x + col * i) * PX, y * PX, PX / 2, (size + 18) * PX);
		}
	}
}

/** The app's mark and questpump.com, centred at `y` (the words' baseline). */
async function siteLine(ctx, look, w, y) {
	const text = t("card.site");
	const size = 13;
	const mark = num("--qp-surface-saga-card-site-mark");
	const tw = measure(ctx, text, {size, weight: 500}) / PX;
	const x = (w - (mark + 8 + tw)) / 2;
	const glyph = await art("app_mark", look.gold);
	if (glyph) ctx.drawImage(glyph, x * PX, (y - mark + 3) * PX, mark * PX, mark * PX);
	words(ctx, text, (x + mark + 8) * PX, y * PX, {size, weight: 500, colour: look.words2});
}

/** A day's word as its card's tag: Best day ever filled gold with the star, else the word outlined in its quality's tone
 * (its ink twin on the parchment, its own on the sticker); none before the gate. */
function dayTag(quality) {
	const label = dayWord(quality);
	if (!label) return null;
	const best = quality === "legendary";
	return {label, tone: (look) => (best ? token("--qp-color-gold") : look.quality(quality)),
		filled: best, star: best};
}

/** The streak card's tag on a milestone count (SlantTag's milestone, slice 45): Milestone filled gold with goldInk words
 * and no star, outlined on the sticker. */
function milestoneTag() {
	return {label: t("share.streak.tag"), tone: () => token("--qp-color-gold"), filled: true, star: false};
}

/** XP gained as the cards' stats write it (+45). */
const SIGNED = new Intl.NumberFormat(undefined, {signDisplay: "always"});

/** The Saga card's model from a day of the `log` topic: its head (My saga and the chapter line), number and unit, the
 * day's word as its tag, knight, line, strip (the day's bars, or the week's days) and stats; the week card's for `week`.
 * The sticker's line is the chapter line. */
function sagaModel(card) {
	const d = card.day;
	const name = (move) => card.ctx.names.get(move) ?? {name: "", key: null};
	const still = (key) => (key ? () => pose(key) : null);
	const line = (hl, today) => {
		if (!hl) return null;
		const move = name(hl.move);
		if (hl.kind === "unlock") {
			return {caps: t(today ? "saga.unlocked.today" : "saga.unlocked"), name: move.name,
				quality: hl.quality, word: card.ctx.qualities[hl.quality - 1] ?? ""};
		}
		return {caps: t(today ? "saga.record.today" : "saga.record"), name: move.name,
			value: setWords(hl.best, hl.hold, move.load === "each")};
	};
	if (card.kind === "week") {
		const w = d.week;
		return {
			title: t("card.saga"), chapter: weekOf(w.start), number: w.moves, unit: movesLabel(w.moves, "week"),
			tag: null, knight: still(w.hero ? name(w.hero.move).key : null), line: line(w.highlight, false), week: w,
			stats: [{value: w.active, text: number(w.active), label: t("card.days"), glyph: "action_log"},
				{value: w.records, text: number(w.records), label: t("card.records"), glyph: "reward_record"},
				{value: w.xp, text: w.xp ? SIGNED.format(w.xp) : "", label: t("pump.today.xp"), glyph: "reward_xp"}],
			spoken: `${weekOf(w.start)}: ${t("common.moves.count", {count: w.moves})}`,
		};
	}
	const today = d.ago === 0;
	const chapter = `${chapterWords(d.chapter)} · ${shortDate(d.day)}`;
	return {
		title: t("card.saga"), chapter, number: d.moves, unit: movesLabel(d.moves, today ? "today" : "day"),
		tag: dayTag(d.quality), knight: still(d.hero ? name(d.hero.move).key : null), line: line(d.highlight, today),
		bars: d.bars, buckets: d.buckets,
		stats: [{value: d.records.length, text: number(d.records.length), label: t("card.records"), glyph: "reward_record"},
			{value: d.xp, text: d.xp ? SIGNED.format(d.xp) : "", label: t("pump.today.xp"), glyph: "reward_xp"},
			{value: d.streak, text: d.streak ? number(d.streak) : "", label: t("pump.home.streak.label"),
				glyph: "action_log"}],
		sticker: chapter, spoken: heroSpoken(d),
	};
}

/** A list's words as a screen reader hears them ("30-day streak, A quest a day., Tue 14 Oct"). */
const SAID = new Intl.ListFormat(LOCALE, {type: "conjunction", style: "narrow"});

/** The streak sticker's model (slice 45; surface.StreakCard): the streak's days (its whole years from 730 days, with
 * Year streak) with Day streak and, on a milestone count, the Milestone tag; its line A quest a day. */
function streakModel(card) {
	const years = streakYears(card.days);
	return {
		number: years ?? card.days, unit: t(years === null ? "pump.home.streak.label" : "share.streak.years.label"),
		tag: card.milestone ? milestoneTag() : null, sticker: t("pump.home.streak.line"),
	};
}

/** A card's model: the streak sticker's, or the Saga card's of a day or a week. */
function familyModel(card) {
	return card.kind === "streak" ? streakModel(card) : sagaModel(card);
}

/** The least room the strip and the week's bars are drawn in. */
const FOOT_LEAST = 40;
/** A reel shot's Quest Bill line's height in the highlight slot: the gold rule and two rows of words at SagaReel's
 * lineSize. */
const BILL_LINE = 48;

/** The Saga card's family laid out (the Saga card, the week card and the day reel's frame): a model's head, number with
 * its unit and tag, knight, line, foot (the day's strip or the week's bars) and stats, in a format and a style, each part's place and its drawing, so a card draws them all and the reel each
 * as a layer of its own. The foot is laid from the bottom and the head from the top, and the number (with the knight
 * and its unit) shrinks below its token where the format leaves the foot too little room. The knight stands under the
 * tag right of the number (on Night cut out, only its mask's clear rim under the number; on the parchment in its well),
 * never under a word: the line beside it is cut before it, and the foot starts below its feet. `reel` lays out the day
 * reel's frame: the story with the reel's insets (every word `reelSafeTop` from the top, `reelSafeBottom` from the
 * bottom and `reelSafeRight` from the right edge, where a Story's, a Reel's and a TikTok's own buttons lie), the knight's
 * slot there for the films whatever the day's hero, and the line's slot `reel.lineH` high for the shots' words. */
async function sagaLayout(m, format, style, reel = null) {
	await sagaFonts();
	const look = sagaLook(style);
	const scratch = document.createElement("canvas").getContext("2d");
	const w = num("--qp-surface-saga-card-width");
	const ht = num(`--qp-surface-saga-card-${format}-height`);
	const margin = num("--qp-surface-saga-card-margin");
	const story = format === "story";
	const square = format === "square";
	const right = reel ? Math.max(margin, num("--qp-surface-saga-card-reel-safe-right")) : margin;
	const top = reel ? num("--qp-surface-saga-card-reel-safe-top")
		: story ? num("--qp-surface-saga-card-story-safe-top") : margin * (square ? 0.75 : 1);
	const bottom = ht - (reel ? num("--qp-surface-saga-card-reel-safe-bottom")
		: story ? num("--qp-surface-saga-card-story-safe-bottom") : margin * (square ? 0.75 : 1));
	// the foot, from the bottom: questpump.com's baseline with its mark and descenders above the bottom (the story's safe
	// zone), the stats or the name over it
	const site = bottom - 5;
	const statSize = num("--qp-surface-saga-card-stat-size") * (square ? 0.8 : 1);
	const statsTop = site - 20 - (statSize + 18);
	// the head: the mark and the title, the chapter line (a date) at the right, shrunk to the room the title leaves it
	const markSize = num("--qp-surface-saga-card-mark-size");
	const titleStyle = {size: num("--qp-surface-saga-card-title-size"), weight: 700, family: "wordmark", caps: true,
		tracking: 0.16};
	const titleW = measure(scratch, m.title, titleStyle) / PX;
	const chapterStyle = {size: 11, weight: 600, caps: true, tracking: 0.12};
	const wordsX = margin + markSize + 8;
	const chapterRoom = w - right - wordsX - titleW - 12;
	const chapterSize = 11 * Math.min(1, chapterRoom / Math.max(1, measure(scratch, m.chapter, chapterStyle) / PX));
	const headH = markSize;
	// the number at its token's size, less where the foot would get too little room
	const numberTop = top + headH + (square ? 4 : 10);
	const lineH = reel ? reel.lineH : m.line ? 40 : 0;
	const footLeast = square ? 58 : 72;
	const token_ = num(`--qp-surface-saga-card-number-${format}`);
	const scale = Math.min(1, (statsTop - 12 - footLeast - lineH - numberTop) / 0.84 / token_);
	const text = number(m.number);
	const numberStyle = (size) => ({size, weight: 700, family: FIGURES, tracking: -0.03});
	const widthAt = (size) => measure(scratch, text, numberStyle(size)) / PX;
	const unitSize = num(`--qp-surface-saga-card-moves-${format}`) * scale;
	const unitStyle = {size: unitSize, weight: 700, family: FIGURES, tracking: 0.02};
	// its unit's widest word and the tag beside the number stay inside the margin: the number shrinks for them only as
	// far as it must (three digits and more), never under numberLeastFraction of its size
	const unitW = Math.max(...m.unit.toLocaleUpperCase(LOCALE).split(" ").map((word) => measure(scratch, word, unitStyle) / PX));
	const tagW = m.tag ? tagWidth(scratch, m.tag.label, m.tag.star) : 0;
	const roomW = w - margin - right - 8 - Math.max(unitW, tagW);
	const numberSize = token_ * scale * Math.max(numberLeast(), Math.min(1, roomW / widthAt(token_ * scale)));
	const base = numberTop + numberSize * 0.8;
	const numberW = widthAt(numberSize);
	const side = margin + numberW + 8;
	scratch.font = font(700, unitSize, FIGURES);
	const unitLines = wrap(scratch, m.unit.toLocaleUpperCase(LOCALE), (w - right - side) * PX);
	const unitY = numberTop + numberSize * 0.12 + unitLines.length * unitSize * 0.86;
	const tagTop = unitY + 12;
	const tagBottom = m.tag ? tagTop + 24 : unitY;
	// the line under the number (under its grouping comma's foot, which falls below the digits)
	scratch.font = font(700, numberSize, FIGURES);
	const descent = Math.max(0, scratch.measureText(text).actualBoundingBoxDescent) / PX;
	const lineTop = base + descent + 18;
	// the knight under the tag, right of the number, the foot below it (its feet above the strip's totals): cut out on
	// Night (its mask's clear rim may lie under the number's right, the figure beside it), on the parchment in its well
	// right of the number; its size what that room leaves, and none in a room too small for it
	const figureToken = num(`--qp-surface-saga-card-figure-${format}`) * scale;
	const knightTop = tagBottom + 8;
	const rim = look.night ? (1 - num("--qp-surface-saga-hero-mask-fraction")) / 2 : 0;
	const inset = look.night ? margin / 3 : margin;
	const across = w - inset - (margin + numberW + 12);
	const size = reel || m.knight ? Math.min(figureToken, across / (1 - rim), statsTop - 12 - footLeast - knightTop) : 0;
	const drawn = size >= 48;
	const knightX = w - inset - size;
	// the line under the number, cut with "…" before the knight where it stands beside it
	const beside = drawn && knightTop + size > lineTop - 16;
	const lineRight = beside ? knightX + rim * size - 12 : w - right;
	// the foot in the room between the line (or the knight) and the stats
	const footTop = Math.max(lineTop + lineH, drawn ? knightTop + size : 0);
	const footRoom = statsTop - 12 - footTop;
	const footH = Math.min(story ? 104 : 84, footRoom);
	const footY = footTop + (footRoom - footH) / 2;
	const parts = {
		ground(ctx) {
			sagaGround(ctx, look, w * PX, ht * PX, format);
		},
		/** The mark and the title, the chapter line at the right. */
		async head(ctx) {
			const mark = await art("app_mark", look.gold);
			if (mark) ctx.drawImage(mark, margin * PX, top * PX, markSize * PX, markSize * PX);
			words(ctx, m.title, wordsX * PX, (top + markSize / 2) * PX, {...titleStyle,
				colour: look.night ? token("--qp-color-gold-hi") : look.gold, baseline: "middle"});
			words(ctx, m.chapter, (w - right) * PX, (top + markSize / 2) * PX, {...chapterStyle, size: chapterSize,
				colour: look.words2, align: "right", baseline: "middle"});
		},
		/** The number, `value` (the reel's count on its way to the model's), scaled by `pop` about its foot (the digit
		 * pop's): a count keeps the whole number's right edge, so it never moves as it counts. */
		number(ctx, value = m.number, pop = 1) {
			ctx.save();
			ctx.translate(margin * PX, base * PX);
			ctx.scale(pop, pop);
			ctx.translate(-margin * PX, -base * PX);
			ctx.font = font(700, numberSize, FIGURES);
			ctx.letterSpacing = `${-0.03 * numberSize * PX}px`;
			if (look.night) {
				const g = ctx.createLinearGradient(0, numberTop * PX, 0, base * PX);
				for (const [o, c] of stops(token("--qp-surface-saga-hero-number-stops"))) g.addColorStop(Math.min(1, o), c);
				ctx.fillStyle = g;
			} else {
				ctx.fillStyle = look.gold;
			}
			ctx.textAlign = "right";
			ctx.fillText(number(value), (margin - numberSize * 0.03 + numberW) * PX, base * PX);
			ctx.restore();
		},
		/** The number's unit and its tag beside it. */
		async aside(ctx) {
			let y = numberTop + numberSize * 0.12;
			for (const l of unitLines) {
				y += unitSize * 0.86;
				words(ctx, l, side * PX, y * PX, {...unitStyle, colour: look.words});
			}
			if (m.tag) await slantTag(ctx, side, tagTop, m.tag.label, m.tag.tone(look), m.tag.filled, m.tag.star);
		},
		/** The parchment's well under the knight (the reel's films are laid over it). */
		well(ctx) {
			if (drawn) well(ctx, look, knightX, knightTop, size);
		},
		/** The knight's still: the move that stands for the card. */
		async figure(ctx) {
			if (drawn && m.knight) await knight(ctx, look, await m.knight(), knightX, knightTop, size);
		},
		/** The line under the number: the card's highlight (`slot` none or a highlight), or a reel shot's Quest Bill
		 * line (`slot` a bill line). */
		async line(ctx, slot = null) {
			if (slot?.kind === "bill") {
				await billLine(ctx, look, slot, margin, lineTop, lineRight);
				return;
			}
			if (m.line) highlightLine(ctx, look, m.line, margin, lineTop, lineRight);
		},
		/** The foot: the day's strip (its bars grown by `grow`, the reel's reveal) or the week's bars. */
		async foot(ctx, grow = null) {
			if (footH < FOOT_LEAST) return;
			if (m.week) cardWeek(ctx, look, m.week, margin, footY, w - margin - right, footH);
			else cardStrip(ctx, look, m.bars, m.buckets, margin, footY, w - margin - right, footH, grow);
		},
		/** The stats. */
		async stats(ctx) {
			await cardStats(ctx, look, m.stats, margin, statsTop, w - margin - right, statSize);
		},
		async site(ctx) {
			await siteLine(ctx, look, w, site);
		},
	};
	return {
		w, ht, look, parts,
		knight: drawn ? {x: knightX, y: knightTop, size} : null,
		// where the reel's changing parts stand, in points: the number with its pop's growth, the line's slot, the foot
		number: {top: numberTop, base, left: margin, width: numberW, size: numberSize, descent},
		line: {top: lineTop, height: lineH},
		foot: {top: footY, height: footH},
	};
}

/** A card's highlight line under the number (the Saga card's): the gold rule; an unlock's caps in gold, a record's in
 * text2 (ink2 on parchment); the move's name, cut with "…" before `right`, then an unlock's gem and its quality's word,
 * or a record's best. */
function highlightLine(ctx, look, line, x, top, right) {
	let y = top;
	const ruleW = num("--qp-surface-saga-hero-rule-width");
	const gem = num("--qp-layout-space10");
	const lead = line.quality ? num("--qp-layout-space8") + gem + num("--qp-layout-space4") : 10;
	ctx.fillStyle = look.gold;
	ctx.fillRect(x * PX, (y - 4) * PX, ruleW * PX, num("--qp-surface-saga-hero-rule-height") * PX);
	words(ctx, line.caps, (x + ruleW + 8) * PX, y * PX, {size: 12, weight: 600,
		colour: line.quality ? look.gold : look.words2, caps: true, tracking: 0.12});
	y += 22;
	const after = line.quality ? measure(ctx, line.word, {size: 12, weight: 700, caps: true, tracking: 0.1})
		: measure(ctx, line.value, {size: 18, weight: 700, family: FIGURES});
	const name = fitted(ctx, line.name, right - x - after / PX - lead, {size: 16, weight: 600});
	const nameW = words(ctx, name, x * PX, y * PX, {size: 16, weight: 600, colour: look.words}) / PX;
	if (line.quality) {
		const tone = look.quality(qualityName(line.quality));
		diamond(ctx, tone, x + nameW + num("--qp-layout-space8") + gem / 2, y - 16 * 0.35, gem);
		words(ctx, line.word, (x + nameW + lead) * PX, y * PX, {size: 12, weight: 700, colour: tone, caps: true,
			tracking: 0.1});
	} else {
		words(ctx, line.value, (x + nameW + 10) * PX, y * PX, {size: 18, weight: 700, family: FIGURES,
			colour: look.gold});
	}
}

/** A day's Quest Bill line as a reel shot shows it in the highlight slot (docs/ui.md, "The share card": the bill's
 * words at SagaReel's lineSize, no dotted leader): the gold rule; its count (none for a move's name alone, the end of a
 * day with no record and no unlock) and, QuestBill's countGap after it, its name in caps, cut with "…" before `right`;
 * then New {value} in gold after the small seal (drawn as is, SagaReel's recordSeal wide), or Unlocked · {quality} in
 * its tone, and its reps or seconds. */
async function billLine(ctx, look, slot, x, top, right) {
	const caps = {size: num("--qp-surface-saga-reel-line-size"), weight: 700, family: FIGURES, caps: true};
	const sealSize = num("--qp-surface-saga-reel-record-seal");
	ctx.fillStyle = look.gold;
	ctx.fillRect(x * PX, (top - 4) * PX, num("--qp-surface-saga-hero-rule-width") * PX,
		num("--qp-surface-saga-hero-rule-height") * PX);
	let y = top + 14;
	// the name after the count and its gap, at the rule's start without one
	const countW = slot.count == null ? 0
		: words(ctx, number(slot.count), x * PX, y * PX, {...caps, colour: look.words2}) / PX
			+ num("--qp-surface-quest-bill-count-gap");
	const named = {...caps, tracking: num("--qp-surface-quest-bill-name-tracking-fraction")};
	const name = wholeFitted(ctx, slot.name, right - x - countW, named);
	words(ctx, name, (x + countW) * PX, y * PX, {...named, colour: look.words});
	y += 24;
	let at = x;
	if (slot.record) {
		const seal = await art("seal", token("--qp-surface-quest-bill-wax"));
		if (seal) ctx.drawImage(seal, at * PX, (y - sealSize + 1) * PX, sealSize * PX, sealSize * PX);
		at += sealSize + 4;
		at += words(ctx, t("bill.new", {value: slot.record}), at * PX, y * PX, {...caps, colour: look.gold}) / PX + 6;
	} else if (slot.unlock) {
		at += words(ctx, t("bill.unlocked", {quality: slot.unlock.word}), at * PX, y * PX, {...caps,
			colour: look.quality(qualityName(slot.unlock.quality))}) / PX + 6;
	}
	if (slot.total) words(ctx, slot.total, at * PX, y * PX, {...caps, colour: look.words});
}

/** Draws a card of the Saga's family (a day's or a week's Saga card) in a format and a style. */
async function drawSaga(card, format, style) {
	const laid = await sagaLayout(familyModel(card), format, style);
	const canvas = document.createElement("canvas");
	canvas.width = laid.w * PX;
	canvas.height = laid.ht * PX;
	const ctx = canvas.getContext("2d");
	const p = laid.parts;
	p.ground(ctx);
	await p.site(ctx);
	await p.stats(ctx);
	await p.head(ctx);
	await p.figure(ctx);
	p.number(ctx);
	await p.aside(ctx);
	await p.line(ctx);
	await p.foot(ctx);
	return canvas;
}

/** Draws a sticker (the day's Saga card's and the streak card's; SagaCard): on a clear ground the number in SagaHero's
 * gold with its unit beside it and its tag outlined under that (never filled; Best day ever a gold outline with its
 * star, Milestone a gold outline), then the sticker's line (the day's chapter line; A quest a day.), every word light
 * (the number gold) over its dark outline and shadow, so it reads on any picture. The number shrinks to leave its unit
 * and tag room (three digits and more); the sticker is as high as its words. */
async function drawSticker(card) {
	await sagaFonts();
	const m = familyModel(card);
	const w = num("--qp-surface-saga-card-width");
	const margin = num("--qp-surface-saga-card-margin");
	const halo = {colour: token("--qp-surface-saga-card-sticker-stroke-colour"),
		width: num("--qp-surface-saga-card-sticker-stroke") * PX, glow: shadow(token("--qp-surface-saga-card-sticker-shadow"))};
	const light = token("--qp-color-text");
	const unitSize = num("--qp-surface-saga-card-moves-post");
	const unitWords = {size: unitSize, weight: 700, family: FIGURES, tracking: 0.02};
	const lineWords = {size: 15, weight: 600, caps: true, tracking: 0.12};
	const scratch = document.createElement("canvas").getContext("2d");
	const units = m.unit.toLocaleUpperCase(LOCALE).split(" ");
	const side = Math.max(...units.map((l) => measure(scratch, l, unitWords) / PX),
		m.tag ? tagWidth(scratch, m.tag.label, m.tag.star) : 0);
	const text = number(m.number);
	const numberToken = num("--qp-surface-saga-card-number-post");
	const widthAt = (size) => measure(scratch, text, {size, weight: 700, family: FIGURES, tracking: -0.03}) / PX;
	const gap = 12;
	const fit = (w - 2 * margin - gap - side) / widthAt(numberToken);
	const numberSize = numberToken * Math.max(numberLeast(), Math.min(1, fit));
	const top = margin;
	const base = top + numberSize * 0.8;
	let unitY = top + numberSize * 0.12;
	const unitLines = units.map(() => {
		unitY += unitSize * 0.86;
		return unitY;
	});
	const tagTop = unitY + 12;
	const blockBottom = Math.max(base, m.tag ? tagTop + 16 + 2 * num("--qp-surface-slant-tag-pad-v") : unitY);
	const lineY = blockBottom + 10 + lineWords.size;
	const canvas = document.createElement("canvas");
	canvas.width = w * PX;
	canvas.height = Math.ceil((lineY + margin) * PX);
	const ctx = canvas.getContext("2d");
	const gold = ctx.createLinearGradient(0, top * PX, 0, base * PX);
	for (const [o, c] of stops(token("--qp-surface-saga-hero-number-stops"))) gold.addColorStop(Math.min(1, o), c);
	const nw = words(ctx, text, (margin - numberSize * 0.03) * PX, base * PX, {size: numberSize, weight: 700,
		family: FIGURES, colour: gold, tracking: -0.03, halo}) / PX;
	const x = margin + nw + gap;
	units.forEach((l, i) => words(ctx, l, x * PX, unitLines[i] * PX, {...unitWords, colour: light, halo}));
	// the tag outlined in its tone on the iron (Night's tones, which read on a picture)
	if (m.tag) await slantTag(ctx, x, tagTop, m.tag.label, m.tag.tone(sagaLook("night")), false, m.tag.star, halo);
	words(ctx, m.sticker, margin * PX, lineY * PX, {...lineWords, colour: light, halo});
	return canvas;
}

/** Draws the Quest Bill of a day in a format and a style: a receipt with torn edges on its iron ground (the light
 * paper for Parchment, the card inside the gold band for Gilded, the dark paper with a gold edge for Night): The Quest
 * Bill, the day's span, a line a move in desk/saga.rs' order (the day's records and unlocks first, then the most done;
 * at most four, then the more line, + in the counts' column and the rest's words where the names start; its reps for
 * a move dosed in reps, blank while none of its sets has a count, else its seconds), the total and the day's XP (none
 * where no sheet knows it, the barcode then closing up under the number unless the seal's word needs the room),
 * the wax seal turned a 30th of a turn back with My best day or the day's word in the record's red under it (none
 * before the gate), the barcode strip between the day's turns and the foot, My game day, a bar a pump with the mark and
 * questpump.com at its end; every Barlow Condensed word in FIGURES, so its digits are tabular. */
async function drawBill(card, format, style) {
	await sagaFonts();
	const d = card.day;
	const look = sagaLook(style);
	const night = look.night;
	const w = num("--qp-surface-saga-card-width");
	const ht = num(`--qp-surface-saga-card-${format}-height`);
	const canvas = document.createElement("canvas");
	canvas.width = w * PX;
	canvas.height = ht * PX;
	const ctx = canvas.getContext("2d");
	ctx.fillStyle = token("--qp-color-page");
	ctx.fillRect(0, 0, canvas.width, canvas.height);
	radial(ctx, token("--qp-surface-quest-bill-ground"), 0, 0, canvas.width, canvas.height);
	const story = format === "story";
	const square = format === "square";
	const margin = num("--qp-surface-quest-bill-paper-margin") * (square ? 0.6 : 1);
	const pad = num("--qp-surface-quest-bill-padding");
	// the post and the square keep half the margin above and below; the square draws its words at a smaller size
	const top = story ? num("--qp-surface-saga-card-story-safe-top") - pad : margin / 2;
	const bottom = story ? ht - num("--qp-surface-saga-card-story-safe-bottom") + pad : ht - margin / 2;
	const paper = {x: margin, y: top, w: w - 2 * margin, h: bottom - top};
	const k = square ? 0.84 : 1;
	const ink = night ? token("--qp-surface-quest-bill-night-ink") : token("--qp-surface-quest-bill-ink");
	const ink2 = night ? token("--qp-surface-quest-bill-night-ink2") : token("--qp-surface-quest-bill-ink2");
	const red = night ? token("--qp-surface-quest-bill-night-record") : token("--qp-surface-quest-bill-record");
	ctx.save();
	// the paper turned a little, its shadow, its torn top and foot
	ctx.translate((paper.x + paper.w / 2) * PX, (paper.y + paper.h / 2) * PX);
	ctx.rotate(-num("--qp-surface-quest-bill-turn-fraction") * 2 * Math.PI);
	ctx.translate(-(paper.x + paper.w / 2) * PX, -(paper.y + paper.h / 2) * PX);
	const tooth = num("--qp-surface-quest-bill-tooth-width");
	const depth = num("--qp-surface-quest-bill-tooth-depth");
	const torn = () => {
		ctx.beginPath();
		const teeth = Math.round(paper.w / tooth);
		const step = paper.w / teeth;
		ctx.moveTo(paper.x * PX, (paper.y + depth) * PX);
		for (let i = 0; i < teeth; i += 1) {
			ctx.lineTo((paper.x + step * (i + 0.5)) * PX, paper.y * PX);
			ctx.lineTo((paper.x + step * (i + 1)) * PX, (paper.y + depth) * PX);
		}
		ctx.lineTo((paper.x + paper.w) * PX, (paper.y + paper.h - depth) * PX);
		for (let i = teeth; i > 0; i -= 1) {
			ctx.lineTo((paper.x + step * (i - 0.5)) * PX, (paper.y + paper.h) * PX);
			ctx.lineTo((paper.x + step * (i - 1)) * PX, (paper.y + paper.h - depth) * PX);
		}
		ctx.closePath();
	};
	const drop = shadow(token("--qp-surface-quest-bill-shadow"));
	ctx.save();
	ctx.shadowColor = drop.colour;
	ctx.shadowBlur = drop.blur * PX;
	ctx.shadowOffsetY = drop.y * PX;
	torn();
	const fill = ctx.createLinearGradient(0, paper.y * PX, 0, (paper.y + paper.h) * PX);
	for (const [o, c] of stops(token(night ? "--qp-surface-quest-bill-night-paper-stops"
		: "--qp-surface-quest-bill-paper-stops"))) fill.addColorStop(o, c);
	ctx.fillStyle = fill;
	ctx.fill();
	ctx.restore();
	ctx.save();
	torn();
	ctx.clip();
	if (!night) {
		radial(ctx, token("--qp-surface-quest-bill-paper-light"), paper.x * PX, paper.y * PX, paper.w * PX, paper.h * PX);
		radial(ctx, token("--qp-surface-quest-bill-paper-shade"), paper.x * PX, paper.y * PX, paper.w * PX, paper.h * PX);
	}
	ctx.restore();
	if (night) {
		torn();
		ctx.strokeStyle = token("--qp-surface-quest-bill-night-edge-color");
		ctx.lineWidth = num("--qp-surface-quest-bill-night-edge-width") * PX;
		ctx.stroke();
	}
	// the words in the paper's own space (scaled by k about its corner)
	ctx.save();
	ctx.translate(paper.x * PX, paper.y * PX);
	ctx.scale(k, k);
	ctx.translate(-paper.x * PX, -paper.y * PX);
	const space = {x: paper.x, y: paper.y, w: paper.w / k, h: paper.h / k};
	const x = space.x + pad;
	const inner = space.w - 2 * pad;
	const mid = space.x + space.w / 2;
	let y = space.y + pad + depth + 18;
	words(ctx, t("bill.title"), mid * PX, y * PX, {size: num("--qp-surface-quest-bill-title-size"), weight: 700,
		family: "wordmark", colour: ink, align: "center", caps: true});
	y += 22;
	words(ctx, t("bill.span", {date: shortDate(d.day), from: TURN, to: TURN}), mid * PX, y * PX,
		{size: num("--qp-surface-quest-bill-date-size"), weight: 700, family: FIGURES, colour: ink2, align: "center",
			caps: true, tracking: num("--qp-surface-quest-bill-date-tracking-fraction")});
	y += 12;
	const ruleColour = night ? token("--qp-surface-share-card-night-rule") : token("--qp-surface-quest-bill-rule-color");
	const dashed = (ly) => {
		ctx.save();
		ctx.strokeStyle = ruleColour;
		ctx.lineWidth = num("--qp-surface-quest-bill-rule-width") * PX;
		ctx.setLineDash([num("--qp-surface-quest-bill-rule-dash") * PX, num("--qp-surface-quest-bill-rule-dash") * PX]);
		ctx.beginPath();
		ctx.moveTo(x * PX, ly * PX);
		ctx.lineTo((x + inner) * PX, ly * PX);
		ctx.stroke();
		ctx.restore();
	};
	dashed(y);
	// the foot: a line as high as the mark, questpump.com whole after the mark at its end, My game day, a bar a pump in
	// the room before them, smaller down to footLeastFraction of its size where it has none, then cut with an ellipsis
	// (the iPhone's), its words on one baseline
	const footSize = num("--qp-surface-quest-bill-foot-size");
	const siteMark = num("--qp-surface-quest-bill-site-mark");
	const site = t("card.site");
	const siteWords = {size: footSize, weight: 600, colour: ink2};
	const siteW = measure(ctx, site, siteWords) / PX;
	const [space4, space6, space8] = [num("--qp-layout-space4"), num("--qp-layout-space6"), num("--qp-layout-space8")];
	const footRoom = inner - siteW - space6 - siteMark - space6 - space4 - space6;
	const stripFull = {size: footSize, weight: 700, family: FIGURES, caps: true,
		tracking: num("--qp-surface-quest-bill-foot-tracking-fraction")};
	const fit = footRoom / (measure(ctx, t("bill.strip"), stripFull) / PX);
	const footWords = {...stripFull, colour: ink2,
		size: footSize * Math.max(num("--qp-surface-quest-bill-foot-least-fraction"), Math.min(1, fit))};
	const strip = fitted(ctx, t("bill.strip"), footRoom, footWords);
	const footLine = Math.max(siteMark, footSize * LINE);
	// the lines: as many as the paper holds, at most four, then the rest in one line
	const lineSize = num("--qp-surface-quest-bill-line-size");
	const numberSize = num("--qp-surface-quest-bill-number-size");
	const below = 12 + 22 + numberSize * 0.86 + 20 + 22 + num("--qp-surface-quest-bill-barcode-height") + space8
		+ footLine + pad + depth;
	const fits = Math.floor((space.y + space.h - below - y - (lineSize + 6)) / (lineSize + 8));
	const most = Math.max(1, Math.min(num("--qp-surface-quest-bill-line-count"), fits));
	const lines = d.bill.slice(0, most);
	const rest = d.bill.slice(most);
	const seal = await art("seal", token("--qp-surface-quest-bill-wax"));
	// the counts' column: as wide as the widest count shown, so every name starts at one place
	const countWords = {size: lineSize, weight: 700, family: FIGURES};
	const column = Math.max(0, ...lines.map((l) => measure(ctx, number(l.count), countWords) / PX));
	const leader = (from, to, ly) => {
		ctx.save();
		ctx.fillStyle = night ? ink2 : token("--qp-surface-quest-bill-leader-color");
		const dot = num("--qp-surface-quest-bill-leader-dot") * PX;
		for (let lx = from + 4; lx < to - 4; lx += 5) ctx.fillRect(lx * PX, (ly - 4) * PX, dot, dot);
		ctx.restore();
	};
	for (const l of lines) {
		y += lineSize + 8;
		const move = card.ctx.names.get(l.move) ?? {name: "", load: null};
		const lineWords = {size: lineSize, weight: 700, family: FIGURES, caps: true};
		const rec = d.records.find((r) => r.move === l.move);
		const unlock = d.unlocks.find((u) => u.move === l.move);
		const sealSize = num("--qp-surface-quest-bill-record-seal");
		const tag = rec ? t("bill.new", {value: setWords(rec.best, rec.hold, move.load === "each")})
			: unlock ? t("bill.unlocked", {quality: card.ctx.qualities[unlock.quality - 1] ?? ""}) : "";
		const tagW = tag ? measure(ctx, tag, lineWords) / PX + 6 + (rec ? sealSize + 4 : 0) : 0;
		// a rep move's reps, any other's time; blank for a rep move none of whose sets has a count
		const total = l.reps != null ? number(l.reps) : l.seconds != null ? clock(l.seconds) : "";
		const tw = measure(ctx, total, {size: lineSize, weight: 700, family: FIGURES}) / PX;
		words(ctx, number(l.count), (x + column) * PX, y * PX, {...countWords, colour: ink2, align: "right"});
		let lx = x + column + num("--qp-surface-quest-bill-count-gap");
		const named = {...lineWords, tracking: num("--qp-surface-quest-bill-name-tracking-fraction")};
		const name = fitted(ctx, move.name, x + inner - tw - 16 - tagW - lx, named);
		lx += words(ctx, name, lx * PX, y * PX, {...named, colour: ink}) / PX + 6;
		if (rec) {
			if (seal) ctx.drawImage(seal, lx * PX, (y - sealSize + 1) * PX, sealSize * PX, sealSize * PX);
			lx += sealSize + 4;
			lx += words(ctx, tag, lx * PX, y * PX, {...lineWords, colour: red}) / PX + 6;
		} else if (unlock) {
			lx += words(ctx, tag, lx * PX, y * PX, {...lineWords, colour: look.quality(qualityName(unlock.quality))}) / PX
				+ 6;
		}
		leader(lx, x + inner - tw - 4, y);
		words(ctx, total, (x + inner) * PX, y * PX, {size: lineSize, weight: 700, family: FIGURES, colour: ink,
			align: "right"});
	}
	// the more line: + at the start of the counts' column, its words where the names start, its moves at the end
	if (rest.length) {
		y += lineSize + 6;
		const more = {size: num("--qp-surface-quest-bill-more-size"), weight: 700, family: FIGURES, caps: true,
			tracking: num("--qp-surface-quest-bill-more-tracking-fraction")};
		const restMoves = rest.reduce((a, l) => a + l.count, 0);
		const start = x + column + num("--qp-surface-quest-bill-count-gap");
		words(ctx, MORE_MARK, x * PX, y * PX, {...more, colour: ink2});
		const lw = words(ctx, t("bill.more", {count: rest.length}), start * PX, y * PX, {...more, colour: ink2}) / PX;
		const movesText = t("common.moves.count", {count: restMoves});
		const tw = measure(ctx, movesText, more) / PX;
		leader(start + lw, x + inner - tw - 4, y);
		words(ctx, movesText, (x + inner) * PX, y * PX, {...more, colour: ink2, align: "right"});
	}
	// the double rule, the total, the seal
	y += 12;
	ctx.fillStyle = ink;
	ctx.fillRect(x * PX, y * PX, inner * PX, num("--qp-surface-quest-bill-total-rule-top") * PX);
	ctx.fillRect(x * PX, (y + num("--qp-surface-quest-bill-total-rule-top") + 2) * PX, inner * PX,
		num("--qp-surface-quest-bill-total-rule-bottom") * PX);
	y += 22;
	words(ctx, t("bill.total"), x * PX, y * PX, {size: num("--qp-surface-quest-bill-total-size"), weight: 700,
		family: "wordmark", colour: ink, caps: true, tracking: 0.04});
	const base = y + numberSize * 0.86;
	// the number shrinks where it and Moves would run under the seal (three digits and more), to numberLeastFraction
	const movesWords = {size: num("--qp-surface-quest-bill-moves-word-size"), weight: 700, family: "wordmark",
		caps: true};
	const room = inner - num("--qp-surface-quest-bill-seal-size") - 8
		- measure(ctx, movesLabel(d.moves, "day"), movesWords) / PX - 8;
	const full = measure(ctx, number(d.moves), {size: numberSize, weight: 700, family: FIGURES, tracking: -0.02}) / PX;
	const totalSize = numberSize * Math.max(numberLeast(), Math.min(1, room / full));
	const nw = words(ctx, number(d.moves), x * PX, base * PX, {size: totalSize, weight: 700, family: FIGURES,
		colour: ink, tracking: -0.02}) / PX;
	words(ctx, movesLabel(d.moves, "day"), (x + nw + 8) * PX, base * PX, {...movesWords, colour: ink});
	// the day's XP on its own line under the number; without it the line goes, and the barcode comes up under the
	// number unless the seal's word stands there (the room `below` counts holds either)
	const sealWord = d.best ? t("bill.seal.best") : dayWord(d.quality);
	const sealed = Boolean(sealWord && seal);
	y = d.xp || sealed ? base + 20 : base;
	if (d.xp) {
		words(ctx, t("saga.xp", {xp: d.xp}), x * PX, y * PX,
			{size: num("--qp-surface-quest-bill-xp-size"), weight: 600, colour: ink2});
	}
	if (sealed) {
		const s = num("--qp-surface-quest-bill-seal-size");
		const sx = x + inner - s;
		const sy = base - numberSize * 0.86 - 6;
		ctx.save();
		ctx.translate((sx + s / 2) * PX, (sy + s / 2) * PX);
		ctx.rotate(SEAL_TURN * 2 * Math.PI);
		ctx.drawImage(seal, (-s / 2) * PX, (-s / 2) * PX, s * PX, s * PX);
		ctx.restore();
		words(ctx, sealWord, (sx + s / 2) * PX, (sy + s + 14) * PX, {size: num("--qp-surface-quest-bill-seal-word-size"),
			weight: 700, family: "wordmark", colour: red, align: "center", caps: true});
	}
	// the barcode between the day's turns: a bar a pump at its start, as wide as its moves, on the hours' ticks in ink2
	// (a full-height one every six)
	y += 22;
	const barH = num("--qp-surface-quest-bill-barcode-height");
	const turnWords = {size: footSize, weight: 700, family: FIGURES, colour: ink2, baseline: "middle"};
	const ends = measure(ctx, TURN, turnWords) / PX + space6;
	const from = x + ends;
	const span = inner - 2 * ends;
	ctx.save();
	ctx.fillStyle = ink2;
	ctx.globalAlpha = num("--qp-surface-quest-bill-tick-opacity");
	const tickW = num("--qp-surface-quest-bill-tick-width");
	for (let hr = 0; hr <= 24; hr += 1) {
		const tickH = hr % 6 === 0 ? barH : num("--qp-surface-quest-bill-tick-height");
		ctx.fillRect((from + (span * hr) / 24 - tickW / 2) * PX, (y + (barH - tickH) / 2) * PX, tickW * PX, tickH * PX);
	}
	ctx.restore();
	// the strip's placement (lib/saga.js `placeBars`): a bar a pump as wide as its moves, in minute order at or right of
	// its start, the gap closing and then every bar narrowing on a crowded hour
	ctx.fillStyle = ink;
	const placed = placeBars(d.bars, from, span, (b) => num("--qp-surface-quest-bill-barcode-base")
		+ num("--qp-surface-quest-bill-barcode-per-move") * b.kinds.reduce((a, k) => a + k, 0),
	num("--qp-surface-day-strip-bar-gap"), 1 / PX);
	d.bars.forEach((b, i) => ctx.fillRect(placed.xs[i] * PX, y * PX, placed.widths[i] * PX, barH * PX));
	words(ctx, TURN, x * PX, (y + barH / 2) * PX, turnWords);
	words(ctx, TURN, (x + inner) * PX, (y + barH / 2) * PX, {...turnWords, align: "right"});
	// the foot's line: its words on one baseline, centred in it as its line box, the mark centred too
	const footTop = y + barH + space8;
	const baseline = footTop + (footLine - footSize * LINE) / 2 + footSize * ASCENT;
	words(ctx, strip, x * PX, baseline * PX, footWords);
	const mark = await art("app_mark", look.gold);
	if (mark) {
		ctx.drawImage(mark, (x + inner - siteW - space6 - siteMark) * PX, (footTop + (footLine - siteMark) / 2) * PX,
			siteMark * PX, siteMark * PX);
	}
	words(ctx, site, (x + inner) * PX, baseline * PX, {...siteWords, align: "right"});
	ctx.restore();
	ctx.restore();
	if (look.gilded) gildedBand(ctx, canvas.width, canvas.height);
	return canvas;
}

// The day reel's layers (slice 45; docs/ui.md, "The share card": the Reel) -----------------------------------------

/** A cubic Bézier easing's value at `x` (0-1) for its control points [x1, y1, x2, y2] (tokens.json's curves): its
 * parameter where the curve's x is `x`, halved toward it, then its y there. */
function eased([x1, y1, x2, y2], x) {
	if (x <= 0) return 0;
	if (x >= 1) return 1;
	const at = (a, b, u) => 3 * a * u * (1 - u) ** 2 + 3 * b * u ** 2 * (1 - u) + u ** 3;
	let [lo, hi] = [0, 1];
	for (let i = 0; i < 40; i += 1) {
		const mid = (lo + hi) / 2;
		if (at(x1, x2, mid) < x) lo = mid;
		else hi = mid;
	}
	return at(y1, y2, (lo + hi) / 2);
}

/** A reel shot's words in the highlight slot (rule 18's `words`, always the move whose film plays under them): its
 * move's Quest Bill line in the bill's words (its count, its name, New {value} or Unlocked · {quality}, its reps or
 * seconds), the day's highlight, or the move's name alone in the bill's words (the end of a day with no record and no
 * unlock); none for a line the day's bill lacks. */
function shotSlot(card, shot) {
	if (shot.words === "highlight") return {kind: "highlight"};
	if (shot.words === "name") {
		return {kind: "bill", count: null, name: (card.ctx.names.get(shot.move) ?? {name: ""}).name, record: null,
			unlock: null, total: ""};
	}
	const d = card.day;
	const l = shot.words === "line" ? d.bill.find((b) => b.move === shot.move) : null;
	if (!l) return null;
	const move = card.ctx.names.get(l.move) ?? {name: "", load: null};
	const rec = d.records.find((r) => r.move === l.move);
	const unlock = d.unlocks.find((u) => u.move === l.move);
	return {
		kind: "bill", count: l.count, name: move.name,
		record: rec ? setWords(rec.best, rec.hold, move.load === "each") : null,
		unlock: !rec && unlock ? {quality: unlock.quality, word: card.ctx.qualities[unlock.quality - 1] ?? ""} : null,
		// a rep move's reps, any other's time; blank for a rep move none of whose sets has a count
		total: l.reps != null ? number(l.reps) : l.seconds != null ? clock(l.seconds) : "",
	};
}

/** The day reel's layers in `style` (the `reel` topic's `timing`; rule 18's `reel` of a day of the `log` topic, drawn as
 * the day's Saga card in its Story layout with the reel's insets): under the knight the ground (the parchment's well
 * in it); over it the number counting up from 0 by the Saga's reveal from `revealFromMs` and popping as the reveal
 * ends, the card's words that stand still over it (the head, the unit and tag, the stats, questpump.com), the strip's
 * bars growing by the reveal (the bare axis until it starts), and each shot's words in the highlight slot (`shotSlot`:
 * its move's Quest Bill line or the day's highlight; on the end the day's highlight where the lead is its move, else
 * the lead's bill line, or its name on a day with no record and no unlock); and the knight's square and look, where
 * Rust lays each shot's film between them. Each layer is the box of the frame it changes, in pixels, with its
 * frames. */
export async function reelLayers(card, style, timing) {
	const m = sagaModel(card);
	const plan = card.day.reel;
	const shots = [...plan.moves, plan.end];
	const slots = shots.map((shot) => shotSlot(card, shot));
	const lineH = slots.some((s) => s?.kind === "bill") ? BILL_LINE : m.line && slots.some(Boolean) ? 40 : 0;
	const laid = await sagaLayout(m, "story", style, {lineH});
	const [w, ht] = [laid.w * PX, laid.ht * PX];
	if (!laid.knight) throw new Error("the reel's knight has no room");
	if (w !== timing.width || ht !== timing.height) throw new Error(`a card of ${w} × ${ht} for a reel of ${timing.width} × ${timing.height}`);
	const frame = (ms) => Math.floor((ms * timing.fps) / 1000);
	const total = frame(plan.ms);
	// a frame's time since the reveal began (ms)
	const since = (f) => (f * 1000) / timing.fps - timing.revealFromMs;
	const layers = [];
	const add = async (box, from, to, over, draw) => {
		const [x, y] = [Math.max(0, Math.floor(box.x * PX)), Math.max(0, Math.floor(box.y * PX))];
		const [r, b] = [Math.min(w, Math.ceil((box.x + box.w) * PX)), Math.min(ht, Math.ceil((box.y + box.h) * PX))];
		if (r <= x || b <= y || to <= from) return;
		const canvas = document.createElement("canvas");
		[canvas.width, canvas.height] = [r - x, b - y];
		const ctx = canvas.getContext("2d");
		ctx.translate(-x, -y);
		await draw(ctx);
		layers.push({canvas, x, y, from, to, over});
	};
	const p = laid.parts;
	const whole = {x: 0, y: 0, w: laid.w, h: laid.ht};
	await add(whole, 0, total, false, (ctx) => {
		p.ground(ctx);
		p.well(ctx);
	});
	// the number: a layer for each count on its way and each step of its pop, its box grown by the pop about its foot,
	// under the words that stand still as the card draws it under its unit
	const n = laid.number;
	const grown = timing.popFrom;
	const numberBox = {x: 0, y: n.base - n.size * grown - 8, w: n.left + n.width * grown + 8,
		h: n.size * grown + n.descent * grown + 16};
	const counts = [];
	for (let f = 0; f < total; f += 1) {
		const value = Math.round(m.number * eased(timing.revealCurve, since(f) / timing.revealMs));
		const u = (since(f) - timing.revealMs) / timing.popMs;
		const pop = u >= 0 && u < 1 ? grown + (1 - grown) * eased(timing.popCurve, u) : 1;
		const last = counts.at(-1);
		if (last?.value === value && last.pop === pop) last.to = f + 1;
		else counts.push({value, pop, from: f, to: f + 1});
	}
	for (const c of counts) await add(numberBox, c.from, c.to, true, (ctx) => p.number(ctx, c.value, c.pop));
	await add(whole, 0, total, true, async (ctx) => {
		await p.site(ctx);
		await p.stats(ctx);
		await p.head(ctx);
		await p.aside(ctx);
	});
	// the strip: a layer for each step of its bars' growth, each bar from its stagger behind the one before
	const footBox = {x: 0, y: laid.foot.top - 24, w: laid.w, h: laid.foot.height + 48};
	const grows = [];
	for (let f = 0; f < total; f += 1) {
		const g = (m.bars ?? []).map((_, i) => eased(timing.revealCurve, (since(f) - i * timing.staggerMs) / timing.revealMs));
		const key = g.join();
		const last = grows.at(-1);
		if (last?.key === key) last.to = f + 1;
		else grows.push({key, g, from: f, to: f + 1});
	}
	for (const s of grows) await add(footBox, s.from, s.to, true, (ctx) => p.foot(ctx, (i) => s.g[i]));
	// each shot's words in the highlight slot, cut with its shot
	const lineBox = {x: 0, y: laid.line.top - 24, w: laid.w, h: laid.line.height + 32};
	for (const [k, shot] of shots.entries()) {
		const slot = slots[k];
		const to = k + 1 < shots.length ? frame(shots[k + 1].from) : total;
		if (slot) await add(lineBox, frame(shot.from), to, true, (ctx) => p.line(ctx, slot.kind === "bill" ? slot : null));
	}
	const k = laid.knight;
	const knight = {x: Math.round(k.x * PX), y: Math.round(k.y * PX), size: Math.round(k.size * PX),
		night: laid.look.night, mask: num("--qp-surface-saga-hero-mask-fraction"),
		radius: num("--qp-surface-saga-card-well-radius") * PX};
	return {knight, layers};
}

// The scene cards (slice 51; Rapha, 2026-10-22: the timed pump card and the streak card on a hand-painted scene) -----

/** The scene cards' measures in pixels on the 1080-px Story (docs/ui.md, "The share card"), each other format's scaled
 * by its share (`SCENE_SCALE`): the scene's top edge and the band its bare parchment foot is laid on in; the margin;
 * from the card's foot up the name and questpump.com's baseline, the stats' number's (its label `label` under it, the
 * dividers `divider` above and below it), the rule's and the head's first baseline, then down from that the date's,
 * the headline's, its line's and the rarity line's (`rareAlone` under a headline without a line, `rareLead` between
 * its lines), the least room between the last words and the rule; the seal beside the headline; the words' sizes and
 * caps' tracking; the standard-bearer's room above the words and over its feet; where the scenes' paint ends and
 * starts to fade (shares of their height), the fade a scene takes where its paint would reach the words, the room it
 * leaves above them and the soft edge of a scene narrower than the card (a share of its width). */
const SCENE = {
	top: -170, band: 150, margin: 80,
	site: 70, stats: 225, label: 48, divider: [70, 56], rule: 360, stroke: 2, head: 695,
	date: 50, brag: 160, line: 64, rare: 135, rareAlone: 80, rareLead: 50, ruleRoom: 40,
	seal: {size: 84, gap: 24, rise: 78},
	sizes: {title: 46, date: 32, brag: 96, line: 44, rare: 38, stat: 84, label: 30, site: 32},
	tracking: {title: 6, date: 3, label: 4, site: 3},
	room: {top: 30, gap: 50},
	bare: 0.82, soft: 0.64, fade: 120, clear: 10, side: 0.15,
};
/** Each format's share of the Story's measures: the Post and the Square draw their words smaller, so the scene keeps
 * the standard-bearer whole above them. */
const SCENE_SCALE = {story: 1, post: 0.78, square: 0.66};
/** The headline's least share of its size where it must shrink to its room (a long move's name). */
const BRAG_LEAST = 0.6;
/** The luminance key that cuts the standard-bearer out of its still's dark ground: a pixel's alpha is its brightest
 * channel past `floor`, times `gain`. */
const KEY = {floor: 24, gain: 10};
/** The shadow at its feet: its centre and radius as shares of the still's side, flattened to `flat`, in ink at
 * `alpha`. */
const FOOT_SHADOW = {x: 0.47, y: 0.93, r: 0.3, flat: 0.18, alpha: 0.45};
/** Night's scene under the page's colour at this share, before it fades into Night's ground. */
const NIGHT_DIM = 0.45;
/** The streak card's and the timed pump card's figure (contract/figures/figures.json `extras`): the standard-bearer. */
const BANNER = "streak_banner"; // copy-lint: not shown (a figure's name in figures.json)
/** The streaks the standing counts (contract/standing-api.yaml `streak_thresholds`), and the shares a rarity line is
 * drawn for: rarer than one in four, and from one in a thousand ("Fewer than 1 in 1,000" under it). */
const STREAK_STEPS = [3, 7, 14, 30, 60, 100, 180, 365];
const RARE_MOST = 0.25;
const RARE_LEAST = 0.001;

/** The standing while it counts (`standing`, desk/standing.rs: an hour after its answer), else null. */
function standing() {
	const held = get("standing");
	return held && held.until > Date.now() ? held : null;
}

/** A share as the rarity line writes it, in the app's language: one decimal under 1 % ("0.8%"), whole above. */
function percent(share) {
	const digits = Math.round(share * 1000) < 10 ? 1 : 0;
	return new Intl.NumberFormat(undefined, {style: "percent", minimumFractionDigits: digits,
		maximumFractionDigits: digits}).format(share);
}

/** The rarity line (slice 51): under a colour headline the share of players who reached that colour or above, under a
 * streak or Day headline the share whose best streak reached the highest threshold at or under its days; only for a
 * share from RARE_LEAST to RARE_MOST, under it Fewer than 1 in 1,000; none under any other headline and none while
 * the standing is empty, unreachable or older than its hour. */
function rarity(of) {
	const held = standing();
	if (!held || !of) return null;
	if (of.colour) {
		const share = held.colour?.[String(of.colour.quality)];
		if (share == null || share > RARE_MOST) return null;
		const colour = of.colour.word;
		return share < RARE_LEAST ? t("share.rare.colour.few", {colour})
			: t("share.rare.colour", {percent: percent(share), colour});
	}
	const days = STREAK_STEPS.filter((d) => d <= of.days).at(-1);
	const share = days ? held.streak?.[String(days)] : null;
	if (share == null || share > RARE_MOST) return null;
	return share < RARE_LEAST ? t("share.rare.streak.few", {days}) : t("share.rare.streak", {percent: percent(share), days});
}

/** The headline, its line under it, whether the seal stands beside it and what its rarity line counts (slice 51, "The
 * headline"): on a timed pump card the first that is true of a colour the player had not reached (the move that stands
 * for the step), a record (the biggest step past the record it beat), the longest timed pump yet and the streak's day;
 * on the streak card the streak over My best streak yet or the best. */
function brag(card) {
	if (card.kind === "streak") {
		const best = Math.max(card.best ?? 0, card.days);
		return {headline: t("share.brag.streak", {days: card.days}),
			line: card.days >= best ? t("share.brag.streak.best") : t("share.brag.streak.was", {days: best}), seal: false,
			of: {days: card.days}};
	}
	if (card.colour) {
		return {headline: t("share.brag.colour", {colour: card.colour.word}),
			line: t("share.brag.colour.step", {move: card.colour.move}), seal: true, of: {colour: card.colour}};
	}
	if (card.record) {
		const r = card.record;
		return {headline: t("share.brag.record", {move: r.move, value: setWords({value: r.value, load: r.load}, r.hold,
			r.each)}), line: r.from == null ? t("share.brag.record.new")
			: t("share.brag.record.up", {old: setWords({value: r.from}, r.hold, false)}), seal: true, of: null};
	}
	if (card.longest) {
		return {headline: t("share.brag.minutes", {minutes: card.minutes}), line: t("share.brag.longest"), seal: false,
			of: null};
	}
	const days = Math.max(1, card.streak ?? 0);
	return {headline: t("share.brag.day", {days}), line: t("share.brag.day.line", {days}), seal: false, of: {days}};
}

/** A scene card's model: its head (the card's name, the date it is shared on and a timed pump's place), its headline
 * with its line, seal and rarity line, its stats (Moves, Records, Day streak; the streak card's Days, Best streak,
 * Rest tokens), the character's name and what a screen reader hears. */
function sceneModel(card) {
	const b = brag(card);
	const timed = card.kind === "expedition";
	const head = timed ? t("expedition.named", {minutes: card.minutes}) : t("pump.home.streak.line");
	const date = [card.day ? shortDate(card.day) : "", timed ? card.place ?? "" : ""].filter(Boolean).join(" · ");
	const stats = timed
		? [[card.moves, unit(card.moves, "move")], [card.records, unit(card.records, "record")],
			[card.streak ?? 0, t("pump.home.streak.label")]]
		: [[card.days, t("card.days")], [Math.max(card.best ?? 0, card.days), t("pump.streak.best")],
			[card.tokens ?? 0, t("pump.streak.tokens")]];
	const rare = rarity(b.of);
	return {head, date, headline: b.headline, line: b.line, seal: b.seal, rare, stats, name: card.character || null,
		spoken: SAID.format([head, date, b.headline, b.line, rare].filter(Boolean))};
}

/** The scene a card stands on (the `figures` topic's `scenes`): a timed pump's place template's, the streak card's,
 * else the fallback; null without the table or the pages. */
function sceneOf(card) {
	const scenes = get("figures")?.scenes;
	if (!scenes?.files) return null;
	const name = card.kind === "streak" ? scenes.streak : scenes.places[card.scene] ?? scenes.fallback;
	const scene = scenes.scenes[name];
	return scene ? {...scene, src: scenes.files + encodeURIComponent(scene.file)} : null;
}

/** The standard-bearer in the colour the game draws the character in, cut out of its still's dark ground (KEY), as a
 * canvas its tile square wide; null without its still. */
async function bearer(tint) {
	const still = await extraStill(BANNER, tint);
	if (!still) return null;
	const side = Math.min(still.img.width, still.img.height);
	const [cx, cy, cs] = still.crop ?? [0, 0, 1];
	const px = Math.round(cs * side);
	const cut = document.createElement("canvas");
	cut.width = px;
	cut.height = px;
	const o = cut.getContext("2d", {willReadFrequently: true});
	o.drawImage(still.img, cx * side, cy * side, cs * side, cs * side, 0, 0, px, px);
	const pixels = o.getImageData(0, 0, px, px);
	const p = pixels.data;
	for (let i = 0; i < p.length; i += 4) {
		p[i + 3] = Math.max(0, Math.min(255, (Math.max(p[i], p[i + 1], p[i + 2]) - KEY.floor) * KEY.gain));
	}
	o.putImageData(pixels, 0, 0);
	return cut;
}

/** A "#RRGGBB" token's colour at `alpha`. */
function tinted(hex, alpha) {
	const [r, g, b] = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16));
	return `rgb(${r} ${g} ${b} / ${alpha})`; // copy-lint: not shown (a canvas colour)
}

/** The scene's bare parchment foot (its last `band` card pixels' rows, at the card's width) laid on in bands from
 * `foot` (the scene's last row), every other one mirrored so the paper's grain runs on unstretched: the bands from
 * `from` to `to`. */
function parchmentBands(ctx, img, w, band, foot, from, to) {
	const src = (band * img.naturalWidth) / w;
	for (let i = Math.floor((from - foot) / band); foot + i * band < to; i += 1) {
		const y = foot + i * band;
		ctx.save();
		if (i % 2 === 0) {
			ctx.translate(0, 2 * y + band);
			ctx.scale(1, -1);
		}
		ctx.drawImage(img, 0, img.naturalHeight - src, img.naturalWidth, src, 0, y, w, band);
		ctx.restore();
	}
}

/** Draws a timed pump card or a streak card in a format and a style (slice 51, "The share cards tell one true thing";
 * docs/ui.md, "The share card"): the scene its width wide from its top (on the Post and the Square as low as keeps the
 * standard-bearer whole above the words, and narrower with soft sides where even that leaves too little room), the
 * scene's bare parchment foot laid on under it (Night: its iron, the scene darkened and faded into it before the
 * words), the standard-bearer cut out at the scene's `stand` and `height` with its soft shadow at its feet; then the
 * head, the headline with its seal, its line and the rarity line, the rule, the stats and the name with questpump.com
 * on the foot; Gilded the parchment card in its gold band. */
async function drawScene(card, format, style) {
	const f = SCENE_SCALE[format] ?? 1;
	const at = (n) => n * f;
	const look = sagaLook(style);
	const m = sceneModel(card);
	const w = num("--qp-surface-saga-card-width") * PX;
	const ht = num(`--qp-surface-saga-card-${format}-height`) * PX;
	const sizes = Object.fromEntries(Object.entries(SCENE.sizes).map(([k, v]) => [k, at(v) / PX]));
	const tracks = (k) => SCENE.tracking[k] / SCENE.sizes[k];
	const type = {
		title: {size: sizes.title, weight: 700, family: "wordmark", caps: true, tracking: tracks("title")},
		date: {size: sizes.date, weight: 600, caps: true, tracking: tracks("date")},
		brag: {size: sizes.brag, weight: 700, family: "num-saga"},
		line: {size: sizes.line, weight: 600},
		rare: {size: sizes.rare, weight: 500, italic: true},
		stat: {size: sizes.stat, weight: 700, family: "num-saga", align: "center"},
		label: {size: sizes.label, weight: 600, caps: true, tracking: tracks("label"), align: "center"},
		site: {size: sizes.site, weight: 600, tracking: tracks("site"), align: "center"},
	};
	await Promise.all(Object.values(type).map((s) => document.fonts.load(
		`${s.italic ? "italic " : ""}${font(s.weight, s.size, s.family ?? "body")}`)));
	const canvas = document.createElement("canvas");
	canvas.width = w;
	canvas.height = ht;
	const ctx = canvas.getContext("2d");
	const margin = at(SCENE.margin);
	const room = w - 2 * margin;
	// the words from the foot up: the head moves up as far as the rarity line's lines would reach the rule
	const ruleY = ht - at(SCENE.rule);
	const sealW = m.seal ? at(SCENE.seal.size + SCENE.seal.gap) : 0;
	const bragSize = sizes.brag * Math.max(BRAG_LEAST, Math.min(1, (room - sealW) / (measure(ctx, m.headline, type.brag)
		/ PX)));
	const lineSize = sizes.line * Math.min(1, room / (measure(ctx, m.line, type.line) / PX));
	ctx.font = `italic ${font(type.rare.weight, sizes.rare, "body")}`; // copy-lint: not shown (a canvas font)
	const rareLines = m.rare ? wrap(ctx, m.rare, room) : [];
	const lastBelow = (top) => {
		const brag = top + at(SCENE.brag);
		if (rareLines.length) return brag + at(m.line ? SCENE.rare : SCENE.rareAlone) + (rareLines.length - 1) * at(SCENE.rareLead);
		return m.line ? brag + at(SCENE.line) : brag;
	};
	let headY = ht - at(SCENE.head);
	headY -= Math.max(0, lastBelow(headY) - (ruleY - at(SCENE.ruleRoom)));
	const wordsTop = headY - at(SCENE.sizes.title);
	// the ground, the scene and the standard-bearer
	const scene = sceneOf(card);
	const img = scene ? await image(scene.src) : null;
	if (look.night || !img) sagaGround(ctx, look, w, ht, format);
	if (img) {
		const fit = w / img.naturalWidth;
		const feetMost = wordsTop - at(SCENE.room.gap);
		const k = Math.min(fit, (feetMost - at(SCENE.room.top)) / (scene.height * img.naturalHeight));
		const [sw, sh] = [img.naturalWidth * k, img.naturalHeight * k];
		const sx = (w - sw) / 2;
		const tall = scene.height * sh;
		const sy = Math.max(Math.min(at(SCENE.top), feetMost - scene.stand[1] * sh), at(SCENE.room.top) + tall
			- scene.stand[1] * sh);
		const foot = sy + sh - 1;
		if (!look.night) {
			// the scene's own foot colour under all, its parchment bands under the scene
			const probe = document.createElement("canvas");
			probe.width = img.naturalWidth;
			probe.height = 4;
			const pc = probe.getContext("2d", {willReadFrequently: true});
			pc.drawImage(img, 0, img.naturalHeight - 4, img.naturalWidth, 4, 0, 0, img.naturalWidth, 4);
			const [r, g, b] = pc.getImageData(img.naturalWidth / 2, 2, 1, 1).data;
			ctx.fillStyle = `rgb(${r} ${g} ${b})`; // copy-lint: not shown (a canvas colour)
			ctx.fillRect(0, 0, w, ht);
			parchmentBands(ctx, img, w, at(SCENE.band), foot, 0, foot);
		}
		// where its paint would reach the words (and on Night always), it fades out above them; narrower than the card,
		// its sides fade into the ground
		const paintEnd = sy + SCENE.bare * sh;
		const fadeEnd = Math.min(paintEnd, wordsTop - at(SCENE.clear));
		const fades = look.night || paintEnd > wordsTop - at(SCENE.clear);
		const fadeStart = look.night ? Math.min(sy + SCENE.soft * sh, fadeEnd - at(SCENE.fade)) : fadeEnd - at(SCENE.fade);
		const sides = sw < w - 0.5;
		if (!fades && !sides) {
			ctx.drawImage(img, sx, sy, sw, sh);
		} else {
			const layer = document.createElement("canvas");
			layer.width = w;
			layer.height = ht;
			const lc = layer.getContext("2d");
			lc.drawImage(img, sx, sy, sw, sh);
			if (look.night) {
				lc.globalCompositeOperation = "source-atop";
				lc.fillStyle = tinted(token("--qp-color-page"), NIGHT_DIM);
				lc.fillRect(0, 0, w, ht);
			}
			lc.globalCompositeOperation = "destination-in";
			if (fades) {
				const g = lc.createLinearGradient(0, fadeStart, 0, fadeEnd);
				g.addColorStop(0, "rgb(0 0 0)");
				g.addColorStop(1, "rgb(0 0 0 / 0)");
				lc.fillStyle = g;
				lc.fillRect(0, 0, w, ht);
			}
			if (sides) {
				const edge = SCENE.side * sw;
				const g = lc.createLinearGradient(sx, 0, sx + sw, 0);
				g.addColorStop(0, "rgb(0 0 0 / 0)");
				g.addColorStop(edge / sw, "rgb(0 0 0)");
				g.addColorStop(1 - edge / sw, "rgb(0 0 0)");
				g.addColorStop(1, "rgb(0 0 0 / 0)");
				lc.fillStyle = g;
				lc.fillRect(0, 0, w, ht);
			}
			ctx.drawImage(layer, 0, 0);
		}
		if (!look.night) parchmentBands(ctx, img, w, at(SCENE.band), foot, foot, ht);
		const box = get("figures")?.extras?.[BANNER]?.box;
		const cut = box ? await bearer(card.tint) : null;
		if (cut) {
			const ks = tall / (box[3] - box[1]);
			const kx = sx + scene.stand[0] * sw - ((box[0] + box[2]) / 2) * ks;
			const ky = sy + scene.stand[1] * sh - box[3] * ks;
			const [cx, cy, r] = [kx + FOOT_SHADOW.x * ks, ky + FOOT_SHADOW.y * ks, FOOT_SHADOW.r * ks];
			const shade = ctx.createRadialGradient(cx, cy, 0, cx, cy, r);
			shade.addColorStop(0, tinted(token("--qp-color-ink"), FOOT_SHADOW.alpha));
			shade.addColorStop(1, tinted(token("--qp-color-ink"), 0));
			ctx.save();
			ctx.translate(cx, cy);
			ctx.scale(1, FOOT_SHADOW.flat);
			ctx.translate(-cx, -cy);
			ctx.fillStyle = shade;
			ctx.beginPath();
			ctx.arc(cx, cy, r, 0, 2 * Math.PI);
			ctx.fill();
			ctx.restore();
			ctx.drawImage(cut, kx, ky, ks, ks);
		}
	}
	// the head, the headline with its seal, its line and the rarity line
	words(ctx, m.head, margin, headY, {...type.title, colour: look.night ? token("--qp-color-gold-hi") : look.gold});
	if (m.date) words(ctx, m.date, margin, headY + at(SCENE.date), {...type.date, colour: look.words2});
	const bragY = headY + at(SCENE.brag);
	const bragW = words(ctx, m.headline, margin, bragY, {...type.brag, size: bragSize, colour: look.words});
	if (m.seal) {
		const seal = await art("seal", token("--qp-surface-quest-bill-wax"));
		const side = at(SCENE.seal.size);
		if (seal) ctx.drawImage(seal, margin + bragW + at(SCENE.seal.gap), bragY - at(SCENE.seal.rise), side, side);
	}
	if (m.line) words(ctx, m.line, margin, bragY + at(SCENE.line), {...type.line, size: lineSize, colour: look.gold});
	let rareY = bragY + at(m.line ? SCENE.rare : SCENE.rareAlone);
	for (const l of rareLines) {
		words(ctx, l, margin, rareY, {...type.rare, colour: look.words2});
		rareY += at(SCENE.rareLead);
	}
	// the rule, the stats between their dividers, the name and questpump.com
	ctx.fillStyle = look.rule;
	ctx.fillRect(margin, ruleY - at(SCENE.stroke) / 2, room, at(SCENE.stroke));
	const statY = ht - at(SCENE.stats);
	const col = room / m.stats.length;
	m.stats.forEach(([value, label], i) => {
		const cx = margin + col * i + col / 2;
		words(ctx, number(value), cx, statY, {...type.stat, colour: look.words});
		words(ctx, label, cx, statY + at(SCENE.label), {...type.label, colour: look.words2});
		if (i > 0) {
			ctx.fillRect(margin + col * i - at(SCENE.stroke) / 2, statY - at(SCENE.divider[0]), at(SCENE.stroke),
				at(SCENE.divider[0] + SCENE.divider[1]));
		}
	});
	const site = m.name ? `${m.name.toLocaleUpperCase()}  ·  ${t("card.site")}` : t("card.site");
	words(ctx, site, w / 2, ht - at(SCENE.site), {...type.site, colour: look.words2});
	if (look.gilded) gildedBand(ctx, w, ht);
	return canvas;
}

// The cards, their window and its choices ----------------------------------------------------------------------------

/** Draws any card from its model and the choice: a record's or an unlock's quest page (square), a day's Saga card or
 * Quest Bill, a week's Saga card, the streak card or the timed pump card in a format and a style, the day's and the
 * streak's sticker; the Reel's still card is the day's Saga card in Story. */
export async function drawCard(card, choice) {
	if (card.kind === "record" || card.kind === "unlock") return drawQuest(card, choice.style);
	const format = choice.format === "reel" ? "story" : choice.format;
	if ((card.kind === "streak" || card.kind === "expedition") && format !== "sticker") {
		return drawScene(card, format, choice.style);
	}
	if (format === "sticker" && (card.kind === "streak" || (card.kind === "day" && choice.card === "saga"))) {
		return drawSticker(card);
	}
	if (card.kind === "day" && choice.card === "bill") return drawBill(card, format, choice.style);
	return drawSaga(card, format, choice.style);
}

/** A card's words: what a screen reader hears for its picture. */
function spoken(card) {
	if (card.kind === "record" || card.kind === "unlock") return questWords(card);
	if (card.kind === "streak" || card.kind === "expedition") return sceneModel(card).spoken;
	return familyModel(card).spoken;
}

/** The file's name, before its .png or .mp4: the move's, a day's chapter and its date's key, a week's first day's key,
 * a streak's words or a timed pump's name and the day it is shared on. A header carries an image's, which takes no letter past Latin-1 and
 * which Rust reads as ASCII alone: every other letter goes, and a name left empty lets the save dialog name it
 * QuestPump.png (a reel QuestPump.mp4). */
function fileName(card) {
	const name = card.kind === "week" ? `${t("saga.week")} ${card.day.week.start}`
		: card.kind === "day" ? `${chapterWords(card.day.chapter)} ${card.day.day}`
			: card.kind === "streak" ? `${milestoneWords(card.days)} ${card.day ?? ""}`
				: card.kind === "expedition" ? `${t("expedition.named", {minutes: card.minutes})} ${card.day ?? ""}`
					: card.name;
	return name.replace(/[^\x20-\x7E]/g, "").replace(/[/\\:]/g, " ").trim();
}

/** The streak card of a streak of `days` days (slice 45: a milestone's from the Result's banner and row, the
 * StreakBoard's as it stands) with the best streak and the rest tokens held (the `streak` topic), shared today (the
 * `home` topic's game day) and naming the character of the sheet shown, the standard-bearer in the colour the game
 * draws it in (`book`); its sticker with the Milestone tag where `milestone` (the Result's) or where the count is one
 * of `streak_milestones` (`home.milestones`). */
export function streakCard(days, milestone = null) {
	const home = get("home");
	const book = get("book");
	const streak = get("streak");
	return {kind: "streak", days, milestone: milestone ?? Boolean(home?.milestones?.includes(days)),
		best: streak?.best ?? days, tokens: streak?.tokens ?? 0, day: home?.day ?? null,
		character: book?.character ?? null, tint: book?.tint ?? null};
}

/** The timed pump card of a timed pump that ended (slice 51: Timed pump complete's, `pump.expedition.done`): its
 * minutes, its place's words and template (`scene`: its scene's key), its moves and records, the headline's facts (a
 * colour reached for the first time, the record with the biggest step, the longest timed pump yet) and the streak it
 * kept (the `streak` topic), shared today and naming the character of the sheet shown, the standard-bearer in the
 * colour the game draws it in (`book`). */
export function expeditionCard({minutes, place, scene, moves, records, colour, record, longest}) {
	const home = get("home");
	const book = get("book");
	return {kind: "expedition", minutes, place, scene, moves, records, colour: colour ?? null, record: record ?? null,
		longest: Boolean(longest), streak: get("streak")?.days ?? 0, day: home?.day ?? null,
		character: book?.character ?? null, tint: book?.tint ?? null};
}

/** What the window's status line says of the reel in each state, heard as it comes (idle and cancelled: nothing). */
const REEL_WORDS = {
	making: () => t("share.reel.making"),
	ready: () => t("share.reel.ready"),
	failed: () => t("share.reel.failed"),
	unsupported: () => t("share.reel.unsupported"),
	none: () => t("share.reel.none"),
};

/** The Share window: "Your card" over the card as it will be saved; for a day the Card row (Saga, Quest Bill), for a
 * day, a week and a streak the Format row (Story, Post, Square, the Sticker of the day's Saga card and of the streak
 * card, and the day's Saga card's Reel), with the Sound row (On, Off) under it while Reel is chosen, and with Pro the
 * Style row (Parchment, Night, Gilded; none for a sticker), each choice kept on this computer (`share.card`,
 * `share.format`, `share.sound`, `share.style`; nothing kept yet: the Saga card in Story, Sound on); "Copy image" and
 * the gold "Save image" side by side in its foot, each leaving the window open, and while Reel is chosen "Copy video"
 * and the gold "Save video" in their place (lib/reel.js: the reel made as soon as Reel is chosen and again when its
 * style or Sound changes, its preview the made file, Save video busy while it is made, either made again where none
 * is; a made reel kept while another format is chosen and shown again at once where Reel comes back with the same
 * day's plan, style and Sound, 2026-10-19); a card that could not be drawn says so there (`share.failed`), the window
 * kept, and so does a reel that could not be made. A day's reel plan follows the `log` topic while the window is open:
 * the films landing after their download give the day its plan, made then where Reel is chosen (2026-10-19). Closing
 * the window stops a reel being made, which leaves no file. `card.card` and `card.format` open it
 * on a card and a format for that opening alone, kept only where the player then chooses (the best day's Saga card in
 * Story). A timed pump card or a streak card asks for questpump.com's standing as the window opens (`standing.look`)
 * and is drawn again with its rarity line as it comes (slice 51). Every card and format is free; a record's and an
 * unlock's stay square. A pump that starts while it is open
 * (the router's cover, over everything) closes it, as a stage's own windows close when the stage moves on, its reel
 * stopped by Rust; one opened over a pump's Result stays through that pump. */
export function openShare(card) {
	const book = get("book");
	const pro = Boolean(book?.pro);
	const saga = card.kind === "day" || card.kind === "week" || card.kind === "streak" || card.kind === "expedition";
	// a scene card's rarity line: the standing asked as the window opens, the card drawn again when it comes
	const scened = card.kind === "streak" || card.kind === "expedition";
	if (scened) intent("standing.look").catch(console.error);
	const choice = {
		style: pro && STYLES.includes(book?.style) ? book.style : "parchment",
		card: card.kind === "day" && CARDS.includes(card.card ?? book?.card) ? card.card ?? book.card : "saga",
		format: FORMATS.includes(card.format ?? book?.format) ? card.format ?? book.format : "story",
		sound: book?.sound === false ? "off" : "on",
	};
	// the sticker is the day's Saga card's and the streak card's alone, the reel the day's Saga card's: the others take
	// the story
	const sticker = () => card.kind === "streak" || (card.kind === "day" && choice.card === "saga");
	const reelable = () => card.kind === "day" && choice.card === "saga";
	const fit = () => {
		if ((choice.format === "sticker" && !sticker()) || (choice.format === "reel" && !reelable())) choice.format = "story";
	};
	fit();
	const reeling = () => choice.format === "reel";
	// the day's reel plan (rule 18's), as the `log` topic gives it now (below)
	let plan = card.kind === "day" ? card.day.reel ?? null : null;
	let drawn = null;
	let still = null;
	let told = null;
	const preview = h("div", {class: "qp-share-preview", role: "img", "aria-label": spoken(card)});
	const status = h("p", {class: "qp-type-meta qp-share-status", "aria-hidden": "true"});
	// the status line, heard where it says something (an empty line clears it, unheard, so it never overtakes another)
	const say = (text) => {
		status.textContent = text;
		if (text) announce(text);
	};
	const copy = h("qp-secondary-button", {label: t("share.copy"), "on:click": () => (reeling() ? copyReel() : copyImage())});
	const save = h("qp-primary-button", {label: t("share.save"), "on:click": () => (reeling() ? saveReel() : saveImage())});
	// the Reel: its preview, its words and its buttons by the state of the reel this window asked for
	const showReel = () => {
		if (!reeling()) return;
		const phase = plan ? maker.phase : "none";
		if (phase === "ready" && maker.src) {
			if (preview.dataset.src !== maker.src) {
				preview.dataset.src = maker.src;
				preview.removeAttribute("role");
				preview.removeAttribute("aria-label");
				preview.replaceChildren(reelPreview(maker.src, Math.round(plan.ms / 1000),
					get("reel")?.timing?.fps ?? 30));
			}
		} else {
			delete preview.dataset.src;
			preview.setAttribute("role", "img");
			preview.setAttribute("aria-label", spoken(card));
			// the still Story card under the kit's pips with Making your reel… while it is made
			const making = phase === "making" ? h("div", {class: "qp-reel-making", "aria-hidden": "true"},
				h("span", {class: "qp-busy"}, h("i"), h("i"), h("i")),
				h("p", {class: "qp-type-meta", text: t("share.reel.making")})) : null;
			preview.replaceChildren(...[still, making].filter(Boolean));
		}
		const none = phase === "none" || phase === "unsupported";
		copy.disabled = none || phase === "making";
		save.state = phase === "making" ? "busy" : none ? "disabled" : "normal";
		if (phase !== told) {
			told = phase;
			say(REEL_WORDS[phase]?.() ?? "");
		}
	};
	const maker = new ReelMaker(showReel);
	const makeReel = () => {
		if (!plan) {
			maker.cancel();
			showReel();
			return;
		}
		const {style, sound} = choice;
		// what the reel shows: a made one of the same key comes back at once (lib/reel.js)
		const key = JSON.stringify([card.day.day, plan, style, sound]);
		maker.make({key, plan, sound: sound === "on", name: fileName(card),
			draw: (timing) => reelLayers({...card, day: {...card.day, reel: plan}}, style, timing)});
	};
	// the films landing (or the day's pumps changing its plan) while the window is open: the day's plan as the topic
	// has it now, the reel made of it where Reel is chosen
	const unlog = card.kind === "day" ? subscribe("log", (log) => {
		const day = [log?.today, log?.open].find((d) => d?.day === card.day.day);
		if (!day || JSON.stringify(day.reel ?? null) === JSON.stringify(plan)) return;
		plan = day.reel ?? null;
		// before the window's first draw (this call, at once), that draw makes it
		if (reeling() && drawn) {
			told = null;
			makeReel();
		}
	}) : () => {};
	const draw = () => {
		const sizes = {story: [9, 16], post: [4, 5], square: [1, 1], sticker: [1, 1], reel: [9, 16]};
		const [rw, rh] = saga ? sizes[choice.format] : sizes.square;
		styled(preview, {"--qp-share-ratio": rw / rh});
		preview.classList.toggle("qp-share-clear", choice.format === "sticker");
		copy.label = t(reeling() ? "share.video.copy" : "share.copy");
		save.label = t(reeling() ? "share.video.save" : "share.save");
		const mine = drawCard(card, choice);
		drawn = mine;
		if (reeling()) {
			told = null;
			still = null;
			makeReel();
		} else {
			maker.cancel();
			copy.disabled = false;
			save.state = "normal";
		}
		// a draw a later choice overtook shows nothing: the preview is always what Save and Copy take
		mine.then((canvas) => {
			if (mine !== drawn) return;
			styled(preview, {"--qp-share-ratio": canvas.width / canvas.height});
			if (reeling()) {
				still = canvas;
				showReel();
				return;
			}
			delete preview.dataset.src;
			preview.setAttribute("role", "img");
			preview.setAttribute("aria-label", spoken(card));
			preview.replaceChildren(canvas);
			say("");
		}).catch((e) => {
			console.error(e);
			// the window stays, so another choice or a press tries again
			if (mine === drawn) say(t("share.failed"));
		});
	};
	const row = (label, options, key, keep) => {
		const chips = h("div", {class: "qp-filter-row", role: "group", "aria-label": label});
		const paint = () => keepFocus(chips, () => chips.replaceChildren(...options().map(([value, words_]) =>
			h("qp-choice-chip", {selected: value === choice[key], label: words_, "data-focus-key": `${key}:${value}`,
				"on:click": () => {
					if (value === choice[key]) return;
					choice[key] = value;
					keep(value).catch(console.error);
					rows();
					draw();
				}}))));
		const el = h("div", {class: "qp-share-styles"}, h("span", {class: "qp-type-caps", text: label,
			"aria-hidden": "true"}), chips);
		return {el, paint};
	};
	const cardRow = row(t("share.card"), () => [["saga", t("share.card.saga")], ["bill", t("share.card.bill")]], "card",
		(value) => intent("share.card", {card: value}));
	const formatRow = row(t("share.format"), () => [["story", t("share.format.story")], ["post", t("share.format.post")],
		["square", t("share.format.square")], ...(sticker() ? [["sticker", t("share.format.sticker")]] : []),
		...(reelable() ? [["reel", t("share.format.reel")]] : [])], "format",
	(value) => intent("share.format", {format: value}));
	const soundRow = row(t("share.sound"), () => [["on", t("share.sound.on")], ["off", t("share.sound.off")]], "sound",
		(value) => intent("share.sound", {on: value === "on"}));
	const styleRow = row(t("share.style.title"), () => [["parchment", t("share.style.parchment")],
		["night", t("share.style.night")], ["gilded", t("share.style.gilded")]], "style",
	(value) => intent("share.style", {style: value}));
	const rows = () => {
		fit();
		cardRow.el.hidden = card.kind !== "day";
		formatRow.el.hidden = !saga;
		soundRow.el.hidden = !reeling();
		styleRow.el.hidden = !pro || choice.format === "sticker";
		cardRow.paint();
		formatRow.paint();
		soundRow.paint();
		styleRow.paint();
	};
	rows();
	const copyImage = () => {
		// the PNG promised inside the click, as the clipboard asks
		const png = drawn.then((canvas) => new Promise((resolve, reject) => canvas.toBlob((blob) => (blob ? resolve(blob)
			: reject(new Error("no PNG"))), "image/png")));
		navigator.clipboard.write([new ClipboardItem({"image/png": png})])
			.then(() => {
				say(t("desk.book.share.copied"));
				shared();
			})
			.catch((e) => {
				console.error(e);
				say(t("desk.book.share.copy.failed"));
			});
	};
	const saveImage = async () => {
		save.state = "busy";
		try {
			const canvas = await drawn;
			const blob = await new Promise((resolve) => canvas.toBlob(resolve, "image/png"));
			const bytes = new Uint8Array(await blob.arrayBuffer());
			const name = fileName(card);
			if (await callBytes("save_image", bytes, name ? {"x-qp-name": `${name}.png`} : {})) {
				say(t("desk.book.share.saved"));
				shared();
			}
		} catch (e) {
			console.error(e);
			say(t("desk.book.share.save.failed"));
		} finally {
			save.state = "normal";
		}
	};
	// a reel that fails or goes says so itself (`showReel`); the words here are the file's
	const copyReel = async () => {
		try {
			await copyVideo(await maker.made());
			say(t("desk.book.share.video.copied"));
			shared();
		} catch (e) {
			console.error(e);
			if (!(e instanceof ReelGone)) say(t("desk.book.share.video.copy.failed"));
		}
	};
	const saveReel = async () => {
		try {
			const id = await maker.made();
			save.state = "busy";
			if (await saveVideo(id)) {
				say(t("desk.book.share.video.saved"));
				shared();
			}
		} catch (e) {
			console.error(e);
			if (!(e instanceof ReelGone)) say(t("desk.book.share.video.save.failed"));
		} finally {
			showReel();
		}
	};
	// the preview beside its choices where the window is wide enough, over them where it is not, sized to the room they
	// leave; the status line, Copy and the gold Save in the window's foot, in view whatever scrolls
	const win = h("qp-window", {heading: t("share.title"), class: "qp-share-window"});
	win.content.append(h("div", {class: "qp-share"}, preview, h("div", {class: "qp-share-choices"}, cardRow.el,
		formatRow.el, soundRow.el, styleRow.el)), h("div", {class: "qp-share-foot"}, status,
		h("div", {class: "qp-share-actions"}, copy, save)));
	// the pump it was opened over (a Result's Share), none from a page
	const over = get("pump")?.pump?.id ?? null;
	let ousted = false;
	const unwatch = subscribe("pump", (topic) => {
		const id = topic?.pump?.id ?? null;
		if (id === null || id === over || ousted) return;
		ousted = true;
		win.close();
	});
	// closed by a pump, its opener lies under that pump's cover, out of reach: the router gives the pump's view the
	// focus the window held (lib/router.js, on qp-closed)
	win.addEventListener("qp-closed", () => {
		unwatch();
		unlog();
		unstand();
		maker.close();
		win.remove();
	});
	// the standing landing (or going) while the window is open draws the scene card again (after the first draw)
	let first = true;
	const unstand = scened ? subscribe("standing", () => {
		if (!first) draw();
	}) : () => {};
	draw();
	first = false;
	win.open();
	return win;
}
