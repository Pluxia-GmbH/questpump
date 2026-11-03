//! The addon's release feed and its install on this computer (docs/companion.md, "The addon on this computer";
//! rust-helper `mirror-installs-and-resets.md`, "The addon's feed and install"). Every addon tag puts three files on
//! the public repository's permanent `helper-updater` release beside the helper's `latest.json`: `QuestPump-addon.zip`
//! (the addon packager's zip, byte for byte the one the addon stores get), `addon.json` (`Feed`) and `addon.json.sig`,
//! made with the helper's own updater key by `tauri signer sign`, so `verify` checks it as `tauri-plugin-updater`
//! checks a download: the signature's base64 text decoded, minisign's signature over the file's bytes against
//! `plugins.updater.pubkey` (legacy signatures allowed, as the plugin allows them), and a version the trusted comment
//! names equal to the feed's. Versions compare as `x.y.z` after a leading `v` (`version`); one that does not parse (a
//! checkout's `@project-version@`, `dev`) is never older or newer than another.
//!
//! The feed's zip must be the release's own (`verify` takes its address), and it is unpacked (`unpack`) into a staging
//! folder only when every check holds: each entry under one of the three folders the release ships (`FOLDERS`), no
//! `..`, no absolute path, no link, every file of the feed's list there once with its size and sha256, and
//! `QuestPump/QuestPump.toc`'s `## Version` the feed's. The helper keeps a record of each install it made (`Record`),
//! which it manages only while the folders hold exactly the record's files (`tree`; a game that has no AddOns folder
//! yet holds none); a folder that is or holds a link or a `.git` (a developer's `make link` folder, a clone) is never
//! written. `swap` puts the staged folders in place by renames on the same volume (AddOns made first where a new
//! install has none), putting the old ones back on any error and marking the staging folder `DONE` once the last new
//! one is in, and `recover` puts back what a swap cut short left aside, or, once the swap finished, only removes the
//! staging folder. Nothing here touches `QuestPump_Mirror`, `WTF` or a save.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::{BufWriter, Read as _, Write as _};
use std::path::{Path, PathBuf};

use base64::Engine as _;
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};

use crate::mirror::{rename_retried, toc_field};
use crate::zip::{self, Archive, ZipError};

/// The folders the release zip ships into `Interface/AddOns`, in the order a swap moves the old ones out; the new ones
/// go in the other way round, QuestPump's own last, so a QuestPump folder of the new version means the others came
/// first.
pub const FOLDERS: [&str; 3] = ["QuestPump", "QuestPump_FigArmor", "QuestPump_Lines"];
/// The addon's TOC in the zip, whose `## Version` must be the feed's.
pub const TOC: &str = "QuestPump/QuestPump.toc";
/// The staging folder beside AddOns in the flavour's `Interface` folder (the client walks AddOns alone): the zip, the
/// new folders (`NEW`), the old ones a swap moved aside (`OLD`), once every old one is out the names of the new ones
/// going in (`MOVING`), and once nothing in `OLD` is to go back (the last new folder is in, or a cut swap was put
/// right) the marker `DONE`, which `recover` reads as "only remove".
pub const STAGING: &str = ".questpump-update";
pub const NEW: &str = "new";
pub const OLD: &str = "old";
const MOVING: &str = "moving";
const DONE: &str = "done";
/// The files a system leaves in a folder someone opened (Finder's, Explorer's), which say nothing about who installed
/// the addon.
const LITTER: [&str; 3] = [".DS_Store", "Thumbs.db", "desktop.ini"];

