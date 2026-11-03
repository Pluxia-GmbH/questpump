//! The Bluetooth link's peripheral on macOS (slice 43, with its fixes after the review of 2026-10-17; rust-helper
//! `bluetooth.md`, "The Bluetooth link's peripheral"): a `CBPeripheralManager` whose delegate runs on a serial dispatch
//! queue of its own, the only place the manager, the service, the centrals and their queues are touched (each order
//! from the bridge is a closure run there). At `poweredOn` it publishes the service (`hello` dynamic, answered for each
//! central from `Hellos`; `rx` write and write without response; `tx` notify; nothing asks for encryption: the envelope
//! does that work) and, once added, advertises the service's UUID and the name `QuestPump`; a state below it has
//! dropped every central. A publish that fails is tried again after `super::backoff` (each check of the bridge's tick
//! looks), and an advertising that fails advertises the same service again then, which keeps its subscribed centrals;
//! five failures in a row give up. The check also advertises again when the manager is powered on, the service added
//! and nothing advertises (as after the Mac's sleep). A new pairing code publishes the service again
//! (`removeAllServices`, then `addService`), so a connected phone hears its services changed; every publish first
//! tells the bridge each central subscribed to the service it takes away went, as a state below powered on does. The
//! centrals subscribed to the published `tx` itself (a late word of one a publish took away is ignored) are kept with
//! their notification size, which each write carries to the bridge; their messages go out a chunk at a time, the
//! centrals in turn, and a chunk the transmit queue refuses waits for `peripheralManagerIsReadyToUpdateSubscribers:`.
//! Making the manager raises the Bluetooth prompt while the grant is not answered, so the runtime starts it unasked
//! only once it is (`may_ask`), else where the pairing code shows.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use dispatch2::{DispatchQueue, DispatchRetained};
use objc2::rc::Retained;
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2::{AnyThread as _, DefinedClass as _, Message as _, define_class, msg_send};
use objc2_core_bluetooth::{
	CBATTError, CBATTRequest, CBAdvertisementDataLocalNameKey, CBAdvertisementDataServiceUUIDsKey,
	CBAttributePermissions, CBCentral, CBCharacteristic, CBCharacteristicProperties, CBManager, CBManagerAuthorization,
	CBManagerState, CBMutableCharacteristic, CBMutableService, CBPeripheralManager,
	CBPeripheralManagerConnectionLatency, CBPeripheralManagerDelegate, CBService, CBUUID,
};
use objc2_foundation::{NSArray, NSData, NSDictionary, NSError, NSObject, NSObjectProtocol, NSString};
use questpump_core::ble;
use questpump_core::link::Ble;

use super::{CENTRALS_MAX, Events, Hellos, Latency, Outgoing, Queue, Radio, backoff};

/// The dispatch queue's label.
const QUEUE: &str = "ai.pluxia.questpump.ble";

/// Whether the Bluetooth grant is answered, so making the manager asks nothing (`CBManager.authorization`, macOS
/// 10.15 on).
pub(super) fn may_ask() -> bool {
	// SAFETY: a class property read that takes no arguments and may be called from any thread (Apple's
	// `CBManager.authorization`; the helper needs macOS 14).
	unsafe { CBManager::authorization_class() != CBManagerAuthorization::NotDetermined }
}

/// What only the dispatch queue touches.
#[derive(Default)]
struct OnQueue {
	manager: Option<Retained<CBPeripheralManager>>,
	service: Option<Retained<CBMutableService>>,
	tx: Option<Retained<CBMutableCharacteristic>>,
	/// The subscribed centrals by identifier.
	centrals: HashMap<String, Subscribed>,
	/// Subscriptions numbered as they came, so the oldest goes first past `CENTRALS_MAX`.
	subscriptions: u64,
	/// The last state told the bridge, so a repeat tells nothing.
	told: Option<Ble>,
	/// The service is in the manager's database (`didAddService` without an error, since the last publish).
	added: bool,
	/// `startAdvertising` was called and its `didStartAdvertising` has not come yet.
	starting: bool,
	/// The publishes and advertisings that failed in a row, and when the next try is due: a publish again while the
	/// service is not added, else an advertising.
	failures: u32,
	retry_at: Option<Instant>,
	/// Let go by the bridge: nothing advertises again.
	quiet: bool,
	/// How a subscription's characteristic is matched to the published `tx` (`is_tx`).
	tx_match: TxMatch,
}

