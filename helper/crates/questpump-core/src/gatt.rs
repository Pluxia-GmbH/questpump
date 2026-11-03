//! The Bluetooth characteristics the helper reads (docs/plan.md slices 8 and 25; rust-helper `bluetooth.md`, which
//! quotes the Bluetooth SIG's GSS tables and the Fitness Machine Service 1.0 specification). From a strap that
//! broadcasts the standard heart-rate service (0x180D: the WHOOP's broadcast, a Polar H10), Heart Rate Measurement
//! (0x2A37). From a walking pad that speaks the Fitness Machine Service (0x1826), Treadmill Data (0x2ACD), Fitness
//! Machine Status (0x2ADA), Fitness Machine Feature (0x2ACC) and Supported Speed Range (0x2AD4), and the Fitness
//! Machine Control Point's (0x2AD9) requests and answers. Parsed by their flag bits and op codes, never by fixed
//! offsets; no btleplug here.

/// One Heart Rate Measurement notification.
#[derive(Debug, Clone, PartialEq)]
pub struct HeartMeasurement {
	pub bpm: u16,
	/// Whether the strap feels skin; none when it cannot tell.
	pub contact: Option<bool>,
	/// The beat-to-beat intervals it carried, oldest first, in seconds.
	pub rr: Vec<f64>,
}

impl HeartMeasurement {
	/// The heart rate the verdict may take: none without contact or outside `verdict::BPM_RANGE`.
	#[must_use]
	pub fn reading(&self) -> Option<u16> {
		(self.contact != Some(false) && crate::verdict::BPM_RANGE.contains(&self.bpm)).then_some(self.bpm)
	}
}

/// Reads a 0x2A37 value. Byte 0 is the flags: bit 0 a uint16 heart rate (else uint8), bits 1-2 sensor contact
/// (bit 2 says the strap can tell, bit 1 that it feels skin), bit 3 a uint16 energy expended that follows, bit 4 the
/// RR intervals (uint16, 1/1024 s) that fill the rest; every field little-endian. None for a value too short to hold
/// its heart rate; a trailing half interval is left out.
#[must_use]
pub fn parse_hrm(b: &[u8]) -> Option<HeartMeasurement> {
	let flags = *b.first()?;
	let (bpm, mut at) =
		if flags & 0x01 == 0 { (u16::from(*b.get(1)?), 2) } else { (u16::from_le_bytes([*b.get(1)?, *b.get(2)?]), 3) };
	let contact = (flags & 0x04 != 0).then_some(flags & 0x02 != 0);
	if flags & 0x08 != 0 {
		at += 2;
	}
	let rr = if flags & 0x10 != 0 {
		b.get(at..)
			.unwrap_or_default()
			.as_chunks::<2>()
			.0
			.iter()
			.map(|&p| f64::from(u16::from_le_bytes(p)) / 1024.0)
			.collect()
	} else {
		Vec::new()
	};
	Some(HeartMeasurement { bpm, contact, rr })
}

/// One Treadmill Data record (0x2ACD), merged from its parts: the fields the helper reads, each none while the pad has
/// not sent it.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Treadmill {
	/// Instantaneous Speed, in km/h.
	pub kmh: Option<f64>,
	/// Total Distance: the metres the pad counted since its own session began.
	pub distance_m: Option<u32>,
	/// Elapsed Time: the seconds since the pad's own session began.
	pub elapsed_s: Option<u16>,
}

impl Treadmill {
	/// A later part of the same record: a field it carries wins, a field it lacks keeps the earlier value.
	pub fn merge(&mut self, part: Self) {
		self.kmh = part.kmh.or(self.kmh);
		self.distance_m = part.distance_m.or(self.distance_m);
		self.elapsed_s = part.elapsed_s.or(self.elapsed_s);
	}

	/// Whether `part` carries a field this record already holds: the sign that it begins the next record, for a pad
	/// that never clears More Data.
	#[must_use]
	pub fn overlaps(&self, part: &Self) -> bool {
		(self.kmh.is_some() && part.kmh.is_some())
			|| (self.distance_m.is_some() && part.distance_m.is_some())
			|| (self.elapsed_s.is_some() && part.elapsed_s.is_some())
	}
}

