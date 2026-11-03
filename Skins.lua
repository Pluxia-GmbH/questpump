-- QuestPump under a UI replacement (docs/plan.md slice 23; docs/ui.md, "UI replacements: ElvUI, Tukui, AddOnSkins,
-- ConsolePort"): ElvUI's look on the addon's own windows, and ConsolePort's cursor on the pump panel. Each part sleeps
-- unless its addon is loaded, and a missing or changed API costs one log line, never an error. Blizzard's own frames
-- are never touched here: ElvUI skins those itself.

local ADDON, ns = ...

local ELVUI, WINDTOOLS, CONSOLEPORT = "ElvUI", "ElvUI_WindTools", "ConsolePort"

local function loaded(name)
	return C_AddOns.IsAddOnLoaded(name) == true
end

-- A third party's API called under pcall: a missing or changed one says so in the log once, and the rest goes on.
local warned = {}

local function try(what, fn, ...)
	local ok, err = pcall(fn, ...)
	if not ok and not warned[what] then
		warned[what] = true
		ns.Log("W", "%s failed: %s", what, tostring(err))
	end
	return ok
end

-- ElvUI ---------------------------------------------------------------------------------------------------------------

-- The windows ElvUI skins: every window the kit builds (Kit.lua's Kit.windows, its handler's name for its chrome, or
-- false for a Settings canvas inside Blizzard's panel, which ElvUI skins itself: only its widgets), found as the kit
-- builds it (Kit.Window and Kit.Canvas, hooked with hooksecurefunc) and walked again at the end of each of its renders
-- (Kit.Fit), so a row built later is skinned too; and the windows not built by the kit yet, one line each: `frame`
-- finds the window once `after` has run (an ns table's builder or the opener that builds it, hooked, never edited),
-- `how` names the Skins handler for its own chrome. A window's widgets are found by type (widget, below), again
-- whenever it shows. The kit's own drawing (Style.Box grounds and edges, its boxes, selects, switches and rows) carries
-- no template parts, so the walk leaves it as drawn and restyles only its red and grey buttons, the close X, edit
-- boxes and scroll bars. The QR window is a kit window kept out of the frame skin (Kit.Window's `skin` false: ElvUI
-- strips a frame's own textures, and its code is drawn in textures), its close skinned alone; left out: the HUD, the
-- banner, the Legendary frame and the fanfare (own art on no template).
local WINDOWS = {
	{ after = { "Sheet", "Build" }, frame = function() return _G.QuestPumpSheet end, how = "HandlePortraitFrame" },
	{ after = { "Panel", "Build" }, frame = function() return ns.Panel.frames.stop end, how = "HandleFrame" },
	{ after = { "Streamer", "RenderTally" }, frame = function() return ns.Streamer.tally end, how = "HandleFrame" },
}

-- The builders that add widgets to a window while it shows, after which its walk runs again (skipping what it has
-- handled), since no OnShow comes: the Talents tab's board on its tree's first showing (its scroll bar). One line
-- each; a kit window's draw ends in Kit.Fit, which walks it again (the step card's pooled rows and tiles).
local REWALKS = {
	{ after = { "Tree", "BuildBoard" }, frame = function() return _G.QuestPumpSheet end },
}

-- The widget types the walk hands to widget(), and the children it never enters (a tooltip and a model scene hold no
-- widget of the addon's). No status bar: ElvUI's HandleStatusBar strips a bar's own textures and paints its green, and
-- QuestPump's bars carry their own marks and colours (the sheet's XP bar over its rested twin, the virtual partner's
-- goal mark and colours, the stat bars).
local WIDGETS = { Button = true, EditBox = true, ScrollFrame = true }
local OPAQUE = { GameTooltip = true, ModelScene = true, Cooldown = true }

local S
local skinned = setmetatable({}, { __mode = "k" }) -- the windows and widgets handled, so none is handled twice

-- A close button: Blizzard's red X (UIPanelCloseButtonNoScripts' NormalTexture, Gethe forever
-- SharedUIPanelTemplates.xml, 70205).
local function isClose(b)
	local t = b.GetNormalTexture and b:GetNormalTexture()
	local atlas = t and t:GetAtlas()
	return atlas ~= nil and atlas:lower() == "redbutton-exit"
end

-- One widget by its type and template: a tab of a TabSystemTemplate (the Talents tab's trees, Tree.lua: its
-- TabSystemButtonMixin's GetTabID; it carries Left, Middle and Right too, so it is asked first, or it would be skinned
-- as a push button), a panel button (UIPanelButtonTemplate's Left, Middle and Right) or a grey stretch button
-- (UIMenuButtonStretchTemplate's nine pieces, TopLeft to BottomRight: the setup window's Back and Play without them;
-- Gethe forever SharedUIPanelTemplates.xml, 70374), both of which ElvUI's HandleButton strips (S.Blizzard.Regions,
-- ElvUI e7e5294), the red X, an edit box (a spinner's arrows with it), a scroll frame's bar (the old Slider of
-- UIPanelScrollFrameTemplate or the new MinimalScrollBar of ScrollFrameTemplate). Anything else stays as it is: the
-- kit's boxes and selects are its own drawing, and no check button or dropdown is left (the day-two check-in's joints
-- and the selects were the last, until 2026-10-16).
local function widget(w, kind)
	if kind == "Button" then
		if isClose(w) then
			S:HandleCloseButton(w)
		elseif w.GetTabID then
			S:HandleTab(w)
		elseif (w.Left and w.Middle and w.Right) or (w.TopLeft and w.BottomRight) then
			S:HandleButton(w)
		end
	elseif kind == "EditBox" then
		S:HandleEditBox(w)
		if w.DecrementButton then S:HandleNextPrevButton(w.DecrementButton, "left") end
		if w.IncrementButton then S:HandleNextPrevButton(w.IncrementButton, "right") end
	elseif kind == "ScrollFrame" and w.ScrollBar then
		if w.ScrollBar:GetObjectType() == "Slider" then
			S:HandleScrollBar(w.ScrollBar)
		else
			S:HandleTrimScrollBar(w.ScrollBar)
		end
	end
end

local function walk(f)
	for _, child in ipairs({ f:GetChildren() }) do
		local kind = child:GetObjectType()
		if WIDGETS[kind] and not skinned[child] then
			skinned[child] = true
			try("ElvUI " .. kind, widget, child, kind)
		end
		if not OPAQUE[kind] then walk(child) end
	end
end

local function skinWindow(f, how)
	if not f or skinned[f] then return end
	skinned[f] = true
	if how then try("ElvUI " .. how, S[how], S, f) end
	walk(f)
	f:HookScript("OnShow", walk)
end

local function skinWindows()
	for _, w in ipairs(WINDOWS) do skinWindow(w.frame(), w.how) end
	for f, how in pairs(ns.Kit.windows) do skinWindow(f, how or nil) end
end

-- Hooks a builder (`after`: an ns table and its function) with hooksecurefunc, never an edit.
local function follow(after, fn)
	local owner, method = after[1], after[2]
	if ns[owner] and ns[owner][method] then
		hooksecurefunc(ns[owner], method, fn)
	else
		ns.Log("W", "ElvUI skin: no %s.%s to follow", owner, method)
	end
end

-- Runs when ElvUI's Skins module calls the addon's skin: each builder hooked once, then what is built already.
local function skin()
	local hooked = {}
	for _, w in ipairs(WINDOWS) do
		local key = w.after[1] .. "." .. w.after[2]
		if not hooked[key] then
			hooked[key] = true
			follow(w.after, skinWindows)
		end
	end
	for _, r in ipairs(REWALKS) do
		follow(r.after, function()
			local f = r.frame()
			if f and skinned[f] and f:IsShown() then walk(f) end
		end)
	end
	-- the kit's windows as they are built, and again at the end of each render (its pooled rows)
	follow({ "Kit", "Window" }, skinWindows)
	follow({ "Kit", "Canvas" }, skinWindows)
	follow({ "Kit", "Fit" }, function(win)
		if skinned[win] and win:IsShown() then walk(win) end
	end)
	skinWindows()
end

-- ElvUI's Skins module (github.com/tukui-org/ElvUI, Game/Shared/Modules/Skins/Skins.lua, read 2026-10-05):
-- S:AddCallbackForAddon(addonName, name, func, forceLoad, bypass, position, toggle) runs `func` at the module's start
-- for an addon loaded by then, or at once with forceLoad, and a function as `toggle` decides whether it runs. At
-- PLAYER_LOGIN every addon that is not load-on-demand is loaded, so the order ElvUI and QuestPump load in does not
-- matter: before ElvUI's start the callback waits for it, after it runs at once. It follows ElvUI's Blizzard skins
-- switch (E.private.skins.blizzard.enable): the windows are Blizzard templates. ElvUI_WindTools skins third-party
-- addons its own way, so this file stays out of its way.
local function joinElvUI()
	if not loaded(ELVUI) or loaded(WINDTOOLS) then return end
	local E = type(ElvUI) == "table" and ElvUI[1]
	S = E and E.GetModule and E:GetModule("Skins", true)
	if not (S and S.AddCallbackForAddon) then
		S = nil
		return ns.Log("W", "ElvUI is loaded but its Skins module is not where this addon looks: no skin")
	end
	local function enabled()
		return E.private.skins.blizzard.enable == true
	end
	try("ElvUI AddCallbackForAddon", S.AddCallbackForAddon, S, ADDON, ADDON, skin, S.Initialized == true, nil, nil,
		enabled)
end

-- ConsolePort ---------------------------------------------------------------------------------------------------------

-- ConsolePort's cursor finds every window in UIPanelWindows and UISpecialFrames (ConsolePort_Cursor Database.lua,
-- env.FrameManagers, read 2026-10-05): the sheet, the card window, QuestPump's menu, the focus, import, QR and
-- session-card windows join it by themselves. The pump panel is in neither (Escape must never close a pump), nor are
-- the kit's windows with an Escape of their own (Kit.cursor: the setup screen, closed only by its answers), the
-- check-in card and QuestPump's dialogs (kept out of the game's sweeps: Popups.lua), so they join by name through
-- ConsolePort:AddInterfaceCursorFrame(frame), which waits for the load-on-demand cursor module itself
-- (EventUtil.ContinueOnAddOnLoaded; ConsolePort API.lua, 3.3.8). An unnamed frame cannot join (its Stack keys frames
-- by name). Called at PLAYER_LOGIN, after Panel.lua built the panel and the check-in, again when ConsolePort loads
-- later, and whenever the kit builds a window (Kit.Window: the setup's, the first time it shows); each frame joins
-- once.
local CURSOR = { "dim", "checkin" } -- QuestPumpPanel, QuestPumpCheckin (Panel.frames)
local cursorJoined = {}

local function joinCursor()
	local frames = ns.Panel and ns.Panel.frames
	if not (frames and frames.dim) or not loaded(CONSOLEPORT) then return end
	if not (type(ConsolePort) == "table" and ConsolePort.AddInterfaceCursorFrame) then
		if not warned.cursor then
			warned.cursor = true
			ns.Log("W", "ConsolePort without AddInterfaceCursorFrame: the pump panel is not on its cursor")
		end
		return
	end
	local list = {}
	for _, key in ipairs(CURSOR) do list[#list + 1] = frames[key] end
	for f in pairs(ns.Kit.cursor) do list[#list + 1] = f end
	for _, f in ipairs(ns.Dialog.frames) do list[#list + 1] = f end -- QuestPumpDialog1-3, built at load
	for _, f in ipairs(list) do
		if not cursorJoined[f] then
			cursorJoined[f] = true
			try("ConsolePort AddInterfaceCursorFrame", ConsolePort.AddInterfaceCursorFrame, ConsolePort, f)
		end
	end
end

ns.RegisterEvent("PLAYER_LOGIN", function()
	joinElvUI()
	joinCursor()
end)

hooksecurefunc(ns.Kit, "Window", joinCursor)

ns.RegisterEvent("ADDON_LOADED", function(name)
	if name == CONSOLEPORT then joinCursor() end
end)
