//! How Emscripten links the browser's engine and desk (docs/plan.md slice 63): an ES module factory for a Web Worker
//! (and for Node, where the engine's replay runs by hand), no `eval` (the page's CSP allows WebAssembly's compile
//! alone, its `'wasm-unsafe-eval'`), the heap growing as a fold needs it, the 8 MiB stack the desktop's engine thread
//! has (Lua recurses through `pcall` and `table.sort` on it), the file system with `IDBFS`, on which the Worker mounts
//! the desk's data folder in the browser's `IndexedDB` (`FS`, `FS.filesystems.IDBFS`), the functions of the door, and
//! the host's clock, zone, report, page, keeping and network through `src/imports.js`; and the desktop app's version,
//! which the browser runs. Nothing on the computer's targets, where the crate is empty.

use std::error::Error;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
	if std::env::var("CARGO_CFG_TARGET_OS")? != "emscripten" {
		return Ok(());
	}
	let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?);
	// the desktop app's version, which the browser runs: its About page's and its report's
	let app = manifest.join("../../src-tauri/Cargo.toml");
	println!("cargo::rerun-if-changed={}", app.display());
	let version = std::fs::read_to_string(&app)?
		.lines()
		.find_map(|l| l.strip_prefix("version = \"")?.strip_suffix('"').map(str::to_owned))
		.ok_or("helper/src-tauri/Cargo.toml names no version")?;
	println!("cargo::rustc-env=QP_APP_VERSION={version}");
	let imports = manifest.join("src").join("imports.js");
	println!("cargo::rerun-if-changed={}", imports.display());
	for arg in [
		"-sMODULARIZE=1",
		"-sEXPORT_ES6=1",
		"-sEXPORT_NAME=questpumpEngine",
		"-sENVIRONMENT=worker,node",
		"-sDYNAMIC_EXECUTION=0",
		"-sFORCE_FILESYSTEM=1",
		"-lidbfs.js",
		"-sALLOW_MEMORY_GROWTH=1",
		"-sSTACK_SIZE=8MB",
		"-sEXPORTED_FUNCTIONS=_main,_qp_open,_qp_subscribe,_qp_intent,_qp_due,_qp_look,_qp_answer,_qp_live,_qp_free,_malloc,_free",
		"-sEXPORTED_RUNTIME_METHODS=stringToNewUTF8,UTF8ToString,FS",
	] {
		println!("cargo::rustc-link-arg-bins={arg}");
	}
	println!("cargo::rustc-link-arg-bins=--js-library={}", imports.display());
	Ok(())
}
