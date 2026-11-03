//! The reel on Windows (rust-helper `desktop-reel.md`): the films read by Media Foundation's source reader as RGB32
//! frames (its video processing turns the decoder's YUV into RGB), the reel written by its sink writer into an MPEG-4
//! file (H.264 High at level 4.0 from NV12 frames this module converts in BT.709's video range, a GOP of `key_every`
//! frames, no B-frames, unconstrained VBR at the average bit rate; AAC-LC from 16-bit PCM at 16,000 bytes a second, or
//! no audio stream at all). The output type goes in before the input type, as the H.264 encoder asks, and the encoder's
//! settings ride `SetInputMediaType`'s encoding parameters; no hardware transform is asked for, so the inbox encoder,
//! which honours a B-frame count of 0, makes it. The writer writes into a byte stream of its own file (`MFCreateFile`);
//! as it goes it shuts its MPEG-4 media sink down where it is unfinished (failed or cancelled) and closes that stream
//! in every case, so no handle keeps the file and its folder from being read or deleted. `mp4::faststart` then puts its
//! moov first. Media Foundation's DLLs are loaded at the first reel, never linked: a Windows N edition without its
//! Media Feature Pack lacks them, and QuestPump must still start there; its reels are `Unsupported` ("This computer
//! cannot make videos."), as is a system with no H.264 encoder (`MF_E_TOPO_CODEC_NOT_FOUND`). A file on the clipboard
//! is a `CF_HDROP` (a `DROPFILES` with its one wide path) set while the main window owns the clipboard.

use std::ffi::c_void;
use std::path::Path;
use std::sync::OnceLock;

use anyhow::{Context as _, anyhow, bail};
use tauri::WebviewWindow;
use windows::Win32::Foundation::{GlobalFree, HANDLE, HMODULE, HWND, POINT};
use windows::Win32::Media::MediaFoundation::{
	CODECAPI_AVEncCommonRateControlMode, CODECAPI_AVEncMPVDefaultBPictureCount, CODECAPI_AVEncMPVGOPSize, IMF2DBuffer,
	IMFAttributes, IMFByteStream, IMFMediaBuffer, IMFMediaSink, IMFMediaType, IMFSample, IMFSinkWriter,
	IMFSourceReader, MF_ACCESSMODE_READWRITE, MF_E_TOPO_CODEC_NOT_FOUND, MF_FILE_ACCESSMODE, MF_FILE_FLAGS,
	MF_FILE_OPENMODE, MF_FILEFLAGS_NONE, MF_MT_AAC_PAYLOAD_TYPE, MF_MT_ALL_SAMPLES_INDEPENDENT,
	MF_MT_AUDIO_AVG_BYTES_PER_SECOND, MF_MT_AUDIO_BITS_PER_SAMPLE, MF_MT_AUDIO_BLOCK_ALIGNMENT,
	MF_MT_AUDIO_NUM_CHANNELS, MF_MT_AUDIO_SAMPLES_PER_SECOND, MF_MT_AVG_BITRATE, MF_MT_DEFAULT_STRIDE,
	MF_MT_FRAME_RATE, MF_MT_FRAME_SIZE, MF_MT_INTERLACE_MODE, MF_MT_MAJOR_TYPE, MF_MT_MPEG2_LEVEL, MF_MT_MPEG2_PROFILE,
	MF_MT_PIXEL_ASPECT_RATIO, MF_MT_SUBTYPE, MF_MT_TRANSFER_FUNCTION, MF_MT_VIDEO_NOMINAL_RANGE, MF_MT_VIDEO_PRIMARIES,
	MF_MT_YUV_MATRIX, MF_OPENMODE_DELETE_IF_EXIST, MF_SINK_WRITER_MEDIASINK, MF_SOURCE_READER_ENABLE_VIDEO_PROCESSING,
	MF_SOURCE_READER_FIRST_VIDEO_STREAM, MF_SOURCE_READERF_ENDOFSTREAM, MF_SOURCE_READERF_ERROR, MF_VERSION,
	MFAudioFormat_AAC, MFAudioFormat_PCM, MFMediaType_Audio, MFMediaType_Video, MFNominalRange_16_235, MFSTARTUP_FULL,
	MFVideoFormat_H264, MFVideoFormat_NV12, MFVideoFormat_RGB32, MFVideoInterlace_Progressive, MFVideoPrimaries_BT709,
	MFVideoTransFunc_709, MFVideoTransferMatrix_BT709, eAVEncCommonRateControlMode_UnconstrainedVBR, eAVEncH264VLevel4,
	eAVEncH264VProfile_High,
};
use windows::Win32::System::Com::{COINIT_MULTITHREADED, CoInitializeEx, CoUninitialize};
use windows::Win32::System::DataExchange::{CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData};
use windows::Win32::System::LibraryLoader::{GetProcAddress, LOAD_LIBRARY_SEARCH_SYSTEM32, LoadLibraryExW};
use windows::Win32::System::Memory::{GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalUnlock};
use windows::Win32::System::Ole::CF_HDROP;
use windows::Win32::UI::Shell::DROPFILES;
use windows::core::{BOOL, GUID, HRESULT, HSTRING, Interface as _, PCSTR, PCWSTR, s, w};