/// How a subscription's characteristic is matched to the published `tx`.
#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum TxMatch {
	/// By its UUID, until `CoreBluetooth` is seen handing the published object itself: Apple does not say the
	/// callback's characteristic is that object.
	#[default]
	Uuid,
	/// By the object itself, once seen.
	Object,
}

/// A subscribed central: the most a notification carries to it, and its messages waiting for the radio.
struct Subscribed {
	central: Retained<CBCentral>,
	size: usize,
	number: u64,
	queue: Queue,
}

struct Ivars {
	events: Events,
	hellos: Arc<Hellos>,
	queue: RefCell<OnQueue>,
}

define_class!(
	// SAFETY: NSObject has no subclassing requirements, and `Delegate` implements no `Drop`.
	#[unsafe(super(NSObject))]
	#[ivars = Ivars]
	struct Delegate;

	unsafe impl NSObjectProtocol for Delegate {}

	unsafe impl CBPeripheralManagerDelegate for Delegate {
		#[unsafe(method(peripheralManagerDidUpdateState:))]
		fn did_update_state(&self, peripheral: &CBPeripheralManager) {
			self.state_changed(peripheral);
		}

		#[unsafe(method(peripheralManager:didAddService:error:))]
		fn did_add_service(&self, peripheral: &CBPeripheralManager, _service: &CBService, error: Option<&NSError>) {
			if let Some(e) = error {
				// copy-lint: not shown (a log line)
				self.trouble(&format!("the service was not added: {}", e.localizedDescription()));
			} else {
				self.ivars().queue.borrow_mut().added = true;
				self.advertise(peripheral);
			}
		}

		#[unsafe(method(peripheralManagerDidStartAdvertising:error:))]
		fn did_start_advertising(&self, _peripheral: &CBPeripheralManager, error: Option<&NSError>) {
			self.ivars().queue.borrow_mut().starting = false;
			if let Some(e) = error {
				// copy-lint: not shown (a log line)
				self.trouble(&format!("advertising did not start: {}", e.localizedDescription()));
			} else {
				self.ivars().queue.borrow_mut().failures = 0;
				self.tell(Ble::On);
			}
		}

		#[unsafe(method(peripheralManager:central:didSubscribeToCharacteristic:))]
		fn did_subscribe(
			&self,
			_peripheral: &CBPeripheralManager,
			central: &CBCentral,
			characteristic: &CBCharacteristic,
		) {
			if !self.is_tx(characteristic) {
				return;
			}
			let id = identifier(central);
			// SAFETY: a property read on the central `CoreBluetooth` hands this delegate, on its queue.
			let size = unsafe { central.maximumUpdateValueLength() };
			let oldest = {
				let mut q = self.ivars().queue.borrow_mut();
				q.subscriptions += 1;
				let number = q.subscriptions;
				// past the most, the oldest goes: one whose going `CoreBluetooth` never told
				let full = q.centrals.len() >= CENTRALS_MAX && !q.centrals.contains_key(&id);
				let oldest =
					if full { q.centrals.iter().min_by_key(|(_, s)| s.number).map(|(id, _)| id.clone()) } else { None };
				if let Some(oldest) = &oldest {
					q.centrals.remove(oldest);
				}
				q.centrals.insert(
					id.clone(),
					Subscribed { central: central.retain(), size, number, queue: Queue::default() },
				);
				oldest
			};
			if let Some(central) = oldest {
				self.ivars().events.send(Radio::Unsubscribed { central });
			}
			self.ivars().events.send(Radio::Subscribed { central: id, size });
		}

		#[unsafe(method(peripheralManager:central:didUnsubscribeFromCharacteristic:))]
		fn did_unsubscribe(
			&self,
			_peripheral: &CBPeripheralManager,
			central: &CBCentral,
			characteristic: &CBCharacteristic,
		) {
			if !self.is_tx(characteristic) {
				return;
			}
			let id = identifier(central);
			let gone = self.ivars().queue.borrow_mut().centrals.remove(&id);
			drop(gone); // its waiting messages tell the bridge they went nowhere
			self.ivars().events.send(Radio::Unsubscribed { central: id });
		}

		#[unsafe(method(peripheralManager:didReceiveReadRequest:))]
		fn did_receive_read(&self, peripheral: &CBPeripheralManager, request: &CBATTRequest) {
			// SAFETY: reads and a write of the request `CoreBluetooth` hands this delegate, answered once, on its queue.
			unsafe {
				if !is(&request.characteristic(), ble::HELLO) {
					peripheral.respondToRequest_withResult(request, CBATTError::ReadNotPermitted);
					return;
				}
				let central = identifier(&request.central());
				let ivars = self.ivars();
				let Some(bytes) = ivars.hellos.read(&ivars.events, &central, request.offset()) else {
					peripheral.respondToRequest_withResult(request, CBATTError::InvalidOffset);
					return;
				};
				request.setValue(Some(&NSData::with_bytes(&bytes)));
				peripheral.respondToRequest_withResult(request, CBATTError::Success);
			}
		}

		#[unsafe(method(peripheralManager:didReceiveWriteRequests:))]
		fn did_receive_writes(&self, peripheral: &CBPeripheralManager, requests: &NSArray<CBATTRequest>) {
			self.writes(peripheral, &requests.to_vec());
		}

		#[unsafe(method(peripheralManagerIsReadyToUpdateSubscribers:))]
		fn ready(&self, peripheral: &CBPeripheralManager) {
			self.drain(peripheral);
		}
	}
);

