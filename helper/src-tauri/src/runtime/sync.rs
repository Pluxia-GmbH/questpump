//! The sync on the computer (docs/plan.md slice 60; contract/sync-api.yaml; questpump-core `sync` owns the keys, the
//! codes and the batches, `far` the other computers' rows): this computer's side of the one sync every device of the
//! player shares. Sync is on by default (`sync::Account::on`); off, it sends nothing and takes nothing in. The first
//! sync makes the account in silence (`POST /accounts` with a fresh R), and R and the device token stay in the system's
//! secret store with the pairing key (`crate::secrets`). One request at a time goes to the server, off the loop through
//! the one HTTPS client (`crate::feedback::client`: no `Origin`, the helper's User-Agent), each answer back on the loop
//! as `News`:
//!
//! - **Push**, a few seconds after the stores change: the device batch when its body changed (this computer's name,
//!   kind `desktop`, its engine profile, the shared body with each field's change time as `/v1/body` merges it, each
//!   character's newest game sheet as a book carries it, `sync::carried`, and its install id `hid`), then a `rows`
//!   batch of what it made since its last (the game's rows from `rows.jsonl`, catch-ups and banks among them, and the
//!   rows of its own app pumps, the body notes left out), each text with all its copies, with `chars` (each character
//!   index its game rows use to the GUID), split under `MAX_SEALED`. A batch keeps its seq and bytes until the server
//!   took it; a 409 starts the seq after `last_seq`. Rows it took in (a phone's, another computer's, a backup file's,
//!   another computer's reset this computer took as its own) are never sent: their maker sends them.
//! - **Pull**, at the start, after each push, at each nudge of the live socket and every `PULL_EVERY`: the batches
//!   after the cursor, page by page, each opened, checked against its listing and taken in (`take_listed`): a phone's app
//!   rows as the LAN's are taken, another computer's own app rows the same way and its game rows into the far store
//!   (placed by the batch's `chars`), which every mirror carries as rows of another source, its reset taken as this
//!   computer's own (`Session::take_far`); a device batch's body fields into the shared body and its characters into
//!   the far store's numbering. The cursor moves once the page is in; a batch of a newer version is kept unread
//!   (`menu.sync.newer`) and read again after an update; one that does not open or read is skipped.
//! - **The account's states** (contract/sync-api.yaml): a 401 on any call with the token removes this computer (Sync
//!   off, the token gone, the account and R kept: `sync::Account::removed`, `menu.sync.removed`); Sign out the same once
//!   `DELETE /session` answered 204; Delete account forgets the account and R once `DELETE /account` answered 204; a 404
//!   on `POST /session` for the account this computer holds forgets it and, Sync on, makes a new one at once, the Sync
//!   card saying `menu.sync.deleted` until that one's first sync, and a 403 there is its R replaced while it was out
//!   (below). Email, Restore code and Sign out read whether this computer holds an account and a token
//!   (`Account::held`), whatever the switch.
//! - **The account's new key** (contract/sync-api.yaml, "The account's new key"; the release security review's M5):
//!   each session, page of batches and `GET account/keys` brings the keys' news (`KeyNews`), which a live message's
//!   `keys` moving pulls again. This computer registers its own P-256 key (`Account::device_secret`, made once) with
//!   its MAC under R (`PUT devices/self/key`) after each session and each new R, and whenever the news shows it missing
//!   or under another kid. While a removal makes a new R due it makes one (`sync_rekey`: R' kept beside R before it
//!   goes, a wrap for each other device whose MAC holds under the R its kid names, `prev`, its own MAC under R'), takes
//!   it as R at the 204 or at a read naming its kid, and drops it only at a read naming another kid at a generation
//!   past the one it was made from (`take_news`): an answer that tells nothing sends the same body again (`Posted`), a
//!   409 or a refusal reads first, a start that kept R' posts it again with fresh wraps, and a join its R no longer
//!   opens (403) tries R' (`join_next`). A wrap for this computer, or a push or a registration answered 409 `kid`,
//!   reads `GET account/keys`: the wrap opened with its own key, its kid the account's and the chain checked link by
//!   link back to the R it holds (`walk_chain`), then R replaced and the batches not yet kept sealed again under it; a
//!   wrap that does not open or check signs this computer out (`menu.sync.removed`). The earlier Rs the chain gives
//!   open the batches sealed under older kids and check a MAC made under one (`SyncState::older`, never saved). A join
//!   held until the new R is in (409 `rekey`) says so on the page that asked (`menu.sync.rekey`), and so does a 403 on
//!   joining its own account again (its R replaced while it was out: Sync off, never a new account; an email sign-in's
//!   join on the Email page, its next request code bound to the account). A push answered 409 `kid` before any news
//!   reads the keys first. One rekey is out at a time (`SyncState::rekey_out`), and no reset or join lets a call or a
//!   read's news beside it.
//! - **Live**: one WebSocket while the app runs, its token as the first message, which says only that the cursor moved,
//!   by its `pro` counter that the account's Pro changed, or by its `keys` that the account's keys moved.
//! - **Pro** (slice 64): `GET pro` at the start, after each new session, at a live message whose `pro` differs from the
//!   last this connection heard (the first one heard counts) and every `PRO_EVERY`, never at each message; its token
//!   and status to `runtime/pro.rs`. Pro's page's Manage Pro asks `POST pro/portal` and opens its answer's address in
//!   the browser (Get Pro opens questpump.com's Pro page itself, `runtime/pro.rs`).
//!
//! **The `menu.sync` topic**: `{on, line: "syncing"|"last"|"waiting"|"failed"|"newer"|"full"|"deleted"|"removed"|
//! "rekey"|null, last: unix ms|null, devices: [{device, kind, name|null, seen_at, this}]}`;
//! **`menu.email`**: `{step: "field"|"code"|"taken"|"need_key", purpose: "link"|"change"|"signin", email, address,
//! linked, account, busy, error: "invalid"|"wrong"|"expired"|"limit"|"failed"|"rekey"|null, tries_left, sent_at, said:
//! {seq, what: "linked"|"deleted"|"out"}|null, request: QR|null,
//! linking: "linking"|"linked"|"failed"|"other"|"rekey"|null, out_failed}` (`address` null where another device linked
//! the email; `out_failed` a Sign out, Remove email or Delete account the server did not answer; `other` an answer of
//! another account than the email's, refused; `rekey` a join the account holds until its new key is in);
//! **`menu.restore`**: `{code: text|null, busy, error: "wrong"|"not_found"|"failed"|"rekey"|null, done: seq}`;
//! **`menu.linkdevice`**: `{code: QR|null, kind: "account"|"request"|null,
//! state: "linking"|"linked"|"failed"|"other"|"rekey"|null, check: six digits|null}` (`check` while an answer no email
//! sign-in binds waits for the player's Yes, join or No, the code hidden then).
//! **Intents**: `sync.look`, `sync.switch {on}`, `sync.remove {device}`, `sync.link.start`, `sync.link.stop`,
//! `sync.link.join`, `sync.link.refuse`;
//! `email.send {email}`, `email.code {code}`, `email.again`, `email.change`, `email.join`,
//! `email.signout`, `email.remove`, `email.delete`, `email.start`; `restore.look`, `restore.enter {code}`.

use std::collections::{BTreeMap, HashSet, VecDeque};
use std::fmt::Write as _;
use std::time::{Duration, Instant};

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use futures_util::{SinkExt as _, StreamExt as _};
use questpump_core::backup::{self, GameRow, is_body_note, unstamp};
use questpump_core::body::Body;
use questpump_core::codes::kind;
use questpump_core::far;
use questpump_core::link::Device;
use questpump_core::protocol::RowRecord;
use questpump_core::row::Row;
use questpump_core::sync::{
	self, Batch, DeviceBody, GameRows, KeyPair, Kind, LinkCode, MAX_SEALED, Opened, RowsBody, Sent, SyncKeys,
};
use serde::{Deserialize, Serialize};
use sha2::Digest as _;
use tokio::sync::{mpsc, oneshot};

use super::pro::ProLink;
use super::{Helper, now};
use crate::desk::Intent;
use crate::desk::pump::ran_here;
use crate::lan::{self, AccountState};
use crate::pairing::{self, Qr};

/// The sync's routes on questpump.com (contract/sync-api.yaml).
const BASE: &str = "https://questpump.com/api/sync/v1";
const LIVE: &str = "wss://questpump.com/api/sync/v1/live";
/// A pull at least this often while Sync is on.
const PULL_EVERY: Duration = Duration::from_mins(5);
/// A push this long after the stores changed, so a pump's rows go as one batch.
const PUSH_AFTER: Duration = Duration::from_secs(5);
/// The next try after a failure with no `Retry-After`.
const RETRY: Duration = Duration::from_mins(1);
/// The devices' list read again at most this often.
const DEVICES_EVERY: Duration = Duration::from_mins(1);
/// The live socket's first wait before it tries again, doubled up to `PULL_EVERY`.
const LIVE_FIRST: Duration = Duration::from_secs(5);
/// A link request's answer read at most this often, and for this long (the server's).
const LINK_POLL: Duration = Duration::from_secs(2);
const LINK_LIFE: Duration = Duration::from_mins(10);
/// The batches one page asks for.
const PAGE: u32 = 100;
/// The most an answer of the server may be (a page is at most 4 MiB).
const ANSWER_MOST: usize = 8 << 20;
/// The longest a device name taken in may be, in characters.
const NAME_MOST: usize = 64;
/// The account's Pro asked again at least this often (`GET pro`), and after a failure with no `Retry-After` this long.
const PRO_EVERY: Duration = Duration::from_hours(24);
const PRO_RETRY: Duration = Duration::from_mins(5);

/// The topics.
pub(crate) const SYNC: &str = "menu.sync";
pub(crate) const EMAIL: &str = "menu.email";
pub(crate) const RESTORE: &str = "menu.restore";
pub(crate) const LINKDEVICE: &str = "menu.linkdevice";

/// A server's answer: its status, its `Retry-After` in seconds, its body.
pub(crate) struct Answer {
	status: u16,
	retry: Option<u64>,
	body: Vec<u8>,
}

/// One call's outcome: the answer, or why none came (no connection, a timeout).
type Called = Result<Answer, String>;

/// Why a session was asked (no `Debug`: it holds R).
#[derive(Clone)]
pub(crate) enum Joining {
	/// No account yet: a new one with this R (`POST /accounts`).
	New([u8; sync::ROOT]),
	/// Back into the account whose R this computer holds (no token: a start, Sync turned on again).
	Again,
	/// Into an account with its R: from a restore code, a link answer or a phone's Add (`PUT /v1/account`).
	Other { account: Option<String>, r: [u8; sync::ROOT], from: Via },
}

/// Where another account's R came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Via {
	Restore,
	Link,
	Lan,
}

/// What the server answered, back on the loop, with the generation of the token it went with (a sign-out or a join
/// since makes it stale).
pub(crate) enum News {
	Session(u64, Joining, Called),
	Pushed(u64, Called),
	Page(u64, Called),
	Devices(u64, Called),
	Removed(u64, Called),
	/// Signed out, the email removed and the account deleted: whatever the token's generation, each holds.
	Out(Called),
	Unlinked(Called),
	Deleted(Called),
	/// A session ended beside the queue (logged only): the old account's after a join, or this computer's after a wrap
	/// it did not take.
	Left(Called),
	Email(Mail, Called),
	/// A link request's answer, with the request's id r.
	Link([u8; sync::REQUEST_ID], Called),
	/// A live message: its cursor, whether its `pro` differs from the last this connection heard, and its `keys` (the
	/// account's key generation; none from a server before the account's new key).
	Nudge(u64, u64, bool, Option<u64>),
	/// The account's keys (`GET account/keys`).
	Keys(u64, Called),
	/// This computer's key registered (`PUT devices/self/key`), with what it sent.
	Registered(u64, KeyPut, Called),
	/// The account's new R posted (`POST account/rekey`), with the keys' generation it was made from.
	Rekeyed(u64, u64, Called),
	/// The live socket closed (its close code), and whether it had connected.
	LiveEnded(u64, Option<u16>, bool),
	/// The account's Pro (`GET pro`).
	Pro(u64, Called),
	/// Manage Pro's address (`POST pro/portal`).
	Portal(Called),
}

/// What became of a batch the server listed.
enum Taken {
	In,
	/// Made by a newer QuestPump (its version): kept unread.
	Newer(u32),
	/// It did not open, inflate, read or match its listing, or names another key.
	Skipped,
	/// A store could not write what it took (the cursor stays before it).
	Unsaved,
}

/// An email call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mail {
	Code,
	Link,
	SignIn,
}

/// The state line of the Sync card.
#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Line {
	Syncing,
	Last,
	Waiting,
	Failed,
	Newer,
	Full,
	Deleted,
	Removed,
	/// A join the account holds until another device makes its new R (a 409 `rekey`), or this computer's R replaced
	/// while it was out (a 403 on its own account's join).
	Rekey,
	/// The secrets did not read at this start (`Helper::secrets_unread`): nothing syncs this run, and the switch waits
	/// for the next start.
	Unread,
}

/// A device of the account as the server lists it.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[expect(clippy::struct_field_names, reason = "the server's field names (contract/sync-api.yaml `Device`)")]
pub(crate) struct Seen {
	device: String,
	kind: String,
	made_at: i64,
	seen_at: i64,
}

/// A batch on its way: its seq, kind and sealed bytes (the same on every retry), and what is sent once the server
/// took it: the stores' marks (the game's, the app rows') after the last rows batch, the device batch's digest.
struct Out {
	seq: u64,
	kind: Kind,
	sealed: String,
	marks: Option<(Sent, Sent)>,
	digest: Option<String>,
}

/// The Email page's steps.
#[derive(Serialize, Debug, Clone, Copy, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Step {
	#[default]
	Field,
	Code,
	Taken,
	NeedKey,
}

#[derive(Serialize, Debug, Clone, Copy, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Purpose {
	#[default]
	Link,
	Change,
	Signin,
}

impl Purpose {
	fn word(self) -> &'static str {
		match self {
			Self::Link => "link",
			Self::Change => "change",
			Self::Signin => "signin",
		}
	}
}

#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Problem {
	Invalid,
	Wrong,
	Expired,
	Limit,
	Failed,
	NotFound,
	/// A join the account holds until another device makes its new R (a 409 `rekey`).
	Rekey,
}

