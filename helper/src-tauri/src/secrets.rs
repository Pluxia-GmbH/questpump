//! The desktop app's secrets, kept out of its files (the release security review of 2026-10-31): the pairing key
//! (`Settings::key`), the sync's R and this computer's device token in the account (`Account::r`, `Account::token`), a
//! rekey's new R and this computer's own key's secret (`Account::next_r`, `Account::device_secret`, since 2026-11-01),
//! with the install's id beside the key (`Settings::hid`: no secret, and in the file too, but a phone knows its helper
//! by both, so a settings.json that is gone keeps the pairing). On macOS they are one generic password item of the
//! login keychain, service `ai.pluxia.questpump.bridge` and account `secrets`, which only a build signed as QuestPump's
//! release reads or writes (its designated requirement, `RELEASE`): the item's access list trusts the build that made
//! it by that requirement, so every later release reads it without a prompt, and a `cargo run` or test binary (another
//! signature) never makes one the release would be asked about. On Windows they are a DPAPI blob for the player's
//! account (`CryptProtectData`, no prompt) in the data folder, `secrets.dpapi`, which the uninstaller keeps with the
//! rest of the folder.
//!
//! The rest of the code keeps them in `Settings` as before and saves it as before: `Keeper` alone stands between the
//! settings and their file. At the start (`Keeper::open`) the store is read, and what it holds is laid into
//! settings.json's JSON before that is read (an empty object for a file that is gone or did not read): each value the
//! file holds itself wins (a build before this one wrote it there, or a run whose store refused), each it lacks comes
//! from the store, the token and this computer's own key only beside the R they were kept with and the account and
//! device the file names (R alone joins its account again, and R' goes with the R it was kept with). A file that holds
//! them moves them into the store at the first save (`runtime::sweep` saves at once). Before a save replaces or drops a
//! value the store held and the start did not take, the store's whole set is kept aside once (the item
//! `secrets.before`, on Windows `secrets.dpapi.before`), so a run of an older build or a file that lost a field never
//! costs the stored ones for good; a set that cannot be kept aside is not replaced. Every save (`Keeper::bytes`, on the
//! writer thread, or at once in the start's sweep) writes them into the store when they changed and leaves them out of
//! the file once the store holds them. A store that refuses a write keeps them in settings.json for the rest of the
//! run, as before this build, with one warning; the next start moves them again.
//!
//! A store that cannot be read at the start loses nothing. While the file holds secrets of its own, that run keeps them
//! in the file. While it holds none (a keychain the player kept locked; a build not signed as the release while the
//! release's item is there, which it never replaces), the run neither pairs nor syncs (`RunSecrets::Unread`: no pairing
//! code, no LAN or Bluetooth hello, Sync refused), writes no secret anywhere and keeps the file's account block and
//! install id as the start read them, so the next start reads the store again. A build not signed as the release with
//! no item there keeps them in settings.json, as every build did before. A store whose secrets can never be read again
//! (a DPAPI blob of another account or machine, a damaged item) starts every pairing again, as a pairing key that did
//! not read in settings.json did.

use std::path::{Path, PathBuf};

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use questpump_core::mirror;

use crate::settings;

/// settings.json's fields of the pairing key and the install's id, and of the sync's account, which holds R, the
/// token, a rekey's R' and this computer's own key's secret.
const KEY: &str = "key";
const HID: &str = "hid";
const SYNC: &str = "sync";
const R: &str = "r";
const TOKEN: &str = "token";
const NEXT_R: &str = "next_r";
const DEVICE_SECRET: &str = "device_secret";
/// The account's secrets, each in its block.
const ACCOUNT_SECRETS: [&str; 4] = [R, TOKEN, NEXT_R, DEVICE_SECRET];
/// The account's fields a token belongs with: its account and this computer's device in it.
const NAMES: [&str; 2] = ["account", "device"];
/// The account's Sync switch, off for a run whose store did not read.
const ON: &str = "on";

