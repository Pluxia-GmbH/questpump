//! DXGI Desktop Duplication, written by hand (rust-helper `windows-capture-and-install.md`): the display the game is
//! on, duplicated with a D3D11 device made on the adapter that drives it (windows-capture's `DxgiDuplicationApi` takes
//! the default adapter, which on a hybrid-GPU laptop may not own the display). No capture border ever shows, so
//! Windows 10, whose border cannot be turned off, reads here. Each round takes the newest desktop image, copies the
//! band at the client area's top-left into a staging texture, maps, decodes and releases it; a lost duplication (a
//! mode change, the secure desktop, a full-screen switch) is made again.

use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use tokio::sync::mpsc::UnboundedSender;
use windows::Win32::Foundation::{E_ACCESSDENIED, E_POINTER, HMODULE};
use windows::Win32::Graphics::Direct3D::D3D_DRIVER_TYPE_UNKNOWN;
use windows::Win32::Graphics::Direct3D11::{
	D3D11_BOX, D3D11_CPU_ACCESS_READ, D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_MAP_READ, D3D11_MAPPED_SUBRESOURCE,
	D3D11_SDK_VERSION, D3D11_TEXTURE2D_DESC, D3D11_USAGE_STAGING, D3D11CreateDevice, ID3D11Device, ID3D11DeviceContext,
	ID3D11Texture2D,
};
use windows::Win32::Graphics::Dxgi::Common::{
	DXGI_FORMAT, DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_FORMAT_R16G16B16A16_FLOAT, DXGI_MODE_ROTATION_IDENTITY,
	DXGI_MODE_ROTATION_UNSPECIFIED, DXGI_SAMPLE_DESC,
};
use windows::Win32::Graphics::Dxgi::{
	DXGI_ERROR_ACCESS_LOST, DXGI_ERROR_DEVICE_REMOVED, DXGI_ERROR_DEVICE_RESET, DXGI_ERROR_NOT_CURRENTLY_AVAILABLE,
	DXGI_ERROR_NOT_FOUND, DXGI_ERROR_SESSION_DISCONNECTED, DXGI_ERROR_UNSUPPORTED, DXGI_ERROR_WAIT_TIMEOUT,
	DXGI_OUTDUPL_FRAME_INFO, DXGI_OUTPUT_DESC, IDXGIAdapter1, IDXGIOutput, IDXGIOutput1, IDXGIOutput5,
	IDXGIOutputDuplication, IDXGIResource,
};
use windows::Win32::Graphics::Gdi::HMONITOR;
use windows::core::{Error, HRESULT, Interface, Result};

use super::{BAND, Band, Event, Format, Heard, Origin, Target, Worker, clock, display, send};

/// How long a round waits for a new desktop image: short, so a stop is seen soon.
const ACQUIRE_MS: u32 = 100;
/// How long a lost duplication waits before it is made again (the secure desktop may hold it for a while).
const RETRY: Duration = Duration::from_secs(1);
/// What ends a duplication and asks for a new one, device and all: its access lost (a mode switch, the secure desktop,
/// a full-screen switch) or its GPU removed or reset (a driver update, a TDR).
const LOST: [HRESULT; 3] = [DXGI_ERROR_ACCESS_LOST, DXGI_ERROR_DEVICE_REMOVED, DXGI_ERROR_DEVICE_RESET];
/// What refuses a new duplication only for a while (Microsoft Learn, `IDXGIOutput1::DuplicateOutput`): the secure
/// desktop, a mode switch, too many duplications at once, a disconnected session, the display or the GPU going away
/// (the loop starts a new stream when the window's display changes).
const PASSING: [HRESULT; 7] = [
	E_ACCESSDENIED,
	DXGI_ERROR_UNSUPPORTED,
	DXGI_ERROR_NOT_CURRENTLY_AVAILABLE,
	DXGI_ERROR_SESSION_DISCONNECTED,
	DXGI_ERROR_NOT_FOUND,
	DXGI_ERROR_DEVICE_REMOVED,
	DXGI_ERROR_DEVICE_RESET,
];

/// Starts duplicating the display on a thread of its own; a display DXGI cannot duplicate now is the start's error.
pub fn start(
	monitor: HMONITOR,
	target: &Target,
	origin: Origin,
	events: &UnboundedSender<Event>,
) -> std::result::Result<Worker, String> {
	let monitor = monitor.0.addr();
	let display = move || HMONITOR(std::ptr::without_provenance_mut(monitor));
	let step = Duration::from_secs(1) / target.rate.max(1);
	let band = Band::new(target.hdr, target.white);
	let events = events.clone();
	Worker::spawn(
		"capture-dxgi",
		move || Desktop::open(display()).map_err(Refusal::reason),
		move |desktop, halt, heard| run(desktop, display(), halt, heard, step, &origin, band, &events),
	)
}

