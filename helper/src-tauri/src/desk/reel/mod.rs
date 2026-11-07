//! The day reel on the computer (docs/plan.md slice 45; docs/ui.md, "The share card": the Reel): the day's Saga card in
//! motion, made in Rust by the system's encoder as soon as the Share window's Reel is chosen. The page sends what it
//! drew (`lib/reel.js`): the plan it read from the `log` topic's day (`desk::saga::Saga::reel`, rule 18), whether the
//! reel has sound, its file's name, the knight's square and look, and its layers, each a PNG at its place for its frames
//! (the ground under the knight, the words, the number and the strip at the Saga's reveal, each shot's slot over it).
//! This module checks the plan against the tokens and the films this computer has (`figures::film`), and a job on a
//! blocking thread draws each frame (`compose`: the layers and the knight, the shot's film from its key pose at its real
//! tempo), mixes the sounds (`sound`), encodes the reel (`mac`: `AVAssetReader` and `AVAssetWriter`; `win`: Media
//! Foundation's source reader and sink writer), and writes the file in its one shape (`mp4::faststart`: moov first, no
//! metadata) under its ASCII name in its own folder of the reels' cache folder, which a sweep empties of reels older
//! than a day (`KEEP`) at the start and at each reel.
//!
//! One reel at a time: a new one cancels the one being made; closing the Share window (`reel_cancel`), the main window,
//! a pump starting and the felt-unwell lock as it starts cancel it too (the window stays, and its Save video or Copy
//! video makes the reel again), and a cancelled reel leaves no file; a made one stays until the sweep, for Save video
//! and for Copy video's file on the clipboard. The `reel` topic: `{"id", "state"
//! (idle | making | ready | failed | cancelled | unsupported), "src" (the made file's asset URL, ready only), "timing"
//! (the reel's times from contract/design/tokens.json: fps, a move's shot, the end, the reveal's start and length, the
//! bars' stagger, the digit pop's length and scale, both curves, the most moves, the frame's size in pixels)}`; the page
//! draws its layers from `timing`, never from its CSS, whose reduced-motion block zeroes some of them while the file's
//! times stand.

mod compose;
#[cfg(target_os = "macos")]
mod mac;
mod mp4;
mod sound;
#[cfg(windows)]
mod win;

#[cfg(target_os = "macos")]
use mac as system;
#[cfg(windows)]
use win as system;

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock, Mutex, MutexGuard};
use std::time::{Duration, SystemTime};

use anyhow::{Context as _, bail};
use questpump_desk::facts::facts;
use questpump_desk::saga::{Reel, ReelTune};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Manager as _};

use super::Desk;
use super::figures::{self, MoveFilm};
use crate::lan;
use crate::runtime::Helper;
use compose::{Canvas, FilmCut, Knight, Layer, Look, Square};

/// The page's topic.
pub(crate) const TOPIC: &str = "reel";
/// How long a made reel stays in the cache folder.
const KEEP: Duration = Duration::from_hours(24);
/// The films' frame rate (contract/figures/figures.json: an owner's `loop` is its frames / 30).
const FILM_FPS: f64 = 30.0;
/// The most layers and the largest body a reel's order may bring.
const LAYERS_MAX: usize = 2_048;
const BODY_MAX: usize = 256 << 20;
/// The longest file name a reel takes, before `.mp4`.
const NAME_MAX: usize = 120;
/// A reel's file name when the page names none (`share.js` `fileName`: a name of no Latin letter is left empty).
const FILE: &str = "QuestPump.mp4";

/// The reel's times and its encoding (contract/design/tokens.json: `motion.sagaReel`, `motion.sagaReveal`,
/// `motion.digitPop`, `curve`, `surface.SagaReel`, `surface.SagaCard`'s width and story height at its scale); the page
/// reads the times, the encoders the rest.
#[derive(Serialize, Debug, Clone, Copy, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Timing {
	fps: u32,
	move_ms: i64,
	end_ms: i64,
	reveal_from_ms: i64,
	reveal_ms: i64,
	stagger_ms: i64,
	pop_ms: i64,
	pop_from: f64,
	reveal_curve: [f64; 4],
	pop_curve: [f64; 4],
	move_count: usize,
	width: u32,
	height: u32,
	#[serde(skip)]
	key_every: u32,
	#[serde(skip)]
	bitrate_kbps: u32,
	#[serde(skip)]
	audio_kbps: u32,
	#[serde(skip)]
	audio_hz: u32,
	#[serde(skip)]
	gain_db: f64,
}

