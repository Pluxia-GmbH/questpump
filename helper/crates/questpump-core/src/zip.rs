//! A zip archive read by its central directory (PKWARE's APPNOTE.TXT 6.3.10: 4.3.7 the local file header, 4.3.12 the
//! central directory's records, 4.3.14 and 4.3.15 the Zip64 end and its locator, 4.3.16 the end, 4.5.3 an entry's Zip64
//! sizes): each entry's name, method, sizes, where its local record starts and the mode a Unix-made archive records,
//! and an entry's bytes streamed out of its local record, stored or deflated, with their sha256 taken as they come.
//! Only what the desktop app's downloads need: the move figures' films (`tools/figures/publish.py --zip`, stored) and
//! the addon's release zip (the addon packager's, deflated by Info-ZIP), read whole from a file (`Archive`) or, for the
//! addon, by byte ranges of the zip on its release (`directory` on the zip's last `TAIL` bytes, `entries` on the
//! central directory, `copy_at` on an entry's bytes wherever they were put); an archive that needs encryption or
//! spanning does not read. The central directory, never the local headers alone, names the entries: only it carries
//! the mode that tells a link from a file, and the sizes an entry written to a stream leaves out of its local header.

use std::fmt::Write as _;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom, Write};
use std::path::Path;

use flate2::read::DeflateDecoder;
use sha2::{Digest as _, Sha256};

/// The end of the central directory's, the Zip64 end's and its locator's, a central record's and a local header's
/// first four bytes.
const END: u32 = 0x0605_4b50;
const END64: u32 = 0x0606_4b50;
const LOCATOR: u32 = 0x0706_4b50;
const CENTRAL: u32 = 0x0201_4b50;
const LOCAL: u32 = 0x0403_4b50;
/// The end record's length without its comment, and the longest comment it may carry.
const END_LEN: usize = 22;
const COMMENT_MAX: usize = 0xffff;
/// The Zip64 end's fixed length (without the extensible data no writer here adds) and its locator's.
const END64_LEN: usize = 56;
const LOCATOR_LEN: usize = 20;
/// The bytes at an archive's end that hold its end record whatever its comment, with the Zip64 end and its locator
/// right before it: what `directory` reads.
pub const TAIL: usize = END64_LEN + LOCATOR_LEN + END_LEN + COMMENT_MAX;
/// A central record's and a local header's fixed lengths.
const CENTRAL_LEN: usize = 46;
const LOCAL_LEN: usize = 30;
/// The extra field that carries an entry's Zip64 sizes and offset.
const ZIP64_EXTRA: u16 = 0x0001;
/// The methods read: stored and deflated.
const STORED: u16 = 0;
const DEFLATED: u16 = 8;
/// The flag bit of an encrypted entry.
const ENCRYPTED: u16 = 0x0001;
/// The hosts whose archives keep a Unix mode in the external attributes' high half: Unix (Info-ZIP on Linux and
/// macOS) and OS X.
const UNIX_HOSTS: [u8; 2] = [3, 19];
/// The file type bits of a Unix mode, and a symbolic link's.
const TYPE_BITS: u32 = 0o170_000;
const LINK: u32 = 0o120_000;

/// Why an archive or an entry does not read.
#[derive(Debug, thiserror::Error)]
pub enum ZipError {
	#[error(transparent)]
	Io(#[from] std::io::Error),
	#[error("{0}")]
	Bad(String),
}

fn bad(why: impl Into<String>) -> ZipError {
	ZipError::Bad(why.into())
}

/// One entry, as the central directory records it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
	/// Its name, `/`-separated as the format writes it.
	pub name: String,
	pub method: u16,
	/// Its bytes in the archive and unpacked.
	pub compressed: u64,
	pub size: u64,
	/// Where its local header starts.
	pub offset: u64,
	/// The Unix mode an archive made on Unix records; none from another host.
	pub mode: Option<u32>,
}

impl Entry {
	/// A folder entry: its name ends with `/`.
	#[must_use]
	pub fn is_dir(&self) -> bool {
		self.name.ends_with('/')
	}

