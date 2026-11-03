//! The heart-rate strap and the walking pad over Bluetooth (docs/plan.md slices 8 and 25; rust-helper `bluetooth.md`).
//! The task starts only when the player turns a strap or a pad on in the helper's window, because the first
//! `adapters()` call raises the Bluetooth prompt on macOS, so a player with neither never sees it; the adapter is made
//! once per process (each one is a `CoreBluetooth` thread that never ends), and one run at a time uses it. A run scans
//! for the services of the devices wanted (heart rate 0x180D: the WHOOP's broadcast, a Polar H10; Fitness Machine
//! 0x1826: a walking pad) and keeps a slot for each kind: a slot attaches the first device of its kind it finds, one at
//! a time and never one the other slot holds, and passes on what it reads: a strap's Heart Rate Measurement (0x2A37),
//! a pad's Treadmill Data records (0x2ACD) and status events (0x2ADA), while the pad's own task runs the loop's
//! commands on its control point (0x2AD9), one procedure at a time. A device the system keeps connected advertises
//! nothing (a strap Windows paired), so a free slot also asks the system for its connected devices of the slot's kind.
//! A device that drops, or a strap that sends no usable reading for `SILENT`, is let go: disconnected, then the
//! peripherals forgotten (btleplug keeps a let-go device's old handle on macOS, which would read nothing when the
//! device comes back), so the scan finds it again. Forgetting cancels every connection on macOS, so there, while the
//! other slot holds a device, the let-go one is only disconnected, and should it come back first, the other is let go
//! too (at most once a minute, once in five for a strap let go for its silence), the peripherals forgotten, and both
//! attach again; Windows forgets at once. Refused Bluetooth or a missing adapter stops only this task, a slot no longer
//! wanted lets its device go, and a pad command the task never carried out is refused.

use std::collections::{HashMap, HashSet};
use std::pin::Pin;
use std::time::Duration;

use btleplug::api::bleuuid::uuid_from_u16;
use btleplug::api::{
	Central as _, CentralEvent, CentralState, Characteristic, Manager as _, Peripheral as _,
	RetrievePeripheralsOptions, ScanFilter, ValueNotification, WriteType,
};
use btleplug::platform::{Adapter, Manager, Peripheral, PeripheralId};
use futures_util::StreamExt as _;
use questpump_core::gatt::{self, HeartMeasurement, MachineStatus, cp};
use tokio::sync::{Mutex, OnceCell, mpsc, watch};
use tokio::task::{JoinHandle, JoinSet};
use tokio::time::{Instant, interval_at, timeout};
use uuid::Uuid;

const SVC_HR: Uuid = uuid_from_u16(0x180D);
const CHR_HRM: Uuid = uuid_from_u16(0x2A37);
const SVC_FTMS: Uuid = uuid_from_u16(0x1826);
const CHR_TREADMILL: Uuid = uuid_from_u16(0x2ACD);
const CHR_FTMS_FEATURE: Uuid = uuid_from_u16(0x2ACC);
const CHR_SPEED_RANGE: Uuid = uuid_from_u16(0x2AD4);
const CHR_FTMS_CONTROL: Uuid = uuid_from_u16(0x2AD9);
const CHR_FTMS_STATUS: Uuid = uuid_from_u16(0x2ADA);
/// The log's names for the pad's optional parts.
const CONTROL_POINT: &str = "control point"; // copy-lint: not shown (a log line's label)
const SPEED_RANGE: &str = "speed range"; // copy-lint: not shown (a log line's label)
/// No device of a slot's kind read for this long while scanning: the window says how to make one broadcast, and says
/// it again as long as none is read.
const NOT_FOUND: Duration = Duration::from_secs(30);
/// A connection, a service discovery, a disconnect or a pad's optional GATT call that hangs (a device gone from range)
/// gives up after this.
const GATT_TIMEOUT: Duration = Duration::from_secs(10);
/// An attached strap with no usable reading for this long is let go and looked for again (a strap reports once a
/// second; a WHOOP 5.0 hit by its broadcast bug sends only 0 bpm). A pad is never let go for its silence: it notifies
/// Treadmill Data while it is in use (FTMS 1.0), so a stopped belt may say nothing, and a pad gone shows as a
/// disconnect.
const SILENT: Duration = Duration::from_secs(10);
/// A device that failed to attach is not tried again for this long, so another one nearby gets its turn.
const FAILED_WAIT: Duration = Duration::from_secs(60);
/// A bounce lets the other slot's working device go: at most one in this long, so a device that keeps dropping or
/// going silent beside it costs that one a reconnect at most once a minute.
const BOUNCE_WAIT: Duration = Duration::from_secs(60);
/// A strap let go for its silence while the pad is read brings no bounce for this long: a WHOOP 5.0 hit by its
/// broadcast bug advertises again at once and sends 0 bpm again, and each bounce costs the pad a reconnect.
const SILENT_BOUNCE_WAIT: Duration = Duration::from_mins(5);
/// How often the task checks for a silent strap and a refused grant.
const CHECK: Duration = Duration::from_secs(2);
/// Asking the system for its connected devices gives up after this (Windows reads each connected device's cached
/// services, 5 s for one gone stale).
const RETRIEVE_TIMEOUT: Duration = Duration::from_secs(15);
/// A control point request the pad has not answered in this long has failed.
const RESPONSE_WAIT: Duration = Duration::from_secs(3);

/// The process's one adapter, made at the first strap-on or pad-on.
static ADAPTER: OnceCell<Adapter> = OnceCell::const_new();
/// One run at a time on the shared adapter: a run turned off tears its scan down before the next one starts.
static RUN: Mutex<()> = Mutex::const_new(());

/// The kinds of device the task reads, one slot each.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
	/// A heart-rate strap: the standard heart-rate service.
	Strap,
	/// A walking pad: the Fitness Machine service with Treadmill Data.
	Pad,
}

impl Kind {
	const ALL: [Self; 2] = [Self::Strap, Self::Pad];

	/// The service a device of this kind offers, which the scan and the system's connected devices are searched by.
	fn service(self) -> Uuid {
		match self {
			Self::Strap => SVC_HR,
			Self::Pad => SVC_FTMS,
		}
	}

	fn other(self) -> Self {
		match self {
			Self::Strap => Self::Pad,
			Self::Pad => Self::Strap,
		}
	}

