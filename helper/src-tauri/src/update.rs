//! The helper's updates (rust-helper `tauri-shell.md`, "The updater"): `tauri-plugin-updater` reads the release feed
//! (`plugins.updater.endpoints`, the public repository's permanent `helper-updater` release) on the feeds' schedule
//! (`feeds`: 10 s after the start, then hourly on the wall clock, sooner again after a failure, and when the game
//! closes), downloads a newer helper and verifies its minisign signature against `plugins.updater.pubkey` (the signed
//! version must be the feed's, `requireSignedVersion`); an empty key refuses every download. A Mac account that cannot
//! replace the app downloads nothing (`replaceable`). The verified download is kept in the data folder's `updates`
//! (`FOLDER`), never in memory, so a quit or a failed install keeps it: a release whose file is there is not fetched
//! again (`fetch`; `Update::install` verifies the bytes again), and the files of other releases go. `latest.json` may
//! take `feeds::FILE_TIMEOUT` as a whole; the download has no such bound, only `feeds::READ_TIMEOUT` between its bytes
//! and `feeds::CONNECT_TIMEOUT`, so a slow line still brings it. Only a feed that did not answer is asked again on the
//! feeds' quick retries: a failed download, signature or write waits the hourly check, and bytes refused for their
//! signature or not written are not fetched again in this run until the feed names other bytes (`Met`). The loop
//! installs it once the game and the main window are closed, no pump runs and no watched game is in a match, or at
//! This computer's Restart to update (`runtime/updates.rs`). Only a published build running from its installed bundle
//! updates itself: a development build would be replaced by a release, from `target/debug` the updater would treat that
//! folder as the app, and a Mac helper still in its disk image cannot be replaced (`unmoved`). The new helper starts
//! with `--updated` on both systems, so it opens no window (`desk::launch`), but This computer after the player's
//! Restart to update (`restarted`).

use std::path::{Path, PathBuf};

use anyhow::Context as _;
use questpump_core::mirror;
use tauri::AppHandle;

use crate::desk::launch;
use crate::feeds::{self, Checks};
use crate::ui::Build;
#[cfg(windows)]
use tauri::utils::{config::BundleType, platform::bundle_type};
use tauri_plugin_updater::{Update, Updater, UpdaterExt as _};
use tokio::sync::mpsc::UnboundedSender;

/// The data folder's folder of the verified download, `<version>.<extension>` (`file_name`).
const FOLDER: &str = "updates";
/// The data folder's note that the player's Restart to update installs the version it holds: that version's start opens
/// This computer (`restarted`).
const RESTART: &str = "update-restart";

/// A file of the updater's feed: the folder of its first endpoint (`plugins.updater.endpoints`, the public
/// repository's permanent `helper-updater` release) with `name` in place of `latest.json` (the films' zip, the addon's
/// `addon.json`, its signature and its zip).
pub(crate) fn feed_file(app: &AppHandle, name: &str) -> Option<String> {
	let endpoint = updater_config(app)?.get("endpoints")?.get(0)?.as_str()?;
	let (folder, _) = endpoint.rsplit_once('/')?;
	Some(format!("{folder}/{name}"))
}

/// The updater's public key, the content of `plugins.updater.pubkey` (the base64 of minisign's key text), which the
/// addon's feed is checked against too; none while it is empty.
pub(crate) fn pubkey(app: &AppHandle) -> Option<String> {
	updater_config(app)?.get("pubkey")?.as_str().filter(|k| !k.is_empty()).map(str::to_owned)
}

fn updater_config(app: &AppHandle) -> Option<&serde_json::Value> {
	app.config().plugins.0.get("updater")
}

/// A verified update kept in the data folder: the plugin's update and its file.
pub struct Ready {
	update: Update,
	path: PathBuf,
}

impl Ready {
	pub fn version(&self) -> &str {
		&self.update.version
	}

	/// Its file is still there (`install` deletes one that no longer matches its signature; a check that finds no
	/// newer release, or an account that cannot replace the app, deletes every one).
	pub fn kept(&self) -> bool {
		self.path.is_file()
	}
}

