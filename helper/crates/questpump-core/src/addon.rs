//! The addon's release feed and its install on this computer (docs/companion.md, "The addon on this computer";
//! rust-helper `mirror-installs-and-resets.md`, "The addon's feed and install"). Every addon tag puts three files on
//! the public repository's permanent `helper-updater` release beside the helper's `latest.json`:
//! `QuestPump-addon-v2.zip` (the release's three zips joined, the addon's own and its voice and figure packs', each
//! file with the bytes the addon stores hand out: `tools/addon_feed.py`; docs/plan.md slice 71), `addon-v2.json`
//! (`Feed`) and `addon-v2.json.sig`,
//! made with the helper's own updater key by `tauri signer sign`, so `verify` checks it as `tauri-plugin-updater`
//! checks a download: the signature's base64 text decoded, minisign's signature over the file's bytes against
//! `plugins.updater.pubkey` (legacy signatures allowed, as the plugin allows them), and a version the trusted comment
//! names equal to the feed's. Versions compare as `x.y.z` after a leading `v` (`version`); one that does not parse (a
//! checkout's `@project-version@`, `dev`) is never older or newer than another.
//!
//! The feed's zip must be the release's own (`verify` takes its address). A staging folder takes the feed's files file
//! by file: those this computer holds already (`take_local`: the install's own, the desktop app's voice takes), by
//! their sha256, then the rest out of the zip by byte ranges (`plan` on its central directory, `unpack_span` on each
//! span fetched), else, whenever that cannot finish, the whole zip unpacked (`unpack`). Either way nothing is staged
//! unless every check holds: each entry under one of the folders a release ships (`FOLDERS`), no `..`, no
//! absolute path, no link, every file of the feed's list there once with its size and sha256, and
//! `QuestPump/QuestPump.toc`'s `## Version` the feed's (`check_version`). A job puts in only the folders that differ
//! from the feed's (`Scope`, `differing`): a media pack another updater put in with the feed's own files stays as it
//! is, and a missing pack is added to an install whose QuestPump another updater keeps. The helper keeps a record of
//! each install it
//! made (`Record`), which it manages only while the folders hold exactly the record's files (`tree`; a game that has no
//! AddOns folder yet holds none); a folder that is or holds a link or a `.git` (a developer's `make link` folder, a
//! clone) is never written. `swap` puts the staged folders in place by renames on the same volume (AddOns made first
//! where a new install has none), putting the old ones back on any error and marking the staging folder `DONE` once the
//! last new one is in, and `recover` puts back what a swap cut short left aside, or, once the swap finished, only
//! removes the staging folder. Nothing here touches `QuestPump_Mirror`, `WTF` or a save.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read as _, Write as _};
use std::path::{Path, PathBuf};

use base64::Engine as _;
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};

use crate::mirror::{rename_retried, toc_field};
use crate::zip::{self, Archive, Entry, ZipError};

/// The folders a release ships into `Interface/AddOns` (the addon, its figure pack, its voice pack since slice 71,
/// whose takes lay inside QuestPump until then, and the quests' lines; `tools/addon_feed.py` `FOLDERS`), in the order a
/// swap moves the old ones out; the new ones go in the other way round, QuestPump's own last, so a QuestPump folder of
/// the new version means the others came first. A feed need not hold every one (a release from before the packs has
/// no voice folder).
pub const FOLDERS: [&str; 4] = ["QuestPump", "QuestPump_FigArmor", "QuestPump_Voice", "QuestPump_Lines"];
/// The media packs among them: downloads of their own beside the addon (docs/plan.md slice 71), which the helper also
/// adds where one is missing from an install it does not keep.
pub const PACKS: [&str; 2] = ["QuestPump_FigArmor", "QuestPump_Voice"];
/// The addon's TOC in the zip, whose `## Version` must be the feed's.
pub const TOC: &str = "QuestPump/QuestPump.toc";
/// The staging folder beside AddOns in the flavour's `Interface` folder (the client walks AddOns alone): the zip or the
/// span of it fetched last, the new folders (`NEW`), the old ones a swap moved aside (`OLD`), once every old one is out
/// the names of the new ones going in (`MOVING`), and once nothing in `OLD` is to go back (the last new folder is in,
/// or a cut swap was put right) the marker `DONE`, which `recover` reads as "only remove".
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
	#[error("addon-v2.json: {0}")]
	Json(#[from] serde_json::Error),
	#[error("the signature: {0}")]
	Signature(String),
	#[error("{0}")]
	Bad(String),
}

