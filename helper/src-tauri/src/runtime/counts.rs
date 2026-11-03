//! The anonymous counts on the loop (contract/telemetry-api.yaml, "The anonymous counts"; questpump-core's
//! `telemetry.rs` owns the points, the catalog's checks, the buckets, what is kept and the counting rules). While Usage
//! statistics reads on (`Helper::usage_on`: the player's answer, else on outside an opt-in region), the app keeps, in
//! `counts.json`, the facts of each game day (04:00 local, `history::GameDay`) as they happen: the `desktop` source's
//! (in use: the window, a pump here, the game read or a device served; the build's crash; a pairing of the phone, the
//! strap or the pad; the link's, the Bluetooth link's (`link_ble`, slice 43) and the capture's day; the features used; the talent trees' cards, starts and refusals,
//! `tree_day`; the match offers; a return after three or more game days away) and the `game` source's (the strip read
//! or a row of the game's; a return), the unfinished moves skipped (each on the day it was made, which only a skip
//! taken as it was made, or of a move owed that day, tells) and the moves owed at the day's end, from the rows the
//! stores take (`Helper::counts_rows`, which looks at each new row once, from where the stores stood when the counts
//! first ran: `Helper::counts_load`, first in the start). The facts a later point reads are kept whatever the switch
//! says, and only the points wait for it: the first run, the game's first sight, each source's install due and its last
//! day used (`return`). At the first start of a game day, and when the game day turns while the app runs, every
//! finished game day not sent yet becomes one batch a source (`telemetry::bodies`, at most 40 points each), its points
//! from the facts and from the rows of that day the stores hold, the computer's own pumps (`desktop`: the app rows of
//! the pumps it ran, never one the server or a backup brought, and the catch-ups it ran) and the game's (`game`: its rows without a device's bit), each counted
//! by contract/feedback-api.yaml's rules (`telemetry::tally`); once a week the review's numbers (the computer's by
//! those rules, the game's by the addon's own `ns.FeedbackNumbers` through the engine) and the setup (this computer's
//! place and the focus; the game's profile as the strip last showed it). The batches go through their outbox
//! (`telemetry::OUTBOX`: kept at most 7 days, `runtime/outbox.rs` sends it as every outbox, a backlog one batch a
//! minute, the route's limit for a client address): 204 drops a batch, 400, 401 and 413 drop it with a log line naming
//! the code, a 429 keeps it for the round a minute on, anything else keeps it. Turning the switch off (or a start that
//! reads it off) empties the outbox and the kept days at once, and a day counted while it was off never goes. Nothing
//! here names the player, the computer or a time finer than the game day; the regional default is never stored.

use std::collections::BTreeMap;

use questpump_core::codes::{flags, outcome};
use questpump_core::link::{Ble, Transport};
use questpump_core::outbox::Answer;
use questpump_core::protocol::RowRecord;
use questpump_core::row::Row;
use questpump_core::telemetry::{self, Counts, DayTally, Facts, LinkBle, Point, Review, Source};
use tokio::sync::mpsc::UnboundedSender;

use super::outbox::{Lane, News, Sender};
use super::{Helper, ProState, install_key, now};
use crate::bluetooth::{self, Kind};
use crate::desk::facts::facts;
use crate::desk::history::{self, GameDay};
use crate::desk::pump::ran_here;
use crate::settings::Loaded;

/// The addon's first profile's name (DB.lua's), a key of an install's profiles.
const DEFAULT_PROFILE: &str = "Default"; // copy-lint: not shown (a profile's key)
/// The device bits of a row a device made: a row with none is the game's.
const DEVICES: u16 = flags::PHONE | flags::WATCH | flags::COMPUTER | flags::APP;

/// The counts' state on the loop: what `counts.json` keeps, the game days, the outbox and whether the file waits to be
/// written.
pub(crate) struct CountsState {
	kept: Counts,
	days: GameDay,
	sender: Sender,
	/// The game day of the last look, so its turn sends the day before.
	today: Option<i64>,
	/// The kept facts changed since `counts.json` was last asked to be written.
	dirty: bool,
	/// The engine is asked for the week's numbers: the batches wait for its answer.
	asking: bool,
	/// The player turned the strap or the walking pad on, and its first connection or failure is a pairing's
	/// (`pairing`'s attempt).
	trying: Vec<Kind>,
}

impl CountsState {
	/// The outbox in the data folder, the client, and where its batches report; the kept facts come at the start.
	pub(super) fn new(dir: &std::path::Path, client: Option<reqwest::Client>, news: UnboundedSender<News>) -> Self {
		Self {
			kept: Counts::default(),
			days: GameDay::new(facts().constants().day_offset_h),
			sender: Sender::new(dir, Lane::Counts, client, news),
			today: None,
			dirty: false,
			asking: false,
			trying: Vec::new(),
		}
	}
}

/// A game row of the computer: a catch-up it ran (the computer's bit alone). Its own pumps' rows are app rows of a pump
/// it ran (`pump::ran_here`), never the ones the server, a backup or another computer brought.
fn computer(r: &Row) -> bool {
	r.flags & flags::COMPUTER != 0
}

