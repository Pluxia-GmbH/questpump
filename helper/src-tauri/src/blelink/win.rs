//! The Bluetooth link's peripheral on Windows (slice 43, with its fixes after the review of 2026-10-17; rust-helper
//! `bluetooth.md`, "The Bluetooth link's peripheral"): a `GattServiceProvider` with the service's three
//! characteristics, advertised connectable and discoverable with the service data `QP` 2 (a Windows peripheral
//! advertises no local name of its own). It runs only where the default adapter has the peripheral role
//! (`BluetoothAdapter.IsPeripheralRoleSupported`), else says `no_peripheral` and the phones link on Wi-Fi alone. The
//! handlers answer each central's read of the hello from `Hellos`, pass on `rx`'s writes with the writer's notification
//! size while it stands among `SubscribedClients` (answering a write with response), and diff the subscribed clients
//! for who came and went; a write and a read of the hello from its start read `SubscribedClients` first, so a first
//! write that comes before its subscription's change still finds it, and an unsubscribe Windows applied before a
//! hello's read is told before its challenge. Each central's messages go out in order on a task of its own
//! (`NotifyValueForSubscribedClientAsync`) from its `Queue`, a task made anew when its client is another object (the
//! central subscribed again) and dropped when the bridge lets the central go; the latency is the device's preferred
//! connection parameters where Windows 11 has them. A radio turned off (the radio's change, or a creation or an
//! advertising aborted with `RadioNotAvailable`) says `off`, and the service is published again once it is on; any
//! other failure of a publish or an advertising (an advertising `Aborted`, which Microsoft's sample calls a request it
//! could not submit or one cancelled for resource contention, or stopped by the system, or a provider that says no
//! `Started` `START_WAIT` after its advertising's start) is tried again after `super::backoff` and given up after five
//! in a row: a publish with a new provider; an advertising on the same provider at the first failure in a row, whose
//! service and subscribed centrals stay, and with a new provider from the second on (or when that call fails), as
//! Microsoft's sample makes one at each start. A new pairing code makes a new provider. Wherever a provider goes, its
//! handlers go first and the bridge hears each of its subscribed centrals go, as the Mac's state below powered on tells
//! it. Every read and write is answered (`UnlikelyError` when something of it does not read) and its deferral closed
//! however its handler ends. Nothing asks the player first, so the runtime may start it at any time.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use questpump_core::ble;
use questpump_core::link::Ble;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio::task::JoinHandle;
use tokio::time::Instant;
use windows::Devices::Bluetooth::GenericAttributeProfile::{
	GattCharacteristicProperties, GattCommunicationStatus, GattLocalCharacteristic, GattLocalCharacteristicParameters,
	GattProtectionLevel, GattProtocolError, GattReadRequestedEventArgs, GattServiceProvider,
	GattServiceProviderAdvertisementStatus, GattServiceProviderAdvertisementStatusChangedEventArgs,
	GattServiceProviderAdvertisingParameters, GattSession, GattSubscribedClient, GattWriteOption,
	GattWriteRequestedEventArgs,
};
use windows::Devices::Bluetooth::{
	BluetoothAdapter, BluetoothError, BluetoothLEDevice, BluetoothLEPreferredConnectionParameters,
	BluetoothLEPreferredConnectionParametersRequest,
};
use windows::Devices::Radios::{Radio as SystemRadio, RadioState};
use windows::Foundation::Metadata::ApiInformation;
use windows::Foundation::{Deferral, TypedEventHandler};
use windows::Storage::Streams::{DataReader, DataWriter, IBuffer};
use windows::core::{GUID, HSTRING, IInspectable, Ref};

use super::{Events, Hellos, Latency, Outgoing, Queue, Radio, backoff};

/// The Windows 11 method for a connection's preferred parameters (`BluetoothLEDevice`, build 22000).
const PREFERRED: (&str, &str) = ("Windows.Devices.Bluetooth.BluetoothLEDevice", "RequestPreferredConnectionParameters");
/// The advertisement's service data (`GattServiceProviderAdvertisingParameters.ServiceData`, build 18362).
const SERVICE_DATA: (&str, &str) =
	("Windows.Devices.Bluetooth.GenericAttributeProfile.GattServiceProviderAdvertisingParameters", "ServiceData");

