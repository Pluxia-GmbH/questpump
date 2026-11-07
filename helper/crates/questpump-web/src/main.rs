//! QuestPump in the browser (docs/plan.md slice 63, "The engine" and "One desk core"): the addon's own engine
//! (`questpump-engine`, the add-on's pure Lua files on PUC Lua 5.1.5) and the desk (`questpump-desk`, the desktop
//! app's desk logic behind its `Platform` trait) built with Emscripten for `wasm32-unknown-emscripten`, which the Web
//! Worker of play.questpump.com (`web/desk-worker.js`) loads, so the browser composes, judges and folds with the same
//! Lua on the same Lua as the desktop app, and runs the same desk. The browser's platform is `web::Web`; its own part,
//! the `play` topic that only the browser publishes, is `play::Play`.
//!
//! The door to JavaScript, the module's exports (build.rs), each string NUL-terminated UTF-8:
//!
//! | export | takes | answers |
//! |---|---|---|
//! | `qp_open` | `{"media", "phones", "sync", "name"}` | the desk opened (again: a new one) on the data folder the Worker mounted (`/data`, `IndexedDB`) with the engine loaded on the host's clock and zone, every topic published and the fold of every row asked |
//! | `qp_subscribe` | | the page subscribed, every topic's value: `{"ok": {"topics": {name: {rev, data}}}}` |
//! | `qp_intent` | `{"type", …}` | the intent taken by its area (the browser's `play.*`, else the desk's) |
//! | `qp_due` | | the pump's wake handled: whatever is due happens |
//! | `qp_look` | | the look every 2 s: the views, the pump's gates, the first five minutes, the places' times, the sync's due calls |
//! | `qp_answer` | `{"id", "status", "retry", "body"}` or `{"id", "error"}` | a sync call's answer taken by the desk's sync |
//! | `qp_live` | `{"gen", "text"}` or `{"gen", "end": {"code", "connected"}}` | the live socket's message or end taken by the desk's sync |
//! | `qp_free` | an answer | nothing: the answer's memory goes back |
//!
//! Every answer but `qp_subscribe`'s and `qp_free`'s is `{"ok": {"wake": <unix ms or null>}}`, the pump's next wake,
//! which the Worker sleeps until before it calls `qp_due`. A refusal answers `{"error": {"kind", "message"}}` (`kind`
//! `refused`, `shape` or `panicked`). The caller frees the strings it passed with `_free` and every answer with
//! `qp_free`. The imports (`src/imports.js`): `qp_now`, `qp_offset_at` and `qp_report` as the engine's; `qp_post`, a
//! message to the page (`{"msg": {topic, rev, data}}` a topic's change, `{"audio": …}` a sound, a take, a stop or the
//! output held, `{"wipe": "out"}` signed out and `{"wipe": "removed"}` removed from the account: the browser's data
//! goes, `web::wipe`; `{"open": url}` a site's page in a new tab); `qp_persist`, the data folder changed; `qp_net`, a
//! sync call (`{"fetch": {id, method, url, token, body}}`) or the live socket opened (`{"live": {gen, url, hello}}`) or
//! closed (`{"live": null}`), whose answers come back through `qp_answer` and `qp_live`.

#[cfg(target_os = "emscripten")]
mod door;
#[cfg(target_os = "emscripten")]
mod imports;
#[cfg(target_os = "emscripten")]
mod play;
#[cfg(target_os = "emscripten")]
mod web;

fn main() {}