	/// The kind's word in the log.
	fn word(self) -> &'static str {
		match self {
			Self::Strap => "strap",
			Self::Pad => "pad",
		}
	}
}

/// Which devices the run reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Want {
	pub strap: bool,
	pub pad: bool,
}

impl Want {
	/// Whether any device is wanted: with none the task has nothing to read.
	#[must_use]
	pub fn any(self) -> bool {
		self.strap || self.pad
	}

	fn has(self, kind: Kind) -> bool {
		match kind {
			Kind::Strap => self.strap,
			Kind::Pad => self.pad,
		}
	}

	fn kinds(self) -> impl Iterator<Item = Kind> {
		Kind::ALL.into_iter().filter(move |&kind| self.has(kind))
	}
}

/// What a device slot is doing, for the window and the `status` line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum State {
	Searching,
	/// None read for `NOT_FOUND`, and still looking.
	NotFound,
	/// Reading the device with this name; none for a device that gives none (the page words it).
	Connected(Option<String>),
	/// Bluetooth neither on nor off yet: the prompt is up, or it resets.
	Waiting,
	BluetoothOff,
	/// Bluetooth was refused: the task stopped.
	Refused,
	/// No adapter, or Bluetooth failed: the task stopped.
	Failed(String),
}

/// A command for the pad (`pad.speed`, `pad.stop`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PadCommand {
	/// Run the belt at this many km/h, fitted into the pad's Supported Speed Range, and start it when it stands.
	Speed(f64),
	/// Stop the belt.
	Stop,
}

impl PadCommand {
	/// The command's word in a refusal's detail (`speed`, `stop`).
	#[must_use]
	pub fn word(self) -> &'static str {
		match self {
			Self::Speed(_) => "speed",
			Self::Stop => "stop",
		}
	}

	/// The refusal of a command with no pad to take it.
	fn no_pad(self) -> Ble {
		Ble::PadRefused(format!("{}: no pad", self.word()))
	}
}

/// What the task tells the loop.
#[derive(Debug, Clone, PartialEq)]
pub enum Ble {
	/// What a slot is doing.
	State(Kind, State),
	/// A strap's Heart Rate Measurement, every notification (the verdict takes only its `reading()`).
	Heart(HeartMeasurement),
	/// One Treadmill Data record from the pad, about once a second.
	Pad(gatt::Treadmill),
	/// A Fitness Machine Status event from the pad.
	PadStatus(MachineStatus),
	/// A pad command that did not go through, and why: the `pad_event` `refused` detail, the command's word and the
	/// pad's answer or the reason ("speed: timed out", "stop: no control point").
	PadRefused(String),
}

/// The running task: dropping it or calling `stop` disconnects its devices and ends the scan.
pub struct Task {
	stop: watch::Sender<bool>,
	want: watch::Sender<Want>,
	commands: mpsc::UnboundedSender<PadCommand>,
	/// Where a command answers itself once the run has ended.
	sink: mpsc::UnboundedSender<Ble>,
}

impl Task {
	/// Which devices the run reads now: a slot no longer wanted lets its device go (disconnected) and says nothing
	/// more.
	pub fn want(&self, want: Want) {
		self.want.send_replace(want);
	}

	/// A command for the attached pad; with no pad attached it comes back as `Ble::PadRefused("<speed|stop>: no pad")`.
	pub fn pad(&self, command: PadCommand) {
		if self.commands.send(command).is_err() {
			let _ = self.sink.send(command.no_pad());
		}
	}

	pub fn stop(&self) {
		let _ = self.stop.send(true);
	}
}

impl Drop for Task {
	fn drop(&mut self) {
		self.stop();
	}
}

/// Starts the task on the app's runtime; it waits for a run turned off a moment ago to finish first.
pub fn start(sink: mpsc::UnboundedSender<Ble>, want: Want) -> Task {
	let (stop, stopped) = watch::channel(false);
	let (want, wanted) = watch::channel(want);
	let (commands, commands_rx) = mpsc::unbounded_channel();
	let task = Task { stop, want, commands, sink: sink.clone() };
	tokio::spawn(async move {
		let mut commands = commands_rx;
		let _one = RUN.lock().await;
		if !*stopped.borrow() {
			let wanted_at_end = wanted.clone();
			if let Err(e) = run(&sink, stopped, wanted, &mut commands).await {
				let state = match e {
					btleplug::Error::PermissionDenied => State::Refused,
					other => State::Failed(other.to_string()),
				};
				let want = *wanted_at_end.borrow();
				for kind in want.kinds() {
					let _ = sink.send(Ble::State(kind, state.clone()));
				}
			}
		}
		// a command the run never took (it ended, failed or never began) is refused; one sent from now on comes back
		// from `Task::pad` itself
		commands.close();
		while let Ok(command) = commands.try_recv() {
			let _ = sink.send(command.no_pad());
		}
	});
	task
}

/// The process's adapter; the Bluetooth prompt comes at the first call. An answer given before the process started
/// refuses here and leaves it unmade (a later call asks macOS again, which then refuses at once, without a thread);
/// an answer given at the prompt comes later, as `CoreBluetooth`'s unauthorized state on the adapter kept here.
async fn adapter() -> btleplug::Result<Adapter> {
	ADAPTER
		.get_or_try_init(|| async {
			let manager = Manager::new().await?;
			manager.adapters().await?.into_iter().next().ok_or(btleplug::Error::NoAdapterAvailable)
		})
		.await
		.cloned()
}

/// Whether macOS refused Bluetooth to the helper (the prompt answered no, or a policy): `CoreBluetooth` then reports
/// its state as unauthorized, which btleplug reads as `Unknown`. While the prompt is still up the answer is not known.
#[cfg(target_os = "macos")]
fn refused() -> bool {
	use objc2_core_bluetooth::{CBManager, CBManagerAuthorization};
	// SAFETY: a class property read that takes no arguments and may be called from any thread (Apple's
	// `CBManager.authorization`, macOS 10.15 and later; the helper needs 14).
	let status = unsafe { CBManager::authorization_class() };
	status == CBManagerAuthorization::Denied || status == CBManagerAuthorization::Restricted
}

#[cfg(not(target_os = "macos"))]
fn refused() -> bool {
	false
}