/// Why the feed, the zip or a folder was refused.
#[derive(Debug, thiserror::Error)]
pub enum AddonError {
	#[error(transparent)]
	Io(#[from] std::io::Error),
	#[error(transparent)]
	Zip(#[from] ZipError),
	#[error("addon.json: {0}")]
	Json(#[from] serde_json::Error),
	#[error("the signature: {0}")]
	Signature(String),
	#[error("{0}")]
	Bad(String),
}

fn bad(why: impl Into<String>) -> AddonError {
	AddonError::Bad(why.into())
}

/// `addon.json`, as the release workflow writes it.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Feed {
	/// The tag without its leading `v`, such as `1.0.0`.
	pub version: String,
	/// When the release was made, RFC 3339.
	pub pub_date: String,
	/// The TOC's `## Interface` numbers, Forever's first.
	pub interfaces: Vec<u32>,
	/// Whether the phone apps can be installed from the stores (the repository variable `PHONE_APPS`).
	#[serde(default)]
	pub phone_apps: bool,
	pub zip: Zip,
	/// Every file in the zip.
	pub files: Vec<FeedFile>,
}

/// The zip on the release.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Zip {
	pub url: String,
	pub size: u64,
	pub sha256: String,
}

/// One file in the zip: its path there, its size and its sha256.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct FeedFile {
	pub path: String,
	pub size: u64,
	pub sha256: String,
}

impl Feed {
	/// When the release was made, unix seconds; none for a date that does not read.
	#[must_use]
	pub fn out_at(&self) -> Option<i64> {
		time::OffsetDateTime::parse(&self.pub_date, &time::format_description::well_known::Rfc3339)
			.ok()
			.map(time::OffsetDateTime::unix_timestamp)
	}

	/// Each file's sha256 by its path: what the install record keeps.
	#[must_use]
	pub fn hashes(&self) -> BTreeMap<String, String> {
		self.files.iter().map(|f| (f.path.clone(), f.sha256.clone())).collect()
	}

	/// The feed's own rules: a version that parses, an interface, the zip at the release's own address (`zip_url`, the
	/// feed's folder on github.com: the helper talks to no other host), sha256s in hex, and every file at a safe path
	/// of the three folders, once, the TOC among them.
	fn check(&self, zip_url: &str) -> Result<(), AddonError> {
		if version(&self.version).is_none() {
			return Err(bad(format!("version {:?} does not read", self.version)));
		}
		if self.interfaces.is_empty() {
			return Err(bad("no interface"));
		}
		if self.zip.url != zip_url {
			return Err(bad(format!("the zip at {:?}, not {zip_url}", self.zip.url)));
		}
		if !is_sha256(&self.zip.sha256) {
			return Err(bad("the zip's sha256"));
		}
		let mut seen = BTreeSet::new();
		for f in &self.files {
			if !safe_path(&f.path, false) || !is_sha256(&f.sha256) || !seen.insert(f.path.as_str()) {
				return Err(bad(format!("{:?}: a file the feed may not name", f.path)));
			}
		}
		if !seen.contains(TOC) {
			return Err(bad(format!("no {TOC}")));
		}
		Ok(())
	}
}

fn is_sha256(hex: &str) -> bool {
	hex.len() == 64 && hex.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// A version as `x.y.z` after a leading `v`; none for anything else.
#[must_use]
pub fn version(v: &str) -> Option<[u32; 3]> {
	let v = v.trim();
	let mut parts = v.strip_prefix('v').unwrap_or(v).split('.');
	let mut part = || {
		let p = parts.next().filter(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()))?;
		p.parse().ok()
	};
	let out = [part()?, part()?, part()?];
	parts.next().is_none().then_some(out)
}

/// `candidate` is a later version than `than`; never when either does not parse.
#[must_use]
pub fn newer(candidate: &str, than: &str) -> bool {
	matches!((version(candidate), version(than)), (Some(c), Some(t)) if c > t)
}

/// The same version; never when either does not parse.
#[must_use]
pub fn same(a: &str, b: &str) -> bool {
	matches!((version(a), version(b)), (Some(a), Some(b)) if a == b)
}

/// Text that came as base64, as the updater's key and signatures do.
fn from_base64(text: &str) -> Result<String, AddonError> {
	let bytes = base64::engine::general_purpose::STANDARD
		.decode(text.trim())
		.map_err(|e| AddonError::Signature(format!("not base64: {e}")))?;
	String::from_utf8(bytes).map_err(|_| AddonError::Signature("not UTF-8".to_owned()))
}

