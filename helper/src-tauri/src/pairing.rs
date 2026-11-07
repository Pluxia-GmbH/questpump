//! The pair link the Phones page shows (docs/plan.md slices 5 and 35): the computer's LAN address, the port, the
//! pairing key, the computer's name, its `.local` host name and the install's id in `https://questpump.com/pair#…`, as
//! a QR code's modules the page draws in the kit's `QrWell` (ink on the light well, its quiet zone around). The address
//! is read from the network interfaces and the routing table, the host name from the system's own setting, all three
//! again every minute (`Address::read`): the helper sends nothing to find them.

use std::net::IpAddr;

use questpump_core::pair::{self, PairLink};

/// Interfaces that are never the home network: virtual machines' bridges, VPN tunnels (macOS names every one `utun`;
/// Windows names them after the product), Apple's peer links.
const VIRTUAL: [&str; 15] = [
	"bridge",
	"utun",
	"vmnet",
	"awdl",
	"llw",
	"vEthernet",
	"VirtualBox",
	"VMware",
	"Hyper-V",
	"Tailscale", // copy-lint: not shown (an interface's name)
	"ZeroTier",
	"WireGuard",
	"OpenVPN",
	"NordLynx",
	"Mullvad", // copy-lint: not shown (an interface's name)
];

/// The interface that carries the default route, read from the system's routing table (`route -n get default` on
/// macOS, by its path as every system tool the helper runs; the command only reads, it sends nothing).
#[cfg(target_os = "macos")]
fn default_interface() -> Option<String> {
	let out = std::process::Command::new("/sbin/route").args(["-n", "get", "default"]).output().ok()?;
	let text = String::from_utf8(out.stdout).ok()?;
	text.lines().find_map(|l| l.trim().strip_prefix("interface:").map(|n| n.trim().to_owned()))
}

/// The interface that carries the default route on Windows: the routing table's best interface for an address
/// outside every local network (TEST-NET-1, RFC 5737; a lookup, nothing is sent), named as `if-addrs` names it.
#[cfg(windows)]
fn default_interface() -> Option<String> {
	use windows::Win32::NetworkManagement::IpHelper::GetBestInterfaceEx;
	use windows::Win32::Networking::WinSock::{AF_INET, IN_ADDR, IN_ADDR_0, SOCKADDR, SOCKADDR_IN};

	let outside = SOCKADDR_IN {
		sin_family: AF_INET,
		sin_port: 0,
		sin_addr: IN_ADDR { S_un: IN_ADDR_0 { S_addr: u32::from_ne_bytes([192, 0, 2, 1]) } },
		sin_zero: [0; 8],
	};
	let mut index = 0u32;
	// SAFETY: `outside` is a complete IPv4 socket address alive for the call; `index` receives the interface index.
	let found = unsafe { GetBestInterfaceEx(std::ptr::from_ref(&outside).cast::<SOCKADDR>(), &raw mut index) };
	if found != 0 {
		return None;
	}
	let interfaces = if_addrs::get_if_addrs().ok()?;
	interfaces.into_iter().find(|i| i.index == Some(index) && i.ip().is_ipv4()).map(|i| i.name)
}

/// The computer's address on the home network: the private IPv4 address of the interface that carries the default
/// route, else of the first interface that is up and not a virtual one; a point-to-point link (a tunnel) never.
fn lan_address() -> Option<IpAddr> {
	let usable: Vec<(String, IpAddr)> = if_addrs::get_if_addrs()
		.ok()?
		.into_iter()
		.filter(|i| !i.is_loopback() && !i.is_p2p() && i.is_oper_up() && i.ip().is_ipv4() && pair::is_private(&i.ip()))
		.map(|i| (i.name.clone(), i.ip()))
		.collect();
	choose(&usable, default_interface().as_deref())
}