	/// A symbolic link, as a Unix-made archive marks one (Info-ZIP's `-y`).
	#[must_use]
	pub fn is_link(&self) -> bool {
		self.mode.is_some_and(|m| m & TYPE_BITS == LINK)
	}

	/// Its local header's least length: the fixed part and its name, before the extra field the header alone sizes.
	#[must_use]
	pub fn header_len(&self) -> u64 {
		(LOCAL_LEN + self.name.len()) as u64
	}
}

/// Where an archive's central directory is, as its end says.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Directory {
	/// Its records.
	pub count: u64,
	/// Where it starts and its length in bytes.
	pub offset: u64,
	pub size: u64,
}

fn u16_at(b: &[u8], at: usize) -> u16 {
	u16::from_le_bytes([b[at], b[at + 1]])
}

fn u32_at(b: &[u8], at: usize) -> u32 {
	u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]])
}

fn u64_at(b: &[u8], at: usize) -> u64 {
	u64::from(u32_at(b, at)) | (u64::from(u32_at(b, at + 4)) << 32)
}

/// The central directory an archive's end names: `tail` is the archive's last bytes (its last `TAIL`, or all of it),
/// starting at `tail_at`. The end record is the last whose comment runs to the archive's end; a Zip64 locator right
/// before it names the Zip64 end, whose counts and places then hold (it must lie in the tail); the directory must end
/// before the first end record.
///
/// # Errors
/// No end record, a spanned archive, a Zip64 end missing or outside the tail, or a directory that runs past its end.
pub fn directory(tail: &[u8], tail_at: u64) -> Result<Directory, ZipError> {
	// before any read at a fixed offset: an empty or cut download holds no end record
	if tail.len() < END_LEN {
		return Err(bad("no end of the central directory"));
	}
	let end = (0..=tail.len() - END_LEN)
		.rev()
		.find(|&at| u32_at(tail, at) == END && at + END_LEN + usize::from(u16_at(tail, at + 20)) == tail.len())
		.ok_or_else(|| bad("no end of the central directory"))?;
	let record = &tail[end..end + END_LEN];
	let (disk, cd_disk) = (u16_at(record, 4), u16_at(record, 6));
	let (on_disk, count) = (u16_at(record, 8), u16_at(record, 10));
	let (cd_size, cd_offset) = (u32_at(record, 12), u32_at(record, 16));
	if disk != 0 || cd_disk != 0 || on_disk != count {
		return Err(bad("a spanned archive"));
	}
	let locator = end.checked_sub(LOCATOR_LEN).filter(|&at| u32_at(tail, at) == LOCATOR);
	let (found, first_end) = match locator {
		Some(at) => {
			// the locator: the disk with the Zip64 end, where that starts, and the disks in all
			if u32_at(tail, at + 4) != 0 || u32_at(tail, at + 16) > 1 {
				return Err(bad("a spanned archive"));
			}
			let wide_at = u64_at(tail, at + 8);
			let wide = wide_at
				.checked_sub(tail_at)
				.and_then(|i| usize::try_from(i).ok())
				.filter(|&i| i + END64_LEN <= at && u32_at(tail, i) == END64)
				.ok_or_else(|| bad("no Zip64 end where its locator says"))?;
			let record = &tail[wide..wide + END64_LEN];
			let (on_disk, count) = (u64_at(record, 24), u64_at(record, 32));
			if u32_at(record, 16) != 0 || u32_at(record, 20) != 0 || on_disk != count {
				return Err(bad("a spanned archive"));
			}
			(Directory { count, size: u64_at(record, 40), offset: u64_at(record, 48) }, wide_at)
		}
		None if count == u16::MAX || cd_size == u32::MAX || cd_offset == u32::MAX => {
			return Err(bad("a Zip64 archive without its Zip64 end"));
		}
		None => (
			Directory { count: u64::from(count), size: u64::from(cd_size), offset: u64::from(cd_offset) },
			tail_at + end as u64,
		),
	};
	if found.offset.checked_add(found.size).is_none_or(|cd_end| cd_end > first_end) {
		return Err(bad("the central directory runs past its end"));
	}
	Ok(found)
}

