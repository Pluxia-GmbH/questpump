//! The move figures in the desktop app (docs/ui.md, "The pump on the computer": the figures; docs/plan.md slice 35,
//! "The figures' weight"): the phones' table, `contract/figures/figures.json` (version 6, built in), as the `figures`
//! topic the page draws from (`helper/dist/lib/figure.js`): each library move's owner, its key pose's second, whether
//! the pack mirrors it and its fault's caption key; each owner's box, cut floor edges and tile square; the films'
//! layout, card and crimson; the faults' words (`contract/moves.json`, the built-in facts': none while the file does
//! not read, and the fault pairs show without a caption); and where the files are, as the asset protocol's URL prefixes
//! (Tauri's `convertFileSrc` spelling: a file's URL is the prefix and its name through `encodeURIComponent`). The
//! stills and the fault pairs ship inside the app (`resources/figures/stills/`, `pairs/`), so the safety pairs are
//! always there. The films (H.264, `films/<owner>.mp4`, which Windows' web view plays without an extension) ship in the
//! test builds (`resources/figures/films/`) until the release feed opens (the public release, `docs/tasks.md`); a
//! bundle without them fetches `figures-<films_digest>.zip` once from the updater's feed (its `helper-updater`
//! release), checks every entry's sha256 against the table, unpacks it beside and moves it into the data folder's
//! `figures/<films_digest>/` in one rename, so a partial set is never read, and tries again when it fails, sooner at
//! first (`RETRY`: a start at login before the network is up, a laptop still joining Wi-Fi); a move shows its still
//! until its film is there. This computer's "Animations on this computer" line follows it (`menu.computer`'s
//! `animations`).
//!
//! Each owner's film is one loop of the move at its real tempo, from its start pose back to it (a timing variant has a
//! film of its own), and the table names its length (`loop`, seconds): a move's beat (`beat`), which the pump's rep
//! count runs on and Big view's figure is phased to (`desk/pump.rs`), with the second each rep but the last ends where
//! a loop shows more than one (`counts`, a dead bug's two sides: `rep_ends`); both stay out of the topic, since the
//! pump publishes a running move's own. A gear variant (`<move>.<gear>`, 2026-10-09) inherits its move's timeline,
//! so the move's beat and rep ends are its own too.
//!
//! The day reel (slice 45, `desk::reel`) reads the same films from Rust: a move's film, its key pose, its mirror and
//! its tile (`film`), only where the whole set is here (`filmed`), so the Saga's reel and its maker agree on which
//! moves a reel can show; the films' landing after a download has the Saga built again (`DeskAsk::Films`).
//!
//! The topic: `{"card", "crimson", "film": {"side", "tint"}, "moves": {<move>: {"owner", "key", "mirror", "fault"?,
//! "gear"?: {<gear key>: {"owner", "key", "mirror", "fault"?}}}},
//! "owners": {<owner>: {"box": [l, t, r, b], "fade": [l|null, r|null, b|null], "tile": [x, y, side]}},
//! "extras": {<extra>: {"box", "fade", "tile"}}, "faults": {<caption key>: words}, "stills": prefix|null,
//! "pairs": prefix|null, "films": prefix|null, "scenes": {"files": prefix|null, "scenes": {<scene>: {"file", "stand":
//! [x, y], "height"}}, "places": {<template>: <scene>}, "streak", "fallback"}|null}`: the extras are the figures that
//! are no library move (the streak card's standard-bearer, `streak_banner`), each a still and its tint in the stills'
//! folder with no film. The scenes are the share cards' (docs/plan.md slice 51, "The share cards tell one true
//! thing"; `contract/design/scenes/scenes.json`, built in), whose pages ship inside the app (`resources/scenes/`) and
//! which the page draws the timed pump card and the streak card on.

mod unzip;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};
use std::time::Duration;

use anyhow::Context as _;
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager as _};
use tokio::io::AsyncWriteExt as _;

use super::facts::{Facts, facts};
use super::{Desk, DeskAsk};
use crate::runtime::Helper;
use crate::ui::{self, Animations, Shared, WindowAsk};

