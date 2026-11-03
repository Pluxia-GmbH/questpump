//! Embeds the addon's own engine (docs/plan.md slice 35, "The engine is the addon's own Lua"): the pure files
//! `tools/host.lua`'s `H.FILES` line names, checked against `QuestPump.toc`'s order, then `tools/host.lua` and
//! `tools/apppump.lua`, each through `include_str!` of its path in the repository, and the SHA-256 of them all
//! (`DIGEST`). A change to any of them, or to the TOC, builds the crate again.

use std::error::Error;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

const HOST: &str = "tools/host.lua";
const APPPUMP: &str = "tools/apppump.lua";
const TOC: &str = "QuestPump.toc";

type Result<T> = std::result::Result<T, Box<dyn Error>>;

/// The names on `tools/host.lua`'s one `H.FILES = { ... }` line, in its order.
fn listed(host: &str) -> Result<Vec<String>> {
	let line = host
		.lines()
		.map(str::trim)
		.find(|l| l.starts_with("H.FILES = {"))
		.ok_or_else(|| format!("{HOST} has no `H.FILES = {{ ... }}` line"))?;
	let inner = line
		.strip_prefix("H.FILES = {")
		.and_then(|rest| rest.strip_suffix('}'))
		.ok_or_else(|| format!("{HOST}: H.FILES is not one line: {line}"))?;
	let mut files = Vec::new();
	for item in inner.split(',').map(str::trim).filter(|i| !i.is_empty()) {
		let name = item
			.strip_prefix('"')
			.and_then(|i| i.strip_suffix('"'))
			.filter(|n| Path::new(n).extension().is_some_and(|e| e == "lua") && !n.contains(['"', '\\']))
			.ok_or_else(|| format!("{HOST}: H.FILES holds {item}, not a quoted .lua path"))?;
		files.push(name.to_owned());
	}
	if files.is_empty() {
		return Err(format!("{HOST}: H.FILES is empty").into());
	}
	Ok(files)
}

/// Fails unless every file is a line of the TOC, in the TOC's order (the order the client loads them in).
fn in_toc_order(files: &[String], toc: &str) -> Result<()> {
	let lines: Vec<String> = toc
		.lines()
		.map(str::trim)
		.filter(|l| !l.is_empty() && !l.starts_with('#'))
		.map(|l| l.replace('\\', "/"))
		.collect();
	let mut last = None;
	for file in files {
		let at = lines.iter().position(|l| l == file).ok_or_else(|| format!("{file} is not a line of {TOC}"))?;
		if last.is_some_and(|before| at <= before) {
			return Err(format!("{HOST}: {file} comes before the file it follows in {TOC}").into());
		}
		last = Some(at);
	}
	Ok(())
}

fn main() -> Result<()> {
	let root = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?).join("../../..");
	let read = |name: &str| -> Result<(String, String)> {
		let path = root.join(name);
		println!("cargo::rerun-if-changed={}", path.display());
		let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
		Ok((path.display().to_string(), text))
	};
	let (_, host) = read(HOST)?;
	let files = listed(&host)?;
	in_toc_order(&files, &read(TOC)?.1)?;

	// The digest reads a line end as Lua's lexer does (a checkout with CRLF names the same sources), each file as its
	// name, a NUL, its text and a NUL, in the order they load.
	let mut digest = Sha256::new();
	let mut out = String::from("/// The addon's pure files, in `QuestPump.toc`'s order: (path, source).\n");
	out.push_str("pub(crate) const FILES: &[(&str, &str)] = &[\n");
	let mut embed = |name: &str| -> Result<String> {
		let (path, text) = read(name)?;
		digest.update(name.as_bytes());
		digest.update([0]);
		digest.update(text.replace("\r\n", "\n").as_bytes());
		digest.update([0]);
		Ok(format!("include_str!({path:?})"))
	};
	for file in &files {
		let source = embed(file)?;
		writeln!(out, "\t({file:?}, {source}),")?;
	}
	out.push_str("];\n");
	writeln!(out, "/// `{HOST}`: the files' host.\npub(crate) const HOST: &str = {};", embed(HOST)?)?;
	writeln!(out, "/// `{APPPUMP}`: the app pump's driver.\npub(crate) const APPPUMP: &str = {};", embed(APPPUMP)?)?;
	let mut hex = String::new();
	for b in digest.finalize() {
		write!(hex, "{b:02x}")?;
	}
	writeln!(out, "/// The SHA-256 of the embedded sources.\npub(crate) const DIGEST: &str = {hex:?};")?;
	std::fs::write(PathBuf::from(std::env::var("OUT_DIR")?).join("embedded.rs"), out)?;
	Ok(())
}