/// What an attaching device's own task reports.
enum Attach {
	Connected(Kind, PeripheralId, Option<String>),
	Heart(PeripheralId, HeartMeasurement),
	Pad(PeripheralId, gatt::Treadmill),
	PadStatus(PeripheralId, MachineStatus),
	/// A pad command that did not go through, with the refusal's detail.
	PadRefused(String),
	/// It could not connect or subscribe.
	Failed(Kind, PeripheralId),
	/// It lacks the kind's characteristic (Heart Rate Measurement, Treadmill Data): never tried again in that slot
	/// while the task runs.
	NotOurs(Kind, PeripheralId),
}

/// The device a slot is attaching or reading.
struct Attached {
	id: PeripheralId,
	peripheral: Peripheral,
	task: JoinHandle<()>,
	connected: bool,
	/// A strap's last usable reading, or the attach's start (`SILENT`).
	heard: Instant,
	/// The pad's own task's commands; none for a strap.
	commands: Option<mpsc::UnboundedSender<PadCommand>>,
}

/// One kind's part of the run: its device, and the devices it does not try.
struct Slot {
	attached: Option<Attached>,
	/// Devices without the kind's characteristic.
	not_ours: HashSet<PeripheralId>,
	failed: HashMap<PeripheralId, Instant>,
	/// The "not found" count: from the scan's start, the slot's turn-on or a real drop.
	quiet: tokio::time::Interval,
}

impl Slot {
	fn new() -> Self {
		Self {
			attached: None,
			not_ours: HashSet::new(),
			failed: HashMap::new(),
			quiet: interval_at(Instant::now() + NOT_FOUND, NOT_FOUND),
		}
	}

	/// Whether the slot may try this device now: it holds none, and the device neither proved to be of another kind
	/// nor failed a moment ago.
	fn takes(&self, id: &PeripheralId) -> bool {
		self.attached.is_none()
			&& !self.not_ours.contains(id)
			&& self.failed.get(id).is_none_or(|t| t.elapsed() >= FAILED_WAIT)
	}

	fn holds(&self, id: &PeripheralId) -> bool {
		self.attached.as_ref().is_some_and(|a| a.id == *id)
	}

	/// A slot turned on again starts as a new run's would: every device tried afresh, "not found" counted from now.
	fn renew(&mut self) {
		self.not_ours.clear();
		self.failed.clear();
		self.quiet.reset();
	}
}

/// A device let go while the other slot held one (`Scan::stale`): the slot it was in, and the moment from which its
/// return may bounce the other device.
struct Stale {
	kind: Kind,
	from: Instant,
}

/// The devices the system keeps connected, by the kind each was asked for, the pad's first (`Scan::retrieve`), with the
/// count of forgettings the ask began after: a forgetting since makes its handles stale.
type Retrieved = (u32, Vec<(Kind, Vec<Peripheral>)>);

/// What the scan loop hears besides the adapter's events: its devices' tasks, its own background work and the loop's
/// choices.
struct Inbox<'a> {
	attach: mpsc::UnboundedReceiver<Attach>,
	cleared: mpsc::UnboundedReceiver<()>,
	retrieved: mpsc::UnboundedReceiver<Retrieved>,
	/// The task's own: `start` refuses what is left in it when the run ends.
	commands: &'a mut mpsc::UnboundedReceiver<PadCommand>,
	stopped: watch::Receiver<bool>,
	want: watch::Receiver<Want>,
}

/// The scan and the attaches, as one loop: at most one device a slot.
struct Scan<'a> {
	central: Adapter,
	sink: &'a mpsc::UnboundedSender<Ble>,
	want: Want,
	strap: Slot,
	pad: Slot,
	/// Devices let go on macOS while the other slot held one, so the peripherals were not forgotten: their old handles
	/// read nothing once they come back, and btleplug announces them again only by a name (`kind_for`).
	stale: HashMap<PeripheralId, Stale>,
	/// The last bounce (`bounce`).
	bounced: Option<Instant>,
	/// Forgettings of the peripherals that run (each after its let-go devices' disconnects): no attach until they are
	/// done.
	clearing: u32,
	cleared: mpsc::UnboundedSender<()>,
	attach: mpsc::UnboundedSender<Attach>,
	retrieved: mpsc::UnboundedSender<Retrieved>,
	/// An ask for the system's connected devices runs.
	retrieving: bool,
	/// An ask was skipped (one ran, or a forgetting did) or came back stale: ask again once nothing stands in its way.
	reask: bool,
	/// The forgettings of the peripherals so far, which tell a retrieved handle from before one.
	forgotten: u32,
	scanning: bool,
	/// The adapter's last state; `Unknown` while the prompt is up, it resets, or it refused (`refused` tells which).
	power: CentralState,
}

