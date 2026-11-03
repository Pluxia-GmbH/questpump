//! Finding the game's `Interface/AddOns` folder while the game is not running (docs/companion.md, "Finding the
//! install"; rust-helper `windows-capture-and-install.md`, "Install discovery"): the Battle.net Agent's `product.db`
//! names every World of Warcraft install and its flavour folder; then the default install folders' `_*_` folders; the
//! first that holds QuestPump is the one (`discover`), and while none does, the first of them at all is the game on
//! this computer without QuestPump yet (`discover_game`). The running client's path (`game.rs`) when it holds QuestPump
//! and the folder remembered or picked before while it holds QuestPump come first (`runtime.rs`, `Helper::addons`).
//! The flavour is never hard-coded: the Forever launch's folder is unknown until it ships (the beta is
//! `wow_classic_beta` in `_classic_beta_`, read from a real install on 2026-09-30). Also what the window says about the
//! folder: its product and version, and the move figures' pack; and, for the games on the computer, the folder the
//! Agent names for StarCraft II or Hearthstone and, on Windows, for every other game it installed (slice 48).

use std::path::{Path, PathBuf};

use prost::Message as _;
use questpump_core::mirror::toc_field;

use crate::game::is_flavour;

/// The move figures' load-on-demand folder beside QuestPump (`Figure.lua`, `Figure.pack`).
const PACK: &str = "QuestPump_FigArmor";

/// The Battle.net Agent's database (reverse-engineered `proto_database.Database`, as WowUp and TACTLib read it): only
/// the fields read here are declared, and prost skips the others.
#[derive(Clone, PartialEq, prost::Message)]
struct Database {
	#[prost(message, repeated, tag = "1")]
	products: Vec<ProductInstall>,
}

#[derive(Clone, PartialEq, prost::Message)]
struct ProductInstall {
	/// `wow_classic_beta` for the Forever beta.
	#[prost(string, tag = "1")]
	uid: String,
	#[prost(message, optional, tag = "3")]
	settings: Option<UserSettings>,
	/// `wow` for every World of Warcraft product; empty for the Agent and the Battle.net app.
	#[prost(string, tag = "6")]
	family: String,
}

#[derive(Clone, PartialEq, prost::Message)]
struct UserSettings {
	/// The install's root, such as `/Applications/World of Warcraft`.
	#[prost(string, tag = "1")]
	install_path: String,
	/// The flavour folder, such as `_classic_beta_`.
	#[prost(string, tag = "13")]
	game_subfolder: String,
}

/// The AddOns folders of the World of Warcraft products the Agent's database names, in its order.
fn from_agent() -> Vec<PathBuf> {
	battlenet_products()
		.into_iter()
		.filter(|p| p.family == "wow")
		.filter_map(|p| p.settings)
		.filter(|s| !s.game_subfolder.is_empty() && on_this_computer(Path::new(&s.install_path)))
		.map(|s| PathBuf::from(s.install_path).join(s.game_subfolder).join("Interface").join("AddOns"))
		.collect()
}

/// The Battle.net Agent's own product uids and the Battle.net app's, which no game is.
#[cfg(windows)]
const AGENT_UIDS: [&str; 2] = ["agent", "battle.net"];

/// The products the Battle.net Agent's database names; none when it cannot be read or does not decode (the Agent
/// rewriting it, another schema).
fn battlenet_products() -> Vec<ProductInstall> {
	let Some(db) = agent_database().and_then(|p| std::fs::read(p).ok()) else { return Vec::new() };
	Database::decode(db.as_slice()).map(|db| db.products).unwrap_or_default()
}

/// A product's install folder, when it is there on this computer's own disks (`on_this_computer`).
fn install_dir(product: &ProductInstall) -> Option<PathBuf> {
	let dir = PathBuf::from(&product.settings.as_ref()?.install_path);
	(on_this_computer(&dir) && dir.is_dir()).then_some(dir)
}

/// The install folder the Battle.net Agent's database names for another of Blizzard's games by its product uid (`s2`
/// StarCraft II, `hs_beta` Hearthstone: games/mod.rs, slice 48), when it is there.
pub fn battlenet_install(uid: &str) -> Option<PathBuf> {
	battlenet_products().iter().filter(|p| p.uid == uid).find_map(install_dir)
}

/// The install folders of every game the Battle.net Agent's database names, World of Warcraft's, the Agent's own and
/// the Battle.net app's left out (the games on the computer's detector, games/detect.rs, slice 48).
#[cfg(windows)]
pub fn battlenet_games() -> Vec<PathBuf> {
	battlenet_products()
		.iter()
		.filter(|p| !p.family.is_empty() && p.family != "wow" && !AGENT_UIDS.contains(&p.uid.as_str()))
		.filter_map(install_dir)
		.collect()
}

/// Where the Battle.net Agent keeps its database: the system's `ProgramData` on Windows (where the Agent lives), the
/// shared folder on macOS.
fn agent_database() -> Option<PathBuf> {
	if cfg!(windows) {
		std::env::var_os("ProgramData").map(|d| PathBuf::from(d).join("Battle.net").join("Agent").join("product.db"))
	} else {
		Some(PathBuf::from("/Users/Shared/Battle.net/Agent/product.db"))
	}
}