/// One 0x2ACD notification: its fields, and whether the record goes on in the next notification (bit 0, More Data).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TreadmillPart {
	pub data: Treadmill,
	pub more: bool,
}

/// Bit 0, More Data: the record goes on in the next notification, and this one carries no Instantaneous Speed.
const MORE_DATA: u16 = 0x0001;
const TOTAL_DISTANCE: u16 = 0x0004;
const ELAPSED_TIME: u16 = 0x0400;

/// Treadmill Data's fields in the order of the GSS table (`treadmill_data.yaml`), each with the flag bit that
/// announces it and its size in bytes; Instantaneous Speed comes when More Data is CLEAR, every other field when its
/// bit is set. Bits 13-15 are reserved.
const TREADMILL_FIELDS: [(u16, usize); 13] = [
	(MORE_DATA, 2),      // Instantaneous Speed, uint16, 0.01 km/h
	(0x0002, 2),         // Average Speed, uint16, 0.01 km/h
	(TOTAL_DISTANCE, 3), // Total Distance, uint24, metres
	(0x0008, 4),         // Inclination and Ramp Angle Setting, sint16 each
	(0x0010, 4),         // Positive and Negative Elevation Gain, uint16 each
	(0x0020, 2),         // Instantaneous Pace, uint16, seconds
	(0x0040, 2),         // Average Pace, uint16, seconds
	(0x0080, 5),         // Total Energy uint16, Energy per Hour uint16, Energy per Minute uint8
	(0x0100, 1),         // Heart Rate, uint8
	(0x0200, 1),         // Metabolic Equivalent, uint8
	(ELAPSED_TIME, 2),   // Elapsed Time, uint16, seconds
	(0x0800, 2),         // Remaining Time, uint16, seconds
	(0x1000, 4),         // Force on Belt and Power Output, sint16 each
];

/// Reads a 0x2ACD value. Bytes 0-1 are the flags (uint16), then each field `TREADMILL_FIELDS` lists, in its order, when
/// the flags announce it; every field little-endian. The speed, the distance and the elapsed time are kept, the rest
/// stepped over by their sizes. None for a value too short for its flags or its first field; a field cut short ends
/// the parse with the fields before it. A record may span notifications: `TreadmillRecords` merges them.
#[must_use]
pub fn parse_treadmill(b: &[u8]) -> Option<TreadmillPart> {
	let flags = u16::from_le_bytes([*b.first()?, *b.get(1)?]);
	let more = flags & MORE_DATA != 0;
	let mut data = Treadmill::default();
	let mut at = 2;
	for (bit, len) in TREADMILL_FIELDS {
		let present = if bit == MORE_DATA { !more } else { flags & bit != 0 };
		if !present {
			continue;
		}
		let Some(field) = b.get(at..at + len) else {
			return (at > 2).then_some(TreadmillPart { data, more });
		};
		at += len;
		match bit {
			MORE_DATA => data.kmh = Some(kmh(field)),
			TOTAL_DISTANCE => data.distance_m = Some(le(field)),
			ELAPSED_TIME => data.elapsed_s = u16::try_from(le(field)).ok(),
			_ => {}
		}
	}
	Some(TreadmillPart { data, more })
}

/// Assembles Treadmill Data parts into records (docs/plan.md slice 25): parts merge by field until one arrives with
/// More Data clear, which completes the record; for a pad that never clears it, a part that carries a field the pending
/// record already holds sends the pending record out first and begins the next one. A record never goes out empty: a
/// part with More Data clear carries its speed, and the pending record goes out early only when it holds a field.
#[derive(Debug, Default)]
pub struct TreadmillRecords {
	/// The parts merged since the last record went out.
	pending: Treadmill,
}

impl TreadmillRecords {
	/// The records this part completes, oldest first (0, 1 or 2).
	#[must_use]
	pub fn push(&mut self, part: TreadmillPart) -> Vec<Treadmill> {
		let mut done = Vec::new();
		if self.pending.overlaps(&part.data) {
			done.push(std::mem::take(&mut self.pending));
		}
		self.pending.merge(part.data);
		if !part.more {
			done.push(std::mem::take(&mut self.pending));
		}
		done
	}
}