/// What a check found.
pub enum Found {
	/// Boxed: the plugin's update carries its whole manifest.
	Ready(Box<Ready>),
	Current,
	/// macOS: this version is out, and this account cannot replace the app, so nothing was downloaded.
	Unreplaceable(String),
	Failed(String),
}

/// A published build running from its installed bundle: on Windows the one the NSIS installer put there (the bundler
/// marks only the packaged copy of the program, so `target\release` does not count), on macOS a `.app`'s own program
/// that is not still in its download (`unmoved`).
fn installed() -> bool {
	#[cfg(windows)]
	let bundled = bundle_type() == Some(BundleType::Nsis);
	#[cfg(not(windows))]
	let bundled = app_bundle().is_some() && !unmoved();
	!cfg!(debug_assertions) && bundled
}

/// The Mac helper runs from its download instead of from where it was installed: the root of a mounted disk image
/// (`/Volumes/<image>/QuestPump.app`), or the copy macOS runs a quarantined app from where it was downloaded
/// (`…/AppTranslocation/…`, Gatekeeper's path randomisation). From there it cannot start at login (the path is gone
/// once the image is ejected or the Mac restarts) or replace itself, so the window asks the player to move it into
/// Applications.
#[cfg(target_os = "macos")]
pub fn unmoved() -> bool {
	app_bundle().is_some_and(|bundle| in_download(&bundle))
}

/// An app bundle lies where its download left it: at the root of a mounted disk image, or translocated.
#[cfg(target_os = "macos")]
fn in_download(bundle: &std::path::Path) -> bool {
	bundle.parent().and_then(std::path::Path::parent) == Some(std::path::Path::new("/Volumes"))
		|| bundle.components().any(|c| c.as_os_str() == "AppTranslocation")
}

/// The copy of QuestPump a second launch ran (`program`, its path as `argv[0]` gives it) while this one runs from its
/// download, when that copy lies outside any download: the one the player moved into Applications, as This computer's
/// move card asks, which takes over (`desk::launch::second`, `start_moved`). None for any other launch.
#[cfg(target_os = "macos")]
pub fn moved_copy(program: &std::path::Path) -> Option<std::path::PathBuf> {
	let bundle = bundle_of(program)?;
	(unmoved() && !in_download(&bundle) && app_bundle().as_ref() != Some(&bundle)).then_some(bundle)
}

/// Windows' installer puts the helper where it runs.
#[cfg(not(target_os = "macos"))]
pub fn unmoved() -> bool {
	false
}

/// The `.app` this program runs from.
#[cfg(not(windows))]
fn app_bundle() -> Option<std::path::PathBuf> {
	bundle_of(&std::env::current_exe().ok()?)
}

/// The `.app` a program lies in (`<name>.app/Contents/MacOS/<program>`).
#[cfg(not(windows))]
fn bundle_of(program: &std::path::Path) -> Option<std::path::PathBuf> {
	let macos = program.parent().filter(|p| p.ends_with("Contents/MacOS"))?;
	let bundle = macos.parent()?.parent()?;
	bundle.extension().is_some_and(|e| e == "app").then(|| bundle.to_path_buf())
}

/// Whether this account may replace the helper's app without an administrator (macOS: the plugin moves the bundle
/// out of its folder, and where it may not it asks for an administrator's password through AppleScript on the main
/// thread, holding the helper's loop until answered). Windows' installer runs per user.
#[cfg(target_os = "macos")]
pub fn replaceable() -> bool {
	use std::os::unix::ffi::OsStrExt as _;
	let writable = |path: &std::path::Path| {
		std::ffi::CString::new(path.as_os_str().as_bytes())
			// SAFETY: `access` reads the NUL-terminated path for the call only.
			.is_ok_and(|c| unsafe { libc::access(c.as_ptr(), libc::W_OK) } == 0)
	};
	// moving a folder to another parent rewrites its `..`, so the bundle itself must be writable too
	app_bundle().is_some_and(|bundle| writable(&bundle) && bundle.parent().is_some_and(writable))
}

#[cfg(not(target_os = "macos"))]
pub fn replaceable() -> bool {
	true
}

/// What kind of build runs, for This computer's version line before the first check.
pub fn build() -> Build {
	if installed() {
		Build::Installed
	} else if unmoved() {
		Build::Unmoved
	} else {
		Build::Development
	}
}

