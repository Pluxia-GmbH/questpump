//! The Games tab's games (docs/plan.md slice 56, designs 3 to 5): each game as its tile shows it, in the tab's order
//! (running, then on, then off, by name inside each), the named games of this system not on this computer under More
//! games, the found games put away ("Not a game"), whether Riot's line shows and the tab's dot, the games watched in
//! that order (the tray's state, What QuestPump does, the test offer) and the first of them running (the Now card's,
//! the pause's "while it runs"). A named game is a tile while it is on this computer, installed here or running, never
//! for its switch alone (2026-10-27); its name is the page's (`links.name.<key>`), a found or picked game's its own.
//! Its icon is its file's asset URL (`icons.rs`), gathered at each look (`icons_look`).

use std::collections::{HashMap, HashSet};

use questpump_core::games::Watched;
use serde::Serialize;

use super::icons::Source;
use super::procs::Snapshot;
use super::{Games, Status, fed, hears, watches};
use crate::desk::recent::WOW;
use crate::settings::GamesSettings;

/// What a tile is: a named game (contract/links.json's), a found game (`detect.rs`), a program the player added.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
	Named,
	Found,
	Added,
}

/// When a game's offer comes: at each match's end its feed or its process tells, or when the game closes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Ends {
	Match,
	Close,
}

/// One game on the Games tab.
#[derive(Debug, Clone, Serialize)]
#[expect(
	clippy::struct_excessive_bools,
	// copy-lint: not shown (a lint's reason)
	reason = "the facts the tile shows, each a yes or no of its own"
)]
pub struct Tile {
	pub key: String,
	pub kind: Kind,
	/// A found or picked game's name; none for a named game, which the page names.
	pub name: Option<String>,
	/// Its icon's asset URL, while its file stands.
	pub icon: Option<String>,
	/// Watched: the game window's switch ("Offer a pump after each match" or "when you close" it, by `ends`) on.
	pub on: bool,
	/// A Riot game (Riot's line).
	pub riot: bool,
	/// Its process runs ("Playing").
	pub running: bool,
	/// Whether it asks after each match or when it closes, as it is set up now.
	pub ends: Ends,
	/// A game whose feed needs a file of QuestPump's that is not set up: "Ask after each match" stands.
	pub setup: bool,
	/// A step the player owes while it is on (`Status::owed`), else none.
	pub status: Option<Status>,
	/// The loopback port its feed takes, for a game whose port the player sets.
	pub port: Option<u16>,
	/// For a game marked `aim`, whether "Rest your arms while it runs" is on; none for the others.
	pub aim: Option<bool>,
}

/// A found game the player put away: its key and name.
#[derive(Debug, Clone, Serialize)]
pub struct Hidden {
	pub key: String,
	pub name: String,
}

/// The Games tab's lists.
#[derive(Debug, Clone, Serialize)]
pub struct Lists {
	pub tiles: Vec<Tile>,
	pub more: Vec<Tile>,
	pub hidden: Vec<Hidden>,
	/// A Riot game is listed (a tile or under More games): Riot's line shows.
	pub riot: bool,
	/// The tab's dot: a game found since the tab was last opened.
	pub dot: bool,
}

impl Lists {
	/// The games watched, as the tab lists them (the on tiles in its order): the tray's state names them, What
	/// QuestPump does on this computer lists them, and the test offer names the first.
	pub fn watched(&self) -> Vec<Watched> {
		self.tiles
			.iter()
			.filter(|t| t.on)
			.filter_map(|t| match t.kind {
				Kind::Named => Watched::from_key(&t.key),
				Kind::Found | Kind::Added => Some(Watched::Program { key: t.key.clone(), name: t.name.clone()? }),
			})
			.collect()
	}

	/// The first game running in the tab's order, of those watched (one turned off runs on only until the next look):
	/// the Now card's, and the one "While <game> runs" names (the tab's Pause offers and the tray's) and that pause
	/// holds for, so each names the same game.
	pub fn playing(&self) -> Option<&Tile> {
		self.tiles.iter().find(|t| t.on && t.running)
	}
}

impl Tile {
	/// Its place in the tab's order: running, on, off.
	fn group(&self) -> u8 {
		if self.running {
			0
		} else if self.on {
			1
		} else {
			2
		}
	}
}

impl Games {
	/// A named game's tile.
	fn named_tile(&self, w: &Watched, settings: &GamesSettings) -> Tile {
		let key = w.link();
		let on = watches(settings, w);
		let heard = hears(settings, w);
		let status = self.tracks.get(key).map(|t| t.status).filter(|s| on && heard && s.owed());
		Tile {
			key: key.to_owned(),
			kind: Kind::Named,
			name: None,
			icon: self.icons.url(key),
			on,
			riot: w.riot(),
			running: self.running.iter().any(|r| r == key),
			ends: if heard || !w.writes_file() { Ends::Match } else { Ends::Close },
			setup: w.writes_file() && !settings.is_asked(key),
			status,
			port: fed::port_of(settings, w),
			aim: w.aim().then(|| settings.aim_on(key)),
		}
	}

	/// A found or picked game's tile.
	fn program_tile(&self, key: &str, name: &str, kind: Kind, settings: &GamesSettings) -> Tile {
		Tile {
			key: key.to_owned(),
			kind,
			name: Some(name.to_owned()),
			icon: self.icons.url(key),
			on: !settings.is_off(key),
			riot: false,
			running: self.running.iter().any(|r| r == key),
			ends: Ends::Close,
			setup: false,
			status: None,
			port: None,
			aim: None,
		}
	}