use super::compose::Canvas;
use super::{Encoding, Frame, Unsupported};

/// The AAC encoder's 128 kbps (its output type's bytes a second: 12,000, 16,000, 20,000 or 24,000).
const AAC_BYTES_PER_SECOND: u32 = 16_000;
/// Media Foundation's time unit: 100 ns.
const TICKS_PER_SECOND: i64 = 10_000_000;

type Startup = unsafe extern "system" fn(u32, u32) -> HRESULT;
type Shutdown = unsafe extern "system" fn() -> HRESULT;
type CreateAttributes = unsafe extern "system" fn(*mut *mut c_void, u32) -> HRESULT;
type CreateObject = unsafe extern "system" fn(*mut *mut c_void) -> HRESULT;
type CreateMemoryBuffer = unsafe extern "system" fn(u32, *mut *mut c_void) -> HRESULT;
type CreateSinkWriter = unsafe extern "system" fn(PCWSTR, *mut c_void, *mut c_void, *mut *mut c_void) -> HRESULT;
type CreateSourceReader = unsafe extern "system" fn(PCWSTR, *mut c_void, *mut *mut c_void) -> HRESULT;
type CreateFile =
	unsafe extern "system" fn(MF_FILE_ACCESSMODE, MF_FILE_OPENMODE, MF_FILE_FLAGS, PCWSTR, *mut *mut c_void) -> HRESULT;

/// Media Foundation's functions the reel calls, from `mfplat.dll` and `mfreadwrite.dll` in System32.
struct Mf {
	startup: Startup,
	shutdown: Shutdown,
	create_attributes: CreateAttributes,
	create_media_type: CreateObject,
	create_sample: CreateObject,
	create_memory_buffer: CreateMemoryBuffer,
	create_file: CreateFile,
	create_sink_writer: CreateSinkWriter,
	create_source_reader: CreateSourceReader,
}

static MF: OnceLock<Option<Mf>> = OnceLock::new();

/// A function of a loaded DLL as `T`, a function pointer of its exact signature.
fn function<T: Copy>(module: HMODULE, name: PCSTR) -> Option<T> {
	// SAFETY: `module` stays loaded for the process's life, and the name is a NUL-terminated literal.
	let found = unsafe { GetProcAddress(module, name) }?;
	debug_assert_eq!(std::mem::size_of::<T>(), std::mem::size_of_val(&found));
	// SAFETY: every `T` asked for is the documented signature of the export named (mfapi.h, mfreadwrite.h), a function
	// pointer of the same size.
	Some(unsafe { std::mem::transmute_copy(&found) })
}

/// Media Foundation's functions, loaded once from System32; none where its DLLs are missing (a Windows N edition
/// without its Media Feature Pack).
fn load() -> Option<Mf> {
	// SAFETY: two system DLLs by name from System32 alone, kept loaded for the process's life.
	let (plat, readwrite) = unsafe {
		(
			LoadLibraryExW(w!("mfplat.dll"), None, LOAD_LIBRARY_SEARCH_SYSTEM32).ok()?,
			LoadLibraryExW(w!("mfreadwrite.dll"), None, LOAD_LIBRARY_SEARCH_SYSTEM32).ok()?,
		)
	};
	Some(Mf {
		startup: function(plat, s!("MFStartup"))?,
		shutdown: function(plat, s!("MFShutdown"))?,
		create_attributes: function(plat, s!("MFCreateAttributes"))?,
		create_media_type: function(plat, s!("MFCreateMediaType"))?,
		create_sample: function(plat, s!("MFCreateSample"))?,
		create_memory_buffer: function(plat, s!("MFCreateMemoryBuffer"))?,
		create_file: function(plat, s!("MFCreateFile"))?,
		create_sink_writer: function(readwrite, s!("MFCreateSinkWriterFromURL"))?,
		create_source_reader: function(readwrite, s!("MFCreateSourceReaderFromURL"))?,
	})
}

