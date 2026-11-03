//! The desktop app's sound and voice (docs/ui.md, "The pump on the computer": "Sound and voice carry the moment", since a
//! computer has no haptics; docs/plan.md slice 35, "Sound and voice"). Every sound and line plays from Rust on the output
//! the player picks (its id in settings.json; while it is not connected, the system's own and a note on the Sound and
//! voice page), so a headset left on the desk swallows nothing and a window the game covers keeps its voice:
//! - a cue (`Helper::cue`, DESK-PUMP's schedule; the page's kit and the match card through `sound.event` and
//!   `offer_sound`) names a feedback event of `contract/design/tokens.json`, whose desktop column gives its sound (one
//!   of `contract/design/sound/sounds.json`, the `.ogg` tools/sound.py writes, bundled as `resources/sounds/`) and its
//!   voice line where it is one take (`side.switch`); a family (`cadence.*`, `count.*`) is the schedule's to pick, by
//!   Speech.lua's rules, as a line of its own;
//! - a line is the addon's own take by its VoiceClips.lua id (`resources/voice/<id>.ogg`, the voice the game speaks,
//!   `make voice`'s recordings), the match card's "Match over." (`desk.offer`) and the answer to `/qp ping`
//!   (`desk.ping`) among them; no line is ever said in a system voice (David, 2026-10-16: "the only voice we use is the
//!   elevenlabs v4"), so a take that does not read is a log line and silence; a new line stops the one before, as in
//!   the game;
//! - with a screen reader running (macOS's own, or one Windows says runs) while QuestPump's window has the focus, a
//!   pump's line goes to the page's one polite live region instead (the `shell` topic's `announce`), and its sounds
//!   still play; with the window behind another, hidden or minimized, the voice says it, since a screen reader seldom
//!   speaks a live region there;
//! - the rep count (`count.<n>` on a running set of reps' beat, DESK-PUMP's) is heard only with the sounds, the voice
//!   and "Count reps aloud" on, and never goes to the live region: while a screen reader takes the lines it is skipped
//!   (`Helper::counts_on`), since a number every second or two would pile up there. Those three switches alone decide
//!   whether Big view shows the reps counted and the count step starts from them (`Helper::counts_chosen`), so a
//!   screen reader or a skipped count never hides the number;
//! - a move's guide (Ready's "Read How and Tips aloud", DESK-PUMP's schedule) reads the game's own takes of the move's
//!   How and Tips from the QuestPump addon of the install the app follows (`Media/Voice/guide/`, which the app bundles
//!   none of), each only where that install's VoiceClips.lua holds its id with the hash of the words the app shows and
//!   its file is there, timed by that install's own length of it (`guide_read`), on a player of its own beside the
//!   line's: a line stops none of its takes, nor does a hush; it plays at the voice's level whatever the switches say
//!   (the player asked for it) and never goes to the live region;
//! - the volume is the master level times the sounds' or the voice's (`settings.sound.levels`, 0 to 100 %, 100 as
//!   recorded and the default; 2026-10-23, David: "proper sound settings like volume level (like any game)"), each
//!   level on an even scale of 40 dB (`gain`), set on a sound or a take as it starts and on those playing as a level
//!   moves; a silent level opens no output. A level sets how loud, never whether: the switches decide that, the rep
//!   count's included, and the live region is the screen reader's to voice.
//!
//! One thread owns the output: it opens the sink at the first sound and drops it once nothing has played for `IDLE`,
//! so an idle app holds no audio stream, and it publishes the `menu.sound` topic (the switches, the levels, the outputs,
//! the pick and its note). The page's intents are `sound.*`:
//! `{"type":"sound.on"|"sound.voice"|"sound.callouts"|"sound.count"|"sound.voice_offers", "on": bool}`,
//! `{"type":"sound.level", "channel": "master"|"sounds"|"voice", "percent": 0-100}` (the level kept),
//! `{"type":"sound.output", "id": string|null, "name": string}` (null: the system's output; the pick plays the test on
//! it, opened anew where it was not connected),
//! `{"type":"sound.picked"}` (the first pump's "Speakers or headset?" kept as it is), `{"type":"sound.test",
//! "channel"?: "master"|"sounds"|"voice"}` (the test, or a level's part of it: the master's the whole test and the
//! default, the sounds' the bell, the voice's the line; the page asks a level's once it has rested),
//! `{"type":"sound.look"}` (list the outputs again) and `{"type":"sound.event", "event": <a feedback event>}`.
//! `menu.sound` is `{"sound": bool, "voice": bool, "callouts": bool, "count": bool, "voice_offers": bool, "levels":
//! {"master", "sounds", "voice"} (each 0-100), "outputs": [{"id", "name", "default": bool}], "output": id|null, "gone":
//! name|null, "looked": bool}`; whether a screen reader runs is the `shell` topic's `reader` alone (one owner a fact),
//! which the page's note reads.