/// The page's topic.
const TOPIC: &str = "figures";

/// How long a failed download waits before each next try, on the wall clock (`feeds::sleep`): 1, 5 and 30 minutes,
/// then every 6 hours, so a start before the network is up gets its films within minutes and a feed that stays away is
/// not asked all day. The download has no total bound, only the feeds' `READ_TIMEOUT` and `CONNECT_TIMEOUT`, so a slow
/// line still brings it.
const RETRY: [Duration; 4] =
	[Duration::from_mins(1), Duration::from_mins(5), Duration::from_mins(30), Duration::from_hours(6)];
/// The most a download may bring (the films are about 60 MB): a feed that sends more is refused.
const MAX_BYTES: u64 = 512 << 20;
/// What `encodeURIComponent` leaves as it is.
pub(crate) const URI_COMPONENT: &AsciiSet = &NON_ALPHANUMERIC
	.remove(b'-')
	.remove(b'_')
	.remove(b'.')
	.remove(b'!')
	.remove(b'~')
	.remove(b'*')
	.remove(b'\'')
	.remove(b'(')
	.remove(b')');

/// figures.json, the part the desktop app reads.
#[derive(Deserialize)]
struct Table {
	card: String,
	crimson: String,
	film: Film,
	films_digest: String,
	/// Each file's sha256 by its path in contract/figures/.
	files: BTreeMap<String, String>,
	moves: BTreeMap<String, Move>,
	owners: BTreeMap<String, Owner>,
	/// The figures that are no library move, by name: a still each, no film (`streak_banner`).
	#[serde(default)]
	extras: BTreeMap<String, Extra>,
}

/// An extra figure's numbers: an owner's, read off its still.
#[derive(Deserialize, Serialize)]
struct Extra {
	r#box: [f64; 4],
	fade: [Option<f64>; 3],
	tile: [f64; 3],
}

/// The films' layout: the square's side and the tint's first row.
#[derive(Deserialize, Serialize)]
struct Film {
	side: u32,
	tint: u32,
}

#[derive(Deserialize, Serialize)]
struct Move {
	owner: String,
	key: f64,
	mirror: bool,
	#[serde(skip_serializing_if = "Option::is_none")]
	fault: Option<String>,
	/// The move drawn with the implement each equipment key names (tools/figures' gear variants, 2026-10-09), which
	/// the pump shows a player whose load that implement carries (`lib/figure.js` `figureName`).
	#[serde(default, skip_serializing_if = "Option::is_none")]
	gear: Option<BTreeMap<String, Move>>,
	/// Where its loop shows more than one rep (a dead bug's two sides), the second each rep but the last ends.
	#[serde(default, skip_serializing)]
	counts: Vec<f64>,
}

/// An owner's numbers, shares of its square's side, unmirrored, and its film's length.
#[derive(Deserialize, Serialize)]
struct Owner {
	r#box: [f64; 4],
	fade: [Option<f64>; 3],
	tile: [f64; 3],
	/// The film's length in seconds, one loop at real tempo (`beat`); none in a table written before it.
	#[serde(default, rename = "loop", skip_serializing)]
	film_s: Option<f64>,
}

/// The table, built in; one that does not read leaves the app without figures (logged at the start).
static TABLE: LazyLock<Option<Table>> =
	LazyLock::new(|| serde_json::from_str(include_str!("../../../../contract/figures/figures.json")).ok());

/// The share cards' scenes (`contract/design/scenes/scenes.json`): each scene's page and where the standard-bearer
/// stands in it, the scene of each place template, the streak card's and a place's without a template.
#[derive(Deserialize, Serialize)]
struct SceneTable {
	scenes: BTreeMap<String, Scene>,
	places: BTreeMap<String, String>,
	streak: String,
	fallback: String,
}

/// A scene's page, the standard-bearer's feet as shares of its width and height, and its height as a share of the
/// page's.
#[derive(Deserialize, Serialize)]
struct Scene {
	file: String,
	stand: [f64; 2],
	height: f64,
}

