//! Backup on the computer (docs/ui.md, "The Menu on the computer"; the phones' Backup page, slice 17): save a backup
//! file of this computer's book and open one, a phone's or a computer's (`questpump_core::backup`). Rust makes the
//! file, shows the system's save or open dialog on the main window (the dialog plugin, called from Rust) and writes or
//! reads the file off the loop, then hands the outcome back to the loop (`DeskAsk::Backup`). A file comes in as the
//! phones restore one: its app rows as a union with the ones the computer keeps (each once by its text and copy, so a
//! second time changes nothing), then into every mirror, every paired phone's copy and the engine; the game's rows and
//! sheets come back from the game's own save, never from a file; a profile only from a file of this computer's own and
//! only into a computer whose place is still the default (a phone's place is that phone's); the body never (its
//! exchange is the body's way back, and a restore never counts as a change of it).
//!
//! **The `menu.backup` topic**: `{last: unix ms | null, busy, pump_on, outcome: {seq, state: "restored", rows,
//! settings, kept} | {seq, state: "refused", why: "newer"|"not_backup"|"too_large"|"unreadable"|"not_saved"|
//! "not_kept"|"pump_on"} | null}`: `not_kept` when the computer's store of app rows could not take a row of the file
//! (the rest of it stays in, and the same file again takes in what is missing), `pump_on` when a pump started while
//! the open dialog stood.
//!
//! **Intents**: `backup.look` (the topic again: the page's mount), `backup.save`, `backup.open` (neither while one is
//! under way; no file comes in while a pump runs, also when one starts while the dialog stands).

use std::path::Path;

use questpump_core::backup::{self, File, MAX_BYTES, Refusal};
use questpump_core::body::PlaceSetup;
use serde::Serialize;
use tauri::{AppHandle, Manager as _};
use tauri_plugin_dialog::DialogExt as _;

use super::BACKUP;
use crate::copy;
use crate::desk::facts::facts;
use crate::desk::windows::MAIN;
use crate::desk::{DeskAsk, Intent};
use crate::lan;
use crate::runtime::{Helper, local_offset_at, now};
use crate::ui::{Shared, WindowAsk};

/// What became of the dialog the page asked for, handed back to the loop.
#[derive(Debug)]
pub(crate) enum Picked {
	/// The player closed the dialog.
	Cancelled,
	/// The file was written, or why it was not.
	Saved(Result<(), String>),
	/// The chosen file's bytes, or why they were not read.
	Opened(Result<Vec<u8>, Why>),
}

/// Why nothing, or not all, came of a file.
#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Why {
	Newer,
	NotBackup,
	TooLarge,
	Unreadable,
	NotSaved,
	/// The store of app rows did not take every row of the file (`AppTaken::unwritten`).
	NotKept,
	/// A pump started on this computer while the open dialog stood.
	PumpOn,
}

/// The last outcome, said once per `seq`.
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "state", rename_all = "snake_case")]
enum Outcome {
	/// A file came in: the rows it brought, whether it brought this computer's place, and whether the computer kept a
	/// place of its own over the file's.
	Restored {
		seq: u64,
		rows: usize,
		settings: bool,
		kept: bool,
	},
	Refused {
		seq: u64,
		why: Why,
	},
}

/// The page's state between the intent and the dialog's answer.
#[derive(Default)]
pub(crate) struct State {
	busy: bool,
	outcome: Option<Outcome>,
	seq: u64,
}

impl State {
	fn said(&mut self, outcome: impl FnOnce(u64) -> Outcome) {
		self.seq += 1;
		self.outcome = Some(outcome(self.seq));
	}
}

#[derive(Serialize)]
struct Topic<'a> {
	last: Option<i64>,
	busy: bool,
	pump_on: bool,
	outcome: Option<&'a Outcome>,
}

/// Hands the dialog's outcome back to the loop.
fn back(app: &AppHandle, picked: Picked) {
	if !app.state::<Shared>().ask(WindowAsk::Desk(DeskAsk::Backup(picked))) {
		log::warn!("backup: the loop does not run any more");
	}
}