/// How long after the radio read as off the service is tried again, whatever the radio's changes say.
const OFF_RETRY: Duration = Duration::from_secs(30);
/// How long after its advertising's start a provider may go without saying `Started` (`Publisher::check`): past it,
/// an advertising Windows accepted but never started, or whose status was never raised, counts as a failure.
const START_WAIT: Duration = Duration::from_secs(5);

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
	m.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Windows asks nothing before a GATT server starts.
pub(super) fn may_ask() -> bool {
	true
}

/// What the peripheral's task is asked.
enum Order {
	Send(String, Outgoing),
	Cancel(String, u8),
	Forget(String),
	Latency(String, Latency),
	Republish,
	Check,
	Quiet,
}

/// The peripheral: its task, which owns the provider; closing it stops advertising and lets the service go.
pub(super) struct Peripheral {
	orders: Option<UnboundedSender<Order>>,
	task: Option<JoinHandle<()>>,
}

impl Peripheral {
	pub(super) fn start(events: Events, hellos: Arc<Hellos>) -> Self {
		let (orders, rx) = mpsc::unbounded_channel();
		let task = tokio::spawn(run(events, hellos, rx));
		Self { orders: Some(orders), task: Some(task) }
	}

	fn order(&self, order: Order) {
		if let Some(orders) = &self.orders {
			let _ = orders.send(order);
		}
	}

	pub(super) fn send(&self, central: &str, message: Outgoing) {
		self.order(Order::Send(central.to_owned(), message));
	}

	pub(super) fn cancel(&self, central: &str, id: u8) {
		self.order(Order::Cancel(central.to_owned(), id));
	}

	/// The bridge let a central go: its notification task and preferred parameters go (its waiting messages with them);
	/// a write of it while it stays subscribed tells the bridge again.
	pub(super) fn forget(&self, central: &str) {
		self.order(Order::Forget(central.to_owned()));
	}

	pub(super) fn latency(&self, central: &str, latency: Latency) {
		self.order(Order::Latency(central.to_owned(), latency));
	}

	/// A new pairing code: a new provider.
	pub(super) fn republish(&self) {
		self.order(Order::Republish);
	}

	/// The bridge's tick: a provider that has not said `Started` `START_WAIT` after its advertising's start counts as a
	/// failure (`Publisher::check`); its stops come as `AdvertisementStatusChanged`.
	pub(super) fn check(&self) {
		self.order(Order::Check);
	}

	/// Let go: advertising stops at once, while the answers in flight still go to their centrals.
	pub(super) fn quiet(&self) {
		self.order(Order::Quiet);
	}

	/// The task stops, its provider with it, and this waits for that, so another made next never meets it.
	pub(super) async fn close(&mut self) {
		self.orders = None;
		if let Some(task) = self.task.take() {
			let _ = task.await;
		}
	}
}

/// The subscribed clients by central, each with the most a notification carries to it, as the handlers keep them.
type Clients = Arc<Mutex<HashMap<String, (GattSubscribedClient, usize)>>>;

/// What a central's notification task is told.
enum Note {
	Message(Outgoing),
	Cancel(u8),
	/// The bridge let the central go: what waits is dropped, and the task ends with its sender.
	Clear,
}

/// The published service and what serves it.
struct Served {
	provider: GattServiceProvider,
	/// Each characteristic with its handler's token, removed with the service: none of them tells the bridge more, and
	/// the two that hold `tx` (the hello's and `rx`'s) let it go.
	tx: (GattLocalCharacteristic, i64),
	hello: (GattLocalCharacteristic, i64),
	rx: (GattLocalCharacteristic, i64),
	clients: Clients,
	/// Each central's notification task, with the client it notifies.
	senders: HashMap<String, (GattSubscribedClient, UnboundedSender<Note>)>,
	/// Each central's device and its preferred connection parameters, kept while it is subscribed.
	preferred: HashMap<String, (BluetoothLEDevice, BluetoothLEPreferredConnectionParametersRequest)>,
}

impl Drop for Served {
	fn drop(&mut self) {
		let _ = self.provider.StopAdvertising();
		let _ = self.hello.0.RemoveReadRequested(self.hello.1);
		let _ = self.rx.0.RemoveWriteRequested(self.rx.1);
		let _ = self.tx.0.RemoveSubscribedClientsChanged(self.tx.1);
	}
}

/// What starting the service came to: published, or a state the helper block says, or a failure.
enum Start {
	Served(Served),
	State(Ble),
	Failed(String),
}

