-- The stretch: the play since the previous pump was offered (docs/plan.md slice 1, "Odometer"). A 0.25-s ticker
-- integrates GetUnitSpeed into yards per state (on foot, mounted, swimming, on a taxi) plus the active seconds
-- (online, not AFK, not resting, not on a taxi). When a pump is offered, its stretch is split
-- evenly across the quests it pays and a new one starts, so time and distance are attributed once, never per quest.

local ADDON, ns = ...
local C = ns.C

local MAX_STEP_S = 1     -- a loading screen or a hitch never counts as travel

local S = {}
ns.Stretch = S

function S.New()
	return { foot = 0, mount = 0, swim = 0, taxi = 0, active = 0 }
end

-- One step of the integration; `sample` = { speed, mounted, swimming, taxi, afk, resting }. Pure.
function S.Step(s, dt, sample)
	local yards = sample.speed * dt
	if sample.taxi then
		s.taxi = s.taxi + yards
	elseif sample.swimming then
		s.swim = s.swim + yards
	elseif sample.mounted then
		s.mount = s.mount + yards
	else
		s.foot = s.foot + yards
	end
	if not sample.afk and not sample.resting and not sample.taxi then s.active = s.active + dt end
end

-- The stretch split evenly across n quests. Pure.
function S.Split(s, n)
	local shares = {}
	n = math.max(1, n)
	for i = 1, n do
		shares[i] = { foot = s.foot / n, mount = s.mount / n, swim = s.swim / n, taxi = s.taxi / n, active = s.active / n }
	end
	return shares
end

S.current = S.New()

function S.Current()
	return S.current
end

-- Hands over the current stretch and starts the next one.
function S.Take()
	local s = S.current
	S.current = S.New()
	return s
end

local last

local function sample()
	local speed = GetUnitSpeed("player")
	if issecretvalue(speed) then return nil end -- SecretWhenUnitStatsRestricted: no arithmetic on it
	return {
		speed = speed,
		mounted = IsMounted(),
		swimming = IsSwimming(),
		taxi = UnitOnTaxi("player"),
		afk = ns.Readable(UnitIsAFK("player")), -- secret in chat lockdown (70058): nil counts as not AFK
		resting = IsResting(),
	}
end

function S.Tick()
	local now = GetTime()
	local dt = last and math.min(now - last, MAX_STEP_S) or 0
	last = now
	local sm = sample()
	if sm and dt > 0 then S.Step(S.current, dt, sm) end
end

-- A named ns.Timers handle, so an error in a step reaches ns.Report and the addon's log.
local T = ns.Timers()

ns.RegisterEvent("PLAYER_LOGIN", function()
	if T.active("odometer") then return end
	last = GetTime()
	T.every("odometer", C.odometer_tick_s, S.Tick)
end)

