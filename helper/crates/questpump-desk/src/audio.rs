//! The pump's cue policy (docs/ui.md, "The pump on the computer": "Sound and voice carry the moment", since a computer
//! has no haptics; docs/plan.md slice 35, "Sound and voice"): what a cue of DESK-PUMP's schedule plays, by the player's
//! switches, and where. The platform plays it (`Platform::play_sound`, `play_take`) on its output, the desktop app's on
//! the output the player picks (its `desk/audio.rs`, which also owns the outputs and the `menu.sound` topic):
//! - a cue (`Desk::cue`, DESK-PUMP's schedule; the page's kit through `sound.event`, the match card through the
//!   platform's own) names a
//!   feedback event of `contract/design/tokens.json`, whose desktop column gives its sound (one of
//!   `contract/design/sound/sounds.json`) and its voice line where it is one take (`side.switch`); a family
//!   (`cadence.*`, `count.*`) is the schedule's to pick, by Speech.lua's rules, as a line of its own;
//! - a line is the addon's own take by its VoiceClips.lua id; no line is ever said in a system voice (David,
//!   2026-10-16: "the only voice we use is the elevenlabs v4"), so a take that does not read is silence;
//! - with a screen reader that hears the page (`Platform::reader_hears`), a pump's line goes to the page's one polite
//!   live region instead (`Platform::announce`), and its sounds still play;
//! - the rep count (`count.<n>` on a running set of reps' beat) is heard only with the sounds, the voice and "Count
//!   reps aloud" on, and never goes to the live region: while a screen reader takes the lines it is skipped
//!   (`Desk::counts_on`). Those three switches alone decide whether Big view shows the reps counted and the count step
//!   starts from them (`Desk::counts_chosen`), so a screen reader or a skipped count never hides the number.
//!
//! The Sound and voice page's switches, levels, test and the kit's feedback events are the desk's intents (`sound.*`,
//! `Desk::sound_intent`): `{"type":"sound.on"|"sound.voice"|"sound.callouts"|"sound.count"|"sound.voice_offers",
//! "on": bool}`, `{"type":"sound.level", "channel": "master"|"sounds"|"voice", "percent": 0-100}` (the level kept),
//! `{"type":"sound.picked"}` (the first pump's "Speakers or headset?" kept as it is), `{"type":"sound.test",
//! "channel"?: "master"|"sounds"|"voice"}` (the test, or a level's part of it: the master's the whole test and the
//! default, the sounds' the bell, the voice's the line; the page asks a level's once it has rested) and
//! `{"type":"sound.event", "event": <a feedback event>}`; the platform plays at what is kept
//! (`Platform::sound_changed`) and plays the test (`Platform::sound_test`). Picking an output (`sound.output`,
//! `sound.look`) is the desktop app's.

use std::collections::HashMap;
use std::sync::LazyLock;

use serde::Deserialize;
use serde_json::Value;

use crate::seams::Cue;
use crate::settings::Levels;
use crate::{Desk, Intent, Platform};

/// A feedback event's desktop column (`contract/design/tokens.json` `feedback.<event>.desktop`).
#[derive(Deserialize, Debug, Default)]
pub struct Column {
	pub sound: Option<String>,
	pub voice: Option<String>,
}