/// What a provider's advertising said, with the provider's number.
struct Advert {
	provider: u64,
	status: GattServiceProviderAdvertisementStatus,
	error: BluetoothError,
}

/// The task's state: the service while published, the radio, the retries.
struct Publisher {
	events: Events,
	hellos: Arc<Hellos>,
	adverts: UnboundedSender<Advert>,
	served: Option<Served>,
	/// Whether the radio is on as last heard (on where Windows gives no radio).
	on: bool,
	/// The publishes and advertisings that failed in a row, and when the next try is due.
	failures: u32,
	retry: Option<Instant>,
	/// When the current provider's advertising was started, until it says `Started` or its check looked
	/// (`Publisher::check`).
	advertised: Option<Instant>,
	/// The providers made so far, which numbers each: an old one's advertising says nothing.
	providers: u64,
	/// Let go by the bridge: nothing is published or advertised again.
	quiet: bool,
	/// It gave up, or the computer cannot serve phones: nothing changes that while it runs.
	done: bool,
}

async fn run(events: Events, hellos: Arc<Hellos>, mut orders: UnboundedReceiver<Order>) {
	// the radio's changes come here: off says so and drops the service, on publishes it again
	let (radio_tx, mut radio_rx) = mpsc::unbounded_channel::<bool>();
	let radio = system_radio(radio_tx).await;
	let (adverts, mut advert_rx) = mpsc::unbounded_channel();
	let on = radio.as_ref().is_none_or(|r| r.State().is_ok_and(|s| s == RadioState::On));
	let mut p = Publisher {
		events,
		hellos,
		adverts,
		served: None,
		on,
		failures: 0,
		retry: None,
		advertised: None,
		providers: 0,
		quiet: false,
		done: false,
	};
	if !on {
		p.events.send(Radio::State(Ble::Off));
	}
	loop {
		if p.served.is_none() && p.on && p.retry.is_none() && !p.quiet && !p.done {
			p.publish().await;
		}
		let due = p.retry;
		tokio::select! {
			order = orders.recv() => match order {
				Some(order) => p.order(order).await,
				None => break,
			},
			Some(now_on) = radio_rx.recv() => p.radio(now_on),
			Some(advert) = advert_rx.recv() => p.advert(&advert),
			() = tokio::time::sleep_until(due.unwrap_or_else(Instant::now)), if due.is_some() => p.due(),
		}
	}
	drop(p);
	drop(radio);
}

impl Publisher {
	/// A new provider with the service, its handlers and its advertising (`on` comes with its advertising's start).
	async fn publish(&mut self) {
		self.providers += 1;
		match start(&self.events, &self.hellos, &self.adverts, self.providers).await {
			Start::Served(served) => {
				self.served = Some(served);
				self.advertised = Some(Instant::now());
			}
			Start::State(Ble::Off) => self.off(),
			Start::State(state) => {
				self.events.send(Radio::State(state));
				self.done = true; // no peripheral role, no grant, no Bluetooth
			}
			Start::Failed(why) => self.trouble(&why),
		}
	}

	/// The service goes: its handlers first, so nothing of it tells the bridge more, then the bridge hears each central
	/// subscribed to it go, as the Mac's state below powered on tells it.
	fn unpublish(&mut self) {
		self.advertised = None;
		let Some(served) = self.served.take() else { return };
		let clients = served.clients.clone();
		drop(served);
		let gone: Vec<String> = lock(&clients).drain().map(|(central, _)| central).collect();
		for central in gone {
			self.events.send(Radio::Unsubscribed { central });
		}
	}

	/// The radio is off: the service goes, `off` is said, and the service is tried again at the radio's on, or after
	/// `OFF_RETRY` (a radio Windows gives no change of, or one that read as on while the provider found none).
	fn off(&mut self) {
		self.unpublish();
		self.events.send(Radio::State(Ble::Off));
		self.on = false;
		self.retry = Some(Instant::now() + OFF_RETRY);
	}

	/// A publish or an advertising failed: tried again after the backoff (`due`), or given up after `RETRIES` in a row,
	/// the service going then.
	fn trouble(&mut self, why: &str) {
		self.failures += 1;
		self.advertised = None;
		if let Some(wait) = backoff(self.failures) {
			log::warn!("ble: {why}; tried again in {} s", wait.as_secs());
			self.retry = Some(Instant::now() + wait);
		} else {
			self.unpublish();
			self.events.send(Radio::Failed(why.to_owned()));
			self.done = true;
		}
	}

