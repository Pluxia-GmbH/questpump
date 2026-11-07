//! The addon from QuestPump on this computer (docs/companion.md, "The addon on this computer"; questpump-core `addon`
//! holds the feed's rules, the zip's checks, the record and the swap): the release feed fetched off the loop on the
//! updater's schedule (`feeds`: 10 s after the start, then hourly, sooner again after a failure) and when the loop asks
//! (`Checks::now`: the game closed, a game without QuestPump found, the player's Add QuestPump to the game before a
//! feed came), `addon.json` and `addon.json.sig` from the updater's own `helper-updater` release checked against the
//! updater's key (`update::pubkey`), each within `feeds::FILE_TIMEOUT`, the zip's address the same release's
//! `QuestPump-addon.zip`; then, for the install the loop picked, the feed's files staged in the flavour's
//! `Interface/.questpump-update/new/` (the same volume as AddOns, outside the folder the client walks) file by file:
//! each file this computer holds already by its sha256 (the install's own, the desktop app's voice takes), the rest out
//! of the zip by byte ranges (its end, its central directory, then the spans that hold them; a request that breaks off
//! is asked once more, `again`), and whenever that cannot finish (no 206, a range or a size that is not the feed's
//! zip's, the zip replaced meanwhile, an entry that fails a check, a zip feature not read) the whole zip as before,
//! checked against the feed's size and sha256 as it comes and unpacked there with every file checked (`addon::unpack`);
//! every request with no bound on the whole, only `feeds::READ_TIMEOUT` between its bytes, and every staged file's size
//! and sha256 the signed feed's. The swap and the cleaning up run off the loop too (`swap`, `settle`), each reporting
//! back; the loop decides when (`runtime/addon.rs`).

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::Context as _;
use questpump_core::addon::{self, AddonError, Feed};
use questpump_core::zip::ZipError;
use sha2::{Digest as _, Sha256};
use tauri::AppHandle;
use tokio::io::{AsyncWrite, AsyncWriteExt as _};
use tokio::sync::mpsc;

use crate::feeds::{self, Checks};
use crate::update;

/// The longest `addon.json` or its signature may be.
const MAX_FEED: u64 = 16 << 20;
/// The files on the release, beside the updater's `latest.json`; the zip keeps its name in the staging folder.
const FEED: &str = "addon.json";
const SIG: &str = "addon.json.sig";
const ZIP: &str = "QuestPump-addon.zip";
/// The staging folder's file that holds the span of the zip fetched last (`by_ranges`).
const PART: &str = "QuestPump-addon.part";
/// The longest central directory the zip may have (6,775 entries took 651,563 bytes in 1.0.0).
const MAX_DIRECTORY: u64 = 16 << 20;

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

/// The client every addon request goes through: the helper's User-Agent, the system's TLS, GitHub's redirects to its
/// storage followed, the feeds' wait for the next bytes and for the connect (the small files' bound on the whole is
/// their request's, `small`).
fn client() -> reqwest::Result<reqwest::Client> {
	reqwest::Client::builder()
		.user_agent(crate::feedback::USER_AGENT)
		.tls_backend_native()
		.read_timeout(feeds::READ_TIMEOUT)
		.connect_timeout(feeds::CONNECT_TIMEOUT)
		.build()
}

/// Starts the feed's checks on the feeds' schedule (`feeds`; the returned `Checks` wakes them); none without a feed or a
/// key in this build (logged).
pub fn spawn(app: &AppHandle, news: mpsc::UnboundedSender<News>) -> Checks {
	let checks = Checks::default();
	let (Some(feed), Some(sig), Some(zip), Some(key)) =
		(update::feed_file(app, FEED), update::feed_file(app, SIG), update::feed_file(app, ZIP), update::pubkey(app))
	else {
		log::warn!("addon: this build has no feed or no key: QuestPump cannot add or update the addon");
		return checks;
	};
	let wake = checks.clone();
	tauri::async_runtime::spawn(async move {
		let mut schedule = wake.first().await;
		loop {
			let checked = match client() {
				Ok(client) => fetch(&client, &feed, &sig, &zip, &key).await.map_err(|e| format!("{e:#}")),
				Err(e) => Err(e.to_string()),
			};
			let ok = checked.is_ok();
			if news.send(News::Feed(checked)).is_err() {
				return; // the loop has gone: QuestPump quits
			}
			schedule.next(ok).await;
		}
	});
	checks
}