fn bad(why: impl Into<String>) -> AddonError {
	AddonError::Bad(why.into())
}

/// `addon-v2.json`, as the release workflow writes it.
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
	/// of the folders, once, the TOC among them.
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

/// `addon-v2.json`'s bytes checked against its `.sig` (the file's text) and the updater's public key (the content of
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

/// The folder of `FOLDERS` a release path is in.
fn folder_of(path: &str) -> &str {
	path.split('/').next().unwrap_or_default()
}

/// One folder's files out of paths with their sha256.
fn in_folder<'a>(files: impl Iterator<Item = (&'a String, &'a String)>, folder: &str) -> BTreeMap<&'a str, &'a str> {
	files.filter(|(path, _)| folder_of(path) == folder).map(|(path, sha256)| (path.as_str(), sha256.as_str())).collect()
}

/// The feed's folders that `tree` does not hold as the feed has them, missing or with other files: the ones an install
/// or an update puts in. A folder whose files are exactly the feed's stays as it is (a pack another updater put in is
/// never replaced by the same files), and a folder the feed does not hold is left alone. In a tree that is or holds a
/// link every folder of the feed differs; the swap's check refuses such a tree first.
#[must_use]
pub fn differing(feed: &Feed, tree: &Tree) -> Vec<&'static str> {
	FOLDERS
		.into_iter()
		.filter(|folder| {
			let want = in_folder(feed.files.iter().map(|f| (&f.path, &f.sha256)), folder);
			let Tree::Files(held) = tree else { return !want.is_empty() };
			!want.is_empty() && want != in_folder(held.iter(), folder)
		})
		.collect()
}

/// Which of the feed's folders a job puts in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Scope {
	/// Every folder the install does not hold as the feed has it (`differing`): a first install, an update.
	Differing,
	/// The media packs alone (`PACKS`), each only where it is missing or still holds the files the helper itself put
	/// in (`kept`, a record's files) while the feed's differ: an install whose QuestPump is current or another
	/// updater's. A pack another updater put in is never touched.
	Packs { kept: BTreeMap<String, String> },
}

impl Scope {
	/// The folders to put in, as `tree` stands; none for a packs job in a tree that is or holds a link.
	#[must_use]
	pub fn folders(&self, feed: &Feed, tree: &Tree) -> Vec<&'static str> {
		let differ = differing(feed, tree);
		match (self, tree) {
			(Self::Differing, _) => differ,
			(Self::Packs { .. }, Tree::Linked) => Vec::new(),
			(Self::Packs { kept }, Tree::Files(held)) => differ
				.into_iter()
				.filter(|folder| PACKS.contains(folder))
				.filter(|folder| {
					let on_disk = in_folder(held.iter(), folder);
					on_disk.is_empty() || on_disk == in_folder(kept.iter(), folder)
				})
				.collect(),
		}
	}
}

/// A record's files once a job put `folders` in from `feed`: those folders' files the feed's, every other as it was.
#[must_use]
pub fn merged(files: Option<&BTreeMap<String, String>>, feed: &Feed, folders: &[&str]) -> BTreeMap<String, String> {
	let mut out: BTreeMap<String, String> = files
		.into_iter()
		.flatten()
		.filter(|(path, _)| !folders.contains(&folder_of(path)))
		.map(|(path, sha256)| (path.clone(), sha256.clone()))
		.collect();
	out.extend(
		feed.files.iter().filter(|f| folders.contains(&folder_of(&f.path))).map(|f| (f.path.clone(), f.sha256.clone())),
	);
	out
}

/// The zip's entries (its central directory, in its order) checked against the feed before anything is written: no
/// link, every folder entry one of the folders or under one, every file entry a file the feed lists, once, with
/// the feed's size, and every file the feed lists there.
fn contents<'a>(entries: &'a [Entry], feed: &'a Feed) -> Result<Contents<'a>, AddonError> {
	let wanted: BTreeMap<&str, &FeedFile> = feed.files.iter().map(|f| (f.path.as_str(), f)).collect();
	let (mut folders, mut files) = (Vec::new(), Vec::new());
	let mut seen = BTreeSet::new();
	for entry in entries {
		if entry.is_link() {
			return Err(bad(format!("{}: a link", entry.name)));
		}
		if entry.is_dir() {
			let name = entry.name.trim_end_matches('/');
			if !safe_path(name, true) {
				return Err(bad(format!("{}: a folder outside the addon's", entry.name)));
			}
			folders.push(name);
			continue;
		}
		let want = wanted.get(entry.name.as_str()).ok_or_else(|| bad(format!("{}: not in the feed", entry.name)))?;
		if !seen.insert(entry.name.as_str()) {
			return Err(bad(format!("{}: twice in the zip", entry.name)));
		}
		if entry.size != want.size {
			return Err(bad(format!("{}: {} bytes, the feed says {}", entry.name, entry.size, want.size)));
		}
		files.push((entry, *want));
	}
	if let Some(missing) = wanted.keys().find(|p| !seen.contains(*p)) {
		return Err(bad(format!("{missing}: not in the zip")));
	}
	Ok(Contents { folders, files })
}

