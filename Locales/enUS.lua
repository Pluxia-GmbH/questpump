-- Every user-facing string (enUS), loaded first. Other locales come from the packager's CurseForge substitution and
-- are guarded by GetLocale() (slice 3). Rules tools/lint.lua enforces (make lint): every key the code uses exists
-- here; no string carries a URL, a price, a store name, a donation ask, Pro or the coach; the helper and the phone app
-- are named only in the setup (ui.setup.*), status (status.*, cmd.status.*) and help (cmd.help.*) keys, as strongly
-- recommended companions; no string makes a health claim (the deny-list), the one safety text excepted. Move lines
-- stay under 90 characters.

local ADDON, ns = ...

-- A missing key shows as its own name: visible, never a Lua error.
local L = setmetatable({}, { __index = function(_, key) return key end })
ns.L = L

-- The one safety text (docs/safety.md, David, 2026-09-23): the same string in the listings and the apps (GPSR Art. 9(7)
-- and 19(d)); the addon shows it on Your setup's How hard page and at /qp safety, and nowhere else (David, 2026-10-24:
-- only what is legally necessary).
L["safety.note"] = "A game, not medical advice. Stop and get help for chest pain, fainting, unusual breathlessness "
	.. "or a racing heart. Severe muscle pain, swelling or dark urine after a session needs a doctor that day. Ask a "
	.. "clinician first if you have heart, lung, kidney or metabolic disease or high blood pressure, are pregnant or "
	.. "had recent surgery."

-- The kit's own words (Kit.lua): a window's bar ("More" also a More section's words without its own), and a check
-- box's click hint, the last line of its tooltip
L["ui.more"] = "More"
L["ui.more.click"] = "Click: show the rest"
L["ui.click.on"] = "Click: turn it on"
L["ui.click.off"] = "Click: turn it off"

-- Chat
L["chat.prefix"] = "|cff33ff99QuestPump|r"
L["chat.version"] = "%s (build %s, interface %s). Type /qp help for the commands."
L["chat.unknown"] = "No command \"%s\". Type /qp help."
L["chat.combat"] = "Not in combat."
L["chat.locked"] = "No pumps today: you felt unwell. Banked moves wait for the next game day."
L["chat.notnow"] = "Not now: finish what you are doing, then try again."
L["chat.busy"] = "Finish the current pump first."
L["ui.setup.first"] = "First the setup: set up QuestPump on this computer and the phone app, or play without them."
-- the same while the setup asks for no phone (Setup.AsksPhone: the phone apps are not in the stores yet)
L["ui.setup.first.one"] = "First the setup: set up QuestPump on this computer, or play without it."
L["chat.toolarge"] = "Your saved data is too large: the game may not save it at the next logout."
-- a failed migration of the save (DB.lua): the save from before the update is the backup
L["chat.migrationfailed"] = "QuestPump could not update your saved progress and started fresh. The old save is kept: "
	.. "once a QuestPump update fixes this, /qp restore-backup brings it back."
L["chat.trial"] = "From now on every quest you turn in is a pump: do the move, type your count, beat it next time."

-- /qp help
L["cmd.help.help"] = "/qp help [command]: the commands, or what one does"
L["cmd.help.companions"] = "QuestPump on this computer and the phone app are strongly recommended companions: they "
	.. "keep your progress saved and bring your pumps to your phone and wrist. Find QuestPump for this computer on the "
	.. "addon's page; /qp setup shows how."
-- while the setup asks for the helper alone (Setup.AsksPhone)
L["cmd.help.companions.one"] = "QuestPump on this computer is a strongly recommended companion: it keeps your progress "
	.. "saved and lets you do pumps away from the game. Find QuestPump for this computer on the addon's page; /qp setup "
	.. "shows how."
-- /qp help's lines, a group each ("Play: /qp pump · /qp try"), by what a player does (Core.lua HELP_GROUPS)
L["cmd.help.line"] = "%s: %s"
L["cmd.help.group.play"] = "Play"
L["cmd.help.group.sheet"] = "Your sheet"
L["cmd.help.group.setup"] = "Setup"
L["cmd.help.group.others"] = "Other players"
L["cmd.help.group.data"] = "Your data"
L["cmd.help.group.help"] = "Help and fixes"
L["cmd.status.help"] = "/qp status: today's pumps, banked moves, the stretch, saving, the move figures and the "
	.. "companions"
L["cmd.setup.help"] = "/qp setup: the setup window at any time: QuestPump's two companions, then Your setup"
L["cmd.pump.help"] = "/qp pump: pay your banked moves now, or start a pump"
L["cmd.try.help"] = "/qp try: a practice move of 20 s; nothing is stored"
L["cmd.card.help"] = "/qp card: the day's tally on one card, to share or for a screenshot"
L["cmd.player.help"] = "/qp player [name]: a player's QuestPump card, pact or duel (your target without a name)"
L["cmd.player.usage"] = "Target a player, or name one: /qp player Name"
L["cmd.player.none"] = "Nothing to choose for %s now: others' cards are off (Options > AddOns > QuestPump > Social), "
	.. "and no pact or duel can be offered."
L["cmd.instances.help"] = "/qp instances on|off: pumps inside dungeons and raids (off: they wait until you are out)"
L["cmd.safety.help"] = "/qp safety: the safety note"
L["cmd.save.help"] = "/qp save: reload the interface, so the game writes your saved data now"
L["cmd.restore.help"] = "/qp restore-backup: swap in this game's save kept as the backup (before a reset, or an "
	.. "update that failed)"
L["cmd.why.help"] = "/qp why [quest number]: why this pump picked its moves, or how a quest scores"
L["cmd.how.help"] = "/qp how: the setup, the cue and the stop line of the current move"
L["cmd.gear.help"] = "/qp gear [name]: your equipment, space, quiet mode and Body switches; a name switches one"
L["cmd.undo.help"] = "/qp undo <count> [kg] [move]: correct the count you just typed and its weight (for five "
	.. "minutes), or a move's latest count by its name, as /qp undo 7 archer or /qp undo 12 14kg. To pick another set, "
	.. "right-click the move's record: Correct a count"

L["cmd.instances.on"] = "Pumps run inside dungeons and raids."
L["cmd.instances.off"] = "Pumps inside dungeons and raids wait until you are out."
L["cmd.restore.none"] = "There is no backup to restore."
L["cmd.restore.done"] = "Backup restored in this game. Type /reload to load it."

-- /qp why, /qp how, /qp undo
L["cmd.why.title"] = "Why these moves:"
L["cmd.why.none"] = "No pump yet: turn in a quest or type /qp pump."
L["cmd.why.quest"] = "How this quest scores:"
L["why.quest"] = "Quest %d: score %d%%, tier %d (%s). XP %d%%, level %d%%, tag %d%%, goals %d%%, travel %d%%."
L["why.mapped"] = "Quest list: %s, level %s%s."
L["why.mapped.unknown"] = "unknown"
L["why.mapped.boss"] = "a boss"
L["why.mapped.finale"] = "the last quest of a story"
L["why.mapped.group"] = "a group quest"
L["why.mapped.themes"] = "themes %s"
L["why.unmapped"] = "Quest %d is not in the quest list, so its goals and tag set its kind."
L["why.flavour"] = "Line for %s: %s"
L["flavour.kind.rep"] = "reps"
L["flavour.kind.hold"] = "a hold"
L["flavour.kind.cardio"] = "cardio"
L["flavour.kind.flow"] = "a stretch"
L["why.trial"] = "%s: the first record on its ladder."
L["why.chosen"] = "%s: you started it yourself."
L["why.proving"] = "%s: a clean 30-s all-out set proves your next step."
L["why.move"] = "%s: %d%% (due %d, rested %d, variety %d, fit %d, theme %d)."
L["why.finisher"] = "%s ends it all out: %d%% (due %d, rested %d, variety %d, fit %d, theme %d)."
L["why.next"] = "Next best: %s."
L["why.block"] = "The day plan's turn for %s:"
L["why.pace.light"] = "Moves per quest is Light: each quest gives one move."
L["why.pace.extreme"] = "Moves per quest is Extreme: each quest gives one move more."
L["block.strength"] = "strength"
L["block.cardio"] = "cardio"
L["block.mobility"] = "stretching and joint drills"
L["block.yoga"] = "yoga and balance"
L["why.removed"] = "Left out: %s."
L["why.reason.pump"] = "this pump has that move or region already"
L["why.reason.placement"] = "one first max set on a ladder per pump"
L["why.reason.owed"] = "banked already, it rides along"
L["cmd.how.line"] = "%s. Setup: %s Cue: %s Stop: %s"
L["cmd.how.line.nostop"] = "%s. Setup: %s Cue: %s"
L["cmd.how.none"] = "No move yet: turn in a quest or type /qp pump."
L["cmd.undo.usage"] = "Last count: %s, %d. Type /qp undo <count> to correct it."
L["cmd.undo.noMove"] = "No typed count of a move named \"%s\" to correct."
-- a move done with a weight (ns.Loaded): its set with the weight ("11 kg × 16", Sheet.LoadAmount)
L["cmd.undo.usageLoad"] = "Last set: %s, %s. Type /qp undo <count> <kg> to correct it."
-- a count corrected (slice 44), by /qp undo or Correct a count's window (Settings.lua): the move and the set as it
-- reads now ("15 reps", "45 s", "11 kg × 16", "Good": Settings_.SetWords)
L["cmd.correct.done"] = "Corrected: %s, %s."
L["cmd.undo.noKg"] = "%s takes no weight. Type /qp undo <count>."
L["cmd.undo.none"] = "Nothing to correct: /qp undo works for five minutes after a typed count."

-- /qp gear: the equipment, the space, quiet mode and the Body switches
L["cmd.gear.title"] = "Your setup (/qp gear <name> switches one):"
L["cmd.gear.equipment"] = "Equipment on: %s. The rest, by name: %s."
L["cmd.gear.room"] = "Room: %s."
L["cmd.gear.body"] = "Body: %s."
L["cmd.gear.pain"] = "Pain flags: %s. /qp gear clear <joint> clears one."
L["cmd.gear.pool"] = "Your pool: %d moves in %d patterns."
L["cmd.gear.set"] = "%s: %s. Your pool: %d moves in %d patterns."
L["cmd.gear.few"] = "%d patterns left: pumps will repeat."
L["cmd.gear.unknown"] = "No setting \"%s\". Type /qp gear for the list."
L["cmd.gear.cleared"] = "Pain flag cleared: %s."
L["cmd.gear.notFlagged"] = "No pain flag on %s."
L["gear.on"] = "on"
L["gear.off"] = "off"
L["gear.none"] = "none"
L["gear.stable_chair"] = "Stable chair (no wheels, or a sofa)"
L["gear.table"] = "Sturdy table"
L["gear.bar"] = "Pull-up bar"
L["gear.fixed_bar"] = "Fixed pull-up bar (not a doorway bar)"
L["gear.bands"] = "Bands with a door anchor"
L["gear.kettlebell"] = "Kettlebell"
L["gear.dumbbells"] = "Dumbbells"
L["gear.barbell"] = "Barbell and rack"
L["gear.rope"] = "Jump rope"
L["gear.bike"] = "Exercise bike"
L["gear.stairs"] = "Stairs"
L["gear.pad"] = "Walking pad"
L["gear.step"] = "Step or box"
L["gear.wheel"] = "Ab wheel"
L["gear.parallettes"] = "Parallettes or two stacks of hard books"
L["gear.rings"] = "Rings or straps"
L["gear.vest"] = "Weighted vest"
L["gear.ball"] = "Medicine ball"
L["gear.roller"] = "Foam roller"
L["gear.backpack"] = "Backpack you can load to 5-15 kg (books or water bottles)"
L["gear.pole"] = "A pole outdoors (a steel post fixed in the ground)"
L["gear.floor"] = "Floor moves"
L["gear.space"] = "Room to move"
L["gear.quiet"] = "Quiet mode"
L["gear.nojump"] = "No jumping"
L["gear.nostrain"] = "Avoid breath-hold strain"
L["gear.noheaddown"] = "Avoid head-down positions"

-- /qp status (the day's counts in the day card's words, streamer.share.*: "1 pump, 3 moves, 0 new records")
L["cmd.status.today"] = "Today: %s."
L["cmd.status.today.app"] = "Of these, in the app: %s."
L["cmd.status.owed"] = "Banked moves: %d."
L["cmd.status.stretch"] = "Since your last pump: %s yd on foot, %s yd mounted, %s yd swimming, %s yd on a taxi, "
	.. "%d min active."
L["cmd.status.saving.loaded"] = "Saving: the game loaded your saved data."
L["cmd.status.saving.restored"] = "Saving: the game did not load your saved data; QuestPump on this computer "
	.. "restored it from its copy."
L["cmd.status.saving.lost"] = "Saving: the game did not load your saved data; progress lasts this session. /qp export "
	.. "copies it out."
L["cmd.status.saving.unknown"] = "Saving: not checked yet (it shows after a /reload)."
L["cmd.status.figures.ok"] = "Move figures: installed."
L["cmd.status.figures.old"] = "Move figures: an older pack; the moves it lacks show their key pose."
L["cmd.status.figures.missing"] = "Move figures: missing, so every move shows its key pose. QuestPump Figures is a "
	.. "download of its own. QuestPump on this computer adds it for you."
L["cmd.status.figures.outdated"] = "Move figures: installed, but made for another game version, so the game does not "
	.. "load them. Update QuestPump Figures."
L["cmd.status.figures.disabled"] = "Move figures: turned off in the AddOns list, so every move shows its key pose."
-- the voice pack, a download of its own as the figures are (slice 71): QuestPump plays without it, in silence
L["cmd.status.voice.ok"] = "Voice pack: installed."
L["cmd.status.voice.old"] = "Voice pack: from another version than QuestPump, so some lines are not said. Update "
	.. "both."
L["cmd.status.voice.missing"] = "Voice pack: missing, so nothing is said aloud. QuestPump Voice is a download of its "
	.. "own. QuestPump on this computer adds it for you."
L["cmd.status.voice.outdated"] = "Voice pack: installed, but made for another game version, so the game does not load "
	.. "it. Update QuestPump Voice."
L["cmd.status.voice.unplayable"] = "Voice pack: the game did not play its files, so the voice is off until you "
	.. "restart the game."
L["cmd.status.voice.disabled"] = "Voice pack: turned off in the AddOns list, so nothing is said aloud."
-- each companion on its own line: the helper by ns.HelperState, the phone by its pairing (ns.PhoneQuiet)
L["cmd.status.computer.on"] = "QuestPump on this computer: running, version %s."
L["cmd.status.computer.none"] = "QuestPump on this computer: not set up yet. Find it on the addon's page."
L["cmd.status.computer.off"] = "QuestPump on this computer: it was not running when the game loaded."
L["cmd.status.computer.blind"] = "QuestPump on this computer: running, but it cannot see the game yet. In its window, "
	.. "press Allow screen reading."
L["cmd.status.computer.stopped"] = "QuestPump on this computer: running, but it cannot read the game right now. Open "
	.. "its window."
L["cmd.status.computer.disabled"] = "QuestPump on this computer: QuestPump Mirror is turned off in the AddOns list."
L["cmd.status.computer.outdated"] = "QuestPump on this computer: QuestPump Mirror is out of date for this game version."
L["cmd.status.phone.paired"] = "Phone app: paired with your %s."
L["cmd.status.phone.quiet"] = "Phone app: paired with your %s, which last connected on %s."
L["cmd.status.phone.none"] = "Phone app: not paired yet."
L["cmd.status.phone.ios"] = "iPhone"
L["cmd.status.phone.android"] = "Android phone"
L["cmd.status.setup"] = "Setup: waiting for your choice: set up QuestPump on this computer and the phone app, or "
	.. "play without them. No pump runs until then."
L["cmd.status.setup.yours"] = "Setup: waiting for Your setup: answer the few questions in the setup window, or close "
	.. "it to start. No pump runs until then."
L["cmd.status.setup.how"] = "Both are strongly recommended: /qp setup shows how to set them up."
-- the two lines above while the setup asks for no phone (Setup.AsksPhone: the phone apps are not in the stores yet)
L["cmd.status.setup.one"] = "Setup: waiting for your choice: set up QuestPump on this computer, or play without it. No "
	.. "pump runs until then."
L["cmd.status.setup.how.one"] = "It is strongly recommended: /qp setup shows how to set it up."
L["cmd.status.locked"] = "Locked until %02d:00: you felt unwell today."
L["cmd.status.state"] = "Pump: %s."

L["state.idle"] = "none"
L["state.settling"] = "settling"
L["state.ready"] = "ready"
L["state.card"] = "waiting for you to stop"
L["state.open"] = "open"
L["state.countdown"] = "counting down"
L["state.running"] = "running"
L["state.result"] = "result"
L["state.waiting"] = "waiting for combat to end"
L["state.held"] = "held by combat"
L["state.done"] = "done"

-- Saving and the HUD status lines
L["status.beta.restored"] = "The game did not load your saved data; QuestPump on this computer restored it from its "
	.. "copy."
-- once, when the pairing block names a phone the save has not heard of (Events.lua, Setup.Evaluate)
L["status.paired"] = "Your phone is paired: each pump's countdown goes to it."
L["status.hud.stale"] = "Start QuestPump on this computer"
L["status.hud.stale.title"] = "QuestPump on this computer"
L["status.hud.stale.tip"] = "The phone app gets your progress from it."
L["status.hud.stale.click"] = "Click once it runs: check again"
L["status.hud.lost"] = "Progress lasts this session only"
L["status.hud.lost.click"] = "Click: copy it out"
L["status.hud.lost.title"] = "Saving your progress"
-- the companions' nudges (Setup.Nudge, the unfinished line's and the lost save's tooltips), until that one is set up
L["status.nudge.save"] = "QuestPump on this computer keeps your progress and brings it back at every load. Find it "
	.. "on the addon's page."
L["status.nudge.owed"] = "You can do your banked moves in the QuestPump app on your phone."
L["status.nudge.streak"] = "On days without the game, a pump in the QuestPump app on your phone keeps your streak "
	.. "going."
-- a newer addon in the helper's signed feed (Events.lua Lc.TellUpdate: one chat line a session, and in /qp status), by
-- who puts it in: QuestPump on this computer when the game closes, or the player some other way; no address, no store
L["status.update.ready"] = "QuestPump %s is ready. It installs when you close the game."
L["status.update.out"] = "QuestPump %s is out. QuestPump on this computer can install it."

-- The status strip (Bridge.lua): /qp bridge and /qp ping
L["cmd.bridge.help"] = "/qp bridge: check the small status strip in the top-left corner"
L["cmd.ping.help"] = "/qp ping: send a test signal on the status strip"
L["status.bridge.strip"] = "Status strip: %d × %d pixels in the top-left corner. Screen %d × %d, scale %.3f."
L["status.bridge.shown"] = "It shows now. Load %d, event %d, %d frames waiting. QuestPump on this computer reads it "
	.. "when it runs."
L["status.bridge.hidden"] = "It is hidden now (a cinematic, a movie, or the interface is hidden). Load %d, event %d, "
	.. "%d frames waiting."
L["status.bridge.ping"] = "Test signal sent. If QuestPump on this computer runs, it says so out loud."

-- The HUD
L["hud.title"] = "QuestPump"
L["hud.chip"] = "Next pump: %d moves"
L["hud.chip.one"] = "Next pump: 1 move"
L["hud.chip.both"] = "Next pump: %d moves + %d banked"
L["hud.chip.both.one"] = "Next pump: %d move + %d banked"
L["hud.chip.riders"] = "Next pump: %d banked"
L["tip.chip.title"] = "Your next pump"
L["tip.chip.when"] = "It opens once the quest window closes and you stand still."
L["tip.chip.snoozed"] = "Hidden for a moment. It comes back when you stand still."
-- the pumps that wait for the flight (Lc.Waiting, "On my next flight")
L["hud.wait"] = "%d pumps wait for your next flight"
L["hud.wait.one"] = "%d pump waits for your next flight"
L["tip.wait.title"] = "Pumps for your next flight"
L["tip.wait.body"] = "They open one after another when you fly. What has not opened when you land is banked."
L["tip.wait.click"] = "Click: start the next one now"
L["tip.chip.click"] = "Click: start it now"
L["hud.locked"] = "No pumps until %02d:00 · you felt unwell"
L["hud.owed"] = "%d banked · they join your next pump"
L["hud.owed.one"] = "%d banked · it joins your next pump"
L["hud.owed.some"] = "%d banked · %d join your next pump"
L["hud.owed.some.one"] = "%d banked · %d joins your next pump"
L["hud.owed.waiting"] = "%d banked · waiting on your setup"
L["hud.owed.locked"] = "%d banked · they wait until %02d:00"
L["hud.owed.after.party"] = "%d banked · after this group"
L["hud.owed.after.instance"] = "%d banked · after this instance"
L["hud.owed.after.pvp"] = "%d banked · after this battleground"
L["hud.held.title"] = "Waiting on your setup"
L["hud.held.skip"] = "Skip: %s (since %s)"
L["hud.held.settings"] = "Open settings"
L["hud.held.unknown"] = "A move from another version"
L["tip.owed.title"] = "Banked moves"
L["tip.owed.size"] = "%d moves, about %d min"
L["tip.owed.size.one"] = "%d move, about %d min"
L["tip.owed.move"] = "%s ×%d"
L["tip.owed.more"] = "+%d more"
L["tip.owed.banked"] = "%d you banked"
L["tip.owed.owed"] = "%d left open by a group, a fight, a pause, a logout or a death"
L["tip.owed.next"] = "They join your next pump, after its own moves."
L["tip.owed.after"] = "They join the first pump after this."
L["tip.owed.locked"] = "They wait until %02d:00: you felt unwell today."
L["tip.owed.held"] = "Waiting on your setup: %d (a pain flag, a Body switch, your equipment, an unchecked move or "
	.. "your step)."
L["tip.owed.skip"] = "Skip on a move's Ready view drops it for good. It earns nothing."
L["tip.owed.click"] = "Click: do them now"
L["tip.owed.clickHeld"] = "Click: skip one that waits"
L["tip.owed.rightHeld"] = "Right-click: skip one that waits"
L["hud.cleanSlate"] = "Clean Slate: nothing banked"

-- The pump card
L["ui.panel.title.pump"] = "Pump"
L["ui.panel.title.practice"] = "Practice move"
L["ui.panel.title.standup"] = "Stand up"
L["ui.panel.title.owed"] = "Banked moves"
L["ui.panel.title.asked"] = "A pump you asked for"
-- a stretch check started from a step's card (slice 42; the apps' pump.source.check)
L["ui.panel.title.check"] = "Stretch check"
L["ui.panel.whisper"] = "%s · whisper from %s"
L["ui.panel.move"] = "Move %d of %d"
L["ui.panel.merged"] = "merged %d turn-ins"
L["ui.panel.riders"] = "then %d banked"
L["ui.panel.unfinishedOf"] = "Banked %d of %d"
L["ui.panel.riderOf"] = "Banked %d of %d · move %d of %d"
L["ui.panel.allout"] = "(all out)"
L["ui.panel.start"] = "Start"
L["ui.backup.tip"] = "The same level, done another way."
-- the swap row's click and its arrow (Panel.lua renderSwap, 2026-10-16: one place to swap)
L["ui.backup.click"] = "Click: swap it in"
L["ui.backup.menu"] = "More ways to swap"
L["ui.panel.settings"] = "Settings"
L["ui.panel.settings.tip"] = "Your equipment, the Body switches and the moves you leave out. The pump waits while "
	.. "they are open, and a move that no longer fits becomes its backup."
L["ui.panel.settings.click"] = "Click: open the settings"
L["ui.backup.tip.closest"] = "Nothing at this level fits your setup: the closest below."
L["ui.backup.tip.back"] = "The move you swapped away from."
L["ui.backup.tip.easier"] = "A simpler move that builds up to this one."
L["ui.backup.tip.another"] = "Another move that trains the same way."
L["ui.swap.title"] = "Swap this move"
L["ui.swap.another"] = "Another move: %s"
L["ui.swap.easier"] = "Easier version: %s"
L["ui.swap.back"] = "Back to: %s"
-- the swap row's arrow lists the backup too where another offer is on the row (a first appearance's easier version)
L["ui.swap.backup"] = "Same level: %s"
L["ui.swap.closest"] = "Closest your setup allows: %s"
L["ui.panel.goal"] = "Your goal"
L["ui.panel.how"] = "How"
-- How's bar on a move the player knows (Panel.lua buildReady): its tooltip, the hint by whether it is open
L["ui.panel.how.tip"] = "How to do the move, step by step."
L["ui.panel.how.show"] = "Click: show the steps"
L["ui.panel.how.hide"] = "Click: hide the steps"
L["ui.panel.tips"] = "Tips"
-- the guide's button and its tooltip, at the end of the How heading's line and on the bar while the move runs
-- (Panel.lua guideButton): its words say what a click does
L["ui.panel.guide.play"] = "Read How and Tips aloud"
L["ui.panel.guide.pause"] = "Pause reading"
L["ui.panel.guide.resume"] = "Go on reading"
L["ui.panel.guide.tip"] = "Reads the move's How and Tips aloud. It keeps reading while you do the move."
L["ui.panel.step"] = "%d."
L["ui.panel.moves"] = "Moves in this pump"
L["ui.panel.above"] = "%d more above"
L["ui.panel.below"] = "%d more to come"
L["ui.panel.bank"] = "Bank all"
L["ui.panel.bank.tip"] = "The moves are banked and ride with your next pump."
L["ui.panel.bank.click"] = "Click: bank them and play on"
-- Skip, on an unfinished move riding along (Lc.Skip)
L["ui.panel.snooze"] = "Hide for a moment"
L["ui.panel.snooze.tip"] = "Gone for %d seconds so you can finish what you were doing. It comes back once you "
	.. "stand still."
L["ui.panel.snooze.click"] = "Click: hide it for now"
L["ui.panel.skip"] = "Skip"
L["ui.panel.skip.tip"] = "Drops this banked move for good. It earns nothing and costs nothing."
L["ui.panel.skip.click"] = "Click: drop this move"
-- the Streamer layout's card moves by its title (Panel.lua, slice 23)
L["ui.panel.grip"] = "Move the card"
L["ui.panel.grip.tip"] = "Drag it where your stream needs it. Right-click puts it back in the corner."
-- a mixed party's question at its first turn-in (Lc.AskParty; Panel.lua's popup)
L["ui.party.ask"] = "Not everyone in your group runs QuestPump. What should your turn-ins do while you quest "
	.. "together?"
L["ui.party.pause"] = "Pause while grouped"
L["ui.party.bank"] = "Bank all"
L["ui.party.pause.tip"] = "Your turn-ins cost nothing while you quest with this group."
L["ui.party.bank.tip"] = "Their moves are banked and ride with your next pump."
L["ui.panel.pause"] = "Pause"
L["ui.panel.pause.tip"] = "This pump ends and costs nothing, and so does every turn-in while paused. Banked "
	.. "moves still wait."
L["ui.panel.pause.click"] = "Click: how long, Something hurts or I feel unwell"
-- the Pause menu's title over the body's two stops (Panel.PauseMenu)
L["ui.pause.body"] = "Your body"
L["ui.pause.pain"] = "Something hurts"
L["ui.pause.unwell"] = "I feel unwell"
-- on the Ready view of the first pump of a session whose save the beta lost (Events.lua, lostNote; David, 2026-10-04:
-- one line in the place of the pain question that came before a pump)
L["ui.panel.lost"] = "Settings were lost: set pain flags again under Pause."
L["ui.unwellAsk.title"] = "Feeling unwell?"
L["ui.unwellAsk.how"] = "Pause QuestPump on this character? For how long:"
-- its until-04:00 row (Lc.PausePump's lock of every pump, here and in the apps), in place of hud.pause.day's words
L["ui.unwellAsk.day"] = "Until %02d:00 everywhere"
L["ui.unwellAsk.back.tip"] = "Back to the move. Nothing is paused."
L["ui.unwellAsk.back.click"] = "Click: back to the move"
L["ui.panel.rest"] = "Bank the rest"
-- Bank the rest and Close (Lc.BankRest): what is left of the pump stays as it was, owed or not
L["ui.panel.rest.tip"] = "The banked moves left keep waiting. What you did counts."
L["ui.panel.rest.click"] = "Click: end the pump here"
L["ui.panel.close"] = "Close"
L["ui.panel.close.tip"] = "Ends this pump. A move you asked for owes nothing, and banked moves still wait."
L["ui.panel.close.click"] = "Click: close the pump"
L["ui.panel.back"] = "Back"
L["ui.panel.back.tip"] = "Back to before you pressed Start. Nothing is saved."
L["ui.panel.back.click"] = "Click: undo Start"
L["ui.panel.done"] = "Done"
-- a steady move's last seconds (slice 52, Lc.KeepGoing): the clock runs on to the full move
L["ui.panel.keepGoing"] = "Keep going"
L["ui.panel.go"] = "GO"
L["ui.panel.ready"] = "Ready..."
-- the key that does a bar button's job, on its tooltip (the typed and spinner input modes capture the keyboard)
L["ui.key"] = "Key: %s"
L["key.space"] = "Space"
L["key.enter"] = "Enter"
L["key.spaceEnter"] = "Space or Enter"
L["ui.figure.pause"] = "Click the figure: pause or play"
L["ui.panel.firstSide"] = "First side"
L["ui.panel.switch"] = "Switch sides"
L["ui.side.first"] = "%s: first side"
L["ui.side.other"] = "%s: other side"
-- A side move's name before and after its run, and through a /qp try, which runs both sides as one: both sides, the
-- weaker side counted (David, 2026-10-04: 15 desk archer push-ups typed with both sides added, "it says nowhere")
L["ui.side.each"] = "%s, each side"

-- The goal block (Panel.lua): the count or seconds to reach now, big, its unit beside it, and the line under it.
L["ui.goal.atLeast"] = "%d+"
L["ui.goal.reps"] = "reps"
L["ui.goal.reps.one"] = "rep"
L["ui.goal.repsSide"] = "reps each side"
L["ui.goal.repsSide.one"] = "rep each side"
L["ui.goal.secs"] = "seconds"
L["ui.goal.secsSide"] = "seconds each side"
L["ui.goal.most"] = "As many as you can"
L["ui.goal.longest"] = "As long as you can"
L["ui.goal.onEachSide"] = "on each side"
L["ui.goal.line"] = "Goal: %s"
L["ui.goal.lineRecord"] = "record %s"
-- a move done with a weight (ns.Loaded): the planned weight after the unit, or the start range before one is typed
L["ui.goal.withLoad"] = "with %s kg"
L["ui.goal.withLoadEach"] = "with %s kg each"
L["ui.goal.loadStart"] = "Start with %d-%d kg"
L["ui.goal.loadStartEach"] = "Start with %d-%d kg each"
-- a set done with a weight wherever a record or a set shows (Sheet.LoadAmount): the kg, then the reps
L["ui.load.amount"] = "%s kg × %d"
L["ui.load.amountEach"] = "%s kg each × %d"
L["ui.target.record"] = "Your record: %s · last time: %s"
L["ui.target.recordLast"] = "Your record: %s, set last time"
L["ui.target.first"] = "Stop one rep before your form breaks."
L["ui.target.firstHold"] = "Stop when your form starts to break."
L["ui.target.firstHi"] = "Stop one rep before your form breaks. At %s you can stop: that starts you a step higher."
L["ui.target.firstHiHold"] = "Stop when your form starts to break. At %s you can stop: that starts you a step higher."
L["ui.target.form"] = "Your record: %s · your best lately: %s"
L["ui.target.repeat"] = "Your record: %s · done today already"
L["ui.target.placing"] = "Back from a break · your record: %s"
L["ui.target.cardio"] = "On the spot"
L["ui.target.allout"] = "All out: %d s build, then %d s as fast as you can."
L["ui.target.practice"] = "As many as you can."
-- a counted set (slice 42, 2026-10-16: a count in 30 s is never a goal): its goal in words with no number, its best
-- clean count under it once it has one, and its window (also the phones' counted line, contract/sheet.json's
-- words.target.count)
L["ui.goal.allout"] = "All out, clean reps"
L["ui.target.best"] = "Your best: %d clean reps"
L["ui.target.best.one"] = "Your best: %d clean rep"
L["ui.target.count"] = "%d s all out from the go."
L["ui.target.flow"] = "At your own pace"
L["ui.target.mark"] = "Goal pose: %s"
-- under it where the answer counts the family's day (slice 47, Panel.lua poseDaysLine): the days toward the step so
-- far (ns.PoseNeeds' have) of pose_unlock_days, Perfect days alone below a harder step; then what they still lack
-- (ladders.step.shortMain, .shortPerfect); the apps' pump.ready.mark.days, .perfect take the same words
L["ui.target.poseDays"] = "Days toward your step: %d of %d."
L["ui.target.posePerfect"] = "Perfect days toward your step: %d of %d."
-- under the goal while the climb waits on this move's stage of its path alone (Panel.lua nextStepLine, ns.PathDue,
-- 2026-10-16): what opens the next step, apart from this set's goal, and the days at it still needed; a pose's, Perfect
-- where only Perfect counts; a counted form's clean days (slice 42: its count is never a goal)
L["ui.target.next"] = "Your next step opens at %s on this move, on %d more days."
L["ui.target.next.one"] = "Your next step opens at %s on this move, on %d more day."
L["ui.target.nextPose"] = "Your next step opens once this pose is close enough on %d more days."
L["ui.target.nextPose.one"] = "Your next step opens once this pose is close enough on %d more day."
L["ui.target.nextPerfect"] = "Your next step opens once this pose is Perfect on %d more days."
L["ui.target.nextPerfect.one"] = "Your next step opens once this pose is Perfect on %d more day."
L["ui.target.nextClean"] = "Your next step needs %d more clean days on this move."
L["ui.target.nextClean.one"] = "Your next step needs %d more clean day on this move."
L["ui.target.carry"] = "With a load that makes the last 10 s hard"
L["ui.reason.owed"] = "Banked since %s"
L["ui.reason.stretch"] = "Since your last pump: %s yd on foot, %s yd mounted"

L["ui.running.build"] = "Build up"
L["ui.running.allout"] = "All out!"
L["ui.running.count"] = "All out, clean reps!"
L["ui.running.maxset"] = "Max set. %s"
L["ui.running.practice"] = "Practice: count your reps"
-- the virtual partner's bars on a hold (Panel.lua, slice 23)
L["ui.partner.partner"] = "Partner"
L["ui.partner.you"] = "You"
L["ui.partner.done"] = "Let go at %s"

L["ui.result.hint"] = "How many did you do?"
L["ui.result.holdHint"] = "How many seconds did you hold?"
-- a per-side move's question: the weaker side's count, never both added (also the watches' words.count.hint_side,
-- hold_hint_side)
L["ui.result.hintWeak"] = "How many on your weaker side?"
L["ui.result.holdHintWeak"] = "How many seconds on your weaker side?"
-- the count's view (Panel.lua renderResult, 2026-10-16): its question as the title, the move and its goal under it
L["ui.result.intro"] = "%s · %s"
-- a counted set's Result (slice 42, 2026-10-16): its question as the title (slice 53's "All out and clean?"; "Did
-- your form hold to the end?" until 2026-10-23), the window in its intro, the optional count's label and its words
-- with no number, the live preview of a count against the best clean count, and the three answers (a click each, the
-- digits typing the count on the dial since slice 66), none pushed
L["ui.result.formAsk"] = "All out and clean?"
L["ui.result.window"] = "%d s all out"
L["ui.result.cleanLabel"] = "Clean reps"
L["ui.result.cleanNone"] = "Not counted"
L["ui.result.cleanNew"] = "%d clean reps: new best"
L["ui.result.cleanNew.one"] = "%d clean rep: new best"
L["ui.result.cleanUnder"] = "%d clean reps · best %d"
L["ui.result.cleanUnder.one"] = "%d clean rep · best %d"
L["ui.result.cleanEnd"] = "Yes, to the end"
L["ui.result.slipped"] = "Form slipped"
L["ui.result.notAllOut"] = "Not all out"
L["ui.result.record"] = "Record"
L["ui.result.preview"] = "%d %s: +%d XP, %s"
L["ui.result.empty"] = "Done without a count"
L["ui.result.couldNot"] = "Could not do one"
L["ui.result.couldNotHold"] = "Could not hold it"
L["ui.result.confirm"] = "%s is far above your record of %s. Is that right?"
L["ui.result.confirmFirst"] = "%s is far above this step's top of %s. Is that right?"
L["ui.result.confirmLow"] = "%s is far below your record of %s. Is that right?"
-- a counted set's count far above its best clean count (slice 42): the count and the best
L["ui.result.confirmBest"] = "%d clean reps is far above your best of %d. Is that right?"
L["ui.result.confirmBest.one"] = "%d clean rep is far above your best of %d. Is that right?"
L["ui.result.reallyYes"] = "Yes, record %s"
L["ui.result.reallyKeep"] = "Yes, keep %s"
L["ui.result.reallyChange"] = "Change it"
-- a timed move stopped before half its time (Lc.Whole): the time it ran of the whole move's, then the question
L["ui.result.wholeClock"] = "You stopped at %s of %s."
L["ui.result.wholeAsk"] = "Did you do the whole move?"
L["ui.result.wholeYes"] = "Yes, all of it"
L["ui.result.wholeNo"] = "No, I stopped"
-- the hold ask (slice 52) after a fresh hold set near its goal on a hold ladder's step (ns.HoldAsk): its question, the
-- view's title, and its three answers in this order (ns.Enum.longer's codes, keys 1-3, each in ui.result.markChoice's
-- "%d. %s"), which the Stamina Ledger reads as what the player had left
L["ui.result.longerAsk"] = "Could you have held longer?"
L["ui.result.longerNo"] = "No"
L["ui.result.longerLittle"] = "A little"
L["ui.result.longerLot"] = "A lot"
-- a pose that sets its family's mark, which the player reads as its goal pose (2026-10-14, David: "the 'mark' (please
-- rename for clarity)"), ran its time (Lc.Mark, 2026-10-09): the gold label over the goal pose's sentence, the
-- question and the four answers, their words alone as the apps' and the watches' (each button its key first, the
-- answer's code: "4. Perfect" down to "1. Not yet"). The move's XP is the same whatever the answer (ns.MoveXP).
L["ui.result.goalPose"] = "Goal pose"
L["ui.result.markAsk"] = "How close to the goal pose?"
L["ui.result.markChoice"] = "%d. %s"
L["ui.result.markPerfect"] = "Perfect"
L["ui.result.markGood"] = "Good"
L["ui.result.markOkay"] = "Okay"
L["ui.result.markNotYet"] = "Not yet"
-- the pace ask (slice 42, 2026-10-17; three answers since slice 52) after a steady move that ran its time, where the
-- steady ledger asks (ns.LedgerAsk): its question, the view's title and the voice's line as the ask opens (ask.pace);
-- its three answers in this order (ns.Enum.paceAnswers, keys 1-3, each in ui.result.markChoice's "%d. %s"), the two
-- Yes answers in Yes, the whole time's place
L["ui.result.paceAsk"] = "Did you keep that pace to the end?"
L["ui.result.paceEasily"] = "Yes, easily"
L["ui.result.paceJust"] = "Yes, just"
L["ui.result.paceEased"] = "I eased off"
-- after a check's answer (the recorded view, under its outcome): passed, with what it opened (only where that is open
-- after the set, ns.OpensNow; else that it is not open yet, over the next wait's own words: 2026-10-17, slice 42's
-- review) and the family's lift where it moved up; not passed; and what a cardio set brought (a clean window's day, a
-- full move: slice 52, a kept pace's ready day until then)
L["ui.result.checkPassed"] = "Check passed: %s unlocked"
L["ui.result.checkClimb"] = "Check passed: %s can climb to step %d"
L["ui.result.checkHeld"] = "Check passed. %s is not open yet."
L["ui.result.checkClimbHeld"] = "Check passed. The climb to %s step %d is not open yet."
L["ui.result.checkPlaced"] = "%s moves up to step %d"
L["ui.result.checkAgain"] = "Not there yet."
L["ui.result.cleanDay"] = "Clean day %d of %d toward %s step %d."
L["ui.result.fullMove"] = "Full move %d of %d toward %s step %d."
-- slice 53: a proving window of the step above that did not pass, and a cardio step's measure back from a break that
-- holds it or takes it a step down
L["ui.result.climbWaits"] = "No step up yet."
L["ui.result.stepHolds"] = "Still on step %d."
L["ui.result.stepBack"] = "Back to step %d."
-- a Long Haul's day with a Yes at its length (slice 55): the time held, the days, the length they open
L["ui.result.haulDay"] = "Long Haul %s: day %d of %d toward %s."
L["ui.result.next"] = "Next"
L["ui.result.finish"] = "Finish"
-- the last move's way out of a pump started from the Talents tab, which returns to the tree (the apps'
-- pump.done.back.tree)
L["ui.result.backTree"] = "Back to your tree"
-- every move's pour (slices 65 and 66; the apps' pump.saved.*): the way on, live from the pour's start; the count bar's
-- label (the ladder and the move's step) and its posts in the move's unit (reps as a number, seconds as m:ss); the
-- record chip on the dial and the move row's tag with the record it passed; the day at the top it counted; the step-up
-- reveal's caption; the unlock row's step and colour (its stat and XP rows: ui.rewards.stat and ui.rewards.gain)
L["ui.pour.next"] = "Next: %s"
L["ui.pour.step"] = "%s · step %d"
L["ui.pour.goal"] = "goal %s"
L["ui.pour.best"] = "best %s"
L["ui.pour.top"] = "top %s"
L["ui.pour.record"] = "New record"
L["ui.pour.was"] = "was %s"
L["ui.pour.topDay"] = "Day %d of %d at the top, toward step %d"
L["ui.pour.unlocked"] = "Step %d unlocked"
L["ui.pour.unlockStep"] = "step %d · %s"
L["ui.result.max"] = "Max %d: count one set"
-- under the count's dial while + stands at its most (slice 66, the apps' pump.count.cap and pump.count.cap.goal): that
-- most and what the app counted, or the number a set of reps has to beat where that set it, in the move's unit
-- (Panel.lua Pour.drawCount, Events.lua Lc.CountMost)
L["ui.count.cap"] = "Up to %s: 50%% more than the %s counted"
L["ui.count.cap.goal"] = "Up to %s: 50%% more than the %s to beat"
-- the weight box under the count's dial on a move done with a weight with its - and + (the dial's own too), and after
-- the set the next fresh set's weight and goal (ns.NextLoad)
L["ui.result.kg"] = "kg"
L["ui.result.kgEach"] = "kg each"
L["ui.result.kgDown"] = "-"
L["ui.result.kgUp"] = "+"
-- under the weight box while it holds a weight: empties it, so the set counts its reps alone
L["ui.result.kgNone"] = "Don't know the weight"
L["ui.result.maxKg"] = "Max %d kg"
L["ui.result.confirmKg"] = "%s kg is far above the %s kg planned. Is that right?"
L["ui.result.nextLoad"] = "Next time: %s kg, %d+ reps"
L["ui.result.nextLoadEach"] = "Next time: %s kg each, %d+ reps"
L["ui.result.finished"] = "Finished: %d s · +%d XP"
L["ui.result.stoppedAtXP"] = "Stopped at %d s · +%d XP"
L["ui.result.practiceDone"] = "Practice done: %d. Nothing is stored."
L["ui.unit.reps"] = "reps"
L["ui.unit.seconds"] = "s"
L["ui.unit.repsSide"] = "reps each side"
L["ui.unit.secondsSide"] = "s each side"

L["outcome.beat"] = "new record"
L["outcome.miss"] = "the record stands"
L["outcome.placement"] = "your first record"
L["outcome.placed"] = "placed, your record stays"
L["outcome.censored"] = "a long set, counted as at least this"
L["outcome.done"] = "done without a count"
L["outcome.stopped"] = "stopped early"
L["outcome.offered"] = "offered"
L["outcome.owed"] = "banked"
L["outcome.banked"] = "banked"
L["outcome.paid"] = "paid"
L["outcome.lastbeat"] = "more than last time"
L["outcome.goal"] = "goal reached"
L["outcome.enough"] = "enough for the next step"

-- a %s count is a number of reps or a hold's seconds ("45 s"), as Sheet.Amount writes it
L["ui.outcome.beat"] = "New record: %s! +%d XP"
L["ui.outcome.placement"] = "First record: %s. +%d XP"
L["ui.outcome.placed"] = "Your step: %s. Your record of %s stands. +%d XP"
L["ui.outcome.censored"] = "%s in a long set, counted as at least that. +%d XP"
L["ui.outcome.goal"] = "Goal reached: %s. Record %s stands. +%d XP"
L["ui.outcome.lastbeat"] = "Up %s on last time. Record %s stands. +%d XP"
L["ui.outcome.lastbeatLoad"] = "%s, more than last time. Record %s stands. +%d XP"
L["ui.outcome.miss"] = "%s. Record %s stands. +%d XP"
-- a counted set's (slice 42): a clean count over the best (or the first), one under it, none typed, Form slipped, Not
-- all out (slice 53)
L["ui.outcome.bestNew"] = "New best: %d clean reps! +%d XP"
L["ui.outcome.bestNew.one"] = "New best: %d clean rep! +%d XP"
L["ui.outcome.clean"] = "Clean to the end: %d. Best %d stands. +%d XP"
L["ui.outcome.cleanNone"] = "Clean to the end, not counted. +%d XP"
L["ui.outcome.slipped"] = "Form slipped: saved, no clean day. +%d XP"
L["ui.outcome.notAllOut"] = "Not all out: saved, no clean day. +%d XP"
L["ui.outcome.done"] = "Done. +%d XP"
L["ui.outcome.enough"] = "%s: enough for the next step! +%d XP"
L["ui.outcome.couldNot"] = "Noted: an easier start next time. +%d XP"

L["ui.joint.title"] = "Where does it hurt?"
-- The voice that cannot be heard (Speech.lua): once a session, with the game's setting that silences it
L["speech.silent"] = "QuestPump's voice can't be heard: %s."
L["speech.silent.allOff"] = "the game's sound is off (Options > Audio > Enable Sound)"
L["speech.silent.masterZero"] = "the master volume is 0 (Options > Audio)"
L["speech.silent.dialogOff"] = "the game's Dialog sound is off (Options > Audio > Dialog)"
L["speech.silent.dialogZero"] = "the Dialog volume is 0 (Options > Audio > Dialog)"
L["cmd.status.voice"] = "Voice: silent, %s."
-- /qp log (Log.lua): the log's lines are English diagnostics, printed as they are saved
L["cmd.log.help"] = "/qp log [n]: the last lines of QuestPump's log (saved to disk too); /qp log clear empties it"
L["cmd.log.header"] = "The last %d of %d log lines:"
L["cmd.log.cleared"] = "The log is empty now."
L["ui.joint.confirm"] = "Confirm"
-- a joint the check-in shows flagged already: ticked and greyed, cleared only on the Body page (Panel.lua)
L["ui.joint.flagged"] = "Already flagged. Clear it on the Body page."
L["joint.wrist"] = "Wrist"
L["joint.elbow"] = "Elbow"
L["joint.knee"] = "Knee"
L["joint.achilles"] = "Achilles"
L["joint.back"] = "Low back"
L["joint.shoulder"] = "Shoulder"
L["joint.neck"] = "Neck"
L["joint.hip"] = "Hip"

-- the phone apps' felt-unwell view (contract/sheet.json)
L["ui.unwell.title"] = "Felt unwell"

-- The non-modal card
L["ui.card.stop"] = "Stop to pump"
L["ui.card.moves"] = "%d moves"
L["ui.card.moves.one"] = "1 move"

-- The quest families and themes, for /qp why
L["family.kill"] = "Kill"
L["family.gather"] = "Gather"
L["family.travel"] = "Travel"
L["family.talk"] = "Talk"
L["family.dungeon"] = "Dungeon"
L["family.pvp"] = "PvP"
L["family.repeatable"] = "Repeatable"
L["family.boss"] = "Boss"
L["family.escort"] = "Escort"
L["family.profession"] = "Profession"
L["family.fishing"] = "Fishing"
-- a quest's flavour line, in quotes as item flavour text (the lines themselves are the quest table's, enUS only)
L["ui.flavour"] = "\"%s\""
-- the theme tags (Library.lua ns.THEMES), named in /qp why
L["theme.bear"] = "Bear"
L["theme.crab"] = "Crab"
L["theme.frog"] = "Frog"
L["theme.duck"] = "Duck"
L["theme.worm"] = "Worm"
L["theme.spider"] = "Spider"
L["theme.axe"] = "Axe"
L["theme.pick"] = "Pick"
L["theme.oar"] = "Oar"
L["theme.bow"] = "Bow"
L["theme.hammer"] = "Hammer"
L["theme.climb"] = "Climb"
L["theme.swim"] = "Swim"
L["theme.carry"] = "Carry"
L["theme.dig"] = "Dig"
L["theme.run"] = "Run"
L["theme.sneak"] = "Sneak"
L["theme.fight"] = "Fight"
L["theme.ride"] = "Ride"

-- The setup window (Panel.lua; its pages' look since 2026-10-15, the D boards: the question as each page's title, a
-- name under each picture, the details in tooltips; no line under a title or a name since 2026-10-22): the helper and
-- the phone app, strongly recommended (the companions' page's title); no URL, QR code, price, store, Pro or coach
L["ui.setup.title"] = "QuestPump"
L["ui.setup.welcome"] = "Strongly recommended companions"
-- while the setup asks for no phone (Setup.AsksPhone: the phone apps are not in the stores yet), the page is the
-- helper's alone: its title, its card's tooltip, the chips' head and tooltip and the way out in the singular
L["ui.setup.welcome.one"] = "Strongly recommended companion"
L["ui.setup.step1"] = "QuestPump on this computer"
L["ui.setup.step1.tip"] = "Download it from the addon's page, where you got QuestPump. Open it, and choose Pair your "
	.. "phone there for the code your phone scans. Check again reloads the interface and ticks off what is set up."
L["ui.setup.step1.tip.one"] = "Download it from the addon's page, where you got QuestPump, and open it. Check again "
	.. "reloads the interface and ticks off what is set up."
-- the helper's state by ns.HelperState (none: no helper ever wrote the pairing block, so its word is where to get it),
-- its card's word under its name
L["ui.setup.step1.state.none"] = "Get it on the addon's page"
L["ui.setup.step1.state.on"] = "Running"
L["ui.setup.step1.state.off"] = "Not running"
L["ui.setup.step1.state.blind"] = "Cannot see the game"
L["ui.setup.step1.state.stopped"] = "Cannot read the game"
L["ui.setup.step1.state.disabled"] = "Turned off"
L["ui.setup.step1.state.outdated"] = "Out of date"
-- the helper's card's tooltip once it runs (ns.HelperState "on": its sign of life and its last word on the screen
-- reading, as this load read them; whether it reads the squares right now only its own window can say, 2026-10-10)
L["ui.setup.step1.done"] = "It runs beside the game while you play."
-- the remedy under the helper's state when a helper wrote the pairing block before, by ns.HelperState
L["ui.setup.step1.off"] = "Open QuestPump on this computer, then press Check again."
L["ui.setup.step1.blind"] = "In its window, press Allow screen reading, turn it on, restart it, then press Check "
	.. "again."
L["ui.setup.step1.stopped"] = "Open its window, then press Check again."
-- the remedy when its QuestPump Mirror folder is installed but did not load (ns.HelperState)
L["ui.setup.step1.disabled"] = "Turn QuestPump Mirror on in the AddOns list, then press Check again."
L["ui.setup.step1.outdated"] = "Open QuestPump on this computer so it updates QuestPump Mirror, then press Check again."
L["ui.setup.step2"] = "The phone app"
L["ui.setup.step2.tip"] = "Scan the code with your phone's camera: it opens the app, or shows where to get it. Then "
	.. "press Check again."
L["ui.setup.step2.state.none"] = "Pair it in QuestPump on this computer"
L["ui.setup.step2.state.done"] = "Paired"
-- a pairing whose phone has not reached the helper for C.phone_quiet_days (ns.PhoneQuiet): the day it last did, the
-- remedy under it
L["ui.setup.step2.state.quiet"] = "Not heard from lately"
-- the phone's card's tooltip once it is paired
L["ui.setup.step2.done"] = "Pumps from the game come to your phone while you play."
L["ui.setup.step2.quiet"] = "It last connected on %s. Open the app, or pair it again."
-- what they give (David, 2026-10-09: the saved progress first on a client that does not read its save back, the
-- phone's pumps and streak elsewhere):
-- four chips, the whole of it in their tooltip
L["ui.setup.with"] = "With them"
L["ui.setup.with.one"] = "With it"
L["ui.setup.chip.kept"] = "Keeps your progress"
L["ui.setup.chip.streak"] = "Your streak on days without the game"
L["ui.setup.chip.wrist"] = "Countdown on your wrist"
L["ui.setup.chip.heart"] = "Heart-rate check marks"
L["ui.setup.chip.catchup"] = "Catch up from the phone"
L["ui.setup.gives"] = "With them you can do pumps and banked moves on your phone, away from the game. A pump "
	.. "there keeps your streak going on days without the game, and a daily reminder asks you back. Your sheet shows "
	.. "on your phone, the countdown comes to your wrist, and your pumps get heart-rate check marks."
L["ui.setup.gives.forever"] = "They keep your progress saved when the game does not load it back. "
	.. "With them you can also do pumps on your phone, keep your streak going on days without the game, count down on "
	.. "your wrist and get heart-rate check marks."
L["ui.setup.gives.one"] = "With it you can do pumps and banked moves on this computer, away from the game. A pump "
	.. "there keeps your streak going on days without the game, and with a heart-rate strap your pumps get heart-rate "
	.. "check marks."
L["ui.setup.gives.one.forever"] = "It keeps your progress saved when the game does not load it back. "
	.. "With it you can also do pumps on this computer, away from the game, keep your streak going on days without the "
	.. "game and get heart-rate check marks with a heart-rate strap."
L["ui.setup.check"] = "Check again"
L["ui.setup.check.tip"] = "Reloads the interface, as /reload does, and ticks off what is set up."
-- the way out of the companions' page, named by what the player goes without: both; on a client that showed it does
-- not read its save back (Setup.SaveUnread) while no helper runs, the saved progress (the legal minimum's consequence);
-- with the helper running, the phone; on the helper's page alone (Setup.AsksPhone), the helper
L["ui.setup.without"] = "Play without them"
L["ui.setup.without.forever"] = "Play without saving"
L["ui.setup.without.helper"] = "Play without the phone"
L["ui.setup.without.one"] = "Play without it"
-- On demand before the start while both companions are set up: only Your setup is left (Lc.Demand)
L["ui.setup.first.yours"] = "First your setup: answer the few questions in the setup window, or close it to start."
-- Your setup, the setup window's pages after the companions' (Panel.lua; David, 2026-10-09): where you play (the room's
-- three tiles, the Body page's tip in its tooltip; the pool they leave), what you have at hand, and how many moves per
-- quest (each pace's name with its count of moves) with the voice's count
L["ui.setup.place"] = "Where do you play?"
L["ui.setup.room"] = "Your room"
L["ui.setup.pool"] = "moves in your pool"
L["ui.setup.gear"] = "What do you have at hand?"
-- a piece's name on its tile where gear.<key>'s explanation would not fit two lines of a third of the page (its tooltip
-- keeps the whole name and what counts as it); every other tile shows gear.<key>
L["ui.setup.gearName.stable_chair"] = "Stable chair"
L["ui.setup.gearName.fixed_bar"] = "Fixed pull-up bar"
L["ui.setup.gearName.parallettes"] = "Parallettes or books"
L["ui.setup.gearName.backpack"] = "Loaded backpack"
L["ui.setup.gearName.pole"] = "Outdoor pole"
L["ui.setup.hard"] = "How many moves per quest?"
L["ui.setup.pace.light"] = "Light: 1 move"
L["ui.setup.pace.balanced"] = "Balanced: 1 to 3 moves"
L["ui.setup.pace.extreme"] = "Extreme: 2 to 4 moves"
L["ui.setup.back"] = "Back"
L["ui.setup.next"] = "Next"
L["ui.setup.start"] = "Start"
L["ui.setup.done"] = "Done"

-- The experience question (Your setup's How hard, and the phone apps'; tools/gen-fixtures.lua's vocab)
L["experience"] = "Strength training so far"
L["experience.none"] = "None"
L["experience.some"] = "Some"
L["experience.training"] = "Training now"

-- The day-two check-in (local only)
L["ui.checkin.title"] = "QuestPump"
L["ui.checkin.question"] = "Muscles today?"
L["ui.checkin.fine"] = "Fine"
L["ui.checkin.sore"] = "Sore"
L["ui.checkin.dark"] = "Very sore, or dark urine"

-- The ladders, the first records and the result toasts
L["ladder.1"] = "Push"
L["ladder.2"] = "Squat"
L["ladder.4"] = "Hinge"
L["ladder.6"] = "Cardio"
L["ladder.7"] = "Wall sit"
L["ladder.8"] = "Pull-up"
L["ladder.9"] = "Row"
L["ladder.10"] = "Core"
L["ladder.11"] = "Side core"
L["ladder.12"] = "Front split"
L["ladder.13"] = "Middle split"
L["ladder.14"] = "Pike"
L["ladder.15"] = "Backbend"
L["ladder.16"] = "Twist"
L["ladder.17"] = "Ankles and squat"
L["ladder.18"] = "Balance"
L["ladder.19"] = "Arm balance"
L["ladder.20"] = "Handstand"
L["ladder.21"] = "L-sit"
L["ladder.22"] = "Planche"
L["ladder.23"] = "Power"
L["ladder.24"] = "Sit-up"
-- The three talent trees (Core.lua ns.TREES, slice 36), named as the focus is; contract/sheet.json carries them to the
-- apps as the ladders' names
L["tree.strength"] = "Strength"
L["tree.endurance"] = "Endurance"
L["tree.flexibility"] = "Flexibility"
-- The Talents tab (Tree.lua, slice 37): the game's copy of the trees' words, docs/ui.md "The words" (its one source:
-- "Click" for the pointer, the pain line's link "Body", where the game's pain flags are)
L["tree.step"] = "%s · step %d · %s"
L["tree.standing.owned"] = "reached"
L["tree.standing.now"] = "your step"
L["tree.standing.unplaced"] = "not placed yet"
L["tree.standing.next"] = "next"
L["tree.standing.ahead"] = "ahead"
L["tree.standing.kept"] = "reached before"
L["tree.waits.closed"] = "Not open yet"
L["tree.waits.setup"] = "Off in Your setup"
L["tree.waits.off"] = "Left out of your pumps"
L["tree.days"] = "%d days"
L["tree.days.one"] = "%d day"
L["tree.next.climb"] = "Next: %s %s on %s."
-- the climb whose days are in waits for a stage of its path (slice 42): its move and its top, the apps'
-- ladders.tree.next.proof
L["tree.next.proof"] = "Next: %s %s."
-- the stage whose move is not open yet (2026-10-19, ns.PathNext's wait): its move, then what holds it as a whole
-- sentence with its own full stop, which this line adds none after (the double-check's fix review: "Opens after
-- Bulgarian split squat at 25 a side.", tree.held.*, or "Waits for a record of 12 on Feet-up push-up.",
-- ladders.step.waits; the apps' ladders.tree.next.proof.held)
L["tree.next.pathHeld"] = "Next: %s. %s"
L["tree.next.earn"] = "Next: one fresh set of %s %s earns it."
-- a pose family's orange or gold step still to earn: its standing pose, held and answered Perfect (the apps'
-- ladders.tree.next.earn.mark)
L["tree.next.earnMark"] = "Next: one Perfect hold of %s earns it."
-- a pose family's climb (slice 47: any goal pose of its step counts a day; its standing pose by name until then): its
-- ladder and the days (the apps' ladders.step.next.mark, .perfect)
L["tree.next.pose"] = "Next: any goal pose of your %s step close enough on %s."
L["tree.next.perfect"] = "Next: any goal pose of your %s step Perfect on %s."
L["tree.next.proofPose"] = "Next: %s, close enough to its goal pose on %s."
L["tree.next.proofPerfect"] = "Next: %s, Perfect on %s."
-- the cardio ladder's (2026-10-16, slice 42): a timed step's full moves (slice 52: the 45-s move kept with a Yes; ready
-- days, a kept pace, until then), a counted step's clean days and the form its path waits for with its own clean days
-- (the apps' ladders.step.next and .clean, ladders.tree.next.proof.clean)
L["tree.next.full"] = "Next: %d of %d full moves."
L["tree.next.clean"] = "Next: %d of %d clean days."
L["tree.next.cleanForm"] = "Next: %s: %d of %d clean days."
L["tree.next.cleanForm.one"] = "Next: %s: %d of %d clean day."
-- a counted step whose days and path are in (slice 53): the proving window, one clean window of the step above's form
-- (ns.Proving)
L["tree.next.proving"] = "Next: %s, a clean 30-s all-out set."
L["tree.next.trial"] = "Next: your first set of %s finds your step."
L["tree.next.rest"] = "Rest today. Pumps come back at %s."
L["tree.next.day"] = "Today already counts on every ladder here. The next day counts from %s."
L["tree.next.pain"] = "Paused: you marked pain for every climb here."
L["tree.next.break"] = "Back from a break: your first set on each ladder finds your step again."
L["tree.next.none"] = "Nothing to climb here right now: each ladder is at its top or waiting."
L["tree.key.record"] = "Record %s · Range %s-%s"
L["tree.key.left"] = "%s to go, then %s"
-- the "of %d days" lines take their one form on a climb back's one day (ns.UnlockDays), by the day count
L["tree.key.daysTop"] = "%d of %d days at the top"
L["tree.key.daysTop.one"] = "%d of %d day at the top"
-- the player's step with its days in and a stage of its path still out (slice 42): that stage's move at its top, a
-- pose's close enough or Perfect on its days, a cardio form's clean days (the apps' ladders.measure.proof,
-- .proof.mark, .proof.mark.perfect, .proof.clean)
L["tree.key.path"] = "Next needed: %s %s"
-- held (2026-10-19): the stage's move, then what holds it as tree.next.pathHeld's sentence (the apps'
-- ladders.measure.proof.held)
L["tree.key.pathHeld"] = "Then %s. %s"
L["tree.key.pathPose"] = "Then %s, close enough on %s"
L["tree.key.pathPerfect"] = "Then %s, Perfect on %s"
L["tree.key.pathClean"] = "Then %s: %d of %d clean days"
L["tree.key.pathClean.one"] = "Then %s: %d of %d clean day"
L["tree.key.mark"] = "Your first max set finds your step"
L["tree.key.held"] = "%s · %s"
L["tree.key.clean"] = "%d of %d clean days"
L["tree.key.full"] = "%d of %d full moves"
L["tree.key.pose"] = "%d of %d days"
L["tree.key.pose.one"] = "%d of %d day"
L["tree.key.perfect"] = "%d of %d Perfect days"
L["tree.key.perfect.one"] = "%d of %d Perfect day"
L["tree.key.unplaced"] = "Your first max set finds your step"
L["tree.key.placing"] = "Your next set finds your step again"
-- the cardio ladder's, by what measures its step back from a break (ns.TreeEntry's placingBy: its all-out window from
-- closing_rung, a steady move below it; the apps' ladders.measure.placing.window and .steady)
L["tree.key.placingWindow"] = "Your next 30-s all-out set finds your step again"
L["tree.key.placingSteady"] = "Your next steady move finds your step again"
L["tree.key.next"] = "Opens at %s %s on %s"
-- a pose step's: any goal pose of the step below close enough on its days (slice 47; the apps'
-- ladders.measure.next.mark, .perfect, and .kept.mark, .perfect for the "Comes back" forms)
L["tree.key.nextPose"] = "Opens after any goal pose is close enough on %s"
L["tree.key.nextPerfect"] = "Opens after any goal pose is Perfect on %d days"
L["tree.key.nextPerfect.one"] = "Opens after any goal pose is Perfect on %d day"
L["tree.key.nextFull"] = "Opens at %d full moves"
L["tree.key.nextClean"] = "Opens at %d clean days"
-- a climb with a path: the moves of its stages not done yet after the main move's line (the apps'
-- ladders.measure.next.path)
L["tree.key.andMore"] = "%s and %d more moves"
L["tree.key.andMore.one"] = "%s and %d more move"
L["tree.key.ahead"] = "Opens after %s"
L["tree.key.back"] = "Win it back: %s %s on %s"
L["tree.key.backPose"] = "Comes back after any goal pose is close enough on %s"
L["tree.key.backPerfect"] = "Comes back after any goal pose is Perfect on %d days"
L["tree.key.backPerfect.one"] = "Comes back after any goal pose is Perfect on %d day"
L["tree.key.backFull"] = "Comes back at %d full moves"
L["tree.key.backClean"] = "Comes back at %d clean days"
L["tree.key.backAfter"] = "Comes back after %s"
L["tree.opensRecord"] = "%s opens at a %s record of %s"
L["tree.opensStep"] = "%s opens at %s step %d"
L["tree.opens.record"] = "%s opens at a %s record of %s."
L["tree.opens.step"] = "%s opens at %s step %d."
L["tree.click.start"] = "Click: its card and Start"
L["tree.click.card"] = "Click: its card"
-- on the step whose card shows, which a second click closes (2026-10-15)
L["tree.click.close"] = "Click: close its card"
L["tree.subject"] = "Step %d"
L["tree.subject.ladder"] = "%s step %d"
L["tree.needs.gear"] = "%s needs %s"
L["tree.needs.on"] = "%s needs %s on"
L["tree.needs.off"] = "%s needs %s off"
L["tree.off"] = "You left %s out of your pumps"
L["tree.offToo"] = "You left %s out of your pumps too"
L["tree.pain"] = "Paused: you marked pain for this step"
L["tree.link.line"] = "%s:"
L["tree.link.setup"] = "Your setup"
L["tree.link.include"] = "Include in pumps"
L["tree.link.body"] = "Body"
L["tree.start.own"] = "Start %s"
L["tree.start.climb"] = "Climb with %s"
L["tree.start.back"] = "Climb back with %s"
L["tree.start.earn"] = "Earn %s"
L["tree.start.place"] = "Find my step"
L["tree.start.goal"] = "%s · %s"
L["tree.start.max"] = "%s · Max set"
L["tree.start.allout"] = "%s · all out"
-- a stretch check's Start and the card's check row's button (slice 42; the apps' ladders.start.check): no move and no
-- number, the tile beside it naming the pose
L["tree.start.check"] = "Do the check"
L["tree.start.waiting"] = "Open your waiting pump"
-- the Long Haul beside the cardio step's Start (slice 55): one steady move at its open length
L["tree.haul"] = "Long Haul %s"
L["tree.card.earn"] = "One fresh set of %s earns this step's colour."
L["tree.card.earnPose"] = "A Perfect hold earns it."
L["tree.card.earnLater"] = "Fresh again at %s. Then one set of %s earns its colour."
L["tree.card.earnLaterPose"] = "From %s, a Perfect hold earns it."
L["tree.card.session"] = "Today already counts toward the next step."
L["tree.card.day"] = "Your next set counts toward the climb from %s."
L["tree.card.fresh"] = "Fresh again at %s"
L["tree.card.ahead"] = "Ahead of your step for now"
-- what the player can hold now on a hold ladder's step (slice 52, the Stamina Ledger's estimate and its band:
-- ns.HoldNow), in seconds; power's in reps
L["tree.card.nowHold"] = "You can do now %d-%d s"
L["tree.card.nowReps"] = "You can do now %d-%d"
L["ladders.card.range"] = "Range %s-%s"
-- a cardio step's measure (slice 42; the apps' ladders.card.clean, .best): a counted step's clean days and its window,
-- the shown form's best clean count; a timed step's rows (slice 52, Tree.CardPair: its ready days' line until then),
-- Pace, the seconds its forms keep their pace now of the full move's, and Full moves, those the climb counts of the
-- ones it takes
L["tree.card.clean"] = "%d clean days of %d s all out"
L["tree.card.best"] = "Best %d clean reps"
L["tree.card.best.one"] = "Best %d clean rep"
L["tree.card.pace"] = "Length now"
L["tree.card.paceValue"] = "%d of %d s"
L["tree.card.full"] = "Full moves"
L["tree.card.fullValue"] = "%d of %d"
L["tree.card.placing"] = "Your next set finds your step again."
L["tree.card.placingWindow"] = "Your next 30-s all-out set finds your step again."
L["tree.card.placingSteady"] = "Your next steady move finds your step again."
L["tree.card.toGetHere"] = "To get here"
-- the same row on the player's own step whose Start runs another move (its proof, an easier move, a placement): the
-- apps' ladders.card.foot.own
L["tree.card.trainWith"] = "Train with"
L["tree.card.backToStep"] = "Back to the step"
-- the step's moves in three groups (slice 42, docs/ui.md "The steps' order"; the apps' ladders.card.group.*): their
-- caps labels (the third named for what it is for, with no line under it: 2026-10-22), its fold, and the main move
-- standing in for the anchor under the setup
L["tree.card.group.main"] = "Your main move"
L["tree.card.group.then"] = "Then, for the next step"
L["tree.card.group.same"] = "For variety"
L["tree.card.same.more"] = "For variety · %d moves"
L["tree.card.same.more.one"] = "For variety · %d move"
L["tree.card.standin"] = "%s replaces %s for your equipment and settings."
-- a stage of the path (the apps' ladders.path.need, ladders.card.stage.*): a strength or hold move at its top on its
-- days (two alternatives each at its own top, 2026-10-17), a pose close enough or Perfect on its days, a cardio form's
-- clean days (a done stage's days whole, with no count: 2026-10-17), two alternatives joined; a waiting stage's line,
-- its top "a side" on a side move; a skipped stage's state (the row's name names the move)
L["tree.stage.need"] = "%s %s on %s"
L["tree.stage.needOr"] = "%s %s or %s %s on %s"
L["tree.stage.mark"] = "%s, close enough on %s"
L["tree.stage.perfect"] = "%s, Perfect on %s"
L["tree.stage.clean"] = "%s: %d of %d clean days"
L["tree.stage.clean.one"] = "%s: %d of %d clean day"
L["tree.stage.cleanDone"] = "%s: clean on %s"
L["tree.stage.or"] = "%s or %s"
L["tree.stage.opens"] = "Opens after %s at %s"
L["tree.stage.side"] = "%s a side"
L["tree.stage.opensMark"] = "Opens once %s is rated Good"
L["tree.stage.opensPerfect"] = "Opens after your first Perfect day of %s"
-- what holds a held line's move as a whole sentence (tree.next.pathHeld, tree.key.pathHeld; 2026-10-19, the
-- double-check's fix review): its stage's opener, as tree.stage.opens and its Good and Perfect forms say it, or its
-- own stretch check, as tree.check.need's row says it (the apps' ladders.held.opens, .opens.mark, .opens.perfect,
-- ladders.held.check and .check.perfect)
L["tree.held.opens"] = "Opens after %s at %s."
L["tree.held.opensMark"] = "Opens once %s is rated Good."
L["tree.held.opensPerfect"] = "Opens after your first Perfect day of %s."
L["tree.held.check"] = "Opens after your first Good hold of %s."
L["tree.held.checkPerfect"] = "Opens after your first Perfect hold of %s."
L["tree.stage.skipSetup"] = "Left out by your setup"
L["tree.stage.skipOff"] = "You left it out"
-- an `any` need: its alternatives' chip words joined (the apps' ladders.card.need.or)
L["tree.need.or"] = "%s, or %s"
-- a stretch check (the apps' ladders.card.check*): its caps label, its row, its line while undone (what it opens, a
-- move or the climb to a step; Perfect where only Perfect passes), its line once done
L["tree.card.check"] = "Stretch check"
L["tree.check.need"] = "%s: Good or better, once"
L["tree.check.needPerfect"] = "%s: Perfect, once"
L["tree.check.line"] = "Hold this pose once at Good or better to unlock %s."
L["tree.check.linePerfect"] = "One Perfect hold at its goal pose opens %s."
L["tree.check.lineStep"] = "One hold at its goal pose opens the climb to step %d."
L["tree.check.lineStepPerfect"] = "One Perfect hold at its goal pose opens the climb to step %d."
L["tree.check.done"] = "Passed %s"
L["tree.check.doneStep"] = "Passed: your %s is at step %d."
-- the hint row: a soft need, the pose that helps the step's move (Library.lua hint; the apps' ladders.card.hint); the
-- move's own advice (move.<key>.hint) follows its How on the step card and ends the pump card's tips (2026-10-22)
L["tree.card.hint"] = "Helps: %s"
-- Mastered (slice 42; the apps' ladders.step.mastered, .mastery, ladders.tree.mastered): the card's line on a step the
-- player has, the hover's line 2 after the standing
L["tree.card.mastered"] = "Mastered %s"
L["tree.card.mastery"] = "Toward Mastered: %d of %d moves at their top"
L["tree.card.mastery.one"] = "Toward Mastered: %d of %d move at its top"
L["tree.mastered"] = "mastered"
L["tree.card.opens"] = "Reaching this step unlocks"
L["tree.card.top"] = "This ladder ends at %s, step %d."
-- the card's quiet part folded under the kit's More bar (the apps' ladders.card.more)
L["tree.card.more"] = "More on this step"
L["tree.card.skill"] = "Skill: %s (%s)"
L["tree.chip"] = "%s · step %d · %s"
L["tree.expand"] = "Show the %d steps ahead"
L["tree.expand.one"] = "Show the step ahead"
L["tree.search"] = "Find a move"
L["tree.search.count"] = "%d steps match"
L["tree.search.count.one"] = "%d step matches"
-- why a step's Start cannot run now (Lc.StartBlock, Lc.StartMove), in Tree.BlockLine's one wording: on its card under
-- the greyed Start and in its tooltip, and on the card and in chat at a click a passing state refused; felt unwell
-- reads the trees' rest line (tree.next.rest)
L["tree.block.setup"] = "First finish the setup"
L["tree.block.combat"] = "Not in combat."
L["tree.block.pump"] = "Finish the pump that waits first."
L["tree.block.pvp"] = "No pumps in a battleground or arena."
L["tree.block.dead"] = "Not while you are dead."
L["tree.block.swimming"] = "Not while you swim. Try again on land."
L["tree.block.falling"] = "Not while you fall. Try again once you land."
L["tree.block.vehicle"] = "Not in a vehicle. Try again once you leave it."
L["tree.block.cinematic"] = "Not during a scene. Try again after it."
L["tree.block.breath"] = "Not while the breath bar shows. Try again after it."
L["tree.block.popup"] = "Close the open window first, then try again."
L["tree.block.hold"] = "Close the open window first, then try again."
L["toast.firstRecord"] = "First record on %s: %s"
L["toast.trialComplete"] = "All %d first records set. From now on, beat them."
L["toast.trialComplete.one"] = "Your first record is set. From now on, beat it."
L["toast.star"] = "Star %d of 3: %s"
L["toast.unlock"] = "Unlocked: %s"
L["toast.unlock.sub"] = "%s step %d · %s"
L["toast.unlock.earn"] = "One set of %s earns it"
L["toast.unlock.earnMark"] = "Rate one hold of its pose Perfect to earn it"
L["toast.rekindled"] = "Won back: %s"
-- a step mastered (slice 42; the apps' kit.reward.mastered, .mastered.line): every move the setup offers there reached
-- its top
L["toast.mastered"] = "%s step %d mastered"
L["toast.mastered.sub"] = "Every move of this step your setup allows reached its top."
-- a stretch check passed above its family's step (slice 42): the family lifted to the check's step (the apps'
-- kit.reward.check.placed)
L["toast.checkPlaced"] = "%s moves up to step %d"
L["toast.check.sub"] = "Stretch check passed"
L["toast.legendary"] = "%s: %s"
L["toast.opened"] = "New ladder: %s"
L["toast.opened.sub"] = "Its first move comes with your next pumps."
L["toast.legend.other"] = "%s unlocked %s"
L["toast.legend.other.sub"] = "%s move"
L["toast.legendary.frame"] = "%s step %d · a Feat of Strength"
L["feats.title"] = "Feats of Strength"
L["feats.tip"] = "%s step %d, %s. Unlocked for good."
L["toast.p"] = "Cardio up: your cardio moves now last %d%% longer than at the start."
L["chat.rungDown"] = "%s moves to step %d: a better fit for now. Records and XP stay."
-- A ladder an old rule had moved up onto a step the player never did (Events.lua Lc.UndoOldClimbs, 2026-10-13)
L["chat.oldClimb"] = "%s goes back to step %d: an old rule moved it up too soon. Records stay."
-- A ladder the setup stops (David, 2026-10-04): its rung and what would open the next one, a phrase a reason (ns.Shut;
-- the stable chair, the table and the bar by name, other equipment by its checkbox's label). "Floor moves" is the floor
-- switch's name (gear.floor). A pain flag is never named: it goes when the pain has gone.
L["topout.line"] = "%s tops out at step %d here: %s."
L["topout.open"] = "%s to open %s"
L["topout.and"] = " and "
L["topout.floor"] = "tick Floor moves"
L["topout.space"] = "tick Room to move"
L["topout.stable_chair"] = "tick Stable chair"
L["topout.table"] = "tick Sturdy table"
L["topout.bar"] = "tick Pull-up bar"
L["topout.gear"] = "tick %s"
L["topout.need"] = "reach a %s record of %s"
-- a pose or cardio step another ladder's rung waits for (ns.RungNeed, 2026-10-10): that step's move, its ladder; a
-- pose step is done by doing its pose there, close enough to its mark once (2026-10-12)
L["topout.needStep"] = "do %s on your %s ladder"
L["topout.quiet"] = "turn off Quiet mode"
L["topout.noJump"] = "turn off No jumping"
L["topout.noStrain"] = "turn off Avoid breath-hold strain"
L["topout.noHeadDown"] = "turn off Avoid head-down positions"
L["topout.excluded"] = "check it on the Exercises page"
-- the path a climb waits for once its days are in (2026-10-16, slice 42; a rung's proof from 2026-10-10 until then):
-- the first stage still out, its move at its top on its days (a count, a pose close enough to its goal pose or
-- Perfect, a countable cardio form's clean windows), or what would open its move
L["chat.path"] = "%s: the next step also asks for %s %s+ on %s."
L["chat.path.shut"] = "%s: the next step also asks for one more move at its top: %s."
L["chat.path.mark"] = "%s: the next step also asks for %s, close enough to its goal pose on %d days."
L["chat.path.perfect"] = "%s: the next step also asks for %s, Perfect on %d days."
L["chat.path.clean"] = "%s: the next step also asks for %s, a clean 30-s all-out set on %d days."

-- Speech: the cardio rung's spoken cadence
L["cadence.rung1"] = "Quick and steady, too out of breath for full sentences."
L["cadence.rung2"] = "Fast, too out of breath for full sentences."
L["cadence.rung3"] = "Hard, too fast to talk."
L["cadence.rung4"] = "Fast and light, too fast to talk."
L["cadence.rung5"] = "Hard rhythm, land softly, too fast to talk."
L["cadence.rung6"] = "Strong and hard, too fast to talk."
L["cadence.allout"] = "Build up for ten seconds, then all out."
L["cadence.standup"] = "Easy and steady, just keep moving."
L["cadence.count"] = "All out now. Keep every rep clean."

L["voice.count"] = "%d."
L["voice.switch"] = "Switch sides."
L["voice.legendary"] = "Legendary. %s."
L["voice.artifact"] = "Artifact. %s."
-- the lines QuestPump on the computer says (desk/audio.rs), recorded with the rest so no system voice ever speaks
-- (David, 2026-10-16): the match card's line with the voice on for offers, and the answer to /qp ping
L["voice.desk.offer"] = "Match over."
L["voice.desk.ping"] = "QuestPump here."
-- the callouts (Stats.lua ns.PickCallout, 2026-10-09): one recorded line after a move's name and cue on its Ready
-- view, by the facts the engine knows; no number in a line but a milestone's own; the apps' copy holds the same words
L["callout.record.t1"] = "This one's a record attempt."
L["callout.record.t2"] = "A shot at your record on this one."
L["callout.record.t3"] = "Record attempt. Your number is on the screen."
L["callout.record.t4"] = "One step past your best. That's a record attempt."
L["callout.first.t1"] = "First max set on this one. No number to reach."
L["callout.first.t2"] = "A first max set. Whatever you do becomes the record."
L["callout.first.t3"] = "First time on this one. No number to reach."
L["callout.fresh.t1"] = "Fresh again: a good day for a record."
L["callout.fresh.t2"] = "Fresh on this one again. A record attempt."
L["callout.fresh.t3"] = "This one is fresh again. A good moment for a record."
L["callout.unlock.t1"] = "One more day at the top opens the next step."
L["callout.unlock.t2"] = "A day at the top today, and the next step opens."
L["callout.unlock.t3"] = "The next step is one day at the top away."
L["callout.proof.t1"] = "This move also counts toward your next step."
L["callout.proof.t2"] = "Your main move is done. This one opens your next step."
L["callout.proof.t3"] = "Your next step waits on this move now."
L["callout.back.t1"] = "Welcome back. Good to see you."
L["callout.back.t2"] = "Welcome back. One move at a time."
L["callout.back.t3"] = "Good to have you back."
L["callout.day7.t1"] = "Day seven of your streak."
L["callout.day7.t2"] = "This pump makes a seven-day streak."
L["callout.day30.t1"] = "Day thirty of your streak."
L["callout.day30.t2"] = "This pump makes a thirty-day streak."
L["callout.day100.t1"] = "Day one hundred of your streak."
L["callout.day100.t2"] = "This pump makes a hundred-day streak."
L["callout.day365.t1"] = "Day three hundred and sixty-five of your streak."
L["callout.day365.t2"] = "This pump makes a whole year of your streak."
L["callout.day730.t1"] = "Day seven hundred and thirty of your streak."
L["callout.day730.t2"] = "This pump makes two whole years of your streak."
L["callout.day1000.t1"] = "Day one thousand of your streak."
L["callout.day1000.t2"] = "This pump makes a thousand-day streak."
L["callout.day1095.t1"] = "Day one thousand and ninety-five of your streak."
L["callout.day1095.t2"] = "This pump makes three whole years of your streak."
L["callout.day1460.t1"] = "Day one thousand four hundred and sixty of your streak."
L["callout.day1460.t2"] = "This pump makes four whole years of your streak."
L["callout.day1825.t1"] = "Day one thousand eight hundred and twenty-five of your streak."
L["callout.day1825.t2"] = "This pump makes five whole years of your streak."
L["callout.last.t1"] = "Last move of this pump."
L["callout.last.t2"] = "This is the last one."
L["callout.last.t3"] = "Final move of the pump."
-- every all-out window's shouts (slice 53, C.shout_at_s: at 15 s and 10 s left; Speech.lua), three takes each
L["callout.shout15.t1"] = "Halfway. Faster!"
L["callout.shout15.t2"] = "Halfway there. Go faster!"
L["callout.shout15.t3"] = "Half done. Pick it up!"
L["callout.shout10.t1"] = "Ten more. Everything!"
L["callout.shout10.t2"] = "Ten seconds. Give it everything!"
L["callout.shout10.t3"] = "Last ten. All you've got!"

-- Key bindings
L["binding.header"] = "QuestPump"
L["binding.start"] = "Open the pump now"

-- Moves: name, setup (the one safety fact), cue (one rep, ending with its breath; "Count out loud" on every hold and
-- every L2-L3 rep move), stop (the form break that ends the set; none on a timed move without one, and never a
-- symptom: David, 2026-10-24, only what is legally necessary). /qp how prints them for the active move.
local function move(key, name, setup, cue, stop)
	L["move." .. key] = name
	L["move." .. key .. ".setup"] = setup
	L["move." .. key .. ".cue"] = cue
	L["move." .. key .. ".stop"] = stop
end

-- A move's guide on the pump card: its steps under How and its tips under Tips, one sentence of at most 90 characters
-- a line, joined by line breaks. The cue stays the spoken line. After the guide of a move that names a hint
-- (Library.lua `hint`, slice 42), its advice under the pose that helps it, move.<key>.hint, one sentence of at most 90
-- characters. No line times an ordinary rep or asks a pause its figure does not show (Rapha, 2026-10-28: "we have the
-- animation and the live counting"): seconds stay only where the timing is the move's point (a slow, lowering, tempo,
-- pause or hold variant, counted slow beats), on a hold, a press-and-sink's press, a safety slowness and a timed line.
local function guide(key, how, tips)
	L["move." .. key .. ".how"] = table.concat(how, "\n")
	L["move." .. key .. ".tips"] = table.concat(tips, "\n")
end

-- A move's setup line with one implement that carries its load (Library.lua holds, 2026-10-09: a real weight takes the
-- place of a bag or a jug), and its guide's steps where they name the load: the pump card shows the player's own
-- (ns.Implement), the setup line above for a makeshift one.
local function implement(key, gear, setup, how)
	L["move." .. key .. ".setup." .. gear] = setup
	if how then L["move." .. key .. ".how." .. gear] = table.concat(how, "\n") end
end

-- Squat
move("chair_stand", "Sit to stand with a calf raise",
	"Sit on the edge of a firm bed or sofa, feet flat and hip-width apart.",
	"Stand up without your hands, rise onto your toes. Breathe out as you stand.",
	"Stop when you need your hands or your knees cave in.")
guide("chair_stand", {
	"Sit near the edge of the bed or sofa, feet flat and a little behind your knees.",
	"Sit tall with both arms reaching forward at shoulder height, hands off your legs.",
	"Lean your chest forward over your feet, then press the floor away and stand up tall.",
	"Rise onto your toes, lower your heels, then sit back down slowly with the same lean.",
	"Breathe out as you stand and count one rep each time you are back on the edge.",
}, {
	"Go straight up onto your toes with your weight over the big toes, not rolling outwards.",
	"Push through your whole foot so your knees stay in line with your toes as you rise.",
	"Touch down on the edge lightly on the way back, never drop into it.",
})
move("quarter_squat", "Quarter squat at the desk", "Hold the desk edge lightly; the desk must not move.",
	"Bend a quarter of the way down, stand tall. Breathe out as you stand.",
	"Stop when your knees cave in or you lean on the desk.")
guide("quarter_squat", {
	"Stand facing the desk a small step back, feet shoulder-width apart, toes slightly out.",
	"Rest both hands on the desk edge and stand tall with your weight over the whole foot.",
	"Push your hips back and bend your knees a quarter of the way down.",
	"Breathe out and stand tall again, squeezing your seat.",
	"Count one rep each time you are back up straight.",
}, {
	"Keep only your fingertips on the desk so your legs do all the lifting.",
	"Point your knees the same way as your toes, both on the way down and on the way up.",
	"Keep your chest up and your eyes ahead so the bend comes from your hips and knees.",
})
move("chair_squat", "Chair-height squat", "Feet shoulder-width apart, the desk within reach. No chair needed.",
	"Sit back to chair height, stand tall. Breathe out as you stand.", "Stop when your knees cave in or your heels lift.")
guide("chair_squat", {
	"Stand with the desk within reach, feet shoulder-width apart and toes turned out a little.",
	"Stand tall with your arms relaxed and your weight spread over the whole foot.",
	"Sit your hips back and down to chair height, reaching your arms forward.",
	"Breathe out and drive up to standing, lowering your arms.",
	"Count one rep each time you are standing fully tall.",
}, {
	"Picture a chair behind you and reach back for it with your hips, not your knees.",
	"Keep your heels down and push the floor away through the middle of each foot.",
	"Squeeze your seat at the top so every rep ends fully upright.",
})
move("air_squat", "Full air squat", "Feet shoulder-width apart, the desk within reach.",
	"Hips below knees, chest up, stand tall. Breathe out as you stand.",
	"Stop when your heels lift or your back rounds.")
guide("air_squat", {
	"Stand with the desk within reach, feet shoulder-width apart and toes turned out slightly.",
	"Stand tall with your belly lightly tight and your weight over the middle of each foot.",
	"Sit your hips back and down until they are below your knees, reaching your arms forward.",
	"Breathe out and drive up from the bottom to stand tall.",
	"Count one rep each time you are fully upright again.",
}, {
	"Push your knees out over your toes on the way down so they never drift inwards.",
	"Keep your chest proud and your eyes ahead, not down at the floor.",
	"Drive up through your whole foot, with heels and toes both pressing the floor.",
})
L["move.air_squat.hint"] = "Turn your toes out a little if your heels lift at the bottom."
move("tempo_squat", "Tempo squat", "Feet shoulder-width apart, the desk within reach.",
	"Down for 3, pause 1, up in 1. Count out loud, breathe out as you stand.",
	"Stop when you lose the tempo or your knees cave in.")
guide("tempo_squat", {
	"Stand with the desk within reach, feet shoulder-width apart and toes turned out slightly.",
	"Stand tall with your arms at your sides and your weight over the middle of each foot.",
	"Count three slow beats as you sit your hips down below your knees, arms reaching forward.",
	"Hold still at the bottom for one beat, then breathe out and stand up in one beat.",
	"Count one rep each time you are fully upright, then start the next slow count.",
}, {
	"Say the beats out loud so the lowering never speeds up near the bottom.",
	"Keep breathing during the slow lowering, never hold your breath to stay tight.",
	"At the pause stay tight and still, with no bounce out of the bottom.",
})
move("sumo_squat", "Sumo squat", "Feet wide, toes turned out, the desk within reach.",
	"Knees follow your toes, sit down between your heels. Breathe out as you stand.",
	"Stop when your knees fall inward.")
guide("sumo_squat", {
	"Stand with your feet about twice shoulder-width apart, toes turned out to the corners.",
	"Clasp your hands in front of your chest and stand tall.",
	"Bend your knees out over your toes and sit straight down, thighs almost level.",
	"Breathe out and push through your heels to stand.",
	"Count one rep each time your legs are straight again.",
}, {
	"Turn your toes out only as far as your knees can follow.",
	"Think of pushing the floor apart with your feet to keep your knees wide.",
	"Keep your chest high and lower your hips straight down, not back behind you.",
})
move("squat_punch", "Squat to punch", "Clear arm's length around you.",
	"Squat, stand and punch across your body. Count out loud, breathe out on the punch.",
	"Stop when your squat gets shallow or your knees cave in.")
guide("squat_punch", {
	"Stand with feet shoulder-width apart and raise your fists to your chin, elbows in.",
	"Sit your hips back into a squat until your thighs are nearly level, fists still up.",
	"Stand up, then punch one fist across your body, pivoting on the foot on that side.",
	"Breathe out sharply on the punch, then pull the fist straight back to your chin.",
	"Count one rep out loud for each squat and punch, switching the punching arm each rep.",
}, {
	"Turn your hips and lift the heel on the punching side so the power starts from the floor.",
	"Bring the fist back to your chin after each punch, and never let the other hand drop.",
	"Punch to full reach without snapping the elbow straight at the end.",
})
move("onehalf_squat", "One-and-a-half squat", "Feet shoulder-width apart, the desk within reach.",
	"Down, halfway up, down again, stand. Count out loud, breathe out as you stand.",
	"Stop when your heels lift or your knees cave in.")
guide("onehalf_squat", {
	"Stand with the desk within reach, feet shoulder-width apart and toes turned out slightly.",
	"Stand tall, then sit your hips down into a deep squat, reaching your arms forward.",
	"Rise only halfway up, then sink straight back down to the bottom.",
	"Now breathe out and stand all the way up, lowering your arms to your sides.",
	"Count one rep out loud each time you finish the full pattern and stand.",
}, {
	"Stop at the same halfway height every rep, so it never creeps higher or lower.",
	"Stay bent through the middle part, with no straightening up until the final stand.",
	"Keep your heels down on the second drop, where they most often start to lift.",
})
move("deep_squat_hold", "Deep squat hold", "Hold the desk edge if your heels lift.",
	"Sit deep, chest up, heels down. Count out loud.", "Stop when your heels lift or your knees ache.")
guide("deep_squat_hold", {
	"Stand a small step from the desk, feet shoulder-width apart and toes turned out.",
	"Sit all the way down between your heels until your hips are well below your knees.",
	"Reach your arms forward to balance, or hold the desk edge if your heels start to lift.",
	"Keep your chest up and breathe slowly in and out, counting out loud.",
	"The clock times your stay at the bottom with heels down, then stand up slowly.",
}, {
	"Press your knees out over your toes to make room to sit deeper between them.",
	"Let each slow breath out help you settle a little lower and a little taller.",
	"Keep your weight over the whole foot rather than rocking onto your toes.",
})
move("split_squat", "Split squat", "One hand on the desk edge, feet a long step apart.",
	"Back knee toward the floor, front shin upright. Count out loud, breathe out as you rise.",
	"Stop when your front knee caves in. Count the weaker side.")
guide("split_squat", {
	"Stand side-on to the desk with one hand on its edge and feet about a metre apart.",
	"Keep your feet hip-width apart, the back heel up and your body tall between them.",
	"Lower the back knee straight down until it hovers just above the floor.",
	"Breathe out and push through the front foot to rise, keeping the front shin upright.",
	"Count each rise out loud; finish one leg, press Switch sides, then do the other.",
}, {
	"Keep the front heel planted and press through the whole front foot.",
	"Go straight down and up like a lift, not forward over the front toes.",
	"Place your feet as if on two train tracks, not one tightrope, for better balance.",
})
move("bulgarian", "Bulgarian split squat", "Rear foot on the edge of a bed or sofa behind you.",
	"Drop the back knee, front shin upright. Count out loud, breathe out as you rise.",
	"Stop when you wobble or your knee caves in. Count the weaker side.")
guide("bulgarian", {
	"Stand with your back to a bed or sofa and rest the top of one foot on its edge.",
	"Set the front foot far enough forward that its shin stays upright at the bottom.",
	"Lower the back knee towards the floor, leaning your chest slightly forward.",
	"Breathe out and push through the front foot to rise back to the top.",
	"Count each rise out loud; finish one leg, press Switch sides, then do the other.",
}, {
	"Keep most of your weight on the front foot and let the back leg only balance you.",
	"Lay the laces of the back foot flat on the edge so it rests rather than pushes.",
	"Keep your front knee in line with your middle toes all the way down and up.",
})
L["move.bulgarian.hint"] = "Lower the back foot or shorten your stance if your back thigh feels tight."
move("split_squat_slow", "Split squat, 3-s lowering", "One hand on the desk edge, feet a long step apart.",
	"Lower for 3 seconds, rise in 1. Count out loud, breathe out as you rise.",
	"Stop when you lose the 3-s lowering. Count the weaker side.")
guide("split_squat_slow", {
	"Stand side-on to the desk with one hand on its edge and feet about a metre apart.",
	"Keep your feet hip-width apart, the back heel up and your body tall between them.",
	"Lower the back knee for a slow count of three until it hovers just above the floor.",
	"Breathe out and rise in one count by pushing through the front foot.",
	"Count each rise out loud; finish one leg, press Switch sides, then do the other.",
}, {
	"Count the three seconds out loud so the last part of the lowering stays slow.",
	"Keep breathing in as you lower, never hold your breath to stay steady.",
	"Use the desk for balance only, so your legs control the speed.",
})
-- Beside the Bulgarian (2026-10-12): the pistol with both hands on the desk
move("pistol_two_hands", "Pistol, hands on the desk", "Both hands on the desk edge; the desk must not move.",
	"Sit all the way down on one leg, hands on the desk. Count out loud, breathe out up.",
	"Stop when your heel lifts or you must pull hard. Count the weaker side.")
guide("pistol_two_hands", {
	"Face the desk a small step back and hold its edge with both hands, arms reaching forward.",
	"Stand on one leg and hold the other leg straight out in front, heel off the floor.",
	"Sit down on the standing leg until your thigh rests on your calf, heel flat.",
	"Breathe out and stand back up, pulling on the desk only as much as you need.",
	"Count each rise out loud; finish one leg, press Switch sides, then do the other.",
}, {
	"Keep the standing heel flat and that knee over your middle toes.",
	"Go all the way down every time; a rise from halfway does not count.",
	"Pull on the desk a little less each week, until your fingertips are enough.",
})
move("assisted_pistol", "Assisted pistol", "Fingertips on the desk edge; the desk must not move.",
	"Sit all the way down on one leg, other leg forward. Count out loud, breathe out up.",
	"Stop when your knee caves in or you need more than fingertips. Count the weaker side.")
guide("assisted_pistol", {
	"Face the desk a small step back, fingertips resting on its edge, arms reaching forward.",
	"Stand on one leg and hold the other leg straight out in front, heel off the floor.",
	"Sit back and down on the standing leg until your thigh rests on your calf.",
	"Breathe out and stand back up, the fingertips only steadying you, never pulling.",
	"Count each rise out loud; finish one leg, press Switch sides, then do the other.",
}, {
	"Keep the standing heel flat and that knee tracking over your middle toes.",
	"Lean your chest forward as you sit to keep your weight over the standing foot.",
	"Go all the way down every time; a rise from halfway does not count.",
})
move("pistol", "Pistol squat", "Clear space; the desk within reach to catch yourself.",
	"One leg forward, sit all the way down, stand. Count out loud, breathe out as you rise.",
	"Stop when your heel lifts or your knee caves in. Count the weaker side.")
guide("pistol", {
	"Stand in clear space with the desk just within reach at your side to catch yourself.",
	"Stand on one leg, reach both arms forward and lift the other leg straight out in front.",
	"Sit down on the standing leg all the way to the bottom, heel flat.",
	"Breathe out and stand straight back up, keeping the free leg off the floor.",
	"Count each rise out loud; finish one leg, press Switch sides, then do the other.",
}, {
	"Reach your arms further forward as you sink to keep your balance over the foot.",
	"Pull the toes of the free leg up towards you to keep that leg straight and high.",
	"Stay tight at the bottom and drive straight up, with no bounce.",
})
-- Beside the assisted pistol (2026-10-10): the pistol to a bed edge and the one-leg lowering
move("box_pistol", "Pistol to the bed edge", "Your back to a bed or sofa edge; the desk within reach to catch you.",
	"Other leg forward, sit to the edge, stand on one leg. Count out loud, breathe out up.",
	"Stop when you drop onto the edge or your knee caves in. Count the weaker side.")
guide("box_pistol", {
	"Stand a small step in front of a bed or sofa, your back to it, the desk within reach.",
	"Stand on one leg, reach both arms forward and lift the other leg straight out in front.",
	"Sit back on the standing leg until your seat just touches the edge.",
	"Breathe out and stand straight back up on that leg, with no hands and no rocking.",
	"Count each rise out loud; finish one leg, press Switch sides, then do the other.",
}, {
	"Touch the edge as lightly as you can; never sit down on it and rock back up.",
	"Keep the standing heel flat and that knee over your middle toes.",
	"A lower edge is harder, so keep the same bed or sofa until the count comes easily.",
})
move("pistol_lowering", "One-leg lowering", "Clear space; the desk within reach to catch yourself.",
	"Lower on one leg for 5 seconds, then stand up on both. Count out loud, breathe steadily.",
	"Stop when you drop fast or your heel lifts. Count the weaker side.")
guide("pistol_lowering", {
	"Stand in clear space with the desk just within reach at your side to catch yourself.",
	"Stand on one leg, reach both arms forward and lift the other leg straight out in front.",
	"Lower on the standing leg for a slow count of five, all the way to the bottom, heel flat.",
	"At the bottom set the other foot down and stand up on both legs, breathing out.",
	"Count each lowering out loud; finish one leg, press Switch sides, then do the other.",
}, {
	"Count the five seconds out loud so the last part of the lowering stays slow.",
	"Reach your arms further forward as you sink to keep your weight over the foot.",
	"Count only a lowering that reaches the bottom under control.",
})
move("squat_jump", "Squat jump", "Clear space, shoes on, a floor that does not slip.",
	"Squat, jump, land softly on the balls of your feet. Count out loud, breathe out up.",
	"Stop when your landings get loud or your knees cave in.")
guide("squat_jump", {
	"Stand with feet shoulder-width apart and check nothing hangs low above your head.",
	"Squat down to about chair height and swing your arms back behind you.",
	"Swing your arms up and jump straight up, reaching for the ceiling.",
	"Breathe out as you jump, land on the balls of your feet and sink into your knees.",
	"Stand up tall, count the rep out loud, then start the next jump.",
}, {
	"Land as quietly as you can, rolling from the balls of your feet down to your heels.",
	"Keep your knees pointing over your toes both on take-off and on landing.",
	"Reset to standing tall before each jump, one clean jump at a time.",
})

-- Wall sit
move("wall_sit_120", "Wall sit, quarter", "Back flat on the wall, nothing slippery under your feet.",
	"Slide down a quarter, knees over ankles. Count out loud.", "Stop when you slide or your knees ache.")
guide("wall_sit_120", {
	"Stand with your back to the wall, feet hip-width apart and heels about 50 cm out.",
	"Press your shoulders and lower back to the wall, arms loose by your sides.",
	"Slide down about a quarter of the way, until your knees sit right over your ankles.",
	"Hold still and breathe slowly in and out, counting out loud.",
	"The clock times how long you stay at this depth, then slide back up to stand.",
}, {
	"If your knees drift past your toes, walk your feet a little further from the wall.",
	"Keep your knees pointing straight ahead, in line with your middle toes.",
	"Pick a spot on the far wall and keep your eyes on it to stay still.",
})
move("wall_sit_105", "Wall sit, deep", "Back flat on the wall, nothing slippery under your feet.",
	"Slide down until your thighs are nearly level. Count out loud.", "Stop when you slide or your knees ache.")
guide("wall_sit_105", {
	"Stand with your back to the wall, feet hip-width apart and heels about 60 cm out.",
	"Press your shoulders and lower back to the wall, arms loose by your sides.",
	"Slide down until your thighs are nearly level with the floor, knees over your ankles.",
	"Hold still and breathe slowly in and out, counting out loud.",
	"The clock times how long you stay at this depth, then slide back up to stand.",
}, {
	"Slide down slowly so you stop at the right depth, rather than dropping and adjusting.",
	"Press evenly through both feet so neither leg takes over.",
	"Breathe out on every count so your breath keeps moving as the hold gets hard.",
})
move("wall_sit_90", "Wall sit, thighs level", "Back flat on the wall, nothing slippery under your feet.",
	"Thighs level, knees over ankles. Count out loud.", "Stop when you slide or your knees ache.")
guide("wall_sit_90", {
	"Stand with your back to the wall, feet hip-width apart and heels about 60 cm out.",
	"Press your shoulders and lower back to the wall, arms loose by your sides.",
	"Slide down until your thighs are level with the floor, knees right over your ankles.",
	"Hold still and breathe slowly in and out, counting out loud.",
	"The clock times how long you hold your thighs level, then slide back up to stand.",
}, {
	"Picture your shins straight up and down and your thighs flat like a table top.",
	"Push your heels into the floor to keep your back from sliding down the wall.",
	"Breathe out on every count so your breath keeps moving as the hold gets hard.",
})
move("sl_wall_sit_105", "Single-leg wall sit, 105 degrees", "Back flat on the wall, nothing slippery underfoot.",
	"Thighs nearly level, one leg straight out. Count out loud.",
	"Stop when you slide or your knee aches. Count the weaker side.")
guide("sl_wall_sit_105", {
	"Stand with your back to the wall, feet hip-width apart and heels about 60 cm out.",
	"Slide down until your thighs are nearly level with the floor, knees over your ankles.",
	"Straighten one leg out in front, its heel just off the floor, and hold on the other.",
	"Hold still and breathe slowly in and out, your head and back flat on the wall.",
	"Count out loud; when this side ends, press Switch sides, then change legs.",
}, {
	"Keep the standing knee over its ankle and pointing straight ahead, not caving in.",
	"Let the free leg hover low, since holding it high only tires your hip.",
	"Press the whole standing foot into the floor, heel too, so you do not slide.",
})
move("sl_wall_sit_90", "Single-leg wall sit, 90 degrees", "Back flat on the wall, nothing slippery underfoot.",
	"Thighs level, one leg straight out. Count out loud.",
	"Stop when you slide or your knee aches. Count the weaker side.")
guide("sl_wall_sit_90", {
	"Stand with your back to the wall, feet hip-width apart and heels about 60 cm out.",
	"Slide down until your thighs are level with the floor, knees right over your ankles.",
	"Straighten one leg out in front, its heel just off the floor, and hold on the other.",
	"Hold still and breathe slowly in and out, your head and back flat on the wall.",
	"Count out loud; when this side ends, press Switch sides, then change legs.",
}, {
	"Picture the standing shin straight up and down and its thigh flat like a table top.",
	"Push the standing heel into the floor to keep your back from sliding down the wall.",
	"Breathe out on every count so your breath keeps moving as the hold gets hard.",
})
move("staggered_wall_sit", "Staggered wall sit", "Back flat on the wall, nothing slippery under your feet.",
	"Thighs level, one foot a half step ahead with most of your weight on it. Count out loud.",
	"Stop when you slide or your knee aches. Count the weaker side.")
guide("staggered_wall_sit", {
	"Stand with your back to the wall, heels about 60 cm out, feet hip-width apart.",
	"Step one foot a half step forward and put most of your weight on it.",
	"Slide down until your thighs are level, the front knee over its ankle.",
	"Hold still and breathe slowly, counting out loud.",
	"When this side ends, stand, press Switch sides and put the other foot ahead.",
}, {
	"Keep the back foot light, resting on the floor for balance only.",
	"Keep the front knee over its ankle, never past your toes.",
})
move("wall_sit_punches", "Wall sit punches", "Back flat on the wall, nothing slippery under your feet.",
	"Thighs level, punch straight ahead, one fist then the other. Count out loud.",
	"Stop when you slide or your knees ache.")
guide("wall_sit_punches", {
	"Stand with your back to the wall, feet hip-width apart and heels about 60 cm out.",
	"Slide down until your thighs are level with the floor, knees right over your ankles.",
	"Punch one fist straight ahead and back to your chin, then the other.",
	"Keep punching at an easy pace, breathing out on each punch and counting out loud.",
	"The clock times how long you hold your thighs level, then slide back up to stand.",
}, {
	"Keep your shoulders and lower back on the wall as your arms punch.",
	"Stop each punch just short of a straight elbow.",
})

-- Lunge and single leg
move("reverse_lunge", "Slow reverse lunge", "The desk within reach for balance.",
	"Step back slowly, back knee toward the floor. Count out loud, breathe out as you rise.",
	"Stop when your front knee caves in. Count the weaker side.")
guide("reverse_lunge", {
	"Stand side-on to the desk, feet hip-width apart, close enough to touch the edge.",
	"Stand tall with your weight on one foot, chest up and arms loose at your sides.",
	"Take one long step back, then lower for 2 s until the back knee hovers above the floor.",
	"Breathe in on the way down and out as you push through the front heel to stand.",
	"Each return to standing counts one; finish one side, press Switch sides, then the other.",
}, {
	"Keep the front heel down and push through it, so the front leg does the work.",
	"Step back far enough that the front shin stays close to upright at the bottom.",
	"Fix your eyes on a spot ahead at eye level to stay steady on the way down.",
})
move("lateral_lunge", "Lateral lunge", "A clear step to each side.",
	"Step wide, sit into one hip, the other leg straight. Breathe out as you push back.",
	"Stop when your knee passes your toes or caves in. Count the weaker side.")
guide("lateral_lunge", {
	"Stand with a clear metre on each side of you, feet hip-width apart.",
	"Stand tall, then step one foot out until your feet are about twice shoulder-width apart.",
	"Sit your hips back over that foot, the other leg straight, arms forward.",
	"Breathe in as you sink, then breathe out as you push off that foot back to the start.",
	"Each return to the start counts one; finish one side, press Switch sides, then the other.",
}, {
	"Keep both feet flat and pointing forward, the heel of the bent leg pressed down.",
	"Push your hips back as if closing a drawer behind you, rather than dropping straight down.",
	"Feel a stretch along the inside of the straight leg as you sit into the bent one.",
})
move("curtsy_lunge", "Curtsy lunge", "The desk within reach for balance.",
	"Step back and across, hips square. Count out loud, breathe out as you rise.",
	"Stop when your front knee caves in. Count the weaker side.")
guide("curtsy_lunge", {
	"Stand with feet hip-width apart, hands on your hips, the desk close enough to touch.",
	"Step one foot back and across behind the other, landing on the ball of that foot.",
	"Lower until the back knee hovers above the floor, hips facing ahead.",
	"Breathe in on the way down and out as you push through the front heel to stand.",
	"Each return to standing counts one; finish one side, press Switch sides, then the other.",
}, {
	"Keep your hips square, as if lights on your hip bones shine straight ahead.",
	"Step the foot well back, not only across, so the front knee stays over the front foot.",
	"Keep most of your weight in the front heel and use the back foot only for balance.",
})
move("knee_drive", "Step-back knee drive", "The desk within reach for balance.",
	"Step back, then drive that knee up high. Breathe out on the drive.",
	"Stop when you lose your balance. Count the weaker side.")
guide("knee_drive", {
	"Stand beside the desk with feet hip-width apart, close enough to touch the edge.",
	"Step one foot back into a lunge, both knees bent, chest tall over your hips.",
	"Drive the back knee up to hip height as the opposite arm swings forward.",
	"Breathe in as you step back and breathe out hard on the drive.",
	"Each knee drive counts one; finish one side, press Switch sides, then do the other.",
}, {
	"Stand tall on the standing leg at the top, hips pushed forward, not leaning back.",
	"Fix your eyes on one spot ahead to hold your balance on one leg.",
	"Make the drive fast but the step back soft and under control.",
})
move("spider_lunge", "Spider lunge", "A body length of clear floor.",
	"Step forward to a lunge, both hands inside the front foot. Breathe out as you step back.",
	"Stop when your hips drop or twist. Count the weaker side.")
guide("spider_lunge", {
	"Kneel on the floor with room to stretch out long, hands flat under your shoulders.",
	"Walk your feet back into a straight line from head to heels, arms straight.",
	"Step one foot up to land flat outside the same hand, back leg long, and pause a moment.",
	"Breathe in as you step forward and breathe out as you step back to the straight line.",
	"Each step back counts one; finish one side, press Switch sides, then the other.",
}, {
	"Push the back heel away in the lunge to feel a stretch at the front of that hip.",
	"Keep the back knee off the floor and the back leg straight through the whole step.",
	"Press the floor away with both hands so your shoulders stay over your wrists.",
})
move("duck_walk", "Duck walk", "A clear path, the desk within reach.",
	"Stay low in a squat, small steps forward and back. Count out loud.",
	"Stop when your knees ache or your heels lift.")
guide("duck_walk", {
	"Clear a short path beside the desk, with nothing on the floor to trip on.",
	"Sit down into a low squat, arms held out in front at shoulder height, chest up.",
	"Take small, steady steps forward, then the same small steps back, staying at one height.",
	"Breathe steadily and count out loud as you step.",
	"The clock times how long you stay low and stepping, so keep going until it ends.",
}, {
	"Push your knees out over your toes so your heels stay flat on the floor.",
	"Keep your head at one height the whole time, with no bobbing up between steps.",
	"Reach your arms forward to keep your weight over the middle of your feet.",
})
L["move.duck_walk.hint"] = "Walk a little higher if your heels lift off the floor."
move("frog_jumps", "Frog jumps", "Clear space, shoes on, a floor that does not slip.",
	"Squat deep, jump forward, land softly. Count out loud, breathe out on the jump.",
	"Stop when your landings get loud or your knees cave in.")
guide("frog_jumps", {
	"Clear about a metre ahead and stand with feet wider than your hips, toes turned out.",
	"Sink into a deep squat, chest forward, arms swung back behind you, knees pushed out.",
	"Swing your arms forward and jump a short way ahead, landing softly in the deep squat.",
	"Breathe out as you jump and breathe in as you stand and walk back to the start.",
	"Each landing counts as one, so stand up, walk back and set up again for the next.",
}, {
	"Land on the balls of your feet first, then let your heels come down to soak it up.",
	"Aim for a short, quiet jump, since a soft landing matters more than distance.",
	"Look ahead, not down, so your chest stays up as you land.",
})

-- Hinge
move("good_morning", "Good morning", "Stand tall, hands on your hips.",
	"Hips back, back flat, bow until you feel your hamstrings. Breathe out as you rise.",
	"Stop when your back rounds.")
guide("good_morning", {
	"Stand with feet hip-width apart, hands on your hips, knees very slightly bent.",
	"Stand tall with your chest up, shoulders back and your neck long.",
	"Push your hips back and bow, back flat, until you feel the backs of your thighs.",
	"Breathe in as you bow, then breathe out as you drive your hips forward to stand tall.",
	"Keep a slow, even rhythm until the clock ends.",
}, {
	"Imagine your hips reaching back to touch a wall behind you.",
	"Keep your knees still and slightly bent, since the move happens at the hips.",
	"Look at the floor a little ahead so your neck stays in line with your back.",
})
move("single_leg_rdl", "Single-leg deadlift at the desk", "One hand on the desk edge.",
	"Hinge on one leg, the other leg back, hips level. Breathe out as you rise.",
	"Stop when your hips twist or your back rounds. Count the weaker side.")
guide("single_leg_rdl", {
	"Stand side-on to the desk and rest the nearer hand lightly on its edge.",
	"Stand on the leg away from the desk, knee soft, the other foot just off the floor.",
	"Tip from the hip as the free leg reaches back, back flat, chest nearly level.",
	"Breathe in as you tip forward and breathe out as you squeeze your seat to stand tall.",
	"Each stand counts one; finish one side, press Switch sides, then turn round for the other.",
}, {
	"Point the toes of the back foot at the floor to keep your hips level.",
	"Keep a straight line from your head to your back heel, like a seesaw.",
	"Use the desk only for light balance and press down through the standing heel.",
})
move("glute_bridge", "Glute bridge", "Lie on your back on the floor, knees bent, feet flat.",
	"Push through your heels, hips up until knees, hips and shoulders line up. Breathe out up.",
	"Stop when your back arches or your hamstrings cramp.")
guide("glute_bridge", {
	"Lie on your back, feet flat and hip-width apart, heels about 30 cm from your seat.",
	"Rest your arms flat by your sides, palms down, with your head on the floor.",
	"Push through your heels to lift your hips, then squeeze your seat for a moment at the top.",
	"Breathe out as you lift and breathe in as you lower slowly.",
	"Each time your hips touch down counts as one.",
}, {
	"Tuck your tailbone a little as you lift, so the lift comes from your seat.",
	"Keep your knees hip-width apart, pointing straight over your feet.",
	"Press your arms into the floor so your shoulders stay down and your hips do the lifting.",
})
move("single_leg_bridge", "Single-leg bridge", "Lie on your back on the floor, one knee bent, the other leg up.",
	"Push through one heel, hips level. Count out loud, breathe out as you lift.",
	"Stop when your hips drop or tilt, or your hamstring cramps. Count the weaker side.")
guide("single_leg_bridge", {
	"Lie on your back, one foot flat with the heel about 30 cm from your seat, arms flat.",
	"Straighten the other leg and lift it off the floor, then keep it still.",
	"Push through the flat heel to raise your hips level, pause at the top, then lower slowly.",
	"Breathe out as you lift and breathe in as you lower, counting each rep out loud.",
	"Each time your hips touch down counts one; one side, press Switch sides, then the other.",
}, {
	"Press your arms into the floor to help keep both hips at the same height.",
	"Keep the raised leg still and let only the flat heel drive the lift.",
	"Keep the working knee in line over its foot, not drifting in or out.",
})
move("hamstring_walkout", "Hamstring walkout", "Lie on your back on the floor, knees bent, hips up in a bridge.",
	"Walk your heels out and back, hips up. Count out loud, breathe steadily.",
	"Stop when your hips drop or your hamstrings cramp.")
guide("hamstring_walkout", {
	"Lie on your back on the floor, feet flat and hip-width apart, arms flat by your sides.",
	"Lift your hips into a bridge, then rest on your heels with your toes pulled up.",
	"Walk your heels out in small steps, one foot at a time, to nearly straight legs.",
	"Walk them back the same way while your hips stay off the floor, breathing steadily.",
	"Each full trip out and back counts as one, counted out loud.",
}, {
	"Keep the steps small, only a few centimetres, so your hips can stay lifted.",
	"Dig your heels into the floor and pull toward your seat on the way back.",
	"Keep your seat squeezed the whole time so your hips stay high.",
})
move("sliding_curl", "Sliding leg curl", "Lie on your back, heels on a towel on a smooth floor.",
	"Hips up, slide your heels out and pull them back. Count out loud, breathe out as you pull.",
	"Stop when your hips drop or your hamstrings cramp.")
guide("sliding_curl", {
	"Lie on your back with both heels on a towel, on a smooth floor where it can slide.",
	"Pull your toes up and lift your hips until your knees, hips and shoulders line up.",
	"Slide your heels out until your legs are nearly straight, hips still up.",
	"Breathe in as you slide out and breathe out as you pull your heels back under your knees.",
	"Each pull back to the start counts as one, counted out loud.",
}, {
	"Press your arms and shoulders into the floor to keep your hips high.",
	"Dig your heels into the towel as you pull, as if dragging the floor toward you.",
	"Pull with both heels at the same speed so the towel slides straight back.",
})
move("sl_sliding_curl", "Single-leg sliding curl", "Lie on your back, one heel on a towel on a smooth floor.",
	"Hips up, slide one heel out and back. Count out loud, breathe out as you pull.",
	"Stop when your hips drop or tilt. Count the weaker side.")
guide("sl_sliding_curl", {
	"Lie on your back with one heel on a towel, on a smooth floor where it can slide.",
	"Hold the other leg straight up at an angle, pull your toes up and raise your hips.",
	"Slide the heel out until the leg is nearly straight, then pull it back.",
	"Breathe in as you slide out and breathe out as you pull, keeping your hips level.",
	"Each pull back counts one, out loud; finish one side, press Switch sides, then the other.",
}, {
	"Press both arms into the floor to keep your hips level and high.",
	"Keep the raised leg still and pointing up, so it never swings to help.",
	"Pull the heel back slowly, as if dragging something heavy toward you.",
})
move("sl_slide_lowering", "Single-leg slide out, both back",
	"Lie on your back, both heels on a towel on a smooth floor.",
	"Hips up, one heel out in 3 seconds, pull back with both. Count out loud, breathe out.",
	"Stop when your hips drop or tilt. Count the weaker side.")
guide("sl_slide_lowering", {
	"Lie on your back with both heels on a towel, on a smooth floor where it can slide.",
	"Pull your toes up and lift your hips until your knees, hips and shoulders line up.",
	"Lift one foot and slide the other heel out for a slow count of three, hips still up.",
	"Set the lifted heel down by it and pull both heels back under your knees, breathing out.",
	"Each pull back counts one, out loud; finish one side, press Switch sides, then the other.",
}, {
	"Press both arms into the floor to keep your hips level and high.",
	"Count the three seconds out loud so the slide stays slow to the end.",
	"Keep the lifted leg still, so it never swings to help.",
})
move("nordic_assisted", "Assisted Nordic", "Heels under a heavy sofa, never a chair; hands ready to catch.",
	"Kneel tall, lean as far as you control, catch, push back up. Count out loud.",
	"Stop when you fall sooner than on your first rep or bend at the hips.")
guide("nordic_assisted", {
	"Kneel on a folded towel, heels hooked under a heavy sofa that cannot tip or slide.",
	"Kneel tall in a straight line from knees to head, hands up in front of your chest.",
	"Lean forward as slowly as you can as one straight piece, as far as you still control.",
	"When you cannot slow it any more, let your hands catch you softly, then push back up.",
	"Each return to kneeling tall counts as one, counted out loud.",
}, {
	"Squeeze your seat so your hips stay straight as you lean.",
	"Pull your heels up hard into the sofa to feel the backs of your thighs working.",
	"Go a little further each week before your hands catch you.",
})
move("nordic_eccentric", "Nordic lowering", "Heels under a heavy sofa, never a chair; hands ready to catch.",
	"Lower for 4 to 5 seconds all the way to the floor, hands only at the end. Count out loud.",
	"Stop when you drop fast or bend at the hips.")
guide("nordic_eccentric", {
	"Kneel on a folded towel, heels hooked under a heavy sofa that cannot tip or slide.",
	"Kneel tall in a straight line from knees to head, hands up in front of your chest.",
	"Lower forward for 4 to 5 s all the way to the floor, body straight as a board.",
	"Catch yourself on your hands only at the very bottom, push back to kneeling and breathe.",
	"Count only a lowering that reaches the floor under control, counted out loud.",
}, {
	"Squeeze your seat and belly so the line from knees to head never folds.",
	"Pull your heels up into the sofa to slow the lowering with the backs of your thighs.",
	"Keep your hands ready in front with bent elbows so the catch at the bottom is soft.",
})
move("nordic_hold", "Nordic with a hold", "Heels under a heavy sofa, never a chair; hands ready to catch.",
	"Lower halfway, hold 2 seconds, then lower slowly. Count out loud.",
	"Stop when you cannot hold or you bend at the hips.")
guide("nordic_hold", {
	"Kneel on a folded towel, heels hooked under a heavy sofa that cannot tip or slide.",
	"Kneel tall in a straight line from knees to head, hands up in front of your chest.",
	"Lower slowly to halfway, hold still for 2 s, then keep lowering as slowly as you can.",
	"Breathe steadily through the hold, then catch on your hands and push back to kneeling.",
	"Each lowering with its hold counts as one, counted out loud.",
}, {
	"Squeeze your seat and belly so your hips stay straight through the hold.",
	"Pull your heels up into the sofa to feel the backs of your thighs hold you.",
	"Count the 2 s out loud, which keeps you breathing through the hold.",
})
move("nordic_pullback", "Nordic pull-back", "Heels under a heavy sofa, never a chair; hands ready to catch.",
	"Lower only as far as you can come back from, pull back up with no hands. Count out loud.",
	"Stop when you cannot pull back up or you bend at the hips.")
guide("nordic_pullback", {
	"Kneel on a folded towel, heels hooked under a heavy sofa that cannot tip or slide.",
	"Kneel tall in a straight line from knees to head, hands up in front of your chest.",
	"Lean forward slowly as one straight piece, only as far as you can still come back from.",
	"Pull your heels into the sofa and pull back up to kneeling tall, hands off the floor.",
	"Breathe steadily. Each return to kneeling tall counts as one, counted out loud.",
}, {
	"Go a little further as you get stronger, always stopping short of where you would drop.",
	"Squeeze your seat so your hips stay straight on the way down and back.",
	"Keep your hands ready in front; if you drop, catch yourself and end the set.",
})
-- Beside the assisted Nordic and the pull-back (2026-10-12): the hover at the bottom and the fingertip push
move("nordic_hover", "Nordic hover at the bottom", "Heels under a heavy sofa, never a chair; a towel under your knees.",
	"At the bottom, hands up for 2 seconds, body straight. Count out loud, breathe steadily.",
	"Stop when you cannot hold 2 seconds or your hips bend.")
guide("nordic_hover", {
	"Kneel on a folded towel, heels hooked under a heavy sofa that cannot tip or slide.",
	"Lower forward as one straight piece and catch yourself on your hands, arms straight.",
	"Keep your body straight and lift both hands a few centimetres off the floor for 2 s.",
	"Set your hands down softly, breathe, and lift again; each lift counts as one.",
	"After the last lift, push off your hands back to kneeling tall.",
}, {
	"Squeeze your seat and belly so the line from knees to head never folds.",
	"Pull your heels up into the sofa as your hands lift, so the backs of your thighs hold you.",
	"Lift your hands only a few centimetres, so each set-down stays soft.",
})
-- Its easier version, the hands on books (2026-10-13)
move("nordic_hover_books", "Nordic hover, hands on books",
	"Heels under a heavy sofa, never a chair; a towel under your knees; books 15-20 cm high.",
	"On the books, hands up for 2 seconds, body straight. Count out loud, breathe steadily.",
	"Stop when you cannot hold 2 seconds or your hips bend.")
guide("nordic_hover_books", {
	"Kneel on a folded towel, heels hooked under a heavy sofa that cannot tip or slide.",
	"Stack flat books 15-20 cm high where your hands will land; press on them so they stay put.",
	"Lower forward as one straight piece and catch yourself on the books, arms straight.",
	"Lift both hands a few centimetres off the books for 2 s, keeping your body straight.",
	"Set them down softly; each lift counts one. After the last, push back to kneeling tall.",
}, {
	"Squeeze your seat and belly so the line from knees to head never folds.",
	"Pull your heels up into the sofa as your hands lift, so the backs of your thighs hold you.",
	"Lift your hands only a few centimetres, so each set-down stays soft.",
})
move("nordic_fingertip", "Nordic curl, fingertip push",
	"Heels under a heavy sofa, never a chair; hands ready to catch.",
	"Lower to your fingertips, one small push, pull back up. Count out loud, breathe steadily.",
	"Stop when you need your whole hands to come up or you bend at the hips.")
guide("nordic_fingertip", {
	"Kneel on a folded towel, heels hooked under a heavy sofa that cannot tip or slide.",
	"Kneel tall in a straight line from knees to head, hands up in front of your chest.",
	"Lower forward for 3 s or more as one straight piece until your fingertips touch the floor.",
	"Give one small push with your fingertips, then pull up with the backs of your thighs.",
	"Breathe steadily. Each trip down and back up counts as one, counted out loud.",
}, {
	"Push with your fingertips only, never your whole hands.",
	"Pull your heels up hard into the sofa to start the way back.",
	"Push a little less each week; the full Nordic needs no push at all.",
})
move("hip_thrust_sl", "Single-leg hip thrust",
	"Shoulders on a sofa or a chair without wheels; a 5-10 kg pack on hips if you have one.",
	"Push through one heel, hips up level. Count out loud, breathe out as you lift.",
	"Stop when your hips tilt or your back arches. Count the weaker side.")
guide("hip_thrust_sl", {
	"Sit on the floor with your upper back on the sofa edge or a chair pushed back to a wall.",
	"Plant one foot flat, the other leg out straight, any backpack resting on your hips.",
	"Push through the flat heel until your knee, hips and shoulders form one level line.",
	"Breathe out as you lift, pause at the top, then breathe in as you lower slowly.",
	"Each lift to the level line counts one; one side, press Switch sides, then the other.",
}, {
	"Keep your chin tucked and your eyes on your knee so your ribs stay down.",
	"Place the foot so your shin is upright at the top of the lift.",
	"Have a backpack? Load it to 5-10 kg: a 1-L water bottle or a hardback is about 1 kg.",
})
implement("hip_thrust_sl", "dumbbells",
	"Shoulders on a sofa or a chair without wheels; a 5-10 kg dumbbell on your hips.", {
	"Sit on the floor with your upper back on the sofa edge or a chair pushed back to a wall.",
	"Plant one foot flat, the other leg out straight, the dumbbell across your hips.",
	"Push through the flat heel until your knee, hips and shoulders form one level line.",
	"Breathe out as you lift, pause at the top, then breathe in as you lower slowly.",
	"Each lift to the level line counts one; one side, press Switch sides, then the other.",
})
implement("hip_thrust_sl", "kettlebell",
	"Shoulders on a sofa or a chair without wheels; a 5-10 kg kettlebell on your hips.", {
	"Sit on the floor with your upper back on the sofa edge or a chair pushed back to a wall.",
	"Plant one foot flat, the other leg out straight, the bell held on your hips by its horns.",
	"Push through the flat heel until your knee, hips and shoulders form one level line.",
	"Breathe out as you lift, pause at the top, then breathe in as you lower slowly.",
	"Each lift to the level line counts one; one side, press Switch sides, then the other.",
})
move("bed_single_leg_bridge", "Single-leg bridge on the bed",
	"Lie on a bed or sofa, one foot flat, the other leg straight.",
	"Push through your heel, hips up, lower slowly. Count out loud, breathe out up.",
	"Stop when your lower back arches or your hips drop. Count the weaker side.")
guide("bed_single_leg_bridge", {
	"Lie on your back on the bed or sofa, knees bent and heels near your seat.",
	"Straighten one leg in line with the other thigh, arms flat at your sides.",
	"Push through the heel of the bent leg and lift your hips until body and thigh line up.",
	"Hold a beat at the top, then lower slowly, breathing out as you lift.",
	"Each lift counts as one rep; do one side, press Switch sides, then the other.",
}, {
	"Press the heel down hard: a soft mattress makes your hips work to stay level.",
	"Keep both hips at the same height as you rise.",
	"Squeeze your seat at the top rather than arching your back.",
})
move("bed_hamstring_walkout", "Hamstring walkout on the bed", "Lie on a bed or sofa, heels down, hips lifted.",
	"Walk your heels out in small steps, then back. Count out loud, breathe steadily.",
	"Stop when your hips drop or the backs of your thighs cramp.")
guide("bed_hamstring_walkout", {
	"Lie on your back on the bed or sofa, knees bent, heels near your seat.",
	"Lift your hips into a bridge, arms flat at your sides.",
	"Walk your heels away in small steps until your legs are almost straight, hips up.",
	"Walk them back in, keeping the hips up, and breathe steadily throughout.",
	"Each walk out and back counts as one rep.",
}, {
	"Take small steps: the further out you go, the harder the hold.",
	"Keep your hips lifted the whole set and lower them only at the end.",
	"Dig your heels into the mattress to feel the backs of your thighs.",
})
move("backpack_deadlift", "Deadlift with a weight",
	"A weight of 10-20 kg on the floor in front of you: a bag, a jug, a kettlebell.",
	"Hips back, back flat, lift it, stand tall. Count out loud, breathe out as you stand.",
	"Stop when your back rounds.")
guide("backpack_deadlift", {
	"Any weight you can grip: a bag of books or bottles, a full jug, a kettlebell or dumbbells.",
	"Nothing near? Lift a light chair without wheels by the sides of its seat.",
	"Stand close to it, feet hip-width, push your hips back and grip it with a flat back.",
	"Stand up tall, breathing out, then push your hips back to lower it.",
	"One rep is from below your knees up to standing tall, so count at the top.",
}, {
	"Pick a weight where 10 reps are work and 20 hard: the game says when to go heavier.",
	"1 L of water weighs 1 kg, a hardback about 1 kg: most people start at 10-20 kg.",
	"Keep the weight close so it brushes your legs all the way down and up.",
})
implement("backpack_deadlift", "kettlebell", "A kettlebell of 10-20 kg on the floor in front of you.", {
	"Set the kettlebell on the floor between your feet, its handle in line with your ankles.",
	"Stand close to it, feet hip-width, push your hips back and grip the handle, back flat.",
	"Stand up tall, breathing out, then push your hips back to lower it.",
	"One rep is from below your knees up to standing tall, so count at the top.",
})
implement("backpack_deadlift", "dumbbells", "Two dumbbells, 10-20 kg together, on the floor in front of you.", {
	"Set the dumbbells on the floor just outside your feet, handles in line with your ankles.",
	"Stand between them, feet hip-width, push your hips back and grip both, back flat.",
	"Stand up tall, breathing out, then push your hips back to lower them.",
	"One rep is from below your knees up to standing tall, so count at the top.",
})
move("backpack_sl_rdl", "Single-leg deadlift with a weight",
	"A weight of 5-15 kg in one hand, the desk within reach.",
	"Hinge on one leg, the weight hanging, back flat. Count out loud, breathe out as you rise.",
	"Stop when your hips twist or your back rounds. Count the weaker side.")
guide("backpack_sl_rdl", {
	"Stand side-on to the desk and hold the weight by its handle in the hand away from it.",
	"Stand on the leg nearer the desk, knee soft, the nearer hand's fingertips on its edge.",
	"Tip from the hip as the free leg reaches back and the weight sinks low.",
	"Breathe in as you tip forward and breathe out as you squeeze your seat to stand tall.",
	"Count each stand; every rep on one leg, press Switch sides, turn round and swap hands.",
}, {
	"Pick a weight where 10 reps a leg are work and 20 hard: the game says when to go heavier.",
	"A bag of books or bottles, a full jug, a kettlebell or dumbbell: 1 L of water weighs 1 kg.",
	"Keep a straight line from your head to your back heel, like a seesaw.",
})
implement("backpack_sl_rdl", "dumbbells", "A dumbbell of 5-15 kg in one hand, the desk within reach.")
implement("backpack_sl_rdl", "kettlebell", "A kettlebell of 5-15 kg in one hand, the desk within reach.")
move("backpack_sl_rdl_slow", "Slow single-leg deadlift with a weight",
	"A thick book to stand on, a weight of 5-15 kg in one hand, the desk within reach.",
	"Lower for 3 seconds on one leg, back flat. Count out loud, breathe out as you rise.",
	"Stop when you lose the 3-s lowering or your back rounds. Count the weaker side.")
guide("backpack_sl_rdl_slow", {
	"Lay a thick book flat beside the desk and stand on it with the leg nearer the desk.",
	"Hold the weight in the hand away from the desk and rest the other hand's fingertips on it.",
	"Tip from the hip for a slow 3 s, free leg back, until the weight nearly brushes the floor.",
	"Breathe in as you lower and breathe out as you squeeze your seat to stand tall.",
	"Count each stand; every rep on one leg, press Switch sides, turn round and swap hands.",
}, {
	"Take your single-leg deadlift's weight: 10 slow reps a leg should be work, 20 hard.",
	"The book lets the weight sink lower, so stop where your back starts to round.",
	"Pick a hardback that lies flat and does not slide on your floor.",
})
implement("backpack_sl_rdl_slow", "dumbbells",
	"A thick book to stand on, a 5-15 kg dumbbell in one hand, the desk within reach.")
implement("backpack_sl_rdl_slow", "kettlebell",
	"A thick book to stand on, a 5-15 kg kettlebell in one hand, the desk within reach.")

-- Push
move("wall_pushup", "Wall push-up", "Hands on the wall at shoulder height, feet a step back.",
	"Body straight, chest to the wall. Breathe out as you push.", "Stop when your hips sag or your elbows flare.")
guide("wall_pushup", {
	"Place your hands flat on the wall at shoulder height, a little wider than your shoulders.",
	"Walk your feet back a step and lean in, body straight from heels to head.",
	"Bend your elbows and bring your chest towards the wall.",
	"Breathe out as you push the wall away until your arms are straight.",
	"Count one rep each time your arms are straight again.",
}, {
	"Point your elbows back at an angle, not straight out to the sides.",
	"Squeeze your seat and belly so your hips move with your chest, not after it.",
	"Spread your fingers and press the whole hand into the wall.",
})
move("scap_pushup", "Scapular push-up plus", "Hands on the desk edge; the desk must not move.",
	"Arms straight, let your chest sink, then push the desk away. Breathe out as you push.",
	"Stop when your elbows bend.")
guide("scap_pushup", {
	"Check the desk cannot slide, then grip its edge with your hands shoulder-width apart.",
	"Walk your feet back and lean in, straight from heels to head, arms fully straight.",
	"With straight elbows, let your chest sink so your shoulder blades squeeze together.",
	"Breathe out and push the desk away so your upper back rounds and the blades spread.",
	"Count one rep each time you reach the top of the push.",
}, {
	"The move is only a few centimetres, so go slowly and feel both end points.",
	"Keep your hips and belly still so the movement comes only from your shoulder blades.",
	"Keep your neck long and your eyes on the desk, with no dropping of the head.",
})
move("desk_pushup", "Desk push-up", "Hands on the desk edge; the desk must not move.",
	"Body straight, chest to the edge. Breathe out as you push.", "Stop when your hips sag or your elbows flare.")
guide("desk_pushup", {
	"Check the desk cannot slide, then grip its edge with your hands shoulder-width apart.",
	"Walk your feet back until your body forms a straight line from heels to head.",
	"Bend your elbows and lower your chest to the desk edge.",
	"Breathe out as you push the desk away until your arms are straight.",
	"Count one rep each time your arms are straight again.",
}, {
	"Aim your chest at the edge, not your chin or belly, so your body stays in one line.",
	"Tuck your elbows so they point back towards your hips, not out to the sides.",
	"Brace your belly and squeeze your seat before each rep so your hips stay in line.",
})
move("knee_pushup", "Knee push-up", "Knees on a folded towel if the floor is hard, hands under your shoulders.",
	"Knees to head in one line, chest to the floor. Count out loud, breathe out as you push.",
	"Stop when your hips sag or your elbows flare.")
guide("knee_pushup", {
	"Kneel on the floor and place your hands flat under your shoulders, fingers forward.",
	"Walk your knees back until your body forms a straight line from knees to head.",
	"Bend your elbows and lower your chest until it is just above the floor.",
	"Breathe out as you push the floor away until your arms are straight.",
	"Count each rep out loud as your arms straighten.",
}, {
	"Keep your hips in the line, so they neither sag down nor stick up.",
	"Tuck your elbows so they point back towards your hips, not out to the sides.",
	"Spread your fingers wide and press through the whole hand, not just the heel of it.",
})
move("chair_pushup", "Low incline push-up", "Hands on a chair without wheels, its back against the wall.",
	"Body straight, chest to the seat edge. Breathe out as you push.",
	"Stop when your hips sag or your elbows flare.")
guide("chair_pushup", {
	"Check the chair sits firm against the wall, then grip the front edge of the seat.",
	"Walk your feet back until your body forms a straight line from heels to head.",
	"Bend your elbows and lower your chest to the seat edge.",
	"Breathe out as you push the seat away until your arms are straight.",
	"Count one rep each time your arms are straight again.",
}, {
	"Place your hands near the front corners of the seat, about shoulder-width apart.",
	"Look at the floor just past the seat so your head stays in line with your body.",
	"Tuck your elbows back towards your hips and squeeze your seat on every rep.",
})
move("desk_pushup_slow", "Desk push-up, 3-s lowering", "Hands on the desk edge; the desk must not move.",
	"Lower for 3 seconds, push up in 1. Count out loud, breathe out as you push.",
	"Stop when you lose the 3-s lowering or your hips sag.")
guide("desk_pushup_slow", {
	"Check the desk cannot slide, then grip its edge with your hands shoulder-width apart.",
	"Walk your feet back until your body forms a straight line from heels to head.",
	"Lower your chest to the desk edge for a slow count of three.",
	"Breathe out and push back up to straight arms in one count.",
	"Count each rep out loud as your arms straighten, then start the next slow lowering.",
}, {
	"Count the three out loud so the last part of the lowering does not rush.",
	"Breathe in slowly on the way down, never hold your breath to stay tight.",
	"Squeeze your seat and belly so your body lowers as one straight piece.",
})
move("negative_pushup", "Slow push-up, knees down", "On the floor, hands under your shoulders.",
	"Lower for 3 seconds from your toes, knees down, push up. Count out loud, breathe out up.",
	"Stop when you lose the 3-s lowering or your hips sag.")
guide("negative_pushup", {
	"Place your hands flat on the floor under your shoulders and step back onto your toes.",
	"Hold a straight line from heels to head with your belly and seat tight.",
	"Lower your chest towards the floor for a slow count of three, staying straight.",
	"Set your knees down, then breathe out and push up to straight arms from your knees.",
	"Lift your knees to come back onto your toes, and count each slow lowering as one rep.",
}, {
	"Make the last part of the lowering as slow as the first, since it is the hardest part.",
	"Tuck your elbows back towards your hips as you lower.",
	"Breathe in slowly as you lower and out as you push, never holding your breath.",
})
move("step_pushup", "Step push-up", "Hands on the bottom stair or a step that cannot slide.",
	"Body straight, chest to the edge. Count out loud, breathe out as you push.",
	"Stop when your hips sag or your elbows flare.")
guide("step_pushup", {
	"Put your hands near the front edge of the bottom stair or step, shoulder-width apart.",
	"Walk your feet back until your body forms a straight line from heels to head.",
	"Bend your elbows and lower your chest to the stair edge.",
	"Breathe out as you push the stair away until your arms are straight.",
	"Count each rep out loud as your arms straighten.",
}, {
	"Keep your whole palm on the step, not hanging over its edge.",
	"Look at the step just ahead of your hands so your neck stays in line.",
	"Brace your belly and squeeze your seat so your hips never dip towards the floor.",
})
move("pushup", "Push-up", "On the floor, hands under your shoulders.",
	"Body straight, chest to a fist's height. Count out loud, breathe out as you push.",
	"Stop when your hips sag or your elbows flare.")
guide("pushup", {
	"Place your hands flat on the floor under your shoulders, fingers pointing forward.",
	"Step back onto your toes, feet hip-width apart, body straight from heels to head.",
	"Bend your elbows and lower your chest to about a fist above the floor.",
	"Breathe out as you push the floor away until your arms are straight.",
	"Count each rep out loud as your arms straighten.",
}, {
	"Tuck your elbows so they point back towards your hips, at about 45 degrees.",
	"Squeeze your seat and brace your belly before each rep, and keep them tight.",
	"Grip the floor with your fingers and press through the whole hand.",
})
move("pushup_hold", "Push-up hold", "On the floor, hands under your shoulders.",
	"Lower halfway and hold, body straight. Count out loud.", "Stop when your hips sag or your arms shake hard.")
guide("pushup_hold", {
	"Place your hands flat on the floor under your shoulders, fingers pointing forward.",
	"Step back onto your toes and make a straight line from heels to head.",
	"Lower your chest halfway, until your elbows are bent to about a right angle.",
	"Hold still there, breathing slowly in and out and counting out loud.",
	"The clock times how long you hold the halfway point, then set your knees down.",
}, {
	"Keep your elbows tucked back by your ribs, not out wide.",
	"Press the floor away so your shoulders stay up and your chest does not sink.",
	"Breathe out on every count so your breath keeps moving through the hold.",
})
move("feet_up_pushup", "Feet-elevated push-up", "Toes on the edge of a bed or sofa, hands on the floor.",
	"Body straight, chest to the floor. Count out loud, breathe out as you push.",
	"Stop when your hips sag or your elbows flare.")
guide("feet_up_pushup", {
	"Push the bed or sofa with a foot to check it stays put, then kneel with your back to it.",
	"Put your hands under your shoulders and set your toes on its edge, one foot at a time.",
	"Lift into a straight line from heels to head, hips level with your shoulders.",
	"Lower your chest towards the floor, then breathe out as you push back up.",
	"Count each rep out loud as your arms straighten.",
}, {
	"Keep your hips level with your shoulders and feet, not up in the air.",
	"Let your chest lead towards the floor, with your eyes just ahead of your hands.",
	"Tuck your elbows back towards your hips as you lower.",
})
move("pushup_slow", "Push-up, 3-s lowering", "On the floor, hands under your shoulders.",
	"Lower for 3 seconds, push up in 1. Count out loud, breathe out as you push.",
	"Stop when you lose the 3-s lowering or your hips sag.")
guide("pushup_slow", {
	"Place your hands flat on the floor under your shoulders, fingers pointing forward.",
	"Step back onto your toes, feet hip-width apart, body straight from heels to head.",
	"Lower your chest to about a fist above the floor for a slow count of three.",
	"Breathe out and push up to straight arms in one count.",
	"Count each rep out loud as your arms straighten, then start the next slow lowering.",
}, {
	"Count one, two, three out loud so the last part of the lowering stays slow.",
	"Breathe in slowly as you lower, never hold your breath to stay tight.",
	"Keep your body stiff like a plank so hips and chest reach the bottom together.",
})
-- The moves in between that the climb waits for (2026-10-10)
move("close_pushup", "Close push-up", "On the floor, hands together under your chest.",
	"Elbows brush your ribs, chest to your hands. Count out loud, breathe out as you push.",
	"Stop when your elbows flare or your hips sag.")
guide("close_pushup", {
	"Place your hands on the floor under your chest, close enough that they touch.",
	"Step back onto your toes, feet hip-width apart, body straight from heels to head.",
	"Lower your chest to your hands, your elbows brushing your ribs.",
	"Breathe out as you push the floor away until your arms are straight.",
	"Count each rep out loud as your arms straighten.",
}, {
	"Keep your elbows pointing back along your sides, never out to the sides.",
	"Turn your fingers a little out if your hands feel cramped this close together.",
	"Squeeze your seat and brace your belly so your hips stay level with your shoulders.",
})
move("incline_one_arm_pushup", "One-arm push-up, hand on the bed",
	"One hand on the edge of a bed or sofa under your chest, feet wide behind.",
	"Lower and push up on one arm, body straight. Count out loud, breathe out as you push.",
	"Stop when your hips twist or sag. Count the weaker side.")
guide("incline_one_arm_pushup", {
	"Push the bed or sofa to check it stays put, then stand facing its long edge.",
	"Place one hand on the edge in line with the middle of your chest, the other on your back.",
	"Walk your feet back and wide until your body is one straight line from heels to head.",
	"Lower your chest toward the edge, then push it away, breathing out.",
	"Each lower and push counts one rep; do one side, press Switch sides, then the other.",
}, {
	"Spread your feet wider than your hips: a wide base stops your body twisting.",
	"Keep the working elbow close to your side, pointing back toward your hip.",
	"Put your hand where the edge is firmest, near the frame, fingers spread.",
})
-- The ladder gaps' push-ups (2026-10-12): every rep on one side, then Switch sides and the other
move("staggered_pushup", "Staggered push-up",
	"On the floor; one hand under your chest, the other two hand-lengths out from your side.",
	"Straight down toward the inner hand, which does the work. Count out loud, breathe out up.",
	"Stop when your hips twist or you drift to the outer hand. Count the weaker side.")
guide("staggered_pushup", {
	"Place one hand flat under the middle of your chest, fingers pointing forward.",
	"Set the other hand two hand-lengths out from your side, fingers turned out a little.",
	"Step back onto your toes, feet hip-width apart, body straight from heels to head.",
	"Lower your chest straight down toward the inner hand, then push back up.",
	"Count each rep out loud; every rep on one side, then press Switch sides and swap hands.",
}, {
	"The inner hand does most of the push; the outer hand only steadies you.",
	"Go straight down, never sideways toward the outer hand.",
	"Too hard? Move the outer hand a little closer to your side.",
})
move("sliding_one_arm_pushup", "Sliding one-arm push-up",
	"A hard floor, a small towel under the helping hand (on carpet, a hardcover book).",
	"Down over the working hand, towel hand sliding out. Count out loud, breathe out up.",
	"Stop when your hips twist or the towel hand starts to push. Count the weaker side.")
guide("sliding_one_arm_pushup", {
	"Fold a small towel under one hand; on carpet, put that hand on a hardcover book.",
	"Set the other hand under your chest and walk your feet back and wide, body straight.",
	"Lower your chest slowly toward the working hand as the towel hand slides out.",
	"Breathe out and push straight up with the working arm, the towel hand sliding back in.",
	"Count each rep out loud; every rep on one arm, then press Switch sides and swap arms.",
}, {
	"The towel hand only steadies you: the less it presses, the harder the rep.",
	"Spread your feet wider than your hips so your body does not twist.",
	"Keep the working elbow close to your side, pointing back toward your hip.",
})
move("pike_pushup", "Pike push-up", "On the floor, hands wide, hips high.",
	"Head toward the floor between your hands, push back up. Count out loud, breathe out up.",
	"Stop when your elbows flare.")
guide("pike_pushup", {
	"Place your hands on the floor a little wider than your shoulders, fingers forward.",
	"Walk your feet in and lift your hips high so your body makes an upside-down V.",
	"Bend your elbows and lower the top of your head towards the floor between your hands.",
	"Breathe out as you push back up until your arms are straight, hips staying high.",
	"Count each rep out loud as your arms straighten.",
}, {
	"Keep your hips high the whole time so the push goes up through your shoulders.",
	"Let your elbows point back towards your knees, not out to the sides.",
	"Lower slowly so your head stops just short of the floor, never bumping it.",
})
L["move.pike_pushup.hint"] = "Bend your knees if your legs are tight."
move("archer_pushup", "Archer push-up", "On the floor, hands wide.",
	"Lower toward one hand, the other arm straight, alternate. Count out loud, breathe out up.",
	"Stop when your hips twist or sag. A push-up to each side counts one.")
guide("archer_pushup", {
	"Place your hands on the floor very wide, far outside your shoulders, fingers turned out.",
	"Step back onto your toes, feet hip-width apart, body straight from heels to head.",
	"Bend one elbow and lower your chest towards that hand, keeping the other arm straight.",
	"Breathe out and push back up to the middle, then lower towards the other hand.",
	"Alternate sides every rep; a push-up to one side and then the other counts one.",
}, {
	"Keep your chest and hips facing the floor, with no twisting towards the bent arm.",
	"Keep the straight arm long and pressing lightly while the bent arm does the work.",
	"Shift your chest sideways over the bent arm as you lower, not only straight down.",
})
move("one_arm_pushup", "One-arm push-up", "On the floor, feet wide.",
	"One hand under your chest, lower and push up. Count out loud, breathe out as you push.",
	"Stop when your hips twist or sag. Count the weaker side.")
guide("one_arm_pushup", {
	"Start on your toes with both hands on the floor, then walk your feet out very wide.",
	"Move one hand under the middle of your chest and rest the other hand on your lower back.",
	"Lower your chest towards the floor, keeping hips and shoulders level.",
	"Breathe out and push the floor away until your arm is straight.",
	"Count each rep out loud; finish one arm, press Switch sides, then do the other.",
}, {
	"Squeeze your seat and brace your belly hard so your hips stay square to the floor.",
	"Keep the working elbow close to your side, pointing back towards your hip.",
	"Spread your fingers wide and grip the floor to steady the hand.",
})
move("desk_archer_pushup", "Desk archer push-up", "Hands wide on a sturdy desk edge, keyboard pushed back, feet back.",
	"Lower toward one hand, the other arm straight, alternate. Count out loud, breathe out up.",
	"Stop when your hips twist or sag. A push-up to each side counts one.")
guide("desk_archer_pushup", {
	"Push the keyboard back and grip the desk edge with your hands twice shoulder-width apart.",
	"Walk your feet back until your body is one straight line from heels to head.",
	"Bend one elbow to lower your chest toward that hand, the other arm staying straight.",
	"Push back up, breathing out, then lower toward the other hand.",
	"Alternate sides every rep; a push-up to one side and then the other counts one.",
}, {
	"Keep the straight arm long and light: it steadies you while the bent arm works.",
	"Turn the working hand's fingers slightly out so the elbow tracks over the wrist.",
	"Keep your hips square to the desk so the effort stays in your chest and arm.",
})
move("desk_one_arm_negative", "One-arm desk lowering",
	"One hand on a sturdy desk edge under your chest, feet wide behind.",
	"Lower on one arm in 3 s, push up with both. Count out loud, breathe out as you push.",
	"Stop when your hips twist or you drop fast. Count the weaker side.")
guide("desk_one_arm_negative", {
	"Place one hand on the desk edge in line with the middle of your chest.",
	"Step your feet back and wide, body straight from heels to head, free hand on your hip.",
	"Lower your chest toward the desk on the one arm, counting three slow beats.",
	"At the bottom, set the free hand down and push up with both, breathing out.",
	"Each slow lowering counts one rep; do one side, press Switch sides, then the other.",
}, {
	"Spread your feet wider than your hips: a wide base stops your body twisting.",
	"Squeeze your seat and belly so your hips stay level with your shoulders.",
	"Keep the working elbow close to your side, not flared out.",
})
move("desk_one_arm_pushup", "One-arm desk push-up",
	"One hand on a sturdy desk edge under your chest, feet wide behind.",
	"Lower and push up on one arm, body straight. Count out loud, breathe out as you push.",
	"Stop when your hips twist or sag. Count the weaker side.")
guide("desk_one_arm_pushup", {
	"Place one hand on the desk edge in line with the middle of your chest.",
	"Walk your feet back and wide until your body is one straight line, free hand on your back.",
	"Lower your chest toward the desk edge, elbow close to your side.",
	"Push the desk away until the arm is straight, breathing out as you push.",
	"Each lower and push counts one rep; do one side, press Switch sides, then the other.",
}, {
	"Brace your belly and squeeze your seat so your body moves as one piece.",
	"Turn your hips slightly toward the working arm if they twist away.",
	"Press the whole palm into the desk, fingers spread, for a firm base.",
})
move("desk_one_arm_slow", "Slow one-arm desk push-up", "One hand on a sturdy desk edge under your chest, feet close.",
	"Three seconds down, pause, push up on one arm. Count out loud, breathe out as you push.",
	"Stop when your hips twist or you hold your breath. Count the weaker side.")
guide("desk_one_arm_slow", {
	"Place one hand on the desk edge in line with the middle of your chest.",
	"Walk your feet back, closer together than for the one-arm push-up, body straight.",
	"Lower toward the desk over three slow beats, then pause for one beat.",
	"Push back up in 1 s, breathing out as you push, never holding your breath.",
	"Each slow rep counts as one; do one side, press Switch sides, then the other.",
}, {
	"Narrow feet make it harder to stay square: keep them together only while hips stay level.",
	"Say the beats out loud so the lowering never speeds up.",
	"Keep the free hand on your back or hip, never on the desk.",
})

-- Pull
move("doorframe_row", "Doorframe row", "Grip a solid doorframe at chest height, feet close to it.",
	"Lean back, arms straight, pull your chest to the frame. Breathe out as you pull.",
	"Stop when your shoulders shrug or your hips sag.")
guide("doorframe_row", {
	"Face the edge of a solid doorframe and grip it at chest height, one hand above the other.",
	"Toes near the frame, lean back until your arms are straight, heels to head in one line.",
	"Pull your chest to the frame, elbows close to your sides, then lower with control.",
	"Breathe out as you pull, breathe in as you lean back.",
	"Each pull to the frame and back to straight arms is one rep.",
}, {
	"Start each pull by drawing your shoulder blades back, then bend your arms.",
	"Keep your hips in line with your shoulders so your whole body moves as one piece.",
	"Keep your heels planted and your neck long, eyes on the frame.",
})
move("doorframe_iso", "Doorframe isometric row", "Grip a solid doorframe at chest height.",
	"Pull as if to row, elbows back, and hold. Count out loud.", "Stop when your shoulders shrug or you hold your breath.")
guide("doorframe_iso", {
	"Face the edge of a solid doorframe and grip it at chest height, one hand above the other.",
	"Toes close to the frame, lean back a little, heels to head in one straight line.",
	"Pull until your elbows bend to a right angle, elbows back, and hold still there.",
	"Keep breathing slowly and steadily through the whole hold.",
	"Count out loud while the clock times the hold, from your first pull until you let go.",
}, {
	"Squeeze your shoulder blades together and keep them there the whole time.",
	"Keep your hips in line, as if your body were a stiff board leaning off the frame.",
	"Keep your shoulders low and wide, away from your ears.",
})
move("backpack_row", "One-arm row", "One hand on the desk, a weight of 5-15 kg in the other: a bag, a jug.",
	"Back flat, pull the weight to your hip. Count out loud, breathe out as you pull.",
	"Stop when your back rounds or twists. Count the weaker side.")
guide("backpack_row", {
	"Any weight with a handle: a bag of books or bottles, a full jug, a kettlebell or dumbbell.",
	"Nothing near? Hold a light chair without wheels upright by the side of its seat.",
	"Face the desk, free hand flat on it, step back on the weight's side, back flat.",
	"Pull the weight to your hip, elbow close to your side, then lower it with control.",
	"Breathe out as you pull, count each rep; every rep on one side, then press Switch sides.",
}, {
	"Pick a weight where 10 reps are work and 20 hard: the game says when to go heavier.",
	"1 L of water weighs 1 kg, a hardback about 1 kg: most people start at 5-15 kg.",
	"Drive your elbow towards your back pocket, not straight up to the ceiling.",
})
implement("backpack_row", "dumbbells", "One hand on the desk, a dumbbell of 5-15 kg in the other.", {
	"Face the desk, free hand flat on it, step back on the dumbbell's side, back flat.",
	"Let the dumbbell hang under your shoulder, arm long, palm facing in.",
	"Pull it to your hip, elbow close to your side, then lower it with control.",
	"Breathe out as you pull, count each rep; every rep on one side, then press Switch sides.",
})
implement("backpack_row", "kettlebell", "One hand on the desk, a kettlebell of 5-15 kg in the other.", {
	"Face the desk, free hand flat on it, step back on the kettlebell's side, back flat.",
	"Let the bell hang under your shoulder by its handle, arm long.",
	"Pull it to your hip, elbow close to your side, then lower it with control.",
	"Breathe out as you pull, count each rep; every rep on one side, then press Switch sides.",
})
move("ytw_raise", "Y-T-W raises", "Lie face down on the floor, forehead on a towel.",
	"Lift your arms in a Y, a T, then a W, thumbs up. Breathe out as you lift.",
	"Stop when your shoulders shrug or your lower back arches.")
guide("ytw_raise", {
	"Lie face down on the floor, legs long, forehead resting on a folded towel.",
	"Reach your arms overhead in a Y, thumbs up, lift them off the floor, then lower.",
	"Do the same with straight arms out to the sides in a T, then with bent elbows in a W.",
	"Breathe out as you lift and breathe in as you lower.",
	"One Y, one T and one W together make one rep.",
}, {
	"Pull your shoulder blades down and together before your arms leave the floor.",
	"Keep your forehead on the towel so your neck stays long the whole time.",
	"In the W, draw your elbows down towards your ribs as you lift.",
})
-- The doorframe's shoulder-blade pull (2026-10-28)
move("doorframe_scap_pull", "Doorframe shoulder-blade pull",
	"Grip a solid doorframe at chest height, feet close to it.",
	"Arms straight, squeeze your shoulder blades back, release. Breathe out as you squeeze.",
	"Stop when your elbows bend or your shoulders shrug.")
guide("doorframe_scap_pull", {
	"Face the edge of a solid doorframe and grip it at chest height, one hand above the other.",
	"Toes near the frame, lean back until your arms are straight, heels to head in one line.",
	"Keeping the arms straight, pull your shoulder blades back and down, lifting your chest.",
	"Breathe out as you squeeze, hold a beat, then let your shoulders slide forward again.",
	"Each squeeze counts as one rep.",
}, {
	"Think of putting your shoulder blades in your back pockets.",
	"Never bend the elbows: the move is all in the shoulder blades.",
	"Grip only where your whole hand closes around the frame, never trim with fingertips.",
})
move("inverted_row", "Inverted row under a table",
	"A heavy table on four legs; never glass, folding, a standing desk or one with monitors.",
	"Pull your chest to the edge, lower to straight arms. Count out loud, breathe out up.",
	"Stop when your hips sag or the table moves.")
-- Under a table of 71-76 cm the back meets the floor about when the arms are straight (2026-10-14, David: "The
-- animation looks like you constantly have to hold yourself without resting on the floor inbetween?"): each rep
-- touches down and goes again, never a hover on bent arms
guide("inverted_row", {
	"Lean on the edge, then pull gently: if the table lifts, slides or creaks, stop.",
	"Lie under it, chest below the edge, and grip it a little wider than your shoulders.",
	"Heels on the floor, lift your hips so your body is straight from heels to head.",
	"Pull your chest to the edge, then lower until your arms are fully straight.",
	"Let your back touch the floor lightly, then pull again. Count each rep out loud.",
}, {
	"Squeeze your bottom and belly so your hips rise with your chest, not after it.",
	"Drive your elbows down towards the floor and squeeze your shoulder blades at the top.",
	"Keep your neck long, chin slightly tucked, eyes on the underside of the table.",
})
-- Its easier version, the knees bent (2026-10-13)
move("table_row_bent", "Table row, knees bent",
	"A heavy table on four legs; never glass, folding, a standing desk or one with monitors.",
	"Knees bent, feet flat, pull your chest to the edge. Count out loud, breathe out up.",
	"Stop when your hips sag or the table moves.")
guide("table_row_bent", {
	"Lean on the edge, then pull gently: if the table lifts, slides or creaks, stop.",
	"Lie under it, chest below the edge, and grip it a little wider than your shoulders.",
	"Bend your knees, feet flat, and lift your hips so knees, hips and head make one line.",
	"Pull your chest to the edge, then lower until your arms are fully straight.",
	"Your upper back nears the floor; keep your hips up and pull again. Count out loud.",
}, {
	"Squeeze your bottom and belly so your hips rise with your chest, not after it.",
	"Drive your elbows down towards the floor and squeeze your shoulder blades at the top.",
	"Keep your neck long, chin slightly tucked, eyes on the underside of the table.",
})
-- Its easier version, the lowering (2026-10-14)
move("table_row_lowering", "Table row lowering, knees bent",
	"A heavy table on four legs; never glass, folding, a standing desk or one with monitors.",
	"Push with your feet to the edge, lower in 3 seconds. Count out loud, breathe out down.",
	"Stop when you drop fast or the table moves.")
guide("table_row_lowering", {
	"Lean on the edge, then pull gently: if the table lifts, slides or creaks, stop.",
	"Lie under it, grip the edge a little wider than your shoulders, knees bent, feet flat.",
	"Push through your feet and pull until your chest reaches the edge.",
	"Lower on your arms over three slow beats, hips up, until your arms are straight.",
	"Rest your hips on the floor, then go again. Count each slow lowering out loud.",
}, {
	"Make the last part of the lowering as slow as the first, since it is the hardest part.",
	"Keep your knees, hips and head in one line all the way down.",
	"Keep your neck long, chin slightly tucked, eyes on the underside of the table.",
})
move("backpack_row_slow", "One-arm row, slow lowering",
	"One hand on the desk, a weight of 5-15 kg in the other: a bag, a jug.",
	"Pull it to your hip, lower it in 3 seconds. Count out loud, breathe out as you pull.",
	"Stop when you lose the 3-s lowering. Count the weaker side.")
guide("backpack_row_slow", {
	"The one-arm row's weight: a bag of books or bottles, a full jug, a kettlebell or dumbbell.",
	"Face the desk, free hand flat on it, step back on the weight's side, back flat.",
	"Pull the weight to your hip in 1 s, then lower it for a full 3 s to a long arm.",
	"Breathe out as you pull, breathe in slowly as you lower.",
	"Count each rep out loud; every rep on one side, then press Switch sides and swap hands.",
}, {
	"Count the lowering in your head, one and two and three, so it never speeds up.",
	"10 slow reps should be work and 20 hard: the game says when to go heavier.",
	"Nothing near? Hold a light chair without wheels upright by the side of its seat.",
})
implement("backpack_row_slow", "dumbbells", "One hand on the desk, a dumbbell of 5-15 kg in the other.", {
	"Face the desk, free hand flat on it, step back on the dumbbell's side, back flat.",
	"Pull the dumbbell to your hip in 1 s, then lower it for a full 3 s to a long arm.",
	"Breathe out as you pull, breathe in slowly as you lower.",
	"Count each rep out loud; every rep on one side, then press Switch sides and swap hands.",
})
implement("backpack_row_slow", "kettlebell", "One hand on the desk, a kettlebell of 5-15 kg in the other.", {
	"Face the desk, free hand flat on it, step back on the kettlebell's side, back flat.",
	"Pull the bell to your hip in 1 s, then lower it for a full 3 s to a long arm.",
	"Breathe out as you pull, breathe in slowly as you lower.",
	"Count each rep out loud; every rep on one side, then press Switch sides and swap hands.",
})
move("wall_angels", "Wall angels", "Back, head and hips against the wall.",
	"Slide your arms up and down the wall like a snow angel. Breathe slowly.",
	"Stop when your back leaves the wall.")
guide("wall_angels", {
	"Stand with your heels a foot's length from the wall, knees soft.",
	"Press your bottom, upper back and the back of your head to the wall.",
	"Arms in a W on the wall, slide them up until straight by your ears, then back down.",
	"Breathe slowly as your arms slide up and back down.",
	"Move at an easy pace for the time shown.",
}, {
	"Keep your ribs down so your lower back stays close to the wall as your arms rise.",
	"Slide only as high as your hands can stay within a fist of the wall.",
	"Tuck your chin slightly so the back of your head stays on the wall.",
})
move("chest_opener", "Clasped-hands chest opener", "Stand tall, hands behind your back.",
	"Clasp your hands, straighten your arms, lift them away from your back. Breathe slowly.",
	"Ease off if the front of a shoulder pinches.")
guide("chest_opener", {
	"Stand tall with your feet hip-width apart.",
	"Reach your hands behind you and clasp your fingers.",
	"Straighten your arms and draw your shoulders back and down.",
	"Lift your hands away from your back, chest open, breathing slowly.",
	"Stay for the time shown, then let your arms down slowly.",
}, {
	"Hold a towel between your hands if they do not meet.",
	"Keep your ribs down so your low back does not arch.",
	"Lift only as high as your shoulders stay down.",
})
L["move.chest_opener.mark"] = "Hands clasped, arms straight, hands a hand's width off your back, chest up."
-- The one-arm doorframe row with the toes a foot back, and the one-arm row's hold (2026-10-28): rung 3 without a table
move("one_arm_doorframe_row_back", "One-arm doorframe row, feet back",
	"One hand on a solid doorframe edge at chest height, toes a foot's length from its base.",
	"Pull your chest to the frame on one arm. Count out loud, breathe out as you pull.",
	"Stop when your hand slips or your hips twist. Count the weaker side.")
guide("one_arm_doorframe_row_back", {
	"Face the frame's edge on the side without hinges and wrap one hand around it.",
	"Set your toes one foot's length from the base and lean back until the arm is straight.",
	"Pull your chest to the frame, elbow close to your side, then lower slowly.",
	"Breathe out as you pull and in as you lean back.",
	"Each pull counts as one rep; do one side, press Switch sides, then the other.",
}, {
	"Measure the gap once with your own foot and stand there every time.",
	"Keep the free hand hovering by the frame, ready to catch you.",
	"Grip only where your whole hand closes around the frame, never trim with fingertips.",
})
move("one_arm_doorframe_hold", "One-arm doorframe row hold",
	"One hand on a solid doorframe edge at chest height, toes a hand's length from its base.",
	"Pull to the frame on one arm and hold, elbow back. Count out loud, keep breathing.",
	"Stop when your hand slips or your chest caves in. Count the weaker side.")
guide("one_arm_doorframe_hold", {
	"Face the frame's edge on the side without hinges and wrap one hand around it.",
	"Set your toes a hand's length from the base and lean back until the arm is straight.",
	"Pull your chest to the frame and hold there, elbow back and shoulder down.",
	"Keep breathing slowly and count out loud through the whole hold.",
	"The clock counts up: hold until your chest sinks, then press Switch sides.",
}, {
	"Keep the free hand hovering by the frame, ready to catch you.",
	"Draw the shoulder blade back and down and keep it there the whole hold.",
	"Grip only where your whole hand closes around the frame, never trim with fingertips.",
})
move("one_arm_doorframe_row", "One-arm doorframe row",
	"One hand on a solid doorframe edge at chest height, toes a hand's length from its base.",
	"Pull your chest to the frame on one arm. Count out loud, breathe out as you pull.",
	"Stop when your hand slips or your hips twist. Count the weaker side.")
guide("one_arm_doorframe_row", {
	"Face the frame's edge on the side without hinges and wrap one hand around it.",
	"Set your toes a hand's length from the base and lean back until the arm is straight.",
	"Pull your chest to the frame, elbow close to your side, then lower slowly.",
	"Breathe out as you pull and in as you lean back.",
	"Each pull counts as one rep; do one side, press Switch sides, then the other.",
}, {
	"Keep the free hand hovering by the frame, ready to catch you.",
	"Start each pull by drawing the shoulder blade back, then bend the arm.",
	"Grip only where your whole hand closes around the frame, never trim with fingertips.",
})
move("steep_doorframe_row", "Steep one-arm doorframe row",
	"One hand around a solid doorframe edge, toes against its base.",
	"Lean back far on one arm, pull, lower in 3 s. Count out loud, breathe out as you pull.",
	"Stop when your hand creeps down the frame or your hips twist. Count the weaker side.")
guide("steep_doorframe_row", {
	"Face the frame's edge on the side without hinges and wrap one hand around it.",
	"Put your toes against the base and lean back until the arm is straight.",
	"Pull your chest to the frame in 1 s, then lower over three slow beats.",
	"Breathe out as you pull and in as you lower.",
	"Each pull counts as one rep. Do every rep on one side, then press Switch sides.",
}, {
	"Keep your toes against the base every set; when your hand creeps, the set is done.",
	"Keep your body straight from heels to head, hips in line.",
	"Rest the free hand on your belly, ready to grab the frame if your grip slips.",
})
-- The table's rows with the heels up (2026-10-28): rungs 6-8
move("table_row_feet_up", "Table row, feet up",
	"A heavy table on four legs; heels up on a sofa, a bed or a chair without wheels.",
	"Heels up, body flat, pull your chest to the edge. Count out loud, breathe out up.",
	"Stop when your hips sag or the table moves.")
guide("table_row_feet_up", {
	"Lean on the edge, then pull gently: if the table lifts, slides or creaks, stop.",
	"Set a sofa, a bed or a chair without wheels in line with the table, your height away.",
	"Lie under the table, grip the edge a little wider than your shoulders, heels on the seat.",
	"Lift your hips so your body is flat, then pull your chest to the edge.",
	"Lower until your arms are straight, then pull again. Count each rep out loud.",
}, {
	"Squeeze your bottom and belly so your hips rise with your chest, not after it.",
	"Drive your elbows down towards the floor and squeeze your shoulder blades at the top.",
	"Higher heels make it harder: use the same seat each time so your record stays fair.",
})
move("table_row_feet_up_pack", "Table row, feet up, with a backpack",
	"A heavy table on four legs; a backpack of 5-15 kg worn on your chest, heels up.",
	"Pack on your chest, body flat, pull to the edge. Count out loud, breathe out up.",
	"Stop when your hips sag or the table moves.")
guide("table_row_feet_up_pack", {
	"Lean on the edge, then pull gently: if the table lifts, slides or creaks, stop.",
	"Load a backpack with books or bottles and wear it back to front, on your chest.",
	"Lie under the table, grip the edge, and rest your heels on the sofa, bed or chair.",
	"Lift your hips so your body is flat, then pull your chest to the edge.",
	"Lower until your arms are straight, then pull again. Count each rep out loud.",
}, {
	"Pick a load where 3 reps are work and 12 hard: the game says when to go heavier.",
	"1 L of water weighs 1 kg, a hardback about 1 kg: most people start at 5 kg.",
	"Squeeze your bottom and belly so your hips rise with your chest, not after it.",
})
implement("table_row_feet_up_pack", "vest",
	"A heavy table on four legs; a weighted vest of 5-15 kg, heels up on a sofa or bed.", {
	"Lean on the edge, then pull gently: if the table lifts, slides or creaks, stop.",
	"Put on the weighted vest, snug and even.",
	"Lie under the table, grip the edge, and rest your heels on the sofa, bed or chair.",
	"Lift your hips so your body is flat, then pull your chest to the edge.",
	"Lower until your arms are straight, then pull again. Count each rep out loud.",
})
move("table_row_feet_up_pause", "Paused table row, feet up",
	"A heavy table on four legs; a backpack of 5-15 kg worn on your chest, heels up.",
	"Pull your chest to the edge, hold 2 seconds, lower. Count out loud, breathe steadily.",
	"Stop when your hips sag, the table moves or you lose the 2-s hold.")
guide("table_row_feet_up_pause", {
	"Lean on the edge, then pull gently: if the table lifts, slides or creaks, stop.",
	"Wear the loaded pack on your chest, straps tight, heels up on the sofa, bed or chair.",
	"Lie under the table, grip the edge, and lift your hips so your body is flat.",
	"Pull your chest to the edge, breathing out, and hold it there for 2 s.",
	"Lower until your arms are straight, then pull again. Count each rep out loud.",
}, {
	"Say the two beats out loud at the top, which keeps you breathing.",
	"At the top squeeze your shoulder blades and keep your elbows close to your ribs.",
	"Keep the same load and seat each time: the game says when to go heavier.",
})
implement("table_row_feet_up_pause", "vest",
	"A heavy table on four legs; a weighted vest of 5-15 kg, heels up on a sofa or bed.", {
	"Lean on the edge, then pull gently: if the table lifts, slides or creaks, stop.",
	"Put on the weighted vest, snug and even, heels up on the sofa, bed or chair.",
	"Lie under the table, grip the edge, and lift your hips so your body is flat.",
	"Pull your chest to the edge, breathing out, and hold it there for 2 s.",
	"Lower until your arms are straight, then pull again. Count each rep out loud.",
})
-- The same rows on rings hung low (2026-10-28): rungs 6-8 for rings without a table
move("ring_row_feet_up", "Ring row, feet up",
	"Rings or straps hung low; heels up on a sofa, a bed or a chair without wheels.",
	"Heels up, body flat, pull your chest to your hands. Count out loud, breathe out up.",
	"Stop when your hips sag.")
guide("ring_row_feet_up", {
	"Shut and lock any door the straps hang from, then tug both handles hard to test them.",
	"Hang the rings low, so your back just clears the floor when your arms are straight.",
	"Lie under the rings, grip them, and rest your heels on a sofa, a bed or a chair.",
	"Lift your hips so your body is flat, then pull your chest to your hands.",
	"Lower until your arms are straight, then pull again. Count each rep out loud.",
}, {
	"Squeeze your bottom and belly so your hips rise with your chest, not after it.",
	"Squeeze your shoulder blades together at the top and let your palms face each other.",
	"Higher heels make it harder: use the same seat each time so your record stays fair.",
})
move("ring_row_feet_up_pack", "Ring row, feet up, with a backpack",
	"Rings or straps hung low; a backpack of 5-15 kg worn on your chest, heels up.",
	"Pack on your chest, body flat, pull to your hands. Count out loud, breathe out up.",
	"Stop when your hips sag.")
guide("ring_row_feet_up_pack", {
	"Shut and lock any door the straps hang from, then tug both handles hard to test them.",
	"Load a backpack with books or bottles and wear it back to front, on your chest.",
	"Lie under the rings, grip them, and rest your heels on the sofa, bed or chair.",
	"Lift your hips so your body is flat, then pull your chest to your hands.",
	"Lower until your arms are straight, then pull again. Count each rep out loud.",
}, {
	"Pick a load where 3 reps are work and 12 hard: the game says when to go heavier.",
	"1 L of water weighs 1 kg, a hardback about 1 kg: most people start at 5 kg.",
	"Squeeze your bottom and belly so your hips rise with your chest, not after it.",
})
implement("ring_row_feet_up_pack", "vest",
	"Rings or straps hung low; a weighted vest of 5-15 kg, heels up on a sofa or bed.", {
	"Shut and lock any door the straps hang from, then tug both handles hard to test them.",
	"Put on the weighted vest, snug and even.",
	"Lie under the rings, grip them, and rest your heels on the sofa, bed or chair.",
	"Lift your hips so your body is flat, then pull your chest to your hands.",
	"Lower until your arms are straight, then pull again. Count each rep out loud.",
})
move("ring_row_feet_up_pause", "Paused ring row, feet up",
	"Rings or straps hung low; a backpack of 5-15 kg worn on your chest, heels up.",
	"Pull your chest to your hands, hold 2 seconds, lower. Count out loud, breathe steadily.",
	"Stop when your hips sag or you lose the 2-s hold.")
guide("ring_row_feet_up_pause", {
	"Shut and lock any door the straps hang from, then tug both handles hard to test them.",
	"Wear the loaded pack on your chest, pulled snug, heels up on the sofa, bed or chair.",
	"Lie under the rings, grip them, and lift your hips so your body is flat.",
	"Pull your chest to your hands, breathing out, and hold it there for 2 s.",
	"Lower until your arms are straight, then pull again. Count each rep out loud.",
}, {
	"Say the two beats out loud at the top, which keeps you breathing.",
	"At the top squeeze your shoulder blades and keep your elbows close to your ribs.",
	"Keep the same load and seat each time: the game says when to go heavier.",
})
implement("ring_row_feet_up_pause", "vest",
	"Rings or straps hung low; a weighted vest of 5-15 kg, heels up on a sofa or bed.", {
	"Shut and lock any door the straps hang from, then tug both handles hard to test them.",
	"Put on the weighted vest, snug and even, heels up on the sofa, bed or chair.",
	"Lie under the rings, grip them, and lift your hips so your body is flat.",
	"Pull your chest to your hands, breathing out, and hold it there for 2 s.",
	"Lower until your arms are straight, then pull again. Count each rep out loud.",
})

-- Core
move("incline_plank", "Incline plank", "Forearms on the desk edge; the desk must not move.",
	"Body straight from heels to head, belly tight. Count out loud.", "Stop when your hips sag or pike up.")
guide("incline_plank", {
	"Push on the desk first to check it stays put, then rest your forearms on its edge.",
	"Put your elbows right under your shoulders, forearms parallel and pointing forward.",
	"Walk your feet back onto your toes until your body is straight from heels to head.",
	"Pull your belly in, squeeze your bottom and breathe slowly and steadily.",
	"Count out loud while the clock times the hold, from the moment your body is straight.",
}, {
	"Press your forearms into the desk so your upper back stays wide, not sunk.",
	"Tilt your hips as if tucking your tail, which keeps your lower back from arching.",
	"Look down at the desk, not ahead, so your neck stays in line.",
})
move("knee_plank", "Knee plank", "Forearms and knees on the floor, a folded towel under your knees if it is hard.",
	"Straight line from knees to head, belly tight. Count out loud.", "Stop when your hips sag or pike up.")
guide("knee_plank", {
	"Kneel on the floor, forearms down, elbows right under your shoulders.",
	"Walk your knees back until your body is straight from knees to head.",
	"Lower your hips into that line, belly pulled in, bottom gently squeezed.",
	"Breathe slowly and steadily the whole time.",
	"Count out loud while the clock times the hold, from the moment the line is straight.",
}, {
	"Press the floor away with your forearms so your shoulders do not sink.",
	"Tuck your tail slightly to keep your lower back from arching.",
	"Look at the floor just ahead of your hands to keep your neck long.",
})
move("bird_dog", "Bird dog", "Hands and knees on the floor, a folded towel under your knees if it is hard.",
	"Reach one arm and the opposite leg, hips level, alternate sides. Breathe out as you reach.",
	"Stop when your hips rotate or your back sags. A reach on each side counts one.")
guide("bird_dog", {
	"Kneel on the floor, hands under your shoulders, knees under your hips, back flat.",
	"Reach one arm forward and the opposite leg back until both are level with your back.",
	"Hold the reach for 1 s with your hips level, then come back.",
	"Breathe out as you reach, breathe in as you come back.",
	"Alternate sides every rep; a reach with one side and then the other counts one.",
}, {
	"Push the heel back as if pressing a wall behind you, not up to the ceiling.",
	"Keep both hip bones pointing at the floor as the leg reaches back.",
	"Press the floor away with your standing hand so your upper back does not sag.",
})
-- the back holds (slice 54)
move("hinge_hold", "Hinge hold", "Feet hip-width apart, knees soft, nothing in your hands.",
	"Hips back, back flat at a slant, arms reaching forward. Count out loud.",
	"Stop when your back rounds or your lower back aches.")
guide("hinge_hold", {
	"Stand with feet hip-width apart, knees soft, arms by your sides.",
	"Push your hips back and tip your chest forward with your back flat.",
	"Stop when your chest is about halfway to level and reach your arms forward.",
	"Hold still with your back flat, breathing slowly and counting out loud.",
	"The clock times how long your back stays flat, then stand back up.",
}, {
	"Keep your neck long, your eyes on the floor a step in front of you.",
	"Squeeze your seat and feel the muscles beside your spine working.",
})
move("superman_hold", "Superman hold", "Lie face down on the floor, arms reaching past your head.",
	"Lift your arms, chest and legs a little off the floor. Count out loud.",
	"Stop when your lower back aches or your legs drop.")
guide("superman_hold", {
	"Lie face down on a mat, arms straight out past your head, legs straight.",
	"Squeeze your seat and lift your arms, chest and legs a hand's width off the floor.",
	"Keep your neck long, your eyes on the floor just ahead of your hands.",
	"Hold still and breathe slowly, counting out loud.",
	"The clock times how long you hold the lift, then lower everything slowly.",
}, {
	"Lift with your seat and back, never by throwing your head back.",
	"Reach long through your fingers and toes rather than high.",
})
move("bird_dog_hold", "Bird dog hold", "Hands and knees on the floor, a folded towel under your knees if it is hard.",
	"Reach one arm and the other leg level with your back and hold. Count out loud.",
	"Stop when your hips rotate or your back sags. Count the weaker side.")
guide("bird_dog_hold", {
	"Kneel on the floor, hands under your shoulders, knees under your hips, back flat.",
	"Reach one arm forward and the other leg back until both are level with your back.",
	"Hold still with your hips level, breathing slowly and counting out loud.",
	"When this side ends, press Switch sides and reach the other arm and leg.",
}, {
	"Push the heel back as if pressing a wall behind you, not up to the ceiling.",
	"Keep both hip bones pointing at the floor the whole hold.",
})
move("dead_bug", "Dead bug", "Lie on your back on the floor, knees and arms up.",
	"Lower one arm and the opposite leg, lower back pressed down. Breathe out as you lower.",
	"Stop when your lower back lifts off the floor. Alternate sides; each lower counts one.")
guide("dead_bug", {
	"Lie on your back on the floor, arms straight up above your shoulders.",
	"Lift your knees over your hips, bent to a right angle, lower back pressed down.",
	"Lower one arm overhead and the opposite leg out straight, both just above the floor.",
	"Bring them back to the start, breathing out as you lower and in as you return.",
	"Switch sides every time and count each lower as one rep.",
}, {
	"Breathe out fully before you lower so your ribs sink and your back stays down.",
	"Keep the other arm and leg still, only the working pair moves.",
	"Reach long through the heel of the moving leg, toes pulled towards you.",
})
move("wood_chopper", "Wood-chopper", "Clear arm's length around you.",
	"Hands together, chop from high on one side to low on the other. Breathe out on the chop.",
	"Stop when your hips swing with your arms. Count the weaker side.")
guide("wood_chopper", {
	"Clear an arm's length around you, stand with feet shoulder-width apart, hands clasped.",
	"Raise your hands high above one shoulder, arms long, chest turned that way.",
	"Chop down across your body to outside the opposite knee, bending your knees as you go.",
	"Rise back up slowly after each chop, breathing out on the chop.",
	"Count each chop; do every rep to one side, then press Switch sides.",
}, {
	"Let your chest and arms turn, but keep your hips facing forward.",
	"Keep your arms long and let your eyes follow your hands.",
	"Brace your belly before each chop so the move stays smooth and controlled.",
})
move("overhead_chop", "Overhead chop", "Clear space above and in front of you.",
	"Hands together overhead, chop down to your knees, ribs down. Breathe out on the chop.",
	"Stop when your back arches at the top.")
guide("overhead_chop", {
	"Check the space above and in front of you is clear, feet shoulder-width apart.",
	"Clasp your hands, arms straight overhead, ribs pulled down so your lower back stays flat.",
	"Chop down to your knees, hinging at the hips and bending your knees.",
	"Breathe out on the chop, breathe in as you rise slowly back overhead.",
	"Count each chop from overhead to your knees and back as one rep.",
}, {
	"Push your hips back as you chop so your back stays long and flat.",
	"Reset your ribs down at the top before you start each chop.",
	"Keep your arms straight and let your hands travel in one smooth arc.",
})
move("plank", "Forearm plank", "Forearms and toes on the floor.",
	"Body straight from heels to head, belly tight. Count out loud.", "Stop when your hips sag or pike up.")
guide("plank", {
	"Lie face down on the floor, forearms down, elbows right under your shoulders.",
	"Tuck your toes, feet hip-width apart, and lift your body off the floor.",
	"Hold a straight line from heels to head, belly pulled in, bottom squeezed.",
	"Breathe slowly and steadily through the whole hold.",
	"Count out loud while the clock times the hold, from the moment you lift.",
}, {
	"Drag your elbows towards your toes without moving them to switch your belly on.",
	"Push your heels back as if pressing a wall behind you, legs tight.",
	"Keep your gaze on the floor between your hands so your neck stays long.",
})
move("curl_up", "McGill curl-up", "Lie on the floor, one knee bent, hands under your lower back.",
	"Lift head and shoulders a little, hold 5 seconds, lower. Breathe out as you lift.",
	"Stop when your neck strains or your lower back flattens.")
guide("curl_up", {
	"Lie on your back, one knee bent with the foot flat, the other leg long on the floor.",
	"Slide your hands palms down under the small of your back to keep its natural curve.",
	"Lift your head and shoulders a little off the floor as one piece, chin level, hold 5 s.",
	"Breathe out as you lift, keep breathing gently through the hold, then lower.",
	"Each lift and hold counts as one rep.",
}, {
	"Think of your head and chest as one stiff block, with no nodding at the neck.",
	"Lift only a few centimetres, since the work is in keeping your belly tight.",
	"Brace your belly as if someone were about to poke it before you lift.",
})
move("bear_plank", "Bear plank", "Hands and knees on the floor.",
	"Lift your knees a hand's width off the floor, back flat. Count out loud.",
	"Stop when your back rounds or your knees touch down.")
guide("bear_plank", {
	"Kneel on the floor, hands under your shoulders, knees under your hips, toes tucked.",
	"Flatten your back so it is level from hips to head, eyes on the floor.",
	"Lift your knees a hand's width off the floor and keep them there.",
	"Breathe slowly and steadily through the whole hold.",
	"Count out loud while the clock times the hold, from the moment your knees lift.",
}, {
	"Push the floor away with your hands so your upper back stays full, not sunk.",
	"Keep your hips at the same height as your shoulders, not higher.",
	"Spread your fingers wide and press your whole hand into the floor, not just the heel.",
})
move("inchworm", "Inchworm walkout", "A body length of clear floor in front of you.",
	"Hinge down, walk your hands out to a plank and back. Count out loud, breathe steadily.",
	"Stop when your hips sag.")
guide("inchworm", {
	"Stand tall with a body length of clear floor in front of you, feet hip-width apart.",
	"Hinge at the hips and put your hands flat on the floor, bending your knees as needed.",
	"Walk your hands out in small steps until your body is straight from heels to head.",
	"Walk your hands back to your feet, then roll up slowly to stand tall.",
	"Breathe steadily and count out loud, one rep for each walk out and back.",
}, {
	"At the far end, stop with your hands under your shoulders and hips in line.",
	"Take small hand steps so your belly stays tight the whole way out.",
	"Keep your chin tucked as you roll up so your head comes up last.",
})
move("crab_walk", "Crab walk", "A body length of floor, clear of furniture.",
	"Hands behind you, hips up, walk forward and back. Count out loud.",
	"Stop when your hips drop or your shoulders ache.")
guide("crab_walk", {
	"Clear a body length of floor, then sit with knees bent and feet flat, hip-width apart.",
	"Put your hands behind your hips, fingers pointing to your feet, and lift your hips.",
	"Step one hand and the opposite foot forward together, then the other pair.",
	"Walk feet first for a few steps, then walk back the same way, breathing steadily.",
	"Count out loud while the clock times the walk, from your first step.",
}, {
	"Squeeze your bottom to keep your hips lifted as you step.",
	"Keep your chest open and your shoulders back, away from your ears.",
	"Keep your hands close behind your hips rather than far back.",
})
move("side_plank_knees", "Side plank on the knees",
	"Forearm and knees on the floor, a folded towel under your knees if it is hard.",
	"Hips up in a straight line from knees to head. Count out loud.",
	"Stop when your hips drop. Count the weaker side.")
guide("side_plank_knees", {
	"Lie on your side on the floor, elbow right under your shoulder, forearm pointing forward.",
	"Bend your knees so your feet are behind you, hips and shoulders stacked.",
	"Lift your hips until your body is straight from knees to head, top hand on your hip.",
	"Breathe slowly and steadily through the whole hold.",
	"Count out loud; when this side ends, press Switch sides, then turn over.",
}, {
	"Push the floor away with your forearm so your shoulder does not sink.",
	"Press your hips forward a little so they stay in line and do not drift back.",
	"Keep your head in line with your spine, eyes looking straight ahead.",
})
move("bear_crawl", "Bear crawl", "A clear path of a few steps on a floor that does not slip.",
	"Knees a hand's width off the floor, crawl forward and back. Count out loud.",
	"Stop when your hips rise high or your knees touch down.")
guide("bear_crawl", {
	"Clear a path a few steps long, then kneel with hands under shoulders, toes tucked.",
	"Lift your knees a hand's width off the floor, back flat and level.",
	"Step one hand and the opposite foot forward together, then the other pair.",
	"Take a few small steps forward, then crawl backwards the same way, breathing steadily.",
	"Count out loud while the clock times the crawl, from your first step.",
}, {
	"Keep your steps short, a hand's length at most, so your hips stay still.",
	"Keep your knees low, hovering just above the floor on every step.",
	"Look at the floor a little ahead of your hands to keep your neck long.",
})
move("side_plank", "Side plank", "Forearm and feet on the floor.",
	"Hips up in a straight line from feet to head. Count out loud.",
	"Stop when your hips drop. Count the weaker side.")
guide("side_plank", {
	"Lie on your side on the floor, elbow right under your shoulder, forearm pointing forward.",
	"Stack your feet with your legs straight, hips and shoulders one above the other.",
	"Lift your hips until your body is straight from feet to head, top hand on your hip.",
	"Breathe slowly and steadily through the whole hold.",
	"Count out loud; when this side ends, press Switch sides, then turn over.",
}, {
	"Push the floor away with your forearm so your shoulder stays high.",
	"Lift your lower hip a touch higher than feels natural, since it tends to sit low.",
	"Squeeze your bottom and keep your top hip over the lower one, not rolled forward.",
})
move("long_lever_plank", "Long-lever plank", "Forearms and toes on the floor.",
	"Elbows ahead of your shoulders, tuck your tailbone, squeeze everything. Count out loud.",
	"Stop when your lower back sags.")
guide("long_lever_plank", {
	"Kneel and put your forearms down, elbows a hand's width ahead of your shoulders.",
	"Tuck your toes, feet close together, and lift into a straight line from heels to head.",
	"Tuck your tailbone under and squeeze your bottom, thighs and fists hard.",
	"Keep breathing in short, steady breaths while you squeeze.",
	"Count out loud while the clock times the hold, from the moment you lift.",
}, {
	"Make fists and press the outer edge of your forearms into the floor.",
	"Pull your ribs down towards your hips so your lower back stays flat.",
	"Look at the floor just ahead of your hands to keep your neck long.",
})
move("copenhagen", "Copenhagen side plank", "Top knee on the edge of a bed or sofa, forearm on the floor.",
	"Forearm down, lift your hips, bottom leg hanging free. Count out loud.",
	"Stop when your hips drop or your inner thigh hurts. Count the weaker side.")
guide("copenhagen", {
	"Lie on your side on the floor beside a bed or sofa that stays put when you push it.",
	"Put your top knee on its edge, elbow on the floor right under your shoulder.",
	"Lift your hips into a straight line from knee to head, bottom leg hanging free.",
	"Put your top hand on your hip and breathe slowly and steadily through the hold.",
	"Count out loud; when this side ends, press Switch sides, then turn round.",
}, {
	"Press your top knee down into the edge, since that is what lifts your hips.",
	"Push the floor away with your forearm so your shoulder stays strong.",
	"Keep your hips stacked and pressed forward, not tipping back.",
})
move("copenhagen_thigh", "Copenhagen, thigh on the bed",
	"Top thigh over the edge of a bed or sofa, forearm on the floor.",
	"Press the thigh down, lift your hips, bottom leg hanging free. Count out loud.",
	"Stop when your hips drop or your inner thigh hurts. Count the weaker side.")
guide("copenhagen_thigh", {
	"Lie on your side on the floor beside a bed or sofa that stays put when you push it.",
	"Lay your top leg across it so the edge sits under the middle of your thigh.",
	"Put your elbow on the floor right under your shoulder, bottom leg along the floor.",
	"Press the thigh down into the edge and lift your hips into a straight line.",
	"Count out loud; when this side ends, press Switch sides, then turn round.",
}, {
	"Press your top thigh down into the edge, since that is what lifts your hips.",
	"Let the bottom leg hang free; a foot on the floor takes the work away.",
	"Keep the edge at the middle of your thigh every time, so each hold is the same move.",
})
move("plank_taps", "Plank shoulder taps", "Hands and toes on the floor, feet together.",
	"Tap each shoulder in turn, hips still. Count out loud.", "Stop when your hips rock or sag.")
guide("plank_taps", {
	"Put your hands on the floor right under your shoulders, arms straight, fingers spread.",
	"Tuck your toes, feet together, body straight from heels to head.",
	"Lift one hand to tap the opposite shoulder, put it back, then tap with the other.",
	"Keep a steady rhythm of about one tap a second and breathe freely.",
	"Count out loud while the clock times the taps, from your first tap.",
}, {
	"Push the floor away hard with the standing hand as the other one lifts.",
	"Squeeze your bottom and belly before each tap so your hips stay level.",
	"Imagine a glass of water on your lower back and tap without spilling it.",
})
move("body_saw", "Body saw", "Forearms down, toes on a towel on a smooth floor.",
	"From a plank, slide forward and back from the shoulders. Count out loud.",
	"Stop when your lower back sags.")
guide("body_saw", {
	"Put a towel on a smooth floor, your forearms down ahead of it and your toes on the towel.",
	"Lift into a straight line from heels to head, elbows under your shoulders.",
	"Push from your shoulders to slide your body back a hand's length, then pull it forward.",
	"Slide at an even pace and breathe steadily as you go.",
	"Count out loud while the clock times the sawing, from your first slide.",
}, {
	"Move only at the shoulders, your hips and back stay in one straight line.",
	"Pull your belly in hardest when your elbows are furthest ahead of your head.",
	"Keep both feet together on the towel so it glides as one piece.",
})
move("tuck_plow_raise", "Tuck plow raise", "Lie on your back on a clear floor, arms by your sides, palms down.",
	"Knees tucked, press the floor, hips up, lower in 3 seconds. Count out loud, breathe.",
	"Stop when you swing up or your neck takes the weight.")
guide("tuck_plow_raise", {
	"Lie on your back with your arms by your sides, palms flat on the floor.",
	"Pull your knees up to your chest and keep them tucked tight the whole time.",
	"Press your palms down and lift your hips until only your upper back rests on the floor.",
	"Lower your hips for a slow count of three until they touch down, breathing steadily.",
	"Each lift and lowering counts as one, counted out loud.",
}, {
	"Lift with your belly and the press of your hands, never a swing of the legs.",
	"Rest your weight on your upper back and shoulders, never on your neck or head.",
	"Roll down slowly so the lowering takes the full three seconds.",
})
move("desk_plank_foot_up", "Desk plank, one foot up", "Forearms on a sturdy desk edge, feet back, body straight.",
	"Lift one foot and hold. Count out loud, keep breathing.",
	"Stop when your hips sag or twist. Count the weaker side.")
guide("desk_plank_foot_up", {
	"Rest your forearms on the desk edge, elbows under your shoulders.",
	"Walk your feet back until your body is one straight line from heels to head.",
	"Lift one foot a hand's width off the floor and hold, hips level.",
	"Count out loud; when this side ends, press Switch sides and lift the other foot.",
}, {
	"Squeeze your seat and belly as if bracing for a light poke.",
	"Keep your hips level when the foot lifts, with no twist to one side.",
	"Push the desk away with your forearms so your upper back stays firm.",
})
move("desk_tuck_plank", "Desk tuck plank", "Hands on a sturdy desk edge, feet back, body straight.",
	"Draw one knee to your chest and hold. Count out loud, keep breathing.",
	"Stop when your hips sag or your back rounds. Count the weaker side.")
guide("desk_tuck_plank", {
	"Place your hands on the desk edge under your shoulders, arms straight.",
	"Walk your feet back until your body is one straight line from heels to head.",
	"Draw one knee toward your chest and hold it there, the other leg straight.",
	"Count out loud; when this side ends, press Switch sides and draw in the other knee.",
}, {
	"Keep the straight leg's heel pressing back so the line stays long.",
	"Pull the knee in with your belly, not by rounding your back.",
	"Keep your shoulders over your hands the whole time.",
})
move("desk_side_plank", "Desk side plank", "One forearm on a sturdy desk edge, side-on, feet stacked behind.",
	"Hips up, body straight sideways, hold. Count out loud, keep breathing.",
	"Stop when your hips drop or your shoulder shrugs. Count the weaker side.")
guide("desk_side_plank", {
	"Stand side-on to the desk and rest one forearm on its edge, elbow under the shoulder.",
	"Step your feet out to the side until your body leans in one straight line, feet stacked.",
	"Lift your hips so heels, hips and head line up, and hold.",
	"Breathe slowly; hold as long as you can, press Switch sides, then hold the other side.",
}, {
	"Push the desk away with your forearm so the shoulder stays strong.",
	"Keep your top hip stacked over the bottom one, not rolling forward.",
	"Look straight ahead so your neck stays in line.",
})
move("desk_side_plank_leg", "Desk side plank, leg up",
	"One forearm on a sturdy desk edge, side-on, feet stacked behind.",
	"Hips up, lift the top leg and hold. Count out loud, keep breathing.",
	"Stop when your hips drop or twist. Count the weaker side.")
guide("desk_side_plank_leg", {
	"Rest one forearm on the desk edge, side-on, elbow under the shoulder.",
	"Step your feet out to the side until your body is one straight line, feet stacked.",
	"Lift your hips, then raise the top leg a hand's width and hold.",
	"Breathe slowly; hold as long as you can, press Switch sides, then hold the other side.",
}, {
	"Lift the top leg straight up, not forward, toes pointing ahead.",
	"Press the bottom foot's edge into the floor to stay steady.",
	"Keep both hips stacked as the leg rises.",
})

-- Sit-up (ladder 24, 2026-10-17; David: "yes add"): the trunk curled slowly and few times, the hands on the chest or
-- at the temples and never behind the head, the lower back kept flat wherever the legs lower. The crunch's setup names
-- the temples, where its figure rests the fingertips (Rapha asked the coach to decide, 2026-10-28)
move("crunch", "Crunch", "Lie on the floor, knees bent; fingertips lightly at your temples, never pulling your head.",
	"Curl your shoulders up, ribs towards hips, lower slowly. Breathe out as you curl.",
	"Stop when your neck strains or your lower back hurts.")
guide("crunch", {
	"Lie on your back, knees bent, feet flat and hip-width apart.",
	"Cross your arms on your chest, or rest your fingertips at your temples.",
	"Breathe out and curl your head and shoulders up until your shoulder blades lift.",
	"Lower slowly, then breathe in at the bottom.",
	"Each curl up and back down counts as one rep.",
}, {
	"Never pull on your head; keep a fist of space between your chin and chest.",
	"Think of your ribs sliding down towards your hips, not your chin to your knees.",
	"Not in your first hour out of bed; after a long sit, walk around a few minutes first.",
})
move("lying_knee_raise", "Lying knee raise", "Lie on your back on the floor, palms down by your hips.",
	"Feet off the floor, draw your knees to your chest, lower slowly. Breathe out as you lift.",
	"Stop when your lower back arches off the floor or your legs swing.")
guide("lying_knee_raise", {
	"Lie on your back, arms by your sides, palms flat on the floor.",
	"Bend your knees and lift your feet a little off the floor; keep your lower back down.",
	"Breathe out and draw your knees in until they are over your hips.",
	"Lower your feet slowly until they almost touch the floor.",
	"Each lift and lowering counts as one rep.",
}, {
	"Press your lower back into the floor the whole time; that is the work.",
	"Tap your feet down between reps if your back starts to lift.",
	"Move slowly both ways, never a swing.",
})
move("reverse_crunch", "Reverse crunch", "Lie on your back on the floor, arms by your sides, palms down.",
	"Knees bent, curl your hips up off the floor, lower slowly. Breathe out as you curl.",
	"Stop when your legs swing or your lower back arches as you lower.")
guide("reverse_crunch", {
	"Lie on your back, arms by your sides, palms flat on the floor.",
	"Lift your feet and bend your knees to a right angle, knees over your hips.",
	"Breathe out and curl your hips a little off the floor, knees towards your chest.",
	"Lower your hips slowly until your lower back rests flat again.",
	"Each curl up and back down counts as one rep.",
}, {
	"Lift with your belly, not a kick of the legs; the lift is only a few centimetres.",
	"Keep your knees bent the same amount the whole time.",
	"Press your lower back flat on the floor before each new curl.",
})
move("situp_lowering", "Sit-up, slow lowering", "Sit on a clear floor, knees bent, feet flat, arms across your chest.",
	"Roll down slowly, about 3 s, then sit back up any way. Count out loud.",
	"Stop when you drop the last part of the way or your lower back hurts.")
guide("situp_lowering", {
	"Sit on the floor, knees bent, feet flat, arms crossed on your chest.",
	"Tuck your chin a little and round your back as if curling over a ball.",
	"Roll down slowly for a count of three until your shoulders touch the floor.",
	"Sit back up any way: roll to one side and push up if you need to.",
	"Each slow roll down counts as one, counted out loud.",
}, {
	"Go as slowly as you can the whole way, most of all near the floor.",
	"Feet lifting? Reach your arms forward past your knees.",
	"Breathe out slowly all the way down.",
})
move("situp_anchored", "Sit-up, feet under the sofa",
	"Toes under a heavy sofa or bed; hands on your chest, never behind your head.",
	"Feet hooked, curl up, roll down slowly. Count out loud, breathe out as you rise.",
	"Stop when you yank up with your legs, or your lower back or neck hurts.")
guide("situp_anchored", {
	"Lie on your back, knees bent, toes hooked under the edge of a heavy sofa or bed.",
	"Cross your arms on your chest, or rest your fingertips at your temples.",
	"Breathe out and curl up one part at a time: head, shoulders, then your back.",
	"Sit up tall, then roll down slowly, lower back first.",
	"Each sit-up and roll back down counts as one, counted out loud.",
}, {
	"Curl up with your belly; never yank yourself up with your feet.",
	"Keep your knees bent, so your belly does the work, not your hips.",
	"Never pull on your head or neck; your hands only rest there.",
})
move("situp", "Sit-up", "A clear floor; feet flat and free, hands on your chest or at your temples.",
	"Curl up to sitting, then roll down slowly. Count out loud, breathe out as you rise.",
	"Stop when your feet lift, you jerk up, or your lower back or neck hurts.")
guide("situp", {
	"Lie on your back, knees bent, feet flat on the floor and hip-width apart.",
	"Cross your arms on your chest, or rest your fingertips at your temples.",
	"Breathe out and curl up one part at a time: head, shoulders, then your back.",
	"Sit up tall, then roll down slowly, lower back first.",
	"Each sit-up and roll back down counts as one, counted out loud.",
}, {
	"Never pull on your head or neck; your hands only rest there.",
	"Feet lifting? Reach your arms forward past your knees until you get stronger.",
	"After a long sit at the desk, walk around for a few minutes first.",
})
move("leg_raise_bent", "Leg raise, bent up", "Lie on your back on the floor, legs long, palms down by your hips.",
	"Knees in, legs up straight, lower them straight and slowly. Count out loud.",
	"Stop when your lower back arches off the floor or your legs drop.")
guide("leg_raise_bent", {
	"Lie on your back, legs long and together, palms flat on the floor by your hips.",
	"Breathe out and pull your knees in towards your chest.",
	"Straighten your legs up until they point at the ceiling.",
	"Lower your straight legs slowly, lower back pressed down.",
	"Each lift and lowering counts as one, counted out loud.",
}, {
	"Lower only as far as your lower back stays flat; stop higher if it lifts.",
	"The slow, straight lowering is the work; bending in is the easy part.",
})
move("tuck_up", "Tuck-up", "Lie on your back on a clear floor, legs long, arms past your head.",
	"Sit up as your knees pull in, then lower slowly. Count out loud, breathe out as you tuck.",
	"Stop when you jerk up, your legs drop hard, or your lower back hurts.")
guide("tuck_up", {
	"Lie on your back, legs long and together, arms reaching past your head.",
	"Breathe out, sit up and pull your knees to your chest at the same time.",
	"Bring your arms forward to hug your shins, balanced on your seat.",
	"Lower your back and legs together slowly until your heels touch down lightly.",
	"Each tuck in and back out counts as one, counted out loud.",
}, {
	"Press your lower back into the floor before each tuck.",
	"Lead with your chest, not your chin, so your neck stays long.",
	"Move at an even pace, never a swing or a bounce.",
})
move("leg_raise", "Leg raise", "Lie on your back on the floor, hands flat under your seat.",
	"Legs straight, lift them up, lower slowly. Count out loud, breathe out as you lift.",
	"Stop when your lower back arches off the floor or your legs drop.")
guide("leg_raise", {
	"Lie on your back, legs straight and together, hands flat under your seat.",
	"Press your lower back into the floor and lift your heels a little off it.",
	"Breathe out and raise both straight legs until they point at the ceiling.",
	"Lower them slowly, stopping just before your heels touch.",
	"Each lift and lowering counts as one, counted out loud.",
}, {
	"Lower only as far as your lower back stays flat; stop higher if it lifts.",
	"Squeeze your legs together and point your toes so your knees stay straight.",
})
move("vup_one_leg", "One-leg V-up", "Lie on your back on a clear floor, legs straight, arms past your head.",
	"Lift your chest and one straight leg, reach for that shin, lower. Count out loud, breathe.",
	"Stop when you jerk up or your lower back hurts. A V-up to each side counts one.")
guide("vup_one_leg", {
	"Lie on your back, legs straight, arms reaching past your head.",
	"Breathe out and lift your chest and one straight leg at the same time.",
	"Reach both hands towards the raised shin, the other leg long on the floor.",
	"Lower slowly, then do the same with the other leg.",
	"A V-up with each leg counts as one, counted out loud.",
}, {
	"Press the leg on the floor down to keep you steady.",
	"Lead with your chest, not your chin, so your neck stays long.",
	"Lift the leg only as high as it stays straight.",
})
move("vup", "V-up", "Lie on your back on a clear floor, legs straight, arms past your head.",
	"Lift legs and chest together, hands to your shins, lower slowly. Count out loud, breathe.",
	"Stop when your knees bend, you jerk up, or your lower back hurts.")
guide("vup", {
	"Lie on your back, legs straight and together, arms reaching past your head.",
	"Breathe out and lift your legs and chest at the same time, both straight.",
	"Reach your hands towards your shins or toes at the top, balanced on your seat.",
	"Lower your back and legs together slowly until they touch down lightly.",
	"Each fold up and back down counts as one, counted out loud.",
}, {
	"Press your lower back into the floor before each lift.",
	"Tight legs? Lift a little lower with your legs straight, or do the one-leg V-up.",
	"Fold up smoothly at an even pace, never a swing or a jerk.",
})
move("hollow_rock", "Hollow rock", "Lie on your back on a rug or folded blanket, never on a hard bare floor.",
	"Hold the hollow shape, rock from shoulders to seat and back. Count out loud, breathe.",
	"Stop when your lower back arches or your arms or legs touch down.")
guide("hollow_rock", {
	"Lie on your back on a rug, legs together, arms reaching past your head.",
	"Press your lower back down, then lift your arms, shoulders and legs off the floor.",
	"Hold that curved shape and rock up towards your seat, then back to your shoulders.",
	"Rock at an easy pace, breathing steadily.",
	"Each rock forward and back counts as one, counted out loud.",
}, {
	"Keep the shape stiff, so your body rocks like a chair and never folds.",
	"Bend your knees or bring your arms to your sides to make the shape easier.",
	"Keep your chin a little tucked so your neck stays long.",
})
move("situp_weighted", "Sit-up with a weight", "A weight of 5-10 kg hugged to your chest: a bag of books, a jug.",
	"Hug the weight, curl up, roll down slowly. Count out loud, breathe out as you rise.",
	"Stop when your feet lift, you jerk up, or your lower back or neck hurts.")
guide("situp_weighted", {
	"Lie on your back, knees bent, feet flat on the floor and free.",
	"Hug the weight to your chest with both arms crossed over it.",
	"Breathe out and curl up one part at a time: head, shoulders, then your back.",
	"Sit up as far as the weight lets you, then roll down slowly, back first.",
	"Each sit-up and roll back down counts as one, counted out loud.",
}, {
	"Pick a weight you can sit up with 6 to 10 times; the game says when to go heavier.",
	"Keep the weight on your chest, never behind your head.",
	"Feet lifting? Hold the weight lower, nearer your belly.",
})
implement("situp_weighted", "dumbbells", "A dumbbell of 5-10 kg held on end against your chest.", {
	"Lie on your back, knees bent, feet flat on the floor and free.",
	"Hold the dumbbell on end against your chest, both hands cupping its top.",
	"Breathe out and curl up one part at a time: head, shoulders, then your back.",
	"Sit up as far as the weight lets you, then roll down slowly, back first.",
	"Each sit-up and roll back down counts as one, counted out loud.",
})
implement("situp_weighted", "kettlebell", "A kettlebell of 5-10 kg held on your chest by its horns.", {
	"Lie on your back, knees bent, feet flat on the floor and free.",
	"Hold the kettlebell to your chest by its horns, the bell down, elbows in.",
	"Breathe out and curl up one part at a time: head, shoulders, then your back.",
	"Sit up as far as the weight lets you, then roll down slowly, back first.",
	"Each sit-up and roll back down counts as one, counted out loud.",
})

-- Calf and ankle
move("heel_raise", "Heel raise", "Fingertips on the desk for balance.",
	"Rise high onto your toes, lower slowly. Breathe out as you rise.",
	"Stop when you cannot rise all the way.")
guide("heel_raise", {
	"Stand facing the desk, feet hip-width apart, fingertips resting on its edge.",
	"Stand tall with your knees straight but not locked.",
	"Rise onto the balls of your feet as high as you can, and pause at the top.",
	"Lower your heels slowly to the floor.",
	"Breathe out as you rise and count each rise as one rep.",
}, {
	"Press through your big toes so your ankles stay straight and do not roll out.",
	"Use the desk for balance only, never to push yourself up.",
	"Squeeze at the top as if trying to get a little taller.",
})
move("tibialis_raise", "Tibialis raise", "Back against the wall, heels a step away.",
	"Lift your toes toward your shins, lower slowly. Breathe out as you lift.",
	"Stop when your shins burn too much to lift fully.")
guide("tibialis_raise", {
	"Lean your back and hips against the wall, heels a step away from it.",
	"Keep your legs straight and your feet hip-width apart.",
	"Lift your toes up towards your shins, heels staying on the floor.",
	"Lower your toes slowly back towards the floor.",
	"Breathe out as you lift and count each lift as one rep.",
}, {
	"Lift your toes as high as they go, since the front of your shins does the work.",
	"Keep your bottom on the wall so the rest of your body stays still.",
	"Set your toes down softly, never let them slap the floor.",
})
move("sl_heel_raise_slow", "Single-leg heel raise, slow", "Fingertips on the desk for balance.",
	"Rise on one foot, lower for 3 seconds. Count out loud, breathe out as you rise.",
	"Stop when you cannot rise all the way. Count the weaker side.")
guide("sl_heel_raise_slow", {
	"Stand facing the desk, fingertips on its edge, and lift one foot behind you.",
	"Stand tall on the other foot, knee straight but not locked.",
	"Rise as high as you can onto the ball of your foot in 1 s, and pause at the top.",
	"Lower your heel for a full 3 s, all the way to the floor.",
	"Breathe out as you rise and count each rep; one foot, press Switch sides, then the other.",
}, {
	"Press through your big toe so your ankle stays straight and does not roll out.",
	"Count the lowering in your head, one and two and three, to keep it slow.",
	"Keep your fingertips light on the desk, for balance only.",
})
move("ankle_rocks", "Ankle rocks", "Face the wall, one foot a hand's length from it.",
	"Rock your knee toward the wall, heel down, and back. Breathe slowly.",
	"Stop when your heel lifts.")
guide("ankle_rocks", {
	"Face the wall, hands flat on it below your shoulders, front foot a hand's length from it.",
	"Step the other foot back and keep your front heel flat on the floor.",
	"Rock your front knee forward over your toes towards the wall, then rock back.",
	"Pause at the wall each time and breathe slowly.",
	"The clock times each side, then counts you in to switch feet.",
}, {
	"Aim your knee over your middle toes so it tracks straight, not inwards.",
	"Keep the front heel pressed down so the stretch stays in the ankle.",
	"Keep your hips square to the wall as you rock.",
})

-- Cardio and ballistic
move("step_jacks", "Step jacks", "Clear arm's length around you.",
	"Step one foot out as your arms go up, alternate sides. Breathe in rhythm.")
guide("step_jacks", {
	"Stand with an arm's length clear around you, feet together, arms by your sides.",
	"Stand tall with soft knees and a firm belly.",
	"Step one foot out to the side as both arms sweep out and up over your head.",
	"Step back in as your arms come down, then step the other foot out.",
	"Breathe in rhythm, fast enough that you cannot say a full sentence.",
}, {
	"Tap the stepping foot down lightly so the move stays quiet.",
	"Reach your arms fully overhead each time, elbows straight and hands close together.",
	"Time your arms and foot so they arrive at the same moment.",
})
move("shadow_boxing", "Shadow boxing", "Clear arm's length in front of you.",
	"Straight punches, fists back to your chin, feet light. Breathe out on each punch.",
	"Stop if your elbows snap straight.")
guide("shadow_boxing", {
	"Stand with one foot a short step ahead of the other, feet hip-width apart, knees soft.",
	"Raise both fists to your chin, elbows in, and turn your body a little to the side.",
	"Punch straight out with the front hand, snap it back, then punch with the back hand.",
	"Turn your back hip and heel into the back-hand punch, breathing out on each punch.",
	"Punch fast enough that you cannot say a full sentence.",
}, {
	"Bring each fist back to your chin as fast as it went out.",
	"Keep your chin tucked and your eyes on a target at head height.",
	"Stay light on the balls of your feet, bouncing a little between punches.",
})
move("cross_knee_drives", "Cross-knee drives", "Clear a small space beside the desk.",
	"Each knee up to the other hand, fast. Breathe out on every drive.")
guide("cross_knee_drives", {
	"Stand tall in the cleared space, feet hip-width apart, hands up in front of your chest.",
	"Drive your right knee up and across while your left hand comes down to meet it.",
	"Set the foot down and drive the left knee up to the right hand at once.",
	"Keep switching as fast as you stay balanced, turning from your waist.",
	"Go fast enough that you cannot say a full sentence.",
}, {
	"Bring the knee up to the hand, never the hand down to the knee.",
	"Keep the standing knee soft and your chest up as you turn.",
})
move("quick_steps", "Quick steps", "Shoes on, a floor that does not slip.",
	"Tiny, fast steps, one foot always down, arms pumping. Breathe in rhythm.")
guide("quick_steps", {
	"Stand with your feet hip-width apart, knees soft, weight on the balls of your feet.",
	"Take tiny, fast steps on the spot, lifting each heel while the toes stay down.",
	"Keep one foot on the floor at all times, so you never hop or land.",
	"Pump your bent arms fast in time with your feet.",
	"Go as fast as you can keep it quiet, too fast to talk.",
}, {
	"Faster, not higher: the quicker your feet, the harder it gets.",
	"Stay light and quiet, with no stamping.",
})
move("jog", "Jog on the spot", "Shoes on, a floor that does not slip.",
	"Light, quick steps on the balls of your feet. Breathe hard, too fast to talk.")
guide("jog", {
	"Push your chair back and stand with room to swing your arms, feet hip-width apart.",
	"Stand tall with a slight forward lean, elbows bent at a right angle, hands relaxed.",
	"Run on the spot with light, quick steps, lifting each foot only a little off the floor.",
	"Swing your arms forward and back in time with your feet and breathe in rhythm.",
	"Keep a hard pace, too fast to talk.",
}, {
	"Land softly on the balls of your feet so your steps stay quiet.",
	"Let your shoulders drop and stay loose, so only the arms swing.",
	"Count your steps in fours to settle into an even rhythm.",
})
move("butt_kicks", "Butt kicks", "Shoes on, a floor that does not slip.",
	"Jog on the spot, heels toward your seat. Breathe in rhythm.")
guide("butt_kicks", {
	"Stand with a little room around you, feet hip-width apart, the chair out of the way.",
	"Stand tall with your elbows bent at a right angle and your knees pointing at the floor.",
	"Run on the spot and flick each heel up toward your seat, knees staying under your hips.",
	"Swing your arms in time with your feet and breathe in rhythm with your steps.",
	"Keep a hard pace, too fast to talk.",
}, {
	"Keep your thighs pointing down so only the lower leg swings back.",
	"Stay on the balls of your feet and keep each touch on the floor short and light.",
	"Pull each heel up quickly and let it drop fast, like a quick flick.",
})
move("high_knees", "High knees", "Shoes on, a floor that does not slip.",
	"Run on the spot, knees to hip height, arms driving. Breathe in rhythm.")
guide("high_knees", {
	"Push your chair back so your knees have room in front, feet hip-width apart.",
	"Stand tall with your chest up and your elbows bent at a right angle.",
	"Run on the spot and lift each knee to hip height, pumping the opposite arm forward.",
	"Breathe in rhythm with your steps, never holding your breath.",
	"Keep a fast, light pace, too fast to talk.",
}, {
	"Stay tall and do not lean back to get the knees higher.",
	"Land on the balls of your feet right under your hips, quick and quiet.",
	"Lift each knee straight up in front of its hip, not out to the side.",
})
move("fast_feet", "Fast feet", "Shoes on, a floor that does not slip.",
	"Tiny, fast steps on the balls of your feet, knees soft. Breathe in rhythm.")
guide("fast_feet", {
	"Stand with your feet a little wider than your hips, the chair out of the way.",
	"Bend your knees a little and lean slightly forward, weight on the balls of your feet.",
	"Take tiny, fast steps on the spot, each foot lifting only a few centimetres.",
	"Pump your bent arms in small, quick swings and breathe in rhythm.",
	"Go as fast as you can keep it smooth, too fast to talk.",
}, {
	"Stay low with soft knees so you never drop onto your heels.",
	"Keep your feet on the same spot and the steps small, never big lifts.",
	"Think quick and quiet, as if the floor were too hot to stand on.",
})
move("jumping_jacks", "Jumping jacks", "Shoes on, a floor that does not slip.",
	"Jump feet wide as your arms go up, land softly. Breathe in rhythm.")
guide("jumping_jacks", {
	"Stand with an arm's length clear to each side and nothing low above your head.",
	"Start with feet together, arms by your sides and knees soft.",
	"Jump your feet out wider than your shoulders as your arms swing out and up overhead.",
	"Jump back to feet together as your arms come down, breathing in rhythm with the jumps.",
	"Keep a quick rhythm, too fast to talk.",
}, {
	"Land on the balls of your feet with soft knees so each landing is quiet.",
	"Keep your arms straight and bring them all the way up until your hands nearly meet.",
	"Keep your belly firm so your back does not arch as your arms go up.",
})
move("seal_jacks", "Seal jacks", "Shoes on, a floor that does not slip.",
	"Jump feet wide as you clap your arms open, then close. Breathe in rhythm.")
guide("seal_jacks", {
	"Stand with an arm's length clear to each side and in front of you.",
	"Start with feet together and arms straight out in front at chest height, palms together.",
	"Jump your feet wide as you open your arms out to the sides at shoulder height.",
	"Jump your feet back together as your arms swing in and your hands clap in front.",
	"Breathe in rhythm at a hard pace, too fast to talk.",
}, {
	"Keep your arms at shoulder height and open them wide enough to feel your chest stretch.",
	"Squeeze your shoulder blades together each time the arms open.",
	"Land softly on the balls of your feet so the clap is the loudest sound.",
})
move("skater_steps", "Skater steps", "A clear step to each side, a floor that does not slip.",
	"Step wide, land on one foot and hold. Each landing counts one. Breathe in rhythm.",
	"Stop if a knee caves in or your landings wobble.")
guide("skater_steps", {
	"Clear about a metre to each side and stand on one foot, knee bent, chest forward.",
	"Step wide to the side and land on the other foot, sinking into that knee.",
	"Sweep the free foot behind the landing ankle and touch its toes down lightly.",
	"Hold the landing for a moment, then step straight back the other way.",
	"Each landing counts one: go as fast as steady landings allow, too fast to talk.",
}, {
	"Keep the landing knee over your toes, never caving in.",
	"Never hop: one foot stays on the floor until the other lands.",
	"Keep your chest over the landing knee rather than letting it tip to the outside.",
})
move("skater_hops", "Skater hops", "A clear step to each side, a floor that does not slip.",
	"Hop sideways from foot to foot, land softly; each landing counts one. Breathe in rhythm.",
	"Stop if your landings get loud.")
guide("skater_hops", {
	"Clear about a metre to each side and stand on one foot, knee bent, chest leaning forward.",
	"Let the other foot hang behind and across your standing ankle, off the floor.",
	"Push off sideways and land softly on the other foot, swinging your arms across your body.",
	"Let the free foot sweep behind the landing leg, then push straight back the other way.",
	"Breathe in rhythm with the hops at a hard pace, too fast to talk.",
}, {
	"Land on the ball of the foot and sink into the knee to make each landing quiet.",
	"Land balanced over the foot, so each push starts from a steady leg.",
	"Keep your chest over the landing knee rather than letting it tip to the outside.",
})
move("pogo_hops", "Pogo hops", "Shoes on, a floor that does not slip.",
	"Small bounces on the balls of your feet, knees almost straight. Breathe in rhythm.",
	"Stop if your calves or heels hurt.")
guide("pogo_hops", {
	"Stand with feet hip-width apart, arms loose at your sides, nothing low above your head.",
	"Rise onto the balls of your feet and keep your knees almost straight, body tall.",
	"Bounce a few centimetres off the floor from your ankles, touching down only briefly.",
	"Breathe in rhythm with the bounces, never holding your breath.",
	"Keep a quick rhythm, too fast to talk.",
}, {
	"Think of the floor as hot, so each touch is short and springy.",
	"Keep your heels just off the floor on every landing.",
	"Stay tall with your eyes ahead so your whole body moves as one stiff spring.",
})
move("mountain_climbers", "Mountain climbers", "Hands on the floor under your shoulders.",
	"Each knee drive in counts one, so a left and a right make two. Breathe in rhythm.",
	"Stop when your hips pike up.")
guide("mountain_climbers", {
	"Place your hands flat on the floor, shoulder-width apart, fingers spread wide.",
	"Walk your feet back into a straight line from head to heels, arms straight, hips level.",
	"Drive one knee toward your chest, put that foot back, then drive the other knee.",
	"Each knee drive counts one, so a left and a right make two.",
	"Breathe in rhythm at a hard pace, too fast to talk, never holding your breath.",
}, {
	"Keep your shoulders over your wrists, not drifting back toward your feet.",
	"Keep your hips in line with your shoulders as the knees drive in.",
	"Tap the toe down lightly at the back each time so the switch stays quick and quiet.",
})
move("squat_thrusts", "Squat thrusts", "A body length of clear floor behind you, shoes on.",
	"Squat, hands down, jump your feet back and in, stand. Breathe in rhythm.",
	"Stop when your back sags.")
guide("squat_thrusts", {
	"Stand with a body length of clear floor behind you, feet hip-width apart.",
	"Squat and put your hands flat on the floor, shoulder-width apart, in front of your feet.",
	"Jump both feet back to a straight line from head to heels, then jump them back in.",
	"Stand all the way up, then go straight into the next one, breathing in rhythm.",
	"Keep a strong, hard pace, too fast to talk.",
}, {
	"Brace your belly before the feet jump back so you land in a firm straight line.",
	"Keep your shoulders right over your hands as the feet jump out and in.",
	"Jump the feet in close to your hands so you can stand straight up.",
})
move("sprawl", "Sprawl", "A body length of clear floor behind you, shoes on.",
	"Drop your hands, shoot your legs back, hips down, then stand. Breathe in rhythm.",
	"Stop when your back sags.")
guide("sprawl", {
	"Stand with a body length of clear floor behind you, feet hip-width apart.",
	"Drop your hands flat to the floor, shoulder-width apart, just in front of your feet.",
	"Shoot your legs back and a little wide and let your hips sink toward the floor, chest up.",
	"Lift your hips, jump your feet back in to your hands and stand tall, breathing in rhythm.",
	"Keep a strong, hard pace, too fast to talk.",
}, {
	"Keep your arms straight and press the floor away as the hips sink.",
	"Throw the legs back fast, then snap them back in just as fast.",
	"Look ahead, not down, so your chest stays up in the low position.",
})
move("burpee", "Burpees", "A body length of clear floor behind you, shoes on.",
	"Squat, hands down, jump back, jump in, stand and jump. Breathe in rhythm.",
	"Stop when your back sags.")
guide("burpee", {
	"Stand with a body length of clear floor behind you, feet hip-width apart.",
	"Squat and put your hands flat on the floor, shoulder-width apart, just ahead of your feet.",
	"Jump your feet back to a straight line, then jump them back in to your hands.",
	"Stand up and jump with your arms reaching overhead, then land softly and go again.",
	"Count only reps with all three jumps, as fast as clean reps allow, breathing hard.",
}, {
	"Land every jump on soft knees and the balls of your feet to keep it quiet.",
	"Brace your belly as the feet go back so your body lands in a firm straight line.",
	"Keep your hands flat and right under your shoulders while your legs are back.",
})
move("burpee_chest", "Chest-to-floor burpees", "A body length of clear floor behind you, shoes on.",
	"Jump back, chest to the floor, push up, jump in, jump. Breathe in rhythm.",
	"Stop when your back sags.")
guide("burpee_chest", {
	"Stand with a body length of clear floor behind you, feet hip-width apart.",
	"Squat, put your hands flat on the floor and jump your feet back to a straight line.",
	"Lower your chest to the floor, then push up to the straight line again.",
	"Jump your feet in to your hands, stand and jump with your arms reaching overhead.",
	"Go as fast as clean reps allow, breathing hard and freely all the way through.",
}, {
	"Count a rep only when the chest touched the floor and the jump left it.",
	"Land every jump on soft knees and the balls of your feet to keep it quiet.",
})
move("sprint", "Sprint on the spot", "Shoes on, a floor that does not slip.",
	"Run on the spot as fast as you can, arms driving. Breathe hard and freely.")
guide("sprint", {
	"Push your chair back so your knees and arms have room, feet hip-width apart.",
	"Lean forward slightly from the ankles, elbows bent at a right angle.",
	"Run on the spot as fast as you can, driving your knees up high in front of you.",
	"Pump your arms hard from the shoulders, hands swinging from hip to chin height.",
	"Breathe hard and freely at a pace so fast that you cannot talk.",
}, {
	"Drive your arms faster and your legs will match their speed.",
	"Stay on the balls of your feet and keep each step short and snappy.",
	"Keep your face, jaw and hands relaxed, since tension slows your legs.",
})
move("squat_reach", "Squat to reach", "Clear a small space with room overhead.",
	"Squat, stand and reach both arms up, keep the pace. Breathe out as you reach.")
guide("squat_reach", {
	"Stand with feet shoulder-width apart in the cleared space.",
	"Sit your hips back into a squat, hands reaching forward.",
	"Stand up quickly and reach both arms overhead, rising onto your toes.",
	"Go straight back into the next squat, breathing out as you reach.",
	"Keep a hard pace, too fast to talk.",
}, {
	"Keep your heels down in the squat and rise onto your toes only at the top.",
	"Reach long through your fingertips to open your chest each time.",
	"Keep your knees in line with your toes as you go down.",
})
move("punch_duck", "Punch and duck", "Clear a small space, fists up by your chin.",
	"Two quick punches, then duck into a half squat. Breathe out on every punch.")
guide("punch_duck", {
	"Stand in a boxing stance, one foot ahead, fists by your chin.",
	"Throw two quick punches straight ahead, pulling each fist back to the chin.",
	"Duck down into a half squat, chest up, then rise straight back up.",
	"Breathe out sharply on every punch and keep the rhythm going.",
	"Keep a hard pace where you can say only a word or two.",
}, {
	"Turn your hips and back foot into each punch for speed.",
	"Never lock the elbow: stop each punch just short of straight.",
	"Keep your eyes forward as you duck, back straight.",
})
move("wall_climbers", "Wall climbers", "Hands on a wall at shoulder height, feet back, body straight.",
	"Each knee drive to the wall counts one, so a left and a right make two. Breathe in rhythm.")
guide("wall_climbers", {
	"Put your hands on the wall at shoulder height, arms straight.",
	"Walk your feet back until your body leans in one straight line.",
	"Drive one knee up toward the wall, then switch legs quickly.",
	"Each knee drive counts one, so a left and a right make two.",
	"Keep switching at a fast pace, breathing in rhythm with your knees.",
}, {
	"Keep your hips low and level as the knees drive.",
	"Push the wall away so your shoulders stay strong.",
	"Land each foot softly on the ball to keep it quiet.",
})
move("desk_burpee", "Desk step-back burpee", "Hands on a sturdy desk edge, room behind you to step back.",
	"Step back, step in, stand and reach, no jumping. Breathe out as you stand.")
guide("desk_burpee", {
	"Stand facing the desk, a step back from its edge.",
	"Put your hands on the desk edge and step one foot back, then the other, into a plank.",
	"Step the feet back in one at a time, then stand tall and reach both arms overhead.",
	"Breathe out as you stand and reach, and keep moving without a break.",
	"Keep a hard pace where you can say only a word or two.",
}, {
	"Keep your body straight in the plank at the desk, hips in line.",
	"Step, never jump, so it stays quiet.",
	"Lock your hands on the edge before each step back.",
})
-- the five roads' forms (slice 54, 2026-10-23)
move("side_steps", "Side steps", "Clear a step's width to each side.",
	"Step to the side and bring the other foot in, then back the other way. Breathe in rhythm.")
guide("side_steps", {
	"Stand tall in the cleared space, feet together, knees soft, hands up in front of you.",
	"Step one foot out to the side and bring the other foot in to meet it.",
	"Step back the other way the same, to and fro across the space.",
	"Swing your arms with the steps and keep the pace brisk.",
	"Breathe in rhythm, fast enough that you cannot say a full sentence.",
}, {
	"Stay on the balls of your feet so the steps stay light and quiet.",
	"Keep your knees soft and your chest up, and never cross your feet.",
})
move("punch_march", "Punch march", "Clear arm's length in front of you.",
	"March on the spot and punch straight out with each step. Breathe out on each punch.",
	"Stop if your elbows snap straight.")
guide("punch_march", {
	"Stand tall, feet hip-width apart, fists by your chin, elbows in.",
	"March on the spot, lifting each knee to hip height.",
	"Punch straight out with the hand opposite the lifted knee, then bring it back.",
	"Keep marching and punching in one steady rhythm, breathing out on each punch.",
	"March fast enough that you cannot say a full sentence.",
}, {
	"Bring each fist back to your chin as fast as it went out.",
	"Stop each punch just short of a straight elbow.",
})
move("knee_strikes", "Knee strikes", "Clear a small space, fists up by your chin.",
	"Drive a knee up as your hands pull down; each strike counts one. Breathe out on each.",
	"Slow down if you lose your balance.")
guide("knee_strikes", {
	"Stand in a boxing stance, one foot a short step ahead, fists up by your chin.",
	"Reach both hands forward as if gripping a shoulder at chest height.",
	"Drive one knee up hard as both hands pull down to meet it.",
	"Put the foot back down and drive the other knee; each strike counts one.",
	"Go as fast as clean strikes allow, breathing out on every strike.",
}, {
	"Rise onto the ball of the standing foot as the knee drives up.",
	"Keep your chest up; bring the knee up rather than your chest down.",
})
move("speed_squats", "Speed squats", "Clear a small space, a floor that does not slip.",
	"Squat until your thighs are nearly level, stand fast; each counts one. Breathe in rhythm.",
	"Slow down when your knees cave in or your heels lift.")
guide("speed_squats", {
	"Stand with feet shoulder-width apart, toes turned out a little, hands in front.",
	"Sit your hips back and down until your thighs are nearly level.",
	"Drive up fast through your whole foot and stand tall; each stand counts one.",
	"Go straight into the next squat without a pause at the top.",
	"Go as fast as clean reps allow, breathing hard and freely.",
}, {
	"Keep your knees in line with your toes all the way down and up.",
	"Keep your heels on the floor and never bounce at the bottom.",
})
move("plank_jacks", "Plank jacks", "Hands on the floor under your shoulders, a floor that does not slip.",
	"Jump the feet wide and back in, hips level; each jump in counts one. Breathe in rhythm.",
	"Stop when your hips sag or pike up.")
guide("plank_jacks", {
	"Place your hands flat on the floor under your shoulders, fingers spread.",
	"Step back into a high plank, body straight from heels to head, feet together.",
	"Jump both feet out wide, then back together, like a jumping jack lying down.",
	"Keep your hips level and your shoulders over your hands the whole time.",
	"Go as fast as clean reps allow, breathing hard and freely.",
}, {
	"Land softly on the balls of your feet so the jumps stay light.",
	"Brace your belly so your hips do not bounce with your feet.",
})
move("cross_climbers", "Cross climbers", "Hands on the floor under your shoulders.",
	"Each knee driven across to the other elbow counts one. Breathe in rhythm.",
	"Stop when your hips pike up.")
guide("cross_climbers", {
	"Place your hands flat on the floor, shoulder-width apart, fingers spread wide.",
	"Step back into a high plank, body straight from heels to head.",
	"Drive one knee across your body toward the other elbow, then step it back.",
	"Switch legs fast; each knee across counts one, so a left and a right make two.",
	"Go as fast as clean reps allow, breathing hard and freely.",
}, {
	"Keep your shoulders over your hands as the knees cross.",
	"Let your hips turn a little with each knee, then square them again.",
})
move("lunge_knee_drives", "Lunge knee drives", "Clear a step behind you, a floor that does not slip.",
	"Lunge back, then drive that knee up as you stand; each counts one. Breathe in rhythm.",
	"Slow down when your front knee caves in.")
guide("lunge_knee_drives", {
	"Stand tall with feet hip-width apart, hands up in front of your chest.",
	"Step one foot back into a lunge until the back knee is just above the floor.",
	"Drive up off the front foot and swing the back knee up to hip height.",
	"Put it down beside the other foot, then lunge back with the other leg.",
	"Go as fast as clean reps allow; each knee drive counts one.",
}, {
	"Keep your front knee over your foot as you go down and up.",
	"Stand tall on the front leg at the top before the next lunge.",
})
move("step_back_burpee", "Step-back burpee", "A body length of clear floor behind you.",
	"Step back, chest to the floor, push up, step in, stand and reach. Breathe in rhythm.",
	"Stop when your back sags.")
guide("step_back_burpee", {
	"Stand with a body length of clear floor behind you, feet hip-width apart.",
	"Squat, put your hands flat on the floor and step your feet back one at a time.",
	"Lower your chest to the floor, then push up to the straight line again.",
	"Step your feet back in one at a time, stand tall and reach both arms overhead.",
	"Go as fast as clean reps allow, breathing hard and freely all the way through.",
}, {
	"Count a rep only when the chest touched the floor and the arms reached up.",
	"Step, never jump, so it stays quiet.",
})
move("jump_lunges", "Jump lunges", "Clear space, shoes on, a floor that does not slip.",
	"Jump from a lunge and switch legs in the air; each landing counts one. Breathe in rhythm.",
	"Stop when your landings get loud or your front knee caves in.")
guide("jump_lunges", {
	"Step into a lunge, front knee over your foot, back knee just above the floor.",
	"Swing your arms and jump straight up out of the lunge.",
	"Switch your legs in the air and land softly in a lunge on the other side.",
	"Sink straight into the next jump; each landing counts one.",
	"Go as fast as clean reps allow, breathing hard and freely.",
}, {
	"Land on the whole front foot with a soft knee, never with a straight leg.",
	"Keep your chest up and jump straight up, not forward.",
})
move("sprawl_strike", "Sprawl and strike", "A body length of clear floor behind you, shoes on.",
	"Sprawl, jump in, stand, then a jab and a cross; each round counts one. Breathe in rhythm.",
	"Stop when your back sags.")
guide("sprawl_strike", {
	"Stand with a body length of clear floor behind you, fists up by your chin.",
	"Drop your hands to the floor and shoot your legs back, hips down to the floor.",
	"Jump your feet back in to your hands and stand up, fists back to your chin.",
	"Punch a straight jab, then a cross, fists back to your chin; the round counts one.",
	"Go as fast as clean rounds allow, breathing hard and freely.",
}, {
	"Drop your hips low on the sprawl so your back stays flat.",
	"Turn your back hip into the cross, breathing out on each punch.",
})
move("lunge_strike", "Lunge and strike", "Clear a step behind you and arm's length in front.",
	"Lunge back each side, a jab and a cross at the bottom; both count one. Breathe out.",
	"Slow down when your front knee caves in.")
guide("lunge_strike", {
	"Stand tall, feet hip-width apart, fists up by your chin.",
	"Step one foot back into a lunge until the back knee is just above the floor.",
	"At the bottom, punch a straight jab, then a cross, fists back to your chin.",
	"Stand up, then lunge back with the other foot and punch again; both sides count one.",
	"Go as fast as clean reps allow, breathing out on each punch.",
}, {
	"Keep your front knee over your foot and your chest up as you punch.",
	"Stop each punch just short of a straight elbow.",
})
move("stepped_chest_burpee", "Stepped chest burpee", "A body length of clear floor behind you.",
	"Step back, chest down and push up twice, step in, stand and reach. Breathe in rhythm.",
	"Stop when your back sags.")
guide("stepped_chest_burpee", {
	"Stand with a body length of clear floor behind you, feet hip-width apart.",
	"Squat, put your hands flat on the floor and step your feet back one at a time.",
	"Lower your chest to the floor and push up, then do it once more.",
	"Step your feet back in one at a time, stand tall and reach both arms overhead.",
	"Go as fast as clean reps allow, breathing hard and freely all the way through.",
}, {
	"Count a rep only when the chest touched the floor twice and the arms reached up.",
	"Step, never jump, so it stays quiet.",
})
move("desk_pushup_burpee", "Desk push-up burpee", "Hands on a sturdy desk edge, room behind you to step back.",
	"Step back, chest to the edge, push up, step in, stand and reach. Breathe out as you stand.")
guide("desk_pushup_burpee", {
	"Stand facing the desk, a step back from its edge.",
	"Put your hands on the desk edge and step one foot back, then the other, into a plank.",
	"Lower your chest to the desk edge, then push back up to straight arms.",
	"Step the feet back in one at a time, then stand tall and reach both arms overhead.",
	"Keep a hard pace where you can say only a word or two.",
}, {
	"Keep your body straight from heels to head as your chest goes down.",
	"Lock your hands on the edge before each step back.",
	"Step, never jump, so it stays quiet.",
})
move("desk_burpee_knees", "Desk burpee with knees", "Hands on a sturdy desk edge, room behind you to step back.",
	"Step back, step in, stand, then drive a knee up each side. Breathe out on every knee.")
guide("desk_burpee_knees", {
	"Stand facing the desk, a step back from its edge.",
	"Put your hands on the desk edge and step one foot back, then the other, into a plank.",
	"Step the feet back in one at a time and stand tall.",
	"Drive one knee up to hip height, then the other, and reach both arms overhead.",
	"Keep a hard pace where you can say only a word or two.",
}, {
	"Pull your hands down to meet each knee as if gripping a shoulder.",
	"Lock your hands on the edge before each step back.",
})
move("star_jumps", "Star jumps", "Clear space, shoes on, a floor that does not slip.",
	"From a half squat, jump up with arms and legs wide, land softly. Breathe in rhythm.",
	"Stop when your landings get loud or your knees cave in.")
guide("star_jumps", {
	"Stand with feet hip-width apart in the cleared space, arms by your sides.",
	"Sink into a half squat with your hands low in front of your knees.",
	"Jump straight up and spread your arms and legs wide like a star.",
	"Bring them back in and land softly in the half squat; each jump counts one.",
	"Go as fast as clean reps allow, breathing hard and freely.",
}, {
	"Land on the balls of your feet with soft knees, then let your heels come down.",
	"Keep your knees in line with your toes as you land.",
})
move("stepped_spider_burpee", "Stepped spider burpee", "A body length of clear floor behind you.",
	"Step back, push up twice, a spider step each side, step in, stand. Breathe in rhythm.",
	"Stop when your back sags.")
guide("stepped_spider_burpee", {
	"Stand with a body length of clear floor behind you, feet hip-width apart.",
	"Squat, put your hands flat on the floor and step your feet back one at a time.",
	"Lower your chest to the floor and push up, then do it once more.",
	"Step one foot up outside its hand and back, then the other foot.",
	"Step your feet back in, stand tall and reach both arms overhead.",
}, {
	"Count a rep only when both push-ups and both spider steps were done.",
	"Keep your hips low as each foot steps up beside its hand.",
	"Step, never jump, so it stays quiet.",
})
move("desk_spider_burpee", "Desk spider burpee", "Hands on a sturdy desk edge, room behind you to step back.",
	"Step back, push up, each knee out to its elbow, step in, stand. Breathe out as you stand.")
guide("desk_spider_burpee", {
	"Stand facing the desk, a step back from its edge.",
	"Put your hands on the desk edge and step one foot back, then the other, into a plank.",
	"Lower your chest to the desk edge, then push back up to straight arms.",
	"Bring one knee out to its elbow and back, then the other knee.",
	"Step the feet back in, stand tall and reach both arms overhead.",
}, {
	"Keep your hips level as each knee comes out to the side.",
	"Lock your hands on the edge before each step back.",
})
move("desk_burpee_combo", "Desk burpee combo", "Hands on a sturdy desk edge, room behind you to step back.",
	"Step back, step in, stand, jab, cross, a knee up each side. Breathe out on each strike.")
guide("desk_burpee_combo", {
	"Stand facing the desk, a step back from its edge.",
	"Put your hands on the desk edge and step one foot back, then the other, into a plank.",
	"Step the feet back in one at a time and stand up, fists by your chin.",
	"Punch a jab and a cross, then drive one knee up and then the other.",
	"Keep a hard pace where you can say only a word or two.",
}, {
	"Bring each fist back to your chin before the next strike.",
	"Lock your hands on the edge before each step back.",
})
move("tuck_jump_rally", "Tuck jump rally", "Clear space, shoes on, a floor that does not slip.",
	"Jump, pull your knees up to your chest, land softly and go again. Breathe in rhythm.",
	"Stop when your landings get loud or your knees cave in.")
guide("tuck_jump_rally", {
	"Stand with feet hip-width apart in the cleared space, knees soft.",
	"Dip and swing your arms, then jump straight up.",
	"Pull both knees up toward your chest at the top of the jump.",
	"Land softly on the balls of your feet and go straight into the next jump.",
	"Go as fast as clean reps allow, breathing hard and freely; each jump counts one.",
}, {
	"Bring your knees up to your chest; do not fold your chest down to them.",
	"Land with soft knees in line with your toes, then let your heels come down.",
})

-- Carry
move("suitcase_carry", "Suitcase carry", "A backpack of 8-15 kg or a full water jug, a clear hallway.",
	"Carry it in one hand, stand tall, walk steadily. Breathe slowly.",
	"Stop when you lean to one side.")
guide("suitcase_carry", {
	"Clear the hallway and hold a backpack or a full water jug in one hand at your side.",
	"Stand tall, shoulders level, arm long, the load clear of your leg.",
	"Walk steadily along the hallway, turning round at the end.",
	"Breathe steadily as you walk.",
	"The clock times each hand; load it so the last 10 s of each are hard.",
}, {
	"Keep both shoulders level, the loaded one neither dropping nor hunching up.",
	"Squeeze the handle hard and keep your wrist straight.",
	"Load the pack to 8-15 kg: 1-L water bottles weigh 1 kg each, a hardback about 1 kg.",
})

-- Mobility
move("pec_stretch", "Doorframe chest stretch", "Forearm on a doorframe, elbow at shoulder height.",
	"Step through gently until your chest opens, then rotate open. Breathe slowly.",
	"Ease off if your shoulder pinches.")
guide("pec_stretch", {
	"Stand in an open doorway by a solid frame, with the door fully open and out of the way.",
	"Put one forearm flat on the frame, elbow at shoulder height, hand pointing up.",
	"Stagger your feet, same-side foot ahead, and lean forward until your chest opens.",
	"Turn your chest gently away from the frame, then turn back and step out slowly.",
	"Ease in on each breath out; the clock times each arm, then counts you in to swap.",
}, {
	"Keep your elbow level with your shoulder so the stretch stays across the chest.",
	"Draw your shoulder blade back and down before you lean forward.",
	"Keep your ribs down and do not arch your lower back to reach further.",
})
L["move.pec_stretch.mark"] = "Elbow at shoulder height on the frame, chest stepped through level with the frame."
move("open_book", "Open book", "Lie on your side on the floor, knees bent.",
	"Open your top arm across to the other side, eyes follow. Breathe out as you open.",
	"Ease off if your back pinches.")
guide("open_book", {
	"Bend your hips and knees to a right angle and stack your knees on top of each other.",
	"Reach both arms straight out in front of you at shoulder height, palms together.",
	"Sweep your top arm up and over to the other side, turning your chest to the ceiling.",
	"Follow your hand with your eyes, breathe out as you open, and return slowly.",
	"The clock times each side, then counts you in to roll over.",
}, {
	"Keep your top knee resting on the bottom knee so the turn comes from your upper back.",
	"Let the arm sink towards the floor on each breath out rather than forcing it down.",
	"Keep the top arm long as it sweeps, like the page of a book turning over.",
})
move("hip_flexor_stretch", "Half-kneeling hip stretch", "One knee on a folded towel.",
	"Tuck your tailbone, shift forward gently, tall chest. Breathe slowly.",
	"Ease off if your knee hurts.")
guide("hip_flexor_stretch", {
	"Kneel on one knee with the other foot flat in front, both knees bent at right angles.",
	"Put your hands on your hips and stack your chest tall over your hips.",
	"Tuck your tailbone under, then shift your hips slowly forward.",
	"Shift only until you feel a stretch at the front of the back hip, then breathe slowly.",
	"The clock times each side, then counts you in to switch knees.",
}, {
	"Squeeze your seat on the back-leg side to feel more stretch without leaning further.",
	"Keep a straight line from your back knee through your hip to your head.",
	"Keep your front knee over your ankle as your hips move forward.",
})
L["move.hip_flexor_stretch.mark"] =
	"Back knee down, tailbone tucked, front shin upright, chest tall, hands off the knee."
move("hip_cars", "Hip circles", "One hand on the desk for balance.",
	"Lift one knee and draw the biggest slow circle you can. Breathe slowly.",
	"Stop if your hip pinches.")
guide("hip_cars", {
	"Stand side-on to the desk with one hand on it and the other hand on your hip.",
	"Stand tall and lift your outside knee in front of you to hip height.",
	"Draw the knee out to the side, round behind you and down, taking about 8 s a circle.",
	"Breathe slowly the whole time and keep your standing leg long.",
	"The clock times each leg, then counts you in to turn round.",
}, {
	"Keep your hips and chest facing forward so the thigh turns in the hip, not your body.",
	"Make the circle a little bigger each time while your pelvis stays level.",
	"Press the standing foot into the floor to stay tall and steady.",
})
move("worlds_greatest", "World's greatest stretch", "A body length of clear floor.",
	"Lunge, hand down, rotate the other arm to the ceiling. Breathe out as you open.",
	"Ease off if your back or hip pinches.")
guide("worlds_greatest", {
	"Stand with a body length of clear floor ahead, feet hip-width apart.",
	"Step one foot far forward into a long lunge, back knee off the floor and back leg long.",
	"Plant the hand opposite your front leg flat on the floor under your shoulder.",
	"Breathe out and reach the other arm up to the ceiling, eyes following the hand.",
	"Lower the arm and step back up; the clock times each leg, then counts you in to switch.",
}, {
	"Push the back heel away and squeeze that seat to feel the front of the hip open.",
	"Let the front knee drift out a little so your chest has room to turn.",
	"Turn from your upper back while the hand on the floor presses firmly down.",
})
move("calf_stretch", "Calf stretch at the wall", "Hands on the wall, one foot back.",
	"Back heel down, lean in gently. Breathe slowly.", "Ease off if your heel or ankle hurts.")
guide("calf_stretch", {
	"Face the wall with your hands flat on it at shoulder height, arms long.",
	"Step one foot back about half a metre, toes pointing at the wall, front knee soft.",
	"Keep the back knee straight and press that heel flat into the floor.",
	"Lean in slowly as one straight line from back heel to head, then ease back out.",
	"Breathe slowly; the clock times each leg, then counts you in to switch.",
}, {
	"Point the back toes straight ahead, not turned out, to keep the stretch in the calf.",
	"Keep your hips in line with your body rather than bending at the waist.",
	"Lean a little further on each breath out, only as far as a steady pull.",
})

-- Yoga
move("downward_dog", "Downward dog", "Hands and feet on a floor that does not slip.",
	"Hips high, press the floor away, heels toward the mat. Breathe slowly.",
	"Come down if your wrists hurt.")
guide("downward_dog", {
	"Start on hands and knees, hands shoulder-width apart, fingers spread wide.",
	"Tuck your toes, lift your knees and push your hips up and back into an upside-down V.",
	"Straighten your arms and legs so your arms and back form one long line.",
	"Breathe slowly, easing your heels towards the floor on each breath out.",
	"Stay and breathe at an easy pace for the time shown, then lower your knees slowly.",
}, {
	"Press the whole hand into the floor, most of all the base of the first finger.",
	"Let your head hang between your upper arms instead of looking at your feet.",
	"Think of pushing your hips up and back rather than pressing your chest down.",
})
L["move.downward_dog.mark"] = "Knees straight, back flat, hips high, arms in line with your ears."
move("cat_cow", "Cat-cow", "Hands and knees on the floor, a folded towel under your knees if it is hard.",
	"Round your back up, then let it sink, slowly. Breathe in on the sink.",
	"Ease off if your back or wrists hurt.")
guide("cat_cow", {
	"Set your hands under your shoulders and your knees under your hips, arms straight.",
	"Breathe out and round your back up to the ceiling, tucking your chin and tailbone.",
	"Breathe in and let your belly sink, chest forward, eyes looking just ahead.",
	"Take about 3 s each way and pause briefly at each end.",
	"Keep moving between the two at an easy pace for the time shown.",
}, {
	"Push the floor away as you round up to spread your shoulder blades wide.",
	"Keep your weight even between your hands and knees as your back moves.",
	"Start each change at your tailbone and let it ripple up to your head.",
})
move("cobra", "Cobra", "Lie face down on the floor, hands under your shoulders.",
	"Lift your chest gently, hips down, shoulders away from ears. Breathe slowly.",
	"Ease off if your lower back pinches.")
guide("cobra", {
	"Lie with your legs long, feet hip-width apart, the tops of your feet on the floor.",
	"Keep your elbows bent and close to your sides, forehead resting on the floor.",
	"Press your hands down gently and lift your chest slowly, hips on the floor.",
	"Breathe slowly for a breath or two at the top, then lower slowly.",
	"Keep lifting and lowering at an easy pace for the time shown.",
}, {
	"Lead with your chest, not your chin, and keep the back of your neck long.",
	"Press the tops of your feet and your hips into the floor to lift from the upper back.",
	"Draw your shoulders back and down as you lift so the chest opens forward.",
})
L["move.cobra.mark"] = "Hips on the floor, belly button off it, elbows bent by your sides, shoulders down."
move("childs_pose", "Child's pose", "Kneel on the floor, a folded towel under your knees if it is hard.",
	"Sit back on your heels, arms long in front. Breathe into your back.",
	"Ease off if your knees hurt.")
guide("childs_pose", {
	"Kneel with your big toes together, knees wide and the tops of your feet flat on the floor.",
	"Sit your hips back towards your heels and fold your chest down between your knees.",
	"Rest your forehead on the floor and reach your arms long in front, palms down.",
	"Breathe slowly into your back, feeling it widen on each breath in.",
	"Stay in the pose and breathe at an easy pace for the time shown.",
}, {
	"Press your palms gently into the floor and reach forward to lengthen your sides.",
	"Let your hips sink a little closer to your heels on each breath out.",
	"Let your head rest heavy on the floor so your neck and shoulders let go.",
})
move("thread_needle", "Thread the needle",
	"Hands and knees on the floor, a folded towel under your knees if it is hard.",
	"Slide one arm under the other, shoulder down, then open up. Breathe slowly.",
	"Ease off if your neck or shoulder pinches.")
guide("thread_needle", {
	"Set your hands under your shoulders and your knees under your hips, back flat.",
	"Slide one arm, palm up, under the other along the floor until that shoulder rests down.",
	"Let your head turn with your chest, relax your neck, and breathe slowly there.",
	"Come back up, then reach the same arm to the ceiling, chest turning, eyes on the hand.",
	"The clock times each side, then counts you in to switch arms.",
}, {
	"Keep your hips right above your knees so the twist comes from your upper back.",
	"Press the supporting hand into the floor to turn a little further.",
	"Let your eyes follow the moving hand the whole way to turn your upper back fully.",
})
move("low_lunge", "Low lunge", "Back knee on a folded towel.",
	"Front knee over ankle, hips forward, arms up if steady. Breathe slowly.",
	"Ease off if your knee hurts.")
guide("low_lunge", {
	"Kneel on the towel on one knee and step the other foot forward a long stride, foot flat.",
	"Rest your hands on your front thigh and stack the front knee over the ankle.",
	"Shift your hips slowly forward until you feel a stretch at the front of the back hip.",
	"Reach both arms up beside your ears if you feel steady, and breathe slowly.",
	"The clock times each side, then counts you in to switch legs.",
}, {
	"Tuck your tailbone a little before you shift to feel the stretch higher in the hip.",
	"Keep your chest lifted over your hips instead of leaning onto the front thigh.",
	"Keep your front knee in line with your middle toes as your hips move forward.",
})
L["move.low_lunge.mark"] = "Back knee down, hips sunk lower than the front knee, chest tall, arms up."
move("pigeon", "Pigeon", "On the floor, a cushion under your hip if it lifts.",
	"Front shin across, back leg long, fold forward gently. Breathe slowly.",
	"Ease off if your knee hurts.")
guide("pigeon", {
	"Sit with one shin across in front of you and slide the other leg long behind you.",
	"Rest the top of the back foot on the floor and set your hips square to the front.",
	"Sit tall on your fingertips beside your hips, then walk your hands forward to fold.",
	"Breathe slowly in the fold, then walk your hands back to sit tall again.",
	"The clock times each side, then counts you in to switch legs.",
}, {
	"Keep your hips square to the front, not rolled onto the front hip.",
	"Pull the toes of the front foot back towards the shin to keep that ankle steady.",
	"Fold from your hips with a long back before you let your head drop.",
})
L["move.pigeon.mark"] = "Front shin across, both hips level and down, no cushion, back leg long behind you."
move("figure_four", "Figure-four at the desk", "Sit tall on your chair, feet flat.",
	"Ankle over the other knee, lean forward with a flat back. Breathe slowly.",
	"Ease off if your knee hurts.")
guide("figure_four", {
	"Sit near the front edge of your chair with both feet flat and hip-width apart.",
	"Cross one ankle over the other knee, resting it just past the knee on your thigh.",
	"Rest your hands on the crossed shin and let the raised knee sink towards the floor.",
	"Hinge forward from your hips with a flat back until you feel a stretch in your seat.",
	"Breathe slowly; the clock times each side, then counts you in to switch.",
}, {
	"Pull the toes of the crossed foot back towards your shin to keep the ankle steady.",
	"Lead with your chest rather than your head so your back stays flat.",
	"Sit evenly on both sit bones and keep them level as you lean.",
})
move("forward_fold", "Standing forward fold", "Feet hip-width, knees straight.",
	"Knees straight, fold from the hips, let your head hang. Breathe slowly.")
guide("forward_fold", {
	"Stand with your feet hip-width apart and your knees straight.",
	"Push your hips back and hinge forward with a flat back, arms hanging loose.",
	"Let your back round and your head hang, fingertips reaching towards the floor.",
	"Keep your knees straight and breathe slowly in the fold for the time shown.",
	"Then roll up slowly through your back, head coming up last.",
}, {
	"Let your arms and head hang heavy so your neck and shoulders let go.",
	"Keep your weight over the middle of your feet, not back on your heels.",
	"Start the fold by sending your hips back before your back rounds.",
})
move("spinal_twist", "Seated spinal twist", "Sit tall on the floor.",
	"Sit tall, then turn toward your bent knee. Breathe out as you turn.",
	"Ease off if your back pinches.")
guide("spinal_twist", {
	"Sit with one leg long and bend the other knee up, its foot flat by the long thigh.",
	"Plant the hand on the bent-knee side on the floor just behind your hips.",
	"Hook your other elbow outside the bent knee and sit up tall through the top of your head.",
	"Breathe in to sit tall, then breathe out and turn your chest and head to the bent knee.",
	"The clock times each side, then counts you in to switch legs and turn the other way.",
}, {
	"Grow taller on every breath in and turn a little further on every breath out.",
	"Keep both sit bones heavy on the floor as your chest turns.",
	"Turn your chest first and let your head follow last.",
})
L["move.spinal_twist.mark"] = "Sitting tall, elbow outside the bent knee, chest turned past that knee."
move("warrior2", "Warrior II", "Clear space, feet wide.",
	"Front knee bent over ankle, arms long, gaze over your hand. Breathe slowly.",
	"Ease off if your front knee caves in.")
guide("warrior2", {
	"Step your feet wide apart, a little more than the length of one leg, clear of furniture.",
	"Turn the front foot out to point ahead and the back foot in slightly.",
	"Lift your arms straight out to the sides at shoulder height, palms down.",
	"Bend the front knee over the ankle, keep your hips level, and look over your front hand.",
	"Breathe slowly; the clock times each side, then counts you in to switch.",
}, {
	"Press the outer edge of your back foot into the floor to keep the back leg strong.",
	"Stack your shoulders over your hips so your body does not lean towards the front leg.",
	"Reach both arms long away from each other, shoulders low.",
})
move("chair_pose", "Chair pose", "Feet together, the desk within reach.",
	"Sit back as if onto a chair, arms up. Breathe slowly.", "Ease off if your knees ache.")
guide("chair_pose", {
	"Stand with your feet together and the desk beside you within easy reach.",
	"Lift your arms up beside your ears, palms facing in.",
	"Bend your knees and send your hips back and down, chest lifted over your thighs.",
	"Breathe slowly and keep the pose steady, knees and chest still.",
	"Stay in the pose for the time shown, then stand up slowly.",
}, {
	"Keep your weight in your heels, so you could lift your toes.",
	"Keep your knees together and pointing straight ahead as you sit back.",
	"Draw your belly in lightly so your lower back does not arch as your arms lift.",
})
move("tree_pose", "Tree pose", "Fingertips near the desk, ready to touch it.",
	"Foot on the inner calf or thigh, never the knee, hands together. Breathe slowly.",
	"Put the foot down when you sway.")
guide("tree_pose", {
	"Stand side-on to the desk, feet together, fingertips hovering just above it.",
	"Shift your weight onto one foot and press it firmly into the floor.",
	"Lift the other foot and set its sole on your inner calf, that knee turned out to the side.",
	"Bring your hands together at your chest and breathe slowly.",
	"The clock times each side, then counts you in to switch feet.",
}, {
	"Fix your eyes on one still spot at eye level to steady your balance.",
	"Press the foot and the calf into each other to lock the pose in place.",
	"Keep your hips level and your standing knee soft, not locked.",
})
move("plank_dog_flow", "Plank to downward dog", "Hands and feet on a floor that does not slip.",
	"Flow from a plank up to downward dog and back. Breathe out as you lift.",
	"Ease off if your wrists hurt.")
guide("plank_dog_flow", {
	"Start in a high plank, hands under your shoulders and fingers spread wide.",
	"Hold one straight line from your heels to your head, belly firm.",
	"Breathe out and push your hips up and back into downward dog.",
	"Breathe in and shift forward into the plank, shoulders over wrists.",
	"Keep flowing between the two at an easy pace for the time shown.",
}, {
	"Keep your hands and feet where they are and let your hips do the moving.",
	"Press the floor away in both positions so your shoulders stay broad.",
	"Let your head move with your back, eyes between your hands in the plank.",
})
move("sun_salutation", "Sun salutation A", "A body length of clear floor.",
	"Three rounds: reach, fold, plank, lower, cobra, dog, step up. Breathe with each move.")
guide("sun_salutation", {
	"Stand with room behind you, feet together, and breathe in as you reach your arms up.",
	"Breathe out and fold forward, then put your hands flat on the floor by your feet.",
	"Step back one foot then the other to a plank, then lower yourself slowly to the floor.",
	"Breathe in to lift into cobra, then breathe out and push your hips up into downward dog.",
	"Step up to your hands, rise with arms up, and repeat at an easy pace for the time shown.",
}, {
	"Let each breath start each move, so the flow never runs ahead of your breathing.",
	"Bend your knees in the fold so your hands reach the floor with a long back.",
	"Lower from the plank in one straight line, elbows close to your sides.",
})

-- Equipment: pull-up bar
move("dead_hang", "Dead hang", "A bar fixed firmly; a stool to step down on.",
	"Hang with straight arms, shoulders gently engaged. Count out loud.", "Stop when your grip starts to slip.")
guide("dead_hang", {
	"Check the bar is fixed firmly and set the stool under it to step up and down.",
	"Grip the bar with palms facing away, hands a little wider than your shoulders.",
	"Lift your feet and hang with straight arms, knees slightly bent, feet together.",
	"Draw your shoulders a little away from your ears and breathe slowly throughout.",
	"Count out loud while the clock times the hang, from the moment your feet leave the stool.",
}, {
	"Wrap your thumbs around the bar and squeeze it with your whole hand.",
	"Keep your ribs down and your legs still so your body hangs without swinging.",
	"Relax your neck and look straight ahead, not up at the bar.",
})
move("scap_pulls", "Scapular pulls", "A bar fixed firmly; a stool to step down on.",
	"Hang, then pull your shoulder blades down without bending your arms. Count out loud.",
	"Stop when your arms bend.")
guide("scap_pulls", {
	"Check the bar is fixed firmly and set the stool under it to step down on.",
	"Grip with palms facing away, hands a little wider than your shoulders, and hang.",
	"With straight arms, pull your shoulder blades down so your body rises a little.",
	"Pull down, pause, then let your shoulders rise back slowly.",
	"Breathe out as you pull down and count each rep out loud.",
}, {
	"Think of putting your shoulder blades into your back pockets.",
	"Keep your elbows locked so the whole lift comes from your shoulders.",
	"Let your chest lift slightly towards the bar at the top of each pull.",
})
move("pullup_negative", "Pull-up negative", "A bar fixed firmly; a stool to step up on.",
	"Start at the top, lower yourself as slowly as you can. Count out loud.",
	"Stop when you drop fast.")
guide("pullup_negative", {
	"Check the bar is fixed firmly and set a stool under it, high enough to get your chin over.",
	"Grip with palms facing away, step up and bring your chin over the bar, elbows tight.",
	"Lift your feet and lower yourself as slowly as you can to straight arms.",
	"Step back onto the stool to reset at the top, then lower again.",
	"Breathe out slowly on the way down and count each lowering out loud as one rep.",
}, {
	"Keep the speed even all the way down, no faster near the bottom.",
	"Squeeze your legs together and brace your belly so you do not swing.",
	"Keep your shoulders down and back as you lower, away from your ears.",
})
move("flexed_arm_hang", "Flexed-arm hang", "A bar fixed firmly; a stool to step up on.",
	"Step up, chin over the bar, elbows tight to your sides, hold. Count out loud, breathe.",
	"Stop when your chin sinks below the bar.")
guide("flexed_arm_hang", {
	"Check the bar is fixed firmly and set a stool under it, high enough to get your chin over.",
	"Grip with palms facing away, step up and bring your chin over the bar, elbows tight.",
	"Lift your feet and hold there, chest close to the bar, shoulders pulled down.",
	"Keep breathing slowly; step back onto the stool when your chin sinks below the bar.",
	"The clock counts up from the moment your feet leave the stool.",
}, {
	"Squeeze the bar hard and keep your elbows pressed to your ribs.",
	"Cross your ankles and brace your belly so your body hangs still.",
	"Keep your neck long; do not crane your chin up to stay over the bar.",
})
move("pullup_negative_slow", "Pull-up lowering, 5 seconds", "A bar fixed firmly; a stool to step up on.",
	"Start at the top, lower over five slow beats to straight arms. Count out loud.",
	"Stop when a lowering takes less than 5 seconds.")
guide("pullup_negative_slow", {
	"Check the bar is fixed firmly and set a stool under it, high enough to get your chin over.",
	"Grip with palms facing away, step up and bring your chin over the bar, elbows tight.",
	"Lift your feet and lower over five slow beats all the way to straight arms.",
	"Step back onto the stool to reset at the top, breathing out slowly as you lower.",
	"Count only the lowerings that took 5 s or more, each one out loud as one rep.",
}, {
	"Count the five beats out loud on the way down; it keeps you breathing.",
	"Fight hardest in the bottom half, where the lowering wants to speed up.",
	"Keep your shoulders down and your legs still, with no swinging.",
})
move("pullup_assisted", "Assisted pull-up", "A bar fixed firmly; a band or one foot on a chair without wheels.",
	"Pull your chin over the bar, lower with control. Count out loud, breathe out as you pull.",
	"Stop when you kick or cannot reach the top.")
guide("pullup_assisted", {
	"Check the bar is firm, then hang a band from it or set a chair without wheels below.",
	"Grip with palms away, hands a bit wider than your shoulders, one foot on the seat or band.",
	"Start with straight arms, pull your chin over the bar, then lower slowly.",
	"Push with your foot only as much as you need to reach the top.",
	"Breathe out as you pull and count each rep out loud at the top.",
}, {
	"Pull your elbows down towards your ribs as if bending the bar.",
	"Lower all the way to straight arms before the next pull.",
	"Keep your chest up and aim it at the bar as you pull.",
})
move("half_pullup", "Half pull-up", "A bar fixed firmly; a stool so you start with your elbows at a right angle.",
	"From bent arms, chin over the bar, back to a right angle. Count out loud, breathe out up.",
	"Stop when your chin no longer clears the bar or you kick.")
guide("half_pullup", {
	"Set a stool under a firmly fixed bar so you grip it with your elbows at a right angle.",
	"Grip with palms facing away, hands a little wider than your shoulders, and lift your feet.",
	"Pull your chin over the bar, breathing out.",
	"Lower until your elbows are at a right angle, upper arms level, then pull again.",
	"Each chin over the bar counts as one; step back onto the stool when you are done.",
}, {
	"Stop each lowering at the right angle: going lower makes it a full pull-up.",
	"Keep your legs still and together: a kick or a swing does not count.",
	"Pull your shoulders down before each pull, away from your ears.",
})
move("pullup", "Pull-up", "A bar fixed firmly; a stool to step down on.",
	"From a dead hang, chin over the bar, lower fully. Count out loud, breathe out as you pull.",
	"Stop when you kick or cannot reach the top.")
guide("pullup", {
	"Check the bar is fixed firmly and set a stool under it to step down on.",
	"Grip with palms facing away, hands a little wider than your shoulders.",
	"From straight arms, pull your chin over the bar.",
	"Lower all the way to straight arms, breathing in as you go down.",
	"Breathe out as you pull and count each rep out loud at the top.",
}, {
	"Start each rep by pulling your shoulders down, then drive your elbows to your ribs.",
	"Keep your legs together and a little forward so your body does not swing.",
	"Aim your chest at the bar rather than stretching your chin up.",
})
move("hanging_knee_raise", "Hanging knee raise", "A bar fixed firmly; a stool to step down on.",
	"Hang, knees to your chest, lower slowly. Count out loud, breathe out as you lift.",
	"Stop when you swing.")
guide("hanging_knee_raise", {
	"Check the bar is fixed firmly and set a stool under it to step down on.",
	"Grip with palms facing away and hang with straight arms, shoulders gently engaged.",
	"Lift your knees up towards your chest, tilting your hips up at the top.",
	"Lower your legs slowly until they hang straight and still.",
	"Breathe out as you lift and count each rep out loud.",
}, {
	"Curl your hips up at the top so your belly does the lift, not a swing.",
	"Start each lift from a still hang, never from the swing of the last one.",
	"Squeeze the bar hard and keep your shoulders away from your ears.",
})

-- Equipment: bands
move("pull_apart", "Band pull-apart", "A band with no nicks or tears.",
	"Arms straight in front, pull the band apart to your chest. Breathe out as you pull.",
	"Stop when your shoulders shrug.")
guide("pull_apart", {
	"Stand tall, feet hip-width, holding the band in both hands, shoulder-width, palms down.",
	"Lift your straight arms in front to shoulder height with the band a little taut.",
	"Breathe out and pull the band apart until it touches your chest, arms still straight.",
	"Squeeze your shoulder blades together, then return slowly.",
	"Count one rep each time your hands come back in front of your shoulders.",
}, {
	"Keep your shoulders down and away from your ears as the band stretches.",
	"Move your hands straight out to the sides, level with your shoulders, not down.",
	"Keep your ribs down so your lower back does not arch as the band opens.",
})
move("band_row", "Band row", "A band anchored in a closed door at chest height.",
	"Pull your elbows back, squeeze your shoulder blades. Breathe out as you pull.",
	"Stop when your shoulders shrug.")
guide("band_row", {
	"Anchor the band on the hinge side of a door that opens away from you, then shut it.",
	"Hold one end in each hand, palms facing in, and step back until the band is taut.",
	"Stand tall, feet hip-width, knees soft, arms straight in front at chest height.",
	"Breathe out and pull your elbows back past your ribs, then return slowly.",
	"Count one rep each time your arms are straight again in front of you.",
}, {
	"Start each pull by drawing your shoulder blades back and down, then bend the elbows.",
	"Keep your wrists straight so the band pulls in line with your forearms.",
	"Stay tall and still, with no lean back to help the pull.",
})
move("band_curl", "Band curl", "Stand on the band, feet hip-width.",
	"Elbows at your sides, curl your hands up, lower slowly. Breathe out as you curl.",
	"Stop when your elbows swing forward.")
guide("band_curl", {
	"Stand on the middle of the band with both feet and hold one end in each hand.",
	"Let your arms hang straight, palms facing forward, elbows touching your sides.",
	"Breathe out and curl your hands up to your shoulders.",
	"Squeeze at the top, then lower slowly until your arms are straight.",
	"Count one rep each time your arms are straight again at the bottom.",
}, {
	"Keep your elbows pinned to your sides so only your forearms move.",
	"Keep your wrists straight, in line with your forearms, all the way up.",
	"Stand tall and still, with no lean back to swing the band up.",
})
move("face_pull", "Face pull", "A band anchored in a closed door at head height.",
	"Pull the band to your face, elbows high, hands apart. Breathe out as you pull.",
	"Stop when your shoulders shrug.")
guide("face_pull", {
	"Anchor the band on the hinge side of a door that opens away from you, then shut it.",
	"Hold one end in each hand and step back until the band is taut, arms reaching up to it.",
	"Breathe out and pull the band towards your forehead, elbows high, hands spreading apart.",
	"Pause with your hands beside your ears, then let your arms straighten again slowly.",
	"Count one rep each time your arms are straight again towards the door.",
}, {
	"Keep your elbows at or above shoulder height all the way through the pull.",
	"Finish with your fists turned back, as if flexing both arms beside your head.",
	"Stand tall with your ribs down so the pull comes from your upper back.",
})
move("pallof_press", "Pallof press", "A band anchored in a closed door at chest height, side on.",
	"Press your hands out, resist the twist, bring them back. Breathe out as you press.",
	"Stop when your body turns toward the door. Count the weaker side.")
guide("pallof_press", {
	"Anchor the band on the hinge side of a door that opens away from you, then shut it.",
	"Stand side-on, band held at your chest in both hands, and step away until it is taut.",
	"Set your feet wider than hip-width, knees soft, hips and chest facing straight ahead.",
	"Breathe out and press your hands straight out, hold 1 s, then bring them back.",
	"Count each return to your chest; finish one side, press Switch sides, then turn round.",
}, {
	"Brace your belly as if someone might nudge you, so nothing turns as the arms go out.",
	"Keep your hands in line with the middle of your chest, not drifting towards the door.",
	"Squeeze your seat and press your feet into the floor to stand firm.",
})
move("band_good_morning", "Band good morning", "Stand on the band, loop behind your neck on a towel.",
	"Hips back, back flat, bow and rise. Breathe out as you rise.", "Stop when your back rounds.")
guide("band_good_morning", {
	"Stand on the band, feet hip-width, with a folded towel over the back of your neck.",
	"Loop the band over the towel and hold it with both hands just in front of your shoulders.",
	"With soft knees and a flat back, push your hips back and bow.",
	"Breathe out and drive your hips forward to stand tall.",
	"Count one rep each time you are back up standing tall.",
}, {
	"Push your hips back as if shutting a drawer behind you with your seat.",
	"Keep your shins nearly still so the bend comes from your hips, not your knees.",
	"Squeeze your seat at the top to finish each rep standing tall.",
})
move("monster_walk", "Monster walk", "A small band around your knees.",
	"Half squat, step wide and forward, knees pushing out. Breathe out with each step.",
	"Stop when your knees fall inward.")
guide("monster_walk", {
	"Clear a few metres of floor to walk along and slip the band just above your knees.",
	"Sink into a half squat, feet a bit wider than your hips, hands on hips, chest forward.",
	"Step one foot forward and out wide, then the other, about one step a second.",
	"Keep the band taut with your knees pushing out, and breathe out with each step.",
	"Count each step with either foot as one rep, and turn round when you run out of room.",
}, {
	"Stay at the same squat height the whole way, with no bobbing up between steps.",
	"Keep your toes pointing forward so your hips do the stepping.",
	"Keep each step low and smooth so the band stays taut and never snaps back.",
})

-- Equipment: kettlebell
move("kb_goblet_squat", "Goblet squat", "A kettlebell of 8-16 kg to start, held at your chest by the horns.",
	"Sit down between your heels, chest up, stand. Count out loud, breathe out as you stand.",
	"Stop when your back rounds or your knees cave in.")
guide("kb_goblet_squat", {
	"Stand with your feet about shoulder-width apart, toes turned out a little.",
	"Hold the bell upright against your chest, hands on the horns, elbows pointing down.",
	"Sit straight down between your heels until your hips drop below your knees.",
	"Stand tall again from the bottom, breathing out as you rise.",
	"Count one rep out loud each time you stand fully tall.",
}, {
	"Keep the bell pinned to your chest so it never pulls you forward.",
	"Push your knees out over your toes as you sit, and keep your heels on the floor.",
	"Let your elbows brush the inside of your knees at the bottom as a depth check.",
})
move("kb_swing", "Kettlebell swing", "A kettlebell of 8-16 kg to start, clear space in front and behind you.",
	"Hike the bell back, snap your hips, float it to chest height. Count out loud, breathe out.",
	"Stop when you lift with your arms or your back rounds.")
guide("kb_swing", {
	"Set the bell a short step in front of you, feet a bit wider than your hips.",
	"Push your hips back and grip the handle with both hands, arms long, back flat.",
	"Hike the bell back high between your thighs, then snap your hips forward to stand tall.",
	"Let the bell float up to chest height, then guide it back down between your thighs.",
	"Breathe in as it drops, breathe out hard at the snap, and count each float out loud.",
}, {
	"Keep your wrists high between your thighs on the hike, not down by your knees.",
	"Wait for the bell to fall toward you before you push your hips back.",
	"At the top, stand straight and tall with your seat and stomach squeezed tight.",
})
move("kb_suitcase_carry", "Kettlebell suitcase carry", "A clear hallway.",
	"The bell in one hand, stand tall, walk steadily. Breathe slowly.",
	"Stop when you lean to one side.")
guide("kb_suitcase_carry", {
	"Clear the hallway of shoes, bags and cables, and pick a turning point at each end.",
	"Pick up the bell beside one foot with a flat back, then stand tall, arm long.",
	"Walk with normal, steady steps, turn slowly at each end, and keep the bell still.",
	"Breathe slowly and freely the whole time.",
	"The clock times each hand, then counts you in to switch hands.",
}, {
	"Squeeze the handle hard so the bell does not swing into your leg.",
	"Reach the top of your head toward the ceiling so both shoulders stay level.",
	"Let your free arm swing easily, the way it does on a normal walk.",
})
implement("kb_suitcase_carry", "kettlebell", "A kettlebell, a clear hallway.")
implement("kb_suitcase_carry", "dumbbells", "One dumbbell, a clear hallway.", {
	"Clear the hallway of shoes, bags and cables, and pick a turning point at each end.",
	"Pick up the dumbbell beside one foot with a flat back, then stand tall, arm long.",
	"Walk with normal, steady steps, turn slowly at each end, and keep the dumbbell still.",
	"Breathe slowly and freely the whole time.",
	"The clock times each hand, then counts you in to switch hands.",
})
move("kb_farmer_march", "Kettlebell farmer march", "The bell held at your side.",
	"March on the spot, stand tall, shoulders down. Breathe slowly.", "Stop when you lean or slump.")
guide("kb_farmer_march", {
	"Stand where you have a little room around you, feet hip-width apart.",
	"Hold the bell at your side, arm long, shoulders down and level, chest tall.",
	"March on the spot at a steady beat, lifting each knee in front of you.",
	"Breathe slowly and freely, and never hold your breath.",
	"The clock times each hand, then counts you in to switch the bell over.",
}, {
	"Squeeze the handle hard and keep the bell still against your leg.",
	"Stand tall on the standing leg each step so your hips stay level.",
	"Let your free arm swing gently in time with your knees.",
})
implement("kb_farmer_march", "kettlebell", "The bell held at your side.")
implement("kb_farmer_march", "dumbbells", "One dumbbell held at your side.", {
	"Stand where you have a little room around you, feet hip-width apart.",
	"Hold the dumbbell at your side, arm long, shoulders down and level, chest tall.",
	"March on the spot at a steady beat, lifting each knee in front of you.",
	"Breathe slowly and freely, and never hold your breath.",
	"The clock times each hand, then counts you in to switch the dumbbell over.",
})
move("kb_halo", "Kettlebell halo", "A light bell held upside down by the horns.",
	"Circle it slowly around your head, ribs down. Breathe slowly.", "Stop if your shoulder pinches.")
guide("kb_halo", {
	"Stand with feet hip-width apart, knees soft, the bell held bottom up at chin height.",
	"Squeeze your seat and keep your ribs down so your lower back stays still.",
	"Circle the bell slowly around your head, passing just behind your neck.",
	"Breathe slowly through your nose, in for half a circle and out for the other half.",
	"Move at an easy pace for the time shown, changing direction after each circle.",
}, {
	"Keep your head still and tall, and let the bell travel around it.",
	"Keep the bell close, just clearing your hair as it goes around.",
	"Hold the horns firmly so the bell stays upside down the whole way.",
})

-- Equipment: dumbbells
move("db_lateral_raise", "Lateral raise", "Light dumbbells of 2-5 kg each to start, feet hip-width.",
	"Raise your arms to shoulder height, elbows soft, lower slowly. Breathe out as you lift.",
	"Stop when you swing or shrug.")
guide("db_lateral_raise", {
	"Stand tall with the dumbbells at your sides, palms facing in, knees soft.",
	"Keep a slight bend in your elbows and your arms a little in front of your body.",
	"Raise your arms out to the sides until your elbows reach shoulder height.",
	"Breathe out as you lift, then breathe in as you lower slowly.",
	"Count one rep each time your arms reach shoulder height.",
}, {
	"Lead with your elbows, as if pushing them out toward the walls.",
	"Keep your shoulders down and your neck long as your arms rise.",
	"Keep your palms facing the floor at the top, not tipped up.",
})
move("db_hammer_curl", "Hammer curl", "Dumbbells of 4-8 kg each to start, palms facing in.",
	"Elbows at your sides, curl up, lower slowly. Breathe out as you curl.",
	"Stop when your elbows swing forward.")
guide("db_hammer_curl", {
	"Stand tall, feet hip-width apart, a dumbbell in each hand at your sides, palms in.",
	"Pin your elbows to your sides and keep your wrists straight, thumbs pointing forward.",
	"Curl both dumbbells up toward your shoulders, then squeeze.",
	"Breathe out as you curl, then breathe in as you lower slowly to straight arms.",
	"Count one rep each time the dumbbells reach the top.",
}, {
	"Squeeze the handles hard to feel your forearms work.",
	"Keep your upper arms still so only your forearms move.",
	"Straighten your arms fully at the bottom before each new curl.",
})
move("db_goblet_squat", "Dumbbell goblet squat", "One dumbbell of 8-16 kg to start, held at your chest.",
	"Sit down between your heels, chest up, stand. Count out loud, breathe out as you stand.",
	"Stop when your back rounds or your knees cave in.")
guide("db_goblet_squat", {
	"Stand with your feet about shoulder-width apart, toes turned out a little.",
	"Hold the dumbbell upright at your chest, both palms cupped under the top end.",
	"Sit straight down between your heels until your hips drop below your knees.",
	"Stand tall again from the bottom, breathing out as you rise.",
	"Count one rep out loud each time you stand fully tall.",
}, {
	"Keep your elbows pointing down and tucked in to hold the dumbbell steady.",
	"Push the floor away through your whole foot as you stand, heels down.",
	"Keep your chest high enough that the dumbbell stays upright the whole time.",
})
move("db_reverse_lunge", "Dumbbell reverse lunge", "Dumbbells of 4-10 kg each at your sides, clear space behind you.",
	"Step back, back knee toward the floor, return. Count out loud, breathe out as you rise.",
	"Stop when your front knee caves in. Count the weaker side.")
guide("db_reverse_lunge", {
	"Stand tall, feet hip-width apart, a dumbbell in each hand, palms facing in.",
	"Take a long step back with one foot and land on the ball of that foot.",
	"Lower the back knee until it hovers just above the floor.",
	"Push through the front heel to step back up to standing, breathing out as you rise.",
	"Count each return out loud; every rep stepping back with one leg, then press Switch sides.",
}, {
	"Keep most of your weight on the front foot, the back foot only balances.",
	"Step back far enough that your front shin stays upright at the bottom.",
	"Let the dumbbells hang still at your sides, arms long and relaxed.",
})

-- Equipment: barbell
move("landmine_press", "Landmine press", "One end of the bar anchored in a corner, 5-15 kg of plates on the other.",
	"Press the bar end up and forward, ribs down. Count out loud, breathe out as you press.",
	"Stop when your back arches. Count the weaker side.")
guide("landmine_press", {
	"Wedge one bar end into a room corner, wrapped in a towel, so it cannot slide.",
	"Face the corner, hold the free end at your shoulder in one hand, and step that foot back.",
	"Lean slightly toward the corner so your body makes one line from back heel to head.",
	"Press the bar up and forward until your arm is straight, then lower it back.",
	"Breathe out as you press and count each rep; one side, press Switch sides, then the other.",
}, {
	"Reach forward at the top as if pushing the bar end away from you.",
	"Keep your ribs down and your seat squeezed so your back stays long.",
	"Start each press with your elbow slightly in front of the bar end.",
})
move("bb_rdl", "Barbell deadlift, light", "A light bar from the rack, 20-40 kg with its plates to start, collars on.",
	"Hips back, bar close to your legs, back flat, stand. Count out loud, breathe out up.",
	"Stop when your back rounds.")
guide("bb_rdl", {
	"Set the bar at mid-thigh height and grip it just outside your thighs, palms facing you.",
	"Lift it out, step back, feet hip-width, knees soft, shoulders drawn back.",
	"Push your hips back, sliding the bar down your thighs past your knees.",
	"Stand back up by driving your hips forward, breathing out as you rise.",
	"Count one rep out loud each time you stand tall, then put the bar back in the rack.",
}, {
	"Keep the bar touching your legs the whole time, as if shaving your thighs.",
	"Keep your knees soft but still, so the movement comes from your hips.",
	"Squeeze the bar hard and keep your arms long, never bending the elbows.",
})
move("bb_squat", "Barbell squat, light", "A light bar in the rack, 20-40 kg to start, with the safeties set.",
	"Bar on your upper back, sit down, chest up, stand. Count out loud, breathe out up.",
	"Stop when your back rounds or your knees cave in.")
guide("bb_squat", {
	"Set the bar at mid-chest height and the safeties just below your lowest squat point.",
	"Step under and rest the bar across your upper back, hands wider than your shoulders.",
	"Lift it out, take two short steps back, feet shoulder-width, toes out a little.",
	"Sit down until your hips are level with your knees, then stand.",
	"Breathe out as you stand, count each rep out loud, and rack the bar when you finish.",
}, {
	"Grip the bar firmly and pull it down into your back so it stays still.",
	"Spread the floor with your feet so your knees follow your toes.",
	"Keep your eyes on a spot on the floor about two metres ahead.",
})
move("bb_press", "Overhead press", "A light bar from the rack, 15-25 kg to start, collars on.",
	"Press the bar overhead, ribs down, head through. Count out loud, breathe out as you press.",
	"Stop when your back arches.")
guide("bb_press", {
	"Set the bar just below shoulder height, grip it just wider than your shoulders, step back.",
	"Stand feet hip-width, the bar on the front of your shoulders, elbows a little forward.",
	"Pull your chin back and press the bar straight up past your face.",
	"Push your head through once the bar clears it, then lower to your shoulders.",
	"Breathe out as you press, count each lockout out loud, and rack the bar at the end.",
}, {
	"Squeeze your seat and legs so the press comes from your arms, not a lean back.",
	"Keep your wrists stacked over your elbows, knuckles toward the ceiling.",
	"Finish with the bar over the middle of your feet, arms by your ears.",
})

-- Equipment: jump rope
move("shadow_skip", "Shadow skipping", "Shoes on, a floor that does not slip.",
	"Skip on the spot turning an imaginary rope. Breathe in rhythm.")
guide("shadow_skip", {
	"Clear an arm span of space around you, away from the desk and chair.",
	"Stand tall, elbows close to your ribs, hands at hip height as if holding handles.",
	"Hop just off the floor on the balls of your feet, turning your wrists in small circles.",
	"Breathe in rhythm with the hops and keep your shoulders relaxed.",
	"Keep a hard pace, too fast to talk.",
}, {
	"Keep the hops low, just high enough for a rope to pass under.",
	"Land softly on the balls of your feet with your knees slightly bent.",
	"Turn from the wrists, not the shoulders, as if the rope were real.",
})
move("rope_bounce", "Rope, slow bounce", "Shoes on, clear space overhead and around.",
	"Small two-foot bounces, one per turn of the rope. Breathe in rhythm.")
guide("rope_bounce", {
	"Fit the rope so the handles reach your armpits when you stand on its middle.",
	"Check that a full turn clears the ceiling, lamps and screens around you.",
	"Stand tall, rope behind your heels, elbows near your ribs, and turn it with your wrists.",
	"Bounce on both feet just high enough to clear the rope, once per turn.",
	"Breathe in rhythm, fast enough that you cannot say a full sentence.",
}, {
	"Land on the balls of your feet with soft knees, as quietly as you can.",
	"Keep your eyes forward on the wall, not down on the rope.",
	"Keep your hands level just in front of your hips so the rope turns evenly.",
})
move("rope_skips", "Rope skipping", "Shoes on, clear space overhead and around.",
	"About two skips a second, wrists turning the rope. Breathe in rhythm.")
guide("rope_skips", {
	"Fit the rope to your armpits and check the space above and around you is clear.",
	"Hold the handles lightly at hip height, elbows tucked in, up on the balls of your feet.",
	"Skip about two times a second, one low jump per turn, the wrists doing the turning.",
	"Breathe in rhythm across several skips and keep your shoulders relaxed.",
	"Keep a hard pace, too fast to talk.",
}, {
	"Make small, fast circles with your wrists while your arms barely move.",
	"Keep each jump a couple of centimetres high, just enough for the rope.",
	"If the rope catches, restart at once in the same rhythm.",
})
move("rope_alternate", "Rope, alternating feet", "Shoes on, clear space overhead and around.",
	"Run through the rope, one foot at a time. Breathe in rhythm.")
guide("rope_alternate", {
	"Fit the rope so the handles reach your armpits, and clear the space around you.",
	"Stand tall on the balls of your feet, hands at hip height, elbows by your ribs.",
	"Step over the rope one foot at a time, like light running on the spot.",
	"Lift each foot just enough to clear the rope, one foot for each turn.",
	"Breathe in rhythm, at a hard pace, too fast to talk.",
}, {
	"Land lightly on the ball of each foot, quick and quiet.",
	"Keep your upper body tall and still while your feet do the running.",
	"Let your wrists set the beat and let your feet follow the rope.",
})
move("double_unders", "Double-unders", "Shoes on, clear space overhead and around.",
	"One jump, two turns of the rope, land softly. Breathe hard and freely.")
guide("double_unders", {
	"Fit the rope so the handles reach your armpits, and check the space overhead.",
	"Stand tall, hands just in front of your hips, elbows close to your ribs.",
	"Jump a little higher than usual, legs nearly straight, and whip your wrists twice.",
	"Land softly on the balls of your feet with a small bend in your knees.",
	"Breathe hard and freely, at a pace that is too hard to talk.",
}, {
	"Speed up your wrists for the second turn, not your arms.",
	"Keep every jump the same height so the rhythm stays even.",
	"Keep your body straight in the air, with no knees tucked up.",
})

-- Equipment: bike, stairs, walking pad, step
move("bike_easy", "Quick spin", "Seat at hip height, feet secure on the pedals.",
	"Spin fast with light resistance. Breathe in rhythm.")
guide("bike_easy", {
	"Set the seat so your knee stays slightly bent at the bottom of each pedal turn.",
	"Sit tall, rest your hands lightly on the bars, and fasten any pedal straps.",
	"Pedal in smooth, round circles as fast as you can keep them smooth, light resistance.",
	"Breathe steadily and keep your shoulders loose.",
	"Spin fast enough that you cannot say a full sentence.",
}, {
	"Push the pedals with the ball of your foot, not the heel.",
	"Keep your knees pointing straight ahead over the pedals.",
	"Relax your hands and upper body so your legs do the work.",
})
move("bike_moderate", "Hard ride", "Seat at hip height, feet secure on the pedals.",
	"Add resistance until it is too hard to talk. Breathe in rhythm.")
guide("bike_moderate", {
	"Set the seat so your knee stays slightly bent when the pedal is at its lowest point.",
	"Sit tall, fasten any pedal straps, and spin easily for a few turns first.",
	"Add resistance step by step until the pedals push back firmly.",
	"Keep a smooth, steady rhythm and breathe in time with your pedalling.",
	"Hold a hard pace, too fast to talk.",
}, {
	"Push down, then pull back through the bottom of each turn.",
	"Keep your hips still on the seat, with no rocking from side to side.",
	"Keep the rhythm even, the same speed from start to finish.",
})
move("bike_sprint", "Bike sprint", "Seat at hip height, feet secure on the pedals.",
	"Pedal as hard as you can against firm resistance. Breathe hard and freely.")
guide("bike_sprint", {
	"Check the seat height and pull any pedal straps tight before you start.",
	"Set firm resistance so your legs cannot spin out of control.",
	"Sit tall, grip the bars and pedal as fast and hard as you can.",
	"Breathe hard and freely, and never hold your breath.",
	"Keep going for the time shown, at a pace far too hard to talk.",
}, {
	"Stay seated and drive each pedal down hard, both legs taking turns.",
	"Pull lightly on the bars to keep your upper body steady.",
	"Keep your shoulders relaxed so all the drive comes from your legs.",
})
move("stairs_brisk", "Fast stair climb", "A handrail within reach, the stairs clear.",
	"Climb fast, walk down carefully. Breathe in rhythm.")
guide("stairs_brisk", {
	"Check that the stairs are clear and lit and the rail is within reach.",
	"Stand tall with your arms bent, eyes on the steps a little ahead of you.",
	"Climb fast one step at a time, driving up through your whole foot.",
	"Walk down carefully, one step at a time, a hand on the rail.",
	"Breathe in rhythm, at a hard pace, too fast to talk on the way up.",
}, {
	"Drive your arms in time with your legs to keep the pace up.",
	"Lift your knee and plant your whole foot, heel included, on every step.",
	"Lean slightly forward from your ankles, not your waist.",
})
move("stairs_two", "Stairs two at a time", "A handrail within reach, the stairs clear.",
	"Take two steps at a time going up, walk down one at a time. Breathe in rhythm.")
guide("stairs_two", {
	"Check that the stairs are clear and lit and the rail is within reach.",
	"Stand tall at the bottom, one hand near the rail, eyes a few steps ahead.",
	"Step up two steps at a time, your whole foot on the step, and drive up through it.",
	"Walk down one step at a time with a hand on the rail.",
	"Breathe in rhythm, at a hard pace, too fast to talk on the way up.",
}, {
	"Push through the heel of the front foot and stand tall on each stride.",
	"Lean your chest slightly forward to keep your weight over the front foot.",
	"Keep a steady rhythm instead of rushing the first few strides.",
})
move("stairs_allout", "Stair climb, all out", "A handrail within reach, three clear flights.",
	"Climb as fast as you safely can, walk down slowly. Breathe hard and freely.")
guide("stairs_allout", {
	"Check that all three flights are clear and lit and the rail runs the whole way.",
	"Start at the bottom, standing tall, one hand close to the rail.",
	"Climb one step at a time as fast as you safely can, arms driving hard.",
	"At the top, turn and walk down slowly, holding the rail on every step.",
	"Breathe hard and freely, at a pace too hard to talk on the way up.",
}, {
	"Plant your whole foot on each step so your heel never hangs off the edge.",
	"Look a few steps ahead as you climb, not down at your feet.",
	"Drive your arms fast and let your legs follow their beat.",
})
move("heel_drops", "Eccentric heel drops", "Balls of your feet on a stair edge, a hand on the rail.",
	"Rise on two feet, lower on one for 3 seconds. Breathe out as you rise.",
	"Stop if your heel or tendon hurts. Count the weaker side.")
guide("heel_drops", {
	"Stand on the bottom stair facing up, heels hanging over the edge, a hand on the rail.",
	"Keep your legs almost straight and your body tall, weight on the balls of your feet.",
	"Breathe out as you rise onto your toes with both feet, then lift one foot behind you.",
	"Lower the standing heel below the step edge over 3 s, then put the other foot back down.",
	"Each slow lowering is one rep: all reps on one leg, press Switch sides, then the other.",
}, {
	"Keep the ankle straight, not rolling in or out, as the heel lowers.",
	"Use the rail for balance only, not to take your weight.",
	"Count the seconds in your head so every lowering takes the same time.",
})
move("step_downs", "Step-downs", "On the bottom stair or a step, a hand near the rail.",
	"Lower one heel slowly to tap the floor, rise. Count out loud, breathe out as you rise.",
	"Stop when your knee caves in. Count the weaker side.")
guide("step_downs", {
	"Stand on the bottom stair facing away from the stairs, a hand near the rail.",
	"Put your whole working foot on the step and let the other foot hang off the edge.",
	"Bend the standing knee and push your hips back, lowering the free heel slowly.",
	"Tap the floor lightly with that heel, then press through the standing foot to rise.",
	"Breathe out as you rise and count each tap; one leg, press Switch sides, then the other.",
}, {
	"Keep the standing knee in line with your middle toes all the way down.",
	"Tap the floor with the heel and never rest your weight on it.",
	"Reach your arms forward as you lower to help your balance.",
})
move("pad_easy", "Quick walk on the pad", "Pad at a quick walking speed, clip or hand rail within reach.",
	"Walk fast, arms swinging. Breathe in rhythm.")
guide("pad_easy", {
	"Clip on the safety key if the pad has one, and stand on the edges beside the belt.",
	"Start the belt at its slowest speed, then step on and walk in its middle.",
	"Raise the speed until you walk fast, heel to toe, arms swinging at your sides.",
	"Breathe in rhythm, walking fast enough that you cannot say a full sentence.",
	"Slow the belt to a stop before you step off at the end.",
}, {
	"Stay in the middle of the belt instead of drifting toward the back edge.",
	"Look straight ahead, not down at your feet.",
	"Stand tall and let each foot roll from heel to toe.",
})
move("pad_brisk", "Brisk walk on the pad", "Clip or hand rail within reach.",
	"Walk briskly, arms swinging. Breathe in rhythm.")
guide("pad_brisk", {
	"Clip the safety key to your clothes if the pad has one, with the rail within reach.",
	"Start slow, then raise the speed in small steps until you walk briskly.",
	"Stay near the middle of the belt, body tall, eyes ahead.",
	"Swing your arms, bent a little, in time with your steps.",
	"Breathe in rhythm, out of breath enough that you cannot say a full sentence.",
}, {
	"Take quicker steps rather than longer ones to add pace.",
	"Drive your elbows back as you swing to keep the rhythm.",
	"Push off the toes of the back foot on every step.",
})
move("pad_fast", "Fast walk on the pad", "Clip or hand rail within reach.",
	"Walk as fast as you can without running. Breathe in rhythm.")
guide("pad_fast", {
	"Clip the safety key to your clothes if the pad has one, and keep the rail in reach.",
	"Start slow and raise the speed in small steps up to your fastest walk.",
	"Keep one foot on the belt at all times so you never break into a run.",
	"Bend your arms and swing them fast and close to your body.",
	"Breathe in rhythm, at the fastest walk you can hold, too out of breath for full sentences.",
}, {
	"Take short, quick steps and land with your foot close under your body.",
	"Drive your elbows back hard and your feet will speed up with them.",
	"Stand tall with a slight lean from your ankles, never from your waist.",
})
move("step_ups", "Step-ups", "A step or box that cannot slide.",
	"Whole foot on the step, drive up, step down with control. Count out loud, breathe out up.",
	"Stop when your knee caves in. Count the weaker side.")
guide("step_ups", {
	"Push the step or box against a wall and check that it does not wobble or slide.",
	"Stand close in front of it and place one whole foot on top, heel included.",
	"Lean slightly forward and drive through that foot to stand tall on the step.",
	"Step down with the other foot first, slowly, then bring the top foot down.",
	"Breathe out as you rise and count each rep; one leg, press Switch sides, then the other.",
}, {
	"Push through the heel of the top foot, not off the toes of the bottom foot.",
	"Stand fully tall at the top before you step down.",
	"Keep the top knee in line with your middle toes as you drive up.",
})
move("lateral_step_ups", "Lateral step-ups", "A step or box that cannot slide, side on.",
	"Step up sideways, stand tall, step down slowly. Count out loud, breathe out up.",
	"Stop when your knee caves in. Count the weaker side.")
guide("lateral_step_ups", {
	"Push the step against a wall so it cannot slide, and stand side on, right next to it.",
	"Place the foot nearest the step on top, whole foot, toes pointing forward.",
	"Drive through that foot to stand tall on the step, bringing the other foot up beside it.",
	"Lower the outside foot back to the floor slowly, then step the top foot down.",
	"Breathe out as you rise and count each rep; one side, press Switch sides, then the other.",
}, {
	"Keep your hips level and facing forward the whole time.",
	"Push through the heel of the top foot, the bottom foot only balances.",
	"Lower the outside foot so it touches down softly.",
})
move("rfe_split_squat", "Rear-foot-elevated split squat", "Rear foot on a step that cannot slide.",
	"Drop the back knee, front shin upright. Count out loud, breathe out as you rise.",
	"Stop when you wobble or your knee caves in. Count the weaker side.")
guide("rfe_split_squat", {
	"Push the step against a wall, then stand a long stride in front of it, facing away.",
	"Rest the top of your back foot on the step, laces down, front foot flat on the floor.",
	"Keep your hips square and your chest tall with a slight forward lean.",
	"Lower the back knee straight down, then rise.",
	"Breathe out as you rise and count each rep; one leg, press Switch sides, then the other.",
}, {
	"Push through the front heel, the back foot only balances.",
	"Shuffle the front foot forward if your front heel lifts at the bottom.",
	"Keep the front knee in line with your middle toes.",
})
move("step_over", "Step-over shuffle", "A low step that cannot slide.",
	"Step up and over to the other side and back, quickly. Breathe in rhythm.")
guide("step_over", {
	"Place the step on a floor that does not slip, with clear space on both sides.",
	"Stand side on next to the step, arms bent as if you were running.",
	"Step up with the near foot, bring the other up, then step down the far side.",
	"Come back the same way, with the foot nearest the step always leading.",
	"Breathe in rhythm, at a hard pace, too fast to talk.",
}, {
	"Place your whole foot on the step each time, not just your toes.",
	"Swing your arms in time with your feet to keep the rhythm.",
	"Keep your chest up and your eyes on the far edge of the step.",
})

-- Equipment: ab wheel, rings, vest, ball, roller, backpack
move("rollout_short", "Short rollout", "Knees on a folded towel, the wheel under your shoulders.",
	"Roll out a short way, ribs down, roll back. Count out loud, breathe out as you return.",
	"Stop when your lower back sags.")
guide("rollout_short", {
	"Kneel on the towel, knees hip-width apart, and grip both handles with straight arms.",
	"Start with the wheel under your shoulders, tail tucked under and ribs pulled down.",
	"Roll forward until the wheel is just past your head, hips moving with you.",
	"Breathe in as you roll out, then breathe out as you pull the wheel back.",
	"One rep is out and back to the wheel under your shoulders, so count each return.",
}, {
	"Press the handles down into the floor the whole time, as if pushing them through it.",
	"Keep your chin tucked and look at the floor just behind the wheel.",
	"Pull back with your belly, not by sitting your hips back first.",
})
move("rollout_wall", "Rollout to the wall", "Knees on a folded towel, a wall a short roll away.",
	"Roll out until the wheel touches the wall, roll back. Count out loud, breathe out back.",
	"Stop when your lower back sags.")
guide("rollout_wall", {
	"Kneel facing the wall and test one roll, so the wall stops the wheel where you control it.",
	"Start with the wheel under your shoulders, tail tucked under and ribs pulled down.",
	"Roll out until the wheel touches the wall, then pull straight back.",
	"Breathe in as you roll out and breathe out as you pull the wheel back to your knees.",
	"One rep is out to the wall and back under your shoulders, so count each return.",
}, {
	"Let the wheel touch the wall softly and never bounce off it to start the return.",
	"Keep your tail tucked most of all at the wall, where the lower back most wants to sag.",
	"Press the handles down into the floor the whole way out and back.",
})
move("rollout_hold", "Rollout hold", "Knees on a folded towel.",
	"Roll out halfway and hold, ribs down. Count out loud.", "Stop when your lower back sags.")
guide("rollout_hold", {
	"Kneel on the towel, knees hip-width apart, both hands on the handles with straight arms.",
	"Roll out halfway, until your arms reach out in front of your head, and stop there.",
	"Hold still with your tail tucked, ribs pulled down and your back flat.",
	"Breathe slowly and steadily the whole time and keep counting out loud.",
	"The clock times the hold, and when it ends, roll the wheel back under your shoulders.",
}, {
	"Press the handles into the floor so your upper back stays wide and firm.",
	"Squeeze your seat so your hips stay in line and your lower back stays flat.",
	"Look at the floor just behind the wheel to keep your neck long.",
})
move("rollout_full", "Full rollout", "Knees on a folded towel, clear space in front.",
	"Roll out as far as you can with a flat back, roll back. Count out loud, breathe out back.",
	"Stop when your lower back sags.")
guide("rollout_full", {
	"Kneel so the floor ahead is clear for your whole body length.",
	"Start with the wheel under your shoulders, tail tucked under and ribs pulled down.",
	"Roll out slowly until your body is nearly flat, one line from knees to head.",
	"Breathe in on the way out, then breathe out as you pull back a little faster.",
	"One rep is a full roll out and back to the start, so count each return.",
}, {
	"Squeeze your seat as you roll out to keep your tail tucked the whole way.",
	"Let your hips travel forward with your shoulders so your body never folds in the middle.",
	"Press the handles down hard at the far point to start the pull back.",
})
move("ring_row", "Ring row", "Rings or straps fixed firmly at waist height.",
	"Lean back, body straight, pull your chest to your hands. Count out loud, breathe out.",
	"Stop when your hips sag.")
guide("ring_row", {
	"Shut and lock any door the straps hang from, then tug both handles hard to test them.",
	"Walk your feet forward, heels down, and lean back until your arms are straight.",
	"Pull your chest up to your hands, then lower until your arms are straight.",
	"Breathe out as you pull up and breathe in as you lower.",
	"One rep is from straight arms to chest at your hands and back, so count at the top.",
}, {
	"Squeeze your shoulder blades together at the top before you lower.",
	"Keep your seat tight so your body moves as one straight board from heels to head.",
	"Keep your elbows close to your sides and let your palms turn to face each other.",
})
move("ring_y_raise", "Ring Y-raise", "Rings or straps fixed firmly at chest height.",
	"Lean back, arms straight, raise them into a Y. Count out loud, breathe out as you lift.",
	"Stop when your back arches or your arms bend.")
guide("ring_y_raise", {
	"Face the anchor, a handle in each hand, and tug the straps hard to check they hold.",
	"Walk your feet forward and lean back with straight arms in front, body in one line.",
	"Sweep your arms up and out into a Y beside your ears, letting your body rise.",
	"Breathe out as you lift, then breathe in as you lower back to the lean a little slower.",
	"One rep is from arms in front up to the Y and back, so count at the top.",
}, {
	"Point your thumbs up as your arms rise so your shoulders open cleanly.",
	"Draw your shoulder blades down and back as your arms go up.",
	"Keep your seat and belly tight so your body moves as one piece.",
})
move("ring_press", "Ring chest press", "Rings or straps fixed firmly, hands at chest height.",
	"Lean in, lower your chest between your hands, press back. Count out loud, breathe out.",
	"Stop when your hips sag or the straps wobble hard.")
guide("ring_press", {
	"Stand behind the handles, one in each hand, and tug the straps hard to check they hold.",
	"Walk your feet back and lean onto straight arms, body straight from heels to head.",
	"Lower your chest between your hands until your elbows bend to a right angle.",
	"Breathe in on the way down and breathe out as you press back to straight arms.",
	"One rep is down and back up to straight arms, so count as your arms lock out.",
}, {
	"Keep the handles steady and in line with your chest so the straps do not swing.",
	"Keep your elbows angled back toward your hips, not flared out to the sides.",
	"Squeeze your seat and belly so your hips stay in line with your shoulders.",
})
move("ring_curl", "Ring hamstring curl", "Rings low, heels in the straps, lying on the floor.",
	"Hips up, pull your heels toward your seat. Count out loud, breathe out as you pull.",
	"Stop when your hips drop or your hamstrings cramp.")
guide("ring_curl", {
	"Lie face up, heels in the straps below the anchor, arms flat at your sides.",
	"Press your heels down and lift your hips until you are straight from shoulders to heels.",
	"Pull your heels toward your seat, hips staying high, then slowly push them out.",
	"Breathe out as you pull your heels in and breathe in as your legs straighten.",
	"One rep is heels in and back out to straight legs, so count as your heels come in.",
}, {
	"Keep your hips high as your knees bend, so knees, hips and shoulders line up at the top.",
	"Press your arms and shoulders into the floor to stop the straps swinging sideways.",
	"Pull your toes up toward your shins so your heels stay seated in the straps.",
})
move("ring_fallout", "Ring fallout", "Rings or straps fixed firmly at waist height.",
	"Lean forward, arms reaching overhead, pull back. Count out loud, breathe out back.",
	"Stop when your lower back sags.")
guide("ring_fallout", {
	"Stand behind the handles and hold one in each hand at waist height, arms straight.",
	"Rise onto the balls of your feet, body straight from heels to head, tail tucked under.",
	"Lean forward slowly as your straight arms sweep up overhead, then pull back.",
	"Breathe in as you lean out and breathe out as you pull back to standing.",
	"One rep is out to arms overhead and back to the start, so count each return.",
}, {
	"Press the handles down toward your hips with straight arms to pull yourself back.",
	"Squeeze your seat and keep your ribs pulled down so your body stays in one line.",
	"Keep your head between your arms rather than lifting it to look ahead.",
})
move("ring_pistol", "Ring-assisted pistol", "Rings or straps fixed firmly at chest height.",
	"Hold the straps, sit all the way down on one leg, stand. Count out loud, breathe out up.",
	"Stop when your heel lifts or your knee caves in. Count the weaker side.")
guide("ring_pistol", {
	"Stand a short step behind the handles and hold one in each hand at chest height.",
	"Shift onto one foot and lift the other leg straight out in front, clear of the floor.",
	"Sit all the way down, thigh on calf, heel flat, then stand back up.",
	"Breathe in on the way down and breathe out as you press up to standing.",
	"One rep is down and back up: all reps on one leg, then press Switch sides.",
}, {
	"Push your standing knee out in line with your toes as you sit down.",
	"Keep your heel flat and your weight over the middle of the standing foot.",
	"Point the toes of the lifted leg up so that leg stays long and clear of the floor.",
})
move("vest_wall_sit", "Weighted wall sit", "The vest snug, back flat on the wall.",
	"Slide down a quarter, knees over ankles. Count out loud.", "Stop when you slide or your knees ache.")
guide("vest_wall_sit", {
	"Stand with your back flat on the wall, feet a short step out and hip-width apart.",
	"Slide down the wall until your knees are a little bent, a quarter of the way to sitting.",
	"Shift your feet so your knees sit right above your ankles, then hold still.",
	"Breathe slowly and evenly, counting out loud so you never hold your breath.",
	"The clock times the hold, and when it ends, slide back up the wall to stand.",
}, {
	"Keep your head, shoulders and the back of the vest against the wall the whole time.",
	"Let your arms hang loose and relax your hands and jaw.",
	"Push your whole foot into the floor, heels too, to feel the front of your thighs work.",
})
move("vest_squat", "Weighted squat", "A vest of 5-10 kg to start, snug and even, the desk within reach.",
	"Sit back, chest up, stand. Count out loud, breathe out as you stand.",
	"Stop when your heels lift or your knees cave in.")
guide("vest_squat", {
	"Stand beside the desk within reach, feet shoulder-width apart and toes out a little.",
	"Stand tall with your chest up and your weight spread over the whole foot.",
	"Sit back and down until your thighs are level, arms reaching forward.",
	"Stand back up from the bottom, breathing out as you rise.",
	"One rep is down and back up to standing tall, so count as you reach the top.",
}, {
	"Spread the floor apart with your feet to keep your knees tracking over your toes.",
	"Keep your chest up so the vest does not pull your upper back round.",
	"Keep your heels pressed down and drive up through the middle of each foot.",
})
move("vest_step_ups", "Weighted step-ups", "A vest of 5-10 kg to start, snug, a step that cannot slide.",
	"Whole foot on the step, drive up, step down. Count out loud, breathe out up.",
	"Stop when your knee caves in. Count the weaker side.")
guide("vest_step_ups", {
	"Stand facing the step, close enough that your foot lands in its middle.",
	"Place one whole foot on the step and lean your chest forward a little.",
	"Breathe out and drive up through that heel until you stand tall on top.",
	"Step down slowly with the other foot first, then bring the lead foot down.",
	"One rep is up and down: all reps with one leg leading, then press Switch sides.",
}, {
	"Push through the leg on the step, not off the toes of the foot on the floor.",
	"Let the leg on the step lower you slowly rather than dropping down.",
	"Look ahead, not down at your feet, once you know where the step is.",
})
move("vest_stairs", "Weighted stair climb", "The vest snug, a handrail within reach.",
	"Climb steadily, walk down carefully. Breathe in rhythm.")
guide("vest_stairs", {
	"Check the stairs are clear and well lit, then stand at the bottom beside the handrail.",
	"Rest one hand lightly on the rail and lean your chest a little forward.",
	"Climb at a steady beat, one foot on each step, placing your whole foot every time.",
	"At the top, turn and walk down slowly with a hand on the rail, then climb again.",
	"Keep a hard pace, too fast to talk on the way up.",
}, {
	"Push through the whole foot on the step above rather than springing off your toes.",
	"Keep your eyes a few steps ahead, not on your feet.",
	"Slide your hand along the rail as you go so it is always there to grab.",
})
move("ball_goblet_squat", "Ball squat", "The ball held at your chest.",
	"Sit down between your heels, chest up, stand. Breathe out as you stand.",
	"Stop when your back rounds or your knees cave in.")
guide("ball_goblet_squat", {
	"Stand with your feet a little wider than your hips, toes turned out a little.",
	"Hold the ball against your chest with both hands underneath it, elbows pointing down.",
	"Sit straight down between your heels until your hips reach knee height.",
	"Stand back up from the bottom, breathing out as you rise.",
	"One rep is down and back to standing tall, so count as you reach the top.",
}, {
	"Keep the ball tight to your chest so it does not pull you forward.",
	"Let your elbows travel down inside your knees at the bottom to guide your knees out.",
	"Press your heels down and stand up by pushing the floor away.",
})
move("ball_chest_pass", "Ball chest pass", "A solid wall and a ball that bounces back.",
	"Throw from your chest at the wall, catch softly. Count out loud, breathe out on the throw.",
	"Stop when your throws get sloppy.")
guide("ball_chest_pass", {
	"Stand about two big steps from the wall, feet hip-width apart and knees a little bent.",
	"Hold the ball at your chest, hands on its sides, thumbs behind and elbows down.",
	"Push it hard and straight at the wall at chest height until your arms are straight.",
	"Catch it with soft arms and let your elbows bend to bring it back to your chest.",
	"Breathe out on each throw, and count one rep each time you catch the ball.",
}, {
	"Aim for one spot on the wall so every throw comes straight back to your hands.",
	"Keep your elbows down and close at the start so the push comes from your chest.",
	"Shift your weight onto the balls of your feet as you throw to add snap.",
})
move("ball_chop", "Ball chop", "Clear arm's length around you.",
	"Ball high on one side, chop it low to the other. Count out loud, breathe out on the chop.",
	"Stop when your hips swing with your arms. Count the weaker side.")
guide("ball_chop", {
	"Stand with your feet wider than your hips, knees soft, the ball in both hands.",
	"Lift the ball high above one shoulder with long arms, turning your chest toward it.",
	"Chop it fast across your body to outside the other knee, bending your knees as you go.",
	"Breathe out hard on the chop, then breathe in as you lift it back up more slowly.",
	"One rep is high to low and back up: all reps on one side, then press Switch sides.",
}, {
	"Keep your hips and knees facing forward and let your chest do the turning.",
	"Tighten your belly before each chop so you stop the ball where you choose.",
	"Follow the ball with your eyes so your head turns with your chest.",
})
move("ball_overhead_carry", "Overhead carry", "A light ball, a clear hallway.",
	"Ball overhead, arms straight, ribs down, walk steadily. Breathe evenly.",
	"Stop when your back arches or your arms drop.")
guide("ball_overhead_carry", {
	"Clear the hallway floor and pick a turning point at each end before you start.",
	"Press the ball straight up above your head with both hands, arms fully straight.",
	"Stand tall with your ribs pulled down so your back stays flat under the ball.",
	"Walk at a steady, normal pace and turn with small steps at each end.",
	"Breathe evenly and keep walking until the clock ends.",
}, {
	"Squeeze the ball between your palms and push it up toward the ceiling.",
	"Keep your upper arms close by your ears rather than drifting forward.",
	"Take short, smooth steps so the ball stays still above your head.",
})
move("ball_slam", "Ball slam", "A slam ball on a sturdy floor, never a bouncing ball.",
	"Lift it overhead, slam it down, hinge to pick up. Count out loud, breathe out on the slam.",
	"Stop when your back rounds on the pick-up.")
guide("ball_slam", {
	"Stand with feet shoulder-width apart, the floor just in front of your feet clear.",
	"Bend your hips and knees and pick the ball up with both hands, back flat.",
	"Lift it overhead fast, rising up onto your toes as your arms go long.",
	"Slam it hard onto the floor just ahead of your feet, bending hips and knees as it drops.",
	"Breathe out hard on the slam, and count one rep each time the ball hits the floor.",
}, {
	"Pull the ball down with your belly and back, not just your arms.",
	"Look at one spot on the floor just ahead of your toes and aim every slam there.",
	"Keep a steady rhythm, taking a quick breath in as you lift before each slam.",
})
move("roller_thoracic", "Roller upper-back extension", "Roller across your upper back, hands behind your head.",
	"Lean back over the roller, move it an inch at a time. Breathe slowly.",
	"Ease off if your neck or lower back pinches.")
guide("roller_thoracic", {
	"Sit on the floor, knees bent, feet flat, the roller behind you under your shoulder blades.",
	"Lace your fingers behind your head to hold its weight and keep your neck relaxed.",
	"Keep your hips on the floor and lean back over the roller for a slow breath, then sit up.",
	"Lift your hips a little, shift the roller an inch along your upper back, then lean back.",
	"Breathe slowly and move at an easy pace for the time shown.",
}, {
	"Stay on the upper back between your shoulder blades, never down on the lower back.",
	"Keep your ribs down and let your upper back curve over the roller, not your neck.",
	"Breathe out as you lean back to let your upper back soften over the roller.",
})
move("roller_calf_quad", "Roll calves and thighs", "The roller on the floor.",
	"Roll slowly along your calves, then the front of your thighs. Breathe slowly.",
	"Ease off anything sharp.")
guide("roller_calf_quad", {
	"Sit on the floor with the roller under your calves and your hands behind you.",
	"Lift your hips and roll slowly from just above your ankles to just below your knees.",
	"Turn onto your front and rest on your forearms with the roller under your thighs.",
	"Roll slowly from just above your knees to the top of your thighs and back.",
	"Take about 2 to 3 s each way, breathe slowly, and keep an easy pace for the time shown.",
}, {
	"Hold your body in one straight line from shoulders to knees as you roll the thighs.",
	"Turn your toes in or out a little to roll the inner and outer sides.",
	"Pause for a slow breath on a tight spot instead of rolling fast over it.",
})
move("roller_glute_lat", "Roll hips and back", "The roller on the floor.",
	"Roll slowly along your seat, then the side of your back. Breathe slowly.",
	"Ease off anything sharp.")
guide("roller_glute_lat", {
	"Sit on the roller with your knees bent, feet flat and hands on the floor behind you.",
	"Roll slowly back and forth over your seat, about 2 s each way.",
	"Turn onto your side with the roller under the side of your back, bottom arm long overhead.",
	"Roll slowly from your lower ribs up to just below your armpit and back.",
	"Breathe slowly and move at an easy pace for the time shown.",
}, {
	"Lean onto one hip, then the other, to reach each side of your seat.",
	"Keep the bottom arm reaching long overhead to open the side of your back.",
	"Keep your belly firm on your side so your body rolls as one piece.",
})
move("roller_dead_bug", "Roller dead bug", "Lie on the roller along your spine, feet on the floor.",
	"Lower one arm and the other leg, stay balanced. Count out loud, breathe out as you lower.",
	"Stop when you roll off or your back arches. Alternate sides; each lower counts one.")
guide("roller_dead_bug", {
	"Sit at one end of the roller and lie back so your head and hips both rest on it.",
	"Bend your knees, feet flat and hip-width apart, and reach both arms straight up.",
	"Lower one arm overhead and straighten the other leg over the floor.",
	"Breathe out as you lower, then bring both back and switch sides.",
	"Count one rep each time an arm and the other leg go down and come back.",
}, {
	"Press your lower back gently into the roller the whole time so your ribs stay down.",
	"Press the planted foot into the floor to hold you steady on the roller.",
	"Keep the straight leg hovering off the floor, never resting it down.",
})
move("backpack_squat", "Backpack squat", "A backpack of 8-12 kg on your back, straps tight.",
	"Sit back, chest up, stand. Count out loud, breathe out as you stand.",
	"Stop when your heels lift or your knees cave in.")
guide("backpack_squat", {
	"Pull both straps tight so the pack sits high and close on your back.",
	"Stand with feet shoulder-width apart, toes turned out a little, chest up.",
	"Sit back and down until your thighs pass level, arms reaching forward.",
	"Drive up from the bottom, breathing out as you stand.",
	"One rep is down and back up to standing tall, so count as you reach the top.",
}, {
	"Keep your back flat and chest up so the pack never pulls your shoulders round.",
	"Reach your arms forward as you sit down to balance the pack on your back.",
	"Load it to 8-12 kg: 1-L water bottles weigh 1 kg each, a hardback about 1 kg.",
})
move("backpack_squat_drives", "Backpack squat drives", "A packed backpack, 3-8 kg, on both straps.",
	"Squat, then drive up onto your toes, arms overhead. Breathe out as you rise.")
guide("backpack_squat_drives", {
	"Fill the backpack with books or bottles, 3 to 8 kg, and wear it on both straps, snug.",
	"Stand with your feet shoulder-width apart, arms at your sides.",
	"Sit back and down into a squat, chest up, arms reaching forward.",
	"Drive up fast onto your toes and reach both arms straight overhead.",
	"Drop your heels and sit straight into the next squat, too fast to talk.",
}, {
	"Let the drive come from your legs; the arms only follow.",
	"Take a lighter pack if your squats get shallow before the time is up.",
})
implement("backpack_squat_drives", "vest", "A weighted vest of 3-8 kg, snug and even.", {
	"Put on the vest, 3 to 8 kg, and close it snug so it does not bounce.",
	"Stand with your feet shoulder-width apart, arms at your sides.",
	"Sit back and down into a squat, chest up, arms reaching forward.",
	"Drive up fast onto your toes and reach both arms straight overhead.",
	"Drop your heels and sit straight into the next squat, too fast to talk.",
})
implement("backpack_squat_drives", "backpack", "A packed backpack, 3-8 kg, on both straps.")

-- Slice 21: the stretching and yoga families' new poses, and each family anchor's mark (the standard its Ready view
-- names to aim for, ns.Marks), in the families' order (docs/plan.md slice 21)
-- The front split and the hip flexors
move("quad_stretch", "Standing quad stretch", "Stand side-on to the desk, a hand on it.",
	"Heel to your buttock, knees side by side, stand tall. Breathe slowly.",
	"Ease off at a strong pull or if your knee hurts; never bounce.")
guide("quad_stretch", {
	"Stand side-on to the desk with the near hand on it and your feet together.",
	"Bend the outside knee and hold that ankle or foot with the hand on the same side.",
	"Draw the heel to your buttock, knees side by side, and stand tall.",
	"Tuck your tailbone a little and breathe slowly into the front of the thigh.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Keep the bent knee pointing at the floor, not out to the side.",
	"Loop a towel round the ankle if your hand cannot reach it.",
	"Squeeze your seat to feel more stretch without pulling harder.",
})
L["move.quad_stretch.mark"] = "Heel on the buttock, held by the hand or a towel, knees side by side, standing tall."
move("high_lunge", "High lunge", "A long stride of clear floor beside the desk.",
	"Front knee over ankle, back knee straight, arms up. Breathe slowly.",
	"Ease off at a strong pull or if your front knee hurts; never bounce.")
guide("high_lunge", {
	"Stand tall beside the desk, then step one foot back a long stride onto its toes.",
	"Bend the front knee over the ankle and lower until that thigh is close to level.",
	"Straighten the back knee and press the back heel away from you.",
	"Reach both arms up beside your ears, hands off the knee, and breathe slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Keep your hips square to the front, the back hip drawn forward.",
	"Rest your fingertips on the desk if you wobble, then lift them again.",
	"Tuck your tailbone a little to feel the stretch at the front of the back hip.",
})
L["move.high_lunge.mark"] = "Front thigh level, back knee straight, arms up, no hand on the knee or the desk."
move("pyramid", "Pyramid", "Clear floor, feet a leg-length apart, two books by your front foot.",
	"Hips square, back flat, fold over the front leg. Breathe slowly.",
	"Ease off at a strong pull or a tug high under the buttock; never bounce.")
guide("pyramid", {
	"Step one foot back a leg-length, back toes turned out a little, hips facing forward.",
	"Straighten both knees and hinge forward from the hips with a long, flat back.",
	"Lower your fingertips to the floor or the books beside the front foot.",
	"Breathe slowly, then rise slowly with a flat back at the end of the hold.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Draw the front hip back so both hips face the front.",
	"Keep your hands on the books if the floor is too far, both knees straight.",
	"Lead with your chest, not your head, so your back stays long.",
})
L["move.pyramid.mark"] = "Fingertips on the floor beside the front foot, both knees straight, hips square."
move("couch_stretch", "Couch stretch", "A bed or sofa behind you, a folded towel at its base for your knee.",
	"Back shin up the sofa, chest tall, squeeze your seat. Breathe slowly.",
	"Ease off if your knee hurts or the front of your thigh cramps; never bounce.")
guide("couch_stretch", {
	"Kneel with your back to the bed or sofa, one knee on the towel close to its base.",
	"Slide that shin up the front of the bed or sofa, the top of the foot resting on it.",
	"Step the other foot forward into a lunge, knee over the ankle, hands on that knee.",
	"Lift your chest until your body is upright, hands on your hips, and squeeze your seat.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Stay leaning forward, hands on the front knee, until upright feels easy.",
	"Move the knee a little away from the bed or sofa if the stretch is too strong.",
	"Tuck your tailbone and keep your ribs down so your low back does not arch.",
})
L["move.couch_stretch.mark"] = "Back shin up the bed or sofa, body upright, hands off the front knee."
move("pyramid_desk", "Pyramid at the desk", "Stand facing your desk, about a long step away.",
	"Front leg straight, hips square, chest forward over the front thigh. Breathe slowly.",
	"Ease off at a strong pull or any tingling down the leg; never bounce.")
guide("pyramid_desk", {
	"Stand facing your desk, about a long step away, hands on its edge.",
	"Step one foot back, both feet flat, the back toes turned out a little.",
	"Square your hips to the desk and straighten your front leg.",
	"Lean your chest forward over the front thigh with a long back, breathing slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Shorten your stance if your back heel lifts.",
	"Keep your head up and your back long; the fold comes from the hips.",
	"Slide your hands closer to the desk's edge as the fold gets deeper.",
})
L["move.pyramid_desk.mark"] = "Front leg straight, hips square, back flat and level with the floor."
move("split_prep", "Front split prep", "2 m of clear floor, two books for your hands.",
	"Front heel out, both knees straight, hands down. Breathe slowly.",
	"Ease off at a strong pull or a tug high under the buttock; never bounce.")
guide("split_prep", {
	"Kneel in a low lunge, fingertips wide on the floor or books, just ahead of the front foot.",
	"Slide the front heel forward, toes up, until that knee is straight.",
	"Slide the back leg back and straighten it, the back kneecap facing the floor.",
	"Stop about 30 cm from the floor and breathe slowly, then bend the front knee to come out.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Keep your hips square: back toes on the floor, front toes pointing up.",
	"Take your weight in your hands so you can rise at any moment.",
	"30 cm is about the long side of a sheet of printer paper.",
})
L["move.split_prep.mark"] = "Hips within 30 cm of the floor, both knees straight, hands on the floor or books."
move("split_low", "Low front split", "2 m of clear floor, two books for your hands.",
	"Slide lower, knees straight, hips square, hands down. Breathe slowly.",
	"Ease off at a strong pull or a tug high under the buttock; never bounce.")
guide("split_low", {
	"Kneel in a low lunge, hands wide on the floor or books, just ahead of the front foot.",
	"Slide the front heel forward, toes up, and the back knee back, both legs straight.",
	"Lower slowly, your hands taking your weight, the back kneecap facing the floor.",
	"Stop about 15 cm from the floor and breathe slowly, then bend the front knee to come out.",
	"The clock times each side, then counts you in to switch.",
}, {
	"15 cm is about the width of one and a half fists.",
	"Slide down slowly and never drop or bounce into the split.",
	"Keep your hips square: back toes on the floor, front toes pointing up.",
})
L["move.split_low.mark"] = "Hips within 15 cm of the floor, both knees straight, hands on the floor or books."
move("front_split", "Front split", "2 m of clear floor, two books by your hips.",
	"Both thighs down, hips square, sit tall, hands down. Breathe slowly.",
	"Ease off at a strong pull or a tug high under the buttock; never bounce.")
guide("front_split", {
	"Start in a low lunge, hands wide on the floor or books, just ahead of the front foot.",
	"Slide the front heel forward and the back knee back, both knees straight.",
	"Lower until both thighs rest on the floor, your hands taking your weight.",
	"Sit tall with your hips square and breathe slowly, then bend the front knee to come out.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Rest your hips on the books if they stop short of the floor.",
	"Keep the back kneecap facing the floor and the front toes pointing up.",
	"Slide down slowly and never drop or bounce into the split.",
})
L["move.front_split.mark"] = "Both thighs flat on the floor, no books under the hips, hips square, knees straight."
-- More goal poses on every step (slice 47): standing, lying on a bed, kneeling and seated forms, press and sink
-- and lift and hold.
move("hip_stretch_desk", "Standing hip stretch", "Stand side-on to the desk, a long step of clear floor behind you.",
	"Back knee straight, tuck your tailbone, squeeze your seat. Breathe slowly.",
	"Ease off at a strong pull or if your hip or low back hurts; never bounce.")
guide("hip_stretch_desk", {
	"Stand side-on to the desk, the near hand on it.",
	"Step the outside foot back a long step onto its toes and straighten that knee.",
	"Bend the front knee a little, hands on your hips, body upright.",
	"Tuck your tailbone under and squeeze your seat, then breathe slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Feel the stretch at the front of the back hip, never in your low back.",
	"Keep your ribs down so your low back does not arch.",
	"Rest a hand on the desk again if you wobble.",
})
L["move.hip_stretch_desk.mark"] = "Feet a long step apart, back knee straight, seat squeezed, standing tall and steady."
move("bed_hip_stretch", "Lying hip stretch on the bed", "Sit on the very end of your bed.",
	"Hug one knee in, let the other leg hang off the end. Breathe slowly.",
	"Ease off at a strong pull or if your hip or knee hurts; never bounce.")
guide("bed_hip_stretch", {
	"Sit on the very end of your bed, then lie back along it.",
	"Hug both knees to your chest so your low back rests flat.",
	"Keep one knee hugged and lower the other leg off the end of the bed.",
	"Let that thigh hang, its knee bent, and breathe slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Keep the hugged knee close so your low back stays flat on the bed.",
	"Let the leg's weight do the work; never push it down.",
	"Shuffle up the bed a little if your foot touches the floor.",
})
L["move.bed_hip_stretch.mark"] = "Back flat on the bed, other knee hugged, hanging thigh level or lower, leg relaxed."
move("quad_stretch_press", "Quad stretch, press and sink", "Stand side-on to the desk, a hand on it.",
	"Press your foot gently into your hand for 5, then breathe out and ease in.",
	"Ease off at a strong pull or if your knee hurts; never bounce.")
guide("quad_stretch_press", {
	"Stand side-on to the desk, the near hand on it.",
	"Hold the outside foot behind you, heel near your buttock, knees side by side.",
	"Press the foot gently back into your hand at half effort for 5 counts.",
	"Relax, breathe out and ease the heel a little closer. Do three rounds.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Half effort is gentle: the leg hardly moves while you press.",
	"Keep breathing as you press; never hold your breath.",
	"Loop a towel round the ankle if your hand cannot reach it.",
})
L["move.quad_stretch_press.mark"] = "Heel on the buttock after the last round, knees side by side, standing tall."
move("bed_foot_lunge", "Back foot on the bed lunge", "Stand a long step in front of your bed, facing away from it.",
	"Back foot on the bed, sink straight down, body tall. Breathe slowly.",
	"Ease off at a strong pull or if a knee hurts; never bounce.")
guide("bed_foot_lunge", {
	"Stand facing away from your bed, a long step in front of it.",
	"Rest the top of one foot on the edge of the bed behind you.",
	"Bend the front knee and sink straight down, the front shin upright.",
	"Stay tall, hands on your hips, until the front thigh is level. Breathe slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Move the front foot further out if its knee goes past your toes.",
	"Squeeze the back leg's seat to feel more stretch at the front of the hip.",
	"Rest a hand on a wall if you wobble.",
})
L["move.bed_foot_lunge.mark"] = "Back foot on the bed, front thigh level, front shin upright, body tall, hands off."
move("half_split_books", "Half split on books", "Kneel on a folded towel, a pile of books by each hip.",
	"Front leg straight, toes up, back long, hands on the books. Breathe slowly.",
	"Ease off at a strong pull or a tug high under the buttock; never bounce.")
guide("half_split_books", {
	"Kneel on the towel with one foot forward, a pile of books either side of your hips.",
	"Move your hips back until they sit over the back knee.",
	"Straighten the front leg, heel down and toes up.",
	"Lean forward with a long back, hands on the books, and breathe slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Stack the books higher if your back rounds.",
	"Lead with your chest so the fold comes from the hips.",
	"Keep the back hip right above the back knee.",
})
L["move.half_split_books.mark"] = "Back knee under the hip, front knee straight, back long, hands on books by the hips."
move("lunge_foot_hold", "Low lunge, foot hold", "Kneel on a folded towel in a long low lunge.",
	"Reach back, draw the back heel in, body upright. Breathe slowly.",
	"Ease off at a strong pull or if your knee hurts; never bounce.")
guide("lunge_foot_hold", {
	"Kneel in a low lunge, the back knee on the towel, the front foot a long step ahead.",
	"Bend the back knee and reach back for that foot with the hand on the same side.",
	"Draw the heel toward your seat, the other hand on the front thigh.",
	"Keep your body upright, hips forward, and breathe slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Loop a towel round the foot if you cannot reach it.",
	"Move the back knee further back if the kneecap presses hard.",
	"Squeeze your seat so the stretch stays at the front of the hip.",
})
L["move.lunge_foot_hold.mark"] = "Back knee on the towel, back heel a hand from the seat, body upright and steady."
move("pyramid_desk_press", "Pyramid at the desk, press and sink", "Stand facing your desk, about a long step away.",
	"Press the front heel gently down for 5, then breathe out and fold a little deeper.",
	"Ease off at a strong pull or any tingling down the leg; never bounce.")
guide("pyramid_desk_press", {
	"Stand in the pyramid at the desk, hands on its edge, front leg straight.",
	"Press the front heel gently into the floor at half effort for 5 counts.",
	"Relax, breathe out and lean your chest a little further over the front thigh.",
	"Do three rounds, the hips square and the back long.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Half effort is gentle: the heel presses, the leg does not move.",
	"Keep breathing as you press; never hold your breath.",
	"Shorten your stance if your back heel lifts.",
})
L["move.pyramid_desk_press.mark"] = "Front leg straight, hips square, back level with the floor after the last round."
move("lizard_books", "Lizard on books", "Clear floor, a pile of books inside your front foot.",
	"Back knee straight, hips low, forearms on the books. Breathe slowly.",
	"Ease off at a strong pull or if your hip or knee hurts; never bounce.")
guide("lizard_books", {
	"Start in a lunge, the front foot a long step ahead and turned out a little.",
	"Put the books inside the front foot and lower your forearms onto them.",
	"Let the front knee open out beside your shoulder.",
	"Straighten the back knee, toes tucked, hips low, and breathe slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Use your hands on the books, arms straight, if your forearms do not reach.",
	"Keep the front heel flat on the floor.",
	"Rest the back knee down to come out.",
})
L["move.lizard_books.mark"] = "Forearms on books inside the front foot, back knee straight and off the floor, hips low."
move("half_split_lift", "Half split, heel lift", "Kneel on a folded towel, two piles of books in front of you.",
	"Squeeze the front thigh, lift the heel a little, hold. Breathe slowly.",
	"Ease off at a strong pull or if the front thigh cramps; never bounce.")
guide("half_split_lift", {
	"Set up the half split on books, front leg straight, toes up.",
	"Squeeze the front thigh hard so the knee stays straight.",
	"Lift the front heel a finger off the floor and hold for a few counts.",
	"Lower it slowly, then lift again, breathing all the time.",
	"The clock times each side, then counts you in to switch.",
}, {
	"A tiny lift is plenty; keep the knee straight and the hips still.",
	"Keep breathing as you lift; never hold your breath.",
	"Stack the books higher if your back rounds.",
})
L["move.half_split_lift.mark"] = "Front heel a finger off the floor, knee straight, toes up, back long, hands on books."
move("lizard_forearms", "Lizard on the forearms", "Clear floor and a mat or towel for your forearms.",
	"Back knee straight, hips low, forearms on the floor. Breathe slowly.",
	"Ease off at a strong pull or if your hip or knee hurts; never bounce.")
guide("lizard_forearms", {
	"Start in the lizard on books, the front foot turned out a little.",
	"Take the books away and lower your forearms to the floor inside the front foot.",
	"Keep the back knee straight and off the floor, toes tucked.",
	"Let your hips sink low and breathe slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Go back to the books if your back knee bends to reach the floor.",
	"Keep the front heel flat on the floor.",
	"Rest the back knee down to come out.",
})
L["move.lizard_forearms.mark"] = "Forearms flat on the floor inside the front foot, back knee straight and lifted."
move("couch_arms_up", "Couch stretch, arms up", "A bed or sofa behind you, a folded towel at its base for your knee.",
	"Back shin up the sofa, body upright, arms up, ribs down. Breathe slowly.",
	"Ease off if your knee hurts or the front of your thigh cramps; never bounce.")
guide("couch_arms_up", {
	"Set up the couch stretch, back shin up the bed or sofa, front foot forward.",
	"Lift your chest until your body is upright and squeeze your seat.",
	"Reach both arms straight up beside your ears.",
	"Keep your ribs down so your low back stays long, and breathe slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Lower your hands to your hips if your low back arches.",
	"Move the knee a little away from the bed or sofa if the stretch is too strong.",
	"Tuck your tailbone to feel more stretch without leaning back.",
})
L["move.couch_arms_up.mark"] = "Back shin up the bed or sofa, body upright, arms straight up by your ears."
move("split_prep_press", "Front split prep, press and sink", "2 m of clear floor, two books for your hands.",
	"Press both legs gently down for 5, then breathe out and slide a little lower.",
	"Ease off at a strong pull or a tug high under the buttock; never bounce.")
guide("split_prep_press", {
	"Slide into the front split prep, both knees straight, hands on the floor or books.",
	"Press the front heel and back knee gently down at half effort for 5 counts.",
	"Relax, breathe out and let your hips slide a little lower.",
	"Do three rounds, your hands taking your weight.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Half effort is gentle: your legs press, they do not move.",
	"Keep breathing as you press; never hold your breath.",
	"Keep your hips square: back toes on the floor, front toes pointing up.",
})
L["move.split_prep_press.mark"] = "Hips within 30 cm of the floor, both knees straight, hips square, hands on books."
move("leg_lift_desk", "Standing leg lift at the desk", "Stand side-on to the desk, a hand on it.",
	"Lift the straight leg forward to desk height and hold. Breathe slowly.",
	"Ease off if the front of your hip cramps or pinches; never swing the leg.")
guide("leg_lift_desk", {
	"Stand tall side-on to the desk, the near hand on it, the other on your hip.",
	"Lift the outside leg straight forward, knee straight and toes up.",
	"Raise your toes to the height of the desk and hold them there for a few counts.",
	"Lower it slowly, then lift again, breathing all the time.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Lift with the front of the thigh; never swing the leg up.",
	"Keep the standing knee straight and your body tall.",
	"A lower lift with a straight knee beats a high one with a bent knee.",
})
L["move.leg_lift_desk.mark"] = "Toes up and level with the desk top, both knees straight, standing tall and steady."
move("split_low_tall", "Low front split, upright", "2 m of clear floor, two piles of books by your hips.",
	"Hips low, knees straight, body upright, hands on the books. Breathe slowly.",
	"Ease off at a strong pull or a tug high under the buttock; never bounce.")
guide("split_low_tall", {
	"Slide into the low front split, hands on the floor ahead of the front foot.",
	"Walk your hands back onto the books beside your hips.",
	"Lift your body upright, hips square, both knees straight.",
	"Keep your hips about 15 cm from the floor and breathe slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Stack the books higher if you cannot sit upright.",
	"Squeeze the back leg's seat to feel the front of that hip.",
	"Slide down slowly and never drop or bounce into the split.",
})
L["move.split_low_tall.mark"] = "Hips within 15 cm of the floor, body upright, both knees straight, hands on books."
move("split_low_press", "Low front split, press and sink", "2 m of clear floor, two books for your hands.",
	"Press both legs gently down for 5, then breathe out and slide a little lower.",
	"Ease off at a strong pull or a tug high under the buttock; never bounce.")
guide("split_low_press", {
	"Slide into the low front split, both knees straight, hands on the floor or books.",
	"Press the front heel and back knee gently down at half effort for 5 counts.",
	"Relax, breathe out and let your hips slide a little lower.",
	"Do three rounds, your hands taking your weight.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Half effort is gentle: your legs press, they do not move.",
	"Keep breathing as you press; never hold your breath.",
	"15 cm is about the width of one and a half fists.",
})
L["move.split_low_press.mark"] = "Hips within 15 cm of the floor, both knees straight, hips square, hands on books."
move("bed_leg_pull", "Lying leg pull on the bed", "Lie on your back along your bed.",
	"Hold behind the raised leg, knee straight, other leg flat. Breathe slowly.",
	"Ease off at a strong pull or any tingling down the leg; never bounce.")
guide("bed_leg_pull", {
	"Lie on your back along your bed, both legs straight.",
	"Bend one knee to your chest and hold behind the thigh with both hands.",
	"Straighten that leg up toward the ceiling.",
	"Draw it gently past straight up, the other leg flat, and breathe slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Loop a towel round the foot if you cannot keep the knee straight.",
	"Keep your head and both hips resting on the bed.",
	"Press the back of the other knee into the bed.",
})
L["move.bed_leg_pull.mark"] = "Raised foot over your chest, both knees straight, the other leg flat on the bed."
move("split_arms_up", "Front split, arms up", "2 m of clear floor, a mat or towel.",
	"Both thighs down, hips square, body upright, arms up. Breathe slowly.",
	"Ease off at a strong pull or a tug high under the buttock; never bounce.")
guide("split_arms_up", {
	"Slide into the front split, hands on the floor beside your hips.",
	"Sit tall with your hips square and both knees straight.",
	"Lift your hands off the floor and reach both arms straight up.",
	"Keep your ribs down and breathe slowly, then put your hands down to come out.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Keep your hands down until sitting tall in the split feels easy.",
	"Put a hand down at once if you tip to one side.",
	"Keep the back kneecap facing the floor and the front toes pointing up.",
})
L["move.split_arms_up.mark"] = "Both thighs flat on the floor, hips square, body upright, both arms straight up."
move("split_fold", "Front split fold", "2 m of clear floor, a mat or towel.",
	"Both thighs down, hips square, fold forward with a long back. Breathe slowly.",
	"Ease off at a strong pull or a tug high under the buttock; never bounce.")
guide("split_fold", {
	"Slide into the front split and sit tall, hips square, knees straight.",
	"Lean forward from the hips with a long back over the front leg.",
	"Rest your hands on the floor either side of the front leg.",
	"Let your chest sink low over the front thigh and breathe slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Lead with your chest, not your head, so your back stays long.",
	"Keep both hips down on the floor as you fold.",
	"Sit up slowly with a long back to come out.",
})
L["move.split_fold.mark"] = "Both thighs flat on the floor, hips square, back long, chest low over the front thigh."
move("leg_lift_hip", "Standing leg lift to hip height", "Stand side-on to the desk, a hand on it.",
	"Lift the straight leg forward to hip height and hold. Breathe slowly.",
	"Ease off if the front of your hip cramps or pinches; never swing the leg.")
guide("leg_lift_hip", {
	"Stand tall side-on to the desk, the near hand on it, the other on your hip.",
	"Lift the outside leg straight forward, knee straight and toes up.",
	"Raise the heel to the height of your hip and hold it there for a few counts.",
	"Lower it slowly, then lift again, breathing all the time.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Lift with the front of the thigh; never swing the leg up.",
	"Keep the standing knee straight and your body tall, not leaning back.",
	"Go back to desk height if the knee bends to get higher.",
})
L["move.leg_lift_hip.mark"] = "Heel held at hip height, both knees straight, toes up, standing tall and steady."

-- The pancake and the middle split
move("butterfly", "Butterfly on the bed", "Sit on your bed or the floor, your back on the wall if it helps.",
	"Soles together, heels in, let your knees fall open; never push them. Breathe slowly.",
	"Ease off at a strong pull or if the inside of a knee hurts; never bounce.")
guide("butterfly", {
	"Sit tall on your bed with your knees bent and the soles of your feet together.",
	"Draw your heels in, about a forearm from your body, fingertips on the bed behind you.",
	"Let both knees fall open towards the bed; never press them down.",
	"Sit tall over your sit bones and breathe slowly for the time shown.",
}, {
	"Sit on a folded towel if your back rounds when you sit tall.",
	"Lift your chest and let your fingertips take a little weight.",
	"Let your knees sink a little more with each breath out.",
})
L["move.butterfly.mark"] =
	"Knees two fists off the bed, soles together, heels a forearm from your body, sitting tall."
move("lying_butterfly", "Lying butterfly", "Lie on your back on your bed, a pillow under your head.",
	"Soles together, let your knees fall open; never push them. Breathe slowly.",
	"Ease off at a strong pull or if the inside of a knee hurts.")
guide("lying_butterfly", {
	"Lie on your back on the bed with your knees bent and your feet flat.",
	"Bring the soles of your feet together and slide your heels towards your seat.",
	"Let both knees fall open to the sides, your arms resting by your sides.",
	"Keep your low back resting on the bed and breathe slowly for the time shown.",
	"To finish, bring your knees together with your hands and stretch your legs out.",
}, {
	"Put a pillow under each knee if the stretch is too strong.",
	"Let the bed take your weight; gravity does the stretching, not you.",
})
L["move.lying_butterfly.mark"] =
	"Knees two fists from the bed, soles together, heels a hand from your seat, back resting."
move("wide_chair_stretch", "Wide-leg chair stretch", "Sit near the front of a firm chair that does not roll.",
	"Legs wide and straight, toes up, lean forward with a flat back. Breathe slowly.",
	"Ease off at a strong pull inside the thighs or behind the knees.")
guide("wide_chair_stretch", {
	"Sit near the front edge of your chair and stretch both legs out wide.",
	"Rest your heels on the floor, knees straight and toes pointing up.",
	"Sit tall, then lean forward from your hips with a flat back.",
	"Slide your hands down to your knees and breathe slowly for the time shown.",
}, {
	"Bring your feet a little closer if your knees have to bend.",
	"Lead with your chest so your back stays flat.",
	"Widen your feet a little more each week.",
})
L["move.wide_chair_stretch.mark"] = "Hands on your knees, heels four foot-lengths apart, knees straight, back flat."
move("horse_stance", "Horse stance", "Stand with your feet wide on a floor that does not slip.",
	"Sit halfway down, knees pushed out over the toes, chest up. Breathe slowly.",
	"Stop if your knees cave in, or ease off if a knee hurts.")
guide("horse_stance", {
	"Stand with your feet twice your shoulders' width apart, toes pointing ahead.",
	"Bend your knees and sit your hips down, pushing your knees out over your toes.",
	"Stop when your thighs are halfway to level, chest up and back long.",
	"Hold with your hands on your hips and breathe slowly for the time shown.",
}, {
	"Press the outsides of your feet down to keep your knees out.",
	"Turn your toes out a little if your knees roll in.",
	"Keep your weight over the middle of your feet, not your toes.",
})
L["move.horse_stance.mark"] = "Thighs halfway to level, feet twice shoulder-width, toes ahead, knees out, chest up."
move("side_lunge", "Side lunge stretch", "Face the desk, hands on its edge, feet five foot-lengths apart.",
	"Bend one knee, hips back, other leg straight. Breathe slowly.",
	"Ease off at a strong pull or if the inside of a knee hurts; never bounce.")
guide("side_lunge", {
	"Face the desk with your fingertips on its edge and your feet five foot-lengths apart.",
	"Turn your toes out a little and bend one knee, sending your hips back and down.",
	"Keep the other leg straight with its heel down and its toes pointing up or ahead.",
	"Lower until the bent thigh is level, then breathe slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Keep the bent knee over its toes, not falling in.",
	"Sit back as if onto a chair to keep your chest up.",
	"Use the desk only for balance, so your legs take your weight.",
})
L["move.side_lunge.mark"] = "Bent thigh level, other leg straight with its heel down, feet five foot-lengths apart."
move("butterfly_deep", "Deep butterfly", "Sit on your bed or the floor, your back on the wall if it helps.",
	"Heels in close, let your knees sink; never push them down. Breathe slowly.",
	"Ease off at a strong pull or if the inside of a knee hurts; never bounce.")
guide("butterfly_deep", {
	"Sit tall with your knees bent and the soles of your feet together.",
	"Draw your heels in with your hands to about a hand from your body.",
	"Let both knees sink towards the bed; never press them down.",
	"Lean forward from your hips, fingertips on the bed in front of your feet.",
	"Keep your back long and breathe slowly for the time shown.",
}, {
	"Sit on a folded towel if your back rounds.",
	"Lead with your chest so your back stays long.",
	"Never press your knees down with your hands or elbows.",
})
L["move.butterfly_deep.mark"] =
	"Knees a fist from the bed, heels a hand from your body, fingertips ahead, back long."
move("side_lunge_press", "Side lunge, press and sink",
	"Face the desk, hands on its edge, feet five foot-lengths apart.",
	"Gently press the straight leg's foot down for 5 s, relax, breathe out and sink lower.",
	"Ease off at a strong pull or if the inside of a knee hurts; never bounce.")
guide("side_lunge_press", {
	"Face the desk, fingertips on its edge, feet five foot-lengths apart, toes out a little.",
	"Bend one knee and sit your hips back, the other leg straight with its heel down.",
	"Gently press the straight leg's foot into the floor, as if to pull it in, for 5 s.",
	"Relax, breathe out and sink a little lower; do three rounds.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Press at about half your strength, never as hard as you can.",
	"Keep breathing while you press; never hold your breath.",
	"Keep the bent knee over its toes, not falling in.",
})
L["move.side_lunge_press.mark"] =
	"Bent thigh level, other leg straight with its heel down, feet five foot-lengths apart."
move("wide_childs_pose", "Wide-knee child's pose", "Kneel on a rug or mat, a folded towel under your knees.",
	"Big toes together, knees wide, sink your seat back to your heels. Breathe slowly.",
	"Ease off at a strong pull inside the thighs or if a knee hurts.")
guide("wide_childs_pose", {
	"Kneel with your big toes touching and your knees twice your hips' width apart.",
	"Walk your hands forward and sink your seat back towards your heels.",
	"Reach your arms long and rest your forehead on the floor.",
	"Breathe slowly and let your hips sink a little with each breath out.",
}, {
	"Put a cushion between your seat and heels if they do not meet yet.",
	"Keep your knees wide so the stretch stays inside your thighs.",
	"Rise slowly at the end, walking your hands back in.",
})
L["move.wide_childs_pose.mark"] = "Seat on your heels, knees twice hip-width apart, big toes touching, forehead down."
move("wall_straddle", "Wall straddle", "Lie on the floor, your seat near a wall and your legs up it.",
	"Let your legs slide open down the wall; never push them. Breathe slowly.",
	"Ease off at a strong pull, and bring your legs together if your feet tingle.")
guide("wall_straddle", {
	"Sit side-on to a wall, then swing your legs up it as you lie back on the floor.",
	"Shuffle your seat towards the wall, legs straight up it, knees soft if you need.",
	"Let your legs slide open to the sides, heels resting on the wall.",
	"Rest your arms by your sides and breathe slowly for the time shown.",
	"To finish, bring your legs together, bend your knees and roll onto your side.",
}, {
	"Move your seat a little away from the wall if the backs of your legs pull.",
	"Let gravity open your legs and keep your thighs relaxed.",
})
L["move.wall_straddle.mark"] = "Legs open two thirds of the way to the floor, knees straight, heels on the wall."
move("prasarita", "Wide-leg fold", "Clear floor, feet five foot-lengths apart, two books in front.",
	"Fold from the hips, knees straight, palms down. Breathe slowly.",
	"Ease off at a strong pull or if the inside of a knee hurts; never bounce.")
guide("prasarita", {
	"Stand with your feet wide, five foot-lengths apart, toes pointing ahead.",
	"Put your hands on your hips and hinge forward from the hips with a long, flat back.",
	"Lower your palms to the floor or the books between your feet, under your shoulders.",
	"Breathe slowly in the fold, knees straight.",
	"Put your hands on your hips and rise slowly with a flat back at the end of the hold.",
}, {
	"Keep your hands on the books if the floor is too far, knees straight.",
	"Keep your weight over the middle of your feet, not back on your heels.",
	"Stand still for a breath after you rise, then step your feet in.",
})
L["move.prasarita.mark"] = "Palms flat on the floor between the feet, no books, both knees straight."
move("straddle_desk", "Wide-leg stretch at the desk", "Stand facing your desk, hands on its edge.",
	"Feet wide, knees straight, back long, chest towards the desk. Breathe slowly.",
	"Ease off at a strong pull inside the thighs or behind the knees.")
guide("straddle_desk", {
	"Stand facing your desk with your hands on its edge.",
	"Step your feet wide apart, toes pointing forward.",
	"Straighten your knees and walk your feet back until your back is level.",
	"Lower your chest towards the desk with a long back, breathing slowly.",
	"Stay for the time shown, then walk your feet in and stand up slowly.",
}, {
	"Widen your stance a little more each week.",
	"Keep your head up so your back stays long.",
	"Press your heels down and lift your kneecaps to keep the knees straight.",
})
L["move.straddle_desk.mark"] = "Feet wide, knees straight, back long, chest level with the desk."
move("ninety_ninety", "90-90 sit", "Sit on your bed or the floor.",
	"Front and back shin at right angles, sit tall, hands off. Breathe slowly.",
	"Ease off if a knee or hip hurts.")
guide("ninety_ninety", {
	"Sit on your bed with one shin in front of you, the knee bent to a right angle.",
	"Swing the other leg out to the side and back, that knee also at a right angle.",
	"Both shins rest flat, the front shin across, the back shin pointing back.",
	"Sit tall over your hips, lift your hands off and breathe slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Lean on your hands behind you until sitting tall gets easy.",
	"Turn your chest towards the front shin to keep your back long.",
	"A cushion under the front hip helps if it lifts.",
})
L["move.ninety_ninety.mark"] = "Both shins at right angles, sitting tall, hands off the bed."
move("frog_pose", "Frog", "Kneel on a rug or mat, a folded towel under each knee.",
	"Knees wide, shins parallel, ease your hips back in line with your knees. Breathe slowly.",
	"Ease off at a strong pull inside the thighs or if a knee hurts; never bounce.")
guide("frog_pose", {
	"Start on your hands and knees, a folded towel under each knee.",
	"Slide your knees out wide, shins parallel and ankles in line with your knees.",
	"Lower onto your forearms and ease your hips back until they line up with your knees.",
	"Breathe slowly and let your hips sink a little with each breath out.",
	"To come out, walk your hands back, lift your hips and slide your knees in.",
}, {
	"Flex your feet so the insides of your feet rest on the floor.",
	"Keep your back long, not sagging towards the floor.",
	"Move slowly in and out, since the frog pulls hard on the inner thighs.",
})
L["move.frog_pose.mark"] =
	"Hips in line with your knees, belly a hand from the floor, shins parallel, forearms down."
move("wide_sit_bed", "Wide sit on the bed", "Sit on a double bed, your back on the wall or the headboard.",
	"Legs wide and straight, kneecaps and toes up, sit tall. Breathe slowly.",
	"Ease off at a strong pull or if the inside of a knee hurts; never press your knees.")
guide("wide_sit_bed", {
	"Sit on the bed with your seat and back against the wall or the headboard.",
	"Open your legs wide, knees straight, kneecaps and toes pointing up.",
	"Sit tall and lift your hands off the bed, resting them on your thighs.",
	"Breathe slowly and widen your legs a little with each breath out.",
}, {
	"Sit on a folded towel if your back rounds or your knees bend.",
	"Bring your legs a little closer if your kneecaps roll in.",
	"Keep your toes pointing up, not falling out to the sides.",
})
L["move.wide_sit_bed.mark"] = "Legs a right angle apart or more, knees straight, kneecaps up, sitting tall, hands off."
move("horse_stance_deep", "Deep horse stance", "Stand with your feet wide on a floor that does not slip.",
	"Sit down until your thighs are level, knees out over the toes. Breathe slowly.",
	"Stop if your knees cave in, or ease off if a knee hurts.")
guide("horse_stance_deep", {
	"Stand with your feet twice your shoulders' width apart, toes pointing ahead.",
	"Bend your knees and sit straight down, pushing your knees out over your toes.",
	"Sit until your thighs are level with the floor, chest up and back long.",
	"Reach your arms forward at shoulder height and breathe slowly for the time shown.",
}, {
	"Widen your feet a little if your knees come forward past your toes.",
	"Press the outsides of your feet down to keep your knees out.",
	"Rise slowly at the end and never drop out of the stance.",
})
L["move.horse_stance_deep.mark"] = "Thighs level, feet twice shoulder-width, toes ahead, knees over the toes, chest up."
move("straddle_fold", "Seated straddle fold", "Sit on a folded towel, 2 m of clear floor to the sides.",
	"Legs wide, kneecaps up, fold onto your forearms. Breathe slowly.",
	"Ease off at a strong pull or if the inside of a knee hurts; never bounce.")
guide("straddle_fold", {
	"Sit tall on the towel and open your legs wide, at least a right angle apart.",
	"Point your kneecaps and toes at the ceiling and keep your knees straight.",
	"Walk your hands forward and fold from the hips with a long back.",
	"Lower onto your forearms if they reach, then breathe slowly.",
	"Walk your hands back to sit up at the end of the hold.",
}, {
	"Tip your hips forward first, so the fold comes from the hips, not a round back.",
	"Bring your legs a little closer if your kneecaps roll in.",
	"Sit higher on the towel if your back rounds before you can lean.",
})
L["move.straddle_fold.mark"] = "Legs at least a right angle apart, kneecaps up, forearms on the floor."
move("straddle_press", "Straddle fold, press and sink", "Sit on a folded towel, 2 m of clear floor to the sides.",
	"Gently press your heels down and in for 5 s, relax, breathe out and fold deeper.",
	"Ease off at a strong pull or if the inside of a knee hurts; never bounce.")
guide("straddle_press", {
	"Sit tall on the towel and open your legs wide, at least a right angle apart.",
	"Point your kneecaps and toes up and fold forward from your hips.",
	"Gently press both heels down and in for 5 s, as if to pull your legs together.",
	"Relax, breathe out and fold a little deeper; do three rounds.",
	"Walk your hands back to sit up at the end of the hold.",
}, {
	"Press at about half your strength, never as hard as you can.",
	"Keep breathing while you press; never hold your breath.",
	"Fold from the hips so your back stays long.",
})
L["move.straddle_press.mark"] = "Legs at least a right angle apart, kneecaps up, forearms on the floor, back long."
move("straddle_heel_lift", "Straddle heel lifts", "Sit on a folded towel, 2 m of clear floor to the sides.",
	"Legs wide, hands beside your hips, lift both heels for 5 counts, lower. Breathe slowly.",
	"Stop when your knees bend or a hip cramps.")
guide("straddle_heel_lift", {
	"Sit tall on the towel, legs straight and at least a right angle apart.",
	"Point your kneecaps and toes up and put your hands beside your hips.",
	"Tighten your thighs and lift both heels just off the floor for 5 counts.",
	"Lower them for a breath, then lift again until the time is up.",
}, {
	"Keep sitting tall, since leaning back makes the lift easy but misses the point.",
	"Lift one heel at a time until both together get easier.",
	"Keep breathing through each lift.",
})
L["move.straddle_heel_lift.mark"] =
	"Both heels just off the floor, legs a right angle or wider, knees straight, sitting tall."
move("standing_straddle_bed", "Standing straddle at the bed",
	"Face your bed, hands on it, barefoot on a floor that does not slip.",
	"Step your feet out wide, knees straight, seat a forearm from the floor. Breathe slowly.",
	"Ease off at a strong pull or if the inside of a knee hurts; never bounce.")
guide("standing_straddle_bed", {
	"Stand facing your bed with both hands on it, feet together.",
	"Step your feet out to the sides a little at a time, knees straight.",
	"Keep your kneecaps facing forward and lower your seat to a forearm from the floor.",
	"Lean on your hands and breathe slowly for the time shown.",
	"To come out, lean on your hands, bend your knees and step your feet in.",
}, {
	"Keep your weight in your hands so you can rise at any moment.",
	"Point your toes forward or a little out, never rolling in.",
	"Go a little wider each week, never with a jump or a bounce.",
})
L["move.standing_straddle_bed.mark"] =
	"Seat a forearm from the floor, knees straight, kneecaps forward, hands on the bed."
move("ninety_ninety_lift", "90-90 back shin lift", "Sit on your bed in the 90-90, hands on the bed behind you.",
	"Sit tall, lift the back shin and foot off the bed for 5 counts, lower. Breathe slowly.",
	"Stop if your hip cramps or pinches.")
guide("ninety_ninety_lift", {
	"Sit in the 90-90 on your bed, front shin across, back shin pointing back.",
	"Sit tall with both knees at right angles and your hands light on the bed behind you.",
	"Keep the back knee down and lift the back shin and foot off the bed for 5 counts.",
	"Lower for a breath, then lift again until the time is up.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Turn your chest towards the front shin to keep your back long.",
	"Lift only a little at first; the height comes with practice.",
	"Keep breathing through each lift.",
})
L["move.ninety_ninety_lift.mark"] = "Back shin and foot off the bed, both knees at right angles, sitting tall."
move("pancake", "Pancake", "Sit on a folded towel, 2 m of clear floor to the sides.",
	"Legs wide, kneecaps up, chest down towards the floor. Breathe slowly.",
	"Ease off at a strong pull or if the inside of a knee hurts; never bounce.")
guide("pancake", {
	"Sit tall on the towel with your legs wide, at least a right angle apart.",
	"Point your kneecaps and toes up and keep both knees straight.",
	"Tip your hips forward and walk your hands out, chest leading.",
	"Lower your chest and chin to the floor if they reach, and breathe slowly.",
	"Walk your hands back to sit up at the end of the hold.",
}, {
	"Fold from the hips, since a round back only brings your head down.",
	"Hold your feet or reach your arms ahead, whichever keeps your knees straight.",
	"Bring your legs a little closer if your kneecaps roll in.",
})
L["move.pancake.mark"] = "Legs wide, kneecaps up, chest and chin on the floor."
move("middle_split_prep", "Middle split prep", "2 m of clear floor to the sides, two books for your hands.",
	"Kneecaps forward, slide out slowly, hands take your weight. Breathe slowly.",
	"Ease off at a strong pull or if the inside of a knee hurts; never bounce.")
guide("middle_split_prep", {
	"Stand with your feet wide and your hands on the books in front of you.",
	"Slide your feet out to the sides, knees straight, kneecaps facing forward.",
	"Lower your hips slowly, your hands taking your weight.",
	"Stop with your hips about a hand from the floor and breathe slowly.",
	"To come out, lean on your hands, bend your knees and walk your feet in.",
}, {
	"A hand is about the short side of a sheet of printer paper.",
	"Keep your weight in your hands so you can rise at any moment.",
	"Slide down slowly and never drop or bounce into the split.",
})
L["move.middle_split_prep.mark"] = "Hips a hand from the floor, both knees straight, hands on the floor or books."
move("pancake_press", "Pancake, press and sink", "Sit on a folded towel, 2 m of clear floor to the sides.",
	"Gently press your heels down and in for 5 s, relax, breathe out and fold deeper.",
	"Ease off at a strong pull or if the inside of a knee hurts; never bounce.")
guide("pancake_press", {
	"Sit tall on the towel with your legs wide, kneecaps and toes pointing up.",
	"Tip your hips forward and walk your hands out, chest leading.",
	"Gently press your heels down and in for 5 s, as if to pull your legs together.",
	"Relax, breathe out and fold a little deeper; do three rounds.",
	"Walk your hands back to sit up at the end of the hold.",
}, {
	"Press at about half your strength, never as hard as you can.",
	"Keep breathing while you press; never hold your breath.",
	"Bring your legs a little closer if your kneecaps roll in.",
})
L["move.pancake_press.mark"] = "Chest and chin on the floor, legs wide, kneecaps up, back long."
move("side_split_press_bed", "Side split press at the bed",
	"Face your bed, barefoot on a floor that does not slip, 2 m of clear floor to the sides.",
	"Gently press down and in for 5 s, relax, breathe out, lean on your hands and ease wider.",
	"Ease off at a strong pull or if the inside of a knee hurts; never bounce.")
guide("side_split_press_bed", {
	"Face your bed with both hands on it and your feet wide, knees straight.",
	"Step your feet out a little at a time until your hips are about a hand from the floor.",
	"Gently press your feet down and in for 5 s, as if to pull them together.",
	"Relax, breathe out, then with your weight in your hands ease your feet a little wider.",
	"Do three rounds, then lean on your hands, bend your knees and walk your feet in.",
}, {
	"Press at about half your strength, never as hard as you can.",
	"Keep your weight in your hands so you can rise at any moment.",
	"Keep breathing while you press; never hold your breath.",
})
L["move.side_split_press_bed.mark"] =
	"Hips a hand from the floor, knees straight, kneecaps forward, hands on the bed."
move("frog_chest", "Frog, chest down", "Kneel on a rug or mat, a folded towel under each knee.",
	"Knees wide, shins parallel, lower your chest and belly to the floor. Breathe slowly.",
	"Ease off at a strong pull inside the thighs or if a knee hurts; never bounce.")
guide("frog_chest", {
	"Start in the frog on your forearms, hips in line with your knees.",
	"Slide your knees a little wider, shins parallel and feet flexed.",
	"Lower your chest and belly towards the floor, arms reaching forward.",
	"Breathe slowly and let your hips sink a little with each breath out.",
	"To come out, walk your hands back, lift your hips and slide your knees in.",
}, {
	"Keep your hips in line with your knees, not behind them.",
	"Move slowly in and out, and never rock or bounce at the bottom.",
})
L["move.frog_chest.mark"] = "Chest and belly on the floor, hips in line with the knees, shins parallel, arms long."
move("middle_split", "Middle split", "2 m of clear floor, a folded towel under your seat, books for your hands.",
	"Legs out to the sides, kneecaps forward, hands down. Breathe slowly.",
	"Ease off at a strong pull or if the inside of a knee hurts; never bounce.")
guide("middle_split", {
	"Stand with your feet wide and your hands on the floor or the books in front of you.",
	"Slide your feet out to the sides, knees straight, kneecaps facing forward.",
	"Lower your hips slowly, your hands taking your weight.",
	"Stop at a strong pull or with your hips on the towel, and breathe slowly.",
	"To come out, lean on your hands, bend your knees and walk your feet in.",
}, {
	"Keep your kneecaps facing forward or up, never rolling in.",
	"Keep your weight in your hands so you can rise at any moment.",
	"Slide down slowly and never drop or bounce into the split.",
})
L["move.middle_split.mark"] = "Legs out to the sides, hips on the floor or a folded towel, hands on floor or books."
move("middle_split_fold", "Middle split fold",
	"2 m of clear floor, a folded towel under your seat, books for your hands.",
	"In the middle split, tip your hips forward and fold onto your forearms. Breathe slowly.",
	"Ease off at a strong pull or if the inside of a knee hurts; never bounce.")
guide("middle_split_fold", {
	"Slide into the middle split with your hands on the floor or the books.",
	"Turn your kneecaps up, tip your hips forward and walk your hands out.",
	"Lower onto your forearms with a long back, hips on the floor or the towel.",
	"Breathe slowly for the time shown.",
	"To come out, walk your hands back, bend your knees and walk your feet in.",
}, {
	"Fold from the hips so your back stays long.",
	"Keep your weight in your hands until your hips rest on the floor.",
	"Slide down slowly and never drop or bounce into the split.",
})
L["move.middle_split_fold.mark"] = "Forearms on the floor, hips on the floor or a folded towel, legs out to the sides."
move("middle_split_press", "Middle split, press and sink",
	"2 m of clear floor, a folded towel under your seat, books for your hands.",
	"Gently press your feet down and in for 5 s, relax, breathe out and slide lower.",
	"Ease off at a strong pull or if the inside of a knee hurts; never bounce.")
guide("middle_split_press", {
	"Slide your feet out to the sides with your hands on the floor or the books.",
	"Stop at a strong pull, knees straight and kneecaps facing forward.",
	"Gently press your feet down and in for 5 s, as if to pull them together.",
	"Relax, breathe out and slide a little lower; do three rounds.",
	"To come out, lean on your hands, bend your knees and walk your feet in.",
}, {
	"Press at about half your strength, never as hard as you can.",
	"Keep your weight in your hands so you can rise at any moment.",
	"Keep breathing while you press; never hold your breath.",
})
L["move.middle_split_press.mark"] =
	"Hips on the floor or a folded towel, legs out to the sides, hands on the floor or books."
L["move.figure_four.mark"] = "Crossed shin level, its knee no higher than its ankle, back flat."

-- The hamstrings and the pike
move("chair_hamstring", "Chair hamstring stretch", "Sit near the front edge of your chair, feet flat.",
	"Heel down, toes up, hinge forward, then let your back round. Breathe slowly.",
	"Ease off at a strong pull or any tingling down the leg; never bounce.")
guide("chair_hamstring", {
	"Sit near the front edge of your chair with both feet flat.",
	"Slide one foot forward until that leg is straight, heel down and toes up.",
	"Hinge from your hips, then let your back round as your hands slide down the leg.",
	"Reach for the ankle with the knee straight, then breathe slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Lead with your chest, not your head, so the stretch stays behind the thigh.",
	"Keep both sit bones on the chair and the other foot flat.",
	"Rest your hands on the shin if the ankle is too far.",
})
L["move.chair_hamstring.mark"] = "Leg straight, heel down, fingertips touch the ankle."
L["move.forward_fold.mark"] = "Fingertips touch the floor, both knees straight."
move("toe_hold_fold", "Big toe fold", "Clear floor, feet hip-width apart.",
	"Hook your big toes, knees straight, head hanging. Breathe slowly.",
	"Ease off at a strong pull or any tingling down the leg; never bounce.")
guide("toe_hold_fold", {
	"Stand with your feet hip-width apart and your knees a little bent.",
	"Fold from the hips and hook each big toe with two fingers and the thumb.",
	"Straighten your knees and lift your chest to make your back long.",
	"Breathe slowly, then let your chest fold down towards your legs.",
	"Let go, bend your knees and rise slowly at the end of the hold.",
}, {
	"Bend your knees as much as you need to reach the toes, then straighten them.",
	"Pull on the toes only gently, never to force the fold.",
	"Keep your weight over the balls of your feet, not back on your heels.",
})
L["move.toe_hold_fold.mark"] = "Big toes hooked by two fingers, both knees straight."
move("seated_fold", "Seated forward fold", "Sit on a folded towel on the floor.",
	"Legs straight, fold from the hips, hold your soles. Breathe slowly.",
	"Ease off at a strong pull or any tingling down the leg; never bounce.")
guide("seated_fold", {
	"Sit tall on the towel with your legs straight and together, toes up.",
	"Reach your arms up to make your back long.",
	"Fold forward from the hips and hold the soles of your feet.",
	"Keep the backs of your knees on the floor and breathe slowly.",
	"Walk your hands back along your legs to sit up at the end of the hold.",
}, {
	"Loop a towel round your feet if your hands do not reach the soles.",
	"Grow long on each breath in and fold a little lower on each breath out.",
	"Fold from the hips, since a round back only brings your head down.",
})
L["move.seated_fold.mark"] = "Hands hold the soles, backs of the knees on the floor."
move("forehead_to_knees", "Forehead to knees fold", "Clear floor, feet together.",
	"Palms beside your feet, forehead to your knees. Breathe slowly.",
	"Ease off at a strong pull or any tingling down the leg; never bounce.")
guide("forehead_to_knees", {
	"Stand with your feet together and your knees a little bent.",
	"Fold from the hips and put your palms flat beside your feet.",
	"Straighten your knees and draw your forehead in towards them.",
	"Breathe slowly, letting the fold deepen on each breath out.",
	"Bend your knees and rise slowly at the end of the hold.",
}, {
	"Bend your elbows back to draw your chest closer to your legs.",
	"Keep your weight forward over the balls of your feet, not back on your heels.",
	"Stand still for a breath after you rise before you move on.",
})
L["move.forehead_to_knees.mark"] = "Palms flat beside the feet, forehead to the knees."
move("pike_fold", "Pike fold", "Sit on the floor, legs straight and together.",
	"Chest on your thighs, chin to your shins, knees down. Breathe slowly.",
	"Ease off at a strong pull or any tingling down the leg; never bounce.")
guide("pike_fold", {
	"Sit tall with your legs straight and together, toes up.",
	"Reach your arms up, then fold forward from the hips.",
	"Clasp your hands beyond your soles, or hold your feet.",
	"Lay your chest on your thighs and your chin on your shins, and breathe slowly.",
	"Walk your hands back up your legs to sit up at the end of the hold.",
}, {
	"Keep the backs of your knees on the floor the whole time.",
	"Bend your elbows out to the sides to draw your chest along your thighs.",
	"Fold from the hips, since a round back only brings your head down.",
})
L["move.pike_fold.mark"] = "Chest on the thighs, chin on the shins, knees down."
move("chair_toe_reach", "Chair toe reach", "Sit near the front edge of your chair, feet flat.",
	"Heel down, toes up, hinge forward and reach for your toes. Breathe slowly.",
	"Ease off at a strong pull or any tingling down the leg; never bounce.")
guide("chair_toe_reach", {
	"Sit near the front edge of your chair with both feet flat.",
	"Slide one foot forward until that leg is straight, heel down and toes up.",
	"Hinge from your hips, then let your back round as your hands slide down the leg.",
	"Reach past the ankle for your toes with the knee straight, then breathe slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Lead with your chest, not your head, so the stretch stays behind the thigh.",
	"Keep both sit bones on the chair and the other foot flat.",
	"Stop at the ankle on a tight day; the toes come with time.",
})
L["move.chair_toe_reach.mark"] = "Leg straight, heel down, fingers curled over the tips of the toes."
move("staff_pose_bed", "Staff pose on the bed", "Sit on your bed with your back against the wall or the headboard.",
	"Legs straight and together, sit tall, hands off the bed. Breathe slowly.",
	"Ease off at a strong pull behind the knees or any tingling down the legs.")
guide("staff_pose_bed", {
	"Sit on your bed with your seat and back against the wall or the headboard.",
	"Stretch both legs out straight in front of you, feet together, toes up.",
	"Press the backs of your knees down and sit tall, head against the wall.",
	"Lift your hands off the bed and reach them along your legs, breathing slowly.",
	"Stay for the time shown, then bend your knees and relax.",
}, {
	"Rest your hands beside your hips until sitting tall gets easy.",
	"Keep your low back against the wall instead of rounding it.",
	"Bend your knees a little at first, then straighten them a bit more each day.",
})
L["move.staff_pose_bed.mark"] = "Back on the wall, legs straight and together, backs of the knees down, "
	.. "hands off the bed."
move("seated_fold_bed", "Seated fold on the bed", "Sit on your bed with your legs out straight in front of you.",
	"Legs straight, fold from the hips, hold your soles. Breathe slowly.",
	"Ease off at a strong pull or any tingling down the legs; never bounce.")
guide("seated_fold_bed", {
	"Sit on your bed with both legs straight out in front of you, feet together.",
	"Sit tall, then fold forward from your hips with a long back.",
	"Slide your hands down your legs towards your feet.",
	"Hold your soles with your knees straight, then breathe slowly.",
	"Stay for the time shown, then walk your hands back and sit up slowly.",
}, {
	"Sit on a folded towel if your back rounds before you can lean forward.",
	"Lead with your chest, not your head, so the fold comes from the hips.",
	"Hold your ankles or shins until your hands reach the soles.",
})
L["move.seated_fold_bed.mark"] = "Hands hold the soles, backs of the knees down."
move("chest_to_knees_bed", "Chest to knees on the bed", "Sit on your bed with your legs out straight in front of you.",
	"Legs straight, fold until your forehead meets your knees. Breathe slowly.",
	"Ease off at a strong pull or any tingling down the legs; never bounce.")
guide("chest_to_knees_bed", {
	"Sit on your bed with both legs straight out in front of you, feet together.",
	"Hold your soles and lengthen your back as you breathe in.",
	"As you breathe out, fold from the hips and lower your chest towards your thighs.",
	"Bring your forehead to your knees with your knees straight, breathing slowly.",
	"Stay for the time shown, then sit up slowly.",
}, {
	"Fold from the hips first, then let your back round.",
	"Bend your knees a little first, bring your forehead down, then straighten them.",
	"Relax your shoulders and neck in the fold.",
})
L["move.chest_to_knees_bed.mark"] = "Forehead to the knees, hands round the soles, knees straight."
move("pike_fold_bed", "Pike fold on the bed", "Sit on your bed with your legs out straight in front of you.",
	"Legs straight, fold flat, face down to your shins. Breathe slowly.",
	"Ease off at a strong pull or any tingling down the legs; never bounce.")
guide("pike_fold_bed", {
	"Sit on your bed with both legs straight out in front of you, feet together.",
	"Hold your soles or wrap your hands around your feet.",
	"Fold from the hips, chest along your thighs, back long.",
	"Let your face come down to your shins, knees straight, and breathe slowly.",
	"Stay for the time shown, then sit up slowly.",
}, {
	"Pull gently with your arms only to help, never to force the fold.",
	"Keep your feet flexed, toes pointing up.",
	"Stay with chest to knees until the deeper fold comes easily.",
})
L["move.pike_fold_bed.mark"] = "Face on the shins, knees straight."
move("lying_hamstring", "Lying towel hamstring stretch", "Lie on your back on your bed with a towel or a belt.",
	"Towel round one sole, that leg straight up, the other leg long. Breathe slowly.",
	"Ease off at a strong pull or any tingling down the leg; never bounce.")
guide("lying_hamstring", {
	"Lie on your back on your bed with both legs long.",
	"Loop a towel or a belt round the sole of one foot and hold both ends.",
	"Straighten that leg and draw it up until you feel a gentle pull behind the thigh.",
	"Keep the other leg long on the bed and breathe slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Walk your hands up the towel to bring the leg closer; keep your shoulders down.",
	"Keep the knee straight: a lower straight leg beats a higher bent one.",
	"Relax your neck and shoulders on the bed.",
})
L["move.lying_hamstring.mark"] =
	"Heel above the middle of the other thigh, leg straight, the other leg flat on the bed."
move("doorway_hamstring", "Doorway hamstring stretch", "Lie on your back on the floor in an open doorway.",
	"One heel up the wall, that leg straight, the other flat through the door. Breathe slowly.",
	"Ease off at a strong pull or any tingling down the leg; never bounce.")
guide("doorway_hamstring", {
	"Lie on your back in an open doorway with one leg flat through the door.",
	"Rest the other heel on the wall beside the door frame, knee bent.",
	"Slide that heel up the wall until the leg is straight.",
	"Let the wall hold the leg, arms resting on the floor, and breathe slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Shuffle your seat back from the wall if the knee will not straighten.",
	"Keep your seat and low back flat on the floor.",
	"Let the wall do the holding so the leg can let go.",
})
L["move.doorway_hamstring.mark"] = "Seat a foot's length from the wall, leg straight up it, the other leg flat."
move("chair_hamstring_press", "Chair hamstring press and sink", "Sit near the front edge of your chair, feet flat.",
	"Press the heel gently down for 5 s, relax, breathe out, hinge a little deeper.",
	"Ease off at a strong pull or any tingling down the leg; never bounce.")
guide("chair_hamstring_press", {
	"Sit near the front edge of your chair and slide one leg out straight, heel down.",
	"Hinge forward with a flat back until you feel a gentle pull behind the thigh.",
	"Press the heel gently down into the floor at about half effort for 5 seconds.",
	"Relax, breathe out and hinge a little deeper; do three rounds.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Keep breathing while you press; never hold your breath.",
	"Press at about half effort, never hard.",
	"Sink only as far as the leg lets go; the next round goes a little deeper.",
})
L["move.chair_hamstring_press.mark"] = "Leg straight, heel down, fingertips touch the ankle."
move("lying_leg_lift", "Lying straight leg lift", "Lie on your back on your bed, legs long, arms by your sides.",
	"Lift one leg straight with no hands and hold it there. Breathe slowly.",
	"Ease off at a strong pull or any tingling down the leg; lower it slowly.")
guide("lying_leg_lift", {
	"Lie on your back on your bed with both legs long and your arms by your sides.",
	"Pull the toes of one foot towards you and lock that knee straight.",
	"Lift the leg slowly as high as it goes with the knee straight.",
	"Hold it there with no hands and breathe slowly, then lower it slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Keep the other leg long and resting on the bed.",
	"Stop the lift where the knee starts to bend.",
	"Keep your belly soft and your head on the bed; only the leg works.",
})
L["move.lying_leg_lift.mark"] = "Heel above the other knee, leg straight, no hands, the other leg flat."
move("bed_heel_hamstring", "Bed edge hamstring stretch", "Stand facing your bed or a sofa, about one step away.",
	"Heel on the edge, leg straight, hinge forward with a flat back. Breathe slowly.",
	"Ease off at a strong pull or any tingling down the leg; never bounce.")
guide("bed_heel_hamstring", {
	"Stand facing your bed or a sofa, about one step away.",
	"Rest one heel on the edge with that leg straight and the toes up.",
	"Stand tall on the other leg with the hand on that side on your hip.",
	"Hinge with a flat back, slide your free hand halfway down the thigh and breathe slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Keep the standing leg straight and its foot pointing forward.",
	"Lead with your chest, not your head, so the back stays flat.",
	"Stand beside a wall and touch it if you wobble.",
})
L["move.bed_heel_hamstring.mark"] = "Hand halfway down the raised thigh, both legs straight, back flat."
move("head_knee_reach", "One-leg reach on the bed",
	"Sit on your bed, one leg out straight, the other foot on its thigh.",
	"Leg straight, toes up, fold from the hips and reach for the toes. Breathe slowly.",
	"Ease off at a strong pull or any tingling down the leg; never bounce.")
guide("head_knee_reach", {
	"Sit on your bed with one leg straight out in front of you, toes up.",
	"Bend the other knee and rest that sole against the inside of the straight thigh.",
	"Turn your chest to face the straight leg and sit tall.",
	"Fold forward from the hips, reach your fingertips to the toes and breathe slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Keep the back of the straight knee down on the bed.",
	"Put a pillow under the bent knee if it does not reach the bed.",
	"Lead with your chest, not your head.",
})
L["move.head_knee_reach.mark"] = "Fingertips touch the toes, knee straight and down on the bed."
move("desk_half_fold", "Desk half fold", "Stand facing your desk or a table, hands on its edge.",
	"Walk back and hinge until your back is flat and level. Breathe slowly.",
	"Ease off at a strong pull or any tingling down the legs; never bounce.")
guide("desk_half_fold", {
	"Stand facing your desk, feet hip-width apart, hands on its edge.",
	"Walk your feet back and hinge forward from the hips.",
	"Lower your chest until your back is flat and level with the floor.",
	"Keep your knees straight and your head between your arms, and breathe slowly.",
	"Walk your feet in and stand up slowly at the end of the hold.",
}, {
	"Push your seat back as your chest goes down to keep the back flat.",
	"Bend your knees a little first, then straighten them as the legs let go.",
	"Use a higher surface if your back rounds before it is level.",
})
L["move.desk_half_fold.mark"] = "Back flat and level with the floor, knees straight, head between the arms."
move("doorway_hamstring_wall", "Doorway hamstring at the wall", "Lie on your back on the floor in an open doorway.",
	"Seat at the wall, heel up it, that leg straight, the other flat. Breathe slowly.",
	"Ease off at a strong pull or any tingling down the leg; never bounce.")
guide("doorway_hamstring_wall", {
	"Lie on your back in an open doorway with one leg flat through the door.",
	"Shuffle your seat right up to the wall beside the door frame.",
	"Rest the other heel on the wall and slide it up until the leg is straight.",
	"Let the wall hold the leg, arms resting on the floor, and breathe slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Move your seat back a little if the knee will not straighten.",
	"Keep your seat and low back flat on the floor.",
	"Let the wall do the holding so the leg can let go.",
})
L["move.doorway_hamstring_wall.mark"] = "Seat at the wall, leg straight up it, the other leg flat through the door."
move("heel_reach_fold", "Heel reach fold", "Clear floor, feet together.",
	"Fingertips to the floor beside your heels, knees straight, head hanging. Breathe slowly.",
	"Ease off at a strong pull or any tingling down the leg; never bounce.")
guide("heel_reach_fold", {
	"Stand with your feet together and your knees a little bent.",
	"Fold from the hips and reach your hands down beside your feet.",
	"Straighten your knees and touch your fingertips to the floor beside your heels.",
	"Let your back round and your head hang, and breathe slowly.",
	"Bend your knees and rise slowly at the end of the hold.",
}, {
	"Touch the floor beside your toes first, then walk your fingers back.",
	"Keep your weight over the balls of your feet, not back on your heels.",
	"Stand still for a breath after you rise before you move on.",
})
L["move.heel_reach_fold.mark"] = "Fingertips on the floor beside the heels, both knees straight."
move("head_knee_hold", "One-leg fold on the bed", "Sit on your bed, one leg out straight, the other foot on its thigh.",
	"Leg straight, both hands round the foot, back long. Breathe slowly.",
	"Ease off at a strong pull or any tingling down the leg; never bounce.")
guide("head_knee_hold", {
	"Sit on your bed with one leg straight out, the other sole against its thigh.",
	"Turn your chest to face the straight leg and sit tall.",
	"Fold forward from the hips and wrap both hands round the foot.",
	"Keep your back long and the knee down, and breathe slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Loop a towel round the foot until your hands reach it.",
	"Lead with your chest, not your head.",
	"Put a pillow under the bent knee if it does not reach the bed.",
})
L["move.head_knee_hold.mark"] = "Both hands round the foot, knee down, back long."
move("lying_hamstring_deep", "Deep lying towel stretch", "Lie on your back on your bed with a towel or a belt.",
	"Towel round one sole, that leg straight past upright, the other long. Breathe slowly.",
	"Ease off at a strong pull or any tingling down the leg; never bounce.")
guide("lying_hamstring_deep", {
	"Lie on your back on your bed with both legs long.",
	"Loop a towel or a belt round the sole of one foot and hold both ends.",
	"Straighten that leg and draw it up past upright towards you.",
	"Keep the other leg long on the bed and breathe slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Walk your hands up the towel as the leg comes closer.",
	"Keep the knee straight and your head on the bed.",
	"Keep your low back resting on the bed.",
})
L["move.lying_hamstring_deep.mark"] = "Heel over the belly button, leg straight, the other leg flat on the bed."
move("chair_leg_lift", "Seated straight leg lift", "Sit tall near the front edge of a chair, back off the backrest.",
	"Hands on hips, straighten one leg level with the seat and hold. Breathe slowly.",
	"Ease off at a strong pull or any tingling down the leg; lower it slowly.")
guide("chair_leg_lift", {
	"Sit tall near the front edge of a chair, feet flat, hands on your hips.",
	"Pull the toes of one foot up towards you.",
	"Straighten that knee until the whole leg is level with the seat.",
	"Hold it there with your back tall and breathe slowly, then lower it slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Stop where your back starts to round: sitting tall matters more than the height.",
	"Keep the thigh resting on the seat while the knee straightens.",
	"Keep both sit bones on the chair.",
})
L["move.chair_leg_lift.mark"] = "Leg straight and level with the seat, back tall, no hands."
move("seated_fold_bed_press", "Seated fold press and sink",
	"Sit on your bed with your legs out straight in front of you.",
	"Hold your soles, press the heels gently down for 5 s, relax, breathe out, sink.",
	"Ease off at a strong pull or any tingling down the legs; never bounce.")
guide("seated_fold_bed_press", {
	"Sit on your bed with both legs straight out in front of you, feet together.",
	"Fold from the hips and hold your soles until you feel a gentle pull.",
	"Press your heels gently down into the bed at about half effort for 5 seconds.",
	"Relax, breathe out and fold a little deeper; do three rounds.",
	"Stay for the time shown, then sit up slowly.",
}, {
	"Keep breathing while you press; never hold your breath.",
	"Press at about half effort, never hard.",
	"Fold from the hips, since a round back only brings your head down.",
})
L["move.seated_fold_bed_press.mark"] = "Hands hold the soles, backs of the knees down."
move("head_knee_chest", "One-leg chest to knee on the bed",
	"Sit on your bed, one leg out straight, the other foot on its thigh.",
	"Leg straight, fold until your forehead meets the knee. Breathe slowly.",
	"Ease off at a strong pull or any tingling down the leg; never bounce.")
guide("head_knee_chest", {
	"Sit on your bed with one leg straight out, the other sole against its thigh.",
	"Turn your chest to face the straight leg and hold the foot.",
	"Fold from the hips and bring your forehead to the straight knee.",
	"Keep the knee down, let your back round, and breathe slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Bend your elbows out to the sides to draw your forehead toward the knee.",
	"Fold from the hips first, then let your back round.",
	"Relax your shoulders and neck in the fold.",
})
L["move.head_knee_chest.mark"] = "Forehead to the knee, hands round the foot, knee down."
move("lying_toes_to_chin", "Lying toes to chin", "Lie on your back on your bed with a towel or a belt.",
	"Towel round one sole, draw the straight leg in, toes over your chin. Breathe slowly.",
	"Ease off at a strong pull or any tingling down the leg; never bounce.")
guide("lying_toes_to_chin", {
	"Lie on your back on your bed with both legs long.",
	"Loop a towel or a belt round the sole of one foot and hold both ends.",
	"Draw the straight leg towards you until the toes are over your chin.",
	"Keep the knee straight and the other leg long, and breathe slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Walk your hands up the towel as the leg comes closer.",
	"Keep your head and shoulders resting on the bed.",
	"Pull gently with your arms only to help, never to force the leg.",
})
L["move.lying_toes_to_chin.mark"] = "Toes over the chin, leg straight, head and the other leg flat on the bed."
move("chest_to_knees_press", "Chest to knees press and sink",
	"Sit on your bed with your legs out straight in front of you.",
	"Hold your soles, press the heels gently down for 5 s, relax, breathe out, fold.",
	"Ease off at a strong pull or any tingling down the legs; never bounce.")
guide("chest_to_knees_press", {
	"Sit on your bed with both legs straight out in front of you, feet together.",
	"Hold your soles and fold from the hips, chest towards your thighs.",
	"Press your heels gently down into the bed at about half effort for 5 seconds.",
	"Relax, breathe out and fold a little deeper; do three rounds.",
	"Stay for the time shown, then sit up slowly.",
}, {
	"Keep breathing while you press; never hold your breath.",
	"Press at about half effort, never hard.",
	"Fold from the hips first, then let your back round.",
})
L["move.chest_to_knees_press.mark"] = "Forehead to the knees, hands round the soles, knees straight."
move("head_knee_face", "One-leg face to shin on the bed",
	"Sit on your bed, one leg out straight, the other foot on its thigh.",
	"Leg straight, chest on the thigh, face down to the shin. Breathe slowly.",
	"Ease off at a strong pull or any tingling down the leg; never bounce.")
guide("head_knee_face", {
	"Sit on your bed with one leg straight out, the other sole against its thigh.",
	"Turn your chest to face the straight leg and hold the foot.",
	"Fold from the hips with your chest along the thigh and your back long.",
	"Let your face come down to the shin, knee down, and breathe slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Pull gently with your arms only to help, never to force the fold.",
	"Keep the foot flexed, toes pointing up.",
	"Stay with chest to thigh until the deeper fold comes easily.",
})
L["move.head_knee_face.mark"] = "Face on the shin, chest on the thigh, knee down."
move("face_to_shins", "Standing face to shins", "Clear floor, feet together.",
	"Hands round your calves, knees straight, face to your shins. Breathe slowly.",
	"Ease off at a strong pull or any tingling down the leg; never bounce.")
guide("face_to_shins", {
	"Stand with your feet together and your knees a little bent.",
	"Fold from the hips and wrap your hands round the backs of your calves.",
	"Straighten your knees and draw your chest onto your thighs.",
	"Let your face come to your shins and breathe slowly.",
	"Let go, bend your knees and rise slowly at the end of the hold.",
}, {
	"Bend your elbows back to draw your chest closer to your legs.",
	"Keep your weight forward over the balls of your feet.",
	"Stand still for a breath after you rise before you move on.",
})
L["move.face_to_shins.mark"] = "Face on the shins, chest on the thighs, knees straight."
move("lying_leg_to_face", "Lying leg to face", "Lie on your back on your bed, both legs long.",
	"Hold the lower shin, draw the straight leg in, shin near your face. Breathe slowly.",
	"Ease off at a strong pull or any tingling down the leg; never bounce.")
guide("lying_leg_to_face", {
	"Lie on your back on your bed with both legs long.",
	"Lift one leg straight up and hold the lower shin with both hands.",
	"Draw the leg in until the shin is a hand's width from your face.",
	"Keep the knee straight and the other leg long, and breathe slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Use a towel round the sole until your hands reach the shin.",
	"Keep your head and shoulders resting on the bed.",
	"Pull gently with your arms only to help, never to force the leg.",
})
L["move.lying_leg_to_face.mark"] = "Shin a hand's width from the face, leg straight, the other leg flat."
move("pike_fold_bed_press", "Pike fold press and sink", "Sit on your bed with your legs out straight in front of you.",
	"Fold flat, press the heels gently down for 5 s, relax, breathe out, sink.",
	"Ease off at a strong pull or any tingling down the legs; never bounce.")
guide("pike_fold_bed_press", {
	"Sit on your bed with both legs straight out in front of you, feet together.",
	"Hold your feet and fold from the hips, chest along your thighs.",
	"Press your heels gently down into the bed at about half effort for 5 seconds.",
	"Relax, breathe out and let your face come closer to your shins; do three rounds.",
	"Stay for the time shown, then sit up slowly.",
}, {
	"Keep breathing while you press; never hold your breath.",
	"Press at about half effort, never hard.",
	"Keep your feet flexed, toes pointing up.",
})
L["move.pike_fold_bed_press.mark"] = "Face on the shins, knees straight."

-- The backbend
L["move.wall_angels.mark"] = "Arms straight by your ears, thumbs a fist from the wall, low back within a hand of it."
move("standing_backbend", "Standing backbend", "Clear space, feet hip-width, the desk within reach.",
	"Hands on your low back, lift your chest, look up. Breathe slowly.",
	"Ease off if your low back pinches or your neck hurts; never bounce.")
guide("standing_backbend", {
	"Stand with your feet hip-width apart, knees soft, hips over your feet.",
	"Put your palms on your low back, fingers pointing down.",
	"Lift your chest up and back while your hips stay over your feet.",
	"Let your eyes go to the ceiling and breathe slowly.",
	"Rise slowly, chest leading, at the end of the hold.",
}, {
	"Think of lifting your chest up, not of bending your low back.",
	"Squeeze your seat so your hips stay over your feet.",
	"Keep your neck long and let your head follow your chest, never dropping back hard.",
})
L["move.standing_backbend.mark"] = "Hands on the low back, chest lifted, hips over the feet, eyes on the ceiling."
move("camel", "Camel", "Kneel on a folded towel, hands on your low back before you lean.",
	"Hips over knees, lift your chest, hands to your heels. Breathe slowly.",
	"Ease off if your low back pinches or your neck hurts; never bounce.")
guide("camel", {
	"Kneel tall with your knees hip-width apart and your thighs upright.",
	"Put your hands on your low back and lift your chest up.",
	"Lean back from the chest, hips over your knees, and reach your hands to your heels.",
	"Breathe slowly with your neck long.",
	"Bring your hands back to your low back, then rise slowly, chest first.",
}, {
	"Tuck your toes under to raise your heels closer to your hands.",
	"Keep pressing your hips forward so your thighs stay upright.",
	"Stay with your hands on your back, chest lifted, if your heels are too far.",
})
move("bed_wheel", "Wheel, hands on the bed",
	"A bed or a sofa that cannot slide, with 2 m of clear floor in front of it.",
	"Hands on the edge by your ears, press up, arms straight, hips up. Breathe slowly.",
	"Come down slowly, chin tucked, if your arms shake or your low back pinches.")
guide("bed_wheel", {
	"Lie on your back, your head at the bed's edge, knees bent and feet flat, hip-width apart.",
	"Reach back and set your hands on the edge beside your ears, fingers pointing to your feet.",
	"Lift your hips and press your arms straight, your head coming up free of the floor.",
	"Push your hips up and your chest toward the bed, and breathe slowly.",
	"Tuck your chin and lower slowly, upper back first, at the end of the hold.",
}, {
	"Keep your knees and feet pointing forward, not turning out.",
	"Press the edge away through your hands to open your shoulders.",
	"Once your arms lock easily, the wheel on the floor is the next step.",
})
L["move.bed_wheel.mark"] = "Arms straight, hips up, head free."
move("wall_books_wheel", "Wheel, hands on books",
	"A clear 2 x 2 m floor and two stacks of books, 15 cm high, flat against a wall.",
	"Hands on the books by your ears, press up, arms straight, hips up. Breathe slowly.",
	"Come down slowly, chin tucked, if your arms shake or your low back pinches.")
guide("wall_books_wheel", {
	"Set two stacks of books, about 15 cm high, flat against the wall, shoulder-width apart.",
	"Lie on your back, head toward the wall, knees bent and feet flat, hip-width apart.",
	"Reach back and set your hands on the books by your ears, fingers pointing to your feet.",
	"Lift your hips and press your arms straight, head free of the floor, and breathe slowly.",
	"Tuck your chin and lower slowly, upper back first, at the end of the hold.",
}, {
	"Keep the books pressed against the wall so they cannot slide.",
	"Press the books away through your hands to open your shoulders.",
	"Once your arms lock easily here, the wheel on the floor is the next step.",
})
L["move.wall_books_wheel.mark"] = "Hands on the books, arms straight, hips up, head free."
L["move.camel.mark"] = "Both palms on the heels, thighs upright."
move("wall_lean_back", "Wall lean-back", "Stand with your back to a wall, two foot-lengths away from it.",
	"Reach up and back, walk your hands down the wall, hips forward. Breathe slowly.",
	"Stop and stand tall if your low back pinches.")
guide("wall_lean_back", {
	"Stand with your back to a wall, heels two foot-lengths away, feet hip-width apart.",
	"Squeeze your seat and push your hips forward a little.",
	"Reach your arms up and back until your hands touch the wall.",
	"Walk your hands a little way down the wall to head height, breathing slowly.",
	"Walk your hands back up and stand tall at the end of the hold.",
}, {
	"Keep your hips forward so your upper back bends, not only your low back.",
	"Look up at your hands; never let your head drop back hard.",
	"Stand a little closer to the wall if you cannot reach it.",
})
L["move.wall_lean_back.mark"] = "Hands down the wall to head height, hips forward, arms straight."
move("wheel", "Wheel", "A clear 2 x 2 m floor that does not slip and a wall behind you.",
	"Hands by your ears, press up five times, arms and legs long. Breathe slowly.",
	"Come down slowly, chin tucked, if your arms shake or your low back pinches.")
guide("wheel", {
	"Lie on your back, knees bent, feet flat and hip-width apart near your seat.",
	"Put your hands by your ears, fingers pointing to your feet, elbows up.",
	"Lift your hips, set the top of your head down lightly, then press your arms straight.",
	"Lift your head off the floor, push your legs as straight as they go, and breathe slowly.",
	"Tuck your chin, lower slowly and press up again: five times, the last held 20 s.",
}, {
	"Keep your knees and feet pointing forward, not turning out.",
	"Press the floor away through your hands to open your shoulders.",
	"Come down at once, chin tucked, if your arms start to give.",
})
L["move.wheel.mark"] = "Up to the wheel five times, arms straight, head off the floor, last held 20 s."
move("wall_walk_half", "Wall walk to hip height", "A clear 2 x 2 m floor and a wall behind you, a long step away.",
	"Lean back, walk your hands down the wall to hip height and up. Breathe slowly.",
	"Stop and walk back up if your arms shake or your low back pinches.")
guide("wall_walk_half", {
	"Stand with your back to the wall, a long step away, feet hip-width apart.",
	"Raise your arms, lean back and put your palms on the wall, fingers pointing down.",
	"Walk your hands down the wall to the height of your hips, chest lifting, hips forward.",
	"Walk your hands back up the wall the same way until you stand tall.",
	"Go down and up five times at an easy pace, breathing slowly.",
}, {
	"Walk your hands in small steps and keep your hips pressing forward.",
	"Stop higher on the wall at first and go a little lower each day.",
	"Walk back up as soon as your arms start to tire.",
})
L["move.wall_walk_half.mark"] = "Walk down the wall to hip height and back up, five times."
move("wall_walk", "Wall walk to wheel", "A clear 2 x 2 m floor and a wall behind you, a long step away.",
	"Lean back, walk your hands down the wall and back up. Breathe slowly.",
	"Stop and walk back up if your arms shake or your low back pinches.")
guide("wall_walk", {
	"Stand with your back to the wall, a long step away, feet hip-width apart.",
	"Raise your arms, lean back and put your palms on the wall, fingers pointing down.",
	"Walk your hands down the wall, chest lifting and hips pressing forward.",
	"Set your hands on the floor in the wheel, then walk back up the wall the same way.",
	"Go down and up five times at an easy pace, breathing slowly.",
}, {
	"Walk your hands in small steps and keep your hips pressing forward.",
	"Your head is closest to the floor at the bottom, so go slowly there.",
	"Walk back up as soon as your arms start to tire.",
})
L["move.wall_walk.mark"] = "Walk down the wall to the wheel and back up, five times."
move("bed_drop_back", "Drop back to the bed",
	"A bed or sofa that cannot slide behind you, heels a forearm's length from its edge.",
	"Hips forward, arch back to the edge, rise chest first. Breathe slowly.",
	"Stop and sit on the bed if your low back pinches.")
guide("bed_drop_back", {
	"Stand with your back to the bed or sofa, heels a forearm's length from its edge.",
	"Lift your chest, press your hips forward and raise your arms over your head.",
	"Arch back slowly, eyes on the bed, and set both hands on its edge.",
	"Press your hips forward, lift your chest and rise to standing without a hard push.",
	"Go down and up five times, breathing slowly.",
}, {
	"Keep your hips pressing forward on the way down so your hands land, not your seat.",
	"Stand a little closer to the edge at first, then step away as it gets easy.",
	"Push through your legs to rise, chest leading, never with a jerk of the head.",
})
L["move.bed_drop_back.mark"] = "Heels a forearm from the edge, hands down on it and back up to standing, five times."
move("stand_to_stand", "Stand-to-stand wheel",
	"A clear 2 x 2 m floor that does not slip and a wall behind you for first tries.",
	"Arch back to the wheel, then rock up to standing. Breathe slowly.",
	"Stop and lower to the floor if your arms shake or your low back pinches.")
guide("stand_to_stand", {
	"Stand with your feet hip-width apart and the clear floor behind you.",
	"Lift your chest, press your hips forward and raise your arms over your head.",
	"Arch back slowly, eyes on the floor behind you, and set your hands down.",
	"Rock your weight onto your feet, press through your legs and rise to standing.",
	"Go down and up three times, breathing slowly.",
}, {
	"Keep your hips pressing forward on the way down so your hands land, not your head.",
	"Use the wall to walk down and up until you can do it without it.",
	"Rock a few times in the wheel to move your weight onto your feet before you rise.",
})
L["move.stand_to_stand.mark"] = "Down to the wheel and back up to standing, no wall, three times."
move("sphinx", "Sphinx", "Lie face down on a mat, elbows under your shoulders.",
	"Forearms down, draw your chest forward and up, hips heavy. Breathe slowly.",
	"Ease off if your low back pinches; keep your hips on the floor.")
guide("sphinx", {
	"Lie face down, legs long and hip-width apart, the tops of your feet down.",
	"Set your elbows under your shoulders, forearms flat and side by side.",
	"Draw your chest forward and up between your arms, hips heavy on the floor.",
	"Press your forearms down, shoulders away from your ears, and breathe slowly.",
	"Lower your chest slowly at the end of the hold.",
}, {
	"Think of pulling your chest forward through your arms, not of bending your low back.",
	"Keep your neck long and your eyes on the floor a little ahead.",
})
L["move.sphinx.mark"] = "Elbows under the shoulders, forearms flat, chest lifted, hips on the floor."
move("towel_reach", "Towel reach-back", "Roll a towel tight and lay it across the mat under your shoulder blades.",
	"Arms over your head, let them sink back toward the floor. Breathe slowly.",
	"Ease off if your shoulders pinch or your low back arches hard.")
guide("towel_reach", {
	"Lie on your back with the rolled towel across your upper back, under your shoulder blades.",
	"Bend your knees with your feet flat and hip-width apart, head resting on the mat.",
	"Reach both arms over your head, elbows straight, palms up.",
	"Let your arms and chest sink back over the towel and breathe slowly.",
	"Bring your arms back over your chest at the end of the hold.",
}, {
	"Keep your ribs soft so the bend comes from your upper back.",
	"Move the towel a hand lower for a softer bend.",
})
L["move.towel_reach.mark"] = "Towel under the shoulder blades, arms straight overhead, backs of the hands on the floor."
move("desk_chest_sink", "Desk chest sink", "Stand a long step from your desk, forearms on its top, shoulder-width.",
	"Press your elbows gently into the desk, let go, sink your chest. Breathe slowly.",
	"Ease off if your shoulders pinch or your hands tingle.")
guide("desk_chest_sink", {
	"Stand facing the desk and set your forearms on its top, shoulder-width apart.",
	"Walk your feet back until your hips are over your feet, knees soft.",
	"Let your chest sink down, your head between your upper arms, ears level with them.",
	"Press your elbows gently into the desk for 5 s at half effort, then let go.",
	"Breathe out and let your chest sink a little lower; three rounds in the hold.",
}, {
	"Keep your low back long so the bend stays in your upper back.",
	"Bring your palms together if your elbows slide apart.",
	"Keep breathing while you press; never hold your breath.",
})
L["move.desk_chest_sink.mark"] = "Forearms on the desk, hips over the feet, chest sunk below the elbows."
move("locust", "Locust", "Lie face down on a mat, arms by your sides, palms down.",
	"Lift your chest, arms and legs together, neck long. Breathe slowly.",
	"Come down slowly if your low back pinches or your neck strains.")
guide("locust", {
	"Lie face down, legs long and together, arms by your sides with your palms down.",
	"Breathe in and lift your chest, arms and both legs off the floor together.",
	"Keep your legs straight and reach your hands back toward your feet.",
	"Rest on your belly, eyes on the floor ahead, and breathe slowly.",
	"Lower slowly at the end of the hold.",
}, {
	"Lift through the length of your body, not by throwing your head back.",
	"Squeeze your seat to help your legs lift.",
})
L["move.locust.mark"] = "Chest, arms and both straight legs off the floor, belly down, neck long."
move("upward_plank", "Upward plank", "Sit on a mat, legs straight, hands behind your hips, fingers to your feet.",
	"Press up, hips high, one straight line from shoulders to feet. Breathe slowly.",
	"Come down slowly if your wrists or shoulders pinch.")
guide("upward_plank", {
	"Sit with your legs straight and your hands a hand-length behind your hips.",
	"Point your fingers toward your feet and press your palms into the floor.",
	"Lift your hips until your body is one straight line from shoulders to feet.",
	"Reach your soles toward the floor, open your chest and breathe slowly.",
	"Lower your hips slowly at the end of the hold.",
}, {
	"Squeeze your seat to keep your hips from sagging.",
	"Keep your neck long; let your head go back only as far as is easy.",
})
L["move.upward_plank.mark"] = "Arms straight under the shoulders, body one straight line, soles reaching down."
move("desk_updog", "Desk up-dog", "Stand a long step back from a desk that cannot slide, hands on its edge.",
	"Arms straight, sink your hips toward the desk, lift your chest. Breathe slowly.",
	"Ease off if your low back pinches or your wrists hurt.")
guide("desk_updog", {
	"Stand facing the desk and set your hands on its edge, shoulder-width apart.",
	"Step back onto the balls of your feet until your body leans in a straight line.",
	"Keep your arms straight and let your hips sink toward the desk.",
	"Lift your chest through your arms, shoulders down, and breathe slowly.",
	"Bring your hips back in line at the end of the hold.",
}, {
	"Press the desk away so your shoulders stay over your hands.",
	"Squeeze your seat so the bend spreads through your whole back.",
})
L["move.desk_updog.mark"] = "Arms straight, hips a hand's width from the desk's edge, chest lifted, shoulders down."
move("bow", "Bow", "Lie face down on a mat, knees hip-width apart.",
	"Hold your ankles, kick into your hands, chest and thighs up. Breathe slowly.",
	"Come down slowly if your low back or knees pinch.")
guide("bow", {
	"Lie face down, bend your knees and hold the outsides of your ankles.",
	"Keep your knees about hip-width apart.",
	"Breathe in and press your feet back into your hands to lift your chest and thighs.",
	"Rest on your belly with straight arms and breathe slowly.",
	"Let go of your ankles and lower slowly at the end of the hold.",
}, {
	"The lift comes from your legs pressing back, not from pulling with your arms.",
	"Loop a towel round your ankles if your hands cannot reach them.",
})
L["move.bow.mark"] = "Hands round the ankles, arms straight, chest and thighs off the floor."
move("locust_hands_head", "Locust, hands behind head", "Lie face down on a mat, fingers laced behind your head.",
	"Lift your chest, elbows and legs together, elbows wide. Breathe slowly.",
	"Come down slowly if your low back pinches or your neck strains.")
guide("locust_hands_head", {
	"Lie face down, legs long and together, fingers laced behind your head.",
	"Open your elbows wide to the sides.",
	"Breathe in and lift your chest, elbows and both legs off the floor together.",
	"Draw your elbows back level with your ears, rest on your belly and breathe slowly.",
	"Lower slowly at the end of the hold.",
}, {
	"Keep your hands light on your head; never pull your head forward.",
	"Squeeze your seat to help your legs lift.",
})
L["move.locust_hands_head.mark"] = "Elbows wide and level with the ears, chest and straight legs off the floor."
move("standing_knee_backbend", "Standing backbend, hands down the thighs",
	"Clear space behind you, feet hip-width, a soft mat or rug underfoot.",
	"Hips forward, chest up, slide your palms down your thighs. Breathe slowly.",
	"Ease off if your low back pinches; never bounce.")
guide("standing_knee_backbend", {
	"Stand with your feet hip-width apart, palms on the backs of your hips.",
	"Squeeze your seat and press your hips forward over your feet.",
	"Lift your chest up and back with your knees soft.",
	"Let your palms slide down the backs of your thighs as your back arches.",
	"Breathe slowly, then slide your hands back up and rise, chest first.",
}, {
	"Keep pressing your hips forward so the bend spreads up your back.",
	"Let your head follow your chest, never dropping back hard.",
})
L["move.standing_knee_backbend.mark"] = "Palms halfway down the backs of the thighs, hips over the toes, chest lifted."
move("wheel_lean", "Wheel lean", "A clear 2 x 2 m floor that does not slip and a wall behind you.",
	"In the wheel, up on your toes, push your chest over your hands. Breathe slowly.",
	"Come down slowly, chin tucked, if your arms shake or your low back pinches.")
guide("wheel_lean", {
	"Press up into the wheel, hands by your ears and feet hip-width apart.",
	"Come up onto the balls of your feet.",
	"Straighten your legs to push your chest through your arms, shoulders over your wrists.",
	"Hold with your arms straight and breathe slowly.",
	"Lower your heels, tuck your chin and come down slowly.",
}, {
	"Keep your knees and feet pointing forward, not turning out.",
	"Push from your legs, never by bending your arms.",
})
L["move.wheel_lean.mark"] = "Shoulders over the wrists, arms straight, up on the balls of the feet."
move("wheel_foot_lift", "Wheel foot lift", "A clear 2 x 2 m floor that does not slip and a wall behind you.",
	"In the wheel, lift one foot a hand's height for 2 s, then the other. Breathe slowly.",
	"Come down slowly, chin tucked, if your arms shake or your low back pinches.")
guide("wheel_foot_lift", {
	"Press up into the wheel, hands by your ears and feet hip-width apart.",
	"Shift your weight onto your hands and one foot, hips high.",
	"Lift the other foot a hand's height off the floor and hold it for 2 s.",
	"Set it down softly and lift the other foot the same way, breathing slowly.",
	"Tuck your chin and come down slowly at the end of the set.",
}, {
	"Keep your hips level as the foot lifts.",
	"Lift the foot only as high as your hips stay up.",
})
L["move.wheel_foot_lift.mark"] = "Each foot a hand's height off the floor for 2 s, arms straight, hips high."
move("standing_calf_backbend", "Standing backbend, hands to the calves",
	"A clear 2 x 2 m floor behind you and a soft mat underfoot.",
	"Hips forward, arch back, slide your palms down to your calves. Breathe slowly.",
	"Stop and slide your hands back up if your low back pinches.")
guide("standing_calf_backbend", {
	"Stand with your feet hip-width apart, palms on the backs of your hips.",
	"Squeeze your seat and press your hips well forward over your toes.",
	"Lift your chest and arch back, sliding your palms down the backs of your legs.",
	"Go past your knees to your calves, legs nearly straight, and breathe slowly.",
	"Slide your hands back up the same way and rise, chest first.",
}, {
	"Your head is below your hips here, so move slowly.",
	"Keep your hips pressing forward; if they sit back, slide your hands back up.",
})
L["move.standing_calf_backbend.mark"] = "Palms on the calves, hips forward over the toes, legs nearly straight."
move("one_leg_wheel", "One-leg wheel", "A clear 2 x 2 m floor that does not slip and a wall behind you.",
	"In the wheel, one leg straight up, hips high and level. Breathe slowly.",
	"Come down slowly, chin tucked, if your arms shake or your low back pinches.")
guide("one_leg_wheel", {
	"Press up into the wheel, hands by your ears and feet hip-width apart.",
	"Shift your weight onto your hands and one foot.",
	"Draw the other knee in, then straighten that leg up toward the ceiling.",
	"Keep your hips high and level and breathe slowly.",
	"Bend the knee, set the foot down softly, tuck your chin and come down.",
}, {
	"Point the raised foot and reach it up, not back over your head.",
	"Lift the leg only as high as your hips stay up.",
})
L["move.one_leg_wheel.mark"] = "One leg straight up, arms straight, hips high and level, head free."
move("kneeling_reach_back", "Kneeling reach-back", "Kneel on a folded towel with clear floor behind you.",
	"Arms by your ears, hips forward, arch back toward the floor. Breathe slowly.",
	"Stop and rise if your low back or knees pinch.")
guide("kneeling_reach_back", {
	"Kneel tall with your knees hip-width apart and your thighs upright.",
	"Raise your arms by your ears and press your hips forward.",
	"Arch back from your chest, reaching your hands over your head toward the floor.",
	"Hold with your hips still forward and your hands off the floor; breathe slowly.",
	"Rise slowly with your arms by your ears, chest leading.",
}, {
	"Your head is below your hips here, so move slowly.",
	"Reach only as far as you can rise from; your hands never land.",
})
L["move.kneeling_reach_back.mark"] =
	"Hips forward over the knees, arms by the ears, hands a hand's height off the floor."
move("wheel_walk_in", "Wheel walk-in", "A clear 2 x 2 m floor that does not slip and a wall behind you.",
	"In the wheel, walk your hands in small steps toward your feet. Breathe slowly.",
	"Come down slowly, chin tucked, if your arms shake or your low back pinches.")
guide("wheel_walk_in", {
	"Press up into the wheel, hands by your ears and feet hip-width apart.",
	"Lift one hand and set it a little closer to your feet, then the other.",
	"Keep stepping in until your hands are a shin's length from your heels.",
	"Hold with your arms straight and breathe slowly.",
	"Walk your hands back out the same way, tuck your chin and come down.",
}, {
	"Move one hand at a time, never both at once.",
	"Push your hips up and your chest toward your hands as they come in.",
})
L["move.wheel_walk_in.mark"] = "Hands a shin's length from the heels, arms straight, hips high."
move("wheel_heels", "Wheel, hands to heels", "A clear 2 x 2 m floor that does not slip and a wall behind you.",
	"In the wheel, walk your hands in until your fingers touch your heels. Breathe slowly.",
	"Come down slowly, chin tucked, if your arms shake or your low back pinches.")
guide("wheel_heels", {
	"Press up into the wheel, hands by your ears and feet hip-width apart.",
	"Walk your hands in toward your feet in small steps, one hand at a time.",
	"Keep going until your fingertips touch your heels, arms straight.",
	"Hold with your hips high and breathe slowly.",
	"Walk your hands back out the same way, tuck your chin and come down.",
}, {
	"Move one hand at a time, never both at once.",
	"Push your hips up over your feet as your hands come in.",
})
L["move.wheel_heels.mark"] = "Fingertips on the heels, arms straight, hips high over the feet."
move("standing_hang_back", "Standing hang-back", "A clear 2 x 2 m floor behind you and a soft mat underfoot.",
	"Arms up, hips forward, hang back with your hands toward the floor. Breathe slowly.",
	"Stop and rise at once if your legs shake; never drop.")
guide("standing_hang_back", {
	"Stand with your feet hip-width apart and raise your arms by your ears.",
	"Press your hips well forward over your toes and bend your knees a little.",
	"Arch back slowly, eyes looking for the floor, hands reaching toward it.",
	"Hold there with your hands well off the floor and breathe slowly.",
	"Rise slowly with your hips forward, arms by your ears, chest leading.",
}, {
	"Go only as far as you can rise from; your hands never land.",
	"Your head is below your hips here, so move slowly.",
	"Practise the drop back to the bed until rising feels easy.",
})
L["move.standing_hang_back.mark"] = "Hands at knee height behind you, hips forward over the toes, arms by the ears."

-- Twists
move("chair_twist", "Chair twist", "Sit tall on your chair facing the desk, feet flat.",
	"Sit tall, turn your chest, knees forward. Breathe out as you turn.",
	"Ease off if your back pinches; never pull to turn further.")
guide("chair_twist", {
	"Sit tall on your chair facing the desk, knees together and feet flat.",
	"Breathe in and grow tall through the top of your head.",
	"Breathe out and turn your chest to one side, knees still facing forward.",
	"Hold the chair back with the hand behind you and the desk edge with the other.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Grow taller on each breath in and turn a little further on each breath out.",
	"Keep both sit bones heavy on the chair as your chest turns.",
	"Turn your chest first and let your head follow last.",
})
L["move.chair_twist.mark"] = "Chest turned halfway to the side, knees forward, hands on the desk edge and chair back."
move("wall_twist", "Wall twist", "Stand side-on to a wall, a forearm's length away, feet hip-width.",
	"Hips facing ahead, turn your chest and put both palms on the wall. Breathe out to turn.",
	"Ease off if your back pinches; never push off the wall to turn further.")
guide("wall_twist", {
	"Stand side-on to a wall, a forearm's length from it, feet hip-width and facing ahead.",
	"Breathe in and stand tall through the top of your head.",
	"Breathe out and turn your chest to the wall, both palms flat on it at chest height.",
	"Keep your hips and knees facing ahead, turning a little more on each breath out.",
	"The clock times each side, then counts you in to turn round and face the other way.",
}, {
	"Squeeze a cushion between your knees if your hips want to turn with you.",
	"Let your hands rest on the wall; the turn comes from your middle back, not your arms.",
	"Turn your chest first and let your head follow last.",
})
L["move.wall_twist.mark"] = "Hips and knees facing ahead, both palms flat on the wall at chest height."
move("side_lying_twist", "Side-lying twist", "Lie on your side on the floor, hips and knees bent to a right angle.",
	"Keep your knees stacked, open the top arm behind you, eyes follow. Breathe slowly.",
	"Ease off if your shoulder or back pinches.")
guide("side_lying_twist", {
	"Lie on your side, a cushion under your head, hips and knees bent to a right angle.",
	"Stack your knees and rest your bottom arm on the floor in front of you.",
	"Breathe out and open the top arm up and over to the floor behind you, chest turning up.",
	"Stay there and let the arm and shoulder blade sink a little on each breath out.",
	"The clock times each side, then counts you in to roll over.",
}, {
	"Keep the top knee on the bottom knee so the turn comes from your upper back.",
	"Rest the arm on a cushion while it does not reach the floor yet.",
	"Let the arm hang heavy rather than pressing it down.",
})
L["move.side_lying_twist.mark"] =
	"Top knee resting on the bottom knee, back of the top hand resting on the floor."
move("easy_twist", "Cross-legged twist", "Sit cross-legged on a folded towel on the floor.",
	"Sit tall, one hand behind you, the other on the knee you turn to. Breathe out as you turn.",
	"Ease off if your back pinches; never pull to turn further.")
guide("easy_twist", {
	"Sit cross-legged on a folded towel, one shin in front of the other, knees low.",
	"Put one hand on the floor behind you and the other on the knee on that side.",
	"Breathe in and sit tall, then breathe out and turn your chest towards the back hand.",
	"Keep both sit bones heavy on the towel and grow tall on each breath in.",
	"The clock times each side, then counts you in to turn the other way.",
}, {
	"Lean on the back hand only lightly, so your back stays upright.",
	"Swap which shin is in front when you switch sides.",
	"Turn your chest first and let your head follow last.",
})
L["move.easy_twist.mark"] = "Sitting tall and cross-legged, chest turned halfway to the side, hand on the floor behind."
move("chair_twist_press", "Chair twist, press and sink", "Sit tall on your chair facing the desk, feet flat.",
	"Turn, gently press as if to turn back for 5 s, relax, breathe out and turn a little more.",
	"Ease off if your back pinches; press gently, never hard.")
guide("chair_twist_press", {
	"Sit tall facing the desk and turn your chest to one side, knees still facing forward.",
	"Hold the chair back with the hand behind you and the desk edge with the other.",
	"Gently press your hands as if to turn back, about half effort, for 5 s, still breathing.",
	"Let go of the press, breathe out and turn a little further; do three rounds in all.",
	"The clock times each side, then counts you in to switch.",
}, {
	"The press is light and your body stays still while you press; only the effort changes.",
	"Breathe the whole time and never hold your breath while you press.",
	"Turn only as far as the stretch stays strong but easy.",
})
L["move.chair_twist_press.mark"] =
	"After the third round, chest halfway to the side, knees forward, hands on desk and chair."
move("revolved_chair", "Twisted chair pose", "Feet together, the desk within reach.",
	"Palms together, elbow outside the far knee. Breathe out to turn.",
	"Ease off if your back pinches or a knee hurts; never pull to turn further.")
guide("revolved_chair", {
	"Stand with your feet together and sit back as if onto a chair.",
	"Press your palms together at your chest.",
	"Breathe out and turn, hooking one elbow outside the other knee.",
	"Keep your knees level and breathe slowly, growing long on each breath in.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Keep your knees side by side, since the one you hook will want to slide ahead.",
	"Press your palms together to turn your chest towards the ceiling.",
	"Keep your weight in your heels, so you could lift your toes.",
})
L["move.revolved_chair.mark"] = "Palms together, elbow outside the opposite knee, knees level."
move("twisted_triangle_books", "Twisted triangle on books",
	"Clear floor, feet a leg-length apart, books 15 cm high by your front foot.",
	"Hips square, knees straight, palm on the books, turn. Breathe out to turn.",
	"Ease off if your back pinches; never pull to turn further.")
guide("twisted_triangle_books", {
	"Step one foot back a leg-length, back toes turned out a little, hips facing forward.",
	"Reach the arm on the back-leg side forward and hinge over the front leg, back flat.",
	"Put that palm flat on the books just outside the front foot.",
	"Breathe out and turn your chest open, top arm reaching straight up.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Draw the front hip back so your hips stay level as you turn.",
	"Step the back foot in a little if you wobble, hips still facing forward.",
	"Look down at the books if looking up bothers your neck.",
})
L["move.twisted_triangle_books.mark"] = "Palm flat on the books outside the front foot, knees straight, top arm up."
move("foot_up_twist", "Foot-up standing twist", "Face your bed or sofa and put one foot up on its edge.",
	"Hips level, far upper arm on the outside of the raised knee, turn. Breathe out to turn.",
	"Ease off if your back pinches or a knee hurts; never pull to turn further.")
guide("foot_up_twist", {
	"Face the bed or sofa and put one foot up on its edge, that knee bent and pointing ahead.",
	"Keep the standing leg straight and both hips level and facing the bed.",
	"Breathe in and stand tall, then breathe out and turn towards the raised knee.",
	"Press the far upper arm on the outside of the raised knee, the other hand on your hip.",
	"The clock times each side, then counts you in to switch legs.",
}, {
	"Stand near a wall and touch it with your free hand if you wobble.",
	"Keep the raised knee pointing ahead; it will want to swing in towards your arm.",
	"Grow tall on each breath in and turn a little on each breath out.",
})
L["move.foot_up_twist.mark"] =
	"Foot on the bed edge, hips level, far upper arm against the outside of the raised knee."
move("kneeling_turn", "Kneeling chest turn", "Hands and knees on the floor, a folded towel under your knees.",
	"Hand behind your head, turn the elbow up to the ceiling, eyes follow. Breathe slowly.",
	"Ease off if your neck, shoulder or wrist pinches.")
guide("kneeling_turn", {
	"Set your hands under your shoulders and your knees under your hips, back flat.",
	"Put one hand behind your head, its elbow pointing forwards and down.",
	"Breathe out and turn that elbow up towards the ceiling, your chest turning with it.",
	"Stay at the top and breathe slowly, pressing the floor away with the other hand.",
	"The clock times each side, then counts you in to switch hands.",
}, {
	"Keep your hips right over your knees so the turn comes from your upper back.",
	"Let your eyes follow the elbow, and keep the hand light behind your head.",
	"Spread the fingers of the floor hand wide and press through the whole hand.",
})
L["move.kneeling_turn.mark"] = "Hips over the knees, hand behind your head, elbow pointing straight at the ceiling."
move("spinal_twist_press", "Seated twist, press and sink", "Sit tall on the floor.",
	"Gently press elbow and knee into each other for 5 s, relax, breathe out and turn more.",
	"Ease off if your back pinches; press gently, never hard.")
guide("spinal_twist_press", {
	"Sit with one leg long and bend the other knee up, its foot flat by the long thigh.",
	"Turn towards the bent knee and hook the far elbow outside it, the other hand behind you.",
	"Gently press the knee and the elbow into each other, about half effort, for 5 s.",
	"Let go, breathe out and turn a little further; do three rounds in all.",
	"The clock times each side, then counts you in to switch legs and turn the other way.",
}, {
	"Your body stays still while you press; only the effort changes.",
	"Breathe the whole time and never hold your breath while you press.",
	"Sit up tall before each new turn, both sit bones heavy on the floor.",
})
L["move.spinal_twist_press.mark"] =
	"After the third round, sitting tall, elbow outside the bent knee, chest past that knee."
move("revolved_triangle", "Twisted triangle", "Clear floor, feet a leg-length apart, a book by your front foot.",
	"Hips square, knees straight, top arm up. Breathe out to turn.",
	"Ease off if your back pinches; never pull to turn further.")
guide("revolved_triangle", {
	"Step one foot back a leg-length, back toes turned out a little, hips facing forward.",
	"Reach the arm on the back-leg side forward and hinge over the front leg with a flat back.",
	"Put that palm on the floor or the book outside the front foot.",
	"Breathe out and turn your chest open, top arm reaching straight up.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Draw the front hip back so your hips stay level as you turn.",
	"Use the book or set the hand inside the foot if the floor is too far.",
	"Look down at the floor if looking up bothers your neck.",
})
L["move.revolved_triangle.mark"] = "Palm flat on the floor outside the front foot, no book, knees straight, arm up."
move("revolved_wide_fold", "Twisted wide-leg fold", "Clear floor, feet a leg-length apart and facing ahead.",
	"Back flat, one palm under your shoulder, other arm up, chest open. Breathe out to turn.",
	"Ease off if your back pinches; never pull to turn further.")
guide("revolved_wide_fold", {
	"Stand with your feet a leg-length apart, toes facing ahead and knees straight.",
	"Hinge forward with a flat back and put both palms on the floor under your shoulders.",
	"Keep one palm there, breathe out and turn your chest open, the other arm reaching up.",
	"Breathe slowly, growing long through your back on each breath in.",
	"The clock times each side, then counts you in to turn the other way.",
}, {
	"Keep your weight over your feet rather than leaning on the hand.",
	"Look down at the floor if looking up bothers your neck.",
	"Come up slowly with your hands on your hips, so your head rises last.",
})
L["move.revolved_wide_fold.mark"] = "Knees straight, back flat, palm flat on the floor under your shoulder, top arm up."
move("revolved_half_moon_books", "Twisted half moon on books",
	"Clear floor, books 20 cm high, the desk at your side within reach of your top hand.",
	"On one leg, hand on the books, back leg level, turn your chest up. Breathe slowly.",
	"Put the foot down if you wobble hard; ease off if your back pinches.")
guide("revolved_half_moon_books", {
	"Stand on the leg nearer the desk, the books on the floor a foot ahead of that foot.",
	"Hinge forward and put the hand opposite your standing leg on the books.",
	"Lift the back leg until it is level with your hips, both legs straight.",
	"Breathe out and turn your chest open, the top arm reaching up and the hips level.",
	"The clock times each side; to switch legs, turn round so the desk is by the other leg.",
}, {
	"Keep most of your weight on the standing leg, its knee straight but soft.",
	"Look down at the books if looking up makes you wobble.",
	"Practise with the back foot against a wall until the balance is steady.",
})
L["move.revolved_half_moon_books.mark"] =
	"Standing knee straight, back leg level, hand on the books, chest turned, top arm up."
move("half_fish_twist", "Foot-over seated twist", "Sit tall on a folded towel on the floor, legs long.",
	"Foot over the long knee, upper arm outside the bent knee, turn. Breathe out to turn.",
	"Ease off if your back pinches; never pull to turn further.")
guide("half_fish_twist", {
	"Sit tall with both legs long, then step one foot over the other knee, flat on the floor.",
	"Plant the hand on the bent-knee side on the floor just behind your hips.",
	"Breathe out and turn towards the bent knee, the other arm hooking outside it.",
	"Slide that arm down until your armpit rests against the outside of the knee.",
	"The clock times each side, then counts you in to switch legs and turn the other way.",
}, {
	"Grow taller on every breath in and turn a little further on every breath out.",
	"Keep both sit bones heavy on the towel as your chest turns.",
	"Keep the bent knee pointing up rather than letting it drop across your body.",
})
L["move.half_fish_twist.mark"] =
	"Foot over the long knee, far armpit against the outside of the bent knee, eyes past it."
move("bound_twist", "Bound seated twist", "Sit tall on a folded towel on the floor.",
	"Arm round the knee, clasp behind your back. Breathe out to turn.",
	"Ease off if your back pinches; never pull on the bind.")
guide("bound_twist", {
	"Sit tall with one leg long and the other knee bent, its foot flat by the long thigh.",
	"Breathe in and grow tall, then breathe out and turn towards the bent knee.",
	"Wrap the opposite arm round the outside of the bent knee.",
	"Reach the other hand behind your back, clasp your fingers and breathe slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Hold a towel between your hands if your fingers do not meet.",
	"Keep the long leg flat on the floor and both sit bones down.",
	"Sit tall before each turn and never twist with a round back.",
})
L["move.bound_twist.mark"] = "Arm round the bent knee, hands clasped behind your back or on a towel, long leg flat."
move("lunge_prayer_twist", "Lunge prayer twist", "A long stride of clear floor.",
	"Long lunge, palms together, elbow outside the front knee. Breathe out to turn.",
	"Ease off if your back pinches or your front knee hurts; never pull to turn further.")
guide("lunge_prayer_twist", {
	"Step into a long lunge, front knee over the ankle, back heel up and back knee straight.",
	"Press your palms together at your chest.",
	"Breathe out and turn, hooking the far elbow outside the front knee.",
	"Press your palms together to turn your chest up, and breathe slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Drop the back knee to the floor while you find the hook, then lift it again.",
	"Keep the front knee over the ankle, not drifting in toward the elbow.",
	"Grow tall on each breath in and turn a little on each breath out.",
})
L["move.lunge_prayer_twist.mark"] = "Back knee straight, far elbow outside the front knee, palms together, chest up."
move("revolved_half_moon", "Twisted half moon",
	"Clear floor, books near in case the floor is far, the desk in reach of your top hand.",
	"On one leg, hand on the floor, back leg level, turn your chest up. Breathe slowly.",
	"Put the foot down if you wobble hard; ease off if your back pinches.")
guide("revolved_half_moon", {
	"Stand on the leg nearer the desk and hinge forward, the back leg lifting behind you.",
	"Put the hand opposite your standing leg on the floor, a foot ahead of your standing foot.",
	"Lift the back leg until it is level with your hips, both legs straight.",
	"Breathe out and turn your chest open, the top arm reaching up and the hips level.",
	"The clock times each side; to switch legs, turn round so the desk is by the other leg.",
}, {
	"Keep most of your weight on the standing leg and only fingertip weight on the hand.",
	"Look down at the floor if looking up makes you wobble.",
	"Go back to the books for a day if the floor makes the turn shrink.",
})
L["move.revolved_half_moon.mark"] =
	"Standing knee straight, back leg level, hand on the floor, chest turned, top arm up."
move("half_fish_foot_hold", "Seated twist, hand to foot", "Sit tall on a folded towel on the floor, legs long.",
	"Foot over the long knee, arm straight outside the bent knee to the foot. Breathe out.",
	"Ease off if your back or shoulder pinches; never pull to turn further.")
guide("half_fish_foot_hold", {
	"Sit tall with both legs long, then step one foot over the other knee, flat on the floor.",
	"Plant the hand on the bent-knee side behind your hips and sit up tall.",
	"Breathe out and turn, sliding the other arm outside the bent knee up to the armpit.",
	"Straighten that arm down the outside of the shin and hold the foot or the ankle.",
	"The clock times each side, then counts you in to switch legs and turn the other way.",
}, {
	"Hold the ankle first, then walk the hand down to the foot over the days.",
	"Keep the long leg straight and both sit bones heavy on the towel.",
	"Turn your chest first and let your head follow last.",
})
L["move.half_fish_foot_hold.mark"] =
	"Foot over the long knee, far arm straight outside the bent knee, hand holding that foot."
move("bound_side_angle", "Bound lunge twist", "A long stride of clear floor, a towel ready for your hands.",
	"Turn, arm round the front thigh, clasp behind your back. Breathe out.",
	"Ease off if your back pinches or a knee hurts; never pull on the bind.")
guide("bound_side_angle", {
	"Step into a long lunge, front knee over the ankle, back knee straight.",
	"Press your palms together and turn, hooking the far elbow outside the front knee.",
	"Thread that arm under the front thigh and reach the other hand behind your back.",
	"Clasp your hands or the towel behind you, chest turned up, and breathe slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Drop the back knee to the floor while you find the clasp, then lift it again.",
	"Hold the towel between your hands if your fingers do not meet.",
	"Grow tall on each breath in and turn a little on each breath out.",
})
L["move.bound_side_angle.mark"] = "Back knee straight, arms round the front thigh, hands clasped behind or on a towel."
move("bound_revolved_triangle", "Bound twisted triangle",
	"Clear floor, feet a leg-length apart, a towel ready for your hands.",
	"Turn, lower arm under the front thigh, clasp behind your back. Breathe out to turn.",
	"Ease off if your back pinches or a hamstring tugs hard; never pull on the bind.")
guide("bound_revolved_triangle", {
	"Step one foot back a leg-length, back toes turned out a little, both knees straight.",
	"Hinge over the front leg with a flat back and turn into the twisted triangle.",
	"Thread the lower arm under the front thigh, close to the hip, and bend that elbow.",
	"Reach the top arm behind your back and clasp your hands or the towel; breathe slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Hold the towel between your hands if your fingers do not meet.",
	"Draw the front hip back so your hips stay level as you turn.",
	"Look down at the floor if looking up bothers your neck.",
})
L["move.bound_revolved_triangle.mark"] =
	"Knees straight, lower arm under the front thigh, hands clasped behind or on a towel."
move("half_fish_bound", "Bound folded-leg twist", "Sit on a folded towel on the floor, beside your foot, never on it.",
	"Arm round the raised knee, hands clasped behind your back. Breathe out to turn.",
	"Ease off if a knee or your back pinches; never pull on the bind.")
guide("half_fish_bound", {
	"Fold one leg on the floor, its heel beside the other hip, both sit bones on the towel.",
	"Step the other foot over that knee, flat on the floor, and sit up tall.",
	"Breathe out and turn towards the raised knee, wrapping the far arm round its outside.",
	"Reach the other hand behind your back and clasp your hands or a towel; breathe slowly.",
	"The clock times each side, then counts you in to switch legs and turn the other way.",
}, {
	"Keep the bottom leg long, as in the foot-over twist, if the folded knee feels strained.",
	"Hold a towel between your hands if your fingers do not meet.",
	"Sit tall before each turn and never twist with a round back.",
})
L["move.half_fish_bound.mark"] =
	"Bottom heel by your hip, arm round the raised knee, hands clasped behind or on a towel."

-- The ankles and the deep squat
L["move.calf_stretch.mark"] = "Back heel flat, back knee straight, back foot pointing forward."
move("soleus_stretch", "Bent-knee calf stretch", "Hands on the wall, one foot a short step back.",
	"Back heel down, bend the back knee, sink gently. Breathe slowly.",
	"Ease off at a strong pull or if your heel or ankle hurts; never bounce.")
guide("soleus_stretch", {
	"Face the wall with your hands flat on it at shoulder height.",
	"Step one foot a short step back, toes pointing at the wall, heel flat on the floor.",
	"Bend both knees and sink your hips down until the low calf of the back leg pulls.",
	"Keep the back heel pressed down and breathe slowly.",
	"The clock times each leg, then counts you in to switch.",
}, {
	"A shorter step back moves the stretch lower, towards the heel.",
	"Keep the back knee pointing over your middle toes.",
	"The bent knee stretches the low calf, the part a deep squat needs.",
})
L["move.soleus_stretch.mark"] = "Back heel flat, back knee bent over the toes, back foot pointing at the wall."
move("knee_to_wall", "Knee to the wall", "Face the wall, front toes four fingers from it, hands on the wall.",
	"Press the ball of your foot down gently, relax, breathe out, knee to the wall.",
	"Ease off at a pinch at the front of the ankle; never bounce.")
guide("knee_to_wall", {
	"Face the wall, hands on it, front toes four fingers away and the other foot a step back.",
	"Bend the front knee towards the wall, the front heel flat on the floor.",
	"Press the ball of the front foot gently into the floor at half effort for 5 s.",
	"Relax, breathe out and ease the knee closer to the wall; do three rounds.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Measure four fingers once with your hand and mark the spot on the floor.",
	"Aim your knee over your middle toes, never inwards.",
	"Keep breathing through each press; never hold your breath.",
})
L["move.knee_to_wall.mark"] = "Front toes four fingers from the wall, front knee touching it, front heel flat."
move("heel_raised_desk_squat", "Desk squat, heels raised",
	"Face the desk, hands on its edge, a thick book or rolled towel under your heels.",
	"Sit deep, heels on the book, hands light on the desk. Breathe slowly.",
	"Ease off at a pinch at the front of the ankle or if a knee hurts; never bounce.")
guide("heel_raised_desk_squat", {
	"Face the desk with your hands on its edge and your feet hip-width apart.",
	"Rest your heels on a thick book or a rolled towel, toes on the floor.",
	"Sit your hips down between your heels until your thighs are below level.",
	"Let your hands rest lightly on the desk and breathe slowly.",
	"Stand up slowly, using the desk, at the end of the hold.",
}, {
	"The raised heels let your knees go forward while your ankles loosen.",
	"Push your knees out over your toes to make room for your hips.",
	"Use a thinner book over the weeks until your heels reach the floor.",
})
L["move.heel_raised_desk_squat.mark"] = "Heels raised on a book or towel, thighs below level, hands light on the desk."
move("desk_squat", "Deep squat at the desk", "Face the desk, hands on its edge, feet hip-width.",
	"Sit deep, heels down, hands light on the desk. Breathe slowly.",
	"Ease off at a pinch at the front of the ankle or if a knee hurts; never bounce.")
guide("desk_squat", {
	"Face the desk with your hands on its edge and your feet hip-width apart.",
	"Turn your toes out a little and sit your hips down between your heels.",
	"Sink until your thighs are below level, heels flat on the floor.",
	"Let your hands rest lightly on the desk and breathe slowly.",
	"Stand up slowly, using the desk, at the end of the hold.",
}, {
	"Push your knees out over your toes to make room for your hips.",
	"Put a rolled towel under your heels if they lift.",
	"Hold the desk only for balance, so your legs take your weight.",
})
L["move.desk_squat.mark"] = "Heels flat on the floor, thighs below level, hands light on the desk."
move("half_kneel_ankle", "Half-kneeling ankle stretch",
	"Kneel on one knee on a folded towel, the other foot flat in front.",
	"Drive the front knee over the toes, heel down, body tall. Breathe slowly.",
	"Ease off at a pinch at the front of the ankle or if a knee hurts; never bounce.")
guide("half_kneel_ankle", {
	"Kneel on one knee on a folded towel, the other foot flat on the floor in front.",
	"Rest your hands on your hips and keep your body upright.",
	"Lean your weight forward so the front knee travels past your toes.",
	"Keep the front heel flat on the floor and breathe slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Check it once: toes a fist from a wall, the knee just touches it.",
	"Aim the front knee over your middle toes so it tracks straight.",
	"Lean in a little further on each breath out, only as far as a steady pull.",
})
L["move.half_kneel_ankle.mark"] = "Half kneeling, front heel flat, front knee a fist past the toes, body upright."
move("desk_squat_press", "Desk squat, press and sink", "Face the desk, hands on its edge, feet hip-width.",
	"Press your feet down gently, relax, breathe out and sink lower.",
	"Ease off at a pinch at the front of the ankle or if a knee hurts; never bounce.")
guide("desk_squat_press", {
	"Face the desk, hands on its edge, and sit into the deep squat with your heels flat.",
	"Press your feet gently into the floor at half effort for 5 s, as if to stand up.",
	"Relax, breathe out and let your hips sink a little lower.",
	"Do three rounds, breathing all the way, then stand up slowly using the desk.",
}, {
	"Half effort is enough; the sink after the press is what counts.",
	"Keep breathing as you press; never hold your breath.",
	"Put a rolled towel under your heels if they lift.",
})
L["move.desk_squat_press.mark"] = "Heels flat on the floor, hips sunk below the knees, hands light on the desk."
move("heel_raised_squat", "Deep squat, heels raised",
	"Clear floor, feet hip-width, a thick book or rolled towel under your heels.",
	"Heels on the book, palms together, elbows inside the knees. Breathe slowly.",
	"Ease off at a pinch at the front of the ankle or if a knee hurts; never bounce.")
guide("heel_raised_squat", {
	"Stand with your heels on a thick book or a rolled towel, feet hip-width apart.",
	"Sit your hips down between your heels until your thighs are below level.",
	"Press your palms together at your chest, elbows inside your knees.",
	"Keep your chest up and your hands off the floor, and breathe slowly.",
	"Put your hands on the floor and stand up slowly at the end of the hold.",
}, {
	"The raised heels let you squat with no hands while your ankles loosen.",
	"Use a thinner book over the weeks until your heels reach the floor.",
	"Press your elbows gently into your knees to open them over your toes.",
})
L["move.heel_raised_squat.mark"] = "Heels raised on a book or towel, thighs below level, palms together, no hands down."
move("deep_squat", "Deep squat", "Clear floor, feet hip-width, toes turned out a little.",
	"Heels down, palms together, elbows inside the knees. Breathe slowly.",
	"Ease off at a pinch at the front of the ankle or if a knee hurts; never bounce.")
guide("deep_squat", {
	"Stand with your feet hip-width apart and your toes turned out a little.",
	"Sit your hips down between your heels until your thighs are below level.",
	"Press your palms together at your chest, elbows inside your knees.",
	"Keep your heels flat and your chest up, and breathe slowly.",
	"Put your hands on the floor and stand up slowly at the end of the hold.",
}, {
	"Press your elbows gently into your knees to open them over your toes.",
	"Put a rolled towel under your heels if they lift.",
	"Lift your chest so your back stays long, not rounded.",
})
L["move.deep_squat.mark"] = "Heels on the floor, palms together, elbows inside the knees, no hands down."
move("narrow_squat", "Narrow deep squat", "Clear floor, feet a fist apart, toes pointing straight ahead.",
	"Feet close, heels down, knees over the toes, arms forward. Breathe slowly.",
	"Ease off at a pinch at the front of the ankle or if a knee hurts; never bounce.")
guide("narrow_squat", {
	"Stand with your feet a fist apart and your toes pointing straight ahead.",
	"Sit your hips down until your thighs are below level, heels flat on the floor.",
	"Let your knees travel forward over your toes and reach your arms forward.",
	"Keep your chest up and your heels down, and breathe slowly.",
	"Put your hands on the floor and stand up slowly at the end of the hold.",
}, {
	"Start with your feet a little wider and bring them closer over the days.",
	"Put a rolled towel under your heels if they lift; the goal pose needs them on the floor.",
	"Keep your knees pointing the same way as your toes.",
})
L["move.narrow_squat.mark"] = "Feet a fist apart, toes ahead, heels on the floor, thighs below level, no hands down."
move("deep_squat_press", "Deep squat, press and sink", "Clear floor, feet hip-width, toes turned out a little.",
	"Press your knees gently in against your elbows, relax, breathe out, sink lower.",
	"Ease off at a pinch at the front of the ankle or if a knee hurts; never bounce.")
guide("deep_squat_press", {
	"Sit into the deep squat, heels flat, palms together and elbows inside your knees.",
	"Press your knees gently in against your elbows at half effort for 5 s.",
	"Relax, breathe out and let your elbows ease your knees wider as your hips sink.",
	"Do three rounds, then put your hands on the floor and stand up slowly.",
}, {
	"Half effort is enough; keep breathing the whole time.",
	"Keep your heels flat and your chest up as you sink.",
	"Put a rolled towel under your heels if they lift.",
})
L["move.deep_squat_press.mark"] =
	"Heels on the floor, hips low, palms together, elbows inside the knees, no hands down."
move("bag_hug_squat", "Bag hug squat", "Clear floor, feet hip-width, a light bag or backpack hugged to your chest.",
	"Hug the bag, sit between your heels, elbows inside the knees. Breathe slowly.",
	"Ease off at a pinch at the front of the ankle or if a knee hurts; never bounce.")
guide("bag_hug_squat", {
	"Hold a light bag or backpack against your chest with both arms.",
	"Stand with your feet hip-width apart and your toes turned out a little.",
	"Sit your hips down between your heels until your thighs are below level.",
	"Push your knees out with your elbows, heels flat, and breathe slowly.",
	"Stand up slowly at the end of the hold, the bag still close to your chest.",
}, {
	"A light bag is enough: its weight helps you sit deeper and stay upright.",
	"Keep the bag close to your chest; held far out it loads your back.",
	"Set the bag down before you put your hands on the floor.",
})
L["move.bag_hug_squat.mark"] = "A bag hugged to your chest, heels flat, thighs below level, elbows inside the knees."
move("overhead_squat", "Deep squat, arms up", "Clear floor, feet hip-width, toes turned out a little.",
	"Heels down, arms reaching up, elbows straight. Breathe slowly.",
	"Ease off at a pinch at the front of the ankle, in a knee or a shoulder; never bounce.")
guide("overhead_squat", {
	"Stand with your feet hip-width apart and your toes turned out a little.",
	"Reach both arms straight up, hands shoulder-width apart.",
	"Sit your hips down between your heels until your thighs are below level.",
	"Keep your arms up, your chest tall and your heels flat, and breathe slowly.",
	"Lower your arms, put your hands on the floor and stand up slowly.",
}, {
	"Lift your chest so your arms reach up, not straight ahead.",
	"Hold a towel tight between your hands if it helps keep your arms straight.",
	"Let your arms come forward a little before you let your heels lift.",
})
L["move.overhead_squat.mark"] = "Heels flat, thighs below level, arms reaching up and forward, elbows straight."
move("feet_together_squat", "Feet-together squat", "Clear floor, feet together, a rolled towel at hand.",
	"Feet and knees together, heels down, hold your ankles. Breathe slowly.",
	"Ease off at a pinch at the front of the ankle or if a knee hurts; never bounce.")
guide("feet_together_squat", {
	"Stand with your feet and knees together.",
	"Sit your hips down low, keeping your heels flat.",
	"Lean your chest forward over your knees and hold your ankles from behind.",
	"Breathe slowly and let your head drop towards your toes.",
	"Put your hands on the floor and stand up slowly at the end of the hold.",
}, {
	"Put the rolled towel under your heels if they lift.",
	"Reach your arms forward for balance if you tip back.",
	"Keep your knees together and pointing straight ahead.",
})
L["move.feet_together_squat.mark"] = "Feet and knees together, heels on the floor, hands round the ankles."
move("knee_over_toe_kneel", "Knee over toe kneel", "Kneel on one knee on a folded towel, the other foot flat in front.",
	"Slide the front knee far over the toes, heel down, body tall. Breathe slowly.",
	"Ease off at a pinch at the front of the ankle or if a knee hurts; never bounce.")
guide("knee_over_toe_kneel", {
	"Kneel on one knee on a folded towel, the other foot flat on the floor in front.",
	"Slide the front knee forward over your toes until the thigh rests on the calf.",
	"Keep the front heel flat, your body upright and your hands on your hips.",
	"Breathe slowly and let the front ankle bend a little more on each breath out.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Bring the front foot closer to the back knee if the front heel lifts.",
	"Keep the front knee pointing over your middle toes.",
	"Build up over the weeks; it is a deep bend for the front knee.",
})
L["move.knee_over_toe_kneel.mark"] = "Front heel flat, front thigh resting on the calf, back knee down, body upright."
move("feet_together_reach", "Feet-together squat, arms forward", "Clear floor, feet together, a rolled towel at hand.",
	"Feet and knees together, heels down, arms reaching forward. Breathe slowly.",
	"Ease off at a pinch at the front of the ankle or if a knee hurts; never bounce.")
guide("feet_together_reach", {
	"Stand with your feet and knees together.",
	"Sit your hips down low, keeping your heels flat.",
	"Reach both arms straight forward at shoulder height, hands off your legs.",
	"Hold yourself in the squat with your legs and breathe slowly.",
	"Put your hands on the floor and stand up slowly at the end of the hold.",
}, {
	"Lean your chest a little further forward if you tip back.",
	"Put the rolled towel under your heels if they lift.",
	"Keep your knees together and pointing straight ahead.",
})
L["move.feet_together_reach.mark"] = "Feet and knees together, heels down, arms reaching forward, no hands on the legs."
move("feet_together_press", "Feet-together squat, press and sink",
	"Clear floor, feet together, a rolled towel at hand.",
	"Press the balls of your feet down gently, relax, breathe out, knees forward.",
	"Ease off at a pinch at the front of the ankle or if a knee hurts; never bounce.")
guide("feet_together_press", {
	"Squat with your feet and knees together, heels flat, hands round your ankles.",
	"Press the balls of your feet gently into the floor at half effort for 5 s.",
	"Relax, breathe out and let your knees travel a little further over your toes.",
	"Do three rounds, then put your hands on the floor and stand up slowly.",
}, {
	"Half effort is enough; keep breathing the whole time.",
	"Put the rolled towel under your heels if they lift.",
	"Reach your arms forward for balance if you tip back.",
})
L["move.feet_together_press.mark"] =
	"Feet and knees together, heels on the floor, hips low, hands round the ankles."
move("bound_squat", "Bound squat", "Clear floor, feet together, a towel ready for your hands.",
	"Heels down, arms round your shins, clasp behind. Breathe slowly.",
	"Ease off at a pinch at the front of the ankle or if a knee hurts; never bounce.")
guide("bound_squat", {
	"Squat with your feet and knees together and your heels flat.",
	"Lean forward and wrap your arms round your shins from the front.",
	"Clasp your hands behind your calves, or hold the towel between them.",
	"Let your head drop forward over your knees and breathe slowly.",
	"Let go, put your hands on the floor and stand up slowly.",
}, {
	"Put a rolled towel under your heels if they lift.",
	"Keep your weight over your feet so you do not rock back.",
	"Let your breath fold you in, never a pull on the clasp.",
})
L["move.bound_squat.mark"] = "Feet together, heels on the floor, arms round the shins, hands clasped or on a towel."
move("squat_twist_wall", "Squat twist at the wall",
	"Clear floor, side-on to a wall, feet together a forearm's length from it.",
	"Heels down, turn to the wall, armpit outside the knees, hands on the wall. Breathe out.",
	"Ease off at a pinch at the front of the ankle, in a knee or in your back.")
guide("squat_twist_wall", {
	"Stand side-on to a wall, feet together, a forearm's length from it.",
	"Squat with your feet and knees together and your heels flat.",
	"Breathe out, turn to the wall and bring the arm away from it outside both knees.",
	"Put both hands flat on the wall, that hand low and the other high, and breathe slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Put a rolled towel under your heels if they lift; the goal pose needs them on the floor.",
	"Turn a little more on each breath out, never by pushing hard on the wall.",
	"To switch sides, turn round so the wall is on your other side.",
})
L["move.squat_twist_wall.mark"] = "Feet together, heels on the floor, armpit outside the knees, both hands on the wall."
move("knee_over_toe_hover", "Knee over toe hover", "Clear floor beside the desk, a hand free to catch it.",
	"Front thigh on the calf, back knee up, body tall, hands free. Breathe slowly.",
	"Ease off at a pinch at the front of the ankle or in a knee; touch the desk if you sway.")
guide("knee_over_toe_hover", {
	"Stand beside the desk and take a long step forward with one foot.",
	"Bend both knees and lower until the front thigh rests on the calf, heel flat.",
	"Keep the back knee just off the floor and your body upright, hands free.",
	"Breathe slowly and hold, the front heel down the whole time.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Rest your fingertips on the desk while you learn, then let go.",
	"Keep the back knee a hand's width off the floor, no higher.",
	"Bring the front foot a little closer if the front heel lifts.",
})
L["move.knee_over_toe_hover.mark"] =
	"Front heel flat, front thigh on the calf, back knee off the floor, body upright, no hands."
move("squat_prayer_twist", "Squat prayer twist", "Clear floor, feet together, a rolled towel at hand.",
	"Heels down, palms together, elbow outside both knees, turn. Breathe out.",
	"Ease off at a pinch at the front of the ankle, in a knee or in your back.")
guide("squat_prayer_twist", {
	"Squat with your feet and knees together and your heels flat.",
	"Reach your arms forward for balance, then press your palms together at your chest.",
	"Breathe out, turn to one side and hook the far elbow outside both knees.",
	"Press palm into palm to turn your chest a little further, and breathe slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Put the rolled towel under your heels if they lift; the goal pose needs them flat.",
	"Turn a little more on each breath out, never by forcing the elbow.",
	"Keep your weight over your feet so you do not tip back.",
})
L["move.squat_prayer_twist.mark"] =
	"Feet together, heels on the floor, elbow outside both knees, palms pressed together."
move("noose", "Bound squat twist", "Clear floor, feet together, a towel ready for your hands.",
	"Heels down, far arm round both thighs, clasp behind. Breathe out.",
	"Ease off at a pinch at the front of the ankle or in your back; never bounce.")
guide("noose", {
	"Squat with your feet and knees together and your heels flat.",
	"Breathe out and turn your chest to one side, the far arm outside both knees.",
	"Wrap that arm round both thighs and reach the other hand behind your back.",
	"Clasp your hands or the towel behind you and breathe slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Put a rolled towel under your heels if they lift.",
	"Squat with your back near a wall while you learn, in case you tip back.",
	"Turn on each breath out and never pull on the clasp to go further.",
})
L["move.noose.mark"] = "Heels on the floor, feet together, arm round both thighs, hands or a towel clasped behind."
move("noose_press", "Bound squat twist, press and sink", "Clear floor, feet together, a towel ready for your hands.",
	"Press the arm gently against your thighs, relax, breathe out and turn further.",
	"Ease off at a pinch at the front of the ankle or in your back; never bounce.")
guide("noose_press", {
	"Come into the bound squat twist, heels flat, hands or the towel clasped behind.",
	"Press the wrapped arm gently against your thighs at half effort for 5 s.",
	"Relax, breathe out and turn your chest a little further.",
	"Do three rounds, breathing all the way, then let go slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Half effort is enough; never pull on the clasp to turn.",
	"Keep breathing through each press; never hold your breath.",
	"Put a rolled towel under your heels if they lift.",
})
L["move.noose_press.mark"] = "Feet together, heels on the floor, arm round both thighs, hands or a towel clasped."
move("noose_reach", "Squat twist, arms reaching", "Clear floor, feet together, a rolled towel at hand.",
	"Heels down, armpit past both knees, arms reaching, hands free. Breathe out.",
	"Ease off at a pinch at the front of the ankle or in your back; never bounce.")
guide("noose_reach", {
	"Squat with your feet and knees together and your heels flat.",
	"Breathe out and turn until the far armpit passes the outside of both knees.",
	"Reach that arm down past your shins and the other arm back behind you.",
	"Hold the turn with your own muscles, hands touching nothing, and breathe slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Put the rolled towel under your heels if they lift; the goal pose needs them flat.",
	"Turn a little more on each breath out; never swing into the turn.",
	"It builds the turn the bound squat twist needs before the hands meet.",
})
L["move.noose_reach.mark"] = "Feet together, heels on the floor, armpit past both knees, both arms reaching, no bind."

-- Balance
L["move.tree_pose.mark"] = "Sole on the inner calf or thigh, hands together, no touch-down."
move("eagle", "Eagle", "Stand side-on to the desk, a hand's reach from it.",
	"Sit low, wrap the legs, cross the arms at the elbows. Breathe slowly.",
	"Put the foot down when you sway or your knee hurts.")
guide("eagle", {
	"Stand side-on to the desk and bend your knees a little.",
	"Cross one thigh over the other and hook the top toes behind the calf, or let them hang.",
	"Cross your arms at the elbows and rest each hand on the opposite shoulder.",
	"Sit a little lower, lift your elbows to shoulder height and breathe slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Let the top foot hang if the hook will not come; never force it.",
	"Fix your eyes on one still spot to steady your balance.",
	"Squeeze your thighs together to lock the pose in place.",
})
L["move.eagle.mark"] = "Thighs crossed, top foot hooked or hanging, arms crossed at the elbows, no touch-down."
move("wall_warrior3", "Warrior III at the wall", "Face a wall, a little more than an arm's length from it.",
	"Hands on the wall, back leg up, one level line. Breathe slowly.",
	"Put the foot down when you sway or your back pinches.")
guide("wall_warrior3", {
	"Stand facing the wall, a little more than an arm's length from it.",
	"Hinge forward from the hips and put your palms on the wall at hip height.",
	"Shift onto one foot and lift the other leg behind you until body and leg are level.",
	"Keep the standing leg straight and the hips level, and breathe slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Step a little closer or further until your arms, body and back leg make one line.",
	"Point the back toes at the floor so your hips stay level.",
	"Press the wall lightly so your legs and your back do the holding.",
})
L["move.wall_warrior3.mark"] = "Hands on the wall, body and back leg level with the floor, standing leg straight."
move("warrior3", "Warrior III", "Stand side-on to the desk, fingertips within reach of it.",
	"Arms, body and back leg in one level line. Breathe slowly.",
	"Put the foot down when you sway or your back pinches.")
guide("warrior3", {
	"Stand side-on to the desk and reach both arms up beside your ears.",
	"Shift your weight onto one foot and lift the other foot behind you.",
	"Tip forward from the hip as the back leg rises, until arms, body and leg are level.",
	"Keep the standing leg straight, hips level, and breathe slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Rest your fingertips on the desk while you find the line, then lift them.",
	"Point the back toes at the floor so your hips stay level.",
	"Reach long through your hands and your back heel, as if pulled both ways.",
})
L["move.warrior3.mark"] = "Arms, body and back leg level with the floor, standing leg straight, hands off the desk."
move("tree_eyes_closed", "Tree pose, eyes closed", "Stand side-on to the desk, fingertips just over it.",
	"Foot on the calf, hands together, eyes closed. Breathe slowly.",
	"Open your eyes and put the foot down when you sway.")
guide("tree_eyes_closed", {
	"Stand side-on to the desk, feet together, fingertips hovering just above it.",
	"Lift one foot and set its sole on your inner calf, below the knee.",
	"Bring your hands together at your chest and find your balance.",
	"Close your eyes for 10 s or more and breathe slowly, keeping still.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Fix your eyes on one spot first, then close them once you are steady.",
	"Press the foot and the calf into each other to stay steady.",
	"Keep your standing knee soft, not locked.",
})
L["move.tree_eyes_closed.mark"] = "Eyes closed for 10 s of the hold, no touch-down and no hand on the desk."
move("desk_heel_hold", "Heel on the desk", "Face the desk, about a leg's length from it.",
	"Heel on the desk, both legs straight, stand tall. Breathe slowly.",
	"Ease off at a strong pull or any tingling down the leg; put the foot down if you sway.")
guide("desk_heel_hold", {
	"Stand facing the desk, about a leg's length away, hands on your hips.",
	"Lift one leg and rest its heel on the desk, the leg straight and the toes up.",
	"Stand tall on a straight standing leg, both hips level and facing the desk.",
	"Breathe slowly and let the back of the raised leg lengthen.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Put the heel on a bed's edge or a stair until the leg will straighten on the desk.",
	"Keep your chest lifted, not folded over the leg.",
	"Fix your eyes on one still spot ahead to steady your balance.",
})
L["move.desk_heel_hold.mark"] = "Heel on the desk, both knees straight, standing tall, hips level."
move("big_toe_hold", "Standing big toe hold", "Stand side-on to the desk, a towel at hand.",
	"Hold the big toe, leg straight at hip height. Breathe slowly.",
	"Put the foot down when you sway or your back rounds.")
guide("big_toe_hold", {
	"Stand with the desk beside your standing leg and one hand on your hip.",
	"Lift the other knee and hook its big toe with two fingers and the thumb.",
	"Straighten that leg out in front of you, up to hip height.",
	"Stand tall on a straight standing leg and breathe slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Loop the towel round the foot or keep the knee bent until the leg will straighten.",
	"Keep both hips level, since the raised hip will want to lift.",
	"Fix your eyes on one still spot ahead to steady your balance.",
})
L["move.big_toe_hold.mark"] = "Leg straight at hip height, held by the big toe or a towel round the foot, hand on hip."
move("dancer", "Dancer", "Face a wall an arm's length away, a towel at hand.",
	"Both hands hold the back foot overhead, thigh level. Breathe slowly.",
	"Put the foot down when you sway or your low back pinches.")
guide("dancer", {
	"Stand facing the wall, bend one knee and hold that foot behind you.",
	"Loop the towel round the foot if your hands cannot both reach it.",
	"Take the foot or the towel in both hands over your head, elbows pointing up.",
	"Kick the foot back into your hands and tip your chest forward until the thigh is level.",
	"Breathe slowly; the clock times each side, then counts you in to switch.",
}, {
	"Walk your hands down the towel a little closer to the foot each time.",
	"Keep the standing knee straight and your hips square to the wall.",
	"Kick the foot into your hands rather than pulling with your arms.",
})
L["move.dancer.mark"] = "Both hands hold the back foot, or a towel round it, over your head, thigh level."
move("stork_head_turns", "One-leg stand, head turns", "Stand side-on to the desk, fingertips just over it.",
	"On one foot, turn your head slowly from side to side. Breathe slowly.",
	"Put the foot down when you sway.")
guide("stork_head_turns", {
	"Stand side-on to the desk, feet together, fingertips hovering just above it.",
	"Shift onto one foot, bend the other knee and lift that foot behind you, shin level.",
	"Put your hands on your hips and fix your eyes on a spot ahead.",
	"Turn your head slowly to one side, then the other, and keep turning the whole hold.",
	"The clock times each side, then counts you in to switch feet.",
}, {
	"Turn slowly, about two seconds each way, and let your eyes travel with your head.",
	"Keep your hips level and your standing knee soft, not locked.",
	"Turn less far if you feel dizzy; small turns still train your balance.",
})
L["move.stork_head_turns.mark"] =
	"Shin level behind you, hands on hips, head turning slowly side to side, no touch-down."
move("knee_hug_stand", "Standing knee hug", "Stand side-on to the desk, a hand's reach from it.",
	"Hug one knee to your chest and stand tall on the other leg. Breathe slowly.",
	"Put the foot down when you sway or your hip pinches.")
guide("knee_hug_stand", {
	"Stand side-on to the desk, a hand's reach from it, feet together.",
	"Shift onto one foot and lift the other knee up in front of you.",
	"Wrap both arms round the shin and hug the knee in towards your chest.",
	"Stand tall on a straight standing leg, hips level, and breathe slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Lift the knee up to your chest; do not lean down to meet it.",
	"Fix your eyes on one still spot ahead to steady your balance.",
	"Let one hand go to the desk if you tip, then hug the knee again.",
})
L["move.knee_hug_stand.mark"] = "Knee hugged to your chest, standing leg straight, standing tall, no touch-down."
move("towel_stand", "One-leg stand on a towel",
	"A bath towel folded thick on a floor that does not slip, by the desk.",
	"On the towel on one foot, thigh level, hands on hips. Breathe slowly.",
	"Put the foot down when you sway or your ankle rolls.")
guide("towel_stand", {
	"Fold a bath towel into a thick square and lay it on the floor beside the desk.",
	"Step onto the towel with one foot, side-on to the desk, fingertips just over it.",
	"Lift the other knee in front of you until the thigh is level.",
	"Put your hands on your hips and breathe slowly as the towel shifts under you.",
	"The clock times each side, then counts you in to switch feet.",
}, {
	"Let your foot make small moves on the towel; that is your balance at work.",
	"Fix your eyes on one still spot ahead to steady yourself.",
	"Keep the standing knee soft, not locked, and the hips level.",
})
L["move.towel_stand.mark"] = "On the folded towel, thigh level in front, hands on hips, no touch-down."
move("standing_figure_four", "Standing figure four", "Stand side-on to the desk, fingertips just over it.",
	"Ankle over the standing knee, sit back, palms together. Breathe slowly.",
	"Put the foot down when you sway or the pull in your hip grows strong.")
guide("standing_figure_four", {
	"Stand side-on to the desk, feet together, fingertips hovering just above it.",
	"Shift onto one foot and cross the other ankle over that knee, the crossed knee out.",
	"Bend the standing knee and send your seat back as if to sit on a chair.",
	"Bring your palms together at your chest, chest lifted, and breathe slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Pull the toes of the crossed foot back to keep the ankle steady on the knee.",
	"Sit back until the crossed shin lies level, and no deeper for now.",
	"Keep the standing knee over your toes, not falling in.",
})
L["move.standing_figure_four.mark"] = "Ankle across the standing knee, crossed shin level, seat back, palms together."
move("tree_eyes_up", "Tree pose, eyes up", "Stand side-on to the desk, fingertips just over it.",
	"Foot on the calf or thigh, arms up, look at the ceiling. Breathe slowly.",
	"Put the foot down when you sway.")
guide("tree_eyes_up", {
	"Stand side-on to the desk, feet together, fingertips hovering just above it.",
	"Lift one foot and set its sole on your inner calf or thigh, never on the knee.",
	"Reach both arms straight up beside your ears, palms facing in.",
	"Once steady, tip your head back slowly and look up at the ceiling.",
	"The clock times each side, then counts you in to switch feet.",
}, {
	"Find one still spot on the ceiling to look at, as you would ahead.",
	"Tip your head back only as far as your neck is happy to go.",
	"Press the foot and the leg into each other to lock the pose in place.",
})
L["move.tree_eyes_up.mark"] = "Sole on the inner calf or thigh, arms straight up, eyes on the ceiling, no touch-down."
move("half_moon_books", "Half moon on books",
	"Clear floor, books 25 cm high by your front foot, the desk behind you.",
	"Hand on the books, top leg level, hips stacked, top arm up. Breathe slowly.",
	"Put the foot down when you sway or your hip or back pinches.")
guide("half_moon_books", {
	"Stand with your feet wide, the books by your front foot, the desk a step behind you.",
	"Bend the front knee and put that hand on the books, a foot's length past the toes.",
	"Shift onto the front foot and lift the back leg until it is level with the floor.",
	"Straighten the standing leg, stack the top hip over the bottom one, top arm up.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Point the top toes ahead, not up, so your hips stay stacked.",
	"Look at the floor while you find the line, then turn your eyes to the side.",
	"Press the books lightly and let the standing leg take your weight.",
})
L["move.half_moon_books.mark"] = "Palm on the books, top leg level, hips stacked, top arm straight up."
move("knee_out_hold", "Knee out to the side", "Stand side-on to the desk, fingertips just over it.",
	"Hold the knee out to the side at hip height and stand tall. Breathe slowly.",
	"Put the foot down when you sway or the pull in your hip grows strong.")
guide("knee_out_hold", {
	"Stand side-on to the desk, the desk by your standing leg, feet together.",
	"Lift the other knee in front of you and hold it from below with the same hand.",
	"Open the knee out to the side, keeping it at hip height, the foot hanging.",
	"Stand tall, let go, hands on your hips, and hold the knee up for 5 s.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Open the knee only as far as your hips stay level.",
	"Keep your chest facing ahead as the knee turns out.",
	"Fix your eyes on one still spot ahead to steady your balance.",
})
L["move.knee_out_hold.mark"] = "Bent knee out to the side at hip height, hands on hips for 5 s, hips level."
move("figure_four_fold", "Standing figure four, folded", "Stand side-on to the desk, fingertips just over it.",
	"Ankle over the knee, sit back and fold your chest down. Breathe slowly.",
	"Put the foot down when you sway or the pull in your hip grows strong.")
guide("figure_four_fold", {
	"Come into the standing figure four, ankle over the standing knee, palms together.",
	"Sit back a little more and fold your chest forward over the crossed shin.",
	"Lower the chest until it is directly over the shin, your back long.",
	"Keep your weight in the standing heel and breathe slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Fold from the hips, never by rounding your back.",
	"Rest your hands on the crossed shin if you wobble, then lift them again.",
	"Pull the toes of the crossed foot back to keep the ankle steady on the knee.",
})
L["move.figure_four_fold.mark"] = "Ankle across the standing knee, back long, chest directly over the crossed shin."
move("tiptoe_tree", "Tree pose on tiptoe", "Stand side-on to the desk, fingertips just over it.",
	"Foot on the calf or thigh, rise onto the ball of the foot. Breathe slowly.",
	"Put the foot down when you sway or your calf cramps.")
guide("tiptoe_tree", {
	"Stand side-on to the desk, feet together, fingertips hovering just above it.",
	"Come into the tree, sole on your inner calf or thigh, hands together at your chest.",
	"Rise slowly onto the ball of the standing foot, heel lifted high.",
	"Hold steady at the top and breathe slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Rise straight up over the big toe, not out to the little toe.",
	"Lower the heel for a breath and rise again if you wobble.",
	"Fix your eyes on one still spot at eye level.",
})
L["move.tiptoe_tree.mark"] = "Heel high for 10 s, sole on the inner calf or thigh, hands together, no touch-down."
move("towel_eyes_closed", "Towel stand, eyes closed",
	"A bath towel folded thick on a floor that does not slip, by the desk.",
	"On the towel on one foot, close your eyes for 5 s. Breathe slowly.",
	"Open your eyes and put the foot down when you sway.")
guide("towel_eyes_closed", {
	"Fold a bath towel into a thick square on the floor beside the desk.",
	"Step onto it with one foot, side-on to the desk, fingertips hovering just above it.",
	"Lift the other foot beside the standing ankle and put your hands on your hips.",
	"Once steady, close your eyes for 5 s or more and breathe slowly.",
	"The clock times each side, then counts you in to switch feet.",
}, {
	"Fix your eyes on one spot first, then close them once you are steady.",
	"Let your foot make small moves on the towel; that is your balance at work.",
	"Open your eyes at once if you tip, and touch the desk.",
})
L["move.towel_eyes_closed.mark"] =
	"On the folded towel, eyes closed for 5 s of the hold, hands on hips, no touch-down."
move("eagle_towel", "Eagle on a towel",
	"A bath towel folded thick on a floor that does not slip, by the desk.",
	"On the towel, sit low, wrap the legs, cross the arms at the elbows. Breathe slowly.",
	"Put the foot down when you sway or your knee strains.")
guide("eagle_towel", {
	"Fold a bath towel into a thick square on the floor beside the desk and stand on it.",
	"Bend your knees a little and cross one thigh over the other, toes hooked or hanging.",
	"Cross your arms at the elbows and rest each hand on the opposite shoulder.",
	"Sit a little lower, lift your elbows to shoulder height and breathe slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Learn the eagle on the floor first; the towel makes the same shape harder.",
	"Squeeze your thighs together to lock the pose in place.",
	"Let the top foot hang if the hook will not come; never force it.",
})
L["move.eagle_towel.mark"] =
	"On the folded towel, thighs crossed, foot hooked or hanging, arms crossed, no touch-down."
move("side_toe_hold", "Big toe hold to the side", "Stand side-on to the desk, a towel at hand.",
	"Hold the big toe and open the straight leg out to the side. Breathe slowly.",
	"Put the foot down when you sway or the pull behind your knee grows strong.")
guide("side_toe_hold", {
	"Stand with the desk beside your standing leg and that hand on your hip.",
	"Lift the other knee and hook its big toe with two fingers and the thumb.",
	"Straighten the leg in front of you, then open it out to the side, foot at knee height.",
	"Stand tall on a straight standing leg, hips level, and breathe slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Loop the towel round the foot or keep the knee bent until the leg will straighten.",
	"Keep both hips level as the leg opens, since the raised hip will want to lift.",
	"Fix your eyes on one still spot ahead to steady your balance.",
})
L["move.side_toe_hold.mark"] = "Straight leg out to the side, foot at knee height, toe or towel held, hips level."
move("half_moon_float", "Half moon, hand floating",
	"Clear floor, books 25 cm high by your front foot, the desk behind you.",
	"In the half moon, lift the bottom hand just off the books. Breathe slowly.",
	"Come down to the books with your hand when you sway or your back pinches.")
guide("half_moon_float", {
	"Come into the half moon on books, top leg level and top arm straight up.",
	"Shift a little more of your weight onto the standing foot.",
	"Lift the bottom hand a hand's width off the books and hold it there.",
	"Keep the hips stacked and the top leg level, and breathe slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Lift the hand for a breath, touch down, then lift it again for longer.",
	"Reach long through the top heel to keep the leg level.",
	"Keep your eyes on the floor ahead until the float feels steady.",
})
L["move.half_moon_float.mark"] = "Top leg level, hips stacked, bottom hand a fist above the books, top arm straight up."
move("tiptoe_eagle", "Eagle on tiptoe", "Stand side-on to the desk, a hand's reach from it.",
	"In the eagle, rise onto the ball of the standing foot. Breathe slowly.",
	"Put the foot down when you sway or your knee strains.")
guide("tiptoe_eagle", {
	"Stand side-on to the desk and come into the eagle, sitting a little low.",
	"Cross the thighs, hook the top toes or let them hang, and cross your arms.",
	"Rise slowly onto the ball of the standing foot, heel lifted high.",
	"Keep the knee bent and the thighs squeezed, and breathe slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Rise straight up over the big toe, not out to the little toe.",
	"Lower the heel for a breath and rise again if you wobble.",
	"Fix your eyes on one still spot at eye level.",
})
L["move.tiptoe_eagle.mark"] =
	"Heel high for 10 s, thighs crossed, foot hooked or hanging, arms crossed, no touch-down."
move("half_moon_bow", "Half moon bow",
	"Clear floor, books 25 cm high by your front foot, the desk behind you, a towel at hand.",
	"In the half moon, bend the top knee and hold that foot. Breathe slowly.",
	"Put the foot down when you sway or your low back pinches.")
guide("half_moon_bow", {
	"Come into the half moon on books, top leg level and top arm straight up.",
	"Bend the top knee and reach the top hand back to hold the foot or a towel round it.",
	"Kick the foot back into your hand until the thigh is level again.",
	"Open your chest towards the ceiling and breathe slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Loop the towel round the foot first if your hand cannot reach it.",
	"Kick the foot into your hand rather than pulling with your arm.",
	"Keep the standing leg straight and the bottom hand a fist off the books.",
})
L["move.half_moon_bow.mark"] =
	"Bottom hand a fist off the books, top hand on the foot or towel, thigh level, chest open."
move("leg_up_no_hands", "Leg up, no hands", "Stand side-on to the desk, a towel at hand.",
	"Leg straight at hip height, let go of the toe and hold it up. Breathe slowly.",
	"Put the foot down when you sway or the front of your hip cramps.")
guide("leg_up_no_hands", {
	"Come into the standing big toe hold, the leg straight in front at hip height.",
	"Stand tall on a straight standing leg, the other hand on your hip.",
	"Let go of the toe, that hand to your hip, and hold the leg up by itself for 5 s.",
	"Take the toe again, then lower the leg slowly and breathe.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Keep the knee straight as you let go; a high leg with a bent knee does not count.",
	"Squeeze the front of the thigh hard to keep the leg up.",
	"Stand tall, since leaning back to lift the leg takes the work off the hip.",
})
L["move.leg_up_no_hands.mark"] = "Leg straight ahead at hip height, hands on hips, held with no hands for 5 s."

-- Arm balances
move("wrist_lean", "Wrist lean at the desk", "A steady desk, palms flat on it, fingers pointing away from you.",
	"Arms straight, lean your shoulders past your wrists. Breathe slowly.",
	"Ease off at once if a wrist hurts.")
guide("wrist_lean", {
	"Stand a step back from the desk and put your palms flat on its edge, shoulder-width apart.",
	"Point your fingers away from you and spread them wide.",
	"Keep your arms and body straight and lean forward until your shoulders pass your wrists.",
	"Breathe slowly in the lean, then ease back.",
	"Lean in and ease back at a slow pace for the time shown.",
}, {
	"Press the whole hand into the desk, most of all the base of the first finger.",
	"Lean a little further over the days, only as far as your wrists stay easy.",
	"Check that the desk stays steady before you lean your weight on it.",
})
L["move.wrist_lean.mark"] = "Palms flat, fingers forward, arms straight, shoulders a fist past the wrists."
move("crow_prep", "Crow prep", "Squat on the floor, a pillow in front of your hands.",
	"Shins on your upper arms, heels up, lean forward. Breathe slowly.",
	"Ease off if a wrist or a knee hurts.")
guide("crow_prep", {
	"Squat with your feet close together and your knees wide.",
	"Put your palms flat on the floor, shoulder-width apart, fingers spread.",
	"Bend your elbows a little and rest your shins high on your upper arms.",
	"Rise onto your toes and lean forward until your weight is in your hands.",
	"Breathe slowly in the lean, then lower your heels at the end of the hold.",
}, {
	"Look at the floor a little in front of your hands, not at your feet.",
	"Squeeze your knees into your arms so they stay put.",
	"Press the floor away so your upper back rounds a little.",
})
L["move.crow_prep.mark"] = "Shins on the upper arms, both heels up, shoulders over the fingertips."
move("crow_one_foot", "Crow, one foot up", "A pillow on the floor in front of your hands in case you tip forward.",
	"Knees on upper arms, lean forward, lift one foot. Breathe slowly.",
	"Come down at once if a wrist or an elbow hurts.")
guide("crow_one_foot", {
	"Start in crow prep: hands flat, shins high on your upper arms, heels up.",
	"Lean forward until your toes grow light.",
	"Lift one foot and draw its heel toward your seat, the other toes resting lightly.",
	"Hold with your eyes ahead of your hands and breathe slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Lean only as far as you can press back with the foot still down.",
	"Squeeze your knees into your arms so they stay put.",
	"Look forward at the floor, not down at your hands.",
})
L["move.crow_one_foot.mark"] = "One foot off the floor for the whole hold on each side, knees on the upper arms."
-- Both feet up from books before the floor (2026-10-13)
move("crow_books", "Crow from books", "Toes on a stack of books 12 cm high, a pillow in front of your hands.",
	"Knees on upper arms, lean forward, lift both feet off the books. Breathe slowly.",
	"Come down to the books at once if a wrist or an elbow hurts.")
guide("crow_books", {
	"Stack books about 12 cm high where your feet will go, the pillow in front of your hands.",
	"Start in crow prep with your toes on the books: hands flat, shins high on your upper arms.",
	"Lean forward until your toes grow light on the books.",
	"Lift one foot, then the other, and hold with your eyes ahead of your hands.",
	"Breathe slowly, then set your feet back on the books at the end of the hold.",
}, {
	"Lift off the books, never push off them: they only start you higher.",
	"Squeeze your knees into your arms so they stay put.",
	"Look forward at the floor, not down at your hands.",
})
L["move.crow_books.mark"] = "Both feet off the books for 10 s, knees on the upper arms."
move("crow", "Crow", "A pillow on the floor in front of your hands in case you tip forward.",
	"Knees on upper arms, lean forward, lift both feet. Breathe slowly.",
	"Come down at once if a wrist or an elbow hurts.")
guide("crow", {
	"Start in crow prep: hands flat, shins high on your upper arms, heels up.",
	"Lean forward until your toes grow light.",
	"Lift one foot, then the other, and draw your heels towards your seat.",
	"Hold with your eyes ahead of your hands and breathe slowly.",
	"Lower your feet to the floor slowly at the end of the hold.",
}, {
	"Lift one foot at a time until both come up together.",
	"Press the floor away and round your upper back a little.",
	"Look forward at the floor, not down at your hands.",
})
L["move.crow.mark"] = "Both feet off the floor for 20 s, knees on the upper arms."
move("headstand", "Headstand", "Corner of two walls, a folded blanket, head a hand from each wall.",
	"Forearms down, legs straight up, heels off the walls. Breathe slowly.",
	"Come down at once if your neck hurts.")
guide("headstand", {
	"Kneel facing the corner and set your forearms on the blanket, fingers laced.",
	"Set the top of your head on the blanket, the back of it in your hands.",
	"Lift your knees and walk your feet in until your hips are over your shoulders.",
	"Draw one knee in, then the other, then straighten both legs up and breathe slowly.",
	"Bend your knees, lower one foot, then the other, and rest in child's pose.",
}, {
	"Press your forearms down so your neck takes less of your weight.",
	"Use the walls to steady your heels, then bring them off when you can.",
	"Keep your shoulders lifted away from your ears the whole time.",
})
move("side_crow", "Side crow", "A pillow on the floor in front of and beside your hands in case you tip.",
	"Turn, knees stacked on one arm, lean forward, lift. Breathe slowly.",
	"Come down at once if a wrist or an elbow hurts.")
guide("side_crow", {
	"Squat on your toes with your feet and knees together.",
	"Turn your knees to one side and put your hands flat on the floor on that side.",
	"Bend your elbows and rest the side of your knees on one upper arm.",
	"Lean forward, lift both feet off the floor and breathe slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Squeeze your knees together so they stay stacked.",
	"Lift one foot at a time, or rest the lower foot on a book, until both lift.",
	"Look forward at the floor, not down at your hands.",
})
L["move.side_crow.mark"] = "Both feet in the air (no book under them), knees stacked on one arm, for the whole hold."
move("flying_splits_prep", "Flying splits, back toes down",
	"A pillow on the floor in front of your hands in case you tip forward.",
	"Front leg over the arm, lean, lift the front foot, back toes down. Breathe slowly.",
	"Come down at once if a wrist, an elbow or a hip hurts.")
guide("flying_splits_prep", {
	"Start in a lunge with your hands flat on the floor inside the front foot.",
	"Slide the front shoulder under the front thigh, close to the knee.",
	"Bend your elbows, lean forward and walk the front foot out until it lifts, leg straight.",
	"Keep the back toes on the floor and the back leg straight, and breathe slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Lean forward only as far as you can press back up.",
	"Reach out through the front heel so the front knee stays straight.",
	"Keep the back knee off the floor and the back toes light.",
})
L["move.flying_splits_prep.mark"] = "Front leg straight and lifted, back toes down, back leg straight, "
	.. "for the whole hold."
move("flying_splits", "Flying splits", "A pillow on the floor in front of your hands in case you tip forward.",
	"Front leg over the arm, back leg long, lean and lift. Breathe slowly.",
	"Come down at once if a wrist, an elbow or a hip hurts.")
guide("flying_splits", {
	"Start in a lunge with your hands flat on the floor inside the front foot.",
	"Slide the front shoulder under the front thigh, close to the knee.",
	"Bend your elbows, lean forward and walk the front foot out until that leg is straight.",
	"Lift the back leg straight off the floor, both feet up, and breathe slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Lean forward only as far as you can press back up.",
	"Lift the front foot first and keep the back toes down until it is steady.",
	"Reach back through the back heel and keep the front knee straight.",
})
L["move.flying_splits.mark"] = "Front leg straight over the arm, back leg straight, both off the floor for 10 s."
-- Slice 47 (2026-10-19): the same level beside each step's main pose, in the library's order
move("reverse_wrist_lean", "Reverse wrist lean at the desk",
	"A steady desk, palms flat on it, fingers pointing back at you.",
	"Arms straight, lean back until your forearms stretch. Breathe slowly.",
	"Ease off at a strong pull or if a wrist or an elbow hurts.")
guide("reverse_wrist_lean", {
	"Stand close to the desk and put your palms flat on it, shoulder-width apart.",
	"Turn your hands so your fingers point back at you, thumbs out to the sides.",
	"Keep your arms straight and move your hips back, your shoulders behind your wrists.",
	"Keep the heels of your hands pressed down and breathe slowly.",
	"Ease forward again at the end of the hold.",
}, {
	"Lean back only as far as both palms stay flat.",
	"Keep your elbows straight but soft, never pushed hard.",
	"Lean a little further over the days, only as far as your wrists stay easy.",
})
L["move.reverse_wrist_lean.mark"] = "Fingers pointing at you, palms flat, arms straight, "
	.. "shoulders a fist behind the wrists."
move("side_wrist_lean", "Side wrist lean at the desk",
	"A steady desk, palms flat on it, fingers turned out to the sides.",
	"Arms straight, shift your shoulders out over one hand. Breathe slowly.",
	"Ease off at a strong pull or if a wrist hurts.")
guide("side_wrist_lean", {
	"Stand a step back and put your palms flat on the desk, a bit wider than your shoulders.",
	"Turn your hands so your fingers point out to the sides.",
	"Keep your arms straight and shift your shoulders sideways past one wrist.",
	"Hold the lean with both palms flat and breathe slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Press the heel of the hand you lean over into the desk.",
	"Move your hips and shoulders together, as one.",
})
L["move.side_wrist_lean.mark"] = "Fingers pointing out, palms flat, arms straight, "
	.. "shoulders a fist past the leaning wrist."
move("fingertip_lean", "Fingertip lean at the desk", "A steady desk, fingertips and thumbs on it, palms lifted off.",
	"Press your fingertips down, palms up, lean in. Breathe slowly.",
	"Ease off at once if a finger or a wrist hurts.")
guide("fingertip_lean", {
	"Stand a step back from the desk and spread your fingers on it, shoulder-width apart.",
	"Press into your fingertips and thumbs until your palms lift off the desk.",
	"Keep your arms straight and lean in until your shoulders are over your fingers.",
	"Hold the tent shape with your fingers and breathe slowly.",
	"Lower your palms and ease back at the end of the hold.",
}, {
	"Spread your fingers wide so all five share the weight.",
	"If your palms sink, lean less and lift them again.",
})
L["move.fingertip_lean.mark"] = "Only fingertips and thumbs down, palms lifted, arms straight, "
	.. "shoulders over the fingers."
move("wrist_lean_press", "Wrist lean, press and sink",
	"A steady desk, palms flat on it, fingers pointing away from you.",
	"Lean in, gently press your fingers down for 5 s, relax and lean on. Breathe out.",
	"Ease off at a strong pull or if a wrist hurts.")
guide("wrist_lean_press", {
	"Set up as for the wrist lean: palms flat, fingers forward, arms straight.",
	"Lean until your shoulders pass your wrists and you feel a pull in your forearms.",
	"Gently press your fingers into the desk at about half effort for 5 s.",
	"Relax, breathe out and lean a little further.",
	"Do three rounds in the hold, breathing all the time.",
}, {
	"Press at half effort only: a gentle push, never a strain.",
	"Keep breathing while you press, never hold your breath.",
	"Keep both palms flat as you lean further.",
})
L["move.wrist_lean_press.mark"] = "After three presses: palms flat, fingers forward, shoulders a fist past the wrists."
move("desk_planche_lean", "Planche lean at the desk",
	"A steady desk, hands on it turned out a little, feet back on your toes.",
	"Body straight, push the desk away, lean your shoulders past your hands. Breathe slowly.",
	"Ease off if a wrist or an elbow hurts or the desk moves.")
guide("desk_planche_lean", {
	"Put your hands flat on the desk, shoulder-width apart, fingers turned out a little.",
	"Walk your feet back until your body is one straight line, up on your toes.",
	"Push the desk away so your upper back rounds a little.",
	"Lean forward until your shoulders are over your fingertips, arms straight.",
	"Breathe slowly in the lean, then shift back over your feet.",
}, {
	"Keep your hips in line with your body, never sagging or piked.",
	"Check that the desk stays put before you lean your weight on it.",
	"Lean a little further over the days, only as far as your wrists stay easy.",
})
L["move.desk_planche_lean.mark"] = "Body straight on tiptoes, arms straight, upper back round, "
	.. "shoulders over the fingertips."
move("desk_planche_press", "Planche lean at the desk, press and sink",
	"A steady desk, hands on it turned out a little, feet back on your toes.",
	"Lean, gently press your fingers down for 5 s, relax and lean on. Breathe out.",
	"Ease off if a wrist or an elbow hurts or the desk moves.")
guide("desk_planche_press", {
	"Put your hands flat on the desk, fingers turned out a little, body straight on your toes.",
	"Lean your shoulders past your wrists until your forearms pull.",
	"Gently press your fingers into the desk at about half effort for 5 s.",
	"Relax, breathe out and lean a little further, arms straight.",
	"Do three rounds in the hold, breathing all the time.",
}, {
	"Press at half effort only: a gentle push, never a strain.",
	"Keep your body one straight line from your heels to your head.",
	"Keep breathing while you press, never hold your breath.",
})
L["move.desk_planche_press.mark"] = "After three presses: body straight, arms straight, shoulders over the fingertips."
move("floor_wrist_lean", "Wrist lean on all fours", "Knees on a mat or a folded towel, palms flat, fingers forward.",
	"Arms straight, lean your shoulders forward past your hands. Breathe slowly.",
	"Ease off at a strong pull or if a wrist hurts.")
guide("floor_wrist_lean", {
	"Kneel on all fours, hands under your shoulders, fingers spread and pointing forward.",
	"Keep your arms straight and your back flat.",
	"Lean forward until your shoulders pass over your fingertips.",
	"Breathe slowly in the lean, then rock back over your knees.",
}, {
	"Press the whole hand down, most of all the base of the first finger.",
	"Lean a little further over the days, only as far as your wrists stay easy.",
})
L["move.floor_wrist_lean.mark"] = "Knees down, palms flat, fingers forward, arms straight, "
	.. "shoulders over the fingertips."
move("floor_reverse_lean", "Reverse wrist lean on all fours",
	"Knees on a mat or a folded towel, palms flat, fingers pointing at your knees.",
	"Arms straight, sit back until your forearms stretch. Breathe slowly.",
	"Ease off at a strong pull or if a wrist or an elbow hurts.")
guide("floor_reverse_lean", {
	"Kneel on all fours with your hands under your shoulders.",
	"Turn your hands so your fingers point back at your knees, palms flat.",
	"Keep your arms straight and sit your hips back toward your heels.",
	"Stop when your shoulders are a fist behind your wrists, and breathe slowly.",
	"Rock forward again at the end of the hold.",
}, {
	"Sit back only as far as both palms stay flat.",
	"Keep your elbows straight but soft, never pushed hard.",
})
L["move.floor_reverse_lean.mark"] = "Fingers toward your knees, palms flat, arms straight, "
	.. "shoulders a fist behind the wrists."
move("pendant_prep", "Pendant prep on books", "Two stacks of books beside your knees, about 10 cm high, on a mat.",
	"Shins crossed, press the books away, lift your seat and knees. Breathe slowly.",
	"Come down at once if a wrist or a knee hurts.")
guide("pendant_prep", {
	"Kneel and sit on your heels, then cross your shins, the tops of your feet on the floor.",
	"Put your hands flat on the books beside your thighs.",
	"Round your back and draw your knees toward your chest.",
	"Press the books away and lift your seat and knees, the feet still down.",
	"Breathe slowly in the lift, then lower slowly at the end of the hold.",
}, {
	"Push down through your hands, never lean back onto your feet.",
	"Draw your belly in to help your knees come up.",
})
L["move.pendant_prep.mark"] = "Hands on the books, shins crossed, seat and knees off the floor, back round."
move("crane", "Crane, straight arms", "A pillow on the floor in front of your hands in case you tip forward.",
	"Knees high on your arms, lift both feet, press your arms straight. Breathe slowly.",
	"Come down at once if a wrist or an elbow hurts.")
guide("crane", {
	"Start in the crow: knees high on your upper arms, both feet off the floor.",
	"Press the floor away and slowly straighten your arms.",
	"Lift your hips higher as your arms straighten, knees pressing into your arms.",
	"Hold with your eyes ahead of your hands and breathe slowly.",
	"Bend your arms and lower your feet slowly at the end of the hold.",
}, {
	"Squeeze your knees into your arms so they stay high.",
	"Round your upper back as you press up.",
	"Look forward at the floor, not down at your hands.",
})
L["move.crane.mark"] = "Knees high on the upper arms, arms straight, both feet off the floor for 10 s."
move("pendant", "Pendant on books", "Two stacks of books beside your knees, about 10 cm high, a pillow in front.",
	"Shins crossed, press the books away, lift your whole body. Breathe slowly.",
	"Come down at once if a wrist, an elbow or a knee hurts.")
guide("pendant", {
	"Start in the pendant prep: shins crossed, hands on the books, seat and knees up.",
	"Draw your heels toward your seat and round your back more.",
	"Press the books away until your feet lift too.",
	"Hold your whole body off the floor and breathe slowly.",
	"Lower your feet and knees slowly at the end of the hold.",
}, {
	"Lift one foot first if both will not come up yet.",
	"Draw your belly in and keep pressing down through your hands.",
})
L["move.pendant.mark"] = "Hands on the books, shins crossed, knees and feet off the floor for 10 s."
move("peacock", "Peacock", "A pillow on the floor in front of your face in case you tip forward.",
	"Elbows into your belly, lean forward, lift your legs straight. Breathe slowly.",
	"Come down at once if a wrist or an elbow hurts.")
guide("peacock", {
	"Kneel and put your hands flat on the floor, fingers pointing back at your knees.",
	"Bend your elbows and set them into your belly, just below your ribs.",
	"Stretch your legs back, toes down, so your body is one straight line.",
	"Lean forward over your hands until your feet float and your body is level.",
	"Breathe slowly in the hold, then lower your toes slowly.",
}, {
	"Keep your hands close together so your elbows stay under your belly.",
	"Lift one leg first if both will not come up yet.",
	"Look forward at the floor with your chin level.",
})
L["move.peacock.mark"] = "Elbows in the belly, fingers back, body level and both feet off the floor for 10 s."
move("side_crow_one_leg", "Side crow, one leg straight",
	"A pillow on the floor in front of and beside your hands in case you tip.",
	"Side crow, then stretch your top leg out straight to the side. Breathe slowly.",
	"Come down at once if a wrist or an elbow hurts.")
guide("side_crow_one_leg", {
	"Start in the side crow: knees stacked on one upper arm, both feet up.",
	"Keep the bottom knee on your arm and lean a little further forward.",
	"Slowly straighten your top leg out to the side, level with your hip.",
	"Hold with the leg straight and the foot flexed, and breathe slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Lean forward as the leg straightens so you stay balanced.",
	"Bend the leg back in first if you start to tip.",
})
L["move.side_crow_one_leg.mark"] = "Bottom knee on the arm, top leg straight to the side, both feet up for 10 s."
move("flying_pigeon_prep", "Flying pigeon, back foot on books",
	"A stack of books 12 cm high behind you, a pillow in front of your hands.",
	"Shin across your arms, foot hooked, back foot on the books. Breathe slowly.",
	"Come down at once if a wrist, a hip or a knee hurts.")
guide("flying_pigeon_prep", {
	"Stand on one leg and cross the other ankle over that knee, a figure four.",
	"Fold forward and put your hands flat on the floor, shoulder-width apart.",
	"Set the crossed shin across both upper arms and hook its foot round the arm.",
	"Bend your elbows, lean forward and rest the back foot on the books.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Keep the hooked foot flexed so it holds on to the arm.",
	"Lean forward only as far as you can press back up.",
})
L["move.flying_pigeon_prep.mark"] = "Shin across both upper arms, foot hooked, back foot on the books, hips level."
move("side_crane", "Side crane, straight arms",
	"A pillow on the floor in front of and beside your hands in case you tip.",
	"Knees stacked on one arm, lift both feet, press your arms straight. Breathe slowly.",
	"Come down at once if a wrist or an elbow hurts.")
guide("side_crane", {
	"Start in the side crow: knees stacked on one upper arm, both feet in the air.",
	"Press the floor away and slowly straighten your arms.",
	"Keep your knees together and your feet level with your knees.",
	"Hold with your eyes ahead of your hands and breathe slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Squeeze your knees together so they stay stacked.",
	"Bend your elbows again before you lower your feet.",
})
L["move.side_crane.mark"] = "Knees stacked on one upper arm, arms straight, both feet off the floor for 10 s."
move("firefly", "Firefly", "A pillow on the floor behind you in case you tip back.",
	"Legs over your arms, press up, straighten your legs. Breathe slowly.",
	"Come down at once if a wrist, an elbow or a hip hurts.")
guide("firefly", {
	"Squat, feet wide, shoulders behind your knees, hands flat behind your heels.",
	"Sit your thighs high on your upper arms and lift your feet.",
	"Press the floor away and straighten your arms as far as they go.",
	"Slowly straighten your legs forward and up, and breathe slowly.",
	"Bend your knees and lower your feet slowly at the end of the hold.",
}, {
	"Press the base of your first finger into the floor.",
	"Reach out through your heels to keep your knees straight.",
})
L["move.firefly.mark"] = "Legs straight over the arms, both feet off the floor, arms straight for 10 s."
move("side_crow_straight", "Side crow, straight legs",
	"A pillow on the floor in front of and beside your hands in case you tip.",
	"Hip on one arm, legs out straight to the side, lean and lift. Breathe slowly.",
	"Come down at once if a wrist or an elbow hurts.")
guide("side_crow_straight", {
	"Start in the side crow: knees stacked on one upper arm, both feet up.",
	"Shift so the outer hip rests on that upper arm.",
	"Lean forward and slowly straighten both legs out to the side.",
	"Hold with your legs stacked, level with your hips, and breathe slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Lean forward as your legs straighten so you stay balanced.",
	"Keep your legs straight and stacked, feet flexed.",
})
L["move.side_crow_straight.mark"] = "Hip on one arm, both legs straight to the side and off the floor for 10 s."
move("flying_pigeon", "Flying pigeon", "A pillow on the floor in front of your hands in case you tip forward.",
	"Shin across your arms, lean forward, lift and straighten the back leg. Breathe slowly.",
	"Come down at once if a wrist, a hip or a knee hurts.")
guide("flying_pigeon", {
	"Start in the flying pigeon prep: shin hooked across both arms, back foot down.",
	"Bend your elbows and lean forward until the back foot grows light.",
	"Lift the back foot and straighten that leg behind you.",
	"Hold with your hips level and your eyes ahead, and breathe slowly.",
	"The clock times each side, then counts you in to switch.",
}, {
	"Keep the hooked foot flexed so it holds on to the arm.",
	"Lift the back leg only as high as your hips stay level.",
})
L["move.flying_pigeon.mark"] = "Shin across both upper arms, back leg straight and off the floor for 10 s."

-- Slice 22: the new strength families and the orange tops (docs/plan.md slice 22), in the library's order. A setup
-- line names the one thing that must be true before Start; no line says "pain" (the stop lines say "hurts").

-- Push: the one-arm push-up's other forms
move("one_arm_negative", "One-arm lowering",
	"On the floor, feet wide; one hand under your chest, the other behind your back.",
	"Lower on one arm for 3 seconds, push up on both. Count out loud, breathe steadily.",
	"Stop when you drop fast or your hips twist. Count the weaker side.")
move("one_arm_pushup_hip", "One-arm push-up, feet hip-width",
	"On the floor, feet hip-width; one hand under your chest, the other behind your back.",
	"Lower and push up on one arm, hips level. Count out loud, breathe out as you push.",
	"Stop when your hips twist or sag. Count the weaker side.")
move("one_arm_pushup_close", "One-arm push-up, feet together",
	"On the floor, feet together; one hand under your chest, the other behind your back.",
	"Lower and push up on one arm, hips level. Count out loud, breathe out as you push.",
	"Stop when your hips twist or sag. Count the weaker side.")
-- Squat: past the pistol
move("shrimp_squat", "Shrimp squat", "A cushion behind you for the back knee; the desk within reach.",
	"Back foot up behind you, sit until the knee taps the cushion. Count out loud, breathe out.",
	"Stop when the knee drops fast or caves in. Count the weaker side.")
move("pistol_pause", "Pistol squat with a pause", "Clear space; the desk within reach to catch yourself.",
	"Sit all the way down on one leg, wait 2 seconds, stand. Count out loud, breathe steadily.",
	"Stop when your heel lifts or your knee caves in. Count the weaker side.")
move("shrimp_squat_held", "Shrimp squat, foot held",
	"A cushion behind you; one hand holds the back foot behind your seat.",
	"Foot held behind you, sit until the knee taps the cushion. Count out loud, breathe out.",
	"Stop when you tip forward or the knee drops fast. Count the weaker side.")
move("dragon_pistol", "Dragon pistol", "Clear space; the desk within reach to catch yourself.",
	"Free leg crossed behind the standing leg, sit low, stand. Count out loud, breathe out up.",
	"Stop when your knee caves in or your hips twist. Count the weaker side.")
move("dragon_pistol_assisted", "Dragon pistol, hand on the desk", "Face the desk, one hand on its edge.",
	"Free leg crossed behind, a hand on the desk, sit low, stand. Count out loud, breathe out.",
	"Stop when your knee caves in or your hips twist. Count the weaker side.")
-- Hinge: the full Nordic
move("nordic_full", "Full Nordic curl", "Heels under a heavy sofa, never a chair; hands ready to catch.",
	"Lower slowly as one piece, pull back up with no push. Count out loud, breathe steadily.",
	"Stop when you drop fast or bend at the hips.")
-- Cardio's top
move("burpee_tuck", "Tuck-jump burpees",
	"A body length of clear floor behind you, shoes on, a floor that does not slip.",
	"Burpee, then jump and pull your knees up high. Breathe out on every jump.")
-- The pull-up's purple and orange
move("lsit_pullup", "L-sit pull-up", "A bar fixed firmly; a stool to step down on.",
	"Legs straight out and level, chin over the bar. Count out loud, breathe out as you pull.",
	"Stop when your legs drop or you swing.")
move("chest_to_bar_pullup", "Chest-to-bar pull-up", "A bar fixed firmly; a stool to step down on.",
	"Pull until your chest touches the bar, lower fully. Count out loud, breathe out up.",
	"Stop when your chest no longer reaches the bar.")
move("uneven_towel_pullup", "Uneven towel pull-up",
	"A bar fixed firmly; a hand towel hung over it beside one hand. A stool to step down on.",
	"Pull with the bar hand; the towel hand only helps. Count out loud, breathe out up.",
	"Stop when the towel hand does most of the pull or you twist. Count the weaker side.")
move("archer_pullup", "Archer pull-up", "A bar fixed firmly, hands wide; a stool to step down on.",
	"Pull to one hand, the other arm straight along the bar. Count out loud, breathe out up.",
	"Stop when the straight arm bends or you twist. Count the weaker side.")
move("one_arm_lockoff", "One-arm lock-off", "A bar fixed firmly; a stool high enough to get your chin over the bar.",
	"Chin over the bar on one arm, elbow tight, legs still. Count out loud, breathe.",
	"Stop when your chin sinks below the bar or you start to spin. Count the weaker side.")
move("one_arm_pullup_assisted", "One-arm pull-up, towel at chest height",
	"A bar fixed firmly; a bath towel hung over it beside one hand. A stool below.",
	"Pull up on the bar arm; the towel hand only steadies. Count out loud, breathe out up.",
	"Stop when you cannot lower slowly: step down. Count the weaker side.")
move("one_arm_pullup_negative", "One-arm lowering", "A bar fixed firmly; one hand on the bar. A stool to step up on.",
	"Chin over the bar on one arm, lower in 5 s or more. Count out loud, breathe out down.",
	"Stop when you cannot lower for 5 seconds: step down. Count the weaker side.")
move("one_arm_pullup", "One-arm pull-up", "A bar fixed firmly; one hand on the bar. A stool to step down on.",
	"Chin over the bar on one arm, lower slowly all the way. Count out loud, breathe out up.",
	"Stop when you cannot lower slowly: step down. Count the weaker side.")
-- The row to the front lever
move("tuck_lean_hang", "Tuck lean-back hang", "A fixed bar or rings; a stool to step down on.",
	"Arms straight, knees to chest, back halfway to level. Count out loud, breathe.",
	"Stop when your arms bend or your shoulders shrug.")
move("tuck_fl_row", "Tuck front-lever row", "A fixed bar or rings; a stool to step down on.",
	"Back level, knees tucked, hands to your lower chest. Count out loud, breathe out up.",
	"Stop when your hips drop or you swing.")
move("table_archer_row", "Archer row under a table",
	"A heavy four-legged table, hands wide; never glass, folding, a standing or monitor desk.",
	"Pull your chest to one hand, the other arm straight. Count out loud, breathe out up.",
	"Stop when your hips sag or the table moves. Count the weaker side.")
move("tuck_front_lever", "Tuck front lever", "A fixed bar or rings; a stool to step down on.",
	"Hang, tuck your knees, lift until your back is level. Count out loud, breathe steadily.",
	"Stop when your hips drop or your arms bend.")
move("adv_tuck_fl_row", "Flat-back front-lever row", "A fixed bar or rings; a stool to step down on.",
	"Back flat and level, row the bar to your lower chest. Count out loud, breathe out up.",
	"Stop when your hips drop or your back rounds.")
move("adv_tuck_front_lever", "Front lever, back flat", "A fixed bar or rings; a stool to step down on.",
	"Hang, back flat and level, knees bent over your hips. Count out loud, breathe steadily.",
	"Stop when your hips drop or your back rounds.")
move("one_leg_front_lever", "Front lever, one leg out", "A fixed bar or rings; a stool to step down on.",
	"Back level, one leg long, the other knee tucked. Count out loud, breathe steadily.",
	"Stop when your hips drop or your arms bend. Count the weaker side.")
move("front_lever_half_lay", "Front lever, half lay", "A fixed bar or rings; a stool to step down on.",
	"Back level, hips open, both knees bent to a right angle. Count out loud, breathe steadily.",
	"Stop when your hips drop or your arms bend.")
move("front_lever", "Front lever", "A fixed bar or rings; a stool to step down on.",
	"Hang, lift your straight body level with the floor. Count out loud, breathe steadily.",
	"Stop when your hips drop or your arms bend.")
-- The core's top: the dragon flag, and the standing rollout
move("dragon_flag_tuck", "Tuck dragon flag",
	"Lie by a heavy sofa or bed and grip its frame behind your head. Never a desk or chair.",
	"Knees tucked, lift hips and back off the floor, lower slowly. Count out loud, breathe.",
	"Stop when your lower back arches or your neck takes the weight.")
move("dragon_flag_adv_tuck", "Dragon flag, hips open",
	"Lie by a heavy sofa or bed and grip its frame behind your head. Never a desk or chair.",
	"Hips open, knees bent, lift onto your shoulders, lower slowly. Count out loud, breathe.",
	"Stop when your hips fold or your lower back arches.")
move("dragon_flag_negative", "Dragon flag lowering",
	"Lie by a heavy sofa or bed and grip its frame behind your head. Never a desk or chair.",
	"Tuck up, straighten your body, lower it straight for 3 seconds. Count out loud, breathe.",
	"Stop when your hips bend or your lower back arches.")
move("dragon_flag_straddle", "Straddle dragon flag",
	"Lie by a heavy sofa or bed and grip its frame behind your head. Never a desk or chair.",
	"Legs wide and straight, lower in 3 seconds, lift back up. Count out loud, breathe.",
	"Stop when your hips bend, your lower back arches or your neck takes the weight.")
move("dragon_flag", "Dragon flag",
	"Lie by a heavy sofa or bed and grip its frame behind your head. Never a desk or chair.",
	"Body straight from shoulders to toes, lower and raise it. Count out loud, breathe slowly.",
	"Stop when your hips bend or your lower back arches.")
move("rollout_standing", "Standing rollout",
	"A wheel, a body length of clear floor and a wall ahead as a stop.",
	"From standing, roll out to a flat body and roll back. Count out loud, breathe out back.",
	"Stop when your lower back sags.")
move("rollout_standing_negative", "Standing rollout, lowering",
	"A wheel, a body length of clear floor and a wall ahead as a stop.",
	"Roll out from standing to flat in 3 seconds, knees down, roll back. Count out loud.",
	"Stop when your lower back sags.")
move("rollout_standing_wall", "Standing rollout to the wall",
	"A wheel and a wall, your toes a full stretch from it, clear floor between.",
	"Tail tucked, roll to the wall, pull back to standing, knees straight. Count out loud.",
	"Stop when your lower back sags or you cannot pull back; set your knees down.")
-- The side: the Copenhagen's long lever, the star side plank, the flags
move("copenhagen_long", "Long-lever Copenhagen", "Top ankle on the edge of a bed or sofa, forearm on the floor.",
	"Forearm down, lift your hips, bottom leg hanging free. Count out loud.",
	"Stop when your hips drop or your inner thigh hurts. Count the weaker side.")
move("side_plank_star", "Star side plank", "Forearm and feet on the floor.",
	"Side plank, then lift the top leg and the top arm. Count out loud.",
	"Stop when your hips drop. Count the weaker side.")
move("copenhagen_raises", "Copenhagen raises", "Top ankle on the edge of a bed or sofa, forearm on the floor.",
	"Lower your hips to the floor and lift them level again. Count out loud, breathe out up.",
	"Stop when your hips roll back or your inner thigh hurts. Count the weaker side.")
move("clutch_flag", "Clutch flag",
	"A steel pole fixed in the ground outdoors that does not turn; soft ground under you.",
	"Low elbow wedged at your hip, top arm hugging the pole, body level. Count out loud.",
	"Come down when your hips drop or your elbow slips off your hip. Count the weaker side.")
move("vertical_flag", "Vertical flag",
	"A steel pole fixed in the ground outdoors that does not turn; soft ground under you.",
	"Low arm locked and pushing, high arm pulling, legs to the sky. Count out loud, breathe.",
	"Come down feet first when an arm bends or your legs tip over. Count the weaker side.")
move("flag_tuck", "Tuck flag", "A steel pole fixed in the ground outdoors that does not turn; soft ground under you.",
	"Low arm straight, high arm pulling, knees tucked, hips up level. Count out loud, breathe.",
	"Come down when your low arm bends or your hips drop. Count the weaker side.")
move("flag_one_leg", "One-leg flag",
	"A steel pole fixed in the ground outdoors that does not turn; soft ground under you.",
	"Low arm straight, one leg long, one knee tucked, body level. Count out loud, breathe.",
	"Come down when your low arm bends or your hips drop. Count the weaker side.")
move("straddle_flag", "Straddle flag",
	"A steel pole fixed in the ground outdoors that does not turn; soft ground under you.",
	"Low arm straight, high arm pulling, legs wide, body level. Count out loud, breathe.",
	"Come down when your low arm bends or your legs drop. Count the weaker side.")
move("human_flag", "Human flag",
	"A steel pole fixed in the ground outdoors that does not turn; soft ground under you.",
	"Low arm straight, high arm pulling, whole body level and long. Count out loud, breathe.",
	"Come down when your low arm bends or your legs drop. Count the weaker side.")
-- The handstand
move("pike_pushup_feet_up", "Feet-up pike push-up",
	"Feet on a bed or a low sofa, never on a chair with wheels.",
	"Hips high over your hands, head toward the floor, push up. Count out loud, breathe out up.",
	"Stop when your elbows flare.")
move("wall_handstand", "Wall handstand",
	"A clear 2 by 2 m floor and a wall, never beside the desk. Warm your wrists first.",
	"Walk your feet up the wall, chest to the wall, arms straight. Count out loud, breathe.",
	"Come down when your arms shake or your back arches.")
move("wall_hspu", "Wall handstand push-up",
	"A clear 2 by 2 m floor, a wall, a folded towel under your head. Never beside the desk.",
	"From the wall handstand, head to the towel, push up. Count out loud, breathe out up.",
	"Come down when you cannot lower slowly or your elbows flare.")
move("handstand", "Freestanding handstand",
	"A body length of clear floor on every side, never beside the desk. Warm your wrists.",
	"Kick up, arms straight, ribs in, balance with your fingers. Count out loud, breathe.",
	"Come down when you tip: step one hand forward and turn out.")
move("handstand_pushup", "Handstand push-up",
	"A body length of clear floor on every side and a folded towel under your head.",
	"From a handstand, head to the towel, push up to straight arms. Count out loud, breathe.",
	"Come down when you cannot lower slowly or you tip.")
-- The handstand's moves in between that the climb waits for (2026-10-10)
move("wall_hspu_lowering", "Wall handstand push-up, lowering only",
	"A clear 2 by 2 m floor, a wall, a folded towel under your head. Never beside the desk.",
	"From the wall handstand, lower to the towel in 3 to 5 s. Count out loud, breathe.",
	"Come down when you cannot lower slowly or your elbows flare.")
move("wall_handstand_off", "Wall handstand, toes off",
	"A clear 2 by 2 m floor and a wall, never beside the desk. Warm your wrists first.",
	"Chest to the wall, drift until your toes leave it, balance. Count out loud, breathe.",
	"Come down when your back arches; if you tip away, step one hand forward and turn out.")
move("hspu_lowering", "Handstand push-up, lowering only",
	"A folded towel, your hands a forearm's length from a wall you face. Never by the desk.",
	"From a handstand, lower to the towel in 3 s, then come down. Count out loud, breathe.",
	"Come down when you cannot lower slowly or you tip.")
-- The handstand's ladder gaps (2026-10-12)
move("wall_l_hold", "L-hold at the wall",
	"A clear 2 by 2 m floor and a bare wall, never beside the desk. Warm your wrists first.",
	"Feet on the wall at hip height, hips over your hands, arms by your ears. Count out loud.",
	"Come down when your arms bend or your shoulders sink.")
move("wall_kickup_turnout", "Kick-up to the wall, turn out",
	"A clear 2 by 2 m floor by a bare wall, never beside the desk. Warm your wrists first.",
	"Kick up softly, both heels to the wall, hold for three, turn out. Count out loud.",
	"Stop when your heels bang the wall or your arms bend.")
-- The L-sit, on the floor (2026-10-08)
move("lsit_heels", "Foot-supported L-sit", "A clear floor; palms flat beside your hips. Warm your wrists first.",
	"Push the floor away, lift your hips, heels stay down. Count out loud.",
	"Stop when your hips sink or your shoulders creep up to your ears.")
move("lsit_one_heel", "L-sit, one heel down", "A clear floor; palms flat beside your hips. Warm your wrists first.",
	"Lift your hips, then one straight leg; the other heel stays down. Count out loud.",
	"Stop when the raised leg drops or your hips sink. Count the weaker side.")
move("lsit_tuck_floor", "Tuck L-sit", "A clear floor; palms flat beside your hips. Warm your wrists first.",
	"Push the floor away, lift your hips, knees tucked, both feet off. Count out loud.",
	"Stop when a foot touches down or your shoulders creep up.")
move("lsit_one_leg_floor", "One-leg L-sit", "A clear floor; palms flat beside your hips. Warm your wrists first.",
	"Tuck L-sit, then push one leg out straight and level. Count out loud.",
	"Stop when the straight leg drops or a foot lands. Count the weaker side.")
move("lsit_tuck_one_foot", "Tuck L-sit, one foot down",
	"A clear floor; palms flat beside your hips. Warm your wrists first.",
	"Lift your hips, pull one knee in, the other foot on its toes. Count out loud.",
	"Stop when the lifted foot lands or your shoulders creep up. Count the weaker side.")
move("lsit_cross", "Cross-legged lift",
	"A clear floor; cross your shins, never a foot on your thigh. Warm your wrists first.",
	"Push the floor away, lift your hips and your crossed legs off the floor. Count out loud.",
	"Stop when a foot touches down or your shoulders creep up.")
-- The L-sit with the hands raised (parallettes or books)
move("lsit_foot_supported", "Foot-supported L-sit on books",
	"Parallettes or two equal stacks of hard books, never chairs on wheels, on a firm floor.",
	"Push down, lift your hips, heels stay down. Count out loud.",
	"Stop when your shoulders creep up to your ears.")
move("lsit_tuck", "Tuck L-sit on books",
	"Parallettes or two equal stacks of hard books, never chairs on wheels, on a firm floor.",
	"Push down, lift your hips, knees tucked, both feet off the floor. Count out loud.",
	"Stop when your feet touch down or your shoulders creep up.")
move("lsit_one_leg", "One-leg L-sit on books",
	"Parallettes or two equal stacks of hard books, never chairs on wheels, on a firm floor.",
	"Tuck L-sit on books, then stretch one leg straight out in front. Count out loud.",
	"Stop when the straight leg drops or your shoulders creep up. Count the weaker side.")
move("lsit_books", "L-sit on books",
	"Parallettes or two equal stacks of hard books, never chairs on wheels, on a firm floor.",
	"Push down, lift your hips, both legs straight out and level. Count out loud.",
	"Stop when your legs drop or your knees bend.")
move("lsit_floor", "Floor L-sit", "A clear floor; palms flat beside your hips. Warm your wrists first.",
	"Push the floor away, lift your hips, legs straight and level. Count out loud.",
	"Stop when your heels touch down or your knees bend.")
move("seated_leg_lifts", "Seated leg lifts", "A clear floor; sit tall with your legs straight and together.",
	"Hands by your knees, lift both heels, hold one slow breath, lower. Count out loud.",
	"Stop when your knees bend or you lean back to lift.")
move("vsit_tuck", "Tuck V-sit", "A clear floor; palms flat beside your hips. Warm your wrists first.",
	"L-sit, then pull your knees to your chest and lift your hips higher. Count out loud.",
	"Stop when your feet drop or your arms bend.")
move("vsit_low", "Low V-sit", "A clear floor; palms flat beside your hips. Warm your wrists first.",
	"From the L-sit, lift your straight legs well above level. Count out loud.",
	"Stop when your legs drop to level or your arms bend.")
move("vsit_one_leg", "One-leg V-sit", "A clear floor; palms flat beside your hips. Warm your wrists first.",
	"Tuck V-sit, then one leg straight up to the ceiling, arms locked. Count out loud.",
	"Stop when the straight leg drops or your arms bend. Count the weaker side.")
move("vsit", "V-sit", "A clear floor; palms flat beside your hips. Warm your wrists first.",
	"Lift your straight legs up toward your face, hips ahead of your hands. Count out loud.",
	"Stop when your legs drop or your arms bend.")
-- The planche
move("planche_lean", "Planche lean",
	"On the floor, wrists warm. Lean until your shoulders are over your fingertips.",
	"From a push-up position, lean your shoulders over your fingertips. Count out loud.",
	"Stop when your elbows bend or your hips sag.")
move("frog_stand_one_foot", "Frog stand, one foot up",
	"A pillow on the floor in front of your hands in case you tip forward.",
	"Knees on your bent arms, lean until your toes grow light, lift one foot. Count out loud.",
	"Come down at once if a wrist or an elbow hurts. Count the weaker side.")
move("frog_stand_books", "Frog stand from books",
	"Toes on a stack of books 12 cm high, a pillow in front of your hands.",
	"Knees on your bent arms, lean until both feet float off the books. Count out loud.",
	"Come down to the books at once if a wrist or an elbow hurts.")
move("frog_stand", "Frog stand", "A pillow on the floor in front of your hands in case you tip forward.",
	"Knees on your bent arms, lean forward, lift both feet. Count out loud, breathe slowly.",
	"Come down at once if a wrist or an elbow hurts.")
move("pseudo_planche_pushup", "Planche-lean push-up",
	"On the floor, hands beside your waist, fingers out. Warm your wrists first.",
	"Shoulders ahead of your hands, lower and push up. Count out loud, breathe out as you push.",
	"Stop when your hips sag or your shoulders drift back.")
move("crane_stand", "Straight-arm frog stand",
	"A pillow on the floor in front of your hands in case you tip forward.",
	"Knees high on straight arms, lean forward, lift both feet. Count out loud, breathe slowly.",
	"Come down at once if a wrist or an elbow hurts.")
move("tuck_planche_assisted", "Tuck planche, toes on a book",
	"A stack of books behind your feet, a pillow in front of your hands. Warm your wrists.",
	"Arms straight, lean forward, knees to your chest, toes light on the books. Count out loud.",
	"Come down when your elbows bend or your toes take your weight.")
move("tuck_planche", "Tuck planche", "A pillow on the floor in front of your hands in case you tip forward.",
	"Arms straight, lean forward, knees to your chest, feet off the floor. Count out loud.",
	"Come down when your elbows bend or your hips drop.")
move("open_tuck_planche", "Tuck planche, hips open",
	"A pillow on the floor in front of your hands in case you tip forward.",
	"Tuck planche, then move your knees halfway back and flatten your back. Count out loud.",
	"Come down when your elbows bend or your hips drop.")
move("adv_tuck_planche", "Flat-back tuck planche",
	"A pillow on the floor in front of your hands in case you tip forward.",
	"Tuck planche, then push your hips back until your back is flat. Count out loud.",
	"Come down when your elbows bend or your hips drop.")
move("one_leg_planche", "Flat-back tuck planche, one leg back",
	"A pillow on the floor in front of your hands in case you tip forward.",
	"Flat-back tuck planche, then push one leg straight back, level. Count out loud.",
	"Come down when the straight leg sinks or your elbows bend. Count the weaker side.")
move("straddle_planche", "Straddle planche",
	"A pillow on the floor in front of your hands in case you tip forward.",
	"Arms straight, body level, legs straight and wide apart. Count out loud, breathe.",
	"Come down when your elbows bend or your hips pike up.")
-- Power
move("burpee_strict", "Strict burpee", "A body length of clear floor behind you, shoes on, a floor that does not slip.",
	"Jump back, chest to the floor, push up, jump in, jump. Count out loud, breathe in rhythm.",
	"Stop when your back sags.")
move("tuck_jump", "Tuck jump", "Clear space, shoes on, a floor that does not slip.",
	"Jump and pull your knees up to your chest, land softly. Count out loud, breathe out up.",
	"Stop when your landings get loud or your knees cave in.")
move("explosive_pushup", "Explosive push-up", "On a floor that does not slip.",
	"Push up fast, hands leave the floor, land on soft elbows. Count out loud, breathe out.",
	"Stop when your hips sag or you land on straight arms.")
move("clap_pushup", "Clap push-up", "On a floor that does not slip.",
	"Push up fast, clap once, land on soft elbows. Count out loud, breathe out as you push.",
	"Stop when your hips sag or you land on straight arms.")
move("knee_clap_pushup", "Clap push-up on the knees", "On a floor that does not slip, a folded towel under your knees.",
	"Push up fast from the knees, clap once, land on soft elbows. Count out loud, breathe out.",
	"Stop when your hips sag or you land on straight arms.")
move("explosive_pullup", "Explosive pull-up",
	"A fixed bar with room above it, never a doorway bar; a box to step down on.",
	"Pull fast until your chest reaches the bar, lower fully. Count out loud, breathe out up.",
	"Stop when you swing or your chest no longer reaches the bar.")
move("high_pullup", "High pull-up", "A fixed bar with room above it, never a doorway bar; a box to step down on.",
	"Pull fast until the bar is at your lower ribs, lower fully. Count out loud, breathe out.",
	"Stop when you swing or the bar stays at your chin.")
move("straight_bar_dip", "Straight-bar dip", "A fixed bar at chest height, never a doorway bar; a box to step up on.",
	"On top of the bar, arms locked, lower your chest to the bar and press up. Count out loud.",
	"Stop when you cannot press back to locked arms or the front of a shoulder pinches.")
move("straight_bar_dip_lowering", "Straight-bar dip, lowering only",
	"A fixed bar at chest height, never a doorway bar; a box to step up on.",
	"On top of the bar, lower to the bar in about 5 seconds, then step down. Count out loud.",
	"Stop when you drop instead of lowering or the front of a shoulder pinches.")
move("bar_support_hold", "Straight-bar support hold",
	"A fixed bar at chest height, never a doorway bar; a box to step up on.",
	"On top of the bar, arms locked, shoulders down. Count out loud, breathe slowly.",
	"Stop when your elbows bend, your shoulders shrug or the front of a shoulder pinches.")
move("muscle_up_jump", "Jumping muscle-up",
	"A fixed bar with room above it, never a doorway bar; a sturdy box under it.",
	"Jump from the box, pull, turn your elbows over the bar, press up. Count out loud, breathe.",
	"Stop when one elbow comes over first or you cannot lower slowly.")
move("muscle_up_negative", "Muscle-up lowering",
	"A fixed bar with room above it, never a doorway bar; a sturdy box under it.",
	"Jump to the top, then lower through the turn slowly, elbows together. Count out loud.",
	"Stop when you drop through the turn or one elbow turns under first.")
move("muscle_up", "Muscle-up", "A fixed bar with room above it, never a doorway bar; a box to step down on.",
	"Pull high, turn both elbows over the bar, press to straight arms. Count out loud, breathe.",
	"Stop when one elbow comes over first or you swing.")

-- Slice 22: the guides of the 57 new moves, in the order of the moves above.
guide("one_arm_negative", {
	"Start on your toes with both hands on the floor, then walk your feet out very wide.",
	"Put one hand under the middle of your chest and rest the other on your lower back.",
	"Bend the elbow and lower your chest to just above the floor over three slow beats.",
	"At the bottom set the free hand down, breathe out and push up on both arms.",
	"Each lowering is one. One arm, press Switch sides, the other; type the weaker arm's count.",
}, {
	"Squeeze your seat and brace your belly so your hips stay level with your shoulders.",
	"Keep the working elbow close to your side, pointing back towards your hip.",
	"Say the beats out loud so the last part of the way down stays slow.",
})
guide("one_arm_pushup_hip", {
	"Start in a push-up on the floor, feet hip-width apart, body straight from heels to head.",
	"Put one hand under the middle of your chest and rest the other on your lower back.",
	"Lower your chest towards the floor, hips and shoulders level.",
	"Breathe out and push the floor away until your arm is straight.",
	"Each push-up is one. One arm, press Switch sides, the other; type the weaker arm's count.",
}, {
	"Lean your weight a little over the working hand before you bend the elbow.",
	"Brace your belly and seat hard before each rep so your hips cannot turn open.",
	"Keep the elbow close to your ribs and grip the floor with spread fingers.",
})
guide("one_arm_pushup_close", {
	"Start in a push-up on the floor, feet together, body straight from heels to head.",
	"Put one hand under the middle of your chest and rest the other on your lower back.",
	"Lower your chest towards the floor, hips and shoulders level.",
	"Breathe out and push the floor away until your arm is straight.",
	"Each push-up is one. One arm, press Switch sides, the other; type the weaker arm's count.",
}, {
	"Lean your weight a little over the working hand before you bend the elbow.",
	"Brace your belly and seat hard before each rep so your hips cannot turn open.",
	"Keep the elbow close to your ribs and grip the floor with spread fingers.",
})
guide("shrimp_squat", {
	"Put the cushion just behind the heel you stand on, with the desk in reach at your side.",
	"Stand on one leg, bend the other knee so that foot lifts behind you, arms forward.",
	"Lean your chest forward and sit back until the back knee taps the cushion.",
	"Keep the back foot off the floor, breathe out and stand up on the front leg.",
	"Each stand is one. One leg, press Switch sides, the other; type the weaker leg's count.",
}, {
	"Reach your arms further forward as you sink; they balance the leg behind you.",
	"Keep the front heel flat and the front knee pointing over your toes.",
	"Touch the cushion as softly as you can, with no bounce off it.",
})
guide("pistol_pause", {
	"Stand in clear space with the desk just within reach at your side to catch yourself.",
	"Stand on one leg, reach both arms forward and lift the other leg straight out in front.",
	"Sit down on the standing leg all the way to the bottom, heel flat, taking about 2 s.",
	"Stay tight at the bottom for 2 s, free leg off the floor, then breathe out and stand.",
	"Each stand is one. One leg, press Switch sides, the other; type the weaker leg's count.",
}, {
	"Say the two beats out loud at the bottom, which keeps you breathing.",
	"Stay tight in the pause; do not relax and rest at the bottom.",
	"Reach your arms forward and keep your chest up so you do not roll back.",
})
guide("shrimp_squat_held", {
	"Put the cushion just behind the heel you stand on and stand on one leg in clear space.",
	"Bend the other knee and hold that foot behind your seat with the same side's hand.",
	"Lean your chest forward and sit back until the back knee taps the cushion.",
	"Breathe out and stand up on the front leg, the hand still on the foot, the other arm out.",
	"Each stand is one. One leg, press Switch sides, the other; type the weaker leg's count.",
}, {
	"Lean your chest well forward and reach the free arm ahead to balance.",
	"Let the front knee move forward over your toes while the heel stays flat.",
	"Pull the held heel towards your seat so the back knee points straight down.",
})
guide("dragon_pistol", {
	"Stand in clear space with the desk just within reach at your side to catch yourself.",
	"Stand on one leg and cross the free leg behind it, its foot reaching out the other side.",
	"Sit all the way down on the standing leg, the free leg straight and up.",
	"Keep the heel flat and your arms forward, then breathe out and stand back up.",
	"Each stand is one. One leg, press Switch sides, the other; type the weaker leg's count.",
}, {
	"Slide the free leg further across as you sink, so it never rests on the floor.",
	"Keep the standing knee over your toes as your hips turn.",
	"Stay tight at the bottom and drive straight up, with no bounce.",
})
guide("dragon_pistol_assisted", {
	"Face the desk a small step back, the fingertips of one hand resting on its edge.",
	"Stand on the leg on that side and cross the other leg behind it, the other arm forward.",
	"Sit all the way down on the standing leg, the hand only steadying you.",
	"Keep the heel flat, then breathe out and stand back up.",
	"Each stand is one. One leg, press Switch sides, the other; type the weaker leg's count.",
}, {
	"Slide the free leg further across as you sink, so it never rests on the floor.",
	"Keep the standing knee over your toes as your hips turn.",
	"Lean on the desk less each week; the dragon pistol takes it away.",
})
guide("nordic_full", {
	"Kneel on a folded towel, heels hooked under a heavy sofa that cannot tip or slide.",
	"Kneel tall in a straight line from knees to head, hands up in front of your chest.",
	"Lower forward for 3 s or more until your chest is just above the floor, body straight.",
	"Pull back up to kneeling tall with the backs of your thighs, hands off the floor.",
	"Breathe steadily. Each trip down and back up counts as one, counted out loud.",
}, {
	"Squeeze your seat and belly so the line from knees to head never folds.",
	"Pull your heels up hard into the sofa to start the way back.",
	"Keep your hands in front with bent elbows, ready to catch you if you drop.",
})
guide("burpee_tuck", {
	"Stand at the back of your clear space, feet hip-width apart, with clear space above you.",
	"Squat, put your hands flat on the floor and jump your feet back to a straight line.",
	"Jump your feet back in to your hands and stand up fast.",
	"Jump straight up, pull both knees up towards your chest and land softly.",
	"Breathe hard and freely and keep going, as fast as clean reps allow, until the clock ends.",
}, {
	"Land on the balls of your feet with soft knees, then let your heels come down.",
	"Brace your belly as your feet go back so your hips stay in line.",
	"Bring your knees up to your chest; do not fold your chest down to them.",
})
guide("lsit_pullup", {
	"Check the bar is fixed firmly and set a stool under it to step down on.",
	"Hang with palms facing away, then lift your straight legs until they are level.",
	"Hold your legs there and pull your chin over the bar, breathing out.",
	"Lower to straight arms with your legs still level.",
	"Each pull with level legs counts as one, counted out loud at the top.",
}, {
	"Point your toes and squeeze your thighs so your knees stay straight.",
	"Lean back a little as you pull, so your legs balance in front of the bar.",
	"Start each rep from a still hang with your legs up.",
})
guide("chest_to_bar_pullup", {
	"Check the bar is fixed firmly and set a stool under it to step down on.",
	"Grip with palms facing away, hands a little wider than your shoulders.",
	"From straight arms, pull until your upper chest touches the bar, breathing out.",
	"Lower all the way to straight arms, breathing in.",
	"Each touch of chest to bar counts as one, counted out loud.",
}, {
	"Lean back a little and lift your chest up to the bar as you pull.",
	"Drive your elbows down and back behind your ribs at the top.",
	"Keep your legs still: a kick or a swing does not count.",
})
guide("uneven_towel_pullup", {
	"Check the bar is fixed firmly and hang a towel over it a little out from one hand.",
	"Grip the bar with one hand, both towel ends with the other, that fist at forehead height.",
	"From straight arms, pull your chin over the bar, the bar hand doing most of the work.",
	"Lower to straight arms, breathing in.",
	"Each pull counts as one. Do every rep on one side, then press Switch sides.",
}, {
	"Hold the towel at the same place each time so your record stays fair.",
	"Keep your chest facing the bar; do not turn toward the bar hand.",
	"Squeeze your legs together and brace your belly so you do not swing.",
})
guide("archer_pullup", {
	"Check the bar is fixed firmly, set a stool under it and grip twice shoulder-width.",
	"From straight arms, pull your chin up towards one hand, breathing out.",
	"Let the other arm go straight along the bar as you rise; it only steadies you.",
	"Lower back to the middle until both arms are straight.",
	"Each pull is one. One side, press Switch sides, the other; type the weaker side's count.",
}, {
	"Let the straight arm's hand roll over the top of the bar as you go up.",
	"Keep your chest facing the bar; do not turn towards the pulling arm.",
	"Pull with the bent arm; the straight arm presses down only lightly.",
})
guide("one_arm_lockoff", {
	"Check the bar is fixed firmly and set a stool high enough to get your chin over the bar.",
	"Step up, grip with both hands, palms toward you, and bring your chin over the bar.",
	"Pull your shoulders down, then let go with one hand and lift your feet.",
	"Hold your chin over the bar, elbow tight to your ribs, breathing slowly.",
	"The clock counts up: step down when your chin sinks, then press Switch sides.",
}, {
	"Squeeze your legs together and brace your belly so your body does not spin.",
	"Keep the free hand near the bar, ready to grab it.",
	"Step down before your elbow opens; never drop into a hang on one arm.",
})
guide("one_arm_pullup_assisted", {
	"Check the bar is fixed firmly and hang a bath towel over it a little out from one hand.",
	"Grip the bar with one hand, both towel ends with the other, that fist at chest height.",
	"From a straight arm, pull your chin over the bar, the towel hand only steadying.",
	"Lower slowly all the way to a straight arm, never dropping into the bottom.",
	"Each pull counts as one. Do every rep on one side, then press Switch sides.",
}, {
	"Hold the towel at chest height each time so your record stays fair.",
	"Pull your shoulder down first, then bend the elbow.",
	"Step onto the stool the moment the way down speeds up.",
})
guide("one_arm_pullup_negative", {
	"Check the bar is fixed firmly and set a stool under it, high enough to get your chin over.",
	"Step up, grip with one hand, palm toward you, and bring your chin over the bar.",
	"Take the free hand away, lift your feet and lower for 5 s or more to a straight arm.",
	"Step back onto the stool to reset at the top, breathing out as you lower.",
	"Each lowering is one. One arm, press Switch sides, the other; type the weaker arm's count.",
}, {
	"Squeeze your legs together and brace your belly so your body does not spin.",
	"Keep the elbow close to your side and the shoulder pulled down.",
	"Step onto the stool the moment the way down speeds up; never drop into the bottom.",
})
guide("one_arm_pullup", {
	"Check the bar is fixed firmly and set a stool under it to step down on.",
	"Hang from one hand, palm facing you or to the side, the free hand off the bar and arm.",
	"Pull your shoulder down, then pull until your chin is over the bar, breathing out.",
	"Lower slowly to a straight arm, or step onto the stool if you cannot.",
	"Each pull is one. One arm, press Switch sides, the other; type the weaker arm's count.",
}, {
	"Squeeze your legs together and brace your belly so your body does not spin.",
	"Drive the elbow down towards your ribs, close to your side.",
	"Keep the shoulder set at the bottom; never hang loose from one arm.",
})
guide("tuck_lean_hang", {
	"Check the bar or rings are fixed firmly and set a stool below to step down on.",
	"Hang with straight arms, palms facing away, and pull your shoulders down.",
	"Tuck your knees up to your chest.",
	"Press the bar toward your hips with straight arms until your back is halfway to level.",
	"Hold there and breathe slowly; the clock counts until your arms bend or you sink back.",
}, {
	"Halfway: your back slants between hanging straight down and flat, not yet level.",
	"Keep your knees tight to your chest; a tight tuck is easier to hold.",
	"Think of pushing the bar down to your hips, not of pulling yourself up.",
})
guide("tuck_fl_row", {
	"Check the bar or rings are fixed firmly and set a stool below to step down on.",
	"Hang, tuck your knees up and lift your hips until your back is level.",
	"Bend your arms, pulling your hands toward your lower chest, back level, breathing out.",
	"Lower to straight arms and keep your hips as high as your shoulders.",
	"Each pull from straight arms counts as one, counted out loud.",
}, {
	"Keep your knees tucked in tight; a tight tuck is easier to hold level.",
	"Pull your shoulder blades back and down before you bend your arms.",
	"Start each rep from a still, level hang.",
})
guide("table_archer_row", {
	"Lean on the edge, then pull gently: if the table lifts, slides or creaks, stop.",
	"Lie under it and grip the edge with your hands wide, inside the table's legs.",
	"Heels on the floor, lift your hips so your body is straight from heels to head.",
	"Breathe out and pull your chest up to one hand, the other arm long, then lower.",
	"Each pull is one. One side, press Switch sides, the other; type the weaker side's count.",
}, {
	"Slide your chest sideways towards the pulling hand; do not turn your hips.",
	"Let the long arm rest on the edge; it only steadies you.",
	"Squeeze your seat so your hips rise with your chest.",
})
guide("tuck_front_lever", {
	"Check the bar or rings are fixed firmly and set a stool below to step down on.",
	"Hang with straight arms, palms facing away, and pull your shoulders down.",
	"Tuck your knees to your chest and lean back until your back is level with the floor.",
	"Keep your arms locked, press the bar down towards your hips and breathe slowly.",
	"The clock counts up: hold until your back tips out of level, then step down.",
}, {
	"Keep your head in line with your back, not lifted or tucked.",
	"Keep your knees tight to your chest; the hold gets harder as they move away.",
	"Think of pushing the bar down to your hips, not of pulling yourself up.",
})
guide("adv_tuck_fl_row", {
	"Check the bar or rings are fixed firmly and set a stool below to step down on.",
	"Hang and lean back until your back is flat and level, thighs straight up, knees bent.",
	"Bend your arms and pull your body up to the bar, back still flat, breathing out.",
	"Lower to straight arms with your back still flat and level.",
	"Each pull from straight arms counts as one, counted out loud.",
}, {
	"Keep your hips as high as your shoulders through the whole rep.",
	"Pull your shoulder blades back and down before your arms bend.",
	"Keep your knees over your hips; a tighter tuck is the easier move below this one.",
})
guide("adv_tuck_front_lever", {
	"Check the bar or rings are fixed firmly and set a stool below to step down on.",
	"Hang with straight arms, palms facing away, and pull your shoulders down.",
	"Tuck and lean back to a level back, then open your hips until your knees are over them.",
	"Keep your back flat and your arms locked, and breathe slowly.",
	"The clock counts up: hold until your back rounds or tips out of level, then step down.",
}, {
	"Keep your thighs pointing straight up; knees toward your chest is the easier tuck.",
	"Press the bar down towards your hips with locked elbows.",
	"Keep your head in line with your back, not lifted or tucked.",
})
guide("one_leg_front_lever", {
	"Check the bar or rings are fixed firmly and set a stool below to step down on.",
	"Hang with straight arms, palms facing away, and pull your shoulders down.",
	"Tuck and lean back to a level back, then push one leg out straight and level.",
	"Keep the other knee tucked to your chest, arms locked, and breathe slowly.",
	"The clock counts up: hold until your hips sink, then press Switch sides for the other leg.",
}, {
	"Squeeze the seat and point the toes of the long leg so it stays in line.",
	"Press the bar down towards your hips with locked elbows.",
	"Keep your head in line with your body, not tucked to look at your feet.",
})
guide("front_lever_half_lay", {
	"Check the bar or rings are fixed firmly and set a stool below to step down on.",
	"Hang with straight arms, palms facing away, and pull your shoulders down.",
	"Tuck and lean back to a level back, then open your hips until your thighs line up with it.",
	"Keep both knees bent to a right angle, arms locked, and breathe slowly.",
	"The clock counts up: hold until your hips sink, then tuck and step down.",
}, {
	"Squeeze your seat so your hips stay open, in line with your back.",
	"Press the bar down towards your hips with locked elbows.",
	"Keep your head in line with your body, not tucked to look at your feet.",
})
guide("front_lever", {
	"Check the bar or rings are fixed firmly and set a stool below to step down on.",
	"Hang with straight arms, palms facing away, and pull your shoulders down.",
	"Tuck and lean back to a level back, then push your legs out straight and together.",
	"Hold one line from shoulders to toes, level with the floor, and breathe slowly.",
	"The clock counts up: hold until the line breaks, then tuck and step down.",
}, {
	"Squeeze your seat and point your toes so your hips stay straight.",
	"Press the bar down towards your hips with locked elbows.",
	"Keep your head in line with your body, not tucked to look at your feet.",
})
guide("dragon_flag_tuck", {
	"Lie on your back, head near a heavy sofa or bed, and grip its frame behind your head.",
	"Tuck your knees to your chest and roll up until only your upper back is on the floor.",
	"Lower your back slowly to just above the floor, knees still tucked.",
	"Breathe out, pull on the frame and lift your hips and back up again.",
	"Each lift and slow lowering counts as one, counted out loud.",
}, {
	"Rest your weight on your upper back and shoulders, never on your head.",
	"Pull your ribs down and tuck your tail so your lower back never arches.",
	"Keep pulling on the frame with bent arms; it holds your shoulders down.",
})
guide("dragon_flag_adv_tuck", {
	"Lie on your back, head near a heavy sofa or bed, and grip its frame behind your head.",
	"Tuck your knees and roll up until only your upper back is on the floor.",
	"Open your hips into one line from shoulders to knees, the knees still bent at 90 degrees.",
	"Lower that line slowly, then breathe out and lift it back up.",
	"Each lowering and lift counts as one, counted out loud.",
}, {
	"Squeeze your seat so your hips stay open all the way down.",
	"Rest your weight on your upper back and shoulders, never on your head.",
	"Keep pulling on the frame; your arms hold your shoulders down as you lower.",
})
guide("dragon_flag_negative", {
	"Lie on your back, head near a heavy sofa or bed, and grip its frame behind your head.",
	"Tuck your knees and roll up until only your upper back is on the floor.",
	"Push your legs straight up so your body is one line from shoulders to toes.",
	"Breathe out and lower that line for 3 s or more until your heels are near the floor.",
	"Tuck up again for the next one. Each straight lowering counts as one, out loud.",
}, {
	"Squeeze your seat hard so your body stays in one line all the way down.",
	"Rest your weight on your upper back and shoulders, never on your head.",
	"Keep pulling on the frame; your arms hold your shoulders down as you lower.",
})
guide("dragon_flag_straddle", {
	"Lie on your back, head near a heavy sofa or bed, and grip its frame behind your head.",
	"Tuck up onto your upper back, push your legs straight up and open them wide.",
	"Lower that wide, straight body slowly until your heels are just above the floor.",
	"Breathe out, pull on the frame and lift it back up, legs still wide and straight.",
	"Each lowering and lift counts as one, counted out loud.",
}, {
	"Squeeze your seat so your hips stay open, and keep your legs as wide on every rep.",
	"Rest your weight on your upper back and shoulders, never on your head.",
	"Pull hard on the frame to start the lift; do not kick or fold to get up.",
})
L["move.dragon_flag_straddle.hint"] = "Open your legs only as wide as your hips allow, both legs straight."
guide("dragon_flag", {
	"Lie on your back, head near a heavy sofa or bed, and grip its frame behind your head.",
	"Tuck up onto your upper back, then push your legs straight up into one line.",
	"Lower the line slowly until your heels are just above the floor.",
	"Breathe out and lift it back up, still in one line from shoulders to toes.",
	"Each lowering and lift counts as one, counted out loud.",
}, {
	"Squeeze your seat and legs so the line stays stiff.",
	"Rest your weight on your upper back and shoulders, never on your head.",
	"Pull hard on the frame to start the lift; do not kick or fold to get up.",
})
guide("rollout_standing", {
	"Clear the floor to a wall ahead and stand where it stops the wheel at your full reach.",
	"Feet hip-width apart, bend forward and set the wheel on the floor by your toes.",
	"Roll out slowly until your body is flat just above the floor, arms long.",
	"Breathe out, press the wheel down and pull it back to your feet to stand.",
	"Each roll out and back to your feet counts as one, counted out loud.",
}, {
	"Tuck your tail under and pull your ribs down before the wheel moves.",
	"Keep your arms straight and your head between them at full reach.",
	"If you cannot pull back, set your knees down and roll in from there.",
})
guide("rollout_standing_negative", {
	"Clear the floor to a wall ahead and stand where it stops the wheel at your full reach.",
	"Feet hip-width apart, bend forward and set the wheel on the floor by your toes.",
	"Roll out over 3 s until your body is flat just above the floor, arms long.",
	"Set your knees down, breathe out and roll the wheel back in from your knees.",
	"Stand back up with the wheel. Each slow roll out counts as one, counted out loud.",
}, {
	"Tuck your tail under and pull your ribs down before the wheel moves.",
	"Keep your arms straight and your head between them at full reach.",
	"Set your knees down softly; the work is the slow roll out before they touch.",
})
guide("rollout_standing_wall", {
	"Lie once with your heels at the wall, arms past your head, and mark at your fingertips.",
	"Stand with your toes on that mark, face the wall and set the wheel down by your toes.",
	"Roll out slowly until the wheel touches the wall, arms straight, body in line.",
	"Breathe out, press the wheel down and pull it back to your feet, knees straight.",
	"Each roll to the wall and back to your feet counts as one, counted out loud.",
}, {
	"Tuck your tail under and pull your ribs down before the wheel moves.",
	"Keep your toes on the mark every time, so each roll is the same move.",
	"If you cannot pull back, set your knees down and roll in from there.",
})
guide("copenhagen_long", {
	"Lie on your side on the floor beside a bed or sofa that stays put when you push it.",
	"Put your top ankle on its edge, leg straight, elbow right under your shoulder.",
	"Lift your hips into a straight line from ankle to head, bottom leg hanging free.",
	"Put your top hand on your hip and breathe slowly and steadily.",
	"The clock counts up: hold until your hips sink, then press Switch sides for the other.",
}, {
	"Press the top ankle down into the edge, since that is what lifts your hips.",
	"Keep the top knee straight and your hips stacked, not tipping back.",
	"Push the floor away with your forearm so your shoulder stays strong.",
})
guide("side_plank_star", {
	"Lie on your side on the floor, elbow right under your shoulder, feet stacked.",
	"Lift your hips until your body is straight from feet to head.",
	"Lift the top leg a foot's length above the other and reach the top arm straight up.",
	"Hold the star still and breathe slowly and steadily.",
	"The clock counts up: hold until the star sinks, then press Switch sides for the other.",
}, {
	"Lift the top leg straight up, not forward, toes pointing ahead.",
	"Push the floor away with your forearm and the edge of your bottom foot.",
	"Keep your top hip right over the lower one as the leg goes up.",
})
guide("copenhagen_raises", {
	"Lie on your side on the floor beside a bed or sofa that stays put when you push it.",
	"Put your top ankle on its edge, leg straight, elbow right under your shoulder.",
	"Lift your hips into a straight line, then lower them slowly to touch the floor.",
	"Breathe out and lift your hips back into the straight line.",
	"Each lift is one. One side, press Switch sides, the other; type the weaker side's count.",
}, {
	"Lift by pressing the top leg into the edge, not by pushing off with the bottom foot.",
	"Keep your hips stacked and facing forward on the way down and up.",
	"Move at one even speed, with no drop at the bottom.",
})
guide("clutch_flag", {
	"Use only a steel pole fixed in the ground that does not flex or turn, on soft ground.",
	"Stand with the back of one armpit against the pole and grip it at chest height.",
	"Wedge the other elbow into your belly by the hip and grip the pole low with that hand.",
	"Lean onto that elbow, lift your feet and stretch your body out level, and breathe.",
	"The clock counts up: hold until your hips sink, then press Switch sides and swap arms.",
}, {
	"Pull with the hugging arm as hard as the low elbow pushes, so your body stays level.",
	"Tuck your knees first and straighten your legs once your hips are level.",
	"Long sleeves spare the skin of your inner arm, where the pole presses.",
})
guide("vertical_flag", {
	"Use only a steel pole fixed in the ground that does not flex or turn, on soft ground.",
	"Stand side-on: far hand above your head, near hand lower, a bit wider than your shoulders.",
	"Lock the low arm and push, pull with the high arm and kick your legs up beside the pole.",
	"Hold your body straight up beside the pole, legs together to the sky, low arm locked.",
	"The clock counts up: come down feet first when an arm bends, then press Switch sides.",
}, {
	"Push the low hand into the pole as hard as the high hand pulls.",
	"Squeeze your legs together and point your toes at the sky.",
	"Come down one foot at a time, never by letting go at the top.",
})
guide("flag_tuck", {
	"Use only a steel pole fixed in the ground that does not flex or turn, on soft ground.",
	"Stand side-on: far hand above your head, near hand lower, a bit wider than your shoulders.",
	"Lock the low arm and push, pull with the high arm and lift your feet, knees tucked.",
	"Raise your hips until they are level with your shoulders, and keep breathing.",
	"The clock counts up: hold until your hips sink, then press Switch sides and swap hands.",
}, {
	"Push the low arm long and hard, as if to push the pole away from you.",
	"Keep your chest facing straight ahead; do not let it turn up to the sky.",
	"Come down feet first and with control; never let go at the top.",
})
guide("flag_one_leg", {
	"Use only a steel pole fixed in the ground that does not flex or turn, on soft ground.",
	"Stand side-on: far hand above your head, near hand lower, a bit wider than your shoulders.",
	"Lock the low arm and push, pull with the high arm and lift into the tuck flag.",
	"Stretch the top leg out in line with your body, the lower knee tucked, and breathe.",
	"The clock counts up: hold until your hips sink, then press Switch sides and swap hands.",
}, {
	"Squeeze your seat so the long leg stays as high as your hips and shoulders.",
	"Push the low arm long and hard into the pole the whole time.",
	"Come down feet first and with control; never let go at the top.",
})
guide("straddle_flag", {
	"Use only a steel pole fixed in the ground that does not flex or turn, on soft ground.",
	"Stand side-on: far hand above your head, near hand lower, a bit wider than your shoulders.",
	"Lock the low arm and push, pull with the high arm and lift into the tuck flag.",
	"Open your legs straight and wide apart, body level with the ground, and breathe.",
	"The clock counts up: hold until the line sinks, then press Switch sides and swap hands.",
}, {
	"Open your legs to the same width each set so your record stays fair.",
	"Push with the low arm as hard as you pull with the high one.",
	"Come down feet first and with control; never let go at the top.",
})
L["move.straddle_flag.hint"] = "Open your legs only as wide as your hips allow, both legs straight."
guide("human_flag", {
	"Use only a steel pole fixed in the ground that does not flex or turn, on soft ground.",
	"Stand side-on: far hand above your head, near hand lower, a bit wider than your shoulders.",
	"Lock the low arm and push, pull with the high arm and lift into the tuck flag.",
	"Open out into one straight line, legs together, level with the ground, and breathe.",
	"The clock counts up: hold until the line sinks, then press Switch sides and swap hands.",
}, {
	"Squeeze your legs together and point your toes to keep the line stiff.",
	"Push with the low arm as hard as you pull with the high one.",
	"Come down feet first and with control; never let go at the top.",
})
guide("pike_pushup_feet_up", {
	"Kneel on the floor facing away from a bed or a low sofa, hands flat, shoulder-width apart.",
	"Put your toes up on its edge and walk your hands back until your hips are high.",
	"Bend your elbows and lower the top of your head slowly towards the floor.",
	"Breathe out and push back up until your arms are straight, hips staying high.",
	"Each push to straight arms counts as one, counted out loud.",
}, {
	"Aim your head a hand's length ahead of your hands, not between them.",
	"Let your elbows point back towards your feet, not out to the sides.",
	"Lower slowly so your head stops just short of the floor, never bumping it.",
})
guide("wall_handstand", {
	"Clear a 2 by 2 m floor by a bare wall, away from the desk, and warm your wrists.",
	"Start in a push-up with your feet at the wall, hands shoulder-width apart.",
	"Walk your feet up the wall and your hands in until your chest is near the wall.",
	"Push tall through straight arms, toes on the wall, and breathe slowly.",
	"The clock counts up: hold until your arms tire, then walk your hands out and feet down.",
}, {
	"Push the floor away so your shoulders rise up towards your ears.",
	"Pull your ribs in and squeeze your seat so your back stays flat.",
	"Spread your fingers wide and press your whole palm into the floor.",
})
guide("wall_hspu", {
	"Clear a 2 by 2 m floor by a bare wall, away from the desk, and fold a towel in two.",
	"Lay the towel two hand-lengths out from the wall, then walk up into the wall handstand.",
	"Bend your elbows and lower slowly until your head touches the towel softly.",
	"Breathe out and push the floor away until your arms are straight again.",
	"Each push to straight arms counts as one, counted out loud.",
}, {
	"Keep your elbows in line over your wrists, not flared out to the sides.",
	"Keep your belly tight and your seat squeezed so your back does not arch.",
	"Stuck at the bottom? Walk your feet down the wall; never drop onto your head.",
})
guide("handstand", {
	"Clear a body length of floor on every side, away from the desk, and warm your wrists.",
	"Step one foot forward and put your hands flat on the floor, shoulder-width apart.",
	"Kick the back leg up, push off the front foot and bring your legs together over your hips.",
	"Push tall through straight arms, ribs in, and breathe slowly as you balance.",
	"The clock counts up once both feet are up: hold until you tip, then step one foot down.",
}, {
	"Press your fingertips into the floor when your legs start to tip over.",
	"Kick up gently, since too hard a kick is what tips you over.",
	"Squeeze your seat and legs together so your body moves as one piece.",
})
guide("handstand_pushup", {
	"Clear a body length of floor on every side and lay a folded towel on the floor.",
	"Kick up to a steady handstand with the towel just past your fingers, arms straight.",
	"Bend your elbows and lower slowly until your head touches the towel softly.",
	"Breathe out and push back up to straight arms, legs together over your hips.",
	"Each push to straight arms counts as one, counted out loud.",
}, {
	"Lean your shoulders forward as you lower, so your head lands ahead of your hands.",
	"Keep your elbows in line over your wrists, not flared out to the sides.",
	"Squeeze your seat and legs so your feet stay over your hips the whole way.",
})
guide("wall_hspu_lowering", {
	"Clear a 2 by 2 m floor by a bare wall, away from the desk, and fold a towel in two.",
	"Lay the towel two hand-lengths out from the wall, then walk up into the wall handstand.",
	"Bend your elbows and lower for 3 to 5 slow beats until your head touches the towel.",
	"Walk your hands out and your feet down the wall, back to the plank.",
	"Each slow lowering counts as one, counted out loud; walk up again for the next.",
}, {
	"Keep your elbows in line over your wrists, not flared out to the sides.",
	"Never push back up from the towel: the slow way down is the whole rep.",
	"Say the beats out loud so the last part of the way down stays as slow as the first.",
})
guide("wall_handstand_off", {
	"Clear a 2 by 2 m floor by a bare wall, away from the desk, and warm your wrists.",
	"Walk up into the wall handstand, chest to the wall, arms straight.",
	"Push tall and let your hips drift over your hands until your toes leave the wall.",
	"Balance there with your fingers and palms, ribs in, and breathe slowly.",
	"Type the seconds your toes stayed off, then walk your hands out and your feet down.",
}, {
	"Press your fingertips into the floor as your legs drift away from the wall.",
	"Move your hips over your hands rather than pushing off the wall with your toes.",
	"A light tap back on the wall ends the count: start again when your toes leave it.",
})
guide("hspu_lowering", {
	"Clear the floor by a bare wall, away from the desk, and fold a towel in two.",
	"Face the wall, hands a forearm's length from it, the towel just past your fingers.",
	"Kick up, then lower for three slow beats until your head touches the towel softly.",
	"Step one foot down to the floor, then the other, and stand up.",
	"Each slow lowering counts as one, counted out loud; kick up again for the next.",
}, {
	"Lean your shoulders forward as you lower, so your head lands ahead of your hands.",
	"If you tip toward the wall, let your heels touch it, then step down.",
	"Never push back up from the towel: the slow way down is the whole rep.",
})
guide("wall_l_hold", {
	"Sit with your back against the wall, legs straight, and note where your heels are.",
	"Kneel facing away from the wall, set your hands flat on that spot and lift your hips.",
	"Walk one foot up the wall to the height of your hips, then the other.",
	"Push the floor away, hips over your hands, arms straight by your ears, and breathe.",
	"The clock counts up: hold until your arms tire, then walk your feet down one at a time.",
}, {
	"Hips behind your hands? Walk your hands a little closer to the wall.",
	"Push your shoulders up toward your ears so your chest does not sink.",
	"Keep your legs straight and level, the soles of your feet flat on the wall.",
})
guide("wall_kickup_turnout", {
	"Face the wall and set your hands flat a forearm's length from it, shoulder-width apart.",
	"Kick one leg up and push off the other, so both heels reach the wall softly.",
	"Push tall through straight arms and hold for a slow count of three.",
	"Shift onto one hand, step the other hand round and turn sideways, feet down one by one.",
	"Each soft kick-up that ends in a turn-out counts as one, counted out loud.",
}, {
	"Kick softly: your heels should touch the wall, never bang it.",
	"Keep your arms locked straight the whole time, also while you turn.",
	"Turn out to a different side each time, so both ways feel easy.",
})
guide("lsit_heels", {
	"Warm your wrists, then sit on the floor with your legs straight and together.",
	"Put your palms flat beside your hips, fingers forward or turned out, and lock your arms.",
	"Push the floor away hard until your hips lift off it. Keep your heels on the floor.",
	"Push your shoulders down away from your ears and breathe slowly.",
	"The clock counts up: hold until your hips sink or your shoulders rise.",
}, {
	"Lean a little forward so your hands sit just under your shoulders.",
	"Fists on a folded towel rest your wrists and give your hips a little more room.",
	"Think of growing tall: your neck gets long as your shoulders go down.",
})
guide("lsit_one_heel", {
	"Warm your wrists, then sit on the floor with your legs straight and together.",
	"Put your palms flat beside your hips, lock your arms and push until your hips lift.",
	"Lift one straight leg off the floor, level with your hips. Keep the other heel down.",
	"Keep your shoulders pushed down and breathe slowly.",
	"The clock counts up: hold until the leg sinks, then press Switch sides for the other leg.",
}, {
	"Lock the knee of the raised leg and point its toes.",
	"Squeeze the thigh of the raised leg hard, since that is what holds it up.",
	"Lean a little forward so the leg has room to rise.",
})
guide("lsit_tuck_floor", {
	"Warm your wrists, then sit on the floor with your knees bent and your feet flat.",
	"Put your palms flat beside your hips, a little in front of them, and lock your arms.",
	"Push the floor away hard and lift your hips, then pull both knees up toward your chest.",
	"Hold both feet off the floor, shoulders pushed down, and breathe slowly.",
	"The clock counts up: hold until a foot lands or your shoulders rise.",
}, {
	"Keep your hips between your hands; do not let them swing forward.",
	"Pull your knees up with your belly; do not lean back to lift them.",
	"Fists on a folded towel rest your wrists and give your feet a little more room.",
})
guide("lsit_one_leg_floor", {
	"Warm your wrists, then sit on the floor, palms flat beside your hips, arms locked.",
	"Push the floor away, lift your hips and pull both knees up into the tuck L-sit.",
	"Push one leg out straight in front, level with your hips, the other knee still tucked.",
	"Keep your arms locked and your shoulders down, and breathe slowly.",
	"The clock counts up: hold until the leg sinks, then press Switch sides for the other leg.",
}, {
	"Lock the knee of the straight leg and point its toes.",
	"Squeeze the thigh of the straight leg hard, since that is what holds it level.",
	"Lean your shoulders a little forward to give the leg room.",
})
guide("lsit_tuck_one_foot", {
	"Warm your wrists, then sit on the floor with your knees bent and your feet flat.",
	"Put your palms flat beside your hips, a little in front of them, and lock your arms.",
	"Push the floor away hard and lift your hips, your feet coming up onto their toes.",
	"Pull one knee up toward your chest, the other toes still down, shoulders pushed down.",
	"The clock counts up: hold until the lifted foot lands, then press Switch sides.",
}, {
	"Keep your hips between your hands; do not let them swing forward.",
	"Keep your chest up; do not lean forward to meet the lifted knee.",
	"Rest as little weight as you can on the foot that stays down.",
})
guide("lsit_cross", {
	"Warm your wrists, then sit on the floor with your legs crossed at the shins.",
	"Put your palms flat beside your hips, fingers forward, and lock your arms.",
	"Push the floor away hard and lift your hips, then draw your crossed legs up off it.",
	"Hold your seat and both feet in the air, shoulders pushed down, and breathe slowly.",
	"The clock counts up: hold until a foot lands or your shoulders rise.",
}, {
	"Pull your knees up toward your chest with your belly; keep your chest up.",
	"Cross your shins the other way next time, so both hips get the same work.",
	"Fists on a folded towel rest your wrists and give your feet a little more room.",
})
guide("lsit_foot_supported", {
	"Set parallettes or two equal stacks of hard books just wider than your hips.",
	"Sit between them, legs straight out, and press on each stack to check it stays put.",
	"Lay a hand flat on each stack, lock your arms and push down until your hips lift.",
	"Keep your heels on the floor, push your shoulders down and breathe slowly.",
	"The clock counts up: hold until your hips sink or your shoulders rise.",
}, {
	"Keep your chest up and look straight ahead, not down at your legs.",
	"Press through the whole hand with your fingers pointing forward.",
	"Think of growing tall: your neck gets long as your shoulders go down.",
})
guide("lsit_tuck", {
	"Set parallettes or two equal stacks of hard books just wider than your hips.",
	"Sit between them with your knees bent and lay a hand flat on each stack.",
	"Lock your arms, push down and lift your hips, then pull both knees up toward your chest.",
	"Hold both feet in the air, shoulders pushed down, and breathe slowly.",
	"The clock counts up: hold until a foot lands or your shoulders rise.",
}, {
	"Push the books down hard; the taller you push, the more room your feet have.",
	"Keep your arms locked and your chest lifted, not caved in.",
	"Pull your knees up with your belly; do not lean back to lift them.",
})
guide("lsit_one_leg", {
	"Set parallettes or two equal stacks of hard books just wider than your hips.",
	"Sit between them, a hand flat on each stack, push down and lift into the tuck L-sit.",
	"Push one leg out straight in front, level with your hips, the other knee still tucked.",
	"Keep your arms locked and your shoulders down, and breathe slowly.",
	"The clock counts up: hold until the leg sinks, then press Switch sides for the other leg.",
}, {
	"Lock the knee of the straight leg and point its toes.",
	"Squeeze the thigh of the straight leg hard, since that is what holds it level.",
	"Keep your chest lifted as the leg goes out.",
})
guide("lsit_books", {
	"Set parallettes or two equal stacks of hard books just wider than your hips.",
	"Sit between them with your legs straight and together and lay a hand flat on each stack.",
	"Lock your arms, push down and lift your hips and both heels off the floor.",
	"Hold your legs as high as your hips, shoulders pushed down, and breathe slowly.",
	"The clock counts up: hold until your heels sink below your hips.",
}, {
	"Squeeze your thighs and point your toes to keep your knees locked.",
	"Push your shoulders down and your chest up, as if growing taller.",
	"Keep your arms locked, since bent elbows let your hips sink.",
})
guide("lsit_floor", {
	"Warm your wrists, then sit on the floor with your legs straight and together.",
	"Put your palms flat beside your hips, fingers forward, and lock your arms.",
	"Lean a little forward, push down hard and lift your hips, then your heels.",
	"Hold your legs as high as your hips, shoulders pushed down, and breathe slowly.",
	"The clock counts up: hold until a heel lands or your legs lose their line.",
}, {
	"Push your shoulders down as far as they go, since that gives your hips room.",
	"Squeeze your thighs and point your toes so your knees stay locked.",
	"Turn your fingers out to the sides if your wrists feel tight.",
})
-- the chair forms (slice 54)
move("chair_tuck_hold", "Chair tuck hold", "A stable chair that cannot tip; hands on the front corners of its seat.",
	"Push the seat away, hips off its edge, knees tucked, feet off. Count out loud.",
	"Stop when a foot touches down or your shoulders creep up.")
guide("chair_tuck_hold", {
	"Set a stable chair against a wall and sit on its front edge, feet flat.",
	"Put your hands on the front corners of the seat beside your hips and lock your arms.",
	"Push the seat away, slide your hips just off its edge and lift both feet.",
	"Pull your knees up toward your chest and hold, breathing slowly.",
	"The clock counts up: hold until a foot lands or your shoulders rise.",
}, {
	"Push your shoulders down away from your ears the whole hold.",
	"Keep your chest up; do not let it cave in over your knees.",
})
move("chair_one_leg_hold", "Chair one-leg hold",
	"A stable chair that cannot tip; hands on the front corners of its seat.",
	"Hips off the edge, one leg straight out, the other knee tucked. Count out loud.",
	"Stop when the straight leg drops or your shoulders creep up. Count the weaker side.")
guide("chair_one_leg_hold", {
	"Set a stable chair against a wall and sit on its front edge, feet flat.",
	"Put your hands on the front corners of the seat beside your hips and lock your arms.",
	"Push the seat away, slide your hips just off its edge and tuck both knees up.",
	"Straighten one leg out level with your hips and hold, counting out loud.",
	"When this side ends, press Switch sides and straighten the other leg.",
}, {
	"Push your shoulders down away from your ears the whole hold.",
	"Lock the straight knee and point your toes.",
})
move("chair_lsit", "Chair L-sit", "A stable chair that cannot tip; hands on the front corners of its seat.",
	"Hips off the edge, both legs straight out level with your hips. Count out loud.",
	"Stop when your knees bend or your shoulders creep up.")
guide("chair_lsit", {
	"Set a stable chair against a wall and sit on its front edge, feet flat.",
	"Put your hands on the front corners of the seat beside your hips and lock your arms.",
	"Push the seat away and slide your hips just off its edge.",
	"Lift both legs straight out level with your hips and hold, breathing slowly.",
	"The clock counts up: hold until your heels drop or your knees bend.",
}, {
	"Push your shoulders down away from your ears the whole hold.",
	"Lock both knees and squeeze your thighs together.",
})
guide("seated_leg_lifts", {
	"Sit on the floor, back tall, legs straight and together, toes pointed.",
	"Put your palms flat on the floor beside your knees, arms straight.",
	"Keep your knees locked and lift both heels off the floor with your thighs alone.",
	"Hold them up for one slow breath, then lower them gently and go again.",
	"Each lift counts as one rep, counted out loud.",
}, {
	"Lean your chest a little over your legs; leaning back makes the lift easier, not better.",
	"A lift of a few centimetres is plenty at first; the height comes with time.",
	"Squeeze your thighs and point your toes so your knees stay straight.",
})
guide("vsit_tuck", {
	"Warm your wrists, then sit on the floor, palms flat beside your hips, fingers out or back.",
	"Lock your arms, push the floor away and lift into the floor L-sit.",
	"Draw your knees in to your chest and push your hips forward and up, past your hands.",
	"Hold your feet above your hips, arms locked, and breathe slowly.",
	"The clock counts up: hold until your feet sink below your hips.",
}, {
	"Lean your shoulders back a little as your hips come forward, to keep the balance.",
	"Push your shoulders down hard the whole time.",
	"Keep your knees tight to your chest; a loose tuck pulls your feet down.",
})
guide("vsit_low", {
	"Warm your wrists, then sit on the floor, palms flat beside your hips, fingers out or back.",
	"Lock your arms, push the floor away and lift into the floor L-sit.",
	"Push your hips forward past your hands and raise your legs halfway to straight up.",
	"Hold them there with locked arms and breathe slowly.",
	"The clock counts up: hold until your legs sink back towards the L-sit.",
}, {
	"Lean your shoulders back as your legs rise, but keep your elbows locked.",
	"Squeeze your thighs and point your toes so your knees stay straight.",
	"Push your shoulders down and keep your hips ahead of your hands.",
})
guide("vsit_one_leg", {
	"Warm your wrists, then sit on the floor, palms flat beside your hips, fingers out or back.",
	"Lock your arms, push the floor away and lift into the tuck V-sit, hips past your hands.",
	"Push one leg straight up toward the ceiling, the other knee still tucked to your chest.",
	"Lean your shoulders back, keep both arms locked and breathe slowly.",
	"The clock counts up: hold until the leg drops, then press Switch sides for the other leg.",
}, {
	"Think of lifting your hips, not your foot: the leg follows the hips up.",
	"Squeeze the straight thigh and point your toes so the knee stays locked.",
	"Keep your shoulders pushed down; do not let them shrug up.",
})
guide("vsit", {
	"Warm your wrists, then sit on the floor, palms flat beside your hips, fingers out or back.",
	"Lock your arms, push the floor away and lift into the floor L-sit.",
	"Push your hips forward and up, past your hands, and lean your shoulders back.",
	"Raise your legs until they point straight up, knees locked, and keep breathing.",
	"The clock counts up: hold until your legs sink from straight up.",
}, {
	"Think of lifting your hips, not your feet: the legs follow the hips up.",
	"Squeeze your thighs and point your toes so your knees stay locked.",
	"Keep your shoulders pushed down; do not let them shrug up.",
})
guide("planche_lean", {
	"Warm your wrists, then start in a push-up on the floor, fingers turned out a little.",
	"Lock your arms and push the floor away until your upper back is a little round.",
	"Roll forward on your toes until your shoulders are right over your fingertips.",
	"Hold a straight line from heels to head, arms locked, and breathe slowly.",
	"The clock counts up: hold until your arms unlock or your shoulders slide back.",
}, {
	"Turn the insides of your elbows to face forward, which locks your arms.",
	"Tuck your tail under and squeeze your seat so your hips stay in line.",
	"Lean to the same mark each time so your record stays fair.",
})
guide("frog_stand_one_foot", {
	"Put the pillow on the floor in front of where your hands will go, then squat on the floor.",
	"Set your palms flat, shoulder-width apart, fingers spread, and bend your elbows a little.",
	"Rest your knees on the backs of your upper arms, just above the elbows.",
	"Lean forward until your toes grow light, then lift one foot and breathe slowly.",
	"The clock counts up: hold until the lifted foot lands, then press Switch sides.",
}, {
	"Keep the other toes on the floor as light as you can; they only steady you.",
	"Look at the floor a little ahead of your hands, not back at your feet.",
	"Press your fingertips down to stop a tip forward.",
})
guide("frog_stand_books", {
	"Stack books about 12 cm high where your feet will go, the pillow in front of your hands.",
	"Set your palms flat, shoulder-width apart, fingers spread, and bend your elbows a little.",
	"Toes on the books, rest your knees on the backs of your upper arms, just above the elbows.",
	"Lean forward until both feet float off the books, then breathe slowly.",
	"The clock counts up once both feet are up; when the balance goes, feet back on the books.",
}, {
	"Lift off the books, never push off them: they only start you higher.",
	"Look at the floor a little ahead of your hands, not back at your feet.",
	"Press your fingertips down to stop a tip forward.",
})
guide("frog_stand", {
	"Put the pillow on the floor in front of where your hands will go, then squat on the floor.",
	"Set your palms flat, shoulder-width apart, fingers spread, and bend your elbows a little.",
	"Rest your knees on the backs of your upper arms, just above the elbows.",
	"Lean forward until your toes grow light, then lift both feet and breathe slowly.",
	"The clock counts up once both feet are up: hold until a foot lands.",
}, {
	"Look at the floor a little ahead of your hands, not back at your feet.",
	"Press your fingertips down to stop a tip forward.",
	"Squeeze your knees against your arms so they do not slide off.",
})
guide("pseudo_planche_pushup", {
	"Warm your wrists, then start in a push-up on the floor, fingers turned out to the sides.",
	"Roll forward on your toes until your hands are beside your waist.",
	"Hold that lean and lower your chest to just above the floor.",
	"Breathe out and push up to locked arms, still leaning forward.",
	"Each push to locked arms counts as one, counted out loud.",
}, {
	"Keep your elbows close to your ribs, pointing back towards your feet.",
	"Push the floor away at the top so your upper back rounds a little.",
	"Squeeze your seat and belly so your body moves as one straight line.",
})
guide("crane_stand", {
	"Put the pillow on the floor in front of where your hands will go, then squat on the floor.",
	"Set your palms flat, shoulder-width apart, fingers spread, and lock your elbows.",
	"Rise onto your toes and set your knees high on the backs of your upper arms.",
	"Lean forward on locked arms until both feet lift, and breathe slowly.",
	"The clock counts up once both feet are up: hold until a foot lands.",
}, {
	"Keep your elbows locked; a bend turns it back into the frog stand.",
	"Push the floor away and round your upper back to lift your hips.",
	"Let your shoulders move ahead of your hands as your feet come up.",
})
guide("tuck_planche_assisted", {
	"Stack books about 15 cm high where your feet will go, a pillow in front of your hands.",
	"Squat, palms flat shoulder-width apart, fingers turned out a little, elbows locked.",
	"Lean your shoulders ahead of your hands and draw your knees to your chest, off your arms.",
	"Rest your toes lightly on the books, raise your hips toward your shoulders and breathe.",
	"The clock counts up: hold until your elbows bend or your toes press into the books.",
}, {
	"The books only steady you: press them as little as you can.",
	"Push the floor away so your upper back rounds and your shoulders stay low.",
	"Lift your toes off the books for a moment at a time: that is the tuck planche.",
})
guide("tuck_planche", {
	"Put the pillow in front of where your hands will go, warm your wrists and squat down.",
	"Set your palms flat, shoulder-width apart, fingers turned out a little, elbows locked.",
	"Lean your shoulders ahead of your hands and draw your knees to your chest, off your arms.",
	"Lift both feet, raise your hips as high as your shoulders and breathe slowly.",
	"The clock counts up once both feet are up: hold until a foot lands.",
}, {
	"Keep your knees free of your arms: only your hands touch the floor.",
	"Push the floor away so your upper back rounds and your shoulders stay low.",
	"Turn the insides of your elbows forward to keep your arms locked.",
})
guide("open_tuck_planche", {
	"Put the pillow in front of where your hands will go, warm your wrists and squat down.",
	"Palms flat, shoulder-width apart, elbows locked, lean forward into the tuck planche.",
	"Move your knees halfway back from your chest and let your back go flatter.",
	"Lean a little further forward to balance, hips as high as your shoulders, and breathe.",
	"The clock counts up from the open hips: hold until your hips drop or a foot lands.",
}, {
	"Your knees stay under your belly, halfway between your chest and the flat-back tuck.",
	"Push the floor away and keep the insides of your elbows facing forward.",
	"Open your hips a little further each week as the hold gets steady.",
})
guide("adv_tuck_planche", {
	"Put the pillow in front of where your hands will go, warm your wrists and squat down.",
	"Palms flat, shoulder-width apart, elbows locked, lean forward into the tuck planche.",
	"Move your knees back from your chest until your back is flat and level.",
	"Lean further forward to balance your longer body, and breathe slowly.",
	"The clock counts up from the flat back: hold until your back rounds or a foot lands.",
}, {
	"The further your knees go back, the more your shoulders must lean ahead.",
	"Keep your hips as high as your shoulders, not above them.",
	"Push the floor away and keep the insides of your elbows facing forward.",
})
guide("one_leg_planche", {
	"Put the pillow in front of where your hands will go, warm your wrists and squat down.",
	"Palms flat, shoulder-width apart, elbows locked, lean into the flat-back tuck planche.",
	"Push one leg straight back, as high as your hips, the other knee still tucked.",
	"Lean your shoulders further ahead of your hands to hold the balance, and breathe.",
	"The clock counts up from the straight leg: hold until it sinks, then press Switch sides.",
}, {
	"Squeeze the seat of the straight leg and point its toes so it stays level.",
	"Keep your back flat; a round back takes the straight leg down.",
	"Push the floor away and keep the insides of your elbows facing forward.",
})
guide("straddle_planche", {
	"Put the pillow in front of where your hands will go, warm your wrists and squat down.",
	"Palms flat, shoulder-width apart, elbows locked, lean forward into the tuck planche.",
	"Open your legs wide and push them out straight behind you, as high as your back.",
	"Lean your shoulders far ahead of your hands to hold the balance, and breathe slowly.",
	"The clock counts up once your legs are straight: hold until your feet sink.",
}, {
	"Squeeze your seat and point your toes so your legs stay as high as your hips.",
	"The wider your legs, the easier the hold: start wide and bring them in over time.",
	"Push the floor away and keep the insides of your elbows facing forward.",
})
L["move.straddle_planche.hint"] = "Open your legs only as wide as your hips allow; wider comes with time."
guide("burpee_strict", {
	"Stand at the back of your clear space, feet hip-width apart, with clear space above you.",
	"Squat, put your hands flat on the floor and jump your feet back to a straight line.",
	"Lower your chest to the floor, then push up with your body in one straight piece.",
	"Jump your feet in to your hands, stand and jump with your arms overhead, breathing out.",
	"Each landing from the jump counts as one, counted out loud.",
}, {
	"Push up as one piece: chest and hips leave the floor together.",
	"Land on the balls of your feet with soft knees to keep it quiet.",
	"Stand up tall before each jump, one clean rep at a time.",
})
guide("tuck_jump", {
	"Stand with feet hip-width apart and check nothing hangs low above your head.",
	"Dip into a small squat and swing your arms back behind you.",
	"Swing your arms up, jump straight up and pull both knees up towards your chest.",
	"Breathe out as you jump, stretch your legs back down and land softly on bent knees.",
	"Each soft landing counts as one, counted out loud; stand tall before the next jump.",
}, {
	"Bring your knees up to your chest; do not fold your chest down to your knees.",
	"Land on the balls of your feet, then let your heels come down.",
	"Keep your knees pointing over your toes on take-off and on landing.",
})
guide("explosive_pushup", {
	"Start in a push-up on the floor, hands under your shoulders, body straight.",
	"Lower your chest to about a fist above the floor.",
	"Breathe out and push up as fast as you can, so both hands lift off a little.",
	"Land on both hands at once, elbows bending, and lower into the next rep.",
	"Each push that lifts both hands counts as one, counted out loud.",
}, {
	"Squeeze your seat and belly so your body lifts and lands as one straight piece.",
	"Catch each landing with spread fingers and a bend in your arms.",
	"A small hop of the hands is enough; push for speed, not height.",
})
guide("clap_pushup", {
	"Start in a push-up on the floor, hands under your shoulders, body straight.",
	"Lower your chest to about a fist above the floor.",
	"Breathe out and push up as hard and fast as you can, so your hands lift off.",
	"Clap your hands under your chest, then put them back under your shoulders.",
	"Land with bending elbows. Each clap with a clean landing counts as one, out loud.",
}, {
	"Clap small and quick, close to the floor, so your hands are back in time.",
	"Keep your hips in line; do not pike them up to buy time for the clap.",
	"Spread your fingers and let your arms bend to take the landing.",
})
guide("knee_clap_pushup", {
	"Kneel on the towel, hands under your shoulders, body straight from knees to head.",
	"Lower your chest to about a fist above the floor.",
	"Breathe out and push up as hard and fast as you can, so your hands lift off.",
	"Clap your hands under your chest, then put them back under your shoulders.",
	"Land with bending elbows. Each clap with a clean landing counts as one, out loud.",
}, {
	"Keep your hips in line with your knees and shoulders; do not bend at the hips.",
	"Clap small and quick, close to the floor, so your hands are back in time.",
	"Spread your fingers and let your arms bend to take the landing.",
})
guide("explosive_pullup", {
	"Use a fixed bar with room above it, never a doorway bar, and set a box to step down on.",
	"Hang with palms facing away, hands a little wider than your shoulders, arms straight.",
	"Breathe out and pull as fast as you can until your chest is at the bar.",
	"Lower with control to straight arms and let your body go still.",
	"Each fast pull to the chest counts as one, counted out loud.",
}, {
	"Keep both hands on the bar; the speed is in the pull, not in letting go.",
	"Drive your elbows down and back hard, as if to bring the bar to your chest.",
	"Start each rep from a still hang, with no kick.",
})
guide("high_pullup", {
	"Use a fixed bar with room above it, never a doorway bar, and set a box to step down on.",
	"Hang with palms facing away, hands shoulder-width apart, arms straight.",
	"Breathe out and pull as hard and fast as you can, leaning back a little as you rise.",
	"Keep pulling until the bar is at your lower ribs, then lower slowly to straight arms.",
	"Each pull that brings the bar to your ribs counts as one, counted out loud.",
}, {
	"Pull the bar down towards your waist; do not just lift your chin to it.",
	"Keep your legs together and a little forward so your body stays still.",
	"Let your body go still at the bottom before the next pull.",
})
guide("straight_bar_dip", {
	"Use a fixed bar at about chest height, never a doorway bar, with a box beside it.",
	"Grip the bar shoulder-width, palms down, step up and press to locked arms on top of it.",
	"Lean your chest a little forward, legs a little ahead, and bend your elbows back.",
	"Lower until your chest nears the bar, then breathe out and press back to locked arms.",
	"Each press back to locked arms counts as one, counted out loud.",
}, {
	"Keep your elbows pointing back, close to your sides, not flaring out.",
	"Keep your shoulders pushed down, away from your ears, all the way up.",
	"Step down to the box when your arms tire; never drop back off the bar.",
})
L["move.straight_bar_dip.hint"] = "Stop a little higher if the front of your shoulders feels tight."
guide("straight_bar_dip_lowering", {
	"Use a fixed bar at about chest height, never a doorway bar, with a box beside it.",
	"Grip the bar shoulder-width, palms down, step up and press to locked arms on top of it.",
	"Lean your chest a little forward, legs a little ahead, and bend your elbows back.",
	"Lower over five slow beats, about 5 s, until your chest nears the bar, breathing out.",
	"Step down to the box and step up again. Each slow lowering counts one, out loud.",
}, {
	"Keep your elbows pointing back, close to your sides, not flaring out.",
	"Keep your shoulders pushed down, away from your ears, all the way down.",
	"Get back to the top with your feet on the box, never with a jump.",
})
L["move.straight_bar_dip_lowering.hint"] = "Lower only as far as your shoulders feel comfortable."
guide("bar_support_hold", {
	"Use a fixed bar at about chest height, never a doorway bar, with a box beside it.",
	"Grip the bar shoulder-width, palms down, step up and press to locked arms on top of it.",
	"Push the bar down so your shoulders stay low, away from your ears, hips against the bar.",
	"Hold still with your legs a little ahead and breathe slowly.",
	"The clock counts up: hold until your arms bend or your shoulders creep up, then step down.",
}, {
	"Keep your elbows locked and pointing back, close to your sides.",
	"Lean your chest a little forward so your weight stays over your hands.",
	"Step down to the box at the end; never drop back off the bar.",
})
L["move.bar_support_hold.hint"] = "Press your shoulders down, away from your ears, and keep your chest open."
guide("muscle_up_jump", {
	"Use a fixed bar with room above it, never a doorway bar, and a sturdy box below it.",
	"Stand on the box, the bar at about chest height, and grip it shoulder-width, palms away.",
	"Jump and pull at once, lean your chest over the bar and turn both elbows up together.",
	"Breathe out and press to straight arms, hips at the bar, then lower slowly to the box.",
	"Each press to straight arms above the bar counts as one, counted out loud.",
}, {
	"Lean your chest over the bar as your elbows turn, since that gets you on top.",
	"Jump only as hard as you need, so your arms do the work.",
	"Come down the way you went up: bend your arms, then turn your elbows under.",
})
L["move.muscle_up_jump.hint"] = "Lean your chest over the bar only as far as your shoulders allow."
guide("muscle_up_negative", {
	"Use a fixed bar with room above it, never a doorway bar, and a sturdy box below it.",
	"Stand on the box, the bar at about chest height, and grip it shoulder-width, palms away.",
	"Jump and press to straight arms on top of the bar, hips at the bar.",
	"Lower your chest to the bar, lean back and turn both elbows under it together.",
	"Take 5 s or more until your feet are on the box. Each slow lowering counts one, out loud.",
}, {
	"Go slowest through the turn: that is the part the muscle-up needs.",
	"Keep the bar close to your body all the way down, as if pulling it to your waist.",
	"Breathe out as you lower; jump only as high as you need to reach the top.",
})
L["move.muscle_up_negative.hint"] = "Slow down through the turn if the front of your shoulders feels tight."
guide("muscle_up", {
	"Use a fixed bar with room above it, never a doorway bar, and set a box to step down on.",
	"Hang with palms facing away, hands shoulder-width apart, arms straight.",
	"Breathe out and pull hard and fast, leaning back, until the bar is at your lower ribs.",
	"Lean your chest over the bar, turn both elbows up together and press to straight arms.",
	"Lower slowly back to a still hang. Each press above the bar counts as one, out loud.",
}, {
	"Keep the bar close to your body all the way up, as if pulling it to your waist.",
	"Roll your wrists over the top as your chest leans forward.",
	"Lower through the turn with control; never drop from the top into the hang.",
})
L["move.muscle_up.hint"] = "Open shoulders make the turn over the bar smoother."

-- Slice 3: the character sheet, the HUD, the toasts, the Rewards section and the settings (docs/plan.md slice 3) ----

L["binding.sheet"] = "Open the character sheet"
L["cmd.stats.help"] = "/qp stats: your character sheet"
L["cmd.lock.help"] = "/qp lock: fix or free the HUD line (drag it by its header)"
L["cmd.lock.on"] = "HUD line locked."
L["cmd.lock.off"] = "HUD line unlocked: drag it by its header."
L["cmd.export.help"] = "/qp export: a string that carries your progress"
L["cmd.import.help"] = "/qp import [string]: merge an export string in"
L["cmd.reset.help"] = "/qp reset char|all|move <name>|ladder <name>: the settings of the profile in use, everything, "
	.. "one move's record or one ladder, back to the start"
L["cmd.reset.usage"] = "/qp reset char, /qp reset all, /qp reset move <name> or /qp reset ladder <name>"
L["cmd.reset.char"] = "The settings of the profile %s are back to the defaults; the Body page's switches and your "
	.. "exclusions stay."
L["cmd.reset.all"] = "Everything is reset. /qp restore-backup brings it back in this game only."
L["cmd.reset.move"] = "Record cleared: %s."
L["cmd.reset.ladder"] = "%s ladder started over."
L["cmd.reset.noMove"] = "No move with a record is named \"%s\"."
L["cmd.reset.noLadder"] = "No ladder you can start over is named \"%s\"."
L["cmd.reset.several"] = "More than one name holds \"%s\": %s. Type more of the name."
L["cmd.config.help"] = "/qp config: the settings"
L["mark.missed"] = "–"

-- the stats and ratings (Blizzard's own names are used where the client has them)
L["stat.strength"] = "Strength"
L["stat.agility"] = "Agility"
L["stat.intellect"] = "Intellect"
L["stat.stamina"] = "Stamina"
L["stat.spirit"] = "Spirit"
L["stat.primary"] = "Primary Attributes"
L["stat.modifiers"] = "Modifiers"
L["stat.defense"] = "Defense"
L["rating.crit"] = "Crit Chance"
L["rating.expertise"] = "Expertise"
L["rating.defense"] = "Defense"
L["rating.block"] = "Block"
L["tip.stat.strength"] = "Raised by the push, squat, pull-up, row, hinge and planche ladders: records, learning their "
	.. "moves, a week of practice, stars and every step you reach."
L["tip.stat.agility"] = "Raised by cardio, power, jumps and strikes: records, learning them, a week of practice, "
	.. "every step you reach on the cardio and power ladders and steps of the cardio progression."
L["tip.stat.intellect"] = "Raised by balance, arm balances and the handstand: records, learning them, a week of "
	.. "practice and every step you reach; and by every new pattern, skill moves that turn familiar and a week of "
	.. "learning."
L["tip.stat.stamina"] = "Raised by the core, the side core, the wall sit and the L-sit: records, learning their "
	.. "moves, a week of practice, stars and every step you reach."
L["tip.stat.spirit"] = "Raised by the splits, the pike, backbends, twists, ankles and squats: learning them, a week of "
	.. "practice and every step you reach; and by active weeks, rested days and coming back."
L["tip.stat.last"] = "Last: +%d, %s (%s)"
L["tip.stat.gain"] = "%s %d (%d%s+%d%s)"
L["tip.rating.crit"] = "Critical strike rating %d: 5%% to start, rising for good with each day's first three wins (a "
	.. "record, a first try beaten, more than last time, a star or a step). Last %d days: %d wins in %d rep and hold "
	.. "moves."
L["tip.rating.expertise"] = "Moves learned: done on %d different days."
L["tip.rating.defense"] = "Days in the last %d with a pump, or covered by a rest token."
L["tip.rating.block"] = "Rest tokens held: each keeps your streak through a day off."
L["reason.record"] = "a record"
L["reason.firstTry"] = "a first try beaten"
L["reason.star"] = "a star"
L["reason.rung"] = "a step"
L["reason.tried"] = "a new move"
L["reason.familiar"] = "a move turned familiar"
L["reason.learned"] = "a move learned"
L["reason.skill"] = "a skill move turned familiar"
L["reason.pattern"] = "a new pattern"
L["reason.learning"] = "a week of learning"
L["reason.practice"] = "a week of practice"
L["reason.active"] = "an active week"
L["reason.clean"] = "a week with no banked move left"
L["reason.rested"] = "a rested day"
L["reason.welcome"] = "coming back"
L["reason.p"] = "the cardio progression"
L["build.tiger"] = "of the Tiger"
L["build.bear"] = "of the Bear"
L["build.gorilla"] = "of the Gorilla"
L["build.boar"] = "of the Boar"
L["build.monkey"] = "of the Monkey"
L["build.falcon"] = "of the Falcon"
L["build.wolf"] = "of the Wolf"
L["build.eagle"] = "of the Eagle"
L["build.whale"] = "of the Whale"
L["build.owl"] = "of the Owl"
L["fam.new"] = "New"
L["fam.tried"] = "Tried"
L["fam.familiar"] = "Familiar"
L["fam.learned"] = "Learned"
L["skill.apprentice"] = "Apprentice"
L["skill.journeyman"] = "Journeyman"
L["skill.expert"] = "Expert"
L["skill.artisan"] = "Artisan"
L["quality.0"] = "Starter"
L["quality.1"] = "Common"
L["quality.2"] = "Uncommon"
L["quality.3"] = "Rare"
L["quality.4"] = "Epic"
L["quality.5"] = "Legendary"
L["quality.6"] = "Artifact"

-- the muscles (the glossary's ids), the regions, the patterns and the Exercises page's groups
L["muscle.1"] = "Shoulders"
L["muscle.2"] = "Chest"
L["muscle.3"] = "Biceps"
L["muscle.4"] = "Forearms and grip"
L["muscle.5"] = "Abs"
L["muscle.6"] = "Obliques"
L["muscle.7"] = "Quads"
L["muscle.8"] = "Inner thighs"
L["muscle.9"] = "Neck and traps"
L["muscle.10"] = "Upper back"
L["muscle.11"] = "Lats"
L["muscle.12"] = "Triceps"
L["muscle.13"] = "Lower back"
L["muscle.14"] = "Glutes"
L["muscle.15"] = "Hamstrings"
L["muscle.16"] = "Calves and shins"
L["muscle.17"] = "Heart"
L["muscle.18"] = "Breath"
L["muscle.19"] = "Balance"
L["region.push"] = "Push"
L["region.legs"] = "Legs"
L["region.pull"] = "Pull"
L["region.trunk"] = "Trunk"
L["pattern.squat"] = "Squat"
L["pattern.hinge"] = "Hinge"
L["pattern.lunge"] = "Lunge"
L["pattern.push"] = "Push"
L["pattern.pull"] = "Pull"
L["pattern.core_anti_ext"] = "Core, front"
L["pattern.core_anti_rot"] = "Core, sides"
L["pattern.core_flex"] = "Core, curls"
L["pattern.carry"] = "Carry"
L["pattern.iso"] = "Hold"
L["pattern.mobility"] = "Mobility"
L["pattern.balance"] = "Balance"
L["pattern.ballistic"] = "Jumps"
L["pattern.cardio"] = "Cardio"
L["pattern.run"] = "Running"
L["pattern.calf"] = "Calves"
L["pattern.strike"] = "Strikes"
L["group.backpack"] = "Backpack loaded with books or bottles"
L["group.ball"] = "Medicine ball"
L["group.bands"] = "Bands"
L["group.bar"] = "Pull-up bar"
L["group.barbell"] = "Barbell"
L["group.bike"] = "Exercise bike"
L["group.calf"] = "Calves"
L["group.cardio"] = "Cardio"
L["group.carry"] = "Carries"
L["group.core"] = "Core"
L["group.dumbbells"] = "Dumbbells"
L["group.hinge"] = "Hinge"
L["group.kettlebell"] = "Kettlebell"
L["group.lunge"] = "Lunges"
L["group.mobility"] = "Mobility"
L["group.pad"] = "Walking pad"
L["group.pole"] = "A pole outdoors"
L["group.pull"] = "Pull"
L["group.push"] = "Push"
L["group.rings"] = "Rings"
L["group.roller"] = "Foam roller"
L["group.rope"] = "Jump rope"
L["group.situp"] = "Sit-ups"
L["group.squat"] = "Squats"
L["group.stairs"] = "Stairs"
L["group.step"] = "Step or box"
L["group.vest"] = "Weighted vest"
L["group.wheel"] = "Ab wheel"
L["group.yoga"] = "Yoga"

-- the sheet
L["sheet.tab.body"] = "Body"
L["sheet.tab.ladders"] = "Talents"
L["sheet.tab.today"] = "Today"
L["sheet.tab.log"] = "Log"
L["sheet.tab.records"] = "Records"
L["sheet.tab.deeds"] = "Deeds"
L["sheet.tab.guild"] = "Guild"
L["sheet.tab.back"] = "Character"
L["sheet.tab.dock"] = "QuestPump"
L["sheet.mode.week"] = "This week"
L["sheet.mode.rungs"] = "Steps"
L["sheet.mode.hero"] = "Hero"
-- each map mode's cell's tooltip: what the mode shows
L["sheet.mode.week.tip"] = "Each muscle's button shows its moves this week."
L["sheet.mode.rungs.tip"] = "Each muscle's button shows the best step of its moves."
L["sheet.mode.hero.tip"] = "Your character in place of the muscle map."
L["sheet.total"] = "%d/%d"
L["sheet.total.week"] = "muscles this week"
L["sheet.total.rungs"] = "muscles with a step"
L["sheet.legend.none"] = "0"
L["sheet.legend.some"] = "1-%d"
L["sheet.legend.many"] = "%d+"
L["sheet.legend.tip"] = "Moves this week for each muscle; a muscle that only helps counts half. A dashed line marks %d "
	.. "or more; the colour stays the same."
L["sheet.level"] = "QP %d"
-- the Body tab's line under the level (2026-10-16): the build and the focus
L["sheet.level.build"] = "%s · %s focus"
L["sheet.paragon"] = "Paragon %d"
L["sheet.plate"] = "QP %d"
L["sheet.xp"] = "%s / %s"
L["sheet.xp.paragon"] = "%s / %s to Paragon %d"
-- the ladders' words on the Talents tab (Tree.lua, slice 37; the Ladders tab's rows until then): the step's card and
-- its hover
L["ladders.tip.record"] = "Record: %s → %s since %s"
L["ladders.tip.mark"] = "Goal pose: %s"
L["ladders.tip.opens"] = "%s: %d more moves here, up to %s"
L["ladders.tip.opens.one"] = "%s: %d more move here, up to %s"
L["ladders.tip.restart"] = "Right-click: start this ladder over"
L["ladders.collection.item"] = "%s %d/%d"
-- the Talents foot's badge's tooltip: what its fraction counts (have, can, the quality's words)
L["tree.foot.tip"] = "%d of the %d ladders that go as high as %s have reached it."
L["tree.foot.tip.one"] = "%d of the %d ladders that go as high as %s has reached it."
L["ladders.step.earned"] = "Unlocked %s"
L["ladders.step.start"] = "Yours from the start."
L["ladders.step.kept"] = "Unlocked"
L["ladders.step.toEarn"] = "Unlocked. One fresh set of %s %s+ earns it."
L["ladders.step.toEarnMark"] = "Unlocked. One Perfect hold of its pose earns it."
L["ladders.step.next"] = "Next: %s %s on %d days (%d of %d)."
L["ladders.step.next.one"] = "Next: %s %s on %d day (%d of %d)."
-- a counted cardio step's climb (slice 42; the apps' ladders.step.next.clean): its clean days so far (a timed step's
-- full moves are the card's Full moves row since slice 52)
L["ladders.step.nextClean"] = "Next: %d of %d clean days."
-- once its days and path are in (slice 53, ns.Proving): the proving window, and after a try that did not pass this
-- session the wait
L["ladders.step.nextProving"] = "Next: %s, a clean 30-s all-out set."
L["ladders.step.climbWaits"] = "No step up yet."
-- a pose step's climb (slice 47, 2026-10-19: any goal pose of the step counts a day; its marking pose alone, "aiming
-- for" its goal pose, until then): the days it takes, those toward it so far and the days again (ns.PoseNeeds' have:
-- its main pose's and Perfect days among them), the apps' ladders.step.next.mark
L["ladders.step.nextMark"] = "Next: any goal pose of your step close enough on %d days (%d of %d)"
L["ladders.step.nextMark.one"] = "Next: any goal pose of your step close enough on %d day (%d of %d)"
-- the same where only Perfect counts a day (ns.MarkNeeds below a risky step): the apps' ladders.step.next.mark.perfect
-- with the game's own count
L["ladders.step.nextPerfect"] = "Next: any goal pose of your step Perfect on %d days (%d of %d)"
L["ladders.step.nextPerfect.one"] = "Next: any goal pose of your step Perfect on %d day (%d of %d)"
-- what those days still lack (slice 47, Tree.PoseShort: the card's climb line and the Ready view's goal note): the
-- days still to come on the step's main pose, by name (pose_anchor_days, two where only Perfect counts), and the day
-- still to come answered Perfect (pose_perfect_days, said only where Good counts too); the apps' ladders.step.short.*
L["ladders.step.shortMain"] = "Of the days left, %d must be on %s."
L["ladders.step.shortMain.one"] = "Of the days left, %d must be on %s."
L["ladders.step.shortPerfect"] = "%d more days must be Perfect."
L["ladders.step.shortPerfect.one"] = "%d more day must be Perfect."
L["ladders.step.shut"] = "This step needs equipment or settings that are off in Your setup."
L["ladders.step.pain"] = "Waits while a pain flag is set on this ladder."
L["ladders.step.waits"] = "Waits for a record of %s on %s."
L["ladders.step.waits.step"] = "Waits until you do %s on your %s ladder."
L["ladders.path.top"] = "Every step of this ladder is yours."
L["sheet.titled"] = "%s, %s"
L["sheet.titled.suffix"] = "%s %s"
L["tip.muscle.quality"] = "%s · step %d on %s"
L["tip.muscle.ladders"] = "its ladders"
L["tip.muscle.untrained"] = "Not trained yet"
L["tip.muscle.week"] = "This week: %s, %s"
L["tip.muscle.moves"] = "%s moves"
L["tip.muscle.moves.one"] = "%s move"
L["tip.muscle.hard"] = "%d hard sets"
L["tip.muscle.hard.one"] = "%d hard set"
L["tip.muscle.fresh"] = "Records come back when you are fresh."
L["tip.muscle.click"] = "Click: the moves for %s"
L["tip.muscle.best"] = "%s · record %s"
-- the muscle's flyout (2026-10-16): how many of its moves the pool holds, and the bar that shows the ones ahead too
L["sheet.flyout.count"] = "%d of %d in your pool"
L["sheet.flyout.all"] = "All %d moves"
L["today.hardSets"] = "%s: %d hard sets today. Records come back when you are fresh."
L["today.vault"] = "Resets %s"
L["today.week.hare"] = "Hare week"
L["today.week.fist"] = "Fist week"
L["today.week.frog"] = "Frog week"
L["today.week.bear"] = "Bear week"
L["today.week.oak"] = "Oak week"
L["vault.label.moves"] = "Moves done"
L["vault.label.records"] = "Records and stars"
L["vault.label.embers"] = "Cardio moves"
L["tip.move.step"] = "%s · step %d · %s"
L["tip.move.stat"] = "+1 %s per record"
L["tip.move.statLearn"] = "Raises %s as you learn it"
L["tip.move.trains"] = "Trains: %s"
-- what a move trains as one line (2026-10-16; "Also:" a line of its own until then)
L["tip.move.trainsAlso"] = "Trains: %s · also %s"
L["tip.move.use"] = "Use:"
L["tip.move.record"] = "Record %s · last %s"
L["tip.move.star"] = "%s at %s"
L["tip.move.familiar"] = "%s: %d of %d days to learned"
L["tip.move.fam"] = "%s"
L["tip.move.locked"] = "Above your step for now: it opens as you climb."
L["tip.move.favourite"] = "Favourite: it comes first in your pumps"
L["tip.move.excluded"] = "Not in your pool now"
-- a makeshift move the player's gear does better (ns.BetterOpen): the move that comes instead
L["tip.move.better"] = "Not in pumps: your setup offers %s instead"
-- a ladder's move done with a weight: its weighted record and last set (Sheet.LoadAmount)
L["tip.ladder.load"] = "With weight: %s · last %s"
-- a move whose right-click menu has Correct a count (slice 44), and its record's clear while it has one (Settings.lua
-- Settings_.RecordHint: the Records tab, the Exercises page)
L["tip.move.correct"] = "Right-click: correct a count"
L["tip.move.correctClear"] = "Right-click: correct a count or clear this record"
-- a move's hints where its tooltip shows (2026-10-16): the flyout's click opens its card on the Talents tab, a
-- right-click its menu
L["tip.move.click"] = "Click: its card on Talents"
L["tip.move.menu"] = "Right-click: favourite or leave out"
L["menu.exclude"] = "Leave this move out"
L["menu.favourite"] = "Mark as favourite"
L["menu.unfavourite"] = "Remove favourite"
L["menu.include"] = "Take this move back in"
L["menu.optIn"] = "Add this move to your pumps"
-- a count corrected (slice 44: its window, Settings.lua Settings_.CorrectWindow), a record cleared or a ladder started
-- over (2026-10-08; Settings.lua asks first)
L["menu.correct"] = "Correct a count..."
L["menu.clear"] = "Clear this move's record"
L["menu.restart"] = "Start the %s ladder over"
L["log.ding"] = "Level-up pump"
L["log.death"] = "Death tax"
L["log.level"] = "QP %d reached"
L["log.paragon"] = "Paragon %d reached"
L["log.quest"] = "Quest %d"
L["log.asked"] = "A pump you asked for"
-- slice 26: a pump done in the app, unfinished moves paid there, and the device each was done on (slice 35: the
-- computer too, the desktop app's own pump or its catch-up). They name the app and the watch by decision, outside the
-- setup, status and help keys (docs/plan.md slice 26 asks for a row's source in the Log): contract/copy/words.json's
-- companions rule excepts them, with the Today tab's and the HUD's app share
L["log.app"] = "A pump in the app"
L["log.catchup"] = "Banked moves"
L["log.device.phone"] = "Done in the app"
L["log.device.watch"] = "Done on your watch"
L["log.device.computer"] = "Done on a computer"
L["log.empty"] = "No pumps yet"
-- the Log's own tab (2026-10-16): each day's moves and pumps (the Saga's count)
L["log.day.moves"] = "%d moves"
L["log.day.moves.one"] = "%d move"
L["log.day.pumps"] = "%d pumps"
L["log.day.pumps.one"] = "%d pump"
L["log.hr.proven"] = "Heart rate: counted, peak %d (+%d)"
L["log.hr.counted"] = "Heart rate: counted"
L["log.hr.unproven"] = "Heart rate: unproven"
L["log.hr.none"] = "Heart rate: no HR"
-- the Log's day headings (the game day, from 04:00), its rows' summary and its tooltip's move results
L["log.today"] = "Today"
L["log.yesterday"] = "Yesterday"
L["log.date"] = "%s, %d %s"
L["log.sum.count"] = "%d/%d"
L["log.sum.banked"] = "Banked"
L["log.sum.later"] = "Done later"
L["log.sum.skipped"] = "Skipped"
L["log.sum.none"] = "Nothing done"
L["log.sum.live"] = "In progress"
L["log.tip.none"] = "No move was done in this pump."
L["log.tip.live"] = "This pump is under way."
L["log.fate.later"] = "done later"
L["log.fate.skipped"] = "skipped"
L["log.result.with"] = "%s · %s"
L["log.result.done"] = "Done"
L["log.result.atLeast"] = "%s or more"
L["log.result.max"] = "max set"
-- a counted set saved Form slipped (slice 42: no clean day, never a best), after its count or ui.result.cleanNone
L["log.result.slipped"] = "form slipped"
-- one saved Not all out (slice 53: no clean day, never a best)
L["log.result.notAllOut"] = "not all out"
L["log.reps"] = "%d reps"
L["log.reps.one"] = "%d rep"
L["titles.tier.first"] = "First steps"
L["titles.tier.deed"] = "Milestones"
L["titles.tier.rare"] = "Rare"
L["titles.tier.epic"] = "Epic"
L["titles.tier.legendary"] = "Legendary"
L["titles.tier.legacy"] = "Legacy"
L["titles.legacy.tip"] = "No longer earned. You keep it."
L["records.hold"] = "longest hold"
L["records.holdValue"] = "%d s · %s"
L["records.mostMoves"] = "most moves in a day"
L["records.mostMoves.value"] = "%s · %s"
L["records.streak"] = "longest streak"
-- the Records tab (2026-10-16): its head's line, the streak cell's tooltip, its empty state
L["records.count"] = "%d moves with a record"
L["records.count.one"] = "%d move with a record"
L["records.streak.now"] = "Now: %d days"
L["records.streak.now.one"] = "Now: %d day"
L["records.empty"] = "No records yet"
L["records.other"] = "Not on a ladder"
-- a recorded move's step and its quality word in its colour (the Records and Deeds tabs, 2026-10-16)
L["records.step"] = "Step %d · %s"
L["records.line"] = "%s to %s since %s"
L["today.moves"] = "Moves done"
L["today.moves.tip"] = "In %d pumps. A pump is the moves one turn-in brings."
L["today.moves.tip.one"] = "In %d pump. A pump is the moves one turn-in brings."
-- slice 26: the app's share of the day's moves and records (named by decision, as the Log's keys above)
L["today.app"] = "%d of them in the app."
L["today.wins"] = "Wins"
L["today.wins.tip"] = "Records, +1 on your last fresh set, first tries, stars and unlocks."
L["today.records"] = "New records"
L["today.stars"] = "New stars"
L["today.xp"] = "XP today"
L["today.region.sets"] = "Max sets today: %d"
L["today.empty"] = "No pumps yet today"
L["today.sets"] = "%s %d"
-- the streamer kit (Streamer.lua, slice 23): the day's tally and the day card (the session card until the Saga,
-- 2026-10-15), the game day's numbers
L["streamer.tally.title"] = "QuestPump · the day's tally"
L["streamer.tally.tip"] = "Drag it where your stream needs it. Right-click puts it back."
L["streamer.pumps"] = "Pumps"
L["streamer.records"] = "Records"
L["streamer.stars"] = "Stars"
L["streamer.cardio"] = "Cardio"
L["streamer.finishers"] = "Finishers"
L["streamer.unfinished"] = "Banked"
L["streamer.reps"] = "Reps: %s"
L["streamer.reps.item"] = "%s %d"
-- the day card (Streamer.lua; the Saga's D board, 2026-10-15): its title, the character's QP level after its name
-- (past the cap with its Paragon rank), the day's moves and their word, the best day with the old best and its date,
-- or the day's quality word (the player's own days alone, never awarded: safety.md hazard 12), the stats row (the XP's
-- "+1,240", "QP 23 > 24" with the arrow, past the cap "Paragon 2 > 3", its word where the client lacks the arrow), the
-- strip of the day's pumps from the day's turn to the next and its legend, the buckets of the day, the moves by
-- region, the unlocks' count beside the records' (streamer.share.records), a record's line (its record at the day's
-- start, or none: a first one) and an unlock's
L["streamer.card.title"] = "Today"
L["streamer.card.level"] = "· QP %d"
L["streamer.card.levelParagon"] = "· QP %d · Paragon %d"
L["streamer.card.moves"] = "moves"
L["streamer.card.moves.one"] = "move"
L["streamer.card.best"] = "Best day ever"
L["streamer.card.old"] = "Old best %s · %s"
L["streamer.card.quality.uncommon"] = "Uncommon day"
L["streamer.card.quality.rare"] = "Rare day"
L["streamer.card.quality.epic"] = "Epic day"
L["streamer.card.pumps"] = "pumps"
L["streamer.card.pumps.one"] = "pump"
L["streamer.card.streak"] = "day streak"
L["streamer.card.xp"] = "+%s"
L["streamer.card.xpWord"] = "XP"
L["streamer.card.qp"] = "QP"
L["streamer.card.paragon"] = "Paragon"
L["streamer.card.to"] = "to"
L["streamer.card.strip"] = "Your pumps, %s to %s"
L["streamer.card.game"] = "in the game"
L["streamer.card.away"] = "in the app"
L["streamer.card.time.morning"] = "Morning"
L["streamer.card.time.afternoon"] = "Afternoon"
L["streamer.card.time.evening"] = "Evening"
L["streamer.card.time.night"] = "Night"
L["streamer.card.regions"] = "Moves by region"
L["streamer.card.unlocks"] = "%d unlocks"
L["streamer.card.unlocks.one"] = "%d unlock"
L["streamer.card.record"] = "Record, was %s"
L["streamer.card.record.first"] = "New record"
-- a loaded move's record: its weight (the set's, as the Log writes it, kg or kg each) before its record at the day's
-- start
L["streamer.card.kg"] = "%s kg"
L["streamer.card.kgEach"] = "%s kg each"
L["streamer.card.recordAt"] = "%s, was %s"
L["streamer.card.recordAt.first"] = "%s, new record"
-- a bucket's word and its total over its span ("Morning 14"), and the levels the day went through ("23 > 24", the arrow
-- or streamer.card.to between them)
L["streamer.card.bucket"] = "%s %s"
L["streamer.card.levels"] = "%d %s %d"
L["streamer.card.unlocked"] = "Unlocked · %s"
L["streamer.card.share"] = "Share line"
L["streamer.card.share.tip"] = "Puts a line with the day's pumps, moves, records and stars in the chat box. You send "
	.. "it."
-- why Share line is greyed on a day with no move yet (the day card's and the Today tab's)
L["streamer.card.share.none"] = "No moves yet today, so there is nothing to share."
L["streamer.card.shot"] = "Screenshot"
L["streamer.card.shot.tip"] = "Saves a picture of your screen, this card on it, in the game's Screenshots folder."
L["streamer.shot.key"] = "Press %s to take a screenshot with this card on the screen."
L["streamer.shot.noKey"] = "Bind a key to %s (Options, Keybindings) to take a screenshot with this card on the "
	.. "screen."
L["streamer.share.pumps"] = "%d pumps"
L["streamer.share.pumps.one"] = "%d pump"
L["streamer.share.moves"] = "%d moves"
L["streamer.share.moves.one"] = "%d move"
L["streamer.share.records"] = "%d new records"
L["streamer.share.records.one"] = "%d new record"
L["streamer.share.stars"] = "%d new stars"
L["streamer.share.stars.one"] = "%d new star"
L["share.day"] = "My day: %s [QuestPump]"
L["deeds.progress.count"] = "%s/%s"
L["deeds.progress.yards"] = "%s/%s miles"
L["deeds.progress.seconds"] = "%s/%s min"
-- the Deeds tab with the titles in it (2026-10-16): the head's line, an earned row's state, the worn one's badge, its
-- click's hint, and the ladder tops folded under one More in Legendary
L["deeds.worn"] = "Your title: %s"
L["deeds.worn.none"] = "No title worn"
L["deeds.earned"] = "Earned"
L["deeds.wornBadge"] = "Your title"
L["deeds.click.wear"] = "Click: wear this title"
L["deeds.click.off"] = "Click: take it off"
L["deeds.tops"] = "Ladder tops (%d)"
-- the Mastered steps row under Milestones (slice 42; the apps' ladders.colours.mastered): no title and never worn, its
-- count at the right, its tooltip the steps mastered by ladder, newest first
L["deeds.mastered"] = "Mastered steps"
L["deeds.mastered.line"] = "Steps where every move your setup offers reached its top."
L["deeds.mastered.step"] = "%s step %d · %s · %s"
L["deeds.mastered.more"] = "+%d more steps"
L["deeds.mastered.more.one"] = "+%d more step"
L["vault.moves"] = "%d moves"
L["vault.records"] = "%d records or stars"
L["vault.embers"] = "%d cardio moves"

-- the titles (addon-internal: the game's own titles are never touched) and what earns each
L["title.initiate"] = "the Initiate"
L["title.tested"] = "the Tested"
L["title.recordBreaker"] = "the Record Breaker"
L["title.circuitRider"] = "Circuit Rider"
L["title.hubHopper"] = "Hub Hopper"
L["title.cleanSlate"] = "Clean Slate"
L["title.steadfast"] = "the Steadfast"
L["title.ding"] = "Ding!"
L["title.starryEyed"] = "Starry-eyed"
L["title.relentless"] = "the Relentless"
L["title.wallSitter"] = "Wall-sitter"
L["title.ironLegged"] = "Iron-legged"
L["title.unbroken"] = "the Unbroken"
L["title.unwavering"] = "the Unwavering"
L["title.unfailing"] = "the Unfailing"
L["title.evergreen"] = "the Evergreen"
L["title.undying"] = "the Undying"
L["title.sprinter"] = "Sprinter"
L["title.longRoad"] = "of the Long Road"
L["title.wanderer"] = "the Wanderer"
L["title.duelist"] = "the Duelist"
L["title.paragon"] = "the Paragon"
L["title.ascendant"] = "the Ascendant"
L["title.ageless"] = "the Ageless"
L["title.eternal"] = "the Eternal"
L["title.kindled"] = "the Kindled"
L["title.climber1"] = "Push Climber"
L["title.climber2"] = "Squat Climber"
L["title.climber3"] = "Pull Climber"
L["title.climber4"] = "Hinge Climber"
L["title.climber5"] = "Core Climber"
L["title.climber6"] = "Cardio Climber"
L["title.climber7"] = "Wall-sit Climber"
L["title.mighty"] = "the Mighty"
L["title.swift"] = "the Swift"
L["title.poised"] = "the Poised"
L["title.tireless"] = "the Tireless"
L["title.serene"] = "the Serene"
L["deed.initiate"] = "Finish your first pump."
L["deed.tested"] = "Set a first record on five ladders."
L["deed.recordBreaker"] = "Beat a record."
L["deed.circuitRider"] = "Finish a pump of three moves."
L["deed.hubHopper"] = "Finish a pump that joined two or more turn-ins."
L["deed.cleanSlate"] = "Finish every banked move."
L["deed.steadfast"] = "Pump on seven days."
L["deed.ding"] = "Finish the pump of a level-up."
L["deed.starryEyed"] = "Reach three stars on a step."
L["deed.relentless"] = "Finish 500 pumps."
L["deed.wallSitter"] = "Hold 30 minutes of wall sit in all."
L["deed.ironLegged"] = "Do 1,000 squats in all."
L["deed.unbroken"] = "Keep a 30-day streak."
L["deed.unwavering"] = "Keep a 100-day streak."
L["deed.unfailing"] = "Keep a 365-day streak."
L["deed.evergreen"] = "Keep a 730-day streak."
L["deed.undying"] = "Keep a 1,825-day streak."
L["deed.sprinter"] = "Finish 50 all-out windows."
L["deed.longRoad"] = "Finish 100 cardio moves on travel quests."
L["deed.wanderer"] = "Cover 100 miles of Azeroth on foot."
L["deed.duelist"] = "Win a QuestPump duel."
L["deed.paragon"] = "Reach Paragon 10."
L["deed.ascendant"] = "Reach Paragon 25."
L["deed.ageless"] = "Reach Paragon 50."
L["deed.eternal"] = "Reach Paragon 100."
L["deed.kindled"] = "Finish 15 cardio moves in a week."
L["deed.climber1"] = "Climb a step on the push ladder."
L["deed.climber2"] = "Climb a step on the squat ladder."
L["deed.climber3"] = "Climb a step on the pull-up or the row."
L["deed.climber4"] = "Climb a step on the hinge ladder."
L["deed.climber5"] = "Climb a step on the core or the side core."
L["deed.climber6"] = "Climb a step on the cardio ladder."
L["deed.climber7"] = "Climb a step on the wall sit ladder."
L["deed.mighty"] = "Unlock an Epic step on a Strength ladder."
L["deed.swift"] = "Unlock an Epic step on an Agility ladder: cardio or power."
L["deed.poised"] = "Unlock an Epic step in balance, arm balances or the handstand."
L["deed.tireless"] = "Unlock an Epic step on a Stamina ladder."
L["deed.serene"] = "Unlock an Epic step in stretching."
L["title.proven"] = "the Proven"
L["deed.proven"] = "Unlock a Rare step on any ladder."
L["title.allRounder"] = "the All-Rounder"
L["deed.allRounder"] = "Unlock a Rare step in every stat."
L["title.exalted"] = "the Exalted"
L["deed.exalted"] = "Unlock an Epic step in every stat."
L["title.legendary"] = "the Legendary"
L["deed.legendary"] = "Unlock Legendary steps on three ladders."
L["title.handOfIron"] = "Hand of Iron"
L["deed.handOfIron"] = "Unlock the top of the Push ladder: the one-arm push-up with your feet together."
L["title.masterOfTheBar"] = "Master of the Bar"
L["deed.masterOfTheBar"] = "Unlock the top of the Pull-up ladder: the one-arm pull-up."
L["title.flagbearer"] = "Flagbearer"
L["deed.flagbearer"] = "Unlock the top of the Side core ladder: the human flag."
L["title.defierOfGravity"] = "Defier of Gravity"
L["deed.defierOfGravity"] = "Unlock the top of the Row ladder: the front lever."
L["title.worldlifter"] = "Worldlifter"
L["deed.worldlifter"] = "Unlock the top of the Handstand ladder: the handstand push-up, free of the wall."
L["title.wingsOfStone"] = "Wings of Stone"
L["deed.wingsOfStone"] = "Unlock the top of the Planche ladder: the straddle planche."
L["title.dragonstrider"] = "Dragonstrider"
L["deed.dragonstrider"] = "Unlock the top of the Squat ladder: the dragon pistol."
L["title.dragonbane"] = "Dragonbane"
L["deed.dragonbane"] = "Unlock the top of the Core ladder: the dragon flag."
L["title.championOfTheNorth"] = "Champion of the North"
L["deed.championOfTheNorth"] = "Unlock the top of the Hinge ladder: the full Nordic curl."
L["title.stormvaulter"] = "Stormvaulter"
L["deed.stormvaulter"] = "Unlock the top of the Power ladder: the muscle-up."
L["title.stormrunner"] = "Stormrunner"
L["deed.stormrunner"] = "Unlock the top of the Cardio ladder: tuck-jump burpees as its all-out form."

-- the HUD line and its tooltip
L["hud.qp"] = "QP %d"
L["hud.qp.moves"] = "%d moves today"
L["hud.qp.moves.one"] = "1 move today"
L["hud.qp.streak"] = "%d-day streak"
L["hud.off"] = "QuestPump is off"
L["tip.hud.off.click"] = "Click: open settings"
-- the embers line under the QP line (slice 55): today's embers of three, its click the cardio step's card
L["hud.embers"] = "Cardio today %d of %d"
L["tip.hud.embers.click"] = "Click: your cardio step"
-- this character's pause (Lc.Pause): the QP line's right-click and the pump's Pause button, the line while it holds,
-- /qp pause and /qp resume
L["tip.hud.pause"] = "Right-click: pause QuestPump on this character"
L["hud.pause.title"] = "Pause QuestPump on this character"
L["hud.pause.hour"] = "For an hour"
L["hud.pause.fourHours"] = "For 4 hours"
L["hud.pause.day"] = "Until %02d:00"
L["hud.pause.char"] = "Off on this character"
L["hud.pause.group"] = "While grouped"
L["hud.paused.until"] = "Paused until %s"
L["hud.paused.char"] = "Off on this character"
L["hud.paused.group"] = "Paused while grouped"
L["hud.paused.title"] = "QuestPump paused"
L["hud.paused.tip"] = "Turn-ins cost nothing and owe nothing while it is paused. Banked moves wait, and a pump "
	.. "you ask for (/qp pump) still runs."
L["hud.paused.click"] = "Click: resume now"
L["chat.paused"] = "Paused on this character: this turn-in costs nothing. /qp resume turns QuestPump back on."
L["cmd.pause.help"] = "/qp pause [hour|4h|char]: pause this character until 04:00, for an hour, for 4 hours, or "
	.. "until /qp resume"
L["cmd.pause.day"] = "Paused until %02d:00: turn-ins cost nothing on this character. /qp resume to come back sooner."
L["cmd.pause.hour"] = "Paused for an hour: turn-ins cost nothing on this character."
L["cmd.pause.fourHours"] = "Paused for 4 hours: turn-ins cost nothing on this character."
L["cmd.pause.char"] = "Off on this character until /qp resume: turn-ins cost nothing here."
L["cmd.resume.help"] = "/qp resume: turn QuestPump back on for this character"
L["cmd.resume.done"] = "QuestPump is on again for this character."
L["cmd.focus.help"] = "/qp focus [balanced|strength|endurance|flexibility]: show or set what your pumps lean toward"
L["cmd.focus.set"] = "Focus: %s. The next pump follows it."
L["cmd.focus.usage"] = "A focus is balanced, strength, endurance or flexibility."
-- The focus (David, 2026-10-03): what the day plan leans toward, asked at QP 10 and changed at any time
L["focus.balanced"] = "Balanced"
L["focus.balanced.line"] = "Strength first. Cardio, stretching and yoga every day."
L["focus.strength"] = "Strength"
L["focus.strength.line"] = "More strength moves. Cardio, stretching and yoga stay in every day."
L["focus.endurance"] = "Endurance"
L["focus.endurance.line"] = "More cardio. Your strength sets keep coming."
L["focus.flexibility"] = "Flexibility"
L["focus.flexibility.line"] = "More stretching and yoga. Strength and cardio stay in every day."
-- the focus window's title bar (the kit's medium window, 2026-10-16) and its head, at the ask the level it came at
L["ui.focus.window"] = "QuestPump"
L["ui.focus.title"] = "Your focus"
L["ui.focus.titleAsk"] = "QP %d: pick your focus"
L["ui.focus.recommended"] = "%s (recommended)"
L["ui.focus.keep"] = "Keep Balanced"
L["ui.focus.change"] = "Click: change your focus"
-- a focus tile's tooltip: the click's hint, or on the focus in force the fact in its place
L["ui.focus.choose"] = "Click: choose this focus"
L["ui.focus.now"] = "Your focus now."
L["tip.hud.title"] = "QP %d · %s · %s focus"
L["tip.hud.xp"] = "XP %s/%s (%d%%)"
L["tip.hud.xpParagon"] = "XP %s/%s toward Paragon %d (%d%%)"
L["tip.hud.rested"] = "Rested %s XP: your XP counts double until it is used."
L["tip.hud.paragon"] = "Paragon %d"
-- slice 26: the app's share of the QP line's moves today (named by decision, as the Log's keys)
L["tip.hud.app"] = "In the app today: %d moves."
L["tip.hud.app.one"] = "In the app today: %d move."
L["tip.hud.vaultSum"] = "This week's vault: %d of %d · resets %s"
L["tip.hud.streak"] = "Streak %d days · best %d · rest tokens %d"
L["tip.hud.streak.one"] = "Streak %d day · best %d · rest tokens %d"
L["tip.hud.click"] = "Click: your character sheet"

-- the toasts, the level-up and the lines a right-click shares (the player sends them; nothing is sent by itself)
L["toast.level"] = "QP %d"
L["toast.levelSub"] = "Level up"
L["toast.paragon"] = "Paragon %d"
L["toast.paragonSub"] = "New Paragon rank"
L["toast.record"] = "New record: %s"
L["toast.title"] = "Title earned: %s"
L["toast.vault"] = "Weekly vault: %s"
L["toast.vaultAll"] = "Weekly vault complete"
-- a game day's third ember (slice 55), once a game day, over the week's embers
L["toast.embers"] = "Cardio done today"
L["toast.embers.sub"] = "%d cardio moves this week"
L["toast.xp"] = "+%d XP"
L["toast.welcome"] = "Welcome back!"
L["toast.streak"] = "Streak: day %d"
L["toast.streak.new"] = "Day 1 of a new streak"
L["toast.milestone"] = "A %s-day streak!"
L["toast.milestone.years"] = "A %d-year streak!"
L["toast.token"] = "Rest token earned: a day off keeps your streak"
L["toast.rested"] = "%d rest tokens kept your streak going"
L["toast.rested.one"] = "A rest token kept your streak going"
-- the day's one "New best day" (Stats.lua's bestDay, 2026-10-15): the day passed the player's own best in moves, after
-- their first week; shown, never awarded (no XP), and its right-click's chat line counts moves, never a typed count
L["toast.bestDay"] = "New best day"
L["toast.bestDay.sub"] = "%s moves"
L["fanfare.level"] = "QP %d"
L["fanfare.levelSub"] = "Level up"
L["fanfare.paragon"] = "Paragon %d"
L["fanfare.paragonSub"] = "New Paragon rank"
L["share.record"] = "+1 on my best: %s [QuestPump]"
L["share.star"] = "%d stars on %s [QuestPump]"
L["share.star.one"] = "%d star on %s [QuestPump]"
L["share.first"] = "First record on %s [QuestPump]"
L["share.title"] = "New title: %s [QuestPump]"
L["share.paragon"] = "Reached Paragon %d [QuestPump]"
L["share.unlock"] = "Unlocked %s (%s) [QuestPump]"
L["share.bestDay"] = "My best day: %s moves [QuestPump]"
-- a streak milestone's toast and its banner row (slice 45): its days, or a whole number of years from two, as it says
L["share.milestone"] = "My %s-day streak [QuestPump]"
L["share.milestone.years"] = "My %d-year streak [QuestPump]"
-- with the count, behind Settings > Social's checkbox (never by default)
L["share.record.count"] = "+1 on my best: %s %s [QuestPump]"
L["share.first.count"] = "First record on %s: %s [QuestPump]"

-- what a move earns: the Ready view's rewards line and the recorded Result's rows (Panel.lua, 2026-10-16)
L["ui.rewards.title"] = "Rewards"
L["ui.rewards.gain"] = "+%d XP"
L["ui.rewards.stat"] = "+%d %s"
L["ui.rewards.onRecord"] = "on a new record"
L["ui.rewards.star"] = "%s at %s"
L["ui.rewards.pattern"] = "a new pattern"
L["ui.rewards.practice"] = "this week's practice"
L["ui.rewards.join"] = ", "
L["ui.rewards.missed"] = "– %s"
L["ui.rewards.xpTitle"] = "Experience, line by line"
L["ui.rewards.xpLine"] = "+%d (%d before the streak)"
L["ui.rewards.xpLineRested"] = "+%d (%d before the streak, +%d rested)"
L["unit.seconds"] = "%d s"
L["unit.countIn"] = "%d in %d s"
-- a count as a goal ("20+"), and a loaded move's top with the weight its climb asks (ns.ClimbKg, 2026-10-12: a lighter
-- set climbs nothing), in whole kg, each hand's for a weight in each hand
L["amount.goal"] = "%s+"
L["amount.climb"] = "%s with %d kg or more"
L["amount.climbEach"] = "%s with %d kg or more in each hand"
L["xp.reason.move"] = "The move"
L["xp.reason.pattern"] = "A new pattern"
L["xp.reason.star"] = "A star"
L["xp.reason.rung"] = "A step"
L["xp.reason.firstPump"] = "The day's first pump"
L["xp.reason.welcome"] = "Welcome back"
L["xp.reason.ding"] = "The level-up pump"
L["xp.reason.vault"] = "A weekly vault goal"

-- the settings
L["settings.name"] = "QuestPump"
L["settings.general"] = "General"
-- the General page (2026-10-16): its sections, a greyed row's reason, the Companions row's click and its line on a
-- save the game did not load (red beside the cross: a state not done) with its tooltip, and a select's click (the Data
-- page's selects' too)
L["settings.section.pumps"] = "Pumps"
L["settings.section.sound"] = "Voice and sound"
L["settings.section.screen"] = "On the screen"
L["settings.pause.offAll"] = "Off on every character: nothing to pause."
L["settings.needsVoice"] = "Needs Voice on."
L["settings.needsVoicePack"] = "Needs QuestPump Voice, a download of its own."
L["settings.companions.click"] = "Click: open the setup"
L["settings.click.choose"] = "Click: choose"
L["settings.beta"] = "Your progress may not be saved."
L["settings.beta.tip"] = "The game did not load your saved data. Export on the Data page keeps a copy."
L["settings.enabled"] = "On for every character"
L["settings.enabled.tip"] = "Off on every character: turn-ins cost nothing and QuestPump sleeps. A pump you ask for "
	.. "still runs."
L["settings.focus"] = "Focus"
L["settings.focus.tip"] = "What your pumps lean toward: more strength, more cardio, or more stretching and yoga. "
	.. "Every focus keeps a bit of everything, and it counts for every character."
L["settings.pace"] = "Moves per quest"
-- each pace's line in its tooltip (TIP_ARGS)
L["settings.pace.tip"] = "How many moves each quest asks, from the next quest you turn in.\nLight: %s\nBalanced: %s\n"
	.. "Extreme: %s"
L["settings.pace.light.meta"] = "1 move per quest."
L["settings.pace.balanced.meta"] = "1 to 3 moves, more for harder quests, and often a record try first."
L["settings.pace.extreme.meta"] = "2 to 4 moves, more for harder quests, and often a record try first."
L["settings.pace.light"] = "Light"
L["settings.pace.balanced"] = "Balanced"
L["settings.pace.extreme"] = "Extreme"
L["settings.pause"] = "Pause this character"
L["settings.pause.tip"] = "Pause QuestPump on this character, for playing with friends: turn-ins meanwhile cost "
	.. "nothing and owe nothing. Banked moves wait. A pump you ask for still runs."
L["settings.pause.on"] = "Not paused"
L["settings.pause.hour"] = "Paused for an hour"
L["settings.pause.fourHours"] = "Paused for 4 hours"
L["settings.pause.day"] = "Paused until %02d:00"
L["settings.pause.char"] = "Off on this character"
L["settings.pause.group"] = "Paused while grouped"
L["settings.speech"] = "Voice"
L["settings.speech.tip"] = "A recorded voice reads the move and its cue."
L["settings.callouts"] = "Callouts"
L["settings.callouts.tip"] = "After the move's name, one recorded line when it matters: a record attempt, a streak day."
-- the General page and Your setup's How hard page (Speech.lua's rep count, which the pump's screen shows and puts in
-- the box after Done, Panel.lua; docs/safety.md, the rep count)
L["settings.countReps"] = "Count reps aloud"
L["settings.countReps.tip"] = "With Voice on, the voice counts each rep as the figure does it, and the screen shows "
	.. "the count. Count out loud with it, and go slower whenever you need. Done puts the count in the box: change it "
	.. "to the reps you did. A move done one side at a time counts each side from 1 until you press Switch sides."
L["settings.sounds"] = "Sounds"
L["settings.sounds.tip"] = "Blizzard sound cues for the countdown, records and levels."
L["settings.toasts"] = "Toasts"
L["settings.toasts.tip"] = "Pop-ups for records, stars, titles, levels and the weekly vault."
L["settings.hud"] = "HUD line"
L["settings.hud.tip"] = "Under the quest tracker: your level, today's moves, your streak and any banked moves."
-- when pumps ask (Events.lua Lc.AskWhen, slice 70): General's row and Your setup's two tiles, one tooltip for both
L["settings.askWhen"] = "When pumps ask"
L["settings.askWhen.tip"] = "At the turn-in: each quest you turn in asks for its pump at once.\nOn my next flight: "
	.. "your pumps wait and open one after another when you fly. What has not opened when you land is banked."
L["settings.askWhen.turnin"] = "At the turn-in"
L["settings.askWhen.flight"] = "On my next flight"
L["settings.instances"] = "Pumps in dungeons and raids"
L["settings.instances.tip"] = "Off: turn-ins inside wait as banked moves until you leave."
L["settings.figureAnimate"] = "Animate the move figure"
L["settings.figureAnimate.tip"] = "Off shows the move's key pose, still."
L["settings.figureColor"] = "Figure colour"
L["settings.figureColor.tip"] = "The colour of the figure's sash and crest."
L["settings.figureColor.class"] = "Your class colour"
L["settings.figureColor.crimson"] = "Crimson"
-- The move figure (Figure.lua): its one line when the pack is missing or older, and the phase captions of the
-- New-move view (at most three words each; the motion files contract/art/<move>.json name the phases).
-- /qp qr (QR.lua): no app and no URL named (Blizzard add-on policy rule 4)
-- its window's title bar and the head's name
L["qr.bar"] = "QuestPump"
L["qr.title"] = "Sheet snapshot"
L["qr.none"] = "No snapshot yet: the sheet has not loaded."
L["qr.large"] = "Your sheet is too big for one snapshot code."
L["cmd.qr.help"] = "/qp qr: a snapshot of your sheet as a QR code"
L["figure.missing"] = "Move animations missing: QuestPump Figures is a download of its own. /qp status says more."
L["figure.outdated"] = "Move animations are off: QuestPump Figures is made for another game version. Update it."
-- The New-move view's fault pair (slice 22): the words under the check and the cross; each fault pose's caption is a
-- figure.fault key (a form cue, never a health claim: the content lint reads it).
L["figure.pair.right"] = "Right"
L["figure.pair.wrong"] = "Wrong"
L["figure.fault.arm_bends"] = "Straight arm bends"
L["figure.fault.arms_lift"] = "Arms lift the bell, hips stay back"
L["figure.fault.arms_short"] = "Arms stop at shoulder height"
L["figure.fault.back_knee_bends"] = "Back knee bends"
L["figure.fault.back_rounds"] = "Back rounds"
L["figure.fault.banana_back"] = "Back arches like a banana"
L["figure.fault.bar_at_chin"] = "Bar stops at the chin"
L["figure.fault.body_spins"] = "Body spins round"
L["figure.fault.body_twists"] = "Body twists toward the bent arm"
L["figure.fault.chest_caves"] = "Chest caves in"
L["figure.fault.chest_leans"] = "Chest leans onto the front thigh"
L["figure.fault.chest_short"] = "Chest stops short of the bar"
L["figure.fault.chest_turns"] = "Chest turns toward the bar hand"
L["figure.fault.chin_below_bar"] = "Chin stays below the bar"
L["figure.fault.chin_leads"] = "Chin leads, head tips back"
L["figure.fault.elbow_low"] = "Elbow drops below the shoulder"
L["figure.fault.elbows_bend"] = "Elbows bend"
L["figure.fault.elbows_flare"] = "Elbows flare out to the sides"
L["figure.fault.foot_on_knee"] = "Foot rests on the knee"
L["figure.fault.front_knee_bends"] = "Front knee bends"
L["figure.fault.hands_push_thighs"] = "Hands push off the thighs"
L["figure.fault.head_dodges"] = "Head tilts away from the bell"
L["figure.fault.head_drops"] = "Head drops, eyes look down"
L["figure.fault.head_drops_back"] = "Head drops back"
L["figure.fault.head_lifts"] = "Head lifts instead of hanging"
L["figure.fault.head_pulled_forward"] = "Hands pull the head forward"
L["figure.fault.heel_drops"] = "Standing heel drops to the floor"
L["figure.fault.heel_lifts"] = "Heel lifts off the floor"
L["figure.fault.heels_lift"] = "Heels lift off the floor"
L["figure.fault.hips_bend"] = "Body folds at the hips"
L["figure.fault.hips_drift_back"] = "Hips drift back"
L["figure.fault.hips_drop"] = "Hips drop below the shoulders"
L["figure.fault.hips_lift"] = "Hips lift off the floor"
L["figure.fault.hips_pike"] = "Hips pike up high"
L["figure.fault.hips_sag"] = "Hips sag toward the floor"
L["figure.fault.hips_to_wall"] = "Hips sag toward the wall"
L["figure.fault.hips_twist"] = "Hips twist open"
L["figure.fault.knee_caves"] = "Knee caves inward"
L["figure.fault.knee_drifts_forward"] = "Bent knee drifts forward"
L["figure.fault.knee_drops"] = "Lifted knee drops below hip level"
L["figure.fault.knee_lifts"] = "Top knee lifts off the bottom knee"
L["figure.fault.knee_past_toes"] = "Knee drifts past the toes"
L["figure.fault.knee_too_straight"] = "Standing knee too straight"
L["figure.fault.knees_bend"] = "Knees bend"
L["figure.fault.knees_tuck"] = "Knees tuck up in the air"
L["figure.fault.leans_back"] = "Body leans back"
L["figure.fault.leg_drops"] = "Straight leg sinks below level"
L["figure.fault.legs_drop"] = "Legs drop below level"
L["figure.fault.long_knee_lifts"] = "Long leg bends, knee lifts"
L["figure.fault.low_back_arches"] = "Lower back arches"
L["figure.fault.neck_takes_weight"] = "Neck takes the weight"
L["figure.fault.no_lean"] = "Shoulders stay over the hands"
L["figure.fault.one_elbow_first"] = "One elbow comes over first"
L["figure.fault.thigh_drops"] = "Back thigh drops below level"
L["figure.fault.tips_forward"] = "Tips forward onto the toes"
L["figure.old"] = "Some move animations are missing: update QuestPump Figures."
L["figure.phase.all_out"] = "All out"
L["figure.phase.arm_long"] = "Arm long"
L["figure.phase.arm_straight"] = "Arm straight"
L["figure.phase.arm_to_ceiling"] = "Arm to ceiling"
L["figure.phase.arms_down"] = "Arms down"
L["figure.phase.arms_long"] = "Arms long"
L["figure.phase.arms_open"] = "Arms open"
L["figure.phase.arms_straight"] = "Arms straight"
L["figure.phase.arms_up"] = "Arms up"
L["figure.phase.around_head"] = "Around the head"
L["figure.phase.back_flat"] = "Back flat"
L["figure.phase.back_in"] = "Back in"
L["figure.phase.back_on_wall"] = "Back on wall"
L["figure.phase.back_to_chin"] = "Fists to chin"
L["figure.phase.ball_high"] = "Ball high"
L["figure.phase.bend_front_knee"] = "Bend front knee"
L["figure.phase.bottom"] = "Bottom"
L["figure.phase.breathe"] = "Breathe"
L["figure.phase.breathe_out"] = "Breathe out"
L["figure.phase.breathe_out_sink"] = "Breathe out, sink"
L["figure.phase.bridge"] = "Hips up"
L["figure.phase.catch"] = "Catch"
L["figure.phase.chair_height"] = "Chair height"
L["figure.phase.chest_sinks"] = "Chest sinks"
L["figure.phase.chest_to_bar"] = "Chest to bar"
L["figure.phase.chest_to_edge"] = "Chest to edge"
L["figure.phase.chest_to_frame"] = "Chest to frame"
L["figure.phase.chest_to_hands"] = "Chest to hands"
L["figure.phase.chest_to_wall"] = "Chest to wall"
L["figure.phase.chest_up"] = "Chest up"
L["figure.phase.chin_over_bar"] = "Chin over bar"
L["figure.phase.chop_down"] = "Chop down"
L["figure.phase.chop_low"] = "Chop low"
L["figure.phase.circle_back"] = "Circle back"
L["figure.phase.circle_forward"] = "Circle forward"
L["figure.phase.circle_wrists"] = "Circle wrists"
L["figure.phase.clap"] = "Clap"
L["figure.phase.clasp_behind"] = "Clasp behind"
L["figure.phase.crawl_back"] = "Crawl back"
L["figure.phase.crawl_forward"] = "Crawl forward"
L["figure.phase.curl"] = "Curl"
L["figure.phase.dead_hang"] = "Dead hang"
L["figure.phase.down_again"] = "Down again"
L["figure.phase.drive_up"] = "Drive up"
L["figure.phase.drive"] = "Drive"
L["figure.phase.duck"] = "Duck"
L["figure.phase.ease"] = "Ease in"
L["figure.phase.ease_back"] = "Ease back"
L["figure.phase.ease_up"] = "Ease up"
L["figure.phase.elbows_high"] = "Elbows high"
L["figure.phase.eyes_closed"] = "Eyes closed"
L["figure.phase.feet_wide"] = "Feet wide"
L["figure.phase.fingertip_push"] = "Fingertip push"
L["figure.phase.float"] = "Float"
L["figure.phase.fold_forward"] = "Fold forward"
L["figure.phase.foot_down"] = "Other foot down"
L["figure.phase.foot_on_the_wall"] = "Foot on wall"
L["figure.phase.foot_to_calf"] = "Foot to calf"
L["figure.phase.foot_up"] = "Foot up"
L["figure.phase.forearms_down"] = "Forearms down"
L["figure.phase.front_leg_up"] = "Front leg up"
L["figure.phase.forearm_on_frame"] = "Forearm on frame"
L["figure.phase.halfway_down"] = "Halfway down"
L["figure.phase.halfway_up"] = "Halfway up"
L["figure.phase.hand_back"] = "Hand back"
L["figure.phase.hand_down"] = "Hand down"
L["figure.phase.hand_to_head"] = "Hand to head"
L["figure.phase.hands_and_knees"] = "Hands and knees"
L["figure.phase.hands_at_chest"] = "Hands at chest"
L["figure.phase.hands_behind"] = "Hands behind"
L["figure.phase.hands_by_ears"] = "Hands by ears"
L["figure.phase.hands_down"] = "Hands down"
L["figure.phase.hands_off"] = "Hands off"
L["figure.phase.hands_on_back"] = "Hands on back"
L["figure.phase.hands_on_desk"] = "Hands on desk"
L["figure.phase.hands_on_edge"] = "Hands on edge"
L["figure.phase.hands_slide_down"] = "Hands slide down"
L["figure.phase.hands_to_calves"] = "Hands to calves"
L["figure.phase.hands_to_heels"] = "Hands to heels"
L["figure.phase.hands_to_wall"] = "Hands to wall"
L["figure.phase.hands_together"] = "Hands together"
L["figure.phase.hang"] = "Hang"
L["figure.phase.hang_back"] = "Hang back"
L["figure.phase.head_down"] = "Head down"
L["figure.phase.heel_down"] = "Heel down"
L["figure.phase.heel_on_the_bed"] = "Heel on bed"
L["figure.phase.heel_to_buttock"] = "Heel to buttock"
L["figure.phase.heel_to_toe"] = "Heel to toe"
L["figure.phase.heels_back"] = "Heels back"
L["figure.phase.heels_down"] = "Heels down"
L["figure.phase.heels_out"] = "Heels out"
L["figure.phase.hike_back"] = "Hike back"
L["figure.phase.hips_back"] = "Hips back"
L["figure.phase.hips_down"] = "Hips down"
L["figure.phase.hips_forward"] = "Hips forward"
L["figure.phase.hips_forward_chest_up"] = "Hips forward"
L["figure.phase.hips_high"] = "Hips high"
L["figure.phase.hips_level"] = "Hips level"
L["figure.phase.hips_still"] = "Hips still"
L["figure.phase.hips_up"] = "Hips up"
L["figure.phase.hold"] = "Hold"
L["figure.phase.hold_ankles"] = "Hold the ankles"
L["figure.phase.hold_calves"] = "Hold the calves"
L["figure.phase.hold_the_shin"] = "Hold the shin"
L["figure.phase.hook_toes"] = "Hook the toes"
L["figure.phase.hop_sideways"] = "Hop sideways"
L["figure.phase.jump"] = "Jump"
L["figure.phase.jump_back"] = "Jump back"
L["figure.phase.jump_forward"] = "Jump forward"
L["figure.phase.jump_in"] = "Jump in"
L["figure.phase.kick_and_lift"] = "Kick and lift"
L["figure.phase.kick_up"] = "Kick up"
L["figure.phase.knee_back"] = "Knee back"
L["figure.phase.knee_down"] = "Knee down"
L["figure.phase.knee_in"] = "Knee in"
L["figure.phase.knee_to_chest"] = "Knee to chest"
L["figure.phase.knee_to_wall"] = "Knee to wall"
L["figure.phase.knee_up"] = "Knee up"
L["figure.phase.kneel"] = "Kneel"
L["figure.phase.kneel_tall"] = "Kneel tall"
L["figure.phase.knees_down"] = "Knees down"
L["figure.phase.knees_high"] = "Knees high"
L["figure.phase.knees_out"] = "Knees out"
L["figure.phase.knees_straight"] = "Knees nearly straight"
L["figure.phase.knees_up"] = "Knees up"
L["figure.phase.land_softly"] = "Land softly"
L["figure.phase.lean_back"] = "Lean back"
L["figure.phase.lean_forward"] = "Lean forward"
L["figure.phase.lean_in"] = "Lean in"
L["figure.phase.leg_out"] = "Leg straight out"
L["figure.phase.leg_straight"] = "Leg straight"
L["figure.phase.leg_up"] = "Leg up"
L["figure.phase.leg_up_the_wall"] = "Leg up wall"
L["figure.phase.let_go"] = "Let go"
L["figure.phase.lie"] = "Lie down"
L["figure.phase.lie_flat"] = "Lie flat"
L["figure.phase.lie_on_roll"] = "Lie on towel"
L["figure.phase.lift"] = "Lift"
L["figure.phase.lift_breathe_out"] = "Lift, breathe out"
L["figure.phase.lift_overhead"] = "Lift overhead"
L["figure.phase.light_steps"] = "Light steps"
L["figure.phase.lockout"] = "Lock out"
L["figure.phase.look_up"] = "Look up"
L["figure.phase.lower"] = "Lower"
L["figure.phase.lower_fully"] = "Lower fully"
L["figure.phase.lower_on_one"] = "Lower on one"
L["figure.phase.lower_slowly"] = "Lower slowly"
L["figure.phase.lower_to_one_hand"] = "Toward one hand"
L["figure.phase.lower_with_control"] = "Lower slowly"
L["figure.phase.lunge"] = "Lunge"
L["figure.phase.move_an_inch"] = "Move an inch"
L["figure.phase.one_leg"] = "One leg"
L["figure.phase.onto_side"] = "Onto your side"
L["figure.phase.onto_toes"] = "Onto toes"
L["figure.phase.open_out"] = "Open out"
L["figure.phase.open_up"] = "Open up"
L["figure.phase.other_side"] = "Other side"
L["figure.phase.other_way"] = "Other way"
L["figure.phase.pick_up"] = "Pick up"
L["figure.phase.plank"] = "Plank"
L["figure.phase.press"] = "Press"
L["figure.phase.press_gently"] = "Press gently"
L["figure.phase.pull"] = "Pull"
L["figure.phase.pull_apart"] = "Pull apart"
L["figure.phase.pull_back"] = "Pull back"
L["figure.phase.pull_in"] = "Pull in"
L["figure.phase.pull_to_face"] = "Pull to face"
L["figure.phase.pull_to_hip"] = "Pull to hip"
L["figure.phase.punch"] = "Punch"
L["figure.phase.punch_across"] = "Punch across"
L["figure.phase.push_away"] = "Push away"
L["figure.phase.push_back"] = "Push back"
L["figure.phase.push_steadily"] = "Push steadily"
L["figure.phase.push_up"] = "Push up"
L["figure.phase.put_down"] = "Put down"
L["figure.phase.quarter_down"] = "Quarter down"
L["figure.phase.quick_steps"] = "Quick steps"
L["figure.phase.rack"] = "Rack"
L["figure.phase.reach_back"] = "Reach back"
L["figure.phase.reach_long"] = "Reach long"
L["figure.phase.reach_the_heels"] = "Reach the heels"
L["figure.phase.release"] = "Release"
L["figure.phase.resist_twist"] = "Resist the twist"
L["figure.phase.return_slowly"] = "Return slowly"
L["figure.phase.ribs_down"] = "Ribs down"
L["figure.phase.rise"] = "Rise"
L["figure.phase.rise_on_two"] = "Rise on two"
L["figure.phase.rise_slowly"] = "Rise slowly"
L["figure.phase.rock_back"] = "Rock back"
L["figure.phase.rock_forward"] = "Rock forward"
L["figure.phase.roll_back"] = "Roll back"
L["figure.phase.roll_calves"] = "Roll calves"
L["figure.phase.roll_out"] = "Roll out"
L["figure.phase.roll_seat"] = "Roll seat"
L["figure.phase.roll_side"] = "Roll your side"
L["figure.phase.roll_thighs"] = "Roll thighs"
L["figure.phase.rotate_open"] = "Rotate open"
L["figure.phase.round_back"] = "Round your back"
L["figure.phase.shift_forward"] = "Shift forward"
L["figure.phase.shoulder_down"] = "Shoulder down"
L["figure.phase.shoulder_height"] = "Shoulder height"
L["figure.phase.shoulder_rolls"] = "Shoulder rolls"
L["figure.phase.shoulders_down"] = "Shoulders down"
L["figure.phase.shoulders_over_hands"] = "Shoulders over hands"
L["figure.phase.shoulders_up"] = "Shoulders up"
L["figure.phase.side_step"] = "Side step"
L["figure.phase.sink"] = "Let it sink"
L["figure.phase.sit"] = "Sit"
L["figure.phase.sit_back"] = "Sit back"
L["figure.phase.sit_down"] = "Sit down"
L["figure.phase.sit_tall"] = "Sit tall"
L["figure.phase.sit_up"] = "Sit up"
L["figure.phase.skip_a_step"] = "Skip a step"
L["figure.phase.slam_down"] = "Slam down"
L["figure.phase.slide_back"] = "Slide back"
L["figure.phase.slide_down"] = "Slide down"
L["figure.phase.slide_forward"] = "Slide forward"
L["figure.phase.slide_out"] = "Slide out"
L["figure.phase.slide_under"] = "Slide under"
L["figure.phase.slide_up"] = "Slide up"
L["figure.phase.small_bounce"] = "Small bounces"
L["figure.phase.snap_hips"] = "Snap hips"
L["figure.phase.spin_easily"] = "Spin easily"
L["figure.phase.squat"] = "Squat"
L["figure.phase.squat_deep"] = "Squat deep"
L["figure.phase.squeeze"] = "Squeeze"
L["figure.phase.stand"] = "Stand"
L["figure.phase.stand_tall"] = "Stand tall"
L["figure.phase.stand_up"] = "Stand up"
L["figure.phase.start_at_top"] = "Start at top"
L["figure.phase.step_back"] = "Step back"
L["figure.phase.step_down"] = "Step down"
L["figure.phase.step_forward"] = "Step forward"
L["figure.phase.step_in"] = "Step in"
L["figure.phase.step_out"] = "Step out"
L["figure.phase.step_through"] = "Step through"
L["figure.phase.step_up"] = "Step up"
L["figure.phase.step_wide"] = "Step wide"
L["figure.phase.stretched"] = "Stretched"
L["figure.phase.swim_strokes"] = "Swim strokes"
L["figure.phase.switch"] = "Switch"
L["figure.phase.t_raise"] = "T raise"
L["figure.phase.tall"] = "Stand tall"
L["figure.phase.tap_floor"] = "Tap the floor"
L["figure.phase.tap_shoulder"] = "Tap shoulder"
L["figure.phase.throw"] = "Throw"
L["figure.phase.to_wall"] = "To the wall"
L["figure.phase.toes_off"] = "Toes off"
L["figure.phase.toes_up"] = "Toes up"
L["figure.phase.top"] = "Top"
L["figure.phase.top_leg_up"] = "Top leg up"
L["figure.phase.tuck_and_squeeze"] = "Tuck and squeeze"
L["figure.phase.tuck_chin"] = "Tuck the chin"
L["figure.phase.tuck_tailbone"] = "Tuck tailbone"
L["figure.phase.turn_around"] = "Turn around"
L["figure.phase.turn_head"] = "Turn your head"
L["figure.phase.turn_out"] = "Turn out"
L["figure.phase.turn_over"] = "Turn over"
L["figure.phase.turn_rope"] = "Turn the rope"
L["figure.phase.turn_to_knee"] = "Turn to knee"
L["figure.phase.two_turns"] = "Two turns"
L["figure.phase.w_raise"] = "W raise"
L["figure.phase.walk_back"] = "Walk back"
L["figure.phase.walk_briskly"] = "Walk briskly"
L["figure.phase.walk_easily"] = "Walk easily"
L["figure.phase.walk_fast"] = "Walk fast"
L["figure.phase.walk_feet_down"] = "Walk feet down"
L["figure.phase.walk_feet_up"] = "Walk feet up"
L["figure.phase.walk_forward"] = "Walk forward"
L["figure.phase.walk_hands_back"] = "Walk hands back"
L["figure.phase.walk_hands_down"] = "Walk hands down"
L["figure.phase.walk_hands_in"] = "Walk hands in"
L["figure.phase.walk_hands_out"] = "Walk hands out"
L["figure.phase.walk_hands_up"] = "Walk hands up"
L["figure.phase.wheel_to_wall"] = "Wheel to wall"
L["figure.phase.wrap"] = "Wrap and sink"
L["figure.phase.wrists_turn"] = "Turn with wrists"
L["figure.phase.y_raise"] = "Y raise"
L["settings.standup"] = "Stand-up move"
L["settings.standup.tip"] = "A %d-second set of step jacks after %d minutes without a pump."
L["settings.deathTax"] = "Death tax"
L["settings.deathTax.tip"] = "Each time your character dies, one banked move is added. It comes with your next "
	.. "pump."
L["settings.inputMode"] = "Count input"
L["settings.inputMode.tip"] = "Mouse only leaves the keys to the game while you count. The others pick the box or "
	.. "the spinner where you correct a count."
L["settings.input.auto"] = "Automatic (the spinner with a gamepad)"
L["settings.input.type"] = "Type the number"
L["settings.input.spinner"] = "Spinner"
L["settings.input.mouse"] = "Mouse only"
L["settings.chat"] = "Chat messages"
L["settings.chat.tip"] = "Off, the rare warning only, or every toast as a chat line too."
L["settings.chat.off"] = "Off"
L["settings.chat.warnings"] = "Warnings"
L["settings.chat.all"] = "All"
L["settings.afterPump"] = "After a pump"
L["settings.afterPump.tip"] = "A banner with what the pump paid: always, only when you won something, or never."
L["settings.afterPump.summary"] = "Summary"
L["settings.afterPump.wins"] = "Only with a win"
L["settings.afterPump.none"] = "Nothing"
-- General > Layout (slice 23): the Streamer layout's pump card and the day's tally (Panel.lua, Streamer.lua)
L["settings.layout"] = "Layout"
L["settings.layout.tip"] = "Streamer: the screen never goes dark for a pump, the pump card sits in a corner you can "
	.. "move, and the day's tally stays on the screen for streams and videos."
L["settings.layout.standard"] = "Standard"
L["settings.layout.streamer"] = "Streamer"
-- the banner that closes a pump (Sheet.lua): the pump's name and its moves, the XP and the stats it paid
L["banner.title"] = "Pump complete"
L["banner.sub"] = "%s · %s"
L["banner.moves"] = "%d moves"
L["banner.xp"] = "+%s XP"
L["banner.stat"] = "+%d %s"
L["banner.more"] = "+%d more"
-- the banner's and a toast's mouse actions, on their tooltips
L["banner.tip.log"] = "Click: open the Log"
L["banner.tip.close"] = "Right-click: close"
L["banner.tip.share"] = "Right-click: share in chat"
L["settings.equipment"] = "Equipment"
-- the equipment groups' captions on Your setup and the Equipment page (ns.GearGroups, Rapha, 2026-11-03); the apps'
-- places.gear.group.<key> in the same words
L["settings.gear.group.home"] = "At home"
L["settings.gear.group.bars"] = "Bars and rings"
L["settings.gear.group.weights"] = "Weights and bands"
L["settings.gear.group.cardio"] = "Cardio"
L["settings.gear.group.core"] = "Core and mobility"
-- each piece's tooltip on the Equipment page and Your setup (ns.Settings.GearTip), under its name: what counts as it,
-- what it is for and its setup lines' safety words (David, 2026-10-13: "Add a hover explanation to the equpiments
-- to give more details"), then while unchecked what it would open now; the apps show the same words
-- (contract/sheet.json gear_tips)
L["settings.gear.stable_chair.tip"] = "A chair with no wheels, its back against a wall, or a sofa. For incline "
	.. "push-ups, single-leg hip thrusts and a foot rest in assisted pull-ups. Nordic curls want a heavy sofa, "
	.. "never a chair."
L["settings.gear.table.tip"] = "A heavy table on four legs that does not lift, slide or creak when you lean on its "
	.. "edge and pull. Never glass, folding, a standing desk or one with monitors. You lie under it and pull your "
	.. "chest to the edge."
L["settings.gear.bar.tip"] = "A pull-up bar fixed firmly, in a doorway or on a wall, with a stool to step down on. "
	.. "For hangs, pull-ups, hanging knee raises and the front lever."
L["settings.gear.fixed_bar.tip"] = "A bar bolted to a wall, a ceiling or a rack, with room above it. Never a "
	.. "doorway bar. For muscle-ups, bar dips and fast pull-ups, and it counts for every pull-up bar move too."
L["settings.gear.bands.tip"] = "Resistance bands, and a door anchor you shut in a closed door. Check each band for "
	.. "nicks or tears. For rows, face pulls, curls and help in pull-ups. A small band goes round your knees for "
	.. "the monster walk."
L["settings.gear.kettlebell.tip"] = "A kettlebell, 8-16 kg to start. For swings, goblet squats, halos and carries. "
	.. "In rows, deadlifts and hip thrusts it takes the place of a bag or a jug."
L["settings.gear.dumbbells.tip"] = "A pair of dumbbells, light ones of 2-5 kg for raises to start. For goblet "
	.. "squats, lunges, curls, raises and carries. In rows, deadlifts and hip thrusts they take the place of a bag "
	.. "or a jug."
L["settings.gear.barbell.tip"] = "A barbell with plates in a rack. Set the safeties and put the collars on. For "
	.. "light squats, deadlifts, overhead presses and the landmine press."
L["settings.gear.rope.tip"] = "A jump rope. Wear shoes, with clear space overhead and around you. For rope cardio, "
	.. "from slow bounces up to double-unders."
L["settings.gear.bike.tip"] = "An exercise bike, its seat at hip height and your feet secure on the pedals. For "
	.. "quick spins, hard rides and short sprints as cardio."
L["settings.gear.stairs.tip"] = "Stairs with a handrail, kept clear. For stair climbs, step push-ups, step-downs "
	.. "and heel drops. The all-out climb wants three clear flights."
L["settings.gear.pad.tip"] = "A walking pad or a treadmill, its safety key clipped on or a hand rail in reach. For "
	.. "quick, brisk and fast walks as cardio."
L["settings.gear.step.tip"] = "A sturdy step or box that cannot slide or wobble. For step-ups, step-overs and "
	.. "split squats with the back foot up. It stands in for the bottom stair, and goes under a fixed bar for "
	.. "muscle-ups."
L["settings.gear.wheel.tip"] = "An ab wheel, with a folded towel under your knees. For rollouts, from short ones "
	.. "to a wall up to full ones standing."
L["settings.gear.parallettes.tip"] = "Parallettes, or two equal stacks of hard books on a firm floor, never chairs "
	.. "on wheels. They raise your hands in the L-sits."
L["settings.gear.rings.tip"] = "Gymnastics rings or straps, fixed firmly to something that holds your full weight. "
	.. "For ring rows, presses, Y-raises, fallouts, hamstring curls, help in pistols and the front lever."
L["settings.gear.vest.tip"] = "A weighted vest, 5-10 kg to start, snug and even. It adds load to squats, step-ups, "
	.. "wall sits and stair climbs, and takes the backpack's place in squat drives."
L["settings.gear.ball.tip"] = "A medicine ball. Slams want a slam ball that does not bounce, chest passes one that "
	.. "bounces back off a solid wall. For squats, carries, chops, slams and passes."
L["settings.gear.roller.tip"] = "A foam roller. For slow rolls along your calves, thighs, hips and back, leaning "
	.. "your upper back over it, and the dead bug lying along it."
L["settings.gear.backpack.tip"] = "A backpack you load with books or water bottles, straps tight. For backpack "
	.. "squats, squat drives and suitcase carries. A kettlebell, dumbbells or a vest take its place in some moves "
	.. "when you have them."
L["settings.gear.pole.tip"] = "A steel pole fixed in the ground outdoors that does not turn, with soft ground "
	.. "under you. Never a sign post. For the human flag and the steps that lead up to it."
L["settings.gear.opens"] = "Now it would open %d moves, up to %s."
L["settings.gear.opens.one"] = "Now it would open %d move, up to %s."
L["settings.gear.opens.plain"] = "Now it would open %d moves."
L["settings.gear.opens.plain.one"] = "Now it would open %d move."
L["settings.body"] = "Body"
L["settings.body.pain"] = "Pain right now"
L["settings.body.painTip"] = "Moves that load this joint leave your pool until you clear it."
-- the Body page (2026-10-16): a pain flag's click hint
L["settings.body.click.flag"] = "Click: flag it"
L["settings.body.click.clear"] = "Click: clear the flag"
L["settings.social"] = "Social"
-- the Social page (2026-10-16): its sections, the muted row's name and a greyed row's reason
L["settings.section.carries"] = "Your card carries"
L["settings.section.others"] = "From others"
L["settings.social.muted"] = "Muted and hidden"
L["settings.social.needsShare"] = "Needs your card shared."
L["settings.social.share"] = "Who sees your card"
L["settings.social.share.nobody"] = "Nobody (private)"
L["settings.social.share.circle"] = "Friends, guild and group"
L["settings.social.share.anyone"] = "Anyone who asks"
L["settings.shareRecords"] = "Include my records"
L["settings.shareRecords.tip"] = "Your first and best count on each ladder travels with your card, marked as typed "
	.. "by you."
L["settings.tellLegend"] = "Announce Legendary unlocks"
L["settings.tellLegend.tip"] = "When you unlock a Legendary or Artifact move while others can see your card, party and "
	.. "guild members who run QuestPump with Cheers on see a line naming it. Off unless you switch it on."
L["settings.shareCount"] = "Put the count in share lines"
L["settings.shareCount.tip"] = "A share line says \"+1 on my best: 23 push-ups\" instead of \"+1 on my best: "
	.. "push-ups\"."
L["settings.showCards"] = "Show others' cards"
L["settings.showCards.tip"] = "Their QuestPump line in tooltips, their card from their QuestPump menu (/qp player), a "
	.. "badge on their unit frames, a line under their nameplate, a mark in chat, the Inspect tab and the Guild tab. "
	.. "Your party's moves today on the HUD. While you share your card, theirs is asked for when you hover, target, "
	.. "inspect or open it."
L["settings.cheers"] = "Cheers and party pump invites"
L["settings.cheers.tip"] = "Guildmates' and party members' finished pumps with a Cheer, their Legendary and Artifact "
	.. "unlocks, the Join line when a party member starts a pump, and the cheers you get."
L["settings.invites"] = "Pact and duel invites"
L["settings.invites.tip"] = "Friends can offer you a pact or a duel, and you can offer them one from their QuestPump "
	.. "menu (/qp player). Both need your card shared and a duel their card seen this session; off, their offers are "
	.. "ignored and the menu's entries hide."
L["settings.partner"] = "A partner on holds when you play alone"
L["settings.partner.tip"] = "On a hold, a partner's bar runs beside yours and holds a few seconds past your goal. "
	.. "Only a picture: it changes no goal, record or count. Not shown in a group."
L["settings.socialOffInInstances"] = "Off in dungeons, raids and battlegrounds"
L["settings.socialOffInInstances.tip"] = "Nothing is sent or shown there."
L["settings.social.session"] = "%d muted · %d hidden from"
L["settings.social.clear"] = "Clear"
L["settings.social.lists"] = "Mute and hide come from a player's QuestPump menu (/qp player) and last for this "
	.. "session. The game's ignore list works both ways and lasts."
L["settings.switch.floor.tip"] = "Moves with hands, forearms or knees on the floor, or lying down. No mat needed."
L["settings.switch.space.tip"] = "Room for a few steps or a clear 2 x 2 m: skater hops, lateral lunges, crawls, "
	.. "handstands, frog stands and planches join your pool."
L["settings.switch.quiet.tip"] = "No loud moves (jumps, stomps, slams)."
L["settings.switch.nojump.tip"] = "No move where both feet leave the floor."
L["settings.switch.nostrain.tip"] = "No move taken to a breath-holding effort."
L["settings.switch.noheaddown.tip"] = "No move with the head below the heart."
L["settings.exercises"] = "Exercises"
-- the Exercises page (2026-10-16): the search, the switch's two halves, the notes' bar, the regions, a group's count,
-- a move's line (left out, or its record) and the mark's reasons (the setup keeps a ticked move out), the click hints
L["settings.exercises.search"] = "Search moves"
L["settings.exercises.bodyweight"] = "Bodyweight"
L["settings.exercises.gear"] = "With equipment"
L["settings.exercises.none"] = "No move is named \"%s\"."
L["settings.warn.more"] = "%d notes on your pool"
L["settings.warn.more.one"] = "%d note on your pool"
L["settings.region.pushpull"] = "Push and pull"
L["settings.region.legs"] = "Legs"
L["settings.region.trunk"] = "Trunk"
L["settings.region.cardio"] = "Cardio and stretching"
L["settings.group.count"] = "%d of %d in your pool"
L["settings.group.leftOut"] = "Left out with its group: tick the group to choose its moves."
L["settings.move.withRecord"] = "Record %s"
L["settings.move.leftOut"] = "Left out"
L["settings.move.clickOut"] = "Click: leave it out of your pumps"
L["settings.move.clickIn"] = "Click: put it back in your pumps"
L["settings.group.clickShow"] = "Click: show its moves"
L["settings.group.clickHide"] = "Click: hide its moves"
L["settings.group.clickOut"] = "Click: leave them out of your pumps"
L["settings.group.clickIn"] = "Click: put them back in your pumps"
L["settings.out.gear"] = "Needs: %s"
L["settings.out.switch"] = "%s: %s"
L["settings.out.pain"] = "Pain flag: %s"
L["settings.out.better"] = "Set aside for %s"
L["settings.warn.ladder"] = "The %s ladder has no move left and drops out."
L["settings.warn.ladder.open"] = "The %s ladder has no move here: %s."
L["settings.warn.stat.strength"] = "No Strength moves left: Strength stands still."
L["settings.warn.stat.agility"] = "No cardio or power moves left: Agility stands still."
L["settings.warn.stat.intellect"] = "No balance, arm-balance or handstand moves left: Intellect rises only from "
	.. "learning."
L["settings.warn.stat.stamina"] = "No core, side, wall-sit or L-sit moves left: Stamina stands still."
L["preset.apartment"] = "Apartment, quiet"
L["preset.office"] = "Office, no floor"
L["preset.title"] = "Presets"
L["preset.wrists"] = "Leave out wrist moves"
L["preset.knees"] = "Leave out knee moves"
L["preset.everything"] = "Tick every move"
L["preset.apartment.tip"] = "Turns on Quiet mode and No jumping on the Body page."
L["preset.office.tip"] = "Turns off Floor moves and Room to move on the Body page."
L["preset.wrists.tip"] = "Unticks every move that loads the wrists, for good. A wrist pain flag on the Body page "
	.. "leaves them out only until you clear it."
L["preset.knees.tip"] = "Unticks every move that loads the knees, for good. A knee pain flag on the Body page leaves "
	.. "them out only until you clear it."
L["preset.everything.tip"] = "Ticks every move on this page again. The Body page's switches stay as they are."
-- the Body page's switches that narrow the pool, under it
L["settings.pool.body"] = "Body page switches on: %s."
L["settings.data"] = "Data"
L["settings.profile"] = "Profile"
L["settings.perChar"] = "Own settings for this character"
L["settings.perChar.tip"] = "On: this character keeps its own copy of your QuestPump settings, so what you change "
	.. "here leaves your other characters alone. Off: it uses the settings all your characters share. Your progress "
	.. "stays the same either way."
L["settings.profile.char"] = "%s - %s"
L["settings.reset.char"] = "Reset this profile's settings"
L["settings.reset.all"] = "Reset it all"
L["settings.restore"] = "Restore the backup"
L["settings.restart"] = "Start a ladder over"
L["settings.clear"] = "Clear a move's record"
-- the Data page (2026-10-16): its sections, the profile's and the backup's state lines, what a row does (its
-- tooltip's words), the pickers' prompts, the buttons named for their action, a greyed control's reason, and the rows'
-- click hints (a picker's is settings.click.choose)
L["settings.section.progress"] = "Your progress"
L["settings.section.over"] = "Start over"
L["settings.data.shared"] = "Shared by all your characters"
L["settings.restart.pick"] = "Pick a ladder"
L["settings.restart.line"] = "Its records and steps go."
L["settings.clear.pick"] = "Pick a move"
L["settings.clear.line"] = "Its next set is a max set."
L["settings.clear.none"] = "No record to clear yet."
L["settings.reset.charButton"] = "Reset settings"
L["settings.reset.charLine"] = "Your progress stays."
L["settings.restore.button"] = "Restore"
L["settings.restore.line"] = "The save from before your last reset."
L["settings.restore.none"] = "No backup yet."
L["settings.reset.allLine"] = "Deletes every pump and record."
L["settings.click.export"] = "Click: open the export window"
L["settings.click.import"] = "Click: open the import window"
L["settings.click.ask"] = "Click: it asks you first"
L["ui.reset.confirm"] = "%s"
-- the reset dialog's decline, and its accept for a record cleared and a ladder started over (the other kinds take
-- their Data page buttons' words)
L["ui.reset.keep"] = "Keep it"
L["ui.reset.do.move"] = "Clear record"
L["ui.reset.do.ladder"] = "Start over"
L["ui.reset.char"] = "The settings of the profile %s go back to the defaults, for every character on it. The Body "
	.. "page's switches, your exclusions and your progress stay."
L["ui.reset.all"] = "Every pump, record and setting is deleted. The save is kept as a backup, which /qp "
	.. "restore-backup brings back in this game only."
L["ui.reset.restore"] = "The backup replaces the save you have now, which becomes the backup: restoring again swaps "
	.. "them back."
L["ui.reset.move"] = "Clear your %s record? Your next set of it is a max set again: as many as you can. Your pumps, "
	.. "your XP and your ladders stay."
L["ui.reset.ladder"] = "Start your %s ladder over? Its moves' records and its unlocked steps go, and you go back to "
	.. "its start. Your next set places you again. Your pumps, your XP and your other ladders stay."
L["status.reset.beta"] = "The game loads your progress from QuestPump on this computer at /reload: keep it running, "
	.. "or the reset is undone."
L["status.restore.beta"] = "The game loads your progress from QuestPump on this computer at /reload, and its copy has "
	.. "no backup: the restored save lasts only until you type /reload."
-- a /qp reset all is the player's, everywhere (slice 26): the popup and the chat line say it reaches the app and the
-- other game installs, and that a restore brings back only this game's save (DB.lua ns.RestoreBackup)
L["status.reset.everywhere"] = "QuestPump on this computer passes the reset on, so the app and your other game "
	.. "installs start over too."
L["status.restore.here"] = "Only this game's save comes back: QuestPump on this computer, the app and your other "
	.. "game installs keep the reset."
-- a record cleared or a ladder started over while the helper runs (Settings.lua): the clear row is the account's
L["status.clear.everywhere"] = "QuestPump on this computer passes it on, so the app and your other game installs do "
	.. "the same."
L["status.clear.beta"] = "The game loads your progress from QuestPump on this computer at /reload: start it before "
	.. "then, or this is undone."
-- a count corrected in the game (slice 44, Correct a count's window over Save): on a client that showed it loads no
-- save the correction row outlasts /reload only through the helper's copy
L["status.correct.beta"] = "The game loads your progress from QuestPump on this computer at /reload: start it before "
	.. "then, or this correction is undone."
-- Correct a count's window (slice 44, 2026-10-18; Settings.lua Settings_.CorrectWindow: the phones' and the desktop
-- app's correct.window in the game's words). Its head (the move's name); the list's label and the line in its place
-- with no set (correct_days, both plural); the line over the clear where the record's own set is not listed; a set's
-- row: its game day and time, and the record's mark; a set's view: what it saved and when (a pose's answer, in the
-- short words; a set saved with no number, Done without a count, Could not do one or a counted form's
-- Not counted, its own words: slice 44's review, 2026-10-18), Save and the way back to the list; after Save: the set
-- as the fold reads it now (a count corrected to 0 reads Done without a count, as typing 0 does; a capped max set
-- corrected over its window's top keeps the top), with its own words where it reads with no number, and why a step
-- stays where it is (a set from the app, whose steps only the app changes; a step change since the set that stands on
-- it); a set that left the window meanwhile (its days ran out at 04:00, or a clear came)
L["correct.title"] = "Correct a %s count"
L["correct.sets"] = "Your sets of the last %d days"
L["correct.sets.one"] = "Your sets of the last day"
L["correct.empty"] = "No set of the last %d days to correct."
L["correct.empty.one"] = "No set of the last day to correct."
L["correct.old"] = "Your record is from an older set, which you can no longer correct."
L["correct.when"] = "%s, %s"
L["correct.record"] = "Record"
L["correct.click"] = "Click: correct this set"
L["correct.saved"] = "You saved %s on %s."
L["correct.savedPose"] = "You answered %s on %s."
L["correct.savedNone"] = "You saved no count on %s."
L["correct.mark.perfect"] = "Perfect"
L["correct.mark.good"] = "Good"
L["correct.mark.okay"] = "Okay"
L["correct.mark.notyet"] = "Not yet"
L["correct.save"] = "Save"
L["correct.other"] = "Pick another set"
L["correct.done"] = "Corrected to %s."
L["correct.doneNone"] = "Corrected to no count."
L["correct.step.source"] = "Your step stays: a step from an app set changes only when you correct it in the app."
L["correct.step.later"] = "Your step stays: this ladder has moved on since that set."
L["correct.gone"] = "That set can no longer be corrected."
-- what the correction changed on the steps (2026-10-19, the program's double-check; Lc.WriteCorrection's second
-- return, ns.CorrectionRows' third), under the done line and in /qp undo's line, a line a ladder and a step: a climb
-- taken back (a stretch check's lift among them) or made, by the ladder's name and its step on the one scale; a
-- Mastered seal taken back or won, by the step and the ladder's name (the apps' correct.window.step.back and .up,
-- correct.window.mastered.lost and .won)
L["correct.step.back"] = "%s back to step %d."
L["correct.step.up"] = "%s up to step %d."
L["correct.mastered.lost"] = "Step %d of %s is no longer Mastered."
L["correct.mastered.won"] = "Step %d of %s is Mastered."
-- over Save on the set's own side, where another device's correction won and this side reads it otherwise (the
-- entry's ownSide, ns.OwnSide; 2026-10-19): the set as it reads now, at which Save stands ready (the apps'
-- correct.window.elsewhere)
L["correct.elsewhere"] = "Corrected to %s on another device."
-- correct.elsewhere where the set reads with no number (a count corrected to 0, a counted form's Not counted), whose
-- words read wrong after "Corrected to", as correct.doneNone's (the apps' correct.window.elsewhere.none)
L["correct.elsewhereNone"] = "Corrected to no count on another device."
-- once, when a /qp reset all newer than this save reset it: through the helper at a login (another game install's,
-- or this install's own whose save never reached the disk; DB.lua ns.MirrorReset), or in an import string
L["status.reset.elsewhere"] = "A /qp reset all newer than this save was found, so your progress starts over from it, "
	.. "here and, through QuestPump on this computer, in the app and your other game installs. The old save is kept "
	.. "as a backup, which /qp restore-backup brings back in this game only."
-- the Body switches, the Exercises page's list and Home's setup (its equipment and its switches: slice 51) one body
-- shares with the other devices (ns.TakeBody): what came
L["status.body.synced"] = "From your other devices: %s."
L["status.body.moves"] = "the Exercises page's list of moves"
L["status.body.home"] = "your equipment and where you train"
L["ui.io.title"] = "Export and import"
L["ui.io.export"] = "Export"
L["ui.io.import"] = "Import"
L["ui.io.copyLabel"] = "Copy with %s"
L["ui.io.pasteLabel"] = "Paste with %s"
-- what the string holds, said where it is made (docs/tasks.md), and what a merge does; the export's way out
L["ui.io.holds"] = "What it holds"
L["ui.io.holdsNote"] = "Your pumps and counts with their dates, the pain flags and stops you entered, your settings "
	.. "(with the name of a friend you have a pact or a duel with), and your characters' names, realms and ids. Share it "
	.. "only where you would share those."
L["ui.io.merge"] = "When you import"
L["ui.io.mergeNote"] = "Its rows are merged in; a profile with the same name takes the string's settings."
L["ui.io.done"] = "Done"
L["ui.io.copy"] = "Ctrl+C"
L["ui.io.copyMac"] = "Cmd+C"
L["ui.io.paste"] = "Ctrl+V"
L["ui.io.pasteMac"] = "Cmd+V"
L["import.done"] = "Imported: the progress is merged in."
L["import.prefix"] = "That is not a QuestPump export string (it starts with !QP1!)."
L["import.broken"] = "The string is damaged or incomplete."
L["import.large"] = "That string is too long to be a QuestPump export."
L["import.newer"] = "That string comes from a newer QuestPump: update the addon first."
L["import.readonly"] = "This save belongs to a newer QuestPump and is read-only."
L["import.old"] = "That string is from before your latest /qp reset all: nothing in it comes after the reset, so "
	.. "nothing was imported."

-- Slice 24: the feedback button (Settings.lua's feedback window; contract/feedback-api.yaml, "What every window
-- says"), in the phones' words (contract/copy/en.json feedback.*) with the game's source. The helper and the phone app
-- are named only in the status keys, and the game names no URL or address: "the addon's page".
L["cmd.feedback.help"] = "/qp feedback: send us a message, with your numbers if you want"
L["settings.feedback.button"] = "Send feedback"
L["settings.feedback.tip"] = "Tell the people who make QuestPump what you like, what is wrong or what you miss. "
	.. "Nothing is sent until you press Send."
-- the General page's Companions row (Setup.Open; David, 2026-10-09): the settings keys name neither companion
L["settings.companions"] = "Companions"
L["settings.companions.tip"] = "The setup window: QuestPump's strongly recommended companions, what they give and "
	.. "how to set each one up, then Your setup: where you train, your equipment and how hard."
-- the companions set up of those the setup asks for (two, or the helper alone while the phone apps are not out)
L["settings.companions.done"] = "%d of %d set up"
L["ui.feedback.title"] = "Send feedback"
L["ui.feedback.placeholder"] = "Tell us what you like, what is wrong or what you miss."
-- under the box: the characters typed of the most it takes (120/500)
L["ui.feedback.counter"] = "%d/%d"
L["ui.feedback.tick"] = "Add my numbers"
-- under the tick, while it is on and while it is off
L["ui.feedback.note"] = "Only the counts below go with your words, and no name. Nothing is sent until you press Send."
L["ui.feedback.note.off"] = "Only your words and the version go, and no name. Nothing is sent until you press Send."
L["ui.feedback.reply"] = "We read every message, but we cannot write back: it has no name or address. If you want "
	.. "an answer, write to us on the addon's page."
-- what goes: its first line, ticked or not (never "Sent from": nothing has gone yet), then the counts
L["ui.feedback.from"] = "From the game, version %s"
L["ui.feedback.days"] = "Days since you started: %d"
L["ui.feedback.unfinished"] = "Banked moves now: %d"
L["ui.feedback.window"] = "In the last %d days:"
L["ui.feedback.finished"] = "Pumps finished: %d"
L["ui.feedback.banked"] = "Pumps banked: %d"
-- %s is 0, a whole number (3) or a half (2.5)
L["ui.feedback.maxsets"] = "Max sets on one ladder on a usual play day: %s"
L["ui.feedback.moves"] = "Easier picks and early stops, by move:"
L["ui.feedback.move"] = "%s: easier pick %d, stopped early %d"
L["ui.feedback.none"] = "None"
L["ui.feedback.send"] = "Send"
-- the window's two sections: the box (its name in the contract, "Your message") and what goes with it
L["ui.feedback.message"] = "Your message"
L["ui.feedback.goes"] = "What goes"
-- at the top while Send is off, by ns.HelperState (the setup screen's step for each, with /reload for Check again):
-- never set up, not running at this load, running blind, the mirror folder turned off or out of date
L["status.feedback.none"] = "The game sends feedback through it. Download it from the addon's page, open it, then "
	.. "type /reload. Or send your feedback on the addon's page."
L["status.feedback.off"] = "Send is off until it runs. Open it, then type /reload. Or send your feedback on the "
	.. "addon's page."
L["status.feedback.blind"] = "Send is off until it can see the game. In its window, press Allow screen reading, turn "
	.. "it on, restart it, then type /reload. Or send your feedback on the addon's page."
L["status.feedback.stopped"] = "Send is off until it can read the game. Open its window, then type /reload. Or send "
	.. "your feedback on the addon's page."
-- Send's tooltip, and its red reason while QuestPump on this computer does not run at this load
L["status.feedback.sendTip"] = "Hands your words to QuestPump on this computer, which sends them."
L["status.feedback.sendOff"] = "Send needs QuestPump on this computer running."
L["status.feedback.disabled"] = "Send is off: QuestPump Mirror is turned off in the AddOns list. Turn it on there, "
	.. "then type /reload. Or send your feedback on the addon's page."
L["status.feedback.outdated"] = "Send is off: QuestPump Mirror is out of date for this game version. Open QuestPump "
	.. "on this computer so it updates it, then type /reload. Or send your feedback on the addon's page."
-- beside Send after it is pressed, while the message is still going onto the strip, and once it is on it (also in
-- chat): handed to the helper, never "sent", since nothing reaches a running addon
L["status.feedback.pending"] = "Passing your feedback to QuestPump on this computer. Wait a few seconds before you "
	.. "type /reload or log out."
L["status.feedback.handed"] = "Your feedback went to QuestPump on this computer. Its window shows when it is sent. "
	.. "Thank you!"
-- in chat, when a fight closes the window on words not yet sent
L["status.feedback.kept"] = "Your feedback is kept. Type /qp feedback after the fight."

-- Slice 13: the social core (docs/plan.md slice 13): the card, the party, cheers, the Guild tab --------------------
-- Nothing here ranks a typed number or names who has not paid; every line a player sends is theirs to send.

-- what the card carries, on the Social page
L["ui.share.note"] = "The card shows your QP level, title, streak and this week's pumps, never your pain flags, heart "
	.. "rate or where you are. Addon messages travel through Blizzard's servers, which can see them."
-- the tooltip line
L["social.tip.level"] = "QuestPump %d"
L["social.tip.paragon"] = "QuestPump %d · Paragon %d"
L["social.tip.private"] = "QuestPump (private)"
L["social.tip.streak"] = "%d-day streak"
L["social.tip.pumps"] = "%d pumps this week"
L["social.tip.pumps.one"] = "%d pump this week"
L["social.tip.records"] = "%d records beaten"
L["social.tip.records.one"] = "%d record beaten"
L["social.tip.pumping"] = "In a pump now"
L["social.tip.milestone"] = "Guild milestone: %d pumps this week"
-- a player's QuestPump menu (/qp player, the card window's More) and the card window: each player's card (its stat
-- rows, the clock's tooltip), the typed lines one under the other with their owner's name, what the two share (the
-- pact's and the duel's rows, the duel's score cells)
L["social.menu.card"] = "QuestPump card"
L["social.menu.mute"] = "Mute their lines"
L["social.menu.unmute"] = "Show their lines"
L["social.menu.hide"] = "Hide my card from them"
L["social.menu.unhide"] = "Show my card to them"
L["social.card.title"] = "QuestPump card"
L["social.card.streak"] = "Streak (days)"
L["social.card.active"] = "Active days (last %d)"
L["social.card.pumps"] = "Pumps this week"
L["social.card.records"] = "Records beaten this week"
L["social.card.cardio"] = "Cardio minutes this week"
L["social.card.pumping"] = "In a pump now"
L["social.card.typed"] = "Own progress, typed by each player"
L["social.card.typedLine"] = "%s: %s %s → %s since %s"
L["social.card.typedBy"] = "%s · %s"
L["social.card.together"] = "You and %s"
L["social.card.pactLine"] = "Pact · %d days"
L["social.card.pactLine.one"] = "Pact · %d day"
L["social.card.duel"] = "Duel · day %d of %d"
-- the card window's duel row's tooltip: the other rivals' scores, its row showing the player's and this player's
L["social.card.duelOthers"] = "Also in the duel: %s"
L["social.card.you"] = "you"
L["social.card.reported"] = "Reported by %s's addon."
L["social.card.asking"] = "Asking for their card..."
L["social.card.none"] = "No card from them this session."
-- beside a player (slice 23, Badges.lua): the badge on the unit frames and the nameplate line ("QP 12 · the
-- Relentless"), past the cap the rank as the sheet's level line reads it; the Inspect window's tab
L["social.badge.level"] = "QP %d"
L["social.badge.paragon"] = "Paragon %d"
L["social.inspect.tab"] = "QuestPump"
-- the party and cheers on the HUD, the lines a player sends; the party's line is the player's own moves this game day,
-- then each member's ("Party today: you 14 · Jaina 9"), joined by " · "
L["social.hud.join"] = "%s starts a pump. Join?"
L["social.hud.party"] = "Party today: you %d"
L["social.hud.party.member"] = "%s %d"
L["social.hud.party.title"] = "Party today"
L["social.hud.party.tip"] = "Moves finished this game day, as each player's QuestPump counts them. A player who keeps "
	.. "their card private does not show."
L["social.hud.finished"] = "%s finished a pump · Cheer"
L["social.hud.digest"] = "Guild today: %d pumps from %d of us"
L["social.hud.digest.one"] = "Guild today: %d pump from %d of us"
L["social.cheer.title"] = "Cheer for %s"
L["cheer.cheer"] = "Cheer"
L["cheer.salute"] = "Salute"
L["cheer.flex"] = "Flex"
L["toast.cheer.cheer"] = "%s cheers your pump"
L["toast.cheer.salute"] = "%s salutes your pump"
L["toast.cheer.flex"] = "%s flexes for your pump"
L["social.guild.line"] = "QuestPump week: %d pumps from %d of us"
-- pacts and duels (slice 23): with friends only, one pact and one duel at a time; a pact never says who has not
-- paid, a duel scores only what QuestPump counts, and neither changes a pump, XP or a record
L["social.menu.pact"] = "Swear a pact"
L["social.menu.duel"] = "Challenge to a duel"
L["social.invite.later"] = "Not now"
L["social.pact.daily"] = "One pump every day"
L["social.pact.evening"] = "We pay every pump the same evening"
L["social.pact.offer"] = "%s offers you a QuestPump pact:"
L["social.pact.swear"] = "Swear it"
L["social.pact.sent"] = "Pact offered to %s."
L["social.pact.sworn"] = "You and %s swore a QuestPump pact."
L["social.pact.ended"] = "You ended the pact with %s."
L["social.pact.over"] = "The pact with %s has ended."
L["social.pact.hud"] = "Pact with %s: %d days · %s"
L["social.pact.hud.one"] = "Pact with %s: %d day · %s"
L["social.pact.both"] = "both done today"
L["social.pact.open"] = "today open"
L["social.pact.unknown"] = "today unknown"
L["social.pact.title"] = "QuestPump pact"
L["social.pact.done.daily"] = "Done: a pump paid today."
L["social.pact.done.evening"] = "Done: a pump paid today and no banked move."
L["social.pact.free"] = "A pact changes no pump, XP or record. Ending it costs nothing."
L["social.pact.end"] = "End the pact"
L["social.pact.endTip"] = "Right-click: end the pact"
L["social.duel.offer"] = "%s challenges you to a %d-day QuestPump duel, scored on what QuestPump counts."
L["social.duel.accept"] = "Join"
L["social.duel.sent"] = "Duel challenge sent to %s."
L["social.duel.joined"] = "%s joined your QuestPump duel."
L["social.duel.accepted"] = "You joined %s's QuestPump duel."
L["social.duel.left"] = "%s is out of the duel."
L["social.duel.leave"] = "Leave the duel"
L["social.duel.youLeft"] = "You left the duel."
L["social.duel.empty"] = "The duel ended: no one else is left in it."
L["social.duel.dropped"] = "The duel ended without a result: not every final score came."
L["social.duel.won"] = "Duel over: %s. You win the duel!"
L["social.duel.done"] = "Duel over: %s. Well fought."
L["social.duel.day"] = "Duel day %d: you %d"
L["social.duel.over"] = "Duel over: you %d"
L["social.duel.waiting"] = "waiting for final scores"
L["social.duel.you"] = "you %d"
L["social.duel.rival"] = "%s %d"
L["social.duel.best"] = "best %d"
L["social.duel.title"] = "QuestPump duel"
L["social.duel.tip.day"] = "Day %d of %d"
L["social.duel.tip.rules"] = "Each day: +%d a pump (%d at most, %d of them without a quest), +%d for a day you "
	.. "moved, +%d a cardio minute (%d at most). The score stops there; your pumps never do."
L["social.duel.tip.unknown"] = "%s: no score this session"
L["social.duel.leaveTip"] = "Right-click: leave the duel"
-- the Guild tab: its rest states (others' cards off; QuestPump asleep or off in this instance) with the button to the
-- Social page, where both switch
L["guild.off"] = "Others' cards are off"
L["guild.quiet"] = "Nothing social while QuestPump sleeps or in this instance"
L["guild.openSocial"] = "Social settings"
-- the Guild tab's three stats (2026-10-16; one sentence until then)
L["guild.pumps"] = "pumps this week"
L["guild.cardio"] = "cardio minutes"
L["guild.share"] = "your share"
L["guild.milestone"] = "Milestone: %s pumps this week"
L["guild.sendLine"] = "Share the week in guild chat"
L["guild.row"] = "%s · QP %d"
L["guild.streak"] = "%d-day streak"
L["guild.today"] = "today"
L["guild.click"] = "Click: their QuestPump card"
L["guild.empty"] = "No guildmate's card yet this session."