impl Scan<'_> {
	fn slot(&self, kind: Kind) -> &Slot {
		match kind {
			Kind::Strap => &self.strap,
			Kind::Pad => &self.pad,
		}
	}

	fn slot_mut(&mut self, kind: Kind) -> &mut Slot {
		match kind {
			Kind::Strap => &mut self.strap,
			Kind::Pad => &mut self.pad,
		}
	}

	fn is(&self, kind: Kind, id: &PeripheralId) -> bool {
		self.slot(kind).holds(id)
	}

	/// The slot that holds this device, if one does.
	fn holder(&self, id: &PeripheralId) -> Option<Kind> {
		Kind::ALL.into_iter().find(|&kind| self.is(kind, id))
	}

	/// Whether a slot is wanted and may try this device now.
	fn open(&self, kind: Kind, id: &PeripheralId) -> bool {
		self.want.has(kind) && self.slot(kind).takes(id)
	}

	fn say(&self, kind: Kind, state: State) {
		let _ = self.sink.send(Ble::State(kind, state));
	}

	/// The adapter's state, for every slot wanted.
	fn say_all(&self, state: &State) {
		for kind in self.want.kinds() {
			self.say(kind, state.clone());
		}
	}

	fn services(&self) -> Vec<Uuid> {
		self.want.kinds().map(Kind::service).collect()
	}

	/// Starts the scan for the wanted slots' services, when it does not run: every slot's "not found" count starts with
	/// it. Each wanted slot says it searches, and asks for a device the system keeps connected.
	async fn scan(&mut self) -> btleplug::Result<()> {
		let services = self.services();
		if services.is_empty() {
			return Ok(());
		}
		if !self.scanning {
			self.central.start_scan(ScanFilter { services }).await?;
			self.scanning = true;
			self.strap.quiet.reset();
			self.pad.quiet.reset();
		}
		for kind in self.want.kinds() {
			self.say(kind, State::Searching);
		}
		self.retrieve();
		Ok(())
	}

	/// The loop's new choice of devices: a slot no longer wanted lets its device go and says nothing more, a slot
	/// wanted again starts afresh, and the scan starts again on the new services.
	async fn rewant(&mut self, want: Want) -> btleplug::Result<()> {
		let was = std::mem::replace(&mut self.want, want);
		if was == want {
			return Ok(());
		}
		let mut added = Vec::new();
		for kind in Kind::ALL {
			match (was.has(kind), want.has(kind)) {
				(true, false) => self.let_go(kind, true, Duration::ZERO),
				(false, true) => {
					self.slot_mut(kind).renew();
					added.push(kind);
				}
				_ => {}
			}
		}
		if self.power == CentralState::PoweredOn {
			// stopped before it starts on the new services: never a start over a running scan
			if self.scanning {
				self.central.stop_scan().await?;
				self.scanning = false;
			}
			let services = self.services();
			if !services.is_empty() {
				self.central.start_scan(ScanFilter { services }).await?;
				self.scanning = true;
			}
		}
		for kind in added {
			match self.power {
				CentralState::PoweredOn => self.say(kind, State::Searching),
				CentralState::PoweredOff => self.say(kind, State::BluetoothOff),
				CentralState::Unknown => self.say(kind, State::Waiting),
			}
		}
		self.retrieve();
		Ok(())
	}

	/// Lets a slot's device go: its task ends, and the device is disconnected (`release`), then the peripherals are
	/// forgotten. On macOS only while the other slot holds no device, because there forgetting cancels every connection
	/// (btleplug 0.13.3 `corebluetooth/internal.rs`); otherwise the device is stale until the next forgetting, and its
	/// return bounces the other device no sooner than `wait` from now. On Windows forgetting only empties btleplug's
	/// list of peripherals and a kept handle connects again, so it always forgets and never bounces.
	fn let_go(&mut self, kind: Kind, disconnect: bool, wait: Duration) {
		let Some(a) = self.slot_mut(kind).attached.take() else { return };
		a.task.abort();
		let forget = cfg!(not(target_os = "macos")) || self.slot(kind.other()).attached.is_none();
		if !forget {
			self.stale.insert(a.id, Stale { kind, from: Instant::now() + wait });
		}
		self.release(vec![a.peripheral], disconnect, forget);
	}

	/// Lets both slots' devices go and forgets the peripherals.
	fn let_go_all(&mut self, disconnect: bool) {
		let mut devices = Vec::new();
		for kind in Kind::ALL {
			if let Some(a) = self.slot_mut(kind).attached.take() {
				a.task.abort();
				devices.push(a.peripheral);
			}
		}
		if !devices.is_empty() {
			self.release(devices, disconnect, true);
		}
	}

	/// In one task of their own (a disconnect can hang on a device gone from range, and the loop never waits on it)
	/// the let-go devices are disconnected, a connect still pending cancelled, then with `forget` the peripherals
	/// forgotten, in that order, so the disconnect still finds them.
	fn release(&mut self, devices: Vec<Peripheral>, disconnect: bool, forget: bool) {
		if !disconnect && !forget {
			return;
		}
		if forget {
			self.clearing += 1;
			self.forgotten = self.forgotten.wrapping_add(1);
		}
		let (central, cleared) = (self.central.clone(), self.cleared.clone());
		tokio::spawn(async move {
			if disconnect {
				let mut ends = JoinSet::new();
				for p in devices {
					ends.spawn(async move {
						let _ = timeout(GATT_TIMEOUT, p.disconnect()).await;
					});
				}
				while ends.join_next().await.is_some() {}
			}
			if forget {
				let _ = central.clear_peripherals().await;
				let _ = cleared.send(());
			}
		});
	}

	/// A device let go while the other slot held one is back, and its old handle would read nothing (macOS) until the
	/// peripherals are forgotten, which cancels the other slot's connection there too: so the other device is let go
	/// as well, the peripherals forgotten, and both attach again on their next discovery.
	fn bounce(&mut self) {
		if self.bounced.is_some_and(|t| t.elapsed() < BOUNCE_WAIT) {
			return;
		}
		self.bounced = Some(Instant::now());
		for kind in Kind::ALL {
			if self.slot(kind).attached.is_some() {
				self.slot_mut(kind).quiet.reset(); // a device read until now went: "not found" only after a while
				self.say(kind, State::Searching);
			}
		}
		self.let_go_all(true);
	}

	/// Asks the system for the devices of the free slots' kinds that it keeps connected (one that advertises nothing,
	/// as a strap Windows paired), in a task of its own (Windows reads each connected device's cached services): one
	/// ask per kind, since `WinRT` names no device's services, the pad's first, so a treadmill that also offers a heart
	/// rate goes to the pad (`on_retrieved`).
	fn retrieve(&mut self) {
		let kinds: Vec<Kind> = [Kind::Pad, Kind::Strap]
			.into_iter()
			.filter(|&kind| self.want.has(kind) && self.slot(kind).attached.is_none())
			.collect();
		if !self.scanning || kinds.is_empty() {
			return;
		}
		if self.retrieving || self.clearing > 0 {
			self.reask = true;
			return;
		}
		(self.retrieving, self.reask) = (true, false);
		let (central, retrieved, forgotten) = (self.central.clone(), self.retrieved.clone(), self.forgotten);
		tokio::spawn(async move {
			let mut all = Vec::new();
			for kind in kinds {
				let options = RetrievePeripheralsOptions { identifiers: None, services: Some(vec![kind.service()]) };
				let found = match timeout(RETRIEVE_TIMEOUT, central.retrieve_peripherals(options)).await {
					Ok(Ok(found)) => found,
					Ok(Err(e)) => {
						log::warn!("{}: the system's connected devices: {e}", kind.word());
						Vec::new()
					}
					Err(_) => {
						log::warn!("{}: the system's connected devices: timed out", kind.word());
						Vec::new()
					}
				};
				all.push((kind, found));
			}
			let _ = retrieved.send((forgotten, all));
		});
	}

	/// Which slot takes a device found advertising `services`: the pad's for the Fitness Machine service while the pad
	/// is wanted (a treadmill that also relays a heart rate is the pad, unless it proved to be none), else the strap's
	/// for the heart-rate service. With no service known yet (on macOS the advertised services follow the first
	/// sight, and a stale device, announced again only by a name, keeps the services its old handle saw, none for one
	/// the system kept connected), a stale device goes to the slot it was in, and otherwise the scan's filter vouches
	/// for the service it looks for, when it looks for one only.
	fn kind_for(&self, id: &PeripheralId, services: &[Uuid]) -> Option<Kind> {
		let kind = if services.is_empty() {
			match (self.stale.get(id).map(|stale| stale.kind), self.want.strap, self.want.pad) {
				(Some(kind), ..) => kind,
				(None, true, false) => Kind::Strap,
				(None, false, true) => Kind::Pad,
				_ => return None,
			}
		} else if self.want.pad && services.contains(&SVC_FTMS) && !self.pad.not_ours.contains(id) {
			Kind::Pad
		} else if services.contains(&SVC_HR) {
			Kind::Strap
		} else {
			return None;
		};
		self.open(kind, id).then_some(kind)
	}

	/// A device found by the scan.
	async fn on_found(&mut self, id: PeripheralId, advertised: Option<Vec<Uuid>>) {
		if !self.scanning
			|| self.clearing > 0
			|| self.holder(&id).is_some()
			|| !Kind::ALL.into_iter().any(|kind| self.open(kind, &id))
		{
			return;
		}
		let Ok(p) = self.central.peripheral(&id).await else { return };
		// the scan's filter (the system's on macOS, btleplug's own on Windows) found a wanted service already; the
		// advertised services and a name may come later than the first sight
		let props = p.properties().await.ok().flatten();
		let services = advertised.or_else(|| props.as_ref().map(|props| props.services.clone())).unwrap_or_default();
		let Some(kind) = self.kind_for(&id, &services) else { return };
		self.take(kind, id, p, props.and_then(|props| props.local_name));
	}

	/// What the system keeps connected of the free slots' kinds, taken as discovered devices: a device the pad's ask
	/// returned is the pad's while the pad is wanted and it has not proved to be none, never the strap's (as
	/// `kind_for` does for an advertised one).
	fn on_retrieved(&mut self, (forgotten, found): Retrieved) {
		self.retrieving = false;
		if forgotten != self.forgotten || self.clearing > 0 {
			self.reask = true; // handles from before a forgetting: CoreBluetooth no longer knows them
		} else {
			self.take_retrieved(found);
		}
		if self.reask {
			self.retrieve();
		}
	}

	/// The retrieved devices, taken as discovered ones; the pad's are never the strap's (`on_retrieved`).
	fn take_retrieved(&mut self, found: Vec<(Kind, Vec<Peripheral>)>) {
		let pads: HashSet<PeripheralId> = found
			.iter()
			.filter(|(kind, _)| *kind == Kind::Pad)
			.flat_map(|(_, list)| list.iter().map(btleplug::api::Peripheral::id))
			.collect();
		for (kind, list) in found {
			for p in list {
				let id = p.id();
				let a_pad =
					kind == Kind::Strap && self.want.pad && pads.contains(&id) && !self.pad.not_ours.contains(&id);
				if !a_pad && self.scanning && self.holder(&id).is_none() && self.open(kind, &id) {
					self.take(kind, id, p, None);
				}
			}
		}
	}

	/// Attaches a device in a slot, on a task of its own the loop owns and never waits on, unless its old handle is
	/// stale: then the bounce, once its wait is over.
	fn take(&mut self, kind: Kind, id: PeripheralId, p: Peripheral, name: Option<String>) {
		if let Some(stale) = self.stale.get(&id) {
			if stale.from <= Instant::now() {
				self.bounce();
			}
			return;
		}
		let attach = self.attach.clone();
		let (task, commands) = match kind {
			Kind::Strap => (tokio::spawn(attach_strap(p.clone(), id.clone(), name, attach)), None),
			Kind::Pad => {
				let (commands, received) = mpsc::unbounded_channel();
				(tokio::spawn(attach_pad(p.clone(), id.clone(), name, attach, received)), Some(commands))
			}
		};
		self.slot_mut(kind).attached =
			Some(Attached { id, peripheral: p, task, connected: false, heard: Instant::now(), commands });
	}

	async fn on_event(&mut self, event: CentralEvent) -> btleplug::Result<()> {
		match event {
			CentralEvent::DeviceDiscovered(id) | CentralEvent::DeviceUpdated(id) => self.on_found(id, None).await,
			CentralEvent::ServicesAdvertisement { id, services } => self.on_found(id, Some(services)).await,
			CentralEvent::DeviceDisconnected(id) => {
				if let Some(kind) = self.holder(&id) {
					self.let_go(kind, false, Duration::ZERO);
					// a device read until now dropped: "not found" only after a while without it
					self.slot_mut(kind).quiet.reset();
					self.say(kind, State::Searching);
				}
			}
			CentralEvent::StateUpdate(CentralState::PoweredOff) => {
				self.power = CentralState::PoweredOff;
				self.let_go_all(false);
				self.scanning = false;
				self.say_all(&State::BluetoothOff);
			}
			CentralEvent::StateUpdate(CentralState::PoweredOn) => {
				self.power = CentralState::PoweredOn;
				self.scanning = false; // a scan does not outlive the adapter's reset
				self.scan().await?;
			}
			CentralEvent::StateUpdate(CentralState::Unknown) => {
				self.let_go_all(false);
				self.scanning = false;
				self.power = CentralState::Unknown;
				self.say_all(&State::Waiting);
			}
			_ => {}
		}
		Ok(())
	}

	/// What an attach task reports; a report from an attach let go meanwhile changes nothing, but a pad's refusal still
	/// answers its command.
	fn on_attach(&mut self, a: Attach) {
		match a {
			Attach::Connected(kind, id, name) if self.is(kind, &id) => {
				if let Some(a) = self.slot_mut(kind).attached.as_mut() {
					a.connected = true;
					a.heard = Instant::now();
				}
				self.say(kind, State::Connected(name));
			}
			Attach::Heart(id, m) if self.is(Kind::Strap, &id) => {
				if let Some(a) = self.strap.attached.as_mut().filter(|_| m.reading().is_some()) {
					a.heard = Instant::now();
				}
				let _ = self.sink.send(Ble::Heart(m));
			}
			Attach::Pad(id, record) if self.is(Kind::Pad, &id) => {
				let _ = self.sink.send(Ble::Pad(record));
			}
			Attach::PadStatus(id, status) if self.is(Kind::Pad, &id) => {
				let _ = self.sink.send(Ble::PadStatus(status));
			}
			Attach::PadRefused(why) => {
				let _ = self.sink.send(Ble::PadRefused(why));
			}
			Attach::Failed(kind, id) if self.is(kind, &id) => {
				self.slot_mut(kind).failed.insert(id, Instant::now());
				self.let_go(kind, true, Duration::ZERO);
				self.say(kind, State::Searching);
			}
			Attach::NotOurs(kind, id) => {
				if self.is(kind, &id) {
					self.let_go(kind, true, Duration::ZERO);
					self.say(kind, State::Searching);
				}
				self.slot_mut(kind).not_ours.insert(id);
			}
			Attach::Connected(..)
			| Attach::Heart(..)
			| Attach::Pad(..)
			| Attach::PadStatus(..)
			| Attach::Failed(..) => {}
		}
	}

	/// A forgetting of the peripherals is done: no handle is stale now.
	fn on_cleared(&mut self) {
		self.clearing = self.clearing.saturating_sub(1);
		self.stale.clear();
		if self.clearing == 0 && self.reask {
			self.retrieve();
		}
	}

	/// A command for the pad: to the pad's own task while a pad is read, else refused at once.
	fn on_command(&self, command: PadCommand) {
		let pad = self.pad.attached.as_ref().filter(|a| a.connected).and_then(|a| a.commands.as_ref());
		if pad.is_none_or(|commands| commands.send(command).is_err()) {
			let _ = self.sink.send(command.no_pad());
		}
	}

	/// A slot's "not found" tick, every `NOT_FOUND` while the scan runs and the slot reads no device: the window says
	/// so again, and a free slot asks for a device the system keeps connected.
	fn on_quiet(&mut self, kind: Kind) {
		if self.scanning && self.want.has(kind) && self.slot(kind).attached.as_ref().is_none_or(|a| !a.connected) {
			self.say(kind, State::NotFound);
			self.retrieve();
		}
	}

	/// Every `CHECK`: a refused grant ends the run; a strap with no usable reading for `SILENT` is let go, and the
	/// window says how to make it broadcast.
	fn on_check(&mut self) -> btleplug::Result<()> {
		if self.power == CentralState::Unknown && refused() {
			return Err(btleplug::Error::PermissionDenied);
		}
		if self.strap.attached.as_ref().is_some_and(|a| a.connected && a.heard.elapsed() >= SILENT) {
			self.let_go(Kind::Strap, true, SILENT_BOUNCE_WAIT);
			self.say(Kind::Strap, State::NotFound);
		}
		Ok(())
	}

	async fn serve(
		&mut self,
		events: &mut (impl futures_util::Stream<Item = CentralEvent> + Unpin),
		inbox: &mut Inbox<'_>,
	) -> btleplug::Result<()> {
		self.power = self.central.adapter_state().await?;
		match self.power {
			CentralState::PoweredOn => self.scan().await?,
			CentralState::PoweredOff => self.say_all(&State::BluetoothOff),
			CentralState::Unknown => self.say_all(&State::Waiting),
		}
		let mut check = tokio::time::interval(CHECK);
		loop {
			tokio::select! {
				_ = inbox.stopped.changed() => return Ok(()),
				Ok(()) = inbox.want.changed() => {
					let want = *inbox.want.borrow_and_update();
					self.rewant(want).await?;
				}
				Some(event) = events.next() => self.on_event(event).await?,
				Some(a) = inbox.attach.recv() => self.on_attach(a),
				Some(()) = inbox.cleared.recv() => self.on_cleared(),
				Some(found) = inbox.retrieved.recv() => self.on_retrieved(found),
				Some(command) = inbox.commands.recv() => self.on_command(command),
				_ = self.strap.quiet.tick() => self.on_quiet(Kind::Strap),
				_ = self.pad.quiet.tick() => self.on_quiet(Kind::Pad),
				_ = check.tick() => self.on_check()?,
			}
		}
	}
}

