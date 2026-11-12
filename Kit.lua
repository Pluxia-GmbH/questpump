-- The kit (2026-10-16; the design system the first-run setup window grew, its D boards and Spoken's welcome window):
-- every QuestPump window's parts in one table, ns.Kit, so each window is built from the same grid, type, colours and
-- states. It owns the measures of each window tier (Kit.TIER), the pieces a window is laid out from (a window and a
-- Settings canvas, a page's head, a picture's screen, cards and choices, section labels, framed lists and their rows,
-- the bottom bar with its step dots, chips, numbers and stats, the check box, the select, the segmented switch, the
-- More section, the scroll list, the empty state, badges and bars) and the layout helpers that place them
-- from their words' heights at render, so a larger Text Size makes a window taller and never makes its words overlap.
-- It draws on Style.lua's names and primitives (fonts, atlases, sounds, Style.Box) and owns no text: callers pass their
-- L["..."] words (the bar's "More" and a box's click hints excepted). Nothing is built at load;
-- every part is made by the window that calls it. No secure template, no Blizzard shared menu (a select opens
-- QuestPump's own, Popups.lua's ns.Menu); a key frame takes Escape alone and lets go of the keyboard in a fight
-- (Kit.EscapeKeys). Every window it builds is in Kit.windows, which Skins.lua follows for a UI replacement's skin.

local ADDON, ns = ...
local L, Style = ns.L, ns.Style
local saga = Style.saga
local T = ns.Timers()

local Kit = { windows = {}, cursor = {} }
ns.Kit = Kit

-- The grid ------------------------------------------------------------------------------------------------------------

-- A window tier's measures in UI units: its width and least height, the content's inset at each side, the title's top,
-- the title font, the gaps from the title to its rule, from the rule to the intro and from the intro to the content,
-- the bottom bar's rule from the bottom (its least; 0 without a rule), the bar's buttons' height and inset from the
-- bottom (and BAR, the buttons' inset from the sides where it is not the content's), the primary's and a secondary's
-- least widths, a list row's name column (where its line starts), and how the window fits the screen (Kit.Fit). TEXT
-- is the content's width.
Kit.TIER = {
	-- the setup window (Spoken's width) and the feedback window
	L = { kind = "window", W = 760, H = 680, IN = 51, HEAD = 44, title = "pageTitle", ruleGap = 12, introGap = 12,
		contentGap = 14, FOOT = 86, BUTTON = 28, BOTTOM = 22, PRIMARY = 160, BACK = 130, rowName = 230,
		fit = "screen" },
	-- the pump card in every view, the day card, the QuestPump card window, the focus, export and QR windows: on the
	-- pump card the move's figure's 216 screen beside a 302 column (split), at least `least` high, about `budget` at
	-- the default Text Size and never over `cap` at UI scale 1 (`edge` of the dim shows above and below), past which a
	-- view folds and then scales, never under minScale; a first appearance may use the cap before How folds (firstCap)
	M = { kind = "window", W = 600, IN = 31, HEAD = 36, title = "move", ruleGap = 10, introGap = 10, contentGap = 12,
		FOOT = 60, BUTTON = 28, BOTTOM = 16, PRIMARY = 140, BACK = 100, rowName = 180, least = 360, budget = 560,
		cap = 640, edge = 64, minScale = 0.9, firstCap = true, split = { 302, 20, 216 }, fit = "cap",
		-- the move figure's screen in the split's right column, round the figure's 192 box (Panel.lua)
		screen = { 216, 204 } },
	-- ns.Dialog's four kinds and the mixed-party ask: their text from the top, no rule over the buttons
	S = { kind = "dialog", W = 380, IN = 20, HEAD = 16, FOOT = 0, BUTTON = 22, BOTTOM = 16, PRIMARY = 128, BACK = 128 },
	-- the step card (Tree.lua) and the day-two check-in (Panel.lua): the dialog's width in the flat panel with its title
	-- bar, its parts' height, at most `cap` with its More closed (the caller's budget); with More open it grows to the
	-- screen less `room`, never more (Kit.Fit), and More's content scrolls past that
	card = { kind = "window", W = 380, IN = 20, HEAD = 36, title = "cardTitle", ruleGap = 10, introGap = 10,
		contentGap = 12, FOOT = 0, BUTTON = 22, BOTTOM = 16, PRIMARY = 128, BACK = 100, rowName = 140, cap = 400,
		room = 64, fit = "clamp" },
	-- a page of the character sheet: the inset under the portrait, fixed, its title at the left with a line on its row
	-- and the rule under them, its one primary at the bottom right and a grey secondary at the bottom left, 16 in; its
	-- content scrolls (beside a bar, scrollW)
	page = { kind = "page", W = 623, H = 420, IN = 20, BAR = 16, HEAD = 14, title = "sectionTitle", ruleGap = 8,
		contentGap = 9, FOOT = 0, BUTTON = 22, BOTTOM = 12, PRIMARY = 140, BACK = 100, rowName = 200, x = 4, y = -60,
		scrollW = 575 },
	-- a Settings page in Blizzard's Options panel (its canvas, 665 x 601 at the panel's default size): Blizzard's
	-- title at (TITLE_X, -HEAD), the rule at -RULE, lists and cards IN in, bare rows BARE in (Blizzard's); a page's
	-- scroll list keeps SCROLLBAR at its right for its thin bar
	canvas = { kind = "canvas", W = 665, H = 601, IN = 16, BARE = 37, TITLE_X = 7, HEAD = 20,
		title = "canvasTitle", RULE = 50, contentGap = 14, FOOT = 0, BUTTON = 22, BOTTOM = 0, PRIMARY = 0,
		BACK = 100, rowName = 242, SCROLLBAR = 14 },
}
for _, t in pairs(Kit.TIER) do t.TEXT = t.W - 2 * t.IN end

Kit.MARGIN = 16 -- the least room between a window and the screen's edge (a window taller scales down)
Kit.BUTTON_H = 26 -- a button's height outside a tier's bar (the pump card's bar until its redo)
Kit.CHIP_ICON = 24 -- a chip's picture, at its own shape inside this square
Kit.CONTROL_H = 26 -- a select's, a segmented switch's and a More bar's height
Kit.WHEEL = 40 -- a scroll list's step for one turn of the mouse wheel

-- A tier by its name, or the tier itself.
function Kit.Tier(tier)
	return type(tier) == "table" and tier or Kit.TIER[tier]
end

-- The width of each of `n` columns across `content` units and the gap between two: 10, but 11 between three (so the
-- large tier's widths are whole numbers).
function Kit.Grid(n, content)
	local gap = n == 3 and 11 or 10
	return (content - (n - 1) * gap) / n, gap
end

-- The pictures' moves -------------------------------------------------------------------------------------------------

-- A move's id by its library key (a choice's picture), nil when the library has none.
function Kit.MoveByKey(key)
	for id, m in pairs(ns.Library) do
		if m.key == key then return id end
	end
end

local ids -- the library's move ids in order, sorted once at the first ask

-- The move a gear piece's picture shows, the first by id with a key-pose icon of: the moves with the piece itself as a
-- need, then those that take it as one of a need's choices (ns.HasNeed, Progress.lua's rule), then those that carry
-- their load on it (Library.lua's holds).
function Kit.GearMove(gear)
	if not ids then
		ids = {}
		for id in pairs(ns.Library) do ids[#ids + 1] = id end
		table.sort(ids)
	end
	local alone = { [gear] = true }
	local function any(list, test)
		for _, entry in ipairs(list or {}) do
			if test(entry) then return true end
		end
		return false
	end
	local passes = {
		function(m) return any(m.needs, function(need) return need == gear end) end,
		function(m) return any(m.needs, function(need) return ns.HasNeed(alone, need) end) end,
		function(m) return any(m.holds, function(hold) return hold == gear end) end,
	}
	for _, uses in ipairs(passes) do
		for _, id in ipairs(ids) do
			if uses(ns.Library[id]) and Style.MoveIcon(id) then return id end
		end
	end
end

-- Words and rules -----------------------------------------------------------------------------------------------------

-- A font string in a Style font, in one of Style.saga's colours (or Blizzard's, as { r, g, b }); white without one.
function Kit.Text(parent, font, justify, color, width)
	local fs = Style.Text(parent, font, justify, width)
	if color then fs:SetTextColor(color[1], color[2], color[3], color[4] or 1) end
	return fs
end

-- A small caps label ("YOUR PUMPS, 04:00 TO 04:00") in the notes' grey; Kit.SetCaps writes its words in capitals
-- (font strings have no letter spacing, so the boards' tracked caps are plain capitals in the client).
function Kit.Caps(parent, justify)
	return Kit.Text(parent, "label", justify or "LEFT", saga.note)
end

function Kit.SetCaps(fs, words)
	fs:SetText(words:upper())
end

-- A line one unit thick in `color`, `vertical` or across; the caller sets its length and places it.
function Kit.Rule(parent, color, vertical)
	local t = parent:CreateTexture(nil, "ARTWORK")
	t:SetColorTexture(color[1], color[2], color[3], color[4] or 1)
	if vertical then t:SetWidth(1) else t:SetHeight(1) end
	return t
end

-- The Options window's faded rule `w` units across (Style's rule atlas), its colour where the client lacks it.
function Kit.HRule(parent, w)
	local t = parent:CreateTexture(nil, "ARTWORK")
	t:SetSize(w, 1)
	if not Style.SetAtlas(t, "rule") then t:SetColorTexture(unpack(saga.rule)) end
	return t
end

-- A quality's word on a font string in its colour (Sheet.QualityColor: Blizzard's item colours with the player's
-- overrides), its numeral (1 Starter to 7 Artifact) before it under the colour-blind mode or with `numeral`: never the
-- colour alone.
function Kit.QualityWords(q, numeral)
	local word = L["quality." .. q]
	if numeral or GetCVarBool("colorblindMode") then return ("%d %s"):format(q + 1, word) end
	return word
end

function Kit.Quality(fs, q, numeral)
	fs:SetText(Kit.QualityWords(q, numeral))
	fs:SetTextColor(ns.Sheet.QualityColor(q):GetRGB())
end

-- Tooltips ------------------------------------------------------------------------------------------------------------

-- A tooltip's filler in one shape, top to bottom: the title (white, or opts.color: a quality's for a move), the body in
-- white (two sentences at most), up to three facts in the notes' grey (opts.facts), one red reason (opts.reason: why
-- it is greyed or refused) and one green hint last (opts.hint, "Click: ..."; a list where a right-click acts too, the
-- click first: { "Click: ...", "Right-click: ..." }). The title, the body, the facts, the reason and the hint may each
-- be a function asked at the hover; a nil one is left out.
function Kit.Tip(head, body, opts)
	opts = opts or {}
	local function value(v)
		if type(v) == "function" then return v() end
		return v
	end
	return function(tooltip)
		GameTooltip_SetTitle(tooltip, value(head), value(opts.color))
		local line = value(body)
		if line then tooltip:AddLine(line, 1, 1, 1, true) end
		for _, fact in ipairs(value(opts.facts) or {}) do
			tooltip:AddLine(fact, saga.note[1], saga.note[2], saga.note[3], true)
		end
		local reason = value(opts.reason)
		if reason then GameTooltip_AddErrorLine(tooltip, reason) end
		local hint = value(opts.hint)
		if type(hint) == "table" then
			for _, words in ipairs(hint) do GameTooltip_AddInstructionLine(tooltip, words) end
		elseif hint then
			GameTooltip_AddInstructionLine(tooltip, hint)
		end
	end
end

