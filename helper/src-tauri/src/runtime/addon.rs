//! The addon in the game, kept by QuestPump on this computer (docs/companion.md, "The addon on this computer";
//! `crate::addon` fetches, downloads, unpacks and swaps off the loop, questpump-core `addon` holds the rules). The loop
//! keeps the newest verified feed (an older one that comes later is a replay and is dropped), carries its version,
//! date and `phone_apps` in the mirror's pairing block with who manages the install the mirror goes into (`addon_by`),
//! and decides what to write and when:
//!
//! - **Ownership.** The helper keeps a record of each install it made (`addon-installs.json`: the AddOns folder, the
//!   version, every file's sha256). An install is managed (`helper`) only while its folders hold exactly the
//!   record's files; one another updater rewrote no longer matches, and the helper lets it go (`other`, the record
//!   dropped). A folder that is or holds a link or a `.git` (a developer's `make link` folder) is never written and
//!   never managed. The folders are read off the loop when the followed install changes, at a verified feed whose
//!   version they were not read under yet (`read_under`: the hourly feed of the same version reads nothing), and again
//!   right before any swap; a read that a job's start or end overtook (`epoch`) is dropped and made again, so a look
//!   from before a swap never drops the record the swap wrote.
//! - **The media packs** (slice 71): where QuestPump is in the followed install and a pack the feed holds is missing,
//!   or one the helper itself put in is behind the feed's, a packs job adds or replaces those folders alone
//!   (`rules::Scope::Packs`), whoever keeps QuestPump itself; a pack another updater put in is never touched. Every
//!   job puts in only the folders that differ from the feed's.
//! - **A first install**: the player's Add QuestPump to the game (the welcome's game row, Pump home's card, This
//!   computer) while the game on this computer has no QuestPump; allowed while the game runs (it only adds folders),
//!   after which the line says to start the game again until that client is gone. The install is a folder that holds
//!   none yet, picked by its client's version: a folder the player picked whose number the feed lists; else one whose
//!   number is the feed's first interface (Forever's), else one whose number the feed lists, of those the running
//!   client's, else the preferred one (`install::preferred`: Forever's live install, never its beta beside it); with
//!   none, the folder the player picks (the folder picker opens).
//! - **Updates**: a managed install whose version is older than the feed's is staged by itself (the feed's files staged
//!   beside AddOns file by file while the game may run, `crate::addon`) and swapped in once the game is closed and no
//!   pump runs on this computer (`addon_held`, checked again right before the swap's first rename), before any update
//!   of QuestPump itself (`addon_busy` holds that one, the cleaning up after a swap included); an install another
//!   updater made shows Update now on This computer, which makes it managed. An update staged for an install another
//!   updater took over meanwhile is dropped, and no swap writes an install without a record unless it is a first
//!   install or the player asked. A failed version is not tried again by itself until the next check of the feed, and
//!   This computer offers Update now beside the failure; a refused write on Windows shows This computer's grant card
//!   (`runtime/folder.rs`).
//! - **A swap cut short** (a quit, a crash, a power cut) leaves the staging folder: at the start and at every look while
//!   no job runs, the staging folder of a recorded install or of the game on this computer is put right first
//!   (`addon::recover`, a `Settling` job), so the addon a swap moved out is back before the game or the player needs it.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use questpump_core::addon::{self as rules, Feed, Record, Tree};
use questpump_core::mirror;
use questpump_desk::platform::FoundGame;
use serde::{Deserialize, Serialize};
use tauri::Manager as _;
use tokio::sync::mpsc;

use super::Helper;
use crate::addon::{self, Failure, News};
use crate::desk::audio;
use crate::feeds::Checks;
use crate::install;
use crate::ui::{self, AddonLine, AddonOffer, AddonView};

/// The install records in the data folder.
const RECORDS: &str = "addon-installs.json";
/// `addon_by`'s two words.
const BY_HELPER: &str = "helper";
const BY_OTHER: &str = "other";

/// Who an install's folders are by, as the helper last read them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Owner {
	/// The record's files, as the helper put them.
	Helper,
	/// Another updater's, or a hand install's.
	Other,
	/// A folder that is or holds a link or a `.git`: never written.
	Linked,
}

/// What the helper is doing to an install, with the feed it does it for (its files become the install's record).
#[derive(Debug, Clone, PartialEq, Eq)]
enum Job {
	Idle,
	/// The feed's files coming into the staging folder for this install (`crate::addon::stage`).
	/// `packs`: a job for the media packs alone (`rules::Scope::Packs`), in an install whose QuestPump is current or
	/// another updater's: a missing pack added, one of the helper's own brought up to the feed's.
	Staging {
		addons: PathBuf,
		feed: Box<Feed>,
		first: bool,
		packs: bool,
	},
	/// Staged, waiting for its moment.
	Staged {
		addons: PathBuf,
		staging: PathBuf,
		feed: Box<Feed>,
		first: bool,
		packs: bool,
	},
	/// The folders being swapped; staged again when the game or a pump started first (`News::Held`).
	Swapping {
		addons: PathBuf,
		staging: PathBuf,
		feed: Box<Feed>,
		first: bool,
		packs: bool,
	},
	/// The staging folder being put right and removed (`addon::recover`): after a swap or a failed staging, for a
	/// dropped update, or found while nothing ran.
	Settling {
		addons: PathBuf,
	},
}