#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Said {
	Linked,
	Deleted,
	Out,
}

#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum LinkState {
	Linking,
	Linked,
	Failed,
	/// An answer that names another account than the one the email sign-in found: refused, a new request shown.
	Other,
	/// A join the account holds until another device makes its new R (a 409 `rekey`).
	Rekey,
}

/// The Email page's flow between its intents and the server's answers.
#[derive(Default)]
pub(crate) struct EmailFlow {
	step: Step,
	purpose: Purpose,
	email: Option<String>,
	sent_at: Option<i64>,
	error: Option<Problem>,
	tries_left: Option<u32>,
	busy: bool,
	/// The account an email sign-in found, which a restore code or a link answer joins.
	found: Option<String>,
	said: Option<(u64, Said)>,
	seq: u64,
	/// A Sign out, Remove email or Delete account the server did not answer: the linked view says so.
	out_failed: bool,
}

/// A link request this computer shows while it lacks R: its id r, the key pair its answer is sealed to (the public key
/// in the code, the secret only here), when it ends, the account it may join and the answer that waits for the player.
struct Request {
	id: [u8; sync::REQUEST_ID],
	key: KeyPair,
	until: Instant,
	qr: Option<Qr>,
	/// The account the email sign-in had found when the request was made, the only one its answer joins, at once; none
	/// without one, and an answer then waits for the player (`held`).
	binds: Option<String>,
	/// An answer under no binding, opened and waiting with its check code for Yes, join or No (`sync.link.join`,
	/// `sync.link.refuse`): whoever saw the code on the screen could have answered first, so only the player's press on
	/// the code their own phone shows joins it.
	held: Option<sync::LinkOpened>,
}

/// A read of the account's chain going back page by page (`GET account/keys?from=`; contract/sync-api.yaml, "Taking
/// the wrap"): the account's kid when it began, the kid the next page starts at, the R the next link's `prev` opens
/// under, the earlier Rs it opened so far by kid, and the new R a wrap brought, taken only once the walk reaches the
/// kid this computer holds (none: a walk for the earlier Rs alone). No `Debug`: it holds Rs.
struct Walk {
	head: Option<String>,
	from: String,
	at: [u8; sync::ROOT],
	opened: Vec<(String, [u8; sync::ROOT])>,
	wrap: Option<[u8; sync::ROOT]>,
}

/// This computer's rekey as it last posted it (contract/sync-api.yaml, "Making the new R"): its R' (base64url, as
/// `Account::next_r` holds it), the body's bytes and the keys' generation the body was made from. While no answer
/// tells, the same bytes go again (`SyncState::resend`), which the server answers 204 once a copy is in; only a read
/// that names another kid than R''s at a generation past this one drops R' (`take_news`). Never saved: a start that
/// kept R' posts it again with fresh wraps. No `Debug`: the body holds the wraps and R sealed under R'.
struct Posted {
	next: String,
	body: Vec<u8>,
	generation: u64,
}

/// The sync's state in the loop, beside the account in settings.json.
#[expect(clippy::struct_excessive_bools, reason = "independent waits of the sync's one queue, never one state")]
pub(crate) struct SyncState {
	client: Option<reqwest::Client>,
	news: mpsc::UnboundedSender<News>,
	/// The token's generation: a join, a sign-out or a removal makes every answer of the last one stale.
	generation: u64,
	/// A session, push, pull or devices' call is out (one at a time).
	busy: bool,
	pull_due: bool,
	push_at: Option<Instant>,
	pulled_at: Option<Instant>,
	devices_at: Option<Instant>,
	wait_until: Option<Instant>,
	out: VecDeque<Out>,
	/// The account is full (a 413): nothing more is pushed in this run.
	full: bool,
	line: Option<Line>,
	/// A line said in place of syncing until the account's first sync ends (`menu.sync.deleted` while a new account
	/// replaces one that is gone).
	told: Option<Line>,
	devices: Vec<Seen>,
	live: Option<tokio::task::JoinHandle<()>>,
	live_at: Option<Instant>,
	live_wait: Duration,
	/// Pages that show a link request or link code: while one does, a request is read.
	link_wanted: bool,
	request: Option<Request>,
	link_busy: bool,
	linked_at: Option<Instant>,
	linking: Option<LinkState>,
	email: EmailFlow,
	restore_busy: bool,
	restore_error: Option<Problem>,
	restore_done: u64,
	/// A phone's Add waits for the join's answer.
	lan_join: Option<oneshot::Sender<Option<AccountState>>>,
	/// When the account's Pro is asked next (`GET pro`): at the start, after a new session and at a live message whose
	/// `pro` changed at once; none while it is out.
	pro_at: Option<Instant>,
	/// The keys' news last heard (a session, a page of batches, `GET account/keys`); none before the first of this
	/// session, and from a server before the account's new key, which sends none and so asks for no key's call.
	heard: Option<KeyNews>,
	/// The account's earlier Rs by kid, given by the chain or replaced here: what opens a batch sealed under an older
	/// kid and checks a MAC made under one. Never saved: the chain gives them again.
	older: BTreeMap<String, [u8; sync::ROOT]>,
	/// `GET account/keys` due: a removal made a new R due, a push or a registration named another kid, a rekey was
	/// answered 409 or refused, the chain's next page, or an older kid that `older` lacks (`chain_wanted`).
	keys_due: bool,
	walk: Option<Walk>,
	/// An older kid a batch or a device's MAC names is not in `older`: the next keys' read walks the chain from this
	/// computer's R back.
	chain_wanted: bool,
	/// The kid of the R a whole walk of the chain went back from: an older kid unknown since opens nothing.
	chain_read: Option<String>,
	/// The keys' generation of a rekey the server refused: none is made again from it, only from news that moved.
	refused_generation: Option<u64>,
	/// No rekey before this (a 429: the account's rekeys of the day, or too many requests), the rest of the queue
	/// going on.
	rekey_wait: Option<Instant>,
	/// This computer's rekey as last posted, while its R' waits (`Helper::posted` checks it is still that R').
	posted: Option<Posted>,
	/// The posted body goes again unchanged before any other call of the keys: no answer told what became of it (none
	/// came, a server's failure, a 429, or a reset that left it unread).
	resend: bool,
	/// A rekey is out: from its post until its answer or the call's timeout, whatever reset or join came since, no
	/// call of the queue goes and no news drops R' (`take_news`), so only a read sent after that answer settles it.
	rekey_out: bool,
	/// A push answered 409 `kid`, which only a server that keeps the account's keys sends: the keys read goes even
	/// before this session heard any news (a start during a desktop pump skips the pulls that bring it), and no push
	/// before it.
	kid_refused: bool,
}

impl SyncState {
	/// The state, and where the server's answers come back to the loop.
	pub(crate) fn new(client: Option<reqwest::Client>) -> (Self, mpsc::UnboundedReceiver<News>) {
		let (news, answers) = mpsc::unbounded_channel();
		let state = Self {
			client,
			news,
			generation: 0,
			busy: false,
			pull_due: true,
			push_at: Some(Instant::now()),
			pulled_at: None,
			devices_at: None,
			wait_until: None,
			out: VecDeque::new(),
			full: false,
			line: None,
			told: None,
			devices: Vec::new(),
			live: None,
			live_at: None,
			live_wait: LIVE_FIRST,
			link_wanted: false,
			request: None,
			link_busy: false,
			linked_at: None,
			linking: None,
			email: EmailFlow::default(),
			restore_busy: false,
			restore_error: None,
			restore_done: 0,
			lan_join: None,
			pro_at: Some(Instant::now()),
			heard: None,
			older: BTreeMap::new(),
			keys_due: false,
			walk: None,
			chain_wanted: false,
			chain_read: None,
			refused_generation: None,
			rekey_wait: None,
			posted: None,
			resend: false,
			rekey_out: false,
			kid_refused: false,
		};
		(state, answers)
	}

	/// The keys' state of this session forgotten (a new session, the token gone, Sync off): the next news tells it
	/// again. A rekey's posted body stays and goes again, since the answer this reset leaves unread may be its own.
	fn keys_forget(&mut self) {
		(self.heard, self.walk, self.chain_read) = (None, None, None);
		(self.refused_generation, self.rekey_wait) = (None, None);
		(self.keys_due, self.chain_wanted, self.kid_refused) = (false, false, false);
		self.older.clear();
		self.resend = self.posted.is_some();
	}

	/// A call to the server off the loop, its answer back as `news`.
	fn call(
		&self,
		method: reqwest::Method,
		path: String,
		token: Option<String>,
		body: Option<Vec<u8>>,
		news: impl FnOnce(Called) -> News + Send + 'static,
	) {
		let Some(client) = self.client.clone() else {
			let _ = self.news.send(news(Err("no HTTPS client".to_owned()))); // copy-lint: not shown (a log line)
			return;
		};
		let tx = self.news.clone();
		tokio::spawn(async move {
			let answer = request(&client, method, &path, token.as_deref(), body).await;
			let _ = tx.send(news(answer));
		});
	}

	fn stop_live(&mut self) {
		if let Some(live) = self.live.take() {
			live.abort();
		}
	}

	/// A call is out: the Sync card says syncing, or the line told in its place (`told`).
	fn syncing(&mut self) {
		self.line = Some(self.told.unwrap_or(Line::Syncing));
	}
}

/// One request to the sync's routes: JSON in and out, the token as its bearer, no `Origin` (the client's).
async fn request(
	client: &reqwest::Client,
	method: reqwest::Method,
	path: &str,
	token: Option<&str>,
	body: Option<Vec<u8>>,
) -> Called {
	let mut req = client.request(method, format!("{BASE}{path}"));
	if let Some(token) = token {
		req = req.bearer_auth(token);
	}
	if let Some(body) = body {
		req = req.header(reqwest::header::CONTENT_TYPE, "application/json").body(body);
	}
	let why = |e: reqwest::Error| format!("{:#}", anyhow::Error::new(e.without_url()));
	let mut res = req.send().await.map_err(why)?;
	let status = res.status().as_u16();
	let retry = res
		.headers()
		.get(reqwest::header::RETRY_AFTER)
		.and_then(|v| v.to_str().ok())
		.and_then(|v| v.trim().parse::<u64>().ok());
	let mut body = Vec::new();
	while let Some(chunk) = res.chunk().await.map_err(why)? {
		if body.len() + chunk.len() > ANSWER_MOST {
			return Err(format!("an answer past {ANSWER_MOST} bytes")); // copy-lint: not shown (a log line)
		}
		body.extend_from_slice(&chunk);
	}
	Ok(Answer { status, retry, body })
}

/// The live socket: the token as its first message, then each `{"cursor":n,"pro":k,"keys":g}` the server sends as a
/// nudge, with whether its `pro` counter differs from the last this connection heard (none heard yet: the first that
/// comes does; a message without one, an older server's, does not) and its key generation, until it closes (its close
/// code: 4401 for a token it did not take).
async fn live(token: String, generation: u64, news: mpsc::UnboundedSender<News>) {
	use tokio_tungstenite::tungstenite::Message;
	use tokio_tungstenite::tungstenite::client::IntoClientRequest as _;
	use tokio_tungstenite::tungstenite::http::{HeaderValue, header};

	#[derive(Deserialize)]
	struct Moved {
		cursor: u64,
		pro: Option<u64>,
		keys: Option<u64>,
	}
	let mut connected = false;
	let mut heard_pro = None;
	let code = async {
		let mut request = LIVE.into_client_request().ok()?;
		request.headers_mut().insert(header::USER_AGENT, HeaderValue::from_static(crate::feedback::USER_AGENT));
		let dialled = tokio::time::timeout(Duration::from_secs(20), tokio_tungstenite::connect_async(request));
		let (mut ws, _) = dialled.await.ok()?.ok()?;
		let first = serde_json::json!({ "token": token }).to_string();
		ws.send(Message::text(first)).await.ok()?;
		connected = true;
		while let Some(message) = ws.next().await {
			match message.ok()? {
				Message::Text(text) => {
					if let Ok(Moved { cursor, pro, keys }) = serde_json::from_str(text.as_str()) {
						let changed = pro.is_some() && pro != heard_pro;
						heard_pro = pro.or(heard_pro);
						let _ = news.send(News::Nudge(generation, cursor, changed, keys));
					}
				}
				Message::Close(frame) => return frame.map(|f| u16::from(f.code)),
				_ => {}
			}
		}
		None
	}
	.await;
	let _ = news.send(News::LiveEnded(generation, code, connected));
}

/// The server's bodies this computer reads.
#[derive(Deserialize)]
struct Session {
	account: String,
	device: String,
	token: String,
	#[serde(default)]
	email: bool,
	keys: Option<KeyNews>,
}

/// A 409's body: why (`rekey`, `devices`, `seq`, `kid`, `key`, `generation`), and a push's `last_seq`.
#[derive(Deserialize)]
struct Refusal {
	reason: Option<String>,
	last_seq: Option<u64>,
}

impl Refusal {
	/// The reason of an answer's 409, none for another answer or one without.
	fn of(called: &Called) -> Option<String> {
		let answer = called.as_ref().ok().filter(|a| a.status == 409)?;
		serde_json::from_slice::<Self>(&answer.body).ok()?.reason
	}
}

#[derive(Deserialize)]
struct Page {
	batches: Vec<Listed>,
	more: bool,
	cursor: u64,
	keys: Option<KeyNews>,
}

/// The keys' news (`KeyNews`; `GET account/keys` gives the same, its wrap in place of `wrapped`): the account's kid
/// (none before its first rekey), its generation, whether a new R is due, whether a wrap waits for this computer, and
/// while due every device's key.
#[derive(Deserialize, Clone)]
struct KeyNews {
	kid: Option<String>,
	generation: u64,
	due: bool,
	#[serde(default)]
	wrapped: bool,
	#[serde(default)]
	devices: Vec<DeviceKey>,
}

/// A device and the key it registered, its MAC and the kid that MAC was made under, each none while it registered
/// none.
#[derive(Deserialize, Clone)]
struct DeviceKey {
	device: String,
	key: Option<String>,
	mac: Option<String>,
	kid: Option<String>,
}