	/// A retry is due: nothing once the bridge let it go or it gave up. An advertising that failed first in a row
	/// starts again on the same provider, which keeps its subscribed centrals; one that failed again, or a call that
	/// fails, goes with its provider, its centrals told gone, and the loop publishes a new one, as it does with none
	/// (a publish failed, or the radio read as off).
	fn due(&mut self) {
		self.retry = None;
		if self.quiet || self.done {
			return;
		}
		let Some(served) = &self.served else {
			self.on = true;
			return;
		};
		if self.failures > 1 {
			self.unpublish();
			return;
		}
		match advertise(&served.provider) {
			Ok(()) => self.advertised = Some(Instant::now()),
			Err(e) => {
				self.unpublish();
				// copy-lint: not shown (a log line)
				self.trouble(&format!("advertising could not start again: {}", e.message()));
			}
		}
	}

	/// The bridge's tick: the current provider's advertising, started `START_WAIT` ago with no `Started` heard, is
	/// looked at once (`AdvertisementStatus`): started, it says `on` as the event would; otherwise (still `Created`,
	/// or `Stopped` or `Aborted` with no event) it counts as a failure.
	fn check(&mut self) {
		let Some(at) = self.advertised else { return };
		if self.quiet || self.done || self.retry.is_some() || at.elapsed() < START_WAIT {
			return;
		}
		let Some(served) = &self.served else { return };
		self.advertised = None;
		match served.provider.AdvertisementStatus() {
			Ok(
				status @ (GattServiceProviderAdvertisementStatus::Started
				| GattServiceProviderAdvertisementStatus::StartedWithoutAllAdvertisementData),
			) => self.started(status),
			Ok(status) => {
				let (wait, status) = (START_WAIT.as_secs(), status.0);
				// copy-lint: not shown (a log line)
				self.trouble(&format!("advertising not started {wait} s after its start (status {status})"));
			}
			// copy-lint: not shown (a log line)
			Err(e) => self.trouble(&format!("advertising's status did not read: {}", e.message())),
		}
	}

	/// The current provider advertises: `on`, the failures in a row and a retry waiting ended.
	fn started(&mut self, status: GattServiceProviderAdvertisementStatus) {
		if status == GattServiceProviderAdvertisementStatus::StartedWithoutAllAdvertisementData {
			log::warn!("ble: advertising without all its data (the service data did not fit)");
		}
		self.failures = 0;
		self.retry = None;
		self.advertised = None;
		self.events.send(Radio::State(Ble::On));
	}

	/// The radio's change: off says so and drops the service, on publishes it again at once, a radio off being no
	/// failure; nothing once the computer cannot serve phones or it gave up.
	fn radio(&mut self, now_on: bool) {
		if self.done {
			return;
		}
		if self.on && !now_on {
			self.unpublish();
			self.events.send(Radio::State(Ble::Off));
		}
		if now_on && !self.on {
			self.retry = None;
			self.failures = 0;
		}
		self.on = now_on;
	}

	/// The current provider's advertising: started says `on`; aborted with the radio not available is `off`; aborted
	/// otherwise, or stopped by the system, is a failure tried again (`due`), unless one counted already (its check's)
	/// waits for its retry.
	fn advert(&mut self, advert: &Advert) {
		if advert.provider != self.providers || self.served.is_none() || self.quiet {
			return;
		}
		match advert.status {
			GattServiceProviderAdvertisementStatus::Started
			| GattServiceProviderAdvertisementStatus::StartedWithoutAllAdvertisementData => self.started(advert.status),
			GattServiceProviderAdvertisementStatus::Aborted if advert.error == BluetoothError::RadioNotAvailable => {
				self.off();
			}
			_ if self.retry.is_some() => {}
			GattServiceProviderAdvertisementStatus::Aborted => {
				// copy-lint: not shown (a log line)
				self.trouble(&format!("advertising aborted (Bluetooth error {})", advert.error.0));
			}
			// copy-lint: not shown (a log line)
			GattServiceProviderAdvertisementStatus::Stopped => self.trouble("advertising stopped by the system"),
			_ => {}
		}
	}