/// What the player asked.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Asked {
	/// Add QuestPump to the game: waiting for the feed, or for the folder picker's answer.
	Add,
	/// Update now, for this install: it is managed from then on.
	Update(PathBuf),
}

/// A step that failed, kept for the line and so the version is not tried again by itself until the next feed.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Failed {
	addons: Option<PathBuf>,
	version: Option<String>,
	first: bool,
	error: String,
}

#[derive(Serialize, Deserialize, Default)]
struct Records {
	installs: Vec<Record>,
}

/// The addon's state in the loop.
pub(crate) struct AddonState {
	news_tx: mpsc::UnboundedSender<News>,
	checks: Option<Checks>,
	/// The newest verified feed this run.
	feed: Option<Feed>,
	records: Vec<Record>,
	owners: HashMap<PathBuf, Owner>,
	/// The feed's version each install's folders were last read under (`tree_read`): a verified feed of that version
	/// reads them no more (`feed_verified`).
	read_under: HashMap<PathBuf, String>,
	/// The installs with QuestPump in which a media pack is missing, or one of the helper's own is behind the feed's,
	/// as their folders were last read (`tree_read`): a packs job is due there.
	packs_due: HashSet<PathBuf>,
	/// When the folders were last read for a pack's folder gone since (`pack_gone`).
	pack_look: Option<Instant>,
	/// The installs whose folders are being read now.
	looking: HashSet<PathBuf>,
	/// The install whose ownership the pairing block states now.
	stated: Option<PathBuf>,
	job: Job,
	/// Moves on at every change of `job`: a folders' read from an earlier one is dropped.
	epoch: u64,
	/// The game or a pump runs, as of the last look: the swap's last check before its first rename.
	held: Arc<AtomicBool>,
	/// The installs whose staging folder did not settle this run: the look leaves them to the next staging.
	unsettled: HashSet<PathBuf>,
	asked: Option<Asked>,
	failed: Option<Failed>,
	/// The client that ran when QuestPump was added to its folder: it loads QuestPump once started again.
	added_while: Option<u32>,
	/// The last feed error logged, so the log says each once.
	feed_error: Option<String>,
	/// The feed was checked early once for the game found without QuestPump (`addon_game_found`).
	checked_early: bool,
}

impl AddonState {
	/// The state and the receiving end of its tasks' news.
	pub(crate) fn new() -> (Self, mpsc::UnboundedReceiver<News>) {
		let (news_tx, news) = mpsc::unbounded_channel();
		let state = Self {
			news_tx,
			checks: None,
			feed: None,
			records: Vec::new(),
			owners: HashMap::new(),
			read_under: HashMap::new(),
			packs_due: HashSet::new(),
			pack_look: None,
			looking: HashSet::new(),
			stated: None,
			job: Job::Idle,
			epoch: 0,
			held: Arc::new(AtomicBool::new(false)),
			unsettled: HashSet::new(),
			asked: None,
			failed: None,
			added_while: None,
			feed_error: None,
			checked_early: false,
		};
		(state, news)
	}

	fn record(&self, addons: &Path) -> Option<&Record> {
		self.records.iter().find(|r| r.addons == addons)
	}

	/// The job from now on.
	fn set_job(&mut self, job: Job) {
		self.epoch += 1;
		self.job = job;
	}

	/// The install an update or a first install is under way for (the cleaning up after it shows nothing).
	fn job_addons(&self) -> Option<&Path> {
		match &self.job {
			Job::Idle | Job::Settling { .. } => None,
			Job::Staging { addons, .. } | Job::Staged { addons, .. } | Job::Swapping { addons, .. } => Some(addons),
		}
	}

	fn job_first(&self) -> bool {
		matches!(
			&self.job,
			Job::Staging { first: true, .. } | Job::Staged { first: true, .. } | Job::Swapping { first: true, .. }
		)
	}

	/// The feed the job works for.
	fn job_feed(&self) -> Option<&Feed> {
		match &self.job {
			Job::Idle | Job::Settling { .. } => None,
			Job::Staging { feed, .. } | Job::Staged { feed, .. } | Job::Swapping { feed, .. } => Some(feed),
		}
	}
}