/// `GET account/keys`' answer: the news, the wrap of the account's R for this computer, and a page of the chain from
/// `from` (else the account's kid) back, the newest first.
#[derive(Deserialize)]
struct Keys {
	kid: Option<String>,
	generation: u64,
	due: bool,
	wrap: Option<String>,
	#[serde(default)]
	devices: Vec<DeviceKey>,
	#[serde(default)]
	chain: Vec<KeyLink>,
	#[serde(default)]
	more: bool,
}

/// A link of the account's chain: a kid, the kid of the R it replaced and that R sealed under its seal key (base64).
#[derive(Deserialize)]
struct KeyLink {
	kid: String,
	prev_kid: String,
	prev: String,
}

/// `PUT devices/self/key`'s body (`KeyPut`): this computer's key (SEC1 uncompressed), its MAC under R and R's kid,
/// base64url.
#[derive(Serialize, Clone)]
pub(crate) struct KeyPut {
	key: String,
	mac: String,
	kid: String,
}

#[derive(Deserialize)]
struct Listed {
	cursor: u64,
	device: String,
	seq: u64,
	kid: String,
	/// `rows` or `device`; a kind this build does not know is skipped.
	kind: String,
	sealed: String,
}

#[derive(Deserialize)]
struct Devices {
	devices: Vec<Seen>,
	/// Whether an email holds the account (slice 60's double-check: a link made on one device reaches all).
	#[serde(default)]
	email: bool,
}

#[derive(Deserialize)]
struct Found {
	account: String,
}

#[derive(Deserialize)]
struct Tries {
	tries_left: Option<u32>,
}

#[derive(Deserialize)]
struct Answered {
	sealed: String,
}

/// `ProState`: the account's token, none without one, and its status (`active`, `trialing` and `past_due` stand); its
/// other fields are what the token says.
#[derive(Deserialize)]
struct ProAnswer {
	token: Option<String>,
	status: Option<String>,
}

/// `POST pro/portal`'s answer: the address the browser opens.
#[derive(Deserialize)]
struct ProUrl {
	url: String,
}

/// The `menu.sync` topic.
#[derive(Serialize)]
struct SyncTopic<'a> {
	on: bool,
	line: Option<Line>,
	last: Option<i64>,
	devices: Vec<DeviceRow<'a>>,
}

#[derive(Serialize)]
struct DeviceRow<'a> {
	device: &'a str,
	kind: &'a str,
	name: Option<&'a str>,
	seen_at: i64,
	this: bool,
}

#[derive(Serialize)]
#[expect(clippy::struct_excessive_bools, reason = "the page's facts, one yes or no a field, as the topic names them")]
struct EmailTopic<'a> {
	step: Step,
	purpose: Purpose,
	email: Option<&'a str>,
	address: Option<&'a str>,
	linked: bool,
	account: bool,
	busy: bool,
	error: Option<Problem>,
	tries_left: Option<u32>,
	sent_at: Option<i64>,
	said: Option<SaidTopic>,
	request: Option<&'a Qr>,
	linking: Option<LinkState>,
	out_failed: bool,
}

#[derive(Serialize)]
struct SaidTopic {
	seq: u64,
	what: Said,
}

#[derive(Serialize)]
struct RestoreTopic {
	code: Option<String>,
	busy: bool,
	error: Option<Problem>,
	done: u64,
}

#[derive(Serialize)]
struct LinkTopic<'a> {
	code: Option<Qr>,
	kind: Option<&'static str>,
	state: Option<LinkState>,
	check: Option<&'a str>,
}

/// The SHA-256 of bytes in hex: the device batch's digest.
fn digest(bytes: &[u8]) -> String {
	sha2::Sha256::digest(bytes).iter().fold(String::new(), |mut s, b| {
		let _ = write!(s, "{b:02x}");
		s
	})
}

/// A rows batch's body of these groups: each group the copies of one text, a game row's or an app row's; `chars` each
/// character index its game rows use with its GUID (`guids`: the helper's numbering, a placeholder without one left
/// out, whose rows another computer places once this one's `device` batch names it).
fn rows_body(groups: &[Group], hid: &str, guids: &BTreeMap<u8, String>) -> RowsBody {
	let mut body = RowsBody { hid: Some(hid.to_owned()), ..RowsBody::default() };
	let mut app: Vec<Row> = Vec::new();
	let mut game: Vec<GameRow> = Vec::new();
	for group in groups {
		match group {
			Group::Game(rows) => game.extend(rows),
			Group::App(rows) => app.extend(rows),
		}
	}
	app.sort_by_key(|r| r.ts); // stable, as a book's
	body.chars = game.iter().filter_map(|r| Some((r.char.to_string(), guids.get(&r.char)?.clone()))).collect();
	body.rows = app.iter().map(ToString::to_string).collect();
	body.game = GameRows { rows: game };
	body
}

/// The copies of one text a rows batch carries together.
enum Group {
	Game(Vec<GameRow>),
	App(Vec<Row>),
}

/// The rows of a store that go, grouped by text: every copy of each text that has a copy past `from` (`keep` picks the
/// store's rows that go at all).
fn groups_of<T: Clone, K: std::hash::Hash + Eq + Clone>(all: &[T], from: usize, key: impl Fn(&T) -> K) -> Vec<Vec<T>> {
	let new: HashSet<K> = all.get(from..).unwrap_or_default().iter().map(&key).collect();
	let mut order: Vec<K> = Vec::new();
	let mut groups: std::collections::HashMap<K, Vec<T>> = std::collections::HashMap::new();
	for item in all {
		let k = key(item);
		if !new.contains(&k) {
			continue;
		}
		let group = groups.entry(k.clone()).or_insert_with(|| {
			order.push(k);
			Vec::new()
		});
		group.push(item.clone());
	}
	order.into_iter().filter_map(|k| groups.remove(&k)).collect()
}

/// A device's kind as the server names it, as the link names a phone: none for a computer (`desktop`), a browser
/// (`web`, which the server takes from web play on) and a kind this build does not know, none of them a device of the
/// link, whose app rows are kept as the computer's are.
fn sender(kind: &str) -> Option<Device> {
	match kind {
		"ios" => Some(Device::Ios),
		"android" => Some(Device::Android),
		_ => None,
	}
}

/// Whether text reads as an email address enough to send it a code (the server checks it again).
fn address(text: &str) -> Option<String> {
	let text = text.trim();
	let (user, host) = text.split_once('@')?;
	(!user.is_empty() && host.contains('.') && !host.starts_with('.') && text.len() <= 254 && !text.contains(' '))
		.then(|| text.to_owned())
}

/// A fresh P-256 key from the OS random source, with its secret (a link request's, this computer's own, a wrap's): a
/// random 32 bytes are one but for about one draw in 2^32.
fn fresh_key() -> ([u8; sync::SECRET], KeyPair) {
	loop {
		let mut secret = [0u8; sync::SECRET];
		rand::fill(&mut secret);
		if let Some(key) = KeyPair::from_bytes(&secret) {
			return (secret, key);
		}
	}
}

impl Helper {
	/// The account and its keys while Sync is on and in an account; none otherwise.
	fn sync_in(&self) -> Option<(String, String, String, SyncKeys)> {
		let a = &self.settings.sync;
		if !a.on {
			return None;
		}
		Some((a.account.clone()?, a.device.clone()?, a.token.clone()?, a.keys()?))
	}

	/// This computer's token while it is in an account, whatever Sync's switch says (`Account::held`): what the Email
	/// page's steps call with.
	fn held_token(&self) -> Option<String> {
		let a = &self.settings.sync;
		a.held().then(|| a.token.clone()).flatten()
	}

	/// At the start: a newer batch kept unread that this build reads is taken in again, and the whole account when
	/// `far.jsonl` lost rows (`far_lost`: the other computers' rows come back from their batches), the topics published,
	/// and the first pull and push due.
	pub(crate) fn sync_start(&mut self, far_lost: bool) {
		if far_lost && self.settings.sync.cursor > 0 {
			log::info!("sync: {} lost rows: the account taken in again from its start", far::FILE);
			self.settings.sync.cursor = 0;
			self.save_settings();
		}
		if let Some((cursor, v)) = self.settings.sync.newer
			&& v <= sync::VERSION
		{
			log::info!("sync: batches of version {v} read now: taken in again from cursor {cursor}");
			let a = &mut self.settings.sync;
			(a.cursor, a.newer) = (a.cursor.min(cursor.saturating_sub(1)), None);
			self.save_settings();
		}
		self.sync_publish_all();
		self.sync_drive();
	}

	/// The stores changed: a push soon.
	pub(crate) fn sync_changed(&mut self) {
		let soon = Instant::now() + PUSH_AFTER;
		self.sync.push_at = Some(self.sync.push_at.map_or(soon, |at| at.min(soon)));
	}

	/// What is due now, at most one call of the queue at a time: the session, the devices' list, the account's new
	/// key's calls (`sync_keys_step`), the account's Pro, a pull, a push; and the live socket and a link request's
	/// answer beside them; nothing in a run whose secrets did not read.
	pub(crate) fn sync_drive(&mut self) {
		if self.account_open("sync").is_err() {
			return;
		}
		self.link_drive();
		let a = &self.settings.sync;
		if !a.on || self.sync.client.is_none() {
			self.sync.stop_live();
			return;
		}
		let now = Instant::now();
		if self.sync.busy || self.sync.rekey_out || self.sync.wait_until.is_some_and(|at| at > now) {
			return;
		}
		if a.token.is_none() {
			self.sync_session();
			return;
		}
		self.live_drive(now);
		let pull = self.sync.pull_due || self.sync.pulled_at.is_none_or(|at| now >= at + PULL_EVERY);
		if self.sync.devices_at.is_none() {
			// the devices' kinds first: a phone's rows are kept with their phone, as the LAN's are
			self.sync_devices();
		} else if self.sync_keys_step(now) {
			// the keys before the rows: a batch sealed under an R the account no longer names goes nowhere
		} else if self.sync.pro_at.is_some_and(|at| at <= now) {
			self.sync_pro();
		} else if pull && !self.desk_pump_running() {
			self.sync_pull();
		} else if !self.sync.out.is_empty() || self.sync.push_at.is_some_and(|at| at <= now) {
			self.sync_push();
		} else if self.sync.devices_at.is_none_or(|at| now >= at + DEVICES_EVERY) {
			self.sync_devices();
		}
	}

	/// No token: back into the account this computer holds R of, else a new account. Its R is kept before it is
	/// asked for, so a retry whose answer was lost sends the same auth key, which the server joins to the account it
	/// made.
	fn sync_session(&mut self) {
		let a = &self.settings.sync;
		let joining = match (a.account.is_some(), a.root()) {
			(true, Some(_)) => Joining::Again,
			(false, Some(r)) => Joining::New(r),
			(_, None) => {
				let mut r = [0u8; sync::ROOT];
				rand::fill(&mut r);
				(self.settings.sync.account, self.settings.sync.r) = (None, Some(sync::text(&r)));
				self.save_settings();
				Joining::New(r)
			}
		};
		self.sync_join(joining);
	}

	/// `POST /accounts` for a new account, else `POST /session` with the auth key R gives; none in a run whose secrets
	/// did not read (each way in refuses it first, `account_open`).
	fn sync_join(&mut self, joining: Joining) {
		if let Err(why) = self.account_open("sync: a join") {
			log::warn!("{why}");
			return;
		}
		let a = &self.settings.sync;
		let (path, body) = match &joining {
			Joining::New(r) => {
				let auth = SyncKeys::derive(r).auth;
				("/accounts", serde_json::json!({ "auth": auth, "kind": sync::DESKTOP }))
			}
			Joining::Again => {
				let (Some(account), Some(keys)) = (a.account.clone(), a.keys()) else { return };
				("/session", serde_json::json!({ "account": account, "auth": keys.auth, "kind": sync::DESKTOP }))
			}
			Joining::Other { account, r, .. } => {
				// the old token's calls still out are stale: the join decides what comes next
				self.sync.generation += 1;
				self.sync.stop_live();
				self.sync.out.clear();
				let auth = SyncKeys::derive(r).auth;
				let mut body = serde_json::json!({ "auth": auth, "kind": sync::DESKTOP });
				if let Some(account) = account {
					body["account"] = serde_json::Value::String(account.clone());
				}
				("/session", body)
			}
		};
		self.sync.busy = true;
		self.sync.syncing();
		self.sync_publish();
		let generation = self.sync.generation;
		let bytes = serde_json::to_vec(&body).unwrap_or_default();
		self.sync.call(reqwest::Method::POST, path.to_owned(), None, Some(bytes), move |c| {
			News::Session(generation, joining, c)
		});
	}

	/// What the server answered, to its owner.
	pub(crate) fn on_sync(&mut self, news: News) {
		let current = |g: u64, s: &SyncState| g == s.generation;
		match news {
			News::Session(g, joining, called) if current(g, &self.sync) => self.on_session(&joining, &called),
			News::Pushed(g, called) if current(g, &self.sync) => self.on_pushed(&called),
			News::Page(g, called) if current(g, &self.sync) => self.on_page(&called),
			News::Devices(g, called) if current(g, &self.sync) => self.on_devices(&called),
			News::Removed(g, called) if current(g, &self.sync) => {
				match &called {
					Ok(a) if a.status == 401 => self.sync_removed(),
					// the removal made a new R due: the keys read at once, so this computer makes it
					Ok(a) if a.status == 204 => self.sync.keys_due = true,
					_ => {}
				}
				self.sync.devices_at = None;
			}
			News::Keys(g, called) if current(g, &self.sync) => self.on_keys(&called),
			News::Registered(g, put, called) if current(g, &self.sync) => self.on_registered(put, &called),
			News::Rekeyed(g, made_from, called) => {
				self.sync.rekey_out = false;
				if current(g, &self.sync) {
					self.on_rekeyed(made_from, &called);
				} else {
					// an answer of a token gone or a join since: the same body again while it is still this device's
					self.sync.resend = self.posted().is_some();
				}
			}
			News::Out(called) => self.on_out(&called, Said::Out),
			News::Unlinked(called) => self.on_unlinked(&called),
			News::Deleted(called) => self.on_out(&called, Said::Deleted),
			News::Left(called) => match called {
				Ok(a) => log::info!("sync: a session ended ({})", a.status),
				Err(e) => log::info!("sync: a session not ended: {e}"),
			},
			News::Email(mail, called) => self.on_email(mail, &called),
			News::Link(id, called) => self.on_link(id, &called),
			News::Nudge(g, cursor, pro_changed, keys) if current(g, &self.sync) => {
				self.sync.live_wait = LIVE_FIRST;
				let heard = self.sync.heard.as_ref().map(|n| n.generation);
				if cursor > self.settings.sync.cursor || keys.is_some_and(|k| heard != Some(k)) {
					self.sync.pull_due = true;
				}
				if pro_changed {
					self.sync.pro_at = Some(Instant::now());
				}
			}
			News::Pro(g, called) if current(g, &self.sync) => self.on_pro(&called),
			News::Portal(called) => self.on_portal(&called),
			News::LiveEnded(g, code, connected) if current(g, &self.sync) => {
				self.sync.live = None;
				// a socket that connected and ran starts the waits again; a token it refused does not
				if connected && code != Some(4401) {
					self.sync.live_wait = LIVE_FIRST;
				}
				self.sync.live_at = Some(Instant::now() + self.sync.live_wait);
				self.sync.live_wait = (self.sync.live_wait * 2).min(PULL_EVERY);
				if code == Some(4401) {
					// the token was not taken: the next pull hears why (a 401 removes it)
					self.sync.pull_due = true;
				}
			}
			// an answer of a token gone since
			News::Session(..)
			| News::Pushed(..)
			| News::Page(..)
			| News::Devices(..)
			| News::Removed(..)
			| News::Keys(..)
			| News::Registered(..)
			| News::Nudge(..)
			| News::LiveEnded(..) => {}
			// the account's Pro of a token gone since: asked again with the token that stands
			News::Pro(..) => {
				self.sync.pro_at.get_or_insert_with(Instant::now);
			}
		}
		self.sync_publish();
		self.sync_drive();
	}

