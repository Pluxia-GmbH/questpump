-- QuestPump core: the version, the client flavour, the numeric codes of the row grammar, the clock, the one event
-- frame with its dispatch table, the addon's own message bus and the /qp dispatcher. Every other file hangs its event
-- handlers, commands and listeners on the functions here; no other file creates an event frame.

local ADDON, ns = ...
local L = ns.L

ns.version = C_AddOns.GetAddOnMetadata(ADDON, "Version") or "dev"
if ns.version:find("^@") then ns.version = "dev" end

-- Forever is any 16xxx interface (the packager's own rule), never `== 16001`, which breaks at the first client patch.
ns.IS_FOREVER = math.floor(select(4, GetBuildInfo()) / 1000) == 16

-- The row grammar's numeric codes (docs/plan.md slice 1), shared by the rows, the pixel strip and questpump-core.
-- Never renumber a code: rows in players' saves and in the helper's mirror carry these numbers. `ding` (5, a character
-- level-up's pump) and the title `ding` (8, "Ding!") stay for the saves and cards that hold them; none is made since
-- 2026-09-30. `load` (18, 2026-10-09): the weight one set of a loaded move was done with, in tenths of a kg in its
-- `reps`, written just before its set row with the same ts, questID and slot (DB.lua's fold; an older reader skips it).
-- `mastered` (19, 2026-10-16, slice 42): a step mastered, its `ladder` and `rung`, written once by ns.Progression after
-- the set that brought the last move the setup offers there to its top (DB.lua's fold keeps the first; an older reader
-- skips it).
ns.Enum = {
	kind = { rep = 1, hold = 2, cardio = 3, flow = 4, ding = 5, pump = 6, correction = 7, rung = 8, pain = 9,
		token = 10, season = 11, checkin = 12, stretch = 13, reset = 14, pause = 15, focus = 16, clear = 17, load = 18,
		mastered = 19 },
	outcome = { beat = 1, miss = 2, placement = 3, censored = 4, done = 5, stopped = 6, offered = 7, owed = 8,
		banked = 9, paid = 10, lastbeat = 11 },
	-- flags bits 0-2 on a move row, only on rows from before 2026-10-04, when the stopped-because row went (unwell
	-- still locks that game day: DB.lua's fold)
	stopped = { form = 1, time = 2, pain = 3, unwell = 4, gaveup = 5 },
	-- flags bits 0-2 on a banked row: 0 from Bank it, unwell on the felt-unwell lock's pump row (the helper's for an
	-- app pump's felt-unwell stop, the addon's for "I feel unwell" with a pause until 04:00); desk, time and pain only
	-- on rows from before 2026-10-04, when Bank it's reason list went
	bank = { desk = 1, time = 2, pain = 3, unwell = 4 },
	-- flags bits 0-2 on an owed row (slice 23, 2026-10-05): 0 a turn-in's or an open pump's move, death the death tax's
	-- (5: none of bank's codes, whose unwell (4) every fold reads on any row; the helper and the phones read only 3 and
	-- 4, so they keep it as any owed row)
	owedBy = { death = 5 },
	checkin = { fine = 1, sore = 2, verySore = 3 },                      -- a checkin row's reps (very sore or dark urine)
	-- a pose family's marking pose's answer to "How close to the goal pose?" (2026-10-09; "How close did you get?"
	-- until 2026-10-14), in its flow row's reps (with the mark bit, flagBit below), from Not yet to Perfect; the ask
	-- lists them from Perfect down (markKeys), each on the key of its code, 4 Perfect to 1 Not yet, so a stray 1 is the
	-- lowest answer (2026-10-31)
	mark = { notyet = 1, okay = 2, good = 3, perfect = 4 },
	markKeys = { "perfect", "good", "okay", "notyet" },
	-- a cardio move row's answer after the move (2026-10-16, slice 42), in flags bits 14-15 (ns.Answer): on a counted
	-- set its answer to "All out and clean?" (slice 53), `answer.slipped` for "Form slipped" and `answer.notAllOut` for
	-- "Not all out" (2026-10-23; no older row holds 2), 0 for "Yes, to the end" (and every counted row written before
	-- the answer), which the ask lists Yes, to the end first (its keys 1-3); on a steady move its answer to "Did you
	-- keep that pace to the end?" (`pace`, 2026-10-17; 0 none: not asked), or Keep going's answer saved without the ask
	-- (slice 52): Yes, just (1, "Yes, the whole time" until slice 52, so an older row reads as the cautious answer), I
	-- eased off (2) and Yes, easily (3, the free code since slice 52), which the ask lists in paceAnswers' order (keys
	-- 1-3)
	answer = { slipped = 1, notAllOut = 2 },
	pace = { just = 1, eased = 2, easily = 3 },
	paceAnswers = { "easily", "just", "eased" },
	-- a rep or hold row's bits 14-15 (slice 48's double-check; ns.Answer, which no rep or hold row used): `set.warm` the
	-- easy set of a match pump's warm move (Engine.lua dose), which the fold counts as no fresh set (DB.lua applyRow)
	set = { warm = 1 },
	-- an app pump's offered row's bits 14-15 (slice 63; ns.Answer, which no pump row used): `by.browser` a pump a
	-- browser ran at play.questpump.com, whose rows carry the phone's bit beside the app's, so the Sagas of the desktop
	-- app, the browser and the phones say "in a browser" (contract/fixtures/rows.json `codes.by`); 0 any other. No game
	-- file reads it: the game reads the pump as the phone's (ns.Source)
	by = { browser = 1 },
	-- a hold row's reps (slice 52): the answer to "Could you have held longer?" after a fresh set near its goal on a
	-- hold ladder of the Stamina Ledger (ns.HoldAsk), 0 not asked; the ask lists them in this order, keys 1-3 the codes
	longer = { no = 1, little = 2, lot = 3 },
	longerKeys = { "no", "little", "lot" },
	-- a clear row's reps (2026-10-08): one move's record cleared (the row's `move`) or one ladder started over (its
	-- `ladder`), for the whole account, from any device
	clear = { move = 1, ladder = 2 },
	-- skipped (David, 2026-10-04): on the paid row of an unfinished move the player skipped (Lc.Skip), which clears it
	-- from the queue with no move done
	-- counted (2026-10-05): a counted cardio set, its typed count in reps (Progress.lua ns.ResultRow; 0 where none was
	-- typed since 2026-10-16), its answer in bits 14-15 (`answer` above, ns.Answer), which a steady cardio row uses for
	-- the pace answer (`pace` above)
	-- app (slice 26): a row an app wrote, its device in the watch, the phone or the computer bit (ns.Source)
	-- computer (slice 35): a row of the desktop app, its own pump's with the app bit, a catch-up the helper wrote for it
	-- without
	-- mark (2026-10-09): a flow row written since the poses' ask, its reps the answer to "How close did you get?"
	-- (`mark` above, 0 none: a pose that did not ask, or one left at its ask); a flow row without it is from before
	flagBit = { confirmed = 3, watch = 4, owed = 5, easier = 6, phone = 7, firstTry = 8, skipped = 9, counted = 10,
		app = 11, computer = 12, mark = 13 },
	-- Where a row came from (slice 26), as ns.Source reads it from the flags: the game's own row, a catch-up the helper
	-- wrote for unfinished moves paid on the phone, the watch or the computer, or an app row (the apps' own pumps); and
	-- the device of a catch-up or an app row. The sheet's Log entries keep both (Stats.lua logEntry), so neither is ever
	-- renumbered.
	source = { game = 1, catchup = 2, app = 3 },
	device = { phone = 1, watch = 2, computer = 3 },
	-- One ladder a family (slice 21). The pull ladder (3) and the core hold (5) split in two on 2026-10-04 (the pull-up
	-- and the row, the core and the side): their ids are retired, never reused, and the fold reads their rows on the
	-- ladders that took their moves (DB.lua, ns.Translate). 20-23 are slice 22's families, 24 the sit-up (2026-10-17),
	-- 25 the lower back (2026-11-10).
	ladder = { push = 1, squat = 2, hinge = 4, cardio = 6, wallsit = 7, pullup = 8, row = 9, core = 10, side = 11,
		frontsplit = 12, midsplit = 13, pike = 14, backbend = 15, twist = 16, ankle = 17, balance = 18,
		armbalance = 19, handstand = 20, lsit = 21, planche = 22, power = 23, situp = 24, lowerback = 25 },
	retired = { pull = 3, corehold = 5 },
	-- The pain flags' joints, in the library's tag order; a pain row carries the id in `move`.
	joint = { wrist = 1, elbow = 2, knee = 3, achilles = 4, back = 5, shoulder = 6, neck = 7, hip = 8 },
	jointKeys = { "wrist", "elbow", "knee", "achilles", "back", "shoulder", "neck", "hip" },
	-- Blizzard's quest tag ids, not ours: the build's QuestInfo rows (1.60.1), GetQuestTagInfo's tagID.
	tag = { elite = 1, life = 21, pvp = 41, raid = 62, dungeon = 81, event = 82, legendary = 83 },
	-- The card protocol's codes (docs/plan.md slice 13): other players' addons read these numbers, so a new title or
	-- cheer takes the next free code and none is ever reused. The titles are Stats.lua's closed list, by id; a title
	-- renamed keeps its code (2: All-Rounder until 2026-10-03, 7: Seven Days, 26: the Learned).
	title = { initiate = 1, tested = 2, recordBreaker = 3, circuitRider = 4, hubHopper = 5, cleanSlate = 6,
		steadfast = 7, ding = 8, starryEyed = 9, relentless = 10, wallSitter = 11, ironLegged = 12, unbroken = 13,
		sprinter = 14, longRoad = 15, wanderer = 16, climber1 = 17, climber2 = 18, climber3 = 19, climber4 = 20,
		climber5 = 21, climber6 = 22, climber7 = 23, mighty = 24, swift = 25, poised = 26, tireless = 27, serene = 28,
		proven = 29, allRounder = 30, exalted = 31, legendary = 32, handOfIron = 33, masterOfTheBar = 34,
		flagbearer = 35, defierOfGravity = 36, worldlifter = 37, wingsOfStone = 38, dragonstrider = 39,
		dragonbane = 40, championOfTheNorth = 41, stormvaulter = 42, stormrunner = 43, duelist = 44, unwavering = 45,
		unfailing = 46, evergreen = 47, undying = 48, paragon = 49, ascendant = 50, ageless = 51, eternal = 52,
		kindled = 53 },
	-- A step's colour (slice 20): Enum.ItemQuality's numbers, Poor shown as "Starter" (C.quality_by_step), Artifact the
	-- elite tier above Legendary since 2026-10-05.
	quality = { starter = 0, common = 1, uncommon = 2, rare = 3, epic = 4, legendary = 5, artifact = 6 },
	-- The player's focus (a focus row's reps; David, 2026-10-03): the day plan's shares it picks (C.focus_share).
	focus = { balanced = 1, strength = 2, endurance = 3, flexibility = 4 },
	focusKeys = { "balanced", "strength", "endurance", "flexibility" },
	-- The player's pace (General > Moves per quest, profile.pace, a key; David, 2026-10-05), in the dropdown's order:
	-- the moves a quest gives (C.pace_moves), the major move on top up to the pace's most (Engine.lua quota).
	paceKeys = { "light", "balanced", "extreme" },
	-- The player's strength training so far (profile.experience; Your setup's question, the phones' too), in the
	-- question's order: the start rungs C.start_rung names.
	experienceKeys = { "none", "some", "training" },
	-- A pact's joint plan (slice 23): the id its offer carries (P?<id>.<plan>) and the profile's pact keeps, never
	-- renumbered: one counted pump every game day, or that and nothing left unfinished.
	pact = { daily = 1, evening = 2 },
	pactKeys = { "daily", "evening" },
	cheer = { cheer = 1, salute = 2, flex = 3 },
}

-- The ladders by role, from their ids: the cardio ladder, the strength and hold ladders (their set and their order),
-- the pose ladders that climb by a mark (slice 21; their set and their order), and every ladder grouped by the stat it
-- feeds in Forever's order (the order of the Talents tab's collection line, Settings' pickers and contract/sheet.json's
-- `order`, the apps' Book).
local LAD = ns.Enum.ladder
ns.CARDIO = LAD.cardio
ns.STRENGTH_ORDER = { LAD.push, LAD.squat, LAD.pullup, LAD.row, LAD.hinge, LAD.core, LAD.side, LAD.wallsit,
	LAD.planche, LAD.power, LAD.lsit, LAD.handstand, LAD.situp, LAD.lowerback }
ns.STRENGTH = {}
for _, l in ipairs(ns.STRENGTH_ORDER) do ns.STRENGTH[l] = true end
ns.POSE_ORDER = { LAD.frontsplit, LAD.midsplit, LAD.pike, LAD.backbend, LAD.twist, LAD.ankle, LAD.balance,
	LAD.armbalance }
ns.POSE = {}
for _, l in ipairs(ns.POSE_ORDER) do ns.POSE[l] = true end
ns.LADDER_ORDER = { LAD.push, LAD.squat, LAD.pullup, LAD.row, LAD.hinge, LAD.lowerback, LAD.planche, LAD.situp,
	LAD.cardio, LAD.power, LAD.core, LAD.side, LAD.wallsit, LAD.lsit, LAD.balance, LAD.armbalance, LAD.handstand,
	LAD.frontsplit, LAD.midsplit, LAD.pike, LAD.backbend, LAD.twist, LAD.ankle }

-- The ladders as three talent trees (docs/plan.md slice 36; David, 2026-10-13: "The three trees are great. Let's go
-- with this."), named as the focus is but read by no focus rule: each tree its branches, each branch its ladders in
-- column order, every live ladder once (tools/lint.lua). A branch is a wider gap between columns with no name of its
-- own (four of the names it would take are ladder names already), so its key names it only here; a tree's name is the
-- locale's `tree.<key>`.
ns.TREES = {
	{ key = "strength", branches = { { key = "push", ladders = { LAD.push, LAD.handstand, LAD.planche } },
		{ key = "pull", ladders = { LAD.pullup, LAD.row } },
		{ key = "legs", ladders = { LAD.squat, LAD.hinge, LAD.situp } } } },
	{ key = "endurance", branches = { { key = "cardio", ladders = { LAD.cardio, LAD.power } },
		{ key = "core", ladders = { LAD.core, LAD.side, LAD.lowerback, LAD.lsit, LAD.wallsit } } } },
	{ key = "flexibility", branches = {
		{ key = "hips", ladders = { LAD.frontsplit, LAD.midsplit, LAD.pike, LAD.ankle } },
		{ key = "spine", ladders = { LAD.backbend, LAD.twist } },
		{ key = "balance", ladders = { LAD.balance, LAD.armbalance } } } },
}
local treeOf = {}
for t, tree in ipairs(ns.TREES) do
	local column = 0
	for b, branch in ipairs(tree.branches) do
		for _, l in ipairs(branch.ladders) do
			column = column + 1
			treeOf[l] = { tree = t, branch = b, column = column }
		end
	end
end

-- A ladder's tree (its index in ns.TREES), its branch's index in that tree and its column from the tree's left (1 the
-- first); nil for a retired id.
function ns.TreeOf(ladder)
	local at = treeOf[ladder]
	if not at then return nil end
	return at.tree, at.branch, at.column
end

-- A tree's ladders in column order, across its branches.
function ns.TreeLadders(tree)
	local out = {}
	for _, branch in ipairs(ns.TREES[tree].branches) do
		for _, l in ipairs(branch.ladders) do out[#out + 1] = l end
	end
	return out
end

-- Flags: bits 0-2 hold a reason (1-5), bits 3-13 are single flags, bits 14-15 a cardio move's answer (ns.Answer).
-- Arithmetic, so no bit library is needed.
function ns.Reason(flags)
	return (flags or 0) % 8
end

-- A cardio move row's answer after the move, bits 14-15 (2026-10-16, slice 42; the strip carries the row's 16 bits):
-- ns.Enum.answer on a counted set, ns.Enum.pace on a steady move; 0 none. A rep or hold row's are ns.Enum.set (slice
-- 48's double-check: a warm move's easy set), an app pump's offered row's ns.Enum.by (slice 63). WithAnswer sets it in
-- place of any.
local ANSWER = 2 ^ 14

