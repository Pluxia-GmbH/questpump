// One stage changing into the next (contract/design/tokens.json `motion.stage`, slice 72: a pump's stages and the first
// five minutes' steps), never a hard cut: `change(host, next, stays)` puts the nodes `next` in `host`; what comes fades
// in (kit.css `.qp-stage-in`) around the nodes of `stays` it holds (the move's figure, which stays where it stands; a
// part that goes on from stage to stage), so nothing over them ever fades, and what stood there fades out under it
// (`.qp-stage-out`: inert, out of the keys' and a screen reader's way, gone once it has faded or the next change comes)
// unless `under` is false: a stage whose words the next one made untrue leaves at once (the phones' StageFade). Reduce
// Motion keeps the fade at its reduced time (tokens.css). The first content of a host comes as it is.

import {h} from "../kit/element.js";

const HOST = "qp-stage-host";
const OUT = "qp-stage-out";
const IN = "qp-stage-in";

/** The parts under `root` that fade in: `root` whole where it holds none of `stays`, else every part off the way down
 * to them. */
function coming(root, stays) {
	if (stays.includes(root)) return [];
	if (!stays.some((node) => root.contains(node))) return [root];
	return [...root.children].flatMap((child) => coming(child, stays));
}

/** Changes `host`'s content to `next`: what comes fades in, and what leaves fades out under it, or with `under` false
 * leaves at once. */
export function change(host, next, stays = [], {under = true} = {}) {
	host.classList.add(HOST);
	host.querySelector(`:scope > .${OUT}`)?.remove();
	const kept = stays.filter(Boolean);
	const leaving = [...host.childNodes].filter((node) => !next.includes(node));
	if (!leaving.length) {
		host.replaceChildren(...next);
		return;
	}
	if (under) {
		// a figure the leaving stage held for its owner goes on in the next: the stage that leaves draws it no more
		for (const node of leaving) {
			if (node instanceof Element) for (const stage of node.querySelectorAll("qp-figure-stage")) stage.release();
		}
		const out = h("div", {class: OUT, inert: true, "aria-hidden": "true"}, ...leaving);
		out.addEventListener("animationend", (e) => {
			if (e.target === out) out.remove();
		});
		host.replaceChildren(...next, out);
	} else {
		host.replaceChildren(...next);
	}
	for (const part of next.flatMap((root) => coming(root, kept))) {
		part.classList.add(IN);
		const done = (e) => {
			if (e.target !== part) return;
			part.classList.remove(IN);
			part.removeEventListener("animationend", done);
		};
		part.addEventListener("animationend", done);
	}
}