	/// The live socket while in the account, opened again after it closed.
	fn live_drive(&mut self, now: Instant) {
		if self.sync.live.is_some() || self.sync.live_at.is_some_and(|at| at > now) {
			return;
		}
		let Some(token) = self.settings.sync.token.clone() else { return };
		let (generation, news) = (self.sync.generation, self.sync.news.clone());
		self.sync.live = Some(tokio::spawn(live(token, generation, news)));
	}

	/// A call failed: the line says so, and the queue waits `Retry-After` or `RETRY`.
	fn sync_failed(&mut self, called: &Called) {
		let (line, wait) = match called {
			Ok(a) => {
				log::info!("sync: answered {}", a.status);
				(Line::Failed, a.retry.map_or(RETRY, Duration::from_secs))
			}
			Err(e) => {
				log::info!("sync: no answer: {e}");
				(Line::Waiting, RETRY)
			}
		};
		self.sync.line = Some(line);
		self.sync.wait_until = Some(Instant::now() + wait);
	}

	fn on_session(&mut self, joining: &Joining, called: &Called) {
		self.sync.busy = false;
		let status = called.as_ref().map(|a| a.status).ok();
		let session = called
			.as_ref()
			.ok()
			.filter(|a| a.status == 200)
			.and_then(|a| serde_json::from_slice::<Session>(&a.body).ok());
		let Some(Session { account, device, token, email, keys }) = session else {
			self.session_refused(joining, status, called);
			return;
		};
		let r = match joining {
			Joining::New(r) | Joining::Other { r, .. } => *r,
			Joining::Again => self.settings.sync.root().unwrap_or_default(),
		};
		if let Some(old) = self.settings.sync.token.clone().filter(|t| *t != token) {
			// the old device is left once the new token works (this one did): its account keeps what it sent
			self.sync.call(reqwest::Method::DELETE, "/session".to_owned(), Some(old), None, News::Left);
		}
		log::info!("sync: in the account, device {device}");
		// an email sign-in's address is the account's own
		let signed =
			(email && self.sync.email.found.as_deref() == Some(&account)).then(|| self.sync.email.email.clone());
		self.settings.sync.joined(account, sync::text(&r), device, token, email);
		if let Some(address) = signed.flatten() {
			self.settings.sync.address = Some(address);
		}
		self.save_settings();
		self.sync.generation += 1;
		self.sync.stop_live();
		(self.sync.out, self.sync.full, self.sync.pull_due) = (VecDeque::new(), false, true);
		(self.sync.push_at, self.sync.devices_at, self.sync.live_at) = (Some(Instant::now()), None, None);
		// a new session registers this computer's key in it (`sync_keys_step`, `Account::joined` forgot its kid); a
		// rekey the old device posted is never this one's to send again (its MAC, its device): an R' that stays is
		// posted again with fresh wraps
		self.sync.posted = None;
		self.sync.keys_forget();
		if let Some(news) = keys {
			self.take_news(news);
		}
		// the account's Pro, which this session may be the first to hear
		self.sync.pro_at = Some(Instant::now());
		self.sync.syncing();
		self.sync.request = None;
		if let Joining::Other { from, .. } = joining {
			match from {
				Via::Restore => {
					self.sync.restore_busy = false;
					self.sync.restore_done += 1;
				}
				Via::Link => self.sync.linking = Some(LinkState::Linked),
				Via::Lan => {}
			}
			// the sign-in that asked for R is over: the Email page shows the account it joined
			let seq = self.sync.email.seq;
			self.sync.email = EmailFlow { seq, ..EmailFlow::default() };
		}
		if let Some(done) = self.sync.lan_join.take() {
			let _ = done.send(Some(self.account_state()));
		}
		self.sync_publish_all();
	}

	/// A session the server did not give: a deleted account (404) of the account this computer holds means it is gone:
	/// forgotten with its R, and with Sync on a new one made at once (`sync_drive` after the answer: no token and no R
	/// start `POST /accounts`), the Sync card saying `menu.sync.deleted` until its first sync ends; a wrong key (403) is
	/// its R replaced by a new one made while this computer was out (contract/sync-api.yaml, "The account's new key"),
	/// once the R' of its own rekey, when one waits, did not open it either (`join_next`: that rekey may have gone in,
	/// its 204 lost): Sync off, the account and R kept, the Sync card saying `menu.sync.rekey`, since only a new code
	/// from another device brings it back, and so does the Email page for an email sign-in's join of this account, its
	/// next request code bound to the account so a phone answers with the new R; another account's R says so on the
	/// page that brought it. An account that holds every join until another device makes its new R (409 `rekey`) says
	/// so on the Sync card, asked again after `RETRY`, or on the page that brought another account's R (an email
	/// sign-in's join on the Email page too).
	fn session_refused(&mut self, joining: &Joining, status: Option<u16>, called: &Called) {
		let rekey = Refusal::of(called).as_deref() == Some("rekey");
		if rekey {
			log::info!("sync: a join held until the account's new key is in");
		}
		let held = self.settings.sync.account.clone();
		match (joining, status) {
			(Joining::Again, Some(403)) if self.settings.sync.next_root().is_some() => {
				// the next drive joins with it
				self.join_next();
			}
			(Joining::Other { account: Some(account), from: Via::Restore, .. }, Some(403))
				if held.as_ref() == Some(account) && self.sync.email.found.as_ref() == Some(account) =>
			{
				if let Some(r) = self.join_next() {
					self.sync.busy = false;
					self.sync_join(Joining::Other { account: Some(account.clone()), r, from: Via::Restore });
				} else {
					log::info!("sync: the email sign-in's account was given a new key while this computer was out");
					self.sync.restore_busy = false;
					self.sync.email.error = Some(Problem::Rekey);
					self.link_drop();
					self.sync.email.step = Step::NeedKey;
				}
			}
			(Joining::Again, Some(404)) => {
				log::info!("sync: the account is gone: a new one");
				self.settings.sync.gone();
				self.save_settings();
				(self.sync.told, self.sync.line) = (Some(Line::Deleted), Some(Line::Deleted));
			}
			(Joining::Again, Some(403)) => {
				log::info!("sync: the account's key was replaced while this computer was out: Sync off");
				self.settings.sync.removed();
				self.save_settings();
				self.sync_reset(Some(Line::Rekey));
			}
			(Joining::Other { from, .. }, _) => {
				let problem = match status {
					_ if rekey => Problem::Rekey,
					Some(404 | 403) => Problem::NotFound,
					_ => Problem::Failed,
				};
				log::info!("sync: a join not taken ({status:?})");
				match from {
					Via::Restore => {
						self.sync.restore_busy = false;
						self.sync.restore_error = Some(problem);
						if rekey && self.sync.email.found.is_some() {
							self.sync.email.error = Some(Problem::Rekey);
						}
					}
					Via::Link => self.sync.linking = Some(if rekey { LinkState::Rekey } else { LinkState::Failed }),
					Via::Lan => {}
				}
				self.sync.line = None;
			}
			(Joining::Again | Joining::New(_), _) if rekey => {
				(self.sync.line, self.sync.wait_until) = (Some(Line::Rekey), Some(Instant::now() + RETRY));
			}
			_ => self.sync_failed(called),
		}
		if let Some(done) = self.sync.lan_join.take() {
			let _ = done.send(None);
		}
		self.sync_publish_all();
	}

	/// A join of this computer's own account that its R no longer opens (403): the R' of its rekey, when one waits,
	/// becomes R and is joined with next (contract/sync-api.yaml, "Making the new R": a device keeps R' until the
	/// account settles it), since the account may hold it, its 204 lost on the way. The R it replaces goes: the account
	/// opens it no more, and the chain gives it back once R' is in.
	fn join_next(&mut self) -> Option<[u8; sync::ROOT]> {
		let next = self.settings.sync.next_root()?;
		log::info!("sync: the account's key is not this computer's R: its new key tried");
		self.settings.sync.new_root(&next, None);
		self.save_settings();
		Some(next)
	}

	/// Removed from the account (a 401): Sync off, the account and R kept.
	fn sync_removed(&mut self) {
		log::info!("sync: this computer's token was not taken: removed from sync");
		self.settings.sync.removed();
		self.save_settings();
		self.sync_reset(Some(Line::Removed));
	}

	/// The token's state goes: every answer of it stale, the queue, the socket and the keys' state gone.
	fn sync_reset(&mut self, line: Option<Line>) {
		self.sync.generation += 1;
		self.sync.stop_live();
		(self.sync.busy, self.sync.out, self.sync.line, self.sync.told) = (false, VecDeque::new(), line, None);
		self.sync.devices.clear();
		self.sync.keys_forget();
		self.sync_publish_all();
	}

	/// The batches that go next: the device batch when its body changed, then the rows made since the last batch.
	fn sync_build(&mut self) {
		let Some((account, device, _, keys)) = self.sync_in() else { return };
		let hid = self.settings.hid_text();
		let a = &self.settings.sync;
		let mut seq = a.seq;
		let made_at = lan::now_ms();
		let batch = |kind: Kind, body: serde_json::Value, seq: u64| {
			let batch =
				Batch { v: sync::VERSION, account: account.clone(), device: device.clone(), seq, made_at, kind, body };
			sync::seal_batch(&keys, &batch).ok().map(|sealed| STANDARD.encode(sealed))
		};
		let body = DeviceBody {
			name: self.address.name.clone(),
			kind: sync::DESKTOP.to_owned(),
			profile: serde_json::to_value(self.engine_profile()).ok(),
			stamps: Some(self.body.body.clone()).filter(|b| *b != Body::default()),
			sheets: sync::carried(&self.session.sheet(now().unix).characters),
			hid: Some(hid.clone()),
		};
		let value = serde_json::to_value(&body).unwrap_or_default();
		let sum = digest(value.to_string().as_bytes());
		if a.device_digest.as_deref() != Some(&sum)
			&& let Some(sealed) = batch(Kind::Device, value, seq)
		{
			self.sync.out.push_back(Out { seq, kind: Kind::Device, sealed, marks: None, digest: Some(sum) });
			seq += 1;
		}
		// the rows in batches under `MAX_SEALED`: a part halved by its texts until it fits (one text's copies stay
		// together), in their order
		let (groups, game, app) = self.rows_due();
		let guids: BTreeMap<u8, String> = (1..=u8::MAX)
			.zip(&self.session.people().chars)
			.filter_map(|(id, c)| Some((id, c.as_ref()?.guid.clone())))
			.filter(|(_, guid)| !guid.is_empty())
			.collect();
		let mut parts = if groups.is_empty() { Vec::new() } else { vec![groups] };
		let (mut sealed, mut whole) = (Vec::new(), true);
		while let Some(mut part) = parts.pop() {
			let at = seq + u64::try_from(sealed.len()).unwrap_or(u64::MAX);
			let body = serde_json::to_value(rows_body(&part, &hid, &guids)).unwrap_or_default();
			let Some(bytes) = batch(Kind::Rows, body, at) else {
				whole = false;
				continue;
			};
			if bytes.len() / 4 * 3 > MAX_SEALED && part.len() > 1 {
				let back = part.split_off(part.len() / 2);
				parts.extend([back, part]);
				continue;
			}
			sealed.push(bytes);
		}
		let count = sealed.len();
		for (i, sealed) in sealed.into_iter().enumerate() {
			// the stores' marks move with the last batch, once every part sealed
			let marks = (whole && i + 1 == count).then(|| (game.clone(), app.clone()));
			let seq = seq + u64::try_from(i).unwrap_or(u64::MAX);
			self.sync.out.push_back(Out { seq, kind: Kind::Rows, sealed, marks, digest: None });
		}
		if count == 0 && whole && (self.settings.sync.game != game || self.settings.sync.app != app) {
			// nothing new that goes (body notes alone, rows taken in): what was looked at counts as sent
			(self.settings.sync.game, self.settings.sync.app) = (game, app);
			self.save_settings();
		}
	}