impl Helper {
	/// The start's part: the records read, a swap the last run left cut short put right, the feed's checks started.
	pub(super) fn addon_start(&mut self) {
		match mirror::read_data(&self.dir, RECORDS, |b| serde_json::from_slice::<Records>(b).map_err(|e| e.to_string()))
		{
			Ok(records) => self.addon.records = records.unwrap_or_default().installs,
			Err(e) => log::warn!("{RECORDS}: {e}"),
		}
		self.settle_due();
		self.addon.checks = Some(addon::spawn(&self.app, self.addon.news_tx.clone()));
	}

	fn save_records(&self) {
		let records = Records { installs: self.addon.records.clone() };
		match serde_json::to_vec_pretty(&records) {
			Ok(bytes) => self.save_file(RECORDS, bytes),
			Err(e) => log::warn!("{RECORDS}: {e}"),
		}
	}

	/// The feed checked now, unless a check succeeded lately (`feeds::Checks::now`).
	pub(super) fn check_feed(&self) {
		if let Some(checks) = &self.addon.checks {
			checks.now();
		}
	}

	/// What the addon's tasks say.
	pub(super) fn on_addon(&mut self, news: News) {
		match news {
			News::Feed(Ok(feed)) => self.feed_verified(feed),
			News::Feed(Err(e)) => {
				if self.addon.feed_error.as_ref() != Some(&e) {
					log::warn!("addon: the feed: {e}");
					self.addon.feed_error = Some(e.clone());
				}
				if self.addon.asked == Some(Asked::Add) && self.addon.feed.is_none() {
					self.addon.asked = None;
					self.addon.failed = Some(Failed { addons: None, version: None, first: true, error: e });
				}
			}
			News::Tree { addons, epoch, tree } => self.tree_read(addons, epoch, tree),
			News::Staged { addons, result } => self.staged(addons, result),
			News::Swapped { addons, result } => self.swapped(&addons, result),
			News::Held { addons } => self.swap_held(&addons),
			News::Settled { addons, result } => self.settled(addons, result),
		}
		self.addon_show();
	}

	/// A verified feed: kept unless an older one than the one kept (a replay), its words into the pairing block, the
	/// followed install read again unless it was read under this version (an hourly check of the same feed hashes no
	/// folder), and a first install the player asked for goes on.
	fn feed_verified(&mut self, feed: Feed) {
		self.addon.feed_error = None;
		if let Some(kept) = &self.addon.feed
			&& rules::newer(&kept.version, &feed.version)
		{
			log::warn!(
				"addon: the feed offers {}, older than {} seen before: kept {}",
				feed.version,
				kept.version,
				kept.version
			);
			return;
		}
		if self.addon.feed.as_ref().is_none_or(|kept| kept.version != feed.version) {
			log::info!("addon: the feed offers {} (phones in the stores: {})", feed.version, feed.phone_apps);
		}
		let p = &mut self.desk.settings.pairing;
		let said = (p.addon_latest.clone(), p.addon_out, p.phone_apps);
		p.addon_latest = Some(feed.version.clone());
		p.addon_out = feed.out_at();
		p.phone_apps = Some(feed.phone_apps);
		let changed = said != (p.addon_latest.clone(), p.addon_out, p.phone_apps);
		self.addon.feed = Some(feed);
		self.addon.failed = None;
		if changed {
			self.desk.save_settings();
			self.due_by(Instant::now());
			// the phone's step shows or hides with `phone_apps`
			self.desk.welcome_found();
		}
		if let Some(followed) = self.desk.p.followed.clone()
			&& self.addon.read_under.get(&followed) != self.addon.feed.as_ref().map(|f| &f.zip.sha256)
		{
			self.look_at(followed);
		}
		if self.addon.asked == Some(Asked::Add) {
			self.add_first();
		}
	}

	/// The folders of `addons` read off the loop, unless they are being read.
	fn look_at(&mut self, addons: PathBuf) {
		if self.addon.looking.insert(addons.clone()) {
			addon::look(addons, self.addon.epoch, self.addon.news_tx.clone());
		}
	}

