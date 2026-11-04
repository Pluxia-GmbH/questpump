-- The pump lifecycle, one visible state machine (docs/plan.md slice 1): settling → ready → card → open → countdown →
-- running → result, with waiting and held for combat. It owns the gates above it (no pump before setup; the felt-unwell
-- day lock), the hub merge (a pump merges only the turn-ins made before it opens), the owed queue (moves banked,
-- held through a logout, earned in an instance or a mixed party, every one paid with the next pump or at once on
-- demand, or skipped by the player one at a time), a mixed party's choice (bank its turn-ins or pause while grouped),
-- the unsafe list, the popups and cinematics that pause a pump, and every row a pump writes. The panel, the HUD and
-- speech only draw what it announces through ns.Emit("State", pump).

local ADDON, ns = ...
local C, E, L = ns.C, ns.Enum, ns.L
local K, O = E.kind, E.outcome

local Lc = {
	pump = nil,      -- the current pump
	queue = {},      -- pumps that begin after it (turn-ins made after it opened)
	npc = {},        -- open quest, gossip, merchant and other NPC windows, by source
	holds = {},      -- a popup, a cinematic or the game menu over an open pump
	mirrorTimers = {}, -- breath and fatigue
	seen = {},       -- quests whose turn-in counts (accepted this session or shown in a quest-complete window)
	info = {},       -- what the client said about a quest while it was in the log
	combat = false,
	stillFor = 0,
	movingFor = 0,
}
ns.Lifecycle = Lc

-- A mixed party's question (Lc.AskParty): `asked` once this group has had it (the popup, or "While grouped" picked in
-- a pause menu), `pause` while "Pause while grouped" is the group's answer, `waiting` the turn-ins that wait for the
-- popup's answer.
local party = { waiting = {} }
local holdWhile -- the hold a popup, a cinematic or a movie puts on a pump (below, with the events that announce one)

local Setup = { dismissed = false }
ns.Setup = Setup

local PENDING = { settling = true, ready = true, card = true }
local VISIBLE = { open = true, countdown = true, running = true, result = true }
local MERGES = { quest = true, standup = true } -- pending pumps a turn-in joins
local NPC_FRAMES = { "QuestFrame", "GossipFrame", "MerchantFrame", "TaxiFrame", "FlightMapFrame", "MailFrame",
	"ClassTrainerFrame", "BankFrame", "ImmersionFrame" }

-- Timers ---------------------------------------------------------------------------------------------------------

local T = ns.Timers()
local cancel, after, every = T.cancel, T.after, T.every

local function cancelPumpTimers()
	for _, name in ipairs({ "settle", "tick", "go", "runEnd", "advance" }) do cancel(name) end
end

-- Gates ----------------------------------------------------------------------------------------------------------

function Lc.TrialBegun()
	if Lc.trialThisSession then return true end
	return ns.meta ~= nil and (ns.meta.trialAt ~= nil or (ns.state ~= nil and ns.state.anyMove))
end

-- Before setup (the first login, until the setup window is finished, or a paired player's Your setup answered; while
-- the setup asks for no phone, Setup.AsksPhone, any player's answered Your setup, as a paired one's) no pump runs and
-- a turn-in counts for nothing and owes nothing; after it the game starts at once (no question about pain comes
-- before a pump: lostNote).
function Setup.Blocks()
	return not Lc.TrialBegun() and not ((ns.IsPaired() or not Setup.AsksPhone()) and Setup.Answered())
end

function Lc.Locked()
	return ns.state ~= nil and ns.LockedToday(ns.state, ns.now())
end

-- A mixed party: a group outside a rested area with a member not known to run QuestPump (Social.lua's cards say who
-- does), so friends never wait through a circuit. When every member runs it, the pump comes as it does alone.
function Lc.MixedParty()
	return IsInGroup() and not IsResting() and not ns.Social.AllMembersRun()
end

-- A turn-in that is owed instead of offered: battlegrounds and arenas always, an instance unless /qp instances on,
-- a mixed party.
function Lc.OwingContext()
	local inInstance, kind = IsInInstance()
	if kind == "pvp" or kind == "arena" then return "pvp" end
	if inInstance and not ns.profile.instances then return "instance" end
	if Lc.MixedParty() then return "party" end
	return nil
end

local function frameShown(name)
	local f = _G[name]
	return f ~= nil and f.IsShown ~= nil and f:IsShown()
end

local function npcFrameShown()
	for _, name in ipairs(NPC_FRAMES) do
		if frameShown(name) then return true end
	end
	return false
end

-- Blizzard's UI panels: a window in one of the panel manager's slots.
local function panelSlotShown()
	for _, key in ipairs({ "left", "center", "right", "doublewide", "fullscreen" }) do
		if GetUIPanel(key) then return true end
	end
	return false
end

