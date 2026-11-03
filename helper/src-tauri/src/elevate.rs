//! The one-time elevated steps on Windows (rust-helper `windows-capture-and-install.md`, "The firewall rule" and "The
//! game folder's ACL and Controlled Folder Access"): the helper never runs elevated; when Windows keeps the phone out,
//! the game folder closed or a folder of another game whose setup file the helper writes (slice 35: a Game State
//! Integration cfg folder, Rocket League's `TAGame/Config`), the window offers one step that Windows runs as an
//! administrator after its UAC prompt.
//! What the step names (the helper's path, the player's SID) is read here, unelevated, since a standard user elevates
//! as another account. Elsewhere there is nothing to allow: macOS asks nothing for a listener, and the game folder is
//! writable.

use std::path::Path;

/// What a one-time elevated step came to.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(not(windows), expect(dead_code, reason = "only Windows' elevated steps succeed or are declined"))]
pub enum Outcome {
	Done,
	/// The player said no to the UAC prompt.
	Declined,
	Failed(String),
}

/// What the firewall does with the phone's connections to the helper.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(windows), expect(dead_code, reason = "only Windows has a firewall that asks"))]
pub enum Gate {
	/// Lets them in.
	Open,
	/// No rule lets the helper in yet: the one-time elevated step adds one.
	Rule,
	/// "Block all incoming connections" is on, which no rule opens: only the player can turn it off.
	BlocksAll,
}

/// The firewall rule's name, the one the uninstaller's hook and the player see in Windows Defender Firewall.
const RULE: &str = "QuestPump"; // copy-lint: not shown (the program's name, as the rule's)

/// `cmd.exe`'s arguments for the firewall step: every inbound rule for this program removed (the block rules a
/// first-listen prompt wrote win over any allow rule), then one rule that lets devices on the local subnet reach it on
/// every profile (Windows files a new home network as public; a work PC's network is a domain one), the internet kept
/// out by `remoteip=LocalSubnet`. `netsh` goes by its path in the system folder, never by a name cmd would look up in
/// its working folder and on the search path first. `/d` skips cmd's `AutoRun`, `/v:off` its delayed `!` expansion, `/s`
/// keeps the inner quotes; `&` after the removal, which fails when there was nothing to remove; the exit code is the new
/// rule's. A path with `%` (cmd would expand it) or `"` is refused.
#[cfg_attr(all(not(windows), not(test)), expect(dead_code, reason = "the firewall step exists on Windows only"))]
fn firewall_command(netsh: &Path, program: &Path) -> Option<String> {
	let plain = |path: &Path| path.to_str().filter(|p| !p.contains(['%', '"'])).map(str::to_owned);
	let (n, p) = (plain(netsh)?, plain(program)?);
	Some(format!(
		// copy-lint: not shown (cmd's command line)
		"/d /v:off /s /c \"\"{n}\" advfirewall firewall delete rule name=all dir=in program=\"{p}\" & \"{n}\" \
		 advfirewall firewall add rule name=\"{RULE}\" dir=in action=allow program=\"{p}\" enable=yes profile=any \
		 remoteip=LocalSubnet\""
	))
}

/// `icacls.exe`'s arguments for the folder step: Modify for the player's SID on the folder, inherited by what it holds
/// and what the helper creates there, and on a link itself, never on what it names, should one take the folder's place
/// while Windows asks (`/L`). A trailing backslash would escape the closing quote.
#[cfg_attr(all(not(windows), not(test)), expect(dead_code, reason = "the folder step exists on Windows only"))]
fn grant_args(folder: &Path, sid: &str) -> Option<String> {
	let p = folder.to_str().filter(|p| !p.contains('"'))?.trim_end_matches('\\');
	Some(format!("\"{p}\" /grant \"*{sid}:(OI)(CI)M\" /L"))
}

/// Nothing keeps the phone out of a listener on macOS.
///
/// # Errors
/// None: the signature is Windows'.
#[cfg(not(windows))]
#[expect(clippy::unnecessary_wraps, reason = "the same signature as the Windows version, which reads the rules")]
pub fn firewall() -> anyhow::Result<Gate> {
	Ok(Gate::Open)
}