/// Today's date on this computer as the file's name gives it (`2026-10-08`).
fn today() -> String {
	let t = time::OffsetDateTime::now_utc();
	let offset = time::UtcOffset::from_whole_seconds(i32::try_from(local_offset_at(t.unix_timestamp())).unwrap_or(0))
		.unwrap_or(time::UtcOffset::UTC);
	let d = t.to_offset(offset).date();
	format!("{:04}-{:02}-{:02}", d.year(), u8::from(d.month()), d.day())
}

/// The chosen file's bytes, refused past `MAX_BYTES` before they are read.
fn read(path: &Path) -> Result<Vec<u8>, Why> {
	let size = std::fs::metadata(path).map_err(|_| Why::Unreadable)?.len();
	if size > MAX_BYTES {
		return Err(Why::TooLarge);
	}
	let bytes = std::fs::read(path).map_err(|_| Why::Unreadable)?;
	if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > MAX_BYTES {
		return Err(Why::TooLarge);
	}
	Ok(bytes)
}

impl Helper {
	/// The `backup` area's intents.
	pub(crate) fn backup_intent(&mut self, intent: &Intent) -> Result<(), String> {
		let kind = intent.kind();
		if self.menu.backup.busy && kind != "backup.look" {
			return Err(format!("{kind}: a backup file is under way"));
		}
		match kind {
			"backup.look" => self.publish_backup(),
			"backup.save" => self.backup_save(),
			"backup.open" => {
				if self.desk_pump_running() {
					return Err("backup.open: a pump runs".to_owned());
				}
				self.backup_open();
			}
			_ => return Err(intent.unknown()),
		}
		Ok(())
	}

	/// `menu.backup` as it stands.
	pub(crate) fn publish_backup(&self) {
		let state = &self.menu.backup;
		let topic = Topic {
			last: self.settings.desk.backup_at,
			busy: state.busy,
			pump_on: self.desk_pump_running(),
			outcome: state.outcome.as_ref(),
		};
		self.publish(BACKUP, &topic);
	}

	/// This computer's book as a file, then the save dialog: the file is written where the player picks.
	fn backup_save(&mut self) {
		let people = self.session.people();
		let game: Vec<_> = self
			.session
			.store()
			.records()
			.iter()
			.map(|r| {
				let mut r = r.clone();
				r.char = people.resolve(r.char);
				r
			})
			.collect();
		// the file keeps Home's setup, never the place in use: a restore gives it back to Home alone (slice 51)
		let mut profile = self.engine_profile();
		let home = self.home_setup();
		(profile.gear, profile.floor, profile.space, profile.quiet) = (home.gear, home.floor, home.space, home.quiet);
		let profile = serde_json::to_value(profile).unwrap_or_default();
		let file = backup::book(
			now().unix,
			self.session.app_rows().records(),
			&game,
			self.session.sheet_log().records(),
			profile,
		);
		let bytes = match backup::encode(&file) {
			Ok(bytes) => bytes,
			Err(e) => {
				log::warn!("backup: the file did not write: {e}");
				self.menu.backup.said(|seq| Outcome::Refused { seq, why: Why::NotSaved });
				self.publish_backup();
				return;
			}
		};
		self.menu.backup.busy = true;
		self.publish_backup();
		let name = format!("{}.json", copy::menu_backup_filename(&today()));
		let handle = self.app.clone();
		let mut dialog = self
			.app
			.dialog()
			.file()
			.set_title(copy::menu_backup_save())
			.add_filter("JSON", &["json"])
			.set_file_name(name);
		if let Some(window) = self.app.get_webview_window(MAIN) {
			dialog = dialog.set_parent(&window);
		}
		dialog.save_file(move |picked| {
			let outcome = match picked.and_then(|p| p.into_path().ok()) {
				None => Picked::Cancelled,
				Some(path) => {
					Picked::Saved(std::fs::write(&path, &bytes).map_err(|e| format!("{}: {e}", path.display())))
				}
			};
			back(&handle, outcome);
		});
	}

