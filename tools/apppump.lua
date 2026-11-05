-- An app pump on the addon's own engine (docs/plan.md slices 17, 31 and 35): the port of what the phones and the
-- watches run (QuestPumpCore's AppPump.swift and AgnosticEngine.swift, the Android core's AppPump.kt and
-- AgnosticEngine.kt), on the addon's own rules, so the desktop app (helper/crates/questpump-engine, which embeds this
-- file with tools/host.lua and its H.FILES) and the fixture writer (tools/gen-engine.lua,
-- contract/fixtures/engine.json) run one driver: the start rows, a pump composed for a family and a size (a match pump
-- with its match's facts, slice 48, and the unfinished moves riding it, slice 61) or the Quick pump, Swap and its way
-- back, a rider's Skip, the Ready view's Pain (composed again
-- before any result, else the rest kept where the new flags allow it), a move's result as the game judges it with the
-- rung rows the rules add, the felt-unwell stop, the check-in, the pain and focus rows, the one progression folded from
-- the app's own rows and the game's kept ones (the game's rows marking the app's own sheet, never paying it), the app's
-- own sheet summary (and, for the desktop app, the game's own as its save folds at its next load, while no sheet the
-- strip brought is current), and what a pump's screens read beside it (the Ready view's rewards and callout, the titles
-- earned, a first record's next step); and for the desktop app a catch-up's moves dosed, swapped and their counts
-- judged as the game's own catch-up doses, swaps and judges them (A.catchup, Engine:judge), whose rows the helper
-- writes.
-- An app pump names a family and a size, never a quest. One a match sizes owes its moves left at Bank it (slice 58,
-- written by the helper) and carries the unfinished moves its device may pay as the game's quest pump carries them
-- (slice 61: A.compose's `rows` and `queue`), judged here and paid by the helper as a catch-up's; every other pump pays
-- and owes nothing. Indexes count from 1, as Lua's (the engine's slot `i` writes slot i - 1); the crate converts at
-- its boundary.
-- Usage, under tools/host.lua's environment after H.load: local A = chunk()(ns, { array = mark }), where `mark(t)`
-- returns the list `t` marked to encode as a JSON array even when empty (tools/json.lua's json.array; the crate's
-- mlua array metatable).

return function(ns, opts)
	local C, E = ns.C, ns.Enum
	local K, O = E.kind, E.outcome
	local array = opts.array

	-- Outside WoW the week is Monday's game day to Sunday's, as the phones count it (Stats.lua ns.WeekOf).
	ns.weekByDay = true

	local A = {}

	-- Profiles ---------------------------------------------------------------------------------------------------------

	-- A set from a list (the profile keeps its gear, exclusions, opt-in moves and favourites as sets).
	local function set(list)
		local out = {}
		for _, x in ipairs(list or {}) do out[x] = true end
		return out
	end

	-- A set's keys, sorted (numbers or strings alike).
	local function keys(t, want)
		local out = {}
		for k, v in pairs(t or {}) do
			if v and (not want or type(k) == want) then out[#out + 1] = k end
		end
		table.sort(out)
		return out
	end

	-- The engine's profile from the phones' EngineProfile (its JSON keys: gear, floor, space, quiet, noJump, noStrain,
	-- noHeadDown, experience and exclude { ids, groups }, with favourites, include, title and matchPace where set) over
	-- the addon's defaults: the pace (Moves per quest) is Balanced, as every app pump's that is no match pump; the match
	-- pace (`matchPace`, Moves per match: the body's match_pace, slice 48) sizes a pump composed with a match's facts
	-- (Engine:compose), light, balanced or extreme, any other value or none Balanced (ns.Pace).
	function A.profile(p)
		local profile = CopyTable(ns.defaults)
		for _, key in ipairs({ "floor", "space", "quiet", "noJump", "noStrain", "noHeadDown" }) do
			assert(type(p[key]) == "boolean", key)
			profile[key] = p[key]
		end
		profile.experience = assert(p.experience, "experience")
		profile.gear = set(assert(p.gear, "gear"))
		local exclude = set(assert(assert(p.exclude, "exclude").ids, "exclude.ids"))
		for _, group in ipairs(assert(p.exclude.groups, "exclude.groups")) do exclude[group] = true end
		profile.exclude, profile.include, profile.favourites = exclude, set(p.include), set(p.favourites)
		profile.title, profile.pace, profile.matchPace = p.title, "balanced", ns.Pace(p.matchPace)
		return profile
	end

	-- The pace a pump with `match`'s facts composes at: Light, one move at every size, when the match's own minutes are
	-- under match_short_min (2026-11-04, Rapha: "2 min to 5 be 1 move pump"), else Moves per match; nil (the pump's
	-- Balanced) for a pump no match sizes. The phones' AppPump.pace is its twin.
	function A.matchPace(profile, match)
		if not match then return nil end
		if match.minutes and match.minutes < C.match_short_min then return "light" end
		return profile.matchPace
	end

	-- The profile as the phones encode it (EngineProfile): favourites only where it marks some, the title only when
	-- one is chosen and the match pace only when it is not Balanced.
	function A.profileJSON(profile)
		local favourites = keys(profile.favourites)
		return { gear = array(keys(profile.gear)), floor = profile.floor, space = profile.space, quiet = profile.quiet,
			noJump = profile.noJump, noStrain = profile.noStrain, noHeadDown = profile.noHeadDown,
			experience = profile.experience,
			exclude = { ids = array(keys(profile.exclude, "number")), groups = array(keys(profile.exclude, "string")) },
			include = array(keys(profile.include)), favourites = #favourites > 0 and array(favourites) or nil,
			title = profile.title, matchPace = profile.matchPace ~= "balanced" and profile.matchPace or nil }
	end

	-- Slots ------------------------------------------------------------------------------------------------------------

	-- A slot as the contract carries it (the phones' EngineSlot): its fields, and its flags present only when true. A
	-- loaded move's planned weight in tenths of a kg (`load`) and whether it is a step up (`loadUp`, false when not;
	-- both absent before a set with a weight: Engine.lua dose, 2026-10-09), and on a step up the working weight it came
	-- from (`loadFrom`, 2026-10-10); an easier version taken for a ladder's attempt at its rung, the ladder and the rung
	-- its result steps down from (`down`, `downFrom`: Engine.lua easierSlot, 2026-10-13); an all-out window that closes
	-- a pump or a started one (`closing`, whose Swap offers the closing window's forms; 2026-10-15); a counted set with
	-- no goal (`counted`, its countGoal and countKind until 2026-10-16) and the steady move after which the pump asks
	-- "Did you keep that pace to the end?" (`pace`, slice 42: Engine.lua paceSlot), with the full move's seconds Keep
	-- going runs a steady move on to (`keep`, slice 52: Engine.lua keepSlot); since slice 42's fix round
	-- (2026-10-17, Engine.lua markSlot and checkSlot) a pose's `goalAsk` (the answer its goal pose waits for, beside
	-- `ask`, the least answer that counts the day) and a stretch slot's `check` (the waiting stretch check its pose runs,
	-- an entry as contract/library.json carries it); since slice 48's double-check a match pump's warm move (`warm`:
	-- Engine.lua dose, its easy dose); since slice 55 a Long Haul's steady move (`haul`: Engine.lua chosenSlot, its
	-- seconds its length, its row's reps those seconds).
	local SLOT_FIELDS = { "move", "kind", "ladder", "rung", "tier", "target", "goal", "hi", "cap", "seconds", "ask",
		"goalAsk", "load", "loadUp", "loadFrom", "down", "downFrom", "keep" }
	local SLOT_FLAGS = { "maxSet", "placing", "allout", "closing", "trial", "anchor", "easier", "lower", "mark",
		"counted", "pace", "warm", "haul" }

	-- An entry of a family's `opens`, a rung's `needs` or a move's `checks` (not an `any` group) as contract/library.json
	-- carries it: { ladder, step, count?, move?, mark? }.
	local function entryJSON(o)
		return { ladder = o.ladder, step = o.step, count = o.count, move = o.move, mark = o.mark }
	end

	-- Two entries alike (both none too).
	local function sameEntry(a, b)
		if a == nil or b == nil then return a == b end
		return a.ladder == b.ladder and a.step == b.step and a.count == b.count and a.move == b.move and a.mark == b.mark
	end

	function A.slotJSON(slot)
		local out = {}
		for _, k in ipairs(SLOT_FIELDS) do out[k] = slot[k] end
		for _, k in ipairs(SLOT_FLAGS) do out[k] = slot[k] == true or nil end -- false and nil read alike
		out.check = slot.check and entryJSON(slot.check) or nil
		return out
	end

	function A.slotsJSON(slots)
		local out = {}
		for i, slot in ipairs(slots) do out[i] = A.slotJSON(slot) end
		return array(out)
	end

	-- Swap's options for a slot on its Ready view (ns.SwapOptions, or after a swap the way back, Pump:swapOptions), each
	-- a slot or absent.
	function A.optionsJSON(options)
		return { another = options.another and A.slotJSON(options.another),
			easier = options.easier and A.slotJSON(options.easier),
			backup = options.backup and A.slotJSON(options.backup), back = options.back and A.slotJSON(options.back) }
	end

	-- The engine of one player ----------------------------------------------------------------------------------------

	-- The app pump a quest of `tier` and `family` stands for: no quest, nothing owed, no stretch; its active seconds a
	-- match's `minutes` (slice 57: the match's own, the minutes sat where none was measured; slice 48: its steady cardio
	-- moves share them, as a quest's), none for any other pump.
	local function quest(tier, family, minutes)
		return { questID = 0, tier = tier, family = family,
			share = { active = (minutes or 0) * 60, foot = 0, mount = 0, swim = 0 } }
	end

	-- `flags` with every bit of `bits` set (each bit once).
	local function withBits(flags, bits)
		local p = 1
		while bits >= p do
			if math.floor(bits / p) % 2 == 1 and math.floor(flags / p) % 2 == 0 then flags = flags + p end
			p = p * 2
		end
		return flags
	end

	local QUEUE = { [O.owed] = true, [O.banked] = true, [O.paid] = true }
	local MOVE = { [K.rep] = true, [K.hold] = true, [K.cardio] = true, [K.flow] = true }

	local Engine = {}
	Engine.__index = Engine

	-- One player's engine: `profile` (A.profile's), `o.stamp` the device bits every row it writes carries (0 for a
	-- phone's own rows, which the phones stamp only when they send them; the desktop's app and computer bits). Its
	-- state is the fold of the rows so far (E.state); E.own and E.game the app's own rows and the game's rows it folded
	-- (E:fold, E:merge) and wrote (E:write), whole. Beside them the fold keeps what a later merge folds on from (E:fold):
	-- the baseline (`base`, the rows past the ring, ns.Trim's), the ring by side (`ringSave`: the shown focus and the
	-- game's kept rows; `ringOwn`: the app's own), each side's running time (`saveAt`, `ownAt`: MergeMirror sorts a row
	-- at the latest time its side reached), the sides' rows as sets (`gameRows`, `ownRows`), the latest reset, the
	-- latest time of any row (`newest`) and whether no row stands on both sides (`exact`); the times of the game's owed,
	-- banked and paid move rows from the reset on (`queued`: no fold reads them, but the game's ring holds them, so
	-- Correct a count's reach counts them, E:correctable); and, once E:gameSummary asked for it, the game's own sheet
	-- as its save folds (`gameSheet`, in the weeks of `gameAnchor`, from the save `gameSave`), dropped whenever rows come
	-- or the engine writes one.
	function A.new(profile, o)
		return setmetatable({ profile = profile, stamp = o and o.stamp or 0, state = ns.NewState(nil),
			save = ns.NewState(nil), own = {}, game = {}, focus = nil, newest = 0, ringSave = {}, ringOwn = {},
			saveAt = 0, ownAt = 0, gameRows = {}, ownRows = {}, exact = true, queued = {} }, Engine)
	end

	-- Whether a row is one of character creation's start rungs (ns.StartRows' rung rows).
	local function startRung(r)
		return r.kind == K.rung and r.reps == ns.RungCause.start
	end

	-- A fold's apply, as DB.lua's applyAll applies each row: a row that does not fold (a newer addon's row, a bug in a
	-- rule) is skipped, as a fold of every row skips it, so the rows after it still fold; `report()` hands the first
	-- such error to ns.Report once the loop is done. Returns apply(s, r, part) and report().
	local function applier()
		local failed
		local function apply(s, r, part)
			local ok, err = pcall(ns.ApplyRead, s, r, part)
			if not ok then failed = failed or err end
		end
		local function report()
			if failed then ns.Report(failed) end
		end
		return apply, report
	end

	-- A row of the app's own taken into the ring: its side's running time, its set, and whether the game's side holds
	-- it too.
	local function takeOwn(self, str, ts)
		self.ringOwn[#self.ringOwn + 1] = str
		self.ownAt = math.max(self.ownAt, ts)
		self.ownRows[str] = true
		if self.gameRows[str] then self.exact = false end
	end

	-- A row with every field of the grammar, 0 where it has none.
	local function filled(r)
		for _, field in ipairs(ns.FIELDS) do r[field] = r[field] or 0 end
		return r
	end

	-- A row as the app writes it: character 0, every field, the device bits.
	local function stamped(self, r)
		r.char = 0
		filled(r)
		r.flags = withBits(r.flags, self.stamp)
		return r
	end

	-- A row as written (stamped), folded at once into the app's own sheet, the same table (the fold marks it for
	-- ns.Progression), and the game's own sheet folded again at its next ask (the save merges the app's rows). Returns
	-- its text, which E.own keeps.
	function Engine:write(r)
		stamped(self, r)
		local str = ns.EncodeRow(r)
		ns.ApplyRead(self.state, r, "own")
		self.own[#self.own + 1] = str
		self.gameSheet = nil
		takeOwn(self, str, r.ts)
		self.newest = math.max(self.newest, r.ts)
		return str
	end

	-- Writes each row of a list, in order; returns their texts.
	local function writeAll(self, list)
		local out = {}
		for _, r in ipairs(list) do out[#out + 1] = self:write(r) end
		return array(out)
	end

	-- Whether a state's ladders have a rung yet (the cardio ladder's or another's).
	local function rungsIn(state)
		if state.cardio.rung then return true end
		for _, lad in pairs(state.ladders) do
			if lad.rung then return true end
		end
		return false
	end

	-- Whether a ladder has a rung yet, the app's or the game's (the start rows come once, before the first pump).
	function Engine:hasRungs()
		return rungsIn(self.state)
	end

	-- Character creation's start rungs (ns.StartRows) at `at`.
	local function startList(self, at)
		return ns.StartRows(self.profile.experience, ns.FilterContext(self.profile, self.state), at)
	end

	-- Character creation's start rungs, written.
	function Engine:startRows(at)
		return writeAll(self, startList(self, at))
	end

	-- The state a Start and the Book read at `at` (slice 36): the app's own, and before the first pump, whose compose
	-- writes the start rungs first (A.compose), a copy with them folded in, so a Start's label, the pump it composes and
	-- the tree read the same rungs.
	function Engine:startState(at)
		if self:hasRungs() then return self.state end
		local s = CopyTable(self.state)
		for _, r in ipairs(startList(self, at)) do ns.ApplyRead(s, stamped(self, r), "own") end
		return s
	end

	-- Pain flags set or cleared (Events.lua flagPain): a row per joint whose flag differs, by ascending id.
	function Engine:painRows(joints, on, at)
		local rows = {}
		for _, id in ipairs(keys(set(joints))) do
			if (self.state.pain[id] ~= nil) ~= on then
				rows[#rows + 1] = self:write({ ts = at, kind = K.pain, move = id, reps = on and 1 or 0 })
			end
		end
		return array(rows)
	end

	-- The check-in's answer (ns.Enum.checkin), the focus chosen (ns.Enum.focus), written.
	function Engine:checkinRow(answer, at)
		return self:write({ ts = at, kind = K.checkin, reps = answer })
	end

	function Engine:focusRow(code, at)
		return self:write({ ts = at, kind = K.focus, reps = code })
	end

	-- A clear row (ns.Enum.clear's `scope`, "move" or "ladder"), written: a move's record cleared (`id` a move of the
	-- library), or a strength ladder started over (the page offers it on an open one; a pose family keeps no record);
	-- anything else is refused. The fold takes it from there (DB.lua applyRow), here and in every fold that reads it.
	function Engine:clearRow(scope, id, at)
		local code = assert(E.clear[scope], "clearRow: the scope is move or ladder")
		if scope == "move" then
			assert(ns.Library[id], "clearRow: no such move")
			return self:write({ ts = at, kind = K.clear, reps = code, move = id })
		end
		assert(ns.STRENGTH[id], "clearRow: no strength ladder")
		return self:write({ ts = at, kind = K.clear, reps = code, ladder = id })
	end

	-- A pump stopped for feeling unwell while no timed move runs: banked with the unwell reason and nothing in it, so it
	-- owes nothing and locks every pump until the next game day (ns.LockedToday).
	function Engine:stopRow(tier, at)
		return self:write({ ts = at, kind = K.pump, tier = tier, reps = 0, outcome = O.banked, flags = E.bank.unwell })
	end

	-- A pump's offered row, written when it starts (before its moves' rows).
	function Engine:pumpRow(slots, tier, at)
		return self:write({ ts = at, kind = K.pump, outcome = O.offered, tier = tier, reps = #slots, seconds = 1 })
	end

	-- The slots the engine composes for a family (nil: the Quick pump's, no quest's fit) and a size at `at`, written
	-- nowhere: an app pump's roll counts every source's pumps of the day (ns.Compose's app); with `chosen` ({ move,
	-- allout, rung }, slice 36) the one move the player started from a step's card or a move's window. With `match` (a
	-- pump after a game's match, slices 48 and 57, docs/computer-games.md §3 and §5: { minutes, warm, arms }, warm and
	-- arms present only when true) it is sized at the match pace (Moves per match, profile.matchPace) instead of
	-- Balanced, or at Light, one move, when its minutes are under match_short_min (A.matchPace; 2026-11-04), its
	-- cardio moves share the match's minutes as their active minutes, with `warm` (the sitting clock held
	-- match_sat_min as the offer was made) it opens on a warm move (Engine.lua warmPick), and with `arms` it keeps the
	-- arms rest (ns.ArmsRest, no all-out window, an owed one neither). With `queue` (slice 61: the game's owed rows by
	-- their keys, ns.OwedKey, as the game's state holds its queue, which the app's fold leaves out) it is composed on a
	-- state that holds it, as the game composes a quest's pump (Events.lua compose: no fresh pick takes a queued move,
	-- one owed this game day or the one before counts in its block's share, one on a ladder or a pose family is its
	-- turn), and `owed` (the rows of it the device may pay, oldest first, as the game's availableOwed) ride it where
	-- they fit (Engine.lua pickRiders), dosed and placed as the game's riders. With `expedition` (slice 51: { stage,
	-- regions }) an expedition's round, whose stage sets its tier and its kind of moves (ns.Compose), `family` and `tier`
	-- aside.
	function Engine:compose(family, tier, at, chosen, match, owed, queue, expedition)
		local minutes = match and match.minutes
		local state = queue and setmetatable({ owed = queue }, { __index = self.state }) or self.state
		return ns.Compose({ kind = "quest", quests = { quest(tier, family, minutes) }, owed = owed, state = state,
			profile = self.profile, now = at, app = true, chosen = chosen, pace = A.matchPace(self.profile, match),
			warm = match ~= nil and match.warm == true or nil, arms = match and match.arms == true or nil,
			expedition = expedition })
	end

	-- A typed count and its weight (tenths of a kg, nil none) against the game's limits (ns.Validate): "max" refused,
	-- "heavy" (the weight), "really" or "low" asked once, nil.
	function Engine:validate(slot, value, load)
		return ns.Validate(self.state, slot, value, load)
	end

	-- Whether a hold set of `value` seconds at `at` asks "Could you have held longer?" (ns.HoldAsk, slice 52): its
	-- answer (ns.Enum.longer) goes in the result's `longer`.
	function Engine:holdAsk(slot, value, at)
		return ns.HoldAsk(self.state, slot, value, at)
	end

	-- A move's result as the game judges it (the phones' AgnosticEngine.finish; Events.lua Lc.WriteResult): `result` =
	-- { value, elapsed, stopped, couldNot, confirmed, reason, mark, answer, pace, longer, load }. A counted set that
	-- ran its time is judged on its count with no goal (ns.Judge), none typed the set done with no count, and one saved
	-- with any answer to "All out and clean?" but Yes, to the end (`answer`, ns.Enum.answer: Form slipped since
	-- 2026-10-16, Not all out since slice 53; 0 or nil Yes) done whatever its count (Events.lua Lc.Counted); a timed
	-- move done or stopped with its seconds;
	-- "Could not do one" the stored 0 (ns.CouldNot); a typed count judged (ns.Judge) at its weight (`load`, tenths of a
	-- kg; nil or 0 none). Its row (ns.ResultRow, at `at`) carries the confirmed bit, `reason` (the felt-unwell stop of
	-- a running timed move, ns.Enum.stopped.unwell) and the player's answer: a pose's `mark` (its answer to "How close
	-- to the goal pose?", ns.Enum.mark, on a slot that asks it, `ask`), a counted set's answer (ns.Enum.answer),
	-- a steady move's `pace` (ns.Enum.pace, on a slot that asks it, `pace`, or Keep going's answer, slice 52: Yes, easily
	-- at the full move, Yes, just where Done ended it past the plan, with the seconds done its `value`) and a hold's
	-- `longer` (ns.Enum.longer, where its set asks it, Engine:holdAsk; slice 52). Returns the row and, on a rep set that
	-- counted one or more, the weight its load row (ns.LoadRow) carries before it, at the same time.
	local function judged(self, slot, index, result, at)
		local value, outcome = result.value
		local load = slot.kind == K.rep and not result.couldNot and result.load and result.load > 0 and result.load or nil
		local answer = result.mark
		if slot.counted and not result.stopped then
			if not (value and value > 0) then value = nil end
			answer = result.answer ~= 0 and result.answer or nil
			outcome, value = ns.Judge(self.state, slot, value, result.elapsed, nil, at, answer ~= nil)
		elseif slot.kind == K.cardio and not slot.allout then
			outcome = result.stopped and O.stopped or O.done
			answer = not result.stopped and result.pace or nil
		elseif slot.kind == K.cardio or slot.kind == K.flow then
			outcome = result.stopped and O.stopped or O.done
		elseif result.couldNot then
			outcome, value, answer = ns.CouldNot(self.state, slot), 0, nil
		else
			outcome, value = ns.Judge(self.state, slot, value, result.elapsed, load, at)
			answer = slot.kind == K.hold and result.longer or nil
		end
		local row = ns.ResultRow(slot, index, outcome, value, result.elapsed, result.confirmed, result.reason, answer)
		row.ts = at
		return row, (value or 0) > 0 and load or nil
	end

	-- A move's result (judged), written: its load row, its row, then the rung rows the rules add (ns.Progression) and
	-- an easier pick's step down after them (ns.EasierDown), as Events.lua Lc.WriteResult writes them. Returns the rows
	-- as written, as Lc.WriteResult does the cardio day the set brought (ns.CardioDay, read before the rung rows) or
	-- nil, the week's embers where the row lit the game day's last ember (ns.Embers: today's reached embers_day,
	-- Stats.lua lightEmber's "embers" event, the game's "Cardio done today" toast; slice 55), else nil, and whether the
	-- row lights an ember (ns.LightsEmber against the cardio step before it; the Result's Cardio today, 2026-10-29).
	function Engine:finish(slot, index, result, at)
		local row, load = judged(self, slot, index, result, at)
		local lights = ns.LightsEmber(row, ns.Rung(self.state, ns.CARDIO))
		local rows = {}
		local lit = ns.Embers(self.state, at)
		if load then rows[1] = self:write(ns.LoadRow(row, load)) end
		rows[#rows + 1] = self:write(row)
		local today, most, week = ns.Embers(self.state, at)
		local kindled = lit < most and today >= most and week or nil
		local day = ns.CardioDay(self.state, row)
		local after = ns.Progression(self.state, row, ns.FilterContext(self.profile, self.state))
		for _, r in ipairs(after) do
			r.ts = at
			rows[#rows + 1] = self:write(r)
		end
		for _, r in ipairs((ns.EasierDown(self.state, slot, row, after))) do rows[#rows + 1] = self:write(r) end
		return array(rows), day, kindled, lights
	end

	-- Whether a judged row is a catch-up's set (contract/companion-api.yaml `WristEvent`'s `set`): a typed count (DB.lua's
	-- ns.TYPED) and, since slice 42's fix round (2026-10-17), a counted window that ran its time saved Yes, to the end
	-- with no count or saved Form slipped or Not all out (done, the counted bit and its answer: its clean day counts or
	-- none does, as
	-- the game's own pump writes it) and a steady move whose pace the pump asked, answered, or that Keep going ran past
	-- its plan (done, its answer's bits: the steady ledger's, a full move among them, slice 52); a move done any other
	-- way pays done with no set.
	local function isSet(slot, row, result)
		if ns.TYPED[row.outcome] then return true end
		if row.outcome ~= O.done or result.stopped then return false end
		return slot.counted == true or (slot.pace == true or slot.keep ~= nil) and result.pace ~= nil
	end

	-- Whether a move's result lights an ember (ns.LightsEmber against the cardio step now), written nowhere: a rider's
	-- move (a catch-up's, a match pump's), whose rows the game folds at its next load, for its Result's Cardio today
	-- (2026-10-29), as the row its payment writes reads. A set (isSet) as the judge writes it (Engine:judge); any other
	-- result of a move that pays the owed row `owed`, stopped short or not, as the helper's Session::catch_up writes a
	-- move paid with no set: done in the seconds it ran, as the move run (`slot`'s move, kind, ladder and rung; the owed
	-- row's for a move with no slot, which runs by its time), a window's reps 1 (the owed row's), none of the owed row's
	-- flags. A move of the pump's own (`owed` nil) as its judged row; false with neither a slot nor an owed row.
	function Engine:lights(slot, owed, index, result, at)
		local row = slot and (judged(self, slot, index, result, at))
		if owed and not (row and isSet(slot, row, result)) then
			row = {}
			for k, v in pairs(owed) do row[k] = v end
			if slot then row.move, row.kind, row.ladder, row.rung = slot.move, slot.kind, slot.ladder, slot.rung end
			row.ts, row.outcome, row.flags, row.seconds = at, O.done, 0, result.elapsed or 0
			row.reps = owed.kind == K.cardio and (owed.reps == 1 or owed.reps == 2) and 1 or 0
		end
		return row ~= nil and ns.LightsEmber(row, ns.Rung(self.state, ns.CARDIO))
	end

	-- A row a catch-up's result writes on the computer, folded into the state at once as the game's rows fold there (its
	-- sheet part a mark) and kept (judgedRows), folded in again after every fold of the state (refold), so the pump's
	-- later moves read it whatever rows merge meanwhile, until E:settle lets it go at the catch-up's end.
	local function keepJudged(self, r)
		filled(r)
		self.judgedRows = self.judgedRows or {}
		self.judgedRows[#self.judgedRows + 1] = ns.EncodeRow(r)
		ns.ApplyRead(self.state, r, "mark")
	end

	-- The step down an easier pick makes after a catch-up's result `row` of `slot` (ns.EasierDown, after the rung rows
	-- `after` its set brought, none without a set), as Events.lua Lc.WriteResult writes it after every outcome: since
	-- 2026-10-26 a move Swap took to its easier version, on the attempt at the rung its owed row stood for. Kept as the
	-- set's rows (keepJudged); returns { ladder, rung } (the rung it steps down to), which the helper writes as a rung
	-- row of cause 8 (contract/companion-api.yaml `ran.down`), or nil.
	local function judgedDown(self, slot, row, after)
		local down = ns.EasierDown(self.state, slot, row, after)[1]
		if not down then return nil end
		keepJudged(self, down)
		return { ladder = down.ladder, rung = down.rung }
	end

	-- A catch-up's count on the computer (2026-10-12; David: it "did not ask me how many I could do"), judged as the
	-- game's own catch-up judges an owed move's (judged) and written nowhere: the helper writes the catch-up's rows from
	-- it (Session::catch_up), which reach the engine as the game's, while an app row would pay the move's XP twice (the
	-- game's sheet counts an app row as a catch-up's). A set (isSet: a typed count, a counted window saved with its
	-- answer, a steady move's kept pace) has its load row, its row, the rung rows the rules add and the steps it mastered
	-- (ns.Progression) kept and folded (keepJudged); then, set or not, an easier pick's step down (judgedDown). Returns
	-- the set as the judge wrote it, nil (nothing of it folded) for a result that is no set: { reps, seconds, outcome,
	-- flags, load, rungs = { { ladder, rung, cause } }, mastered = { { ladder, rung } } }, and the step down or nil; the
	-- desktop's pump (helper/src-tauri/src/desk/pump.rs) keeps the judge's own bits of `flags` alone.
	function Engine:judge(slot, index, result, at)
		local row, load = judged(self, slot, index, result, at)
		if not isSet(slot, row, result) then return nil, judgedDown(self, slot, row, {}) end
		if load then keepJudged(self, ns.LoadRow(row, load)) end
		keepJudged(self, row)
		local rungs, mastered = {}, {}
		local after = ns.Progression(self.state, row, ns.FilterContext(self.profile, self.state))
		for _, r in ipairs(after) do
			r.ts = at
			keepJudged(self, r)
			if r.kind == K.mastered then
				mastered[#mastered + 1] = { ladder = r.ladder, rung = r.rung }
			else
				rungs[#rungs + 1] = { ladder = r.ladder, rung = r.rung, cause = r.reps }
			end
		end
		return { reps = row.reps, seconds = row.seconds, outcome = row.outcome, flags = row.flags, load = load,
			rungs = array(rungs), mastered = array(mastered) }, judgedDown(self, slot, row, after)
	end

	-- A catch-up's move done on its count step without a count (Events.lua Lc.EmptyDone's done row, which the helper
	-- pays done with no set): the step down its easier pick makes (judgedDown, no rung rows before it) or nil.
	function Engine:emptyDone(slot, index, at)
		local row = ns.ResultRow(slot, index, O.done, slot.kind == K.cardio and slot.seconds or 0)
		row.ts = at
		return judgedDown(self, slot, row, {})
	end

	-- Whether the one-tap check-in is due at `at` (ns.CheckinDue).
	function Engine:checkinDue(at)
		return ns.CheckinDue(self.state, at)
	end

	-- The XP the app's own sheet counted on the game day of `at`, its rested bonus in it (the phones' todayXP; Stats.lua
	-- ns.DayXP): Pump home's Today while Hero shows the app's own sheet.
	function Engine:todayXp(at)
		return ns.DayXP(self.state, at)
	end

	-- The latest felt-unwell row's time, the app's own fold's or the game's save's (whose owed and banked move rows for
	-- feeling unwell lock too); nil for none.
	function Engine:unwell()
		local own, save = self.state.unwellTs, self.save.unwellTs
		if own and save then return math.max(own, save) end
		return own or save
	end

	-- Whether every pump is locked at `at` (the latest felt-unwell stop's game day), and that stop's time. The lock's
	-- end is the embedder's: the next game day's first second in its time zone.
	function Engine:locked(at)
		local ts = self:unwell()
		return ts ~= nil and ns.LockedToday({ unwellTs = ts }, at), ts
	end

	-- The records of `moves` (by the move as the fold reads it), each with the goal an app pump asks of it at `at`;
	-- moves without one left out, by ascending id. A loaded move's carry its weighted record (topLoad, topReps), its
	-- last set's weight (lastLoad) and the weight planned at `at` (load, ns.LoadPlan), the goal the goal at it.
	function Engine:records(moves, at)
		local ids = keys(set(moves))
		local out = {}
		for _, move in ipairs(ids) do
			local best = ns.Record(self.state, move)
			if best then
				local load = ns.Loaded(move) and ns.LoadPlan(self.state, move, at, self.profile.gear) or nil
				local topLoad, topReps, lastLoad = ns.LoadRecord(self.state, move)
				out[#out + 1] = { move = move, record = best, goal = (ns.Goal(self.state, move, at, load)),
					topLoad = topLoad, topReps = topReps, lastLoad = lastLoad, load = load }
			end
		end
		return array(out)
	end

	-- A loaded move's next fresh set after the sets so far (ns.NextLoad: the Result's "Next time: 11 kg, 15+ reps"):
	-- { load, target }, nil for a move without a set with a weight.
	function Engine:nextLoad(move, at)
		local load, target = ns.NextLoad(self.state, move, at, self.profile.gear)
		if not load then return nil end
		return { load = load, target = target }
	end

	-- Each ladder's rung now, its best rung and the days its top (a pose family's goal poses of its step) was reached at
	-- it, a pose family's on its main pose and answered Perfect among them (slice 47, DB.lua markAttempt: `main`,
	-- `perfect`; 0 on every other ladder).
	function Engine:rungs(ladders)
		local out = {}
		for i, l in ipairs(ladders) do
			local lad = self.state.ladders[l]
			out[i] = { ladder = l, rung = ns.Rung(self.state, l), best = ns.BestRung(self.state, l),
				days = lad and lad.hiDays or 0, main = lad and lad.anchorDays or 0, perfect = lad and lad.perfectDays or 0 }
		end
		return array(out)
	end

	-- Each hold ledger ladder (slice 52, ns.HOLD_LEDGER): its entry as the fold keeps it (ns.HoldEntry: u, v, b10, the
	-- game day of its last set, the rung's prior u0, the game day of its last hold ask, the A lot it keeps, `prior`
	-- while no set of its step was read since a rung change; absent before its first set) and what the step card and
	-- Pro's Path read from it at `at` for the move standing for the step (ns.RungMove, the card's, as Tree.lua's):
	-- `now` (ns.HoldNow: lo, middle, hi in that move's unit on its own top) and `top_in` (ns.HoldTopIn: the least,
	-- middle and most game days to that top), each absent where none (a rep anchor on a hold ladder, a rung changed
	-- since the last set; `now` also while that move has no record).
	function Engine:holds(ladders, at)
		local out, f = {}, ns.FilterContext(self.profile, self.state)
		for i, l in ipairs(ladders) do
			local e = ns.HoldEntry(self.state, l)
			local move = ns.RungMove(l, ns.Rung(self.state, l), f)
			local lo, mid, hi = ns.HoldNow(self.state, l, move, at)
			local soon, days, late = ns.HoldTopIn(self.state, l, move, at)
			out[i] = { ladder = l, now = lo and array({ lo, mid, hi }), top_in = soon and array({ soon, days, late }),
				entry = e and { u = e.u, v = e.v, b10 = e.b10, day = e.day, u0 = e.u0, ask = e.ask, lot = e.lot,
					prior = e.prior } }
		end
		return array(out)
	end

	-- Whether each family is open now (ns.Open).
	function Engine:open(ladders)
		local out = {}
		for i, l in ipairs(ladders) do out[i] = { ladder = l, open = ns.Open(self.state, l) } end
		return array(out)
	end

	-- Whether moves pass the player's filters now (ns.Allowed).
	function Engine:allowed(ids)
		local out = {}
		for i, id in ipairs(ids) do out[i] = { move = id, allowed = ns.Allowed(id, self.state, self.profile) } end
		return array(out)
	end

	-- An entry of ns.SetupOpens as the screens name it: its key, how many moves it opens and their best quality as the
	-- screens count it (1 Starter to 7 Artifact, the rule's 0 to 6 one up), none where no move of them has one. A
	-- switch (Floor moves, Room to move) is nil unless `switches` keeps it: Your setup has its own rows for them, while
	-- a step's card names whichever piece opens more on its ladder.
	local function setupStep(open, switches)
		if not switches and (open.key == "floor" or open.key == "space") then return nil end
		return { key = open.key, count = open.count, quality = open.quality >= 0 and open.quality + 1 or nil }
	end

	-- Your setup's equipment lines (the phones' AgnosticEngine.setupSteps, over ns.SetupOpens): up to `count` pieces of
	-- equipment not added, one after another, each the piece that opens the most (ns.SetupOpens' order: the best colour,
	-- then the most moves) with the pieces before it added, so each line names another step, with how many moves it
	-- opens past the ones before it.
	function Engine:setupSteps(count)
		local profile = CopyTable(self.profile)
		local out = {}
		while #out < count do
			local step
			for _, open in ipairs(ns.SetupOpens(profile, self.state)) do
				step = setupStep(open)
				if step then break end
			end
			if not step then break end
			out[#out + 1] = step
			profile.gear[step.key] = true
		end
		return array(out)
	end

	-- What each piece of equipment not added would open now on its own (the phones' AgnosticEngine.setupOpens, the
	-- game's equipment tooltip, ns.Settings.GearTip, over ns.SetupOpens), which a gear chip's tooltip in Your setup
	-- names; a piece that opens nothing is left out.
	function Engine:setupOpens()
		local out = {}
		for _, open in ipairs(ns.SetupOpens(self.profile, self.state)) do
			out[#out + 1] = setupStep(open) -- nothing for a switch
		end
		return array(out)
	end

	-- What one more piece of the setup, a switch among them, would open on each of `ladders` (ns.SetupOpens with the
	-- ladder, as the Talents tab's card lists it, Tree.lua Tree.CardMore): a step's card's gear line on the computer, a
	-- list a ladder in the order asked.
	function Engine:ladderOpens(ladders)
		local out = {}
		for i, l in ipairs(ladders) do
			local list = {}
			for _, open in ipairs(ns.SetupOpens(self.profile, self.state, l)) do
				list[#list + 1] = setupStep(open, true)
			end
			out[i] = array(list)
		end
		return array(out)
	end

	-- Each step of another ladder in `list` (contract/library.json's entries of a family's `opens`, a rung's `needs` and
	-- a move's `checks`) as a card reads it at `at` (slice 38; the desktop's `needs` op, in their order): done
	-- (ns.NeedDone on E:startState, the test a climb's needs, a move's checks and a family's opening walk; an `any`
	-- group once one of its entries is), what it waits for (ns.NeedAt with the player's filter: the move the setup offers
	-- on that step and the record that does it, a check's pose), and on a stretch check (ns.IsCheck, slice 42; an `any`
	-- group's where the choice the setup offers, ns.EntryFor with the setup alone as the game's Tree.Alternative, is one,
	-- that choice's place in it from 1, `choice`, and its pose) the least answer to "How close to the goal pose?" it
	-- counts (`check`, ns.CheckAsk) and, once done, how its row says it (ns.CheckPassed, the game's Tree.CheckDone):
	-- when it was first passed (`at`, the earliest over the group's checks: "Passed {date}"), else, where no answer did
	-- it, the stretch ladder of the choice that is done and its step now (`ladder`, `step`: "Passed: your {ladder} is at
	-- step {n}.").
	function Engine:needs(list, at)
		local s = self:startState(at)
		local f = ns.FilterContext(self.profile, s)
		local setup = ns.FilterContext(self.profile, nil)
		local out = {}
		for i, need in ipairs(list) do
			local shown = ns.EntryFor(need, setup)
			local check = ns.IsCheck(shown)
			local move, count = ns.NeedAt(check and shown or need, f)
			local e = { done = ns.NeedDone(s, need), move = move, count = count }
			if check then
				e.check = ns.CheckAsk(shown)
				for c, o in ipairs(need.any or {}) do
					if o == shown then e.choice = c end
				end
				if e.done then e.at, e.ladder, e.step = ns.CheckPassed(s, shown, need) end
			end
			out[i] = e
		end
		return array(out)
	end

	-- The titles earned now (ns.TitlesEarned) by their codes (ns.Enum.title), in the closed list's order (ns.Titles):
	-- an app pump's Result names those earned after it and not before it, as the phones' titlesEarned does.
	function Engine:titles()
		local earned, out = ns.TitlesEarned(self.state), {}
		for _, t in ipairs(ns.Titles) do
			if earned[t.id] then out[#out + 1] = E.title[t.id] end
		end
		return array(out)
	end


	-- The level of each lifetime XP in `xps` (ns.LevelOf, Stats.lua's levelOf), in their order: { level, into (the XP
	-- into it), need (what the next one needs; at the cap the next Paragon rank's), paragon (the rank past the cap) }
	-- each. The desktop app's Saga reads a day's level at its start and its end through it (history.json saga rule 15),
	-- and its Result a paired player's bar with the pump's XP on the game's (2026-10-29), never a Rust copy of the curve.
	function Engine:levels(xps)
		local out = {}
		for i, xp in ipairs(xps) do
			local level, into, need, paragon = ns.LevelOf(xp)
			out[i] = { level = level, into = into, need = need, paragon = paragon }
		end
		return array(out)
	end

	-- The fresh goal after a count of `move` (ns.FreshGoal: one step past it, a long hold of a hold ladder in fives;
	-- the phones' nextGoal): the least the next fresh set asks after a first record of `value`, which the Result names
	-- under it.
	function Engine:nextGoal(move, value)
		return ns.FreshGoal(move, value)
	end


	-- A Body switch (its profile key) or an opt-in move (its id) set on the Exercises page.
	function Engine:setting(key, value)
		if type(key) == "number" then
			self.profile.include[key] = value or nil
		else
			self.profile[key] = value
		end
	end

	-- The app's own sheet summary at `now` (Stats.lua ns.SheetSummary, v 6, as the phones' own summary): no save stamp,
	-- the week from Monday's game day, the profile's filters and title; its lists marked (dx and bd the game's alone).
	local SUMMARY_LISTS = { "st", "wk", "lad", "ear", "nx", "mq", "mw", "held", "lk", "last", "ls", "lu", "ld", "lx",
		"va", "lv", "dx", "bd" }
	local SUMMARY_NESTED = { lad = true, ear = true, nx = true, lu = true, lx = true, dx = true, bd = true }

	-- A summary with its lists marked.
	local function marked(sum)
		for _, key in ipairs(SUMMARY_LISTS) do
			local list = sum[key]
			if list then
				if SUMMARY_NESTED[key] then
					for _, entry in ipairs(list) do array(entry) end
				end
				array(list)
			end
		end
		return sum
	end

	-- The app's own summary carries no day's XP and no best days (dx, dxFrom, bd): with only the app's own sheet the
	-- Saga shows today's XP alone (Engine:todayXp) and anchors on the game's sheet alone (history.json saga rules 4 and
	-- 15), as the phones' own summaries leave them out; the game's (Engine:gameSummary) carries them.
	function Engine:summary(now)
		local sum = ns.SheetSummary(self.state, now, nil, self.profile)
		sum.dx, sum.dxFrom, sum.bd = nil, nil, nil
		return marked(sum)
	end

	-- The first game day the game's rows the helper holds give whole (the game's sheet's dxFrom: Stats.HoldFrom), nil
	-- for every day: they begin the history after a /qp reset all (its row's time `resetAt`, the rows from it on) or
	-- with the start (a start rung row, Setup.BeginTrial's, on the earliest row's game day); else the earliest row's
	-- day lost the rows a save's ring had let go before the helper took the save in (into the save's baseline, which
	-- the fold starts from only once the helper read a save its client reads back: gameSummary's `save`), and the day
	-- after it is the first whole one.
	local function wholeFrom(rows, resetAt)
		if resetAt then return nil end
		local first, starts = nil, {}
		for _, str in ipairs(rows) do
			local r = ns.DecodeRow(str)
			if r then
				first = math.min(first or r.ts, r.ts)
				if startRung(r) then starts[ns.Stats.DayIndex(r.ts)] = true end
			end
		end
		local day = first and ns.Stats.DayIndex(first)
		if not day or starts[day] then return nil end
		return day + 1
	end

	-- The game's own sheet as the game folds its save at its next load (Stats.lua ns.SheetSummary, v 6), for the
	-- desktop app while no sheet the strip brought is as new as the game's rows (no Screen Recording grant, or a session
	-- played out of the strip's sight), from `save` (the gameSave op's, the helper's Session::game_save; nil before one
	-- came): the save's baseline (its history from before its ring, ns.Trim; none from before the latest reset, whose
	-- history went with it, as ns.Import leaves an export's out), then the game's rows from the latest reset on, whole
	-- (its owed, banked and paid moves too), and the app's own, merged as the save merges its mirror (ns.MergeMirror: a
	-- row at or before the baseline's horizon is in it already, each row under the save's number of its character),
	-- then folded with every row through the sheet (ns.Fold without a sheetPart, the game's own fold) in the game's
	-- weeks (from the region's weekly reset `reset`, a reset instant a sheet of the game named, as ns.BindDB anchors it;
	-- the US reset without one), at `at` with the save's stamp (`saved`: its newest game row), read with the profile of
	-- the character it is folded for (gameProfile) and the save's ring horizon (rh, ns.Horizon; 2026-10-19). The XP it
	-- counted on each game day is the summary's `dx`: without a baseline (none before the helper read a save, and none
	-- of a save its client never reads back, the beta's, whose next load folds every row as this does), from the first
	-- day the rows give whole (wholeFrom). Folded again only after rows came (E:fold, E:merge, E:write), for another
	-- reset or another save. nil while the game's rows hold none.
	local WEEK_S = 7 * 86400

	-- The game's own sheet's state (gameSummary's fold of `save`, kept in `gameSheet`) handed to `use` while the game's
	-- weeks from `reset` hold, `use`'s answer returned; the app's own weeks and the profile bound before it (ns.profile,
	-- which `use` may bind) again after it, whatever the fold or `use` did. nil while the game's rows hold none.
	local function inGameSheet(self, reset, save, use)
		if next(self.gameRows) == nil then return nil end
		local anchor = reset and ns.DEFAULT_RESET + (reset - ns.DEFAULT_RESET) % WEEK_S or ns.DEFAULT_RESET
		local bound = ns.profile
		ns.weekByDay, ns.weekAnchor = false, anchor
		local ok, out = pcall(function()
			if not self.gameSheet or self.gameAnchor ~= anchor or self.gameSave ~= save then
				local floor, saveSide, rows = self.reset, {}, {}
				for _, str in ipairs(self.game) do
					local r = ns.DecodeRow(str)
					if r and (not floor or r.ts >= floor) and ns.Source(r.flags) ~= E.source.app then
						saveSide[#saveSide + 1] = str
						rows[#rows + 1] = str
					end
				end
				-- a phone's Start over never reaches the game (the mirror leaves it out): its rows from before it stay
				for _, str in ipairs(self.own) do
					local r = ns.DecodeRow(str)
					if r and (not floor or r.ts >= floor) and r.kind ~= K.reset then rows[#rows + 1] = str end
				end
				local seed = save and save.baseline
				if seed and floor and (tonumber(seed.through) or 0) < floor then seed = nil end
				-- every row comes in as the mirror's does, the game's before the app's own (each side sorted at its
				-- running time, the game's row first within one second), each under the save's number of its character
				local db = { log = {}, chars = seed and CopyTable(save.chars or {}) or {}, baseline = seed and CopyTable(seed) }
				ns.MergeMirror(db, { rows = rows, chars = seed and save.rowChars or {} })
				self.gameSheet, self.gameAnchor, self.gameSave = ns.Fold(db), anchor, save
				-- its ring horizon (ns.Horizon: the merge trims the save's log to its ring, as the save does), the
				-- summary's rh, by which the phones' Correct a count lists only the sets the game's ring still holds
				self.gameHorizon = ns.Horizon(db)
				local whole = not seed and wholeFrom(saveSide, floor)
				if whole then ns.Stats.HoldFrom(self.gameSheet.sheet, whole) end
				ns.state = self.state -- ns.Fold names the state it folded; the engine's own stays the one
			end
			return use(self.gameSheet)
		end)
		ns.weekByDay, ns.weekAnchor, ns.profile = true, nil, bound
		if not ok then error(out, 0) end
		return out
	end

	-- The profile the game's sheet is read with (its filters, its chosen title, its held moves): the addon's profile of
	-- `save`'s character in the game as the game binds it at its next load (ns.BindDB): the addon's defaults filled in
	-- (ns.ProfileFor), then the body every device shares taken in (ns.TakeBody: `save.body`, the mirror's, each field
	-- changed later than the game's own change of it), so a Body switch or a move left out on a phone while the game is
	-- closed reads as it will there; without one, the app's profile without its title (it holds that body already;
	-- the title is the game's choice, which only its sheet names).
	local function gameProfile(self, save)
		if save and save.profile then
			local profile = ns.ProfileFor({ profiles = { Default = CopyTable(save.profile) } }, nil)
			ns.TakeBody(profile, save.body)
			return profile
		end
		local profile = {}
		for k, v in pairs(self.profile) do profile[k] = v end
		profile.title = nil
		return profile
	end

	function Engine:gameSummary(at, reset, save)
		return inGameSheet(self, reset, save, function(state)
			local profile = gameProfile(self, save)
			ns.profile = profile -- bound as the game binds it: a deed reads it (a duel's wins, Stats.lua ns.Titles)
			return marked(ns.SheetSummary(state, at, self.saveAt, profile, self.gameHorizon))
		end)
	end

	-- Today's embers on the app's own sheet at `at` (slice 55, Stats.lua ns.Embers: the cardio moves done in full this
	-- game day, of embers_day), Pump home's embers row where Hero shows no game sheet (with one, the row reads that
	-- summary's `em`, as the phones do); and the week's omen on the player's setup and cardio step by the app's week
	-- (ns.Omen, slice 54: its key, the vault's "Hare week").
	function Engine:embers(at)
		local today, most = ns.Embers(self.state, at)
		local omen = ns.Omen(at, ns.FilterContext(self.profile, self.state), ns.Rung(self.state, ns.CARDIO))
		return { today = today, most = most, omen = omen.key }
	end

	-- The Book ----------------------------------------------------------------------------------------------------------

	-- The desktop app's Book (docs/plan.md slice 35; the phones' Book, slice 30: QuestPumpCore's SheetBook.swift,
	-- HeroBody.swift and BookModel, the Android core's twins), read from the one progression beside the game's sheet the
	-- app shows (`game`: its summary's lad, ear and nx, by ladder id; nil without one), in the game's own rules: every
	-- ladder as Book's row lists it (the higher rung of the game's sheet and the app's own, a record lifted by the app's
	-- own on its move, the move standing for the rung where the sheet names none, a family not open yet with what opens
	-- it), each ladder's fourteen steps as the game's Talents tab reads them (Tree.lua Tree.State, Tree.KeyLines and the
	-- step's card, from the source whose entry that rule picks) and its road to the top (the Path, `path`, from that
	-- source too), each talent tree's glow, rows and hidden steps (the Up next rows read its glow; the engine fixture's
	-- `tree` events carry the rest for the iPhone's port), and every move's facts: its record, last count and familiarity,
	-- trained, above the player's reach, switched off in the pumps, the first rule of the setup that leaves it out, its
	-- next goal, its own quality and step, the stat it raises, a loaded move's weighted record, the move that takes its
	-- place while one is set aside, and since slice 42 whether it reached its top once (`topped`, ns.TopOnce), what it
	-- waits for while not open (`waits`, ns.MoveOpen) and the pose that helps it where the setup offers that pose (`hint`,
	-- a soft need), and the gear that carries a loaded move's load for this profile (`implement`, ns.Implement: How's
	-- steps and the setup line of its page and its window, as the phones' Book picks them; 2026-10-24); and what of the
	-- game's sheet it set aside for the app's own fold (setAside: the ladders started over, and the moves whose record was
	-- cleared or whose sets this fold corrected, where the fold's record and stars stand instead of the game sheet's),
	-- which Hero's body map sets aside too.

	-- `out` with the weight a loaded move's climb asks (ns.ClimbKg: `kg` in whole kg, `each` a weight in each hand,
	-- absent when not), as Panel.lua's nextStepLine and the game's Sheet.ClimbTop add it ("20 with 15 kg or more").
	local function climbWeight(out, move)
		local kg, each = ns.ClimbKg(move)
		out.kg, out.each = kg, kg and each or nil
		return out
	end

	-- A summary's ladder entry (Stats.lua ns.SheetSummary's lad: rung, the move standing for it, its record, its stars or
	-- a pose's days), all zeros where the list holds none.
	local function ladderEntry(list, l)
		local e = list and list[l]
		return { rung = e and e[1] or 0, move = e and e[2] or 0, record = e and e[3] or 0, stars = e and e[4] or 0 }
	end

	-- A summary's progress entry (nx: have, need, have2, need2, and on a pose family its main pose's and its Perfect
	-- days still to come, slice 47; ns.UnlockProgress), zeros where it holds none (the last two nil off a pose family
	-- and from an older addon).
	local function progressEntry(list, l)
		local p = list and list[l]
		return { p and p[1] or 0, p and p[2] or 0, p and p[3] or 0, p and p[4] or 0, p and p[5], p and p[6] }
	end

	-- What the climb from `rung` takes with nothing counted yet (ns.UnlockProgress's needs at that rung, the path's
	-- second number from this engine's records, a pose family's and a counted cardio rung's too since 2026-10-12, a timed
	-- one's full moves since slice 52, and a climb back to a cardio step held before its return_days since slice 53, as
	-- Progress.lua cardioProgress counts them): a game sheet that sent no progress.
	local function needsAt(s, l, rung, f)
		if not ns.CanClimb(l, rung, f, s) then return { 0, 0, 0, 0 } end
		local open = ns.PathIn(s, l, rung, f) and 0 or 1
		if l == ns.CARDIO then
			if ns.CountedRung(rung, f) then return { 0, ns.CleanDays(rung, s), open, 0 } end
			return { 0, ns.Returning(s, ns.CARDIO, rung) and C.return_days or C.cardio_full_days, 0, 0 }
		end
		if ns.POSE[l] then
			local p = ns.PoseNeeds({}, l, rung)
			return { p.have, p.need, open, 0, p.mainLeft, p.perfectLeft }
		end
		return { 0, ns.UnlockDays(l, rung, s), open, 0 }
	end

	-- The game's progress `p` on the app's own rung and move, the app's own (`own`) taken where it counts more, where
	-- both count the same climb: the higher count, and the lower second (1 while the climb's path is out, ns.PathIn),
	-- with what the higher count still lacks (a pose family's fifth and sixth numbers).
	local function liftedProgress(p, own)
		if own[2] <= 0 or own[2] ~= p[2] or own[4] ~= p[4] then return p end
		local from = own[1] > p[1] and own or p
		return { math.max(p[1], own[1]), p[2], math.min(p[3], own[3]), p[4], from[5], from[6] }
	end

	-- What a Start from a step's card or a move's window starts (ns.StartFor, slice 36; `target` { ladder, rung } or
	-- { move }) on `state` with `profile` at `at`: the move, its kind (`as`), whether it is a counted set (`allout`), the
	-- ladder and rung it trains at, its goal and the goal's kind, when a set of it is fresh again, whether the day's
	-- sets passed start_tired_sets, when the climb's day can count again and why (`dayKind`), an orange or gold step's
	-- earning count, the ladder whose climb it runs instead (`via`: opens or needs) and whether it climbs back to a step
	-- the player had (`back`); or why nothing starts (`refused`: setup, off or pain) with the move behind it, its
	-- ladder and rung, the first of the setup's rules it fails (`gap`, as Your setup names it) and whether the
	-- player's own list leaves it out (`off`); a Long Haul's (A.haul) its length too (`haul`, seconds). A.pick takes it
	-- as it is.
	local function startShape(profile, st, why, subject)
		if not st then
			local move = subject.move
			return { refused = why, move = move, ladder = subject.ladder, rung = subject.rung, off = subject.off,
				gap = move and ns.Library[move] and ns.SetupGap(ns.Shut(move, ns.FilterContext(profile, nil))) or nil }
		end
		return { move = st.move, as = st.as, allout = st.allout, ladder = st.ladder, rung = st.rung, goal = st.goal,
			goalKind = st.goalKind, freshFrom = st.freshFrom, tired = st.tired, dayIn = st.dayIn, dayKind = st.dayKind,
			earn = st.earn, ask = st.ask, via = st.via, back = st.back, haul = st.haul }
	end

	local function startWith(state, profile, target, at)
		return startShape(profile, ns.StartFor(state, profile, target, at))
	end

	-- A step's rung reached: from the start, kept, or first reached on a game day (`ear`'s 0, 1 or day index), at or
	-- under the best rung the ladder ever stood on; nil above it.
	local function reachedAt(earned, r)
		if r > (earned[1] or 0) then return nil end
		local v = earned[r + 1] or 1
		if v == 0 then return { kind = "start" } end
		if v == 1 then return { kind = "kept" } end
		return { kind = "day", day = v }
	end

	-- What the one progression sets aside of the game's sheet (the clear row): the ladders started over (their
	-- restartTs) and the moves whose record was cleared (s.cleared), and since 2026-10-19 (the program's double-check:
	-- a phone's correction of a game record down kept the game's older max on the Book, Hero and the move page) the moves
	-- a correction changed a set of in the fold (s.fixed's moves), each a set by id. The game's sheet shows them as they
	-- stood before the clear or the correction until the game folds the row at its next load, while the app's own fold
	-- holds the game's rows and the clear or the correction, so it speaks for them: its record and stars.
	local function setAside(s)
		local ladders, moves = {}, {}
		for l, lad in pairs(s.ladders) do
			if lad.restartTs then ladders[l] = true end
		end
		for id, ts in pairs(s.cleared or {}) do moves[id] = ts end
		for _, x in pairs(s.fixed or {}) do
			if x.move then moves[x.move] = moves[x.move] or true end
		end
		return ladders, moves
	end

	-- A game sheet's list by ladder id (lad, ear or nx) without the ladders started over; nil for none.
	local function without(list, ladders)
		if not list then return nil end
		local out = {}
		for l, v in pairs(list) do
			if not ladders[l] then out[l] = v end
		end
		return out
	end

	function Engine:book(game, at)
		game = game and ns.ShiftSummary(CopyTable(game)) -- an older game's sheet in this library's rungs
		local s, profile = self:startState(at), self.profile
		local f = ns.FilterContext(profile, s)
		-- the setup alone (no pain flag), as Your setup and a move's switch name what leaves a move out
		local setupOnly = ns.FilterContext(profile, nil)
		local own = ns.SheetSummary(s, at, nil, profile)
		-- the game's sheet less what the one progression sets aside: a ladder started over has no entry, earned steps or
		-- progress from it, and an entry whose move's record was cleared or a set of it corrected no record, no stars and
		-- no progress (the clear counts its days again, the app's own fold its progress and its record)
		local restarted, cleared = setAside(s) -- `cleared`: the moves cleared or corrected
		local glad, gear, gnx = without(game and game.lad, restarted), without(game and game.ear, restarted),
			without(game and game.nx, restarted)
		for l, e in pairs(glad or {}) do
			if cleared[e[2]] then
				glad[l] = { e[1], e[2], 0, 0 }
				if gnx then gnx[l] = nil end
			end
		end

		local function record(id)
			return id and id > 0 and ns.Record(s, id) or 0
		end

		-- Where a ladder's steps and its colour are read: the game's entry where it stands on the higher rung (a tie the
		-- higher record, the game's lifted by the app's own on its move; a full tie the game's: `sheet`), else the app's
		-- own.
		local function source(l)
			local o = ladderEntry(own.lad, l)
			local strength = ns.STRENGTH[l] == true
			local lad = s.ladders and s.ladders[l]
			local placedTs = lad and lad.placedTs
			local e = glad and glad[l] and ladderEntry(glad, l)
			-- a strength ladder's stars where its standing move has a window on the rung: the most the entries give
			-- the chosen entry's move on its rung
			local function stars(chosen, standing)
				if not (strength and standing and ns.Window(l, chosen.rung, standing)) then return nil end
				local best
				for _, x in ipairs({ e or false, o }) do
					if x and x.rung == chosen.rung and x.move == chosen.move then best = math.max(best or 0, x.stars) end
				end
				return best
			end
			if e and e.rung > 0 and (e.rung > o.rung or e.rung == o.rung and math.max(e.record, record(e.move)) >= o.record)
			then
				local standing = e.move > 0 and e.move or ns.UnlockMove(l, e.rung, f)
				local rec = math.max(e.record, record(standing))
				local earned = gear and gear[l]
				if not earned then -- a sheet without its earned steps: every rung up to its own kept, on no known day
					earned = { e.rung }
					for k = 1, e.rung do earned[k + 1] = 1 end
				end
				local nx = gnx and gnx[l]
				local progress = nx and progressEntry(gnx, l) or needsAt(s, l, e.rung, f)
				if e.rung == o.rung and e.move == o.move then
					progress = liftedProgress(progress, progressEntry(own.nx, l))
				end
				return { open = true, rung = e.rung, earned = earned, progress = progress, standing = standing, record = rec,
					stars = stars(e, standing), game = nx ~= nil, sheet = true,
					placed = not strength or not standing or rec > 0 or progress[1] > 0 or placedTs ~= nil }
			end
			local standing = o.move > 0 and o.move or nil
			return { open = o.rung > 0, rung = o.rung, earned = own.ear[l] or { 0 }, progress = progressEntry(own.nx, l),
				standing = standing, record = record(standing), stars = stars(o, standing), game = false,
				placed = not strength or not standing or placedTs ~= nil }
		end

		-- The move standing for rung `r` on its step: the rung's own move the setup allows, else its anchor.
		local function stepMove(l, r)
			local id = ns.UnlockMove(l, r, f)
			if id and ns.Library[id].rung == r then return id end
			return ns.Anchor(l, r)
		end

		-- A climb that waits for the step of another ladder (ns.RungNeed): that step's anchor and the record that does
		-- it (`seconds` on a hold), or on a pose or cardio step that step's move and its ladder, whose climb does it
		-- (`waitsStep`, 2026-10-10).
		local function waits(need, count)
			if not count then return { kind = "waitsStep", move = need, ladder = ns.Library[need].ladder } end
			return { kind = "waits", move = need, count = count, seconds = ns.Library[need].dose == "hold" }
		end

		-- What shuts the climb from `rung` (ns.TopOut): on the app's own source the setup's rule that leaves out the
		-- first anchor shut, else (only the player's own choice) that anchor switched off, or the step of another ladder
		-- the rung waits for (`waits`); no rule where a pain flag is among what shuts it, nor on cardio where the setup
		-- shuts it (no anchor to name).
		local function shut(l, rung)
			local id, why = ns.TopOut(l, rung, f, s)
			if not id then return { kind = "shut" } end
			if why[1].reason == "need" then return waits(why[1].move, why[1].value) end
			local gap = ns.SetupGap(why)
			if gap then return { kind = "shut", gap = gap } end
			return { kind = "off", move = id }
		end

		-- What shuts the climb from the rung now where its source's progress counts nothing: on the app's own source
		-- what of the setup or the player's choice does (shut), on the game's sheet the step of another ladder it waits
		-- for, on the one progression's records (ns.RungNeed), else shut with no rule.
		local function closed(l, src)
			if not src.game then return shut(l, src.rung) end
			local need, count = ns.RungNeed(s, l, src.rung + 1)
			if need then return waits(need, count) end
			return { kind = "shut" }
		end

		-- A counted cardio rung's forms (2026-10-16: their clean windows count its clean days; with their top counts
		-- until then): the forms the setup allows (the setup alone, as ns.CountedRung decides counted or timed: a pain
		-- flag holds the climb, never changes its measure; slice 42's fix round, 2026-10-17), else (the game's sheet on
		-- another setup) every countable form of the rung.
		local function countedForms(rung)
			local ids = ns.CountForms(rung, setupOnly)
			if #ids == 0 then
				for id, m in pairs(ns.Library) do
					if m.count and m.ladder == ns.CARDIO and m.rung == rung then ids[#ids + 1] = id end
				end
				table.sort(ids)
			end
			return array(ids)
		end

		-- Why the setup skips a stage (ns.Stages' `skipped`: no move of it passes the setup), as a card's skip line names
		-- it (slice 42): the player's own list where it alone leaves one of its moves out (`off`, that move: its Include
		-- in pumps opens it again), else the setup's rules (`setup`, its first move, whose gap Your setup opens at).
		local function skipOf(ids)
			for _, id in ipairs(ids) do
				local why = ns.Shut(id, setupOnly)
				local listed = #why > 0
				for _, w in ipairs(why) do listed = listed and w.reason == "excluded" end
				if listed then return "off", id end
			end
			return "setup", ids[1]
		end

		-- A wait of a move not open yet (ns.MoveOpen's answer, `why`) as the Book carries it: a check of the move's own
		-- (`check`, its place among the move's `checks` from 1, which library.json lists and the Book's needs read), or
		-- the stage of a step's path it is in, its step's ladder and rung, the stage, and the move whose top opens it
		-- with that top and a loaded opener's weight (`path`, climbWeight: Tree.OpensAfter's Sheet.ClimbTop); none for
		-- none, or a check the move does not list.
		local function waitOf(id, why)
			if not why then return nil end
			if why.kind == "check" then
				for i, o in ipairs(ns.Library[id].checks or {}) do
					if o == why.need then return { kind = "check", check = i } end
				end
				return nil
			end
			return climbWeight({ kind = "path", ladder = why.ladder, rung = why.rung, stage = why.stage,
				after = why.after, top = why.top }, why.after)
		end

		-- The path the climb from `rung` waits for (ns.Stages, 2026-10-16; the proof until then): each stage in order with
		-- its state (done, skipped, open, waiting), the move to name, its moves (`ids`), the game days at its top it takes
		-- and has (none on a done stage, which counts nothing more: slice 42's fix round, 2026-10-17), its top (ns.MoveTop: a
		-- count, a pose's least answer, true for a countable form's clean window) and each of its moves' own (`tops`, by
		-- move: an alternative's top may differ, the sliding one-arm push-up's 18 beside the hand-on-the-bed one's 12), a
		-- skipped one's why (`skip`, skipOf, which names its move) and, waiting, the move whose top opens it (`after`,
		-- `afterTop`, with a loaded opener's weight, `afterKg` and `afterEach`: ns.ClimbKg); its moves the setup offers
		-- (`offered`, the setup alone, as Your setup names what leaves a move out: a stage's line joins only those, slice
		-- 42's fix round); none for a rung with no path, and every stage done where the sheet the steps read stands above the
		-- rung or its path is in (`done`: a game sheet whose path is in, as kept unlocks are).
		local function pathAt(l, rung, done)
			local out = {}
			for i, st in ipairs(ns.Stages(s, l, rung, f)) do
				local tops, offered = {}, {}
				for _, id in ipairs(st.ids) do
					local top = ns.MoveTop(l, rung, id)
					if top ~= nil then tops[#tops + 1] = { move = id, top = top } end
					if ns.Passes(id, setupOnly) then offered[#offered + 1] = id end
				end
				local e = { state = done and "done" or st.state, move = st.move, ids = array(st.ids), days = st.days,
					have = not done and st.have or nil, top = ns.MoveTop(l, rung, st.move), tops = array(tops),
					offered = array(offered), after = st.after, afterTop = st.top }
				if e.state == "skipped" then e.skip, e.move = skipOf(st.ids) end
				if st.after then e.afterKg, e.afterEach = ns.ClimbKg(st.after) end
				out[i] = e
			end
			return #out > 0 and array(out) or nil
		end

		-- The path's next stage as every Next and key line names it (ns.PathNext, 2026-10-19, the program's
		-- double-check: the game's Tree.PathNext, the phones' ports; the move the climb's turn runs, else what holds it):
		-- its place in the path from 1 (`stage`), the move to name, its top (ns.MoveTop), the game days it takes and
		-- has, a loaded move's weight with its top (climbWeight, as Tree.PathKey's Sheet.ClimbTop), and where that move
		-- is not open what holds it (`waits`, waitOf: a check of its own, or the move whose top opens its stage); none
		-- where the path is in, the rung has none, or the sheet the steps read has the path in (`done`, as pathAt).
		local function pathNext(l, rung, done)
			if done then return nil end
			local n = ns.PathNext(s, l, rung, f)
			if not n then return nil end
			return climbWeight({ stage = n.stage, move = n.move, top = n.top, days = n.days, have = n.have,
				waits = waitOf(n.move, n.waits) }, n.move)
		end

		-- A strength climb's move and top with the weight its climb asks where it is a loaded move (ns.ClimbKg,
		-- 2026-10-12: `kg` in whole kg, `each` a weight in each hand); `out` the climb or the Path's need.
		local function climbAt(out, move, hi)
			out.move, out.top = move, move and hi or nil
			return move and hi and climbWeight(out, move) or out
		end

		-- What the climb to the next step takes (the Talents tab's next step: Tree.lua Tree.OpensLine, Tree.ClimbHead),
		-- with its path (pathAt) and the path's next stage (`next`, pathNext: what a Next and key line name).
		local function climb(l, src)
			local p, rung, move = src.progress, src.rung, src.standing
			if not src.placed then return { kind = "unplaced" } end
			local gameIn = src.game and p[3] == 0
			if l == ns.CARDIO then
				-- a timed rung's full moves (slice 52; its ready days, 2026-10-16, until then: the kind keeps its name, which
				-- the desktop app's engine and Book read), a counted one's clean days
				if not ns.CountedRung(rung, f) then return { kind = "ready", have = p[1], need = p[2] } end
				return { kind = "clean", have = p[1], need = p[2], forms = countedForms(rung), path = pathAt(l, rung, gameIn),
					next = pathNext(l, rung, gameIn) }
			end
			if ns.POSE[l] then
				-- `move` the step's main pose (ns.MainPose), `main` and `perfect` the days still to come on it and
				-- answered Perfect (slice 47; ns.PoseNeeds' mainLeft, perfectLeft), and `mainPose` the main pose the
				-- line under the climb names, as Tree.lua Tree.PoseShort does (2026-10-20): the one the pumps bring for
				-- the step (ns.PoseMain: one the player stepped down from or that is not open gives way to the next of
				-- its chain) where the app's state stands on the rung, else `move` (the game's sheet on another rung),
				-- else the step's anchor
				local named = move and ns.MainPose(l, rung, move) and move or nil
				local main = named
				if ns.Rung(s, l) == rung then main = ns.PoseMain(s, l, f) end
				return { kind = "mark", have = p[1], need = p[2], move = named, mainPose = main or ns.Anchor(l, rung),
					main = p[5] or 0, perfect = p[6] or 0, ask = ns.MarkNeeds(l, rung), path = pathAt(l, rung, gameIn),
					next = pathNext(l, rung, gameIn) }
			end
			local _, hi = ns.Window(l, rung, move or 0)
			return climbAt({ kind = "strength", have = p[1], need = p[2], path = pathAt(l, rung, gameIn),
				next = pathNext(l, rung, gameIn) }, move, hi)
		end

		-- The rung above the rung now: pain while a flag holds the climb (ns.PainOnLadder: on a joint the ladder's anchors
		-- load, on the cardio ladder one that leaves its step or the next no form the setup has), next where the source's
		-- progress needs something, else what shuts it (closed).
		local function above(l, src)
			if ns.PainOnLadder(s, l, src.rung, f) then return { kind = "pain" } end
			if src.progress[2] <= 0 then return closed(l, src) end
			return { kind = "next", climb = climb(l, src) }
		end

		-- An orange or gold rung the ladder stands on that no set has earned (its step not reached: the earned steps leave
		-- it out, ns.BestRung): what earns it, one fresh set of `move`, the move standing for it, at `count` (ns.EarnFloor;
		-- `seconds` on a hold), on a pose family its pose held whole and answered Perfect (`mark`), on the cardio ladder a
		-- fresh clean window of its own form (`clean`, 2026-10-16); nil under orange and where the setup leaves the rung no
		-- move of its level.
		local function toEarn(l, r, move)
			if not ns.TopTier(l, r) then return nil end
			if ns.POSE[l] then return { mark = true } end
			if l == ns.CARDIO then return { clean = true } end
			local count = move and ns.EarnFloor(l, r, move)
			if not count then return nil end
			return { move = move, count = count, seconds = ns.Library[move].dose == "hold" }
		end

		-- What a Start from a step's card starts (startWith, as A.start), on the one progression the app composes from,
		-- so its label and the pump agree where the game's sheet is ahead of it.
		local function startOf(target)
			return startWith(s, profile, target, at)
		end

		-- What mastering step `r` still takes (slice 42, ns.MasteryMoves and ns.MasteryLeft): the moves the setup offers
		-- there that reached their top once (`have`) of all of them (`need`); none where the setup offers none.
		local function mastery(l, r)
			local need = #ns.MasteryMoves(l, r, f)
			if need == 0 then return nil end
			return { have = need - #ns.MasteryLeft(s, l, r, f), need = need }
		end

		-- A ladder's fourteen steps from the top of the scale down, each with its rung, the move standing for it and the
		-- rung's anchor (a step's card says "{move} stands in for {anchor}" where they differ, slice 42), its state, when
		-- it was reached and what its card's Start starts; since slice 42 its path's stages (pathAt: done on a step the
		-- sheet the steps read stands above, or on its own step where the game's sheet counts the climb and has its path
		-- in, as need's `gameIn`; a climb held counts 0 days of 0, ns.UnlockProgress, which says nothing of its path), when
		-- it was mastered (ns.Mastered: kept until its strength ladder starts over, 2026-10-31) and, on a step the player
		-- has (their own too) not mastered yet, what mastering it still takes (mastery); on the cardio ladder each step's
		-- countable forms the setup allows (`forms`, ns.CountForms with the setup alone: the forms its rung climbs on,
		-- which its card's measure names; a pain flag holds the climb, never changes its measure, slice 42's fix round,
		-- 2026-10-17) and on a rung that climbs by clean days (ns.CountedRung) the days it takes (`clean`, ns.CleanDays),
		-- none on a timed rung or where the setup shuts every form, whose rung climbs by full moves as the game's card
		-- says (Tree.Counted), never countedForms' fallback; there its Pace instead (slice 52: `pace`, the seconds its
		-- forms keep their pace now, ns.SteadyPace, of `full`, the full move's).
		local function steps(l, src)
			local byStep = {}
			for r, w in ipairs(ns.Ladders[l].rungs) do byStep[w.step] = r end
			local out = {}
			local stood = math.max(src.earned[1] or 0, src.rung)
			for step = #C.quality_by_step, 1, -1 do
				local quality, r = C.quality_by_step[step] + 1, byStep[step]
				if not r then
					out[#out + 1] = { step = step, quality = quality, state = { kind = "empty" } }
				else
					local reached = reachedAt(src.earned, r)
					local state
					if not src.open then
						state = { kind = "locked" }
					elseif r == src.rung then
						state = { kind = "now", stars = src.stars, earn = not reached and toEarn(l, r, src.standing) or nil }
					elseif r == src.rung + 1 then
						state = above(l, src)
					else
						state = { kind = (r < src.rung or reached) and "held" or "locked" }
					end
					local move = r == src.rung and src.standing or stepMove(l, r)
					local done = r < stood or r == src.rung and src.game and src.progress[2] > 0 and src.progress[3] == 0
					local mastered = ns.Mastered(s, l, r)
					local counted = l == ns.CARDIO and ns.CountedRung(r, f)
					-- a timed cardio step's Pace (slice 52, ns.SteadyPace: Tree.lua Tree.CardPace), its seconds now
					local pace = l == ns.CARDIO and not counted and (ns.SteadyPace(s, r, at)) or nil
					out[#out + 1] = { step = step, quality = quality, rung = r, move = move, anchor = ns.Anchor(l, r),
						state = state, reached = reached, start = startOf({ ladder = l, rung = r }),
						stages = pathAt(l, r, done), mastered = mastered,
						mastery = not mastered and src.open and (r <= src.rung or reached) and mastery(l, r) or nil,
						forms = l == ns.CARDIO and array(ns.CountForms(r, setupOnly)) or nil,
						clean = counted and ns.CleanDays(r) or nil, pace = pace, full = pace and ns.FULL_MOVE_S or nil }
				end
			end
			return array(out)
		end

		-- The Path (2026-10-09; the cores' AgnosticEngine.ladderPath, the phones' twins): what reaching step `from` + 1
		-- takes, by the rules ns.UnlockProgress counts, with `move` standing for rung `from`: a strength ladder not placed
		-- yet its first max set; the move at its window's top on its game days (ns.UnlockDays) with the climb's path
		-- (pathAt); a pose family's mark answered close enough on its days, `ask` the least answer that counts one
		-- (ns.MarkNeeds: Perfect alone below a risky step), with its path; a counted cardio rung's clean days of its forms
		-- (ns.CleanDays), with its path; a timed one's full moves (cardio_full_days, slice 52; its ready days from
		-- 2026-10-16); each path with its next
		-- stage (`next`, pathNext; 2026-10-19). The climb from the rung
		-- now as its steps read it (its source's progress: the days the game's sheet may name), each one above by the
		-- rules and this setup.
		local function need(l, from, move, src)
			local p = from == src.rung and src.progress[2] > 0 and src.progress or nil
			if from == src.rung and not src.placed then return { kind = "unplaced" } end
			local gameIn = src.game and p ~= nil and p[3] == 0
			if l == ns.CARDIO then
				-- a timed rung's full moves (slice 52, under the kind its ready days had), a counted one's clean days
				-- (2026-10-16)
				if not ns.CountedRung(from, f) then return { kind = "ready", days = p and p[2] or C.cardio_full_days } end
				return { kind = "clean", forms = countedForms(from), days = p and p[2] or ns.CleanDays(from),
					seconds = C.count_s, path = pathAt(l, from, gameIn), next = pathNext(l, from, gameIn) }
			end
			if ns.POSE[l] then
				return { kind = "mark", move = move and ns.MainPose(l, from, move) and move or nil,
					days = p and p[2] or C.pose_unlock_days, ask = ns.MarkNeeds(l, from), path = pathAt(l, from, gameIn),
					next = pathNext(l, from, gameIn) }
			end
			local _, hi = ns.Window(l, from, move or 0)
			return climbAt({ kind = "strength", days = p and p[2] or ns.UnlockDays(l, from, s),
				path = pathAt(l, from, gameIn), next = pathNext(l, from, gameIn) }, move, hi)
		end

		-- The least game days a step's climb takes by its need: the days it counts, else one (a first max set, a counted
		-- rung's two sets hours apart).
		local function leastDays(n)
			return n.days or 1
		end

		-- A span of days as the Path says it (C.path_units): days, whole weeks, months or years, the low end rounded down
		-- (at least 1) and the high end up, so the range only widens.
		local function span(low, high)
			local u = C.path_units
			if high < u[1] then return { unit = "days", low = low, high = high } end
			if high < u[2] then
				return { unit = "weeks", low = math.max(1, math.floor(low / 7)), high = math.ceil(high / 7) }
			end
			if high < u[3] then
				return { unit = "months", low = math.max(1, math.floor(low * 100 / 3044)),
					high = math.ceil(high * 100 / 3044) }
			end
			return { unit = "years", low = math.max(1, math.floor(low * 100 / 36525)),
				high = math.ceil(high * 100 / 36525) }
		end

		-- The pace to `target` at the player's own pace, from the game days of the ladder's dated climbs (`ear`): none
		-- under C.path_climbs of them, without a day between two of them, or while the setup shuts a step on the way. Each
		-- step left takes the mean days between two climbs at the fast end and the longest, the wait since the last climb
		-- among them, at the slow end, never under the days its rule takes. The fast end is rounded down once, over all the
		-- steps left, never per step.
		local function pace(src, list, target, today)
			if not target then return nil end
			local least = 0
			for _, st in ipairs(list) do
				if st.rung > target then break end
				if st.shut then return nil end
				least = least + leastDays(st.need)
			end
			local days = {}
			for r = 1, src.earned[1] or 0 do
				local v = src.earned[r + 1]
				if v and v > 1 then days[#days + 1] = v end
			end
			if #days < C.path_climbs then return nil end
			table.sort(days)
			local sum, n, slow = 0, 0, 0
			for i = 2, #days do
				local gap = days[i] - days[i - 1]
				if gap > 0 then sum, n, slow = sum + gap, n + 1, math.max(slow, gap) end
			end
			if n == 0 then return nil end
			slow = math.max(slow, today - days[#days])
			local left = target - src.rung
			local low = math.max(math.floor(left * sum / n), least)
			return span(low, math.max(left * slow, low))
		end

		-- A ladder's road from the rung now to its top (the Path): each step above it, the next first, with its rung, step,
		-- quality and the move standing for it (as its steps name it), what reaching it takes (`need`), the days counted
		-- toward the next one (`have`) and what shuts its climb (`shut`: the next step's as its steps read it, closed, one
		-- above by this setup); the rung it leads to (`target`: the first orange or gold step above the rung now, else the
		-- top) and the pace to it (`pace`); none for a family not open yet.
		local function path(l, src, today)
			if not src.open then return nil end
			local out, target, top = {}, nil, ns.HeldTop(l)
			for r = src.rung + 1, top do
				local from = r - 1
				-- the move standing for the rung climbed from: the steps' own on the rung now (none where the setup leaves it
				-- none), the rung's own above it
				local move
				if from == src.rung then move = src.standing else move = stepMove(l, from) end
				local st = { rung = r, step = ns.Step(l, r), quality = ns.Quality(l, r) + 1, move = stepMove(l, r),
					need = need(l, from, move, src) }
				if from == src.rung then
					if src.progress[2] > 0 then
						st.have = src.progress[1]
					else
						st.shut = closed(l, src)
					end
				elseif not ns.CanClimb(l, from, f, s) then
					st.shut = shut(l, from)
				end
				out[#out + 1] = st
				if not target and ns.TopTier(l, r) then target = r end
			end
			target = target or (top > src.rung and top or nil)
			return { steps = array(out), target = target, pace = pace(src, out, target, today) }
		end

		-- What opens a family not open yet, its first step not done (ns.OpensAt; a list since 2026-10-16): the anchor's
		-- record at its count (else the window's floor) where it names a strength step, a pose's goal pose answered at its
		-- `mark` (the planche's wrist lean), else the step.
		local function opening(l)
			local id, count, o = ns.OpensAt(l, s)
			if not o then return nil end
			if count then return { kind = "record", move = id, count = count, seconds = ns.Library[id].dose == "hold" } end
			if o.mark then return { kind = "pose", move = o.move, ask = o.mark } end
			return { kind = "step", ladder = o.ladder, step = o.step }
		end

		local ladders, sources, entries = {}, {}, {}
		local today = ns.DayIndex(ns.day(at))
		for l = 1, ns.LADDER_MAX do
			if ns.Ladders[l] and ns.HeldTop(l) > 0 then
				local src = source(l)
				sources[l] = src
				-- Book's row (HeroBody.ladder): the entry on the higher rung of the game's and the app's own, each record
				-- lifted by the app's own on its move, a tie the higher record, the game's first
				local entry = { rung = 0, move = 0, record = 0, stars = 0 }
				local list = {}
				for _, sheet in ipairs({ glad or false, own.lad }) do
					if sheet and sheet[l] then
						local x = ladderEntry(sheet, l)
						x.record = math.max(x.record, record(x.move))
						list[#list + 1] = x
						if x.rung > entry.rung or x.rung == entry.rung and x.record > entry.record then entry = x end
					end
				end
				local rung = math.max(1, entry.rung)
				local move = entry.move > 0 and entry.move or ns.UnlockMove(l, rung, f) or ns.Anchor(l, rung) or 0
				local stars = 0
				for _, x in ipairs(list) do
					if x.rung == entry.rung and x.move == entry.move then stars = math.max(stars, x.stars) end
				end
				local rec = math.max(entry.record, record(move))
				-- the next goal the record asks (ns.FreshGoal), the record lifted by the game's sheet's
				local standing = src.standing and ns.Library[src.standing]
				-- its entry in its tree (ns.TreeEntry, slice 36): the ring around its step, its day, a ladder that waits
				-- whole and why, whether a pain flag holds its climb, read from the source the steps read
				local e = ns.TreeEntry(s, l, { open = src.open, rung = src.rung, best = src.earned[1] or 0,
					progress = src.progress, standing = src.standing, record = src.record, placed = src.placed }, f,
					setupOnly, at)
				entries[l] = e
				local appRung = src.sheet and ns.Rung(s, l) ~= src.rung and ns.Rung(s, l) or nil
				-- the Stamina Ledger on the player's step (slice 52): what a hold or power ladder's player can do now of
				-- the move standing for it (ns.HoldNow: lo, middle, hi in that move's unit on its own top, as Tree.lua
				-- Tree.CardNow reads it) and Pro's date for that top (ns.HoldTopIn), the cardio climb's (ns.SteadyForecast:
				-- a timed step's, and a counted one's once its path is in), each from half to twice its middle as the Path
				-- says a span (none at the top);
				-- on the cardio ladder the proving window (slice 53, ns.Proving: due, next or waits, with its form) and
				-- the Long Haul (slice 55: its length, ns.Haul, and its start, ns.HaulStart)
				local nowLo, nowMid, nowHi = ns.HoldNow(s, l, src.standing, at)
				local soon, _, late = ns.HoldTopIn(s, l, src.standing, at)
				if not soon and l == ns.CARDIO then
					local steady = ns.SteadyForecast(s, at, f)
					if steady then soon, late = steady[1], steady[3] end
				end
				local forecast = soon and late > 0 and span(math.max(1, soon), late) or nil
				local proving, provingForm, haul
				if l == ns.CARDIO then
					proving, provingForm = ns.Proving(s, f, at)
					-- the Long Haul, which measures nothing, waits while a steady move finds the step again
					if e.placingBy ~= "steady" then
						haul = { seconds = (ns.Haul(s, at)), start = startShape(profile, ns.HaulStart(s, profile, at)) }
					end
				end
				ladders[#ladders + 1] = { ladder = l, rung = entry.rung, move = move, record = rec,
					next = rec > 0 and move > 0 and ns.FreshGoal(move, rec) or nil, stars = stars,
					quality = entry.rung > 0 and ns.Quality(l, entry.rung) + 1 or 0,
					opens = entry.rung == 0 and opening(l) or nil, steps = steps(l, src), path = path(l, src, today),
					ring = not (e.waits or e.placing) and { fill = e.ring, ticks = e.ticks and array(e.ticks) or nil,
						move = e.move, record = e.record } or nil,
					dayIn = e.dayIn, dayKind = e.dayKind, waits = e.waits, placing = e.placing, placingBy = e.placingBy,
					pain = e.pain or nil,
					appRung = appRung,
					freshFrom = standing and (standing.dose == "rep" or standing.dose == "hold")
						and ns.FreshFrom(s, src.standing, at) or nil,
					now = nowLo and array({ nowLo, nowMid, nowHi }) or nil, forecast = forecast,
					proving = proving and { kind = proving, move = provingForm } or nil, haul = haul }
			end
		end

		-- the trees (slice 36): each one's glowing step and its kind, or why none glows and when a day counts again
		-- (ns.TreeGlow: the highest ring among the climbs a set can move now, else the tree's ladder the Trial places
		-- next; none while felt unwell locks every pump), and the rows it draws in full (ns.TreeRows), from its ladders'
		-- entries
		local trees, locked = {}, self:locked(at)
		for t, tree in ipairs(ns.TREES) do
			local rows, hidden = ns.TreeRows(t, entries)
			local glow, kind, when = ns.TreeGlow(t, entries, ns.TrialNext(s, f, t), locked)
			trees[t] = { tree = tree.key, glow = glow, kind = glow and kind or nil, reason = not glow and kind or nil,
				at = when, rows = array(rows), hidden = hidden }
		end

		-- What a move not open yet waits for (ns.MoveOpen, slice 42; waitOf): none for an open move.
		local function waitsOf(id)
			local open, why = ns.MoveOpen(s, id, f)
			if open then return nil end
			return waitOf(id, why)
		end

		-- The stage of a step's path that holds a move beside a check of its own (ns.PathWaits, slice 42's fix round,
		-- 2026-10-17: the desk pistol's window names its stage after the Bulgarian beside its ankle check, as the game's
		-- does); none where its `waits` names that stage already, or no stage holds it.
		local function stageWait(id, held)
			if not held or held.kind ~= "check" then return nil end
			return waitOf(id, ns.PathWaits(s, id, f))
		end

		local moves = {}
		for id, m in pairs(ns.Library) do
			local best, last = ns.Record(s, id)
			local step, days = ns.Familiarity(s, id)
			local trained = step >= 1 or (best or 0) > 0
			for l in pairs(glad or {}) do
				if glad[l][2] == id and (glad[l][3] or 0) > 0 then trained = true end
			end
			-- above the reach on the ladder it scales by, against the rung the steps read where it is higher
			local scale = ns.ScaleLadder(m)
			local beyond = scale ~= nil and scale ~= ns.CARDIO and (m.rung or 1)
				> math.max(ns.Rung(s, scale), sources[scale] and sources[scale].rung or 0) + ns.Reach(scale)
			local why = ns.Shut(id, setupOnly)
			-- its own rung's quality and step on its own ladder (a tile's edge), none off the ladders
			local quality = m.ladder and ns.Quality(m.ladder, m.rung)
			local topLoad, topReps, lastLoad = ns.LoadRecord(s, id)
			local held = waitsOf(id)
			moves[#moves + 1] = { move = id, record = best, last = last, familiarity = step, days = days,
				trained = trained, above = beyond, off = why[1] ~= nil and why[1].reason == "excluded",
				gap = ns.SetupGap(why), next = best and ns.FreshGoal(id, best) or nil,
				quality = quality and quality + 1 or nil, step = quality and ns.Step(m.ladder, m.rung) or nil,
				stat = ns.MoveStat(id), topLoad = topLoad, topReps = topReps, lastLoad = lastLoad,
				better = ns.BetterOpen(id, f), topped = ns.TopOnce(s, id) or nil, waits = held,
				pathWaits = stageWait(id, held), hint = m.hint and ns.Passes(m.hint, setupOnly) and m.hint or nil,
				implement = ns.Implement(id, profile.gear) }
		end
		table.sort(moves, function(a, b) return a.move < b.move end)
		return { ladders = array(ladders), moves = array(moves), trees = array(trees),
			setAside = { ladders = array(keys(restarted)), moves = array(keys(cleared)) } }
	end

	-- The one progression -------------------------------------------------------------------------------------------

	-- The latest reset row's time among row texts, or nil.
	local function latestReset(list)
		local reset
		for _, str in ipairs(list) do
			local r = ns.DecodeRow(str)
			if r and r.kind == K.reset and (not reset or r.ts > reset) then reset = r.ts end
		end
		return reset
	end

	-- The kinds "Start over in the app" keeps from before it (slice 28; the phones' Backup.Floor, the Android core's
	-- Backup.KEPT_KINDS, questpump-core app_rows::after_start_over): the pain flags (the body's state), the focus (the
	-- account's), the clear rows (a record cleared or a ladder started over, the account's) and, since slice 44
	-- (contract/fixtures/backup.json imports case 10), every correction and load row, its set's or not: a correction
	-- carries the time of the set it names, so one written after a Start over, of a game set from before it, is the
	-- player's now; a kept row whose app set the floor dropped names nothing here, and the game keeps its own sets.
	local FLOOR_KEEPS = { [K.pain] = true, [K.focus] = true, [K.clear] = true, [K.correction] = true, [K.load] = true }

	-- Whether a row of the app's own folds into its own sheet after the app's latest Start over at `startOver` (a
	-- phone's reset row, which the helper keeps since 2026-10-09; none: every row), never the reset row itself.
	local function pastStartOver(r, startOver)
		return r.kind ~= K.reset and (not startOver or r.ts >= startOver or FLOOR_KEEPS[r.kind] == true)
	end

	-- The joints the game's own rows flag (its save's pain flags), ascending.
	function Engine:gameFlags()
		return array(keys(self.save.pain))
	end

	-- An owed row text read as the fold reads an owed row (ns.Translate: a retired ladder's on the ladder that took its
	-- move, a merged move's as the move that took it, at that move's own rung), keyed as the game's queue keys it
	-- (ns.OwedKey).
	local function owedRow(str, op)
		local r = ns.Translate((assert(ns.DecodeRow(str), op .. ": a row that does not read")))
		r.key = ns.OwedKey(r)
		return r
	end

	-- The move each owed row rides as now where it is another (ns.RidesAs, 2026-10-29: an owed backup of its rung the
	-- setup leaves unneeded rides as the move that stands for that rung, as A.catchup doses it), which the unfinished
	-- list on the computer names as the game's HUD does: `rows` the owed rows as the helper's store keeps them
	-- (Session::payable_kept), each read as the fold reads an owed row (ns.Translate) and held as the game's state
	-- holds its queue (as Pump:swapOptions holds a catch-up's). Returns { move (as owed now), as } for each row that
	-- rides as another move, in their order.
	function Engine:ridesAs(rows, at)
		local queue, list = {}, {}
		for i, str in ipairs(rows) do
			local r = owedRow(str, "ridesAs")
			queue[r.key], list[i] = r, r
		end
		local as = ns.RidesAs(setmetatable({ owed = queue }, { __index = self.state }), self.profile, at)
		local out = {}
		for _, r in ipairs(list) do
			if as[r.key] then out[#out + 1] = { move = r.move, as = as[r.key] } end
		end
		return array(out)
	end

	-- Folds the rows from the baseline `base` on (nil: every row) as the save folds with its mirror: `saveSide` (the shown
	-- focus and the game's kept rows, in their order) and `ownSide` (the app's own, in theirs) merged by time
	-- (ns.MergeMirror: each side sorted at its running time, the game's row first within one second, a row both sides
	-- hold kept once, as the game's, the rows past the ring folded into the baseline, ns.Trim), then folded (ns.Fold:
	-- the game's corrections applied, its undone rung changes left out). The app's own side runs on from the horizon as
	-- the game's does, so each row sorts where a fold of every row sorts it. The ring is kept again by side, each in its
	-- order (the merge keeps every side's). The catch-up's sets E:judge kept fold in again on top, as the game's rows
	-- fold (a mark), until E:settle lets them go.
	local function refold(self, base, saveSide, ownSide)
		local gameRows, focusRow = self.gameRows, self.focusRow
		local db = { log = saveSide, chars = {}, baseline = base }
		function db.sheetPart(str)
			if gameRows[str] then return "mark" end
			if str == focusRow then return "skip" end
			return "own"
		end
		ns.MergeMirror(db, { rows = ownSide, chars = {}, through = base and base.through })
		self.state, self.base = ns.Fold(db), db.baseline
		for _, str in ipairs(self.judgedRows or {}) do ns.ApplyRead(self.state, ns.DecodeRow(str), "mark") end
		local ringSave, ringOwn = {}, {}
		for _, str in ipairs(db.log) do
			if gameRows[str] or str == focusRow then ringSave[#ringSave + 1] = str else ringOwn[#ringOwn + 1] = str end
		end
		self.ringSave, self.ringOwn = ringSave, ringOwn
	end

	-- The player's one progression (slice 26; the phones' AgnosticEngine.fold): the app's own rows (`own`, every device's)
	-- and the game's rows as the helper serves them (`game`, row texts), both from the game's latest reset row on (a
	-- `/qp reset all` is the player's, everywhere), the app's own also from its latest Start over on (a phone's reset
	-- row among them: pastStartOver, as the phones prune theirs; the game's sheet, E:gameSummary, keeps the rows before
	-- it, as the game does), folded as a save with its mirror (refold). The game's owed, banked and
	-- paid move rows stay out (a match pump's compose takes the queue apart, A.compose's `queue`; a catch-up's, its
	-- rows), and so does a game row with the app bit (an app's own,
	-- which the history never serves). The app's own sheet takes the app's own rows whole and the game's only as marks
	-- (ns.SheetMark: a climb's first reach and a felt-unwell day), and `sheetFocus` (the shown game sheet's focus, its
	-- summary's fc) folds as the oldest focus row of all, read by the engine and never by the sheet. Beside the state
	-- (E.state) the save's own fold (E.save: every game row from the reset on in its order, its owed and banked moves too)
	-- gives the game's pain flags and its felt-unwell stop. Returns { gameFlags (E:gameFlags) }.
	function Engine:fold(own, game, sheetFocus)
		local reset = latestReset(game)
		local save, kept, gameRows, queued, newest, saveAt = ns.NewState(nil), {}, {}, {}, 0, 0
		local apply, report = applier()
		for _, str in ipairs(game) do
			local r = ns.DecodeRow(str)
			if r then
				newest = math.max(newest, r.ts)
				if (not reset or r.ts >= reset) and ns.Source(r.flags) ~= E.source.app then
					apply(save, ns.DecodeRow(str), "skip")
					if MOVE[r.kind] and QUEUE[r.outcome] then
						queued[#queued + 1] = r.ts
					else
						kept[#kept + 1] = str
						gameRows[str], saveAt = true, math.max(saveAt, r.ts)
					end
				end
			end
		end
		report()
		self.queued = queued
		self.gameRows, self.ownRows, self.ownAt, self.exact = gameRows, {}, 0, true
		local mine, startOver = {}, latestReset(own)
		self.startOver = startOver
		for _, str in ipairs(own) do
			local r = ns.DecodeRow(str)
			if r then
				newest = math.max(newest, r.ts)
				if (not reset or r.ts >= reset) and pastStartOver(r, startOver) then
					mine[#mine + 1] = str
					self.ownRows[str], self.ownAt = true, math.max(self.ownAt, r.ts)
					if gameRows[str] then self.exact = false end
				end
			end
		end
		-- the shown game sheet's focus, the oldest focus row of all, which the sheet never reads
		local focusRow = sheetFocus and ns.EncodeRow({ ts = 0, char = 0, questID = 0, tier = 0, slot = 0, ladder = 0,
			rung = 0, move = 0, kind = K.focus, reps = sheetFocus, seconds = 0, outcome = 0, flags = 0 })
		local log = {}
		if focusRow then log[1] = focusRow end
		for _, str in ipairs(kept) do log[#log + 1] = str end
		self.reset, self.focus, self.focusRow, self.saveAt = reset, sheetFocus, focusRow, saveAt
		refold(self, nil, log, mine)
		self.save, self.own, self.game, self.newest, self.gameSheet = save, {}, {}, newest, nil
		for i, str in ipairs(own) do self.own[i] = str end
		for i, str in ipairs(game) do self.game[i] = str end
		return { gameFlags = self:gameFlags() }
	end

	-- Rows from elsewhere since the fold (the app's own of another device, `own`, and the game's as the helper serves
	-- them, `game`; never a row E:write wrote, which the state holds already), taken the cheapest way that ends where a
	-- fold of every row ends: the game's into its save at once, in their order (as E:fold folds the save); into the
	-- state, "order" while every row it takes is newer than all the engine folded and wrote (each applied in turn, as
	-- the merge sorts them: at its side's running time, the game's first within one second); else "ring" while each
	-- sorts past the baseline's horizon (the ring merged again and folded from the baseline, refold); else "fold",
	-- everything folded again (E:fold, the shown focus kept): a reset among the game's rows, a correction or an undo
	-- among either side's (each changes rows folded already: rewrites), a row at or before the horizon, or a row both
	-- sides hold. Rows that bring
	-- nothing into the state (none, or only the game's owed, banked and paid moves, which only its save takes) go in
	-- "order", as nothing to apply; a Start over among the app's own (a phone's reset row) folds everything again. An
	-- app row (the app bit) sorts at its own time, as the merge sorts it (DB.lua ns.MergeMirror), so one of another
	-- device older than rows folded already takes the fold. Returns { how (which) }.
	-- Whether a row changes rows folded already, so a merge folds every row again for it (Engine:merge): a correction
	-- (and its correcting load row, which comes with it) names an older set, and an undo (a rung row's or a mastered
	-- row's, the cause undo) takes back an older step change; an app device writes both since slice 44, the game its
	-- /qp undo and Correct a count. Applied in turn, an undo would set the rung it names, never the change it pairs
	-- with undone (DB.lua undone).
	local function rewrites(r)
		return r.kind == K.correction or ((r.kind == K.rung or r.kind == K.mastered) and r.reps == ns.RungCause.undo)
	end

	function Engine:merge(own, game)
		local through = self.base and tonumber(self.base.through) or 0
		local incoming, newest, inOrder, inRing, startOver = {}, self.newest, true, true, false
		local function take(r, str, isGame, at)
			incoming[#incoming + 1] = { r = r, str = str, game = isGame, at = at, n = #incoming + 1 }
			if r.ts <= self.newest then inOrder = false end
			if at <= through then inRing = false end
		end
		local apply, report = applier()
		for _, str in ipairs(game) do
			self.game[#self.game + 1] = str
			local r = ns.DecodeRow(str)
			if r then
				newest = math.max(newest, r.ts)
				if r.kind == K.reset or rewrites(r) then inOrder, inRing = false, false end
				if (not self.reset or r.ts >= self.reset) and ns.Source(r.flags) ~= E.source.app then
					apply(self.save, ns.DecodeRow(str), "skip")
					if MOVE[r.kind] and QUEUE[r.outcome] then
						self.queued[#self.queued + 1] = r.ts
					else
						self.gameRows[str], self.saveAt = true, math.max(self.saveAt, r.ts)
						if self.ownRows[str] then self.exact = false end
						take(r, str, true, self.saveAt)
					end
				end
			end
		end
		for _, str in ipairs(own) do
			self.own[#self.own + 1] = str
			local r = ns.DecodeRow(str)
			if r then
				newest = math.max(newest, r.ts)
				startOver = startOver or r.kind == K.reset
				if rewrites(r) then inOrder, inRing = false, false end
				if (not self.reset or r.ts >= self.reset) and pastStartOver(r, self.startOver) then
					self.ownRows[str], self.ownAt = true, math.max(self.ownAt, r.ts)
					if self.gameRows[str] then self.exact = false end
					take(r, str, false, ns.Source(r.flags) == E.source.app and r.ts or self.ownAt)
				end
			end
		end
		if #game > 0 or #own > 0 then self.gameSheet = nil end
		local how = "fold"
		if startOver then
			self:fold(self.own, self.game, self.focus)
		elseif #incoming == 0 then
			how = "order"
		elseif not self.exact then
			self:fold(self.own, self.game, self.focus)
		elseif inOrder then
			-- the merge's order: each side's running time never falls, and an app row sorts at its own time
			table.sort(incoming, function(a, b)
				if a.at ~= b.at then return a.at < b.at end
				if a.game ~= b.game then return a.game end
				return a.n < b.n
			end)
			-- a row that does not fold is skipped here exactly as a fold of every row skips it
			for _, e in ipairs(incoming) do
				apply(self.state, e.r, e.game and "mark" or "own")
				local side = e.game and self.ringSave or self.ringOwn
				side[#side + 1] = e.str
			end
			how = "order"
		elseif inRing then
			for _, e in ipairs(incoming) do
				local side = e.game and self.ringSave or self.ringOwn
				side[#side + 1] = e.str
			end
			refold(self, self.base, self.ringSave, self.ringOwn)
			how = "ring"
		else
			self:fold(self.own, self.game, self.focus)
		end
		-- a fold of every row reported the rows it skipped; the order and the ring report theirs here
		if how ~= "fold" then report() end
		self.newest = math.max(self.newest, newest)
		return { how = how }
	end

	-- A catch-up's end: the sets E:judge kept leave the state, the ring folded again from the baseline as a merge past
	-- the horizon folds it (refold). Asked just before the merge of the rows the helper wrote for them (they come in as
	-- the game's, and so count once), or where nothing was paid. Returns whether it folded (nothing judged: nothing to
	-- do).
	function Engine:settle()
		if not self.judgedRows then return false end
		self.judgedRows = nil
		refold(self, self.base, self.ringSave, self.ringOwn)
		return true
	end

	-- Correct a count (slice 44, 2026-10-18) ---------------------------------------------------------------------------
	-- One correction, from any device (docs/plan.md slice 44, "One correction, from any device"): the desktop app's
	-- window over Progress.lua's rules, as the game's and the phones' read them (ns.Correctable, ns.CorrectionCheck,
	-- ns.CorrectionRows), on the log this engine folds.

	-- The log the engine folds, as Correct a count reads it: its ring of each side merged over its baseline as refold
	-- merges them (ns.MergeMirror, which may trim it into a baseline of its own: a copy's), from which ns.Correctable
	-- lists the sets and ns.Replay replays them.
	local function engineLog(self)
		local db = { log = {}, chars = {}, baseline = self.base and CopyTable(self.base) or nil }
		for i, str in ipairs(self.ringSave) do db.log[i] = str end
		ns.MergeMirror(db, { rows = self.ringOwn, chars = {}, through = self.base and self.base.through })
		return db
	end

	-- The apps' sweep at their load and after a set or a catch-up, written at `at`, quietly (no Result row, no toast):
	-- first an undo row for each strength ladder's latest climb of the app's own that today's rules would not make
	-- (ns.HeldClimb, 2026-10-24: a tuck jump's climb into power step 7, whose clap push-up waits for 12 feet-up
	-- push-ups), naming it (its ts in questID) as Correct a count's undo does, a second after it where a phone's clock
	-- ran ahead of `at` (the merges sort app rows by their own times, and an undo before its climb pairs with none),
	-- folded again (refold), one step a ladder a pass; then the mastered rows the state lacks (ns.MasterySweep, slice
	-- 42's fix round, 2026-10-17: every step reached whose moves the setup offers all reached their top once and that
	-- holds none; the seal and the count follow the fold), so a step whose last top came another way (a catch-up, a row
	-- from elsewhere, a move left out) masters too. Returns their texts. The game takes back its own climbs at login
	-- (Events.lua Lc.UndoOldClimbs).
	function Engine:sweep(at)
		local log, undo = engineLog(self).log, {}
		for _, l in ipairs(ns.STRENGTH_ORDER) do
			local r = ns.LatestRung(self.state, log, l)
			if r and ns.Source(r.flags) == E.source.app and ns.HeldClimb(self.state, r) then
				undo[#undo + 1] = { ts = math.max(at, r.ts + 1), questID = r.ts, kind = K.rung, ladder = l,
					rung = r.rung - 1, reps = ns.RungCause.undo }
			end
		end
		local out = writeAll(self, undo)
		if #undo > 0 then refold(self, self.base, self.ringSave, self.ringOwn) end
		for _, str in ipairs(writeAll(self, ns.MasterySweep(self.state, ns.FilterContext(self.profile, self.state), at))) do
			out[#out + 1] = str
		end
		return out
	end

	-- The state Correct a count lists and judges the sets of `db` (engineLog's) by: the engine's own, which is a fold of
	-- that log, but while a catch-up's judged sets are held (E:judge, until E:settle) a fold of the log alone (DB.lua
	-- ns.FoldOf): they are the game's rows to come, never a set to correct, so every surface lists and judges from the
	-- fold of the log it lists from (slice 44's review, 2026-10-18), and the record the window names is a set of that
	-- log, never a held count's.
	local function correctState(self, db)
		return self.judgedRows and ns.FoldOf(db) or self.state
	end

	-- Correct a count's window for move `move` at `at` (ns.Correctable on the log the engine folds), its sets dated after
	-- the window's reach (DB.lua ns.CorrectReach: 2 x ring_batch rows of that log and of the game's owed, banked and paid
	-- move rows, E.queued, which the game's ring holds beside it, past `horizon`, the largest rh of the game's sheets the
	-- app knows, where one is given, or past the engine's own baseline, whichever is later; slice 44's review), so the
	-- device never offers a set the game's ring let go or a trim before the game's next load could fold, whose
	-- correction the game's merge would drop: each { row (as stored), kind, value, outcome, load, round, record }, newest
	-- first.
	-- A set's `ownSide` (ns.OwnSide: another side's correction won, and how the set's own side reads it) stays only on
	-- the app's own sets, of which the desktop app is a source: on the game's, the game's window offers its Save
	-- (2026-10-19).
	function Engine:correctable(move, at, horizon)
		local db = engineLog(self)
		local found = ns.Correctable(correctState(self, db), db, move, at, horizon, self.queued)
		for _, e in ipairs(found) do
			if self.gameRows[ns.EncodeRow(e.row)] then e.ownSide = nil end
		end
		return found
	end

	-- Engine:correctable's entries as the contract carries them, a JSON array: each set as stored (`ts`, `quest`, `slot`,
	-- `move`), what a correction may change (`kind`: count, seconds or answer), its value, its outcome (ns.Enum.outcome:
	-- done where a count of 0 reads "Done without a count", a count corrected to 0 too; ns.SetOutcome) and its weight
	-- (tenths of a kg, absent for none) as the fold reads them now, the round a correction of it takes, `record`
	-- (true, else absent) on a set holding the record and `ownSide` (2026-10-19; absent else) where another device's
	-- correction won on an app set and this side reads it otherwise: { value, load (absent for none) } as this side's
	-- own corrections read it, beside which the window offers Save at `value` with its line
	-- (correct.window.elsewhere).
	function A.correctableJSON(found)
		local out = {}
		for i, e in ipairs(found) do
			local r = e.row
			out[i] = { ts = r.ts, quest = r.questID, slot = r.slot, move = r.move, kind = e.kind, value = e.value,
				outcome = e.outcome, load = e.load, round = e.round, record = e.record or nil,
				ownSide = e.ownSide and { value = e.ownSide.value, load = e.ownSide.load } or nil }
		end
		return array(out)
	end

	-- The entry of Engine:correctable at `at` (after the reach `horizon` sets) of the set `key` names ({ ts, quest, slot,
	-- move }: the set as stored, as A.correctableJSON names it), nil once it has left the window (its game days ran out,
	-- a clear row or a newer horizon came, or it left the ring): read again at each step of a correction, so nothing is
	-- written for a set the window no longer holds.
	function Engine:correctSet(key, at, horizon)
		for _, e in ipairs(self:correctable(key.move, at, horizon)) do
			local r = e.row
			if r.ts == key.ts and r.questID == key.quest and r.slot == key.slot and r.move == key.move then return e end
		end
		return nil
	end

	-- The count check a correction of `entry` (Engine:correctable's) to `value` at `load` (tenths of a kg; nil the
	-- set's own weight) meets (ns.CorrectionCheck: the same limits and "Is that right?" as typing it, against the state
	-- just before the set):
	-- nil where it asks nothing (a pose's answer never asks), else { check ("max" refused; "heavy", "really" or "low"
	-- asked once) and what "Is that right?" names: `record` the record before the set (none on a placement or before
	-- the first), `top` the top a placement or a first max set is checked against (none where the window counts in
	-- another unit), `topLoad` and `topReps` the weighted record before the set (none before a set with a weight),
	-- `plan` the set's own weight, and whether the set is a hold (`hold`, its seconds) or a counted set (`counted`, its
	-- best clean count; ns.CorrectionCheck asks no low count of it, as its count step asks none) }.
	function Engine:correctCheck(entry, value, load)
		local check, before, slot = ns.CorrectionCheck(engineLog(self), entry.row, value, load)
		if not check then return nil end
		local m = before.moves[slot.move]
		return { check = check, record = not slot.placing and ns.Record(before, slot.move) or nil,
			top = ns.FirstTop(slot), topLoad = m and m.topLoad, topReps = m and m.topReps, plan = slot.load,
			hold = slot.kind == K.hold or nil, counted = slot.counted }
	end

	-- Corrects `entry` (Engine:correctable's) at `at` to `value` (a count, a hold's seconds) at `load` (a loaded rep
	-- set's weight in tenths of a kg, 0 none; nil keeps its own), or a pose's `answer` (ns.Enum.mark): the rows
	-- ns.CorrectionRows gives (the load row where the weight changes or the set's own side reads another, the correction
	-- row, then the step rows where the corrected set changes them, asked only of the app's own sets, every row of the
	-- app's own store, with the engine's profile, as the engine log keeps no character's (ns.CharProfile): a set among
	-- the game's rows keeps its steps, `stays` "source", and one a later change of any character stands on, "later",
	-- each only where the steps as the mastered sweep after it leaves them still lack the change), each written
	-- (E:write: the app's character and the device bits), then the ring folded again from the baseline as a merge past
	-- the horizon folds it (refold: the set is in the ring, which Engine:correctable reads, and so is every row an undo
	-- pairs with). Returns the rows' texts,
	-- `stays` and the set as the fold reads it now, { value, outcome, load } (ns.SetValue, ns.SetOutcome, ns.SetLoad:
	-- a capped max set corrected over its window's top keeps the top, a count corrected to 0 is done; slice 44's
	-- review), which the window's done view says, as its list says it next; or nil (nothing written) where the set may
	-- not be corrected or its value or answer is none. Fourth, what the rows changed on the steps (ns.CorrectionRows'
	-- third, 2026-10-19), which the done view says ("Push-ups back to step 6.", "Step 7 of Core is no longer
	-- Mastered."): { rungs = [ { ladder, from, to, fromStep, toStep } ], mastered = [ { ladder, rung, step, won } ] }
	-- (each step on the one scale, ns.Step), or nil where they changed none. The mastered sweep after it is the
	-- caller's, as after a set (Engine:sweep).
	function Engine:correctRows(entry, value, load, answer, at)
		local row, db = entry.row, engineLog(self)
		local rows, stays, changes = ns.CorrectionRows(db, correctState(self, db), row, value, load, answer, at,
			self.profile, not self.gameRows[ns.EncodeRow(row)])
		if not rows then return nil end
		local out = writeAll(self, rows)
		refold(self, self.base, self.ringSave, self.ringOwn)
		local s = self.state
		local steps
		if changes then
			local rungs, mastered = {}, {}
			for i, x in ipairs(changes.rungs) do
				rungs[i] = { ladder = x.ladder, from = x.from, to = x.to, fromStep = ns.Step(x.ladder, x.from),
					toStep = ns.Step(x.ladder, x.to) }
			end
			for i, x in ipairs(changes.mastered) do
				mastered[i] = { ladder = x.ladder, rung = x.rung, step = ns.Step(x.ladder, x.rung), won = x.won }
			end
			steps = { rungs = array(rungs), mastered = array(mastered) }
		end
		return out, stays, { value = ns.SetValue(s, row), outcome = ns.SetOutcome(s, row), load = ns.SetLoad(s, row) },
			steps
	end

	-- The pump on screen ------------------------------------------------------------------------------------------------

	local Pump = {}
	Pump.__index = Pump

	-- The queue a match pump composes with (slice 61) from the game's owed rows (`queue`, row texts: every owed row the
	-- game's store keeps) and the rows of it the device may pay (`rows`, row texts as the helper's store keeps them,
	-- Session::payable_kept), each read as the fold reads it (owedRow), a row of both one table: the queue by key, the
	-- rows that may ride oldest first (ns.OwedList's order, the game's availableOwed), and each such row's place in
	-- `rows` (from 1). Nil for none (a pump no match sizes, or a match pump while the queue is empty); a queue with no
	-- row the device may pay still keeps the pump's own picks off its moves, as the game's compose does.
	local function riderQueue(rows, queue)
		if not (rows and #rows > 0 or queue and #queue > 0) then return nil end
		local byKey, payable, place = {}, {}, {}
		for _, str in ipairs(queue or {}) do
			local r = owedRow(str, "compose")
			byKey[r.key] = r
		end
		for i, str in ipairs(rows or {}) do
			local r = owedRow(str, "compose")
			r = byKey[r.key] or r
			byKey[r.key], payable[r.key], place[r] = r, r, i
		end
		return { queue = byKey, owed = ns.OwedList({ owed = payable }), place = place }
	end

	-- The pump a kind and a size ask for at `at` (the phones' AppPump.compose): the start rungs first while no ladder
	-- has a rung, the app's or the game's (a pain flag set before the first pump comes before them, as in the game), then
	-- its slots and its offered row, which a pump with no move never writes (the game drops it too; its `reps` every
	-- move, the riders too, as the game's writeOffered counts them). `id` is the device's (`d-<uuid>` on the desktop),
	-- `kind` the plain kind's key (contract/kinds.json), "quick" or "move", `family` the engine's family for it,
	-- `chosen` a move pump's move (A.pick), `match` a match pump's facts (slices 48 and 57, Engine:compose: { minutes =
	-- the match's whole minutes, warm = true or none, arms = true or none }; nil for every other pump), which the pump
	-- keeps for its recompose and its Swap. A pump a match sized (slice 61) takes `rows`, the owed row texts its device
	-- may pay, and `queue`, every owed row text the game's store keeps (riderQueue): composed on a state holding the
	-- queue, those of `rows` that fit ride it as the game's riders (Engine:compose), its slots of the pump's own and its
	-- riders in the game's order, each rider paying its owed row (`slot.owed`), which A.ridersJSON names; the pump keeps
	-- the queue (Swap reads it, Skip takes a rider out of it), the rows that may ride and their places. `expedition` is
	-- an expedition round's (slice 51, A.expedition: { stage, regions }), which the pump keeps for its recompose and its
	-- Swap. Returns the pump { id, kind, family, tier, chosen, match, expedition, slots, swapped, queue?, owed?, place? }
	-- and the rows written.
	function A.compose(E_, id, kind, family, tier, at, chosen, match, rows, queue, expedition)
		local written = array({})
		if not E_:hasRungs() then written = E_:startRows(at) end
		local riding = riderQueue(rows, queue)
		local slots = E_:compose(family, tier, at, chosen, match, riding and riding.owed, riding and riding.queue,
			expedition)
		if #slots > 0 then written[#written + 1] = E_:pumpRow(slots, tier, at) end
		local pump = setmetatable({ id = id, kind = kind, family = family, tier = tier, chosen = chosen, match = match,
			expedition = expedition, slots = slots, swapped = {} }, Pump)
		if riding then pump.queue, pump.owed, pump.place = riding.queue, riding.owed, riding.place end
		return pump, written
	end

	-- The Quick pump at `at` (David, 2026-10-07: "your next most important exercises for when you only have short
	-- time"): a Small pump that no quest flavours, so the player's state alone picks its moves and how many: a new
	-- player's Trial step, else the major move once a strength or hold ladder has waited major_h since its last record
	-- set, then the day plan's next move, the closing rule as in any pump: one move, or two. Picked from a match's
	-- offer (slice 58, it banks), it carries the unfinished moves `rows` of `queue` as A.compose does (slice 61).
	function A.quick(E_, id, at, rows, queue)
		return A.compose(E_, id, "quick", nil, 1, at, nil, nil, rows, queue)
	end

	-- The riders of a match pump (slice 61; the phones' AppPump riders): for each slot that pays an owed row, in the
	-- pump's order, its index (from 1), its row's place in the rows A.compose took (from 1) and the time its row was
	-- owed, which the helper's payment names; an empty list for a pump with none (a catch-up's riders are its moves,
	-- A.catchup).
	function A.ridersJSON(pump)
		local out = {}
		if pump.place then
			for i, slot in ipairs(pump.slots) do
				if slot.owed then out[#out + 1] = { index = i, row = pump.place[slot.owed], ts = slot.owed.ts } end
			end
		end
		return array(out)
	end

	-- Expeditions (slice 51; Rapha, 2026-10-20: "select how much time you have") ------------------------------------

	local STAGE = {}
	for _, stage in ipairs(ns.EXPEDITION_STAGES) do STAGE[stage] = true end

	-- An expedition's round at `at`: `stage` one of ns.EXPEDITION_STAGES (the one A.expeditionNext names) and `regions`
	-- the regions the rounds before it took (Pump:regions after each round; none before the first), an app pump of the
	-- kind key `expedition` at its stage's tier (ns.ExpeditionTier), composed by ns.Compose's expedition rules; its
	-- start rows when it is the first pump, then its offered row. Returns as A.compose.
	function A.expedition(E_, id, stage, regions, at)
		assert(STAGE[stage], "expedition: no such stage")
		return A.compose(E_, id, "expedition", nil, ns.ExpeditionTier(stage), at, nil, nil, nil, nil,
			{ stage = stage, regions = regions or {} })
	end

	-- The stage of an expedition's next round at `at` (ns.ExpeditionNext on the state a Start reads, the start rungs
	-- folded in before the first pump: Engine:startState): `exp` = { budget (seconds, the length chosen), elapsed
	-- (seconds since the expedition began, the breathers in), last (the stage that ran last; none before the first),
	-- regions (Pump:regions after it) }; nil once the expedition is done.
	function A.expeditionNext(E_, exp, at)
		return (ns.ExpeditionNext({ state = E_:startState(at), profile = E_.profile, now = at, app = true },
			{ budget = exp.budget, elapsed = exp.elapsed or 0, last = exp.last, regions = exp.regions or {} }))
	end

	-- A move done at its goal, the preview's result: a rep or hold move its goal (a first max set none, done with no
	-- count), a timed move its seconds, nothing answered.
	local function atGoal(slot)
		if slot.kind == K.rep or slot.kind == K.hold then
			return { value = slot.target, elapsed = slot.kind == K.hold and slot.target or nil, load = slot.load }
		end
		return { elapsed = slot.seconds }
	end

	-- The plan of an expedition of `minutes` at `at` (slice 51's preview, "never a promise": its words say "about"):
	-- the expedition's rules (A.expeditionNext, A.expedition) played on a copy of the engine, every move done at its
	-- goal (atGoal) and the clock moved by each round's estimate (Pump:time) with the breather (expedition_rest_s)
	-- before every round after the first; nothing is written to `E_`. Returns { rounds = { { stage, slots
	-- (A.slotsJSON) } }, moves (the moves in all), records (the moves whose goal beats the record: the records to
	-- beat), regions (the record moves' regions, Pump:regions), seconds (the estimate, the breathers in) }.
	function A.expeditionPreview(E_, minutes, at)
		local copy = A.new(E_.profile, { stamp = E_.stamp })
		copy.state = CopyTable(E_.state)
		local exp = { budget = minutes * 60, elapsed = 0, regions = {} }
		local rounds, moves, records = {}, 0, 0
		local now = at
		while true do
			local stage = A.expeditionNext(copy, exp, now)
			if not stage then break end
			if exp.last then now, exp.elapsed = now + C.expedition_rest_s, exp.elapsed + C.expedition_rest_s end
			local pump = A.expedition(copy, "preview", stage, exp.regions, now)
			local t = now
			for i, slot in ipairs(pump.slots) do
				t = t + C.countdown_s
				pump:finish(copy, i, atGoal(slot), t)
				t = t + ns.MoveTime(slot)
				moves = moves + 1
				if slot.goal == "record" then records = records + 1 end
			end
			rounds[#rounds + 1] = { stage = stage, slots = A.slotsJSON(pump.slots) }
			local took = pump:time()
			now, exp.elapsed = now + took, exp.elapsed + took
			exp.last, exp.regions = stage, pump:regions(exp.regions)
		end
		return { rounds = array(rounds), moves = moves, records = records, regions = array(exp.regions),
			seconds = exp.elapsed }
	end

	-- What a Start from a step's card or a move's window starts at `at` (startWith) on the app's own progression, its
	-- start rungs folded in before the first pump (Engine:startState), as A.pick composes it.
	function A.start(E_, target, at)
		if target.haul then return A.haul(E_, at) end
		return startWith(E_:startState(at), E_.profile, target, at)
	end

	-- The Long Haul from the cardio step's card at `at` (slice 55, ns.HaulStart; A.start's target { haul = true }) on
	-- the app's own progression, as A.start's start (its length `haul`); A.pick composes it.
	function A.haul(E_, at)
		return startShape(E_.profile, ns.HaulStart(E_:startState(at), E_.profile, at))
	end

	-- A move started from a step's card or a move's window at `at` (slice 36, the book key `move`; David, 2026-10-13: "a
	-- use should be able to start a move from the tree anytime"): `chosen` = { move, allout, rung, ask, haul }, as
	-- ns.StartFor names it (A.start; a Long Haul's A.haul), the pump's one move (an all-out one its window alone, a Long
	-- Haul its length), dosed at the start's rung, a Small pump that pays and owes nothing, as every app pump. Returns
	-- as A.compose.
	function A.pick(E_, id, chosen, at)
		assert(chosen and ns.Library[chosen.move], "pick: no such move")
		return A.compose(E_, id, "move", nil, 1, at, { move = chosen.move, allout = chosen.allout == true,
			rung = chosen.rung, ask = chosen.ask, haul = chosen.haul })
	end

	-- Whether a catch-up's move asks the player for something before it is paid: a rep move's or a hold's count, a
	-- counted set's form, a steady move's pace, or Keep going; any other runs by its time.
	local function asking(slot)
		return slot.kind == K.rep or slot.kind == K.hold or slot.counted or slot.pace or slot.keep ~= nil
	end

	-- A catch-up of the unfinished moves on the computer (2026-10-12; David: it "did not ask me how many I could do"):
	-- every owed row it may pay (`rows`, row texts as the helper's store keeps them, Session::payable_kept), each read as
	-- the fold reads an owed row (owedRow), and the rows it pays (`pays`: for each of its moves in its order,
	-- the place of its row in `rows`, from 1). The whole list is dosed as the game doses the owed moves that ride a pump
	-- (ns.Compose's demand kind, Engine.lua riderSlot), so a rep move or a hold asks its count at the owed ladder and rung
	-- (an owed anchor at the ladder's rung now is the rung's attempt) and an owed all-out window (reps 1, or reps 2 owed
	-- by a closing move's jog before 2026-10-15) is the window of its own move alone, a counted set where that move
	-- counts, and (slice 42's fix round, 2026-10-17) an owed steady cardio move asks its pace where the game's own pump
	-- would (riderSlot through paceSlot: where the steady ledger asks, ns.LedgerAsk, since slice 52) and runs on with
	-- Keep going under the full move (keepSlot, slice 52). It becomes the pump on screen, whose slots are the moves it
	-- pays that ask a count, a form or a pace or take Keep going (asking) and a move Swap put in (Pump:swap), by their
	-- place in the catch-up (from 1; Engine:validate, Pump:judge and A.catchupCallout read them), and whose riders are
	-- the whole list dosed, as the game's pump holds its riders, with the place among them of every move it pays that
	-- the engine placed (`riders`, `rider`), the owed row each place pays (`pays`) and the list by its owed keys
	-- (`queue`), which Swap reads (Pump:swapOptions: since 2026-10-26 for a move that asks, since 2026-11-02 for every
	-- move, as the game offers Swap on every owed move's Ready view; David: "Why is there still no alternative exercise
	-- to switch to like in wow addon") and Skip takes a move out of (Pump:skip); a timed move, a pose, a window of a form
	-- that counts nothing and a move the rules hold back run by their time, as before. No row is written: the helper
	-- writes a catch-up's. Returns the pump and its moves that ask, each { index (from 1), slot (A.slotJSON), move
	-- (Pump:move) }.
	function A.catchup(E_, id, rows, pays, at)
		local owed, place, queue, paid = {}, {}, {}, {}
		for i, str in ipairs(rows) do
			local r = owedRow(str, "catchup")
			owed[i], queue[r.key] = r, r
		end
		for k, i in ipairs(pays) do
			paid[k] = assert(owed[i], "catchup: a move pays a row the list lacks")
			place[paid[k]] = k
		end
		local riders = ns.Compose({ kind = "demand", owed = owed, state = E_.state, profile = E_.profile, now = at })
		local pump = setmetatable({ id = id, kind = "catchup", tier = 0, slots = {}, swapped = {}, riders = riders,
			rider = {}, pays = paid, queue = queue }, Pump)
		local asks = {}
		for k, slot in ipairs(riders) do
			local i = slot.owed and place[slot.owed]
			if i then pump.rider[i] = k end
			if i and asking(slot) then
				pump.slots[i] = slot
				asks[#asks + 1] = { index = i, slot = A.slotJSON(slot), move = pump:move(E_, i) }
			end
		end
		return pump, array(asks)
	end

	-- Composed again for the same kind and size (a move pump for its move, Engine.lua's chosen compose: its backup at
	-- its level where the new flag refuses it, else step jacks; a match pump with its match's facts as they were at its
	-- start, slice 48, and its riders from the rows it may pay still queued, slice 61), its offered row standing and its
	-- swaps undone: a pain flag set before any move was recorded.
	function Pump:recompose(E_, at)
		local owed
		if self.owed then
			owed = {}
			for _, r in ipairs(self.owed) do
				if self.queue[r.key] then owed[#owed + 1] = r end
			end
		end
		self.slots = E_:compose(self.family, self.tier, at, self.chosen, self.match, owed, self.queue, self.expedition)
		self.swapped = {}
	end

	-- The moves Swap reads and the place of the move at `i` among them: a catch-up's riders, every move it placed among
	-- them whether it asks or not (its slots being the moves that ask), else the pump's slots; no place for an index the
	-- pump lacks.
	local function swapsIn(pump, i)
		if pump.riders then return pump.riders, pump.rider[i] end
		return pump.slots, pump.slots[i] and i
	end

	-- Swap's options for the move at `i` on its Ready view (ns.SwapOptions, a move pump's as `chosen`; a match pump's
	-- under its arms rest, slice 48, while its warm move swaps as any move, its riders' of every kind among them since
	-- slice 61; a catch-up's every move since 2026-11-02, one that asks since 2026-10-26), an owed slot's as the game
	-- offers an owed move's, nil for an index the pump lacks. A swapped move offers
	-- only the way back to the slot it replaced (`back`, as it was) while the filters still pass that move, as
	-- Events.lua Lc.SwapOptions does (David, 2026-10-03: "when switching to alternative it should be possible to switch
	-- back again the same way"); back on it, its own options return, so the two can be traded until Start.
	function Pump:swapOptions(E_, i, at)
		local moves, index = swapsIn(self, i)
		if not index then return nil end
		local from = self.swapped[i]
		if from then return { back = ns.Allowed(from.move, E_.state, E_.profile) and from or nil } end
		-- a match pump's riders and a catch-up's Swap read the unfinished list as the game's state holds its queue
		-- (Engine.lua context's `ctx.owed`: never another owed move), which the app's fold leaves out
		local state = E_.state
		if self.queue then state = setmetatable({ owed = self.queue }, { __index = E_.state }) end
		return ns.SwapOptions({ state = state, profile = E_.profile, now = at,
			arms = self.match and self.match.arms == true or nil, expedition = self.expedition }, moves,
			index, self.chosen ~= nil)
	end

	-- The move at `i` dosed again as its Ready view opens (ns.ReadyGoal, Rapha, 2026-10-28; Events.lua Lc.Next): a
	-- second set of a ladder in the pump asks the repeat share, never more than it asked, at `at` on the state the
	-- pump's sets before it left (a catch-up's judged rows among them, keepJudged), under a match pump's arms rest where
	-- it holds one. Never a move pump's, whose goal its card named. True where the goal changed.
	function Pump:ready(E_, i, at)
		if self.chosen then return false end
		return ns.ReadyGoal({ state = E_.state, profile = E_.profile, now = at,
			arms = self.match and self.match.arms == true or nil }, self.slots[i])
	end

	-- The move at `i` becomes the option the player picked (a catch-up's swapped slot keeps the owed row it pays:
	-- Engine.lua swapSlot), the slot it replaced kept for the way back (`swapped`), and the way back puts that slot back
	-- as it was (Events.lua Lc.Swap): false, and nothing changed, for an index the pump lacks or a swapped move's option
	-- other than its way back. A catch-up's slots take the move Swap put in, which the helper runs and pays as the move
	-- run (Pump:move, Pump:lights; it asks a count only where it is asking), and on the way back the move it replaced
	-- only where that one asks, as A.catchup placed it: one that runs by its time runs so again.
	function Pump:swap(i, option)
		local moves, index = swapsIn(self, i)
		local from = self.swapped[i]
		if not index or (from and option ~= from) then return false end
		self.swapped[i] = not from and moves[index] or nil
		moves[index] = option
		if self.riders then self.slots[i] = (not from or asking(option)) and option or nil end
		return true
	end

	-- An expedition round's time by the estimate (ns.PumpTime: each move's countdown and time, its transition after it),
	-- for the ExpeditionBar's minutes left and the next round's elapsed time in the preview.
	function Pump:time()
		return ns.PumpTime(self.slots)
	end

	-- The regions an expedition's rounds took once this round is done, `regions` those before it (ns.RoundRegions over
	-- its moves as they stand, a swap's in): the next round's.
	function Pump:regions(regions)
		return array(ns.RoundRegions(regions, self.slots))
	end

	-- The keys of `t` (a per-move table: Swap's way back, a pose's checks) above `i` one place down, `i`'s own gone, as
	-- a move at `i` leaves the slots (table.remove).
	local function shiftDown(t, i)
		if not t then return end
		local n = 0
		for k in pairs(t) do n = math.max(n, k) end
		for k = i, n do t[k] = t[k + 1] end
	end

	-- Skip of a rider at `i` (Events.lua Lc.Skip: the slot leaves the pump and its owed row is paid, skipped, which the
	-- helper writes), so no later move's Swap counts it in the pump or owed. A match pump's (slice 61): the slot leaves
	-- the slots as the game's table.remove takes it, every move after it one place down (Swap's way back and a pose's
	-- checks by place too), and its row the queue; false for a move of the pump's own or one it lacks. A catch-up's:
	-- the move leaves the riders and the unfinished list Swap reads (`riders`, `queue`), and its slot and its place go;
	-- false for a place the catch-up lacks.
	function Pump:skip(i)
		if not self.riders then
			local slot = self.slots[i]
			if not (slot and slot.owed and self.queue) then return false end
			self.queue[slot.owed.key] = nil
			table.remove(self.slots, i)
			shiftDown(self.swapped, i)
			shiftDown(self.asked, i)
			return true
		end
		local r = self.pays and self.pays[i]
		if not r then return false end
		self.queue[r.key], self.pays[i], self.slots[i], self.swapped[i], self.rider[i] = nil, nil, nil, nil, nil
		for k = #self.riders, 1, -1 do
			if self.riders[k].owed == r then
				table.remove(self.riders, k)
				for p, at in pairs(self.rider) do
					if at > k then self.rider[p] = at - 1 end
				end
			end
		end
		return true
	end

	-- After the Ready view's Pain flagged a joint (its pain rows written already), the move at `from` next: composed
	-- again while no move has ended (`recorded`), else the rest from `from` on kept where the new flags allow it and the
	-- step jacks where they refuse it (ns.PainRest): the player saw those moves. A slot whose move the flag changed
	-- takes Swap again. A match pump's rest stays under its arms rest (slice 48): each move kept passed it, and the step
	-- jacks pass every filter. A rider the new flag refuses leaves the pump to wait in the queue (slice 61), the moves
	-- after it a place down, so a kept slot keeps its way back by the slot itself, never by its place.
	function Pump:afterPain(E_, from, recorded, at)
		if not recorded then return self:recompose(E_, at) end
		local rest = ns.PainRest(E_.state, E_.profile, self.slots, from, at)
		local back = {}
		for j = from, #self.slots do
			if self.swapped[j] then back[self.slots[j]] = self.swapped[j] end
			self.swapped[j] = nil
		end
		for j = #self.slots, from, -1 do self.slots[j] = nil end
		for _, slot in ipairs(rest) do
			self.slots[#self.slots + 1] = slot
			self.swapped[#self.slots] = back[slot]
		end
	end

	-- The result of the move at `i` (Engine:finish); none for an index the pump lacks, and false, nothing judged, for a
	-- rider (slice 61): its row would be an app row that pays its XP a second time and leaves its owed row unpaid, so
	-- it is judged (Pump:judge) and the helper pays it.
	function Pump:finish(E_, i, result, at)
		local slot = self.slots[i]
		if not slot then return array({}) end
		if slot.owed then return false end
		return E_:finish(slot, i, result, at)
	end

	-- Whether the hold set of `value` seconds at `i` asks "Could you have held longer?" at `at` (Engine:holdAsk, slice
	-- 52); false for an index the pump lacks.
	function Pump:holdAsk(E_, i, value, at)
		if not self.slots[i] then return false end
		return E_:holdAsk(self.slots[i], value, at)
	end

	-- A rider's count for the move at `i` (Engine:judge: a catch-up's, or since slice 61 a match pump's rider): its set,
	-- nil for a move that asks none or a count that is no typed one, and the step down its easier pick makes, or nil;
	-- false, nothing judged, for a move of the pump's own, whose rows Pump:finish writes (judged, they would be held and
	-- never written).
	function Pump:judge(E_, i, result, at)
		local slot = self.slots[i]
		if not slot then return nil end
		if not slot.owed then return false end
		return E_:judge(slot, i, result, at)
	end

	-- Whether the move at `i` lights an ember with `result` (Engine:lights), as the row its payment writes reads: a
	-- rider's by the owed row it pays (its slot's, a catch-up's move with no slot its place's), a move of the pump's own
	-- as its judged row; false for an index the pump lacks.
	function Pump:lights(E_, i, result, at)
		local slot = self.slots[i]
		return E_:lights(slot, slot and slot.owed or self.pays and self.pays[i], i, result, at)
	end

	-- A rider's move at `i` done without a count (Engine:emptyDone): the step down its easier pick makes, nil without
	-- one or for a move the engine did not dose; false, nothing kept, for a move of the pump's own (Pump:judge).
	function Pump:emptyDone(E_, i, at)
		local slot = self.slots[i]
		if not slot then return nil end
		if not slot.owed then return false end
		return E_:emptyDone(slot, i, at)
	end

	-- A felt-unwell stop while no timed move runs (a running one ends as its own row with the reason): the pump banked
	-- on its tier with the unwell reason, which locks every pump until 04:00.
	function Pump:stop(E_, at)
		return array({ E_:stopRow(self.tier, at) })
	end

	-- What the answer to "How close to the goal pose?" of the move at `i` counts toward (ns.MarkToward, 2026-10-14),
	-- read on its Ready view (slice 47, the game's Panel.lua poseDaysLine): `kind` earn, step (the days counted toward
	-- the step, `have` of `need`, on a family's own days what they still lack: `main`, `perfect`, `mainPose`), next, and
	-- since 2026-10-16 (slice 42) check (a check above its family's step) or goal (the pose's own goal pose, toward its
	-- step's Mastered or a check). Nil for a move that asks nothing or an index the pump lacks.
	function Pump:toward(E_, i)
		local s = self.slots[i]
		if not (s and s.ask) then return nil end
		local f = ns.FilterContext(E_.profile, E_.state)
		local kind, _, have, need, p = ns.MarkToward(E_.state, s, f)
		-- a family's day (slice 47): what its days still lack, the days on its main pose (`mainPose`, ns.PoseMain) and
		-- answered Perfect (ns.PoseNeeds' mainLeft, perfectLeft), as the game's Ready view says it (Panel.lua
		-- poseDaysLine)
		return { kind = kind, have = have, need = need, main = p and p.mainLeft or nil,
			perfect = p and p.perfectLeft or nil, mainPose = p and ns.PoseMain(E_.state, s.ladder, f) or nil }
	end

	-- What a stretch check opens as the Result names it: its `opens` (Progress.lua ns.AllChecks'), and the move to name
	-- (the move it opens, a family's first step's anchor) or the climb's ladder and its step.
	local function opensJSON(o)
		local out = { opens = { move = o.move, family = o.family, ladder = o.ladder, rung = o.rung } }
		if o.ladder then
			out.ladder, out.step = o.ladder, ns.Step(o.ladder, o.rung)
		else
			out.name = o.move or ns.Anchor(o.family, 1)
		end
		return out
	end

	-- Whether a check's `opens` is the one a card or a window named (`hint`: its move, its family, or the climb's ladder
	-- and rung).
	local function opensAs(o, hint)
		if not hint then return false end
		if hint.ladder then return o.ladder == hint.ladder and o.rung == hint.rung end
		if hint.family then return o.family == hint.family end
		return o.move ~= nil and o.move == hint.move
	end

	-- The check a check's pump runs (slice 42's last review, 2026-10-18; ns.RunsCheck, the game's alike): from the
	-- check a card or a window started (`started` { need?, opens? }: "Do the check", its entry where the start named it,
	-- a check row's, and what that card names it opening), the row of that entry (the one opening what the card names
	-- first) as its Start runs it, the check the climb to its pose's step waits for first where one does (ns.CheckFirst:
	-- ankle step 11's twist step 5 check runs the pike's big toe fold); without its entry, of the live checks opening what
	-- the card names, the first whose Start runs a check pose `id` answers. Nil where none is (a pose pump of no check).
	local function ranCheck(state, id, started, setup)
		local all, need = ns.AllChecks(), started.need
		if need then
			local row
			for _, c in ipairs(all) do
				if sameEntry(c.need, need) then
					if opensAs(c.opens, started.opens) then
						row = c
						break
					end
					row = row or c
				end
			end
			return row and ns.RunsCheck(state, row, setup)
		end
		for _, c in ipairs(all) do
			if opensAs(c.opens, started.opens) and ns.CheckLive(state, c) then
				local run = ns.RunsCheck(state, c, setup)
				if run and ns.AnswersCheck(run.need, id) then return run end
			end
		end
		return nil
	end

	-- How many stretch checks the pose at `i` can pass, read as its ask opens, before the answer (slice 42's fix round,
	-- 2026-10-17; the one list of the game's Panel.lua readChecks and the phones since slice 42's last review,
	-- 2026-10-18, ns.PoseChecks): the check a check's pump runs first (ranCheck: what the card or window that started it
	-- names, as its Start runs it), the check the pump's stretch slot runs (Engine.lua checkSlot, the slot's `check`),
	-- then every other the pose answers, each once and live (ns.CheckLive: never one whose opening is open already, a
	-- family placed or open, a climb into a rung its ladder stands on or above, which no pass opens); kept for
	-- Pump:checked with the one the pump runs. 0 for a move that asks nothing, an index the pump lacks or a pose no
	-- check waits on.
	function Pump:checks(E_, i, started)
		local s = self.slots[i]
		self.asked = self.asked or {}
		self.asked[i] = nil
		if not (s and s.ask) then return 0 end
		local run = started and ranCheck(E_.state, s.move, started, ns.FilterContext(E_.profile, nil))
		local first = {}
		if run then first[1] = run.need end
		first[#first + 1] = s.check
		local list = ns.PoseChecks(E_.state, s.move, first)
		self.asked[i] = { list = list, run = run }
		return #list
	end

	-- What still holds what a passed check opens (ns.OpensNow's answer), as the Result words it: another entry not done
	-- (`check`: the move the setup offers for it, ns.NeedAt with the setup alone, its record's count where one does it,
	-- its least answer where it is a stretch check, its move's ladder, a hold's count in seconds) or the stage of a step's
	-- path (`path`: the move whose top opens it, that top and a loaded opener's weight, climbWeight).
	local function waitJSON(why, setup)
		if why.kind == "path" then return climbWeight({ kind = "path", after = why.after, top = why.top }, why.after) end
		local x = ns.EntryFor(why.need, setup)
		local move, count = ns.NeedAt(x, setup)
		local m = move and ns.Library[move]
		return { kind = "check", move = move, count = count, ask = ns.IsCheck(x) and ns.CheckAsk(x) or nil,
			ladder = m and m.ladder or x.ladder, seconds = m ~= nil and m.dose == "hold" }
	end

	-- What the checks the pose at `i` could pass (Pump:checks, as its ask opened) came to after its answer (slice 42's
	-- fix round, 2026-10-17: "Check passed: {move} is open" only where it is open after the set, ns.OpensNow), in their
	-- order: each its entry, passed or not (ns.NeedDone of its whole entry) and, passed, open or not with what still
	-- holds it (waitJSON), the move or climb it opens (opensJSON) and whether it is the one a check's pump runs (`runs`,
	-- ranCheck: the Result says "Not there yet." where that one did not pass, slice 42's last review). None where the
	-- ask read none.
	function Pump:checked(E_, i)
		local asked = self.asked and self.asked[i]
		if not asked then return array({}) end
		local state, f = E_.state, ns.FilterContext(E_.profile, E_.state)
		local setup = ns.FilterContext(E_.profile, nil)
		local out = {}
		for k, c in ipairs(asked.list) do
			local e = opensJSON(c.opens)
			e.need, e.passed, e.runs = entryJSON(c.need), ns.NeedDone(state, c.group), c == asked.run or nil
			if e.passed then
				local open, why = ns.OpensNow(state, c.opens, f)
				e.open = open == true
				e.waits = not open and why and waitJSON(why, setup) or nil
			end
			out[k] = e
		end
		return array(out)
	end

	-- The move at `i` as a device runs it (the phones' PlannedMove), nil for an index the pump lacks: its slot, move,
	-- kind and count to reach, a timed move's seconds, an all-out window's build from its go (allout_build_s; none on a
	-- counted set, all out from its go), whether it runs one side, then the other (`side`), the record its goal beats
	-- (a counted set's best clean count, never a goal; none on a max set) and the goal's kind (ns.SlotGoal's: record,
	-- form, repeat or heavier), on a form goal the player's form its Ready line names (`form`, ns.Form's; with a weight
	-- planned on a move with a weighted record, a goal read in strength, ns.LoadForm's reps at its weight `formLoad`,
	-- as Panel.lua recordLine), a first max set's top, a placement's record, whether it is a first max set (`firstMax`,
	-- ns.FirstMaxSet: a count under the minimum saves as "Could not do one", never a bank), whether a steady move asks
	-- "Did you keep that pace to the end?" (`pace`, slice 42), the full move's seconds Keep going runs it on to (`keep`,
	-- slice 52), whether it is a Long Haul
	-- (`haul`, slice 55), whether it is a match pump's warm move, dosed easy (`warm`, slice 48's
	-- double-check: a cardio move at the stand-up's easy cadence, a rep or hold move at the repeat share of its form),
	-- a pose's mark and, where it asks "How close to the goal pose?" once it ran its whole time, the least answer that
	-- counts the day (`ask`: 3 Good, 4 Perfect); a loaded move's planned weight in tenths of a kg (`load`, none before
	-- a set with one) and whether it is a step up (`loadUp`, from the working weight `loadFrom`, the top reached twice
	-- at it `loadTop`, ns.LoadTop, on every loaded move), whether it is a weight in each hand (`loadEach`), the kg a
	-- first set starts with (`start`: { lo, hi }), its weighted record (`topLoad`, `topReps`) and its last set's weight
	-- and reps (`lastLoad`, `lastReps`, when that set had a weight); the gear that carries a move's load for this
	-- profile (`implement`, ns.Implement: the setup line's, nil for a makeshift load or none); and where the climb
	-- waits on this move's stage of its path alone (ns.PathDue, slice 42: the Ready view's next-step line, Panel.lua
	-- nextStepLine) the move's top there (`next.top`: reps or seconds, a pose's least answer, true for a countable
	-- form's clean window), the game days at it the stage still needs (`next.left`) and on a loaded move the weight its
	-- climb asks (`next.kg` in whole kg, `next.each` in each hand: ns.ClimbKg); on a rep or hold move with a count to
	-- reach, the sets its ladder took this game day before it (`today`, ns.SetsToday, the repeat share's count; none for
	-- none: its Ready view's "Goal · set N today", 2026-10-28).
	function Pump:move(E_, i)
		local s = self.slots[i]
		if not s then return nil end
		local allout, counted = s.allout == true, s.counted == true
		local m = ns.Library[s.move]
		local loaded = s.kind == K.rep and ns.Loaded(s.move) ~= nil
		local topLoad, topReps, lastLoad, lastReps
		if loaded then topLoad, topReps, lastLoad, lastReps = ns.LoadRecord(E_.state, s.move) end
		local _, goal = ns.SlotGoal(s)
		local nextTop, nextLeft = ns.PathDue(E_.state, s, ns.FilterContext(E_.profile, E_.state), ns.now())
		local today = s.target and (s.kind == K.rep or s.kind == K.hold) and ns.SetsToday(E_.state, s.move, ns.now())
		local form, formLoad
		if goal == "form" and topLoad and s.load then -- a goal in strength (ns.Goal: a planned weight)
			formLoad, form = ns.LoadForm(E_.state, s.move)
		elseif goal == "form" then
			form = (ns.Form(E_.state, s.move))
		end
		return { slot = i, move = s.move, kind = s.kind, target = s.target, goal = goal, form = form, formLoad = formLoad,
			seconds = s.kind >= K.cardio and s.seconds or nil, allout = allout,
			build = allout and not counted and C.allout_build_s or nil, side = m.side == true,
			record = (s.target ~= nil or counted) and ns.Record(E_.state, s.move) or nil,
			top = s.maxSet and s.hi or nil, placing = s.placing and ns.Record(E_.state, s.move) or nil,
			firstMax = ns.FirstMaxSet(E_.state, s), counted = counted, pace = s.pace == true, keep = s.keep,
			haul = s.haul == true,
			warm = s.warm == true, mark = s.mark == true, ask = s.ask,
			load = s.load, loadUp = s.loadUp == true, loadFrom = s.loadFrom, loadTop = loaded and ns.LoadTop(s.move) or nil,
			loadEach = ns.Loaded(s.move) == "each",
			start = loaded and array({ m.kg[1], m.kg[2] }) or nil, implement = ns.Implement(s.move, E_.profile.gear),
			topLoad = topLoad, topReps = topReps, lastLoad = lastLoad, lastReps = lastReps,
			next = nextTop and climbWeight({ top = nextTop, left = nextLeft }, s.move) or nil,
			today = today and today > 0 and today or nil }
	end

	-- The callout of the move at `i` on its Ready view (ns.PickCallout): the take family it says, or nil, with the streak
	-- the device's board shows before the pump and the one its first finished move makes (`before`, `after`; no
	-- `after` once today has a performed move).
	function Pump:callout(E_, i, at, before, after)
		if not self.slots[i] then return nil end
		return ns.PickCallout(E_.state, E_.profile, self.slots[i], i, #self.slots, at, before, after)
	end

	-- A catch-up's move `i` of `n` on its Ready view (Speech.lua calloutTake, which reads the rider's slot): the slot
	-- A.catchup dosed it as or Swap put in, at its `place` in the catch-up `pump` (the pump on screen; none after a
	-- catch-up that failed), none for a move with no slot there, whose callout then reads only the pump's own facts.
	function A.catchupCallout(E_, pump, place, i, n, at, before, after)
		local slot = pump and place and pump.slots[place] or nil
		return ns.PickCallout(E_.state, E_.profile, slot, i, n, at, before, after)
	end

	-- The pump's moves as a device runs them, by slot.
	function Pump:moves(E_)
		local out = {}
		for i in ipairs(self.slots) do out[i] = self:move(E_, i) end
		return array(out)
	end

	-- What each move can earn before its Start, as the phones' Ready view puts it on the quest page (their
	-- rewardPreview: ns.RewardPreview's XP and its record chip), by slot: the XP it pays done and, on a move that asks
	-- for a count past a record standing, the stat that record raises (`stat`, its key) by `amount` (stat_record) if
	-- the set beats `beat` (the chip's `at`: the record, or on a loaded move the reps at the planned weight that match
	-- the weighted record); and whether the move was done before (`seen`, the phones' done(move)): its first
	-- appearance shows the right-versus-wrong pair under its figure.
	function Pump:preview(E_, at)
		local out = {}
		for i, slot in ipairs(self.slots) do
			local p = ns.RewardPreview(E_.state, slot, at)
			out[i] = { xp = p.xp, seen = E_.state.seen[slot.move] == true }
			for _, chip in ipairs(p.chips) do
				if chip.kind == "record" then out[i].stat, out[i].amount, out[i].beat = chip.stat, chip.amount, chip.at end
			end
		end
		return array(out)
	end

	return A
end