/// The zip's folders and each of its files' entry with the feed's, in the zip's order, as `contents` checked them.
struct Contents<'a> {
	folders: Vec<&'a str>,
	files: Vec<(&'a Entry, &'a FeedFile)>,
}

/// One entry written into `into` (its folders made) by `copy`, which writes its bytes and answers their sha256, the
/// feed's `sha256`.
fn write_entry(
	entry: &Entry,
	sha256: &str,
	into: &Path,
	copy: impl FnOnce(&mut BufWriter<File>) -> Result<String, ZipError>,
) -> Result<(), AddonError> {
	let path = into.join(&entry.name);
	if let Some(parent) = path.parent() {
		std::fs::create_dir_all(parent)?;
	}
	let mut out = BufWriter::new(File::create_new(&path)?);
	let got = copy(&mut out)?;
	out.flush()?;
	if got != sha256 {
		return Err(bad(format!("{}: its sha256 is not the feed's", entry.name)));
	}
	Ok(())
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
	let mut archive = Archive::open(zip_path)?;
	let entries = archive.entries().to_vec();
	let Contents { folders, files } = contents(&entries, feed)?;
	for folder in folders {
		std::fs::create_dir_all(into.join(folder))?;
	}
	for (entry, want) in files {
		write_entry(entry, &want.sha256, into, |out| archive.copy(entry, out))?;
	}
	check_version(into, feed)
}

/// The staged TOC's `## Version` is the feed's; nothing to check where QuestPump's own folder is not among those the
/// job puts in (`Scope`: `swap` refuses a folder that is due and not staged).
///
/// # Errors
/// The TOC does not read or names another version.
pub fn check_version(into: &Path, feed: &Feed) -> Result<(), AddonError> {
	if !into.join(TOC).exists() {
		return Ok(());
	}
	let toc = std::fs::read_to_string(into.join(TOC))?;
	let shipped = toc_field(&toc, "Version").unwrap_or_default(); // copy-lint: not shown (a TOC field)
	if !same(shipped, &feed.version) {
		return Err(bad(format!("{TOC} says {shipped:?}, the feed {}", feed.version)));
	}
	Ok(())
}

/// Stages into `into`, made anew, every file of the folders the job puts in (`scope`, as `addons` stands) that this
/// computer holds already, found by its sha256: in the folders in `addons` (`tree`; none where they are or hold a
/// link) and among the desktop app's own copies of the addon's voice takes (`voice`, a folder of files). Each is copied
/// while its sha256 is taken and kept only when its size and sha256 are the feed's, so a file changed since it was
/// read is fetched instead. Answers those folders' files still to fetch, in the feed's order.
///
/// # Errors
/// `into` or a file in it cannot be made or written (a source that does not read is fetched instead).
pub fn take_local(
	feed: &Feed,
	addons: &Path,
	voice: Option<&Path>,
	scope: &Scope,
	into: &Path,
) -> Result<Vec<FeedFile>, AddonError> {
	if into.exists() {
		std::fs::remove_dir_all(into)?;
	}
	std::fs::create_dir_all(into)?;
	let read = tree(addons).unwrap_or(Tree::Linked);
	let folders = scope.folders(feed, &read);
	let mut held: BTreeMap<String, PathBuf> = BTreeMap::new();
	if let Tree::Files(files) = read {
		for (path, sha256) in files {
			held.entry(sha256).or_insert_with(|| addons.join(path));
		}
	}
	for take in voice.and_then(|v| std::fs::read_dir(v).ok()).into_iter().flatten().flatten() {
		let path = take.path();
		if take.file_type().is_ok_and(|t| t.is_file())
			&& let Ok(sha256) = sha256_file(&path)
		{
			held.entry(sha256).or_insert(path);
		}
	}
	let mut wanted = Vec::new();
	for file in feed.files.iter().filter(|f| folders.contains(&folder_of(&f.path))) {
		let taken = match held.get(&file.sha256) {
			Some(from) => copy_checked(from, file, into)?,
			None => false,
		};
		if !taken {
			wanted.push(file.clone());
		}
	}
	Ok(wanted)
}

