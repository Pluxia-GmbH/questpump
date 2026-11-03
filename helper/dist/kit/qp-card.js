// Card: the one container: card with a 1-px line edge (lifted under Increase Contrast), an inner top highlight and an
// 18 radius, its content padded; `rows`, a card of rows: no padding, each row under a line separator but the first.

import {KitElement, define} from "./element.js";

class QpCard extends KitElement {}

define("qp-card", QpCard, {rows: Boolean});