/// `GetDriveTypeW`'s answer for a network drive (`DRIVE_REMOTE`, `WinBase.h`; the windows crate keeps it in a feature
/// the helper needs for nothing else).
#[cfg(windows)]
const DRIVE_REMOTE: u32 = 4;

/// Whether `path` is on this computer's own disks, which a path from a record other accounts of the computer may write
/// (the Battle.net Agent's `product.db`, Steam's `libraryfolders.vdf`, the Epic and Riot records in `ProgramData`) must
/// be before the helper reads, writes or grants on it: on Windows a full path on a drive letter that is no network
/// drive, never a UNC path (`\\server\share`, `\\?\UNC\…`), a device path or a drive-relative one, so no read sends
/// the player's sign-in to another computer and nothing is written or granted there; elsewhere any full path (macOS
/// reaches a network volume only once the player mounted it).
pub fn on_this_computer(path: &Path) -> bool {
	#[cfg(windows)]
	{
		use std::path::{Component, Prefix};
		let Some(Component::Prefix(prefix)) = path.components().next() else { return false };
		let (Prefix::Disk(letter) | Prefix::VerbatimDisk(letter)) = prefix.kind() else { return false };
		if !path.has_root() {
			return false;
		}
		let root = windows::core::HSTRING::from(format!("{}:\\", char::from(letter)));
		// SAFETY: a NUL-ended root path, which the call only reads.
		let kind = unsafe { windows::Win32::Storage::FileSystem::GetDriveTypeW(windows::core::PCWSTR(root.as_ptr())) };
		kind != DRIVE_REMOTE
	}
	#[cfg(not(windows))]
	{
		path.is_absolute()
	}
}

/// The default install folders, for a game the Agent does not know (a copied install).
fn default_roots() -> Vec<PathBuf> {
	if cfg!(windows) {
		vec![
			PathBuf::from(r"C:\Program Files (x86)\World of Warcraft"),
			PathBuf::from(r"C:\Program Files\World of Warcraft"),
		]
	} else {
		vec![PathBuf::from("/Applications/World of Warcraft")]
	}
}

/// The AddOns folders of a root's flavour folders, in name order, whether they exist yet or not (a new install has
/// none until an addon is put there).
fn flavours(root: &Path) -> Vec<PathBuf> {
	let mut found: Vec<PathBuf> = std::fs::read_dir(root)
		.into_iter()
		.flatten()
		.flatten()
		.filter(|e| is_flavour(&e.file_name().to_string_lossy()) && e.path().is_dir())
		.map(|e| e.path().join("Interface").join("AddOns"))
		.collect();
	found.sort();
	found
}

/// Whether an AddOns folder holds QuestPump: only such a folder gets the mirror.
pub fn holds_questpump(addons: &Path) -> bool {
	addons.join("QuestPump").is_dir()
}

/// Whether an AddOns folder belongs to a game that is there: its flavour folder is (the AddOns folder itself may not
/// be yet: a new install has none until an addon is put there).
pub fn is_game(addons: &Path) -> bool {
	addons.parent().and_then(Path::parent).is_some_and(Path::is_dir)
}

/// The AddOns folders of the Agent's installs, then of the roots' flavour folders, in that order.
fn installs_in(roots: &[PathBuf]) -> Vec<PathBuf> {
	from_agent().into_iter().chain(roots.iter().flat_map(|r| flavours(r))).collect()
}

/// The game folder the mirror goes into when the game is not running and none that holds QuestPump was found or picked
/// before: the first install the Agent or the default folders know that holds QuestPump.
pub fn discover() -> Option<PathBuf> {
	installs_in(&default_roots()).into_iter().find(|a| holds_questpump(a))
}

/// The game on this computer while no install holds QuestPump yet: the AddOns folder of the first install the Agent or
/// the default folders know whose flavour folder is there, with or without QuestPump (and with or without an AddOns
/// folder, which a new install lacks until an addon is put there).
pub fn discover_game() -> Option<PathBuf> {
	installs_in(&default_roots()).into_iter().find(|a| is_game(a))
}

/// Whether a folder is an `Interface/AddOns` folder (Windows spells folders in any case).
fn is_addons(path: &Path) -> bool {
	let name = |p: &Path| p.file_name().map(|n| n.to_string_lossy().to_ascii_lowercase());
	name(path).as_deref() == Some("addons") && path.parent().and_then(name).as_deref() == Some("interface")
}

/// The AddOns folder of a folder the player picked: the AddOns folder itself, a flavour folder or anything inside one
/// (the game's app, its `Interface` folder), or the install's root (its flavour folder that holds QuestPump, else its
/// first).
pub fn from_picked(path: &Path) -> Option<PathBuf> {
	for dir in path.ancestors().take(4) {
		if is_addons(dir) && dir.is_dir() {
			return Some(dir.to_path_buf());
		}
		let addons = dir.join("Interface").join("AddOns");
		if addons.is_dir() {
			return Some(addons);
		}
		let inside: Vec<PathBuf> = flavours(dir).into_iter().filter(|a| a.is_dir()).collect();
		if let Some(first) = inside.first() {
			return inside.iter().find(|a| holds_questpump(a)).or(Some(first)).cloned();
		}
	}
	None
}