/// Only Windows has the firewall step.
#[cfg(not(windows))]
pub fn allow_network(_program: &Path) -> Outcome {
	Outcome::Failed("only Windows has this step".into())
}

/// Only Windows has the folder step.
#[cfg(not(windows))]
pub fn allow_folder(_folder: &Path) -> Outcome {
	Outcome::Failed("only Windows has this step".into())
}

#[cfg(windows)]
pub use win::{Com, allow_folder, allow_network, firewall, user_sid};

#[cfg(windows)]
mod win {
	use std::os::windows::ffi::OsStringExt as _;
	use std::path::{Path, PathBuf};

	use anyhow::{Context as _, anyhow};
	use windows::Win32::Foundation::{
		ERROR_CANCELLED, HANDLE, HLOCAL, RPC_E_CHANGED_MODE, S_OK, WAIT_OBJECT_0, WIN32_ERROR,
	};
	use windows::Win32::NetworkManagement::WindowsFirewall::{
		INetFwPolicy2, INetFwRule, NET_FW_ACTION_ALLOW, NET_FW_ACTION_BLOCK, NET_FW_MODIFY_STATE_GP_OVERRIDE,
		NET_FW_MODIFY_STATE_INBOUND_BLOCKED, NET_FW_PROFILE2_DOMAIN, NET_FW_PROFILE2_PRIVATE, NET_FW_PROFILE2_PUBLIC,
		NET_FW_RULE_DIR_IN, NetFwPolicy2,
	};
	use windows::Win32::Security::Authorization::ConvertSidToStringSidW;
	use windows::Win32::Security::{GetTokenInformation, TOKEN_QUERY, TOKEN_USER, TokenUser};
	use windows::Win32::Storage::FileSystem::{
		BY_HANDLE_FILE_INFORMATION, FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_REPARSE_POINT, FILE_FLAG_BACKUP_SEMANTICS,
		FILE_FLAG_OPEN_REPARSE_POINT, FILE_FLAGS_AND_ATTRIBUTES, FILE_READ_ATTRIBUTES, GetFileInformationByHandle,
	};
	use windows::Win32::System::Com::{
		CLSCTX_INPROC_SERVER, COINIT, COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE, CoCreateInstance,
		CoInitializeEx, CoUninitialize,
	};
	use windows::Win32::System::Ole::IEnumVARIANT;
	use windows::Win32::System::SystemInformation::GetSystemDirectoryW;
	use windows::Win32::System::Threading::{
		GetCurrentProcess, GetExitCodeProcess, INFINITE, OpenProcessToken, WaitForSingleObject,
	};
	use windows::Win32::System::Variant::{VARIANT, VT_DISPATCH, VariantClear};
	use windows::Win32::UI::Shell::{SEE_MASK_NOASYNC, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW, ShellExecuteExW};
	use windows::Win32::UI::WindowsAndMessaging::SW_HIDE;
	use windows::core::{HSTRING, Interface as _, Owned, PCWSTR, PWSTR, w};

	use super::{Gate, Outcome, firewall_command, grant_args};
	use crate::install;

	/// COM on this thread for the call (single-threaded, as `ShellExecuteEx` asks): released when dropped, unless the
	/// thread already had COM in another mode, which stays as it was.
	pub struct Com(bool);

	impl Com {
		/// # Errors
		/// COM could not start on this thread.
		pub fn init() -> anyhow::Result<Self> {
			Self::start(COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE)
		}

		fn start(mode: COINIT) -> anyhow::Result<Self> {
			// SAFETY: the reserved pointer is None; a successful call is balanced in Drop.
			let hr = unsafe { CoInitializeEx(None, mode) };
			if hr == RPC_E_CHANGED_MODE {
				return Ok(Self(false));
			}
			hr.ok().context("COM")?;
			Ok(Self(true))
		}
	}

	impl Drop for Com {
		fn drop(&mut self) {
			if self.0 {
				// SAFETY: balances this thread's successful CoInitializeEx above.
				unsafe { CoUninitialize() };
			}
		}
	}

