//! The addon from QuestPump on this computer (docs/companion.md, "The addon on this computer"; questpump-core `addon`
//! holds the feed's rules, the zip's checks, the record and the swap): the release feed fetched off the loop on the
//! updater's schedule (a minute after the start, then every 6 hours, `FIRST` and `EVERY`) and at once when the loop
//! asks (`Checks::now`: a game without QuestPump found, the player's Add QuestPump to the game before a feed came),
//! `addon.json` and `addon.json.sig` from the updater's own `helper-updater` release checked against the updater's key
//! (`update::pubkey`), the zip's address the same release's `QuestPump-addon.zip`; then, for the install the loop
//! picked, the zip downloaded into the flavour's `Interface/.questpump-update/` (the same volume as AddOns, outside the
//! folder the client walks), checked against the feed's size and sha256 as it comes, and unpacked there with every
//! file checked (`addon::unpack`). The swap and the cleaning up run off the loop too (`swap`, `settle`), each
//! reporting back; the loop decides when (`runtime/addon.rs`).

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use anyhow::Context as _;
use questpump_core::addon::{self, AddonError, Feed};
use questpump_core::zip::ZipError;
use sha2::{Digest as _, Sha256};
use tauri::AppHandle;
use tokio::io::AsyncWriteExt as _;
use tokio::sync::{Notify, mpsc};

use crate::update;

/// The first check after the start, and the next ones: the helper's own updater's schedule.
const FIRST: Duration = Duration::from_mins(1);
const EVERY: Duration = Duration::from_hours(6);
/// The longest `addon.json` or its signature may be, and the longest the zip may take to come.
const MAX_FEED: u64 = 16 << 20;
const TIMEOUT: Duration = Duration::from_mins(30);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);
/// The files on the release, beside the updater's `latest.json`; the zip keeps its name in the staging folder.
const FEED: &str = "addon.json";
const SIG: &str = "addon.json.sig";
const ZIP: &str = "QuestPump-addon.zip";

