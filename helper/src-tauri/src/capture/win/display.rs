//! The display the strip is on: its GDI name, whether it is an HDR output and its SDR white level (rust-helper
//! `windows-capture-and-install.md`, "HDR and the decoder rule"), and the DXGI adapters and outputs that drive it.

use windows::Win32::Devices::Display::{
	DISPLAYCONFIG_DEVICE_INFO_GET_SDR_WHITE_LEVEL, DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME,
	DISPLAYCONFIG_DEVICE_INFO_HEADER, DISPLAYCONFIG_DEVICE_INFO_TYPE, DISPLAYCONFIG_MODE_INFO, DISPLAYCONFIG_PATH_INFO,
	DISPLAYCONFIG_SDR_WHITE_LEVEL, DISPLAYCONFIG_SOURCE_DEVICE_NAME, DisplayConfigGetDeviceInfo,
	GetDisplayConfigBufferSizes, QDC_ONLY_ACTIVE_PATHS, QueryDisplayConfig,
};
use windows::Win32::Foundation::{ERROR_INSUFFICIENT_BUFFER, HWND, LUID, POINT};
use windows::Win32::Graphics::Dxgi::Common::DXGI_COLOR_SPACE_RGB_FULL_G2084_NONE_P2020;
use windows::Win32::Graphics::Dxgi::{
	CreateDXGIFactory1, DXGI_ERROR_NOT_FOUND, DXGI_OUTPUT_DESC, IDXGIAdapter1, IDXGIFactory1, IDXGIOutput, IDXGIOutput6,
};
use windows::Win32::Graphics::Gdi::{
	GetMonitorInfoW, HMONITOR, MONITOR_DEFAULTTONULL, MONITORINFO, MONITORINFOEXW, MonitorFromPoint, MonitorFromWindow,
};
use windows::core::{Error, HRESULT, Interface, Result};

use super::{byte_size, window};

/// A display, as `GetMonitorInfoW` names it.
pub struct Monitor {
	pub handle: HMONITOR,
	/// The GDI device name, such as `\\.\DISPLAY1`.
	pub name: String,
	device: [u16; 32],
}

/// The display the strip is on: the one under the client area's top-left, or, with that corner off every display, the
/// one the window overlaps most (WGC reads the window wherever it is; DXGI then reads the nearest corner of that
/// display); none when the window is on no display.
pub fn of(hwnd: HWND) -> Option<Monitor> {
	let under = |(x, y)| {
		// SAFETY: MonitorFromPoint only reads the display layout.
		unsafe { MonitorFromPoint(POINT { x, y }, MONITOR_DEFAULTTONULL) }
	};
	let handle = window::client_origin(hwnd).map(under).filter(|m| !m.is_invalid()).unwrap_or_else(|| {
		// SAFETY: MonitorFromWindow only reads the window's position; a stale handle gives a null monitor.
		unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTONULL) }
	});
	if handle.is_invalid() {
		return None;
	}
	let mut info = MONITORINFOEXW::default();
	info.monitorInfo.cbSize = byte_size::<MONITORINFOEXW>();
	// SAFETY: the pointer covers the whole MONITORINFOEXW, and its cbSize asks GetMonitorInfoW for that form.
	if !unsafe { GetMonitorInfoW(handle, (&raw mut info).cast::<MONITORINFO>()) }.as_bool() {
		return None;
	}
	let device = info.szDevice;
	Some(Monitor { handle, name: String::from_utf16_lossy(until_nul(&device)), device })
}

/// Every adapter and output that drives the display, with the output's description, in DXGI's order: on a hybrid-GPU
/// laptop more than one adapter may list it, and only the one that owns it can duplicate it.
pub fn outputs(monitor: HMONITOR) -> Result<Vec<(IDXGIAdapter1, IDXGIOutput, DXGI_OUTPUT_DESC)>> {
	// SAFETY: CreateDXGIFactory1 has no preconditions; the factory is a counted COM reference.
	let factory: IDXGIFactory1 = unsafe { CreateDXGIFactory1()? };
	let mut found = Vec::new();
	for a in 0.. {
		// SAFETY: EnumAdapters1 reads the factory's adapter list; DXGI_ERROR_NOT_FOUND past its end.
		let adapter = match unsafe { factory.EnumAdapters1(a) } {
			Ok(adapter) => adapter,
			Err(e) if e.code() == DXGI_ERROR_NOT_FOUND => break,
			Err(e) => return Err(e),
		};
		for o in 0.. {
			// SAFETY: EnumOutputs reads the adapter's output list; DXGI_ERROR_NOT_FOUND past its end.
			let output = match unsafe { adapter.EnumOutputs(o) } {
				Ok(output) => output,
				Err(e) if e.code() == DXGI_ERROR_NOT_FOUND => break,
				Err(e) => return Err(e),
			};
			// SAFETY: GetDesc only reads the output's description.
			let desc = unsafe { output.GetDesc()? };
			if desc.Monitor == monitor {
				found.push((adapter.clone(), output, desc));
			}
		}
	}
	Ok(found)
}

