//! The games' icons on the Games tab (docs/plan.md slice 56, design 8): each listed game's icon read from its program
//! on this computer once a run, 128 px, into the data folder's `icons/` as `<id>.png` (`file`: the first 16 bytes of
//! its key's SHA-256 in hex, so no key, which may be a path in the player's folders, names a file), and shown through
//! the asset protocol (`tauri.conf.json`'s scope `$LOCALDATA/QuestPump/icons/**`, `figures::asset_prefix`'s URLs).
//! The reads run on a thread of their own, never on the loop: macOS draws `NSWorkspace`'s `iconForFile` of the game's
//! app bundle into a 128-px bitmap and writes it as PNG; Windows takes the shell's image of its executable
//! (`IShellItemImageFactory::GetImage`, the icon alone, fit in 128 px), its premultiplied BGRA rows written as PNG
//! with the `png` crate. A Steam game's icon on Windows comes from Steam's own copy of it first (`steam_png`), since a
//! game's program may carry its engine's icon (Mechabellum's exe the Unity cube, Rapha's PC, 2026-10-28): the `.ico`
//! a Steam shortcut to the game names, else the small icon in Steam's library cache (`steam::icon_files`), each read
//! with WIC and scaled to 128 px (`image_png`), else its program's as for any game. On macOS a Steam game is an app
//! bundle with its own icon, read as any other. Every run makes each listed game's icon again (`made` starts empty), so
//! a better source found since (a shortcut made later, this rule itself) replaces the file an earlier run left at the
//! next start. A game's program is its app bundle or executable where a record, its key or its running
//! process names it, or its install folder (`Source::Folder`: the first app bundle in it on macOS, one of its own
//! executables under it on Windows); with none, or a read that fails, the tile shows its name's first letter. On macOS
//! a program is read only where the detector reads one (`readable`): under `/Applications` or `~/Applications`, or in
//! a Steam library's games on the startup volume, and on a volume under `/Volumes` only for a game the player asked for
//! (as `steam.rs` reads a library there); never in the Desktop, Documents or Downloads folder, iCloud Drive or a cloud
//! storage folder, whose read asks the player's consent. A file whose game left every list goes (`prune`), and with it
//! what this run made or asked of it, so it is made again once the game comes back. Nothing of an icon is logged but a
//! count.

use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::sync::mpsc;

use sha2::{Digest as _, Sha256};

/// The icons' side in pixels.
const SIDE: u32 = 128;
/// How deep and how wide an install folder is searched for its program (`EXE_DEPTH`'s and `EXE_FOLDERS`' measure in
/// `detect.rs`).
const DEPTH: usize = 3;
const FOLDERS: usize = 256;

/// Where a game's icon is read from.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) enum Source {
	/// The program itself: an app bundle (macOS) or an executable (Windows).
	File(PathBuf),
	/// A game's install folder: the first app bundle in it (macOS), or one of these executables under it (Windows,
	/// by name in any case).
	Folder { dir: PathBuf, images: Vec<String> },
}

/// What the loop asks of the icons' thread.
enum Job {
	/// The icon of `source` into the folder's `file`; `volumes`: the game is one the player asked for, whose program
	/// may be read on a volume under `/Volumes` (macOS).
	Make { key: String, source: Source, file: String, volumes: bool },
	/// Every icon file but these goes.
	Prune(HashSet<String>),
}

/// The icon files, owned by the loop: those made in this run and those found from an earlier one, by key, and the
/// thread that makes them.
#[derive(Default)]
pub(super) struct Icons {
	/// The folder and the asset protocol's URL of its files, once known.
	folder: Option<(PathBuf, String)>,
	/// The keys whose file stands: made in this run, or kept from an earlier one until this run makes it again.
	files: HashSet<String>,
	/// The keys whose icon was made in this run, and every source asked for (a key whose read failed is asked again
	/// only from another source).
	made: HashSet<String>,
	asked: HashSet<(String, Source)>,
	/// The sources the looks of this run saw a game's process run from (macOS: the app bundle around it; Windows: a
	/// Steam game's folder with its processes' images), by key: a game that ran once keeps its source after its close,
	/// ahead of the one its records give.
	seen: HashMap<String, Source>,
	/// The keys every list holds, as last pruned to.
	kept: HashSet<String>,
	jobs: Option<mpsc::Sender<Job>>,
	done: Option<mpsc::Receiver<String>>,
}