	/// What the firewall does with the phone's connections, read for the current profiles (a standard user may read
	/// them): open with no firewall on, under a policy the PC's owner manages (a local rule would change nothing, so
	/// listening costs nothing), or when an inbound default of allow or an enabled inbound rule for this program lets it
	/// in and no such rule blocks it (the step removes those); closed to everything while "Block all incoming connections" is on for a
	/// current profile (the helper cannot tell which profile the phone's network is); else it waits for a rule.
	///
	/// # Errors
	/// The rules could not be read (the firewall service stopped).
	pub fn firewall() -> anyhow::Result<Gate> {
		let program = std::env::current_exe()?;
		let _com = Com::init()?;
		// SAFETY: creates the firewall's policy object in this process; COM is initialised on this thread.
		let policy: INetFwPolicy2 = unsafe { CoCreateInstance(&NetFwPolicy2, None, CLSCTX_INPROC_SERVER) }?;
		// SAFETY: getters of the live policy object, one profile a call as the API asks.
		let current = unsafe { policy.CurrentProfileTypes() }?;
		let mut on = Vec::new();
		for profile in [NET_FW_PROFILE2_DOMAIN, NET_FW_PROFILE2_PRIVATE, NET_FW_PROFILE2_PUBLIC] {
			// SAFETY: as above.
			if current & profile.0 != 0 && unsafe { policy.get_FirewallEnabled(profile) }?.as_bool() {
				on.push(profile);
			}
		}
		// SAFETY: a getter of the live policy object.
		let modify = unsafe { policy.LocalPolicyModifyState() }?;
		if on.is_empty() || modify == NET_FW_MODIFY_STATE_GP_OVERRIDE {
			return Ok(Gate::Open);
		}
		let mut blocks_all = modify == NET_FW_MODIFY_STATE_INBOUND_BLOCKED;
		let mut allow = false;
		for &profile in &on {
			// SAFETY: getters of the live policy object for one profile.
			blocks_all |= unsafe { policy.get_BlockAllInboundTraffic(profile) }?.as_bool();
			// SAFETY: as above.
			allow |= unsafe { policy.get_DefaultInboundAction(profile) }? == NET_FW_ACTION_ALLOW;
		}
		if blocks_all {
			return Ok(Gate::BlocksAll);
		}
		let program = program.to_string_lossy().to_lowercase();
		let mut block = false;
		// SAFETY: the rules' enumerator, cast to the interface its IUnknown carries.
		let rules: IEnumVARIANT = unsafe { policy.Rules()?._NewEnum() }?.cast()?;
		loop {
			let mut item = [VARIANT::default()];
			let mut fetched = 0u32;
			// SAFETY: `item` holds one VARIANT for the enumerator to fill; `fetched` receives the count.
			if unsafe { rules.Next(&mut item, &raw mut fetched) } != S_OK || fetched == 0 {
				break; // S_FALSE: the end
			}
			// SAFETY: the union is read as the dispatch pointer only when its tag says so; the VARIANT is cleared at
			// once, releasing its own reference (the cast took one of its own).
			let rule = unsafe {
				let v = &*item[0].Anonymous.Anonymous;
				let rule = if v.vt == VT_DISPATCH {
					(*v.Anonymous.pdispVal).as_ref().and_then(|d| d.cast::<INetFwRule>().ok())
				} else {
					None
				};
				let _ = VariantClear(&raw mut item[0]);
				rule
			};
			let Some(rule) = rule else { continue };
			// SAFETY: getters of a live rule object.
			let mine = unsafe { rule.ApplicationName() }.is_ok_and(|a| a.to_string().to_lowercase() == program);
			// SAFETY: as above.
			let here = mine
				&& unsafe { rule.Direction() }? == NET_FW_RULE_DIR_IN
				&& unsafe { rule.Enabled() }?.as_bool()
				&& unsafe { rule.Profiles() }? & current != 0;
			if here {
				// SAFETY: as above.
				if unsafe { rule.Action() }? == NET_FW_ACTION_BLOCK {
					block = true;
				} else {
					allow = true;
				}
			}
		}
		Ok(if allow && !block { Gate::Open } else { Gate::Rule })
	}

