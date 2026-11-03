//! The reel on macOS (rust-helper `desktop-reel.md`): the films read by `AVAssetReader` as 32-bit BGRA frames, the reel
//! written by `AVAssetWriter` (H.264 High 4.0 from BGRA pixel buffers of its adaptor's pool, BT.709, a key frame at
//! most every `key_every` frames and no frame reordering, the average bit rate; AAC-LC from 16-bit PCM sample buffers,
//! or no audio input at all; an MPEG-4 file with no metadata, its moov put first by `mp4::faststart`), through
//! `objc2-av-foundation` 0.3.2 on the Objective-C API (Apple's macOS 27 SDK deprecates `startWriting`, `addInput` and
//! the pixel-buffer adaptor for Swift's receivers, which have no Objective-C form, and 0.3.2 marks none of them
//! deprecated). A writer whose MPEG-4 file type does not take the H.264 settings is `Unsupported`. Each call that may
//! raise an Objective-C exception runs inside `objc2::exception::catch`, so a refused input or a broken file is an
//! error, never an abort. A file on the clipboard is its `NSURL` written to the general pasteboard after
//! `clearContents`, the system's file copy, which a chat's paste takes.

use std::collections::VecDeque;
use std::ffi::c_void;
use std::panic::AssertUnwindSafe;
use std::path::Path;
use std::ptr::{NonNull, null, null_mut};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use anyhow::{Context as _, anyhow, bail};
use block2::RcBlock;
use objc2::rc::{Retained, autoreleasepool};
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2_app_kit::NSPasteboard;
use objc2_av_foundation::{
	AVAssetReader, AVAssetReaderStatus, AVAssetReaderTrackOutput, AVAssetWriter, AVAssetWriterInput,
	AVAssetWriterInputPixelBufferAdaptor, AVAssetWriterStatus, AVFileTypeMPEG4, AVMediaTypeAudio, AVMediaTypeVideo,
	AVURLAsset, AVVideoAllowFrameReorderingKey, AVVideoAverageBitRateKey, AVVideoCodecKey, AVVideoCodecTypeH264,
	AVVideoColorPrimaries_ITU_R_709_2, AVVideoColorPrimariesKey, AVVideoColorPropertiesKey,
	AVVideoCompressionPropertiesKey, AVVideoExpectedSourceFrameRateKey, AVVideoH264EntropyModeCABAC,
	AVVideoH264EntropyModeKey, AVVideoHeightKey, AVVideoMaxKeyFrameIntervalKey, AVVideoProfileLevelH264High40,
	AVVideoProfileLevelKey, AVVideoTransferFunction_ITU_R_709_2, AVVideoTransferFunctionKey, AVVideoWidthKey,
	AVVideoYCbCrMatrix_ITU_R_709_2, AVVideoYCbCrMatrixKey,
};
use objc2_avf_audio::{AVEncoderBitRateKey, AVFormatIDKey, AVNumberOfChannelsKey, AVSampleRateKey};
use objc2_core_audio_types::{
	AudioStreamBasicDescription, kAudioFormatFlagIsPacked, kAudioFormatFlagIsSignedInteger, kAudioFormatLinearPCM,
	kAudioFormatMPEG4AAC,
};
use objc2_core_foundation::{CFRetained, CFString};
use objc2_core_media::{
	CMAudioFormatDescriptionCreate, CMAudioSampleBufferCreateReadyWithPacketDescriptions, CMBlockBuffer,
	CMFormatDescription, CMSampleBuffer, CMTime, kCMBlockBufferAssureMemoryNowFlag,
};
use objc2_core_video::{
	CVPixelBuffer, CVPixelBufferGetBaseAddress, CVPixelBufferGetBytesPerRow, CVPixelBufferGetHeight,
	CVPixelBufferGetWidth, CVPixelBufferLockBaseAddress, CVPixelBufferLockFlags, CVPixelBufferPool,
	CVPixelBufferUnlockBaseAddress, kCVPixelBufferHeightKey, kCVPixelBufferPixelFormatTypeKey, kCVPixelBufferWidthKey,
	kCVPixelFormatType_32BGRA, kCVReturnSuccess,
};
use objc2_foundation::{NSArray, NSDictionary, NSError, NSNumber, NSString, NSURL};

