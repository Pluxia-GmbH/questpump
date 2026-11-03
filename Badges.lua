-- QuestPump beside a player (docs/plan.md slice 23: (3) the unit-frame badges, (4) the nameplate line, (5) the chat
-- mark, (7) the Inspect tab, (8) the portrait borders). Everything here reads the session's cards (Social.lua's
-- peers, clamped by Social.Decode) and stores nothing: a surface shows a card the session already holds and never asks
-- for one, except the Inspect window, which asks its player as the tooltip asks on hover. Every surface is quiet while
-- the social layer is (Social.Listening) or the player keeps others' cards off (the Social page's "Show others'
-- cards"); a muted or ignored player gets no badge, line or mark (Social.Shown), and the Inspect pane, opened on
-- purpose, shows what the card window shows. The frames here are QuestPump's own, never a secure template, anchored to
-- Blizzard's regions and never measured (an anchor's secrecy passes to what hangs on it); the ones on the unit frames
-- and the nameplates are made out of combat, and none of them takes the mouse from a unit frame.

local ADDON, ns = ...
local C, L = ns.C, ns.L
local Q = ns.Enum.quality
local Lc, Sheet, Social, Style, Kit = ns.Lifecycle, ns.Sheet, ns.Social, ns.Style, ns.Kit

local Badges = {}
ns.Badges = Badges
local T = ns.Timers()
local readable = ns.Readable