/// The secrets as settings.json's JSON holds them, with the install's id: the pairing key's base64url, the id's, R's
/// base64url, the token, R''s base64url and this computer's own key's secret's base64url. None is absent from the
/// file (or `null` there); a store an earlier build wrote lacks the last two.
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
struct Secrets {
	#[serde(default, skip_serializing_if = "Option::is_none")]
	key: Option<Value>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	hid: Option<Value>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	r: Option<Value>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	token: Option<Value>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	next_r: Option<Value>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	device_secret: Option<Value>,
}

impl Secrets {
	/// The secrets the settings' JSON holds, left in it.
	fn of(file: &Value) -> Self {
		let field = |value: Option<&Value>| value.filter(|v| !v.is_null()).cloned();
		let sync = |name: &str| field(file.get(SYNC).and_then(|s| s.get(name)));
		Self {
			key: field(file.get(KEY)),
			hid: field(file.get(HID)),
			r: sync(R),
			token: sync(TOKEN),
			next_r: sync(NEXT_R),
			device_secret: sync(DEVICE_SECRET),
		}
	}

	/// The account's secrets with their fields in its block.
	fn account(&self) -> [(&'static str, &Option<Value>); 4] {
		[(R, &self.r), (TOKEN, &self.token), (NEXT_R, &self.next_r), (DEVICE_SECRET, &self.device_secret)]
	}

	/// The secrets the settings' JSON holds, taken out of it (the install's id stays: it is no secret).
	fn take(file: &mut Value) -> Self {
		let secrets = Self::of(file);
		if let Some(object) = file.as_object_mut() {
			object.remove(KEY);
		}
		if let Some(sync) = file.get_mut(SYNC).and_then(Value::as_object_mut) {
			for name in ACCOUNT_SECRETS {
				sync.remove(name);
			}
		}
		secrets
	}

	/// Laid into the settings' JSON, where `of` reads them (an account block made for an account's secret).
	fn put(&self, file: &mut Value) {
		let Some(object) = file.as_object_mut() else { return };
		for (name, value) in [(KEY, &self.key), (HID, &self.hid)] {
			if let Some(value) = value {
				object.insert(name.to_owned(), value.clone());
			}
		}
		let account = self.account();
		if account.iter().all(|(_, value)| value.is_none()) {
			return;
		}
		let sync = object.entry(SYNC).or_insert_with(|| Value::Object(Map::new()));
		if let Some(sync) = sync.as_object_mut() {
			for (name, value) in account {
				if let Some(value) = value {
					sync.insert(name.to_owned(), value.clone());
				}
			}
		}
	}

	/// Whether these hold a secret (the install's id is none).
	fn secret(&self) -> bool {
		self.key.is_some() || self.account().iter().any(|(_, value)| value.is_some())
	}

	/// The file's own (`self`) over the store's: each value the file holds wins, each it lacks comes from the store,
	/// R' beside the R it was kept with, whatever else the file names (a rekey's R' lost would be an account lost when
	/// its 204 was, and one beside another R belongs to a rekey the account settled or to another account), the token
	/// and this computer's own key only beside the R they were kept with and the account and device the file names (a
	/// token of an account the file does not name stalls the sync, where R alone joins its account again; a key with
	/// none joins with a new one). An older build's file is not told apart from this build's: its R and token go with
	/// the account, device, cursor and seq only the file holds, so they win, and the store's are kept aside
	/// (`Keeper::keep_aside`).
	fn over(&self, stored: &Self, file: &Value) -> Self {
		let r = self.r.clone().or_else(|| stored.r.clone());
		let named = NAMES.iter().all(|name| file.get(SYNC).and_then(|s| s.get(name)).is_some_and(|v| !v.is_null()));
		let beside = |own: &Option<Value>, kept: &Option<Value>| {
			own.clone().or_else(|| kept.clone().filter(|_| named && r == stored.r))
		};
		Self {
			key: self.key.clone().or_else(|| stored.key.clone()),
			hid: self.hid.clone().or_else(|| stored.hid.clone()),
			token: beside(&self.token, &stored.token),
			next_r: self.next_r.clone().or_else(|| stored.next_r.clone().filter(|_| r == stored.r)),
			device_secret: beside(&self.device_secret, &stored.device_secret),
			r,
		}
	}

	/// Whether `laid` replaces or drops a value these held.
	fn displaced_by(&self, laid: &Self) -> bool {
		let account = self.account().into_iter().zip(laid.account()).map(|((_, held), (_, now))| (held, now));
		[(&self.key, &laid.key), (&self.hid, &laid.hid)]
			.into_iter()
			.chain(account)
			.any(|(held, now)| held.is_some() && held != now)
	}
}

/// What the store gave at the start.
enum Read {
	/// It holds none: a first start, or a store the player emptied.
	Absent,
	Held(Vec<u8>),
	/// It holds them, but no start ever reads them again (a DPAPI blob of another account or machine).
	#[cfg_attr(
		not(windows),
		// copy-lint: not shown (a lint's reason)
		expect(dead_code, reason = "a Keychain item reads or is refused; only a DPAPI blob is lost")
	)]
	Lost(String),
	/// It could not be read now (a locked keychain the player kept locked, a build not signed as the release while the
	/// release's item is there): the next start tries again.
	Refused(String),
}

