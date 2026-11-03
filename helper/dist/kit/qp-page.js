// Page: the page colour under its warm light from above (Tokens.Surface.Page.glow); `light="result"` (behind the
// fanfare and the XP gain) or `light="welcome"` (behind the figure and the wordmark) light it from the middle instead.
// Every view stands on one; kit.css draws it.

import {KitElement, define} from "./element.js";

class QpPage extends KitElement {}

define("qp-page", QpPage, {light: String});
