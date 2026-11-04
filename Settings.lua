-- The settings (docs/plan.md slice 3, "Settings"): Options > AddOns > QuestPump through the Settings API, a row that
-- only opens and closes the pages under it: General, Equipment, Exercises (the groups and their moves, a search, the
-- presets, the pool and its notes), Body (the pain flags and the switches, their only home), Social (who sees the
-- QuestPump card and what it carries, slice 13) and Data (profiles, export and import, a ladder started over or a
-- move's record cleared, reset, restore), and the export and feedback windows (slice 24); `/qp gear` is the Equipment
-- and Body pages' chat twin, `/qp export`, `/qp import` and `/qp reset` the Data page's, `/qp feedback` the General
-- page's "Send feedback". Every page is a canvas of the kit's (Kit.lua: Kit.Canvas, its framed lists, boxes, selects
-- and switches; the look of Your setup, whose equipment and switch builders live here) drawn from the live profile,
-- never proxy settings, so a profile switch takes effect at once. Every switch widens or narrows the pool of moves,
-- never the pump: the step jacks are the floor.

local ADDON, ns = ...
local E, L = ns.Enum, ns.L
local K = E.kind
local Kit, Style = ns.Kit, ns.Style

local Settings_ = {}
ns.Settings = Settings_
local F = {}
Settings_.frames = F

-- The switches /qp gear names besides the equipment, and the profile key each one flips: where the player trains, then
-- the body's (Your setup's two groups, Panel.lua; the Body page and /qp gear list all six in this order).
local SWITCHES = { floor = "floor", space = "space", quiet = "quiet", nojump = "noJump", nostrain = "noStrain",
	noheaddown = "noHeadDown" }