mod output;
mod system;

use std::collections::HashMap;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::sync::{Arc, LazyLock};
use std::time::{Duration, Instant, SystemTime};

use rodio::Player;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Manager as _};

use super::Intent;
use super::seams::{Cue, GuideTake, GuideTakes};
use crate::runtime::Helper;
use crate::settings::{Levels, Output, SoundSettings};
use crate::ui::{self, Announce};
use output::Listed;
pub(crate) use system::screen_reader;

/// The notices of what the sound brings into the app (Symphonia's MPL 2.0, which asks to say where its source is): the
/// desktop app's Acknowledgements lists them.
pub(crate) const NOTICES: &str = include_str!("../../notices/audio.txt");

/// The topic of the Sound and voice page.
const TOPIC: &str = "menu.sound";

/// How long the output stays open after the last sound ended.
const IDLE: Duration = Duration::from_secs(3);
/// How often the thread looks whether the output fell idle while it is open.
const LOOK: Duration = Duration::from_millis(500);
/// The pause between the test's sound and its line.
const TEST_GAP: Duration = Duration::from_millis(450);
/// The test: the go's struck bell, then a line of the game's voice.
const TEST_SOUND: &str = "go";
const TEST_LINE: &str = "cadence.count";
/// The match card's one soft sound: a window opening (`feedback.open`).
const OFFER_EVENT: &str = "open";
/// The match card's line, "Match over.", and the answer to `/qp ping`, "QuestPump here.": takes of the voice's list
/// (Speech.Lines).
const OFFER_LINE: &str = "desk.offer";
const PING_LINE: &str = "desk.ping";
/// The longest output name kept.
const NAME_MAX: usize = 200;
/// The span of the volume's scale, from 0 % to 100 %: twenty steps of 5 % at 2 dB each, the step Dr Lex's
/// "Programming volume controls" calls ideal. The system's own volume sits over it, so it need not reach down to the
/// threshold of hearing, as a device's 60 dB does, and its lowest levels stay audible beside a game.
const RANGE_DB: f32 = 40.0;
/// The level under which the scale fades straight to silence, so 0 % is silent and the lowest levels stay apart.
const ROLL_OFF: f32 = 0.1;

/// A level (0 to 100 %) as an amplitude: 0 silent, 100 as recorded, and between them an even scale of `RANGE_DB`, 0.4
/// dB a percent (half way is -20 dB, a step of 5 % 2 dB), faded linearly to silence under `ROLL_OFF`. A level taken as
/// the amplitude itself would leave most of the audible range in the slider's lowest fifth.
fn gain(level: u8) -> f32 {
	match level {
		0 => 0.0,
		100.. => 1.0,
		level => {
			let x = f32::from(level) / 100.0;
			let amplitude = rodio::math::db_to_linear(RANGE_DB * (x - 1.0));
			if x < ROLL_OFF { amplitude * x / ROLL_OFF } else { amplitude }
		}
	}
}

/// A level of the Sound and voice page (`sound.level`'s `channel`), and the part of the test it plays.
#[derive(Clone, Copy)]
enum Channel {
	Master,
	Sounds,
	Voice,
}

impl Channel {
	fn named(name: &str) -> Option<Self> {
		match name {
			"master" => Some(Self::Master),
			"sounds" => Some(Self::Sounds),
			"voice" => Some(Self::Voice),
			_ => None,
		}
	}

	fn level(self, levels: &mut Levels) -> &mut u8 {
		match self {
			Self::Master => &mut levels.master,
			Self::Sounds => &mut levels.sounds,
			Self::Voice => &mut levels.voice,
		}
	}
}

/// A feedback event's desktop column (`contract/design/tokens.json` `feedback.<event>.desktop`).
#[derive(Deserialize, Debug, Default)]
struct Column {
	sound: Option<String>,
	voice: Option<String>,
}

/// The feedback events' desktop columns by event, built in; a table that does not read leaves every event silent.
static FEEDBACK: LazyLock<HashMap<String, Column>> = LazyLock::new(|| {
	#[derive(Deserialize)]
	struct Event {
		desktop: Column,
	}
	#[derive(Deserialize)]
	struct Tokens {
		feedback: HashMap<String, Value>,
	}
	let tokens: Option<Tokens> = serde_json::from_str(include_str!("../../../../contract/design/tokens.json")).ok();
	tokens
		.map(|t| {
			t.feedback
				.into_iter()
				.filter_map(|(name, event)| Some((name, serde_json::from_value::<Event>(event).ok()?.desktop)))
				.collect()
		})
		.unwrap_or_default()
});