	/// An order of the bridge; one for a central while no service is published finds none (a message dropped, its
	/// `Done` telling).
	async fn order(&mut self, order: Order) {
		match order {
			Order::Republish => {
				self.unpublish();
				self.failures = 0;
				self.retry = None;
			}
			Order::Check => self.check(),
			Order::Quiet => {
				self.quiet = true;
				self.retry = None;
				if let Some(served) = &self.served {
					let _ = served.provider.StopAdvertising();
				}
			}
			Order::Send(central, message) => {
				if let Some(served) = &mut self.served {
					served.send(&central, message);
				}
			}
			Order::Cancel(central, id) => {
				if let Some(served) = &self.served {
					served.cancel(&central, id);
				}
			}
			Order::Forget(central) => {
				if let Some(served) = &mut self.served {
					served.forget(&central);
				}
			}
			Order::Latency(central, latency) => {
				if let Some(served) = &mut self.served {
					served.latency(central, latency).await;
				}
			}
		}
	}
}

/// The default adapter's radio, whose changes reach `changed` (on or not); none where Windows gives none.
async fn system_radio(changed: UnboundedSender<bool>) -> Option<SystemRadio> {
	let adapter = BluetoothAdapter::GetDefaultAsync().ok()?.await.ok()?;
	let radio = adapter.GetRadioAsync().ok()?.await.ok()?;
	let handler =
		TypedEventHandler::<SystemRadio, IInspectable>::new(move |radio: Ref<SystemRadio>, _: Ref<IInspectable>| {
			if let Some(radio) = radio.as_ref() {
				let _ = changed.send(radio.State()? == RadioState::On);
			}
			Ok(())
		});
	radio.StateChanged(&handler).ok()?;
	Some(radio)
}

/// The adapter checked, the service made with its characteristics and handlers, and its advertising started.
async fn start(events: &Events, hellos: &Arc<Hellos>, adverts: &UnboundedSender<Advert>, number: u64) -> Start {
	match publish(events, hellos, adverts, number).await {
		Ok(start) => start,
		Err(e) => Start::Failed(format!("the service could not be published: {}", e.message())),
	}
}

async fn publish(
	events: &Events,
	hellos: &Arc<Hellos>,
	adverts: &UnboundedSender<Advert>,
	number: u64,
) -> windows::core::Result<Start> {
	let Ok(adapter) = BluetoothAdapter::GetDefaultAsync()?.await else { return Ok(Start::State(Ble::Missing)) };
	if !adapter.IsLowEnergySupported()? {
		return Ok(Start::State(Ble::Missing));
	}
	if !adapter.IsPeripheralRoleSupported()? {
		return Ok(Start::State(Ble::NoPeripheral));
	}
	let made = GattServiceProvider::CreateAsync(GUID::from_u128(ble::SERVICE))?.await?;
	let error = made.Error()?;
	if error != BluetoothError::Success {
		return Ok(refused(error));
	}
	let provider = made.ServiceProvider()?;
	let service = provider.Service()?;
	let add = async |id: u128, properties: GattCharacteristicProperties| -> windows::core::Result<_> {
		let params = GattLocalCharacteristicParameters::new()?;
		params.SetCharacteristicProperties(properties)?;
		params.SetReadProtectionLevel(GattProtectionLevel::Plain)?;
		params.SetWriteProtectionLevel(GattProtectionLevel::Plain)?;
		let made = service.CreateCharacteristicAsync(GUID::from_u128(id), &params)?.await?;
		Ok((made.Error()?, made.Characteristic()))
	};
	let (e1, hello_chr) = add(ble::HELLO, GattCharacteristicProperties::Read).await?;
	let (e2, rx) =
		add(ble::RX, GattCharacteristicProperties::Write | GattCharacteristicProperties::WriteWithoutResponse).await?;
	let (e3, tx) = add(ble::TX, GattCharacteristicProperties::Notify).await?;
	if let Some(error) = [e1, e2, e3].into_iter().find(|e| *e != BluetoothError::Success) {
		return Ok(refused(error));
	}
	let (hello_chr, rx, tx) = (hello_chr?, rx?, tx?);
	let clients: Clients = Arc::default();
	let (reads, subscribed, seen, told) = (hellos.clone(), tx.clone(), clients.clone(), events.clone());
	let read =
		hello_chr.ReadRequested(&TypedEventHandler::<GattLocalCharacteristic, GattReadRequestedEventArgs>::new(
			move |_, args| answer_read(args.ok()?, &subscribed, &seen, &reads, &told),
		))?;
	let (writes, seen, told) = (tx.clone(), clients.clone(), events.clone());
	let written =
		rx.WriteRequested(&TypedEventHandler::<GattLocalCharacteristic, GattWriteRequestedEventArgs>::new(
			move |_, args| take_write(args.ok()?, &writes, &seen, &told),
		))?;
	let (seen, told) = (clients.clone(), events.clone());
	let changed = tx.SubscribedClientsChanged(&TypedEventHandler::<GattLocalCharacteristic, IInspectable>::new(
		move |chr, _| {
			look_at_subscribers(chr.ok()?, &seen, &told);
			Ok(())
		},
	))?;
	let status = adverts.clone();
	provider.AdvertisementStatusChanged(&TypedEventHandler::<
		GattServiceProvider,
		GattServiceProviderAdvertisementStatusChangedEventArgs,
	>::new(move |_, args| {
		let args = args.ok()?;
		let _ = status.send(Advert { provider: number, status: args.Status()?, error: args.Error()? });
		Ok(())
	}))?;
	let served = Served {
		provider,
		tx: (tx, changed),
		hello: (hello_chr, read),
		rx: (rx, written),
		clients,
		senders: HashMap::new(),
		preferred: HashMap::new(),
	};
	advertise(&served.provider)?;
	Ok(Start::Served(served))
}

