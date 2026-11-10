//! The desktop app as the desk's platform (docs/plan.md slice 63, "One desk core"): [`Shell`] implements
//! `questpump_desk::Platform` over what the desk reads of the computer (the game QuestPump follows and the screen's
//! reading of it, the games on the computer and the match offer that stands, the Pro grants, the sitting clock, the
//! audio thread) and what it asks of it (`settings.json` and the data folder's files through the writer, This
//! computer's and the frame's view, the window, the tray, the day reel, the sync's calls and live socket through
//! `runtime::sync::SyncLink`). What only the whole loop can do (the socket's lines and the devices' commands, the
//! mirror, Pro, the card, the reminders, the counts) the shell puts in its outbox ([`Out`]), which the loop runs once
//! the desk's call returned (`Helper::drain`), in the order it was asked.

use std::path::PathBuf;
use std::sync::atomic::Ordering;

use questpump_core::offers::{Offer, Pause, Standing, Tunables};
use questpump_core::protocol::Source;
use questpump_core::session::{Effects, Zone};
use questpump_core::sitting::Sitting;
use questpump_core::sync::DESKTOP;
use questpump_core::work::Hold;
use questpump_desk::Platform;
use questpump_desk::audio::Channel;
use questpump_desk::home::Board;
use questpump_desk::platform::{
	EngineState, Fix, Fold, FoundGame, Game, GameTile, GuideTake, GuideTakes, HomeShell, Step,
};
use questpump_desk::saga::ReelTune;
use questpump_desk::settings::{GamesSettings, SoundSettings};
use questpump_desk::sync::{Call, Then};
use questpump_desk::work::WorkView;
use serde_json::Value;
use tauri::{AppHandle, Manager as _};

use super::Desk;
use super::audio::{self, Audio};
use super::tray::State;
use super::windows::{self, Landing, MAIN};
use super::{awake, figures, offers, reel, tray};
use crate::games::procs::Front;
use crate::games::{Games, Wow};
use crate::lan;
use crate::runtime::sync::SyncLink;
use crate::runtime::{self, Job, Sighting, local_offset_at};
use crate::ui::{self, Dot, FirewallCard, Folder, Mirror, Screen, View};

/// What the desk asked that only the whole loop does, run once the desk's call returned (`Helper::drain`).
pub(crate) enum Out {
	/// The effects of rows the session took, beyond the desk's own: the socket's lines, the devices' commands, the
	/// logs, the game's feedback, a new load's saves, `/qp ping`, the mirror's due time, the counts.
	Effects(Box<Effects>),
	/// The body changed: the mirror carries it soon.
	MirrorSoon,
	/// A pump came on screen or left it: Backup's word on it.
	Backup,
	/// The offers' pause changed: the tray's items and the Games tab drawn again.
	OffersPaused,
	/// A pump's start from this match offer went through: its card goes.
	OfferStarted(String),
	/// The pump from this match offer started: the offer is taken.
	OfferTaken(String),
	/// The streak's board Pump home drew: the reminders' tray line.
	Board { board: Board, game: bool, resting: bool },
	/// A Result was put away: "Remind me once a day?" may be due.
	ReminderOfferDue,
	/// A step's card opened, with its Start's refused reason: the counts.
	TreeCard(Option<String>),
	/// A Start from a step's card or a move's window composed its pump: the counts.
	TreeStart(String),
	/// An engine answer's part that is the loop's (the counts' weekly numbers).
	Then(Box<dyn FnOnce(&mut runtime::Helper) + Send>),
}