/// Where the secrets stand this run.
enum State {
	/// The store holds these.
	Stored(Secrets),
	/// The store holds none this run knows: the first save puts them there.
	New,
	/// settings.json keeps them, as before this build, until a start moves them: the store refused a write, or a read
	/// while the file held them, or this build has none.
	InFile,
	/// The store could not be read at the start and the file held no secret: this run's pairing key is its own and goes
	/// nowhere, and the file keeps the account block and the install's id as the start read them (none: it had none).
	Unread { sync: Option<Value>, hid: Option<Value> },
}

/// Whether this run's secrets reach the next start (`Keeper::run`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RunSecrets {
	/// The store's or the file's: what this run pairs and joins with stays.
	Kept,
	/// The store could not be read at the start and the file held no secret: this run's pairing key and anything it
	/// joined would be gone at the next start, so it neither pairs nor syncs.
	Unread,
}

/// The one owner of the secrets' store, between the settings and settings.json: the loop's writer thread holds it
/// (the start's sweep before that thread runs).
pub(crate) struct Keeper {
	dir: PathBuf,
	state: State,
	/// settings.json held secrets at the start: the start saves at once, and its cut writes go once the store has them.
	in_file: bool,
	/// The store's set as the start found it, when the start did not take it whole: kept aside before the first write
	/// replaces it.
	aside: Option<Secrets>,
}