/// A row of the game: no device's bit.
fn game(r: &Row) -> bool {
	r.flags & DEVICES == 0
}

/// A skip of an unfinished move: a paid row with the skip's bit.
fn skip(r: &Row) -> bool {
	r.outcome == outcome::PAID && r.flags & flags::SKIPPED != 0
}

/// A record the stores took as it was made: the strip carried it (its load) or the app wrote it for a pump or a skip
/// (its pump's id); one taken in from a file (a save, the mirror, a restore) has neither.
fn made_here(r: &RowRecord) -> bool {
	r.load_id.is_some() || r.pump.is_some()
}

impl Helper {
	/// Usage statistics as it reads: the player's answer, else on outside an opt-in region.
	pub(crate) fn usage_on(&self) -> bool {
		self.settings.usage.reads(self.region)
	}

	/// The game day of unix second `t` (04:00 local).
	fn counts_day(&self, t: i64) -> i64 {
		self.counts.days.index(t)
	}

	/// First in the start, before anything takes rows (the install's people and every `apply` after them look at the
	/// stores' rows through `counts_rows`): the kept facts read, and on the first run with counts the first run and the
	/// game's first sight set once and the stores' rows as they stand taken as seen (a computer that ran an earlier build
	/// has no install to count: its first run is its earliest row, its first pumps came before the counts did, and the
	/// rows it holds are no new activity and no skip).
	pub(super) fn counts_load(&mut self, loaded: Loaded) {
		let read = questpump_core::mirror::read_data(&self.dir, Counts::FILE, |b| {
			serde_json::from_slice::<Counts>(b).map_err(|e| e.to_string())
		});
		match read {
			Ok(Some(kept)) => {
				self.counts.kept = kept;
				return;
			}
			Ok(None) => {}
			Err(why) => log::warn!("{}: {why}; the counts start again", Counts::FILE),
		}
		let unix = now().unix;
		let (game_rows, app_rows) = (self.session.store().rows(), self.session.app_rows().rows());
		let earliest = game_rows.iter().chain(&app_rows).map(|r| i64::from(r.ts)).min();
		let desk_pumped = telemetry::first_pump(&self.counts_rows_of(Source::Desktop)).is_some();
		let game_pumped = telemetry::first_pump(&self.counts_rows_of(Source::Game)).is_some();
		let kept = &mut self.counts.kept;
		kept.first_run = Some(if loaded == Loaded::First { unix } else { earliest.map_or(unix, |e| e.min(unix)) });
		kept.game_first = game_rows.iter().filter(|r| game(r)).map(|r| i64::from(r.ts)).min();
		kept.first_pump.desktop = desk_pumped || loaded != Loaded::First;
		kept.first_pump.game = game_pumped;
		kept.install_due.desktop = loaded == Loaded::First;
		kept.seen = telemetry::Seen { game: game_rows.len(), app: app_rows.len() };
		self.counts.dirty = true;
	}

	/// In the start, once the stores are read and the switches answered (`counts_load` ran first): the build's crash
	/// noted and the finished days sent; with the switch off, the kept days and the outbox emptied instead.
	pub(super) fn counts_start(&mut self, crashed: bool) {
		if !self.usage_on() {
			self.counts_off();
			return;
		}
		let today = self.counts_day(now().unix);
		self.counts.today = Some(today);
		self.counts_install(today);
		if crashed {
			self.counts.kept.facts(Source::Desktop, today).crashed = true;
		}
		self.counts_turn();
	}

	/// Usage statistics turned off, or read off at the start: the kept days go and so does every batch in the outbox.
	pub(crate) fn counts_off(&mut self) {
		if !self.counts.kept.days.is_empty() {
			self.counts.kept.clear();
			self.counts.dirty = true;
		}
		if let Some(outbox) = self.counts.sender.outbox.as_mut() {
			match outbox.clear() {
				Ok((deleted, notes)) => {
					if !deleted.is_empty() {
						log::info!("counts: {} batches deleted unsent: Usage statistics is off", deleted.len());
					}
					for note in notes {
						log::warn!("counts: {note}");
					}
				}
				Err(e) => log::warn!("counts: the outbox could not be emptied ({e})"),
			}
		}
		self.counts_save();
	}

	/// Usage statistics turned on: the days from now on count (none before: they were counted with it off), and the
	/// stores' rows are looked at from where they stand.
	pub(crate) fn counts_resume(&mut self) {
		let today = self.counts_day(now().unix);
		let kept = &mut self.counts.kept;
		kept.sent_through = Some(kept.sent_through.map_or(today - 1, |s| s.max(today - 1)));
		kept.seen = telemetry::Seen {
			game: self.session.store().records().len(),
			app: self.session.app_rows().records().len(),
		};
		self.counts.today = Some(today);
		self.counts_install(today);
		self.counts.dirty = true;
		self.counts_save();
	}

