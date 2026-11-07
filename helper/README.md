# QuestPump for Mac and Windows (the helper; Rust, macOS and Windows)

The desktop app QuestPump, "QuestPump on this computer" in the game and in the app (its folder and the code keep the
name "helper"). A Cargo workspace (slices 5, 7, 8, 16, 24, 25 and 35 of the plan; rules: `.claude/skills/rust-helper`):

- `crates/questpump-core`: every rule, no platform code. The pixel-strip decoder, the profile and sheet chunks, the row
  grammar and the row store, the session that turns strip frames into rows, device commands and mirror writes (with the
  logout rule and the phone's catch-up rows), the `QuestPump_Mirror` writer and the readers of the mirror and of the
  game's save, the socket protocol, the pair link, the link's seal and command queue, the heart-rate verdict with each
  pump's proof and its calibration, the Bluetooth parsers (the strap's heart rate, the walking pad's treadmill data and
  control point), the feedback message's rules and its outbox, and since slice 35 the body every app device shares
  (`body.rs`), the games on the computer and the offer after a match (`games.rs`, `gsi.rs`, `rocket.rs`, `vdf.rs`,
  `offers.rs`), the offline check of a phone's Pro proof (`pro.rs`), the backup file (`backup.rs`), the crash reports
  (`crash.rs`) and the anonymous counts (`telemetry.rs`, with `telemetry_catalog.rs`, which `tools/telemetry.py` writes
  from `contract/telemetry.json`: every name and value a count may carry). Its security
  tests replay the link's contract in
  `contract/fixtures/` (`seal.json`, `pair.json`, `wire.json`) and the strip frames of `rows.json`, which the addon's
  own code writes (`make fixtures` at the repo root).
- `crates/questpump-engine` (slice 35): the addon's own engine in the desktop app. It embeds the addon's seven pure
  files (`tools/host.lua`'s `H.FILES`: `Core.lua`, `Constants.lua`, `DB.lua`, `Library.lua`, `Engine.lua`,
  `Progress.lua`, `Stats.lua`), `tools/host.lua` and `tools/apppump.lua` at build time and runs them on PUC Lua 5.1.5
  (`mlua`) on one thread, so the computer composes, judges and folds a pump with the game's own code. An edit to any of
  those files is an edit to the helper.
- `src-tauri` (`questpump-helper`): the desktop app, in the menu bar or the tray and an ordinary app while its window
  is open. The macOS capture (ScreenCaptureKit), the Windows capture
  (Windows.Graphics.Capture, then DXGI Desktop Duplication, then GDI), the install discovery (the running game, the
  Battle.net Agent's list of installs, the game's flavour folders, then a folder picker), the local socket, the mirror
  written into the game's AddOns folder, the window with the pair link's QR code, the LAN API the phone and the watch
  call (`src/lan.rs`, `contract/companion-api.yaml`), whose security tests serve it on the loopback, the heart-rate
  strap and the walking pad over Bluetooth (`src/bluetooth.rs`, btleplug, one task for both), start at login (a login
  item through SMAppService on macOS, the user's Run key on Windows), the updater (`tauri-plugin-updater`), the
  feedback button's post (`src/feedback.rs`), the crash reports and the app kept alive (`src/crash.rs`, `src/keep.rs`,
  `src/runtime/crash.rs`), Usage statistics and its anonymous counts (`src/runtime/usage.rs`, `src/runtime/counts.rs`),
  and since slice 35 the desktop app (`src/desk/`: the window, the tray, the
  macOS menu, the pump on the computer with its sound and voice, Pump home, Hero, the Book, the Log, since slice 56
  the Games tab, the Menu, the match-over card, the first five minutes) and the games on the computer (`src/games/`). Its window's pages are
  `helper/dist/`, plain ES modules and custom elements with no build step; their words, colours, fonts and art come
  from `contract/` through the generators (`copy.js`, `copy.rs`, `tokens.css`, `art.js`, `body.js`).

It never sends input into the game and never reads its memory ("What the helper does and does not do", below).

## What the helper does and does not do

The helper does these things:

- **It reads the colours of the addon's status strip**, a small row of coloured squares the QuestPump addon paints in
  the top-left corner of the game window. It looks at that corner only, and it never saves or sends a picture. On
  macOS this is the Screen Recording permission, which the helper asks for only when you press "Allow screen reading"
  in its window.