/// Checks for an update on the feeds' schedule (`feeds`; the returned `Checks` wakes it) and reports each check; a newer
/// release found later replaces a pending one, and the same release found again is reported again from its file, so the
/// loop holds it whatever became of it meanwhile.
pub fn spawn(app: AppHandle, dir: &Path, found: UnboundedSender<Found>) -> Checks {
	let checks = Checks::default();
	if !installed() {
		return checks;
	}
	let (wake, folder) = (checks.clone(), dir.join(FOLDER));
	tauri::async_runtime::spawn(async move {
		#[cfg(windows)]
		{
			let at = app.clone();
			let _ = tokio::task::spawn_blocking(move || sweep_installers(&at)).await;
		}
		let mut schedule = wake.first().await;
		let mut met = Met::default();
		loop {
			let (news, answered) = check(&app, &folder, &mut met).await;
			if let Some(news) = news
				&& found.send(news).is_err()
			{
				return; // the loop has gone: the helper quits
			}
			schedule.next(answered).await;
		}
	});
	checks
}

/// A release's bytes, by its version and signature (a release published again under the same version signs other
/// bytes).
type Bytes = (String, String);

/// The releases this run met: the one whose file it wrote or verified (`kept`), and the one whose download failed its
/// signature or could not be written (`refused`), which is not downloaded again until the feed names other bytes.
#[derive(Default)]
struct Met {
	kept: Option<Bytes>,
	refused: Option<Bytes>,
}

/// One check, and whether the feed answered it (a failure after that, the download's, its signature's or its write's,
/// waits the hourly check, never the quick retries): the feed read, and for a newer release its file kept in `folder`
/// (downloaded unless it is there) and the other files there deleted; no newer release, or an account that cannot
/// replace the app, deletes them all. No news when the feed names bytes this run refused, which was reported once.
async fn check(app: &AppHandle, folder: &Path, met: &mut Met) -> (Option<Found>, bool) {
	let checked = match updater(app) {
		Ok(updater) => updater.check().await,
		Err(e) => Err(e),
	};
	let update = match checked {
		Ok(Some(update)) => update,
		Ok(None) => {
			clear(folder, None).await;
			return (Some(Found::Current), true);
		}
		Err(e) => return (Some(Found::Failed(e.to_string())), false),
	};
	if !replaceable() {
		clear(folder, None).await;
		return (Some(Found::Unreplaceable(update.version)), true);
	}
	if met.refused.as_ref() == Some(&(update.version.clone(), update.signature.clone())) {
		return (None, true);
	}
	let name = file_name(&update);
	if let Err(e) = fetch(&update, folder, &name, met).await {
		return (Some(Found::Failed(format!("{}: {e:#}", update.version))), true);
	}
	let path = folder.join(name);
	clear(folder, Some(&path)).await;
	(Some(Found::Ready(Box::new(Ready { update, path }))), true)
}

/// The updater: `latest.json` may take `feeds::FILE_TIMEOUT` as a whole (2.14.1 hands the builder's timeout to the check
/// alone; the update it finds carries none to its download), and every request, the download too (`configure_client`
/// reaches it), waits `feeds::READ_TIMEOUT` at most for its next bytes and `feeds::CONNECT_TIMEOUT` to connect.
/// Windows: the installer starts the new helper with what follows its /ARGS, which runs to its next option, so the
/// installer's own argument after the plugin's /ARGS reaches the helper (macOS relaunches it with `--updated` itself).
fn updater(app: &AppHandle) -> tauri_plugin_updater::Result<Updater> {
	app.updater_builder()
		.timeout(feeds::FILE_TIMEOUT)
		.configure_client(|client| client.read_timeout(feeds::READ_TIMEOUT).connect_timeout(feeds::CONNECT_TIMEOUT))
		.installer_arg(launch::UPDATED)
		.build()
}