impl Timing {
	/// Rule 18's times.
	fn tune(&self) -> ReelTune {
		ReelTune {
			move_ms: self.move_ms,
			end_ms: self.end_ms,
			reveal_from_ms: self.reveal_from_ms,
			reveal_ms: self.reveal_ms,
			moves: self.move_count,
		}
	}

	/// A time in the reel (ms) as its frame.
	fn frame(&self, ms: i64) -> usize {
		usize::try_from(ms.max(0) * i64::from(self.fps) / 1000).unwrap_or(0)
	}
}

/// The tokens, built in; none where they do not read (no reel is planned or made then).
static TIMING: LazyLock<Option<Timing>> = LazyLock::new(|| {
	let t: Value = serde_json::from_str(include_str!("../../../../../contract/design/tokens.json")).ok()?;
	let int = |v: &Value| v.as_i64();
	let word = |v: &Value| v.as_u64().and_then(|n| u32::try_from(n).ok());
	let curve = |name: &Value| -> Option<[f64; 4]> {
		let points = t["curve"][name.as_str()?].as_array()?;
		let p = |i: usize| points.get(i)?.as_f64();
		Some([p(0)?, p(1)?, p(2)?, p(3)?])
	};
	let (reel, reveal, pop) = (&t["motion"]["sagaReel"], &t["motion"]["sagaReveal"], &t["motion"]["digitPop"]);
	let (surface, card) = (&t["surface"]["SagaReel"], &t["surface"]["SagaCard"]);
	let scale = word(&card["scaleCount"])?;
	Some(Timing {
		fps: word(&surface["fps"])?,
		move_ms: int(&reel["ms"])?,
		end_ms: int(&reel["endMs"])?,
		reveal_from_ms: int(&reel["revealFromMs"])?,
		reveal_ms: int(&reveal["ms"])?,
		stagger_ms: int(&reveal["staggerMs"])?,
		pop_ms: int(&pop["ms"])?,
		pop_from: pop["scaleFrom"].as_f64()?,
		reveal_curve: curve(&reveal["curve"])?,
		pop_curve: curve(&pop["curve"])?,
		move_count: usize::try_from(surface["moveCount"].as_u64()?).ok()?,
		width: word(&card["width"])? * scale,
		height: word(&card["storyHeight"])? * scale,
		key_every: word(&surface["keyEveryFrames"])?,
		bitrate_kbps: word(&surface["bitrateKbps"])?,
		audio_kbps: word(&surface["audioKbps"])?,
		audio_hz: word(&surface["audioHz"])?,
		gain_db: surface["gainDb"].as_f64()?,
	})
});

/// Rule 18's times for the Saga's reel (`Saga::reel`); none where the tokens do not read.
pub(crate) fn tune() -> Option<ReelTune> {
	TIMING.as_ref().map(Timing::tune)
}

/// The encoding both systems' writers take: the frame's size, its rate, a key frame every `key_every` frames, the
/// average bit rate (bit/s) and, with sound, the AAC's rate (Hz) and bit rate (kbit/s).
pub(super) struct Encoding {
	width: u32,
	height: u32,
	fps: u32,
	key_every: u32,
	bitrate: u32,
	audio: Option<(u32, u32)>,
}

/// A decoded film frame: its time in seconds, its BGRA rows (top-down) `stride` bytes apart, its width and height.
pub(super) struct Frame<'a> {
	pub(super) at: f64,
	pub(super) px: &'a [u8],
	pub(super) width: usize,
	pub(super) height: usize,
	pub(super) stride: usize,
}

/// The system has no H.264 encoder (a Windows N edition without its Media Feature Pack): `share.reel.unsupported`.
#[derive(Debug)]
pub(super) struct Unsupported;

impl fmt::Display for Unsupported {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str("this computer has no H.264 encoder") // copy-lint: not shown (an error's words for the log)
	}
}

impl std::error::Error for Unsupported {}

/// The reel was cancelled while it was made.
#[derive(Debug)]
struct Cancelled;

impl fmt::Display for Cancelled {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str("the reel was cancelled") // copy-lint: not shown (an error's words for the log)
	}
}

