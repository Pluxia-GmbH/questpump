//! A made reel's MP4 in its one shape on both systems (contract/design/tokens.json `surface.SagaReel`: "an MP4 whose
//! moov box comes before its mdat, with no metadata items"): the encoder's file read back and written again as `ftyp`,
//! `moov`, `mdat`, every other top-level box left out (`free`, `skip`, a Windows sink's `uuid`, a `udta` or `meta`),
//! and the `udta` and `meta` boxes inside the movie, its tracks and their media left out too, the movie's, each
//! track's and each media's creation and modification times zeroed (`mvhd`, `tkhd`, `mdhd`: no date in the file), each
//! chunk offset (`stco`, `co64`) moved by what the media data moved: qt-faststart's rule. Both systems' writers put the
//! moov last here (Media Foundation's MPEG-4 sink unless `MF_MPEG4SINK_MOOV_BEFORE_MDAT`, which remuxes the file
//! again; `AVAssetWriter` unless `shouldOptimizeForNetworkUse`, which leaves a temporary file of its own beside it), so
//! the same pass gives both files the same layout.

use anyhow::{Context as _, bail};

/// The containers whose children are read: the movie, a track, its media, its media's information and its samples'
/// table.
const CONTAINERS: [&[u8; 4]; 5] = [b"moov", b"trak", b"mdia", b"minf", b"stbl"];
/// The metadata boxes left out wherever they stand.
const METADATA: [&[u8; 4]; 2] = [b"udta", b"meta"];
/// The headers whose creation and modification times are zeroed.
const DATED: [&[u8; 4]; 3] = [b"mvhd", b"tkhd", b"mdhd"];

/// A box in a byte slice: its type, its whole bytes, where its payload starts within them and where it starts in the
/// slice.
struct Atom<'a> {
	kind: [u8; 4],
	bytes: &'a [u8],
	head: usize,
	at: usize,
}

impl Atom<'_> {
	fn payload(&self) -> &[u8] {
		&self.bytes[self.head..]
	}
}

/// The boxes of `data`, in order.
fn atoms(data: &[u8]) -> anyhow::Result<Vec<Atom<'_>>> {
	let mut out = Vec::new();
	let mut at = 0;
	while at < data.len() {
		let rest = &data[at..];
		let head: [u8; 8] = rest.get(..8).and_then(|h| h.try_into().ok()).context("a box's head is cut short")?;
		let kind: [u8; 4] = [head[4], head[5], head[6], head[7]];
		let short = u32::from_be_bytes([head[0], head[1], head[2], head[3]]);
		let (len, head_len) = match short {
			0 => (rest.len(), 8),
			1 => {
				let large: [u8; 8] = rest.get(8..16).and_then(|l| l.try_into().ok()).context("a large box's size")?;
				(usize::try_from(u64::from_be_bytes(large))?, 16)
			}
			n => (usize::try_from(n)?, 8),
		};
		if len < head_len || len > rest.len() {
			bail!("the box {} says {len} bytes where {} are left", String::from_utf8_lossy(&kind), rest.len());
		}
		out.push(Atom { kind, bytes: &rest[..len], head: head_len, at });
		at += len;
	}
	Ok(out)
}

/// A box of `kind` around `payload`.
fn wrap(kind: [u8; 4], payload: &[u8]) -> anyhow::Result<Vec<u8>> {
	let mut out = Vec::with_capacity(payload.len() + 16);
	if let Ok(len) = u32::try_from(payload.len() + 8) {
		out.extend_from_slice(&len.to_be_bytes());
		out.extend_from_slice(&kind);
	} else {
		// a box past 4 GB: size 1 and its 64-bit size after the type
		out.extend_from_slice(&1_u32.to_be_bytes());
		out.extend_from_slice(&kind);
		out.extend_from_slice(&u64::try_from(payload.len() + 16)?.to_be_bytes());
	}
	out.extend_from_slice(payload);
	Ok(out)
}