/// The release's file `name` in `folder`: the one there kept when it is this release's (read and verified once a run),
/// else downloaded, verified (`Update::download`) and written through a file of its own and a rename, so a write cut
/// short leaves nothing under the release's name; the bytes leave memory once written. Bytes that fail their signature
/// or cannot be written are refused for the run (`Met::refused`).
async fn fetch(update: &Update, folder: &Path, name: &str, met: &mut Met) -> anyhow::Result<()> {
	let this = (update.version.clone(), update.signature.clone());
	let path = folder.join(name);
	if met.kept.as_ref() == Some(&this) && path.is_file() {
		return Ok(());
	}
	met.kept = None;
	let release = update.clone();
	let there =
		tokio::task::spawn_blocking(move || std::fs::read(&path).is_ok_and(|b| release.verify(&b).is_ok())).await?;
	if there {
		log::info!("update: {} kept from an earlier download", update.version);
	} else {
		log::info!("update: {} downloading", update.version);
		let bytes = update.download(|_, _| {}, || {}).await.inspect_err(|e| {
			if refused_signature(e) {
				refuse(met, &this);
			}
		})?;
		let (folder, name) = (folder.to_path_buf(), name.to_owned());
		let written = tokio::task::spawn_blocking(move || {
			std::fs::create_dir_all(&folder)?;
			mirror::write_atomic(&folder, &name, &bytes)
		})
		.await?;
		written.inspect_err(|_| refuse(met, &this))?;
		log::info!("update: {} downloaded", update.version);
	}
	met.kept = Some(this);
	Ok(())
}

/// A download's failure that the same bytes would fail again: its signature (`requireSignedVersion` included).
fn refused_signature(e: &tauri_plugin_updater::Error) -> bool {
	use tauri_plugin_updater::Error;
	matches!(
		e,
		Error::Minisign(_)
			| Error::Base64(_)
			| Error::SignatureUtf8(_)
			| Error::MissingSignedVersion
			| Error::SignedVersionMismatch { .. }
	)
}

/// These bytes are not downloaded again in this run (logged once: later checks that name them report nothing).
fn refuse(met: &mut Met, bytes: &Bytes) {
	log::warn!("update: {} refused: not downloaded again until the feed names other bytes", bytes.0);
	met.refused = Some(bytes.clone());
}

/// The release's file name, `<version>.<the download's extension>` (`gz` for the Mac's `.app.tar.gz`, `exe` for
/// Windows' installer; `bin` when its address names none); the plugin writes the version as semver, so it names no
/// folder.
fn file_name(update: &Update) -> String {
	let extension = Path::new(update.download_url.path())
		.extension()
		.and_then(|e| e.to_str())
		.filter(|e| !e.is_empty() && e.chars().all(|c| c.is_ascii_alphanumeric()))
		.unwrap_or("bin");
	format!("{}.{extension}", update.version)
}

/// Every file in `folder` deleted but `keep`: the files of other releases, and a write a quit cut short.
async fn clear(folder: &Path, keep: Option<&Path>) {
	let (folder, keep) = (folder.to_path_buf(), keep.map(Path::to_path_buf));
	let _ = tokio::task::spawn_blocking(move || {
		let Ok(entries) = std::fs::read_dir(&folder) else { return };
		for entry in entries.flatten() {
			let path = entry.path();
			if keep.as_ref() != Some(&path)
				&& entry.file_type().is_ok_and(|t| t.is_file())
				&& let Err(e) = std::fs::remove_file(&path)
			{
				log::warn!("update: {}: {e}", path.display());
			}
		}
	})
	.await;
}

/// Installs a kept update from its file and starts the new helper, brought forward when the player asked for it
/// (`front`: This computer's Restart to update), else in the background. On Windows the installer takes over and ends
/// this process inside `install` (passive, then it starts the helper again); on macOS the bundle is replaced in place
/// and the new one is started as below. A file that no longer matches its signature is deleted, so the next check
/// downloads the release again.
///
/// # Errors
/// The file did not read, the update could not be installed, or the new helper could not be started.
pub fn install(app: &AppHandle, ready: &Ready, front: bool) -> anyhow::Result<()> {
	let bytes = std::fs::read(&ready.path).with_context(|| ready.path.display().to_string())?;
	if let Err(e) = ready.update.install(&bytes) {
		if matches!(e, tauri_plugin_updater::Error::Minisign(_)) {
			let _ = std::fs::remove_file(&ready.path);
		}
		return Err(e.into());
	}
	relaunch(app, front)
}