async fn run(
	sink: &mpsc::UnboundedSender<Ble>,
	stopped: watch::Receiver<bool>,
	mut want: watch::Receiver<Want>,
	commands: &mut mpsc::UnboundedReceiver<PadCommand>,
) -> btleplug::Result<()> {
	let central = adapter().await?;
	if *stopped.borrow() {
		return Ok(()); // turned off while the prompt was up: the adapter stays for the next run
	}
	let mut events = central.events().await?; // before the scan, so no discovery is missed
	let (attach_tx, attach) = mpsc::unbounded_channel();
	let (cleared_tx, cleared) = mpsc::unbounded_channel();
	let (retrieved_tx, retrieved) = mpsc::unbounded_channel();
	let wanted = *want.borrow_and_update();
	let mut inbox = Inbox { attach, cleared, retrieved, commands, stopped, want };
	let mut s = Scan {
		central,
		sink,
		want: wanted,
		strap: Slot::new(),
		pad: Slot::new(),
		stale: HashMap::new(),
		bounced: None,
		clearing: 0,
		cleared: cleared_tx,
		attach: attach_tx,
		retrieved: retrieved_tx,
		retrieving: false,
		reask: false,
		forgotten: 0,
		scanning: false,
		power: CentralState::Unknown,
	};
	// the scan and the devices end with the run, whether it was turned off or failed
	let served = s.serve(&mut events, &mut inbox).await;
	s.let_go_all(true);
	// the next run may start as this one returns: its devices are disconnected and forgotten before
	while s.clearing > 0
		&& let Ok(Some(())) = timeout(GATT_TIMEOUT + CHECK, inbox.cleared.recv()).await
	{
		s.clearing -= 1;
	}
	if s.scanning {
		let _ = s.central.stop_scan().await;
	}
	// a pad command its task held when it was let go is refused as the task drops (`Commands`): the loop's own sender
	// goes first, so the channel ends once every device task has dropped its own
	drop(s);
	while let Ok(Some(a)) = timeout(GATT_TIMEOUT, inbox.attach.recv()).await {
		if let Attach::PadRefused(why) = a {
			let _ = sink.send(Ble::PadRefused(why));
		}
	}
	served
}

