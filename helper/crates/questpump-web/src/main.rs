//! QuestPump in the browser (docs/plan.md slice 63, "The engine" and "One desk core"): the addon's own engine
//! (`questpump-engine`, the add-on's pure Lua files on PUC Lua 5.1.5) built with Emscripten for
//! `wasm32-unknown-emscripten`, which the Web Worker of questpump.com/play loads, so the browser composes, judges and
//! folds with the same Lua on the same Lua as the desktop app. The desk core (`questpump-desk`, the desktop's desk logic
//! behind a `Platform` trait) joins after the desktop app's release; until then this is the engine's door alone.
//!
//! The door to JavaScript, the module's exports (build.rs), each string NUL-terminated UTF-8:
//!
//! | export | takes | answers |
//! |---|---|---|
//! | `qp_start` | | the engine loaded (again: a new one) on the host's clock, zone and report (`src/imports.js`): `{"ok": {"digest": <the embedded sources' SHA-256>}}` |
//! | `qp_call` | `op`, `args` (JSON) | an op of questpump-engine's door (its `lib.rs` table: `init`, `fold`, `compose`, `quick`, `finish`, `judge`, `summary`, …): `{"ok": <its answer>}` |
//! | `qp_free` | an answer | nothing: the answer's memory goes back |
//!
//! A refusal answers `{"error": {"kind", "message"}}` (`kind` `lua`, with `fileLine` and `traceback`, `refused`, `shape`
//! or `panicked`, as `EngineError` says). The caller frees the strings it passed with `_free` and every answer with
//! `qp_free`.

#[cfg(target_os = "emscripten")]
mod door;

fn main() {}
