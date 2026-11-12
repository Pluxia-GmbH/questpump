-- QuestPumpDB: SavedVariables first, never trusted blindly on the Forever beta (docs/plan.md slice 1, "Persistence").
-- The log is the truth: raw rows in the plan's grammar, and every number the addon shows is folded from them at
-- PLAYER_LOGIN (records, the ladders and their rungs, the cardio ladder and its steady ledger, the clocks selection
-- reads, the owed queue, the felt-unwell day lock, the pain flags, the day counts, and through Stats.lua the sheet: XP,
-- stats, familiarity, the streak, the vault, titles, the body map); nothing derived is stored. The helper's mirror (a
-- separate QuestPump_Mirror addon, outside this repo) restores rows, characters and the profile when it is newer than
-- what the client loaded, brings the other installs' rows and the apps' (one progression, slice 26: an app row moves
-- the records, every clock and the rungs, never the owed queue or the game's day counts; since slice 27 the sheet
-- counts an app pump as a catch-up's, a rung its climb reaches first paid once) and the latest reset made anywhere,
-- and carries the pairing block the setup screen reads.

local ADDON, ns = ...
local E = ns.Enum
local K, O = E.kind, E.outcome

ns.SCHEMA = 2
ns.FIELDS = { "ts", "char", "questID", "tier", "slot", "ladder", "rung", "move", "kind", "reps", "seconds", "outcome",
	"flags" }

-- One profile's defaults, copied in with CopyTable, never shared (a table shared by two keys does not persist).
ns.defaults = {
	enabled = true,    -- General: off, turn-ins count for nothing and the addon sleeps (as disabling it would)
	sounds = true,     -- Blizzard sound kits for the countdown, records and pauses
	toasts = true,     -- the alert-frame toasts (records, stars, titles, levels, the vault)
	hud = true,        -- the HUD line under the objective tracker
	-- /qp lock, without LibEditMode only: the HUD header's drag off (its place, hudPos, stays in the profile once
	-- moved; with LibEditMode, Edit Mode places it, per layout in hudLayouts; Sheet.lua, "The HUD")
	hudLocked = false,
	chat = "warnings", -- chat messages: "off", "warnings" (at most the rare warning) or "all" (every toast too)
	-- General > Layout (slice 23): "standard", or "streamer": no dim, the pump card in a corner (its place, cardPos, made
	-- on the first move, as the day's tally's, tallyPos), and the day's tally on the screen (Streamer.lua)
	layout = "standard",
	-- General > Moves per quest (David, 2026-10-05): "light", "balanced" or "extreme" (ns.Pace; C.pace_moves)
	pace = "balanced",
	figureAnimate = true, -- General: "Animate the move figure" (off shows the key pose, still; slice 10)
	figureColor = "class", -- General: "Figure colour", the sash and crest in the class colour or "crimson"
	mapMode = "week",  -- the sheet's body map: "week" (moves this week), "rungs" or "hero"
	speech = true,     -- the in-client voice reads the move and its cue
	callouts = true,   -- and, after it, one callout from the engine's facts (Speech.lua; 2026-10-09)
	countReps = true,  -- it counts a rep move's reps at the move figure's pace (Speech.lua; David, 2026-10-09: on)
	instances = false, -- /qp instances on: pumps inside dungeons and raids (never in battlegrounds or arenas)
	standup = false,   -- the stand-up move after 30 min without a pump (David, 2026-09-23: off by default)
	deathTax = false,  -- General: each death owes one move, which rides the next pump (slice 23; off by default)
	inputMode = "auto", -- the rep input: "type", "spinner" or "mouse"; "auto" is the spinner with a gamepad
	-- How the player trains (Your setup, /qp gear, slice 3's Equipment, Exercises and Body pages), from the defaults:
	-- floor moves, room to move, not quiet, a chair on wheels, no equipment (David, 2026-09-21: "bodyweight-only
	-- defaults for everyone"; the backpack was on by default until 2026-09-30, when a player without one still met its
	-- row).
	gear = {}, -- equipment keys (ns.Gear), stable_chair, table and towel (2026-11-11) included
	-- Floor moves: hands, forearms or knees on the floor, or lying down, no mat needed. On by default since
	-- 2026-10-04 (David: "Everyone has a floor and a mat is luxury"); off until then, when the game assumed a desk
	-- and no mat. `floorChoice` marks a profile made or switched on since: its `floor` is the player's own
	-- (ns.ProfileFor).
	floor = true,
	floorChoice = true,
	-- Room to move: room for a few steps or a clear 2 x 2 m (skater hops, a lateral lunge, a crawl, a handstand). On by
	-- default since 2026-10-09 (David: "make sure room to move is default on"); a profile from before has it switched
	-- on once (ns.ProfileFor, by `setupAnswered`) and is asked Your setup, where the player can turn it off.
	space = true,
	quiet = false,     -- quiet mode: no loud moves
	noJump = false,    -- Body: no jumping (removes impact moves)
	noStrain = false,  -- Body: avoid breath-hold strain
	noHeadDown = false, -- Body: avoid head-down positions
	exclude = {},      -- the Exercises page's unchecked moves (ids) and groups (names)
	include = {},      -- the opt-in moves the player checked there (ids; the headstand, off until checked: slice 21)
	favourites = {},   -- the moves the player marked (the sheet's move menu): the engine takes them first (slice 20)
	experience = "none", -- none / some / training (ns.Enum.experienceKeys): the start rungs 2 / 3 / 4
	-- Your setup (the setup window's pages after the companions': where the player trains, the equipment, the body,
	-- how hard; Events.lua Setup.Pages) was finished or closed in this profile. Asked once per profile, before the
	-- start (David, 2026-10-09: "We need a proper onboarding also for the WoW addon").
	setupAnswered = false,
	-- The social core (slice 13, Settings > Social).
	share = "circle",  -- who sees the card: "nobody", "circle" (friends, guild and group; David, 2026-09-30) or "anyone"
	shareRecords = false, -- "Include my records": the typed first → best per ladder travels, labelled typed
	showCards = true,  -- "Show others' cards": the tooltip line, the card, the Guild tab, the party line (asks for cards)
	cheers = true,     -- cheers, the Join line and the guildmates' finished pumps on the HUD
	partner = false,   -- the virtual partner's bar on a hold while not in a group (slice 23), off until switched on
	socialOffInInstances = true, -- nothing is sent or shown in a dungeon, a raid, a battleground or an arena
	shareCount = false, -- the count in a share line ("+1 on my best: 23 push-ups"), never by default
	tellLegend = false, -- a Legendary unlock told to the party and the guild (slice 22), only once switched on
	-- pact and duel invites from friends (slice 23), on as cheers are; off, an offer is ignored and the menu entries
	-- hide. The pact itself (`pact`), the duel (`duel`, and `duelPast`, a settled one whose final score the card
	-- still carries) and `duelWins` are no defaults: nil while there is none (Social.lua)
	invites = true,
	-- `milestoneDay`, `bestDay` and `embersDay` are no defaults either: the game day (Stats.DayIndex) of the newest
	-- streak milestone, best day and day's third ember (slice 55) the game toasted (Sheet.lua, Sheet.Toasted), in every
	-- profile because the beta reads no save back and only the profiles return through the mirror, and every profile
	-- alike because the streak, the day's moves and the embers are the account's
}

local MOVE_KIND = { [K.rep] = true, [K.hold] = true, [K.cardio] = true, [K.flow] = true }
-- The outcomes of a typed count (a record, a miss, a placement, censored, last beat), read by tools/apppump.lua (a
-- desktop catch-up's judge) and tools/gen-fixtures.lua
local TYPED = { [O.beat] = true, [O.miss] = true, [O.placement] = true, [O.censored] = true, [O.lastbeat] = true }
ns.TYPED = TYPED

-- Rows: "ts|char|questID|tier|slot|ladder|rung|move|kind|reps|seconds|outcome|flags", non-negative integers only. The
-- conventions the grammar leaves open (recorded in docs/plan.md with it): an owed or banked row keeps the planned
-- seconds of a cardio move in `seconds`, and `reps = 1` on a cardio row marks the all-out window, so an owed window
-- is paid as one, of the move the row names (`reps = 2`, a closing move owed under its jog's id until 2026-10-15, is
-- read alike: Engine.lua isFinisherRow); a counted set's row carries its count in `reps`, count_s in `seconds` and
-- the counted bit, never on an owed row (2026-10-05); a paid row repeats the owed row's ts, questID and slot, which is
-- how the fold matches them; a move row's ladder and rung are the ladder rung it was dosed at (the cardio rung for
-- cardio, 0 off the ladders); a rung row carries the new rung and its cause in `reps` (ns.RungCause); a correction row
-- repeats the corrected row's ts, questID, tier, slot, ladder, rung and move with the corrected count in `reps` (a
-- hold's in `seconds`, a pose's answer to "How close to the goal pose?" in `reps`) and, since slice 44 (2026-10-18),
-- its round in `outcome` (ns.CorrectionRound: one more than the highest round of that set its writer held, at most
-- 255; 0 on a row from before, /qp undo's), the writer's own char and source bits; a checkin row carries the answer
-- in `reps` (1 fine, 2 sore, 3 very sore or dark urine); a clear row its scope in `reps` (E.clear: 1 one move's
-- record, in `move`; 2 a ladder started over, in `ladder`); a load row (2026-10-09) repeats its set row's ts, char,
-- questID, tier, slot, ladder, rung and move with the set's weight in tenths of a kg in `reps` (1 to load_max_kg x 10),
-- seconds and outcome 0 and the set row's source bits in `flags`, written just before the set row, and a set with no
-- weight writes none; a correction's load row (0 takes the weight off) carries the correction's round in `outcome`
-- and its writer's char and source bits, written just before its correction row (ns.CorrectionRows); the rung and
-- mastered rows a correction asks again (and their undo rows) carry the set's char and the time they were written; a
-- row on the retired ladders 3 and 5 is read on the ladders that took their moves, a row of a move merged into
-- another (Library.lua ns.Merged) as that move's and an owed or banked row of reps or a hold whose move is timed now
-- as that timed move's (ns.Translate), never rewritten.
function ns.EncodeRow(r)
	local parts = {}
	for i, field in ipairs(ns.FIELDS) do
		parts[i] = string.format("%d", math.max(0, math.floor(r[field] or 0)))
	end
	return table.concat(parts, "|")
end

function ns.DecodeRow(s)
	if type(s) ~= "string" then return nil end
	local r, i = {}, 0
	for piece in (s .. "|"):gmatch("([^|]*)|") do
		i = i + 1
		local field = ns.FIELDS[i]
		local n = tonumber(piece)
		if not field or not n or n < 0 or n ~= math.floor(n) then return nil end
		r[field] = n
	end
	if i ~= #ns.FIELDS then return nil end
	return r
end

function ns.OwedKey(r)
	return string.format("%d:%d:%d", r.ts, r.questID, r.slot)
end

-- The set a correction or a load row names, as the fold reads a move row: its owed key and its move (a merged move's,
-- the move that took it).
local function setKey(r)
	return ns.OwedKey(r) .. ":" .. (ns.Merged[r.move] or r.move)
end

-- One correction, from any device (slice 44, 2026-10-18): of two correction rows naming one set, or two load rows,
-- whether `a` takes the place of `b`, the one held so far (nil none). The higher round wins (its `outcome`); a tie of
-- rounds goes to the smaller value (a correction's `reps`, then its `seconds`; a load row's weight, the lower), and
-- of two alike the one held stays; rows of round 0 (written before the rounds, /qp undo's and a set's own load row)
-- keep "the last in the log wins" among themselves. So every fold of the same rows, the game's, the helper's and each
-- app's, whatever order its store keeps them in, reads one correction of each set.
function ns.CorrectionBeats(a, b)
	if not b then return true end
	if a.outcome ~= b.outcome then return a.outcome > b.outcome end
	if a.outcome == 0 then return true end
	if a.reps ~= b.reps then return a.reps < b.reps end
	return a.seconds < b.seconds
end

-- The most rounds a set takes: the outcome's one byte on the strip.
local ROUND_MAX = 255

-- The round a new correction of set row `r` takes in state `s` (its corrections and loads, the fold's): one more than
-- the highest round of that set the state holds (a correction's or a load row's), at most ROUND_MAX, where the tie's
-- smaller value then wins.
function ns.CorrectionRound(s, r)
	local key = setKey(r)
	local c, l = s.corrections and s.corrections[key], s.loads and s.loads[key]
	return math.min(ROUND_MAX, math.max(c and c.outcome or 0, l and l.outcome or 0) + 1)
end

-- The weight the fold gives the set row `r` (tenths of a kg; its key's load row that wins, ns.CorrectionBeats; nil
-- none, a load row of 0 too).
function ns.SetLoad(s, r)
	local l = s.loads and s.loads[setKey(r)]
	return l and l.reps > 0 and l.reps or nil
end

-- The save's ring horizon (slice 44): the latest time its baseline folded (ns.Trim's `through`), 0 while no row has
-- left the ring. A set at or before it lives in the baseline alone, so no correction reaches it in the game's fold;
-- the game's sheet summary carries it (rh), from which the apps list only the sets the game's ring still holds.
function ns.Horizon(db)
	local b = type(db) == "table" and type(db.baseline) == "table" and db.baseline or nil
	return b and tonumber(b.through) or 0
end

-- The time a set must be dated after for Correct a count to offer it (slice 44's review, 2026-10-18): a margin of rows
-- past the game's ring horizon `rh` (its sheet's rh, ns.Horizon; nil reads as 0), counted among `rows` dated after it
-- by time (the log a window reads: the game's ring, a device's game and app rows) and the times `more` (nil none: the
-- rows the game's ring holds beside a device's log, which no fold reads there, the game's owed, banked and paid move
-- rows), so a trim before the game's next load never reaches a set a device corrected meanwhile, whose correction the
-- game's merge would then drop (ns.MergeMirror) while the device showed it. A trim cuts the ring back to ring_rows once
-- it runs ring_batch over, so the rows a trim can reach before ring_batch more are written: once a row has left the
-- ring (rh above 0), 2 x ring_batch (the most a full ring holds past ring_rows - ring_batch, whatever the device has
-- heard of it); before that, those past ring_rows - ring_batch, none while fewer (a short log, far from its first
-- trim). The time of the row that ends the margin; rh itself with none; no set (math.huge) where fewer rows than the
-- margin follow a horizon above 0 (a device that has not caught up with the ring).
function ns.CorrectReach(rows, rh, more)
	rh = rh or 0
	local times = {}
	for _, str in ipairs(rows or {}) do
		local r = ns.DecodeRow(str)
		if r and r.ts > rh then times[#times + 1] = r.ts end
	end
	for _, ts in ipairs(more or {}) do
		if ts > rh then times[#times + 1] = ts end
	end
	local spare = ns.C.ring_rows - ns.C.ring_batch
	local margin = rh > 0 and 2 * ns.C.ring_batch or math.max(0, #times - spare)
	if margin == 0 then return rh end
	if #times < margin then return math.huge end
	table.sort(times)
	return times[margin]
end

-- The fold ------------------------------------------------------------------------------------------------------

local DAY_S = 86400
local STRENGTH, CARDIO, POSE = ns.STRENGTH, ns.CARDIO, ns.POSE

-- The split ladders (slice 21, 2026-10-04): the pull ladder (3) and the core hold (5) went in two, and a row of theirs,
-- in this save, in the helper's mirror or in an export, is read on the ladder that took its move, never written again.
-- Both held their rungs on steps 1-8 (rung r on step r), so a rung becomes the highest at or under its step there, by
-- the successors' steps as they stood at the split (SPLIT_STEPS: a later re-step, as the side's Copenhagen raises from
-- 8 to 9 on 2026-10-05, never moves an old row's rung). A rung row names no move: it lands on the ladder of the old
-- ladder's last set (the rung rules write it right after the set that earned it), a climb one rung above that set's
-- rung (the core's plank climbed to the rung above it, where its old step 4 falls in the core's gap), a start on each
-- ladder that has a rung at its step, one with no set before it on its old rung's anchor's ladder; the other ladder is
-- left as it was, for its own first max set to place, so no ladder takes a rung, a climb or a colour the player never
-- trained on it. A set keeps the rung its XP was paid at (`xpRung`).
local LAD = E.ladder
local OLD_KEY = 10 -- a rung's key in a schema-1 sheet: ladder × 10 + rung (ns.RungKey's 100 since schema 2)
local SPLIT = { -- the ladder each old rung's anchor went to, by old rung (ids, so a retired move breaks nothing)
	[E.retired.pull] = { LAD.row, LAD.row, LAD.row, LAD.pullup, LAD.pullup, LAD.pullup, LAD.pullup, LAD.pullup },
	[E.retired.corehold] = { LAD.core, LAD.core, LAD.core, LAD.side, LAD.side, LAD.core, LAD.core, LAD.core },
}
local SUCCESSORS = {
	[E.retired.pull] = { LAD.row, LAD.pullup },
	[E.retired.corehold] = { LAD.core, LAD.side },
}
-- The successors' steps at the split (2026-10-04), up to step 8, the old ladders' last: rung r on SPLIT_STEPS[l][r].
local SPLIT_STEPS = { [LAD.row] = { 1, 2, 3, 4, 5, 6, 7, 8 }, [LAD.pullup] = { 4, 5, 6, 7, 8 },
	[LAD.core] = { 1, 2, 3, 6, 7, 8 }, [LAD.side] = { 4, 5, 6, 7, 8 } }
-- read by tools/gen-engine.lua for the phone cores' library
ns.SPLIT = { rungs = SPLIT, to = SUCCESSORS, steps = SPLIT_STEPS }

-- The highest rung of successor `l` at or under an old rung's step, by the split's steps; nil when its first is higher.
local function splitRung(l, step)
	local found
	for r, at in ipairs(SPLIT_STEPS[l]) do
		if at <= step then found = r end
	end
	return found
end

local function successorOf(old, l)
	return l ~= nil and (l == SUCCESSORS[old][1] or l == SUCCESSORS[old][2])
end

-- Reads one row as the fold reads it, in place, and returns the rows to fold: the row, nothing (a rung under its
-- ladder's first, or a climb to a rung the ladder holds already), or for an old start a second row for the other
-- ladder. `s`, the state the rows fold into, keeps each split ladder's last set (`s.splitSets`: its ladder and rung,
-- kept in the baseline so a trim between a set and its rung row reads them alike); without a state (an owed row in a
-- baseline) a row reads on its move's ladder. A set read off a retired ladder keeps that ladder in `retiredLadder`.
local function translate(r, s)
	-- a clear row's ladder or move is the one the player named, a load row's its set row's: never a set to read again
	if r.kind == K.clear or r.kind == K.load then return r end
	local m = ns.Library[r.move]
	if r.ladder == 0 and r.kind == K.flow and m and POSE[m.ladder] then
		-- a pose done before its family existed (slice 21): read in its family at its rung, paying the flat flow XP it
		-- paid then (every flow at rung 1)
		r.ladder, r.rung, r.xpRung = m.ladder, m.rung, 1
		return r
	end
	local old = SPLIT[r.ladder]
	if not old then return r end
	local was, step = r.ladder, math.max(1, math.min(r.rung, #old))
	if r.kind ~= K.rung then
		local l = m and successorOf(was, m.ladder) and m.ladder or old[step]
		r.ladder, r.rung, r.xpRung, r.retiredLadder = l, splitRung(l, step) or 1, r.rung, was
		if s then s.splitSets[was] = { ladder = l, rung = r.rung } end
		return r
	end
	local cause = ns.RungCause
	if r.reps == cause.start then
		local first, other = SUCCESSORS[was][1], SUCCESSORS[was][2]
		r.ladder, r.rung = first, splitRung(first, step) or 1
		local rung = splitRung(other, step) -- a ladder whose first rung is above the start's step takes no start
		if not rung then return r end
		local second = CopyTable(r)
		second.ladder, second.rung = other, rung
		return r, second
	end
	local last = s and s.splitSets[was]
	local l = last and last.ladder or old[step]
	local rung = splitRung(l, step)
	if last and (r.reps == cause.up or r.reps == cause.placementHigh) then
		rung = last.rung + 1 -- the set's own ladder, one rung up
		if rung > #ns.Ladders[l].rungs then rung = nil end
	end
	if not rung or (r.reps == cause.up and s and rung <= ns.Rung(s, l)) then return nil end
	r.ladder, r.rung = l, rung
	return r
end

-- A move row of a merged move reads as the move that took it (2026-10-04): returns that move, or nil for any other
-- row. Once its ladder is read, `lowered` puts it on that move's own rung where it was dosed above it (the kettlebell
-- and dumbbell deadlifts and the dumbbell row sat a rung above the move that took them, so an old set of theirs is
-- never that rung's attempt), paying the XP of the rung it was paid at. Never a cardio row: its rung is the player's
-- cardio rung when it was dosed, whatever the form's own rung, and the cardio run counts it there.
local function mergedMove(r)
	local to = MOVE_KIND[r.kind] and ns.Merged[r.move]
	if not to then return nil end
	r.move = to
	return ns.Library[to]
end

local function lowered(r, m)
	if m and r.kind ~= K.cardio and r.ladder == m.ladder and r.rung > m.rung then
		r.xpRung, r.rung = r.xpRung or r.rung, m.rung
	end
end

-- An unfinished row (owed or banked) of reps or a hold whose move is timed now (Library.lua's class line: the carries,
-- the good morning, since 2026-11-11 the bird dog and its kin) reads as the row a bank of that move writes today: a
-- flow row at the ladder and rung a fresh pick of it carries (Engine.lua slotOf: 0 off the ladders, at its own rung)
-- and at its own seconds (ns.FlowSeconds), which the move that stands in for it runs too, as for any owed flow row
-- (riderSlot). Every reader of the queue then takes it as the timed move (the riders' dose, a Start that pays it, its
-- stand-in, the lists' kinds and times), never as a set one step past the record its old sets left, and its paid row
-- still names it by its ts, questID and slot. A set done stays as written.
local function timedNow(r)
	local m = (r.outcome == O.owed or r.outcome == O.banked) and (r.kind == K.rep or r.kind == K.hold)
		and ns.Library[r.move]
	if m and m.dose == "flow" then
		r.kind, r.ladder, r.rung, r.seconds = K.flow, m.ladder or 0, m.rung, ns.FlowSeconds(r.move)
	end
end

-- A set of a move that switches sides since a time (Library.lua ns.Sided) from before that time held both sides on one
-- clock, so its seconds read as each side's: half (its correction's too, `correct`).
local function sidedHalf(r)
	local at = ns.Sided[r.move]
	if at and r.kind == K.hold and r.ts < at then
		r.halved, r.seconds = true, math.floor(r.seconds / 2)
	end
end

-- A rung one lower on a ladder that lost its first rung, the first as the first; 0 or none as it was (a row's, a
-- baseline's, a sheet summary's: Stats.lua ns.ShiftSummary).
local function lowerRung(rung)
	return type(rung) == "number" and rung >= 1 and math.max(1, rung - 1) or rung
end
ns.ShiftedRung = lowerRung

-- A row of a ladder that lost its first rung (Library.lua ns.Shifted, the wall sit's 135° sit on 2026-10-26), written
-- before that moment, names the rung as it stood then: it reads one rung lower, the first as the first, in place, a set
-- paying the XP of the rung it was paid at (`xpRung`); a mastered row of the first rung reads as nothing (nil), since
-- the step it mastered is gone. Every reader of a stored row's rung reads it so: the fold (ns.Translate), the undo
-- pairing, the latest rung row (ns.LatestRung) and Correct a count's step rows and count check (Progress.lua
-- stepRowsAfter, ns.CorrectionCheck). A row keeps its stored rung on disk, and a correction of a set names the set's
-- (its own time's, so it reads as the set does).
function ns.ShiftRow(r)
	local at = ns.Shifted[r.ladder]
	if not at or r.ts >= at or r.rung < 1 then return r end
	if r.kind == K.mastered and r.rung == 1 then return nil end
	if MOVE_KIND[r.kind] then r.xpRung = r.xpRung or r.rung end
	r.rung = lowerRung(r.rung)
	return r
end

-- A row as the fold reads it, but one of a ladder that lost its first rung as written where `unshifted` (a schema-1
-- baseline's owed rows, ns.SplitBaseline, which ns.ShiftBaseline lowers with the rest of that baseline).
local function readRow(r, s, unshifted)
	local m = mergedMove(r)
	timedNow(r)
	sidedHalf(r)
	if not (unshifted or ns.ShiftRow(r)) then return nil end
	local a, b = translate(r, s)
	if a then lowered(a, m) end
	return a, b
end

function ns.Translate(r, s)
	return readRow(r, s)
end

-- A baseline from schema 1 (a save, or an export a save takes in): each old rung read on the successor whose rung on
-- that step the player set a move of (the most recent of them where both; else the old rung's anchor's ladder), for
-- the ladder's state at its old rung as for each rung reached, and the sheet's rungs reached keyed ladder × 100 + rung
-- (ns.RungKey). It marks the baseline `split`, as every baseline ns.Trim writes since is, so it never runs twice
-- (on the beta the mirror's rows are merged, and their baseline written by this fold, before the migrations run).
function ns.SplitBaseline(b)
	if type(b) ~= "table" or b.split then return end
	b.split = true
	local moves = type(b.moves) == "table" and b.moves or {}
	local sheet = type(b.sheet) == "table" and b.sheet or nil
	local reached = sheet and type(sheet.rungs) == "table" and sheet.rungs or nil
	-- the latest set of a move of ladder `l`'s rung on `step` exactly (that old rung, trained there), or nil
	local function setOn(l, step)
		local rung = splitRung(l, step)
		if not rung or SPLIT_STEPS[l][rung] ~= step then return nil end
		local at
		for id, mv in pairs(moves) do
			local m = ns.Library[id]
			if m and m.ladder == l and m.rung == rung then
				at = math.max(at or 0, type(mv) == "table" and tonumber(mv.lastTs) or 0)
			end
		end
		return at
	end
	-- the successor an old rung is read on: where the player set a move of it (the latest where both), else its anchor's
	local function home(old, step)
		local l, at = SPLIT[old][step], nil
		for _, s in ipairs(SUCCESSORS[old]) do
			local t = setOn(s, step)
			if t and (not at or t > at) then l, at = s, t end
		end
		return l
	end
	local lads = type(b.ladders) == "table" and b.ladders or {}
	local kept = {}
	for old, byRung in pairs(SPLIT) do
		local lad = lads[old]
		lads[old] = nil
		local l = SUCCESSORS[old][1]
		if type(lad) == "table" then
			-- one from before 2026-10-03 kept each rung-up's date on its ladder (`ups`): onto the sheet's old keys first
			for rung, ts in pairs(type(lad.ups) == "table" and lad.ups or {}) do
				local key = old * OLD_KEY + rung
				if reached and reached[key] == true then reached[key] = ts end
			end
			lad.ups = nil
			l = home(old, math.max(1, math.min(tonumber(lad.rung) or 1, #byRung)))
			if not lads[l] then
				lad.rung = lad.rung and (splitRung(l, lad.rung) or 1) or nil
				lad.from = lad.from and splitRung(l, lad.from) or nil
				lad.start = lad.start and splitRung(l, lad.start) or nil
				-- its sets a day were the retired ladder's, as its rows' are (translate)
				lad.retiredDays = type(lad.days) == "table" and CopyTable(lad.days) or nil
				lads[l] = lad
			end
		end
		kept[old] = l
	end
	if reached then
		local rungs = {}
		local function mark(l, rung, at)
			local key = ns.RungKey(l, rung)
			if rungs[key] == nil or (type(at) == "number" and (type(rungs[key]) ~= "number" or at < rungs[key])) then
				rungs[key] = at
			end
		end
		for key, at in pairs(reached) do
			local l, rung = math.floor(key / OLD_KEY), key % OLD_KEY
			local byRung = SPLIT[l]
			if byRung then
				local any = false
				for _, s in ipairs(SUCCESSORS[l]) do
					if setOn(s, rung) then
						mark(s, splitRung(s, rung), at)
						any = true
					end
				end
				local anchor = byRung[math.max(1, math.min(rung, #byRung))]
				local on = not any and splitRung(anchor, rung)
				if on then mark(anchor, on, at) end
			elseif ns.Ladders[l] then
				mark(l, rung, at)
			end
		end
		sheet.rungs = rungs
	end
	for _, field in ipairs({ "climbs", "weekSets" }) do
		local t = sheet and type(sheet[field]) == "table" and sheet[field]
		for old in pairs(t and SPLIT or {}) do
			local to = kept[old]
			if t[old] then t[to], t[old] = (t[to] or 0) + t[old], nil end
		end
	end
	for _, mu in pairs(sheet and type(sheet.muscles) == "table" and sheet.muscles or {}) do
		local m = type(mu) == "table" and mu.best and ns.Library[mu.best]
		local scale = m and ns.ScaleLadder(m)
		-- a muscle from before 2026-10-03 had no step: on a split ladder its old rung was its step
		if scale and mu.step == nil and (successorOf(E.retired.pull, scale) or successorOf(E.retired.corehold, scale)) then
			mu.step = mu.rung
		end
		if scale and (mu.step or 0) > 0 then mu.rung = ns.RungAtStep(scale, mu.step) or mu.rung end
	end
	for key, str in pairs(type(b.owed) == "table" and b.owed or {}) do
		local r = ns.DecodeRow(str)
		if r and r.kind ~= K.rung then
			readRow(r, nil, true)
			b.owed[key] = ns.EncodeRow(r)
		end
	end
end

-- A baseline folded before a ladder lost its first rung (Library.lua ns.Shifted) holds that ladder's rungs as they
-- stood: one that names no such shift of the ladder (`shifted`, which every baseline ns.Trim writes since names) and
-- whose last folded row (`through`, 0 for none) is from before that moment reads each one rung lower, the first as
-- the first, as its rows do (ns.ShiftRow): the ladder's rung, the rung it came from and its start, the sheet's rungs
-- reached and earned and the steps mastered (ladder × 100 + rung, ns.RungKey; of two that meet the earlier, a
-- mastered first rung none), the rung of a muscle whose best move is on the ladder (from its step, which stays), the
-- pump's and the Saga days' unlocks and the owed rows (a schema-1 baseline's too: ns.SplitBaseline reads them
-- unshifted, in a save's migration as on a copy). It marks the baseline, so it never runs twice.
local function rekeyed(map, l, dropFirst)
	if type(map) ~= "table" then return end
	local moved = {}
	for key, at in pairs(map) do
		if type(key) == "number" and math.floor(key / 100) == l then
			map[key] = nil
			local rung = key % 100
			if not (dropFirst and rung == 1) then
				moved[#moved + 1] = { key = ns.RungKey(l, lowerRung(rung)), at = at }
			end
		end
	end
	for _, e in ipairs(moved) do
		local have = map[e.key]
		if have == nil or (type(e.at) == "number" and (type(have) ~= "number" or e.at < have)) then
			map[e.key] = e.at
		end
	end
end

local function shiftUnlocks(list, l)
	for _, u in ipairs(type(list) == "table" and list or {}) do
		if type(u) == "table" and u.ladder == l then u.rung = lowerRung(u.rung) end
	end
end

-- Whether baseline `b` is from before some ladder lost its first rung and names no shift of it.
local function shiftDue(b, l)
	return not (type(b.shifted) == "table" and b.shifted[l]) and (tonumber(b.through) or 0) < ns.Shifted[l]
end

function ns.ShiftDue(b)
	for l in pairs(ns.Shifted) do
		if shiftDue(b, l) then return true end
	end
	return false
end

function ns.ShiftBaseline(b)
	if type(b) ~= "table" then return end
	b.shifted = type(b.shifted) == "table" and b.shifted or {}
	local sheet = type(b.sheet) == "table" and b.sheet or {}
	for l in pairs(ns.Shifted) do
		local due = shiftDue(b, l)
		b.shifted[l] = true
		if due then
			local lad = type(b.ladders) == "table" and b.ladders[l]
			if type(lad) == "table" then
				lad.rung, lad.from, lad.start = lowerRung(lad.rung), lowerRung(lad.from), lowerRung(lad.start)
			end
			rekeyed(sheet.rungs, l)
			rekeyed(sheet.earned, l)
			rekeyed(b.mastered, l, true)
			for _, mu in pairs(type(sheet.muscles) == "table" and sheet.muscles or {}) do
				local m = type(mu) == "table" and mu.best and ns.Library[mu.best]
				if m and ns.ScaleLadder(m) == l and (mu.rung or 0) > 0 then
					mu.rung = (mu.step or 0) > 0 and ns.RungAtStep(l, mu.step) or lowerRung(mu.rung)
				end
			end
			for _, field in ipairs({ "pump", "appPump" }) do
				if type(sheet[field]) == "table" then shiftUnlocks(sheet[field].unlocks, l) end
			end
			for _, d in pairs(type(sheet.days) == "table" and sheet.days or {}) do
				if type(d) == "table" then shiftUnlocks(d.unlocks, l) end
			end
			for key, str in pairs(type(b.owed) == "table" and b.owed or {}) do
				local r = ns.DecodeRow(str)
				if r and r.ladder == l and r.rung >= 1 then
					r.rung = lowerRung(r.rung)
					b.owed[key] = ns.EncodeRow(r)
				end
			end
		end
	end
end

-- A merged move's record joins the record of the move that took it: the better best, the stronger weighted record
-- (ns.LoadScore), the later last set and form with their weights, the earlier first count.
local function mergedRecord(a, b)
	local m = CopyTable(a)
	if (b.best or 0) > (a.best or 0) then m.prev, m.best, m.bestTs = a.best, b.best, b.bestTs end
	if b.topLoad and (not a.topLoad or ns.LoadScore(b.topLoad, b.topReps) > ns.LoadScore(a.topLoad, a.topReps)) then
		m.topLoad, m.topReps, m.topTs = b.topLoad, b.topReps, b.topTs
	end
	if (b.lastTs or 0) > (a.lastTs or 0) then
		m.last, m.lastTs, m.lastLoad, m.fresh, m.freshLoad = b.last, b.lastTs, b.lastLoad, b.fresh, b.freshLoad
	end
	if (b.firstTs or math.huge) < (a.firstTs or math.huge) then m.first, m.firstTs = b.first, b.firstTs end
	return m
end

-- A baseline written before 2026-10-04 keeps a merged move's entries by its old id (Library.lua ns.Merged): its record,
-- its sets of the last two days, the recent pumps' moves, and the sheet's familiarity, body map, Log and last gains.
-- Each is read as the move that took it, as its rows are (ns.Translate); a baseline written since holds none.
local function mergeBaseline(s)
	local sh = s.sheet
	for old, to in pairs(ns.Merged) do
		local a, b = s.moves[to], s.moves[old]
		if b then s.moves[to], s.moves[old] = a and mergedRecord(a, b) or b, nil end
		if s.seen[old] then s.seen[to], s.seen[old] = true, nil end
		for _, d in pairs(s.days) do
			local done = d.done
			if done and done[old] then done[to], done[old] = (done[to] or 0) + done[old], nil end
		end
		for _, p in ipairs(s.recent) do
			if p.moves[old] then p.moves[to], p.moves[old] = true, nil end
		end
		local fa, fb = sh.fam[to], sh.fam[old]
		if fb then
			sh.fam[to] = fa and { n = math.max(fa.n or 0, fb.n or 0), last = math.max(fa.last or 0, fb.last or 0) } or fb
			sh.fam[old] = nil
		end
		for _, mu in pairs(sh.muscles) do
			if mu.best == old then mu.best = to end
			for _, e in ipairs(mu.last or {}) do
				if e.move == old then e.move = to end
			end
		end
		for _, e in ipairs(sh.log) do
			for _, mv in ipairs(e.moves or {}) do
				if mv.move == old then mv.move = to end
			end
		end
		for _, gain in pairs(sh.last) do
			if gain.move == old then gain.move = to end
		end
	end
end

-- What survives the ring in the baseline: every folded table and time below except the volatile ones.
local KEEP = { "moves", "pain", "pause", "moveRows", "anyMove", "paidEarly", "days", "unwellTs", "ladders",
	"cardio", "regions", "patterns", "recent", "seen", "firstTs", "createdTs", "breakTs", "lastDoneTs", "lastMove",
	"checkinTs", "focus", "sheet", "splitSets", "cleared", "marks", "mastered", "ledger", "formTs" }

-- A ladder's counts toward a rung up or down start again: at every rung row and at a break's placement.
local function resetCounts(lad)
	lad.hiDays, lad.hiTs, lad.plateau, lad.plateauTs, lad.underLo, lad.halfLo = 0, nil, 0, nil, 0, nil
	lad.anchorDays, lad.perfectDays = 0, 0
end

-- The cardio ladder's counters a new state starts from, and a baseline that lacks one takes.
local function cardioDefaults()
	return { stops = 0, clean = 0, cleanForms = {}, cleanFormTs = {}, cleanEver = {}, full = 0, haul = 1, haulDays = 0 }
end

-- The moment before which a ladder's mastered row masters nothing (applyRow's mastered rows, and a baseline's in
-- NewState): a pose family's before slice 47's goal poses (pose_mastery_ts), the cardio ladder's before its forms'
-- first clean windows came from answered windows alone (cardio_mastery_ts, 2026-10-31: until then a window nobody
-- answered set one); nil for a strength ladder, whose row of any date counts.
local function masteryCut(l)
	return POSE[l] and ns.C.pose_mastery_ts or l == CARDIO and ns.C.cardio_mastery_ts or nil
end

-- A state starts from the baseline (what fell out of the ring) and takes every row in order through ns.Apply. A
-- baseline from before the split ladders (slice 21) is read split, on a copy: the save's own is split by its migration.
function ns.NewState(baseline)
	local split = type(baseline) == "table" and not baseline.split
	if split then
		baseline = CopyTable(baseline)
		ns.SplitBaseline(baseline)
	end
	if type(baseline) == "table" and ns.ShiftDue(baseline) then
		if not split then baseline = CopyTable(baseline) end
		ns.ShiftBaseline(baseline)
	end
	local s = {
		-- [move id] = { best, bestTs, prev, last, lastTs, first, firstTs, fresh, easier, easierUntil, freshLoad,
		-- lastLoad, topLoad, topReps, topTs, topDays, topDayTs }: the typed counts, `fresh` the last goal_form_n made
		-- fresh (ns.FreshAt) or above the form (recordCount), oldest first, that ns.Goal reads the player's form from,
		-- `easier` a set taken as the easier version and `easierUntil` the count that clears it (easierMark); a loaded
		-- move's weights in tenths of a kg (2026-10-09; `best` stays the most reps at any weight): each fresh count's
		-- (`freshLoad`, 0 for none), the last set's, and the weighted record, the strongest set with a weight
		-- (ns.LoadScore; a tie keeps the older); the game days a set reached the move's own window's top at a weight that
		-- climbs and the set that counted the last (`topDays`, `topDayTs`, 2026-10-16: a step's path, topDay)
		moves = {},
		cleared = {}, -- [move id] = the latest clear row that dropped its record (clearRow), its own or its ladder's
		-- [set key] = { value, outcome, move }: a corrected set as the fold read it (correct, slice 44), its count (a
		-- hold's seconds, a pose's answer), outcome and move (as the fold reads it: tools/apppump.lua's setAside,
		-- 2026-10-19), which ns.SetValue and ns.SetOutcome read: only the fold holds the state before the set that judged
		-- it (a capped max set's top, ns.CappedSet; a count of 0 done)
		fixed = {},
		-- [pose id] = { good, goodTs, perfect, perfectTs, best, goodFirst, perfectFirst, askTs, heldTs }: the game days
		-- its mark was answered Good or better, and Perfect, each a new session by the night rule after the one at its ts
		-- (markDays, 2026-10-12); the closest answer asked (slice 42), the first asked Good or better and Perfect
		-- (2026-10-17), the last answer asked (slice 47) and the last hold its whole time (2026-10-20)
		marks = {},
		splitSets = {}, -- [retired ladder] = { ladder, rung } its last set was read on (ns.Translate)
		owed = {},      -- [owed key] = the owed or banked row, until a paid row names it
		pain = {},      -- [joint id] = when the flag was set
		pause = {},     -- [char index] = { ts, mode }: the character's pause (ns.Paused), until a resume row
		moveRows = 0,   -- move rows ever written, owed ones included
		anyMove = false, -- a pump or a move exists: the Trial has begun
		paidEarly = {}, -- a paid row seen before its owed row (a merge out of order); the owed row is then skipped
		-- [ns.day] = { pumps, moves, records, banked, blocks = { [plan block] = moves }, done = { [move id] = sets } };
		-- the counts are the game's rows' (slice 26), the two maps every source's: they feed the day plan and its
		-- rotation (Engine.lua) and are kept two days. The feedback numbers (slice 24, ns.FeedbackNumbers) add, by
		-- move id, the game's performed rows taken as the easier version (`easier`) and stopped early (`stopped`)
		days = {},
		unwellTs = nil, -- the latest "felt unwell" row
		-- [ladder] = { rung, from, anchorTs (the anchor's last set at its rung), standTs (a stand-in's: anchorAttempt),
		-- lastTs, opened, placing, placedTs, hiDays and hiTs (the days the anchor reached the window's top at this rung, the
		-- unlock, and the last set counted; on a pose family the days a goal pose of its step counted, markAttempt),
		-- anchorDays and perfectDays (a pose family's days among hiDays on its main pose and answered Perfect, slice 47:
		-- ns.PoseNeeds; 0 on every other ladder), plateau and plateauTs, underLo, halfLo, days, retiredDays (the sets of
		-- `days` read here off a retired ladder, which the feedback numbers count on none), appDays (the sets of `days` an
		-- app row gave, slice 26, which the game's feedback numbers leave out), start (the rung the start or a lower Trial
		-- placement gave; the rungs reached are the sheet's, ns.BestRung) and restartTs (when the player last started it
		-- over, clearRow: the rungs reached before it count no more) }
		ladders = {},
		-- the cardio ladder (cardioMove): rung, stops in a row, the last move, the start rung (as on the ladders);
		-- since 2026-10-16 (slice 42) a counted rung's clean days (`clean`, the last at cleanTs) and each countable
		-- form's among them (cleanForms, cleanFormTs), each counted at the rung, and each form's first clean window at
		-- any rung (cleanEver, a step's mastery); since slice 52 a timed rung's full moves (`full`, the last at fullTs,
		-- steadyRow); its finished run and the run's days (a timed rung) and a counted rung's top counts until then, a
		-- timed rung's ready days and P (pSteps, weeks) until slice 52
		cardio = cardioDefaults(),
		-- [move id] = its latest row's ts, any source, of a move that keeps no record and no marks: a cardio form (slice
		-- 52, 2026-10-23: the fair draw's clocks of each form and road, Engine.lua roadClocks and fairDraw, which read the
		-- forms with a road alone) and, since 2026-10-31, a flow move outside a pose family (the carries and the good
		-- morning, which kept no clock); through ns.DoneAt the ties of a path's turn and of the picks (Progress.lua
		-- PathTurn, Engine.lua byScore)
		formTs = {},
		-- [ladder × 100 + rung] = the first mastered row's ts (2026-10-16, slice 42: ns.Mastered), kept through a step
		-- down and a setup change; a strength ladder started over drops its own (clearRow, 2026-10-31)
		mastered = {},
		-- the Stamina Ledger (slice 52): `holds`, [ladder] = the hold ledger's entry of each ladder of ns.HOLD_LEDGER
		-- (holdLedger; Progress.lua ns.HoldEntry names its fields), and `steady`, what the player keeps of cardio's
		-- steady pace (steadyRow; its fields at Progress.lua's "The steady ledger"), created at the first steady row
		ledger = { holds = {} },
		regions = {},   -- [region] = { days }: the max sets a region took each day (the Today tab)
		patterns = {},  -- [pattern] = { lastTs, days, acc = { [half-life h] = { v, ts } } }: due, variety, load
		recent = {},    -- the last five pumps: { moves = set, patterns = set }
		seen = {},      -- [move id] = true once performed (a first-ever move)
		firstTs = nil,  -- the earliest game row of any kind: the feedback numbers' days since the start
		createdTs = nil, breakTs = nil, lastDoneTs = nil, checkinTs = nil,
		lastMove = nil, -- the move of the latest performed row (any source), after which a catch-up spreads its owed moves
		focus = nil,    -- the latest focus row's code (ns.Focus; nil: never chosen, Balanced, and still to be asked)
		sheet = ns.Stats.New(), -- the sheet's running totals (Stats.lua)
	}
	if type(baseline) == "table" then
		for _, key in ipairs(KEEP) do
			local v = baseline[key]
			if type(v) == "table" then
				s[key] = CopyTable(v)
			elseif v ~= nil and type(v) == type(s[key] or v) then
				s[key] = v
			end
		end
		for key, str in pairs(type(baseline.owed) == "table" and baseline.owed or {}) do
			local r = ns.DecodeRow(str)
			if r then
				lowered(r, mergedMove(r))
				timedNow(r)
				r.key = key
				s.owed[key] = r
			end
		end
		s.moveRows = type(s.moveRows) == "number" and s.moveRows or 0
		s.anyMove = s.anyMove == true
		-- a baseline from before slice 24 kept no earliest row: the first move it performed (createdTs) is the
		-- earliest it knows, where the game began (anyMove; an app row moves createdTs, never anyMove or firstTs)
		if s.firstTs == nil and s.anyMove then s.firstTs = s.createdTs end
		-- a baseline from before 2026-09-29 kept `halfLo` as true (one attempt under lo / 2); now it counts them; one
		-- from before 2026-10-03 counted sets at the top (atHi) and near it (plateau), where both now count days
		-- one from before 2026-10-19 (slice 47) kept no main-pose or Perfect days: a pose family's days were all on its
		-- main pose then, and all Perfect where only Perfect counted (a Good step's Perfect days are not known, so one
		-- answer Perfect still counts toward them; nor can a day there that a row from before the ask brought be told
		-- apart, though such a row counts no day since 2026-10-31 and no Perfect one since 2026-10-26, markAttempt, so
		-- it stays)
		for l, lad in pairs(s.ladders) do
			if type(lad) == "table" then
				if lad.halfLo == true then lad.halfLo = 1 end
				if lad.hiDays == nil then lad.hiDays, lad.atHi, lad.plateau = 0, nil, 0 end
				if lad.anchorDays == nil then
					local pose = POSE[l] and ns.Ladders[l] ~= nil
					local perfectOnly = pose and ns.MarkNeeds(l, ns.Rung(s, l)) >= E.mark.perfect
					lad.anchorDays = pose and lad.hiDays or 0
					lad.perfectDays = perfectOnly and lad.hiDays or 0
				end
			end
		end
		for field, default in pairs(cardioDefaults()) do
			if s.cardio[field] == nil then s.cardio[field] = default end
		end
		-- one folded before 2026-10-31 kept the steps mastered that its rows no longer master (the mastered row and
		-- clearRow, below): a pose family's and the cardio ladder's from before their cut (masteryCut) and a strength
		-- ladder's from before it was started over (restartTs; a step mastered again since cannot be told apart, so the
		-- sweep, ns.MasterySweep, writes it again where its moves still hold their tops); and a form's first clean window
		-- from before counted_ask_ts, which nothing asked (cardioMove: the form's next answered window sets it again)
		for l, w in pairs(ns.Ladders) do
			local lad = s.ladders[l]
			local before = masteryCut(l) or STRENGTH[l] and type(lad) == "table" and lad.restartTs
			for rung in ipairs(before and w.rungs or {}) do
				local key = ns.RungKey(l, rung)
				if type(s.mastered[key]) == "number" and s.mastered[key] < before then s.mastered[key] = nil end
			end
		end
		for id, ts in pairs(s.cardio.cleanEver) do
			if type(ts) == "number" and ts < ns.C.counted_ask_ts then s.cardio.cleanEver[id] = nil end
		end
		-- a baseline from before slice 52 kept no ledger: its next sets start it (its cardio state's `pSteps` stays for
		-- the steady ledger's first row, steadyRow)
		if type(s.ledger.holds) ~= "table" then s.ledger.holds = {} end
		-- a baseline from an older addon lacks the sheet's newer fields, at the top and inside its counters: they start
		-- as a new sheet's
		if type(s.sheet) ~= "table" then s.sheet = ns.Stats.New() end
		local legacy = s.sheet.legacy == nil -- an older addon's baseline: its counters predate the retired titles
		local untopped = s.sheet.topDays == nil -- one from before the Saga's best days (2026-10-15)
		-- one from before slice 55, whose vault counted the week's cardio seconds (`cardio`): the card's minutes keep them
		local unembered = s.sheet.weekCardio == nil and type(s.sheet.vault) == "table" and s.sheet.vault.cardio
		-- one from before 2026-10-27, whose vault counted the week's pumps (`pumps`): the card's pumps keep them, the
		-- vault's moves start from the rows the ring holds, and a pumps tier paid this week stands for the moves' tier
		local pumped = s.sheet.weekPumps == nil and type(s.sheet.vault) == "table" and s.sheet.vault.pumps
		for field, default in pairs(ns.Stats.New()) do
			local have = s.sheet[field]
			if have == nil then
				s.sheet[field] = default
			elseif type(default) == "table" and type(have) == "table" then
				for k, v in pairs(default) do
					if have[k] == nil then have[k] = v end
				end
			end
		end
		if unembered then s.sheet.weekCardio, s.sheet.vault.cardio = unembered, nil end
		if pumped then
			s.sheet.weekPumps, s.sheet.vault.pumps = pumped, nil
			local paid = s.sheet.paid
			if type(paid) == "table" and paid.pumps ~= nil then paid.moves, paid.pumps = paid.moves or paid.pumps, nil end
		end
		if legacy then
			for _, t in ipairs(ns.Titles) do
				if t.retired and t.value(s) >= t.goal then s.sheet.legacy[t.id] = true end
			end
		end
		-- its best days start from the day counts it holds (60 days; the game's alone counted its best day until then,
		-- with no date: dropped), so the best before today does not fall back once they leave it
		if untopped then
			ns.Stats.SeedTop(s)
			s.sheet.counts.maxMoves = nil
		end
		-- its pose and cardio steps start where their forms' familiarity stands, which it paid already
		ns.Stats.SeedFamSteps(s.sheet)
		-- one from before 2026-10-15 kept only the XP of the last game day it counted (xpDay, counted), where each day
		-- keeps its own now (xp): it goes to that day, and the rows the ring still holds add to it; the days before it
		-- kept none, so the day map is whole from that day on (Stats.HoldFrom)
		local sh = s.sheet
		local counted = type(sh.xpDay) == "number" and sh.days[sh.xpDay]
		if counted and (tonumber(sh.counted) or 0) > 0 then counted.xp = (counted.xp or 0) + sh.counted end
		if type(sh.xpDay) == "number" then ns.Stats.HoldFrom(sh, sh.xpDay) end
		sh.xpDay, sh.counted = nil, nil
		-- one from slice 26 kept the app pump as its offered row's time (its Log entry's key), where the sheet keeps the
		-- app pump itself since slice 27 (2026-10-06): it reads as no app pump
		if type(s.sheet.appPump) ~= "table" then s.sheet.appPump = nil end
		-- one from before 2026-10-03 kept each rung-up's date on its ladder (`ups`), where its sheet kept true, and no
		-- step per muscle: the dates move to the sheet, and a muscle's step comes from its best move's rung (no start
		-- rung is known there, so a climb back to it pays once more)
		for l, lad in pairs(s.ladders) do
			for rung, ts in pairs(type(lad) == "table" and type(lad.ups) == "table" and lad.ups or {}) do
				local key = ns.RungKey(l, rung)
				if s.sheet.rungs[key] == true then s.sheet.rungs[key] = ts end
			end
			if type(lad) == "table" then lad.ups = nil end
		end
		mergeBaseline(s)
		-- one folded before a move's form changed (its `through` before the move's goalSince: ns.BeforeForm) holds that
		-- move's recent form and days at its top from the old form, which go; its record and stars stay (2026-10-19)
		local through = tonumber(baseline.through) or 0
		for id, m in pairs(s.moves) do
			if type(m) == "table" and ns.BeforeForm(id, through) then
				m.fresh, m.freshLoad, m.topDays, m.topDayTs = nil, nil, nil, nil
			end
		end
		-- and so does the climb a ladder counted toward from its current rung where such a move stands for that rung
		-- (its chain: the anchor and its twins, whose sets count the days at the top, the plateau and the step down):
		-- its counts start again as a climb starts them (2026-10-19, the double-check's fix review: core step 7's days
		-- at the top counted from the feet-wide taps, so one feet-together day completed the climb)
		for l, lad in pairs(s.ladders) do
			if type(lad) == "table" then
				for _, id in ipairs(ns.RungChain(l, ns.Rung(s, l))) do
					if ns.BeforeForm(id, through) then
						resetCounts(lad)
						break
					end
				end
			end
		end
		-- and the hold ledger's entry of a ladder whose step holds such a move the ledger reads (ns.LedgerTop; holdLedger
		-- never reads a set from before its form changed): it starts again at the step's prior, as a rung change starts it
		for l, e in pairs(s.ledger.holds) do
			for id in pairs(type(e) == "table" and ns.Library or {}) do
				if ns.BeforeForm(id, through) and ns.LedgerTop(s, l, id) then
					ns.HoldClimb(e, ns.HoldPrior(l, ns.Rung(s, l)))
					break
				end
			end
		end
		for _, mu in pairs(s.sheet.muscles) do
			local m = mu.step == nil and mu.best and ns.Library[mu.best]
			if m then mu.step = ns.StepOn(ns.ScaleLadder(m), mu.rung or 0) end
		end
	end
	return s
end

local function dayOf(s, ts)
	local d = ns.day(ts)
	local t = s.days[d]
	if not t then
		t = { pumps = 0, moves = 0, records = 0, banked = 0 }
		s.days[d] = t
	end
	return t
end

-- `field`: a game count (pumps, moves, records, banked) or the app's own beside it (appPumps, appMoves, appRecords;
-- slice 26), which a day folded before them lacks.
local function bump(s, ts, field)
	local t = dayOf(s, ts)
	t[field] = (t[field] or 0) + 1
end

local function count(t, key)
	t[key] = (t[key] or 0) + 1
end

-- Whether a set of move `moveId` at `ts` came before the move's form changed (Library.lua `goalSince`: the wall
-- angels' wall test since slice 42's review, plank shoulder taps' feet together since #138; 2026-10-19, the program's
-- double-check, which made it every changed move's): such a set keeps its record and stars, and counts toward no
-- recent form (recordCount, ns.Form), no day at its top (topDay, Progress.lua strengthDays), no climb's day at the
-- top, plateau or step down (anchorAttempt) and no check (a pose's: markDays; tools/lint.lua lets no need or check
-- name another such move by its count).
function ns.BeforeForm(moveId, ts)
	local m = ns.Library[moveId]
	return m ~= nil and m.goalSince ~= nil and ts < m.goalSince
end

-- `fresh`: the set came goal_fresh_h or more after the ladder's last (ns.FreshAt, asked before the row moved the
-- clocks) and is no warm move's easy set (applyRow, slice 48's double-check); a placement (a first record, a rung's or
-- a break's max set) starts the player's form again from its count.
-- A count above the player's form (ns.Form; with a weight on a move with a weighted record its strength, ns.LoadForm)
-- joins the form on a tired set too, since the player's fresh form is at least that (David, 2026-10-10: "when the goal
-- is a new record and the old one is 12 then it should not say 12+ but 13+"; until then a record or a tie set on a
-- repeat stayed out of it, and the next fresh goal asked the record again). A set from before the move's form changed
-- (ns.BeforeForm) joins no form, and the first set since starts it (no form to be above: ns.Form). `load`: the set's
-- weight (its load row's, tenths of a kg), nil for none.
local function recordCount(s, move, value, ts, fresh, placement, load)
	local m = s.moves[move]
	if not m then
		m = {}
		s.moves[move] = m
	end
	if not m.first then m.first, m.firstTs = value, ts end
	local old = ns.BeforeForm(move, ts)
	local above
	if load and m.topLoad then
		above = ns.LoadScore(load, value) > ns.LoadScore(ns.LoadForm(s, move))
	else
		local form = m.best ~= nil and ns.Form(s, move)
		above = m.best ~= nil and (not form or value > form)
	end
	if not m.best or value > m.best then m.prev, m.best, m.bestTs = m.best, value, ts end
	m.last, m.lastTs, m.lastLoad = value, ts, load
	if load and (not m.topLoad or ns.LoadScore(load, value) > ns.LoadScore(m.topLoad, m.topReps)) then
		m.topLoad, m.topReps, m.topTs = load, value, ts
	end
	if old then return end
	if placement then
		m.fresh, m.freshLoad = { value }, { load or 0 }
	elseif fresh or above then
		m.fresh = m.fresh or {}
		-- a baseline from before the weights kept none for its fresh counts: they had none
		local loads = m.freshLoad or {}
		while #loads < #m.fresh do table.insert(loads, 1, 0) end
		m.fresh[#m.fresh + 1], loads[#loads + 1] = value, load or 0
		if #m.fresh > ns.C.goal_form_n then
			table.remove(m.fresh, 1)
			table.remove(loads, 1)
		end
		m.freshLoad = loads
	end
end

local function unwell(s, ts)
	if not s.unwellTs or ts > s.unwellTs then s.unwellTs = ts end
end

-- Whether a row locks its game day for feeling unwell, from any source (the lock, ns.LockedToday; the streak's locked
-- days, Stats.lua lockDay): a move row but a paid one with the unwell reason (whatever its count; on rows from before
-- 2026-10-04, when the stopped-because row went, and on an app's running timed move stopped for it), or a pump's or a
-- level-up's banked row with it (an app pump's felt-unwell stop, the helper's; "I feel unwell" with a pause until
-- 04:00, Lc.PausePump; an old Bank it for feeling unwell). The helper's Row::locks and the phones' UnwellLock read the
-- same.
function ns.UnwellRow(r)
	local reason = ns.Reason(r.flags)
	if MOVE_KIND[r.kind] then return r.outcome ~= O.paid and reason == E.stopped.unwell end
	return (r.kind == K.pump or r.kind == K.ding) and r.outcome == O.banked and reason == E.bank.unwell
end

local function ladderState(s, l)
	local lad = s.ladders[l]
	if not lad then
		lad = { hiDays = 0, anchorDays = 0, perfectDays = 0, plateau = 0, underLo = 0, days = {} }
		s.ladders[l] = lad
	end
	return lad
end

-- A pattern's load decays with the half-life of the moves that built it: tendon moves 48 h, runs 12 h, strength,
-- ballistic, all-out work and the other cardio-pattern moves (jacks, rope, bike, stairs) 36 h.
local function halfLife(m, allout)
	local hl = ns.C.half_life_h
	if allout then return hl.allout end
	for _, flag in ipairs(m.flags) do
		if flag == "tendon" then return hl.tendon end
	end
	if m.pattern == "run" then return hl.run end
	return m.pattern == "ballistic" and hl.ballistic or hl.strength
end

local function patternState(s, pattern)
	local p = s.patterns[pattern]
	if not p then
		p = { days = {}, acc = {} }
		s.patterns[pattern] = p
	end
	return p
end

local function addLoad(p, hl, class, ts)
	local a = p.acc[hl]
	if a then
		a.v = a.v * 0.5 ^ ((ts - a.ts) / (hl * 3600)) + class
		a.ts = ts
	else
		p.acc[hl] = { v = class, ts = ts }
	end
end

local function currentPump(s)
	local n = #s.recent
	if n == 0 then
		s.recent[1] = { moves = {}, patterns = {} }
		n = 1
	end
	return s.recent[n]
end

-- An anchor attempt at the ladder's current rung moves the counters the rung rules read, and marks the row for
-- ns.Progression: the attempt, a placement (the ladder's first record, or the first set after a break), a Trial
-- placement (the ladder's first record), the rung's hi reached, and the stars the record newly passed. The unlock
-- counts game days with the window's top reached, and the plateau one attempt a game day, so one long evening of sets
-- never unlocks a rung (slice 20), and both only on a move of the rung's own level (2026-10-09: a stand-in from below
-- keeps the ladder going, its records, stars and the rung down, but never climbs the rung it stands for); only a fresh
-- set (ns.FreshAt) counts toward a rung down: a tired fifth set of the day says nothing about the rung. Only the
-- anchor's own set moves its clock (`anchorTs`); a stand-in's, a twin's or a move's from below, moves `standTs` (slice
-- 52, 2026-10-23: any stand-in's set moved anchorTs, so the core's long-lever plank stopped coming once the desk plank
-- stood in for it; Engine.lua ladderDue reads which).
local function anchorAttempt(s, r, lad, value, typed, fresh)
	r.anchorAttempt = true
	if r.move == ns.Anchor(r.ladder, r.rung) then lad.anchorTs = r.ts else lad.standTs = r.ts end
	lad.opened = nil
	local rec = s.moves[r.move]
	-- only a real placement climbs at once from the top (2026-10-10, David: "You should be strong enough to easily
	-- handle the next rung"): any other first set (a new rung's anchor, a move whose record the player cleared on a
	-- ladder that stays placed) is a max set whose top counts the first of the climb's days
	r.trialPlacement = lad.placedTs == nil
	r.firstPlacement = r.trialPlacement or lad.placing == true
	if not typed then return end -- done without a count: attempted, nothing measured, a break placement still owed
	lad.placing = nil
	r.value = value
	local lo, hi = ns.Window(r.ladder, r.rung, r.move)
	if not lo then return end
	local out = r.outcome
	-- a loaded move's set climbs only at the weight its climb asks (ns.LoadClimbs, 2026-10-12): a lighter one keeps its
	-- record and stars, and counts no day at the top, no plateau day and no placement at the top
	local climbs = ns.LoadClimbs(r.move, r.load)
	r.hitHi = value >= hi and climbs
	-- the unlock and the plateau count days, each a new session by the night rule (ns.NewSession) after the set that
	-- counted last, so one long evening across 04:00 never gives two
	-- a set from before the move's form changed (ns.BeforeForm) counts no day at the top, no plateau day and nothing
	-- toward a step down: they measure the form the move asks now
	local old = ns.BeforeForm(r.move, r.ts)
	if old then r.hitHi = false end
	local own = ns.Library[r.move].rung == r.rung
	if own and r.hitHi and (lad.hiTs == nil or ns.NewSession(lad.hiTs, r.ts)) then
		-- Overpower (slice 52): a fresh set at or past overpower_pct % of the top on a ladder of the Stamina Ledger
		-- counts two days at once (the row's `overpower`, ns.Progression), never where the climb takes the tendon days
		-- (ns.UnlockDays: an orange or gold rung above whose anchor is tendon-heavy)
		local over = fresh and ns.HOLD_LEDGER[r.ladder] and value * 100 >= ns.C.overpower_pct * hi
			and ns.UnlockDays(r.ladder, r.rung) ~= ns.C.unlock_days_tendon
		lad.hiTs, lad.hiDays, r.topDay, r.overpower = r.ts, lad.hiDays + (over and 2 or 1), true, over or nil
	end
	if own and out == O.beat and not old then
		lad.plateau = 0
	elseif own and climbs and not old and value >= ns.C.plateau_fraction * hi
		and (lad.plateauTs == nil or ns.NewSession(lad.plateauTs, r.ts)) then
		lad.plateauTs, lad.plateau, r.plateauDay = r.ts, lad.plateau + 1, true
	end
	if fresh and not old then
		if out ~= O.censored and value < lo then lad.underLo = lad.underLo + 1 else lad.underLo = 0 end
		lad.halfLo = out ~= O.censored and value < lo / 2 and (lad.halfLo or 0) + 1 or nil -- in a row
	end
	if value > 0 and not lad.placedTs then
		lad.placedTs = r.ts
		r.placed = true
	end
	local before = rec and rec.best or 0
	if value > before then
		local was, now = ns.Stars(r.ladder, r.rung, r.move, before), ns.Stars(r.ladder, r.rung, r.move, value)
		if now > was and before > 0 then r.stars, r.starsFrom = now, was end
	end
end

-- An orange or gold rung earned (2026-10-09; ns.EarnFloor): the climb only opened rung `k`, the set this row carries
-- earns it, once, while the ladder stands on it or above (a rung down before it keeps nothing). The row says so for
-- ns.Progression (`earns`) and the sheet, which marks the rung earned (Stats.lua markEarned); `rekindles` where the
-- sheet holds an earning of it from before the player started the ladder over, which counts again, a smaller win, as a
-- climb back does.
local function earnMark(s, r, k)
	if k > ns.Rung(s, r.ladder) or ns.EarnCounts(s, r.ladder, k) then return end
	r.earns, r.rekindles = k, ns.RungEarned(s, r.ladder, k) ~= nil or nil
end

-- A goal pose of a pose family's step at the family's rung (ns.Marks; slice 21, any goal pose of the step since
-- slice 47, 2026-10-19, its main pose alone until then): its attempt (anchorTs: the family's turn, and its mark came
-- today, ns.MarkedToday, but for a hold still in the night after a day counted on an earlier game day), and the pose
-- held its whole time and answered close enough to its goal pose (ns.MarkNeeds: Good or Perfect, Perfect alone below
-- a risky step; 2026-10-09, David: "Reached the mark was very unclear. Think through a better ask") counts the
-- family's day toward the unlock, as an anchor's set at the window's top does, each day a new session by the night
-- rule after the one that counted last (hiDays, the row's `topDay`; whichever pose was asked), on its main pose
-- (ns.MainPose) a day of the main pose's floor too (anchorDays, `mainDay`) and answered Perfect a Perfect day
-- (perfectDays, `perfectDay`; ns.PoseNeeds reads the three); Okay and Not yet are practice, and so is a hold with no
-- answer (the slot asked none, or the pump ended at the ask), and a stop before the time counts nothing. Proof counts
-- only from an answer the player gave: a row from before the ask (no mark bit) is practice on every pose, its main
-- pose's too, since nothing asked how close it came (David, 2026-10-31: "I had stretches that now are marked done in
-- the desktop app even so I never said I did them perfect etc"; on the main pose it read as held whole until then, a
-- Perfect day until 2026-10-26 and Good since): its hold is the family's attempt and counts no day. The main pose
-- answered Perfect earns an orange rung the climb opened (earnMark).
local function markAttempt(s, r, lad)
	r.anchorAttempt = true
	lad.anchorTs, lad.opened = r.ts, nil
	if r.outcome ~= O.done then return end
	local main = ns.MainPose(r.ladder, r.rung, r.move)
	local answer = ns.HasFlag(r.flags, "mark") and (r.reps or 0) or 0
	local perfect = answer >= E.mark.perfect
	if answer >= ns.MarkNeeds(r.ladder, r.rung) and (lad.hiTs == nil or ns.NewSession(lad.hiTs, r.ts)) then
		lad.hiTs, lad.hiDays, r.topDay = r.ts, lad.hiDays + 1, true
		if main then lad.anchorDays, r.mainDay = (lad.anchorDays or 0) + 1, true end
		if perfect then lad.perfectDays, r.perfectDay = (lad.perfectDays or 0) + 1, true end
	end
	if main and perfect and ns.TopTier(r.ladder, r.rung) then earnMark(s, r, r.rung) end
end

-- Every pose's mark days (2026-10-12; ns.MarkDays): a hold of a pose family's pose held its whole time and answered
-- Good or better counts its game day toward its Good days, Perfect toward its Perfect days too, each a new session by
-- the night rule after the one that counted last, whatever pose it is and whatever rung the family stands on (a pose's
-- days on a step's path, ns.TopDays, a pose step done and a goal pose a check names, Progress.lua stepDone, read them);
-- a row from before the ask (no mark bit) counts no day, practice as markAttempt reads it (2026-10-31; a Good day until
-- then, toward a step's path too, which no answer proved). An asked answer keeps the closest one (`best`, slice 42),
-- the first time each of Good or better and Perfect came (`goodFirst`, `perfectFirst`, 2026-10-17: when a stretch check
-- was passed, ns.CheckPassedAt, the cards' "Passed {date}") and the moment of the last one (`askTs`, 2026-10-19, slice
-- 47: the pose a family's turn asks next, ns.DuePose); every hold its whole time, asked or not, the moment of the last
-- (`heldTs`, 2026-10-20, slice 47's fix round: the pose a family's turn brings next on its top step, where no pose is
-- asked once its own goal pose is reached). A pose whose goal pose changed (ns.BeforeForm: the wall angels' wall test)
-- counts only rows from then on, every one of these. A catch-up's pose is practice (performed), as there.
local function markDays(s, r)
	if r.outcome ~= O.done or ns.BeforeForm(r.move, r.ts) then return end
	local answer = ns.HasFlag(r.flags, "mark") and (r.reps or 0) or 0
	local d = s.marks[r.move] or {}
	d.heldTs = r.ts
	if answer > 0 then
		d.best, d.askTs = math.max(d.best or 0, answer), r.ts
		if answer >= E.mark.good and not d.goodFirst then d.goodFirst = r.ts end
		if answer >= E.mark.perfect and not d.perfectFirst then d.perfectFirst = r.ts end
	end
	for _, k in ipairs({ "good", "perfect" }) do
		if answer >= E.mark[k] then
			local ts = k .. "Ts"
			if d[ts] == nil or ns.NewSession(d[ts], r.ts) then d[k], d[ts] = (d[k] or 0) + 1, r.ts end
		end
	end
	s.marks[r.move] = d
end

-- An all-out row: an all-out window's reps 1, or a counted set (its reps the count, 2026-10-05).
function ns.AllOutRow(r)
	return r.kind == K.cardio and (r.reps == 1 or ns.HasFlag(r.flags, "counted"))
end

-- The steady ledger's row (slice 52; Progress.lua's "The steady ledger"): a steady row of the cardio ladder ages the
-- ledger to its game day (ns.SteadyAged), then moves `a` by g = max(3, ⌊d/3⌋) by its outcome and answer (ledger_moves:
-- Yes, easily, Yes, just, I eased off and stopped, never a stop for feeling unwell, ns.UnwellRow, the felt-unwell
-- lock's row and no fitness signal, 2026-10-31; finished and not asked +max(1, ⌊g/4⌋) once a game day, `bumpDay`; a Yes
-- or a finish on a form already at the top M moves nothing up, as `make sim` showed the estimate running away there and
-- on a long evening's unasked finishes otherwise); a Yes floors `a` at the seconds it ran on its form's rung (Yes,
-- easily ledger_easy_floor more), which binds only after Keep going ran past the plan; an answer (asked, or Keep
-- going's) lowers the doubt by a sixth, at least 1, to its floor, and counts toward the ask's room (askDay, asks). A
-- Yes on a move dosed at the player's step while the step's own M (ns.SteadyRung) stood at the top before it, on any
-- form of the step's pool (so a setup that leaves no form of the step's own rung still climbs), that ran the full
-- move's ns.FULL_MOVE_S, times the row's span (ns.RowSpan, over which the ledger reads a row's seconds), by its dose or
-- Keep going's past it (a card's or an app pump's Start at the top M runs 30 s;
-- the double-check of slices 52-55, 2026-10-24) is a full move, once a session by the night rule, counted on the cardio
-- state beside the clean days (`full`, fullTs; the row's `fullMove`, for ns.Progression and the Result's line); a stop
-- leaves them as they are. `mHigh`, the best M in steps of 50, takes one step where a move dosed past it ran to its end
-- and was not eased off, at most one every ledger_m_days game days (`mDay`), the row naming it (`mSteps`, the cardio
-- stat's, Stats.lua): an M played, never the estimate after the row, which a new ledger's doubt carries to the top in
-- two days while the player keeps far less (`make sim`: 30 Agility by day 3, where P paid it over ten weeks); and the
-- ring each play day's last `a`. The doubt grows only with the game days between play days, so the fold reads the rows
-- in time order (ns.Fold, the merge).
local function steadyRow(s, r, m)
	local C, P = ns.C, E.pace
	local L = s.ledger.steady
	if not L then
		L = ns.SteadyLedger(ns.Rung(s, CARDIO))
		-- after a baseline from before slice 52, whose sheet holds the Agility its P steps paid (`pSteps`, one step of
		-- Agility each, as each new 50 of the best M pays one now), the best M starts past them, so no step pays twice
		local paid = s.cardio.pSteps
		if type(paid) == "number" and paid > 0 then L.mHigh = math.min(C.ledger_m[2], L.mHigh + 50 * paid) end
		s.ledger.steady, s.cardio.pSteps = L, nil
	end
	local day = ns.LedgerDay(r.ts)
	L.a, L.d = ns.SteadyAged(L, day)
	L.day = day
	local g, moves = math.max(3, math.floor(L.d / 3)), C.ledger_moves
	local step = ns.Rung(s, CARDIO)
	local own = ns.SteadyRung(step)
	local before = ns.SteadyMOf(L.a, L.d, own) -- the step's own M before the move
	local answer = ns.Answer(r.flags)
	local yes = r.outcome ~= O.stopped and ns.PaceYes(answer)
	-- at the top M the dose can go no longer, so a Yes there tests nothing past it: it moves `a` by its floors alone
	-- (below), or the estimate would climb on answers no move can check and start the next step long
	local dosed = ns.SteadyMOf(L.a, L.d, m.rung) -- the M the move was dosed at, on its form's rung
	local top = dosed == C.ledger_m[2]
	if r.outcome == O.stopped then
		if not ns.UnwellRow(r) then L.a = L.a + moves[4] * g end
	elseif answer == P.easily then
		if not top then L.a = L.a + moves[1] * g end
	elseif answer == P.just then
		if not top then L.a = L.a + moves[2] * g end
	elseif answer == P.eased then
		L.a = L.a + moves[3] * g
	elseif not top and L.bumpDay ~= day then -- finished, not asked: once a game day, so volume alone never inflates it
		L.a, L.bumpDay = L.a + math.max(1, math.floor(g / 4)), day
	end
	local span = ns.RowSpan(r, m)
	if yes then
		local kept = math.floor(r.seconds / span)
		local floor = ns.LN[math.max(1, math.min(#ns.LN, kept))] + C.ledger_rung_ln * (m.rung - 1)
		if answer == P.easily then floor = floor + C.ledger_easy_floor end
		L.a = math.max(L.a, floor)
	end
	if r.outcome ~= O.stopped and answer > 0 then
		L.asks = L.askDay == day and L.asks + 1 or 1
		L.askDay = day
		L.d = math.max(C.ledger_doubt[2], L.d - math.max(1, math.floor(L.d / 6)))
	end
	local c = s.cardio
	if yes and before == C.ledger_m[2] and r.rung == step and m.rung <= own and r.seconds >= ns.FULL_MOVE_S * span
		and (c.fullTs == nil or ns.NewSession(c.fullTs, r.ts)) then
		c.full, c.fullTs, r.fullMove = c.full + 1, r.ts, true
	end
	if r.outcome ~= O.stopped and answer ~= P.eased and math.floor(dosed / 50) * 50 > L.mHigh
		and (L.mDay == nil or day - L.mDay >= C.ledger_m_days) then
		r.mSteps, L.mHigh, L.mDay = 1, L.mHigh + 50, day
	end
	local ring, last = L.ring, L.ring[#L.ring]
	if last and last[1] == day then
		last[2] = L.a
	else
		ring[#ring + 1] = { day, L.a }
		if #ring > C.ledger_plays then table.remove(ring, 1) end
	end
end

-- A Long Haul's row (slice 55): a steady cardio row whose reps hold its planned seconds (an all-out window's are 1, an
-- old owed closing move's 2, a steady move's 0), never counted.
function ns.HaulRow(r)
	return r.kind == K.cardio and r.reps > 2 and not ns.HasFlag(r.flags, "counted")
end

-- A Long Haul (slice 55, ns.HaulRow), which nothing else of the cardio ladder reads (no clean day, no stop, no steady
-- ledger move or full move): a Yes to its pace ask (ns.PaceYes) on a haul that ran its time at the length open
-- (c.haul, its place in long_haul_s) counts a day (c.haulDays), each a new session by the night rule after the last
-- (c.haulTs); long_haul_days of them open the next length, whose days start again. The row says what it counted
-- (haulDay, haulUp) for the Result's line.
local function haulMove(s, r)
	local c = s.cardio
	local lengths = ns.C.long_haul_s
	if r.outcome ~= O.done or not ns.PaceYes(ns.Answer(r.flags)) or r.reps < lengths[c.haul] then return end
	if c.haulTs and not ns.NewSession(c.haulTs, r.ts) then return end
	c.haulTs, c.haulDays, r.haulDay = r.ts, c.haulDays + 1, true
	if c.haulDays >= ns.C.long_haul_days and c.haul < #lengths then
		c.haul, c.haulDays, r.haulUp = c.haul + 1, 0, true
	end
end

-- The cardio ladder (2026-10-16, slice 42): a counted rung's clean days, game days with a window of the rung's own
-- forms that ran its 30 s and was saved "Yes, to the end" (a counted row, any outcome but stopped, its answer 0: never
-- Form slipped nor, since slice 53, Not all out; never one written before the answer, counted_ask_ts, which nothing
-- asked and so is practice: 2026-10-31, it read clean until then), each a new session by the night rule, and each
-- form's clean days among them (a step's path); a steady row (not counted, `reps` 0: never an all-out window, whose
-- reps are 1, nor a Long Haul, whose reps hold its planned seconds, slice 55) the steady ledger's (steadyRow, slice 52:
-- the full moves a timed rung climbs by, in its ready days' place). A fresh clean window of an orange or gold rung's
-- own form earns the rung its climb opened (earnMark; its count at its window's floor until then). Each form's first
-- clean window at any rung counts toward mastering its step (cleanEver). The row says what it counted (cleanDay,
-- fullMove, mSteps) for ns.Progression and the sheet. A Long Haul's row counts its own days alone (haulMove), and any
-- cardio row after a layoff fades its length (ns.HaulPlace).
-- Slice 53 (2026-10-23): a counted row of a form one rung above its row's rung is the proving window of the step above
-- (no row field: Engine.lua provingMove; stopped, such a window carries no counted bit, ns.AllOutRow reads it;
-- `proving`, and `proved` where it ran its 30 s saved Yes, to the end), which counts no clean day and no stop day,
-- never one stopped for feeling unwell (ns.UnwellRow: the felt-unwell lock's row, no try at the step, which read as a
-- proving that did not pass until 2026-10-31); every window of a countable form at the rung dates the proving's night
-- rule (countTs), and a proving that did not pass dates the card's "No step up yet." (proveTs). Footing: a stop at the
-- rung counts a stop day (stops, stopTs; ns.Progression steps down at cardio_rung_down), each a new session by the
-- night rule, never on the game day of a climb (climbTs) and never a stop for feeling unwell (ns.UnwellRow: the
-- felt-unwell lock's row, no fitness signal; 2026-10-31), and a finished move of the step starts them again; a stop
-- wipes no clean day nor full move. The return: placement_idle_days without a cardio move (lastTs) put a step above the
-- first back to placing (ns.Placing), its counts (the full moves among them) started again as a strength ladder's
-- break placement starts them (cardioBreak, at `ts`, breakTs: Pro's Path dates a counted step's clean days from it,
-- ns.SteadyForecast), and the step's first measure (cardioMeasure) holds it (`holds` true) or takes it a step down
-- (`holds` false: the rung row ns.Progression writes).
local function cardioBreak(s, ts)
	local c = s.cardio
	c.placing, c.stops, c.stopTs, c.breakTs = true, 0, nil, ts
	c.clean, c.cleanTs, c.cleanForms, c.cleanFormTs, c.full, c.fullTs = 0, nil, {}, {}, 0, nil
end

-- A cardio row's measure of a step back from a break (cardioMove), the step's own (slice 53): a countable form's
-- window by its answer (Yes, to the end holds; any other answer or a stop does not); from closing_rung, whose every
-- pump closes on an all-out window, the window alone, another form's by its time (run in full it holds the step,
-- stopped it does not), so a steady move of a lower form measures nothing there; below it, whose pumps hold no closing
-- window, a steady move asked its pace by that answer (any Yes holds, I eased off does not) or stopped (it does not).
-- True where it holds the step, false where it takes it down, nil where the row measures nothing (a steady move from
-- closing_rung, an all-out window below it that counts nothing, a steady move finished without the ask, a stop for
-- feeling unwell, ns.UnwellRow, which is the felt-unwell lock's and no fitness signal: 2026-10-31).
local function cardioMeasure(r, window, clean, pace)
	if r.outcome == O.stopped and ns.UnwellRow(r) then return nil end
	if window then return clean end
	if r.rung >= ns.C.closing_rung then
		if not ns.AllOutRow(r) then return nil end
		return r.outcome ~= O.stopped
	end
	if r.reps ~= 0 then return nil end -- an all-out window that counts nothing
	if pace > 0 then return pace ~= E.pace.eased end
	if r.outcome == O.stopped then return false end
	return nil
end

local function cardioMove(s, r, fresh)
	local c = s.cardio
	if c.lastTs and r.ts - c.lastTs >= ns.C.placement_idle_days * DAY_S and ns.Rung(s, CARDIO) > 1 then
		cardioBreak(s, r.ts)
	end
	local place, faded = ns.HaulPlace(c, r.ts)
	if faded then c.haul, c.haulDays, c.haulTs = place, 0, nil end
	c.lastTs = r.ts
	if ns.HaulRow(r) then return haulMove(s, r) end
	local m = ns.Library[r.move]
	local counted = m ~= nil and m.count ~= nil and ns.HasFlag(r.flags, "counted")
	local clean = counted and r.outcome ~= O.stopped and ns.Answer(r.flags) == 0 and r.ts >= ns.C.counted_ask_ts
	if clean and not c.cleanEver[r.move] then c.cleanEver[r.move] = r.ts end
	local pace = not counted and r.reps == 0 and ns.Answer(r.flags) or 0
	if m and not counted and r.reps == 0 then steadyRow(s, r, m) end
	if r.rung ~= (c.rung or ns.Rung(s, CARDIO)) then return end
	-- a countable form's all-out window (counted, or stopped before its count: reps 1, no counted bit)
	local window = m ~= nil and m.count ~= nil and ns.AllOutRow(r)
	local proving = window and m.rung == r.rung + 1 and not ns.UnwellRow(r)
	if window then c.countTs = r.ts end
	if proving then
		r.proving, r.proved = true, clean or nil
		if not clean then c.proveTs = r.ts end
	elseif c.placing then
		r.holds = cardioMeasure(r, window, clean, pace)
		if r.holds ~= nil then c.placing = nil end
	end
	if r.outcome ~= O.stopped then -- finished: done, or a counted set's typed outcome
		if clean and m.rung == c.rung then
			if c.cleanTs == nil or ns.NewSession(c.cleanTs, r.ts) then
				c.cleanTs, c.clean, r.cleanDay = r.ts, c.clean + 1, true
			end
			local last = c.cleanFormTs[r.move]
			if last == nil or ns.NewSession(last, r.ts) then
				c.cleanFormTs[r.move], c.cleanForms[r.move] = r.ts, (c.cleanForms[r.move] or 0) + 1
			end
			if fresh and ns.TopTier(CARDIO, m.rung) then earnMark(s, r, m.rung) end
		end
		c.stops, c.stopTs = 0, nil
	elseif not proving and not ns.UnwellRow(r) and not (c.climbTs and ns.day(c.climbTs) == ns.day(r.ts))
		and (c.stopTs == nil or ns.NewSession(c.stopTs, r.ts)) then
		c.stops, c.stopTs = c.stops + 1, r.ts
	end
end

-- A match pump's warm move's easy set (slice 48's double-check): a rep or hold row whose answer bits carry E.set.warm
-- (ns.ResultRow), its goal the repeat share of the player's form (ns.Goal's `warm`); the fold counts it as no fresh set
-- (applyRow) and as no set of its ladder (performed).
local function warmSet(r)
	return (r.kind == K.rep or r.kind == K.hold) and ns.Answer(r.flags) == E.set.warm
end

-- A performed move (any outcome but owed, banked or paid) moves the clocks the engine reads; `fresh` says whether the
-- set was fresh before this row moved them (ns.FreshAt), `game` whether the row is no app row's (the feedback numbers
-- are the game's). A warm set (warmSet) moves no clock of its strength ladder and counts no set of its game day there
-- (2026-11-01: an easy split squat after a long sit made the squat ladder's record set of the next 16 hours a repeat,
-- ns.FreshAt, and took its major move's turn, Engine.lua mostDue); its move's own record and last set stay its own.
local function performed(s, r, m, fresh, game)
	if s.lastDoneTs and r.ts - s.lastDoneTs >= ns.C.placement_idle_days * DAY_S then
		s.breakTs = r.ts
		for l, lad in pairs(s.ladders) do
			-- the placement after a break measures the player again: the counts toward a rung start from it
			if STRENGTH[l] and lad.placedTs then
				lad.placing = true
				resetCounts(lad)
			end
		end
	end
	if not s.lastDoneTs or r.ts >= s.lastDoneTs then s.lastDoneTs, s.lastMove = r.ts, r.move end
	s.createdTs = s.createdTs or r.ts
	s.seen[r.move] = true
	local day = ns.day(r.ts)
	local allout = ns.AllOutRow(r)
	local p = patternState(s, m.pattern)
	p.lastTs = r.ts
	count(p.days, day)
	-- the day plan: the block's moves and the move's own sets that game day
	local today = dayOf(s, r.ts)
	today.blocks, today.done = today.blocks or {}, today.done or {}
	count(today.blocks, ns.Block(r.move))
	count(today.done, r.move)
	local clockless = r.kind == K.cardio or r.kind == K.flow and not POSE[m.ladder]
	if clockless and r.ts > (s.formTs[r.move] or 0) then s.formTs[r.move] = r.ts end
	-- the feedback numbers (slice 24), the game's: the version Swap's "Easier version" took, and a timed move stopped
	-- early
	if game and ns.HasFlag(r.flags, "easier") then
		today.easier = today.easier or {}
		count(today.easier, r.move)
	end
	if game and r.outcome == O.stopped then
		today.stopped = today.stopped or {}
		count(today.stopped, r.move)
	end
	if r.kind ~= K.flow then addLoad(p, halfLife(m, allout), ns.C.load_class[allout and 3 or m.class], r.ts) end
	local pump = currentPump(s)
	pump.moves[r.move], pump.patterns[m.pattern] = true, true
	local record = r.kind == K.rep or r.kind == K.hold
	if record and m.region then
		local reg = s.regions[m.region]
		if not reg then
			reg = { days = {} }
			s.regions[m.region] = reg
		end
		count(reg.days, day)
	end
	if record and STRENGTH[r.ladder] then
		local lad = ladderState(s, r.ladder)
		if not warmSet(r) then
			lad.lastTs = r.ts -- the major move's turn (Engine.lua, mostDue)
			count(lad.days, day)
			if r.retiredLadder then
				lad.retiredDays = lad.retiredDays or {}
				count(lad.retiredDays, day)
			elseif not game then
				lad.appDays = lad.appDays or {}
				count(lad.appDays, day)
			end
		end
		local value = r.kind == K.hold and r.seconds or r.reps
		if r.rung == ns.Rung(s, r.ladder) and ns.StandsFor(r.ladder, r.rung, r.move) then
			local typed = TYPED[r.outcome] and (value > 0 or r.outcome ~= O.done)
			anchorAttempt(s, r, lad, value, typed, fresh)
		end
		-- a fresh set of an orange or gold rung's own move at its window's floor earns the rung its climb opened
		local floor = fresh and TYPED[r.outcome] and ns.EarnFloor(r.ladder, m.rung, r.move)
		if floor and value >= floor then earnMark(s, r, m.rung) end
	elseif r.kind == K.cardio and r.ladder == CARDIO then
		cardioMove(s, r, fresh)
	elseif r.kind == K.flow and POSE[r.ladder] then
		local lad = ladderState(s, r.ladder)
		lad.lastTs = r.ts -- the family's turn in its block (Engine.lua, planned)
		count(lad.days, day)
		-- a catch-up pays the pose as practice: a day of the mark comes from a pump's answer (its slot's `ask`)
		if ns.Source(r.flags) ~= E.source.catchup then
			markDays(s, r)
			if r.rung == ns.Rung(s, r.ladder) and ns.Marks(r.ladder, r.rung, r.move) then markAttempt(s, r, lad) end
		end
	end
end

-- A strength or hold move's day at its own window's top (2026-10-16, slice 42: a step's path, ns.TopDays): a set at its
-- top (ns.EasierTop) at a weight that climbs (ns.LoadClimbs), each game day a new session by the night rule after the
-- set that counted the last. `days`: the move's top days before the set, its record's from a baseline written before
-- them (ns.TopDaysFrom).
local function topDay(s, r, value, days)
	local m = s.moves[r.move]
	m.topDays = days
	local top = ns.EasierTop(r.move)
	if not (top and value >= top and ns.LoadClimbs(r.move, r.load)) then return end
	if m.topDayTs == nil or ns.NewSession(m.topDayTs, r.ts) then m.topDays, m.topDayTs = days + 1, r.ts end
end

-- Swap's easier version teaches the engine (David, 2026-10-05: "at the next quest it tried again to make me do pike
-- push ups"): a set of a strength move taken as the easier version marks it in its record (`easier`, the set's time,
-- and `easierUntil`, the count that clears it); while marked, no pump offers a move the player has never done whose
-- named regression leads to it (ns.SteppedDown), and the pumps offer the marked move itself (Engine.lua candidates)
-- until a set of it not taken as the easier version reaches that count at a weight that counts toward a climb
-- (ns.LoadClimbs: 20 one-arm rows with a 2-kg bag cleared the table row's hold until 2026-10-13), which clears the mark
-- and marks the row (`unmarked`: ns.Progression names a family the mark held back). Each easier set sets the count
-- again: the top of its window (ns.EasierTop), or, once its record (this set's count included; a move with a climb
-- weight its weighted record, and only at such a weight) has reached that top, one step past the record
-- (ns.RecordStep): a step down from a move whose easier version is at its top says the top was not enough for this
-- player (2026-10-13, David, after one-foot frog stands of 15, 20 and 25 s a side, each at or past its top and so
-- marking nothing until then, brought the frog stand back at the next pump: "I already clicked easier version last
-- time").
-- A mark from a baseline written before the count clears at the top.
local function easierMark(s, r, value)
	local top = ns.EasierTop(r.move)
	if not top then return end
	local m = s.moves[r.move]
	if ns.HasFlag(r.flags, "easier") then
		local held = m.best
		if ns.ClimbKg(r.move) then held = m.topLoad and ns.LoadClimbs(r.move, m.topLoad) and m.topReps or 0 end
		m.easier, m.easierUntil = r.ts, held < top and top or held + ns.RecordStep(r.move, held)
	elseif m.easier and value >= (m.easierUntil or top) and ns.LoadClimbs(r.move, r.load) then
		m.easier, m.easierUntil, r.unmarked = nil, nil, true
	end
end

-- Whether flow row `r` is a pose that ran its time and was answered "How close to the goal pose?" (its answer in reps,
-- 1-4, beside the mark bit): the one a correction may change (slice 44). Every flow row since the ask (2026-10-09)
-- carries the bit, with reps 0 where it asked nothing or the pump ended at the ask (a stretch's, a carry's too), and a
-- row from before it has no bit. ns.CorrectableRow reads it too.
function ns.PoseAnswered(r)
	return r.kind == K.flow and ns.HasFlag(r.flags, "mark") and r.outcome == O.done and (r.reps or 0) >= E.mark.notyet
end

-- Whether censored set row `r` (as the fold reads it) is a capped max set, judged on `s`, the state just before it
-- folds (slice 44's review, 2026-10-18): stored at its window's top (ns.Judge ends a max set there, "enough: next
-- rung") on a move that held no record yet or a ladder being placed again (ns.Placing), the only sets ns.Judge caps.
-- Every other censored set is a long one (ns.Classify's censor_s, which needs a record held before it and no placement
-- under way), its count as typed, whatever it is: a long set can stop at the top too, so the top alone never tells.
function ns.CappedSet(s, r)
	if r.outcome ~= O.censored then return false end
	local id = ns.Merged[r.move] or r.move
	local _, hi = ns.Window(r.ladder, r.rung, id)
	local stored = r.kind == K.hold and r.seconds or r.reps
	return hi ~= nil and stored == hi and (ns.Record(s, id) == nil or ns.Placing(s, r.ladder, r.ts))
end

-- The value the winning correction of move row `r` (as the fold reads it, ns.Translate) gives it, nil where none
-- names it or it may change nothing (slice 44: never a set stopped short, whose outcome is no typed one and not done:
-- a stopped counted window corrected would count a clean day; never a timed cardio row, a pose that did not run its
-- time or was not answered, ns.PoseAnswered; an owed, banked or paid row never reaches it), as ns.CorrectableRow lists
-- none of them: a rep set's or a counted set's count, a hold's seconds (half of them on a sided hold from before its
-- sides, ns.Sided), a pose's answer to "How close to the goal pose?". A capped max set (`capped`, ns.CappedSet on the
-- state before it) keeps its stored top for a count at or over it; a long censored set takes the count.
local function fixOf(s, r, capped)
	local c = s.corrections and s.corrections[setKey(r)]
	if not c or not MOVE_KIND[r.kind] or not (TYPED[r.outcome] or r.outcome == O.done) then return nil end
	if r.kind == K.flow then return ns.PoseAnswered(r) and c.reps or nil end
	if r.kind == K.cardio and not ns.HasFlag(r.flags, "counted") then return nil end
	local hold = r.kind == K.hold
	local value = hold and c.seconds or c.reps
	if r.halved then value = math.floor(value / 2) end
	local stored = hold and r.seconds or r.reps
	if capped and value >= stored then value = stored end
	return value
end

-- Set row `r` as stored (a decoded log row), read as the fold reads it (ns.Translate, a copy), and what `s`, a fold
-- that read it, kept of its correction (s.fixed; nil none).
local function asFolded(s, r)
	local t = {}
	for k, v in pairs(r) do t[k] = v end
	t = ns.Translate(t) or t
	return t, s.fixed and s.fixed[setKey(t)]
end

-- The value the fold gives set row `r` as stored in `s`, a fold that read it: its winning correction's as the fold
-- took it (correct: a capped max set's top for a count at or over it), else its own (a pose's answer, a hold's
-- seconds, any other move's reps).
function ns.SetValue(s, r)
	local t, fixed = asFolded(s, r)
	if fixed then return fixed.value end
	return t.kind == K.hold and t.seconds or t.reps
end

-- The outcome the fold gives set row `r` as stored in `s` (slice 44's review, 2026-10-18): its corrected one where a
-- correction changed it (a count corrected to 0 done, a capped max set under its top a placement or a beat), else its
-- own. A window words a set of 0 by it: done, "Done without a count"; a miss or a placement, "Could not do one".
function ns.SetOutcome(s, r)
	local t, fixed = asFolded(s, r)
	return fixed and fixed.outcome or t.outcome
end

-- A correction row names the row it corrects (its ts, questID, slot and move) and carries the corrected count; the fold
-- reads the corrected row with that count (fixOf) and classifies it again against the state before it, with the goal
-- that state gives the move at the row's time and the row's own weight (the row stores no goal; a warm set its easy
-- goal, warmSet), so a corrected count that reached it is a win; a counted set is judged with no goal, and one saved
-- Form slipped stays done whatever its count (ns.Judge). Since slice 44 (2026-10-18): a pose's correction takes its
-- answer and keeps its outcome (its mark days, ns.MarkDays, and a check read the answer); a capped max set
-- (ns.CappedSet, judged on the state before it) corrected under its top reads as a placement of that count (a beat
-- where it beats the record before it, as a placement does), and one at or over it stays as stored; a long censored set
-- (ns.Classify's censor_s) keeps its outcome and takes the count; a count corrected to 0 reads as typing 0 does, done
-- without a count (ns.Classify, ns.Judge: no attempt measured, so no record, no placement and no step down), whatever
-- the set's outcome was. The row is marked corrected (r.corrected, on the decoded row alone): the Saga's day lists no
-- record line for it (Stats.lua sagaMove), as contract/fixtures/history.json's `saga` rule 12 and the apps say.
local function correct(s, r)
	local capped = ns.CappedSet(s, r)
	local value = fixOf(s, r, capped)
	if not value then return end
	r.corrected = true
	if r.kind == K.flow then
		r.reps = value
		s.fixed[setKey(r)] = { value = value, outcome = r.outcome, move = r.move }
		return
	end
	local hold = r.kind == K.hold
	local out = r.outcome
	local counted = ns.HasFlag(r.flags, "counted")
	if value <= 0 then
		out = O.done
	else
		if capped and value < (hold and r.seconds or r.reps) then out = O.placement end
		if out ~= O.censored and not (counted and ns.Answer(r.flags) ~= 0) then
			local opts = counted and { counted = true } or { target = ns.Goal(s, r.move, r.ts, r.load, warmSet(r)) }
			local fresh = ns.Classify(s, r.move, value, nil, opts, r.load)
			out = (out == O.placement and fresh ~= O.beat) and O.placement or fresh
		end
	end
	if hold then r.seconds = value else r.reps = value end
	r.outcome = out
	s.fixed[setKey(r)] = { value = value, outcome = out, move = r.move }
end

-- A clear row (David, 2026-10-08: "reset specific excericises in the settings and delte the records"), the account's
-- from any device as focus is, read forward at its place in time: what came before it stays as it was (XP, stats,
-- wins, the Log, familiarity, the day plan, the queue, pain). One move's (E.clear.move, read as the move that took a
-- merged one) loses its record, so its next set is a first max set, and where it stands for its strength or pose
-- ladder's rung now the ladder's counts toward a rung start again. A strength ladder started over
-- (E.clear.ladder) loses every one of its moves' records and goes back to its start, keeping only its clocks (the major
-- move's turn, the fresh-set clock, its days): unplaced, so its next anchor set places it again (a Trial ladder's
-- opens the Trial again), the rungs reached before it count no more (ns.ReachCounts), and nor do its steps mastered
-- before it (2026-10-31: the fold reads in time order, so the climb back masters each again, as the earnings count
-- again after restartTs, ns.EarnCounts; they stayed Mastered until then). A newer addon's code changes
-- nothing; so does a pose family's or cardio's restart (a pose family keeps no record and no placement: its steps come
-- by marks over game days, so a start over would only take them away).
-- Whether a clear row takes the record of a move (as the fold reads a merged one): its own move's, or every move's of
-- the strength ladder it starts over; Events.lua Lc.Cleared reads it too (/qp undo's last typed count, an open pump's
-- moves), and Correct a count lists no set before it (the fold's `cleared`, ns.CorrectableRow).
function ns.ClearTakes(c, moveId)
	local id = ns.Merged[moveId] or moveId
	local m = ns.Library[id]
	if not m then return false end
	if c.reps == E.clear.move then return (ns.Merged[c.move] or c.move) == id end
	return c.reps == E.clear.ladder and STRENGTH[c.ladder] and m.ladder == c.ladder or false
end

local function dropRecord(s, id, ts)
	s.moves[id] = nil
	s.cleared[id] = ts
end

local function clearRow(s, r)
	if r.reps == E.clear.move then
		local id = ns.Merged[r.move] or r.move
		local m = ns.Library[id]
		if not m then return end
		dropRecord(s, id, r.ts)
		local l = m.ladder
		local lad = (STRENGTH[l] or POSE[l]) and s.ladders[l]
		if lad and ns.StandsFor(l, ns.Rung(s, l), id) then resetCounts(lad) end
	elseif r.reps == E.clear.ladder and STRENGTH[r.ladder] then
		local l = r.ladder
		for id in pairs(ns.Library) do
			if ns.ClearTakes(r, id) then dropRecord(s, id, r.ts) end
		end
		local old = s.ladders[l] or {}
		s.ladders[l] = { hiDays = 0, anchorDays = 0, perfectDays = 0, plateau = 0, underLo = 0, days = old.days or {},
			lastTs = old.lastTs, anchorTs = old.anchorTs, standTs = old.standTs, retiredDays = old.retiredDays,
			appDays = old.appDays, rung = old.start, start = old.start, restartTs = r.ts }
		s.ledger.holds[l] = nil -- its next set starts the hold ledger again (slice 52)
		for rung in ipairs(ns.Ladders[l].rungs) do s.mastered[ns.RungKey(l, rung)] = nil end
	end
end

-- Whether a pause row's code is one of ns.PAUSE's modes (below, read when a row folds).
local function knownPause(code)
	for _, mode in pairs(ns.PAUSE) do
		if mode == code then return true end
	end
	return false
end

-- The Stamina Ledger's holds (slice 52; Progress.lua ns.HoldRead): a typed set of a ladder of ns.HOLD_LEDGER in the
-- row kind it reads (power's reps, the others' holds), of a move of the ladder's step, against the top of the move's
-- own window (ns.EasierTop), with the goal it had (ns.Goal on the state before the set, as correct() judges it; none on
-- a break's placement, ns.Placing) and its answer to "Could you have held longer?" (its reps), read before performed()
-- moves the ladder's clock; a set from before its move's form changed (ns.BeforeForm) reads nothing. The ladder's
-- first set starts its entry at the step's prior (ns.HoldPrior).
local function holdLedger(s, r, m, fresh)
	local l = r.ladder
	local hi = ns.HOLD_LEDGER[l] == r.kind and ns.LedgerTop(s, l, r.move)
	local value = r.kind == K.hold and r.seconds or r.reps
	if not (hi and TYPED[r.outcome] and value > 0) or ns.BeforeForm(r.move, r.ts) then return end
	local placing = ns.StandsFor(l, m.rung, r.move) and ns.Placing(s, l, r.ts)
	local goal = not placing and ns.Goal(s, r.move, r.ts, r.load, warmSet(r)) or nil
	local answer = r.kind == K.hold and r.reps > 0 and r.reps or nil
	local how = ns.HoldHow(value, goal, r.outcome, fresh, answer)
	s.ledger.holds[l] = ns.HoldRead(s.ledger.holds[l], r.ts, value, hi, how, ns.HoldPrior(l, m.rung))
end

-- One progression from every source (slice 26): an app row folds as a game row does (records, form and the fresh-set
-- clock, the ladders' last sets and days, rungs and placement, the cardio ladder, the day plan, pain, the check-in,
-- focus, a record cleared or a ladder started over, the felt-unwell lock), but for what stays the game's: the owed
-- queue (an app row never pays or owes), the game's own start (anyMove, moveRows: Lc.TrialBegun, Setup.BeginTrial)
-- and the game's numbers: the day counts (a game pump's roll; the app's own count beside them, appPumps, appMoves
-- and appRecords, ns.DayCounts, whose moves the day's displays and the Saga's day add in, Stats.lua dayMoves, and an
-- app pump's roll with the game's: ns.Compose's `app`), the feedback numbers' easier picks and early stops and the
-- earliest row (firstTs). A catch-up's row (a device's bit without the app's: a phone's, a watch's or the desktop
-- app's payment and Skip of owed moves, and since slice 58 a match pump's Bank it there, the helper's
-- Session::catch_up, Session::skip and Session::bank) pays and owes as the game's own, a banked move counted among the
-- game day's banked moves, and of whatever kind and outcome (its pump row and its done moves too) moves none of the
-- game's start (firstTs, anyMove, moveRows): it pays only moves a turn-in or a device's bank owed, so it never begins
-- the game, and a player who banks on the computer before the addon is set up still meets the setup.
local function applyRow(s, r)
	local kind, out = r.kind, r.outcome
	local source = ns.Source(r.flags)
	local game = source ~= E.source.app
	local begins = source == E.source.game
	if begins and (not s.firstTs or r.ts < s.firstTs) then s.firstTs = r.ts end
	if ns.UnwellRow(r) then unwell(s, r.ts) end
	if MOVE_KIND[kind] then
		local key = ns.OwedKey(r)
		if out == O.paid then
			if game then
				if s.owed[key] then s.owed[key] = nil else s.paidEarly[key] = true end
			end
			return
		end
		if begins then
			s.anyMove = true
			s.moveRows = s.moveRows + 1
		end
		if out == O.owed or out == O.banked then
			if not game then return end
			if s.paidEarly[key] then
				s.paidEarly[key] = nil
			else
				r.key = key
				s.owed[key] = r
			end
			if out == O.banked then bump(s, r.ts, "banked") end
			return
		end
		local m = ns.Library[r.move]
		if not m then return end -- a newer addon's move, through the mirror
		-- a loaded set's weight, its load row's (the loads map, read before the fold: a correction's corrects it)
		if kind == K.rep then r.load = ns.SetLoad(s, r) end
		correct(s, r)
		out = r.outcome
		bump(s, r.ts, game and "moves" or "appMoves")
		-- before performed() moves the ladder's clock; a warm set (warmSet) is no fresh set, so its share of the form
		-- joins the form only where it passes it (recordCount), never lowering it
		local fresh = ns.FreshAt(s, r.move, r.ts) and not warmSet(r)
		holdLedger(s, r, m, fresh)
		performed(s, r, m, fresh, game)
		-- a counted cardio set keeps a record of its clean count as a rep move does (2026-10-05; a window saved "Form
		-- slipped" none since 2026-10-16); the row keeps its typed count (r.typed) for the sheet's Saga day (Stats.lua
		-- sagaMove)
		local counted = kind == K.cardio and ns.HasFlag(r.flags, "counted")
		local typedKind = kind == K.rep or kind == K.hold or counted and ns.Answer(r.flags) == 0
		local value = kind == K.hold and r.seconds or r.reps
		if typedKind and TYPED[out] and value > 0 then
			r.typed = value
			local days = not counted and not ns.BeforeForm(r.move, r.ts)
				and (s.moves[r.move] and s.moves[r.move].topDays or ns.TopDaysFrom(s, r.move))
			recordCount(s, r.move, value, r.ts, fresh, out == O.placement, r.load)
			if days then topDay(s, r, value, days) end
			easierMark(s, r, value)
		end
		if out == O.beat then bump(s, r.ts, game and "records" or "appRecords") end
	elseif kind == K.pump or kind == K.ding then
		if out == O.offered then
			if begins then s.anyMove = true end
			bump(s, r.ts, game and "pumps" or "appPumps")
			-- a pump that performed nothing leaves no entry, and a catch-up's (a device's payment of owed moves, a match
			-- pump's riders among them, Session::catch_up) opens none: its moves join the pump before, as owed moves riding
			-- a quest's pump do (2026-11-01: each match pump with riders took two of the last five, so a move came again
			-- after two pumps)
			local last = s.recent[#s.recent]
			if source ~= E.source.catchup and not (last and next(last.moves) == nil) then
				s.recent[#s.recent + 1] = { moves = {}, patterns = {} }
				while #s.recent > ns.C.variety_recent_pumps do table.remove(s.recent, 1) end
			end
		end
	elseif kind == K.rung and r.undone then -- a rung change /qp undo reversed, and the undo row, never happened
		return
	elseif kind == K.rung then
		local l = r.ladder
		local cause = ns.RungCause
		-- a start row changes nothing on a ladder that has a rung already: the start writes one only for a ladder
		-- without, and a merge or an import can bring another machine's later
		local start = r.reps == cause.start
		-- nor does a climb (up, plateau, placementHigh, a check's since 2026-10-16) to a rung not above the ladder's rung
		-- now (slice 26): the game and the app climbed the same rung before they synced, and taking the later climb
		-- again would wipe the counts the sets since earned, or lower a rung the other side climbed past (a rung down, a
		-- placementLow and an undo keep their rules)
		local climb = r.reps == cause.up or r.reps == cause.plateau or r.reps == cause.placementHigh
			or r.reps == cause.check
		if l == CARDIO and ns.Ladders[CARDIO].rungs[r.rung] and not (start and s.cardio.rung)
			and not (climb and r.rung <= ns.Rung(s, CARDIO)) then
			local c = s.cardio
			if start then c.start = r.rung end -- the rung the player starts on is theirs too (ns.BestRung)
			c.rung, c.stops, c.stopTs, c.placing, c.countTs, c.proveTs, c.breakTs = r.rung, 0, nil, nil, nil, nil, nil
			c.clean, c.cleanForms, c.cleanFormTs, c.full = 0, {}, {}, 0
			-- the game day of a climb counts no stop (slice 53's footing, cardioMove), and a step down's session no day
			-- toward the climb back (return_days: its first day comes in a new session by the night rule), a full move
			-- included (slice 52: the full moves are the step's, and a climb doubts the ledger again, at least
			-- ledger_ask_doubt[2], so the new step's moves start shorter by the rung's offset and the ask comes back)
			c.climbTs = climb and r.ts or nil
			local downTs = r.reps == cause.down and r.ts or nil
			c.cleanTs, c.fullTs = downTs, downTs
			local L = s.ledger.steady
			if L and climb then L.d = math.max(L.d, ns.C.ledger_ask_doubt[2]) end
		elseif (STRENGTH[l] or POSE[l]) and ns.Ladders[l].rungs[r.rung]
			and not (start and s.ladders[l] and (s.ladders[l].rung or s.ladders[l].placedTs))
			and not (climb and r.rung <= ns.Rung(s, l)) then
			local lad = ladderState(s, l)
			lad.from, lad.rung = ns.Rung(s, l), r.rung
			resetCounts(lad)
			-- the hold ledger starts the new step at its prior (slice 52)
			local e = s.ledger.holds[l]
			if e then ns.HoldClimb(e, ns.HoldPrior(l, r.rung)) end
			-- the rung the player starts on is theirs too (ns.BestRung), and a Trial placement below it lowers it
			if start or r.reps == cause.placementLow then lad.start = r.rung end
			-- a rung-up's new anchor is placed from the next game day, not a cadence later (a start row, a rung down
			-- or an undo opens nothing)
			lad.opened = climb and r.ts or nil
		end
	elseif kind == K.pain then
		s.pain[r.move] = r.reps == 1 and r.ts or nil
	elseif kind == K.checkin then
		s.checkinTs = r.ts
	elseif kind == K.focus then
		if E.focusKeys[r.reps] then s.focus = r.reps end -- the latest wins; a newer addon's code changes nothing
	elseif kind == K.clear then
		clearRow(s, r)
	elseif kind == K.mastered then
		-- a step mastered (2026-10-16, slice 42; ns.Progression's row or a sweep's, ns.MasterySweep, any source's): the
		-- first kept until its strength ladder starts over (clearRow), on a rung this addon has; one /qp undo took back
		-- (undone) and the undo row itself (reps the undo cause, 2026-10-17) never happened, and nor did one from before its
		-- ladder's cut (masteryCut: a pose family's before slice 47's goal poses, its F3, built 2026-10-31; the cardio
		-- ladder's before its first clean windows came from answers alone, 2026-10-31): the step masters again once its
		-- moves are topped again, its row the sweep's (ns.MasterySweep: the game at login, the apps at load) or the last
		-- top's
		local w = ns.Ladders[r.ladder]
		local cut = masteryCut(r.ladder)
		if w and w.rungs[r.rung] and not r.undone and r.reps ~= ns.RungCause.undo and not (cut and r.ts < cut) then
			local key = ns.RungKey(r.ladder, r.rung)
			s.mastered[key] = s.mastered[key] or r.ts
		end
	elseif kind == K.load then
		-- a row written since the fold (ns.AddRow, an app's own pump): its set row folds next and reads it; a full fold
		-- reads every load row before it begins (loads, below) and skips them
		s.loads = s.loads or {}
		local key = setKey(r)
		if ns.CorrectionBeats(r, s.loads[key]) then s.loads[key] = r end
	elseif kind == K.pause then
		-- a resume (0) or a mode of ns.PAUSE; a newer addon's code changes nothing, as a focus row's
		if r.reps == 0 then
			s.pause[r.char] = nil
		elseif knownPause(r.reps) then
			s.pause[r.char] = { ts = r.ts, mode = r.reps }
		end
	end
end

-- A character's pause (David, 2026-10-01: "an option to pause the addon or just deactivate on one character. If I play
-- with friends I don't want to do it sometimes"): a pause row's reps names how long (ns.PAUSE: an hour, four hours
-- since 2026-10-04, the rest of the game day, off on the character until a resume row, reps 0, or, since 2026-10-04,
-- while grouped: until the resume row the lifecycle writes once the party is no longer mixed, Events.lua
-- Lc.CheckParty). While it holds, turn-ins count for nothing, as the General page's Enabled switch does for every
-- character. Returns the mode and its end (nil but for an hour or four), or nil.
ns.PAUSE = { hour = 1, day = 2, char = 3, fourHours = 4, group = 5 }

function ns.Paused(s, char, now)
	local p = s.pause and s.pause[char]
	if not p then return nil end
	local length = p.mode == ns.PAUSE.hour and ns.C.pause_hour_s
		or p.mode == ns.PAUSE.fourHours and ns.C.pause_four_hours_s
	if length then
		local ends = p.ts + length
		if now < ends then return p.mode, ends end
	elseif p.mode == ns.PAUSE.day then
		if ns.day(p.ts) == ns.day(now) then return p.mode end
	elseif p.mode == ns.PAUSE.char or p.mode == ns.PAUSE.group then
		return p.mode
	end
	return nil
end

-- Every row goes through the fold's rules, then through the sheet's (Stats.lua), which also learns what the fold held
-- before the row: whether anything was unfinished (a payment that empties the queue, a day that ended clean) and a
-- move row's record (`best`, and a weighted record's `topLoad` and `topReps`), from which the Saga's day card names a
-- record's "was" (2026-10-15). `part` says how much of the row the sheet takes: nil or "own" all of it
-- (ns.SheetApply; the game's every row), "mark" a climb's first reach and a felt-unwell day alone (ns.SheetMark: the
-- game's kept rows on the app's own sheet, slice 35's tools/apppump.lua, as the phones fold them), "skip" nothing.
function ns.Apply(s, r, part)
	local mv = MOVE_KIND[r.kind] and s.moves[r.move]
	local before = { owed = next(s.owed) ~= nil, best = mv and mv.best, topLoad = mv and mv.topLoad,
		topReps = mv and mv.topReps }
	applyRow(s, r)
	if part == nil or part == "own" then
		ns.SheetApply(s, r, before)
	elseif part == "mark" then
		ns.SheetMark(s, r)
	elseif part ~= "skip" then
		error("ns.Apply: no sheet part " .. tostring(part))
	end
end

-- The rows of `kind` (correction or load), keyed by the set each names (its move as the fold reads it, a merged
-- move's the move that took it): the one that wins (ns.CorrectionBeats, by the round). The fold keeps both maps in
-- the state, so trimming the ring reads them instead of decoding the whole log again for every row written. `keep(r,
-- str)`, where given, takes only the rows it accepts (ns.Replay's).
local function named(log, kind, keep)
	local map = {}
	for _, str in ipairs(log or {}) do
		local r = ns.DecodeRow(str)
		if r and r.kind == kind and (not keep or keep(r, str)) then
			local key = setKey(r)
			if ns.CorrectionBeats(r, map[key]) then map[key] = r end
		end
	end
	return map
end

-- Correction rows; load rows (2026-10-09), whose weight is in tenths of a kg, one of 0 taking the weight off.
local function corrections(log) return named(log, K.correction) end
local function loads(log) return named(log, K.load) end

-- The rung changes a later undo row reverses (a corrected count's: Correct a count and /qp undo, ns.CorrectionRows),
-- with the undo rows themselves, keyed by the row: neither happened, so the fold skips both (the ladder keeps its
-- counts, and the start rung a Trial placement would have lowered) and the sheet pays nothing, so a rung reached again
-- later pays as the first time did. An undo names the row it reverses (2026-10-19, the program's double-check, which
-- decided slice 44's open note: its ts in the undo's questID, ns.CorrectionRows) and pairs with that row alone, of its
-- own character on that ladder, not paired yet and no undo itself, at another rung: a watch's climb that reached the
-- phone after the phone's undo of an older climb sorts before the undo, and an undo by "the latest" took it back
-- instead. A named undo whose row the log lacks reverses nothing (it never happened either). An undo from before
-- (questID 0) reverses the latest rung row of its own character on that ladder, up or down, at another rung, and none
-- once that latest was paired. A correction writes it with the corrected set's character: the app's rung rows (slice
-- 26, character 0, and since slice 44 an app's undo of its own set's) never pair with a game's undo, nor a game's with
-- an app's, here and in the phones' ports alike. A mastered row's undo (kind mastered, reps the undo cause;
-- 2026-10-17, slice 42's review: the count that mastered a step corrected, which the corrected count no longer
-- masters) pairs the same way with the mastered row it names of its own character on that ladder and step (questID 0:
-- the latest); an undo with nothing to pair masters nothing (applyRow). Beside the set, what an undo from before
-- written after the log would pair with (ns.RungsLatest, slice 44): the rung row of each ladder and character (the
-- second, keyed "ladder:char") and the mastered row of each step and character (the third, "ladder:rung:char"), by
-- text, none where the latest was paired already; fourth, every other row a named undo may still pair with (no undo,
-- not paired, not its key's latest), by text. The pairing runs on from `base`, the log's baseline, which carries
-- those maps as they stood at its edge (`pairing`, ns.Trim's: `rungs`, `steps` and `kept`, the other rows a named
-- undo may name, of the last correct_days game days to the edge): an undo whose row a trim folded into the baseline
-- ahead of it still pairs with it (2026-10-19, slice 44: a corrected set's climb, older than its undo row, leaves the
-- ring first, and an undo read unpaired reset the ladder's counts), so a fold of the ring pairs every row as a fold of
-- every row did. `upto` stops it at that row of the log (ns.Trim's edge); nil, at its end.
local function pairing(log, base, upto)
	local UNDO = ns.RungCause.undo
	local set, last, step, open = {}, {}, {}, { rung = {}, step = {} }
	-- the rows of a key (`map`: open.rung or open.step) a named undo may pair with, in the log's order
	local function push(map, key, e)
		local list = map[key] or {}
		list[#list + 1] = e
		map[key] = list
	end
	local function take(map, key, e)
		local list = map[key] or {}
		for k = #list, 1, -1 do
			if list[k] == e then
				table.remove(list, k)
				return
			end
		end
	end
	-- the newest of them at `ts` (an undo's questID), at another rung than `rung` where given
	local function nameOf(map, key, ts, rung)
		local list = map[key] or {}
		for k = #list, 1, -1 do
			local e = list[k]
			if e.ts == ts and (rung == nil or e.rung ~= rung) then return e end
		end
		return nil
	end
	-- a step row as the fold reads its rung (ns.ShiftRow: nil for a mastered row of a rung that is gone)
	local function read(str)
		local r = type(str) == "string" and ns.DecodeRow(str)
		return r and ns.ShiftRow(r)
	end
	local carried = type(base) == "table" and type(base.pairing) == "table" and base.pairing or {}
	local older = {}
	for _, str in ipairs(type(carried.kept) == "table" and carried.kept or {}) do
		local r = read(str)
		if r and (r.kind == K.rung or r.kind == K.mastered) and r.reps ~= UNDO then
			older[#older + 1] = { str = str, r = r }
		end
	end
	table.sort(older, function(a, b)
		if a.r.ts ~= b.r.ts then return a.r.ts < b.r.ts end
		return a.str < b.str
	end)
	for _, x in ipairs(older) do
		local r = x.r
		if r.kind == K.rung then
			push(open.rung, r.ladder .. ":" .. r.char, { str = x.str, rung = r.rung, ts = r.ts })
		else
			push(open.step, r.ladder .. ":" .. r.rung .. ":" .. r.char, { str = x.str, ts = r.ts })
		end
	end
	for key, str in pairs(type(carried.rungs) == "table" and carried.rungs or {}) do
		local r = read(str)
		if r then
			last[key] = { str = str, rung = r.rung, ts = r.ts }
			if r.reps ~= UNDO then push(open.rung, key, last[key]) end
		end
	end
	for _, str in pairs(type(carried.steps) == "table" and carried.steps or {}) do
		local r = read(str)
		if r then
			local key = r.ladder .. ":" .. r.rung .. ":" .. r.char -- its step as read, which a shift lowers
			step[key] = { str = str, ts = r.ts }
			push(open.step, key, step[key])
		end
	end
	log = log or {}
	for i = 1, upto or #log do
		local str = log[i]
		local r = read(str)
		if r and r.kind == K.rung then
			local key = r.ladder .. ":" .. r.char
			if r.reps == UNDO then
				local prev
				if r.questID > 0 then
					prev = nameOf(open.rung, key, r.questID, r.rung)
				elseif last[key] and last[key].rung ~= r.rung then
					prev = last[key]
				end
				if prev then
					set[prev.str], set[str] = true, true
					take(open.rung, key, prev)
					if last[key] == prev then last[key] = nil end
				elseif r.questID > 0 then
					set[str] = true
				else
					last[key] = { str = str, rung = r.rung, ts = r.ts }
				end
			else
				last[key] = { str = str, rung = r.rung, ts = r.ts }
				push(open.rung, key, last[key])
			end
		elseif r and r.kind == K.mastered then
			local key = r.ladder .. ":" .. r.rung .. ":" .. r.char
			if r.reps ~= UNDO then
				step[key] = { str = str, ts = r.ts }
				push(open.step, key, step[key])
			else
				local prev
				if r.questID > 0 then prev = nameOf(open.step, key, r.questID) else prev = step[key] end
				if prev then
					set[prev.str], set[str] = true, true
					take(open.step, key, prev)
					if step[key] == prev then step[key] = nil end
				elseif r.questID > 0 then
					set[str] = true
				end
			end
		end
	end
	local rungs, steps, others = {}, {}, {}
	for key, e in pairs(last) do rungs[key] = e.str end
	for key, e in pairs(step) do steps[key] = e.str end
	for _, kind in ipairs({ "rung", "step" }) do
		local latest = kind == "rung" and last or step
		for key, list in pairs(open[kind]) do
			for _, e in ipairs(list) do
				if latest[key] ~= e then others[#others + 1] = e.str end
			end
		end
	end
	table.sort(others)
	return set, rungs, steps, others
end

local function undone(log, base)
	return (pairing(log, base))
end

ns.UndoPairs = pairing

-- The latest rung row of ladder `l` in `log` (row texts, oldest first) that the fold of `state` applied (not undone:
-- its `undone`, keyed by the texts), decoded, its rung as the fold reads it (ns.ShiftRow); nil where none. The repairs
-- read it (Events.lua Lc.UndoOldClimbs, the desktop app's sweep: Progress.lua ns.HeldClimb).
function ns.LatestRung(state, log, l)
	for i = #log, 1, -1 do
		local r = ns.DecodeRow(log[i])
		if r and r.kind == K.rung and r.ladder == l and not state.undone[log[i]] then return ns.ShiftRow(r) end
	end
	return nil
end

-- A state ready to fold a log onto its baseline: the log's corrections, weights and undone rung-ups, or the ones the
-- caller has.
local function begin(db, fixes, gone, weights)
	local s = ns.NewState(db.baseline)
	s.corrections = fixes or corrections(db.log)
	s.loads = weights or loads(db.log)
	s.undone = gone or undone(db.log, db.baseline)
	return s
end

-- One row as the fold reads it (a split ladder's on its successors, an old pose's in its family: ns.Translate),
-- through as much of the sheet as `part` says (ns.Apply); the generators fold their rows through it too.
function ns.ApplyRead(s, r, part)
	local a, b = ns.Translate(r, s)
	if a then ns.Apply(s, a, part) end
	if b then ns.Apply(s, b, part) end
end

-- Folds log[first..last] into s. A row that does not fold (a newer addon's row through the mirror, a bug in a rule)
-- is skipped and the first such error reported, so the rows after it still fold: the pain flags, the felt-unwell lock
-- and the owed queue never vanish with it. `stop(r)` ends the fold at the first row it accepts and returns that row,
-- once it folded or, with `ahead`, before it folds (the row as stored then). `part(str)`, the db's sheetPart when it
-- has one (tools/apppump.lua's), names each row's sheet part (ns.Apply); the game's save has none, and every row folds
-- whole.
local function applyAll(s, log, first, last, stop, part, ahead)
	local failed, found
	for i = first, last do
		local r = ns.DecodeRow(log[i])
		if r and r.kind ~= K.correction and r.kind ~= K.load then
			if ahead and stop(r) then
				found = r
				break
			end
			if s.undone[log[i]] then r.undone = true end
			local ok, err = pcall(ns.ApplyRead, s, r, part and part(log[i]))
			if not ok then failed = failed or err end
			if stop and not ahead and stop(r) then
				found = r
				break
			end
		end
	end
	if failed then ns.Report(failed) end
	return found
end

-- A fold of `db`'s whole log onto its baseline, which ns.state never sees (a correction's trial log: ns.CorrectionRows
-- reads the state its rows would leave); ns.Fold keeps it as the addon's state.
function ns.FoldOf(db)
	local log = db.log or {}
	local s = begin(db)
	applyAll(s, log, 1, #log, nil, db.sheetPart)
	return s
end

function ns.Fold(db)
	local s = ns.FoldOf(db)
	ns.state = s
	return s
end

-- Whether move row `r` is set row `set` (each as stored or as the fold read it, a merged move's as the move that took
-- it): its kind, owed key, move and source, never a paid row. Correct a count's window finds its set by it too
-- (Settings.lua listedSet).
function ns.IsSet(r, set)
	return r.kind == set.kind and (ns.Merged[r.move] or r.move) == (ns.Merged[set.move] or set.move)
		and r.outcome ~= O.paid and ns.OwedKey(r) == ns.OwedKey(set) and ns.Source(r.flags) == ns.Source(set.flags)
end

-- The place of set row `set` in a log (its first row that is that set, isSet), nil once it left the ring.
function ns.SetIndex(log, set)
	for i, str in ipairs(log or {}) do
		local r = ns.DecodeRow(str)
		if r and ns.IsSet(r, set) then return i end
	end
	return nil
end

-- The state just after one move row and that row as the fold read it (its correction applied, marked for
-- ns.Progression), from a fold that stops there; with `ahead` the state just before it and the row as stored: a
-- correction asks the rung rules again with the corrected count (ns.CorrectionRows) and judges it against the state
-- before the set (ns.CorrectionCheck). `keep(r, str)`, where given, takes only the correction and load rows it accepts
-- (ns.CorrectionRows: the set's own side's, whose step rows the log holds). nil when the row fell out of the ring.
function ns.Replay(db, target, ahead, keep)
	local log = db.log or {}
	local s = begin(db, keep and named(log, K.correction, keep), nil, keep and named(log, K.load, keep))
	local r = applyAll(s, log, 1, #log, function(r) return ns.IsSet(r, target) end, db.sheetPart, ahead)
	if r then return s, r end
	return nil
end

-- How set row `r`'s own side reads it (2026-10-19, the program's double-check: the device a set came from could not act
-- on another device's correction of it, though only its own side asks the rung rules), where the correction or the
-- load row that wins in `state` (the fold of `db`'s log) is the other side's: the set's side is an app's for a set with
-- the app bit and the game's for every other (ns.CorrectionRows' sides), and its step rows followed its own side's
-- corrections and load rows alone. nil where both winners are none or its own side's, or where its own side's replay
-- (ns.Replay with that side's rows) reads the set alike; else { value, load } as that replay reads it (ns.SetValue; the
-- weight in tenths of a kg, ns.SetLoad, nil none). A window of the set's own source offers Save at the winning value
-- with one line saying so ("Corrected to 15 on another device. Save it here to move your step with it."): a new
-- round of the same value, which asks the rung rules again (ns.CorrectionRows).
function ns.OwnSide(state, db, r)
	local app = ns.Source(r.flags) == E.source.app
	local function mine(x) return (ns.Source(x.flags) == E.source.app) == app end
	local key = setKey(r)
	local c, l = state.corrections and state.corrections[key], state.loads and state.loads[key]
	if (not c or mine(c)) and (not l or mine(l)) then return nil end
	local s = ns.Replay(db, r, nil, mine)
	if not s then return nil end
	local value, load = ns.SetValue(s, r), ns.SetLoad(s, r)
	if value == ns.SetValue(state, r) and load == ns.SetLoad(state, r) then return nil end
	return { value = value, load = load }
end

-- The owed queue, oldest first; every payment pays moves one for one.
function ns.OwedList(s)
	local list = {}
	for _, r in pairs(s.owed) do list[#list + 1] = r end
	table.sort(list, function(a, b)
		if a.ts ~= b.ts then return a.ts < b.ts end
		if a.slot ~= b.slot then return a.slot < b.slot end
		return a.questID < b.questID
	end)
	return list
end

function ns.OwedCount(s)
	local n = 0
	for _ in pairs(s.owed) do n = n + 1 end
	return n
end

-- "Felt unwell" locks every pump until the next game day (04:00); folded from the rows, so a relog keeps it.
function ns.LockedToday(s, now)
	return s.unwellTs ~= nil and ns.day(s.unwellTs) == ns.day(now)
end

-- A game day's counts: the game's pumps, moves, records beaten and banked moves (a game pump's roll), and in `app` the
-- app's own pumps, moves and records (slice 26). The day's displays read its moves, every source's, with the app's
-- share (the HUD's QP line; Stats.lua dayMoves for the Saga's day, the Today tab, /qp status and the day card); their
-- pumps and records are the Saga's since 2026-10-15 (Stats.lua dayCount: the pumps with a move done that day, each
-- move's records once), and these pumps and records stay the rows fixture's (tools/gen-fixtures.lua) and a pump roll's.
function ns.DayCounts(s, day)
	local t = s.days[day] or {}
	return { pumps = t.pumps or 0, moves = t.moves or 0, records = t.records or 0, banked = t.banked or 0,
		app = { pumps = t.appPumps or 0, moves = t.appMoves or 0, records = t.appRecords or 0 } }
end

-- The ring and its baseline --------------------------------------------------------------------------------------

-- Keeps the entries of a day-keyed map from `since` on (the day keys are "YYYY-MM-DD", so they compare as strings).
local function recentDays(map, since)
	local out = {}
	for d, v in pairs(map or {}) do
		if d >= since then out[d] = v end
	end
	return out
end

-- The baseline keeps what the rules still read: the day maps only as far back as they look (a ladder's sets a day
-- for the feedback numbers' feedback_days, two days for the regions' counts, variety and the day plan, sixty for the
-- day counts), so the save stays small however long the player plays.
local function storeBaseline(db, s)
	local b = { owed = {}, split = true, shifted = {} }
	for l in pairs(ns.Shifted) do b.shifted[l] = true end
	for _, key in ipairs(KEEP) do
		local v = s[key]
		b[key] = type(v) == "table" and CopyTable(v) or v
	end
	for key, r in pairs(s.owed) do b.owed[key] = ns.EncodeRow(r) end
	db.baseline = b
	local now = s.lastDoneTs
	if not now then return end -- no move yet: nothing dated to prune
	-- a day longer than the feedback window, which ends at the current game day (never before the last move)
	local window, twoDays = ns.day(now - ns.C.feedback_days * DAY_S), ns.day(now - 2 * DAY_S)
	b.days = recentDays(b.days, ns.day(now - 60 * DAY_S))
	for d, t in pairs(b.days) do
		if d < twoDays then t.blocks, t.done = nil, nil end
	end
	for _, lad in pairs(b.ladders) do
		lad.days = recentDays(lad.days, window)
		if lad.retiredDays then lad.retiredDays = recentDays(lad.retiredDays, window) end
		if lad.appDays then lad.appDays = recentDays(lad.appDays, window) end
	end
	for _, reg in pairs(b.regions) do reg.days = recentDays(reg.days, twoDays) end
	for _, p in pairs(b.patterns) do p.days = recentDays(p.days, twoDays) end
	b.cardio.days = nil -- its one reader, the least-used ladder of a repeatable quest, went with the day plan
	ns.Stats.Prune(b.sheet, now)
end

-- Rows past the ring fold into the baseline: the records, the ladders, the clocks, the unpaid owed moves, the pain
-- flags and the characters' pauses survive. `fixes`, `gone` and `weights` are the fold's correction map, undone set and
-- loads map when the caller has them (a row just written), so a full ring is not decoded again for every row. A load
-- row leaves with the set row after it, which the ring could not read without it. The baseline also keeps its
-- horizon: the latest time it folded (`through`, the running maximum, as the merge orders rows) and the rows at that
-- time (`edge`), so a merge never folds a row twice; and the undo pairing as it stands after the rows it folded
-- (`pairing`: ns.UndoPairs' rung and mastered rows an undo after them pairs with, `rungs` and `steps`, and `kept`, the
-- other rows a named undo may still name: those of the last correct_days game days to the edge's time, the days a
-- correction reaches back), so an undo row left in the ring still pairs with its row folded here.
function ns.Trim(db, fixes, gone, weights)
	local over = #db.log - ns.C.ring_rows
	if over <= 0 then return end
	while over < #db.log and (ns.DecodeRow(db.log[over]) or {}).kind == K.load do over = over + 1 end
	local old = type(db.baseline) == "table" and db.baseline or {}
	local through = tonumber(old.through) or 0
	local edge = type(old.edge) == "table" and old.edge or {}
	local s = begin(db, fixes, gone, weights)
	applyAll(s, db.log, 1, over, nil, db.sheetPart)
	for i = 1, over do
		local r = ns.DecodeRow(db.log[i])
		if r and r.ts > through then through, edge = r.ts, {} end
		edge[db.log[i]] = true
	end
	local _, rungs, steps, others = pairing(db.log, old, over)
	local first, kept = ns.day(ns.DayMid(through, 1 - ns.C.correct_days)), {}
	for _, str in ipairs(others) do
		local r = ns.DecodeRow(str)
		if r and ns.day(r.ts) >= first then kept[#kept + 1] = str end
	end
	storeBaseline(db, s)
	db.baseline.through, db.baseline.edge = through, edge
	db.baseline.pairing = { rungs = rungs, steps = steps, kept = kept }
	local keep = {}
	for i = over + 1, #db.log do keep[#keep + 1] = db.log[i] end
	db.log = keep
end

-- The mirror ----------------------------------------------------------------------------------------------------

-- A count of rows by source (ns.Source: the game's, the catch-ups', the apps'), every source present: the addon's log
-- names a save's rows and a merge's this way (Log.lua).
local function bySource()
	local counts = {}
	for _, id in pairs(E.source) do counts[id] = 0 end
	return counts
end

function ns.RowsBySource(list)
	local counts = bySource()
	for _, str in ipairs(list or {}) do
		local r = ns.DecodeRow(str)
		if r then
			local source = ns.Source(r.flags)
			counts[source] = counts[source] + 1
		end
	end
	return counts
end

-- The incoming characters `chars` (a mirror's or an export's list, each with its GUID) on this DB's: each index to the
-- index of the character of that GUID here, a character this DB lacks added (at most 255: the strip's one byte). A
-- row's `char` is an index into its own save's list, so every row that comes in, and every part of an export's
-- baseline that names a character, takes its number through this map (ns.MergeMirror, ns.Import).
local function charMap(db, chars)
	db.chars = type(db.chars) == "table" and db.chars or {}
	local byGuid, map = {}, {}
	for i, c in ipairs(db.chars) do
		if type(c) == "table" and c.guid then byGuid[c.guid] = i end
	end
	for i, c in ipairs(type(chars) == "table" and chars or {}) do
		if type(c) == "table" and type(c.guid) == "string" then
			local j = byGuid[c.guid]
			if not j and #db.chars < 255 then
				db.chars[#db.chars + 1] = { guid = c.guid, name = c.name, realm = c.realm, class = c.class }
				j = #db.chars
				byGuid[c.guid] = j
			end
			map[i] = j
		end
	end
	return map
end

-- An export's baseline (ns.Import) under this DB's character numbers, `map` (charMap's): every part of it that keys
-- or carries a character takes the number the merge gives that character's rows, so what the ring's rows read in it
-- names the same character: the characters' pauses (keyed by character), the owed rows and the horizon's edge (row
-- texts, the edge's its keys) and the undo pairing at its edge (ns.Trim's `pairing`: the rung row of each ladder and
-- character, keyed "ladder:char", and the mastered row of each step and character, "ladder:rung:char", each by its
-- text, and the other rows a named undo may name, `kept`, by text). A character the map lacks keeps its number, as
-- the merge keeps a row's. Returns a new baseline; the rest is shared with `b`.
local function renumbered(b, map)
	local function text(str)
		local r = type(str) == "string" and ns.DecodeRow(str)
		if not r then return str, nil end
		r.char = map[r.char] or r.char
		return ns.EncodeRow(r), r
	end
	local out = {}
	for key, v in pairs(b) do out[key] = v end
	if type(b.pause) == "table" then
		out.pause = {}
		for char, p in pairs(b.pause) do out.pause[map[char] or char] = p end
	end
	if type(b.owed) == "table" then
		out.owed = {}
		for key, str in pairs(b.owed) do out.owed[key] = (text(str)) end
	end
	if type(b.edge) == "table" then
		out.edge = {}
		for str, v in pairs(b.edge) do out.edge[(text(str))] = v end
	end
	if type(b.pairing) == "table" then
		local rungs, steps = {}, {}
		for key, str in pairs(type(b.pairing.rungs) == "table" and b.pairing.rungs or {}) do
			local new, r = text(str)
			rungs[r and r.ladder .. ":" .. r.char or key] = new
		end
		for key, str in pairs(type(b.pairing.steps) == "table" and b.pairing.steps or {}) do
			local new, r = text(str)
			steps[r and r.ladder .. ":" .. r.rung .. ":" .. r.char or key] = new
		end
		local kept = {}
		for _, str in ipairs(type(b.pairing.kept) == "table" and b.pairing.kept or {}) do kept[#kept + 1] = (text(str)) end
		table.sort(kept)
		out.pairing = { rungs = rungs, steps = steps, kept = kept }
	end
	return out
end

-- Merges the helper's mirror into the DB: its characters are mapped onto this DB's by GUID (a row's `char` is an index,
-- charMap), the rows are the union of both logs in time order (a row both sides wrote is kept once), the profiles are
-- the mirror's when it has any (each unless this save's changed later, ns.ProfileNewer), and a character without a
-- profile of its own here takes the one the mirror names (profileKeys; on the beta, which loads no save, that is every
-- character). Rows the addon wrote while the helper was off stay; only a mirror row the baseline already folded (at or
-- before its horizon) is left out. `m.through` is the incoming log's own horizon, when known; `floor` leaves out
-- incoming rows older than it (the mirror after /qp reset all: the helper keeps every row). The mirror's rows are this
-- install's, the other installs' and the apps' (slice 26: an app row keeps char 0, which no remap touches). Returns the
-- incoming rows it took, by source.
function ns.MergeMirror(db, m, floor)
	db.log = type(db.log) == "table" and db.log or {}
	local map = charMap(db, m.chars)
	-- Each log keeps its own order: a row that repeats an older ts (a paid row, a correction) stays after the rows
	-- written before it, so the fold reads the queue as it was; the two logs interleave by time. An app row (the app
	-- bit, ns.Source) never pays or owes, so it has no queue order to keep: it sorts at its own time wherever its log
	-- holds it and moves no other row's time (2026-10-09: the helper keeps the app rows in the order it took them, and
	-- a phone's row taken after another device's newer one folded after it). A mirror row at or before the baseline's
	-- horizon is folded in it already, or too old to fold after it: it is left out.
	local base = type(db.baseline) == "table" and db.baseline or {}
	local through = tonumber(base.through) or 0
	local edge = type(base.edge) == "table" and base.edge or {}
	-- A row both logs hold is kept once, counted by copies: two rows alike in one log (the same second) stay two.
	local copies, merged, beforeReset, taken = {}, {}, 0, bySource()
	local function takeAll(list, remap, from)
		local at = from
		for _, str in ipairs(list) do
			local r = ns.DecodeRow(str)
			if r then
				if remap then
					r.char = remap[r.char] or r.char
					str = ns.EncodeRow(r)
				end
				local rowAt = r.ts
				if ns.Source(r.flags) ~= E.source.app then
					at = math.max(at, r.ts)
					rowAt = at
				end
				local keep = true
				if not remap then
					copies[str] = (copies[str] or 0) + 1
				elseif rowAt < through or (rowAt == through and edge[str]) then
					keep = false -- folded in the baseline already
				elseif floor and r.ts < floor then
					keep = false -- from before a reset (a paid row repeats its owed row's time: that owed move went too)
					beforeReset = beforeReset + 1
				elseif (copies[str] or 0) > 0 then
					copies[str], keep = copies[str] - 1, false -- this save holds it
				end
				if keep then
					merged[#merged + 1] = { s = str, ts = rowAt, n = #merged + 1 }
					if remap then
						local source = ns.Source(r.flags)
						taken[source] = taken[source] + 1
					end
				end
			end
		end
	end
	takeAll(db.log, nil, through)
	takeAll(m.rows, map, tonumber(m.through) or 0) -- an export's ring runs on from its own horizon
	table.sort(merged, function(a, b)
		if a.ts ~= b.ts then return a.ts < b.ts end
		return a.n < b.n
	end)
	local log = {}
	for i, e in ipairs(merged) do log[i] = e.s end
	db.log = log
	ns.Trim(db)
	-- The mirror is newer, so its profiles win by name, but a profile this save changed after the mirror's copy (its
	-- own change stamp, ns.Touch, is later: the beta loaded a mirror one /reload behind, 2026-10-09) stays, and so does
	-- a profile only this save has. A mirror that still holds rows from before a reset was written before the helper
	-- heard of it: its profiles are the deleted ones.
	if type(m.profiles) == "table" and beforeReset == 0 then
		db.profiles = type(db.profiles) == "table" and db.profiles or {}
		for name, p in pairs(m.profiles) do
			if type(name) == "string" and type(p) == "table" and ns.ProfileNewer(p, db.profiles[name]) then
				db.profiles[name] = CopyTable(p)
			end
		end
	end
	if type(m.profileKeys) == "table" and beforeReset == 0 then
		db.profileKeys = type(db.profileKeys) == "table" and db.profileKeys or {}
		for guid, name in pairs(m.profileKeys) do
			if type(guid) == "string" and type(name) == "string" and not db.profileKeys[guid] then
				db.profileKeys[guid] = name
			end
		end
	end
	return taken
end

-- The export string (docs/plan.md slice 3; wow-addon-dev quests-and-persistence.md, "Export and import strings"):
-- "!QP1!" and the standard Base64 of the raw Deflate of the CBOR of the payload, no library. The payload carries the
-- ring, the baseline, the characters, the profiles and the folded sheet (the whole sheet summary, ns.SheetSummary,
-- without the strip's char, body and fig: the phone app shows the game's sheet from it without the game's rows); an
-- import merges like the mirror (the union of the rows, characters by GUID, the profiles by name where they changed
-- later, ns.ProfileNewer, from the save's reset on) and takes the baseline only into a save that has no history of its
-- own, under this save's character numbers (renumbered: its pauses, owed rows, edge and undo pairing name the
-- characters the merged rows name); an export that carries a /qp reset all newer than the save's resets the save first,
-- as the mirror's does (ns.MirrorReset), so what the import shows is what the next login folds. An export older than
-- the save's latest reset brings nothing from before it: neither its rows from before the reset (the merge's floor) nor
-- a baseline whose horizon is older than the reset; with nothing left, it is refused. Never loadstring.
ns.EXPORT_PREFIX = "!QP1!"
-- /qp qr's snapshot (QR.lua): the strip's sheet payload (ns.SheetPayload) without what a phone reading a snapshot has
-- no use for, in the export's encoding
ns.SNAPSHOT_PREFIX = "!QPS1!"

function ns.Export(db, state)
	-- the save's own schema: a read-only save, written by a newer addon, never passes for this one's
	local payload = { schema = type(db.schema) == "number" and db.schema or ns.SCHEMA, exportedAt = ns.now(),
		log = db.log, baseline = db.baseline, chars = db.chars, profiles = db.profiles, profileKeys = db.profileKeys,
		sheet = ns.SheetSummary(state, nil, db.savedAt, ns.profile, ns.Horizon(db)) }
	local cbor = C_EncodingUtil.SerializeCBOR(payload)
	local packed = C_EncodingUtil.CompressString(cbor, Enum.CompressionMethod.Deflate)
	return ns.EXPORT_PREFIX .. C_EncodingUtil.EncodeBase64(packed)
end

-- Whether a time from outside this save's own writing may be one: no earlier than C.row_min_ts, before QuestPump wrote
-- any row, and at most `ahead` seconds past this clock.
local function timely(ts, ahead)
	return ts >= ns.C.row_min_ts and ts <= ns.now() + ahead
end

-- Whether a row from outside this save's own writing may carry its time (2026-10-31, the release's security review):
-- timely, at most C.row_ahead_s past this clock, and a reset (a phone's Start over among them) or a felt-unwell row
-- (ns.UnwellRow), which floors the save or locks it until its game day ends, at most C.row_ahead_floor_s, so no crafted
-- one dated in the future floors or locks the save for long. An import string with one is refused whole (ns.Import), a
-- mirror's row left out (ns.BindDB); the helper and the phones hold their take-ins to the same numbers
-- (contract/library.json).
function ns.RowTimely(r)
	local floors = r.kind == K.reset or ns.UnwellRow(r)
	return timely(r.ts, floors and ns.C.row_ahead_floor_s or ns.C.row_ahead_s)
end

-- Returns true and the time of a newer reset it took (nil without one: the caller says it once the import holds), or
-- false and why ("large", "prefix", "broken", "newer", "old"); on success the caller folds again. A string longer than
-- C.import_max_chars bytes as it arrived is refused before anything of it is read, as the phones refuse one; one
-- holding a row no clock could have written (ns.RowTimely), or a baseline horizon past this clock's floor allowance
-- (the merge leaves out every row at or before it, as a reset's floor does), is refused as broken.
function ns.Import(db, text)
	text = text or ""
	if #text > ns.C.import_max_chars then return false, "large" end
	text = strtrim(text)
	local prefix = ns.EXPORT_PREFIX
	if text:sub(1, #prefix) ~= prefix then return false, "prefix" end
	local ok, payload = pcall(function()
		local packed = C_EncodingUtil.DecodeBase64(text:sub(#prefix + 1))
		local cbor = C_EncodingUtil.DecompressString(packed, Enum.CompressionMethod.Deflate)
		return C_EncodingUtil.DeserializeCBOR(cbor)
	end)
	if not ok or type(payload) ~= "table" or type(payload.log) ~= "table" then return false, "broken" end
	if type(payload.schema) ~= "number" or payload.schema > ns.SCHEMA then return false, "newer" end
	for _, str in ipairs(payload.log) do
		local r = ns.DecodeRow(str)
		if r and not ns.RowTimely(r) then return false, "broken" end
	end
	local horizon = type(payload.baseline) == "table" and payload.baseline.through or nil
	if horizon ~= nil and not (type(horizon) == "number" and timely(horizon, ns.C.row_ahead_floor_s)) then
		return false, "broken"
	end
	if type(db.meta) ~= "table" then db.meta = {} end
	-- the reset first, bound again as /qp reset all binds its new save (ns.ResetSave): its migrations shape it
	local reset = ns.MirrorReset(db, { rows = payload.log })
	if reset then
		ns.BindDB()
		ns.Log("I", "import: a reset at %s, newer than this save, reset it first (the old one is the backup)",
			date("%Y-%m-%d %H:%M:%S", reset))
	end
	-- the history from before the save's latest reset is gone with it: the export's baseline whose horizon is older
	-- (all it folded came before the reset) is left out, as the merge's floor leaves out its rows from before it
	local floor = tonumber(db.meta.resetAt)
	local baseline = type(payload.baseline) == "table" and payload.baseline or nil
	if baseline and floor and (tonumber(baseline.through) or 0) < floor then baseline = nil end
	-- a save without history of its own takes the export's baseline: no move or pump yet (the start's rung rows do
	-- not count)
	local empty = not (type(db.baseline) == "table" and db.baseline.anyMove == true)
	for _, str in ipairs(db.log or {}) do
		local r = ns.DecodeRow(str)
		if r and (MOVE_KIND[r.kind] or r.kind == K.pump or r.kind == K.ding) then empty = false end
	end
	local left = empty and baseline ~= nil
	for _, str in ipairs(payload.log) do
		local r = ns.DecodeRow(str)
		if r and (not floor or r.ts >= floor) then left = true end
	end
	if not left and floor then return false, "old" end -- nothing from after the reset: the save stays as it was
	-- the baseline first: the rows a full ring pushes out fold onto the export's history, never onto one it replaces;
	-- its characters numbered as the merge numbers the export's rows (charMap, which the merge finds the same)
	if empty and baseline then db.baseline = renumbered(baseline, charMap(db, payload.chars)) end
	ns.MergeMirror(db, { rows = payload.log, chars = payload.chars, profiles = payload.profiles,
		profileKeys = payload.profileKeys, through = type(payload.baseline) == "table" and payload.baseline.through or nil },
		floor)
	db.savedAt = ns.now()
	-- rows came in: a helper that has not read the save since is stale (ns.HelperStale, at the next login)
	if type(db.meta) == "table" then db.meta.rowAt = db.savedAt end
	return true, reset
end

-- The pairing block the helper writes on pairing and on every mirror write; `/qp dev pair` stands in for the session.
function ns.Pairing()
	if ns.devPairing then return ns.devPairing end
	local m = QuestPumpMirror
	local p = type(m) == "table" and m.pairing
	return type(p) == "table" and p or nil
end

function ns.IsPaired()
	local p = ns.Pairing()
	return p ~= nil and (p.phone == "ios" or p.phone == "android")
end

-- A paired phone that has not reached the helper for C.phone_quiet_days at this load (David, 2026-10-09): the later of
-- the last time a phone or a watch held the link (`linked_at`, kept to the hour) and the pairing itself, while that is
-- older; nil with no phone or a recent one. The setup screen's phone step is open again while it holds (the app
-- removed, a new phone, an Unpair the computer never heard of). The computer's clock, as ns.HelperState's.
function ns.PhoneQuiet()
	local p = ns.Pairing()
	if not ns.IsPaired() then return nil end
	local last = math.max(tonumber(p.linked_at) or 0, tonumber(p.phone_since) or 0)
	if last > 0 and last < ns.loadedAt - ns.C.phone_quiet_days * 86400 then return last end
	return nil
end

-- The helper as this load found it: "on" (its sign of life, QuestPumpMirrorAlive from the mirror folder's Alive.lua,
-- was written at most C.helper_alive_slack_s before this load: a helper that runs writes it every 10 to 12 s while
-- the game does, so a login, a /reload and another character each read a fresh one), "blind" (running, but it cannot
-- read the game: the macOS Screen Recording grant is missing), "stopped" (running with the grant, but its capture
-- stopped or the strip in view does not read: the block's capture = "stopped"), "off" (the sign of life is older or
-- missing: the helper is not running), "disabled" or "outdated" (the mirror folder is installed but did not load:
-- unticked in the AddOns list, or its Interface line is older than this client's, which the helper rewrites at its
-- next write), or nil (no helper ever wrote a pairing block). Both times are the computer's clock (time(), never the
-- dev clock) and none is the client's: GetSessionTime starts at a point no source names, and measured against it a
-- helper that ran read as "off" (2026-10-01, 2026-10-03). `/qp dev pair` stands in for one that runs.
local MIRROR_ADDON = "QuestPump_Mirror"
local MIRROR_UNLOADED = { DISABLED = "disabled", DEP_DISABLED = "disabled", INTERFACE_VERSION = "outdated" }

function ns.HelperState()
	local p = ns.Pairing()
	if not (p and p.helper) then
		-- a mirror folder that did not load hides the block it holds (GetAddOnInfo's reason, as Figure.Check reads the
		-- figure pack's)
		if QuestPumpMirror ~= nil or QuestPumpMirrorAlive ~= nil or not C_AddOns.DoesAddOnExist(MIRROR_ADDON) then
			return nil
		end
		return MIRROR_UNLOADED[select(5, C_AddOns.GetAddOnInfo(MIRROR_ADDON))]
	end
	local alive = ns.devPairing and ns.loadedAt or QuestPumpMirrorAlive
	if type(alive) ~= "number" or alive < ns.loadedAt - ns.C.helper_alive_slack_s then return "off" end
	if p.sees_game == false then return "blind" end
	return p.capture == "stopped" and "stopped" or "on"
end

-- A pump's heart-rate proof by its time, as the helper wrote it into the mirror (slice 8: `verdict` "proven",
-- "unproven" or "no_hr", with `peak` and `delta`): read at every load, never merged or folded, so it changes no XP. A
-- verdict of another name (a newer helper's) reads as none.
local VERDICTS = { proven = true, unproven = true, no_hr = true }

function ns.Proof(ts)
	local m = QuestPumpMirror
	local proof = type(m) == "table" and type(m.proof) == "table" and m.proof[ts]
	return type(proof) == "table" and VERDICTS[proof.verdict] and proof or nil
end

-- After pairing, a helper that is not running at this load and whose last mirror write is older than the last row the
-- addon wrote: its copy, the phone's, misses rows. The row time (meta.rowAt as loaded), not savedAt, which every login
-- stamps, so a quiet /reload never flags the helper. A helper that runs ("on", "blind" or "stopped": it still reads
-- the files) takes the client's save in at each load (rust-helper protocol-and-mirror.md), the rows it had not written
-- yet among them (its mirror writes trail a row by up to 10 s, and the rows PLAYER_LOGOUT writes reach it only that
-- way). Only a client that read its own save back can tell; on the beta there is nothing to compare.
function ns.HelperStale()
	local p = ns.Pairing()
	return ns.IsPaired() and ns.HelperState() == "off" and ns.loadedStamp ~= nil and ns.loadedRowAt ~= nil
		and type(p.seen_at) == "number" and p.seen_at < ns.loadedRowAt
end

-- Binding --------------------------------------------------------------------------------------------------------

-- The current shape; it also mends a save whose tables went missing. The helper reads the saved file back
-- (questpump-core `mirror::parse_save`: savedAt, log, chars, profiles, profileKeys, baseline, meta.resetAt), so a
-- change to those keys changes that reader too.
local function shape(db)
	db.meta = type(db.meta) == "table" and db.meta or {}
	db.log = type(db.log) == "table" and db.log or {}
	db.baseline = type(db.baseline) == "table" and db.baseline or { moves = {}, owed = {}, pain = {}, moveRows = 0 }
	db.chars = type(db.chars) == "table" and db.chars or {}
	db.profiles = type(db.profiles) == "table" and db.profiles or {}
	db.profileKeys = type(db.profileKeys) == "table" and db.profileKeys or {}
end

-- Every step must also succeed on an empty table, and be idempotent on profiles already in the current format: a
-- reset (ns.ResetSave: /qp reset all, one made elsewhere through ns.MirrorReset or an import) keeps the characters
-- and the profiles and binds again from schema 0, so every step runs once more on them.
ns.migrations = {
	-- 0 → 1: the first shape (slice 1).
	shape,
	-- 1 → 2: the pull ladder and the core hold split in two, the rungs reached keyed ladder × 100 + rung (slice 21,
	-- 2026-10-04); the rows stay as written and the fold reads them split (ns.Translate)
	function(db) ns.SplitBaseline(db.baseline) end,
}

local function runMigrations(db)
	while db.schema < ns.SCHEMA do
		local ok, err = pcall(ns.migrations[db.schema + 1], db)
		if not ok then
			ns.Report(err)
			return false
		end
		db.schema = db.schema + 1
	end
	return true
end

-- Migrations run in order under pcall; a failed one is reported, keeps the data from before the migration in
-- QuestPumpDB.backup (/qp restore-backup), starts a fresh table and tells the player in chat.
local function migrate(db)
	if db.schema >= ns.SCHEMA then return end
	local snap = CopyTable(db)
	snap.backup = nil
	if runMigrations(db) then return end
	wipe(db)
	db.backup = snap
	db.meta = { migrationFailed = snap.schema or 0 }
	db.schema = 0
	runMigrations(db)
	ns.Print("chat.migrationfailed")
end

local readable = ns.Readable

-- The character's body for the sheet's map, from UnitSex (2 male, 3 female; 1, unknown, keeps what is stored).
local BODY = { [2] = "male", [3] = "female" }

function ns.CharFor(db, guid, name, realm, class, body)
	if not guid then return 0 end
	for i, c in ipairs(db.chars) do
		if c.guid == guid then
			c.name, c.realm, c.class, c.body = name or c.name, realm or c.realm, class or c.class, body or c.body
			return i
		end
	end
	if #db.chars >= 255 then return 0 end -- the index is one byte on the strip
	db.chars[#db.chars + 1] = { guid = guid, name = name, realm = realm, class = class, body = body }
	return #db.chars
end

-- The body map's figure for the active character, the sheet's and the apps' (the sheet summary's `body`): its stored
-- sex, or "male" when no login has read one (David, 2026-09-30: the figure looked female on his male character).
function ns.CharBody()
	local c = ns.db and ns.charIndex and ns.db.chars[ns.charIndex]
	return c and c.body or "male"
end

-- The name of the profile a character uses: "Default" unless it chose another (profileKeys holds only the others).
function ns.ProfileName(db, guid)
	return guid and db.profileKeys[guid] or "Default"
end

-- The profile character `char` of `db` uses (its db.chars entry's, by ns.ProfileName), as stored; nil where the db
-- holds none for it (an app's character 0, the desktop app's engine log, which keeps no profiles): Correct a count
-- asks the rung rules again with the set's own character's filters (Progress.lua ns.CorrectionRows; slice 44's
-- review), never the playing character's, whose gear may differ.
function ns.CharProfile(db, char)
	local c = type(db.chars) == "table" and db.chars[char]
	if type(c) ~= "table" or type(db.profiles) ~= "table" then return nil end
	local p = db.profiles[type(db.profileKeys) == "table" and c.guid and db.profileKeys[c.guid] or "Default"]
	return type(p) == "table" and p or nil
end

-- The titles renamed on 2026-10-03 (David: "do your recommendations"), old id to new; their codes stayed.
local RENAMED_TITLES = { trialComplete = "tested", sevenDays = "steadfast", learned = "poised" }

function ns.ProfileFor(db, guid)
	local name = ns.ProfileName(db, guid)
	local p = db.profiles[name]
	if type(p) ~= "table" then
		p = CopyTable(ns.defaults)
		db.profiles[name] = p
	end
	-- A profile from before floor moves were on by default (2026-10-04; no `floorChoice`) held the old default, off,
	-- which was never a choice: it is switched on once, here, on whatever path it came back (the client's own save,
	-- the helper's mirror, an import, the backup). The mark lives in the profile, never in the schema or the meta:
	-- the beta reads no save back, so every load starts at schema 0 with an empty meta, takes its profiles from the
	-- mirror and runs every migration again, and a step there would switch the floor back on at each login.
	if p.floorChoice == nil then p.floor = true end
	-- A profile from before Room to move was on by default and Your setup asked it (2026-10-09; no `setupAnswered`)
	-- held the old default, off: switched on once, the same way, and Your setup comes once at its next login (the fill
	-- below makes `setupAnswered` false), where the player sees it ticked and can untick it.
	if p.setupAnswered == nil then p.space = true end
	for k, v in pairs(ns.defaults) do
		-- a missing key, or a table-typed one that came back as something else (a mirror, an import)
		if p[k] == nil or (type(v) == "table" and type(p[k]) ~= "table") then
			p[k] = type(v) == "table" and CopyTable(v) or v
		end
	end
	-- a title worn under its id from before the renames of 2026-10-03 (its code stayed) is worn under the new one
	p.title = RENAMED_TITLES[p.title] or p.title
	return p
end

-- The fold at binding. It skips a row that does not fold on its own (applyAll); what is left to fail is a log that is
-- no table at all (a read-only save is never shaped), which holds no row to fold: reported, and the state is empty.
local function bindFold(db)
	local ok, err = pcall(ns.Fold, db)
	if ok then return end
	ns.Report(err)
	ns.state = ns.NewState(nil)
end

-- A new start for the whole save at `at` (/qp reset all, Settings.lua; a newer one found, ns.MirrorReset): the
-- save is kept as the backup /qp restore-backup brings back (the one before it goes), and the new save takes over
-- the keys `keep` names. The caller binds it again (ns.BindDB), whose migrations shape it.
function ns.ResetSave(db, at, keep)
	local backup = CopyTable(db)
	backup.backup = nil
	wipe(db)
	db.backup = backup
	for _, key in ipairs(keep or {}) do
		if type(backup[key]) == "table" then db[key] = CopyTable(backup[key]) end
	end
	db.meta = { resetAt = at }
	db.schema = 0
end

-- A /qp reset all is the player's, everywhere (slice 26): the mirror carries the latest reset row of every install,
-- and one newer than this save's own (meta.resetAt) resets the save to it before the merge: another install's, or this
-- install's own whose save never reached the disk (a crash before the logout wrote it). The rows and the baseline
-- from before it go, the save is kept as the backup, the characters and the profiles stay (a reset elsewhere is no
-- reset of this install's settings), and this save's rows from the reset on stay. A save with nothing from before it
-- (the beta, which loads none, at every load) only takes its time as the merge's floor. An import string's rows are
-- read the same way (ns.Import). Returns the reset's time when it reset the save.
function ns.MirrorReset(db, m)
	local at
	for _, str in ipairs(m.rows) do
		local r = ns.DecodeRow(str)
		if r and r.kind == K.reset and (not at or r.ts > at) then at = r.ts end
	end
	if not at or at <= (tonumber(db.meta.resetAt) or 0) then return nil end
	local base = type(db.baseline) == "table" and db.baseline or {}
	local before = (tonumber(base.through) or 0) > 0 or base.anyMove == true
	local after = {}
	for _, str in ipairs(type(db.log) == "table" and db.log or {}) do
		local r = ns.DecodeRow(str)
		if r and r.ts < at then before = true elseif r then after[#after + 1] = str end
	end
	if not before then
		db.meta.resetAt = at
		return nil
	end
	ns.ResetSave(db, at, { "chars", "profiles", "profileKeys" })
	db.log = after
	return at
end

-- At PLAYER_LOGIN, which fires on login and on every /reload: a newer reset, the mirror when it is newer, the
-- migrations, the character read out of combat, then the stamp and the fold. What the mirror did is kept for the
-- login's log line and chat line (ns.mirrorMerge: the rows taken by source, a reset's time).
function ns.BindDB()
	if type(QuestPumpDB) ~= "table" then QuestPumpDB = {} end
	local db = QuestPumpDB
	ns.db = db
	-- the region's weekly reset (the vault, the stats' weekly gain): the next one, rounded to the hour it falls on
	local untilReset = ns.Readable(C_DateAndTime.GetSecondsUntilWeeklyReset())
	-- a fixed phase against the default reset, so a row keeps its week number from one week to the next
	if untilReset then
		local nextReset = math.floor((time() + untilReset) / 3600 + 0.5) * 3600
		ns.weekAnchor = ns.DEFAULT_RESET + (nextReset - ns.DEFAULT_RESET) % (7 * DAY_S)
	end
	local guid = readable(UnitGUID("player"))
	-- A newer addon wrote this save: read it, never write to it, not even its meta.
	ns.readOnly = type(db.schema) == "number" and db.schema > ns.SCHEMA
	if ns.readOnly then
		ns.charIndex = 0
		for i, c in ipairs(type(db.chars) == "table" and db.chars or {}) do
			if type(c) == "table" and c.guid == guid then ns.charIndex = i end
		end
		-- copies, so nothing writes into the save: the character's own profile with the defaults filled in (the Body
		-- switches, the exclusion list and the equipment still filter every pump), and the meta
		local keys = type(db.profileKeys) == "table" and db.profileKeys or {}
		local saved = type(db.profiles) == "table" and db.profiles[ns.ProfileName({ profileKeys = keys }, guid)]
		ns.profile = ns.ProfileFor({ profiles = { Default = type(saved) == "table" and CopyTable(saved) or nil } }, nil)
		ns.meta = type(db.meta) == "table" and CopyTable(db.meta) or {}
		bindFold(db)
		ns.Emit("DB")
		return
	end
	if type(db.meta) ~= "table" then db.meta = {} end
	if type(db.schema) ~= "number" then db.schema = 0 end
	-- The mirror counts as a restore only when it is newer than what the client loaded and than the last /qp reset all,
	-- and brings something back. A mirror written before the reset (the one this login loaded, when the reset binds
	-- again) is the deleted save's; a later one that still holds rows from before the reset (the helper keeps every row
	-- until it sees the reset row) has them left out. On the beta, which loads no save, the helper drops them itself on
	-- the reset row the strip paints (slice 5). A mirror row no clock could have written (ns.RowTimely) is left out, of a
	-- copy: the mirror is the helper's file, read and never written.
	local m = QuestPumpMirror
	local mirrored = type(m) == "table" and type(m.rows) == "table"
	if mirrored then
		local rows = {}
		for _, str in ipairs(m.rows) do
			local r = ns.DecodeRow(str)
			if not r or ns.RowTimely(r) then rows[#rows + 1] = str end
		end
		if #rows < #m.rows then
			ns.Log("W", "mirror: %d rows dated before QuestPump or too far ahead of this clock left out",
				#m.rows - #rows)
			local copy = {}
			for k, v in pairs(m) do copy[k] = v end
			copy.rows = rows
			m = copy
		end
	end
	ns.mirrorMerge = { resetAt = mirrored and ns.MirrorReset(db, m) or nil }
	local resetAt = tonumber(db.meta.resetAt)
	local restore = mirrored and type(m.savedAt) == "number"
		and (ns.loadedStamp == nil or m.savedAt > ns.loadedStamp) and m.savedAt > (resetAt or 0)
		and (#m.rows > 0 or (type(m.profiles) == "table" and next(m.profiles) ~= nil))
	if restore then ns.mirrorMerge.taken = ns.MergeMirror(db, m, resetAt) end
	db.meta.restoredFrom = restore and "mirror" or nil
	if ns.loadedStamp ~= nil then db.meta.readBackSeen = true end -- only the client's own read proves the loader
	migrate(db)
	shape(db)
	ns.meta = db.meta -- the addon reads its meta through ns.meta, the same table while the save is writable
	ns.charIndex = ns.CharFor(db, guid, readable(UnitName("player")), readable(GetRealmName()),
		readable(UnitClassBase("player")), BODY[readable(UnitSex("player"))])
	ns.profile = ns.ProfileFor(db, guid)
	-- the body the apps share comes in whether or not the mirror restores: each field by its own time (ns.TakeBody)
	ns.mirrorMerge.body = type(m) == "table" and ns.TakeBody(ns.profile, m.body) or nil
	db.savedAt = ns.now() -- stamped after the capture at ADDON_LOADED, and on every later change
	bindFold(db)
	ns.Emit("DB")
end

-- Trims the ring once it runs ring_batch rows over (ns.Trim), with the fold's correction map, undone set and loads
-- where `fresh` says ns.state holds them for every row of the log, else decoded from it again.
local function trimDue(fresh)
	if #ns.db.log <= ns.C.ring_rows + ns.C.ring_batch then return end
	local s = fresh and ns.state or {}
	ns.Trim(ns.db, s.corrections, s.undone, s.loads)
end

-- The ring's trim after a batch of rows written `held` (ns.AddRow), once the log is folded again: a correction's rows
-- (Events.lua Lc.WriteCorrection, slice 44's review, 2026-10-18) are written whole before any trim, so none moves the
-- set or the rows its undo rows pair with into the baseline between them (the undo would then pair with nothing).
function ns.TrimDue()
	if ns.db and not ns.readOnly then trimDue(true) end
end

-- Appends one row: it is folded at once and written to the log (never in read-only mode). Returns the row as the fold
-- reads it back. The ring is trimmed once it runs ring_batch rows over, so a row's cost stays independent of the log's
-- length (wow-addon-dev performance.md); never after a row `held` for a batch, whose writer trims once at its end
-- (ns.TrimDue).
function ns.AddRow(r, held)
	r.ts = r.ts or ns.now()
	r.char = r.char or ns.charIndex or 0
	local encoded = ns.EncodeRow(r)
	local row = ns.DecodeRow(encoded)
	if not ns.readOnly then
		local log = ns.db.log
		log[#log + 1] = encoded
		-- a load row never trims: its set row or its correction row follows at once, and that row's trim reads complete
		-- maps (a load row's trim ran before the fold knew it, and a /qp undo's weight could leave with its set); the
		-- maps are not complete after a correction or an undo row, which the incremental fold never takes into them
		if row.kind ~= K.load and not held then
			local undo = (row.kind == K.rung or row.kind == K.mastered) and row.reps == ns.RungCause.undo
			trimDue(row.kind ~= K.correction and not undo)
		end
		ns.db.savedAt = ns.now()
		ns.db.meta.rowAt = ns.db.savedAt
	end
	if row.kind == K.correction then
		if not ns.readOnly then ns.Fold(ns.db) end -- a correction changes what came after the row it names
	else
		-- what the row earned, for the toasts and the pump card: the sheet's gains and the titles it completed
		local s = ns.state
		local titles = ns.TitlesEarned(s)
		s.live = {}
		ns.Apply(s, row)
		local live = s.live
		s.live = nil
		for _, id in ipairs(ns.NewTitles(titles, s)) do live[#live + 1] = { kind = "title", id = id } end
		if #live > 0 then ns.Emit("Sheet", live, row) end
	end
	ns.Emit("Row", row)
	return row
end

-- Stamps a change that is not a row (a setting, the HUD's place) and announces it ("Touch": the strip paints the
-- profile again, Bridge.lua). The profile in use keeps the change's time (`changedAt`), and so does each profile of
-- `profiles` (a list or a map) the change also wrote into: the helper takes a profile from a save, and the save one
-- from the mirror, only when it changed later than the copy held (ns.ProfileNewer; 2026-10-09).
function ns.Touch(profiles)
	if ns.db and not ns.readOnly then
		local now = ns.now()
		ns.db.savedAt = now
		if ns.profile then ns.profile.changedAt = now end
		for _, p in pairs(profiles or {}) do
			if type(p) == "table" then p.changedAt = now end
		end
		ns.Emit("Touch")
	end
end

-- Whether a profile from elsewhere (the mirror's, an import's) takes the place of this save's of its name (`mine`):
-- when the save has none, or it changed later (`changedAt`, ns.Touch's stamp). A stamped one beats one without, and
-- of two without the incoming one wins, as every incoming one did before the stamps (2026-10-09). The helper's take-in
-- of a save reads the same rule (`Session::adopt_profiles`).
function ns.ProfileNewer(theirs, mine)
	if type(mine) ~= "table" then return true end
	local a, b = tonumber(theirs.changedAt), tonumber(mine.changedAt)
	if not b then return true end
	return a ~= nil and a > b
end

-- One body everywhere (docs/plan.md slice 35, "One body on every app device"; David, 2026-10-09: "if I have one it
-- should keep in sync with all"): the Body page's three switches and the moves the Exercises page leaves out or opts
-- in (`exclude` and `include`, one field: `moves`) are the player's on every device, the latest change winning field by
-- field. The profile keeps when each changed here (`bodyAt`, unix seconds), which the helper reads off the profile as
-- the game's change; the mirror's `body` carries each field with the time it last changed anywhere. Since slice 51 the
-- game's setup is the place Home, whose equipment (`gear`) and three switches (`floor`, `space`, `quiet`) are one more
-- field, `home` (Settings.lua stamps it), travelling between the game, the computer and the phones as the body does;
-- the other places stay the phones' and the computer's.
ns.BODY_SWITCHES = { "noJump", "noStrain", "noHeadDown" }
-- Home's switches
local HOME_SWITCHES = { "floor", "space", "quiet" }
-- each switch's name on the Body page, the key after "gear." in the locale
local BODY_NAMES = { noJump = "nojump", noStrain = "nostrain", noHeadDown = "noheaddown" }

-- A Body field of the profile `p` changed at `at` (now by default): the player's change here, or a body field taken in.
function ns.BodyStamp(p, field, at)
	if type(p.bodyAt) ~= "table" then p.bodyAt = {} end
	p.bodyAt[field] = at or ns.now()
end

-- A set from a mirror's list: the move ids (positive whole numbers) or the group names (strings) it holds.
local function setOf(list, ids)
	local out = {}
	for _, v in ipairs(type(list) == "table" and list or {}) do
		if ids and type(v) == "number" and v > 0 and v == math.floor(v) then
			out[v] = true
		elseif not ids and type(v) == "string" then
			out[v] = true
		end
	end
	return out
end

-- Two sets alike.
local function sameSet(a, b)
	for k in pairs(a) do
		if not b[k] then return false end
	end
	for k, v in pairs(b) do
		if v and not a[k] then return false end
	end
	return true
end

-- The mirror's body (`body`: noJump, noStrain and noHeadDown as { value, at }, moves as { at, ids, groups, include },
-- since slice 51 home as { at, gear (the equipment's keys; one not in ns.Gear is dropped), floor, space, quiet })
-- into the profile in use `p`: each field that changed later than the profile's own change of it (`bodyAt`; none
-- counts as before every time) takes the body's value and time, so a switch set on a phone, a watch or the computer
-- reaches the game's pumps at its next load, and one set here later than that stays. The profile counts as changed
-- when the body did (`changedAt` raised to the latest field's own time, never the load's clock: on the beta, which
-- loads a mirror one /reload behind, a stale copy of the profile taking the body again would otherwise outrank a
-- setting changed here since, ns.ProfileNewer and the helper's `profile_newer`), and the `bodyAt` stamps carry the
	-- fields back to the helper. Returns the fields whose value came (`field`, a switch's key with its new state `on`,
	-- "moves", the Exercises page's list, or "home", Home's equipment and switches), for the login's chat line
	-- (bodyWords); nil when none did. It reads no text, so the desktop app's engine takes the body the same way into its
	-- fold of the game's save (tools/apppump.lua gameProfile).
function ns.TakeBody(p, body)
	if type(body) ~= "table" then return nil end
	local own = type(p.bodyAt) == "table" and p.bodyAt or {}
	local function later(f, field)
		return type(f) == "table" and type(f.at) == "number" and f.at > (tonumber(own[field]) or 0)
	end
	local came, latest = {}, nil
	for _, key in ipairs(ns.BODY_SWITCHES) do
		local f = body[key]
		if later(f, key) and type(f.value) == "boolean" then
			if p[key] ~= f.value then came[#came + 1] = { field = key, on = f.value } end
			p[key] = f.value
			ns.BodyStamp(p, key, f.at)
			latest = math.max(latest or 0, f.at)
		end
	end
	local moves = body.moves
	if later(moves, "moves") then
		local exclude, include = setOf(moves.ids, true), setOf(moves.include, true)
		for g in pairs(setOf(moves.groups, false)) do exclude[g] = true end
		if not (sameSet(exclude, p.exclude or {}) and sameSet(include, p.include or {})) then
			came[#came + 1] = { field = "moves" }
		end
		p.exclude, p.include = exclude, include
		ns.BodyStamp(p, "moves", moves.at)
		latest = math.max(latest or 0, moves.at)
	end
	local home = body.home
	if later(home, "home") and type(home.gear) == "table" and type(home.floor) == "boolean"
		and type(home.space) == "boolean" and type(home.quiet) == "boolean" then
		local gear, listed = {}, setOf(home.gear, false)
		for _, key in ipairs(ns.Gear) do gear[key] = listed[key] end
		local same = sameSet(gear, p.gear or {})
		for _, key in ipairs(HOME_SWITCHES) do same = same and p[key] == home[key] end
		if not same then came[#came + 1] = { field = "home" } end
		p.gear = gear
		for _, key in ipairs(HOME_SWITCHES) do p[key] = home[key] end
		ns.BodyStamp(p, "home", home.at)
		latest = math.max(latest or 0, home.at)
	end
	if latest then p.changedAt = math.max(tonumber(p.changedAt) or 0, latest) end
	return #came > 0 and came or nil
end

-- The login's chat line's words for the body fields that came (ns.TakeBody): a switch's name on the Body page with
-- its new state, the Exercises page's list, Home's equipment and switches.
local function bodyWords(came)
	local words = {}
	for i, c in ipairs(came) do
		words[i] = c.field == "moves" and ns.L["status.body.moves"]
			or c.field == "home" and ns.L["status.body.home"]
			or ns.L["gear." .. BODY_NAMES[c.field]] .. " " .. ns.L[c.on and "gear.on" or "gear.off"]
	end
	return table.concat(words, ", ")
end

-- Writes one key of the save's meta and stamps the save; the meta is no part of the profile, so nothing repaints.
function ns.SetMeta(key, value)
	if ns.db and not ns.readOnly then
		ns.meta[key] = value
		ns.db.savedAt = ns.now()
	end
end

ns.RegisterEvent("ADDON_LOADED", function(name)
	if name ~= ADDON then return end
	-- When the client read this load's files, the mirror folder's right after this addon's, on the computer's clock
	-- as the helper's sign of life is (ns.HelperState).
	ns.loadedAt = time()
	-- The one read of what the client loaded, before anything writes (never at file scope): nil on the beta.
	local db = QuestPumpDB
	ns.loadedStamp = type(db) == "table" and type(db.savedAt) == "number" and db.savedAt or nil
	local meta = type(db) == "table" and type(db.meta) == "table" and db.meta or {}
	ns.loadedRowAt = type(meta.rowAt) == "number" and meta.rowAt or nil
	if type(QuestPumpDB) ~= "table" then QuestPumpDB = {} end
end)

ns.RegisterEvent("PLAYER_LOGIN", ns.BindDB)

-- PLAYER_LOGIN has already stamped savedAt, so the beta bug shows here as a reload with nothing loaded. (A first load
-- through a UI reload, such as enabling the addon in game, reads the same; there is no data to lose then.) The chat
-- line says only that the helper's copy brought rows back, and never while setup blocks: a load with an empty log has
-- nothing to report, as at the Check again that follows a first pairing (and on the beta without the helper every
-- load is empty, so its loss is the HUD's line and the setup screen's).
ns.RegisterEvent("PLAYER_ENTERING_WORLD", function(_, isReloadingUi)
	-- a reset newer than this save that reset it at the bind (ns.MirrorReset): said once
	local merge = ns.mirrorMerge
	if merge and merge.resetAt and not merge.told then
		merge.told = true
		ns.Print("status.reset.elsewhere")
	end
	-- Body switches, the Exercises page's list or Home's setup that came from the other devices at the bind
	-- (ns.TakeBody): said once
	if merge and merge.body and not merge.bodyTold then
		merge.bodyTold = true
		ns.Print("status.body.synced", bodyWords(merge.body))
	end
	if isReloadingUi and ns.loadedStamp == nil and not ns.svBroken then
		ns.svBroken = true
		if ns.meta.restoredFrom == "mirror" and #ns.db.log > 0 and not ns.Setup.Blocks() then
			ns.Print("status.beta.restored")
		end
		ns.Emit("Saving")
	end
end)

ns.RegisterEvent("SAVED_VARIABLES_TOO_LARGE", function(name)
	if name == ADDON and not ns.tooLargeWarned then
		ns.tooLargeWarned = true
		ns.Chat("warning", ns.L["chat.toolarge"])
	end
end)

-- The client writes SavedVariables on logout, /reload, quit and disconnect; an addon cannot force a write.
ns.AddCommand("save", function()
	if InCombatLockdown() then
		ns.Print("chat.combat")
		return
	end
	C_UI.Reload()
end, "cmd.save.help")

-- Swaps the backup in (a /qp reset all undone, the save a failed migration kept); the save it replaces becomes the
-- backup, so a restore is undone the same way. It brings back this game's save only: the helper's floor keeps the
-- rows from before a reset out of the mirror, so the app and the other installs keep the reset (the popup says so,
-- status.restore.here). The pump, the queue and the last typed count belong to the save that
-- goes, and end unwritten.
function ns.RestoreBackup()
	if ns.readOnly then return ns.Print("import.readonly") end
	local db = ns.db
	if not (db and type(db.backup) == "table") then
		ns.Print("cmd.restore.none")
		return
	end
	ns.Lifecycle.Drop()
	local backup, current = db.backup, CopyTable(db)
	current.backup = nil
	wipe(db)
	for k, v in pairs(backup) do db[k] = v end
	db.backup = current
	if type(db.meta) ~= "table" then db.meta = {} end
	db.meta.backupRestored = ns.now()
	-- the latest reset the save that goes had taken (its own /qp reset all, or one made elsewhere: ns.MirrorReset) is
	-- taken here too, so the mirror's reset row never resets the restored save again at the next login
	local heard = type(current.meta) == "table" and tonumber(current.meta.resetAt)
	if heard and heard > (tonumber(db.meta.resetAt) or 0) then db.meta.resetAt = heard end
	ns.meta = db.meta
	-- a backup of this schema (a /qp reset all undone) folds at once; an older one waits for the next login, whose
	-- migrations it failed before, so a still-broken step never swallows it again
	if db.schema == ns.SCHEMA then
		ns.BindDB()
		ns.Emit("Trial")
	end
	ns.Print("cmd.restore.done")
end

-- /qp restore-backup asks first (Settings.lua's popup), as the Data page's button does.
ns.AddCommand("restore-backup", function()
	if ns.readOnly then return ns.Print("import.readonly") end
	if not (ns.db and type(ns.db.backup) == "table") then return ns.Print("cmd.restore.none") end
	ns.Settings.ConfirmReset("restore")
end, "cmd.restore.help")