/// Notes in the data folder `dir`, right before the install, that the player's Restart to update installs `version`:
/// that version's start opens This computer (`restarted`), on Windows too, whose installer starts it with the fixed
/// `--updated`.
pub fn note_restart(dir: &Path, version: &str) {
	if let Err(e) = mirror::write_atomic(dir, RESTART, version.as_bytes()) {
		log::warn!("update: {RESTART}: {e}");
	}
}

/// The note goes again: the install it was for failed.
pub fn drop_restart(dir: &Path) {
	let _ = std::fs::remove_file(dir.join(RESTART));
}

/// At the start: the note in the data folder `dir` says the player's Restart to update installed the version that runs
/// (`running`), so the window opens on This computer. The note goes either way, so no later start opens it.
pub fn restarted(dir: &Path, running: &str) -> bool {
	let path = dir.join(RESTART);
	let Ok(version) = std::fs::read_to_string(&path) else { return false };
	if let Err(e) = std::fs::remove_file(&path) {
		log::warn!("update: {RESTART}: {e}");
	}
	version == running
}

/// Windows: the folders the plugin's installs left in the temporary folder, `<productName>-<version>-updater-<random>`
/// with the installer in it (2.14.1 `make_temp_dir` keeps it, and the installer ends this process before anything could
/// remove it), deleted at the start but the running version's, whose installer may still be closing.
#[cfg(windows)]
fn sweep_installers(app: &AppHandle) {
	let info = app.package_info();
	let (prefix, running) = (format!("{}-", info.name), info.version.to_string());
	let Ok(entries) = std::fs::read_dir(std::env::temp_dir()) else { return };
	for entry in entries.flatten() {
		let name = entry.file_name();
		let version = name.to_str().and_then(|n| n.strip_prefix(&prefix)).and_then(|n| n.rsplit_once("-updater-"));
		if version.is_none_or(|(v, _)| v == running) || !entry.file_type().is_ok_and(|t| t.is_dir()) {
			continue;
		}
		let path = entry.path();
		match std::fs::remove_dir_all(&path) {
			Ok(()) => log::info!("update: {} deleted", path.display()),
			Err(e) => log::warn!("update: {}: {e}", path.display()),
		}
	}
}

/// This computer's "Restart QuestPump" on the Screen Recording card (macOS applies a grant only to a process started
/// after it): the same helper started again as an update's is, with its window open on This computer.
///
/// # Errors
/// The helper runs from no app bundle, or the shell that starts it again could not be started.
#[cfg(target_os = "macos")]
pub fn restart(app: &AppHandle) -> anyhow::Result<()> {
	start_again(app, &[], &["--args", launch::WINDOW])
}

/// Windows asks for no grant, so its window never offers a restart.
///
/// # Errors
/// Always: only the macOS helper restarts itself.
#[cfg(not(target_os = "macos"))]
pub fn restart(_app: &AppHandle) -> anyhow::Result<()> {
	anyhow::bail!("only the macOS helper restarts itself")
}

/// The new bundle started once this process has gone, and this process exits: in the background with no window
/// (`start_updated`), or, when the player asked for the update (`front`), as the grant card's restart starts it, brought
/// forward, its start opening This computer (`restarted`).
#[cfg(target_os = "macos")]
fn relaunch(app: &AppHandle, front: bool) -> anyhow::Result<()> {
	if front {
		return start_again(app, &[], &["--args", launch::UPDATED]);
	}
	start_updated()?;
	app.exit(0);
	Ok(())
}

/// The helper's bundle started through LaunchServices once this process has gone, in the background and hidden, with
/// no window (`--updated`): an update's relaunch, and a restart asked on the socket, armed in the loop's quit right
/// before the exit (a reinstall moved the new bundle to the same path first, rust-helper step 3).
///
/// # Errors
/// The helper runs from no app bundle, or the shell that starts it again could not be started.
#[cfg(target_os = "macos")]
pub fn start_updated() -> anyhow::Result<()> {
	start_after_exit(&own_bundle()?, &["-g", "-j"], &["--args", launch::UPDATED])
}