/// An icon's file name for a game's key.
fn file(key: &str) -> String {
	let digest = Sha256::digest(key.as_bytes());
	let mut name = digest.iter().take(16).fold(String::with_capacity(36), |mut hex, b| {
		let _ = write!(hex, "{b:02x}");
		hex
	});
	name.push_str(".png");
	name
}

impl Icons {
	/// The folder, made at the first ask, with the files an earlier run left (each shown until this run makes it
	/// again); none without a data folder.
	fn folder(&mut self) -> Option<&(PathBuf, String)> {
		if self.folder.is_none() {
			let dir = crate::runtime::data_home()?.join("icons");
			if let Err(e) = std::fs::create_dir_all(&dir) {
				log::warn!("games: no icons' folder: {e}");
				return None;
			}
			let prefix = crate::desk::figures::asset_prefix(&dir)?;
			self.folder = Some((dir, prefix));
		}
		self.folder.as_ref()
	}

	/// A source a game's running process gave.
	pub(super) fn saw(&mut self, key: &str, source: Source) {
		self.seen.insert(key.to_owned(), source);
	}

	/// The listed keys whose icon is not made yet in this run.
	pub(super) fn wanted<'a>(&self, listed: &'a HashSet<String>) -> impl Iterator<Item = &'a String> {
		listed.iter().filter(|key| !self.made.contains(*key))
	}

	/// The asset protocol's URL of a game's icon, while its file stands.
	pub(super) fn url(&self, key: &str) -> Option<String> {
		let (_, prefix) = self.folder.as_ref()?;
		self.files.contains(key).then(|| format!("{prefix}{}", file(key)))
	}

	/// A look: the icons the thread made since taken in, the files of the keys `listed` from an earlier run kept, each
	/// listed key's icon not made yet asked for from the source its process gave, else from `sources` (its records'),
	/// once a source, and every other file pruned when the lists changed; `asked` the games the player asked for
	/// (`settings.games.asked`).
	pub(super) fn look(&mut self, listed: &HashSet<String>, sources: &HashMap<String, Source>, asked: &[String]) {
		let Some((dir, _)) = self.folder().cloned() else { return };
		let made: Vec<String> = self.done.as_ref().map(|done| done.try_iter().collect()).unwrap_or_default();
		// an icon made for a key pruned since it was asked for went with that prune: it counts for nothing
		for key in made.into_iter().filter(|key| self.kept.contains(key)) {
			self.files.insert(key.clone());
			self.made.insert(key);
		}
		if self.kept != *listed {
			self.kept.clone_from(listed);
			self.files.retain(|key| listed.contains(key));
			// a game that left every list is made again once it comes back
			self.made.retain(|key| listed.contains(key));
			self.asked.retain(|(key, _)| listed.contains(key));
			self.seen.retain(|key, _| listed.contains(key));
			for key in listed {
				if !self.files.contains(key) && dir.join(file(key)).is_file() {
					self.files.insert(key.clone());
				}
			}
			self.send(Job::Prune(listed.iter().map(|key| file(key)).collect()));
		}
		let wanted: Vec<(String, Source)> = self
			.wanted(listed)
			.filter_map(|key| Some((key.clone(), self.seen.get(key).or_else(|| sources.get(key))?.clone())))
			.collect();
		for (key, source) in wanted {
			if self.asked.insert((key.clone(), source.clone())) {
				let (file, volumes) = (file(&key), asked.contains(&key));
				self.send(Job::Make { key, source, file, volumes });
			}
		}
	}

	/// A job to the icons' thread, started at the first.
	fn send(&mut self, job: Job) {
		if self.jobs.is_none() {
			let Some((dir, _)) = self.folder.clone() else { return };
			let (jobs, inbox) = mpsc::channel();
			let (made, done) = mpsc::channel();
			match std::thread::Builder::new().name("games-icons".to_owned()).spawn(move || run(&dir, &inbox, &made)) {
				Ok(_) => (self.jobs, self.done) = (Some(jobs), Some(done)),
				Err(e) => return log::warn!("games: the icons could not be made: {e}"),
			}
		}
		if let Some(jobs) = &self.jobs
			&& jobs.send(job).is_err()
		{
			log::warn!("games: the icons' thread ended");
			self.jobs = None;
		}
	}
}