function ns.Answer(flags)
	return math.floor((flags or 0) / ANSWER) % 4
end

function ns.WithAnswer(flags, answer)
	flags = flags or 0
	return flags - ns.Answer(flags) * ANSWER + (answer or 0) * ANSWER
end

-- Whether a steady move's answer (ns.Answer) is a Yes to "Did you keep that pace to the end?": any pace answer but I
-- eased off (slice 55: an ember and a Long Haul's day); 0, not asked, is none.
function ns.PaceYes(answer)
	return answer ~= 0 and answer ~= ns.Enum.pace.eased
end

function ns.HasFlag(flags, name)
	return math.floor((flags or 0) / 2 ^ ns.Enum.flagBit[name]) % 2 == 1
end

function ns.WithFlag(flags, name)
	flags = flags or 0
	if ns.HasFlag(flags, name) then return flags end
	return flags + 2 ^ ns.Enum.flagBit[name]
end

-- A row's source and device from its flags (slice 26; the one reader of the source bits): the app bit makes an app
-- row, with exactly one device's bit beside it (11|7 the phone's, 11|4 the watch's, 11|12 the computer's, slice 35),
-- the phone where none is set; without it, a device's bit marks a catch-up the helper wrote; neither, a game row (no
-- device).
function ns.Source(flags)
	local E = ns.Enum
	local device = ns.HasFlag(flags, "watch") and E.device.watch or ns.HasFlag(flags, "phone") and E.device.phone
		or ns.HasFlag(flags, "computer") and E.device.computer or nil
	if ns.HasFlag(flags, "app") then return E.source.app, device or E.device.phone end
	if device then return E.source.catchup, device end
	return E.source.game, nil
