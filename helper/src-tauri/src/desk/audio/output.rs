//! The outputs and the open output of the desktop app's sound (`desk::audio`), over rodio and cpal: the outputs the
//! system lists (each by its id, `<host>:<id>`, and its name), an output opened by its id while it is connected (else
//! the system's own), what stops it while it plays, and a sound or a line played on it.

use std::io::{Read, Seek};
use std::time::Duration;

use rodio::cpal::traits::HostTrait as _;
use rodio::{
	CpalError, CpalErrorKind, DeviceSinkBuilder, DeviceSinkError, DeviceTrait as _, MixerDeviceSink, Player,
	Source as _,
};
use serde::Serialize;

/// An output as the page lists it.
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub(crate) struct Listed {
	pub(crate) id: String,
	pub(crate) name: String,
	/// The system's own output now.
	pub(crate) default: bool,
}

/// The outputs the system has now.
///
/// # Errors
/// The system does not list its outputs.
pub(crate) fn list() -> Result<Vec<Listed>, CpalError> {
	let host = rodio::cpal::default_host();
	let default = host.default_output_device().and_then(|d| d.id().ok());
	Ok(host
		.output_devices()?
		.filter_map(|device| {
			let id = device.id().ok()?;
			let name = device.description().ok()?.name().to_owned();
			Some(Listed { default: default.as_ref() == Some(&id), id: id.to_string(), name })
		})
		.collect())
}

/// An open output: the one picked (`on_picked`), or the system's own.
pub(crate) struct Opened {
	pub(crate) sink: MixerDeviceSink,
	pub(crate) on_picked: bool,
}

/// Opens the output `picked` (an id `list` gave) while it is connected, else the system's own; `lost` hears what
/// stops it while it plays (unplugged, taken by the system, its format changed: `stops`), and what does not stop it is
/// a debug line.
///
/// # Errors
/// No output, or the one found does not open in any configuration it offers.
pub(crate) fn open<E>(picked: Option<&str>, mut lost: E) -> Result<Opened, DeviceSinkError>
where
	E: FnMut(CpalError) + Send + Clone + 'static,
{
	let host = rodio::cpal::default_host();
	let device = picked.and_then(|id| id.parse().ok()).and_then(|id: rodio::cpal::DeviceId| host.device_by_id(&id));
	let on_picked = device.is_some();
	let device = device.or_else(|| host.default_output_device()).ok_or(DeviceSinkError::NoDevice)?;
	let heard = move |e: CpalError| {
		if stops(&e) {
			lost(e);
		} else {
			log::debug!("sound: the output plays on: {e}");
		}
	};
	let mut sink = DeviceSinkBuilder::from_device(device)?.with_error_callback(heard).open_sink_or_fallback()?;
	sink.log_on_drop(false);
	Ok(Opened { sink, on_picked })
}

/// Whether an error the open output reports stops it. Since cpal 0.18 its error callback also hears what leaves the
/// output playing: a glitch (`Xrun`, on macOS at each of the output's processor overloads), the system's own output
/// moved under one that follows it (`DeviceChanged`) and a refused real-time priority (`RealtimeDenied`); the rest
/// (unplugged, its format changed, an error cpal cannot name) stops it, and the next sound opens it anew.
fn stops(e: &CpalError) -> bool {
	!matches!(e.kind(), CpalErrorKind::Xrun | CpalErrorKind::DeviceChanged | CpalErrorKind::RealtimeDenied)
}

/// Plays `data` (Ogg Vorbis) on the output after `delay`, on a player of its own at `volume` (an amplitude, 1 as
/// recorded), which holds from its first sample.
///
/// # Errors
/// The data does not decode.
pub(crate) fn play<R>(
	sink: &MixerDeviceSink,
	data: R,
	delay: Duration,
	volume: f32,
) -> Result<Player, rodio::decoder::DecoderError>
where
	R: Read + Seek + Send + Sync + 'static,
{
	let decoded = rodio::Decoder::new(data)?;
	let player = Player::connect_new(sink.mixer());
	player.set_volume(volume);
	player.append(decoded.delay(delay));
	Ok(player)
}