/// The address among the usable ones (up, private IPv4): the default route's interface unless it is a virtual one (a
/// full-tunnel VPN takes the default route, and the phone on the home network never reaches its address) or holds only
/// a link-local address; else the `en` interfaces (Wi-Fi, Ethernet) first and an RFC 1918 address before a link-local
/// 169.254 one.
fn choose(usable: &[(String, IpAddr)], default: Option<&str>) -> Option<IpAddr> {
	let real = |name: &str| !VIRTUAL.iter().any(|v| name.starts_with(v));
	let link_local = |ip: &IpAddr| matches!(ip, IpAddr::V4(v4) if v4.is_link_local());
	if let Some(name) = default.filter(|n| real(n))
		&& let Some((_, ip)) = usable.iter().find(|(n, ip)| n == name && !link_local(ip))
	{
		return Some(*ip);
	}
	let mut ranked: Vec<&(String, IpAddr)> = usable.iter().filter(|(n, _)| real(n)).collect();
	ranked.sort_by_key(|(n, ip)| (link_local(ip), !n.starts_with("en"), n.clone()));
	ranked.first().map(|(_, ip)| *ip)
}

/// The computer's name for the phone to show: the Mac's Sharing name; Windows' device name as Settings shows it (its
/// DNS host name, in the player's own spelling), `COMPUTERNAME` (upper case, cut to 15 characters) only without one.
fn computer_name() -> String {
	let name = if cfg!(target_os = "macos") {
		std::process::Command::new("/usr/sbin/scutil")
			.args(["--get", "ComputerName"])
			.output()
			.ok()
			.filter(|o| o.status.success())
			.and_then(|o| String::from_utf8(o.stdout).ok())
	} else {
		system_host_name().filter(|n| !n.trim().is_empty()).or_else(|| std::env::var("COMPUTERNAME").ok())
	};
	name.map(|s| s.trim().to_owned())
		.filter(|s| !s.is_empty())
		.unwrap_or_else(|| crate::copy::desk_menu_phones_unnamed().to_owned())
}

/// The computer's own `.local` host name, the one its mDNS responder answers (the Mac's Local Hostname in Sharing,
/// Windows' DNS host name), for the phone to find the helper by when the address moves; none when the system gives no
/// valid `.local` name.
fn local_hostname() -> Option<String> {
	local_name(&system_host_name()?)
}

/// `<host>.local` when that is a valid `.local` host name: never a name every app would refuse the link for.
fn local_name(host: &str) -> Option<String> {
	let name = format!("{host}.local");
	pair::is_local_name(&name).then_some(name)
}

/// The Mac's Local Hostname, the value `scutil --get LocalHostName` prints, asked of configd through
/// `SCDynamicStoreCopyLocalHostName`: no process, so the pair link reads it on the runtime's loop.
#[cfg(target_os = "macos")]
fn system_host_name() -> Option<String> {
	use std::ffi::{CStr, c_char, c_void};

	/// `kCFStringEncodingUTF8`.
	const UTF8: u32 = 0x0800_0100;
	const SIZE: isize = 256;

	// SAFETY: the C signatures of <SystemConfiguration/SCDynamicStoreCopySpecific.h>, <CoreFoundation/CFString.h> and
	// <CoreFoundation/CFBase.h>; both frameworks ship with every macOS.
	#[link(name = "SystemConfiguration", kind = "framework")]
	unsafe extern "C" {
		fn SCDynamicStoreCopyLocalHostName(store: *const c_void) -> *const c_void;
	}
	#[link(name = "CoreFoundation", kind = "framework")]
	unsafe extern "C" {
		fn CFStringGetCString(string: *const c_void, buffer: *mut c_char, size: isize, encoding: u32) -> u8;
		fn CFRelease(object: *const c_void);
	}

	// SAFETY: a null store asks through a temporary session; the result is null (no name set) or a CFString this
	// call owns.
	let name = unsafe { SCDynamicStoreCopyLocalHostName(std::ptr::null()) };
	if name.is_null() {
		return None;
	}
	let mut buffer = [0u8; SIZE.unsigned_abs()];
	// SAFETY: `name` is a live CFString and `buffer` holds SIZE writable bytes; the call writes at most that many, the
	// terminating null included, and fails when the name does not fit.
	let copied = unsafe { CFStringGetCString(name, buffer.as_mut_ptr().cast::<c_char>(), SIZE, UTF8) } != 0;
	// SAFETY: `name` came from a Copy function, and this is its one release.
	unsafe { CFRelease(name) };
	if !copied {
		return None;
	}
	CStr::from_bytes_until_nul(&buffer).ok()?.to_str().ok().map(str::to_owned)
}