/// A voice line's receipt: the words it says, their djb2 hash (as VoiceClips.lua keeps it) and its take's length in
/// seconds.
#[derive(Deserialize)]
struct Clip {
	text: String,
	#[serde(default)]
	hash: u64,
	#[serde(default)]
	seconds: f64,
}

/// The voice's lines by their VoiceClips.lua id, with the words each says and its length (`tools/voice/clips.json`,
/// which the takes are recorded from), built in: the ids a take may be asked for, the words the live region gets in a
/// take's place while a screen reader hears the lines, and how long a line lasts before a callout follows it.
static CLIPS: LazyLock<HashMap<String, Clip>> =
	LazyLock::new(|| serde_json::from_str(include_str!("../../../../tools/voice/clips.json")).unwrap_or_default());

/// A move's guide's parts in the order the voice reads them (Speech.lua `GUIDE`), each line a take
/// `guide/<part>.<key>[.<gear>].<n>`.
const GUIDE: [&str; 2] = ["how", "tips"];

/// The guide's `["guide/…"] = <value>,` lines of a table of an install's VoiceClips.lua (`ns.<table> = {` to its `}`),
/// each its id and its value's text.
fn guide_lines<'a>(text: &'a str, table: &str) -> impl Iterator<Item = (&'a str, &'a str)> {
	let head = format!("ns.{table} = {{");
	text.lines()
		.skip_while(move |line| !line.starts_with(&head))
		.skip(1)
		.take_while(|line| !line.starts_with('}'))
		.filter_map(|line| line.trim().strip_prefix("[\"")?.split_once("\"] = "))
		.filter(|(id, _)| id.starts_with("guide/"))
		.map(|(id, value)| (id, value.trim_end_matches(',')))
}

/// The guide's takes in an install's VoiceClips.lua: each id's hash (`ns.VoiceClips`) and its take's length in seconds
/// (`ns.VoiceSeconds`, by which the game times its guide: Speech.lua), a take with both alone.
fn guide_takes(text: &str) -> HashMap<String, Take> {
	let seconds: HashMap<&str, f64> =
		guide_lines(text, "VoiceSeconds").filter_map(|(id, value)| Some((id, value.parse().ok()?))).collect();
	guide_lines(text, "VoiceClips")
		.filter_map(|(id, value)| Some((id.to_owned(), Take { hash: value.parse().ok()?, seconds: *seconds.get(id)? })))
		.collect()
}

/// A take of an install's guide: the djb2 hash of the words it says and its length in seconds.
#[derive(Clone, Copy)]
struct Take {
	hash: u64,
	seconds: f64,
}

/// An install's VoiceClips.lua as last read: its file, its modified time and its guide's takes (`guide_takes`; none
/// where it did not read).
struct Installed {
	file: PathBuf,
	modified: SystemTime,
	takes: HashMap<String, Take>,
}

/// A feedback event's name as the table holds it, for a cue (none for an event it lacks).
fn event(name: &str) -> Option<&'static str> {
	FEEDBACK.get_key_value(name).map(|(name, _)| name.as_str())
}

/// The loop's handle on the audio thread.
pub(crate) struct Audio {
	jobs: mpsc::Sender<Job>,
	/// The live region's last line.
	announced: u64,
	/// The followed install's VoiceClips.lua, read again once it changes (`Helper::guide_read`).
	installed: Option<Installed>,
}

/// What the loop asks the audio thread.
enum Job {
	/// The switches, the levels or the pick changed.
	Settings(SoundSettings),
	/// A sound by its sounds.json id.
	Sound(String),
	/// A take by its VoiceClips.lua id.
	Take(String),
	/// The test: the sound, then the line (`Master`), or a level's own part alone (the sounds' bell, the voice's line).
	Test(Channel),
	/// List the outputs again.
	Look,
	/// The player picked an output (`sound.output`, after its `Settings`): one picked again while it was not connected
	/// opens anew at the next sound, so an output plugged back in plays the test.
	Picked,
	/// The output failed (unplugged, the system took it): it opens again at the next sound.
	Lost,
	/// The line playing stops (`Helper::hush`).
	Hush,
	/// A take of a move's guide by its file, on the guide's player, the guide's take before stopping.
	Guide(PathBuf),
	/// The guide's take stops.
	GuideStop,
	/// The output stays open between sounds (a pump runs, its ticks a few seconds apart), or may fall idle again.
	Hold(bool),
}

