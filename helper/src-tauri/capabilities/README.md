# The capabilities folder, kept empty

This folder must exist although the app has no capability file (2026-10-13). tauri-build watches it
(`cargo:rerun-if-changed=<src-tauri>/capabilities`) while it exists or `app > security > capabilities` in
`tauri.conf.json` lists none, as here (2.7.4, `acl.rs`), and Cargo counts a missing watched path as changed at every
look: until this folder came, every `cargo check`, `clippy`, `test` and `build` ran `build.rs` again, whose
`tauri_build::build()` copies the bundle's resources (about 140 MB) into `target/` once more, and then compiled
`questpump-helper` again (Cargo: "the file `src-tauri/capabilities` is missing"; tauri PR #15918, issue #15829).

tauri-build reads only `.json`, `.json5` and `.toml` files here as capabilities, so this file grants nothing. The page
calls only the app's own commands and gets its state through a `Channel`, which no capability gates
(`.claude/skills/rust-helper`, rule 4): a capability file comes here only with a plugin's JavaScript API.
