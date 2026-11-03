//! The racing games' UDP telemetry (docs/plan.md slice 48; docs/computer-games.md §6.2): F1 25's and Forza
//! Motorsport's, which the player sends to 127.0.0.1 at the port the game's window shows. While the game runs and is
//! watched (slice 56), one UDP socket on 127.0.0.1 alone (never another address, so nothing from the network reaches
//! it) at that port, which stays free for the player's other telemetry programs otherwise; a port another program
//! holds (a telemetry tool, SimHub) is said once and tried again every `RETRY`. On Windows the table of UDP listeners
//! says whether another program holds it, read before the bind and every `RETRY` while bound (a bind beside another
//! program's socket would succeed there and take its datagrams), and ours is let go while one does; macOS keeps the
//! plain bind, which fails there. Nothing is ever sent.
//! Each datagram goes to questpump-core's `feeds::formula` or `feeds::forza` and is dropped: names, positions and car
//! data are never read. F1's telemetry in a UDP format not read here (F1 23's 2023, an older one) is said once, as the
//! step to change it, until a datagram in a format read here comes.

use std::time::{Duration, Instant};

use questpump_core::feeds::{formula, forza};
use questpump_core::games::Fed;
use tokio::net::UdpSocket;

use super::task::{self, Task, Teller};
use super::{Status, Word};
use crate::lan;

/// A port another program holds is tried again this much later, and a port listened on is looked at this often for
/// another program that holds it too (Windows).
const RETRY: Duration = Duration::from_secs(5);
/// The clock's look at a race gone quiet (Forza's end is a silence).
const IDLE: Duration = Duration::from_secs(1);
/// Room for the largest datagram either game sends (F1 25's car telemetry, 1352 bytes).
const DATAGRAM: usize = 4096;

/// Which game's telemetry a socket reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Racing {
	Formula,
	Forza,
}

/// One game's reader: whether a datagram is its own, and its word.
enum Reader {
	Formula(formula::Race),
	Forza(forza::Race),
}

impl Reader {
	fn ours(&self, packet: &[u8]) -> bool {
		match self {
			Self::Formula(_) => formula::ours(packet),
			Self::Forza(_) => forza::ours(packet),
		}
	}

	/// Whether a datagram is the game's telemetry in a format not read here, which the player changes in the game.
	fn other(&self, packet: &[u8]) -> bool {
		match self {
			Self::Formula(_) => formula::other_format(packet),
			Self::Forza(_) => false,
		}
	}

	fn push(&mut self, packet: &[u8]) -> Option<Fed> {
		match self {
			Self::Formula(race) => race.push(packet),
			Self::Forza(race) => race.push(packet, lan::now_ms()),
		}
	}

	fn idle(&mut self) -> Option<Fed> {
		match self {
			Self::Formula(_) => None,
			Self::Forza(race) => race.idle(lan::now_ms()),
		}
	}
}

/// Whether another program holds `port`: on Windows as its table of UDP listeners says (a bind on 127.0.0.1 succeeds
/// beside another program's socket on every address when both run as the same user, and takes its datagrams:
/// Microsoft's "Using `SO_REUSEADDR` and `SO_EXCLUSIVEADDRUSE`"); never elsewhere, where such a bind fails.
fn held_elsewhere(port: u16) -> bool {
	#[cfg(windows)]
	{
		super::win::udp_port_held_elsewhere(port).unwrap_or(false)
	}
	#[cfg(not(windows))]
	{
		let _ = port;
		false
	}
}

/// Listens on 127.0.0.1 at `port` until the handle is dropped; while another program holds the port, ours is let go
/// and taken again once it is free.
pub(super) fn start(teller: Teller, racing: Racing, port: u16) -> Task {
	Task::spawn(move |mut stop| async move {
		let mut reader = match racing {
			Racing::Formula => Reader::Formula(formula::Race::default()),
			Racing::Forza => Reader::Forza(forza::Race::default()),
		};
		let (mut up, mut other, mut taken, mut buf) = (false, false, false, vec![0u8; DATAGRAM]);
		loop {
			let socket = loop {
				let refused = if held_elsewhere(port) {
					None
				} else {
					match UdpSocket::bind(("127.0.0.1", port)).await {
						Ok(socket) => break socket,
						Err(e) => Some(e),
					}
				};
				if !std::mem::replace(&mut taken, true) {
					match refused {
						Some(e) => log::info!("games: port {port} on 127.0.0.1 could not be listened on: {e}"),
						None => log::info!("games: port {port} is held by another program"),
					}
					teller.tell(Word::Step(Status::PortTaken));
				}
				if !task::wait(RETRY, &mut stop).await {
					return;
				}
			};
			teller.tell(Word::Cleared);
			let mut checked = Instant::now();
			loop {
				// the port another program took meanwhile is its own: ours goes, and is taken again once it is free
				if checked.elapsed() >= RETRY {
					checked = Instant::now();
					if held_elsewhere(port) {
						log::info!("games: port {port} is held by another program: let go");
						taken = true;
						teller.tell(Word::Step(Status::PortTaken));
						break;
					}
				}
				let word = tokio::select! {
					got = socket.recv(&mut buf) => match got.ok().and_then(|n| buf.get(..n)) {
						Some(packet) if reader.ours(packet) => {
							if !std::mem::replace(&mut up, true) {
								teller.tell(Word::Up);
							}
							reader.push(packet)
						}
						// the format to change, once until the game sends one read here (`up` again, which tells it)
						Some(packet) if reader.other(packet) => {
							if std::mem::replace(&mut up, false) || !std::mem::replace(&mut other, true) {
								log::info!("games: telemetry on port {port} in another format");
								teller.tell(Word::Step(Status::UdpFormat));
							}
							None
						}
						Some(_) => None,
						// an error (a datagram larger than the room, Windows' reset): the next one is waited for
						None => {
							if !task::wait(IDLE, &mut stop).await {
								return;
							}
							None
						}
					},
					() = tokio::time::sleep(IDLE) => reader.idle(),
					_ = stop.changed() => return,
				};
				if let Some(word) = word {
					teller.tell(Word::Match(word));
				}
			}
			drop(socket);
			if !task::wait(RETRY, &mut stop).await {
				return;
			}
		}
	})
}