impl std::error::Error for Cancelled {}

/// A reel's state on the topic.
#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
enum State {
	#[default]
	Idle,
	Making,
	Ready,
	Failed,
	Cancelled,
	Unsupported,
}

#[derive(Serialize)]
struct Topic<'a> {
	id: u64,
	state: State,
	src: Option<&'a str>,
	timing: Option<&'a Timing>,
}

/// The reels' state the loop, the commands and the jobs share (in `Desk`): the reel shown (its id and state), the job
/// making it (its cancel flag) and the made file, and whether the felt-unwell lock held at the loop's last look.
#[derive(Default)]
pub(crate) struct Reels(Mutex<Shown>);

#[derive(Default)]
struct Shown {
	id: u64,
	state: State,
	job: Option<Arc<AtomicBool>>,
	made: Option<PathBuf>,
	src: Option<String>,
	locked: bool,
}

impl Reels {
	fn lock(&self) -> MutexGuard<'_, Shown> {
		self.0.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
	}
}

/// Publishes the reel shown.
fn publish(app: &AppHandle, shown: &Shown) {
	let topic = Topic { id: shown.id, state: shown.state, src: shown.src.as_deref(), timing: TIMING.as_ref() };
	app.state::<Desk>().publisher.publish(TOPIC, &topic);
}

/// The reel `id` reached `state` (its file with it when made): false where it was cancelled or a later reel took its
/// place meanwhile, which leaves the topic as it is.
fn settle(app: &AppHandle, id: u64, state: State, made: Option<PathBuf>) -> bool {
	let reels = &app.state::<Desk>().reels;
	let mut shown = reels.lock();
	if shown.id != id || shown.state != State::Making {
		return false;
	}
	shown.state = state;
	shown.job = None;
	shown.src = made.as_deref().and_then(asset_url);
	shown.made = made;
	publish(app, &shown);
	true
}

/// Cancels the reel being made, if one is: its job stops at its next frame and deletes what it began (the main
/// window's close, a pump starting, the felt-unwell lock as it starts, a new reel).
pub(crate) fn cancel(app: &AppHandle) {
	cancel_reel(app, None);
}

/// Cancels the reel being made where it is `id` (none: whichever it is): the Share window's close, or a reel it no
/// longer waits for.
pub(crate) fn cancel_reel(app: &AppHandle, id: Option<u64>) {
	let reels = &app.state::<Desk>().reels;
	let mut shown = reels.lock();
	if id.is_some_and(|id| id != shown.id) {
		return;
	}
	let Some(job) = shown.job.take() else { return };
	job.store(true, Ordering::Relaxed);
	shown.state = State::Cancelled;
	publish(app, &shown);
}

/// The felt-unwell lock as the loop's look sees it (`locked`): its start cancels the reel being made, as a pump's
/// start does; a reel the player makes while it holds is made.
pub(crate) fn locked(app: &AppHandle, locked: bool) {
	let started = {
		let reels = &app.state::<Desk>().reels;
		let mut shown = reels.lock();
		let started = locked && !shown.locked;
		shown.locked = locked;
		started
	};
	if started {
		cancel(app);
	}
}

/// The made reel `id`'s file, while it is the one shown.
pub(crate) fn made(app: &AppHandle, id: u64) -> Option<PathBuf> {
	let reels = &app.state::<Desk>().reels;
	let shown = reels.lock();
	shown.made.clone().filter(|_| shown.id == id && shown.state == State::Ready)
}

/// Puts the made reel `id` on the clipboard as a file.
///
/// # Errors
/// No such reel, or the system refused it.
pub(crate) fn copy(app: &AppHandle, id: u64) -> anyhow::Result<()> {
	let file = made(app, id).context("that reel is not made")?;
	#[cfg(target_os = "macos")]
	{
		mac::copy_file(&file)
	}
	#[cfg(windows)]
	{
		let window = app.get_webview_window(super::windows::MAIN).context("the window is closed")?;
		win::copy_file(&window, &file)
	}
}

/// The asset protocol's URL of a made file (`figures::asset_prefix`'s spelling).
fn asset_url(file: &Path) -> Option<String> {
	let prefix = figures::asset_prefix(file.parent()?)?;
	let name = file.file_name()?.to_str()?;
	Some(format!("{prefix}{}", percent_encoding::utf8_percent_encode(name, figures::URI_COMPONENT)))
}