/// A connected device's notifications, of every characteristic subscribed.
type Notes = Pin<Box<dyn futures_util::Stream<Item = ValueNotification> + Send>>;

/// Connects to a device and discovers its services; its notification stream is taken before any subscribe, so no
/// notification is missed.
async fn open(p: &Peripheral) -> btleplug::Result<Notes> {
	p.connect_with_timeout(GATT_TIMEOUT).await?;
	p.discover_services_with_timeout(GATT_TIMEOUT).await?;
	p.notifications().await
}

/// What an attach that went wrong reports: a device without the kind's characteristic is not of that kind.
fn attach_failed(kind: Kind, id: PeripheralId, e: &btleplug::Error) -> Attach {
	log::warn!("{}: {e}", kind.word());
	match e {
		btleplug::Error::NoSuchCharacteristic => Attach::NotOurs(kind, id),
		_ => Attach::Failed(kind, id),
	}
}

/// The device's name: the one its advertisement gave, else the one the system knows by now.
async fn name_of(p: &Peripheral, name: Option<String>) -> Option<String> {
	match name {
		Some(n) => Some(n),
		None => p.properties().await.ok().flatten().and_then(|props| props.local_name),
	}
}

/// Connects to one strap and forwards its readings until the scan loop, which owns the task, ends it (btleplug's
/// notification stream never ends on its own: a drop is seen as `DeviceDisconnected`, or as `SILENT`). A GATT call on
/// a strap gone from range can block for seconds: the loop never waits on it, and disconnects it when it lets go.
async fn attach_strap(p: Peripheral, id: PeripheralId, name: Option<String>, attach: mpsc::UnboundedSender<Attach>) {
	let subscribed = async {
		let notes = open(&p).await?;
		let hrm =
			p.characteristics().into_iter().find(|c| c.uuid == CHR_HRM).ok_or(btleplug::Error::NoSuchCharacteristic)?;
		p.subscribe(&hrm).await?;
		btleplug::Result::Ok(notes)
	}
	.await;
	let mut notes = match subscribed {
		Ok(notes) => notes,
		Err(e) => {
			let _ = attach.send(attach_failed(Kind::Strap, id, &e));
			return;
		}
	};
	let name = name_of(&p, name).await;
	let _ = attach.send(Attach::Connected(Kind::Strap, id.clone(), name));
	while let Some(n) = notes.next().await {
		if n.uuid == CHR_HRM
			&& let Some(m) = gatt::parse_hrm(&n.value)
		{
			let _ = attach.send(Attach::Heart(id.clone(), m));
		}
	}
}

