// Portrait: the helm in its ring, 64 (tokens' surface.Portrait): the ring piece with the helm inset in it. Decoration;
// the hero's name beside it says who it is.

import {KitElement, define, h} from "./element.js";

class QpPortrait extends KitElement {
	build() {
		this.append(h("qp-art", {name: "portrait_ring", class: "qp-ring"}), h("qp-art", {name: "helm", class: "qp-helm"}));
	}
}

define("qp-portrait", QpPortrait);
