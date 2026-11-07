//! Your setup and the first five minutes (docs/plan.md slice 35; docs/ui.md, "The Menu on the computer" and "The first
//! five minutes"): Your setup and Your moves (`setup`) and the welcome flow (`welcome`). The desk holds their state and
//! publishes it as topics, each a whole value: `menu.setup` and `welcome`; the page draws them
//! (`helper/dist/views/menu/`, `views/welcome/`) and sends intents `setup.*` and `welcome.*`. The words they show come
//! from `contract/copy/en.json` through the page's `copy.js`, the addon's own words (the equipment, the Body switches,
//! the experience question, the joints, the focus, the muscles, the safety note) from `contract/sheet.json` as the
//! phones bundle it, the moves' names and groups from `contract/moves.json` and `contract/library.json`, all through
//! the built-in facts (`facts.rs`): a file that does not read leaves its words empty, and the pages show no row without
//! a word. Safety and about and the safety note (`about`, the `menu.about` topic) are every platform's, each adding its
//! own parts; the desktop app's own Menu pages (Backup, what QuestPump does on this computer, the Acknowledgements) are
//! its `desk/menu_pages.rs`'s.

pub mod about;
pub mod setup;
pub mod welcome;

use std::collections::BTreeMap;
use std::sync::LazyLock;

use serde::Serialize;

use crate::facts::{ById, Focus, GearGroup, Named, facts};
use crate::{Desk, Platform};

/// The topics these pages publish.
pub const SETUP: &str = "menu.setup";
pub const WELCOME: &str = "welcome";

/// The pages' state in the loop, beside what settings.json and body.json keep.
#[derive(Default)]
pub struct MenuPages {
	/// What the engine said of the setup at its last look; none before one, or while it does not answer.
	setup: Option<setup::Look>,
	/// A look at the setup is on the engine's door, and another was asked meanwhile (`Desk::setup_look`).
	looking: bool,
	look_again: bool,
	/// The muscles the first pump trained, the welcome's Hero step's; none before its look.
	trained: Option<welcome::Trained>,
	/// What QuestPump found as the first five minutes last read it (`Desk::welcome_found`).
	found: crate::seams::Found,
}

/// The words of the sheet the pages show, one value built once: the equipment in the game's order and its groups, the
/// place's and the body's switches by their profile keys (each with the sheet's key a refused Start's gap names it by),
/// the experience question and its answers, the joints, the focus choices, the muscles and the qualities.
#[derive(Serialize)]
struct Words {
	gear: Vec<GearWord>,
	/// The equipment's groups (2026-11-03), each its key and its pieces in the game's order: Your setup's and a
	/// place's window's heads that open and close.
	gear_groups: Vec<GearGroup>,
	switches: BTreeMap<&'static str, SwitchWord>,
	experience: Experience,
	joints: Vec<ById>,
	focus: Vec<Focus>,
	muscles: Vec<ById>,
	/// The qualities' names, 1 Starter to 7 Artifact (`qualities[0]` is Starter's).
	qualities: Vec<String>,
}

/// A piece of equipment by its key, its name and what it is (the game's equipment tooltip's words, which a gear chip's
/// tooltip shows; none where the sheet has none).
#[derive(Serialize)]
struct GearWord {
	key: String,
	name: String,
	tip: Option<String>,
}

/// A switch of Your setup: its name, and its key in the sheet's words, by which the game's rules name it
/// (Progress.lua `ns.SetupGap`'s `key`: a refused Start's Your setup row opens at the switch it names).
#[derive(Serialize)]
struct SwitchWord {
	name: String,
	gap: &'static str,
}

/// The experience question (`experience.title`) and its answers in their order.
#[derive(Serialize)]
struct Experience {
	title: String,
	answers: Vec<Named>,
}

/// The switches' profile keys and the sheet's keys: their words', and the key the game's rules name each by
/// (`ns.SetupGap`).
const SWITCHES: [(&str, &str); 6] = [
	("floor", "floor"),
	("space", "space"),
	("quiet", "quiet"),
	("no_jump", "nojump"),
	("no_strain", "nostrain"),
	("no_head_down", "noheaddown"),
];

pub static WORDS: LazyLock<serde_json::Value> = LazyLock::new(|| {
	let sheet = &facts().sheet;
	let word = |key: &str| sheet.gear.get(key).cloned();
	let words = Words {
		gear: sheet
			.gear_order
			.iter()
			.filter_map(|key| {
				word(key).map(|name| GearWord { key: key.clone(), name, tip: sheet.gear_tips.get(key).cloned() })
			})
			.collect(),
		gear_groups: sheet.gear_groups.clone(),
		switches: SWITCHES.iter().filter_map(|&(key, gap)| Some((key, SwitchWord { name: word(gap)?, gap }))).collect(),
		experience: Experience {
			title: sheet.experience.get("title").cloned().unwrap_or_default(),
			answers: questpump_core::body::experiences()
				.into_iter()
				.filter_map(|key| {
					sheet.experience.get(key).map(|name| Named { key: key.to_owned(), name: name.clone() })
				})
				.collect(),
		},
		joints: sheet.joints.clone(),
		focus: sheet.focus.clone(),
		muscles: sheet.muscles.clone(),
		qualities: sheet.qualities.clone(),
	};
	serde_json::to_value(words).unwrap_or_else(|e| {
		log::error!("menu: the sheet's words did not serialize: {e}");
		serde_json::Value::Null
	})
});

impl<P: Platform> Desk<P> {
	/// A topic of these pages, whole (the publisher sends it only when it changed).
	pub fn publish(&self, topic: &str, value: &impl Serialize) {
		self.publisher.publish(topic, value);
	}
}