end

-- A value the client may hand back as a secret (12.x): nil then, so it is never compared or counted.
function ns.Readable(v)
	if v == nil or issecretvalue(v) then return nil end
	return v
end

-- The clock. Pure files read time only through these two, so the tools' stand-in clock drives them
-- (tools/wow_stub.lua); `/qp dev clock` moves the offset.
ns.clockOffset = 0

function ns.now()
	return time() + ns.clockOffset
end

-- One day function serves every day rule; the day turns at 04:00 local, so a late session is one day.
function ns.day(ts)
	return date("%Y-%m-%d", (ts or ns.now()) - ns.C.day_offset_h * 3600)
end

-- The middle of the game day `n` days after the one `ts` falls in: stepped from the middle, a day of 23 or 25 hours
-- (a clock change falls before 04:00) never lands on the wrong one.
function ns.DayMid(ts, n)
	local t = date("*t", ts - ns.C.day_offset_h * 3600)
	return ts + 12 * 3600 - (t.hour * 3600 + t.min * 60 + t.sec) + (n or 0) * 86400
end

-- The start of the game day after the one `ts` falls in, the moment ns.day turns: 12 h before the next day's middle
-- (ns.DayMid), moved an hour where the day `ts` falls in has 23 or 25 hours (a clock change falls before 04:00, so the
-- middle stepped by 24 h lands an hour off the turn on such a day). The felt-unwell lock's end, a step's next game day,
-- the streamer's and the pacts' day turn.
function ns.NextDayStart(ts)
	local turn, today = ns.DayMid(ts, 1) - 12 * 3600, ns.day(ts)
	if ns.day(turn - 3600) ~= today then return turn - 3600 end
	if ns.day(turn) == today then return turn + 3600 end
	return turn