use super::compose::Canvas;
use super::{Encoding, Frame, Unsupported};

/// The longest an input may stay not ready for more media, and finishing the file may take.
const PATIENCE: Duration = Duration::from_secs(30);

/// A weak-linked `AVFoundation` constant, or why it is missing.
fn key<T: ?Sized>(value: Option<&'static T>, name: &str) -> anyhow::Result<&'static T> {
	value.ok_or_else(|| anyhow!("AVFoundation lacks {name}"))
}

/// A Core Video key as the Foundation string it is toll-free bridged to.
fn bridged(key: &CFString) -> &NSString {
	// SAFETY: CFString is toll-free bridged to NSString: the same object read through its Objective-C class.
	unsafe { &*std::ptr::from_ref(key).cast::<NSString>() }
}

/// `AVFoundation`'s settings: string keys to objects.
type Settings = NSDictionary<NSString, AnyObject>;

/// A dictionary of settings.
fn settings(pairs: &[(&NSString, &AnyObject)]) -> Retained<Settings> {
	let keys: Vec<&NSString> = pairs.iter().map(|(k, _)| *k).collect();
	let values: Vec<&AnyObject> = pairs.iter().map(|(_, v)| *v).collect();
	NSDictionary::from_slices(&keys, &values)
}

/// An `NSError`'s words, or `what` without one.
fn why(error: Option<Retained<NSError>>, what: &str) -> anyhow::Error {
	match error {
		Some(e) => anyhow!("{what}: {}", e.localizedDescription()),
		None => anyhow!("{what}"),
	}
}

/// A file URL.
fn url(path: &Path) -> anyhow::Result<Retained<NSURL>> {
	let text = path.to_str().with_context(|| format!("{} is not UTF-8", path.display()))?;
	Ok(NSURL::fileURLWithPath(&NSString::from_str(text)))
}

/// Reads a film's frames in order, handing each over (its time, its BGRA rows, its size), inside an autorelease pool of
/// its own (the reel's job runs on a thread without one).
///
/// # Errors
/// The film does not open or read, or holds no video.
pub(super) fn read_film(path: &Path, each: &mut dyn FnMut(Frame<'_>)) -> anyhow::Result<()> {
	autoreleasepool(|_| read_frames(path, each))
}

fn read_frames(path: &Path, each: &mut dyn FnMut(Frame<'_>)) -> anyhow::Result<()> {
	let video = key(
		// SAFETY: a static AVFoundation exports, read once.
		unsafe { AVMediaTypeVideo },
		"AVMediaTypeVideo",
	)?;
	let url = url(path)?;
	// SAFETY: a file URL and no options; the asset reads its tracks from the file as they are asked for.
	let asset = unsafe { AVURLAsset::URLAssetWithURL_options(&url, None) };
	// SAFETY: the asset's tracks and each one's media type, read from the open asset.
	let track = unsafe { asset.tracks() }
		.iter()
		// SAFETY: a track of this asset.
		.find(|t| unsafe { t.mediaType() }.isEqualToString(video))
		.with_context(|| format!("{} holds no video", path.display()))?;
	// SAFETY: a reader of the asset just opened.
	let reader =
		unsafe { AVAssetReader::assetReaderWithAsset_error(&asset) }.map_err(|e| why(Some(e), "the film's reader"))?;
	let format = NSNumber::new_u32(kCVPixelFormatType_32BGRA);
	// SAFETY: a static Core Video exports.
	let format_key = bridged(unsafe { kCVPixelBufferPixelFormatTypeKey });
	let options = settings(&[(format_key, format.as_ref())]);
	// SAFETY: the video track of the asset the reader reads, decoded to 32-bit BGRA.
	let output =
		unsafe { AVAssetReaderTrackOutput::assetReaderTrackOutputWithTrack_outputSettings(&track, Some(&options)) };
	let started = objc2::exception::catch(AssertUnwindSafe(|| {
		// SAFETY: the output's track belongs to the reader's asset, and the reader has not started.
		unsafe {
			output.setAlwaysCopiesSampleData(false);
			reader.canAddOutput(&output) && {
				reader.addOutput(&output);
				reader.startReading()
			}
		}
	}));
	if !matches!(started, Ok(true)) {
		// SAFETY: the reader's error, read after it refused to start.
		return Err(why(unsafe { reader.error() }, "the film's reader did not start"));
	}
	loop {
		let more = autoreleasepool(|_| -> anyhow::Result<bool> {
			// SAFETY: the reader has started; a None is the end or a failure, which its status tells.
			let Some(sample) = (unsafe { output.copyNextSampleBuffer() }) else { return Ok(false) };
			// SAFETY: a sample buffer the output handed over.
			let (time, image) = unsafe { (sample.presentation_time_stamp(), sample.image_buffer()) };
			let Some(image) = image else { return Ok(true) };
			#[expect(clippy::cast_precision_loss, reason = "a film's time in its timescale")]
			let seconds = if time.timescale > 0 { time.value as f64 / f64::from(time.timescale) } else { 0.0 };
			// SAFETY: the decoded frame is locked for reading while its rows are read, then unlocked.
			unsafe {
				if CVPixelBufferLockBaseAddress(&image, CVPixelBufferLockFlags::ReadOnly) != kCVReturnSuccess {
					bail!("a film's frame did not lock");
				}
				let (width, height) = (CVPixelBufferGetWidth(&image), CVPixelBufferGetHeight(&image));
				let stride = CVPixelBufferGetBytesPerRow(&image);
				let base = CVPixelBufferGetBaseAddress(&image).cast::<u8>();
				if !base.is_null() {
					let px = std::slice::from_raw_parts(base, stride * height);
					each(Frame { at: seconds, px, width, height, stride });
				}
				CVPixelBufferUnlockBaseAddress(&image, CVPixelBufferLockFlags::ReadOnly);
			}
			Ok(true)
		})?;
		if !more {
			break;
		}
	}
	// SAFETY: the reader's status and error once its output ran dry.
	unsafe {
		if reader.status() == AVAssetReaderStatus::Failed {
			return Err(why(reader.error(), "the film did not read"));
		}
	}
	Ok(())
}

/// The video input's settings (H.264 High 4.0 at the frame's size, its bit rate, a key frame at most every
/// `key_every` frames, no frame reordering, CABAC, BT.709) and its adaptor's (BGRA buffers of the frame's size).
fn video_settings(e: &Encoding) -> anyhow::Result<(Retained<Settings>, Retained<Settings>)> {
	// SAFETY: statics AVFoundation exports, each read once.
	let k = unsafe {
		[
			key(AVVideoCodecKey, "AVVideoCodecKey")?,
			key(AVVideoWidthKey, "AVVideoWidthKey")?,
			key(AVVideoHeightKey, "AVVideoHeightKey")?,
			key(AVVideoCompressionPropertiesKey, "AVVideoCompressionPropertiesKey")?,
			key(AVVideoAverageBitRateKey, "AVVideoAverageBitRateKey")?,
			key(AVVideoMaxKeyFrameIntervalKey, "AVVideoMaxKeyFrameIntervalKey")?,
			key(AVVideoAllowFrameReorderingKey, "AVVideoAllowFrameReorderingKey")?,
			key(AVVideoProfileLevelKey, "AVVideoProfileLevelKey")?,
			key(AVVideoExpectedSourceFrameRateKey, "AVVideoExpectedSourceFrameRateKey")?,
			key(AVVideoH264EntropyModeKey, "AVVideoH264EntropyModeKey")?,
			key(AVVideoColorPropertiesKey, "AVVideoColorPropertiesKey")?,
			key(AVVideoColorPrimariesKey, "AVVideoColorPrimariesKey")?,
			key(AVVideoTransferFunctionKey, "AVVideoTransferFunctionKey")?,
			key(AVVideoYCbCrMatrixKey, "AVVideoYCbCrMatrixKey")?,
		]
	};
	let [
		codec,
		width,
		height,
		compression,
		bitrate,
		key_every,
		reorder,
		profile,
		rate,
		entropy,
		colour,
		primaries,
		transfer,
		matrix,
	] = k;
	// SAFETY: statics AVFoundation exports, each read once.
	let v = unsafe {
		[
			key(AVVideoCodecTypeH264, "AVVideoCodecTypeH264")?,
			key(AVVideoProfileLevelH264High40, "AVVideoProfileLevelH264High40")?,
			key(AVVideoH264EntropyModeCABAC, "AVVideoH264EntropyModeCABAC")?,
			key(AVVideoColorPrimaries_ITU_R_709_2, "AVVideoColorPrimaries_ITU_R_709_2")?,
			key(AVVideoTransferFunction_ITU_R_709_2, "AVVideoTransferFunction_ITU_R_709_2")?,
			key(AVVideoYCbCrMatrix_ITU_R_709_2, "AVVideoYCbCrMatrix_ITU_R_709_2")?,
		]
	};
	let [h264, high40, cabac, bt709_primaries, bt709_transfer, bt709_matrix] = v;
	let (wide, high) = (NSNumber::new_u32(e.width), NSNumber::new_u32(e.height));
	let (average, every, no, fps) = (
		NSNumber::new_u32(e.bitrate),
		NSNumber::new_u32(e.key_every),
		NSNumber::new_bool(false),
		NSNumber::new_u32(e.fps),
	);
	let properties = settings(&[
		(bitrate, average.as_ref()),
		(key_every, every.as_ref()),
		(reorder, no.as_ref()),
		(profile, high40.as_ref()),
		(rate, fps.as_ref()),
		(entropy, cabac.as_ref()),
	]);
	let colours = settings(&[
		(primaries, bt709_primaries.as_ref()),
		(transfer, bt709_transfer.as_ref()),
		(matrix, bt709_matrix.as_ref()),
	]);
	let output = settings(&[
		(codec, h264.as_ref()),
		(width, wide.as_ref()),
		(height, high.as_ref()),
		(compression, properties.as_ref()),
		(colour, colours.as_ref()),
	]);
	let format = NSNumber::new_u32(kCVPixelFormatType_32BGRA);
	// SAFETY: statics Core Video exports.
	let (format_key, width_key, height_key) = unsafe {
		(bridged(kCVPixelBufferPixelFormatTypeKey), bridged(kCVPixelBufferWidthKey), bridged(kCVPixelBufferHeightKey))
	};
	let source = settings(&[(format_key, format.as_ref()), (width_key, wide.as_ref()), (height_key, high.as_ref())]);
	Ok((output, source))
}

/// A reel being written: the writer, its video input with the pixel-buffer adaptor, and its audio input with the PCM's
/// format where the reel has sound.
pub(super) struct Writer {
	asset: Retained<AVAssetWriter>,
	video: Retained<AVAssetWriterInput>,
	adaptor: Retained<AVAssetWriterInputPixelBufferAdaptor>,
	audio: Option<(Retained<AVAssetWriterInput>, CFRetained<CMFormatDescription>)>,
	/// The sound handed over and not appended yet: each chunk and its first audio frame. `AVAssetWriter` interleaves its
	/// inputs, so the input running ahead is not ready until the other catches up: the sound goes in whenever its input
	/// takes it (as `requestMediaDataWhenReady` would hand it over), and a frame never waits on it.
	pending: VecDeque<(Vec<i16>, i64)>,
	/// The whole sound is appended and its input marked finished: the writer interleaves no more sound, so the frames
	/// after the sound's end (an encoder holds its last packets until then) never wait for it.
	sounded: bool,
	fps: i32,
	hz: i32,
}

// SAFETY: AVAssetWriter, its inputs and adaptor are used by one thread at a time (the reel's job owns the writer);
// Apple documents an asset writer as usable from any one thread, its inputs' appends serialised by the caller.
unsafe impl Send for Writer {}

impl Writer {
	/// A writer at `path` (no file may stand there) for `encoding`, started at time zero.
	///
	/// # Errors
	/// `AVFoundation` refused the settings or the file, or its file type does not take H.264 (`Unsupported`).
	pub(super) fn new(path: &Path, encoding: &Encoding) -> anyhow::Result<Self> {
		autoreleasepool(|_| Self::open(path, encoding))
	}

	fn open(path: &Path, e: &Encoding) -> anyhow::Result<Self> {
		let (output, source) = video_settings(e)?;
		// SAFETY: statics AVFoundation exports.
		let (mp4, video_type) =
			unsafe { (key(AVFileTypeMPEG4, "AVFileTypeMPEG4")?, key(AVMediaTypeVideo, "AVMediaTypeVideo")?) };
		let url = url(path)?;
		// SAFETY: a file URL where no file stands, and an MPEG-4 file type AVFoundation declares.
		let writer = unsafe { AVAssetWriter::assetWriterWithURL_fileType_error(&url, mp4) }
			.map_err(|e| why(Some(e), "the reel's writer"))?;
		// SAFETY: the writer has not started; this asks whether its file type takes the H.264 settings.
		if !unsafe { writer.canApplyOutputSettings_forMediaType(Some(&output), video_type) } {
			return Err(anyhow::Error::new(Unsupported));
		}
		// SAFETY: a video input of fully specified H.264 settings, and its adaptor of BGRA buffers of the frame's size.
		let (video, adaptor) = unsafe {
			let video = AVAssetWriterInput::assetWriterInputWithMediaType_outputSettings(video_type, Some(&output));
			video.setExpectsMediaDataInRealTime(false);
			let adaptor =
				AVAssetWriterInputPixelBufferAdaptor::assetWriterInputPixelBufferAdaptorWithAssetWriterInput_sourcePixelBufferAttributes(
					&video,
					Some(&source),
				);
			(video, adaptor)
		};
		let audio = match e.audio {
			Some((hz, kbps)) => Some(Self::audio_input(hz, kbps)?),
			None => None,
		};
		let started = objc2::exception::catch(AssertUnwindSafe(|| {
			// SAFETY: the writer has not started; its metadata is set before, and its inputs added, as it starts.
			unsafe {
				// mp4::faststart puts the moov first: asked here too, the writer leaves its media data's temporary file
				// (`<name>.sb-…`) beside the reel (seen 2026-10-17, macOS 27)
				writer.setShouldOptimizeForNetworkUse(false);
				writer.setMetadata(&NSArray::new());
				if !writer.canAddInput(&video) {
					return false;
				}
				writer.addInput(&video);
				if let Some((input, _)) = &audio {
					if !writer.canAddInput(input) {
						return false;
					}
					writer.addInput(input);
				}
				if !writer.startWriting() {
					return false;
				}
				writer.startSessionAtSourceTime(CMTime::new(0, 1));
				true
			}
		}));
		if !matches!(started, Ok(true)) {
			// SAFETY: the writer's error, read after it refused to start.
			return Err(why(unsafe { writer.error() }, "the reel's writer did not start"));
		}
		Ok(Self {
			asset: writer,
			video,
			adaptor,
			audio,
			pending: VecDeque::new(),
			sounded: false,
			fps: i32::try_from(e.fps)?,
			hz: i32::try_from(e.audio.map_or(1, |(hz, _)| hz))?,
		})
	}

	/// The AAC input at `hz` in stereo at `kbps`, and the format of the 16-bit interleaved PCM it takes.
	fn audio_input(
		hz: u32,
		kbps: u32,
	) -> anyhow::Result<(Retained<AVAssetWriterInput>, CFRetained<CMFormatDescription>)> {
		// SAFETY: statics AVFoundation and AVFAudio export.
		let (format_key, rate_key, channels_key, bitrate_key, audio_type) = unsafe {
			(
				key(AVFormatIDKey, "AVFormatIDKey")?,
				key(AVSampleRateKey, "AVSampleRateKey")?,
				key(AVNumberOfChannelsKey, "AVNumberOfChannelsKey")?,
				key(AVEncoderBitRateKey, "AVEncoderBitRateKey")?,
				key(AVMediaTypeAudio, "AVMediaTypeAudio")?,
			)
		};
		let (aac, rate, two, bitrate) = (
			NSNumber::new_u32(kAudioFormatMPEG4AAC),
			NSNumber::new_f64(f64::from(hz)),
			NSNumber::new_u32(2),
			NSNumber::new_u32(kbps * 1000),
		);
		let output = settings(&[
			(format_key, aac.as_ref()),
			(rate_key, rate.as_ref()),
			(channels_key, two.as_ref()),
			(bitrate_key, bitrate.as_ref()),
		]);
		// SAFETY: an audio input of fully specified AAC settings (its format, rate and channels).
		let input =
			unsafe { AVAssetWriterInput::assetWriterInputWithMediaType_outputSettings(audio_type, Some(&output)) };
		// SAFETY: the input is not added yet.
		unsafe { input.setExpectsMediaDataInRealTime(false) };
		let pcm = AudioStreamBasicDescription {
			mSampleRate: f64::from(hz),
			mFormatID: kAudioFormatLinearPCM,
			mFormatFlags: kAudioFormatFlagIsSignedInteger | kAudioFormatFlagIsPacked,
			mBytesPerPacket: 4,
			mFramesPerPacket: 1,
			mBytesPerFrame: 4,
			mChannelsPerFrame: 2,
			mBitsPerChannel: 16,
			mReserved: 0,
		};
		let mut described: *const CMFormatDescription = null();
		// SAFETY: a valid stream description, no channel layout, cookie or extensions, and an out pointer to fill.
		let status = unsafe {
			CMAudioFormatDescriptionCreate(
				None,
				NonNull::from(&pcm),
				0,
				null(),
				0,
				null(),
				None,
				NonNull::from(&mut described),
			)
		};
		let described = NonNull::new(described.cast_mut()).filter(|_| status == 0);
		let described = described.with_context(|| format!("the PCM's format description: {status}"))?;
		// SAFETY: a format description CMAudioFormatDescriptionCreate made, whose one reference is ours.
		let described = unsafe { CFRetained::from_raw(described) };
		Ok((input, described))
	}

	/// Fails where the writer stopped writing, or waited past `PATIENCE` since `since`.
	fn writing(&self, since: Instant) -> anyhow::Result<()> {
		// SAFETY: the writer's status, read while it writes.
		if unsafe { self.asset.status() } != AVAssetWriterStatus::Writing {
			// SAFETY: the writer's error once it stopped writing.
			return Err(why(unsafe { self.asset.error() }, "the reel's writer stopped"));
		}
		if since.elapsed() > PATIENCE {
			bail!("the reel's writer took no media for {} s", PATIENCE.as_secs());
		}
		Ok(())
	}

	/// Appends the sound waiting while its input takes it; with `wait`, all of it, waiting for the input. Once the whole
	/// sound is in, its input is marked finished.
	fn feed(&mut self, wait: bool) -> anyhow::Result<()> {
		let Some((input, _)) = &self.audio else { return Ok(()) };
		let input = input.clone();
		let since = Instant::now();
		while let Some((pcm, first)) = self.pending.front() {
			// SAFETY: the input's readiness, read while the writer writes.
			if unsafe { input.isReadyForMoreMediaData() } {
				self.append_audio(pcm, *first)?;
				self.pending.pop_front();
			} else if wait {
				self.writing(since)?;
				std::thread::sleep(Duration::from_millis(2));
			} else {
				break;
			}
		}
		if self.pending.is_empty() && !self.sounded {
			self.sounded = true;
			// SAFETY: every chunk of the sound has been appended; no append to this input follows.
			unsafe { input.markAsFinished() };
		}
		Ok(())
	}

	/// Appends the frame as frame `index` of the reel.
	///
	/// # Errors
	/// The writer stopped, or refused the frame.
	pub(super) fn video(&mut self, frame: &Canvas, index: i64) -> anyhow::Result<()> {
		let since = Instant::now();
		// SAFETY: the input's readiness, read while the writer writes; the sound goes in meanwhile as it is taken.
		while !unsafe { self.video.isReadyForMoreMediaData() } {
			self.feed(false)?;
			self.writing(since)?;
			std::thread::sleep(Duration::from_millis(2));
		}
		autoreleasepool(|_| {
			// SAFETY: the adaptor's pool, there once the writer started.
			let pool: Retained<CVPixelBufferPool> =
				unsafe { self.adaptor.pixelBufferPool() }.context("the reel's pixel buffers")?;
			let mut made: *mut CVPixelBuffer = null_mut();
			// SAFETY: a buffer from the adaptor's own pool into an out pointer.
			let status = unsafe { CVPixelBufferPool::create_pixel_buffer(None, &pool, NonNull::from(&mut made)) };
			let made = NonNull::new(made).filter(|_| status == kCVReturnSuccess);
			let made = made.with_context(|| format!("a reel's pixel buffer: {status}"))?;
			// SAFETY: a buffer the pool created, whose one reference is ours.
			let buffer = unsafe { CFRetained::from_raw(made) };
			// SAFETY: the buffer is locked while its rows are written, each within its width and stride, then unlocked.
			unsafe {
				if CVPixelBufferLockBaseAddress(&buffer, CVPixelBufferLockFlags::empty()) != kCVReturnSuccess {
					bail!("a reel's pixel buffer did not lock");
				}
				let stride = CVPixelBufferGetBytesPerRow(&buffer);
				let rows = CVPixelBufferGetHeight(&buffer).min(frame.height);
				let row = (CVPixelBufferGetWidth(&buffer).min(frame.width) * 4).min(stride);
				let base = CVPixelBufferGetBaseAddress(&buffer).cast::<u8>();
				if !base.is_null() {
					for y in 0..rows {
						let from = &frame.px[y * frame.width * 4..y * frame.width * 4 + row];
						std::ptr::copy_nonoverlapping(from.as_ptr(), base.add(y * stride), row);
					}
				}
				CVPixelBufferUnlockBaseAddress(&buffer, CVPixelBufferLockFlags::empty());
			}
			let appended = objc2::exception::catch(AssertUnwindSafe(|| {
				// SAFETY: the input is ready, the time numeric, the buffer from its pool and no longer written.
				unsafe { self.adaptor.appendPixelBuffer_withPresentationTime(&buffer, CMTime::new(index, self.fps)) }
			}));
			if !matches!(appended, Ok(true)) {
				// SAFETY: the writer's error once it refused a frame.
				return Err(why(unsafe { self.asset.error() }, "the reel's writer refused a frame"));
			}
			Ok(())
		})
	}

	/// Hands over the reel's whole sound, 16-bit stereo PCM (`pcm`, interleaved) from its start, appended in chunks of a
	/// tenth of a second as its input takes them; nothing without sound.
	///
	/// # Errors
	/// The writer stopped, or refused the sound.
	pub(super) fn audio(&mut self, pcm: &[i16]) -> anyhow::Result<()> {
		if self.audio.is_none() {
			return Ok(());
		}
		let chunk = usize::try_from(self.hz / 10).unwrap_or(4_800).max(1) * 2;
		for (k, part) in pcm.chunks(chunk).enumerate() {
			self.pending.push_back((part.to_vec(), i64::try_from(k * chunk / 2)?));
		}
		self.feed(false)
	}

	/// Appends PCM to the audio input, which takes it now.
	fn append_audio(&self, pcm: &[i16], first: i64) -> anyhow::Result<()> {
		let Some((input, described)) = &self.audio else { return Ok(()) };
		autoreleasepool(|_| {
			let bytes = std::mem::size_of_val(pcm);
			let mut block: *mut CMBlockBuffer = null_mut();
			// SAFETY: a block buffer Core Media allocates now for `bytes`, into an out pointer.
			let status = unsafe {
				CMBlockBuffer::create_with_memory_block(
					None,
					null_mut(),
					bytes,
					None,
					null(),
					0,
					bytes,
					kCMBlockBufferAssureMemoryNowFlag,
					NonNull::from(&mut block),
				)
			};
			let block =
				NonNull::new(block).filter(|_| status == 0).with_context(|| format!("a sound block: {status}"))?;
			// SAFETY: a block buffer Core Media made, whose one reference is ours.
			let block = unsafe { CFRetained::from_raw(block) };
			let source = NonNull::new(pcm.as_ptr().cast_mut().cast::<c_void>()).context("the sound's samples")?;
			// SAFETY: `bytes` bytes copied from the samples into the block of that length.
			let status = unsafe { CMBlockBuffer::replace_data_bytes(source, &block, 0, bytes) };
			if status != 0 {
				bail!("the sound's samples did not copy: {status}");
			}
			let mut sample: *mut CMSampleBuffer = null_mut();
			let frames = isize::try_from(pcm.len() / 2)?;
			// SAFETY: the block of PCM in the format described, its frames at their time, no packet descriptions
			// (LPCM needs none), into an out pointer.
			let status = unsafe {
				CMAudioSampleBufferCreateReadyWithPacketDescriptions(
					None,
					&block,
					described,
					frames,
					CMTime::new(first, self.hz),
					null(),
					NonNull::from(&mut sample),
				)
			};
			let sample =
				NonNull::new(sample).filter(|_| status == 0).with_context(|| format!("a sound sample: {status}"))?;
			// SAFETY: a sample buffer Core Media made, whose one reference is ours.
			let sample = unsafe { CFRetained::from_raw(sample) };
			let appended = objc2::exception::catch(AssertUnwindSafe(|| {
				// SAFETY: the input is ready and the sample buffer is whole.
				unsafe { input.appendSampleBuffer(&sample) }
			}));
			if !matches!(appended, Ok(true)) {
				// SAFETY: the writer's error once it refused the sound.
				return Err(why(unsafe { self.asset.error() }, "the reel's writer refused the sound"));
			}
			Ok(())
		})
	}

	/// Finishes the file and waits for it.
	///
	/// # Errors
	/// The writer did not complete the file.
	pub(super) fn finish(mut self) -> anyhow::Result<()> {
		self.feed(true)?;
		let (done, finished) = mpsc::channel();
		let handler = RcBlock::new(move || {
			let _ = done.send(());
		});
		let asked = objc2::exception::catch(AssertUnwindSafe(|| {
			// SAFETY: every append has returned; the inputs are marked finished before the writer is.
			unsafe {
				self.video.markAsFinished();
				self.asset.finishWritingWithCompletionHandler(&handler);
			}
		}));
		if asked.is_err() {
			bail!("the reel's writer did not finish");
		}
		finished.recv_timeout(PATIENCE).context("the reel's file took too long to finish")?;
		// SAFETY: the writer's status and error once its completion handler ran.
		unsafe {
			if self.asset.status() != AVAssetWriterStatus::Completed {
				return Err(why(self.asset.error(), "the reel's file did not complete"));
			}
		}
		Ok(())
	}

	/// Stops writing: `AVAssetWriter` deletes the file it began.
	pub(super) fn cancel(self) {
		let _ = objc2::exception::catch(AssertUnwindSafe(|| {
			// SAFETY: no append is under way (the job that owns the writer calls this); a writer that did not start or
			// already stopped ignores it.
			unsafe {
				if self.asset.status() == AVAssetWriterStatus::Writing {
					self.asset.cancelWriting();
				}
			}
		}));
	}
}

/// Puts the file on the clipboard as a file (the general pasteboard's file URL), so a chat's paste takes it.
///
/// # Errors
/// The pasteboard refused it.
pub(super) fn copy_file(path: &Path) -> anyhow::Result<()> {
	autoreleasepool(|_| {
		let url = url(path)?;
		let board = NSPasteboard::generalPasteboard();
		board.clearContents();
		let object: Retained<ProtocolObject<dyn objc2_app_kit::NSPasteboardWriting>> =
			ProtocolObject::from_retained(url);
		if !board.writeObjects(&NSArray::from_retained_slice(&[object])) {
			bail!("the pasteboard did not take the reel");
		}
		Ok(())
	})
}