/// A small file of the feed, at most `MAX_FEED` bytes within `feeds::FILE_TIMEOUT`.
async fn small(client: &reqwest::Client, url: &str) -> anyhow::Result<Vec<u8>> {
	let mut response = client.get(url).timeout(feeds::FILE_TIMEOUT).send().await?.error_for_status()?;
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

/// Stages the feed for `addons`: a swap cut short before is put right first (`addon::recover`; a first install whose
/// QuestPump that puts back stops there), then the feed's files come into the staging folder file by file (`staged`),
/// every check made. `voice` is the desktop app's own folder of the addon's voice takes.
pub fn stage(addons: PathBuf, feed: Feed, first: bool, voice: Option<PathBuf>, news: mpsc::UnboundedSender<News>) {
	tauri::async_runtime::spawn(async move {
		let result = staged(&addons, &feed, first, voice).await.map_err(|e| Failure::of(&e));
		let _ = news.send(News::Staged { addons, result });
	});
}

/// The feed's files into the staging folder's `new/`: those this computer holds already (`addon::take_local`), then
/// the rest out of the zip by byte ranges (`by_ranges`), else, when that cannot finish, the whole zip as before
/// (`whole`); the TOC's version checked last.
async fn staged(addons: &Path, feed: &Feed, first: bool, voice: Option<PathBuf>) -> anyhow::Result<Option<PathBuf>> {
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
	let into = staging.join(addon::NEW);
	let (local, at, to) = (feed.clone(), addons.to_path_buf(), into.clone());
	let wanted = tokio::task::spawn_blocking(move || addon::take_local(&local, &at, voice.as_deref(), &to)).await??;
	let (all, here) = (feed.files.len(), feed.files.len() - wanted.len());
	log::info!("addon: {}: {here} of its {all} files from this computer", feed.version);
	let client = client()?;
	if !wanted.is_empty()
		&& let Err(e) = by_ranges(&client, feed, &wanted, &staging).await
	{
		log::warn!("addon: {}: file by file did not finish ({e:#}): the whole zip instead", feed.version);
		whole(&client, feed, &staging).await?;
		return Ok(Some(staging));
	}
	let (checked, to) = (feed.clone(), into);
	tokio::task::spawn_blocking(move || addon::check_version(&to, &checked)).await??;
	Ok(Some(staging))
}

/// The files `wanted` out of the feed's zip by byte ranges into the staging folder's `new/`: the zip's end and central
/// directory first (`zip::directory`, `zip::entries`), every entry checked against the feed as `unpack` checks them
/// (`addon::plan`), then each span into `PART` and unpacked out of it (`addon::unpack_span`), every file's sha256 the
/// feed's.
async fn by_ranges(
	client: &reqwest::Client,
	feed: &Feed,
	wanted: &[addon::FeedFile],
	staging: &Path,
) -> anyhow::Result<()> {
	let mut zip = Ranges { client, zip: &feed.zip, answered: false, etag: None };
	let size = feed.zip.size;
	let tail_at = size.saturating_sub(questpump_core::zip::TAIL as u64);
	let tail = zip.bytes(tail_at, size).await?;
	let directory = questpump_core::zip::directory(&tail, tail_at)?;
	anyhow::ensure!(directory.size <= MAX_DIRECTORY, "a central directory of {} bytes", directory.size);
	let central = zip.bytes(directory.offset, directory.offset + directory.size).await?;
	let entries = questpump_core::zip::entries(&central, directory.count)?;
	let plan = addon::plan(&entries, directory.offset, feed, wanted)?;
	let (files, bytes, requests) = (plan.files(), plan.bytes(), plan.spans().len());
	let read = tail.len() + central.len();
	log::info!(
		"addon: {}: {files} files to fetch, {bytes} bytes in {requests} requests after {read} of the zip's directory",
		feed.version
	);
	let into = staging.join(addon::NEW);
	let plan = Arc::new(plan);
	let part = staging.join(PART);
	for (i, span) in plan.spans().iter().enumerate() {
		zip.file(span.start, span.end, &part).await?;
		let (from, plan, to) = (part.clone(), Arc::clone(&plan), into.clone());
		tokio::task::spawn_blocking(move || addon::unpack_span(&from, &plan.spans()[i], &to)).await??;
	}
	// one left goes with the staging folder after the swap (`addon::recover`)
	let _ = tokio::fs::remove_file(&part).await;
	Ok(())
}

/// The feed's zip read by byte ranges. Every request goes to its address on `github.com`, which sends it on to the
/// asset's signed address in GitHub's storage, resolved anew each time, so none outlives its signature; the answer
/// must be 206 with exactly the range asked of a file of the feed's size, from the file the first answer came from
/// (its `ETag`), so a zip replaced on the release meanwhile fails and the staging takes the whole zip instead (whose
/// sha256 then decides).
struct Ranges<'a> {
	client: &'a reqwest::Client,
	zip: &'a addon::Zip,
	/// Whether an answer came, and the first one's `ETag` (none where it had none).
	answered: bool,
	etag: Option<reqwest::header::HeaderValue>,
}