/// `addon.json`'s bytes checked against its `.sig` (the file's text) and the updater's public key (the content of
/// `plugins.updater.pubkey`), then read and its rules checked, the zip's address `zip_url` among them.
///
/// # Errors
/// The key or the signature does not decode, the signature is not the key's or not over these bytes, the trusted
/// comment names another version, or the feed does not read or breaks its rules.
pub fn verify(body: &[u8], sig: &str, pubkey: &str, zip_url: &str) -> Result<Feed, AddonError> {
	let key = minisign_verify::PublicKey::decode(&from_base64(pubkey)?)
		.map_err(|e| AddonError::Signature(format!("the key: {e}")))?;
	let signature =
		minisign_verify::Signature::decode(&from_base64(sig)?).map_err(|e| AddonError::Signature(e.to_string()))?;
	key.verify(body, &signature, true).map_err(|e| AddonError::Signature(e.to_string()))?;
	let feed: Feed = serde_json::from_slice(body)?;
	feed.check(zip_url)?;
	// covered by the signature once `verify` passed: a version it names must be the feed's (the plugin's rule)
	if let Some(signed) = signature.trusted_comment().split('\t').find_map(|f| f.strip_prefix("version:"))
		&& !(same(signed, &feed.version) || signed == feed.version)
	{
		return Err(AddonError::Signature(format!("signed for {signed}, the feed says {}", feed.version)));
	}
	Ok(feed)
}

/// A path the release may hold, `/`-separated: under one of `FOLDERS`, every part a plain name (no `.`, `..`, empty
/// part, backslash, colon or control character), never absolute; a folder entry (`dir`, its `/` taken off) may be one
/// of the folders itself, a file is always inside one.
#[must_use]
pub fn safe_path(path: &str, dir: bool) -> bool {
	let parts: Vec<&str> = path.split('/').collect();
	let plain = |p: &&str| {
		!p.is_empty() && *p != "." && *p != ".." && !p.chars().any(|c| c == '\\' || c == ':' || c.is_control())
	};
	parts.iter().all(plain) && FOLDERS.contains(&parts[0]) && (dir || parts.len() > 1)
}

/// Unpacks the release zip into `into`, made anew: every entry checked as the module says, and the TOC's version the
/// feed's. The caller has checked the zip's own sha256 and size.
///
/// # Errors
/// An entry the feed does not allow, a file that differs from the list or is missing, a TOC of another version, or a
/// file that cannot be written.
pub fn unpack(zip_path: &Path, feed: &Feed, into: &Path) -> Result<(), AddonError> {
	if into.exists() {
		std::fs::remove_dir_all(into)?;
	}
	std::fs::create_dir_all(into)?;
	let wanted: BTreeMap<&str, &FeedFile> = feed.files.iter().map(|f| (f.path.as_str(), f)).collect();
	let mut archive = Archive::open(zip_path)?;
	let entries = archive.entries().to_vec();
	let mut seen = BTreeSet::new();
	for entry in &entries {
		if entry.is_link() {
			return Err(bad(format!("{}: a link", entry.name)));
		}
		if entry.is_dir() {
			let name = entry.name.trim_end_matches('/');
			if !safe_path(name, true) {
				return Err(bad(format!("{}: a folder outside the addon's", entry.name)));
			}
			std::fs::create_dir_all(into.join(name))?;
			continue;
		}
		let want = wanted.get(entry.name.as_str()).ok_or_else(|| bad(format!("{}: not in the feed", entry.name)))?;
		if !seen.insert(entry.name.as_str()) {
			return Err(bad(format!("{}: twice in the zip", entry.name)));
		}
		if entry.size != want.size {
			return Err(bad(format!("{}: {} bytes, the feed says {}", entry.name, entry.size, want.size)));
		}
		let path = into.join(&entry.name);
		if let Some(parent) = path.parent() {
			std::fs::create_dir_all(parent)?;
		}
		let mut out = BufWriter::new(File::create_new(&path)?);
		let got = archive.copy(entry, &mut out)?;
		out.flush()?;
		if got != want.sha256 {
			return Err(bad(format!("{}: its sha256 is not the feed's", entry.name)));
		}
	}
	if let Some(missing) = wanted.keys().find(|p| !seen.contains(*p)) {
		return Err(bad(format!("{missing}: not in the zip")));
	}
	let toc = std::fs::read_to_string(into.join(TOC))?;
	let shipped = toc_field(&toc, "Version").unwrap_or_default(); // copy-lint: not shown (a TOC field)
	if !same(shipped, &feed.version) {
		return Err(bad(format!("{TOC} says {shipped:?}, the feed {}", feed.version)));
	}
	Ok(())
}