	/// Each source's install, once, on the first game day the switch reads on.
	fn counts_install(&mut self, today: i64) {
		for source in Source::ALL {
			if std::mem::take(self.counts.kept.install_due.of_mut(source)) {
				self.counts.kept.facts(source, today).install = true;
				self.counts.dirty = true;
			}
		}
	}

	/// The kept facts into `counts.json`, on the writer thread, when they changed.
	pub(super) fn counts_save(&mut self) {
		if !std::mem::take(&mut self.counts.dirty) {
			return;
		}
		match serde_json::to_vec(&self.counts.kept) {
			Ok(bytes) => self.save_file(Counts::FILE, bytes),
			Err(e) => log::warn!("{}: {e}", Counts::FILE),
		}
	}

	/// Today's facts of a source, while the switch reads on; none while it is off.
	fn counts_today(&mut self, source: Source) -> Option<&mut Facts> {
		if !self.usage_on() {
			return None;
		}
		let today = self.counts_day(now().unix);
		self.counts.dirty = true;
		Some(self.counts.kept.facts(source, today))
	}

	/// A source in use today: its first activity of the day, and a return after `telemetry::RETURN_DAYS` or more away.
	/// The facts a later point reads are kept whatever the switch says (the last day used, for `return`; the game's
	/// first sight, for its cohort, its age and its install, counted on the first game day the switch reads on); the
	/// day's own facts only while it reads on.
	pub(crate) fn counts_active(&mut self, source: Source) {
		let today = self.counts_day(now().unix);
		if source == Source::Game && self.counts.kept.game_first.is_none() {
			self.counts.kept.game_first = Some(now().unix);
			self.counts.kept.install_due.game = true;
			self.counts.dirty = true;
		}
		if !self.usage_on() {
			if self.counts.kept.used(source, today) {
				self.counts.dirty = true;
			}
			return;
		}
		if self.counts.kept.active(source, today) {
			self.counts.dirty = true;
			if source == Source::Game {
				// the moves owed so far today, for the day's end, as each row the game writes moves them
				let owed = u32::try_from(self.session.store().owed().len()).unwrap_or(u32::MAX);
				self.counts.kept.facts(Source::Game, today).owed = Some(owed);
			}
		}
		self.counts_install(today);
	}

	/// A feature of the desktop app used today (`feature_day`'s list, or `fitness_feature_day`'s for the strap and the
	/// walking pad: `telemetry::feature_point`; a name outside both, from the page, is dropped with a log line); its use
	/// is the app in use too.
	pub(crate) fn counts_feature(&mut self, feature: &str) {
		if telemetry::feature_point(feature).is_none() {
			log::warn!("counts: {feature:?} is no feature of the catalog: dropped");
			return;
		}
		self.counts_active(Source::Desktop);
		if let Some(facts) = self.counts_today(Source::Desktop) {
			facts.features.insert(feature.to_owned());
		}
	}

	/// A step's card opened in the Book's trees (`tree_day`'s `cards`), with its Start's refused or greyed reason
	/// (`refused_<reason>`: setup, off, game, reading; a pain flag or felt unwell counted under no name, so the page
	/// sends none for them); the tree in use today too (`feature_day`'s `tree`).
	pub(crate) fn counts_tree_card(&mut self, refused: Option<&str>) {
		self.counts_feature("tree");
		if let Some(facts) = self.counts_today(Source::Desktop) {
			facts.tree.card(refused);
		}
	}

	/// A Start from a step's card or a move's window that composed its pump, by its kind (`tree_day`'s
	/// `starts_<kind>`, `ns.StartFor`'s `as`); a kind outside the catalog's is dropped with a log line.
	pub(crate) fn counts_tree_start(&mut self, kind: &str) {
		self.counts_active(Source::Desktop);
		if let Some(facts) = self.counts_today(Source::Desktop)
			&& !facts.tree.start(kind)
		{
			log::warn!("counts: {kind:?} is no start of tree_day: dropped");
		}
	}

	/// A point of something that happened (a pairing, a match offer), into today's facts of the source.
	pub(crate) fn counts_event(&mut self, source: Source, point: Option<Point>) {
		if let (Some(point), Some(facts)) = (point, self.counts_today(source)) {
			facts.events.push(point);
		}
	}

	/// A pairing of the computer's (`pairing`): a phone's with its code, the strap's or the walking pad's first
	/// connection after the player turned it on; `failed` names why one did not.
	pub(crate) fn counts_pairing(&mut self, target: &str, failed: Option<&str>) {
		let (result, reason) = failed.map_or(("done", "none"), |why| ("failed", why));
		let point = Point::new("pairing", &[("target", target), ("result", result), ("reason", reason)], &[]);
		self.counts_event(Source::Desktop, point);
	}

	/// The strap or the walking pad turned on by the player (a pairing's attempt, until it connects or fails), or off.
	pub(crate) fn counts_trying(&mut self, kind: Kind, on: bool) {
		self.counts.trying.retain(|k| *k != kind);
		if on {
			self.counts.trying.push(kind);
		}
	}

