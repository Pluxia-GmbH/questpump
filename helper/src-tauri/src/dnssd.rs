//! The helper's Bonjour record on Windows (docs/companion.md, "The companion", item 1; rust-helper
//! `windows-capture-and-install.md`, "The Bonjour record"): service `_questpump._tcp`, the instance `QuestPump`
//! (Windows' responder renames it on a clash, `QuestPump (2)`), the computer's `.local` host name with the LAN's IPv4
//! address, port 8787, and TXT `txtvers=2` alone, so a phone's browse finds a Windows helper whose address moved, an
//! Android phone too old to resolve `.local` names included. The instance names no computer and the TXT no id of the
//! install or the pairing key (since the release security review of 2026-10-31, L3; the instance was the computer's
//! name and the TXT carried `hid` and `kid` until then): the phone tells its helper among the answers by the hello's
//! proofs (`lan::hello`). The host stays the `.local` name, which resolving the record needs. Windows' DNS Client
//! answers for it (`DnsServiceRegister`), so the helper opens no socket and the registration ends with the process; it
//! is registered again when the address or the `.local` name changes, and withdrawn while the LAN API does not listen
//! and at quit. macOS registers nothing: advertising needs the Local Network privilege there, whose answer can never be
//! reset, and the Mac's own `.local` name finds it without one.

use std::net::{IpAddr, Ipv4Addr};

use questpump_core::pair;

/// The service type in the `.local` domain.
const SERVICE: &str = "_questpump._tcp.local";
/// The instance's label, the same on every computer. No phone reads it: each resolves every record and asks its hello.
const INSTANCE: &str = "QuestPump"; // copy-lint: not shown (a network label, never translated)

/// What the record says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
	/// `QuestPump._questpump._tcp.local`.
	pub instance: String,
	/// The computer's `.local` host name, which the record's address answers for.
	pub host: String,
	pub ip: Ipv4Addr,
	pub port: u16,
	/// The TXT keys (at most 9 characters, RFC 6763 section 6.4) and their values.
	pub txt: [(&'static str, &'static str); 1],
}

impl Record {
	/// The record of the computer's `.local` name and LAN address; none without a `.local` name or an IPv4 address to
	/// give.
	pub fn new(mdns: Option<&str>, host: Option<IpAddr>) -> Option<Self> {
		let IpAddr::V4(ip) = host? else { return None };
		Some(Self {
			instance: format!("{INSTANCE}.{SERVICE}"),
			host: mdns?.to_owned(),
			ip,
			port: pair::PORT,
			txt: [("txtvers", "2")],
		})
	}
}

/// The record the helper shows: registered on Windows by a thread of its own (Windows confirms through a callback,
/// which the thread waits for), nothing on macOS.
pub struct Advert {
	/// The record asked for last.
	shown: Option<Record>,
	/// Windows refused or failed the last registration: the next `show` (the next look at the address, each minute)
	/// asks again, as if nothing were shown.
	failed: std::sync::Arc<std::sync::atomic::AtomicBool>,
	#[cfg(windows)]
	jobs: std::sync::mpsc::Sender<win::Job>,
}

impl Advert {
	pub fn start() -> Self {
		let failed = std::sync::Arc::default();
		Self {
			shown: None,
			#[cfg(windows)]
			jobs: win::spawn(std::sync::Arc::clone(&failed)),
			failed,
		}
	}

	/// Shows this record from now on, or none; the record already shown changes nothing, unless its registration failed.
	pub fn show(&mut self, record: Option<Record>) {
		let failed = self.failed.swap(false, std::sync::atomic::Ordering::Relaxed);
		if record == self.shown && !failed {
			return;
		}
		#[cfg(windows)]
		let _ = self.jobs.send(win::Job::Show(record.clone()));
		self.shown = record;
	}

	/// Withdraws the record before the helper exits or restarts, waiting at most 5 s for Windows to confirm.
	#[cfg_attr(not(windows), expect(clippy::unused_async, reason = "only Windows has a record to wait for"))]
	pub async fn stop(&mut self) {
		self.shown = None;
		#[cfg(windows)]
		{
			let (done, wait) = tokio::sync::oneshot::channel();
			if self.jobs.send(win::Job::Stop(done)).is_ok() {
				let _ = tokio::time::timeout(std::time::Duration::from_secs(5), wait).await;
			}
		}
	}
}

