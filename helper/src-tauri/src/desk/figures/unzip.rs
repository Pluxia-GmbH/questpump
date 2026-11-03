//! The films' download unpacked (`desk::figures`): `figures-<films_digest>.zip` as `tools/figures/publish.py --zip`
//! writes it, read by questpump-core's `zip` (the addon's release zip is read by the same reader), each entry stored
//! under its name in figures.json's `files`, checked against its sha256 there as it is written.

use std::collections::{BTreeMap, BTreeSet};
use std::io::{BufWriter, Write as _};
use std::path::Path;

use anyhow::Context as _;
use questpump_core::zip::Archive;

/// Unpacks the zip into `into` (made anew): only the films `files` names (`films/<owner>.mp4` with its sha256), each
/// once, written while its sha256 is taken and refused when it differs; every film must be there.
///
/// # Errors
/// The zip does not read, holds another file, a film twice or not at all, or a film whose sha256 is not the table's;
/// or a file cannot be written.
pub(super) fn unpack(zip: &Path, into: &Path, files: &BTreeMap<String, String>) -> anyhow::Result<()> {
	if into.exists() {
		std::fs::remove_dir_all(into)?;
	}
	std::fs::create_dir_all(into.join("films"))?;
	let mut archive = Archive::open(zip)?;
	let mut seen = BTreeSet::new();
	for entry in archive.entries().to_vec() {
		// only the table's films: a name from the table is a safe path
		let want = files
			.get(&entry.name)
			.filter(|_| entry.name.starts_with("films/") && !entry.is_link())
			.with_context(|| format!("{}: not a film of the table", entry.name))?;
		anyhow::ensure!(seen.insert(entry.name.clone()), "{}: twice in the zip", entry.name);
		let path = into.join(&entry.name);
		let mut out = BufWriter::new(std::fs::File::create(&path).with_context(|| path.display().to_string())?);
		let got = archive.copy(&entry, &mut out)?;
		out.flush()?;
		anyhow::ensure!(&got == want, "{}: its sha256 is not the table's", entry.name);
	}
	let wanted = files.keys().filter(|name| name.starts_with("films/")).count();
	anyhow::ensure!(seen.len() == wanted, "{} of the {wanted} films", seen.len());
	Ok(())
}