/// Media Foundation, or `Unsupported` without it.
fn mf() -> anyhow::Result<&'static Mf> {
	MF.get_or_init(load).as_ref().ok_or_else(|| anyhow::Error::new(Unsupported))
}

/// An interface a Media Foundation function made through its out pointer.
fn made<T: windows::core::Interface>(status: HRESULT, raw: *mut c_void, what: &str) -> anyhow::Result<T> {
	status.ok().with_context(|| what.to_owned())?;
	if raw.is_null() {
		bail!("{what}: nothing came back");
	}
	// SAFETY: the function succeeded and handed over one reference to an object of the interface it documents.
	Ok(unsafe { T::from_raw(raw) })
}

fn attributes(mf: &Mf, size: u32) -> anyhow::Result<IMFAttributes> {
	let mut raw = std::ptr::null_mut();
	// SAFETY: an out pointer and an initial size.
	let status = unsafe { (mf.create_attributes)(&raw mut raw, size) };
	made(status, raw, "an attribute store") // copy-lint: not shown (an error's words for the log)
}

fn media_type(mf: &Mf) -> anyhow::Result<IMFMediaType> {
	let mut raw = std::ptr::null_mut();
	// SAFETY: an out pointer.
	let status = unsafe { (mf.create_media_type)(&raw mut raw) };
	made(status, raw, "a media type") // copy-lint: not shown (an error's words for the log)
}

/// A sample of one memory buffer holding `bytes`, at `time` for `duration` (100-ns units).
fn sample(mf: &Mf, bytes: &[u8], time: i64, duration: i64) -> anyhow::Result<IMFSample> {
	let len = u32::try_from(bytes.len())?;
	let mut raw = std::ptr::null_mut();
	// SAFETY: a buffer's size and an out pointer.
	let status = unsafe { (mf.create_memory_buffer)(len, &raw mut raw) };
	let buffer: IMFMediaBuffer = made(status, raw, "a media buffer")?; // copy-lint: not shown (an error's words for the log)
	// SAFETY: the buffer is locked while `len` bytes are copied into it, its length set and then unlocked.
	unsafe {
		let mut data = std::ptr::null_mut();
		buffer.Lock(&raw mut data, None, None)?;
		std::ptr::copy_nonoverlapping(bytes.as_ptr(), data, bytes.len());
		buffer.Unlock()?;
		buffer.SetCurrentLength(len)?;
	}
	let mut raw = std::ptr::null_mut();
	// SAFETY: an out pointer.
	let status = unsafe { (mf.create_sample)(&raw mut raw) };
	let sample: IMFSample = made(status, raw, "a sample")?; // copy-lint: not shown (an error's words for the log)
	// SAFETY: the sample takes the buffer and its timing (the AAC encoder needs both, its duration non-zero).
	unsafe {
		sample.AddBuffer(&buffer)?;
		sample.SetSampleTime(time)?;
		sample.SetSampleDuration(duration.max(1))?;
	}
	Ok(sample)
}

/// COM and Media Foundation on the job's thread while a reel is made, shut down as it ends.
pub(super) struct Session(&'static Mf);

impl Session {
	/// Starts both on this thread.
	///
	/// # Errors
	/// Media Foundation is missing (`Unsupported`) or does not start.
	pub(super) fn start() -> anyhow::Result<Self> {
		let mf = mf()?;
		// SAFETY: this thread joins the multithreaded apartment for the job's life; the drop leaves it.
		unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }.ok().context("COM")?;
		// SAFETY: Media Foundation's full start, balanced by the drop's shutdown.
		if let Err(e) = unsafe { (mf.startup)(MF_VERSION, MFSTARTUP_FULL) }.ok() {
			// SAFETY: the apartment this thread joined just above.
			unsafe { CoUninitialize() };
			return Err(anyhow!(e).context("Media Foundation"));
		}
		Ok(Self(mf))
	}
}

impl Drop for Session {
	fn drop(&mut self) {
		// SAFETY: the start this session made, then the apartment it joined.
		unsafe {
			let _ = (self.0.shutdown)();
			CoUninitialize();
		}
	}
}