#[cfg(windows)]
mod win {
	use std::ffi::c_void;
	use std::sync::mpsc;
	use std::time::Duration;

	use windows::Win32::Foundation::{DNS_REQUEST_PENDING, HANDLE};
	use windows::Win32::NetworkManagement::Dns::{
		DNS_QUERY_REQUEST_VERSION1, DNS_SERVICE_INSTANCE, DNS_SERVICE_REGISTER_REQUEST, DnsServiceConstructInstance,
		DnsServiceDeRegister, DnsServiceFreeInstance, DnsServiceRegister,
	};
	use windows::core::{BOOL, PCWSTR};

	use super::Record;

	/// How long Windows gets to confirm a registration or a withdrawal.
	const CONFIRM: Duration = Duration::from_secs(2);

	/// What the advert's thread is asked.
	pub(super) enum Job {
		/// Withdraw the record shown, then register this one.
		Show(Option<Record>),
		/// Withdraw the record shown, then say so.
		Stop(tokio::sync::oneshot::Sender<()>),
	}

	/// The thread that registers and withdraws, one job at a time.
	pub(super) fn spawn(failed: std::sync::Arc<std::sync::atomic::AtomicBool>) -> mpsc::Sender<Job> {
		let (jobs, rx) = mpsc::channel::<Job>();
		let spawned = std::thread::Builder::new().name("bonjour".into()).spawn(move || {
			let mut current: Option<Registration> = None;
			for job in rx {
				if let Some(old) = current.take() {
					old.withdraw();
				}
				match job {
					Job::Show(Some(record)) => match Registration::register(&record) {
						Ok(registration) => current = Some(registration),
						Err(e) => {
							log::warn!(
								"bonjour: {} not registered ({e}); tried again at the next look",
								record.instance
							);
							failed.store(true, std::sync::atomic::Ordering::Relaxed);
						}
					},
					Job::Show(None) => {}
					Job::Stop(done) => {
						let _ = done.send(());
					}
				}
			}
		});
		if let Err(e) = spawned {
			log::warn!("bonjour: no thread ({e}); this PC shows no record");
		}
		jobs
	}

	/// Windows' completion callback, for a registration and for its withdrawal: the copy of the instance it passes is
	/// the callback's to free, and the status goes to the registration's channel.
	unsafe extern "system" fn completed(status: u32, context: *const c_void, instance: *const DNS_SERVICE_INSTANCE) {
		if !instance.is_null() {
			// SAFETY: DNS_SERVICE_REGISTER_COMPLETE hands the callback a copy it must free with DnsServiceFreeInstance.
			unsafe { DnsServiceFreeInstance(instance) };
		}
		// SAFETY: the context is the registration's sender (`Box::into_raw`), freed only after Windows' last callback
		// for it, or never; `Sender` is `Sync`, so this thread may send through a shared reference.
		if let Some(sender) = unsafe { context.cast::<mpsc::Sender<u32>>().as_ref() } {
			let _ = sender.send(status);
		}
	}

	/// A registration Windows holds. Its request, the instance the request points to and the callback's context stay
	/// alive until Windows confirmed the withdrawal, and are leaked when a confirmation never comes (Windows may still
	/// use them): dropping one frees nothing.
	struct Registration {
		request: *mut DNS_SERVICE_REGISTER_REQUEST,
		context: *mut mpsc::Sender<u32>,
		done: mpsc::Receiver<u32>,
		/// Windows confirmed the registration.
		confirmed: bool,
	}