end

-- Named C_Timer handles, one per purpose, so a feature always cancels what it started (C_Timer.After cannot be
-- cancelled; NewTimer and NewTicker return handles). A callback that raises is reported (ns.Report), as a handler is.
function ns.Timers()
	local handles, set = {}, {}
	function set.cancel(name)
		if handles[name] then
			handles[name]:Cancel()
			handles[name] = nil
		end
	end
	function set.after(name, seconds, fn)
		set.cancel(name)
		handles[name] = C_Timer.NewTimer(math.max(0, seconds), function()
			handles[name] = nil
			local ok, err = ns.Guarded(fn)
			if not ok then ns.Report(err) end
		end)
	end
	function set.every(name, seconds, fn)
		set.cancel(name)
		handles[name] = C_Timer.NewTicker(seconds, function()
			local ok, err = ns.Guarded(fn)
			if not ok then ns.Report(err) end
		end)
	end
	function set.active(name)
		return handles[name] ~= nil
	end
	return set
end

-- An error of the addon's own, reported and never dropped: into the addon's log (Log.lua, once it has loaded) and on to
-- the client's handler, which shows it with /console scriptErrors 1 and writes it to the client's log (the message
-- names the file and line, and, where the call ran guarded, the addon's own frames it came through: ns.Guarded).
function ns.Report(err)
	if ns.Log then ns.Log("E", err) end
	geterrorhandler()(err)
end

-- The client's xpcall hands its extra arguments to the function; PUC Lua 5.1's, which runs the engine's files in the
-- desktop app (tools/host.lua), does not, so there the calls stay plain pcalls (the desktop app logs a Lua error's
-- traceback itself).
local passesArgs = select(2, xpcall(function(arg) return arg end, function() end, true)) == true

-- The addon's files and lines a stack (debugstack's text) came through, the innermost first, compact so a log line
-- holds them (" [Events.lua:120 < Core.lua:236]"), or "" when it came through none.
function ns.StackTrail(stack)
	local frames = {}
	for file, line in stack:gmatch("QuestPump/([%w_/]+%.lua)[\"%]]*:(%d+)") do
		if #frames < 6 then frames[#frames + 1] = file .. ":" .. line end
	end
	return #frames > 0 and (" [" .. table.concat(frames, " < ") .. "]") or ""
