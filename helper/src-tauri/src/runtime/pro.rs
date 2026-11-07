//! Pro on the computer in the loop (docs/plan.md slice 35, "Pro on the computer", and slice 64, "Every device reads it"
//! and "The screens"; `questpump_core::pro`): a paired phone's proof rides its sync and is checked at once, the
//! account's token comes from the sync server (`GET pro`, questpump-desk's `sync`) and is checked offline the same way,
//! the grants live in `pro.json` in the data folder (checked again at every start under this build's rules), a new
//! pairing code clears the phones' (the window's, or the start's when the settings did not read) and the pair tick
//! drops the ones that ended and a token of another account. Pro's extras show on the computer while any phone's grant
//! or the account's stands, paired or not, also after another phone's Unpair ended the pairings (`end_pairings`): a
//! phone that holds the code keeps its grant and names itself paired again at its next sync. The log names the device
//! or the account's source, the outcome and the date a grant holds until, never a proof, a token or an id it carries;
//! no proof or token reaches diagnostics, feedback, the mirror, `Alive.lua` or anything the game reads. The Menu's Pro
//! row follows each change of the state (`menu.about`, `Helper::pro_state`), which keeps each phone's last refusal in
//! memory until its next proof is taken or a new pairing code ends the pairings, so a paired phone whose proof this
//! build could not confirm (no Play licensing key built in, Apple's Sandbox in a release) reads so, never "Comes with
//! Pro on your phone".
//!
//! **The `menu.pro` topic** (Pro's page): `{state: "on"|"off"|"refused"|null, from: "app_store"|"play"|"web"|null,
//! get, sync, busy, failed}`: `state` as `menu.about`'s `pro`; `from` where the account's Pro was bought while its
//! token stands; `get` whether questpump.com sells Pro (`web.paddle.environment` is `live` and `web.paddle.prices`
//! holds a price: Get Pro shows only then);
//! `sync` Sync's switch (Get Pro asks to turn it on first); `busy` while Manage Pro waits for the server; `failed` once
//! the browser did not open or the server did not answer, until the next.
//! **Intents**: `pro.look`, `pro.get {sync}` (`sync: true` turns Sync on first), `pro.manage`.

use std::collections::BTreeMap;
use std::path::Path;

use questpump_core::link::{Device, ProProof};
use questpump_core::pro::{self, Grants, Refusal, Rules, Source, Taken, TokenTaken};
use serde::Serialize;

use super::Helper;
use crate::desk::Intent;
use crate::lan;

/// Pro's page's topic.
pub(crate) const PAGE: &str = "menu.pro";
/// Where Get Pro opens the browser: questpump.com's Pro page, where a purchase comes to the account that links the
/// email it is paid with (contract/sync-api.yaml, "From the website").
const PRO_PAGE: &str = "https://questpump.com/pro";

/// A release build takes Apple's `Production` only; a debug build `Sandbox` too.
const RULES: Rules = Rules { sandbox: cfg!(debug_assertions) };

/// The paired phones' Pro grants in the loop.
pub(crate) struct Pro {
	grants: Grants,
	/// Pro's state on the computer as the log last named it (`Helper::pro_state`).
	shown: Option<ProState>,
	/// A kept grant was dropped at the start's check: the file is written again at the start.
	due: bool,
	/// Each phone's last proof this build refused, by the sync's `client`, until its next proof is taken or it unpairs.
	refused: BTreeMap<String, Refusal>,
	/// Get Pro and Manage Pro on Pro's page.
	pub(super) page: Page,
}

/// What Pro's page opens in the browser: Get Pro's questpump.com page (`PRO_PAGE`) or Manage Pro's portal (`POST
/// pro/portal`, `questpump_desk::sync`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ProLink {
	Page,
	Portal,
}

/// Get Pro's and Manage Pro's state on Pro's page.
#[derive(Default)]
pub(super) struct Page {
	/// Manage Pro's call is out.
	pub(super) out: bool,
	/// The last one did not open the browser (or the server did not answer).
	pub(super) failed: bool,
}

#[derive(Serialize)]
#[expect(clippy::struct_excessive_bools, reason = "the page's facts, one yes or no a field, as the topic names them")]
struct PageTopic {
	state: Option<ProState>,
	from: Option<Source>,
	get: bool,
	sync: bool,
	busy: bool,
	failed: bool,
}

/// Pro's state on the computer while a phone is paired or a grant stands (the Menu's Pro row and its page).
#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ProState {
	/// A phone's grant or the account's stands: the extras show.
	On,
	/// No grant stands: Pro comes with the phone app's subscription, or from questpump.com.
	Off,
	/// No grant stands, and a paired phone's last proof could not be confirmed here (`Refusal::lapsed` names the
	/// refusals that mean the subscription does not hold, which read as `Off`).
	Refused,
}

