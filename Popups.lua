-- QuestPump's own menus and dialogs, drawn on frames the addon owns. Never Blizzard's shared systems (Menu.ModifyMenu,
-- MenuUtil and DropdownButton menus, StaticPopup, HelpTip): addon code that runs inside them taints the frames and
-- tables Blizzard reuses, and a protected action that reads them later is blocked in QuestPump's name, behind
-- Blizzard's "QuestPump has been blocked from an action" popup. On 2026-10-05 removing a Battle.net friend through the
-- menu QuestPump had added entries to fired ADDON_ACTION_FORBIDDEN for C_BattleNet.SearchFriends(); Blizzard's open
-- reports: WoWUIBugs #816 and #826 (an entry added to a menu breaks its protected entries), #783 and #637 (an addon's
-- context menu taints the menu frames Blizzard reuses). The look stays Blizzard's: the tooltip backdrop, the menu's
-- highlight, divider, arrow and radio art and the dialog border (Gethe forever 70235: Blizzard_Menu MenuVariants.lua,
-- Blizzard_SharedXML DialogTemplates.xml).

local ADDON, ns = ...
local Kit, Style = ns.Kit, ns.Style
local T = ns.Timers()

-- Menus -------------------------------------------------------------------------------------------------------------

-- A menu as its opener describes it, in the part of Blizzard's menu description the addon uses, so an opener reads as
-- it did: CreateButton (an action, or a submenu when entries are created under it), CreateTitle, CreateDivider and
-- CreateRadio (isSelected is asked as the menu draws); an entry's SetTooltip(tip) gives it a tooltip in the kit's shape
-- (a Kit.Tip), shown at its right while the cursor is on it.
local Entry = {}
Entry.__index = Entry

local function entry(kind, text, fn)
	return setmetatable({ kind = kind, text = text, fn = fn, items = {} }, Entry)
end