Settings_.SWITCHES = SWITCHES
Settings_.PLACE = { "floor", "space", "quiet" }
Settings_.BODY = { "nojump", "nostrain", "noheaddown" }
local SWITCH_ORDER = {}
for _, group in ipairs({ Settings_.PLACE, Settings_.BODY }) do
	for _, name in ipairs(group) do SWITCH_ORDER[#SWITCH_ORDER + 1] = name end
end
-- The switches of the one body every device shares (ns.BODY_SWITCHES): a change of one keeps its time (ns.BodyStamp).
local BODY = {}
for _, key in ipairs(ns.BODY_SWITCHES) do BODY[key] = true end
-- The game's setup is the place Home (slice 51): its equipment and its three switches (Settings_.PLACE) travel as one
-- field, `home`, whose change keeps its time too.
local HOME = {}
for _, name in ipairs(Settings_.PLACE) do HOME[SWITCHES[name]] = true end

-- A switch of the profile `p` the player changed: a Body one keeps the change's time, a place one Home's.
local function switched(p, key)
	if BODY[key] then ns.BodyStamp(p, key) end
	if HOME[key] then ns.BodyStamp(p, "home") end
end

-- The pool the filters leave: the moves and the patterns (the Exercises page warns below three patterns).
function Settings_.Pool(profile, state)
	local f = ns.FilterContext(profile, state)
	local moves, patterns, n = 0, {}, 0
	for id, m in pairs(ns.Library) do
		if ns.Passes(id, f) then
			moves = moves + 1
			if not patterns[m.pattern] then
				patterns[m.pattern] = true
				n = n + 1
			end
		end
	end
	return moves, n
end

-- A change to the profile: stamped (and each of `profiles` the change also wrote into, ns.Touch), and every view that
-- reads the pool redraws.
local function changed(profiles)
	ns.Touch(profiles)
	ns.Emit("Settings")
end

-- The setup's writers, for the Equipment and Body pages, /qp gear and Your setup (Panel.lua's setup window): a switch
-- by its /qp gear name (a Body one keeps its change's time), a piece of equipment, and any other key of the profile.
function Settings_.SetSwitch(name, on)
	local p = ns.profile
	if not p then return end
	p[SWITCHES[name]] = on and true or false
	switched(p, SWITCHES[name])
	changed()
end

function Settings_.SetGear(key, on)
	if not ns.profile then return end
	ns.profile.gear[key] = on and true or nil
	ns.BodyStamp(ns.profile, "home")
	changed()
end

function Settings_.Set(key, value)
	if not ns.profile then return end
	ns.profile[key] = value
	changed()
end

-- Whether a move or a group is unchecked on the Exercises page: in the exclusion list, or an opt-in move the player has
-- not checked (the headstand, slice 21; the profile's `include`).
function Settings_.Excluded(key)
	local m = type(key) == "number" and ns.Library[key]
	if m and m.optIn then return not ns.profile.include[key] end
	return ns.profile.exclude[key] == true
end

-- The Exercises page's switch, shared with the sheet's flyout: an excluded move leaves the pool like a pain flag, and
-- an opt-in move joins it only once checked.
function Settings_.SetExcluded(key, excluded)
	if not ns.profile then return end
	local m = type(key) == "number" and ns.Library[key]
	if m and m.optIn then
		ns.profile.include[key] = (not excluded) or nil
	else
		ns.profile.exclude[key] = excluded and true or nil
	end
	ns.BodyStamp(ns.profile, "moves")
	changed()
end

-- A favourite move (the sheet's move menu): the engine takes it first among the moves the day's rules leave.
function Settings_.SetFavourite(id, on)
	if not ns.profile then return end
	ns.profile.favourites[id] = on and true or nil
	changed()
end

-- The presets, applied on top of the current list.
Settings_.PRESETS = { "apartment", "office", "wrists", "knees", "everything" }

function Settings_.ApplyPreset(name)
	local p = ns.profile
	if not p then return end
	if name == "apartment" then
		if not p.noJump then switched(p, "noJump") end
		if not p.quiet then switched(p, "quiet") end
		p.quiet, p.noJump = true, true
	elseif name == "office" then
		if p.floor or p.space then switched(p, "floor") end
		p.floor, p.space = false, false
	elseif name == "wrists" or name == "knees" then
		local tag = name == "wrists" and "wrist" or "knee"
		for id, m in pairs(ns.Library) do
			for _, t in ipairs(m.pain) do
				if t == tag and id ~= ns.FLOOR_MOVE then p.exclude[id] = true end
			end
		end
		ns.BodyStamp(p, "moves")
	elseif name == "everything" then
		wipe(p.exclude)
		ns.BodyStamp(p, "moves")
	end
	changed()
end

-- The strength and hold ladders the filters leave no move for (they drop out of the rotation) and the ones the setup
-- stops below their top, each with what would open it (the top-out line, 2026-10-04; a pose family's is in its step's
-- card on the Talents tab and in chat, or the page would carry a dozen lines), the stats whose every kind of move is
-- unchecked (they stand still) and the Body page's switches that narrow the pool (the Apartment preset turns on two):
-- the Exercises page's warnings.
function Settings_.Warnings(profile, state)
	local f = ns.FilterContext(profile, state)
	local out = {}
	local on = {}
	for _, name in ipairs(SWITCH_ORDER) do
		-- Floor moves and Room to move widen the pool when on; the other four narrow it
		if name ~= "floor" and name ~= "space" and profile[SWITCHES[name]] == true then on[#on + 1] = L["gear." .. name] end
	end
	if #on > 0 then out[1] = L["settings.pool.body"]:format(table.concat(on, ", ")) end
	for _, l in ipairs(ns.TrialOrder) do
		local rung = ns.Rung(state, l)
		if not ns.RungMove(l, rung, f) then
			local shut, why = ns.TopOut(l, rung, f)
			out[#out + 1] = shut and L["settings.warn.ladder.open"]:format(L["ladder." .. l], ns.Sheet.Opens(shut, why))
				or L["settings.warn.ladder"]:format(L["ladder." .. l])
		else
			local at, shut, why = ns.TopOutAhead(state, l, f)
			if at then out[#out + 1] = ns.Sheet.TopOutLine(l, at, shut, why) end
		end
	end
	local kinds = {}
	for id in pairs(ns.Library) do
		if ns.Passes(id, f) then kinds[ns.MoveStat(id)] = true end
	end
	for _, stat in ipairs(ns.STATS) do
		if not kinds[stat] and stat ~= "spirit" then out[#out + 1] = L["settings.warn.stat." .. stat] end
	end
	return out
end

local function onOff(v)
	return L[v and "gear.on" or "gear.off"]
end

-- /qp gear: the room's three switches, the body's three, then the equipment on with its names and keys and the rest
-- by key alone (the name to switch one), the pain flags while one is set with their names and keys (the key to clear
-- one: "Low back" is back), and the pool.
function Settings_.Print()
	local p = ns.profile
	local function words(list) return #list > 0 and table.concat(list, ", ") or L["gear.none"] end
	ns.Print("cmd.gear.title")
	for _, group in ipairs({ { "cmd.gear.room", Settings_.PLACE }, { "cmd.gear.body", Settings_.BODY } }) do
		local parts = {}
		for _, name in ipairs(group[2]) do
			parts[#parts + 1] = L["gear." .. name] .. " (" .. name .. ") " .. onOff(p[SWITCHES[name]])
		end
		ns.Print(group[1], table.concat(parts, ", "))
	end
	local on, off = {}, {}
	for _, key in ipairs(ns.Gear) do
		if p.gear[key] then on[#on + 1] = L["gear." .. key] .. " (" .. key .. ")" else off[#off + 1] = key end
	end
	ns.Print("cmd.gear.equipment", words(on), words(off))
	local flags = {}
	for id, key in ipairs(E.jointKeys) do
		if ns.state.pain[id] then flags[#flags + 1] = L["joint." .. key] .. " (" .. key .. ")" end
	end
	if #flags > 0 then
		ns.Print("cmd.gear.pain", table.concat(flags, ", "))
	end
	local moves, patterns = Settings_.Pool(p, ns.state)
	ns.Print("cmd.gear.pool", moves, patterns)
end

-- /qp gear              the setup and the pool
-- /qp gear <name>       flips one piece of equipment or one switch
-- /qp gear clear <joint> clears a pain flag (the Body page's box)
ns.AddCommand("gear", function(arg)
	if not ns.db then return end
	local name, rest = strtrim(arg or ""):lower():match("^(%S*)%s*(%S*)")
	if name == "" then return Settings_.Print() end
	local p = ns.profile
	if name == "clear" then
		local joint = E.joint[rest]
		if not joint then return ns.Print("cmd.gear.unknown", rest) end
		if not ns.state.pain[joint] then return ns.Print("cmd.gear.notFlagged", L["joint." .. rest]) end
		ns.AddRow({ move = joint, kind = K.pain, reps = 0 })
		ns.Print("cmd.gear.cleared", L["joint." .. rest])
		return
	end
	local value
	if SWITCHES[name] then
		value = not p[SWITCHES[name]]
		Settings_.SetSwitch(name, value)
	else
		local known = false
		for _, key in ipairs(ns.Gear) do known = known or key == name end
		if not known then return ns.Print("cmd.gear.unknown", name) end
		value = not p.gear[name]
		Settings_.SetGear(name, value)
	end
	local moves, patterns = Settings_.Pool(p, ns.state)
	ns.Print("cmd.gear.set", L["gear." .. name], onOff(value), moves, patterns)
	if patterns < 3 then ns.Print("cmd.gear.few", patterns) end
end, "cmd.gear.help")

-- One setting, one look (decision G5, 2026-10-16) --------------------------------------------------------------------

-- Your setup (Panel.lua's setup window) and the Settings pages ask the same things with the same builders: the
-- equipment's picture list (Your setup's "What do you have at hand?" and the Equipment page) and the switches' rows
-- (Your setup's Body list and the Body page's room and body), each its picture, name and box.

-- The key pose that stands for each switch, by library key: the room's tiles in Your setup and its rows on the Body
-- page, the body's rows in both.
Settings_.PICTURE = { floor = "plank", space = "skater_hops", quiet = "air_squat", nojump = "jumping_jacks",
	nostrain = "plank", noheaddown = "downward_dog" }

-- A switch of the profile (Floor moves, Room to move, Quiet mode, the Body switches): on or off as the profile says.
function Settings_.SwitchOn(name)
	return ns.profile[SWITCHES[name]] == true
end

-- The switches `names` as rows of `list` (Kit.Row): each its key pose, its name and its box (what it leaves out is
-- its tooltip's, never a line under the name: David, 2026-10-22), a click anywhere on it flipping the switch;
-- opts.font, the name's (the list row's without); opts.care, the tick alone carrying "on" with the name white
-- (decision G4: a Settings page's list); opts.hint, the box's click hint last in the tooltip.
-- Settings_.RenderSwitchRows draws them from the profile and answers the list's height.
function Settings_.BuildSwitchRows(list, names, opts)
	list.switchCare = opts.care == true
	for _, name in ipairs(names) do
		local hint = opts.hint and { hint = function() return Kit.BoxHint(Settings_.SwitchOn(name)) end } or nil
		local row = Kit.Row(list, { picture = Kit.MoveByKey(Settings_.PICTURE[name]), name = L["gear." .. name],
			font = opts.font, box = true, onClick = function()
				local on = not Settings_.SwitchOn(name)
				Style.Play(on and "checkOn" or "checkOff")
				Settings_.SetSwitch(name, on)
			end, tip = Kit.Tip(L["gear." .. name], L["settings.switch." .. name .. ".tip"], hint) })
		row.switch = name
	end
end

function Settings_.RenderSwitchRows(list)
	for _, row in ipairs(list.rows) do Kit.Choose(row, Settings_.SwitchOn(row.switch), list.switchCare) end
	return Kit.LayList(list)
end

-- What you have at hand: the pieces of ns.Gear on `view`, `w` units across, in ns.GearGroups' five groups, each under
-- its caption (Kit.Section, settings.gear.group.<key>; Rapha, 2026-11-03), each piece a picture of a move that uses
-- it, its name on up to three lines (ui.setup.gearName.<key>, a short one, where gear.<key>'s explanation would not
-- fit) and the check box, its tooltip the piece's (Settings_.GearTip: the whole name, what counts as it and what it
-- would open); within a group in three columns down the columns, or two where any name's longest word is wider than a
-- third's name column (a large Text Size: a word is never broken), every group in the same columns and rows of one
-- height. Settings_.RenderGear lays them out from `y` and answers the bottom.
local GEAR_COLS = 3
local GEAR_ROOM = 4 + 44 + 10 + 40 -- a tile's width besides its name: the picture, the gaps and the check box

local function gearPiece(view, key)
	local c = Kit.Choice(view, 1, 40, function()
		local on = not ns.profile.gear[key]
		Style.Play(on and "checkOn" or "checkOff")
		Settings_.SetGear(key, on)
	end)
	Settings_.GearTip(c.button, key)
	c.screen = Kit.Screen(c, 44, 32)
	c.screen:SetPoint("LEFT", 4, 0)
	c.icons = { Kit.Icon(c.screen, Kit.GearMove(key), 34) }
	c.label = rawget(L, "ui.setup.gearName." .. key) or L["gear." .. key]
	c.name = Kit.Text(c, "bodyWhite", "LEFT", nil, 1)
	c.name:SetPoint("LEFT", c.screen, "RIGHT", 10, 0)
	c.name:SetMaxLines(3)
	c.name:SetText(c.label)
	c.box = Kit.Box(c, 22)
	c.box:SetPoint("RIGHT", -10, 0)
	c.gear = key
	return c
end

function Settings_.BuildGear(view, w)
	view.gearW, view.pieces, view.gearGroups = w, {}, {}
	view.measure = Kit.Text(view, "bodyWhite", "LEFT") -- one word at a time, never shown (Settings_.RenderGear)
	view.measure:Hide()
	for _, group in ipairs(ns.GearGroups) do
		local g = { head = Kit.Section(view, L["settings.gear.group." .. group.key]), pieces = {} }
		for _, key in ipairs(group.gear) do
			local c = gearPiece(view, key)
			g.pieces[#g.pieces + 1] = c
			view.pieces[#view.pieces + 1] = c
		end
		view.gearGroups[#view.gearGroups + 1] = g
	end
end

function Settings_.RenderGear(view, y)
	local cols = GEAR_COLS
	local nameW = (view.gearW - (cols - 1) * 10) / cols - GEAR_ROOM
	for _, c in ipairs(view.pieces) do
		for word in c.label:gmatch("%S+") do
			view.measure:SetText(word)
			if view.measure:GetUnboundedStringWidth() > nameW then cols = GEAR_COLS - 1 end
		end
	end
	local w = (view.gearW - (cols - 1) * 10) / cols
	local rowH = 40
	for _, c in ipairs(view.pieces) do
		c:SetWidth(w)
		c.name:SetWidth(w - GEAR_ROOM)
		Kit.Choose(c, ns.profile.gear[c.gear] == true)
		rowH = math.max(rowH, c.name:GetStringHeight() + 12)
	end
	-- each group's caption, its pieces 14 under it down the first column, then the next, the next caption 16 under
	-- the group above
	for i, g in ipairs(view.gearGroups) do
		if i > 1 then y = y + 10 end
		g.head:SetWidth(view.gearW)
		Kit.At(g.head, view, y)
		y = y + 26
		local rows = math.ceil(#g.pieces / cols)
		for j, c in ipairs(g.pieces) do
			local col, row = math.floor((j - 1) / rows), (j - 1) % rows
			c:SetHeight(rowH)
			Kit.At(c, view, y + row * (rowH + 6), col * (w + 10))
		end
		y = y + rows * (rowH + 6)
	end
	return y
end

-- Profiles ------------------------------------------------------------------------------------------------------------

local function guid()
	return ns.Readable(UnitGUID("player"))
end

-- The profile names in the save, Default first.
function Settings_.ProfileNames()
	local names = {}
	for name in pairs(ns.db.profiles) do
		if name ~= "Default" then names[#names + 1] = name end
	end
	table.sort(names)
	table.insert(names, 1, "Default")
	return names
end

function Settings_.CurrentProfile()
	return ns.ProfileName(ns.db, guid())
end

-- A pact and a duel go with the character from the profile it leaves to the one it takes (slice 23), so the sworn
-- character's card keeps carrying them and a partner's or a rival's name is stored once. Where the profile taken has
-- its own, the one left behind ends while it is still the profile in use (its partner and rivals told: Social.EndPact,
-- Social.LeaveDuel), so no name stays behind with no one to end it; a settled duel's score (no name) stays only with
-- the profile taken.
local STAKES = { "pact", "duel", "duelPast" }

local function carryStakes(from, to)
	if from.pact and to.pact then ns.Social.EndPact() end
	if from.duel and to.duel then ns.Social.LeaveDuel() end
	for _, key in ipairs(STAKES) do
		if from[key] and not to[key] then to[key] = from[key] end
		from[key] = nil
	end
end

-- This character uses the named profile from now on (Default when nil).
function Settings_.UseProfile(name)
	if not ns.db or ns.readOnly or not guid() then return end
	local from = ns.profile
	ns.db.profileKeys[guid()] = name ~= "Default" and name or nil
	local to = ns.ProfileFor(ns.db, guid())
	if from and from ~= to then carryStakes(from, to) end
	ns.profile = to
	if ns.HUD and ns.HUD.frame then ns.HUD.Place(ns.HUD.frame) end
	changed(from and { from } or nil) -- the profile left gave up its pact and duel
	-- the setup window follows the profile, and a paired player's start comes with one whose Your setup is answered
	ns.Setup.Evaluate()
end

-- The name of this character's own profile ("Name - Realm").
function Settings_.OwnProfile()
	local char = ns.db and ns.db.chars[ns.charIndex]
	return L["settings.profile.char"]:format(char and char.name or "?", char and char.realm or "?")
end

-- A profile of this character's own: a copy of the one in use, named after the character; a pact and a duel are not
-- copied but go with the character (UseProfile, carryStakes), so a name is never stored twice.
function Settings_.PerCharacter(on)
	if not ns.db or ns.readOnly or not guid() then return end
	if not on then return Settings_.UseProfile("Default") end
	local name = Settings_.OwnProfile()
	if not ns.db.profiles[name] then
		local copy = CopyTable(ns.profile)
		for _, key in ipairs(STAKES) do copy[key] = nil end
		ns.db.profiles[name] = copy
	end
	Settings_.UseProfile(name)
end

-- Reset and restore ---------------------------------------------------------------------------------------------------

-- What a reset of the profile's settings keeps: the Body page's six switches (the room and the body the player trains
-- with: floor, space, quiet, no jumping, no breath-hold strain, no head-down positions), the exclusion list, the
-- opt-in moves checked (with when each Body field changed, `bodyAt`: one body everywhere), the favourites and who sees
-- the card, so a reset never shares a card the player keeps private (the Body and Social pages are canvases for the
-- same reasons: no Defaults button changes them); the pact, the duel and the duels won (slice 23), which are no
-- settings: a reset never ends them by surprise; and that Your setup was answered, so a settings reset never brings
-- the setup window's questions back (/qp setup asks them at any time).
local KEPT = { "floor", "space", "quiet", "noJump", "noStrain", "noHeadDown", "exclude", "include", "bodyAt",
	"favourites", "share", "pact", "duel", "duelPast", "duelWins", "setupAnswered" }

-- One move's record cleared or one ladder started over (David, 2026-10-08: "maybe its good if we cna reset specific
-- excericises in the settings and delte the records"): offered for a move with a record (ns.HoldsRecord) and for a
-- strength ladder that is open, never cardio or a pose family (CanRestart; a pose family has no record to clear);
-- refused in a fight and on a save of a newer addon. While a pump is open too (David, 2026-10-14: "it should not be
-- blocked during the pump. that's exactly when you need it"; refused until then): the pump's moves it took aim again
-- (Events.lua Lc.Cleared). A clear row the fold reads forward
-- (DB.lua clearRow), the account's, so the helper passes it on to the app and the other game installs.
function Settings_.CanRestart(ladder)
	return ns.STRENGTH[ladder] and ns.Open(ns.state, ladder) or false
end

-- In a fight: Lc.combat (Events.lua sets it at PLAYER_REGEN_DISABLED before the handlers that draw these pages run,
-- and the lockdown only begins after that event: warcraft.wiki.gg API_InCombatLockdown), or the lockdown itself. A
-- control that opens a window or writes a row greys then with "Not in combat." as its reason (fightReason), as its
-- click refuses.
local function fighting()
	return ns.Lifecycle.combat or InCombatLockdown()
end

local function fightReason()
	return fighting() and L["chat.combat"] or nil
end

-- Why a record cleared, a ladder started over or a count corrected is refused now: a save of a newer addon, a fight.
local function writeRefused()
	if ns.readOnly then return "import.readonly" end
	if fighting() then return "chat.combat" end
	return nil
end

local function moveName(id)
	return L["move." .. ns.Library[id].key]
end

local function writeClear(what, id)
	local why = writeRefused()
	if why then return ns.Print(why) end
	local row
	if what == "move" then
		ns.Log("I", "reset: the record of %s", ns.Library[id].key)
		row = ns.AddRow({ kind = K.clear, reps = E.clear.move, move = id })
		ns.Print("cmd.reset.move", moveName(id))
	else
		ns.Log("I", "reset: the %s ladder started over", ns.Ladders[id].key)
		row = ns.AddRow({ kind = K.clear, reps = E.clear.ladder, ladder = id })
		ns.Print("cmd.reset.ladder", L["ladder." .. id])
	end
	ns.Lifecycle.Cleared(row)
	ns.Emit("DB")
end

-- The sets of move `id` Correct a count's window lists (ns.Correctable on the game's own log, which needs no horizon),
-- kept until the log, the fold or the game day changes: a menu, a tooltip and the window read the log once between
-- two rows.
local listed = {}

local function correctable(id)
	local db, state = ns.db, ns.state
	local log = db and db.log
	if not (log and state) then return {} end
	local now = ns.now()
	local day = ns.day(now)
	if listed.state ~= state or listed.n ~= #log or listed.last ~= log[#log] or listed.day ~= day then
		listed = { state = state, n = #log, last = log[#log], day = day, moves = {} }
	end
	local list = listed.moves[id]
	if not list then
		list = ns.Correctable(state, db, id, now)
		listed.moves[id] = list
	end
	return list
end

-- Whether move `id` offers Correct a count (slice 44, 2026-10-18; docs/plan.md "One correction, from any device"): it
-- holds a record (its window lists the record's set, or says it is older and offers the clear), or the window lists a
-- set of it (a pose's answer, a counted form's count with no record yet).
function Settings_.CanCorrect(id)
	return ns.HoldsRecord(ns.state, id) or #correctable(id) > 0
end

-- A move's record items in a menu: "Correct a count..." (its window, Settings_.CorrectWindow) and, while it has a
-- record, "Clear this move's record" under it, which asks first; nothing for a move without either. In a menu of their
-- own (Settings_.RecordMenu: the Exercises page's and the Records tab's right-click) or under a divider in another (the
-- sheet's muscle flyout and the Talents tab's move tile, Sheet.FlyoutMenu).
function Settings_.AddRecordItems(root, id, divider)
	if not Settings_.CanCorrect(id) then return end
	if divider then root:CreateDivider() end
	root:CreateButton(L["menu.correct"], function() Settings_.CorrectWindow(id) end)
	if ns.HoldsRecord(ns.state, id) then
		root:CreateButton(L["menu.clear"], function() Settings_.ConfirmReset("move", id) end)
	end
end

function Settings_.RecordMenu(owner, id)
	if not Settings_.CanCorrect(id) then return end
	ns.Menu.Open(owner, function(_, root)
		root:CreateTitle(moveName(id))
		Settings_.AddRecordItems(root, id)
	end)
end

-- The hint under a move's tooltip where a right-click opens those items.
function Settings_.RecordHint(tooltip, id)
	if not Settings_.CanCorrect(id) then return end
	GameTooltip_AddInstructionLine(tooltip,
		L[ns.HoldsRecord(ns.state, id) and "tip.move.correctClear" or "tip.move.correct"])
	tooltip:Show()
end

-- A ladder's menu (a column header's right-click on the Talents tab, Tree.lua).
function Settings_.RestartMenu(owner, ladder)
	if not Settings_.CanRestart(ladder) then return end
	ns.Menu.Open(owner, function(_, root)
		root:CreateTitle(L["ladder." .. ladder])
		root:CreateButton(L["menu.restart"]:format(L["ladder." .. ladder]),
			function() Settings_.ConfirmReset("ladder", ladder) end)
	end)
end

-- "char": the settings of the profile in use back to the defaults, except what KEPT names (every character on that
-- profile shares it; the progress stays); "all": every row, record and setting gone, the save kept as the backup
-- /qp restore-backup brings back, and the helper's older copy never restored over it; "restore": the backup swapped in;
-- "move" and "ladder": the record of the move `id` cleared, the ladder `id` started over (above).
function Settings_.Reset(what, id)
	local db = ns.db
	if not db or ns.readOnly then return end
	if what == "move" or what == "ladder" then return writeClear(what, id) end
	ns.Log("I", "reset: %s", what)
	if what == "restore" then return ns.RestoreBackup() end
	if what == "char" then
		local p = ns.profile
		local keep = {}
		for _, key in ipairs(KEPT) do keep[key] = p[key] end
		wipe(p)
		for k, v in pairs(CopyTable(ns.defaults)) do p[k] = v end
		for key, v in pairs(keep) do p[key] = v end
		if ns.HUD and ns.HUD.frame then ns.HUD.Place(ns.HUD.frame) end -- the HUD's place went with the settings
		changed()
		ns.Print("cmd.reset.char", Settings_.CurrentProfile())
		return
	end
	ns.Lifecycle.Drop()
	ns.ResetSave(db, ns.now())
	ns.BindDB()
	-- the new save's first row says so: the strip paints it (slice 5), and the helper drops every row before it from
	-- its mirror before the next /reload loads that mirror, which on the beta is the only copy of the save
	ns.AddRow({ ts = db.meta.resetAt, kind = ns.Enum.kind.reset })
	ns.Print("cmd.reset.all")
	ns.Print("status.reset.everywhere")
	ns.Setup.Restart() -- the profiles went too: Your setup asks again, and the new start comes when it is finished
end

-- The popup names what goes: the profile a settings reset touches, what a restore swaps, the move's record or the
-- ladder and what stays, and under it in the notes' grey what reaches beyond this game: a reset of it all reaches the
-- app and the other game installs, and so does a record cleared or a ladder started over while the helper runs; a
-- restore after a reset brings back this game's save only (slice 26: the helper's floor keeps the rows from before it
-- out). Its buttons: Keep it, and the action by its name. Asked over an open pump (its Settings, /qp reset), the popup
-- holds the pump as every popup does, so the pump's dim never covers it.
local ACCEPT = { all = "settings.reset.all", char = "settings.reset.charButton", restore = "settings.restore.button",
	move = "ui.reset.do.move", ladder = "ui.reset.do.ladder" }

local function ask(line, notes, data)
	data.note = #notes > 0 and table.concat(notes, " ") or nil
	if ns.Dialog.Show("QUESTPUMP_RESET", line, nil, data, { accept = L[ACCEPT[data.what]] }) then
		ns.Lifecycle.Popup()
	end
end

-- What reaches beyond this game, said before a row of the account's is written (a record cleared or a ladder started
-- over under its popup, a count corrected over its Save): the words of key `everywhere` while the helper runs, which
-- passes the row on to the app and the other game installs (a confirmation's consequence; none for a count corrected,
-- which asks nothing: `everywhere` nil); on a client that showed it loads no save the words of `beta`, since the row
-- outlasts /reload only through the helper's copy; else none.
local function reachNote(everywhere, beta)
	if ns.HelperState() == "on" then return everywhere and L[everywhere] end
	if ns.Setup.SaveUnread() and type(QuestPumpMirror) == "table" then return L[beta] end
	return nil
end

function Settings_.ConfirmReset(what, id)
	local line
	if what == "move" or what == "ladder" then
		local why = writeRefused()
		if why then return ns.Print(why) end
		line = what == "move" and L["ui.reset.move"]:format(moveName(id))
			or L["ui.reset.ladder"]:format(L["ladder." .. id])
		ask(line, { reachNote("status.clear.everywhere", "status.clear.beta") }, { what = what, id = id })
		return
	end
	line = what == "char" and L["ui.reset.char"]:format(Settings_.CurrentProfile()) or L["ui.reset." .. what]
	local notes = {}
	if what == "all" then
		notes[1] = L["status.reset.everywhere"]
	elseif what == "restore" and ns.meta and ns.meta.resetAt then
		notes[1] = L["status.restore.here"]
	end
	-- on a client that showed it loads no save, the helper's copy is the save: a reset outlasts /reload only while it
	-- runs, and a restore not at all (that copy holds no backup, and the helper dropped the rows from before the reset)
	local beta = ns.Setup.SaveUnread() and type(QuestPumpMirror) == "table"
	if what ~= "char" and beta then
		notes[#notes + 1] = what == "all" and L["status.reset.beta"] or L["status.restore.beta"]
	end
	ask(line, notes, { what = what })
end

ns.Dialog.Register("QUESTPUMP_RESET", {
	text = L["ui.reset.confirm"],
	note = function(data) return data and data.note end,
	button1 = L["settings.reset.all"], -- each show names its own (ACCEPT)
	button2 = L["ui.reset.keep"],
	OnAccept = function(_, data) Settings_.Reset(data.what, data.id) end,
})

-- The move with a record or the ladder the player can start over that `words` names: the one whose whole name it is,
-- else the one name that holds the words (any case); else nil and the names that hold them (none, or several).
local function named(what, words)
	local want, list = words:lower(), {}
	if what == "move" then
		for _, group in ipairs(ns.RecordedMoves(ns.state)) do
			for _, id in ipairs(group.moves) do list[#list + 1] = { id = id, name = moveName(id) } end
		end
	else
		for _, l in ipairs(ns.LADDER_ORDER) do
			if Settings_.CanRestart(l) then list[#list + 1] = { id = l, name = L["ladder." .. l] } end
		end
	end
	local found, names = nil, {}
	for _, e in ipairs(list) do
		local name = e.name:lower()
		if name == want then return e.id end
		if name:find(want, 1, true) then
			found = e.id
			names[#names + 1] = e.name
		end
	end
	if #names == 1 then return found end
	return nil, names
end

-- /qp reset char|all behind the popup (it names what goes), and /qp reset move <name> and /qp reset ladder <name>.
ns.AddCommand("reset", function(arg)
	local what, words = strtrim(arg or ""):match("^(%S*)%s*(.-)$")
	what = what:lower()
	if (what == "char" or what == "all") and words == "" then return Settings_.ConfirmReset(what) end
	if (what ~= "move" and what ~= "ladder") or words == "" then return ns.Print("cmd.reset.usage") end
	local id, names = named(what, words)
	if id then return Settings_.ConfirmReset(what, id) end
	if #names > 1 then return ns.Print("cmd.reset.several", words, table.concat(names, ", ")) end
	ns.Print(what == "move" and "cmd.reset.noMove" or "cmd.reset.noLadder", words)
end, "cmd.reset.help")

-- Correct a count (slice 44, 2026-10-18) ------------------------------------------------------------------------------

-- Why a step stays where it is after a correction (Lc.WriteCorrection's `stays`): an app's set, whose steps only the
-- app changes; a step change of its character's since the set. The window's line, and /qp undo's (Events.lua).
Settings_.STAYS = { source = "correct.step.source", later = "correct.step.later" }

-- What a correction did to the steps, as the done view and /qp undo's lines say it (2026-10-19, the program's
-- double-check: a climb, a stretch check's lift or a Mastered step went back with no word): each ladder whose step
-- moved (Lc.WriteCorrection's `changes`, ns.CorrectionRows' third: "Push back to step 6.", "Push up to step 7."), each
-- step whose seal changed ("Step 7 of Core is no longer Mastered."), then why a step stays (`stays`, Settings_.STAYS).
-- Each line a locale key and its arguments, { key, ... }: the done view formats them, /qp undo prints them (ns.Print).
function Settings_.StepLines(stays, changes)
	local out = {}
	for _, x in ipairs(changes and changes.rungs or {}) do
		out[#out + 1] = { x.to < x.from and "correct.step.back" or "correct.step.up", L["ladder." .. x.ladder],
			ns.Step(x.ladder, x.to) }
	end
	for _, x in ipairs(changes and changes.mastered or {}) do
		out[#out + 1] = { x.won and "correct.mastered.won" or "correct.mastered.lost", ns.Step(x.ladder, x.rung),
			L["ladder." .. x.ladder] }
	end
	if stays then out[#out + 1] = { Settings_.STAYS[stays] } end
	return out
end

-- A pose answer's word (ns.Enum.mark), a set's value and the name of its row in a pose's correction, from Perfect
-- down: the answer's words alone, as on every surface (David, 2026-10-22: no second labels).
local MARK_WORD = { [E.mark.perfect] = "correct.mark.perfect", [E.mark.good] = "correct.mark.good",
	[E.mark.okay] = "correct.mark.okay", [E.mark.notyet] = "correct.mark.notyet" }

-- A set's value of move `id` as the player reads it (`kind` ns.CorrectableRow's): a pose's answer in its short word; a
-- counted form's count in its window ("12 in 30 s"), "Not counted" without one; a hold's seconds; a rep set's reps,
-- with its weight on a loaded move (`load` tenths of a kg: "11 kg × 16", Sheet.LoadAmount); a rep or hold set at 0 in
-- the count step's words by its `outcome` as the fold reads it (ns.SetOutcome, never the stored row's): done "Done
-- without a count" (saved with an empty box, or a count corrected to 0, which folds as typing 0 does: slice 44's
-- review, 2026-10-18), a stored miss or placement "Could not do one" or "Could not hold it". The window's rows and
-- lines and a correction's chat line (/qp undo's).
function Settings_.SetWords(id, kind, value, load, outcome)
	if kind == "answer" then return L[MARK_WORD[value]] end
	local m = ns.Library[id]
	if m.count then
		return value > 0 and L["unit.countIn"]:format(value, ns.C.count_s) or L["ui.result.cleanNone"]
	end
	local hold = m.dose == "hold"
	if value == 0 then
		if outcome == E.outcome.done then return L["ui.result.empty"] end
		return L[hold and "ui.result.couldNotHold" or "ui.result.couldNot"]
	end
	if hold then return L["unit.seconds"]:format(value) end
	if load and load > 0 then return ns.Sheet.LoadAmount(id, load, value) end
	return ns.Plural("log.reps", value):format(value)
end

-- Set row `row` (as stored) of move `id` as the fold reads it now (`kind` ns.CorrectableRow's), in Settings_.SetWords:
-- its value, weight and outcome on ns.state (DB.lua ns.SetValue, ns.SetLoad, ns.SetOutcome), never the count typed
-- into a correction, which the fold may read otherwise (a capped max set corrected over its window's top keeps the
-- top; a count corrected to 0 is done). A correction's done view and chat line, the window's and /qp undo's
-- (Events.lua), so they say what the window's list shows next (slice 44's review, 2026-10-18).
function Settings_.FoldedWords(id, kind, row)
	local s = ns.state
	return Settings_.SetWords(id, kind, ns.SetValue(s, row), ns.SetLoad(s, row), ns.SetOutcome(s, row))
end

-- The window (QuestPumpCorrect: "Correct a count..." in a move's menu, Settings_.CorrectWindow), the kit's medium
-- window, one view at a time (the phones' correct.window in the game's look, docs/ui.md "The game's look: one kit"):
-- - the list: the move's sets of the last correct_days game days the save holds (ns.Correctable: the newest
--   correct_sets, the record's set where older), each a row with its value as the fold reads it (Settings_.SetWords),
--   its day and time and Record on the record's own; where the move holds a record whose set is not listed, the line
--   that says so and "Clear this move's record" grey at the bar's left (the reset dialog asks first); with no set, the
--   line that says so; no line over the list (the title names the move, the section the days);
-- - a set: what it saved and when (a pose's answer, and a value with no number, a 0 or a counted form's none, each
--   under a heading of its own: slice 44's review), the count step's question, its input (the rep box, or the spinner
--   with a gamepad or when chosen, Panel.InputMode; the weight box beside it on a loaded set; a pose's four answers as
--   rows, their words alone), what Save keeps under it once the input holds a number (the question asks a one-sided
--   move's weaker side), and on a beta client what Save needs to outlast /reload (reachNote); Save, the one red,
--   greyed until the value, the weight or the answer differs and while the count is past one set's limit, and "Pick
--   another set" grey at the left. On a game set another device corrected, whose game side reads it otherwise (the
--   entry's ownSide, 2026-10-19), Save stands ready at the set as it reads now, with the line that says so in the
--   preview's place (correctResave).
--   Save asks the count step's "Is that right?" once where ns.CorrectionCheck asks (against the state before the set:
--   its record, its weight, its window's top), in the preview's place with its two grey answers and no red, then
--   writes the rows (Lc.WriteCorrection) and prints the chat line;
-- - done: "Corrected to ..." (the set as the fold reads it, Settings_.FoldedWords; "Corrected to no count." where it
--   reads with no number), with what it did to the steps or why a step stays where it is (Settings_.StepLines), and
--   Done.
-- A set that left the window meanwhile (its days ran out at 04:00, a clear came, or Save found nothing to write) goes
-- back to the list with a red line under it that says so. Refused in a fight and on a save of a newer addon, as a
-- clear is; allowed while a pump is open, whose moves of that move with no result yet are dosed again (Lc.Corrected),
-- and the pump stays hidden while the window shows, as under the export window (Events.lua MENU_WINDOWS: under the dim
-- its box would take the card's keys). The game's Escape closes it (UISpecialFrames), and so does a fight.
local BOX_W, BOX_H = 120, 60 -- the count step's rep box (Panel.lua)
-- the weight box at the template's own font and size (Panel.lua buildLoad: the rep box's font crashed the client in
-- it), and how far left of the centre the count's input stands beside it: the rep box, the spinner
local KG_W, KG_H, KG_SHIFT_BOX, KG_SHIFT_SPIN = 50, 20, 70, 96

-- Set `set`'s entry on the window's list of move `id` now (DB.lua ns.IsSet, the fold's set identity), nil once it
-- left the window.
local function listedSet(id, set)
	for _, e in ipairs(correctable(id)) do
		if ns.IsSet(e.row, set) then return e end
	end
	return nil
end

-- The count in the set's input (nil while the rep box is empty), its weight on a loaded set (tenths of a kg, 0 for
-- none; nil on any other set), and whether Save has anything to write.
local function correctValue(f)
	if f.st.typed then
		if f.box:GetText() == "" then return nil end
		return f.box:GetNumber()
	end
	return f.spin:GetValue()
end

local function correctLoad(f)
	if not f.st.loaded then return nil end
	return ns.Sheet.ParseKg(f.kg:GetText()) or 0
end

-- Whether the input differs from the set as the fold reads it (the entry's value, weight or answer).
local function correctDiffers(f)
	local e = f.st.entry
	if e.kind == "answer" then return f.st.answer ~= nil and f.st.answer ~= e.value end
	local value = correctValue(f)
	if value == nil then return false end
	return value ~= e.value or (f.st.loaded and correctLoad(f) ~= (e.load or 0)) or false
end

-- Whether Save stands ready at the set as it reads now (2026-10-19, the program's double-check: the game could not act
-- on a phone's correction of its set, whose step only the game asks again): another device's correction of a game set
-- won and the game's own side reads it otherwise (the entry's ownSide, DB.lua ns.OwnSide), and the input still holds
-- the set as it reads now; Save then writes it again as the game's own (a new round of the same value, which asks the
-- step rules again: ns.CorrectionRows). An app's set is its app's to save again, never the game's.
local function correctResave(f)
	local e = f.st.entry
	if not e.ownSide or ns.Source(e.row.flags) == E.source.app or correctDiffers(f) then return false end
	if e.kind == "answer" then return f.st.answer == e.value end
	return correctValue(f) ~= nil
end

local function correctChanged(f)
	return correctDiffers(f) or correctResave(f)
end

-- Past one set's limit: the count (ns.CountMax, the box's own) or the weight (load_max_kg), ns.Validate's "max".
local function overLimit(f, value, load)
	return value > ns.CountMax(f.st.slot) or (load ~= nil and load > ns.C.load_max_kg * 10)
end

-- The count's input on its holder: centred, or on a loaded set left of the centre with the weight box beside it
-- (beside the rep box, or between - and + beside the spinner), as the count step places it (Panel.lua placeInputs);
-- placeKg places the weight box and its label.
local function placeKg(f, boxes)
	local kg = f.kg
	for _, part in ipairs({ kg, kg.label, kg.down, kg.up }) do part:ClearAllPoints() end
	kg.label:SetText(L[ns.Loaded(f.st.move) == "each" and "ui.result.kgEach" or "ui.result.kg"])
	if boxes then
		kg:SetPoint("LEFT", f.box, "RIGHT", 16, 0)
		kg.label:SetPoint("LEFT", kg, "RIGHT", 6, 0)
	else
		kg.down:SetPoint("LEFT", f.spin.IncrementButton, "RIGHT", 24, 0)
		kg:SetPoint("LEFT", kg.down, "RIGHT", 10, 0)
		kg.up:SetPoint("LEFT", kg, "RIGHT", 4, 0)
		kg.label:SetPoint("LEFT", kg.up, "RIGHT", 6, 0)
	end
end

local function placeCorrectInputs(f)
	local kg, boxes, loaded = f.kg, f.st.typed, f.st.loaded
	local shift = loaded and (boxes and KG_SHIFT_BOX or KG_SHIFT_SPIN) or 0
	f.box:ClearAllPoints()
	f.box:SetPoint("TOP", f.inputs, "TOP", -shift, 0)
	f.spin:ClearAllPoints()
	f.spin:SetPoint("TOP", f.inputs, "TOP", -shift, -(BOX_H - f.spin:GetHeight()) / 2)
	f.box:SetShown(boxes)
	f.spin:SetShown(not boxes)
	if loaded then placeKg(f, boxes) end
	kg:SetShown(loaded)
	kg.label:SetShown(loaded)
	kg.down:SetShown(loaded and not boxes)
	kg.up:SetShown(loaded and not boxes)
	local pair = loaded and boxes -- Tab and Shift-Tab between the two boxes (InputBoxTemplate's nextEditBox)
	f.box.nextEditBox, f.box.previousEditBox = pair and kg or nil, pair and kg or nil
	kg.nextEditBox, kg.previousEditBox = pair and f.box or nil, pair and f.box or nil
end

-- Save: the set still in the window, "Is that right?" once where the check asks (`confirmed` its Yes), then the rows,
-- the chat line and the done view; back to the list where the set left the window or nothing was written.
local function saveCorrection(confirmed)
	local f = F.correct
	local e = f and f.st.entry
	if not (e and correctChanged(f)) then return end
	local why = writeRefused()
	if why then return ns.Print(why) end
	if not listedSet(f.st.move, e.row) then return Settings_.CorrectWindow(f.st.move, e.row) end
	local value, load, answer
	if e.kind == "answer" then
		answer = f.st.answer
	else
		value, load = correctValue(f), correctLoad(f)
		if overLimit(f, value, load) then return end -- the preview says so
		if not confirmed then
			local check, before, slot = ns.CorrectionCheck(ns.db, e.row, value, load)
			if check == "max" then return end
			if check then
				f.st.ask = { check = check, value = value, load = load, before = before, slot = slot }
				return Settings_.RenderCorrect()
			end
		end
	end
	local stays, changes = ns.Lifecycle.WriteCorrection(e.row, value, load, answer)
	if stays == false then return Settings_.CorrectWindow(f.st.move, e.row) end
	local words = Settings_.FoldedWords(f.st.move, e.kind, e.row)
	ns.Print("cmd.correct.done", moveName(f.st.move), words)
	-- a set that now reads with no number (a 0: Done without a count, Not counted) takes the done view's own head
	local none = e.kind ~= "answer" and ns.SetValue(ns.state, e.row) == 0
	f.st.ask, f.st.result = nil, { words = words, none = none, steps = Settings_.StepLines(stays, changes) }
	Settings_.RenderCorrect()
end

-- "Change it" on "Is that right?": the weight it asked about goes back to the set's own, or the count goes (the rep box
-- empty and focused; the spinner back at the set's value).
local function changeCount(f)
	local a = f.st.ask
	f.st.ask = nil
	if a and a.check == "heavy" then
		f.kg:SetText(f.st.entry.load and ns.Sheet.Kg(f.st.entry.load) or "")
		if f.st.typed then f.kg:SetFocus() end
	elseif f.st.typed then
		f.box:SetText("")
		f.box:SetFocus()
	else
		f.spin:SetValue(f.st.entry.value)
	end
	Settings_.RenderCorrect()
end

-- The weight box's - and + beside the spinner (ns.LoadStep, the count step's one rule).
local function stepCorrectLoad(f, dir)
	local gear = ns.FilterContext(ns.profile, ns.state).gear
	f.kg:SetText(ns.Sheet.Kg(ns.LoadStep(f.st.move, correctLoad(f), dir, gear, f.st.entry.load)))
end

local function correctFrame()
	if F.correct then return F.correct end
	local f = Kit.Window("QuestPumpCorrect", "M", L["settings.name"], { escape = "special", fight = true })
	local v = Kit.Head(f.body, "", "", f.tier)
	local w = v.contentW
	-- what it shows (Settings_.CorrectWindow): the move, the set's entry and its input's mode, weight and slot, the
	-- answer chosen, the ask, the result, and whether the set asked for left the window
	f.st = {}
	f.view = v
	-- the list
	f.section = Kit.Section(v, "")
	f.list = Kit.List(v, w, f.tier.rowName)
	f.none = Kit.Text(v, "bodyWhite", "LEFT", Style.saga.note, w)
	f.old = Kit.Text(v, "bodyWhite", "LEFT", Style.saga.note, w)
	f.gone = Kit.Text(v, "bodyWhite", "LEFT", { RED_FONT_COLOR:GetRGB() }, w)
	-- a set: the question, the input or a pose's answers, the preview or "Is that right?", the notes
	f.question = Kit.Text(v, "bodyWhite", "CENTER", nil, w)
	f.inputs = CreateFrame("Frame", nil, v)
	f.inputs:SetSize(w, BOX_H)
	local function escape(self)
		if f.st.ask then return changeCount(f) end
		self:ClearFocus()
	end
	local box = CreateFrame("EditBox", nil, f.inputs, "InputBoxTemplate") -- unnamed: the template keys its parts
	box:SetNumeric(true)
	box:SetMaxLetters(3)
	box:SetAutoFocus(false)
	box:SetFontObject(Style.font.repBox or GameFontNormal)
	box:SetJustifyH("CENTER")
	box:SetSize(BOX_W, BOX_H)
	box:SetScript("OnEnterPressed", function() saveCorrection() end)
	box:SetScript("OnEscapePressed", escape)
	box:SetScript("OnTextChanged", function() Settings_.RenderCorrect() end)
	f.box = box
	local spin = CreateFrame("EditBox", nil, f.inputs, "NumericInputSpinnerTemplate")
	spin:SetAutoFocus(false)
	spin:HookScript("OnTextChanged", function() Settings_.RenderCorrect() end)
	spin:SetScript("OnEnterPressed", function(self)
		self:ClearFocus()
		saveCorrection()
	end)
	f.spin = spin
	local kg = CreateFrame("EditBox", nil, f.inputs, "InputBoxTemplate")
	kg:SetAutoFocus(false)
	kg:SetMaxLetters(5)
	kg:SetJustifyH("CENTER")
	kg:SetSize(KG_W, KG_H)
	kg:SetScript("OnEnterPressed", function() saveCorrection() end)
	kg:SetScript("OnEscapePressed", escape)
	kg:SetScript("OnTextChanged", function(self, userInput)
		local s = self:GetText()
		local clean = ns.Panel.CleanKg(s)
		if userInput and clean ~= s then return self:SetText(clean) end -- the SetText comes back here
		Settings_.RenderCorrect()
	end)
	kg.label = Kit.Text(f.inputs, "large", "LEFT")
	kg.down = Kit.Secondary(f.inputs, L["ui.result.kgDown"], 26, function() stepCorrectLoad(f, -1) end)
	kg.up = Kit.Secondary(f.inputs, L["ui.result.kgUp"], 26, function() stepCorrectLoad(f, 1) end)
	f.kg = kg
	f.answers = Kit.List(v, w, w)
	for _, key in ipairs(E.markKeys) do
		local row = Kit.Row(f.answers, { name = L[MARK_WORD[E.mark[key]]], font = "bodyWhite", below = true,
			onClick = function()
				Style.Play("checkOn")
				f.st.answer = E.mark[key]
				Settings_.RenderCorrect()
			end })
		row.answer = E.mark[key]
	end
	f.preview = Kit.Text(v, "bodyWhite", "CENTER", nil, w)
	f.really = Kit.Text(v, "bodyWhite", "CENTER", nil, w)
	f.reallyRow = CreateFrame("Frame", nil, v)
	f.reallyRow:SetSize(w, Kit.BUTTON_H)
	f.reallyYes = Kit.Secondary(f.reallyRow, "", f.tier.BACK, function() saveCorrection(true) end)
	f.reallyChange = Kit.Secondary(f.reallyRow, L["ui.result.reallyChange"], f.tier.BACK, function() changeCount(f) end)
	f.reach = Kit.Text(v, "bodyWhite", "CENTER", Style.saga.note, w)
	-- done: what the correction did to the steps, or why a step stays (Settings_.StepLines), a line each
	f.steps = Kit.Text(v, "bodyWhite", "CENTER", Style.saga.note, w)
	f.foot = Kit.Footer(f)
	f.clear = f.foot:Secondary(L["menu.clear"], function()
		Kit.Open(f, false)
		Settings_.ConfirmReset("move", f.st.move)
	end)
	f.other = f.foot:Secondary(L["correct.other"], function() Settings_.CorrectWindow(f.st.move) end)
	f.save = f.foot:Primary(L["correct.save"], function() saveCorrection() end, "tab")
	f.done = f.foot:Primary(L["ui.io.done"], function() Kit.Open(f, false) end, "tab")
	f:HookScript("OnHide", function()
		box:ClearFocus()
		kg:ClearFocus()
		spin:ClearFocus()
	end)
	F.correct = f
	return f
end

-- The two answers to "Is that right?", centred 10 apart, each as wide as its words + 40 and at least the tier's grey.
local function layAsk(f)
	local yes, change = f.reallyYes, f.reallyChange
	for _, b in ipairs({ yes, change }) do b:SetWidth(math.max(f.tier.BACK, b:GetTextWidth() + 40)) end
	local x = -(yes:GetWidth() + 10 + change:GetWidth()) / 2
	yes:ClearAllPoints()
	yes:SetPoint("LEFT", f.reallyRow, "CENTER", x, 0)
	change:ClearAllPoints()
	change:SetPoint("LEFT", yes, "RIGHT", 10, 0)
end

-- The list's rows, from the window's pool: each set's value, its day and time with Record before them on the
-- record's own, a click opening it.
local function correctRow(f, i)
	local row = f.list.rows[i]
	if row then return row end
	row = Kit.Row(f.list, { name = "", line = "", chevron = true,
		onClick = function(r) Settings_.CorrectWindow(f.st.move, r.set) end,
		tip = function(tooltip)
			Kit.Tip(row.name:GetText(), row.line:GetText(), { hint = L["correct.click"] })(tooltip)
		end })
	row.badge = Kit.Badge(row, "gold", L["correct.record"])
	row.lead = row.badge -- before the line (Kit.LayList takes its width off the line's)
	Kit.Choose(row, true, true)
	return row
end

local function renderList(f, v)
	local entries = correctable(f.st.move)
	local days = ns.C.correct_days
	local recordListed = false
	for i, e in ipairs(entries) do
		local row = correctRow(f, i)
		row.set = e.row
		row.name:SetText(Settings_.SetWords(f.st.move, e.kind, e.value, e.load, e.outcome))
		row.line:SetText(L["correct.when"]:format(ns.Sheet.DayDate(ns.day(e.row.ts)), ns.Sheet.ClockTime(e.row.ts)))
		row.badge:SetShown(e.record == true)
		row:Show()
		recordListed = recordListed or e.record == true
	end
	for i = #entries + 1, #f.list.rows do f.list.rows[i]:Hide() end
	local old = ns.HoldsRecord(ns.state, f.st.move) and not recordListed
	v.intro:Hide()
	Kit.SetCaps(f.section.words, ns.Plural("correct.sets", days):format(days))
	f.none:SetText(ns.Plural("correct.empty", days):format(days))
	f.old:SetText(L["correct.old"])
	f.gone:SetText(L["correct.gone"])
	f.clear:SetShown(old)
	return {
		{ f.section, 0, show = #entries > 0 },
		{ f.list, 14, #entries > 0 and Kit.LayList(f.list) or nil, show = #entries > 0 },
		{ f.none, 0, show = #entries == 0 },
		{ f.old, 14, show = old },
		{ f.gone, 14, show = f.st.gone and true or false },
	}
end

-- The count step's question over a set's input: a pose's ask, a counted set's label, a hold's, a rep set's, each of a
-- one-sided move asking its weaker side (the count step's words).
local function questionOf(e, m)
	if e.kind == "answer" then return L["ui.result.markAsk"] end
	if m.count then return L["ui.result.cleanLabel"] end
	if m.side then return L[e.kind == "seconds" and "ui.result.holdHintWeak" or "ui.result.hintWeak"] end
	return L[e.kind == "seconds" and "ui.result.holdHint" or "ui.result.hint"]
end

-- Under a set's input: "Is that right?" with its answers while the ask is about what the input holds (an ask the
-- input no longer holds goes), else what Save keeps or the limit the input is past, nothing while it holds no number
-- (renderSet hides the line). Answers the ask that shows: `f.st.ask`, { check, value, load, before, slot }
-- (ns.CorrectionCheck's answer for them, the state before the set and the slot it judged), in the count step's words
-- judged against the state before the set.
local function correctPreview(f, e, value, load)
	local a = f.st.ask
	if a and (a.value ~= value or a.load ~= load) then a, f.st.ask = nil, nil end
	if a then
		local line, yes = ns.Panel.ReallyWords(a.before, a.slot, a.check, a.value, a.load ~= 0 and a.load or nil)
		f.really:SetText(line)
		f.reallyYes:SetText(yes)
		layAsk(f)
	elseif value == nil then
		f.preview:SetText("")
	elseif overLimit(f, value, load) then
		f.preview:SetText(value > ns.CountMax(f.st.slot) and L["ui.result.max"]:format(ns.CountMax(f.st.slot))
			or L["ui.result.maxKg"]:format(ns.C.load_max_kg))
	else
		local shown = load
		if shown == nil then shown = e.load end
		-- a 0 as Save would keep it (a count corrected to 0 folds done, DB.lua correct), the set's own while unchanged
		local outcome = correctDiffers(f) and E.outcome.done or e.outcome
		f.preview:SetText(Settings_.SetWords(f.st.move, e.kind, value, shown, outcome))
	end
	return a
end

local function renderSet(f, v)
	local e, m = f.st.entry, ns.Library[f.st.move]
	local pose = e.kind == "answer"
	local when = L["correct.when"]:format(ns.Sheet.DayDate(ns.day(e.row.ts)), ns.Sheet.ClockTime(e.row.ts))
	if pose then
		v.intro:SetText(L["correct.savedPose"]:format(Settings_.SetWords(f.st.move, e.kind, e.value), when))
	elseif e.value == 0 then
		v.intro:SetText(L["correct.savedNone"]:format(when)) -- Done without a count, Could not do one, Not counted
	else
		v.intro:SetText(L["correct.saved"]:format(Settings_.SetWords(f.st.move, e.kind, e.value, e.load), when))
	end
	f.question:SetText(questionOf(e, m))
	local value, load, a = nil, nil, nil
	if pose then
		for _, row in ipairs(f.answers.rows) do Kit.Choose(row, true, row.answer ~= f.st.answer) end
	else
		placeCorrectInputs(f)
		value, load = correctValue(f), correctLoad(f)
		a = correctPreview(f, e, value, load)
	end
	-- another device's correction the game saves again (correctResave): its line in the preview's place, a pose's too;
	-- a set that reads with no number in the done view's words for it (correct.doneNone's)
	local resave = a == nil and correctResave(f)
	if resave then
		f.preview:SetText(not pose and e.value == 0 and L["correct.elsewhereNone"]
			or L["correct.elsewhere"]:format(Settings_.SetWords(f.st.move, e.kind, e.value, e.load, e.outcome)))
	end
	local reach = reachNote(nil, "status.correct.beta")
	f.reach:SetText(reach or "")
	f.other:Show()
	f.save:SetShown(a == nil)
	f.save:SetEnabled(correctChanged(f) and not (value and overLimit(f, value, load)))
	return {
		{ f.question, 0, show = true },
		{ f.inputs, 12, BOX_H, show = not pose },
		{ f.answers, 12, pose and Kit.LayList(f.answers) or nil, show = pose },
		{ f.preview, 10, show = a == nil and (resave or not pose and value ~= nil) and true or false },
		{ f.really, 10, show = a ~= nil },
		{ f.reallyRow, 8, Kit.BUTTON_H, show = a ~= nil },
		{ f.reach, 16, show = reach ~= nil },
	}
end

local function renderDone(f, v)
	local r = f.st.result
	v.intro:SetText(r.none and L["correct.doneNone"] or L["correct.done"]:format(r.words))
	local lines = {}
	for i, line in ipairs(r.steps) do lines[i] = L[line[1]]:format(unpack(line, 2)) end
	f.steps:SetText(table.concat(lines, "\n"))
	f.done:Show()
	return { { f.steps, 0, show = #lines > 0 } }
end

-- The window in its view: the list, a set, or done (Settings_.CorrectWindow); every part the other views carry hides.
function Settings_.RenderCorrect()
	local f = F.correct
	if not (f and f:IsShown()) then return end
	local v = f.view
	v.title:SetText(L["correct.title"]:format(moveName(f.st.move)))
	for _, part in ipairs({ f.section, f.list, f.none, f.old, f.gone, f.question, f.inputs, f.answers, f.preview,
		f.really, f.reallyRow, f.reach, f.steps }) do
		part:Hide()
	end
	for _, b in ipairs({ f.clear, f.other, f.save, f.done }) do b:Hide() end
	v.intro:Show() -- a set's and done's value line; the list has none
	local parts
	if f.st.result then
		parts = renderDone(f, v)
	elseif f.st.entry then
		parts = renderSet(f, v)
	else
		parts = renderList(f, v)
	end
	Kit.Fit(f, Kit.Stack(v, Kit.Top(v), parts), f.foot)
end

-- Opens Correct a count's window on move `id` (as the fold reads it): its list, or with `set` (a set row as stored, a
-- row of the list's) that set's view, back on the list with the red line under it where the set left the window.
-- Refused in a fight and on a save of a newer addon (chat), as a clear is.
function Settings_.CorrectWindow(id, set)
	local why = writeRefused()
	if why then return ns.Print(why) end
	local f = correctFrame()
	f.st.move, f.st.entry, f.st.ask, f.st.result, f.st.gone = id, nil, nil, nil, false
	local e = set and listedSet(id, set)
	f.st.gone = set ~= nil and e == nil
	if e then
		f.st.entry = e
		f.st.typed = ns.Panel.InputMode() == "type"
		f.st.loaded = e.row.kind == K.rep and ns.Loaded(id) ~= nil
		f.st.slot = { kind = e.row.kind, counted = ns.HasFlag(e.row.flags, "counted") or nil }
		f.st.answer = e.kind == "answer" and e.value or nil
		if e.kind ~= "answer" then
			f.spin:SetMinMaxValues(0, ns.CountMax(f.st.slot))
			f.spin:SetValue(e.value)
			f.box:SetText(tostring(e.value))
			f.kg:SetText(e.load and ns.Sheet.Kg(e.load) or "")
		end
	end
	if not f:IsShown() then
		-- an open pump stays hidden while the window shows (Events.lua MENU_WINDOWS), as under Options
		ns.Lifecycle.MenuShown()
		Kit.Open(f)
		ns.Lifecycle.MenuHidden()
	end
	Settings_.RenderCorrect()
	if e and e.kind ~= "answer" and f.st.typed then f.box:SetFocus() end
end

-- Export and import ---------------------------------------------------------------------------------------------------

function Settings_.ExportString()
	return ns.Export(ns.db, ns.state)
end

-- Returns true or false with the reason's locale key; a good string is folded in at once. A reset the string carried
-- that reset this save (ns.Import) is said once the import succeeded: a merge that fails puts the old save back.
function Settings_.ImportString(text)
	if not ns.db or ns.readOnly then return false, "import.readonly" end
	local before = CopyTable(ns.db)
	-- `why`: the reason a string is refused, or the time of the newer reset an import took
	local called, ok, why = pcall(function()
		local merged, reason = ns.Import(ns.db, text)
		if merged then ns.Fold(ns.db) end
		return merged, reason
	end)
	if not called then -- a string whose merge or fold raised partway: reported, and the save as it was
		ns.Report(ok)
		ns.Log("W", "import failed partway: the save is as it was")
		wipe(ns.db)
		for k, v in pairs(before) do ns.db[k] = v end
		ns.Fold(ns.db)
		-- the save's own tables again, never the copies the merge left bound
		ns.profile, ns.meta = ns.ProfileFor(ns.db, guid()), ns.db.meta
		return false, "import.broken"
	end
	if not ok then
		ns.Log("W", "import refused: %s", tostring(why))
		return false, "import." .. why
	end
	ns.Log("I", "import merged: %d rows now", #ns.db.log)
	if why then ns.Print("status.reset.elsewhere") end
	-- the merge replaced the profile tables by name: the live profile is the save's again (and the HUD's place, on "DB")
	ns.profile = ns.ProfileFor(ns.db, guid())
	ns.Emit("DB")
	ns.Emit("Trial")
	ns.Emit("Settings")
	return true
end

-- The export and import window (QuestPumpIO: the Data page's rows, /qp export, /qp import), one window for both jobs
-- at the kit's medium tier, its mode a switch at its top, the box under it with its label naming the copy or paste key
-- of this computer (addon code cannot write the clipboard; no line under the title: David, 2026-10-22): Export's
-- string selected in the box, what it holds said under it (where the string is made: docs/tasks.md), and Done;
-- Import's box empty and focused, what a merge does under it, its refusal in red under that (and in chat), and Import,
-- greyed while the box is empty. Export's box has no limit; Import's takes at most C.import_max_chars + 1 letters, so
-- a longer paste is cut there and refused as too long (DB.lua ns.Import) before the box holds megabytes; neither shows
-- a count. The game's Escape closes it (UISpecialFrames), and so does a fight (its box never holds
-- the keyboard into one); an open pump stays hidden while it shows (Events.lua MENU_WINDOWS).
local IO_BOX = 170

local function exportFrame()
	if F.io then return F.io end
	local f = Kit.Window("QuestPumpIO", "M", L["settings.name"], { escape = "special", fight = true })
	local v = Kit.Head(f.body, L["ui.io.title"], nil, f.tier)
	v.intro:Hide()
	f.view, f.mode = v, "export"
	f.modes = Kit.Segments(v, 240, { { "export", L["ui.io.export"] }, { "import", L["ui.io.import"] } },
		function() return f.mode end, function(mode) Settings_.OpenIO(mode == "export") end)
	f.boxLabel = Kit.Section(v, "")
	local scroll = CreateFrame("ScrollFrame", nil, v, "InputScrollFrameTemplate")
	scroll:SetSize(v.contentW - 18, IO_BOX)
	local box = scroll.EditBox
	box:SetAutoFocus(false)
	box:SetWidth(v.contentW - 30)
	if scroll.CharCount then scroll.CharCount:Hide() end -- no count: the letters left say nothing a player needs
	box:SetScript("OnEscapePressed", function() Kit.Open(f, false) end)
	box:HookScript("OnTextChanged", function(_, typed)
		if typed then f.refusal = nil end -- a refusal stays until the player changes the string
		Settings_.RenderIO()
	end)
	f.scroll, f.box = scroll, box
	f.section = Kit.Section(v, L["ui.io.holds"])
	f.note = Kit.Text(v, "bodyWhite", "LEFT", Style.saga.note, v.contentW)
	f.refused = Kit.Text(v, "bodyWhite", "LEFT", { RED_FONT_COLOR:GetRGB() }, v.contentW)
	f.foot = Kit.Footer(f)
	f.done = f.foot:Primary(L["ui.io.done"], function() Kit.Open(f, false) end, "tab")
	f.import = f.foot:Primary(L["ui.io.import"], function()
		local ok, why = Settings_.ImportString(box:GetText() or "")
		ns.Print(ok and "import.done" or why)
		if ok then return Kit.Open(f, false) end
		f.refusal = why
		Settings_.RenderIO()
	end)
	f:HookScript("OnHide", function() box:ClearFocus() end)
	F.io = f
	return f
end

-- The window for its mode: the switch, the box under its label (the copy or paste key of this computer), what the
-- string holds or what a merge does, a refusal and the mode's primary.
function Settings_.RenderIO()
	local f = F.io
	if not f then return end
	local v, export = f.view, f.mode == "export"
	local mac = IsMacClient and IsMacClient()
	local key = export and (mac and "ui.io.copyMac" or "ui.io.copy") or (mac and "ui.io.pasteMac" or "ui.io.paste")
	f.modes:Refresh()
	local y = Kit.Top(v)
	f.modes:ClearAllPoints()
	f.modes:SetPoint("TOP", v, "TOP", 0, -y)
	y = y + Kit.CONTROL_H + 16
	Kit.SetCaps(f.boxLabel.words, L[export and "ui.io.copyLabel" or "ui.io.pasteLabel"]:format(L[key]))
	Kit.At(f.boxLabel, v, y)
	y = y + 26
	Kit.At(f.scroll, v, y)
	y = y + IO_BOX + 16
	Kit.SetCaps(f.section.words, L[export and "ui.io.holds" or "ui.io.merge"])
	Kit.At(f.section, v, y)
	y = y + 26
	f.note:SetText(L[export and "ui.io.holdsNote" or "ui.io.mergeNote"])
	Kit.At(f.note, v, y)
	y = y + f.note:GetStringHeight()
	local refusal = not export and f.refusal and L[f.refusal] or nil
	f.refused:SetText(refusal or "")
	f.refused:SetShown(refusal ~= nil)
	if refusal then
		Kit.At(f.refused, v, y + 12)
		y = y + 12 + f.refused:GetStringHeight()
	end
	f.done:SetShown(export)
	f.import:SetShown(not export)
	f.import:SetEnabled(strtrim(f.box:GetText() or "") ~= "")
	Kit.Fit(f, y, f.foot)
end

-- Opens the window with the export string selected (copy with Ctrl+C, Cmd+C on a Mac), or empty for an import.
function Settings_.OpenIO(export)
	if fighting() then return ns.Print("chat.combat") end
	local f = exportFrame()
	f.mode, f.refusal = export and "export" or "import", nil
	-- an open pump stays hidden while the window shows, as under Options (Settings_.Open; Events.lua MENU_WINDOWS)
	ns.Lifecycle.MenuShown()
	Kit.Open(f)
	ns.Lifecycle.MenuHidden()
	-- one letter past the import's cap (a sheet string is ASCII, a letter a byte): a longer paste still reads as too long
	f.box:SetMaxLetters(export and 0 or ns.C.import_max_chars + 1)
	f.box:SetText(export and Settings_.ExportString() or "")
	Settings_.RenderIO()
	f.box:SetFocus()
	if export then f.box:HighlightText() end
end

ns.AddCommand("export", function() if ns.db then Settings_.OpenIO(true) end end, "cmd.export.help")

ns.AddCommand("import", function(arg)
	if not ns.db then return end
	arg = strtrim(arg or "")
	if arg ~= "" then
		local ok, why = Settings_.ImportString(arg)
		return ns.Print(ok and "import.done" or why)
	end
	Settings_.OpenIO(false)
end, "cmd.import.help")

-- The settings pages --------------------------------------------------------------------------------------------------

-- Every page is a canvas of QuestPump's own boxes and dropdowns (General and Equipment since 2026-10-05), never
-- Blizzard's settings list: an addon page in its vertical layout gets the Options search for "Discord" blocked in
-- QuestPump's name (WoWUIBugs #873, reproduced on Forever 1.60.1), and its dropdowns build Blizzard's shared menus from
-- the addon's options (Popups.lua). So the panel's Defaults button and its search leave QuestPump's pages, as the Body
-- and Social pages always kept them out.

-- What a row's tooltip names: the numbers from the constants, and each pace's line (Your setup shows the same lines
-- under its answers).
local TIP_ARGS = { standup = { ns.C.standup_s, ns.C.standup_after_min }, pace = {} }
for _, key in ipairs(ns.Enum.paceKeys) do TIP_ARGS.pace[#TIP_ARGS.pace + 1] = L["settings.pace." .. key .. ".meta"] end

-- this character's pause (Lc.Pause), under QuestPump's own switch, which covers every character on the profile; "while
-- grouped" is a choice only in a mixed party, where it holds (Lc.AskParty)
local PAUSE_KEY = { [ns.PAUSE.hour] = "hour", [ns.PAUSE.fourHours] = "fourHours", [ns.PAUSE.day] = "day",
	[ns.PAUSE.char] = "char", [ns.PAUSE.group] = "group" }

-- A choice of the profile's (a select's or a switch's): its choices as { value, words } pairs, from `values` and the
-- words' key `prefix` .. value.
local function choices(prefix, values)
	return function()
		local list = {}
		for _, v in ipairs(values) do list[#list + 1] = { v, L[prefix .. v] } end
		return list
	end
end

-- The General page's rows, a framed list each block, top down (2026-10-16): the master switch with this character's
-- pause and the companions under it, then what a pump asks, then the voice and sound beside what shows on the screen;
-- the rare ones under More (GENERAL_MORE). A row is { key, kind, ... }: its name settings.<key> (the row's one line:
-- no second label, David, 2026-10-22) and its tooltip's words settings.<key>.tip; kind "box" a switch of the profile,
-- "select" or "segments" a choice (choices() asked at each opening, so a choice of the moment shows only then; get()
-- and set(value), the profile's key itself without them, `default` while it is unset; `w` the control's width), "open"
-- a row that opens a window; `needs` a switch it needs on (or a function answering whether what it needs is there),
-- else it greys with the reason `why`.
local GENERAL_PAUSE = { "pause", "select", w = 170, needs = "enabled",
	why = "settings.pause.offAll",
	choices = function()
		local list = { { "on", L["settings.pause.on"] }, { "hour", L["settings.pause.hour"] },
			{ "fourHours", L["settings.pause.fourHours"] },
			{ "day", L["settings.pause.day"]:format(ns.C.day_offset_h) }, { "char", L["settings.pause.char"] } }
		if ns.Lifecycle.MixedParty() then list[#list + 1] = { "group", L["settings.pause.group"] } end
		return list
	end,
	get = function() return PAUSE_KEY[ns.Lifecycle.Paused() or 0] or "on" end,
	set = function(v)
		if v == "on" then ns.Lifecycle.Resume() else ns.Lifecycle.Pause(ns.PAUSE[v]) end
	end }

local GENERAL = {
	{ rows = {
		{ "enabled", "box" },
		GENERAL_PAUSE,
		{ "companions", "open" },
	} },
	{ head = "settings.section.pumps", rows = {
		-- how many moves each quest gives (ns.Pace, C.pace_moves; David, 2026-10-05), from the next quest turned in
		{ "pace", "segments", w = 252, choices = choices("settings.pace.", ns.Enum.paceKeys),
			get = function() return ns.Pace(ns.profile and ns.profile.pace) end,
			set = function(v) Settings_.Set("pace", v) end },
		-- the player's focus (Lc.SetFocus): one for every character, as the rows are
		{ "focus", "select", w = 170, choices = choices("focus.", ns.Enum.focusKeys),
			get = function() return ns.Focus(ns.state) end, set = function(v) ns.Lifecycle.SetFocus(v) end },
		{ "instances", "box" },
	} },
	{ side = {
		{ head = "settings.section.sound", rows = {
			{ "speech", "box" },
			{ "callouts", "box", needs = "speech", why = "settings.needsVoice" },
			{ "countReps", "box", needs = "speech", why = "settings.needsVoice" },
			{ "sounds", "box" },
		} },
		{ head = "settings.section.screen", rows = {
			{ "hud", "box" },
			{ "toasts", "box" },
			{ "afterPump", "select", w = 130, choices = choices("settings.afterPump.", { "summary", "wins", "none" }),
				default = "summary" },
			-- the Streamer layout (slice 23): no dim, the pump card in a corner and the day's tally (Panel.lua,
			-- Streamer.lua)
			{ "layout", "segments", w = 180, choices = choices("settings.layout.", { "standard", "streamer" }),
				default = "standard" },
		} },
	} },
}

-- Under General's More, the rare ones: the figure, the count input, chat, the partner on holds (a picture on the pump
-- card, slice 23) and the rules that add a move.
local GENERAL_MORE = {
	{ "figureAnimate", "box" },
	{ "figureColor", "select", w = 240, choices = choices("settings.figureColor.", { "class", "crimson" }),
		default = "class" },
	{ "inputMode", "select", w = 240, choices = choices("settings.input.", { "auto", "type", "spinner", "mouse" }),
		default = "auto" },
	{ "chat", "select", w = 240, choices = choices("settings.chat.", { "off", "warnings", "all" }),
		default = "warnings" },
	{ "partner", "box" },
	{ "standup", "box" },
	{ "deathTax", "box" },
}

-- What an unchecked piece of equipment would open now (ns.SetupOpens, slice 22: "Now it would open 9 moves, up to
-- Legendary"); nil while it is checked or opens nothing.
local function gearOpens(key)
	if not (ns.profile and ns.state) or ns.profile.gear[key] then return nil end
	for _, o in ipairs(ns.SetupOpens(ns.profile, ns.state)) do
		if o.key == key then
			-- moves with no colour (none stands on a ladder's rungs, a medicine ball's flow) are only counted
			if o.quality < 0 then return ns.Plural("settings.gear.opens.plain", o.count):format(o.count) end
			return ns.Plural("settings.gear.opens", o.count):format(o.count, L["quality." .. o.quality])
		end
	end
	return nil
end

-- A canvas page draws itself when the Settings panel displays it (SettingsPanelMixin:DisplayLayout calls the frame's
-- OnRefresh, Blizzard_SettingsPanel.lua on the forever branch, read 2026-09-25) and when it shows; it starts hidden,
-- so its first show is one.
local function canvas(f, render)
	f:Hide()
	f.OnRefresh = function() render() end
	f:SetScript("OnShow", function() render() end)
end

-- An equipment box's tooltip, on the Equipment page and Your setup's equipment (Panel.lua): the piece's name, what
-- counts as it and what it is for (David, 2026-10-13: "Add a hover explanation to the equpiments to give more
-- details"), while it is unchecked what it would open now, and its click last; drawn again when the box is clicked
-- under the cursor, so the lines follow the piece's state.
function Settings_.GearTip(cb, key)
	Kit.Hover(cb, Kit.Tip(L["gear." .. key], L["settings.gear." .. key .. ".tip"], { hint = function()
		local opens = gearOpens(key)
		local click = Kit.BoxHint(ns.profile and ns.profile.gear[key])
		return opens and { opens, click } or click
	end }))
	cb:HookScript("OnClick", function(self)
		if GameTooltip:IsOwned(self) then self:GetScript("OnEnter")(self) end
	end)
end

-- A Settings page on the kit's canvas (Kit.Canvas): its title and the rule under it (no tagline: its section labels
-- name its parts, David, 2026-10-22), the page's one action where Blizzard puts Defaults (a grey button: { label,
-- onClick, tip }) and its content in a scroll list from the head down, which a larger Text Size makes scroll (the head
-- stays); it draws itself whenever the Settings panel shows it (canvas).
local function newPage(title, action)
	return Kit.Canvas(title, { action = action, scroll = true })
end

-- Whether this client may lose the player's progress: it showed it does not read its save back (Setup.SaveUnread) and
-- the helper's mirror did not restore the save at this load; the General page's Companions row and the HUD's status
-- line both say it from here.
function Settings_.BetaUnsaved()
	return ns.Setup.SaveUnread() and not (ns.meta and ns.meta.restoredFrom == "mirror")
end

-- A switch of the profile, on or off.
local function isOn(key)
	return ns.profile ~= nil and ns.profile[key] == true
end

-- The list rows' name column on a list half a page wide (two side by side), where no line follows the name; on a
-- pair of `wide` lists (box rows alone: the Social page's), the name runs to the box, WIDE_ROOM short of the list's
-- edge (the box, its inset and a gap).
local HALF_NAME, WIDE_ROOM = 150, 3 + 12 + 22 + 12

-- A row of a page's list from its spec (GENERAL's, above): the name, its control at the right and its tooltip in the
-- kit's shape (the row's words, a greyed row's reason, a box's or a select's click hint last); a click anywhere on a
-- box's or a select's row does what its control does, while a switch's row only shows (its cells take the clicks).
-- Kit.Row's, with row.spec.
local function settingRow(list, spec)
	local key, kind = spec[1], spec[2]
	local name = L["settings." .. key]
	local row
	local tip = Kit.Tip(name, L["settings." .. key .. ".tip"]:format(unpack(TIP_ARGS[key] or {})), {
		reason = function() return row.unavailable end,
		hint = function()
			if row.unavailable then return nil end
			if kind == "box" then return Kit.BoxHint(isOn(key)) end
			if kind == "select" then return L["settings.click.choose"] end
		end })
	local opts = { name = name, tip = tip }
	local get = spec.get or function() return ns.profile and ns.profile[key] or spec.default end
	local set = spec.set or function(v) Settings_.Set(key, v) end
	if kind == "box" then
		opts.box = true
		opts.onClick = function()
			local on = not isOn(key)
			Style.Play(on and "checkOn" or "checkOff")
			Settings_.Set(key, on)
		end
	elseif kind == "select" then
		opts.control = Kit.Select(list, spec.w, function()
			local now = get()
			for _, c in ipairs(spec.choices()) do
				if c[1] == now then return c[2] end
			end
			return ""
		end, function(_, root)
			for _, c in ipairs(spec.choices()) do
				root:CreateRadio(c[2], function() return get() == c[1] end, function() set(c[1]) end)
			end
		end, tip)
		opts.onClick = function() opts.control:Click() end
	elseif kind == "segments" then
		opts.control = Kit.Segments(list, spec.w, spec.choices(), get, set, tip)
	end
	row = Kit.Row(list, opts)
	row.spec = spec
	return row
end

-- A row drawn from the profile: a box ticked or not (the tick carries "on", the name stays white: decision G4), a
-- choice's words, greyed with its reason while what it needs is off.
local function renderRow(row)
	local spec = row.spec
	local needs = spec.needs
	local met = needs == nil or (type(needs) == "function" and needs()) or (type(needs) == "string" and isOn(needs))
	local reason = not met and L[spec.why] or nil
	local on = spec[2] == "box" and isOn(spec[1])
	Kit.Choose(row, on, true, reason)
	if row.control then
		row.control.Refresh(row.control)
		if spec[2] == "select" then row.control:SetEnabled(reason == nil) end
	end
end

-- A page's blocks in its scroll list, top down: each { head, rows } a framed list under its section label (head, the
-- label's key; none on the page's first block), or { side = { block, block } } two side by side; f.rows holds every
-- row, made by `makeRow(list, spec)`.
local function buildBlocks(f, blocks, makeRow)
	local content = f.content
	f.blocks, f.rows = {}, {}
	for _, spec in ipairs(blocks) do
		local parts = spec.side or { spec }
		local w, gap = Kit.Grid(#parts, content.contentW)
		local block = { gap = gap }
		for i, part in ipairs(parts) do
			local b = {}
			if part.head then
				b.head = Kit.Section(content, L[part.head])
				b.head:SetWidth(w)
			end
			b.list = Kit.List(content, w, spec.wide and w - WIDE_ROOM or #parts > 1 and HALF_NAME
				or Kit.TIER.canvas.rowName)
			for _, r in ipairs(part.rows) do f.rows[#f.rows + 1] = makeRow(b.list, r) end
			block[i] = b
		end
		f.blocks[#f.blocks + 1] = block
	end
end

-- Lays the blocks out from `y` (16 between two, a label's list 14 under its 12-unit label) and answers the bottom.
local function layBlocks(f, y)
	local content = f.content
	for i, block in ipairs(f.blocks) do
		if i > 1 then y = y + 16 end
		local top = block[1].head and y + 12 + 14 or y
		local bottom = top
		for k, b in ipairs(block) do
			local x = (k - 1) * (b.list:GetWidth() + block.gap)
			if b.head then Kit.At(b.head, content, y, x) end
			Kit.At(b.list, content, top, x)
			bottom = math.max(bottom, top + Kit.LayList(b.list))
		end
		y = bottom
	end
	return y
end

-- The scroll list's content as tall as what it holds, so it scrolls only past the canvas.
local function fitContent(f, bottom)
	f.content:SetHeight(math.max(1, bottom))
end

-- QuestPump's own row in the list only opens and closes the pages under it (Settings_.Register); its page, which only a
-- gamepad's select reaches, is its title alone (its pages show under the row in the list), with nothing to click: a
-- link there would call Settings.OpenToCategory from the open panel, whose Open wipes the player's Blizzard settings
-- not yet applied (WipeModifiedTable, Gethe forever Blizzard_SettingsPanel.lua, read again 2026-10-22).
Settings_.PAGES = { "general", "equipment", "exercises", "body", "social", "data" }

local function buildTop()
	local f = Kit.Canvas(L["settings.name"])
	f:Hide()
	return f
end

-- The General page (Options > AddOns > QuestPump > General, /qp config, the pump card's Settings): how QuestPump asks,
-- sounds and shows, for every character; Send feedback its one action (slice 24), the Companions row opening the setup
-- screen at any time with how many of the two are set up (David, 2026-10-09), and on a Forever client that has not
-- shown it reads its save back yet (the beta bug) that row's red line with its cross.
local function buildGeneral()
	local f = newPage(L["settings.general"], { L["settings.feedback.button"],
		function() Settings_.OpenFeedback() end, Kit.Tip(L["settings.feedback.button"], L["settings.feedback.tip"], {
			reason = function() return fightReason() end }) })
	buildBlocks(f, GENERAL, function(list, spec)
		if spec[2] ~= "open" then return settingRow(list, spec) end
		local row = Kit.Row(list, { name = L["settings.companions"], line = "", chevron = true,
			onClick = function() ns.Setup.Open() end,
			tip = Kit.Tip(L["settings.companions"], L["settings.companions.tip"], {
				facts = function() return f.beta and { L["settings.beta.tip"] } or nil end,
				hint = L["settings.companions.click"] }) })
		row.spec = spec
		f.companions = row
		return row
	end)
	f.more = Kit.More(f.content, f.content.contentW, L["ui.more"], function(holder)
		f.moreList = Kit.List(holder, f.content.contentW, Kit.TIER.canvas.rowName)
		f.moreList:SetPoint("TOPLEFT")
		for _, spec in ipairs(GENERAL_MORE) do f.rows[#f.rows + 1] = settingRow(f.moreList, spec) end
	end)
	f.more.onToggle = function() Settings_.RenderGeneral() end
	canvas(f, Settings_.RenderGeneral)
	F.general = f
	return f
end

function Settings_.RenderGeneral()
	local f = F.general
	if not (f and ns.profile and ns.state) then return end
	for _, row in ipairs(f.rows) do
		if row ~= f.companions then renderRow(row) end
	end
	-- the companions the setup asks for (Setup.AsksPhone: the helper alone while the phone apps are not in the stores)
	local helper, phone = ns.Setup.Steps()
	local asks = ns.Setup.AsksPhone()
	local done = L["settings.companions.done"]:format((helper and 1 or 0) + (asks and phone and 1 or 0), asks and 2 or 1)
	f.beta = Settings_.BetaUnsaved()
	f.companions.line:SetText(f.beta and done .. "\n" .. Style.Cross(14) .. " "
		.. RED_FONT_COLOR:WrapTextInColorCode(L["settings.beta"]) or done)
	Kit.Choose(f.companions, false, true)
	f.action:SetEnabled(not fighting()) -- Send feedback: its window opens out of a fight only
	local y = layBlocks(f, 0) + 16
	Kit.At(f.more, f.content, y)
	y = y + f.more:GetHeight()
	if f.more:IsOpen() then
		Kit.At(f.more.content, f.content, y + 6)
		f.more.content:SetHeight(Kit.LayList(f.moreList))
		y = y + 6 + f.more.content:GetHeight()
	end
	fitContent(f, y)
end

-- The Equipment page: what you have at hand, the pool it leaves beside the question, and Your setup's own picture list
-- of the pieces (Settings_.BuildGear: the same groups under their captions, pictures, short names and boxes, down
-- the columns), each piece's tooltip saying what it is and what it would open (Settings_.GearTip); laid out at each
-- showing, at the player's text size.
local function buildEquipment()
	local f = newPage(L["settings.equipment"])
	f.pool = Kit.NumberRow(f.content, L["ui.setup.gear"], L["ui.setup.pool"], { NORMAL_FONT_COLOR:GetRGB() })
	Settings_.BuildGear(f.content, f.content.contentW)
	canvas(f, Settings_.RenderEquipment)
	F.equipment = f
	return f
end

function Settings_.RenderEquipment()
	local f = F.equipment
	if not (f and ns.profile and ns.state) then return end
	local y = Kit.LayNumber(f.pool, 0, tostring((Settings_.Pool(ns.profile, ns.state)))) + 16
	fitContent(f, Settings_.RenderGear(f.content, y))
end

-- The Body page: a canvas, never proxy settings, so the Settings panel's Defaults button never clears a pain flag or a
-- switch (both are safety filters): the pain flags as care rows (the tick alone: no gold, nothing alarming), then Your
-- setup's two groups of switches as its own rows (Settings_.BuildSwitchRows: their pictures and names), the room with
-- the pool it leaves. /qp gear is its chat twin.
local function painRow(list, i, key)
	local function flagged() return ns.state ~= nil and ns.state.pain[i] ~= nil end
	local row = Kit.Row(list, { name = L["joint." .. key], box = true, onClick = function()
		local on = not flagged()
		Style.Play(on and "checkOn" or "checkOff")
		if on then
			ns.Lifecycle.FlagPain({ [i] = true })
		else
			ns.AddRow({ move = i, kind = K.pain, reps = 0 })
		end
		ns.Emit("Settings")
	end, tip = Kit.Tip(L["joint." .. key], L["settings.body.painTip"], { hint = function()
		return L[flagged() and "settings.body.click.clear" or "settings.body.click.flag"]
	end }) })
	row.flagged = flagged
	return row
end

local function buildBody()
	local f = newPage(L["settings.body"])
	local content = f.content
	local half, gap = Kit.Grid(2, content.contentW)
	f.painHead = Kit.Section(content, L["settings.body.pain"])
	f.pain = {}
	for side = 1, 2 do
		local list = Kit.List(content, half, HALF_NAME)
		for i = side * 4 - 3, side * 4 do painRow(list, i, E.jointKeys[i]) end
		f.pain[side] = list
	end
	f.painGap = gap
	f.roomHead = Kit.Section(content, L["ui.setup.room"], false, "")
	f.room = Kit.List(content, content.contentW, Kit.TIER.canvas.rowName)
	Settings_.BuildSwitchRows(f.room, Settings_.PLACE, { care = true, hint = true })
	f.bodyHead = Kit.Section(content, L["settings.body"])
	f.body = Kit.List(content, content.contentW, Kit.TIER.canvas.rowName)
	Settings_.BuildSwitchRows(f.body, Settings_.BODY, { care = true, hint = true })
	canvas(f, Settings_.RenderBody)
	F.body = f
	return f
end

-- The Body page from the rows and the profile: a flag set from a pump's Something hurts shows at the next render.
function Settings_.RenderBody()
	local f = F.body
	if not (f and ns.profile and ns.state) then return end
	local content = f.content
	Kit.At(f.painHead, content, 0)
	local h = 0
	for side, list in ipairs(f.pain) do
		for _, row in ipairs(list.rows) do Kit.Choose(row, row.flagged(), true) end
		h = math.max(h, Kit.LayList(list))
		Kit.At(list, content, 26, (side - 1) * (list:GetWidth() + f.painGap))
	end
	local y = 26 + h + 16
	f.roomHead.aside:SetText(BreakUpLargeNumbers((Settings_.Pool(ns.profile, ns.state))) .. " " .. L["ui.setup.pool"])
	Kit.At(f.roomHead, content, y)
	y = y + 26
	Kit.At(f.room, content, y)
	y = y + Settings_.RenderSwitchRows(f.room) + 16
	Kit.At(f.bodyHead, content, y)
	y = y + 26
	Kit.At(f.body, content, y)
	fitContent(f, y + Settings_.RenderSwitchRows(f.body))
end

-- The Social page (slice 13): a canvas too, so the Defaults button never shares a card the player keeps private. Who
-- sees the card (three tiles: nobody, friends, guild and group, anyone who asks; one chosen, a choice clicked again
-- writes nothing) with what the card shows, what it carries beside what the player sees of others, and the players
-- muted or hidden from this session (never stored: the game's ignore list is the lasting one).
local SHARE_ART = { nobody = "shareNobody", circle = "shareCircle", anyone = "shareAnyone" }

local function shared()
	return ns.profile ~= nil and ns.profile.share ~= "nobody"
end

local SOCIAL = {
	{ side = {
		{ head = "settings.section.carries", rows = {
			{ "shareRecords", "box", needs = shared, why = "settings.social.needsShare" },
			{ "shareCount", "box", needs = shared, why = "settings.social.needsShare" },
			{ "tellLegend", "box", needs = shared, why = "settings.social.needsShare" },
		} },
		{ head = "settings.section.others", rows = {
			{ "showCards", "box" },
			{ "cheers", "box" },
			{ "invites", "box", needs = shared, why = "settings.social.needsShare" },
			{ "socialOffInInstances", "box" },
		} },
	}, wide = true },
}

-- Every row key the General and Social pages show (tools/lint.lua holds each to its settings.<key> and .tip).
function Settings_.RowKeys()
	local keys = {}
	local function add(rows)
		for _, r in ipairs(rows) do
			if r[2] ~= "open" then keys[#keys + 1] = r[1] end
		end
	end
	for _, blocks in ipairs({ GENERAL, SOCIAL }) do
		for _, b in ipairs(blocks) do
			for _, part in ipairs(b.side or { b }) do add(part.rows) end
		end
	end
	add(GENERAL_MORE)
	return keys
end

local function buildSocial()
	local f = newPage(L["settings.social"])
	local content = f.content
	local w = Kit.Grid(3, content.contentW)
	f.shareHead = Kit.Section(content, L["settings.social.share"])
	f.share = {}
	for i, key in ipairs(ns.Social.SHARE) do
		local c = Kit.Choice(content, w, 88, function()
			if ns.profile.share == key then return end -- the chosen one stays chosen and writes nothing
			Style.Play("checkOn")
			ns.Social.SetShare(key)
		end)
		c.icon = c:CreateTexture(nil, "ARTWORK")
		c.icon:SetPoint("CENTER", c, "TOP", 0, -(14 + Kit.CHIP_ICON / 2))
		c.icon:SetShown(Style.FitAtlas(c.icon, SHARE_ART[key], Kit.CHIP_ICON))
		c.icons = { c.icon }
		c.name = Kit.Text(c, "cardTitle", "CENTER", nil, w - 20)
		c.name:SetPoint("TOP", 0, -(14 + Kit.CHIP_ICON + 10))
		c.name:SetText(L["settings.social.share." .. key])
		c.key = key
		f.share[i] = c
	end
	f.note = Kit.Text(content, "bodyWhite", "LEFT", Style.saga.note, content.contentW)
	f.note:SetText(L["ui.share.note"])
	buildBlocks(f, SOCIAL, settingRow)
	f.lists = Kit.List(content, content.contentW, Kit.TIER.canvas.rowName)
	f.clear = Kit.Secondary(f.lists, L["settings.social.clear"], 1, function()
		wipe(ns.Social.mute)
		wipe(ns.Social.hide)
		ns.Emit("Social")
		Settings_.RenderSocial()
	end, Kit.TIER.canvas.BUTTON)
	f.clear:SetWidth(math.max(Kit.TIER.canvas.BACK, f.clear:GetTextWidth() + 40))
	f.muted = Kit.Row(f.lists, { name = L["settings.social.muted"], line = "", control = f.clear,
		tip = Kit.Tip(L["settings.social.muted"], L["settings.social.lists"]) })
	canvas(f, Settings_.RenderSocial)
	F.social = f
	return f
end

-- The Social page from the profile and the session's lists.
function Settings_.RenderSocial()
	local f = F.social
	if not (f and ns.profile) then return end
	local content = f.content
	Kit.At(f.shareHead, content, 0)
	local tileH = 88
	for _, c in ipairs(f.share) do
		Kit.Choose(c, ns.profile.share == c.key)
		tileH = math.max(tileH, 14 + Kit.CHIP_ICON + 10 + c.name:GetStringHeight() + 14)
	end
	for i, c in ipairs(f.share) do
		c:SetHeight(tileH)
		Kit.At(c, content, 26, (i - 1) * (c:GetWidth() + 11))
	end
	local y = 26 + tileH + 10
	Kit.At(f.note, content, y)
	y = y + f.note:GetStringHeight() + 16
	for _, row in ipairs(f.rows) do renderRow(row) end
	y = layBlocks(f, y) + 16
	local muted, hidden = 0, 0
	for _ in pairs(ns.Social.mute) do muted = muted + 1 end
	for _ in pairs(ns.Social.hide) do hidden = hidden + 1 end
	f.muted.line:SetText(L["settings.social.session"]:format(muted, hidden))
	f.clear:SetEnabled(muted + hidden > 0)
	Kit.Choose(f.muted, false, true)
	Kit.At(f.lists, content, y)
	fitContent(f, y + Kit.LayList(f.lists))
end

-- The Exercises page: leave a group or a single move out of your pumps, and see why a move never comes. Its fixed part
-- under the head: the search, the pool (its tooltip the patterns), Bodyweight or With equipment, and the bar of the
-- notes on the pool (Settings_.Warnings); then a scroll list of the notes, one row each while the bar is open, and the
-- groups, the bodyweight ones by region, a label each, the equipment's in the Equipment page's order. A group opens on
-- a click to its moves (its box leaves the whole group out); a move your setup keeps out shows the one mark with its
-- reason, its box still yours (a choice for later). Nothing is built at PLAYER_LOGIN: the group rows at the page's
-- first show, a group's move rows from its list's pool when it opens, given back when it closes; which groups are open
-- lasts the session.
local REGIONS = {
	{ "settings.region.pushpull", { "push", "pull" } },
	{ "settings.region.legs", { "squat", "lunge", "hinge", "calf" } },
	{ "settings.region.trunk", { "core", "situp", "carry" } },
	{ "settings.region.cardio", { "cardio", "mobility", "yoga", "balance", "breath" } },
}

-- A bodyweight group's picture, a move of it by library key; an equipment group shows its piece's (Kit.GearMove, as
-- the Equipment page does).
local GROUP_PICTURE = { push = "pushup", pull = "inverted_row", squat = "air_squat", lunge = "reverse_lunge",
	hinge = "glute_bridge", calf = "heel_raise", core = "plank", situp = "crunch", carry = "suitcase_carry",
	cardio = "jumping_jacks", mobility = "worlds_greatest", yoga = "downward_dog" }

-- The list rows' name column on the Exercises page: a group's or a move's name from its picture to its line.
local EXERCISE_NAME = 230

-- A piece's short name (Your setup's, where gear.<key>'s explanation would not fit a line).
local function gearName(key)
	return rawget(L, "ui.setup.gearName." .. key) or L["gear." .. key]
end

-- Each switch's /qp gear name by the profile key ns.Passes names it by.
local SWITCH_NAME = {}
for name, key in pairs(SWITCHES) do SWITCH_NAME[key] = name end

-- Why the setup keeps a ticked move out of the pumps, in the words of its mark (the first of ns.Shut's reasons that is
-- the setup's: a piece missing, a switch, a pain flag; else a move the player's gear does better, ns.BetterOpen), or
-- nil for a move the pumps take.
function Settings_.KeptOut(id, f)
	for _, w in ipairs(ns.Shut(id, f)) do
		if w.reason == "gear" then
			return L["settings.out.gear"]:format(gearName(type(w.need) == "table" and w.need[1] or w.need))
		elseif w.reason == "pain" then
			for _, tag in ipairs(ns.Library[id].pain) do
				local joint = E.joint[tag]
				if joint and f.pain[joint] then return L["settings.out.pain"]:format(L["joint." .. tag]) end
			end
		elseif SWITCH_NAME[w.reason] then
			local name = SWITCH_NAME[w.reason]
			return L["settings.out.switch"]:format(L["gear." .. name], L[f[w.reason] and "gear.on" or "gear.off"])
		end
	end
	local better = ns.BetterOpen(id, f)
	return better and L["settings.out.better"]:format(moveName(better)) or nil
end

-- The moves of each group (the step jacks have no row: they are the floor), by ladder (ns.LADDER_ORDER), then rung,
-- then name, the moves off the ladders last; asked once, at the page's first show.
local function groupMoves()
	local at, byGroup = {}, {}
	for i, l in ipairs(ns.LADDER_ORDER) do at[l] = i end
	for id, m in pairs(ns.Library) do
		if id ~= ns.FLOOR_MOVE then
			byGroup[m.group] = byGroup[m.group] or {}
			table.insert(byGroup[m.group], id)
		end
	end
	for _, ids in pairs(byGroup) do
		table.sort(ids, function(a, b)
			local ma, mb = ns.Library[a], ns.Library[b]
			local la, lb = at[ma.ladder] or math.huge, at[mb.ladder] or math.huge
			if la ~= lb then return la < lb end
			if (ma.rung or 0) ~= (mb.rung or 0) then return (ma.rung or 0) < (mb.rung or 0) end
			return moveName(a) < moveName(b)
		end)
	end
	return byGroup
end

-- A move's row of a group's list, from the list's pool or new: its key pose, its name, its step's badge and its
-- ladder (or the mark, or "Left out"), its box; a click ticks it, a right-click opens its menu (a count corrected, its
-- record cleared: Kit.Row's rightClick, as the Records tab's rows; neither while its group is left out, the row
-- greyed). The row reads row.move at each click and hover.
local function moveRow(list)
	local row = table.remove(list.free)
	if row then return row end
	row = Kit.Row(list, { picture = ns.FLOOR_MOVE, name = "", line = "", indent = 24, box = true,
		onClick = function(r)
			local excluded = not Settings_.Excluded(r.move)
			Style.Play(excluded and "checkOff" or "checkOn")
			Settings_.SetExcluded(r.move, excluded)
		end,
		rightClick = function(r) Settings_.RecordMenu(r.button, r.move) end,
		tip = function(tooltip)
			ns.Sheet.MoveTooltip(tooltip, row.move)
			if row.unavailable then
				GameTooltip_AddErrorLine(tooltip, row.unavailable)
			else
				GameTooltip_AddInstructionLine(tooltip, L[Settings_.Excluded(row.move) and "settings.move.clickIn"
					or "settings.move.clickOut"])
				Settings_.RecordHint(tooltip, row.move)
			end
		end })
	row.badge = Kit.Badge(row, 0)
	row.lead = row.badge -- before the line (Kit.LayList takes its width off the line's)
	table.remove(list.rows) -- Kit.Row appended it: the render puts the list's rows in order, a group then its moves
	return row
end

-- A group's row: its picture, its name, how many of its moves your pool holds (or what it needs), its own box; a click
-- opens or closes it, a click on its box leaves the whole group out or puts it back (each tooltip ends in its click).
local function groupRow(f, list, group)
	local gear = false
	for _, key in ipairs(ns.Gear) do gear = gear or key == group end
	local picture = gear and Kit.GearMove(group) or Kit.MoveByKey(GROUP_PICTURE[group] or "")
	local row
	row = Kit.GroupRow(list, { picture = picture, name = L["group." .. group], line = "", open = f.open[group],
		-- while a search is on, every group with a match stays open on its matches
		tip = Kit.Tip(L["group." .. group], function() return row.count end, { hint = function()
			if (f.search:GetText() or "") ~= "" then return nil end
			return L[row.open and "settings.group.clickHide" or "settings.group.clickShow"]
		end }),
		boxTip = Kit.Tip(L["group." .. group], function() return row.count end, { hint = function()
			return L[Settings_.Excluded(group) and "settings.group.clickIn" or "settings.group.clickOut"]
		end }),
		onToggle = function(open)
			f.open[group] = open or nil
			Settings_.RenderExercises()
		end,
		onBox = function(on) Settings_.SetExcluded(group, not on) end })
	row.group, row.gear, row.moves = group, gear and group or false, {}
	list.groups[#list.groups + 1] = row
	return row
end

local function buildExercises()
	local f = newPage(L["settings.exercises"], { L["preset.title"], function(b)
		Kit.OpenMenu(b, function(_, root)
			root:CreateTitle(L["preset.title"])
			for i, name in ipairs(Settings_.PRESETS) do
				if i == #Settings_.PRESETS then root:CreateDivider() end
				local entry = root:CreateButton(L["preset." .. name], function() Settings_.ApplyPreset(name) end)
				entry:SetTooltip(Kit.Tip(L["preset." .. name], L["preset." .. name .. ".tip"]))
			end
		end)
	end })
	-- it opens a menu: the kit's grey menu button, its arrow and its least width as the bar's More
	f.action.barWidth = Kit.TIER.canvas.BACK
	Kit.Caret(f.action)
	local w = f.content.contentW
	f.top = Kit.Top(f)
	f.search = Kit.Search(f, 240, L["settings.exercises.search"], function() Settings_.RenderExercises() end)
	f.pool = Kit.NumberRow(f, "", L["ui.setup.pool"], { NORMAL_FONT_COLOR:GetRGB() })
	f.poolHit = CreateFrame("Frame", nil, f)
	f.poolHit:SetPoint("TOPLEFT", f.pool.number, "TOPLEFT")
	f.poolHit:SetPoint("BOTTOMRIGHT", f.pool.word, "BOTTOMRIGHT")
	Kit.Hover(f.poolHit, Kit.Tip(L["ui.setup.pool"], function()
		return L["cmd.gear.pool"]:format(Settings_.Pool(ns.profile, ns.state))
	end, { reason = function()
		local _, patterns = Settings_.Pool(ns.profile, ns.state)
		return patterns < 3 and L["cmd.gear.few"]:format(patterns) or nil
	end }))
	f.half = "body"
	f.halves = Kit.Segments(f, 300, { { "body", L["settings.exercises.bodyweight"] },
		{ "gear", L["settings.exercises.gear"] } }, function()
		return (f.search:GetText() or "") == "" and f.half or nil
	end, function(v)
		f.half = v
		Settings_.RenderExercises()
	end)
	-- the notes' bar in the fixed part, its list at the top of the scroll list over the groups (so the groups keep the
	-- room from the switch down however many notes there are), scrolled into view as it opens
	f.notes = Kit.More(f, w - 300 - 10, "", function(holder)
		f.noteList = Kit.List(holder, w, w)
		f.noteList:SetPoint("TOPLEFT")
	end)
	f.notes.content:SetParent(f.content)
	f.notes.onToggle = function(open)
		if open then f.scroll:SetVerticalScroll(0) end
		Settings_.RenderExercises()
	end
	f.open = {}
	f.built = false
	canvas(f, Settings_.RenderExercises)
	F.exercises = f
	return f
end

-- The group rows, at the page's first show: a section and its list a region, then the equipment's one list.
local function buildGroups(f)
	f.built = true
	f.byGroup = groupMoves()
	f.sections = {}
	local w = f.content.contentW
	local function list()
		local l = Kit.List(f.content, w, EXERCISE_NAME)
		l.groups, l.free = {}, {}
		return l
	end
	for _, region in ipairs(REGIONS) do
		local s = { head = Kit.Section(f.content, L[region[1]]), list = list(), half = "body" }
		for _, group in ipairs(region[2]) do
			if f.byGroup[group] then groupRow(f, s.list, group) end
		end
		f.sections[#f.sections + 1] = s
	end
	local s = { head = Kit.Section(f.content, L["settings.exercises.gear"]), list = list(), half = "gear" }
	for _, key in ipairs(ns.Gear) do
		if f.byGroup[key] then groupRow(f, s.list, key) end
	end
	f.sections[#f.sections + 1] = s
	f.empty = Kit.Empty(f.content, w, {})
end

-- A move's row drawn: in your pumps (ticked and the filters pass), its step's badge and a ladder move's record when it
-- has one; left out by you, an opt-in move (the headstand) too until ticked; kept out by your setup (the mark and its
-- first reason, the picture dimmed, the box still ticked); greyed with its group left out.
local function renderMove(row, id, fc, groupOut)
	local m = ns.Library[id]
	row.move = id
	row.name:SetText(moveName(id))
	Kit.SetIcon(row.icons[1], id)
	local ticked = not Settings_.Excluded(id)
	local kept = ticked and not groupOut and Settings_.KeptOut(id, fc) or nil
	Kit.Choose(row, ticked, true, groupOut and L["settings.group.leftOut"] or nil)
	if kept then
		row.icons[1]:SetDesaturated(true)
		row.icons[1]:SetAlpha(0.5)
	end
	row:Mark(kept)
	local q = ticked and not kept and not groupOut and ns.MoveQuality(id) or nil
	row.badge:SetShown(q ~= nil)
	if q then row.badge:Set(q) end
	local best = ticked and m.ladder and ns.Record(ns.state, id)
	row.line:SetText(not ticked and L["settings.move.leftOut"]
		or best and L["settings.move.withRecord"]:format(ns.Sheet.MoveAmount(id, best)) or "")
end

-- A group's rows drawn and its moves shown while it is open (all of them, or a search's matches), its list's rows put
-- in order (the group, then its moves); answers whether the group shows.
local function renderGroup(f, list, row, fc, query)
	local group = row.group
	local ids, shown = f.byGroup[group], {}
	for _, id in ipairs(ids) do
		if query == "" or moveName(id):lower():find(query, 1, true) then shown[#shown + 1] = id end
	end
	if query ~= "" and #shown == 0 then return false end
	local open = query ~= "" or f.open[group] == true
	row:SetOpen(open)
	local groupOut = Settings_.Excluded(group)
	Kit.Choose(row, not groupOut, true)
	local passes = 0
	for _, id in ipairs(ids) do
		if ns.Passes(id, fc) then passes = passes + 1 end
	end
	row.count = L["settings.group.count"]:format(passes, #ids)
	local needs = row.gear and not ns.profile.gear[row.gear] and L["settings.out.gear"]:format(gearName(row.gear)) or nil
	row.line:SetText(row.count)
	row:Mark(needs)
	list.rows[#list.rows + 1] = row
	local want = open and #shown or 0
	for i = want + 1, #row.moves do
		row.moves[i]:Hide()
		list.free[#list.free + 1] = row.moves[i]
		row.moves[i] = nil
	end
	for i = 1, want do
		local r = row.moves[i] or moveRow(list)
		row.moves[i] = r
		r:Show()
		renderMove(r, shown[i], fc, groupOut)
		list.rows[#list.rows + 1] = r
	end
	return true
end

-- Lays the page out for the search text and the switch: the fixed part, then the regions' lists (or the equipment's),
-- each group with its moves while open; the pool, the notes and the moves' marks from the profile.
function Settings_.RenderExercises()
	local f = F.exercises
	if not (f and f:IsVisible() and ns.profile and ns.state) then return end
	if not f.built then buildGroups(f) end
	local fc = ns.FilterContext(ns.profile, ns.state)
	local query = (f.search:GetText() or ""):lower()
	-- the fixed part: the search and the pool on a line, the switch and the notes under them
	local top = f.top
	Kit.At(f.search, f, top + 16)
	Kit.LayNumber(f.pool, top - 2, BreakUpLargeNumbers((Settings_.Pool(ns.profile, ns.state))))
	f.pool.word:ClearAllPoints()
	f.pool.word:SetPoint("BOTTOMRIGHT", f, "TOPRIGHT", -(f.inset + Kit.TIER.canvas.SCROLLBAR), -(top + 38))
	local y = top + 54
	Kit.At(f.halves, f, y)
	f.halves:Refresh()
	local notes = Settings_.Warnings(ns.profile, ns.state)
	f.notes:SetShown(#notes > 0)
	f.notes.words:SetText(ns.Plural("settings.warn.more", #notes):format(#notes))
	Kit.At(f.notes, f, y, 300 + 10)
	y = y + Kit.CONTROL_H
	f.scroll:ClearAllPoints()
	f.scroll:SetPoint("TOPLEFT", f.inset, -(y + 14))
	f.scroll:SetPoint("BOTTOMRIGHT", -f.inset - Kit.TIER.canvas.SCROLLBAR, 8)
	local content, cy, any = f.content, 0, false
	-- the opened notes first in the scroll list, then the groups 16 under them
	if #notes > 0 and f.notes:IsOpen() then
		local list = f.noteList
		for i, text in ipairs(notes) do
			local r = list.rows[i] or Kit.Row(list, { name = "", font = "bodyWhite", below = true })
			r.name:SetText(text)
			Kit.Choose(r, false, true)
			r:Show()
		end
		for i = #notes + 1, #list.rows do list.rows[i]:Hide() end
		f.notes.content:SetHeight(Kit.LayList(list))
		f.notes.content:Show()
		Kit.At(f.notes.content, content, 0)
		cy = f.notes.content:GetHeight() + 16
	else
		f.notes.content:Hide()
	end
	-- the groups: while searching, every group with a match, open on its matches, under both halves' labels
	for _, s in ipairs(f.sections) do
		local list = s.list
		wipe(list.rows)
		local shown = 0
		for _, row in ipairs(list.groups) do
			local on = (query ~= "" or s.half == f.half) and renderGroup(f, list, row, fc, query)
			row:SetShown(on and true or false)
			if not on then
				for i = #row.moves, 1, -1 do
					row.moves[i]:Hide()
					list.free[#list.free + 1] = row.moves[i]
					row.moves[i] = nil
				end
			else
				shown = shown + 1
			end
		end
		for _, r in ipairs(list.free) do list.rows[#list.rows + 1] = r end
		-- the equipment's list carries no label of its own but while searching, beside the regions'
		local labelled = s.half == "body" or query ~= ""
		s.head:SetShown(shown > 0 and labelled)
		list:SetShown(shown > 0)
		if shown > 0 then
			if any then cy = cy + 16 end
			if labelled then
				Kit.At(s.head, content, cy)
				cy = cy + 26
			end
			Kit.At(list, content, cy)
			cy = cy + Kit.LayList(list)
			any = true
		end
	end
	f.empty:SetShown(not any)
	if not any then
		f.empty:Set(L["settings.exercises.none"]:format(f.search:GetText() or ""))
		Kit.At(f.empty, content, cy + 24)
		cy = cy + 24 + f.empty:Lay()
	end
	fitContent(f, cy)
end

-- The Data page: your profile (one select: the profiles, and this character's own), a copy of your progress (the
-- export window in each of its modes) and starting over, the dangerous actions apart and last: a ladder started over
-- or a move's record cleared (picked from a menu: the ladders the player can start over, the moves with a record by
-- ladder, as the Records tab lists them), the profile's settings reset, the backup restored, and it all reset, each a
-- grey button named for its action that asks QUESTPUMP_RESET first. A row is its name and its control, what it does
-- in its tooltip and the window or the confirmation it opens, never a line under the name; only a state shows there
-- (the profile shared by every character, no backup yet: `line` "" makes its room). A control that cannot act now
-- greys with its reason in its tooltip: a read-only save, a fight, nothing to clear, no backup.
local function dataRow(list, name, line, control, tip, open)
	return Kit.Row(list, { name = name, line = line, control = control, chevron = control == nil, tip = tip,
		onClick = function()
			if control then control:Click() else open() end
		end })
end

-- A row's tooltip: its name, its words, and why it is greyed while it is (row.unavailable, Kit.Choose's), else its
-- click's hint (the key `click`).
local function dataTip(rowOf, name, body, click)
	return Kit.Tip(name, body, { reason = function() return rowOf().unavailable end,
		hint = function() return not rowOf().unavailable and L[click] or nil end })
end

local function buildData()
	local f = newPage(L["settings.data"])
	local content, w, nameW = f.content, f.content.contentW, Kit.TIER.canvas.rowName
	local rows = {}
	local function tip(key, name, body, click)
		return dataTip(function() return rows[key] end, name, body, click)
	end
	f.profileHead = Kit.Section(content, L["settings.profile"])
	f.profileList = Kit.List(content, w, nameW)
	local profileTip = tip("profile", L["settings.profile"], L["settings.perChar.tip"], "settings.click.choose")
	f.profile = Kit.Select(f.profileList, 200, Settings_.CurrentProfile, function(_, root)
		local own = Settings_.OwnProfile()
		for _, name in ipairs(Settings_.ProfileNames()) do
			if name ~= own then
				root:CreateRadio(name, function() return Settings_.CurrentProfile() == name end,
					function() Settings_.UseProfile(name) end)
			end
		end
		root:CreateDivider()
		local entry = root:CreateRadio(L["settings.perChar"], function() return Settings_.CurrentProfile() == own end,
			function() Settings_.PerCharacter(true) end)
		entry:SetTooltip(Kit.Tip(L["settings.perChar"], L["settings.perChar.tip"]))
	end, profileTip)
	rows.profile = dataRow(f.profileList, L["settings.profile"], "", f.profile, profileTip)
	f.progressHead = Kit.Section(content, L["settings.section.progress"])
	f.progressList = Kit.List(content, w, nameW)
	rows.export = dataRow(f.progressList, L["ui.io.export"], nil, nil,
		tip("export", L["ui.io.export"], L["ui.io.holdsNote"], "settings.click.export"),
		function() Settings_.OpenIO(true) end)
	rows.import = dataRow(f.progressList, L["ui.io.import"], nil, nil,
		tip("import", L["ui.io.import"], L["ui.io.mergeNote"], "settings.click.import"),
		function() Settings_.OpenIO(false) end)
	f.overHead = Kit.Section(content, L["settings.section.over"])
	f.overList = Kit.List(content, w, nameW)
	local restartTip = tip("restart", L["settings.restart"], L["settings.restart.line"], "settings.click.choose")
	f.restart = Kit.Select(f.overList, 180, function() return L["settings.restart.pick"] end, function(_, root)
		for _, l in ipairs(ns.LADDER_ORDER) do
			if Settings_.CanRestart(l) then
				root:CreateButton(L["ladder." .. l], function() Settings_.ConfirmReset("ladder", l) end)
			end
		end
	end, restartTip)
	rows.restart = dataRow(f.overList, L["settings.restart"], nil, f.restart, restartTip)
	local clearTip = tip("clear", L["settings.clear"], L["settings.clear.line"], "settings.click.choose")
	f.clear = Kit.Select(f.overList, 180, function() return L["settings.clear.pick"] end, function(_, root)
		for _, group in ipairs(ns.RecordedMoves(ns.state)) do
			local l = group.ladder
			local sub = root:CreateButton(l > 0 and L["ladder." .. l] or L["records.other"])
			for _, id in ipairs(group.moves) do
				sub:CreateButton(moveName(id), function() Settings_.ConfirmReset("move", id) end)
			end
		end
	end, clearTip)
	rows.clear = dataRow(f.overList, L["settings.clear"], nil, f.clear, clearTip)
	-- a grey button that asks first: its row's tooltip says what it does (`body`), the row's line only a state (`line`)
	local function action(key, words, body, what, name, line)
		local t = tip(key, name, body, "settings.click.ask")
		local b = Kit.Secondary(f.overList, words, 150, function() Settings_.ConfirmReset(what) end,
			Kit.TIER.canvas.BUTTON)
		Kit.Hover(b, t)
		rows[key] = dataRow(f.overList, name, line, b, t)
		f[key] = b
	end
	action("resetChar", L["settings.reset.charButton"], L["settings.reset.charLine"], "char", L["settings.reset.char"])
	action("restore", L["settings.restore.button"], L["settings.restore.line"], "restore", L["settings.restore"], "")
	action("resetAll", L["settings.reset.all"], L["settings.reset.allLine"], "all", L["settings.reset.all"])
	f.dataRows = rows
	canvas(f, Settings_.RenderData)
	F.data = f
	return f
end

function Settings_.RenderData()
	local f = F.data
	if not (f and ns.state) then return end
	local rows, readOnly = f.dataRows, ns.readOnly and L["import.readonly"] or nil
	local refused = writeRefused()
	refused = refused and L[refused] or nil
	local backup = type(ns.db and ns.db.backup) == "table"
	local recorded = #ns.RecordedMoves(ns.state) > 0
	f.profile.Refresh()
	f.profile:SetEnabled(readOnly == nil)
	rows.profile.line:SetText(Settings_.CurrentProfile() == "Default" and L["settings.data.shared"] or "")
	rows.restore.line:SetText(backup and "" or L["settings.restore.none"])
	-- the export window opens out of a fight only (Settings_.OpenIO)
	local why = {
		profile = readOnly, export = fightReason(), import = readOnly or fightReason(), restart = refused,
		clear = refused or not recorded and L["settings.clear.none"] or nil,
		resetChar = readOnly, restore = readOnly or not backup and L["settings.restore.none"] or nil, resetAll = readOnly,
	}
	for key, row in pairs(rows) do
		Kit.Choose(row, false, true, why[key])
		if row.control then row.control:SetEnabled(why[key] == nil) end
	end
	local content, y = f.content, 0
	for _, block in ipairs({ { f.profileHead, f.profileList }, { f.progressHead, f.progressList },
		{ f.overHead, f.overList } }) do
		if y > 0 then y = y + 16 end
		Kit.At(block[1], content, y)
		y = y + 26
		Kit.At(block[2], content, y)
		y = y + Kit.LayList(block[2])
	end
	fitContent(f, y)
end

-- The feedback window (slice 24; docs/ui.md, "The feedback window"): the export window's sibling at the kit's large
-- tier, opened by the General page's "Send feedback" and /qp feedback, in the words every window says
-- (contract/feedback-api.yaml, "What every window says"), one column so "the counts below" stays true: that nobody
-- can write back, under the title; while Send is off, the setup's companion card (QuestPump on this computer, its
-- cross and state, and what turns Send on and where else feedback goes); the player's text, held to
-- feedback_max_chars, with its counter, the "Add my numbers" tick, on at first, and the note of what goes by it; then
-- what goes: "From the game, version X" and, ticked, the numbers listed in full (its headings in the notes' grey),
-- counted when the window opens and sent as shown (ns.FeedbackNumbers); the bar's status after a Send at its left and
-- the red Send at its right. The addon has no network, so Send hands the message to the helper over the strip
-- (Bridge.lua B.SendFeedback), and it needs the helper running at this load. The text stays in the box until its
-- message is on the strip; it is never logged.
local function feedbackLines(n)
	local note = Style.saga.note
	local function heading(key, ...)
		return CreateColor(note[1], note[2], note[3]):WrapTextInColorCode(L[key]:format(...))
	end
	-- a median of whole counts: 0, 3 or 2.5, never 3.0 (the wire keeps the number)
	local maxSets = n.max_sets % 1 == 0 and tostring(n.max_sets) or ("%.1f"):format(n.max_sets)
	local lines = { L["ui.feedback.days"]:format(n.days), L["ui.feedback.unfinished"]:format(n.owed),
		heading("ui.feedback.window", ns.C.feedback_days), L["ui.feedback.finished"]:format(n.finished),
		L["ui.feedback.banked"]:format(n.banked), L["ui.feedback.maxsets"]:format(maxSets), heading("ui.feedback.moves") }
	for _, m in ipairs(n.moves) do
		lines[#lines + 1] = L["ui.feedback.move"]:format(L["move." .. m.move], m.easier, m.stopped)
	end
	if #n.moves == 0 then lines[#lines + 1] = L["ui.feedback.none"] end
	return lines
end

-- The box's and the list's heights: the box keeps its own, the list gives way to the window's least height (and keeps
-- at least FEEDBACK_LIST, the window growing past it and then scaling down, Kit.Fit), its lines scrolling.
local FEEDBACK_BOX, FEEDBACK_LIST, FEEDBACK_SCREEN = 116, 88, { 72, 54 }

local function feedbackFrame()
	if F.feedback then return F.feedback end
	-- named for UISpecialFrames: Escape closes it with or without the box's focus
	local f = Kit.Window("QuestPumpFeedback", "L", L["settings.name"], { escape = "special" })
	local v = Kit.Head(f.body, L["ui.feedback.title"], L["ui.feedback.reply"], f.tier)
	v.intro:SetWidth(v.contentW - 80)
	f.view = v
	-- while Send is off: the setup's companion card, its state and what turns it on
	local card = Kit.Card(v, v.contentW, 1)
	card.screen = Kit.Screen(card, FEEDBACK_SCREEN[1], FEEDBACK_SCREEN[2])
	card.screen:SetPoint("TOPLEFT", 12, -12)
	local icon = card.screen:CreateTexture(nil, "ARTWORK")
	icon:SetSize(40, 40)
	icon:SetPoint("CENTER")
	icon:SetTexture(Style.mark)
	card.name = Kit.Text(card, "cardTitle", "LEFT", nil, v.contentW - 24 - FEEDBACK_SCREEN[1] - 14)
	card.name:SetPoint("TOPLEFT", card.screen, "TOPRIGHT", 14, -2)
	card.name:SetText(L["ui.setup.step1"])
	card.mark = card:CreateTexture(nil, "ARTWORK")
	card.mark:SetSize(18, 18)
	card.mark:SetPoint("TOPLEFT", card.name, "BOTTOMLEFT", 0, -8)
	Style.SetMark(card.mark, false)
	card.state = Kit.Text(card, "large", "LEFT", { RED_FONT_COLOR:GetRGB() })
	card.state:SetPoint("LEFT", card.mark, "RIGHT", 8, 0)
	card.how = Kit.Text(card, "bodyWhite", "LEFT", Style.saga.note, v.contentW - 24)
	card.how:SetPoint("TOPLEFT", card.screen, "BOTTOMLEFT", 0, -10)
	Kit.Choose(card, false, true) -- a step not done: its name white, as the setup's card draws it
	f.card = card
	-- the text box at the player's text size (the template's own fonts are the small ones); its counter, the characters
	-- typed of the most it takes, under its right edge
	f.messageHead = Kit.Section(v, L["ui.feedback.message"])
	local scroll = CreateFrame("ScrollFrame", nil, v, "InputScrollFrameTemplate")
	scroll:SetSize(v.contentW - 18, FEEDBACK_BOX)
	local edit = scroll.EditBox
	edit:SetAutoFocus(false)
	edit:SetWidth(v.contentW - 30)
	edit:SetMaxLetters(ns.C.feedback_max_chars)
	Style.SetFont(edit, "bodyWhite")
	edit.Instructions:SetText(L["ui.feedback.placeholder"])
	edit.Instructions:SetWidth(v.contentW - 30)
	Style.SetFont(edit.Instructions, "bodyWhite")
	edit.Instructions:SetTextColor(DISABLED_FONT_COLOR:GetRGB())
	edit:HookScript("OnTextChanged", function(_, typing)
		if typing then f.handed = nil end -- the line after a Send stays until the player types again
		Settings_.RenderFeedback()
	end)
	edit:SetScript("OnEscapePressed", function() Kit.Open(f, false) end)
	f.scroll, f.edit, f.count = scroll, edit, scroll.CharCount
	f.count:ClearAllPoints()
	f.count:SetPoint("TOPRIGHT", scroll, "BOTTOMRIGHT", 0, -6)
	InputScrollFrame_OnTextChanged(edit) -- the template's placeholder, now that the box has its limit
	f.tick = Kit.Check(v, L["ui.feedback.tick"], function() Settings_.RenderFeedback() end)
	f.tick:SetChecked(true)
	f.note = Kit.Text(v, "bodyWhite", "LEFT", Style.saga.note, v.contentW)
	-- what goes, a line each, in a framed list that scrolls when the moves run long
	f.goesHead = Kit.Section(v, L["ui.feedback.goes"])
	f.goes = Kit.List(v, v.contentW)
	f.list = Kit.Scroll(f.goes)
	f.list:SetPoint("TOPLEFT", 12, -10)
	f.list:SetPoint("BOTTOMRIGHT", -12 - Kit.TIER.canvas.SCROLLBAR, 10)
	f.list.content:SetWidth(v.contentW - 24 - Kit.TIER.canvas.SCROLLBAR)
	f.numbers = Kit.Text(f.list.content, "bodyWhite", "LEFT", nil, v.contentW - 24 - Kit.TIER.canvas.SCROLLBAR)
	f.numbers:SetPoint("TOPLEFT")
	f.content = f.list.content
	-- the bar: after a Send, its message is going to the helper, then went (the notes' grey, at its left); Send
	f.foot = Kit.Footer(f)
	f.status = Kit.Text(f.foot, "bodyWhite", "LEFT", Style.saga.note, 420)
	f.status:SetPoint("BOTTOMLEFT", f.tier.IN, f.tier.BOTTOM)
	f.send = f.foot:Primary(L["ui.feedback.send"], function() Settings_.SendFeedback() end)
	Kit.Hover(f.send, Kit.Tip(L["ui.feedback.send"], L["status.feedback.sendTip"], { reason = function()
		return ns.HelperState() ~= "on" and L["status.feedback.sendOff"] or nil
	end }), "ANCHOR_TOP")
	f:HookScript("OnHide", function()
		edit:ClearFocus()
		f.handed = nil
	end)
	F.feedback = f
	return f
end

-- The text as typed, trimmed: the box gives a typed "|" back as "||", its escape.
local function typed(f)
	return strtrim((f.edit:GetText():gsub("||", "|")))
end

-- Send needs the helper running at this load, which carries the message, some text, and no message of the window's
-- still waiting for the strip.
local function sendable(f)
	return ns.HelperState() == "on" and not f.pending and typed(f) ~= ""
end

-- Why Send is off, by ns.HelperState (the setup screen's step for each state, ui.setup.step1.*, with /reload for its
-- Check again); nothing while the helper carries the message.
local FEEDBACK_OFF = { none = "status.feedback.none", off = "status.feedback.off", blind = "status.feedback.blind",
	stopped = "status.feedback.stopped", disabled = "status.feedback.disabled", outdated = "status.feedback.outdated" }

-- The window laid out: the card while Send is off (its state's words), the box and its counter, the tick and its note,
-- what goes by the tick, the bar's line of the last Send while it waits for the strip and once it went, and Send on
-- or off; the list takes the room the window's least height leaves.
function Settings_.RenderFeedback()
	local f = F.feedback
	if not f then return end
	local v, state = f.view, ns.HelperState() or "none"
	local why = FEEDBACK_OFF[state]
	local y = Kit.Top(v)
	f.card:SetShown(why ~= nil)
	if why then
		local card = f.card
		card.state:SetText(L["ui.setup.step1.state." .. state])
		card.how:SetText(L[why])
		card:SetHeight(12 + FEEDBACK_SCREEN[2] + 10 + card.how:GetStringHeight() + 14)
		Kit.At(card, v, y)
		y = y + card:GetHeight() + 16
	end
	Kit.At(f.messageHead, v, y)
	y = y + 26
	Kit.At(f.scroll, v, y)
	y = y + FEEDBACK_BOX
	f.count:SetText(L["ui.feedback.counter"]:format(f.edit:GetNumLetters(), ns.C.feedback_max_chars))
	Kit.At(f.tick, v, y + 10)
	y = y + 10 + f.tick:GetHeight() + 4
	local ticked = f.tick:GetChecked() and true or false
	if ticked ~= f.ticked then
		f.ticked = ticked
		f.note:SetText(L[ticked and "ui.feedback.note" or "ui.feedback.note.off"])
		f.numbers:SetText(ticked and f.goesAll or f.goesFrom)
		f.content:SetHeight(math.max(1, f.numbers:GetStringHeight()))
	end
	Kit.At(f.note, v, y)
	y = y + f.note:GetStringHeight() + 16
	Kit.At(f.goesHead, v, y)
	y = y + 26
	f.status:SetText(f.pending and L["status.feedback.pending"] or f.handed and L["status.feedback.handed"] or "")
	local foot = f.foot:Lay()
	local listH = math.max(FEEDBACK_LIST, f.tier.H - foot - 20 - y)
	f.goes:SetHeight(listH)
	Kit.At(f.goes, v, y)
	f.send:SetEnabled(sendable(f))
	Kit.Fit(f, y + listH, f.foot)
end

function Settings_.OpenFeedback()
	if not ns.state then return end
	if fighting() then return ns.Print("chat.combat") end
	local f = feedbackFrame()
	f.counted = ns.FeedbackNumbers(ns.state, ns.now())
	f.goesFrom = L["ui.feedback.from"]:format(ns.version)
	f.goesAll = f.goesFrom .. "\n" .. table.concat(feedbackLines(f.counted), "\n")
	f.ticked = nil -- new counts: RenderFeedback lays them out again
	-- an open pump stays hidden while the window shows, as under Options (Settings_.Open; Events.lua MENU_WINDOWS)
	ns.Lifecycle.MenuShown()
	Kit.Open(f)
	ns.Lifecycle.MenuHidden()
	Settings_.RenderFeedback()
	-- the box takes the keys only while Send can work: otherwise the card comes first
	if ns.HelperState() == "on" then f.edit:SetFocus() end
end

-- The message as the contract's GameFeedback: the text, the addon's version, when Send was pressed and, ticked, the
-- numbers the window showed. The window stays open and says the message is going; the box keeps the text, and the
-- lines that say it went (in the window while it shows, and in chat) wait, until the message's last chunk is on the
-- strip (a /reload before then loses no text the game said it handed over). A text changed in the box meanwhile stays.
function Settings_.SendFeedback()
	local f = F.feedback
	if not (f and sendable(f)) then return end
	local numbers = f.tick:GetChecked() and f.counted or nil
	local sent = f.edit:GetText()
	local message = { text = typed(f), version = ns.version, at = ns.now(), numbers = numbers }
	f.pending, f.handed = true, nil -- before the hand-over: a one-chunk message on an idle strip paints at once
	local queued = ns.Bridge.SendFeedback(message, function()
		f.pending = nil
		ns.Log("I", "feedback handed to the helper (%s)", numbers and "with the numbers" or "the text only")
		if f.edit:GetText() == sent then
			f.edit:SetText("")
			f.tick:SetChecked(true)
		end
		f.handed = f:IsShown() or nil
		Settings_.RenderFeedback()
		ns.Print("status.feedback.handed")
	end)
	if not queued then f.pending = nil end
	Settings_.RenderFeedback()
end

ns.AddCommand("feedback", function() Settings_.OpenFeedback() end, "cmd.feedback.help")

-- QuestPump's row in Options > AddOns opens and closes its pages, and nothing else (David, 2026-10-09: "That should
-- only be for expanding and retracting"). Blizzard's row click selects its category (SettingsCategoryListMixin:OnLoad's
-- row initializer) and only the +/- toggle at its left opens it (SettingsCategoryListButtonMixin:OnLoad;
-- Blizzard_CategoryList.lua lines 193-211 and 38-44 on the forever branch at 70291, read 2026-10-09), so the toggle's
-- hit area covers QuestPump's whole row and a click there is Blizzard's own toggle. QuestPump's code runs only in the
-- list's callback as it lays the rows out, which CallbackRegistry calls through its secure dispatch, the event key
-- inserted behind its taint barrier (CallbackRegistry.lua). The rows are pooled: one later bound to another category
-- gets its own area back.
local wideRows = setmetatable({}, { __mode = "k" }) -- a row seen showing QuestPump: true while it does

-- The row's rectangle, measured from both frames' edges, so a toggle a skin moves or resizes is followed (ElvUI's
-- Settings skin sets it to 18 and nudges it after the list's update: ElvUI/Game/Shared/Skins/SettingsPanel.lua, read
-- 2026-10-09); while the list has not placed them the edges are nil, and the size hooks fit it once it has.
local function fitToggle(row)
	local toggle = row.Toggle
	local left, right, top, bottom = row:GetLeft(), row:GetRight(), row:GetTop(), row:GetBottom()
	local tLeft, tRight, tTop, tBottom = toggle:GetLeft(), toggle:GetRight(), toggle:GetTop(), toggle:GetBottom()
	if not (left and tLeft) then return end
	toggle:SetHitRectInsets(left - tLeft, tRight - right, tTop - top, bottom - tBottom)
end

local function wideToggle(_, row, elementData)
	if not row.Toggle then return end -- a header or a spacer
	local data = elementData and elementData.data
	if data and data.category == Settings_.category then
		-- the row's width follows the list's scroll bar, which shows when the AddOns list grows past the panel
		if wideRows[row] == nil then
			local function refit() if wideRows[row] then fitToggle(row) end end
			row:HookScript("OnSizeChanged", refit)
			row.Toggle:HookScript("OnSizeChanged", refit)
		end
		wideRows[row] = true
		fitToggle(row)
	elseif wideRows[row] then
		wideRows[row] = false
		row.Toggle:SetHitRectInsets(0, 0, 0, 0)
	end
end

-- Registered once at PLAYER_LOGIN, after ns.BindDB: the profile and the fold exist then. The list lays QuestPump's row
-- out again at each rebuild (the AddOns tab, a toggle, an open to a page), all after the row's callback joins it.
function Settings_.Register()
	if Settings_.category then return end
	local category = Settings.RegisterCanvasLayoutCategory(buildTop(), L["settings.name"])
	Settings_.category = category
	local build = { general = buildGeneral, equipment = buildEquipment, exercises = buildExercises, body = buildBody,
		social = buildSocial, data = buildData }
	Settings_.pages = {}
	for _, key in ipairs(Settings_.PAGES) do
		Settings_.pages[key] = Settings.RegisterCanvasLayoutSubcategory(category, build[key](), L["settings." .. key])
	end
	Settings.RegisterAddOnCategory(category)
	ScrollUtil.AddInitializedFrameCallback(SettingsPanel.CategoryList.ScrollBox, wideToggle, Settings_)
end

-- Opens QuestPump's settings at the General page as the game menu would, or at another of its pages by its key
-- (`page`: "body", a pain refusal's link on the Talents tab's card, Tree.lua, where the pain flags are; "social", the
-- Guild tab's Social settings, SheetPages.lua, where others' cards and instances switch): a pump on the screen hides
-- and a running move pauses until the Settings panel closes, a change re-fits the moves not yet begun, and a record
-- cleared or a ladder started over aims the pump's moves it took again (Events.lua).
function Settings_.Open(page)
	if InCombatLockdown() then return ns.Print("chat.combat") end
	local pages = Settings_.pages
	local category = pages and (pages[page or "general"] or pages.general)
	if not category then return end
	ns.Lifecycle.MenuShown()
	Settings.OpenToCategory(category:GetID())
	ns.Lifecycle.MenuHidden()
end

ns.AddCommand("config", function() Settings_.Open() end, "cmd.config.help")

-- A change of the profile or of the save (an import, a reset, a restore, a record cleared, a count corrected) redraws
-- the pages that read them, and Correct a count's window while it shows (its list, a clear's line).
for _, message in ipairs({ "Settings", "DB" }) do
	ns.On(message, function()
		Settings_.RenderGeneral()
		Settings_.RenderEquipment()
		Settings_.RenderExercises()
		Settings_.RenderBody()
		Settings_.RenderSocial()
		Settings_.RenderData()
		Settings_.RenderCorrect()
	end)
end
ns.On("Social", function() Settings_.RenderSocial() end)
ns.RegisterEvent("PLAYER_LOGIN", function() Settings_.Register() end)
-- A Text Size or UI scale change while a page or a window shows (Kit.OnRelayout): each that shows draws again.
Kit.OnRelayout(function()
	local renders = { general = Settings_.RenderGeneral, equipment = Settings_.RenderEquipment,
		exercises = Settings_.RenderExercises, body = Settings_.RenderBody, social = Settings_.RenderSocial,
		data = Settings_.RenderData, io = Settings_.RenderIO, correct = Settings_.RenderCorrect, feedback = function()
			F.feedback.ticked = nil -- the list of what goes measured again
			Settings_.RenderFeedback()
		end }
	for key, render in pairs(renders) do
		if F[key] and F[key]:IsVisible() then render() end
	end
end)
-- the feedback box never holds the keyboard into a fight (its OnHide clears the focus; the export window is the kit's
-- fight window): the window keeps its text for the next open and says so once, when the fight closes it on words not
-- yet sent; Send feedback and the Data page's rows that open a window or write a row grey for the fight and come back
-- after it
ns.RegisterEvent("PLAYER_REGEN_ENABLED", function()
	Settings_.RenderGeneral()
	Settings_.RenderData()
end)
ns.RegisterEvent("PLAYER_REGEN_DISABLED", function()
	Settings_.RenderGeneral()
	Settings_.RenderData()
	local f = F.feedback
	if f and f:IsShown() then
		f:Hide()
		if not f.pending and typed(f) ~= "" then ns.Print("status.feedback.kept") end
	end
end)
