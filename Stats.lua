-- The sheet's numbers, pure (docs/plan.md slice 3): five stats by kind of move, the familiarity track, XP and levels
-- with the streak and rested XP, rest tokens, the weekly vault, titles, the ratings, the body map
-- and the skills. Nothing here is stored: DB.lua's fold hands every row to ns.SheetApply in order, which keeps its
-- running totals in state.sheet (they survive the ring in the baseline like the rest of the fold), and the readers
-- below turn them into what the sheet, the HUD and the pump card show. Time comes only from the rows and ns.now().

local ADDON, ns = ...
local C, E = ns.C, ns.Enum
local K, O, Q = E.kind, E.outcome, E.quality

local Stats = {}
ns.Stats = Stats

local WEEK = 7 * 86400

-- Forever's order (the Character window's Primary Attributes, "must match UNITSTAT order": Gethe `forever`
-- Camelot/PaperDollFrameConstants.lua, 70170; Stamina before Intellect since 2026-10-01), which also breaks ties.
ns.STATS = { "strength", "agility", "stamina", "intellect", "spirit" }
local STAT_ORDER = {}
for i, key in ipairs(ns.STATS) do STAT_ORDER[key] = i end

-- The two-stat beasts of Classic's random item suffixes, keyed by the pair in Forever's order.
local BUILD = {
	strength_agility = "tiger", strength_stamina = "bear", strength_intellect = "gorilla", strength_spirit = "boar",
	agility_stamina = "monkey", agility_intellect = "falcon", agility_spirit = "wolf", stamina_intellect = "eagle",
	stamina_spirit = "whale", intellect_spirit = "owl",
}

-- The glossary's muscles (Library.lua's mp / ms ids): 1-16 on the paper doll, 17 Heart, 18 Breath, 19 Balance.
ns.MUSCLES = 19
local HEART = 17 -- 18 Breath and 19 Balance colour as every muscle does since slice 21

local FAMILIAR, LEARNED = C.fam_familiar_days, C.fam_learned_days

-- A familiarity step by its finished days: 1 tried (its first), 2 familiar, 3 learned; nil between them.
local function famLevel(n)
	return n == 1 and 1 or n == FAMILIAR and 2 or n == LEARNED and 3 or nil
end

-- Whether a move's familiarity step `level` (famLevel) pays its stat, and with `mark` the fold's note that it paid. A
-- pose family's step (ns.POSE) pays each step once, on the first of its poses to reach it, and its other poses reach
-- theirs in words alone (their line in the Book, the pump's familiarity count): a step's poses are one skill in its
-- forms, any goal pose of them counting the family's day (slice 47). The cardio ladder's step does the same since slice
-- 54, whose fair draw runs every form of the step its turn: the step's forms are one skill, any of them counting its
-- clean day or full move. A move of any other family pays its own, as its own record and top stand beside its step's
-- others. Per form, the three to four poses slice 47 put on every step paid Spirit and Intellect, and slice 54's five
-- roads the cardio family's Agility, by the library's breadth, not the player's climb (docs/plan.md slices 47 and 54,
-- "The stat economy's call").
local function famPays(sh, m, level, mark)
	if not (m.ladder and (ns.POSE[m.ladder] or m.ladder == ns.CARDIO)) then return true end
	local key = ns.RungKey(m.ladder, m.rung)
	if (sh.famSteps[key] or 0) >= level then return false end
	if mark then sh.famSteps[key] = level end
	return true
end

-- Days and weeks --------------------------------------------------------------------------------------------------

-- A game day ("YYYY-MM-DD", ns.day) as a day number (days since 1970-01-01, the civil calendar), so gaps and windows
-- are plain subtraction.
function ns.DayIndex(day)
	local y, m, d = day:match("^(%d+)-(%d+)-(%d+)$")
	y, m, d = tonumber(y), tonumber(m), tonumber(d)
	if m <= 2 then y = y - 1 end
	local era = math.floor(y / 400)
	local yoe = y - era * 400
	local doy = math.floor((153 * ((m + 9) % 12) + 2) / 5) + d - 1
	return era * 146097 + yoe * 365 + math.floor(yoe / 4) - math.floor(yoe / 100) + doy - 719468
end

local function dayIndex(ts)
	return ns.DayIndex(ns.day(ts))
end
Stats.DayIndex = dayIndex

-- The weekly reset: an instant the region reset at, read from the client at login (C_DateAndTime), rounded to the
-- hour; until then the US reset, Tuesday 15:00 UTC. A week is the time between two resets. Outside WoW (ns.weekByDay:
-- the app's own sheet, tools/apppump.lua and the app player's world of tools/simulator.lua) the week is Monday's game
-- day to Sunday's (decision of 2026-10-06), as the phones count it: it turns on Monday at the day's turn, local time,
-- whatever a clock change did, numbered from the week of 1970-01-01 (day 0, a Thursday, so a Monday is day 4 of 7).
ns.DEFAULT_RESET = 1790089200 -- Tuesday 2026-09-22 15:00 UTC

function ns.WeekOf(ts)
	if ns.weekByDay then return math.floor((dayIndex(ts) + 3) / 7) end
	return math.floor((ts - (ns.weekAnchor or ns.DEFAULT_RESET)) / WEEK)
end

-- The next reset after `now`: outside WoW the next Monday's game day's first second (its middle, read again on that
-- day so a clock change between lands on its turn, less 12 hours).
function ns.NextReset(now)
	if ns.weekByDay then return ns.DayMid(ns.DayMid(now, 7 - (dayIndex(now) + 3) % 7)) - 12 * 3600 end
	return (ns.weekAnchor or ns.DEFAULT_RESET) + (ns.WeekOf(now) + 1) * WEEK
end

-- When the week holding `now` began: the last reset, outside WoW its Monday's game day's first second (as above).
function ns.WeekStart(now)
	if ns.weekByDay then return ns.DayMid(ns.DayMid(now, -((dayIndex(now) + 3) % 7))) - 12 * 3600 end
	return ns.NextReset(now) - WEEK
end

-- The stat a ladder feeds (ns.Ladders' `stat`, slice 21: Strength push, legs, the pull-up, the row, the hinge, the
-- lower back and the planche; Agility cardio and power; Stamina the core, the side, the wall sit, the L-sit and the
-- sit-up; Intellect balance, the arm balances and the handstand; Spirit the splits, the pike, the backbend, twists and
-- ankles): its stars and rungs pay it, and so does every move it holds.
function ns.LadderStat(ladder)
	local lad = ns.Ladders[ladder]
	return lad and lad.stat
end

-- The stat a move raises: its family's (its ladder's); a move no ladder holds by its pattern, else by the ladder it
-- scales by, else Spirit (ns.C.stat_rule). So two players who climb different families end on different stats.
function ns.MoveStat(moveId)
	local m = ns.Library[moveId]
	if m.ladder then return ns.LadderStat(m.ladder) end
	local rule = C.stat_rule
	if rule[m.pattern] then return rule[m.pattern] end
	local scale = ns.ScaleLadder(m)
	return scale and ns.LadderStat(scale) or rule.other
end

-- The familiarity step of a move: 0 new, 1 tried, 2 familiar, 3 learned; and its distinct days.
function ns.Familiarity(state, moveId)
	local f = state.sheet and state.sheet.fam[moveId]
	local n = f and f.n or 0
	return n >= LEARNED and 3 or n >= FAMILIAR and 2 or n >= 1 and 1 or 0, n
end

-- The fold's side --------------------------------------------------------------------------------------------------