/// Reads the display until halted, marking `heard` whenever the desktop image updates. A lost duplication is the strip
/// out of sight once, and a new one is made every second from the display up (a fresh factory, device and duplication)
/// until it comes back; a refusal that lasts (the display turned) ends the stream, and one that passes is waited out:
/// the capture loop ends a stream that stays silent on the player's desktop (`STALL`), and GDI takes over (four other
/// duplications holding the display, a mode DXGI cannot duplicate, a display whose image never updates here).
#[expect(clippy::too_many_arguments, reason = "the stream's thread owns each of these")]
fn run(
	first: Desktop,
	monitor: HMONITOR,
	halt: &AtomicBool,
	heard: &Heard,
	step: Duration,
	origin: &Origin,
	mut band: Band,
	events: &UnboundedSender<Event>,
) -> std::result::Result<(), String> {
	let mut desktop = Some(first);
	let mut refusing = false;
	while !halt.load(Ordering::Relaxed) {
		let Some(open) = desktop.as_mut() else {
			thread::sleep(RETRY);
			match Desktop::open(monitor) {
				Ok(again) => {
					log::info!("capture: the display's duplication is back");
					desktop = Some(again);
					refusing = false;
				}
				Err(Refusal::Passing(why)) => {
					if !std::mem::replace(&mut refusing, true) {
						log::info!("capture: the display's duplication, again: {why}");
					}
				}
				Err(Refusal::Lasting(why)) => return Err(why),
			}
			continue;
		};
		match open.round(origin, &mut band, heard, events) {
			Ok(true) => thread::sleep(step),
			Ok(false) => {}
			Err(e) if LOST.contains(&e.code()) => {
				log::info!("capture: the display's duplication was lost ({e}); making it again");
				desktop = None;
				let _ = events.send(Event::NoStrip);
			}
			Err(e) => return Err(e.to_string()),
		}
	}
	Ok(())
}

/// Why the display could not be duplicated: for a while, which the stream waits out, or for good, which ends it.
enum Refusal {
	Passing(String),
	Lasting(String),
}

impl Refusal {
	fn of(what: &str, e: &Error) -> Self {
		let why = format!("{what}: {e}");
		if PASSING.contains(&e.code()) { Self::Passing(why) } else { Self::Lasting(why) }
	}

	fn reason(self) -> String {
		match self {
			Self::Passing(why) | Self::Lasting(why) => why,
		}
	}
}

/// A display's duplication and the device it copies with; every COM object lives on the stream's thread.
struct Desktop {
	device: ID3D11Device,
	context: ID3D11DeviceContext,
	duplication: IDXGIOutputDuplication,
	/// The output's top-left on the desktop: screen coordinates minus this are the desktop image's texels.
	corner: (i32, i32),
	/// The frequency of the QPC clock `LastPresentTime` counts on, fixed at boot.
	qpc_per_second: i64,
	staging: Option<(ID3D11Texture2D, DXGI_FORMAT)>,
}

impl Desktop {
	/// Duplicates the display with a device on the first adapter, in DXGI's order, that can.
	fn open(monitor: HMONITOR) -> std::result::Result<Self, Refusal> {
		let found = display::outputs(monitor).map_err(|e| Refusal::of("the display's DXGI outputs", &e))?;
		let mut refused = Refusal::Passing("no DXGI output drives the display".into());
		for (adapter, output, desc) in found {
			let corner = corner(&desc).map_err(Refusal::Lasting)?;
			let made =
				device_on(&adapter).and_then(|(device, context)| Ok((duplicate(&output, &device)?, device, context)));
			match made {
				Ok((duplication, device, context)) => {
					let qpc_per_second = clock::qpc().map_or(0, |(_, per_second)| per_second);
					return Ok(Self { device, context, duplication, corner, qpc_per_second, staging: None });
				}
				Err(e) => refused = Refusal::of("no duplication of the display", &e),
			}
		}
		Err(refused)
	}