/// The reels' cache folder (`tauri.conf.json`'s asset scope names it: `$APPCACHE/reels`).
fn folder(app: &AppHandle) -> Option<PathBuf> {
	app.path().app_cache_dir().ok().map(|dir| dir.join("reels"))
}

/// Deletes what the cache folder holds from before `KEEP` ago, but the folder `keep`.
fn sweep(folder: &Path, keep: Option<&Path>) {
	let Ok(entries) = std::fs::read_dir(folder) else { return };
	let now = SystemTime::now();
	for entry in entries.flatten() {
		let path = entry.path();
		if keep == Some(path.as_path()) {
			continue;
		}
		let old =
			entry.metadata().and_then(|m| m.modified()).is_ok_and(|t| now.duration_since(t).unwrap_or_default() > KEEP);
		if !old {
			continue;
		}
		let removed = if path.is_dir() { std::fs::remove_dir_all(&path) } else { std::fs::remove_file(&path) };
		if let Err(e) = removed {
			log::warn!("reel: {}: {e}", path.display());
		}
	}
}

/// A reel's file name from the name the page gave: printable ASCII but the characters a path or a system refuses, at
/// most `NAME_MAX` long, then `.mp4`; `FILE` where nothing is left.
fn file_name(name: &str) -> String {
	let kept: String = name
		.chars()
		.filter(|c| (' '..='~').contains(c) && !matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|'))
		.take(NAME_MAX)
		.collect();
	let kept = kept.trim().trim_end_matches('.').trim();
	if kept.is_empty() { FILE.to_owned() } else { format!("{kept}.mp4") }
}

/// The order's head (the body's JSON): the plan the page drew, the sound, the file's name, the knight and the layers.
#[derive(Deserialize)]
struct Head {
	plan: Reel,
	sound: bool,
	name: String,
	knight: KnightOrder,
	layers: Vec<LayerOrder>,
}

/// The knight's square in pixels and its look (Night's mask, a share of its radius; the well's corner radius).
#[derive(Deserialize)]
struct KnightOrder {
	x: usize,
	y: usize,
	size: usize,
	night: bool,
	mask: f64,
	radius: f64,
}

/// A layer: its place in pixels, its frames (`from..to`), whether it lies over the knight, its PNG's length.
#[derive(Deserialize)]
struct LayerOrder {
	x: usize,
	y: usize,
	from: usize,
	to: usize,
	over: bool,
	bytes: usize,
}

/// What a job makes: the checked plan, its films by shot (the moves, then the end), the sounds' files and times, the
/// knight, the layers' orders with their PNGs, the file's name and folder.
struct Job {
	plan: Reel,
	films: Vec<MoveFilm>,
	sounds: Vec<(PathBuf, i64)>,
	knight: Knight,
	layers: Vec<(LayerOrder, Vec<u8>)>,
	name: String,
	folder: PathBuf,
	timing: Timing,
}

/// The body: a 32-bit little-endian length, the head's JSON, then each layer's PNG in the head's order.
fn read(body: &[u8], t: &Timing) -> anyhow::Result<(Head, Vec<Vec<u8>>)> {
	if body.len() > BODY_MAX {
		bail!("a reel's order of {} bytes", body.len());
	}
	let len = body.get(..4).and_then(|b| b.try_into().ok()).map(u32::from_le_bytes).context("a reel's order")?;
	let end = 4 + usize::try_from(len)?;
	let head: Head = serde_json::from_slice(body.get(4..end).context("a reel's order is cut short")?)?;
	if head.layers.len() > LAYERS_MAX {
		bail!("{} layers", head.layers.len());
	}
	check(&head.plan, t)?;
	let frames = t.frame(head.plan.ms);
	let (w, h) = (usize::try_from(t.width)?, usize::try_from(t.height)?);
	let k = &head.knight;
	let inside = |at: usize, side: usize| at.checked_add(k.size).is_some_and(|reach| reach <= side);
	if k.size == 0 || !inside(k.x, w) || !inside(k.y, h) || !(0.0..=1.0).contains(&k.mask) || k.radius < 0.0 {
		bail!("a knight of {} at {}, {}", k.size, k.x, k.y);
	}
	let mut at = end;
	let mut pngs = Vec::with_capacity(head.layers.len());
	for l in &head.layers {
		if l.from >= l.to || l.to > frames {
			bail!("a layer on frames {}..{} of a reel of {frames}", l.from, l.to);
		}
		let next = at.checked_add(l.bytes).context("a layer's PNG is cut short")?;
		let png = body.get(at..next).context("a layer's PNG is cut short")?;
		pngs.push(png.to_vec());
		at = next;
	}
	if at != body.len() {
		bail!("{} bytes after the layers", body.len() - at);
	}
	Ok((head, pngs))
}

/// The plan as rule 18 lays it out with these tokens: its moves, at most `move_count`, each its shot from its place, the
/// end on the lead after them, its length, and its sounds inside it.
fn check(plan: &Reel, t: &Timing) -> anyhow::Result<()> {
	let n = plan.moves.len();
	if n == 0 || n > t.move_count || plan.moves.first().map(|s| s.mv) != Some(plan.lead) || plan.end.mv != plan.lead {
		bail!("a reel of {n} moves led by {}", plan.lead);
	}
	for (k, shot) in (0_i64..).zip(&plan.moves) {
		if shot.from != k * t.move_ms {
			bail!("a shot from {} ms", shot.from);
		}
	}
	let ends = i64::try_from(n)? * t.move_ms;
	if plan.end.from != ends || plan.ms != ends + t.end_ms || plan.sounds.iter().any(|c| !(0..plan.ms).contains(&c.at))
	{
		bail!("a reel of {} ms", plan.ms);
	}
	Ok(())
}

impl Helper {
	/// The start's part of the reels: the cache swept, the topic with its timing.
	pub(crate) fn reel_start(&self) {
		if let Some(folder) = folder(&self.app) {
			sweep(&folder, None);
		}
		publish(&self.app, &self.app.state::<Desk>().reels.lock());
	}

	/// The page's order for a reel (`reel_make`): refused while a pump runs, or where its plan, a move's film or a layer
	/// does not hold; else the reel being made is cancelled, this one's job starts on a blocking thread and its id comes
	/// back.
	pub(crate) fn reel_make(&mut self, body: &[u8]) -> Result<u64, String> {
		if self.desk.desk_pump_running() {
			return Err("a pump runs".to_owned()); // copy-lint: not shown (a command's error)
		}
		let timing = *TIMING.as_ref().ok_or("the reel's tokens do not read")?;
		let (head, pngs) = read(body, &timing).map_err(|e| format!("{e:#}"))?;
		let f = facts();
		let films = head
			.plan
			.moves
			.iter()
			.chain([&head.plan.end])
			.map(|shot| figures::film(f, shot.mv).ok_or_else(|| format!("move {} has no film here", shot.mv)))
			.collect::<Result<Vec<_>, _>>()?;
		let sounds_dir = self.app.path().resource_dir().ok().map(|dir| dir.join("resources").join("sounds"));
		let sounds = if head.sound {
			let dir = sounds_dir.ok_or("no sounds folder")?;
			head.plan.sounds.iter().map(|c| (dir.join(format!("{}.ogg", c.sound.id())), c.at)).collect()
		} else {
			Vec::new()
		};
		let root = folder(&self.app).ok_or("no cache folder")?;
		cancel(&self.app);
		let reels = &self.app.state::<Desk>().reels;
		let mut shown = reels.lock();
		let id = u64::try_from(lan::now_ms()).unwrap_or(0).max(shown.id + 1);
		let folder = root.join(id.to_string());
		sweep(&root, None);
		std::fs::create_dir_all(&folder).map_err(|e| format!("{}: {e}", folder.display()))?;
		let k = &head.knight;
		let look = if k.night { Look::Night { mask: k.mask } } else { Look::Well { radius: k.radius } };
		let job = Job {
			films,
			sounds,
			knight: Knight { x: k.x, y: k.y, size: k.size, look },
			layers: head.layers.into_iter().zip(pngs).collect(),
			name: file_name(&head.name),
			folder,
			timing,
			plan: head.plan,
		};
		let flag = Arc::new(AtomicBool::new(false));
		*shown = Shown { id, state: State::Making, job: Some(Arc::clone(&flag)), made: None, src: None, ..*shown };
		publish(&self.app, &shown);
		drop(shown);
		let app = self.app.clone();
		tauri::async_runtime::spawn_blocking(move || run(&app, id, &flag, &job));
		Ok(id)
	}
}

/// The job: the reel made, or why not; what it began is deleted unless it is made.
fn run(app: &AppHandle, id: u64, cancelled: &AtomicBool, job: &Job) {
	let folder = job.folder.clone();
	// a panic is a bug: the hook reports it, and the reel fails rather than stay making
	let made = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| make(job, cancelled)));
	let state = match made {
		Ok(Ok(file)) => {
			if settle(app, id, State::Ready, Some(file)) {
				log::info!("reel {id}: made, {} ms", job.plan.ms);
				return;
			}
			// cancelled as its last frame went in, or another took its place: it leaves no file
			log::info!("reel {id}: cancelled");
			None
		}
		Ok(Err(e)) if e.is::<Cancelled>() => {
			log::info!("reel {id}: cancelled");
			None
		}
		Ok(Err(e)) if e.is::<Unsupported>() => {
			log::warn!("reel {id}: {e:#}");
			Some(State::Unsupported)
		}
		Ok(Err(e)) => {
			log::warn!("reel {id}: {e:#}");
			Some(State::Failed)
		}
		Err(_) => Some(State::Failed),
	};
	if let Err(e) = std::fs::remove_dir_all(&folder) {
		log::warn!("reel {id}: {}: {e}", folder.display());
	}
	if let Some(state) = state {
		settle(app, id, state, None);
	}
}