/// `from` copied into `into` as the feed's `file`, its sha256 taken as it is written; true when its size and sha256 are
/// the feed's, else the copy is removed (as when the source stops reading).
fn copy_checked(from: &Path, file: &FeedFile, into: &Path) -> Result<bool, AddonError> {
	let Ok(mut source) = File::open(from) else { return Ok(false) };
	let path = into.join(&file.path);
	if let Some(parent) = path.parent() {
		std::fs::create_dir_all(parent)?;
	}
	let mut out = BufWriter::new(File::create_new(&path)?);
	let mut hasher = Sha256::new();
	let mut buf = vec![0_u8; 64 << 10];
	let mut got = 0_u64;
	let read = loop {
		match source.read(&mut buf) {
			Ok(0) => break true,
			Ok(n) => {
				got += n as u64;
				if got > file.size {
					break false;
				}
				hasher.update(&buf[..n]);
				out.write_all(&buf[..n])?;
			}
			Err(_) => break false,
		}
	};
	out.flush()?;
	drop(out);
	if read && got == file.size && zip::hex(&hasher.finalize()) == file.sha256 {
		return Ok(true);
	}
	std::fs::remove_file(&path)?;
	Ok(false)
}

/// How far apart two files to fetch may lie in the release zip and still come in one request: the bytes between them
/// cost less than another request's two round trips (github.com, then its storage) on most lines.
const GAP: u64 = 256 << 10;

/// A run of the release zip's bytes fetched in one request, `start..end`, and the files in it, each with its sha256 in
/// the feed.
#[derive(Debug)]
pub struct Span {
	pub start: u64,
	pub end: u64,
	files: Vec<(Entry, String)>,
}

/// The files to fetch out of the release zip by byte ranges: the spans that hold them. The zip's folder entries make
/// nothing here, since neither the feed's signature nor the zip's sha256 covers the central directory they come from;
/// every file's own folders are made as it is written (`write_entry`).
#[derive(Debug)]
pub struct Plan {
	spans: Vec<Span>,
}

impl Plan {
	/// The spans to fetch, in the zip's order.
	#[must_use]
	pub fn spans(&self) -> &[Span] {
		&self.spans
	}

	/// The files to fetch.
	#[must_use]
	pub fn files(&self) -> usize {
		self.spans.iter().map(|s| s.files.len()).sum()
	}

	/// The bytes to fetch.
	#[must_use]
	pub fn bytes(&self) -> u64 {
		self.spans.iter().map(|s| s.end - s.start).sum()
	}
}

/// The spans of the release zip that hold `wanted`, once its central directory (`entries`, starting at `directory`)
/// passed every check `unpack` makes before a write. An entry's bytes run from its local header to the next entry's
/// (or to the central directory) and must hold at least its header and its packed bytes; entries in a row, or no more
/// than `GAP` apart, come in one span.
///
/// # Errors
/// An entry the feed does not allow (as `unpack`'s), or one whose bytes do not fit before the next.
pub fn plan(entries: &[Entry], directory: u64, feed: &Feed, wanted: &[FeedFile]) -> Result<Plan, AddonError> {
	let Contents { files, .. } = contents(entries, feed)?;
	let wanted: BTreeSet<&str> = wanted.iter().map(|f| f.path.as_str()).collect();
	let mut starts: Vec<u64> = entries.iter().map(|e| e.offset).collect();
	starts.sort_unstable();
	starts.dedup();
	let mut picked = Vec::new();
	for (entry, file) in files.into_iter().filter(|(entry, _)| wanted.contains(entry.name.as_str())) {
		let end = starts.get(starts.partition_point(|&at| at <= entry.offset)).copied().unwrap_or(directory);
		let least = entry.offset.checked_add(entry.header_len()).and_then(|at| at.checked_add(entry.compressed));
		if least.is_none_or(|least| least > end) {
			return Err(bad(format!("{}: its bytes run into the next entry's", entry.name)));
		}
		picked.push((entry.offset, end, entry.clone(), file.sha256.clone()));
	}
	picked.sort_by_key(|(start, ..)| *start);
	let mut spans: Vec<Span> = Vec::new();
	for (start, end, entry, sha256) in picked {
		match spans.last_mut() {
			Some(span) if start <= span.end.saturating_add(GAP) => {
				span.end = span.end.max(end);
				span.files.push((entry, sha256));
			}
			_ => spans.push(Span { start, end, files: vec![(entry, sha256)] }),
		}
	}
	Ok(Plan { spans })
}