-- A panel a pump waits behind: Blizzard's, the sheet, which stands where one does but outside the panel manager
-- (Sheet.lua: a panel the manager pushes aside runs its scripts inside Blizzard's code, 2026-10-05), the loot and the
-- mail windows.
local function panelShown()
	return panelSlotShown() or frameShown("QuestPumpSheet") or frameShown("LootFrame") or frameShown("MailFrame")
end

-- The unsafe list: nothing opens while one of these holds; the pump waits for the next safe moment. A pump the player
-- asks for (the chip, the keybinding, /qp pump) opens over an NPC window, a UI panel or the chat box they just typed
-- in, never while dead, swimming, falling, in a vehicle, a cinematic, a popup or a battleground.
function Lc.Unsafe(asked)
	if Lc.combat or InCombatLockdown() then return "combat" end
	if UnitIsDeadOrGhost("player") then return "dead" end
	if IsSwimming() or IsSubmerged() then return "swimming" end
	if IsFalling() then return "falling" end
	if UnitInVehicle("player") or UnitHasVehicleUI("player") then return "vehicle" end
	if InCinematic() or IsInCinematicScene() then return "cinematic" end
	if next(Lc.mirrorTimers) then return "breath" end
	if not asked then
		if GetCurrentKeyBoardFocus() then return "typing" end
		if next(Lc.npc) or npcFrameShown() then return "npc" end
		if panelShown() then return "panel" end
	end
	if StaticPopup_IsAnyDialogShown() or ns.Dialog.AnyShown() then return "popup" end
	if next(Lc.holds) then return "hold" end
	local _, kind = IsInInstance()
	if kind == "pvp" or kind == "arena" then return "pvp" end
	return nil
end

-- Standing still: speed 0 (guarded, it can be secret), or on a taxi once the taxi map has closed. nil when unknown.
function Lc.Moving()
	if UnitOnTaxi("player") then return false end
	local speed = GetUnitSpeed("player")
	if issecretvalue(speed) then return nil end
	return speed > 0
end

-- Pumps ----------------------------------------------------------------------------------------------------------

local seq = 0

local function setState(p, state)
	p.state = state
	ns.Emit("State", p)
end

local function newPump(kind)
	seq = seq + 1
	return { id = seq, kind = kind, quests = {}, results = {}, index = 1, state = "settling", ts = ns.now(), moves = {} }
end

-- Owed moves already riding a pending pump are not offered to another one.
local function availableOwed(except)
	local taken = {}
	local function scan(p)
		if not p or p == except then return end
		for i, slot in ipairs(p.moves) do
			if slot.owed and not p.results[i] then taken[slot.owed.key] = true end
		end
	end
	scan(Lc.pump)
	for _, p in ipairs(Lc.queue) do scan(p) end
	local list = {}
	for _, r in ipairs(ns.OwedList(ns.state)) do
		if not taken[r.key] then list[#list + 1] = r end
	end
	return list
end

-- What the engine's filters read (ns.FilterContext's switches, equipment and exclusions, and the pain flags) as one
-- string: a pump composed under other filters is composed again or re-fitted (below, refilter). The favourites are a
-- preference, not a filter: a new one leads from the next pump.
local function filterKey()
	local f, parts = ns.FilterContext(ns.profile, ns.state), {}
	for _, k in ipairs({ "floor", "space", "quiet", "noJump", "noStrain", "noHeadDown" }) do
		parts[#parts + 1] = f[k] and k or ""
	end
	for _, set in ipairs({ f.gear, f.exclude, f.include, f.pain }) do
		local keys = {}
		for k, on in pairs(set) do
			if on then keys[#keys + 1] = tostring(k) end
		end
		table.sort(keys)
		parts[#parts + 1] = table.concat(keys, ",")
	end
	return table.concat(parts, "|")
end

-- `as`: the kind the engine composes for, where it is not the pump's own (a death's: a pump asked for, deathTax). A
-- move started from the Talents tab (`p.chosen`, Lc.StartMove) composes as its one slot on every call, the first
-- opening's and a recompose's alike, with the owed list beside it, so the engine pays its oldest owed row of that
-- shape where one fits (Engine.lua chosenSlot) and no other owed move rides.
local function compose(p, noRiders, as)
	if p.started then return end -- a started pump keeps its moves
	p.filterKey = filterKey()
	-- a pump keeps the pace its first quest was composed at, so a new "Moves per quest" sizes the turn-ins after it and
	-- never a pump on its way: queued, on the "Stop to pump" card or open
	if #p.quests > 0 then p.pace = p.pace or ns.Pace(ns.profile.pace) end
	p.moves = ns.Compose({ kind = as or p.kind, quests = p.quests, owed = not noRiders and availableOwed(p) or nil,
		state = ns.state, profile = ns.profile, now = ns.now(), pace = p.pace, chosen = p.chosen })
	Lc.lastComposed = p
end

local function maxTier(p)
	local t = 0
	for _, slot in ipairs(p.moves) do
		if not slot.owed and slot.tier > t then t = slot.tier end
	end
	return t
end

-- Writes the row of one move that goes to the owed queue. Keys (ts, questID, slot) stay unique inside one batch. `ts`
-- is the time the row carries, now unless the caller names one (a logout: the pump's offered time, below).
local function oweSlot(p, i, slot, outcome, flags, ts)
	-- a rider was owed already; practice stores nothing; a move the player asked for with nothing owed was voluntary
	if slot.owed or p.kind == "practice" or p.kind == "demand" then return end
	local s = i - 1
	ts = ts or ns.now()
	while ns.state.owed[ns.OwedKey({ ts = ts, questID = slot.questID, slot = s })] do s = s + 1 end
	-- an all-out window, a finisher or a closing window (from cardio rung closing_rung) alike, is owed as its window
	-- (reps 1) under its form's id, a counted set too, never with its count
	local timed = slot.kind == K.cardio or slot.kind == K.flow
	ns.AddRow({ ts = ts, questID = slot.questID, tier = slot.tier, slot = s, ladder = slot.ladder, rung = slot.rung,
		move = slot.move, kind = slot.kind, reps = slot.allout and 1 or 0,
		seconds = timed and slot.seconds or 0, outcome = outcome or O.owed, flags = flags or 0 })
end

-- The stretch belongs to the quests: a pump that pays none (unfinished moves, the stand-up move, one asked for with
-- nothing owed) leaves it to the next pump that does.
local function takeStretch(p)
	if #p.quests == 0 then return end
	local s = ns.Stretch.Take()
	p.stretch = s
	local shares = ns.Stretch.Split(s, #p.quests)
	for i, q in ipairs(p.quests) do
		q.share = shares[i]
		q.info.footYards = shares[i].foot
		q.family = ns.Family(q.info)
	end
end

-- The stretch the pump's quests paid, as a row of its own right after the pump's (the yards on foot in `reps`, the
-- active seconds in `seconds`, each at most 65,535, the strip's uint16): the sheet counts the miles and knows a travel
-- pump by it.
local function writeStretch(p, ts)
	local s = p.stretch
	if not s then return end
	local first = p.quests[1]
	ns.AddRow({ ts = ts, questID = first and first.questID or 0, tier = maxTier(p), kind = K.stretch,
		reps = math.min(65535, math.floor(s.foot + 0.5)), seconds = math.min(65535, math.floor(s.active + 0.5)) })
end

-- A pump's open moves go to the owed queue (a logout, a turn-in before a held pump resumes). The move in its result
-- view was performed: it is recorded as done without a count (a typed count is recorded at once; a counted set's
-- window with none, slice 42: its count is the player's, never its seconds, and Form slipped, since its form was never
-- answered, so it counts no clean day: 2026-10-19, the program's double-check; at its "Did you do the whole move?" ask
-- too), a timed move that asks whether the whole move was done as done with its whole seconds, a pose asking how close
-- the player got with no answer (practice: DB.lua markAttempt), a hold asking whether it could have gone longer as
-- typed with no answer (slice 52), never owed again. `ts` dates the owed rows of an opened
-- pump (the logout's, below); nil is now.
function Lc.OweOpen(p, ts)
	if p.kind == "standup" and not p.opened then return end -- it pays no turn-in: dropped, as promote drops it
	if not p.opened and not p.started then -- dosed as it would have opened: its quests' stretch, its cardio seconds
		takeStretch(p)
		compose(p)
		writeStretch(p, ns.now())
	end
	local performed = p.state == "result" or p.heldFrom == "result"
	for i, slot in ipairs(p.moves) do
		if not p.results[i] then
			local pend = performed and i == p.index and (p.longerAsk or p.pour) and p.pending
			if pend then
				-- a hold left at "Could you have held longer?": its typed set, no answer; a hold whose seconds still
				-- pour (slice 65): saved as its landing would (Lc.Commit's row, its answer kept); only Back drops it
				Lc.WriteResult(p, i, pend.outcome, pend.value, pend.confirmed, p.pour and pend.answer or nil, pend.load)
			elseif performed and i == p.index then -- its count never typed: done, a timed move with its seconds
				Lc.WriteResult(p, i, O.done,
					(slot.kind == K.cardio or slot.kind == K.flow) and not slot.counted and slot.seconds or 0, nil,
					slot.counted and E.answer.slipped or nil)
			else
				oweSlot(p, i, slot, nil, nil, p.opened and ts or nil)
			end
		end
	end
end

-- A turn-in composed on its own and owed at once (the felt-unwell lock, an instance, a mixed party); its
-- stretch row carries the tier it composed to.
local function owePump(p)
	takeStretch(p) -- a turn-in's quest takes its stretch as an offered one does (its share, its family)
	compose(p, true)
	writeStretch(p, ns.now())
	for i, slot in ipairs(p.moves) do oweSlot(p, i, slot) end
	ns.Emit("Owed", p)
end

local function stopTicker()
	cancel("tick")
end

local function startTicker()
	if T.active("tick") then return end
	Lc.lastTick = GetTime()
	every("tick", C.lifecycle_tick_s, function() Lc.Tick() end)
end

-- A pump the player asks for (the chip, the keybinding, /qp pump, /qp try, a step card's Start): asked, so when it
-- waits again (a fight, a snooze) it reopens over the NPC window, the panel (the sheet a Start came from) or the chat
-- box it was asked over (Lc.Tick, Lc.Unsafe(p.asked)); it opens at once standing still, and moving (auto-run, a
-- mount), where the dim would swallow the keys that stop the character, it waits for the stop on the "Stop to pump"
-- card.
local function ask(p)
	p.asked = true
	if Lc.Moving() then
		Lc.stillFor, Lc.movingFor = 0, 0
		setState(p, "card")
		startTicker()
		return
	end
	Lc.Open()
end

function Lc.CheckSettle()
	local p = Lc.pump
	if not p or p.state ~= "settling" then return end
	startTicker()
	if next(Lc.npc) then
		cancel("settle")
		return
	end
	after("settle", C.settle_s, function()
		if Lc.pump == p and p.state == "settling" and not next(Lc.npc) then
			Lc.stillFor, Lc.movingFor = 0, 0
			p.graceOver = true -- from here a turn-in begins the next pump
			setState(p, "ready")
		end
	end)
end

local function begin(p)
	Lc.pump = p
	compose(p)
	if Lc.combat then
		setState(p, "waiting")
	else
		setState(p, "settling")
		Lc.CheckSettle()
	end
end

-- Whether a queued pump begins as the pump on screen ends (promote), the one rule of it: a turn-in's pump waits behind
-- it and the player is not where a turn-in is owed (an instance, a mixed party, the felt-unwell lock). The "State"
-- message of a pump's end comes before promote runs, so its listeners ask this to know whether a pump asks at once.
function Lc.QueueBegins()
	return #Lc.queue > 0 and not (Lc.Locked() or Lc.OwingContext())
end

-- The next queued pump begins (Lc.QueueBegins); else every queued pump is owed (a stand-up move is dropped).
local function promote()
	while #Lc.queue > 0 do
		if Lc.QueueBegins() then return begin(table.remove(Lc.queue, 1)) end
		local p = table.remove(Lc.queue, 1)
		if p.kind ~= "standup" then Lc.OweOpen(p) end
		ns.Emit("Owed", p)
	end
end

function Lc.Finish(how)
	local p = Lc.pump
	if not p then return end
	cancelPumpTimers()
	Lc.pump = nil
	p.state = "done"
	ns.Emit("State", p, how)
	promote()
end

-- A save replaced under the addon (/qp reset all, a restored backup): the pump, the queue and the last typed count
-- belong to the save that goes, and end unwritten.
function Lc.Drop()
	wipe(Lc.queue)
	Lc.lastTyped = nil
	-- the pause while grouped the save that goes held, and the turn-ins a mixed party's question still holds
	party.pause = nil
	if #party.waiting > 0 then
		wipe(party.waiting)
		ns.Emit("PartyAsk", false)
	end
	Lc.Finish("reset")
end

local function enqueue(p)
	if Lc.pump then
		compose(p)
		Lc.queue[#Lc.queue + 1] = p
		ns.Emit("State", Lc.pump)
	else
		begin(p)
	end
end

function Lc.Counts(questID)
	if Lc.seen[questID] then return true end
	if C_QuestLog.IsQuestTask and C_QuestLog.IsQuestTask(questID) then return true end
	if C_QuestLog.IsWorldQuest and C_QuestLog.IsWorldQuest(questID) then return true end
	return false
end

-- The difficulty score's inputs for a quest the client told us about and the quest table maps (info.mapped: its xr,
-- level, tag, boss, finale and group; the stretch so far stands for the travel part; the tier is fixed at the
-- turn-in, so the chip's count never changes under the player). Money stands in for XP only for a quest the table
-- does not map: a mapped quest without xr pays no XP by the game's own record, and scores the neutral middle.
function Lc.ScoreInput(info, xp, money)
	local m = info.mapped
	return { xp = xp, xpMax = info.xpMax or ns.Readable(UnitXPMax("player")), xr = m and m.xr,
		money = not m and money or nil,
		level = info.level or (m and m.lvl), difficulty = info.difficulty, levelDiff = info.levelDiff,
		trivialRange = info.trivialRange, tagID = info.tagID or (m and m.tag > 0 and m.tag or nil),
		objectives = info.objectives, required = info.required, trivial = info.trivial,
		footYards = ns.Stretch.Current().foot, boss = ns.Family(info) == "boss" or (m and m.boss) or nil,
		finale = m and m.fin or nil, group = m and m.group or nil }
end

-- The lines are English only in v1: a client in another language shows and speaks its own move line alone and
-- never loads them (docs/plan.md slice 3, "Public-addon hygiene").
local function english()
	return GetLocale():sub(1, 2) == "en"
end

-- A quest's flavour line for a kind of move (ns.Enum.kind), or nil.
function Lc.QuestFlavour(questID, kind)
	local lines = kind and english() and ns.Lines(questID)
	return lines and lines[kind] or nil
end

-- The flavour line of a pump's move: the line its own quest has for that kind of move. An unfinished move riding
-- along belongs to no quest of this pump, so it has none.
function Lc.Flavour(p, slot)
	if not slot or slot.owed then return nil end
	for _, q in ipairs(p.quests or {}) do
		if q.questID == slot.questID then return Lc.QuestFlavour(q.questID, slot.kind) end
	end
	return nil
end

-- A quest's title, nil until the client has it cached: asked for once a session (a failed load keeps the caller's
-- fallback), and "QuestTitle" goes out with the quest's id when it comes, so whatever showed the fallback draws again
-- (the Log's rows, the pump card's title).
local titleAsked = {}

function Lc.QuestTitle(questID)
	local title = C_QuestLog.GetTitleForQuestID(questID)
	if not title and not titleAsked[questID] and QuestEventListener and QuestEventListener.AddCallback then
		titleAsked[questID] = true
		QuestEventListener:AddCallback(questID, function() ns.Emit("QuestTitle", questID) end)
	end
	return title
end

-- The lines' folder loads at login, under the loading screen, out of combat, and never inside ADDON_LOADED; a /reload
-- in a fight loads it a moment after the fight ends, once the pump that waited has resumed. A missing or disabled
-- folder leaves the pumps without lines.
local LINES_ADDON = "QuestPump_Lines"

function Lc.LoadLines()
	if Lc.linesAsked or not english() then return end
	if Lc.combat or InCombatLockdown() then
		Lc.linesWaiting = true
		return
	end
	Lc.linesAsked, Lc.linesWaiting = true, nil
	if C_AddOns.DoesAddOnExist(LINES_ADDON) then C_AddOns.LoadAddOn(LINES_ADDON) end
end

-- Every turn-in is a pump (David, 2026-09-21); its score picks the tier.
function Lc.TurnIn(questID, xp, money)
	if not ns.db or Setup.Blocks() or not Lc.Counts(questID) then return end
	Lc.seen[questID] = nil -- a repeatable quest counts again when it is taken again
	local info = Lc.info[questID] or { mapped = ns.Mapping(questID) }
	local score, tier, parts = ns.Score(Lc.ScoreInput(info, xp or 0, money or 0))
	Lc.Add({ questID = questID, tier = tier, score = score, parts = parts, xp = xp or 0, money = money or 0,
		info = info, family = ns.Family(info), themes = info.mapped and info.mapped.th })
end

-- The General page's switch, or this character's pause (ns.Paused): off, turn-ins and the stand-up move count for
-- nothing and owe nothing (the addon sleeps, as it would disabled in the AddOns list); a pump the player asks for still
-- runs, and owed moves wait.
-- Returns ns.Paused's mode and its end (the HUD shows the end), or nil before the save is bound.
local function paused()
	if ns.state == nil or ns.charIndex == nil then return nil end
	return ns.Paused(ns.state, ns.charIndex, ns.now())
end
Lc.Paused = paused

local function asleep()
	return (ns.profile ~= nil and ns.profile.enabled == false) or paused() ~= nil
end
Lc.Asleep = asleep

local function writePause(mode)
	ns.AddRow({ kind = K.pause, reps = mode }) -- its "Row" redraws the HUD
	Lc.pauseSaid = nil
end

-- Pauses this character (ns.PAUSE: hour, fourHours, day, char, group) or, with 0, resumes it: a row, so the mirror
-- keeps it on the beta. A pump already on its way stays: the pause counts the turn-ins after it (the pump's own Pause
-- button ends it, Lc.PausePump). The player's choice is the mixed party's answer too (Lc.AskParty): paused while
-- grouped, which holds only in a mixed party, or no longer.
function Lc.Pause(mode)
	if not ns.db or (mode == ns.PAUSE.group and not Lc.MixedParty()) then return end
	party.pause = mode == ns.PAUSE.group
	if party.pause then party.asked = true end
	writePause(mode)
end

function Lc.Resume()
	if paused() then Lc.Pause(0) end
end

-- A pause while grouped ends once the party is no longer mixed (the player left the group, entered a rested area or
-- learnt that every member runs QuestPump; Lc.CheckParty) and at a login outside a mixed party: a resume row. The
-- group's answer stays (party.pause), so the pause comes back with the mixed party.
local function endGroupPause()
	if paused() == ns.PAUSE.group then writePause(0) end
end

-- The player's focus (ns.Focus): a row, account-wide as the body is one, so every character and the mirror keep it;
-- the next pump follows it. "Later" on the QP 10 ask writes Balanced, so the ask never comes back.
function Lc.SetFocus(key)
	if not (ns.db and ns.Enum.focus[key]) then return end
	ns.AddRow({ kind = K.focus, reps = ns.Enum.focus[key] })
end

-- A turn-in that counts: owed where a turn-in is owed, else merged into the pump on its way or the next pump.
local function countTurnIn(q)
	ns.Emit("TurnIn", q) -- the strip paints it (Bridge.lua), whether the pump runs or is owed
	local p = Lc.pump
	if Lc.Locked() or Lc.OwingContext() then
		local owed = newPump("quest")
		owed.quests[1] = q
		owePump(owed)
		return
	end
	-- A pump merges the turn-ins made in the same NPC conversation or inside the 1-s grace (also while combat keeps it
	-- waiting); once the grace is over, a turn-in at the next quest giver begins the next pump.
	if p and not p.opened and not p.graceOver and MERGES[p.kind] then
		p.kind = "quest"
		p.quests[#p.quests + 1] = q
		compose(p)
		if p.state == "waiting" then
			ns.Emit("State", p)
		else
			setState(p, "settling")
			Lc.CheckSettle()
		end
		return
	end
	-- a turn-in before a held pump resumes (in combat or after it, until it reopens) puts its open moves in the queue;
	-- a snoozed one resumes as it was, the turn-in queued behind it
	if p and p.started and not p.snooze and (p.state == "held" or PENDING[p.state]) then
		Lc.OweOpen(p)
		Lc.Finish("owed")
	end
	local np = newPump("quest")
	np.quests[1] = q
	enqueue(np)
end

-- A mixed party's question (David, 2026-10-04, after an hour grouped with a friend who does not run QuestPump left 33
-- unfinished moves: "Yes offer the options at the first return. Bank all or Pause during this group up."): the
-- group's first turn-in asks once (Panel.lua's popup, on "PartyAsk") and waits for the answer with the turn-ins made
-- meanwhile. "Bank all", and closing or ignoring the popup, keeps the rule: this group's turn-ins go unfinished.
-- "Pause while grouped" makes them count for nothing, the waiting ones too, while the party is mixed: a pause row with
-- its own code (ns.PAUSE.group), which Lc.CheckParty ends with a resume row and writes again when the same group
-- turns mixed again. The answer holds until the player leaves the group. Never in a battleground or an arena. Returns
-- whether the turn-in waits.
function Lc.AskParty(q)
	if party.asked or not Lc.MixedParty() or Lc.OwingContext() == "pvp" then return false end
	party.waiting[#party.waiting + 1] = q
	if #party.waiting == 1 then
		ns.Emit("PartyAsk", true)
		-- a hold, as every popup is: a pump on the screen hides for it (its dim would cover the popup) and a running
		-- move pauses; none when the client refused the popup and the answer came at once
		if #party.waiting > 0 then holdWhile("popup") end
	end
	return true
end

-- The answer: `pause` true for "Pause while grouped", false for "Bank all" and every other close of the popup (Escape,
-- its timeout), nil when the question went unanswered (the client refused the popup, the party stopped being mixed, a
-- logout), which asks the group again at its next turn-in. All but the pause take the waiting turn-ins as they count
-- now.
function Lc.PartyAnswer(pause)
	if #party.waiting == 0 then return end -- answered already: the popup hides after its button too
	local waiting = party.waiting
	party.waiting = {}
	if pause and not Lc.MixedParty() then pause = nil end -- the party changed under the popup
	if pause ~= nil then party.asked = true end
	if pause then return Lc.Pause(ns.PAUSE.group) end
	for _, q in ipairs(waiting) do countTurnIn(q) end
end

function Lc.Add(q)
	if asleep() then
		-- once a session, so a turn-in that counts for nothing never surprises the player
		if paused() and not Lc.pauseSaid then
			Lc.pauseSaid = true
			ns.Print("chat.paused")
		end
		return
	end
	if Lc.AskParty(q) then return end -- a mixed party's first turn-in waits for the group's answer
	countTurnIn(q)
end

-- A pump that has not opened when the player enters an instance or the party turns mixed is owed like a turn-in made
-- there, so friends never wait through it; the queue behind it too. A stand-up move is simply dropped (it pays no
-- turn-in), its pump ending "dropped", never "owed" (nothing of it comes back), and owed moves riding along stay owed.
function Lc.OweIfOwing()
	if not ns.db or not Lc.OwingContext() then return end
	for _, q in ipairs(Lc.queue) do -- the queue behind a pump, whatever state the pump itself is in
		if q.kind ~= "standup" then Lc.OweOpen(q) end
	end
	wipe(Lc.queue)
	local p = Lc.pump
	if not p or p.started or VISIBLE[p.state] or p.kind == "practice" then return end
	if p.kind == "standup" then
		Lc.Finish("dropped")
	else
		Lc.OweOpen(p)
		Lc.Finish("owed")
	end
	ns.Emit("Owed", p)
end

-- The game menu, and every window the player opens from it (Options, Support, AddOns, Edit Mode, Macros), keep an
-- open pump hidden; the menu hides before the window it opened shows, so the hold ends only when none shows. The
-- addon's own feedback, export and Correct a count windows (Settings.lua, opened from Options, the sheet or by /qp)
-- hold it alike: under the dim their box would take the keys meant for the card. The sheet is none of them: a Start
-- from the Talents tab keeps it open under its pump, which a hold the sheet kept would leave hidden, and every Start
-- refused, until it closed.
local MENU_WINDOWS = { "SettingsPanel", "HelpFrame", "QuestPumpFeedback", "QuestPumpIO", "QuestPumpCorrect" }

local function menuWindowShown()
	for _, name in ipairs(MENU_WINDOWS) do
		if frameShown(name) then return true end
	end
	return panelSlotShown()
end

function Lc.MenuShown()
	cancel("menu")
	Lc.Hold("menu")
end

function Lc.MenuHidden()
	every("menu", C.lifecycle_tick_s, function()
		if not menuWindowShown() then
			cancel("menu")
			Lc.Unhold("menu")
		end
	end)
end

-- No question comes before a pump: it opens on its Ready view at once (David, 2026-10-04, to removing the blocking
-- question: "yes do that"; until then "Any pain right now?" opened a pump while the player had done no move yet, and
-- the first pump of a session on a lost save). The pain flags are the player's to set: "Something hurts" in the Pause
-- menu, the Body page, the check-in card. They are a safety filter and must never reset silently, so the first pump
-- of a session while the beta lost the save and no mirror restored it carries one line on its Ready view, which says
-- where to set them again. True for that pump, once a session.
local function lostNote(p)
	if p.kind == "practice" or Lc.lostNoted or not ns.svBroken or ns.meta.restoredFrom == "mirror" then return nil end
	Lc.lostNoted = true
	return true
end

local function writeOffered(p)
	local first = p.quests[1]
	local ts = ns.now()
	p.offeredTs = ts -- the pump's identity on the strip (Bridge.lua) and the time its moves are owed at a logout
	ns.AddRow({ ts = ts, questID = first and first.questID or 0, tier = maxTier(p), kind = K.pump, reps = #p.moves,
		seconds = #p.quests, outcome = O.offered })
	writeStretch(p, ts)
end

function Lc.Open()
	local p = Lc.pump
	if not p or VISIBLE[p.state] or p.state == "waiting" or p.state == "held" then return end
	stopTicker()
	cancel("settle")
	if not p.opened then
		p.opened = true
		if p.kind ~= "practice" then
			takeStretch(p)
			compose(p)
			writeOffered(p)
			Lc.lastPumpAt = ns.now()
			Lc.ScheduleStandup()
		end
		p.lostNote = lostNote(p)
	end
	p.taxi = UnitOnTaxi("player") and true or false -- no keyboard capture on a taxi
	p.snooze = nil
	local target = (p.heldFrom == "result" or (p.results[p.index] and p.committed)) and "result" or "open"
	if target == "open" and p.view ~= "unwellAsk" then p.view = "ready" end
	p.heldFrom = nil
	setState(p, target)
end

-- The X on a Ready view (David, 2026-10-04: "an X on the exercise which hides it for 10 seconds (and again stop to
-- move). so you can finish something real quick"): the opened pump steps back to waiting for its safe moment, out
-- of sight for snooze_s, then asks again as an offer does: the dim once the character stands still, the "Stop to
-- pump" card while it moves. Its moves, its place and what it owes stay as they were.
function Lc.Snooze()
	local p = Lc.pump
	if not p or p.state ~= "open" or p.view ~= "ready" then return end
	p.snooze = "hidden" -- then "waiting" until it opens again (Lc.Open)
	Lc.stillFor, Lc.movingFor = 0, 0
	setState(p, "ready")
	after("snooze", C.snooze_s, function()
		if Lc.pump ~= p or p.snooze ~= "hidden" then return end
		p.snooze = "waiting"
		startTicker()
	end)
end

-- The lifecycle's ticker runs only while a pump waits for its safe moment (settling, ready, card).
function Lc.Tick()
	local p = Lc.pump
	if not p or not PENDING[p.state] or p.snooze == "hidden" then
		stopTicker()
		return
	end
	local now = GetTime()
	local dt = now - (Lc.lastTick or now)
	Lc.lastTick = now
	-- an NPC window flagged open while none of its frames shows: its close event went missing, in any waiting state
	if next(Lc.npc) and not npcFrameShown() then
		Lc.stuck = (Lc.stuck or 0) + dt
		if Lc.stuck >= C.npc_stuck_s then
			wipe(Lc.npc)
			Lc.stuck = 0
			if p.state == "settling" then Lc.CheckSettle() end
		end
	else
		Lc.stuck = 0
	end
	if p.state == "settling" then return end
	local moving = Lc.Moving()
	if moving == nil then
		Lc.stillFor = 0
	elseif moving then
		Lc.stillFor = 0
		Lc.movingFor = Lc.movingFor + dt
		if p.state == "ready" and Lc.movingFor >= C.card_after_move_s then setState(p, "card") end
	else
		Lc.movingFor = 0
		Lc.stillFor = Lc.stillFor + dt
		if Lc.stillFor >= C.still_s and not Lc.Unsafe(p.asked) then Lc.Open() end
	end
end

-- The moves ------------------------------------------------------------------------------------------------------

function Lc.Slot()
	local p = Lc.pump
	return p and p.moves[p.index]
end

-- The go of a countdown that starts now: the 0.5-s "Ready…" beat, then the five digits.
local function countdownGo()
	return GetTime() + C.lead_ms / 1000 + C.countdown_s
end

-- The countdown from the press: 0.5 s and the five digits. A side move starts on its first side, whatever an
-- interrupted try had done (a fight or a popup takes a move back to its Ready).
function Lc.Start()
	local p = Lc.pump
	if not p or p.state ~= "open" or p.view ~= "ready" or next(Lc.holds) or Lc.Locked() then return false end
	local slot = p.moves[p.index]
	p.started = true
	p.side, p.sides = not slot.practice and ns.Library[slot.move].side and 1 or nil, nil
	p.goAt = countdownGo()
	setState(p, "countdown")
	after("go", p.goAt - GetTime(), function() Lc.Go() end)
	return true
end

-- The move back on its Ready view as if Start was never pressed, nothing written (David, 2026-10-05: "I want to be
-- able during the exercise to go back and have it like I never started the exercise"): the go and the run's clock
-- stop, a side move starts again on its first side, the result view's answer, "Really?" and question go, and the pump
-- counts as started only while a move of it is recorded (a fight then leaves it waiting, a turn-in queues behind it,
-- a new filter re-rolls it). The caller announces it: backToReady below, Lc.Hold its "Hold".
local function toReady(p)
	cancel("go")
	cancel("runEnd")
	p.view, p.state, p.goAt = "ready", "open", nil
	p.runStart, p.paused, p.pausedAt, p.elapsed, p.limit, p.side, p.sides = nil, nil, nil, nil, nil, nil, nil
	p.pending, p.really, p.reallyLoad, p.reallyCheck, p.asking, p.marking = nil, nil, nil, nil, nil, nil
	p.pacing, p.reallyAnswer, p.keep, p.longerAsk, p.pours, p.pour = nil, nil, nil, nil, nil, nil
	p.started = next(p.results) ~= nil
end

-- Back to the Ready view: the Back button, Escape in the countdown.
local function backToReady(p)
	toReady(p)
	setState(p, "open")
end

-- The Back button (David, 2026-10-05: "there needs to be a back button"), in the countdown, while the move runs and on
-- the result view until its count is saved (a hold's while its seconds pour, slice 65); a saved count is corrected with
-- /qp undo.
function Lc.Back()
	local p = Lc.pump
	if not p or not (p.state == "countdown" or p.state == "running" or (p.state == "result" and not p.committed)) then
		return
	end
	ns.Log("I", "pump %d: move %d back to its Ready view from %s", p.id, p.index, p.state)
	backToReady(p)
end

-- Whether the player is in a pump, for the card: one on the screen, or held by a fight after Start (never practice).
function Lc.InPump()
	local p = Lc.pump
	return p ~= nil and p.kind ~= "practice" and (VISIBLE[p.state] == true or p.state == "held")
end

function Lc.Go()
	local p = Lc.pump
	if not p or p.state ~= "countdown" then return end
	local slot = p.moves[p.index]
	p.runStart, p.paused, p.pausedAt, p.elapsed, p.keep = GetTime(), 0, nil, nil, nil
	p.limit = slot.limit or ((slot.kind == K.cardio or slot.kind == K.flow) and slot.seconds) or nil
	if p.limit and p.side then p.limit = p.limit / 2 end -- each side runs half the move's seconds
	if p.limit then after("runEnd", p.limit, function() Lc.Done(true) end) end
	setState(p, "running")
end

function Lc.Elapsed()
	local p = Lc.pump
	if not p or not p.runStart then return 0 end
	return math.max(0, (p.pausedAt or GetTime()) - p.runStart - (p.paused or 0))
end

-- The clock the move figure's loop keeps to in the countdown and while the move runs (Panel.lua showFigure), so its
-- start pose lands on the go and each loop ends where the voice's rep count, on Lc.Elapsed, says the rep's number
-- (Speech.lua): seconds from the go, negative in the countdown and held at 0 for the frame the go's timer fires after
-- goAt, then Lc.Elapsed (a Hold freezes it); nil otherwise.
function Lc.SinceGo()
	local p = Lc.pump
	if not p then return nil end
	if p.state == "countdown" and p.goAt then return math.min(0, GetTime() - p.goAt) end
	if p.state == "running" then return Lc.Elapsed() end
	return nil
end

-- A timed move's result: done with its whole seconds when it ran its time (or the player says it did), else stopped
-- with the clock's.
local function timedResult(p, slot, whole)
	if whole then return { outcome = O.done, value = slot.seconds } end
	return { outcome = O.stopped, value = math.floor(p.elapsed) }
end

-- Whether a pose that ran its time asks "How close to the goal pose?" (its slot's `ask`, Engine.lua: its family's mark
-- below the top or on an orange step no hold has earned, the day's first; never in /qp try).
local function markAsks(p, slot)
	return slot.ask ~= nil and p.kind ~= "practice" and not slot.practice
end

-- The pose's ask on the result view, nothing written until the answer (Lc.Mark).
local function askMark(p)
	p.pending, p.marking = nil, true
	setState(p, "result")
end

-- "Did you keep that pace to the end?" (slice 42, 2026-10-17; three answers since slice 52) after a steady move whose
-- slot asks it (its `pace`, Engine.lua paceSlot: the steady ledger's ask, ns.LedgerAsk) that ran its whole time,
-- never in /qp try: on the result view, nothing written until the answer (Lc.Pace).
local function paceAsks(p, slot)
	return slot.pace == true and p.kind ~= "practice"
end

local function askPace(p)
	p.pending, p.pacing = nil, true
	setState(p, "result")
end

-- Whether Keep going shows on the running move (slice 52): a steady move whose slot offers it (`keep`, Engine.lua
-- keepSlot), in its plan's last keep_going_s, not pressed yet, never in /qp try.
function Lc.KeepShows()
	local p = Lc.pump
	if not p or p.state ~= "running" or p.keep or p.side or p.kind == "practice" then return false end
	local slot = p.moves[p.index]
	return slot.keep ~= nil and p.limit ~= nil and p.limit - Lc.Elapsed() <= C.keep_going_s
end

-- Keep going (slice 52; the button, or Space while it shows): the clock runs on past the plan to the full move (the
-- slot's `keep`, 45 s) and stops there by itself; nothing asks after it (Lc.Done saves its seconds and the answer). Its
-- "State" paints the strip's keep (Bridge.lua), so a phone or a watch that follows the move counts on with it.
function Lc.KeepGoing()
	if not Lc.KeepShows() then return end
	local p = Lc.pump
	local slot = p.moves[p.index]
	p.keep, p.limit = true, slot.keep
	cancel("runEnd")
	if not p.pausedAt then after("runEnd", p.limit - Lc.Elapsed(), function() Lc.Done(true) end) end
	ns.Log("I", "pump %d: %s, keep going past %d s", p.id, ns.Library[slot.move].key, slot.seconds)
	ns.Emit("State", p)
end

-- "Could you have held longer?" (slice 52) after a fresh hold set near its goal (ns.HoldAsk: a hold ladder's step, once
-- a ladder a game day), never in /qp try: on the result view, the typed set kept pending until the answer (Lc.Longer).
local function longerAsks(p, slot, value)
	return p.kind ~= "practice" and not slot.practice and ns.HoldAsk(ns.state, slot, value)
end

-- A hold's whole seconds on its stopwatch (a side hold's weaker side's), or nil when it shows under timed_ask_share of
-- the goal (or of the record without one): a clock started late or stopped early never stands in for the real hold
-- (David, 2026-10-05: "Do not assume that the time is the actual time they took"), and the count step asks it.
function Lc.HeldCount(p, slot)
	local seconds = math.floor(p.elapsed or 0)
	local mark = ns.SlotGoal(slot) or ns.Record(ns.state, slot.move)
	if mark and seconds < C.timed_ask_share * mark then return nil end
	return seconds
end

-- Whether a hold's Done saves its stopwatch's seconds with no count step (slice 65, Rapha, 2026-11-03): a hold of a
-- pump (never /qp try's, which stores nothing) whose stopwatch ran its share (Lc.HeldCount) and min_seconds (a first
-- max set under them, which Lc.Done never banks, asks its count).
local function savesSeconds(p, slot)
	return slot.kind == K.hold and not slot.practice and p.kind ~= "practice" and p.elapsed >= C.min_seconds
		and Lc.HeldCount(p, slot) ~= nil
end

-- Done (the button, Enter, or Escape while a move runs); timeUp when a timed move ran its full time. A side move's
-- first side ends in the switch, a timed move's by its clock or by Done, a hold's and a move of reps' by Done alone
-- (a hold counts up past its goal, which is the least, never the most; David, 2026-10-04; reps count on until the
-- player presses it, 2026-10-13): the countdown again ("Switch sides"), then the other side from zero; after it a
-- timed move counts both sides' seconds, a hold or a move of reps the weaker side's (its stopwatch the shorter one).
function Lc.Done(timeUp)
	local p = Lc.pump
	if not p or p.state ~= "running" then return end
	cancel("runEnd")
	local slot = p.moves[p.index]
	local elapsed = timeUp and p.limit or Lc.Elapsed()
	if p.side == 1 then
		p.side, p.sides, p.runStart = 2, { elapsed }, nil
		p.goAt = countdownGo()
		setState(p, "countdown")
		after("go", p.goAt - GetTime(), function() Lc.Go() end)
		return
	end
	p.elapsed = elapsed
	if p.side == 2 then
		p.sides[2] = elapsed
		if slot.kind == K.cardio or slot.kind == K.flow then
			p.elapsed = p.sides[1] + elapsed
			timeUp = timeUp and p.sides[1] >= p.limit
		else
			p.elapsed = math.min(p.sides[1], elapsed)
		end
	end
	p.view = "result"
	local timed = slot.kind == K.cardio or slot.kind == K.flow
	-- a timed move or a hold ended under min_seconds (a hold's weaker side, a timed move's two sides summed) counts
	-- nothing: banked without a word (Lc.ShortBank), never in /qp try, which stores nothing; a hold's first max set goes
	-- on to its count, whose Back undoes a Done pressed by mistake (ns.FirstMaxSet, savesSeconds)
	if not timeUp and slot.kind ~= K.rep and p.kind ~= "practice" and not slot.practice
		and p.elapsed < C.min_seconds and not (slot.kind == K.hold and ns.FirstMaxSet(ns.state, slot)) then
		return Lc.ShortBank()
	end
	-- past the plan after Keep going (slice 52): the seconds done, saved at once, Yes, easily at the full move and Yes,
	-- just where Done ended it between its plan and 45 s (the steady ledger reads both as answers, DB.lua steadyRow)
	if p.keep and p.elapsed >= (slot.seconds or 0) then
		p.pending = { outcome = O.done, value = math.floor(p.elapsed), answer = timeUp and E.pace.easily or E.pace.just }
		Lc.Commit()
		return
	end
	-- ended before timed_ask_share of its seconds (both sides' on a side move), it asks once whether the whole move was
	-- done, nothing written until the answer (Lc.Whole; David, 2026-10-05: "Sometimes people forget to press start or
	-- just start the exercise without pressing it. Or press way too late etc. Do not assume that the time is the actual
	-- time they took ... just ask if it was fully done"); a counted set too
	if timed and not timeUp and p.elapsed < C.timed_ask_share * (slot.seconds or 0) then
		p.pending, p.asking = nil, true
		setState(p, "result")
		return
	end
	if slot.counted and timeUp then -- a counted set that ran its time asks "All out and clean?" (slice 53: Lc.Counted)
		p.pending = nil
		setState(p, "result")
		return
	end
	if timed and timeUp and markAsks(p, slot) then return askMark(p) end
	if timed and timeUp and paceAsks(p, slot) then return askPace(p) end
	if timed then
		-- later, it is recorded at once with the clock's seconds, as one that ran its time is with its whole seconds
		p.pending = timedResult(p, slot, timeUp)
		Lc.Commit()
		return
	end
	p.pending = nil
	-- a hold that ran its share saves its seconds as if typed (slice 65): "Is that right?" and "Could you have held
	-- longer?" still ask where they do, then the seconds pour (Lc.Enter); a count over one set's most opens the count
	-- step with it, as one typed there does
	if savesSeconds(p, slot) then
		p.pours, p.state = true, "result" -- Lc.Enter reads the result view; each of its ways announces it
		if Lc.Enter(Lc.HeldCount(p, slot)) == "max" then setState(p, "result") end
		return
	end
	setState(p, "result")
end

-- A typed count (the rep box's Enter, the spinner's Record, the "Really?" row's Yes), recorded at once (David,
-- 2026-10-04: "let people type and that's it"; a stopped-because row asked why below the goal until then), with the
-- weight box's `load` on a loaded move (tenths of a kg, nil: none). Returns "max" when the box asks for one set or a
-- weight past load_max_kg, "really" when the "Really?" row asks about a count far above or far below the record or a
-- weight far over the planned one (ns.Validate's answer kept as p.reallyCheck for its words; the same count and weight
-- again keep them, stored as typed and marked confirmed), or nothing once the move is recorded.
function Lc.Enter(value, load)
	local p = Lc.pump
	if not p or p.state ~= "result" or p.committed or p.pour or not value then return end
	local slot = p.moves[p.index]
	if slot.practice then
		p.practiceValue = value
		p.committed = true
		ns.Emit("State", p)
		return
	end
	-- 1 or 2 reps, or a hold under min_seconds, count nothing: banked without a word (Lc.ShortBank), a first max set's
	-- saved as "Could not do one" (ns.FirstMaxSet); 0 stays "Could not do one"
	if ns.ShortCount(slot, value) then
		if ns.FirstMaxSet(ns.state, slot) then return Lc.CouldNot() end
		return Lc.ShortBank()
	end
	local check = ns.Validate(ns.state, slot, value, load)
	if check == "max" then return "max" end
	local confirmed = check ~= nil and p.really == value and p.reallyLoad == load
	if check ~= nil and not confirmed then
		p.really, p.reallyLoad, p.reallyCheck = value, load, check
		ns.Emit("State", p)
		return "really"
	end
	p.really, p.reallyLoad, p.reallyCheck = nil, nil, nil
	local outcome, stored = ns.Judge(ns.state, slot, value, p.elapsed, load)
	p.pending = { outcome = outcome, value = stored, confirmed = confirmed, load = load }
	if longerAsks(p, slot, stored) then
		p.longerAsk = true
		setState(p, "result")
		return
	end
	Lc.Save()
end

-- The answer to "Did you do the whole move?" (Lc.Done): all of it records the move as one that ran its time, its whole
-- seconds (a counted set then asks its count, as after its whole window); else it is stopped with the clock's seconds.
function Lc.Whole(all)
	local p = Lc.pump
	if not p or p.state ~= "result" or not p.asking or p.committed then return end
	local slot = p.moves[p.index]
	ns.Log("I", "pump %d: %s stopped at %d of %d s, the whole move: %s", p.id, ns.Library[slot.move].key,
		math.floor(p.elapsed), math.floor(slot.seconds or 0), all and "yes" or "no")
	p.asking = nil
	if all then p.elapsed = slot.seconds end
	if all and slot.counted then return ns.Emit("State", p) end
	if all and markAsks(p, slot) then return askMark(p) end
	if all and paceAsks(p, slot) then return askPace(p) end
	p.pending = timedResult(p, slot, all)
	Lc.Commit()
end

-- The answer to "How close to the goal pose?" (ns.Enum.mark; David, 2026-10-09: "Maybe 4 options or so on how well the
-- pose could be done between perfect, good, okay, bad?"): the pose recorded as one that ran its time, its whole
-- seconds, the answer in its row (DB.lua markAttempt counts the day at Good or better, Perfect alone below a risky
-- step).
function Lc.Mark(answer)
	local p = Lc.pump
	if not p or p.state ~= "result" or not p.marking or p.committed then return end
	local slot = p.moves[p.index]
	ns.Log("I", "pump %d: %s, how close: %d", p.id, ns.Library[slot.move].key, answer)
	p.marking = nil
	p.pending = timedResult(p, slot, true)
	p.pending.answer = answer
	Lc.Commit()
end

-- The answer to "Did you keep that pace to the end?" (ns.Enum.pace, slice 42, 2026-10-17; three answers since slice
-- 52): the steady move recorded as one that ran its time, its whole seconds, the answer in its row's flag bits
-- (ns.ResultRow) for the steady ledger (DB.lua steadyRow: its estimate, its doubt, a full move).
function Lc.Pace(answer)
	local p = Lc.pump
	if not p or p.state ~= "result" or not p.pacing or p.committed then return end
	local slot = p.moves[p.index]
	ns.Log("I", "pump %d: %s, kept the pace: %d", p.id, ns.Library[slot.move].key, answer)
	p.pacing = nil
	p.pending = timedResult(p, slot, true)
	p.pending.answer = answer
	Lc.Commit()
end

-- The answer to "Could you have held longer?" (ns.Enum.longer, slice 52): the hold recorded as typed, the answer in its
-- row's reps (ns.ResultRow; DB.lua holdLedger reads it, A lot's leap on the next fresh goal, ns.HoldLeap).
function Lc.Longer(answer)
	local p = Lc.pump
	if not p or p.state ~= "result" or not p.longerAsk or not p.pending or p.committed then return end
	ns.Log("I", "pump %d: %s, could have held longer: %d", p.id, ns.Library[p.moves[p.index].move].key, answer)
	p.longerAsk = nil
	p.pending.answer = answer
	Lc.Save()
end

-- A counted set's answer after its 30 s (slice 53, 2026-10-23: "All out and clean?"; slice 42's "Did your form hold to
-- the end?" until then): Yes, to the end (`answer` 0 or nil), Form slipped (E.answer.slipped) or Not all out
-- (E.answer.notAllOut), each with the count the optional box holds (nil or 0: none, "Not counted"). A count far above
-- the best clean count asks "Is that right?" once (ns.Validate's "really"; one under it is a normal all-out window,
-- never asked), and that answer again keeps it, confirmed; over one set it is refused ("max"). Recorded at once:
-- judged with no goal (ns.Judge: any answer but Yes, to the end is done whatever its count and keeps no best), the
-- answer in the row's flag bits (ns.ResultRow: a window saved Yes, to the end counts its game day, DB.lua cardioMove;
-- the XP is the same whatever it is). Returns "max", "really", or nothing once recorded.
function Lc.Counted(value, answer)
	local p = Lc.pump
	if not p or p.state ~= "result" or p.committed or p.asking then return end
	local slot = p.moves[p.index]
	if not slot.counted then return end
	value = (value or 0) > 0 and value or nil
	-- a count of 1 or 2 counts nothing: banked without a word (Lc.ShortBank)
	if value and ns.ShortCount(slot, value) and p.kind ~= "practice" then return Lc.ShortBank() end
	local check = value and ns.Validate(ns.state, slot, value)
	if check == "max" then return "max" end
	local confirmed = check == "really" and p.really == value
	answer = answer ~= 0 and answer or nil
	if check == "really" and not confirmed then
		p.really, p.reallyLoad, p.reallyCheck, p.reallyAnswer = value, nil, check, answer
		ns.Emit("State", p)
		return "really"
	end
	p.really, p.reallyLoad, p.reallyCheck, p.reallyAnswer = nil, nil, nil, nil
	local outcome, stored = ns.Judge(ns.state, slot, value, p.elapsed, nil, nil, answer ~= nil)
	ns.Log("I", "pump %d: %s, %s, count %s", p.id, ns.Library[slot.move].key,
		answer == E.answer.slipped and "form slipped" or answer == E.answer.notAllOut and "not all out" or "clean",
		tostring(value or "none"))
	p.pending = { outcome = outcome, value = stored, confirmed = confirmed or nil, answer = answer }
	Lc.Commit()
end

-- "Change it" on the "Really?" row: the count it asked about goes, nothing written.
function Lc.ClearReally()
	local p = Lc.pump
	if not p or not p.really then return end
	p.really, p.reallyLoad, p.reallyCheck, p.reallyAnswer = nil, nil, nil, nil
	ns.Emit("State", p)
end

-- "Could not do one": a typed 0 is skipped as done without a count, this button stores the 0 at once (ns.CouldNot: a
-- miss against a record; without one a placement of 0, which places the ladder a rung lower; an attempt under half the
-- rung's floor moves it down).
function Lc.CouldNot()
	local p = Lc.pump
	if not p or p.state ~= "result" or p.committed or p.pour then return end
	local slot = p.moves[p.index]
	if slot.practice or (slot.kind ~= K.rep and slot.kind ~= K.hold) then return end
	p.pending = { outcome = ns.CouldNot(ns.state, slot), value = 0 }
	Lc.Commit()
end

-- Done with an empty box: the move counts as done, no record change (a counted set answers its own two rows instead:
-- Lc.Counted).
function Lc.EmptyDone()
	local p = Lc.pump
	if not p or p.state ~= "result" or p.committed or p.pour then return end
	local slot = p.moves[p.index]
	if slot.practice then return Lc.Finish("practice") end
	p.pending = { outcome = O.done, value = slot.kind == K.cardio and slot.seconds or 0 }
	Lc.Commit()
end

-- Writes the move's row (and, for an owed move, the paid row that clears it from the queue), then the rung rows the
-- result calls for and after them an easier pick's step down (ns.EasierDown, which /qp undo never reverses); what is
-- worth a toast goes out as "Progress". A set done with a weight (`load`, tenths of a kg) writes its load row first,
-- with the set row's own time (ns.LoadRow), so the fold has the weight when the set row folds. `answer` is the
-- player's answer after the move (ns.ResultRow: a pose's to its goal pose, a counted set's Form slipped, a steady
-- move's kept pace). Returns what the rows earned on the sheet (the XP and stat gains, the titles), which the Result
-- view's Rewards section fills in, the cardio day the set brought (ns.CardioDay: the fold marks the row, DB.lua
-- cardioMove), read before a climb its rung rows make resets the counts, or nil, the set's row as the fold left it and
-- what it moved (ns.Progression's events: a hold's pour reads its day at the top and its step-up, slice 65).
function Lc.WriteResult(p, i, outcome, value, confirmed, answer, load)
	local slot = p.moves[i]
	p.results[i] = outcome
	if p.kind == "practice" then return {} end
	local earned = {}
	Lc.earning = earned
	local set = ns.ResultRow(slot, i, outcome, value, p.elapsed, confirmed, nil, answer)
	if load then
		set.ts = ns.now() -- one time for both rows
		ns.AddRow(ns.LoadRow(set, load))
	end
	local row = ns.AddRow(set)
	local day = ns.CardioDay(ns.state, row)
	local rows, events = ns.Progression(ns.state, row, ns.FilterContext(ns.profile, ns.state))
	-- /qp undo corrects this count (and its weight) for five minutes, a counted set's count too, and asks the rung rules
	-- again under the guard (Lc.WriteCorrection, ns.CorrectionRows: never a rung or a step another row moved)
	if slot.kind == K.rep or slot.kind == K.hold or ns.HasFlag(row.flags, "counted") then Lc.lastTyped = { row = row } end
	for _, r in ipairs(rows) do ns.AddRow(r) end
	local down, stepped = ns.EasierDown(ns.state, slot, row, rows)
	for _, r in ipairs(down) do ns.AddRow(r) end
	for _, e in ipairs(stepped) do events[#events + 1] = e end
	if #events > 0 then ns.Emit("Progress", events, row) end
	if slot.owed then ns.AddRow(ns.PaidRow(slot.owed)) end
	Lc.earning = nil
	return earned, day, row, events
end

-- The sheet's gains of the rows a result writes, collected while Lc.WriteResult runs.
ns.On("Sheet", function(live)
	if not Lc.earning then return end
	for _, e in ipairs(live) do Lc.earning[#Lc.earning + 1] = e end
end)

-- The XP a result earned in all: its move, the wins, the day's first pump, the streak and the rested pool.
local function earnedXP(earned)
	local xp = 0
	for _, e in ipairs(earned) do
		if e.kind == "xp" then xp = xp + e.amount end
	end
	return xp
end

-- Records the pending result and moves on after record_hold_s, so the recorded set is read: its outcome, its Rewards
-- rows, a loaded move's next weight and a timed move's "Finished: 60 s · +12 XP" (David, 2026-10-01: a countdown
-- "simply disappeared without further feedback"). Every set holds: a set to a goal, a timed move, and a max set, the
-- first one (a new player's first mark, which moved on at once until 2026-10-16) and a placement after a break alike
-- (the engine gives every rep or hold set a goal or a max set: Engine.lua dose); the practice move never records.
function Lc.Commit()
	local p = Lc.pump
	if not p or not p.pending or p.committed then return end
	local pend, i = p.pending, p.index
	local slot = p.moves[i]
	-- the record (a loaded move's weighted one too), the last fresh count and the goal at the weight typed, all before
	-- this set, which its outcome line compares with
	local best = ns.Record(ns.state, slot.move)
	local topLoad, topReps = ns.LoadRecord(ns.state, slot.move)
	local _, previous = ns.Form(ns.state, slot.move)
	local target = ns.TargetAt(ns.state, slot, pend.load)
	local earned, day, row, events = Lc.WriteResult(p, i, pend.outcome, pend.value, pend.confirmed, pend.answer,
		pend.load)
	p.pending, p.committed = nil, true
	p.last = { outcome = pend.outcome, value = pend.value, best = best, previous = previous, load = pend.load,
		answer = pend.answer, day = day, row = row, events = events,
		topLoad = topLoad, topReps = topReps, target = target, xp = earnedXP(earned), earned = earned }
	-- after a set with a weight, the next fresh set's weight and goal on the state the set made (ns.NextLoad)
	if pend.load then
		p.last.nextLoad, p.last.nextTarget = ns.NextLoad(ns.state, slot.move, ns.now(),
			ns.FilterContext(ns.profile, ns.state).gear)
	end
	-- a hold's pour waits for Next once its rows are in (slice 65: no move on by itself)
	if not p.pour then after("advance", C.record_hold_s, function() Lc.Next() end) end
	setState(p, "result")
end

-- Saves the pending result: a hold whose Done saved its seconds (p.pours) pours them first (slice 65, Rapha,
-- 2026-11-03: "the seconds should collect into a bar that shows the hold against its step"), its row written when the
-- pour lands (Lc.Land), so Back while it pours saves nothing; `p.pour` keeps the seconds and the record before them,
-- which the bar's best post and the record's jingle read. With the game's reduced motion (Tree.Moving) there is no
-- pour: it lands at once.
function Lc.Save()
	local p = Lc.pump
	if not p or not p.pending or p.committed or p.pour then return end
	if not p.pours then return Lc.Commit() end
	local slot = p.moves[p.index]
	p.pour = { value = p.pending.value, best = ns.Record(ns.state, slot.move), beat = p.pending.outcome == O.beat }
	ns.Log("I", "pump %d: %s, %d s pour", p.id, ns.Library[slot.move].key, p.pending.value)
	if not ns.Tree.Moving() then return Lc.Commit() end
	setState(p, "result")
end

-- The pour landed (its last grain, a click on the stage, Next or Finish): the hold's row is written (Lc.Commit).
function Lc.Land()
	local p = Lc.pump
	if not p or p.state ~= "result" or not p.pour or p.committed then return end
	Lc.Commit()
end

-- The move now on its Ready view dosed again on the records as the pump's sets before it left them (ns.ReadyGoal,
-- Rapha, 2026-10-28: a second set of a ladder in the pump asks the repeat share, never more than it asked), never a
-- move started from a step's card, whose goal its card named.
local function readyGoal(p)
	local slot = p.moves[p.index]
	if p.chosen or not ns.ReadyGoal({ state = ns.state, profile = ns.profile, now = ns.now() }, slot) then return end
	ns.Log("I", "pump %d: %s asks %d now, a repeat", p.id, ns.Library[slot.move].key, slot.target)
end

-- The next move of the pump, or the pump is done. The next move's Ready view waits for Start or Space, as the first
-- one's does, so the player reads it and can swap it (David, 2026-10-02: "When several excercises are queued it
-- autostarts and there is also no option to switch"), its goal set again on the sets before it (readyGoal).
function Lc.Next()
	local p = Lc.pump
	if p and p.pour then Lc.Land() end -- Next while a hold's seconds pour: they land first (slice 65)
	if not p or p.state ~= "result" or not p.committed then return end
	cancel("advance")
	if p.index >= #p.moves then return Lc.Finish("done") end
	p.index = p.index + 1
	p.view, p.committed, p.last, p.pending, p.elapsed, p.runStart = "ready", nil, nil, nil, nil, nil
	p.pours, p.pour = nil, nil
	p.really, p.reallyLoad, p.reallyCheck = nil, nil, nil -- a "Really?" of the move before is not this one's
	readyGoal(p)
	setState(p, "open")
end

-- Bank it, one click (David, 2026-10-04: "remove the hold and just make it one click"; a 3-s hold and a reason from
-- 2026-09-23): the pump's open moves go owed with no reason; owed moves riding along stay owed. Deferred, never
-- forgiven: only the player's own Skip takes a move off the queue undone (Lc.Skip).
function Lc.Bank()
	local p = Lc.pump
	if not p or p.state ~= "open" or p.kind == "demand" or p.kind == "practice" then return end
	local rest = next(p.results) ~= nil and Lc.OnlyRidersLeft(p) -- "Bank the rest": the banner pays what was done
	-- its own moves ended too short went owed before this (Lc.ShortBank); Bank the rest's row stays at 0: the pump was
	-- done, as one that ends on its short move is
	local banked = rest and 0 or p.shortBanked or 0
	for i = p.index, #p.moves do
		if not p.results[i] then
			oweSlot(p, i, p.moves[i], O.banked)
			if not p.moves[i].owed then
				p.results[i] = O.banked
				banked = banked + 1
			end
		end
	end
	-- the pump's own row, written even when it banks none of its own moves (Bank the rest)
	local first = p.quests[1]
	ns.AddRow({ questID = first and first.questID or 0, tier = maxTier(p), kind = K.pump, reps = banked,
		outcome = O.banked })
	Lc.Finish(rest and "rest" or "banked")
end

-- Whether every move left in the open pump rides along unfinished: once a move of it is done, its way out reads
-- "Bank the rest" (Panel.lua), as in a pump the player asked for, since nothing new would be banked.
function Lc.OnlyRidersLeft(p)
	for i = p.index, #p.moves do
		if not p.results[i] and not p.moves[i].owed then return false end
	end
	return true
end

-- Skip (David, 2026-10-04: "There should be a way to just skip/trash/reset all unfinished moves. Otherwise this
-- becomes a blockade for people. You should be able to just start playing right away without having to catch up";
-- "the skip should be per unfinished move without a confirm. you just need to press 31 times then"): on the Ready view
-- of a move that rides the pump as unfinished, one click takes it off the queue for good. Its paid row carries the
-- skipped flag and no move row comes with it, so it earns nothing (no XP, no record, no stat, no clock moves) and
-- costs nothing. The slot leaves the pump as a re-fit's does and the next move shows; with none left the pump ends,
-- as done when a move of it was performed. Never on a quest's own move: Bank it and Pause stay their way.
function Lc.Skip()
	local p = Lc.pump
	-- a move started from the Talents tab pays an owed row as its own set, never a skip of it (Panel.lua shows no Skip)
	if not p or p.state ~= "open" or p.view ~= "ready" or p.chosen then return end
	local slot = p.moves[p.index]
	if not (slot and slot.owed) or p.results[p.index] then return end
	ns.AddRow(ns.PaidRow(slot.owed, true))
	p.skipped = (p.skipped or 0) + 1
	ns.Log("I", "pump %d: skipped %s, unfinished since %s", p.id, ns.Library[slot.owed.move].key,
		date("%Y-%m-%d %H:%M", slot.owed.ts))
	table.remove(p.moves, p.index)
	if p.index > #p.moves then return Lc.Finish(next(p.results) and "done" or "skipped") end
	readyGoal(p)
	ns.Emit("State", p)
end

-- Skip for an unfinished move the setup holds back (a pain flag, a Body switch, the equipment, an unchecked move, the
-- rung rule): no pump carries it to a Ready view, so the HUD's unfinished line lists it (ns.OwedHeldRows) for the same
-- one click and the same paid row as Lc.Skip, earning nothing (David, 2026-10-04: "Otherwise this becomes a blockade
-- for people").
function Lc.SkipHeld(row)
	if not (ns.state and ns.state.owed[ns.OwedKey(row)]) then return end -- paid or skipped meanwhile
	ns.AddRow(ns.PaidRow(row, true))
	local m = ns.Library[row.move] -- nil for a move this addon lacks: held for good, and Skip is its way out
	ns.Log("I", "skipped %s from the HUD, unfinished since %s", m and m.key or tostring(row.move),
		date("%Y-%m-%d %H:%M", row.ts))
end

-- A move ended too short to count (Lc.Done, Lc.Enter, Lc.Counted: under min_reps reps or min_seconds seconds; Rapha,
-- 2026-11-03: "an excercise is considered as banked if they end it (silent bank it)"): banked without a word or a
-- record, as Bank all banks a move (oweSlot: a rider stays the owed row it is, a move the player asked for with nothing
-- owed owes nothing). It leaves the pump as a Skip's slot does and the next move shows; after the last the pump ends,
-- done when a move of it was recorded, else as Bank all ends one, with its own banked row. The pump's tallies count it
-- (`shortBanked`, `shortRiders`: Lc.Bank, Log.lua), since it left the list.
function Lc.ShortBank()
	local p = Lc.pump
	local i, slot = p.index, p.moves[p.index]
	ns.Log("I", "pump %d: %s ended too short to count, banked", p.id, ns.Library[slot.move].key)
	local tier = maxTier(p)
	oweSlot(p, i, slot, O.banked)
	if slot.owed then
		p.shortRiders = (p.shortRiders or 0) + 1
	else
		p.shortBanked = (p.shortBanked or 0) + 1
	end
	toReady(p)
	p.started = true -- a recompose would bring the banked move back beside its owed row
	table.remove(p.moves, i)
	if i <= #p.moves then
		readyGoal(p)
		return setState(p, "open")
	end
	if next(p.results) then return Lc.Finish("done") end
	if p.kind == "demand" then return Lc.Finish("rest") end
	local first = p.quests[1]
	ns.AddRow({ questID = first and first.questID or 0, tier = tier, kind = K.pump, reps = p.shortBanked or 0,
		outcome = O.banked })
	Lc.Finish("banked")
end

-- The pump's own Pause button (David, 2026-10-04), on a turn-in's pump and on one the player asked for: the pump ends
-- and its own moves count for nothing, as every turn-in while paused does; the moves done stay written and the owed
-- moves riding along stay owed. The turn-ins queued behind it came before the pause and go owed; then this character
-- pauses (Lc.Pause). `unwell` when the player chose it on the "I feel unwell" view: "until 04:00" there also writes
-- the felt-unwell lock, the pump row banked for feeling unwell that the helper writes for an app pump's felt-unwell
-- stop, so every pump in the game and the apps rests until the next game day (DB.lua's fold, the helper's
-- Row::locks); an hour, four hours or until resumed stay plain pauses.
function Lc.PausePump(mode, unwell)
	local p = Lc.pump
	if not p or p.state ~= "open" or p.kind == "practice" then return end
	if mode == ns.PAUSE.group and not Lc.MixedParty() then return end -- the party changed under the open menu
	for _, q in ipairs(Lc.queue) do Lc.OweOpen(q) end
	wipe(Lc.queue)
	if unwell and mode == ns.PAUSE.day then
		local first = p.quests[1]
		ns.AddRow({ questID = first and first.questID or 0, tier = maxTier(p), kind = K.pump, outcome = O.banked,
			flags = E.bank.unwell })
	end
	Lc.Pause(mode)
	Lc.Finish(unwell and "unwell" or "paused")
end

-- The Pause menu's "I feel unwell" (David, 2026-10-04: "I feel unwell should not directly pause anyway. It needs an
-- additional confimration that you really want to pause and how long"): its own view, the pause choices with a way
-- back; nothing is written until a choice (Lc.PausePump).
function Lc.UnwellAsk()
	local p = Lc.pump
	if not p or p.state ~= "open" or p.kind == "practice" then return end
	p.view = "unwellAsk"
	ns.Emit("State", p)
end

-- Back from the "I feel unwell" view to the Ready view.
function Lc.BackToReady()
	local p = Lc.pump
	if not p or p.state ~= "open" then return end
	p.view = "ready"
	ns.Emit("State", p)
end

-- "Bank the rest" in a pump the player opened on demand: one click, those moves were owed already. Also closes a
-- practice move or an on-demand move nobody owed, and a turn-in's pump whose own moves are done (Bank it then).
function Lc.BankRest()
	local p = Lc.pump
	if not p or p.state == "countdown" or p.state == "running" then return end
	if p.kind ~= "demand" and p.kind ~= "practice" then return Lc.Bank() end
	Lc.Finish("rest")
end

-- Every checked joint not flagged yet gets its pain flag (a pain row). Returns whether one was added.
local function flagPain(joints)
	local ids = {}
	for id in pairs(joints or {}) do
		if not ns.state.pain[id] then ids[#ids + 1] = id end
	end
	table.sort(ids)
	for _, id in ipairs(ids) do
		ns.AddRow({ move = id, kind = K.pain, reps = 1 })
	end
	return #ids > 0
end

-- A pump that opened but has not started takes new filters at once (a pain flag, a setting): its moves are composed
-- again.
local function recompose(p)
	if not p or p.started or not p.opened or p.kind == "practice" then return end
	compose(p)
	ns.Emit("State", p)
end

-- A started pump after the equipment, the Body switches, the exclusion list or the pain flags changed (the card's "Your
-- setup"; David, 2026-09-30: "open the settings during an excercise for example to change whats available"): every
-- move not yet begun that the filters now refuse becomes its backup, the same level done another way, else the step
-- jacks; an owed move that no longer fits leaves the pump and waits in the queue, as after a pain stop.
local function refit(p)
	local input = { state = ns.state, profile = ns.profile, now = ns.now() }
	local i = (p.state == "open" and p.view == "ready") and p.index or p.index + 1
	while i <= #p.moves do
		local slot = p.moves[i]
		if ns.Allowed(slot.move, ns.state, ns.profile) then
			i = i + 1
		elseif slot.owed then
			table.remove(p.moves, i)
		else
			p.moves[i] = ns.SwapOptions(input, p.moves, i).backup
				or ns.FloorSlot(ns.state, slot, input.now)
			i = i + 1
		end
	end
	p.filterKey = filterKey()
	if p.index > #p.moves then return Lc.Finish("done") end
	ns.Emit("State", p)
end

-- The filters changed (the Body switches, the equipment, the exclusion list, a pain flag; all safety filters): an
-- opened, unstarted pump re-rolls and a started one re-fits the moves not yet begun.
local function refilter()
	local p = Lc.pump
	if p and p.filterKey and p.filterKey ~= filterKey() then
		if p.started then refit(p) else recompose(p) end
	end
end

-- Pain flags, wherever the player sets them (the Pause menu's "Something hurts", the check-in card, the Body page).
function Lc.FlagPain(joints)
	if flagPain(joints) then refilter() end
end

-- A settings change re-rolls or re-fits the pump, and the stand-up switch schedules its move at once.
ns.On("Settings", function()
	refilter()
	Lc.ScheduleStandup()
end)

-- Escape past an open pump's views (on them the key frame lets it through, so the game menu opens and nothing is
-- banked). `boxEmpty` tells whether the rep box holds a number. "Did you do the whole move?", "How close to the goal
-- pose?", "Did you keep that pace to the end?", "Could you have held longer?" and a counted set's three answers take
-- a click: Escape, which ends a running move, never answers them.
function Lc.Escape(boxEmpty)
	local p = Lc.pump
	if not p or not p.started then return false end
	if p.state == "countdown" then
		backToReady(p)
	elseif p.state == "running" then
		Lc.Done()
	elseif p.state == "result" and p.pour then
		-- Back while a hold's seconds pour (slice 65); once they landed Escape does nothing: Next is Enter
		if not p.committed then Lc.Back() end
	elseif p.state == "result" and not p.asking and not p.marking and not p.pacing and not p.longerAsk then
		if p.committed and p.moves[p.index].practice then
			Lc.Finish("practice")
		elseif p.committed then
			Lc.Next()
		elseif boxEmpty and not p.moves[p.index].counted then -- a counted set's answers take a click (Lc.Counted)
			Lc.EmptyDone()
		end
	end
	return true
end

-- Swap (ns.SwapOptions): on a move's Ready view before its Start.
-- A swapped move offers only the way back to the move it replaced, while the filters still allow that one (David,
-- 2026-10-03: "when switching to alternative it should be possible to switch back again the same way"); back on it,
-- the move's own options return, so the two can be traded until Start. A move started from the Talents tab offers what
-- ns.SwapOptions gives a `chosen` pump (slice 36: no other move but at the cardio rung's counted set or all-out window;
-- tools/apppump.lua Pump:swapOptions alike).
function Lc.SwapOptions()
	local p = Lc.pump
	if not p or p.state ~= "open" or p.view ~= "ready" then return {} end
	local slot = p.moves[p.index]
	if not slot or p.results[p.index] then return {} end
	if slot.swapped then
		local from = slot.swappedFrom
		return { back = from and ns.Allowed(from.move, ns.state, ns.profile) and from or nil }
	end
	return ns.SwapOptions({ state = ns.state, profile = ns.profile, now = ns.now() }, p.moves, p.index, p.chosen ~= nil)
end

-- `option`: the slot the menu showed, so the move played is the one its label named (the options move with the
-- clock). "back" puts the replaced slot back as it was.
function Lc.Swap(which, option)
	local p = Lc.pump
	local chosen = Lc.SwapOptions()[which]
	if not chosen then return false end
	if which == "back" then
		p.moves[p.index] = chosen
	else
		chosen = option or chosen
		chosen.swapped, chosen.swappedFrom = which, p.moves[p.index]
		p.moves[p.index] = chosen
	end
	ns.Emit("State", p)
	return true
end

-- Popups, cinematics and the game menu hide the dim and pause a running move until they close.
function Lc.Hold(reason)
	if Lc.holds[reason] then return end
	Lc.holds[reason] = true
	local p = Lc.pump
	if p then
		if p.state == "countdown" then toReady(p) end
		if p.state == "running" and not p.pausedAt then
			p.pausedAt = GetTime()
			cancel("runEnd")
		end
	end
	ns.Emit("Hold")
end

function Lc.Unhold(reason)
	if not Lc.holds[reason] then return end
	Lc.holds[reason] = nil
	if next(Lc.holds) then return end
	local p = Lc.pump
	if p and p.state == "running" and p.pausedAt then
		p.paused = (p.paused or 0) + GetTime() - p.pausedAt
		p.pausedAt = nil
		if p.limit then after("runEnd", p.limit - Lc.Elapsed(), function() Lc.Done(true) end) end
	end
	ns.Emit("Hold")
end

function Lc.Visible()
	local p = Lc.pump
	return p ~= nil and VISIBLE[p.state] == true and not next(Lc.holds) and not Lc.combat and not InCombatLockdown()
end

-- On demand: the HUD chip, the keybinding and /qp pump open the pending pump at once, or a pump of every owed move,
-- oldest first and spread after the latest set (Engine.lua spreadRiders), or with nothing owed one move that earns XP
-- and counts for the streak.
function Lc.Demand()
	if not ns.db then return end
	if Setup.Blocks() then
		-- the window is what starts the game: shown again when nothing shows it (a pause, a page closed this session)
		if not Setup.ShouldShow() then Setup.Open() end
		ns.Print(Setup.Page() ~= "companions" and "ui.setup.first.yours"
			or Setup.AsksPhone() and "ui.setup.first" or "ui.setup.first.one")
		return
	end
	if Lc.Locked() then return ns.Print("chat.locked") end
	if Lc.combat or InCombatLockdown() then return ns.Print("chat.combat") end
	local p = Lc.pump
	if p and (VISIBLE[p.state] or p.state == "waiting" or p.state == "held") then return end
	if Lc.Unsafe(true) then return ns.Print("chat.notnow") end
	if not p then
		p = newPump("demand")
		Lc.pump = p
		compose(p)
	end
	ask(p)
end

-- Start from a step's card on the Talents tab (slice 37; David, 2026-10-13: "a use should be able to start a move from
-- the tree anytime"). What keeps a Start from running, by the states an event announces, so the card greys its Start
-- with the reason's words (Tree.BlockLine) and draws again when one changes (Tree.lua: PLAYER_REGEN_* for a fight,
-- PLAYER_DEAD, PLAYER_ALIVE and PLAYER_UNGHOST for a death, PLAYER_ENTERING_WORLD for the loading screen into or out
-- of a battleground, the "State" and "Setup" messages, and a timer to the lock's end, Lc.LockEnd): "setup" (the setup
-- is not finished: the card links the setup window, as a pump asked for opens it), "locked" (felt unwell, until the
-- next game day), "combat", "pump" (a pump of the queue's waits: the button opens it, Lc.Demand), "pvp" (a battleground
-- or an arena) and "dead"; nil when a Start may run.
function Lc.StartBlock()
	if not ns.db or Setup.Blocks() then return "setup" end
	if Lc.Locked() then return "locked" end
	if Lc.combat or InCombatLockdown() then return "combat" end
	if Lc.pump then return "pump" end
	local _, kind = IsInInstance()
	if kind == "pvp" or kind == "arena" then return "pvp" end
	if UnitIsDeadOrGhost("player") then return "dead" end
	return nil
end

-- When the felt-unwell lock (Lc.Locked, the game day of the felt-unwell row) ends: the next game day's start
-- (ns.NextDayStart), the moment a greyed Start and the trees' "Rest today" line name (Tree.BlockLine) and Tree.lua's
-- timer waits for.
function Lc.LockEnd()
	return ns.NextDayStart(ns.now())
end

-- What a pump started from a step's card composes as its one slot (`p.chosen`, Engine.lua chosenSlot): ns.StartFor's
-- move, whether it is a counted set or another all-out window, the rung its slot is dosed at, a stretch check's least
-- answer to its goal pose (`ask`, slice 42: its pose asks it at that answer) and a Long Haul's seconds (`haul`, slice
-- 55: ns.HaulStart).
local function chosenOf(s)
	return { move = s.move, allout = s.allout == true, rung = s.rung, ask = s.ask, haul = s.haul }
end

-- The click on a card's Start: `choice` is ns.StartFor's start, `from` where it was started (the step whose card
-- started it, for Tree.Return, and the target Lc.Cleared asks ns.StartFor about again). It checks Lc.StartBlock again
-- and then the states no event reports (swimming, falling, a vehicle, a cinematic, the breath timer, a popup or a
-- window that holds a pump: Lc.Unsafe's for a pump asked for) and returns false and the reason, which the card words
-- in red and in chat (Tree.BlockLine); else opens a pump asked for with that one move, which pays and owes no other
-- unfinished move (David, 2026-10-14: "yes"), on its Ready view (never the countdown), and returns true. It never
-- takes the place of a pump on its way.
function Lc.StartMove(choice, from)
	local why = Lc.StartBlock() or Lc.Unsafe(true)
	if why then return false, why end
	local p = newPump("demand")
	p.chosen = chosenOf(choice)
	p.fromTree = from
	Lc.pump = p
	compose(p)
	ns.Log("I", "pump %d: started %s from the Talents tab (%s)", p.id, ns.Library[choice.move].key, choice.as or "own")
	ask(p)
	return true
end

-- /qp try: 20 s of wall push-ups and the rep box, at any time (before setup too); nothing is stored.
function Lc.Try()
	if not ns.db then return end
	if Lc.Locked() then return ns.Print("chat.locked") end
	if Lc.combat or InCombatLockdown() then return ns.Print("chat.combat") end
	if Lc.pump then return ns.Print("chat.busy") end
	if Lc.Unsafe(true) then return ns.Print("chat.notnow") end
	local p = newPump("practice")
	Lc.pump = p
	compose(p)
	ask(p)
end

-- The stand-up move (General, off by default): after 30 min out of combat without a pump, outside instances.
function Lc.ScheduleStandup()
	cancel("standup")
	if not (ns.profile and ns.profile.standup) or Lc.combat then return end
	local since = math.max(Lc.lastPumpAt or Lc.loginAt or ns.now(), Lc.combatEndAt or 0)
	after("standup", since + C.standup_after_min * 60 - ns.now(), function() Lc.Standup() end)
end

function Lc.Standup()
	if not (ns.profile and ns.profile.standup) or Lc.combat or asleep() then return end
	if Lc.pump or Setup.Blocks() or Lc.Locked() or IsInInstance() or Lc.OwingContext() then
		after("standup", C.standup_retry_s, function() Lc.Standup() end)
		return
	end
	begin(newPump("standup"))
end

-- The death tax (General, off by default; slice 23): the "push-ups every death" format of fitness streams. A death owes
-- one move, the record attempt on the ladder that has waited longest that a pump asked for with nothing owed opens on
-- (ns.Compose's demand kind, no riders; a move the queue holds is its ladder's turn, so the next death takes the next
-- ladder), written as a turn-in's unfinished move is: it rides the next pump like every one, Skip takes it off, and it
-- never opens a pump. Only where a turn-in counts: the game on, this character not paused, setup done (in the
-- felt-unwell lock, an instance or a mixed party a turn-in is owed, and so is a death); never during /qp try. One
-- death owes one move: PLAYER_DEAD fires once a death and never for a feign death, and Lc.dead holds from it until the
-- character lives again (a spirit release fires PLAYER_ALIVE as a ghost; a /reload while dead sends no PLAYER_DEAD,
-- Blizzard's own death popup comes back at PLAYER_ENTERING_WORLD), so no repeated event owes a second move.
local function deathTax()
	if UnitIsFeignDeath("player") or Lc.dead then return end
	Lc.dead = true
	if not (ns.db and ns.profile.deathTax) or Setup.Blocks() or asleep() then return end
	if Lc.pump and Lc.pump.kind == "practice" then return end
	local p = newPump("death")
	compose(p, true, "demand")
	local slot = p.moves[1]
	if not slot then return end
	oweSlot(p, 1, slot, nil, E.owedBy.death)
	ns.Log("I", "death tax: %s unfinished", ns.Library[slot.move].key)
	ns.Emit("Owed", p)
	-- a pump on its way carries it (Lc.Open composes again): its chip counts it now
	local q = Lc.pump
	if q and not q.opened and MERGES[q.kind] then
		compose(q)
		ns.Emit("State", q)
	end
end

-- Alive again (a resurrection, the spirit healer, back at the corpse; at every loading screen too): the next death
-- owes its move.
local function lives()
	if not UnitIsDeadOrGhost("player") then Lc.dead = false end
end

-- Setup ----------------------------------------------------------------------------------------------------------

-- Whether the setup asks for the phone app. A phone pairs only through the helper's code and no store offers the apps
-- yet, so not with no pairing block (no helper ever ran here) and not while the helper's signed feed says the apps are
-- not in the stores (the block's phone_apps false): a helper that reads the game is the whole setup then. A paired
-- phone keeps its card (a tester's); a block with no word (an older helper, a feed it could not read) or the feed's
-- yes asks for it.
function Setup.AsksPhone()
	local p = ns.Pairing()
	return ns.IsPaired() or (p ~= nil and p.phone_apps ~= false)
end

-- Each companion's step as this load found it: the helper reads the game, and a phone is paired and has reached the
-- helper lately (ns.PhoneQuiet: one quiet for weeks is asked for again, as a missing one is), or none is asked for.
function Setup.Steps()
	return ns.HelperState() == "on", not Setup.AsksPhone() or ns.IsPaired() and not ns.PhoneQuiet()
end

-- The helper runs at this load, whether or not it reads the game: its copy keeps the progress.
local RUNS = { on = true, blind = true, stopped = true }

function Setup.HelperRuns()
	return RUNS[ns.HelperState()] == true
end

-- A client that showed it does not read its own save back (AGENTS.md's beta bugs): a /reload found nothing loaded
-- (ns.svBroken, DB.lua), so its progress outlasts a load only through the helper's copy. A login that loads nothing
-- proves nothing, since a first install does that too. On Forever alone: retail reads its save, and a first load
-- through a UI reload reads the same as the bug (DB.lua).
function Setup.SaveUnread()
	return ns.IS_FOREVER and ns.svBroken == true
end

-- The setup window's first page, the companions': until both steps are done (a paired player whose helper is not
-- running, or cannot see the game, is asked again: David, 2026-09-30, nothing told him to start it), as often as
-- Setup.Enter lets it come; /qp setup and the General page's Companions row show it at any time (Setup.Open).
local function companionsDue()
	if Setup.dismissed then return false end
	local helper, phone = Setup.Steps()
	return Setup.asked or not (helper and phone) and not paused()
end

-- Your setup, the window's pages after the companions' (David, 2026-10-09: "We need a proper onboarding also for the
-- WoW addon. To check your equipment to say wether you have floor and room to move etc"): where the player trains
-- (Floor moves, Room to move, Quiet mode) with the Body switches, the equipment, then how hard (the strength training
-- so far, Setup.AsksExperience, the moves per quest, and the voice's "Count reps aloud", 2026-10-09). Each answer is
-- written at once through ns.Settings' writers, so the strip carries it to the helper's copy. Asked once per profile
-- (setupAnswered), after the companions' page and before the start, which waits for it (BeginTrial's start rows read
-- the setup and the experience); /qp setup and the Companions row ask it again. A save of a newer addon writes
-- nothing, so only those two show it there.
Setup.YOURS = { "place", "gear", "hard" }

function Setup.Answered()
	return ns.readOnly or (ns.profile ~= nil and ns.profile.setupAnswered == true)
end

-- The window's pages now, in order: the companions', then Your setup's.
function Setup.Pages()
	local pages = {}
	if companionsDue() then pages[1] = "companions" end
	if Setup.asked or not Setup.Answered() and not paused() then
		for _, page in ipairs(Setup.YOURS) do pages[#pages + 1] = page end
	end
	return pages
end

-- The page the window shows (the one Back or Next moved to while it is still among them, else the first), its place
-- among them and their count; nil while there is none.
function Setup.Page()
	if ns.db == nil then return nil end
	local pages = Setup.Pages()
	for i, page in ipairs(pages) do
		if page == Setup.page then return page, i, #pages end
	end
	return pages[1], 1, #pages
end

-- The footer's step ("Step 2 of 4"), for the page Setup.Page gives: the companions' page counts while it is among the
-- pages and, once it showed in this load (Setup.seen), after Play without them closed it too, so the count never
-- starts again on Your setup's first page (2026-10-15). Nil while there is no page.
function Setup.Step()
	local page, at, of = Setup.Page()
	if not page then return nil end
	if page ~= "companions" and Setup.seen and not companionsDue() then return at + 1, of + 1 end
	return at, of
end

function Setup.ShouldShow()
	return Setup.Page() ~= nil
end

-- Whether the window asks for its Check again (the strip's setup word, Bridge.lua; 2026-10-13): its page is the
-- companions' with a step still open, on screen or held back by a fight, a pump or a cinematic (Panel.RenderSetup),
-- since the button comes back with the window once that ends, so the phones go on naming it meanwhile. False before
-- this load's PLAYER_ENTERING_WORLD decides whether the window comes in this session (Setup.Enter), and once Play
-- without them or the window's end closes the page. Setup.Page's table is built only while that page is due with a
-- step open, never at a paint for a player set up or playing without them.
function Setup.CheckAgainDue()
	if not (Lc.entered and companionsDue()) then return false end
	local helper, phone = Setup.Steps()
	return not (helper and phone) and Setup.Page() == "companions"
end

-- Whether How hard asks the strength training so far: only while no ladder holds a rung or a placement, as the phones
-- ask it; the start rows (the game's, or the app's the helper brought) and every first max set place them.
function Setup.AsksExperience()
	if not ns.state then return false end
	if ns.state.cardio and ns.state.cardio.rung then return false end
	for _, lad in pairs(ns.state.ladders) do
		if lad.rung or lad.placedTs then return false end
	end
	return true
end

-- Whether the save holds a start row of the game's (this install's or another's). The app's rows place ladders too
-- (slice 26), and a start row on a ladder that has a rung changes nothing in the fold, so until the game has one every
-- start row is written: the game's start is moved only by the game's rows.
local function gameStarted()
	for _, str in ipairs(ns.db.log or {}) do
		local r = ns.DecodeRow(str)
		if r and r.kind == K.rung and r.reps == ns.RungCause.start and ns.Source(r.flags) == E.source.game then
			return true
		end
	end
	return false
end

-- The start, once the setup window is finished (or a paired player's Your setup was answered before this load), and
-- again after /qp reset all: every turn-in is a pump from here, and no question about pain comes before the first
-- (David, 2026-10-04; lostNote). Your setup came first (David, 2026-10-09; until then no form came, 2026-09-30: "just
-- start the addon right away"), so the start rung rows read the player's experience and setup: one for each ladder
-- not placed yet, every ladder until the game has a start row (gameStarted), so a start the rows remember (the beta's
-- lost meta, the helper's mirror) places no ladder twice.
function Setup.BeginTrial()
	Lc.trialThisSession = true
	ns.SetMeta("trialAt", ns.now())
	local wrote = false
	if not ns.readOnly then
		local p = ns.profile
		local f = ns.FilterContext(p, ns.state)
		local all = not gameStarted()
		for _, r in ipairs(ns.StartRows(p.experience, f, ns.now())) do
			local lad = r.ladder == ns.CARDIO and ns.state.cardio or ns.state.ladders[r.ladder]
			if all or not (lad and (lad.rung or lad.placedTs)) then
				ns.AddRow(r)
				wrote = true
			end
		end
	end
	ns.Log("I", "start: %s", wrote and "the start rung rows written" or "the start rows were there already")
	ns.Print("chat.trial")
	ns.Emit("Trial")
end

-- The one-tap check-in (day two after creation, two days after a 14-day placement): 1 fine, 2 sore, 3 very sore or
-- dark urine; the card offers the pain flags. A counted checkin row, local only.
function Lc.CheckinDue()
	return ns.db ~= nil and not ns.readOnly and Lc.TrialBegun() and ns.CheckinDue(ns.state, ns.now())
end

function Lc.CheckinAnswer(answer)
	if not ns.db or ns.readOnly then return end
	ns.AddRow({ kind = K.checkin, reps = answer })
end

-- A phone that paired since the save last told of one (the pairing the reload behind Check again reads) gets one chat
-- line, before the start's own. The save remembers the pairing it told of (meta.phoneSince); a client that never
-- reads its save back (the beta, ns.loadedStamp nil) forgets its meta at every load, so there a row newer than the
-- pairing (the start's, the first pump's, which the helper's copy brings back) ends it too. A client that reads its
-- save keeps meta.phoneSince, which alone decides: a pump made before the reload never hides a new pairing.
local function announcePhone()
	local pairing = ns.Pairing()
	local since = ns.IsPaired() and ns.meta and tonumber(pairing.phone_since)
	if not since or ns.meta.phoneSince == since then return end
	if ns.loadedStamp == nil then
		local log = ns.db.log or {}
		for i = #log, 1, -1 do
			local r = ns.DecodeRow(log[i])
			if r and r.ts >= since and ns.Source(r.flags) ~= E.source.app then return end -- the app's rows are no start
		end
	end
	ns.SetMeta("phoneSince", since)
	ns.Print("status.paired")
end

-- At every login (so every /reload): a paired phone, or a setup that asks for none, starts the game once Your setup is
-- answered (Setup.Blocks); until then, and without a phone, the setup window shows.
function Setup.Evaluate()
	announcePhone()
	if not Lc.TrialBegun() and not Setup.Blocks() then Setup.BeginTrial() end
	ns.Emit("Setup")
end

local function dayNumber(ts)
	return ns.DayIndex(ns.day(ts))
end

-- Whether the screen comes in this session (David, 2026-10-09: "slowly it should push you"): always before the game
-- starts; then at every login for C.setup_first_days game days after it first showed (meta.setupFirstAt), then at most
-- once in C.setup_every_days game days (meta.setupShownAt), and on every reload of a session it showed in (Check again)
-- until it is closed. A client that never reads its save back (the beta) forgets both stamps at every load, so there
-- it comes at every login, as it did before the cadence.
local function setupDue()
	if Setup.Blocks() then return true end
	local m = ns.meta or {}
	local first, shown = tonumber(m.setupFirstAt), tonumber(m.setupShownAt)
	if not (first and shown) then return true end
	if m.loginAt and shown >= m.loginAt then return true end
	local today = dayNumber()
	return today - dayNumber(first) < C.setup_first_days or today - dayNumber(shown) >= C.setup_every_days
end

-- The screen's first appearance after a login or a reload, decided at PLAYER_ENTERING_WORLD, which knows which it is.
function Setup.Enter(isReloadingUi)
	if not isReloadingUi then ns.SetMeta("loginAt", ns.now()) end -- a new login: a new session
	if isReloadingUi and ns.meta and ns.meta.setupClosedAt and ns.meta.loginAt
		and ns.meta.setupClosedAt >= ns.meta.loginAt then
		Setup.dismissed = true
	end
	if not setupDue() then Setup.dismissed = true end
	Setup.Evaluate()
	if Lc.CheckinDue() then ns.Emit("CheckinDue") end
end

-- The screen showed (Panel.RenderSetup, as it opens): the stamps its cadence counts from, and this load's mark.
function Setup.Shown()
	Setup.seen = true
	if not (ns.meta and ns.meta.setupFirstAt) then ns.SetMeta("setupFirstAt", ns.now()) end
	ns.SetMeta("setupShownAt", ns.now())
end

-- "Play without them" on the companions' page: the page closes for the session (it comes again as Setup.Enter's
-- cadence lets it), and Your setup comes next while it is asked; with nothing after it the window is finished.
function Setup.PlayWithout()
	ns.Log("I", "setup: play without the companions")
	Setup.dismissed = true
	Setup.page = nil
	Setup.Remember()
	if Setup.ShouldShow() then return ns.Emit("Setup") end
	Setup.Close()
end

-- Back and Next between the window's pages (`step` -1 or 1); Next on the last page finishes the window, and Back on
-- Your setup's first page after Play without them, which Setup.Step counts behind it, opens the companions' page again
-- (the D-Setup2 board's Back on step 2), forgetting that this session played without them (Setup.Reopen).
function Setup.Go(step)
	local page, i, n = Setup.Page()
	if not page then return end
	if step > 0 and i >= n then return Setup.Close() end
	if step < 0 and i == 1 and Setup.Step() > 1 then
		Setup.page = "companions"
		return Setup.Reopen()
	end
	Setup.page = Setup.Pages()[math.max(1, i + step)]
	ns.Emit("Setup")
end

-- Escape and the close button (David, 2026-10-03: closing it "should be the same as play without them"): on the
-- companions' page while Your setup is unanswered they are Play without them, which moves on to it, so no reflex skips
-- the questions; anywhere else they finish the window (Setup.Close). Two presses close it all: it never traps.
function Setup.Dismiss()
	if Setup.Page() == "companions" and not Setup.Answered() then return Setup.PlayWithout() end
	Setup.Close()
end

-- The window finished: the last page's Start (Done once the game has started), and Setup.Dismiss past the
-- companions' page. What the player set stays and the rest keeps its defaults: Your setup counts as answered, the
-- companions' page closes for the session (again as the cadence lets it), and the game starts unless it has.
function Setup.Close()
	ns.Log("I", "setup: finished on the %s page", Setup.Page() or "no")
	Setup.dismissed = true
	Setup.asked = nil
	Setup.page = nil
	Setup.Remember()
	if not Setup.Answered() then ns.Settings.Set("setupAnswered", true) end
	if not Lc.TrialBegun() then Setup.BeginTrial() end
	ns.Emit("Setup")
end

-- The screen closes for the session, and a /reload is the same session: the choice is kept in the save and honoured
-- only on a reload (a new login shows the screen again; on the beta, which reads nothing back, every load is new).
function Setup.Remember()
	ns.SetMeta("setupClosedAt", ns.now())
end

-- Reopened (/qp pump before setup): the close is forgotten too, so the reload Check again asks for shows it again.
function Setup.Reopen()
	Setup.dismissed = false
	ns.SetMeta("setupClosedAt", nil)
	ns.Emit("Setup")
end

-- /qp setup and the General page's Companions row (David, 2026-10-09): the window at any time, from its first page,
-- the companions' (both steps done or not) and Your setup's, until it is finished; at Your setup's `page` (one of
-- Setup.YOURS) where one is named: a step's card on the Talents tab opens it at the rule that shuts the step (slice
-- 37).
function Setup.Open(page)
	if not ns.db then return end
	Setup.asked = true
	Setup.page = nil
	for _, key in ipairs(Setup.YOURS) do
		if key == page then Setup.page = page end
	end
	Setup.Reopen()
end

-- /qp reset all (Settings.lua): every row and the profiles went, so the start is a new one: the window asks from its
-- first page again (whatever the companions' cadence said), and the start waits for it to be finished.
function Setup.Restart()
	Lc.trialThisSession = false
	Setup.dismissed = false
	Setup.asked = nil
	Setup.page = nil
	ns.Emit("Trial")
	ns.Emit("Setup")
end
ns.AddCommand("setup", function() Setup.Open() end, "cmd.setup.help") -- its first page: the argument names no page

-- A nudge toward the phone app at a moment it helps (David, 2026-10-09): one chat line after a pump the player
-- finished or banked, the unfinished moves' when it left some, else the streak's at C.nudge_streak_days or more; at
-- most one in C.nudge_every_days game days (meta.nudgeAt), never while a fight or another pump is on, never in a
-- session the setup screen showed in (it said it all), none once the phone's step is done, and none with chat
-- messages off. The beta forgets meta.nudgeAt at every load, so there the cap holds for the session.
local NUDGE_AFTER = { done = true, rest = true, banked = true }

function Setup.Nudge(how)
	if not (ns.db and NUDGE_AFTER[how]) or Lc.pump or Lc.QueueBegins() or Lc.combat or InCombatLockdown() then return end
	if select(2, Setup.Steps()) then return end
	local m = ns.meta
	if not m or Setup.seen or (m.setupShownAt and m.loginAt and m.setupShownAt >= m.loginAt) then return end
	if m.nudgeAt and dayNumber() - dayNumber(m.nudgeAt) < C.nudge_every_days then return end
	local key
	if how ~= "done" and ns.OwedCount(ns.state) > 0 then
		key = "status.nudge.owed"
	elseif ns.Streak(ns.state, ns.now()) >= C.nudge_streak_days then
		key = "status.nudge.streak"
	end
	if not key then return end
	ns.SetMeta("nudgeAt", ns.now())
	ns.Chat("nudge", L[key])
end
ns.On("State", function(p, how)
	if p and p.state == "done" then Setup.Nudge(how) end
end)

-- A newer addon (2026-10-31, the release's addon feed): the helper reads its signed addon.json and names the newest
-- version in the pairing block, `addon_latest`, with `addon_by` "helper" when it keeps this install up to date (it puts
-- the new one in when the game closes) and "other" when the player updates it some other way. Versions compare as
-- x.y.z with a leading "v" dropped; one that does not parse (a checkout's "dev", a build between tags) is never older
-- or newer, so it brings no line.
local function versionParts(v)
	if type(v) ~= "string" then return nil end
	local major, minor, patch = v:match("^v?(%d+)%.(%d+)%.(%d+)$")
	return major and { tonumber(major), tonumber(minor), tonumber(patch) } or nil
end

-- The newer version as x.y.z and the line that tells it, or nil while this addon is as new as the feed's.
function Lc.Update()
	local p = ns.Pairing()
	local latest, ours = versionParts(p and p.addon_latest), versionParts(ns.version)
	if not (latest and ours) then return nil end
	for i = 1, 3 do
		if latest[i] ~= ours[i] then
			if latest[i] < ours[i] then return nil end
			return table.concat(latest, "."), p.addon_by == "helper" and "status.update.ready" or "status.update.out"
		end
	end
	return nil
end

-- One chat line a session (a login and its reloads: told at or after meta.loginAt, which Setup.Enter stamped just
-- before; the beta forgets its meta at every load, so there it comes at every load), with its log line; /qp status
-- says it at any time. No address and no store: the helper or the player's own way gets it.
function Lc.TellUpdate()
	local version, key = Lc.Update()
	if not version then return end
	local m = ns.meta
	if m and m.updateToldAt and m.loginAt and m.updateToldAt >= m.loginAt then return end
	ns.SetMeta("updateToldAt", ns.now())
	ns.Log("I", "update: QuestPump %s in the helper's feed, this addon %s, %s", version, ns.version,
		key == "status.update.ready" and "the helper installs it when the game closes" or "installed some other way")
	ns.Chat("update", L[key]:format(version))
end

-- Events ---------------------------------------------------------------------------------------------------------

local function npcOpen(source)
	Lc.npc[source] = true
	cancel("settle")
	if Lc.pump and Lc.pump.state == "settling" then ns.Emit("State", Lc.pump) end
end

local function npcClose(source)
	Lc.npc[source] = nil
	Lc.CheckSettle()
end

local readable = ns.Readable

-- Remembers a quest while it is in the log, with what its score and family need and the quest table's mapping (quest
-- data is never secret; the player's level and XP are read the same guarded way); false for a hidden quest, which
-- never counts.
local function capture(questID)
	local index = C_QuestLog.GetLogIndexForQuestID(questID)
	local log = index and C_QuestLog.GetInfo(index)
	if log and log.isHidden then return false end
	local info = Lc.info[questID] or { types = {} }
	if log then
		info.types = {}
		local n, required, kills = 0, 0, 0
		for _, o in ipairs(C_QuestLog.GetQuestObjectives(questID) or {}) do
			n = n + 1
			required = required + (o.numRequired or 0)
			if o.type then info.types[o.type] = true end
			if o.type == "monster" and o.numRequired == 1 then kills = kills + 1 end
		end
		info.objectives, info.required, info.singleKill = n, required, n == 1 and kills == 1 or nil
	end
	local tag = C_QuestLog.GetQuestTagInfo(questID) -- by id, so a turn-in-only quest has its tag too
	info.tagID = tag and tag.tagID or info.tagID
	info.difficulty = C_PlayerInfo.GetContentDifficultyQuestForPlayer(questID) or info.difficulty
	local qLevel, level = C_QuestLog.GetQuestDifficultyLevel(questID), readable(UnitEffectiveLevel("player"))
	info.level = qLevel or info.level
	if qLevel and level then info.levelDiff = qLevel - level end
	info.trivialRange = readable(UnitQuestTrivialLevelRange("player")) or info.trivialRange
	info.trivial = C_QuestLog.IsQuestTrivial(questID) or nil
	info.xpMax = readable(UnitXPMax("player")) or info.xpMax
	info.repeatable = info.repeatable or C_QuestLog.IsRepeatableQuest(questID) or nil
	info.mapped = ns.Mapping(questID)
	Lc.info[questID] = info
	return true
end

-- Why a game climb (`r`, its ladder's latest rung row the fold applied, ns.LatestRung) goes back, nil where it stays:
-- "placement" for one the rules before 2026-10-10 made at once, by a placement at the window's top or a plateau, onto
-- a step today's rules open only by the days at its top (ns.DaysOnly: an orange or gold step, or one whose anchor is a
-- skill or a tendon-heavy move), on a step the player has never tried (ns.StepTried); "held" for one onto a step whose
-- moves of its own level wait for another step with any setup (ns.HeldClimb, 2026-10-24: power step 7's clap push-up
-- for 12 feet-up push-ups).
local function oldClimb(r, rung)
	if not (r.char > 0 and ns.Source(r.flags) == E.source.game and r.rung == rung and rung > 1) then return nil end
	if (r.reps == ns.RungCause.placementHigh or r.reps == ns.RungCause.plateau) and ns.DaysOnly(r.ladder, rung - 1)
		and not ns.StepTried(ns.state, r.ladder, rung) then
		return "placement"
	end
	return ns.HeldClimb(ns.state, r) and "held" or nil
end

local OLD_CLIMB = { placement = "a placement or a plateau onto a step its days at the top open",
	held = "onto a step whose moves wait for another step" }

-- The climbs today's rules would not make (oldClimb), on a step the player has never tried: each goes back as /qp undo
-- takes a rung change back, an undo row under the climb's own character naming the climb (its ts in questID,
-- 2026-10-19) that the fold, the desktop app and both phones pair with it (DB.lua undone), so the ladder stands on the
-- step below with its counts toward the climb kept (David, 2026-10-13, after 15 Bulgarian split squats had placed him
-- on the assisted pistol: "undo old climbs onto skill moves you've never done"). An app's climb is the desktop app's
-- to take back (its sweep). Today's rules write no such climb (one turns held only when a step it needed starts over
-- before the player tries the new one), so the login that finds one is the last; a stack of them goes one step a pass.
function Lc.UndoOldClimbs()
	if not ns.db or ns.readOnly or not ns.state then return end
	local moved = {}
	for _, l in ipairs(ns.STRENGTH_ORDER) do
		local rung = ns.Rung(ns.state, l)
		local r = ns.LatestRung(ns.state, ns.db.log, l)
		local why = r and oldClimb(r, rung)
		while why do
			ns.AddRow({ char = r.char, questID = r.ts, kind = K.rung, ladder = l, rung = rung - 1,
				reps = ns.RungCause.undo })
			ns.Fold(ns.db)
			ns.Log("I", "repair: the %s ladder's climb to rung %d of %s undone (never done, %s)", ns.Ladders[l].key, rung,
				date("%Y-%m-%d %H:%M:%S", r.ts), OLD_CLIMB[why])
			if ns.Rung(ns.state, l) ~= rung - 1 then break end
			moved[#moved + 1] = l
			rung = rung - 1
			r = ns.LatestRung(ns.state, ns.db.log, l)
			why = r and oldClimb(r, rung)
		end
	end
	if #moved == 0 then return end
	-- one line a ladder, printed as a command's answer is: the session's one warning (ns.Chat) stays for the pumps
	local said = {}
	for _, l in ipairs(moved) do
		if not said[l] then
			said[l] = true
			ns.Print("chat.oldClimb", L["ladder." .. l], ns.Step(l, ns.Rung(ns.state, l)))
		end
	end
	ns.Emit("DB")
end

-- The steps mastered that no live set's rows mastered (2026-10-17, slice 42's review: a step whose last top came in a
-- phone's, the watch's or the computer's catch-up, in a save from before slice 42, or whose last untopped move the
-- player left out or the setup shut showed "7 of 7 moves topped" with no seal): their rows (ns.MasterySweep) at login,
-- after the mirror's merge, and after every change of the setup or the player's own list and an import ("Settings").
-- No toast says them (only a live set's rows go out as "Progress"); the seal and the Deeds count read the fold.
function Lc.SweepMastered()
	if not (ns.db and ns.state and ns.profile) or ns.readOnly then return end
	for _, r in ipairs(ns.MasterySweep(ns.state, ns.FilterContext(ns.profile, ns.state), ns.now())) do ns.AddRow(r) end
end
ns.On("Settings", Lc.SweepMastered)

ns.RegisterEvent("PLAYER_LOGIN", function()
	-- a /reload or a relog in the middle of a fight: no PLAYER_REGEN_DISABLED comes, the fight is already on
	Lc.combat = (InCombatLockdown() or UnitAffectingCombat("player")) and true or false
	Lc.dead = UnitIsDeadOrGhost("player") and true or false -- a death before this load owed its move then
	Lc.loginAt = ns.now()
	Lc.mixed = Lc.MixedParty()
	Lc.ScheduleStandup()
	Lc.LoadLines()
	Lc.UndoOldClimbs()
	Lc.SweepMastered()
end)

ns.RegisterEvent("PLAYER_ENTERING_WORLD", function(isInitialLogin, isReloadingUi)
	if not Lc.entered then
		Lc.entered = true
		Setup.Enter(isReloadingUi)
		Lc.TellUpdate()
	end
	wipe(Lc.npc)
	lives()
	if isInitialLogin or isReloadingUi then
		-- a pause while grouped that the save or the mirror brought back: still this group's answer (no popup again), or
		-- ended at a login outside a mixed party
		if not Lc.MixedParty() then
			endGroupPause()
		elseif paused() == ns.PAUSE.group then
			party.asked, party.pause = true, true
		end
	end
	Lc.OweIfOwing()
	ns.Emit("Owed") -- an instance entered or left: the HUD's unfinished line says where its moves go
end)

-- No frame renders after PLAYER_LOGOUT, so these rows never reach the strip: the helper writes the same owed rows for
-- the opened pump from its pump_offered frames (rust-helper protocol-and-mirror.md, "The logout rule"). Both sides date
-- them at the pump's offered time, the one time the strip carried, so the merge keeps one copy of each on retail.
ns.RegisterEvent("PLAYER_LOGOUT", function()
	if not ns.db then return end
	Lc.PartyAnswer(nil) -- the turn-ins a mixed party's question still holds count: owed, as they are there
	local p = Lc.pump
	if p and p.kind ~= "practice" then Lc.OweOpen(p, p.offeredTs) end
	for _, q in ipairs(Lc.queue) do Lc.OweOpen(q) end
end)

ns.RegisterEvent("QUEST_ACCEPTED", function(questID)
	if capture(questID) then Lc.seen[questID] = true end
end)

ns.RegisterEvent("QUEST_DETAIL", function() npcOpen("quest") end)
ns.RegisterEvent("QUEST_PROGRESS", function() npcOpen("quest") end)
ns.RegisterEvent("QUEST_GREETING", function() npcOpen("quest") end)
-- A quest-complete window counts the quest even when it never entered the log (a turn-in-only quest); hidden
-- tracking quests never show one.
ns.RegisterEvent("QUEST_COMPLETE", function()
	npcOpen("quest")
	local questID = GetQuestID()
	if questID and questID > 0 and capture(questID) then
		if QuestIsDaily() or QuestIsWeekly() then Lc.info[questID].repeatable = true end
		Lc.seen[questID] = true
	end
end)
ns.RegisterEvent("QUEST_FINISHED", function() npcClose("quest") end)
ns.RegisterEvent("QUEST_TURNED_IN", function(questID, xp, money) Lc.TurnIn(questID, xp, money) end)
-- A character level-up starts no pump (David, 2026-09-30: "just annoys on new characters and is rare on aged ones"):
-- every pump pays a quest.

for open, close in pairs({ GOSSIP_SHOW = "GOSSIP_CLOSED", MERCHANT_SHOW = "MERCHANT_CLOSED",
	TAXIMAP_OPENED = "TAXIMAP_CLOSED", TRAINER_SHOW = "TRAINER_CLOSED", BANKFRAME_OPENED = "BANKFRAME_CLOSED",
	MAIL_SHOW = "MAIL_CLOSED" }) do
	ns.RegisterEvent(open, function() npcOpen(open) end)
	ns.RegisterEvent(close, function() npcClose(open) end)
end
ns.RegisterEvent("PLAYER_INTERACTION_MANAGER_FRAME_SHOW", function(kind) npcOpen("interaction" .. tostring(kind)) end)
ns.RegisterEvent("PLAYER_INTERACTION_MANAGER_FRAME_HIDE", function(kind) npcClose("interaction" .. tostring(kind)) end)

ns.RegisterEvent("MIRROR_TIMER_START", function(name) Lc.mirrorTimers[name or "?"] = true end)
ns.RegisterEvent("MIRROR_TIMER_STOP", function(name) Lc.mirrorTimers[name or "?"] = nil end)

ns.RegisterEvent("PLAYER_REGEN_DISABLED", function()
	Lc.combat = true
	cancel("standup")
	local p = Lc.pump
	if not p or p.state == "waiting" or p.state == "held" then return end
	local wasShown = VISIBLE[p.state] or p.state == "card"
	if p.kind == "practice" then
		Lc.Finish("practice")
	else
		cancel("settle")
		stopTicker()
		if p.started then
			-- where to resume; a second pull before the pump reopened keeps the first answer
			if VISIBLE[p.state] then p.heldFrom = p.state end
			for _, name in ipairs({ "go", "runEnd", "advance" }) do cancel(name) end
			p.pausedAt = nil
			setState(p, "held")
		else
			setState(p, "waiting")
		end
	end
	if wasShown then ns.Style.Play("pause") end
end)

ns.RegisterEvent("PLAYER_DEAD", deathTax)
ns.RegisterEvent("PLAYER_ALIVE", lives)
ns.RegisterEvent("PLAYER_UNGHOST", lives)

ns.RegisterEvent("PLAYER_REGEN_ENABLED", function()
	Lc.combat = false
	Lc.combatEndAt = ns.now()
	Lc.ScheduleStandup()
	local p = Lc.pump
	if p and (p.state == "waiting" or p.state == "held") then
		setState(p, "settling")
		Lc.CheckSettle()
	end
	if Lc.linesWaiting then after("lines", 1, Lc.LoadLines) end
end)

-- The party turned mixed or stopped being one (the roster, rest, a member's card): a party turning mixed owes a pump
-- that has not opened, as a turn-in made there is; leaving one opens nothing, its unfinished moves ride the next
-- turn-in's pump (David, 2026-10-08: "it should only trigger when returning quest or when I click it on the sidebar").
-- The group's answer to Lc.AskParty follows: a pause while grouped ends with the mixed party (a resume row) and comes
-- back when the same group turns mixed again, a question still open goes (its turn-ins count as they do now, and the
-- group is asked again), and leaving the group forgets the answer. The pause and the question end on what the party
-- is now, whatever Lc.mixed last saw: a member the peer list dropped turns a party mixed with no event.
function Lc.CheckParty()
	if not IsInGroup() then party.asked, party.pause = nil, nil end
	local mixed = Lc.MixedParty()
	if not mixed then
		endGroupPause()
		if #party.waiting > 0 then
			Lc.PartyAnswer(nil)
			ns.Emit("PartyAsk", false)
		end
	end
	if mixed == Lc.mixed then return end
	Lc.mixed = mixed
	if mixed then
		if party.pause and ns.db then writePause(ns.PAUSE.group) end
		Lc.OweIfOwing()
	end
	ns.Emit("Owed") -- the HUD's unfinished line: "after this group" or the next pump
end

ns.RegisterEvent("GROUP_ROSTER_UPDATE", Lc.CheckParty)
ns.RegisterEvent("PLAYER_UPDATE_RESTING", Lc.CheckParty)
ns.On("Peers", Lc.CheckParty)

-- A taxi flight takes and gives back control: the key frame follows (no capture on a taxi, capture after landing).
local function refreshTaxi()
	local p = Lc.pump
	if not p or not VISIBLE[p.state] then return end
	local taxi = UnitOnTaxi("player") and true or false
	if taxi ~= p.taxi then
		p.taxi = taxi
		ns.Emit("State", p)
	end
end
ns.RegisterEvent("PLAYER_CONTROL_GAINED", refreshTaxi)
ns.RegisterEvent("PLAYER_CONTROL_LOST", refreshTaxi)

local function popupShown()
	return StaticPopup_IsAnyDialogShown() or ns.Dialog.AnyShown() or frameShown("ReadyCheckFrame")
		or frameShown("LFDRoleCheckPopup") or frameShown("PVPReadyDialog") or Lc.lfgProposal == true
end

-- What a polled hold waits on: a popup, an in-game cinematic, a movie (it hides UIParent while it plays).
local SHOWN = {
	popup = popupShown,
	cinematic = function() return InCinematic() or IsInCinematicScene() end,
	movie = function() return not UIParent:IsShown() end,
}

-- Holds the pump while `reason` shows, from the event that announced it: the poll ends the hold once it is gone (its
-- frame or state shows popup_grace_s after the event at most), since the client sends no end for a popup, and a
-- cinematic or a movie cut short (a loading screen, a disconnect) may send none either.
function holdWhile(reason)
	Lc.Hold(reason)
	local since = GetTime()
	every(reason, C.lifecycle_tick_s, function()
		if GetTime() - since >= C.popup_grace_s and not SHOWN[reason]() then
			cancel(reason)
			Lc.Unhold(reason)
		end
	end)
end

local function popup()
	holdWhile("popup")
end

-- A question of the addon's own over an open pump (Settings.lua's reset, asked from the pump's Settings or by /qp
-- reset): a hold, as every popup is, so the dim never covers it.
function Lc.Popup()
	if Lc.pump then popup() end
end

-- A cinematic's or a movie's own end event ends its hold at once.
local function ended(reason)
	cancel(reason)
	Lc.Unhold(reason)
end

for _, event in ipairs({ "READY_CHECK", "CONFIRM_SUMMON", "RESURRECT_REQUEST", "DUEL_REQUESTED",
	"PARTY_INVITE_REQUEST", "LFG_ROLE_CHECK_SHOW" }) do
	ns.RegisterEvent(event, popup)
end
-- a battleground queue that popped: its dialog shows on the status "confirm" (Gethe forever PVPHelper.lua, 70058)
ns.RegisterEvent("UPDATE_BATTLEFIELD_STATUS", function(index)
	if index and GetBattlefieldStatus(index) == "confirm" then popup() end
end)
ns.RegisterEvent("LFG_PROPOSAL_SHOW", function()
	Lc.lfgProposal = true
	popup()
end)
for _, event in ipairs({ "LFG_PROPOSAL_DONE", "LFG_PROPOSAL_FAILED", "LFG_PROPOSAL_SUCCEEDED" }) do
	ns.RegisterEvent(event, function() Lc.lfgProposal = false end)
end
ns.RegisterEvent("CINEMATIC_START", function() holdWhile("cinematic") end)
ns.RegisterEvent("CINEMATIC_STOP", function() ended("cinematic") end)
ns.RegisterEvent("PLAY_MOVIE", function() holdWhile("movie") end)
ns.RegisterEvent("STOP_MOVIE", function() ended("movie") end)

-- Commands -------------------------------------------------------------------------------------------------------

ns.AddCommand("pump", function() Lc.Demand() end, "cmd.pump.help")
ns.AddCommand("try", function() Lc.Try() end, "cmd.try.help")

ns.AddCommand("instances", function(arg)
	arg = (arg or ""):lower()
	if arg == "on" or arg == "off" then
		ns.profile.instances = arg == "on"
		ns.Touch()
	end
	ns.Print(ns.profile.instances and "cmd.instances.on" or "cmd.instances.off")
end, "cmd.instances.help")

ns.AddCommand("safety", function() ns.Print("safety.note") end, "cmd.safety.help")

-- /qp pause [hour|4h|char]: this character pauses for the rest of the game day (until 04:00), for an hour, for four
-- hours, or until /qp resume; turn-ins meanwhile count for nothing (David, 2026-10-01: playing with friends).
ns.AddCommand("pause", function(arg)
	arg = (arg or ""):lower()
	if arg == "hour" then
		Lc.Pause(ns.PAUSE.hour)
		ns.Print("cmd.pause.hour")
	elseif arg == "4h" then
		Lc.Pause(ns.PAUSE.fourHours)
		ns.Print("cmd.pause.fourHours")
	elseif arg == "char" then
		Lc.Pause(ns.PAUSE.char)
		ns.Print("cmd.pause.char")
	else
		Lc.Pause(ns.PAUSE.day)
		ns.Print("cmd.pause.day", ns.C.day_offset_h)
	end
end, "cmd.pause.help")

ns.AddCommand("resume", function()
	Lc.Resume()
	ns.Print("cmd.resume.done")
end, "cmd.resume.help")

-- /qp focus [balanced|strength|endurance|flexibility]: sets the focus; without one, opens the window that shows it.
ns.AddCommand("focus", function(arg)
	local key = (arg or ""):lower()
	if ns.Enum.focus[key] then
		Lc.SetFocus(key)
		ns.Print("cmd.focus.set", L["focus." .. key])
	elseif key == "" then
		ns.Sheet.ShowFocus(false)
	else
		ns.Print("cmd.focus.usage")
	end
end, "cmd.focus.help")

-- /qp why: the current (or the last composed) pump's score parts, its quests' mappings and flavour, the five scores
-- per move, the removed candidates; /qp why <quest id>: how that quest scores now and what the quest table says.
ns.AddCommand("why", function(arg)
	local questID = tonumber(arg)
	if questID then
		-- a quest in the log is read again (a /reload forgets what was captured) with the rewards the log shows
		local inLog = C_QuestLog.GetLogIndexForQuestID(questID) and capture(questID)
		local info = Lc.info[questID] or { mapped = ns.Mapping(questID) }
		local xp = inLog and GetQuestLogRewardXP(questID) or 0
		local money = inLog and GetQuestLogRewardMoney(questID) or 0
		local score, tier, parts = ns.Score(Lc.ScoreInput(info, xp, money))
		ns.Print("cmd.why.quest")
		for _, line in ipairs(ns.WhyQuest({ questID = questID, score = score, tier = tier, parts = parts,
			family = ns.Family(info), info = info })) do print(line) end
		return
	end
	local p = Lc.pump or Lc.lastComposed
	local lines = p and ns.Why(p) or {}
	if #lines == 0 then return ns.Print("cmd.why.none") end
	ns.Print("cmd.why.title")
	for _, line in ipairs(lines) do print(line) end
end, "cmd.why.help")

-- /qp how: the name, setup, cue and stop line of the move in front of the player.
ns.AddCommand("how", function()
	local p = Lc.pump or Lc.lastComposed
	local slot = p and p.moves[p.index or 1]
	if not slot then return ns.Print("cmd.how.none") end
	local key = "move." .. ns.Library[slot.move].key
	-- the setup line for the implement the player's gear gives the move, as the Ready view shows it; a timed move with
	-- no form break has no stop line
	local setup, stop = ns.Panel.MoveText(slot.move, "setup"), ns.Panel.MoveStop(slot.move)
	if stop then return ns.Print("cmd.how.line", L[key], setup, L[key .. ".cue"], stop) end
	ns.Print("cmd.how.line.nostop", L[key], setup, L[key .. ".cue"])
end, "cmd.how.help")

-- Corrects a saved set (slice 44, 2026-10-18: Correct a count's window, and /qp undo): the rows ns.CorrectionRows names
-- for it (`set` as stored; `value` the count, a hold's seconds; `load` a loaded set's weight in tenths of a kg, nil
-- keeping its own; `answer` a pose's), written in order, as the game, a game set's own source (the rung rules ask the
-- set's own character's profile, this character's for the sweep). The rows are written whole before the ring's one
-- trim (ns.AddRow held, then ns.TrimDue: slice 44's review, 2026-10-18), so no trim moves the set or the rows its undo
-- rows pair with into the baseline between them. The correction row folds the log again (ns.AddRow); once a rung or a
-- step row went in, it is folded again as the next login would (the sheet pays an undone rung-up nothing and an undone
-- step is no step mastered: the fold skips both). Then the mastered sweep, as after a setup change (Lc.SweepMastered:
-- a step whose moves all still reached their top once another way is mastered again at once, never at the next login
-- alone, which ns.CorrectionRows' `stays` already reads), "DB" (the strip's sheet and profile repaint: the records,
-- the XP and the steps the phones read moved), and an open pump doses its moves of that move again (Lc.Corrected).
-- Returns why the steps stay ("source": an app's set, whose steps are the app's; "later": a later change stands on
-- them; nil: they follow), or false where nothing was written (a read-only save, or the set may not be corrected or
-- has left the ring); second, what the rows changed on the steps (ns.CorrectionRows' third: { rungs = { { ladder,
-- from, to } }, mastered = { { ladder, rung, won } } }, nil none; 2026-10-19), which the done view and /qp undo's line
-- say.
function Lc.WriteCorrection(set, value, load, answer)
	if ns.readOnly then return false end
	local rows, stays, changes = ns.CorrectionRows(ns.db, ns.state, set, value, load, answer, ns.now(), ns.profile,
		ns.Source(set.flags) ~= E.source.app)
	if not rows then return false end
	local steps = false
	for _, r in ipairs(rows) do
		ns.AddRow(r, true)
		steps = steps or r.kind == K.rung or r.kind == K.mastered
	end
	if steps then ns.Fold(ns.db) end
	ns.TrimDue()
	Lc.SweepMastered()
	ns.Emit("DB")
	Lc.Corrected(set)
	return stays, changes
end

-- An open pump's moves without a result yet whose move `takes(id)` names, the one on the screen, a running one and a
-- swapped one's way back too, dosed again where they stand on the records as they stand now (ns.Regoal); what the
-- screen's move showed against the old record goes with it (a "Really?", the Rewards preview, Panel.lua
-- renderRewards). `why` names it in the log.
local function regoal(p, takes, why)
	local slots, here = {}, false
	for i, slot in ipairs(p.moves) do
		if not p.results[i] then
			if takes(slot.move) then
				slots[#slots + 1] = slot
				here = here or i == p.index
			end
			local from = slot.swappedFrom
			if from and takes(from.move) then slots[#slots + 1] = from end
		end
	end
	if #slots == 0 then return end
	ns.Regoal({ state = ns.state, profile = ns.profile, now = ns.now() }, slots)
	if here then p.really, p.reallyLoad, p.reallyCheck, p.preview = nil, nil, nil, nil end
	ns.Log("I", "pump %d: %d move(s) dosed again after %s", p.id, #slots, why)
	ns.Emit("State", p)
end

-- A count corrected while a pump is open (slice 44: allowed, as a record cleared is, David, 2026-10-14: "it should not
-- be blocked during the pump. that's exactly when you need it"): every move of the corrected set's move that has no
-- result yet is dosed again on the record as it now stands (regoal). A pump not opened yet composes at its open
-- anyway, and a practice pump writes nothing.
function Lc.Corrected(set)
	local p = Lc.pump
	if not p or p.kind == "practice" or not p.opened then return end
	local id = ns.Merged[set.move] or set.move
	regoal(p, function(move) return (ns.Merged[move] or move) == id end, "a correction")
end

-- A record cleared or a ladder started over (Settings.lua): /qp undo no longer corrects a count typed before it, whose
-- record went (ns.ClearTakes), and an open pump's moves it took aim as a fresh pick of them would (David, 2026-10-14:
-- "it should not be blocked during the pump. that's exactly when you need it"): a ladder started over while the pump
-- has no move done or under way (p.started) composes the pump again (recompose: its placement, the rung rule), else
-- every move it took that has no result yet is dosed again where it stands (regoal: a max set). A pump not opened yet
-- composes at its open anyway. A pump started from a step's card that has not started asks ns.StartFor again for the
-- target the card started (`p.fromTree`; a Long Haul, `haul`, ns.HaulStart) after a ladder started over, opened or not,
-- so its one move is dosed at the
-- rung the ladder stands on now (a Find my step where the restart unplaced it); a target the restart refuses ends the
-- pump unwritten, nothing of it done, as a pump dropped before it shows, and the card says why at its next draw.
function Lc.Cleared(row)
	if Lc.lastTyped and ns.ClearTakes(row, Lc.lastTyped.row.move) then Lc.lastTyped = nil end
	local p = Lc.pump
	if not p or p.kind == "practice" then return end
	if row.reps == E.clear.ladder and p.chosen and not p.started then
		local from = p.fromTree
		local s
		if from.haul then
			s = ns.HaulStart(ns.state, ns.profile, ns.now())
		else
			s = ns.StartFor(ns.state, ns.profile, from.check and { check = from.check }
				or from.move and { move = from.move } or { ladder = from.ladder, rung = from.rung }, ns.now())
		end
		if not s then return Lc.Finish("dropped") end
		p.chosen = chosenOf(s)
		-- the ladder the new start runs, so the return lands on the ladder the set trains (Tree.Return)
		from.via, from.start = s.via, s.ladder
	end
	if not p.opened then return end
	if row.reps == E.clear.ladder and not p.started then return recompose(p) end
	regoal(p, function(move) return ns.ClearTakes(row, move) end, "a clear")
end

-- The latest typed count of a move whose name holds `words`, any day the ring holds: /qp undo's named form, for a
-- count found wrong after the five minutes (David, 2026-10-04: a side move's reps typed with both sides added). Only a
-- count typed in the game (a rep set, a hold, a counted set; never a correction's or its load row, which repeat the
-- set's move with their round in `outcome`): an app row (slice 26) is the app's own. None when a clear row, the game's
-- or the app's, took its record since (ns.CorrectableRow).
local function typedRow(words)
	local want, log = words:lower(), ns.db and ns.db.log or {}
	for i = #log, 1, -1 do
		local r = ns.DecodeRow(log[i])
		local m = r and ns.Library[r.move]
		if m and (r.kind == K.rep or r.kind == K.hold or r.kind == K.cardio) and ns.TYPED[r.outcome]
			and ns.Source(r.flags) == E.source.game and L["move." .. m.key]:lower():find(want, 1, true) then
			return ns.CorrectableRow(ns.state, r) and r or nil
		end
	end
end

-- The weight after /qp undo's count ("14", "14kg", "14 kg", "12,5"): its tenths of a kg (false when it is 0) and the
-- words after it; nil and the text as it was when the text starts with no number (a move's name).
local function undoWeight(text)
	local number, tail = text:match("^(%d+[.,]?%d*)(.*)$")
	if not number then return nil, text end
	local unit, rest = tail:match("^%s*([kK][gG])(.*)$")
	if unit then tail = rest end
	if tail ~= "" and not tail:find("^%s") then return nil, text end
	return ns.Sheet.ParseKg(number) or false, tail:match("^%s*(.-)$")
end

-- /qp undo's lines after its correction of set row `row`, in the window's words (Settings.lua): the set as the fold
-- reads it now, Settings_.FoldedWords ("15 reps", "45 s", "11 kg × 16"; a 0 done, "Done without a count", as typing
-- 0 is; a capped max set corrected over its top, its top: slice 44's review), then what it did to the steps or why a
-- step stays where it is (`stays` and `changes`, Lc.WriteCorrection's: Settings_.StepLines, the done view's lines,
-- 2026-10-19), a chat line each.
local function undoDone(row, name, stays, changes)
	local Settings_ = ns.Settings
	ns.Print("cmd.correct.done", name, Settings_.FoldedWords(row.move, row.kind == K.hold and "seconds" or "count", row))
	for _, line in ipairs(Settings_.StepLines(stays, changes)) do ns.Print(unpack(line)) end
end

-- /qp undo <count> [kg]: corrects the last typed count (and the weight it was done with) for five minutes; /qp undo
-- <count> [kg] <move>: that move's latest typed count, any time. Both through Correct a count's one rule
-- (Lc.WriteCorrection, slice 44): its round, and the rung rules asked again only while the set's steps are the latest.
ns.AddCommand("undo", function(arg)
	if ns.readOnly then return ns.Print("cmd.undo.none") end
	local digits, rest = (arg or ""):match("^%s*(%d*)%s*(.-)%s*$")
	local n = tonumber(digits)
	local load, words = undoWeight(rest)
	local row
	if words ~= "" then
		row = typedRow(words)
		if not row then return ns.Print("cmd.undo.noMove", words) end
	else
		local last = Lc.lastTyped
		if not last or ns.now() - last.row.ts > C.undo_min * 60 then return ns.Print("cmd.undo.none") end
		row = last.row
	end
	-- the count and the weight as the fold reads them now, an earlier correction's
	local name, count, was = L["move." .. ns.Library[row.move].key], ns.SetValue(ns.state, row), ns.SetLoad(ns.state, row)
	local loaded = ns.Loaded(row.move) ~= nil
	if load ~= nil and not loaded then return ns.Print("cmd.undo.noKg", name) end
	if not n or n > ns.CountMax({ kind = row.kind, counted = ns.HasFlag(row.flags, "counted") }) or load == false
		or (load and load > C.load_max_kg * 10) then
		if loaded then return ns.Print("cmd.undo.usageLoad", name, ns.Sheet.LoadAmount(row.move, was, count)) end
		return ns.Print("cmd.undo.usage", name, count)
	end
	if row == (Lc.lastTyped and Lc.lastTyped.row) then Lc.lastTyped = nil end
	local stays, changes = Lc.WriteCorrection(row, n, load)
	if stays == false then return ns.Print("cmd.undo.none") end
	undoDone(row, name, stays, changes)
end, "cmd.undo.help")

local function yards(v)
	return BreakUpLargeNumbers(math.floor(v + 0.5))
end

-- The move figures' pack as Figure.Check read it at login: why the key poses show where the animation should.
local FIGURE_STATUS = { ok = "cmd.status.figures.ok", old = "cmd.status.figures.old",
	missing = "cmd.status.figures.missing", disabled = "cmd.status.figures.disabled" }
-- The paired phone by the pairing block's id (ns.IsPaired knows these two).
local PHONES = { ios = "cmd.status.phone.ios", android = "cmd.status.phone.android" }
-- The helper by ns.HelperState, none for one that never wrote a pairing block ("on" names its version).
local COMPUTER = { none = "cmd.status.computer.none", off = "cmd.status.computer.off",
	blind = "cmd.status.computer.blind", stopped = "cmd.status.computer.stopped",
	disabled = "cmd.status.computer.disabled", outdated = "cmd.status.computer.outdated" }

-- A day's pumps, moves and records in the words of the day card's Share line, each singular at 1 (ns.Plural).
local function dayCounts(c)
	return table.concat({ ns.Plural("streamer.share.pumps", c.pumps):format(c.pumps),
		ns.Plural("streamer.share.moves", c.moves):format(c.moves),
		ns.Plural("streamer.share.records", c.records):format(c.records) }, ", ")
end

ns.AddCommand("status", function()
	local s = ns.state
	-- the day's pumps, moves and records every source's, as the Today tab and the day card count them, then the app's
	-- share (slice 26)
	local today = ns.Today(s, ns.now())
	local app = today.app
	ns.Print("cmd.status.today", dayCounts(today))
	if app.pumps + app.moves > 0 then ns.Print("cmd.status.today.app", dayCounts(app)) end
	ns.Print("cmd.status.owed", ns.OwedCount(s))
	local st = ns.Stretch.Current()
	ns.Print("cmd.status.stretch", yards(st.foot), yards(st.mount), yards(st.swim), yards(st.taxi),
		math.floor(st.active / 60))
	if ns.svBroken then
		ns.Print(ns.meta.restoredFrom == "mirror" and "cmd.status.saving.restored" or "cmd.status.saving.lost")
	elseif ns.loadedStamp ~= nil then
		ns.Print("cmd.status.saving.loaded")
	else
		ns.Print("cmd.status.saving.unknown")
	end
	if FIGURE_STATUS[ns.Figure.state] then ns.Print(FIGURE_STATUS[ns.Figure.state]) end
	local silence = ns.profile.speech and ns.Speech.Silence()
	if silence then ns.Print("cmd.status.voice", L["speech.silent." .. silence]) end
	-- each companion on its own line, as the setup screen's steps read them (truth-1, 2026-10-09), the phone's only
	-- while the setup asks for it; a newer addon in the helper's feed under the computer's
	local pairing = ns.Pairing()
	local helper = ns.HelperState()
	if helper == "on" then
		ns.Print("cmd.status.computer.on", tostring(pairing.helper))
	else
		ns.Print(COMPUTER[helper or "none"])
	end
	local update, updateKey = Lc.Update()
	if update then ns.Print(updateKey, update) end
	local asksPhone = Setup.AsksPhone()
	local quiet = ns.PhoneQuiet()
	if quiet then
		ns.Print("cmd.status.phone.quiet", L[PHONES[pairing.phone]], date("%Y-%m-%d", quiet))
	elseif ns.IsPaired() then
		ns.Print("cmd.status.phone.paired", L[PHONES[pairing.phone]])
	elseif asksPhone then
		ns.Print("cmd.status.phone.none")
	end
	local helperDone, phoneDone = Setup.Steps()
	if Setup.Blocks() then
		ns.Print((Setup.Page() or "companions") ~= "companions" and "cmd.status.setup.yours"
			or asksPhone and "cmd.status.setup" or "cmd.status.setup.one")
	elseif not (helperDone and phoneDone) then
		ns.Print(asksPhone and "cmd.status.setup.how" or "cmd.status.setup.how.one")
	end
	if Lc.Locked() then ns.Print("cmd.status.locked", C.day_offset_h) end
	ns.Print("cmd.status.state", Lc.pump and L["state." .. Lc.pump.state] or L["state.idle"])
end, "cmd.status.help")

-- The Ready keybinding (Bindings.xml).
BINDING_HEADER_QUESTPUMP = L["binding.header"]
BINDING_NAME_QUESTPUMP_START = L["binding.start"]

function QuestPump_StartPump()
	Lc.Demand()
end

--@debug@
-- Debug commands (stripped by the packager): they fire the same handlers as the real events, so the hub merge, the
-- owed queue, combat, the day lock and the setup screen are tested in seconds.
ns.AddCommand("dev", function(arg)
	local cmd, rest = (arg or ""):match("^(%S*)%s*(.-)$")
	if cmd == "turnin" then
		local questID, xp = rest:match("^(%d+)%s*(%d*)")
		questID = tonumber(questID)
		if not questID then return end
		Lc.seen[questID] = true
		ns.Dispatch("QUEST_TURNED_IN", questID, tonumber(xp) or 0, 0)
	elseif cmd == "hub" then
		ns.Dispatch("QUEST_GREETING")
		for i = 1, tonumber(rest) or 3 do
			Lc.seen[900000 + i] = true
			ns.Dispatch("QUEST_TURNED_IN", 900000 + i, 0, 0)
		end
		ns.Dispatch("QUEST_FINISHED")
	elseif cmd == "combat" then
		ns.Dispatch(Lc.combat and "PLAYER_REGEN_ENABLED" or "PLAYER_REGEN_DISABLED")
	elseif cmd == "clock" then
		local n, unit = rest:match("^%+?(%d+)([dh])$")
		if n then
			ns.clockOffset = ns.clockOffset + tonumber(n) * (unit == "d" and 86400 or 3600)
			-- the rows keep the shifted ts, so the offset outlives a /reload (the clock never runs behind them)
			ns.SetMeta("devClock", ns.clockOffset)
		end
		ns.Emit("Clock")
	elseif cmd == "pair" then
		-- `/qp dev pair nophone`: a helper whose feed says the phone apps are not in the stores yet, and no phone
		ns.devPairing = rest == "nophone" and { helper = "dev", phone_apps = false, seen_at = ns.now() }
			or { helper = "dev", phone = "ios", phone_since = ns.now(), seen_at = ns.now() }
		Setup.Evaluate()
	elseif cmd == "peer" then
		ns.Social.DevPeer(rest) -- forged cards and messages through the addon-message handler (Social.lua)
	end
end)
ns.On("DB", function() ns.clockOffset = ns.meta and tonumber(ns.meta.devClock) or 0 end)
--@end-debug@