/// Makes the reel's file: each frame drawn and encoded with the sound interleaved half a second ahead, the encoder's file
/// written again as `mp4::faststart` shapes it under the reel's name.
fn make(job: &Job, cancelled: &AtomicBool) -> anyhow::Result<PathBuf> {
	#[cfg(windows)]
	let _media = system::Session::start()?;
	let t = &job.timing;
	let (w, h) = (usize::try_from(t.width)?, usize::try_from(t.height)?);
	let pcm = if job.sounds.is_empty() {
		None
	} else {
		let cues: Vec<(&Path, i64)> = job.sounds.iter().map(|(p, at)| (p.as_path(), *at)).collect();
		Some(sound::mix(&cues, job.plan.ms, t.audio_hz, t.gain_db)?)
	};
	let encoding = Encoding {
		width: t.width,
		height: t.height,
		fps: t.fps,
		key_every: t.key_every,
		bitrate: t.bitrate_kbps * 1000,
		audio: pcm.as_ref().map(|_| (t.audio_hz, t.audio_kbps)),
	};
	let part = job.folder.join("reel.part.mp4");
	let mut writer = system::Writer::new(&part, &encoding)?;
	if let Err(e) = frames(job, pcm.as_deref(), &mut writer, cancelled, (w, h)) {
		writer.cancel();
		return Err(e);
	}
	writer.finish()?;
	let data = std::fs::read(&part).with_context(|| part.display().to_string())?;
	let file = job.folder.join(&job.name);
	std::fs::write(&file, mp4::faststart(&data)?).with_context(|| file.display().to_string())?;
	std::fs::remove_file(&part).with_context(|| part.display().to_string())?;
	Ok(file)
}