-- Blizzard's follower portrait (GarrisonFollowerPortraitTemplate, Blizzard_GarrisonBase, Gethe `forever` 70205): a
-- 44-unit round portrait, its ring's top 26 units above the portrait's centre, the level plate's centre 25 below it,
-- 58 units wide for a level and 70 for an item level (GarrisonFollowerPortraitMixin:SetLevel and SetILevel), 24 high.
local PORTRAIT, RING_TOP, PLATE_Y, PLATE_W, PLATE_WIDE, PLATE_H, CLOCK = 44, 26, -25, 58, 70, 24, 16
-- The border's rising order (ns.Border: 1-4 at C.border_levels, 5 for any Paragon rank, 0 none), Blizzard's own for its
-- followers: the ring in the colour of item quality from Uncommon to Legendary, one a border level, then the
-- champion's ring (Enum.GarrFollowerQuality.Title). A level added to C.border_levels needs its colour here.
local TIER = { Q.uncommon, Q.rare, Q.epic, Q.legendary }
local CHAMPION = #C.border_levels + 1
-- How far over its owner a medallion draws (the unit frames' own art and their ready-check and leader marks).
local LEVEL_UP = 10

-- The cards -----------------------------------------------------------------------------------------------------------

-- The card a surface shows for a player (Social.Shown: theirs from the session, the player's own while they share it);
-- nil for none.
local function cardOf(key)
	local shown, p = Social.Shown(key)
	return shown and p and p.card or nil
end

-- "QP 12", and past the cap the rank as the sheet's level line reads it ("Paragon 2").
local function levelText(c)
	if (c.P or 0) > 0 then return L["social.badge.paragon"]:format(c.P) end
	return L["social.badge.level"]:format(c.L)
end

-- A unit token that shows a player now (the target, the focus, the mouseover, a group member), for a portrait; nil
-- when none does.
local AT_HAND = { "target", "focus", "mouseover" }

local function unitFor(key)
	if not key then return nil end
	if key == Social.me then return "player" end
	for _, unit in ipairs(AT_HAND) do
		if Social.UnitKey(unit) == key then return unit end
	end
	return (Social.GroupKeys())[key]
end

local function inFight()
	return Lc.combat or InCombatLockdown()
end

-- The medallion: the ring and the level plate -------------------------------------------------------------------------

-- A ring's atlas at `s` times its own size (Blizzard anchors both rings by their top); false, and hidden, where the
-- client lacks the art.
local function setRing(tex, key, s)
	local info = C_Texture.GetAtlasInfo(Style.atlas[key])
	if not (info and Style.SetAtlas(tex, key)) then
		tex:Hide()
		return false
	end
	tex:SetSize(info.width * s, info.height * s)
	tex:Show()
	return true
end

-- A medallion for a round portrait `size` across, which its caller anchors by the portrait's centre: the ring and, with
-- `plateScale`, the level plate on the ring's foot (the badge: "QP 12", the clock while that player pumps). It takes no
-- mouse: over a unit frame the cursor stays the frame's (its unit is the mouseover, its tooltip shows the card's
-- lines). `portrait`: a round portrait of its own (the card window, the Inspect pane); a unit frame draws its own.
local function medallion(parent, size, plateScale, portrait)
	local m = CreateFrame("Frame", nil, parent)
	m:SetSize(size, size)
	m:SetFrameLevel(parent:GetFrameLevel() + LEVEL_UP)
	m.s = size / PORTRAIT
	if portrait then
		m.portrait = m:CreateTexture(nil, "ARTWORK")
		m.portrait:SetAllPoints()
		local mask = m:CreateMaskTexture()
		mask:SetAllPoints(m.portrait)
		if Style.SetAtlas(mask, "circle") then m.portrait:AddMaskTexture(mask) end
	end
	-- the ring and its tinted twin over it, as Blizzard layers PortraitRing and PortraitRingQuality
	m.ring = m:CreateTexture(nil, "OVERLAY", nil, 1)
	m.tint = m:CreateTexture(nil, "OVERLAY", nil, 2)
	m.ring:SetPoint("TOP", m, "CENTER", 0, RING_TOP * m.s)
	m.tint:SetPoint("TOP", m, "CENTER", 0, RING_TOP * m.s)
	if plateScale then
		local plate = CreateFrame("Frame", nil, m)
		plate:SetScale(plateScale)
		plate:SetSize(PLATE_W, PLATE_H)
		plate:SetPoint("CENTER", m, "CENTER", 0, PLATE_Y * m.s / plateScale) -- in the plate's own scale
		plate.bg = plate:CreateTexture(nil, "ARTWORK")
		plate.bg:SetAllPoints()
		plate.text = Style.Text(plate, "labelWhite", "CENTER")
		plate.text:SetPoint("LEFT", 4, 0)
		plate.text:SetPoint("RIGHT", -4, 0)
		plate.text:SetWordWrap(false)
		plate.clock = plate:CreateTexture(nil, "OVERLAY")
		plate.clock:SetSize(CLOCK, CLOCK)
		plate.clock:SetPoint("CENTER")
		m.plate = plate
	end
	return m
end

-- Paints a medallion from a card's clamped fields (L, P, B, F): the ring by the border id, the plate's level, or the
-- clock while that player pumps (the level always with `clockApart`: the card window shows the clock beside it).
-- Without a card the ring and the plate go and a portrait stays.
local function paint(m, c, clockApart)
	local border = c and c.B or 0
	local tier = TIER[border]
	m.tint:Hide()
	if border == CHAMPION then
		setRing(m.ring, "ringTop", m.s)
	elseif tier and setRing(m.ring, "ring", m.s) and setRing(m.tint, "ring", m.s) then
		m.tint:SetVertexColor(Sheet.QualityColor(tier):GetRGB())
	else
		m.ring:Hide()
	end
	local plate = m.plate
	if not plate then return end
	plate:SetShown(c ~= nil)
	if not c then return end
	-- the plate in the ring's colour (Blizzard tints LevelBorder with PortraitRingQuality), the champion's its own art
	plate.bg:SetShown(Style.SetAtlas(plate.bg, border == CHAMPION and "plateTop" or "plate"))
	if tier then
		plate.bg:SetVertexColor(Sheet.QualityColor(tier):GetRGB())
	else
		plate.bg:SetVertexColor(1, 1, 1)
	end
	plate:SetWidth((c.P or 0) > 0 and PLATE_WIDE or PLATE_W)
	local pumping = not clockApart and c.F == 1 and Style.SetAtlas(plate.clock, "clock")
	plate.clock:SetShown(pumping)
	plate.text:SetShown(not pumping)
	plate.text:SetText(levelText(c))
end

-- (3) and (8) The unit frames -----------------------------------------------------------------------------------------

-- The frames a badge sits on, by their portrait (Blizzard_UnitFrame's Mainline family, which Forever loads with its
-- Camelot overrides at 70205 and retail at 12.1.0): the target's and the focus's 58-unit portrait
-- (TargetFrameTemplate's TargetFrameContainer.Portrait), a party member's 37-unit one (PartyMemberFrameTemplate's
-- Portrait, in PartyFrame's pool); each medallion a few units wider, so the ring lies on Blizzard's own, and the plate
-- scaled to the frame. Raid-style party frames (CompactPartyFrame) carry no portrait and get no badge.
local TARGET = { size = 62, plate = 0.9 }
local PARTY = { size = 40, plate = 0.7 }

local badges = {} -- unit frame → { m = its medallion, unit = its unit; a party member's frame carries its own }
local pending = false -- a frame waiting for the end of a fight to get its medallion

local function attach(owner, portrait, spot, unit)
	local m = medallion(owner, spot.size, spot.plate, false)
	m:SetPoint("CENTER", portrait, "CENTER")
	m:Hide()
	badges[owner] = { m = m, unit = unit }
end

-- Gives every unit frame its medallion, out of combat only; a fight defers it to its end.
local function ensure()
	if InCombatLockdown() then
		pending = true
		return
	end
	pending = false
	for unit, frame in pairs({ target = TargetFrame, focus = FocusFrame }) do
		local portrait = frame and frame.TargetFrameContainer and frame.TargetFrameContainer.Portrait
		if portrait and not badges[frame] then attach(frame, portrait, TARGET, unit) end
	end
	local pool = PartyFrame and PartyFrame.PartyMemberFramePool
	if not pool then return end
	for member in pool:EnumerateActive() do
		if member.Portrait and not badges[member] then attach(member, member.Portrait, PARTY) end
	end
end

-- The badges of the units that show `key` (all of them without one): the card's medallion, or none.
local function refreshFrames(key)
	for owner, b in pairs(badges) do
		local m, unit = b.m, b.unit or owner.unitToken -- a member frame Blizzard has not set up yet has none
		local unitKey = unit and Social.UnitKey(unit)
		if not key or unitKey == key then
			local c = cardOf(unitKey)
			m:SetShown(c ~= nil)
			if c then paint(m, c) end
		end
	end
end

-- (4) The nameplates --------------------------------------------------------------------------------------------------

-- "QP 12 · the Relentless" under the name on a friendly player's nameplate in the open world, for a card the session
-- holds (a nameplate never asks: walking through a city sends nothing); none in an instance or a fight, none on a
-- forbidden plate. Each plate (C_NamePlate's base frame, which keeps its place while Blizzard's pooled UnitFrame comes
-- and goes) gets its own line frame once, out of combat.
local plates, lines = {}, {} -- nameplate unit → its plate; plate → its line

local function renderPlate(unit)
	local plate = plates[unit]
	if not plate then return end
	local key = not (inFight() or IsInInstance()) and readable(UnitIsFriend("player", unit)) and Social.UnitKey(unit)
	local c = key and key ~= Social.me and cardOf(key)
	local line = lines[plate]
	if c and not line and not InCombatLockdown() then
		line = CreateFrame("Frame", nil, plate)
		line:SetAllPoints()
		line.text = Style.Text(line, "plateLine", "CENTER")
		line.text:SetTextColor(NORMAL_FONT_COLOR:GetRGB())
		lines[plate] = line
	end
	if not line then return end
	if not c then return line:Hide() end
	-- under the name where the plate draws one (its place moves with the plate's style), else under the plate
	local name = plate.UnitFrame and plate.UnitFrame.name
	line.text:ClearAllPoints()
	if name then line.text:SetPoint("TOP", name, "BOTTOM", 0, -1) else line.text:SetPoint("TOP", plate, "BOTTOM") end
	local text = levelText(c)
	if c.title then text = text .. " · " .. L["title." .. c.title] end
	line.text:SetText(text)
	line:Show()
end

local function refreshPlates(key)
	for unit in pairs(plates) do
		if not key or Social.UnitKey(unit) == key then renderPlate(unit) end
	end
end

ns.RegisterEvent("NAME_PLATE_UNIT_ADDED", function(unit)
	local plate = C_NamePlate.GetNamePlateForUnit(unit)
	if not plate or plate:IsForbidden() then return end
	plates[unit] = plate
	renderPlate(unit)
end)

ns.RegisterEvent("NAME_PLATE_UNIT_REMOVED", function(unit)
	local plate = plates[unit]
	plates[unit] = nil
	if plate and lines[plate] then lines[plate]:Hide() end
end)

-- (5) The chat mark ---------------------------------------------------------------------------------------------------

-- The addon's icon before the name of a player known to run QuestPump this session (a card or a private answer), never
-- a muted or ignored one, and before the player's own while they share the card. Blizzard's filter registry calls it
-- with the decorated name and the chat event's arguments (text, sender, ...) and skips it while the name is secret, in
-- chat lockdown (ChatFrameFilters.lua, 70205); a nil answer leaves the name.
local function onSender(_, decorated, _, sender)
	sender = readable(sender)
	if not sender or sender == "" then return nil end
	local shown, p = Social.Shown(Social.Key(sender))
	if not (shown and p and p.known) then return nil end
	return Style.AddonMark() .. decorated
end

ChatFrameUtil.AddSenderNameFilter(onSender)

-- (7) The Inspect tab -------------------------------------------------------------------------------------------------

-- A QuestPump side tab on the Inspect window (Sheet.SideTab), an own frame under Blizzard's side tabs on Forever
-- (InspectFrame.ModeTabs, the Camelot Blizzard_InspectUI, 70291), where Forever's would start on retail's (whose
-- Inspect window has bottom tabs only); never in Blizzard's tab arrays. Its pane covers the window's inset (the paper
-- doll under it stays as it is, shown or not, and takes no clicks: the pane stands over the Camelot paper doll's item
-- slots, frame level 100 in InspectPaperDollFrame.xml, and under the window's border at 500) with the inspected
-- player's card: their portrait in their border, their title and the card window's rows. Any of Blizzard's tabs takes
-- the window back (InspectSwitchTabs, hooked, which InspectFrame_Show runs for every player inspected); the window asks
-- its player for the card when it opens and when it shows another player, and keeps whom it asked: the unit token it
-- was opened with may name someone else later (a focus or a mouseover inspected).
local inspect = {}
local TAB_Y, PANE_LEVEL, PANE_PORTRAIT, PANE_ROW = -30, 120, 56, 24

local function inspected()
	return inspect.unit, inspect.key
end

-- The tab under the last of Blizzard's side tabs shown, with no gap, as InspectFrameMixin:InitializeModeTabLayout
-- chains them (once, at OnLoad, all three shown then: each keeps its anchor while UpdateTabs shows or hides the guild
-- tab, the last); on retail at the place where Forever's column starts.
local function placeTab()
	local tab, tabs = inspect.tab, InspectFrame.ModeTabs
	local last
	for _, t in ipairs(tabs and tabs.Tabs or {}) do
		if t:IsShown() then last = t end
	end
	tab:ClearAllPoints()
	if last then
		tab:SetPoint("TOPLEFT", last, "BOTTOMLEFT")
	elseif tabs then
		tab:SetPoint("TOPLEFT", tabs, "TOPLEFT")
	else
		tab:SetPoint("TOPLEFT", InspectFrame, "TOPRIGHT", 0, TAB_Y)
	end
end

-- The pane's card, as the card window shows the other player's (a private answer, the card, or none): the medallion
-- with its level, the title, the card window's stat rows only with a card (Social.CardRows, the kit's inline stats:
-- the number, its words beside it), and the card window's status line (Social.CardStatus).
local function renderPane()
	local pane = inspect.pane
	if not (pane and pane:IsShown()) then return end
	local _, key = inspected()
	local p = key and Social.peers[key] or {}
	local theirs = not p.private and p.card or nil
	paint(pane.medal, theirs)
	pane.title:SetText(theirs and theirs.title and L["title." .. theirs.title] or "")
	for i, r in ipairs(Social.CardRows(theirs)) do
		pane.rows[i]:SetShown(theirs ~= nil)
		Kit.FillStat(pane.rows[i], BreakUpLargeNumbers(r[1]), r[2])
	end
	pane.status:SetText(Social.CardStatus(key, p, theirs))
end

-- The window's player, taken when it opens or shows another (Blizzard shows the window once its unit's GUID answered,
-- INSPECT_READY, so the token names them then), with their portrait: later the token may name someone else.
local function takeInspected()
	local unit = InspectFrame.unit
	inspect.unit, inspect.key = unit, unit and Social.UnitKey(unit)
	local portrait = inspect.pane and inspect.pane.medal.portrait
	if not portrait then return end
	if inspect.key then SetPortraitTexture(portrait, unit) else portrait:SetTexture(Style.addonIcon) end
end

-- The window's player asked while the token still names them (a friendly player online, never a private player's
-- ask), and the "Asking" line gives way once an answer would have come.
local function askInspected()
	local unit, key = inspected()
	if not (key and Social.Listening() and ns.profile.showCards and Social.UnitKey(unit) == key) then return end
	Social.AskUnit(unit)
	T.after("inspect", C.probe_s, renderPane)
end

-- Blizzard's side tabs look unselected while the pane stands in front (SidePanelTabButtonMixin:SetChecked only shows or
-- hides their art and writes no field), the one that was selected kept for setPane's `back`.
local function quietTabs()
	for _, t in ipairs(InspectFrame.ModeTabs and InspectFrame.ModeTabs.Tabs or {}) do
		if t.SelectedTexture:IsShown() then inspect.selected = t end
		t:SetChecked(false)
	end
end

-- The pane in front, or gone; `back` gives the Blizzard tab that was selected its look again, which a Blizzard tab
-- taking the window back does itself (InspectSwitchTabs selects its own).
local function setPane(shown, back)
	local pane, tab = inspect.pane, inspect.tab
	pane:SetShown(shown)
	tab:SetChecked(shown)
	if shown then
		quietTabs()
		return renderPane()
	end
	if back and inspect.selected then inspect.selected:SetChecked(true) end
	inspect.selected = nil
end

-- The tab while the layer shows others' cards; gone, it takes its pane with it.
local function showTab()
	local tab = inspect.tab
	if not tab then return end
	local on = Social.Listening() and ns.profile.showCards == true
	tab:SetShown(on)
	if not on and inspect.pane:IsShown() then setPane(false, true) end
end

local function buildPane()
	local inset = InspectFrame.Inset
	local pane = CreateFrame("Frame", nil, InspectFrame)
	pane:SetPoint("TOPLEFT", inset, "TOPLEFT", 3, -3)
	pane:SetPoint("BOTTOMRIGHT", inset, "BOTTOMRIGHT", -3, 3)
	pane:SetFrameLevel(InspectFrame:GetFrameLevel() + PANE_LEVEL)
	pane:EnableMouse(true)
	pane:Hide()
	-- the inset's own marble (InsetFrameTemplate), so the pane reads as the window's page
	pane.bg = pane:CreateTexture(nil, "BACKGROUND")
	pane.bg:SetAllPoints()
	pane.bg:SetTexture(Style.insetBackground, "REPEAT", "REPEAT")
	pane.bg:SetHorizTile(true)
	pane.bg:SetVertTile(true)
	pane.medal = medallion(pane, PANE_PORTRAIT, 1, true)
	pane.medal:SetPoint("CENTER", pane, "TOP", 0, -PANE_PORTRAIT)
	pane.title = Style.Text(pane, "bodyWhite", "CENTER")
	pane.title:SetPoint("TOP", pane, "TOP", 0, -2 * PANE_PORTRAIT - 4)
	pane.rows = {}
	for i = 1, #Social.CardRows() do
		local row = Kit.Stat(pane, { inline = true })
		row:SetHeight(PANE_ROW)
		row:SetPoint("TOPLEFT", 16, -2 * PANE_PORTRAIT - 16 - (i - 1) * PANE_ROW)
		row:SetPoint("RIGHT", -16, 0)
		pane.rows[i] = row
	end
	pane.status = Style.Text(pane, "labelWhite", "CENTER")
	pane.status:SetPoint("BOTTOMLEFT", 12, 10)
	pane.status:SetPoint("BOTTOMRIGHT", -12, 10)
	pane.status:SetWordWrap(true)
	return pane
end

-- The tab and its pane, once Blizzard_InspectUI has loaded (load on demand: its ADDON_LOADED, or at login when another
-- addon loaded it first).
local function buildInspect()
	if inspect.tab or not (InspectFrame and InspectFrame.Inset) then return end
	inspect.tab = Sheet.SideTab(InspectFrame.ModeTabs or InspectFrame, Style.addonIcon, L["social.inspect.tab"],
		function()
			askInspected()
			setPane(true)
		end)
	inspect.pane = buildPane()
	placeTab()
	showTab()
	-- Forever's INSPECT_READY for the window's player, at every open and again whenever anything inspects them:
	-- SetupModeTabsForUnit selects Blizzard's tab again, the window shows, and UpdateTabs shows or hides the guild tab
	-- (retail's InspectFrame has no UpdateTabs and its tab never moves)
	if InspectFrame.UpdateTabs then
		hooksecurefunc(InspectFrame, "UpdateTabs", function()
			placeTab()
			if inspect.pane:IsShown() then quietTabs() end
		end)
	end
	hooksecurefunc("InspectSwitchTabs", function()
		setPane(false)
		if InspectFrame:IsShown() and InspectFrame.unit ~= inspect.unit then
			takeInspected()
			askInspected()
		end
	end)
	InspectFrame:HookScript("OnShow", function()
		showTab()
		takeInspected()
		askInspected()
	end)
	InspectFrame:HookScript("OnHide", function() setPane(false, true) end)
end

ns.RegisterEvent("ADDON_LOADED", function(name)
	if name == "Blizzard_InspectUI" then buildInspect() end
end)

-- (8) The card window -------------------------------------------------------------------------------------------------

-- Social.lua's card window draws each player's medallion at its card's top left: the player's portrait, and theirs
-- while a unit shows them (the addon's icon otherwise), each in its own card's border with the level plate on its
-- foot ("QP 24"; the card shows the clock apart, at its top right).
local CARD_PORTRAIT, CARD_PLATE, CARD_IN = 48, 0.65, 12

-- The medallions, one a card, built with the window.
function Badges.CardMedals(cards)
	local medals = {}
	for i, card in ipairs(cards) do
		local m = medallion(card, CARD_PORTRAIT, CARD_PLATE, true)
		m:SetPoint("CENTER", card, "TOPLEFT", CARD_IN + CARD_PORTRAIT / 2, -(CARD_IN + CARD_PORTRAIT / 2))
		medals[i] = m
	end
	return medals
end

function Badges.CardBorders(f, mineCard, theirs)
	local function side(m, unit, c)
		m:SetShown(c ~= nil)
		if not c then return end
		if unit then SetPortraitTexture(m.portrait, unit) else m.portrait:SetTexture(Style.addonIcon) end
		paint(m, c, true)
	end
	side(f.medals[1], "player", mineCard)
	side(f.medals[2], unitFor(f.key), theirs)
end

-- Updates -------------------------------------------------------------------------------------------------------------

-- Every surface that shows `key` (all of them without one): a card came or went, a peer was muted, a setting changed.
local function refresh(key)
	refreshFrames(key)
	refreshPlates(key)
	local _, inspectedKey = inspected()
	if not key or key == inspectedKey then renderPane() end
end

ns.On("Social", refresh)
ns.On("Peers", function() refresh() end)
ns.On("Settings", function()
	showTab()
	refresh()
end)

ns.RegisterEvent("PLAYER_LOGIN", function()
	ensure()
	refresh()
	if C_AddOns.IsAddOnLoaded("Blizzard_InspectUI") then buildInspect() end
end)
-- a zone or an instance: the layer may listen again or go quiet there, the plates go in an instance
ns.RegisterEvent("PLAYER_ENTERING_WORLD", function() refresh() end)
ns.RegisterEvent("PLAYER_TARGET_CHANGED", function() refreshFrames() end)
ns.RegisterEvent("PLAYER_FOCUS_CHANGED", function() refreshFrames() end)
ns.RegisterEvent("GROUP_ROSTER_UPDATE", function()
	ensure()
	refreshFrames()
end)

-- A fight takes the nameplate lines (Lc.combat is set before this handler runs, Events.lua coming first in the TOC);
-- its end brings them back and gives the frames that came meanwhile their medallions.
ns.RegisterEvent("PLAYER_REGEN_DISABLED", function()
	for _, line in pairs(lines) do line:Hide() end
end)
ns.RegisterEvent("PLAYER_REGEN_ENABLED", function()
	if pending then ensure() end
	refresh()
end)