impl Pro {
	/// The grants `pro.json` keeps, checked again now, the account's token for `account`, the one the computer is in;
	/// an absent file keeps none. The log says once a run when the build checks no proof, no Play proof or no account
	/// token.
	pub(super) fn load(dir: &Path, account: Option<&str>) -> Self {
		if let Some(why) = pro::facts_problem() {
			log::error!("pro: the built-in contract/store.json: {why}: every proof is refused");
		} else {
			if !pro::play_key_built_in() {
				log::info!("pro: no Play licensing key built in: every Play proof is refused");
			}
			if pro::token_keys_built_in() == 0 {
				log::info!("pro: no account token key built in: every account token is ignored");
			}
		}
		let now = lan::now_ms();
		let (grants, dropped) = std::fs::read(dir.join(pro::FILE))
			.map_or_else(|_| (Grants::default(), 0), |bytes| Grants::from_file(&bytes, now, RULES, account));
		if dropped > 0 {
			log::info!("pro: {dropped} kept grants dropped at the start's check");
		}
		Self { grants, shown: None, due: dropped > 0, refused: BTreeMap::new(), page: Page::default() }
	}

	/// Any verified grant stands at `now_ms`, a phone's or the account's (`account`: `Grants::on`).
	pub(crate) fn grants_on(&self, now_ms: i64, account: Option<&str>) -> bool {
		self.grants.on(now_ms, account)
	}
}

/// The date a grant holds until, as the log names it ("2026-11-14", UTC).
fn day(ms: i64) -> String {
	time::OffsetDateTime::from_unix_timestamp(ms.div_euclid(1_000))
		.map_or_else(|_| ms.to_string(), |t| t.date().to_string())
}

impl Helper {
	/// Where the account's Pro was bought while its token stands (Pro's page's line and Manage Pro).
	fn pro_from(&self) -> Option<Source> {
		self.desk.p.pro.grants.account_pro(lan::now_ms(), self.desk.settings.sync.account.as_deref()).map(|g| g.source)
	}

	/// Pro's state as the Menu's Pro row and its page say it: none while no phone is paired and no grant stands.
	pub(crate) fn pro_state(&self) -> Option<ProState> {
		self.desk.pro_on().map(|on| {
			if on {
				ProState::On
			} else if self.desk.p.pro.refused.values().any(|why| !why.lapsed()) {
				ProState::Refused
			} else {
				ProState::Off
			}
		})
	}

	/// At the start: every kept grant goes when the settings made a new pairing code (`new_key`: they did not read, or
	/// there were none), since no phone holds that code yet; the file again when its check dropped a grant; and Pro's
	/// state in the log.
	pub(super) fn start_pro(&mut self, new_key: bool) {
		if new_key && self.desk.p.pro.grants.clear() {
			log::info!("pro: a new pairing code at the start: every kept grant dropped");
			self.desk.p.pro.due = true;
		}
		if self.desk.p.pro.due {
			self.desk.p.pro.due = false;
			self.save_pro();
		}
		self.show_pro();
	}

	/// A phone's proof from its sync: kept, dropped or refused; a refusal never touches the sync it rode on.
	pub(super) fn take_pro(&mut self, client: &str, device: Device, proof: &ProProof) {
		let (who, store) = (device.name(), proof.store.name());
		let taken = self.desk.p.pro.grants.take(client, device, proof, lan::now_ms(), RULES);
		// the phone's last refusal stands until a proof of it is taken
		if let Taken::Refused(why) = taken {
			self.desk.p.pro.refused.insert(client.to_owned(), why);
		} else {
			self.desk.p.pro.refused.remove(client);
		}
		match taken {
			Taken::Kept(grant) => {
				match grant.renewal {
					Some(why) => log::info!(
						"pro: {who} {store} verified, until {} (its renewal info refused: {why:?})",
						day(grant.until)
					),
					None => log::info!("pro: {who} {store} verified, until {}", day(grant.until)),
				}
				self.save_pro();
			}
			Taken::Same | Taken::Dropped(false) => {}
			Taken::Dropped(true) => {
				log::info!("pro: {who} has no Pro any more: its grant dropped");
				self.save_pro();
			}
			Taken::Refused(why) => log::info!("pro: {who} {store} refused: {why:?}"),
		}
		self.pro_changed();
	}

	/// The account's token and status as the sync server answered (`GET pro`; no token and no status that stands: the
	/// account has no Pro): kept, dropped or refused; a refusal keeps the token kept before.
	pub(crate) fn take_account_pro(&mut self, token: Option<&str>, status: Option<&str>) {
		let Some(account) = self.desk.settings.sync.account.clone() else { return };
		match self.desk.p.pro.grants.take_token(token, status, &account, lan::now_ms()) {
			TokenTaken::Kept(grant) => {
				log::info!("pro: the account's Pro ({:?}) verified, until {}", grant.source, day(grant.ends));
				self.save_pro();
			}
			TokenTaken::Same | TokenTaken::Dropped(false) => {}
			TokenTaken::Dropped(true) => {
				log::info!("pro: the account has no Pro any more: its token dropped");
				self.save_pro();
			}
			TokenTaken::Refused(why) => log::info!("pro: the account's token refused: {why:?}"),
		}
		self.pro_changed();
		self.publish_pro();
	}

