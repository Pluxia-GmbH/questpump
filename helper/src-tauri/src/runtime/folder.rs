//! The game folder on This computer: where the mirror goes and how it was found, the folder the player picks, the move
//! figures' pack beside QuestPump (`docs/plan.md` slice 16: at most This computer says the pack is missing or from
//! another download; the helper has no other part in it), and on Windows the card for a folder the helper may not
//! write into (rust-helper `windows-capture-and-install.md`, "The game folder's ACL and Controlled Folder Access"): a
//! refused write (os error 5) offers the one-time elevated grant (`elevate::allow_folder`); a refusal that outlives the
//! grant, or any write error in a folder Controlled folder access protects by default (its blocks surface as "not
//! found" as often as "denied"), is Controlled folder access, which only the player can allow.

use std::path::{Path, PathBuf};
use std::time::Instant;

use super::{Elevated, Helper};
use crate::elevate::{self, Outcome};
use crate::install::{self, Pack};
use crate::ui::{self, AdminNote, Folder, FolderNote};

/// Why Windows refused the mirror write, for the folder it refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Access {
	Fine,
	/// The folder's permissions: This computer offers the elevated grant for this folder.
	Denied(PathBuf),
	/// The grant's UAC prompt is up: another click asks nothing.
	Asking(PathBuf),
	/// The grant was made; the next write shows whether it was enough.
	Granted(PathBuf),
	/// Controlled folder access: This computer says where the player allows the helper.
	Blocked(PathBuf),
}

impl Access {
	/// This computer's card.
	fn card(&self) -> ui::Access {
		match self {
			Self::Denied(_) | Self::Asking(_) => ui::Access::Acl,
			Self::Blocked(_) => ui::Access::Cfa,
			Self::Fine | Self::Granted(_) => ui::Access::Fine,
		}
	}
}

/// Whether Controlled folder access protects the folder by default: under the player's or the public Documents,
/// Pictures, Music, Videos or the player's Favorites (Microsoft Learn, "Protect important folders with controlled
/// folder access").
fn protected(path: &Path) -> bool {
	let public = std::env::var_os("PUBLIC").map(PathBuf::from);
	let mut roots: Vec<PathBuf> = [dirs::document_dir(), dirs::picture_dir(), dirs::audio_dir(), dirs::video_dir()]
		.into_iter()
		.flatten()
		.collect();
	roots.extend(dirs::home_dir().map(|h| h.join("Favorites")));
	// copy-lint: not shown (folder names)
	roots.extend(public.iter().flat_map(|p| ["Documents", "Pictures", "Music", "Videos"].map(|d| p.join(d))));
	roots.iter().any(|r| path.starts_with(r))
}

/// The card after a refused write into `addons`: Controlled folder access for a folder it protects, or when the grant
/// was made for this folder and Windows still says no; the folder's permissions for a first "denied" (in this folder:
/// another folder starts again); unchanged while the grant's prompt is up, and none for other errors.
fn after_refusal(access: &Access, addons: &Path, protected: bool, denied: bool) -> Option<Access> {
	let here = addons.to_path_buf();
	match access {
		_ if protected => Some(Access::Blocked(here)),
		Access::Asking(at) if at == addons => None,
		Access::Granted(at) | Access::Blocked(at) if denied && at == addons => Some(Access::Blocked(here)),
		_ if denied => Some(Access::Denied(here)),
		_ => None,
	}
}

/// The grant step's line on the card for how it came out.
fn grant_note(outcome: &Outcome) -> Option<AdminNote> {
	match outcome {
		Outcome::Done => None,
		Outcome::Declined => Some(AdminNote::Declined),
		Outcome::Failed(e) => Some(AdminNote::Failed { error: e.clone() }),
	}
}

/// The move figures' line on This computer ("Exercise animations").
fn pack_line(addons: &Path) -> ui::Pack {
	let state = install::addons_state(addons);
	if state.questpump.is_none() {
		return ui::Pack::Absent;
	}
	match state.pack {
		Pack::Same => ui::Pack::Installed,
		Pack::Missing => ui::Pack::Missing,
		Pack::Other { pack, addon } => ui::Pack::Other { pack, addon },
	}
}

impl Helper {
	/// The folder and its figures on This computer.
	pub(super) fn show_folder(&mut self, addons: &Path) {
		self.shown_folder = Some(addons.to_path_buf());
		let folder = Folder::Found { path: addons.display().to_string(), label: install::label(addons) };
		let pack = pack_line(addons);
		ui::update(&self.app, |v| {
			v.computer.folder = folder;
			v.computer.pack = pack;
		});
	}