/// Windows' DNS host name (`GetComputerNameExW`, `ComputerNameDnsHostname`): the name its DNS Client answers on mDNS,
/// never `COMPUTERNAME`, the legacy name cut to 15 characters.
#[cfg(windows)]
fn system_host_name() -> Option<String> {
	use windows::Win32::System::SystemInformation::{ComputerNameDnsHostname, GetComputerNameExW};
	use windows::core::PWSTR;

	let mut name = [0u16; 256];
	let mut len = u32::try_from(name.len()).ok()?;
	// SAFETY: `name` holds `len` writable UTF-16 units; the call writes at most that many and sets `len` to the units
	// written, without the terminator.
	unsafe { GetComputerNameExW(ComputerNameDnsHostname, Some(PWSTR(name.as_mut_ptr())), &raw mut len) }.ok()?;
	String::from_utf16(name.get(..usize::try_from(len).ok()?)?).ok()
}

/// The computer as a device finds it, read at the start and again every minute: its name, its LAN address and its
/// `.local` host name, which the system renames when another computer on the network claims it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Address {
	pub name: String,
	pub host: Option<IpAddr>,
	pub mdns: Option<String>,
}

impl Address {
	/// Reads all three; it runs processes (`route`, `scutil`), so off the runtime's loop.
	pub fn read() -> Self {
		Self { name: computer_name(), host: lan_address(), mdns: local_hostname() }
	}
}

pub use questpump_desk::qr::{Qr, qr};

/// The link and its QR code; none when the link does not fit a QR code.
pub fn link_and_qr(host: IpAddr, key: [u8; 32], name: &str, mdns: Option<&str>, hid: [u8; 16]) -> (String, Option<Qr>) {
	let mdns = mdns.map(str::to_owned);
	let link = PairLink { host, port: pair::PORT, key, name: name.to_owned(), mdns, hid: Some(hid) };
	let url = link.to_url();
	let qr = qr(&url);
	(url, qr)
}

#[cfg(test)]
mod tests {
	use super::*;

	fn usable(list: &[(&str, &str)]) -> Vec<(String, IpAddr)> {
		list.iter().map(|&(n, ip)| (n.to_owned(), ip.parse().unwrap_or(IpAddr::from([0, 0, 0, 0])))).collect()
	}

	#[test]
	fn a_vpn_that_takes_the_default_route_never_becomes_the_pair_links_host() {
		let list = usable(&[("utun4", "10.8.0.2"), ("en0", "192.168.1.20")]);
		assert_eq!(choose(&list, Some("utun4")), "192.168.1.20".parse().ok());
		assert_eq!(choose(&list, Some("en0")), "192.168.1.20".parse().ok());
	}

	#[test]
	fn a_home_network_address_comes_before_a_link_local_one() {
		let list = usable(&[("en1", "169.254.3.4"), ("en7", "10.0.0.5")]);
		assert_eq!(choose(&list, None), "10.0.0.5".parse().ok());
		assert_eq!(
			choose(&list, Some("en1")),
			"10.0.0.5".parse().ok(),
			"a default route with only a link-local address"
		);
		assert_eq!(choose(&usable(&[("bridge100", "192.168.64.1")]), None), None, "a VM's bridge alone is no LAN");
	}

	#[test]
	fn the_window_shows_a_link_every_app_reads_back_with_this_computers_local_name_and_the_install_id() {
		let host = IpAddr::from([192, 168, 1, 20]);
		let mdns = local_hostname();
		let (url, qr) = link_and_qr(host, [1; 32], "Mac", mdns.as_deref(), [2; 16]);
		let link = PairLink::parse(&url).unwrap();
		assert_eq!((link.host, link.hid), (host, Some([2; 16])));
		assert_eq!(link.mdns, mdns, "{url}");
		let qr = qr.unwrap();
		assert_eq!(qr.modules.len(), qr.width * qr.width, "a square of modules");
	}

	#[test]
	fn a_host_name_the_apps_would_refuse_leaves_the_link_without_a_local_name() {
		assert_eq!(local_name("Davids-MacBook-Pro-M5").as_deref(), Some("Davids-MacBook-Pro-M5.local"));
		// Windows allows an underscore in a computer's name, DNS labels do not
		for refused in ["GAMING_PC", "", "-pc", &"a".repeat(64)] {
			assert_eq!(local_name(refused), None, "{refused}");
		}
	}
}
