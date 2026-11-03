//! QuestPump's own posts to questpump.com (docs/plan.md slice 24; questpump-core's `feedback.rs` and `crash.rs` own the
//! bodies and `outbox.rs` the outbox, `runtime/outbox.rs` when they go): one HTTPS POST per entry to its route, the
//! feedback button's (a message the player sent with Send, in the game or in the helper's window) or the crash
//! reports' (contract/crash-api.yaml). The client names the helper and its version in its User-Agent and nothing else
//! (Cloudflare's Browser Integrity Check challenges a bare library's, and with the tick off only the text, the source
//! and the version may leave), sends no `Origin`, waits 20 s at most, follows no redirect and goes over the system's
//! own TLS and proxy, as the updater does. Also the system's name and version for the window's diagnostics and the
//! crash reports' `os`.

use std::time::Duration;

/// The User-Agent of QuestPump's own requests (the desktop app's films' download too).
pub const USER_AGENT: &str = concat!("QuestPump-Helper/", env!("CARGO_PKG_VERSION"));
/// The longest a post may take, from the connection to the answer.
const TIMEOUT: Duration = Duration::from_secs(20);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// The client every post goes through.
///
/// # Errors
/// The system's TLS could not be set up.
pub fn client() -> reqwest::Result<reqwest::Client> {
	reqwest::Client::builder()
		.user_agent(USER_AGENT)
		.tls_backend_native()
		.timeout(TIMEOUT)
		.connect_timeout(CONNECT_TIMEOUT)
		.redirect(reqwest::redirect::Policy::none())
		.build()
}

/// Posts one entry's body to its route: the route's status, or why no answer came.
///
/// # Errors
/// No answer: no connection, a timeout, a TLS failure.
pub async fn post(client: &reqwest::Client, route: &str, body: Vec<u8>) -> Result<u16, String> {
	client
		.post(route)
		.header(reqwest::header::CONTENT_TYPE, "application/json")
		.body(body)
		.send()
		.await
		.map(|r| r.status().as_u16())
		.map_err(|e| format!("{:#}", anyhow::Error::new(e.without_url())))
}

/// The system as the window's diagnostics name it, such as "macOS 27.0" (a crash report's `os` adds the architecture).
#[cfg(target_os = "macos")]
pub fn os() -> String {
	let mut buf = [0u8; 32];
	let mut len = buf.len();
	// SAFETY: the name is a NUL-terminated C string; `buf` holds `len` bytes, the call writes at most that many and sets
	// `len` to what it wrote, and no new value is set.
	let read = unsafe {
		libc::sysctlbyname(
			c"kern.osproductversion".as_ptr(),
			buf.as_mut_ptr().cast(),
			&raw mut len,
			std::ptr::null_mut(),
			0,
		)
	};
	let version = (read == 0)
		.then(|| std::ffi::CStr::from_bytes_until_nul(buf.get(..len)?).ok()?.to_str().ok())
		.flatten()
		.filter(|v| !v.is_empty());
	version.map_or_else(|| "macOS".to_owned(), |v| format!("macOS {v}"))
}

/// The system as the window's diagnostics name it, such as "Windows 11 24H2 (26100)": the build decides 10 or 11, since
/// the registry's product name says Windows 10 on Windows 11 too.
#[cfg(windows)]
pub fn os() -> String {
	use windows::Win32::System::Registry::{HKEY_LOCAL_MACHINE, RRF_RT_REG_SZ, RegGetValueW};
	use windows::core::{PCWSTR, w};

	const KEY: PCWSTR = w!(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion");
	let read = |name: PCWSTR| -> Option<String> {
		let mut buf = [0u16; 64];
		let mut len = u32::try_from(std::mem::size_of_val(&buf)).ok()?;
		// SAFETY: `buf` holds `len` bytes; the call writes at most that many, the string's NUL included, and sets `len`
		// to what it wrote.
		let e = unsafe {
			RegGetValueW(
				HKEY_LOCAL_MACHINE,
				KEY,
				name,
				RRF_RT_REG_SZ,
				None,
				Some(buf.as_mut_ptr().cast()),
				Some(&raw mut len),
			)
		};
		if !e.is_ok() {
			return None;
		}
		let units = buf.get(..usize::try_from(len).ok()? / 2)?;
		let text = String::from_utf16(units).ok()?;
		Some(text.trim_end_matches('\0').to_owned()).filter(|t| !t.is_empty())
	};
	let Some(build) = read(w!("CurrentBuild")) else { return "Windows".to_owned() };
	let major = if build.parse::<u32>().is_ok_and(|b| b >= 22_000) { 11 } else { 10 };
	match read(w!("DisplayVersion")) {
		Some(release) => format!("Windows {major} {release} ({build})"),
		None => format!("Windows {major} ({build})"),
	}
}