/// Reads a film's frames in order, handing each over (its time, its BGRA rows top-down, its size).
///
/// # Errors
/// The film does not open or read.
pub(super) fn read_film(path: &Path, each: &mut dyn FnMut(Frame<'_>)) -> anyhow::Result<()> {
	let mf = mf()?;
	let options = attributes(mf, 1)?;
	// SAFETY: a flag on a store this function owns.
	unsafe { options.SetUINT32(&MF_SOURCE_READER_ENABLE_VIDEO_PROCESSING, 1)? };
	let url = HSTRING::from(path);
	let mut raw = std::ptr::null_mut();
	// SAFETY: the film's path, the store and an out pointer.
	let status = unsafe { (mf.create_source_reader)(PCWSTR(url.as_ptr()), options.as_raw(), &raw mut raw) };
	let reader: IMFSourceReader = made(status, raw, "the film's reader")?; // copy-lint: not shown (an error's words for the log)
	#[expect(clippy::cast_sign_loss, reason = "the first video stream's documented constant, 0xFFFFFFFC")]
	let stream = MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32;
	let rgb = media_type(mf)?;
	// SAFETY: the reader's first video stream as RGB32 and the type it then reads.
	let current = unsafe {
		rgb.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)?;
		rgb.SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_RGB32)?;
		reader.SetCurrentMediaType(stream, None, &rgb)?;
		reader.GetCurrentMediaType(stream)?
	};
	// SAFETY: the frame size of the reader's current type.
	let size = unsafe { current.GetUINT64(&MF_MT_FRAME_SIZE)? };
	let (width, height) = (usize::try_from(size >> 32)?, usize::try_from(size & 0xFFFF_FFFF)?);
	// RGB in memory is bottom-up unless the type says otherwise
	// SAFETY: the stride of the reader's current type, where it names one.
	let named = unsafe { current.GetUINT32(&MF_MT_DEFAULT_STRIDE) }.ok().map(u32::cast_signed);
	let fallback = named.map_or(-(i64::try_from(width)? * 4), i64::from);
	let mut top_down = vec![0_u8; width * height * 4];
	loop {
		let (mut flags, mut time, mut got) = (0_u32, 0_i64, None::<IMFSample>);
		// SAFETY: one synchronous read of the stream into the out parameters.
		unsafe { reader.ReadSample(stream, 0, None, Some(&raw mut flags), Some(&raw mut time), Some(&raw mut got))? };
		// after an error the reader takes no further call
		if flags.cast_signed() & MF_SOURCE_READERF_ERROR.0 != 0 {
			bail!("the film's reader failed");
		}
		if flags.cast_signed() & MF_SOURCE_READERF_ENDOFSTREAM.0 != 0 {
			break;
		}
		let Some(sample) = got else { continue };
		// SAFETY: the sample's one buffer, locked for reading through its 2-D interface where it has one (its scan line
		// 0 and its pitch, negative bottom-up), else as one block of the type's stride, and unlocked after.
		unsafe {
			let buffer = sample.ConvertToContiguousBuffer()?;
			let (base, pitch, two_d) = if let Ok(two_d) = buffer.cast::<IMF2DBuffer>() {
				let (mut line, mut pitch) = (std::ptr::null_mut(), 0_i32);
				two_d.Lock2D(&raw mut line, &raw mut pitch)?;
				(line.cast_const(), i64::from(pitch), Some(two_d))
			} else {
				let mut data = std::ptr::null_mut();
				buffer.Lock(&raw mut data, None, None)?;
				// a bottom-up block's scan line 0 is its last row
				let base = if fallback < 0 { data.add(width * 4 * height.saturating_sub(1)) } else { data };
				(base.cast_const(), fallback, None)
			};
			if !base.is_null() {
				for (y, row) in top_down.chunks_exact_mut(width * 4).enumerate() {
					let from = base.offset(isize::try_from(i64::try_from(y)? * pitch)?);
					std::ptr::copy_nonoverlapping(from, row.as_mut_ptr(), width * 4);
				}
			}
			match two_d {
				Some(two_d) => two_d.Unlock2D()?,
				None => buffer.Unlock()?,
			}
		}
		#[expect(clippy::cast_precision_loss, reason = "a film's time in 100-ns units")]
		each(Frame { at: time as f64 / 1e7, px: &top_down, width, height, stride: width * 4 });
	}
	Ok(())
}

/// The reel's file as Media Foundation's byte stream, closed as it drops (`IMFByteStream::Close` releases the file's
/// handle), whatever the sink writer that wrote into it did.
struct File(IMFByteStream);

impl Drop for File {
	fn drop(&mut self) {
		// SAFETY: the file's byte stream, closed once nothing writes into it: its writer's media sink is shut down, or
		// there never was one.
		if let Err(e) = unsafe { self.0.Close() } {
			log::warn!("reel: the reel's file did not close: {e}");
		}
	}
}