/// The icons' thread: each job in turn until the loop drops its sender.
fn run(dir: &Path, inbox: &mpsc::Receiver<Job>, made: &mpsc::Sender<String>) {
	#[cfg(windows)]
	let _com = match crate::elevate::Com::init() {
		Ok(com) => com,
		Err(e) => return log::warn!("games: no icons without COM: {e}"),
	};
	for job in inbox {
		match job {
			Job::Make { key, source, file, volumes } => {
				let Some(png) = icon(&source, volumes) else {
					// one line a source a run, naming nothing of the player's folders
					log::info!("games: an icon could not be read");
					continue;
				};
				match write(&dir.join(&file), &png) {
					Ok(()) => {
						let _ = made.send(key);
					}
					Err(e) => log::warn!("games: an icon could not be written: {e}"),
				}
			}
			Job::Prune(keep) => prune(dir, &keep),
		}
	}
}

/// A game's icon as PNG: Steam's own copy of a Steam game's on Windows, else its program's.
fn icon(source: &Source, volumes: bool) -> Option<Vec<u8>> {
	#[cfg(windows)]
	if let Some(png) = steam_png(source) {
		return Some(png);
	}
	program(source, volumes).and_then(|program| png_of(&program))
}

/// An icon's file, written whole beside it first and moved into place, so the page never reads half of one.
fn write(path: &Path, png: &[u8]) -> std::io::Result<()> {
	let part = path.with_extension("png.part");
	std::fs::write(&part, png)?;
	std::fs::rename(&part, path)
}

/// Every icon file but those `keep` names goes; how many went is logged.
fn prune(dir: &Path, keep: &HashSet<String>) {
	let Ok(entries) = std::fs::read_dir(dir) else { return };
	let mut gone = 0_usize;
	for entry in entries.flatten() {
		let name = entry.file_name();
		let Some(name) = name.to_str() else { continue };
		if !keep.contains(name) && std::fs::remove_file(entry.path()).is_ok() {
			gone += 1;
		}
	}
	if gone > 0 {
		log::info!("games: {gone} icons of games no list holds went");
	}
}

/// The program a source names: the file itself, or the first one its folder holds (`DEPTH` folders below it at most,
/// `FOLDERS` folders read, nearest first; a link to another folder never followed); none where it may not be read
/// (`readable`, before any read).
fn program(source: &Source, volumes: bool) -> Option<PathBuf> {
	let (dir, images) = match source {
		Source::File(file) => return (readable(file, volumes) && file.exists()).then(|| file.clone()),
		Source::Folder { dir, images } => (dir, images),
	};
	if !readable(dir, volumes) {
		return None;
	}
	if is_bundle(dir) {
		return Some(dir.clone());
	}
	let mut folders = VecDeque::from([(dir.clone(), 0)]);
	for _ in 0..FOLDERS {
		let (folder, depth) = folders.pop_front()?;
		let Ok(entries) = std::fs::read_dir(&folder) else { continue };
		let mut below = Vec::new();
		for entry in entries.flatten() {
			let Ok(kind) = entry.file_type() else { continue };
			let path = entry.path();
			if kind.is_dir() && is_bundle(&path) {
				if cfg!(target_os = "macos") {
					return Some(path);
				}
			} else if kind.is_dir() && depth < DEPTH {
				below.push((path, depth + 1));
			} else if kind.is_file() && cfg!(windows) {
				let name = entry.file_name();
				if images.iter().any(|image| name.to_str().is_some_and(|n| n.eq_ignore_ascii_case(image))) {
					return Some(path);
				}
			}
		}
		below.sort();
		folders.extend(below);
	}
	None
}

/// Whether a program or a folder may be read for an icon where it stands, by its path alone: under `/Applications` or
/// `~/Applications` (the folders the detector reads an app's `Info.plist` in) or in a Steam library's
/// `steamapps/common` on the startup volume (where it notices a game by its path), and under `/Volumes` only for a game
/// the player asked for (`volumes`); never in a folder whose read asks the player's consent.
#[cfg(target_os = "macos")]
fn readable(path: &Path, volumes: bool) -> bool {
	super::readable(path, volumes)
		&& (path.starts_with("/Volumes")
			|| super::detect::app_folders().iter().any(|folder| path.starts_with(folder))
			|| super::steam::commons().iter().any(|common| path.starts_with(common)))
}