/// The move figures' pack beside QuestPump.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pack {
	/// Not installed: a manual install that left the second folder out.
	Missing,
	/// From the same download as QuestPump (both TOCs carry the release's version).
	Same,
	/// From another download than QuestPump: one folder was updated without the other.
	Other { pack: String, addon: String },
}

/// What an AddOns folder holds for the window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddonsState {
	/// QuestPump's `## Version`, when it is installed (empty when its TOC names none).
	pub questpump: Option<String>,
	pub pack: Pack,
}

/// A folder's TOC `## Version`, when the folder is there (empty when the TOC cannot be read or names none).
fn version(addons: &Path, folder: &str) -> Option<String> {
	let dir = addons.join(folder);
	if !dir.is_dir() {
		return None;
	}
	let text = std::fs::read_to_string(dir.join(format!("{folder}.toc"))).unwrap_or_default();
	Some(toc_field(&text, "Version").unwrap_or_default().to_owned()) // copy-lint: not shown (a TOC field)
}

/// QuestPump's version and the pack's state, from the two folders' TOCs (the packager writes the same release version
/// into both, so a difference means one folder came from another download).
pub fn addons_state(addons: &Path) -> AddonsState {
	let questpump = version(addons, "QuestPump"); // copy-lint: not shown (the addon's folder)
	let pack = match (version(addons, PACK), &questpump) {
		(None, _) => Pack::Missing,
		(Some(pack), Some(addon)) if pack != *addon => Pack::Other { pack, addon: addon.clone() },
		(Some(_), _) => Pack::Same,
	};
	AddonsState { questpump, pack }
}

/// The product a flavour folder holds (`.flavor.info`: a header line, then the product's uid).
fn flavour_uid(text: &str) -> Option<&str> {
	let mut lines = text.lines();
	lines.next().filter(|h| h.starts_with("Product Flavor!"))?;
	lines.next().map(str::trim).filter(|s| !s.is_empty())
}

/// A product's version in the root's `.build.info` (Blizzard's pipe-separated values: a header of `Name!TYPE:size`
/// cells, then a row per product).
fn build_version<'a>(text: &'a str, uid: &str) -> Option<&'a str> {
	let mut lines = text.lines().filter(|l| !l.is_empty());
	let header: Vec<&str> = lines.next()?.split('|').map(|c| c.split_once('!').map_or(c, |(name, _)| name)).collect();
	let column = |name: &str| header.iter().position(|h| *h == name);
	let (product, version) = (column("Product")?, column("Version")?); // copy-lint: not shown (product.db's columns)
	lines.map(|l| l.split('|').collect::<Vec<_>>()).find(|row| row.get(product) == Some(&uid))?.get(version).copied()
}

/// The product an AddOns folder's install holds and its version in the root's `.build.info` (none where the root names
/// none for it).
fn product(addons: &Path) -> Option<(String, Option<String>)> {
	let flavour = addons.parent()?.parent()?;
	let uid = std::fs::read_to_string(flavour.join(".flavor.info")).ok()?;
	let uid = flavour_uid(&uid)?;
	let build = std::fs::read_to_string(flavour.parent()?.join(".build.info")).unwrap_or_default();
	Some((uid.to_owned(), build_version(&build, uid).map(str::to_owned)))
}

/// The window's name for the install an AddOns folder belongs to, such as `wow_classic_beta 1.60.1.70058`.
pub fn label(addons: &Path) -> Option<String> {
	let (uid, version) = product(addons)?;
	Some(version.map_or_else(|| uid.clone(), |v| format!("{uid} {v}")))
}

/// The interface number an install's client runs, from its version in `.build.info` as a TOC's `## Interface` numbers
/// it (`1.60.1.70058` is 16001: major × 10000 + minor × 100 + patch), so Forever's folder is told apart by the
/// number the addon names first, never by a folder or product name.
pub fn client_interface(addons: &Path) -> Option<u32> {
	let (_, version) = product(addons)?;
	let mut parts = version?.split('.').map(str::parse::<u32>).collect::<Vec<_>>().into_iter();
	let (major, minor, patch) = (parts.next()?.ok()?, parts.next()?.ok()?, parts.next()?.ok()?);
	major.checked_mul(10_000)?.checked_add(minor.checked_mul(100)?)?.checked_add(patch)
}

/// Every install the Agent or the default folders know whose flavour folder is there, with QuestPump or without (a
/// first install picks among them: `runtime/addon.rs`).
pub fn games() -> Vec<PathBuf> {
	let mut seen = Vec::new();
	for addons in installs_in(&default_roots()).into_iter().filter(|a| is_game(a)) {
		if !seen.contains(&addons) {
			seen.push(addons);
		}
	}
	seen
}