	/// An install's folders came back: its owner, a record they no longer match let go, and an update staged for it
	/// dropped once it is no longer the helper's to make. A read that a job's start or end overtook (it may have
	/// changed the folders, or written the record) is made again.
	fn tree_read(&mut self, addons: PathBuf, epoch: u64, tree: Result<Tree, String>) {
		self.addon.looking.remove(&addons);
		if epoch != self.addon.epoch {
			self.look_at(addons);
			return;
		}
		let tree = match tree {
			Ok(tree) => tree,
			Err(e) => {
				log::warn!("addon: {}: {e}", addons.display());
				return;
			}
		};
		if let Some(feed) = &self.addon.feed {
			self.addon.read_under.insert(addons.clone(), feed.zip.sha256.clone());
		}
		let owner = match (&tree, self.addon.record(&addons)) {
			(Tree::Linked, _) => Owner::Linked,
			(_, Some(record)) if record.matches(&tree) && record.files.contains_key(rules::TOC) => Owner::Helper,
			_ => Owner::Other,
		};
		// an install the helper no longer keeps as a whole: its record goes, but for the media packs that still hold
		// the files the helper put in, which it goes on keeping current
		if owner != Owner::Helper
			&& let Some(record) = self.addon.record(&addons).cloned()
		{
			let kept = record.kept_packs(&tree);
			if kept != record.files {
				if record.files.contains_key(rules::TOC) {
					log::info!(
						"addon: {} changed since QuestPump installed it: another updater keeps it now",
						addons.display()
					);
				}
				self.addon.records.retain(|r| r.addons != addons);
				if !kept.is_empty() {
					self.addon.records.push(Record { files: kept, ..record });
				}
				self.save_records();
			}
		}
		let scope = self.scope(&addons, true);
		let due = owner != Owner::Linked
			&& matches!(&tree, Tree::Files(files) if files.contains_key(rules::TOC))
			&& self.addon.feed.as_ref().is_some_and(|feed| !scope.folders(feed, &tree).is_empty());
		if due {
			self.addon.packs_due.insert(addons.clone());
		} else {
			self.addon.packs_due.remove(&addons);
		}
		if self.addon.owners.get(&addons) != Some(&owner) {
			log::info!("addon: {} is {owner:?}'s", addons.display());
		}
		self.addon.owners.insert(addons.clone(), owner);
		if let Job::Staged { addons: at, first, packs, .. } = &self.addon.job
			&& *at == addons
			&& !self.update_wanted(&addons, *first || *packs)
		{
			log::info!("addon: the update staged for {} is dropped: QuestPump no longer keeps it", addons.display());
			self.settle(addons);
		}
		self.state_owner();
	}

	/// The pairing block says who manages the install the mirror goes into; a change goes into the next mirror.
	fn state_owner(&mut self) {
		let Some(followed) = self.desk.p.followed.clone() else { return };
		let Some(owner) = self.addon.owners.get(&followed).copied() else {
			self.look_at(followed);
			return;
		};
		self.addon.stated = Some(followed);
		let by = Some(if owner == Owner::Helper { BY_HELPER } else { BY_OTHER }.to_owned());
		if self.desk.settings.pairing.addon_by != by {
			self.desk.settings.pairing.addon_by = by;
			self.desk.save_settings();
			self.due_by(Instant::now());
		}
	}

	/// The 2-s look: the client QuestPump was added under gone, the followed install's owner, an update to stage or to
	/// swap in, and the line.
	pub(super) fn addon_look(&mut self) {
		self.addon.held.store(self.addon_held(), Ordering::Relaxed);
		if self.addon.added_while.is_some() && self.desk.p.game.as_ref().map(|g| g.pid) != self.addon.added_while {
			self.addon.added_while = None;
		}
		if self.desk.p.followed != self.addon.stated {
			self.state_owner();
		}
		self.settle_due();
		if self.addon.job == Job::Idle {
			self.stage_due();
		}
		if let Job::Staged { addons, staging, feed, first, packs } = self.addon.job.clone()
			&& (first || !self.addon_held())
		{
			self.swap_in(addons, staging, feed, first, packs);
		}
		self.addon_show();
	}

	/// What holds a staged update back: the game runs (the client reads the addon's files, its figures on demand), or
	/// a pump runs on this computer (it plays the voice from the game folder).
	fn addon_held(&self) -> bool {
		self.desk.p.game.is_some() || self.desk.desk_pump_running()
	}

	/// An addon job is under way, its cleaning up included: QuestPump's own update waits for it (`runtime/updates.rs`).
	pub(super) fn addon_busy(&self) -> bool {
		self.addon.job != Job::Idle
	}

	/// The followed install's version is older than the feed's and it is the helper's to update (managed, or the
	/// player's Update now), and the version did not just fail there: its files are staged (`crate::addon::stage`).
	/// Else, where QuestPump is there and a media pack is missing or one of the helper's own is behind the feed's
	/// (`packs_due`, whoever keeps QuestPump itself), a packs job is staged; a pack's folder gone since the folders
	/// were read has them read again.
	fn stage_due(&mut self) {
		let (Some(feed), Some(followed)) = (self.addon.feed.clone(), self.desk.p.followed.clone()) else { return };
		let asked = self.addon.asked == Some(Asked::Update(followed.clone()));
		let managed = self.addon.owners.get(&followed) == Some(&Owner::Helper);
		let installed = install::addons_state(&followed).questpump.unwrap_or_default();
		let update = (asked || managed) && rules::newer(&feed.version, &installed);
		if asked && !update {
			self.addon.asked = None;
		}
		let failed = self.addon.failed.as_ref().is_some_and(|f| {
			f.addons.as_ref() == Some(&followed) && f.version.as_ref() == Some(&feed.version) && !(asked && update)
		});
		if failed {
			return;
		}
		if update {
			self.stage(followed, feed, false, false);
		} else if self.addon.packs_due.contains(&followed) {
			self.stage(followed, feed, false, true);
		} else if self.pack_gone(&followed, &feed) {
			self.addon.pack_look = Some(Instant::now());
			self.look_at(followed);
		}
	}

