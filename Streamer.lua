-- The streamer kit (docs/plan.md slice 23; docs/game-design.md, "Growth": streams and screenshots carry QuestPump to
-- players who do not run it): the day's tally, a movable frame the Streamer layout keeps on the screen for a stream's
-- capture, and the day card (the session card until the Saga, 2026-10-15), the game day on one card for a screenshot
-- or a chat line the player sends: its moves, its best day or quality word, its pumps through the day, its moves by
-- region, its records and unlocks. "The day" is the game day (ns.day), so the numbers survive a /reload mid-stream:
-- they come from the fold (ns.DayTally, ns.DayStrip). The Streamer layout's pump card (no dim, a corner) is
-- Panel.lua's. Nothing here names the helper; the card's strip says where a pump was done, in the game or in the app,
-- as the sheet's Log names an app pump. Nothing here is sent by itself.

local ADDON, ns = ...
local C, L = ns.C, ns.L
local Kit, Lc, Setup, Sheet, Style = ns.Kit, ns.Lifecycle, ns.Setup, ns.Sheet, ns.Style
local Q = ns.Enum.quality

local Streamer = {}
ns.Streamer = Streamer
local T = ns.Timers()
local text = Style.Text
local saga = Style.saga

-- The tally: three numbers a row in its digits with their words under them, then the reps by pattern. It stands at
-- TALLY_HOME until the player drags it (tallyPos, its top-left corner), under the pump card's strata: bottom right,
-- above the bag bar and in line with the objective tracker's right edge (Forever 70205's Edit Mode presets), where the
-- pump card in its corner leaves it in sight.
local TALLY_COLS, CELL_W, CELL_H, PAD, TITLE_H = 3, 112, 64, 12, 20
local TALLY_HOME = { point = "BOTTOMRIGHT", x = -110, y = 100 }

local tally, card

-- The numbers ---------------------------------------------------------------------------------------------------------

-- The tally's numbers as { word, value }: the pumps, the records, the stars, the cardio time, the finishers and the
-- unfinished moves.
local function numbers(t)
	return { { L["streamer.pumps"], tostring(t.pumps) }, { L["streamer.records"], tostring(t.records) },
		{ L["streamer.stars"], tostring(t.stars) }, { L["streamer.cardio"], ns.Panel.Clock(t.cardio) },
		{ L["streamer.finishers"], tostring(t.finishers) }, { L["streamer.unfinished"], tostring(t.unfinished) } }
end

-- "Push 45", the reps of each pattern, the most first.
local function repsItems(t)
	local out = {}
	for i, r in ipairs(t.reps) do out[i] = { L["pattern." .. r.pattern], tostring(r.reps) } end
	return out
end