/// The scenes as the topic gives them: the table and the asset protocol's prefix of the pages' folder.
#[derive(Serialize)]
struct ScenesView<'a> {
	files: Option<String>,
	#[serde(flatten)]
	table: &'a SceneTable,
}

/// The scenes' table, built in; one that does not read leaves the share cards on their plain ground (logged at the
/// start).
static SCENES: LazyLock<Option<SceneTable>> =
	LazyLock::new(|| serde_json::from_str(include_str!("../../../../contract/design/scenes/scenes.json")).ok());

/// The films' folder while every owner's film is in it (the bundle's or the download's), none before.
static FILMS: Mutex<Option<PathBuf>> = Mutex::new(None);

#[derive(Serialize)]
struct Topic<'a> {
	card: &'a str,
	crimson: &'a str,
	film: &'a Film,
	moves: &'a BTreeMap<String, Move>,
	owners: &'a BTreeMap<String, Owner>,
	extras: &'a BTreeMap<String, Extra>,
	faults: &'a BTreeMap<String, String>,
	stills: Option<String>,
	pairs: Option<String>,
	films: Option<String>,
	scenes: Option<ScenesView<'a>>,
}

impl Helper {
	/// The start's part of the figures: the topic with the bundled stills and pairs and the films where they are, and
	/// the download when they are nowhere.
	pub(crate) fn figures_start(&self) {
		let Some(table) = TABLE.as_ref() else {
			log::error!("figures: the built-in figures.json does not read: no figure shows");
			return;
		};
		if SCENES.is_none() {
			log::error!("figures: the built-in scenes.json does not read: the share cards draw no scene");
		}
		let bundled = self.app.path().resource_dir().ok().map(|dir| dir.join("resources").join("figures"));
		let kept = self.dir.join("figures");
		let in_app = bundled.as_ref().map(|b| b.join("films")).filter(|films| complete(films, table));
		let downloaded = Some(kept.join(&table.films_digest).join("films")).filter(|films| complete(films, table));
		let (films, animations) = match (in_app, downloaded) {
			(Some(films), _) => (Some(films), Animations::Bundled),
			(None, Some(films)) => (Some(films), Animations::Kept),
			(None, None) => (None, Animations::Downloading { percent: 0 }),
		};
		publish(&self.app, table, bundled.as_deref(), films.as_deref());
		let fetch = films.is_none();
		ui::update(&self.app, |v| v.computer.animations = animations);
		if !fetch {
			return;
		}
		let Some(url) = crate::update::feed_file(&self.app, &format!("figures-{}.zip", table.films_digest)) else {
			log::warn!("figures: the films are not in this build and it has no feed to fetch them from");
			ui::update(&self.app, |v| v.computer.animations = Animations::Missing);
			return;
		};
		tauri::async_runtime::spawn(fetch_films(self.app.clone(), kept, url, bundled));
	}
}

/// A move's beat in seconds by its key: its owner's film's length, one loop at its real tempo; none for a move without
/// a figure, an owner whose length the table does not name, or a table that does not read.
pub(crate) fn beat(key: &str) -> Option<f64> {
	let table = TABLE.as_ref()?;
	table.owners.get(&table.moves.get(key)?.owner)?.film_s
}

/// The seconds in a move's loop at which each of its reps but the last ends (a dead bug's two sides), by its key; none
/// for a loop of one rep, a move without a figure or a table that does not read.
pub(crate) fn rep_ends(key: &str) -> Vec<f64> {
	TABLE.as_ref().and_then(|t| t.moves.get(key)).map(|m| m.counts.clone()).unwrap_or_default()
}

/// A move's film as the day reel reads it (`desk::reel`): the file, the key pose's second, whether the pack mirrors it,
/// its owner's tile square (shares of the film's square: x, y, side) and length in seconds; the films' layout (the
/// square's side and the tint's first row) and the crimson the films are tinted in.
pub(crate) struct MoveFilm {
	pub(crate) path: PathBuf,
	pub(crate) key_s: f64,
	pub(crate) mirror: bool,
	pub(crate) tile: [f64; 3],
	pub(crate) loop_s: f64,
	pub(crate) side: u32,
	pub(crate) tint: u32,
	pub(crate) crimson: [u8; 3],
}