	/// What the Bluetooth task says of a device: in use today once it connects, and an attempt's end once it connects,
	/// finds none, is refused or fails.
	pub(crate) fn counts_ble(&mut self, kind: Kind, state: &bluetooth::State) {
		let failed = match state {
			bluetooth::State::Connected(_) => None,
			bluetooth::State::NotFound => Some("unreachable"),
			bluetooth::State::Refused => Some("refused"),
			bluetooth::State::Failed(_) | bluetooth::State::BluetoothOff => Some("other"),
			bluetooth::State::Searching | bluetooth::State::Waiting => return,
		};
		let target = match kind {
			Kind::Strap => "strap",
			Kind::Pad => "pad",
		};
		// read today: the feature in use
		if failed.is_none() {
			self.counts_feature(target);
		}
		if self.counts.trying.contains(&kind) {
			self.counts_trying(kind, false);
			self.counts_pairing(target, failed);
		}
	}

	/// A match offer on the computer (`game_offer_desk`): shown, started or dismissed by the player (no cap holds one
	/// back since slice 48, so `capped` comes only from older apps); `link` its game's links.json key, a picked or found
	/// program's `custom`. An offer shown or started is the games in use too.
	pub(crate) fn counts_offer(&mut self, link: &str, result: &str) {
		let game = match link {
			"counterstrike" => "cs2",
			"dota" => "dota2",
			"league" => "lol",
			"rocketleague" => "rocket",
			"starcraft" => "sc2",
			"mtgarena" => "mtga",
			"hearthstone" => "hearthstone",
			"runeterra" => "lor",
			"formulaone" => "f1",
			"forza" => "forza",
			"iracing" => "iracing",
			"assettocorsa" => "acc",
			"elite" => "elite",
			"mechabellum" => "mechabellum",
			"slaythespire" => "sts2",
			"assettocorsaone" => "ac",
			"guildwars" => "gw2",
			"pathofexile" => "poe",
			"warframe" => "warframe",
			"warships" => "wows",
			"siege" => "r6",
			"tarkov" => "eft",
			"deadlock" => "deadlock",
			"fortnite" => "fortnite",
			"teamfortress" => "tf2",
			"naraka" => "naraka",
			"fallguys" => "fallguys",
			"deadbydaylight" => "dbd",
			"hunt" => "hunt",
			"valorant" => "valorant",
			"pubg" => "pubg",
			"halo" => "mcc",
			"brawlhalla" => "brawlhalla",
			"rfactor" => "rf2",
			"lemans" => "lmu",
			"automobilista" => "ams2",
			"osu" => "osu",
			"stardew" => "stardew",
			"warcraft" => "wc3",
			"eldenring" => "eldenring",
			"nightreign" => "nightreign",
			_ => "custom",
		};
		if result == "shown" || result == "started" {
			self.counts_feature("games");
		}
		let point = Point::new("game_offer_desk", &[("game", game), ("result", result)], &[]);
		self.counts_event(Source::Desktop, point);
	}

	/// The capture as it stands while the game runs (`capture_day`: the day keeps the most telling state).
	pub(crate) fn counts_capture(&mut self, state: &str) {
		if self.game.is_none() {
			return;
		}
		let os = if cfg!(windows) { "win" } else { "mac" };
		// no stream started yet in this run: no backend
		let backend = if self.sighting.backend.is_empty() { "none" } else { self.sighting.backend };
		if let Some(facts) = self.counts_today(Source::Desktop) {
			facts.capture(os, backend, state);
		}
	}

	/// The link as it stands (`link_day`, on the link's 5-s tick): `never` without a paired phone, `unreachable` while
	/// the LAN API cannot listen, `ok` once a device linked, else `dropped`.
	pub(crate) fn counts_link(&mut self) {
		let state = if self.settings.pairing.phone.is_none() {
			"never"
		} else if self.served.is_none() {
			"unreachable"
		} else if self.linked.is_some() {
			"ok"
		} else {
			"dropped"
		};
		let today = self.counts_day(now().unix);
		let kept = self.counts.kept.days.get(&today).and_then(|d| d.desktop.link.as_deref());
		if kept == Some(state) || kept == Some("ok") {
			return;
		}
		if let Some(facts) = self.counts_today(Source::Desktop) {
			facts.link(state);
		}
	}

	/// The Bluetooth link on a game day a phone is paired (`link_ble`, on the link's 5-s tick, slice 43): the helper
	/// block's `ble` as it stands now, the day keeping the last word said.
	pub(crate) fn counts_link_ble(&mut self, ble: Option<Ble>) {
		if self.settings.pairing.phone.is_none() {
			return;
		}
		let word = ble.map(Ble::word);
		if self.link_ble_today().is_some_and(|day| day.ble.as_deref() == word) {
			return;
		}
		if let Some(facts) = self.counts_today(Source::Desktop) {
			facts.link_ble.get_or_insert_default().ble = word.map(str::to_owned);
		}
	}