	/// The rows made since the last batch, by text with all their copies, and the stores' marks once they went: the
	/// game's rows (characters as the helper numbers them, `GET /v1/history`'s) and the rows of this computer's own app
	/// pumps (the ones `keep_computer` gave a pump id), each without the body notes, and the game's without a reset
	/// that came in from another computer (`FarRows::came_in`), which its maker sent.
	fn rows_due(&self) -> (Vec<Group>, Sent, Sent) {
		let a = &self.settings.sync;
		let (store, app, far) = (self.session.store(), self.session.app_rows(), self.session.far_rows());
		let (game_id, app_id) = (store.id(), app.id());
		let goes = |r: &RowRecord| {
			let row = r.row();
			!(is_body_note(&row) || (row.kind == kind::RESET && far.came_in(row.ts)))
		};
		let game_rows: Vec<RowRecord> = self.session.history_rows(0).filter(|r| goes(r)).collect::<Vec<_>>();
		// the rows that never go out, the marks still count every record
		let game_from = a.game.from(&game_id);
		let skipped = store.records().get(..game_from).unwrap_or_default().iter().filter(|r| !goes(r)).count();
		let game_groups = groups_of(&game_rows, game_from.saturating_sub(skipped).min(game_rows.len()), RowRecord::row);
		let own = |r: &&RowRecord| ran_here(r) && !is_body_note(&r.row());
		let own_rows: Vec<Row> = app.records().iter().filter(own).map(|r| unstamp(&r.row())).collect();
		let app_from = a.app.from(&app_id);
		let before = app.records().get(..app_from).unwrap_or_default().iter().filter(own).count();
		let app_groups = groups_of(&own_rows, before.min(own_rows.len()), |r: &Row| *r);
		let mut groups: Vec<Group> =
			game_groups.into_iter().map(|g| Group::Game(g.iter().map(GameRow::of).collect())).collect();
		groups.extend(app_groups.into_iter().map(Group::App));
		let game = Sent { store: game_id, upto: store.records().len() };
		let app = Sent { store: app_id, upto: app.records().len() };
		(groups, game, app)
	}

	/// The front batch of the queue to the server (built first when none waits).
	fn sync_push(&mut self) {
		self.sync.push_at = None;
		if self.sync.full {
			return;
		}
		if self.sync.out.is_empty() {
			self.sync_build();
		}
		let Some(front) = self.sync.out.front() else { return };
		let Some((_, _, token, keys)) = self.sync_in() else { return };
		let body =
			serde_json::json!({ "seq": front.seq, "kid": keys.kid, "kind": front.kind.name(), "sealed": front.sealed });
		self.sync.busy = true;
		self.sync.syncing();
		let generation = self.sync.generation;
		let bytes = serde_json::to_vec(&body).unwrap_or_default();
		self.sync.call(reqwest::Method::POST, "/batches".to_owned(), Some(token), Some(bytes), move |c| {
			News::Pushed(generation, c)
		});
	}

	fn on_pushed(&mut self, called: &Called) {
		self.sync.busy = false;
		match called {
			Ok(a) if a.status == 200 => {
				if let Some(out) = self.sync.out.pop_front() {
					let s = &mut self.settings.sync;
					s.seq = s.seq.max(out.seq + 1);
					if let Some((game, app)) = out.marks {
						(s.game, s.app) = (game, app);
					}
					if out.digest.is_some() {
						s.device_digest = out.digest;
					}
					log::info!("sync: batch {} ({}) kept", out.seq, out.kind.name());
					self.save_settings();
				}
				if self.sync.out.is_empty() {
					self.sync.pull_due = true; // after each push
				}
			}
			Ok(a) if a.status == 409 => {
				let refusal = serde_json::from_slice::<Refusal>(&a.body).ok();
				if refusal.as_ref().and_then(|r| r.reason.as_deref()) == Some("kid") {
					// sealed under an R the account no longer names: its new R first, then the same seq under it
					let seq = self.sync.out.front().map_or(self.settings.sync.seq, |o| o.seq);
					log::info!("sync: batch {seq} under another key than the account's: its new key first");
					self.sync.out.clear();
					(self.sync.keys_due, self.sync.kid_refused) = (true, true);
					return;
				}
				let last = refusal.and_then(|r| r.last_seq).unwrap_or(self.settings.sync.seq);
				log::info!("sync: seq {} taken already: from {} on", self.settings.sync.seq, last + 1);
				self.settings.sync.seq = last + 1;
				self.save_settings();
				self.sync.out.clear();
				self.sync.push_at = Some(Instant::now());
			}
			Ok(a) if a.status == 401 => self.sync_removed(),
			Ok(a) if a.status == 413 => {
				log::info!("sync: the account is full; nothing more goes in this run");
				(self.sync.full, self.sync.line) = (true, Some(Line::Full));
				self.sync.out.clear();
			}
			_ => self.sync_failed(called),
		}
	}

	/// The batches after the cursor, one page.
	fn sync_pull(&mut self) {
		let Some((_, _, token, _)) = self.sync_in() else { return };
		self.sync.pull_due = false;
		self.sync.busy = true;
		self.sync.syncing();
		let generation = self.sync.generation;
		let path = format!("/batches?after={}&limit={PAGE}", self.settings.sync.cursor);
		self.sync.call(reqwest::Method::GET, path, Some(token), None, move |c| News::Page(generation, c));
	}

	fn on_page(&mut self, called: &Called) {
		self.sync.busy = false;
		let page = match called {
			Ok(a) if a.status == 200 => serde_json::from_slice::<Page>(&a.body).ok(),
			Ok(a) if a.status == 401 => {
				self.sync_removed();
				return;
			}
			_ => None,
		};
		let Some(mut page) = page else {
			self.sync_failed(called);
			return;
		};
		if let Some(news) = page.keys.take() {
			self.take_news(news);
		}
		let Some((account, own, _, _)) = self.sync_in() else { return };
		if self.keys_behind() {
			// the account names a new R this computer takes from its wrap first: the page comes again under it
			self.sync.pull_due = true;
			return;
		}
		let unknown = page.batches.iter().any(|b| b.device != own && self.root_of(&b.kid).is_none());
		if unknown && self.sync.heard.is_some() && self.sync.chain_read != self.own_kid() {
			// batches under older kids: the chain gives their Rs, then the page comes again
			log::info!("sync: batches under an older key: the account's chain read first");
			(self.sync.chain_wanted, self.sync.keys_due, self.sync.pull_due) = (true, true, true);
			return;
		}
		let mut saved = true;
		let mut newer = None;
		for listed in &page.batches {
			if listed.device == own {
				continue; // this computer's own: the union makes it a no-op
			}
			match self.take_listed(&account, listed) {
				Taken::Newer(v) => newer = newer.or(Some((listed.cursor, v))),
				Taken::Unsaved => saved = false,
				Taken::In | Taken::Skipped => {}
			}
		}
		if !saved {
			// the cursor stays: the page comes again
			log::info!("sync: a page not kept whole, taken in again in a minute");
			(self.sync.line, self.sync.wait_until) = (Some(Line::Failed), Some(Instant::now() + RETRY));
			return;
		}
		let a = &mut self.settings.sync;
		if let Some(seen) = newer.filter(|_| a.newer.is_none()) {
			a.newer = Some(seen);
		}
		let last = page.batches.last().map_or(a.cursor, |b| b.cursor);
		a.cursor = if page.more { last.max(a.cursor) } else { last.max(page.cursor).max(a.cursor) };
		self.save_settings();
		if page.more {
			self.sync.pull_due = true;
		} else {
			self.sync.pulled_at = Some(Instant::now());
			// what changed without a row (a sheet, the profile) goes at the latest after the next pull
			self.sync.push_at.get_or_insert_with(Instant::now);
			self.settings.sync.synced_at = Some(lan::now_ms());
			self.save_settings();
			self.sync.told = None;
			self.sync.line = Some(if self.settings.sync.newer.is_some() { Line::Newer } else { Line::Last });
		}
	}

	/// One listed batch taken in, opened with the R its kid names (`root_of`).
	fn take_listed(&mut self, account: &str, listed: &Listed) -> Taken {
		let keys = self.root_of(&listed.kid).map(|r| SyncKeys::derive(&r));
		let opened = keys
			.zip(STANDARD.decode(&listed.sealed).ok())
			.map(|(keys, sealed)| sync::open_batch(&keys, &sealed, account, &listed.device, listed.seq));
		let batch = match opened {
			Some(Ok(Opened::Read(batch))) if batch.kind.name() == listed.kind => batch,
			Some(Ok(Opened::Newer(v))) => return Taken::Newer(v),
			other => {
				let why = match other {
					Some(Err(e)) => e.to_string(),
					None => "another key or not base64".to_owned(), // copy-lint: not shown (a log line)
					Some(Ok(_)) => "its kind differs from the listing".to_owned(), // copy-lint: not shown (a log line)
				};
				log::info!("sync: batch {} of {} skipped: {why}", listed.seq, listed.device);
				return Taken::Skipped;
			}
		};
		match batch.kind {
			Kind::Rows => self.take_rows_batch(&listed.device, batch.body),
			Kind::Device => {
				self.take_device_batch(&listed.device, batch.body);
				Taken::In
			}
		}
	}

	/// A rows batch: its app rows as the LAN's are taken (a phone's with its phone, another computer's as the
	/// computer's), and another computer's game rows into the far store, each character by the batch's `chars`, its
	/// reset taken as this computer's own (`Session::take_far`).
	fn take_rows_batch(&mut self, device: &str, body: serde_json::Value) -> Taken {
		let Ok(body) = serde_json::from_value::<RowsBody>(body) else {
			log::info!("sync: a rows batch of {device} did not read");
			return Taken::Skipped;
		};
		let t = now();
		let own = self.settings.hid_text();
		let mut saved = true;
		let rows = backup::app_rows(&body.rows);
		if !rows.is_empty() {
			let kind = self.sync.devices.iter().find(|d| d.device == device).map(|d| d.kind.clone());
			let from = body.hid.as_ref().map_or_else(|| kind.as_deref().and_then(sender), |_| None);
			let (taken, fx) = self.session.restore_app_rows(from, &rows, &t);
			log::info!("sync: {} app rows of {device}, {} new, {} dropped", rows.len(), taken.kept, taken.dropped);
			saved &= !taken.unwritten;
			self.apply(fx);
		}
		let guids = body.guids();
		if let Some(hid) = body.hid.filter(|h| *h != own)
			&& !body.game.rows.is_empty()
		{
			let (kept, written, fx) = self.session.take_far(&hid, &body.game.rows, &guids, &t);
			log::info!("sync: {} game rows of another computer, {kept} new", body.game.rows.len());
			saved &= written;
			self.apply(fx);
		}
		if saved { Taken::In } else { Taken::Unsaved }
	}

	/// A device batch: its name for the Devices list, its body fields into the shared body (the later change wins, as
	/// `/v1/body` merges a phone's), and a computer's characters into the far store's numbering.
	fn take_device_batch(&mut self, device: &str, body: serde_json::Value) {
		let Ok(body) = serde_json::from_value::<DeviceBody>(body) else {
			log::info!("sync: a device batch of {device} did not read");
			return;
		};
		let name: String = body.name.chars().filter(|c| !c.is_control()).take(NAME_MOST).collect();
		if self.settings.sync.names.get(device) != Some(&name) {
			self.settings.sync.names.insert(device.to_owned(), name);
			self.save_settings();
		}
		if let Some(mut stamps) = body.stamps {
			stamps.validate(lan::now_ms());
			self.take_stamps(&stamps);
		}
		if let Some(hid) = body.hid.filter(|h| *h != self.settings.hid_text()) {
			let fx = self.session.far_characters(&hid, &body.sheets);
			self.apply(fx);
		}
	}

	/// Another device's body stamps: its later changes taken (the later change wins, as `/v1/body` merges a phone's).
	fn take_stamps(&mut self, stamps: &Body) {
		if self.body.take(stamps) {
			log::info!("sync: body: another device's later changes taken");
			self.save_body();
			self.profile_changed();
			self.publish_setup();
			self.publish_welcome();
		}
	}

	fn sync_devices(&mut self) {
		let Some((_, _, token, _)) = self.sync_in() else { return };
		self.sync.busy = true;
		self.sync.devices_at = Some(Instant::now());
		let generation = self.sync.generation;
		self.sync.call(reqwest::Method::GET, "/devices".to_owned(), Some(token), None, move |c| {
			News::Devices(generation, c)
		});
	}

	fn on_devices(&mut self, called: &Called) {
		self.sync.busy = false;
		match called {
			Ok(a) if a.status == 200 => {
				if let Ok(list) = serde_json::from_slice::<Devices>(&a.body) {
					self.sync.devices = list.devices;
					self.email_held(list.email);
				}
			}
			Ok(a) if a.status == 401 => self.sync_removed(),
			_ => {}
		}
	}

	/// The kid of the R this computer holds.
	fn own_kid(&self) -> Option<String> {
		self.settings.sync.keys().map(|k| k.kid)
	}

	/// Whether the account names another kid than the R this computer holds: a new R waits in its wrap.
	fn keys_behind(&self) -> bool {
		let account = self.sync.heard.as_ref().and_then(|n| n.kid.as_ref());
		account.is_some_and(|kid| Some(kid) != self.own_kid().as_ref())
	}

	/// The R a kid names: this computer's own, or an earlier one the chain gave.
	fn root_of(&self, kid: &str) -> Option<[u8; sync::ROOT]> {
		let r = self.settings.sync.root()?;
		if SyncKeys::derive(&r).kid == kid { Some(r) } else { self.sync.older.get(kid).copied() }
	}

	/// This computer's own key, made at its first need and kept as R is (`Account::device_secret`).
	fn device_key(&mut self) -> KeyPair {
		if let Some(key) = self.settings.sync.device_key() {
			return key;
		}
		let (secret, key) = fresh_key();
		log::info!("sync: this computer's own key made");
		(self.settings.sync.device_secret, self.settings.sync.key_kid) = (Some(sync::text(&secret)), None);
		self.save_settings();
		key
	}

	/// This computer's rekey as posted, while its R' is still the one that waits (none once R' went or changed).
	fn posted(&self) -> Option<&Posted> {
		let next = self.settings.sync.next_r.as_deref()?;
		self.sync.posted.as_ref().filter(|p| p.next == next)
	}

	/// The keys' news heard (contract/sync-api.yaml, "Making the new R"): a new R this computer posted becomes its R
	/// once the news names its kid, and is dropped only once the news names another kid at a generation past the one
	/// its body was made from, since no copy of that body can go in any more; with no body in memory (a start that kept
	/// R'), news no longer due is past it, since only a new generation clears `due`; never while a rekey is out
	/// (`rekey_out`). So a lost answer never loses the account, nor does a read that comes before a copy still on its
	/// way.
	fn take_news(&mut self, news: KeyNews) {
		if let Some(next) = self.settings.sync.next_root() {
			let past = self.posted().map_or(!news.due, |p| news.generation > p.generation);
			if news.kid.as_deref() == Some(SyncKeys::derive(&next).kid.as_str()) {
				log::info!("sync: the account names this computer's new key: it is in");
				self.rekey_in(next);
			} else if past && !self.sync.rekey_out {
				log::info!("sync: the account moved past this computer's new key: dropped");
				self.settings.sync.next_r = None;
				self.save_settings();
				(self.sync.posted, self.sync.resend) = (None, false);
			}
		}
		self.sync.heard = Some(news);
	}

