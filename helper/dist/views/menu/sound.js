// Sound and voice (the Menu's page; docs/ui.md, "The Menu on the computer"): one card of switches, the Sounds switch,
// the voice in pumps, its callouts, the rep count ("Count reps aloud", 2026-10-09) and the voice for match offers, each
// its name alone (2026-10-22); the volume, a card of Sliders, Master, Sounds and Voice (2026-10-23); the outputs they
// play on (an OutputPicker); and the note while a screen reader takes the voice's lines (the `shell` topic's `reader`,
// its one owner, which the Book's Tree | List default reads too). Each switch asks Rust (`sound.on`, `sound.voice`,
// `sound.callouts`, `sound.count`, `sound.voice_offers`) and shows the answer the `menu.sound` topic brings
// (helper/src-tauri/src/desk/audio.rs); a level goes as it is set (`sound.level`), so closing the window keeps it, and
// its part of the test plays once it rests (`sound.test`).

import {intent} from "../../lib/bridge.js";
import {t} from "../../lib/copy.js";
import {register} from "../../lib/router.js";
import {h} from "../../kit/element.js";
import {page, watch, words} from "../page.js";

/** How long a level rests before its test plays: a held arrow key or quick steps play one test, after the last. */
const SETTLE_MS = 150;

register("desk.menu.sound", () => {
	const view = page({heading: t("desk.sound.title")});
	const toggle = (heading, glyph, type) => {
		// the switch moves when the topic brings Rust's answer (Toggle's contract), never before
		const row = h("qp-row", {heading, glyph, trailing: "toggle"});
		row.addEventListener("qp-change", (e) => intent(type, {on: e.detail}).catch(console.error));
		return row;
	};
	const sounds = toggle(t("menu.sounds"), "rating_expertise", "sound.on");
	const voice = toggle(t("desk.sound.voice"), "action_character", "sound.voice");
	const callouts = toggle(t("desk.sound.callouts"), "reward_record", "sound.callouts");
	const count = toggle(t("menu.repcount"), "mark_play", "sound.count");
	const offers = toggle(t("desk.sound.offers"), "action_games", "sound.voice_offers");
	/** A level's Slider: it moves at once and the level goes at once; the topic's answer confirms it, and an answer
	 * sent before it (another switch's, a level's earlier step) is held back until one carries the level set, so the
	 * knob never steps back where WebKit gives a dragged range no focus. */
	const level = (heading, channel) => {
		const slider = h("qp-slider", {heading});
		let want = null;
		let settle;
		slider.addEventListener("qp-change", (e) => {
			want = e.detail;
			intent("sound.level", {channel, percent: want}).catch((error) => {
				want = null;
				console.error(error);
			});
			clearTimeout(settle);
			settle = setTimeout(() => intent("sound.test", {channel}).catch(console.error), SETTLE_MS);
		});
		return {slider, take(percent) {
			if (want !== null && percent !== want) return;
			want = null;
			slider.value = percent;
		}};
	};
	const levels = {master: level(t("desk.sound.volume.master"), "master"), sounds: level(t("menu.sounds"), "sounds"),
		voice: level(t("desk.sound.volume.voice"), "voice")};
	const reader = words("", "meta");
	view.body.append(h("qp-card", {rows: true}, sounds, voice, callouts, count, offers),
		h("qp-section-label", {label: t("desk.sound.volume")}), h("qp-card", {rows: true}, ...Object.values(levels).map((l) => l.slider)),
		h("qp-section-label", {label: t("desk.sound.output")}), h("qp-output-picker"), reader);
	watch(view, "menu.sound", (s) => {
		sounds.checked = s.sound;
		voice.checked = s.voice;
		callouts.checked = s.callouts;
		count.checked = s.count;
		offers.checked = s.voice_offers;
		for (const [channel, l] of Object.entries(levels)) l.take(s.levels[channel]);
	});
	watch(view, "shell", (shell) => {
		reader.hidden = !shell.reader;
		reader.textContent = t(shell.platform === "windows" ? "desk.sound.reader.windows" : "desk.sound.reader");
	});
	view.primary = () => sounds;
	return view;
});