impl Delegate {
	fn new(events: Events, hellos: Arc<Hellos>) -> Retained<Self> {
		let this = Self::alloc().set_ivars(Ivars { events, hellos, queue: RefCell::new(OnQueue::default()) });
		// SAFETY: NSObject's designated initialiser, on a fresh allocation whose ivars are set.
		unsafe { msg_send![super(this), init] }
	}

	/// Tells the bridge a state once.
	fn tell(&self, state: Ble) {
		let mut q = self.ivars().queue.borrow_mut();
		if q.told != Some(state) {
			q.told = Some(state);
			self.ivars().events.send(Radio::State(state));
		}
	}

	/// A publish or an advertising failed: tried again after the backoff (the next check that finds it due: a service
	/// not added is published again, an added one advertised again), or given up after `RETRIES` in a row.
	fn trouble(&self, why: &str) {
		let mut q = self.ivars().queue.borrow_mut();
		q.failures += 1;
		if let Some(wait) = backoff(q.failures) {
			log::warn!("ble: {why}; tried again in {} s", wait.as_secs());
			q.retry_at = Some(Instant::now() + wait);
		} else {
			// given up: the check neither publishes nor advertises again (the bridge lets every central go, and the
			// runtime stops the link)
			q.retry_at = None;
			q.added = false;
			q.told = None;
			drop(q);
			self.ivars().events.send(Radio::Failed(why.to_owned()));
		}
	}

	/// The manager's state: at `poweredOn` the service is published anew (a state below it cleared it), and a state
	/// below has dropped every central.
	fn state_changed(&self, peripheral: &CBPeripheralManager) {
		// SAFETY: the manager's state, read on its queue.
		let state = unsafe { peripheral.state() };
		if state != CBManagerState::PoweredOn {
			// resetting too: the bridge hears each central go, whatever the state says next
			{
				let mut q = self.ivars().queue.borrow_mut();
				q.added = false;
				q.starting = false;
			}
			self.let_centrals_go();
		}
		match state {
			CBManagerState::PoweredOn => {
				{
					let mut q = self.ivars().queue.borrow_mut();
					q.failures = 0;
					q.retry_at = None;
				}
				self.publish(peripheral);
			}
			CBManagerState::PoweredOff => self.tell(Ble::Off),
			CBManagerState::Unauthorized => self.tell(Ble::NoGrant),
			CBManagerState::Unsupported => self.tell(Ble::Missing),
			// resetting or not known yet (the grant's question on the screen): the next update says
			_ => {}
		}
	}