	/// The account's new key's call due now, sent (contract/sync-api.yaml, "The account's new key"): a rekey whose
	/// answer told nothing goes again as it was (`resend`, once a 429's wait is over); then the keys read (asked for, a
	/// wrap waiting, the account under another kid); then this computer's key registered (another kid, or the news shows
	/// it missing or other); then a new R while one is due, or the R' that waits posted again with fresh wraps. None but
	/// the resend and the keys read after a push's 409 `kid` (`kid_refused`) before this session heard the keys' news:
	/// a server before the rule sends none.
	fn sync_keys_step(&mut self, now: Instant) -> bool {
		if self.sync.resend && self.sync.rekey_wait.is_none_or(|at| at <= now) {
			if self.posted().is_some() {
				self.sync_rekey_post();
				return true;
			}
			self.sync.resend = false;
		}
		let Some(news) = self.sync.heard.clone() else {
			if self.sync.kid_refused {
				self.sync_keys_read();
				return true;
			}
			return false;
		};
		if self.sync.keys_due || self.keys_behind() {
			self.sync_keys_read();
			return true;
		}
		let (Some(device), Some(keys)) = (self.settings.sync.device.clone(), self.settings.sync.keys()) else {
			return false;
		};
		let Some(id) = sync::bytes::<{ sync::DEVICE_ID }>(&device) else { return false };
		let listed = news.devices.iter().find(|d| d.device == device);
		// a wrap that waits while this computer holds the account's R ends with its registration
		let stale = self.settings.sync.key_kid.as_deref() != Some(keys.kid.as_str()) || news.wrapped;
		if stale || listed.is_some() {
			let own = self.device_key().public_uncompressed();
			let put = KeyPut { key: sync::text(&own), mac: sync::text(&keys.mac(&id, &own)), kid: keys.kid.clone() };
			let differs = |d: &DeviceKey| {
				d.key.as_ref() != Some(&put.key) || d.mac.as_ref() != Some(&put.mac) || d.kid.as_ref() != Some(&put.kid)
			};
			if stale || listed.is_some_and(differs) {
				self.sync_register(put);
				return true;
			}
		}
		let rekey = news.due
			&& self.sync.refused_generation != Some(news.generation)
			&& self.sync.rekey_wait.is_none_or(|at| at <= now);
		// the news lists every device while due, this one too: never a rekey from a list that lacks it, which would
		// remove every device it left out
		if !rekey || listed.is_none() {
			return false;
		}
		let unknown = news.devices.iter().filter_map(|d| d.kid.as_deref()).any(|kid| self.root_of(kid).is_none());
		if unknown && self.sync.chain_read.as_deref() != Some(keys.kid.as_str()) {
			// a MAC made under an older R is checked under it: the chain gives it first
			self.sync.chain_wanted = true;
			self.sync_keys_read();
			return true;
		}
		self.sync_rekey(&news, &id);
		true
	}

	/// `GET account/keys`, from the chain's next page while a walk goes on.
	fn sync_keys_read(&mut self) {
		let Some((_, _, token, _)) = self.sync_in() else { return };
		let path = self
			.sync
			.walk
			.as_ref()
			.map_or_else(|| "/account/keys".to_owned(), |walk| format!("/account/keys?from={}", walk.from));
		self.sync.busy = true;
		let generation = self.sync.generation;
		self.sync.call(reqwest::Method::GET, path, Some(token), None, move |c| News::Keys(generation, c));
	}

	/// The account's keys: the news taken (`take_news`); then a walk of the chain goes on, or a wrap for this computer
	/// while the account names another kid is opened with its own key and its kid checked, and the chain walked from it
	/// back to the R this computer holds (`walk_chain`); the account under another kid with no wrap that opens is this
	/// computer left out (`wrap_refused`); and an older kid wanted walks the chain from this computer's R.
	fn on_keys(&mut self, called: &Called) {
		self.sync.busy = false;
		let keys = match called {
			Ok(a) if a.status == 200 => serde_json::from_slice::<Keys>(&a.body).ok(),
			Ok(a) if a.status == 401 => {
				self.sync_removed();
				return;
			}
			_ => None,
		};
		let Some(Keys { kid, generation, due, wrap, devices, chain, more }) = keys else {
			// `keys_due` stays: read again after the wait
			self.sync_failed(called);
			return;
		};
		(self.sync.keys_due, self.sync.kid_refused) = (false, false);
		let walk = self.sync.walk.take();
		self.take_news(KeyNews { kid: kid.clone(), generation, due, wrapped: wrap.is_some(), devices });
		if let Some(walk) = walk {
			if walk.head == kid {
				self.walk_chain(walk, &chain, more);
			} else {
				// the account moved on mid-walk: this page starts at an older kid, so the walk starts again at its head
				self.sync.keys_due = true;
			}
		} else if self.keys_behind() {
			let opened = (wrap.as_deref().and_then(|w| STANDARD.decode(w).ok()))
				.zip(self.settings.sync.device_key())
				.and_then(|(wrap, key)| sync::open_wrap(&key, &wrap).ok())
				.filter(|r| Some(SyncKeys::derive(r).kid) == kid);
			match (opened, kid) {
				(Some(r), Some(head)) => {
					let walk = Walk { head: Some(head.clone()), from: head, at: r, opened: Vec::new(), wrap: Some(r) };
					self.walk_chain(walk, &chain, more);
				}
				_ => self.wrap_refused(),
			}
		} else if self.sync.chain_wanted
			&& let (Some(r), Some(from)) = (self.settings.sync.root(), self.own_kid())
		{
			self.sync.chain_wanted = false;
			self.walk_chain(Walk { head: kid, from, at: r, opened: Vec::new(), wrap: None }, &chain, more);
		}
	}

	/// A page of the chain walked back (contract/sync-api.yaml, "Taking the wrap"): each link's `prev` opened under the R
	/// before it, its R kept only when it is the link's `prev_kid`'s (`sync::open_prev`), until a wrap's walk reaches the
	/// kid this computer holds; the next page asked for while `more` says older links remain.
	fn walk_chain(&mut self, mut walk: Walk, chain: &[KeyLink], more: bool) {
		let own = self.own_kid();
		for link in chain {
			let prev = STANDARD.decode(&link.prev).ok();
			let Some(r) = prev.and_then(|prev| sync::open_prev(&walk.at, &link.kid, &link.prev_kid, &prev)) else {
				log::info!("sync: a link of the account's chain that does not open or check");
				self.walk_ended(walk, false);
				return;
			};
			walk.opened.push((link.prev_kid.clone(), r));
			walk.at = r;
			if walk.wrap.is_some() && own.as_ref() == Some(&link.prev_kid) {
				self.walk_ended(walk, true);
				return;
			}
		}
		match chain.last() {
			Some(last) if more => {
				walk.from.clone_from(&last.prev_kid);
				self.sync.walk = Some(walk);
				self.sync.keys_due = true;
			}
			_ => self.walk_ended(walk, false),
		}
	}

	/// A walk of the chain at its end: a wrap's that reached this computer's R makes the wrap's R this computer's (its
	/// key registered again under it, what was not kept sealed again), one that did not is this computer left out
	/// (`wrap_refused`); a walk for the earlier Rs keeps what it opened.
	fn walk_ended(&mut self, walk: Walk, reached: bool) {
		match walk.wrap {
			Some(r) if reached => {
				log::info!("sync: the account's new key taken from its wrap");
				self.sync.older.extend(walk.opened);
				self.settings.sync.new_root(&r, None);
				self.save_settings();
				self.sync_new_root();
			}
			Some(_) => self.wrap_refused(),
			None => {
				self.sync.older.extend(walk.opened);
				self.sync.chain_read = self.own_kid();
			}
		}
	}

	/// A wrap that does not open or check, or none while the account names another kid: not taken (a server that lies
	/// can withhold a wrap, never hand this computer an R of its own). This computer signs out (`DELETE /session`) and
	/// Sync turns off (`menu.sync.removed`).
	fn wrap_refused(&mut self) {
		log::info!("sync: the account's new key did not open or check here: signed out");
		if let Some(token) = self.settings.sync.token.clone() {
			self.sync.call(reqwest::Method::DELETE, "/session".to_owned(), Some(token), None, News::Left);
		}
		self.sync_removed();
	}

	/// `PUT devices/self/key`: this computer's key, its MAC under R and R's kid.
	fn sync_register(&mut self, put: KeyPut) {
		let Some((_, _, token, _)) = self.sync_in() else { return };
		self.sync.busy = true;
		let generation = self.sync.generation;
		let bytes = serde_json::to_vec(&put).unwrap_or_default();
		self.sync.call(reqwest::Method::PUT, "/devices/self/key".to_owned(), Some(token), Some(bytes), move |c| {
			News::Registered(generation, put, c)
		});
	}

	/// This computer's key registered: its kid kept, and the news as the server holds it now (no wrap waits, the key
	/// listed), read again while a new R is due, whose generation the registration moved. A 409 `kid` is an R the account
	/// no longer names: its wrap first.
	fn on_registered(&mut self, put: KeyPut, called: &Called) {
		self.sync.busy = false;
		match called {
			Ok(a) if a.status == 204 => {
				log::info!("sync: this computer's key registered");
				self.settings.sync.key_kid = Some(put.kid.clone());
				self.save_settings();
				let own = self.settings.sync.device.clone();
				if let Some(news) = &mut self.sync.heard {
					news.wrapped = false;
					if let Some(d) = news.devices.iter_mut().find(|d| Some(&d.device) == own.as_ref()) {
						(d.key, d.mac, d.kid) = (Some(put.key), Some(put.mac), Some(put.kid));
					}
					self.sync.keys_due |= news.due;
				}
			}
			Ok(a) if a.status == 401 => self.sync_removed(),
			Ok(_) if Refusal::of(called).as_deref() == Some("kid") => {
				log::info!("sync: this computer's key under another kid than the account's: its new key first");
				self.sync.keys_due = true;
			}
			_ => self.sync_failed(called),
		}
	}

	/// The account's new R (contract/sync-api.yaml, "Making the new R"): R' from the OS random source, kept beside R
	/// before it goes, or the R' that waits for its rekey again (a start kept it but not its body, or news still names
	/// R's kid at the generation it was made from); a wrap for each other device of the news whose MAC holds under the R
	/// its kid names, each with a fresh key, none for one without a key or a MAC that holds, which the server then
	/// removes; this computer's MAC under R'; R sealed under R' (`prev`); posted with the news' generation and kept as
	/// posted (`Posted`).
	fn sync_rekey(&mut self, news: &KeyNews, own: &[u8; sync::DEVICE_ID]) {
		let (Some((_, device, _, keys)), Some(r)) = (self.sync_in(), self.settings.sync.root()) else { return };
		let next = if let Some(next) = self.settings.sync.next_root() {
			log::info!("sync: this computer's new key posted again with fresh wraps");
			next
		} else {
			let mut next = [0u8; sync::ROOT];
			rand::fill(&mut next);
			self.settings.sync.next_r = Some(sync::text(&next));
			self.save_settings();
			next
		};
		let next_keys = SyncKeys::derive(&next);
		let mac_holds = |d: &DeviceKey| -> Option<[u8; sync::PUBLIC_KEY]> {
			let (id, key) = (sync::bytes(&d.device)?, sync::bytes(d.key.as_deref()?)?);
			let mac: [u8; sync::MAC] = sync::bytes(d.mac.as_deref()?)?;
			let under = SyncKeys::derive(&self.root_of(d.kid.as_deref()?)?);
			under.mac_holds(&id, &key, &mac).then_some(key)
		};
		let mut wraps = Vec::new();
		for d in news.devices.iter().filter(|d| d.device != device) {
			match mac_holds(d).and_then(|key| sync::wrap_root(&fresh_key().1, &key, &next)) {
				Some(wrap) => wraps.push(serde_json::json!({ "device": d.device, "wrap": STANDARD.encode(wrap) })),
				None => log::info!("sync: device {} gets no wrap: no key, or its MAC does not hold", d.device),
			}
		}
		log::info!("sync: a new key for the account, wrapped to {} devices", wraps.len());
		let own_key = self.device_key().public_uncompressed();
		let body = serde_json::json!({
			"generation": news.generation,
			"auth": next_keys.auth,
			"kid": next_keys.kid,
			"mac": sync::text(&next_keys.mac(own, &own_key)),
			"prev_kid": keys.kid,
			"prev": STANDARD.encode(next_keys.seal.seal(&r)),
			"wraps": wraps,
		});
		let body = serde_json::to_vec(&body).unwrap_or_default();
		self.sync.posted = Some(Posted { next: sync::text(&next), body, generation: news.generation });
		self.sync_rekey_post();
	}

	/// `POST account/rekey` with the body as posted (`Posted`), the first time or the same bytes again.
	fn sync_rekey_post(&mut self) {
		let Some((_, _, token, _)) = self.sync_in() else { return };
		let Some(Posted { body, generation: made_from, .. }) = self.posted() else { return };
		let (body, made_from) = (body.clone(), *made_from);
		self.sync.resend = false;
		(self.sync.busy, self.sync.rekey_out) = (true, true);
		let generation = self.sync.generation;
		self.sync.call(reqwest::Method::POST, "/account/rekey".to_owned(), Some(token), Some(body), move |c| {
			News::Rekeyed(generation, made_from, c)
		});
	}