	/// A phone's or a watch's sync came on a transport while a phone is paired (`link_ble`'s `transport`, and `link`
	/// `linked` once one came over Bluetooth): each transport once a day.
	pub(crate) fn counts_transport(&mut self, transport: Transport) {
		if self.settings.pairing.phone.is_none() {
			return;
		}
		let seen = |day: &LinkBle| match transport {
			Transport::Lan => day.lan,
			Transport::Ble => day.over_ble,
		};
		if self.link_ble_today().is_some_and(seen) {
			return;
		}
		if let Some(facts) = self.counts_today(Source::Desktop) {
			let day = facts.link_ble.get_or_insert_default();
			match transport {
				Transport::Lan => day.lan = true,
				Transport::Ble => day.over_ble = true,
			}
		}
	}

	/// Today's `link_ble` facts as kept, without marking the counts changed.
	fn link_ble_today(&self) -> Option<&LinkBle> {
		let today = self.counts_day(now().unix);
		self.counts.kept.days.get(&today).and_then(|d| d.desktop.link_ble.as_ref())
	}

	/// The rows the stores took since the last look, each once: a row of the computer's or of the game's dated today
	/// is that source in use (its last day used kept whatever the switch says, `counts_active`); while the switch reads
	/// on, a skip of an unfinished move counts today for the source that made it when it was made today, and the moves
	/// owed now are the game's at the day's end so far. A skip's row repeats its owed move's time (`ns.PaidRow`), so
	/// its own moment is known only for one taken as it was made (`made_here`) or owed today: one taken in later from a
	/// file (a save, the mirror, a restore) with an older move counts on no day. A store that shrank (a reset compacts
	/// it) is looked at from where it stands.
	pub(crate) fn counts_rows(&mut self) {
		let (game_len, app_len) = (self.session.store().records().len(), self.session.app_rows().records().len());
		let seen = self.counts.kept.seen;
		if seen.game == game_len && seen.app == app_len {
			return;
		}
		let today = self.counts_day(now().unix);
		let records = self.session.store().records();
		let apps = self.session.app_rows().records();
		let taken: Vec<(Row, bool)> = records
			.get(seen.game.min(game_len)..)
			.unwrap_or_default()
			.iter()
			.chain(apps.get(seen.app.min(app_len)..).unwrap_or_default().iter().filter(|r| ran_here(r)))
			.map(|record| (record.row(), made_here(record)))
			.collect();
		self.counts.kept.seen = telemetry::Seen { game: game_len, app: app_len };
		self.counts.dirty = true;
		let on = self.usage_on();
		for (r, made) in &taken {
			let source = if computer(r) {
				Source::Desktop
			} else if game(r) {
				Source::Game
			} else {
				continue; // a phone's or a watch's: theirs to count
			};
			let day = self.counts_day(i64::from(r.ts));
			if day == today {
				self.counts_active(source);
			}
			if on && skip(r) && (*made || day >= today) {
				self.counts.kept.facts(source, today).skipped += 1;
			}
		}
		if on {
			let owed = u32::try_from(self.session.store().owed().len()).unwrap_or(u32::MAX);
			self.counts.kept.facts(Source::Game, today).owed = Some(owed);
		}
	}

	/// Every row the stores hold of a source: the computer's own pumps and its catch-ups, or the game's.
	fn counts_rows_of(&self, source: Source) -> Vec<Row> {
		let game_rows = self.session.store().rows();
		match source {
			Source::Desktop => {
				let apps = self.session.app_rows().records().iter().filter(|r| ran_here(r));
				let mut rows: Vec<Row> = apps.map(RowRecord::row).collect();
				rows.extend(game_rows.into_iter().filter(computer));
				rows
			}
			Source::Game => game_rows.into_iter().filter(game).collect(),
		}
	}

	/// The game day's turn (checked on the minute's tick) and the start: the finished days go while the switch reads on;
	/// what changed is kept either way.
	pub(super) fn counts_tick(&mut self) {
		let today = self.counts_day(now().unix);
		if self.usage_on() && self.counts.today != Some(today) {
			self.counts.today = Some(today);
			self.counts_turn();
		}
		self.counts_save();
	}

	/// Every finished game day not sent yet becomes its batches, into the outbox and out; in a week whose weekly
	/// points have not gone, the engine is asked first (the game's review by the addon's own rules, the focus in force),
	/// and the batches wait for its answer.
	fn counts_turn(&mut self) {
		if self.counts.asking {
			return;
		}
		let today = self.counts_day(now().unix);
		let week = telemetry::iso_week(today);
		let kept = &self.counts.kept;
		let due = week.is_some()
			&& kept.days.keys().any(|d| *d < today)
			&& (kept.week_sent.desktop != week || kept.week_sent.game != week);
		if !due {
			self.counts_batches(&Weekly::default());
			return;
		}
		self.counts.asking = true;
		let game = !self.session.store().records().is_empty();
		let (at, reset) = (now().unix, history::reset(self.logged_sheets()));
		let save = if game { self.game_save_cbor(&self.session.sheet(at)) } else { None };
		self.engine_call(
			move |e| {
				if game && let Err(err) = e.game_save(save.as_deref()) {
					log::warn!("counts: the game's save for its fold: {err}");
				}
				let review = if game { e.feedback_numbers(at, reset)? } else { None };
				let focus =
					e.summary(at)?.get("fc").and_then(serde_json::Value::as_u64).and_then(|f| u8::try_from(f).ok());
				Ok(Weekly {
					review: review.map(|n| Review { finished: n.finished, banked: n.banked, max_sets: n.max_sets }),
					focus,
				})
			},
			|h, answer| {
				h.counts.asking = false;
				let weekly = answer.unwrap_or_else(|e| {
					log::warn!("counts: the engine's weekly numbers: {e}");
					Weekly::default()
				});
				h.counts_batches(&weekly);
			},
		);
	}

