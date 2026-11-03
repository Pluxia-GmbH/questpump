//! The helper's updates (rust-helper `tauri-shell.md`, "The updater"): `tauri-plugin-updater` reads the release feed
//! (`plugins.updater.endpoints`, the public repository's permanent `helper-updater` release) a minute after the start
//! and every 6 hours, downloads a newer helper and verifies its minisign signature against `plugins.updater.pubkey`
//! (the signed version must be the feed's, `requireSignedVersion`); an empty key refuses every download. A release
//! already downloaded is not fetched again, and no request waits more than 10 minutes. The loop installs it once the
//! game and the main window are closed, no pump runs and no watched game is in a match (`runtime/updates.rs`). Only a
//! published build running from its installed bundle updates itself: a development build would be replaced by a
//! release, from `target/debug` the updater would treat that folder as the app, and a Mac helper still in its disk
//! image cannot be replaced (`unmoved`). The new helper starts with `--updated` on both systems, so it opens no window
//! (`desk::launch`).

use std::time::Duration;

use tauri::AppHandle;

use crate::desk::launch;
use crate::ui::Build;
#[cfg(windows)]
use tauri::utils::{config::BundleType, platform::bundle_type};
use tauri_plugin_updater::{Update, UpdaterExt as _};
use tokio::sync::mpsc::UnboundedSender;

/// The first check, once the start's own work is done.
const FIRST: Duration = Duration::from_mins(1);
const EVERY: Duration = Duration::from_hours(6);
/// The longest a check or a download may take (the plugin sets none): a stalled one ends, and the next check comes.
const TIMEOUT: Duration = Duration::from_mins(10);

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

/// A downloaded update whose signature checked out.
pub struct Ready {
	update: Update,
	bytes: Vec<u8>,
}

impl Ready {
	pub fn version(&self) -> &str {
		&self.update.version
	}
}

/// What a check found.
pub enum Found {
	/// Boxed: the plugin's update carries its whole manifest.
	Ready(Box<Ready>),
	Current,
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

/// Checks for an update in the background and reports each check; a newer one found later replaces a pending one, and
/// the one already handed over is not downloaded or reported again (the loop keeps it, or has shown why it cannot
/// install it).
pub fn spawn(app: AppHandle, found: UnboundedSender<Found>) {
	if !installed() {
		return;
	}
	tauri::async_runtime::spawn(async move {
		tokio::time::sleep(FIRST).await;
		let mut handed: Option<String> = None;
		loop {
			let news = check(&app, handed.as_deref()).await;
			match &news {
				Some(Found::Ready(ready)) => handed = Some(ready.version().to_owned()),
				Some(Found::Current) => handed = None,
				_ => {}
			}
			if let Some(news) = news
				&& found.send(news).is_err()
			{
				return; // the loop has gone: the helper quits
			}
			tokio::time::sleep(EVERY).await;
		}
	});
}

/// One check; none when it found the release already handed over.
async fn check(app: &AppHandle, handed: Option<&str>) -> Option<Found> {
	// Windows: the installer starts the new helper with what follows its /ARGS, which runs to its next option, so the
	// installer's own argument after the plugin's /ARGS reaches the helper (macOS relaunches it with `--updated` itself)
	let checked = match app.updater_builder().timeout(TIMEOUT).installer_arg(launch::UPDATED).build() {
		Ok(updater) => updater.check().await,
		Err(e) => Err(e),
	};
	Some(match checked {
		Ok(None) => Found::Current,
		Ok(Some(update)) if handed == Some(update.version.as_str()) => return None,
		Ok(Some(update)) => match update.download(|_, _| {}, || {}).await {
			Ok(bytes) => Found::Ready(Box::new(Ready { update, bytes })),
			Err(e) => Found::Failed(format!("{}: {e}", update.version)),
		},
		Err(e) => Found::Failed(e.to_string()),
	})
}

/// Installs a downloaded update and starts the new helper. On Windows the installer takes over and ends this process
/// inside `install` (passive, then it starts the helper again); on macOS the bundle is replaced in place and the new
/// one is started as below.
///
/// # Errors
/// The update could not be installed, or the new helper could not be started.
pub fn install(app: &AppHandle, ready: &Ready) -> anyhow::Result<()> {
	ready.update.install(&ready.bytes)?;
	relaunch(app)
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

/// The new bundle started once this process has gone (`start_updated`), and this process exits.
#[cfg(target_os = "macos")]
fn relaunch(app: &AppHandle) -> anyhow::Result<()> {
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
fn relaunch(_app: &AppHandle) -> anyhow::Result<()> {
	Ok(())
}