/// The provider's advertising, connectable and discoverable with the service data where Windows has it: at its
/// publish, and again on the same provider at the first failure in a row of its advertising (`Publisher::due`).
fn advertise(provider: &GattServiceProvider) -> windows::core::Result<()> {
	let params = GattServiceProviderAdvertisingParameters::new()?;
	params.SetIsConnectable(true)?;
	params.SetIsDiscoverable(true)?;
	if ApiInformation::IsPropertyPresent(&HSTRING::from(SERVICE_DATA.0), &HSTRING::from(SERVICE_DATA.1))? {
		params.SetServiceData(&buffer(&ble::SERVICE_DATA)?)?;
	}
	provider.StartAdvertisingWithParameters(&params)
}

/// A creation the system refused: the helper block's word for its reason, or a failure.
fn refused(error: BluetoothError) -> Start {
	match error {
		BluetoothError::RadioNotAvailable => Start::State(Ble::Off),
		BluetoothError::NotSupported | BluetoothError::TransportNotSupported => Start::State(Ble::NoPeripheral),
		BluetoothError::DisabledByUser | BluetoothError::DisabledByPolicy | BluetoothError::ConsentRequired => {
			Start::State(Ble::NoGrant)
		}
		other => Start::Failed(format!("the service was refused (Bluetooth error {})", other.0)),
	}
}

/// A central's id: its session's device id.
fn central(session: &GattSession) -> windows::core::Result<String> {
	Ok(session.DeviceId()?.Id()?.to_string())
}

/// Bytes as a `WinRT` buffer: a value of the link's, and the band ELDEN RING's boss banners hand Windows' OCR
/// (`capture::banner`).
pub(crate) fn buffer(bytes: &[u8]) -> windows::core::Result<IBuffer> {
	let writer = DataWriter::new()?;
	writer.WriteBytes(bytes)?;
	writer.DetachBuffer()
}

fn bytes(buffer: &IBuffer) -> windows::core::Result<Vec<u8>> {
	let mut out = vec![0; usize::try_from(buffer.Length()?).unwrap_or(0)];
	DataReader::FromBuffer(buffer)?.ReadBytes(&mut out)?;
	Ok(out)
}

/// A request's deferral, closed however its handler ends, as C#'s `using` closes it (`Deferral.Close` completes it
/// unless it was, and may be called again), so no read or write waits for the ATT timeout.
struct Deferred(Deferral);

impl Drop for Deferred {
	fn drop(&mut self) {
		let _ = self.0.Close();
	}
}

/// A request something of which did not read, answered with the error Windows gives a request no handler takes.
fn refuse(respond: impl FnOnce(u8) -> windows::core::Result<()>) {
	let _ = GattProtocolError::UnlikelyError().and_then(respond);
}

/// Who is subscribed now, its failure logged: the request it was read for is answered all the same.
fn look_at_subscribers(tx: &GattLocalCharacteristic, clients: &Clients, events: &Events) {
	if let Err(e) = subscribers_changed(tx, clients, events) {
		log::warn!("ble: the subscribed clients did not read ({e})");
	}
}