-- The chat line the card's Share line puts in the chat box: what QuestPump counted, never a typed count (slice 13's
-- rule).
local function shareLine(t)
	local parts = { ns.Plural("streamer.share.pumps", t.pumps):format(t.pumps),
		ns.Plural("streamer.share.moves", t.moves):format(t.moves) }
	if t.records > 0 then parts[#parts + 1] = ns.Plural("streamer.share.records", t.records):format(t.records) end
	if t.stars > 0 then parts[#parts + 1] = ns.Plural("streamer.share.stars", t.stars):format(t.stars) end
	return L["share.day"]:format(table.concat(parts, ", "))
end

-- The tally -----------------------------------------------------------------------------------------------------------

local function placeTally(f)
	f:ClearAllPoints()
	local pos = ns.profile and ns.profile.tallyPos
	if type(pos) == "table" and pos.x and pos.y then
		f:SetPoint("TOPLEFT", UIParent, "TOPLEFT", pos.x, pos.y)
	else
		f:SetPoint(TALLY_HOME.point, UIParent, TALLY_HOME.point, TALLY_HOME.x, TALLY_HOME.y)
	end
end

-- `cells` numbers in rows of TALLY_COLS. Dragged anywhere on it, its top-left corner is kept in the profile; a
-- right-click puts it back at TALLY_HOME.
local function buildTally(cells)
	local f = CreateFrame("Frame", nil, UIParent, "TooltipBackdropTemplate")
	f:SetFrameStrata("MEDIUM")
	f:SetWidth(TALLY_COLS * CELL_W + 2 * PAD)
	f:SetClampedToScreen(true)
	f:SetMovable(true)
	f:EnableMouse(true)
	f:RegisterForDrag("LeftButton")
	f:Hide()
	f.title = text(f, "label", "LEFT")
	f.title:SetPoint("TOPLEFT", PAD, -PAD)
	f.title:SetText(L["streamer.tally.title"])
	f.cells = {}
	for i = 1, cells do
		local col, row = (i - 1) % TALLY_COLS, math.floor((i - 1) / TALLY_COLS)
		local c = { value = text(f, "tally", "CENTER", CELL_W), word = text(f, "tallyLabel", "CENTER", CELL_W) }
		c.value:SetPoint("TOPLEFT", PAD + col * CELL_W, -(PAD + TITLE_H + row * CELL_H))
		c.word:SetPoint("TOP", c.value, "BOTTOM", 0, -2)
		f.cells[i] = c
	end
	f.rows = math.ceil(#f.cells / TALLY_COLS)
	f.reps = text(f, "large", "LEFT", TALLY_COLS * CELL_W)
	f.reps:SetPoint("TOPLEFT", PAD, -(PAD + TITLE_H + f.rows * CELL_H))
	f:SetScript("OnDragStart", function(self) if not InCombatLockdown() then self:StartMoving() end end)
	f:SetScript("OnDragStop", function(self)
		self:StopMovingOrSizing()
		if ns.profile then
			ns.profile.tallyPos = { x = self:GetLeft(), y = self:GetTop() - UIParent:GetHeight() }
			ns.Touch()
		end
		placeTally(self)
	end)
	f:SetScript("OnMouseUp", function(self, mouse)
		if mouse ~= "RightButton" or not ns.profile or ns.profile.tallyPos == nil then return end
		ns.profile.tallyPos = nil
		ns.Touch()
		placeTally(self)
	end)
	f:SetScript("OnEnter", function(self)
		GameTooltip:SetOwner(self, "ANCHOR_RIGHT")
		GameTooltip_SetTitle(GameTooltip, L["streamer.tally.title"])
		GameTooltip:AddLine(L["streamer.tally.tip"], 1, 1, 1, true)
		GameTooltip:Show()
	end)
	f:SetScript("OnLeave", function() GameTooltip:Hide() end)
	tally = f
	Streamer.tally = f -- for a UI replacement's skin (Skins.lua)
	return f
end

-- The tally while the Streamer layout is chosen (and QuestPump is on), in and out of combat: redrawn on the bus
-- messages that change its numbers (and at 04:00: Streamer.Render).
function Streamer.RenderTally()
	if not (ns.db and ns.state and ns.Panel.Streamer() and ns.profile.enabled ~= false) then
		if tally then tally:Hide() end
		return
	end
	local now = ns.now()
	local t = ns.DayTally(ns.state, now)
	local items = numbers(t)
	local f = tally or buildTally(#items)
	for i, item in ipairs(items) do
		f.cells[i].word:SetText(item[1])
		f.cells[i].value:SetText(item[2])
	end
	local reps = {}
	for i, item in ipairs(repsItems(t)) do reps[i] = L["streamer.reps.item"]:format(item[1], item[2]) end
	f.reps:SetText(#reps > 0 and L["streamer.reps"]:format(table.concat(reps, " · ")) or "")
	f.reps:SetShown(#reps > 0)
	f:SetHeight(PAD + TITLE_H + f.rows * CELL_H + (#reps > 0 and f.reps:GetStringHeight() + 4 or 0) + PAD)
	placeTally(f)
	f:Show()
end

-- The day (2026-10-15; the Saga's game part, its D board) -------------------------------------------------------------

-- One body of the day's blocks for two hosts (2026-10-16; play SPEC 18, sheet SPEC 6): the day card's window and the
-- character sheet's Today tab (SheetPages.lua), each stacking them in its own order (Kit.Stack) on its own column, b.w
-- across. The blocks, top to bottom as the card shows them: the character with its level and the date (the card's
-- alone: the tab's head carries them), the day's moves (the hero, HERO_H high), the stats row, the strip of the day's
-- pumps, the moves by region, and the records and unlocks in two columns of cells. The strip spans the 24 hours from
-- day_offset_h (STRIP_H high, a bar BAR_W wide and BAR_GAP from the next, on a crowded day the gap closed and then all
-- narrower, BAR_MAX at the day's busiest pump: fillStrip); at most ITEMS cells, the unlocks first when there are more.
local HERO_H, STATS_H, REGION_H = 80, 42, 28
-- the strip's ground under its heading and the buckets' totals, with room above the bars for the best day's star
local STRIP_TOP, STRIP_H, BAR_W, BAR_GAP, BAR_MAX, BAR_MIN = 46, 40, 4, 1, 36, 4
-- a cell at least ITEM_H high (its name and its line, each a line, and 12: taller where one takes a second line), its
-- words TEXT_X from its left, ITEM_GAP under the row above; the line under a name shrinks to SUB_LEAST of its size
-- before it takes a second line
local ITEM_H, ITEM_GAP, ITEMS, ICON_W, ICON_H = 44, 8, 6, 60, 40
local TEXT_X, VALUE_R, SUB_LEAST = 4 + ICON_W + 10, 12, 0.8
local LEVEL_GAP = 4 -- between the character's name and its level, a space's width
local LABEL_GAP = 10 -- from a section's label (12 high) to what it heads: SYSTEM.md 2.3's 26 from label to content
local HOURS = 24

local function moveName(id)
	return L["move." .. ns.Library[id].key]
end

-- Where the Screenshot button cannot take the picture: the key bound to the game's own screenshot.
local function keyLine()
	local key = GetBindingKey("SCREENSHOT")
	if key then return ns.Print("streamer.shot.key", GetBindingText(key)) end
	ns.Print("streamer.shot.noKey", GetBindingName("SCREENSHOT"))
end

local shooting -- the Screenshot button's picture, until the client says how it went

-- The game's own screenshot, the day on it (the card, or the sheet on its Today tab); where the client refuses it, the
-- key that takes one.
function Streamer.Screenshot()
	GameTooltip:Hide() -- the button's own tooltip stays out of the picture
	shooting = true
	local ok = Screenshot ~= nil and pcall(Screenshot)
	if not ok then
		shooting = nil
		keyLine()
	end
end

-- The day's line in the chat box, for the player to send.
function Streamer.ShareLine()
	ChatFrameUtil.OpenChat(shareLine(ns.DayTally(ns.state, ns.now())))
end

-- The day's two buttons on a kit bar (Kit.Footer: the day card's, the Today tab's), each with its tooltip over it:
-- Screenshot the grey at the left, Share line the one red at the right (decision G2, 2026-10-16), which its host greys
-- on a day with no move yet (nothing to share), its tooltip saying why.
function Streamer.DayButtons(foot)
	local shot = foot:Secondary(L["streamer.card.shot"], Streamer.Screenshot)
	Kit.Hover(shot, Kit.Tip(L["streamer.card.shot"], L["streamer.card.shot.tip"]), "ANCHOR_TOP")
	local share = foot:Primary(L["streamer.card.share"], Streamer.ShareLine)
	Kit.Hover(share, Kit.Tip(L["streamer.card.share"], L["streamer.card.share.tip"], {
		reason = function() return not share:IsEnabled() and L["streamer.card.share.none"] or nil end }), "ANCHOR_TOP")
	return shot, share
end

-- The level after a name or a date: "· QP 24", past the cap "· QP 60 · Paragon 12", as the session card showed it.
function Streamer.LevelWords()
	local level, _, _, paragon = ns.Level(ns.state)
	return paragon > 0 and L["streamer.card.levelParagon"]:format(level, paragon) or L["streamer.card.level"]:format(level)
end

-- The character's row: the name and title, then the level, which always shows (a long name and title end in "..."
-- before it: FillDayBody sizes the name, and the cut words show whole in a tooltip), and the date at the right.
local function buildWho(b)
	local r = CreateFrame("Frame", nil, b.parent)
	r:SetWidth(b.w)
	r.date = Kit.Text(r, "labelWhite", "RIGHT", saga.note)
	r.date:SetPoint("TOPRIGHT", 0, -3)
	r.name = Kit.Text(r, "large", "LEFT")
	r.name:SetPoint("TOPLEFT")
	r.name:SetWordWrap(false)
	Kit.WholeWords(r.name)
	r.level = Kit.Text(r, "large", "LEFT")
	r.level:SetPoint("LEFT", r.name, "RIGHT", LEVEL_GAP, 0)
	r.level:SetWordWrap(false)
	b.who = r
end

-- The hero: the day's moves (dayNumber) and their word, and at the right "Best day ever" with its star over the old
-- best, or the day's quality word.
local function buildHero(b)
	local h = CreateFrame("Frame", nil, b.parent)
	h:SetSize(b.w, HERO_H)
	h.moves = Kit.Number(h, "dayNumber", nil, "dayWord")
	h.moves.number:SetPoint("TOPLEFT")
	h.old = Kit.Text(h, "labelWhite", "RIGHT", saga.note)
	h.old:SetPoint("BOTTOMRIGHT", 0, 14)
	h.badge = Kit.Text(h, "dayBadge", "RIGHT")
	h.star = h:CreateTexture(nil, "ARTWORK")
	h.star:SetTexture(Style.star)
	h.star:SetSize(18, 18)
	h.star:SetPoint("RIGHT", h.badge, "LEFT", -6, 0)
	b.hero = h
end

-- The stats row: four cells (pumps, day streak, XP, QP), each a number and its word side by side, centred, with a
-- rule above, below and between them.
local function buildStats(b)
	local row = CreateFrame("Frame", nil, b.parent)
	row:SetSize(b.w, STATS_H)
	local top, bottom = Kit.Rule(row, saga.rule), Kit.Rule(row, saga.rule)
	top:SetPoint("TOPLEFT")
	top:SetPoint("TOPRIGHT")
	bottom:SetPoint("BOTTOMLEFT")
	bottom:SetPoint("BOTTOMRIGHT")
	row.cells = {}
	local w = b.w / 4
	for i = 1, 4 do
		local cell = Kit.Stat(row, { rule = i > 1 })
		cell:SetPoint("TOPLEFT", (i - 1) * w, 0)
		cell:SetSize(w, STATS_H)
		row.cells[i] = cell
	end
	b.stats = row
end

-- A pump's bar, or the legend's swatch: solid gold for a pump in the game, a frost blue outline round a faint fill for
-- one done outside it (an app pump, a catch-up), so the two differ in shape as well as colour (a greyscale screenshot,
-- a reader who cannot tell blue from gold).
local function paintBar(box, away)
	if away then box:Paint(saga.awayFill, saga.away) else box:Paint(saga.game) end
end

-- The strip: its section label with the legend (in the game, in the app) at its rule's end, the buckets' totals
-- (placed by fillStrip), the ground of the 24 hours with a line every 4, the bars, the axis with its times, and the
-- star over the pump that passed the old best on a best day.
local function buildStrip(b)
	local w = b.w
	local s = CreateFrame("Frame", nil, b.parent)
	s:SetSize(w, STRIP_TOP + STRIP_H + 18)
	s.w = w
	-- the legend, right to left: in the app, in the game
	local legend = CreateFrame("Frame", nil, s)
	legend:SetHeight(12)
	local x = 0
	for _, key in ipairs({ "away", "game" }) do
		local label = Kit.Text(legend, "labelWhite", "RIGHT", saga.note)
		label:SetText(L["streamer.card." .. key])
		label:SetPoint("RIGHT", -x, 0)
		local swatch = Style.Box(legend)
		swatch:SetSize(8, 8)
		paintBar(swatch, key == "away")
		swatch:SetPoint("RIGHT", label, "LEFT", -5, 0)
		x = x + label:GetStringWidth() + 5 + 8 + 12
	end
	legend:SetWidth(math.max(1, x - 12))
	local turn = ("%02d:00"):format(C.day_offset_h)
	s.head = Kit.Section(s, L["streamer.card.strip"]:format(turn, turn), false, legend)
	s.head:SetPoint("TOPLEFT")
	s.buckets = {}
	for _, bucket in ipairs(ns.DAY_BUCKETS) do s.buckets[bucket.key] = Kit.Text(s, "labelWhite", "LEFT", saga.note) end
	s.ground = s:CreateTexture(nil, "BACKGROUND")
	s.ground:SetColorTexture(unpack(saga.track))
	s.ground:SetPoint("TOPLEFT", 0, -STRIP_TOP)
	s.ground:SetSize(w, STRIP_H)
	for i = 1, 5 do
		local g = Kit.Rule(s, saga.grid, true)
		g:SetPoint("TOPLEFT", s.ground, "TOPLEFT", i * w / 6, 0)
		g:SetHeight(STRIP_H)
	end
	local axis = Kit.Rule(s, saga.rule)
	axis:SetPoint("TOPLEFT", s.ground, "BOTTOMLEFT")
	axis:SetPoint("TOPRIGHT", s.ground, "BOTTOMRIGHT")
	for i = 0, 6 do
		local label = Kit.Text(s, "labelWhite", i == 0 and "LEFT" or i == 6 and "RIGHT" or "CENTER", saga.note)
		label:SetText(("%02d:00"):format((C.day_offset_h + i * HOURS / 6) % HOURS))
		local point = i == 0 and "TOPLEFT" or i == 6 and "TOPRIGHT" or "TOP"
		label:SetPoint(point, s.ground, "BOTTOMLEFT", i * w / 6, -4)
	end
	s.bars = {}
	-- the star on a frame over the bars (each bar is a frame of its own), so a taller neighbour never hides it
	s.over = CreateFrame("Frame", nil, s)
	s.over:SetAllPoints()
	s.over:SetFrameLevel(s:GetFrameLevel() + 3)
	s.star = s.over:CreateTexture(nil, "OVERLAY")
	s.star:SetTexture(Style.star)
	s.star:SetSize(12, 12)
	b.strip = s
end

-- The strip of the day (ns.DayStrip, in minute order, then pump order; the tokens' DayStrip, history.json saga rule
-- 9): each pump a bar as tall as its moves against the day's busiest, its start its minute's place on the axis less
-- a full bar's width, all at one pitch for the day: the largest that lets every bar stand at or right of its own start
-- and inside the axis (the least, over each bar, of the room from its start to the axis's end over it and the bars
-- after it), BAR_W + BAR_GAP at most; below that the gap closes first and then the bars narrow alike, a unit at least,
-- so a crowded evening reads as a block at its hour; each bar at its start or a pitch right of the one before,
-- whichever is later, and only what still runs past the axis's end at a unit and no gap shifts back left from it,
-- never past the axis's start; in the game or outside it (paintBar); each bucket with moves its total centred over its
-- bars, moved apart from its neighbour's and kept inside the strip; the star over the pump during which the day passed
-- its old best.
local TOTAL_GAP = 10 -- the least room between two buckets' totals

-- The bars' left edges on an axis `width` across, as above, and their width.
local function placeBars(bars, width)
	local n, starts = #bars, {}
	local pitch = BAR_W + BAR_GAP
	for i, bar in ipairs(bars) do
		starts[i] = bar.minute / (HOURS * 60) * (width - BAR_W)
		pitch = math.min(pitch, (width - starts[i]) / (n - i + 1))
	end
	pitch = math.max(1, pitch)
	local w = math.min(BAR_W, pitch)
	local xs, cursor = {}, -math.huge
	for i = 1, n do
		xs[i] = math.max(starts[i], cursor)
		cursor = xs[i] + pitch
	end
	local limit = width - w
	for i = n, 1, -1 do
		xs[i] = math.max(0, math.min(xs[i], limit))
		limit = xs[i] - pitch
	end
	return xs, w
end

local function fillStrip(s, bars, buckets, rank)
	local most = 1
	for _, bar in ipairs(bars) do most = math.max(most, bar.n) end
	local xs, w = placeBars(bars, s.w)
	local crossed, star = 0, nil
	local byOrder = {}
	for _, bar in ipairs(bars) do byOrder[bar.order] = bar end
	if rank.best then
		for _, bar in ipairs(byOrder) do
			crossed = crossed + bar.n
			if crossed > rank.old.moves then
				star = bar
				break
			end
		end
	end
	for i, bar in ipairs(bars) do
		local t = s.bars[i]
		if not t then
			t = Style.Box(s)
			s.bars[i] = t
		end
		paintBar(t, bar.away)
		t:ClearAllPoints()
		t:SetPoint("BOTTOMLEFT", s.ground, "BOTTOMLEFT", xs[i], 0)
		t:SetSize(w, math.max(BAR_MIN, BAR_MAX * bar.n / most))
		t:Show()
		if bar == star then
			s.star:ClearAllPoints()
			s.star:SetPoint("BOTTOM", t, "TOP", 0, 2)
		end
	end
	for i = #bars + 1, #s.bars do s.bars[i]:Hide() end
	s.star:SetShown(star ~= nil)
	-- the totals, morning to night as the axis runs: each centred over its bars, then left to right clear of the one
	-- before, then from the strip's end back clear of the one after
	local shown = {}
	for _, bucket in ipairs(ns.DAY_BUCKETS) do
		local label, n = s.buckets[bucket.key], buckets[bucket.key]
		label:SetShown(n > 0)
		if n > 0 then
			label:SetText(L["streamer.card.bucket"]:format(L["streamer.card.time." .. bucket.key],
				HIGHLIGHT_FONT_COLOR:WrapTextInColorCode(BreakUpLargeNumbers(n))))
			local lo, hi
			for i, bar in ipairs(bars) do
				if bar.bucket == bucket.key then lo, hi = math.min(lo or xs[i], xs[i]), math.max(hi or xs[i], xs[i]) end
			end
			local width = label:GetStringWidth()
			shown[#shown + 1] = { label = label, width = width, left = (lo + hi + w) / 2 - width / 2 }
		end
	end
	local edge = 0
	for _, t in ipairs(shown) do
		t.left = math.max(t.left, edge)
		edge = t.left + t.width + TOTAL_GAP
	end
	edge = s.w
	for k = #shown, 1, -1 do
		local t = shown[k]
		t.left = math.min(t.left, edge - t.width)
		edge = t.left - TOTAL_GAP
		t.label:ClearAllPoints()
		t.label:SetPoint("BOTTOMLEFT", s, "TOPLEFT", t.left, -(STRIP_TOP - 18))
	end
end

-- The moves by region: its section label over four columns, each its region's word and count over a bar (Kit.Bar)
-- against the day's busiest region.
local function buildRegions(b)
	local r = CreateFrame("Frame", nil, b.parent)
	r:SetSize(b.w, 12 + LABEL_GAP + REGION_H)
	r.head = Kit.Section(r, L["streamer.card.regions"])
	r.head:SetPoint("TOPLEFT")
	r.cols = {}
	local w, gap = Kit.Grid(4, b.w)
	for i = 1, 4 do
		local col = CreateFrame("Frame", nil, r)
		col:SetPoint("TOPLEFT", (i - 1) * (w + gap), -(12 + LABEL_GAP))
		col:SetSize(w, REGION_H)
		col.word = Kit.Text(col, "labelWhite", "LEFT")
		col.word:SetPoint("TOPLEFT")
		col.count = Kit.Text(col, "large", "RIGHT")
		col.count:SetPoint("TOPRIGHT")
		col.bar = Kit.Bar(col, w)
		col.bar:SetPoint("BOTTOMLEFT")
		r.cols[i] = col
	end
	b.regions = r
end

-- The records and unlocks: their section label ("3 new records · 1 unlock") over the cells, two a row.
local function buildWins(b)
	local f = CreateFrame("Frame", nil, b.parent)
	f:SetWidth(b.w)
	f.head = Kit.Section(f, "")
	f.head:SetPoint("TOPLEFT")
	f.cells, f.count = {}, 0
	b.wins = f
end

-- One body of the day on `parent`, `w` across: its blocks (b.who, b.hero, b.stats, b.strip, b.regions, b.wins), each a
-- frame of its own height for the host to stack, filled by Streamer.FillDayBody.
function Streamer.BuildDayBody(parent, w)
	local b = { parent = parent, w = w }
	buildWho(b)
	buildHero(b)
	buildStats(b)
	buildStrip(b)
	buildRegions(b)
	buildWins(b)
	return b
end

-- A record's or an unlock's cell (a card, no button): the move's key-pose icon in its screen, its edge in the step's
-- colour, the move's name and the line under it, and a record's best of the day at the right.
local function cell(f, i, w, move)
	local c = f.cells[i]
	if c then return c end
	c = Kit.Card(f, w, ITEM_H)
	c.light:Hide()
	c.screen = Kit.Screen(c, ICON_W, ICON_H)
	c.screen:SetPoint("LEFT", 4, 0)
	c.icon = Kit.Icon(c.screen, move, ICON_H)
	c.value = Kit.Text(c, "statNumber", "RIGHT")
	c.value:SetPoint("RIGHT", -VALUE_R, 0)
	c.name = Kit.Text(c, "labelWhite", "LEFT")
	c.name:SetWordWrap(true)
	c.name:SetMaxLines(2)
	c.sub = Kit.Text(c, "labelWhite", "LEFT", saga.note)
	c.sub:SetPoint("TOPLEFT", c.name, "BOTTOMLEFT", 0, -2)
	c.sub:SetWordWrap(true)
	c.sub:SetMaxLines(2)
	f.cells[i] = c
	return c
end

-- A set's amount as the card writes it, the day's best and the record before it alike (one unit on one card): a hold's
-- seconds as m:ss ("1:42", "was 1:35", as the D board shows them), a weighted set as the Log writes it ("16 kg × 10"),
-- any other a bare count (a counted cardio set's too: "24", "was 22"), grouped.
local function amount(move, v, load, reps)
	if load then return Sheet.LoadAmount(move, load, reps) end
	return ns.Library[move].dose == "hold" and ns.Panel.Clock(v) or BreakUpLargeNumbers(v)
end

-- A record's line: its weight where the set carried one ("20 kg, was 16 kg × 10"), and its record at the day's start
-- ("Record, was 30"), or none: a first one.
local function recordLine(r)
	local was = r.was and amount(r.move, r.was.v, r.was.load, r.was.reps)
	if not r.load then return was and L["streamer.card.record"]:format(was) or L["streamer.card.record.first"] end
	local kg = L[ns.Loaded(r.move) == "each" and "streamer.card.kgEach" or "streamer.card.kg"]:format(Sheet.Kg(r.load))
	return was and L["streamer.card.recordAt"]:format(kg, was) or L["streamer.card.recordAt.first"]:format(kg)
end

-- Fills one cell: a record ({ record }: its best of the day, amount's, and its weight on the line under its name; its
-- screen's edge in its move's colour) or an unlock ({ unlock }: the move and the cell's edge in the step's colour). The
-- name and the line take the room beside the number: the name wraps to a second line, the line shrinks to fit (to
-- SUB_LEAST) before it does, so a long name or a weighted record is cut only past two lines. Returns the height its
-- words need (placeCell lays them out).
local function fillCell(c, item)
	local width = c:GetWidth() - TEXT_X - VALUE_R
	if item.record then
		local r = item.record
		Kit.SetIcon(c.icon, r.move)
		c.name:SetText(moveName(r.move))
		c.name:SetTextColor(HIGHLIGHT_FONT_COLOR:GetRGB())
		c.sub:SetText(recordLine(r))
		c.value:SetText(amount(r.move, r.v))
		c.value:SetTextColor(NORMAL_FONT_COLOR:GetRGB())
		c.value:Show()
		c:Paint(saga.cell, saga.edge)
		Kit.PaintScreen(c.screen, ns.MoveQuality(r.move))
		width = width - c.value:GetStringWidth() - 8
	else
		local u = item.unlock
		local q = u.quality or Q.starter
		local r, g, bl = Sheet.QualityColor(q):GetRGB()
		Kit.SetIcon(c.icon, u.move)
		c.name:SetText(u.move and moveName(u.move) or L["ladder." .. u.ladder])
		c.name:SetTextColor(r, g, bl)
		c.sub:SetText(L["streamer.card.unlocked"]:format(L["quality." .. q]))
		c.value:Hide()
		c:Paint(saga.cell, { r, g, bl, 0.45 })
		Kit.PaintScreen(c.screen, q)
	end
	c.name:SetWidth(width)
	c.sub:SetTextScale(1)
	c.sub:SetWidth(width)
	local full = c.sub:GetUnboundedStringWidth()
	if full > width then c.sub:SetTextScale(math.max(SUB_LEAST, width / full)) end
	return c.name:GetStringHeight() + 2 + c.sub:GetStringHeight()
end

-- A cell `h` high at `y` in column `col` (`gap` between the two), its words (`textH` high, fillCell's) centred in it.
local function placeCell(c, col, y, h, textH, gap)
	c:SetHeight(h)
	c:ClearAllPoints()
	c:SetPoint("TOPLEFT", col * (c:GetWidth() + gap), -y)
	c.name:ClearAllPoints()
	c.name:SetPoint("TOPLEFT", TEXT_X, -math.floor((h - textH) / 2))
end

-- The records and unlocks: every unlock (ITEMS at most), then the records in the room left, two cells a row, each row
-- as tall as its taller cell's words; the block's height, and wins.count, how many show.
local function fillWins(f, t, w)
	local items = {}
	local room = math.max(0, ITEMS - #t.unlocks)
	for i = 1, math.min(room, #t.newRecords) do items[#items + 1] = { record = t.newRecords[i] } end
	for i = 1, math.min(ITEMS, #t.unlocks) do items[#items + 1] = { unlock = t.unlocks[i] } end
	local heads = {}
	if t.records > 0 then heads[#heads + 1] = ns.Plural("streamer.share.records", t.records):format(t.records) end
	if #t.unlocks > 0 then heads[#heads + 1] = ns.Plural("streamer.card.unlocks", #t.unlocks):format(#t.unlocks) end
	Kit.SetCaps(f.head.words, table.concat(heads, " · "))
	local cw, gap = Kit.Grid(2, w)
	local y = 12 + LABEL_GAP
	for first = 1, #items, 2 do
		local textH, h = {}, ITEM_H
		for i = first, math.min(first + 1, #items) do
			local item = items[i]
			local c = cell(f, i, cw, item.record and item.record.move or item.unlock.move)
			textH[i] = fillCell(c, item)
			h = math.max(h, math.ceil(textH[i]) + 12)
			c:Show()
		end
		for i = first, math.min(first + 1, #items) do placeCell(f.cells[i], i - first, y, h, textH[i], gap) end
		y = y + h + ITEM_GAP
	end
	for i = #items + 1, #f.cells do f.cells[i]:Hide() end
	f.count = #items
	f:SetHeight(#items > 0 and y - ITEM_GAP or 1)
end

-- The body filled from the day's tally (ns.DayTally) at `now`: the character with its QP level and the date; the
-- day's moves, beside them "Best day ever" with the old best (gold, the star), or the day's quality word in its colour,
-- or nothing before the gate (ns.DayRank); the pumps with a move done, the day streak, the XP gained and the QP level
-- from the day's start (past the cap the Paragon rank from it); the strip of the day's pumps; the moves by region; the
-- records ("Record, was 30") and the unlocks ("Unlocked · Epic") as cells. Each block's height is set for its host's
-- stack.
function Streamer.FillDayBody(b, t, now)
	local rank = t.rank
	local level, _, _, paragon = ns.Level(ns.state)
	-- the character: the name's own width (a unit over it, so rounding never cuts a name that fits), at most the room
	-- the level and the date leave
	local who = b.who
	who.name:SetText(Sheet.TitleText())
	who.level:SetText(Streamer.LevelWords())
	who.date:SetText(Sheet.DayDate(ns.day(now)))
	who.name:SetWidth(math.max(1, math.min(who.name:GetUnboundedStringWidth() + 1,
		b.w - who.date:GetStringWidth() - 8 - LEVEL_GAP - who.level:GetStringWidth())))
	who:SetHeight(math.max(who.name:GetStringHeight(), 3 + who.date:GetStringHeight()))
	-- the hero: the best day, or the day's word in its quality's colour: a word and its colour, never the colour alone;
	-- a Common day none (2026-10-29: an ordinary day needs no tag)
	local h = b.hero
	Kit.SetNumber(h.moves, t.moves, ns.Plural("streamer.card.moves", t.moves))
	local best = rank.best
	local quality = not best and rank.quality ~= "common" and rank.quality or nil
	h.badge:ClearAllPoints()
	if best then
		h.badge:SetText(L["streamer.card.best"])
		h.badge:SetTextColor(NORMAL_FONT_COLOR:GetRGB())
		h.badge:SetPoint("BOTTOMRIGHT", h.old, "TOPRIGHT", 0, 4)
		h.old:SetText(L["streamer.card.old"]:format(BreakUpLargeNumbers(rank.old.moves), Sheet.DayDate(rank.old.day)))
	elseif quality then
		h.badge:SetText(L["streamer.card.quality." .. quality])
		h.badge:SetTextColor(Sheet.QualityColor(Q[quality]):GetRGB())
		h.badge:SetPoint("BOTTOMRIGHT", 0, 14)
	end
	h.badge:SetShown(best or quality ~= nil)
	h.star:SetShown(best)
	h.old:SetShown(best)
	-- the stats row
	local cells = b.stats.cells
	Kit.FillStat(cells[1], BreakUpLargeNumbers(t.pumps), ns.Plural("streamer.card.pumps", t.pumps))
	Kit.FillStat(cells[2], BreakUpLargeNumbers(t.streak), L["streamer.card.streak"])
	Kit.FillStat(cells[3], L["streamer.card.xp"]:format(BreakUpLargeNumbers(t.xp)), L["streamer.card.xpWord"], false,
		NORMAL_FONT_COLOR)
	-- past the cap the Paragon rank from the day's start, except on the day the first rank came, which shows the QP
	-- level's cell (never "Paragon 0 > 1")
	local from, to, word = t.levelFrom, level, L["streamer.card.qp"]
	if paragon > 0 and t.paragonFrom > 0 then from, to, word = t.paragonFrom, paragon, L["streamer.card.paragon"] end
	Kit.FillStat(cells[4], from < to and L["streamer.card.levels"]:format(from, Style.Arrow(16), to) or tostring(to), word,
		true)
	-- the strip and the regions
	local bars, buckets = ns.DayStrip(ns.state, now)
	fillStrip(b.strip, bars, buckets, rank)
	local most = 1
	for _, r in ipairs(t.regions) do most = math.max(most, r.moves) end
	for i, col in ipairs(b.regions.cols) do
		local r = t.regions[i]
		col.word:SetText(L["region." .. r.region])
		col.count:SetText(BreakUpLargeNumbers(r.moves))
		col.bar:Set(r.moves, most)
	end
	fillWins(b.wins, t, b.w)
end

-- The day card (2026-10-15; the session card until then): a medium window (Kit.TIER.M, 600 wide since 2026-10-16:
-- decision G2) holding the day's body on its column in the D board's order, and the kit's bar under it with Screenshot
-- and Share line; on a day with no move yet the Today tab's rule: the empty state ("No pumps yet today") in place of
-- the strip, the regions and the wins, and Share line greyed. As tall as its blocks, scaled down past the medium tier's
-- cap (Kit.Fit). Named for UISpecialFrames (Escape closes it, as it does the sheet and the focus window) and hidden by
-- a fight (Kit.Window's opts).
local function buildCard()
	local f = Kit.Window("QuestPumpSessionCard", "M", L["streamer.card.title"], { escape = "special", fight = true,
		y = 40 })
	local t = f.tier
	f.column = CreateFrame("Frame", nil, f.body)
	f.column:SetPoint("TOPLEFT", t.IN, -t.HEAD)
	f.column:SetWidth(t.TEXT)
	f.day = Streamer.BuildDayBody(f.column, t.TEXT)
	f.empty = Kit.Empty(f.column, t.TEXT, { name = L["today.empty"] })
	f.foot = Kit.Footer(f)
	f.shot, f.share = Streamer.DayButtons(f.foot)
	card = f
	return f
end

function Streamer.RenderCard()
	local f = card
	if not (f and f:IsShown() and ns.state) then return end
	local now = ns.now()
	local b, t = f.day, ns.DayTally(ns.state, now)
	Streamer.FillDayBody(b, t, now)
	local empty = t.moves == 0
	if empty then f.empty:Lay() end
	f.share:SetEnabled(not empty)
	local bottom = Kit.Stack(nil, 0, { { b.who, 0 }, { b.hero, 6 }, { b.stats, 6 }, { b.strip, 16, show = not empty },
		{ b.regions, 16, show = not empty }, { b.wins, 16, show = not empty and b.wins.count > 0 },
		{ f.empty, 16, show = empty } })
	f.column:SetHeight(bottom)
	Kit.Fit(f, f.tier.HEAD + bottom, f.foot)
end

-- /qp card, and once a game day on entering a rested area (Streamer.Offer); out of combat, closed by a fight.
function Streamer.ShowCard()
	if not (ns.db and ns.state) then return end
	if Lc.combat or InCombatLockdown() then return ns.Print("chat.combat") end
	if Lc.Visible() then return ns.Print("chat.notnow") end -- under the pump's dim it could not be clicked
	local f = card or buildCard()
	if not f:IsShown() then Style.Play("open") end
	f:Show()
	Streamer.Render() -- the card, and its 04:00 redraw
end

ns.AddCommand("card", Streamer.ShowCard, "cmd.card.help")

-- Entering a rested area after session_card_pumps pumps that game day (as the titles count them, ns.CountedPumps: the
-- opening as it was before the day card) opens the card by itself, at most once a game
-- day (kept in the save's meta, cardDay, and in this session for a save it cannot write), never in a fight, while a
-- pump is on its way or up, under the felt-unwell lock, with QuestPump off or over the setup screen. "Entering" is
-- IsResting() turning true in this session, by PLAYER_UPDATE_RESTING or across a loading screen (a hearthstone, a
-- portal to a capital: Blizzard's PlayerFrame.lua reads the state again at every PLAYER_ENTERING_WORLD); a login or a
-- /reload inside an inn reads the state it starts in.
local resting -- IsResting() as last seen, nil until the load's first PLAYER_ENTERING_WORLD
local offeredDay

function Streamer.Offer()
	local today = ns.day()
	if not (ns.db and ns.state) or offeredDay == today or (ns.meta and ns.meta.cardDay == today) then return end
	if ns.CountedPumps(ns.state) < C.session_card_pumps then return end
	if Lc.combat or InCombatLockdown() or Lc.pump or Lc.Locked() or ns.profile.enabled == false
		or Setup.ShouldShow() or Sheet.ScreenTaken() then
		return
	end
	offeredDay = today
	ns.SetMeta("cardDay", today)
	Streamer.ShowCard()
end

-- The rested state again; an entering offers the card.
local function restingNow()
	local was = resting
	resting = IsResting()
	if resting and was == false then Streamer.Offer() end
end

ns.RegisterEvent("PLAYER_ENTERING_WORLD", function(isInitialLogin, isReloadingUi)
	if isInitialLogin or isReloadingUi or resting == nil then
		resting = IsResting()
	else
		restingNow()
	end
end)

ns.RegisterEvent("PLAYER_UPDATE_RESTING", function() if resting ~= nil then restingNow() end end)

-- Wiring --------------------------------------------------------------------------------------------------------------

-- Both, and once more when the game day turns at 04:00 without a row while either shows (the tally in the Streamer
-- layout, the day card in either layout): the next game day's middle less 12 h, a minute at least, so a clock
-- change's hour (a game day of 23 or 25 h) looks again a little later instead of at once.
function Streamer.Render()
	Streamer.RenderTally()
	Streamer.RenderCard()
	if (tally and tally:IsShown()) or (card and card:IsShown()) then
		local now = ns.now()
		T.after("day", math.max(60, ns.NextDayStart(now) - now + 1), Streamer.Render)
	else
		T.cancel("day")
	end
end

-- every row (a "Sheet" message always comes with its row), a save bound again, a setting, the dev clock
for _, message in ipairs({ "Row", "DB", "Settings", "Clock" }) do ns.On(message, Streamer.Render) end
-- a Text Size, UI scale or screen change: the day card laid out again while it shows
Kit.OnRelayout(Streamer.RenderCard)

-- A pump on the screen closes the card (the pump's own card takes the screen), as a fight does (Kit.Window's `fight`).
ns.On("State", function() if card and card:IsShown() and Lc.Visible() then card:Hide() end end)

ns.RegisterEvent("SCREENSHOT_FAILED", function()
	if not shooting then return end
	shooting = nil
	keyLine()
end)
ns.RegisterEvent("SCREENSHOT_SUCCEEDED", function() shooting = nil end)