	/// A media pack the feed holds has no folder in `followed`, where QuestPump is and no link: looked for at most once
	/// a minute, so a folder that cannot be read is not read at every look.
	fn pack_gone(&self, followed: &Path, feed: &Feed) -> bool {
		self.addon.owners.get(followed).is_some_and(|owner| *owner != Owner::Linked)
			&& self.addon.pack_look.is_none_or(|at| at.elapsed() >= Duration::from_secs(60))
			&& followed.join(rules::TOC).is_file()
			&& rules::PACKS.iter().any(|pack| {
				let folder = format!("{pack}/");
				!followed.join(pack).is_dir() && feed.files.iter().any(|f| f.path.starts_with(&folder))
			})
	}

	/// The folders a job puts in: every one that differs from the feed's, or, for a packs job, the missing packs and
	/// those the helper's record still holds.
	fn scope(&self, addons: &Path, packs: bool) -> rules::Scope {
		if packs {
			rules::Scope::Packs { kept: self.addon.record(addons).map(|r| r.files.clone()).unwrap_or_default() }
		} else {
			rules::Scope::Differing
		}
	}

	fn stage(&mut self, addons: PathBuf, feed: Feed, first: bool, packs: bool) {
		if packs {
			log::info!("addon: {} for {}: downloading its packs", feed.version, addons.display());
		} else {
			log::info!("addon: {} for {}: downloading", feed.version, addons.display());
		}
		self.addon.failed = None;
		self.addon.set_job(Job::Staging { addons: addons.clone(), feed: Box::new(feed.clone()), first, packs });
		// the desktop app's own copies of the addon's voice takes, which a first install need not fetch
		let voice = self.app.path().resource_dir().ok().map(|dir| dir.join("resources").join(audio::VOICE));
		let scope = self.scope(&addons, packs);
		addon::stage(addons, feed, first, voice, scope, self.addon.news_tx.clone());
	}

	/// The player's Update now for `addons` stands (it is managed from its swap on).
	fn asked_update(&self, addons: &Path) -> bool {
		matches!(&self.addon.asked, Some(Asked::Update(asked)) if asked == addons)
	}

	/// An update of `addons` is still the helper's to make: a first install, the player's Update now, or an install the
	/// helper keeps.
	fn update_wanted(&self, addons: &Path, first: bool) -> bool {
		first || self.asked_update(addons) || self.addon.owners.get(addons) == Some(&Owner::Helper)
	}

	fn staged(&mut self, addons: PathBuf, result: Result<Option<PathBuf>, Failure>) {
		let Job::Staging { addons: at, feed, first, packs } = &self.addon.job else { return };
		if *at != addons {
			return;
		}
		let (feed, first, packs) = (feed.clone(), *first, *packs);
		match result {
			Ok(None) => {
				log::info!("addon: QuestPump is back in {}: nothing to add", addons.display());
				self.addon.set_job(Job::Idle);
				self.addon.asked = None;
				self.folders_back(addons);
			}
			Ok(Some(_)) if !self.update_wanted(&addons, first || packs) => {
				log::info!(
					"addon: the update staged for {} is dropped: QuestPump no longer keeps it",
					addons.display()
				);
				self.settle(addons);
			}
			Ok(Some(staging)) => {
				log::info!("addon: {} staged in {}", feed.version, staging.display());
				self.addon.set_job(Job::Staged { addons, staging, feed, first, packs });
			}
			Err(failure) => {
				log::warn!("addon: {} for {}: {}", feed.version, addons.display(), failure.error);
				self.settle(addons.clone());
				self.addon_failed(&addons, Some(feed.version), first, &failure);
			}
		}
	}