	/// The folder the player picked: remembered, shown, and the mirror written there (after taking in what it holds).
	/// While a game with QuestPump runs its own folder is the one, so a pick then is refused with a line that says so.
	/// A game folder without QuestPump is not followed (no mirror goes where the game cannot read it): its line says
	/// so, and while no install with QuestPump is known it is the game on this computer, followed once QuestPump is in
	/// it (`Helper::addons`).
	pub(super) fn picked(&mut self, path: &Path) {
		if let Some(running) = &self.game_addons {
			let note = FolderNote::Running { path: running.display().to_string() };
			ui::update(&self.app, |v| v.computer.folder_note = Some(note));
			return;
		}
		let Some(addons) = install::from_picked(path) else {
			let note = FolderNote::NoAddons { path: path.display().to_string() };
			ui::update(&self.app, |v| v.computer.folder_note = Some(note));
			return;
		};
		if !install::holds_questpump(&addons) {
			log::info!("the player chose {}, which holds no QuestPump yet", addons.display());
			let note = FolderNote::NoQuestPump { path: addons.display().to_string() };
			ui::update(&self.app, |v| v.computer.folder_note = Some(note));
			if self.followed.is_none() {
				// Add QuestPump to the game, waiting for this pick, goes here
				self.addon_picked(&addons);
				self.without_addon = Some(addons);
				self.without_addon_picked = true;
				self.due_by(Instant::now()); // the next mirror job shows it as the game on this computer
			}
			return;
		}
		log::info!("the player chose {}", addons.display());
		self.settings.addons = Some(addons.clone());
		self.save_settings();
		self.mirror_error = None;
		self.access = Access::Fine; // a new folder: its own card, if any, after its first write
		ui::update(&self.app, |v| {
			v.computer.folder_note = None;
			v.computer.access = ui::Access::Fine;
			v.computer.access_note = None;
		});
		self.show_folder(&addons);
		self.due_by(Instant::now());
	}

	/// A mirror write went through: the card goes away.
	pub(super) fn written_fine(&mut self) {
		if self.access != Access::Fine {
			self.access = Access::Fine;
			ui::update(&self.app, |v| {
				v.computer.access = ui::Access::Fine;
				v.computer.access_note = None;
			});
		}
	}

	/// A mirror write Windows refused: the card for Controlled folder access when the folder is one it protects by
	/// default or the grant did not help, else for the folder's permissions on os error 5 (`ERROR_ACCESS_DENIED`).
	/// Other errors only show in the progress line.
	pub(super) fn refused(&mut self, addons: &Path, e: &std::io::Error) {
		if !cfg!(windows) {
			return;
		}
		let Some(access) = after_refusal(&self.access, addons, protected(addons), e.raw_os_error() == Some(5)) else {
			return;
		};
		self.access = access;
		let card = self.access.card();
		ui::update(&self.app, |v| v.computer.access = card);
	}

	/// This computer's "Allow writing to the game folder": the elevated grant runs off the loop; a second click while
	/// its prompt is up asks nothing.
	pub(super) fn allow_folder(&mut self) {
		let Access::Denied(addons) = &self.access else { return };
		let addons = addons.clone();
		self.access = Access::Asking(addons.clone());
		ui::update(&self.app, |v| v.computer.access_note = Some(AdminNote::Waiting));
		let tx = self.elevated.clone();
		tokio::spawn(async move {
			let outcome = tokio::task::spawn_blocking(move || elevate::allow_folder(&addons))
				.await
				.unwrap_or_else(|e| Outcome::Failed(e.to_string()));
			let _ = tx.send(Elevated::Folder(outcome));
		});
	}

	/// The grant came back: the mirror is written again at once, and the next write tells whether it was enough; a
	/// declined or failed step says so on the card, which offers the step again.
	pub(super) fn folder_allowed(&mut self, outcome: Outcome) {
		let Access::Asking(addons) = &self.access else { return };
		let addons = addons.clone();
		let note = grant_note(&outcome);
		match outcome {
			Outcome::Done => {
				log::info!("folder: the helper may write into {}", addons.display());
				self.access = Access::Granted(addons);
				ui::update(&self.app, |v| v.computer.access = ui::Access::Fine);
				self.due_by(Instant::now());
				self.addon_retry();
			}
			Outcome::Declined => {
				log::info!("folder: the administrator step was declined");
				self.access = Access::Denied(addons);
			}
			Outcome::Failed(e) => {
				log::warn!("folder: the grant could not be made: {e}");
				self.access = Access::Denied(addons);
			}
		}
		ui::update(&self.app, |v| v.computer.access_note = note);
	}
}