impl Audio {
	/// Starts the audio thread on the bundle's resources (`resources/sounds/`, `resources/voice/`); without a resource
	/// folder nothing plays and the log says so once.
	pub(crate) fn start(app: &AppHandle, settings: &SoundSettings) -> Self {
		let (jobs, rx) = mpsc::channel();
		let resources = app.path().resource_dir().ok().map(|dir| dir.join("resources"));
		if resources.is_none() {
			log::warn!("sound: no resource folder, so no sound or take plays");
		}
		let (app, settings, own) = (app.clone(), settings.clone(), jobs.clone());
		// the thread makes its state itself: the open output stays on the thread that opened it
		let spawned = std::thread::Builder::new().name("audio".to_owned()).spawn(move || {
			let mut thread = Thread {
				app,
				resources,
				settings,
				jobs: own,
				sink: None,
				sounds: Vec::new(),
				voice: None,
				guide: None,
				bank: HashMap::new(),
				outputs: Vec::new(),
				gone: None,
				looked: false,
				quiet_since: None,
				held: false,
			};
			thread.run(&rx);
		});
		if let Err(e) = spawned {
			log::error!("sound: the audio thread did not start: {e}");
		}
		Self { jobs, announced: 0, installed: None }
	}

	fn send(&self, job: Job) {
		// the thread lives as long as the app; a send that fails finds it gone at the quit
		let _ = self.jobs.send(job);
	}
}

impl Helper {
	/// Plays a cue of the pump's sound and voice (`seams::Cue`): an event's sound and its one take, a line, words.
	pub(super) fn play_cue(&mut self, cue: Cue) {
		match cue {
			Cue::Event(name) => {
				let Some(column) = FEEDBACK.get(name) else {
					log::warn!("sound: no feedback event {name}");
					return;
				};
				if let Some(sound) = &column.sound
					&& self.settings.sound.on
				{
					self.audio.send(Job::Sound(sound.clone()));
				}
				// a family (`count.*`) is the schedule's to pick, as a line of its own
				if let Some(line) = column.voice.as_deref().filter(|v| !v.ends_with(".*")) {
					self.voice_line(line.to_owned());
				}
			}
			Cue::Line(id) => self.voice_line(id),
		}
	}

	/// The card's one soft sound and, for a match's card (`line`) with the voice on for offers, its take "Match over."
	/// (never the live region: the card is no page a screen reader reads, and the offer stands on Pump home where it
	/// does); a reminder's card plays the sound alone.
	pub(super) fn play_offer(&mut self, line: bool) {
		if self.settings.sound.on
			&& let Some(sound) = FEEDBACK.get(OFFER_EVENT).and_then(|c| c.sound.clone())
		{
			self.audio.send(Job::Sound(sound));
		}
		if line && self.settings.sound.voice_offers {
			self.audio.send(Job::Take(OFFER_LINE.to_owned()));
		}
	}

	/// The answer to `/qp ping`, the take "QuestPump here.", whatever the switches say: the player sent the signal to
	/// hear it (the Computer page's screen line shows it too). The levels still set how loud.
	pub(crate) fn play_ping(&self) {
		self.audio.send(Job::Take(PING_LINE.to_owned()));
	}

	/// A take by its id: to the live region while the screen reader hears it (`reader_hears`), else the voice when it
	/// is on, a move's guide held while it plays (`guide_hold`). An id the voice does not know says nothing.
	fn voice_line(&mut self, id: String) {
		let Some((words, seconds)) = CLIPS.get(&id).map(|clip| (clip.text.clone(), clip.seconds)) else {
			log::warn!("sound: no voice line {id}");
			return;
		};
		if self.reader_hears() {
			self.announce(words);
		} else if self.settings.sound.voice {
			self.guide_hold(&id, seconds);
			self.audio.send(Job::Take(id));
		}
	}

