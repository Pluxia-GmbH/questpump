// The Acknowledgements on the computer (docs/ui.md, "The licence ledger and Acknowledgements"; the phones' page): the
// fonts (each its name, maker, copyright line and licence) and the sound packs (each its name, maker, licence and its
// credit line, the row opening its page in the browser) as contract/design/acknowledgements.json lists them for every
// app, the code the desktop app builds in, each a row opening its notice in a window (the engine's Lua 5.1.5, mlua and
// lua-src under the MIT licence; Symphonia, which plays the sounds and the voice, under the MPL 2.0; every Rust crate's
// licence texts), then each licence by name, its text in a window: a licence's own verbatim text from the file (the
// OFL's), CC0's plain words from the copy. A text keeps its own line breaks. From the `menu.acknowledgements` topic
// (helper/src-tauri/src/desk/menu_pages/about.rs).

import {intent} from "../../lib/bridge.js";
import {t} from "../../lib/copy.js";
import {register} from "../../lib/router.js";
import {h} from "../../kit/element.js";
import {page, watch, words} from "../page.js";

/** A licence's name in the page's words; an id the copy has no name for shows as written (an SPDX id). */
function licenceName(id) {
	switch (id) {
		case "OFL-1.1": return t("menu.acknowledgements.licence.ofl");
		case "CC0-1.0": return t("menu.acknowledgements.licence.cc0");
		default: return id;
	}
}

/** A licence's text: CC0's plain words from the copy, any other the file's verbatim text; none for an id with
 * neither. */
function licenceText(id, licences) {
	return id === "CC0-1.0" ? t("menu.acknowledgements.licence.cc0.text") : licences[id] ?? null;
}

register("desk.menu.acknowledgements", () => {
	const view = page({heading: t("menu.acknowledgements.title")});
	const fonts = h("qp-card", {rows: true});
	const assets = h("qp-card", {rows: true});
	const code = h("qp-card", {rows: true});
	const texts = h("qp-card", {rows: true});
	const win = h("qp-window");
	const text = words("", "meta");
	text.classList.add("qp-notice");
	win.content.append(text);
	const show = (heading, body) => {
		win.heading = heading;
		text.textContent = body;
		win.open();
	};
	view.body.append(h("qp-section-label", {label: t("menu.acknowledgements.fonts")}), fonts,
		h("qp-section-label", {label: t("menu.acknowledgements.assets")}), assets,
		h("qp-section-label", {label: t("desk.menu.acknowledgements.code")}), code,
		h("qp-section-label", {label: t("menu.acknowledgements.text")}), texts);
	watch(view, "menu.acknowledgements", (credits) => {
		const listed = [];
		const name = (id) => {
			if (!listed.includes(id)) listed.push(id);
			return licenceName(id);
		};
		fonts.replaceChildren(...(credits.fonts ?? []).map((font) => {
			const row = h("qp-row", {heading: font.title, glyph: "action_book", trailing: "shown"});
			row.lines = [font.author, font.copyright, name(font.licence)];
			return row;
		}));
		assets.replaceChildren(...(credits.assets ?? []).map((asset, i) => {
			const row = h("qp-row", {heading: asset.title, glyph: "mark_heart", trailing: "external",
				"on:click": () => intent("about.open", {asset: i}).catch(console.error)});
			row.lines = [asset.author, name(asset.licence), asset.credit];
			return row;
		}));
		const notice = (name, licence, body) => h("qp-row", {heading: t(name), meta: t(licence), glyph: "action_pump",
			"on:click": () => show(t(name), body)});
		code.replaceChildren(
			notice("desk.menu.acknowledgements.engine", "desk.menu.acknowledgements.licence.mit", credits.engine),
			notice("desk.menu.acknowledgements.audio", "desk.menu.acknowledgements.licence.mpl", credits.audio),
			notice("desk.menu.acknowledgements.crates", "desk.menu.acknowledgements.crates.meta", credits.crates));
		const licences = credits.licences ?? {};
		texts.replaceChildren(...listed.filter((id) => licenceText(id, licences) !== null).map((id) => h("qp-row", {
			heading: licenceName(id), glyph: "reward_title",
			"on:click": () => show(licenceName(id), licenceText(id, licences))})));
	});
	view.stops.push(() => win.remove());
	return view;
});