	/// The new R's answer (contract/sync-api.yaml, "Making the new R"): 204 makes it R. A 409, or another refusal but a
	/// 429, tells: the keys are read first (`take_news`), where R''s kid makes it R all the same (a copy that went in,
	/// its 204 lost) and another kid at a generation past the one it was made from drops it; none is posted again from
	/// that generation (`generation`: another device's rekey won, or the devices moved; `key`: this computer registers
	/// first; `kid`: its wrap first). No answer, a server's failure or a 429 tells nothing: the same body goes again
	/// after the wait or its `Retry-After` (the rest of the queue going on through a 429), which the server answers 204
	/// once a copy is in, even while the first is still on its way.
	fn on_rekeyed(&mut self, made_from: u64, called: &Called) {
		self.sync.busy = false;
		let Some(next) = self.settings.sync.next_root() else { return };
		match called {
			Ok(a) if a.status == 204 => {
				log::info!("sync: the account's new key is in");
				self.rekey_in(next);
				let kid = self.own_kid();
				if let Some(news) = &mut self.sync.heard {
					(news.kid, news.due, news.wrapped) = (kid, false, false);
					news.devices.clear();
				}
			}
			Ok(a) if a.status == 401 => self.sync_removed(),
			Ok(a) if a.status == 429 => {
				log::info!("sync: the account's new key waits its Retry-After: the same body then");
				self.sync.rekey_wait = Some(Instant::now() + a.retry.map_or(RETRY, Duration::from_secs));
				self.sync.resend = true;
			}
			Ok(a) if (400..500).contains(&a.status) => {
				let reason = Refusal::of(called);
				let why = reason.as_deref().unwrap_or("-");
				log::info!("sync: the account's new key answered {} ({why}): the keys read first", a.status);
				if reason.as_deref() == Some("key") {
					self.settings.sync.key_kid = None;
					self.save_settings();
				}
				self.sync.refused_generation = Some(made_from);
				self.sync.keys_due = true;
			}
			_ => {
				log::info!("sync: the account's new key not answered: the same body again after the wait");
				self.sync.resend = true;
				self.sync_failed(called);
			}
		}
	}

	/// This computer's new R in: R replaced, the R it replaced kept for its batches, and its key registered under the
	/// new R, as the rekey set its MAC.
	fn rekey_in(&mut self, next: [u8; sync::ROOT]) {
		if let Some(r) = self.settings.sync.root() {
			self.sync.older.insert(SyncKeys::derive(&r).kid, r);
		}
		let kid = SyncKeys::derive(&next).kid;
		self.settings.sync.new_root(&next, Some(kid));
		self.save_settings();
		self.sync.refused_generation = None;
		self.sync_new_root();
	}

	/// R replaced (a wrap taken, this computer's new R in, either way with no R' waiting since): what was sealed under
	/// the old R and not yet kept goes again under the new one from the same seq, a pull takes in what the new R opens,
	/// and the pages that show R's codes (Restore code, Link a device) show the new R's.
	fn sync_new_root(&mut self) {
		(self.sync.out, self.sync.push_at, self.sync.pull_due) = (VecDeque::new(), Some(Instant::now()), true);
		(self.sync.chain_read, self.sync.posted, self.sync.resend) = (None, None, false);
		self.sync_publish_all();
	}

	/// The account's Pro (`GET pro`).
	fn sync_pro(&mut self) {
		let Some((_, _, token, _)) = self.sync_in() else { return };
		self.sync.busy = true;
		self.sync.pro_at = None;
		let generation = self.sync.generation;
		self.sync.call(reqwest::Method::GET, "/pro".to_owned(), Some(token), None, move |c| News::Pro(generation, c));
	}

	/// The account's Pro as the server answered: its token to `runtime/pro.rs`, asked again after `PRO_EVERY`; a
	/// failure leaves the Sync card's line alone and asks again after `Retry-After` or `PRO_RETRY`. A live message
	/// whose `pro` changed while it was out asks again at once (`pro_at` stands then).
	fn on_pro(&mut self, called: &Called) {
		self.sync.busy = false;
		let answer = match called {
			Ok(a) if a.status == 200 => serde_json::from_slice::<ProAnswer>(&a.body).ok(),
			Ok(a) if a.status == 401 => {
				self.sync_removed();
				return;
			}
			_ => None,
		};
		let now = Instant::now();
		if let Some(ProAnswer { token, status }) = answer {
			self.sync.pro_at.get_or_insert(now + PRO_EVERY);
			self.take_account_pro(token.as_deref(), status.as_deref());
		} else {
			let retry = called.as_ref().ok().and_then(|a| a.retry).map_or(PRO_RETRY, Duration::from_secs);
			match called {
				Ok(a) => log::info!("sync: the account's Pro answered {}", a.status),
				Err(e) => log::info!("sync: the account's Pro not answered: {e}"),
			}
			self.sync.pro_at.get_or_insert(now + retry);
		}
	}

	/// Manage Pro's call with this computer's token (none outside an account: the page says the server did not
	/// answer).
	pub(super) fn pro_portal(&mut self) {
		let Some((_, _, token, _)) = self.sync_in() else {
			self.pro_link_done(ProLink::Portal, None);
			return;
		};
		self.pro.page.out = true;
		self.sync.call(reqwest::Method::POST, "/pro/portal".to_owned(), Some(token), None, News::Portal);
	}

	/// Manage Pro's answer: its address, when it is an HTTPS one (Paddle's portal), opened in the browser.
	fn on_portal(&mut self, called: &Called) {
		self.pro.page.out = false;
		if let Ok(a) = called
			&& a.status == 401
		{
			self.pro_link_done(ProLink::Portal, None);
			self.sync_removed();
			return;
		}
		let url = called
			.as_ref()
			.ok()
			.filter(|a| a.status == 200)
			.and_then(|a| serde_json::from_slice::<ProUrl>(&a.body).ok())
			.map(|u| u.url)
			.filter(|url| url.starts_with("https://"));
		match (called, &url) {
			(_, Some(_)) => {}
			(Ok(a), None) => log::info!("sync: the portal answered {} with no address it opens", a.status),
			(Err(e), None) => log::info!("sync: the portal not answered: {e}"),
		}
		self.pro_link_done(ProLink::Portal, url.as_deref());
	}

	/// Whether an email holds the account, as each devices' read says (an email linked, changed or removed on another
	/// device reaches the Email row here): the address this computer typed goes with the email.
	fn email_held(&mut self, email: bool) {
		let a = &mut self.settings.sync;
		if a.email == email {
			return;
		}
		a.email = email;
		if !email {
			a.address = None;
		}
		self.save_settings();
		self.email_publish();
	}

	/// Signed out (`said` Out) or the account deleted (`said` Deleted), each once the server answered 204: Sync off and
	/// every row kept; signed out, the account and R stay (`Account::removed`: Sync on again rejoins it), deleted, they
	/// go. A 401 is this computer removed (`menu.sync.removed`); any other answer changes nothing and the page says the
	/// server did not answer.
	fn on_out(&mut self, called: &Called, said: Said) {
		self.sync.email.busy = false;
		match called {
			Ok(a) if a.status == 204 => {
				log::info!("sync: out of the account ({said:?})");
				if said == Said::Deleted {
					self.settings.sync.left();
				} else {
					self.settings.sync.removed();
				}
				self.save_settings();
				self.sync.email = EmailFlow { seq: self.sync.email.seq + 1, ..EmailFlow::default() };
				self.sync.email.said = Some((self.sync.email.seq, said));
				self.sync_reset(None);
			}
			Ok(a) if a.status == 401 => self.email_removed(),
			_ => self.out_failed(called),
		}
	}

	/// The email removed (`DELETE /email` answered 204): the account and its progress stay, the Email page shows Not
	/// linked. A 401 is this computer removed; any other answer changes nothing.
	fn on_unlinked(&mut self, called: &Called) {
		self.sync.email.busy = false;
		match called {
			Ok(a) if a.status == 204 => {
				log::info!("sync: the email removed");
				let a = &mut self.settings.sync;
				(a.email, a.address) = (false, None);
				self.save_settings();
				self.sync.email = EmailFlow { seq: self.sync.email.seq, ..EmailFlow::default() };
				self.sync_publish_all();
			}
			Ok(a) if a.status == 401 => self.email_removed(),
			_ => self.out_failed(called),
		}
	}

	/// An Email page's step answered 401: this computer was removed from the account; the page starts again.
	fn email_removed(&mut self) {
		self.sync.email = EmailFlow { seq: self.sync.email.seq, ..EmailFlow::default() };
		self.sync_removed();
	}

	/// A Sign out, Remove email or Delete account the server did not answer: nothing changes, and the page says so.
	fn out_failed(&mut self, called: &Called) {
		match called {
			Ok(a) => log::info!("sync: an Email page's step answered {}", a.status),
			Err(e) => log::info!("sync: an Email page's step not answered: {e}"),
		}
		self.sync.email.out_failed = true;
		self.email_publish();
	}

	/// The account as a paired phone hears it (`GET /v1/account`): none while Sync is off.
	pub(crate) fn account_state(&self) -> AccountState {
		let a = &self.settings.sync;
		let (account, r) = if a.on && a.token.is_some() { (a.account.clone(), a.r.clone()) } else { (None, None) };
		AccountState {
			devices: if account.is_some() {
				u32::try_from(self.sync.devices.len().max(1)).unwrap_or(u32::MAX)
			} else {
				0
			},
			email: account.is_some() && a.email,
			account,
			r,
		}
	}

	/// A phone's Add (`PUT /v1/account`): this computer joins that account (the join turns its Sync on) and answers its
	/// account after the join; one already in it answers at once.
	pub(crate) fn account_put(
		&mut self,
		account: String,
		r: [u8; sync::ROOT],
		done: oneshot::Sender<Option<AccountState>>,
	) {
		if let Err(why) = self.account_open("sync: a phone's Add") {
			log::info!("{why}");
			let _ = done.send(None);
			return;
		}
		let a = &self.settings.sync;
		if a.on && a.token.is_some() && a.account.as_deref() == Some(&account) && a.root() == Some(r) {
			let _ = done.send(Some(self.account_state()));
			return;
		}
		log::info!("sync: a phone's Add: this computer joins its account");
		self.sync.lan_join = Some(done);
		self.sync.busy = false;
		self.sync_join(Joining::Other { account: Some(account), r, from: Via::Lan });
	}

	/// The Sync card's, the Email page's, Restore code's and Link a device's topics.
	fn sync_publish_all(&self) {
		self.sync_publish();
		self.email_publish();
		self.restore_publish();
		self.link_publish();
	}

	fn sync_publish(&self) {
		let a = &self.settings.sync;
		let devices = self
			.sync
			.devices
			.iter()
			.map(|d| DeviceRow {
				device: &d.device,
				kind: &d.kind,
				name: a.names.get(&d.device).map(String::as_str),
				seen_at: d.seen_at,
				this: a.device.as_deref() == Some(&d.device),
			})
			.collect();
		let line = if self.secrets_unread() {
			Some(Line::Unread)
		} else {
			self.sync.line.filter(|l| a.on || matches!(l, Line::Deleted | Line::Removed | Line::Rekey))
		};
		let topic = SyncTopic { on: a.on, line, last: a.synced_at, devices };
		self.publish(SYNC, &topic);
	}

	fn email_publish(&self) {
		let (a, e) = (&self.settings.sync, &self.sync.email);
		// the page's steps act in the account this computer holds a token for, whatever Sync's switch says
		let account = a.held();
		let topic = EmailTopic {
			step: e.step,
			purpose: e.purpose,
			email: e.email.as_deref(),
			address: a.address.as_deref().filter(|_| account && a.email),
			linked: account && a.email,
			account,
			busy: e.busy,
			error: e.error,
			tries_left: e.tries_left,
			sent_at: e.sent_at,
			said: e.said.map(|(seq, what)| SaidTopic { seq, what }),
			request: self.sync.request.as_ref().filter(|r| r.held.is_none()).and_then(|r| r.qr.as_ref()),
			linking: self.sync.linking,
			out_failed: e.out_failed,
		};
		self.publish(EMAIL, &topic);
	}

	fn restore_publish(&self) {
		let topic = RestoreTopic {
			code: self.settings.sync.account.as_ref().and(self.settings.sync.root()).map(|r| sync::restore_code(&r)),
			busy: self.sync.restore_busy,
			error: self.sync.restore_error,
			done: self.sync.restore_done,
		};
		self.publish(RESTORE, &topic);
	}

	/// Link a device's code: the link code while this computer is in its account (a phone scans it and joins), else
	/// the request code while one stands (a phone in an account scans it and answers with R, sealed), or, once an
	/// answer no email binds came, its check code in the code's place.
	fn link_publish(&self) {
		let a = &self.settings.sync;
		let account = self.sync_in().and_then(|_| Some((sync::bytes(a.account.as_deref()?)?, a.root()?)));
		let held = self.sync.request.as_ref().and_then(|r| r.held.as_ref());
		let (code, kind) = match (account, &self.sync.request) {
			(Some((account, r)), _) => (pairing::qr(&LinkCode::Account { account, r }.to_string()), Some("account")),
			(None, Some(request)) if held.is_none() => (request.qr.clone(), Some("request")),
			(None, _) => (None, None),
		};
		let check = held.map(|h| h.check.as_str());
		self.publish(LINKDEVICE, &LinkTopic { code, kind, state: self.sync.linking, check });
	}

	/// The main window closed: no page shows a request code any more.
	pub(crate) fn sync_pages_gone(&mut self) {
		self.sync.link_wanted = false;
	}

	/// While a page shows the request code and this computer lacks R: a request made (again after it ended, a held
	/// answer with it), and its answer read every `LINK_POLL` until one came.
	fn link_drive(&mut self) {
		let wants = self.sync.link_wanted && (self.sync_in().is_none() || self.sync.email.step == Step::NeedKey);
		if !wants || self.sync.link_busy || self.sync.client.is_none() {
			return;
		}
		let now = Instant::now();
		if self.sync.request.as_ref().is_none_or(|r| r.until <= now) {
			self.link_request(now);
		}
		if self.sync.linked_at.is_some_and(|at| now < at + LINK_POLL) {
			return;
		}
		let Some(request) = self.sync.request.as_ref().filter(|r| r.held.is_none()) else { return };
		self.sync.link_busy = true;
		self.sync.linked_at = Some(now);
		let (id, path) = (request.id, format!("/link/{}", sync::text(&request.id)));
		self.sync.call(reqwest::Method::GET, path, None, None, move |c| News::Link(id, c));
	}

	/// A new link request: a fresh id and key pair, its code as a QR, shown until `LINK_LIFE` ends, bound to the account
	/// an email sign-in found, where one did.
	fn link_request(&mut self, now: Instant) {
		let mut id = [0u8; sync::REQUEST_ID];
		rand::fill(&mut id);
		let (_, key) = fresh_key();
		let qr = pairing::qr(&LinkCode::Request { id, key: key.public() }.to_string());
		let binds = self.sync.email.found.clone();
		self.sync.request = Some(Request { id, key, until: now + LINK_LIFE, qr, binds, held: None });
		self.sync.linking = None;
		self.email_publish();
		self.link_publish();
	}