/// The desktop app's side of the desk: what the desk reads of the computer and the handles it asks through.
pub(crate) struct Shell {
	pub(crate) app: AppHandle,
	/// The data folder.
	pub(crate) dir: PathBuf,
	/// The writer's inbox: `settings.json` and the data folder's files.
	pub(crate) writes: std::sync::mpsc::Sender<Job>,
	/// The running client, when there is one.
	pub(crate) game: Option<crate::game::Game>,
	/// The running game's folder when it holds QuestPump: the mirror and the sign of life go there, and a game without
	/// it (another flavour beside the Forever install) gets neither.
	pub(crate) game_addons: Option<PathBuf>,
	/// The install the session follows (`addons`): a game folder with QuestPump, the running client's or the one found
	/// or picked; none while none with QuestPump is known. The welcome's game, the grant and the pairing's next step read
	/// it.
	pub(crate) followed: Option<PathBuf>,
	/// The game on this computer without QuestPump yet, while no install with it is known (`show_without_addon`): the
	/// running client's, a folder the player picked (`without_addon_picked`), else the preferred of the one remembered
	/// and the installs the Agent or the default folders know (`install::preferred`, `install::discover_game`).
	pub(crate) without_addon: Option<PathBuf>,
	/// What the window's screen line knows beside the dot.
	pub(crate) sighting: Sighting,
	pub(crate) screen: Dot,
	/// The stores hold what this computer has: the first mirror job took in the install the session follows, or found
	/// none known (`stores_are_read`). The first five minutes decide nothing before (`Found::ready`).
	pub(crate) stores_read: bool,
	/// The games on the computer; none when the offers' numbers did not build in (logged at the start).
	pub(crate) games: Option<Games>,
	/// The match offer that stands on the computer.
	pub(crate) standing: Standing,
	/// The input's part of the sitting clock, which the sitting cue and the match offers share (slice 48).
	pub(crate) sitting: Sitting,
	/// What holds work mode's clock on this computer at the last look while it is on (`runtime/work.rs`: a call, a game
	/// in front or in a match, presenting or mirroring, full screen), beside the desk's own holds (slice 67).
	pub(crate) work_hold: Option<Hold>,
	/// The app last in front other than QuestPump while work mode is on (`presence::look`).
	pub(crate) front: Option<Front>,
	/// The app a work pump's Back to work brings to the front, noted as the pump came (`Platform::back_note`).
	pub(crate) back_to: Option<Front>,
	/// The paired phones' Pro grants (slice 35).
	pub(crate) pro: runtime::Pro,
	/// The desktop app's sound and voice on the picked output (slice 35).
	pub(crate) audio: Audio,
	/// The sync's calls and live socket (slice 60).
	pub(crate) sync: SyncLink,
	/// What the loop runs once the desk's call returned.
	pub(crate) out: Vec<Out>,
}

impl Shell {
	/// The game QuestPump knows on this computer now: a client whose AddOns hold QuestPump runs, else the install the
	/// session follows, else the game without QuestPump or a client without it, else none.
	pub(crate) fn found_game(&self) -> FoundGame {
		if self.game.is_some() && self.game_addons.is_some() {
			FoundGame::Running
		} else if self.followed.is_some() {
			FoundGame::Found
		} else if self.without_addon.is_some() || self.game.is_some() {
			FoundGame::Installed
		} else {
			FoundGame::Unfound
		}
	}
}

/// What needs fixing in the view, if anything: a step left on This computer (the grant, the folder's rights, the move
/// to Applications, the screen reading stopped), a client without QuestPump running, the squares out of sight for a
/// minute or not read (a strip of another version, colours that do not read) while they are not being read; then a
/// save that failed; then the firewall between this PC and a paired phone. The game found, read, waited for at its
/// start or out of view for a moment, the games watched, and no game at all are no fault.
fn fix_of(view: &View, paired: bool) -> Option<Fix> {
	let c = &view.computer;
	let unread = matches!(c.screen, Screen::Unseen | Screen::Newer | Screen::Older | Screen::Colours { .. });
	if c.state == State::Step || (c.state == State::Lacks && c.game.is_some()) || (unread && c.state != State::Reading)
	{
		Some(Fix::Computer)
	} else if matches!(c.mirror, Mirror::Failed { .. } | Mirror::Unfound) {
		Some(Fix::Save)
	} else if paired && view.phones.firewall != FirewallCard::Open {
		Some(Fix::Firewall)
	} else {
		None
	}
}

impl Platform for Shell {
	const DEVICE: Source = Source::Computer;
	const PUMP_IDS: &'static str = "d-";

	fn now_ms(&self) -> i64 {
		lan::now_ms()
	}

	fn zone(&self) -> fn(i64) -> i64 {
		local_offset_at
	}

	fn save_settings(&self, file: Value) {
		let _ = self.writes.send(Job::Settings { dir: self.dir.clone(), file });
	}

	fn save_file(&self, name: &'static str, bytes: Vec<u8>) {
		let _ = self.writes.send(Job::File { dir: self.dir.clone(), name, bytes });
	}

	fn body_changed(&mut self) {
		self.out.push(Out::MirrorSoon);
	}

	fn effects(&mut self, fx: Effects) {
		self.out.push(Out::Effects(Box::new(fx)));
	}

	const SYNC_KIND: &'static str = DESKTOP;

	fn device_name(&self) -> String {
		self.sync.name.clone()
	}

	fn sync_call(&mut self, call: Call, then: Then) {
		self.sync.call(call, then);
	}

