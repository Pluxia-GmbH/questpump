//! A zip archive read by its central directory (PKWARE's APPNOTE.TXT 6.3.10: 4.3.7 the local file header, 4.3.12 the
//! central directory's records, 4.3.16 its end): each entry's name, method, sizes and the mode a Unix-made archive
//! records, and an entry's bytes streamed out of its local record, stored or deflated, with their sha256 taken as they
//! come. Only what the desktop app's two downloads need: the move figures' films (`tools/figures/publish.py --zip`,
//! stored) and the addon's release zip (the addon packager's, deflated by Info-ZIP); an archive that needs Zip64,
//! encryption or spanning does not read. The central directory, never the local headers alone, names the entries: only
//! it carries the mode that tells a link from a file, and the sizes an entry written to a stream leaves out of its
//! local header.

use std::fmt::Write as _;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom, Write};
use std::path::Path;

use flate2::read::DeflateDecoder;
use sha2::{Digest as _, Sha256};

/// The end of the central directory's, a central record's and a local header's first four bytes.
const END: u32 = 0x0605_4b50;
const CENTRAL: u32 = 0x0201_4b50;
const LOCAL: u32 = 0x0403_4b50;
/// The end record's length without its comment, and the longest comment it may carry.
const END_LEN: usize = 22;
const COMMENT_MAX: usize = 0xffff;
/// A central record's and a local header's fixed lengths.
const CENTRAL_LEN: usize = 46;
const LOCAL_LEN: usize = 30;
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
	offset: u64,
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
}

/// An archive open for reading, its entries read from its central directory.
pub struct Archive {
	file: BufReader<File>,
	entries: Vec<Entry>,
}

fn u16_at(b: &[u8], at: usize) -> u16 {
	u16::from_le_bytes([b[at], b[at + 1]])
}

fn u32_at(b: &[u8], at: usize) -> u32 {
	u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]])
}

impl Archive {
	/// Opens an archive and reads its central directory.
	///
	/// # Errors
	/// The file does not read, or holds no central directory this reader takes (Zip64, spanned, cut short, shorter than
	/// an end record).
	pub fn open(path: &Path) -> Result<Self, ZipError> {
		let mut file = BufReader::new(File::open(path)?);
		let len = file.seek(SeekFrom::End(0))?;
		// before any read at a fixed offset: an empty or cut download holds no end record
		if len < END_LEN as u64 {
			return Err(bad("no end of the central directory"));
		}
		let tail_len = len.min((END_LEN + COMMENT_MAX) as u64);
		file.seek(SeekFrom::Start(len - tail_len))?;
		let mut tail = vec![0_u8; usize::try_from(tail_len).map_err(|_| bad("an archive too large"))?];
		file.read_exact(&mut tail)?;
		// the end record is the last one whose comment runs to the file's end
		let end = (0..=tail.len().saturating_sub(END_LEN))
			.rev()
			.find(|&at| u32_at(&tail, at) == END && at + END_LEN + usize::from(u16_at(&tail, at + 20)) == tail.len())
			.ok_or_else(|| bad("no end of the central directory"))?;
		let record = &tail[end..end + END_LEN];
		let (disk, cd_disk) = (u16_at(record, 4), u16_at(record, 6));
		let (on_disk, count) = (u16_at(record, 8), u16_at(record, 10));
		let (cd_size, cd_offset) = (u32_at(record, 12), u32_at(record, 16));
		if disk != 0 || cd_disk != 0 || on_disk != count {
			return Err(bad("a spanned archive"));
		}
		if count == u16::MAX || cd_size == u32::MAX || cd_offset == u32::MAX {
			return Err(bad("a Zip64 archive"));
		}
		if u64::from(cd_offset) + u64::from(cd_size) > len {
			return Err(bad("the central directory runs past the file"));
		}
		file.seek(SeekFrom::Start(u64::from(cd_offset)))?;
		let mut cd = vec![0_u8; cd_size as usize];
		file.read_exact(&mut cd)?;
		let mut entries = Vec::with_capacity(usize::from(count));
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
			let name = cd.get(at + CENTRAL_LEN..at + CENTRAL_LEN + name_len).ok_or_else(|| bad("a name cut short"))?;
			let name = String::from_utf8(name.to_vec()).map_err(|_| bad("a name that is not UTF-8"))?;
			if flags & ENCRYPTED != 0 {
				return Err(bad(format!("{name}: encrypted")));
			}
			if compressed == u32::MAX || size == u32::MAX || offset == u32::MAX {
				return Err(bad(format!("{name}: a Zip64 entry")));
			}
			let mode = UNIX_HOSTS.contains(&made_by).then_some(external >> 16).filter(|m| *m != 0);
			entries.push(Entry {
				name,
				method,
				compressed: u64::from(compressed),
				size: u64::from(size),
				offset: u64::from(offset),
				mode,
			});
			at += CENTRAL_LEN + name_len + extra_len + comment_len;
		}
		Ok(Self { file, entries })
	}

	/// The entries in the central directory's order.
	#[must_use]
	pub fn entries(&self) -> &[Entry] {
		&self.entries
	}

	/// Writes one entry's bytes into `out`, unpacked, and answers their sha256 in lower-case hex; never more bytes than
	/// the central directory says it holds.
	///
	/// # Errors
	/// Its local header does not match the central record, its method is neither stored nor deflated, it unpacks to
	/// another size, or reading or writing fails.
	pub fn copy(&mut self, entry: &Entry, out: &mut impl Write) -> Result<String, ZipError> {
		self.file.seek(SeekFrom::Start(entry.offset))?;
		let mut local = [0_u8; LOCAL_LEN];
		self.file.read_exact(&mut local)?;
		if u32_at(&local, 0) != LOCAL {
			return Err(bad(format!("{}: no local header", entry.name)));
		}
		let (name_len, extra_len) = (u64::from(u16_at(&local, 26)), u64::from(u16_at(&local, 28)));
		let mut name = vec![0_u8; usize::from(u16_at(&local, 26))];
		self.file.read_exact(&mut name)?;
		if name != entry.name.as_bytes() || u16_at(&local, 8) != entry.method {
			return Err(bad(format!("{}: its local header names another entry", entry.name)));
		}
		self.file.seek(SeekFrom::Start(entry.offset + LOCAL_LEN as u64 + name_len + extra_len))?;
		let packed = (&mut self.file).take(entry.compressed);
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
}

/// Bytes in lower-case hex, as the feeds write a sha256.
#[must_use]
pub fn hex(bytes: &[u8]) -> String {
	bytes.iter().fold(String::with_capacity(bytes.len() * 2), |mut out, b| {
		let _ = write!(out, "{b:02x}");
		out
	})
}