- **It keeps your progress in World of Warcraft's folder:** it writes `Interface/AddOns/QuestPump_Mirror/`, which the
  addon reads
  at login and at `/reload`. To find the folder and to take in what you played without it, it reads only QuestPump's
  own files in the game folder (the addon's and the exercise animations' TOCs and the addon's saves), the Battle.net
  Agent's list of installs, and the install's own version files (`.flavor.info`, `.build.info`) for the name its window
  gives the folder.
- **It pairs your phone:** the QR code on its Phones page pairs the QuestPump app on your phone and its watch, which
  then
  reach the helper over your home network. Your Body switches, your experience and the moves you leave out are the same
  on the computer and every paired phone (the latest change wins); a Body switch you change in the game joins them.
- **It runs pumps on the computer's screen** while World of Warcraft with QuestPump is closed: a Quick pump, Start a
  pump and the moves you owe, with the addon's own rules and its safety steps, and Hero, the Book and the Log of your
  progress from every device. While the game runs, the game asks your pumps.
- **It watches the games it finds on this computer** and lists them on its Games tab, each with its own off there: a
  game it hears no match from by its close, the rest at each match's end. Counter-Strike 2 and Dota 2 tell it about
  their matches through Valve's Game State Integration, a settings file it puts into the game's own `cfg` folder once
  you tap "Ask after each match", which posts to a listener on 127.0.0.1 only; Rocket League through Psyonix's Stats
  API, which its own `TAStatsAPI.ini` turns on (or yours) after the same tap, read over one connection on 127.0.0.1;
  League of Legends and Teamfight Tactics, and any program you add, by their process closing. When a match ends it
  offers a pump on a small card that never takes the focus, and on your phone or watch. It reads only that a match
  ended: never your name, a Steam ID, a score or a match id, and it never sends a game a key, a click or a command.
- **It reads a heart-rate strap** over Bluetooth, only after you turn it on in its window.
- **It reads a walking pad** over Bluetooth, only after you turn it on in its window: the distance a pad that speaks
  the Fitness Machine standard counts while you play, kept with the quests you turn in. A program on this computer
  may set the belt's speed or stop it through the helper's local socket, never the game and never over the network.
  It never changes a pump.
- **It sends the feedback you write**, only when you press Send in QuestPump's feedback window in the game or on its
  own Feedback page: your text, the version of QuestPump or of the helper and the time you pressed Send, and, unless you
  untick it, the counted numbers
  the game's window lists (such as the pumps you finished, banked and still owe over 14 days) or the system and
  screen-reading state its own window lists, all shown before you send. It goes to questpump.com, which mails it to Pluxia's support mailbox and
  keeps nothing; no name, character, realm or health data goes with it. A message that cannot go yet waits in the
  helper's data folder until it can, 30 days and 20 messages at most.
- **It asks what it may send.** Two switches in the Menu's QuestPump group, Send crash reports and Usage statistics,
  each on by default except in the EEA, India and Canada (as the computer's region or time zone says), where each stays
  off until you say yes. Until you have answered both, the first window you open shows them on one page (how each is
  used, at questpump.com/privacy, and Done, which keeps your answers; where they start off, Accept all, or Save choices
  once you moved a switch, over Reject all in Done's place). Your answer is kept on this computer with its time and the
  version of the words you saw; the default of your region is never stored, so it is read again wherever you are.
  Nothing else in QuestPump depends on either switch.
- **It sends a report when it crashes**, by itself at its next start (first thing, before anything else starts), while
  Send crash reports is on; where it is not answered yet in those regions, the window asks after a crash (Send, Always
  send or Don't send) and shows the report whole before anything goes. A report holds QuestPump's version and the
  system's, when and how it crashed and where in QuestPump's code (a Rust panic's message, file and line and backtrace,
  or what macOS's or Windows' own crash report says of QuestPump's process); never your name, your computer's name or
  identifiers, anything from your game, your pumps or health data, and your home folder's path reads `~`. It goes to
  questpump.com, which mails it to Pluxia's support mailbox and keeps nothing. One that cannot go yet waits in the
  helper's data folder, 30 days and 5 reports at most; turning the switch off deletes them. When its main loop, its
  network listener, its local socket or its writer of the game's folder crashes, QuestPump starts again once by itself,
  in the background; a page's own error and macOS's web content process ending only go into its log.
- **It sends anonymous counts** while Usage statistics is on: once a game day (at its first start that day, or when
  the day turns at 04:00), a batch for the day before with no id of any kind, no name, no time finer than the day and
  no free text, to questpump.com, which adds your country to an install's count and keeps the counts three months
  (Cloudflare's Analytics Engine): that QuestPump on this computer was installed and used that day (and how many days
  after your first, by week), its version and whether it crashed the run before, whether a phone, a strap or a walking
  pad paired, whether the link with your phone and the screen reading worked (its state, the system and the capture's
  kind), which of its pages and parts you used, the match offers shown, started, dismissed or held back by the day's
  cap, a return after three days or more away, the delay to your first pump; and of the pumps (this computer's own, and
  the game's from the rows the addon already keeps): how many you finished and banked that day, the unfinished moves
  you skipped and still owed at its end, which moves you took as the easier version or stopped early, and once a week
  your 14-day done ratio and the max sets on one ladder on a usual play day, in bands, and your setup (the pace, the
  focus, the floor, room to move, quiet, how much equipment) in bands. Never a count you typed, a record, a heart rate
  or other health data, a pain flag, a pause, a pump offered but not done, a quest, a character, a realm or anything
  that names you. A batch that cannot go yet waits in the helper's data folder at most 7 days; turning the switch off
  deletes it and what the day has counted so far. The game itself sends nothing.
- **It syncs your progress** while Sync your progress is on (on by default; the Menu's Backup and sync): the game's
  rows, the pumps it ran itself, the game's sheets, your setup and this computer's name go to your other devices
  through questpump.com, sealed with a key only your devices hold, so QuestPump's makers can't read them; heart rate,
  pain notes and unwell stops stay here. The server keeps the sealed batches, a hash of each device's token, each
  device's kind and dates and, once you add an email, a keyed hash of the address, never the address. You make no
  account: your devices share one without a step, and an email is your choice. Turning Sync off sends and takes in
  nothing and keeps everything here; Delete account on the Menu's Email page empties the account on the server.

The helper never:

- sends input to the game, not even a key press;
- reads the game's memory. It opens no handle to the game beyond what reading its window and its program path takes
  (on Windows the query-only right `PROCESS_QUERY_LIMITED_INFORMATION`, never a right to read memory);
- injects anything into the game or hooks it, or writes into the game folder anywhere but `QuestPump_Mirror` (in
  another game's folder only the one settings file that game needs once you tap "Ask after each match", which turning
  the game off removes);
- talks to Blizzard's servers;
- connects to the internet, except for six things over HTTPS: its update check (GitHub, where the releases live), the
  moves' films, fetched from there once for each new set and each checked against its SHA-256 (the stills and the
  fault pairs ship inside the app), the feedback you send yourself, its crash reports, its anonymous counts and the
  sync (above); it has no analytics service, advertising id or tracker of anyone else's. Crash reports and the counts
  go by themselves to questpump.com unless you turn them off, and in the EEA, India and Canada only once you say yes.

Everything the game needs is free, and its source is open and unobfuscated (MIT). Since slice 35 Pro's extras on the
computer (Pro's weeks in the Log, every chart and the share-card styles) unlock with a paired phone's Pro subscription:
the phone hands over the store's signed record, which the helper checks offline (Apple's chain to the pinned Apple Root
CA - G3, Play's signature under the app's licensing key) and keeps in `pro.json`; nothing is sold on the computer, no
WoW part is paid, and a new pairing code clears the record. A build from source can skip the check, and a Play record
copied from a phone passes it for 37 days after the last receipt; both are accepted, since the extras are small and
Pro's value lives on the phone. A refund or a lapse reaches the computer at the phone's next sync.

## The anti-cheat question

Blizzard's End User License Agreement forbids cheats, bots, hacks and unauthorised software that changes or
facilitates gameplay. Warden, Blizzard's anti-cheat scanner, looks for known cheats and for hooks into the game; the
helper hooks nothing. Every kind of program Blizzard has acted against automated play or read the game's memory.
Screen recorders and pixel readers have never been acted against: OBS and Discord (OBS even hooks into the game's
graphics) and the Discord Rich Presence pixel readers since 2019; nor have WeakAuras Companion and TradeSkillMaster's
desktop app, which write into the AddOns folder the way the helper does. For what the helper does, the risk is low.

The risk that is left, stated plainly: Blizzard's Anti-Cheating Agreement (2016-02-03) counts as unauthorised any
program that "collects information from or through Blizzard games", in Blizzard's "sole determination". Reading the
strip collects information in that literal sense, and Blizzard has never answered the question for a pixel reader (a
2019 forum question about one got no answer). Nobody can promise more than that.

The other games follow the same posture: seen by their process or through their publishers' own documented local
feeds, read only, nothing injected, no memory read and no input or command sent; League of Legends by its game process
alone, never a Riot API (`docs/companion.md`, "The other games").

If Blizzard ever objects to the pixel reader, the helper stops, and the addon still plays the full game: choose "Play
without them" on its setup screen. Every pump, the sheet, the records and XP stay. What stops with the helper: the
mirror (on the Forever beta, the saved progress, with the `!QP1!` export and import as the manual path), the countdown
on the wrist, the heart-rate check marks and catch-up pumps from the phone.

## Install

The download page is <https://questpump.com/helper>: a disk image for Macs with Apple silicon, one for Intel Macs
(macOS 14 or later) and an installer for 64-bit Windows 10 (22H2) or 11. Or with a package manager:

```bash
winget install --exact --id Pluxia.QuestPump          # Windows, once it is in winget-pkgs
brew install --cask pluxia-gmbh/tap/questpump        # macOS
```

On a Mac, drag QuestPump into Applications and open it from there.

On Windows, SmartScreen shows "Windows protected your PC" when you open the installer. Click "More info", check that
the publisher reads Pluxia GmbH, then click "Run anyway". The installer is signed; SmartScreen warns about every new
publisher until its files have built a reputation, which takes "several weeks and hundreds of clean installs" whatever
the certificate, and only apps from the Microsoft Store skip it.

## On Windows

The installer runs for your account only and asks for no administrator: the helper goes to
`%LOCALAPPDATA%\QuestPump\questpump-helper.exe`, its bundled files (the voice, the sounds, the move figures) under
`resources\` beside it, in the same folder as its data (no data file shares a name with a resource). A few things may
need one administrator step each, and its window offers each:

- **"Allow on my network".** Windows Defender Firewall asks the first time a program listens for connections, and when
  a standard user answers, Windows blocks the program whatever they click. So the helper does not listen until a rule
  lets your phone in, and until then its window shows this step instead of the pairing code. The button runs one
  `netsh` command as an administrator after the Windows prompt (UAC): an inbound rule for the helper's own program
  that lets in only devices on your local network (`remoteip=LocalSubnet`), on every kind of network (Windows files a
  new home network as public). It replaces every earlier inbound rule for the helper, such as the block rules a
  firewall prompt left. A rule you made yourself, or a firewall that is off, works too: the helper reads the rules
  again every minute. Uninstalling removes the rule the helper made, with one more prompt. When "Block all incoming
  connections" is on in Windows Security, no rule lets anything in: the window says where to turn it off instead.
- **"Allow writing to the game folder".** When Windows refuses the helper's write into the game's AddOns folder, the
  button gives your Windows account the right to change that folder (one `icacls` command as an administrator, after
  the prompt). A game whose settings folder refuses the helper's file after "Ask after each match" (Rocket League's
  under Program Files, a Steam library Windows keeps closed) gets the same one-time step in its window on the Games
  tab.

If Windows Security's Controlled folder access blocks the helper (a game installed under Documents, or a folder
someone added to the protected ones), only you can allow it: Windows Security → Virus & threat protection → Ransomware
protection → Allow an app through Controlled folder access → add
`%LOCALAPPDATA%\QuestPump\questpump-helper.exe`, then quit QuestPump and open it again. The window says so when
it happens.

The capture uses Windows.Graphics.Capture on Windows 11 (with no yellow border), DXGI Desktop Duplication where that
cannot hide its border (Windows 10) or fails, read from whichever graphics chip drives the display, and GDI last. On an
HDR display it captures in HDR and converts the colours back; if the strip still does not read while you play, the
window says "Your display uses HDR: if this stays while you play, turn HDR off for World of Warcraft".

The data folder is `%LOCALAPPDATA%\QuestPump` (`rows.jsonl`, `app.jsonl`, `sheets.jsonl`, `people.cbor`, `proof.json`,
`settings.json`, `body.json`, `pro.json`, `counts.json`, the feedback, crash and counts outboxes, `crashes\` with a
crash's report until the next start, the films' `figures\<digest>\` once downloaded,
`game-files` with every file it wrote into a game's folder, which the uninstaller removes, `firewall-rule` once the
helper made its firewall rule, and `secrets.dpapi`, the pairing key and the sync's keys sealed for your Windows account
with DPAPI, with `secrets.dpapi.before` once a start replaced a set), the log
`%LOCALAPPDATA%\ai.pluxia.questpump.bridge\logs\helper.log`. The local socket is the named pipe
`\\.\pipe\questpump-helper`, with the same lines as `helper.sock` on the Mac. "Start with your computer" on This
computer adds QuestPump to your account's startup programs (the `Run` value "QuestPump" with `--login`, so a start at
login opens no window; the "QuestPump Helper" values of builds before slice 35 go at every start); uninstalling
removes it and the games' files and keeps the data folder, so a reinstall keeps your progress.

## Updates and support

The helper checks for an update 10 seconds after it starts and then every hour by the clock (a Mac that slept past
the hour checks within a minute of waking), sooner again while `latest.json` does not answer (30 seconds, then 1, 2,
5, 10 and 15 minutes, then every 30 minutes until it does) and when the game closes, over HTTPS against `latest.json`
on the `helper-updater` release of Pluxia-GmbH/questpump; the add-on's feed follows the same schedule. It downloads a
new version (no limit on the whole download, only a minute without data ends it), checks its minisign signature
against the public key built into the helper (the signature covers the version number too) and keeps it in the data
folder's `updates` folder, so a quit or a restart does not download it again. A download that fails waits the next
hourly check, and one whose signature fails, or that cannot be written, is not downloaded again until the helper
restarts or the feed names another release. It installs it only while World of Warcraft and
the main window are closed, no pump runs and no game you turned on is in a match, so nobody loses a session; while
only the main window keeps it waiting, This computer's version line offers "Restart to update", which installs it at
once and opens the new version on This computer. An install that fails is tried again 10 minutes later, then every
hour. Otherwise the new version starts with `--updated` and opens no window.
On Windows the installer shows a progress bar and starts the new helper; on macOS the helper replaces its app and
starts again. The helper never asks for an administrator to update itself: where your Mac account cannot replace the
app (another account installed it), it downloads nothing and the window says to download the new version from
questpump.com/helper.

For a bug report, "Copy a report for help" in the Menu's QuestPump group puts QuestPump's version, your system, the
engine's digest and its last fold, what the pages show (never the pairing code or a path that names your account) and
the end of its log on the clipboard; nothing is sent until you paste it somewhere.

Pluxia GmbH supports the helper with security updates for at least five years after its first public release, and at
least until 2031-11-04, always for the newest version (the helper updates itself). How to report a vulnerability, and
what the helper exposes: [SECURITY.md](SECURITY.md).

## Check

```bash
cd helper
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo deny check
sh tools/windows-check.sh    # on the Mac: the same clippy for Windows x64 and arm64 (types and lints; nothing runs or
links)
```

No CI builds or checks the helper since 2026-10-09 (`helper.yml` went; David: "Can remove that CI that builds"):
these commands, run before every pull request that touches `helper/` or a file the helper builds in, are its only
check, with `python3 tools/notices.py --check` after a `Cargo.lock` change. Windows is checked only by
`tools/windows-check.sh` here and by `helper-release.yml` at a release.

The tests are the security tests, the one exception to no automated tests (2026-10-03). In
`crates/questpump-core/tests/`: the pairing key's derived bearer, seal key and id, the AES-GCM seal, the wire bodies and
the held link's command queue (`link.rs`), the pair link (`pair.rs`), and the Bluetooth link's key, envelope and frames
with the holder over Bluetooth (`ble.rs`, `contract/fixtures/ble.json`, slice 43). In `src-tauri/`: the sealed LAN
routes over the loopback (`tests/lan.rs`) and the inline checks of the pair link's host (`src/pairing.rs`), the LAN's
admission (`src/lan.rs`) and the elevated steps' command lines (`src/elevate.rs`). After a change to `Bridge.lua`, the
row writer or the lifecycle, run `make fixtures` at the repo root, then `cargo test`. The engine crate has no test: a
probe outside the repo replays `contract/fixtures/engine.json`'s flows through it once, and `make check` holds the seven
files and `tools/host.lua` to the TOC (`tools/lint.lua`'s `host` section).

To test the phone and watch apps in an emulator or a simulator with no game running, `cargo run --example sim_helper`
(from `helper/`, `src-tauri/examples/sim_helper.rs`) starts a stand-in helper: the real LAN routes on 127.0.0.1:8787
with a new pairing key, a pair link for the Android emulator (host 10.0.2.2) and one for the iOS simulator (host
127.0.0.1, which the apps' pair link refuses until a debug build allows loopback), and a control port on 8788 where a
script queues the game's commands, sets the felt-unwell lock and the unfinished list, and reads the state with `curl`.
It prints one JSON line per pairing, sync, event and command; the file's header lists the flags and the routes.

## Build and run (on the Mac)

The Screen Recording grant belongs to the signed bundle, never to a terminal, so the helper is tested as a bundle
launched through LaunchServices, and it is the one copy in `/Applications`: build it with the identity the installed
copy carries (the grant belongs to it), swap the bundle while the running helper goes on, and ask that helper to
restart: it waits until no pump runs and no match's offer stands, then starts the new one as the updater relaunches it,
with no window over the game; then delete the old copy:

```bash
cargo install tauri-cli --version "^2" --locked     # once
cd helper/src-tauri
APPLE_SIGNING_IDENTITY="Developer ID Application: Pluxia GmbH (PQFUQJH377)" cargo tauri build --bundles app
mv /Applications/QuestPump.app "$TMPDIR/QuestPump.old.app"   # the running helper goes on from here
mv ../target/release/bundle/macos/QuestPump.app /Applications/
printf '{"cmd":"restart"}\n' | nc -w 5 -U "$HOME/Library/Application Support/QuestPump/helper.sock"   # once no pump runs
# with no helper running, start it instead: open -g -j -a /Applications/QuestPump.app --args --updated
# once a new pid runs (pgrep -f QuestPump.app/Contents/MacOS/questpump-helper): rm -rf "$TMPDIR/QuestPump.old.app"
```

A player's launch (the Finder, the Dock, Launchpad, `open`, a second launch) opens the main window; a start at login
or after an update opens none, and the tray's Open QuestPump does. On a computer that has not had its first
five minutes the welcome covers the window (the experience, where you pump, a first Quick pump, Hero; the games too
until slice 56), and "Add my phone" goes to the Phones page with the pairing code. Once a phone or a watch is paired or the strap or the
walking pad is on, This computer also says why the helper asks to see the game's corner, and its "Allow screen reading"
button shows the
Screen Recording prompt (macOS then offers Quit & Reopen); the helper never prompts on its own, and before that a
missing grant only keeps the tray grey, since the saves reach the mirror without it. "Use a heart-rate strap" starts
Bluetooth (the macOS prompt comes then, never before): the helper finds a strap that broadcasts the standard heart-rate
service (a WHOOP with Broadcast Heart Rate on, a Polar H10), reads it during pumps, and says after 30 s without one how
to turn the WHOOP's broadcast on; the same button turns it off. "Use a walking pad" reads a pad that speaks the Fitness
Machine standard (FTMS: a Mobvoi, a newer KingSmith) on the same Bluetooth task; close the pad's own app first, since a
pad talks to one app at a time. The window then shows its speed and the day's walk kept with your quests, in your own
road units, and each stretch row (the one the game writes as a pump opens) gets the metres walked since the one before
(`pad_metres`).

The data folder is `~/Library/Application Support/QuestPump/` (`rows.jsonl`, `app.jsonl` with the phones' app rows,
which every paired phone reads back, `sheets.jsonl`, `people.cbor`, `proof.json`, `settings.json`, `body.json`,
`pro.json`, `counts.json`, the feedback, crash and counts outboxes, `crashes/` with a crash's report until the next
start, the films' `figures/<digest>/` once downloaded, `helper.sock`), the log
`~/Library/Logs/ai.pluxia.questpump.bridge/helper.log`. The pairing key and the sync's keys are no file: they are the
login keychain's item `ai.pluxia.questpump.bridge` (account `secrets`, and `secrets.before` once a start replaced a
set), which only the signed release reads. Before
its first write into a game folder the helper reads the `QuestPump_Mirror/State.lua` there and the game's saves of the
addon (`WTF/Account/*/SavedVariables/QuestPump.lua`), and the saves again when the game quits, so a lost data folder or
a session played without the helper costs no rows; a `State.lua` it cannot read is kept as `State.lua.unread`. `nc -U
~/Library/Application\ Support/QuestPump/helper.sock` shows the lines: the strip, the rows, each `command` queued for
the phone or the watch, each `wrist` event a device sent, each move's heart-rate `verdict` (with the source and the rise
it needed), the strap's `hr` readings, the walking pad's `pad` records and `pad_event`s, and their `status`, and a
device's `link` statistics (delivery p50 and p95, battery per hour) when its session ends.
`{"cmd":"display_lag","ms":25}` on the socket keeps the correction the latency harness's camera test measured: how much
later the wrist's "go" came than the screen's, negative when it came earlier. `{"cmd":"pad.speed","kmh":3.5}` and
`{"cmd":"pad.stop"}` set the walking pad's belt, each answered by the pad's own `pad_event`s or a `refused` one.

The LAN API listens on every address of the computer, IPv4 and IPv6, port 8787, from the start (bound once, so an
address that moves needs no new bind), for the phone that scanned the pair link and its watch. Every body is sealed for
its route, so a body opens only on the route and the way it was sent; a request whose `Host` does not name the computer
(an address, `localhost` or its own `.local` name) gets 421, before any route. `GET /v1/hello`, the one route without
the pairing key's token, tells a phone looking for its helper after the address moved which helper answers: over the
phone's random challenge it proves the install's id and the pairing key and names nothing of the computer (no id, name
or port). On Windows the helper is also listed over Bonjour as `QuestPump._questpump._tcp` with the TXT `txtvers=2`
alone (through Windows' own DNS Client), never on a Mac. Each refused connection or request, each pairing request and
each hello goes into the log, at most once a minute per address; with the macOS firewall on, macOS asks once whether
QuestPump may accept incoming connections, and the phone and the watch reach it only after Allow. The window's "New
pairing code" (pressed twice) ends every pairing, on connections already open too: scan the new code with the phone.
When the helper cannot listen (another app holds the port), the window says so under the code. On battery power the
capture runs at 10 frames a second, 60 on power.

The icons are the one vector mark of the app icon, the quest scroll barbell (`contract/design/icon/`): `python3 -B
tools/icons.py` writes the drawings and `AppIcon.icon`, and `python3 tools/helper-icons.py` rasterises the PNGs,
`icon.icns`, `icon.ico` and the tray's state icons (the mark's silhouette in the state's colour, 44 px for the menu bar
and 32 px for the Windows tray) with `rsvg-convert` (`brew install librsvg`). The `.icon`, the stills and pairs, the voice and the sounds are bundled
from their sources (`bundle.icon`, `bundle.resources`); the films only in test builds until the feed opens.

## Releasing

QuestPump releases from the public repository Pluxia-GmbH/questpump, never from the private Pluxia-GmbH/questfit: the
updater and the add-on's feed cannot read a private repository's release files. The public repository holds one commit
a release, which `tools/public/export.py` (in the private repository) makes from a merged `main` commit, with the paths
`tools/public/tree.txt` lists. Two workflows run there, and only there (in any other repository every job is skipped):
`.github/workflows/helper-release.yml` at a `helper-vX.Y.Z` tag and `.github/workflows/addon-release.yml` at a `vX.Y.Z`
tag. Both tags may sit on the same commit.

1. **Versions** (in the private repository, through a pull request): the helper's in
   `helper/src-tauri/tauri.conf.json` and `helper/src-tauri/Cargo.toml` (let `cargo` update `Cargo.lock`); the add-on's
   is its tag, and `CHANGELOG.md` gets its section, `## [X.Y.Z]` (with ` - YYYY-MM-DD` once the day is known), which
   the packager sends as the release's notes; `make changelog` then writes the site's `/changelog` and its feed
   from it. Run the gates above.
2. **Export:** `python3 -B tools/public/export.py` builds the public tree from `origin/main`, checks it (every listed
   path, every file the helper's build or a public workflow reads, `publish.py --check`, betterleaks) and prints its
   diff against the public `main`. `python3 -B tools/public/export.py --push --version X.Y.Z` then pushes it as one
   commit "QuestPump X.Y.Z" by Pluxia GmbH (David's step, with his GitHub sign-in).
3. **Rehearse the helper:** run Helper release from the public repository's Actions tab (rehearse ticked). It runs
   `check`, a draft named `helper-rehearsal-<run id>` with no tag, `build`, `sbom` and `verify` on the exported commit,
   then `cleanup` deletes the draft; nothing is published, and the tap, winget and `helper-updater` stay as they are.
4. **Tag the helper** `helper-vX.Y.Z` on the exported commit and push the tag. The jobs run in this order: `check` (the
   tag equals both versions, the updater's public key and feed are set, every secret and variable below is set, Google
   Play's licensing key is in `contract/store.json`, `cargo deny`), `draft` (a draft release for the tag), `build`
   (macOS Apple silicon and Intel, both on the arm64 `xcode-27` runner with Xcode 27.2, since the `macos-26` image's
   Xcode 26 cannot compile `icons/AppIcon.icon`, `actool` through a shim that gives it a stdin, which the CLI under
   tauri-action leaves closed, and `hdiutil create` through a shim that retries it, since XProtect can hold a new disk
   image; Developer ID signed and notarised; Windows x64
   on `windows-2025`, NSIS, signed through Artifact Signing; the updater files and `latest.json`; each build's symbols,
   the release profile's line tables split out of the program, a dSYM on macOS and a PDB on Windows, kept as the run's
   artifact `symbols-<version>-<target>` for 90 days, the longest a public repository keeps one, so a crash report's
   frames can be named: download a release's to keep them longer) beside `sbom` (one CycloneDX SBOM per shipped
   target), then `verify` (from the draft: the Windows installer and the program it installs signed by Pluxia GmbH,
   installed, started until its pipe says hello, uninstalled, on x64 and on arm64 under emulation; each disk image's
   app signed by the team, notarised and stapled), then `publish` (checks that `latest.json` names all six platforms
   and that every updater file is signed with the key installed helpers trust, points its URLs at github.com
   downloads, publishes the release without making it Latest, which the add-on's releases keep, creates the permanent
   `helper-updater` release the first time, puts the films' `figures-<digest>.zip` there once for each new set, and
   then `latest.json` and the three installers under fixed names, `QuestPump-mac-apple-silicon.dmg`,
   `QuestPump-mac-intel.dmg` and `QuestPump-windows-setup.exe`, that the updater and the site's download links read
   through the report Worker's `/get/` redirects, `contract/get-api.yaml`), then `cask` and `winget`.
5. **Tag the add-on** `vX.Y.Z` on the same commit and push the tag, once the helper's release made `helper-updater`.
   `check` (the tag is vX.Y.Z, `CHANGELOG.md` has its section, the updater's key and `PHONE_APPS` are set,
   `helper-updater` exists, the packager's dry run passes `tools/package-check.sh` and its `QuestPump.toc` names the
   tag), then `release` (the packager, BigWigsMods/packager pinned to a commit, in the environment `addon-release`:
   CurseForge for Forever and retail, Wago, and this repository's release of the tag with its `release.json`, which
   takes Latest), then `feed` (`tools/addon_feed.py` writes `addon.json` for that release's zip, `tauri signer sign`
   signs it with the updater's key into `addon.json.sig`, its key id is checked against `plugins.updater.pubkey`, and
   the zip as `QuestPump-addon.zip`, the signature and `addon.json` go to `helper-updater`, from which QuestPump on this
   computer installs and updates the add-on).
6. **The day the phone apps can be installed:** set the variable `PHONE_APPS` to `true`, then run Addon release from
   the Actions tab (feed_only ticked): it signs `addon.json` again for the newest `vX.Y.Z` release with the new
   `phone_apps` and no new version.
7. A failed job leaves its draft (the helper) or its uploads so far (the add-on): re-run the failed jobs, and the
   helper's draft and files are reused. When `publish` says `latest.json` lacks a platform (two build jobs wrote it at
   once), re-run that platform's build job. When one of the add-on's store uploads fails, the packager still makes the
   others and the GitHub release, and the job fails: upload that store's file by hand from the release's zip, then run
   Addon release from the Actions tab for the feed.

The add-on's feed, `addon.json` on `helper-updater` (`tools/addon_feed.py` has the format): `version` (the tag without
its "v"), `pub_date` (the release's publication, RFC 3339 in UTC), `interfaces` (the TOC's), `phone_apps`, `zip` (the URL
of `QuestPump-addon.zip` on `helper-updater`, its size and SHA-256) and `files` (every file in the zip by path, with its
size and SHA-256). `addon.json.sig` is the base64 of a minisign signature by the updater's key whose trusted comment ends
in `version:X.Y.Z`, checked as `tauri-plugin-updater` checks an update's.

A shipped build watches and lists the same games as a local one. Until 2026-10-28 the workflow set
`QUESTPUMP_SHIP=1`, and a shipped build left out each game `contract/links.json` marked `pending` (League of Legends
with TFT and Legends of Runeterra, waiting for Riot's review of QuestPump's registrations, still Pending Review); David
chose to ship them on by default while it is pending ("full features", `docs/computer-games.md` §6.5), so a release
no longer waits on Riot.

What the releases need, set up once in Pluxia-GmbH/questpump (each marked set or not set as of 2026-10-31):

- **The updater's key** (made 2026-09-30, key id `1A8A204133C16D6E`): its public key's content is
  `plugins.updater.pubkey` in `tauri.conf.json`, the private key and its password are the two secrets below, with a
  copy kept away from the Mac. Never make another: installed helpers trust only this key, and the `publish` job and the
  add-on's `feed` job fail when a file was signed with any other.
- **Secrets**, in the environment `release-signing`, whose deployment rules admit the tags `helper-v*` and `v*` and
  the branch `main` (the rehearsal and the feed's hand run) alone, and which the helper's `check`, `build`, `cask` and
  `winget` and the add-on's `check` and `feed` name (all set but `WINGET_TOKEN`; repository secrets until
  2026-10-31, which those jobs read too until they are deleted after the first run from the environment):
  - `TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`: the updater's minisign key, which signs the
    helper's updates and the add-on's feed.
  - `APPLE_CERTIFICATE`: the "Developer ID Application: Pluxia GmbH (PQFUQJH377)" certificate with its private key as a
    `.p12`, in base64; `APPLE_CERTIFICATE_PASSWORD`: the `.p12`'s password. `tools/signing/export-identity.swift`
    exports that one identity from the login keychain (and again after the renewal before 2027-02-01).
  - `APPLE_API_KEY`, `APPLE_API_ISSUER`, `APPLE_API_KEY_P8`: the App Store Connect team key for notarisation, from
    Secret Manager's `ASC_KEY_ID`, `ASC_ISSUER_ID` and `ASC_KEY` (the `.p8` in base64).
  - `AZURE_CLIENT_ID`, `AZURE_CLIENT_SECRET`, `AZURE_TENANT_ID`: an app registration with the role "Artifact Signing
    Certificate Profile Signer" on the Artifact Signing account (`artifact-signing-cli` logs in with a client secret).
  - `HOMEBREW_TAP_DEPLOY_KEY`: the private half of a deploy key with write access on Pluxia-GmbH/homebrew-tap, which
    reaches that repository only.
  - `WINGET_TOKEN`: a classic token with the `public_repo` scope, of an account with no rights in Pluxia-GmbH;
    `wingetcreate` forks microsoft/winget-pkgs under its owner. Without it the `winget` job skips with a notice.
- **Variables** (repository variables): `AZURE_ARTIFACT_SIGNING_ACCOUNT` (the Artifact Signing account's name) and
  `AZURE_ARTIFACT_SIGNING_CERTIFICATE_PROFILE` (the certificate profile's name), which
  `tauri.release-windows.conf.json`'s sign command reads (both set); `PHONE_APPS`, `true` once the phone apps can be
  installed from the stores, `false` or unset before (not set, so `false`).
- **The environment `addon-release`** (made 2026-10-31), with a deployment rule that admits tags `v*` alone: the secrets
  `CF_API_KEY` (a CurseForge API token, from the author's account settings) and `WAGO_API_TOKEN` (a Wago Addons API
  token), and the variables `CURSEFORGE_PROJECT_ID` (the project's number, 1776368, which `QuestPump.toc` names for
  add-on managers as `## X-Curse-Project-ID`) and `WAGO_PROJECT_ID` (the project's id, aNerod62, which it names as
  `## X-Wago-ID`). Without any of the four the add-on's `release` job stops before it uploads.
- **The tap:** a public repository Pluxia-GmbH/homebrew-tap; the `cask` job writes `Casks/questpump.rb` there from
  `helper/packaging/homebrew/questpump.rb` (change the template here, never the tap's copy).
- **The repository's settings:** private vulnerability reporting on (`.github/SECURITY.md` names it; on), and
  immutable releases off (off), since every release replaces the files of the `helper-updater` release.
- **The first notarisation** (done 2026-09-30): a new Apple team's first uploads can be held for a closer look, up to
  a day, and the build job gives up after 3 hours, so one Developer ID build went through by hand first (`cargo tauri
  build --bundles app` with `APPLE_SIGNING_IDENTITY`, `APPLE_API_KEY`, `APPLE_API_ISSUER` and `APPLE_API_KEY_PATH`
  set, as the build job does it); Apple accepted it within minutes.
- **winget, after the first release:** `wingetcreate update` only updates a package winget-pkgs already has, so the
  first version is submitted by hand from `helper/packaging/winget/`: fill in `@VERSION@`, `@URL@` (the release's
  `QuestPump_<version>_x64-setup.exe`) and `@SHA256@` (its SHA-256 in upper case, `Get-FileHash`), check the
  folder with `winget validate --manifest <folder>` (and `winget install --manifest <folder>` after
  `winget settings --enable LocalManifestFiles`), then open a pull request that adds the three files under
  `manifests/p/Pluxia/QuestPump/<version>/` in microsoft/winget-pkgs (or `wingetcreate submit <folder>`), and
  sign Microsoft's CLA when its bot asks. Once it is merged, set `WINGET_TOKEN` and turn on `WINGET` in
  `site/src/launch.ts`, which shows the winget line on `/helper` and `/start`.