/// A file's sha256 in lower-case hex.
///
/// # Errors
/// The file does not read.
pub fn sha256_file(path: &Path) -> std::io::Result<String> {
	let mut file = File::open(path)?;
	let mut hasher = Sha256::new();
	let mut buf = vec![0_u8; 64 << 10];
	loop {
		let n = file.read(&mut buf)?;
		if n == 0 {
			return Ok(zip::hex(&hasher.finalize()));
		}
		hasher.update(&buf[..n]);
	}
}

/// What the addon's folders in an AddOns folder hold.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tree {
	/// A folder that is or holds a link, or holds a `.git`: never written.
	Linked,
	/// Every file by its path from AddOns (`QuestPump/QuestPump.toc`) with its sha256; the system's litter left out.
	Files(BTreeMap<String, String>),
}

/// What the three folders in `addons` hold (an AddOns folder that is itself a link is `Linked`; one that is not there
/// yet, as in a new install until an addon is put there, holds nothing).
///
/// # Errors
/// A folder or a file that does not read.
pub fn tree(addons: &Path) -> std::io::Result<Tree> {
	match std::fs::symlink_metadata(addons) {
		Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Tree::Files(BTreeMap::new())),
		Err(e) => return Err(e),
		Ok(meta) if meta.file_type().is_symlink() => return Ok(Tree::Linked),
		Ok(_) => {}
	}
	let mut files = BTreeMap::new();
	for name in FOLDERS {
		let path = addons.join(name);
		match std::fs::symlink_metadata(&path) {
			Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
			Err(e) => return Err(e),
			Ok(meta) if !meta.is_dir() => return Ok(Tree::Linked),
			Ok(_) => {
				if !walk(&path, name, &mut files)? {
					return Ok(Tree::Linked);
				}
			}
		}
	}
	Ok(Tree::Files(files))
}

/// Every file under `dir` into `files` by its path `rel`; false at a link, a `.git` or anything that is no plain file
/// or folder.
fn walk(dir: &Path, rel: &str, files: &mut BTreeMap<String, String>) -> std::io::Result<bool> {
	for entry in std::fs::read_dir(dir)? {
		let entry = entry?;
		let name = entry.file_name().to_string_lossy().into_owned();
		let kind = entry.file_type()?;
		if name == ".git" || kind.is_symlink() {
			return Ok(false);
		}
		let path = format!("{rel}/{name}");
		if kind.is_dir() {
			if !walk(&entry.path(), &path, files)? {
				return Ok(false);
			}
		} else if !kind.is_file() {
			return Ok(false);
		} else if !LITTER.contains(&name.as_str()) {
			files.insert(path, sha256_file(&entry.path())?);
		}
	}
	Ok(true)
}

/// An install the helper made: its AddOns folder, the version and every file's sha256, as the feed listed them.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Record {
	pub addons: PathBuf,
	pub version: String,
	pub files: BTreeMap<String, String>,
}

impl Record {
	/// The folders hold exactly what the helper put there: no other updater has rewritten them since.
	#[must_use]
	pub fn matches(&self, tree: &Tree) -> bool {
		matches!(tree, Tree::Files(files) if *files == self.files)
	}
}

/// The staging folder of an AddOns folder: beside it in the flavour's `Interface`.
#[must_use]
pub fn staging(addons: &Path) -> Option<PathBuf> {
	Some(addons.parent()?.join(STAGING))
}

