// CardRow: one row of a card of rows drawn on its own (a long list's item): its `place` in the card (alone, first,
// middle, last) gives it the card's fill, the card's rounded corners and edge where the card's top or bottom is, and
// the separator over every row but the first.

import {KitElement, define} from "./element.js";

class QpCardRow extends KitElement {}

define("qp-card-row", QpCardRow, {place: String});
