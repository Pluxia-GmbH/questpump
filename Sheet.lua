-- The character sheet of the body, the HUD and the toasts (docs/plan.md slice 3; docs/ui.md "The character sheet").
-- The sheet copies Forever's Character window: a 631 × 484 portrait frame with side tabs (Body, Talents, Today, Log,
-- Records, Deeds, and Guild while the character is in a guild; Talents is Tree.lua's, slice 37; the order and the Log's
-- own tab since 2026-10-16) that docks in the Character window's place from its own QuestPump tab, sixteen muscle
-- buttons in the gear slots and Heart, Breath and Balance in the weapon row around the player's own character (the body
-- map's art plugs in through Style.bodymap), and Forever's stat pane with the level, the XP bar, Primary Attributes,
-- Modifiers and Defense (its Stats and Titles sidebar went on 2026-10-16: the titles are worn from Deeds). The list
-- pages (the Log, Records, Today, Deeds and Guild) are SheetPages.lua's since 2026-10-16. The HUD stands under the
-- objective tracker with the QP line, the pump chip and what needs saying; the toasts ride the AlertFrame. Every
-- number comes from the fold (Stats.lua): nothing here counts anything. It says unfinished, never owed or debt.

local ADDON, ns = ...
local C, L = ns.C, ns.L
local Q, O = ns.Enum.quality, ns.Enum.outcome
local Lc, Style = ns.Lifecycle, ns.Style

local Sheet = { frames = {}, muscles = {}, pages = {}, tabs = {} }
ns.Sheet = Sheet
local HUD = { lines = {} }
ns.HUD = HUD
local T = ns.Timers()
local F = Sheet.frames

local WIDTH, HEIGHT, LEFT_W, RIGHT_W = 631, 484, 398, 233
local SLOT, PITCH = 37, 43
-- The Body page's stage between the muscle columns (docs/ui.md, "The body map"): 5 units from each column, under the
-- mode strip at the top and above the bottom row; both figures stand in it at full height.
local STAGE_GAP, STAGE_TOP, STAGE_BOTTOM = 5, 32, 52
local HOVER_TICK_S = 0.05 -- the map's cursor sample while the mouse is over it
-- the HUD: its width, a line's height and the lines it holds (11 since slice 23's pact and duel lines; 9 until then)
local HUD_W, LINE_H, MAX_LINES = 260, 16, 11
local CHIP = { settling = true, ready = true, card = true } -- a pump waiting out a fight has none: the HUD hides then
local HEART, BREATH, BALANCE = 17, 18, 19
-- the side tabs top down (2026-10-16: Today after Talents, where the Start lives, and the Log a tab of its own); Guild
-- shows only in a guild (layoutTabs)
local TABS = { "body", "ladders", "today", "log", "records", "deeds", "guild" }
-- The side tabs as on Forever's Character window (CharacterFrame.xml's ModeTabs and UpdateTabLayout, 70124): the first
-- 30 px down from the frame's top right corner, each under the one before, 2 px apart.
local TAB_Y, TAB_GAP = -30, -2
-- Blizzard's own stat names (SPELL_STAT1_NAME … 5), so the pane is localised for free: 1 Strength, 2 Agility, 3
-- Stamina, 4 Intellect, 5 Spirit.
local STAT_INDEX = { strength = 1, agility = 2, stamina = 3, intellect = 4, spirit = 5 }
local FAM_WORD = { [0] = "new", "tried", "familiar", "learned" }

-- Small helpers -----------------------------------------------------------------------------------------------------

local function statName(key)
	return _G["SPELL_STAT" .. STAT_INDEX[key] .. "_NAME"] or L["stat." .. key]
end
Sheet.StatName = statName

local function moveName(id)
	return L["move." .. ns.Library[id].key]
end
Sheet.MoveName = moveName -- the Talents tab's too (Tree.lua)

-- The colour of an item quality, following the player's colour overrides (never a hard-coded hex).
local function qualityColor(q)
	local c = q and ColorManager and ColorManager.GetColorDataForItemQuality
		and ColorManager.GetColorDataForItemQuality(q)
	return c and c.color or c or (q and ITEM_QUALITY_COLORS and ITEM_QUALITY_COLORS[q] and ITEM_QUALITY_COLORS[q].color)
		or NORMAL_FONT_COLOR
end
Sheet.QualityColor = qualityColor

local function colored(color, text)
	return color and color.WrapTextInColorCode and color:WrapTextInColorCode(text) or text
end
Sheet.Colored = colored -- the list pages' too (SheetPages.lua)

-- A quality's name, Poor shown as "Starter" (a WoW player reads grey as junk; David, 2026-10-03), in its colour.
local function qualityWord(q)
	return colored(qualityColor(q), L["quality." .. q])
end
Sheet.QualityWord = qualityWord -- the Talents tab's dividers and card (Tree.lua)

local function number(n)
	return BreakUpLargeNumbers(math.floor((n or 0) + 0.5))
end
Sheet.Number = number -- the list pages' too (SheetPages.lua)

-- A count as the player reads it: seconds on a hold ("45 s"), a plain number otherwise.
function Sheet.Amount(v, hold)
	return hold and L["unit.seconds"]:format(v) or tostring(v)
end

-- A move's count as the player reads it; a countable cardio form's is a count in count_s ("12 in 30 s", 2026-10-05).
local function moveAmount(moveId, v)
	local m = ns.Library[moveId]
	if m.count then return L["unit.countIn"]:format(v, C.count_s) end
	return Sheet.Amount(v, m.dose == "hold")
end
Sheet.MoveAmount = moveAmount -- the Talents tab's proof lines too (Tree.lua)

