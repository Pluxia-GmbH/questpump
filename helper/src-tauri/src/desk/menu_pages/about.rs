//! QuestPump's group of the Menu on the computer (docs/ui.md, "The Menu on the computer" and "Pro on the computer"):
//! Safety and about and the safety note (the desk's, `questpump_desk::menu_pages::about`, which the browser shows too),
//! How the heart-rate check works (the verdict rule in words, its numbers `questpump_core::verdict`'s own, in the same
//! topic), with the desktop app's own parts: what QuestPump does and does not do on this computer, the
//! Acknowledgements (`contract/design/acknowledgements.json`: the fonts and the sounds; the code the app builds in: the
//! engine's `NOTICES` (Lua 5.1.5, mlua and lua-src), the sound's (`desk::audio::NOTICES`: Symphonia under the MPL 2.0)
//! and every Rust crate's licence texts (`helper/notices/crates.txt`, written by `helper/tools/notices.py`)), and the
//! Pro row's state (`Helper::pro_state`); Pro's page has its own topic, `menu.pro` (`runtime/pro.rs`). No price on the
//! computer.
//!
//! **The `menu.about` topic**: `{version, pro: "on"|"off"|"refused"|null, safety, contact, heart: {before, window,
//! reps, hold, after}, does: {data, mirror: path|null, games: [name], files: [path], added: [path], lan, gsi:
//! port|null, client: [name], logs: [name], udp: [name], shared: [name], saves: [name], detect, steam: [name]}}`: `pro`
//! none without a paired phone, `refused` while a paired phone's last proof could not be confirmed here; `heart` the
//! verdict rule's numbers How the heart-rate check works names (whole seconds and beats a minute); `does` what this
//! computer runs and keeps (its data folder, the progress it writes for the game, the games it watches by name, as the
//! Games tab lists them, the files it wrote into their folders and the shared files it added its part to, the LAN API's
//! port for the phones and the games' listener's on 127.0.0.1, and by name the named games of those it asks on the
//! loopback, whose logs it reads, whose UDP telemetry it hears, whose shared memory it reads and whose saved runs it
//! notices (2026-10-28), so a game not on this computer or not watched is never named, and whether it notices games
//! ("Notice games I play"): slice 48; while it does not, the Steam games watched, which Steam's own records still tell:
//! the second fix round). A game a shipped build leaves out (contract/links.json's `pending`, `Watched::named`) is
//! never named.
//!
//! **The `menu.acknowledgements` topic**: `{fonts, assets, licences, engine, audio, crates}`, the first three as
//! `acknowledgements.json` holds them, the others the notices' texts.
//!
//! **Intents**: `about.look` (the topic again: the page's mount), `about.open {page:
//! "privacy"|"telemetry"|"sync"|"imprint"|"security"}` or `about.open {asset: n}` (the site's page, or the n-th
//! credited sound pack's, in the system's browser).

use std::path::PathBuf;

use questpump_core::games::{Watched, computer_links};
use questpump_core::pair;
use questpump_desk::menu_pages::about::{About, CREDITS, page_url};
use serde::Serialize;
use serde_json::Value;

use super::ACKNOWLEDGEMENTS;
use crate::desk::Intent;
use crate::runtime::Helper;

/// The licence texts of every Rust crate the app builds in, grouped by text, as `helper/tools/notices.py` writes them
/// from Cargo.lock (its `--check` fails in CI while they are stale).
const CRATES: &str = include_str!("../../../../notices/crates.txt");

/// What QuestPump does on this computer.
#[derive(Serialize)]
struct Does {
	/// Its own folder: the settings, the rows, the body, the grants.
	data: String,
	/// The folder of the progress it keeps for the game (`QuestPump_Mirror` beside the addon), once a game folder is
	/// known.
	mirror: Option<String>,
	/// The games it watches for a match's end or their close, by name, as the Games tab lists them (`Games::watched`).
	games: Vec<String>,
	/// The files it wrote into those games' folders (turning a game off removes its file).
	files: Vec<String>,
	/// The shared files it added its part to (Hearthstone's `log.config`), which stay when the game is turned off.
	added: Vec<String>,
	/// The LAN API's port, where the paired phones and watches reach it.
	lan: u16,
	/// The games' listener's port on 127.0.0.1, while a game that reports through it is on.
	gsi: Option<u16>,
	/// The named games of `games` by how each is heard (slice 48), by name: asked on the loopback, by their logs, by
	/// their UDP telemetry, by their shared memory (a game whose feed needs a file of QuestPump's only once it is set
	/// up), by the runs they save (2026-10-28: Slay the Spire 2's history, whose files are never opened), by their
	/// window's boss banners (2026-10-29: ELDEN RING's and NIGHTREIGN's, Windows alone).
	client: Vec<String>,
	logs: Vec<String>,
	udp: Vec<String>,
	shared: Vec<String>,
	saves: Vec<String>,
	screen: Vec<String>,
	/// It notices the games that run (the Games page's "Notice games I play", on by default).
	detect: bool,
	/// While it does not, the Steam games watched (`steam:<app>`) by name: Steam's own records still tell them
	/// (`games/detect.rs`'s `steam_records`).
	steam: Vec<String>,
}