local function add(parent, e)
	parent.items[#parent.items + 1] = e
	return e
end

function Entry:CreateButton(text, fn) return add(self, entry("button", text, fn)) end
function Entry:CreateTitle(text) return add(self, entry("title", text)) end
function Entry:CreateDivider() return add(self, entry("divider")) end

function Entry:CreateRadio(text, isSelected, onSelect)
	local e = add(self, entry("radio", text, onSelect))
	e.isSelected = isSelected
	return e
end

function Entry:SetTooltip(tip)
	self.tooltip = tip
	return self
end

local Menu = { levels = {}, base = 9500 }
ns.Menu = Menu

-- Blizzard's menu metrics: 20-unit rows, a 13-unit divider (MenuVariants.lua), the tooltip box's padding around them.
local ROW_H, DIVIDER_H, PAD_X, PAD_Y, MIN_W, ICON = 20, 13, 10, 8, 100, 16

-- One frame per open level (the menu, its submenu, a submenu of that), above the pump's dim, which can sit at
-- FULLSCREEN_DIALOG. The first takes Escape out of combat (Kit.EscapeKeys) and is named for UISpecialFrames, the game's
-- Escape in a fight and its sweeps (a loading screen, death), which close the menu with every other window.
local function level(depth)
	local f = Menu.levels[depth]
	if f then return f end
	f = CreateFrame("Frame", depth == 1 and "QuestPumpMenu" or nil, UIParent, "TooltipBackdropTemplate")
	f:SetFrameStrata("FULLSCREEN_DIALOG")
	f:SetClampedToScreen(true)
	f:EnableMouse(true)
	f:Hide()
	f.depth, f.rows = depth, {}
	if depth == 1 then
		Kit.EscapeKeys(f, function() Menu.Close() end)
		tinsert(UISpecialFrames, "QuestPumpMenu")
		f:SetScript("OnHide", function() Menu.Close() end)
	end
	Menu.levels[depth] = f
	return f
end

-- A level's frame level as it shows, Blizzard's rule (Gethe forever Blizzard_Menu/Menu.lua, 1.60.1 70425: the
-- owner's level + 500, or 9500 without an owner, "as there are some frames on the FULLSCREEN_DIALOG with questionably
-- high frame levels"), so a menu is always above the window it opens from, however that window was raised; a submenu
-- 20 over the level it opens from. Under the client's cap of 10000 with the deepest level's rows.
local LEVEL_MAX = 9900

local function lift(f)
	f:SetFrameLevel(math.min(LEVEL_MAX, Menu.base + f.depth * 20))
end

local fill

-- The submenu of a row, opened beside it; entering or clicking another row closes the deeper levels first.
local function openSubmenu(row)
	local depth = row:GetParent().depth
	for i = depth + 1, #Menu.levels do Menu.levels[i]:Hide() end
	if #row.item.items == 0 then return end
	local f = level(depth + 1)
	fill(f, row.item)
	f:ClearAllPoints()
	f:SetPoint("TOPLEFT", row, "TOPRIGHT", PAD_X, PAD_Y)
	lift(f)
	f:Show()
end

local function onClick(row)
	local item = row.item
	if item.fn then
		-- the menu goes first, so an action that opens a window or another menu keeps it; an opener that shows the
		-- choice (a dropdown) shows the new one
		local owner = Menu.owner
		Menu.Close()
		Style.Play("checkOn")
		item.fn()
		if owner and owner.Refresh then owner.Refresh() end
	else
		openSubmenu(row)
	end
end

local function makeRow(f, i)
	local row = CreateFrame("Button", nil, f)
	row.highlight = row:CreateTexture(nil, "BACKGROUND")
	row.highlight:SetAllPoints()
	row.highlight:SetTexture(Style.rowHighlight)
	row.highlight:SetBlendMode("ADD")
	row.highlight:Hide()
	row.text = Style.Text(row, "labelWhite")
	row.radio = row:CreateTexture(nil, "ARTWORK")
	row.radio:SetSize(ICON, ICON)
	row.radio:SetPoint("LEFT", -3, 0)
	row.tick = row:CreateTexture(nil, "OVERLAY")
	row.tick:SetAllPoints(row.radio)
	row.arrow = row:CreateTexture(nil, "ARTWORK")
	row.arrow:SetSize(ICON, ICON)
	row.arrow:SetPoint("RIGHT")
	row.arrow:SetTexture(Style.menuArrow)
	row.divider = row:CreateTexture(nil, "ARTWORK")
	row.divider:SetPoint("LEFT")
	row.divider:SetPoint("RIGHT")
	row.divider:SetHeight(DIVIDER_H)
	row.divider:SetTexture(Style.menuDivider)
	row:SetScript("OnEnter", function(self)
		self.highlight:Show()
		openSubmenu(self)
		local tip = self.item.tooltip
		if tip then
			GameTooltip:SetOwner(self, "ANCHOR_RIGHT")
			tip(GameTooltip)
			GameTooltip:Show()
		end
	end)
	row:SetScript("OnLeave", function(self)
		self.highlight:Hide()
		if GameTooltip:IsOwned(self) then GameTooltip:Hide() end
	end)
	row:SetScript("OnClick", onClick)
	f.rows[i] = row
	return row
end

-- Lays a level out from an entry's items: a row each, as wide as the widest text with its radio and arrow.
function fill(f, parent)
	local y, width = -PAD_Y, MIN_W
	for i, item in ipairs(parent.items) do
		local row = f.rows[i] or makeRow(f, i)
		local divider, radio, title = item.kind == "divider", item.kind == "radio", item.kind == "title"
		local submenu = #item.items > 0
		row.item = item
		row:SetHeight(divider and DIVIDER_H or ROW_H)
		row:ClearAllPoints()
		row:SetPoint("TOPLEFT", PAD_X, y)
		row:SetPoint("TOPRIGHT", -PAD_X, y)
		y = y - row:GetHeight()
		row.divider:SetShown(divider)
		row.text:SetShown(not divider)
		row.radio:SetShown(radio and Style.SetAtlas(row.radio, "menuRadio"))
		row.tick:SetShown(radio and item.isSelected() and Style.SetAtlas(row.tick, "menuRadioTick") or false)
		row.arrow:SetShown(submenu)
		Style.SetFont(row.text, title and "label" or "labelWhite")
		row.text:ClearAllPoints()
		row.text:SetPoint("LEFT", radio and ICON or 0, 0)
		row.text:SetText(item.text or "")
		row.highlight:Hide()
		row:EnableMouse(not (divider or title))
		if not divider then
			width = math.max(width, row.text:GetStringWidth() + (radio and ICON or 0) + (submenu and ICON + 2 or 0))
		end
		row:Show()
	end
	for i = #parent.items + 1, #f.rows do f.rows[i]:Hide() end
	f:SetSize(width + 2 * PAD_X, PAD_Y - y)
end

-- Opens a menu at the cursor, as MenuUtil.CreateContextMenu did: generator(owner, root) describes it. The owner going
-- (a pump that ends under its Pause menu, a window that closes) takes the menu with it; a menu with no owner (a slash
-- command's) closes on a click off it or an entry only. An owner that shows the choice (a select, Kit.Select) hears
-- it through owner.Refresh() and the menu's close through owner.OnMenuClosed() (its arrow).
function Menu.Open(owner, generator)
	Menu.Close()
	local root = entry("root")
	generator(owner, root)
	if #root.items == 0 then return end
	local f = level(1)
	fill(f, root)
	local x, y = GetCursorPosition()
	local scale = f:GetEffectiveScale()
	f:ClearAllPoints()
	f:SetPoint("TOPLEFT", UIParent, "BOTTOMLEFT", x / scale, y / scale)
	Menu.owner = owner
	Menu.base = owner and owner:GetFrameLevel() + 500 or 9500
	lift(f)
	f:Show()
	if owner then
		T.every("owner", 0.2, function()
			if not (Menu.owner and Menu.owner:IsVisible()) then Menu.Close() end
		end)
	end
	Menu.isOpen = true
	ns.Emit("Menu")
end

function Menu.Close()
	T.cancel("owner")
	local owner, was = Menu.owner, Menu.isOpen
	Menu.owner, Menu.isOpen = nil, nil
	for i = #Menu.levels, 1, -1 do Menu.levels[i]:Hide() end
	if owner and owner.OnMenuClosed then owner.OnMenuClosed() end
	if was then ns.Emit("Menu") end
end

-- Whether a menu stands open, every way of closing it going through Menu.Close (the first level's OnHide).
function Menu.Shown()
	return Menu.isOpen == true
end

-- A left or right click anywhere off the menu closes it, as Blizzard's close (Menu.lua's GLOBAL_MOUSE_DOWN); the click
-- still goes where it was aimed.
ns.RegisterEvent("GLOBAL_MOUSE_DOWN", function(button)
	local first = Menu.levels[1]
	if not (first and first:IsShown()) or (button ~= "LeftButton" and button ~= "RightButton") then return end
	-- a click on the opener is its own: a dropdown's closes the menu, a right-click opens it afresh
	if Menu.owner and Menu.owner:IsMouseOver() then return end
	for _, f in ipairs(Menu.levels) do
		if f:IsShown() and f:IsMouseOver() then return end
	end
	Menu.Close()
end)

-- Dialogs -----------------------------------------------------------------------------------------------------------

-- A confirmation as Blizzard's popup draws one, on QuestPump's own frames, registered once: { text (a format filled
-- from the two text arguments), name (what is offered, on a line of its own in a card's title font under the text: a
-- pact's plan; a string, or a function answering it from the show's data), note (a paragraph under it in the notes'
-- grey: what the destructive action it confirms takes away; or a function answering it at each show), button1 (the
-- accept, named for its action: never Yes), button2 (the decline: "Keep it", "Not now"), tip1 and tip2 (each
-- button's tooltip words), answers (true: a question with two answers, neither the way on, so both are grey, the
-- accept in its place: the kit's rule that an ask's answers are never red), OnAccept(frame, data), OnHide(frame,
-- data), timeout (s) }. At the kit's small tier (Kit.TIER.S): the decline grey at the bottom left, the accept red at
-- the bottom right, each as wide as its words + 40 and at least the tier's least; kept inside the screen. OnHide
-- runs on every close, after OnAccept for the accept (Blizzard's order, which the party question relies on); the
-- decline, Escape (Kit.EscapeKeys) and the timeout only close it. Like Blizzard's popups the dialogs stay out of
-- UISpecialFrames, so the game's sweeps (a loading screen, death, a loss of control, a cinematic's start) never
-- answer one. One of a kind at a time, at most three, stacked from the top of the screen under Blizzard's own
-- popups.
local Dialog = { defs = {}, frames = {} }
ns.Dialog = Dialog

local DIALOGS, DIALOG_TOP = 3, -135
local TIER = Kit.TIER.S

function Dialog.Register(which, def)
	Dialog.defs[which] = def
end

-- The lowest edge of Blizzard's popups on the screen (read only: StaticPopup1-4, which sit on UIParent's scale), nil
-- with none shown.
local function blizzardBottom()
	local bottom
	for i = 1, 4 do
		local popup = _G["StaticPopup" .. i]
		local edge = popup and popup:IsShown() and popup:GetBottom()
		if edge and (not bottom or edge < bottom) then bottom = edge end
	end
	return bottom
end

local function restack()
	local above, bottom = nil, blizzardBottom()
	for _, f in ipairs(Dialog.frames) do
		if f:IsShown() then
			f:ClearAllPoints()
			if above then
				f:SetPoint("TOP", above, "BOTTOM", 0, 0)
			elseif bottom then
				f:SetPoint("TOP", UIParent, "BOTTOM", 0, bottom)
			else
				f:SetPoint("TOP", UIParent, "TOP", 0, DIALOG_TOP)
			end
			above = f
		end
	end
end

-- A Blizzard popup that shows while one of QuestPump's does takes the top; QuestPump's move under it (a post-hook:
-- Blizzard's call has run).
hooksecurefunc("StaticPopup_Show", function()
	if Dialog.AnyShown() then restack() end
end)

local function dialogFrame(i)
	local name = "QuestPumpDialog" .. i
	-- the frame is Blizzard's dialog border itself (its nine-slice and background); named, so ConsolePort's cursor
	-- finds it (Skins.lua)
	local f = CreateFrame("Frame", name, UIParent, "DialogBorderTemplate")
	-- the template draws at its parent's level (useParentLevel, for a border inside a window); this frame is the window
	if f.SetUsingParentLevel then f:SetUsingParentLevel(false) end
	f:SetFrameStrata("DIALOG")
	f:SetToplevel(true)
	f:SetClampedToScreen(true)
	f:EnableMouse(true)
	f:Hide()
	f.text = Kit.Text(f, "bodyWhite", "CENTER", nil, TIER.TEXT)
	f.text:SetPoint("TOP", 0, -TIER.HEAD)
	f.name = Kit.Text(f, "cardTitle", "CENTER", nil, TIER.TEXT)
	f.name:SetPoint("TOP", f.text, "BOTTOM", 0, -8)
	f.note = Kit.Text(f, "bodyWhite", "CENTER", Style.saga.note, TIER.TEXT)
	local function accept()
		local def, data = Dialog.defs[f.which], f.data
		if def and def.OnAccept then def.OnAccept(f, data) end
		f:Hide()
	end
	f.accept = Kit.Primary(f, "", TIER.PRIMARY, accept)
	f.answer = Kit.Secondary(f, "", TIER.PRIMARY, accept, TIER.BUTTON) -- the accept of a dialog of answers
	f.decline = Kit.Secondary(f, "", TIER.BACK, function() f:Hide() end, TIER.BUTTON)
	f.accept:SetHeight(TIER.BUTTON)
	f.accept:SetPoint("BOTTOMRIGHT", -TIER.IN, TIER.BOTTOM)
	f.answer:SetPoint("BOTTOMRIGHT", -TIER.IN, TIER.BOTTOM)
	f.decline:SetPoint("BOTTOMLEFT", TIER.IN, TIER.BOTTOM)
	-- each button's tooltip where the dialog has words for it (tip1, tip2), under its name
	for _, part in ipairs({ { f.accept, "tip1" }, { f.answer, "tip1" }, { f.decline, "tip2" } }) do
		local b, key = part[1], part[2]
		b:SetScript("OnEnter", function(self)
			local def = Dialog.defs[f.which]
			local words = def and def[key]
			if not words then return end
			GameTooltip:SetOwner(self, "ANCHOR_TOP")
			Kit.Tip(self:GetText(), words)(GameTooltip)
			GameTooltip:Show()
		end)
		b:SetScript("OnLeave", function() GameTooltip:Hide() end)
	end
	Kit.EscapeKeys(f, function() f:Hide() end)
	f:SetScript("OnHide", function()
		T.cancel(name)
		local def, data = Dialog.defs[f.which], f.data
		f.which, f.data = nil, nil
		restack()
		if def and def.OnHide then def.OnHide(f, data) end
	end)
	Dialog.frames[i] = f
	return f
end

-- All three at load, so Skins.lua can hand them to ConsolePort's cursor at PLAYER_LOGIN.
for i = 1, DIALOGS do dialogFrame(i) end

local function freeFrame()
	for _, f in ipairs(Dialog.frames) do
		if not f:IsShown() then return f end
	end
end

-- Shows a registered dialog; returns its frame, or nil when none can show: a movie or a cinematic is on (the caller
-- asks again later, as Blizzard's popups refuse to show then), or three show already. A dialog of the same kind closes
-- first, so a second show starts afresh. opts.accept names the accept button for this show (the reset dialog's "Reset
-- it all", "Clear its record").
function Dialog.Show(which, a1, a2, data, opts)
	local def = Dialog.defs[which]
	if not def or not UIParent:IsShown() or InCinematic() or IsInCinematicScene() then return nil end
	Dialog.Hide(which)
	local f = freeFrame()
	if not f then return nil end
	f.which, f.data = which, data
	f.text:SetWidth(TIER.TEXT)
	f.text:SetText(def.text:format(a1, a2))
	local name = def.name
	if type(name) == "function" then name = name(data) end
	f.name:SetWidth(TIER.TEXT)
	f.name:SetText(name or "")
	f.name:SetShown(name ~= nil)
	local note = def.note
	if type(note) == "function" then note = note(data) end -- nil: no note this time
	f.note:SetWidth(TIER.TEXT)
	f.note:SetText(note or "")
	f.note:SetShown(note ~= nil)
	f.note:ClearAllPoints()
	f.note:SetPoint("TOP", name and f.name or f.text, "BOTTOM", 0, -8)
	local accept = def.answers and f.answer or f.accept
	f.accept:SetShown(accept == f.accept)
	f.answer:SetShown(accept == f.answer)
	accept:SetText(opts and opts.accept or def.button1)
	accept:SetWidth(math.max(TIER.PRIMARY, accept:GetTextWidth() + 40))
	f.decline:SetShown(def.button2 ~= nil)
	local width = TIER.W
	if def.button2 then
		f.decline:SetText(def.button2)
		f.decline:SetWidth(math.max(TIER.BACK, f.decline:GetTextWidth() + 40))
		width = math.max(width, accept:GetWidth() + f.decline:GetWidth() + 2 * TIER.IN + 12)
	end
	local textH = f.text:GetStringHeight() + (name and 8 + f.name:GetStringHeight() or 0)
		+ (note and 8 + f.note:GetStringHeight() or 0)
	f:SetSize(width, TIER.HEAD + textH + 16 + TIER.BUTTON + TIER.BOTTOM)
	f:Show()
	f:Raise()
	restack()
	if def.timeout and def.timeout > 0 then
		T.after(f:GetName(), def.timeout, function()
			if f.which == which then f:Hide() end
		end)
	end
	return f
end

function Dialog.Hide(which)
	for _, f in ipairs(Dialog.frames) do
		if f:IsShown() and f.which == which then f:Hide() end
	end
end

function Dialog.Visible(which)
	for _, f in ipairs(Dialog.frames) do
		if f:IsShown() and f.which == which then return true end
	end
	return false
end

function Dialog.AnyShown()
	for _, f in ipairs(Dialog.frames) do
		if f:IsShown() then return true end
	end
	return false
end