	/// The takes of move `key`'s guide (Speech.lua `GuideTakes`): How's, then Tips' from where `tips` says, each part
	/// the implement's own (`gear`) where the voice has it, each take from the QuestPump addon of the install the app
	/// follows, timed by that install's own length of it, and only where that install's VoiceClips.lua holds its id with
	/// the hash of the words the app shows (`tools/voice/clips.json`, the words of `contract/moves.json`) and its file is
	/// there, so an addon of another version never reads other words; a part stops at its first take without them, as
	/// the game's stops at the first id it lacks, so the voice skips no step the screen shows; none without such an
	/// install.
	pub(super) fn guide_read(&mut self, key: &str, gear: Option<&str>) -> GuideTakes {
		let Some(addon) = self.followed.as_ref().map(|addons| addons.join("QuestPump")) else {
			return GuideTakes::default();
		};
		let Some(installed) = self.installed_guide(&addon.join("VoiceClips.lua")) else {
			return GuideTakes::default();
		};
		let voice = addon.join("Media").join("Voice").join("guide");
		let mut guide = GuideTakes::default();
		for part in GUIDE {
			let own = format!("guide/{part}.{key}"); // copy-lint: not shown (a voice take's id)
			let base = gear
				.map(|gear| format!("{own}.{gear}"))
				.filter(|base| CLIPS.contains_key(&format!("{base}.1")))
				.unwrap_or(own);
			let from = guide.takes.len();
			for n in 1.. {
				let id = format!("{base}.{n}");
				let Some(clip) = CLIPS.get(&id) else { break };
				let Some(take) = installed.get(&id).filter(|take| take.hash == clip.hash) else { break };
				let Some(file) = id.strip_prefix("guide/").map(|file| voice.join(format!("{file}.ogg"))) else { break };
				if !file.is_file() {
					break;
				}
				guide.takes.push(GuideTake { file, seconds: take.seconds });
			}
			if part == "tips" && from > 0 && guide.takes.len() > from {
				guide.tips = Some(from);
			}
		}
		guide
	}

	/// The guide's takes of the install's VoiceClips.lua at `file` (`guide_takes`), read again only once it changes;
	/// none where it is not there.
	fn installed_guide(&mut self, file: &Path) -> Option<&HashMap<String, Take>> {
		let modified = std::fs::metadata(file).and_then(|m| m.modified()).ok()?;
		if !self.audio.installed.as_ref().is_some_and(|i| i.file == file && i.modified == modified) {
			let takes = std::fs::read_to_string(file).map_or_else(
				|e| {
					log::warn!("sound: {}: {e}", file.display());
					HashMap::new()
				},
				|text| guide_takes(&text),
			);
			self.audio.installed = Some(Installed { file: file.to_owned(), modified, takes });
		}
		self.audio.installed.as_ref().map(|i| &i.takes)
	}

	/// Plays a take of a move's guide (`guide_read`'s file) on the guide's own player, stopping its take before.
	pub(super) fn play_guide(&self, file: PathBuf) {
		self.audio.send(Job::Guide(file));
	}

	/// Stops the guide's take.
	pub(super) fn stop_guide(&self) {
		self.audio.send(Job::GuideStop);
	}

	/// A take's length in seconds (its receipt's), none for a line with no take.
	pub(super) fn take_length(id: &str) -> Option<f64> {
		CLIPS.get(id).map(|clip| clip.seconds)
	}

	/// The Sound and voice page's Callouts switch: a move's callout follows its line (the voice's switch and a screen
	/// reader still decide whether a line is heard, `voice_line`).
	pub(super) fn callouts_wanted(&self) -> bool {
		self.settings.sound.callouts
	}

	/// The player's switches let the voice count reps (`Helper::counts_chosen`): the sounds, the voice and "Count reps
	/// aloud" on, whether or not a count is heard now.
	pub(super) fn count_switched(&self) -> bool {
		let s = &self.settings.sound;
		s.on && s.voice && s.count
	}

	/// The rep count goes to the voice now (`Helper::counts_on`; at the voice's level, a silent one playing nothing):
	/// the player's switches let the voice count (`count_switched`), and no screen reader takes the voice's lines
	/// (`reader_hears`), since a count never goes to the live region.
	pub(super) fn counts_wanted(&self) -> bool {
		self.count_switched() && !self.reader_hears()
	}

	/// A screen reader runs and the main window has the keyboard's focus (the `shell` topic's `focused`, Rust's
	/// `WindowEvent::Focused`, which a hidden, minimized or closed window has lost): only then does the page's live
	/// region reach the player.
	fn reader_hears(&self) -> bool {
		screen_reader() && self.app.state::<ui::Shared>().view.lock().is_ok_and(|v| v.shell.focused)
	}

	/// Stops the line playing (`Helper::hush`).
	pub(super) fn hush_voice(&self) {
		self.audio.send(Job::Hush);
	}

	/// Keeps the output open while a pump runs (`Helper::hold_sound`).
	pub(super) fn hold_output(&self, on: bool) {
		self.audio.send(Job::Hold(on));
	}

