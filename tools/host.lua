-- The host of the addon's pure files (docs/plan.md slice 35, "The engine is the addon's own Lua"): the files the
-- engine, the row grammar and the fold need (H.FILES, in QuestPump.toc's order) and the WoW surface they touch,
-- exactly, so the same files compose, judge and fold outside the client: in tools/gen-engine.lua
-- (contract/fixtures/engine.json) under PUC Lua 5.1, and in the desktop app, whose helper/crates/questpump-engine
-- embeds this file, tools/apppump.lua and H.FILES and runs them on PUC Lua 5.1 through mlua. Loading the files needs
-- C_AddOns.GetAddOnMetadata (no version: ns.version "dev"), GetBuildInfo, CreateFrame (Core.lua's one event frame:
-- SetScript and RegisterEvent, which do nothing here), C_EventUtils.IsEventValid and SlashCmdList; running them, date
-- ("%Y-%m-%d", "*t" and "%Y-%m-%d %H:%M:%S"), time, CopyTable, geterrorhandler and print (nothing printed), with wipe,
-- strtrim and issecretvalue (nothing is secret here) beside them. Any other global a file reads raises at once, naming
-- it (H.env), but the addon's own globals the files read nil-safely (H.NIL); math.random and math.randomseed raise: the
-- engine is deterministic. The clock and the time zone come from the embedder: `now`, the unix time, and `offset`, the
-- seconds east of UTC at an instant (the tools keep UTC, so `make fixtures` writes the same bytes on every machine; the
-- desktop passes the computer's zone). tools/wow_stub.lua installs these globals first and its richer stand-ins over
-- them, and tools/lint.lua checks that H.FILES follows QuestPump.toc and that the files load under H.env alone.
-- Usage: local H = dofile("tools/host.lua"); local env = H.env({ now = ..., offset = ... }); H.load(env, read, ns)

local H = {}

-- One line, which helper/crates/questpump-engine/build.rs reads: a file joins it in QuestPump.toc's order.
H.FILES = { "Core.lua", "Constants.lua", "DB.lua", "Library.lua", "Engine.lua", "Progress.lua", "Stats.lua" }

-- The addon's globals the files read nil-safely: the generated quest table (Quests.lua and QuestPump_Lines), which only
-- a quest's pump reads, and the save and the mirror, which only the client's login binds.
H.NIL = { QuestPumpQuests = true, QuestPumpLines = true, QuestPumpMoneyRef = true, QuestPumpDB = true,
	QuestPumpMirror = true, QuestPumpMirrorAlive = true }

-- The base library the files and tools/apppump.lua use; every other base function stays out (no dofile, loadfile,
-- loadstring, setfenv, getfenv, require, io or os).
local BASE = { "assert", "error", "getmetatable", "ipairs", "next", "pairs", "pcall", "rawget", "rawset", "select",
	"setmetatable", "tonumber", "tostring", "type", "unpack", "xpcall" }

local DAY = 86400

local function noop() end

-- The civil date of a day number (days since 1970-01-01): Hinnant's civil_from_days, the inverse of Stats.lua's
-- ns.DayIndex. Returns the year, the month, the day and the day of the year (1 on 1 January).
local function civil(days)
	local z = days + 719468
	local era = math.floor(z / 146097)
	local doe = z - era * 146097
	local yoe = math.floor((doe - math.floor(doe / 1460) + math.floor(doe / 36524) - math.floor(doe / 146096)) / 365)
	local doy = doe - (365 * yoe + math.floor(yoe / 4) - math.floor(yoe / 100))
	local mp = math.floor((5 * doy + 2) / 153)
	local d = doy - math.floor((153 * mp + 2) / 5) + 1
	local m = mp < 10 and mp + 3 or mp - 9
	local y = yoe + era * 400 + (m <= 2 and 1 or 0)
	-- the day of the year: the days since 1 March (doy) moved to 1 January, a leap day counted from March on
	local leap = (y % 4 == 0 and y % 100 ~= 0 or y % 400 == 0) and 1 or 0
	local yday = m <= 2 and doy - 305 or doy + 60 + leap
	return y, m, d, yday
end

-- A deep copy, as Blizzard's CopyTable makes one (no metatable).
local function copyTable(src)
	local copy = {}
	for k, v in pairs(src) do copy[k] = type(v) == "table" and copyTable(v) or v end
	return copy
end

-- The globals the files touch, on the embedder's clock: opts = { now = function() -> unix seconds, offset =
-- function(t) -> seconds east of UTC at t, report = function(err) (geterrorhandler's; nil raises the error) }. A
-- fresh table each call.
function H.globals(opts)
	local now, offset = opts.now, opts.offset
	local lastDays, lastDay -- the last day's "%Y-%m-%d": the fold asks for the same day again and again
	local function date(fmt, t)
		t = math.floor(t or now())
		local lt = t + offset(t)
		local days = math.floor(lt / DAY)
		if fmt == "%Y-%m-%d" then
			if days ~= lastDays then lastDays, lastDay = days, ("%04d-%02d-%02d"):format(civil(days)) end
			return lastDay
		end
		local secs = lt - days * DAY
		local y, m, d, yday = civil(days)
		local hour, min, sec = math.floor(secs / 3600), math.floor(secs % 3600 / 60), secs % 60
		if fmt == "*t" then
			-- wday 1 is Sunday, as os.date gives it (1970-01-01 was a Thursday)
			return { year = y, month = m, day = d, hour = hour, min = min, sec = sec, wday = (days + 4) % 7 + 1,
				yday = yday, isdst = false }
		elseif fmt == "%Y-%m-%d %H:%M:%S" then
			return ("%04d-%02d-%02d %02d:%02d:%02d"):format(y, m, d, hour, min, sec)
		end
		error(("tools/host.lua: date format %q is not the engine's"):format(tostring(fmt)), 2)
	end
	local frame = { SetScript = noop, RegisterEvent = function() return true end }
	return {
		print = noop,
		date = date,
		time = function(t)
			if t ~= nil then error("tools/host.lua: time() takes no table here", 2) end
			return now()
		end,
		CopyTable = copyTable,
		wipe = function(t)
			for k in pairs(t) do t[k] = nil end
			return t
		end,
		strtrim = function(s) return (s:gsub("^%s+", ""):gsub("%s+$", "")) end,
		issecretvalue = function() return false end,
		-- the client reports an error and goes on; here the embedder hears it, or it stays loud
		geterrorhandler = function() return opts.report or function(err) error(err, 0) end end,
		GetBuildInfo = function() return "1.60.1", "0", "", 16001 end,
		C_AddOns = { GetAddOnMetadata = noop },
		CreateFrame = function() return frame end,
		C_EventUtils = { IsEventValid = function() return true end },
		SlashCmdList = {},
	}
end

-- Raises: the engine draws no random number (its rolls are seeded from the quest, the day and the pump: Engine.lua).
local function deterministic()
	error("tools/host.lua: the engine is deterministic: no math.random", 2)
end

local function copyOf(lib)
	local t = {}
	for k, v in pairs(lib) do t[k] = v end
	return t
end

-- A fresh environment for the files and tools/apppump.lua: BASE, math (random and randomseed raising), string and
-- table, then H.globals(opts); reading any other global raises, but H.NIL's, which read nil. Writes land in it
-- (Core.lua's SLASH_QUESTPUMP1 and QuestPump_OnAddonCompartmentClick).
function H.env(opts)
	local env = {}
	for _, name in ipairs(BASE) do env[name] = _G[name] end
	env.math = copyOf(math)
	env.math.random, env.math.randomseed = deterministic, deterministic
	env.string, env.table = copyOf(string), copyOf(table)
	for name, value in pairs(H.globals(opts)) do env[name] = value end
	return setmetatable(env, { __index = function(_, name)
		if H.NIL[name] then return nil end
		error(("reads the global %s tools/host.lua lacks"):format(tostring(name)), 2)
	end })
end

-- Loads H.FILES into `ns` under `env`, each as the client runs an addon file (its chunk named by its path, called with
-- the addon's name and the namespace); `read(path)` returns a file's source. Returns ns.
function H.load(env, read, ns)
	ns = ns or {}
	for _, path in ipairs(H.FILES) do
		local chunk = assert(loadstring(read(path), "@" .. path))
		setfenv(chunk, env)
		chunk("QuestPump", ns)
	end
	return ns
end

return H