-- Shows `tip` (a function filling GameTooltip, Kit.Tip's) while the cursor is over `frame`, at its right unless
-- `anchor` says otherwise ("ANCHOR_TOP" on a bar's buttons).
function Kit.Hover(frame, tip, anchor)
	frame.kitTip = tip -- what a menu entry standing in for the frame shows (the bar's More)
	frame:EnableMouse(true)
	frame:SetScript("OnEnter", function(self)
		GameTooltip:SetOwner(self, anchor or "ANCHOR_RIGHT")
		tip(GameTooltip)
		GameTooltip:Show()
	end)
	frame:SetScript("OnLeave", function() GameTooltip:Hide() end)
end

-- Opens QuestPump's own menu (Popups.lua's ns.Menu) for `owner` from `generator(owner, root)`, under the owner, or
-- over it (`above`: a bar's button at a window's foot); a second click on the owner closes it. The owner hears the
-- menu close (owner.OnMenuClosed) and the choice made (owner.Refresh), as Menu.Open says. The owner's own tooltip goes
-- as the menu shows (a bar's, over the button, would cover the menu's lowest rows), as a dropdown's does at its click.
function Kit.OpenMenu(owner, generator, above)
	local Menu = ns.Menu
	if Menu.owner == owner then return Menu.Close() end
	Menu.Open(owner, generator)
	local f = Menu.levels[1]
	if not (f and f:IsShown()) then return end -- nothing to choose
	f:ClearAllPoints()
	if above then
		f:SetPoint("BOTTOMLEFT", owner, "TOPLEFT", 0, 2)
	else
		f:SetPoint("TOPLEFT", owner, "BOTTOMLEFT", 0, -2)
	end
	if GameTooltip:IsOwned(owner) then GameTooltip:Hide() end
end

-- Pictures ------------------------------------------------------------------------------------------------------------

-- A picture's dark screen (Style.Box), clipping what it shows to itself; with `quality` its edge in that quality's
-- colour (Kit.PaintScreen).
function Kit.Screen(parent, w, h, quality)
	local screen = Style.Box(parent)
	screen:SetSize(w, h)
	screen:SetClipsChildren(true)
	Kit.PaintScreen(screen, quality)
	return screen
end

-- A screen's ground and edge: the plain edge, or a quality's colour at 0.6 (none for Starter, as its slot wears none;
-- the quality's word goes beside it on the row).
function Kit.PaintScreen(screen, quality)
	local edge = saga.edge
	if quality and quality ~= ns.Enum.quality.starter then
		local r, g, b = ns.Sheet.QualityColor(quality):GetRGB()
		edge = { r, g, b, 0.6 }
	end
	screen:Paint(saga.screen, edge)
end

-- A move's key pose in a screen: the figure pack's still at `size` (Figure.lua, its caption hidden and its clicks
-- passed to the choice under it), or the key-pose icon without the pack; `y` moves it up or down in the screen; `key`
-- names the move by its library key. Kit.Pose draws it at a render, once: a pose that fell back to the icon (the pack
-- not loaded, which it never is in a fight before its first load: Figure.Data) is drawn again at the next render, as
-- the step card's figure is (Tree.CardHead).
function Kit.Figure(screen, key, size, y)
	local fig = ns.Figure.Create(screen, size)
	fig.frame:SetPoint("CENTER", screen, "CENTER", 0, y or 0)
	fig.frame:EnableMouse(false)
	fig.caption:Hide()
	fig.move = Kit.MoveByKey(key)
	return fig
end

function Kit.Pose(fig)
	if fig.posed or not fig.move then return end
	fig.posed = fig:Play(fig.move, "key")
end

-- A move's picture (Style.MovePicture: its key-pose icon, else its slot silhouette) at `size` units in a screen, never
-- over Style.iconPx (a cell drawn larger blurs).
function Kit.Icon(screen, id, size)
	local t = screen:CreateTexture(nil, "ARTWORK")
	t:SetSize(size, size)
	t:SetPoint("CENTER")
	Kit.SetIcon(t, id)
	return t
end

function Kit.SetIcon(t, id)
	local file, l, r, top, bottom = Style.MovePicture(id)
	t:SetTexture(file)
	t:SetTexCoord(l or 0, r or 1, top or 0, bottom or 1)
end

-- The Options window's minimal check box at a choice's corner: its box and its mark (the classic check where the client
-- lacks the atlas), which Kit.Choose shows on a chosen choice; on a frame of its own over `parent`'s children (a
-- figure's frame in a screen).
function Kit.Box(parent, size)
	local holder = CreateFrame("Frame", nil, parent)
	holder:SetSize(size, size)
	holder:SetFrameLevel(parent:GetFrameLevel() + 5)
	local box = holder:CreateTexture(nil, "OVERLAY")
	box:SetAllPoints()
	if not Style.SetAtlas(box, "box") then box:SetColorTexture(unpack(saga.box)) end
	holder.check = holder:CreateTexture(nil, "OVERLAY", nil, 1)
	holder.check:SetAllPoints()
	if not Style.SetAtlas(holder.check, "boxCheck") then Style.SetMark(holder.check, true) end
	return holder
end

-- A box ticked or not, and greyed while unavailable: the box at half its alpha and the disabled mark.
function Kit.BoxState(box, on, unavailable)
	box:SetAlpha(unavailable and 0.5 or 1)
	if not Style.SetAtlas(box.check, unavailable and "boxOff" or "boxCheck") then Style.SetMark(box.check, true) end
	box.check:SetShown(on and true or false)
end

-- A box's click hint, the green last line of its tooltip (Kit.Tip's hint): what a click does to a box ticked or not.
function Kit.BoxHint(on)
	return on and L["ui.click.off"] or L["ui.click.on"]
end

-- Cards and choices ---------------------------------------------------------------------------------------------------

-- A choice's look (Spoken's): chosen, a gold edge, the gold light from the left, the name in gold and the picture in
-- colour; left off, the grey edge and name, the line in the notes' grey and the picture greyed (only the picture is
-- dimmed, so the words keep their contrast). A row of a list (`c.row`) has no edge of its own while left off, so the
-- list reads as one box with its hairline rules between the rows (the D-Setup2 board). `care`: the tick alone, the name
-- white and no gold (the pain flags, the felt-unwell and check-in answers; and a list's check box rows, where the tick
-- carries "on": decision G4). `unavailable` (the choice's reason why, or true): the picture at 0.35, the name in the
-- disabled grey, no click; the reason goes in its tooltip (Kit.Tip's reason).
function Kit.Choose(c, on, care, unavailable)
	c.unavailable = unavailable or nil
	if c.hover then c.hover:SetShown(c.clicks == true and not unavailable) end -- a row that cannot act now never lights
	local gold = on and not care and not unavailable
	c:Paint(saga.cell, gold and saga.chosenEdge or not c.row and saga.edge or nil)
	c.light:SetShown(gold and true or false)
	if c.name then
		if unavailable then
			c.name:SetTextColor(DISABLED_FONT_COLOR:GetRGB())
		elseif gold then
			c.name:SetTextColor(NORMAL_FONT_COLOR:GetRGB())
		elseif care then
			c.name:SetTextColor(HIGHLIGHT_FONT_COLOR:GetRGB())
		else
			c.name:SetTextColor(unpack(saga.note))
		end
	end
	if c.box then Kit.BoxState(c.box, on, unavailable) end
	if c.view then c.view:Dim(not on or unavailable ~= nil) end
	for _, t in ipairs(c.icons or {}) do
		t:SetDesaturated(not on or unavailable ~= nil)
		t:SetAlpha(unavailable and 0.35 or on and 1 or 0.5)
	end
end

-- A card: a box (Style.Box) with the gold light Kit.Choose shows when it is chosen or done, and its tooltip where it
-- has one; no button (the companions' cards, which show a state the player sets up elsewhere).
function Kit.Card(parent, w, h, tip)
	local c = Style.Box(parent)
	c:SetSize(w, h)
	c.light = c:CreateTexture(nil, "BACKGROUND", nil, 1)
	c.light:SetAllPoints()
	c.light:SetColorTexture(1, 1, 1, 1)
	local gold = saga.chosen
	c.light:SetGradient("HORIZONTAL", CreateColor(gold[1], gold[2], gold[3], gold[4]),
		CreateColor(gold[1], gold[2], gold[3], 0))
	if tip then Kit.Hover(c, tip) end
	return c
end

-- A choice: a card that is a button, with a soft light under the cursor, `onClick(c)` on a click anywhere on it (which
-- plays the check box's cue and writes the answer; none while it is unavailable), and its tooltip where it has one. A
-- radio choice clicked again writes nothing and plays nothing (its onClick returns early). onClick(c, mouse): the
-- button that clicked, for a choice that takes right-clicks too (a list row's rightClick). c.hover is the light, which
-- a row that does nothing at a click hides (Kit.Row).
function Kit.Choice(parent, w, h, onClick, tip)
	local c = Kit.Card(parent, w, h)
	c.button = CreateFrame("Button", nil, c)
	c.button:SetAllPoints()
	c.button:SetScript("OnClick", function(_, mouse)
		if c.unavailable then return end
		onClick(c, mouse)
	end)
	c.hover = c.button:CreateTexture(nil, "HIGHLIGHT")
	c.hover:SetAllPoints()
	c.hover:SetColorTexture(unpack(saga.hover))
	c.clicks = true
	if tip then Kit.Hover(c.button, tip) end
	return c
end

-- A chip: what a choice gives or a fact, never a setting and never a button: a box with a picture (Kit.ChipArt, a flat
-- atlas drawn at its own shape in a square Kit.CHIP_ICON across) 16 from its top and its words centred under it, the
-- long words in its tooltip. Kit.ChipHeight answers the height its words need (a row of chips takes the tallest's).
function Kit.Chip(parent, w, art, words, tip)
	local tile = Style.Box(parent)
	tile:SetSize(w, 88)
	tile:Paint(saga.cell, saga.edge)
	if tip then Kit.Hover(tile, tip) end
	tile.icon = tile:CreateTexture(nil, "ARTWORK")
	tile.icon:SetPoint("CENTER", tile, "TOP", 0, -(16 + Kit.CHIP_ICON / 2))
	tile.words = Kit.Text(tile, "bodyWhite", "CENTER", nil, w - 16)
	tile.words:SetPoint("TOP", 0, -(16 + Kit.CHIP_ICON + 6))
	if art then Kit.ChipArt(tile.icon, art) end
	if words then tile.words:SetText(words) end
	return tile
end

-- A chip's picture: the atlas Style.atlas names at its own shape, left out where the client lacks it, or QuestPump's
-- own mark there when `mark` says so.
function Kit.ChipArt(t, art, mark)
	if Style.FitAtlas(t, art, Kit.CHIP_ICON) then return t:Show() end
	t:SetShown(mark == true)
	if mark then
		t:SetTexture(Style.mark)
		t:SetTexCoord(0, 1, 0, 1)
		t:SetSize(Kit.CHIP_ICON, Kit.CHIP_ICON)
	end
end

function Kit.ChipHeight(tile)
	return math.max(88, 16 + Kit.CHIP_ICON + 6 + tile.words:GetStringHeight() + 14)
end

-- Windows and pages ---------------------------------------------------------------------------------------------------

-- Escape closes a window, a menu or a dialog alone, before the game's own Escape, which closes every window in
-- UISpecialFrames, every UI panel and the bags at once, and comes after a cast's stop: a key frame on it that takes
-- Escape only, out of combat. The propagate flag cannot change in a fight, and a shown frame holding the keys there
-- blocks every binding (AGENTS.md), so the key frames hide in a fight and show again after it, each resetting the flag
-- as it shows; the buttons still answer.
local keyFrames, fightWindows = {}, {}

function Kit.EscapeKeys(parent, close)
	local keys = CreateFrame("Frame", nil, parent)
	keys:SetAllPoints()
	keys:EnableKeyboard(true)
	keys:SetScript("OnShow", function(self)
		if not InCombatLockdown() then self:SetPropagateKeyboardInput(true) end
	end)
	keys:SetScript("OnKeyDown", function(self, key)
		if InCombatLockdown() then return end
		self:SetPropagateKeyboardInput(key ~= "ESCAPE")
		if key == "ESCAPE" then close() end
	end)
	keys:SetScript("OnKeyUp", function(self)
		if not InCombatLockdown() then self:SetPropagateKeyboardInput(true) end
	end)
	if InCombatLockdown() then keys:Hide() else keys:SetPropagateKeyboardInput(true) end
	keyFrames[#keyFrames + 1] = keys
	return keys
end

ns.RegisterEvent("PLAYER_REGEN_DISABLED", function()
	for _, keys in ipairs(keyFrames) do keys:Hide() end
	for _, win in ipairs(fightWindows) do win:Hide() end
end)
ns.RegisterEvent("PLAYER_REGEN_ENABLED", function()
	for _, keys in ipairs(keyFrames) do keys:Show() end
end)

-- A window of the tier `tier` (Kit.TIER's name, or the tier) in the game's own frame (DefaultPanelFlatTemplate, the
-- Options window's flat ground in the panel's metal) with its title in the title bar and its close button, centred,
-- clamped to the screen and above the world's frames (DIALOG), built hidden; its content goes on win.body, a frame of
-- its own over it (a UI replacement's skin strips the window's own textures). Named, so a gamepad cursor can reach it
-- (ConsolePort keys the frames it tracks by name: Skins.lua). opts: `onClose` (the close button's and Escape's answer;
-- it hides the window without one); `escape`: "special" (in UISpecialFrames: the game's Escape closes it, a side
-- window), or "own" (an Escape-only key frame, for a flow only its answers close: kept out of UISpecialFrames, whose
-- CloseSpecialWindows would only hide it, and handed to ConsolePort's cursor by name, Kit.cursor); `fight` (hidden when
-- a fight begins); `movable` (dragged by its body, out of combat); `strata`; `y` (from the screen's middle); `skin`
-- false (kept out of a UI replacement's frame skin, its widgets skinned alone: the QR window, whose code a strip of
-- its textures would take); `fit` "fixed" (Kit.Fit sizes it to its content and never scales it, whatever its tier's
-- rule: the QR window, whose code's modules a scale would move off the pixel grid); `toplevel` false (a click never
-- raises it over the other frames of its strata: the pump card, alone in its dim at ns.Menu's strata, where a raise
-- has nothing to lift it over). Recorded in Kit.windows for Skins.lua.
function Kit.Window(name, tier, title, opts)
	opts = opts or {}
	local t = Kit.Tier(tier)
	local win = CreateFrame("Frame", name, UIParent, "DefaultPanelFlatTemplate")
	win:SetSize(t.W, t.H or t.least or t.W)
	win:SetPoint("CENTER", 0, opts.y or 0)
	win:SetClampedToScreen(true)
	win:SetFrameStrata(opts.strata or "DIALOG")
	win:SetToplevel(opts.toplevel ~= false)
	win:EnableMouse(true)
	win:SetTitle(title)
	win:Hide()
	win.tier, win.fit = t, opts.fit or t.fit
	win.body = CreateFrame("Frame", nil, win)
	win.body:SetAllPoints()
	local close = opts.onClose or function() win:Hide() end
	win.close = CreateFrame("Button", nil, win, "UIPanelCloseButtonDefaultAnchors")
	win.close:SetScript("OnClick", function() close() end)
	if opts.escape == "own" then
		win.keys = Kit.EscapeKeys(win, close)
		Kit.cursor[win] = true
	elseif opts.escape == "special" then
		tinsert(UISpecialFrames, name)
	end
	if opts.fight then fightWindows[#fightWindows + 1] = win end
	if opts.movable then
		win:SetMovable(true)
		win:RegisterForDrag("LeftButton")
		win:SetScript("OnDragStart", function(self) if not InCombatLockdown() then self:StartMoving() end end)
		win:SetScript("OnDragStop", function(self) self:StopMovingOrSizing() end)
	end
	Kit.windows[win] = opts.skin ~= false and "HandleFrame" or false
	return win
end

-- Shows a window, or hides it (`open` false).
function Kit.Open(win, open)
	win:SetShown(open ~= false)
end

-- A Settings page in Blizzard's Options panel (a canvas, never the vertical list: Settings.lua): Blizzard's title at
-- its top left, the Options window's faded rule across under it and the page's one action, a grey button where
-- Blizzard puts Defaults (opts.action: { label, onClick, tip }). No red on a canvas: a change applies at once.
-- opts.scroll: a scroll list from the content's top to the foot (f.scroll, its content f.content, as wide as the
-- page's content beside the bar: content.contentW, at its own left, so Kit.At and Kit.Section place in it).
-- Kit.Top(f) answers where the content starts. Recorded in Kit.windows with no chrome of its own to skin.
function Kit.Canvas(title, opts)
	opts = opts or {}
	local t = Kit.TIER.canvas
	local f = CreateFrame("Frame")
	f:SetSize(t.W, t.H)
	f.tier, f.inset, f.contentW = t, t.IN, t.TEXT
	f.title = Kit.Text(f, t.title, "LEFT")
	f.title:SetPoint("TOPLEFT", t.TITLE_X, -t.HEAD)
	f.title:SetText(title)
	f.rule = Kit.HRule(f, t.W - t.TITLE_X - t.IN)
	f.rule:SetPoint("TOPLEFT", t.TITLE_X, -t.RULE)
	f.rule:SetPoint("TOPRIGHT", -t.IN, -t.RULE)
	if opts.action then
		local a = opts.action
		f.action = Kit.Secondary(f, a[1], 1, a[2], t.BUTTON)
		f.action:SetWidth(math.max(t.BACK, f.action:GetTextWidth() + 40))
		f.action:SetPoint("TOPRIGHT", -t.IN, -t.IN)
		if a[3] then Kit.Hover(f.action, a[3], "ANCHOR_TOP") end
	end
	if opts.scroll then
		f.scroll = Kit.Scroll(f)
		f.scroll:SetPoint("TOPLEFT", t.IN, -Kit.Top(f))
		f.scroll:SetPoint("BOTTOMRIGHT", -t.IN - t.SCROLLBAR, 8)
		f.content = f.scroll.content
		f.content.contentW, f.content.inset = t.TEXT - t.SCROLLBAR, 0
		f.content:SetWidth(f.content.contentW)
	end
	Kit.windows[f] = false
	return f
end

-- A cut label's tooltip on `owner`: nothing while its words show whole, or while it hides.
local function cutTip(owner, fs, head, anchor)
	if not (fs:IsVisible() and fs:IsTruncated()) then return end
	local words = fs:GetText()
	GameTooltip:SetOwner(owner, anchor or "ANCHOR_RIGHT")
	Kit.Tip(head or words, head and words or nil)(GameTooltip)
	GameTooltip:Show()
end

-- Words the client cuts with its ellipsis (a font string held to a width with no wrap) show whole in a tooltip, under
-- `head` (its title: words, or a function asked at the hover; without one the words are the title), only while they
-- are cut: a frame laid over them takes the hover and lets clicks through (a sheet page's line, the Body page's count
-- label, a name on the card window or the day card; a link chip answers on its own button). Answers the frame.
function Kit.WholeWords(fs, head, anchor)
	local hit = CreateFrame("Frame", nil, fs:GetParent())
	hit:SetAllPoints(fs)
	hit:EnableMouse(true)
	hit:SetMouseClickEnabled(false)
	hit:SetScript("OnEnter", function(self)
		cutTip(self, fs, type(head) == "function" and head() or head, anchor)
	end)
	hit:SetScript("OnLeave", function(self)
		if GameTooltip:IsOwned(self) then GameTooltip:Hide() end
	end)
	return hit
end

-- A page's head on `view` by the tier's kind. A window's (L, M, card): the title (the tier's font, centred, at its
-- top), the Options window's faded rule under it and the intro line under that (soft, centred: a value, a status or
-- the safety note, never a tagline; its words may change with the page's state, its title never does). A sheet
-- page's: the title at the left, the line right-aligned on its row in the notes' grey, the rule under them; `right`, a
-- control on the title's row at its right (the Talents tab's search box), puts the line right of the title instead,
-- one line up to the control. A sheet page's line runs from 16
-- after the title (whose words may change: the Guild tab's name) to the row's end or 12 before the control, held there
-- by its anchors at any Text Size, and a line cut there (the client's ellipsis) shows its words whole in a tooltip.
-- The view keeps its tier, its content's inset and width (Kit.At, Kit.Section, Kit.Top).
function Kit.Head(view, title, intro, tier, right)
	local t = Kit.Tier(tier)
	view.tier, view.inset, view.contentW = t, t.IN, t.TEXT
	if t.kind == "page" then
		view.title = Kit.Text(view, t.title, "LEFT")
		view.title:SetPoint("TOPLEFT", t.IN, -t.HEAD)
		view.rule = Kit.HRule(view, t.TEXT)
		view.rule:SetPoint("TOPLEFT", view.title, "BOTTOMLEFT", 0, -t.ruleGap)
		view.intro = Kit.Text(view, "bodyWhite", right and "LEFT" or "RIGHT", saga.note)
		view.intro:SetWordWrap(false)
		-- both ends on the title's bottom line: the rule hangs ruleGap under it
		view.intro:SetPoint("BOTTOMLEFT", view.title, "BOTTOMRIGHT", 16, 0)
		if right then
			right:ClearAllPoints()
			right:SetPoint("BOTTOMRIGHT", view.rule, "TOPRIGHT", 0, t.ruleGap - 2)
			view.intro:SetPoint("BOTTOMRIGHT", view.rule, "TOPRIGHT", -(12 + right:GetWidth()), t.ruleGap)
		else
			view.intro:SetPoint("BOTTOMRIGHT", view.rule, "TOPRIGHT", 0, t.ruleGap)
		end
		Kit.WholeWords(view.intro, function() return view.title:GetText() end, "ANCHOR_BOTTOM")
	else
		view.title = Kit.Text(view, t.title, "CENTER", nil, t.TEXT)
		view.title:SetPoint("TOP", 0, -t.HEAD)
		view.rule = Kit.HRule(view, t.TEXT)
		view.rule:SetPoint("TOP", view.title, "BOTTOM", 0, -t.ruleGap)
		view.intro = Kit.Text(view, "bodyWhite", "CENTER", saga.soft, t.TEXT)
		view.intro:SetPoint("TOP", view.rule, "BOTTOM", 0, -t.introGap)
	end
	view.title:SetText(title)
	if intro then view.intro:SetText(intro) end
	return view
end

-- A page of a window: a frame over the whole window (its content in the window's coordinates, under the bar's
-- buttons), hidden until shown, with its head.
function Kit.Page(win, title, intro)
	local v = CreateFrame("Frame", nil, win)
	v:SetAllPoints()
	v:Hide()
	return Kit.Head(v, title, intro, win.tier)
end

-- The y under a page's head, where its content starts.
function Kit.Top(view)
	local t = view.tier
	if t.kind == "canvas" then return t.RULE + t.contentGap end
	if t.kind == "page" then return t.HEAD + view.title:GetStringHeight() + t.ruleGap + 1 + t.contentGap end
	-- a window's intro hidden (the pump card's move of a pump of one) takes no room
	local intro = view.intro:IsShown() and t.introGap + view.intro:GetStringHeight() or 0
	return t.HEAD + view.title:GetStringHeight() + t.ruleGap + 1 + intro + t.contentGap
end

-- Puts a frame's top left at `y` on the page, the content's inset in (plus `x`).
function Kit.At(frame, view, y, x)
	frame:ClearAllPoints()
	frame:SetPoint("TOPLEFT", view, "TOPLEFT", (view.inset or 0) + (x or 0), -y)
end

-- A section's label: its name in small caps and a faded rule beside it (both sides when `centred`), on a frame of its
-- own the content's width, 12 high, which Kit.At places (16-18 under the block above, its content 14 under it).
-- `aside`: a note right-aligned at the rule's end ("59 moves · 5 pumps"), the rule stopping 12 before it; sec.aside
-- takes new words. A frame as `aside` (the day's strip's legend, its width set) stands there in the note's place.
function Kit.Section(view, words, centred, aside)
	local sec = CreateFrame("Frame", nil, view)
	sec:SetSize(view.contentW or view:GetWidth(), 12)
	local fs = Kit.Caps(sec, centred and "CENTER" or "LEFT")
	Kit.SetCaps(fs, words)
	fs:SetPoint(centred and "CENTER" or "LEFT")
	sec.words = fs
	local stop = { "RIGHT", sec, "RIGHT", 0 }
	if aside then
		if type(aside) == "string" then
			sec.aside = Kit.Text(sec, "bodyWhite", "RIGHT", saga.note)
			sec.aside:SetText(aside)
		else
			sec.aside = aside
			aside:SetParent(sec)
			aside:ClearAllPoints()
		end
		sec.aside:SetPoint("RIGHT")
		stop = { "RIGHT", sec.aside, "LEFT", -12 }
	end
	local function line(from, to)
		local t = sec:CreateTexture(nil, "ARTWORK")
		t:SetHeight(1)
		t:SetColorTexture(unpack(saga.rule))
		t:SetPoint(from[1], from[2], from[3], from[4], 0)
		t:SetPoint(to[1], to[2], to[3], to[4], 0)
	end
	if centred then line({ "LEFT", sec, "LEFT", 0 }, { "RIGHT", fs, "LEFT", -12 }) end
	line({ "LEFT", fs, "RIGHT", 12 }, stop)
	return sec
end

-- Numbers -------------------------------------------------------------------------------------------------------------

-- A row of a big number: the section's name (sectionTitle) at the left, the number (poolNumber) and its word (soft)
-- at the right, the number in `color` ({ r, g, b }: gold for a number chosen or won, white without), which
-- Kit.LayNumber writes and places at `y`, answering the row's height. One big number a view.
function Kit.NumberRow(view, name, word, color)
	local r = { view = view }
	r.name = Kit.Text(view, "sectionTitle", "LEFT")
	r.name:SetText(name)
	r.word = Kit.Text(view, "bodyWhite", "RIGHT", saga.soft)
	r.word:SetText(word)
	r.number = Kit.Text(view, "poolNumber", "RIGHT", color)
	r.number:SetPoint("BOTTOMRIGHT", r.word, "BOTTOMLEFT", -6, -4)
	return r
end

function Kit.LayNumber(r, y, value)
	r.number:SetText(value)
	local h = math.max(r.number:GetStringHeight(), r.name:GetStringHeight())
	local inset = r.view.inset or 0
	r.name:ClearAllPoints()
	r.name:SetPoint("BOTTOMLEFT", r.view, "TOPLEFT", inset, -(y + h - 4))
	r.word:ClearAllPoints()
	r.word:SetPoint("BOTTOMRIGHT", r.view, "TOPRIGHT", -inset, -(y + h - 4))
	return h
end

-- A big number and its word to its right on the number's line (4 up: the large font's descent), which the caller
-- places by n.number: `font` poolNumber (40), dayNumber (72) or goal (24); the word in `wordFont` (bodyWhite, soft;
-- dayWord beside a day's number); the number in `color` (gold for a number won or chosen). Kit.SetNumber writes both.
function Kit.Number(parent, font, color, wordFont)
	local n = {}
	n.number = Kit.Text(parent, font, "LEFT", color)
	n.word = Kit.Text(parent, wordFont or "bodyWhite", "LEFT", saga.soft)
	n.word:SetPoint("BOTTOMLEFT", n.number, "BOTTOMRIGHT", 6, 4)
	return n
end

function Kit.SetNumber(n, value, word)
	n.number:SetText(type(value) == "number" and BreakUpLargeNumbers(value) or value)
	n.word:SetText(word or "")
end

-- A stat: its number (statNumber) and its word (the notes' grey) side by side, centred in a cell the caller sizes and
-- places, with a rule at its left (opts.rule, between two cells); or, opts.inline, a stat as a row's part: the number
-- right-aligned in a 36-unit column, its word beside it. Kit.FillStat writes it.
local STAT_PAD, STAT_COLUMN = 6, 36

function Kit.Stat(parent, opts)
	opts = opts or {}
	local cell = CreateFrame("Frame", nil, parent)
	cell.inline = opts.inline == true
	cell.number = Kit.Text(cell, "statNumber", cell.inline and "RIGHT" or "LEFT")
	cell.word = Kit.Text(cell, "labelWhite", "LEFT", saga.note)
	if cell.inline then
		cell.number:SetWidth(STAT_COLUMN)
		cell.number:SetPoint("LEFT")
		cell.word:SetPoint("BOTTOMLEFT", cell.number, "BOTTOMRIGHT", 6, 2)
	end
	if opts.rule then
		local sep = Kit.Rule(cell, saga.rule, true)
		sep:SetPoint("TOPLEFT", 0, -9)
		sep:SetPoint("BOTTOMLEFT", 0, 9)
	end
	return cell
end

-- One stat's number and word: centred in the cell side by side, the word on the number's line (its bottom two units
-- up: the large font's descent); the word after the number, or before it (`wordFirst`: "QP 23 > 24"). The number in
-- `color`, else white, and smaller where it would not fit beside its word ("Paragon 12 > 13").
function Kit.FillStat(cell, number, word, wordFirst, color)
	local n, w = cell.number, cell.word
	n:SetText(number)
	w:SetText(word)
	n:SetTextColor((color or HIGHLIGHT_FONT_COLOR):GetRGB())
	if cell.inline then return end
	n:SetTextScale(1)
	local room, numberW = cell:GetWidth() - 2 * STAT_PAD - 5 - w:GetStringWidth(), n:GetStringWidth()
	if numberW > room then
		local scale = math.max(0.5, room / numberW)
		n:SetTextScale(scale)
		numberW = numberW * scale
	end
	local width = numberW + 5 + w:GetStringWidth()
	local left = (cell:GetWidth() - width) / 2 + (wordFirst and w:GetStringWidth() + 5 or 0)
	n:ClearAllPoints()
	w:ClearAllPoints()
	n:SetPoint("LEFT", cell, "LEFT", left, 0)
	if wordFirst then
		w:SetPoint("BOTTOMRIGHT", n, "BOTTOMLEFT", -5, 2)
	else
		w:SetPoint("BOTTOMLEFT", n, "BOTTOMRIGHT", 5, 2)
	end
end

-- A progress bar `w` units across, 6 high: its track (a picture's ground with the list's edge), the fill in the dots'
-- darker gold while under way and gold once full; bar:Set(have, need). The count goes beside it in words, never the bar
-- alone (and the check with a full one).
local function setBar(bar, have, need)
	local share = need and need > 0 and math.max(0, math.min(1, (have or 0) / need)) or 0
	bar.fill:SetShown(share > 0)
	bar.fill:SetWidth(math.max(1, (bar:GetWidth() - 2) * share))
	local color = share >= 1 and { NORMAL_FONT_COLOR:GetRGB() } or saga.dotBehind
	bar.fill:SetColorTexture(color[1], color[2], color[3], 1)
end

function Kit.Bar(parent, w)
	local bar = Style.Box(parent)
	bar:SetSize(w, 6)
	bar:Paint(saga.screen, saga.listEdge)
	bar.fill = bar:CreateTexture(nil, "ARTWORK")
	bar.fill:SetPoint("TOPLEFT", 1, -1)
	bar.fill:SetPoint("BOTTOMLEFT", 1, 1)
	bar.Set = setBar
	bar:Set(0, 1)
	return bar
end

-- A quality's badge: a small box 18 high, its ground the quality's colour at 0.12 and its edge at 0.6, its word inside
-- (Kit.QualityWords, the numeral with it under the colour-blind mode); q "gold" with `words`: the same shape in gold
-- (the "your step" marker, "Your title"). badge:Set(q, words) changes it; it only shows (no state).
local function setBadge(badge, q, words)
	local r, g, b
	if q == "gold" then r, g, b = NORMAL_FONT_COLOR:GetRGB() else r, g, b = ns.Sheet.QualityColor(q):GetRGB() end
	badge:Paint({ r, g, b, 0.12 }, { r, g, b, 0.6 })
	badge.words:SetText(words or Kit.QualityWords(q))
	badge.words:SetTextColor(r, g, b)
	badge:SetWidth(badge.words:GetStringWidth() + 12)
end

function Kit.Badge(parent, q, words)
	local badge = Style.Box(parent)
	badge:SetHeight(18)
	badge.words = Kit.Text(badge, "label", "CENTER")
	badge.words:SetPoint("CENTER")
	badge.Set = setBadge
	badge:Set(q, words)
	return badge
end

-- Blizzard's own NEW (NewFeatureLabelTemplate, Gethe forever and live Blizzard_SharedXML/NewFeatureLabel.xml: its word
-- the client's NEW_CAPS, its glow pulsing while it shows), shown, for the caller to place (10 right of a title) and to
-- hide once the player has seen it (on hover).
function Kit.NewLabel(parent)
	local label = CreateFrame("Frame", nil, parent, "NewFeatureLabelTemplate")
	label:Show()
	return label
end

-- Lists ---------------------------------------------------------------------------------------------------------------

-- A framed list: one box (the list's ground, its bronze edge) holding rows with no gap between them and a hairline 8
-- in from each side between two (Kit.Row, laid out by Kit.LayList). `nameW`: the rows' name column (where a row's line
-- starts), the large tier's by default.
function Kit.List(parent, w, nameW)
	local list = Style.Box(parent)
	list:SetWidth(w)
	list:Paint(saga.list, saga.listEdge)
	list.rows, list.nameW = {}, nameW or Kit.TIER.L.rowName
	return list
end

-- A row's mark (the setup keeps it out): the setting's gear tinted in the notes' grey at the line's place and the
-- reason beside it, in place of the line; nil takes it away.
local function rowMark(row, text)
	local mark = rawget(row, "mark")
	if not mark then
		if text == nil then return end
		mark = row:CreateTexture(nil, "ARTWORK")
		mark:SetSize(15, 16)
		if Style.SetAtlas(mark, "gear") then mark:SetVertexColor(unpack(saga.note)) else mark:SetSize(1, 16) end
		row.mark = mark
		row.markText = Kit.Text(row, "bodyWhite", "LEFT", saga.note)
		row.markText:SetPoint("LEFT", mark, "RIGHT", 6, 0)
	end
	mark:SetShown(text ~= nil)
	row.markText:SetShown(text ~= nil)
	row.markText:SetText(text or "")
	if row.line then row.line:SetShown(text == nil) end
end

-- A list's row: a choice as wide as the list's inside, with no edge of its own, a click anywhere on it doing what its
-- control does (onClick(row)), Kit.Choose showing its state. opts: `name`; `font` (the name's: rowName, the list row
-- name; bodyWhite for an answer that is a sentence); `line` (the row's own value or state in the notes' grey, never a
-- hint: at the name column, or under the name with `below`); a picture at the left: `picture` (a move id, its key-pose
-- icon 40 in a 52 x 36 screen; true for the screen and an empty icon, row.icons[1], that a pooled row's render fills:
-- Kit.SetIcon, Kit.ChipArt), `atlas` (Style's atlas key, at its own shape inside 28 in the same screen; the screen
-- empty where the client lacks it) or `icon24` (a texture file, 24 with no screen); `indent` (everything that much
-- further right: a group's moves); the control at its right: `box` (a check box), `chevron` (a row that opens
-- something), `value` (words in white, row.value), or `control` (a frame the caller made, which the row takes as its
-- child: a select, the Log's time and result); `menu` (a generator: a 26-unit segment at the row's right behind a
-- hairline, with the select's arrow, opening ns.Menu apart from the row's own click); `rightClick(row)` (the row takes
-- right-clicks too, apart from its click: a record's clear menu); `tip`. A row with neither onClick nor rightClick
-- shows no light under the cursor: it only shows (its control, if any, takes its own clicks).
-- row:Mark(text) shows the setup's mark in place of the line (nil takes it away). row.lead, a frame the caller makes
-- (a move's quality badge), stands before the line while it shows, the line starting 8 after it (Kit.LayList).
local ROW_SEGMENT = 26

function Kit.Row(list, opts)
	local w = list:GetWidth() - 6
	local click, rightClick = opts.onClick, opts.rightClick
	local row = Kit.Choice(list, w, 44, function(c, mouse)
		if mouse == "RightButton" then
			if rightClick then rightClick(c) end
		elseif click then
			click(c)
		end
	end, opts.tip)
	if rightClick then row.button:RegisterForClicks("LeftButtonUp", "RightButtonUp") end
	row.clicks = click ~= nil or rightClick ~= nil
	row.hover:SetShown(row.clicks)
	row.row, row.below, row.Mark, row.lead = true, opts.below == true, rowMark, false
	local x = 12 + (opts.indent or 0)
	row.screen, row.icon = false, false
	if opts.picture or opts.atlas then
		row.screen = Kit.Screen(row, 52, 36)
		row.screen:SetPoint("LEFT", 6 + (opts.indent or 0), 0)
		if opts.picture then
			row.icons = { Kit.Icon(row.screen, opts.picture ~= true and opts.picture or nil, 40) }
		else
			local art = row.screen:CreateTexture(nil, "ARTWORK")
			art:SetPoint("CENTER")
			art:SetShown(Style.FitAtlas(art, opts.atlas, 28))
			row.icons = { art }
		end
		x = 6 + (opts.indent or 0) + 52 + 12
	elseif opts.icon24 then
		row.icon = row:CreateTexture(nil, "ARTWORK")
		row.icon:SetSize(24, 24)
		row.icon:SetPoint("LEFT", x, 0)
		row.icon:SetTexture(opts.icon24)
		row.icons = { row.icon }
		x = x + 24 + 12
	end
	row.x, row.nameW = x, list.nameW
	row.least = (opts.picture or opts.atlas or opts.icon24) and 44 or 36
	row.name = Kit.Text(row, opts.font or "rowName", "LEFT", nil, (row.below and w - x - 12 or list.nameW - 12))
	if opts.name then row.name:SetText(opts.name) end
	row.line = false
	if opts.line then
		row.line = Kit.Text(row, "bodyWhite", "LEFT", saga.note, 1)
		row.line:SetText(opts.line)
	end
	-- the control and the room it takes at the right
	-- (a control of its own takes its clicks over the row's button, above it)
	local right, above = -12, row.button:GetFrameLevel() + 1
	if opts.menu then
		row.menu = CreateFrame("Button", nil, row)
		row.menu:SetFrameLevel(above)
		row.menu:SetPoint("TOPRIGHT")
		row.menu:SetPoint("BOTTOMRIGHT")
		row.menu:SetWidth(ROW_SEGMENT)
		row.menu.arrow = row.menu:CreateTexture(nil, "ARTWORK")
		row.menu.arrow:SetSize(20, 20)
		row.menu.arrow:SetPoint("CENTER")
		if not Style.SetAtlas(row.menu.arrow, "selectArrow") then
			row.menu.arrow:SetTexture(Style.menuArrow)
			row.menu.arrow:SetRotation(-math.pi / 2)
		end
		local hover = row.menu:CreateTexture(nil, "HIGHLIGHT")
		hover:SetAllPoints()
		hover:SetColorTexture(unpack(saga.hover))
		local rule = Kit.Rule(row.menu, saga.listRule, true)
		rule:SetPoint("TOPLEFT", 0, -6)
		rule:SetPoint("BOTTOMLEFT", 0, 6)
		row.menu:SetScript("OnClick", function(self) Kit.OpenMenu(self, opts.menu) end)
		right = right - ROW_SEGMENT
	end
	row.box, row.chevron, row.value, row.control = false, false, false, false
	if opts.box then
		row.box = Kit.Box(row, 22)
		row.box:SetPoint("RIGHT", right, 0)
		row.controlW = 22
	elseif opts.chevron then
		row.chevron = row:CreateTexture(nil, "ARTWORK")
		row.chevron:SetSize(16, 16)
		row.chevron:SetPoint("RIGHT", right, 0)
		row.chevron:SetTexture(Style.menuArrow)
		row.controlW = 16
	elseif opts.value then
		row.value = Kit.Text(row, "bodyWhite", "RIGHT")
		row.value:SetPoint("RIGHT", right, 0)
		row.value:SetText(opts.value)
		row.controlW = 0 -- the words' width, at each layout
	elseif opts.control then
		row.control = opts.control
		row.control:SetParent(row)
		row.control:SetFrameLevel(above)
		row.control:ClearAllPoints()
		row.control:SetPoint("RIGHT", row, "RIGHT", right, 0)
		row.controlW = 0 -- the control's width, at each layout
	else
		row.controlW = 0
	end
	row.right = -right
	-- the hairline over it, shown on every row but the first that shows (Kit.LayList)
	row.sep = row:CreateTexture(nil, "ARTWORK")
	row.sep:SetColorTexture(unpack(saga.listRule))
	row.sep:SetHeight(1)
	row.sep:SetPoint("TOPLEFT", 8, 0)
	row.sep:SetPoint("TOPRIGHT", -8, 0)
	list.rows[#list.rows + 1] = row
	return row
end

-- A group's row (the Exercises page): a list row 44 high that opens and closes the rows under it, the triangle at its
-- left (closed, or open), its picture after it, and its own check box at its right, which acts apart from the row: a
-- click on the row opens or closes it (opts.onToggle(open)), a click on the box ticks it (opts.onBox(on)), the box's
-- target its own button over the row's whole height from the row's right edge, the box centred in it (wider than
-- WCAG 2.2's 24, so a near miss beside the box never opens the group instead), with its own tooltip (opts.boxTip).
-- row:SetOpen(open) draws the triangle; Kit.BoxState(row.box, on, unavailable) the box.
local GROUP_BOX = 22

local function setGroupOpen(row, open)
	row.open = open and true or false
	if not Style.SetAtlas(row.triangle, row.open and "groupOpen" or "groupClosed") then
		row.triangle:SetTexture(Style.menuArrow)
		row.triangle:SetRotation(row.open and -math.pi / 2 or 0)
	end
end

function Kit.GroupRow(list, opts)
	local row
	row = Kit.Row(list, { picture = opts.picture, name = opts.name, line = opts.line, tip = opts.tip, indent = 24,
		onClick = function()
			Style.Play("tab")
			row:SetOpen(not row.open)
			if opts.onToggle then opts.onToggle(row.open) end
		end })
	row.triangle = row:CreateTexture(nil, "ARTWORK")
	row.triangle:SetSize(16, 11)
	row.triangle:SetPoint("LEFT", 10, 0)
	row.SetOpen = setGroupOpen
	row:SetOpen(opts.open)
	-- the box where a row's box stands (row.right in from the edge), its button as wide again on its other side
	row.boxButton = CreateFrame("Button", nil, row)
	row.boxButton:SetFrameLevel(row.button:GetFrameLevel() + 1) -- its clicks over the row's
	row.boxButton:SetPoint("TOPRIGHT")
	row.boxButton:SetPoint("BOTTOMRIGHT")
	row.boxButton:SetWidth(2 * row.right + GROUP_BOX)
	row.box = Kit.Box(row.boxButton, GROUP_BOX)
	row.box:SetPoint("CENTER")
	row.controlW = GROUP_BOX
	row.boxButton:SetScript("OnClick", function()
		if row.unavailable then return end
		local on = not row.box.check:IsShown()
		Style.Play(on and "checkOn" or "checkOff")
		Kit.BoxState(row.box, on)
		if opts.onBox then opts.onBox(on) end
	end)
	if opts.boxTip then Kit.Hover(row.boxButton, opts.boxTip) end
	return row
end

-- Stacks a list's rows that show, each as tall as its words need and its least, its line at the name column (or under
-- the name), after the row's lead where one shows, as wide as the control leaves, and answers the list's height.
function Kit.LayList(list)
	local y, first = 3, true
	for _, row in ipairs(list.rows) do
		if row:IsShown() then
			local w = row:GetWidth()
			local controlW = row.value and row.value:GetStringWidth() or row.control and row.control:GetWidth()
				or row.controlW
			local lineX = row.below and row.x or row.x + row.nameW
			local room = w - row.right - controlW - (controlW > 0 and 12 or 0) -- the words' right end
			local line, lead = row.line, row.lead and row.lead:IsShown() and row.lead or nil
			local leadW = lead and lead:GetWidth() + 8 or 0
			if line then line:SetWidth(math.max(1, room - lineX - leadW)) end
			if row.below then row.name:SetWidth(math.max(1, room - row.x)) end
			local nameH = row.name:GetStringHeight()
			local lineH = math.max(line and line:IsShown() and line:GetStringHeight() or 0, lead and lead:GetHeight() or 0)
			local h
			if row.below then
				h = math.max(row.least, nameH + (lineH > 0 and 2 + lineH or 0) + 12)
				local top = math.floor((h - nameH - (lineH > 0 and 2 + lineH or 0)) / 2)
				row.name:ClearAllPoints()
				row.name:SetPoint("TOPLEFT", row.x, -top)
				if lead then
					lead:ClearAllPoints()
					lead:SetPoint("TOPLEFT", row.name, "BOTTOMLEFT", 0, -2)
				end
				if line then
					line:ClearAllPoints()
					line:SetPoint("TOPLEFT", row.name, "BOTTOMLEFT", leadW, -2)
				end
			else
				h = math.max(row.least, nameH + 12, lineH + 12)
				row.name:ClearAllPoints()
				row.name:SetPoint("LEFT", row.x, 0)
				if lead then
					lead:ClearAllPoints()
					lead:SetPoint("LEFT", row, "LEFT", lineX, 0)
				end
				if line then
					line:ClearAllPoints()
					line:SetPoint("LEFT", lineX + leadW, 0)
				end
			end
			local mark = rawget(row, "mark")
			if mark then
				mark:ClearAllPoints()
				if row.below then
					mark:SetPoint("TOPLEFT", row.name, "BOTTOMLEFT", 0, -2)
				else
					mark:SetPoint("LEFT", lineX, 0)
				end
			end
			row:SetHeight(h)
			row:ClearAllPoints()
			row:SetPoint("TOPLEFT", 3, -y)
			row.sep:SetShown(not first)
			first = false
			y = y + h
		end
	end
	list:SetHeight(y + 3)
	return y + 3
end

-- Buttons and the bottom bar ------------------------------------------------------------------------------------------

-- The red primary: Blizzard's panel button, `width` units (140 without), Kit.BUTTON_H high; `sound`: the cue its click
-- plays, Style's key ("start" unless named; b.sound, which a render can change). onClick(b). Its tooltip shows while it
-- is greyed too, so a disabled one can say why (Kit.Tip's reason).
function Kit.Primary(parent, label, width, onClick, sound)
	local b = CreateFrame("Button", nil, parent, "UIPanelButtonTemplate")
	b:SetSize(width or 140, Kit.BUTTON_H)
	b:SetText(label)
	b:SetMotionScriptsWhileDisabled(true)
	b.sound = sound or "start"
	b:SetScript("OnClick", function(self)
		Style.Play(self.sound)
		onClick(self)
	end)
	return b
end

-- A grey secondary (a way back or out): Blizzard's grey stretch button, in the label's white font, `height` units
-- high (Kit.BUTTON_H without); its click plays the tab's cue. onClick(b).
function Kit.Secondary(parent, label, width, onClick, height)
	local b = CreateFrame("Button", nil, parent, "UIMenuButtonStretchTemplate")
	b:SetSize(width, height or Kit.BUTTON_H)
	b:SetNormalFontObject(Style.font.labelWhite)
	b:SetHighlightFontObject(Style.font.labelWhite)
	b:SetText(label)
	b:SetMotionScriptsWhileDisabled(true)
	b.kitClick = onClick
	b:SetScript("OnClick", function(self)
		Style.Play("tab")
		onClick(self)
	end)
	return b
end

-- A grey button that opens a menu (the pump card's Pause, a bar's More): the dropdown toggle's arrow at its right,
-- turned down (Blizzard's UIResettableDropdownButtonTemplate Icon, Gethe forever SharedUIPanelTemplates.xml, 70425:
-- the template's mixin nudges b.Icon as it is pressed), its words moved left of it; its words' width counts the
-- arrow, so the bar's fit rule leaves the arrow its room.
local CARET = 14

function Kit.Caret(b)
	b.Icon = b:CreateTexture(nil, "ARTWORK")
	b.Icon:SetSize(10, 12)
	b.Icon:SetPoint("RIGHT", -9, 0)
	b.Icon:SetTexture(Style.menuArrow)
	b.Icon:SetRotation(-math.pi / 2)
	local words = b.Text or b:GetFontString()
	if words then
		words:ClearAllPoints()
		words:SetPoint("CENTER", -CARET / 2, -1)
	end
	local width = b.GetTextWidth
	b.GetTextWidth = function(self) return width(self) + CARET end
	if b.barWidth then
		b.barWidth = math.max(b.barWidth, b:GetTextWidth() + 40)
		b:SetWidth(b.barWidth)
	end
	return b
end

-- The tutorial's yellow glow round a red button while something waits on it (the talent frame's apply button's, Gethe
-- forever Blizzard_SharedTalentFrameTemplates.xml): its three pieces at their own size from 12 left of the button to
-- 12 right of it, drawn ADD; the action bar's proc glow where the client lacks them (Style.ProcGlow). Built hidden;
-- Kit.ShowGlow shows or hides it.
function Kit.Glow(b)
	local glow = CreateFrame("Frame", nil, b)
	glow:SetPoint("LEFT", -12, 0)
	glow:SetPoint("RIGHT", 12, 0)
	glow:SetHeight(1)
	local left, right, mid = glow:CreateTexture(nil, "ARTWORK"), glow:CreateTexture(nil, "ARTWORK"),
		glow:CreateTexture(nil, "ARTWORK")
	if Style.SetAtlas(left, "nextGlowLeft", true) and Style.SetAtlas(right, "nextGlowRight", true)
		and Style.SetAtlas(mid, "nextGlowMid") then
		for _, t in ipairs({ left, right, mid }) do t:SetBlendMode("ADD") end
		left:SetPoint("LEFT")
		right:SetPoint("RIGHT")
		mid:SetPoint("TOPLEFT", left, "TOPRIGHT")
		mid:SetPoint("BOTTOMRIGHT", right, "BOTTOMLEFT")
	else
		for _, t in ipairs({ left, right, mid }) do t:Hide() end
		glow.proc = Style.ProcGlow(b, 6, 6)
	end
	glow:Hide()
	return glow
end

function Kit.ShowGlow(glow, on)
	glow:SetShown(on and true or false)
	if glow.proc then Style.ShowGlow(glow.proc, on and true or false) end
end

-- The step's dots on a frame of their own, which the caller places: one diamond a step, gold for this one, darker gold
-- for those behind it, iron for those ahead; dots:Set(at, of) draws them again (none under two steps). Beside words
-- (the pump card's intro) they sit 12 after them; in a bar's middle, on its buttons' middle line (Kit.Footer).
local DOT, DOT_GAP = 6, 8

local function setDots(d, at, of)
	d:SetShown(of > 1)
	d:SetSize(math.max(1, of * DOT + (of - 1) * DOT_GAP), DOT)
	for i = 1, math.max(of, #d.dots) do
		local dot = d.dots[i]
		if not dot and i <= of then
			dot = d:CreateTexture(nil, "ARTWORK")
			dot:SetSize(DOT, DOT)
			dot:SetRotation(math.pi / 4)
			d.dots[i] = dot
		end
		if dot then
			dot:SetShown(i <= of)
			dot:ClearAllPoints()
			dot:SetPoint("BOTTOMLEFT", (i - 1) * (DOT + DOT_GAP), 0)
			local color = i == at and { NORMAL_FONT_COLOR:GetRGB() } or i < at and saga.dotBehind or saga.dotAhead
			dot:SetColorTexture(color[1], color[2], color[3], 1)
		end
	end
end

function Kit.Dots(parent, at, of)
	local d = CreateFrame("Frame", nil, parent)
	d.dots, d.Set = {}, setDots
	d:Set(at or 0, of or 0)
	return d
end

-- The bottom bar of a window (Kit.Footer): its rule (where the tier has one), grey ways back or out from its left, the
-- red way on at its right, the step's dots or a control in its middle (a red message over the first secondary until
-- 2026-10-22: a refusal now stands in the window's body). Its parts show and hide with the window's state; foot:Lay()
-- places them and answers the bar's height (Kit.Fit calls it). Where the secondaries that show meet the primary, each
-- shrinks to its words + 24; where they still meet, those marked `rare`
-- fold into one grey "More" with the menu's arrow (Kit.Caret) that opens them in QuestPump's menu (a way out of a pump
-- never folds: Bank it, Bank the rest, Skip and Pause are never rare).
local Footer = {}

-- A grey secondary at the bar's left, `width` units at its least (the tier's least without) or its words + 40, the
-- larger (a longer label never runs past its art); `rare` lets it fold.
function Footer:Secondary(label, onClick, width, rare)
	local t = self.tier
	local b = Kit.Secondary(self, label, width or t.BACK, onClick, t.BUTTON)
	b:SetPoint("BOTTOMLEFT", t.BAR or t.IN, t.BOTTOM)
	b.least, b.rare, b.folded = width or t.BACK, rare == true, false
	self:Label(b, label)
	-- a hide or a show of the window's own clears the fold's mark, so the fold never shows what the window hid
	hooksecurefunc(b, "Hide", function() b.folded = false end)
	hooksecurefunc(b, "SetShown", function() b.folded = false end)
	self.left[#self.left + 1] = b
	return b
end

function Footer:Primary(label, onClick, sound)
	local t = self.tier
	local b = Kit.Primary(self, label, t.PRIMARY, onClick, sound)
	b:SetHeight(t.BUTTON)
	b:SetPoint("BOTTOMRIGHT", -(t.BAR or t.IN), t.BOTTOM)
	self.right[#self.right + 1] = b
	return b
end

-- New words on a secondary of the bar (the setup's Play without, named by what the player goes without), its width
-- from them as Footer:Secondary sets it.
function Footer:Label(b, label)
	b:SetText(label)
	b.barWidth = math.max(b.least, b:GetTextWidth() + 40)
	b:SetWidth(b.barWidth)
end

-- The step's dots in the bar's middle, shown from two steps on.
function Footer:Step(at, of)
	self.dots:Set(at, of)
end

-- A control in the bar's middle (the pump card's guide on its countdown and Running views), centred in the room
-- between the secondaries' end and the primary's start (or the bar's right end without one); the fit rule leaves it
-- its width while it shows.
function Footer:Middle(frame)
	frame:SetParent(self)
	self.middle = frame
end

-- The secondaries the window shows, folded back in first, at their least widths; then the fit rule.
local function fitLeft(self, room)
	local shown = {}
	for _, b in ipairs(self.left) do
		if b.folded then
			b:Show()
			b.folded = false
		end
		b:SetWidth(b.barWidth)
		if b:IsShown() then shown[#shown + 1] = b end
	end
	local function need(list)
		local n = -8
		for _, b in ipairs(list) do n = n + b:GetWidth() + 8 end
		return n
	end
	if need(shown) > room then
		for _, b in ipairs(shown) do b:SetWidth(math.min(b.barWidth, b:GetTextWidth() + 24)) end
	end
	local folded = {}
	if need(shown) > room then
		local kept = {}
		for _, b in ipairs(shown) do
			if b.rare then
				b:Hide()
				b.folded = true
				folded[#folded + 1] = b
			else
				kept[#kept + 1] = b
			end
		end
		shown = kept
	end
	self.more.folded = folded
	self.more:SetShown(#folded > 0)
	if #folded > 0 then shown[#shown + 1] = self.more end
	return shown
end

-- The bar laid out: the secondaries from the left 8 apart (the fit rule), the red message over them; the rule over it
-- all. Answers the bar's height.
function Footer:Lay()
	local t = self.tier
	local primary = 0
	for _, b in ipairs(self.right) do
		if b:IsShown() then primary = math.max(primary, b:GetWidth()) end
	end
	local x = t.BAR or t.IN
	local middle = self.middle and self.middle:IsShown() and self.middle or nil
	local right = t.W - x - primary - (primary > 0 and 8 or 0) -- where the primary's room begins
	for _, b in ipairs(fitLeft(self, right - x - (middle and middle:GetWidth() + 8 or 0))) do
		b:ClearAllPoints()
		b:SetPoint("BOTTOMLEFT", x, t.BOTTOM)
		x = x + b:GetWidth() + 8
	end
	if middle then
		middle:ClearAllPoints()
		middle:SetPoint("BOTTOM", self, "BOTTOMLEFT", (x + right) / 2, t.BOTTOM + (t.BUTTON - middle:GetHeight()) / 2)
	end
	if self.rule then
		self.rule:ClearAllPoints()
		self.rule:SetPoint("BOTTOM", 0, t.FOOT)
	end
	return t.FOOT
end

-- A window's bottom bar, on a frame over the window (win.tier's measures).
function Kit.Footer(win)
	local t = win.tier
	local foot = CreateFrame("Frame", nil, win)
	foot:SetAllPoints()
	foot.tier, foot.left, foot.right = t, {}, {}
	foot.rule = t.FOOT > 0 and Kit.HRule(foot, t.TEXT) or false
	foot.dots = Kit.Dots(foot)
	foot.dots:SetPoint("BOTTOM", 0, t.BOTTOM + (t.BUTTON - DOT) / 2)
	for name, fn in pairs(Footer) do foot[name] = fn end
	foot.more = Kit.Secondary(foot, L["ui.more"], t.BACK, function(self)
		Kit.OpenMenu(self, function(_, root)
			for _, b in ipairs(self.folded) do
				root:CreateButton(b:GetText(), function() b.kitClick(b) end):SetTooltip(rawget(b, "kitTip"))
			end
		end, true)
	end, t.BUTTON)
	Kit.Hover(foot.more, Kit.Tip(L["ui.more"], nil, { hint = L["ui.more.click"] }), "ANCHOR_TOP")
	-- its least, which the arrow widens to its words and the arrow + 40, as a secondary's (Footer:Secondary)
	foot.more.barWidth = t.BACK
	Kit.Caret(foot.more)
	foot.more.folded = {}
	foot.more:Hide()
	return foot
end

-- A window as tall as its content's `bottom` and its bar (`foot`, Kit.Footer's, laid out here), at least its tier's
-- height; then by its fit (its tier's, or Kit.Window's `fit`): the large one scales down to stay inside the screen
-- where a large Text Size outgrows it; the medium one past its cap (or the screen less its dim's edge above and below)
-- scales down to minScale at most (its view folds first, the caller's); the card stops at the screen less its room,
-- its content scrolling past it (the caller's scroll list, sized from the answer); a "fixed" one stays at scale 1.
-- Every render of a kit window ends here. Answers the height.
function Kit.Fit(win, bottom, foot)
	local t, fit = win.tier, win.fit
	local height = math.ceil(math.max(t.H or t.least or 0, bottom + 20 + (foot and foot:Lay() or 0)))
	if fit == "clamp" then height = math.min(height, math.floor(UIParent:GetHeight() - t.room)) end
	win:SetHeight(height)
	if fit == "screen" then
		win:SetScale(math.min(1, (UIParent:GetHeight() - 2 * Kit.MARGIN) / height))
	elseif fit == "cap" then
		local limit = math.min(t.cap, UIParent:GetHeight() - 2 * t.edge)
		win:SetScale(height > limit and math.max(t.minScale, limit / height) or 1)
	end
	return height
end

-- Controls ------------------------------------------------------------------------------------------------------------

-- A check box with its label (the feedback window's "Add my numbers"): a button as wide as its words, the box at its
-- left and the label (bodyWhite) 6 after it, so a click anywhere on the words ticks it; the label stays white and the
-- tick shows the state (G4). set(on, cb) takes a click (the cue plays); cb:SetChecked and cb:GetChecked as a check
-- button's.
local CHECK_BOX, CHECK_GAP = 22, 6

local function checkSetChecked(cb, on)
	cb.checked = on and true or false
	Kit.BoxState(cb.box, cb.checked)
end

local function checkGetChecked(cb)
	return cb.checked
end

function Kit.Check(parent, label, set)
	local cb = CreateFrame("Button", nil, parent)
	cb.checked = false
	cb.box = Kit.Box(cb, CHECK_BOX)
	cb.box:SetPoint("LEFT")
	cb.label = Kit.Text(cb, "bodyWhite", "LEFT")
	cb.label:SetPoint("LEFT", cb.box, "RIGHT", CHECK_GAP, 0)
	cb.label:SetText(label)
	local hover = cb:CreateTexture(nil, "HIGHLIGHT")
	hover:SetAllPoints()
	hover:SetColorTexture(unpack(saga.hover))
	cb.SetChecked, cb.GetChecked = checkSetChecked, checkGetChecked
	cb:SetScript("OnClick", function(self)
		local on = not self.checked
		Style.Play(on and "checkOn" or "checkOff")
		self:SetChecked(on)
		set(on, self)
	end)
	cb:SetSize(CHECK_BOX + CHECK_GAP + cb.label:GetStringWidth(), math.max(CHECK_BOX, cb.label:GetStringHeight()))
	cb:SetChecked(false)
	return cb
end

-- The dark select (a value that rarely changes, or more than four choices): a well 26 high (a picture's ground, the
-- list's edge) with the shown words (label(): the value, or a prompt such as "Pick a ladder") and the dropdown's arrow
-- at its right; a click opens QuestPump's own menu under it (Kit.OpenMenu: generator(owner, root)), never Blizzard's
-- dropdown; `tip`, a Kit.Tip shown under the cursor. b.Refresh() writes label() again (the menu calls it after a
-- choice); b:SetEnabled greys it.
local function selectArrow(b)
	local key = not b:IsEnabled() and "selectArrowOff" or ns.Menu.owner == b and "selectArrowOpen"
		or b:IsMouseOver() and "selectArrowHover" or "selectArrow"
	if not Style.SetAtlas(b.arrow, key) then
		b.arrow:SetTexture(Style.menuArrow)
		b.arrow:SetRotation(-math.pi / 2)
	end
end

function Kit.Select(parent, w, label, generator, tip)
	local b = Style.Box(parent, "Button") -- the well is the button's own, under its words, arrow and highlight
	b:SetSize(w, Kit.CONTROL_H)
	b:SetMotionScriptsWhileDisabled(true)
	b:Paint(saga.screen, saga.listEdge)
	b.text = Kit.Text(b, "bodyWhite", "LEFT")
	b.text:SetPoint("LEFT", 10, 0)
	b.text:SetPoint("RIGHT", -30, 0)
	b.text:SetWordWrap(false)
	b.arrow = b:CreateTexture(nil, "ARTWORK")
	b.arrow:SetSize(20, 20)
	b.arrow:SetPoint("RIGHT", -3, 0)
	local hover = b:CreateTexture(nil, "HIGHLIGHT")
	hover:SetAllPoints()
	hover:SetColorTexture(unpack(saga.hover))
	local enable = b.SetEnabled
	function b.SetEnabled(self, on)
		enable(self, on)
		self.text:SetTextColor(((on and true or false) and HIGHLIGHT_FONT_COLOR or DISABLED_FONT_COLOR):GetRGB())
		selectArrow(self)
	end
	b:SetScript("OnClick", function(self) Kit.OpenMenu(self, generator) selectArrow(self) end)
	b:HookScript("OnEnter", function(self)
		selectArrow(self)
		if not tip then return end
		GameTooltip:SetOwner(self, "ANCHOR_RIGHT")
		tip(GameTooltip)
		GameTooltip:Show()
	end)
	b:HookScript("OnLeave", function(self)
		selectArrow(self)
		if tip then GameTooltip:Hide() end
	end)
	function b.Refresh() b.text:SetText(label()) end
	function b.OnMenuClosed() selectArrow(b) end
	b.Refresh()
	selectArrow(b)
	return b
end

-- The segmented switch (two to four modes of one view, in place of a row of red buttons): one box 26 high (the list's
-- ground and edge) of equal cells, a word each; the chosen cell with the gold light, a gold inset edge and its word in
-- gold, the rest in the notes' grey. options: { value, words } pairs; get() answers the chosen value, set(value) takes
-- a click (the tab's cue; the chosen cell clicked again does nothing); `tip`, a Kit.Tip on every cell. seg:Refresh()
-- draws the choice.
local function segRefresh(seg)
	local chosen = seg.get()
	for _, cell in ipairs(seg.cells) do
		local on = cell.value == chosen
		cell.light:SetShown(on)
		cell.inset:SetShown(on)
		local color = on and { NORMAL_FONT_COLOR:GetRGB() } or saga.note
		cell.words:SetTextColor(color[1], color[2], color[3])
	end
end

function Kit.Segments(parent, w, options, get, set, tip)
	local seg = Style.Box(parent)
	seg:SetSize(w, Kit.CONTROL_H)
	seg:Paint(saga.list, saga.listEdge)
	seg.get, seg.cells = get, {}
	seg.Refresh = segRefresh
	local cw = (w - 2) / #options
	local gold = saga.chosen
	for i, option in ipairs(options) do
		local cell = CreateFrame("Button", nil, seg)
		cell:SetPoint("TOPLEFT", 1 + (i - 1) * cw, -1)
		cell:SetSize(cw, Kit.CONTROL_H - 2)
		cell.value = option[1]
		cell.light = cell:CreateTexture(nil, "BACKGROUND", nil, 1)
		cell.light:SetAllPoints()
		cell.light:SetColorTexture(1, 1, 1, 1)
		cell.light:SetGradient("HORIZONTAL", CreateColor(gold[1], gold[2], gold[3], gold[4]),
			CreateColor(gold[1], gold[2], gold[3], 0))
		cell.inset = Style.Box(cell)
		cell.inset:SetAllPoints()
		cell.inset:Paint(nil, saga.chosenEdge)
		cell.words = Kit.Text(cell, "labelWhite", "CENTER")
		cell.words:SetPoint("CENTER")
		cell.words:SetText(option[2])
		local hover = cell:CreateTexture(nil, "HIGHLIGHT")
		hover:SetAllPoints()
		hover:SetColorTexture(unpack(saga.hover))
		if i > 1 then
			local rule = Kit.Rule(seg, saga.listRule, true)
			rule:SetPoint("TOPLEFT", 1 + (i - 1) * cw, -5)
			rule:SetPoint("BOTTOMLEFT", 1 + (i - 1) * cw, 5)
		end
		cell:SetScript("OnClick", function(self)
			if seg.get() == self.value then return end
			Style.Play("tab")
			set(self.value)
			seg:Refresh()
		end)
		if tip then Kit.Hover(cell, tip) end
		seg.cells[i] = cell
	end
	seg:Refresh()
	return seg
end

-- The More section, the one second level of disclosure (never nested): a bar 26 high on the Settings panel's
-- expandable section's art (its open end while open) with its words (label, "More" without), and under it the content
-- build(content) made at its first opening, shown only while open; a click plays
-- the tab's cue and calls more.onToggle(open), which lays the page out again (more.content is the page's to place,
-- Kit.Stack's { more.content, gap, h, show = more:IsOpen() }). more:SetOpen(open) opens or closes it without a click.
local function moreArt(more)
	local open = more.open
	if not Style.SetAtlas(more.rightEnd, open and "expandOpen" or "expandRight") then
		more.rightEnd:SetTexture(Style.menuArrow)
		more.rightEnd:SetRotation(open and math.pi / 2 or -math.pi / 2)
	end
end

local function moreSetOpen(more, open)
	more.open = open and true or false
	if more.open and not more.built then
		more.built = true
		more.build(more.content)
	end
	more.content:SetShown(more.open)
	moreArt(more)
end

local function moreIsOpen(more)
	return more.open
end

function Kit.More(parent, w, label, build)
	local more = CreateFrame("Button", nil, parent)
	more:SetSize(w, Kit.CONTROL_H)
	more.open, more.built, more.build = false, false, build
	more.left = more:CreateTexture(nil, "BACKGROUND")
	more.left:SetSize(12, Kit.CONTROL_H)
	more.left:SetPoint("LEFT")
	more.rightEnd = more:CreateTexture(nil, "BACKGROUND")
	more.rightEnd:SetSize(28, Kit.CONTROL_H)
	more.rightEnd:SetPoint("RIGHT")
	more.mid = more:CreateTexture(nil, "BACKGROUND")
	more.mid:SetPoint("TOPLEFT", more.left, "TOPRIGHT")
	more.mid:SetPoint("BOTTOMRIGHT", more.rightEnd, "BOTTOMLEFT")
	if not (Style.SetAtlas(more.left, "expandLeft") and Style.SetAtlas(more.mid, "expandMid")) then
		more.left:SetColorTexture(unpack(saga.cell))
		more.mid:SetColorTexture(unpack(saga.cell))
	end
	more.words = Kit.Text(more, "labelWhite", "LEFT")
	more.words:SetPoint("LEFT", 12, 0)
	more.words:SetText(label or L["ui.more"])
	local hover = more:CreateTexture(nil, "HIGHLIGHT")
	hover:SetAllPoints()
	hover:SetColorTexture(unpack(saga.hover))
	more.content = CreateFrame("Frame", nil, parent)
	more.content:SetSize(w, 1)
	more.content:Hide()
	more.SetOpen, more.IsOpen = moreSetOpen, moreIsOpen
	more:SetScript("OnClick", function(self)
		Style.Play("tab")
		self:SetOpen(not self.open)
		if self.onToggle then self.onToggle(self.open) end
	end)
	moreArt(more)
	return more
end

-- The search box (SearchBoxTemplate, 22 high) with its grey placeholder, which calls onChange(text) once a frame at
-- most however fast the player types.
local searches = 0

function Kit.Search(parent, w, placeholder, onChange)
	local box = CreateFrame("EditBox", nil, parent, "SearchBoxTemplate")
	box:SetSize(w, 22)
	box:SetAutoFocus(false)
	if box.Instructions then box.Instructions:SetText(placeholder) end
	searches = searches + 1
	local name = "search" .. searches
	box:HookScript("OnTextChanged", function(self)
		T.after(name, 0, function() onChange(self:GetText() or "") end)
	end)
	return box
end

-- A link chip (the Talents board's chips into another tree, a card's "This step opens" rows): a button 22 high on a
-- cell's ground and edge with its words (labelWhite) and the row's chevron 8 in from its right, the light under the
-- cursor; never red. onClick(chip) after the tab's cue. Words its caller cuts to a width (the client's ellipsis) show
-- whole in its tooltip.
local function chipTip(chip)
	cutTip(chip, chip.words)
end

function Kit.LinkChip(parent, words, onClick)
	local chip = Style.Box(parent, "Button") -- its ground and edge its own, under its words and highlight
	chip:Paint(saga.cell, saga.edge)
	chip.words = Kit.Text(chip, "labelWhite", "LEFT")
	chip.words:SetPoint("LEFT", 8, 0)
	chip.words:SetText(words)
	chip.chevron = chip:CreateTexture(nil, "ARTWORK")
	chip.chevron:SetSize(16, 16)
	chip.chevron:SetPoint("RIGHT", -8, 0)
	chip.chevron:SetTexture(Style.menuArrow)
	local hover = chip:CreateTexture(nil, "HIGHLIGHT")
	hover:SetAllPoints()
	hover:SetColorTexture(unpack(saga.hover))
	chip:SetSize(8 + chip.words:GetStringWidth() + 6 + 16 + 8, 22)
	chip:SetScript("OnClick", function(self)
		Style.Play("tab")
		onClick(self)
	end)
	chip:SetScript("OnEnter", chipTip)
	chip:SetScript("OnLeave", function(self)
		if GameTooltip:IsOwned(self) then GameTooltip:Hide() end
	end)
	return chip
end

-- The scroll list: a scroll frame (ScrollFrameTemplate with its thin MinimalScrollBar, the Settings panel's) and its
-- content, holding real rows (pooled by the caller, built when needed), vertical only, a wheel step of Kit.WHEEL, the
-- bar hidden while everything fits. Never a ScrollBox: ConsolePort's cursor and the ElvUI walk do not follow its
-- recycled frames. Its content passes the cursor by to its rows (`nodepass`, ConsolePortNode). Without `w` and `h` the
-- caller anchors it, and the content takes its width.
function Kit.Scroll(parent, w, h)
	local scroll = CreateFrame("ScrollFrame", nil, parent, "ScrollFrameTemplate")
	if w and h then scroll:SetSize(w, h) end
	if scroll.SetPanExtent then scroll:SetPanExtent(Kit.WHEEL) end
	if scroll.ScrollBar and scroll.ScrollBar.SetHideIfUnscrollable then scroll.ScrollBar:SetHideIfUnscrollable(true) end
	local content = CreateFrame("Frame", nil, scroll)
	content:SetSize(w or 1, 1)
	content:SetAttribute("nodepass", true)
	scroll:SetScrollChild(content)
	scroll:HookScript("OnSizeChanged", function(_, sw) content:SetWidth(sw) end)
	scroll.content = content
	return scroll
end

-- The empty state, never red and never a loss: a dimmed picture in a 120 x 80 screen (a move's key pose, `move`, or
-- QuestPump's own mark), its name (cardTitle, white: "No pumps yet") and at most one grey button (opts.button: {
-- label, onClick }), centred `w` units across. e:Set(name) changes the name; e:Lay() places them and answers the
-- height.
local function emptySet(e, name)
	e.name:SetText(name)
end

local function emptyLay(e)
	local y = 80 + 12
	e.name:ClearAllPoints()
	e.name:SetPoint("TOP", 0, -y)
	y = y + e.name:GetStringHeight()
	if e.button then
		e.button:ClearAllPoints()
		e.button:SetPoint("TOP", 0, -(y + 12))
		y = y + 12 + e.button:GetHeight()
	end
	e:SetHeight(y)
	return y
end

function Kit.Empty(parent, w, opts)
	opts = opts or {}
	local e = CreateFrame("Frame", nil, parent)
	e:SetWidth(w)
	e.screen = Kit.Screen(e, 120, 80)
	e.screen:SetPoint("TOP")
	local pic = e.screen:CreateTexture(nil, "ARTWORK")
	pic:SetSize(Style.iconPx, Style.iconPx)
	pic:SetPoint("CENTER")
	if opts.move then Kit.SetIcon(pic, opts.move) else pic:SetTexture(Style.mark) end
	pic:SetDesaturated(true)
	pic:SetAlpha(0.5)
	e.name = Kit.Text(e, "cardTitle", "CENTER", nil, w)
	e.button = false
	if opts.button then e.button = Kit.Secondary(e, opts.button[1], 1, opts.button[2], 22) end
	if e.button then e.button:SetWidth(math.max(100, e.button:GetTextWidth() + 40)) end
	e.Set, e.Lay = emptySet, emptyLay
	e:Set(opts.name or "")
	e:Lay()
	return e
end

-- Motion --------------------------------------------------------------------------------------------------------------

-- The moving parts every window shares (slice 72, the feel pass), on Style.motion's times. Each shows its end at once
-- with the game's reduced motion (Tree.Moving: the General page's "Animate the move figure" off; Tree.lua loads after
-- the kit, so it is read at call time). Alpha never gates a frame's mouse or keys, so nothing waits on a fade.

-- An Alpha animation in `ag` on `target`, from `from` to `to` over `duration` s in its `order`.
function Kit.Alpha(ag, target, from, to, duration, order)
	local a = ag:CreateAnimation("Alpha")
	a:SetTarget(target)
	a:SetFromAlpha(from)
	a:SetToAlpha(to)
	a:SetDuration(duration)
	a:SetOrder(order)
	return a
end

-- An entrance of `region` (its own group): hidden for `delay` s, then over `dur` s from alpha 0, (dx, dy) away and
-- `scale` of its size to its place, where it stays; ag.hold is the wait, whose length a caller may change.
function Kit.Entrance(region, delay, dur, dx, dy, scale)
	local ag = region:CreateAnimationGroup()
	ag.hold = Kit.Alpha(ag, region, 0, 0, delay, 1)
	if dx or dy then
		local away = ag:CreateAnimation("Translation")
		away:SetTarget(region)
		away:SetOffset(dx or 0, dy or 0)
		away:SetDuration(0)
		away:SetOrder(1)
		local back = ag:CreateAnimation("Translation")
		back:SetTarget(region)
		back:SetOffset(-(dx or 0), -(dy or 0))
		back:SetDuration(dur)
		back:SetOrder(2)
		back:SetSmoothing("OUT")
	end
	if scale then
		local grow = ag:CreateAnimation("Scale")
		grow:SetTarget(region)
		grow:SetScaleFrom(scale, scale)
		grow:SetScaleTo(1, 1)
		grow:SetDuration(dur)
		grow:SetOrder(2)
		grow:SetSmoothing("OUT")
	end
	Kit.Alpha(ag, region, 0, 1, dur, 2):SetSmoothing("OUT")
	ag:SetToFinalAlpha(true)
	return ag
end

-- A frame that comes by itself (the pump's dim with its card, the "Stop to pump" card): shown, and faded in over
-- Style.motion.appear with all it holds; one that shows already stays as it is. Answers whether it came. Kit.Leave
-- hides it, its fade stopped with it (what the client does with a hidden group mid-play is undocumented).
function Kit.Appear(frame)
	if frame:IsShown() then return false end
	frame:Show()
	if not ns.Tree.Moving() then return true end
	if not frame.kitAppear then
		frame.kitAppear = frame:CreateAnimationGroup()
		Kit.Alpha(frame.kitAppear, frame, 0, 1, Style.motion.appear, 1):SetSmoothing("OUT")
	end
	frame.kitAppear:Play()
	return true
end

function Kit.Leave(frame)
	if frame.kitAppear then frame.kitAppear:Stop() end
	frame:Hide()
end

-- A number or a word that just changed pops once: from Style.motion.popFrom of its size to its own over
-- Style.motion.pop (the apps' digitPop). A font string scales by its vertices, as Blizzard's warnings do, and from
-- the side its words stand at (a left-aligned line in a wide row would slide in from the row's middle otherwise).
function Kit.Pop(region)
	if not ns.Tree.Moving() then return end
	local ag = region.kitPop
	if not ag then
		ag = region:CreateAnimationGroup()
		local grow = ag:CreateAnimation("Scale")
		grow:SetScaleFrom(Style.motion.popFrom, Style.motion.popFrom)
		grow:SetScaleTo(1, 1)
		grow:SetDuration(Style.motion.pop)
		grow:SetSmoothing("OUT")
		if region:GetObjectType() == "FontString" then
			region:SetScaleAnimationMode(Enum.FontStringScaleAnimationMode.Vertex)
			grow:SetOrigin(region:GetJustifyH(), 0, 0)
		end
		region.kitPop = ag
	end
	ag:Stop()
	ag:Play()
end

-- A value carried home: step(k) with k from 0 to 1, slowing into its end, over Style.motion.carry, then done(). It
-- runs on a ticker of its own under `name` only while the value visibly counts or fills (one carry a name: a new one,
-- a Kit.CarryStop or a Kit.CarryAfter takes its place); with the game's reduced motion step(1) and done() at once.
function Kit.Carry(name, step, done)
	local key, m = "carry:" .. name, Style.motion
	T.cancel(key)
	if not ns.Tree.Moving() then
		step(1)
		if done then done() end
		return
	end
	local from = GetTime()
	step(0)
	T.every(key, m.tick, function()
		local k = math.min(1, (GetTime() - from) / m.carry)
		if k >= 1 then T.cancel(key) end
		step(1 - (1 - k) * (1 - k))
		if k >= 1 and done then done() end
	end)
end

function Kit.CarryStop(name)
	T.cancel("carry:" .. name)
end

-- The wait between two parts of one carry (a bar full at a level's turn before it starts the next level), under the
-- carry's own name, so whatever stops the carry stops the wait.
function Kit.CarryAfter(name, seconds, fn)
	T.after("carry:" .. name, seconds, fn)
end

-- The XP bar (the sheet's, the pump banner's, the HUD's): Forever's own bar, its track and silver capsule, the XP fill
-- over the rested fill (a plain backing and fill on a client without the art), `w` by `h`; `thin` a bare line of it,
-- no capsule and no words. bar.words(level, into, need, paragon), the caller's, gives its words. It is placed by
-- bar.rested, the track every other part hangs on. `name` is its carry's (Kit.Carry).
--   bar:SetXP(xp, rested)   a lifetime XP at once: its share of its level (ns.LevelOf), the rested XP's share under it
--   bar:FillXP(from, to, rested)   carried from lifetime XP `from` to `to`: the fill runs to its new share, and
--     through a level or a Paragon rank to the bar's end first, where it flashes once and the new level starts from
--     empty and fills to its share with its words popping in (never one jump, never backwards: a gain past several
--     levels turns once, into the last); no gain, or the game's reduced motion: SetXP(to).
local function xpWords(bar, level, into, need, paragon)
	if bar.words then bar.text:SetText(bar.words(level, math.floor(into + 0.5), need, paragon)) end
end

local function setXP(bar, xp, rested)
	Kit.CarryStop(bar.carry)
	local level, into, need, paragon = ns.LevelOf(xp)
	bar.xp = xp
	bar:SetMinMaxValues(0, need)
	bar:SetValue(into)
	bar.rested:SetMinMaxValues(0, need)
	bar.rested:SetValue(math.min(need, into + (rested or 0)))
	xpWords(bar, level, into, need, paragon)
end

local function fillXP(bar, from, to, rested)
	if to <= from or not ns.Tree.Moving() then return setXP(bar, to, rested) end
	local level, into, need, paragon = ns.LevelOf(from)
	local level2, into2, need2, paragon2 = ns.LevelOf(to)
	local turns = level2 ~= level or paragon2 ~= paragon
	local ends = turns and need or into2
	bar.xp = to
	bar:SetMinMaxValues(0, need)
	Kit.Carry(bar.carry, function(k)
		local v = into + (ends - into) * k
		bar:SetValue(v)
		xpWords(bar, level, v, need, paragon)
	end, function()
		if not turns then return setXP(bar, to, rested) end
		bar.flashAnim:Stop()
		bar.flashAnim:Play()
		Kit.CarryAfter(bar.carry, Style.motion.turn, function()
			bar:SetMinMaxValues(0, need2)
			bar.rested:SetValue(0) -- the new level's rested share comes with the end (setXP)
			if bar.text then Kit.Pop(bar.text) end
			Kit.Carry(bar.carry, function(k)
				bar:SetValue(into2 * k)
				xpWords(bar, level2, into2 * k, need2, paragon2)
			end, function() setXP(bar, to, rested) end)
		end)
	end)
end

function Kit.XPBar(parent, w, h, name, thin)
	local rested = CreateFrame("StatusBar", nil, parent)
	rested:SetSize(w, h)
	rested.bg = rested:CreateTexture(nil, "BACKGROUND")
	rested.bg:SetAllPoints()
	if thin or not Style.SetAtlas(rested.bg, "xpBackground") then rested.bg:SetColorTexture(0, 0, 0, 0.5) end
	local restedFill = rested:CreateTexture(nil, "ARTWORK")
	if not Style.SetAtlas(restedFill, "xpRested") then restedFill:SetColorTexture(0, 0.4, 0.8, 0.6) end
	rested:SetStatusBarTexture(restedFill)
	local bar = CreateFrame("StatusBar", nil, parent)
	bar:SetAllPoints(rested)
	bar:SetFrameLevel(rested:GetFrameLevel() + 1)
	local fill = bar:CreateTexture(nil, "ARTWORK")
	if not Style.SetAtlas(fill, "xpFill") and not Style.SetAtlas(fill, "barFill") then
		fill:SetColorTexture(0.6, 0, 0.8)
	end
	bar:SetStatusBarTexture(fill)
	-- set, never left nil: the tools' stand-in frames answer a missing field with a truthy object
	bar.xp, bar.words, bar.text = false, false, false
	if not thin then
		bar.frame = bar:CreateTexture(nil, "ARTWORK", nil, 1)
		bar.frame:SetAllPoints()
		Style.SetAtlas(bar.frame, "xpFrame")
		bar.text = Kit.Text(bar, "labelWhite", "CENTER") -- a fixed size: the bar's art has a fixed height
		bar.text:SetPoint("CENTER")
	end
	-- the flash a level's turn gives the full bar: white over the fill, up and away
	bar.flash = bar:CreateTexture(nil, "OVERLAY")
	bar.flash:SetAllPoints()
	bar.flash:SetBlendMode("ADD")
	bar.flash:SetColorTexture(1, 1, 1, 1)
	bar.flash:SetAlpha(0)
	bar.flashAnim = bar.flash:CreateAnimationGroup()
	Kit.Alpha(bar.flashAnim, bar.flash, 0, 0.8, 0.1, 1)
	Kit.Alpha(bar.flashAnim, bar.flash, 0.8, 0, Style.motion.turn + 0.2, 2)
	bar.rested, bar.carry = rested, name
	bar.SetXP, bar.FillXP = setXP, fillXP
	return bar
end

-- Layout --------------------------------------------------------------------------------------------------------------

-- Lays parts out from `y` down and answers the bottom: each { node, gap above it, height } is a font string (its own
-- height), a frame (its own height) or a part of a given height (a rule, a grid), at the view's inset (plus the part's
-- `x`), or at its parent's top left without a view. A part with `show` set shows or hides by it, and a hidden one takes
-- no space (Spoken's ShowWhen); a part without it is placed as it stands.
function Kit.Stack(view, y, parts)
	for _, part in ipairs(parts) do
		local node, gap, fixed = part[1], part[2], part[3]
		if part.show ~= nil then node:SetShown(part.show) end
		if part.show ~= false then
			local h = fixed or (node:GetObjectType() == "FontString" and node:GetStringHeight() or node:GetHeight())
			node:ClearAllPoints()
			if view then
				node:SetPoint("TOPLEFT", view, "TOPLEFT", (view.inset or 0) + (part.x or 0), -(y + gap))
			else
				node:SetPoint("TOPLEFT", part.x or 0, -(y + gap))
			end
			y = y + gap + h
		end
	end
	return y
end

-- A Text Size or UI scale change while a window shows: its words change size at once, but every height the kit
-- measured is the old one until the window draws again. Each window registers its draw (Kit.OnRelayout(fn), which
-- draws only what shows), and the kit calls them all, each under ns.Guarded, once a frame at most after
-- UI_SCALE_CHANGED, DISPLAY_SIZE_CHANGED or the client's Text Size change: Blizzard's TextSizeManager sets the
-- user-scaled fonts' heights from the userFontScale CVar and then triggers "TextSizeManager.OnTextScaleUpdated" on
-- EventRegistry, whoever changed the CVar (Gethe `forever` Blizzard_AccessibilityTemplates/TextSizeManager.lua,
-- 1.60.1 70425, read 2026-10-16).
local relayouts = {}

function Kit.OnRelayout(fn)
	relayouts[#relayouts + 1] = fn
end

local function relayout()
	T.after("relayout", 0, function()
		for _, fn in ipairs(relayouts) do
			local ok, err = ns.Guarded(fn)
			if not ok then ns.Report(err) end
		end
	end)
end

ns.RegisterEvent("UI_SCALE_CHANGED", relayout)
ns.RegisterEvent("DISPLAY_SIZE_CHANGED", relayout)
EventRegistry:RegisterCallback("TextSizeManager.OnTextScaleUpdated", relayout, Kit)