	fn announce(&mut self, text: String) {
		self.audio.announced += 1;
		let seq = self.audio.announced;
		ui::update(&self.app, |v| v.shell.announce = Some(Announce { text, seq }));
	}

	/// The start's part of the sound: whether the player chose an output yet.
	pub(crate) fn sound_start(&self) {
		let picked = self.settings.sound.output_picked;
		ui::update(&self.app, |v| v.shell.output_picked = picked);
	}

	/// The Sound and voice page's intents and the kit's feedback events (`sound.*`, the module doc).
	pub(crate) fn sound_intent(&mut self, intent: &Intent) -> Result<(), String> {
		let switch = || intent.arg("on").and_then(Value::as_bool).ok_or_else(|| format!("{} takes on", intent.kind()));
		let mut picked = false;
		match intent.kind() {
			"sound.on" => self.settings.sound.on = switch()?,
			"sound.voice" => self.settings.sound.voice = switch()?,
			"sound.callouts" => self.settings.sound.callouts = switch()?,
			"sound.count" => self.settings.sound.count = switch()?,
			"sound.voice_offers" => self.settings.sound.voice_offers = switch()?,
			"sound.level" => {
				let channel = intent
					.arg("channel")
					.and_then(Value::as_str)
					.and_then(Channel::named)
					.ok_or("sound.level takes a channel: master, sounds or voice")?;
				let percent = intent
					.arg("percent")
					.and_then(Value::as_u64)
					.and_then(|p| u8::try_from(p).ok())
					.filter(|p| *p <= 100)
					.ok_or("sound.level takes a percent from 0 to 100")?;
				*channel.level(&mut self.settings.sound.levels) = percent;
			}
			"sound.output" => {
				let output = match intent.arg("id") {
					Some(Value::Null) | None => None,
					Some(Value::String(id)) => {
						let name = intent.arg("name").and_then(Value::as_str).unwrap_or_default();
						Some(Output { id: id.clone(), name: name.chars().take(NAME_MAX).collect() })
					}
					Some(_) => return Err("sound.output takes an id or null".to_owned()),
				};
				self.settings.sound.output = output;
				self.settings.sound.output_picked = true;
				picked = true;
			}
			"sound.picked" => self.settings.sound.output_picked = true,
			"sound.test" => {
				let channel = match intent.arg("channel") {
					None | Some(Value::Null) => Channel::Master,
					Some(name) => name
						.as_str()
						.and_then(Channel::named)
						.ok_or("sound.test takes a channel: master, sounds or voice")?,
				};
				self.audio.send(Job::Test(channel));
				return Ok(());
			}
			"sound.look" => {
				self.audio.send(Job::Look);
				return Ok(());
			}
			"sound.event" => {
				let name = intent.arg("event").and_then(Value::as_str).ok_or("sound.event takes an event")?;
				let event = event(name).ok_or_else(|| format!("sound.event: no feedback event {name}"))?;
				self.cue(Cue::Event(event));
				return Ok(());
			}
			_ => return Err(intent.unknown()),
		}
		self.save_settings();
		self.audio.send(Job::Settings(self.settings.sound.clone()));
		self.sound_start();
		if picked {
			self.audio.send(Job::Picked);
			self.audio.send(Job::Test(Channel::Master));
		}
		Ok(())
	}
}

/// The `menu.sound` topic.
#[derive(Serialize)]
#[expect(clippy::struct_excessive_bools, reason = "the page's switches and notes, read as they are")]
struct Page<'a> {
	sound: bool,
	voice: bool,
	callouts: bool,
	count: bool,
	voice_offers: bool,
	levels: &'a Levels,
	outputs: &'a [Listed],
	output: Option<&'a str>,
	gone: Option<&'a str>,
	looked: bool,
}

/// The audio thread's state.
struct Thread {
	app: AppHandle,
	resources: Option<PathBuf>,
	settings: SoundSettings,
	jobs: mpsc::Sender<Job>,
	sink: Option<output::Opened>,
	/// The sounds playing (each its own player, so two overlap).
	sounds: Vec<Player>,
	/// The line playing.
	voice: Option<Player>,
	/// A move's guide's take playing (`Job::Guide`), beside the line.
	guide: Option<Player>,
	/// The sounds read so far, by id.
	bank: HashMap<String, Arc<[u8]>>,
	outputs: Vec<Listed>,
	/// The picked output's name while it is not connected.
	gone: Option<String>,
	looked: bool,
	/// When the last sound ended, while the output is open.
	quiet_since: Option<Instant>,
	/// A pump runs: the output stays open between its sounds.
	held: bool,
}