/// The Fitness Machine Status (0x2ADA) events the helper reads.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MachineStatus {
	/// 0x01: the pad was reset, which ends a client's control.
	Reset,
	/// 0x02 with its stop parameter: the user stopped the belt.
	Stopped,
	/// 0x02 with its pause parameter: the user paused the belt.
	Paused,
	/// 0x03: the safety key stopped the belt.
	SafetyKey,
	/// 0x04: the user started or resumed the belt.
	Started,
	/// 0x05: the target speed changed, to this many km/h.
	TargetSpeed(f64),
	/// 0xFF: the client lost control of the pad.
	ControlLost,
	/// An op code the helper does not read (a changed incline, among others).
	Other(u8),
}

/// Reads a 0x2ADA value (FTMS 1.0, as `bluetooth.md` quotes it). Byte 0 is the op code, its parameter follows: 0x01
/// reset, 0x02 stopped or paused by the user (uint8: 2 a pause; 1, another value or none a stop, since the belt halted
/// either way), 0x03 stopped by the safety key, 0x04 started or resumed by the user, 0x05 target speed changed
/// (uint16, 0.01 km/h), 0xFF control permission lost. None for an empty value or a target speed cut short.
#[must_use]
pub fn parse_status(b: &[u8]) -> Option<MachineStatus> {
	let (&op, param) = b.split_first()?;
	Some(match op {
		0x01 => MachineStatus::Reset,
		0x02 if param.first() == Some(&0x02) => MachineStatus::Paused,
		0x02 => MachineStatus::Stopped,
		0x03 => MachineStatus::SafetyKey,
		0x04 => MachineStatus::Started,
		0x05 => MachineStatus::TargetSpeed(kmh(param.get(..2)?)),
		0xFF => MachineStatus::ControlLost,
		other => MachineStatus::Other(other),
	})
}

/// Reads a Fitness Machine Feature value (0x2ACC): Fitness Machine Features (uint32), then Target Setting Features
/// (uint32), whose bit 0 says the pad takes a target speed. None for a value under 8 bytes.
#[must_use]
pub fn speed_target(feature: &[u8]) -> Option<bool> {
	Some(le(feature.get(4..8)?) & 0x01 != 0)
}

/// Supported Speed Range (0x2AD4): the target speeds the pad takes, in km/h.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpeedRange {
	pub min_kmh: f64,
	pub max_kmh: f64,
	/// Minimum Increment: the step between two speeds the pad takes, counted from `min_kmh`.
	pub step_kmh: f64,
}

/// Reads a 0x2AD4 value: Minimum Speed, Maximum Speed and Minimum Increment, uint16 each in 0.01 km/h (GSS
/// `supported_speed_range.yaml`). None for a value under 6 bytes, a maximum below the minimum or a maximum of 0 (a range
/// the pad left unset, which would turn every speed into 0): the speed asked for then goes to the pad as it is.
#[must_use]
pub fn parse_speed_range(b: &[u8]) -> Option<SpeedRange> {
	let at = |at: usize| Some(kmh(b.get(at..at + 2)?));
	let range = SpeedRange { min_kmh: at(0)?, max_kmh: at(2)?, step_kmh: at(4)? };
	(range.max_kmh > 0.0 && range.max_kmh >= range.min_kmh).then_some(range)
}

impl SpeedRange {
	/// The target speed the pad takes nearest `kmh`: clamped into min..max and rounded to the step from min, never past
	/// max when the step does not divide the range (a zero step only clamps; NaN reads as min). Worked in 0.01 km/h,
	/// the pad's own unit, so `cp::set_target_speed` writes the result exactly.
	#[must_use]
	pub fn fit(&self, kmh: f64) -> f64 {
		let (min, step) = (hundredths(self.min_kmh), hundredths(self.step_kmh));
		let max = hundredths(self.max_kmh).max(min);
		let at = hundredths(kmh).max(min).min(max);
		let fitted =
			if step > 0.0 { min + ((at - min) / step).round().min(((max - min) / step).floor()) * step } else { at };
		fitted / 100.0
	}
}