/// A reel being written: the sink writer, its file (held for its drop, declared after the writer, so it closes after
/// the writer is let go, finished or not), whether `Finalize` finished it, its video stream and its audio stream where
/// the reel has sound. Dropped unfinished (failed, cancelled or unwinding), it shuts the writer's media sink down
/// first: a sink writer let go unfinished documents no shutdown, and the job deletes that reel's folder at once.
pub(super) struct Writer {
	mf: &'static Mf,
	sink: IMFSinkWriter,
	_file: File,
	finished: bool,
	video: u32,
	audio: Option<u32>,
	/// The reel's whole sound and how many of its audio frames are written: each frame's sample goes in after the
	/// sound up to half a second past it, so the file's chunks interleave.
	sound: Vec<i16>,
	sounded: usize,
	fps: i64,
	hz: i64,
	nv12: Vec<u8>,
}

/// Sets a media type's attributes from `(key, value)` pairs.
fn set(
	kind: &IMFMediaType,
	guids: &[(&GUID, &GUID)],
	words: &[(&GUID, u32)],
	longs: &[(&GUID, u64)],
) -> anyhow::Result<()> {
	// SAFETY: attributes set on a media type this module owns.
	unsafe {
		for (k, v) in guids {
			kind.SetGUID(*k, *v)?;
		}
		for (k, v) in words {
			kind.SetUINT32(*k, *v)?;
		}
		for (k, v) in longs {
			kind.SetUINT64(*k, *v)?;
		}
	}
	Ok(())
}

/// Two numbers packed as Media Foundation's ratios and sizes take them.
fn pair(high: u32, low: u32) -> u64 {
	(u64::from(high) << 32) | u64::from(low)
}

#[expect(
	clippy::cast_sign_loss,
	// copy-lint: not shown (a lint's reason)
	reason = "Media Foundation's enumerations, all positive, as the UINT32 they are stored as"
)]
fn word(value: i32) -> u32 {
	value as u32
}