	/// The Games tab's games as they stand.
	pub fn lists(&self, settings: &GamesSettings) -> Lists {
		let (listed, more): (Vec<Watched>, Vec<Watched>) = self.named().partition(|w| self.listed(w));
		let found = settings.detected.iter().map(|d| self.program_tile(&d.key, &d.name, Kind::Found, settings));
		let picked = settings.programs.iter().filter(|p| settings.detected.iter().all(|d| d.key != p.key));
		let added = picked.map(|p| self.program_tile(&p.key, &p.name, Kind::Added, settings));
		let mut tiles: Vec<Tile> =
			listed.iter().map(|w| self.named_tile(w, settings)).chain(found).chain(added).collect();
		let name = |tile: &Tile| {
			let named = || Watched::from_key(&tile.key).map(|w| w.name().to_lowercase());
			tile.name.as_deref().map(str::to_lowercase).or_else(named).unwrap_or_else(|| tile.key.clone())
		};
		tiles.sort_by_cached_key(|tile| (tile.group(), name(tile)));
		// a game under More games is watched as any named game is, and is a tile once it is found or runs, never for
		// its switch
		let mut more: Vec<Tile> = more.iter().map(|w| self.named_tile(w, settings)).collect();
		more.sort_by_cached_key(name);
		let hidden = settings.dismissed.iter().map(|d| Hidden { key: d.key.clone(), name: d.name.clone() }).collect();
		let riot = tiles.iter().chain(&more).any(|t| t.riot);
		Lists { tiles, more, hidden, riot, dot: dot(settings) }
	}

	/// The games watched, as the tab lists them (`Lists::watched`).
	pub fn watched(&self, settings: &GamesSettings) -> Vec<Watched> {
		self.lists(settings).watched()
	}

	/// The icons at a look: each watched game running gives the program its process runs from, the folders of the
	/// named games installed and the records of the found and picked games give the others, and the icons not made
	/// yet in this run are asked for (`Icons::look`; on a volume under `/Volumes` only a game's the player asked for).
	/// World of Warcraft's too while Pump home's recent games may show it (`set_wow`): its running client's program,
	/// else its install folder's.
	pub(super) fn icons_look(&mut self, snap: &Snapshot, settings: &GamesSettings) {
		for id in self.running.clone() {
			let named = Watched::from_key(&id);
			for p in &snap.procs {
				let of = match &named {
					Some(w) => Self::is_process_of(w, p),
					None => self.is_program(&id, p),
				};
				let source = if of { self.detector.icon_seen(&id, p) } else { None };
				if let Some(source) = source {
					self.icons.saw(&id, source);
				}
			}
		}
		// World of Warcraft (Pump home's recent games): its running client's program, else one in its install's folder
		let wow_proc = self.wow.as_ref().and_then(|w| w.pid).and_then(|pid| snap.procs.iter().find(|p| p.pid == pid));
		if let Some(source) = wow_proc.and_then(|p| wow_source(p.path.as_deref()?)) {
			self.icons.saw(WOW, source);
		}
		let listed: HashSet<String> = self
			.named()
			.map(|w| w.link().to_owned())
			.chain(settings.detected.iter().chain(&settings.dismissed).map(|d| d.key.clone()))
			.chain(settings.programs.iter().map(|p| p.key.clone()))
			.chain(self.wow.is_some().then(|| WOW.to_owned()))
			.collect();
		let sources: HashMap<String, Source> = self
			.icons
			.wanted(&listed)
			.filter_map(|key| {
				let source = match Watched::from_key(key) {
					Some(w) => self.dirs.get(w.link()).map(|dir| Source::Folder {
						dir: dir.clone(),
						images: w.windows_images().iter().map(|&i| i.to_owned()).collect(),
					}),
					None if key == WOW => self.wow_folder(),
					None => self.detector.icon_source(key),
				};
				Some((key.clone(), source?))
			})
			.collect();
		self.icons.look(&listed, &sources, &settings.asked);
	}

	/// World of Warcraft's install folder as its icon's source, searched for its client's program.
	fn wow_folder(&self) -> Option<Source> {
		let dir = self.wow.as_ref()?.root.clone()?;
		Some(Source::Folder { dir, images: WOW_IMAGES.iter().map(|&i| i.to_owned()).collect() })
	}
}

/// World of Warcraft's client executables an install folder is searched for on Windows (`game::is_client_name`'s,
/// WowUp's list): its running client's program comes first, whatever its name.
const WOW_IMAGES: [&str; 12] = [
	"Wow.exe",
	"WowB.exe",
	"WowT.exe",
	"WowClassic.exe",
	"WowClassicB.exe",
	"WowClassicT.exe",
	"Wow-arm64.exe",
	"WowB-arm64.exe",
	"WowT-arm64.exe",
	"WowClassic-arm64.exe",
	"WowClassicB-arm64.exe",
	"WowClassicT-arm64.exe",
];

/// The icon's source of a World of Warcraft client running from `path`: its app bundle on macOS, its executable on
/// Windows.
fn wow_source(path: &std::path::Path) -> Option<Source> {
	if cfg!(windows) { Some(Source::File(path.to_path_buf())) } else { super::icons::bundle_source(path) }
}

/// The tab's dot: a game found since the tab was last opened (any found game before its first open).
fn dot(settings: &GamesSettings) -> bool {
	let since = settings.seen_at.unwrap_or(i64::MIN);
	settings.detected.iter().any(|d| d.seen > since)
}
