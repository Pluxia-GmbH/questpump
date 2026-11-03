-- The pump's composition, pure (docs/plan.md slice 2: "The difficulty score and the tiers", "Selection"). The quest
-- decides the composition and the theme (its score picks the tier, its family and stretch tilt the choice); the
-- player's records decide the reps and the holds; cardio seconds come from the quest's length times M, the steady
-- ledger's (slice 52; P until then). Four steps per slot: the hard filters (Progress.lua's ns.Passes, the rung rule,
-- one record move a region among a pump's own moves), the score of every candidate (0.35 due + 0.15 recovered + 0.15
-- variety + 0.30 fit + 0.05 theme), the composition by tier at the player's pace (Light, Balanced or Extreme:
-- C.pace_moves) with a Park-Miller roll over the top three, and the dose.
-- Once a strength or hold ladder has gone major_h without a set, a quest's pump opens on the major move, a record
-- attempt on the one of those that has waited longest by its cadence, on top of the quest's own moves while the quest
-- stays within its pace's most moves, else in place of one of them (never under Light, whose one move the day plan
-- picks). Slice 1's rules hold for any tier: the hub merge (the highest tier's size plus the T1 size per extra quest),
-- every owed move that fits riding along after the pump's own moves, spread so that none follows an effort of its own
-- body part while another can go there (spreadRiders), and the all-out windows always its last efforts: the pump's own,
-- or in its place every owed one that fits, oldest first.
-- Same inputs, same pump.

local ADDON, ns = ...
local C, E, L = ns.C, ns.Enum, ns.L
local K = E.kind

local HOUR, DAY = 3600, 86400
local PRACTICE = 12 -- /qp try: 20 s of wall push-ups, nothing stored

local function clamp(v)
	if v < 0 then return 0 end
	if v > 1 then return 1 end
	return v
end

-- Each move's flags and themes as sets, built once.
local FLAGS, THEMES = {}, {}
for id, m in pairs(ns.Library) do
	FLAGS[id], THEMES[id] = {}, {}
	for _, flag in ipairs(m.flags) do FLAGS[id][flag] = true end
	for _, theme in ipairs(m.theme) do THEMES[id][theme] = true end
end

-- Whether a move carries a flag of the library's vocabulary.
function ns.Flagged(moveId, flag)
	return FLAGS[moveId] ~= nil and FLAGS[moveId][flag] == true
end

-- The dose -------------------------------------------------------------------------------------------------------

-- Cardio is never a record (David, 2026-09-22): a steady move's seconds come from the quest's active minutes, 20-30
-- s, times M, the steady ledger's multiplier for its form in thousandths (ns.SteadyM, 1000-1500: 20-45 s; slice 52, the
-- cardio progression P until then).
function ns.CardioSeconds(activeMinutes, M)
	local base = math.min(C.cardio_cap_s, C.cardio_base_s + C.cardio_per_min_s * (activeMinutes or 0))
	return math.floor((base * (M or C.ledger_m[1]) + 500) / 1000)
end

-- An uncounted all-out window: a 10-s build on the same move, added to its time, then 20 s × M (David, 2026-09-23;
-- × P until slice 52): 30-40 s.
function ns.AlloutSeconds(M)
	return C.allout_build_s + math.floor((C.allout_s * (M or C.ledger_m[1]) + 500) / 1000)
end

-- The steady ledger's M for a form `id` at the compose's time (its form's own rung, ns.SteadyM).
local function formM(ctx, id)
	return ns.SteadyM(ctx.state, ns.Library[id].rung, ctx.now)
end

-- The time estimate (slice 51), its one owner: tools/gen-engine.lua's and tools/simulator.lua's clocks, the
-- expedition's next round and its preview, and the phones' ports read it here. A move's time in a pump, the
-- transition after it included (transition_s): a timed slot's seconds (a cardio move, an all-out window, a pose), else
-- a hold's seconds, else time_rep_s a rep and at least time_move_min_s; the count is `value` where the move has one (a
-- result), else the slot's goal, else a first max set's top (`hi`, `cap`), else none (a hold's least then).
function ns.MoveTime(slot, value)
	local work = slot.seconds
	if not work then
		local n = value or slot.target or slot.hi or slot.cap
		if slot.kind == K.hold then
			work = n or C.time_move_min_s
		else
			work = math.max(C.time_move_min_s, (n or 0) * C.time_rep_s)
		end
	end
	return work + C.transition_s
end

-- A pump's time by its moves' estimates (ns.MoveTime), each after its countdown (countdown_s).
function ns.PumpTime(slots)
	local t = 0
	for _, slot in ipairs(slots) do t = t + C.countdown_s + ns.MoveTime(slot) end
	return t
end

-- The difficulty score and the tiers ------------------------------------------------------------------------------

local LEVEL = { [0] = 0, 0.25, 0.5, 0.75, 1 } -- Enum.RelativeContentDifficulty: Trivial 0 ... Impossible 4
local TAG = { [E.tag.raid] = "tag_raid", [E.tag.dungeon] = "tag_dungeon", [E.tag.legendary] = "tag_legendary",
	[E.tag.elite] = "tag_elite", [E.tag.pvp] = "tag_pvp", [E.tag.event] = "tag_event" }

-- The level colour from the level difference when the server's own rule is unknown (DifficultyUtil's thresholds).
local function levelFromDiff(diff, trivialRange)
	if diff >= 5 then return 1 end
	if diff >= 3 then return 0.75 end
	if diff >= -4 then return 0.5 end
	if diff >= -(trivialRange or 5) then return 0.25 end
	return 0
end

-- The xr a quest's money stands for at its level (the level cap pays no XP): QuestPumpMoneyRef[level] lists each XP
-- tier's money and xr in pairs, ascending, and the money falls between two tiers (interpolated) or beyond the last
-- (in proportion). Money and XP tiers part ways from tier 6 (it pays four times the tier-3 money but 2.5 times its
-- XP), so the money is read through the tier table, never as a plain ratio. nil without a reference for that level.
local function moneyXr(money, level)
	local ref = level and QuestPumpMoneyRef and QuestPumpMoneyRef[level]
	if not ref or #ref < 2 then return nil end
	local lowMoney, lowXr = 0, 0
	for i = 1, #ref - 1, 2 do
		local tierMoney, tierXr = ref[i], ref[i + 1]
		if money <= tierMoney then
			return lowXr + (tierXr - lowXr) * (money - lowMoney) / (tierMoney - lowMoney)
		end
		lowMoney, lowXr = tierMoney, tierXr
	end
	return lowXr * money / lowMoney
end

-- q: { xp, xpMax, xr, money, level, difficulty, levelDiff, trivialRange, tagID, objectives, required, footYards, boss,
-- finale, group (suggested for two or more), trivial }. Returns the score, the tier (1-3) and the five parts. The XP
-- part is the quest table's xr (level-invariant), else the turn-in's XP over the player's level, else (no XP: the
-- level cap, or a quest that pays none; the caller passes money only for a quest the table does not map) the xr its
-- money stands for at the quest's level, interpolated between that level's tiers, else 0.5. A grey quest is capped at
-- 0.25, a single move, never a circuit, even with a dungeon tag; a Dungeon or Raid tag, a boss or a chain finale is
-- otherwise T3.
function ns.Score(q)
	local p = {}
	local fromMoney = (q.money or 0) > 0 and moneyXr(q.money, q.level)
	if q.xr then
		p.xp = clamp(C.xr_scale * q.xr)
	elseif (q.xp or 0) > 0 and (q.xpMax or 0) > 0 then
		p.xp = clamp(q.xp / q.xpMax / C.xp_norm)
	elseif fromMoney then
		p.xp = clamp(C.xr_scale * fromMoney)
	else
		p.xp = 0.5
	end
	if q.difficulty and LEVEL[q.difficulty] then
		p.lvl = LEVEL[q.difficulty]
	elseif q.levelDiff then
		p.lvl = levelFromDiff(q.levelDiff, q.trivialRange)
	else
		p.lvl = 0.5
	end
	p.tag = TAG[q.tagID] and C[TAG[q.tagID]] or 0
	if q.objectives then
		p.obj = 0.5 * clamp((q.objectives - 1) / C.obj_count_norm) + 0.5 * clamp((q.required or 0) / C.obj_req_norm)
	else
		p.obj = C.obj_prior
	end
	p.trav = clamp((q.footYards or 0) / C.trav_norm_yd)
	local score = C.w_xp * p.xp + C.w_lvl * p.lvl + C.w_tag * p.tag + C.w_obj * p.obj + C.w_trav * p.trav
		+ (q.boss and C.bonus_boss or 0) + (q.finale and C.bonus_finale or 0)
		+ (q.group and C.bonus_group or 0)
	local tier
	if q.trivial then
		score = math.min(score, C.trivial_cap)
		tier = 1
	elseif score >= C.tier_t3 or q.tagID == E.tag.dungeon or q.tagID == E.tag.raid or q.boss or q.finale then
		tier = 3
	elseif score >= C.tier_t2 then
		tier = 2
	else
		tier = 1
	end
	return score, tier, p
end