/// A container's payload again: its metadata boxes left out, its containers rebuilt, every chunk offset moved by
/// `delta`.
fn rebuild(payload: &[u8], delta: i64) -> anyhow::Result<Vec<u8>> {
	let mut out = Vec::with_capacity(payload.len());
	for atom in atoms(payload)? {
		if METADATA.contains(&&atom.kind) {
			continue;
		}
		if CONTAINERS.contains(&&atom.kind) {
			out.extend(wrap(atom.kind, &rebuild(atom.payload(), delta)?)?);
		} else if &atom.kind == b"stco" || &atom.kind == b"co64" {
			out.extend(wrap(atom.kind, &offsets(atom.payload(), &atom.kind == b"co64", delta)?)?);
		} else if DATED.contains(&&atom.kind) {
			out.extend(wrap(atom.kind, &undated(atom.payload())?)?);
		} else {
			out.extend_from_slice(atom.bytes);
		}
	}
	Ok(out)
}

/// A header's payload with its creation and modification times zeroed: two 32-bit times after its version and flags,
/// or two 64-bit ones in version 1.
fn undated(payload: &[u8]) -> anyhow::Result<Vec<u8>> {
	let mut out = payload.to_vec();
	let times = if payload.first() == Some(&1) { 4..20 } else { 4..12 };
	out.get_mut(times).context("a header is cut short")?.fill(0);
	Ok(out)
}

/// A chunk offset table's payload with each offset moved by `delta` (`wide`: 64-bit offsets).
fn offsets(payload: &[u8], wide: bool, delta: i64) -> anyhow::Result<Vec<u8>> {
	let count = payload.get(4..8).and_then(|c| c.try_into().ok()).map(u32::from_be_bytes).context("a chunk table")?;
	let width = if wide { 8 } else { 4 };
	let mut out = payload.get(..8).context("a chunk table")?.to_vec();
	let entries = payload.get(8..8 + usize::try_from(count)? * width).context("a chunk table is cut short")?;
	for entry in entries.chunks_exact(width) {
		let old = if wide {
			i64::try_from(u64::from_be_bytes(entry.try_into()?))?
		} else {
			i64::from(u32::from_be_bytes(entry.try_into()?))
		};
		let moved = old + delta;
		if wide {
			out.extend_from_slice(&u64::try_from(moved)?.to_be_bytes());
		} else {
			out.extend_from_slice(&u32::try_from(moved).context("a chunk moved past 4 GB")?.to_be_bytes());
		}
	}
	Ok(out)
}

/// The encoder's MP4 (`data`) as `ftyp`, `moov`, `mdat` with no metadata (the module's rule).
///
/// # Errors
/// The file has no `ftyp`, `moov` or one `mdat`, or a box does not read.
pub(super) fn faststart(data: &[u8]) -> anyhow::Result<Vec<u8>> {
	let top = atoms(data)?;
	let find = |kind: &[u8; 4]| top.iter().filter(|a| &a.kind == kind).collect::<Vec<_>>();
	let (ftyp, moov, mdat) = (find(b"ftyp"), find(b"moov"), find(b"mdat"));
	let ([ftyp], [moov], [mdat]) = (ftyp.as_slice(), moov.as_slice(), mdat.as_slice()) else {
		bail!("an MP4 with {} ftyp, {} moov and {} mdat boxes", ftyp.len(), moov.len(), mdat.len());
	};
	let old_start = mdat.at + mdat.head;
	// the movie's size does not hang on its offsets' values, so lay it out once to place the media data
	let sized = wrap(*b"moov", &rebuild(moov.payload(), 0)?)?;
	let media = wrap(*b"mdat", mdat.payload())?;
	let new_start = ftyp.bytes.len() + sized.len() + (media.len() - mdat.payload().len());
	let delta = i64::try_from(new_start)? - i64::try_from(old_start)?;
	let moved = wrap(*b"moov", &rebuild(moov.payload(), delta)?)?;
	let mut out = Vec::with_capacity(ftyp.bytes.len() + moved.len() + media.len());
	out.extend_from_slice(ftyp.bytes);
	out.extend(moved);
	out.extend(media);
	Ok(out)
}