/// An entry's Zip64 sizes and offset from its extra field (`extra`), each there only where its record's field holds
/// the most it can (`wide`: the size, the compressed size, the offset), in that order.
fn zip64(extra: &[u8], wide: [bool; 3], name: &str) -> Result<[Option<u64>; 3], ZipError> {
	let mut at = 0;
	while at + 4 <= extra.len() {
		let (id, len) = (u16_at(extra, at), usize::from(u16_at(extra, at + 2)));
		let data = extra.get(at + 4..at + 4 + len).ok_or_else(|| bad(format!("{name}: an extra field cut short")))?;
		if id == ZIP64_EXTRA {
			let mut values = [None; 3];
			let mut next = 0;
			for (value, wide) in values.iter_mut().zip(wide) {
				if wide {
					if next + 8 > data.len() {
						return Err(bad(format!("{name}: a Zip64 field cut short")));
					}
					*value = Some(u64_at(data, next));
					next += 8;
				}
			}
			return Ok(values);
		}
		at += 4 + len;
	}
	Err(bad(format!("{name}: a Zip64 entry without its sizes")))
}

/// The entries of a central directory (`cd`, all of its bytes) that `count` records make, in its order.
///
/// # Errors
/// A record cut short or out of place, a name that is not UTF-8, an encrypted entry, or a Zip64 entry without its
/// sizes.
pub fn entries(cd: &[u8], count: u64) -> Result<Vec<Entry>, ZipError> {
	let room = cd.len() / CENTRAL_LEN;
	let mut entries = Vec::with_capacity(usize::try_from(count).map_or(room, |count| count.min(room)));
	let mut at = 0_usize;
	for _ in 0..count {
		let fixed = cd.get(at..at + CENTRAL_LEN).ok_or_else(|| bad("a central record cut short"))?;
		if u32_at(fixed, 0) != CENTRAL {
			return Err(bad("no central record where one should be"));
		}
		let made_by = fixed[5];
		let (flags, method) = (u16_at(fixed, 8), u16_at(fixed, 10));
		let (compressed, size) = (u32_at(fixed, 20), u32_at(fixed, 24));
		let (name_len, extra_len, comment_len) =
			(usize::from(u16_at(fixed, 28)), usize::from(u16_at(fixed, 30)), usize::from(u16_at(fixed, 32)));
		let (external, offset) = (u32_at(fixed, 38), u32_at(fixed, 42));
		let name_at = at + CENTRAL_LEN;
		let name = cd.get(name_at..name_at + name_len).ok_or_else(|| bad("a name cut short"))?;
		let name = String::from_utf8(name.to_vec()).map_err(|_| bad("a name that is not UTF-8"))?;
		if flags & ENCRYPTED != 0 {
			return Err(bad(format!("{name}: encrypted")));
		}
		let extra =
			cd.get(name_at + name_len..name_at + name_len + extra_len).ok_or_else(|| bad("an extra cut short"))?;
		let wide = [size == u32::MAX, compressed == u32::MAX, offset == u32::MAX];
		let [wide_size, wide_compressed, wide_offset] =
			if wide.contains(&true) { zip64(extra, wide, &name)? } else { [None; 3] };
		let mode = UNIX_HOSTS.contains(&made_by).then_some(external >> 16).filter(|m| *m != 0);
		entries.push(Entry {
			name,
			method,
			compressed: wide_compressed.unwrap_or(u64::from(compressed)),
			size: wide_size.unwrap_or(u64::from(size)),
			offset: wide_offset.unwrap_or(u64::from(offset)),
			mode,
		});
		at = name_at + name_len + extra_len + comment_len;
	}
	Ok(entries)
}