/// A library move's film by its id (its key in `contract/moves.json`), where the whole set of films is on this
/// computer; none for a move without a figure, before the films came, or with a table that does not read.
pub(crate) fn film(f: &Facts, mv: u16) -> Option<MoveFilm> {
	let table = TABLE.as_ref()?;
	let folder = FILMS.lock().ok()?.clone()?;
	let entry = table.moves.get(&f.text(mv)?.key)?;
	let owner = table.owners.get(&entry.owner)?;
	Some(MoveFilm {
		path: folder.join(format!("{}.mp4", entry.owner)),
		key_s: entry.key,
		mirror: entry.mirror,
		tile: owner.tile,
		loop_s: owner.film_s?,
		side: table.film.side,
		tint: table.film.tint,
		crimson: hex(&table.crimson)?,
	})
}

/// Whether a move's film is on this computer (`film`): the Saga's reel leaves out the moves it is not.
pub(crate) fn filmed(f: &Facts, mv: u16) -> bool {
	film(f, mv).is_some()
}

/// "#RRGGBB" as its three bytes.
fn hex(colour: &str) -> Option<[u8; 3]> {
	let digits = colour.strip_prefix('#').filter(|d| d.len() == 6)?;
	let byte = |i: usize| u8::from_str_radix(digits.get(i..i + 2)?, 16).ok();
	Some([byte(0)?, byte(2)?, byte(4)?])
}

/// Publishes the topic: the stills, the pairs and the scenes from the bundle, the films from `films`, which the day
/// reel reads too.
fn publish(app: &AppHandle, table: &Table, bundled: Option<&Path>, films: Option<&Path>) {
	if let Ok(mut kept) = FILMS.lock() {
		*kept = films.map(Path::to_path_buf);
	}
	let topic = Topic {
		card: &table.card,
		crimson: &table.crimson,
		film: &table.film,
		moves: &table.moves,
		owners: &table.owners,
		extras: &table.extras,
		faults: &facts().moves.faults,
		stills: bundled.and_then(|b| asset_prefix(&b.join("stills"))),
		pairs: bundled.and_then(|b| asset_prefix(&b.join("pairs"))),
		films: films.and_then(asset_prefix),
		scenes: SCENES.as_ref().map(|scenes| ScenesView {
			files: bundled.and_then(Path::parent).and_then(|resources| asset_prefix(&resources.join("scenes"))),
			table: scenes,
		}),
	};
	app.state::<Desk>().publisher.publish(TOPIC, &topic);
}

/// Every owner's film is in `folder`.
fn complete(folder: &Path, table: &Table) -> bool {
	folder.is_dir() && table.owners.keys().all(|owner| folder.join(format!("{owner}.mp4")).is_file())
}

/// The asset protocol's URL prefix of a folder's files, as Tauri's `convertFileSrc` spells a file's URL (the whole path
/// through `encodeURIComponent`; `asset://localhost/` on macOS, `http://asset.localhost/` on Windows); none for a path
/// that is not UTF-8.
pub(crate) fn asset_prefix(folder: &Path) -> Option<String> {
	let path = folder.to_str()?;
	let base = if cfg!(windows) { "http://asset.localhost/" } else { "asset://localhost/" };
	let mut with_separator = path.to_owned();
	with_separator.push(std::path::MAIN_SEPARATOR);
	Some(format!("{base}{}", utf8_percent_encode(&with_separator, URI_COMPONENT)))
}

