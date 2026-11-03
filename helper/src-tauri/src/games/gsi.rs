//! The Game State Integration listener (docs/plan.md slice 35; rust-helper rule 8): plain HTTP on 127.0.0.1 alone
//! (never `[::1]`, never the network: no firewall rule and no Local Network grant), `POST /cs2` and `POST /dota` as each
//! game's cfg names them, a body of at most 64 KiB, answered at once with an empty 200 (the game sends no new post
//! while one is in flight). Each post is read for its token and its phase alone (`questpump_core::gsi::read`) and
//! dropped; a wrong token gets 401 and a log line at most once a minute, without the body. Bound and served as the LAN
//! API is: its listener from `lan::listen` (on Windows its port is its alone), hyper-util's builder with a 30-s header
//! timer. It runs while CS2 or Dota 2 is set up ("Ask after each match", slice 56).

use std::net::{Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

use axum::Router;
use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, State};
use axum::http::StatusCode;
use axum::routing::post;
use hyper_util::rt::{TokioExecutor, TokioIo, TokioTimer};
use hyper_util::server::conn::auto;
use hyper_util::service::TowerToHyperService;
use questpump_core::gsi::{self, Gsi, Refused};
use tokio::net::TcpListener;
use tokio::sync::{mpsc, watch};
use tower::Service as _;

/// The largest post: a `map` block is well under a kilobyte, and a game that sends more than it was asked for still
/// fits.
const BODY_BYTES: usize = 64 * 1024;
/// A refused post is logged at most this often.
const NOTE_EVERY: Duration = Duration::from_mins(1);

/// What a post says: its game and its phase (none in the menus).
pub type Heard = (Gsi, Option<String>);

/// The listener while it runs; dropping it stops it.
pub struct Server {
	/// The port it listens on, which the cfg files name.
	pub port: u16,
	_stop: watch::Sender<bool>,
}

struct Listening {
	token: String,
	heard: mpsc::UnboundedSender<Heard>,
	/// When a refused post was last logged.
	noted: Mutex<Option<Instant>>,
}

/// Listens on 127.0.0.1 at `port`, or at a port the system picks when none is kept or another program holds it.
///
/// # Errors
/// No port of the loopback could be bound.
pub fn start(port: Option<u16>, token: String, heard: mpsc::UnboundedSender<Heard>) -> std::io::Result<Server> {
	let on = |port| crate::lan::listen(SocketAddr::from((Ipv4Addr::LOCALHOST, port)));
	let listener = match port.map(on) {
		Some(Ok(listener)) => listener,
		Some(Err(e)) => {
			log::info!("games: the listener's port is taken ({e}): another one, and the games' cfg files with it");
			on(0)?
		}
		None => on(0)?,
	};
	let port = listener.local_addr()?.port();
	let (stop, stopped) = watch::channel(false);
	let state = Arc::new(Listening { token, heard, noted: Mutex::new(None) });
	tokio::spawn(serve(listener, state, stopped));
	log::info!("games: listening on 127.0.0.1:{port}");
	Ok(Server { port, _stop: stop })
}

/// One post: its token checked and its phase passed on, or refused.
fn take(state: &Listening, game: Gsi, body: &[u8]) -> StatusCode {
	match gsi::read(game, body, &state.token) {
		Ok(phase) => {
			let _ = state.heard.send((game, phase));
			StatusCode::OK
		}
		Err(why) => {
			let mut noted = state.noted.lock().unwrap_or_else(PoisonError::into_inner);
			if noted.is_none_or(|at| at.elapsed() >= NOTE_EVERY) {
				*noted = Some(Instant::now());
				log::warn!("games: a post to {} refused: {why}", game.path());
			}
			match why {
				Refused::Token => StatusCode::UNAUTHORIZED,
				Refused::Body(_) => StatusCode::BAD_REQUEST,
			}
		}
	}
}

fn router(state: Arc<Listening>) -> Router {
	Router::new()
		.route(
			Gsi::CounterStrike.path(),
			post(|State(s): State<Arc<Listening>>, body: Bytes| async move { take(&s, Gsi::CounterStrike, &body) }),
		)
		.route(
			Gsi::Dota.path(),
			post(|State(s): State<Arc<Listening>>, body: Bytes| async move { take(&s, Gsi::Dota, &body) }),
		)
		.layer(DefaultBodyLimit::max(BODY_BYTES))
		.with_state(state)
}

/// Serves the posts until the server is dropped.
async fn serve(listener: TcpListener, state: Arc<Listening>, mut stop: watch::Receiver<bool>) {
	let mut make = router(state).into_make_service();
	let mut http = auto::Builder::new(TokioExecutor::new()).http1_only();
	http.http1().timer(TokioTimer::new()).header_read_timeout(Duration::from_secs(30));
	loop {
		let accepted = tokio::select! {
			accepted = listener.accept() => accepted,
			_ = stop.changed() => return,
		};
		let tcp = match accepted {
			Ok((tcp, _)) => tcp,
			Err(e) => {
				log::warn!("games: accept: {e}");
				tokio::time::sleep(Duration::from_millis(100)).await; // out of file descriptors: no hot loop
				continue;
			}
		};
		let Ok(svc) = make.call(&tcp).await;
		let http = http.clone();
		tokio::spawn(async move {
			let _ = http.serve_connection(TokioIo::new(tcp), TowerToHyperService::new(svc)).await;
		});
	}
}
