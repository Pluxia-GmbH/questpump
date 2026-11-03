-- The move figure (docs/plan.md slice 2, "The move figure"; slice 10): plays the pack's sheets on the pump card. The
-- pack is the load-on-demand folder QuestPump_FigArmor, whose data-only Manifest.lua fills QuestPumpFigures.armor
-- (the layout, the run table and the tint sheet of each move, keyed by the move's library key); this file loads it on
-- first need, out of combat, and draws a move with four textures stepped from one OnUpdate that runs only while the
-- figure shows (the addon's only OnUpdate). Without a usable pack the views show the move's key-pose icon.
--
-- The pure core (Resolve, CellCoords, NewState, Step) is line for line the twin of tools/figures/steps.py, which
-- writes contract/fixtures/figure-steps.json from the pack.

local ADDON, ns = ...
local C, L = ns.C, ns.L

local floor = math.floor

local Core = {}
ns.FigureCore = Core

-- The move's entry and the entry whose sheets it draws on (a variant names its reference move in `ref`).
function Core.Resolve(fig, key)
	local m = fig and fig.moves[key]
	if not m then return nil end
	return m, m.ref and fig.moves[m.ref] or m
end

-- SetTexCoord(l, r, t, b) of cell c (0-based); a mirrored move swaps left and right, which draws the cell flipped.
function Core.CellCoords(base, c, mirror)
	local cols = base.cols
	local col = c % cols
	local row = floor(c / cols)
	local sw, sh = base.sheet[1], base.sheet[2]
	local x = col * base.cell[1] + base.pad
	local y = row * base.cell[2] + base.pad
	local l, r = x / sw, (x + base.size[1]) / sw
	local t, b = y / sh, (y + base.size[2]) / sh
	if mirror then l, r = r, l end
	return l, r, t, b
end

function Core.NewState()
	return { t = 0, run = 1, runStart = 0 }
end

-- Advances by `elapsed` seconds; returns the cell, the cell blended over it (or nil) and the blend's alpha. The loop
-- wraps with an explicit floor, so Lua and the other players compute the same doubles.
function Core.Step(st, m, hz, elapsed)
	local seq = m.seq
	local nruns = #seq / 2
	local loop = m.ticks / hz
	local t = st.t + elapsed
	if t >= loop then
		t = t - floor(t / loop) * loop
		st.run, st.runStart = 1, 0
	end
	st.t = t
	local tick = t * hz
	while st.run < nruns and tick >= st.runStart + seq[2 * st.run] do
		st.runStart = st.runStart + seq[2 * st.run]
		st.run = st.run + 1
	end
	local cell = seq[2 * st.run - 1]
	local fade = m.fade
	for i = 1, #fade do
		if fade[i] == st.run then
			local nxt = st.run % nruns + 1
			return cell, seq[2 * nxt - 1], (tick - st.runStart) / seq[2 * st.run]
		end
	end
	return cell, nil, 0
end

-- The pack -----------------------------------------------------------------------------------------------------------

local Figure = { pack = "QuestPump_FigArmor" }
ns.Figure = Figure
local CRIMSON = { 0.7674, 0.3132, 0.3132 }
local UNUSABLE = { MISSING = true, INTERFACE_VERSION = true, CORRUPT = true }

-- The pack's state, read at PLAYER_LOGIN without loading it: "ok", "old" (below figure_pack_min: it plays what it
-- has), "disabled" (the player's choice: the icons, no message) or "missing" (not installed or not loadable).
function Figure.Check()
	local name = Figure.pack
	if not C_AddOns.DoesAddOnExist(name) then
		Figure.state = "missing"
		return Figure.state
	end
	local reason = select(5, C_AddOns.GetAddOnInfo(name))
	if reason == "DISABLED" then
		Figure.state = "disabled"
	elseif reason and UNUSABLE[reason] then
		Figure.state = "missing"
	else
		local v = tonumber(C_AddOns.GetAddOnMetadata(name, "X-QuestPump-Pack"))
		Figure.state = (v or 0) < C.figure_pack_min and "old" or "ok"
	end
	return Figure.state
end

-- The pack's data, loaded on first need and only out of combat (LoadAddOn never runs inside ADDON_LOADED); nil when
-- it cannot load, and the views keep the icons.
function Figure.Data()
	if QuestPumpFigures and QuestPumpFigures.armor then return QuestPumpFigures.armor end
	if Figure.tried or InCombatLockdown() or (Figure.state ~= "ok" and Figure.state ~= "old") then return nil end
	Figure.tried = true
	local loaded = C_AddOns.LoadAddOn(Figure.pack)
	local data = loaded and QuestPumpFigures and QuestPumpFigures.armor or nil
	if not data then Figure.state = "missing" end -- it would not load, or loaded without its data: the missing line
	return data
end

-- One loop of a pack entry in seconds, where Core.Step wraps: the figure does the move once at real tempo.
local function loopSeconds(fig, m)
	return m.ticks / fig.hz
end

-- One loop of a move's figure in seconds (a timing variant's own run table, never its reference's), the beat of the
-- voice's rep count (Speech.lua), with "Animate the move figure" off too, and the second in the loop at which each of
-- its reps ends, the last at the loop's end (one rep, or where the pack's `counts` name earlier ends, a dead bug's two
-- sides); nil without a usable pack (in combat before its first load too) or for a move an older pack lacks. Read by
-- the move's id, never from a view's play, which the icon fallback leaves on the move before.
function Figure.Loop(moveId)
	local move = ns.Library[moveId]
	local fig = move and Figure.Data()
	local m = Core.Resolve(fig, move and move.key)
	if not m then return nil end
	local beat, ends = loopSeconds(fig, m), {}
	for i, tick in ipairs(m.counts or {}) do ends[i] = tick / fig.hz end
	ends[#ends + 1] = beat
	return beat, ends
end

-- The one line a missing or older pack earns, once per session, as the session's one warning under the General
-- "Chat messages" setting (ns.Chat); a disabled pack says nothing.
function Figure.Notice()
	if Figure.noticed then return end
	if Figure.state == "missing" then
		ns.Chat("warning", L["figure.missing"])
	elseif Figure.state == "old" then
		ns.Chat("warning", L["figure.old"])
	else
		return
	end
	Figure.noticed = true
end

-- The tint's colour: the character's class colour (read once, out of combat, from the stored class) or crimson.
function Figure.ReadColor()
	local c = CRIMSON
	local ch = ns.db and ns.db.chars and ns.charIndex and ns.db.chars[ns.charIndex]
	if not (ns.profile and ns.profile.figureColor == "crimson") and ch and ch.class then
		local cc = C_ClassColor.GetClassColor(ch.class)
		local r, g, b = cc and ns.Readable(cc.r), cc and ns.Readable(cc.g), cc and ns.Readable(cc.b)
		if r and g and b then c = { r, g, b } end
	end
	Figure.color = c
	return c
end

-- The tint as one number, 0xRRGGBB, each channel rounded to 0-255: the sheet summary's `fig` (Bridge.lua
-- ns.SheetPayload, slice 27), which the phones tint their figure in as the game does.
function Figure.ColorCode()
	local c = Figure.color or Figure.ReadColor()
	local function byte(v) return math.floor(v * 255 + 0.5) end
	return byte(c[1]) * 65536 + byte(c[2]) * 256 + byte(c[3])
end

ns.RegisterEvent("PLAYER_LOGIN", Figure.Check)
-- every path that binds or swaps the profile (the login's bind, a profile switch, a reset, an import, the dropdown)
ns.On("DB", Figure.ReadColor)
ns.On("Settings", Figure.ReadColor)

-- The pack's key of a move's figure: its gear variant "<key>.<gear>" (tools/figures, 2026-10-09) where the implement
-- that carries its load for the player (ns.Implement with the profile's gear: a dumbbell row's dumbbell) has one in the
-- pack, else the move's own.
local function figureKey(fig, moveId, key)
	local gear = ns.profile and ns.Implement(moveId, ns.profile.gear)
	local geared = gear and key .. "." .. gear
	return geared and fig.moves[geared] and geared or key
end

-- The view ----------------------------------------------------------------------------------------------------------

local View = {}
View.__index = View

local function tex(f, sublevel, add)
	local t = f:CreateTexture(nil, "ARTWORK", nil, sublevel)
	t:SetPoint("CENTER")
	if add then t:SetBlendMode("ADD") end
	t:Hide()
	return t
end

-- The fault pair (slice 22; docs/safety.md hazards 8 and 16): where the pack names a fault cell for a move, the
-- New-move view shows two stills under the caption, the key pose and the fault pose, each in a screen of its own
-- (2026-10-16, the kit's screens), tagged at its top left with its check or cross and its word ("Right" in green,
-- "Wrong" in red) and captioned in the notes' grey under it, never by colour alone. A fault never moves and never
-- shows in "loop" mode (the Ready, countdown and Running views), and an older pack or the icon fallback shows no pair.
local PAIR_GAP, PAIR_H, PAIR_MARK, PAIR_TOP, PAIR_SCALE = 10, 72, 18, 8, 0.5

-- One still of the pair on `parent`, `w` units wide: its screen, the cell with its tint inside, the mark and the word
-- at the screen's top left, the caption under the screen.
local function pairStill(parent, right, w)
	local Kit = ns.Kit
	local s = { screen = Kit.Screen(parent, w, PAIR_H) }
	s.screen:SetPoint("TOPLEFT", right and 0 or w + PAIR_GAP, 0)
	s.base, s.tint = s.screen:CreateTexture(nil, "ARTWORK", nil, 0), s.screen:CreateTexture(nil, "ARTWORK", nil, 1)
	s.tint:SetBlendMode("ADD")
	s.tint:SetAllPoints(s.base)
	s.base:SetPoint("CENTER")
	s.mark = s.screen:CreateTexture(nil, "OVERLAY")
	s.mark:SetSize(PAIR_MARK, PAIR_MARK)
	s.mark:SetPoint("TOPLEFT", 4, -4)
	ns.Style.SetMark(s.mark, right)
	s.word = Kit.Text(s.screen, "label", "LEFT", { (right and GREEN_FONT_COLOR or RED_FONT_COLOR):GetRGB() })
	s.word:SetPoint("LEFT", s.mark, "RIGHT", 3, 0)
	s.word:SetText(L[right and "figure.pair.right" or "figure.pair.wrong"])
	s.cap = Kit.Text(parent, "bodyWhite", "LEFT", ns.Style.saga.note, w)
	s.cap:SetJustifyV("TOP")
	s.cap:SetPoint("TOPLEFT", s.screen, "BOTTOMLEFT", 0, -4)
	return s
end

-- A figure box of `size` UI units on `parent`: base A (sublevel 0), base B for a cross-fade (1), tint A (2) and tint
-- B (3) with ADD, the icon for the fallback and the still, a caption under the box in the notes' grey, the fault pair
-- under the caption and two hidden 1 x 1 textures that preload the next move's sheets. `opts`: `width`, the caption's
-- and the pair's width (the box's + 60 without), and `under`, the region the caption hangs 6 under (the pump card's
-- screen round the box; the box itself, 4 under, without). A click pauses and resumes the motion (WCAG 2.2.2); hidden,
-- it stops, and shown again it moves on where it stopped (a loop on a clock: at the clock's phase).
function Figure.Create(parent, size, opts)
	opts = opts or {}
	local wide = opts.width or size + 60
	local f = CreateFrame("Frame", nil, parent)
	f:SetSize(size, size)
	local v = setmetatable({ frame = f, size = size }, View)
	v.baseA, v.baseB, v.tintA, v.tintB = tex(f, 0), tex(f, 1), tex(f, 2, true), tex(f, 3, true)
	v.icon = tex(f, 4)
	v.caption = ns.Kit.Text(f, "bodyWhite", "CENTER", ns.Style.saga.note, wide)
	if opts.under then
		v.caption:SetPoint("TOP", opts.under, "BOTTOM", 0, -6)
	else
		v.caption:SetPoint("TOP", f, "BOTTOM", 0, -4)
	end
	v.caption:SetJustifyV("TOP")
	v.pair = CreateFrame("Frame", nil, f)
	v.pair:SetPoint("TOP", v.caption, "BOTTOM", 0, -PAIR_TOP)
	v.pair:SetWidth(wide)
	v.pair:Hide()
	local still = (wide - PAIR_GAP) / 2
	v.pairW = still
	v.right, v.wrong = pairStill(v.pair, true, still), pairStill(v.pair, false, still)
	v.preBase, v.preTint = f:CreateTexture(nil, "BACKGROUND"), f:CreateTexture(nil, "BACKGROUND")
	for _, t in ipairs({ v.preBase, v.preTint }) do
		t:SetSize(1, 1)
		t:SetAlpha(0)
		t:SetPoint("TOPLEFT")
	end
	f:EnableMouse(true)
	f:SetScript("OnMouseUp", function() v:TogglePause() end)
	f:SetScript("OnHide", function(self) self:SetScript("OnUpdate", nil) end)
	f:SetScript("OnShow", function() v:Shown() end)
	return v
end

local function place(t, file, base, k)
	t:SetTexture(file)
	t:SetSize(base.size[1] * k, base.size[2] * k)
end

-- The figure's strength: unseen while its sheets load (View:Play), half while dimmed (View:Dim), else full; times its
-- fade (View:Fade).
local function strength(v)
	v.frame:SetAlpha(v.loading and 0 or (v.dimmed and 0.5 or 1) * (v.faded or 1))
end

-- Whether the move's two sheets have loaded: they load asynchronously (the test of Blizzard's
-- TextureLoadingGroupMixin, Region:IsObjectLoaded), and the tint drawn alone over the empty well, before its armour,
-- flashed the cloth's colour as a figure started (David, 2026-10-15: "a weird red flashing when the animation
-- starts", the Talents tab's card, which plays a move with nothing preloaded).
local function loaded(v)
	return v.baseA:IsObjectLoaded() and v.tintA:IsObjectLoaded()
end

function View:HideAll()
	for _, t in ipairs({ self.baseA, self.baseB, self.tintA, self.tintB, self.icon }) do t:Hide() end
	self.pair:Hide()
	self.pairH = nil
	self.frame:SetScript("OnUpdate", nil)
	self.shown, self.shownBlend, self.moving, self.loading = nil, nil, nil, nil
	strength(self)
end

-- A picture of a choice left off (the setup window's tiles, 2026-10-15): the figure greyed and at half strength, its
-- words staying at full strength beside it (their contrast holds); `on` false gives it its colour back.
function View:Dim(on)
	for _, t in ipairs({ self.baseA, self.baseB, self.tintA, self.tintB, self.icon }) do t:SetDesaturated(on) end
	self.dimmed = on
	strength(self)
end

-- The figure faded to `alpha` behind what plays over it, its colour kept (a hold's seconds pour, slice 65: 0.3); nil
-- gives it its strength back.
function View:Fade(alpha)
	self.faded = alpha
	strength(self)
end

-- The fault pair of a move whose pack entry names a fault cell: the key pose and the fault pose as stills at
-- PAIR_SCALE of the figure's size, smaller where the cell would not fit its screen (a standing pose), in the view's
-- tint; the pair's height joins the caption's (View:CaptionHeight).
function View:ShowPair(p, color)
	local m, base = p.m, p.base
	local k = math.min(self.size / p.fig.box * PAIR_SCALE, (self.pairW - 4) / base.size[1], (PAIR_H - 4) / base.size[2])
	local mirror = (m.mirror == true) ~= p.flip
	local path = p.fig.path
	local function still(s, cell, cap)
		local l, r, t, b = Core.CellCoords(base, cell, mirror)
		s.base:SetTexCoord(l, r, t, b)
		s.tint:SetTexCoord(l, r, t, b)
		s.base:SetTexture(path .. base.file)
		s.base:SetSize(base.size[1] * k, base.size[2] * k)
		s.tint:SetTexture(path .. base.tint)
		s.tint:SetVertexColor(color[1], color[2], color[3])
		s.cap:SetText(cap or "")
	end
	still(self.right, m.key or 0, m.key_cap and L["figure.phase." .. m.key_cap])
	still(self.wrong, m.fault, L["figure.fault." .. m.fault_cap])
	local caps = math.max(self.right.cap:GetText() ~= "" and self.right.cap:GetStringHeight() or 0,
		self.wrong.cap:GetStringHeight())
	self.pair:SetHeight(PAIR_H + 4 + caps)
	self.pair:Show()
	self.pairH = PAIR_TOP + self.pair:GetHeight()
end

-- The key-pose icon, centred, at one UI unit a pixel like the sheets (ns.Style.iconPx): the fallback without a pack,
-- and the still where no sheet can show; a move without an icon shows its primary muscle's gear-slot silhouette.
-- `flip` mirrors it (a side move's other side).
function View:ShowIcon(moveId, flip)
	self:HideAll()
	local file, l, r, t, b = ns.Style.MovePicture(moveId)
	if not file then return end
	if flip then l, r = r, l end
	self.icon:SetTexture(file)
	self.icon:SetTexCoord(l, r, t, b)
	self.icon:SetSize(ns.Style.iconPx, ns.Style.iconPx)
	self.icon:Show()
end

-- Draws one step: base A at the cell, tint A over it; during a cross-fade base B and tint B at the next cell at alpha
-- a and tint A at 1 - a. Texture coordinates change only when the cell does.
function View:Apply(cell, blend, alpha)
	local p = self.play
	local mirror = (p.m.mirror == true) ~= p.flip
	if cell ~= self.shown then
		local l, r, t, b = Core.CellCoords(p.base, cell, mirror)
		self.baseA:SetTexCoord(l, r, t, b)
		self.tintA:SetTexCoord(l, r, t, b)
		self.shown = cell
	end
	if blend then
		if blend ~= self.shownBlend then
			local l, r, t, b = Core.CellCoords(p.base, blend, mirror)
			self.baseB:SetTexCoord(l, r, t, b)
			self.tintB:SetTexCoord(l, r, t, b)
			self.shownBlend = blend
			self.baseB:Show()
			self.tintB:Show()
		end
		self.baseB:SetAlpha(alpha)
		self.tintB:SetAlpha(alpha)
		self.tintA:SetAlpha(1 - alpha)
	elseif self.shownBlend then
		self.baseB:Hide()
		self.tintB:Hide()
		self.tintA:SetAlpha(1)
		self.shownBlend = nil
	end
end

-- How far a step moves the loop: the frame's seconds, or with a clock (View:Play) on to the clock's phase, forward
-- through the wrap, so the figure shows clock() mod the loop whatever ran before: a frozen clock holds it, a jump
-- re-phases it, and a clock with no answer holds it too.
local function stepBy(p, elapsed)
	if not p.clock then return elapsed end
	local now = p.clock()
	if not now then return 0 end
	local loop = loopSeconds(p.fig, p.m)
	return (now % loop - p.state.t) % loop
end

local function onUpdate(frame, elapsed)
	local v = frame.view
	local p = v.play
	if not p then return end
	if v.loading then -- unseen and unmoved until both sheets have loaded
		if not loaded(v) then return end
		v.loading = nil
		strength(v)
	end
	if p.stage == "start" then
		p.stageLeft = p.stageLeft - elapsed
		if p.stageLeft <= 0 then p.stage = "reps" end
		return                                  -- the reps start from their first tick on the next frame
	end
	v:Apply(Core.Step(p.state, p.m, p.fig.hz, stepBy(p, elapsed)))
	if p.stage == "reps" then -- the New-move view's reps loop with their phase captions
		local cap = p.m.caps and p.m.caps[p.state.run]
		v.caption:SetText(cap and L["figure.phase." .. cap] or "")
	end
end

-- A still whose sheets load shows once they have, then runs nothing.
local function waitLoad(frame)
	local v = frame.view
	if not loaded(v) then return end
	v.loading = nil
	strength(v)
	frame:SetScript("OnUpdate", nil)
end

-- Holds one cell (the key pose, or the start pose), still: no OnUpdate runs once its sheets have loaded.
function View:Still(cell, cap)
	self.frame:SetScript("OnUpdate", nil)
	self.moving = nil
	self.baseB:Hide()
	self.tintB:Hide()
	self.tintA:SetAlpha(1)
	self.shownBlend = nil
	self.shown = nil
	self:Apply(cell)
	self.caption:SetText(cap and L["figure.phase." .. cap] or "")
	if self.loading then self.frame:SetScript("OnUpdate", waitLoad) end
end

-- The New-move view's caption keeps the height its first words (the setup line) take while the phase words follow,
-- so the tile and the Rewards hung under it never move; on the other views it carries no words and takes no room.
local function fixCaption(v, mode)
	v.caption:SetHeight(0) -- sized to its words for the measure
	v.captionH = mode == "new" and v.caption:GetStringHeight() or nil
	if v.captionH then v.caption:SetHeight(v.captionH) end
end

-- The caption's height in its column, the fault pair under it included (Panel.lua lays the right column out with it).
function View:CaptionHeight()
	return (self.captionH or self.caption:GetStringHeight()) + (self.pairH or 0)
end

-- The region the column goes on under: the fault pair while it shows, else the caption.
function View:Foot()
	return self.pair:IsShown() and self.pair or self.caption
end

-- Plays a move: mode "loop" (the Ready, countdown and Running views: the figure moves through the whole move, David
-- 2026-10-01), "new" (the New-move view: the start pose with the setup line, the player's implement's where it has
-- one, ns.Panel.MoveText, then the reps looping with their phase captions, David 2026-10-02), "key" (the key pose
-- held still: a pose's "How close to the goal pose?", 2026-10-09, captioned "Goal pose" since 2026-10-14, the icon
-- without a pack too) or "still" (the key pose held still with no caption: "Did you keep that pace to the end?" after
-- a steady cardio move, slice 42), mirrored when `flip` (a side move's other side). With "Animate the move figure" off
-- the key pose holds still.
-- Falls back to the key-pose icon (and the one missing line) without a usable pack. `color` lights the cloth in place
-- of the class colour (a Legendary unlock's orange light, Sheet.Legend). `clock`, in "loop" mode, is a function
-- answering seconds on the clock the loop keeps to, negative before the go (the countdown's and the run's,
-- Lc.SinceGo): the loop then shows clock() mod its length from the first frame on, so its start pose meets the go and
-- every loop's end a rep's count, and a play after a hide, a resume after a click's pause and the UI shown again
-- re-phase to it rather than starting over.
function View:Play(moveId, mode, flip, color, clock)
	local move = ns.Library[moveId]
	local fig = move and Figure.Data()
	local m, base = Core.Resolve(fig, fig and figureKey(fig, moveId, move.key))
	self.paused = false
	self.caption:SetText("")
	if not m then
		Figure.Notice()
		self:ShowIcon(moveId, flip)
		if mode == "new" and move then self.caption:SetText(ns.Panel.MoveText(moveId, "setup")) end
		if mode == "key" then self.caption:SetText(L["ui.result.goalPose"]) end
		fixCaption(self, mode)
		return false
	end
	self:HideAll()
	color = color or Figure.color or CRIMSON
	local k = self.size / fig.box
	local path = fig.path
	place(self.baseA, path .. base.file, base, k)
	place(self.baseB, path .. base.file, base, k)
	place(self.tintA, path .. base.tint, base, k)
	place(self.tintB, path .. base.tint, base, k)
	for _, t in ipairs({ self.tintA, self.tintB }) do t:SetVertexColor(color[1], color[2], color[3]) end
	self.baseA:Show()
	self.tintA:Show()
	self.tintA:SetAlpha(1)
	self.loading = not loaded(self) or nil
	strength(self)
	local key = m.key or 0
	self.play = { fig = fig, m = m, base = base, state = Core.NewState(), flip = flip == true,
		clock = mode == "loop" and clock or nil }
	self.frame.view = self
	if mode == "new" and m.fault and m.fault_cap then self:ShowPair(self.play, color) end
	if mode == "key" or mode == "still" or not (ns.profile and ns.profile.figureAnimate ~= false) then
		self:Still(key, mode == "new" and m.key_cap or nil)
		if mode == "key" then self.caption:SetText(L["ui.result.goalPose"]) end
		fixCaption(self, mode)
		return true
	end
	if mode == "new" then
		self.play.stage, self.play.stageLeft = "start", C.figure_new_hold_s
		self:Still(m.seq[1])
		self.caption:SetText(ns.Panel.MoveText(moveId, "setup"))
	else
		self:Apply(Core.Step(self.play.state, m, fig.hz, stepBy(self.play, 0)))
	end
	fixCaption(self, mode)
	self.frame:SetScript("OnUpdate", onUpdate)
	self.moving = true
	return true
end

-- A click pauses a moving figure (its OnUpdate cleared, the state kept) and resumes it where it stopped, or with a
-- clock at the clock's phase, which ran on (the rep count with it).
function View:TogglePause()
	if self.loading then return end -- nothing shows yet to pause
	if self.paused then
		self.paused = false
		onUpdate(self.frame, 0)
		self.frame:SetScript("OnUpdate", onUpdate)
	elseif self.play and self.frame:GetScript("OnUpdate") then
		self.paused = true
		self.frame:SetScript("OnUpdate", nil)
	end
end

-- Shown again with no new Play between (the whole UI hidden and shown, Alt+Z, which renders nothing): OnHide stopped
-- the motion, so a moving figure that was not paused moves on, with a clock from the clock's phase.
function View:Shown()
	if self.moving and not self.paused then
		onUpdate(self.frame, 0)
		self.frame:SetScript("OnUpdate", onUpdate)
	elseif self.loading then
		self.frame:SetScript("OnUpdate", waitLoad)
	end
end

-- Sets the next move's sheets on the hidden 1 x 1 textures (they load asynchronously), so its first frame is never
-- blank; a no-op without the pack.
function View:Preload(moveId)
	local move = ns.Library[moveId]
	local fig = move and Figure.Data()
	local _, base = Core.Resolve(fig, fig and figureKey(fig, moveId, move.key))
	if not base then return end
	self.preBase:SetTexture(fig.path .. base.file)
	self.preTint:SetTexture(fig.path .. base.tint)
end