	/// Every subscribed central goes, each told to the bridge (its waiting messages telling they went nowhere): their
	/// subscriptions went with the state, or go with the service taken away.
	fn let_centrals_go(&self) {
		let gone: Vec<(String, Subscribed)> = self.ivars().queue.borrow_mut().centrals.drain().collect();
		for (central, _) in gone {
			self.ivars().events.send(Radio::Unsubscribed { central });
		}
	}

	/// The service with its three characteristics, added to the manager's database in place of the one there, whose
	/// subscribed centrals the bridge first hears go (its advertising starts once
	/// `peripheralManager:didAddService:error:` says it was added).
	fn publish(&self, peripheral: &CBPeripheralManager) {
		if self.ivars().queue.borrow().quiet {
			return;
		}
		self.let_centrals_go();
		let props = CBCharacteristicProperties::Read;
		let write = CBCharacteristicProperties::Write | CBCharacteristicProperties::WriteWithoutResponse;
		let notify = CBCharacteristicProperties::Notify;
		let hello = characteristic(ble::HELLO, props, CBAttributePermissions::Readable);
		let rx = characteristic(ble::RX, write, CBAttributePermissions::Writeable);
		let tx = characteristic(ble::TX, notify, CBAttributePermissions::empty());
		// SAFETY: a new service made and filled on the manager's queue, then handed to the manager there, after its
		// advertising and the service before it are taken away.
		let service = unsafe {
			let service = CBMutableService::initWithType_primary(CBMutableService::alloc(), &uuid(ble::SERVICE), true);
			let all: [&CBCharacteristic; 3] = [&hello, &rx, &tx];
			service.setCharacteristics(Some(&NSArray::from_slice(&all)));
			peripheral.stopAdvertising();
			peripheral.removeAllServices();
			peripheral.addService(&service);
			service
		};
		let mut q = self.ivars().queue.borrow_mut();
		q.service = Some(service);
		q.tx = Some(tx);
		q.added = false;
		q.starting = false;
	}

	/// Advertises the service's UUID and the name (`startAdvertising` takes only these two).
	fn advertise(&self, peripheral: &CBPeripheralManager) {
		{
			let mut q = self.ivars().queue.borrow_mut();
			if q.quiet {
				return;
			}
			q.starting = true;
		}
		let name = NSString::from_str(ble::LOCAL_NAME);
		let services = NSArray::from_retained_slice(&[uuid(ble::SERVICE)]);
		// SAFETY: the two keys `CoreBluetooth`'s advertising takes, with an NSString and an NSArray of CBUUID, on the
		// manager's queue.
		unsafe {
			let keys = [CBAdvertisementDataLocalNameKey, CBAdvertisementDataServiceUUIDsKey];
			let values: [&AnyObject; 2] = [&name, &services];
			peripheral.startAdvertising(Some(&NSDictionary::from_slices(&keys, &values)));
		}
	}

	/// The bridge's look, each second: a retry that is due publishes again while the service is not added and else
	/// advertises it again, its subscribed centrals kept; and a manager powered on whose service is added but which does
	/// not advertise (the Mac slept, or macOS stopped it without a word) advertises again.
	fn check(&self, peripheral: &CBPeripheralManager) {
		// SAFETY: the manager's state and advertising, read on its queue.
		let (state, advertising) = unsafe { (peripheral.state(), peripheral.isAdvertising()) };
		let (due, added, again) = {
			let mut q = self.ivars().queue.borrow_mut();
			if q.quiet || state != CBManagerState::PoweredOn {
				return;
			}
			let due = q.retry_at.is_some_and(|at| Instant::now() >= at);
			if due {
				q.retry_at = None;
			}
			(due, q.added, q.retry_at.is_none() && q.added && !q.starting && !advertising)
		};
		if due && !added {
			self.publish(peripheral);
		} else if again {
			if !due {
				log::info!("ble: not advertising while powered on: advertising again");
			}
			self.advertise(peripheral);
		}
	}

