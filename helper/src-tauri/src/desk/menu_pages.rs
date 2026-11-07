//! The desktop app's own Menu pages (docs/plan.md slice 35; docs/ui.md, "The Menu on the computer" and "Pro on the
//! computer"): Backup (`backup`), QuestPump's Safety and about, the safety note, what QuestPump does on this computer
//! and the Acknowledgements (`about`). Rust holds their state and publishes it as topics, each a whole value:
//! `menu.backup`, `menu.about` and `menu.acknowledgements`; the page draws them (`helper/dist/views/menu/`) and sends
//! intents `backup.*` and `about.*`. Your setup and the first five minutes are the desk's (`questpump_desk::menu_pages`:
//! `menu.setup`, `welcome`).

pub(crate) mod about;
pub(crate) mod backup;

use serde::Serialize;
use tauri::Manager as _;

use super::Desk;
use crate::runtime::Helper;

/// The topics these pages publish.
pub(crate) const BACKUP: &str = "menu.backup";
pub(crate) const ACKNOWLEDGEMENTS: &str = "menu.acknowledgements";

impl Helper {
	/// A topic of these pages, whole (the publisher sends it only when it changed).
	pub(crate) fn publish(&self, topic: &str, value: &impl Serialize) {
		self.app.state::<Desk>().publisher.publish(topic, value);
	}

	/// At the start, once the settings and the body are read: every page's topic, the desk's with them. The engine's
	/// look at the setup comes when its first fold lands (`Desk::setup_look`, which the door asks; the welcome asks no
	/// experience of a player whose ladders hold rungs), and the welcome's first step once the stores are read
	/// (`Desk::welcome_found`).
	pub(crate) fn menu_start(&self) {
		self.publish_acknowledgements();
		self.publish_about();
		self.desk.publish_setup();
		self.publish_backup();
		self.desk.publish_welcome();
	}
}