end

-- An error's own frames at the moment it was raised, before the stack unwinds.
local function traced(err)
	if type(err) ~= "string" or not debugstack then return err end
	return err .. ns.StackTrail(debugstack(2, 12, 0))
end

-- Calls fn with its arguments under protection: ok, or false and the error with its frames (traced) where the client's
-- xpcall can carry the arguments.
function ns.Guarded(fn, ...)
	if passesArgs then return xpcall(fn, traced, ...) end
	return pcall(fn, ...)
end

-- Runs every function of a list in order. One that raises is reported (ns.Report) and the next one still runs, so a
-- broken view never leaves the lifecycle half-way through a transition and one broken feature never takes the login
-- (the DB, the setup screen, the HUD) down with it.
local function run(list, ...)
	for i = 1, #list do
		local ok, err = ns.Guarded(list[i], ...)
		if not ok then ns.Report(err) end
	end
end

-- The addon's own message bus: the lifecycle announces, the panel, the HUD and speech listen.
local listeners = {}

function ns.On(message, fn)
	listeners[message] = listeners[message] or {}
	table.insert(listeners[message], fn)
end

function ns.Emit(message, ...)
	run(listeners[message] or {}, ...)
end

-- One event frame, a dispatch table with every file's handlers in TOC order (DB.lua binds first at PLAYER_LOGIN).
local handlers = {}
local frame = CreateFrame("Frame")