	/// The writes of one ATT request or prepared batch: each `rx` value at its offset a chunk (one write's value, or a
	/// long write's values joined), answered once with the first request, each with the writer's notification size
	/// while its subscription stands; a batch with another characteristic or a value out of place is refused whole.
	fn writes(&self, peripheral: &CBPeripheralManager, requests: &[Retained<CBATTRequest>]) {
		let Some(first) = requests.first() else { return };
		let mut chunks: Vec<(String, Vec<u8>)> = Vec::new();
		let mut result = CBATTError::Success;
		for request in requests {
			// SAFETY: reads of the requests `CoreBluetooth` hands this delegate, on its queue.
			let (central, chr, offset, value) = unsafe {
				(identifier(&request.central()), request.characteristic(), request.offset(), request.value())
			};
			if !is(&chr, ble::RX) {
				result = CBATTError::WriteNotPermitted;
				break;
			}
			let value = value.map(|v| v.to_vec()).unwrap_or_default();
			let joins = offset > 0 && chunks.last().is_some_and(|(c, chunk)| *c == central && chunk.len() == offset);
			if joins && let Some((_, chunk)) = chunks.last_mut() {
				chunk.extend(value);
			} else if offset == 0 {
				chunks.push((central, value));
			} else {
				result = CBATTError::InvalidOffset;
				break;
			}
		}
		// SAFETY: the one answer to this callback, with its first request (Apple's
		// `peripheralManager(_:didReceiveWrite:)`), on the manager's queue.
		unsafe { peripheral.respondToRequest_withResult(first, result) };
		if result == CBATTError::Success {
			for (central, value) in chunks {
				let size = self.ivars().queue.borrow().centrals.get(&central).map(|s| s.size);
				self.ivars().events.send(Radio::Written { central, value, size });
			}
		}
	}

	/// The waiting chunks, one from each central in turn, until none waits or the transmit queue is full. No borrow of
	/// the queue state is held across a call into `CoreBluetooth`.
	fn drain(&self, peripheral: &CBPeripheralManager) {
		let (tx, ids) = {
			let q = self.ivars().queue.borrow();
			(q.tx.clone(), q.centrals.keys().cloned().collect::<Vec<_>>())
		};
		let Some(tx) = tx else { return };
		loop {
			let mut any = false;
			for id in &ids {
				let next = self.ivars().queue.borrow().centrals.get(id).and_then(|sub| {
					let chunk = sub.queue.next()?;
					Some((NSArray::from_retained_slice(std::slice::from_ref(&sub.central)), NSData::with_bytes(chunk)))
				});
				let Some((to, chunk)) = next else { continue };
				// SAFETY: a notification to one subscribed central, on the manager's queue.
				let sent =
					unsafe { peripheral.updateValue_forCharacteristic_onSubscribedCentrals(&chunk, &tx, Some(&to)) };
				if !sent {
					return; // the transmit queue is full: `peripheralManagerIsReadyToUpdateSubscribers:` comes
				}
				if let Some(sub) = self.ivars().queue.borrow_mut().centrals.get_mut(id) {
					sub.queue.sent();
				}
				any = true;
			}
			if !any {
				return;
			}
		}
	}

	/// Whether a subscription's characteristic is the published service's `tx`: the object itself once `CoreBluetooth`
	/// has been seen handing it over, since a word it sends late of the `tx` a publish took away (its centrals told gone
	/// then) would else end, or stand for, a subscription to the new one; until then its UUID, so a `CoreBluetooth`
	/// that hands a copy still links (Apple's page names no identity).
	fn is_tx(&self, characteristic: &CBCharacteristic) -> bool {
		let mut q = self.ivars().queue.borrow_mut();
		let same = q.tx.as_deref().is_some_and(|tx| {
			let tx: &CBCharacteristic = tx;
			std::ptr::eq(tx, characteristic)
		});
		if same {
			q.tx_match = TxMatch::Object;
			return true;
		}
		q.tx.is_some() && q.tx_match == TxMatch::Uuid && is(characteristic, ble::TX)
	}

	fn manager(&self) -> Option<Retained<CBPeripheralManager>> {
		self.ivars().queue.borrow().manager.clone()
	}