-- A move's top count as its climb asks it: a loaded move's with the weight its climb asks (ns.ClimbKg, 2026-10-12: "20
-- with 15 kg or more"), any other move's count alone; `goal` reads it as a goal ("20+"). A field of Sheet, as the
-- file's main chunk holds as many locals as Lua allows.
function Sheet.ClimbTop(moveId, v, goal)
	local amount = moveAmount(moveId, v)
	if goal then amount = L["amount.goal"]:format(amount) end
	local kg, each = ns.ClimbKg(moveId)
	if not kg then return amount end
	return L[each and "amount.climbEach" or "amount.climb"]:format(amount, kg)
end

-- A weight as the player reads it: tenths of a kg (the load row's unit, ns.LoadRow) with one decimal, the decimal
-- dropped when 0 (125 → "12.5", 120 → "12"), in whole-number arithmetic as the phones and the desktop app format it.
function Sheet.Kg(tenths)
	local whole, tenth = math.floor(tenths / 10), tenths % 10
	return tenth == 0 and tostring(whole) or whole .. "." .. tenth
end

-- A typed weight in tenths of a kg: digits with one "." or "," ("12,5" and "12.5" are 125), a second decimal rounding
-- the first (12.46 → 125); nil for 0, nothing or anything else (no weight: the set counts reps alone).
function Sheet.ParseKg(text)
	local whole, frac = (text or ""):match("^%s*(%d*)[.,]?(%d*)%s*$")
	if not whole or whole .. frac == "" then return nil end
	local tenths = (tonumber(whole) or 0) * 10
	if frac ~= "" then tenths = tenths + math.floor((tonumber((frac .. "0"):sub(1, 2)) + 5) / 10) end
	return tenths > 0 and tenths or nil
end

-- A set of a loaded move with its weight ("11 kg × 16", "6 kg each × 12"; ns.Loaded); a set without one, the count.
function Sheet.LoadAmount(moveId, load, reps)
	if not load then return moveAmount(moveId, reps) end
	return L[ns.Loaded(moveId) == "each" and "ui.load.amountEach" or "ui.load.amount"]:format(Sheet.Kg(load), reps)
end

-- The steady ledger's best M (thousandths, slice 52; P until then) as how much longer the cardio moves run than at
-- the start, in percent.
local function cardioLonger(m)
	return math.floor((m - C.ledger_m[1]) / 10 + 0.5)
end

-- Moves this week on a muscle: a secondary counts half ("2.5 moves").
local function weekMoves(n)
	return ns.Plural("tip.muscle.moves", n):format(n % 1 == 0 and tostring(n) or ("%.1f"):format(n))
end

local function clockTime(ts)
	return date("%H:%M", ts)
end
Sheet.ClockTime = clockTime -- the Talents tab's day and fresh lines (Tree.lua)

-- A date as the phones write it, the day without a leading zero ("4 Oct").
local function shortDate(ts)
	return ("%d %s"):format(date("*t", ts).day, date("%b", ts))
end
Sheet.ShortDate = shortDate

-- A date with its year, for what is kept for good (a Feat of Strength).
local function fullDate(ts)
	return shortDate(ts) .. date(" %Y", ts)
end
Sheet.FullDate = fullDate -- a Feat's date on the Deeds tab (SheetPages.lua)

-- A game day ("YYYY-MM-DD", ns.day) with its weekday, as the Saga writes it ("Thu 2 Oct"; the day card, the best day
-- on the Records tab), with its year when that is not this game day's ("Thu 2 Oct 2026", as the apps write it), read
-- at the day's middle so a clock change never moves it. A field of Sheet, as the file's main chunk holds as many
-- locals as Lua allows.
function Sheet.DayDate(day)
	local today = ns.day()
	local mid = ns.DayMid(ns.now(), ns.DayIndex(day) - ns.DayIndex(today))
	return date("%a ", mid) .. (day:sub(1, 4) == today:sub(1, 4) and shortDate(mid) or fullDate(mid))
end

local text = Style.Text

local function inCombat()
	return Lc.combat or InCombatLockdown()
end

local function filter()
	return ns.FilterContext(ns.profile, ns.state)
end

-- Whether the QP line and the plate show: once setup is behind the player.
local function qpShown()
	return ns.state ~= nil and Lc.TrialBegun()
end

-- Chat (General "Chat messages": off, warnings, all): slash-command answers always print; toasts only on "all", the
-- rare warning, a companion's nudge (Setup.Nudge, which keeps its own cap), a newer addon's line (Lc.TellUpdate, one a
-- session) and a step card's Start refused at its click (Tree.Start: the card says it in red too) unless "off".
-- At most one warning a session (plan slice 3, "In game"): the rare line in chat, never a stream.
local warned = false

function ns.Chat(kind, line)
	local mode = ns.profile and ns.profile.chat or "warnings"
	if mode == "off" or (kind == "toast" and mode ~= "all") then return end
	if kind == "warning" then
		if warned then return end
		warned = true
	end
	print(L["chat.prefix"] .. " " .. line)
end

-- Move tooltips, in item-tooltip grammar (docs/ui.md, "Move tooltips everywhere") ---------------------------------

-- The hover preview (David, 2026-10-03: hovering a move "should auto play that exercise in a larger window"): the move
-- figure at the pump card's size inside the move's tooltip, under its name, so every move tooltip carries it (the
-- flyout, the Records tab, the Exercises page, the pump card's backup) and it goes when the tooltip clears
-- (SharedTooltip_ClearInsertedFrames hides it, and a hidden figure stops its OnUpdate). The key pose shows at once,
-- the motion once the cursor rests on the move, so a sweep down a list loads no move's sheets.
local preview

local function showPreview(tooltip, moveId)
	if not preview then
		preview = ns.Figure.Create(UIParent, C.figure_tip_px)
		preview.frame:EnableMouse(false) -- a tooltip takes no mouse, and neither does its figure
	end
	-- GameTooltip_InsertFrame reads the frame's height before it clears its anchors, and the figure is still anchored
	-- to the tooltip line it took last time: a line that has since held secret text (a unit's name) makes what is
	-- anchored to it secret too (12.0, ScriptRegion:IsAnchoringSecret), and the arithmetic on it raised (David,
	-- 2026-10-04). Freed and sized again it measures plainly; a value still secret leaves this tooltip without one.
	preview.frame:ClearAllPoints()
	preview.frame:SetSize(preview.size, preview.size)
	local line = tooltip:GetLeftLine(2)
	if not (ns.Readable(preview.frame:GetHeight()) and line and ns.Readable(line:GetLineHeight())) then return end
	preview.previewFor = moveId
	GameTooltip_InsertFrame(tooltip, preview.frame, 4)
	preview:ShowIcon(moveId)
	T.after("preview", C.figure_preview_s, function()
		if preview.previewFor == moveId and preview.frame:IsVisible() then preview:Play(moveId, "loop") end
	end)
end

-- `card`: the pump card's move card, which leaves out the name the view's title shows and the figure the card plays
-- above it; the cue stays, its one place on the card since 2026-10-16. A move above the player's reach shows its
-- figure, cue and stop line under the line that says so, and still never starts on its own (David, 2026-10-14: "it
-- should not be locked. you can still watch future excercises"; slice 36's design). Since 2026-10-16 in Kit.Tip's order
-- and colours (docs/ui.md, "Move tooltips everywhere"): the name in its quality's colour, the figure, the step and what
-- it trains, its cue in white, the facts (the record, the stat, how well it is known) in the notes' grey, one state
-- line, and the hints in green last.
-- `hints`: { click = "talents" (the flyout: its step's card on the Talents tab) or "card" (a card's tile), menu = true
-- (a right-click opens the move's menu), record = true (a right-click opens its record menu, Correct a count and the
-- clear: the Records tab, Settings.lua Settings_.RecordHint), span = true (the record's first count to now: the
-- Records tab) }.
function Sheet.MoveTooltip(tooltip, moveId, card, hints)
	local state = ns.state
	local m = ns.Library[moveId]
	local f = filter()
	local locked = ns.AboveReach(state, moveId)
	local q = ns.MoveQuality(moveId) -- a move no ladder places has none: white, as a spell's name
	if not card then
		GameTooltip_SetTitle(tooltip, moveName(moveId), q and qualityColor(q) or HIGHLIGHT_FONT_COLOR)
		showPreview(tooltip, moveId)
	end
	local where = m.ladder and L["ladder." .. m.ladder] or L["pattern." .. m.pattern]
	local missing = {}
	for _, need in ipairs(m.needs or {}) do
		local names = type(need) == "table" and need or { need }
		local have = false
		for _, key in ipairs(names) do have = have or (ns.profile and ns.profile.gear[key]) or false end
		if not have then missing[#missing + 1] = L["gear." .. names[1]] end
	end
	local requires = #missing > 0 and table.concat(missing, ", ") or nil
	local r, g, b = 1, 1, 1
	if requires then r, g, b = RED_FONT_COLOR:GetRGB() end
	-- beside its colour a move names its step on the one scale, as its unlock toast and the phones do (the quality's
	-- word in white: the name above wears its colour); a move no ladder places names only where it belongs
	local stepLine = q and L["tip.move.step"]:format(where, ns.MoveStep(moveId), L["quality." .. q]) or where
	tooltip:AddDoubleLine(stepLine, requires or "", 1, 1, 1, r, g, b)
	local trains, also = {}, {}
	for _, id in ipairs(m.mp) do trains[#trains + 1] = L["muscle." .. id] end
	for _, id in ipairs(m.ms) do also[#also + 1] = L["muscle." .. id] end
	if #trains > 0 then
		tooltip:AddLine(#also > 0 and L["tip.move.trainsAlso"]:format(table.concat(trains, ", "), table.concat(also, ", "))
			or L["tip.move.trains"]:format(table.concat(trains, ", ")), 1, 1, 1, true)
	end
	tooltip:AddLine((ITEM_SPELL_TRIGGER_ONUSE or L["tip.move.use"]) .. " " .. L["move." .. m.key .. ".cue"], 1, 1, 1, true)
	local note = Style.saga.note
	if locked then
		tooltip:AddLine(L["tip.move.locked"], note[1], note[2], note[3], true)
		local stop = ns.Panel.MoveStop(moveId)
		if not card and stop then tooltip:AddLine(stop, 1, 1, 1, true) end
	end
	local best, last = ns.Record(state, moveId)
	if best and ns.HoldsRecord(state, moveId) then -- none for a move that turned timed, whose old sets stay
		-- a loaded move's record is its weighted one once a set had a weight, the last set with its own (ns.LoadRecord);
		-- the star stays a count, as the rung windows read reps at any weight
		local topLoad, topReps, lastLoad, lastReps = ns.LoadRecord(state, moveId)
		local line = topLoad and L["tip.move.record"]:format(Sheet.LoadAmount(moveId, topLoad, topReps),
			Sheet.LoadAmount(moveId, lastLoad, lastReps or last or topReps))
			or L["tip.move.record"]:format(moveAmount(moveId, best), moveAmount(moveId, last or best))
		if m.ladder and ns.STRENGTH[m.ladder] then
			local at, stars = ns.NextStar(m.ladder, m.rung, moveId, best)
			if at then line = line .. " · " .. L["tip.move.star"]:format(Style.Stars(stars), moveAmount(moveId, at)) end
		end
		tooltip:AddLine(line, note[1], note[2], note[3])
		local first, firstTs, top
		if hints and hints.span then first, firstTs, top = ns.RecordSpan(state, moveId) end
		if first then
			tooltip:AddLine(L["records.line"]:format(moveAmount(moveId, first), moveAmount(moveId, top), shortDate(firstTs)),
				note[1], note[2], note[3])
		end
	end
	-- cardio and flow moves have no record, but a countable cardio form's count is one (2026-10-05)
	local counted = m.dose == "rep" or m.dose == "hold" or m.count ~= nil
	tooltip:AddLine(L[counted and "tip.move.stat" or "tip.move.statLearn"]:format(statName(ns.MoveStat(moveId))),
		note[1], note[2], note[3])
	-- familiarity in plain words, no colour of its own: the colours mean the quality (slice 20)
	local step, days = ns.Familiarity(state, moveId)
	local word = L["fam." .. FAM_WORD[step]]
	tooltip:AddLine(step == 2 and L["tip.move.familiar"]:format(word, days, C.fam_learned_days)
		or L["tip.move.fam"]:format(word), note[1], note[2], note[3])
	-- one state line: left out of the pumps, set aside for a move the player's gear does better (ns.BetterOpen; the
	-- Exercises page's rows show this tooltip too), or a favourite (gold: chosen); never red or orange
	local better = ns.BetterOpen(moveId, f)
	if not ns.Passes(moveId, f) then
		tooltip:AddLine(L["tip.move.excluded"], note[1], note[2], note[3])
	elseif better then
		tooltip:AddLine(L["tip.move.better"]:format(moveName(better)), note[1], note[2], note[3])
	elseif ns.profile and ns.profile.favourites[moveId] then
		tooltip:AddLine(L["tip.move.favourite"], NORMAL_FONT_COLOR:GetRGB())
	end
	hints = hints or {}
	if hints.click == "talents" then
		GameTooltip_AddInstructionLine(tooltip, L["tip.move.click"])
	elseif hints.click == "card" then
		GameTooltip_AddInstructionLine(tooltip, L["tree.click.card"])
	end
	if hints.menu then GameTooltip_AddInstructionLine(tooltip, L["tip.move.menu"]) end
	if hints.record then ns.Settings.RecordHint(tooltip, moveId) end
	tooltip:Show()
end

-- Toasts --------------------------------------------------------------------------------------------------------------

local toaster, legendToaster

-- A toast's or a banner row's icon: a texture file, or a move's key-pose cell ({ file, left, right, top, bottom }:
-- moveIcon); none, the XP icon.
local function setToastIcon(texture, icon)
	if type(icon) == "table" then
		texture:SetTexture(icon[1])
		texture:SetTexCoord(icon[2], icon[3], icon[4], icon[5])
	else
		texture:SetTexture(icon or Style.xpIcon)
		texture:SetTexCoord(0, 1, 0, 1)
	end
end

local function moveIcon(id)
	local file, l, r, t, b = Style.MoveIcon(id)
	return file and { file, l, r, t, b } or nil
end

-- A toast's and a banner row's right-click on its tooltip: the share line into chat where it has one, else close.
local function rightClickLine(tooltip, share)
	GameTooltip_AddInstructionLine(tooltip, L[share and "banner.tip.share" or "banner.tip.close"])
end

local function setUpToast(frame, icon, headline, subline, share)
	setToastIcon(frame.Icon, icon)
	if frame.Ring then -- the Legendary toast's art (QuestPumpLegendToastTemplate), each piece behind its guard
		Style.SetAtlas(frame.Background, "legendBG", true)
		Style.SetAtlas(frame.Ring, "legendRing", true)
		Style.SetAtlas(frame.Particles, "legendParticles", true)
		Style.SetAtlas(frame.Starglow, "legendGlow")
	end
	frame.Headline:SetText(headline)
	frame.Subline:SetText(subline or "")
	frame.share = share
	frame:SetScript("OnClick", function(self, button, down)
		if button == "RightButton" and self.share then ChatFrameUtil.OpenChat(self.share) end
		AlertFrame_OnClick(self, button, down)
	end)
	if not frame.tipHooked then -- the alert system reuses its frames: hooked once, beside its hover pause
		frame.tipHooked = true
		frame:HookScript("OnEnter", function(self)
			GameTooltip:SetOwner(self, "ANCHOR_BOTTOM")
			rightClickLine(GameTooltip, self.share)
			GameTooltip:Show()
		end)
		frame:HookScript("OnLeave", function() GameTooltip:Hide() end)
	end
end

-- One toast: the alert frame (the client may refuse an addon template on the beta: then the raid-warning line), a
-- sound behind the Sounds switch, and the chat line only on "all". `share` is the line a right-click puts in chat,
-- `rank` its place on the banner where its sound does not set one (rank, below); `quality` and `move`, for a win of a
-- move or a step, are its banner row's: the edge in the step's colour round its picture and the move's key pose in it
-- (the toast keeps its own icon); `moment`, for the day's moments (Sheet.Earned: Welcome back, the rest tokens that
-- kept the streak, the streak's day, Clean Slate), is what the banner tells in rows of their own in the toast's
-- place (Sheet.MomentRows, slice 72). The "legendary" sound takes the
-- Legendary toast's own frame where the client has its template and art (slice 22), else the plain one. No toast
-- carries a button: a toast goes by itself, and a button on it was gone before the player reached it (David,
-- 2026-10-03, of the Flex button: "random", "disappears too fast").
-- While the pump's dim shows, toasts and the fanfare wait for it to lift: under it the fanfare cannot be seen, and
-- the dim takes the mouse a toast's share click needs. A started pump that a fight or a popup hid keeps them too,
-- since it comes back and its banner carries them. They follow when the pump ends (release, below); the fanfare
-- waits out a fight on its own too.
local held = {}
-- the banner's places for a title (and a streak's milestone), an unlock and a record (rank, below)
local RANK_TITLE, RANK_UNLOCK, RANK_RECORD = 1, 2, 3

local function dimmed()
	local dim = ns.Panel and ns.Panel.frames.dim
	return dim ~= nil and dim:IsShown() == true
end

local function underWay()
	return Lc.pump ~= nil and Lc.pump.started == true
end

-- A pump offered on its "Stop to pump" card: what follows a banner (a Legendary frame, the level's fanfare) waits for
-- it as for the dim.
local function carded()
	return Lc.pump ~= nil and Lc.pump.state == "card"
end

-- Whether the pump's dim covers the HUD and the sheet: their XP bars fill only once it lifts, where the fill is seen
-- whole (slice 72); the unfinished count never waits (HUD.Rolled). The Streamer layout's dim hides nothing.
function HUD.Covered()
	return dimmed() and not ns.Panel.Streamer()
end

-- A toast as one line: its headline and subline (chat, the raid-warning fallback).
local function toastLine(headline, subline)
	return subline and subline ~= "" and (headline .. " · " .. subline) or headline
end

function Sheet.Toast(icon, headline, subline, share, sound, rank, quality, move, moment)
	local line = toastLine(headline, subline)
	if ns.profile and ns.profile.toasts == false then
		ns.Chat("toast", line)
		return
	end
	if dimmed() or underWay() then
		held[#held + 1] = { icon, headline, subline, share, sound, rank, quality, move, moment }
		return
	end
	-- AddAlert answers false when its queue is full (3 shown, 6 waiting) or alerts are ignored: the line falls back too
	local ok, added = false, nil
	local system = sound == "legendary" and legendToaster or toaster
	if system then ok, added = pcall(system.AddAlert, system, icon, headline, subline, share) end
	-- the fallback: RaidWarningUtil on 69913 (Blizzard_RaidWarning)
	if not (ok and added ~= false) and RaidWarningUtil and RaidWarningUtil.AddMessage then
		RaidWarningUtil.AddMessage(line, NORMAL_FONT_COLOR)
	end
	ns.Chat("toast", line)
	if sound then Style.Play(sound) end
end


-- What would open a ladder's next rung, in words ("tick Floor moves to open One-arm push-up"): ns.TopOut's anchor and
-- the filters that shut it, one locale key a reason (topout.<reason>; the stable chair, the table and the bar by keys
-- of their own, any other equipment by its checkbox's name; another ladder's step the rung waits for by the record
-- that does it, "reach a Pull-up record of 1", or on a pose or cardio step by the step's move and its ladder, "reach
-- Twisted triangle on your Twist ladder", ns.RungNeed).
local TOPOUT_GEAR = { stable_chair = true, table = true, bar = true }

function Sheet.Opens(move, why)
	local parts = {}
	for i, w in ipairs(why) do
		local need = type(w.need) == "table" and w.need[1] or w.need
		if w.reason == "need" and w.value then
			parts[i] = L["topout.need"]:format(moveName(w.move), moveAmount(w.move, w.value))
		elseif w.reason == "need" then
			parts[i] = L["topout.needStep"]:format(moveName(w.move), L["ladder." .. ns.Library[w.move].ladder])
		elseif need and TOPOUT_GEAR[need] then
			parts[i] = L["topout." .. need]
		elseif need then
			parts[i] = L["topout.gear"]:format(L["gear." .. need])
		else
			parts[i] = L["topout." .. w.reason]
		end
	end
	return L["topout.open"]:format(table.concat(parts, L["topout.and"]), moveName(move))
end

-- A ladder's top-out line (David, 2026-10-04): the rung this setup stops it at, by its step on the one scale, and what
-- would open the next one. Chat says it once when the climb comes due, and the Exercises page's warnings carry it
-- (Settings.lua).
function Sheet.TopOutLine(ladder, rung, move, why)
	return L["topout.line"]:format(L["ladder." .. ladder], ns.Step(ladder, rung), Sheet.Opens(move, why))
end

-- A rung up's sound (its rungUp event): by its colour, the rarer the step, the louder (green the world-quest chime,
-- blue the new-record fanfare, purple the epic-loot toast), and the small chime for a rung earned again (Rekindled) or
-- one its climb only opens; the unlock toast's and the step-up reveal's after a hold's pour (Panel.lua).
function Sheet.UnlockSound(e)
	local q = e.quality or Q.starter
	if e.rekindled or e.opened then return "win" end
	return q >= Q.epic and "title" or q >= Q.rare and "record" or q >= Q.uncommon and "star" or "win"
end

-- What a result moved (ns.Progression's events): a first record and the Trial's steps, a star, a rung, a ladder that
-- tops out with this setup, a cardio step. The result view's own sound already marks a record. A win of a move or a
-- step names its colour and its move (a step's anchor where the event names none) for its banner row.
function HUD.Progress(events)
	for _, e in ipairs(events) do
		local name = e.move and moveName(e.move)
		local pictured = e.move or (e.ladder and e.rung and ns.Anchor(e.ladder, e.rung))
		if e.kind == "trial" then
			local amount = moveAmount(e.move, e.value or 0)
			-- a first record is a record (slice 72): its sound as a toast and its place on the banner
			Sheet.Toast(nil, L["toast.firstRecord"]:format(name, amount), nil, ns.profile.shareCount
				and L["share.first.count"]:format(name, amount) or L["share.first"]:format(name), "record", RANK_RECORD,
				ns.MoveQuality(e.move), e.move)
			-- the Trial holds as many ladders as the filters leave a move for
			if e.trial and e.placed == e.total then
				Sheet.Toast(nil, ns.Plural("toast.trialComplete", e.total):format(e.total), nil, nil, "title")
			end
		elseif e.kind == "star" then
			Sheet.Toast(nil, L["toast.star"]:format(e.stars, name), nil, ns.Plural("share.star", e.stars):format(e.stars,
				name), "star", nil, ns.MoveQuality(e.move), e.move)
		elseif e.kind == "rungUp" then
			-- an unlock in its colour (slice 20): the rarer the step, the louder (Sheet.UnlockSound; the epic-loot toast
			-- also leads the pump's banner) and blue and up with a share line; a rung earned again after a rung down is a
			-- smaller win, "Rekindled"
			local q = e.quality or Q.starter
			local label = name or L["ladder." .. e.ladder]
			local sub = L["toast.unlock.sub"]:format(L["ladder." .. e.ladder], ns.Step(e.ladder, e.rung),
				qualityWord(q))
			if e.rekindled then
				Sheet.Toast(nil, L["toast.rekindled"]:format(label), sub, nil, "win", RANK_UNLOCK, q, pictured)
			elseif e.opened then
				-- an orange or gold rung its climb only opens (2026-10-09): a quiet unlock that says what earns it, one set
				-- at its floor (ns.EarnFloor; a pose family's mark answered Perfect); its moment waits for that set
				-- ("earned"); a strength rung whose standing move has no floor to name says its step and colour
				local earn = e.floor and L["toast.unlock.earn"]:format(moveAmount(e.move, e.floor))
					or ns.POSE[e.ladder] and L["toast.unlock.earnMark"] or sub
				Sheet.Toast(nil, L["toast.unlock"]:format(colored(qualityColor(q), label)), earn, nil, "win", RANK_UNLOCK, q,
					pictured)
			else
				Sheet.Toast(nil, L["toast.unlock"]:format(colored(qualityColor(q), label)), sub,
					q >= Q.rare and L["share.unlock"]:format(label, L["quality." .. q]) or nil, Sheet.UnlockSound(e), RANK_UNLOCK,
					q, pictured)
			end
		elseif e.kind == "earned" then
			-- an orange or gold rung a set earned (slice 22; gold since 2026-10-05; on the set since 2026-10-09): its own
			-- toast and sound, leading the pump's banner, then the move in its colour's light with the voice naming it
			-- (Sheet.Legend), and the line to the party and the guild for a player who chose to send it
			-- (Social.Legend); from now on a Feat of Strength (ns.Feats); earned again after the player started the
			-- ladder over, Rekindled
			local q = e.quality or Q.legendary
			local label = name or L["ladder." .. e.ladder]
			local sub = L["toast.unlock.sub"]:format(L["ladder." .. e.ladder], ns.Step(e.ladder, e.rung),
				qualityWord(q))
			if e.rekindled then
				Sheet.Toast(nil, L["toast.rekindled"]:format(label), sub, nil, "win", RANK_UNLOCK, q, pictured)
			else
				Sheet.Toast(e.move and moveIcon(e.move), L["toast.legendary"]:format(L["quality." .. q],
					colored(qualityColor(q), label)), sub,
					L["share.unlock"]:format(label, L["quality." .. q]), "legendary", nil, q, pictured)
				Sheet.Legend(e)
				if e.move then ns.Social.Legend(e.move) end
			end
		elseif e.kind == "rungDown" then
			ns.Chat("warning", L["chat.rungDown"]:format(L["ladder." .. e.ladder], ns.Step(e.ladder, e.rung)))
		elseif e.kind == "topout" then
			ns.Chat("warning", Sheet.TopOutLine(e.ladder, e.rung, e.move, e.why))
		elseif e.kind == "opened" then
			-- a family that waited for this rung is open (ns.Open): its ladder joins the pumps from the next one
			Sheet.Toast(nil, L["toast.opened"]:format(L["ladder." .. e.ladder]), L["toast.opened.sub"], nil, "star",
				RANK_UNLOCK)
		elseif e.kind == "mastered" then
			-- a step whose every move the setup offers reached its top (slice 42, ns.Mastered: a reward, never a gate),
			-- once, as the set that topped the last move saves: "Squat step 6 mastered" in the unlock toast's frame with
			-- the small chime, the step's anchor pictured in its colour (docs/ui.md, "Mastered")
			local q = ns.Quality(e.ladder, e.rung)
			Sheet.Toast(nil, L["toast.mastered"]:format(L["ladder." .. e.ladder], ns.Step(e.ladder, e.rung)),
				L["toast.mastered.sub"], nil, "win", RANK_UNLOCK, q, ns.Anchor(e.ladder, e.rung))
		elseif e.kind == "check" then
			-- a stretch check passed above its family's step (slice 42): the family lifted to the check's step, a placement
			-- (cause check) in the step's colour; the Result names what the check opened
			Sheet.Toast(nil, L["toast.checkPlaced"]:format(L["ladder." .. e.ladder], ns.Step(e.ladder, e.rung)),
				L["toast.check.sub"], nil, "star", RANK_UNLOCK, ns.Quality(e.ladder, e.rung), e.move)
		elseif e.kind == "path" then
			-- the days at the top are in, and the climb still asks for its path (2026-10-16; its proof until then): the
			-- first stage still out, its move at its top on its days, one the setup leaves out with what would open it; a
			-- pose with the game days of its goal pose, a countable form with its clean windows
			local ladder = L["ladder." .. e.ladder]
			if e.why then
				ns.Chat("warning", L["chat.path.shut"]:format(ladder, Sheet.Opens(e.move, e.why)))
			elseif ns.POSE[e.ladder] then
				local perfect = e.value == ns.Enum.mark.perfect
				ns.Chat("warning", L[perfect and "chat.path.perfect" or "chat.path.mark"]:format(ladder, moveName(e.move),
					e.days))
			elseif e.ladder == ns.CARDIO then
				ns.Chat("warning", L["chat.path.clean"]:format(ladder, moveName(e.move), e.days))
			else
				ns.Chat("warning", L["chat.path"]:format(ladder, moveName(e.move), moveAmount(e.move, e.value),
					ns.Plural("tree.days", e.days):format(e.days)))
			end
		elseif e.kind == "p" then
			Sheet.Toast(nil, L["toast.p"]:format(cardioLonger(e.value)))
		end
	end
end
ns.On("Progress", HUD.Progress)

-- The pump's close: the banner ------------------------------------------------------------------------------------

-- After a pump with a move done in it (David, 2026-10-01: "a summary screen ... of all that was gained"), a banner at
-- the top centre in the boss-kill banner's art (BossBannerToast.xml, Forever 70124): the pump's name and moves, the XP
-- and the stats it paid (the fold's sh.pump, the totals the phone's line shows), the XP bar filling from the XP before
-- the pump to the XP after it (through a level: Kit.XPBar), then its rows, one after another (Style.motion.row
-- apart; slice 72): the day's moments first (Sheet.MomentRows: Welcome back, the rest tokens that kept the streak, the
-- streak's day with its count turning, Clean Slate, each with its own sound), then a row for each win its toasts
-- would have raised (held while the dim showed), the strongest last, with its sound as it comes, and the rest on the
-- banner's tooltip. A pump that finished with no row plays the one short finish sound. With the game's reduced
-- motion every row stands at once under one sound. It never sits
-- under the dim and takes no key, so walking on needs no click, and it goes by itself: hovering holds it for a read
-- (never for good: a cursor left on it would keep it), a right-click closes it (on a row with a share line, after
-- putting the line in chat), a left-click opens the Log; its tooltip says so. A QP level's fanfare follows it.
local BANNER_W, BANNER_TOP, ROW_W, ROW_H, ROW_STEP = 460, 10, 300, 44, 50

-- A held toast's rank on the banner: a Legendary unlock leads it, then a title (and an Epic unlock), a star or an
-- unlock, a record (a first one too, which sounds as one), then the rest.
local function rank(h)
	if h[5] == "legendary" then return 0 end
	if h[5] == "title" then return RANK_TITLE end
	if h[6] then return h[6] end
	if h[5] == "star" or h[5] == "record" then return RANK_UNLOCK end
	return 4
end

local bannerOut

-- Hovering holds the banner a full read (banner_hold_s) from the moment the cursor comes; leaving it gives it the
-- plain hold again.
function Sheet.BannerHold(on)
	local f = F.banner
	if not (f and f:IsShown()) then return end
	T.cancel("banner")
	T.cancel("bannerOut")
	f.fadeOut:Stop()
	f:SetAlpha(1)
	T.after("banner", on and C.banner_hold_s or C.banner_plain_s, bannerOut)
end

-- The banner's tooltip, owned by the banner or one of its rows.
local function bannerTipShown()
	if not GameTooltip:IsShown() then return false end
	local owner = GameTooltip:GetOwner()
	return owner ~= nil and (owner == F.banner or owner:GetParent() == F.banner)
end

function Sheet.BannerHide()
	for _, name in ipairs({ "banner", "bannerOut", "bannerRow", "bannerRoll" }) do T.cancel(name) end
	local f = F.banner
	if f then
		if bannerTipShown() then GameTooltip:Hide() end
		f.fadeOut:Stop()
		for _, row in ipairs(f.rows) do row.enter:Stop() end
		ns.Kit.CarryStop("bannerXP")
		f:Hide()
	end
	Sheet.AfterBanner()
end

function bannerOut()
	local f = F.banner
	if not (f and f:IsShown()) then return end
	if bannerTipShown() then GameTooltip:Hide() end
	f.fadeOut:Play()
	T.after("bannerOut", C.banner_fade_s, Sheet.BannerHide)
end

-- The sheet on its Log tab, where every pump's lines stay.
function Sheet.ShowLog()
	Sheet.tab = "log"
	if F.sheet and F.sheet:IsShown() then return Sheet.Refresh() end
	Sheet.Toggle()
end

local function bannerClick(button, share)
	if button == "RightButton" then
		if share then ChatFrameUtil.OpenChat(share) end
		Sheet.BannerHide()
	elseif button == "LeftButton" then
		Sheet.BannerHide()
		Sheet.ShowLog()
	end
end

-- Hovering the banner or a row holds it and shows what the mouse does there; on the plate, also the wins it had no
-- row for.
local function bannerEnter(owner, row)
	Sheet.BannerHold(true)
	local f = F.banner
	GameTooltip:SetOwner(owner, "ANCHOR_BOTTOM")
	if not row then
		for i = C.banner_rows + 1, #(f.wins or {}) do
			local h = f.wins[i].h
			GameTooltip:AddLine(toastLine(h[2], h[3]), 1, 1, 1, true)
		end
	end
	GameTooltip_AddInstructionLine(GameTooltip, L["banner.tip.log"])
	rightClickLine(GameTooltip, row and owner.share)
	GameTooltip:Show()
end

local function bannerLeave()
	Sheet.BannerHold(false)
	GameTooltip:Hide()
end

local function bannerFrame()
	if F.banner then return F.banner end
	local f = CreateFrame("Frame", nil, UIParent)
	f:SetSize(BANNER_W, 180)
	f:SetPoint("TOP", 0, -120) -- the boss banner's place
	f:SetFrameStrata("HIGH")
	f:EnableMouse(true)
	f:Hide()
	f.top = f:CreateTexture(nil, "BACKGROUND")
	f.top:SetPoint("TOP", 0, -BANNER_TOP)
	f.bottom = f:CreateTexture(nil, "BACKGROUND")
	f.bottom:SetPoint("BOTTOM")
	f.middle = f:CreateTexture(nil, "BACKGROUND", nil, -1)
	if Style.SetAtlas(f.top, "bannerTop", true) and Style.SetAtlas(f.bottom, "bannerBottom", true)
		and Style.SetAtlas(f.middle, "bannerMid") then
		f.middle:SetPoint("TOPLEFT", f.top, "TOPLEFT", 0, -34) -- BossBannerToast.xml's BannerMiddle
		f.middle:SetPoint("BOTTOMRIGHT", f.bottom, "BOTTOMRIGHT", 0, 25)
	else -- a client without the art: a dark plate
		f.top:Hide()
		f.bottom:Hide()
		f.middle:SetAllPoints()
		f.middle:SetColorTexture(0, 0, 0, 0.7)
	end
	f.title = text(f, "banner", "CENTER")
	f.title:SetPoint("TOP", 0, -(BANNER_TOP + 47))
	f.sub = text(f, "large", "CENTER", BANNER_W - 80)
	f.sub:SetPoint("TOP", f.title, "BOTTOM", 0, -4)
	f.gain = text(f, "levelLine", "CENTER", BANNER_W - 80)
	f.gain:SetPoint("TOP", f.sub, "BOTTOM", 0, -2)
	-- the XP bar under what the pump paid: the level it fills (past the cap the Paragon rank held) and the XP into it
	f.xp = ns.Kit.XPBar(f, ROW_W, 13, "bannerXP")
	f.xp.rested:SetPoint("TOP", f.gain, "BOTTOM", 0, -8)
	f.xp.words = function(level, into, need, paragon)
		return (paragon > 0 and L["sheet.paragon"]:format(paragon) or L["sheet.level"]:format(level)) .. " · "
			.. L["sheet.xp"]:format(number(into), number(need))
	end
	f.rows = {}
	f.more = text(f, "bodyWhite", "CENTER") -- "+2 more": the wins with no row, listed on the tooltip
	f.fadeIn = f:CreateAnimationGroup()
	local a = f.fadeIn:CreateAnimation("Alpha")
	a:SetFromAlpha(0)
	a:SetToAlpha(1)
	a:SetDuration(0.25)
	f.fadeOut = f:CreateAnimationGroup()
	a = f.fadeOut:CreateAnimation("Alpha")
	a:SetFromAlpha(1)
	a:SetToAlpha(0)
	a:SetDuration(C.banner_fade_s)
	f:SetScript("OnEnter", function(self) bannerEnter(self) end)
	f:SetScript("OnLeave", bannerLeave)
	f:SetScript("OnMouseUp", function(_, button) bannerClick(button) end)
	F.banner = f
	return f
end

-- A row: the toast's icon (a win of a move: its key pose) in a picture's screen, its edge in the step's colour
-- (Kit.PaintScreen: none for Starter or a win of no step), the headline and subline on the loot banner's item plate;
-- it comes in from the right as the pour's rows do (row.enter, Kit.Entrance).
local function bannerRow(f, i)
	local r = f.rows[i]
	if r then return r end
	r = CreateFrame("Frame", nil, f)
	r:SetSize(ROW_W, ROW_H)
	r.bg = r:CreateTexture(nil, "BACKGROUND")
	r.bg:SetPoint("CENTER")
	if not Style.SetAtlas(r.bg, "bannerRow", true) then
		r.bg:SetAllPoints()
		r.bg:SetColorTexture(0, 0, 0, 0.4)
	end
	r.screen = ns.Kit.Screen(r, 39, 39)
	r.screen:SetPoint("LEFT", 9, 0)
	r.icon = r.screen:CreateTexture(nil, "ARTWORK")
	r.icon:SetSize(37, 37)
	r.icon:SetPoint("CENTER")
	r.headline = text(r, "label")
	r.headline:SetPoint("TOPLEFT", 56, -7)
	r.headline:SetPoint("RIGHT", -10, 0)
	r.headline:SetWordWrap(false)
	r.subline = text(r, "bodyWhite")
	r.subline:SetPoint("TOPLEFT", r.headline, "BOTTOMLEFT", 0, -1)
	r.subline:SetPoint("RIGHT", -10, 0)
	r.subline:SetWordWrap(false)
	r:EnableMouse(true)
	r:SetScript("OnEnter", function(self) bannerEnter(self, true) end)
	r:SetScript("OnLeave", bannerLeave)
	r:SetScript("OnMouseUp", function(self, button) bannerClick(button, self.share) end)
	r.enter = ns.Kit.Entrance(r, 0, Style.motion.rowIn, Style.motion.rowFrom)
	f.rows[i] = r
	return r
end

-- The day's moments as the banner's rows, from a held toast's `moment` (Sheet.Earned), in the order they come (slice
-- 72): Welcome back with the day's XP under it and the star's sound; the rest tokens that kept the streak, one row;
-- the streak's day, its count turning as the row stands (`from`, Sheet.BannerRoll) from the count the player last had:
-- the day before's, or with rest tokens the day before the days they covered (the fold adds those days and today's
-- at once, and the moment holds how many: `rested`), the day against the streak's next milestone under it ("Day 3 of
-- 7"; past the last one the XP alone) and its own sound; Clean Slate with the star's. Each row { headline, subline,
-- sound, from }. They name nothing lost: a streak that started again shows its new count alone.
function Sheet.MomentRows(m)
	local rows = {}
	if m.welcome then rows[#rows + 1] = { headline = L["toast.welcome"], subline = m.welcome, sound = "star" } end
	if m.rested then rows[#rows + 1] = { headline = ns.Plural("toast.rested", m.rested):format(m.rested) } end
	if m.days then
		local goal
		for _, n in ipairs(C.streak_milestones) do
			if not goal and n > m.days then goal = n end
		end
		local of = goal and L["toast.streak.goal"]:format(m.days, goal)
		local was = m.days - 1 - (m.rested or 0)
		rows[#rows + 1] = { headline = L["toast.streak"]:format(m.days),
			from = was > 0 and L["toast.streak"]:format(was) or nil,
			subline = of and m.xp and of .. " · " .. m.xp or of or m.xp, sound = "streak" }
	end
	if m.clean then rows[#rows + 1] = { headline = L["hud.cleanSlate"], sound = "star" } end
	return rows
end

-- Row `i` of the banner comes (its entrance starts now, `spec.at` s after the banner: Sheet.Banner): its sound, and
-- the next row's turn at its own time. The streak's row stands first with the count the player last had, and turns
-- to today's with a pop and its sound once it is in, a beat the row after it waits out.
function Sheet.BannerArrive(i)
	local f = F.banner
	local spec = f and f:IsShown() and f.order[i]
	if not spec then return end
	if spec.from then
		T.after("bannerRoll", Style.motion.rowIn, function() Sheet.BannerRoll(i) end)
	elseif spec.sound then
		Style.Play(spec.sound)
	end
	local after = f.order[i + 1]
	if after then T.after("bannerRow", after.at - spec.at, function() Sheet.BannerArrive(i + 1) end) end
end

function Sheet.BannerRoll(i)
	local f = F.banner
	local spec = f and f:IsShown() and f.order[i]
	if not spec then return end
	f.rows[i].headline:SetText(spec.headline)
	ns.Kit.Pop(f.rows[i].headline)
	Style.Play(spec.sound)
end

-- The moves the pump did: their names when one or two, else how many.
local function movesLine(p)
	local names = {}
	for i, slot in ipairs(p.moves) do
		if p.results[i] then names[#names + 1] = moveName(slot.move) end
	end
	if #names <= 2 then return table.concat(names, " · ") end
	return L["banner.moves"]:format(#names)
end

-- What the pump paid: its XP, then each stat it raised in the stats' order (the day's moves are the HUD's QP line's,
-- which shows them at the same moment: play SPEC 16).
local function gainLine()
	local pump = ns.state and ns.state.sheet.pump
	if not pump then return "" end
	local parts = { L["banner.xp"]:format(number(pump.xp or 0)) }
	for _, key in ipairs(ns.STATS) do
		local n = pump.stats and pump.stats[key]
		if n and n > 0 then parts[#parts + 1] = L["banner.stat"]:format(n, L["stat." .. key]) end
	end
	return table.concat(parts, " · ")
end

-- The banner for a pump that ended (`how` "done", or "rest": Bank the rest after a move), with the toasts the pump
-- held; true when it shows. "After a pump" off, or on "wins" with nothing won, leaves the toasts as they were.
function Sheet.Banner(p, how, list)
	local mode = ns.profile and ns.profile.afterPump or "summary" -- the setting "After a pump", nil until changed
	if mode == "none" or not (how == "done" or how == "rest") or p.kind == "practice" or not next(p.results) then
		return false
	end
	local wins, fanfare, legends, order = {}, nil, {}, {}
	for i, h in ipairs(list) do
		if h.fanfare then
			fanfare = h.fanfare
		elseif h.legend then
			legends[#legends + 1] = h.legend
		elseif h[9] then
			for _, row in ipairs(Sheet.MomentRows(h[9])) do order[#order + 1] = row end
		else
			wins[#wins + 1] = { h = h, i = i }
		end
	end
	if mode == "wins" and #wins == 0 and #order == 0 and not fanfare and #legends == 0 then return false end
	table.sort(wins, function(a, b)
		if rank(a.h) ~= rank(b.h) then return rank(a.h) < rank(b.h) end
		return a.i < b.i
	end)
	-- the wins' rows after the moments', the strongest last: its sound, the one the wins play, comes with it (its
	-- toast's; a record's row, a first record's too, the small chime: its own cue played as its count was saved)
	local shown = math.min(#wins, C.banner_rows)
	for i = shown, 1, -1 do
		local h = wins[i].h
		order[#order + 1] = { icon = h[8] and moveIcon(h[8]) or h[1], headline = h[2], subline = h[3], share = h[4],
			quality = h[7], sound = i == 1 and (rank(h) ~= RANK_RECORD and h[5] or "win") or nil }
	end
	local f = bannerFrame()
	f.wins, f.order = wins, order
	f.title:SetText(L["banner.title"])
	local moves = movesLine(p)
	f.sub:SetText(moves ~= "" and L["banner.sub"]:format(ns.Panel.Title(p), moves) or ns.Panel.Title(p))
	f.gain:SetText(gainLine())
	local moving, m = ns.Tree.Moving(), Style.motion
	local top = BANNER_TOP + 47 + f.title:GetStringHeight() + 4 + f.sub:GetStringHeight() + 2
		+ f.gain:GetStringHeight() + 8 + f.xp.rested:GetHeight() + 12
	-- the banner shows, fading in, before any row's entrance plays: what the client does with a group played under a
	-- hidden frame is undocumented, and a row whose entrance never ran would stay unseen
	for _, name in ipairs({ "bannerOut", "bannerRow", "bannerRoll" }) do T.cancel(name) end
	f.fadeOut:Stop()
	f:SetAlpha(1)
	f:Show()
	f.fadeIn:Play()
	for i, row in ipairs(f.rows) do
		row.enter:Stop()
		row:SetShown(i <= #order)
	end
	local at = 0
	for i, spec in ipairs(order) do
		at = at + m.row
		spec.at = at
		local r = bannerRow(f, i)
		r:ClearAllPoints()
		r:SetPoint("TOP", 0, -(top + (i - 1) * ROW_STEP))
		setToastIcon(r.icon, spec.icon)
		ns.Kit.PaintScreen(r.screen, spec.quality)
		r.headline:SetText(moving and spec.from or spec.headline)
		r.subline:SetText(spec.subline or "")
		r.share = spec.share
		r:Show()
		r:SetAlpha(moving and 0 or 1) -- an entrance shows nothing before its group runs
		if moving then
			r.enter.hold:SetDuration(at)
			r.enter:Play()
			if spec.from then at = at + m.rowIn end -- its count turns before the next row comes
		end
	end
	local more = #wins - shown -- listed on the banner's tooltip (bannerEnter)
	f.more:SetShown(more > 0)
	f.more:SetText(more > 0 and L["banner.more"]:format(more) or "")
	f.more:ClearAllPoints()
	f.more:SetPoint("TOP", 0, -(top + #order * ROW_STEP))
	f:SetHeight(top + #order * ROW_STEP + (more > 0 and f.more:GetStringHeight() + 4 or 0) + 40)
	-- what waits already (a banner that ended under this pump's dim) goes first
	local queued = Sheet.bannerLegends or {}
	for _, e in ipairs(legends) do queued[#queued + 1] = e end
	Sheet.bannerFanfare, Sheet.bannerLegends = fanfare or Sheet.bannerFanfare, queued
	-- the XP the pump paid, carried from where the level stood before it (the fold's sh.pump keeps the pump's XP)
	local sheet = ns.state.sheet
	f.xp:SetXP(sheet.xp - (sheet.pump and sheet.pump.xp or 0))
	f.xp:FillXP(f.xp.xp, sheet.xp)
	-- each row sounds as it comes: a moment its own, the wins the strongest one's alone with its row, the last; with
	-- the game's reduced motion the rows stand at once under that one sound.
	-- A pump that finished with no row plays the short finish sound (never one banked part way), and a level's fanfare
	-- plays its own.
	if #order == 0 then
		if how == "done" then Style.Play("finish") end
	elseif moving then
		T.after("bannerRow", order[1].at, function() Sheet.BannerArrive(1) end)
	else
		local sound
		for _, spec in ipairs(order) do sound = spec.sound or sound end
		if sound then Style.Play(sound) end
	end
	-- long enough to read what it lists (David, 2026-10-03: it "disappears too fast"): a base, and more for each row
	T.after("banner", #order > 0 and C.banner_hold_s + #order * C.banner_row_s or C.banner_plain_s, bannerOut)
	return true
end

-- Panel.lua's State listener runs first (TOC order): the dim is hidden or shown already. A pump that ends puts its held
-- toasts on its banner; a pump that comes to the screen ends the banner of the one before.
-- The held toasts and fanfare go out once nothing hides them: the dim gone, no fight on and no started pump waiting
-- to come back (a fight hides the dim with them still held, and they wait for the pump's end, never sounding as it
-- starts, so its banner carries every win). After "felt unwell" (the day's lock) nothing celebrates: no banner, toast,
-- sound or fanfare; the wins stay on the sheet, and in chat on "All". The focus ask follows what they show.
-- What follows a banner (each Legendary frame, the level's fanfare) and a Legendary frame on screen go back to wait in
-- `held`, the frame on screen first, when something takes the screen: a fight, or the next pump's dim or card. They
-- come out at the next release, on that pump's banner or after it.
local function holdBack()
	if F.legend and F.legend:IsShown() then
		held[#held + 1] = { legend = F.legend.e }
		T.cancel("legend")
		F.legend:Hide()
	end
	for _, e in ipairs(Sheet.bannerLegends or {}) do held[#held + 1] = { legend = e } end
	if Sheet.bannerFanfare then held[#held + 1] = { fanfare = Sheet.bannerFanfare } end
	Sheet.bannerFanfare, Sheet.bannerLegends = nil, nil
end

local function release(p, how)
	if dimmed() or inCombat() or underWay() then return end
	local list = held
	held = {}
	if Lc.Locked() then
		for _, h in ipairs(list) do
			if not (h.fanfare or h.legend) then ns.Chat("toast", toastLine(h[2], h[3])) end
		end
		return
	end
	if p and p.state == "done" and Sheet.Banner(p, how, list) then return end
	-- no banner of its own: the toasts go out now, a Legendary frame or a fanfare joins what follows a banner, and that
	-- starts only while no banner, Legendary frame or fanfare is up (a turn-in during another pump's banner neither
	-- drops its queue nor sounds over it; each of them starts the queue again as it ends)
	Sheet.bannerLegends = Sheet.bannerLegends or {}
	for _, h in ipairs(list) do
		if h.fanfare then
			Sheet.bannerFanfare = h.fanfare
		elseif h.legend then
			Sheet.bannerLegends[#Sheet.bannerLegends + 1] = h.legend
		else
			Sheet.Toast(unpack(h, 1, 9))
		end
	end
	local up = (F.banner and F.banner:IsShown()) or (F.legend and F.legend:IsShown())
		or (F.fanfare and F.fanfare:IsShown())
	if not up then Sheet.AfterBanner() end
end

ns.On("State", function(p, how)
	-- the Log's "In progress" goes as the pump ends, which may write no row of its own; the Body tab's XP bar fills
	-- from what it showed before the pump
	if p and p.state == "done" and (Sheet.tab == "log" or Sheet.tab == "body") then Sheet.Refresh() end
	if Lc.pump and (dimmed() or carded()) then
		holdBack()
		Sheet.BannerHide()
	end
	if dimmed() then return end
	-- a pump finished, or banked the rest after a move: its close may ask for the focus (Sheet.AskFocus)
	if p and p.state == "done" and (how == "done" or how == "rest") then Sheet.focusDue = true end
	release(p, how)
end)

-- A Legendary unlock (slice 22): the move in orange light -----------------------------------------------------------

-- After the toast (on the pump's banner, its first row), an own centre frame: the figure performing the move with its
-- cloth's light in the Legendary colour, the move's name, its ladder and rung, and the voice naming it. It goes by
-- itself or at a click, and never shows under the dim, in a fight or while a started pump waits (held, as the level's
-- fanfare is). Several in one pump follow each other.
local LEGEND_W = 420

local function legendFrame()
	if F.legend then return F.legend end
	local f = CreateFrame("Button", nil, UIParent)
	f:SetSize(LEGEND_W, C.figure_ready_px + 150)
	f:SetPoint("CENTER", 0, 90)
	f:SetFrameStrata("HIGH")
	f:Hide()
	f.glow = f:CreateTexture(nil, "BACKGROUND")
	f.glow:SetPoint("CENTER", 0, 10)
	f.glow:SetSize(C.figure_ready_px + 180, C.figure_ready_px + 180)
	f.glow:SetBlendMode("ADD")
	if not Style.SetAtlas(f.glow, "legendGlow") then Style.SetAtlas(f.glow, "levelGlow") end
	f.title = text(f, "banner", "CENTER")
	f.title:SetPoint("TOP", 0, -4)
	f.figure = ns.Figure.Create(f, C.figure_ready_px)
	f.figure.frame:SetPoint("TOP", f.title, "BOTTOM", 0, -8)
	f.figure.frame:EnableMouse(false) -- a click anywhere closes the frame
	f.name = text(f, "move", "CENTER", LEGEND_W)
	f.name:SetPoint("TOP", f.figure.frame, "BOTTOM", 0, -8)
	f.sub = text(f, "large", "CENTER", LEGEND_W)
	f.sub:SetPoint("TOP", f.name, "BOTTOM", 0, -4)
	f.fadeIn = f:CreateAnimationGroup()
	local a = f.fadeIn:CreateAnimation("Alpha")
	a:SetFromAlpha(0)
	a:SetToAlpha(1)
	a:SetDuration(0.4)
	f:SetScript("OnClick", function() Sheet.LegendHide() end)
	F.legend = f
	return f
end

function Sheet.LegendHide()
	T.cancel("legend")
	if F.legend then F.legend:Hide() end
	Sheet.AfterBanner()
end

function Sheet.Legend(e)
	-- it waits, as the banner does, for a fight, the next pump's dim or its "Stop to pump" card, and a banner up
	if inCombat() or dimmed() or underWay() or carded() or (F.banner and F.banner:IsShown()) then
		held[#held + 1] = { legend = e }
		return
	end
	local f = legendFrame()
	f.e = e
	local q = e.quality or Q.legendary
	local color = qualityColor(q)
	f.title:SetText(colored(color, L["quality." .. q]))
	f.name:SetText(colored(color, e.move and moveName(e.move) or L["ladder." .. e.ladder]))
	f.sub:SetText(L["toast.legendary.frame"]:format(L["ladder." .. e.ladder], ns.Step(e.ladder, e.rung)))
	f:Show()
	f.fadeIn:Play()
	if e.move then
		f.figure:Play(e.move, "loop", false, { color:GetRGB() })
		ns.Speech.Say("legend." .. ns.Library[e.move].key)
	else
		f.figure:HideAll()
	end
	T.after("legend", C.legend_s, Sheet.LegendHide)
end

-- What follows a pump's banner (or, with no banner, its toasts), one after the other: each Legendary unlock's frame,
-- then a QP level's or a Paragon rank's fanfare, then the focus ask.
function Sheet.AfterBanner()
	local legends = Sheet.bannerLegends
	if legends and #legends > 0 then return Sheet.Legend(table.remove(legends, 1)) end
	local fanfare = Sheet.bannerFanfare
	Sheet.bannerFanfare = nil
	if fanfare then Sheet.Fanfare(fanfare) else Sheet.AskFocus() end
end

-- The QP level-up: an own centre frame (Blizzard's LevelUpDisplay does not exist on Forever) ----------------------

-- `e`: the fold's level event ({ kind = "level", level }) or, past the cap, its Paragon rank's ({ kind = "paragon",
-- rank }; slice 23), which plays the same fanfare as "Paragon 3". Held and queued alike; a later one waiting takes the
-- place of an earlier one, so a gain that crosses two shows the higher. It comes in parts (slice 72; f.anim): the gold
-- bars grow out from the middle, the glow flashes up behind them, the number pops in as the countdown's digits do
-- (Style.motion's bars, number and numberFrom), then the words under it; whole at once with the game's reduced motion.
function Sheet.Fanfare(e)
	if inCombat() or dimmed() or underWay() or carded() then
		held[#held + 1] = { fanfare = e }
		return
	end
	local f = F.fanfare
	if not f then
		f = CreateFrame("Frame", nil, UIParent)
		f:SetSize(520, 120)
		f:SetPoint("TOP", 0, -170)
		f:SetFrameStrata("HIGH")
		f.bar = f:CreateTexture(nil, "BACKGROUND")
		f.bar:SetPoint("CENTER")
		f.bar:SetSize(520, 90)
		Style.SetAtlas(f.bar, "recordBar")
		f.glow = f:CreateTexture(nil, "ARTWORK")
		f.glow:SetPoint("CENTER")
		f.glow:SetSize(520, 120)
		Style.SetAtlas(f.glow, "levelGlow")
		f.text = text(f, "fanfare", "CENTER")
		f.text:SetPoint("CENTER", 0, 6)
		f.sub = text(f, "large", "CENTER")
		f.sub:SetPoint("TOP", f.text, "BOTTOM", 0, -2)
		local Kit, m = ns.Kit, Style.motion
		f.text:SetScaleAnimationMode(Enum.FontStringScaleAnimationMode.Vertex) -- a smooth grow, as Blizzard's warnings
		f.anim = f:CreateAnimationGroup()
		local grow = f.anim:CreateAnimation("Scale")
		grow:SetTarget(f.bar)
		grow:SetScaleFrom(0.01, 1)
		grow:SetScaleTo(1, 1)
		grow:SetDuration(m.bars)
		grow:SetSmoothing("OUT")
		grow:SetOrder(1)
		for _, part in ipairs({ f.glow, f.text, f.sub }) do Kit.Alpha(f.anim, part, 0, 0, m.bars, 1) end
		Kit.Alpha(f.anim, f.glow, 0, 1, m.number, 2)
		local pop = f.anim:CreateAnimation("Scale")
		pop:SetTarget(f.text)
		pop:SetScaleFrom(m.numberFrom, m.numberFrom)
		pop:SetScaleTo(1, 1)
		pop:SetDuration(m.number)
		pop:SetSmoothing("OUT")
		pop:SetOrder(2)
		Kit.Alpha(f.anim, f.text, 0, 1, m.number, 2):SetSmoothing("OUT")
		Kit.Alpha(f.anim, f.sub, 0, 0, m.number, 2)
		Kit.Alpha(f.anim, f.sub, 0, 1, m.number, 3)
		f.anim:SetToFinalAlpha(true)
		F.fanfare = f
	end
	local paragon = e.kind == "paragon"
	f.text:SetText(paragon and L["fanfare.paragon"]:format(e.rank) or L["fanfare.level"]:format(e.level))
	f.sub:SetText(L[paragon and "fanfare.paragonSub" or "fanfare.levelSub"])
	f:Show()
	f.anim:Stop()
	if ns.Tree.Moving() then f.anim:Play() end
	Style.Play("levelUp")
	T.after("fanfare", 3.5, function()
		f:Hide()
		Sheet.AfterBanner() -- a Legendary frame that waited behind it, else the focus ask
	end)
end

-- The focus (David, 2026-10-03; Lc.SetFocus): what the day plan leans toward. At QP focus_level, Blizzard's talent
-- level, the window asks once with no tile chosen, after a pump's close has shown (its banner and the level's fanfare
-- gone) and out of combat; a choice, or "Keep Balanced", writes the row and the ask never comes back, and closing it
-- asks again next session. Opened from the sheet's level line or /qp focus, it shows the focus in force chosen
-- (Balanced before any focus row, as the Settings page's Focus shows it) and changes it for the next pump. Since
-- 2026-10-16 the kit's medium window (sheet SPEC 13): the head (at the ask its title names the level it came at; no
-- line under it), four choice tiles each with its key pose (the figure pack's still) and its name, Balanced's with
-- "(recommended)" (what each does is its tooltip's: no second label, David, 2026-10-22), and the bar's one red only at
-- the ask.
local FOCUS_POSE = { balanced = "sun_salutation", strength = "pushup", endurance = "jumping_jacks",
	flexibility = "downward_dog" }

local function buildFocus()
	local Kit = ns.Kit
	local f = Kit.Window("QuestPumpFocus", "M", L["ui.focus.window"], { escape = "special", y = 80 })
	f.view = Kit.Head(f.body, L["ui.focus.title"], nil, f.tier)
	f.view.intro:Hide()
	local w, gap = Kit.Grid(4, f.tier.TEXT)
	f.tiles, f.gap = {}, gap
	for i, key in ipairs(ns.Enum.focusKeys) do
		-- a click chooses it and closes; the focus in force clicked again writes nothing and plays nothing, and its
		-- tooltip says it is the focus now in place of the click's hint
		local c = Kit.Choice(f.view, w, 170, function(c)
			if c.key ~= f.current then
				Style.Play("checkOn")
				ns.Lifecycle.SetFocus(c.key)
			end
			f:Hide()
		end, Kit.Tip(L["focus." .. key], L["focus." .. key .. ".line"], {
			facts = function() return key == f.current and { L["ui.focus.now"] } or nil end,
			hint = function() return key ~= f.current and L["ui.focus.choose"] or nil end }))
		c.screen = Kit.Screen(c, w - 20, 88)
		c.screen:SetPoint("TOP", 0, -10)
		c.view = Kit.Figure(c.screen, FOCUS_POSE[key], 88)
		c.name = Kit.Text(c, "cardTitle", "LEFT", nil, w - 20)
		c.name:SetPoint("TOPLEFT", c.screen, "BOTTOMLEFT", 0, -11)
		c.name:SetText(key == "balanced" and L["ui.focus.recommended"]:format(L["focus.balanced"])
			or L["focus." .. key])
		c.key = key
		f.tiles[i] = c
	end
	f.foot = Kit.Footer(f)
	f.keep = f.foot:Primary(L["ui.focus.keep"], function()
		ns.Lifecycle.SetFocus("balanced")
		f:Hide()
	end, "tab")
	F.focus = f
	return f
end

-- `asking`: the QP focus_level ask, with "Keep Balanced" and no tile chosen; else the focus in force, chosen, to
-- change. Each tile as tall as its words at the player's text size, the four alike.
function Sheet.ShowFocus(asking)
	local Kit = ns.Kit
	local f = F.focus or buildFocus()
	f.asking = asking -- for a lay-out again (Kit.OnRelayout)
	f.current = not asking and ns.Focus(ns.state) or nil
	f.view.title:SetText(asking and L["ui.focus.titleAsk"]:format(C.focus_level) or L["ui.focus.title"])
	local y, h = Kit.Top(f.view), 170
	for _, c in ipairs(f.tiles) do
		Kit.Pose(c.view)
		-- the one in force gold; the others at rest, their pictures in colour (a choice among four, never "off")
		Kit.Choose(c, true, c.key ~= f.current)
		h = math.max(h, 10 + 88 + 11 + c.name:GetStringHeight() + 12)
	end
	for i, c in ipairs(f.tiles) do
		c:SetHeight(h)
		Kit.At(c, f.view, y, (i - 1) * (c:GetWidth() + f.gap))
	end
	f.foot:SetShown(asking)
	Kit.Fit(f, y + h, asking and f.foot or nil)
	Kit.Open(f)
end

-- Whether one of QuestPump's own windows that come by themselves is up: the pump's closing banner, a Legendary frame,
-- the fanfare, the focus window, the day-2 check-in, or one of its questions in a popup (the mixed party's, a pact's
-- or a duel's offer, whose Escape would answer them before a window opened over them closes). What comes after them
-- waits (the focus ask, the day card: Streamer.lua), so none opens over another.
local OWN_POPUPS = { "QUESTPUMP_PARTY", "QUESTPUMP_PACT", "QUESTPUMP_DUEL" }

function Sheet.ScreenTaken()
	local checkin = ns.Panel and ns.Panel.frames.checkin
	for _, f in ipairs({ F.banner, F.fanfare, F.legend, F.focus, checkin }) do
		if f and f:IsShown() then return true end
	end
	for _, which in ipairs(OWN_POPUPS) do
		if ns.Dialog.Visible(which) then return true end
	end
	return false
end

-- The ask at QP focus_level, after a pump's close (Sheet.focusDue): once a session until a focus row exists, never
-- while a pump is on its way, in a fight, beside the closing banner or the level's fanfare (each asks again as it
-- goes) or over the day-two check-in in the same place.
function Sheet.AskFocus()
	local state = ns.state
	if not Sheet.focusDue or Sheet.focusAsked or not state or state.focus or inCombat() or Lc.pump then return end
	if Sheet.ScreenTaken() then return end
	if ns.Level(state) < C.focus_level then return end
	Sheet.focusAsked = true
	Sheet.ShowFocus(true)
end

-- A bonus's XP, rested XP included.
local function xpLine(e)
	return L["toast.xp"]:format(e.xp or 0)
end

-- The streak's day in words: its day, or a new streak's first (never a word about a streak before it).
function Sheet.StreakWords(days)
	return days > 1 and L["toast.streak"]:format(days) or L["toast.streak.new"]
end

-- What a row earned on the sheet (Stats.lua's live events): a level, a Paragon rank, a title, a vault goal, the rest
-- tokens that covered the days before the day's first move (before its streak), Welcome back, the day's first move and
-- its streak or the streak's milestone (in the level's sound, ranked with a title; since slice 45 its right-click puts
-- its chat line in the chat box, as a title's does), a rest token, Clean Slate, a record, and the day's one "New best
-- day" (Stats.lua's bestDay, 2026-10-15: shown, never awarded, so no XP line; the epic loot toast's sound, ranked with
-- a title, a row of the pump's banner when it came in one), and the day's last ember's "Cardio done today" (slice 55,
-- the small chime). Every profile keeps the game day of the newest milestone, best day and day's embers toasted
-- (milestoneDay, bestDay, embersDay: the beta reads no save back, and the profiles, unlike the meta, come back through
-- the helper's mirror), so no /reload, later login or other character toasts it again (Sheet.MirroredMilestone,
-- Sheet.Toasted). A game day's first move that comes with the rest tokens that kept its
-- streak tells them with its Welcome back or its streak in one toast: "Welcome back!" over the tokens' words and the XP
-- (play SPEC 15: one telling per win; two toasts until then); a Welcome back with no token (Stats.lua's activeDay sends
-- it in place of the streak's event) says the streak's day under it, "Day 1 of a new streak" or, where locked days kept
-- the streak, its day (ns.Streak), and the XP. Since slice 72 these are the day's moments: each toast carries what
-- the pump's banner tells in rows of their own in its place (`moment`, Sheet.MomentRows: Welcome back, the rest
-- tokens, the streak's day, Clean Slate), so the toast itself shows only where no banner does (a pump with "After a
-- pump" off); Welcome back and Clean Slate sound as a star does, the streak's day by its own cue, a first record as
-- a record.
-- The very first title earned wears itself while none is chosen, so the title bar shows it at once (the 2026-10-21
-- review: it stayed unworn until the Deeds tab); a later one waits for the player's pick, as on the phones.
local function wearFirst(id)
	if not ns.profile or ns.profile.title then return end
	local earned = ns.TitlesEarned(ns.state)
	if not earned[id] or next(earned, next(earned)) then return end
	ns.profile.title = id
	ns.Touch()
end

function Sheet.Earned(live)
	local day = {}
	for _, e in ipairs(live) do
		if e.kind == "welcome" or e.kind == "streak" or e.kind == "rested" then day[e.kind] = e end
	end
	local merged = day.rested and (day.welcome or day.streak) -- Stats.lua's activeDay sends one of the two
	for _, e in ipairs(live) do
		if merged and day[e.kind] == e then
			if e == merged then
				local words = ns.Plural("toast.rested", day.rested.count):format(day.rested.count)
				local days = e.days or ns.Streak(ns.state, ns.now())
				Sheet.Toast(nil, L["toast.welcome"], words .. " · " .. xpLine(e), nil, "star", nil, nil, nil,
					e.kind == "welcome" and { welcome = Sheet.StreakWords(days) .. " · " .. xpLine(e),
						rested = day.rested.count } or { rested = day.rested.count, days = days, xp = xpLine(e) })
			end
		elseif e.kind == "level" then
			Sheet.Fanfare(e)
			Sheet.Toast(Style.xpIcon, L["toast.level"]:format(e.level), L["toast.levelSub"], nil)
		elseif e.kind == "paragon" then
			-- past the cap (slice 23): the fanfare as "Paragon 3", and the toast with a share line, as a title's
			Sheet.Fanfare(e)
			Sheet.Toast(Style.xpIcon, L["toast.paragon"]:format(e.rank), L["toast.paragonSub"],
				L["share.paragon"]:format(e.rank))
		elseif e.kind == "title" then
			local name = L["title." .. e.id]
			wearFirst(e.id)
			Sheet.Toast(nil, L["toast.title"]:format(name), L["deed." .. e.id], L["share.title"]:format(name), "title")
		elseif e.kind == "vault" then
			Sheet.Toast(nil, L["toast.vault"]:format(L["vault." .. e.goal]:format(e.need or 0)), xpLine(e), nil, "star")
		elseif e.kind == "vaultAll" then
			Sheet.Toast(nil, L["toast.vaultAll"], nil, nil, "title")
		elseif e.kind == "embers" and Sheet.Toasted("embersDay") < e.day then
			-- the game day's last ember (slice 55, Stats.lua lightEmber), once a game day as the best day's: shown, with the
			-- week's embers under it (its vault tier toasts apart)
			Sheet.MarkToasted("embersDay", e.day)
			Sheet.Toast(nil, L["toast.embers"], L["toast.embers.sub"]:format(e.week), nil, "win")
		elseif e.kind == "rested" then
			Sheet.Toast(nil, ns.Plural("toast.rested", e.count):format(e.count), nil, nil, nil, nil, nil, nil,
				{ rested = e.count })
		elseif e.kind == "welcome" or e.kind == "streak" then
			local days = e.days or ns.Streak(ns.state, ns.now())
			local words = Sheet.StreakWords(days)
			if e.kind == "welcome" then
				Sheet.Toast(nil, L["toast.welcome"], words .. " · " .. xpLine(e), nil, "star", nil, nil, nil,
					{ welcome = words .. " · " .. xpLine(e) })
			else
				Sheet.Toast(nil, words, xpLine(e), nil, "streak", nil, nil, nil, { days = days, xp = xpLine(e) })
			end
		elseif e.kind == "milestone" then
			-- every profile keeps it, as the streak is the account's: another character's login toasts it no more
			if e.day then Sheet.MarkToasted("milestoneDay", e.day) end
			-- a whole number of years from two ("A 2-year streak!"), else its days ("A 1,000-day streak!"); its right-click
			-- puts the same count in chat (slice 45: "My 30-day streak [QuestPump]", "My 2-year streak [QuestPump]")
			local words = L["toast.milestone"]:format(number(e.days))
			local share = L["share.milestone"]:format(number(e.days))
			if e.days >= 730 and e.days % 365 == 0 then
				words = L["toast.milestone.years"]:format(e.days / 365)
				share = L["share.milestone.years"]:format(e.days / 365)
			end
			Sheet.Toast(nil, words, xpLine(e), share, "levelUp", RANK_TITLE)
		elseif e.kind == "token" then
			Sheet.Toast(nil, L["toast.token"], nil, nil, "star")
		elseif e.kind == "clean" then
			Sheet.Toast(nil, L["hud.cleanSlate"], nil, nil, "star", nil, nil, nil, { clean = true })
		elseif e.kind == "record" then
			local name, amount = moveName(e.move), Sheet.LoadAmount(e.move, e.load, e.value)
			Sheet.Toast(Style.statIcon[ns.MoveStat(e.move)], L["toast.record"]:format(amount), name, ns.profile.shareCount
				and L["share.record.count"]:format(amount, name) or L["share.record"]:format(name), nil, RANK_RECORD,
				ns.MoveQuality(e.move), e.move)
		elseif e.kind == "bestDay" and Sheet.Toasted("bestDay") < e.day then
			-- once a game day: a /reload whose mirror lacks the row that passed the old best folds today back under it, and
			-- the next row passes it again
			Sheet.MarkToasted("bestDay", e.day)
			local moves = number(e.moves)
			Sheet.Toast(Style.star, L["toast.bestDay"], L["toast.bestDay.sub"]:format(moves),
				L["share.bestDay"]:format(moves), "title")
		end
	end
end
ns.On("Sheet", Sheet.Earned)

-- A streak milestone the login's fold reached on rows the game never folds live (an app pump's or a catch-up's, which
-- come only through the helper's mirror; a game row's milestone was toasted where its row was written), toasted once
-- while its streak still counts: the streak now is at least the milestone only when nothing broke it since, as a new
-- streak that grew past the first milestone would have passed a newer one.
function Sheet.MirroredMilestone()
	local state = ns.state
	local m = state and state.sheet.lastMilestone
	if not (m and m.mirrored) or Sheet.Toasted("milestoneDay") >= m.day then return end
	if ns.Streak(state, ns.now()) < m.days then return end
	Sheet.Earned({ { kind = "milestone", days = m.days, day = m.day, xp = m.xp } })
end

-- The game day of the newest toast of `field` on this account (milestoneDay: a streak milestone; bestDay: a new best
-- day, 2026-10-15; embersDay: the day's last ember, slice 55): the newest any profile keeps, so a profile made since
-- (its defaults hold none) toasts nothing another character already did.
function Sheet.Toasted(field)
	local newest = 0
	for _, p in pairs(ns.db and ns.db.profiles or {}) do
		newest = math.max(newest, type(p) == "table" and tonumber(p[field]) or 0)
	end
	return newest
end

-- Keeps a toast's game day in every profile, as the streak and the best day are the account's.
function Sheet.MarkToasted(field, day)
	if Sheet.Toasted(field) >= day then return end
	for _, p in pairs(ns.db and ns.db.profiles or {}) do
		if type(p) == "table" then p[field] = day end
	end
	ns.Touch(ns.db and ns.db.profiles)
end

-- The HUD -------------------------------------------------------------------------------------------------------------

-- Where the HUD stands (docs/plan.md slice 15). With LibEditMode (Libs/, the packager's externals) it is an Edit Mode
-- frame: the player places it in Edit Mode, once per layout, and outside Edit Mode it never moves; the profile's
-- hudLayouts keeps each layout's place under the name the library reports ({ point, x, y } on UIParent). Without the
-- library (a checkout before `make libs`, or a Forever build that breaks it) the header stays the drag handle, with
-- /qp lock, and the profile's hudPos keeps one place for every layout, so the HUD stays placeable. Neither place set,
-- or a layout's place reset in Edit Mode: under the objective tracker. The pixel strip (Bridge.lua) joins neither: the
-- helper's crop is fixed. Both places are made on the first move, never defaults, so an unmoved HUD adds nothing to
-- the profile the strip paints.
local editMode -- the library, once it took the HUD (attachEditMode)

-- The HUD's spot when the client has no objective tracker, and the default Edit Mode's "Reset position" puts back (the
-- library anchors only on UIParent), after which placeHUD hangs the HUD under the tracker again.
local HUD_HOME = { point = "TOPRIGHT", x = -90, y = -320 }

local function savedPlace(profile)
	if not editMode then return profile.hudPos end
	local layout, places = editMode:GetActiveLayoutName(), profile.hudLayouts
	return layout and type(places) == "table" and places[layout] or nil
end

local function placeHUD(f)
	f:ClearAllPoints()
	local pos = ns.profile and savedPlace(ns.profile)
	if type(pos) == "table" and pos.point then
		f:SetPoint(pos.point, UIParent, pos.point, pos.x or 0, pos.y or 0)
		return
	end
	local tracker = ObjectiveTrackerFrame and ObjectiveTrackerFrame.NineSlice
	if tracker then
		f:SetPoint("TOPRIGHT", tracker, "BOTTOMRIGHT", 0, -8) -- the tracker's real bottom; never parented to it
	else
		f:SetPoint(HUD_HOME.point, UIParent, HUD_HOME.point, HUD_HOME.x, HUD_HOME.y)
	end
end
HUD.Place = placeHUD

-- A save bound again (/qp reset all, a restore, an import) holds its own place, or none: the HUD takes it at once, as
-- the next /reload would. At PLAYER_LOGIN the save binds before the HUD exists, and the HUD places itself once built.
ns.On("DB", function() if HUD.frame then placeHUD(HUD.frame) end end)

-- A place the player made, in its top form: TOPLEFT, TOP or TOPRIGHT by the point's horizontal part, the same x, and y
-- from the HUD's top. Edit Mode (LibEditMode's normalizePosition) and the client's drag (StopMovingOrSizing) anchor the
-- HUD by its nearest point, a centre or bottom one below the top quarter of the screen, while HUD.Render grows the
-- frame down from its header: only a top point keeps the header where the player left it as lines come and go. The
-- HUD keeps scale 1 on UIParent, so its top and UIParent's height share units; it is read before placeHUD clears the
-- points, which drops the rect at once (11.2.0).
local function topPlace(f, point, x)
	return { point = "TOP" .. (point:match("LEFT") or point:match("RIGHT") or ""), x = x,
		y = f:GetTop() - UIParent:GetHeight() }
end

-- Edit Mode moved the HUD in the active layout, or put it back with "Reset position", which reports HUD_HOME itself.
-- When the library kept the frame but its setup failed after (attachEditMode: editMode nil, the header drags it), the
-- move is the header's one place, hudPos, the place savedPlace reads then. Either way the HUD then stands where its
-- place says: a move's top form, or under the tracker after a reset.
local function editModeMoved(f, layout, point, x, y)
	local profile = ns.profile
	if not profile or (editMode and not layout) then return end
	local reset = point == HUD_HOME.point and x == HUD_HOME.x and y == HUD_HOME.y
	local place = not reset and topPlace(f, point, x) or nil
	local changed
	if editMode then
		-- a reset of a layout without a place (the library offers it while the HUD hangs under the tracker) writes nothing
		changed = place ~= nil or (type(profile.hudLayouts) == "table" and profile.hudLayouts[layout] ~= nil)
		if changed then
			if type(profile.hudLayouts) ~= "table" then profile.hudLayouts = {} end
			profile.hudLayouts[layout] = place
		end
	else
		changed = place ~= nil or profile.hudPos ~= nil
		profile.hudPos = place
	end
	placeHUD(f)
	if changed then ns.Touch() end
end

-- The presets under the names the library reports them by ("Modern", "Classic", "Gamepad"; LibEditMode.lua), from the
-- client's localized ones, by which a copy of a preset names its source (EditModePresetLayoutsManager.lua: layoutName
-- = LAYOUT_STYLE_CLASSIC, "Klassisch" on deDE).
local function layoutKey(name)
	for _, preset in ipairs({ "Modern", "Classic", "Gamepad" }) do
		if name == _G["LAYOUT_STYLE_" .. preset:upper()] then return preset end
	end
	return name
end

-- A layout renamed, copied or deleted in Edit Mode: the places in every profile follow it by name, since the layouts
-- belong to the account and its characters, never to a profile. The name is the whole key, across the profiles
-- characters share: two characters' own layouts of one name share one place, and renaming or deleting one moves or
-- drops it for both. The strip paints only the active profile (Bridge.lua), so a change in another profile reaches the
-- helper's mirror only when that profile is next active. Both cosmetic; accepted on 2026-10-01. `fn` says whether it
-- changed a profile's places; the save is stamped only then.
local function eachPlaces(fn)
	if not ns.db or ns.readOnly then return end
	local changed = false
	for _, profile in pairs(ns.db.profiles) do
		local places = type(profile) == "table" and profile.hudLayouts
		if type(places) == "table" then changed = fn(places) or changed end
	end
	if changed then ns.Touch(ns.db.profiles) end
end

-- The account's and the characters' own layouts by name (C_EditMode.GetLayouts lists no preset), as the HUD last saw
-- them, or nil before it saw any. The library reports a new layout by the index its cache lacks, the last one, while
-- an account layout goes in before the character ones (EditModeManager.lua): the new layout is the name this set lacks.
local seenLayouts
local function layoutNames()
	local names, info = {}, C_EditMode.GetLayouts()
	for _, layout in ipairs(info and info.layouts or {}) do names[layout.layoutName] = true end
	return names
end

-- The HUD joins Edit Mode when LibEditMode is loaded (ours, or another addon's newer copy: LibStub keeps the newest).
-- An API probe: a library whose file stopped half-way (a Forever build without a frame it hooks leaves it registered
-- without AddFrame) or that refuses the frame leaves the HUD to its header, and the log says why.
local function attachEditMode(f)
	local lib = LibStub and LibStub("LibEditMode", true)
	if not (lib and lib.AddFrame) then
		ns.Log("I", "no LibEditMode: the HUD's header moves it")
		return false
	end
	editMode = lib -- before the "layout" callback, which the library calls at once when it knows the layout
	local ok, err = pcall(function()
		lib:AddFrame(f, editModeMoved, HUD_HOME, L["hud.title"])
		lib:RegisterCallback("layout", function()
			seenLayouts = layoutNames()
			placeHUD(f)
		end)
		lib:RegisterCallback("rename", function(old, new)
			seenLayouts = layoutNames()
			eachPlaces(function(places)
				if places[old] == nil then return false end
				places[new], places[old] = places[old], nil
				return true
			end)
		end)
		-- A new layout starts where its source stood: the layout copied, else the active one, which "New layout" and
		-- "Save" on a changed preset copy without naming it to the library (EditModeManager.lua: ShowNewLayoutDialog()
		-- with no layout; MakeNewLayout saves, which calls back here, before it activates the new layout). The new
		-- layout is the name `seenLayouts` lacks, not the one the library reports (above); only a layout without a place
		-- takes one.
		lib:RegisterCallback("create", function(reported, _, source)
			local from = source and layoutKey(source) or lib:GetActiveLayoutName()
			local now, made = layoutNames(), {}
			for name in pairs(now) do
				if seenLayouts and not seenLayouts[name] then made[#made + 1] = name end
			end
			if #made == 0 then made[1] = reported end
			seenLayouts = now
			eachPlaces(function(places)
				local copied = false
				for _, name in ipairs(made) do
					if places[name] == nil and from and places[from] then
						places[name], copied = CopyTable(places[from]), true
					end
				end
				return copied
			end)
		end)
		lib:RegisterCallback("delete", function(name)
			seenLayouts = layoutNames()
			eachPlaces(function(places)
				if places[name] == nil then return false end
				places[name] = nil
				return true
			end)
		end)
		-- The client switched the active layout: the library calls back only when the active layout's index moved, and
		-- a new account layout made while a character layout is active takes that one's index.
		ns.RegisterEvent("EDIT_MODE_LAYOUTS_UPDATED", function() placeHUD(f) end)
	end)
	if ok then return true end
	editMode = nil
	ns.Log("W", "LibEditMode refused the HUD (%s): its header moves it", tostring(err))
	return false
end

-- Without the library: the header drags the HUD out of combat unless /qp lock fixed it, one place for every layout.
local function dragHandle(f)
	local handle = f.handle
	f:SetMovable(true)
	handle:RegisterForDrag("LeftButton")
	handle:SetScript("OnDragStart", function()
		if ns.profile and not ns.profile.hudLocked and not inCombat() then f:StartMoving() end
	end)
	handle:SetScript("OnDragStop", function()
		f:StopMovingOrSizing() -- anchors the HUD's nearest point to the same point of UIParent
		if not ns.profile then return end
		local point, _, _, x = f:GetPoint(1)
		ns.profile.hudPos = topPlace(f, point, x)
		placeHUD(f)
		ns.Touch()
	end)
	ns.AddCommand("lock", function()
		if not ns.profile then return end
		ns.profile.hudLocked = not ns.profile.hudLocked
		ns.Touch()
		ns.Print(ns.profile.hudLocked and "cmd.lock.on" or "cmd.lock.off")
	end, "cmd.lock.help")
end

-- A click on a line that acts ({ action?, menu?, rightMenu? }, a HUD line's or a Today row's): the right button opens
-- the right menu, the left one the menu, else runs the action.
function Sheet.Act(owner, item, button)
	if button == "RightButton" then
		if item.rightMenu then ns.Menu.Open(owner, item.rightMenu) end
	elseif item.menu then
		ns.Menu.Open(owner, item.menu)
	elseif item.action then
		item.action()
	end
end

local function build()
	if HUD.frame then return end
	local f = CreateFrame("Frame", nil, UIParent)
	f:SetSize(HUD_W, 26)
	f:SetFrameStrata("LOW")
	f:SetClampedToScreen(true)
	f.header = f:CreateTexture(nil, "BACKGROUND")
	f.header:SetPoint("TOPLEFT")
	f.header:SetSize(HUD_W, 26)
	Style.SetAtlas(f.header, "hudHeader")
	f.title = f:CreateFontString(nil, "OVERLAY")
	Style.SetFont(f.title, "hudHeader")
	f.title:SetPoint("LEFT", f.header, "LEFT", 10, 0)
	f.title:SetText(L["hud.title"])
	-- the header opens the sheet; without LibEditMode it is the drag handle too (dragHandle)
	local handle = CreateFrame("Button", nil, f)
	handle:SetAllPoints(f.header)
	handle:SetScript("OnClick", function() if not inCombat() then Sheet.Toggle() end end)
	f.handle = handle
	f:Hide()
	for i = 1, MAX_LINES do
		local line = CreateFrame("Button", nil, f)
		line:SetSize(HUD_W - 20, LINE_H)
		line:SetPoint("TOPLEFT", f, "TOPLEFT", 20, -26 - (i - 1) * (LINE_H + 2))
		line.text = line:CreateFontString(nil, "OVERLAY")
		Style.SetFont(line.text, "hudLine")
		line.text:SetPoint("TOPLEFT")
		line.text:SetWidth(HUD_W - 20)
		line.text:SetJustifyH("LEFT")
		-- the action bar's proc glow while the chip glows (Style.ProcGlow)
		line.glow = Style.ProcGlow(line, 8, 6)
		line:RegisterForClicks("LeftButtonUp", "RightButtonUp")
		line:SetScript("OnClick", function(self, button)
			if not InCombatLockdown() then Sheet.Act(self, self, button) end
		end)
		-- a line that acts lights up under the cursor, as the tracker's do (OBJECTIVE_TRACKER_COLOR's NormalHighlight)
		line.r, line.g, line.b = line.text:GetTextColor()
		line:SetScript("OnEnter", function(self)
			if self.action or self.menu or self.rightMenu then self.text:SetTextColor(HIGHLIGHT_FONT_COLOR:GetRGB()) end
			if not self.tooltip then return end
			GameTooltip:SetOwner(self, "ANCHOR_LEFT")
			self.tooltip(GameTooltip)
			GameTooltip:Show()
		end)
		line:SetScript("OnLeave", function(self)
			self.text:SetTextColor(self.r, self.g, self.b)
			GameTooltip:Hide()
		end)
		line:Hide()
		HUD.lines[i] = line
	end
	-- the thin XP bar under the QP line (slice 72; HUD.Render places and fills it)
	HUD.xp = ns.Kit.XPBar(f, HUD_W - 30, 3, "hudXP", true)
	HUD.frame = f
end

-- The QP line: "QP 12 · 38 moves today · 4-day streak", one row of the tracker's width (the XP, which moves with every
-- move, is the thin bar under it since slice 72, HUD.xp, and in numbers in its tooltip): moves, the player's own count
-- (David, 2026-10-04: "I did about 40 today and it only
-- counted 9", the pumps), the game's and the app's (slice 26; the app's share in the tooltip). It names no next move:
-- the quest decides the pump, which stays a surprise (David, 2026-09-30); the unfinished moves have their own line
-- (HUD.Lines).
function HUD.QPLine()
	local state = ns.state
	local parts = { L["hud.qp"]:format((ns.Level(state))) }
	local today = ns.DayCounts(state, ns.day())
	local moves = today.moves + today.app.moves
	parts[#parts + 1] = ns.Plural("hud.qp.moves", moves):format(moves)
	local streak = ns.Streak(state, ns.now())
	if streak > 0 then parts[#parts + 1] = L["hud.qp.streak"]:format(streak) end
	return table.concat(parts, " · ")
end

-- The QP line's tooltip in the kit's shape (Kit.Tip): the level, its XP (past the cap the rank's: the rank held, then
-- the XP toward the next one, slice 23), then in grey the rested XP, the app's share of today's moves, the week's vault
-- in one line (its goals stand on the Today tab) and the streak with its tokens; never what comes next (the quest
-- decides the pump; the Talents tab shows each ladder's next unlock). `where` keeps the hints true to the click there:
-- the HUD's line (nil) opens the sheet and pauses on a right-click, the Character window's QP plate ("plate") docks the
-- sheet, and the sheet's XP bar ("sheet") takes no click.
function HUD.QPTooltip(tooltip, where)
	local state = ns.state
	local level, into, need, paragon = ns.Level(state)
	local pct = math.floor(100 * into / math.max(1, need))
	local xp = level >= C.level_cap and L["tip.hud.xpParagon"]:format(number(into), number(need), paragon + 1, pct)
		or L["tip.hud.xp"]:format(number(into), number(need), pct)
	if paragon > 0 then xp = L["tip.hud.paragon"]:format(paragon) .. "\n" .. xp end
	local facts = {}
	local rested = ns.Rested(state)
	if rested > 0 then -- the game's own XP tooltip speaks of rest only while the character is rested
		facts[#facts + 1] = L["tip.hud.rested"]:format(number(rested))
	end
	local today = ns.DayCounts(state, ns.day())
	if today.app.moves > 0 then facts[#facts + 1] = ns.Plural("tip.hud.app", today.app.moves):format(today.app.moves) end
	-- each goal's tiers (the embers' three, slice 55)
	local vault, reset = ns.Vault(state, ns.now())
	local done, goals = 0, 0
	for _, v in ipairs(vault) do done, goals = done + v.paid, goals + v.tiers end
	facts[#facts + 1] = L["tip.hud.vaultSum"]:format(done, goals, date("%a %H:%M", reset))
	local streak, tokens, best = ns.Streak(state, ns.now())
	facts[#facts + 1] = ns.Plural("tip.hud.streak", streak):format(streak, best, tokens)
	local hint = where ~= "sheet" and { L["tip.hud.click"], where == nil and L["tip.hud.pause"] or nil } or nil
	ns.Kit.Tip(L["tip.hud.title"]:format(level, L["build." .. ns.Build(state)], L["focus." .. ns.Focus(state)]), xp,
		{ facts = facts, hint = hint })(tooltip)
end

-- The embers line (slice 55): today's embers of embers_day, lit and unlit, under the QP line; its click opens the
-- player's cardio step's card, whose Start runs the next one, its tooltip the week's. No streak and no loss words: a
-- game day starts at none.
local function embersLine()
	local lit, most, week = ns.Embers(ns.state, ns.now())
	return { text = L["hud.embers"]:format(lit, most) .. " " .. Style.Embers(lit, most),
		action = function() Sheet.OpenStepCard(ns.CARDIO, ns.Rung(ns.state, ns.CARDIO)) end,
		tooltip = ns.Kit.Tip(L["vault.label.embers"], L["toast.embers.sub"]:format(week),
			{ hint = L["tip.hud.embers.click"] }) }
end

-- This character's pause choices (David, 2026-10-01: "If I play with friends I don't want to do it sometimes"): for an
-- hour, for four hours (2026-10-04), for the rest of the game day, or off on it until resumed. The QP line's
-- right-click (Lc.Pause), the pump's Pause button and its "I feel unwell" view (Panel.lua, Lc.PausePump) list them,
-- each as { mode, label }. The two menus add "While grouped" in a mixed party (Sheet.PauseChoices).
function Sheet.PauseOptions()
	return { { ns.PAUSE.hour, L["hud.pause.hour"] }, { ns.PAUSE.fourHours, L["hud.pause.fourHours"] },
		{ ns.PAUSE.day, L["hud.pause.day"]:format(C.day_offset_h) }, { ns.PAUSE.char, L["hud.pause.char"] } }
end

-- The choices as a menu under its title; `act(mode)` pauses. In a mixed party "While grouped" leads them, the pause
-- the party's question offers (Lc.AskParty), for a player who banked first or closed the popup.
function Sheet.PauseChoices(root, act)
	root:CreateTitle(L["hud.pause.title"])
	if Lc.MixedParty() then root:CreateButton(L["hud.pause.group"], function() act(ns.PAUSE.group) end) end
	for _, option in ipairs(Sheet.PauseOptions()) do
		root:CreateButton(option[2], function() act(option[1]) end)
	end
end

local function pauseMenu(_, root)
	Sheet.PauseChoices(root, Lc.Pause)
end

-- The pause's line: how long it holds (a timed pause's end, else 04:00, while grouped or off); a click resumes, as its
-- tooltip's hint says (a HUD line names a state, never "click to": play SPEC 14).
local function pausedLine(mode, ends)
	local line = ends and L["hud.paused.until"]:format(clockTime(ends))
		or mode == ns.PAUSE.day and L["hud.paused.until"]:format(("%02d:00"):format(C.day_offset_h))
		or mode == ns.PAUSE.group and L["hud.paused.group"]
		or L["hud.paused.char"]
	return { text = line, action = Lc.Resume,
		tooltip = ns.Kit.Tip(L["hud.paused.title"], L["hud.paused.tip"], { hint = L["hud.paused.click"] }) }
end

-- The chip: the pump that comes, its own moves apart from the unfinished ones riding along, so the quest's size stays
-- readable while its window is open ("Next pump: 3 moves + 31 unfinished"); it names no move. A click starts it now. A
-- pump started from a step's card (`p.chosen`) has no riders: its one move is its own, the unfinished row it pays too,
-- as its card's context line says (Panel.lua).
local function chipLine(p)
	local riders = 0
	for _, slot in ipairs(p.moves) do
		if slot.owed and not p.chosen then riders = riders + 1 end
	end
	local own = #p.moves - riders
	local line = own == 0 and L["hud.chip.riders"]:format(riders)
		or riders == 0 and ns.Plural("hud.chip", own):format(own)
		or ns.Plural("hud.chip.both", own):format(own, riders)
	return { text = line, action = Lc.Demand, glow = p.state == "ready" or p.state == "card",
		tooltip = ns.Kit.Tip(L["tip.chip.title"],
			function() return L[p.snooze == "hidden" and "tip.chip.snoozed" or "tip.chip.when"] end,
			{ hint = L["tip.chip.click"] }) }
end

-- The pumps that wait for the flight (Lc.Waiting; "On my next flight", docs/plan.md slice 70): how many, never their
-- moves, which each composes as it opens. A click starts the first now, as the chip's does.
local function waitingLine(n)
	return { text = ns.Plural("hud.wait", n):format(n), action = Lc.Demand,
		tooltip = ns.Kit.Tip(L["tip.wait.title"], L["tip.wait.body"], { hint = L["tip.wait.click"] }) }
end

-- "since 14:02" today, "since 3 Oct" before (the HUD's unfinished moves, the pump card's reason line)
local function since(ts)
	return ns.day(ts) == ns.day() and clockTime(ts) or shortDate(ts)
end
Sheet.Since = since

-- An owed move's name; a row can name a move this addon lacks (retired before the merge map, or a newer addon's
-- through the mirror), which waits on the setup and only Skip clears
local function owedName(id)
	return ns.Library[id] and moveName(id) or L["hud.held.unknown"]
end

-- The unfinished moves the setup holds back, each with Skip, one click and no confirm (Lc.SkipHeld): no pump carries
-- them to a Ready view, where Skip lives for the others; then the settings, where a switch or a flag lets them in.
local function heldMenu(_, root)
	root:CreateTitle(L["hud.held.title"])
	local aside = ns.OwedAside(ns.state, ns.profile, ns.now())
	for _, r in ipairs(ns.OwedHeldRows(ns.state, ns.profile, ns.now())) do
		if not aside[r.key] then
			root:CreateButton(L["hud.held.skip"]:format(owedName(r.move), since(r.ts)), function() Lc.SkipHeld(r) end)
		end
	end
	root:CreateDivider()
	root:CreateButton(L["hud.held.settings"], function() ns.Settings.Open() end)
end

-- The unfinished line (David, 2026-10-04, after 31 of them met him as one pump of 34): where they go and when, never
-- the same number twice; its tooltip, in the kit's shape, says what they take and names them on one line (up to
-- owed_tip_moves, "Push-up ×2 · Plank", then "+n more"), then in grey where they came from, where they go and the way
-- out (Skip on a move's Ready view; for the ones the setup holds back, the line's menu). `shown`: the count the line
-- says while it rolls down to `owed` after a move was paid (HUD.Rolled; with none left, the count's words alone).
local function owedLine(owed, locked, shown)
	local says = shown or owed
	if owed == 0 then return { text = ns.Plural("hud.owed", says):format(says) } end
	local aside, asideCount = ns.OwedAside(ns.state, ns.profile, ns.now())
	local context = not locked and Lc.OwingContext()
	local ready = locked and 0 or ns.RidersReady(ns.state, ns.profile, ns.now())
	local waiting = (locked or ready == owed) and 0 or #ns.OwedHeldRows(ns.state, ns.profile, ns.now()) - asideCount
	local line
	if locked then
		line = L["hud.owed.locked"]:format(says, C.day_offset_h)
	elseif ready == 0 then -- every one waits on the setup, wherever the player is
		line = L["hud.owed.waiting"]:format(says)
	elseif context then
		line = L["hud.owed.after." .. context]:format(says)
	elseif ready == owed then
		line = ns.Plural("hud.owed", says):format(says)
	else
		line = ns.Plural("hud.owed.some", ready):format(says, ready)
	end
	local function tooltip(tip)
		local order, count, banked = {}, {}, 0
		local as = ns.RidesAs(ns.state, ns.profile, ns.now()) -- an owed backup by the move it rides as
		for _, r in ipairs(ns.OwedList(ns.state)) do
			if not aside[r.key] then
				local id = as[r.key] or r.move
				if not count[id] then order[#order + 1] = id end
				count[id] = (count[id] or 0) + 1
				if r.outcome == O.banked then banked = banked + 1 end
			end
		end
		local names = {}
		for i = 1, math.min(#order, C.owed_tip_moves) do
			local n = count[order[i]]
			names[i] = n > 1 and L["tip.owed.move"]:format(owedName(order[i]), n) or owedName(order[i])
		end
		if #order > C.owed_tip_moves then names[#names + 1] = L["tip.owed.more"]:format(#order - C.owed_tip_moves) end
		local from = {}
		if banked > 0 then from[#from + 1] = L["tip.owed.banked"]:format(banked) end
		if owed > banked then from[#from + 1] = L["tip.owed.owed"]:format(owed - banked) end
		local facts = { table.concat(from, " · ") }
		-- where they go: after the lock, after the place that owes them, or into the next pump (none while all wait)
		local where = locked and L["tip.owed.locked"]:format(C.day_offset_h)
			or ready > 0 and (context and L["tip.owed.after"] or L["tip.owed.next"])
		if where then facts[#facts + 1] = where end
		if waiting > 0 then facts[#facts + 1] = L["tip.owed.held"]:format(waiting) end
		facts[#facts + 1] = L["tip.owed.skip"]
		-- the phone's way to pay them, until its step is done (David, 2026-10-09: nudges at a moment of value)
		if not select(2, ns.Setup.Steps()) then facts[#facts + 1] = L["status.nudge.owed"] end
		local hint = not locked and { L[ready > 0 and "tip.owed.click" or "tip.owed.clickHeld"],
			ready > 0 and waiting > 0 and L["tip.owed.rightHeld"] or nil } or nil
		local body = ns.Plural("tip.owed.size", owed):format(owed, math.ceil(owed * C.move_estimate_s / 60))
		if #names > 0 then body = body .. "\n" .. table.concat(names, " · ") end
		ns.Kit.Tip(L["tip.owed.title"], body, { facts = facts, hint = hint })(tip)
	end
	if locked then return { text = line, tooltip = tooltip } end
	if ready == 0 then return { text = line, menu = heldMenu, tooltip = tooltip } end
	return { text = line, action = Lc.Demand, rightMenu = waiting > 0 and heldMenu or nil, tooltip = tooltip }
end
HUD.OwedLine = owedLine -- the Today tab's unfinished row too (SheetPages.lua)

-- How many of the `owed` unfinished moves the lists show: all but those the place sets aside (ns.OwedAside).
function HUD.OwedShown(owed)
	if owed == 0 then return 0 end
	return owed - select(2, ns.OwedAside(ns.state, ns.profile, ns.now()))
end

-- The unfinished count the HUD's line says for the `owed` it shows (slice 72): as a banked move is paid (HUD.paid, a
-- paid row that is no Skip's) the count rolls down from what the line said to what is left, a number a tick over
-- Style.motion.carry (Kit.Carry draws the HUD again at each), at the payment, under the pump's dim too (the dim hides
-- nothing whole, half of it in a group), so the line never says a count that is paid. A Skip, a bank and the game's
-- reduced motion roll nothing: the count stands at once, a Skip's during a roll as the roll ends.
function HUD.Rolled(owed)
	if HUD.rollOwed then return HUD.rollOwed end
	local seen = HUD.seenOwed
	local down = HUD.paid and seen ~= nil and seen > owed and ns.Tree.Moving()
	HUD.paid, HUD.seenOwed = nil, owed
	if not down then return owed end
	HUD.rollOwed = seen
	ns.Kit.Carry("hudOwed", function(k)
		local n = math.floor(seen + (owed - seen) * k + 0.5)
		if n ~= HUD.rollOwed then
			HUD.rollOwed = n
			HUD.Render()
		end
	end, function()
		HUD.rollOwed = nil
		HUD.Render()
	end)
	return seen
end

-- The lines as { text, action?, menu?, rightMenu?, glow?, tooltip?, xp? }, top to bottom (a menu opens on the click,
-- the cheers' and the held unfinished moves'; a right menu on the right-click, the pause; `xp`: the thin XP bar stands
-- under it). Pure apart from what it reads and the unfinished count's roll (HUD.Rolled).
function HUD.Lines()
	local out = {}
	local p = Lc.pump
	if p and CHIP[p.state] then out[#out + 1] = chipLine(p) end
	local waiting = Lc.Waiting()
	if waiting > 0 then out[#out + 1] = waitingLine(waiting) end
	local pauseMode, pauseEnds = Lc.Paused()
	local locked = Lc.Locked()
	if ns.profile and ns.profile.enabled == false then
		out[#out + 1] = { text = L["hud.off"], action = function() ns.Settings.Open() end,
			tooltip = ns.Kit.Tip(L["hud.off"], nil, { hint = L["tip.hud.off.click"] }) }
	elseif pauseMode then
		out[#out + 1] = pausedLine(pauseMode, pauseEnds)
	elseif qpShown() then
		out[#out + 1] = { text = HUD.QPLine(), action = Sheet.Toggle, tooltip = HUD.QPTooltip, rightMenu = pauseMenu,
			xp = true }
		if not locked then out[#out + 1] = embersLine() end -- the felt-unwell lock asks for no cardio
	end
	if locked then out[#out + 1] = { text = L["hud.locked"]:format(C.day_offset_h) } end
	local owed = HUD.OwedShown(ns.state and ns.OwedCount(ns.state) or 0)
	local says = HUD.Rolled(owed)
	if says > 0 then out[#out + 1] = owedLine(owed, locked, says) end
	-- the pact and the duel, the party's moves today, the Join line, a finished pump to cheer, the guild's digest
	-- (Social.lua)
	for _, item in ipairs(ns.Social.HudLines()) do out[#out + 1] = item end
	-- a click once the helper runs is the reload that reads its copy (2026-10-13)
	if ns.HelperStale() then
		out[#out + 1] = { text = L["status.hud.stale"], action = C_UI.Reload,
			tooltip = ns.Kit.Tip(L["status.hud.stale.title"], L["status.hud.stale.tip"],
				{ hint = L["status.hud.stale.click"] }) }
	end
	-- a save the game did not load back and the helper's copy did not restore: the manual way to keep a session without
	-- the helper, the export string; the tooltip names the helper, which keeps it, while it does not run at this load
	if ns.Settings.BetaUnsaved() then
		out[#out + 1] = { text = L["status.hud.lost"], action = function() ns.Settings.OpenIO(true) end,
			tooltip = ns.Kit.Tip(L["status.hud.lost.title"], not ns.Setup.HelperRuns() and L["status.nudge.save"] or nil,
				{ hint = L["status.hud.lost.click"] }) }
	end
	return out
end

function HUD.Render()
	if not HUD.frame then return end
	local shown = not Lc.combat and not InCombatLockdown() and ns.db and not (ns.profile and ns.profile.hud == false)
	local lines = shown and HUD.Lines() or {}
	local y = -26 -- each line as tall as its wrapped text, the frame as tall as all of them (the clamp covers it)
	local bar, barY = HUD.xp, nil
	for i, line in ipairs(HUD.lines) do
		local item = lines[i]
		if item then
			line.text:SetText(item.text)
			local h = math.max(LINE_H, math.ceil(line.text:GetStringHeight()))
			line:SetHeight(h)
			line:ClearAllPoints()
			line:SetPoint("TOPLEFT", HUD.frame, "TOPLEFT", 20, y)
			y = y - h - 2
			if item.xp then
				barY = y
				y = y - bar.rested:GetHeight() - 4
			end
			line.action, line.menu, line.rightMenu = item.action, item.menu, item.rightMenu
			line.tooltip = item.tooltip
			Style.ShowGlow(line.glow, item.glow == true)
			line:Show()
		else
			line.action, line.menu, line.rightMenu, line.tooltip = nil, nil, nil, nil
			Style.ShowGlow(line.glow, false)
			line:Hide()
		end
	end
	HUD.frame:SetHeight(-y)
	HUD.frame:SetShown(#lines > 0)
	-- the thin XP bar under the QP line, once its frame shows: it fills on a gain from the XP it showed before
	-- (Kit.XPBar), under the pump's dim only once the dim has gone
	bar:SetShown(barY ~= nil)
	bar.rested:SetShown(barY ~= nil)
	if barY then
		bar.rested:ClearAllPoints()
		bar.rested:SetPoint("TOPLEFT", HUD.frame, "TOPLEFT", 20, barY)
		local xp = ns.state.sheet.xp
		if not bar.xp then
			bar:SetXP(xp)
		elseif xp ~= bar.xp and not HUD.Covered() then
			bar:FillXP(bar.xp, xp)
		end
	end
	-- The lock and a pause end by the clock alone, so while either holds the HUD looks again every minute.
	if Lc.Locked() or Lc.Paused() then
		if not T.active("lock") then T.every("lock", 60, HUD.Render) end
	else
		T.cancel("lock")
	end
end

-- a banked move paid (never a Skip's row, which earns nothing): the unfinished line's count rolls down (HUD.Rolled)
ns.On("Row", function(row)
	if row and row.outcome == O.paid and not ns.HasFlag(row.flags, "skipped") then HUD.paid = true end
end)

for _, message in ipairs({ "State", "Row", "Setup", "Trial", "Saving", "Clock", "Owed", "DB", "Progress",
	"Settings", "Social" }) do
	ns.On(message, HUD.Render)
end

-- The sheet -----------------------------------------------------------------------------------------------------------

-- A plain scroll list: a scroll frame and its content, rows laid out by the caller from the top; vertical only, so
-- ConsolePort's cursor scrolls it to whatever it reaches (the Talents tab's board too, Tree.lua).
local function scrollList(parent, width, height)
	local scroll = CreateFrame("ScrollFrame", nil, parent, "UIPanelScrollFrameTemplate")
	scroll:SetSize(width, height)
	local content = CreateFrame("Frame", nil, scroll)
	content:SetSize(width, 1)
	scroll:SetScrollChild(content)
	scroll.content = content
	scroll.rows = {}
	return scroll
end
Sheet.ScrollList = scrollList

-- The muscle buttons -----------------------------------------------------------------------------------------------

-- A button's picture: the move's key-pose cell, or the muscle's gear-slot silhouette uncropped (a reused button keeps
-- the last cell's coordinates otherwise); without a muscle (a ladder's row), the empty slot.
local function setIcon(b, file, l, r, t, bb, slot)
	SetItemButtonTexture(b, file or Style.SlotIcon(slot))
	if b.icon then
		if file then b.icon:SetTexCoord(l, r, t, bb) else b.icon:SetTexCoord(0, 1, 0, 1) end
	end
end
Sheet.SetIcon = setIcon

local function muscleIcon(map)
	-- the key pose of the move that earned the muscle's best rung, else its latest move
	local best = map.best or (map.last and map.last[1] and map.last[1].move)
	local file, l, r, t, b
	if best then file, l, r, t, b = Style.MoveIcon(best) end
	return file, l, r, t, b
end

local function muscleTooltip(button)
	local id = button.muscle
	local map = Sheet.map and Sheet.map[id]
	if not map then return end
	-- over the stat pane, never over the figures the muscle lights on
	GameTooltip:SetOwner(button, "ANCHOR_NONE")
	GameTooltip:SetPoint("TOPLEFT", F.bodyPane or button, "TOPRIGHT", 4, -40)
	GameTooltip_SetTitle(GameTooltip, L["muscle." .. id], qualityColor(map.quality))
	if map.quality then
		local best = map.best and ns.Library[map.best]
		local ladder = id == HEART and L["ladder.6"] or (best and best.ladder and L["ladder." .. best.ladder])
			or L["tip.muscle.ladders"]
		-- its best step on the one scale; a muscle only moves no ladder places trained has none (Starter alone)
		local quality = L["quality." .. map.quality]
		GameTooltip:AddLine(map.ladderStep > 0 and L["tip.muscle.quality"]:format(quality, map.ladderStep, ladder)
			or quality, 1, 1, 1)
		if best then
			-- its name alone for a move that turned timed, whose old sets stay but hold no record (ns.HoldsRecord)
			local record = ns.HoldsRecord(ns.state, map.best) and ns.Record(ns.state, map.best) or nil
			local stars = record and best.ladder and ns.STRENGTH[best.ladder]
				and ns.Stars(best.ladder, best.rung, map.best, record) or 0
			local line = record and L["tip.muscle.best"]:format(moveName(map.best), moveAmount(map.best, record))
				or moveName(map.best)
			if stars > 0 then line = line .. " " .. Style.Stars(stars) end
			GameTooltip:AddLine(line, 1, 1, 1)
		end
	else
		GameTooltip:AddLine(L["tip.muscle.untrained"], unpack(Style.saga.note))
	end
	local hard = ns.Plural("tip.muscle.hard", map.hard):format(map.hard)
	GameTooltip:AddLine(L["tip.muscle.week"]:format(weekMoves(map.exact or map.week), hard), 1, 1, 1)
	local note = Style.saga.note
	if map.step >= 3 then GameTooltip:AddLine(L["tip.muscle.fresh"], note[1], note[2], note[3], true) end
	-- its three latest moves (the fold keeps five), each with its day and time in the notes' grey: nine lines at most
	for i = 1, math.min(3, #(map.last or {})) do
		local e = map.last[i]
		GameTooltip:AddDoubleLine(moveName(e.move), date("%a %H:%M", e.ts), 1, 1, 1, note[1], note[2], note[3])
	end
	GameTooltip_AddInstructionLine(GameTooltip, L["tip.muscle.click"]:format(L["muscle." .. id]))
	GameTooltip:Show()
end

-- The flyout beside a muscle, in the equipment flyout's look (2026-10-16; one row of every move by rung until then):
-- the muscle's name and how many of its moves the pool holds, then its moves by ladder in the trees' order (each
-- ladder's name in the notes' grey on its own line across the flyout, wrapped where a large Text Size outgrows it and
-- never cut, its moves under it, six to a line, by step; the moves no ladder places under "Not on a ladder"), each
-- bordered in its quality. By default only the moves within the rung rule (ns.AboveReach), a left-out one greyed (its
-- tooltip says "Not in your pool now"); "All N moves", Kit.More's bar, shows the ones ahead too, greyed, for that
-- opening. A left-click opens the move's step card on the Talents tab (Sheet.ShowMoveCard), in a fight too; a
-- right-click's menu marks it a favourite or leaves it out or takes it back in (the Exercises page's switch), and
-- clears its record where it has one (Settings.lua). 8 + 6 x 43 + 14 = 280 across (fly.w and the scroll bar's room),
-- so it never reaches the stat pane.
local FLYOUT_ROW = 6

-- The ladders in the trees' order (Core.lua's ns.TREES, each tree's columns left to right): the flyout's lines and the
-- Records tab's sections (SheetPages.lua) follow it, as the Talents tab shows them. A fresh list each call.
function Sheet.TreeOrder()
	local order = {}
	for t in ipairs(ns.TREES) do
		for _, l in ipairs(ns.TreeLadders(t)) do order[#order + 1] = l end
	end
	return order
end

local function primaryMoves(muscle)
	local list = {}
	for id, m in pairs(ns.Library) do
		for _, mu in ipairs(m.mp) do
			if mu == muscle then list[#list + 1] = id end
		end
	end
	return list
end

local function renderFlyout(fly, button)
	local state, f = ns.state, filter()
	local list = primaryMoves(button.muscle)
	-- the moves by the ladder they climb or scale by (ns.ScaleLadder; 0 for none), each group by step, then id
	local groups, pool, ahead = {}, 0, 0
	for _, id in ipairs(list) do
		local m = ns.Library[id]
		local above = ns.AboveReach(state, id)
		if above then ahead = ahead + 1 elseif ns.Passes(id, f) then pool = pool + 1 end
		if fly.all or not above then
			local l = ns.ScaleLadder(m) or 0
			groups[l] = groups[l] or {}
			table.insert(groups[l], id)
		end
	end
	local order = Sheet.TreeOrder()
	order[#order + 1] = 0
	fly.name:SetText(L["muscle." .. button.muscle])
	fly.count:SetText(L["sheet.flyout.count"]:format(pool, #list))
	local w = fly.w
	local y, used, lines = 8 + fly.name:GetStringHeight(), 0, 0
	fly.count:ClearAllPoints()
	-- on the name's line (the large font's descent up), or under the name where a large Text Size leaves no room
	if fly.name:GetStringWidth() + 12 + fly.count:GetStringWidth() <= w - 2 then
		fly.count:SetPoint("BOTTOMRIGHT", fly, "TOPRIGHT", -8, -(y - 1))
	else
		fly.count:SetPoint("TOPLEFT", fly, "TOPLEFT", 8, -(y + 2))
		y = y + 2 + fly.count:GetStringHeight()
	end
	y = y + 6
	fly.rule:SetPoint("TOPLEFT", 8, -y)
	fly.rule:SetPoint("TOPRIGHT", -8, -y)
	y = y + 7
	-- the lines on the scroll list's content (ry from its top), which scrolls where they outgrow the screen: each
	-- ladder's name over its moves, 6 under the moves of the ladder before
	local top, ry = y, 0
	for _, l in ipairs(order) do
		local ids = groups[l]
		if ids then
			table.sort(ids, function(a, b)
				local sa, sb = ns.MoveStep(a) or ns.Library[a].rung or 0, ns.MoveStep(b) or ns.Library[b].rung or 0
				if sa ~= sb then return sa < sb end
				return a < b
			end)
			lines = lines + 1
			local label = fly.labels[lines]
			if not label then
				label = ns.Kit.Text(fly.rows, "bodyWhite", "LEFT", Style.saga.note, w - 16)
				fly.labels[lines] = label
			end
			label:SetText(l > 0 and L["ladder." .. l] or L["records.other"])
			if lines > 1 then ry = ry + 6 end
			label:ClearAllPoints()
			label:SetPoint("TOPLEFT", fly.rows, "TOPLEFT", 8, -ry)
			label:Show()
			ry = ry + label:GetStringHeight() + 4
			for i, id in ipairs(ids) do
				used = used + 1
				local b = fly.buttons[used]
				b.move = id
				b:ClearAllPoints()
				b:SetPoint("TOPLEFT", 8 + ((i - 1) % FLYOUT_ROW) * PITCH, -(ry + math.floor((i - 1) / FLYOUT_ROW) * PITCH))
				local file, ml, mr, mt, mb = Style.MoveIcon(id)
				setIcon(b, file, ml, mr, mt, mb, button.muscle)
				SetItemButtonQuality(b, ns.MoveQuality(id))
				SetItemButtonDesaturated(b, ns.AboveReach(state, id) or not ns.Passes(id, f))
				b:Show()
			end
			ry = ry + math.ceil(#ids / FLYOUT_ROW) * PITCH
		end
	end
	for i = used + 1, #fly.buttons do fly.buttons[i]:Hide() end
	for i = lines + 1, #fly.labels do fly.labels[i]:Hide() end
	-- the lines as tall as they are, at most the screen less the flyout's head, its More bar and 32 above and below
	-- (at UI scale 1 a muscle's every move outgrows the 768 units: Quads' 74 take 17 lines); the scroll bar beside them
	local room = math.max(PITCH, math.floor(UIParent:GetHeight() - 2 * 32 - top - (ahead > 0 and 2 + 26 or 0) - 8))
	local h = math.min(ry, room)
	fly.rows:SetSize(w, math.max(1, ry))
	fly.scroll:ClearAllPoints()
	fly.scroll:SetPoint("TOPLEFT", 0, -top)
	fly.scroll:SetSize(w, h)
	y = top + h
	-- the ones ahead one click away, where there are any
	fly.more:SetShown(ahead > 0)
	if ahead > 0 then
		fly.more.words:SetText(L["sheet.flyout.all"]:format(#list))
		fly.more:SetOpen(fly.all)
		fly.more:ClearAllPoints()
		fly.more:SetPoint("TOPLEFT", 8, -(y + 2))
		y = y + 2 + fly.more:GetHeight()
	end
	fly:SetSize(w + 14 + (ry > h and 14 or 0), y + 8)
end

local function showFlyout(button)
	local fly = F.flyout
	if fly.muscle == button.muscle and fly:IsShown() then
		fly:Hide()
		return
	end
	fly.muscle, fly.button, fly.all = button.muscle, button, false
	renderFlyout(fly, button)
	fly.scroll:SetVerticalScroll(0)
	fly:ClearAllPoints()
	-- toward the figure: the right column's to its left, since the muscle tooltip stands over the stat pane
	if button.rightColumn then
		fly:SetPoint("TOPRIGHT", button, "TOPLEFT", -6, 6)
	else
		fly:SetPoint("TOPLEFT", button, "TOPRIGHT", 6, 6)
	end
	fly:Show()
end

local function flyoutMenu(b)
	local id = b.move
	ns.Menu.Open(b, function(_, root)
		root:CreateTitle(moveName(id))
		-- a favourite: the engine takes it first among the moves the day's rules leave (slice 20)
		local favourite = ns.profile.favourites[id] == true
		root:CreateButton(L[favourite and "menu.unfavourite" or "menu.favourite"], function()
			ns.Settings.SetFavourite(id, not favourite)
			Sheet.Refresh()
		end)
		local excluded = ns.Settings.Excluded(id)
		-- an opt-in move (the headstand) was never in: it is added, not taken back
		local add = ns.Library[id].optIn and "menu.optIn" or "menu.include"
		root:CreateButton(L[excluded and add or "menu.exclude"], function()
			ns.Settings.SetExcluded(id, not excluded)
			Sheet.Refresh()
			local fly = F.flyout
			if fly:IsShown() and fly.button then renderFlyout(fly, fly.button) end
		end)
		-- under a divider, Correct a count and the record's clear (slice 44)
		ns.Settings.AddRecordItems(root, id, true)
	end)
end
Sheet.FlyoutMenu = flyoutMenu -- a move tile's right-click on the Talents tab's card (Tree.lua)

-- A move's step card on the Talents tab (the flyout's and the Records and Feats rows' left-click, 2026-10-16): the tab
-- switches, the step is selected and scrolled into view and its card opens (Tree.ShowStep), on the move itself where it
-- is not the step's own ("Back to the step", Tree.ShowMove); a move no ladder places opens a card on that move alone
-- (Tree.ShowLoose). In a fight too: none of it is protected, and the card greys its own Start there.
function Sheet.ShowMoveCard(id)
	local m = ns.Library[id]
	if not (m.ladder and m.rung) then
		Sheet.ShowTalents()
		return ns.Tree.ShowLoose(id)
	end
	Sheet.ShowStepCard(m.ladder, m.rung)
	if id ~= ns.Tree.StepMove(m.ladder, m.rung) then ns.Tree.ShowMove(id) end
end

-- The Talents tab shown (the flyout closed): the sheet's tab switched and drawn.
function Sheet.ShowTalents()
	if F.flyout then F.flyout:Hide() end
	Sheet.tab = "ladders"
	Sheet.Refresh()
end

-- A step's card on the Talents tab (Sheet.ShowMoveCard's, and the Deeds tab's Mastered steps row, slice 42): the tab
-- switched, the step selected, scrolled into view and its card open (Tree.ShowStep).
function Sheet.ShowStepCard(l, r)
	Sheet.ShowTalents()
	ns.Tree.ShowStep(l, r)
end

local function buildFlyout(parent)
	local fly = CreateFrame("Frame", nil, parent, "TooltipBackdropTemplate")
	fly:SetSize(200, SLOT + 12)
	fly:SetFrameStrata("DIALOG")
	fly:SetClampedToScreen(true)
	fly.buttons, fly.labels = {}, {}
	fly.name = ns.Kit.Text(fly, "large", "LEFT")
	fly.name:SetPoint("TOPLEFT", 8, -8)
	fly.count = ns.Kit.Text(fly, "bodyWhite", "RIGHT", Style.saga.note)
	fly.rule = ns.Kit.Rule(fly, Style.saga.rule)
	-- its lines in a scroll list (the kit's), its buttons and labels on the list's content (fly.rows)
	fly.scroll = ns.Kit.Scroll(fly)
	fly.rows = fly.scroll.content
	-- the ones ahead: the More bar's look, its own rows the flyout's (renderFlyout lays them out again)
	fly.w = 8 + FLYOUT_ROW * PITCH -- the lines' width, the scroll bar's room beside it
	fly.more = ns.Kit.More(fly, fly.w + 14 - 16, nil, function() end)
	fly.more.onToggle = function(open)
		fly.all = open
		if fly.button then renderFlyout(fly, fly.button) end
	end
	-- as many buttons as the muscle with the most primary moves needs
	local most = 1
	for id = 1, ns.MUSCLES do most = math.max(most, #primaryMoves(id)) end
	for i = 1, most do
		local b = CreateFrame("ItemButton", nil, fly.rows)
		b:SetSize(SLOT, SLOT)
		b:RegisterForClicks("LeftButtonUp", "RightButtonUp")
		b:SetScript("OnEnter", function(self)
			GameTooltip:SetOwner(self, "ANCHOR_RIGHT")
			Sheet.MoveTooltip(GameTooltip, self.move, nil, { click = "talents", menu = true })
		end)
		b:SetScript("OnLeave", function() GameTooltip:Hide() end)
		b:SetScript("OnClick", function(self, mouse)
			if mouse == "RightButton" then return flyoutMenu(self) end
			Sheet.ShowMoveCard(self.move)
		end)
		fly.buttons[i] = b
	end
	fly:Hide()
	F.flyout = fly
end

-- A gear slot as the Character window draws one: an item button on the slot's frame, the move's key pose in it and
-- the quality as Blizzard's border (setIcon, SetItemButtonQuality). The muscle buttons and the Talents tab's steps and
-- tiles wear it (Tree.lua).
-- Starter (Poor) wears none, as a junk item in the bags does (BAG_ITEM_QUALITY_COLORS starts at Common, Forever
-- 70205): a Poor grey frame (#9D9D9D) would only copy Common's (COMMON_GRAY_COLOR, #A8A8A8).
local function slotButton(parent)
	local b = CreateFrame("ItemButton", nil, parent)
	b:SetSize(SLOT, SLOT)
	b.backing = b:CreateTexture(nil, "BACKGROUND", nil, -1)
	b.backing:SetPoint("CENTER")
	b.backing:SetSize(PITCH, PITCH)
	Style.SetAtlas(b.backing, "gearSlot")
	return b
end
Sheet.SlotButton = slotButton

local function muscleButton(parent, id)
	local b = slotButton(parent)
	b.muscle = id
	b.rightColumn = false -- buildBody sets it for the right column, whose flyout opens to its left
	b.minDisplayCount = 0 -- SetItemButtonCount shows a count above this (1 by default): one move, rung 1 show too
	-- a muscle trained since the sheet was last opened pulses once (Blizzard's new-item cadence)
	b.pulse = b:CreateAnimationGroup()
	local a1 = b.pulse:CreateAnimation("Alpha")
	a1:SetFromAlpha(1)
	a1:SetToAlpha(0.2)
	a1:SetDuration(0.5)
	a1:SetOrder(1)
	local a2 = b.pulse:CreateAnimation("Alpha")
	a2:SetFromAlpha(0.2)
	a2:SetToAlpha(1)
	a2:SetDuration(0.5)
	a2:SetOrder(2)
	b:SetScript("OnEnter", function(self)
		Sheet.Spotlight(self.muscle)
		muscleTooltip(self)
	end)
	b:SetScript("OnLeave", function()
		Sheet.Spotlight(nil)
		GameTooltip:Hide()
	end)
	b:SetScript("OnClick", function(self) showFlyout(self) end)
	Sheet.muscles[id] = b
	return b
end

local function renderMuscles()
	local map = ns.BodyMap(ns.state, ns.now(), filter())
	Sheet.map = map
	local mode = ns.profile.mapMode or "week"
	local colorblind = GetCVarBool("colorblindMode")
	local wk = ns.WeekOf(ns.now())
	if Sheet.seenWk ~= wk then Sheet.seenWeek, Sheet.seenWk = nil, wk end
	local seen = Sheet.seenWeek or {}
	Sheet.seenWeek = seen
	for id, b in pairs(Sheet.muscles) do
		local m = map[id]
		local file, l, r, t, bb = muscleIcon(m)
		setIcon(b, file, l, r, t, bb, id)
		SetItemButtonQuality(b, m.quality)
		SetItemButtonDesaturated(b, m.quality == nil)
		local count
		if colorblind and m.quality then
			-- the colour's numeral, 1 Starter to 7 Artifact as the apps' badges, so quality never rests on colour
			-- alone (a rung no longer names one colour: rung 4 is Common on the squat and Uncommon on the wall sit)
			count = m.quality + 1
		elseif mode == "rungs" or colorblind then
			-- the best step on the one scale (the tooltip's "step 9"; ns.BodyMap's rung, which this read, went with the
			-- one scale on 2026-10-09, and Steps raised on its nil since)
			count = m.ladderStep
		elseif mode == "week" then
			count = m.week
		end
		SetItemButtonCount(b, count and count > 0 and count or nil)
		if (m.exact or 0) > (seen[id] or 0) then -- once for each rise since the muscle was last seen
			b.pulse:Play()
			seen[id] = m.exact
		end
	end
end

-- The stat pane -------------------------------------------------------------------------------------------------------

local function statTooltip(row)
	local key = row.stat
	if not key then return end
	local values = ns.StatValues(ns.state, ns.now())
	local v
	for _, x in ipairs(values) do
		if x.key == key then v = x end
	end
	GameTooltip:SetOwner(row, "ANCHOR_RIGHT")
	local gain = v.value - v.base
	local title = statName(key) .. " " .. v.value
	if gain > 0 then
		title = L["tip.stat.gain"]:format(statName(key), v.value, v.base, GREEN_FONT_COLOR_CODE, gain, FONT_COLOR_CODE_CLOSE)
	end
	GameTooltip_SetTitle(GameTooltip, title, HIGHLIGHT_FONT_COLOR)
	-- Kit.Tip's shape: the body white (gold until 2026-10-16), the last gain a fact in the notes' grey
	GameTooltip:AddLine(L["tip.stat." .. key], 1, 1, 1, true)
	local last = v.last
	if last then
		local what = L["reason." .. last.reason]
		if last.move then what = what .. " · " .. moveName(last.move) end
		local note = Style.saga.note
		GameTooltip:AddLine(L["tip.stat.last"]:format(last.amount, what, shortDate(last.ts)), note[1], note[2], note[3],
			true)
	end
	GameTooltip:Show()
end

local function ratingTooltip(row)
	GameTooltip:SetOwner(row, "ANCHOR_RIGHT")
	GameTooltip_SetTitle(GameTooltip, row.label:GetText(), HIGHLIGHT_FONT_COLOR)
	if row.rating == "crit" then -- the rating behind it, and the window's honest share of wins
		local r = ns.Ratings(ns.state, ns.now())
		GameTooltip:AddLine(L["tip.rating.crit"]:format(r.critRating, C.rating_days, r.wins, r.moves), 1, 1, 1, true)
	else
		local window = row.rating == "expertise" and C.fam_learned_days or C.rating_days
		GameTooltip:AddLine(L["tip.rating." .. row.rating]:format(window), 1, 1, 1, true)
	end
	GameTooltip:Show()
end

-- One category header (Forever's own template, guarded; plain on retail) and its rows, at the sizes of Forever's own
-- stat pane (docs/ui.md, "The character sheet", rule 5: a header 197 x 40, a row 187 x 23, 6 between two categories;
-- 30 and 20 until 2026-10-16).
local function statCategory(parent, y, title)
	local h = CreateFrame("Frame", nil, parent)
	h:SetSize(197, 40)
	h:SetPoint("TOP", 0, y)
	h.bg = h:CreateTexture(nil, "BACKGROUND")
	h.bg:SetAllPoints()
	Style.SetAtlas(h.bg, "statTitle")
	h.label = text(h, "label", "CENTER")
	h.label:SetPoint("CENTER")
	h.label:SetText(title)
	return h
end

local function statRow(parent, y, odd)
	local row = CreateFrame("Frame", nil, parent)
	row:SetSize(187, 23)
	row:SetPoint("TOP", 0, y)
	if odd then
		row.bg = row:CreateTexture(nil, "BACKGROUND")
		row.bg:SetAllPoints()
		if not Style.SetAtlas(row.bg, "statLine") then row.bg:SetColorTexture(1, 1, 1, 0.05) end
	end
	row.label = text(row, "label")
	row.label:SetPoint("LEFT", 8, 0)
	row.value = text(row, "labelWhite", "RIGHT")
	row.value:SetPoint("RIGHT", -8, 0)
	row:EnableMouse(true)
	row:SetScript("OnLeave", function() GameTooltip:Hide() end)
	return row
end

local function buildStats(pane)
	local page = CreateFrame("Frame", nil, pane)
	page.top = 92 -- under the level, the build and the XP bar (renderBody places the bar over it)
	page:SetPoint("TOPLEFT", 0, -page.top)
	page:SetPoint("BOTTOMRIGHT")
	local y = 0
	statCategory(page, y, STAT_CATEGORY_PRIMARY_ATTRIBUTES or L["stat.primary"])
	y = y - 40
	page.stats = {}
	for i, key in ipairs(ns.STATS) do
		local row = statRow(page, y, i % 2 == 1)
		row.stat = key
		row.label:SetText(STAT_FORMAT and STAT_FORMAT:format(statName(key)) or statName(key))
		row:SetScript("OnEnter", statTooltip)
		page.stats[key] = row
		y = y - 23
	end
	y = y - 6
	statCategory(page, y, STAT_CATEGORY_MODIFIERS or L["stat.modifiers"])
	y = y - 40
	page.ratings = {}
	local labels = { crit = CRIT_CHANCE or L["rating.crit"], expertise = STAT_EXPERTISE or L["rating.expertise"],
		defense = DEFENSE or L["rating.defense"], block = STAT_BLOCK or L["rating.block"] }
	for i, key in ipairs({ "crit", "expertise" }) do
		local row = statRow(page, y, i % 2 == 1)
		row.rating = key
		row.label:SetText(labels[key])
		row:SetScript("OnEnter", ratingTooltip)
		page.ratings[key] = row
		y = y - 23
	end
	y = y - 6
	statCategory(page, y, STAT_CATEGORY_DEFENSE or L["stat.defense"])
	y = y - 40
	for i, key in ipairs({ "defense", "block" }) do
		local row = statRow(page, y, i % 2 == 1)
		row.rating = key
		row.label:SetText(labels[key])
		row:SetScript("OnEnter", ratingTooltip)
		page.ratings[key] = row
		y = y - 23
	end
	return page
end

local function renderStats(page)
	for _, v in ipairs(ns.StatValues(ns.state, ns.now())) do
		local row = page.stats[v.key]
		local rose = v.value > v.base
		row.value:SetText(rose and colored(GREEN_FONT_COLOR, tostring(v.value)) or tostring(v.value))
	end
	local r = ns.Ratings(ns.state, ns.now())
	page.ratings.crit.value:SetText(("%.1f%%"):format(r.crit * 100)) -- one decimal, as Forever's own sheet prints it
	page.ratings.expertise.value:SetText(tostring(r.expertise))
	page.ratings.defense.value:SetText(tostring(r.defense))
	page.ratings.block.value:SetText(tostring(r.block))
	-- a hidden-at-0 row as Blizzard's hideAt = 0 does
	page.ratings.block:SetShown(r.block > 0)
end

-- The pages -----------------------------------------------------------------------------------------------------------

-- A list tab's page sits in an inset under the portrait, as the Character window's own tabs do (CharacterFrameInset at
-- y -60: the kit's page tier, Kit.TIER.page's x and y), so its first row never hides behind the portrait (David,
-- 2026-09-30); Body keeps the whole frame under the title bar.
local function page(name, inset)
	local p = CreateFrame("Frame", nil, F.sheet, inset and "InsetFrameTemplate" or nil)
	local t = ns.Kit.TIER.page
	p:SetPoint("TOPLEFT", inset and t.x or 4, inset and t.y or -24)
	p:SetPoint("BOTTOMRIGHT", -4, 4)
	p:Hide()
	Sheet.pages[name] = p
	return p
end

local function buildBody()
	local p = page("body")
	local left = CreateFrame("Frame", nil, p)
	left:SetPoint("TOPLEFT")
	left:SetSize(LEFT_W - 4, HEIGHT - 28)
	left.bg = left:CreateTexture(nil, "BACKGROUND")
	left.bg:SetAllPoints()
	Style.SetAtlas(left.bg, "generalBG")
	for i = 1, 8 do
		muscleButton(left, i):SetPoint("TOPLEFT", 24, -40 - (i - 1) * PITCH)
		local right = muscleButton(left, 8 + i)
		right:SetPoint("TOPRIGHT", -20, -40 - (i - 1) * PITCH)
		right.rightColumn = true -- its flyout opens toward the figure
	end
	for i, id in ipairs({ HEART, BREATH, BALANCE }) do
		muscleButton(left, id):SetPoint("BOTTOM", (i - 2) * PITCH, 12)
	end
	F.bodyPane = left
	-- the stage: the player's own character (the Hero view, and the stage whenever the body map's art is missing)
	local stageW = LEFT_W - 4 - (24 + SLOT + STAGE_GAP) - (20 + SLOT + STAGE_GAP)
	local stageH = HEIGHT - 28 - STAGE_TOP - STAGE_BOTTOM
	local stage = CreateFrame("ModelScene", nil, left, "PanningModelSceneMixinTemplate")
	stage:SetPoint("TOPLEFT", 24 + SLOT + STAGE_GAP, -STAGE_TOP)
	stage:SetSize(stageW, stageH)
	F.stage = stage
	-- the body map (tools/bodymap.py's atlas, Style.bodymap) on the same stage: the character's figure (female or male,
	-- ns.CharBody), front and back, one texture per muscle and view (its shading in the atlas's grey, its colour from
	-- paintBodyMap), the dashed outline of a muscle past 10 moves this week, and its glow (ADD) when trained or under
	-- the cursor
	local art = Style.bodymap and Style.bodymap[ns.CharBody()]
	if art then
		local map = CreateFrame("Frame", nil, left)
		map:SetSize(art.w, art.h)
		map:SetPoint("CENTER", stage, "CENTER", 0, 0)
		map:SetScale(math.min(1, stageW / art.w, stageH / art.h))
		map.cells, map.has = {}, {}
		for i, c in ipairs(art.cells) do
			local kind, id = "fill", c.key
			if c.key == "base" then
				kind, id = "base", nil
			elseif type(c.key) == "string" then
				local letter, n = c.key:match("^([og])(%d+)$")
				kind, id = letter == "o" and "ring" or "glow", tonumber(n)
			end
			local t = map:CreateTexture(nil, kind == "glow" and "OVERLAY" or "ARTWORK", nil,
				kind == "base" and 0 or kind == "fill" and 1 or 2)
			t:SetTexture(art.file)
			t:SetTexCoord(c.l, c.r, c.t, c.b)
			t:SetSize(c.w, c.h)
			t:SetPoint("TOPLEFT", c.x, -c.y)
			if kind == "glow" then t:SetBlendMode("ADD") end
			t.kind, t.muscle = kind, id
			if kind == "fill" then map.has[id] = true end
			map.cells[i] = t
		end
		-- hovering a muscle lights it and its button and shows its tooltip, as hovering the button does; a click opens
		-- its moves (the map samples the cursor only while the mouse is over it)
		map:EnableMouse(true)
		map:SetScript("OnEnter", function(self)
			T.every("mapHover", HOVER_TICK_S, function() Sheet.HoverAt(Sheet.CursorOnMap(self)) end)
			Sheet.HoverAt(Sheet.CursorOnMap(self))
		end)
		map:SetScript("OnLeave", function()
			T.cancel("mapHover")
			Sheet.HoverAt(nil)
		end)
		map:SetScript("OnHide", function()
			T.cancel("mapHover")
			Sheet.HoverAt(nil)
		end)
		map:SetScript("OnMouseUp", function()
			local b = Sheet.hovered and Sheet.muscles[Sheet.hovered]
			if b then showFlyout(b) end
		end)
		map.art = art
		F.bodymap = map
		-- the count of the muscles lit (bottom left) and This week's key (bottom right), beside the bottom row
		p.total = ns.Kit.Text(left, "large", "LEFT")
		p.total:SetPoint("BOTTOMLEFT", 14, 30)
		p.totalLabel = ns.Kit.Text(left, "bodyWhite", "LEFT", Style.saga.note, 124)
		p.totalLabel:SetWordWrap(false)
		p.totalLabel:SetPoint("TOPLEFT", p.total, "BOTTOMLEFT", 0, -2)
		p.totalHit = ns.Kit.WholeWords(p.totalLabel, function()
			return L[ns.profile.mapMode == "rungs" and "sheet.mode.rungs" or "sheet.mode.week"]
		end)
		local legend = CreateFrame("Frame", nil, left)
		-- three keys 40 apart, clear of the bottom row's Balance button (x 258 at most) and of the right column
		legend:SetSize(3 * 40, 14)
		legend:SetPoint("BOTTOMRIGHT", -8, 20)
		local keys = { L["sheet.legend.none"], L["sheet.legend.some"]:format(C.map_steps[1] - 1),
			L["sheet.legend.many"]:format(C.map_steps[1]) }
		for i, key in ipairs(keys) do
			local swatch = legend:CreateTexture(nil, "ARTWORK")
			swatch:SetSize(10, 10)
			swatch:SetPoint("LEFT", (i - 1) * 40, 0)
			swatch:SetColorTexture(unpack(Style.bodyColor[i - 1]))
			local label = ns.Kit.Text(legend, "labelWhite", "LEFT", Style.saga.note) -- a fixed size: the keys stand 40 apart
			label:SetPoint("LEFT", swatch, "RIGHT", 3, 0)
			label:SetText(key)
		end
		legend:EnableMouse(true)
		legend:SetScript("OnEnter", function(self)
			GameTooltip:SetOwner(self, "ANCHOR_TOP")
			GameTooltip_SetTitle(GameTooltip, L["sheet.mode.week"])
			GameTooltip:AddLine(L["sheet.legend.tip"]:format(C.map_steps[2]), 1, 1, 1, true)
			GameTooltip:Show()
		end)
		legend:SetScript("OnLeave", function() GameTooltip:Hide() end)
		p.legend = legend
	end
	local controls = CreateFrame("Frame", nil, stage, "ModelSceneControlFrameTemplate")
	controls:SetPoint("BOTTOM", 0, 4)
	controls:SetModelScene(stage)
	-- the map modes along the top, one switch (the kit's, 2026-10-16; three red buttons until then): This week (moves
	-- this week on each button), Steps (the step's numeral), Hero (the character only); its cell's click plays the tab
	p.modes = ns.Kit.Segments(left, 240, { { "week", L["sheet.mode.week"] }, { "rungs", L["sheet.mode.rungs"] },
		{ "hero", L["sheet.mode.hero"] } }, function() return ns.profile.mapMode or "week" end, function(mode)
		ns.profile.mapMode = mode
		ns.Touch() -- a profile change: stamped, and painted for the mirror
		Sheet.Refresh()
	end)
	p.modes:SetPoint("TOP", 0, -5)
	-- each cell's one line: what the mode shows on the buttons
	for i, tip in ipairs({ { L["sheet.mode.week"], L["sheet.mode.week.tip"] },
		{ L["sheet.mode.rungs"], L["sheet.mode.rungs.tip"] }, { L["sheet.mode.hero"], L["sheet.mode.hero.tip"] } }) do
		ns.Kit.Hover(p.modes.cells[i], ns.Kit.Tip(tip[1], tip[2]))
	end
	-- the right pane, Forever's stat pane: the level on its art with the build and the focus under it, the XP bar and
	-- the stats (the Stats and Titles sidebar went on 2026-10-16: the titles are worn from the Deeds tab)
	local right = CreateFrame("Frame", nil, p)
	right:SetPoint("TOPLEFT", left, "TOPRIGHT", 4, 0)
	right:SetPoint("BOTTOMRIGHT")
	right.bg = right:CreateTexture(nil, "BACKGROUND")
	right.bg:SetAllPoints()
	Style.SetAtlas(right.bg, "statBG")
	right.divider = right:CreateTexture(nil, "ARTWORK")
	right.divider:SetPoint("TOPLEFT", -6, 0)
	right.divider:SetPoint("BOTTOMLEFT", -6, 0)
	right.divider:SetWidth(8)
	Style.SetAtlas(right.divider, "divider")
	-- "QP 12" on Forever's item-level art, as its sheet shows the item level, and the build and the focus under it in
	-- two lines at most, so nothing spills past the art (the one line of 2026-10-03 to 10-16 did)
	right.levelBg = right:CreateTexture(nil, "ARTWORK")
	right.levelBg:SetPoint("TOPLEFT", 5, -8)
	right.levelBg:SetSize(RIGHT_W - 18, 34)
	Style.SetAtlas(right.levelBg, "levelLine")
	right.level = text(right, "levelLine", "CENTER")
	right.level:SetPoint("CENTER", right.levelBg, "CENTER")
	right.build = ns.Kit.Text(right, "bodyWhite", "CENTER", Style.saga.note, RIGHT_W - 20)
	right.build:SetMaxLines(2)
	right.build:SetPoint("TOP", right.levelBg, "BOTTOM", 0, -4)
	-- the level and the build name the focus: one click over both opens the focus window
	right.focus = CreateFrame("Button", nil, right)
	right.focus:SetPoint("TOPLEFT", right.levelBg, "TOPLEFT")
	right.focus:SetWidth(RIGHT_W - 18)
	right.focus:SetScript("OnClick", function() Sheet.ShowFocus(false) end)
	ns.Kit.Hover(right.focus, ns.Kit.Tip(function() return L["focus." .. ns.Focus(ns.state)] end,
		function() return L["focus." .. ns.Focus(ns.state) .. ".line"] end, { hint = L["ui.focus.change"] }))
	-- the XP bar (the kit's: the rested bar under the XP one, the track behind it and the HUD bar's capsule over it, so
	-- an empty bar still shows on the pane), placed under the build's words (renderBody); past the cap its words say
	-- the Paragon rank it fills toward (slice 23)
	local bar = ns.Kit.XPBar(right, RIGHT_W - 32, 13, "sheetXP")
	bar.words = function(level, into, need, paragon)
		return level >= C.level_cap and L["sheet.xp.paragon"]:format(number(into), number(need), paragon + 1)
			or L["sheet.xp"]:format(number(into), number(need))
	end
	bar:EnableMouse(true)
	bar:SetScript("OnEnter", function(self)
		GameTooltip:SetOwner(self, "ANCHOR_RIGHT")
		HUD.QPTooltip(GameTooltip, "sheet")
		GameTooltip:Show()
	end)
	bar:SetScript("OnLeave", function() GameTooltip:Hide() end)
	right.xp = bar
	p.right = right
	p.stats = buildStats(right)
	buildFlyout(p)
end

-- Colours the body map: This week in absolute steps of the moves that trained each muscle (none, under 4, 4 and more,
-- the 10-and-more mark its dashed outline, never a brighter fill) with a glow under the trained ones, Rungs in each
-- muscle's quality colour (untrained ones in the untrained grey). A muscle under the cursor, on the map or on its
-- button, glows in the hover light while the others dim (Sheet.spot).
local function paintBodyMap(mode)
	local colors, glow, spot = Style.bodyColor, Style.bodyGlow, Sheet.spot
	for _, t in ipairs(F.bodymap.cells) do
		local m = t.muscle and Sheet.map[t.muscle]
		local step = m and math.min(m.step, 2) or 0
		if t.kind == "base" then
			t:SetVertexColor(unpack(colors.base))
		elseif t.kind == "fill" then
			local q = mode == "rungs" and m and m.quality
			if q then
				local col = qualityColor(q)
				t:SetVertexColor(col.r, col.g, col.b)
			else
				t:SetVertexColor(unpack(mode == "rungs" and colors.none or colors[step]))
			end
			t:SetAlpha((spot and t.muscle ~= spot) and glow.dim or 1)
		elseif t.kind == "ring" then
			t:SetShown(mode == "week" and m and m.step >= 3)
			t:SetVertexColor(unpack(colors.outline))
			t:SetAlpha((spot and t.muscle ~= spot) and glow.dim or 1)
		elseif spot == t.muscle then
			t:SetVertexColor(unpack(colors.hover))
			t:SetAlpha(glow.hover)
			t:Show()
		else
			local lit = mode == "week" and not spot and step > 0
			if lit then
				t:SetVertexColor(unpack(colors[step]))
				t:SetAlpha(glow[step])
			end
			t:SetShown(lit)
		end
	end
end

-- Lights one muscle on the map (nil: none), from its button or from the map itself; Heart, Breath and Balance have no
-- place on it and light nothing.
function Sheet.Spotlight(id)
	local map = F.bodymap
	if not map then return end
	id = map.has[id] and id or nil
	if Sheet.spot == id then return end
	Sheet.spot = id
	if map:IsShown() then paintBodyMap(ns.profile.mapMode or "week") end
end

-- The cursor on the map's canvas (design units from its top left), or nil while the map has no place yet.
function Sheet.CursorOnMap(map)
	local left, top = map:GetLeft(), map:GetTop()
	if not left or not top then return nil end
	local x, y = GetCursorPosition()
	local scale = map:GetEffectiveScale()
	return x / scale - left, top - y / scale
end

-- The map under the cursor at canvas point x, y (nil: off the map): the muscle there by the hover grid, lit with its
-- button (LockHighlight, as the paper doll lights a slot) and its tooltip, as hovering the button does.
function Sheet.HoverAt(x, y)
	local art = F.bodymap and F.bodymap.art
	local id
	if x and y and art then
		local row = art.hit[math.floor(y / art.hitStep) + 1]
		local c = row and row:byte(math.floor(x / art.hitStep) + 1)
		id = c and c >= 65 and c - 64 or nil
	end
	if id == Sheet.hovered then return end
	local old = Sheet.hovered and Sheet.muscles[Sheet.hovered]
	if old then old:UnlockHighlight() end
	Sheet.hovered = id
	Sheet.Spotlight(id)
	local b = id and Sheet.muscles[id]
	if b then
		b:LockHighlight()
		muscleTooltip(b)
	else
		GameTooltip:Hide()
	end
end

local function renderBody()
	local p = Sheet.pages.body
	renderMuscles()
	local mode = ns.profile.mapMode or "week"
	local hero = mode == "hero" or not F.bodymap
	F.stage:SetShown(hero)
	if F.bodymap then
		F.bodymap:SetShown(not hero)
		if not hero then paintBodyMap(mode) end
		-- how many of the map's muscles this week (or the rungs) lit, and This week's key
		local lit, all = 0, 0
		for id in pairs(F.bodymap.has) do
			local m = Sheet.map[id]
			all = all + 1
			if (mode == "rungs" and m.quality) or (mode ~= "rungs" and m.step > 0) then lit = lit + 1 end
		end
		p.total:SetText(L["sheet.total"]:format(lit, all))
		p.totalLabel:SetText(L[mode == "rungs" and "sheet.total.rungs" or "sheet.total.week"])
		p.total:SetShown(not hero)
		p.totalLabel:SetShown(not hero)
		p.totalHit:SetShown(not hero)
		p.legend:SetShown(mode == "week")
	end
	p.modes:Refresh()
	-- the Hero view: the character sheet's own scene (ModelSceneUtil, scene 595), set up once per showing
	if not Sheet.stageReady and ModelSceneUtil and ModelSceneUtil.SetUpCharacterSheetScene then
		Sheet.stageReady = pcall(ModelSceneUtil.SetUpCharacterSheetScene, F.stage)
	end
	local level, _, _, paragon = ns.Level(ns.state)
	-- the level on the art (its Paragon rank beside it past the cap), and under it the build ("of the Owl", back with
	-- the families that feed the stats, slice 21) with the focus that held its place from 2026-10-03; a click opens the
	-- focus window; the XP bar under them, over the stats at their place
	local right = p.right
	local line = L["sheet.level"]:format(level)
	if paragon > 0 then line = line .. " · " .. L["sheet.paragon"]:format(paragon) end
	right.level:SetText(line)
	right.build:SetText(L["sheet.level.build"]:format(L["build." .. ns.Build(ns.state)],
		L["focus." .. ns.Focus(ns.state)]))
	-- the art 34 high 8 down, the build 4 under it, the bar at 70 or 4 under a build of two lines, never over the stats
	local buildH = right.build:GetStringHeight()
	right.focus:SetHeight(34 + 4 + buildH)
	right.xp.rested:ClearAllPoints()
	right.xp.rested:SetPoint("TOPLEFT", 12, -math.min(p.stats.top - 13 - 1, math.max(70, 8 + 34 + 4 + buildH + 4)))
	-- the bar fills from the XP it last showed (slice 72: a gain is carried home, Kit.XPBar), under a pump's dim only
	-- once the pump has gone (HUD.Covered; its end draws the page again)
	local bar, xp = right.xp, ns.state.sheet.xp
	if not bar.xp then
		bar:SetXP(xp, ns.Rested(ns.state))
	elseif xp ~= bar.xp and not HUD.Covered() then
		bar:FillXP(bar.xp, xp, ns.Rested(ns.state))
	end
	renderStats(p.stats)
end

-- Talents: the ladders as three talent trees, in Tree.lua (slice 37; ns.Tree.Build and ns.Tree.Render), on the page
-- "ladders" keeps.

-- The Log, Records, Today, Deeds and Guild: SheetPages.lua's (ns.Pages), on the pages Sheet.Build hands it.

local RENDER = { body = renderBody, ladders = function() ns.Tree.Render() end,
	today = function() ns.Pages.RenderToday() end, log = function() ns.Pages.RenderLog() end,
	records = function() ns.Pages.RenderRecords() end, deeds = function() ns.Pages.RenderDeeds() end,
	guild = function() ns.Pages.RenderGuild() end }

-- The frame -----------------------------------------------------------------------------------------------------------

local layoutTabs

-- Talents (the ladders' tab, key "ladders") wears the classic talent icon (slice 37), the Log a scroll (2026-10-16; a
-- file icon since vanilla, on retail too)
local TAB_ICON = { body = "Interface\\Icons\\Spell_Nature_Strength", ladders = "Interface\\Icons\\Ability_Marksmanship",
	today = "Interface\\Icons\\INV_Misc_PocketWatch_01", log = "Interface\\Icons\\INV_Scroll_03",
	records = "Interface\\Icons\\INV_Misc_Note_01", deeds = "Interface\\Icons\\INV_Misc_Book_09", guild = Style.guildIcon }

-- An own side tab with a file icon (the sheet's, the Character window's and the Inspect window's: Badges.lua), on
-- SidePanelTabButtonMixin (shared Mainline). Forever's fills its interior with the icon (SetFillToInterior, Gethe
-- forever 70205); retail's (live 12.1.5) has no SetFillToInterior, sizes no file icon (LargeSideTabButtonTemplate's
-- 43 × 55 tab anchors its Icon by the centre) and sets the icon's atlas in SetChecked whatever it is, so there the
-- icon is sized here and checking the tab only shows its selected art.
local RETAIL_TAB_ICON = 32

local function checkRetailTab(self, checked)
	self.SelectedTexture:SetShown(checked)
end

function Sheet.SideTab(parent, icon, tooltip, onClick)
	local tab = CreateFrame("Frame", nil, parent, "LargeSideTabButtonTemplate")
	if tab.SetFillToInterior then
		tab:SetFillToInterior(true)
	else
		tab.Icon:SetSize(RETAIL_TAB_ICON, RETAIL_TAB_ICON)
		tab.Icon:SetTexCoord(0.08, 0.92, 0.08, 0.92) -- the icon file's own border cropped
		tab.SetChecked = checkRetailTab
	end
	tab:SetChecked(false) -- the template's selected art shows until SetChecked hides it
	if icon then tab.Icon:SetTexture(icon) end
	tab.tooltipText = tooltip
	tab:SetCustomOnMouseUpHandler(function(_, button, upInside)
		if button == "LeftButton" and upInside ~= false then onClick() end
	end)
	return tab
end

-- Docking (docs/ui.md, "Docking"): the Character window's own QuestPump tab opens this sheet in its place (its corner),
-- and the portrait tab ahead of Body goes back.
local dock = {}

-- The sheet shows and hides itself, outside Blizzard's UI-panel manager: the manager runs a panel's own OnShow and
-- OnHide inside its call when it pushes the panel aside or closes it, and the addon code in them tainted the window it
-- opened next (the friends list's restricted SearchFriends, 2026-10-05; Popups.lua). Alone it stands at the left
-- panel's place, or beside the Blizzard windows open there, as the manager pushed it aside; a Blizzard window that
-- takes a panel's place later closes it (the ShowUIPanel hook, Sheet.Toggle); Escape closes it (UISpecialFrames).
local function standAlone(sheet)
	sheet:ClearAllPoints()
	-- read, never anchored: the sheet takes the open window's right edge as a point of its own
	local beside = GetUIPanel("doublewide") or GetUIPanel("center") or GetUIPanel("left")
	local right, top = beside and beside:GetRight(), beside and beside:GetTop()
	if right and top then
		local ratio = beside:GetEffectiveScale() / UIParent:GetEffectiveScale()
		sheet:SetPoint("TOPLEFT", UIParent, "BOTTOMLEFT", right * ratio, top * ratio)
	else
		sheet:SetPoint("TOPLEFT", 16, -116)
	end
end

-- The side tabs top down, the portrait tab first while docked, as Blizzard's UpdateTabLayout chains them; Guild only
-- while the character is in a guild, the tabs under it closing the gap (PLAYER_GUILD_UPDATE lays them out again).
function layoutTabs()
	local previous
	local function place(tab)
		tab:ClearAllPoints()
		if previous then
			tab:SetPoint("TOPLEFT", previous, "BOTTOMLEFT", 0, TAB_GAP)
		else
			tab:SetPoint("TOPLEFT", F.sheet, "TOPRIGHT", 0, TAB_Y)
		end
		previous = tab
	end
	F.back:SetShown(dock.on == true)
	if dock.on then
		SetPortraitTexture(F.back.Icon, "player")
		place(F.back)
	end
	for _, key in ipairs(TABS) do
		local tab, shown = Sheet.tabs[key], key ~= "guild" or IsInGuild()
		tab:SetShown(shown)
		if shown then place(tab) end
	end
end

local function setDocked(on)
	dock.on = on or nil
	if F.sheet then layoutTabs() end
end

function Sheet.Build()
	if F.sheet then return end
	local sheet = CreateFrame("Frame", "QuestPumpSheet", UIParent, "PortraitFrameTemplate")
	sheet:SetSize(WIDTH, HEIGHT)
	standAlone(sheet)
	sheet:SetToplevel(true)
	sheet:EnableMouse(true)
	sheet:SetMovable(true)
	sheet:Hide()
	tinsert(UISpecialFrames, "QuestPumpSheet") -- Escape closes it
	sheet:SetScript("OnShow", function()
		if not (dock.switching or Sheet.quiet) then Style.Play("open") end
		Sheet.Refresh()
	end)
	sheet:SetScript("OnHide", function()
		if not (dock.switching or Sheet.quiet) then Style.Play("close") end
		if dock.on and not dock.switching then setDocked(false) end
		if F.flyout then F.flyout:Hide() end
		-- what the muscles showed, so the next opening pulses the ones trained since
		if Sheet.map then
			Sheet.seenWeek = {}
			for id, m in pairs(Sheet.map) do Sheet.seenWeek[id] = m.exact end
		end
		Sheet.stageReady = nil
	end)
	F.sheet = sheet
	buildBody()
	ns.Tree.Build(page("ladders", true))
	ns.Pages.BuildToday(page("today", true))
	ns.Pages.BuildLog(page("log", true))
	ns.Pages.BuildRecords(page("records", true))
	ns.Pages.BuildDeeds(page("deeds", true))
	ns.Pages.BuildGuild(page("guild", true))
	-- while the sheet stands in the Character window's place, the player's portrait ahead of Body goes back to it
	F.back = Sheet.SideTab(sheet, nil, L["sheet.tab.back"], function() Sheet.Undock() end)
	F.back:Hide()
	for _, key in ipairs(TABS) do
		local function select()
			Sheet.tab = key
			Sheet.Refresh()
		end
		local tab = Sheet.SideTab(sheet, TAB_ICON[key], L["sheet.tab." .. key], select)
		tab.select = select
		Sheet.tabs[key] = tab
	end
	layoutTabs()
end

-- The title bar: the character's name in its class colour and the title the player chose.
local function titleText()
	local char = ns.db and ns.db.chars[ns.charIndex]
	local name = (char and char.name) or ns.Readable(UnitName("player")) or "" -- secret when identity is restricted
	local class = char and char.class
	local color = class and C_ClassColor.GetClassColor(class)
	local chosen = ns.ChosenTitle(ns.state, ns.profile)
	if chosen then
		-- "River Sight, Record Breaker", but "River Sight the Relentless" (a title in lower case follows the name)
		local title = L["title." .. chosen]
		name = L[title:find("^%l") and "sheet.titled.suffix" or "sheet.titled"]:format(name, title)
	end
	return color and color:WrapTextInColorCode(name) or name
end
Sheet.TitleText = titleText -- the day card's character line too (Streamer.lua)

function Sheet.Refresh()
	local sheet = F.sheet
	if not (sheet and sheet:IsShown() and ns.state) then return end
	-- Body by default, and in place of Guild once the character left its guild (the tab hides: layoutTabs)
	if not Sheet.tab or (Sheet.tab == "guild" and not IsInGuild()) then Sheet.tab = "body" end
	sheet:SetTitle(titleText())
	if sheet.SetPortraitToUnit then sheet:SetPortraitToUnit("player") end
	for key, p in pairs(Sheet.pages) do p:SetShown(key == Sheet.tab) end
	for key, tab in pairs(Sheet.tabs) do
		if tab.SetChecked then tab:SetChecked(key == Sheet.tab) end
	end
	RENDER[Sheet.tab]()
end

-- Opens or closes the sheet, never before the save is bound; it opens alone at the left panel's place.
function Sheet.Toggle()
	if not ns.db then return end
	Sheet.Build()
	if F.sheet:IsShown() then return F.sheet:Hide() end
	standAlone(F.sheet)
	F.sheet:Show()
end
ns.ToggleSheet = Sheet.Toggle

-- The sheet back on tab `key` after a pump started from it (the Talents tab's return, Tree.Return): shown where it
-- stood, or at its place again when Escape or the game menu closed it, without its open sound (the pump's result
-- sounded already: slice 36's design).
function Sheet.Reopen(key)
	if not ns.db then return end
	Sheet.Build()
	Sheet.tab = key
	if F.sheet:IsShown() then return Sheet.Refresh() end
	standAlone(F.sheet)
	Sheet.quiet = true
	F.sheet:Show()
	Sheet.quiet = nil
end

-- A step's card from outside the sheet (Sheet.ShowStepCard's; the HUD's embers line, slice 55): the sheet opened on the
-- Talents tab where it is closed, at its place as Sheet.Toggle opens it.
function Sheet.OpenStepCard(l, r)
	if not ns.db then return end
	Sheet.Build()
	if F.sheet:IsShown() then return Sheet.ShowStepCard(l, r) end
	Sheet.tab = "ladders"
	standAlone(F.sheet)
	F.sheet:Show()
	ns.Tree.ShowStep(l, r)
end

-- The sheet closed without its close sound, for a pump that asks at once as a pump started from the Talents tab ends
-- (a turn-in's queued behind it: the sheet open over it would hold it back as a panel, Tree.lua).
function Sheet.HideQuiet()
	if not (F.sheet and F.sheet:IsShown()) then return end
	Sheet.quiet = true
	F.sheet:Hide()
	Sheet.quiet = nil
end

-- A Blizzard window that took a panel's place closes the sheet standing there (a post-hook: Blizzard's call has run
-- when it does, and the sheet's OnHide runs outside it); a ShowUIPanel the manager refused, or of a frame it does not
-- place (another addon's own window), leaves it.
hooksecurefunc("ShowUIPanel", function(frame)
	if not (F.sheet and F.sheet:IsShown()) or frame == F.sheet or dock.switching then return end
	-- (a literal list: this file's main chunk runs near Lua 5.1's limit of 200 locals)
	for _, slot in ipairs({ "left", "center", "right", "doublewide", "fullscreen" }) do
		if GetUIPanel(slot) == frame then return F.sheet:Hide() end
	end
end)

-- The sheet in the Character window's place, and back: out of combat only, as the Character window's show and hide
-- (Blizzard's panel manager refuses a tainted one in combat), without the sheet's own open and close sounds (the tab
-- plays). The sheet takes the window's corner, read before the window goes.
function Sheet.Dock()
	if not ns.db then return end
	if inCombat() then return ns.Print("chat.combat") end
	Sheet.Build()
	local sheet = F.sheet
	local ratio = CharacterFrame:GetEffectiveScale() / UIParent:GetEffectiveScale()
	local left, top = CharacterFrame:GetLeft(), CharacterFrame:GetTop()
	dock.switching = true
	setDocked(true)
	HideUIPanel(CharacterFrame)
	-- a window the manager slid into the left place as the Character window went keeps it: the sheet stands beside it
	if left and top and not GetUIPanel("left") then
		sheet:ClearAllPoints()
		sheet:SetPoint("TOPLEFT", UIParent, "BOTTOMLEFT", left * ratio, top * ratio)
	else
		standAlone(sheet)
	end
	sheet:Show()
	dock.switching = nil
end

function Sheet.Undock()
	if inCombat() then return ns.Print("chat.combat") end
	dock.switching = true
	F.sheet:Hide()
	setDocked(false)
	ShowUIPanel(CharacterFrame)
	dock.switching = nil
end

-- The Character window's QuestPump tab under Forever's own side tabs (an own frame on their column, never one of
-- Blizzard's tabs, whose array its selection and gamepad code walk), placed under the last one shown when the window
-- opens (UpdateTabLayout runs once at load on 70124; it is hooked for a later build), and the "QP 12" plate under the
-- character model on the paper doll. Retail's Character window has no side tabs: the compartment opens the sheet.
local function placeDockTab()
	local last
	for _, tab in ipairs(CharacterFrame.ModeTabs.Tabs or {}) do
		if tab:IsShown() then last = tab end
	end
	F.dockTab:ClearAllPoints()
	if last then
		F.dockTab:SetPoint("TOPLEFT", last, "BOTTOMLEFT", 0, TAB_GAP)
	else
		F.dockTab:SetPoint("TOPLEFT", CharacterFrame.ModeTabs, "TOPLEFT")
	end
end

local function renderPlate()
	local plate = F.plate
	if not plate then return end
	plate:SetShown(qpShown())
	if plate:IsShown() then plate.text:SetText(L["sheet.plate"]:format((ns.Level(ns.state)))) end
end

local function buildDock()
	if F.dockTab or not (CharacterFrame and CharacterFrame.ModeTabs and CharacterFrame.UpdateTabLayout) then return end
	local tab = Sheet.SideTab(CharacterFrame.ModeTabs, Style.addonIcon, L["sheet.tab.dock"], function() Sheet.Dock() end)
	F.dockTab = tab
	placeDockTab()
	hooksecurefunc(CharacterFrame, "UpdateTabLayout", placeDockTab)
	-- the Character window's own key while the sheet stands in its place: the sheet leaves
	EventRegistry:RegisterCallback("CharacterFrame.Show", function()
		placeDockTab()
		if dock.on and F.sheet and F.sheet:IsShown() then F.sheet:Hide() end
	end, dock)
	if not (PaperDollFrame and CharacterSecondaryHandSlot) then return end
	local plate = CreateFrame("Button", nil, PaperDollFrame)
	plate:SetSize(90, 22)
	plate:SetPoint("BOTTOM", CharacterSecondaryHandSlot, "TOP", 0, 14)
	plate:SetFrameLevel(CharacterSecondaryHandSlot:GetFrameLevel())
	plate.bg = plate:CreateTexture(nil, "BACKGROUND")
	plate.bg:SetAllPoints()
	Style.SetAtlas(plate.bg, "levelLine")
	plate.text = text(plate, "levelLine", "CENTER")
	plate.text:SetPoint("CENTER")
	plate:SetScript("OnClick", function() Sheet.Dock() end)
	plate:SetScript("OnEnter", function(self)
		GameTooltip:SetOwner(self, "ANCHOR_RIGHT")
		HUD.QPTooltip(GameTooltip, "plate")
		GameTooltip:Show()
	end)
	plate:SetScript("OnLeave", function() GameTooltip:Hide() end)
	F.plate = plate
	EventRegistry:RegisterCallback("PaperDollFrame.VisibilityUpdated", function(_, shown)
		if shown then renderPlate() end
	end, dock)
	ns.On("Sheet", renderPlate) -- a QP level gained while the paper doll shows
end

for _, message in ipairs({ "Row", "DB", "Sheet", "Clock", "Settings" }) do ns.On(message, Sheet.Refresh) end
-- a Text Size, UI scale or screen change lays out again what shows: the HUD's lines, the sheet's page (the Talents
-- tab's step card with it), the muscle flyout and the focus window
ns.Kit.OnRelayout(function()
	HUD.Render()
	Sheet.Refresh()
	local fly = F.flyout
	if fly and fly:IsShown() and fly.button then renderFlyout(fly, fly.button) end
	if F.focus and F.focus:IsShown() then Sheet.ShowFocus(F.focus.asking) end
end)
-- a card that came or the guild's week: only the Guild tab reads them
ns.On("Social", function() if Sheet.tab == "guild" then Sheet.Refresh() end end)
-- joining or leaving a guild shows or hides the Guild tab (the event also fires before PLAYER_LOGIN, before any sheet)
ns.RegisterEvent("PLAYER_GUILD_UPDATE", function()
	if not F.sheet then return end
	layoutTabs()
	Sheet.Refresh()
end)

-- The keybinding and the slash command.
BINDING_NAME_QUESTPUMP_SHEET = L["binding.sheet"]

function QuestPump_ToggleSheet()
	Sheet.Toggle()
end

ns.AddCommand("stats", function() Sheet.Toggle() end, "cmd.stats.help")

-- Login ---------------------------------------------------------------------------------------------------------------

ns.RegisterEvent("PLAYER_LOGIN", function()
	build()
	buildDock()
	if not attachEditMode(HUD.frame) then dragHandle(HUD.frame) end
	placeHUD(HUD.frame)
	if AlertFrame and AlertFrame.AddQueuedAlertFrameSubSystem then
		local ok, sub = pcall(AlertFrame.AddQueuedAlertFrameSubSystem, AlertFrame, "QuestPumpToastTemplate", setUpToast,
			3, 6)
		toaster = ok and sub or nil
		-- the Legendary toast's own frame, where the client has its art (the plain toast stands in otherwise)
		if C_Texture.GetAtlasInfo(Style.atlas.legendBG) then
			ok, sub = pcall(AlertFrame.AddQueuedAlertFrameSubSystem, AlertFrame, "QuestPumpLegendToastTemplate",
				setUpToast, 1, 3)
			legendToaster = ok and sub or nil
		end
	end
	HUD.Render()
	T.after("milestone", 3, Sheet.MirroredMilestone)
end)

ns.RegisterEvent("PLAYER_REGEN_DISABLED", function()
	HUD.Render()
	-- a fight ends the banner; the wins it had no row for and the fanfare it held wait for the fight's end (release)
	local f = F.banner
	if f and f:IsShown() then
		for i = C.banner_rows + 1, #(f.wins or {}) do held[#held + 1] = f.wins[i].h end
	end
	holdBack()
	Sheet.BannerHide()
	if F.fanfare then F.fanfare:Hide() end
	if F.flyout then F.flyout:Hide() end
end)
ns.RegisterEvent("PLAYER_REGEN_ENABLED", function()
	HUD.Render()
	release(Lc.pump) -- what a fight held, unless a pump's dim came back with its end
end)
