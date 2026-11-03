//! How Emscripten links the browser's engine (docs/plan.md slice 63): an ES module factory for a Web Worker (and for
//! Node, where the engine's replay runs by hand), no `eval` (the page's CSP allows WebAssembly's compile alone, its
//! `'wasm-unsafe-eval'`), the heap growing as a fold needs it, the 8 MiB stack the desktop's engine thread has (Lua
//! recurses through `pcall` and `table.sort` on it), no file system, the four functions of the door, and the host's
//! clock, zone and report through `src/imports.js`. Nothing on the computer's targets, where the crate is empty.

use std::error::Error;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
	if std::env::var("CARGO_CFG_TARGET_OS")? != "emscripten" {
		return Ok(());
	}
	let imports = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?).join("src").join("imports.js");
	println!("cargo::rerun-if-changed={}", imports.display());
	for arg in [
		"-sMODULARIZE=1",
		"-sEXPORT_ES6=1",
		"-sEXPORT_NAME=questpumpEngine",
		"-sENVIRONMENT=worker,node",
		"-sDYNAMIC_EXECUTION=0",
		"-sFILESYSTEM=0",
		"-sALLOW_MEMORY_GROWTH=1",
		"-sSTACK_SIZE=8MB",
		"-sEXPORTED_FUNCTIONS=_main,_qp_start,_qp_call,_qp_free,_malloc,_free",
		"-sEXPORTED_RUNTIME_METHODS=stringToNewUTF8,UTF8ToString",
	] {
		println!("cargo::rustc-link-arg-bins={arg}");
	}
	println!("cargo::rustc-link-arg-bins=--js-library={}", imports.display());
	Ok(())
}