/// Puts the folders staged in `<staging>/new` in place of the ones in `addons` (made first where a new install has no
/// AddOns folder yet): the old ones moved into `<staging>/old`, the names of the new ones written into `MOVING`, the
/// new ones moved in, QuestPump's own last, then `DONE` written (each rename on the same volume, tried again on
/// Windows while a file is held); on any error every folder moved goes back where it was.
///
/// # Errors
/// No new QuestPump folder staged, the step that failed, and whether every old folder went back.
pub fn swap(addons: &Path, staging: &Path) -> std::io::Result<()> {
	let (new, old) = (staging.join(NEW), staging.join(OLD));
	// QuestPump's own goes in last, so `recover` reads `MOVING` with no QuestPump in `new` as a finished swap
	if !new.join(FOLDERS[0]).is_dir() {
		return Err(std::io::Error::other(format!("no {} folder staged", FOLDERS[0])));
	}
	std::fs::create_dir_all(addons)?;
	std::fs::create_dir_all(&old)?;
	let mut out: Vec<&str> = Vec::new();
	let mut moved_in: Vec<&str> = Vec::new();
	let mut step = || -> std::io::Result<()> {
		for name in FOLDERS {
			if std::fs::symlink_metadata(addons.join(name)).is_ok() {
				rename_retried(&addons.join(name), &old.join(name))?;
				out.push(name);
			}
		}
		let coming: Vec<&str> = FOLDERS.into_iter().rev().filter(|name| new.join(name).is_dir()).collect();
		std::fs::write(staging.join(MOVING), coming.join("\n"))?;
		for name in coming {
			rename_retried(&new.join(name), &addons.join(name))?;
			moved_in.push(name);
		}
		Ok(())
	};
	let Err(e) = step() else {
		// every folder is in: a marker that does not write leaves `MOVING` with no QuestPump in `new`, which `recover`
		// reads the same way
		let _ = std::fs::write(staging.join(DONE), "");
		return Ok(());
	};
	let mut lost = Vec::new();
	for name in moved_in.iter().rev() {
		if let Err(back) = rename_retried(&addons.join(name), &new.join(name)) {
			lost.push(format!("{name}: {back}"));
		}
	}
	for name in out.iter().rev() {
		if let Err(back) = rename_retried(&old.join(name), &addons.join(name)) {
			lost.push(format!("{name} (kept in {}): {back}", old.display()));
		}
	}
	if lost.is_empty() {
		Err(e)
	} else {
		Err(std::io::Error::other(format!("{e}; and not put back: {}", lost.join(", "))))
	}
}

/// What a swap left in `staging` put right, then the staging folder removed. A swap cut short (the helper quit or the
/// computer stopped mid-rename) has each new folder `MOVING` names that is in `addons` and no longer in `new` moved
/// back into `new`, then every old folder whose place in `addons` is empty put back, then `DONE` written. A finished
/// swap (`DONE` there, or `MOVING` with QuestPump's own new folder, which goes in last, no longer in `new`: cut between
/// that rename and the marker) only has the folder removed, never an old folder put back; `old` and `new` go before
/// the markers, so a removal cut short still reads as finished. Answers the folders put back.
///
/// # Errors
/// A rename, the marker or a removal failed.
pub fn recover(addons: &Path, staging: &Path) -> std::io::Result<Vec<&'static str>> {
	if std::fs::symlink_metadata(staging).is_err() {
		return Ok(Vec::new());
	}
	let (new, old) = (staging.join(NEW), staging.join(OLD));
	let moving = std::fs::read_to_string(staging.join(MOVING)).ok();
	let done = staging.join(DONE).exists() || (moving.is_some() && !new.join(FOLDERS[0]).exists());
	let mut back = Vec::new();
	if !done {
		if let Some(coming) = &moving {
			for name in FOLDERS.into_iter().filter(|name| coming.lines().any(|line| line == *name)) {
				if std::fs::symlink_metadata(addons.join(name)).is_ok() && !new.join(name).exists() {
					rename_retried(&addons.join(name), &new.join(name))?;
				}
			}
		}
		for name in FOLDERS {
			if old.join(name).is_dir() && std::fs::symlink_metadata(addons.join(name)).is_err() {
				rename_retried(&old.join(name), &addons.join(name))?;
				back.push(name);
			}
		}
		std::fs::write(staging.join(DONE), "")?;
	}
	for folder in [old.as_path(), new.as_path(), staging] {
		match std::fs::remove_dir_all(folder) {
			Err(e) if e.kind() != std::io::ErrorKind::NotFound => return Err(e),
			_ => {}
		}
	}
	Ok(back)
}
