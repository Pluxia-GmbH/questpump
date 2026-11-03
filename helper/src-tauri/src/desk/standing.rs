//! Where the player stands among QuestPump's players, for the share cards' rarity line (docs/plan.md slice 51, "The
//! share cards tell one true thing"; contract/standing-api.yaml): `GET https://questpump.com/api/standing` as the Share
//! window opens a card that can carry the line (the intent `standing.look`), 2 s at most, its answer kept in memory
//! for the hour its `max-age` allows and handed to the page as the `standing` topic, since the page's CSP lets it fetch
//! nothing. The request carries nothing of the player's (no id, no body, no cookie; QuestPump's User-Agent alone, as the
//! feedback's posts) and follows no redirect; a failed or late answer leaves the card without the line, and the next
//! opening asks again.
//!
//! **The `standing` topic**: `{streak: {"3": share, …}, colour: {"2": share, …}, until}`, each share a fraction from 0
//! to 1 under its threshold (both maps empty while the Worker holds too few points), `until` the answer's end (unix
//! ms), after which the page draws no line; absent before the first answer.

use std::collections::BTreeMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager as _};

use super::{Desk, Intent};
use crate::lan;
use crate::runtime::Helper;

/// The page's topic.
const TOPIC: &str = "standing";
/// The route (contract/standing-api.yaml).
const ROUTE: &str = "https://questpump.com/api/standing";
/// The longest the answer may take, from the connection on: the card is drawn without the line meanwhile.
const TIMEOUT: Duration = Duration::from_secs(2);
/// How long an answer is kept: its `Cache-Control` `max-age` (`x-limits.max_age_s`), in ms.
const KEEP_MS: i64 = 3_600_000;
/// The most an answer may bring (the shares of 14 thresholds are a few hundred bytes).
const MOST_BYTES: u64 = 16 << 10;
/// The thresholds the route names: a streak's days and a colour's quality.
const STREAKS: [&str; 8] = ["3", "7", "14", "30", "60", "100", "180", "365"];
const COLOURS: [&str; 6] = ["2", "3", "4", "5", "6", "7"];

/// The route's shares, each threshold to its fraction of players (`Standing` less its `day` and `days`).
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
struct Shares {
	streak: BTreeMap<String, f64>,
	colour: BTreeMap<String, f64>,
}

impl Shares {
	/// Only the thresholds the contract names, each with a share from 0 to 1.
	fn checked(mut self) -> Self {
		let keep = |names: &[&str], map: &mut BTreeMap<String, f64>| {
			map.retain(|k, v| names.contains(&k.as_str()) && v.is_finite() && (0.0..=1.0).contains(v));
		};
		keep(&STREAKS, &mut self.streak);
		keep(&COLOURS, &mut self.colour);
		self
	}
}

/// The topic: the shares and when they stop counting.
#[derive(Serialize, Clone, Debug, PartialEq)]
struct View {
	#[serde(flatten)]
	shares: Shares,
	until: i64,
}

/// The answer kept, and whether an ask is on its way.
static KEPT: Mutex<Option<View>> = Mutex::new(None);
static ASKING: AtomicBool = AtomicBool::new(false);

impl Helper {
	/// The `standing` area's intent: `standing.look`, the Share window opened a card that can carry the rarity line.
	pub(crate) fn standing_intent(&self, intent: &Intent) -> Result<(), String> {
		if intent.kind() != "standing.look" {
			return Err(intent.unknown());
		}
		self.standing_look();
		Ok(())
	}

	/// The standing asked once, unless an answer younger than an hour is kept (the topic holds it) or an ask is on its
	/// way.
	fn standing_look(&self) {
		let now = lan::now_ms();
		let fresh = KEPT.lock().is_ok_and(|kept| kept.as_ref().is_some_and(|v| v.until > now));
		if fresh || ASKING.swap(true, Ordering::AcqRel) {
			return;
		}
		tauri::async_runtime::spawn(ask(self.app.clone()));
	}
}

/// Asks the route, then keeps and publishes the answer; a failure is logged and leaves the topic as it was (an answer
/// past its `until`, which the page no longer draws, or none).
async fn ask(app: AppHandle) {
	let got = fetch().await;
	ASKING.store(false, Ordering::Release);
	match got {
		Ok(shares) => {
			let view = View { shares, until: lan::now_ms() + KEEP_MS };
			if let Ok(mut kept) = KEPT.lock() {
				*kept = Some(view.clone());
			}
			app.state::<Desk>().publisher.publish(TOPIC, &view);
		}
		Err(e) => log::info!("standing: no answer: {e:#}"),
	}
}

/// One GET of the route: its shares, or why none came.
async fn fetch() -> anyhow::Result<Shares> {
	let client = reqwest::Client::builder()
		.user_agent(crate::feedback::USER_AGENT)
		.tls_backend_native()
		.timeout(TIMEOUT)
		.redirect(reqwest::redirect::Policy::none())
		.build()?;
	let response = client.get(ROUTE).send().await.map_err(reqwest::Error::without_url)?.error_for_status()?;
	anyhow::ensure!(
		response.content_length().is_none_or(|n| n <= MOST_BYTES),
		// copy-lint: not shown (a log line)
		"the answer offers {:?} bytes",
		response.content_length()
	);
	let body = response.bytes().await?;
	anyhow::ensure!(u64::try_from(body.len()).is_ok_and(|n| n <= MOST_BYTES), "the answer brings {} bytes", body.len());
	Ok(serde_json::from_slice::<Shares>(&body)?.checked())
}
