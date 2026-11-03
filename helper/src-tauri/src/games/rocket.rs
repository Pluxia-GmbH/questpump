//! Rocket League's Stats API on the computer (docs/plan.md slice 35; rust-helper rules 8 and 10): where the game is
//! installed (Steam's `appmanifest_252950`, then the Epic Games Launcher's manifests, then Epic's default folder), and
//! the helper's one outgoing connection, to 127.0.0.1 at the port its `TAStatsAPI.ini` names, while `RocketLeague.exe`
//! runs and the game is set up ("Ask after each match", slice 56): tried every 5 s, read only (nothing is ever written
//! to the socket, whose commands would steer the game), each envelope's `Event` passed on
//! (`questpump_core::rocket::Frames`) and the rest dropped.

use std::path::PathBuf;
use std::time::Duration;

use questpump_core::games::Watched;
use questpump_core::rocket::Frames;
use tokio::io::AsyncReadExt as _;
use tokio::net::TcpStream;
use tokio::sync::{mpsc, watch};

use super::{epic, steam};

/// A connection that failed or ended is tried again this much later.
const RETRY: Duration = Duration::from_secs(5);

/// What the connection says.
#[derive(Debug)]
pub enum News {
	/// Connected: the game's feed is on.
	Up,
	Event(String),
	/// The connection ended.
	Gone,
}

/// The connection while the game runs; dropping it closes it.
pub struct Client {
	_stop: watch::Sender<bool>,
}

/// Connects to the feed on `port`, and again after every failure or end, until dropped; its news goes on the games'
/// channel.
pub fn start(port: u16, news: mpsc::UnboundedSender<super::News>) -> Client {
	let (stop, stopped) = watch::channel(false);
	tokio::spawn(run(port, news, stopped));
	Client { _stop: stop }
}

async fn run(port: u16, games: mpsc::UnboundedSender<super::News>, mut stop: watch::Receiver<bool>) {
	let news = |news: News| {
		let _ = games.send(super::News::Rocket(news));
	};
	loop {
		let connected = tokio::select! {
			connected = TcpStream::connect(("127.0.0.1", port)) => connected,
			_ = stop.changed() => return,
		};
		if let Ok(mut stream) = connected {
			news(News::Up);
			let (mut frames, mut buf) = (Frames::default(), vec![0u8; 16 * 1024]);
			loop {
				let read = tokio::select! {
					read = stream.read(&mut buf) => read,
					_ = stop.changed() => return,
				};
				let Some(bytes) = read.ok().filter(|&n| n > 0).and_then(|n| buf.get(..n)) else { break };
				for event in frames.push(bytes) {
					news(News::Event(event));
				}
			}
			news(News::Gone);
		}
		tokio::select! {
			() = tokio::time::sleep(RETRY) => {}
			_ = stop.changed() => return,
		}
	}
}

/// The install the Epic Games Launcher knows: the manifest whose launch executable is `RocketLeague.exe` (its
/// `AppName` is not relied on).
fn from_epic() -> Option<PathBuf> {
	let image = Watched::RocketLeague.windows_images().first()?;
	epic::items().into_iter().filter(|item| item.launches(image)).find_map(|item| item.install_dir())
}

/// Rocket League's folder for the ini (`<install>/TAGame/Config`): a Steam install, an Epic one, or Epic's default;
/// none outside Windows, where the game does not run.
pub fn config_dir() -> Option<PathBuf> {
	if !cfg!(windows) {
		return None;
	}
	let epic_default = || Some(PathBuf::from(r"C:\Program Files\Epic Games\rocketleague")).filter(|p| p.is_dir());
	let install = steam::game_dir(&Watched::RocketLeague, true).or_else(from_epic).or_else(epic_default)?;
	Some(install.join("TAGame").join("Config")).filter(|d| d.is_dir())
}