/// Every program may be read for an icon on Windows.
#[cfg(windows)]
fn readable(_: &Path, _: bool) -> bool {
	true
}

/// Whether a folder is an app bundle.
fn is_bundle(path: &Path) -> bool {
	path.extension().is_some_and(|e| e.eq_ignore_ascii_case("app"))
}

/// The innermost app bundle around a program's path (macOS), which holds its `Info.plist` and its icon; none outside
/// a bundle.
pub(super) fn bundle_of(path: &Path) -> Option<&Path> {
	path.ancestors().find(|a| is_bundle(a))
}

/// The app bundle a program's path runs in, as its icon's source (macOS: outside a bundle a program's icon is the
/// system's for any program, and the tile shows its name's first letter).
pub(super) fn bundle_source(path: &Path) -> Option<Source> {
	bundle_of(path).map(|bundle| Source::File(bundle.to_path_buf()))
}

/// A program's icon as PNG: drawn at `SIDE` pixels into a bitmap of its own, inside an autorelease pool of its own.
#[cfg(target_os = "macos")]
fn png_of(program: &Path) -> Option<Vec<u8>> {
	use objc2::AllocAnyThread as _;
	use objc2::rc::autoreleasepool;
	use objc2_app_kit::{
		NSBitmapImageFileType, NSBitmapImageRep, NSCompositingOperation, NSDeviceRGBColorSpace, NSGraphicsContext,
		NSImageInterpolation, NSWorkspace,
	};
	use objc2_foundation::{NSDictionary, NSPoint, NSRect, NSSize, NSString};

	let path = program.to_str()?;
	autoreleasepool(|_| {
		let image = NSWorkspace::sharedWorkspace().iconForFile(&NSString::from_str(path));
		let side = i64::from(SIDE).try_into().ok()?;
		// SAFETY: reads an extern static of AppKit's, a constant string that lives for the process.
		let space = unsafe { NSDeviceRGBColorSpace };
		// SAFETY: no planes: the rep allocates its own buffer of `SIDE` × `SIDE` pixels, four 8-bit samples each
		// with alpha, meshed, its row bytes and pixel bits worked out by AppKit (the zeros).
		let rep = unsafe {
			NSBitmapImageRep::initWithBitmapDataPlanes_pixelsWide_pixelsHigh_bitsPerSample_samplesPerPixel_hasAlpha_isPlanar_colorSpaceName_bytesPerRow_bitsPerPixel(
				NSBitmapImageRep::alloc(),
				std::ptr::null_mut(),
				side,
				side,
				8,
				4,
				true,
				false,
				space,
				0,
				0,
			)
		}?;
		let context = NSGraphicsContext::graphicsContextWithBitmapImageRep(&rep)?;
		NSGraphicsContext::saveGraphicsState_class();
		NSGraphicsContext::setCurrentContext(Some(&context));
		context.setImageInterpolation(NSImageInterpolation::High);
		let whole = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(f64::from(SIDE), f64::from(SIDE)));
		image.drawInRect_fromRect_operation_fraction(whole, NSRect::ZERO, NSCompositingOperation::Copy, 1.0);
		context.flushGraphics();
		NSGraphicsContext::restoreGraphicsState_class();
		// SAFETY: an empty properties dictionary, of the key and value types the method takes.
		let data = unsafe { rep.representationUsingType_properties(NSBitmapImageFileType::PNG, &NSDictionary::new()) }?;
		Some(data.to_vec())
	})
}