	fn live_open(&mut self, token: String, generation: u64) {
		self.sync.live_open(token, generation);
	}

	fn live_close(&mut self) {
		self.sync.live_close();
	}

	fn sync_reachable(&self) -> bool {
		self.sync.reachable()
	}

	fn secrets_unread(&self) -> bool {
		self.sync.secrets_unread
	}

	fn account_pro(&mut self, token: Option<String>, status: Option<String>) {
		self.out.push(Out::Then(Box::new(move |h| h.take_account_pro(token.as_deref(), status.as_deref()))));
	}

	fn portal_answered(&mut self, url: Option<String>) {
		self.out.push(Out::Then(Box::new(move |h| h.portal_done(url.as_deref()))));
	}

	fn sync_switched(&mut self) {
		self.out.push(Out::Then(Box::new(|h| h.publish_pro())));
	}

	fn play_sound(&self, id: &str) {
		self.audio.sound(id);
	}

	fn play_take(&self, id: &str) {
		self.audio.take(id);
	}

	fn hush(&self) {
		self.audio.hush();
	}

	fn hold_output(&self, on: bool) {
		self.audio.hold(on);
	}

	fn sound_changed(&mut self, sound: &SoundSettings) {
		self.audio.settings(sound.clone());
		audio::show_picked(&self.app, sound.output_picked);
	}

	fn sound_test(&self, channel: Channel) {
		self.audio.test(channel);
	}

	fn reader_hears(&self) -> bool {
		audio::reader_hears(&self.app)
	}

	fn announce(&mut self, text: String) {
		self.audio.announce(&self.app, text);
	}

	fn guide_takes(&mut self, key: &str, gear: Option<&str>) -> GuideTakes {
		self.audio.guide_read(self.followed.as_deref(), key, gear)
	}

	fn play_guide(&self, take: &GuideTake) {
		self.audio.guide(take.file.clone());
	}

	fn stop_guide(&self) {
		self.audio.guide_stop();
	}

	fn films(&self) -> bool {
		figures::films_here()
	}

	fn reel_tune(&self) -> Option<ReelTune> {
		reel::tune()
	}

	fn game(&self) -> Game {
		let with_addon = self.game.is_some() && self.game_addons.is_some();
		Game {
			client: self.game.is_some(),
			with_addon,
			unseen: with_addon && !self.sighting.read && self.sighting.line == Screen::Unseen,
			followed: self.followed.is_some(),
			live: self.screen == Dot::Live,
			found: self.found_game(),
		}
	}

	fn stores_read(&self) -> bool {
		self.stores_read
	}

	fn games_open(&self) -> Vec<String> {
		self.games.as_ref().map(|g| g.open().to_vec()).unwrap_or_default()
	}

	fn game_tiles(&self, games: &GamesSettings) -> Vec<GameTile> {
		let tiles = self.games.as_ref().map(|g| g.lists(games).tiles).unwrap_or_default();
		tiles.into_iter().map(|t| GameTile { key: t.key, name: t.name, icon: t.icon }).collect()
	}

	fn wow_icon(&self) -> Option<String> {
		self.games.as_ref().and_then(Games::wow_icon)
	}

	fn recent_wow(&mut self, known: bool) {
		let addons = self.game.as_ref().map(|g| &g.addons).or(self.followed.as_ref());
		let wow = known.then(|| Wow {
			// the install's flavour folder, above its Interface/AddOns
			root: addons.and_then(|a| a.parent()?.parent()).map(std::path::Path::to_path_buf),
			pid: self.game.as_ref().map(|g| g.pid),
		});
		if let Some(games) = self.games.as_mut() {
			games.set_wow(wow);
		}
	}

	fn offer(&self) -> Option<&Offer> {
		self.standing.current()
	}

	fn tunables(&self) -> Option<&Tunables> {
		self.games.as_ref().map(Games::tunables)
	}

	fn arms_game(&self, games: &GamesSettings) -> Option<String> {
		runtime::arms_game(self.games.as_ref(), games)
	}

	fn seated_s(&self, at: i64, last_move: Option<i64>) -> i64 {
		self.sitting.sat_s(at, last_move, None, Zone::new(local_offset_at).game_day_start(at))
	}

	fn offers_pause(&self, kind: &str, games: &GamesSettings) -> Result<Pause, String> {
		offers::pause_of(kind, self.games.as_ref(), games, lan::now_ms().div_euclid(1_000))
	}

	fn offers_paused(&mut self) {
		self.out.push(Out::OffersPaused);
	}