impl Keeper {
	/// The start, with settings.json's JSON (an empty object for a file that is gone or did not read): the store read
	/// and what it holds laid in under the file's own values (`Secrets::over`); while the store cannot be read now and
	/// the file holds no secret, a pairing key of this run alone and the account switched off for this run, its block
	/// and the install's id kept as the start read them.
	pub(crate) fn open(dir: &Path, file: &mut Value) -> Self {
		let dir = dir.to_path_buf();
		let theirs = Secrets::of(file);
		let read = match store::usable() {
			Ok(()) => store::load(&dir),
			// the release's item, which this build must never replace: as a store that refused
			Err(why) if !theirs.secret() && store::held_elsewhere() => Read::Refused(why),
			Err(why) => {
				log::info!("secrets: {} keeps them ({why})", settings::FILE);
				return Self::at(dir, State::InFile);
			}
		};
		let stored = match read {
			Read::Held(bytes) => match serde_json::from_slice::<Secrets>(&bytes) {
				Ok(stored) => Some(stored),
				Err(e) => {
					log::warn!("secrets: {} holds none that read ({e})", store::NAME);
					None
				}
			},
			Read::Lost(why) => {
				log::warn!("secrets: {} holds none that read ({why})", store::NAME);
				None
			}
			Read::Absent => None,
			Read::Refused(why) if theirs.secret() => {
				log::warn!("secrets: {} did not read ({why}): {} keeps them this run", store::NAME, settings::FILE);
				return Self::at(dir, State::InFile);
			}
			Read::Refused(why) => {
				log::warn!(
					"secrets: {} did not read ({why}): this run neither pairs nor syncs, {}",
					store::NAME,
					"and the next start reads it again", // copy-lint: not shown (a log line's words)
				);
				let state = State::Unread { sync: file.get(SYNC).cloned(), hid: file.get(HID).cloned() };
				if let Some(object) = file.as_object_mut() {
					let account = object.entry(SYNC).or_insert_with(|| Value::Object(Map::new()));
					if let Some(account) = account.as_object_mut() {
						account.insert(ON.to_owned(), Value::Bool(false));
					}
				}
				Secrets { key: Some(run_key()), ..Secrets::default() }.put(file);
				return Self::at(dir, state);
			}
		};
		let laid = theirs.over(stored.as_ref().unwrap_or(&Secrets::default()), file);
		laid.put(file);
		let in_file = theirs.secret();
		if in_file {
			log::info!("secrets: {} holds them: they move into {}", settings::FILE, store::NAME);
		}
		let aside = stored.clone().filter(|stored| stored.displaced_by(&laid));
		Self { dir, state: stored.map_or(State::New, State::Stored), in_file, aside }
	}

	fn at(dir: PathBuf, state: State) -> Self {
		Self { dir, state, in_file: false, aside: None }
	}

	/// The file still holds secrets the store should keep: the start saves at once.
	pub(crate) fn moving(&self) -> bool {
		self.in_file && matches!(self.state, State::Stored(_) | State::New)
	}

	/// Whether this run's secrets reach the next start: not while the store could not be read at this start and the
	/// file held none.
	pub(crate) fn run(&self) -> RunSecrets {
		if matches!(self.state, State::Unread { .. }) { RunSecrets::Unread } else { RunSecrets::Kept }
	}

	/// settings.json's bytes for the settings' JSON `file`, every save's: the secrets written into the store when it
	/// does not hold them yet (the store's set the start did not take kept aside first), and out of the file once it
	/// does; in the file while the store refuses (one warning a run); nowhere while the store did not read, the start's
	/// account block and install id in place of this run's.
	///
	/// # Errors
	/// A value that does not write as JSON.
	pub(crate) fn bytes(&mut self, mut file: Value) -> serde_json::Result<Vec<u8>> {
		self.keep(&mut file);
		serde_json::to_vec_pretty(&file)
	}

	fn keep(&mut self, file: &mut Value) {
		let secrets = Secrets::take(file);
		match &self.state {
			State::Unread { sync, hid } => {
				if let Some(object) = file.as_object_mut() {
					for (name, start) in [(SYNC, sync), (HID, hid)] {
						match start {
							Some(value) => object.insert(name.to_owned(), value.clone()),
							None => object.remove(name),
						};
					}
				}
				return;
			}
			State::InFile => {
				secrets.put(file);
				return;
			}
			State::Stored(held) if *held == secrets => {
				self.out_of_file();
				return;
			}
			State::Stored(_) | State::New => {}
		}
		if let Err(why) = self.keep_aside() {
			log::warn!(
				"secrets: {} could not keep its set aside ({why}): it stays as it was, {} {}",
				store::NAME,
				settings::FILE,
				"keeps them until the next start", // copy-lint: not shown (a log line's words)
			);
			secrets.put(file);
			self.state = State::InFile;
			return;
		}
		let saved = serde_json::to_vec(&secrets).map_err(|e| e.to_string()).and_then(|b| store::save(&self.dir, &b));
		match saved {
			Ok(()) => {
				self.out_of_file();
				self.state = State::Stored(secrets);
			}
			Err(why) => {
				log::warn!(
					"secrets: {} refused them ({why}): {} keeps them until the next start",
					store::NAME,
					settings::FILE
				);
				secrets.put(file);
				self.state = State::InFile;
			}
		}
	}

