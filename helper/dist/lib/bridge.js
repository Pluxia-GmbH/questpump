// The page's line to QuestPump on this computer (docs/plan.md slice 35, "The UI stack"): Rust owns the state and sends
// it as topics through one channel (helper/crates/questpump-desk/src/publisher.rs), each a whole value at a rev that counts every
// change; the page keeps the newest of each and hands it to its subscribers. The page answers with intents
// ({type: "<area>.<what>", ...}: desk_intent, answered once the area's handler took it) and the Menu pages' own app
// commands. No plugin command is called, so the page needs no capability file.

const topics = new Map(); // a topic's name -> {rev, data}
const listeners = new Map(); // a topic's name -> Set of functions

function tauri() {
	return window.__TAURI__.core;
}

/** Takes a topic's value when it is newer than the one held, and tells its subscribers: those subscribed when it came
 * (one that subscribes meanwhile had the value from subscribe), each still subscribed, and one's error never keeps the
 * value from the others. */
function take(name, rev, data) {
	const held = topics.get(name);
	if (held && held.rev >= rev) return;
	topics.set(name, {rev, data});
	const now = listeners.get(name);
	if (!now) return;
	for (const fn of [...now]) {
		if (!now.has(fn)) continue;
		try {
			fn(data);
		} catch (error) {
			console.error(error);
		}
	}
}

/** Subscribes this page to every topic: its channel first, then every topic's value as it stands. */
export async function start() {
	const core = tauri();
	const channel = new core.Channel();
	channel.onmessage = (msg) => take(msg.topic, msg.rev, msg.data);
	const boot = await core.invoke("desk_subscribe", {on: channel});
	for (const [name, held] of Object.entries(boot.topics)) take(name, held.rev, held.data);
}

/** A topic's value now, or undefined before Rust published it. */
export function get(name) {
	return topics.get(name)?.data;
}

/** Calls `fn` with the topic's value now (when there is one) and on every change; returns the way to stop. Its error is
 * said on the console and never reaches the caller, as on a change (`take`): a view that cannot draw a value never
 * stops the views made after it. */
export function subscribe(name, fn) {
	if (!listeners.has(name)) listeners.set(name, new Set());
	listeners.get(name).add(fn);
	if (topics.has(name)) {
		try {
			fn(topics.get(name).data);
		} catch (error) {
			console.error(error);
		}
	}
	return () => listeners.get(name).delete(fn);
}

/** Sends an intent: resolves once its area's handler took it, rejects with why it did not. */
export function intent(type, args = {}) {
	return tauri().invoke("desk_intent", {intent: {type, ...args}});
}

/** Calls an app command (the Menu pages' own: allow_screen, new_pairing, set_strap, …). */
export function call(command, args = {}) {
	return tauri().invoke(command, args);
}

/** Calls an app command with raw bytes as its body (save_image) and the headers that name them. */
export function callBytes(command, bytes, headers = {}) {
	return tauri().invoke(command, bytes, {headers});
}