	/// One round: the newest desktop image, its band decoded and sent. True when a new image was taken, which paces
	/// the next round.
	fn round(
		&mut self,
		origin: &Origin,
		band: &mut Band,
		heard: &Heard,
		events: &UnboundedSender<Event>,
	) -> Result<bool> {
		let duplication = self.duplication.clone();
		let mut info = DXGI_OUTDUPL_FRAME_INFO::default();
		let mut resource: Option<IDXGIResource> = None;
		// SAFETY: AcquireNextFrame writes the frame's info and its desktop resource through pointers to locals that
		// live for the call; no frame is held here (each is released before the next round).
		match unsafe { duplication.AcquireNextFrame(ACQUIRE_MS, &raw mut info, &raw mut resource) } {
			Ok(()) => {}
			Err(e) if e.code() == DXGI_ERROR_WAIT_TIMEOUT => return Ok(false),
			Err(e) => return Err(e),
		}
		let _held = Held(&duplication);
		// no present time: only the pointer moved
		let Some(resource) = resource.filter(|_| info.LastPresentTime != 0) else { return Ok(false) };
		heard.now();
		let texture: ID3D11Texture2D = resource.cast()?;
		let mut desc = D3D11_TEXTURE2D_DESC::default();
		// SAFETY: GetDesc writes the texture's description into `desc`, which lives for the call.
		unsafe { texture.GetDesc(&raw mut desc) };
		let format = match desc.Format {
			DXGI_FORMAT_B8G8R8A8_UNORM => Format::Bgra8,
			DXGI_FORMAT_R16G16B16A16_FLOAT => Format::Rgba16F,
			other => return Err(Error::new(DXGI_ERROR_UNSUPPORTED, format!("a desktop image in format {}", other.0))),
		};
		let (x, y) = origin.get();
		let at = (x.saturating_sub(self.corner.0), y.saturating_sub(self.corner.1));
		let Some((left, top)) = clamp(at, (desc.Width, desc.Height)) else { return Ok(true) };
		let staging = self.staging(desc.Format)?;
		let (w, h) = BAND;
		let area = D3D11_BOX { left, top, front: 0, right: left + w, bottom: top + h, back: 1 };
		// SAFETY: both textures belong to `self.device` and share the format, neither is mapped, and `clamp` put the
		// box inside the desktop texture.
		unsafe { self.context.CopySubresourceRegion(&staging, 0, 0, 0, 0, &texture, 0, Some(&raw const area)) };
		let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
		// SAFETY: Map on this device's CPU-readable staging texture, writing the mapping into `mapped`; it waits for
		// the copy above.
		unsafe { self.context.Map(&staging, 0, D3D11_MAP_READ, 0, Some(&raw mut mapped))? };
		let _mapped = Mapped(&self.context, &staging);
		let (Ok(pitch), Ok(rows)) = (usize::try_from(mapped.RowPitch), usize::try_from(h)) else { return Ok(true) };
		// SAFETY: while mapped, `pData` holds the staging texture's `Height` rows `RowPitch` bytes apart
		// (D3D11_MAPPED_SUBRESOURCE; windows-capture's FrameBuffer reads the same span), and nothing writes to it
		// before the Unmap that `_mapped` makes after the last use of this slice.
		let pixels = unsafe { std::slice::from_raw_parts(mapped.pData.cast::<u8>(), pitch * rows) };
		let read = band.read(format, pixels, pitch);
		send(events, read, || clock::shown(info.LastPresentTime, self.qpc_per_second));
		Ok(true)
	}

	/// The staging texture for the band in `format`, made again when the desktop's format changes.
	fn staging(&mut self, format: DXGI_FORMAT) -> Result<ID3D11Texture2D> {
		if let Some((texture, made)) = &self.staging
			&& *made == format
		{
			return Ok(texture.clone());
		}
		let (w, h) = BAND;
		let desc = D3D11_TEXTURE2D_DESC {
			Width: w,
			Height: h,
			MipLevels: 1,
			ArraySize: 1,
			Format: format,
			SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
			Usage: D3D11_USAGE_STAGING,
			BindFlags: 0,
			CPUAccessFlags: D3D11_CPU_ACCESS_READ.0.cast_unsigned(),
			MiscFlags: 0,
		};
		let mut texture = None;
		// SAFETY: CreateTexture2D reads `desc` and writes the new texture into `texture`; both live for the call.
		unsafe { self.device.CreateTexture2D(&raw const desc, None, Some(&raw mut texture))? };
		let texture = texture.ok_or_else(|| Error::from_hresult(E_POINTER))?;
		self.staging = Some((texture.clone(), format));
		Ok(texture)
	}
}

