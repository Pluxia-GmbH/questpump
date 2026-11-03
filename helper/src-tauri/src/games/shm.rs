//! The games' documented shared memory (docs/plan.md slice 48; rust-helper rule 10): iRacing's SDK
//! (`Local\IRSDKMemMapFileName`), the graphics page Assetto Corsa Competizione and, since 2026-10-28, Assetto Corsa
//! write (`Local\acpmf_graphics`), Guild Wars 2's `MumbleLink` (2026-10-28) and Automobilista 2's `$pcars2$`
//! (2026-10-29, there once the player sets its Shared Memory to Project CARS 2), each a named file mapping the game
//! makes for other programs to read. At each look the mapping is opened for reading alone (`OpenFileMappingW` with
//! `FILE_MAP_READ`, never `CreateFileMappingW`, so no mapping is made while the game is off), and the reader copies
//! only the byte ranges it asks for (`read`: iRacing's header, its variables' headers and two values of the newest
//! buffer, never the session's YAML with the drivers' names; the first 12 bytes of the Kunos games' page; Guild Wars
//! 2's game name and map type, never its identity with the character's name; Automobilista 2's first 20 bytes, its
//! version and its game, session and race states, never the participants), each within the view's first region while
//! it is committed and readable, before the view is unmapped and the mapping closed again: no handle to the game's
//! process and none kept between looks. The copies go to questpump-core's readers (`feeds::iracing`, `feeds::acc`,
//! `feeds::guildwars`, `feeds::automobilista`) and are dropped. Windows only.

use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Memory::{
	FILE_MAP_READ, MEM_COMMIT, MEMORY_BASIC_INFORMATION, MapViewOfFile, OpenFileMappingW, PAGE_GUARD, PAGE_NOACCESS,
	UnmapViewOfFile, VirtualQuery,
};
use windows::core::{HSTRING, Owned};

/// iRacing's mapping, the Kunos games' graphics page (Assetto Corsa Competizione's and Assetto Corsa's, the same
/// name) and Guild Wars 2's link (the GW2 wiki's API:MumbleLink: `MumbleLink`, the name Mumble's Link plugin opens, in
/// the session's own namespace like the others; the game's `-mumble` option can rename it, and is not followed), by
/// their documented names.
pub(super) const IRACING: &str = r"Local\IRSDKMemMapFileName";
pub(super) const ACC: &str = r"Local\acpmf_graphics";
pub(super) const GUILD_WARS: &str = "MumbleLink";
/// Automobilista 2's mapping, by the name its `SharedMemory.h`'s readers open (no `Local\` prefix).
pub(super) const AUTOMOBILISTA: &str = "$pcars2$";
/// The most one range copies (iRacing's variables' headers, a few hundred of 144 bytes each, are the largest).
const MOST_BYTES: usize = 1024 * 1024;

/// The mapping `name`'s view read by `read`, which asks for copies of byte ranges of it (`(at, len)`, none past the
/// view's first region or over `MOST_BYTES`); none while the game has not made it, or when its pages cannot be read.
pub(super) fn read<T>(
	name: &str,
	read: impl FnOnce(&mut dyn FnMut(usize, usize) -> Option<Vec<u8>>) -> Option<T>,
) -> Option<T> {
	// SAFETY: opens an existing named mapping for reading; the handle is checked and owned below.
	let mapping: HANDLE = unsafe { OpenFileMappingW(FILE_MAP_READ.0, false, &HSTRING::from(name)) }.ok()?;
	// SAFETY: OpenFileMappingW returned a fresh handle that nothing else owns; `Owned` closes it once on drop.
	let mapping = unsafe { Owned::new(mapping) };
	// SAFETY: maps the whole mapping for reading; the address is checked and unmapped below before the handle closes.
	let view = unsafe { MapViewOfFile(*mapping, FILE_MAP_READ, 0, 0, 0) };
	if view.Value.is_null() {
		return None;
	}
	let mut info = MEMORY_BASIC_INFORMATION::default();
	// SAFETY: asks the size of the region the view starts at; `info` lives for the call and its size is passed.
	let asked =
		unsafe { VirtualQuery(Some(view.Value.cast_const()), &raw mut info, size_of::<MEMORY_BASIC_INFORMATION>()) };
	// only pages with storage behind them that may be read: a view of a mapping made with `SEC_RESERVE` starts at
	// reserved pages until its maker commits them, and reading one would end the helper (`Protect` is undefined then)
	let readable =
		info.State == MEM_COMMIT && info.Protect.0 != 0 && (info.Protect & (PAGE_NOACCESS | PAGE_GUARD)).0 == 0;
	let region = if asked == 0 || !readable { 0 } else { info.RegionSize };
	let start = view.Value.cast::<u8>();
	let mut copy = |at: usize, len: usize| -> Option<Vec<u8>> {
		let end = at.checked_add(len)?;
		if len == 0 || len > MOST_BYTES || end > region {
			return None;
		}
		let mut bytes = vec![0u8; len];
		// SAFETY: the view is mapped, committed and readable for `region` bytes from its start (one region, all its
		// pages alike), and `at..end` lies inside it; `len` bytes are copied into a buffer of that length. The game may
		// change them meanwhile, and any byte is a valid `u8`, which the readers check as they read.
		unsafe { std::ptr::copy_nonoverlapping(start.add(at), bytes.as_mut_ptr(), len) };
		Some(bytes)
	};
	let out = if region > 0 { read(&mut copy) } else { None };
	// SAFETY: unmaps the view mapped above, once; nothing reads it after.
	if let Err(e) = unsafe { UnmapViewOfFile(view) } {
		log::warn!("games: a shared memory view could not be unmapped: {e}");
	}
	out
}
