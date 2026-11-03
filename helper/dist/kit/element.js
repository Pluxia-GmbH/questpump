// The kit's base (docs/ui.md, "What stays native, and the kit"): every component of the phones' kit is a custom element
// named qp-<kebab-name>, in the light DOM, drawn only from tokens.css through kit.css. An interactive one renders a
// native control inside (a <button>, an <input>, a <dialog>), so Space and Enter, the focus, hover and :focus-visible
// and the accessibility tree are the platform's own; it keeps that control as `control` and hands it the focus.
// Configuration comes as attributes (reflected as properties: `label`, `disabled`) and data as properties; an element
// builds its children once and redraws them on every change.

/** A component: `build` once when first connected, `update` then and on every attribute change. */
export class KitElement extends HTMLElement {
	connectedCallback() {
		this.ensure();
		this.update();
	}

	/** Builds the children once, whether or not the element is in the page yet. */
	ensure() {
		if (!this.built) {
			this.built = true;
			this.build();
		}
	}

	attributeChangedCallback() {
		if (this.built) this.update();
	}

	/** Makes the element's children. */
	build() {}

	/** Draws the element from its attributes and properties. */
	update() {}

	/** An interactive element focuses its native control. */
	focus(options) {
		if (this.control) this.control.focus(options);
		else super.focus(options);
	}
}

/** Defines a component once, its attributes (kebab-case, by type: String, Number or Boolean) reflected as camelCase
 * properties and observed. */
export function define(name, cls, attributes = {}) {
	Object.defineProperty(cls, "observedAttributes", {get: () => Object.keys(attributes)});
	for (const [attr, type] of Object.entries(attributes)) {
		const prop = attr.replace(/-([a-z])/g, (_, c) => c.toUpperCase());
		if (Object.hasOwn(cls.prototype, prop)) continue;
		Object.defineProperty(cls.prototype, prop, {
			get() {
				if (type === Boolean) return this.hasAttribute(attr);
				const value = this.getAttribute(attr);
				return type === Number ? (value === null ? null : Number(value)) : value;
			},
			set(value) {
				if (type === Boolean) this.toggleAttribute(attr, Boolean(value));
				else if (value === null || value === undefined) this.removeAttribute(attr);
				else this.setAttribute(attr, String(value));
			},
		});
	}
	if (!customElements.get(name)) customElements.define(name, cls);
}

/** An element: `h("button", {class: "qp-x", text: "…", "on:click": fn, hidden: true}, ...children)`. Attributes by
 * name (a boolean sets or leaves out), `text` its text, `on:<event>` a listener; children are nodes or strings. */
export function h(tag, props = {}, ...children) {
	const el = tag.includes(":") ? document.createElementNS("http://www.w3.org/2000/svg", tag.slice(4))
		: document.createElement(tag);
	for (const [key, value] of Object.entries(props)) {
		if (value === undefined || value === null || value === false) continue;
		if (key === "text") el.textContent = value;
		else if (key.startsWith("on:")) el.addEventListener(key.slice(3), value);
		else el.setAttribute(key, value === true ? "" : String(value));
	}
	for (const child of children.flat()) {
		if (child === null || child === undefined || child === false) continue;
		el.append(child instanceof Node ? child : String(child));
	}
	return el;
}

/** Sets CSS properties on an element through the CSSOM (the CSP refuses a `style` attribute) and returns it:
 * `styled(h("i"), {"flex-grow": 3, "--qp-share": 0.5})`. */
export function styled(el, props) {
	for (const [name, value] of Object.entries(props)) el.style.setProperty(name, String(value));
	return el;
}

/** The qualities' tones' names (contract/sheet.json `qualities`, 1 Starter to 7 Artifact). */
const QUALITIES = ["starter", "common", "uncommon", "rare", "epic", "legendary", "artifact"];

/** A quality's tone's name for its CSS custom properties (--qp-quality-<name>), or null for none. */
export function qualityName(quality) {
	return QUALITIES[quality - 1] ?? null;
}

/** A tone's name's quality (1 Starter to 7 Artifact; a Saga day's word is named by its tone), or null for none. */
export function qualityNumber(name) {
	const i = QUALITIES.indexOf(name);
	return i < 0 ? null : i + 1;
}

const missing = new Set();

/** A token's number as tokens.css gives it on `el` (a length's px, a duration's ms, a count): the kit's numbers come
 * only from there. One tokens.css lacks is a stale or broken page, said once on the console; its number is NaN (as on
 * an element out of the page, which has no style to read). */
export function token(el, name) {
	const value = parseFloat(getComputedStyle(el).getPropertyValue(name));
	if (Number.isNaN(value) && el.isConnected && !missing.has(name)) {
		missing.add(name);
		console.error(`kit: tokens.css has no ${name}`);
	}
	return value;
}

/** The animation an element runs now, in ms (0 for none): a step that waits for its end waits no longer than this, since
 * a window the system hides runs no animation and so ends none. */
export function animationMs(el) {
	const style = getComputedStyle(el);
	if (style.animationName === "none") return 0;
	const one = (s) => (s.trim().endsWith("ms") ? parseFloat(s) : parseFloat(s) * 1000) || 0;
	return Math.max(...style.animationDuration.split(",").map(one)) + Math.max(...style.animationDelay.split(",").map(one));
}

/** Fires a component's own event, which bubbles to the view (and the router: qp-tab, qp-menu, qp-back). */
export function emit(el, type, detail) {
	el.dispatchEvent(new CustomEvent(type, {bubbles: true, detail}));
}
