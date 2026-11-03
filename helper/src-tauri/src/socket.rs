//! The local socket (rust-helper `protocol-and-mirror.md`, "The emitter and the local socket"): a Unix domain socket
//! `helper.sock` in the data folder on macOS, the named pipe `questpump-helper` on Windows, open only to the player's
//! own account (a pipe's default security lets every local account read it, and the lines carry heart rates). A
//! client gets the `hello` line, then every line as it happens (no replay: the row store is the history), and may send
//! commands the other way. A client whose write fails is dropped.

use std::path::Path;

#[cfg(not(windows))]
use interprocess::local_socket::GenericFilePath;
#[cfg(windows)]
use interprocess::local_socket::GenericNamespaced;
use interprocess::local_socket::ListenerOptions;
use interprocess::local_socket::tokio::prelude::*;
use questpump_core::protocol::SocketCommand;
use tokio::io::{AsyncBufReadExt as _, AsyncWriteExt as _, BufReader};
use tokio::sync::{broadcast, mpsc};

/// The Unix domain socket in the data folder, inside the player's own Library folder, which other accounts cannot open.
#[cfg(not(windows))]
fn options(dir: &Path) -> std::io::Result<ListenerOptions<'static>> {
	Ok(ListenerOptions::new().name(dir.join("helper.sock").into_os_string().to_fs_name::<GenericFilePath>()?))
}

/// The named pipe, its access limited to the player's account and the system (SDDL: a protected DACL of two ACEs).
#[cfg(windows)]
fn options(_dir: &Path) -> std::io::Result<ListenerOptions<'static>> {
	use interprocess::os::windows::local_socket::ListenerOptionsExt as _;
	use interprocess::os::windows::security_descriptor::SecurityDescriptor;

	let sid = crate::elevate::user_sid().map_err(std::io::Error::other)?;
	let sddl =
		widestring::U16CString::from_str(format!("D:P(A;;GA;;;SY)(A;;GA;;;{sid})")).map_err(std::io::Error::other)?;
	let name = "questpump-helper".to_ns_name::<GenericNamespaced>()?;
	Ok(ListenerOptions::new().name(name).security_descriptor(SecurityDescriptor::deserialize(&sddl)?))
}

/// Serves the socket until the helper quits. `hello` builds the first line for each client.
pub async fn serve(
	dir: &Path,
	lines: broadcast::Sender<String>,
	hello: impl Fn() -> String + Send + Sync + 'static,
	commands: mpsc::UnboundedSender<SocketCommand>,
) -> std::io::Result<()> {
	// a socket file a crashed helper left behind is taken over (single-instance keeps a second helper out)
	let listener = options(dir)?.try_overwrite(true).create_tokio()?;
	let hello = std::sync::Arc::new(hello);
	loop {
		let conn = match listener.accept().await {
			Ok(c) => c,
			Err(e) => {
				log::warn!("socket: a client could not connect: {e}");
				tokio::time::sleep(std::time::Duration::from_millis(100)).await; // out of descriptors: never spin
				continue;
			}
		};
		// the two tasks read and write the one stream through `&Stream` (interprocess 2.4.7 deprecates its split
		// halves for this), which closes once both have ended
		let stream = std::sync::Arc::new(conn);
		let writer = std::sync::Arc::clone(&stream);
		let mut rx = lines.subscribe();
		let first = hello();
		tokio::spawn(async move {
			let mut send = &*writer;
			if send.write_all(format!("{first}\n").as_bytes()).await.is_err() {
				return;
			}
			loop {
				match rx.recv().await {
					Ok(line) => {
						if send.write_all(format!("{line}\n").as_bytes()).await.is_err() {
							return;
						}
					}
					Err(broadcast::error::RecvError::Lagged(n)) => log::warn!("socket: a slow client missed {n} lines"),
					Err(broadcast::error::RecvError::Closed) => return,
				}
			}
		});
		let commands = commands.clone();
		tokio::spawn(async move {
			let mut reader = BufReader::new(&*stream).lines();
			while let Ok(Some(line)) = reader.next_line().await {
				match serde_json::from_str::<SocketCommand>(&line) {
					Ok(cmd) => {
						let _ = commands.send(cmd);
					}
					Err(e) => log::warn!("socket: ignored a command that does not read ({e}): {line}"),
				}
			}
		});
	}
}
