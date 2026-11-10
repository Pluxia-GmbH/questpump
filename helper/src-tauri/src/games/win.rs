//! Windows' own records as the games' readers read them (docs/plan.md slices 35 and 48): a text value of the registry,
//! a number value (work mode's microphone and camera, slice 67: `presence.rs`), a key's subkeys, a known folder's path
//! (Saved Games, `AppData\LocalLow`), whether another program holds a UDP port (the table of UDP listeners), a program
//! file's product name (its version resource, read from the file with no process opened) and when Windows started.
//! Read only, never written.

use std::path::{Path, PathBuf};

use windows::Win32::Foundation::{ERROR_INSUFFICIENT_BUFFER, ERROR_NO_MORE_ITEMS, NO_ERROR};
use windows::Win32::NetworkManagement::IpHelper::{GetExtendedUdpTable, UDP_TABLE_OWNER_PID};
use windows::Win32::Networking::WinSock::AF_INET;
use windows::Win32::System::Com::CoTaskMemFree;
use windows::Win32::System::Registry::{
	HKEY, KEY_READ, RRF_RT_REG_QWORD, RRF_RT_REG_SZ, RegEnumKeyExW, RegGetValueW, RegOpenKeyExW,
};
use windows::Win32::UI::Shell::{KF_FLAG_DEFAULT, SHGetKnownFolderPath};
use windows::core::{GUID, HSTRING, Owned, PCWSTR, PWSTR};

/// The longest subkey name the registry allows, in UTF-16 units, its NUL included.
const KEY_UNITS: u32 = 256;
/// How often the table of UDP listeners is asked for at one read, while it grows between its size and its rows.
const TABLE_TRIES: usize = 4;

/// A text value of the registry, in two calls: its size, then the text (an expandable one expanded).
pub(super) fn text(root: HKEY, key: PCWSTR, name: PCWSTR) -> Option<String> {
	let mut len = 0u32;
	// SAFETY: a size query without a buffer; `len` receives the size in bytes, the NUL included.
	let e = unsafe { RegGetValueW(root, key, name, RRF_RT_REG_SZ, None, None, Some(&raw mut len)) };
	if !e.is_ok() || len == 0 {
		return None;
	}
	let mut buf = vec![0u16; usize::try_from(len).ok()?.div_ceil(2)];
	// SAFETY: `buf` holds at least `len` bytes; the call writes at most that many and sets `len` to what it wrote.
	let e = unsafe {
		RegGetValueW(root, key, name, RRF_RT_REG_SZ, None, Some(buf.as_mut_ptr().cast()), Some(&raw mut len))
	};
	if !e.is_ok() {
		return None;
	}
	let units = buf.get(..usize::try_from(len).ok()? / 2)?;
	let text = String::from_utf16(units).ok()?;
	Some(text.trim_end_matches('\0').to_owned()).filter(|t| !t.is_empty())
}

/// A 64-bit number value of the registry (`REG_QWORD`); none when the key or the value is missing or another type.
pub(crate) fn qword(root: HKEY, key: &str, name: &str) -> Option<u64> {
	let (mut value, mut len) = (0u64, u32::try_from(size_of::<u64>()).ok()?);
	// SAFETY: `value` holds the 8 bytes `len` names and lives for the call, which writes at most that many; the key's and
	// the value's names live for the call too.
	let e = unsafe {
		RegGetValueW(
			root,
			&HSTRING::from(key),
			&HSTRING::from(name),
			RRF_RT_REG_QWORD,
			None,
			Some((&raw mut value).cast()),
			Some(&raw mut len),
		)
	};
	e.is_ok().then_some(value)
}

/// The names of a key's subkeys; none when the key cannot be opened for reading.
pub(crate) fn subkeys(root: HKEY, key: &str) -> Vec<String> {
	let mut opened = HKEY::default();
	// SAFETY: opens a key for reading; `opened` receives the handle, owned below once the call succeeded.
	if !unsafe { RegOpenKeyExW(root, &HSTRING::from(key), None, KEY_READ, &raw mut opened) }.is_ok() {
		return Vec::new();
	}
	// SAFETY: RegOpenKeyExW returned a fresh handle that nothing else owns; `Owned` closes it once on drop.
	let opened = unsafe { Owned::new(opened) };
	let mut out = Vec::new();
	for index in 0u32.. {
		let mut name = [0u16; KEY_UNITS as usize];
		let mut len = KEY_UNITS;
		// SAFETY: `name` holds `len` UTF-16 units; the call writes at most that many and sets `len` to the name's
		// length without its NUL. No class or time is asked.
		let e = unsafe {
			RegEnumKeyExW(*opened, index, Some(PWSTR(name.as_mut_ptr())), &raw mut len, None, None, None, None)
		};
		if e == ERROR_NO_MORE_ITEMS || !e.is_ok() {
			break;
		}
		if let Some(units) = usize::try_from(len).ok().and_then(|n| name.get(..n)) {
			out.push(String::from_utf16_lossy(units));
		}
	}
	out
}