/// The files of `span` written into `into` out of `part`, which holds the span's bytes from its start, each checked
/// against its sha256 in the feed as it is written.
///
/// # Errors
/// An entry that does not unpack (`zip::copy_at`), one whose sha256 is not the feed's, or a file that cannot be
/// written.
pub fn unpack_span(part: &Path, span: &Span, into: &Path) -> Result<(), AddonError> {
	let mut bytes = BufReader::new(File::open(part)?);
	for (entry, sha256) in &span.files {
		write_entry(entry, sha256, into, |out| zip::copy_at(&mut bytes, entry.offset - span.start, entry, out))?;
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

/// What the addon's folders in `addons` hold (an AddOns folder that is itself a link is `Linked`; one that is not there
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

	/// The record's files of the media packs that `tree` still holds exactly as recorded: what the helper goes on
	/// keeping of an install whose other folders another updater rewrote.
	#[must_use]
	pub fn kept_packs(&self, tree: &Tree) -> BTreeMap<String, String> {
		let Tree::Files(held) = tree else { return BTreeMap::new() };
		let mut kept = BTreeMap::new();
		for pack in PACKS {
			let mine = in_folder(self.files.iter(), pack);
			if !mine.is_empty() && mine == in_folder(held.iter(), pack) {
				kept.extend(mine.into_iter().map(|(path, sha256)| (path.to_owned(), sha256.to_owned())));
			}
		}
		kept
	}
}

/// The staging folder of an AddOns folder: beside it in the flavour's `Interface`.
#[must_use]
pub fn staging(addons: &Path) -> Option<PathBuf> {
	Some(addons.parent()?.join(STAGING))
}

/// Puts `folders` (a `Scope`'s, of `FOLDERS`) from `<staging>/new` in place of the ones in `addons` (made first where a
/// new install has no AddOns folder yet), and no other folder: the old ones moved into `<staging>/old`, the names of
/// the new ones written into `MOVING`, the new ones moved in, QuestPump's own last where it is among them, then `DONE`
/// written (each rename on the same volume, tried again on Windows while a file is held); on any error every folder
/// moved goes back where it was. With no folder to put in it only writes `DONE`.
///
/// # Errors
/// A folder of `folders` is not staged, the step that failed, and whether every old folder went back.
pub fn swap(addons: &Path, staging: &Path, folders: &[&'static str]) -> std::io::Result<()> {
	let (new, old) = (staging.join(NEW), staging.join(OLD));
	// in the other way round, so the name `MOVING` lists last is the one that goes in last (QuestPump's own where it
	// is among them): `recover` reads `MOVING` with that folder no longer in `new` as a finished swap
	let coming: Vec<&'static str> = FOLDERS.into_iter().rev().filter(|name| folders.contains(name)).collect();
	if let Some(name) = coming.iter().find(|name| !new.join(name).is_dir()) {
		return Err(std::io::Error::other(format!("no {name} folder staged")));
	}
	std::fs::create_dir_all(addons)?;
	std::fs::create_dir_all(&old)?;
	let mut out: Vec<&str> = Vec::new();
	let mut moved_in: Vec<&str> = Vec::new();
	let mut step = || -> std::io::Result<()> {
		for name in FOLDERS.into_iter().filter(|name| folders.contains(name)) {
			if std::fs::symlink_metadata(addons.join(name)).is_ok() {
				rename_retried(&addons.join(name), &old.join(name))?;
				out.push(name);
			}
		}
		std::fs::write(staging.join(MOVING), coming.join("\n"))?;
		for name in &coming {
			rename_retried(&new.join(name), &addons.join(name))?;
			moved_in.push(name);
		}
		Ok(())
	};
	let Err(e) = step() else {
		// every folder is in: a marker that does not write leaves `MOVING` with its last folder no longer in `new`,
		// which `recover` reads the same way
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
/// swap (`DONE` there, or `MOVING` with the folder it lists last, the one that goes in last, no longer in `new`: cut
/// between that rename and the marker) only has the folder removed, never an old folder put back; `old` and `new` go before
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
	// the folder that went in last, as `swap` wrote it: one of ours, or the list says nothing
	let last = moving.as_deref().and_then(|coming| coming.lines().last()).filter(|name| FOLDERS.contains(name));
	let done = staging.join(DONE).exists() || last.is_some_and(|name| !new.join(name).exists());
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