function Stats.New()
	local stats = {}
	for _, key in ipairs(ns.STATS) do stats[key] = C.stat_base end
	return {
		stats = stats, base = CopyTable(stats), week = nil, last = {},
		-- xp: lifetime (each game day's share in its day's xp, below); pool: rested XP; restTs: the last performed move,
		-- from which time away fills the pool; heldFrom: the first game day (an index) whose xp the day map holds whole,
		-- nil while it holds every day (Stats.HoldFrom)
		xp = 0, pool = 0, restTs = nil, heldFrom = nil,
		fam = {},        -- [move] = { n = distinct days, last = day index }
		famSteps = {},   -- [ns.RungKey] = the familiarity step a pose or cardio step paid last (famPays; slices 47, 54)
		patterns = {},   -- [pattern] = true once done
		rungs = {},      -- [ns.RungKey] = its first date once reached by a climb from any source, paid once (true in
		                 -- older saves)
		earned = {},     -- [ns.RungKey] = the date a set earned an orange or gold rung its climb opened (2026-10-09)
		-- [day index] = { rh, wins, active, rested, int, agi, stars; xp, the XP gained on that game day by each gain's
		-- own row time, its rested bonus in it (the Today tab, the day card, the summary's dx; contract/fixtures/
		-- history.json saga rule 15); pumps (counted at a pump's first fresh move, as the titles count them,
		-- ns.CountedPumps) and started (those with no quest, the player's own), cardio (seconds), reps = { [pattern] =
		-- reps }, finishers: the day's tally (ns.DayTally) and a duel's day (ns.DuelScore); pumpsFinished and
		-- pumpsBanked, the pumps offered that day (the feedback numbers, countPump); and the Saga's day (sagaMove and a
		-- climb's first reach in ns.SheetApply; kept for today and the day before, Stats.Prune): bars = { { ts (the
		-- pump's start), n (its moves that day), source (ns.Enum.source: where they were done) } } in the order of
		-- their first move, regions = { [region] = moves }, sets = { [move] = { v, load } } the day's best typed set of
		-- each move, was = { [move] = { v, load, reps } or false } its record at the day's start, records = { move ... }
		-- the moves stored as a record in the order of their first (a corrected set is none), appRecordMoves =
		-- { [move] = true } those with a record of the app's (DB.lua's appRecords counts the app's record rows),
		-- unlocks = { { ts, ladder, rung } } the rungs first reached }
		days = {},
		-- every player starts holding token_start rest tokens (slice 51, the streak's freeze)
		activeIdx = nil, streak = 0, best = 0, tokens = C.token_start, activeDays = 0,
		-- the two best game days in moves from every source, { moves, day ("YYYY-MM-DD") }, the most first and of two
		-- alike the earlier (raiseTop): with DB.lua's day counts they give the best day ever and the best before a day
		-- (ns.DayRank, 2026-10-15)
		topDays = {},
		-- [day index] = true: the game days after activeIdx with a felt-unwell row and no performed move yet (lockDay),
		-- which the streak skips (activeDay, ns.Streak; the summary's lk)
		locked = {},
		-- the newest streak milestone the fold passed: { days (of C.streak_milestones), day (its game day index), xp (the
		-- day's first-pump XP), mirrored (its day began with a row the game never folds live, an app's or a catch-up's,
		-- which come only through the helper's mirror) }; the game's alone, for the toast a login fold owes (Sheet.lua)
		lastMilestone = nil,
		-- the week's vault: its moves done (2026-10-27; its pumps, `pumps`, until then), records and embers (slice 55; the
		-- cardio seconds, `cardio`, until then), and each goal's tiers paid (`paid`: [goal] = how many, the embers' up to
		-- #vault_embers; true from before slice 55)
		vault = { moves = 0, records = 0, embers = 0 }, paid = {},
		weekPumps = 0,   -- pumps counted this week, at their first fresh move: the card's pumps (ns.Week)
		weekBeats = 0,   -- records beaten this week (the vault's records count stars too): the card's R
		weekCardio = 0,  -- cardio seconds this week, from the timer: the card's cardio minutes (ns.Week)
		weekKinds = {},  -- [stat] = true once the week's practice point of that kind paid
		weekLearn = false, -- the week's learning point paid
		-- the game's or a catch-up's pump the rows belong to: { ts, questID, tier, quests, ding, performed, banked,
		-- travel } and what it paid so far, for the sheet summary's last pump (ns.LastPump): xp, pts (stat points) and
		-- stats ([stat] = points), stars, fam (familiarity steps), unlocks ({ ladder, rung, rekindled, opened: an orange
		-- or gold rung its climb only opened, earned: one a set earned } in the order they came) and its game day's start
		-- (streakDay: the streak's day it began; welcome: the XP Welcome back paid; token: a rest token earned; rested:
		-- the game days rest tokens covered; milestone: the one of C.streak_milestones the streak reached); a pump from
		-- before 2026-10-06 lacks the unlocks and the day's fields,
		-- one from before slice 32 the milestone
		pump = nil,
		-- the app pump an app row belongs to (slice 27, 2026-10-06), of the same shape: the app's moves pay the sheet as
		-- a catch-up's do, on it and never on the game's pump above (a slice 26 baseline kept its time here: DB.lua
		-- reads that as none)
		appPump = nil,
		counts = { pumps = 0, t3 = 0, merged = 0, beats = 0, clean = 0, squats = 0, wallsit = 0, finishers = 0,
			travelCardio = 0, foot = 0, dings = 0, stars3 = 0, placed = 0, kindled = 0 },
		weekSets = {},   -- [ladder] = hard sets (rep, hold and counted moves) this week
		climbs = {},     -- [ladder] = rung-ups
		legacy = {},     -- [title id] = true: a retired title its counters reached in rows before titles_retired_ts
		units = {},      -- [pattern] = lifetime reps or seconds (the skills)
		muscles = {},    -- [muscle] = { wk, week, hard, step (the best step trained, its colour), rung and best (that
		                 -- step's rung and move), last = { { move, ts } ... } }
		-- the last 20 pumps: { ts, questID, tier, ding, death, level, rank, moves = { { move, out (its row's outcome), v
		-- (a done move's count or seconds), count (a counted set), key and fate (an unfinished one's queue key until its
		-- end, "later" or "skipped") } } }; an entry from before 2026-10-05 holds { move, done } only
		log = {},
		-- [owed key] = "later" or "skipped": a paid row the fold saw before its owed row (DB.lua `paidEarly`), the fate
		-- that owed row's Log move takes; it goes when that row comes, as the fold's own mark does (logMove)
		paidEarly = {},
		lastMove = nil,  -- { ts, idx, win }: a rung-up row right after it makes it a win
		critR = 0,       -- the critical strike rating: each game day's first crit_day_wins wins, for good
	}
end

-- The live buffer: while ns.AddRow applies a row, the gains land in state.live for the toasts and the pump card.
local function emit(s, e)
	if s.live then s.live[#s.live + 1] = e end
end

-- Whether the row ns.SheetApply folds is an app row (slice 27): its gains land on the app pump (sheet.appPump) and its
-- moves join that pump's Log entry, a game or catch-up row's on sheet.pump. ns.SheetApply sets it first, before any
-- gain below reads it.
local fromApp = false

-- The pump the row being folded belongs to.
local function pumpOf(sh)
	if fromApp then return sh.appPump end
	return sh.pump
end

local function statGain(s, ts, stat, amount, reason, move)
	local sh = s.sheet
	if amount <= 0 or not stat then return end
	sh.stats[stat] = sh.stats[stat] + amount
	sh.last[stat] = { amount = amount, reason = reason, move = move, ts = ts }
	local pump = pumpOf(sh)
	if pump then
		pump.pts = (pump.pts or 0) + amount
		pump.stats[stat] = (pump.stats[stat] or 0) + amount -- the banner's "+1 Strength"
	end
	emit(s, { kind = "stat", stat = stat, amount = amount, reason = reason, move = move })
end

-- The level of a lifetime XP, the XP into it and what the next one needs; at the cap, the Paragon rank (a rank every
-- paragon_xp) with the XP into the next rank and what it needs (ns.Level, the fold's gains).
local function levelOf(xp)
	local level, into = ns.LevelFor(xp)
	if level >= C.level_cap then
		return level, into % C.paragon_xp, C.paragon_xp, math.floor(into / C.paragon_xp)
	end
	return level, into, ns.XPToNext(level), 0
end

-- The Log entry of a pump or of a turn-in owed on its own, by its time; with `app`, of an app pump (slice 26), which
-- no game pump's move ever joins, however close their times, and with `device` too, of that device's app pump.
local function entryAt(sh, ts, app, device)
	for i = #sh.log, 1, -1 do
		local e = sh.log[i]
		if e.ts == ts and (e.source == E.source.app) == (app == true) and (device == nil or e.device == device) then
			return e
		end
	end
end

local function day(sh, idx)
	local d = sh.days[idx]
	if not d then
		d = { rh = 0, wins = 0 }
		sh.days[idx] = d
	end
	return d
end

-- XP, with the streak multiplier and rested XP (plan slice 3, "Level and XP"; David, 2026-10-03: heavy players "ofc
-- always want to get xp"): every gain counts in full, and draws as much again from the rested pool while it lasts. The
-- 600-a-day line went that day: past it a gain counted 0 and went into the pool, the "+0 XP" Blizzard's own beta
-- testers rejected (Pardo, GDC 2010). A QP level, and past the cap each Paragon rank the gain crosses (slice 23: one
-- event a rank, should a gain ever cross two), plays its fanfare and toast (Sheet.Earned) and marks the Log entry of
-- the pump that reached it; that entry adds up the XP its pump paid (xp, since slice 34: the summary's lx). The gain
-- counts on the game day of its own row's time (the day's xp; 2026-10-15, history.json saga rule 15), in whatever
-- order the fold meets the rows (until then a row older than the day last counted counted on that day).
local function gainXP(s, ts, amount, reason)
	local sh = s.sheet
	if amount <= 0 then return end
	local base = amount
	amount = math.floor(amount * (1 + C.streak_step * math.min(sh.streak, C.streak_cap)) + 0.5)
	local bonus = math.min(amount, sh.pool)
	local levelBefore, _, _, rankBefore = levelOf(sh.xp)
	sh.pool = sh.pool - bonus
	local d = day(sh, dayIndex(ts))
	d.xp = (d.xp or 0) + amount + bonus
	sh.xp = sh.xp + amount + bonus
	local level, _, _, rank = levelOf(sh.xp)
	local pump = pumpOf(sh)
	-- the pump's Log entry keeps what the pump paid too (the summary's lx, slice 34), while the Log holds it
	local entry = pump and entryAt(sh, pump.ts, fromApp)
	if pump then pump.xp = (pump.xp or 0) + amount + bonus end
	if entry then entry.xp = (entry.xp or 0) + amount + bonus end
	emit(s, { kind = "xp", amount = amount + bonus, base = base, rested = bonus, reason = reason })
	if level > levelBefore then
		emit(s, { kind = "level", level = level })
		if entry then entry.level = level end
	end
	for r = rankBefore + 1, rank do emit(s, { kind = "paragon", rank = r }) end
	if entry and rank > rankBefore then entry.rank = rank end
	return amount + bonus
end

-- Time away fills the rested pool, as WoW's does: rested_per_day of the level's xpToNext for every day since the last
-- performed move, from a gap of rested_min_h, up to rested_cap_levels of it. The pool a move at `ts` finds: a late row
-- (before the last) adds none.
local function filled(sh, ts)
	local since = sh.restTs and ts - sh.restTs
	if not since or since < C.rested_min_h * 3600 then return sh.pool end
	local need = ns.XPToNext(ns.LevelFor(sh.xp))
	local fill = math.floor(C.rested_per_day * need * since / 86400)
	return math.min(sh.pool + fill, math.floor(C.rested_cap_levels * need))
end

local function rest(s, ts)
	local sh = s.sheet
	sh.pool = filled(sh, ts)
	if not sh.restTs or ts > sh.restTs then sh.restTs = ts end
end

-- The rested XP now: the pool as the next move would find it, with the time away since the last one (the XP bar's
-- rested part, the HUD tooltip and the Today tab; the sheet summary carries the pool and the last move's time, rest and
-- rt, from which the phones work out the same). The fold fills the pool only at a move, so the pool alone reads the
-- rest the last pump left, mostly none.
function ns.Rested(state, now)
	return filled(state.sheet, now or ns.now())
end

-- The feedback numbers' pumps (slice 24, contract/feedback-api.yaml): a pump counts on the game day it was offered,
-- finished from its first fresh move (as the titles count it) unless Bank it put some of its own moves off, which
-- makes it banked.
local function countPump(sh, pump, field, n)
	local d = day(sh, dayIndex(pump.ts))
	d[field] = (d[field] or 0) + n
end

-- A rep or hold move's win on its day: the 28-day count, and the critical strike rating's point while the day has paid
-- fewer than crit_day_wins (so a fifteen-pump day cannot race the rating).
local function addWin(sh, idx)
	local d = day(sh, idx)
	d.wins = d.wins + 1
	if d.wins <= C.crit_day_wins then sh.critR = sh.critR + 1 end
end

-- The step (its colour) a move row stands on and its rung: a cardio row's cardio rung, else the move's own rung on the
-- ladder it scales by.
local function rowStep(r, m)
	local rung = r.kind == K.cardio and r.rung or m.rung or 0
	return ns.StepOn(r.kind == K.cardio and ns.CARDIO or ns.ScaleLadder(m), rung), rung
end

-- The week turns: the stats' weekly base, the vault and the week's practice points start again. (A week
-- whose active days all ended with nothing unfinished paid Spirit 1 until slice 21, 2026-10-04: with the six Spirit
-- families its flat 52 points a year made the Strength focus "of the Boar".)
local function newWeek(sh, wk)
	sh.week, sh.base = wk, CopyTable(sh.stats)
	sh.vault, sh.paid = { moves = 0, records = 0, embers = 0 }, {}
	sh.weekPumps, sh.weekBeats, sh.weekCardio = 0, 0, 0
	sh.weekKinds, sh.weekLearn = {}, false
	sh.weekSets = {}
end

-- At most one point a week for each kind practised (the move's stat), and one to Intellect for a week of learning (a
-- new pattern, a familiarity step, or a pattern's skill rising a point): the stats that rise every week are the kinds
-- a player does, the records, stars, rungs and learned moves on top make the difference.
local function practised(s, ts, stat, move)
	local sh = s.sheet
	if sh.weekKinds[stat] then return end
	sh.weekKinds[stat] = true
	statGain(s, ts, stat, C.stat_practice_week, stat == "spirit" and "active" or "practice", move)
end

local function learned(s, ts, move)
	local sh = s.sheet
	if sh.weekLearn then return end
	sh.weekLearn = true
	statGain(s, ts, "intellect", C.stat_learning_week, "learning", move)
end

-- A pattern's skill from its lifetime units (ns.Skill's curve): min(300, floor(units ^ 0.62)).
local function skillOf(units)
	return math.min(C.skill_cap, math.floor(units ^ C.skill_exp))
end

local VAULT = { moves = "vault_moves", records = "vault_records", embers = "vault_embers" }
local VAULT_ORDER = { "moves", "records", "embers" }
local VAULT_INDEX = { moves = 1, records = 2, embers = 3 } -- a goal's place in VAULT_ORDER (the summary's lv)

-- A vault goal's tiers: its one need, or the embers' three (vault_embers, slice 55). The need of tier `n` (nil past the
-- last) and how many there are.
local function needOf(goal, n)
	local need = C[VAULT[goal]]
	if type(need) == "table" then return need[n] end
	return n == 1 and need or nil
end

local function tiersOf(goal)
	local need = C[VAULT[goal]]
	return type(need) == "table" and #need or 1
end

-- The tiers of a goal paid this week (a goal paid before slice 55 holds true).
local function paidOf(sh, goal)
	local paid = sh.paid[goal]
	return paid == true and 1 or paid or 0
end

-- Each of a goal's tiers reached the first time in the week pays xp_vault_goal, and the pump it belongs to keeps it
-- (the summary's lv: the Result's "Weekly goal reached" row outside the game); every tier of every goal paid is the
-- vault complete. The week's last embers tier counts the Kindled's weeks.
local function vaultAdd(s, ts, goal, amount)
	local sh = s.sheet
	sh.vault[goal] = (sh.vault[goal] or 0) + amount
	local from = paidOf(sh, goal)
	local paid = from
	local need = needOf(goal, paid + 1)
	while need and sh.vault[goal] >= need do
		paid = paid + 1
		sh.paid[goal] = paid
		local pump = pumpOf(sh)
		if pump then
			pump.vault = pump.vault or {}
			pump.vault[#pump.vault + 1] = VAULT_INDEX[goal]
		end
		local xp = gainXP(s, ts, C.xp_vault_goal, "vault")
		emit(s, { kind = "vault", goal = goal, need = need, tier = paid, xp = xp })
		if goal == "embers" and paid == tiersOf(goal) then sh.counts.kindled = sh.counts.kindled + 1 end
		need = needOf(goal, paid + 1)
	end
	if paid == from then return end
	for _, g in ipairs(VAULT_ORDER) do
		if paidOf(sh, g) < tiersOf(g) then return end
	end
	emit(s, { kind = "vaultAll" })
end

-- Whether a performed cardio row lights an ember (slice 55): a cardio move of the ladder done in full, as an all-out
-- window that ran its time (a counted set saved "Yes, to the end": its answer 0, any outcome but stopped), a steady
-- move paced at the player's step `rung` or above (the row's rung, the step it was dosed at, against the cardio rung
-- at the row: a quest's steady move of a lower step's form, paced at the player's step, lights one, and a lower step's
-- card Start, dosed at its own step, none; the double-check of slices 52-55, 2026-10-24, where the form's own rung
-- left a newcomer's quest moves unlit) that ran its time without "I eased off", or a Long Haul (ns.HaulRow) that ran
-- its time with a Yes (ns.PaceYes). Never the stand-up move (off the ladder) nor a move stopped early. Pure: the apps'
-- and the desktop app's folds port it.
function ns.LightsEmber(r, rung)
	if r.kind ~= K.cardio or r.ladder ~= ns.CARDIO or not ns.Library[r.move] then return false end
	local answer = ns.Answer(r.flags)
	if ns.HasFlag(r.flags, "counted") then return r.outcome ~= O.stopped and answer == 0 end
	if r.outcome ~= O.done then return false end
	if ns.AllOutRow(r) then return true end
	if ns.HaulRow(r) then return ns.PaceYes(answer) end
	return answer ~= E.pace.eased and r.rung >= rung
end

-- An ember the row lit (ns.LightsEmber) on game day `idx`, embers_day a game day at most: the fourth lights none and
-- still pays everything else. Each counts toward the week's vault, and the day's last plays its toast (the event, with
-- the week's embers).
local function lightEmber(s, ts, idx)
	local d = day(s.sheet, idx)
	if (d.embers or 0) >= C.embers_day then return end
	d.embers = (d.embers or 0) + 1
	vaultAdd(s, ts, "embers", 1)
	if d.embers == C.embers_day then emit(s, { kind = "embers", week = s.sheet.vault.embers, day = idx }) end
end

-- A felt-unwell row (DB.lua ns.UnwellRow, every source the lock folds from) after the last active game day locks its
-- game day for the streak, until a performed move makes it, or a later day, active (activeDay prunes it then).
local function lockDay(sh, r)
	if not ns.UnwellRow(r) then return end
	local idx = dayIndex(r.ts)
	if not sh.activeIdx or idx > sh.activeIdx then sh.locked[idx] = true end
end

-- The locked days between the last active game day and game day `idx` (each lies after the last active one).
local function covered(sh, idx)
	local n = 0
	for d in pairs(sh.locked) do
		if d < idx then n = n + 1 end
	end
	return n
end

-- The largest of the streak's milestones the streak passed from `before` to `streak` (29 to 31 over a rest token
-- reaches 30), 0 for none.
local function milestone(before, streak)
	local reached = 0
	for _, m in ipairs(C.streak_milestones) do
		if before < m and m <= streak then reached = math.max(reached, m) end
	end
	return reached
end

-- The first finished move of a game day: the streak (the locked days of a gap skipped: they neither break it nor add a
-- day to it; the rest of the gap covered by rest tokens counts as rested days, one token a day, only when the tokens
-- cover it whole, said by a "rested" event before the day's streak), a token every token_days active days, Spirit for
-- each rested day and the week's Spirit practice, the milestone the streak reached, Welcome back after two days
-- without a pump (the gap as it is, locked days and all), the first pump of the day. The pump the move belongs to
-- keeps what its game day began with (the sheet summary's ld); `mirrored`: the move is an app's or a catch-up's
-- (sheet.lastMilestone).
local function activeDay(s, ts, idx, mirrored)
	local sh = s.sheet
	local pump = pumpOf(sh)
	local before = sh.streak
	local gap = sh.activeIdx and idx - sh.activeIdx - 1
	local late = gap and gap < 0 -- a late row of an earlier day (a catch-up merged after today's): the streak stays
	local free = gap and gap - covered(sh, idx)
	if late then
		gap = nil
	elseif not gap then
		sh.streak = 1
	elseif free <= 0 then
		sh.streak = sh.streak + 1
	elseif free <= sh.tokens then
		sh.tokens = sh.tokens - free
		for i = 1, gap do
			if not sh.locked[idx - i] then
				day(sh, idx - i).rested = true
				statGain(s, ts, "spirit", C.stat_rested_day, "rested")
			end
		end
		sh.streak = sh.streak + free + 1
		if pump then pump.rested = free end
		emit(s, { kind = "rested", count = free })
	else
		sh.streak = 1
	end
	if not late then
		sh.activeIdx = idx
		for d in pairs(sh.locked) do
			if d <= idx then sh.locked[d] = nil end
		end
	end
	sh.best = math.max(sh.best, sh.streak)
	local reached = milestone(before, sh.streak)
	if pump and reached > 0 then pump.milestone = reached end
	sh.activeDays = sh.activeDays + 1
	if sh.activeDays % C.token_days == 0 and sh.tokens < C.token_max then
		sh.tokens = sh.tokens + 1
		if pump then pump.token = true end
		emit(s, { kind = "token", tokens = sh.tokens })
	end
	local d = day(sh, idx)
	d.active, d.wk = true, ns.WeekOf(ts)
	if d.wk == sh.week then
		practised(s, ts, "spirit") -- an active week is Spirit's practice, whatever the moves
	end
	local first = gainXP(s, ts, C.xp_first_pump, "firstPump")
	if gap and gap >= C.welcome_after_days then
		local xp = gainXP(s, ts, C.xp_welcome, "welcome")
		statGain(s, ts, "spirit", C.stat_welcome, "welcome")
		if pump then pump.welcome = xp end
		emit(s, { kind = "welcome", xp = xp })
	elseif not late then
		-- the day's first move says so, the streak's day and what it paid, on the pump's banner (Duolingo's lesson: the
		-- small daily act is what to celebrate; no reminder, nothing at the day's end); a milestone's day says so by its
		-- own toast
		if pump then pump.streakDay = sh.streak end
		if reached == 0 then emit(s, { kind = "streak", days = sh.streak, xp = first }) end
	end
	if reached > 0 then
		sh.lastMilestone = { days = reached, day = idx, xp = first, mirrored = mirrored }
		emit(s, { kind = "milestone", days = reached, day = idx, xp = first })
	end
end

-- A Log entry; `mark` "ding" (an old save's level-up) or "death" (the death tax) for one that is no quest's pump. A
-- pump row's source (ns.Source, slice 26) names a catch-up's or an app pump's entry with its device; a game entry
-- keeps neither. An app pump whose banked moves came first (logMove) takes the entry they opened.
local function logEntry(sh, r, mark)
	local source, device = ns.Source(r.flags)
	if source == E.source.game then source = nil end
	local held = source == E.source.app and not mark and entryAt(sh, r.ts, true, device)
	if held then
		held.questID, held.tier = r.questID, r.tier
		return
	end
	sh.log[#sh.log + 1] = { ts = r.ts, questID = r.questID, tier = r.tier, ding = mark == "ding" or nil,
		death = mark == "death" or nil, source = source, device = device, moves = {} }
	if #sh.log > C.sheet_log_pumps then table.remove(sh.log, 1) end
end

-- A done move's line in its Log entry: the outcome, then a rep move's or a counted set's count (marked `count`, and
-- `slipped` where it was saved Form slipped: slice 42, 2026-10-17, the Log said such a set as a clean count;
-- `notAllOut` where it was saved Not all out, slice 53) or a
-- hold's or a timed move's seconds, a loaded set's weight in tenths of a kg (`load`, its load row's as the fold read
-- it: "11 kg × 16" in the Log, as every surface writes a set with a weight), and whether a correction names it
-- (`corrected`, DB.lua's r.corrected: the Log gives it no record, as the Saga's day and the apps' Logs do).
local function doneMove(r)
	local count = ns.HasFlag(r.flags, "counted")
	local mv = { move = r.move, out = r.outcome, count = count or nil, load = r.load, corrected = r.corrected,
		slipped = count and ns.Answer(r.flags) == E.answer.slipped or nil,
		notAllOut = count and ns.Answer(r.flags) == E.answer.notAllOut or nil }
	local v = (r.kind == K.rep or mv.count) and r.reps or r.seconds
	if v > 0 then mv.v = v end
	return mv
end

-- A move joins its pump's entry (an app pump's move its app pump's, slice 26, never a game pump's however close their
-- times); an unfinished one written with a turn-in owed on its own (the same second) joins that turn-in's. A move a
-- device banked (a match pump's Bank all, work mode's Later: slices 58 and 67, dated its app pump's start with the
-- device's bit alone) joins that app pump's entry, which it opens where the fold meets it first (the mirror folds the
-- game's rows first at a tie), never the game pump in progress. One owed on its own with quest 0 (no row at its
-- second): the death tax's (its reason, slice 23) opens a death's entry, or joins the last one while no pump came
-- after it (an evening of deaths never pushes the pumps out of the Log's last sheet_log_pumps); before
-- ding_retired_ts any other opens a level-up's (a ding has no stretch), unless the pump in progress is that ding;
-- after it, it joins the pump in progress (an open stand-up move owed at a turn-in).
local function logMove(sh, r, done)
	local pump = pumpOf(sh)
	-- an unfinished move whose paid row came first takes the fate logPaid kept for it, as the fold skips its owed row
	local key = not done and ns.OwedKey(r) or nil
	local fate = key and sh.paidEarly[key]
	if fate then sh.paidEarly[key] = nil end
	local entry = not done and entryAt(sh, r.ts)
	local source, device = ns.Source(r.flags)
	if not entry and not done and source == E.source.catchup then
		entry = entryAt(sh, r.ts, true, device)
		if not entry then
			logEntry(sh, { ts = r.ts, questID = 0, tier = r.tier, flags = ns.WithFlag(r.flags, "app") })
			entry = sh.log[#sh.log]
		end
	end
	if not entry and not done and r.questID == 0 then
		if r.outcome == O.owed and ns.Reason(r.flags) == E.owedBy.death then
			local last = sh.log[#sh.log]
			if last and last.death then
				entry = last
			else
				logEntry(sh, r, "death")
				entry = sh.log[#sh.log]
			end
		elseif r.ts < C.ding_retired_ts and not (pump and pump.ding) then
			logEntry(sh, r, "ding")
			entry = sh.log[#sh.log]
		end
	end
	entry = entry or (pump and entryAt(sh, pump.ts, fromApp))
	if not entry then return end
	-- what the Log tells of it: a done move's result (doneMove), an unfinished one's outcome and its queue key until
	-- its end (logPaid) or its fate already
	local mv
	if done then
		mv = doneMove(r)
	else
		mv = { move = r.move, out = r.outcome, fate = fate, key = not fate and key or nil }
	end
	entry.moves[#entry.moves + 1] = mv
end

-- A climb: a rung up, a plateau's or a first max set's placement high, a check's placement (2026-10-16), not undone, to
-- a rung this addon has (the fold's guard).
local function climbRow(r)
	local cause = ns.RungCause
	return r.kind == K.rung and (r.reps == cause.up or r.reps == cause.plateau or r.reps == cause.placementHigh
		or r.reps == cause.check) and not r.undone and ns.Ladders[r.ladder] and ns.Ladders[r.ladder].rungs[r.rung]
end

-- A climb marks its rung reached (sheet.rungs, ns.BestRung's), once, from any source, and only above the rung the
-- ladder started on, which was the player's from the start: a climb back to a rung reached before, after a rung down,
-- is "Rekindled" and pays nothing. After the player started the ladder over (DB.lua clearRow), a rung reached before
-- it is dated anew by the climb, so it counts again (ns.ReachCounts), and still pays nothing. True when it marked it.
local function markReached(s, r)
	local sh = s.sheet
	local key = ns.RungKey(r.ladder, r.rung)
	local lad = r.ladder == ns.CARDIO and s.cardio or s.ladders[r.ladder]
	if sh.rungs[key] and not ns.ReachCounts(s, r.ladder, r.rung) then sh.rungs[key] = r.ts end
	if sh.rungs[key] or r.rung <= (lad and lad.start or 0) then return false end
	sh.rungs[key] = r.ts
	return true
end

-- An orange or gold rung a set earned (DB.lua earnMark, 2026-10-09; its climb paid it and only opened it): its date
-- in sheet.earned, from which its colour, its Feat of Strength and its top title count (ns.EarnCounts, ns.BestRung);
-- dated anew where the player earned it before starting the ladder over.
local function markEarned(s, r)
	s.sheet.earned[ns.RungKey(r.ladder, r.earns)] = r.ts
end

-- One of the game's kept rows on the app's own sheet (DB.lua ns.Apply's "mark", tools/apppump.lua; the phones'
-- sheetMark): a felt-unwell row locks its game day for the streak (lockDay), a climb marks its rung reached
-- (markReached) and a set that earned an orange or gold rung marks it earned (markEarned), paying nothing there, as
-- this sheet marks an app's climb, so each sheet pays a rung once, at its first reach from any source. Nothing else of
-- a game row reaches the app's own sheet (no XP, stat, win, unlock or week turn).
function ns.SheetMark(s, r)
	lockDay(s.sheet, r)
	if climbRow(r) then markReached(s, r) end
	if r.earns then markEarned(s, r) end
end

-- An unfinished move's end, in the Log entry that left it open: the paid row repeats its queue key, and its skipped
-- flag says the player skipped it (Lc.Skip, Lc.SkipHeld); else a move done later paid it. An entry the Log no longer
-- keeps is not looked for. A paid row the fold saw before its owed row (DB.lua `paidEarly`, a merge out of order)
-- finds no move open: its fate waits for that owed row (logMove), as long as the fold's own mark does.
local function logPaid(s, r)
	local sh = s.sheet
	local key = ns.OwedKey(r)
	local fate = ns.HasFlag(r.flags, "skipped") and "skipped" or "later"
	for i = #sh.log, 1, -1 do
		for _, mv in ipairs(sh.log[i].moves) do
			if mv.key == key then
				mv.key, mv.fate = nil, fate
				return
			end
		end
	end
	if s.paidEarly[key] then sh.paidEarly[key] = fate end
end

local function muscle(sh, id)
	local mu = sh.muscles[id]
	if not mu then
		mu = { week = 0, hard = 0, rung = 0, last = {} }
		sh.muscles[id] = mu
	end
	return mu
end

-- The body map: moves this week per muscle (primary 1, secondary ½), hard sets, the best step among the moves that
-- train it as primary (its colour: the rung's step on the row's own ladder, the cardio rung for a cardio row, else the
-- ladder the move scales by) with that step's move and rung (a tie: the higher rung), its last five moves.
local function trainMuscles(sh, r, m, wk)
	local record = r.kind == K.rep or r.kind == K.hold or ns.HasFlag(r.flags, "counted")
	local function train(id, weight, primary)
		local mu = muscle(sh, id)
		if not mu.wk or wk > mu.wk then mu.wk, mu.week, mu.hard = wk, 0, 0 end
		if wk == mu.wk then -- a late row of an older week leaves this week's counts
			mu.week = mu.week + weight
			if primary and record then mu.hard = mu.hard + 1 end
		end
		if primary then
			local step, rung = rowStep(r, m)
			local best = mu.step or 0
			if step > best or step == best and (rung > mu.rung or not mu.best) then
				mu.step, mu.rung, mu.best = step, rung, r.move
			end
		end
		table.insert(mu.last, 1, { move = r.move, ts = r.ts })
		if #mu.last > 5 then mu.last[6] = nil end
	end
	for _, id in ipairs(m.mp) do train(id, 1, true) end
	for _, id in ipairs(m.ms) do train(id, 0.5, false) end
end

-- One performed move row (any outcome but owed, banked or paid), after the fold applied it: XP, stats, familiarity,
-- the counts titles read, the vault, the body map and the skills, on the pump it belongs to (pumpOf).
local function performedMove(s, r, m)
	local sh = s.sheet
	local ts, idx, wk = r.ts, dayIndex(r.ts), ns.WeekOf(r.ts)
	local stat = ns.MoveStat(r.move)
	local out = r.outcome
	local first = ns.HasFlag(r.flags, "firstTry")
	rest(s, ts)
	if not day(sh, idx).active then activeDay(s, ts, idx, ns.Source(r.flags) ~= E.source.game) end
	local pump = pumpOf(sh)
	-- a pump counts (the titles, the card's week, an app pump too: its quest is 0) at its first fresh move: one of only
	-- unfinished moves paid counts nothing, so banking never pays more than pumping at once; the feedback numbers count
	-- the game's own pumps (countPump)
	if pump and not pump.performed and not ns.HasFlag(r.flags, "owed") then
		pump.performed = true
		if not pump.banked and not fromApp then countPump(sh, pump, "pumpsFinished", 1) end
		sh.counts.pumps = sh.counts.pumps + 1
		sh.weekPumps = sh.weekPumps + 1
		if pump.tier >= 3 then sh.counts.t3 = sh.counts.t3 + 1 end
		if pump.quests >= 2 then sh.counts.merged = sh.counts.merged + 1 end
		if pump.ding then -- an old save's level-up pump (none is made since 2026-09-30): its XP and its title stay
			sh.counts.dings = sh.counts.dings + 1
			gainXP(s, ts, C.xp_ding, "ding")
		end
		local d = day(sh, idx) -- a duel's day and the day card's opening count the pumps as the titles do
		d.pumps = (d.pumps or 0) + 1
		if pump.questID == 0 then d.started = (d.started or 0) + 1 end
	end
	-- every performed move counts one toward the week's vault, an unfinished one paid too (2026-10-27, David: "Instead
	-- of pumps count moves please"; a pump at its first fresh move until then)
	vaultAdd(s, ts, "moves", 1)
	local allout = ns.AllOutRow(r)
	local counted = ns.HasFlag(r.flags, "counted")
	-- a set read on a split ladder or an old pose read in its family pays at the rung it was paid at (ns.Translate)
	local rung = (r.xpRung or 0) > 0 and r.xpRung or r.rung > 0 and r.rung or m.rung
	gainXP(s, ts, ns.MoveXP(r.move, out, ns.HasFlag(r.flags, "owed"), { rung = rung, allout = allout,
		firstTry = first }), "move")
	if not sh.patterns[m.pattern] then
		sh.patterns[m.pattern] = true
		gainXP(s, ts, C.xp_first_pattern, "pattern")
		statGain(s, ts, "intellect", C.stat_pattern_new, "pattern", r.move)
		learned(s, ts, r.move)
	end
	local win = false
	if out == O.beat then
		win = true
		sh.counts.beats = sh.counts.beats + 1
		sh.weekBeats = sh.weekBeats + 1
		statGain(s, ts, stat, C.stat_record, "record", r.move)
		vaultAdd(s, ts, "records", 1)
		emit(s, { kind = "record", move = r.move, value = r.kind == K.hold and r.seconds or r.reps, load = r.load })
	elseif out == O.lastbeat then
		win = true
	end
	if first then
		win = true
		statGain(s, ts, stat, C.stat_first_try, "firstTry", r.move)
	end
	-- an orange or gold rung this set earned (markEarned): a win, and the pump's unlock as earned (Rekindled where the
	-- player had earned it before starting the ladder over)
	if r.earns then
		win = true
		markEarned(s, r)
		if pump and ts >= pump.ts then
			pump.unlocks = pump.unlocks or {}
			pump.unlocks[#pump.unlocks + 1] = { ladder = r.ladder, rung = r.earns, rekindled = r.rekindles == true,
				earned = true }
		end
	end
	if r.stars and r.stars > (r.starsFrom or 0) then
		win = true
		for _ = (r.starsFrom or 0) + 1, r.stars do
			if pump then pump.stars = (pump.stars or 0) + 1 end
			gainXP(s, ts, C.xp_star, "star")
			statGain(s, ts, ns.LadderStat(r.ladder), C.stat_star, "star", r.move)
			vaultAdd(s, ts, "records", 1)
		end
		day(sh, idx).stars = (day(sh, idx).stars or 0) + 1
		if r.stars >= 3 then sh.counts.stars3 = sh.counts.stars3 + 1 end
	end
	if r.placed then sh.counts.placed = sh.counts.placed + 1 end
	-- the cardio family's progression: each new 50 of the steady ledger's best M (DB.lua steadyRow, slice 52; a P step
	-- until then), the gain's reason still "p"
	if r.mSteps then statGain(s, ts, ns.LadderStat(ns.CARDIO), C.stat_p_step * r.mSteps, "p") end
	-- familiarity: a finished move on a new distinct day
	if out ~= O.stopped then
		local fam = sh.fam[r.move]
		if not fam then
			fam = { n = 0 }
			sh.fam[r.move] = fam
		end
		if not fam.last or idx > fam.last then -- a late row of an earlier day is not known to be a new one
			fam.last, fam.n = idx, fam.n + 1
			local level = famLevel(fam.n)
			if level then
				if pump then pump.fam = (pump.fam or 0) + 1 end
				learned(s, ts, r.move)
			end
			local pays = level and famPays(sh, m, level, true)
			if level == 1 then
				if pays then statGain(s, ts, stat, C.stat_fam.tried, "tried", r.move) end
			elseif level == 2 then
				if pays then statGain(s, ts, stat, C.stat_fam.familiar, "familiar", r.move) end
				for _, flag in ipairs(m.flags) do
					if flag == "skill" then statGain(s, ts, "intellect", C.stat_skill_familiar, "skill", r.move) end
				end
			elseif level == 3 then
				if pays then statGain(s, ts, stat, C.stat_fam.learned, "learned", r.move) end
			end
		end
	end
	practised(s, ts, stat, r.move)
	local d = day(sh, idx)
	-- what the titles count
	local c = sh.counts
	if r.kind == K.rep and m.pattern == "squat" then c.squats = c.squats + r.reps end
	if r.kind == K.hold and m.ladder == ns.Enum.ladder.wallsit then c.wallsit = c.wallsit + r.seconds end
	if allout and out ~= O.stopped then c.finishers = c.finishers + 1 end
	if r.kind == K.cardio then
		if pump and pump.travel then c.travelCardio = c.travelCardio + 1 end
		sh.weekCardio = sh.weekCardio + r.seconds
		d.cardio = (d.cardio or 0) + r.seconds -- the day's tally and a duel's day
		if ns.LightsEmber(r, ns.Rung(s, ns.CARDIO)) then lightEmber(s, ts, idx) end
	end
	-- the day's tally (ns.DayTally): reps by pattern (a counted set's count too), the all-out windows finished
	if (r.kind == K.rep or counted) and r.reps > 0 then
		d.reps = d.reps or {}
		d.reps[m.pattern] = (d.reps[m.pattern] or 0) + r.reps
	end
	if allout and out ~= O.stopped then d.finishers = (d.finishers or 0) + 1 end
	local units = sh.units[m.pattern] or 0
	sh.units[m.pattern] = units + (r.kind == K.rep and r.reps or r.seconds)
	-- a skill-up (the profession metaphor) is learning too, so Intellect moves on evenings without a yoga pose
	if skillOf(sh.units[m.pattern]) > skillOf(units) then learned(s, ts, r.move) end
	trainMuscles(sh, r, m, wk)
	if r.kind == K.rep or r.kind == K.hold or counted then -- a counted set chases its count as a rep move does
		d.rh = d.rh + 1
		if win then addWin(sh, idx) end
		sh.lastMove = { ts = ts, idx = idx, win = win }
		local l = r.ladder > 0 and r.ladder or ns.ScaleLadder(m)
		if l then sh.weekSets[l] = (sh.weekSets[l] or 0) + 1 end
	end
	logMove(sh, r, true)
end

-- The Saga's day (2026-10-15; contract/fixtures/history.json `saga` states the rules) ------------------------------

-- Day d's moves, every source's (DB.lua's day counts): the number the HUD, the Today tab and the day card show.
local function dayMoves(s, d)
	local c = ns.DayCounts(s, d)
	return c.moves + c.app.moves
end

-- The order of the best days: the most moves first, of two alike the earlier day.
local function topFirst(a, b)
	if a.moves ~= b.moves then return a.moves > b.moves end
	return a.day < b.day
end

-- sheet.topDays after day `d` reached `moves`: the two best days (topFirst). Two are enough, because a day's count only
-- rises: a day that leaves the pair has two days at least as good beside it, so the best before today is in the pair
-- or among the fold's recent day counts (ns.DayRank), unless the pair holds today and a day dated after it.
local function raiseTop(sh, d, moves)
	local top, found = sh.topDays, false
	for _, t in ipairs(top) do
		if t.day == d then t.moves, found = moves, true end
	end
	if not found then top[#top + 1] = { moves = moves, day = d } end
	table.sort(top, topFirst)
	top[3] = nil
end

-- A state from a baseline written before the best days (DB.lua ns.NewState): they start from the day counts it holds.
function Stats.SeedTop(s)
	for d in pairs(s.days) do
		local moves = dayMoves(s, d)
		if moves > 0 then raiseTop(s.sheet, d, moves) end
	end
end

-- A sheet from a baseline (DB.lua ns.NewState): each pose family's and cardio step starts at least at the familiarity
-- step the furthest of its forms reached, which the baseline paid already. One written before the step paid once
-- (slice 47's poses, slice 54's cardio) gets its steps here; for one written after, the step already stands there
-- (famPays marks every step a form reaches), so this changes nothing.
function Stats.SeedFamSteps(sheet)
	for id, f in pairs(sheet.fam) do
		local m = ns.Library[id]
		if m and m.ladder and (ns.POSE[m.ladder] or m.ladder == ns.CARDIO) then
			local n = f.n or 0
			local level = n >= LEARNED and 3 or n >= FAMILIAR and 2 or n >= 1 and 1 or 0
			local key = ns.RungKey(m.ladder, m.rung)
			if level > (sheet.famSteps[key] or 0) then sheet.famSteps[key] = level end
		end
	end
end

-- Whether a typed set beats the day's best so far: by load score where both carry a weight (Progress.lua
-- ns.LoadScore, as the weighted record compares them), a weighted set over one without, else the count; a tie keeps
-- the earlier.
local function betterSet(v, load, top)
	if load and top.load then return ns.LoadScore(load, v) > ns.LoadScore(top.load, top.v) end
	if load or top.load then return load ~= nil end
	return v > top.v
end

-- A performed move on its game day `idx` (rules 9-12, every source): its pump's bar (the pump it belongs to, by its
-- start, else the move's own time, and its source), its region's count, and for a typed set (the count DB.lua's fold
-- kept a record of, r.typed: a rep or hold set or a counted cardio set) the day's best set of the move, its record at
-- the day's start (from the fold's record before the move's first set that day: `before`, ns.Apply) and, stored as a
-- record (outcome beat) that no correction names (DB.lua's r.corrected, as the apps leave a corrected set out), its
-- place among the day's records.
local function sagaMove(s, r, m, before, idx)
	local sh = s.sheet
	local d = day(sh, idx)
	local pump = pumpOf(sh)
	local at, source = pump and pump.ts or r.ts, ns.Source(r.flags)
	d.bars = d.bars or {}
	local bar
	for i = #d.bars, 1, -1 do
		local b = d.bars[i]
		if b.ts == at and b.source == source then
			bar = b
			break
		end
	end
	if not bar then
		bar = { ts = at, n = 0, source = source }
		d.bars[#d.bars + 1] = bar
	end
	bar.n = bar.n + 1
	if m.region then
		d.regions = d.regions or {}
		d.regions[m.region] = (d.regions[m.region] or 0) + 1
	end
	local value = r.typed
	if not value then return end
	d.was, d.sets = d.was or {}, d.sets or {}
	if d.was[r.move] == nil then
		d.was[r.move] = before.best ~= nil and { v = before.best, load = before.topLoad, reps = before.topReps } or false
	end
	local top = d.sets[r.move]
	if not top or betterSet(value, r.load, top) then d.sets[r.move] = { v = value, load = r.load } end
	if r.outcome ~= O.beat or r.corrected then return end
	if source == E.source.app then
		d.appRecordMoves = d.appRecordMoves or {}
		d.appRecordMoves[r.move] = true
	end
	d.records = d.records or {}
	for _, id in ipairs(d.records) do
		if id == r.move then return end
	end
	d.records[#d.records + 1] = r.move
end

-- Called by the fold after every row, with what it saw before the row (`before.owed`: something was unfinished; a
-- move row's record, ns.Apply). An app row (slice 26) pays as a catch-up's does (slice 27, 2026-10-06: one character
-- for a player with both, call 3): its XP, its stats and its wins, on the app pump it belongs to (pumpOf), its moves
-- joining that pump's Log entry, which names its source and device; never paying or owing unfinished moves (an app's
-- owed, banked or paid move row and its pump's felt-unwell stop add nothing) and never in the game's own numbers (the
-- feedback numbers' pumps, countPump). Its moves count in the Saga's day as the game's do (the best day in moves since
-- 2026-10-15). A felt-unwell row of any source locks its game day for the streak first (lockDay).
function ns.SheetApply(s, r, before)
	local sh = s.sheet
	fromApp = ns.Source(r.flags) == E.source.app
	local wk = ns.WeekOf(r.ts)
	if sh.week ~= wk and (not sh.week or wk > sh.week) then newWeek(sh, wk) end
	lockDay(sh, r)
	local kind, out = r.kind, r.outcome
	if (kind == K.pump or kind == K.ding) and out == O.offered then
		local pump = { ts = r.ts, questID = r.questID, tier = r.tier, quests = kind == K.pump and r.seconds or 0,
			ding = kind == K.ding, performed = false, xp = 0, pts = 0, stars = 0, fam = 0, stats = {} }
		if fromApp then sh.appPump = pump else sh.pump = pump end
		logEntry(sh, r, kind == K.ding and "ding" or nil)
	elseif kind == K.pump and out == O.banked and r.reps > 0 and ns.Reason(r.flags) ~= E.bank.unwell and not fromApp then
		-- Bank it put some of the pump's own moves off (reps 0 is "Bank the rest", and a stop for feeling unwell puts
		-- nothing off): a banked pump, never a finished one, in the feedback numbers
		local pump = sh.pump
		if pump and not pump.banked then
			pump.banked = true
			if pump.performed then countPump(sh, pump, "pumpsFinished", -1) end
			countPump(sh, pump, "pumpsBanked", 1)
		end
	elseif kind == K.stretch then
		local pump = pumpOf(sh)
		sh.counts.foot = sh.counts.foot + r.reps
		if pump and pump.ts == r.ts then
			pump.travel = r.reps / math.max(1, pump.quests) >= C.travel_yd
		else
			-- a turn-in owed without a pump of its own (a party, an instance, a pump that never opened): a Log entry for
			-- its unfinished moves; the pump in progress stays the one its moves count for
			logEntry(sh, r)
		end
	elseif kind == K.rep or kind == K.hold or kind == K.cardio or kind == K.flow then
		local m = ns.Library[r.move]
		if not m then return end
		if out ~= O.paid and out ~= O.owed and out ~= O.banked then
			performedMove(s, r, m)
			local d = ns.day(r.ts)
			local idx = ns.DayIndex(d)
			sagaMove(s, r, m, before, idx)
			-- the best days in moves, from the fold's day count this row just raised; a game row written now that lifts
			-- today one past its old best, the gate passed, is the day's one "New best day" (a fold at a login or a
			-- /reload writes no live rows; Sheet.Earned keeps the day it toasted, should a reload's mirror lack the row)
			local moves = dayMoves(s, d)
			raiseTop(sh, d, moves)
			if s.live and not fromApp then
				local rank = ns.DayRank(s, r.ts)
				if rank.best and moves == rank.old.moves + 1 then emit(s, { kind = "bestDay", moves = moves, day = idx }) end
			end
		elseif not fromApp then -- an app row never pays or owes an unfinished move
			if out == O.paid then
				-- a skip (Lc.Skip) earns nothing: a queue it empties is no Clean Slate
				if before.owed and next(s.owed) == nil and not ns.HasFlag(r.flags, "skipped") then
					sh.counts.clean = sh.counts.clean + 1
					emit(s, { kind = "clean" })
				end
				logPaid(s, r)
			else
				logMove(sh, r, false)
			end
		end
	elseif climbRow(r) then
		-- a first max set at the rung's top places the player there: calibration, never progress (2026-10-01; it paid
		-- a fit player 100 XP, 5 points and a win per ladder on day one), so the rung is only marked reached; and so
		-- does a check that lifts a pose family to its step (2026-10-16, slice 42)
		local placed = r.reps == ns.RungCause.placementHigh or r.reps == ns.RungCause.check
		local first = markReached(s, r)
		if first then -- the Saga's day: a step reached for the first time, a placement's too (rule 13)
			local d = day(sh, dayIndex(r.ts))
			d.unlocks = d.unlocks or {}
			d.unlocks[#d.unlocks + 1] = { ts = r.ts, ladder = r.ladder, rung = r.rung }
		end
		-- an orange or gold rung no set has earned only opens (2026-10-09: markEarned)
		local opened = ns.TopTier(r.ladder, r.rung) and not ns.EarnCounts(s, r.ladder, r.rung)
		-- the pump's unlocks, in the order they came (the sheet summary's lu): a placement's first reach is one too,
		-- though it pays nothing; a rung reached before, or at or under the start, comes back Rekindled
		local pump = pumpOf(sh)
		if pump and r.ts >= pump.ts then
			pump.unlocks = pump.unlocks or {}
			pump.unlocks[#pump.unlocks + 1] = { ladder = r.ladder, rung = r.rung, rekindled = not first and not opened,
				opened = opened or nil }
		end
		if first and not placed then
			sh.climbs[r.ladder] = (sh.climbs[r.ladder] or 0) + 1
			gainXP(s, r.ts, C.xp_rung_up, "rung")
			-- the steps the rung climbs from the one below, so every colour pays alike on every ladder
			local steps = ns.Step(r.ladder, r.rung) - (ns.Step(r.ladder, r.rung - 1) or 0)
			statGain(s, r.ts, ns.LadderStat(r.ladder), C.stat_step * steps, "rung")
		end
		local lm = sh.lastMove
		if not placed and lm and lm.ts == r.ts and not lm.win then
			lm.win = true
			addWin(sh, lm.idx)
		end
	end
	-- a retired title is held from the rows before it retired, never earned after (ns.Deeds shows it as Legacy)
	if r.ts < C.titles_retired_ts then
		for _, t in ipairs(ns.Titles) do
			if t.retired and not sh.legacy[t.id] and t.value(s) >= t.goal then sh.legacy[t.id] = true end
		end
	end
end

-- Readers ----------------------------------------------------------------------------------------------------------

-- The sheet's level now, the XP into it and what the next one needs, and its Paragon rank (levelOf).
function ns.Level(state)
	return levelOf(state.sheet.xp)
end

-- The level of any lifetime XP, the XP into it, what the next one needs and its Paragon rank (levelOf): the desktop
-- app's Saga, a day's level at its start and its end (tools/apppump.lua Engine:levels; history.json saga rule 15).
function ns.LevelOf(xp)
	return levelOf(xp)
end

-- The last pump the sheet counted: the game's (or a catch-up's) or the app's (slice 27), the later by its offered
-- time; nil before any. What the sheet summary tells of it (last, ls, lu, ld) reads it here; the pump's banner keeps
-- the game pump's live path (Sheet.lua).
function ns.LastPump(state)
	local p, a = state.sheet.pump, state.sheet.appPump
	if a and (not p or a.ts > p.ts) then return a end
	return p
end

-- The stats now, each with its value at the last weekly reset (the tooltip's "142 (136+6)").
function ns.StatValues(state, now)
	local sh = state.sheet
	local sameWeek = sh.week == ns.WeekOf(now or ns.now())
	local out = {}
	for i, key in ipairs(ns.STATS) do
		out[i] = { key = key, value = sh.stats[key], base = sameWeek and sh.base[key] or sh.stats[key], last = sh.last[key] }
	end
	return out
end

-- The build: the two highest stats, ties in Forever's order. Hidden from 2026-10-03 to slice 21 (the day plan gave
-- every player the same stats, so every build read "of the Gorilla"); since the stats come from the families a player
-- climbs (ns.MoveStat), the sheet's level line, the HUD tooltip and the sheet summary name it again.
function ns.Build(state)
	local list = {}
	for _, key in ipairs(ns.STATS) do list[#list + 1] = key end
	local st = state.sheet.stats
	table.sort(list, function(a, b)
		if st[a] ~= st[b] then return st[a] > st[b] end
		return STAT_ORDER[a] < STAT_ORDER[b]
	end)
	local a, b = list[1], list[2]
	if STAT_ORDER[a] > STAT_ORDER[b] then a, b = b, a end
	return BUILD[a .. "_" .. b], a, b
end

-- The streak as it stands now: kept while today, yesterday or a gap the locked days and the tokens cover.
function ns.Streak(state, now)
	local sh = state.sheet
	if not sh.activeIdx then return 0, sh.tokens, sh.best end
	local today = dayIndex(now or ns.now())
	local gap = today - sh.activeIdx - 1
	if gap <= 0 or gap - covered(sh, today) <= sh.tokens then return sh.streak, sh.tokens, sh.best end
	return 0, sh.tokens, sh.best
end

-- The locked days after the game day of the last performed move (rt), as days after it, ascending (the summary's lk);
-- nil when none.
local function lockedAfter(sh)
	if not sh.restTs then return nil end
	local anchor = dayIndex(sh.restTs)
	local out = {}
	for d in pairs(sh.locked) do
		if d > anchor then out[#out + 1] = d - anchor end
	end
	table.sort(out)
	return #out > 0 and out or nil
end

-- The pumps of the sheet's Log that paid XP (the summary's lx, slice 34), in the Log's order: { the pump's offered
-- time, the XP it paid } each, as gainXP added it up on its entry; nil when none.
local function loggedXP(sh)
	local out = {}
	for _, e in ipairs(sh.log) do
		if (e.xp or 0) > 0 then out[#out + 1] = { e.ts, e.xp } end
	end
	return #out > 0 and out or nil
end

-- The XP of the game days the summary covers (its dx and dxFrom, 2026-10-15; history.json saga rule 15): from the
-- later of the dx_days-th game day back from `now`'s (that one included) and the first the sheet holds whole
-- (heldFrom, Stats.HoldFrom), { "YYYY-MM-DD", the day's xp } for each covered day with XP, ascending (nil when none: a
-- covered day absent gained none), a day dated after `now`'s (a clock set back) among them, so the lifetime XP less a
-- day's and every later day's stays exact; and the first covered day, "YYYY-MM-DD" (a day before it is unknown). Each
-- date is stepped from `now`'s game day by its middle (ns.DayMid, so a clock change never moves it).
local function dayXPs(sh, now)
	local today = dayIndex(now)
	local from = math.max(today - C.dx_days + 1, sh.heldFrom or -math.huge)
	local held = {}
	for i, d in pairs(sh.days) do
		if i >= from and (d.xp or 0) > 0 then held[#held + 1] = i end
	end
	table.sort(held)
	local out = {}
	for k, i in ipairs(held) do out[k] = { ns.day(ns.DayMid(now, i - today)), sh.days[i].xp } end
	return #out > 0 and out or nil, ns.day(ns.DayMid(now, from - today))
end

-- The game's two best days of all time (the summary's bd, 2026-10-15; history.json saga rule 4): sheet.topDays as
-- { "YYYY-MM-DD", moves } each, the most first and of two alike the earlier; nil before a move.
local function bestDays(sh)
	local out = {}
	for i, t in ipairs(sh.topDays) do out[i] = { t.day, t.moves } end
	return #out > 0 and out or nil
end

-- The days of the ratings' window (the last C.rating_days game days, today included) that the fold has, in order.
local function ratingDays(sh, now)
	local today = dayIndex(now or ns.now())
	local i = today - C.rating_days
	return function()
		while i < today do
			i = i + 1
			if sh.days[i] then return sh.days[i] end
		end
	end
end

-- The ratings (plan slice 3): Crit Chance, Expertise, Defense, Block. Crit Chance is WoW's 5 % base plus a rating
-- that only grows, with diminishing returns towards crit_base + crit_span (David, 2026-10-01: "Crit chance is at
-- 50%?": the 28-day share of wins it was read like a world-buffed raider's and fell as records got harder); the share
-- stays in its tooltip (`wins` of `moves` in the window). Defense counts the window's days, Block the tokens held.
function ns.Ratings(state, now)
	local sh = state.sheet
	local rh, wins, days = 0, 0, 0
	for d in ratingDays(sh, now) do
		rh, wins = rh + d.rh, wins + d.wins
		if d.active or d.rested then days = days + 1 end
	end
	local known = 0
	for _, f in pairs(sh.fam) do
		if f.n >= LEARNED then known = known + 1 end
	end
	local R = sh.critR
	return { crit = (C.crit_base + C.crit_span * R / (R + C.crit_half)) / 100, critRating = R, wins = wins, moves = rh,
		expertise = known, defense = days, block = sh.tokens }
end

-- The days with a finished move in the ratings' window (a rested day is no active day), and whether today is one.
function ns.ActiveDays(state, now)
	local n = 0
	for d in ratingDays(state.sheet, now) do
		if d.active then n = n + 1 end
	end
	local today = state.sheet.days[dayIndex(now or ns.now())]
	return n, today ~= nil and today.active == true
end

-- This week's counted numbers: the pumps counted (at their first fresh move), the records beaten and the cardio
-- minutes from the timer (the card's, Social.lua).
function ns.Week(state, now)
	local sh = state.sheet
	if sh.week ~= ns.WeekOf(now or ns.now()) then return 0, 0, 0 end
	return sh.weekPumps, sh.weekBeats, math.floor(sh.weekCardio / 60)
end

-- The embers (slice 55): today's lit, the most a game day lights (embers_day) and this week's.
function ns.Embers(state, now)
	local sh = state.sheet
	now = now or ns.now()
	local d = sh.days[dayIndex(now)]
	local week = sh.week == ns.WeekOf(now) and sh.vault.embers or 0
	return d and d.embers or 0, C.embers_day, week
end

-- The pumps counted this game day, as the titles count them (at a pump's first fresh move): a pact's day (Social.lua)
-- and the day card's opening on entering a rested area (Streamer.Offer).
function ns.CountedPumps(state, now)
	local d = state.sheet.days[dayIndex(now or ns.now())]
	return d and d.pumps or 0
end

-- A duel's score (slice 23) over its game days from `start` (a day index) to today, its last day at most: each day
-- duel_pump_pts a pump counted (at most duel_pumps_day, duel_started_day of them with no quest), duel_active_pts for an
-- active day and duel_cardio_pts a cardio minute from the timer (at most duel_cardio_min_day). Counted units only, and
-- the caps stop the score, never the pumps, their XP or their records (safety.md, hazard 12).
function ns.DuelScore(state, start, now)
	local sh, score = state.sheet, 0
	for idx = start, math.min(dayIndex(now or ns.now()), start + C.duel_days - 1) do
		local d = sh.days[idx]
		if d then
			local started = d.started or 0
			local pumps = math.min(C.duel_pumps_day, (d.pumps or 0) - started + math.min(C.duel_started_day, started))
			local minutes = math.min(C.duel_cardio_min_day, math.floor((d.cardio or 0) / 60))
			score = score + pumps * C.duel_pump_pts + (d.active and C.duel_active_pts or 0) + minutes * C.duel_cardio_pts
		end
	end
	return score
end

-- The title the player chose, while it is earned (the sheet's title bar, the card).
function ns.ChosenTitle(state, profile)
	local chosen = profile and profile.title
	if chosen and ns.TitlesEarned(state)[chosen] then return chosen end
	return nil
end

-- The portrait border the QP level earns (docs/plan.md slice 13): 0 none, then one per level of C.border_levels, and
-- the last one past the cap with a Paragon rank.
function ns.Border(state)
	local level, _, _, paragon = ns.Level(state)
	if paragon > 0 then return #C.border_levels + 1 end
	local border = 0
	for i, at in ipairs(C.border_levels) do
		if level >= at then border = i end
	end
	return border
end

-- The QuestPump card's own numbers (docs/plan.md slice 13): what QuestPump counts, never a typed count, the game day's
-- moves among them (the day's count the sheet's Log heads the day with, for the party's line). The title and the
-- border travel as their codes; Social.lua adds the in-a-pump flag and the typed records when the player shares them
-- (ns.TypedRecords).
function ns.Card(state, profile, now)
	now = now or ns.now()
	local level, _, _, paragon = ns.Level(state)
	local title = ns.ChosenTitle(state, profile)
	local active, today = ns.ActiveDays(state, now)
	local pumps, beats, cardio = ns.Week(state, now)
	return { level = level, paragon = paragon, title = title and ns.Enum.title[title] or 0, border = ns.Border(state),
		streak = (ns.Streak(state, now)), active = active, today = today, pumps = pumps, beats = beats, cardio = cardio,
		movesToday = ns.DayCount(state, now).moves }
end

-- The typed progress a player may share (Settings > Social, "Include my records"): per strength ladder the anchor of
-- the rung the player stands on, its first and best count and the game day of the first. The anchor, never the move
-- the filters put in its place: a substitute would tell a pain flag, which the card never carries.
function ns.TypedRecords(state)
	local out = {}
	for _, l in ipairs(ns.STRENGTH_ORDER) do
		local rung = ns.Rung(state, l)
		local move = ns.Anchor(l, rung)
		local first, firstTs, best = ns.RecordSpan(state, move)
		if best then
			out[#out + 1] = { ladder = l, rung = rung, move = move, first = first, best = best,
				day = dayIndex(firstTs) }
		end
	end
	return out
end

-- The weekly vault now: each goal's progress up to its last tier, the need of its next tier (its last once all are
-- paid), its tiers paid and how many it has (the embers' three, slice 55; one for the others), and whether all paid.
function ns.Vault(state, now)
	local sh = state.sheet
	local same = sh.week == ns.WeekOf(now or ns.now())
	local out = {}
	for i, goal in ipairs(VAULT_ORDER) do
		local tiers = tiersOf(goal)
		local paid = same and math.min(paidOf(sh, goal), tiers) or 0
		local v = same and sh.vault[goal] or 0
		out[i] = { goal = goal, value = math.min(v, needOf(goal, tiers)), need = needOf(goal, math.min(paid + 1, tiers)),
			paid = paid, tiers = tiers, done = paid == tiers }
	end
	return out, ns.NextReset(now or ns.now())
end

-- A pattern's skill (the profession metaphor): min(300, floor(units ^ 0.62)) and its tier word, the highest of
-- C.skill_tiers' lines it reached.
local TIERS = { "apprentice", "journeyman", "expert", "artisan" }

function ns.Skill(state, pattern)
	local skill = skillOf(state.sheet.units[pattern] or 0)
	local tier = 1
	for i, line in ipairs(C.skill_tiers) do
		if skill >= line then tier = i + 1 end
	end
	return skill, TIERS[tier]
end

-- The body map now, per muscle: moves this week, hard sets, the quality of its best step and its last moves. Heart
-- takes the colour the cardio ladder earned; Breath and Balance, like every muscle, the best step trained, on the
-- stretches and the poses of their families since slice 21. `f` is the filter context (the cardio rung's move).
function ns.BodyMap(state, now, f)
	local sh = state.sheet
	now = now or ns.now()
	local wk = ns.WeekOf(now)
	local out = {}
	for id = 1, ns.MUSCLES do
		local mu = sh.muscles[id]
		local week = (mu and mu.wk == wk) and mu.week or 0
		local quality
		if id == HEART then
			-- the colour the cardio ladder earned, kept through a rung down like every unlock (ns.LadderColour)
			quality = mu and ns.LadderColour(state, ns.CARDIO) or nil
		elseif mu and mu.rung > 0 then
			-- the best step trained, Breath's and Balance's on the stretches and poses of their families (slice 21; their
			-- learned share until then); Starter for a muscle only moves no ladder places trained (step 0)
			quality = C.quality_by_step[mu.step or 0] or Q.starter
		end
		-- the best step's place on the one scale (the tooltip's "step 9"), and its move
		local ladderStep, best = mu and mu.step or 0, mu and mu.best
		if id == HEART and mu then -- the cardio rung now, its step and icon with it (after a rung down too)
			local rung = ns.Rung(state, ns.CARDIO)
			ladderStep = ns.StepOn(ns.CARDIO, rung)
			best = (f and ns.RungMove(ns.CARDIO, rung, f)) or ns.Anchor(ns.CARDIO, rung) or best
		end
		out[id] = { week = math.floor(week), exact = week, hard = (mu and mu.wk == wk) and mu.hard or 0,
			ladderStep = ladderStep, best = best, quality = quality, last = mu and mu.last or {},
			step = week >= C.map_steps[2] and 3 or week >= C.map_steps[1] and 2 or week > 0 and 1 or 0 }
	end
	return out
end

-- The game day's pumps, moves and records every source's, the Saga's (rules 9 and 12; 2026-10-15): the pumps with a
-- move done that day (the strip's bars: a pump across 04:00 counts on each day it holds a move, one banked or stopped
-- before its first move on none), the moves (dayMoves) and the moves stored as a record that day, each once (two
-- records of one move are one); with the app's share of each in `app` (an app pump's bar, the app's own moves, a move
-- with a record of the app's), as slice 26 named it. The one count the day card, its Share line and the Streamer
-- tally (ns.DayTally), the Today tab and /qp status (ns.Today) show.
local function dayCount(state, now)
	local key = ns.day(now)
	local d = state.sheet.days[dayIndex(now)] or {}
	local appPumps, appRecords = 0, 0
	for _, b in ipairs(d.bars or {}) do
		if b.source == E.source.app then appPumps = appPumps + 1 end
	end
	for _ in pairs(d.appRecordMoves or {}) do appRecords = appRecords + 1 end
	return { pumps = #(d.bars or {}), moves = dayMoves(state, key), records = #(d.records or {}),
		app = { pumps = appPumps, moves = ns.DayCounts(state, key).app.moves, records = appRecords } }
end

-- A game day's moves and pumps as dayCount counts them, for any moment `now` of that day: the sheet's Log heads each
-- day with them (SheetPages.lua, 2026-10-16). The moves reach back as far as the day counts (60 days); the pumps are
-- nil once the sheet no longer holds the day's bars (Stats.Prune keeps the Saga's detail two days), never a guess.
function ns.DayCount(state, now)
	local c = dayCount(state, now)
	if not (state.sheet.days[dayIndex(now)] or {}).bars then c.pumps = nil end
	return c
end

-- The XP the sheet counted on the game day of `now`, its rested bonus in it (the day's xp, gainXP): the Today tab, the
-- day card, the desktop app's today (tools/apppump.lua) and the summary's dx for that day show this one number.
function ns.DayXP(state, now)
	local d = state.sheet.days[dayIndex(now or ns.now())]
	return d and d.xp or 0
end

-- What today looks like, and the week's vault (the Today tab, /qp status): the pumps, moves and records every source's,
-- the app's among them also in `app` (dayCount); the stars, wins and XP the sheet's, which counts every source's too
-- (slice 27).
function ns.Today(state, now)
	local sh = state.sheet
	now = now or ns.now()
	local today = dayIndex(now)
	local d = sh.days[today] or {}
	local count = dayCount(state, now)
	local regions = {}
	for _, region in ipairs(ns.REGIONS) do
		local reg = state.regions[region]
		local n = reg and reg.days and reg.days[ns.day(now)] or 0
		regions[#regions + 1] = { region = region, sets = n }
	end
	-- the tenth hard set on one ladder in a day (safety.md, row 10): the Today tab says so
	local hard = {}
	for _, l in ipairs(ns.STRENGTH_ORDER) do
		local lad = state.ladders[l]
		local n = lad and lad.days and lad.days[ns.day(now)] or 0
		if n >= C.hard_set_note then hard[#hard + 1] = { ladder = l, sets = n } end
	end
	local vault, reset = ns.Vault(state, now)
	return {
		pumps = count.pumps, moves = count.moves, records = count.records,
		app = count.app, stars = d.stars or 0, wins = d.wins or 0, unfinished = ns.OwedCount(state),
		xp = ns.DayXP(state, now), rested = ns.Rested(state, now), regions = regions, hard = hard,
		vault = vault, reset = reset,
	}
end

-- The Saga's rank of the game day of `now` (contract/fixtures/history.json `saga`, rules 1-8; 2026-10-15), as it
-- stands: its moves from every source; earlier, the game days with a move before it (sheet.activeDays counts today
-- once today has one); the gate, earlier at best_day_min_days; old, the best day before it, { moves, day }, the
-- earliest that reached the most (nil before any); best, the gate passed and more moves than old ("Best day ever");
-- window and above, the days with a move in the day_rank_days before it and how many of them have more moves; quality,
-- an ns.Enum.quality key: "legendary" for a best day, else "epic", "rare", "uncommon" or "common" by the share above
-- (day_rank_shares), nil before the gate, without a move or with an empty window; and top, the best day ever with
-- today's moves. The day counts reach 60 days back (DB.lua's baseline) and sheet.topDays every day before: the day
-- card, the best-day toast and the Records tab read this one count. Today is live: its word only rises.
function ns.DayRank(state, now)
	now = now or ns.now()
	local today = ns.day(now)
	local idx = ns.DayIndex(today)
	local moves = dayMoves(state, today)
	local old, top
	local function seen(n, d)
		if n < 1 then return end
		if not top or n > top.moves or n == top.moves and d < top.day then top = { moves = n, day = d } end
		if d < today and (not old or n > old.moves or n == old.moves and d < old.day) then
			old = { moves = n, day = d }
		end
	end
	for _, t in ipairs(state.sheet.topDays) do seen(t.moves, t.day) end
	seen(moves, today)
	local window, above = 0, 0
	for d in pairs(state.days) do
		local n = dayMoves(state, d)
		seen(n, d)
		local back = idx - ns.DayIndex(d)
		if n >= 1 and back >= 1 and back <= C.day_rank_days then
			window = window + 1
			if n > moves then above = above + 1 end
		end
	end
	local earlier = state.sheet.activeDays - (moves > 0 and 1 or 0)
	local gate = earlier >= C.best_day_min_days
	local best = gate and moves > 0 and old ~= nil and moves > old.moves
	local quality
	if gate and moves > 0 then
		local shares = C.day_rank_shares
		if best then
			quality = "legendary"
		elseif window > 0 then
			quality = above * 100 < shares[1] * window and "epic" or above * 100 < shares[2] * window and "rare"
				or above * 100 < shares[3] * window and "uncommon" or "common"
		end
	end
	return { moves = moves, earlier = earlier, gate = gate, old = old, best = best, window = window, above = above,
		quality = quality, top = top }
end

-- The Saga's strip of the game day of `now` (rules 9-10): the pumps with a move done that day, each a bar at the
-- minute of its start's wall clock past day_offset_h (0 for a pump begun the game day before) in its bucket by that
-- hour, with its moves that day and whether they were done outside the game (away: the app's, a catch-up's), in minute
-- order (then the order of their first move, `order`); and each bucket's moves. The buckets by the local hour, from
-- `from` up to `to` (night's runs past midnight to 28, 04:00): the day card draws each total over its span.
ns.DAY_BUCKETS = { { key = "morning", from = 4, to = 12 }, { key = "afternoon", from = 12, to = 17 },
	{ key = "evening", from = 17, to = 22 }, { key = "night", from = 22, to = 28 } }

local function bucketOf(hour)
	local h = hour < 4 and hour + 24 or hour
	for _, b in ipairs(ns.DAY_BUCKETS) do
		if h >= b.from and h < b.to then return b.key end
	end
end

function ns.DayStrip(state, now)
	now = now or ns.now()
	local today = ns.day(now)
	local d = state.sheet.days[ns.DayIndex(today)] or {}
	local bars, buckets = {}, {}
	for _, b in ipairs(ns.DAY_BUCKETS) do buckets[b.key] = 0 end
	for i, b in ipairs(d.bars or {}) do
		local minute, hour = 0, C.day_offset_h
		if ns.day(b.ts) == today then
			local t = date("*t", b.ts)
			minute, hour = (t.hour - C.day_offset_h) % 24 * 60 + t.min, t.hour
		end
		local bucket = bucketOf(hour)
		buckets[bucket] = buckets[bucket] + b.n
		bars[#bars + 1] = { ts = b.ts, minute = minute, bucket = bucket, n = b.n, away = b.source ~= E.source.game,
			order = i }
	end
	table.sort(bars, function(a, b)
		if a.minute ~= b.minute then return a.minute < b.minute end
		return a.order < b.order
	end)
	return bars, buckets
end

-- The day's tally (Streamer.lua: the Streamer layout's tally frame and the day card): the game day's pumps, moves and
-- records every source's, as the Today tab counts them (dayCount: the pumps with a move done that day, the moves
-- stored as a record each once, since 2026-10-15), its stars, the unfinished moves, the reps by pattern (the most
-- first), the cardio seconds and the all-out windows finished; and the Saga's day (2026-10-15): its rank
-- (ns.DayRank), the XP gained and the QP level and Paragon rank it began on (history.json saga rule 15: the day's xp,
-- the number the summary's dx carries for it, and the lifetime XP less it and any later day's through the level
-- curve), the streak, the moves by region (ns.REGIONS), the
-- records (rule 12: each move stored as a record, in the order of its first, with the day's best set, `v` and its
-- weight `load`, and its record at the day's start, `was`, nil without one) and the unlocks (rule 13: each step
-- reached for the first time, in time order, with the rung's anchor move and its quality). The records and unlocks
-- took the place of the day's best win (a star or a record) on the day card.
function ns.DayTally(state, now)
	now = now or ns.now()
	local sh = state.sheet
	local today = dayIndex(now)
	local d = sh.days[today] or {}
	local count = dayCount(state, now)
	local reps = {}
	for pattern, n in pairs(d.reps or {}) do reps[#reps + 1] = { pattern = pattern, reps = n } end
	table.sort(reps, function(a, b)
		if a.reps ~= b.reps then return a.reps > b.reps end
		return a.pattern < b.pattern
	end)
	local regions = {}
	for i, region in ipairs(ns.REGIONS) do
		regions[i] = { region = region, moves = d.regions and d.regions[region] or 0 }
	end
	local records = {}
	for i, move in ipairs(d.records or {}) do
		local set, was = d.sets[move], d.was[move]
		records[i] = { move = move, v = set.v, load = set.load, was = was or nil }
	end
	local unlocks = {}
	for i, u in ipairs(d.unlocks or {}) do
		unlocks[i] = { ladder = u.ladder, rung = u.rung, move = ns.Anchor(u.ladder, u.rung),
			quality = ns.Quality(u.ladder, u.rung) }
	end
	-- the level the day began on: the lifetime XP less that of today and of any day dated after it (rule 15)
	local since = 0
	for i, e in pairs(sh.days) do
		if i >= today then since = since + (e.xp or 0) end
	end
	local xp = ns.DayXP(state, now)
	local levelFrom, _, _, paragonFrom = levelOf(sh.xp - since)
	return { pumps = count.pumps, moves = count.moves, records = count.records, stars = d.stars or 0,
		reps = reps, cardio = d.cardio or 0, finishers = d.finishers or 0, unfinished = ns.OwedCount(state),
		rank = ns.DayRank(state, now), xp = xp, levelFrom = levelFrom, paragonFrom = paragonFrom,
		streak = (ns.Streak(state, now)), regions = regions, newRecords = records, unlocks = unlocks }
end

-- The feedback button's numbers (slice 24; contract/feedback-api.yaml, "The counting rules"): counted totals of the
-- whole save, every character, over the current game day and the feedback_days - 1 before it (a day dated ahead by a
-- clock set back counts as today), from what the fold keeps a day so a trim of the ring never shortens them: the
-- pumps finished and banked by the day they were offered (countPump), the median of the max sets a strength or hold
-- ladder took on a day it was trained (DB.lua's ladders' days, less the sets read there off a retired ladder, which
-- count on none), and per move the easier picks and early stops (DB.lua's day counts); with the game days since the
-- earliest row and the moves unfinished now, each from the game's rows only (slice 26: an app row's sets, picks, stops
-- and dates are left out). Keys and shapes as the message carries them: a move by its library key.
function ns.FeedbackNumbers(state, now)
	now = now or ns.now()
	local today = dayIndex(now)
	local from = today - C.feedback_days + 1
	-- a game day's index in the window, a later one as today's; nil before it
	local function inWindow(idx)
		if idx < from then return nil end
		return math.min(idx, today)
	end
	local finished, banked = 0, 0
	for idx, d in pairs(state.sheet.days) do
		if inWindow(idx) then finished, banked = finished + (d.pumpsFinished or 0), banked + (d.pumpsBanked or 0) end
	end
	local perMove = {}
	for key, d in pairs(state.days) do
		if inWindow(ns.DayIndex(key)) then
			for field, byMove in pairs({ easier = d.easier or {}, stopped = d.stopped or {} }) do
				for id, n in pairs(byMove) do
					perMove[id] = perMove[id] or { easier = 0, stopped = 0 }
					perMove[id][field] = perMove[id][field] + n
				end
			end
		end
	end
	-- each ladder's sets a play day, as written (a set the fold read here off a retired ladder counts on none, and an
	-- app row's is no game set), the days a clock set back put ahead joined to today's
	local sets = {}
	for _, l in ipairs(ns.STRENGTH_ORDER) do
		local lad, byDay = state.ladders[l], {}
		local retired, app = lad and lad.retiredDays or {}, lad and lad.appDays or {}
		for key, n in pairs(lad and lad.days or {}) do
			local idx = inWindow(ns.DayIndex(key))
			if idx then byDay[idx] = (byDay[idx] or 0) + n - (retired[key] or 0) - (app[key] or 0) end
		end
		for _, n in pairs(byDay) do
			if n > 0 then sets[#sets + 1] = n end
		end
	end
	table.sort(sets)
	local half = #sets / 2
	local median = #sets == 0 and 0 or #sets % 2 == 1 and sets[math.ceil(half)] or (sets[half] + sets[half + 1]) / 2
	local moves = {}
	for id, n in pairs(perMove) do
		local m = ns.Library[id]
		if m then moves[#moves + 1] = { move = m.key, easier = n.easier, stopped = n.stopped } end
	end
	table.sort(moves, function(a, b)
		local na, nb = a.easier + a.stopped, b.easier + b.stopped
		if na ~= nb then return na > nb end
		return a.move < b.move
	end)
	for i = #moves, C.feedback_moves_max + 1, -1 do moves[i] = nil end
	local first = state.firstTs
	return { days = first and math.max(0, today - dayIndex(first)) or 0, finished = finished, banked = banked,
		owed = ns.OwedCount(state), max_sets = median, moves = moves }
end

-- Titles and deeds ------------------------------------------------------------------------------------------------

-- The closed list (plan slice 3): each with what it counts and its goal, in the order the Deeds tab starts from.

-- The colour titles (slice 20): a ladder's colour earned is the quality of the best rung it ever stood on
-- (ns.LadderColour: unlocks are kept), and each ladder feeds its stat (ns.LadderStat). `reach` says whether the
-- library can give the colour at all (the title's goal passed): a title its ladders cannot reach stays off the Deeds
-- tab (none since slice 22: the L-sit gave Stamina its purple step, so the Tireless and the Exalted can be earned).

-- The ladders of a stat that hold the colour `q` (earned, or `can` at their top).
local function statHas(s, stat, q, can)
	for _, l in ipairs(ns.LADDER_ORDER) do
		if ns.LadderStat(l) == stat and (can and ns.LadderTop(l) or ns.LadderColour(s, l)) >= q then return true end
	end
	return false
end

-- A stat's first family of colour `q` (the Mighty: purple in Strength).
local function colourTitle(stat, q)
	return function(s) return statHas(s, stat, q) and 1 or 0 end, function() return statHas(nil, stat, q, true) end
end

-- How many stats hold colour `q` (the All-Rounder: blue in all five), and whether every stat can.
local function everyStat(q)
	return function(s)
		local n = 0
		for _, stat in ipairs(ns.STATS) do
			if statHas(s, stat, q) then n = n + 1 end
		end
		return n
	end, function()
		for _, stat in ipairs(ns.STATS) do
			if not statHas(nil, stat, q, true) then return false end
		end
		return true
	end
end

-- The ladders that hold colour `q`, and how many can.
local function ladders(q)
	return function(s)
		local n = 0
		for _, l in ipairs(ns.LADDER_ORDER) do
			if ns.LadderColour(s, l) >= q then n = n + 1 end
		end
		return n
	end, function(goal)
		local n = 0
		for _, l in ipairs(ns.LADDER_ORDER) do
			if ns.LadderTop(l) >= q then n = n + 1 end
		end
		return n >= goal
	end
end

local mightyV, mightyR = colourTitle("strength", Q.epic)
local swiftV, swiftR = colourTitle("agility", Q.epic)
local poisedV, poisedR = colourTitle("intellect", Q.epic)
local tirelessV, tirelessR = colourTitle("stamina", Q.epic)
local sereneV, sereneR = colourTitle("spirit", Q.epic)
local roundV, roundR = everyStat(Q.rare)
local exaltedV, exaltedR = everyStat(Q.epic)
local legendV, legendR = ladders(Q.legendary)
local provenV = ladders(Q.rare)

-- A Legendary title (slice 22; approved 2026-10-03): one for each top move, earned with the rung of the move the title
-- names (by a set at its floor since 2026-10-09, ns.BestRung), its ladder's top (orange, or gold since 2026-10-05:
-- Master of the Bar is the one-arm pull-up's) but for the dragon flag's Dragonbane, under the core's gold rollout.
-- Their names start with a capital, so the sheet sets them after a comma ("River Sight, Hand of Iron": Sheet.lua
-- titleText).
local function topTitle(ladder, move)
	-- the move the title names: its ladder's top, or its own rung where a rung came above it (the core's gold
	-- rollout above the dragon flag, 2026-10-05)
	local top = move and ns.Library[move].rung or #ns.Ladders[ladder].rungs
	return function(s) return ns.BestRung(s, ladder) >= top and 1 or 0 end,
		function() return ns.HeldTop(ladder) >= top and ns.Quality(ladder, top) >= Q.legendary end, top
end

-- A retired Climber title's rung-ups: its ladder's, and since the split (slice 21) the pull's on the pull-up and the
-- row, the core hold's on the core and the side, where the fold reads them now.
local function ladderTitle(...)
	local ls = { ... }
	return function(s)
		local n = 0
		for _, l in ipairs(ls) do n = n + (s.sheet.climbs[l] or 0) end
		return n
	end
end

-- The Paragon rank past the cap (ns.Level): the Paragon titles' count (2026-10-09), something to aim for in the years
-- after QP 60.
local function paragonV(s)
	return select(4, ns.Level(s))
end

ns.TITLE_TIERS = { "first", "deed", "rare", "epic", "legendary", "legacy" }

-- Retired titles (2026-10-03, David: "do your recommendations") are no longer earned; a player who holds one keeps it
-- under Legacy. They hung on the quest's type (Circuit Rider, Sprinter, of the Long Road), a way of playing (Hub
-- Hopper), falling behind (Clean Slate) or typed volume (Wall-sitter, Iron-legged), or the colours replace them
-- (Starry-eyed and the seven Climbers, slice 20).
ns.Titles = {
	{ id = "initiate", tier = "first", goal = 1, value = function(s) return s.sheet.counts.pumps end },
	-- five first records, fewer than the Trial's eight ladders, so a setup that leaves ladders no move (the pull-up with
	-- no bar, the wall sit with a knee flag) still earns it, and the fold needs no filter to know
	{ id = "tested", tier = "first", goal = 5, value = function(s) return s.sheet.counts.placed end },
	{ id = "recordBreaker", tier = "first", goal = 1, value = function(s) return s.sheet.counts.beats end },
	{ id = "circuitRider", goal = 1, value = function(s) return s.sheet.counts.t3 end, retired = true },
	{ id = "hubHopper", goal = 1, value = function(s) return s.sheet.counts.merged end, retired = true },
	{ id = "cleanSlate", goal = 1, value = function(s) return s.sheet.counts.clean end, retired = true },
	{ id = "steadfast", tier = "first", goal = 7, value = function(s) return s.sheet.activeDays end },
	{ id = "ding", goal = 1, value = function(s) return s.sheet.counts.dings end, retired = true },
	{ id = "starryEyed", goal = 1, value = function(s) return s.sheet.counts.stars3 end, retired = true },
	{ id = "relentless", tier = "deed", goal = 500, value = function(s) return s.sheet.counts.pumps end },
	-- the week's embers at the vault's last tier (slice 55), the first week that reaches it
	{ id = "kindled", tier = "deed", goal = 1, value = function(s) return s.sheet.counts.kindled end },
	{ id = "wallSitter", goal = 1800, value = function(s) return s.sheet.counts.wallsit end, unit = "seconds",
		retired = true },
	{ id = "ironLegged", goal = 1000, value = function(s) return s.sheet.counts.squats end, retired = true },
	-- a chain (`chain`, the streak's and the Paragon's): titles of one count in rising goals, of which the Deeds tab
	-- shows the ones earned and the next (SheetPages.lua, 2026-10-16)
	{ id = "unbroken", tier = "deed", goal = 30, value = function(s) return s.sheet.best end, chain = "streak" },
	-- the streak's milestones of 100 and 365 days (slice 32), of two and five years (2026-10-09)
	{ id = "unwavering", tier = "deed", goal = 100, value = function(s) return s.sheet.best end, chain = "streak" },
	{ id = "unfailing", tier = "deed", goal = 365, value = function(s) return s.sheet.best end, chain = "streak" },
	{ id = "evergreen", tier = "deed", goal = 730, value = function(s) return s.sheet.best end, chain = "streak" },
	{ id = "undying", tier = "deed", goal = 1825, value = function(s) return s.sheet.best end, chain = "streak" },
	{ id = "sprinter", goal = 50, value = function(s) return s.sheet.counts.finishers end, retired = true },
	{ id = "longRoad", goal = 100, value = function(s) return s.sheet.counts.travelCardio end, retired = true },
	{ id = "wanderer", tier = "deed", goal = 176000, value = function(s) return s.sheet.counts.foot end,
		unit = "yards" },
	-- a QuestPump duel won (slice 23): the profile's count, which Social.lua raises when a duel is settled (no row
	-- carries it, so it emits the title's toast itself)
	{ id = "duelist", tier = "deed", goal = 1, value = function() return ns.profile and ns.profile.duelWins or 0 end },
	-- the Paragon ranks 10, 25, 50 and 100 past QP 60 (2026-10-09)
	{ id = "paragon", tier = "deed", goal = 10, value = paragonV, chain = "paragon" },
	{ id = "ascendant", tier = "deed", goal = 25, value = paragonV, chain = "paragon" },
	{ id = "ageless", tier = "deed", goal = 50, value = paragonV, chain = "paragon" },
	{ id = "eternal", tier = "deed", goal = 100, value = paragonV, chain = "paragon" },
	{ id = "climber1", goal = 1, value = ladderTitle(1), retired = true },
	{ id = "climber2", goal = 1, value = ladderTitle(2), retired = true },
	{ id = "climber3", goal = 1, value = ladderTitle(E.ladder.pullup, E.ladder.row), retired = true },
	{ id = "climber4", goal = 1, value = ladderTitle(4), retired = true },
	{ id = "climber5", goal = 1, value = ladderTitle(E.ladder.core, E.ladder.side), retired = true },
	{ id = "climber6", goal = 1, value = ladderTitle(6), retired = true },
	{ id = "climber7", goal = 1, value = ladderTitle(7), retired = true },
	{ id = "proven", tier = "rare", goal = 1, value = provenV },
	{ id = "allRounder", tier = "rare", goal = #ns.STATS, value = roundV, reach = roundR },
	-- the first purple family in each stat (100 in the stat until 2026-10-03, which every player reached in time)
	{ id = "mighty", tier = "epic", goal = 1, value = mightyV, reach = mightyR },
	{ id = "swift", tier = "epic", goal = 1, value = swiftV, reach = swiftR },
	{ id = "poised", tier = "epic", goal = 1, value = poisedV, reach = poisedR },
	{ id = "tireless", tier = "epic", goal = 1, value = tirelessV, reach = tirelessR },
	{ id = "serene", tier = "epic", goal = 1, value = sereneV, reach = sereneR },
	{ id = "exalted", tier = "epic", goal = #ns.STATS, value = exaltedV, reach = exaltedR },
	{ id = "legendary", tier = "legendary", goal = 3, value = legendV, reach = legendR },
}
for _, t in ipairs({ { "handOfIron", E.ladder.push }, { "masterOfTheBar", E.ladder.pullup },
	{ "flagbearer", E.ladder.side }, { "defierOfGravity", E.ladder.row }, { "worldlifter", E.ladder.handstand },
	{ "wingsOfStone", E.ladder.planche }, { "dragonstrider", E.ladder.squat }, { "dragonbane", E.ladder.core, 321 },
	{ "championOfTheNorth", E.ladder.hinge }, { "stormvaulter", E.ladder.power },
	{ "stormrunner", E.ladder.cardio } }) do
	-- `top`, the step the title names (the Deeds tab folds the ladder tops and shows each with its move's key pose)
	local value, reach, rung = topTitle(t[2], t[3])
	ns.Titles[#ns.Titles + 1] = { id = t[1], tier = "legendary", goal = 1, value = value, reach = reach,
		top = { ladder = t[2], rung = rung } }
end

-- Every title with its progress: { id, value, goal, earned, tier, chain?, top? }, the earned ones first, then the
-- nearest; a retired title (no longer earned) only when it is held, in the "legacy" tier, and a colour title today's
-- moves cannot reach only when it is held. The tiers group the Deeds tab (ns.TITLE_TIERS).
function ns.Deeds(state)
	local out = {}
	for i, t in ipairs(ns.Titles) do
		local goal, value = t.goal, t.value(state)
		local earned
		if t.retired then earned = state.sheet.legacy[t.id] == true else earned = goal > 0 and value >= goal end
		if earned or not (t.retired or (t.reach and not t.reach(t.goal))) then
			out[#out + 1] = { id = t.id, value = math.min(value, goal), goal = goal, earned = earned, unit = t.unit,
				tier = t.retired and "legacy" or t.tier, order = i, chain = t.chain, top = t.top }
		end
	end
	table.sort(out, function(a, b)
		if a.earned ~= b.earned then return a.earned end
		local pa, pb = a.goal > 0 and a.value / a.goal or 0, b.goal > 0 and b.value / b.goal or 0
		if a.earned or pa == pb then return a.order < b.order end
		return pa > pb
	end)
	return out
end

-- The Feats of Strength (slice 22): every orange or gold rung the player earned (Legendary, and Artifact since
-- 2026-10-05; since 2026-10-09 by a set at its floor, ns.EarnFloor, never by its climb alone), with its colour and the
-- date it was earned, the newest first; an empty list before the first (the Deeds tab shows no heading for it then:
-- hidden until earned, as WoW's own). A feat names its rung's anchor: the rung row carries no move.
function ns.Feats(state)
	local out = {}
	for i, l in ipairs(ns.LADDER_ORDER) do
		for r = 1, ns.BestRung(state, l) do
			local q = ns.Quality(l, r)
			if q >= Q.legendary then
				local at = ns.RungEarned(state, l, r)
				out[#out + 1] = { ladder = l, rung = r, move = ns.Anchor(l, r), ts = type(at) == "number" and at or nil,
					order = i * 100 + r, quality = q }
			end
		end
	end
	table.sort(out, function(a, b)
		if (a.ts or 0) ~= (b.ts or 0) then return (a.ts or 0) > (b.ts or 0) end
		return a.order < b.order
	end)
	return out
end

-- The ids of the titles earned now (a set).
function ns.TitlesEarned(state)
	local set = {}
	for _, d in ipairs(ns.Deeds(state)) do
		if d.earned then set[d.id] = true end
	end
	return set
end

-- The titles earned now that were not in `before` (a set from ns.TitlesEarned), in the closed list's order.
function ns.NewTitles(before, state)
	local now, out = ns.TitlesEarned(state), {}
	for _, t in ipairs(ns.Titles) do
		if now[t.id] and not before[t.id] then out[#out + 1] = t.id end
	end
	return out
end

-- The pump card -----------------------------------------------------------------------------------------------------

-- The next star above a record on a rung's window: the value that reaches it and how many stars that is; nil at three.
function ns.NextStar(ladder, rung, moveId, record)
	local lo, hi = ns.Window(ladder, rung, moveId)
	if not lo then return nil end
	local now = ns.Stars(ladder, rung, moveId, record or 0)
	if now >= 3 then return nil end
	for v = (record or 0) + 1, hi do
		local stars = ns.Stars(ladder, rung, moveId, v)
		if stars > now then return v, stars end
	end
	return nil
end

-- What a move can earn before Start (the Rewards section): the XP it pays done, and the conditional chips (the stat a
-- record raises, a star within this set's goal, a familiarity step, a new pattern, the week's practice point of its
-- kind). No chip promises a rung: an unlock takes two days, and the max set that ends at hi only places (no stat).
-- `state` is the fold's.
function ns.RewardPreview(state, slot, now)
	local m = ns.Library[slot.move]
	local sh = state.sheet
	now = now or ns.now()
	local idx = dayIndex(now)
	local stat = ns.MoveStat(slot.move)
	local chips = {}
	local record = slot.kind == K.rep or slot.kind == K.hold or slot.counted
	local hold = slot.kind == K.hold -- the chip's values are seconds
	if record and not slot.practice then
		local best = ns.Record(state, slot.move)
		-- a record is a count above the one standing (ns.Classify), against the slot's own goal (ns.SlotGoal); a loaded
		-- set planned at a weight against its weighted record's reps at that weight (ns.RepsAt, 2026-10-10)
		local topLoad, topReps = ns.LoadRecord(state, slot.move)
		local beat = slot.load and topLoad and ns.RepsAt(ns.LoadScore(topLoad, topReps), slot.load) or best
		if ns.SlotGoal(slot) and best then
			chips[#chips + 1] = { kind = "record", stat = stat, amount = C.stat_record, at = beat, hold = hold }
		end
		local l = slot.ladder
		-- stars count only on the move that decides the rung, at the ladder's current rung (the fold's rule)
		local decides = ns.STRENGTH[l] and slot.rung == ns.Rung(state, l) and ns.StandsFor(l, slot.rung, slot.move)
		if decides and best and slot.target then
			local at, stars = ns.NextStar(l, slot.rung, slot.move, best)
			if at and at <= slot.target then
				chips[#chips + 1] = { kind = "star", stat = ns.LadderStat(l), amount = C.stat_star, stars = stars, at = at,
					hold = hold }
			end
		end
	end
	local fam = sh.fam[slot.move]
	local n = fam and fam.n or 0
	if not (fam and fam.last == idx) and not slot.practice then
		local level = famLevel(n + 1)
		local step = level == 1 and "tried" or level == 2 and "familiar" or level == 3 and "learned" or nil
		-- a step that pays no point (the first finish, since 2026-10-01; a pose family's step another pose paid,
		-- famPays) is no chip
		if step and C.stat_fam[step] > 0 and famPays(sh, m, level) then
			chips[#chips + 1] = { kind = "fam", step = step, stat = stat, amount = C.stat_fam[step] }
		end
	end
	if not sh.patterns[m.pattern] and not slot.practice then
		chips[#chips + 1] = { kind = "pattern", stat = "intellect", amount = C.stat_pattern_new }
	end
	local sameWeek = sh.week == ns.WeekOf(now)
	if not (sameWeek and sh.weekKinds[stat]) and not slot.practice and not slot.standup then
		chips[#chips + 1] = { kind = "practice", stat = stat, amount = C.stat_practice_week }
	end
	local xp = slot.practice and 0 or ns.MoveXP(slot.move, O.done, slot.owed ~= nil,
		{ rung = slot.rung, allout = slot.allout })
	return { xp = xp, stat = stat, chips = chips }
end

-- The voice's callouts (2026-10-09; docs/plan.md slice 1, "The voice") ---------------------------------------------

-- The streak before and after the game day's first finished move, as activeDay would make it (the raw streak it
-- starts from, one more across a gap the locked days fill, the free days on tokens that cover them, else 1); nil when
-- today has a finished move already. The addon's own reading of what the phones' and the desktop app's boards call
-- `afterPump`.
function ns.StreakAhead(state, now)
	local sh = state.sheet
	local today = dayIndex(now or ns.now())
	if sh.activeIdx and sh.activeIdx >= today then return nil end
	if not sh.activeIdx then return 0, 1 end
	local free = today - sh.activeIdx - 1 - covered(sh, today)
	if free <= 0 then return sh.streak, sh.streak + 1 end
	if free <= sh.tokens then return sh.streak, sh.streak + free + 1 end
	return sh.streak, 1
end

-- Each fact of C.callout_order (`c`: the move's slot, nil for a device's catch-up move it did not dose, its index, the
-- pump's count of moves, the state, the filters, now and the streak ahead): the take family it says, or nil. With no
-- slot only the pump's own facts (the milestone, Welcome back, the last move) speak.
local CALLOUT = {
	-- the streak's milestone this pump's first finished move makes (the fold's largest one passed)
	day = function(c)
		if c.i ~= 1 or not c.after then return nil end
		local m = milestone(c.before, c.after)
		return m > 0 and "day" .. m or nil
	end,
	-- Welcome back: the pump's first move after welcome_after_days whole game days with no performed move
	back = function(c)
		local last = c.state.lastDoneTs
		return c.i == 1 and last and dayIndex(c.now) - dayIndex(last) - 1 >= C.welcome_after_days and "back" or nil
	end,
	-- one more day at the window's top opens the next step: the move that decides the rung, its day not counted yet;
	-- only a move of the rung's own level counts a day (DB.lua anchorAttempt), never a stand-in from below
	unlock = function(c)
		local l = c.slot and c.slot.ladder
		local lad = l and c.state.ladders[l]
		if not (lad and ns.STRENGTH[l] and c.slot.rung == ns.Rung(c.state, l)) then return nil end
		if not ns.StandsFor(l, c.slot.rung, c.slot.move) or ns.Library[c.slot.move].rung ~= c.slot.rung then
			return nil
		end
		local have, need, open = ns.UnlockProgress(c.state, l, c.f)
		local counted = lad.hiTs and not ns.NewSession(lad.hiTs, c.now)
		return need > 0 and have == need - 1 and open == 0 and not counted and "unlock" or nil
	end,
	-- a step that has its days and waits only for its path, whose one stage still out this move's day at its top brings
	-- (ns.PathDue's third value, the test the Ready view's next-step line shares: its last day, not counted this
	-- session); a pose family's and a counted cardio rung's since 2026-10-12, a counted set the form's (a timed cardio
	-- move counts nothing toward it); the path in place of the proof since 2026-10-16
	proof = function(c)
		local _, _, gives = ns.PathDue(c.state, c.slot, c.f, c.now)
		return gives and "proof" or nil
	end,
	-- a first max set: a rep move or a hold with no number to reach (never a placement); never a counted set, whose
	-- count is optional and becomes its best only when clean (2026-10-17, slice 42's review: "Whatever you do becomes
	-- the record" said otherwise; its Ready view's words say how its first count counts)
	first = function(c)
		local s = c.slot
		if not s or s.placing then return nil end
		local counts = s.kind == K.rep or s.kind == K.hold
		return counts and ns.SlotGoal(s) == nil and "first" or nil
	end,
	-- a record attempt on a strength or hold ladder that took repeats on the game day of its last set: fresh again
	fresh = function(c)
		if not c.slot then return nil end
		local _, kind = ns.SlotGoal(c.slot)
		local lad = c.slot.ladder and ns.STRENGTH[c.slot.ladder] and c.state.ladders[c.slot.ladder]
		local sets = lad and lad.lastTs and lad.days and lad.days[ns.day(lad.lastTs)] or 0
		return kind == "record" and sets >= 2 and "fresh" or nil
	end,
	record = function(c)
		local _, kind
		if c.slot then _, kind = ns.SlotGoal(c.slot) end
		return kind == "record" and "record" or nil
	end,
	-- the pump's last move, in a pump of callout_last_moves or more
	last = function(c)
		return c.i == c.n and c.n >= C.callout_last_moves and "last" or nil
	end,
}

-- The callout for move `i` of a pump of `n` moves on its Ready view, as the take family it says ("record", "day30"),
-- or nil: the first fact of C.callout_order that holds, the most specific first (Valve's rule base; the phones'
-- Callouts.swift and Callouts.kt are its twins). `slot` is the move's slot: a game pump's or an app pump's, or the one
-- a catch-up on a device dosed its move as where it asks a count (Engine.lua riderSlot, as the game's rider;
-- tools/apppump.lua A.catchupCallout), nil only for a catch-up's move it did not dose (a timed move, a pose, one the
-- rules hold back); `before` and `after` the streak the surface shows and the one the game day's first finished move
-- would make (ns.StreakAhead; nil when today has one). None for /qp try or the stand-up move, and none at all while a
-- pain flag is set or felt unwell locks the day: no line rides on a pump the body said no to. The move's own "Name.
-- Cue." line keeps playing; the callout follows it.
function ns.PickCallout(state, profile, slot, i, n, now, before, after)
	now = now or ns.now()
	if slot and (slot.practice or slot.standup) or next(state.pain) or ns.LockedToday(state, now) then return nil end
	local c = { slot = slot, i = i, n = n, state = state, f = ns.FilterContext(profile, state), now = now,
		before = before or 0, after = after }
	for _, fact in ipairs(C.callout_order) do
		local family = CALLOUT[fact](c)
		if family then return family end
	end
	return nil
end

-- A callout family's takes ("day30" is a "day" fact's, as C.callout_takes counts them): their voice ids, in order.
function ns.CalloutTakes(family)
	local out = {}
	for k = 1, C.callout_takes[family:match("^%a+")] or 0 do out[k] = "callout." .. family .. ".t" .. k end
	return out
end

-- Every callout family in C.callout_order's order, the "day" fact's one per milestone of C.streak_milestones, then the
-- all-out window's shouts, one family per second of C.shout_at_s ("shout15", "shout10"; slice 53, Speech.lua, never a
-- fact ns.PickCallout picks): the lines the voice records (Speech.Lines) and the phones bundle.
function ns.CalloutFamilies()
	local out = {}
	for _, fact in ipairs(C.callout_order) do
		if fact == "day" then
			for _, m in ipairs(C.streak_milestones) do out[#out + 1] = "day" .. m end
		else
			out[#out + 1] = fact
		end
	end
	for _, s in ipairs(C.shout_at_s) do out[#out + 1] = "shout" .. s end
	return out
end

-- A ladder's earned steps (the summary's ear): its best rung (every rung up to it is held: unlocks are kept; an orange
-- or gold rung once earned, ns.BestRung), then per rung the game day it was first reached (an orange or gold one's:
-- earned; Stats.DayIndex), 0 at or under the rung it started on (ns.StartedOn: yours from the start), 1 where the save
-- does not know the day (a sheet from before 2026-10-03 kept true, or no date).
local function earnedSteps(state, l)
	local best, start = ns.BestRung(state, l), ns.StartedOn(state, l)
	local steps = { best }
	for k = 1, best do
		local at = ns.TopTier(l, k) and ns.RungEarned(state, l, k) or ns.RungReached(state, l, k)
		steps[k + 1] = k <= start and 0 or type(at) == "number" and dayIndex(at) or 1
	end
	return steps
end

-- What the summary tells of the last pump (ns.LastPump) beside `last`: its stat points by stat (ls), its unlocks in the
-- order they came (lu: { ladder, rung, the move standing for it now with filter context `f` and the player's `state`
-- (ns.UnlockMove: the open one of its own level; 0 none), its quality as mq counts it, 1 Rekindled, since 2026-10-09
-- how it came: 0 a climb, 1 an orange or gold rung its climb only opened, 2 such a rung a set earned }; nil when none),
-- its game day's start (ld: { the streak's day it began, Welcome back's XP, 1 a rest token earned, the days rest tokens
-- covered, the milestone its streak reached (0 none) }) and the weekly vault's goals it reached (lv: each goal's place
-- in VAULT_ORDER, 1 the moves (2026-10-27; the pumps until then), 2 records, 3 an embers tier (slice 55; the cardio
-- minutes until then), as they came; nil when none). A pump from before 2026-10-06 has none of the newer fields, one
-- from before slice 32 no milestone: they read as zeros.
local function lastPumpParts(p, f, state)
	p = p or {}
	local ls, lu, stats = {}, {}, p.stats or {}
	for i, key in ipairs(ns.STATS) do ls[i] = stats[key] or 0 end
	for _, u in ipairs(p.unlocks or {}) do
		-- a rung a later library retired (as slice 21's split did) stays out, as the fold's climb guard keeps it out
		if ns.Ladders[u.ladder] and ns.Ladders[u.ladder].rungs[u.rung] then
			lu[#lu + 1] = { u.ladder, u.rung, ns.UnlockMove(u.ladder, u.rung, f, state) or 0,
				ns.Quality(u.ladder, u.rung) + 1, u.rekindled and 1 or 0, u.earned and 2 or u.opened and 1 or 0 }
		end
	end
	local lv
	for i, goal in ipairs(p.vault or {}) do
		lv = lv or {}
		lv[i] = goal
	end
	return ls, #lu > 0 and lu or nil,
		{ p.streakDay or 0, p.welcome or 0, p.token and 1 or 0, p.rested or 0, p.milestone or 0 }, lv
end

-- The sheet summary's version since a ladder lost its first rung (ns.ShiftSummary; ns.SheetSummary's comment).
local SHIFTED_V = 6

-- The sheet summary (docs/plan.md slice 5, defined there once): the folded sheet in a few numbers, for the strip's
-- `sheet` event (the helper serves it to the phone and the watch as GET /v1/sheet), the export and /qp qr. Compact
-- keys, arrays in a fixed order (the stats in ns.STATS order, the ladders by id, the muscles by id), whole numbers (the
-- build a word); /qp qr's snapshot of it (QR.lua) stays under ns.C.qr_max_bytes. The phones show the game's summary
-- once one has come, and since slice 27 fold their own app pumps into a summary of this shape (the app's own sheet, a
-- port of this fold), which Hero shows a player without the game. Version 3 since 2026-10-03: mq counts from Starter (1
-- Starter … 6 Legendary, 7 Artifact since 2026-10-05, 0 none), where v 2 counted from Common. Version 4 since slice 21
-- (2026-10-04): lad lists every ladder id from 1 (the split's retired 3 and 5 as zeros, so the list stays whole), a
-- pose ladder's fourth number is the days toward its next step at the rung (ns.PoseNeeds' have, of pose_unlock_days:
-- the need less the counted days still to come, its main pose's and Perfect days among them; the game days its marking
-- pose counted until 2026-10-19, slice 47), 0 where a stand-in from below stands for the rung, and the build is
-- back. Version 5 since slice 27 (2026-10-06), in keys of their own (lad and last keep four numbers, which the phones
-- read as they are): rest and rt (the rested pool as the last performed move left it and that move's time; a
-- reader at a later moment adds the time away since rt as ns.Rested does and counts the streak's gap from rt's game
-- day, the last active one, as ns.Streak does, so the summary changes only with a fold), ttl (the chosen title's code
-- while it is earned, ns.Enum.title; absent when none), the last pump's (ns.LastPump) stat points by stat (ls), unlocks
-- (lu: { ladder, rung, move, quality as mq counts it, 1 Rekindled, since 2026-10-09 0 a climb, 1 an orange or gold
-- rung opened, 2 one earned } each; absent when none) and day (ld: { the streak's
-- day it began, Welcome back's XP, 1 a rest token earned, the days rest tokens covered, since slice 32 the milestone
-- its streak reached }), each ladder's earned steps (ear: { best rung, then per rung 1 … best its game day as
-- Stats.DayIndex, 0 at or under the rung it started on, 1 a day the save does not know }) and its unlock progress
-- (nx: ns.UnlockProgress); since slice 30 the player's focus (fc, the latest focus row's code as the fold keeps it, a
-- trimmed row's through the baseline; absent before a focus row: Balanced, still to be asked), so a phone shows and
-- plans the game's focus where it holds no focus row; since slice 32 the game days locked for feeling unwell after
-- rt's with no performed move (lk: days after it, ascending; absent when none), which a reader's streak skips as
-- ns.Streak does, still v 5 (an older reader skips lk and ld's fifth number); since slice 32's third review the
-- active days the fold counted (ad, from which a reader earns the next rest token as activeDay does) and the streak as
-- rt's day left it (rs, which streak reads as 0 once broken and a later merge of an earlier day can continue), still
-- v 5; since slice 34 the pumps of the sheet's Log (its last sheet_log_pumps) that paid XP, the game's, a catch-up's
-- and an app pump's alike (lx: { the pump's offered time, the XP it paid } each, in the Log's order; absent when
-- none), by which a phone's Log shows each pump's XP, still v 5 (an older reader skips lx); since 2026-10-09 the weekly
-- vault (va: { moves (2026-10-27; pumps until then), records, embers (slice 55; cardio minutes until then), each up to
-- its last tier, then the time the week turns }) and the last pump's vault goals (lv), by which the apps show the vault
-- and the Result's "Weekly goal reached", still v 5; since slice 55 (2026-10-23) the embers lit on at's game day (em,
-- ns.Embers), which a phone's home shows while Hero shows the game's sheet, still v 5; since 2026-10-15 (the Saga's
-- double-check) each covered game day's XP (dx, the last dx_days at most, from dxFrom, the first covered day, which
-- comes even when dx does not: dayXPs) and the two best days of all time (bd, bestDays), by which the apps' Saga reads
-- a day's XP and level and anchors its chapters as the game's (history.json saga rules 3, 4 and 15), still v 5 (an
-- older reader skips them, a reader of an older addon finds none); since slice 44 (2026-10-18) the save's ring horizon
-- (rh, `horizon`: ns.Horizon, 0 while no row has left the ring), which the game's own summaries carry (the strip's and
-- the export's; the snapshot leaves it out) and an app's own never does: the apps' Correct a count lists only the sets
-- after it, which the game's fold still reads, still v 5 (an older reader skips it, a reader of an older addon finds
-- none and bounds nothing); the strip and the snapshot add the figure's colour (fig, ns.SheetPayload). Version 6 since
-- 2026-10-26, when the wall sit lost its 135° sit (Library.lua ns.Shifted): the rungs of a ladder that lost its first
-- rung count from the rung that is first now, which a reader of an older summary reads through ns.ShiftSummary.
-- `profile` is the one whose filters, chosen title and held moves it reads: the bound character's (ns.profile) in the
-- game, the app's own in tools/apppump.lua.
function ns.SheetSummary(state, now, savedAt, profile, horizon)
	now = now or ns.now()
	local sh = state.sheet
	local level, into, need, paragon = ns.Level(state)
	local st, wk = {}, {}
	for i, v in ipairs(ns.StatValues(state, now)) do st[i], wk[i] = v.value, v.value - v.base end
	local f = ns.FilterContext(profile, state)
	-- { rung, the rung's move (0 for cardio), its record, its stars (a pose: its days toward the step) }; all four 0 for a
	-- retired id and a family that is not open yet (ns.Open, slice 22), which the apps leave out as the game's Ladders
	-- tab shows it closed; ear `{ 0 }` for them
	local lad, ear, nx = {}, {}, {}
	for l = 1, ns.LADDER_MAX do
		if not ns.Ladders[l] or ns.HeldTop(l) == 0 or not ns.Open(state, l) then
			lad[l], ear[l] = { 0, 0, 0, 0 }, { 0 }
		else
			local rung = ns.Rung(state, l)
			local id = l ~= ns.CARDIO and ns.RungMove(l, rung, f) or nil
			local record = id and ns.Record(state, id) or nil
			local fourth = record and ns.Stars(l, rung, id, record) or 0
			if ns.POSE[l] then
				fourth = id and ns.MainPose(l, rung, id) and ns.PoseNeeds(state, l, rung).have or 0
			end
			lad[l], ear[l] = { rung, id or 0, record or 0, fourth }, earnedSteps(state, l)
		end
		nx[l] = { ns.UnlockProgress(state, l, f) }
	end
	-- per muscle: its best step's quality as an index of the vocab's qualities (0 none, 1 Starter … 7 Artifact), and
	-- its moves this week
	local mq, mw = {}, {}
	for id, mu in ipairs(ns.BodyMap(state, now, f)) do mq[id], mw[id] = mu.quality and mu.quality + 1 or 0, mu.week end
	local streak, tokens = ns.Streak(state, now)
	local r = ns.Ratings(state, now)
	local p = ns.LastPump(state)
	local ls, lu, ld, lv = lastPumpParts(p, f, state)
	-- va: the weekly vault at `now` (ns.Vault): each goal's progress up to its last tier's need (the embers' tiers paid
	-- are those of vault_embers it reached), then when the week turns; a reader at or past that time reads every goal at 0
	local vault, reset = ns.Vault(state, now)
	local held = ns.OwedHeld(state, profile, now) -- the owed moves a catch-up may not pay now (absent when none)
	local aside = select(3, ns.OwedAside(state, profile, now)) -- those of them no list shows (absent when none)
	local title = ns.ChosenTitle(state, profile) -- every title's rule runs once, and only while one is chosen
	local dx, dxFrom = dayXPs(sh, now)
	-- ws: when the game week began (the region's reset; outside WoW Monday's game day), so the phones' weekly charts bin
	-- as the stats' weekly gain does
	return { v = SHIFTED_V, at = now, ws = ns.WeekStart(now), saved = savedAt, lvl = level, xp = sh.xp, into = into,
		need = need, par = paragon, rest = sh.pool, rt = sh.restTs, build = (ns.Build(state)),
		ttl = title and E.title[title] or nil, fc = state.focus,
		st = st, wk = wk, crit = math.floor(r.crit * 100 + 0.5), exp = r.expertise,
		def = r.defense, streak = streak, tokens = tokens, lad = lad, ear = ear, nx = nx, mq = mq, mw = mw,
		owed = ns.OwedCount(state), held = #held > 0 and held or nil, aside = #aside > 0 and aside or nil,
		lk = lockedAfter(sh),
		last = { p and p.xp or 0, p and p.pts or 0, p and p.stars or 0, p and p.fam or 0 }, ls = ls, lu = lu, ld = ld,
		ad = sh.activeDays, rs = sh.streak, lx = loggedXP(sh), dx = dx, dxFrom = dxFrom, bd = bestDays(sh),
		va = { vault[1].value, vault[2].value, vault[3].value, reset }, em = (ns.Embers(state, now)), lv = lv,
		rh = horizon }
end

-- A sheet summary of a version before SHIFTED_V (an older addon's, an older app's, a phone's or the desktop app's kept
-- one, an export's or a snapshot's) as this one reads it: on each ladder that lost its first rung (Library.lua
-- ns.Shifted) its rung one lower, the first as the first (lad's and each unlock's of lu), and its earned steps
-- (ear) from the rung that is first now, the first rung's day kept as the earlier (ns.ShiftBaseline's rule for the
-- rungs reached); an entry that stood on the rung that went has no move, record or stars (its move is retired: a
-- reader takes the rung's own, as for a 0) and no progress (nx, its climb's rung is the first now); the rest as it
-- was. It marks the summary SHIFTED_V, so it never runs twice, and changes `sum` in place; a summary from SHIFTED_V on
-- as it is.
function ns.ShiftSummary(sum)
	if type(sum) ~= "table" or (tonumber(sum.v) or 0) >= SHIFTED_V then return sum end
	local lower = ns.ShiftedRung
	for l in pairs(ns.Shifted) do
		local lad = type(sum.lad) == "table" and sum.lad[l]
		if type(lad) == "table" then
			if lad[1] == 1 then
				lad[2], lad[3], lad[4] = 0, 0, 0
				local nx = type(sum.nx) == "table" and sum.nx[l]
				for i in ipairs(type(nx) == "table" and nx or {}) do nx[i] = 0 end
			end
			lad[1] = lower(lad[1])
		end
		local ear = type(sum.ear) == "table" and sum.ear[l]
		if type(ear) == "table" and type(ear[1]) == "number" and ear[1] >= 1 then
			if ear[1] >= 2 and #ear >= 3 then table.remove(ear, 3) end -- the second rung's day: the first's is earlier
			ear[1] = lower(ear[1])
		end
		for _, u in ipairs(type(sum.lu) == "table" and sum.lu or {}) do
			if type(u) == "table" and u[1] == l then u[2] = lower(u[2]) end
		end
	end
	sum.v = SHIFTED_V
	return sum
end

-- The baseline -----------------------------------------------------------------------------------------------------

-- The Saga's day detail (sagaMove, a climb's first reach), which the day's displays read for today alone (dayCount,
-- ns.DayTally, ns.DayStrip): the baseline keeps it for today and the day before.
local SAGA_DETAIL = { "bars", "regions", "sets", "was", "records", "appRecordMoves", "unlocks" }

-- Marks the day map whole from game day `idx` on (the summary's dxFrom, dayXPs): a day before it may have gained XP
-- the map no longer holds. It only moves later, as the map loses days.
function Stats.HoldFrom(sheet, idx)
	sheet.heldFrom = math.max(sheet.heldFrom or idx, idx)
end

-- Prunes the day map to what the readers look back on (the 28-day ratings and the week, and with them each day's XP,
-- the summary's dx, which covers the days kept from here on; the Saga's detail two days).
function Stats.Prune(sheet, now)
	if not now then return end
	local today = dayIndex(now)
	local keep = today - math.max(C.rating_days, 8)
	local days = {}
	for i, d in pairs(sheet.days) do
		if i >= keep then
			if i < today - 1 then
				for _, field in ipairs(SAGA_DETAIL) do d[field] = nil end
			end
			days[i] = d
		end
	end
	sheet.days = days
	Stats.HoldFrom(sheet, keep)
end