/// Whether the display is an HDR output: its colour space is HDR10's (`IDXGIOutput6::GetDesc1`); SDR when it cannot
/// be read.
pub fn is_hdr(monitor: &Monitor) -> bool {
	let read = || -> Result<bool> {
		let (_, output, _) =
			outputs(monitor.handle)?.into_iter().next().ok_or_else(|| Error::from_hresult(DXGI_ERROR_NOT_FOUND))?;
		let output: IDXGIOutput6 = output.cast()?;
		// SAFETY: GetDesc1 only reads the output's description.
		let desc = unsafe { output.GetDesc1()? };
		Ok(desc.ColorSpace == DXGI_COLOR_SPACE_RGB_FULL_G2084_NONE_P2020)
	};
	read().unwrap_or_else(|e| {
		log::info!("capture: the colour space of {}: {e}; read as SDR", monitor.name);
		false
	})
}

/// The display's SDR white as a multiple of 80 nits (`SDRWhiteLevel` / 1000, Microsoft Learn
/// `DISPLAYCONFIG_SDR_WHITE_LEVEL`): where the game's white sits in an HDR desktop's scRGB. 1.0 with a log line when
/// it cannot be read.
pub fn sdr_white(monitor: &Monitor) -> f32 {
	match white_level(&monitor.device).map(|l| l.and_then(|l| u16::try_from(l).ok()).filter(|&l| l > 0)) {
		Ok(Some(level)) => f32::from(level) / 1000.0,
		Ok(None) => {
			log::info!("capture: no SDR white level for {}; 1.0", monitor.name);
			1.0
		}
		Err(e) => {
			log::info!("capture: the SDR white level of {}: {e}; 1.0", monitor.name);
			1.0
		}
	}
}

/// `SDRWhiteLevel` of the active path whose source is the GDI device: the source's name asked on the source, the
/// white level on the target.
fn white_level(device: &[u16; 32]) -> Result<Option<u32>> {
	for path in active_paths()? {
		let mut source = DISPLAYCONFIG_SOURCE_DEVICE_NAME {
			header: header::<DISPLAYCONFIG_SOURCE_DEVICE_NAME>(
				DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME,
				path.sourceInfo.adapterId,
				path.sourceInfo.id,
			),
			..Default::default()
		};
		// SAFETY: the pointer covers the whole request, whose header names its type and size.
		let named = unsafe { DisplayConfigGetDeviceInfo((&raw mut source).cast()) } == 0;
		if !named || until_nul(&source.viewGdiDeviceName) != until_nul(device) {
			continue;
		}
		let mut white = DISPLAYCONFIG_SDR_WHITE_LEVEL {
			header: header::<DISPLAYCONFIG_SDR_WHITE_LEVEL>(
				DISPLAYCONFIG_DEVICE_INFO_GET_SDR_WHITE_LEVEL,
				path.targetInfo.adapterId,
				path.targetInfo.id,
			),
			..Default::default()
		};
		// SAFETY: as above, for the white level's request.
		let status = unsafe { DisplayConfigGetDeviceInfo((&raw mut white).cast()) };
		return if status == 0 {
			Ok(Some(white.SDRWhiteLevel))
		} else {
			Err(Error::from_hresult(HRESULT::from_win32(status.cast_unsigned())))
		};
	}
	Ok(None)
}

fn header<T>(kind: DISPLAYCONFIG_DEVICE_INFO_TYPE, adapter: LUID, id: u32) -> DISPLAYCONFIG_DEVICE_INFO_HEADER {
	DISPLAYCONFIG_DEVICE_INFO_HEADER { r#type: kind, size: byte_size::<T>(), adapterId: adapter, id }
}

/// The active display paths, asked again when the configuration changes between sizing and reading them.
fn active_paths() -> Result<Vec<DISPLAYCONFIG_PATH_INFO>> {
	for _ in 0..3 {
		let (mut paths_len, mut modes_len) = (0u32, 0u32);
		// SAFETY: writes the two counts through pointers to locals that live for the call.
		unsafe { GetDisplayConfigBufferSizes(QDC_ONLY_ACTIVE_PATHS, &raw mut paths_len, &raw mut modes_len) }.ok()?;
		let mut paths = vec![DISPLAYCONFIG_PATH_INFO::default(); usize::try_from(paths_len).unwrap_or(0)];
		let mut modes = vec![DISPLAYCONFIG_MODE_INFO::default(); usize::try_from(modes_len).unwrap_or(0)];
		// SAFETY: each array holds as many elements as its count says; the call writes at most that many and lowers
		// the counts to what it wrote.
		let status = unsafe {
			QueryDisplayConfig(
				QDC_ONLY_ACTIVE_PATHS,
				&raw mut paths_len,
				paths.as_mut_ptr(),
				&raw mut modes_len,
				modes.as_mut_ptr(),
				None,
			)
		};
		if status == ERROR_INSUFFICIENT_BUFFER {
			continue;
		}
		status.ok()?;
		paths.truncate(usize::try_from(paths_len).unwrap_or(0));
		return Ok(paths);
	}
	Err(Error::from_hresult(ERROR_INSUFFICIENT_BUFFER.to_hresult()))
}

/// A NUL-terminated UTF-16 name up to its terminator.
fn until_nul(name: &[u16]) -> &[u16] {
	name.iter().position(|&c| c == 0).map_or(name, |end| &name[..end])
}