	/// The open dialog: the chosen file is read off the loop and comes back as `DeskAsk::Backup`.
	fn backup_open(&mut self) {
		self.menu.backup.busy = true;
		self.publish_backup();
		let handle = self.app.clone();
		let mut dialog = self.app.dialog().file().set_title(copy::menu_backup_restore()).add_filter("JSON", &["json"]);
		if let Some(window) = self.app.get_webview_window(MAIN) {
			dialog = dialog.set_parent(&window);
		}
		dialog.pick_file(move |picked| {
			let outcome = match picked.and_then(|p| p.into_path().ok()) {
				None => Picked::Cancelled,
				Some(path) => Picked::Opened(read(&path)),
			};
			back(&handle, outcome);
		});
	}

	/// The dialog's outcome: a file written, a file read and taken in, or nothing.
	pub(crate) fn on_backup(&mut self, picked: Picked) {
		let state = &mut self.menu.backup;
		state.busy = false;
		match picked {
			Picked::Cancelled => {}
			Picked::Saved(Ok(())) => {
				log::info!("backup: a backup file saved");
				state.outcome = None;
				self.settings.desk.backup_at = Some(lan::now_ms());
				self.save_settings();
			}
			Picked::Saved(Err(e)) => {
				log::warn!("backup: {e}");
				state.said(|seq| Outcome::Refused { seq, why: Why::NotSaved });
			}
			Picked::Opened(Err(why)) => state.said(|seq| Outcome::Refused { seq, why }),
			Picked::Opened(Ok(_)) if self.desk_pump_running() => {
				log::info!("backup: a file picked while a pump runs, not taken in");
				self.menu.backup.said(|seq| Outcome::Refused { seq, why: Why::PumpOn });
			}
			Picked::Opened(Ok(bytes)) => match backup::decode(&bytes) {
				Ok(file) => self.restore(&file),
				Err(refusal) => {
					let why = match refusal {
						Refusal::Newer => Why::Newer,
						Refusal::NotBackup => Why::NotBackup,
					};
					log::info!("backup: a file refused: {refusal}");
					self.menu.backup.said(|seq| Outcome::Refused { seq, why });
				}
			},
		}
		self.publish_backup();
	}

	/// A backup file taken in: its app rows as a union, and Home's setup from a file of this computer's while Home has
	/// none of its own yet; a file whose rows the store could not all take says so, never "nothing new".
	fn restore(&mut self, file: &File) {
		let rows = file.app_rows();
		let (taken, fx) = self.session.restore_app_rows(file.sender(), &rows, &now());
		self.apply(fx);
		let (mut settings, mut kept) = (false, false);
		if let Some(place) = (file.device == backup::COMPUTER).then(|| file_place(file)).flatten()
			&& !place.same(&self.home_setup())
		{
			// none of its own yet: no device ever set Home, and this computer's own setup is still the default
			if self.body.body.home.is_none() && self.home_setup() == PlaceSetup::default() {
				self.set_home(place);
				settings = true;
			} else {
				kept = true;
			}
		}
		log::info!(
			"backup: a file of {} rows from {} taken in: {} new, {} dropped",
			rows.len(),
			file.device,
			taken.kept,
			taken.dropped
		);
		if taken.unwritten {
			log::warn!("backup: the store of app rows did not take every row of the file");
			self.menu.backup.said(|seq| Outcome::Refused { seq, why: Why::NotKept });
		} else {
			self.menu.backup.said(|seq| Outcome::Restored { seq, rows: taken.kept, settings, kept });
		}
	}
}

/// The place a computer's file carries in its profile (the engine's, whose `gear`, `floor`, `space` and `quiet` are
/// the place); none when it carries none that reads.
fn file_place(file: &File) -> Option<PlaceSetup> {
	let profile = file.profile.as_ref()?;
	let place = PlaceSetup {
		gear: serde_json::from_value(profile.get("gear")?.clone()).ok()?,
		floor: profile.get("floor")?.as_bool()?,
		space: profile.get("space")?.as_bool()?,
		quiet: profile.get("quiet")?.as_bool()?,
	};
	// equipment the sheet does not name is no equipment of this build's
	Some(PlaceSetup {
		gear: place.gear.into_iter().filter(|k| facts().sheet.gear_order.contains(k)).collect(),
		..place
	})
}