/// Fetches the films until they are kept, then shows them: one try now, the next after each failure as `RETRY` spaces
/// them.
async fn fetch_films(app: AppHandle, kept: PathBuf, url: String, bundled: Option<PathBuf>) {
	let Some(table) = TABLE.as_ref() else { return };
	for failed in 0_usize.. {
		match download(&app, &kept, &url, table).await {
			Ok(films) => {
				log::info!("figures: the films {} are kept", table.films_digest);
				publish(&app, table, bundled.as_deref(), Some(&films));
				ui::update(&app, |v| v.computer.animations = Animations::Kept);
				// the Saga's reels can show these moves now
				app.state::<Shared>().ask(WindowAsk::Desk(DeskAsk::Films));
				prune(&kept, &table.films_digest);
				return;
			}
			Err(e) => {
				log::warn!("figures: the films did not download: {e:#}");
				let error = format!("{e:#}");
				ui::update(&app, |v| v.computer.animations = Animations::Failed { error });
			}
		}
		crate::feeds::sleep(RETRY[failed.min(RETRY.len() - 1)]).await;
	}
}

/// Downloads the zip beside the kept folder, unpacks it into a folder of its own with every entry checked, and moves
/// that into `<kept>/<films_digest>` whole; answers the films' folder.
async fn download(app: &AppHandle, kept: &Path, url: &str, table: &'static Table) -> anyhow::Result<PathBuf> {
	let digest = table.films_digest.clone();
	tokio::fs::create_dir_all(kept).await.with_context(|| kept.display().to_string())?;
	let zip = kept.join(format!(".figures-{digest}.zip.part"));
	let client = reqwest::Client::builder()
		.user_agent(crate::feedback::USER_AGENT)
		.tls_backend_native()
		.read_timeout(crate::feeds::READ_TIMEOUT)
		.connect_timeout(crate::feeds::CONNECT_TIMEOUT)
		.build()?;
	let mut response = client.get(url).send().await?.error_for_status()?;
	let total = response.content_length();
	anyhow::ensure!(total.is_none_or(|t| t <= MAX_BYTES), "the feed offers {total:?} bytes");
	let mut file = tokio::fs::File::create(&zip).await.with_context(|| zip.display().to_string())?;
	let (mut got, mut shown) = (0_u64, 0_u8);
	while let Some(chunk) = response.chunk().await? {
		got += chunk.len() as u64;
		anyhow::ensure!(got <= MAX_BYTES, "the feed sends more than {MAX_BYTES} bytes");
		file.write_all(&chunk).await?;
		let percent = total.filter(|t| *t > 0).map_or(0, |t| u8::try_from(got * 100 / t).unwrap_or(100).min(99));
		if percent != shown {
			shown = percent;
			ui::update(app, |v| v.computer.animations = Animations::Downloading { percent });
		}
	}
	file.flush().await?;
	drop(file);
	let part = kept.join(format!(".{digest}.part"));
	let into = part.clone();
	let from = zip.clone();
	let unpacked = tauri::async_runtime::spawn_blocking(move || unzip::unpack(&from, &into, &table.files)).await?;
	let _ = tokio::fs::remove_file(&zip).await;
	if let Err(e) = unpacked {
		let _ = tokio::fs::remove_dir_all(&part).await;
		return Err(e);
	}
	let folder = kept.join(&digest);
	// a folder of this set that lost a film (the start found it short) gives way to the whole one
	if tokio::fs::try_exists(&folder).await? {
		tokio::fs::remove_dir_all(&folder).await.with_context(|| folder.display().to_string())?;
	}
	tokio::fs::rename(&part, &folder).await.with_context(|| folder.display().to_string())?;
	Ok(folder.join("films"))
}

/// Removes what an older film set or a broken try left in the kept folder.
fn prune(kept: &Path, digest: &str) {
	let Ok(entries) = std::fs::read_dir(kept) else { return };
	for entry in entries.flatten() {
		let name = entry.file_name();
		if name.to_str() == Some(digest) {
			continue;
		}
		let path = entry.path();
		let removed = if path.is_dir() { std::fs::remove_dir_all(&path) } else { std::fs::remove_file(&path) };
		if let Err(e) = removed {
			log::warn!("figures: {}: {e}", path.display());
		}
	}
}