/// The layers as the frames draw them: each decoded from its PNG as its first frame comes and let go after its last,
/// so a reel holds only those it shows at once.
struct Stack<'a> {
	orders: &'a [(LayerOrder, Vec<u8>)],
	decoded: Vec<Option<Layer>>,
	size: (usize, usize),
}

impl<'a> Stack<'a> {
	fn new(orders: &'a [(LayerOrder, Vec<u8>)], size: (usize, usize)) -> Self {
		Self { orders, decoded: orders.iter().map(|_| None).collect(), size }
	}

	/// The layers of frame `frame` decoded, those past it let go.
	///
	/// # Errors
	/// A layer's PNG does not decode or leaves the frame.
	fn at(&mut self, frame: usize) -> anyhow::Result<()> {
		let (w, h) = self.size;
		for ((o, png), slot) in self.orders.iter().zip(&mut self.decoded) {
			if frame >= o.to {
				*slot = None;
			} else if frame >= o.from && slot.is_none() {
				*slot = Some(Layer::read(png, o.x, o.y, w, h)?);
			}
		}
		Ok(())
	}

	/// Draws the layers of the frame `at` last took in, under the knight or over it, in the page's order.
	fn draw(&self, canvas: &mut Canvas, over: bool) {
		for ((order, _), layer) in self.orders.iter().zip(&self.decoded) {
			if let Some(layer) = layer.as_ref().filter(|_| order.over == over) {
				layer.draw(canvas);
			}
		}
	}
}

