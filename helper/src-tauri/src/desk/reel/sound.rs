//! A reel's sound (docs/ui.md, "The share card": the Reel; contract/design/sound/sounds.json, the app's own CC0
//! sounds, never licensed music): the reel's cues (`Reel::sounds`, rule 18: `reveal` at the number's pop, `fanfare` or
//! `record` as the end begins), each the bundle's `resources/sounds/<id>.ogg` decoded whole by rodio (the Ogg Vorbis
//! the app plays everywhere), made stereo (a mono sound on both sides, a sound of more channels by its first two),
//! brought to the reel's rate between its two nearest samples where its own differs, and mixed at
//! `surface.SagaReel.gainDb` from its time into one buffer as long as the reel: 16-bit PCM, interleaved, the encoders'
//! AAC input. A sound whose file is missing, does not decode or holds no sound is logged and its cue left silent, the
//! reel made without it (`surface.SagaReel`'s one rule on the three apps). Each sound is read through to its end before
//! it is mixed: rodio's resampling iterator ends at once on a decoder whose first span reads as empty, as these files'
//! do (seen 2026-10-16: a reel mixed silent).

use std::path::Path;

use anyhow::{Context as _, bail};
use rodio::Source as _;

/// A sound decoded whole: its frames as left and right, and its rate (Hz).
struct Decoded {
	frames: Vec<[f32; 2]>,
	rate: u32,
}

/// Decodes a sound's file whole, in stereo.
///
/// # Errors
/// The file does not read or decode, or holds no sound.
fn decode(path: &Path) -> anyhow::Result<Decoded> {
	let file = std::fs::File::open(path).with_context(|| path.display().to_string())?;
	let decoder = rodio::Decoder::try_from(file).with_context(|| path.display().to_string())?;
	let (rate, channels) = (decoder.sample_rate().get(), usize::from(decoder.channels().get()));
	let samples: Vec<f32> = decoder.collect();
	let frames = samples
		.chunks_exact(channels)
		.map(|frame| match *frame {
			[mono] => [mono, mono],
			[left, right, ..] => [left, right],
			[] => [0.0, 0.0],
		})
		.collect::<Vec<_>>();
	if frames.is_empty() {
		bail!("{} holds no sound", path.display());
	}
	Ok(Decoded { frames, rate })
}

/// The mixed sound of a reel `ms` long at `hz` in stereo: each of `cues` (its file and its time in ms) at `gain_db`,
/// clipped where they meet; silence between them and for a cue whose sound does not read, which is logged.
///
/// # Errors
/// The reel's length or a cue's time is out of range.
pub(super) fn mix(cues: &[(&Path, i64)], ms: i64, hz: u32, gain_db: f64) -> anyhow::Result<Vec<i16>> {
	let frames = usize::try_from(ms.max(0) * i64::from(hz) / 1000)?;
	let mut out = vec![[0.0_f32; 2]; frames];
	#[expect(clippy::cast_possible_truncation, reason = "a gain near 1")]
	let gain = 10_f64.powf(gain_db / 20.0) as f32;
	for &(path, at) in cues {
		let sound = match decode(path) {
			Ok(sound) => sound,
			Err(e) => {
				log::warn!("reel: its sound at {at} ms left silent: {e:#}");
				continue;
			}
		};
		// `decode` hands over a frame at least
		let last = sound.frames.len() - 1;
		let from = usize::try_from(at.max(0) * i64::from(hz) / 1000)?;
		// the reel's frame k is the sound's frame k x its rate / the reel's, between its two nearest
		let step = f64::from(sound.rate) / f64::from(hz);
		for (k, slot) in out.iter_mut().skip(from).enumerate() {
			#[expect(clippy::cast_precision_loss, reason = "a frame count far below 2^52")]
			let place = k as f64 * step;
			#[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "a frame index, floored")]
			let before = place.floor() as usize;
			if before > last {
				break;
			}
			let after = (before + 1).min(last);
			#[expect(clippy::cast_possible_truncation, reason = "a share between two frames")]
			let share = (place - place.floor()) as f32;
			for (c, value) in slot.iter_mut().enumerate() {
				let (a, b) = (sound.frames[before][c], sound.frames[after][c]);
				*value += (a + (b - a) * share) * gain;
			}
		}
	}
	#[expect(clippy::cast_possible_truncation, reason = "a sample clipped to 16 bits")]
	Ok(out.into_iter().flatten().map(|s| (s.clamp(-1.0, 1.0) * f32::from(i16::MAX)).round() as i16).collect())
}
