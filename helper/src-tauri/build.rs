fn main() {
	// screencapturekit's Swift bridge links @rpath/libswift_Concurrency.dylib, and a dependency's link arguments never
	// reach this crate's binaries (Cargo passes only its libraries and search paths on): without this rpath the helper
	// stops at launch with "Library not loaded". The runtime is the system's, in /usr/lib/swift since macOS 12.
	if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
		println!("cargo:rustc-link-arg=-Wl,-rpath,/usr/lib/swift");
	}
	tauri_build::build();
}