impl Ranges<'_> {
	/// The zip's bytes `start..end`, asked once more when the first request breaks off (`again`).
	async fn bytes(&mut self, start: u64, end: u64) -> anyhow::Result<Vec<u8>> {
		let mut retried = false;
		loop {
			let mut out = Vec::new();
			match self.fetch(start, end, &mut out).await {
				Err(e) if again(&e, &mut retried) => {}
				done => return done.map(|()| out),
			}
		}
	}

	/// The zip's bytes `start..end` into the file `to`, made anew, asked once more when the first request breaks off
	/// (`again`).
	async fn file(&mut self, start: u64, end: u64, to: &Path) -> anyhow::Result<()> {
		let mut retried = false;
		loop {
			let mut file = tokio::fs::File::create(to).await.with_context(|| to.display().to_string())?;
			let done = self.fetch(start, end, &mut file).await;
			file.flush().await?;
			match done {
				Err(e) if again(&e, &mut retried) => {}
				done => return done,
			}
		}
	}

	/// The zip's bytes `start..end` into `out`.
	async fn fetch(&mut self, start: u64, end: u64, out: &mut (impl AsyncWrite + Unpin)) -> anyhow::Result<()> {
		use reqwest::header::{CONTENT_RANGE, ETAG, RANGE};
		anyhow::ensure!(start < end && end <= self.zip.size, "bytes {start}..{end} of a zip of {}", self.zip.size);
		let last = end - 1;
		let mut response = self.client.get(&self.zip.url).header(RANGE, format!("bytes={start}-{last}")).send().await?;
		let status = response.status();
		anyhow::ensure!(status == reqwest::StatusCode::PARTIAL_CONTENT, "{start}-{last}: answered {status}, not 206");
		let range = response.headers().get(CONTENT_RANGE).and_then(|v| v.to_str().ok()).unwrap_or_default();
		let size = self.zip.size;
		anyhow::ensure!(range == format!("bytes {start}-{last}/{size}"), "asked {start}-{last}/{size}, came {range:?}");
		let etag = response.headers().get(ETAG).cloned();
		if !std::mem::replace(&mut self.answered, true) {
			self.etag.clone_from(&etag);
		}
		anyhow::ensure!(self.etag == etag, "the zip changed on the release (its ETag)");
		let want = end - start;
		let mut got = 0_u64;
		while let Some(chunk) = response.chunk().await? {
			got += chunk.len() as u64;
			anyhow::ensure!(got <= want, "{start}-{last}: more than the {want} bytes asked");
			out.write_all(&chunk).await?;
		}
		anyhow::ensure!(got == want, "{start}-{last}: came {got} of its {want} bytes");
		Ok(())
	}
}

/// Whether a failed range request is asked once more: only when the connection broke off (a `reqwest` error: a reset,
/// a timeout) and it has not been asked again yet, since every failure ends in the whole zip, 272 MB for 1.0.0. An
/// answer that came but fails a check (its status, range, `ETag` or length) is not asked again.
fn again(e: &anyhow::Error, retried: &mut bool) -> bool {
	if *retried || e.downcast_ref::<reqwest::Error>().is_none() {
		return false;
	}
	*retried = true;
	log::info!("addon: a range request broke off ({e:#}); asking once more");
	true
}

/// The whole zip into the staging folder and unpacked into its `new/` (`addon::unpack`, every check made), as before
/// the files came one by one.
async fn whole(client: &reqwest::Client, feed: &Feed, staging: &Path) -> anyhow::Result<()> {
	// the span the ranged path fetched last, if any: the disk room for the zip
	let _ = tokio::fs::remove_file(staging.join(PART)).await;
	let zip = staging.join(ZIP);
	download(client, feed, &zip).await?;
	let (from, into, feed) = (zip.clone(), staging.join(addon::NEW), feed.clone());
	tokio::task::spawn_blocking(move || addon::unpack(&from, &feed, &into)).await??;
	let _ = tokio::fs::remove_file(&zip).await;
	Ok(())
}

/// The zip into `to`, no more bytes than the feed says, its sha256 the feed's.
async fn download(client: &reqwest::Client, feed: &Feed, to: &Path) -> anyhow::Result<()> {
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