	impl Registration {
		/// Registers the record and waits for Windows' answer, at most `CONFIRM`.
		fn register(record: &Record) -> Result<Self, String> {
			let wide = |s: &str| s.encode_utf16().chain(Some(0)).collect::<Vec<u16>>();
			let (name, host) = (wide(&record.instance), wide(&record.host));
			let keys: Vec<Vec<u16>> = record.txt.iter().map(|(k, _)| wide(k)).collect();
			let values: Vec<Vec<u16>> = record.txt.iter().map(|(_, v)| wide(v)).collect();
			let keys: Vec<PCWSTR> = keys.iter().map(|k| PCWSTR(k.as_ptr())).collect();
			let values: Vec<PCWSTR> = values.iter().map(|v| PCWSTR(v.as_ptr())).collect();
			// IP4_ADDRESS: the four bytes in network order, as they lie in memory
			let ip = u32::from_ne_bytes(record.ip.octets());
			let count = u32::try_from(keys.len()).map_err(|e| e.to_string())?;
			// SAFETY: every string is a NUL-terminated UTF-16 buffer and both arrays hold `count` of them, all alive for
			// the call; the instance it returns holds its own copies.
			let instance = unsafe {
				DnsServiceConstructInstance(
					PCWSTR(name.as_ptr()),
					PCWSTR(host.as_ptr()),
					Some(&raw const ip),
					None,
					record.port,
					0,
					0,
					count,
					keys.as_ptr(),
					values.as_ptr(),
				)
			};
			if instance.is_null() {
				return Err("DnsServiceConstructInstance gave none".into());
			}
			let (sender, done) = mpsc::channel();
			let context = Box::into_raw(Box::new(sender));
			let request = Box::into_raw(Box::new(DNS_SERVICE_REGISTER_REQUEST {
				Version: DNS_QUERY_REQUEST_VERSION1.0,
				InterfaceIndex: 0, // every interface
				pServiceInstance: instance,
				pRegisterCompletionCallback: Some(completed),
				pQueryContext: context.cast(),
				hCredentials: HANDLE::default(),
				unicastEnabled: BOOL::from(false), // mDNS, never a DNS server
			}));
			let registration = Self { request, context, done, confirmed: false };
			// SAFETY: the request, its instance and the context stay alive until Windows' last callback for them.
			let status = unsafe { DnsServiceRegister(request, None) };
			if status != DNS_REQUEST_PENDING.unsigned_abs() {
				registration.free(); // no callback comes for a request Windows did not take
				return Err(format!("DnsServiceRegister: {status}"));
			}
			registration.confirm(record)
		}

		/// Waits for Windows' answer to the registration: kept when it took it (or has not answered yet), freed when
		/// it refused it, since no other callback comes then.
		fn confirm(mut self, record: &Record) -> Result<Self, String> {
			match self.done.recv_timeout(CONFIRM) {
				Ok(0) => {
					self.confirmed = true;
					log::info!("bonjour: {} at {}:{}", record.instance, record.ip, record.port);
					Ok(self)
				}
				Ok(status) => {
					self.free();
					Err(format!("Windows refused it: {status}"))
				}
				Err(_) => {
					log::warn!("bonjour: Windows has not confirmed {} yet", record.instance);
					Ok(self)
				}
			}
		}

		/// Withdraws the record: once Windows confirmed the registration and then the withdrawal, everything is freed;
		/// a confirmation that does not come leaves it all to Windows.
		fn withdraw(self) {
			if !self.confirmed {
				match self.done.recv_timeout(CONFIRM) {
					Ok(0) => {}
					Ok(_) => return self.free(), // refused late: no other callback comes
					Err(_) => {
						log::warn!("bonjour: Windows never confirmed the registration; it ends with the helper");
						return;
					}
				}
			}
			// SAFETY: the request Windows registered, alive until its withdrawal is confirmed.
			let status = unsafe { DnsServiceDeRegister(self.request, None) };
			if status != DNS_REQUEST_PENDING.unsigned_abs() {
				log::warn!("bonjour: DnsServiceDeRegister: {status}; the record ends with the helper");
				return;
			}
			if self.done.recv_timeout(CONFIRM).is_ok() {
				self.free();
			} else {
				log::warn!("bonjour: Windows did not confirm the withdrawal in time");
			}
		}

		/// Frees the request, its instance and the context: only once no callback can come for them any more.
		fn free(self) {
			// SAFETY: the request and the context came from `Box::into_raw` and the instance from
			// DnsServiceConstructInstance; Windows took none of them or sent its last callback for them, and each is
			// freed once, here.
			unsafe {
				let request = Box::from_raw(self.request);
				DnsServiceFreeInstance(request.pServiceInstance);
				drop(Box::from_raw(self.context));
			}
		}
	}
}