impl Writer {
	/// A sink writer into a new file at `path` (`.mp4`, the container its name gives) for `encoding`, begun.
	///
	/// # Errors
	/// No H.264 encoder (`Unsupported`), or Media Foundation refused a type or the file.
	pub(super) fn new(path: &Path, e: &Encoding) -> anyhow::Result<Self> {
		let mf = mf()?;
		let frame_bytes = usize::try_from(e.width)? * usize::try_from(e.height)? * 3 / 2;
		let url = HSTRING::from(path);
		let mut raw = std::ptr::null_mut();
		// SAFETY: the file's path and an out pointer; the MPEG-4 sink seeks back to write its boxes, so read and write.
		let status = unsafe {
			(mf.create_file)(
				MF_ACCESSMODE_READWRITE,
				MF_OPENMODE_DELETE_IF_EXIST,
				MF_FILEFLAGS_NONE,
				PCWSTR(url.as_ptr()),
				&raw mut raw,
			)
		};
		let file = File(made(status, raw, "the reel's file")?); // copy-lint: not shown (an error's words for the log)
		let mut raw = std::ptr::null_mut();
		// SAFETY: the file's path (its extension names the container; the writer writes into the byte stream), the byte
		// stream, no attributes, and an out pointer.
		let status = unsafe {
			(mf.create_sink_writer)(PCWSTR(url.as_ptr()), file.0.as_raw(), std::ptr::null_mut(), &raw mut raw)
		};
		let sink: IMFSinkWriter = made(status, raw, "the reel's writer")?; // copy-lint: not shown (an error's words for the log)
		// from here on the drop shuts the writer down and closes the file, whatever fails below
		let mut this = Self {
			mf,
			sink,
			_file: file,
			finished: false,
			video: 0,
			audio: None,
			sound: Vec::new(),
			sounded: 0,
			fps: i64::from(e.fps),
			hz: i64::from(e.audio.map_or(1, |(hz, _)| hz)),
			nv12: vec![0; frame_bytes],
		};
		let writer = &this.sink;
		let size = pair(e.width, e.height);
		let colour = [
			(&MF_MT_VIDEO_PRIMARIES, word(MFVideoPrimaries_BT709.0)),
			(&MF_MT_TRANSFER_FUNCTION, word(MFVideoTransFunc_709.0)),
			(&MF_MT_YUV_MATRIX, word(MFVideoTransferMatrix_BT709.0)),
			(&MF_MT_VIDEO_NOMINAL_RANGE, word(MFNominalRange_16_235.0)),
			(&MF_MT_INTERLACE_MODE, word(MFVideoInterlace_Progressive.0)),
		];
		let h264 = media_type(mf)?;
		set(
			&h264,
			&[(&MF_MT_MAJOR_TYPE, &MFMediaType_Video), (&MF_MT_SUBTYPE, &MFVideoFormat_H264)],
			&colour,
			&[(&MF_MT_FRAME_SIZE, size), (&MF_MT_FRAME_RATE, pair(e.fps, 1)), (&MF_MT_PIXEL_ASPECT_RATIO, pair(1, 1))],
		)?;
		set(
			&h264,
			&[],
			&[
				(&MF_MT_AVG_BITRATE, e.bitrate),
				(&MF_MT_MPEG2_PROFILE, word(eAVEncH264VProfile_High.0)),
				(&MF_MT_MPEG2_LEVEL, word(eAVEncH264VLevel4.0)),
			],
			&[],
		)?;
		let nv12 = media_type(mf)?;
		set(
			&nv12,
			&[(&MF_MT_MAJOR_TYPE, &MFMediaType_Video), (&MF_MT_SUBTYPE, &MFVideoFormat_NV12)],
			&colour,
			&[(&MF_MT_FRAME_SIZE, size), (&MF_MT_FRAME_RATE, pair(e.fps, 1)), (&MF_MT_PIXEL_ASPECT_RATIO, pair(1, 1))],
		)?;
		set(&nv12, &[], &[(&MF_MT_DEFAULT_STRIDE, e.width)], &[])?;
		let encoder = attributes(mf, 3)?;
		// SAFETY: the encoder's settings on a store this function owns.
		unsafe {
			encoder.SetUINT32(&CODECAPI_AVEncMPVGOPSize, e.key_every)?;
			encoder.SetUINT32(&CODECAPI_AVEncMPVDefaultBPictureCount, 0)?;
			encoder.SetUINT32(
				&CODECAPI_AVEncCommonRateControlMode,
				word(eAVEncCommonRateControlMode_UnconstrainedVBR.0),
			)?;
		}
		// SAFETY: the output type first, then the input type with the encoder's settings, on the writer just made.
		let video = unsafe {
			let video = writer.AddStream(&h264)?;
			match writer.SetInputMediaType(video, &nv12, &encoder) {
				Err(err) if err.code() == MF_E_TOPO_CODEC_NOT_FOUND => return Err(anyhow::Error::new(Unsupported)),
				other => other?,
			}
			video
		};
		let audio = match e.audio {
			Some((hz, _)) => Some(Self::audio_stream(mf, writer, hz)?),
			None => None,
		};
		// SAFETY: every stream's types are set.
		unsafe { writer.BeginWriting()? };
		this.video = video;
		this.audio = audio;
		Ok(this)
	}

	/// The AAC stream at `hz` in stereo and its 16-bit PCM input.
	fn audio_stream(mf: &Mf, writer: &IMFSinkWriter, hz: u32) -> anyhow::Result<u32> {
		let aac = media_type(mf)?;
		set(
			&aac,
			&[(&MF_MT_MAJOR_TYPE, &MFMediaType_Audio), (&MF_MT_SUBTYPE, &MFAudioFormat_AAC)],
			&[
				(&MF_MT_AUDIO_BITS_PER_SAMPLE, 16),
				(&MF_MT_AUDIO_SAMPLES_PER_SECOND, hz),
				(&MF_MT_AUDIO_NUM_CHANNELS, 2),
				(&MF_MT_AUDIO_AVG_BYTES_PER_SECOND, AAC_BYTES_PER_SECOND),
				(&MF_MT_AAC_PAYLOAD_TYPE, 0),
			],
			&[],
		)?;
		let pcm = media_type(mf)?;
		set(
			&pcm,
			&[(&MF_MT_MAJOR_TYPE, &MFMediaType_Audio), (&MF_MT_SUBTYPE, &MFAudioFormat_PCM)],
			&[
				(&MF_MT_AUDIO_BITS_PER_SAMPLE, 16),
				(&MF_MT_AUDIO_SAMPLES_PER_SECOND, hz),
				(&MF_MT_AUDIO_NUM_CHANNELS, 2),
				(&MF_MT_AUDIO_BLOCK_ALIGNMENT, 4),
				(&MF_MT_AUDIO_AVG_BYTES_PER_SECOND, hz * 4),
				(&MF_MT_ALL_SAMPLES_INDEPENDENT, 1),
			],
			&[],
		)?;
		// SAFETY: the AAC output type, then its PCM input, on the writer not begun yet.
		unsafe {
			let audio = writer.AddStream(&aac)?;
			match writer.SetInputMediaType(audio, &pcm, None) {
				Err(err) if err.code() == MF_E_TOPO_CODEC_NOT_FOUND => return Err(anyhow::Error::new(Unsupported)),
				other => other?,
			}
			Ok(audio)
		}
	}