/// Whether a process other than this one holds UDP `port` on any local IPv4 address, as Windows' own table of UDP
/// listeners says (`GetExtendedUdpTable`, read only); none when the table cannot be read.
pub(super) fn udp_port_held_elsewhere(port: u16) -> Option<bool> {
	let own = std::process::id();
	let mut size = 0u32;
	// the table grows between the size's ask and the read while programs bind: asked again a few times
	for _ in 0..TABLE_TRIES {
		// a buffer of u32s, the table's own alignment: its count, then each row's address, port and process
		let mut table = vec![0u32; usize::try_from(size).ok()?.div_ceil(4)];
		let buf = (!table.is_empty()).then(|| table.as_mut_ptr().cast());
		// SAFETY: `table` holds `size` bytes (rounded up to whole u32s) and lives for the call, which writes at most that
		// many and sets `size` to what the table needs; with no buffer it only sets `size`.
		let e = unsafe { GetExtendedUdpTable(buf, &raw mut size, false, u32::from(AF_INET.0), UDP_TABLE_OWNER_PID, 0) };
		if e == ERROR_INSUFFICIENT_BUFFER.0 {
			continue;
		}
		if e != NO_ERROR.0 {
			return None;
		}
		let count = usize::try_from(*table.first()?).ok()?;
		let mut rows = table.get(1..)?.as_chunks::<3>().0.iter().take(count);
		// `dwLocalPort` holds the port in network byte order in its low 16 bits
		let held = rows
			.any(|&[_, local, pid]| pid != own && u16::try_from(local & 0xffff).is_ok_and(|p| u16::from_be(p) == port));
		return Some(held);
	}
	None
}

/// A known folder's path for the player (`FOLDERID_SavedGames`, `FOLDERID_LocalAppDataLow`); none when the system
/// names none.
pub(super) fn known_folder(id: &GUID) -> Option<PathBuf> {
	// SAFETY: asks the shell for the folder's path; on success the string is the caller's, freed below.
	let path = unsafe { SHGetKnownFolderPath(id, KF_FLAG_DEFAULT, None) }.ok()?;
	// SAFETY: `path` is the NUL-terminated string the shell just returned and nothing has freed.
	let text = unsafe { path.to_string() }.ok();
	// SAFETY: the shell's string is freed once, with the allocator that made it, and not read after.
	unsafe { CoTaskMemFree(Some(path.0.cast_const().cast())) };
	text.filter(|t| !t.is_empty()).map(PathBuf::from)
}

/// The product name a program file's version resource holds (`StringFileInfo`'s `ProductName` in its first
/// translation), read from the file alone; none without one.
pub(super) fn product_name(path: &Path) -> Option<String> {
	use windows::Win32::Storage::FileSystem::{GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW};
	let file = HSTRING::from(path.as_os_str());
	// SAFETY: a size query for the file's version resource; no handle is asked.
	let size = unsafe { GetFileVersionInfoSizeW(&file, None) };
	if size == 0 {
		return None;
	}
	let mut block = vec![0u8; usize::try_from(size).ok()?];
	// SAFETY: `block` holds `size` bytes and lives for the call, which writes at most that many.
	unsafe { GetFileVersionInfoW(&file, None, size, block.as_mut_ptr().cast()) }.ok()?;
	let value = |sub: &str| -> Option<&[u8]> {
		let mut at = std::ptr::null_mut();
		let mut len = 0u32;
		// SAFETY: `block` is the version resource just read, alive while the value is read; the call points `at` into
		// it, at `len` units of the value.
		let found = unsafe { VerQueryValueW(block.as_ptr().cast(), &HSTRING::from(sub), &raw mut at, &raw mut len) };
		if !found.as_bool() || at.is_null() {
			return None;
		}
		let start = at.cast::<u8>().addr().checked_sub(block.as_ptr().addr())?;
		block.get(start..)
	};
	let translation = value(r"\VarFileInfo\Translation")?;
	let lang = u16::from_le_bytes(translation.get(..2)?.try_into().ok()?);
	let page = u16::from_le_bytes(translation.get(2..4)?.try_into().ok()?);
	let text = value(&format!(r"\StringFileInfo\{lang:04x}{page:04x}\ProductName"))?;
	let units: Vec<u16> =
		text.as_chunks::<2>().0.iter().map(|b| u16::from_le_bytes(*b)).take_while(|&u| u != 0).collect();
	let name = String::from_utf16_lossy(&units).trim().to_owned();
	(!name.is_empty()).then_some(name)
}

/// When Windows started, in unix seconds at `now`: its tick count's seconds before it.
pub(super) fn boot_at(now: i64) -> i64 {
	// SAFETY: reads the milliseconds since the system started; no argument.
	let up = unsafe { windows::Win32::System::SystemInformation::GetTickCount64() };
	now - i64::try_from(up / 1_000).unwrap_or(0)
}