/// A program's icon as PNG: the shell's image of it, the icon alone fit in `SIDE` pixels, read as top-down 32-bit
/// rows and written with its alpha taken out of its colours. The path is made whole first (`std::path::absolute`,
/// `GetFullPathNameW`, which turns `/` into `\`): the shell refuses a path with forward slashes (`E_INVALIDARG`), and
/// the Riot Client's record writes League's folder so (`C:/Games/Riot Games/League of Legends`, Rapha's PC,
/// 2026-10-27, whose League tile showed its letter).
#[cfg(windows)]
fn png_of(program: &Path) -> Option<Vec<u8>> {
	use windows::Win32::Foundation::SIZE;
	use windows::Win32::Graphics::Gdi::{
		BI_RGB, BITMAP, BITMAPINFO, BITMAPINFOHEADER, CreateCompatibleDC, DIB_RGB_COLORS, DeleteDC, GetDIBits,
		GetObjectW, HGDIOBJ,
	};
	use windows::Win32::UI::Shell::{IShellItemImageFactory, SHCreateItemFromParsingName, SIIGBF_ICONONLY};
	use windows::core::{HSTRING, Owned};

	let path = HSTRING::from(std::path::absolute(program).ok()?.as_os_str());
	// SAFETY: the path is a wide string that lives for the call; no bind context.
	let factory: IShellItemImageFactory = unsafe { SHCreateItemFromParsingName(&path, None) }.ok()?;
	let side = i32::try_from(SIDE).ok()?;
	// SAFETY: the factory is alive; the bitmap it returns is the caller's, which `Owned` deletes once.
	let bitmap = unsafe { Owned::new(factory.GetImage(SIZE { cx: side, cy: side }, SIIGBF_ICONONLY).ok()?) };
	let mut info = BITMAP::default();
	let bytes = i32::try_from(size_of::<BITMAP>()).ok()?;
	// SAFETY: the bitmap is alive; GetObjectW writes at most `bytes` bytes into `info`, a local of that size.
	let read = unsafe { GetObjectW(HGDIOBJ::from(*bitmap), bytes, Some((&raw mut info).cast())) };
	let (width, height) = (info.bmWidth, info.bmHeight.abs());
	if read == 0 || width <= 0 || height <= 0 || width > side || height > side {
		return None;
	}
	let (columns, rows) = (u32::try_from(width).ok()?, u32::try_from(height).ok()?);
	let mut header = BITMAPINFO {
		bmiHeader: BITMAPINFOHEADER {
			biSize: u32::try_from(size_of::<BITMAPINFOHEADER>()).ok()?,
			biWidth: width,
			biHeight: -height, // negative: top-down rows
			biPlanes: 1,
			biBitCount: 32,
			biCompression: BI_RGB.0,
			..Default::default()
		},
		..Default::default()
	};
	let mut pixels = vec![0_u8; usize::try_from(columns * rows * 4).ok()?];
	// SAFETY: CreateCompatibleDC(None) makes a memory DC like the screen's, deleted below on every path.
	let dc = unsafe { CreateCompatibleDC(None) };
	if dc.is_invalid() {
		return None;
	}
	// SAFETY: the DC and the bitmap are alive and the bitmap is selected into no DC; GetDIBits writes `height` rows
	// of `width` 32-bit pixels, the buffer's length, as `header` asks for them.
	let lines =
		unsafe { GetDIBits(dc, *bitmap, 0, rows, Some(pixels.as_mut_ptr().cast()), &raw mut header, DIB_RGB_COLORS) };
	// SAFETY: the memory DC made above, deleted once.
	unsafe {
		let _ = DeleteDC(dc);
	}
	if lines != height {
		return None;
	}
	straighten(&mut pixels);
	encode(&pixels, columns, rows)
}

/// The most of an image file read for a Steam game's icon (Mechabellum's `.ico`, every size to 256 px, is 104 KB).
#[cfg(windows)]
const IMAGE_BYTES: u64 = 1024 * 1024;

/// A Steam game's own icon as PNG: the first of the files Steam keeps of it (`steam::icon_files`) that reads, for a
/// source in a Steam library's `steamapps\common` (`steam::app_holding`); none for any other.
#[cfg(windows)]
fn steam_png(source: &Source) -> Option<Vec<u8>> {
	let (Source::File(path) | Source::Folder { dir: path, .. }) = source;
	let app = super::steam::app_holding(path)?;
	super::steam::icon_files(app).iter().find_map(|file| image_png(file))
}

