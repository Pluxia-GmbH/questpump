//! Usage statistics and the privacy page on the computer (contract/telemetry-api.yaml, "The switches and the consent").
//! The Menu's switch Usage statistics is the app's own setting (`settings.json` `usage`: the player's answer with its
//! moment and the words' version, `telemetry::COPY_VERSION`) with the crash reports' three states and default: on
//! outside an opt-in region, off inside one until the player says yes (the region read at each start while a switch is
//! not answered, never stored; `Helper::region`). It holds the anonymous counts (`runtime/counts.rs`): turning it off
//! stops them at once and deletes what waits; nothing else depends on it. While a switch is not answered, the first main
//! window the player opens in a run shows the page `desk.privacy` (`Desk::privacy_page`; never at login or after an
//! update, which open no window): what each purpose sends, the two switches as the region reads them, how each is used
//! (`questpump.com/privacy#telemetry` in the browser, `about.open`), and Done, which keeps both as the player's answers;
//! in an opt-in region learnclash's buttons instead: Accept all, which answers both on (Save choices in its place once
//! the player moved a switch there, which keeps both as set), over Reject all, which answers both off; closed any other
//! way, nothing is answered and it comes again at the next start. The `usage` topic carries the switch as it reads and
//! whether this start read an opt-in region. Intents: `{"type":"usage.switch","on":bool}` (the Menu's switch),
//! `{"type":"usage.seen","feature"?:<feature_day's feature>}` (the page in use, and the root or the Menu it shows: the
//! counts' `active` and `feature_day`), `{"type":"privacy.done","usage":bool,"crash":bool}` (Done and Save choices as
//! set, Accept all with both true, Reject all with both false).

use std::sync::atomic::Ordering;

use questpump_core::telemetry::Source;
use serde::Serialize;
use serde_json::Value;
use tauri::Manager as _;

use super::{Helper, now};
use crate::desk::{Desk, Intent};
use crate::settings::Region;

/// The topic the Menu's switch and the privacy page follow.
pub(crate) const TOPIC: &str = "usage";

/// The `usage` topic: the switch as it reads (not answered reads the region's default), and whether this start read
/// the computer in an opt-in region, where the privacy page offers Accept all and Reject all.
#[derive(Serialize)]
struct UsageTopic {
	on: bool,
	opt_in: bool,
}

impl Helper {
	/// In the start, after the crash sweep: the privacy page waits for the first main window while a switch is not
	/// answered, and the switch goes to the Menu.
	pub(super) fn usage_start(&mut self) {
		let (crash, usage) = (self.desk.settings.crash.choice.send, self.desk.settings.usage.send);
		match usage {
			Some(on) => log::info!("usage: Usage statistics {}, the player's answer", if on { "on" } else { "off" }),
			None if self.region == Region::OptIn => {
				log::info!("usage: Usage statistics not answered, an opt-in region");
			}
			None => log::info!("usage: Usage statistics not answered, on outside the opt-in regions"),
		}
		if crash.is_none() || usage.is_none() {
			self.app.state::<Desk>().privacy_page.store(true, Ordering::Relaxed);
		}
		self.usage_publish();
	}

	/// The `usage` and `privacy` areas' intents.
	pub(crate) fn usage_intent(&mut self, intent: &Intent) -> Result<(), String> {
		let flag = |name: &str| {
			intent.arg(name).and_then(Value::as_bool).ok_or_else(|| format!("{} takes {name}", intent.kind()))
		};
		match intent.kind() {
			"usage.switch" => self.usage_answer(flag("on")?),
			"usage.seen" => match intent.arg("feature") {
				None | Some(Value::Null) => self.counts_active(Source::Desktop),
				Some(Value::String(feature)) => self.counts_feature(feature),
				Some(_) => return Err("usage.seen's feature is a name".to_owned()),
			},
			"privacy.done" => {
				let (usage, crash) = (flag("usage")?, flag("crash")?);
				self.usage_answer(usage);
				self.crash_answer(crash);
			}
			_ => return Err(intent.unknown()),
		}
		Ok(())
	}

	/// The player's answer to Usage statistics (the Menu's switch, the privacy page's answer), kept with its moment and
	/// the words' version: off empties the counts' outbox and their kept days at once; on counts from now on.
	pub(crate) fn usage_answer(&mut self, on: bool) {
		let was = self.usage_on();
		self.desk.settings.usage.answer(on, now().unix);
		self.desk.save_settings();
		log::info!("usage: Usage statistics {}, the player's answer", if on { "on" } else { "off" });
		match (was, on) {
			(_, false) => self.counts_off(),
			(false, true) => self.counts_resume(),
			(true, true) => {}
		}
		self.privacy_settled();
		self.usage_publish();
	}

	/// Both switches answered (here, on the crash page or in the Menu): the privacy page asks no more.
	pub(crate) fn privacy_settled(&self) {
		if self.desk.settings.crash.choice.send.is_some() && self.desk.settings.usage.send.is_some() {
			self.app.state::<Desk>().privacy_page.store(false, Ordering::Relaxed);
		}
	}

	/// The `usage` topic as it stands.
	fn usage_publish(&self) {
		let topic = UsageTopic { on: self.usage_on(), opt_in: self.region == Region::OptIn };
		self.app.state::<Desk>().publisher.publish(TOPIC, &topic);
	}
}