/// Writes one entry's bytes into `out`, unpacked, out of `file`, where its local header starts at `at` (its `offset`
/// in the archive, or wherever the archive's bytes from there were put), and answers their sha256 in lower-case hex;
/// never more bytes than the central directory says it holds.
///
/// # Errors
/// Its local header does not match the central record, its method is neither stored nor deflated, it unpacks to
/// another size, or reading or writing fails.
pub fn copy_at(
	file: &mut (impl Read + Seek),
	at: u64,
	entry: &Entry,
	out: &mut impl Write,
) -> Result<String, ZipError> {
	file.seek(SeekFrom::Start(at))?;
	let mut local = [0_u8; LOCAL_LEN];
	file.read_exact(&mut local)?;
	if u32_at(&local, 0) != LOCAL {
		return Err(bad(format!("{}: no local header", entry.name)));
	}
	let (name_len, extra_len) = (u64::from(u16_at(&local, 26)), u64::from(u16_at(&local, 28)));
	let mut name = vec![0_u8; usize::from(u16_at(&local, 26))];
	file.read_exact(&mut name)?;
	if name != entry.name.as_bytes() || u16_at(&local, 8) != entry.method {
		return Err(bad(format!("{}: its local header names another entry", entry.name)));
	}
	file.seek(SeekFrom::Start(at + LOCAL_LEN as u64 + name_len + extra_len))?;
	let packed = file.take(entry.compressed);
	// one byte past the size, so an entry that unpacks to more shows as such
	let mut unpacked: Box<dyn Read + '_> = match entry.method {
		STORED => Box::new(packed.take(entry.size + 1)),
		DEFLATED => Box::new(DeflateDecoder::new(packed).take(entry.size + 1)),
		other => return Err(bad(format!("{}: method {other}", entry.name))),
	};
	let mut hasher = Sha256::new();
	let mut buf = vec![0_u8; 64 << 10];
	let mut got = 0_u64;
	loop {
		let n = unpacked.read(&mut buf)?;
		if n == 0 {
			break;
		}
		got += n as u64;
		if got > entry.size {
			return Err(bad(format!("{}: unpacks to more than its size", entry.name)));
		}
		hasher.update(&buf[..n]);
		out.write_all(&buf[..n])?;
	}
	if got != entry.size {
		return Err(bad(format!("{}: unpacks to {got} of its {} bytes", entry.name, entry.size)));
	}
	Ok(hex(&hasher.finalize()))
}

/// An archive open for reading, its entries read from its central directory.
pub struct Archive {
	file: BufReader<File>,
	entries: Vec<Entry>,
}

impl Archive {
	/// Opens an archive and reads its central directory (`directory`, `entries`).
	///
	/// # Errors
	/// The file does not read, or holds no central directory this reader takes (spanned, cut short, shorter than an
	/// end record).
	pub fn open(path: &Path) -> Result<Self, ZipError> {
		let mut file = BufReader::new(File::open(path)?);
		let len = file.seek(SeekFrom::End(0))?;
		let tail_len = len.min(TAIL as u64);
		file.seek(SeekFrom::Start(len - tail_len))?;
		let mut tail = vec![0_u8; usize::try_from(tail_len).map_err(|_| bad("an archive too large"))?];
		file.read_exact(&mut tail)?;
		let found = directory(&tail, len - tail_len)?;
		file.seek(SeekFrom::Start(found.offset))?;
		let mut cd = vec![0_u8; usize::try_from(found.size).map_err(|_| bad("a central directory too large"))?];
		file.read_exact(&mut cd)?;
		let entries = entries(&cd, found.count)?;
		Ok(Self { file, entries })
	}

	/// The entries in the central directory's order.
	#[must_use]
	pub fn entries(&self) -> &[Entry] {
		&self.entries
	}

	/// Writes one entry's bytes into `out`, unpacked, and answers their sha256 (`copy_at` at its offset).
	///
	/// # Errors
	/// As `copy_at`'s.
	pub fn copy(&mut self, entry: &Entry, out: &mut impl Write) -> Result<String, ZipError> {
		copy_at(&mut self.file, entry.offset, entry, out)
	}
}

/// Bytes in lower-case hex, as the feeds write a sha256.
#[must_use]
pub fn hex(bytes: &[u8]) -> String {
	bytes.iter().fold(String::with_capacity(bytes.len() * 2), |mut out, b| {
		let _ = write!(out, "{b:02x}");
		out
	})
}
