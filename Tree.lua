-- The Talents tab (docs/plan.md slice 37, on slice 36's design; docs/ui.md "The talent trees" and "The words"): the
-- ladders drawn as three talent trees in the sheet's ladders page, one tree at a time, with Forever's own tabs under
-- the sheet to switch them, as the player spells frame hangs its own. A column is a ladder and a row one of the 14
-- steps, Starter at the top; a node is a rung, the sheet's square ItemButton with its anchor's key pose in its step's
-- quality. Four things speak (the ring round the player's step on each ladder, one glow a tree on the climb to train
-- next, the diamond that ends each column, the colour of the steps owned); the links show only for the selected step.
-- A click opens the step's card, QuestPumpStepCard, beside the sheet, whose Start opens a pump asked for with that one
-- move on its Ready view (Lc.StartMove); after the set the sheet comes back on the step and the ring fills once the
-- dim has gone (Tree.Return). Every rule comes from Progress.lua's trees section (ns.StartFor, ns.TreeEntry,
-- ns.StepRing, ns.TreeGlow, ns.TreeRows) and the ladder reads beside it, over the game's own fold: nothing here
-- decides a step, a ring or a start. The board is built once a tree, on its first showing, by fixed math in a vertical
-- scroll list (Sheet.ScrollList: ConsolePort's cursor scrolls it to a node), and drawn only while it shows.

local ADDON, ns = ...
local C, L = ns.C, ns.L
local Lc, Style, Sheet, Setup, Kit = ns.Lifecycle, ns.Style, ns.Sheet, ns.Setup, ns.Kit
local saga = Style.saga

-- `shown`: what the card shows, { l, r } its step, `move` another move of it, `choice` its Start (ns.StartFor's),
-- `refused` the passing state a click met (`refusedPump` whether that click was to open a waiting pump), `playing`
-- what its figure drew last (Tree.CardHead); kept apart from the card's frame, whose fields are the client's
local Tree = { boards = {}, tree = 1, expanded = {}, opened = {}, shown = {} }
ns.Tree = Tree

local T = ns.Timers()

-- The page and the board, in UI units (fixed layout sizes: docs/ui.md, "The talent trees"; the inset page under the
-- sheet's portrait is 623 x 420, its scroll list 587 wide with the scroll bar beside it). The head is one row (the
-- kit's page head: the tree's name, its Next line and the search box, the rule under them at 46); the column names'
-- row under it, where they show, then the board from under the names (NAMES_TOP) or the head (BARE_TOP) down to the
-- foot's badges at FOOT_Y, or to the page's foot where no foot shows.
local PAGE_H, BOARD_W, BOARD_X = 420, 587, 8
local HEAD_NAMES_Y, NAMES_TOP, BARE_TOP, FOOT_Y, FOOT_GAP = 58, 74, 56, 398, 2
local GUTTER, EDGE, PITCH_MAX, BRANCH_GAP = 24, 6, 76, 12 -- the step numbers' gutter; two branches stand 12 apart
local ROW_H, THIN_H, DIVIDER_H, TOP_PAD = 50, 12, 16, 4 -- a full row, a thin row (treeThinRow), a quality divider
local SLOT, DOT, DOT_HIT = 37, 6, 20 -- a node (the sheet's slot), a thin row's dot and its hit area
local RING = SLOT + 12 -- the ring round the player's step, past the slot's dark backing (its gap)
-- the glow round the ring, its halo (contract/design/tokens.json treeHalo) past the ring on each side: the action
-- bar's proc loop draws its band round a frame 1/1.4 of its own size (ActionButton's spell alert), so its frame is 1.4
-- times that, and Style.ProcGlow's alpha
local HALO = 2
local GLOW, GLOW_A = math.floor((RING + 2 * HALO) * 1.4), 0.6
-- the beads under a ringed step whose climb has a path (slice 42; the tokens' treeBead 4, treeBeadGap 3, treeBeadRow
-- 8, beadStroke and beadWaitingFraction 0.5, each 1.5 times as large here, where a UI unit is 1.5 to 1.9 pixels on a
-- 1440-line screen and a phone's point 3, so the beads read at about the phones' size under this board's 37-unit
-- nodes): a bead, the gap between two, the band its row gains under the tiles, an open bead's ring and a waiting one's
-- dot; the mastered seal inside a node's bottom-right corner (treeMastered 10; 12 here, a third of the node, as the
-- token is of the phones' tile)
local BEAD, BEAD_GAP, BEAD_ROW, BEAD_STROKE, BEAD_WAIT = 6, 4, 12, 1.5, 3
local SEAL = 12
local LINE_W, HEAD_W, HEAD_H = 4, 14, 12 -- a column's line and a link's stroke; a link's arrow head
local LINKS, CHIPS = 8, 4 -- the most links one step shows (the arm balance's step 11 waits for two), and chips
local CHIP_MAX = 260 -- a cross-tree chip's widest, its words cut short past it
local FIG, TILE = 120, 30 -- the step's card's figure and a move tile
-- the return's motion (contract/design/tokens.json: ringFill's hold and fill, nodeFlash, glowMove, pulse) and the one
-- frame a still glow's flipbook needs before it holds
local HOLD_S, FILL_S, FLASH_S, GLOW_S, PULSE_S, STILL_S = 0.15, 0.4, 0.6, 0.24, 1.2, 0.05
local POSE, STRENGTH = ns.POSE, ns.STRENGTH

-- Words ---------------------------------------------------------------------------------------------------------------

local moveName = Sheet.MoveName

-- A count as the tree says it, as the pump's own words do: a hold's seconds ("45 s"), else the number.
local function value(id, v)
	return Sheet.Amount(v, ns.Library[id].dose == "hold")
end

local function goal(id, v)
	return L["amount.goal"]:format(value(id, v))
end

local function days(n)
	return ns.Plural("tree.days", n):format(n)
end

-- A region centred on a point of the board, from its top left.
local function pin(board, region, x, y)
	region:ClearAllPoints()
	region:SetPoint("CENTER", board.content, "TOPLEFT", x, -y)
end

-- The facts -----------------------------------------------------------------------------------------------------------

-- One ladder's numbers as the tree reads them on the game's own fold, the shape of the Book's source in
-- tools/apppump.lua: its family open (ns.Open), its rung (the start rung before any), its best (ns.BestRung), its
-- climb's progress (ns.UnlockProgress), the move standing for its rung (ns.RungMove; none on the cardio ladder and on a
-- family not open yet, as the sheet summary's lad), that move's record, and whether a strength ladder is placed (a
-- closed family is no ladder to place: the foot shows the collection's badges, Tree.FootBadges).
function Tree.Source(state, l, f)
	local rung, open = ns.Rung(state, l), ns.Open(state, l)
	local standing = open and l ~= ns.CARDIO and ns.RungMove(l, rung, f) or nil
	local lad = state.ladders and state.ladders[l]
	return { open = open, rung = rung, best = ns.BestRung(state, l),
		progress = { ns.UnlockProgress(state, l, f) }, standing = standing,
		record = standing and ns.Record(state, standing) or 0,
		placed = not STRENGTH[l] or not standing or (lad ~= nil and lad.placedTs ~= nil) }
end

-- Every ladder's source and entry (ns.TreeEntry) and each tree's rows (ns.TreeRows) and glow or reason (ns.TreeGlow
-- with the tree's own Trial ladder, ns.TrialNext; none while felt unwell locks every pump), at `now`.
function Tree.Facts()
	local state, now = ns.state, ns.now()
	local f, setup = ns.FilterContext(ns.profile, state), ns.FilterContext(ns.profile, nil)
	local facts = { f = f, setup = setup, now = now, ladders = {}, entries = {}, trees = {} }
	for l = 1, ns.LADDER_MAX do
		if ns.Ladders[l] and ns.HeldTop(l) > 0 and ns.TreeOf(l) then
			local src = Tree.Source(state, l, f)
			facts.ladders[l], facts.entries[l] = src, ns.TreeEntry(state, l, src, f, setup, now)
		end
	end
	local locked = Lc.Locked()
	for t in ipairs(ns.TREES) do
		local rows, hidden = ns.TreeRows(t, facts.entries)
		local glow, kind, at = ns.TreeGlow(t, facts.entries, ns.TrialNext(state, f, t), locked)
		facts.trees[t] = { rows = rows, hidden = hidden, glow = glow, kind = glow and kind or nil,
			reason = not glow and kind or nil, at = at }
	end
	return facts
end

-- A step's place on the board against its ladder: "waits" (the ladder waits whole), "now" (the player's step),
-- "owned" (below it), "kept" (above it, reached before a step down: ns.BestRung keeps it), "next" (the one above),
-- "ahead".
function Tree.State(src, entry, r)
	if entry.waits then return "waits" end
	if r == src.rung then return "now" end
	if r < src.rung then return "owned" end
	if r <= src.best then return "kept" end
	if r == src.rung + 1 then return "next" end
	return "ahead"
end

-- The move a step shows: the rung's own move the setup allows (ns.UnlockMove), else its anchor; on the player's step
-- the move standing for it (the cardio ladder's counted form nearest its top, as its ring reads it).
function Tree.StepMove(l, r)
	local facts = Tree.facts
	local src = facts and facts.ladders[l]
	if src and r == src.rung then
		local entry = facts.entries[l]
		if entry.move then return entry.move end
		if src.standing then return src.standing end
	end
	local id = ns.UnlockMove(l, r, facts and facts.f or ns.FilterContext(ns.profile, ns.state))
	if id and ns.Library[id].rung == r then return id end
	return ns.Anchor(l, r)
end

-- The links between ladders (a family that opens at other ladders' steps, ns.Ladders' `opens`, a list since slice 42;
-- a rung that waits for another ladder's step, one of its `needs`; an `any` group a link from each of its
-- alternatives), each { from = { ladder, rung }, to = { ladder, rung }, opens, need }, `need` the entry or the `any`
-- group the link stands for, read once.
function Tree.Links()
	if Tree.links then return Tree.links end
	local list = {}
	local function add(n, to, opens)
		for _, e in ipairs(n.any or { n }) do
			list[#list + 1] = { from = { e.ladder, ns.RungAtStep(e.ladder, e.step) }, to = to, opens = opens, need = n }
		end
	end
	for l = 1, ns.LADDER_MAX do
		local lad = ns.Ladders[l]
		if lad and ns.TreeOf(l) then
			for _, o in ipairs(lad.opens or {}) do add(o, { l, 1 }, true) end
			for r, w in ipairs(lad.rungs) do
				for _, n in ipairs(w.needs or {}) do add(n, { l, r }) end
			end
		end
	end
	Tree.links = list
	return list
end

-- The moves of each step by ladder and rung (its own move, its twins and variants on its rung), read once: the
-- search and the card's other moves, by id, a cardio step's forms in road order first (ns.ROADS, slice 54, 2026-10-23).
local ROAD_ORDER = {}
for i, road in ipairs(ns.ROADS) do ROAD_ORDER[road] = i end

local function stepOrder(a, b)
	local ra, rb = ROAD_ORDER[ns.Library[a].road] or 0, ROAD_ORDER[ns.Library[b].road] or 0
	if ra ~= rb then return ra < rb end
	return a < b
end

function Tree.StepMoves(l, r)
	if not Tree.byStep then
		local by = {}
		for id, m in pairs(ns.Library) do
			if m.ladder and m.rung then
				by[m.ladder] = by[m.ladder] or {}
				by[m.ladder][m.rung] = by[m.ladder][m.rung] or {}
				table.insert(by[m.ladder][m.rung], id)
			end
		end
		for _, rungs in pairs(by) do
			for _, ids in pairs(rungs) do table.sort(ids, stepOrder) end
		end
		Tree.byStep = by
	end
	return Tree.byStep[l] and Tree.byStep[l][r] or {}
end

-- Whether what a link asks is done: the step of another ladder a family or a rung waits for (ns.NeedDone, each link its
-- own entry or `any` group, as the card's rows), or the family open whatever its entries (ns.Open: placed, or a rung).
function Tree.LinkDone(link)
	if link.opens and ns.Open(ns.state, link.to[1]) then return true end
	return ns.NeedDone(ns.state, link.need)
end

-- What a refused Start says (docs/ui.md, "The words": a refused Start), before its link: the line, and the link's kind
-- and the page it opens (Your setup's page for the gap, the Body page, the move's Include in pumps). `l` the ladder of
-- the step asked for: a level on another ladder is named with its ladder. The setup's line names its first rule that
-- leaves the move out as Your setup names it (ns.SetupGap): a piece of equipment, a switch it needs on (Floor moves,
-- Room to move) or one it needs off (Quiet mode, the Body switches), each on Your setup's page that holds it.
function Tree.Refusal(why, subject, l)
	if why == "pain" then return L["tree.pain"], "body" end
	local id = subject and subject.move
	if why == "off" and id then return L["tree.off"]:format(moveName(id)), "include", id end
	if not id then return L["ladders.step.shut"] end
	local m = ns.Library[id]
	local sl, sr = subject.ladder or m.ladder, subject.rung or m.rung
	if not (sl and sr) then return L["ladders.step.shut"] end
	local who = L["tree.subject"]:format(ns.Step(sl, sr) or sr)
	if sl ~= l then who = L["tree.subject.ladder"]:format(L["ladder." .. sl], ns.Step(sl, sr) or sr) end
	local gap = ns.SetupGap(ns.Shut(id, Tree.facts.setup))
	if not gap then return L["ladders.step.shut"] end
	local setting = L["gear." .. gap.key]
	if gap.kind == "gear" then return L["tree.needs.gear"]:format(who, setting), "setup", "gear" end
	return L[gap.kind == "off" and "tree.needs.on" or "tree.needs.off"]:format(who, setting), "setup", "place"
end

-- The hover ---------------------------------------------------------------------------------------------------------

-- A rung's window for move `id` only where it counts in the move's own unit (ns.Window's own window, or the unit of
-- the rung's anchor: ns.OwnUnit, as Progress.lua's ring and earn floor read it), so a hold standing for a rep step
-- never shows reps as seconds; nil otherwise.
function Tree.Window(l, rung, id)
	local lo, hi, own = ns.Window(l, rung, id)
	if lo and ns.OwnUnit(id, ns.Anchor(l, rung), own) then return lo, hi end
	return nil
end

-- Whether a cardio rung is a counted one, its clean days its measure (ns.CountedRung, 2026-10-16), else a timed one,
-- its full moves (slice 52; its ready days until then): by the rung, never by a progress that a shut climb leaves at
-- zero.
function Tree.Counted(rung)
	return ns.CountedRung(rung, Tree.facts.f)
end

-- The next stage of the climb from `rung`'s path (ns.PathNext, 2026-10-19: the move the climb's turn runs, as the
-- step's Start and the pumps run it, else what holds the climb), with the move to name, its days and `waits` where it
-- is not open, and its top (ns.MoveTop); nil where the path is in or the rung has none.
function Tree.PathNext(l, rung)
	local n = ns.PathNext(ns.state, l, rung, Tree.facts.f)
	if not n then return nil end
	return n, n.top
end

-- The sentence a next stage held by a wait says after its move (ns.PathNext's `waits`): its check or the move whose
-- top opens its stage, as a whole sentence with its own full stop (Tree.WaitWords' `sentence`: the held lines
-- tree.next.pathHeld and tree.key.pathHeld add none after it, 2026-10-19, the double-check's fix review); nil where
-- the move is open.
function Tree.PathHeldWords(n)
	if not n.waits then return nil end
	local words = Tree.WaitWords({ move = n.move }, n.waits, true)
	return words ~= "" and words or nil
end

-- How many stages of the climb from `rung`'s path are still out (open or waiting): the moves the next step waits for
-- after the main move (the hover's "and 2 more moves", the beads' count).
function Tree.StagesLeft(l, rung)
	local n = 0
	for _, st in ipairs(ns.Stages(ns.state, l, rung, Tree.facts.f)) do
		if st.state == "open" or st.state == "waiting" then n = n + 1 end
	end
	return n
end

-- The player's step's key line once its days are in and its path is not (slice 42; the apps' ladders.measure.proof,
-- .proof.mark, .proof.clean, .proof.held): the path's next stage (Tree.PathNext), its move at its top ("Then Pistol,
-- hands on the desk 8+"), a pose close enough or Perfect on its days, a cardio form's clean days, or, held, its move
-- and what holds it ("Then Knee clap push-up. Waits for a record of 12 on Feet-up push-up.", Tree.PathHeldWords); nil
-- where none is out.
function Tree.PathKey(l, rung)
	local st, top = Tree.PathNext(l, rung)
	if not st then return nil end
	local held = Tree.PathHeldWords(st)
	if held then return L["tree.key.pathHeld"]:format(moveName(st.move), held) end
	if l == ns.CARDIO then
		return ns.Plural("tree.key.pathClean", st.days):format(moveName(st.move), st.have, st.days)
	end
	if POSE[l] then
		return L[top == ns.Enum.mark.perfect and "tree.key.pathPerfect" or "tree.key.pathPose"]:format(moveName(st.move),
			days(st.days))
	end
	return top and L["tree.key.path"]:format(moveName(st.move), Sheet.ClimbTop(st.move, top, true)) or nil
end

-- The key line of the player's step (docs/ui.md, "The hover's lines"), in the small-area framing: the progress made
-- while the record is under the window's middle, what is left past it, then the days, the pips beside them, and once
-- they are in the path's stage still out (Tree.PathKey).
function Tree.NowLine(l, src, entry)
	local p = src.progress
	local have, need, have2 = p[1], p[2], p[3]
	local path = have >= need and have2 == 1 and Tree.PathKey(l, src.rung)
	if path then return path end
	if l == ns.CARDIO then return Tree.NowCardio(src) end
	if POSE[l] then
		local key = ns.MarkNeeds(l, src.rung) == ns.Enum.mark.perfect and "tree.key.perfect" or "tree.key.pose"
		return ns.Plural(key, need):format(have, need) .. " " .. Style.Pips(have, need)
	end
	local id, rec = src.standing, src.record or 0
	if not (entry.ring and id) then return L["tree.key.mark"] end
	-- the record's numbers only where the window counts in the move's own unit; else the days alone
	local lo, hi = Tree.Window(l, src.rung, id)
	local record = lo and L["tree.key.record"]:format(value(id, rec), value(id, lo), value(id, hi))
	local daysLine = ns.Plural("tree.key.daysTop", need):format(have, need) .. " " .. Style.Pips(have, need)
	if lo and rec < lo + (hi - lo) / 2 then return record end
	if lo and rec < hi then return L["tree.key.left"]:format(value(id, hi - rec), days(math.max(0, need - have))) end
	if have < need then return daysLine end
	return record or daysLine
end

-- The cardio ladder's key line (2026-10-16): a timed rung's full moves (slice 52; its ready days until then), a
-- counted one's clean days, and once its days and path are in its proving window, or after a try that did not pass
-- this session that the climb waits (slice 53, ns.Proving), as the Next line says (Tree.NextCardio).
function Tree.NowCardio(src)
	local have, need = src.progress[1], src.progress[2]
	local counted = Tree.Counted(src.rung)
	if counted then
		local proving, form = ns.Proving(ns.state, Tree.facts.f, Tree.facts.now)
		if proving == "waits" then return L["ladders.step.climbWaits"] end
		if proving then return L["tree.next.proving"]:format(moveName(form)) end
	end
	local key = counted and "tree.key.clean" or "tree.key.full"
	return L[key]:format(have, need) .. " " .. Style.Pips(have, need)
end

-- The player's step's record on its range, "Record 22 · Range 20-30": the move standing for it in its own unit
-- (Tree.Window); nil with no record or no range (a cardio step: a count is never a goal since 2026-10-16).
function Tree.RecordKey(l, src)
	if l == ns.CARDIO then return nil end
	local id, rec = src.standing, src.record or 0
	local lo, hi
	if id then lo, hi = Tree.Window(l, src.rung, id) end
	if not (lo and rec > 0) then return nil end
	return L["tree.key.record"]:format(value(id, rec), value(id, lo), value(id, hi))
end

-- What holds a climb that is shut below the ladder's top (ns.UnlockProgress's need 0 there): the step of another
-- ladder the next step waits for (Tree.WaitsFor), else what of the setup shuts it or holds this step with a stand-in
-- from below, in the top-out line's words (ns.TopOut, Sheet.TopOutLine), else that the next step needs other
-- equipment or switches. `rows`: the card's, whose need rows each say what they wait for (Tree.CardNeeds), so a
-- climb that waits for another ladder's step gets no line of its own there (nil); the hover, with no rows, says it.
function Tree.ClosedLine(l, rung, rows)
	local waits = Tree.WaitsFor(l, rung + 1)
	if waits then return not rows and waits or nil end
	local shut, why = ns.TopOut(l, rung, Tree.facts.f, ns.state)
	return shut and Sheet.TopOutLine(l, rung, shut, why) or L["ladders.step.shut"]
end

-- What opens the step above rung `from` (the hover's "next" and a kept step's "Comes back at" from the player's step;
-- on a step further up the Path's need, the climb from the step under it): the move standing for `from` at its top on
-- the days its climb takes; a pose's days close enough (Perfect ones where only Perfect counts); a timed cardio rung's
-- full moves (slice 52; ready days until then); a counted one's clean days (2026-10-16); from the player's step with
-- the moves of its path still out after it ("… and 2 more moves", slice 42: a card's road gives every stage a row of
-- its own, Tree.CardRoad). `p` the climb's numbers from the player's step (ns.UnlockProgress), nil from a step above
-- it, whose need is the rule's own (the Book's Path, tools/apppump.lua `need`: pose_unlock_days, cardio_full_days,
-- ns.CleanDays). `key` the line's
-- family: "tree.key.next" or "tree.key.back". A climb that is shut says what holds it instead (Tree.KeyLines).
function Tree.OpensLine(l, from, p, key)
	local id = Tree.StepMove(l, from)
	local line
	if l == ns.CARDIO then
		line = Tree.Counted(from) and L[key .. "Clean"]:format(p and p[2] or ns.CleanDays(from))
			or L[key .. "Full"]:format(p and p[2] or C.cardio_full_days)
	elseif POSE[l] then
		local need = p and p[2] or C.pose_unlock_days
		line = ns.MarkNeeds(l, from) == ns.Enum.mark.perfect and ns.Plural(key .. "Perfect", need):format(need)
			or L[key .. "Pose"]:format(days(need))
	else
		local hi
		if id then hi = select(2, Tree.Window(l, from, id)) end
		if not hi then return L["ladders.step.shut"] end
		line = L[key]:format(moveName(id), Sheet.ClimbTop(id, hi, true), days(ns.UnlockDays(l, from, ns.state)))
	end
	local left = p and Tree.StagesLeft(l, from) or 0
	if left > 0 then line = ns.Plural("tree.key.andMore", left):format(line, left) end
	return line
end

-- The line a step's own Start's refusal or a climb that waits gives it (shut, off, pain, waits for another ladder's
-- step), else nil: the hover's line 3 for such a step, whatever its state.
function Tree.HeldLine(l, r, src, entry, start, why, subject)
	if not start and why == "pain" then return L["ladders.step.pain"] end
	if not start and why then return (Tree.Refusal(why, subject, l)) end
	if entry.pain and r > src.rung then return L["ladders.step.pain"] end
	if r == src.rung + 1 then return Tree.WaitsFor(l, r) end
	return nil
end

-- The step of another ladder rung `r`'s climb waits for and has not got yet (ns.RungNeed), named as the card names it
-- (ns.NeedAt with the player's filter: an `any` group's entry the setup runs, so a setup that shuts the tuck jump reads
-- step 11's squat step, slice 54), in the ladder pages' words (Tree.WaitsWords); nil where it waits for none.
function Tree.WaitsFor(l, r)
	local _, _, entry = ns.RungNeed(ns.state, l, r)
	if not entry then return nil end
	local need, count = ns.NeedAt(entry, Tree.facts and Tree.facts.f or ns.FilterContext(ns.profile, ns.state))
	return need and Tree.WaitsWords(need, count) or nil
end

-- What a step of another ladder waits for, the one wording of it (the hover, the card's shut climb, a via start, each
-- need row): a record of move `id` that does it, `count` in its own unit, or a pose or cardio step's move `id` on its
-- ladder (no count: ns.NeedAt's and ns.RungNeed's answer).
function Tree.WaitsWords(id, count)
	if count then return L["ladders.step.waits"]:format(value(id, count), moveName(id)) end
	return L["ladders.step.waits.step"]:format(moveName(id), L["ladder." .. ns.Library[id].ladder])
end

-- What still holds what a stretch check opens (`opens`, as the checks' readers hold it: Tree.AllChecks), in that
-- wait's own words as the card writes it (the pump's Result under a check passed whose move or climb is not open yet,
-- 2026-10-17, slice 42's review; `why`: ns.OpensNow's second answer, ns.MoveOpen's for a move): a stage of a step's
-- path its "Opens after {move} at {goal}" (Tree.OpensAfter), a family's first entry not done its sentence
-- (Tree.Opens), a stretch check its row's "{pose}: Good or better, once" (Tree.CheckNeed: the alternative the setup
-- offers), a step of another ladder what it waits for (Tree.WaitsWords, a sentence); "" where none names it. With
-- `sentence`, a move's wait as a whole sentence with its own full stop, "Opens after {move} at {goal}." and "Opens
-- after your first Good hold of {pose}." (a held line's, Tree.PathHeldWords: no code adds or strips a full stop).
function Tree.WaitWords(opens, why, sentence)
	if why.kind == "path" then return Tree.OpensAfter(why.after, why.top, sentence) or "" end
	if opens.family then return Tree.Opens(opens.family, true) or "" end
	local e = Tree.Alternative(why.need)
	if ns.IsCheck(e) then return Tree.CheckNeed(e, sentence) end
	local id, count = ns.NeedAt(why.need, ns.FilterContext(ns.profile, ns.state))
	return id and Tree.WaitsWords(id, count) or ""
end

-- What finds the step of a ladder back from a break again (ns.TreeEntry's `placing`), as its key line (`key`) or its
-- card says it: a strength ladder's next set, the cardio ladder's next all-out window or steady move, whichever
-- measures its step (`placingBy`).
function Tree.PlacingLine(entry, key)
	if entry.placingBy == "window" then return key and L["tree.key.placingWindow"] or L["tree.card.placingWindow"] end
	if entry.placingBy == "steady" then return key and L["tree.key.placingSteady"] or L["tree.card.placingSteady"] end
	return key and L["tree.key.placing"] or L["tree.card.placing"]
end

-- The player's step's key line: not placed, back from a break, at the ladder's top, held (its own Start refused, a
-- pain flag), to earn, its climb shut (Tree.ClosedKey), else its measure (Tree.NowLine).
function Tree.NowKey(l, r, src, entry, held)
	if not src.placed then return L["tree.key.unplaced"] end
	if entry.placing then return Tree.PlacingLine(entry, true) end
	if src.rung >= ns.HeldTop(l) then return L["ladders.path.top"] end
	if held then return held end
	if entry.pain then return L["ladders.step.pain"] end
	local earn, move, floor = ns.ToEarn(ns.state, l, Tree.facts.f)
	if earn == r and floor then return L["ladders.step.toEarn"]:format(moveName(move), value(move, floor)) end
	if earn == r and POSE[l] then return L["ladders.step.toEarnMark"] end
	if src.progress[2] == 0 then return Tree.ClosedKey(l, src) end
	return Tree.NowLine(l, src, entry)
end

-- The player's step's key line while its climb is shut below the top: its record on its range where it has one, then
-- what holds the climb, never a count of days the climb does not count.
function Tree.ClosedKey(l, src)
	local holds = Tree.ClosedLine(l, src.rung)
	local record = Tree.RecordKey(l, src)
	return record and L["tree.key.held"]:format(record, holds) or holds
end

-- The hover's lines 2 and 3 for step (l, r) of an open ladder (docs/ui.md, "The hover's lines"), with the step's state:
-- line 3 the step's held line (its Start refused, a pain flag, the next step waiting for another ladder's) in any
-- state but the player's own, whose key line weighs it beside the others (Tree.NowKey); the step above the player's
-- what opens it, or while the climb is shut what holds it.
function Tree.KeyLines(l, r, start, why, subject)
	local facts = Tree.facts
	local src, entry = facts.ladders[l], facts.entries[l]
	local state = Tree.State(src, entry, r)
	local q, step = ns.Quality(l, r), ns.Step(l, r)
	local standing = state
	if state == "now" and not src.placed then standing = "unplaced" end
	local two = L["tree.step"]:format(Sheet.QualityWord(q), step, L["tree.standing." .. standing])
	-- a step mastered says so after the standing in every state that is the player's (slice 42)
	if (state == "now" or state == "owned" or state == "kept") and ns.Mastered(ns.state, l, r) then
		two = two .. " · " .. L["tree.mastered"]
	end
	local held = Tree.HeldLine(l, r, src, entry, start, why, subject)
	local three
	if state == "now" then
		three = Tree.NowKey(l, r, src, entry, held)
	elseif held then
		three = held
	elseif state == "owned" then
		local at = ns.TopTier(l, r) and ns.RungEarned(ns.state, l, r) or ns.RungReached(ns.state, l, r)
		three = type(at) == "number" and L["ladders.step.earned"]:format(Sheet.ShortDate(ns.DayMid(at, 0)))
			or L[at and "ladders.step.kept" or "ladders.step.start"]
	elseif r == src.rung + 1 and src.progress[2] == 0 then
		three = Tree.ClosedLine(l, src.rung)
	elseif r == src.rung + 1 then
		three = Tree.OpensLine(l, src.rung, src.progress, state == "next" and "tree.key.next" or "tree.key.back")
	else
		three = L[state == "kept" and "tree.key.backAfter" or "tree.key.ahead"]:format(moveName(Tree.StepMove(l, r - 1)))
	end
	return two, three, state
end

-- The hover's lines of a ladder that waits whole (closed, setup or off): its name, its note, its sentence.
function Tree.WaitsLines(l, entry)
	local note = L["tree.waits." .. entry.waits]
	if entry.waits == "closed" then return note, Tree.Opens(l) end
	-- the level ns.TreeEntry found shut: the anchor of the ladder's rung (its first before any)
	local r = math.max(1, entry.rung or 1)
	return note, (Tree.Refusal(entry.waits, { move = ns.Anchor(l, r), ladder = l, rung = r }, l))
end

-- What opens a family not open yet, its first entry not done (ns.OpensAt with the state; a list since slice 42): a
-- strength step's anchor and count, "Handstand opens at a Push-up record of 15", a step of another ladder, "Power
-- opens at Cardio step 5", or a stretch check, its pose's words as its check row says them ("Wrist lean: Perfect,
-- once": Tree.CheckNeed; the card's check row runs it); as the card's sentence with `sentence`; nil with every entry
-- done.
function Tree.Opens(l, sentence)
	if not ns.Ladders[l].opens then return nil end
	local id, count, o = ns.OpensAt(l, ns.state)
	if not o then return nil end
	if ns.IsCheck(o) then return Tree.CheckNeed(o) end
	if id and count then
		return L[sentence and "tree.opens.record" or "tree.opensRecord"]:format(L["ladder." .. l], moveName(id),
			value(id, count))
	end
	local key = sentence and "tree.opens.step" or "tree.opensStep"
	return L[key]:format(L["ladder." .. l], L["ladder." .. o.ladder], o.step)
end

-- The four lines of a step's hover (GameTooltip, ANCHOR_RIGHT): the move in its quality's colour (a waiting
-- ladder's name), its quality, step and where the player stands, the key line, and what a click does: closes the card
-- the step shows, else opens it, "and Start" only where its Start can run now (Lc.StartBlock: none while the setup
-- waits, felt unwell, in a fight, a battleground or arena, dead, or while another pump waits).
function Tree.Hover(owner)
	local l, r = owner.ladder, owner.rung
	local facts = Tree.facts
	if not (facts and facts.entries[l]) then return end
	local entry = facts.entries[l]
	local start, why, subject = ns.StartFor(ns.state, ns.profile, { ladder = l, rung = r }, facts.now)
	GameTooltip:SetOwner(owner, "ANCHOR_RIGHT")
	local two, three
	if entry.waits then
		GameTooltip_SetTitle(GameTooltip, L["ladder." .. l])
		two, three = Tree.WaitsLines(l, entry)
	else
		GameTooltip_SetTitle(GameTooltip, moveName(Tree.StepMove(l, r)), Sheet.QualityColor(ns.Quality(l, r)))
		two, three = Tree.KeyLines(l, r, start, why, subject)
	end
	GameTooltip:AddLine(two, 1, 1, 1, true)
	GameTooltip:AddLine(three, NORMAL_FONT_COLOR.r, NORMAL_FONT_COLOR.g, NORMAL_FONT_COLOR.b, true)
	local click = (start and Lc.StartBlock() == nil) and "tree.click.start" or "tree.click.card"
	if Tree.CardOn(l, r) then click = "tree.click.close" end
	GameTooltip_AddInstructionLine(GameTooltip, L[click])
	GameTooltip:Show()
end

-- The board ---------------------------------------------------------------------------------------------------------

-- A tree's columns: each ladder's centre x on the board and the pitch, the branches a gap apart, centred.
function Tree.Columns(t)
	local tree = ns.TREES[t]
	local n = #ns.TreeLadders(t)
	local room = BOARD_W - GUTTER - EDGE - (#tree.branches - 1) * BRANCH_GAP
	local pitch = math.min(PITCH_MAX, math.floor(room / n))
	local x = GUTTER + math.floor((BOARD_W - GUTTER - EDGE - n * pitch - (#tree.branches - 1) * BRANCH_GAP) / 2)
	local cols = {}
	for _, branch in ipairs(tree.branches) do
		for _, l in ipairs(branch.ladders) do
			cols[l] = x + pitch / 2
			x = x + pitch
		end
		x = x + BRANCH_GAP
	end
	return cols, pitch
end

-- The rows from the top: each step's top, its tiles' centre y, its height and whether it shows in full (`rows`,
-- ns.TreeRows, or every one while the tree is expanded), the quality dividers where a row's quality changes, and the
-- board's height. A full row that holds a ringed step with beads (`bands`, by step: Tree.BeadSteps) grows by the beads'
-- band under its tiles (treeBeadRow, slice 42), its tiles where every full row has them.
function Tree.Rows(rows, expanded, bands)
	local y, at, dividers, last = TOP_PAD, {}, {}, nil
	for step = 1, #C.quality_by_step do
		local q = C.quality_by_step[step]
		if q ~= last then
			dividers[#dividers + 1] = { y = y, quality = q }
			y, last = y + DIVIDER_H, q
		end
		local full = expanded or rows[step] == true
		local h = full and ROW_H or THIN_H
		local band = full and bands[step] == true
		at[step] = { top = y, y = y + h / 2, h = h + (band and BEAD_ROW or 0), full = full, band = band }
		y = y + at[step].h
	end
	return at, dividers, y
end

-- A node: the sheet's slot, a step's tile, hovered and clicked as one.
local function nodeEnter(self)
	Tree.Hover(self)
	Tree.DrawLinks(Tree.boards[Tree.tree], self.ladder, self.rung)
end

local function nodeLeave()
	GameTooltip:Hide()
	local sel = Tree.selected
	Tree.DrawLinks(Tree.boards[Tree.tree], sel and sel[1], sel and sel[2])
end

-- A click on a step opens its card; on the step whose card shows it closes the card again, as its close button does
-- (David, 2026-10-15: "clicking again on an excercise should close the details window again"); the hover still under
-- the pointer then says what the next click does, and previews the step's links again.
local function nodeClick(self)
	Tree.Land()
	if Tree.CardOn(self.ladder, self.rung) then Tree.HideCard() else Tree.ShowCard(self.ladder, self.rung) end
	if self:IsMouseOver() then nodeEnter(self) end
end

function Tree.BuildNode(board, l, r)
	local b = Sheet.SlotButton(board.content)
	b:SetFrameLevel(board.content:GetFrameLevel() + 3)
	b.ladder, b.rung = l, r
	b:SetScript("OnEnter", nodeEnter)
	b:SetScript("OnLeave", nodeLeave)
	b:SetScript("OnClick", nodeClick)
	b.match = b:CreateTexture(nil, "OVERLAY", nil, 6)
	b.match:SetPoint("CENTER")
	b.match:SetSize(SLOT + 22, SLOT + 22)
	b.match:Hide()
	b.seal = Tree.Seal(b)
	return b
end

-- A mastered step's seal (slice 42, MoveTile's `mastered`): inside the tile's bottom-right corner, over its quality
-- border, the Adventures table's gold check in its ring (Style.atlas.mastered), the kit's check where a client lacks
-- it. Built hidden.
function Tree.Seal(b)
	local seal = b:CreateTexture(nil, "OVERLAY", nil, 7)
	seal:SetSize(SEAL, SEAL)
	seal:SetPoint("BOTTOMRIGHT", -2, 2)
	if not Style.SetAtlas(seal, "mastered") then Style.SetMark(seal, true) end
	seal:Hide()
	return seal
end

-- A thin row's step, or a step of a ladder that waits whole: a dot on the ladder's line with a hit area of DOT_HIT.
function Tree.BuildDot(board, l, r)
	local d = CreateFrame("Button", nil, board.content)
	d:SetSize(DOT_HIT, DOT_HIT)
	d:SetFrameLevel(board.content:GetFrameLevel() + 3)
	d.ladder, d.rung = l, r
	d.fill = d:CreateTexture(nil, "ARTWORK")
	d.fill:SetSize(DOT, DOT)
	d.fill:SetPoint("CENTER")
	d.mask = d:CreateMaskTexture()
	d.mask:SetAllPoints(d.fill)
	if Style.SetAtlas(d.mask, "circle") then d.fill:AddMaskTexture(d.mask) end
	d.match = d:CreateTexture(nil, "OVERLAY")
	d.match:SetPoint("CENTER")
	d.match:SetSize(DOT_HIT, DOT_HIT)
	d.match:Hide()
	d:SetScript("OnEnter", nodeEnter)
	d:SetScript("OnLeave", nodeLeave)
	d:SetScript("OnClick", nodeClick)
	return d
end

-- The ring round a ladder's step: the track (the yellow node border, dimmed) and its fill by the client's radial
-- progress (12.1.0's TextureBase:SetRadialProgressBarPercent, on Forever 1.60.1 and retail 12.1.5; Gethe `forever`
-- 70374 and `live` 70391, 2026-10-15) animated by a RadialProgress animation, or where a client lacks it a Cooldown
-- paused at the fill (CooldownFrame_SetDisplayAsPercentage), with the star ticks.
function Tree.BuildRing(board)
	local content = board.content
	local ring = { ticks = {} }
	ring.track = content:CreateTexture(nil, "ARTWORK", nil, 2)
	ring.track:SetSize(RING, RING)
	if Style.SetAtlas(ring.track, "nodeRing") then
		ring.track:SetDesaturated(true)
		ring.track:SetAlpha(0.35)
	else
		ring.track:SetColorTexture(0, 0, 0, 0.6)
	end
	ring.fill = content:CreateTexture(nil, "ARTWORK", nil, 3)
	ring.fill:SetAllPoints(ring.track)
	if not Style.SetAtlas(ring.fill, "nodeRing") then ring.fill:SetColorTexture(1, 0.82, 0, 0.9) end
	ring.radial = ring.fill.SetRadialProgressBarPercent and true or false
	if ring.radial then
		ring.anim = ring.fill:CreateAnimationGroup()
		-- the RadialProgress animation is 12.1.0's too (UI.xsd); a client that refuses the type fills at once
		local ok, a = pcall(ring.anim.CreateAnimation, ring.anim, "RadialProgress")
		ring.sweep = ok and a or nil
		if ring.sweep then
			ring.sweep:SetSmoothing("OUT")
			ring.anim:SetScript("OnFinished", function() ring.fill:SetRadialProgressBarPercent(ring.to or 0) end)
		end
	else
		ring.fill:Hide()
		ring.cd = CreateFrame("Cooldown", nil, content)
		ring.cd:SetAllPoints(ring.track)
		ring.cd:SetFrameLevel(content:GetFrameLevel() + 2)
		ring.cd:SetReverse(true)
		ring.cd:SetDrawEdge(false)
		ring.cd:SetDrawBling(false)
		ring.cd:SetHideCountdownNumbers(true)
		if not Style.SetSwipeAtlas(ring.cd, "nodeRing") then ring.cd:SetSwipeColor(1, 0.82, 0, 0.9) end
	end
	return ring
end

-- The ring's fill, 0 to 1: at once, or from `from` over FILL_S after HOLD_S (the return's).
function Tree.FillRing(ring, to, from)
	ring.to = to
	if not ring.radial then
		ring.cd:Show()
		CooldownFrame_SetDisplayAsPercentage(ring.cd, to)
		return
	end
	ring.anim:Stop()
	if from and ring.sweep and from ~= to then
		ring.fill:SetRadialProgressBarPercent(from)
		ring.sweep:SetFromPercent(from)
		ring.sweep:SetToPercent(to)
		ring.sweep:SetStartDelay(HOLD_S)
		ring.sweep:SetDuration(FILL_S)
		ring.anim:Play()
	else
		ring.fill:SetRadialProgressBarPercent(to)
	end
end

-- A tick at `t` per mille of the ring, from its start at the bottom (TextureBase's radial origin) round clockwise, on
-- the square's edge (fixed math on the node's own centre).
function Tree.PlaceTick(board, tick, cx, cy, t)
	local a = 2 * math.pi * t / 1000
	local dx, dy = -math.sin(a), -math.cos(a)
	local k = (RING / 2) / math.max(math.abs(dx), math.abs(dy))
	pin(board, tick, cx + dx * k, cy - dy * k)
	tick:Show()
end

function Tree.BuildBoard(t)
	local page = Tree.page
	local board = { t = t, nodes = {}, dots = {}, rings = {}, lines = {}, diamonds = {}, heads = {}, numbers = {},
		dividers = {}, links = {}, chips = {}, notes = {}, beads = {} }
	-- placed and sized at each draw, from whether the column names and the foot show (Tree.Render)
	board.scroll = Sheet.ScrollList(page, BOARD_W, PAGE_H - NAMES_TOP - 30)
	board.top = NAMES_TOP
	local content = board.scroll.content
	board.content = content
	-- a click on the board's ground (Tree.Ground); ConsolePort's cursor passes the ground by and keeps its steps (its
	-- `nodepass` attribute: "include children, skip node", seblindfors/ConsolePortNode b86ed651), so no stop of the
	-- cursor lands between them
	content:EnableMouse(true)
	content:SetAttribute("nodepass", true)
	content:SetScript("OnMouseDown", function() Tree.Ground() end)
	board.header = CreateFrame("Frame", nil, page)
	board.header:SetPoint("TOPLEFT", BOARD_X, -HEAD_NAMES_Y)
	board.header:SetSize(BOARD_W, NAMES_TOP - HEAD_NAMES_Y)
	board.cols, board.pitch = Tree.Columns(t)
	for _, l in ipairs(ns.TreeLadders(t)) do
		board.nodes[l], board.dots[l] = {}, {}
		for r in ipairs(ns.Ladders[l].rungs) do
			board.nodes[l][r] = Tree.BuildNode(board, l, r)
			board.dots[l][r] = Tree.BuildDot(board, l, r)
		end
		local earned, ahead = content:CreateLine(nil, "ARTWORK"), content:CreateLine(nil, "ARTWORK")
		earned:SetThickness(LINE_W)
		ahead:SetThickness(LINE_W)
		if not Style.SetAtlas(earned, "arrowLine") then earned:SetColorTexture(1, 0.82, 0, 0.8) end
		-- the flight map's dashes tile along the line (its atlas tiles horizontally; to see in the client)
		if Style.SetAtlas(ahead, "aheadLine") then ahead:SetHorizTile(true) else ahead:SetColorTexture(0.5, 0.5, 0.5, 0.6) end
		board.lines[l] = { earned = earned, ahead = ahead }
		local d = content:CreateTexture(nil, "ARTWORK", nil, 4)
		d:SetSize(10, 10)
		d:SetColorTexture(1, 1, 1, 1)
		d.mask = content:CreateMaskTexture()
		d.mask:SetAllPoints(d)
		if Style.SetAtlas(d.mask, "diamond") then d:AddMaskTexture(d.mask) end
		board.diamonds[l] = d
		board.rings[l] = Tree.BuildRing(board)
		board.heads[l] = Tree.BuildHeader(board, l)
	end
	for step = 1, #C.quality_by_step do
		board.numbers[step] = Kit.Text(content, "label", "CENTER", saga.note)
	end
	board.glow = Style.ProcGlow(content, 0, 0)
	board.glow:SetSize(GLOW, GLOW)
	-- the return's flash over the node it lights: a frame above the nodes (a scroll child's own texture draws under
	-- its child frames), taking no mouse
	board.over = CreateFrame("Frame", nil, content)
	board.over:SetAllPoints()
	board.over:SetFrameLevel(content:GetFrameLevel() + 4)
	-- a waiting ladder's note at the top of its dashed column (its ladder's name stands over the board), within its
	-- column, two lines at most, on the board's ground so the column's line breaks under its words; the frame over the
	-- steps takes no mouse, so the dots under it still give their tooltip
	for _, l in ipairs(ns.TreeLadders(t)) do
		local note = { ground = board.over:CreateTexture(nil, "ARTWORK"),
			words = Kit.Text(board.over, "label", "CENTER", saga.note, board.pitch - 4) }
		note.ground:SetColorTexture(unpack(saga.screen))
		note.words:SetMaxLines(2)
		board.notes[l] = note
	end
	board.flash = board.over:CreateTexture(nil, "OVERLAY")
	board.flash:SetSize(SLOT, SLOT)
	board.flash:SetColorTexture(1, 1, 1, 1)
	board.flash:SetBlendMode("ADD")
	board.flash:SetAlpha(0)
	board.flashAnim = board.flash:CreateAnimationGroup()
	local up = board.flashAnim:CreateAnimation("Alpha")
	up:SetFromAlpha(0)
	up:SetToAlpha(0.6)
	up:SetDuration(FLASH_S / 3)
	up:SetOrder(1)
	local down = board.flashAnim:CreateAnimation("Alpha")
	down:SetFromAlpha(0.6)
	down:SetToAlpha(0)
	down:SetDuration(FLASH_S * 2 / 3)
	down:SetOrder(2)
	board.glowAnim = board.glow:CreateAnimationGroup()
	board.glowAnim:SetToFinalAlpha(true) -- it ends at the glow's own alpha
	local fade = board.glowAnim:CreateAnimation("Alpha")
	fade:SetFromAlpha(0)
	fade:SetToAlpha(GLOW_A)
	fade:SetDuration(GLOW_S)
	-- the steps ahead behind the More section's bar (the kit's), which opens them once for the tree: no red on the page
	board.expander = Kit.More(content, BOARD_W - 2 * GUTTER, nil, function() end)
	board.expander.onToggle = function()
		Tree.expanded[t] = true
		Tree.Render()
	end
	-- the cross-tree chips (the kit's link chips: its cue plays at the click, so the tree's switch plays none)
	for i = 1, CHIPS do
		board.chips[i] = Kit.LinkChip(content, "", function(self) Tree.ShowStep(self.ladder, self.rung, true) end)
		board.chips[i]:SetFrameLevel(content:GetFrameLevel() + 5)
		board.chips[i].words:SetWordWrap(false)
		board.chips[i]:Hide()
	end
	Tree.boards[t] = board
	return board
end

-- A column's header over the board: the ladder's name (sentence case, the notes' grey) where every column holds its
-- name on one line, centred over its column; its right-click starts the ladder over (Settings.lua asks first), named
-- in its tooltip. A waiting ladder's note stands at the top of its column on the board (BuildBoard's notes).
function Tree.BuildHeader(board, l)
	local h = CreateFrame("Button", nil, board.header)
	h:SetSize(board.pitch, NAMES_TOP - HEAD_NAMES_Y)
	h:SetPoint("TOP", board.header, "TOPLEFT", board.cols[l], 0)
	h.ladder = l
	h.name = Kit.Text(h, "label", "CENTER", saga.note)
	h.name:SetPoint("TOP")
	h:RegisterForClicks("RightButtonUp")
	h:SetScript("OnClick", function(self) ns.Settings.RestartMenu(self, self.ladder) end)
	h:SetScript("OnEnter", function(self)
		local entry = Tree.facts and Tree.facts.entries[self.ladder]
		GameTooltip:SetOwner(self, "ANCHOR_TOP")
		GameTooltip_SetTitle(GameTooltip, L["ladder." .. self.ladder])
		if entry and entry.waits then GameTooltip:AddLine(select(2, Tree.WaitsLines(self.ladder, entry)), 1, 1, 1, true) end
		if ns.Settings.CanRestart(self.ladder) then GameTooltip_AddInstructionLine(GameTooltip, L["ladders.tip.restart"]) end
		GameTooltip:Show()
	end)
	h:SetScript("OnLeave", function() GameTooltip:Hide() end)
	return h
end

-- A search match's outline: Forever's talent search outline (exact on the step's own move), else a gold square.
function Tree.Outline(tex, match)
	if not Style.SetAtlas(tex, match == "exact" and "searchExact" or "searchMatch") then
		tex:SetColorTexture(1, 0.82, 0, 0.35)
	end
end

-- One node or dot as its step stands (Tree.State), its quality as Blizzard's border, a step ahead greyed; a search
-- match outlined and the rest dimmed while a search is on. The selected step shows it: a node by its locked
-- highlight, a dot by the outline a match has (docs/ui.md: the selected step's outline and Find a move's are one mark).
function Tree.RenderStep(board, l, r, row, src, entry)
	local node, dot = board.nodes[l][r], board.dots[l][r]
	local state = Tree.State(src, entry, r)
	local x = board.cols[l]
	local q = ns.Quality(l, r)
	local match = Tree.matches and Tree.matches[l] and Tree.matches[l][r]
	local alpha = (Tree.matches and not match) and 0.4 or 1
	local full = row.full and not entry.waits
	node:SetShown(full)
	dot:SetShown(not full)
	local sel = Tree.selected
	local picked = sel and sel[1] == l and sel[2] == r
	if full then
		pin(board, node, x, row.y)
		local file, lt, rt, tp, bt = Style.MoveIcon(Tree.StepMove(l, r))
		Sheet.SetIcon(node, file, lt, rt, tp, bt)
		SetItemButtonQuality(node, q)
		SetItemButtonDesaturated(node, state == "next" or state == "ahead")
		node:SetAlpha(alpha)
		node.seal:SetShown(ns.Mastered(ns.state, l, r) ~= nil)
		if picked then node:LockHighlight() else node:UnlockHighlight() end
		node.match:SetShown(match ~= nil)
		if match then Tree.Outline(node.match, match) end
	else
		pin(board, dot, x, row.y)
		local owned = state == "now" or state == "owned" or state == "kept"
		local c = owned and Sheet.QualityColor(q) or DISABLED_FONT_COLOR
		dot.fill:SetColorTexture(c.r, c.g, c.b, 1)
		dot:SetAlpha(alpha)
		dot.match:SetShown(match ~= nil or picked == true)
		if match or picked then Tree.Outline(dot.match, match or "exact") end
	end
end

-- A ladder's column: its steps, its line (solid to its best step, dashed beyond; a ladder that waits whole dashed all
-- through), its diamond in its top step's colour, its ring on the player's step.
function Tree.RenderColumn(board, l, rowsAt, facts)
	local src, entry = facts.ladders[l], facts.entries[l]
	local x = board.cols[l]
	local rungs = ns.Ladders[l].rungs
	for r, w in ipairs(rungs) do Tree.RenderStep(board, l, r, rowsAt[w.step], src, entry) end
	local first, last = rowsAt[rungs[1].step], rowsAt[rungs[#rungs].step]
	local split = entry.waits and 0 or math.max(src.best, src.rung)
	local earned, ahead = board.lines[l].earned, board.lines[l].ahead
	local mid = split > 0 and rowsAt[rungs[math.min(split, #rungs)].step]
	earned:SetShown(mid ~= false and mid ~= nil and split > 1)
	if mid and split > 1 then
		earned:SetStartPoint("TOPLEFT", board.content, x, -first.y)
		earned:SetEndPoint("TOPLEFT", board.content, x, -mid.y)
	end
	ahead:SetShown(split < #rungs)
	if split < #rungs then
		ahead:SetStartPoint("TOPLEFT", board.content, x, -(mid and mid.y or first.y))
		ahead:SetEndPoint("TOPLEFT", board.content, x, -last.y)
	end
	local d = board.diamonds[l]
	pin(board, d, x, last.y + (last.full and not entry.waits and SLOT / 2 or DOT / 2) + 7)
	local c = Sheet.QualityColor(ns.Quality(l, #rungs))
	d:SetVertexColor(c.r, c.g, c.b)
	Tree.RenderNote(board, l, first, entry)
	Tree.RenderRing(board, l, rowsAt, src, entry)
	local w = ns.Ladders[l].rungs[src.rung]
	Tree.RenderBeads(board, l, w and rowsAt[w.step], src, entry)
end

-- A waiting ladder's note ("Not open yet", "Shut by your setup", "Left out of your pumps") at the top of its column,
-- from the top of its first step's row, on its ground; none on a ladder that waits for nothing.
function Tree.RenderNote(board, l, first, entry)
	local note = board.notes[l]
	local shown = entry.waits ~= nil
	note.words:SetShown(shown)
	note.ground:SetShown(shown)
	if not shown then return end
	note.words:SetText(L["tree.waits." .. entry.waits])
	note.words:ClearAllPoints()
	note.words:SetPoint("TOP", board.content, "TOPLEFT", board.cols[l], -(first.top + 2))
	local w = math.min(board.pitch - 4, ns.Readable(note.words:GetStringWidth()) or board.pitch - 4)
	note.ground:ClearAllPoints()
	note.ground:SetPoint("TOP", note.words, "TOP", 0, 2)
	note.ground:SetSize(w + 4, (ns.Readable(note.words:GetStringHeight()) or 12) + 4)
end

function Tree.RenderRing(board, l, rowsAt, src, entry)
	local ring = board.rings[l]
	local row = rowsAt[ns.Ladders[l].rungs[src.rung] and ns.Ladders[l].rungs[src.rung].step or 1]
	local shown = not (entry.waits or entry.placing) and row and row.full
	ring.track:SetShown(shown == true)
	ring.fill:SetShown(shown == true and ring.radial)
	if ring.cd then ring.cd:SetShown(shown == true) end
	for _, tick in ipairs(ring.ticks) do tick:Hide() end
	if not shown then return end
	local x = board.cols[l]
	pin(board, ring.track, x, row.y)
	local motion = Tree.motion and Tree.motion.rings and Tree.motion.rings[l]
	Tree.FillRing(ring, (entry.ring or 0) / 1000, motion)
	for i, t in ipairs(entry.ticks or {}) do
		if not ring.ticks[i] then
			ring.ticks[i] = board.content:CreateTexture(nil, "OVERLAY", nil, 2)
			ring.ticks[i]:SetSize(3, 3)
			ring.ticks[i]:SetColorTexture(0.1, 0.08, 0.02, 1)
		end
		Tree.PlaceTick(board, ring.ticks[i], x, row.y, t)
	end
end

-- The rows' furniture: the step numbers in the gutter on the rows that show in full (the notes' grey), the quality
-- dividers with their word in its colour (so no quality rests on colour alone; the numeral too under the colour-blind
-- mode, Kit.Quality) on a ground that breaks the column lines under it, the expander under the board (the kit's More
-- bar).
function Tree.RenderRows(board, rowsAt, dividers, height, hidden)
	for step, number in ipairs(board.numbers) do
		local row = rowsAt[step]
		number:SetShown(row.full)
		if row.full then
			number:SetText(step)
			pin(board, number, GUTTER / 2, row.y)
		end
	end
	for i, d in ipairs(dividers) do
		local div = board.dividers[i]
		if not div then
			div = { word = Kit.Text(board.content, "label", "LEFT"),
				ground = board.content:CreateTexture(nil, "ARTWORK", nil, 5),
				line = board.content:CreateTexture(nil, "BACKGROUND") }
			div.ground:SetColorTexture(unpack(saga.screen))
			div.line:SetHeight(1)
			div.line:SetColorTexture(1, 1, 1, 0.12)
			board.dividers[i] = div
		end
		Kit.Quality(div.word, d.quality)
		div.word:ClearAllPoints()
		div.word:SetPoint("LEFT", board.content, "TOPLEFT", 4, -(d.y + DIVIDER_H / 2))
		div.ground:ClearAllPoints()
		div.ground:SetPoint("TOPLEFT", div.word, "TOPLEFT", -3, 1)
		div.ground:SetPoint("BOTTOMRIGHT", div.word, "BOTTOMRIGHT", 6, -1)
		div.line:ClearAllPoints()
		div.line:SetPoint("LEFT", div.word, "RIGHT", 6, 0)
		div.line:SetPoint("RIGHT", board.content, "TOPRIGHT", -EDGE, -(d.y + DIVIDER_H / 2))
		div.word:Show()
		div.ground:Show()
		div.line:Show()
	end
	for i = #dividers + 1, #board.dividers do
		board.dividers[i].word:Hide()
		board.dividers[i].ground:Hide()
		board.dividers[i].line:Hide()
	end
	local more = hidden > 0 and not Tree.expanded[board.t]
	local expander = board.expander
	expander:SetShown(more)
	if more then
		expander:SetOpen(false)
		expander.words:SetText(ns.Plural("tree.expand", hidden):format(hidden))
		expander:ClearAllPoints()
		expander:SetPoint("TOP", board.content, "TOPLEFT", BOARD_W / 2, -(height + 10))
		height = height + 42
	end
	board.content:SetHeight(height + 12)
	board.height = height + 12
	-- the scroll range follows the new height now, not at the client's next layout (Blizzard's ItemTextFrame calls
	-- UpdateScrollChildRect after a child's resize), and the template's own handler gives the bar that range
	-- (SecureScrollTemplates.lua, Gethe `forever` 70374), so a scroll in the same draw (the frontier's) moves the view,
	-- the thumb and the arrows together
	board.scroll:UpdateScrollChildRect()
	ScrollFrame_OnScrollRangeChanged(board.scroll)
end

-- The column names stand over the columns only where every one of the tree's names fits its column on one line, else
-- none and the header row goes, the board starting under the head (the hover and the card's title bar name them).
-- Answers whether they show.
function Tree.RenderHeads(board)
	local fits = true
	for l, h in pairs(board.heads) do
		h.name:SetText(L["ladder." .. l])
		fits = fits and (ns.Readable(h.name:GetStringWidth()) or 0) <= board.pitch - 4
	end
	board.header:SetShown(fits)
	return fits
end

-- The tree's one glow on the step ns.TreeGlow names (the player's step of the climb to train next, or of the ladder
-- the Trial places next), a static halo: its flipbook held still, after one PULSE_S pulse at the tree's first showing
-- this session; none while felt unwell (reason "rest") or where no tree glows.
function Tree.RenderGlow(board, facts, rowsAt)
	local tf = facts.trees[board.t]
	local l = tf.glow
	local src = l and facts.ladders[l]
	local row = src and ns.Ladders[l].rungs[src.rung] and rowsAt[ns.Ladders[l].rungs[src.rung].step]
	local glow = board.glow
	if not (row and row.full) then
		Style.ShowGlow(glow, false)
		board.glowOn = nil
		return
	end
	glow:ClearAllPoints()
	glow:SetPoint("CENTER", board.nodes[l][src.rung])
	local key = l .. ":" .. src.rung
	-- a board shown again whose flipbook its hide stopped (neither held nor playing) plays it again for its hold, so the
	-- texture never rests on the whole sprite sheet
	local flip = glow.pulse
	local fresh = board.glowOn ~= key or not glow:IsShown() or (flip ~= nil and not (flip:IsPlaying() or flip:IsPaused()))
	board.glowOn = key
	if fresh then Style.ShowGlow(glow, true) end
	-- held still on every render, so a board shown again never runs on looping
	local pulse = fresh and not Tree.opened[board.t] and Tree.Moving()
	T.after("glow" .. board.t, pulse and PULSE_S or STILL_S, function() Style.HoldGlow(glow) end)
end

-- Whether the tree moves: "Animate the move figure" off is the game's reduced motion (the game has no system switch),
-- and shows every end at once.
function Tree.Moving()
	return not (ns.profile and ns.profile.figureAnimate == false)
end

-- Whether a ladder's ringed step carries beads (slice 42, docs/ui.md "The beads"): its ring shows (not waiting whole,
-- not back from a break), it is below the ladder's top and its climb has a path; never on another step.
function Tree.Beaded(l, src, entry)
	return not (entry.waits or entry.placing) and src.rung >= 1 and src.rung < ns.HeldTop(l)
		and #ns.Path(l, src.rung) > 0
end

-- The steps of tree `t` whose row holds a ringed step with beads, by step (Tree.Rows' band).
function Tree.BeadSteps(t, facts)
	local steps = {}
	for _, l in ipairs(ns.TreeLadders(t)) do
		local src, entry = facts.ladders[l], facts.entries[l]
		if src and Tree.Beaded(l, src, entry) then steps[ns.Ladders[l].rungs[src.rung].step] = true end
	end
	return steps
end

-- A ladder's beads, built at its first need: one a stage, a disc in its fill (done gold, open white, waiting dim) and,
-- on an open bead, the hole that makes it a ring, each round through CircleMask (the thin row's dot's mask). They carry
-- no words of their own and take no mouse (docs/ui.md, "The beads"): the node's hover names the stage still out
-- (Tree.PathKey) and the card's rows name each one.
local function roundTexture(parent, layer, sub)
	local t = parent:CreateTexture(nil, layer, nil, sub)
	t:SetPoint("CENTER")
	local mask = parent:CreateMaskTexture()
	mask:SetAllPoints(t)
	if Style.SetAtlas(mask, "circle") then t:AddMaskTexture(mask) end
	return t
end

function Tree.Bead(board)
	local b = CreateFrame("Frame", nil, board.over)
	b:SetSize(BEAD, BEAD)
	b.disc = roundTexture(b, "ARTWORK", 1)
	b.hole = roundTexture(b, "ARTWORK", 2)
	b.hole:SetSize(BEAD - 2 * BEAD_STROKE, BEAD - 2 * BEAD_STROKE)
	b.hole:SetColorTexture(unpack(saga.screen))
	b.fade = b.disc:CreateAnimationGroup()
	local a = b.fade:CreateAnimation("Alpha")
	a:SetFromAlpha(0)
	a:SetToAlpha(1)
	a:SetStartDelay(HOLD_S)
	a:SetDuration(FILL_S)
	a:SetSmoothing("OUT")
	return b
end

-- One bead in its stage's state: a stage done (or skipped, which counts as done) a gold disc, one open a white ring of
-- BEAD_STROKE, one waiting for the move before it a small dim dot, so done, open and waiting differ by fill and size,
-- never by colour alone; a stage a set has just finished fades in with the ring's fill (Tree.motion, the return).
function Tree.PaintBead(b, st, fresh)
	local done = st.state == "done" or st.state == "skipped"
	local size = st.state == "waiting" and BEAD_WAIT or BEAD
	b.disc:SetSize(size, size)
	local c = done and { NORMAL_FONT_COLOR:GetRGB() } or st.state == "open" and { HIGHLIGHT_FONT_COLOR:GetRGB() }
		or saga.note
	b.disc:SetColorTexture(c[1], c[2], c[3], 1)
	b.hole:SetShown(st.state == "open")
	b.fade:Stop()
	if fresh and done then b.fade:Play() end
end

-- The beads under ladder `l`'s ringed step (Tree.Beaded), centred on the column's line in its row's band, in the path's
-- order from the left; none elsewhere.
function Tree.RenderBeads(board, l, row, src, entry)
	local list = board.beads[l] or {}
	board.beads[l] = list
	local stages = row and row.full and Tree.Beaded(l, src, entry) and ns.Stages(ns.state, l, src.rung, Tree.facts.f)
		or {}
	local fresh = Tree.motion and Tree.motion.beads and Tree.motion.beads[l] or {}
	local x0 = board.cols[l] - ((#stages - 1) * (BEAD + BEAD_GAP)) / 2
	for i, st in ipairs(stages) do
		local b = list[i] or Tree.Bead(board)
		list[i] = b
		Tree.PaintBead(b, st, fresh[i])
		pin(board, b, x0 + (i - 1) * (BEAD + BEAD_GAP), row.y + ROW_H / 2 + BEAD_ROW / 2)
		b:Show()
	end
	for i = #stages + 1, #list do list[i]:Hide() end
end

function Tree.RenderBoard(board, facts)
	local tf = facts.trees[board.t]
	local rowsAt, dividers, height = Tree.Rows(tf.rows, Tree.expanded[board.t], Tree.BeadSteps(board.t, facts))
	board.rowsAt = rowsAt
	for _, l in ipairs(ns.TreeLadders(board.t)) do Tree.RenderColumn(board, l, rowsAt, facts) end
	Tree.RenderRows(board, rowsAt, dividers, height, tf.hidden)
	Tree.RenderGlow(board, facts, rowsAt)
	local sel = Tree.selected
	Tree.DrawLinks(board, sel and sel[1], sel and sel[2])
end

-- The links of step (l, r): every link that touches it (the steps it opens, the steps it waits for), each an arrow
-- into the step it opens, yellow once what it asks is done (Tree.LinkDone), else locked; a link to a step of another
-- tree ends at the board's edge in a chip, a button that switches the tab and selects that step. The selected step's
-- show, and the step under the pointer previews its own; the arrow heads turn by math.atan2 on the board's own
-- coordinates, never a frame's read centre.
function Tree.DrawLinks(board, l, r)
	if not board then return end
	for _, link in ipairs(board.links) do
		link.line:Hide()
		link.head:Hide()
	end
	for _, chip in ipairs(board.chips) do chip:Hide() end
	if not (l and r and board.rowsAt) then return end
	local n, c = 0, 0
	for _, link in ipairs(Tree.Links()) do
		local touches = (link.from[1] == l and link.from[2] == r) or (link.to[1] == l and link.to[2] == r)
		if touches then
			local other = (link.from[1] == l and link.from[2] == r) and link.to or link.from
			local done = Tree.LinkDone(link)
			if ns.TreeOf(other[1]) == board.t and n < LINKS then
				n = n + 1
				Tree.DrawArrow(board, n, link, done)
			elseif ns.TreeOf(other[1]) ~= board.t and c < CHIPS then
				c = c + 1
				Tree.DrawChip(board, c, l, r, other)
			end
		end
	end
end

-- A link's parts: its line on the scroll child under the steps, its head on the frame over them (a scroll child's own
-- texture draws under its child frames), so no step covers it.
function Tree.LinkParts(board, i)
	local link = board.links[i]
	if link then return link end
	link = { line = board.content:CreateLine(nil, "ARTWORK", nil, 1), head = board.over:CreateTexture(nil, "OVERLAY") }
	link.line:SetThickness(LINE_W)
	link.head:SetSize(HEAD_W, HEAD_H)
	board.links[i] = link
	return link
end

-- How far a step's visible edge stands from its centre along `angle`: a node's dark backing, a square (PlaceTick's
-- maths), or a dot (a thin row's step, a ladder that waits whole).
function Tree.EdgeReach(board, l, r, angle)
	local row = board.rowsAt[ns.Ladders[l].rungs[r].step]
	local entry = Tree.facts and Tree.facts.entries[l]
	if not (row.full and entry and not entry.waits) then return DOT / 2 end
	local half = (ns.Readable(board.nodes[l][r].backing:GetWidth()) or SLOT) / 2
	return half / math.max(math.abs(math.cos(angle)), math.abs(math.sin(angle)))
end

-- A link's line from the centre of the step it comes from to its head, whose tip touches the visible edge of the step
-- it opens, turned toward it (Blizzard_SharedTalentEdgeTemplates.lua's arrow: the head's rotation is the angle from
-- the end to the start less a quarter turn; its atlas points down, so its height runs along the line).
function Tree.DrawArrow(board, i, link, done)
	local parts = Tree.LinkParts(board, i)
	local fx, fy = Tree.StepPoint(board, link.from[1], link.from[2])
	local tx, ty = Tree.StepPoint(board, link.to[1], link.to[2])
	if not (fx and tx) then return end
	-- the board's y runs down: the angle in screen terms, y up
	local angle = math.atan2(ty - fy, fx - tx)
	local reach = Tree.EdgeReach(board, link.to[1], link.to[2], angle) + HEAD_H / 2
	local ex, ey = tx + math.cos(angle) * reach, ty - math.sin(angle) * reach
	if not Style.SetAtlas(parts.line, done and "arrowLine" or "arrowLineLocked") then
		parts.line:SetColorTexture(done and 1 or 0.5, done and 0.82 or 0.5, done and 0 or 0.5, 0.9)
	end
	parts.line:SetStartPoint("TOPLEFT", board.content, fx, -fy)
	parts.line:SetEndPoint("TOPLEFT", board.content, ex, -ey)
	parts.line:Show()
	if Style.SetAtlas(parts.head, done and "arrowHead" or "arrowHeadLocked") then
		parts.head:SetRotation(angle - math.pi / 2)
		pin(board, parts.head, ex, ey)
		parts.head:Show()
	end
end

-- Where a step stands on the board: its column's x and its row's y; nil off this board.
function Tree.StepPoint(board, l, r)
	local x, w = board.cols[l], ns.Ladders[l].rungs[r]
	local row = x and w and board.rowsAt[w.step]
	if not row then return nil end
	return x, row.y
end

-- A cross-tree link's chip: "{move} · step {n} · {tree}" at the board's edge level with the step, the far edge.
function Tree.DrawChip(board, i, l, r, other)
	local chip = board.chips[i]
	local x, y = Tree.StepPoint(board, l, r)
	if not x then return end
	chip.ladder, chip.rung = other[1], other[2]
	Tree.ChipWords(chip, Tree.ChipText(other[1], other[2]), CHIP_MAX)
	chip:ClearAllPoints()
	-- on the far side of the board from its step, clear of the step and its neighbours
	local right = x <= BOARD_W / 2
	chip:SetPoint(right and "RIGHT" or "LEFT", board.content, right and "TOPRIGHT" or "TOPLEFT", right and -EDGE or EDGE,
		-(y - SLOT / 2 - 12 + (i - 1) * 26))
	chip:Show()
end

-- A link chip's words (Kit.LinkChip: its words, 6, its chevron, 8 in at each end), the chip as wide as they need up to
-- `most` units, past which they end in the client's ellipsis.
function Tree.ChipWords(chip, words, most)
	chip.words:SetWidth(0)
	chip.words:SetText(words)
	local w = math.min(most - 38, ns.Readable(chip.words:GetStringWidth()) or most - 38)
	chip.words:SetWidth(w)
	chip:SetSize(w + 38, 22)
end

function Tree.ChipText(l, r)
	local t = ns.TreeOf(l)
	return L["tree.chip"]:format(moveName(Tree.StepMove(l, r)), ns.Step(l, r) or r, L["tree." .. ns.TREES[t].key])
end

-- The header and the foot ---------------------------------------------------------------------------------------------

-- A tree's Next line under its name (docs/ui.md, "The tree's lines"): the glowing climb's next step in its ladder's
-- words, its proof or the set that earns it (a pose family's Perfect hold of its standing pose, as the hover's
-- Tree.NowKey says it); the Trial's first set; or why nothing glows (felt unwell in the greyed Start's words,
-- Tree.BlockLine).
function Tree.NextLine(t, facts)
	local tf = facts.trees[t]
	if tf.reason == "rest" then return (Tree.BlockLine("locked")) end
	if tf.reason == "day" then return L["tree.next.day"]:format(Sheet.ClockTime(tf.at)) end
	if tf.reason then return L["tree.next." .. tf.reason] end
	local l = tf.glow
	local src = facts.ladders[l]
	local standing = src.standing or Tree.StepMove(l, src.rung)
	if tf.kind == "trial" then return L["tree.next.trial"]:format(moveName(standing)) end
	local earn, move, floor = ns.ToEarn(ns.state, l, facts.f)
	if earn == src.rung and floor then return L["tree.next.earn"]:format(moveName(move), goal(move, floor)) end
	if earn == src.rung and POSE[l] then return L["tree.next.earnMark"]:format(moveName(standing)) end
	if l == ns.CARDIO then return Tree.NextCardio(src) end
	return Tree.NextClimb(l, src)
end

-- The cardio ladder's Next line (2026-10-16): a timed rung's full moves (slice 52; ready days until then); a counted
-- one's clean days, and the form its
-- path waits for on its own clean days once the step's are in (Tree.PathNext), or that form and what holds it
-- (Tree.PathHeldWords, 2026-10-19), and once the path is in too the proving window (slice 53, ns.Proving), or after a
-- try that did not pass this session that the climb waits, as the step's card says (Tree.ClimbLine).
function Tree.NextCardio(src)
	local p = src.progress
	if not Tree.Counted(src.rung) then return L["tree.next.full"]:format(p[1], p[2]) end
	local proving, form = ns.Proving(ns.state, Tree.facts.f, Tree.facts.now)
	if proving == "waits" then return L["ladders.step.climbWaits"] end
	if proving then return L["tree.next.proving"]:format(moveName(form)) end
	if p[1] >= p[2] and p[3] == 1 then
		local st = Tree.PathNext(ns.CARDIO, src.rung)
		local held = st and Tree.PathHeldWords(st)
		if held then return L["tree.next.pathHeld"]:format(moveName(st.move), held) end
		if st then return ns.Plural("tree.next.cleanForm", st.days):format(moveName(st.move), st.have, st.days) end
	end
	return L["tree.next.clean"]:format(p[1], p[2])
end

-- A strength, hold or pose ladder's Next line: the path's next stage alone once the days are in (Tree.PathNext,
-- 2026-10-16), in the ladder's measure (a pose's days close enough to its goal pose, or Perfect where only Perfect
-- counts, ns.MarkNeeds; a move's top on its days), or its move and what holds it (Tree.PathHeldWords, 2026-10-19);
-- else the move at its top on its days, any goal pose of the step close enough (or Perfect) on its days (slice 47,
-- 2026-10-19; the standing pose until then).
function Tree.NextClimb(l, src)
	local p = src.progress
	local id = src.standing
	local perfect = POSE[l] and ns.MarkNeeds(l, src.rung) == ns.Enum.mark.perfect
	if p[1] >= p[2] and p[3] == 1 then
		local st, top = Tree.PathNext(l, src.rung)
		local held = st and Tree.PathHeldWords(st)
		if held then return L["tree.next.pathHeld"]:format(moveName(st.move), held) end
		if st and POSE[l] then
			return L[perfect and "tree.next.proofPerfect" or "tree.next.proofPose"]:format(moveName(st.move), days(st.days))
		end
		if st and top then return L["tree.next.proof"]:format(moveName(st.move), Sheet.ClimbTop(st.move, top, true)) end
	end
	if POSE[l] then return L[perfect and "tree.next.perfect" or "tree.next.pose"]:format(L["ladder." .. l], days(p[2])) end
	local hi
	if id then hi = select(2, Tree.Window(l, src.rung, id)) end
	if not hi then return L["tree.next.none"] end
	return L["tree.next.climb"]:format(moveName(id), Sheet.ClimbTop(id, hi, true), days(p[2]))
end

-- The foot: the count per colour across the ladders that can hold it (every tree's, as the desktop app's Book counts
-- them), a quality badge each in the colour's own words ("Rare 1/6", Kit.Badge: never the colour alone; its tooltip
-- says what the fraction counts), left to right up to the first colour no ladder holds yet, the next to collect; where
-- they outrun the page's width the lowest go first (every ladder holds them). None while a strength ladder waits for
-- its first max set (the Next line and that step's card say how the start works), the board then taking the room.
-- Answers whether it shows.
function Tree.FootBadges(facts)
	local foot = Tree.page.foot
	local placed = true
	for _, src in pairs(facts.ladders) do
		if not src.placed then placed = false end
	end
	foot:SetShown(placed)
	if not placed then return false end
	local shown = {}
	for q = ns.Enum.quality.starter, ns.Enum.quality.artifact do
		local have, can = 0, 0
		for _, l in ipairs(ns.LADDER_ORDER) do
			if ns.LadderTop(l) >= q then
				can = can + 1
				if ns.LadderColour(ns.state, l) >= q then have = have + 1 end
			end
		end
		if can > 0 then
			local badge = foot.badges[#shown + 1]
			if not badge then
				badge = Kit.Badge(foot, q)
				-- what its fraction counts, in its colour's words
				Kit.Hover(badge, Kit.Tip(function() return Kit.QualityWords(badge.q) end, function()
					return ns.Plural("tree.foot.tip", badge.have):format(badge.have, badge.can, Kit.QualityWords(badge.q))
				end, { color = function() return Sheet.QualityColor(badge.q) end }), "ANCHOR_TOP")
				foot.badges[#shown + 1] = badge
			end
			badge.q, badge.have, badge.can = q, have, can
			badge:Set(q, L["ladders.collection.item"]:format(Kit.QualityWords(q), have, can))
			shown[#shown + 1] = badge
			if have == 0 then break end
		end
	end
	for i = #shown + 1, #foot.badges do foot.badges[i]:Hide() end
	local width = -6
	for _, badge in ipairs(shown) do width = width + badge:GetWidth() + 6 end
	local first = 1
	while width > foot:GetWidth() and first < #shown do
		shown[first]:Hide()
		width = width - shown[first]:GetWidth() - 6
		first = first + 1
	end
	local x = 0
	for i = first, #shown do
		shown[i]:ClearAllPoints()
		shown[i]:SetPoint("TOPLEFT", x, 0)
		shown[i]:Show()
		x = x + shown[i]:GetWidth() + 6
	end
	return true
end

-- The page --------------------------------------------------------------------------------------------------------

-- Built once with the sheet (Sheet.Build): the head in one row (the kit's page head: the tree's name, its Next line
-- right of it in the intro's soft grey, which holds the search's count while a search is on, and the search box at the
-- right, the rule under them), the foot's badges, and the three tabs under the sheet (TabSystemTemplate, as
-- PlayerSpellsFrame anchors its own: TOPLEFT to the frame's BOTTOMLEFT, 22, 2; Gethe `forever` 70374), shown with
-- this page alone. The template's own sound would skip the Sounds switch, so it has none and the selection plays
-- Style's "tab" (TabSystemTemplates.lua: PlayTabSelectSound plays tabSelectSound when set). The boards come with their
-- tree's first showing.
function Tree.Build(page)
	Tree.page = page
	page.search = Kit.Search(page, 160, L["tree.search"], Tree.Search)
	Kit.Head(page, L["tree." .. ns.TREES[Tree.tree].key], nil, "page", page.search)
	page.intro:SetTextColor(unpack(saga.soft)) -- cut where it outruns its room, its words whole in a tooltip (Kit.Head)
	page.foot = CreateFrame("Frame", nil, page)
	page.foot:SetPoint("TOPLEFT", Kit.TIER.page.IN, -FOOT_Y)
	page.foot:SetSize(Kit.TIER.page.TEXT, 18)
	page.foot.badges = {}
	local tabs = CreateFrame("Frame", nil, page, "TabSystemTemplate")
	tabs:SetPoint("TOPLEFT", page:GetParent(), "BOTTOMLEFT", 22, 2)
	tabs.tabSelectSound = nil
	tabs:SetTabSelectedCallback(function(t, user)
		Tree.Select(t, user)
		return false
	end)
	for _, tree in ipairs(ns.TREES) do tabs:AddTab(L["tree." .. tree.key]) end
	Tree.tabs = tabs
	page:SetScript("OnHide", function()
		Tree.HideCard(true)
		Tree.selected = nil -- the card's selection goes with it (Escape closes the two together)
		T.cancel("due")
		Tree.Land()
		GameTooltip:Hide()
	end)
end

-- Draws the page while it shows (the sheet's Refresh, on its own messages): the shown tree's board (built on its
-- first showing), the head's row, the column names and the board's place under them, the foot and the card where it
-- pins.
function Tree.Render()
	local page = Tree.page
	if not (page and page:IsVisible() and ns.state) then return end
	local facts = Tree.Facts()
	Tree.facts = facts
	local t = Tree.tree
	for i, b in pairs(Tree.boards) do
		b.scroll:SetShown(i == t)
		b.header:SetShown(i == t)
	end
	local board = Tree.boards[t] or Tree.BuildBoard(t)
	board.scroll:Show()
	Tree.tabs:SetTabVisuallySelected(t)
	page.title:SetText(L["tree." .. ns.TREES[t].key])
	local found = Tree.found
	page.intro:SetText(found and ns.Plural("tree.search.count", found):format(found) or Tree.NextLine(t, facts))
	-- the board under the names (or the head where they do not show), down to the foot (or the page's end without it)
	board.top = Tree.RenderHeads(board) and NAMES_TOP or BARE_TOP
	local bottom = Tree.FootBadges(facts) and FOOT_Y - FOOT_GAP or PAGE_H - 4
	board.scroll:ClearAllPoints()
	board.scroll:SetPoint("TOPLEFT", BOARD_X, -board.top)
	board.scroll:SetHeight(bottom - board.top)
	Tree.RenderBoard(board, facts)
	if not Tree.opened[t] then
		Tree.opened[t] = true
		Tree.ScrollTo(board, Tree.Frontier(t, facts), 1 / 3)
	end
	-- the card's draw arms the page's next moment with its own Start's (Tree.WatchTimes)
	if Tree.card and Tree.card:IsShown() then Tree.RenderCard() else Tree.WatchTimes() end
end

-- The step the board scrolls to at the tree's first opening: the glowing step, else the highest of the player's steps
-- in the tree (the furthest climbed).
function Tree.Frontier(t, facts)
	local l = facts.trees[t].glow
	if l then return { l, facts.ladders[l].rung } end
	local best, top
	for _, x in ipairs(ns.TreeLadders(t)) do
		local src, entry = facts.ladders[x], facts.entries[x]
		local w = src and not entry.waits and ns.Ladders[x].rungs[src.rung]
		if w and (not top or w.step > top) then best, top = { x, src.rung }, w.step end
	end
	return best
end

-- The board scrolled so step `step` ({ ladder, rung }) stands `share` of the way down the view (a third for the
-- frontier; 0.4 for a step brought into view), within the board's own height: fixed math on the rows. It scrolls
-- through the bar (its OnValueChanged scrolls the view, UIPanelScrollBar_OnValueChanged), so the thumb stands where
-- the view does and the wheel goes on from there.
function Tree.ScrollTo(board, step, share)
	if not step then return end
	local _, y = Tree.StepPoint(board, step[1], step[2])
	if not y then return end
	local view = board.scroll:GetHeight()
	local most = math.max(0, (board.height or 0) - view)
	board.scroll.ScrollBar:SetValue(math.max(0, math.min(most, y - view * share)))
end

-- Another tree (a tab's click, `user`; a search, a chip, a card's button to another ladder's step, the return): its
-- board shows, the card goes (it pins beside its own tree's step), and the tab plays its sound on a click alone. A tab
-- click is a click: the return's motion lands first (Tree.Land), so a fade the board's hiding cuts off never leaves
-- the glow unseen.
function Tree.Select(t, user)
	Tree.Land()
	if user then Style.Play("tab") end
	if t ~= Tree.tree then
		Tree.HideCard(true)
		Tree.selected = nil
		Tree.tree = t
	end
	if Tree.tabs then Tree.tabs:SetTabVisuallySelected(t) end
	Tree.Render()
end

-- A step selected (a card's button, a chip, the return, a search's first match): its tree shows, its links draw and
-- it scrolls into view. Switching the tree lands the return's motion first, as Tree.Select does, and on a click
-- (`user`: a chip, a card's row) plays the tab's sound, as a tab's click does (never for the return or a search).
function Tree.SelectStep(l, r, user)
	local t = ns.TreeOf(l)
	Tree.selected = { l, r }
	if t ~= Tree.tree then
		Tree.Land()
		if user then Style.Play("tab") end
		Tree.HideCard(true)
		Tree.tree = t
		Tree.selected = { l, r }
	end
	Tree.Render()
	local board = Tree.boards[t]
	if board then Tree.ScrollTo(board, { l, r }, 0.4) end
end

-- A step of another ladder shown with its card at a click (a chip, a card's row to the step it opens or waits for, a
-- tile's step): a card that showed already moves to it with no open sound of its own, across trees too (the tab's
-- sound says the switch, unless the click played the kit's own cue: `sounded`, a link chip's).
function Tree.ShowStep(l, r, sounded)
	local was = Tree.card ~= nil and Tree.card:IsShown()
	Tree.SelectStep(l, r, not sounded)
	Tree.ShowCard(l, r, was)
end

-- The search ----------------------------------------------------------------------------------------------------------

-- A search over the moves' names (Forever's own talent window has one): a step that holds a matching move is lit,
-- `exact` where the step's own move matches, else a match among its other moves; the rest dim; the tab switches to the
-- tree of the first match only when the tree shown has none, so typing never jumps away from a match in view, and the
-- count stands in the Next line's place while the search is on (Tree.found).
function Tree.Search(query)
	query = strtrim(query or ""):lower()
	if query == "" then
		Tree.matches, Tree.found = nil, nil
		return Tree.Render()
	end
	local matches, n, first = {}, 0, nil
	for t in ipairs(ns.TREES) do
		for _, l in ipairs(ns.TreeLadders(t)) do
			for r in ipairs(ns.Ladders[l].rungs) do
				local kind = Tree.MatchKind(l, r, query)
				if kind then
					matches[l] = matches[l] or {}
					matches[l][r] = kind
					n = n + 1
					first = first or { l, r }
				end
			end
		end
	end
	Tree.matches, Tree.found = matches, n
	if first and not Tree.MatchesIn(Tree.tree) then return Tree.SelectStep(first[1], first[2]) end
	Tree.Render()
end

-- Whether step (l, r) holds a move named by `query`: "exact" its own move, "match" another of its moves.
function Tree.MatchKind(l, r, query)
	local own = Tree.StepMove(l, r)
	if own and moveName(own):lower():find(query, 1, true) then return "exact" end
	for _, id in ipairs(Tree.StepMoves(l, r)) do
		if moveName(id):lower():find(query, 1, true) then return "match" end
	end
	return nil
end

function Tree.MatchesIn(t)
	for _, l in ipairs(ns.TreeLadders(t)) do
		if Tree.matches[l] then return true end
	end
	return false
end

-- The step's card -------------------------------------------------------------------------------------------------

-- QuestPumpStepCard (docs/ui.md, "The step's card"): the kit's card window (Kit.Window "card": the Options window's
-- flat panel with its title bar and close, 380 wide), at DIALOG strata (an item link's TOOLTIP strata would draw over
-- the pump's FULLSCREEN_DIALOG dim), movable and clamped, in UISpecialFrames (Escape closes it with the sheet, and
-- ConsolePort's cursor finds it there), beside the sheet level with its step. Its title bar names the step (its
-- ladder, step and where the player stands); its head is the step's own figure (a step ahead's too) in a screen with
-- the move's name in its quality's colour, its quality badge, the step's measure and on a step the player has its
-- mastered line beside it; its face, under a hairline, the step's moves in their three groups (slice 42, docs/ui.md
-- "The steps' order": Your main move with the climb and the steps of other ladders and the stretch checks it waits
-- for, Then, for the next step with the path's stages, the hint row, For variety); then, under a hairline, on a step
-- above the player's what reaching it takes ("To get here", Tree.CardRoad) heading the one Start (Kit.Primary, its
-- motion scripts on while greyed) with at most two lines under it, or a refused Start's line with its links; the
-- quiet part goes under More on this step (Kit.More), closed each time the card shows a new step. With More closed
-- the card stays at its cap (Kit.TIER.card's 400): past it the face folds (Tree.RenderCard); with More open it grows
-- to the screen less its room, More's content scrolling past.
local CARD = Kit.TIER.card
-- the figure's screen at (IN, FIG_TOP), the head's column right of it from NAME_Y, the head's least bottom
local FIG_TOP, NAME_Y = 36, 38
local HEAD_X = CARD.IN + FIG + 12
local HEAD_COL = CARD.W - HEAD_X - CARD.IN
local SKIPPED_A = 0.6 -- a skipped stage's row, dimmed

-- A zone of the card: a frame its parts are laid in from its own top, `w` wide, with its pools (lines, section
-- labels, link chips, tiles, grey buttons, and framed lists by kind). Tree.CardZone picks the one the writers write
-- into (card.at).
local function newZone(parent, w)
	local z = CreateFrame("Frame", nil, parent)
	z:SetSize(w, 1)
	z.w, z.y = w, 0
	z.texts, z.sections, z.links, z.tiles, z.buttons, z.lists = {}, {}, {}, {}, {}, {}
	z.nt, z.ns, z.nl, z.ni, z.nb = 0, 0, 0, 0, 0
	return z
end

function Tree.BuildCard()
	local card = Kit.Window("QuestPumpStepCard", "card", "", { movable = true, strata = "DIALOG", escape = "special",
		onClose = function() Tree.HideCard() end })
	-- where the player put it, it stays this session, held by its top left (the client anchors a moved frame by its
	-- nearest point, and RenderCard's height change would then grow a card left low upward), and out of the client's
	-- layout cache (SetUserPlaced): the place is the session's alone; read before ClearAllPoints makes it unreadable
	card:SetScript("OnDragStop", function(self)
		self:StopMovingOrSizing()
		local left, top = ns.Readable(self:GetLeft()), ns.Readable(self:GetTop())
		if left and top then
			self:ClearAllPoints()
			self:SetPoint("TOPLEFT", UIParent, "BOTTOMLEFT", left, top)
		end
		self:SetUserPlaced(false)
		Tree.shown.moved = true
	end)
	card:SetScript("OnHide", function()
		GameTooltip:Hide()
		Tree.shown.choice = nil -- its Start's moments no longer time a draw (Tree.WatchTimes)
		if Tree.quiet then return end
		Style.Play("close")
		Tree.Deselect()
	end)
	local body = card.body
	card.screen = Kit.Screen(body, FIG, FIG)
	card.screen:SetPoint("TOPLEFT", CARD.IN, -FIG_TOP)
	card.figure = ns.Figure.Create(card.screen, FIG)
	card.figure.frame:SetPoint("CENTER")
	card.figure.caption:Hide()
	card.name = Kit.Text(body, "large", "LEFT", nil, HEAD_COL)
	card.name:SetPoint("TOPLEFT", HEAD_X, -NAME_Y)
	card.badge = Kit.Badge(body, ns.Enum.quality.starter)
	card.badge:SetPoint("TOPLEFT", card.name, "BOTTOMLEFT", 0, -6)
	card.zones = { head = newZone(body, HEAD_COL), face = newZone(body, CARD.TEXT) }
	-- the measure's tooltip (the record's span), where the head names one (card.measureTip)
	local head = card.zones.head
	head:EnableMouse(true)
	head:SetScript("OnEnter", function(self)
		if not card.measureTip then return end
		GameTooltip:SetOwner(self, "ANCHOR_RIGHT")
		card.measureTip(GameTooltip)
		GameTooltip:Show()
	end)
	head:SetScript("OnLeave", function() GameTooltip:Hide() end)
	-- a drag that starts on it moves the card, as anywhere else on it
	head:RegisterForDrag("LeftButton")
	head:SetScript("OnDragStart", function() card:GetScript("OnDragStart")(card) end)
	head:SetScript("OnDragStop", function() card:GetScript("OnDragStop")(card) end)
	card.hair = Kit.Rule(body, saga.rule)
	card.hair:SetWidth(CARD.TEXT)
	local face = card.zones.face
	card.faceHair = Kit.Rule(face, saga.rule)
	card.faceHair:SetWidth(CARD.TEXT)
	-- For variety folded past the cap (card.fold 1): its label stays as a More row that opens its tiles in place
	card.same = Kit.More(face, CARD.TEXT, "", function() end)
	card.same.onToggle = function() Tree.RenderCard() end
	card.start = Kit.Primary(face, "", CARD.PRIMARY, function() Tree.Start() end)
	card.start:SetHeight(CARD.BUTTON)
	-- its tooltip: the whole label (its goal, where the button shows less: Tree.CardStart), why it is greyed (red, but
	-- felt unwell's rest line, a fact in the notes' grey: the care tone, never red) and the lines the face leaves to it
	-- (Tree.StartFace)
	card.start:SetScript("OnEnter", function(self)
		local shown = Tree.shown
		local why = not self:IsEnabled() and shown.why or nil
		local rest = why ~= nil and shown.block == "locked"
		local facts = shown.lines or {}
		if rest then facts = { why, unpack(facts) } end
		if not why and #facts == 0 and not shown.whole then return end
		GameTooltip:SetOwner(self, "ANCHOR_RIGHT")
		Kit.Tip(shown.whole or self:GetText(), nil, { facts = facts, reason = not rest and why or nil })(GameTooltip)
		GameTooltip:Show()
	end)
	card.start:SetScript("OnLeave", function() GameTooltip:Hide() end)
	-- the Long Haul beside the player's own cardio step's Start (slice 55: Tree.CardHaul), greyed with its reason (red
	-- in its tooltip) where it cannot run
	card.haul = Kit.Secondary(face, "", CARD.BACK, function() Tree.StartHaul() end, CARD.BUTTON)
	card.haul:SetScript("OnEnter", function(self)
		local why = Tree.shown.haulWhy
		if not why then return end
		GameTooltip:SetOwner(self, "ANCHOR_RIGHT")
		Kit.Tip(self:GetText(), nil, { reason = why })(GameTooltip)
		GameTooltip:Show()
	end)
	card.haul:SetScript("OnLeave", function() GameTooltip:Hide() end)
	card.mine = Tree.BuildTile(face)
	card.more = Kit.More(body, CARD.TEXT, L["tree.card.more"], function() end)
	card.more.onToggle = function() Tree.RenderCard() end
	card.moreScroll = Kit.Scroll(card.more.content, CARD.TEXT, 1)
	card.moreScroll:SetPoint("TOPLEFT")
	card.zones.more = newZone(card.moreScroll.content, CARD.TEXT)
	card.zones.more:SetPoint("TOPLEFT")
	Tree.card = card
	return card
end

-- A move's tile: the sheet's slot, smaller; its hover the move's tooltip, its click the move shown in the card, its
-- right-click the flyout's menu (favourite, leave out, correct a count, clear its record); the objective tracker's
-- check at its bottom-right corner once the move reached its top (slice 42: topped, toward Mastered).
function Tree.BuildTile(parent)
	local b = Sheet.SlotButton(parent)
	b:SetScale(TILE / SLOT)
	b:RegisterForClicks("LeftButtonUp", "RightButtonUp")
	b:SetScript("OnEnter", function(self)
		if not self.move then return end
		GameTooltip:SetOwner(self, "ANCHOR_RIGHT")
		Sheet.MoveTooltip(GameTooltip, self.move, nil, { click = "card", menu = true })
	end)
	b:SetScript("OnLeave", function() GameTooltip:Hide() end)
	b:SetScript("OnClick", function(self, mouse)
		if not self.move then return end
		if mouse == "RightButton" then return Sheet.FlyoutMenu(self) end
		if self.step then return Tree.ShowStep(self.step[1], self.step[2]) end
		Tree.ShowMove(self.move)
	end)
	b.topped = b:CreateTexture(nil, "OVERLAY", nil, 7)
	b.topped:SetSize(SEAL + 4, SEAL + 4)
	b.topped:SetPoint("BOTTOMRIGHT", 3, -3)
	if not Style.SetAtlas(b.topped, "done") then Style.SetMark(b.topped, true) end
	b.topped:Hide()
	return b
end

-- `ahead`: the move's figure greyed, a move above the player's reach, as a step ahead's node draws; `topped`: the
-- tracker's check at its corner (a tile of Tree.BuildTile's; the step-up reveal's slots wear none, Panel.lua).
local function setTile(b, id, q, ahead, topped)
	local file, l, r, t, bb = Style.MoveIcon(id)
	Sheet.SetIcon(b, file, l, r, t, bb, ns.Library[id].mp[1])
	SetItemButtonQuality(b, q)
	SetItemButtonDesaturated(b, ahead == true)
	if b.topped then b.topped:SetShown(topped == true) end
	b.move = id
	b:Show()
end
Tree.SetTile = setTile

-- A step's other move as its tile shows it: in its quality only where the player has it, so the card never shows a
-- move in a colour the player has not got (the Book's grid's rule, slice 39): above the player's reach (ns.AboveReach)
-- greyed with none, and none on a move never trained (no set to its familiarity, no record).
function Tree.TileQuality(id)
	if ns.AboveReach(ns.state, id) then return nil, true end
	local trained = ns.Familiarity(ns.state, id) >= 1 or (ns.Record(ns.state, id) or 0) > 0
	return trained and ns.MoveQuality(id) or nil, false
end

-- The writers: each part under the one before in the zone being written (card.at), from the zone's top; the first
-- part of a zone takes no gap above it.
function Tree.CardZone(card, key)
	card.at = card.zones[key]
end

local function gap(z, units)
	return z.y == 0 and 0 or (units or 4)
end

function Tree.CardBegin(card)
	local shown = Tree.shown
	shown.opensSaid, shown.roadSaid, shown.cardChoice, shown.startCheck, card.measureTip = nil, nil, nil, nil, nil
	for _, z in pairs(card.zones) do
		z.y, z.nt, z.ns, z.nl, z.ni, z.nb = 0, 0, 0, 0, 0, 0
		for _, pool in ipairs({ z.texts, z.sections, z.links, z.tiles, z.buttons }) do
			for _, part in ipairs(pool) do part:Hide() end
		end
		for _, pool in pairs(z.lists) do
			pool.n = 0
			for _, list in ipairs(pool) do list:Hide() end
		end
	end
	card.start:Hide()
	card.haul:Hide()
	shown.haul, shown.haulWhy = nil, nil
	card.mine:Hide()
	card.same:Hide()
	card.faceHair:Hide()
	card.figure.frame:Hide()
	Tree.CardZone(card, "face")
end

-- A line under the last, in `font` (bodyWhite without) and `color` (white without; a saga colour or Blizzard's), set
-- every time: a pooled line never keeps an earlier line's red, nor a value row's right edge (Tree.CardPair).
function Tree.CardLine(card, str, font, units, color)
	if not str or str == "" then return nil end
	local z = card.at
	z.nt = z.nt + 1
	local fs = z.texts[z.nt]
	if not fs then
		fs = Kit.Text(z, "bodyWhite", "LEFT", nil, z.w)
		z.texts[z.nt] = fs
	end
	fs:SetJustifyH("LEFT")
	Style.SetFont(fs, font or "bodyWhite")
	fs:SetText(str)
	local c = color or HIGHLIGHT_FONT_COLOR
	if c.GetRGB then fs:SetTextColor(c:GetRGB()) else fs:SetTextColor(c[1], c[2], c[3]) end
	local y = z.y + gap(z, units)
	fs:ClearAllPoints()
	fs:SetPoint("TOPLEFT", 0, -y)
	fs:Show()
	z.y = y + (ns.Readable(fs:GetStringHeight()) or 12)
	return fs
end

-- A value row under the last (slice 52: a timed cardio step's Pace and Full moves): its name at the left in the notes'
-- grey and its value in white at the zone's right edge, on one line.
function Tree.CardPair(card, name, words, units)
	local left = Tree.CardLine(card, name, "bodyWhite", units, saga.note)
	if not left then return nil end
	local z = card.at
	z.nt = z.nt + 1
	local fs = z.texts[z.nt]
	if not fs then
		fs = Kit.Text(z, "bodyWhite", "LEFT", nil, z.w)
		z.texts[z.nt] = fs
	end
	Style.SetFont(fs, "bodyWhite")
	fs:SetJustifyH("RIGHT")
	fs:SetText(words)
	fs:SetTextColor(HIGHLIGHT_FONT_COLOR:GetRGB())
	fs:ClearAllPoints()
	fs:SetPoint("TOPLEFT", left, "TOPLEFT")
	fs:Show()
	return left
end

-- A timed cardio step's Pace (slice 52, ns.SteadyPace): the seconds its own forms keep their pace now, of the full
-- move's.
function Tree.CardPace(card, rung, units)
	local now, full = ns.SteadyPace(ns.state, rung, ns.now())
	return Tree.CardPair(card, L["tree.card.pace"], L["tree.card.paceValue"]:format(now, full), units)
end

-- A section's label (the kit's caps and its rule) and the room under it to its content.
function Tree.CardSection(card, words, units)
	local z = card.at
	z.ns = z.ns + 1
	local sec = z.sections[z.ns]
	if not sec then
		sec = Kit.Section(z, words)
		z.sections[z.ns] = sec
	end
	Kit.SetCaps(sec.words, words)
	local y = z.y + gap(z, units or 14)
	sec:ClearAllPoints()
	sec:SetPoint("TOPLEFT", 0, -y)
	sec:Show()
	z.y = y + 12 + 12
end

-- A quiet link of the card (the kit's link chip, never red): a step this one opens, a refused Start's link, Back to the
-- step; `act` after the chip's own cue.
function Tree.CardLink(card, words, act, units)
	local z = card.at
	z.nl = z.nl + 1
	local chip = z.links[z.nl]
	if not chip then
		chip = Kit.LinkChip(z, "", function(self) self.act() end)
		chip.words:SetWordWrap(false)
		z.links[z.nl] = chip
	end
	chip.act = act
	Tree.ChipWords(chip, words, z.w)
	local y = z.y + gap(z, units or 6)
	chip:ClearAllPoints()
	chip:SetPoint("TOPLEFT", 0, -y)
	chip:Show()
	z.y = y + 22
end

-- A grey kit button at the zone's right under the last part (a check row's "Do the check"), greyed with `why` (its
-- reason, red in its tooltip) where it cannot run; `act` after the button's own cue.
function Tree.CardButton(card, words, act, why, units)
	local z = card.at
	z.nb = z.nb + 1
	local b = z.buttons[z.nb]
	if not b then
		b = Kit.Secondary(z, "", CARD.BACK, function(self) self.act() end, CARD.BUTTON)
		b:SetScript("OnEnter", function(self)
			if not self.why then return end
			GameTooltip:SetOwner(self, "ANCHOR_RIGHT")
			Kit.Tip(self:GetText(), nil, { reason = self.why })(GameTooltip)
			GameTooltip:Show()
		end)
		b:SetScript("OnLeave", function() GameTooltip:Hide() end)
		z.buttons[z.nb] = b
	end
	b.act, b.why = act, why
	b:SetText(words)
	b:SetWidth(math.max(CARD.BACK, (ns.Readable(b:GetTextWidth()) or 80) + 40))
	b:SetEnabled(why == nil)
	local y = z.y + gap(z, units or 6)
	b:ClearAllPoints()
	b:SetPoint("TOPRIGHT", z, "TOPRIGHT", 0, -y)
	b:Show()
	z.y = y + CARD.BUTTON
end

-- A framed list of kit rows of one kind (Kit.List: "stage", "need", "check", "hint"), the next of its kind's pool in
-- the zone being written, its rows taken in order (Tree.ListRow) and laid under the last part (Tree.CardLayList).
function Tree.CardList(card, kind)
	local z = card.at
	local pool = z.lists[kind] or { n = 0 }
	z.lists[kind] = pool
	pool.n = pool.n + 1
	local list = pool[pool.n]
	if not list then
		list = Kit.List(z, z.w, CARD.rowName)
		pool[pool.n] = list
	end
	list.used = 0
	return list
end

-- The list's next row, made by `build(list)` at its first need.
function Tree.ListRow(list, build)
	list.used = list.used + 1
	local row = list.rows[list.used] or build(list)
	row:SetAlpha(1)
	row:Show()
	return row
end

function Tree.CardLayList(card, list, units)
	for i = list.used + 1, #list.rows do list.rows[i]:Hide() end
	if list.used == 0 then return list:Hide() end
	local z = card.at
	local y = z.y + gap(z, units or 8)
	list:ClearAllPoints()
	list:SetPoint("TOPLEFT", z, "TOPLEFT", 0, -y)
	list:Show()
	z.y = y + Kit.LayList(list)
end

-- A hairline across the face under what is written so far (none on an empty face), before the Start.
function Tree.CardHair(card)
	local z = card.at
	if z.y == 0 then return end
	card.faceHair:ClearAllPoints()
	card.faceHair:SetPoint("TOPLEFT", 0, -(z.y + 12))
	card.faceHair:Show()
	z.y = z.y + 12 + 1
end

-- Tiles of moves (`ids`) in rows across the zone, each in its quality where the player has it (Tree.TileQuality) with
-- its topped mark (ns.TopOnce).
function Tree.CardTiles(card, ids, units)
	if #ids == 0 then return end
	local z = card.at
	local per = math.floor(z.w / (TILE + 6))
	local y = z.y + gap(z, units or 6)
	local scale = TILE / SLOT
	for i, id in ipairs(ids) do
		z.ni = z.ni + 1
		local b = z.tiles[z.ni]
		if not b then
			b = Tree.BuildTile(z)
			z.tiles[z.ni] = b
		end
		b:ClearAllPoints()
		b:SetPoint("TOPLEFT", z, "TOPLEFT", (3 + ((i - 1) % per) * (TILE + 6)) / scale,
			-(y + 3 + math.floor((i - 1) / per) * (TILE + 6)) / scale)
		b.step = false
		local q, ahead = Tree.TileQuality(id)
		setTile(b, id, q, ahead, ns.TopOnce(ns.state, id))
	end
	z.y = y + math.ceil(#ids / per) * (TILE + 6)
end

-- A refused Start's line and its links (Your setup at its page, the Body page, the move's Include in pumps).
function Tree.CardRefusal(card, why, subject, l)
	local line, link, page = Tree.Refusal(why, subject, l)
	Tree.CardLine(card, link and L["tree.link.line"]:format(line) or line, "bodyWhite", 10)
	if link == "setup" then
		Tree.CardLink(card, L["tree.link.setup"], function() Setup.Open(page) end)
	elseif link == "body" then
		Tree.CardLink(card, L["tree.link.body"], function() ns.Settings.Open("body") end)
	elseif link == "include" then
		Tree.CardLink(card, L["tree.link.include"], function() ns.Settings.SetExcluded(page, false) end)
	end
	if why == "setup" and subject and subject.off then
		Tree.CardLine(card, L["tree.link.line"]:format(L["tree.offToo"]:format(moveName(subject.move))), "bodyWhite", 8)
		Tree.CardLink(card, L["tree.link.include"], function() ns.Settings.SetExcluded(subject.move, false) end)
	end
end

-- The Start label and its lines -------------------------------------------------------------------------------------

-- The Start's label (docs/ui.md, "The Start label"): its verb by the kind (`as`, `back`), the move and the start's own
-- goal (ns.StartFor's `goal`, as the pump will ask it): "Start Push-up · 28+", "Climb with Close push-up · 12+", "Find
-- my step" (a placement's, the move it runs beside it: 2026-10-29), "Start Push-up · Max set" (a counting move's
-- with no goal yet), "Start Jog" (a timed move or a pose shows no number), "Start Jumping jacks · all out" (an all-out
-- window, a counted set's too since slice 42, as its own or a held step's start; under the climb, back and earn verbs
-- its plain label, as the desktop app's and the phones'), "Earn Archer pull-up · 4+", and a stretch check's "Do the
-- check", no move and no number (slice 42: its tile names the pose). `short`: the verb and the move alone, for a
-- Start whose room the whole label outruns (Tree.CardStart).
function Tree.StartLabel(s, short)
	if s.as == "check" then return L["tree.start.check"] end
	if s.as == "place" then return L["tree.start.place"] end
	local m = ns.Library[s.move]
	local verb = s.back and "back" or (s.as == "path" or s.as == "climb") and "climb" or s.as == "earn" and "earn"
		or "own"
	local label = L["tree.start." .. verb]:format(moveName(s.move))
	if short then return label end
	if s.goal then return L["tree.start.goal"]:format(label, goal(s.move, s.goal)) end
	if m.dose == "rep" or m.dose == "hold" then return L["tree.start.max"]:format(label) end
	if s.allout and verb == "own" then return L["tree.start.allout"]:format(label) end
	return label
end

-- The lines beside the Start (docs/ui.md, "The Start label" and "The card's other lines"), in their order: what earns
-- the step, why a set cannot earn it yet, what a stretch check opens, what the Start waits on where it runs another
-- step (`via`), then when the day counts again and when a set is fresh; and apart, the tired line. The Start's own
-- label says the rest (a climb back, the move another ladder's Start trains: no line beside it, David, 2026-10-22). An
-- earn line that names its time (a pose held today, a set not fresh) is the step's one time: no day line and no fresh
-- line repeat it (the phones' waitsToEarn, every surface since 2026-10-15). Answers the lines and the tired line (nil
-- where it does not apply).
function Tree.StartLines(s, l, r)
	local facts = Tree.facts
	local src = l and facts.ladders[l]
	local earnStep = src and r == src.rung and ns.ToEarn(ns.state, l, facts.f) == r
	local out = {}
	local fresh, timed = s.freshFrom, false
	if s.as == "earn" then
		out[#out + 1] = POSE[l] and L["tree.card.earnPose"] or L["tree.card.earn"]:format(value(s.move, s.earn))
		fresh = nil
	elseif earnStep and POSE[l] and s.dayIn then
		out[#out + 1] = L["tree.card.earnLaterPose"]:format(Sheet.ClockTime(s.dayIn))
		timed = true
	elseif earnStep and s.earn and s.freshFrom then
		out[#out + 1] = L["tree.card.earnLater"]:format(Sheet.ClockTime(s.freshFrom), value(s.move, s.earn))
		timed = true
	end
	-- a stretch check's: what it opens, where no check row on the card says it already (Tree.shown.startCheck)
	if s.as == "check" and not Tree.shown.startCheck then
		local c = Tree.StartedCheck(s)
		if c then out[#out + 1] = Tree.CheckLine(c) end
	end
	out[#out + 1] = Tree.ViaLine(s, l, r, src)
	Tree.DayLines(out, s, fresh, timed)
	return out, s.tired and L["tip.muscle.fresh"] or nil
end

-- The line of a Start that runs another ladder (`via`): what opens the family (once: a closed family's card says it
-- above, Tree.shown.opensSaid), the step of another ladder the climb waits for; nil else (a Start on the ladder the
-- move scales by, whose label names the move it trains; a stretch check's what it opens, Tree.StartLines; a move that
-- waits for its stage, its card's own line).
function Tree.ViaLine(s, l, r, src)
	if s.via == "opens" and not Tree.shown.opensSaid and s.as ~= "check" then return Tree.Opens(l, true) end
	if s.via == "needs" then return Tree.NeedsLine(s, l, r, src) end
	return nil
end

-- The waits line of a `via` needs start (ns.StartFor's needStart): of what the card can be waiting on, the shown
-- move's or the step's move's own checks (a move waiting for a step of another ladder: the explosive push-up's 12
-- feet-up push-ups), on the climb from the player's step the checks of the path's move that holds it (ns.PathHeld,
-- 2026-10-17, slice 42's review: power step 6's knee clap push-up, whose 12 feet-up push-ups the step ahead's Start
-- climbs toward with no line saying why) and the next step's needs, the first not done whose step is on the ladder
-- the start runs, in Tree.WaitsWords' words (ns.NeedAt with the setup's filter); else the next step's first need
-- (Tree.WaitsFor).
function Tree.NeedsLine(s, l, r, src)
	local f, id = Tree.facts.f, Tree.shown.move or Tree.StepMove(l, r)
	local cur = src and src.rung or 0
	local w = ns.Ladders[l] and ns.Ladders[l].rungs[cur + 1]
	local held = src and r >= cur and ns.PathHeld(ns.state, l, f, cur)
	for _, list in ipairs({ id and ns.Library[id].checks or {}, held and ns.Library[held].checks or {},
		w and w.needs or {} }) do
		for _, o in ipairs(list) do
			local e = Tree.Alternative(o)
			if not ns.NeedDone(ns.state, o) and not ns.IsCheck(e) and e.ladder == s.ladder then
				return Tree.WaitsWords(ns.NeedAt(o, f))
			end
		end
	end
	return Tree.WaitsFor(l, cur + 1)
end

-- When the climb's day counts again (`dayKind`) and when a set is fresh (`fresh`, none on a step to earn: its own line
-- says it), neither where an earn line named the time (`said`), onto `out`.
function Tree.DayLines(out, s, fresh, said)
	if said then return end
	if s.dayKind == "session" then
		out[#out + 1] = L["tree.card.session"]
	elseif s.dayKind == "day" then
		out[#out + 1] = L["tree.card.day"]:format(Sheet.ClockTime(s.dayIn))
	end
	if fresh then out[#out + 1] = L["tree.card.fresh"]:format(Sheet.ClockTime(fresh)) end
end

-- Why a Start cannot run (Lc.StartBlock's reasons, and Lc.Unsafe's for a pump asked for at the click), in the one
-- wording the greyed Start's line and tooltip, the card's red line at a refused click and the chat share (docs/ui.md,
-- "A greyed Start"; "pump" turns the Start into "Open your waiting pump" instead): felt unwell as the trees' Next line
-- says it, the lock's end in the client's clock (Lc.LockEnd); every other reason its tree.block key. The setup's line
-- ends in its link ("First finish the setup: Your setup"): its second value says so, the card's link.
function Tree.BlockLine(why)
	if why == "locked" then return L["tree.next.rest"]:format(Sheet.ClockTime(Lc.LockEnd())) end
	if why == "setup" then return L["tree.link.line"]:format(L["tree.block.setup"]), true end
	return L["tree.block." .. why]
end

-- The same as one line of text, the link's words after the colon: the greyed Start's tooltip and the chat.
function Tree.BlockText(why)
	local line, link = Tree.BlockLine(why)
	return link and line .. " " .. L["tree.link.setup"] or line
end

-- The Start: under the head where it runs the move the card shows, else (`away`: a step ahead, a kept step, a family
-- not open yet, a start that runs another ladder, the player's own step whose Start trains a stage of its path or a
-- stretch check, an easier move stepped down to or a placement) under "To get here" beside the tile of the move it
-- trains in its step's quality, which shows that step's card, or that move in this one where the step is this one, so
-- the clicked step's figure never stands over another move's button; on the player's own step (`ownStep`) the label
-- reads "Train with" (the apps' foot.own), since that step is the player's already, and none where the face's road
-- said "To get here" already (Tree.shown.roadSaid). On the player's own cardio step the Long Haul (`haul`:
-- Tree.HaulChoice) stands beside it where the Start keeps its verb and move there, else under it. Then its lines
-- (Tree.StartFace).
function Tree.CardStart(card, s, l, r, away, ownStep, haul)
	local shown, z = Tree.shown, card.at
	shown.choice = s
	local b, x = card.start, 0
	b:ClearAllPoints()
	if away then
		if ownStep or not Tree.shown.roadSaid then
			Tree.CardSection(card, L[ownStep and "tree.card.trainWith" or "tree.card.toGetHere"], 12)
		end
		local q = s.ladder and ns.Quality(s.ladder, s.rung) or ns.MoveQuality(s.move)
		local scale = TILE / SLOT
		card.mine:ClearAllPoints()
		card.mine:SetPoint("TOPLEFT", z, "TOPLEFT", 3 / scale, -(z.y + 3) / scale)
		setTile(card.mine, s.move, q)
		card.mine.step = s.ladder and not (s.ladder == l and s.rung == r) and { s.ladder, s.rung } or false
		x = TILE + 14
		b:SetPoint("TOPLEFT", z, "TOPLEFT", x, -(z.y + 3 + (TILE - CARD.BUTTON) / 2))
		z.y = z.y + 3 + TILE + 3
	else
		local y = z.y + gap(z, 10)
		b:SetPoint("TOPLEFT", z, "TOPLEFT", 0, -y)
		z.y = y + CARD.BUTTON
	end
	local block = Lc.StartBlock()
	-- as wide as its words need, at most the room beside it: past that the verb and the move alone (the goal goes into
	-- its tooltip with the whole label, shown.whole), and words that still outrun it end in the client's ellipsis, so
	-- no word ever runs past the button
	local room, label = z.w - x, block == "pump" and L["tree.start.waiting"] or Tree.StartLabel(s)
	local words = b.Text
	words:SetWidth(0)
	words:SetWordWrap(true)
	local beside = false
	if haul then
		local haulW = Tree.CardHaul(card, haul, block)
		b:SetText(block == "pump" and label or Tree.StartLabel(s, true))
		beside = (ns.Readable(b:GetTextWidth()) or 160) + 40 + 8 + haulW <= room
		if beside then room = room - haulW - 8 end
	end
	b:SetText(label)
	local textW = ns.Readable(b:GetTextWidth()) or 160
	shown.whole = nil
	if textW + 40 > room then
		shown.whole = label
		if block ~= "pump" then
			b:SetText(Tree.StartLabel(s, true))
			textW = ns.Readable(b:GetTextWidth()) or 160
		end
		if textW + 40 > room then
			words:SetWidth(room - 24)
			words:SetWordWrap(false)
		end
	end
	b:SetWidth(math.max(CARD.PRIMARY, math.min(room, textW + 40)))
	b:SetEnabled(block == nil or block == "pump")
	shown.why, shown.block = block and Tree.BlockText(block) or nil, block
	b:Show()
	if haul then
		card.haul:ClearAllPoints()
		if beside then
			card.haul:SetPoint("LEFT", b, "RIGHT", 8, 0)
		else
			card.haul:SetPoint("TOPLEFT", b, "BOTTOMLEFT", 0, -6)
			z.y = z.y + 6 + CARD.BUTTON
		end
	end
	Tree.StartFace(card, s, l, r, block)
end

-- The Long Haul on the player's own cardio step's card (slice 55; David's call 1, 2026-10-23): its start
-- (ns.HaulStart) and its words at the length open, "Long Haul 1:30" (ns.Haul).
function Tree.HaulChoice()
	return { start = ns.HaulStart(ns.state, ns.profile, Tree.facts.now),
		words = L["tree.haul"]:format(ns.Panel.Clock((ns.Haul(ns.state, Tree.facts.now)))) }
end

-- The Long Haul's button laid out (Tree.CardStart places it): its words, greyed while the Start's block holds (a
-- waiting pump's too: its Start opens that pump, the haul waits for it); its width.
function Tree.CardHaul(card, haul, block)
	local b, shown = card.haul, Tree.shown
	shown.haul = haul.start
	shown.haulWhy = block and Tree.BlockText(block) or nil
	b:SetText(haul.words)
	local w = math.max(CARD.BACK, (ns.Readable(b:GetTextWidth()) or 80) + 40)
	b:SetWidth(w)
	b:SetEnabled(shown.haulWhy == nil)
	b:Show()
	return w
end

-- The Long Haul's click: its start runs as the card's Start's (Tree.Run), a refused click said on the card and in chat.
function Tree.StartHaul()
	local s = Tree.card and Tree.shown.haul
	if s then Tree.Run(s, { haul = true }) end -- marked, so a ladder's restart asks its start again (Lc.Cleared)
end

-- The Start's lines on the face, at most two in the notes' grey: why it is greyed (Lc.StartBlock), else the passing
-- state a click met (`refused`) while the Start still does what that click asked (`refusedPump`: over a waiting pump's
-- line where it kept that pump from opening; never once a pump comes to wait after a refused start, nor once the pump
-- it kept goes), in red but a waiting pump's line (its Start opens it) and felt unwell's rest line (the care tone:
-- red is never the body's, docs/ui.md "The colour rule"), with the setup's link; else the first of its lines
-- (Tree.StartLines); then the tired line where it applies. The rest goes into the Start's tooltip (shown.lines), and
-- past the card's cap (card.fold 4) the tired line too.
function Tree.StartFace(card, s, l, r, block)
	local shown = Tree.shown
	local lines, tired = Tree.StartLines(s, l, r)
	local why, pump = block, block == "pump"
	if (block == nil or pump) and shown.refused and pump == (shown.refusedPump == true) then why = shown.refused end
	if why then
		local line, link = Tree.BlockLine(why)
		Tree.CardLine(card, line, "bodyWhite", 8, (why == "pump" or why == "locked") and saga.note or RED_FONT_COLOR)
		if link then Tree.CardLink(card, L["tree.link.setup"], function() Setup.Open() end) end
	else
		Tree.CardLine(card, table.remove(lines, 1), "bodyWhite", 8, saga.note)
	end
	if tired and card.fold >= 4 then
		lines[#lines + 1] = tired
	elseif tired then
		Tree.CardLine(card, tired, "bodyWhite", 2, saga.note)
	end
	shown.lines = lines
end

-- The path's stages ---------------------------------------------------------------------------------------------------

-- A stage's moves as one name (docs/ui.md, "The step's groups"): of its alternatives those the setup offers (ns.Passes
-- with the setup alone, no pain flag: an alternative the setup leaves out stays out of the words), both joined where it
-- offers two ("Pistol, hands on the desk or Ring-assisted pistol"), with whether either counts; else the stage's move.
function Tree.StageName(st)
	local offered = Tree.StageOffered(st)
	if #offered >= 2 then return L["tree.stage.or"]:format(moveName(offered[1]), moveName(offered[2])), true end
	return moveName(st.move), false
end

-- A stage's alternatives the setup offers (ns.Passes with the setup alone, no pain flag), in the path's order.
function Tree.StageOffered(st)
	local setup, offered = Tree.facts.setup, {}
	for _, id in ipairs(st.ids) do
		if ns.Passes(id, setup) then offered[#offered + 1] = id end
	end
	return offered
end

-- A stage of the climb from `rung` in words: a strength or hold move at its top on its days ("Pistol, hands on the
-- desk 8+ on 2 days": its own window's top, ns.MoveTop, with the weight its climb asks; two alternatives the setup
-- offers each at its own top, 2026-10-17, slice 42's review: "Sliding one-arm push-up 18+ or One-arm push-up, hand on
-- the bed 12+ on 2 days", where one top for both named the sliding one's), a pose close enough or Perfect on its days,
-- a cardio form's clean days so far (a done stage's days whole, ns.Stages counting none on it: 2026-10-17).
function Tree.StageWords(l, rung, st)
	local name = Tree.StageName(st)
	local top = ns.MoveTop(l, rung, st.move)
	if POSE[l] then
		return L[top == ns.Enum.mark.perfect and "tree.stage.perfect" or "tree.stage.mark"]:format(name, days(st.days))
	end
	if l == ns.CARDIO then
		if not st.have then return L["tree.stage.cleanDone"]:format(name, days(st.days)) end
		return ns.Plural("tree.stage.clean", st.days):format(name, st.have, st.days)
	end
	local offered = Tree.StageOffered(st)
	if #offered >= 2 then
		local function at(id)
			local t = ns.MoveTop(l, rung, id)
			return t and Sheet.ClimbTop(id, t, true) or ""
		end
		return L["tree.stage.needOr"]:format(moveName(offered[1]), at(offered[1]), moveName(offered[2]), at(offered[2]),
			days(st.days))
	end
	return L["tree.stage.need"]:format(name, top and Sheet.ClimbTop(st.move, top, true) or "", days(st.days))
end

-- What a waiting stage or a waiting move opens after (ns.Stages' `after` and `top`, ns.MoveOpen's): a strength or hold
-- move at its top ("Opens after Bulgarian split squat at 25 a side"), a pose's first Good or Perfect day; with
-- `sentence` the same as a sentence with its full stop (a held line's, Tree.WaitWords); nil where it opens with the
-- step.
function Tree.OpensAfter(after, top, sentence)
	if not after then return nil end
	local m = ns.Library[after]
	if POSE[m.ladder] then
		local key
		if top == ns.Enum.mark.perfect then
			key = sentence and "tree.held.opensPerfect" or "tree.stage.opensPerfect"
		else
			key = sentence and "tree.held.opensMark" or "tree.stage.opensMark"
		end
		return L[key]:format(moveName(after))
	end
	local amount = top and Sheet.ClimbTop(after, top) or ""
	if m.side then amount = L["tree.stage.side"]:format(amount) end
	return L[sentence and "tree.held.opens" or "tree.stage.opens"]:format(moveName(after), amount)
end

-- Why a skipped stage is skipped (every move of it shut): by the player's own list where that alone shuts its move
-- ("You left …"), else by the setup (ns.Shut's reasons, as a refused Start reads them).
function Tree.SkipOff(st)
	for _, w in ipairs(ns.Shut(st.move, Tree.facts.setup)) do
		if w.reason ~= "excluded" then return false end
	end
	return true
end

-- A skipped stage's state ("You left it out", "Left out by your setup"): the row's name names the move, and its
-- skipped pip that the next step does not wait for it.
function Tree.SkipLine(st)
	return L[Tree.SkipOff(st) and "tree.stage.skipOff" or "tree.stage.skipSetup"]
end

-- A skipped stage's link (docs/ui.md, "The steps' order": what to change): its move's Include in pumps where the
-- player's own list leaves it out, else Your setup at the page of the setup's first rule that does (Tree.Refusal's
-- link, as a refused Start's).
function Tree.SkipLink(card, st, l)
	if Tree.SkipOff(st) then
		return Tree.CardLink(card, L["tree.link.include"], function() ns.Settings.SetExcluded(st.move, false) end)
	end
	local _, link, page = Tree.Refusal("setup", { move = st.move }, l)
	if link == "setup" then Tree.CardLink(card, L["tree.link.setup"], function() Setup.Open(page) end) end
end

-- A move's row in a card's group (a two-line kit row: Your main move's, a stage's): the move's key pose in a screen
-- (a stage's two alternatives the setup offers side by side; in its quality's edge once done; greyed while it waits,
-- dimmed with the row while skipped), the objective tracker's check before a stage's words when done, its day
-- pips (the check a day, the nub a day to go) while open or waiting, a dimmed nub while skipped; under them what
-- opens it or why it is skipped (a stage of two alternatives says so in its name: "X or Y"); a click shows its move in
-- place with its own Start (a waiting move's the Start of the move that opens it: ns.StartFor). Past the card's cap
-- (card.fold 3) its second line goes to its tooltip alone.
function Tree.MoveRow(list)
	local row
	row = Kit.Row(list, { picture = true, name = "", line = "", below = true, font = "bodyWhite", chevron = true,
		onClick = function() Tree.ShowMove(row.move) end,
		tip = function(tooltip) Kit.Tip(row.words, row.under, { hint = L["tree.click.card"] })(tooltip) end })
	return row
end

function Tree.FillStage(card, row, l, rung, st)
	local words = Tree.StageWords(l, rung, st)
	local _, either = Tree.StageName(st)
	local mark = st.state == "done" and Style.Check() or Style.Pips(st.have, st.days)
	local under
	if st.state == "waiting" then
		under = Tree.OpensAfter(st.after, st.top)
	elseif st.state == "skipped" then
		under = Tree.SkipLine(st)
		mark = Style.Pips(0, 1)
	end
	row.move, row.words, row.under = st.move, words, under
	row.name:SetText(mark .. " " .. words)
	row.line:SetText(under or "")
	row.line:SetShown(under ~= nil and card.fold < 3)
	Tree.RowPictures(row, either and Tree.StageOffered(st) or { st.move })
	Kit.PaintScreen(row.screen, st.state == "done" and ns.MoveQuality(st.move) or nil)
	Kit.Choose(row, st.state ~= "skipped", true)
	for _, t in ipairs(row.icons) do t:SetDesaturated(st.state == "waiting" or st.state == "skipped") end
	row:SetAlpha(st.state == "skipped" and SKIPPED_A or 1)
end

-- A move row's picture: one move's key pose at 40 in its screen, or two side by side at 24 (a stage's alternatives,
-- either of which counts), the second made at its first need.
function Tree.RowPictures(row, ids)
	local one, two = row.icons[1], row.icons[2]
	if #ids >= 2 and not two then
		two = Kit.Icon(row.screen, nil, 24)
		row.icons[2] = two
	end
	one:ClearAllPoints()
	if #ids >= 2 then
		one:SetSize(24, 24)
		one:SetPoint("LEFT", 2, 0)
		two:ClearAllPoints()
		two:SetPoint("RIGHT", -2, 0)
		Kit.SetIcon(two, ids[2])
		two:Show()
	else
		one:SetSize(40, 40)
		one:SetPoint("CENTER")
		if two then two:Hide() end
	end
	Kit.SetIcon(one, ids[1])
end

-- The climb from `rung`'s path as rows, in order (ns.Stages with the filter), each stage whose moves wait for what
-- opens them followed by those rows (Tree.MovesWait: a stretch check's row, a step of another ladder's need row);
-- nothing where the climb has no path. `label`: under the caps "Then, for the next step" (the step's own group).
function Tree.CardStages(card, l, rung, label)
	local stages = ns.Stages(ns.state, l, rung, Tree.facts.f)
	if #stages == 0 then return end
	if label then Tree.CardSection(card, L["tree.card.group.then"]) end
	local list = Tree.CardList(card, "stage")
	for _, st in ipairs(stages) do
		Tree.FillStage(card, Tree.ListRow(list, Tree.MoveRow), l, rung, st)
		local needs = st.state ~= "done" and st.state ~= "skipped" and Tree.MovesWait(st.ids) or {}
		if #needs > 0 or st.state == "skipped" then
			Tree.CardLayList(card, list, label and 0 or 8)
			label = nil
			if st.state == "skipped" then Tree.SkipLink(card, st, l) end
			Tree.CardWaits(card, needs, { move = st.move, name = (Tree.StageName(st)) })
			list = Tree.CardList(card, "stage")
		end
	end
	Tree.CardLayList(card, list, label and 0 or 8)
end

-- Stretch checks --------------------------------------------------------------------------------------------------

-- Every stretch check the rules hold (slice 42): Progress.lua ns.AllChecks, the one list the desktop app's engine reads
-- too, as { need, group, opens = { ladder, rung } | { move } | { family } }.
function Tree.AllChecks()
	return ns.AllChecks()
end

-- The pose a check runs (ns.NeedAt with the setup alone, as ns.StartFor's check does).
function Tree.CheckPose(need)
	return (ns.NeedAt(need, ns.FilterContext(ns.profile, nil)))
end

-- The live stretch checks that pose `id` answers (ns.ChecksOf: not done, their opening not open already, in the rules'
-- order): what a card's Start of a check names (Tree.StartedCheck); a pump's pose reads them through ns.PoseChecks
-- (Panel.lua readChecks).
function Tree.ChecksOf(id)
	return ns.ChecksOf(ns.state, id)
end

-- The check row the Start of check row `c` runs (ns.RunsCheck with the setup alone, 2026-10-18, slice 42's last review:
-- the twist's step 5 check runs the pike's big toe fold first, ns.CheckFirst), `c` itself where start `s` runs no
-- stretch check (a step of a strength or cardio ladder first, `via` needs): the pump's note and Result name it
-- (Panel.lua readChecks).
function Tree.RunsCheck(c, s)
	if s.as ~= "check" then return c end
	return ns.RunsCheck(ns.state, c, ns.FilterContext(ns.profile, nil)) or c
end

-- The check a card's Start of kind "check" runs (ns.StartFor's checkStart, its pose `s.move` at the least answer
-- `s.ask`): the check row the card shows for it (Tree.CardChecks, Tree.shown.startCheck) as its Start runs it
-- (Tree.RunsCheck), else the first live check its pose answers at that answer (Tree.ChecksOf); nil where none is (the
-- line then names nothing).
function Tree.StartedCheck(s)
	if Tree.shown.startCheck then return Tree.RunsCheck(Tree.shown.startCheck, s) end
	local checks = Tree.ChecksOf(s.move)
	for _, c in ipairs(checks) do
		if ns.CheckAsk(c.need) == s.ask then return c end
	end
	return checks[1]
end

-- A check's row words: "{pose}: Good or better, once", or Perfect where only Perfect passes it (ns.CheckAsk); with
-- `sentence` what it holds as a sentence, "Opens after your first Good hold of {pose}." (a held line's:
-- Tree.WaitWords).
function Tree.CheckNeed(need, sentence)
	local perfect = ns.CheckAsk(need) >= ns.Enum.mark.perfect
	local key
	if sentence then
		key = perfect and "tree.held.checkPerfect" or "tree.held.check"
	else
		key = perfect and "tree.check.needPerfect" or "tree.check.need"
	end
	return L[key]:format(moveName(Tree.CheckPose(need)))
end

-- What a check not done opens, in its row's words: a move ("One hold at its goal pose opens Pistol, hands on the
-- desk."; a stage's alternatives by its name, `opens.name`), a family's first step, or the climb to a step; Perfect
-- where only Perfect passes it.
function Tree.CheckLine(c)
	local perfect = ns.CheckAsk(c.need) >= ns.Enum.mark.perfect
	local o = c.opens
	if o.ladder then
		local key = perfect and "tree.check.lineStepPerfect" or "tree.check.lineStep"
		return L[key]:format(ns.Step(o.ladder, o.rung) or o.rung)
	end
	local name = o.name or moveName(o.move or Tree.StepMove(o.family, 1))
	return L[perfect and "tree.check.linePerfect" or "tree.check.line"]:format(name)
end

-- A check done (its entry `need` of the entry or `any` group `group`), in its row's words (ns.CheckPassed, the desktop
-- app's and the phones' one rule): the game day it was first passed ("Passed 16 Oct", the earliest of the group's
-- checks; 2026-10-17, slice 42's review: the latest Good or Perfect day until then, which moved with every later hold),
-- else, a step its family stood above with no answer that did it, the step now of the family of the choice that is
-- done, the shown one where it is ("Passed: your Ankle is at step 4.").
function Tree.CheckDone(need, group)
	local ts, l, n = ns.CheckPassed(ns.state, need, group)
	if ts then return L["tree.check.done"]:format(Sheet.ShortDate(ns.DayMid(ts, 0))) end
	return L["tree.check.doneStep"]:format(L["ladder." .. l], n)
end

-- A check's row: the pose's key pose in a screen, its words, under them what it opens while undone or that it passed
-- (the tracker's check before its words); a click shows the pose in place.
function Tree.CheckRow(list)
	local row
	row = Kit.Row(list, { picture = true, name = "", line = "", below = true, font = "bodyWhite", chevron = true,
		onClick = function() Tree.ShowMove(row.move) end,
		tip = function(tooltip) Kit.Tip(row.words, row.under, { hint = L["tree.click.card"] })(tooltip) end })
	return row
end

-- The stretch check rows `checks` under the caps "Stretch check" (docs/ui.md, "Stretch checks"): each a framed row
-- (Tree.CheckRow) and, while not done, the grey "Do the check" under it at the right (none where the card's Start is
-- that same check, which reads "Do the check" itself; greyed with the refusal's words where the setup or a pain flag
-- refuses it, or why no Start can run now).
function Tree.CardChecks(card, checks)
	if #checks == 0 then return end
	Tree.CardSection(card, L["tree.card.check"], 10)
	local first = true
	for _, c in ipairs(checks) do
		local done = ns.NeedDone(ns.state, c.group)
		local pose = Tree.CheckPose(c.need)
		local list = Tree.CardList(card, "check")
		local row = Tree.ListRow(list, Tree.CheckRow)
		row.move, row.words = pose, Tree.CheckNeed(c.need)
		row.under = done and Tree.CheckDone(c.need, c.group) or Tree.CheckLine(c)
		row.name:SetText((done and Style.Check() or Style.Pips(0, 1)) .. " " .. row.words)
		row.line:SetText(row.under)
		row.line:SetShown(card.fold < 3)
		Kit.SetIcon(row.icons[1], pose)
		Kit.PaintScreen(row.screen, done and ns.MoveQuality(pose) or nil)
		Kit.Choose(row, true, true)
		Tree.CardLayList(card, list, first and 0 or 8)
		first = false
		-- the card's Start runs what this row's "Do the check" runs (Tree.RunsCheck: this check, or the one its climb
		-- waits for first): it carries the entry of the check that runs (Tree.Start through Tree.StartedCheck)
		local s = Tree.shown.cardChoice
		local run = s and s.as == "check" and Tree.RunsCheck(c, s)
		local same = run and s.move == Tree.CheckPose(run.need) and ns.CheckAsk(run.need) == s.ask
		if same and not done and not Tree.shown.startCheck then Tree.shown.startCheck = c end
		if not done and not same then
			local start, why, subject = ns.StartFor(ns.state, ns.profile, { check = c.need }, Tree.facts.now)
			local block = start and Lc.StartBlock()
			local reason = not start and (Tree.Refusal(why, subject, c.need.ladder))
				or block and block ~= "pump" and Tree.BlockText(block) or nil
			Tree.CardButton(card, L["tree.start.check"], function() Tree.StartCheck(c) end, reason)
		end
	end
end

-- What moves `ids` wait for before they open (their `checks` not done, of the moves the setup offers), each once where
-- two ask the same (the desk and the ring pistols' ankle step 2): the entries, `any` groups whole. A stage's
-- alternatives (Tree.CardStages), Your main move alone (Tree.CardMain).
function Tree.MovesWait(ids)
	local out, seen = {}, {}
	for _, id in ipairs(ids) do
		if ns.Passes(id, Tree.facts.setup) then
			for _, o in ipairs(ns.Library[id].checks or {}) do
				local key = Tree.NeedKey(o)
				if not seen[key] and not ns.NeedDone(ns.state, o) then
					seen[key] = true
					out[#out + 1] = o
				end
			end
		end
	end
	return out
end

-- A need entry as one key (its ladder, step, move, count and mark, an `any` group's entries joined), so two moves
-- asking the same are told as one.
function Tree.NeedKey(o)
	if o.any then
		local parts = {}
		for i, e in ipairs(o.any) do parts[i] = Tree.NeedKey(e) end
		return table.concat(parts, "|")
	end
	return table.concat({ o.ladder, o.step, o.move or "", o.count or "", o.mark or "" }, ":")
end

-- Of an `any` group, the entry the setup offers a move for (ns.NeedAt with the setup; the first where none), as a
-- Start runs it; any other entry itself.
function Tree.Alternative(o)
	if not o.any then return o end
	local setup = Tree.facts and Tree.facts.setup or ns.FilterContext(ns.profile, nil)
	for _, e in ipairs(o.any) do
		local id = ns.NeedAt(e, setup)
		if id and ns.Passes(id, setup) then return e end
	end
	return o.any[1]
end

-- "Do the check" from a check row: the check's own Start (ns.StartFor's target { check }), carrying the entry of the
-- check it runs (Tree.RunsCheck: this one, or the check its climb waits for first), whose row the pump's note and
-- Result name first (Panel.lua readChecks, ns.PoseChecks: its `any` group, whether it is the last wait, what it
-- opens), and the return to the pose's family (`via` check); refused, the card says why as a refused Start does.
function Tree.StartCheck(c)
	local s, why, subject = ns.StartFor(ns.state, ns.profile, { check = c.need }, ns.now())
	if not s then
		ns.Chat("start", (Tree.Refusal(why, subject, c.need.ladder)))
		return
	end
	s.via = s.via or "check"
	-- a start that runs no stretch check (a record first) keeps this one: Lc.Cleared starts it again after a ladder's
	-- restart, and a pump's strength move reads no check's words
	Tree.Run(s, { check = Tree.RunsCheck(c, s).need })
end

-- The steps of other ladders -----------------------------------------------------------------------------------------

-- The steps of other ladders rung `rung`'s climb waits for (its `needs`; Tree.CardWaits), what each check among them
-- opens the climb into `rung`.
function Tree.CardNeeds(card, l, rung)
	local w = ns.Ladders[l].rungs[rung]
	if not (w and w.needs) then return end
	Tree.CardWaits(card, w.needs, { ladder = l, rung = rung })
end

-- What `needs` wait for as rows (entries of a rung's `needs` or a move's `checks`, an `any` group whole: its
-- alternative the setup offers, as ns.StartFor's needStart picks it), each opening `opens` (Tree.CheckLine's): a step
-- of another ladder a need row in a framed list (Tree.FillNeed), a stretch check a check row (Tree.CardChecks).
function Tree.CardWaits(card, needs, opens)
	local list, checks = Tree.CardList(card, "need"), {}
	for _, o in ipairs(needs) do
		local e = Tree.Alternative(o)
		if ns.IsCheck(e) then
			checks[#checks + 1] = { need = e, group = o, opens = opens }
		else
			Tree.FillNeed(card, Tree.ListRow(list, Tree.NeedRow), o, e)
		end
	end
	Tree.CardLayList(card, list, 8)
	Tree.CardChecks(card, checks)
end

-- A step of another ladder as a need row: that step's chip words (an `any` group's alternatives joined, done when one
-- is) with a check where it is done and the objective tracker's nub where not (WoW's own tracker marks an undone
-- objective so: the game's form of the apps' dash), each by the engine's own test (ns.NeedDone, as the links' colour),
-- never a guess from the first one undone; under an undone one's name what it waits for, from the engine's one rule
-- for that need (ns.NeedAt with the setup's filter: a move the player can do), in Tree.WaitsWords' words, and in the
-- row's tooltip too, where it goes alone past the card's cap (card.fold 3); a click shows that step's card (`e`, the
-- alternative the setup offers).
function Tree.FillNeed(card, row, o, e)
	local done = ns.NeedDone(ns.state, o)
	local id, count = ns.NeedAt(o, Tree.facts.f)
	local title
	for _, x in ipairs(o.any or { o }) do
		local words = Tree.ChipText(x.ladder, ns.RungAtStep(x.ladder, x.step))
		title = title and L["tree.need.or"]:format(title, words) or words
	end
	row.step, row.title = { e.ladder, ns.RungAtStep(e.ladder, e.step) }, title
	row.waits = not done and id and Tree.WaitsWords(id, count) or nil
	row.name:SetText((done and Style.Check() or Style.Pips(0, 1)) .. " " .. title)
	row.line:SetText(row.waits or "")
	row.line:SetShown(row.waits ~= nil and card.fold < 3)
end

function Tree.NeedRow(list)
	local row
	row = Kit.Row(list, { name = "", below = true, line = "", font = "bodyWhite", chevron = true,
		onClick = function() Tree.ShowStep(row.step[1], row.step[2]) end,
		tip = function(tooltip) Kit.Tip(row.title, row.waits, { hint = L["tree.click.card"] })(tooltip) end })
	Kit.Choose(row, false, true)
	return row
end

-- The hint row (a soft need, never a gate; docs/ui.md "Stretch checks"): the pose that helps move `id` (Library.lua
-- `hint`), only where the setup offers that pose (ns.Passes), as a row: its key pose and "Helps: Downward dog", a
-- click showing the pose in place. The move's own advice (move.<key>.hint) follows its How's steps (Tree.CardMore) and
-- ends the pump card's tips (Panel.MoveTips), never a line under the row (2026-10-22).
function Tree.CardHint(card, id)
	local pose = id and ns.Library[id].hint
	if not (pose and ns.Passes(pose, Tree.facts.setup)) then return end
	local list = Tree.CardList(card, "hint")
	local row = Tree.ListRow(list, function(lst)
		local r
		r = Kit.Row(lst, { picture = true, name = "", below = true, font = "bodyWhite", chevron = true,
			onClick = function() Tree.ShowMove(r.move) end,
			tip = function(tooltip) Kit.Tip(r.words, nil, { hint = L["tree.click.card"] })(tooltip) end })
		return r
	end)
	row.move, row.words = pose, L["tree.card.hint"]:format(moveName(pose))
	row.name:SetText(row.words)
	Kit.SetIcon(row.icons[1], pose)
	Kit.PaintScreen(row.screen, nil)
	Kit.Choose(row, true, true)
	Tree.CardLayList(card, list, 10)
end

-- The climb -----------------------------------------------------------------------------------------------------------

-- What opens the next step from the player's step `src.rung`, with its day pips (slices 20-22's step lines): the move
-- at its top on its days, a pose's days close enough, a counted cardio rung's clean days, a timed one's full moves as
-- its Full moves row (slice 52; its ready days' line until then, with no line under it that says what such a day is:
-- David, 2026-10-22); a pain flag holding it says so instead, and a shut
-- climb (ns.UnlockProgress's need 0: another ladder's step, the setup, a stand-in from below) what holds it
-- (Tree.ClosedLine; the step it waits for in that step's own row alone, Tree.CardNeeds), never its days at zero. A
-- ladder back from a break says what finds its step again (Tree.PlacingLine), with no climb and no pips: that move
-- wipes the days the climb counted (ns.TreeEntry). Answers whether a climb is there to show its path's rows and its
-- needs after it (Tree.CardRoad, Tree.CardFace): not on the top or back from a break, and a climb a pain flag holds
-- keeps them.
function Tree.ClimbHead(card, l, src)
	local state, rung = ns.state, src.rung
	if rung >= ns.HeldTop(l) then
		Tree.CardLine(card, L["ladders.path.top"], "bodyWhite", 8)
		return false
	end
	if Tree.facts.entries[l].placing then
		Tree.CardLine(card, Tree.PlacingLine(Tree.facts.entries[l]), "bodyWhite", 8)
		return false
	end
	if ns.PainOnLadder(state, l, rung, Tree.facts.f) then
		Tree.CardLine(card, L["ladders.step.pain"], "bodyWhite", 8)
		return true -- held, never shut: its path's moves and needs stand in their rows
	end
	if src.progress[2] == 0 then
		Tree.CardLine(card, Tree.ClosedLine(l, rung, true), "bodyWhite", 8)
		return true
	end
	if l == ns.CARDIO and not Tree.Counted(rung) then
		local have, need = src.progress[1], src.progress[2]
		Tree.CardPair(card, L["tree.card.full"], L["tree.card.fullValue"]:format(have, need) .. " " .. Style.Pips(have,
			need), 8)
	else
		Tree.CardLine(card, (Tree.ClimbLine(l, src, Tree.facts.f)), "bodyWhite", 8)
	end
	local short = POSE[l] and Tree.PoseShort(l, rung, Tree.facts.f)
	if short then Tree.CardLine(card, short, "bodyWhite", 2, saga.note) end
	return true
end

-- Whether the card's climb line from the player's step shows (Tree.ClimbLine, which Tree.ClimbHead draws): below the
-- top, a climb open (ns.UnlockProgress's need above 0), not back from a break, no pain flag holding it.
function Tree.ClimbShows(l, src)
	if src.rung >= ns.HeldTop(l) or src.progress[2] == 0 or Tree.facts.entries[l].placing then return false end
	return not ns.PainOnLadder(ns.state, l, src.rung, Tree.facts.f)
end

-- Whether the card's climb from the player's cardio step counts a timed rung's full moves (Tree.ClimbHead's Full moves
-- row, Tree.ClimbShows), on a rung that climbs by no clean days (Tree.Counted).
function Tree.TimedClimb(l, src)
	return l == ns.CARDIO and Tree.ClimbShows(l, src) and not Tree.Counted(src.rung)
end

-- What a pose family's days toward its next step still lack beyond their count (slice 47, 2026-10-19; ns.PoseNeeds
-- on the player's step `rung` with filter context `f`): the days its main pose still needs, by its name (ns.PoseMain,
-- else the step's anchor), and the Perfect day (none asked where only Perfect counts), each a sentence; nil where
-- neither is short.
-- The card's climb line (Tree.ClimbHead) and the Ready view's days line (Panel.lua poseDaysLine) say it (the Result
-- view's note under the four answers too until 2026-10-22). `p` the family's ns.PoseNeeds where the caller holds it.
function Tree.PoseShort(l, rung, f, p)
	p = p or ns.PoseNeeds(ns.state, l, rung)
	local parts = {}
	if p.mainLeft > 0 then
		local main = ns.PoseMain(ns.state, l, f) or ns.Anchor(l, rung)
		parts[#parts + 1] = ns.Plural("ladders.step.shortMain", p.mainLeft):format(p.mainLeft, moveName(main))
	end
	if p.perfectLeft > 0 then
		parts[#parts + 1] = ns.Plural("ladders.step.shortPerfect", p.perfectLeft):format(p.perfectLeft)
	end
	return #parts > 0 and table.concat(parts, " ") or nil
end

-- The climb's line with its pips and the move standing for the rung where one names it (its top in its own unit); a
-- counted cardio rung's clean days (a timed one's full moves are a row, Tree.ClimbHead), and a counted cardio step
-- whose days and path are in its proving window with no pips (slice 53, ns.Proving: "Next: Burpee, one clean
-- window."), and after a try that did not pass this session that the climb waits.
function Tree.ClimbLine(l, src, f)
	local rung = src.rung
	local have, need = src.progress[1], src.progress[2]
	local proving, form
	if l == ns.CARDIO then proving, form = ns.Proving(ns.state, f, Tree.facts.now) end
	if proving == "waits" then return L["ladders.step.climbWaits"] end
	if proving then return L["ladders.step.nextProving"]:format(moveName(form)) end
	if l == ns.CARDIO then return L["ladders.step.nextClean"]:format(have, need) .. " " .. Style.Pips(have, need) end
	local id = ns.RungMove(l, rung, f)
	if POSE[l] then
		-- any goal pose of the step counts a day (slice 47), Perfect alone below a risky step (ns.MarkNeeds), as the
		-- hover and the Next line say; what the days still lack is the line under it (Tree.PoseShort)
		local key = ns.MarkNeeds(l, rung) == ns.Enum.mark.perfect and "ladders.step.nextPerfect" or "ladders.step.nextMark"
		return ns.Plural(key, need):format(need, have, need) .. " " .. Style.Pips(have, need), id
	end
	local hi
	if id then hi = select(2, Tree.Window(l, rung, id)) end
	return ns.Plural("ladders.step.next", need):format(id and moveName(id) or "", hi and Sheet.ClimbTop(id, hi, true)
		or "", need, have, need) .. " " .. Style.Pips(have, need), id
end

-- What reaching step (l, r) above the player's takes, under the hairline before the Start, under the caps "To get
-- here" (Tree.shown.roadSaid, so the Start's foot does not say it again): on the step above the player's (next, or
-- kept after a step down) the climb from the player's step (Tree.ClimbHead) with its path's rows (Tree.CardStages on
-- the player's step); further up the Path's need, the climb from the step under it in that step's own measure
-- (Tree.OpensLine; "Comes back at" on a kept step) with that climb's rows, their states on every kind; then each step
-- of another ladder and each stretch check that climb waits for (Tree.CardNeeds). None where the setup or the
-- player's list stops the way (`stopped`: the refusal names it).
function Tree.CardRoad(card, l, r, state, src, stopped)
	if stopped or state == "owned" then return end
	Tree.CardSection(card, L["tree.card.toGetHere"], 10)
	Tree.shown.roadSaid = true
	if r == src.rung + 1 then
		if Tree.ClimbHead(card, l, src) then
			Tree.CardStages(card, l, src.rung)
			Tree.CardNeeds(card, l, r)
		end
		return
	end
	Tree.CardLine(card, Tree.OpensLine(l, r - 1, nil, state == "kept" and "tree.key.back" or "tree.key.next"))
	Tree.CardStages(card, l, r - 1)
	Tree.CardNeeds(card, l, r)
end

-- The groups (slice 42, docs/ui.md "The steps' order") -------------------------------------------------------------

-- Your main move (docs/ui.md, "The steps' order"): the move standing for the step as a row (its key pose in its
-- quality's edge where the player has it, its name, and where it stands in for the anchor under the setup "Wall pistol
-- stands in for Pistol with your setup."; a click shows it in place) with what it waits for before it opens (its own
-- checks: the explosive push-up's 12 feet-up push-ups), then on the player's own step (`src`) what opens the next step
-- (Tree.ClimbHead), whose answer, whether the climb is open, it returns.
function Tree.CardMain(card, l, r, src)
	Tree.CardSection(card, L["tree.card.group.main"])
	local main, anchor = Tree.StepMove(l, r), ns.Anchor(l, r)
	if main then
		local list = Tree.CardList(card, "main")
		local row = Tree.ListRow(list, Tree.MoveRow)
		local standin = l ~= ns.CARDIO and anchor and main ~= anchor
			and L["tree.card.standin"]:format(moveName(main), moveName(anchor)) or nil
		row.move, row.words, row.under = main, moveName(main), standin
		row.name:SetText(row.words)
		row.line:SetText(standin or "")
		row.line:SetShown(standin ~= nil and card.fold < 3)
		Tree.RowPictures(row, { main })
		local q, ahead = Tree.TileQuality(main)
		Kit.PaintScreen(row.screen, not ahead and q or nil)
		Kit.Choose(row, true, true)
		row.icons[1]:SetDesaturated(ahead == true)
		Tree.CardLayList(card, list, 0)
		Tree.CardWaits(card, Tree.MovesWait({ main }), { move = main })
	end
	return src ~= nil and Tree.ClimbHead(card, l, src)
end

-- For variety: the step's other moves (its own moves but its main move and its path's), never required, each a tile
-- with its topped mark, under the label that says what they are for (no line under it: David, 2026-10-22; "Same level"
-- with its line until then); past the card's cap (card.fold 1) its label stays as a More row ("For variety · 9 moves")
-- that opens the tiles in place.
function Tree.CardSame(card, l, r)
	local main, path = Tree.StepMove(l, r), {}
	for _, st in ipairs(ns.Path(l, r)) do
		for _, id in ipairs(st.ids) do path[id] = true end
	end
	local ids = {}
	for _, id in ipairs(Tree.StepMoves(l, r)) do
		if id ~= main and not path[id] then ids[#ids + 1] = id end
	end
	if #ids == 0 then return end
	if card.fold >= 1 then
		local z, more = card.at, card.same
		more.words:SetText(ns.Plural("tree.card.same.more", #ids):format(#ids))
		local y = z.y + gap(z, 14)
		more:ClearAllPoints()
		more:SetPoint("TOPLEFT", z, "TOPLEFT", 0, -y)
		more:Show()
		z.y = y + Kit.CONTROL_H
		if not more:IsOpen() then return end
	else
		Tree.CardSection(card, L["tree.card.group.same"])
	end
	Tree.CardTiles(card, ids, 6)
end

-- The face of step (l, r) (docs/ui.md, "The step's card"): on the player's own step the set that earns an orange or
-- gold step, then Your main move with the climb and the steps of other ladders and the stretch checks the climb waits
-- for (a need that holds the climb stands with the climb it holds: squat step 6's ankle step 4), then Then, for the
-- next step with the path's stages; what finds the step again where it is back from a break (Tree.PlacingLine). On a
-- step above it Your main move and its own path, what reaching it takes heading the Start's block after them
-- (Tree.CardRoad, Tree.CardStep). On every step then the hint row of its move and For variety.
function Tree.CardFace(card, l, r, state, src, entry)
	local move = Tree.StepMove(l, r)
	if state == "now" then
		if not src.placed then return end
		if entry.placing then return Tree.CardLine(card, Tree.PlacingLine(entry)) end
		local earn, earnMove, floor = ns.ToEarn(ns.state, l, Tree.facts.f)
		if earn == r and floor then
			Tree.CardLine(card, L["ladders.step.toEarn"]:format(moveName(earnMove), value(earnMove, floor)))
		elseif earn == r and POSE[l] then
			Tree.CardLine(card, L["ladders.step.toEarnMark"])
		end
		if Tree.CardMain(card, l, r, src) then
			Tree.CardNeeds(card, l, r + 1)
			Tree.CardStages(card, l, r, true)
		end
	else
		Tree.CardMain(card, l, r)
		if r < ns.HeldTop(l) then Tree.CardStages(card, l, r, true) end
	end
	Tree.CardHint(card, move)
	Tree.CardSame(card, l, r)
end

-- More on this step and the head ---------------------------------------------------------------------------------

-- The player's step's skill as its tier word (ns.Skill, the profession metaphor), which slice 20's ladder row carried;
-- nil on a ladder with no pattern to its move.
function Tree.SkillLine(l, src)
	local id = src.standing
	local pattern = id and ns.Library[id].pattern or (l == ns.CARDIO and "cardio") or nil
	if not pattern then return nil end
	return L["tree.card.skill"]:format(L["skill." .. select(2, ns.Skill(ns.state, pattern))], L["pattern." .. pattern])
end

-- More on this step, the quiet part (docs/ui.md, "The words"): How (the move's own steps, in the notes' grey, and its
-- advice where the move names a helping pose: move.<key>.hint) on a step ahead or a family not open yet, with its stop
-- line where the face could not keep it (`stop`); then on a step's card (`l`) the player's skill on their own step
-- (`skill`, the step's source), the steps it opens on other ladders as links under "This step opens" (each switches the
-- tab where it must), the ladder's top, the weight line, what one more piece of the setup opens here and on the cardio
-- ladder how much longer its moves run until slice 52 (its steps' Pace rows say it since). A step not placed yet says
-- how the start works through its Start alone ("Find
-- my step"; its paragraph went on 2026-10-22). The step's other moves, under "Other moves" here until slice 42, are the
-- face's groups.
function Tree.CardMore(card, l, r, move, opts)
	if opts.how then
		Tree.CardSection(card, L["ui.panel.how"])
		Tree.CardLine(card, ns.Panel.MoveText(move, "how"), "bodyWhite", 0, saga.note)
		local advice = rawget(L, "move." .. ns.Library[move].key .. ".hint")
		if advice then Tree.CardLine(card, advice, "bodyWhite", 6, saga.note) end
		if opts.stop then Tree.CardStop(card, move) end
	end
	if not l then return end
	if opts.skill then Tree.CardLine(card, Tree.SkillLine(l, opts.skill), "bodyWhite", 12) end
	local opens = {}
	for _, link in ipairs(Tree.Links()) do
		if link.from[1] == l and link.from[2] == r then opens[#opens + 1] = link.to end
	end
	if #opens > 0 then Tree.CardSection(card, L["tree.card.opens"]) end
	for i, to in ipairs(opens) do
		Tree.CardLink(card, Tree.ChipText(to[1], to[2]), function() Tree.ShowStep(to[1], to[2], true) end, i == 1 and 0 or 6)
	end
	local top = ns.HeldTop(l)
	if r < top then
		Tree.CardLine(card, L["tree.card.top"]:format(moveName(Tree.StepMove(l, top)), ns.Step(l, top) or top), "bodyWhite",
			12, saga.note)
	end
	local topLoad, topReps, lastLoad, last = ns.LoadRecord(ns.state, move)
	if topLoad then
		Tree.CardLine(card, L["tip.ladder.load"]:format(Sheet.LoadAmount(move, topLoad, topReps),
			Sheet.LoadAmount(move, lastLoad, last or topReps)), "bodyWhite", 4, saga.note)
	end
	for _, o in ipairs(ns.SetupOpens(ns.profile, ns.state, l)) do
		Tree.CardLine(card, ns.Plural("ladders.tip.opens", o.count):format(L["gear." .. o.key], o.count,
			Sheet.QualityWord(o.quality)), "bodyWhite", 4, saga.note)
	end
end

-- The head's measure for step (l, r): on the player's step its record on its range with the stars (Tree.CardNow),
-- nothing on a ladder not placed yet (its title's "not placed yet" and its Start, "Find my step", say it) nor while it
-- is back from a break (the face says what finds the step again);
-- on any other step its own limit with the player's record of its move (Tree.CardLimit) and, below the player's step,
-- when it was reached (the hover's key line); then on a step the player has its mastered line (Tree.CardMastery). A
-- timed cardio step's full moves show once: the climb's row or the Path's line says them where it counts them
-- (Tree.TimedClimb; the Path's from a timed rung under the step, Tree.OpensLine), as the desktop app's measure leaves
-- them to its `timed` line.
function Tree.CardMeasureHead(card, l, r, state, src, entry, move, stopped)
	if state ~= "now" then
		local climb = r == src.rung + 1
		local path = not climb and state ~= "owned" and not stopped
		Tree.CardLimit(card, l, r, move, climb and Tree.TimedClimb(l, src)
			or path and l == ns.CARDIO and not Tree.Counted(r - 1))
		if state == "owned" then Tree.CardLine(card, select(2, Tree.KeyLines(l, r)), "bodyWhite", 6) end
	elseif src.placed and not entry.placing then
		Tree.CardNow(card, l, r, src)
	end
	if state == "now" or state == "owned" or state == "kept" then Tree.CardMastery(card, l, r) end
end

-- Mastered on a step the player has (slice 42, docs/ui.md "Mastered"): "Mastered 16 Oct" (ns.Mastered, kept for
-- good), or before it "Toward Mastered: 4 of 7 moves at their top" over the moves the setup offers there
-- (ns.MasteryMoves, ns.MasteryLeft); nothing on a step with none.
function Tree.CardMastery(card, l, r)
	local at = ns.Mastered(ns.state, l, r)
	if at then
		return Tree.CardLine(card, L["tree.card.mastered"]:format(Sheet.ShortDate(ns.DayMid(at, 0))), "bodyWhite", 6,
			NORMAL_FONT_COLOR)
	end
	local need = #ns.MasteryMoves(l, r, Tree.facts.f)
	if need == 0 then return end
	local have = need - #ns.MasteryLeft(ns.state, l, r, Tree.facts.f)
	Tree.CardLine(card, ns.Plural("tree.card.mastery", need):format(have, need), "bodyWhite", 6, saga.note)
end

-- A step's own limit, on every step's card but the player's own (docs/ui.md, "The step's card"): a strength or hold
-- step's range in its move's own unit (Tree.Window), a counted cardio step's clean days and its window with the shown
-- form's best clean count, a timed one's Pace (slice 52: what its forms keep now, Tree.CardPace) and the full moves
-- its climb takes, a pose step's goal pose; then the player's record of the move where there is one. A timed step's
-- full moves show once a card: not on the ladder's top, which no climb leaves, nor where the card's climb or Path line
-- counts them already (`said`, Tree.CardMeasureHead).
function Tree.CardLimit(card, l, r, move, said)
	if l == ns.CARDIO then
		if Tree.Counted(r) then
			Tree.CardClean(card, r)
		else
			Tree.CardPace(card, r)
			if not said and r < ns.HeldTop(l) then
				Tree.CardPair(card, L["tree.card.full"], tostring(C.cardio_full_days), 2)
			end
		end
	elseif POSE[l] then
		local id = ns.MainPose(l, r, move) and move or ns.Anchor(l, r)
		if id and ns.MainPose(l, r, id) then Tree.CardLine(card, L["ladders.tip.mark"]:format(ns.MarkText(id))) end
	else
		local lo, hi = Tree.Window(l, r, move)
		if lo then Tree.CardLine(card, L["ladders.card.range"]:format(value(move, lo), value(move, hi))) end
	end
	if l == ns.CARDIO then return end -- a counted form's best clean count is its own line (Tree.CardClean)
	local best, last = ns.Record(ns.state, move)
	if best then Tree.CardLine(card, L["tip.move.record"]:format(value(move, best), value(move, last or best)), nil, 2) end
end

-- A counted cardio rung's measure (2026-10-16; slice 42's words): its clean days and its window ("3 clean days of 30 s
-- all out"), then the best clean count of the form it shows (ns.CountedForm), "Best 31 clean reps"; `bestOnly`: the
-- best alone (the player's step, whose climb line counts the clean days).
function Tree.CardClean(card, rung, bestOnly)
	if not bestOnly then Tree.CardLine(card, L["tree.card.clean"]:format(ns.CleanDays(rung), C.count_s)) end
	local _, best = ns.CountedForm(ns.state, rung, Tree.facts.f)
	if best and best > 0 then Tree.CardLine(card, ns.Plural("tree.card.best", best):format(best), nil, 2, saga.note) end
end

-- The player's step's measure: a strength or hold step's record on its range ("Record 23 · Range 20-30") with its
-- stars under it and, on a ladder of the Stamina Ledger, what the player can do now ("Now 164-225 s", slice 52;
-- power's in reps), the record's span in the head's tooltip ("Record: 8 → 23 since 1 Nov"), in the move's own unit
-- (Tree.Window; else its record alone); a pose's goal pose; a counted cardio rung's best clean count, a timed one's
-- Pace (slice 52, Tree.CardPace; its full moves are the climb's row); a strength or hold step with no record yet its
-- range and that its first set sets its mark (docs/ui.md, its empty ring's key line), never a record of 0. The clean
-- days show once: where the card's climb line shows (Tree.ClimbShows) it names them, so the measure leaves them out,
-- and the top, a shut climb and one a pain flag holds keep them here; a pose's goal
-- pose (its main pose's) shows here always, since the climb line names none (slice 47: any goal pose of the step counts
-- its day; the climb line named it until then).
function Tree.CardNow(card, l, r, src)
	local id = src.standing
	local lo, hi
	local named = Tree.ClimbShows(l, src)
	if id then lo, hi = Tree.Window(l, r, id) end
	if lo then
		local first, firstTs, best = ns.RecordSpan(ns.state, id)
		if first then
			local line = L["tree.key.record"]:format(value(id, best), value(id, lo), value(id, hi))
			Tree.CardLine(card, line)
			Tree.CardLine(card, Style.Stars(ns.Stars(l, r, id, best), 3), nil, 4)
			card.measureTip = Kit.Tip(line, L["ladders.tip.record"]:format(value(id, first), value(id, best),
				Sheet.ShortDate(firstTs)))
			-- what the player can hold now on a ladder of the Stamina Ledger (slice 52, ns.HoldNow): seconds, power's reps
			local nowLo, _, nowHi = ns.HoldNow(ns.state, l, id, ns.now())
			if nowLo then
				local hold = ns.HOLD_LEDGER[l] == ns.Enum.kind.hold
				Tree.CardLine(card, (hold and L["tree.card.nowHold"] or L["tree.card.nowReps"]):format(nowLo, nowHi), nil, 4)
			end
		else
			Tree.CardLine(card, L["ladders.card.range"]:format(value(id, lo), value(id, hi)))
			Tree.CardLine(card, L["tree.key.mark"], nil, 2, saga.note)
		end
	elseif id and not POSE[l] then
		local best, last = ns.Record(ns.state, id)
		Tree.CardLine(card, best and L["tip.move.record"]:format(value(id, best), value(id, last or best))
			or L["tree.key.mark"])
	elseif POSE[l] and id and ns.MainPose(l, r, id) then
		Tree.CardLine(card, L["ladders.tip.mark"]:format(ns.MarkText(id)))
	elseif l == ns.CARDIO and Tree.Counted(r) then
		Tree.CardClean(card, r, named)
	elseif l == ns.CARDIO then
		Tree.CardPace(card, r)
	end
end

-- The card's title bar for step (l, r): its ladder, step and where the player stands ("Push · step 5 · your step"), a
-- ladder that waits whole its note in the standing's place ("Push · step 1 · Not open yet").
function Tree.CardTitle(l, r)
	local facts = Tree.facts
	local src, entry = facts.ladders[l], facts.entries[l]
	local standing
	if entry.waits then
		standing = L["tree.waits." .. entry.waits]
	else
		local state = Tree.State(src, entry, r)
		if state == "now" and not src.placed then state = "unplaced" end
		standing = L["tree.standing." .. state]
	end
	return L["tree.step"]:format(L["ladder." .. l], ns.Step(l, r) or r, standing)
end

-- The title bar, the move in its quality's colour (white for a move no ladder places) with its quality's badge, and its
-- figure playing its loop, played again only when what it draws changes (another step, another move, Back to the step;
-- "Animate the move figure", the loop or its still, which Figure.lua reads as it plays; the figure's colour,
-- Figure.color; the gear its pack draws for the player, the profile's implement) or when it last fell back to the
-- key-pose icon (no pack yet in a fight, Figure.Data), so a redraw keeps the loop where it is and a pause the player
-- clicked. Under a pump's dim the figure rests (hidden: its OnUpdate stops, and shown again it moves on, View:Shown).
function Tree.CardHead(card, title, move, q)
	card:SetTitle(title)
	card.name:SetText(moveName(move))
	card.name:SetTextColor((q and Sheet.QualityColor(q) or HIGHLIGHT_FONT_COLOR):GetRGB())
	card.badge:SetShown(q ~= nil)
	if q then card.badge:Set(q) end
	local f = card.figure
	f.frame:SetShown(not Lc.Visible())
	local playing = table.concat({ move, Tree.Moving() and "loop" or "still", ns.Figure.ColorCode(),
		ns.Implement(move, ns.profile.gear) or "" }, ":")
	local shown = Tree.shown
	if shown.playing ~= playing then shown.playing = f:Play(move, "loop") and playing or nil end
end

-- A move's stop line where it has one (its spoken words, unchanged): on the face of a step ahead, the next step and a
-- family not open yet, beside How under More past the card's cap (card.fold 2).
function Tree.CardStop(card, move)
	local stop = ns.Panel.MoveStop(move)
	if stop then Tree.CardLine(card, stop, "bodyWhite", 8) end
end

-- The card of step (l, r): the title bar and the head (a ladder that waits whole: its step's limit; a family not open
-- yet, what opens it on the face, its stretch checks as check rows, and its How and stop line as any step ahead), the
-- face (Tree.CardFace) and Start, More.
function Tree.CardStep(card, l, r)
	local facts = Tree.facts
	local src, entry = facts.ladders[l], facts.entries[l]
	local start, why, subject = ns.StartFor(ns.state, ns.profile, { ladder = l, rung = r }, facts.now)
	local move = Tree.StepMove(l, r)
	local state = entry.waits and "waits" or select(3, Tree.KeyLines(l, r, start, why, subject))
	local closed = entry.waits == "closed"
	local ahead = closed or state == "next" or state == "ahead"
	-- a far step's road says nothing where the setup or the player's list stops the way to it: its refusal does
	local stopped = not start and (why == "setup" or why == "off")
	Tree.CardBegin(card)
	Tree.shown.cardChoice = start
	Tree.CardHead(card, Tree.CardTitle(l, r), move, ns.Quality(l, r))
	Tree.CardZone(card, "head")
	if entry.waits then
		Tree.CardLimit(card, l, r, move)
	else
		Tree.CardMeasureHead(card, l, r, state, src, entry, move, stopped)
	end
	Tree.CardZone(card, "face")
	if closed then
		local _, _, first = ns.OpensAt(l, ns.state)
		if not (first and ns.IsCheck(first)) then Tree.CardLine(card, Tree.Opens(l, true)) end
		Tree.shown.opensSaid = true
		local checks = {}
		for _, o in ipairs(ns.Ladders[l].opens or {}) do
			local e = Tree.Alternative(o)
			if ns.IsCheck(e) then checks[#checks + 1] = { need = e, group = o, opens = { family = l } } end
		end
		Tree.CardChecks(card, checks)
	end
	if ahead and card.fold < 2 then Tree.CardStop(card, move) end
	if not entry.waits then Tree.CardFace(card, l, r, state, src, entry) end
	Tree.CardHair(card)
	-- on a step above the player's, what reaching it takes heads the Start's block (docs/ui.md, "The step's card")
	if not entry.waits and state ~= "now" then Tree.CardRoad(card, l, r, state, src, stopped) end
	if start then
		local away = entry.waits ~= nil or start.via ~= nil or r > src.rung or start.move ~= move
		local ownStep = not entry.waits and r == src.rung
		-- the Long Haul, which measures nothing, waits while a steady move finds the step again (placingBy)
		local haul = ownStep and l == ns.CARDIO and entry.placingBy ~= "steady"
		Tree.CardStart(card, start, l, r, away, ownStep, haul and Tree.HaulChoice() or nil)
	else
		Tree.shown.choice = nil
		Tree.CardRefusal(card, why, subject, l)
	end
	Tree.CardZone(card, "more")
	local own = state == "now"
	Tree.CardMore(card, l, r, move, { how = ahead, stop = ahead and card.fold >= 2, skill = own and src or nil })
end

-- Another move shown in the card (a tile or a row of a step, with "Back to the step"), or a move no ladder places
-- (`shown.l` none: Tree.ShowLoose), with its own Start: its ladder (or the one it scales by, ns.ScaleLadder, which
-- places its step: a calf raise, a carry, the good morning), step and quality in the title bar (the move's name where
-- no ladder places it), its record in the head; a move of a family not open yet says what opens it, once (Tree.ViaLine
-- leaves it then), and a move above the player's reach shows its stop line and How as a step ahead does. A move that
-- waits for what opens it (ns.MoveOpen, slice 42) says so: its stage's "Opens after {move} at {goal}" while the path
-- holds it (ns.PathWaits, beside a check that holds it too: its Start runs the check first, as ns.MoveOpen orders
-- them) and its checks as rows (Tree.CardWaits: a stretch check's, a step of another ladder's); then the hint line of
-- its pose.
function Tree.CardMove(card, id)
	local m = ns.Library[id]
	local facts, shown = Tree.facts, Tree.shown
	local start, why, subject = ns.StartFor(ns.state, ns.profile, { move = id }, facts.now)
	local q = ns.MoveQuality(id)
	local closed = m.ladder and facts.entries[m.ladder] and facts.entries[m.ladder].waits == "closed"
	local ahead = closed or ns.AboveReach(ns.state, id)
	Tree.CardBegin(card)
	Tree.shown.cardChoice = start
	Tree.CardHead(card, q and L["tip.move.step"]:format(L["ladder." .. ns.ScaleLadder(m)], ns.MoveStep(id),
		Kit.QualityWords(q)) or moveName(id), id, q)
	Tree.CardZone(card, "head")
	if ahead then Tree.CardLine(card, L["tree.card.ahead"], nil, 0, saga.note) end
	local best, last = ns.Record(ns.state, id)
	if best then Tree.CardLine(card, L["tip.move.record"]:format(value(id, best), value(id, last or best)), nil, 6) end
	Tree.CardZone(card, "face")
	if closed then
		Tree.CardLine(card, Tree.Opens(m.ladder, true))
		Tree.shown.opensSaid = true
	end
	if ahead and card.fold < 2 then Tree.CardStop(card, id) end
	local path = ns.PathWaits(ns.state, id, facts.f)
	if path then Tree.CardLine(card, Tree.OpensAfter(path.after, path.top), "bodyWhite", 8, saga.note) end
	Tree.CardWaits(card, m.checks or {}, { move = id })
	Tree.CardHint(card, id)
	Tree.CardHair(card)
	if start then
		Tree.CardStart(card, start, m.ladder or shown.l, m.rung or shown.r, start.move ~= id or start.via ~= nil)
	else
		shown.choice = nil
		Tree.CardRefusal(card, why, subject, m.ladder)
	end
	if shown.l then
		Tree.CardLink(card, L["tree.card.backToStep"], function()
			shown.move, shown.refused = nil, nil
			Tree.RenderCard()
		end, 12)
	end
	Tree.CardZone(card, "more")
	Tree.CardMore(card, nil, nil, id, { how = ahead, stop = ahead and card.fold >= 2 })
end

-- The card laid out from its zones: the head's column under the name and badge, the hairline under the head or the
-- figure (whichever ends lower), the face under it, More's bar under the face (none where More holds nothing) and,
-- while open, its content in its scroll list down to the screen less the card's room. Answers the card's height with
-- More closed, which the fold weighs against the cap, and keeps the content's bottom for Kit.Fit (card.bottom).
function Tree.LayCard(card)
	local z = card.zones
	local headTop = NAME_Y + (ns.Readable(card.name:GetStringHeight()) or 18) + (card.badge:IsShown() and 6 + 18 or 0) + 8
	z.head:ClearAllPoints()
	z.head:SetPoint("TOPLEFT", card.body, "TOPLEFT", HEAD_X, -headTop)
	z.head:SetHeight(math.max(1, z.head.y))
	local hair = math.max(FIG_TOP + FIG, headTop + z.head.y) + 12
	card.hair:ClearAllPoints()
	card.hair:SetPoint("TOPLEFT", card.body, "TOPLEFT", CARD.IN, -hair)
	local faceTop = hair + 1 + 10
	z.face:ClearAllPoints()
	z.face:SetPoint("TOPLEFT", card.body, "TOPLEFT", CARD.IN, -faceTop)
	z.face:SetHeight(math.max(1, z.face.y))
	local bottom = faceTop + z.face.y
	local more = card.more
	local any = z.more.y > 0
	more:SetShown(any)
	if not any and more:IsOpen() then more:SetOpen(false) end
	if any then
		more:ClearAllPoints()
		more:SetPoint("TOPLEFT", card.body, "TOPLEFT", CARD.IN, -(bottom + 14))
		bottom = bottom + 14 + Kit.CONTROL_H
	end
	local closed = bottom + 20
	if any and more:IsOpen() then
		local top = bottom + 8
		local room = math.floor((ns.Readable(UIParent:GetHeight()) or 768) - CARD.room) - 20 - top
		local h = math.max(40, math.min(z.more.y, room))
		more.content:ClearAllPoints()
		more.content:SetPoint("TOPLEFT", card.body, "TOPLEFT", CARD.IN, -top)
		more.content:SetSize(CARD.TEXT, h)
		card.moreScroll:SetSize(CARD.TEXT, h)
		card.moreScroll.content:SetHeight(math.max(1, z.more.y))
		z.more:SetHeight(math.max(1, z.more.y))
		bottom = top + h
	end
	card.bottom = bottom
	return closed
end

-- The card drawn: its step or its move, laid out (Tree.LayCard); with More closed past the cap (Kit.TIER.card's 400:
-- a larger Text Size, several need rows, a crowded step) the face folds, the card drawn and laid out again after each,
-- in this order: (1) For variety's tiles fold under its More row; (2) a step ahead's or the next step's stop line joins
-- How under More; (3) each main move, stage, need and check row's second line goes into its row's tooltip; (4)
-- the Start's second line into the Start's tooltip. Never folded: the head, the climb, the stage, need and check rows
-- themselves, the Start, its block reason, a refusal and its links. Then its height (Kit.Fit: at most the screen less
-- its room, More's content scrolling past).
function Tree.RenderCard()
	local card, shown = Tree.card, Tree.shown
	if not (card and (shown.l or shown.move) and Tree.facts) then return end
	for fold = 0, 4 do
		card.fold = fold
		if shown.move then Tree.CardMove(card, shown.move) else Tree.CardStep(card, shown.l, shown.r) end
		if Tree.LayCard(card) <= CARD.cap then break end
	end
	Kit.Fit(card, card.bottom)
	Tree.WatchTimes()
end

-- Beside the sheet, level with its step (a move no ladder places: level with the board's top), on the sheet's right
-- past its side tabs, on its left where the screen ends first; where the player dragged it, it stays this session.
-- Fixed math on the board's own rows and scroll.
function Tree.PlaceCard()
	local card, sheet, shown = Tree.card, Tree.page:GetParent(), Tree.shown
	if shown.moved then return end
	local board = Tree.boards[Tree.tree]
	local y, scrolled = 0, 0
	if board and shown.l then
		y = select(2, Tree.StepPoint(board, shown.l, shown.r)) or 0
		scrolled = ns.Readable(board.scroll:GetVerticalScroll()) or 0
	end
	local top = 60 + (board and board.top or NAMES_TOP) + math.max(0, y - scrolled - SLOT / 2 - CARD.IN)
	local right = ns.Readable(sheet:GetRight())
	local room = ns.Readable(UIParent:GetWidth())
	card:ClearAllPoints()
	if right and room and right + 60 + CARD.W > room then
		card:SetPoint("TOPRIGHT", sheet, "TOPLEFT", -8, -top)
	else
		card:SetPoint("TOPLEFT", sheet, "TOPRIGHT", 60, -top)
	end
end

-- The card shown: drawn, placed and shown with its sound unless `quiet` (the return's, a card that showed already); its
-- More closed on a step or a move it did not show before.
function Tree.OpenCard(quiet, fresh)
	local card = Tree.card
	if fresh then card.more:SetOpen(false) end
	local was = card:IsShown()
	Tree.Render()
	Tree.RenderCard()
	Tree.PlaceCard()
	if not was then
		card:Show()
		if not quiet then Style.Play("open") end
	end
end

-- Step (l, r)'s card, pinned beside the sheet: selected, its links drawn. `quiet`: the return's, with no sound of its
-- own.
function Tree.ShowCard(l, r, quiet)
	local card = Tree.card or Tree.BuildCard()
	local shown = Tree.shown
	local fresh = shown.l ~= l or shown.r ~= r or not card:IsShown()
	Tree.selected = { l, r }
	shown.l, shown.r, shown.move, shown.refused = l, r, nil, nil
	Tree.OpenCard(quiet, fresh)
end

-- Another move of the step in the card (a tile's click).
function Tree.ShowMove(id)
	if not Tree.card then return end
	Tree.shown.move, Tree.shown.refused = id, nil
	Tree.RenderCard()
end

-- A move no ladder places in a card of its own (the sheet's flyout, Sheet.ShowMoveCard), with its own Start and no step
-- selected on the board.
function Tree.ShowLoose(id)
	local card = Tree.card or Tree.BuildCard()
	local shown = Tree.shown
	local fresh = shown.l ~= nil or shown.move ~= id or not card:IsShown()
	Tree.selected = nil
	shown.l, shown.r, shown.move, shown.refused = nil, nil, id, nil
	Tree.OpenCard(false, fresh)
end

-- Whether the card shows step (l, r) (itself or another move of it).
function Tree.CardOn(l, r)
	local card, shown = Tree.card, Tree.shown
	return card ~= nil and card:IsShown() and shown.l == l and shown.r == r
end

-- The card goes (its close, a second click on its step, the board's ground, a tab switch, the sheet's hide), `quiet`
-- with no sound of its own.
function Tree.HideCard(quiet)
	local card = Tree.card
	if not (card and card:IsShown()) then return end
	Tree.quiet = quiet
	card:Hide()
	Tree.quiet = nil
end

-- The selection goes with the card the player closes (its close, a second click on its step, the board's ground,
-- Escape): the step's highlight and its links. A tab switch, a step on another tree and
-- the sheet's hide close the card quietly and set the selection themselves.
function Tree.Deselect()
	Tree.selected = nil
	Tree.Render()
end

-- A click on the board's ground lands the return's motion at its end, as a click on a step does, and closes the card,
-- or with none the selection a search made.
function Tree.Ground()
	Tree.Land()
	if Tree.card and Tree.card:IsShown() then return Tree.HideCard() end
	if Tree.selected then Tree.Deselect() end
end

-- Start -----------------------------------------------------------------------------------------------------------

-- The rings, the steps, the glows and the beads' states as the board shows them as a pump starts: the return fills
-- from there.
function Tree.Snapshot()
	local facts = Tree.facts
	local out = { rings = {}, steps = {}, glows = {}, beads = {} }
	for l, e in pairs(facts.entries) do
		local src = facts.ladders[l]
		out.rings[l] = e.ring
		out.steps[l] = { rung = src.rung, open = src.open }
		if Tree.Beaded(l, src, e) then
			local states = {}
			for i, st in ipairs(ns.Stages(ns.state, l, src.rung, facts.f)) do states[i] = st.state end
			out.beads[l] = states
		end
	end
	for t, tf in ipairs(facts.trees) do out.glows[t] = tf.glow end
	return out
end

-- The card's Start (Tree.Run with its choice): a stretch check's carries the entry of the check it runs
-- (Tree.StartedCheck), whose row the pump's note and Result name first (Panel.lua readChecks, ns.PoseChecks).
function Tree.Start()
	local s = Tree.card and Tree.shown.choice
	if not s then return end
	local c = s.as == "check" and Tree.StartedCheck(s) or nil
	Tree.Run(s, c and { check = c.need } or nil)
end

-- A start from the card (its Start, a check row's "Do the check"): the waiting pump opens (Lc.Demand) where one waits;
-- else start `s` (ns.StartFor's) starts (Lc.StartMove), with where it came from: the step, the ladder a `via` start
-- ran and the board as it stood, for the return (Tree.Return), `move`, the move shown in the card when it is not the
-- step's own, the target Lc.Cleared asks ns.StartFor about again after a ladder started over, and `extra`'s fields (a
-- check's `check`, the entry of the check it runs). Where either cannot now (a passing state: for the waiting pump
-- Lc.Unsafe's for a pump asked for, a popup or a window holding it among them, which Lc.Demand would pass by in
-- silence), the card says why in red and the chat says it too.
function Tree.Run(s, extra)
	local shown = Tree.shown
	local ok, why
	local pump = Lc.StartBlock() == "pump"
	if pump then
		why = Lc.Unsafe(true)
		if not why then
			shown.refused = nil
			return Lc.Demand()
		end
	else
		local from = { ladder = shown.l, rung = shown.r, move = shown.move, via = s.via, start = s.ladder,
			before = Tree.Snapshot() }
		for k, v in pairs(extra or {}) do from[k] = v end
		ok, why = Lc.StartMove(s, from)
	end
	if ok then
		Style.Play("start")
		shown.refused = nil
	else
		shown.refused, shown.refusedPump = why, pump
		ns.Chat("start", Tree.BlockText(why))
		Tree.RenderCard()
	end
end

-- The return ------------------------------------------------------------------------------------------------------

-- What moved since the snapshot: the rings that filled on the same step (filled from their old share), the beads whose
-- stage a set finished on that step (filled with the ring's motion), the steps a set opened (a ladder that climbed, a
-- family that opened), the trees whose glow moved.
function Tree.MotionFrom(before)
	local facts = Tree.facts
	local m = { rings = {}, opened = {}, glows = {}, beads = {} }
	for l, e in pairs(facts.entries) do
		local was, src = before.steps[l], facts.ladders[l]
		local from = before.rings[l] or 0 -- an empty ring (no mark yet) fills from nothing
		if was and was.rung == src.rung and e.ring and from < e.ring then m.rings[l] = from / 1000 end
		if was and (src.rung > was.rung or (src.open and not was.open)) then m.opened[l] = src.rung end
		local states = was and was.rung == src.rung and before.beads[l]
		if states and Tree.Beaded(l, src, e) then
			m.beads[l] = {}
			for i, st in ipairs(ns.Stages(ns.state, l, src.rung, facts.f)) do
				local done = st.state == "done" or st.state == "skipped"
				m.beads[l][i] = done and states[i] ~= "done" and states[i] ~= "skipped" or nil
			end
		end
	end
	for t, tf in ipairs(facts.trees) do
		if tf.glow and tf.glow ~= before.glows[t] then m.glows[t] = true end
	end
	return m
end

-- After a pump started from the card (Lc.Finish's "State", once the dim has gone: Panel.lua's listener runs first):
-- the sheet back on the Talents tab where Escape or the game menu closed it, the tree and the started step selected
-- (for a start that ran another ladder, `via`, that ladder's step, switching the tab), its card pinned on it, the step
-- scrolled into view, then the ring's fill from the share last seen, a step the set opened flashing once in its tone
-- and the glow fading in on its new step, with no sound (the Result sounded); under reduced motion the end at once.
function Tree.Return(from)
	if not (ns.db and ns.state) then return end
	local l, r = from.ladder, from.rung
	if from.via and from.start then l, r = from.start, ns.Rung(ns.state, from.start) end
	Sheet.Reopen("ladders")
	if not Tree.facts then return end
	Tree.motion = Tree.Moving() and Tree.MotionFrom(from.before) or nil
	Tree.SelectStep(l, r)
	Tree.ShowCard(l, r, true)
	Tree.PlayMotion()
	Tree.motion = nil
end

function Tree.PlayMotion()
	local m, board = Tree.motion, Tree.boards[Tree.tree]
	if not (m and board) then return end
	for l, rung in pairs(m.opened) do
		local node = board.nodes[l] and board.nodes[l][rung]
		if node and node:IsShown() then
			local c = Sheet.QualityColor(ns.Quality(l, rung))
			board.flash:ClearAllPoints()
			board.flash:SetAllPoints(node)
			board.flash:SetVertexColor(c.r, c.g, c.b)
			board.flashAnim:Stop()
			for _, a in ipairs({ board.flashAnim:GetAnimations() }) do
				if a:GetOrder() == 1 then a:SetStartDelay(HOLD_S + FILL_S) end
			end
			board.flashAnim:Play()
		end
	end
	if m.glows[board.t] and board.glow:IsShown() then
		board.glowAnim:Stop()
		for _, a in ipairs({ board.glowAnim:GetAnimations() }) do a:SetStartDelay(HOLD_S + FILL_S) end
		board.glow:SetAlpha(0) -- unseen through the ring's hold and fill, then fading in: never shown first
		board.glowAnim:Play()
	end
end

-- A click lands every part of the return's motion at its end.
function Tree.Land()
	for _, board in pairs(Tree.boards) do
		for _, ring in pairs(board.rings) do
			if ring.anim and ring.anim:IsPlaying() then
				ring.anim:Stop()
				ring.fill:SetRadialProgressBarPercent(ring.to or 0)
			end
		end
		board.flashAnim:Stop()
		if board.glowAnim:IsPlaying() then
			board.glowAnim:Stop()
			board.glow:SetAlpha(GLOW_A)
		end
		for _, list in pairs(board.beads) do
			for _, b in ipairs(list) do b.fade:Stop() end
		end
	end
end

-- The page draws again at the next moment its words name, which no event announces: a ladder's day counting again
-- (ns.TreeEntry's dayIn, a tree's `at`: its Next line and glow), the shown card's Start (its dayIn and freshFrom: the
-- day, fresh and earn lines, the Start's kind) and the game day's turn (ns.NextDayStart: the felt-unwell lock's end,
-- Lc.LockEnd, and the day's sets behind the tired line), as the desktop app's trees draw again at their `due`. One
-- timer, armed at each draw while the page shows and cancelled as it hides.
function Tree.WatchTimes()
	local facts, now = Tree.facts, ns.now()
	local at = ns.NextDayStart(now)
	local function sooner(t)
		if t and t > now and t < at then at = t end
	end
	for _, e in pairs(facts.entries) do sooner(e.dayIn) end
	for _, tf in ipairs(facts.trees) do sooner(tf.at) end
	local s = Tree.shown.choice
	if s then
		sooner(s.dayIn)
		sooner(s.freshFrom)
	end
	T.after("due", at - now + 1, function() Tree.Render() end)
end

-- The page drawn again for the card (its Start's state changed: the events and messages below): the board and the card
-- from one read of the facts at this moment. Under a pump's dim the card is covered, so only its figure rests (the
-- pump's own figure is the one that moves), and the card draws as the dim goes.
local function redrawCard()
	if not (Tree.card and Tree.card:IsShown()) then return end
	if Lc.Visible() then return Tree.card.figure.frame:Hide() end
	Tree.Render()
end

-- The card's Start reads Lc.StartBlock again as each of its states changes: a pump comes or goes ("State"), a popup or
-- a menu hides its dim or gives it back ("Hold"), the setup moves on ("Setup"), a fight starts or ends, the player
-- dies, releases or comes back to life, and a loading screen leads into or out of a battleground; the moments no event
-- announces have their timer (Tree.WatchTimes). A pump started from it brings the sheet back as it ends (Tree.Return)
-- once it opened, by the player's own way out on the panel (done, closed, paused, felt unwell, or dropped by a ladder
-- started over under it): never one the lifecycle owed (a loading screen into an instance, a party turning mixed, a
-- turn-in while it waited: Lc.OweIfOwing, Lc.TurnIn; a demand pump's moves are never owed, so nothing of it is left),
-- nor into a fight, nor for a save replaced under it, which ends it unwritten. Where a queued turn-in's pump begins
-- after it (Lc.QueueBegins, the rule Lc.Finish's promote follows after this message) the sheet closes quietly instead:
-- that pump asks at once (AGENTS.md), and the sheet open over it would hold it back as a panel (Lc.Unsafe) until the
-- player closed it; a queue promote owes instead (felt unwell, an instance, a mixed party) asks nothing, and the sheet
-- comes back.
local NO_RETURN = { owed = true, reset = true }

ns.On("State", function(p, how)
	if p and p.state == "done" and p.fromTree and p.opened and not NO_RETURN[how] then
		if Lc.QueueBegins() then return Sheet.HideQuiet() end
		if not (Lc.combat or InCombatLockdown()) then return Tree.Return(p.fromTree) end
	end
	redrawCard()
end)
ns.On("Hold", redrawCard)
ns.On("Setup", redrawCard)
for _, event in ipairs({ "PLAYER_REGEN_DISABLED", "PLAYER_REGEN_ENABLED", "PLAYER_DEAD", "PLAYER_ALIVE",
	"PLAYER_UNGHOST", "PLAYER_ENTERING_WORLD" }) do
	ns.RegisterEvent(event, redrawCard)
end