/// An image file's picture as PNG, fit in `SIDE` pixels: WIC decodes it (an `.ico`, a JPEG), takes its largest frame
/// (an `.ico` holds one a size), converts it to premultiplied BGRA and scales it with its high-quality cubic filter,
/// up as well as down; none for a file over `IMAGE_BYTES` or one WIC does not read.
#[cfg(windows)]
fn image_png(file: &Path) -> Option<Vec<u8>> {
	use windows::Win32::Foundation::GENERIC_READ;
	use windows::Win32::Graphics::Imaging::{
		CLSID_WICImagingFactory, GUID_WICPixelFormat32bppPBGRA, IWICBitmapFrameDecode, IWICImagingFactory,
		WICBitmapDitherTypeNone, WICBitmapInterpolationModeHighQualityCubic, WICBitmapPaletteTypeCustom,
		WICDecodeMetadataCacheOnDemand,
	};
	use windows::Win32::System::Com::{CLSCTX_INPROC_SERVER, CoCreateInstance};
	use windows::core::HSTRING;

	if std::fs::metadata(file).ok()?.len() > IMAGE_BYTES {
		return None;
	}
	// SAFETY: COM runs on this thread (`run`); the factory is an in-process object of the system's.
	let factory: IWICImagingFactory =
		unsafe { CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER) }.ok()?;
	let path = HSTRING::from(file.as_os_str());
	// SAFETY: the path is a wide string that lives for the call; the decoder reads the file alone.
	let decoder =
		unsafe { factory.CreateDecoderFromFilename(&path, None, GENERIC_READ, WICDecodeMetadataCacheOnDemand) }.ok()?;
	let mut best: Option<(u32, u32, IWICBitmapFrameDecode)> = None;
	// SAFETY: the decoder is alive.
	let frames = unsafe { decoder.GetFrameCount() }.ok()?;
	for index in 0..frames {
		// SAFETY: the decoder is alive and the index is below its count of frames.
		let Ok(frame) = (unsafe { decoder.GetFrame(index) }) else { continue };
		let (mut width, mut height) = (0, 0);
		// SAFETY: the frame is alive; GetSize writes two locals.
		if unsafe { frame.GetSize(&raw mut width, &raw mut height) }.is_err() {
			continue;
		}
		let area = |w: u32, h: u32| u64::from(w) * u64::from(h);
		if best.as_ref().is_none_or(|(w, h, _)| area(width, height) > area(*w, *h)) {
			best = Some((width, height, frame));
		}
	}
	let (width, height, frame) = best.filter(|(w, h, _)| *w > 0 && *h > 0)?;
	let longest = width.max(height);
	let fit = |side: u32| u32::try_from(u64::from(side) * u64::from(SIDE) / u64::from(longest)).ok().map(|s| s.max(1));
	let (columns, rows) = (fit(width)?, fit(height)?);
	// SAFETY: the factory is alive; the converter and the scaler it makes read the frame, which outlives them.
	let mut pixels = unsafe {
		let converter = factory.CreateFormatConverter().ok()?;
		converter
			.Initialize(
				&frame,
				&GUID_WICPixelFormat32bppPBGRA,
				WICBitmapDitherTypeNone,
				None,
				0.0,
				WICBitmapPaletteTypeCustom,
			)
			.ok()?;
		let scaler = factory.CreateBitmapScaler().ok()?;
		scaler.Initialize(&converter, columns, rows, WICBitmapInterpolationModeHighQualityCubic).ok()?;
		let mut pixels = vec![0_u8; usize::try_from(columns * rows * 4).ok()?];
		scaler.CopyPixels(std::ptr::null(), columns * 4, &mut pixels).ok()?;
		pixels
	};
	straighten(&mut pixels);
	encode(&pixels, columns, rows)
}

/// Premultiplied BGRA pixels, as the shell and WIC give them, turned into PNG's straight RGBA in place.
#[cfg(windows)]
fn straighten(pixels: &mut [u8]) {
	for [blue, green, red, alpha] in pixels.as_chunks_mut::<4>().0 {
		let straight = |colour: u8| {
			let colour = (u16::from(colour) * 255).checked_div(u16::from(*alpha)).unwrap_or(0);
			u8::try_from(colour).unwrap_or(u8::MAX)
		};
		(*red, *green, *blue) = (straight(*blue), straight(*green), straight(*red));
	}
}

/// RGBA rows as PNG.
#[cfg(windows)]
fn encode(rgba: &[u8], width: u32, height: u32) -> Option<Vec<u8>> {
	let mut out = Vec::new();
	let mut encoder = png::Encoder::new(&mut out, width, height);
	encoder.set_color(png::ColorType::Rgba);
	encoder.set_depth(png::BitDepth::Eight);
	let mut writer = encoder.write_header().ok()?;
	writer.write_image_data(rgba).ok()?;
	writer.finish().ok()?;
	Some(out)
}
