-- The addon's log (2026-09-30; David: "make sure that general logging works great"): one line for each thing worth
-- knowing afterwards, kept in its own SavedVariable, QuestPumpLog, which the client writes to disk at every /reload and
-- logout. A client that never reads its save back (the Forever beta's bug) starts it empty at each load, so its file
-- holds the last session only. No reset of the save touches it, and no export, mirror or strip carries it: it never
-- leaves the player's computer. Its warnings and notes also go into the client's own log through C_Log, beside the Lua
-- error Blizzard's handler writes there for every addon (Blizzard_ScriptErrors, Gethe `forever`), and every Lua error
-- the client shows comes into this log too (below). The lines are English diagnostics for whoever reads the file, never
-- player text; /qp log prints the last ones. The helper parses the save file whole (questpump-core `lua::globals`), so
-- a line holds no control character and is cut at log_line_chars.

local ADDON, ns = ...
local L = ns.L
local T = ns.Timers()

-- The client's log takes the warnings and notes; an error reaches it through the client's handler when ns.Report hands
-- it on (Blizzard_ScriptErrors' HandleLuaError), so it is never written there twice.
local CLIENT_LOG = { W = "LogWarningMessage", I = "LogMessage" }

local pending = {} -- the lines written before the save was loaded (ADDON_LOADED), moved into it then
local lines = pending
-- the newest line's text, the text as it came in, how often it came in a row, and when first
local last = { text = nil, raw = nil, count = 0, at = nil }

-- A text as a line holds it: no control character, cut at log_line_chars.
local function clean(text)
	return (tostring(text):gsub("%c", " "):sub(1, ns.C.log_line_chars))
end

-- One line: "2026-09-30 14:31:05 E <text>"; the same text again in a row counts on the line before, which keeps its
-- first time and names the last ("(x3, last 14:31:40)"), so a repeating error never floods the file. `text` may carry
-- format arguments; a secret value never enters the log.
function ns.Log(level, text, ...)
	if select("#", ...) > 0 then text = tostring(text):format(...) end
	if issecretvalue(text) then text = "(a secret value)" end
	last.raw = text
	text = clean(text)
	if last.text == text and #lines > 0 then
		last.count = last.count + 1
		lines[#lines] = ("%s %s %s (x%d, last %s)"):format(last.at, level, text, last.count, date("%H:%M:%S"))
		return -- counted here; the client's log has the line once
	end
	last.text, last.count, last.at = text, 1, date("%Y-%m-%d %H:%M:%S")
	lines[#lines + 1] = ("%s %s %s"):format(last.at, level, text)
	local over = #lines - ns.C.log_lines
	if over >= ns.C.log_batch then -- trimmed in steps, as the save's ring is, so a line's cost never grows
		for i = 1, #lines - over do lines[i] = lines[i + over] end
		for i = #lines, #lines - over + 1, -1 do lines[i] = nil end
	end
	if CLIENT_LOG[level] then C_Log[CLIENT_LOG[level]](ADDON .. ": " .. text) end
end

-- The last `n` lines, oldest first.
function ns.LogTail(n)
	local out = {}
	for i = math.max(1, #lines - n + 1), #lines do out[#out + 1] = lines[i] end
	return out
end

ns.RegisterEvent("ADDON_LOADED", function(name)
	if name ~= ADDON then return end
	local saved = type(QuestPumpLog) == "table" and QuestPumpLog or {}
	if type(saved.lines) ~= "table" then saved.lines = {} end
	saved.v = 1
	QuestPumpLog = saved
	lines = saved.lines
	for _, line in ipairs(pending) do lines[#lines + 1] = line end
	wipe(pending)
end)

-- What the client reports for any addon: a Lua warning (an execution limit hit, a deprecated call) and a protected
-- action refused to tainted code (the first argument names the addon; Gethe `forever`, ScriptWarnings and
-- RestrictedActions documentation).
ns.RegisterEvent("LUA_WARNING", function(warningText) ns.Log("W", "Lua warning: %s", warningText) end)
ns.RegisterEvent("ADDON_ACTION_BLOCKED", function(addon, fn) ns.Log("W", "blocked: %s called %s", addon, fn) end)
ns.RegisterEvent("ADDON_ACTION_FORBIDDEN", function(addon, fn) ns.Log("W", "forbidden: %s called %s", addon, fn) end)

-- Every Lua error the client's handler shows, any addon's and from any path (a frame script, a hook, a file's load),
-- not only the ones ns.Report hands on (David, 2026-10-13, "a few lua crashes last play session": none reached this
-- log, and the client empties Logs/General.log at its next start, which lost their text). A post-hook on the error
-- window's DisplayMessageInternal, which the handler Blizzard_ScriptErrorsFrame registers calls for every error,
-- shown or not (Gethe `forever` 1.60.1, 70338); never a wrapper of the error handler, which would run HandleLuaError
-- tainted. The error ns.Report has just logged comes in as the newest line's text did and is not written twice; an
-- error already logged this session counts only on the newest line, so two raised every frame never flood the ring.
local ERROR_MESSAGE = 0 -- the window's ErrorMessageType; its warnings (1) are LUA_WARNING's line above
local shownErrors = {}

local function errorShown(_, message, messageType, stack)
	if messageType ~= ERROR_MESSAGE then return end
	local text = "Lua error: (a secret value)"
	if not (issecretvalue(message) or issecretvalue(stack)) then
		if message == last.raw then return end
		text = "Lua error: " .. tostring(message) .. (type(stack) == "string" and ns.StackTrail(stack) or "")
	end
	local line = clean(text)
	if shownErrors[line] and last.text ~= line then return end
	shownErrors[line] = true
	ns.Log("E", text)
end

if ScriptErrorsFrame and ScriptErrorsFrame.DisplayMessageInternal then
	hooksecurefunc(ScriptErrorsFrame, "DisplayMessageInternal", errorShown)
else
	ns.Log("W", "no Lua error window to hook: an error outside ns.Report reaches the client's log alone")
end

-- The login's summary, once the save is bound and the setup screen decided (a second after PLAYER_ENTERING_WORLD):
-- the build, the save and its rows by source, the companions, the helper's state, the sound the voice needs and the
-- figure pack; then what the mirror brought.
local function saveState()
	if ns.readOnly then return "read-only (a newer QuestPump's save)" end
	if ns.svBroken then
		return ns.meta.restoredFrom == "mirror" and "not read back by the game, restored from the helper's copy"
			or "not read back by the game"
	end
	return ns.loadedStamp and "loaded" or "new"
end

-- Rows by source (DB.lua ns.RowsBySource, ns.MergeMirror): "game 812, catch-up 4, app 96".
local function sources(counts)
	local S = ns.Enum.source
	return ("game %d, catch-up %d, app %d"):format(counts[S.game], counts[S.catchup], counts[S.app])
end

local function setupState()
	local pairing = ns.Pairing()
	if ns.IsPaired() then
		return ("paired (helper %s, phone %s)"):format(tostring(pairing.helper), tostring(pairing.phone))
	end
	local where = ns.Lifecycle.TrialBegun() and "playing without the companions" or "at the setup screen"
	-- the helper's feed says the phone apps are not in the stores yet: the setup asks for the helper alone
	return ns.Setup.AsksPhone() and where or where .. ", no phone asked"
end

-- The helper as ns.HelperState read it, with how long before this load its sign of life was written, so a running
-- helper read as "off" can be traced from the saved log.
local function helperState()
	local alive = QuestPumpMirrorAlive
	return ("helper %s, alive %s s before the load"):format(ns.HelperState() or "none",
		type(alive) == "number" and ns.loadedAt - alive or "?")
end

ns.RegisterEvent("PLAYER_ENTERING_WORLD", function(isInitialLogin, isReloadingUi)
	if not (isInitialLogin or isReloadingUi) then return end
	T.after("login", 1, function()
		local _, build, _, toc = GetBuildInfo()
		local how = isReloadingUi and "reload" or "login"
		if not (ns.db and ns.meta and ns.profile) then -- the bind raised: its error is the line before
			return ns.Log("W", "%s: v%s, build %s: the save is not bound", how, ns.version, tostring(build))
		end
		local silence = ns.Speech.Silence()
		ns.Log("I", "%s: v%s, build %s, interface %s, %s; save %s, %d rows (%s); %s; %s; voice %s; figures %s", how,
			ns.version, tostring(build), tostring(toc), GetLocale(), saveState(), #ns.db.log,
			sources(ns.RowsBySource(ns.db.log)), setupState(), helperState(),
			not ns.profile.speech and "off" or silence and ("silent: " .. silence) or "on", tostring(ns.Figure.state))
		-- what the helper's mirror brought at the bind (ns.BindDB): a reset newer than the save, the rows merged by source
		local merge = ns.mirrorMerge or {}
		if merge.resetAt then
			ns.Log("I", "mirror: a reset at %s, newer than this save, reset it (the old one is the backup)",
				date("%Y-%m-%d %H:%M:%S", merge.resetAt))
		end
		if merge.taken then ns.Log("I", "mirror merged: %s", sources(merge.taken)) end
	end)
end)

-- The pumps, as the lifecycle moves them: offered (its kind and quests, each move with its dose and why it is in the
-- pump, the unfinished moves left waiting, the lost-save line when its Ready view carries it) and ended (how, and how
-- many moves have a result, were banked, or were left unfinished: owed moves carry no result, only their rows).
-- David, 2026-10-04: a 221-yd quest's pump of three moves could not be read from the log, which named the moves
-- only: the major move, the quest's own move and an unfinished all-out window riding along looked alike.
local O, K = ns.Enum.outcome, ns.Enum.kind
local seen = setmetatable({}, { __mode = "k" }) -- the state each pump was last seen in
local opened = setmetatable({}, { __mode = "k" }) -- the pumps whose opening is logged

-- Why a move is in its pump: unfinished (riding along), the Trial's placement, the major move, the stand-up move, or
-- the day plan's block that took it (the engine's why list); an all-out window says so too.
local function role(p, slot)
	local tag
	if slot.owed then
		tag = ("unfinished since %s, quest %d"):format(date("%m-%d %H:%M", slot.owed.ts), slot.owed.questID)
	elseif slot.trial then
		tag = "placement"
	elseif slot.major then
		tag = slot.waited and ("major, %s %.1f h since its last set"):format(L["ladder." .. slot.ladder], slot.waited)
			or ("major, %s never set"):format(L["ladder." .. slot.ladder])
	elseif slot.standup then
		tag = "stand-up"
	else
		for _, w in ipairs(p.moves.why or {}) do
			if w.move == slot.move then tag = w.block or (w.finisher and "finisher") end
		end
	end
	if slot.allout then tag = (tag and tag .. ", " or "") .. "all-out" end
	return tag
end

local function moveEntry(p, slot)
	local m = ns.Library[slot.move]
	local text = m and m.key or tostring(slot.move)
	if slot.target then
		text = text .. (slot.kind == K.hold and " %d s" or " %d reps"):format(slot.target)
	elseif slot.seconds then
		text = text .. (" %d s"):format(slot.seconds)
	end
	local tag = role(p, slot)
	return tag and text .. " [" .. tag .. "]" or text
end

-- The pump's kind, then each quest it pays with its tier and the yards walked to it, and the moves per quest that sized
-- them when not the default (ns.Compose's why.pace).
local function pumpHead(p)
	local parts = { p.kind }
	for _, q in ipairs(p.quests or {}) do
		local foot = q.share and q.share.foot
		parts[#parts + 1] = ("quest %d tier %d%s"):format(q.questID, q.tier or 0,
			foot and foot >= 1 and (", %d yd"):format(foot) or "")
	end
	local pace = p.moves and p.moves.why and p.moves.why.pace
	if pace and pace ~= ns.defaults.pace then parts[#parts + 1] = "pace " .. pace end
	return table.concat(parts, "; ")
end

ns.On("State", function(p, how)
	if not p or p.kind == "practice" or seen[p] == p.state then return end
	if p.state == "open" and not opened[p] then
		opened[p] = true
		local entries, riders = {}, 0
		for i, slot in ipairs(p.moves) do
			entries[i] = moveEntry(p, slot)
			if slot.owed then riders = riders + 1 end
		end
		local waiting = ns.state and ns.OwedCount(ns.state) - riders or 0
		ns.Log("I", "pump %d (%s) opened: %s%s%s", p.id, pumpHead(p), table.concat(entries, ", "),
			waiting > 0 and ("; %d unfinished still waiting"):format(waiting) or "",
			p.lostNote and ", with the lost-save line" or "")
	elseif p.state == "done" then
		local done, banked = 0, 0
		for _, outcome in pairs(p.results) do
			if outcome == O.banked then banked = banked + 1 else done = done + 1 end
		end
		-- a skipped move left the pump's list (Lc.Skip, which logs each as it goes): counted on its own
		ns.Log("I", "pump %d ended (%s): %d done, %d banked, %d skipped, %d unfinished of %d", p.id,
			tostring(how or "done"), done, banked, p.skipped or 0, #p.moves - done - banked, #p.moves + (p.skipped or 0))
	end
	seen[p] = p.state
end)

-- /qp log [n]: the last lines (15 by default); /qp log clear empties the log.
ns.AddCommand("log", function(arg)
	arg = strtrim(arg or ""):lower()
	if arg == "clear" then
		wipe(lines)
		last.text, last.count = nil, 0
		return ns.Print("cmd.log.cleared")
	end
	local tail = ns.LogTail(tonumber(arg) or ns.C.log_show)
	ns.Print("cmd.log.header", #tail, #lines)
	for _, line in ipairs(tail) do print(line) end
end, "cmd.log.help")