impl Thread {
	fn run(&mut self, jobs: &mpsc::Receiver<Job>) {
		self.publish();
		loop {
			// open: a look every LOOK for the idle drop; closed: nothing wakes the thread but a job
			let job = if self.sink.is_some() {
				match jobs.recv_timeout(LOOK) {
					Ok(job) => Some(job),
					Err(RecvTimeoutError::Timeout) => None,
					Err(RecvTimeoutError::Disconnected) => return,
				}
			} else {
				match jobs.recv() {
					Ok(job) => Some(job),
					Err(_) => return,
				}
			};
			if let Some(job) = job {
				self.take(job);
			}
			self.idle();
		}
	}

	fn take(&mut self, job: Job) {
		match job {
			Job::Settings(settings) => {
				let moved = settings.output != self.settings.output;
				self.settings = settings;
				if moved {
					self.close();
					self.gone = None;
				}
				// a level that moved reaches what plays now too
				let (sounds, voice) = (self.sounds_gain(), self.voice_gain());
				for player in &self.sounds {
					player.set_volume(sounds);
				}
				for player in self.voice.iter().chain(&self.guide) {
					player.set_volume(voice);
				}
				self.publish();
			}
			Job::Sound(id) => self.sound(&id, Duration::ZERO),
			Job::Take(id) => self.take_line(&id, Duration::ZERO),
			Job::Test(channel) => {
				let bell = !matches!(channel, Channel::Voice);
				if bell {
					self.sound(TEST_SOUND, Duration::ZERO);
				}
				if !matches!(channel, Channel::Sounds) {
					self.take_line(TEST_LINE, if bell { TEST_GAP } else { Duration::ZERO });
				}
			}
			Job::Look => {
				self.look();
				self.publish();
			}
			Job::Picked => {
				if self.gone.is_some() {
					self.close();
				}
			}
			Job::Lost => self.close(),
			Job::Hush => self.stop_voice(),
			Job::Guide(file) => self.guide_take(&file),
			Job::GuideStop => {
				if let Some(guide) = self.guide.take() {
					guide.stop();
				}
			}
			Job::Hold(on) => self.held = on,
		}
	}

	/// The amplitude the sounds play at: the master level's times their own.
	fn sounds_gain(&self) -> f32 {
		gain(self.settings.levels.master) * gain(self.settings.levels.sounds)
	}

	/// The amplitude the voice's takes play at: the master level's times its own.
	fn voice_gain(&self) -> f32 {
		gain(self.settings.levels.master) * gain(self.settings.levels.voice)
	}

	/// Plays a sound by its sounds.json id after `delay` at the sounds' level, none at a silent one; one that does not
	/// read is logged.
	fn sound(&mut self, id: &str, delay: Duration) {
		let volume = self.sounds_gain();
		if volume <= 0.0 {
			return;
		}
		let Some(bytes) = self.read_sound(id) else { return };
		let Some(open) = self.open() else { return };
		match output::play(&open.sink, Cursor::new(bytes), delay, volume) {
			Ok(player) => {
				self.sounds.push(player);
				self.quiet_since = None;
			}
			Err(e) => log::warn!("sound: {id} does not decode: {e}"),
		}
	}

	/// A sound's file, read once and kept.
	fn read_sound(&mut self, id: &str) -> Option<Arc<[u8]>> {
		if let Some(bytes) = self.bank.get(id) {
			return Some(Arc::clone(bytes));
		}
		let path = self.resources.as_ref()?.join("sounds").join(format!("{id}.ogg"));
		match std::fs::read(&path) {
			Ok(bytes) => {
				let bytes: Arc<[u8]> = bytes.into();
				self.bank.insert(id.to_owned(), Arc::clone(&bytes));
				Some(bytes)
			}
			Err(e) => {
				log::warn!("sound: {}: {e}", path.display());
				None
			}
		}
	}

	/// Plays a take after `delay` at the voice's level, stopping the line before (a silent level plays none); one that
	/// does not read is a log line and silence.
	fn take_line(&mut self, id: &str, delay: Duration) {
		self.stop_voice();
		let volume = self.voice_gain();
		if volume <= 0.0 {
			return;
		}
		let file = self.resources.as_ref().map(|dir| dir.join("voice").join(format!("{id}.ogg")));
		if let Some(Ok(file)) = file.as_deref().map(std::fs::File::open) {
			self.voice = self.player(file, delay, volume, id);
		} else {
			log::warn!("sound: the take {id} does not read");
		}
	}