	/// A new pairing code ends every pairing, and every phone's grant with them.
	pub(super) fn clear_pro(&mut self) {
		self.desk.p.pro.grants.clear();
		self.desk.p.pro.refused.clear();
		self.save_pro();
		self.pro_changed();
	}

	/// A phone unpaired (`POST /v1/unpair`): its grant and its last refusal go with it, so a phone paired later shows
	/// Pro on the computer only by its own.
	pub(super) fn forget_pro(&mut self, client: &str) {
		if self.desk.p.pro.grants.forget(client) {
			log::info!("pro: the unpaired phone's grant dropped");
			self.save_pro();
		}
		self.desk.p.pro.refused.remove(client);
		self.pro_changed();
	}

	/// The pair tick: the grants that ended go, and a token of another account than the one the computer is in, and
	/// the state is computed again.
	pub(super) fn on_pro_tick(&mut self) {
		if self.desk.p.pro.grants.prune(lan::now_ms(), self.desk.settings.sync.account.as_deref()) {
			log::info!("pro: a grant ended");
			self.save_pro();
		}
		self.pro_changed();
	}

	/// Pro's state on the computer (`pro_state`) again, in the log when it changed.
	pub(super) fn pro_changed(&mut self) {
		if self.pro_state() != self.desk.p.pro.shown {
			self.show_pro();
		}
	}

	/// Pro's state on the computer in the log: at the start, and at each change.
	fn show_pro(&mut self) {
		self.desk.p.pro.shown = self.pro_state();
		match self.desk.p.pro.shown {
			Some(ProState::On) => log::info!("pro: the extras are on"),
			Some(ProState::Off) => log::info!("pro: the extras are off"),
			Some(ProState::Refused) => log::info!("pro: the extras are off: the phone's proof was refused here"),
			None => log::info!("pro: no phone paired"),
		}
		// the Menu's Pro row (`desk::menu_pages::about`) and Pro's page
		self.publish_about();
		self.publish_pro();
		// the Book's and the Log's Pro parts (`desk::book`)
		self.desk.book_log_pro();
	}

	/// The `pro` area's intents: Pro's page's look, Get Pro (questpump.com's Pro page in the browser, `PRO_PAGE`; with
	/// `sync: true` Sync turned on first, since Pro goes with the account) and Manage Pro (Paddle's customer portal,
	/// `Desk::sync_portal`).
	pub(crate) fn pro_intent(&mut self, intent: &Intent) -> Result<(), String> {
		match intent.kind() {
			"pro.look" => self.desk.p.pro.page.failed = false,
			"pro.get" => {
				if !pro::web_prices() {
					return Err("pro.get: questpump.com sells no Pro".to_owned());
				}
				if !self.desk.settings.sync.on {
					if intent.arg("sync").and_then(serde_json::Value::as_bool) != Some(true) {
						return Err("pro.get: Sync is off".to_owned());
					}
					self.desk.sync_switch(true)?;
				}
				self.pro_link_done(ProLink::Page, Some(PRO_PAGE));
			}
			"pro.manage" => {
				if self.pro_from() != Some(Source::Web) {
					return Err("pro.manage: the account's Pro is not questpump.com's".to_owned());
				}
				self.desk.p.pro.page.failed = false;
				if self.desk.sync_portal() {
					self.desk.p.pro.page.out = true;
				} else {
					self.pro_link_done(ProLink::Portal, None);
				}
			}
			_ => return Err(intent.unknown()),
		}
		self.publish_pro();
		self.desk.sync_drive();
		Ok(())
	}

	/// What Get Pro or Manage Pro came to: the browser opened at its address, or the page says it did not open or the
	/// server did not answer (no address).
	pub(super) fn pro_link_done(&mut self, link: ProLink, url: Option<&str>) {
		let opened = url.map(crate::desk::menu_pages::about::open_url);
		match &opened {
			Some(Ok(())) => log::info!("pro: {link:?} opened in the browser"),
			Some(Err(e)) => log::warn!("pro: {link:?} not opened: {e}"),
			None => {}
		}
		self.desk.p.pro.page.failed = !matches!(opened, Some(Ok(())));
		self.publish_pro();
	}

	/// Manage Pro's call answered (`Platform::portal_answered`): the portal's address opened, or the page says the
	/// server did not answer.
	pub(crate) fn portal_done(&mut self, url: Option<&str>) {
		self.desk.p.pro.page.out = false;
		self.pro_link_done(ProLink::Portal, url);
	}

	/// Pro's page's topic as it stands.
	pub(crate) fn publish_pro(&self) {
		let page = &self.desk.p.pro.page;
		let topic = PageTopic {
			state: self.pro_state(),
			from: self.pro_from(),
			get: pro::web_prices(),
			sync: self.desk.settings.sync.on,
			busy: page.out,
			failed: page.failed,
		};
		self.publish(PAGE, &topic);
	}

	fn save_pro(&self) {
		match self.desk.p.pro.grants.to_json() {
			Ok(bytes) => self.save_file(pro::FILE, bytes),
			Err(e) => log::warn!("{}: {e}", pro::FILE),
		}
	}
}
