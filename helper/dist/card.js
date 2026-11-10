// The match-over card's page (docs/ui.md, "The match-over card"): a window that never takes the focus, drawn in the
// kit's MatchCard from the `card` topic (helper/src-tauri/src/desk/card.rs: the offer or a reminder, or null while no
// card shows).
// Start asks for the pump at the size chosen (`card.start`), Later and the × close the card (`card.later`); each names
// the card it shows, so a click on a card a newer offer replaced does nothing. Its own errors go into QuestPump's log
// first.

import "./lib/errors.js"; // first: its listeners go on before any other module evaluates
import "./kit/index.js";
import {intent, start, subscribe} from "./lib/bridge.js";
import {startKeys} from "./lib/keys.js";
import {h} from "./kit/element.js";

await start();
startKeys();

const card = h("qp-match-card", {hidden: true});
card.addEventListener("qp-start", (e) => {
	if (card.offer) intent("card.start", {offer: card.offer.id, size: e.detail}).catch(console.error);
});
card.addEventListener("qp-later", () => {
	if (card.offer) intent("card.later", {offer: card.offer.id}).catch(console.error);
});
document.getElementById("qp-card").append(card);
subscribe("card", (offer) => {
	card.offer = offer;
	card.hidden = !offer;
});