frame:SetScript("OnEvent", function(_, event, ...)
	run(handlers[event], ...)
end)

local function add(event, fn)
	if handlers[event] then
		table.insert(handlers[event], fn)
		return false
	end
	handlers[event] = { fn }
	return true
end

-- An event this client does not know (a build renamed or dropped it) is refused, since RegisterEvent raises on an
-- unknown name: logged once (Log.lua keeps the lines written before the save loads), so the feature it fed going quiet
-- shows in /qp log.
local refused = {}

function ns.RegisterEvent(event, fn)
	if not C_EventUtils.IsEventValid(event) then
		if not refused[event] and ns.Log then
			refused[event] = true
			ns.Log("W", "event %s is unknown to this client: its handlers never run", event)
		end
		return false
	end
	if add(event, fn) then frame:RegisterEvent(event) end
	return true
end

-- Runs the handlers of an event as the client would; the debug commands use it to stand in for real events.
function ns.Dispatch(event, ...)
	run(handlers[event] or {}, ...)
end

-- A count in words: the key with ".one" when n is 1 and the locale has it, else the key itself.
function ns.Plural(key, n)
	if n == 1 and rawget(L, key .. ".one") then return L[key .. ".one"] end
	return L[key]
end

function ns.Print(key, ...)
	print(L["chat.prefix"] .. " " .. L[key]:format(...))