/// Connects to one pad: it subscribes to Treadmill Data (a device without it is no pad), to the pad's status and its
/// control point when it has them, and reads what the control needs; then, until the scan loop ends the task (as a
/// strap's), it passes on each record and status event and runs the loop's commands, one procedure at a time.
async fn attach_pad(
	p: Peripheral,
	id: PeripheralId,
	name: Option<String>,
	attach: mpsc::UnboundedSender<Attach>,
	commands: mpsc::UnboundedReceiver<PadCommand>,
) {
	let mut commands = Commands { queue: commands, running: None, attach: attach.clone() };
	let subscribed = async {
		let notes = open(&p).await?;
		let chars = p.characteristics();
		let find = |uuid: Uuid| chars.iter().find(|c| c.uuid == uuid);
		p.subscribe(find(CHR_TREADMILL).ok_or(btleplug::Error::NoSuchCharacteristic)?).await?;
		if let Some(status) = find(CHR_FTMS_STATUS) {
			optional("status", p.subscribe(status)).await;
		}
		let mut control = None;
		if let Some(chr) = find(CHR_FTMS_CONTROL)
			&& optional(CONTROL_POINT, p.subscribe(chr)).await.is_some()
		{
			let feature = read_optional(&p, find(CHR_FTMS_FEATURE), "feature").await;
			let range = read_optional(&p, find(CHR_SPEED_RANGE), SPEED_RANGE).await;
			control = Some(Control {
				chr: chr.clone(),
				speed_target: feature.and_then(|v| gatt::speed_target(&v)),
				range: range.and_then(|v| gatt::parse_speed_range(&v)),
				held: false,
			});
		}
		btleplug::Result::Ok((notes, control))
	}
	.await;
	let (mut notes, control) = match subscribed {
		Ok(subscribed) => subscribed,
		Err(e) => {
			let _ = attach.send(attach_failed(Kind::Pad, id, &e));
			return;
		}
	};
	let name = name_of(&p, name).await;
	let _ = attach.send(Attach::Connected(Kind::Pad, id.clone(), name));
	let mut pad = Pad {
		peripheral: p,
		id,
		attach,
		records: gatt::TreadmillRecords::default(),
		moving: false,
		halted: false,
		control,
	};
	loop {
		tokio::select! {
			n = notes.next() => {
				let Some(n) = n else { return };
				pad.note(&n);
			}
			Some(command) = commands.queue.recv() => {
				commands.running = Some(command);
				pad.run(command, &mut notes).await;
				commands.running = None;
			}
		}
	}
}

/// The commands a pad's task holds: the one it runs and the ones queued behind it. When the task ends (the pad let
/// go mid-procedure, a drop, the run's end), each is refused, so a socket client hears what became of every command.
struct Commands {
	queue: mpsc::UnboundedReceiver<PadCommand>,
	running: Option<PadCommand>,
	attach: mpsc::UnboundedSender<Attach>,
}

impl Drop for Commands {
	fn drop(&mut self) {
		let mut left: Vec<PadCommand> = self.running.take().into_iter().collect();
		while let Ok(command) = self.queue.try_recv() {
			left.push(command);
		}
		for command in left {
			let _ = self.attach.send(Attach::PadRefused(format!("{}: disconnected", command.word())));
		}
	}
}