	/// The finished days' batches into the outbox, the weekly points with the last of them while this ISO week's have
	/// not gone (`weekly`: what the engine said), then a round of the outbox.
	fn counts_batches(&mut self, weekly: &Weekly) {
		if !self.usage_on() {
			return;
		}
		let today = self.counts_day(now().unix);
		let days = self.counts.kept.finished(today);
		let last = days.last().map(|(d, _)| *d);
		let strength = &facts().library.strength_order;
		let (desk_rows, game_rows) = (self.counts_rows_of(Source::Desktop), self.counts_rows_of(Source::Game));
		let desk_tally = telemetry::tally(&desk_rows, |t| self.counts_day(t), strength);
		let game_tally = telemetry::tally(&game_rows, |t| self.counts_day(t), strength);
		let week = telemetry::iso_week(today);
		for (day, facts) in days {
			let Some(name) = telemetry::day_name(day) else { continue };
			let mut desk = self.desk_points(day, &facts.desktop, desk_tally.get(&day), &desk_rows);
			let mut game = self.game_points(day, &facts.game, game_tally.get(&day), &game_rows);
			if Some(day) == last && self.counts.kept.week_sent.desktop != week {
				let first = self.counts.kept.first_run.map_or(today, |t| self.counts_day(t));
				desk.extend(Review::of(&desk_tally, today, facts_window()).point(today - first));
				desk.extend(self.desk_setup(weekly.focus));
				self.counts.kept.week_sent.desktop.clone_from(&week);
			}
			if Some(day) == last && self.counts.kept.week_sent.game != week {
				let first = self.counts.kept.game_first.map_or(today, |t| self.counts_day(t));
				game.extend(weekly.review.and_then(|r| r.point(today - first)));
				game.extend(self.game_setup());
				self.counts.kept.week_sent.game.clone_from(&week);
			}
			// where the player stands as it stands now, so with the last day alone, and only from a computer no
			// phone was paired with that day (a paired phone sends its own: one player counts once a day)
			let unpaired = facts.desktop.link.as_deref().is_none_or(|l| l == "never");
			if Some(day) == last && facts.desktop.active && unpaired && self.settings.pairing.phone.is_none() {
				desk.extend(self.desk_standing());
			}
			self.counts_keep(Source::Desktop, &name, &desk);
			self.counts_keep(Source::Game, &name, &game);
			self.counts.kept.sent_through = Some(self.counts.kept.sent_through.map_or(day, |s| s.max(day)));
			self.counts.dirty = true;
		}
		self.counts_save();
		self.send_counts();
	}

	/// The `desktop` source's points of a finished game day: from its facts, and from the rows of that day the
	/// computer's pumps wrote.
	fn desk_points(&mut self, day: i64, f: &Facts, tally: Option<&DayTally>, rows: &[Row]) -> Vec<Point> {
		let mut points = Vec::new();
		let first = self.counts.kept.first_run.map(|t| self.counts_day(t));
		if f.install {
			points.extend(Point::new("install", &[], &[]));
		}
		if f.active {
			points.extend(active(day, first));
			let tier = if self.pro_state() == Some(ProState::On) { "pro" } else { "free" };
			points.extend(Point::new("build", &[("tier", tier)], &[("crashed", if f.crashed { 1.0 } else { 0.0 })]));
		}
		points.extend(f.events.iter().cloned());
		if let Some(link) = &f.link {
			points.extend(Point::new("link_day", &[("state", link)], &[]));
		}
		points.extend(f.link_ble.as_ref().and_then(LinkBle::point));
		if let Some(c) = &f.capture {
			points.extend(Point::new(
				"capture_day",
				&[("os", &c.os), ("backend", &c.backend), ("state", &c.state)],
				&[],
			));
		}
		for feature in &f.features {
			points.extend(telemetry::feature_point(feature).and_then(|p| Point::new(p, &[("feature", feature)], &[])));
		}
		points.extend(f.tree.point());
		points.extend(returned(f));
		points.extend(self.counts_first_pump(Source::Desktop, day, rows, "computer"));
		// the computer's own pumps owe nothing: its unfinished moves are the game's
		points.extend(day_pumps("computer", tally, f.skipped, None));
		points.extend(move_quality(tally));
		points
	}

