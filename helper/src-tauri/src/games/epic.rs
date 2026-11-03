//! The Epic Games Launcher's manifests (docs/plan.md slice 35; jayd.ml and Epic's `FInstalledApp`): one JSON file a
//! game in `%ProgramData%\Epic\EpicGamesLauncher\Data\Manifests\*.item`, of which the helper reads the install's folder,
//! its launch executable and the game's name. None outside Windows, where the launcher keeps no such folder.

use std::path::PathBuf;

use serde::Deserialize;

/// A manifest's fields the helper reads.
#[derive(Deserialize)]
pub(super) struct Item {
	#[serde(rename = "InstallLocation")]
	pub install_location: Option<String>,
	#[serde(rename = "LaunchExecutable")]
	pub launch_executable: Option<String>,
	/// The game's name as the launcher shows it (the detector's found game, slice 48).
	#[cfg_attr(not(windows), expect(dead_code, reason = "the detector reads it on Windows alone"))]
	#[serde(rename = "DisplayName")]
	pub display_name: Option<String>,
}

impl Item {
	/// The game's install folder, when it is there on this computer's own disks (`install::on_this_computer`: the
	/// manifests are in `ProgramData`, which other accounts may write).
	pub(super) fn install_dir(&self) -> Option<PathBuf> {
		self.install_location
			.as_deref()
			.map(PathBuf::from)
			.filter(|d| crate::install::on_this_computer(d) && d.is_dir())
	}

	/// Whether the game launches `image` (its launch executable's file name, in any case).
	pub(super) fn launches(&self, image: &str) -> bool {
		self.launch_executable
			.as_deref()
			.map(|l| l.replace('\\', "/"))
			.is_some_and(|l| l.rsplit('/').next().is_some_and(|f| f.eq_ignore_ascii_case(image)))
	}
}

/// Every manifest that reads; none when the folder is missing.
pub(super) fn items() -> Vec<Item> {
	let Some(data) = std::env::var_os("ProgramData") else { return Vec::new() };
	let manifests = PathBuf::from(data).join("Epic").join("EpicGamesLauncher").join("Data").join("Manifests");
	let Ok(entries) = std::fs::read_dir(manifests) else { return Vec::new() };
	entries
		.flatten()
		.map(|entry| entry.path())
		.filter(|path| path.extension().is_some_and(|e| e.eq_ignore_ascii_case("item")))
		.filter_map(|path| serde_json::from_slice(&std::fs::read(&path).ok()?).ok())
		.collect()
}