	/// Plays a take of a move's guide at the voice's level, stopping the guide's take before (a silent level plays
	/// none); the line playing goes on. One that does not read is a log line and silence.
	fn guide_take(&mut self, file: &Path) {
		if let Some(guide) = self.guide.take() {
			guide.stop();
		}
		let volume = self.voice_gain();
		if volume <= 0.0 {
			return;
		}
		match std::fs::File::open(file) {
			Ok(data) => self.guide = self.player(data, Duration::ZERO, volume, &file.display().to_string()),
			Err(e) => log::warn!("sound: {}: {e}", file.display()),
		}
	}

	/// The line before stops: a new one has come (Speech.lua's rule).
	fn stop_voice(&mut self) {
		if let Some(voice) = self.voice.take() {
			voice.stop();
		}
	}

	/// A take on a player of its own after `delay` at `volume`; none where the output does not open or the take does
	/// not decode (logged).
	fn player<R>(&mut self, data: R, delay: Duration, volume: f32, what: &str) -> Option<Player>
	where
		R: std::io::Read + std::io::Seek + Send + Sync + 'static,
	{
		let open = self.open()?;
		match output::play(&open.sink, data, delay, volume) {
			Ok(player) => {
				self.quiet_since = None;
				Some(player)
			}
			Err(e) => {
				log::warn!("sound: {what} does not decode: {e}");
				None
			}
		}
	}

	/// The output, opened when it is not: the one picked while it is connected, else the system's own (and the note
	/// that the picked one is gone).
	fn open(&mut self) -> Option<&output::Opened> {
		if self.sink.is_none() {
			let picked = self.settings.output.as_ref();
			let lost = self.jobs.clone();
			let opened = output::open(picked.map(|o| o.id.as_str()), move |e| {
				log::warn!("sound: the output: {e}");
				let _ = lost.send(Job::Lost);
			});
			let opened = match opened {
				Ok(opened) => opened,
				Err(e) => {
					log::warn!("sound: the output does not open: {e}");
					return None;
				}
			};
			let gone = picked.filter(|_| !opened.on_picked).map(|o| o.name.clone());
			if gone != self.gone {
				if let Some(name) = &gone {
					log::info!("sound: {name} is not connected; the system's output plays");
				}
				self.gone = gone;
				self.publish();
			}
			self.sink = Some(opened);
		}
		self.sink.as_ref()
	}

	/// Lets the output go: nothing more plays on it.
	fn close(&mut self) {
		self.sounds.clear();
		self.voice = None;
		self.guide = None;
		if self.sink.take().is_some() {
			log::debug!("sound: the output closes");
		}
		self.quiet_since = None;
	}

	/// Drops the output once nothing has played for `IDLE`, unless a pump holds it.
	fn idle(&mut self) {
		if self.sink.is_none() {
			return;
		}
		self.sounds.retain(|p| !p.empty());
		for player in [&mut self.voice, &mut self.guide] {
			if player.as_ref().is_some_and(Player::empty) {
				*player = None;
			}
		}
		if self.held || !self.sounds.is_empty() || self.voice.is_some() || self.guide.is_some() {
			self.quiet_since = None;
			return;
		}
		let since = *self.quiet_since.get_or_insert_with(Instant::now);
		if since.elapsed() >= IDLE {
			self.close();
		}
	}

	/// Lists the outputs the system has now, and from them whether the picked one is connected (its note, before any
	/// sound opens the output); an output open on the system's own while the picked one is back closes, so the next
	/// sound plays on the picked one.
	fn look(&mut self) {
		match output::list() {
			Ok(outputs) => {
				self.outputs = outputs;
				let picked = self.settings.output.as_ref();
				let back = picked.is_some_and(|p| self.outputs.iter().any(|o| o.id == p.id));
				self.gone = picked.filter(|_| !back).map(|p| p.name.clone());
				if back && self.sink.as_ref().is_some_and(|open| !open.on_picked) {
					self.close();
				}
			}
			Err(e) => {
				log::warn!("sound: the outputs do not list: {e}");
				self.outputs = Vec::new();
			}
		}
		self.looked = true;
	}

	fn publish(&self) {
		let page = Page {
			sound: self.settings.on,
			voice: self.settings.voice,
			callouts: self.settings.callouts,
			count: self.settings.count,
			voice_offers: self.settings.voice_offers,
			levels: &self.settings.levels,
			outputs: &self.outputs,
			output: self.settings.output.as_ref().map(|o| o.id.as_str()),
			gone: self.gone.as_deref(),
			looked: self.looked,
		};
		self.app.state::<super::Desk>().publisher.publish(TOPIC, &page);
	}
}