	/// The `game` source's points of a finished game day: from its facts, and from the game's rows of that day.
	fn game_points(&mut self, day: i64, f: &Facts, tally: Option<&DayTally>, rows: &[Row]) -> Vec<Point> {
		let mut points = Vec::new();
		let first = self.counts.kept.game_first.map(|t| self.counts_day(t));
		if f.install {
			points.extend(Point::new("install", &[], &[]));
		}
		if f.active {
			points.extend(active(day, first));
		}
		points.extend(returned(f));
		points.extend(self.counts_first_pump(Source::Game, day, rows, "game"));
		points.extend(day_pumps("game", tally, f.skipped, f.active.then_some(f.owed.unwrap_or(0))));
		points.extend(move_quality(tally));
		points
	}

	/// `first_pump` once a source: the first finished pump of its rows when it came on `day` (its delay from the first
	/// run, or the game's first sight); one that came on an earlier day (counted while the switch was off, or before
	/// this build counted) is passed by, none sent.
	fn counts_first_pump(&mut self, source: Source, day: i64, rows: &[Row], surface: &str) -> Option<Point> {
		if *self.counts.kept.first_pump.of(source) {
			return None;
		}
		let at = telemetry::first_pump(rows)?;
		let pump_day = self.counts_day(at);
		if pump_day > day {
			return None;
		}
		*self.counts.kept.first_pump.of_mut(source) = true;
		let since = match source {
			Source::Desktop => self.counts.kept.first_run,
			Source::Game => self.counts.kept.game_first,
		}?;
		(pump_day == day && at >= since)
			.then(|| Point::new("first_pump", &[("surface", surface), ("delay", telemetry::delay(at - since))], &[]))
			.flatten()
	}

	/// This computer's setup (`setup_week`): the app's pumps at the balanced pace (an app pump's, apppump.lua), the
	/// focus in force (`focus`, the engine's), the place's floor, room and quiet, and how much equipment it holds.
	fn desk_setup(&self, focus: Option<u8>) -> Option<Point> {
		let (_, place) = self.place_in_use();
		let focus = focus.and_then(|id| facts().focus(id)).map_or("unset", |f| f.key.as_str());
		Point::new(
			"setup_week",
			&[
				("pace", "balanced"),
				("focus", focus),
				("floor", telemetry::yes(place.floor)),
				("room", telemetry::yes(place.space)),
				("quiet", telemetry::yes(place.quiet)),
				("equipment", telemetry::equipment(place.gear.len())),
			],
			&[],
		)
	}

	/// Where the player stands (`standing`, 2026-10-21; contract/standing-api.yaml): the best streak the streak page
	/// shows and the highest colour on the Book's colours card, as they stand now; none before the Book's first answer.
	fn desk_standing(&self) -> Option<Point> {
		let top = self.book_top_colour()?.to_string();
		let at = now().unix;
		let best = self.board(&self.session.sheet(at), at).best;
		Point::new("standing", &[("streak_best", telemetry::streak_best(best)), ("colour_top", &top)], &[])
	}

	/// The game's setup (`setup_week`): the profile of the install the app follows as the strip or a file last showed
	/// it (its Default, or its only one), at the addon's defaults where it says nothing, and the focus of the game's
	/// newest sheet; none without one.
	fn game_setup(&self) -> Option<Point> {
		let install = self.followed.as_deref().and_then(install_key)?;
		let profiles = &self.session.people().installs.get(&install)?.profiles;
		let profile = profiles
			.get(DEFAULT_PROFILE)
			.or_else(|| (profiles.len() == 1).then(|| profiles.values().next()).flatten())?;
		let field = |key: &str| questpump_core::blob::get(profile, key);
		let switch = |key: &str, default: bool| field(key).and_then(ciborium::Value::as_bool).unwrap_or(default);
		// the game's own value, kept only when the catalog lists it (DB.lua's default otherwise)
		let pace = field("pace")
			.and_then(questpump_core::blob::as_text)
			.filter(|p| telemetry::listed("setup_week", "pace", p))
			.unwrap_or("balanced");
		let gear = match field("gear") {
			Some(ciborium::Value::Map(m)) => m.iter().filter(|(_, on)| on.as_bool().unwrap_or(false)).count(),
			_ => 0,
		};
		let focus = self
			.session
			.people()
			.sheets
			.values()
			.filter_map(|sheet| crate::desk::hero::Summary::read(&questpump_core::blob::to_json(sheet)))
			.max_by_key(|s| s.at)
			.and_then(|s| s.focus)
			.and_then(|id| facts().focus(id))
			.map_or("unset", |f| f.key.as_str());
		Point::new(
			"setup_week",
			&[
				("pace", pace),
				("focus", focus),
				("floor", telemetry::yes(switch("floor", true))),
				("room", telemetry::yes(switch("space", true))),
				("quiet", telemetry::yes(switch("quiet", false))),
				("equipment", telemetry::equipment(gear)),
			],
			&[],
		)
	}