	/// The store's set the start did not take whole, kept aside once before the first write replaces it (one line).
	fn keep_aside(&mut self) -> Result<(), String> {
		let Some(before) = &self.aside else { return Ok(()) };
		serde_json::to_vec(before).map_err(|e| e.to_string()).and_then(|b| store::save_before(&self.dir, &b))?;
		log::warn!("secrets: {} held others than this start took: kept aside as {}", store::NAME, store::BEFORE);
		self.aside = None;
		Ok(())
	}

	/// The store holds what the file held at the start: the file goes without them from this save on, and so do the
	/// writes of it a crash or a power cut left.
	fn out_of_file(&mut self) {
		if std::mem::take(&mut self.in_file) {
			log::info!("secrets: in {} now, out of {}", store::NAME, settings::FILE);
			remove_cut_writes(&self.dir);
		}
	}
}

/// The writes of settings.json a crash or a power cut left before their rename (`mirror::is_temp_of`), which held the
/// secrets as the file did.
fn remove_cut_writes(dir: &Path) {
	let Ok(listed) = std::fs::read_dir(dir) else { return };
	let cut =
		listed.flatten().filter(|e| e.file_name().to_str().is_some_and(|n| mirror::is_temp_of(n, settings::FILE)));
	for entry in cut {
		if let Err(e) = std::fs::remove_file(entry.path()) {
			log::warn!("{}: {e}", entry.path().display());
		}
	}
}

/// A pairing key for a run whose store did not read, never kept: the stored one comes back at the next start.
fn run_key() -> Value {
	let mut key = [0u8; 32];
	rand::fill(&mut key);
	Value::String(URL_SAFE_NO_PAD.encode(key))
}

/// The login keychain: one generic password item, written without a dialog (a locked keychain refuses the write,
/// and settings.json keeps the secrets that run), read with the system's own unlock prompt should the player have
/// kept the keychain locked (the login keychain is unlocked while the player is logged in, and nothing else asks).
#[cfg(target_os = "macos")]
mod store {
	use std::path::Path;

	use security_framework::os::macos::code_signing::{Flags, SecCode, SecRequirement};
	use security_framework::os::macos::keychain::SecKeychain;
	use security_framework::passwords::{self, PasswordOptions};

	use super::Read;

	pub(super) const NAME: &str = "the Keychain"; // copy-lint: not shown (the log lines' name of the store)
	/// The item's service, the bundle id, and its account.
	const SERVICE: &str = "ai.pluxia.questpump.bridge";
	const ACCOUNT: &str = "secrets";
	/// The account of the item that keeps the set a start replaced (`Keeper::keep_aside`).
	pub(super) const BEFORE: &str = "secrets.before";
	/// QuestPump's release as `codesign -d -r-` prints its designated requirement (helper/README.md, "Signing"): the
	/// bundle id, a Developer ID Application certificate, Pluxia's team.
	// copy-lint: not shown (a code requirement)
	const RELEASE: &str = "identifier \"ai.pluxia.questpump.bridge\" and anchor apple generic and certificate \
	                       1[field.1.2.840.113635.100.6.2.6] and certificate leaf[field.1.2.840.113635.100.6.1.13] and \
	                       certificate leaf[subject.OU] = PQFUQJH377";
	/// `errSecItemNotFound` (Security's `SecBase.h`).
	const NOT_FOUND: i32 = -25_300;

	/// Whether this process is QuestPump's release (`SecCodeCheckValidity` of its own code against `RELEASE`): only it
	/// keeps the secrets in the Keychain.
	pub(super) fn usable() -> Result<(), String> {
		let release: SecRequirement = RELEASE.parse().map_err(|e| format!("{e}"))?;
		SecCode::for_self(Flags::NONE)
			.and_then(|code| code.check_validity(Flags::NONE, &release))
			.map_err(|e| format!("not signed as QuestPump's release: {e}"))
	}