	/// Writes the frame as frame `index` of the reel.
	///
	/// # Errors
	/// The writer refused it.
	pub(super) fn video(&mut self, frame: &Canvas, index: i64) -> anyhow::Result<()> {
		let ahead = usize::try_from(((index + 1) * self.hz / self.fps + self.hz / 2).max(0))?;
		self.write_sound(ahead)?;
		nv12(frame, &mut self.nv12);
		let time = index * TICKS_PER_SECOND / self.fps;
		let duration = (index + 1) * TICKS_PER_SECOND / self.fps - time;
		let sample = sample(self.mf, &self.nv12, time, duration)?;
		// SAFETY: a whole NV12 frame of the stream's size at its time.
		unsafe { self.sink.WriteSample(self.video, &sample)? };
		Ok(())
	}

	/// Hands over the reel's whole sound, 16-bit stereo PCM (`pcm`, interleaved) from its start, written half a second
	/// ahead of the frames; nothing without sound.
	///
	/// # Errors
	/// None here; the writes come with the frames.
	#[expect(clippy::unnecessary_wraps, reason = "macOS' twin appends at once and can fail")]
	pub(super) fn audio(&mut self, pcm: &[i16]) -> anyhow::Result<()> {
		if self.audio.is_some() {
			self.sound = pcm.to_vec();
		}
		Ok(())
	}

	/// Writes the sound up to audio frame `until` (the whole of it at most), a tenth of a second a sample.
	fn write_sound(&mut self, until: usize) -> anyhow::Result<()> {
		let Some(stream) = self.audio else { return Ok(()) };
		let until = until.min(self.sound.len() / 2);
		let chunk = usize::try_from(self.hz / 10)?.max(1);
		while self.sounded < until {
			let end = (self.sounded + chunk).min(until);
			let bytes: Vec<u8> = self.sound[self.sounded * 2..end * 2].iter().flat_map(|s| s.to_le_bytes()).collect();
			let (first, frames) = (i64::try_from(self.sounded)?, i64::try_from(end - self.sounded)?);
			let time = first * TICKS_PER_SECOND / self.hz;
			let duration = (first + frames) * TICKS_PER_SECOND / self.hz - time;
			let sample = sample(self.mf, &bytes, time, duration)?;
			// SAFETY: PCM of the stream's format at its time, its duration non-zero.
			unsafe { self.sink.WriteSample(stream, &sample)? };
			self.sounded = end;
		}
		Ok(())
	}

	/// Finishes the file; the drop then closes it, so it can be read and deleted.
	///
	/// # Errors
	/// The writer could not finish it (the drop then shuts it down as a cancel's does).
	pub(super) fn finish(mut self) -> anyhow::Result<()> {
		self.write_sound(usize::MAX)?;
		// SAFETY: every sample is written; the writer flushes its encoders and writes the file's boxes.
		unsafe { self.sink.Finalize()? };
		self.finished = true;
		Ok(())
	}

	/// Lets the writer go unfinished; the drop closes the file, and the job deletes what it began.
	pub(super) fn cancel(self) {
		drop(self);
	}
}

impl Drop for Writer {
	/// Shuts an unfinished writer's MPEG-4 media sink down (Learn: an application that creates a media sink shuts it
	/// down), so it writes nothing more; then the writer is let go and its file closed.
	fn drop(&mut self) {
		if self.finished {
			return;
		}
		let mut raw = std::ptr::null_mut();
		// SAFETY: the writer's media sink (`MF_SINK_WRITER_MEDIASINK`, `GUID_NULL`: its own `IMFMediaSink`) through an
		// out pointer, handed over with one reference.
		let found = unsafe {
			self.sink.GetServiceForStream(MF_SINK_WRITER_MEDIASINK.0, &GUID::zeroed(), &IMFMediaSink::IID, &raw mut raw)
		};
		match found {
			Ok(()) if !raw.is_null() => {
				// SAFETY: the reference just handed over, released as it drops.
				let media: IMFMediaSink = unsafe { IMFMediaSink::from_raw(raw) };
				// SAFETY: the sink's shutdown; every later call on it, the writer's own at its release included,
				// answers MF_E_SHUTDOWN.
				if let Err(e) = unsafe { media.Shutdown() } {
					log::warn!("reel: the writer's media sink did not shut down: {e}");
				}
			}
			Ok(()) => {}
			Err(e) => log::warn!("reel: the writer's media sink: {e}"),
		}
	}
}