#[derive(Serialize)]
struct Credits<'a> {
	fonts: &'a Value,
	assets: &'a Value,
	licences: &'a Value,
	engine: &'static str,
	audio: &'static str,
	crates: &'static str,
}

/// Opens a page in the system's browser (also Pro's page's Get Pro and Manage Pro, `runtime/pro.rs`), and on macOS a
/// folder in the Finder or a System Settings pane, whatever the system opens it with (`/usr/bin/open` by its path,
/// never a name looked up on the search path).
pub(crate) fn open_url(url: &str) -> Result<(), String> {
	#[cfg(target_os = "macos")]
	{
		std::process::Command::new("/usr/bin/open").arg(url).spawn().map(|_| ()).map_err(|e| e.to_string())
	}
	#[cfg(windows)]
	{
		use windows::Win32::UI::Shell::ShellExecuteW;
		use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
		use windows::core::{HSTRING, w};
		let _com = crate::elevate::Com::init().map_err(|e| e.to_string())?;
		// SAFETY: the operation and the address are valid wide strings for the call's length; no window, no
		// parameters, no folder. ShellExecuteW returns a value above 32 when it succeeded.
		let done = unsafe { ShellExecuteW(None, w!("open"), &HSTRING::from(url), None, None, SW_SHOWNORMAL) };
		if done.0 as usize > 32 { Ok(()) } else { Err(format!("ShellExecuteW: {}", done.0 as usize)) }
	}
}

impl Helper {
	/// The `about` area's intents.
	pub(crate) fn about_intent(&mut self, intent: &Intent) -> Result<(), String> {
		match intent.kind() {
			"about.look" => self.publish_about(),
			"about.open" => open_url(page_url(intent)?)?,
			_ => return Err(intent.unknown()),
		}
		Ok(())
	}

	/// `menu.about` as it stands: at the start, when Pro's state changes (`runtime/pro.rs`) and at the page's look.
	pub(crate) fn publish_about(&self) {
		let games = &self.desk.settings.games;
		let watched = self.desk.p.games.as_ref().map(|g| g.watched(games)).unwrap_or_default();
		let named = watched.iter().map(|w| w.name().to_owned()).collect();
		let path = |p: &PathBuf| p.display().to_string();
		// the named games watched (the Games tab's on tiles) whose feed QuestPump hears, by how it hears them (its
		// contract/links.json trigger: gsi, client, log, udp, shared, files, screen); a found or added game is heard at
		// its close alone
		let links = computer_links().unwrap_or_default();
		let heard_by = |trigger: &str| -> Vec<String> {
			watched
				.iter()
				.filter(|w| !matches!(w, Watched::Program { .. }) && crate::games::hears(games, w))
				.filter(|w| links.iter().any(|l| l.key == w.link() && l.trigger == trigger))
				.map(|w| w.name().to_owned())
				.collect()
		};
		let does = Does {
			data: path(&self.dir),
			mirror: self.desk.p.followed.as_ref().map(|a| path(&a.join(questpump_core::mirror::FOLDER))),
			games: named,
			files: games.written.iter().filter(|f| !f.shared).map(|f| path(&f.path)).collect(),
			added: games.written.iter().filter(|f| f.shared).map(|f| path(&f.path)).collect(),
			lan: pair::PORT,
			gsi: games.gsi_port.filter(|_| !heard_by("gsi").is_empty()),
			client: heard_by("client"),
			logs: heard_by("log"),
			udp: heard_by("udp"),
			shared: heard_by("shared"),
			saves: heard_by("files"),
			screen: heard_by("screen"),
			detect: !games.notice_off,
			steam: watched
				.iter()
				.filter(|w| games.notice_off && crate::games::Games::is_steam_key(w.id()))
				.map(|w| w.name().to_owned())
				.collect(),
		};
		let topic = About::new(env!("CARGO_PKG_VERSION"), self.pro_state(), Some(does));
		self.publish(questpump_desk::menu_pages::about::ABOUT, &topic);
	}

	/// `menu.acknowledgements`, once at the start: what the app ships with and their makers.
	pub(super) fn publish_acknowledgements(&self) {
		let null = &Value::Null;
		let credits = Credits {
			fonts: CREDITS.get("fonts").unwrap_or(null),
			assets: CREDITS.get("assets").unwrap_or(null),
			licences: CREDITS.get("licences").unwrap_or(null),
			engine: questpump_engine::NOTICES,
			audio: crate::desk::audio::NOTICES,
			crates: CRATES,
		};
		self.publish(ACKNOWLEDGEMENTS, &credits);
	}
}