	/// One source's points of one game day into the outbox, a batch at most 40 points (`c-<source>-<day>-<n>`).
	fn counts_keep(&mut self, source: Source, day: &str, points: &[Point]) {
		let version = env!("CARGO_PKG_VERSION");
		for (n, body) in telemetry::bodies(source, version, day, points).into_iter().enumerate() {
			let id = format!("c-{}-{day}-{n}", source.name());
			let kept = match self.counts.sender.outbox.as_mut() {
				Some(outbox) => outbox.keep(&id, &body, false, now().unix).map_err(|e| e.to_string()),
				None => Err("no outbox".to_owned()),
			};
			match kept {
				Ok(_) => log::info!("counts: {id} kept to send"),
				Err(e) => log::warn!("counts: {id} could not be kept ({e}): it is lost"),
			}
		}
	}

	/// Posts what waits in the outbox while the switch reads on, unless a batch is out (then another follows it) or the
	/// round a minute after a batch the route slowed is still to come, which carries what waits: a backlog of finished
	/// days goes one batch a minute, and neither the outbox's tick nor a day's turn posts into the route's minute for a
	/// 429. (The feedback's and the crash reports' rounds post at once: a player's Send there waits for its answer.)
	pub(super) fn send_counts(&mut self) {
		if self.counts.sender.held() {
			return;
		}
		self.counts.sender.soon = None;
		if !self.usage_on() {
			return;
		}
		if self.counts.sender.batch.is_some() {
			self.counts.sender.again = true;
			return;
		}
		let Some(waiting) = self.counts.sender.waiting() else { return };
		if !waiting.messages.is_empty() && self.counts.sender.can_post() {
			self.counts.sender.post(waiting.messages);
		}
	}

	/// The round `OUTBOX_SOON` after a batch the route slowed, once its time came (checked on the 2-s game tick).
	pub(super) fn counts_due(&mut self) {
		if self.counts.sender.due() {
			self.send_counts();
		}
	}

	/// What the batch out reported: a batch's answer, or its end.
	pub(super) fn on_counts(&mut self, news: News) {
		match news {
			News::Posted(_, id, posted) => match posted.map(|status| (status, self.counts.sender.answer(status))) {
				Ok((_, Answer::Sent)) => {
					log::info!("counts: {id} sent");
					self.counts.sender.remove(&id);
				}
				Ok((status, Answer::Refused)) => {
					log::warn!("counts: {id} refused by the route ({status}): dropped");
					self.counts.sender.remove(&id);
				}
				Ok((status, _)) => log::info!("counts: {id} not sent yet (the route answered {status}); it goes later"),
				Err(e) => log::info!("counts: {id} not sent yet ({e}); it goes later"),
			},
			News::Done(_, slowed) => {
				let (_, again) = self.counts.sender.ended(slowed);
				if again {
					self.send_counts();
				}
			}
		}
	}
}

/// What the engine says once a week: the game's review by the addon's own `ns.FeedbackNumbers` (none without the
/// game's rows) and the focus in force on the computer (the app's own sheet's `fc`; none before a focus row).
#[derive(Default)]
struct Weekly {
	review: Option<Review>,
	focus: Option<u8>,
}

/// The review's window, library.json's `feedback_days`.
fn facts_window() -> i64 {
	facts().constants().feedback_days
}

/// `active`: the cohort's ISO week and the game days since the first run (`first`, its game day).
fn active(day: i64, first: Option<i64>) -> Option<Point> {
	let first = first.unwrap_or(day);
	let week = telemetry::iso_week(first)?;
	Point::new("active", &[("cohort_week", &week), ("age", telemetry::age(day - first))], &[])
}

/// `return`: the days away before the day's first activity.
fn returned(f: &Facts) -> Option<Point> {
	let away = telemetry::away(f.away?)?;
	Point::new("return", &[("away", away)], &[])
}

/// `day_pumps` of a surface: its pumps finished and banked, the unfinished moves skipped and the moves owed at the
/// day's end (`owed`: none for a surface that owes nothing, or the game on a day it was not played); none on a day
/// without a pump or a skip, unless the game was played and owed moves at its end.
fn day_pumps(surface: &str, tally: Option<&DayTally>, skipped: u32, owed: Option<u32>) -> Option<Point> {
	let (finished, banked) = tally.map_or((0, 0), |t| (t.finished, t.banked));
	let owed = owed.unwrap_or(0);
	if finished + banked + skipped + owed == 0 {
		return None;
	}
	Point::new(
		"day_pumps",
		&[("surface", surface)],
		&[
			("finished", Point::count(finished)),
			("banked", Point::count(banked)),
			("skipped_moves", Point::count(skipped)),
			("unfinished_end", Point::count(owed)),
		],
	)
}

/// `move_quality`: each move with an easier pick or an early stop that day, by its library key (a move the library
/// lacks, a newer addon's, counts on none).
fn move_quality(tally: Option<&DayTally>) -> Vec<Point> {
	let Some(tally) = tally else { return Vec::new() };
	let named: BTreeMap<&str, (u32, u32)> =
		tally.moves.iter().filter_map(|(id, n)| Some((facts().text(*id)?.key.as_str(), *n))).collect();
	named
		.into_iter()
		.filter_map(|(key, (easier, stopped))| {
			Point::new(
				"move_quality",
				&[("move", key)],
				&[("easier", Point::count(easier)), ("stopped", Point::count(stopped))],
			)
		})
		.collect()
}
