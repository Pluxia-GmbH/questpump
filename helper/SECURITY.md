# Security policy

To report a vulnerability in QuestPump for Mac and Windows (the desktop app built from this folder), the QuestPump
add-on or the QuestPump apps, write privately, never in a public issue:

- by email to [security@questpump.com](mailto:security@questpump.com), or
- through GitHub's private vulnerability reporting:
  <https://github.com/Pluxia-GmbH/questpump/security/advisories/new>.

We aim to acknowledge your report within 5 working days, and for a confirmed issue to ship a fix, or to tell you our
plan, within 30 days.

The full policy, with what to include, the supported versions (the helper gets security updates for at least five
years after its first public release, and at least until 2031-11-04), how releases are protected, what the helper
exposes on the network and the computer, and how Pluxia GmbH reports actively exploited vulnerabilities under the EU
Cyber Resilience Act: <https://questpump.com/security>.

What the desktop app adds since slice 35, beside the LAN API, the local socket and Bluetooth the policy describes:

- **The Game State Integration listener,** only while Counter-Strike 2 or Dota 2 is turned on: HTTP on 127.0.0.1 alone
  (never the network, so no firewall rule), a port the system picks, a 24-byte random token in the cfg file it writes
  into the game's folder, a post without that token refused (401), 64 KiB a post at most, and only the token and the
  match's phase read.
- **One outgoing connection on the loopback,** only while Rocket League is turned on and runs: to 127.0.0.1 at the port
  its `TAStatsAPI.ini` names, read only; nothing is ever written to that socket, whose commands would steer the game.
- **Pro's proof, checked offline** (`crates/questpump-core/src/pro.rs`): an iPhone's App Store JWS chain to the pinned
  Apple Root CA - G3 (a release build takes `Production` only), Android's Play purchase signature under the app's
  licensing key; no `rsa` crate (RUSTSEC-2023-0071). The check sits in MIT code, so a build from source can skip it, and
  a Play proof replayed (its signed text never changes) holds 30 days from its last receipt plus a grace week: accepted,
  since it unlocks only Pro's small extras on the computer and nothing the game reads. A refund or a lapse reaches the
  computer at the phone's next sync (`none`); a new pairing code clears every grant (`pro.json`).
- **Outgoing HTTPS connections to two known hosts only:** the updater's (GitHub) and the moves' films (the same
  release, each file checked against its SHA-256 before it plays); and questpump.com for the feedback the player sends,
  the crash reports and anonymous counts (each under its switch) and the sync (slice 60, while Sync is on: batches
  sealed with AES-256-GCM under a key derived from a secret only the player's devices hold, a bearer token whose
  SHA-256 alone the server keeps, and one WebSocket that says only that the account's cursor moved).