/// Hands the sound to the writer, which interleaves it with the picture, then draws and encodes every frame, shot by
/// shot.
fn frames(
	job: &Job,
	pcm: Option<&[i16]>,
	writer: &mut system::Writer,
	cancelled: &AtomicBool,
	(w, h): (usize, usize),
) -> anyhow::Result<()> {
	let t = &job.timing;
	let total = t.frame(job.plan.ms);
	if let Some(pcm) = pcm {
		writer.audio(pcm)?;
	}
	let mut canvas = Canvas::new(w, h);
	let mut layers = Stack::new(&job.layers, (w, h));
	let shots: Vec<_> = job.plan.moves.iter().chain([&job.plan.end]).collect();
	for (k, (shot, film)) in shots.iter().zip(&job.films).enumerate() {
		let first = t.frame(shot.from);
		let last = shots.get(k + 1).map_or(total, |next| t.frame(next.from));
		let knights = squares(film, last - first, t.fps, cancelled)?;
		for (frame, square) in (first..last).zip(&knights) {
			if cancelled.load(Ordering::Relaxed) {
				return Err(anyhow::Error::new(Cancelled));
			}
			layers.at(frame)?;
			canvas.clear();
			layers.draw(&mut canvas, false);
			compose::knight(&mut canvas, square, film.mirror, job.knight);
			layers.draw(&mut canvas, true);
			writer.video(&canvas, i64::try_from(frame)?)?;
		}
	}
	Ok(())
}

/// The knight's square for each of a shot's `count` frames at `fps`: its film from its key pose at its real tempo,
/// wrapping at the film's end, each frame read once and tinted (a film the decoder found longer or shorter than its
/// `loop` says is read again with its real length).
fn squares(
	film: &MoveFilm,
	count: usize,
	fps: u32,
	cancelled: &AtomicBool,
) -> anyhow::Result<Vec<std::rc::Rc<Square>>> {
	let cut = FilmCut {
		side: usize::try_from(film.side)?,
		tint: usize::try_from(film.tint)?,
		tile: film.tile,
		colour: film.crimson,
	};
	let wanted = |length: usize| -> Vec<usize> {
		(0..count)
			.map(|j| {
				#[expect(clippy::cast_precision_loss, reason = "a frame count far below 2^52")]
				let at = film.key_s + j as f64 / f64::from(fps);
				#[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "a film's frame, floored")]
				let index = (at * FILM_FPS + 1e-6).floor().max(0.0) as usize;
				index % length.max(1)
			})
			.collect()
	};
	#[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "a film's frame count")]
	let mut length = (film.loop_s * FILM_FPS).round().max(1.0) as usize;
	for _ in 0..2 {
		let picks = wanted(length);
		let keep: HashSet<usize> = picks.iter().copied().collect();
		let mut kept: HashMap<usize, std::rc::Rc<Square>> = HashMap::new();
		let mut seen = 0;
		system::read_film(&film.path, &mut |frame: Frame<'_>| {
			#[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "a film's frame, rounded")]
			let index = (frame.at * FILM_FPS).round().max(0.0) as usize;
			seen = seen.max(index + 1);
			if keep.contains(&index)
				&& !cancelled.load(Ordering::Relaxed)
				&& let Some(square) = cut.square(frame.px, frame.width, frame.height, frame.stride)
			{
				kept.insert(index, std::rc::Rc::new(square));
			}
		})
		.with_context(|| film.path.display().to_string())?;
		if cancelled.load(Ordering::Relaxed) {
			return Err(anyhow::Error::new(Cancelled));
		}
		if seen != length && seen > 0 {
			length = seen;
			continue;
		}
		// a frame the decoder did not hand over shows the one before it (the film's first kept frame before any)
		let first = kept.keys().min().and_then(|i| kept.get(i));
		let mut last = first.cloned().context("the film has no frame")?;
		return Ok(picks
			.iter()
			.map(|i| {
				if let Some(square) = kept.get(i) {
					last = std::rc::Rc::clone(square);
				}
				std::rc::Rc::clone(&last)
			})
			.collect());
	}
	bail!("{}: its length changed between two reads", film.path.display())
}