	pub(super) fn load(_dir: &Path) -> Read {
		match passwords::generic_password(PasswordOptions::new_generic_password(SERVICE, ACCOUNT)) {
			Ok(bytes) => Read::Held(bytes),
			Err(e) if e.code() == NOT_FOUND => Read::Absent,
			Err(e) => Read::Refused(e.to_string()),
		}
	}

	/// Whether the item is there, for a build not signed as the release, asked with no dialog (the release's item then
	/// answers with a refusal, and no item with `errSecItemNotFound`): an item this build must never replace.
	pub(super) fn held_elsewhere() -> bool {
		let Ok(_quiet) = SecKeychain::disable_user_interaction() else { return true };
		let found = passwords::generic_password(PasswordOptions::new_generic_password(SERVICE, ACCOUNT));
		!matches!(found, Err(e) if e.code() == NOT_FOUND)
	}

	pub(super) fn save(_dir: &Path, bytes: &[u8]) -> Result<(), String> {
		write(ACCOUNT, bytes)
	}

	pub(super) fn save_before(_dir: &Path, bytes: &[u8]) -> Result<(), String> {
		write(BEFORE, bytes)
	}

	fn write(account: &str, bytes: &[u8]) -> Result<(), String> {
		// no dialog while it writes (the user interaction comes back when `_quiet` drops)
		let _quiet = SecKeychain::disable_user_interaction().map_err(|e| e.to_string())?;
		passwords::set_generic_password_options(bytes, PasswordOptions::new_generic_password(SERVICE, account))
			.map_err(|e| e.to_string())
	}
}

/// DPAPI for the player's account: the blob in the data folder, written as every data file is
/// (`mirror::write_atomic`), sealed with no prompt and an entropy of QuestPump's own.
#[cfg(windows)]
mod store {
	use std::path::Path;

	use questpump_core::mirror;
	use windows::Win32::Foundation::HLOCAL;
	use windows::Win32::Security::Cryptography::{
		CRYPT_INTEGER_BLOB, CRYPTPROTECT_UI_FORBIDDEN, CryptProtectData, CryptUnprotectData,
	};
	use windows::core::{Owned, w};

	use super::Read;

	pub(super) const NAME: &str = "Windows' data protection"; // copy-lint: not shown (the log lines' name of the store)
	const FILE: &str = "secrets.dpapi";
	/// The blob that keeps the set a start replaced (`Keeper::keep_aside`).
	pub(super) const BEFORE: &str = "secrets.dpapi.before";
	/// The entropy every blob is sealed with: another program of the player's that unseals it must know it.
	const ENTROPY: &[u8] = b"questpump secrets v1";

	/// DPAPI is every build's on Windows: a blob any of the player's own programs may unseal.
	#[expect(clippy::unnecessary_wraps, reason = "the same signature as the Keychain's, which checks the signature")]
	pub(super) fn usable() -> Result<(), String> {
		Ok(())
	}

	pub(super) fn load(dir: &Path) -> Read {
		let blob = match std::fs::read(dir.join(FILE)) {
			Ok(blob) => blob,
			Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Read::Absent,
			Err(e) => return Read::Refused(e.to_string()),
		};
		match unseal(&blob) {
			Ok(plain) => Read::Held(plain),
			Err(e) => {
				// never unsealed again: kept aside, as a data file that did not read is
				let aside = mirror::keep_aside(dir, FILE).map_or_else(
					|e| format!("{FILE}.unread: {e}"),
					|()| {
						format!("kept aside as {FILE}.unread") // copy-lint: not shown (a log line's words)
					},
				);
				Read::Lost(format!("{e} ({aside})"))
			}
		}
	}