end

-- /qp <name> <args>; each command registers itself with one help line.
local commands, order = {}, {}

function ns.AddCommand(name, fn, helpKey)
	commands[name] = { fn = fn, help = helpKey }
	order[#order + 1] = name
end

-- /qp help's groups, by what a player does with them, the diagnostics last (2026-10-16): each its locale key and its
-- commands in order. A command with a help line that no group names joins the last group (tools/lint.lua fails on
-- one); a name no command took (/qp lock, only where LibEditMode is missing) is left out.
local HELP_GROUPS = {
	{ "cmd.help.group.play", { "pump", "try", "pause", "resume", "instances", "safety" } },
	{ "cmd.help.group.sheet", { "stats", "card", "qr", "focus", "why", "how", "undo" } },
	{ "cmd.help.group.setup", { "config", "setup", "gear", "lock" } },
	{ "cmd.help.group.others", { "player" } },
	{ "cmd.help.group.data", { "export", "import", "reset", "restore-backup", "save" } },
	{ "cmd.help.group.help", { "help", "status", "feedback", "log", "bridge", "ping" } },
}

-- A command's usage: its help line's words before the first ": " ("/qp pause [hour|4h|char]").
local function usage(cmd)
	local line = L[cmd.help]
	return line:match("^(.-): ") or line
end

-- The help's groups as /qp help prints them, each { key, usages }, and the commands with a help line no group names.
function ns.HelpGroups()
	local groups, named, loose = {}, {}, {}
	for _, group in ipairs(HELP_GROUPS) do
		local usages = {}
		for _, name in ipairs(group[2]) do
			named[name] = true
			local cmd = commands[name]
			if cmd and cmd.help then usages[#usages + 1] = usage(cmd) end
		end
		groups[#groups + 1] = { key = group[1], usages = usages }
	end
	for _, name in ipairs(order) do
		local cmd = commands[name]
		if cmd.help and not named[name] then
			loose[#loose + 1] = name
			table.insert(groups[#groups].usages, usage(cmd))
		end
	end
	return groups, loose
end

-- One command's whole help line, or nil for a name that has none.
function ns.HelpLine(name)
	local cmd = commands[name]
	return cmd and cmd.help and L[cmd.help] or nil
end

SLASH_QUESTPUMP1 = "/qp"
SlashCmdList.QUESTPUMP = function(msg)
	local name, rest = strtrim(msg or ""):match("^(%S*)%s*(.-)$")
	name = name:lower()
	if name == "" then
		local _, build, _, toc = GetBuildInfo()
		ns.Print("chat.version", ns.version, tostring(build), tostring(toc))
		return
	end
	local cmd = commands[name]
	if not cmd then
		ns.Print("chat.unknown", name)
		return
	end
	local ok, err = ns.Guarded(cmd.fn, rest)
	if not ok then ns.Report(err) end
end

-- The help names the companions the setup asks for (Setup.AsksPhone) as strongly recommended, never with a URL, a
-- price or a store (docs/tasks.md, "Legal minimum"), then a line a group ("Play: /qp pump · /qp try · ..."); /qp help
-- <command> prints that command's whole line.
ns.AddCommand("help", function(arg)
	local name = strtrim(arg or ""):lower()
	if name ~= "" then
		local line = ns.HelpLine(name)
		if not line then return ns.Print("chat.unknown", name) end
		return print(line)
	end
	ns.Print(ns.Setup.AsksPhone() and "cmd.help.companions" or "cmd.help.companions.one")
	for _, group in ipairs((ns.HelpGroups())) do
		if #group.usages > 0 then
			print(L["cmd.help.line"]:format(L[group.key], table.concat(group.usages, " · ")))
		end
	end
end, "cmd.help.help")

-- The AddOn Compartment click opens the sheet (Sheet.lua); before it loads, the status.
function QuestPump_OnAddonCompartmentClick()
	if ns.ToggleSheet then ns.ToggleSheet() else SlashCmdList.QUESTPUMP("status") end
end