/// The feedback events' desktop columns by event, built in; a table that does not read leaves every event silent.
pub static FEEDBACK: LazyLock<HashMap<String, Column>> = LazyLock::new(|| {
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
pub struct Clip {
	pub text: String,
	#[serde(default)]
	pub hash: u64,
	#[serde(default)]
	pub seconds: f64,
}

/// The voice's lines by their VoiceClips.lua id, with the words each says and its length (`tools/voice/clips.json`,
/// which the takes are recorded from), built in: the ids a take may be asked for, the words the live region gets in a
/// take's place while a screen reader hears the lines, and how long a line lasts before a callout follows it.
pub static CLIPS: LazyLock<HashMap<String, Clip>> =
	LazyLock::new(|| serde_json::from_str(include_str!("../../../../tools/voice/clips.json")).unwrap_or_default());

/// A feedback event's name as the table holds it, for a cue (none for an event it lacks).
pub fn event(name: &str) -> Option<&'static str> {
	FEEDBACK.get_key_value(name).map(|(name, _)| name.as_str())
}

impl<P: Platform> Desk<P> {
	/// Plays a cue of the pump's sound and voice (`seams::Cue`): an event's sound and its one take, a line, words.
	pub(crate) fn play_cue(&mut self, cue: Cue) {
		match cue {
			Cue::Event(name) => {
				let Some(column) = FEEDBACK.get(name) else {
					log::warn!("sound: no feedback event {name}");
					return;
				};
				if let Some(sound) = &column.sound
					&& self.settings.sound.on
				{
					self.p.play_sound(sound);
				}
				// a family (`count.*`) is the schedule's to pick, as a line of its own
				if let Some(line) = column.voice.as_deref().filter(|v| !v.ends_with(".*")) {
					self.voice_line(line);
				}
			}
			Cue::Line(id) => self.voice_line(&id),
		}
	}

	/// A take by its id: to the live region while the screen reader hears it (`reader_hears`), else the voice when it
	/// is on, a move's guide held while it plays (`guide_hold`). An id the voice does not know says nothing.
	fn voice_line(&mut self, id: &str) {
		let Some((words, seconds)) = CLIPS.get(id).map(|clip| (clip.text.clone(), clip.seconds)) else {
			log::warn!("sound: no voice line {id}");
			return;
		};
		if self.p.reader_hears() {
			self.p.announce(words);
		} else if self.settings.sound.voice {
			self.guide_hold(id, seconds);
			self.p.play_take(id);
		}
	}

	/// A take's length in seconds (its receipt's), none for a line with no take.
	pub(crate) fn take_length(id: &str) -> Option<f64> {
		CLIPS.get(id).map(|clip| clip.seconds)
	}

	/// The Sound and voice page's Callouts switch: a move's callout follows its line (the voice's switch and a screen
	/// reader still decide whether a line is heard, `voice_line`).
	pub(crate) fn callouts_wanted(&self) -> bool {
		self.settings.sound.callouts
	}

	/// The player's switches let the voice count reps (`Desk::counts_chosen`): the sounds, the voice and "Count reps
	/// aloud" on, whether or not a count is heard now.
	pub(crate) fn count_switched(&self) -> bool {
		let s = &self.settings.sound;
		s.on && s.voice && s.count
	}

	/// The rep count goes to the voice now (`Desk::counts_on`; at the voice's level, a silent one playing nothing):
	/// the player's switches let the voice count (`count_switched`), and no screen reader takes the voice's lines
	/// (`reader_hears`), since a count never goes to the live region.
	pub(crate) fn counts_wanted(&self) -> bool {
		self.count_switched() && !self.p.reader_hears()
	}
}

/// A level of the Sound and voice page, or the part of the test it plays: the master's (the whole test), the sounds'
/// (the bell) or the voice's (the line).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channel {
	Master,
	Sounds,
	Voice,
}

impl Channel {
	/// The channel the page names (`master`, `sounds`, `voice`).
	#[must_use]
	pub fn named(name: &str) -> Option<Self> {
		match name {
			"master" => Some(Self::Master),
			"sounds" => Some(Self::Sounds),
			"voice" => Some(Self::Voice),
			_ => None,
		}
	}

	/// Its level among `levels`.
	pub fn level(self, levels: &mut Levels) -> &mut u8 {
		match self {
			Self::Master => &mut levels.master,
			Self::Sounds => &mut levels.sounds,
			Self::Voice => &mut levels.voice,
		}
	}
}

impl<P: Platform> Desk<P> {
	/// The Sound and voice page's intents and the kit's feedback events (`sound.*`, the module doc): a switch, a level
	/// or the pick kept is saved and played at (`Platform::sound_changed`).
	///
	/// # Errors
	/// The intent is none of the desk's, or its fields do not read.
	pub fn sound_intent(&mut self, intent: &Intent) -> Result<(), String> {
		let switch = || intent.arg("on").and_then(Value::as_bool).ok_or_else(|| format!("{} takes on", intent.kind()));
		let sound = &mut self.settings.sound;
		match intent.kind() {
			"sound.on" => sound.on = switch()?,
			"sound.voice" => sound.voice = switch()?,
			"sound.callouts" => sound.callouts = switch()?,
			"sound.count" => sound.count = switch()?,
			"sound.voice_offers" => sound.voice_offers = switch()?,
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
				*channel.level(&mut sound.levels) = percent;
			}
			"sound.picked" => sound.output_picked = true,
			"sound.test" => {
				let channel = match intent.arg("channel") {
					None | Some(Value::Null) => Channel::Master,
					Some(name) => name
						.as_str()
						.and_then(Channel::named)
						.ok_or("sound.test takes a channel: master, sounds or voice")?,
				};
				self.p.sound_test(channel);
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
		self.p.sound_changed(&self.settings.sound);
		Ok(())
	}
}