/// Windows' helper never starts itself again (the loop refuses a socket's restart there, `starts_again`).
///
/// # Errors
/// Always: only the macOS helper starts itself again.
#[cfg(not(target_os = "macos"))]
pub fn start_updated() -> anyhow::Result<()> {
	anyhow::bail!("only the macOS helper starts itself again")
}

/// Whether a socket's restart can start QuestPump again (`start_updated`): the helper runs from an app bundle, never a
/// bare program (`cargo run`), where the restart is refused instead of only quitting.
#[cfg(target_os = "macos")]
pub fn starts_again() -> bool {
	app_bundle().is_some()
}

/// Windows' helper never starts itself again: a socket's restart is refused.
#[cfg(not(target_os = "macos"))]
pub fn starts_again() -> bool {
	false
}

/// The helper's bundle started through LaunchServices once this process has gone (a child of this process would carry
/// its responsibility, and the Screen Recording and Bluetooth grants belong to the process LaunchServices starts): a
/// shell waits for this process to end, which gives up the single-instance socket and the LAN port on its way out, then
/// asks `open` to start the app, with `open`'s options before the app's path and what follows it after.
#[cfg(target_os = "macos")]
fn start_again(app: &AppHandle, options: &[&str], after: &[&str]) -> anyhow::Result<()> {
	start_after_exit(&own_bundle()?, options, after)?;
	app.exit(0);
	Ok(())
}

/// The helper's own bundle, for a start after this process.
#[cfg(target_os = "macos")]
fn own_bundle() -> anyhow::Result<std::path::PathBuf> {
	app_bundle().ok_or_else(|| anyhow::anyhow!("the helper runs from no app bundle"))
}

/// The shell that waits for this process to end, then starts `bundle` through `open`, each program by its path (`kill`
/// is the shell's own).
#[cfg(target_os = "macos")]
fn start_after_exit(bundle: &std::path::Path, options: &[&str], after: &[&str]) -> anyhow::Result<()> {
	std::process::Command::new("/bin/sh")
		.args(["-c", "while kill -0 \"$0\" 2>/dev/null; do /bin/sleep 0.2; done; exec /usr/bin/open -n \"$@\""])
		.arg(std::process::id().to_string())
		.args(options)
		.arg(bundle)
		.args(after)
		.spawn()?;
	Ok(())
}

/// A quit asked after the player asked for the Screen Recording grant while it is still missing (`desk::menu`'s handler
/// of the quit Apple event, before the loop's quit): System Settings' "Quit & Reopen", which, for this tray app, never
/// opened it again (David, 2026-09-30). The helper starts again as the grant card's restart does, without a window
/// (decided for slice 35: a start with `--reopened` opens none); that start ends at once unless the grant now holds
/// (`desk::launch::REOPENED`), so another quit then (the Dock's, Activity Monitor's) stays a quit, and the system's own
/// (a logout) never starts it again.
///
/// # Errors
/// The helper runs from no app bundle, or the shell that starts it again could not be started.
#[cfg(target_os = "macos")]
pub fn reopen_after_quit() -> anyhow::Result<()> {
	start_after_exit(&own_bundle()?, &[], &["--args", launch::REOPENED])
}

/// The copy the player moved into Applications (`moved_copy`) started through LaunchServices once this process, which
/// runs from its download, has gone through the loop's quit (armed there, right before the exit): a player's launch of
/// it, whose window opens.
///
/// # Errors
/// The shell that starts it once this process has gone could not be started.
#[cfg(target_os = "macos")]
pub fn start_moved(bundle: &std::path::Path) -> anyhow::Result<()> {
	start_after_exit(bundle, &[], &[])
}

/// Windows' installer puts the helper where it runs: no copy is ever moved (`moved_copy` is the Mac's alone).
#[cfg(not(target_os = "macos"))]
#[expect(clippy::unnecessary_wraps, reason = "the same signature as the Mac's start of the moved copy")]
pub fn start_moved(_bundle: &std::path::Path) -> anyhow::Result<()> {
	Ok(())
}

/// Windows' installer ended this process inside `install`; it never gets here.
#[cfg(not(target_os = "macos"))]
#[expect(clippy::unnecessary_wraps, reason = "the same signature as the macOS relaunch")]
fn relaunch(_app: &AppHandle, _front: bool) -> anyhow::Result<()> {
	Ok(())
}