/// What the addon's tasks report to the loop.
pub enum News {
	/// A check of the feed: the verified feed, or why none.
	Feed(Result<Feed, String>),
	/// What an install's folders hold (`addon::tree`), for its ownership, read in the loop's `epoch` (a job that began
	/// or ended since may have changed them).
	Tree { addons: PathBuf, epoch: u64, tree: Result<addon::Tree, String> },
	/// The zip staged for `addons` (its staging folder), none for a first install that a swap cut short had left
	/// without QuestPump, which is back, or why not.
	Staged { addons: PathBuf, result: Result<Option<PathBuf>, Failure> },
	/// The swap ran: the folders are the new ones, or the old ones are back (and why).
	Swapped { addons: PathBuf, result: Result<(), Failure> },
	/// The swap did not start: the game or a pump started after the loop let it go (nothing was changed).
	Held { addons: PathBuf },
	/// The staging folder of `addons` settled (`addon::recover`): the folders put back, or why not.
	Settled { addons: PathBuf, result: Result<Vec<&'static str>, String> },
}

/// Why a step failed, with the system's error code where it came from one (Windows' refusals: `runtime/folder.rs`).
#[derive(Debug, Clone)]
pub struct Failure {
	pub error: String,
	pub os: Option<i32>,
}

impl Failure {
	fn of(e: &anyhow::Error) -> Self {
		let os = e.chain().find_map(|c| {
			let io = c.downcast_ref::<std::io::Error>().or_else(|| match c.downcast_ref::<AddonError>() {
				Some(AddonError::Io(io) | AddonError::Zip(ZipError::Io(io))) => Some(io),
				_ => None,
			});
			io.and_then(std::io::Error::raw_os_error)
		});
		Self { error: format!("{e:#}"), os }
	}
}

/// The feed's checks: wake one now (`now`).
#[derive(Clone)]
pub struct Checks(Arc<Notify>);

impl Checks {
	/// A check at once, or right after the one that runs.
	pub fn now(&self) {
		self.0.notify_one();
	}
}

/// The client every addon request goes through: the helper's User-Agent, the system's TLS, GitHub's redirects to its
/// storage followed.
fn client() -> reqwest::Result<reqwest::Client> {
	reqwest::Client::builder()
		.user_agent(crate::feedback::USER_AGENT)
		.tls_backend_native()
		.timeout(TIMEOUT)
		.connect_timeout(CONNECT_TIMEOUT)
		.build()
}

/// Starts the feed's checks; none without a feed or a key in this build (logged).
pub fn spawn(app: &AppHandle, news: mpsc::UnboundedSender<News>) -> Checks {
	let checks = Checks(Arc::new(Notify::new()));
	let (Some(feed), Some(sig), Some(zip), Some(key)) =
		(update::feed_file(app, FEED), update::feed_file(app, SIG), update::feed_file(app, ZIP), update::pubkey(app))
	else {
		log::warn!("addon: this build has no feed or no key: QuestPump cannot add or update the addon");
		return checks;
	};
	let wake = checks.0.clone();
	tauri::async_runtime::spawn(async move {
		let _ = tokio::time::timeout(FIRST, wake.notified()).await;
		loop {
			let checked = match client() {
				Ok(client) => fetch(&client, &feed, &sig, &zip, &key).await.map_err(|e| format!("{e:#}")),
				Err(e) => Err(e.to_string()),
			};
			if news.send(News::Feed(checked)).is_err() {
				return; // the loop has gone: QuestPump quits
			}
			let _ = tokio::time::timeout(EVERY, wake.notified()).await;
		}
	});
	checks
}

/// A small file of the feed, at most `MAX_FEED` bytes.
async fn small(client: &reqwest::Client, url: &str) -> anyhow::Result<Vec<u8>> {
	let mut response = client.get(url).send().await?.error_for_status()?;
	let mut body = Vec::new();
	while let Some(chunk) = response.chunk().await? {
		body.extend_from_slice(&chunk);
		anyhow::ensure!(body.len() as u64 <= MAX_FEED, "{url}: more than {MAX_FEED} bytes");
	}
	Ok(body)
}

/// `addon.json` and its signature, checked, its zip at `zip`.
async fn fetch(client: &reqwest::Client, feed: &str, sig: &str, zip: &str, key: &str) -> anyhow::Result<Feed> {
	let body = small(client, feed).await?;
	let sig = String::from_utf8(small(client, sig).await?).context("addon.json.sig is not text")?;
	Ok(addon::verify(&body, &sig, key, zip)?)
}

/// What the three folders in `addons` hold, off the loop, read in the loop's `epoch`.
pub fn look(addons: PathBuf, epoch: u64, news: mpsc::UnboundedSender<News>) {
	tauri::async_runtime::spawn(async move {
		let at = addons.clone();
		let tree = tokio::task::spawn_blocking(move || addon::tree(&at))
			.await
			.map_err(|e| e.to_string())
			.and_then(|r| r.map_err(|e| e.to_string()));
		let _ = news.send(News::Tree { addons, epoch, tree });
	});
}

/// Stages the feed's zip for `addons`: a swap cut short before is put right first (`addon::recover`; a first install
/// whose QuestPump that puts back stops there), then the zip comes into the staging folder and is unpacked there,
/// every check made.
pub fn stage(addons: PathBuf, feed: Feed, first: bool, news: mpsc::UnboundedSender<News>) {
	tauri::async_runtime::spawn(async move {
		let result = staged(&addons, &feed, first).await.map_err(|e| Failure::of(&e));
		let _ = news.send(News::Staged { addons, result });
	});
}

async fn staged(addons: &Path, feed: &Feed, first: bool) -> anyhow::Result<Option<PathBuf>> {
	let staging = addon::staging(addons).context("the game folder has no Interface folder")?;
	let (at, from) = (addons.to_path_buf(), staging.clone());
	let back = tokio::task::spawn_blocking(move || addon::recover(&at, &from)).await??;
	if !back.is_empty() {
		log::warn!("addon: a swap cut short left {} aside; put back in {}", back.join(", "), addons.display());
		if first && back.contains(&addon::FOLDERS[0]) {
			return Ok(None);
		}
	}
	tokio::fs::create_dir_all(&staging).await.with_context(|| staging.display().to_string())?;
	let zip = staging.join(ZIP);
	download(feed, &zip).await?;
	let (from, into, feed) = (zip.clone(), staging.join(addon::NEW), feed.clone());
	tokio::task::spawn_blocking(move || addon::unpack(&from, &feed, &into)).await??;
	let _ = tokio::fs::remove_file(&zip).await;
	Ok(Some(staging))
}

/// The zip into `to`, no more bytes than the feed says, its sha256 the feed's.
async fn download(feed: &Feed, to: &Path) -> anyhow::Result<()> {
	let client = client()?;
	let mut response = client.get(&feed.zip.url).send().await?.error_for_status()?;
	let mut file = tokio::fs::File::create(to).await.with_context(|| to.display().to_string())?;
	let mut hasher = Sha256::new();
	let mut got = 0_u64;
	while let Some(chunk) = response.chunk().await? {
		got += chunk.len() as u64;
		anyhow::ensure!(got <= feed.zip.size, "the zip is larger than the feed says");
		hasher.update(&chunk);
		file.write_all(&chunk).await?;
	}
	file.flush().await?;
	anyhow::ensure!(got == feed.zip.size, "the zip came {got} of its {} bytes", feed.zip.size);
	let sha256 = questpump_core::zip::hex(&hasher.finalize());
	anyhow::ensure!(sha256 == feed.zip.sha256, "the zip's sha256 is not the feed's");
	Ok(())
}

/// The swap, off the loop: `check` first (the folders still as the loop found them: no link, no `.git`, and for a
/// managed install still the record's files), then, unless `held` (the loop's "the game or a pump runs", kept current
/// at every look; none for a first install, which may go in while the game runs) says so now, after the folders' read,
/// `addon::swap`.
pub fn swap(
	addons: PathBuf,
	staging: PathBuf,
	held: Option<Arc<AtomicBool>>,
	check: impl FnOnce(&addon::Tree) -> Result<(), String> + Send + 'static,
	news: mpsc::UnboundedSender<News>,
) {
	tauri::async_runtime::spawn(async move {
		let (at, from) = (addons.clone(), staging);
		let result = tokio::task::spawn_blocking(move || -> anyhow::Result<bool> {
			let tree = addon::tree(&at)?;
			check(&tree).map_err(anyhow::Error::msg)?;
			if held.is_some_and(|held| held.load(Ordering::Relaxed)) {
				return Ok(false);
			}
			addon::swap(&at, &from).with_context(|| at.display().to_string())?;
			Ok(true)
		})
		.await
		.map_err(anyhow::Error::from)
		.and_then(|r| r)
		.map_err(|e| Failure::of(&e));
		let _ = news.send(match result {
			Ok(false) => News::Held { addons },
			result => News::Swapped { addons, result: result.map(|_| ()) },
		});
	});
}

/// The staging folder of `addons` settled off the loop (`addon::recover`): what a swap cut short left aside goes back,
/// then the folder goes, with the old folders a finished swap replaced.
pub fn settle(addons: PathBuf, news: mpsc::UnboundedSender<News>) {
	tauri::async_runtime::spawn(async move {
		let at = addons.clone();
		let result = tokio::task::spawn_blocking(move || match addon::staging(&at) {
			Some(staging) => addon::recover(&at, &staging).map_err(|e| format!("{}: {e}", staging.display())),
			None => Ok(Vec::new()),
		})
		.await
		.map_err(|e| e.to_string())
		.and_then(|r| r);
		let _ = news.send(News::Settled { addons, result });
	});
}