/// Releases the acquired desktop image on every path out of a round.
struct Held<'a>(&'a IDXGIOutputDuplication);

impl Drop for Held<'_> {
	fn drop(&mut self) {
		// SAFETY: an image is held: AcquireNextFrame succeeded and nothing released it since. A lost duplication
		// fails here too, and the next AcquireNextFrame reports it.
		if let Err(e) = unsafe { self.0.ReleaseFrame() } {
			log::debug!("capture: ReleaseFrame: {e}");
		}
	}
}

/// Unmaps the staging texture on every path out of a round.
struct Mapped<'a>(&'a ID3D11DeviceContext, &'a ID3D11Texture2D);

impl Drop for Mapped<'_> {
	fn drop(&mut self) {
		// SAFETY: the texture's subresource 0 is mapped on this context: Map succeeded and nothing unmapped it since.
		unsafe { self.0.Unmap(self.1, 0) };
	}
}

/// The output's top-left on the desktop; a rotated output is refused for good, since where the duplicated image puts a
/// rotated display's pixels is unverified (GDI, next in the chain, reads it as shown).
fn corner(desc: &DXGI_OUTPUT_DESC) -> std::result::Result<(i32, i32), String> {
	if desc.Rotation != DXGI_MODE_ROTATION_IDENTITY && desc.Rotation != DXGI_MODE_ROTATION_UNSPECIFIED {
		return Err("the display is rotated".into());
	}
	Ok((desc.DesktopCoordinates.left, desc.DesktopCoordinates.top))
}

/// A D3D11 device on the adapter (with an adapter the driver type must be `UNKNOWN`, Microsoft Learn
/// `D3D11CreateDevice`).
fn device_on(adapter: &IDXGIAdapter1) -> Result<(ID3D11Device, ID3D11DeviceContext)> {
	let (mut device, mut context) = (None, None);
	// SAFETY: an adapter with the UNKNOWN driver type and no software module, the default feature levels, and out
	// pointers to locals that live for the call.
	unsafe {
		D3D11CreateDevice(
			adapter,
			D3D_DRIVER_TYPE_UNKNOWN,
			HMODULE::default(),
			D3D11_CREATE_DEVICE_BGRA_SUPPORT,
			None,
			D3D11_SDK_VERSION,
			Some(&raw mut device),
			None,
			Some(&raw mut context),
		)
	}?;
	device.zip(context).ok_or_else(|| Error::from_hresult(E_POINTER))
}

/// The output's duplication: `DuplicateOutput1` with BGRA and FP16, so an HDR desktop arrives unclipped, else
/// `DuplicateOutput` (always BGRA) where the output has no `IDXGIOutput5` or refuses it.
fn duplicate(output: &IDXGIOutput, device: &ID3D11Device) -> Result<IDXGIOutputDuplication> {
	let mut refused = None;
	if let Ok(output5) = output.cast::<IDXGIOutput5>() {
		let formats = [DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_FORMAT_R16G16B16A16_FLOAT];
		// SAFETY: `device` was made on the adapter that lists `output` (the call's precondition); the format list
		// lives for the call.
		match unsafe { output5.DuplicateOutput1(device, 0, &formats) } {
			Ok(duplication) => return Ok(duplication),
			Err(e) => refused = Some(e),
		}
	}
	let output1: IDXGIOutput1 = output.cast()?;
	// SAFETY: as above, for the call that always gives BGRA.
	let duplication = unsafe { output1.DuplicateOutput(device)? };
	if let Some(e) = refused {
		// an HDR desktop then arrives clipped to BGRA, which the SDR white cannot bring back
		log::info!("capture: DuplicateOutput1 refused ({e}); the desktop arrives as BGRA");
	}
	Ok(duplication)
}

/// The band's top-left in a desktop image of `size`: the client area's top-left on the output, moved inside the image
/// when the window hangs over its edge (a copy box outside the texture is undefined, Microsoft Learn
/// `CopySubresourceRegion`); none when the image is smaller than the band.
fn clamp((x, y): (i32, i32), (width, height): (u32, u32)) -> Option<(u32, u32)> {
	let (w, h) = BAND;
	let (max_x, max_y) = (width.checked_sub(w)?, height.checked_sub(h)?);
	let fit = |v: i32, max: u32| u32::try_from(v).map_or(0, |v| v.min(max));
	Some((fit(x, max_x), fit(y, max_y)))
}