	/// Never asked: every build keeps them in DPAPI (`usable`).
	pub(super) fn held_elsewhere() -> bool {
		false
	}

	pub(super) fn save(dir: &Path, bytes: &[u8]) -> Result<(), String> {
		write(dir, FILE, bytes)
	}

	pub(super) fn save_before(dir: &Path, bytes: &[u8]) -> Result<(), String> {
		write(dir, BEFORE, bytes)
	}

	fn write(dir: &Path, name: &str, bytes: &[u8]) -> Result<(), String> {
		let blob = seal(bytes)?;
		mirror::write_atomic(dir, name, &blob).map_err(|e| e.to_string())
	}

	/// A blob of `bytes` for a call that only reads it.
	fn blob(bytes: &[u8]) -> Result<CRYPT_INTEGER_BLOB, String> {
		let len = u32::try_from(bytes.len()).map_err(|e| e.to_string())?;
		Ok(CRYPT_INTEGER_BLOB { cbData: len, pbData: bytes.as_ptr().cast_mut() })
	}

	fn seal(plain: &[u8]) -> Result<Vec<u8>, String> {
		let (input, entropy) = (blob(plain)?, blob(ENTROPY)?);
		let mut out = CRYPT_INTEGER_BLOB::default();
		// SAFETY: both blobs point at slices that outlive the call, which only reads them; `out` receives memory the
		// call allocated, which `owned` copies and frees.
		unsafe {
			CryptProtectData(
				&raw const input,
				w!("QuestPump"),
				Some(&raw const entropy),
				None,
				None,
				CRYPTPROTECT_UI_FORBIDDEN,
				&raw mut out,
			)
		}
		.map_err(|e| e.to_string())?;
		Ok(owned(out))
	}

	fn unseal(sealed: &[u8]) -> Result<Vec<u8>, String> {
		let (input, entropy) = (blob(sealed)?, blob(ENTROPY)?);
		let mut out = CRYPT_INTEGER_BLOB::default();
		// SAFETY: as in `seal`; no description is asked for.
		unsafe {
			CryptUnprotectData(
				&raw const input,
				None,
				Some(&raw const entropy),
				None,
				None,
				CRYPTPROTECT_UI_FORBIDDEN,
				&raw mut out,
			)
		}
		.map_err(|e| e.to_string())?;
		Ok(owned(out))
	}

	/// The bytes a DPAPI call allocated with `LocalAlloc`, copied, then freed once.
	fn owned(out: CRYPT_INTEGER_BLOB) -> Vec<u8> {
		if out.pbData.is_null() {
			return Vec::new();
		}
		// SAFETY: the call's own allocation, freed once when `_free` drops, after the copy below.
		let _free = unsafe { Owned::new(HLOCAL(out.pbData.cast())) };
		let len = usize::try_from(out.cbData).unwrap_or(0);
		// SAFETY: `cbData` bytes at `pbData`, which the call wrote and `_free` keeps alive until the copy is done.
		unsafe { std::slice::from_raw_parts(out.pbData, len) }.to_vec()
	}
}

/// No store elsewhere (the helper ships for macOS and Windows): settings.json keeps the secrets.
#[cfg(not(any(target_os = "macos", windows)))]
mod store {
	use std::path::Path;

	use super::Read;

	pub(super) const NAME: &str = "a secret store"; // copy-lint: not shown (the log lines' name of the store)
	pub(super) const BEFORE: &str = "secrets.before";

	pub(super) fn usable() -> Result<(), String> {
		Err("none on this system".to_owned())
	}

	pub(super) fn held_elsewhere() -> bool {
		false
	}

	pub(super) fn load(_dir: &Path) -> Read {
		Read::Absent
	}

	pub(super) fn save(_dir: &Path, _bytes: &[u8]) -> Result<(), String> {
		Err("none on this system".to_owned())
	}

	pub(super) fn save_before(_dir: &Path, _bytes: &[u8]) -> Result<(), String> {
		Err("none on this system".to_owned())
	}
}