	fn offer_started(&mut self, offer: &str) {
		self.out.push(Out::OfferStarted(offer.to_owned()));
	}

	fn offer_taken(&mut self, offer: &str) {
		self.out.push(Out::OfferTaken(offer.to_owned()));
	}

	fn engine_state(&mut self, state: EngineState, fold: Option<Fold>) {
		ui::update(&self.app, |v| {
			v.shell.engine = state;
			if fold.is_some() {
				v.shell.fold = fold;
			}
		});
	}

	fn home_shell(&self, paired: bool) -> HomeShell {
		let view = ui::view(&self.app);
		let step = if view.computer.permission && matches!(view.computer.folder, Folder::Found { .. }) && paired {
			Some(Step::Grant { asked: view.computer.asked })
		} else if view.phones.firewall == FirewallCard::Rule && view.phones.network_note.is_some() {
			Some(Step::Firewall)
		} else {
			None
		};
		HomeShell { fix: fix_of(&view, paired), step, found: view.computer.found }
	}

	fn show_found(&mut self) {
		let found = self.found_game();
		ui::update(&self.app, |v| v.computer.found = found);
	}

	fn reel_locked(&mut self, on: bool) {
		reel::locked(&self.app, on);
	}

	fn guard(&mut self, running: bool) {
		self.app.state::<Desk>().guard.store(running, Ordering::Relaxed);
	}

	fn pump_running(&mut self, running: bool) {
		awake::hold(&self.app, running);
		// a pump starting cancels a day reel being made (slice 45)
		if running {
			reel::cancel(&self.app);
		}
		// Backup says whether a pump is on screen
		self.out.push(Out::Backup);
	}

	fn tray_pumps(&mut self, items: (bool, bool, bool)) {
		let handle = self.app.clone();
		let _ = self.app.run_on_main_thread(move || tray::draw_pumps(&handle, items));
	}

	fn open_window(&mut self) -> Result<(), String> {
		let handle = self.app.clone();
		self.app.run_on_main_thread(move || windows::open_main(&handle, Landing::Here)).map_err(|e| e.to_string())
	}

	fn window_open(&self) -> bool {
		self.app.get_webview_window(MAIN).is_some()
	}

	fn full_screen(&mut self, on: bool) {
		if let Some(window) = self.app.get_webview_window(MAIN)
			&& window.is_fullscreen().is_ok_and(|now| now != on)
			&& let Err(e) = window.set_fullscreen(on)
		{
			log::warn!("pump: full screen: {e}");
		}
	}

	fn view_full(&mut self, on: bool) -> Result<(), String> {
		windows::full_view(&self.app, on)
	}

	fn leave(&mut self, quit: bool) -> Result<(), String> {
		let mut start = None;
		ui::update(&self.app, |v| start = v.shell.leave.take().and_then(|l| l.start));
		let handle = self.app.clone();
		self.app
			.run_on_main_thread(move || if quit { ui::quit_for(&handle, start) } else { windows::close_main(&handle) })
			.map_err(|e| e.to_string())
	}

	fn stay(&mut self) {
		ui::update(&self.app, |v| v.shell.leave = None);
	}

	fn reminders_board(&mut self, board: Board, game: bool, resting: bool) {
		self.out.push(Out::Board { board, game, resting });
	}

	fn reminder_offer_due(&mut self) {
		self.out.push(Out::ReminderOfferDue);
	}

	fn tree_card(&mut self, refused: Option<&str>) {
		self.out.push(Out::TreeCard(refused.map(str::to_owned)));
	}

	fn tree_start(&mut self, kind: &str) {
		self.out.push(Out::TreeStart(kind.to_owned()));
	}

	fn pro_grants(&self, account: Option<&str>) -> bool {
		self.pro.grants_on(lan::now_ms(), account)
	}

	fn idle_s(&self) -> Option<u32> {
		self.sitting.idle_s()
	}

	fn work_hold(&self) -> Option<Hold> {
		self.work_hold
	}

	fn work_changed(&mut self, view: &WorkView) {
		let (handle, view) = (self.app.clone(), view.clone());
		let _ = self.app.run_on_main_thread(move || tray::draw_work(&handle, &view));
	}

	fn back_note(&mut self) -> bool {
		self.back_to = self.front;
		self.back_to.is_some()
	}

	fn back_to_work(&mut self) {
		let Some(front) = self.back_to.take() else { return };
		let _ = self.app.run_on_main_thread(move || windows::back_to(front));
	}
}