	/// A link request's answer, opened with the request's key. A request made under an email sign-in (`Request::binds`)
	/// joins only the account that email found, at once: another's is refused and a new request shown. Any other waits
	/// with its check code for the player (`Request::held`), so whoever answers a code seen on this screen first cannot
	/// bring this computer into their own progress.
	fn on_link(&mut self, id: [u8; sync::REQUEST_ID], called: &Called) {
		self.sync.link_busy = false;
		let Some(request) = self.sync.request.as_mut().filter(|r| r.id == id && r.held.is_none()) else { return };
		match called {
			// a request of an email sign-in that has ended since (its flow began again): answered by no one's choice now
			Ok(a) if a.status == 200 && request.binds != self.sync.email.found => {
				log::info!("sync: a link answer to a request of an email sign-in that ended: a new request");
				self.link_request(Instant::now());
			}
			Ok(a) if a.status == 200 => {
				let opened = serde_json::from_slice::<Answered>(&a.body)
					.ok()
					.and_then(|w| STANDARD.decode(w.sealed).ok())
					.and_then(|sealed| sync::open_link_answer(&request.key, &sealed).ok());
				let Some(opened) = opened else {
					log::info!("sync: a link answer that did not open");
					self.sync.request = None;
					self.sync.linking = Some(LinkState::Failed);
					self.sync_publish_all();
					return;
				};
				let Some(binds) = request.binds.clone() else {
					log::info!("sync: a phone answered the link request: its check code waits for the player");
					// the held answer stands as long as a request would, from now
					(request.until, request.held) = (Instant::now() + LINK_LIFE, Some(opened));
					self.sync.linking = None;
					self.sync_publish_all();
					return;
				};
				self.sync.request = None;
				if sync::text(&opened.account) != binds {
					log::info!("sync: a link answer of another account than the email's, refused");
					self.link_request(Instant::now());
					self.sync.linking = Some(LinkState::Other);
					self.sync_publish_all();
					return;
				}
				log::info!("sync: a phone answered the link request");
				self.link_join(&opened);
			}
			// read already or ended: a new one at the next look
			Ok(a) if a.status == 404 => self.sync.request = None,
			_ => {}
		}
		self.sync_publish_all();
	}

	/// An opened link answer joined: the account and R it brings (`PUT /session`, `Via::Link`).
	fn link_join(&mut self, opened: &sync::LinkOpened) {
		self.sync.linking = Some(LinkState::Linking);
		self.sync.busy = false;
		self.sync_join(Joining::Other { account: Some(sync::text(&opened.account)), r: opened.r, from: Via::Link });
	}

	/// The request code and an answer that waits with it go: the Email page's flow began again or found its account,
	/// whose request must be one bound to it (`Request::binds`).
	fn link_drop(&mut self) {
		self.sync.request = None;
		self.sync.linking = None;
	}

	/// The `sync` area's intents; in a run whose secrets did not read, only a look, the switch turned off and a link
	/// page's end.
	pub(crate) fn sync_intent(&mut self, intent: &Intent) -> Result<(), String> {
		if !matches!(intent.kind(), "sync.look" | "sync.switch" | "sync.link.stop") {
			self.account_open(intent.kind())?;
		}
		match intent.kind() {
			"sync.look" => {
				self.sync.devices_at = None;
				self.sync_publish_all();
			}
			"sync.switch" => {
				let on = intent.arg("on").and_then(serde_json::Value::as_bool).ok_or("sync.switch: no on")?;
				self.sync_switch(on)?;
			}
			"sync.remove" => {
				let device =
					intent.arg("device").and_then(serde_json::Value::as_str).ok_or("sync.remove: no device")?;
				let Some((_, own, token, _)) = self.sync_in() else {
					return Err("sync.remove: not in an account".to_owned());
				};
				if device == own || sync::bytes::<16>(device).is_none() {
					return Err("sync.remove: not another device".to_owned());
				}
				let generation = self.sync.generation;
				self.sync.call(reqwest::Method::DELETE, format!("/devices/{device}"), Some(token), None, move |c| {
					News::Removed(generation, c)
				});
			}
			"sync.link.start" => {
				self.sync.link_wanted = true;
				if self.sync.linking == Some(LinkState::Linked) {
					self.sync.linking = None;
				}
				self.link_publish();
			}
			"sync.link.stop" => self.sync.link_wanted = false,
			// the player's answer on a held answer's check code: Yes, join takes it in, No makes a new request
			"sync.link.join" | "sync.link.refuse" => {
				let Some(opened) = self.sync.request.as_mut().and_then(|r| r.held.take()) else {
					return Err(format!("{}: no answer waits", intent.kind()));
				};
				if intent.kind() == "sync.link.join" {
					log::info!("sync: the player joined the link answer");
					self.sync.request = None;
					self.link_join(&opened);
				} else {
					log::info!("sync: the player refused the link answer: a new request");
					self.link_request(Instant::now());
				}
				self.sync_publish_all();
			}
			_ => return Err(intent.unknown()),
		}
		self.sync_drive();
		Ok(())
	}

	/// Sync's switch (the Sync card's, or Get Pro's ask with Sync off): on asks the session, pull and push at once, and
	/// is refused in a run whose secrets did not read; off sends nothing more.
	///
	/// # Errors
	/// On in a run whose secrets did not read (`account_open`).
	pub(super) fn sync_switch(&mut self, on: bool) -> Result<(), String> {
		if on {
			self.account_open("sync.switch")?;
		}
		log::info!("sync: turned {}", if on { "on" } else { "off" });
		self.settings.sync.on = on;
		self.save_settings();
		(self.sync.line, self.sync.told) = (None, None);
		(self.sync.pull_due, self.sync.wait_until, self.sync.push_at) = (true, None, Some(Instant::now()));
		if !on {
			self.sync_reset(None);
		}
		self.publish_pro();
		Ok(())
	}

	/// Err in a run whose secrets did not read at its start (`Helper::secrets_unread`): no account is joined, made or
	/// changed then, since its R and token would be gone at the next start and a second account made in its place.
	///
	/// # Errors
	/// In such a run, naming `what`.
	pub(super) fn account_open(&self, what: &str) -> Result<(), String> {
		if self.secrets_unread() {
			// copy-lint: not shown (a log line's and an intent's error)
			return Err(format!("{what}: refused while the secrets did not read at this start"));
		}
		Ok(())
	}

	/// The `restore` area's intents.
	pub(crate) fn restore_intent(&mut self, intent: &Intent) -> Result<(), String> {
		match intent.kind() {
			"restore.look" => {
				self.sync.restore_error = None;
				self.restore_publish();
			}
			"restore.enter" => {
				self.account_open("restore.enter")?;
				let code = intent.arg("code").and_then(serde_json::Value::as_str).ok_or("restore.enter: no code")?;
				if self.sync.restore_busy {
					return Err("restore.enter: one is under way".to_owned());
				}
				match sync::read_restore(code) {
					Ok(r) => {
						self.sync.restore_error = None;
						self.sync.restore_busy = true;
						let account = self.sync.email.found.clone();
						self.sync.busy = false;
						self.sync_join(Joining::Other { account, r, from: Via::Restore });
					}
					Err(e) => {
						log::info!("sync: a restore code that does not read: {e}");
						self.sync.restore_error = Some(Problem::Wrong);
					}
				}
				self.restore_publish();
			}
			_ => return Err(intent.unknown()),
		}
		Ok(())
	}

	/// The `email` area's intents, in the account this computer holds a token for, whatever Sync's switch says; in a
	/// run whose secrets did not read, only the flow's start and change, which ask the server nothing.
	pub(crate) fn email_intent(&mut self, intent: &Intent) -> Result<(), String> {
		if !matches!(intent.kind(), "email.start" | "email.change") {
			self.account_open(intent.kind())?;
		}
		let in_account = self.settings.sync.held();
		self.sync.email.out_failed = false;
		match intent.kind() {
			"email.start" => {
				let seq = self.sync.email.seq;
				self.sync.email = EmailFlow { seq, ..EmailFlow::default() };
				self.link_drop();
			}
			"email.send" => {
				let text = intent.arg("email").and_then(serde_json::Value::as_str).unwrap_or_default();
				let Some(email) = address(text) else {
					self.sync.email.error = Some(Problem::Invalid);
					self.email_publish();
					return Ok(());
				};
				let purpose = if !in_account {
					Purpose::Signin
				} else if self.sync.email.purpose == Purpose::Change {
					Purpose::Change
				} else {
					Purpose::Link
				};
				self.email_code(email, purpose);
			}
			"email.again" => {
				let email = self.sync.email.email.clone().ok_or("email.again: no address")?;
				self.email_code(email, self.sync.email.purpose);
			}
			"email.code" => {
				let code = intent.arg("code").and_then(serde_json::Value::as_str).unwrap_or_default().trim().to_owned();
				self.email_answer(&code);
			}
			"email.change" => {
				let linked = self.settings.sync.email;
				let seq = self.sync.email.seq;
				self.sync.email = EmailFlow {
					seq,
					purpose: if linked && in_account { Purpose::Change } else { Purpose::Link },
					..EmailFlow::default()
				};
				self.link_drop();
			}
			"email.join" => {
				let email = self.sync.email.email.clone().ok_or("email.join: no address")?;
				self.email_code(email, Purpose::Signin);
			}
			"email.signout" | "email.remove" | "email.delete" => {
				let Some(token) = self.held_token() else {
					return Err(format!("{}: not in an account", intent.kind()));
				};
				if self.sync.email.busy {
					return Err(format!("{}: a step is under way", intent.kind()));
				}
				self.sync.email.busy = true;
				let (path, news): (&str, fn(Called) -> News) = match intent.kind() {
					"email.signout" => ("/session", News::Out),
					"email.remove" => ("/email", News::Unlinked),
					_ => ("/account", News::Deleted),
				};
				self.sync.call(reqwest::Method::DELETE, path.to_owned(), Some(token), None, news);
			}
			_ => return Err(intent.unknown()),
		}
		self.email_publish();
		Ok(())
	}

	/// A code to `email` for `purpose` (`POST /email/code`): `link` and `change` with this computer's token.
	fn email_code(&mut self, email: String, purpose: Purpose) {
		let e = &mut self.sync.email;
		if e.busy {
			return;
		}
		(e.busy, e.error, e.tries_left, e.purpose) = (true, None, None, purpose);
		let body =
			serde_json::to_vec(&serde_json::json!({ "email": email, "purpose": purpose.word() })).unwrap_or_default();
		e.email = Some(email);
		let token = if purpose == Purpose::Signin { None } else { self.held_token() };
		self.sync
			.call(reqwest::Method::POST, "/email/code".to_owned(), token, Some(body), |c| News::Email(Mail::Code, c));
	}

	/// The code the player typed: `POST /email` to link or change, `POST /signin` to sign in.
	fn email_answer(&mut self, code: &str) {
		let e = &mut self.sync.email;
		let Some(email) = e.email.clone().filter(|_| !e.busy) else { return };
		if code.len() != 6 || !code.bytes().all(|b| b.is_ascii_digit()) {
			e.error = Some(Problem::Wrong);
			return;
		}
		(e.busy, e.error) = (true, None);
		let body = serde_json::to_vec(&serde_json::json!({ "email": email, "code": code })).unwrap_or_default();
		if e.purpose == Purpose::Signin {
			self.sync
				.call(reqwest::Method::POST, "/signin".to_owned(), None, Some(body), |c| News::Email(Mail::SignIn, c));
		} else {
			let token = self.held_token();
			self.sync
				.call(reqwest::Method::POST, "/email".to_owned(), token, Some(body), |c| News::Email(Mail::Link, c));
		}
	}

	fn on_email(&mut self, mail: Mail, called: &Called) {
		let e = &mut self.sync.email;
		e.busy = false;
		let status = called.as_ref().map(|a| a.status).ok();
		let tries = || {
			called.as_ref().ok().and_then(|a| serde_json::from_slice::<Tries>(&a.body).ok()).and_then(|t| t.tries_left)
		};
		let removed = mail != Mail::SignIn && e.purpose != Purpose::Signin && status == Some(401);
		match (mail, status) {
			// a link or change with this computer's token, which the server no longer knows
			_ if removed => {
				self.email_removed();
				return;
			}
			(Mail::Code, Some(204)) => {
				(e.step, e.sent_at) = (Step::Code, Some(lan::now_ms()));
			}
			(Mail::Code, Some(400)) => e.error = Some(Problem::Invalid),
			(Mail::Code, Some(429)) => e.error = Some(Problem::Limit),
			(_, Some(403)) => (e.error, e.tries_left) = (Some(Problem::Wrong), tries()),
			(_, Some(410)) => e.error = Some(Problem::Expired),
			(Mail::Link, Some(409)) => e.step = Step::Taken,
			(Mail::Link, Some(204)) => {
				log::info!("sync: an email linked");
				(e.step, e.purpose) = (Step::Field, Purpose::Link);
				e.seq += 1;
				e.said = Some((e.seq, Said::Linked));
				let s = &mut self.settings.sync;
				(s.email, s.address) = (true, e.email.take());
				self.save_settings();
			}
			(Mail::SignIn, Some(200)) => {
				let found = called.as_ref().ok().and_then(|a| serde_json::from_slice::<Found>(&a.body).ok());
				if let Some(found) = found {
					self.email_found(found.account);
				} else {
					self.sync.email.error = Some(Problem::Failed);
				}
			}
			_ => e.error = Some(Problem::Failed),
		}
		self.sync_publish_all();
	}

	/// An email sign-in found its account: this computer joins it with the R it holds, else asks for R (the request
	/// code, Enter restore code).
	fn email_found(&mut self, account: String) {
		self.sync.email.found = Some(account.clone());
		let a = &self.settings.sync;
		if a.account.as_deref() == Some(&account)
			&& let Some(r) = a.root()
		{
			self.sync.busy = false;
			self.sync_join(Joining::Other { account: Some(account), r, from: Via::Restore });
			return;
		}
		// the request shown from now is one bound to this account
		self.link_drop();
		self.sync.email.step = Step::NeedKey;
	}
}