/// A BGRA frame as NV12 in BT.709's video range: Y from each pixel, Cb and Cr from each 2 × 2 block's average.
fn nv12(frame: &Canvas, out: &mut [u8]) {
	let (width, height) = (frame.width, frame.height);
	let px = &frame.px;
	let (luma, chroma) = out.split_at_mut(width * height);
	// BT.709: Kr 0.2126, Kb 0.0722; Y in 16-235, Cb and Cr in 16-240; coefficients in 1/65536
	for (row, line) in luma.chunks_exact_mut(width).enumerate().take(height) {
		for (column, luma) in line.iter_mut().enumerate() {
			let at = (row * width + column) * 4;
			let (blue, green, red) = (i64::from(px[at]), i64::from(px[at + 1]), i64::from(px[at + 2]));
			*luma = clamp_byte(16 + ((11_966 * red + 40_254 * green + 4_064 * blue + 32_768) >> 16));
		}
	}
	for (row, line) in chroma.chunks_exact_mut(width).enumerate().take(height / 2) {
		for column in 0..width / 2 {
			let (mut red, mut green, mut blue) = (0_i64, 0_i64, 0_i64);
			for (right, down) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
				let at = ((row * 2 + down) * width + column * 2 + right) * 4;
				blue += i64::from(px[at]);
				green += i64::from(px[at + 1]);
				red += i64::from(px[at + 2]);
			}
			// the four pixels' sums: a quarter of each coefficient
			line[column * 2] = clamp_byte(128 + ((-1_649 * red - 5_548 * green + 7_197 * blue + 32_768) >> 16));
			line[column * 2 + 1] = clamp_byte(128 + ((7_197 * red - 6_537 * green - 660 * blue + 32_768) >> 16));
		}
	}
}

fn clamp_byte(v: i64) -> u8 {
	u8::try_from(v.clamp(0, 255)).unwrap_or(u8::MAX)
}

/// Puts the file on the clipboard as a file (`CF_HDROP`), the main window its owner, so a chat's paste takes it.
///
/// # Errors
/// The window or the clipboard refused it.
pub(super) fn copy_file(window: &WebviewWindow, path: &Path) -> anyhow::Result<()> {
	let hwnd: HWND = window.hwnd().map_err(|e| anyhow!("{e}"))?;
	// the path in UTF-16 with its NUL, and the list's own NUL after it
	let list: Vec<u8> = wide_nul(path).into_iter().chain([0]).flat_map(u16::to_le_bytes).collect();
	let head = std::mem::size_of::<DROPFILES>();
	let bytes = head + list.len();
	let files = DROPFILES { pFiles: u32::try_from(head)?, pt: POINT::default(), fNC: BOOL(0), fWide: BOOL(1) };
	// SAFETY: a moveable block of `bytes`, locked while the header and the double-NUL-ended wide path are written into
	// it, then unlocked; the clipboard opened by the main window, emptied, given the block (the system's once taken,
	// freed here only when refused) and closed.
	unsafe {
		let memory = GlobalAlloc(GMEM_MOVEABLE, bytes)?;
		let data = GlobalLock(memory).cast::<u8>();
		if data.is_null() {
			let _ = GlobalFree(Some(memory));
			bail!("the clipboard's memory did not lock");
		}
		std::ptr::write_unaligned(data.cast::<DROPFILES>(), files);
		std::ptr::copy_nonoverlapping(list.as_ptr(), data.add(head), list.len());
		let _ = GlobalUnlock(memory);
		if let Err(e) = OpenClipboard(Some(hwnd)) {
			let _ = GlobalFree(Some(memory));
			return Err(anyhow!(e).context("the clipboard did not open"));
		}
		let set = EmptyClipboard().and_then(|()| SetClipboardData(u32::from(CF_HDROP.0), Some(HANDLE(memory.0))));
		let _ = CloseClipboard();
		if let Err(e) = set {
			let _ = GlobalFree(Some(memory));
			return Err(anyhow!(e).context("the clipboard did not take the reel"));
		}
	}
	Ok(())
}

/// A path as the NUL-ended UTF-16 Windows takes.
fn wide_nul(path: &Path) -> Vec<u16> {
	use std::os::windows::ffi::OsStrExt as _;
	path.as_os_str().encode_wide().chain([0]).collect()
}