/// One of a pad's optional GATT calls (its status, its control point and what the control reads): a failure, or a hang
/// past `GATT_TIMEOUT`, leaves that part out and is logged, and the distance still reads.
async fn optional<T>(what: &str, call: impl Future<Output = btleplug::Result<T>>) -> Option<T> {
	match timeout(GATT_TIMEOUT, call).await {
		Ok(Ok(value)) => Some(value),
		Ok(Err(e)) => {
			log::warn!("pad: {what}: {e}");
			None
		}
		Err(_) => {
			log::warn!("pad: {what}: timed out");
			None
		}
	}
}

/// Reads one of a pad's optional characteristics, when it has it.
async fn read_optional(p: &Peripheral, chr: Option<&Characteristic>, what: &str) -> Option<Vec<u8>> {
	optional(what, p.read(chr?)).await
}

/// A pad's control point, and what running it needs.
struct Control {
	chr: Characteristic,
	/// Whether the pad takes a target speed (Fitness Machine Feature, Target Setting bit 0); none when unread.
	speed_target: Option<bool>,
	/// The target speeds the pad takes (Supported Speed Range); none when unread.
	range: Option<gatt::SpeedRange>,
	/// Request Control succeeded on this connection, and since then no reset, no lost control and no request refused
	/// for want of it.
	held: bool,
}

/// An attached pad as its own task holds it.
struct Pad {
	peripheral: Peripheral,
	id: PeripheralId,
	attach: mpsc::UnboundedSender<Attach>,
	records: gatt::TreadmillRecords,
	/// The last record with a speed had one above 0.
	moving: bool,
	/// The belt is stopped: the last status said it stopped (by the user, the safety key or a reset) or paused, or the
	/// helper's own Stop went through, and neither a status nor the helper's own Start said since that it started.
	halted: bool,
	/// None for a pad without a control point.
	control: Option<Control>,
}

impl Pad {
	/// One notification but a control point answer: Treadmill Data parts make records, and the status events update
	/// the belt's state and the control; both go to the loop. A control point answer outside a procedure is a late
	/// one, its request given up.
	fn note(&mut self, n: &ValueNotification) {
		if n.uuid == CHR_TREADMILL {
			let Some(part) = gatt::parse_treadmill(&n.value) else { return };
			for record in self.records.push(part) {
				// a record without a speed (a pad that sends it apart) leaves the belt as the last one said
				if let Some(kmh) = record.kmh {
					self.moving = kmh > 0.0;
				}
				let _ = self.attach.send(Attach::Pad(self.id.clone(), record));
			}
		} else if n.uuid == CHR_FTMS_STATUS
			&& let Some(status) = gatt::parse_status(&n.value)
		{
			match status {
				MachineStatus::Started => self.halted = false,
				MachineStatus::Stopped | MachineStatus::Paused | MachineStatus::SafetyKey => self.halted = true,
				MachineStatus::Reset => {
					self.halted = true;
					self.hold(false);
				}
				MachineStatus::ControlLost => self.hold(false),
				MachineStatus::TargetSpeed(_) | MachineStatus::Other(_) => {}
			}
			let _ = self.attach.send(Attach::PadStatus(self.id.clone(), status));
		}
	}

	fn hold(&mut self, held: bool) {
		if let Some(control) = self.control.as_mut() {
			control.held = held;
		}
	}

	/// Whether the belt runs: the last record moved, and no status since said it stopped.
	fn running(&self) -> bool {
		self.moving && !self.halted
	}

	/// Runs one command; a refusal goes to the loop.
	async fn run(&mut self, command: PadCommand, notes: &mut Notes) {
		if let Err(why) = self.procedure(command, notes).await {
			let _ = self.attach.send(Attach::PadRefused(format!("{}: {why}", command.word())));
		}
	}

	/// A command's requests in turn: Request Control when control is not held, then for a speed Set Target Speed
	/// (fitted into the Supported Speed Range) and Start or Resume when the belt does not run, for a stop Stop. A pad
	/// without a control point refuses both, one whose feature says it takes no target speed refuses a speed, each
	/// without writing.
	async fn procedure(&mut self, command: PadCommand, notes: &mut Notes) -> Result<(), String> {
		let Some(control) = &self.control else { return Err("no control point".to_owned()) };
		if matches!(command, PadCommand::Speed(_)) && control.speed_target == Some(false) {
			return Err("no speed target".to_owned());
		}
		let (chr, range, held) = (control.chr.clone(), control.range, control.held);
		if !held {
			self.request(&chr, &cp::request_control(), notes).await?;
			self.hold(true);
		}
		match command {
			PadCommand::Speed(kmh) => {
				self.request(&chr, &cp::set_target_speed(range.map_or(kmh, |range| range.fit(kmh))), notes).await?;
				if !self.running() {
					self.request(&chr, &cp::start(), notes).await?;
					self.halted = false;
				}
			}
			PadCommand::Stop => {
				self.request(&chr, &cp::stop(), notes).await?;
				self.halted = true;
			}
		}
		Ok(())
	}

	/// Writes one request to the control point, with a response, and waits `RESPONSE_WAIT` for its Response Code
	/// indication while the pad's other notifications pass on: Ok on success, else the word of what went wrong (a
	/// "not permitted" also means control is no longer held).
	async fn request(&mut self, chr: &Characteristic, request: &[u8], notes: &mut Notes) -> Result<(), String> {
		let op = request.first().copied().unwrap_or_default();
		let answered = timeout(RESPONSE_WAIT, async {
			if let Err(e) = self.peripheral.write(chr, request, WriteType::WithResponse).await {
				log::warn!("pad: control point: {e}");
				return Err("write failed".to_owned());
			}
			while let Some(n) = notes.next().await {
				if n.uuid == CHR_FTMS_CONTROL {
					if let Some((to, outcome)) = cp::parse_response(&n.value)
						&& to == op
					{
						return Ok(outcome);
					}
				} else {
					self.note(&n);
				}
			}
			Err("disconnected".to_owned())
		})
		.await;
		let outcome = answered.map_err(|_| "timed out".to_owned())??;
		if outcome == cp::Outcome::NotPermitted {
			self.hold(false);
		}
		if outcome == cp::Outcome::Success { Ok(()) } else { Err(outcome.word()) }
	}
}
