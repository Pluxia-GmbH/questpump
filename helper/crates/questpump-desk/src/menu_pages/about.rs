//! Safety and about and the safety note, shared by every platform (docs/ui.md, "The Menu on the computer"; docs/plan.md
//! slice 63, "The screens"): who makes QuestPump and where, the not-a-medical-device line, the contact for questions
//! and injuries, the privacy, imprint and security pages, the version, and the safety note the apps' listings carry
//! (the addon's own words, `contract/sheet.json`). The page (`helper/dist/views/menu/about.js`) draws the `menu.about`
//! topic, whose shape is [`About`]: what this module fills, plus the platform's own parts (the desktop app's Pro row
//! and what QuestPump does on this computer; none in the browser), and the platform opens the address [`page_url`]
//! names for `about.open`, never one the page hands over.

use std::sync::LazyLock;

use questpump_core::verdict;
use serde::Serialize;
use serde_json::Value;

use crate::Intent;
use crate::facts::facts;

/// The topic.
pub const ABOUT: &str = "menu.about";

/// The site's pages Safety and about opens (`apple/iOS/Site.swift`'s and Android's twins) and the privacy page's parts
/// the Menu's pages link to.
const PAGES: [(&str, &str); 5] = [
	("privacy", "https://questpump.com/privacy"),
	// the privacy page's "How each is used": the policy's part on crash reports and usage statistics
	("telemetry", "https://questpump.com/privacy#telemetry"),
	// its "Sync between your devices", the Email page's Privacy row (slice 60)
	("sync", "https://questpump.com/privacy#sync"),
	("imprint", "https://questpump.com/imprint"),
	("security", "https://questpump.com/security"),
];

/// Questions, problems and injuries to report: the imprint's address, the EU product-safety contact (GPSR Art. 19),
/// `contract/links.json`'s `contact`, its one owner (empty where the file does not read, which `copytext.py --check`
/// refuses).
pub static CONTACT: LazyLock<String> = LazyLock::new(|| {
	serde_json::from_str::<Value>(include_str!("../../../../../contract/links.json"))
		.ok()
		.and_then(|links| links.get("contact").and_then(Value::as_str).map(str::to_owned))
		.unwrap_or_default()
});

/// The fonts, the sounds and the licences' texts as `tools/ledger.py` writes them for every app.
pub static CREDITS: LazyLock<Value> = LazyLock::new(|| {
	serde_json::from_str(include_str!("../../../../../contract/design/acknowledgements.json")).unwrap_or_default()
});

/// The verdict rule's numbers How the heart-rate check works words (`views/menu/about.js`; the phones read their cores'
/// `VerdictRule`), so the page never drifts from the rule.
#[derive(Serialize)]
pub struct Heart {
	/// The seconds before the pump whose median heart rate is the baseline.
	before: u16,
	/// The seconds after a move's start in which the rise must show.
	window: u16,
	/// The rise in beats a minute a move of reps or a timed one needs, and the one a hold or a stretch needs.
	reps: u16,
	hold: u16,
	/// The seconds after the set a move the hands hold up is judged on, with a watch.
	after: u16,
}

#[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "the rule's windows are whole seconds")]
const HEART: Heart = Heart {
	before: verdict::BASELINE_S as u16,
	window: verdict::WINDOW_S as u16,
	reps: verdict::DYNAMIC_BPM,
	hold: verdict::ISO_BPM,
	after: verdict::AFTER_SET_S as u16,
};

/// The `menu.about` topic: the version, the platform's Pro row (`Pro`, none where it has none), the safety note, the
/// contact, the heart-rate rule's numbers and what QuestPump does on this computer (`Does`, none where it has none).
#[derive(Serialize)]
pub struct About<Pro: Serialize, Does: Serialize> {
	version: &'static str,
	pro: Option<Pro>,
	safety: &'static str,
	contact: &'static str,
	heart: Heart,
	does: Option<Does>,
}

impl<Pro: Serialize, Does: Serialize> About<Pro, Does> {
	/// The topic for the app of `version`, with the platform's own parts.
	#[must_use]
	pub fn new(version: &'static str, pro: Option<Pro>, does: Option<Does>) -> Self {
		Self { version, pro, safety: &facts().sheet.safety, contact: CONTACT.as_str(), heart: HEART, does }
	}
}

/// The address `about.open` names: one of the site's `PAGES` (`page`), or a credited sound pack's page (`asset`, its
/// place in the Acknowledgements' list), never an address the page hands over.
///
/// # Errors
/// An intent that names neither, or one no page or asset answers.
pub fn page_url(intent: &Intent) -> Result<&'static str, String> {
	if let Some(asset) = intent.arg("asset").and_then(Value::as_u64) {
		let at = usize::try_from(asset).map_err(|e| e.to_string())?;
		return CREDITS
			.get("assets")
			.and_then(|assets| assets.get(at))
			.and_then(|a| a.get("page"))
			.and_then(Value::as_str)
			.filter(|url| url.starts_with("https://"))
			.ok_or_else(|| format!("about.open: no asset {at}"));
	}
	let page = intent.arg("page").and_then(Value::as_str).ok_or("about.open takes page or asset")?;
	PAGES
		.iter()
		.find(|(name, _)| *name == page)
		.map(|(_, url)| *url)
		.ok_or_else(|| format!("about.open: no page {page}"))
}