-- The player's pace (General > Moves per quest, profile.pace; David, 2026-10-05): a key of C.pace_moves, the default
-- (Balanced) for none or one this addon does not know (a newer addon's, through the mirror or an import).
function ns.Pace(key)
	return C.pace_moves[key] and key or ns.defaults.pace
end

-- The families a quest can have (the quest table's closed list, slice 4).
ns.FAMILIES = { "kill", "gather", "travel", "talk", "escort", "boss", "dungeon", "pvp", "profession", "fishing",
	"repeatable" }
local FAMILY = {}
for _, f in ipairs(ns.FAMILIES) do FAMILY[f] = true end

-- The quest's kind: the quest table's family when it maps the quest (info.mapped, from ns.Mapping), else what the
-- client says about it.
function ns.Family(info)
	info = info or {}
	local types = info.types or {}
	if info.mapped and FAMILY[info.mapped.f] then return info.mapped.f end
	if info.tagID == E.tag.dungeon or info.tagID == E.tag.raid then return "dungeon" end
	if info.tagID == E.tag.elite and info.objectives == 1 and info.singleKill then return "boss" end
	if info.tagID == E.tag.pvp or types.player then return "pvp" end
	if types.monster then return "kill" end
	if types.item or types.object then return "gather" end
	if info.footYards and info.footYards > C.travel_yd then return "travel" end
	if info.repeatable and not next(types) then return "repeatable" end
	return "talk"
end

-- The quest table (Quests.lua, generated by tools/quests, slice 4): one string per quest,
-- "family|level|tag|xr|flags|themes", decoded here when a quest needs it. Flags: b a boss, f a chain finale, g a group
-- quest; themes: up to two of ns.THEMES, comma-separated; an empty level or xr is unknown. Returns nil for a quest the
-- table does not hold (or when Quests.lua is missing: the addon runs without it); the table it returns is shared,
-- read and never written.
local decoded, decodedLines = {}, {} -- entry string → its table (the same string decodes the same way)

local function decode(s)
	local f, lvl, tag, xr, flags, th = s:match("^(%a+)|(%d*)|(%d*)|([%d%.]*)|(%a*)|([%a,]*)$")
	if not f then return nil end
	local themes = {}
	for theme in th:gmatch("%a+") do themes[#themes + 1] = theme end
	return { f = f, lvl = tonumber(lvl), tag = tonumber(tag) or 0, xr = tonumber(xr), boss = flags:find("b") ~= nil,
		fin = flags:find("f") ~= nil, group = flags:find("g") ~= nil, th = themes }
end

function ns.Mapping(questID)
	local s = QuestPumpQuests and QuestPumpQuests[questID]
	if type(s) ~= "string" then return nil end
	if decoded[s] == nil then decoded[s] = decode(s) or false end
	return decoded[s] or nil
end

-- The kinds of move a quest's four flavour lines are written for, in their order.
ns.LINE_KINDS = { "rep", "hold", "cardio", "flow" }

-- A quest's flavour lines (QuestPump_Lines/Lines.lua, the load-on-demand folder the same tool writes, 2 MB too many
-- for Quests.lua): "reps line|hold line|cardio line|flow line", one per kind of move, because the move is chosen at
-- the turn-in, after the lines were written. Returns them by the move's kind code (ns.Enum.kind), or nil when the
-- folder is not loaded or holds no lines for the quest; the table is shared, read and never written.
local function decodeLines(s)
	local c = { s:match("^([^|]*)|([^|]*)|([^|]*)|([^|]*)$") }
	if not c[1] then return nil end
	local lines = {}
	for i, kind in ipairs(ns.LINE_KINDS) do
		if c[i] ~= "" then lines[K[kind]] = c[i] end
	end
	return lines
end

function ns.Lines(questID)
	local s = QuestPumpLines and QuestPumpLines[questID]
	if type(s) ~= "string" then return nil end
	if decodedLines[s] == nil then decodedLines[s] = decodeLines(s) or false end
	return decodedLines[s] or nil
end

-- The patterns each family favours, in order: 1.0 for the first place, 0.6 the second, 0.3 the third and later (the
-- fit prior); the stretch adds the travel and the swim rows by its shares.
local FIT = {
	kill = { { "strike" }, { "ballistic" }, { "squat", "lunge" }, { "push" },
		{ "core_anti_ext", "core_anti_rot", "core_flex" } },
	gather = { { "hinge" }, { "squat" }, { "lunge", "calf", "carry" } },
	travel = { { "run" }, { "cardio" }, { "ballistic", "carry", "calf" } },
	swim = { { "pull" }, { "core_anti_ext", "core_anti_rot", "core_flex" }, { "cardio", "run" } },
	talk = { { "mobility" }, { "balance" }, { "iso" } },
	escort = { { "iso" }, { "balance" }, { "core_anti_ext", "core_anti_rot", "core_flex" } },
	boss = { { "ballistic" }, { "push" }, { "cardio", "run" } },
	dungeon = { { "squat", "lunge", "hinge" }, { "push", "pull" }, { "ballistic", "run", "cardio" } },
	pvp = { { "strike" }, { "ballistic" }, { "cardio", "lunge" } },
	profession = { { "carry" }, { "pull" }, { "hinge", "push" } },
	fishing = { { "balance" }, { "hinge" }, { "calf", "mobility" } },
}
ns.FIT = FIT

-- The quest's activity vector dotted with the families' pattern priors: per pattern, 0 to 1. The rows add in one
-- fixed order (the family, then travel, then swim), so the sums carry the same last bit on every Lua (a `pairs`
-- order would not, and a near-tie would then sort differently in CI).
function ns.FitVector(q)
	local family = q and q.family
	local share = q and q.share or {}
	local rows = {}
	if family and FIT[family] then rows[#rows + 1] = { family, 1 } end
	local travel = clamp(((share.foot or 0) + (share.mount or 0)) / C.trav_norm_yd)
	local swim = clamp((share.swim or 0) / C.trav_norm_yd)
	if travel > 0 and family ~= "travel" then rows[#rows + 1] = { "travel", travel } end
	if swim > 0 then rows[#rows + 1] = { "swim", swim } end
	local total, fit = 0, {}
	for _, row in ipairs(rows) do total = total + row[2] end
	if total == 0 then return fit end
	for _, row in ipairs(rows) do
		for i, group in ipairs(FIT[row[1]]) do
			local prior = C.fit_prior[math.min(i, #C.fit_prior)]
			for _, pattern in ipairs(group) do fit[pattern] = (fit[pattern] or 0) + row[2] / total * prior end
		end
	end
	return fit
end

-- The context --------------------------------------------------------------------------------------------------------

local function within(ts, days, now)
	return ts ~= nil and now - ts < days * DAY
end

-- Within `days` of character creation or of the first pump after a 14-day break; before the first move too, and on the
-- pump that ends such a break (the fold dates the break only from the row that ends it).
local function sinceStart(state, days, now)
	local fresh = state.createdTs == nil
		or (state.lastDoneTs ~= nil and now - state.lastDoneTs >= C.placement_idle_days * DAY)
	return fresh or within(state.createdTs, days, now) or within(state.breakTs, days, now)
end

-- Whether the player stepped down from a move (ns.SteppedDown: never done, its named regression leading to a move an
-- easier set marked), once a compose. No pump offers it until the mark clears: the candidates, the rung move (so the
-- Trial and the placement), Swap's backup and easier version. An owed one still rides (Swap's easier version pays it).
local function steppedDown(ctx, id)
	local known = ctx.down[id]
	if known == nil then
		known = ns.SteppedDown(ctx.state, id)
		ctx.down[id] = known
	end
	return known
end

-- Whether a move is open (ns.MoveOpen, 2026-10-16, slice 42: its checks done and its stage of a step's path open), once
-- a compose. No pump offers one that is not, Swaps to it or eases into it: the candidates, the rung move, the backup,
-- the easier version. An owed one still rides (it was offered).
local function isOpen(ctx, id)
	local known = ctx.open[id]
	if known == nil then
		known = ns.MoveOpen(ctx.state, id, ctx.f) == true
		ctx.open[id] = known
	end
	return known
end

-- The move an owed rep move or hold rides as where it is a backup of its rung's chain the setup leaves unneeded, the
-- move belowBackup keeps out of a fresh pick: the move that stands for that rung with this setup (ns.RungMove with the
-- rung moves' own held rule), which pays its owed row as Swap's backup does (swapSlot; David, 2026-10-29, after the
-- walkout on a bed, offered as a new move once the walkout was at its top, was banked: "can you fix my banked
-- excercises then"). Nil where it rides as itself: a move that waits (riderFits), a loaded move, a move the player
-- stepped down to or from or one not open (an owed one still rides as it was offered), a family not open, a stand-in of
-- another kind, and one the arms rest leaves out where the owed move is not (ns.ArmsHeld: an owed move that may ride a
-- match pump under its arms rest never rides as one the rest holds).
local riderFits -- below (Owed moves)

local function ridesAs(ctx, r)
	local m, l = ns.Library[r.move], r.ladder
	if not (m and (r.kind == K.rep or r.kind == K.hold) and ns.STRENGTH[l] and ns.Open(ctx.state, l)) then return nil end
	local rec = ctx.state.moves[r.move]
	if ns.Loaded(r.move) or (rec and rec.easier) or ctx.held(r.move) or not ns.InChain(l, m.rung, r.move)
		or not riderFits(ctx, r) then
		return nil
	end
	local as = ns.RungMove(l, m.rung, ctx.f, ctx.held)
	if not as or as == r.move or K[ns.Library[as].dose] ~= r.kind or (ns.ArmsHeld(as) and not ns.ArmsHeld(r.move)) then
		return nil
	end
	return as
end

-- Each road's clock (slice 54, 2026-10-23): the latest row of its forms (the fold's formTs, whose flow moves carry no
-- road), 0 for a road never run, and the week's omen's road (`omen`) the longest ago of all, so the fair draw takes it
-- first where the rules leave it.
local function roadClocks(state, omen)
	local clocks = {}
	for id, ts in pairs(state.formTs) do
		local road = ns.Library[id] and ns.Library[id].road
		if road and ts > (clocks[road] or 0) then clocks[road] = ts end
	end
	clocks[omen.road] = -1
	return clocks
end

local function context(input)
	local state = input.state or ns.NewState(nil)
	local now = input.now or ns.now()
	-- the arms rest (input.arms, slice 48: a match pump while its aim game still runs) filters as pain flags do, before
	-- the rung moves below read the filters; the fold never sees it (ns.ArmsRest)
	local f = ns.FilterContext(input.profile, state)
	if input.arms == true then f = ns.ArmsRest(f) end
	local ctx = { state = state, now = now, day = ns.day(now), f = f,
		setup = ns.FilterContext(input.profile, nil), -- the setup alone, which names a check's pose (ns.NeedAt)
		rung = {}, rungMove = {}, belowMove = {}, down = {}, open = {}, why = {},
		used = { moves = {}, regions = {}, families = {}, placement = false }, marked = {},
		-- slice 48's match pump: its first move warms up (input.warm) and it keeps the arms rest (input.arms)
		warm = input.warm == true, arms = input.arms == true }
	-- an expedition's round (slice 51): the regions its earlier rounds' own record moves took stay taken, so one record
	-- move a region across the expedition (ns.RoundRegions)
	for _, region in ipairs(input.expedition and input.expedition.regions or {}) do ctx.used.regions[region] = true end
	ctx.cardioRung = ns.Rung(state, ns.CARDIO)
	-- the week's omen on the cardio rung (slice 54, ns.Omen) and each road's clock for the fair draw (fairDraw)
	ctx.omen = ns.Omen(now, f, ctx.cardioRung)
	ctx.roadTs = roadClocks(state, ctx.omen)
	-- the pace the pump keeps (input.pace, Events.lua), else the profile's
	ctx.pace = ns.Pace(input.pace or (input.profile and input.profile.pace))
	-- a family that is not open yet (ns.Open, slice 22: the handstand and the planche before the push-up is done, power
	-- before the jumping jacks) has no rung move here, so nothing offers it: the candidates, the major move, the plan,
	-- the Trial. A rung move the player stepped down from, or one that waits for its check (isOpen: the explosive
	-- push-up's 12 feet-up push-ups), gives way to the next of its chain (ns.RungMove's `stepped`); with none left the
	-- ladder has no rung move (a family not placed whose first move is held stays shut); ridesAs reads the same rule
	ctx.held = function(id)
		return steppedDown(ctx, id) or not isOpen(ctx, id)
	end
	for l in pairs(ns.Ladders) do
		ctx.rung[l] = ns.Rung(state, l)
		local open = l ~= ns.CARDIO and ns.Open(state, l)
		ctx.rungMove[l] = open and ns.RungMove(l, ctx.rung[l], ctx.f, ctx.held) or nil
		-- and the one that stands for the rung below by the same rule, which keeps its backups out (belowBackup)
		ctx.belowMove[l] = open and ctx.rung[l] > 1 and ns.RungMove(l, ctx.rung[l] - 1, ctx.f, ctx.held) or nil
	end
	ctx.novelty = sinceStart(state, C.novelty_quiet_days, now)
	-- the moves of the last five pumps, the patterns of the last two, and those in each of the last two (a third
	-- pump in a row would run them three times)
	ctx.recentMoves, ctx.recentPatterns, ctx.lastPatterns = {}, {}, {}
	local recent = state.recent or {}
	local n = #recent
	for i = math.max(1, n - C.variety_recent_pumps + 1), n do
		for id in pairs(recent[i].moves) do ctx.recentMoves[id] = true end
	end
	for i = math.max(1, n - C.variety_pattern_pumps + 1), n do
		for pattern in pairs(recent[i].patterns) do ctx.recentPatterns[pattern] = true end
	end
	if n >= C.variety_pattern_pumps then
		for pattern in pairs(recent[n].patterns) do
			local inAll = true
			for i = n - C.variety_pattern_pumps + 1, n - 1 do inAll = inAll and recent[i].patterns[pattern] == true end
			if inAll then ctx.lastPatterns[pattern] = true end
		end
	end
	-- the pump's seed: the quest, the date and the pump number of the day: the game's pumps for a game pump, every
	-- source's for an app pump (input.app: tools/apppump.lua's, as the phones count them), so the same rows roll the same
	-- pump whichever device stamped them
	local counts = state.days and state.days[ctx.day]
	ctx.pumpNo = (counts and counts.pumps or 0) + (input.app and counts and counts.appPumps or 0) + 1
	-- the day plan: the moves each block took this game day and the one before (two days, so a short evening's few
	-- own moves still reach every block: with one day, a quiet evening under Endurance gave yoga 1-5 %; slice 20), the
	-- pump's own added as it takes them; the sets each move had today, for the rotation; the focus's shares
	local before = state.days and state.days[ns.day(ns.DayMid(now, -1))]
	ctx.blocks, ctx.done, ctx.shares = {}, counts and counts.done or {}, C.focus_share[ns.Focus(state)]
	for _, block in ipairs(ns.BLOCKS) do
		ctx.blocks[block] = (counts and counts.blocks and counts.blocks[block] or 0)
			+ (before and before.blocks and before.blocks[block] or 0)
	end
	-- the moves the queue holds, as taken (David, 2026-10-04: an hour in a mixed party owed the standing quad stretch
	-- on 13 of 14 turn-ins and a record attempt on every one, because an owed move is never performed and the fold
	-- moves no clock for it): a fresh pick never takes one, its owed copy rides instead (so an owed pump never stacks a
	-- second copy, nor the move an owed backup rides as: ridesAs, marked with that row's key, so a move started from a
	-- step's card that pays the row as itself still takes it as Swap's backup, backupMove); one owed this game day or
	-- the one before counts in its block's share (once: take skips it when it rides); one on a strength ladder or a pose
	-- family is that ladder's or family's turn (lastTurn)
	ctx.owed, ctx.turn, ctx.owedCounted = {}, {}, {}
	local yesterday = ns.day(ns.DayMid(now, -1))
	for _, r in ipairs(ns.OwedList(state)) do
		ctx.owed[r.move] = true
		local as = ridesAs(ctx, r)
		if as and not ctx.owed[as] then ctx.owed[as] = r.key end
		local day = ns.day(r.ts)
		if ns.Library[r.move] and (day == ctx.day or day == yesterday) then
			local block = ns.Block(r.move)
			ctx.blocks[block] = ctx.blocks[block] + 1
			ctx.owedCounted[r.key] = true
		end
		local record = r.kind == K.rep or r.kind == K.hold
		if (record and ns.STRENGTH[r.ladder]) or (r.kind == K.flow and ns.POSE[r.ladder]) then
			ctx.turn[r.ladder] = math.max(ctx.turn[r.ladder] or 0, r.ts)
		end
	end
	return ctx
end

local function hoursSince(ts, now)
	return ts and (now - ts) / HOUR or nil
end

-- due: hours since the ladder's last anchor attempt over its cadence, capped at 1; a ladder never attempted is 1.
-- Two more values order the ladders that are all due: the hours waited (the longest wait gets its record attempt, so
-- a 72-h ladder is not starved by the 48-h ones sharing its region) and the cadences waited (a ladder 1.5 cadences
-- overdue is forced).
local function ladderDue(ctx, l)
	local ts
	if l == ns.CARDIO then
		ts = ctx.state.cardio and ctx.state.cardio.lastTs
	else
		-- the anchor's own last set while the anchor is the rung's move, so a stand-in's set never holds the anchor back
		-- (slice 52, 2026-10-23: a desk plank's set held the long-lever plank back from 10-14 on); while the filters, a
		-- step down or a check leave another move in its place, that move's set counts too (`standTs`, DB.lua
		-- anchorAttempt)
		local lad = ctx.state.ladders[l]
		ts = lad and lad.anchorTs
		if lad and lad.standTs and ctx.rungMove[l] ~= ns.Anchor(l, ctx.rung[l]) and lad.standTs > (ts or 0) then
			ts = lad.standTs
		end
	end
	local h = hoursSince(ts, ctx.now)
	if not h then return 1, math.huge, math.huge end
	local cadence = C.cadence_h[l]
	return math.min(1, h / cadence), h, h / cadence
end

-- Moves outside the ladders (the flow moves, and the calf, carry and ballistic moves no ladder holds): days since
-- their pattern was used, capped at 1 and scaled by flow_due_scale, so they surface about daily and first after a
-- heavy day.
local function otherDue(ctx, m)
	local p = ctx.state.patterns[m.pattern]
	local h = hoursSince(p and p.lastTs, ctx.now)
	return (h and math.min(1, h / 24) or 1) * C.flow_due_scale
end

-- The load a pattern carries now: every hard set adds its class, decaying with its half-life (summed in a fixed order
-- of half-lives, for the same reason as the fit).
local HALF_LIVES = {}
for _, hl in pairs(C.half_life_h) do HALF_LIVES[#HALF_LIVES + 1] = hl end
table.sort(HALF_LIVES)
for i = #HALF_LIVES, 2, -1 do
	if HALF_LIVES[i] == HALF_LIVES[i - 1] then table.remove(HALF_LIVES, i) end
end

function ns.Load(state, pattern, now)
	local p = state.patterns[pattern]
	if not p or not p.acc then return 0 end
	local load = 0
	for _, hl in ipairs(HALF_LIVES) do
		local a = p.acc[hl]
		if a then load = load + a.v * 0.5 ^ ((now - a.ts) / (hl * HOUR)) end
	end
	return load
end

-- Candidates ---------------------------------------------------------------------------------------------------------

local function addCandidate(list, id, t)
	t.move = id
	list[#list + 1] = t
end

-- A jump that names a cardio rung (`crung`: the power family's squat jump, frog jumps, strict burpee, tuck jump and
-- jumping muscle-up need cardio rung 5) waits for it, on its ladder as off the ladders (slice 22).
local function jumpReady(ctx, m)
	return not m.crung or ctx.cardioRung >= m.crung
end

-- A backup of the rung below: a move of that rung's chain other than the one that stands for it with this setup
-- (ctx.belowMove), as the walkout on a bed is while the floor's walkout is allowed. The rung below offers what it
-- offered while the player stood on it, whose variants leave the chain out (ns.InChain) as the player's rung's do, so
-- the same move on another surface never comes back as a new move with a first max set (David, 2026-10-29, after the
-- walkout on the bed came once the walkout was at its top: "Why is that a new exercise?"). A loaded move stays, another
-- exercise whose progress is the weight (the deadlift with a weight beside the floor's single-leg bridge), and so does
-- a move the player stepped down to, until a set clears its mark.
local function belowBackup(ctx, l, rung, id)
	local rec = ctx.state.moves[id]
	return id ~= ctx.belowMove[l] and not ns.Loaded(id) and ns.InChain(l, rung - 1, id) and not (rec and rec.easier)
end

-- Every move that passed the hard filters and the rung rule and that the player has not stepped down from
-- (steppedDown), with its ladder, rung and due. A strength ladder offers its anchor (or its substitute) in every pump
-- (David, 2026-10-02: the major moves "need to come regularly with priority"; no rest after a record), and once it is
-- placed its variants at the rung beside it and the moves of the rung below whose record is under their top (a ladder
-- that is not placed or is placing again offers the anchor alone: its max set places it), and placed or not a move the
-- player stepped down to; due only orders them. Once the anchor's days are in toward a rung whose climb waits for its
-- path, the move of the path's first stage still out nearest its days (ns.PathTurn, among those the setup offers that
-- are open, at its own rung) takes the ladder's record turn (`turn`) and the anchor waits, its sets adding nothing to
-- the climb (2026-10-09: the push's proof, the archer's twin, came only through Swap until then; the path in its place
-- since 2026-10-16); where every move of it the setup offers waits for another ladder's record (ns.PathHeld), that
-- record's move takes the turn wherever the pumps offer it (2026-10-17). Either turn yields for the rest of the game
-- day once its move came that day, as a pose family's and cardio's do (2026-10-31: due, overdue and waited 0, or the
-- move it held; a path move's set moves no anchor clock, so an overdue turn was forced in every pump of the day until
-- then). A move that is not open (ns.MoveOpen: a "then" move before the move before it reached its top, a move whose
-- check is not done) is never offered (2026-10-16, slice 42). The cardio ladder offers the forms of the cardio rung and
-- the two below it, run at the rung's cadence (from rung 7 the steady forms of rungs 4-6 at rung 6's: slice 52,
-- 2026-10-23; of rung 3 and below until then). A move whose better tool's move is among the candidates gives it its
-- place (ns.Better, 2026-10-09: the backpack squat for the goblet squat, the band curl for the hammer curl); an owed
-- one is still paid as itself. The rung below's backups never come (belowBackup).
local function candidates(ctx)
	local list, state, f = {}, ctx.state, ctx.f
	local held = {} -- the paths another ladder's record holds (below)
	for _, l in ipairs(ns.STRENGTH_ORDER) do -- in order, as the phone cores walk them
		local rung = ctx.rung[l]
		local due, overdue, waited = ladderDue(ctx, l)
		local placing = ns.Placing(state, l, ctx.now)
		-- a new rung's anchor, never done, is due from the next game day after the rung-up
		local lad = state.ladders[l]
		local id0 = ctx.rungMove[l]
		local opened = lad and lad.opened and ns.day(lad.opened) ~= ctx.day and id0 and not state.moves[id0]
		if opened and due < 1 then due, overdue, waited = 1, hoursSince(lad.opened, ctx.now), 0 end
		local variants = lad and lad.placedTs and not placing
		local turn, holder
		if variants and lad.hiDays >= ns.UnlockDays(l, rung, state) and ns.CanClimb(l, rung, f, state) then
			-- never one the player stepped down from: it would take the anchor's turn and then be dropped
			turn = ns.PathTurn(state, l, f, rung)
			-- none can, every move of the path the setup offers waiting for a check: one of another ladder's record
			-- (power step 6's knee clap push-up, 12 feet-up push-ups) gives the turn to that record's move below, where
			-- the pumps offer it (2026-10-17); a stretch check's pose takes a stretch slot (checkSlot)
			local waits = not turn and select(2, ns.PathHeld(state, l, f, rung))
			local need = waits and waits.kind == "check" and ns.EntryFor(waits.need, f)
			if need and not ns.IsCheck(need) then holder = ns.NeedAt(need, f) end
		end
		if id0 and jumpReady(ctx, ns.Library[id0]) then
			local anchor = { ladder = l, rung = rung, due = turn and 0 or due, anchor = true, placing = placing,
				overdue = overdue, waited = turn and 0 or waited }
			addCandidate(list, id0, anchor)
			if holder then held[#held + 1] = { move = holder, anchor = anchor, due = due, overdue = overdue } end
		end
		if variants then
			for id, m in pairs(ns.Library) do
				if m.ladder == l and m.rung == rung and not ns.InChain(l, rung, id) and ns.Passes(id, f)
					and jumpReady(ctx, m) then
					addCandidate(list, id, { ladder = l, rung = rung, due = due })
				end
			end
			-- the rung below too, each move on its own record and dosed at its own rung, so a long day's later sets
			-- rotate through the player's level and the one under it (slice 20: "variety that stays hard"); they
			-- never stand for the rung, so only the anchor's sets unlock the next one. One whose record holds its top
			-- leaves, its turns the anchor's and the variants' (ns.AtTop; David, 2026-10-14: "Whats the top of shallow
			-- and should then when reached not exclusive come quarter wall sit next?"), but a loaded move (its progress
			-- is the weight), the climb's path and a move the player stepped down to (until a set clears it) stay
			for id, m in pairs(ns.Library) do
				if m.ladder == l and m.rung == rung - 1 and id ~= id0 and ns.Passes(id, f) and jumpReady(ctx, m)
					and (id == turn or ns.Loaded(id) or not ns.AtTop(state, id) or state.moves[id].easier)
					and not belowBackup(ctx, l, rung, id) then
					addCandidate(list, id, { ladder = l, rung = rung - 1, due = due })
				end
			end
		end
		if turn then -- listed already as a variant or a move of the rung below, it takes the turn there
			local c
			for _, x in ipairs(list) do
				if x.move == turn and x.ladder == l then c = x end
			end
			if not c then
				c = { ladder = l, rung = ns.Library[turn].rung }
				addCandidate(list, turn, c)
			end
			-- once it came this game day it yields for the rest of it (2026-10-31): a path move's set moves no anchorTs
			-- (ladderDue), so an overdue turn was forced in every pump of the day
			if (ctx.done[turn] or 0) > 0 then
				c.due, c.turn, c.overdue, c.waited = 0, true, 0, 0
			else
				c.due, c.turn, c.overdue, c.waited = due, true, overdue, waited
			end
		end
		-- a move the player stepped down to (the fold's `easier` mark: an easier set) comes at its own rung, on its own
		-- record, until a set reaches the mark's count and clears it (DB.lua easierMark), which gives back the moves it
		-- holds (ns.SteppedDown; 2026-10-05: the air squat a squat jump stepped down to, two rungs under a squat ladder on
		-- the split squat, came no more, so the squat jump never came back); never above the rung rule, in a family that
		-- is not open, or where it would stand for the player's rung (a twin of its chain, which Swap's backup offers)
		for id, m in pairs(ns.Library) do
			local rec = m.ladder == l and id ~= id0 and id ~= turn and state.moves[id]
			if rec and rec.easier and m.rung <= rung + ns.Reach(l) and not (m.rung == rung and ns.InChain(l, rung, id))
				and not (variants and (m.rung == rung or m.rung == rung - 1)) and ns.Open(state, l)
				and ns.Passes(id, f) and jumpReady(ctx, m) then
				addCandidate(list, id, { ladder = l, rung = m.rung, due = due })
			end
		end
	end
	-- the pose families (slice 21) as the strength ladders: the pose that stands for the rung (its anchor, or a twin
	-- the filters allow; a stand-in from below where the setup shuts the rung), the rung's other poses and the poses of
	-- the step below, never a pose above the player's step. The family's mark comes once a game day (slice 47,
	-- 2026-10-19: a goal pose of its step, ns.Marks; its main pose alone from slice 21 until then): the pose due for its
	-- first turn of the day (ns.DuePose: the main pose while its own days would fall short, else the step's goal pose
	-- asked longest ago, on the top step held longest ago) is the family's anchor, due until the family's mark came
	-- that game day and only while its answer can count (ns.DayIn nil; 2026-10-20, slice 47's fix round), and the
	-- step's other poses join as one more pose of the family (a stand-in from below, which names no goal pose of the
	-- step, one more pose of its own step). Once the step's days are in (ns.PoseDaysIn) toward a rung whose climb waits
	-- for its path (ns.PathTurn, 2026-10-12; the path since 2026-10-16), the pose of its first stage still out takes the
	-- family's turn (`turn`), due, and forced once overdue, while it has not come that game day (its overdue and waited
	-- kept it forced after it came until 2026-10-31), no pose is due for the mark and the main pose waits. The step
	-- below's backups never come (belowBackup).
	for _, l in ipairs(ns.POSE_ORDER) do
		local rung = ctx.rung[l]
		local id0 = ctx.rungMove[l]
		local due, overdue, waited = ladderDue(ctx, l)
		local turn
		if id0 and ns.MainPose(l, rung, id0) and ns.PoseDaysIn(state, l, rung) and ns.CanRungUp(l, rung, f, state) then
			turn = ns.PathTurn(state, l, f, rung)
		end
		local mark = not turn and ns.DuePose(state, l, f, ctx.now)
		if mark then
			addCandidate(list, mark, { ladder = l, rung = rung, due = 1, anchor = true, overdue = overdue,
				waited = waited })
		end
		if id0 and id0 ~= mark then
			addCandidate(list, id0, { ladder = l, rung = ns.Library[id0].rung, due = turn and 0 or due })
		end
		for id, m in pairs(ns.Library) do
			if m.ladder == l and id ~= id0 and id ~= mark
				and ((m.rung == rung and not ns.InChain(l, rung, id))
					or (m.rung == rung - 1 and not belowBackup(ctx, l, rung, id))) and ns.Passes(id, f) then
				addCandidate(list, id, { ladder = l, rung = m.rung, due = due })
			end
		end
		if turn then -- listed already as a pose of the rung or the one below, it takes the turn there
			local c
			for _, x in ipairs(list) do
				if x.move == turn and x.ladder == l then c = x end
			end
			if not c then
				c = { ladder = l, rung = ns.Library[turn].rung }
				addCandidate(list, turn, c)
			end
			if (ctx.done[turn] or 0) > 0 then
				c.due, c.turn, c.overdue, c.waited = 0, true, 0, 0
			else
				c.due, c.turn, c.overdue, c.waited = 1, true, overdue, waited
			end
		end
	end
	-- the steady pool (ns.SteadyForms), from rung 7 (whose own forms are all-out alone) rung 6's
	local cr = ctx.cardioRung
	local due = ladderDue(ctx, ns.CARDIO)
	for _, id in ipairs(ns.SteadyForms(cr, f)) do
		addCandidate(list, id, { ladder = ns.CARDIO, rung = cr, due = due })
	end
	for id, m in pairs(ns.Library) do
		if not m.ladder and ns.Passes(id, f) then
			local scale = ns.ScaleLadder(m)
			local ok = not scale or m.rung <= ctx.rung[scale] + ns.Reach(scale)
			if ok and jumpReady(ctx, m) then
				addCandidate(list, id, { ladder = nil, rung = m.rung, due = otherDue(ctx, m) })
			end
		end
	end
	-- a path another ladder's record holds: that record's move takes the ladder's turn where it is offered, until it
	-- came that game day (then it keeps only its own due, 2026-10-31), and the anchor waits, its sets adding nothing to
	-- the climb (as for a path's own turn)
	for _, h in ipairs(held) do
		for _, x in ipairs(list) do
			if x.move == h.move then
				if (ctx.done[h.move] or 0) == 0 then
					x.turn, x.due = true, math.max(x.due, h.due)
					x.overdue, x.waited = x.overdue or h.overdue, x.waited or h.anchor.waited
				end
				h.anchor.due, h.anchor.waited = 0, 0
				break
			end
		end
	end
	-- and never a move the player stepped down from nor one not open (its rung move is left out in the context already)
	local kept, pool = {}, {}
	for _, c in ipairs(list) do
		if not steppedDown(ctx, c.move) and isOpen(ctx, c.move) then
			kept[#kept + 1] = c
			pool[c.move] = true
		end
	end
	local better = {}
	for _, c in ipairs(kept) do
		if not ns.Better(c.move, pool) then better[#better + 1] = c end
	end
	return better
end

local function recordMove(m)
	return m.dose == "rep" or m.dose == "hold"
end

-- One record move a region among a pump's own moves (an owed rider holds none: take), the one rule between record
-- moves: a second max set on muscles the pump has just taken to their limit would only miss its record. Nothing waits
-- after a record and nothing holds a new player
-- back (David, 2026-10-02: "remove any restrictions after doing a record. Pushups should be often and not limited just
-- because you did a record or are new").
local function regionTaken(ctx, id)
	local m = ns.Library[id]
	return recordMove(m) and m.region ~= nil and ctx.used.regions[m.region] == true
end

local function firstEver(ctx, id)
	return not ctx.state.seen[id]
end

-- A strength or hold ladder whose anchor attempt now is a placement: the Trial's first max set on a ladder never
-- placed, or a break's; a pump carries one (plan convention 3), whichever way it comes (a rider, the Trial, a pick).
local function placementLadder(ctx, l)
	if not ns.STRENGTH[l] then return false end
	local lad = ctx.state.ladders[l]
	return not (lad and lad.placedTs) or ns.Placing(ctx.state, l, ctx.now)
end

-- Why a pick leaves a candidate out of this pump: the move, or one of the pump's own record moves of its region, is in
-- it already ("pump"), or it would be the pump's second placement ("placement"); nil when the pump can take it.
local function leftOut(ctx, c)
	if ctx.used.moves[c.move] or regionTaken(ctx, c.move) then return "pump" end
	if c.anchor and ctx.used.placement and placementLadder(ctx, c.ladder) then return "placement" end
	return nil
end

-- The five scores of a candidate for one quest's fit and theme.
local function score(ctx, c, fit, themes)
	local m = ns.Library[c.move]
	local recovered = clamp(1 - ns.Load(ctx.state, m.pattern, ctx.now) / C.load_cap)
	local p = ctx.state.patterns[m.pattern]
	local usedToday = p and p.days and p.days[ctx.day] or 0
	local variety = usedToday == 0 and 1 or C.variety_once
	if ctx.recentMoves[c.move] or ctx.recentPatterns[m.pattern] then variety = 0 end
	if firstEver(ctx, c.move) and not ctx.novelty then variety = math.min(1, variety + C.novelty) end
	local theme = 0
	for _, tag in ipairs(themes or {}) do -- the quest table's theme list (ns.Mapping's th)
		if THEMES[c.move][tag] then theme = 1 end
	end
	local parts = { due = c.due, recovered = recovered, variety = variety, fit = fit[m.pattern] or 0, theme = theme }
	c.parts = parts
	c.score = C.w_due * parts.due + C.w_recovered * parts.recovered + C.w_variety * parts.variety
		+ C.w_fit * parts.fit + C.w_theme * parts.theme
	c.lastTs = ns.DoneAt(ctx.state, c.move)
	return c.score
end

-- Keeps the candidates a rule allows, unless that leaves none (moves repeat the patterns left, the count stays).
local function prefer(list, keep)
	local out = {}
	for _, c in ipairs(list) do
		if keep(c) then out[#out + 1] = c end
	end
	return #out > 0 and out or list
end

local function byScore(a, b)
	if a.score ~= b.score then return a.score > b.score end
	local la, lb = a.ladder or 99, b.ladder or 99
	if la ~= lb then return la < lb end
	-- the move done longest ago first (ns.DoneAt: a record's last set, a pose's last ask or whole hold, the last row of
	-- a cardio form or of a flow move outside a pose family), so moves that tie on every score take turns across the
	-- days too (the lowest id took every tie a day's rotation left: slice 22's new moves have the highest ids, and the
	-- push proof came almost never; a pose, a stretch, a cardio form or a carry, which keep no record, until
	-- 2026-10-31), then the lowest id
	if a.lastTs ~= b.lastTs then return a.lastTs < b.lastTs end
	return a.move < b.move
end

-- A match pump's warm move's order (pick's `warm`): the move done longest ago first (ns.DoneAt), then byScore.
local function warmOrder(a, b)
	if a.lastTs ~= b.lastTs then return a.lastTs < b.lastTs end
	return byScore(a, b)
end

-- The fair draw (slice 54, 2026-10-23), the cardio picks' last word before the score, steady and window alike: among
-- the forms the rules, the rotation and the favourites leave, the road done longest ago first (its clock the latest
-- row of its forms, the week's omen's road the longest ago of all: roadClocks), then the form done longest ago (its
-- formTs, 0 never run), so the forms of a step take turns road by road and no one form takes every window (a strong
-- player did the tuck burpee in all 306 windows of step 11 until then). A candidate with no road (none in a cardio
-- slot) is left as it is.
local function fairDraw(ctx, list)
	local function clock(c)
		local road = ns.Library[c.move].road
		return road and (ctx.roadTs[road] or 0)
	end
	local oldest
	for _, c in ipairs(list) do
		local t = clock(c)
		if t and (not oldest or t < oldest) then oldest = t end
	end
	if not oldest then return list end
	list = prefer(list, function(c)
		local t = clock(c)
		return not t or t == oldest
	end)
	local formTs, first = ctx.state.formTs, nil
	for _, c in ipairs(list) do
		local t = clock(c) and (formTs[c.move] or 0)
		if t and (not first or t < first) then first = t end
	end
	return prefer(list, function(c) return not clock(c) or (formTs[c.move] or 0) == first end)
end

-- The four winds (slice 54): where a slot's candidates hold holds of more than one facing (Library.lua: the core and
-- side ladders' front, side and back holds), the holds of the facing furthest behind its share (C.facing_share,
-- McGill's normal endurance times, an internal guide never shown) come first, the others leave (every candidate with
-- no facing stays). A facing's measure is its best hold standing on its ladder's current rung or among the slot's
-- candidates, each record over its own window's top in thousandths, 0 with none: a step that holds no hold of a facing
-- (the core's step 6, no back hold) measures it by the holds of the rung below it the slot still offers, never as 0; F
-- is further behind than G when measure(F) × share(G) < measure(G) × share(F). A due anchor keeps its record slot
-- (pick's overload engine reads the rules' candidates).
local function windsFirst(ctx, list)
	local measure, count = {}, 0
	for _, c in ipairs(list) do
		local facing = ns.Library[c.move].facing
		if facing and not measure[facing] then measure[facing], count = 0, count + 1 end
	end
	if count < 2 then return list end
	local function read(id, m)
		local best = ns.Record(ctx.state, id)
		local _, hi = ns.Window(m.ladder, m.rung, id)
		if best and hi then measure[m.facing] = math.max(measure[m.facing], math.floor(best * 1000 / hi)) end
	end
	for _, c in ipairs(list) do
		local m = ns.Library[c.move]
		if m.facing then read(c.move, m) end
	end
	for id, m in pairs(ns.Library) do
		if m.facing and measure[m.facing] and m.rung == ctx.rung[m.ladder] then read(id, m) end
	end
	local share = C.facing_share
	local behind
	for facing, v in pairs(measure) do
		if not behind or v * share[behind] < measure[behind] * share[facing] then behind = facing end
	end
	return prefer(list, function(c)
		local facing = ns.Library[c.move].facing
		return not facing or measure[facing] * share[behind] == measure[behind] * share[facing]
	end)
end

-- Park-Miller (the minimal standard generator): same seed, same pump, so /reload keeps it. Its first outputs are
-- nearly linear in the seed, and neighbouring pumps have neighbouring seeds, so the seed is mixed, a few draws are
-- thrown away, and a pick reads the high-order part of the draw.
local M = 2147483647
local function roller(questID, day, pumpNo)
	local x = ((questID or 0) % 1048576 * 7919 + (tonumber((day:gsub("-", ""))) or 0) % 100000 * 104729
		+ pumpNo * 15485863) % (M - 1) + 1
	local function draw()
		x = x * 16807 % M
		return x
	end
	for _ = 1, 3 do draw() end
	return function(n)
		return math.floor(draw() / M * n) + 1
	end
end

local function why(ctx, entry)
	ctx.why[#ctx.why + 1] = entry
end

-- One slot: the best candidates for a quest's fit, the hard rules, the preferences, the roll over the top three.
-- opts: { distinct = patterns taken in this pump's tier, only = a filter for special slots, warm = a match pump's warm
-- move (the move done longest ago, warmOrder) }.
-- An anchor whose record attempt is due (its cadence passed, or a placement owed), or a move of a climb's path that
-- took its turn (candidates): the variety rules never hold it back (for a player of two pumps a day its
-- cadence falls inside the last five pumps), and the overload engine gives it a record slot. One test for the pick and
-- the day plan.
local function dueAnchor(c)
	return (c.anchor == true or c.turn == true) and (c.due >= 1 or c.placing == true)
end

local function pick(ctx, q, opts, roll)
	opts = opts or {}
	local fit, themes = ns.FitVector(q), q and q.themes or {}
	local pool, removed = {}, {}
	for _, c in ipairs(ctx.owedOut or {}) do
		if not opts.only or opts.only(c) then removed[#removed + 1] = { move = c.move, reason = "owed" } end
	end
	for _, c in ipairs(ctx.pool) do
		if not opts.only or opts.only(c) then -- outside a special slot's scope: not worth naming
			local reason = leftOut(ctx, c)
			if reason then removed[#removed + 1] = { move = c.move, reason = reason } else pool[#pool + 1] = c end
		end
	end
	if #pool == 0 then return nil, removed end
	local eligible = #pool
	for _, c in ipairs(pool) do c.eligible = eligible end
	-- the variety rules first (no pattern three pumps in a row, no move again within five pumps while the pool holds
	-- five), then distinct patterns inside the pump; a rule that would empty the pool gives way
	pool = prefer(pool, function(c) return not ctx.lastPatterns[ns.Library[c.move].pattern] end)
	if eligible >= C.variety_recent_pumps then -- (never a due anchor: dueAnchor)
		pool = prefer(pool, function(c) return not ctx.recentMoves[c.move] or dueAnchor(c) end)
	end
	pool = prefer(pool, function(c) return not (opts.distinct and opts.distinct[ns.Library[c.move].pattern]) end)
	-- the rotation: the moves with the fewest sets this game day go first, so every move the setup allows has its turn
	-- before one comes again (moves of one pattern tie on every score, and the lowest id took them all: David met
	-- ankle rocks eight times in three days and yoga twice)
	local fewest
	for _, c in ipairs(pool) do
		local n = ctx.done[c.move] or 0
		if not fewest or n < fewest then fewest = n end
	end
	-- the overload engine below looks at the moves the variety rules leave (scored, for /qp why), so the rotation and
	-- the favourites never take a due anchor's record attempt
	local rules = pool
	for _, c in ipairs(rules) do score(ctx, c, fit, themes) end
	pool = prefer(pool, function(c) return (ctx.done[c.move] or 0) == fewest end)
	-- then the player's favourites (the sheet's move menu): among the moves the rules leave, the marked ones first;
	-- then a cardio slot's fair draw and a core slot's four winds (slice 54)
	pool = prefer(pool, function(c) return ctx.f.favourites[c.move] == true end)
	local chosen
	if opts.warm then
		-- a match pump's warm move (planned's `keep`): the move done longest ago first (warmOrder), with no fair draw,
		-- winds, roll or record attempt (warmable), so a block's warm moves take turns one by one (2026-11-01, David,
		-- after ten tibialis raises: "generally it's pretty repetetive right now in the first excercise")
		table.sort(pool, warmOrder)
		chosen = pool[1]
		-- a pose family's due pose takes it, as below (an expedition's warm-up or cool-down pose: easeRound's `keep`)
		if ns.POSE[chosen.ladder] and not dueAnchor(chosen) then
			for _, c in ipairs(rules) do
				if c.ladder == chosen.ladder and dueAnchor(c) then chosen = c end
			end
		end
	else
		pool = fairDraw(ctx, pool)
		pool = windsFirst(ctx, pool)
		table.sort(pool, byScore)
		local top = math.min(C.roll_top, #pool)
		chosen = pool[roll and roll(top) or 1]
		-- The overload engine: a ladder 1.5 cadences overdue takes the slot whatever the roll chose, and a slot that goes
		-- to a record attempt goes to the most overdue ladder that is due or placing (variant records never starve the
		-- move that decides a rung); a pose family's slot goes to the pose due for its mark while that is due that game
		-- day (ns.DuePose, slice 47; its standing pose until then), or to the pose of its rung's path that took its turn
		-- while that is due (its other poses, and the main pose waiting for it, never starve the climb); a cardio or flow
		-- pick otherwise stays the quest's.
		local forced
		for _, c in ipairs(rules) do
			if (c.anchor or c.turn) and c.waited >= C.overdue_force and (not forced or c.overdue > forced.overdue) then
				forced = c
			end
		end
		if forced then
			chosen = forced
		elseif recordMove(ns.Library[chosen.move]) then
			for _, c in ipairs(rules) do
				if dueAnchor(c) and (not dueAnchor(chosen) or c.overdue > chosen.overdue) then chosen = c end
			end
		elseif ns.POSE[chosen.ladder] and not dueAnchor(chosen) then
			for _, c in ipairs(rules) do
				if c.ladder == chosen.ladder and dueAnchor(c) then chosen = c end
			end
		end
	end
	-- the pick first, then the next best two (the overload engine may have picked one of the top three)
	local named = { chosen.move }
	for _, c in ipairs(pool) do
		if #named < 3 and c.move ~= chosen.move then named[#named + 1] = c.move end
	end
	why(ctx, { move = chosen.move, score = chosen.score, parts = chosen.parts, removed = removed, top = named,
		block = opts.block })
	return chosen, removed
end

-- The slot a candidate becomes: its row fields (ladder 0 off the ladders), its kind, its dose later.
local function slotOf(c, questID, tier)
	local m = ns.Library[c.move]
	return { move = c.move, kind = K[m.dose], questID = questID, tier = tier, ladder = c.ladder or 0,
		rung = c.rung or m.rung, anchor = c.anchor, placing = c.placing, eligible = c.eligible, check = c.check }
end

-- What a move the pump holds leaves to the picks after it. An owed rider (`c.owed`) holds no region (David, 2026-10-16:
-- "Generally the banked moves should be treated as if they were done for the future picks"): it rides every pump until
-- it is done, so while it held its region a run of banked pumps shut every later pump's record moves out, four banked
-- record moves on the four regions all of them, and a done move holds none either.
local function take(ctx, c, counted)
	local m = ns.Library[c.move]
	ctx.used.moves[c.move] = true
	local block = ns.Block(c.move)
	if not counted then ctx.blocks[block] = ctx.blocks[block] + 1 end -- an owed move of the plan's days counts already
	if c.anchor and placementLadder(ctx, c.ladder) then ctx.used.placement = true end
	if recordMove(m) and m.region and not c.owed then ctx.used.regions[m.region] = true end
	if ns.POSE[c.ladder] then ctx.used.families[c.ladder] = true end
end

-- The countable form a counted cardio rung's path names while its stage still owes clean days (ns.PathTurn,
-- 2026-10-16: the skater steps on one of step 5's, the squat thrusts or the sprawls on two of step 6's): it takes the
-- window's turn from the step on, so its days come among the step's own, and yields for the rest of the game day once
-- it came that day (slice 52, 2026-10-23: it came before the rotation, so the squat thrusts came three times in 18
-- minutes on 10-16); nil otherwise (a timed rung, one whose climb the setup shuts, a path in).
local function cardioTurn(ctx)
	local state, f, rung = ctx.state, ctx.f, ctx.cardioRung
	if not (ns.CountedRung(rung, f) and ns.CanRungUp(ns.CARDIO, rung, f, state)) then return nil end
	local turn = ns.PathTurn(state, ns.CARDIO, f, rung)
	if turn and (ctx.done[turn] or 0) > 0 then return nil end
	return turn
end

-- The all-out form for the pump's end, chosen by the same score among the forms the cardio rung allows (rungs 1-4
-- the all-out cross-knee drives, fast feet, shadow boxing and the step jacks, the floor every filter leaves, the sprint
-- from 4; 5-6 add the jacks, squat jumps, skater hops, the climbers, squat thrusts and the sprawl; 7-9 the burpees);
-- `minRung` asks for the rung's own forms. A countable form of the player's own rung comes first (2026-10-05): its
-- counted set is the rung's climb (ns.Progression), so every all-out window is a try at it. Then the rotation, as
-- every pick's (docs/plan.md, "The pick"), before the pattern rules: the forms with the fewest sets that game day, then
-- one the last five pumps did not run (David, 2026-10-10: "I now also got for the third time skater hops. Why? There
-- should be variety": at rung 5 the two jacks share their pattern, so the pattern rules kept skater hops alone). While
-- the counted rung's path still owes clean days (2026-10-16), the form it names comes first (cardioTurn), as a strength
-- ladder's path takes its record turn. Then the player's favourites, as in every pick, and the fair draw (fairDraw,
-- slice 54, 2026-10-23: the favourites acted in no window until then). A form that is not open never comes.
local function finisherMove(ctx, q, roll, minRung, taken)
	local list = {}
	for id, m in pairs(ns.Library) do
		if m.finisher and m.finisher <= ctx.cardioRung and (not minRung or m.finisher >= minRung)
			and jumpReady(ctx, m) and ns.Passes(id, ctx.f) and not ctx.used.moves[id]
			and not ctx.owed[id] and isOpen(ctx, id) then
			list[#list + 1] = { move = id, ladder = ns.CARDIO, rung = ctx.cardioRung, due = ladderDue(ctx, ns.CARDIO) }
		end
	end
	if #list == 0 then return nil end
	local eligible = #list
	local turn = cardioTurn(ctx)
	list = prefer(list, function(c)
		local m = ns.Library[c.move]
		return c.move == turn or not turn and m.count ~= nil and m.rung == ctx.cardioRung
	end)
	local fewest
	for _, c in ipairs(list) do
		local n = ctx.done[c.move] or 0
		if not fewest or n < fewest then fewest = n end
	end
	list = prefer(list, function(c) return (ctx.done[c.move] or 0) == fewest end)
	list = prefer(list, function(c) return not ctx.recentMoves[c.move] end)
	list = prefer(list, function(c) return not (taken and taken[ns.Library[c.move].pattern]) end)
	list = prefer(list, function(c) return not ctx.lastPatterns[ns.Library[c.move].pattern] end)
	list = prefer(list, function(c) return ctx.f.favourites[c.move] == true end)
	list = fairDraw(ctx, list)
	local fit, themes = ns.FitVector(q), q and q.themes or {}
	for _, c in ipairs(list) do score(ctx, c, fit, themes) end
	table.sort(list, byScore)
	local chosen = list[roll and roll(math.min(C.roll_top, #list)) or 1]
	why(ctx, { move = chosen.move, score = chosen.score, parts = chosen.parts, removed = {}, finisher = true })
	return chosen.move, eligible
end

-- The lowest finisher rung of a closing window's own forms: the rung-7 forms from cardio rung 7, the rung-5 and rung-6
-- forms below it (squat jumps and the stairs among them, beside the counted ones; the closing window comes from
-- closing_rung since 2026-10-05, from 7 until then).
local function ownMin(ctx)
	return ctx.cardioRung >= 7 and 7 or C.closing_rung
end

-- A counted set (2026-10-05, David: "Always 30 s"): an all-out window of a countable form runs count_s all out from
-- its go, with no build (2026-10-08, David: "Just let people go all out from the beginning to get the correct count"),
-- never × M, with no number to reach ("All out, clean reps", 2026-10-16: a count goal from the record until then); its
-- count is the player's to type or not, the window saved Yes, to the end, Form slipped or (slice 53) Not all out. Any
-- other all-out window has no count.
local function countSet(slot)
	if ns.Library[slot.move].count then
		slot.counted, slot.seconds = true, C.count_s
	else
		slot.counted = nil
	end
	return slot
end

-- The proving window's form (slice 53, 2026-10-23), a step of its own before the window's pick (finisherMove): while
-- the proving is due (ns.Proving: the counted step's clean days and path in, the first counted window of a new session
-- by the night rule), the step above's own countable form the filters pass (ns.ProvingForms), its anchor first, else
-- the one with the fewest sets this game day, never one the pump took or owes; nil where none is due or free. It runs
-- one rung up, inside the rung rule, as a counted set at the player's rung: the fold reads a counted row one rung above
-- its row's rung as the proving (DB.lua cardioMove), and its Yes, to the end writes the climb (ns.Progression).
-- Returns it and the forms it was picked from.
local function provingMove(ctx)
	if ns.Proving(ctx.state, ctx.f, ctx.now) ~= "due" then return nil end
	local forms, best = ns.ProvingForms(ctx.state, ctx.cardioRung, ctx.f), nil
	local anchor = ns.Anchor(ns.CARDIO, ctx.cardioRung + 1)
	for _, id in ipairs(forms) do
		if not (ctx.used.moves[id] or ctx.owed[id]) then
			if id == anchor then
				best = id
				break
			end
			if not best or (ctx.done[id] or 0) < (ctx.done[best] or 0) then best = id end
		end
	end
	if not best then return nil end
	why(ctx, { move = best, removed = {}, finisher = true, proving = true })
	return best, #forms
end

local function finisherSlot(ctx, q, questID, tier, roll, taken)
	local id, eligible = provingMove(ctx)
	if not id then id, eligible = finisherMove(ctx, q, roll, nil, taken) end
	id = id or ns.FLOOR_MOVE
	ctx.used.moves[id] = true
	ctx.blocks.cardio = ctx.blocks.cardio + 1
	return { move = id, kind = K.cardio, questID = questID, tier = tier, ladder = ns.CARDIO, rung = ctx.cardioRung,
		allout = true, eligible = eligible or 1 }
end

-- Keep going (slice 52) on a steady move whose plan is under the full move (ns.FULL_MOVE_S, 45 s): its `keep`, the
-- seconds the clock may run on to once the player presses it in its last keep_going_s (Events.lua Lc.KeepGoing);
-- never on an all-out window or a counted set, nor a match pump's warm move (the stand-up move's slot is built
-- without a dose, ns.Compose), nor a Long Haul (chosenSlot takes it off).
local function keepSlot(slot)
	if not (slot.allout or slot.warm) and slot.seconds < ns.FULL_MOVE_S then slot.keep = ns.FULL_MOVE_S end
end

-- Step jacks are always legal: the floor every filter falls back to, in place of the slot `cur`, on its quest and
-- tier, dosed at the cardio rung of `state` by the steady ledger's M for its form at `now`. A pain flag changes a
-- move's form, never the pump's shape: an all-out window stays one (a closing one too), a match pump's warm move stays
-- warm (the step jacks are its own last pick, warmPick), a Long Haul keeps its length (slice 55) and a steady move its
-- pace ask.
function ns.FloorSlot(state, cur, now)
	local slot = { move = ns.FLOOR_MOVE, kind = K.cardio, questID = cur.questID, tier = cur.tier, ladder = ns.CARDIO,
		rung = ns.Rung(state, ns.CARDIO), allout = cur.allout or nil, closing = cur.closing, warm = cur.warm,
		pace = cur.pace, haul = cur.haul }
	if slot.haul then
		slot.seconds = cur.seconds
		return slot
	end
	local mult = ns.SteadyM(state, ns.Library[ns.FLOOR_MOVE].rung, now)
	slot.seconds = slot.allout and ns.AlloutSeconds(mult) or ns.CardioSeconds(0, mult)
	keepSlot(slot)
	return slot
end

-- The rest of a phone app's own pump after its Ready view's Pain flags a joint once a move has a result (the apps
-- port it, tools/gen-engine.lua pins it; the addon's pumps re-fit instead, Events.lua refit), from slot `from` on: a
-- move the new flags still allow stays as it was, one they refuse becomes step jacks, the floor, dosed at `now`
-- (ns.FloorSlot: an all-out window stays one), and an owed move that carries the flag leaves the pump to wait in the
-- queue. Never composed again: the player saw these moves.
function ns.PainRest(state, profile, moves, from, now)
	local rest = {}
	for i = from, #moves do
		local slot = moves[i]
		if ns.Allowed(slot.move, state, profile) then
			rest[#rest + 1] = slot
		elseif not slot.owed then
			rest[#rest + 1] = ns.FloorSlot(state, slot, now)
		end
	end
	return rest
end

-- A flow move's seconds: its own, on each side of a side move (the two sides run one after the other).
local function flowSeconds(id)
	local m = ns.Library[id]
	return m.secs * (m.side and 2 or 1)
end

-- The pieces of a pose's mark and its ask (markSlot), each of a slot `c` ({ move, ladder, rung }). A goal pose of its
-- family's step dosed at the player's step (ns.Marks: any of them since slice 47, 2026-10-19, its main pose alone until
-- then): its row is its family's attempt of the game day whatever its slot asked (DB.lua markAttempt), and its slot
-- sets the mark where that can count now (marks).
local function stepGoal(ctx, c)
	return c.rung == ctx.rung[c.ladder] and ns.Marks(c.ladder, c.rung, c.move)
end

-- A pose its family's step names on its path (2026-10-12; the path since 2026-10-16) whose stage's days are still to
-- come and that has not come that game day: its slot sets its own goal pose as the goal and asks it, at the least
-- answer that climb counts (DB.lua markDays).
local function pathAsk(ctx, c)
	local l = c.ladder
	local up = ns.POSE[l] and ctx.rung[l]
	local at = up and ns.StageOf(c.move)
	return at and at.ladder == l and at.rung == up and (ctx.done[c.move] or 0) == 0
		and ns.TopDays(ctx.state, l, up, c.move) < ns.StageDays(ns.Path(l, up)[at.stage])
end

-- The answer a pose's goal pose still waits for (2026-10-16, slice 42: ns.GoalAsk, its top once, which can master its
-- step, or the mark a check names), which its slot asks once a game day; nil once the pose came that game day.
local function goalWaits(ctx, c)
	return (ctx.done[c.move] or 0) == 0 and ns.GoalAsk(ctx.state, c.move) or nil
end

-- Whether a pose's slot sets its family's mark as the goal (the Ready view's "Mark:" line; slice 21): it is a goal pose
-- of the player's step (stepGoal), its family's mark has not come that game day, no slot of this pump took it
-- (ctx.marked, markSlot), so the ask comes once a game day a family across its poses, and its answer can count now
-- (ns.DayIn nil: no wait by the night rule after the day counted last while the family climbs; 2026-10-20, slice 47's
-- fix round, as the pace ask's full-move reason, ns.LedgerAsk).
local function marks(ctx, slot)
	return stepGoal(ctx, slot) and not ns.MarkedToday(ctx.state, slot.ladder, ctx.now) and not ctx.marked[slot.ladder]
		and ns.DayIn(ctx.state, slot.ladder, ctx.now, ctx.f) == nil or nil
end

-- A pose's mark and its ask: the slot that sets the mark as the goal (marks) asks "How close did you get?" once it ran
-- its whole time (2026-10-09; its answer in the row, DB.lua markAttempt) where the answer decides something; `ask` is
-- the least answer that counts: Perfect on the main pose (ns.MainPose) of an orange or gold step its climb opened and
-- no hold has earned yet (the answer that earns its colour, DB.lua earnMark), else below its family's top, where a
-- step is still to open, the least that counts the day (ns.MarkNeeds: Good, or Perfect below a risky step). A pose of
-- the step's path whose stage still owes days (pathAsk) sets its own goal pose as the goal and asks it. Any pose whose
-- goal pose still waits for an answer (goalWaits) asks it too, once a game day: `goalAsk` the answer the goal pose
-- waits for, while `ask` stays the least answer that counts the day, that answer where no day counts
-- (2026-10-17, slice 42's review: the quad stretch, front split step 1's marking pose, said "Only Perfect counts a
-- day" to every new player because the dancer's check names it at Perfect, where Good counts the day). A pose a pump's
-- stretch slot gives for a check (checkSlot, `check`) asks what passes the check (ns.CheckAsk) where no day counts,
-- before its goal pose's answer (the 90-90 passes its check at Good where its goal pose may wait for Perfect).
local function markSlot(ctx, slot)
	slot.mark = marks(ctx, slot)
	local l, k = slot.ladder, slot.rung
	if slot.mark then ctx.marked[l] = true end
	local earn = slot.mark and ns.MainPose(l, k, slot.move) and ns.TopTier(l, k) and not ns.EarnCounts(ctx.state, l, k)
	slot.ask = earn and E.mark.perfect or slot.mark and k < ns.HeldTop(l) and ns.MarkNeeds(l, k) or nil
	if not slot.mark and pathAsk(ctx, slot) then slot.mark, slot.ask = true, ns.MarkNeeds(l, ctx.rung[l]) end
	if slot.check and not slot.ask then slot.mark, slot.ask = true, ns.CheckAsk(slot.check) end
	local want = goalWaits(ctx, slot)
	if want then slot.mark, slot.goalAsk, slot.ask = true, want, slot.ask or want end
end

-- Whether a steady cardio move's slot asks "Did you keep that pace to the end?" (slice 42, 2026-10-17: the cadence
-- lines set the effort by the talk test, so the ask is whether the player kept it; slice 52: three answers, a steady
-- move of any step): once a pump, the pump's first steady move the steady ledger would ask after (ns.LedgerAsk: room
-- that game day and a reason, its doubt, the full moves owed at the top M on a move of the full move's seconds or a
-- week without an answer); never an all-out window nor a match pump's warm move (slice 48's double-check: it runs
-- easy, at the stand-up's cadence, so a kept pace there proves no step's pace), whose next steady move asks it. Before
-- the dose (the closing window's look ahead) a steady move counts as one of the full move's seconds, since Keep going
-- makes a full move of a shorter one, so the move that can be one keeps its place. Its answer (E.pace) goes in the row
-- (ns.ResultRow) for the ledger (DB.lua steadyRow).
local function asks(ctx, slot)
	return not (ctx.paceAsked or slot.allout or slot.warm)
		and ns.LedgerAsk(ctx.state, slot.move, slot.rung, slot.seconds or ns.FULL_MOVE_S, ctx.now, ctx.f)
end

local function paceSlot(ctx, slot)
	if asks(ctx, slot) then slot.pace, ctx.paceAsked = true, true end
end

-- The dose: strength and holds from the records, never from the quest (the goal and its kind, ns.Goal); cardio from the
-- quest's active minutes × M, the steady ledger's for the form (slice 52; P until then), with Keep going where it runs
-- under the full move (keepSlot), an all-out window from its go at its form's own length (a counted set's count_s, else
-- the build and the fast part at M: one move, one exercise, David, 2026-10-15: "We should have only one exercise per
-- move."); flow at its own seconds, on each side of a side move. A first appearance is a max set with no number to
-- reach (David, 2026-10-03: "why give a first goal at all if we don't know the persons fitness"). A loaded move's set
-- is planned at a weight (ns.LoadPlan; none before a set with one) and its goal is the goal at it, "heavier" on a step
-- up; a placement keeps its max set with the weight planned. A match pump's warm move (`slot.warm`, slice 48's
-- double-check: an easy first effort after the sit, as warm-ups that help are easy, Fradkin 2010, and an easy one
-- before hard running took away the ECG changes of a sudden start, Barnard 1973) is dosed easy: a cardio move at its
-- quest's seconds with no pace ask (paceSlot) nor Keep going (keepSlot), its Ready view and its voice at the
-- stand-up's easy cadence; a rep or
-- hold move at the repeat share of its form, never a fresh record step (ns.Goal's `warm`), its row marked so the fold
-- counts it as no fresh set (ns.ResultRow, DB.lua); a pose at its own seconds.
local function dose(ctx, slot, activeSeconds)
	local state = ctx.state
	if slot.kind == K.cardio then
		if slot.allout then
			slot.seconds = ns.AlloutSeconds(formM(ctx, slot.move))
			countSet(slot)
		else
			slot.seconds = ns.CardioSeconds((activeSeconds or 0) / 60, formM(ctx, slot.move))
			paceSlot(ctx, slot)
			keepSlot(slot)
		end
	elseif slot.kind == K.flow then
		slot.seconds = flowSeconds(slot.move)
		markSlot(ctx, slot)
	elseif slot.kind == K.rep or slot.kind == K.hold then
		-- the weight and the goal (ns.PickGoal, which the Start's label reads too); a placement keeps its max set
		slot.target, slot.goal, slot.load, slot.loadUp, slot.loadFrom =
			ns.PickGoal(state, slot.move, slot.kind, ctx.now, ctx.f, slot.placing, slot.warm)
		if not slot.target then
			slot.maxSet = true
			local l = slot.ladder
			if ns.STRENGTH[l] then
				local lo, hi, own = ns.Window(l, slot.rung, slot.move)
				-- "Really?" above 2 × the rung's hi on any first max set of the ladder (a variant in its units only)
				if lo and ns.OwnUnit(slot.move, ns.RungMove(l, slot.rung, ctx.f), own) then slot.cap = hi end
				-- the anchor's first max set ends at hi ("enough: next rung") only where the ladder can climb by it: a
				-- placement (the ladder's first record or a break's, 2026-10-10) of a move of the rung's own level
				-- (2026-10-09), never into a rung no placement gives (ns.DaysOnly: an orange or gold one, slice 22, or a
				-- skill or tendon-heavy one; ns.Progression), nor one whose climb still waits for its path (ns.PathIn),
				-- nor a loaded move's planned at a weight under the one its climb asks (ns.LoadClimbs, 2026-10-12)
				if lo and slot.rung == ctx.rung[l] and placementLadder(ctx, l) and ns.StandsFor(l, slot.rung, slot.move)
					and ns.Library[slot.move].rung == slot.rung and ns.CanClimb(l, slot.rung, ctx.f, state)
					and not ns.PainOnLadder(state, l, slot.rung, ctx.f) and not ns.DaysOnly(l, slot.rung)
					and ns.PathIn(state, l, slot.rung, ctx.f) and ns.LoadClimbs(slot.move, slot.load) then
					slot.hi = hi
				end
			end
		end
	end
	return slot
end

-- Owed moves ---------------------------------------------------------------------------------------------------------

-- An owed all-out window: reps 1, under its own form (the mark is in the row, so it never depends on today's M), or
-- reps 2, a closing move owed under its jog's id before 2026-10-15, read alike: the window of the move the row names.
local function isFinisherRow(r)
	return r.kind == K.cardio and (r.reps == 1 or r.reps == 2)
end

-- An owed anchor at its ladder's rung that would be a placement.
local function isPlacement(ctx, r)
	local l = r.ladder
	return ns.STRENGTH[l] and r.rung == ctx.rung[l] and ns.StandsFor(l, r.rung, r.move) and placementLadder(ctx, l)
end

-- An owed move rides the next pump as it was owed, whatever that pump holds already (David, 2026-10-02: unfinished
-- moves "should just autoplay with the next quest I return and never stack passively"). Only the player's own setup
-- holds one back: a move the filters now exclude (the pain flags and the Body switches among them), or one more than a
-- rung above the player's after a rung down, waits until the setup or the rung lets it. It is paid with a lesser move
-- only when the player picks Swap's easier version (2026-10-05), and never with the painful one; a backup of its rung
-- the setup leaves unneeded rides as the move that stands for that rung (ridesAs, 2026-10-29). Cardio keeps its own
-- rung rule (ns.AboveReach): a cardio form waits while it is above the cardio rung (an all-out window by its finisher
-- rung), and a climb from rung 7 up, whose steady moves are the forms of rungs 4-6, holds back none of the forms
-- below them. A window of a form with no finisher rung waits by the form's own rung: only a reps-2 row owed before
-- 2026-10-15 names one (its steady lead: the jog, butt kicks, quick steps, high knees, pogo hops), while a reps-1 row
-- always names a finisher form; without the form's rung such a row rode at any cardio rung.
function riderFits(ctx, r)
	if not ns.Passes(r.move, ctx.f) then return false end
	local m = ns.Library[r.move]
	if r.kind == K.cardio then
		if isFinisherRow(r) then return (m.finisher or m.rung) <= ctx.cardioRung end
		return m.rung <= ctx.cardioRung
	end
	if not jumpReady(ctx, m) then return false end
	local scale = ns.ScaleLadder(m)
	return not (scale and scale ~= ns.CARDIO and m.rung > ctx.rung[scale] + ns.Reach(scale))
end

-- `own`: the owed row rides as its own move, never its rung's (chosenSlot: the move the player started stays)
local function riderSlot(ctx, r, own)
	-- an owed backup the setup leaves unneeded rides as its rung's move (ridesAs), which pays its row
	local slot = { move = not own and ridesAs(ctx, r) or r.move, kind = r.kind, questID = r.questID, tier = r.tier,
		ladder = r.ladder, rung = r.rung, owed = r }
	if r.kind == K.cardio then
		slot.allout = isFinisherRow(r)
		if slot.allout then
			dose(ctx, slot) -- the window at its form's own length today, from its go
		else
			-- a row owed before cardio went short and hard (2026-10-04) kept its old length, up to 270 s: it rides at no
			-- more than today's longest for its form, cardio_cap_s × M
			local mult = formM(ctx, r.move)
			slot.seconds = r.seconds > 0 and math.min(r.seconds, ns.CardioSeconds(math.huge, mult))
				or ns.CardioSeconds(0, mult)
			paceSlot(ctx, slot)
			keepSlot(slot)
		end
	elseif r.kind == K.flow then
		slot.seconds = r.seconds > 0 and r.seconds or flowSeconds(r.move) -- its asks: markRiders
	elseif r.kind == K.rep or r.kind == K.hold then
		-- dosed like a fresh move: at the ladder's current rung an owed anchor is the rung's attempt (a placement
		-- after a break, "enough" at hi)
		local l = r.ladder
		if ns.STRENGTH[l] and r.rung == ctx.rung[l] and ns.StandsFor(l, r.rung, slot.move) then
			slot.anchor = true
			slot.placing = ns.Placing(ctx.state, l, ctx.now) or nil
			if isPlacement(ctx, r) then ctx.used.placement = true end -- the pump carries no other
		end
		dose(ctx, slot)
	end
	take(ctx, { move = slot.move, owed = true }, ctx.owedCounted[r.key])
	return slot
end

-- Every owed move that fits: the plain ones oldest first (spreadRiders sets their play order once the effort before
-- them is known), then the all-out windows oldest first, which end the pump as its last efforts in place of the pump's
-- own window (David, 2026-10-14: "It should just be all coming right after the pump and when im done I can always bank
-- the rest"; until then only the oldest window rode and any other waited for the pumps after it, so 27 of his 30
-- unfinished moves rode). Under the arms rest (ctx.arms: a match pump after an aim game, slice 48) every owed window
-- waits, as the pump's own has none (slice 61); it is left out before riderSlot takes its move. The game's own pumps
-- never keep the rest.
local function pickRiders(ctx, owed)
	local plain, windows = {}, {}
	for _, r in ipairs(owed or {}) do
		if not isFinisherRow(r) and riderFits(ctx, r) then plain[#plain + 1] = riderSlot(ctx, r) end
	end
	if ctx.arms then return plain, windows end
	for _, r in ipairs(owed or {}) do
		if isFinisherRow(r) and riderFits(ctx, r) then windows[#windows + 1] = riderSlot(ctx, r) end
	end
	return plain, windows
end

-- The riders of `slots` whose move is none of `moves` (a set of move ids): those that ride beside the pump's own moves.
local function notOwn(slots, moves)
	local out = {}
	for _, slot in ipairs(slots) do
		if not moves[slot.move] then out[#out + 1] = slot end
	end
	return out
end

-- The body part an effort loads, which the spread of the owed moves reads (spreadRiders): a rep or hold move's region
-- (push, pull, legs or trunk), else its pattern; nil for a move this addon lacks.
local function effortPart(id)
	local m = id and ns.Library[id]
	return m and (m.region or m.pattern)
end

-- The plain owed moves a pump carries in the order they play (Rapha, 2026-10-28: an owed move that repeats the one
-- just done should not come straight after it; "yes build the catch up change"): oldest first, except that a move
-- owed twice (the move as it plays, an owed backup as the move it rides as, ridesAs) comes again only once every
-- other move among them has come (David, 2026-11-01, after two banked Bulgarian split squats rode one pump two
-- places apart: "Right after each other. No variety"), and none comes straight after an effort of its own body part
-- (effortPart) while another of them can go there. At each place the oldest of the first of these that holds one
-- goes next: a move not placed yet whose part differs from the effort just before it, a move not placed yet, a part
-- that differs, any. `before` is the move of the effort before the first of them (the pump's own last move, or for a
-- pump of owed moves alone the player's latest set, the fold's `lastMove`), nil for none. Nothing is swapped and
-- nothing dropped: each still pays its own owed row, and the all-out windows still come after them. The helper's
-- Session::payable_kept and the phones' CatchUp.plan keep the same order (the helper's list, which cannot tell what
-- a backup rides as, by the owed move itself).
local function spreadRiders(slots, before)
	local left, out, placed = {}, {}, {}
	for i, slot in ipairs(slots) do left[i] = slot end
	local last = effortPart(before)
	while #left > 0 do
		local at, rank = 1, 4
		for i, slot in ipairs(left) do
			local r = (placed[slot.move] and 2 or 0) + (last ~= nil and effortPart(slot.move) == last and 1 or 0)
			if r < rank then at, rank = i, r end
			if r == 0 then break end
		end
		local slot = table.remove(left, at)
		out[#out + 1] = slot
		placed[slot.move] = true
		last = effortPart(slot.move)
	end
	return out
end

-- An owed pose's asks (markSlot), set once the slots played before it took theirs: the pump's own moves first, then
-- the riders in play order (2026-10-20, slice 47's fix round: an owed goal pose of a family, played after the pump's
-- own moves, took the family's mark from the pump's own due pose, which then asked nothing).
local function markRiders(ctx, slots)
	for _, slot in ipairs(slots) do
		if slot.owed and slot.kind == K.flow then markSlot(ctx, slot) end
	end
end

-- How many owed moves the next pump would carry now (the HUD's "N ride with the next pump"): every one the setup does
-- not hold back (ns.OwedHeldRows), 0 when every owed move waits for a rung, a cleared pain flag, a Body switch or its
-- equipment.
function ns.RidersReady(state, profile, now)
	local owed = ns.OwedCount(state)
	return owed == 0 and 0 or owed - #ns.OwedHeldRows(state, profile, now)
end

-- The owed moves the game holds back now, by move id in order: the rider gates (the filters, the pain flags and the
-- Body switches among them, the rung rule). The sheet carries the list, so a catch-up on the phone or the watch pays
-- only what a pump in the game would carry (the helper leaves the rest out of its list). Until 2026-10-04 every owed
-- move was held too before the player's first move, because the game's first pump asked the pain question; the
-- question went that day (David), and with it the hold.
function ns.OwedHeld(state, profile, now)
	local held, ids = {}, {}
	for _, r in ipairs(ns.OwedHeldRows(state, profile, now)) do
		if not held[r.move] then
			held[r.move] = true
			ids[#ids + 1] = r.move
		end
	end
	table.sort(ids)
	return ids
end

-- The same moves as their owed rows, oldest first: the HUD's unfinished line offers Skip on each (Lc.SkipHeld), since
-- no pump carries them to a Ready view.
function ns.OwedHeldRows(state, profile, now)
	local ctx = context({ state = state, profile = profile, now = now })
	local rows = {}
	for _, r in ipairs(ns.OwedList(state)) do
		if not riderFits(ctx, r) then rows[#rows + 1] = r end
	end
	return rows
end

-- The move each owed row rides as now, by its owed key, for the rows that ride as another move (ridesAs): the HUD's
-- unfinished line names that move, as the pump that carries it shows it.
function ns.RidesAs(state, profile, now)
	local ctx = context({ state = state, profile = profile, now = now })
	local as = {}
	for _, r in ipairs(ns.OwedList(state)) do
		as[r.key] = ridesAs(ctx, r)
	end
	return as
end

-- Special slots ------------------------------------------------------------------------------------------------------

-- The Trial: a turn-in's placement in the Trial's order, whatever the quest; one per pump (plan convention 3).
local function trialSlot(ctx, questID)
	local placed, total = ns.TrialStatus(ctx.state, ctx.f)
	if placed >= total or ctx.used.placement then return nil end
	for _, l in ipairs(ns.TrialOrder) do
		local lad = ctx.state.ladders[l]
		local id = ctx.rungMove[l]
		if not (lad and lad.placedTs) and id and not regionTaken(ctx, id) and not ctx.used.moves[id]
			and not ctx.owed[id] then
			local c = { move = id, ladder = l, rung = ctx.rung[l], due = 1, anchor = true }
			why(ctx, { move = id, trial = true, removed = {} })
			take(ctx, c)
			local slot = slotOf(c, questID, 1)
			slot.trial, slot.eligible = true, 1
			return slot
		end
	end
	return nil
end

-- A ladder's or a pose family's last turn: its last set, or a later move of it the queue owes (ctx.turn).
local function lastTurn(ctx, l)
	local lad, owed = ctx.state.ladders[l], ctx.turn[l]
	local ts = lad and lad.lastTs
	if owed and (not ts or owed > ts) then return owed end
	return ts
end

-- The ladder of `order` (the strength and hold ladders by default, the pose families for the mobility and yoga blocks)
-- that has waited longest since its last set, by its cadences (one never set first), among those that went at least
-- `minHours` (when given) and hold a move the pump can take that `keep` (when given) passes; nil when none does.
local function mostDue(ctx, keep, minHours, order)
	local best, bestWaited
	for _, l in ipairs(order or ns.STRENGTH_ORDER) do
		local h = hoursSince(lastTurn(ctx, l), ctx.now) or math.huge
		local waited = h / C.cadence_h[l]
		local ok = false
		if h >= (minHours or 0) then
			for _, c in ipairs(ctx.pool) do
				if c.ladder == l and not leftOut(ctx, c) and (not keep or keep(c)) then ok = true end
			end
		end
		if ok and (not bestWaited or waited > bestWaited) then best, bestWaited = l, waited end
	end
	return best
end

-- The major move (David, 2026-10-02: "any of the major exercises that are needed for a good body. They need to come
-- regularly with priority"): once a strength or hold ladder has gone major_h without a record set, a quest's pump
-- opens on a record attempt on the one of them that has waited longest by its cadence, before the quest's own moves
-- (within its pace's most, quota), the quest's fit and theme choosing among the ladder's moves. It keeps the variety
-- rule: a move that would run its pattern a third pump in a row is no major move, so a ladder with only such moves
-- leaves the turn to the next one.
local function majorSlot(ctx, q, roll)
	local function fresh(c)
		return not ctx.lastPatterns[ns.Library[c.move].pattern]
	end
	local best = mostDue(ctx, fresh, C.major_h)
	return best and pick(ctx, q, { only = function(x) return x.ladder == best and fresh(x) end }, roll) or nil
end

-- The major move's slot says so, with the hours its ladder waited (nil: never set), for the log (Log.lua).
local function markMajor(ctx, slot)
	slot.major, slot.waited = true, hoursSince(lastTurn(ctx, slot.ladder), ctx.now)
end

-- A record attempt on the ladder that has waited longest, for a pump the player asks for with nothing owed; a cardio
-- move when the pump leaves no ladder.
local function dueSlot(ctx, roll)
	local best = mostDue(ctx)
	local c = best and pick(ctx, nil, { only = function(x) return x.ladder == best end })
	return c or pick(ctx, nil, { only = function(x) return x.ladder == ns.CARDIO end }, roll)
end

-- The day plan (David, 2026-10-03: "its more improtant a good variety and trainingplan is followed than that the
-- excercises follow what happened in the quest"). A move of the pump's own goes to the block furthest behind its share
-- of the game day's moves (the player's focus picks the shares, focus_share), the major move, the riders and the
-- pump's earlier moves counted; the quest's fit and theme then choose inside the block. The variety rules come first:
-- a block whose every move would run its pattern a third pump in a row or repeat a move of the last five pumps waits
-- while another block has a fresh one (a block's pool is small, and inside it the two rules would give way). A tie
-- goes by ns.BLOCKS. Integers only, so the phone engines give the same block. With `keep` (a match pump's warm move,
-- warmable) only the candidates it passes open a block.
local function planBlock(ctx, keep)
	local total = 1
	for _, block in ipairs(ns.BLOCKS) do total = total + ctx.blocks[block] end
	local open, fresh = {}, {}
	for _, c in ipairs(ctx.pool) do
		if not leftOut(ctx, c) and (not keep or keep(ctx, c)) then
			local block = ns.Block(c.move)
			open[block] = true
			if not ctx.lastPatterns[ns.Library[c.move].pattern] and (not ctx.recentMoves[c.move] or dueAnchor(c)) then
				fresh[block] = true
			end
		end
	end
	local best, bestGap, any
	for _, block in ipairs(ns.BLOCKS) do any = any or fresh[block] end
	for _, block in ipairs(ns.BLOCKS) do
		local gap = ctx.shares[block] * total - 100 * ctx.blocks[block]
		if (any and fresh[block] or not any and open[block]) and (not bestGap or gap > bestGap) then
			best, bestGap = block, gap
		end
	end
	return best
end

-- Waiting stretch checks (2026-10-17, slice 42's review; the plan's decision 4: "a step waiting on a check pulls that
-- pose into the next pump's stretch slot"), since 2026-10-24 only a pose within its family's reach (ns.AboveReach: no
-- pose above the player's own step, slice 21; David, after the big toe fold came at pike step 1: "that makes no sense.
-- I'm nowhere close to that") -----------------------------------------------------------------------------------------

-- The moves that wait for checks (Library.lua `checks`), by id, built once.
local CHECKED
local function checkedMoves()
	if not CHECKED then
		CHECKED = {}
		for id, m in pairs(ns.Library) do
			if m.checks and m.ladder then CHECKED[#CHECKED + 1] = id end
		end
		table.sort(CHECKED)
	end
	return CHECKED
end

-- The pose a waiting check `o` (an entry of a rung's `needs` or a move's `checks`) runs in a pump, and the check it
-- runs: of an `any` group the first entry that gives one, and before a check any check its family's climb to its step
-- waits for (ns.CheckFirst, as a card's "Do the check" runs it); nil where a step of a strength or cardio ladder comes
-- first, the check is done, or the pose ns.NeedAt names with the setup alone is shut by the pain flags, not open or
-- above its family's reach (ns.AboveReach, 2026-10-24: the family climbs to it on its own poses, and the card's Start
-- runs it at any step). `depth` counts the checks followed (the library never chains them past four).
local function checkPose(ctx, o, depth)
	depth = depth or 0
	if o.any then
		for _, x in ipairs(o.any) do
			local id, x2 = checkPose(ctx, x, depth)
			if id then return id, x2 end
		end
		return nil
	end
	if depth > 4 or not ns.IsCheck(o) then return nil end
	local first = ns.CheckFirst(ctx.state, o)
	if first ~= o then return checkPose(ctx, first, depth + 1) end
	if ns.NeedDone(ctx.state, o) then return nil end
	local id = ns.NeedAt(o, ctx.setup)
	if id and ns.POSE[ns.Library[id].ladder] and ns.Passes(id, ctx.f) and isOpen(ctx, id)
		and not ns.AboveReach(ctx.state, id) then
		return id, o
	end
	return nil
end

-- The stretch checks that wait, in the order the stretch and yoga slots take them, each { move = its pose, check = the
-- check it runs }, once a compose: by ladder id, those of the climb into each open ladder's next step (the rung's
-- `needs` not done), then by move id those of the moves the pumps would offer but for them (a move of an open ladder
-- at its step or the one below that the filters allow, held by its first check not done, ns.MoveOpen, and by no
-- step's path, ns.PathWaits); each pose once, and only one within its family's reach (checkPose). A family's opening
-- (the planche's wrist lean) waits for the family's turn, never a slot.
local function waitingChecks(ctx)
	if ctx.checks then return ctx.checks end
	local list, seen, state = {}, {}, ctx.state
	local function add(o)
		local id, x = checkPose(ctx, o)
		if id and not seen[id] then
			seen[id] = true
			list[#list + 1] = { move = id, check = x }
		end
	end
	for l = 1, ns.LADDER_MAX do
		local lad = ns.Ladders[l]
		local w = lad and ns.Open(state, l) and lad.rungs[ctx.rung[l] + 1]
		for _, o in ipairs(w and w.needs or {}) do
			if not ns.NeedDone(state, o) then add(o) end
		end
	end
	for _, id in ipairs(checkedMoves()) do
		local m = ns.Library[id]
		local rung = ctx.rung[m.ladder]
		if rung and (m.rung == rung or m.rung == rung - 1) and ns.Open(state, m.ladder) and ns.Passes(id, ctx.f) then
			local open, waits = ns.MoveOpen(state, id, ctx.f)
			if not open and waits.kind == "check" and not ns.PathWaits(state, id, ctx.f) then add(waits.need) end
		end
	end
	ctx.checks = list
	return list
end

-- A stretch or yoga slot's waiting check (waitingChecks): the first whose pose the pump has not taken, does not owe
-- (its owed copy rides instead) and has not had this game day (its ask comes once a game day, as a goal pose's), and
-- that was never asked or whose family is the slot's (`ladder`, the block's most-due family: a check asked once comes
-- again on its family's turn, never ahead of every family each game day; 2026-11-01, after the desk squat David
-- answered Okay took the first stretch slot of five game days in a row and the families behind it waited 10 days), a
-- candidate of its family at the pose's own step with the check it runs (`check`, which markSlot asks), scored for
-- /qp why in the block it fills; nil where none waits.
local function checkSlot(ctx, q, block, ladder)
	local answers = ctx.state.marks or {}
	for _, w in ipairs(waitingChecks(ctx)) do
		local asked = answers[w.move] and answers[w.move].askTs
		if not ctx.used.moves[w.move] and not ctx.owed[w.move] and (ctx.done[w.move] or 0) == 0
			and (not asked or ns.Library[w.move].ladder == ladder) then
			local m = ns.Library[w.move]
			local c = { move = w.move, ladder = m.ladder, rung = m.rung, due = 1, check = w.check }
			score(ctx, c, ns.FitVector(q), q and q.themes or {})
			why(ctx, { move = c.move, score = c.score, parts = c.parts, removed = {}, top = { c.move }, block = block })
			return c
		end
	end
	return nil
end

-- One move of the pump's own: the plan's block, then the pick inside it. A strength slot goes to the ladder that has
-- waited longest since its last record set once it has waited major_h, as the major move does, so a long day brings
-- every ladder round; the accessory moves (calves, carries, jumps) take it while no ladder waits. A stretch or yoga
-- slot goes to the pose family of its block that has waited longest, a waiting stretch check first (checkSlot). With
-- `keep` (a match pump's warm move: warmable, warmPick) the block and the pick inside it take only the candidates it
-- passes, a strength slot any ladder's (a warm leg move is no ladder's record turn), the pick the move done longest ago
-- (pick's `warm`), and a check waits for a later slot (a check asks its goal pose: no warm-up); `block`, when given, is
-- the block (the warm move keeps the day plan's, warmPlan; an expedition's warm-up or cool-down, easeRound, its
-- stage's, nil where it holds no candidate).
local function planned(ctx, q, distinct, roll, keep, block)
	block = block or planBlock(ctx, keep)
	if not block then return nil end
	local function inBlock(c)
		return ns.Block(c.move) == block and (not keep or keep(ctx, c))
	end
	local only = inBlock
	if block == "strength" and not keep then
		local function fresh(c)
			return inBlock(c) and not ctx.lastPatterns[ns.Library[c.move].pattern]
		end
		local ladder = mostDue(ctx, fresh, C.major_h)
		if ladder then only = function(c) return fresh(c) and c.ladder == ladder end end
	elseif block == "mobility" or block == "yoga" then
		-- a stretching or yoga slot goes to the pose family of its block that has waited longest since its last pose,
		-- by its cadence, as a strength slot goes to a ladder (slice 21), so a day brings every family round; inside it
		-- the rotation and the favourites order the poses, and the pose due for its mark comes once a game day (pick,
		-- ns.DuePose)
		local function family(c)
			return inBlock(c) and ns.POSE[c.ladder] == true
		end
		-- a family the pump took already waits for the block's other families (one family a pump while another has a
		-- pose to give, as one record move a region)
		local function fresh(c)
			return family(c) and not ctx.used.families[c.ladder]
		end
		local ladder = mostDue(ctx, fresh, 0, ns.POSE_ORDER) or mostDue(ctx, family, 0, ns.POSE_ORDER)
		local check = not keep and checkSlot(ctx, q, block, ladder)
		if check then return check end
		if ladder then only = function(c) return inBlock(c) and c.ladder == ladder end end
	end
	return pick(ctx, q, { only = only, distinct = distinct, block = block, warm = keep ~= nil }, roll)
end

-- A match pump's warm move (slice 48, docs/computer-games.md §5: after match_sat_min minutes sat a match pump opens on
-- an easy move on the player's feet; standing up after a long sit can drop the blood pressure, and a cold record set
-- feeds the record a worse count), dosed easy (its slot's `warm`: dose, paceSlot), never on the floor, a seat or a
-- jump, and never a record attempt. Since 2026-11-01 at every pace as at Light since slice 48's double-check (David,
-- after ten tibialis raises in a row: "generally it's pretty repetetive right now in the first excercise. While a lot
-- are missing"): the move keeps the day plan's block wherever that block holds a warmable move, else the plan's block
-- among the warmable ones (warmPlan), and inside it the move done longest ago (pick's `warm`). A warm leg move
-- (warmLeg) is a standing leg move (ns.StandingLeg) the player has a record on, so its easy goal comes from it and it
-- is never a first max set, that is no due record turn nor a placement (dueAnchor) and stands for no strength ladder's
-- rung (its set would be the rung's attempt, DB.lua anchorAttempt, where an easy set counts a plateau day and moves the
-- anchor's clock), of the player's step or any step below it (warmPool). A warm cardio move (warmable) is one of the
-- day plan's cardio with no floor, impact or seated flag (slice 48's second fix round: never mountain climbers, squat
-- thrusts, a sprawl, the jacks, a jog or the bike). Never a pose (2026-11-03, Rapha, after a match pump opened on the
-- standing quad stretch: a static stretch belongs after the moves that warm the muscles, never cold after a long sit;
-- from 2026-11-01 an upright pose could take it): the day plan's stretch comes after the pump's other own moves
-- (ns.Compose). Where no block holds a warmable move, the step jacks (ns.FLOOR_MOVE). Swap's other move for it stays
-- warm where it is easy too (swapSlot).
local function warmLeg(ctx, c)
	local l = c.ladder
	return ns.StandingLeg(c.move) and ns.Record(ctx.state, c.move) ~= nil and not dueAnchor(c)
		and not (ns.STRENGTH[l] and c.rung == ctx.rung[l] and ns.StandsFor(l, c.rung, c.move))
end

local function warmable(ctx, c)
	local f = FLAGS[c.move]
	return ns.Block(c.move) == "cardio" and not (f.floor or f.impact or f.seated) or warmLeg(ctx, c)
end

-- The standing leg moves of the strength ladders (ns.StandingLeg), by id, built once.
local STANDING_LEGS
local function standingLegs()
	if not STANDING_LEGS then
		STANDING_LEGS = {}
		for id, m in pairs(ns.Library) do
			if ns.STRENGTH[m.ladder] and ns.StandingLeg(id) then STANDING_LEGS[#STANDING_LEGS + 1] = id end
		end
		table.sort(STANDING_LEGS)
	end
	return STANDING_LEGS
end

-- The warm move's pool: the pump's candidates and, after them by id, every standing leg move of an open strength
-- ladder below the player's step there (the candidates hold the step, and the one below until a move's record holds
-- its top) that the player has a record on, the filters pass, the pump does not owe, the player did not step down
-- from and whose path opened it (2026-11-01: the squat ladder at step 6 left the tibialis raise the only warm leg
-- move, its air, tempo and chair squats and lunges below the candidates' steps), each at its own step, so its row
-- stands for no rung the ladder is on.
local function warmPool(ctx)
	local list, have = {}, {}
	for _, c in ipairs(ctx.pool) do
		list[#list + 1] = c
		have[c.move] = true
	end
	for _, id in ipairs(standingLegs()) do
		local m = ns.Library[id]
		local rung = ctx.rung[m.ladder]
		if not have[id] and rung and m.rung < rung and ns.Open(ctx.state, m.ladder) and ns.Passes(id, ctx.f)
			and not ctx.owed[id] and ns.Record(ctx.state, id) ~= nil and not steppedDown(ctx, id) and isOpen(ctx, id) then
			addCandidate(list, id, { ladder = m.ladder, rung = m.rung, due = 0 })
		end
	end
	return list
end

-- The warm move's block and the rule its pick keeps: the day plan's block where it holds a warmable move, else nil
-- (planBlock's choice among the blocks that do).
local function warmPlan(ctx)
	local block = planBlock(ctx)
	for _, c in ipairs(ctx.pool) do
		if ns.Block(c.move) == block and not leftOut(ctx, c) and warmable(ctx, c) then return block, warmable end
	end
	return nil, warmable
end

-- The warm move's candidate: planned over warmPool, its block warmPlan's.
local function warmPick(ctx, q, taken, roll)
	local pool = ctx.pool
	ctx.pool = warmPool(ctx)
	local block, keep = warmPlan(ctx)
	local c = planned(ctx, q, taken, roll, keep, block)
	ctx.pool = pool
	return c
end

-- Swap ---------------------------------------------------------------------------------------------------------------

-- Swap (slice 1; decided 2026-09-23 on David's delegation), before Start (Lc.SwapOptions adds the way back):
-- "Another move", the best-scoring candidate of the same pattern and class that the pump's other moves leave (one
-- record move a region among its own moves) other than the move its backup or its easier version names, and, on a
-- `skill` move or a move above the player's rung, "Easier version", the first move down its named regression at or
-- under the player's own rung (easierMove). Either is dosed
-- as a fresh pick, on its own record's goal (a first appearance a max set), so nothing is removed or lowered. The
-- stand-up move and /qp try
-- swap nothing; the Trial's placement takes only its backup (below), an owed placement its backup and its easier
-- version; any other owed move swaps to another move and to its backup at its level only (the backup at its rung,
-- never the closest rung below, another move at its rung or above, an all-out window a form of its finisher rung or
-- above) and to its easier version, its one way down (David, 2026-10-05: a banked pike push-up came back "and it does
-- not suggest the easier version"; an owed move had none until then), and the move it swaps to pays its owed row
-- (David, 2026-10-02: every owed move rides the next pump, "and there is also no option to switch"). A pump started
-- from a step's card or a move's window (`chosen`, slice 36) keeps the easier version and the backup and drops
-- "Another move", which would undo the pick, except on a counted set or an all-out window at the player's cardio rung,
-- whose other form of the rung trains the same climb (a held counted step's would climb). input: the pump's compose
-- input (state, profile, now); moves: its slots. Returns { another = slot?, easier = slot?, backup = slot? }.
local function swappable(slot)
	return slot and not slot.standup and not slot.practice
end

-- The rung a move stands for: its slot's rung or its own, whichever is higher.
local function levelOf(cur)
	local m = ns.Library[cur.move]
	return math.max(cur.rung or m.rung or 1, m.rung or 1)
end

local function swapSlot(ctx, cur, c)
	local slot = slotOf(c, cur.questID, cur.tier)
	slot.owed = cur.owed
	if slot.kind == K.cardio and cur.kind == K.cardio then
		-- a cardio move keeps the time its quest's minutes gave, its pace ask and a match pump's warm move its easy
		-- cadence (slice 48's double-check), a Long Haul its length (slice 55: its row the haul's, never the steady
		-- ledger's), and an all-out window stays one (a closing one too), dosed by its own form: a counted set's 30 s,
		-- any other the window's own length
		slot.allout, slot.closing = cur.allout, cur.closing
		if slot.allout then return dose(ctx, slot) end
		slot.seconds, slot.pace, slot.keep, slot.warm, slot.haul = cur.seconds, cur.pace, cur.keep, cur.warm, cur.haul
		return slot
	end
	-- a match pump's warm move stays one where its other move is easy too (a cardio move, a pose, a warm leg move:
	-- warmLeg); a rep or hold move that is no warm leg move takes a fresh pick's dose
	slot.warm = cur.warm and (slot.kind ~= K.rep and slot.kind ~= K.hold or warmLeg(ctx, c)) or nil
	return dose(ctx, slot, 0)
end

-- Another way to do it, same level (David, 2026-09-30: "the same level just different method to do it ... so if you
-- can't do the main one for whatever reason you can click on the other instead of doing nothing or banking"): a move
-- the filters pass that stands for the same rung of the same ladder (its chain, or a variant tagged with that rung), or
-- off the ladders one of the same pattern and rung, a different method first (other needs), the same dose next, the
-- nearest level then (a chain's tail stands in from below), never one a better tool's move takes the place of
-- (ns.BetterOpen) and never a move the queue holds (ctx.owed) but the one `cur`'s own owed row rides as (ridesAs: a
-- move started from a step's card pays that row as itself); where the setup leaves none at that level, the closest rung
-- below. `opts.chain`: the rung's own chain only, never a rung below, whose max set the rung's window would judge (a
-- placement; none when the chain holds no other move); `opts.atLevel`: the move's rung only and no move whose own rung
-- is lower (an owed move). Returns the move, its rung and whether it is lower.
-- A need is one item or an any-of list ({ "stairs", "step" }), which compares by its items, never by its table.
local function needKey(n)
	return type(n) == "table" and table.concat(n, "/") or tostring(n)
end

local function sameNeeds(a, b)
	local na, nb = a.needs or {}, b.needs or {}
	if #na ~= #nb then return false end
	for i, n in ipairs(na) do
		if needKey(n) ~= needKey(nb[i]) then return false end
	end
	return true
end

local function backupMove(ctx, cur, allowed, opts)
	local m = ns.Library[cur.move]
	local ladder = (cur.ladder or 0) ~= 0 and cur.ladder or m.ladder
	local level = levelOf(cur)
	local function fits(id)
		local owed = ctx.owed[id]
		return id ~= cur.move and (not owed or (cur.owed and owed == cur.owed.key)) and ns.Passes(id, ctx.f)
			and allowed(id) and not steppedDown(ctx, id)
			and isOpen(ctx, id) and jumpReady(ctx, ns.Library[id])
			and not (opts.atLevel and (ns.Library[id].rung or level) < (m.rung or level)) and not ns.BetterOpen(id, ctx.f)
	end
	for r = level, (opts.atLevel or opts.chain) and level or 1, -1 do
		local list = {}
		if ladder then
			if ladder ~= ns.CARDIO then
				for _, id in ipairs(ns.RungChain(ladder, r)) do
					if fits(id) then list[#list + 1] = id end
				end
			end
			for id, x in pairs(ns.Library) do
				if not opts.chain and x.ladder == ladder and x.rung == r and not ns.InChain(ladder, r, id) and fits(id) then
					list[#list + 1] = id
				end
			end
		else
			for id, x in pairs(ns.Library) do
				if not x.ladder and x.pattern == m.pattern and x.rung == r and fits(id) then list[#list + 1] = id end
			end
		end
		table.sort(list, function(a, b)
			local ma, mb = ns.Library[a], ns.Library[b]
			local da, db = sameNeeds(ma, m), sameNeeds(mb, m)
			if da ~= db then return db end
			if (ma.dose == m.dose) ~= (mb.dose == m.dose) then return ma.dose == m.dose end
			if (ma.rung or 0) ~= (mb.rung or 0) then return (ma.rung or 0) > (mb.rung or 0) end -- the nearest level
			return a < b
		end)
		-- lower when the move itself sits below the one it backs up (a chain's tail stands in from a rung below)
		if list[1] then return list[1], r, (ns.Library[list[1]].rung or r) < (m.rung or level) end
	end
end

-- The easier version, on a `skill` move or a move above the player's rung: the first move down its named regression
-- (`regress`, a real move: tools/lint.lua) at or under the player's own rung on the ladder it scales by (David,
-- 2026-10-05: "I should not get a hard exercise like this yet. I just started"; so a player whose push ladder stands on
-- the knee push-up gets the knee push-up for a pike push-up, never the push-up, the first on the chain until then)
-- that the filters pass, the cardio rung allows and the player has not stepped down from; none when the pump cannot
-- take that one (`allowed`: it holds it, or another record move of its region). Returns its move, which easierSlot
-- doses.
local function easierMove(ctx, cur, allowed)
	local m = ns.Library[cur.move]
	local scale = m.ladder or ns.ScaleLadder(m)
	local above = scale and ctx.rung[scale] and (m.ladder and cur.rung or m.rung) > ctx.rung[scale]
	if not (FLAGS[cur.move].skill or above) then return nil end
	local function fits(id)
		local e = ns.Library[id]
		local on = ns.ScaleLadder(e)
		return (not on or (e.rung or 1) <= ctx.rung[on]) and ns.Passes(id, ctx.f) and jumpReady(ctx, e)
			and not steppedDown(ctx, id) and isOpen(ctx, id)
	end
	local id, seen = m.regress, {}
	while id and not seen[id] and not fits(id) do
		seen[id] = true
		id = ns.Library[id].regress
	end
	if not id or seen[id] or not allowed(id) then return nil end
	return id
end

-- The easier version easierMove found (`id`; none without one), dosed as a fresh pick; its row carries the easier bit
-- (flag bit 6), which the fold reads (DB.lua easierMark), and an owed move's pays the owed row. Taken for the attempt
-- at the rung a placed strength or hold ladder stands on now, above rung 1 (its fresh rung move, or an owed row dosed
-- as that attempt: riderSlot), it names that ladder and rung (`down`, `downFrom`), and its result steps the ladder one
-- rung down (ns.EasierDown; David, 2026-10-13, whose squat ladder stayed on rung 7 through every easier pick on the
-- assisted pistol, which kept coming back).
local function easierSlot(ctx, cur, id)
	if not id then return nil end
	local rm = ns.Library[id]
	local easier = swapSlot(ctx, cur, { move = id, ladder = rm.ladder, rung = rm.rung })
	easier.easier = true
	local l, lad = cur.ladder, ctx.state.ladders[cur.ladder]
	if cur.anchor and ns.STRENGTH[l] and cur.rung == ctx.rung[l] and cur.rung > 1 and lad and lad.placedTs then
		easier.down, easier.downFrom = l, cur.rung
	end
	return easier
end

local function swapOptions(input, moves, index)
	local cur = moves[index]
	if not swappable(cur) then return {} end
	local ctx = context(input)
	ctx.pool = candidates(ctx)
	for i, s in ipairs(moves) do
		if i ~= index then
			take(ctx, { move = s.move, owed = s.owed ~= nil })
			-- a pose of the pump that holds its family's mark keeps it: the swapped move asks no second (markSlot)
			if s.mark and marks(ctx, s) then ctx.marked[s.ladder] = true end
		end
	end
	-- what a fresh pick meets, one record move a region among the pump's own moves (a rider holds none: take), except
	-- that a record move may trade for one of its own region: a rider may share it with one of the pump's own moves (it
	-- rides whatever the pump holds), and the trade adds none
	local m = ns.Library[cur.move]
	local own = recordMove(m) and m.region
	local function allowed(id)
		if ctx.used.moves[id] then return false end
		return not regionTaken(ctx, id) or (own ~= nil and ns.Library[id].region == own)
	end
	local owed, out = cur.owed ~= nil, {}
	-- a placement, the Trial's or an owed one (the Trial's put off), takes only the Trial's backup (David, 2026-10-01:
	-- "why is there no alternative shown"), a move of the rung's own chain, which stands for the rung, so the max set
	-- still places the ladder
	if cur.trial or (owed and isPlacement(ctx, cur)) then
		local other, _, lower = backupMove(ctx, cur, allowed, { chain = true })
		if other then
			local slot = slotOf({ move = other, ladder = cur.ladder, rung = cur.rung, due = 1, anchor = true,
				placing = cur.placing }, cur.questID, cur.tier)
			if cur.trial then slot.trial, slot.eligible = true, 1 end
			slot.owed = cur.owed
			out.backup = dose(ctx, slot, 0)
			out.backup.lower = lower or nil
		end
		-- an owed one offers its easier version too (David, 2026-10-05: a banked pike push-up came back "and it does not
		-- suggest the easier version"), which pays the owed row and places nothing
		if owed then out.easier = easierSlot(ctx, cur, easierMove(ctx, cur, allowed)) end
		return out
	end
	if cur.allout then
		-- an all-out window swaps to another form the cardio rung allows (a closing window to one of its own forms; an
		-- owed window to a form of its own finisher rung or above), never to a steady move, and has no easier version
		ctx.used.moves[cur.move] = true
		local form = finisherMove(ctx, nil, nil, math.max(cur.closing and ownMin(ctx) or 1, owed and m.finisher or 1))
		if form then out.another = swapSlot(ctx, cur, { move = form, ladder = ns.CARDIO, rung = ctx.cardioRung }) end
		return out
	end
	-- "Another move" never names the backup's move nor the easier version's (2026-11-02: David's push-up offered the
	-- desk archer push-up as "Closest your setup allows" and as "Another move"): it takes the next-best candidate, or
	-- none; the three are dosed in the order another, backup, easier
	local other, r, lower = backupMove(ctx, cur, allowed, { atLevel = owed })
	local down = easierMove(ctx, cur, allowed)
	local level = levelOf(cur)
	local list = {}
	for _, c in ipairs(ctx.pool) do
		local cm = ns.Library[c.move]
		if c.move ~= cur.move and c.move ~= other and c.move ~= down and cm.pattern == m.pattern
			and cm.class == m.class and not ctx.owed[c.move] and allowed(c.move)
			and not (owed and (c.rung or cm.rung or 1) < level) then
			list[#list + 1] = c
		end
	end
	for _, c in ipairs(list) do score(ctx, c, {}, {}) end
	table.sort(list, byScore)
	if list[1] then out.another = swapSlot(ctx, cur, list[1]) end
	if other then
		out.backup = swapSlot(ctx, cur, { move = other, ladder = ns.Library[other].ladder or 0, rung = r })
		out.backup.lower = lower or nil
	end
	out.easier = easierSlot(ctx, cur, down)
	return out
end

function ns.SwapOptions(input, moves, index, chosen)
	local options = swapOptions(input, moves, index)
	local cur = moves[index]
	if chosen and not (cur.allout and cur.rung == ns.Rung(input.state, ns.CARDIO)) then options.another = nil end
	return options
end

-- An open pump's slots dosed again on the records as they stand, after a record cleared, a ladder started over or a
-- count corrected (slice 44) while the pump is open (Events.lua Lc.Cleared, Lc.Corrected; the game's alone: the apps'
-- pumps cover every page that clears or corrects): a rep or
-- hold slot as dose gives a fresh pick of it (its goal, a first max set's cap and hi, a loaded move's weight), a
-- placement kept only while its ladder still places. The move, its ladder, rung and place stay, and any other slot
-- stays as it was, a counted set's too (no goal since 2026-10-16): its quest's minutes are gone.
function ns.Regoal(input, slots)
	local ctx = context(input)
	for _, slot in ipairs(slots) do
		if slot.kind == K.rep or slot.kind == K.hold then
			slot.placing = slot.placing and ns.Placing(ctx.state, slot.ladder, ctx.now) or nil
			slot.target, slot.goal, slot.maxSet, slot.cap, slot.hi = nil, nil, nil, nil, nil
			slot.load, slot.loadUp, slot.loadFrom = nil, nil, nil
			dose(ctx, slot)
		end
	end
end

-- A rep or hold slot's goal set again as its Ready view opens, on the records as the pump's sets before it left them
-- (Rapha, 2026-10-28, "yes build the catch up change"; Events.lua Lc.Next and Lc.Skip, tools/apppump.lua Pump:ready,
-- the phones' runners): a second set of a ladder in the same pump asks the repeat share (ns.Goal, goal_repeat_pct), as
-- a set in a later pump already does, where the pump was composed before its first set moved the ladder's clock. The
-- new dose is taken only where it asks less at the same weight: never a raise, never a max set or a placement's, and
-- the move, its rung and everything else of the slot stay. The caller never asks it of a move started from a step's
-- card. True where the goal changed.
function ns.ReadyGoal(input, slot)
	if not (slot and (slot.kind == K.rep or slot.kind == K.hold) and slot.target) or slot.placing then return false end
	local fresh = { move = slot.move, kind = slot.kind, ladder = slot.ladder, rung = slot.rung, anchor = slot.anchor,
		warm = slot.warm }
	dose(context(input), fresh)
	if not fresh.target or fresh.target >= slot.target or fresh.load ~= slot.load then return false end
	slot.target, slot.goal = fresh.target, fresh.goal
	return true
end

-- Compose ------------------------------------------------------------------------------------------------------------

-- A quest's own moves at a pace (C.pace_moves; David, 2026-10-05: "my suggestions would be Light (max 1 move per
-- quest), Balanced (current), and Extreme"): how many the day plan picks, whether a circuit's all-out window ends them,
-- and how many it picks beside the major move (nil: no major move). The tier's size counts the window, which comes
-- only where the pace gives a T3 quest more than one move. The major move comes on top of the size, but a quest never
-- gives more than its pace's most, its T3's size (David, 2026-10-10: "I am constantly getting 4 excercises in balanced
-- settings. But it says max 3 per quests and I agree"; a Balanced T3 gave the major move and its three until then):
-- a Balanced T1 is the major move and one, a T3 the major move, one and the window, and Light, whose most is one,
-- gives no major move.
local function quota(pace, tier)
	local sizes = C.pace_moves[pace]
	local size = sizes[math.max(1, tier)]
	local allout = tier >= 3 and size > 1
	local window = allout and 1 or 0
	local withMajor = math.min(size + 1, sizes[3])
	return size - window, allout, withMajor > 1 and withMajor - 1 - window or nil
end

-- A move the player started from a step's card or a move's window (slice 36, `chosen` = { move, allout, rung, ask }, as
-- ns.StartFor names it; `ask` a check's least answer, 2026-10-16): the pump's one slot. The oldest owed row of that
-- move and shape that rides now pays it (riderSlot: a steady and an all-out row of one form differ, and a row only at
-- the start's rung, so the start's rung decides the climb's credit as its card says); else the move as the pumps offer
-- it at that rung (its candidate's ladder, rung, anchor and placing, so the fold credits a climb exactly as for any
-- pick), else at that rung alone, the rung's attempt where it stands for a strength ladder's rung (a placement's max
-- set where one is due, as ns.StartFor's goal says) and none for a step the player has (it never climbs and never steps
-- the ladder down), dosed as Swap doses (slotOf, dose). An all-out start is the window alone, from its go, as a closing
-- window (`closing`: Swap offers it the closing window's forms), an owed window it pays too. A move the filters now
-- refuse (a pain flag set on its Ready view) gives way to its backup as Swap's: an all-out window to another form the
-- cardio rung allows (finisherMove), else step jacks all out, any other move to one at its level (backupMove), else
-- step jacks. Never a rider, the Trial, the major move or the day plan; /qp why names the move the slot holds. A Long
-- Haul (`chosen.haul`, its seconds: ns.HaulStart, slice 55) pays no owed row, runs its own length and asks its pace
-- every time (the slot's `haul` and `pace`), never rationed by the steady ledger (ns.LedgerAsk), with no Keep going.
local function sameShape(r, chosen)
	if r.move ~= chosen.move then return false end
	if chosen.rung ~= nil and r.rung ~= chosen.rung then return false end
	if r.kind ~= K.cardio then return not chosen.allout end
	return isFinisherRow(r) == (chosen.allout == true)
end

local function chosenSlot(ctx, chosen, owed)
	local slot
	for _, r in ipairs(not chosen.haul and owed or {}) do
		if not slot and sameShape(r, chosen) and riderFits(ctx, r) then
			slot = riderSlot(ctx, r, true)
			slot.closing = slot.allout or nil
			markRiders(ctx, { slot })
		end
	end
	if not slot then
		local m = ns.Library[chosen.move]
		local rung = chosen.rung or (chosen.allout and ctx.cardioRung) or m.rung
		local c
		if not chosen.allout then
			for _, list in ipairs({ ctx.pool, ctx.owedOut }) do
				for _, x in ipairs(list) do
					if not c and x.move == chosen.move and (x.rung or m.rung) == rung then c = x end
				end
			end
		end
		if not c then
			c = { move = chosen.move, ladder = chosen.allout and ns.CARDIO or m.ladder, rung = rung }
			-- a move standing for its strength ladder's rung is that rung's attempt, as an owed one is (riderSlot)
			if ns.STRENGTH[c.ladder] and rung == ctx.rung[c.ladder] and ns.StandsFor(c.ladder, rung, c.move) then
				c.anchor, c.placing = true, ns.Placing(ctx.state, c.ladder, ctx.now) or nil
			end
		end
		take(ctx, c)
		slot = slotOf(c, 0, 1)
		if chosen.allout then slot.kind, slot.allout, slot.closing = K.cardio, true, true end
		dose(ctx, slot, 0)
	end
	-- a check (ns.StartFor's "check", 2026-10-16): its pose asks its goal pose at the least answer the check counts
	if chosen.ask and slot.kind == K.flow then slot.mark, slot.ask = true, chosen.ask end
	if not ns.Passes(slot.move, ctx.f) then
		local backup
		if slot.allout then
			local n = #ctx.why -- finisherMove notes its pick; the started move's line is the one /qp why reads
			local form = finisherMove(ctx, nil, nil, ownMin(ctx)) or finisherMove(ctx, nil, nil)
			for i = #ctx.why, n + 1, -1 do ctx.why[i] = nil end
			backup = swapSlot(ctx, slot, { move = form or ns.FLOOR_MOVE, ladder = ns.CARDIO, rung = slot.rung })
		else
			local other, r = backupMove(ctx, slot, function() return true end, { atLevel = true })
			backup = other and swapSlot(ctx, slot, { move = other, ladder = ns.Library[other].ladder or 0, rung = r })
				or ns.FloorSlot(ctx.state, slot, ctx.now)
		end
		slot = backup
	end
	if chosen.haul and slot.kind == K.cardio and not slot.allout then
		slot.seconds, slot.haul, slot.pace, slot.keep = chosen.haul, true, true, nil
	end
	why(ctx, { move = slot.move, chosen = true, removed = {} })
	return slot
end

-- Expeditions (slice 51; Rapha, 2026-10-20 and 2026-10-21: "select how much time you have", "warmup and cooldown") ---

-- The stages of an expedition's rounds: the warm-up first, inner rounds, the last round, the cool-down after it.
ns.EXPEDITION_STAGES = { "warmup", "inner", "last", "cooldown" }
local EASED = { warmup = "expedition_warmup_blocks", cooldown = "expedition_cooldown_blocks" }

-- A round's tier by its stage: an inner round the Normal pump (expedition_round_tier), the last the Big pump, whose
-- finisher (or, where no finisher comes, the closing window) is the expedition's one all-out effort; the warm-up and
-- the cool-down a Small pump's.
function ns.ExpeditionTier(stage)
	if stage == "inner" then return C.expedition_round_tier end
	if stage == "last" then return 3 end
	return 1
end

-- A timed move: no rep or hold, so no record attempt (a warm-up's or a cool-down's move).
local function timed(_, c)
	return not recordMove(ns.Library[c.move])
end

-- An expedition's warm-up or cool-down: in order, one move of each block of expedition_warmup_blocks (mobility, then
-- cardio) or expedition_cooldown_blocks (yoga, then mobility), the day plan's own pick inside the block (planned: a
-- waiting stretch check, the pose family that waited longest, a cardio form of the player's step) among the timed moves
-- alone (no record attempt, no major move, no all-out window), dosed as any pick at no active minutes (a steady cardio
-- move, a pose's own seconds). A block the place or the Body switches shut gives no move, and the cool-down adds no
-- move past expedition_cooldown_s by the estimate (ns.PumpTime), its first aside, so stretches held per side never eat
-- a short expedition.
local function easeRound(ctx, stage, roll)
	local slots, taken = {}, {}
	for _, block in ipairs(C[EASED[stage]]) do
		local c = planned(ctx, nil, taken, roll, timed, block)
		if c then
			take(ctx, c)
			local slot = dose(ctx, slotOf(c, 0, ns.ExpeditionTier(stage)), 0)
			if stage == "cooldown" and #slots > 0
				and ns.PumpTime(slots) + ns.PumpTime({ slot }) > C.expedition_cooldown_s then
				break
			end
			slots[#slots + 1] = slot
			taken[ns.Library[c.move].pattern] = true
		end
	end
	return slots
end

-- The regions an expedition's rounds took after a round: `regions` (the rounds' before it, a list) and the region of
-- each record move among `slots`, the round's own moves (as ns.Compose's take holds them), sorted; a new list. The next
-- round's input.expedition.regions.
function ns.RoundRegions(regions, slots)
	local seen, out = {}, {}
	for _, region in ipairs(regions or {}) do seen[region] = true end
	for _, slot in ipairs(slots) do
		local m = ns.Library[slot.move]
		if recordMove(m) and m.region and not slot.owed then seen[m.region] = true end
	end
	for region in pairs(seen) do out[#out + 1] = region end
	table.sort(out)
	return out
end

-- An expedition round's compose input: `input`'s state, profile, clock, P and app, the round's `stage` and the regions
-- the rounds before it took.
local function roundInput(input, stage, regions)
	return { kind = "quest", state = input.state, profile = input.profile, now = input.now, P = input.P,
		app = input.app, expedition = { stage = stage, regions = regions } }
end

-- A round's time with the breather before it (expedition_rest_s), none for a round with no move.
local function withRest(slots)
	return #slots > 0 and C.expedition_rest_s + ns.PumpTime(slots) or 0
end

-- The next round of an expedition, from the real clock, before each round (slice 51, "Before each round the engine
-- decides"): `input` the rounds' compose input (state, profile, now, P, app: ns.Compose's), `exp` = { budget (the
-- length chosen, seconds), elapsed (seconds since the expedition began), last (the stage of the round that ran last;
-- nil before the first), regions (ns.RoundRegions' after it) }. The warm-up always comes first, skipped where it has
-- no move; after it or an inner round the next is the last round when the time so far and what is left (an inner round,
-- the last and the cool-down, each by its estimate, ns.PumpTime, after its breather) pass the budget, else an inner
-- round; the cool-down follows the last round, skipped where it has no move. No round is cut short, and the last round
-- and the cool-down always come: an expedition ends past its budget where its rounds took longer than their estimates.
-- Returns the stage and its round's slots (ns.Compose's), nil when the expedition is done.
function ns.ExpeditionNext(input, exp)
	local function round(stage)
		return ns.Compose(roundInput(input, stage, exp.regions))
	end
	if exp.last == "cooldown" then return nil end
	if exp.last == "last" then
		local cool = round("cooldown")
		if #cool == 0 then return nil end
		return "cooldown", cool
	end
	if exp.last == nil then
		local warm = round("warmup")
		if #warm > 0 then return "warmup", warm end
	end
	local inner, final = round("inner"), round("last")
	if exp.elapsed + withRest(inner) + withRest(final) + withRest(round("cooldown")) > exp.budget then
		return "last", final
	end
	return "inner", inner
end

-- input: { kind, quests = { { questID, tier, family, share = { active, foot, mount, swim }, themes } }, owed (oldest
-- first, reserved ones removed), state, profile, now, pace? (the profile's when nil), app? (an app pump, whose roll
-- counts every source's pumps of the day: context), chosen? (a started move: chosenSlot, whatever the kind), warm? (a
-- match pump after match_sat_min minutes sat, slice 48: its first move the day plan's where its block holds a warmable
-- one, warmPick, its stretches last), arms? (the arms rest, slice 48: ns.ArmsRest's filters and no all-out window),
-- expedition? (an expedition's round, slice 51: { stage, regions }, one of ns.EXPEDITION_STAGES and the regions its
-- earlier rounds took, ns.RoundRegions; its quests are its stage's, never input.quests: the warm-up and the cool-down
-- easeRound's, an inner round the Normal pump with no closing window, the last the Big pump) }; a quest's
-- `share.active` is in seconds.
-- The game's own pumps set neither warm, arms nor expedition (tools/apppump.lua's match pumps and expeditions do).
-- Returns the pump's slots in order: { move, kind, questID, tier, ladder, rung, target?, goal?, load?, loadUp?,
-- loadFrom?, maxSet?, placing?, hi?, cap?, seconds?, allout?, closing?, counted?, owed?, standup?, practice?, trial?,
-- mark?, ask?, pace?, keep?, warm?, haul? }, with `.why` holding what /qp why prints (its `pace`: the pace, Moves per
-- quest, that sized a quest's moves). The slot's `pace` (slice 42, 2026-10-17): the steady move after which the pump
-- asks "Did you keep that pace to the end?" (paceSlot); `keep` (slice 52): the full move's seconds Keep going runs a
-- steady move on to (keepSlot); `warm` (slice 48's double-check): a match pump's warm move, dosed easy (dose); `haul`
-- (slice 55): a Long Haul, its seconds its own length (chosenSlot).
function ns.Compose(input)
	if input.kind == "practice" then
		local slot = { move = PRACTICE, kind = K.rep, questID = 0, tier = 0, ladder = E.ladder.push, rung = 1,
			practice = true, limit = C.try_s }
		return { slot }
	end
	local ctx = context(input)
	if input.kind == "standup" then -- 60 s of step jacks, no ladder credit
		local slot = { move = ns.FLOOR_MOVE, kind = K.cardio, questID = 0, tier = 0, ladder = 0, rung = 1,
			standup = true, seconds = C.standup_s }
		return { slot }
	end
	ctx.pool, ctx.owedOut = {}, {}
	for _, c in ipairs(candidates(ctx)) do
		local list = ctx.owed[c.move] and ctx.owedOut or ctx.pool
		list[#list + 1] = c
	end
	if input.chosen then
		local slots = { chosenSlot(ctx, input.chosen, input.owed) }
		slots.why = ctx.why
		return slots
	end
	-- an expedition's round (slice 51) pays no quest: the player's state alone picks, as for the Quick pump, at its
	-- stage's tier (ns.ExpeditionTier); its warm-up and cool-down are easeRound's
	local exp = input.expedition
	local slots, own, finisher, warmSlot = {}, {}, nil, nil
	local quests = exp and { { questID = 0, tier = ns.ExpeditionTier(exp.stage),
		share = { active = 0, foot = 0, mount = 0, swim = 0 } } } or input.quests or {}
	local first = quests[1]
	local roll = roller(first and first.questID or 0, ctx.day, ctx.pumpNo)
	if exp and EASED[exp.stage] then
		slots = easeRound(ctx, exp.stage, roll)
		slots.why = ctx.why
		return slots
	end

	if input.kind == "demand" and input.owed and #input.owed > 0 then
		-- every owed move that fits, spread after the player's latest set (spreadRiders), every all-out window after
		-- the plain moves, oldest first (pickRiders' rule)
		local plain, windows = pickRiders(ctx, input.owed)
		slots = spreadRiders(plain, ctx.state.lastMove)
		for _, slot in ipairs(windows) do slots[#slots + 1] = slot end
		if #slots > 0 then
			markRiders(ctx, slots)
			slots.why = ctx.why
			return slots
		end -- every owed move waits (a pain flag, a rung): one fresh move instead
	end

	-- Owed moves are taken first, every one, oldest first: they were owed before this pump's own moves, so a fresh pick
	-- never takes their move or a second placement; they hold no region (take: a banked move counts as done for the
	-- picks, as it does for its ladder's turn and the day plan's shares). They ride after the pump's own moves
	-- (docs/plan.md slice 1, "Why after the pump's own moves"), the plain ones first, spread after the pump's own last
	-- move (spreadRiders), then every owed all-out window that fits, which end the pump in place of its own; an owed pose
	-- asks once the pump's own moves took their asks (markRiders). One whose move the pump's own moves run waits for a
	-- later pump (notOwn).
	local plain, windows = pickRiders(ctx, input.owed)

	local function add(c, questID, tier, active)
		if not c then c = { move = ns.FLOOR_MOVE, ladder = ns.CARDIO, rung = ctx.cardioRung, due = 0 } end
		take(ctx, c)
		local slot = slotOf(c, questID, tier)
		slot.active = active
		own[#own + 1] = slot
		return slot
	end

	if input.kind == "demand" then
		-- nothing owed: a record attempt on the ladder that has waited longest, and a major move beside it when
		-- another ladder has gone major_h (slice 1); it pays no quest, so it leaves the stretch to the next pump that
		-- does and doses a cardio move at no active minutes
		add(dueSlot(ctx, roll), 0, 1, 0)
		local second = majorSlot(ctx, nil, nil)
		if second then
			own[1].tier = 2
			markMajor(ctx, add(second, 0, 2, 0))
		end
	elseif #quests > 0 then
		local top = quests[1]
		for _, q in ipairs(quests) do
			if q.tier > top.tier then top = q end
		end
		local topActive = top.share and top.share.active or 0
		local taken = {}
		-- Light gives no major move (quota): the quest's one move is the day plan's, whose strength turn goes to the
		-- ladder that has waited longest, as the major move does (made the one move, the major move took 79-94 % of
		-- Light's pumps and left an evening all strength; a simulator probe, 2026-10-05)
		local n, allout, beside = quota(ctx.pace, top.tier)
		-- the arms rest (slice 48) has no all-out window: its place goes to one more move of the day plan
		if ctx.arms and allout then n, allout, beside = n + 1, false, beside and beside + 1 end
		-- a match pump after a long sit (ctx.warm, slice 48) opens on its warm move (warmPick), one of its own moves, so
		-- their count stays: the major move second, the all-out window last. A Trial step comes beside it where the pace
		-- has room for a record move; under Light, whose most is one, the warm move alone, the Trial the next pump's
		local trial = not (ctx.warm and beside == nil) and trialSlot(ctx, top.questID) or nil
		if trial then
			trial.active = topActive
			if ctx.warm then
				taken[ns.Library[trial.move].pattern] = true
				warmSlot = add(warmPick(ctx, top, taken, roll), top.questID, top.tier, topActive)
			end
			own[#own + 1] = trial
		else
			local major = beside and majorSlot(ctx, top, roll) or nil
			if major then
				local slot = add(major, top.questID, top.tier, topActive)
				markMajor(ctx, slot)
				taken[ns.Library[slot.move].pattern] = true
				n = beside
			end
			for k = 1, n do
				local warm = ctx.warm and k == 1
				local c
				if warm then c = warmPick(ctx, top, taken, roll) else c = planned(ctx, top, taken, roll) end
				local slot = add(c, top.questID, top.tier, topActive)
				taken[ns.Library[slot.move].pattern] = true
				if warm then -- picked after the major move, which keeps its draw, and put before it
					warmSlot = slot
					table.insert(own, 1, table.remove(own))
				end
			end
			if allout and #windows == 0 then
				finisher = finisherSlot(ctx, top, top.questID, top.tier, roll, taken)
			end
		end
		-- the warm move is dosed easy (slice 48's double-check: dose, paceSlot), whatever its kind
		if warmSlot then warmSlot.warm = true end
		-- a merged quest past the first gives the T1 size; its window and the major move are the first quest's
		for _, q in ipairs(quests) do
			if q ~= top then
				for _ = 1, C.pace_moves[ctx.pace][1] do
					local slot = add(planned(ctx, q, taken, roll), q.questID, q.tier, q.share and q.share.active)
					taken[ns.Library[slot.move].pattern] = true
				end
			end
		end
		-- a match pump's stretching and yoga moves come after its other own moves, in their order (2026-11-03, Rapha,
		-- after a match pump opened on the standing quad stretch: a static stretch belongs on muscles the moves before
		-- it warmed, never first after the sit)
		if ctx.warm then
			local moving, poses = {}, {}
			for _, slot in ipairs(own) do
				local block = ns.Block(slot.move)
				local into = (block == "mobility" or block == "yoga") and poses or moving
				into[#into + 1] = slot
			end
			for _, slot in ipairs(poses) do moving[#moving + 1] = slot end
			for i, slot in ipairs(moving) do own[i] = slot end
		end
		-- the pace that sized a quest's moves (a Trial step has none of its own), for /qp why
		if not (trial and #quests == 1) then ctx.why.pace = ctx.pace end
	end

	-- From cardio rung closing_rung (5 since 2026-10-05, when the counted sets came; 7 until then) the first steady
	-- cardio move of a pump without an all-out window (its own or an owed one) becomes the all-out window of the rung's
	-- form, the closing window: only the pump's last effort may run all out, so it moves to the end, dosed as a finisher
	-- (one move, one exercise since 2026-10-15; it ran the steady move's time and then the form until then); any other
	-- stays steady. Where the steady ledger's ask is due (asks, ns.LedgerAsk; slice 52, a timed rung's pace ask from
	-- 2026-10-17, slice 42's review: the closing window took the one steady move of every pump that was not a T3's, and
	-- the step's ready days almost never came), the first steady move is the one that asks, so it stays and the next one
	-- becomes the window, or none does. A match pump's warm cardio move (slice 48) stays steady and easy and never asks
	-- the pace (paceSlot), so the next steady move asks it where the ask is due, and the arms rest has no window. An
	-- expedition's inner round has none (slice 51: its one all-out effort is its last round's).
	local closing
	if not finisher and #windows == 0 and not ctx.arms and ctx.cardioRung >= C.closing_rung
		and (not exp or exp.stage == "last") then
		local asking = true
		for i, slot in ipairs(own) do
			local cardio = slot.kind == K.cardio and not slot.standup and slot ~= warmSlot
			if cardio and asking and asks(ctx, slot) then
				asking = false
			elseif cardio then
				local steady = slot.move
				local form, eligible = provingMove(ctx)
				if not form then form, eligible = finisherMove(ctx, first, roll, ownMin(ctx)) end
				if not form then form, eligible = finisherMove(ctx, first, roll) end
				if form then
					ctx.used.moves[form] = true
					slot.move, slot.allout, slot.closing, slot.eligible = form, true, true, eligible
					closing = table.remove(own, i)
					-- /qp why names the moves the pump runs: the steady pick gives its line up to the window's
					-- (finisherMove's, the last), which keeps the day plan's block that pick filled
					for k = #ctx.why - 1, 1, -1 do
						local w = ctx.why[k]
						if w.move == steady and not w.finisher then
							ctx.why[#ctx.why].block = w.block
							table.remove(ctx.why, k)
							break
						end
					end
				end
				break
			end
		end
	end

	-- a quest's active minutes count once: its own steady cardio moves share them, so an hour of play stays within its
	-- bound
	local cardioOf = {}
	for _, slot in ipairs(own) do
		if slot.kind == K.cardio then cardioOf[slot.questID] = (cardioOf[slot.questID] or 0) + 1 end
	end
	for _, slot in ipairs(own) do
		slots[#slots + 1] = dose(ctx, slot, (slot.active or 0) / (cardioOf[slot.questID] or 1))
	end
	-- an owed move whose move the pump's own moves already run waits for a later pump, still unfinished (Rapha,
	-- 2026-11-02: "it should skip excercises in banked if the excercise of the pump included the same one already"):
	-- the fresh picks never take an owed move's move, so only the major move, a Trial step, the warm move or the
	-- closing window can share one
	local ownMoves = {}
	for _, slot in ipairs(slots) do ownMoves[slot.move] = true end
	if finisher then ownMoves[finisher.move] = true end
	if closing then ownMoves[closing.move] = true end
	plain, windows = notOwn(plain, ownMoves), notOwn(windows, ownMoves)
	plain = spreadRiders(plain, own[#own] and own[#own].move)
	markRiders(ctx, plain)
	for _, slot in ipairs(plain) do slots[#slots + 1] = slot end
	if finisher or closing then slots[#slots + 1] = dose(ctx, finisher or closing) end
	for _, slot in ipairs(windows) do slots[#slots + 1] = slot end
	for _, slot in ipairs(slots) do slot.active = nil end
	slots.why = ctx.why
	return slots
end

-- Explain ------------------------------------------------------------------------------------------------------------

local function pct(v)
	return math.floor((v or 0) * 100 + 0.5)
end

-- A quest's lines for /qp why: its score parts and tier, the quest table's mapping (or that it has none) and the
-- flavour line. q: { questID, score, tier, parts, family, info } (info.mapped from ns.Mapping).
local function questLines(q, lines)
	local p = q.parts
	if p then
		lines[#lines + 1] = L["why.quest"]:format(q.questID, pct(q.score), q.tier, L["family." .. (q.family or "talk")],
			pct(p.xp), pct(p.lvl), pct(p.tag), pct(p.obj), pct(p.trav))
	end
	local m = q.info and q.info.mapped
	if not m then
		lines[#lines + 1] = L["why.unmapped"]:format(q.questID)
		return lines
	end
	local extras = {}
	if m.boss then extras[#extras + 1] = L["why.mapped.boss"] end
	if m.fin then extras[#extras + 1] = L["why.mapped.finale"] end
	if m.group then extras[#extras + 1] = L["why.mapped.group"] end
	if #m.th > 0 then
		local names = {}
		for i, theme in ipairs(m.th) do names[i] = L["theme." .. theme] end
		extras[#extras + 1] = L["why.mapped.themes"]:format(table.concat(names, ", "))
	end
	lines[#lines + 1] = L["why.mapped"]:format(L["family." .. m.f], m.lvl or L["why.mapped.unknown"],
		#extras > 0 and "; " .. table.concat(extras, ", ") or "")
	local quest = ns.Lines(q.questID) or {}
	for _, kind in ipairs(ns.LINE_KINDS) do
		local line = quest[K[kind]]
		if line then lines[#lines + 1] = L["why.flavour"]:format(L["flavour.kind." .. kind], L["ui.flavour"]:format(line)) end
	end
	return lines
end

-- /qp why <questID>: one quest's lines, in the log or not.
function ns.WhyQuest(q)
	return questLines(q, {})
end

-- The lines /qp why prints for a pump: each quest's lines, the pace when not Balanced, then per move the day plan's
-- block it filled, its five scores, the next best candidates and the removed ones with their reasons.
function ns.Why(pump)
	local lines = {}
	for _, q in ipairs(pump and pump.quests or {}) do questLines(q, lines) end
	local whys = pump and pump.moves and pump.moves.why or {}
	-- the pace that sized a quest's moves (ns.Compose's why.pace): nothing for the default, Balanced, the tiers' own
	if whys.pace and whys.pace ~= ns.defaults.pace then lines[#lines + 1] = L["why.pace." .. whys.pace] end
	for _, w in ipairs(whys) do
		local name = L["move." .. ns.Library[w.move].key]
		if w.trial then
			lines[#lines + 1] = L["why.trial"]:format(name)
		elseif w.chosen then
			lines[#lines + 1] = L["why.chosen"]:format(name)
		elseif w.proving then
			lines[#lines + 1] = L["why.proving"]:format(name)
		else
			local p = w.parts or {}
			if w.block then lines[#lines + 1] = L["why.block"]:format(L["block." .. w.block]) end
			lines[#lines + 1] = L[w.finisher and "why.finisher" or "why.move"]:format(name, pct(w.score), pct(p.due),
				pct(p.recovered), pct(p.variety), pct(p.fit), pct(p.theme))
			local next = {}
			for i = 2, 3 do
				if w.top and w.top[i] then next[#next + 1] = L["move." .. ns.Library[w.top[i]].key] end
			end
			if #next > 0 then lines[#lines + 1] = L["why.next"]:format(table.concat(next, ", ")) end
		end
		local removed = {}
		for i = 1, math.min(4, #(w.removed or {})) do
			local r = w.removed[i]
			removed[#removed + 1] = L["move." .. ns.Library[r.move].key] .. " (" .. L["why.reason." .. r.reason] .. ")"
		end
		if #removed > 0 then lines[#lines + 1] = L["why.removed"]:format(table.concat(removed, ", ")) end
	end
	return lines
end