	/// A subscribed central's object.
	fn central(&self, id: &str) -> Option<Retained<CBCentral>> {
		self.ivars().queue.borrow().centrals.get(id).map(|sub| sub.central.clone())
	}
}

/// Whether an attribute is the one of this UUID.
fn is(characteristic: &CBCharacteristic, id: u128) -> bool {
	// SAFETY: the attribute's UUID, read on the manager's queue.
	let given = unsafe { characteristic.UUID().UUIDString() };
	given.to_string().eq_ignore_ascii_case(&ble::uuid_text(id))
}

fn uuid(id: u128) -> Retained<CBUUID> {
	// SAFETY: a CBUUID from a valid UUID's text.
	unsafe { CBUUID::UUIDWithString(&NSString::from_str(&ble::uuid_text(id))) }
}

/// A characteristic whose value is dynamic (`value: nil`): a `hello` given a value at its creation would be cached
/// for every central, and the hello is made for each.
fn characteristic(
	id: u128,
	properties: CBCharacteristicProperties,
	permissions: CBAttributePermissions,
) -> Retained<CBMutableCharacteristic> {
	// SAFETY: a new characteristic of a valid UUID, with no cached value.
	unsafe {
		CBMutableCharacteristic::initWithType_properties_value_permissions(
			CBMutableCharacteristic::alloc(),
			&uuid(id),
			properties,
			None,
			permissions,
		)
	}
}

/// A central's identifier, which names it to the bridge (`Peer::Ble`).
fn identifier(central: &CBCentral) -> String {
	// SAFETY: the peer's identifier, read on the manager's queue.
	unsafe { central.identifier() }.UUIDString().to_string()
}

/// The delegate, to be touched only on the dispatch queue.
struct Queued(Retained<Delegate>);

// SAFETY: the delegate's queue state (`RefCell`) and the `CoreBluetooth` objects it holds are touched only inside
// closures run on the one serial dispatch queue the manager delivers its callbacks on, never two at once.
unsafe impl Send for Queued {}

impl Queued {
	/// The delegate, inside a closure on the queue: a call on the whole wrapper, so the closure captures the wrapper
	/// (which is `Send`) and not its field (which is not).
	fn into_inner(self) -> Retained<Delegate> {
		self.0
	}
}

/// The peripheral: the delegate on its queue.
pub(super) struct Peripheral {
	queue: DispatchRetained<DispatchQueue>,
	delegate: Option<Queued>,
}

impl Peripheral {
	/// Makes the manager on its own queue, with the delegate that tells `events` and answers the hello from `hellos`
	/// (macOS asks for Bluetooth here while the grant is not answered); its state comes in the first callback.
	pub(super) fn start(events: Events, hellos: Arc<Hellos>) -> Self {
		let queue = DispatchQueue::new(QUEUE, None);
		let delegate = Delegate::new(events, hellos);
		let on = Queued(delegate.clone());
		let target = queue.clone();
		queue.exec_async(move || {
			let delegate = on.into_inner();
			// SAFETY: a new manager whose delegate (held by this peripheral, the manager keeps it weakly) and events
			// come on `target`, the queue this closure runs on.
			let manager = unsafe {
				CBPeripheralManager::initWithDelegate_queue(
					CBPeripheralManager::alloc(),
					Some(ProtocolObject::from_ref(&*delegate)),
					Some(&target),
				)
			};
			delegate.ivars().queue.borrow_mut().manager = Some(manager);
		});
		Self { queue, delegate: Some(Queued(delegate)) }
	}

	/// Runs `work` with the delegate and its manager on the queue.
	fn on_queue(&self, work: impl FnOnce(&Delegate, &CBPeripheralManager) + Send + 'static) {
		let Some(Queued(delegate)) = &self.delegate else { return };
		let on = Queued(delegate.clone());
		self.queue.exec_async(move || {
			let delegate = on.into_inner();
			if let Some(manager) = delegate.manager() {
				work(&delegate, &manager);
			}
		});
	}