/// The reading central's hello from the read's offset on (`Hellos::read`: from the start one made for it, its challenge
/// told to the bridge before this answer, after who is subscribed now).
fn answer_read(
	args: &GattReadRequestedEventArgs,
	tx: &GattLocalCharacteristic,
	clients: &Clients,
	hellos: &Hellos,
	events: &Events,
) -> windows::core::Result<()> {
	let _deferral = Deferred(args.GetDeferral()?);
	let request = args.GetRequestAsync()?.join()?;
	let hello = || -> windows::core::Result<Option<Vec<u8>>> {
		let reader = central(&args.Session()?)?;
		let offset = usize::try_from(request.Offset()?).unwrap_or(usize::MAX);
		if offset == 0 {
			// an unsubscribe Windows applied before this read (a phone that turned `tx` off and reads the hello at once)
			// is told before the read's new challenge, whatever order Windows raises their events in
			look_at_subscribers(tx, clients, events);
		}
		Ok(hellos.read(events, &reader, offset))
	};
	match hello() {
		Ok(Some(bytes)) => request.RespondWithValue(&buffer(&bytes)?),
		Ok(None) => request.RespondWithProtocolError(GattProtocolError::InvalidOffset()?),
		Err(e) => {
			refuse(|error| request.RespondWithProtocolError(error));
			Err(e)
		}
	}
}

/// An `rx` write passed on as one chunk with the writer's notification size while it stands among the subscribed
/// clients now (answered when it asks for a response); a write at an offset (a prepared long write, which no phone's
/// chunk needs) is refused.
fn take_write(
	args: &GattWriteRequestedEventArgs,
	tx: &GattLocalCharacteristic,
	clients: &Clients,
	events: &Events,
) -> windows::core::Result<()> {
	let _deferral = Deferred(args.GetDeferral()?);
	let request = args.GetRequestAsync()?.join()?;
	// answered unless it is known to want no answer, so a failure reaches a writer that waits for one
	let respond = !matches!(request.Option(), Ok(GattWriteOption::WriteWithoutResponse));
	let take = || -> windows::core::Result<bool> {
		if request.Offset()? != 0 {
			return Ok(false);
		}
		let writer = central(&args.Session()?)?;
		// who is subscribed now: a client's first write may come before its subscription's change is raised
		look_at_subscribers(tx, clients, events);
		let size = lock(clients).get(&writer).map(|&(_, size)| size);
		events.send(Radio::Written { central: writer, value: bytes(&request.Value()?)?, size });
		Ok(true)
	};
	match take() {
		Ok(_) if !respond => Ok(()),
		Ok(true) => request.Respond(),
		Ok(false) => request.RespondWithProtocolError(GattProtocolError::InvalidOffset()?),
		Err(e) => {
			if respond {
				refuse(|error| request.RespondWithProtocolError(error));
			}
			Err(e)
		}
	}
}

/// The subscribed clients now against the ones known: who came (with its notification size) and who went, told once
/// the new list is whole. The list is read under the lock, so of two handlers looking at once the later tells the later
/// list. A client whose id does not read (one going away) keeps its known entry while it is the same object and else is
/// left out; one new whose notification size does not read keeps the entry known under its id, or is left out until
/// the next look (its change, a read of the hello, a write).
fn subscribers_changed(tx: &GattLocalCharacteristic, clients: &Clients, events: &Events) -> windows::core::Result<()> {
	let mut known = lock(clients);
	let listed = tx.SubscribedClients()?;
	let mut now = HashMap::new();
	let mut came = Vec::new();
	for client in listed {
		let id = match client.Session().and_then(|session| central(&session)) {
			Ok(id) => id,
			Err(e) => {
				log::debug!("ble: a subscribed client's id did not read ({e})");
				if let Some((id, entry)) = known.iter().find(|(_, (was, _))| *was == client) {
					now.insert(id.clone(), entry.clone());
				}
				continue;
			}
		};
		let entry = match known.get(&id) {
			Some((was, size)) if *was == client => (client, *size),
			was => match client.MaxNotificationSize() {
				Ok(size) => {
					let size = usize::from(size);
					came.push((id.clone(), size));
					(client, size)
				}
				Err(e) => {
					log::debug!("ble: a subscribed client's notification size did not read ({e})");
					let Some(was) = was else { continue };
					was.clone()
				}
			},
		};
		now.insert(id, entry);
	}
	for gone in known.keys().filter(|id| !now.contains_key(*id)) {
		events.send(Radio::Unsubscribed { central: gone.clone() });
	}
	for (central, size) in came {
		events.send(Radio::Subscribed { central, size });
	}
	*known = now;
	Ok(())
}