	/// What the elevated process came to.
	enum Ran {
		Declined,
		Exited(u32),
	}

	/// Runs `program` of Windows' system folder `system` with `params` as an administrator after the UAC prompt, hidden,
	/// in that folder (never the helper's own, which the player's account can write and cmd would search first), and
	/// waits for it (blocking: off the helper's loop).
	fn run_elevated(system: &Path, program: &str, params: &str) -> anyhow::Result<Ran> {
		let _com = Com::init()?;
		let (file, folder) = (HSTRING::from(system.join(program).as_os_str()), HSTRING::from(system.as_os_str()));
		let params = HSTRING::from(params);
		let mut info = SHELLEXECUTEINFOW {
			cbSize: u32::try_from(std::mem::size_of::<SHELLEXECUTEINFOW>())?,
			fMask: SEE_MASK_NOCLOSEPROCESS | SEE_MASK_NOASYNC,
			lpVerb: w!("runas"),
			lpFile: PCWSTR(file.as_ptr()),
			lpParameters: PCWSTR(params.as_ptr()),
			lpDirectory: PCWSTR(folder.as_ptr()),
			nShow: SW_HIDE.0,
			..Default::default()
		};
		// SAFETY: `info` is a complete SHELLEXECUTEINFOW whose strings outlive the call.
		if let Err(e) = unsafe { ShellExecuteExW(&raw mut info) } {
			return if WIN32_ERROR::from_error(&e) == Some(ERROR_CANCELLED) {
				Ok(Ran::Declined)
			} else {
				Err(e.into())
			};
		}
		if info.hProcess.is_invalid() {
			return Err(anyhow!("Windows started no process"));
		}
		// SAFETY: SEE_MASK_NOCLOSEPROCESS hands this process's handle to the caller, who closes it: Owned does.
		let process = unsafe { Owned::new(info.hProcess) };
		// SAFETY: waits on a live process handle.
		if unsafe { WaitForSingleObject(*process, INFINITE) } != WAIT_OBJECT_0 {
			return Err(windows::core::Error::from_thread().into());
		}
		let mut code = 0u32;
		// SAFETY: reads the exit code of the process that ended.
		unsafe { GetExitCodeProcess(*process, &raw mut code) }?;
		Ok(Ran::Exited(code))
	}

	/// Windows' system folder, where `cmd.exe`, `netsh.exe` and `icacls.exe` live (never found through the search path
	/// when they run elevated).
	fn system_dir() -> anyhow::Result<PathBuf> {
		let mut buf = [0u16; 260];
		// SAFETY: writes at most `buf.len()` units; returns the length written, or the size needed when `buf` is too
		// small, which `get(..len)` below then refuses.
		let len = usize::try_from(unsafe { GetSystemDirectoryW(Some(&mut buf)) })?;
		let dir = buf.get(..len).filter(|d| !d.is_empty()).ok_or_else(|| anyhow!("no system folder"))?;
		Ok(PathBuf::from(std::ffi::OsString::from_wide(dir)))
	}