	/// A message to one central, into its queue (dropped, its `Done` telling, when the central is not subscribed).
	pub(super) fn send(&self, central: &str, message: Outgoing) {
		let central = central.to_owned();
		self.on_queue(move |delegate, manager| {
			if let Some(sub) = delegate.ivars().queue.borrow_mut().centrals.get_mut(&central) {
				sub.queue.push(message);
			}
			delegate.drain(manager);
		});
	}

	/// A central's waiting messages under an id dropped (its call was replaced).
	pub(super) fn cancel(&self, central: &str, id: u8) {
		let central = central.to_owned();
		self.on_queue(move |delegate, _| {
			if let Some(sub) = delegate.ivars().queue.borrow_mut().centrals.get_mut(&central) {
				sub.queue.cancel(id);
			}
		});
	}

	/// The bridge let a central go: its waiting messages are dropped; its subscription stands as `CoreBluetooth` says.
	pub(super) fn forget(&self, central: &str) {
		let central = central.to_owned();
		self.on_queue(move |delegate, _| {
			if let Some(sub) = delegate.ivars().queue.borrow_mut().centrals.get_mut(&central) {
				sub.queue = Queue::default();
			}
		});
	}

	/// The connection latency asked of one central.
	pub(super) fn latency(&self, central: &str, latency: Latency) {
		let central = central.to_owned();
		self.on_queue(move |delegate, manager| {
			let latency = match latency {
				Latency::Medium => CBPeripheralManagerConnectionLatency::Medium,
				Latency::Low => CBPeripheralManagerConnectionLatency::Low,
			};
			if let Some(central) = delegate.central(&central) {
				// SAFETY: a wish for a connected central, on the manager's queue.
				unsafe { manager.setDesiredConnectionLatency_forCentral(latency, &central) };
			}
		});
	}

	/// A new pairing code: the service is published again while powered on, its subscriptions going with the old one
	/// (`Delegate::publish` tells each), so a connected phone hears its services changed (`didModifyServices`,
	/// `onServiceChanged`); below powered on none stands, and the next `poweredOn` publishes.
	pub(super) fn republish(&self) {
		self.on_queue(|delegate, manager| {
			{
				let mut q = delegate.ivars().queue.borrow_mut();
				q.failures = 0;
				q.retry_at = None;
			}
			// SAFETY: the manager's state, read on its queue.
			if unsafe { manager.state() } == CBManagerState::PoweredOn {
				delegate.publish(manager);
			}
		});
	}

	/// The bridge's tick: a retry due, or advertising that stopped by itself (`Delegate::check`).
	pub(super) fn check(&self) {
		self.on_queue(Delegate::check);
	}

	/// Let go: advertising stops at once, while the answers in flight still go to their centrals.
	pub(super) fn quiet(&self) {
		self.on_queue(|delegate, manager| {
			delegate.ivars().queue.borrow_mut().quiet = true;
			// SAFETY: the manager's call, on its queue.
			unsafe { manager.stopAdvertising() };
		});
	}

	/// Stops advertising and takes the service away on the queue, where the manager and the delegate are let go, and
	/// waits for it, so another manager made next never meets this one's service.
	pub(super) fn close(&mut self) -> std::future::Ready<()> {
		if let Some(delegate) = self.delegate.take() {
			self.queue.exec_sync(move || teardown(delegate));
		}
		std::future::ready(())
	}
}

impl Drop for Peripheral {
	/// A peripheral dropped without `close` (the bridge's task unwinding) is taken down the same way, without a wait.
	fn drop(&mut self) {
		if let Some(delegate) = self.delegate.take() {
			self.queue.exec_async(move || teardown(delegate));
		}
	}
}

/// The manager's last calls, on its queue: advertising stops, the service goes, and the delegate is let go after it
/// (its waiting messages telling the bridge they went nowhere).
fn teardown(on: Queued) {
	let delegate = on.into_inner();
	let taken = std::mem::take(&mut *delegate.ivars().queue.borrow_mut());
	if let Some(manager) = &taken.manager {
		// SAFETY: the manager's last calls, on its queue; its delegate is let go after it.
		unsafe {
			manager.stopAdvertising();
			manager.removeAllServices();
			manager.setDelegate(None);
		}
	}
	drop(taken);
	drop(delegate);
}