impl Served {
	/// A message to one central, after its earlier ones: on its notification task, made anew when the central's client
	/// is another object than the one it notifies (the central subscribed again); dropped, its `Done` telling, when the
	/// central is not subscribed.
	fn send(&mut self, central: &str, message: Outgoing) {
		let client = {
			let known = lock(&self.clients);
			self.senders.retain(|id, (_, sender)| !sender.is_closed() && known.contains_key(id));
			known.get(central).map(|(client, _)| client.clone())
		};
		let Some(client) = client else { return };
		if self.senders.get(central).is_none_or(|(notified, _)| *notified != client) {
			let (sender, notes) = mpsc::unbounded_channel();
			tokio::spawn(notify(self.tx.0.clone(), client.clone(), central.to_owned(), notes));
			self.senders.insert(central.to_owned(), (client, sender));
		}
		if let Some((_, sender)) = self.senders.get(central) {
			let _ = sender.send(Note::Message(message));
		}
	}

	/// A central's waiting messages under an id dropped (its call was replaced).
	fn cancel(&self, central: &str, id: u8) {
		if let Some((_, sender)) = self.senders.get(central) {
			let _ = sender.send(Note::Cancel(id));
		}
	}

	/// The bridge let a central go: its waiting messages are dropped, its notification task ends with its sender, and
	/// its preferred parameters go.
	fn forget(&mut self, central: &str) {
		if let Some((_, sender)) = self.senders.remove(central) {
			let _ = sender.send(Note::Clear);
		}
		self.preferred.remove(central);
	}

	/// The connection parameters asked for one central where Windows 11 has the method: throughput while a pump runs,
	/// balanced while idle.
	async fn latency(&mut self, central: String, latency: Latency) {
		self.preferred.retain(|id, _| lock(&self.clients).contains_key(id));
		let present = ApiInformation::IsMethodPresent(&HSTRING::from(PREFERRED.0), &HSTRING::from(PREFERRED.1));
		if !present.unwrap_or(false) || !lock(&self.clients).contains_key(&central) {
			return;
		}
		let asked = async {
			let device = BluetoothLEDevice::FromIdAsync(&HSTRING::from(central.as_str()))?.await?;
			let wanted = match latency {
				Latency::Low => BluetoothLEPreferredConnectionParameters::ThroughputOptimized()?,
				Latency::Medium => BluetoothLEPreferredConnectionParameters::Balanced()?,
			};
			let request = device.RequestPreferredConnectionParameters(&wanted)?;
			windows::core::Result::Ok((device, request))
		};
		match asked.await {
			Ok(kept) => {
				self.preferred.insert(central, kept);
			}
			Err(e) => log::debug!("ble: no preferred connection parameters ({e})"),
		}
	}
}

/// One central's messages, a chunk at a time from its `Queue`, each notification awaited before the next; what came
/// meanwhile joins the queue first, so an urgent answer goes ahead of a page's next chunk. It ends when the bridge lets
/// the central go, or at a notification that fails (its client went): the messages that wait then tell they went
/// nowhere.
async fn notify(
	tx: GattLocalCharacteristic,
	client: GattSubscribedClient,
	central: String,
	mut notes: UnboundedReceiver<Note>,
) {
	let mut queue = Queue::default();
	let take = |queue: &mut Queue, note: Note| match note {
		Note::Message(message) => queue.push(message),
		Note::Cancel(id) => queue.cancel(id),
		Note::Clear => *queue = Queue::default(),
	};
	loop {
		while let Ok(note) = notes.try_recv() {
			take(&mut queue, note);
		}
		let Some(chunk) = queue.next().map(<[u8]>::to_vec) else {
			match notes.recv().await {
				Some(note) => {
					take(&mut queue, note);
					continue;
				}
				None => return,
			}
		};
		let sent = async {
			// the buffer (an interface, not `Send`) goes before the wait
			let notified = tx.NotifyValueForSubscribedClientAsync(&buffer(&chunk)?, &client)?;
			notified.await?.Status()
		};
		match sent.await {
			Ok(GattCommunicationStatus::Success) => queue.sent(),
			Ok(status) => {
				log::debug!("ble: a notification to {} not sent ({status:?})", ble::central_tag(&central));
				return;
			}
			Err(e) => {
				log::debug!("ble: a notification to {} failed ({e})", ble::central_tag(&central));
				return;
			}
		}
	}
}