	/// The SID of the player running the helper, as text (the folder step's grant, the local socket's access).
	///
	/// # Errors
	/// The process token could not be read.
	pub fn user_sid() -> anyhow::Result<String> {
		let mut token = HANDLE::default();
		// SAFETY: the current process's pseudo-handle, never closed; `token` receives a handle Owned closes.
		unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &raw mut token) }?;
		// SAFETY: `token` is the handle just opened, closed once.
		let token = unsafe { Owned::new(token) };
		let mut len = 0u32;
		// SAFETY: a size query without a buffer; its error (the buffer too small) is expected.
		let _ = unsafe { GetTokenInformation(*token, TokenUser, None, 0, &raw mut len) };
		// pointer-aligned: TOKEN_USER holds a pointer into the same buffer
		let mut buf = vec![0u64; usize::try_from(len)?.div_ceil(8)];
		// SAFETY: `buf` holds at least `len` bytes.
		unsafe { GetTokenInformation(*token, TokenUser, Some(buf.as_mut_ptr().cast()), len, &raw mut len) }?;
		// SAFETY: the call filled `buf` with a TOKEN_USER whose SID points into `buf`, which outlives this borrow.
		let user = unsafe { &*buf.as_ptr().cast::<TOKEN_USER>() };
		let mut text = PWSTR::null();
		// SAFETY: converts the SID into a string LocalFree releases (through Owned).
		unsafe { ConvertSidToStringSidW(user.User.Sid, &raw mut text) }?;
		// SAFETY: the string's memory, released once.
		let _free = unsafe { Owned::new(HLOCAL(text.0.cast())) };
		// SAFETY: a NUL-terminated string the call wrote.
		Ok(unsafe { text.to_string() }?)
	}

	fn outcome(ran: anyhow::Result<Ran>, what: &str) -> Outcome {
		match ran {
			Ok(Ran::Exited(0)) => Outcome::Done,
			Ok(Ran::Exited(code)) => Outcome::Failed(format!("{what} ended with {code}")),
			Ok(Ran::Declined) => Outcome::Declined,
			Err(e) => Outcome::Failed(format!("{what}: {e:#}")),
		}
	}

	/// Adds the firewall rule for `program` after removing every inbound rule for it, then reads the rules again: a
	/// block rule the removal missed still keeps the phone out.
	pub fn allow_network(program: &Path) -> Outcome {
		let system = match system_dir() {
			Ok(system) => system,
			Err(e) => return Outcome::Failed(format!("netsh: {e:#}")),
		};
		let Some(params) = firewall_command(&system.join("netsh.exe"), program) else {
			return Outcome::Failed(format!("{} cannot go on cmd's command line", program.display()));
		};
		outcome(run_elevated(&system, "cmd.exe", &params), "netsh")
	}

	/// Lets the player write into a folder (the game's AddOns folder, a folder of another game's setup file): Modify for
	/// their SID, inherited by what it holds. Only a folder on this computer's own disks, as named and as the system
	/// resolves it (`real_folder`), and only that folder: right before the UAC prompt it and every folder above it must
	/// be real folders, never a link or a junction (`chain_ids`); should one take the place of the folder itself while
	/// Windows asks, icacls grants on the link itself (`grant_args`' `/L`), and the same check after the step finds the
	/// folder or one above it changed and fails it.
	pub fn allow_folder(folder: &Path) -> Outcome {
		let sid = match user_sid() {
			Ok(sid) => sid,
			Err(e) => return Outcome::Failed(format!("the player's SID: {e:#}")),
		};
		let system = match system_dir() {
			Ok(system) => system,
			Err(e) => return Outcome::Failed(format!("icacls: {e:#}")),
		};
		let (real, before) = match real_folder(folder).and_then(|real| chain_ids(&real).map(|ids| (real, ids))) {
			Ok(found) => found,
			Err(why) => return Outcome::Failed(why),
		};
		let Some(params) = grant_args(&real, &sid) else {
			return Outcome::Failed(format!("{} cannot go on icacls' command line", real.display()));
		};
		match outcome(run_elevated(&system, "icacls.exe", &params), "icacls") {
			Outcome::Done if chain_ids(&real).ok() != Some(before) => {
				Outcome::Failed("the folder changed while Windows asked".to_owned())
			}
			done => done,
		}
	}

	/// `folder` as the system resolves it (its links followed) in the plain form icacls reads (`C:\…`, never
	/// `\\?\C:\…`), when it is on this computer's own disks as named and as resolved (`install::on_this_computer`)
	/// and the plain form resolves to the same folder (a component ending in a dot or a space would not).
	fn real_folder(folder: &Path) -> Result<PathBuf, String> {
		// copy-lint: not shown (an error's detail, as icacls' own failures show it)
		let elsewhere = || format!("{} is on no disk of this computer's", folder.display());
		if !install::on_this_computer(folder) {
			return Err(elsewhere());
		}
		let real = std::fs::canonicalize(folder).map_err(|e| format!("{}: {e}", folder.display()))?;
		if !install::on_this_computer(&real) {
			return Err(elsewhere());
		}
		let plain = real
			.to_str()
			.and_then(|r| r.strip_prefix(r"\\?\"))
			.map(PathBuf::from)
			.filter(|plain| std::fs::canonicalize(plain).is_ok_and(|again| again == real))
			.ok_or_else(|| format!("{} has no plain name icacls reads", real.display()))?;
		Ok(plain)
	}

	/// The folder at `real` and every folder above it but its disk's root, each by `folder_id`: a link or a junction at
	/// any of them is refused before the prompt, since icacls' `/L` and the check of the folder itself guard only its
	/// last component, and a junction above it would take the grant elsewhere.
	fn chain_ids(real: &Path) -> Result<Vec<(u32, u64)>, String> {
		real.ancestors().filter(|path| path.parent().is_some()).map(folder_id).collect()
	}

	/// The folder at `path` itself, never what a link there names (`FILE_FLAG_OPEN_REPARSE_POINT`), as its volume's
	/// serial and its file index, when it is a real folder; a link, a junction or a file is refused.
	fn folder_id(path: &Path) -> Result<(u32, u64), String> {
		use std::os::windows::fs::OpenOptionsExt as _;
		use std::os::windows::io::AsRawHandle as _;
		let file = std::fs::OpenOptions::new()
			.access_mode(FILE_READ_ATTRIBUTES.0)
			.custom_flags((FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT).0)
			.open(path)
			.map_err(|e| format!("{}: {e}", path.display()))?;
		let mut info = BY_HANDLE_FILE_INFORMATION::default();
		// SAFETY: the handle `file` holds open through the call, which writes `info` alone.
		unsafe { GetFileInformationByHandle(HANDLE(file.as_raw_handle()), &raw mut info) }
			.map_err(|e| format!("{}: {e}", path.display()))?;
		let kind = FILE_FLAGS_AND_ATTRIBUTES(info.dwFileAttributes);
		if kind.contains(FILE_ATTRIBUTE_REPARSE_POINT) || !kind.contains(FILE_ATTRIBUTE_DIRECTORY) {
			return Err(format!("{} is a link or no folder", path.display()));
		}
		Ok((info.dwVolumeSerialNumber, (u64::from(info.nFileIndexHigh) << 32) | u64::from(info.nFileIndexLow)))
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn the_firewall_step_removes_the_programs_rules_then_allows_the_local_subnet_on_every_profile_in_one_prompt() {
		let netsh = Path::new(r"C:\WINDOWS\system32\netsh.exe");
		let program = Path::new(r"C:\Users\Ann O'Neil\AppData\Local\QuestPump\questpump-helper.exe");
		let line = firewall_command(netsh, program).unwrap();
		let quoted = format!("program=\"{}\"", program.display());
		assert!(
			line.starts_with(
				r#"/d /v:off /s /c ""C:\WINDOWS\system32\netsh.exe" advfirewall firewall delete rule name=all dir=in "#
			),
			"netsh by its path, never by a name cmd looks up: {line}"
		);
		assert_eq!(line.matches(&quoted).count(), 2, "the removal and the rule name the same program: {line}");
		assert!(
			line.contains(
				r#" & "C:\WINDOWS\system32\netsh.exe" advfirewall firewall add rule name="QuestPump" dir=in "#
			),
			"{line}"
		);
		assert!(line.ends_with("profile=any remoteip=LocalSubnet\""), "{line}");
		assert_eq!(firewall_command(netsh, Path::new(r"C:\Users\100%\x.exe")), None, "cmd would expand the %");
		assert_eq!(firewall_command(Path::new(r"C:\%x%\netsh.exe"), program), None, "nor in netsh's path");
	}

	#[test]
	fn the_folder_step_grants_modify_to_the_players_sid_inherited_by_the_folders_content() {
		let addons = Path::new(r"C:\Program Files (x86)\World of Warcraft\_classic_beta_\Interface\AddOns\");
		let args = grant_args(addons, "S-1-5-21-1-2-3-1001").unwrap();
		assert_eq!(
			args,
			r#""C:\Program Files (x86)\World of Warcraft\_classic_beta_\Interface\AddOns" /grant "*S-1-5-21-1-2-3-1001:(OI)(CI)M" /L"#,
			"no trailing backslash to escape the quote, and a link put in the folder's place granted, never its target"
		);
	}
}