	/// The swap off the loop, the folders read again first: no link or `.git`, a managed install still the record's
	/// files, an install without a record only for a first install or the player's Update now, a first install still
	/// without QuestPump (a record left from an addon since removed is replaced); an update that is no first install
	/// waits again when the game or a pump started meanwhile (`held`).
	fn swap_in(&mut self, addons: PathBuf, staging: PathBuf, feed: Box<Feed>, first: bool, packs: bool) {
		log::info!("addon: {} going into {}", feed.version, addons.display());
		let record = self.addon.record(&addons).cloned();
		let asked = self.asked_update(&addons);
		let (scope, of) = (self.scope(&addons, packs), feed.clone());
		// the folders to put in, as they stand right before the first rename: only those that differ from the feed's,
		// and for a packs job only a pack that is missing or still the helper's own
		let check = move |tree: &Tree| -> Result<Vec<&'static str>, String> {
			match (tree, &record) {
				(Tree::Linked, _) => {
					Err("the game folder holds a link or a .git: QuestPump leaves it alone".to_owned())
				}
				(Tree::Files(files), _) if packs && !files.contains_key(rules::TOC) => {
					Err("QuestPump is no longer in the game".to_owned())
				}
				_ if packs => Ok(scope.folders(&of, tree)),
				(Tree::Files(files), _) if first && files.keys().any(|p| p.starts_with("QuestPump/")) => {
					Err("QuestPump is in the game already".to_owned())
				}
				(_, Some(record)) if !first && !asked && !record.matches(tree) => {
					Err("another updater changed it meanwhile".to_owned())
				}
				(_, None) if !first && !asked => Err("another updater keeps it now".to_owned()),
				_ => Ok(scope.folders(&of, tree)),
			}
		};
		let held = (!first).then(|| self.addon.held.clone());
		self.addon.set_job(Job::Swapping { addons: addons.clone(), staging: staging.clone(), feed, first, packs });
		addon::swap(addons, staging, held, check, self.addon.news_tx.clone());
	}

	/// The game or a pump started before the swap's first rename: nothing changed, and the update waits again.
	fn swap_held(&mut self, addons: &Path) {
		let Job::Swapping { addons: at, staging, feed, first, packs } = &self.addon.job else { return };
		if at != addons {
			return;
		}
		log::info!("addon: {} for {}: the game or a pump started, it waits", feed.version, addons.display());
		let (staging, feed) = (staging.clone(), feed.clone());
		let job = Job::Staged { addons: at.clone(), staging, feed, first: *first, packs: *packs };
		self.addon.set_job(job);
	}

	fn swapped(&mut self, addons: &Path, result: Result<Vec<&'static str>, Failure>) {
		let Job::Swapping { addons: at, feed, first, packs, .. } = &self.addon.job else { return };
		if at != addons {
			return;
		}
		let (feed, first, packs) = (feed.clone(), *first, *packs);
		// the old folders go, or any the swap could not put back go back first
		self.settle(addons.to_path_buf());
		let version = feed.version.clone();
		let folders = match result {
			Ok(folders) => folders,
			Err(failure) => {
				log::warn!("addon: {version} not in {}: {}", addons.display(), failure.error);
				self.addon_failed(addons, Some(version), first, &failure);
				return;
			}
		};
		log::info!("addon: {version} is in {} (put in: {})", addons.display(), folders.join(", "));
		let before = self.addon.record(addons).cloned();
		self.addon.records.retain(|r| r.addons != addons);
		if packs {
			// the packs the helper put in join what it kept there; who keeps QuestPump itself stays as it was
			let files = rules::merged(before.as_ref().map(|r| &r.files), &feed, &folders);
			let version = before.map_or(version, |r| r.version);
			if !files.is_empty() {
				self.addon.records.push(Record { addons: addons.to_path_buf(), version, files });
			}
			self.addon.packs_due.remove(addons);
		} else {
			self.addon.asked = None;
			self.addon.records.push(Record { addons: addons.to_path_buf(), version, files: feed.hashes() });
			self.addon.owners.insert(addons.to_path_buf(), Owner::Helper);
			self.addon.stated = None; // stated again at the next look, once the install is followed
		}
		self.save_records();
		if let Some(game) = self.desk.p.game.as_ref().filter(|g| g.addons == addons) {
			self.addon.added_while = Some(game.pid);
		}
		// QuestPump found in the folder at the next look: the mirror, the folder's line, the welcome's game
		self.looked_for_game = None;
		self.shown_folder = None;
		self.due_by(Instant::now());
	}

	fn addon_failed(&mut self, addons: &Path, version: Option<String>, first: bool, failure: &Failure) {
		self.addon.asked = None;
		self.addon.failed =
			Some(Failed { addons: Some(addons.to_path_buf()), version, first, error: failure.error.clone() });
		// Windows: a refused write shows This computer's card for the flavour's Interface folder, which the staging
		// folder and AddOns are in (`Helper::refused`): the grant for a "denied" (os error 5) alone, Controlled folder
		// access in a folder it protects by default, whose blocks show as "not found" (2, 3) as often; a game without an
		// AddOns folder yet is no refusal (`addon::tree`, `addon::swap` make do without one)
		if let (Some(code @ (2 | 3 | 5)), Some(interface)) = (failure.os, addons.parent()) {
			self.refused(interface, &std::io::Error::from_raw_os_error(code));
		}
	}

	/// The staging folder of `addons` put right and removed off the loop, as a job of its own: QuestPump's own update,
	/// Update now and the next staging wait for it.
	fn settle(&mut self, addons: PathBuf) {
		self.addon.set_job(Job::Settling { addons: addons.clone() });
		addon::settle(addons, self.addon.news_tx.clone());
	}

	/// While no job runs, a staging folder that a quit, a crash or a power cut left mid-swap or mid-cleaning (beside a
	/// recorded install, the followed one, the game on this computer without QuestPump or the folder remembered) is put
	/// right before anything else; one that did not settle this run waits for its next staging.
	fn settle_due(&mut self) {
		if self.addon.job != Job::Idle {
			return;
		}
		let known = (self.addon.records.iter().map(|r| r.addons.clone()))
			.chain(self.desk.p.followed.clone())
			.chain(self.desk.p.without_addon.clone())
			.chain(self.desk.settings.addons.clone());
		let found = known
			.filter(|a| !self.addon.unsettled.contains(a))
			.find(|a| rules::staging(a).is_some_and(|staging| std::fs::symlink_metadata(staging).is_ok()));
		if let Some(addons) = found {
			log::info!("addon: {} holds a staging folder: put right first", addons.display());
			self.settle(addons);
		}
	}

	/// The staging folder settled: the job ends, folders put back are looked at again, and a first install the player
	/// asked for meanwhile goes on (or ends, its QuestPump back).
	fn settled(&mut self, addons: PathBuf, result: Result<Vec<&'static str>, String>) {
		let Job::Settling { addons: at } = &self.addon.job else { return };
		if *at != addons {
			return;
		}
		self.addon.set_job(Job::Idle);
		let mut questpump_back = false;
		match result {
			Ok(back) => {
				self.addon.unsettled.remove(&addons);
				if !back.is_empty() {
					log::warn!(
						"addon: a swap cut short left {} aside: put back in {}",
						back.join(", "),
						addons.display()
					);
					questpump_back = back.contains(&rules::FOLDERS[0]);
					self.folders_back(addons);
				}
			}
			Err(e) => {
				log::warn!("addon: {e}");
				self.addon.unsettled.insert(addons);
			}
		}
		if self.addon.asked == Some(Asked::Add) {
			if questpump_back {
				self.addon.asked = None;
			} else {
				self.add_first();
			}
		}
	}

	/// Folders a swap cut short had left aside are back in `addons`: QuestPump found there at the next look (the
	/// mirror, the folder's line, the welcome's game), and its owner read again.
	fn folders_back(&mut self, addons: PathBuf) {
		self.looked_for_game = None;
		self.shown_folder = None;
		self.due_by(Instant::now());
		self.look_at(addons);
	}

	/// The grant on the game folder was made (`runtime/folder.rs`): a failed step goes again.
	pub(super) fn addon_retry(&mut self) {
		if let Some(failed) = self.addon.failed.take() {
			if failed.first {
				self.addon.asked = Some(Asked::Add);
				self.add_first();
			} else if let Some(addons) = failed.addons {
				self.addon.asked = Some(Asked::Update(addons));
			}
		}
	}

	/// The window's Add QuestPump to the game or Update now.
	pub(super) fn addon_install(&mut self) {
		if self.addon.job != Job::Idle {
			return;
		}
		match self.found_game() {
			FoundGame::Installed => {
				self.addon.asked = Some(Asked::Add);
				self.add_first();
			}
			FoundGame::Found | FoundGame::Running => {
				let Some(followed) = self.desk.p.followed.clone() else { return };
				if self.addon.owners.get(&followed) == Some(&Owner::Linked) {
					return;
				}
				log::info!("addon: the player asked to update {}", followed.display());
				self.addon.failed = None;
				self.addon.asked = Some(Asked::Update(followed));
				self.stage_due();
			}
			FoundGame::Unfound => {}
		}
		self.addon_show();
	}

	/// A first install the player asked for: with the feed, into the install its client's version names, else the
	/// folder the player picks; without the feed yet, once it comes.
	fn add_first(&mut self) {
		let Some(feed) = self.addon.feed.clone() else {
			self.check_feed();
			return;
		};
		if self.addon.job != Job::Idle {
			return;
		}
		if let Some(addons) = self.first_target(&feed.interfaces) {
			self.stage(addons, feed, true, false);
		} else {
			log::info!("addon: no game folder runs one of {:?}: the player picks it", feed.interfaces);
			crate::desk::commands::pick_folder(self.app.clone());
		}
	}

	/// Where a first install goes: a folder the player picked (`without_addon_picked`) whose client runs a number the
	/// feed lists; else, of the game folders known without QuestPump (the one shown, the running client's, the Agent's
	/// and the default folders' installs), those whose client runs the feed's first interface, else those that run
	/// another number the feed lists; of them the running client's, else the preferred one (`install::preferred`:
	/// Forever's live install, never its beta beside it, which runs the same interface), else the first known.
	fn first_target(&self, interfaces: &[u32]) -> Option<PathBuf> {
		let picked = self.desk.p.without_addon.as_ref().filter(|a| {
			self.without_addon_picked
				&& install::is_game(a)
				&& !install::holds_questpump(a)
				&& install::client_interface(a).is_some_and(|n| interfaces.contains(&n))
		});
		if let Some(picked) = picked {
			return Some(picked.clone());
		}
		let running = self.desk.p.game.as_ref().map(|g| &g.addons);
		let mut known: Vec<PathBuf> = Vec::new();
		for addons in self.desk.p.without_addon.iter().chain(running).cloned().chain(install::games()) {
			if install::is_game(&addons) && !install::holds_questpump(&addons) && !known.contains(&addons) {
				known.push(addons);
			}
		}
		let runs_one_of = |numbers: &[u32]| {
			let runs: Vec<&PathBuf> =
				known.iter().filter(|a| install::client_interface(a).is_some_and(|n| numbers.contains(&n))).collect();
			runs.iter().copied().find(|a| Some(*a) == running).or_else(|| install::preferred(runs))
		};
		runs_one_of(interfaces.get(..1).unwrap_or_default()).or_else(|| runs_one_of(interfaces)).cloned()
	}

	/// The player picked a folder without QuestPump while Add QuestPump to the game waited for one: it goes there.
	pub(super) fn addon_picked(&mut self, addons: &Path) {
		if self.addon.asked == Some(Asked::Add)
			&& self.addon.job == Job::Idle
			&& let Some(feed) = self.addon.feed.clone()
		{
			self.stage(addons.to_path_buf(), feed, true, false);
		}
	}

	/// The game on this computer without QuestPump was found: the feed is checked at once, so Add QuestPump to the game
	/// has it.
	pub(super) fn addon_game_found(&mut self) {
		if self.addon.feed.is_none() && !std::mem::replace(&mut self.addon.checked_early, true) {
			self.check_feed();
		}
	}

	/// The addon's offer and line, for This computer, the welcome and Pump home's card.
	pub(crate) fn addon_view(&self) -> AddonView {
		let busy = self.addon.job != Job::Idle;
		let failed = self.addon.failed.as_ref();
		match self.found_game() {
			FoundGame::Unfound => AddonView::default(),
			FoundGame::Installed => {
				let line = if busy && self.addon.job_first() {
					AddonLine::Adding
				} else if let Some(f) = failed.filter(|f| f.first) {
					AddonLine::AddFailed { error: f.error.clone() }
				} else {
					AddonLine::Missing
				};
				AddonView { offer: (!busy).then_some(AddonOffer::Add), line }
			}
			FoundGame::Found | FoundGame::Running => {
				let Some(followed) = &self.desk.p.followed else { return AddonView::default() };
				self.installed_view(followed)
			}
		}
	}

	fn installed_view(&self, followed: &Path) -> AddonView {
		let version = install::addons_state(followed).questpump.unwrap_or_default();
		let feed = self.addon.feed.as_ref().filter(|f| rules::newer(&f.version, &version));
		let owner = self.addon.owners.get(followed).copied();
		let line = if self.addon.added_while.is_some() {
			AddonLine::Added
		} else if self.addon.job_addons() == Some(followed) {
			match (&self.addon.job, self.addon.job_feed()) {
				(Job::Staged { .. }, Some(feed)) if self.addon_held() => {
					AddonLine::Ready { version, new: feed.version.clone() }
				}
				_ => AddonLine::Updating,
			}
		} else if let Some(f) = self.addon.failed.as_ref().filter(|f| f.addons.as_deref() == Some(followed)) {
			AddonLine::Failed { error: f.error.clone() }
		} else if let Some(feed) = feed.filter(|_| owner == Some(Owner::Other)) {
			AddonLine::Newer { version, new: feed.version.clone() }
		} else if self.addon.feed.is_some() && feed.is_none() && rules::version(&version).is_some() {
			AddonLine::Current { version }
		} else {
			AddonLine::Version { version }
		};
		// Update now for an addon another updater keeps, and beside a failed update (it tries again at once)
		let retry = matches!(line, AddonLine::Failed { .. }) && feed.is_some() && owner != Some(Owner::Linked);
		let idle = self.addon.job == Job::Idle;
		let offer = (idle && (matches!(line, AddonLine::Newer { .. }) || retry)).then_some(AddonOffer::Update);
		AddonView { offer, line }
	}

	/// The view's addon part now (the publisher sends it only when it changed).
	pub(super) fn addon_show(&self) {
		let view = self.addon_view();
		ui::update(&self.app, |v| v.computer.addon = view);
	}
}