/// A speed in whole 0.01 km/h, the pad's own unit.
fn hundredths(kmh: f64) -> f64 {
	(kmh * 100.0).round()
}

/// A uint16 speed field in 0.01 km/h, in km/h.
fn kmh(field: &[u8]) -> f64 {
	f64::from(le(field)) / 100.0
}

/// A little-endian unsigned field of up to four bytes.
fn le(field: &[u8]) -> u32 {
	field.iter().rev().fold(0, |n, &byte| (n << 8) | u32::from(byte))
}

/// The Fitness Machine Control Point (0x2AD9; FTMS 1.0, as `bluetooth.md` quotes it). A client writes one request at a
/// time, its op code and then its parameter, with a response, and the pad answers each with a Response Code indication
/// on the same characteristic. Every request but Request Control needs control, which lasts until the link drops, a
/// reset or a Control Permission Lost status.
pub mod cp {
	/// Asks the pad for control.
	pub const REQUEST_CONTROL: u8 = 0x00;
	/// Sets the belt's target speed: uint16, 0.01 km/h.
	pub const SET_TARGET_SPEED: u8 = 0x02;
	/// Starts or resumes the belt.
	pub const START_RESUME: u8 = 0x07;
	/// Stops or pauses the belt: uint8, 1 a stop, 2 a pause.
	pub const STOP_PAUSE: u8 = 0x08;
	/// The pad's answer to a request.
	pub const RESPONSE: u8 = 0x80;

	/// The Request Control request.
	#[must_use]
	pub fn request_control() -> [u8; 1] {
		[REQUEST_CONTROL]
	}

	/// The Set Target Speed request: `kmh` in 0.01 km/h, rounded; a speed outside 0..=655.35 km/h is clamped (NaN reads
	/// as 0).
	#[must_use]
	pub fn set_target_speed(kmh: f64) -> [u8; 3] {
		#[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "in 0..=65535 after the clamp")]
		let speed = super::hundredths(kmh).clamp(0.0, 65535.0) as u16;
		let [low, high] = speed.to_le_bytes();
		[SET_TARGET_SPEED, low, high]
	}

	/// The Start or Resume request.
	#[must_use]
	pub fn start() -> [u8; 1] {
		[START_RESUME]
	}

	/// The Stop or Pause request that stops the belt.
	#[must_use]
	pub fn stop() -> [u8; 2] {
		[STOP_PAUSE, 0x01]
	}

	/// A Response Code's result.
	#[derive(Debug, Clone, Copy, PartialEq, Eq)]
	pub enum Outcome {
		/// 0x01.
		Success,
		/// 0x02: the pad does not know the request's op code.
		NotSupported,
		/// 0x03.
		InvalidParameter,
		/// 0x04: the pad could not do it.
		Failed,
		/// 0x05: the client does not hold control.
		NotPermitted,
		/// A result the specification does not name.
		Other(u8),
	}

	impl Outcome {
		/// The `pad_event` `detail` word: `success`, `not_supported`, `invalid_parameter`, `failed`, `not_permitted`,
		/// `code_<n>`.
		#[must_use]
		pub fn word(self) -> String {
			match self {
				Self::Success => "success".into(),
				Self::NotSupported => "not_supported".into(),
				Self::InvalidParameter => "invalid_parameter".into(),
				Self::Failed => "failed".into(),
				Self::NotPermitted => "not_permitted".into(),
				Self::Other(code) => format!("code_{code}"),
			}
		}
	}

	/// Reads a 0x2AD9 indication: a Response Code (0x80, the request's op code, the result: 1 success, 2 op code not
	/// supported, 3 invalid parameter, 4 operation failed, 5 control not permitted) as the request's op code and its
	/// outcome; None for anything else.
	#[must_use]
	pub fn parse_response(b: &[u8]) -> Option<(u8, Outcome)> {
		let &[RESPONSE, op, result, ..] = b else {
			return None;
		};
		let outcome = match result {
			0x01 => Outcome::Success,
			0x02 => Outcome::NotSupported,
			0x03 => Outcome::InvalidParameter,
			0x04 => Outcome::Failed,
			0x05 => Outcome::NotPermitted,
			other => Outcome::Other(other),
		};
		Some((op, outcome))
	}
}
