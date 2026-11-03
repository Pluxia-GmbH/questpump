-- The character sheet's list pages (docs/plan.md slice 3; docs/ui.md, "The character sheet"): the Log, Records, Today,
-- Deeds and Guild, out of Sheet.lua since 2026-10-16 (its main chunk held as many locals as Lua 5.1 allows, and each
-- page now changes beside the others). Sheet.Build hands each builder its page in the inset under the portrait, and
-- Sheet.Refresh draws the tab that shows through its renderer; the formatting helpers are Sheet.lua's exported ones.
-- Every page is built from the kit (Kit.lua, 2026-10-16: a page's head and scroll, sections, framed lists of two-line
-- rows, stats, cards, the empty state), Today's day body too (Streamer.BuildDayBody, the day card's). Every number
-- comes from the fold (Stats.lua): nothing here counts anything. It says unfinished, never owed or debt.

local ADDON, ns = ...
local C, L = ns.C, ns.L
local O = ns.Enum.outcome
local Lc, Style, Kit = ns.Lifecycle, ns.Style, ns.Kit
local Sheet, HUD = ns.Sheet, ns.HUD
local PAGE = Kit.TIER.page

local Pages = {}
ns.Pages = Pages

-- Sheet.lua's helpers and measures, under the names the pages were written with
local colored, number = Sheet.Colored, Sheet.Number
local moveName, moveAmount, qualityColor = Sheet.MoveName, Sheet.MoveAmount, Sheet.QualityColor
local clockTime, fullDate = Sheet.ClockTime, Sheet.FullDate
local owedLine = HUD.OwedLine

-- The pages on the kit -----------------------------------------------------------------------------------------------

-- A list page on the kit (2026-10-16): its head (the page's name at the left, its line at the right, the rule under
-- them; p.intro takes the line's words) and the scroll list under it to the page's foot, or `foot` units over it (a
-- page with a bar of its own); its content, p.body, is where Kit.Stack and Kit.Section place the page's blocks.
local function listPage(p, title, foot)
	Kit.Head(p, title, nil, PAGE)
	local top = Kit.Top(p)
	p.scroll = Kit.Scroll(p, PAGE.scrollW, PAGE.H - PAGE.BOTTOM - (foot or 0) - top)
	p.scroll:SetPoint("TOPLEFT", PAGE.IN, -top)
	p.body = p.scroll.content
	p.body.inset, p.body.contentW = 0, PAGE.scrollW
	return p.body
end

-- A framed list across a page's body, its rows' line under the name.
local function pageList(body)
	return Kit.List(body, PAGE.scrollW, PAGE.rowName)
end

-- Row `i` of a framed list, two lines (Kit.Row's `below`), built the first time from make()'s options (and its
-- `ready(row)`, which finishes the row once built) and drawn again by each render: its tooltip and its clicks are the
-- render's (row.tooltip, row.click, row.onRight; a list whose rows act at a click asks for it with `onClick`, one that
-- takes right-clicks with `rightClick`, so a row that does nothing at a click never lights up under the cursor:
-- Kit.Row), its picture's screen (`picture = true`) the render's to fill (rowPicture). kitRows hides the rows a render
-- no longer needs.
local function kitRow(list, i, make)
	local row = list.rows[i]
	if not row then
		local opts, ready = make()
		opts.below = true
		opts.tip = function(tooltip) if row.tooltip then row.tooltip(tooltip) end end
		if opts.onClick then opts.onClick = function() if row.click then row.click(row) end end end
		if opts.rightClick then opts.rightClick = function() if row.onRight then row.onRight(row) end end end
		row = Kit.Row(list, opts)
		if ready then ready(row) end
	end
	row:Show()
	return row
end

local function kitRows(list, from)
	for i = from, #list.rows do list.rows[i]:Hide() end
end

-- A pooled row's picture: a move's key pose at 40 in its screen, the screen edged in `quality`'s colour where given
-- (Kit.PaintScreen; the word goes on the row beside it), or an atlas at its own shape in 24 (Kit.ChipArt, QuestPump's
-- mark where the client lacks it: a title's shield), or QuestPump's mark in 24 without either (a pump with no move).
local function rowPicture(row, move, quality, art)
	local t = row.icons[1]
	if art then
		Kit.ChipArt(t, art, true)
	elseif not move then
		t:SetTexture(Style.mark)
		t:SetTexCoord(0, 1, 0, 1)
		t:SetSize(Kit.CHIP_ICON, Kit.CHIP_ICON)
		t:Show()
	else
		t:SetSize(40, 40)
		Kit.SetIcon(t, move)
		t:Show()
	end
	Kit.PaintScreen(row.screen, quality)
end

-- The Log -------------------------------------------------------------------------------------------------------------

-- A quest's title the Log showed as its number until the client loaded it (Lc.QuestTitle): the sheet draws again.
ns.On("QuestTitle", function() Sheet.Refresh() end)

-- The tooltip's line for a pump's heart-rate proof (ns.Proof), nil without one.
local function proofLine(proof)
	if not proof then return nil end
	if proof.verdict == "proven" then
		if tonumber(proof.peak) and tonumber(proof.delta) then
			return L["log.hr.proven"]:format(proof.peak, proof.delta)
		end
		return L["log.hr.counted"]
	end
	return proof.verdict == "unproven" and L["log.hr.unproven"] or L["log.hr.none"]
end

-- The Log (a tab of its own since 2026-10-16, the Body page's right pane until then: docs/ui.md, "The right pane";
-- David, 2026-10-05: "all these green checkmarks are super confusing. Improve the UI and UX here."): the pumps by game
-- day, newest first, under Today, Yesterday or the weekday and date, each day a framed list. A row is one pump, on
-- two lines: its picture (its first done move's, dimmed where none was done), its whole title and under it its moves'
-- names (cut with an ellipsis), and at the right its time and what it came to in numbers or one word; its tooltip
-- names each move with its result.

-- A Log move's outcome; an entry folded before 2026-10-05 kept only whether the move was done.
local function logOut(mv)
	return mv.out or (mv.done and O.done or O.owed)
end

local function doneOut(out)
	return out ~= O.owed and out ~= O.banked
end

-- Whether a Log move is a new record: stored as one and named by no correction (Stats.lua doneMove's `corrected`;
-- contract/fixtures/history.json `saga` rule 12, as the Saga's day and the apps' Logs count one).
local function logRecord(mv)
	return logOut(mv) == O.beat and not mv.corrected
end

-- A game day's heading: Today and Yesterday by the game day (from 04:00, as the Today tab counts), else its weekday
-- and date ("Saturday, 3 Oct").
local function dayHeading(ts, now)
	local key = ns.day(ts)
	if key == ns.day(now) then return L["log.today"] end
	if key == ns.day(ns.DayMid(now, -1)) then return L["log.yesterday"] end
	local mid = ns.DayMid(ts)
	return L["log.date"]:format(date("%A", mid), date("*t", mid).day, date("%b", mid))
end

-- What a pump came to, for the row's right column, and whether it is one quiet word: "In progress" while it runs;
-- then the moves done of its moves ("3/3", "2/3": white, never a warning's orange since 2026-10-16), after a star for a
-- new record (logRecord) and the heart for heart-rate proof; with none done, one word in the notes' grey for what
-- became of them: Banked while one waits (a move left open by a fight or a pause too, since 2026-11-03, Rapha: "we
-- always bank them"), else Done later or Skipped; "Nothing done" without a move.
local function logSummary(e, proof, live)
	if live then return L["log.sum.live"], false end
	local done, record, open, later = 0, false, false, false
	for _, mv in ipairs(e.moves) do
		local out = logOut(mv)
		if doneOut(out) then
			done = done + 1
			record = record or logRecord(mv)
		else
			later = later or mv.fate == "later"
			open = open or mv.fate == nil
		end
	end
	if #e.moves == 0 then return L["log.sum.none"], true end
	if done == 0 then
		return open and L["log.sum.banked"] or later and L["log.sum.later"] or L["log.sum.skipped"], true
	end
	local heart = proof and proof.verdict == "proven"
	return (record and Style.Stars(1) .. " " or "") .. (heart and Style.Heart() .. " " or "")
		.. L["log.sum.count"]:format(done, #e.moves), false
end

-- A Log move's result for its tooltip line, with its colour ({ r, g, b }): the count or the seconds and what it won
-- ("12 reps · new record", green), the words of the button the player pressed for a rep or hold set without a count,
-- a counted set's count in its window or "Not counted" with "form slipped" after a set saved so (slice 42's review,
-- 2026-10-17: the Result's words, where the Log read "0 in 30 s" and a slipped count as a clean one), or what the
-- pump did with a move it left open and what became of it since ("Banked · done later"), the words of a move not done
-- in the notes' grey (Kit.Tip's colours, 2026-10-16).
local WHITE = { 1, 1, 1 }

local function moveResult(mv)
	local out = logOut(mv)
	if not doneOut(out) then
		local word = L["log.sum.banked"]
		if not mv.fate then return word, Style.saga.note end
		local fate = mv.fate == "later" and L["log.fate.later"] or L["log.fate.skipped"]
		return L["log.result.with"]:format(word, fate), mv.fate == "later" and WHITE or Style.saga.note
	end
	if not mv.out then return L["log.result.done"], WHITE end
	local m = ns.Library[mv.move]
	if not mv.v and (m.dose == "rep" or m.dose == "hold") then
		-- a rep or hold set saved without a count: Done with an empty box or Escape over it (Lc.EmptyDone) and a typed 0
		-- (ns.Classify) save it done; "Could not do one" / "Could not hold it" (Lc.CouldNot) a miss or a placement of 0:
		-- the Result view's own words
		return out == O.done and L["ui.result.empty"]
			or L[m.dose == "hold" and "ui.result.couldNotHold" or "ui.result.couldNot"], WHITE
	end
	-- a timed move with no seconds (stopped at once) reads 0 s with its note
	local v = mv.v or 0
	local amount = m.dose == "rep" and ns.Plural("log.reps", v):format(v)
		-- a counted set's count (Stats.lua doneMove), or the Result's words for one saved with none
		or mv.count and (mv.v and L["unit.countIn"]:format(v, C.count_s) or L["ui.result.cleanNone"])
		or L["unit.seconds"]:format(v)
	-- a set done with a weight reads as every surface writes one ("11 kg × 16"; the load row Stats.lua folds into
	-- mv.load)
	if mv.load and m.dose == "rep" then amount = Sheet.LoadAmount(mv.move, mv.load, v) end
	if out == O.censored then return L["log.result.atLeast"]:format(amount), WHITE end
	-- a corrected record reads as its corrected count alone (logRecord)
	local record = logRecord(mv)
	local note = mv.slipped and L["log.result.slipped"] or mv.notAllOut and L["log.result.notAllOut"]
		or record and L["outcome.beat"]
		or out == O.lastbeat and L["outcome.lastbeat"] or out == O.placement and L["log.result.max"]
		or out == O.stopped and L["outcome.stopped"]
	return note and L["log.result.with"]:format(amount, note) or amount,
		record and { GREEN_FONT_COLOR:GetRGB() } or WHITE
end

-- A catch-up's or an app pump's entry (slice 26: Stats.lua logEntry keeps its source and device): its title, and the
-- device it was done on for its tooltip.
local SOURCE_TITLE = { [ns.Enum.source.catchup] = "log.catchup", [ns.Enum.source.app] = "log.app" }
local DEVICE_LINE = { [ns.Enum.device.phone] = "log.device.phone", [ns.Enum.device.watch] = "log.device.watch",
	[ns.Enum.device.computer] = "log.device.computer" }

-- The Log's own tab (2026-10-16; a page of the Body tab's sidebar until then: sheet SPEC 5): the last pumps by game
-- day, each day under its section with the day's moves and pumps at its right (the Saga's count, ns.DayCount: never a
-- sum of the rows shown, which may cut a day), then one row a pump: the key pose of its first move done (its first
-- move, dimmed, when none was), its title in full, its moves' names under it, its time and what it came to.
local LOG_TIME_X = 404 -- the time's column from the row's left; the result right-aligned after it

-- A pump's time and result, the row's control: the time in the notes' grey, the result (large) at the right.
local function logState(parent)
	local ctl = CreateFrame("Frame", nil, parent)
	ctl:SetSize(PAGE.scrollW - 6 - 12 - LOG_TIME_X, 24)
	ctl.time = Kit.Text(ctl, "bodyWhite", "LEFT", Style.saga.note)
	ctl.time:SetPoint("LEFT")
	ctl.result = Kit.Text(ctl, "large", "RIGHT")
	ctl.result:SetPoint("RIGHT")
	return ctl
end

function Pages.BuildLog(p)
	local body = listPage(p, L["sheet.tab.log"])
	p.days = {}
	p.empty = Kit.Empty(body, PAGE.scrollW, { name = L["log.empty"] })
end

-- A pump's title: a death tax, a level-up, its quest's whatever its source (a number until the client loads its
-- title; another computer's quest pump comes as an app row, slice 60), a catch-up or an app pump, or one asked for.
local function logTitle(e)
	return e.death and L["log.death"] or e.ding and L["log.ding"]
		or e.questID > 0 and (Lc.QuestTitle(e.questID) or L["log.quest"]:format(e.questID))
		or SOURCE_TITLE[e.source] and L[SOURCE_TITLE[e.source]] or L["log.asked"]
end

-- A pump's tooltip: its title, each move with its result, where it was done, the level it reached, its heart rate.
local function logTooltip(e, title, live, proof)
	local note = Style.saga.note
	return function(tooltip)
		GameTooltip_SetTitle(tooltip, title)
		if live then
			tooltip:AddLine(L["log.tip.live"], note[1], note[2], note[3])
		elseif #e.moves == 0 then
			tooltip:AddLine(L["log.tip.none"], note[1], note[2], note[3])
		end
		for _, mv in ipairs(e.moves) do
			local result, color = moveResult(mv)
			tooltip:AddDoubleLine(moveName(mv.move), result, 1, 1, 1, color[1], color[2], color[3])
		end
		-- where a catch-up or an app pump was done (an app pump counts as any pump since slice 27, Stats.lua); a quest's
		-- pump is the game's wherever it was played
		if e.questID == 0 and DEVICE_LINE[e.device] then
			tooltip:AddLine(L[DEVICE_LINE[e.device]], note[1], note[2], note[3])
		end
		-- the QP level or the Paragon rank the pump reached (Stats.lua gainXP)
		if e.level then tooltip:AddLine(L["log.level"]:format(e.level), note[1], note[2], note[3]) end
		if e.rank then tooltip:AddLine(L["log.paragon"]:format(e.rank), note[1], note[2], note[3]) end
		local heart = proofLine(proof)
		if heart then tooltip:AddLine(heart, note[1], note[2], note[3]) end
	end
end

function Pages.RenderLog()
	local p = Sheet.pages.log
	local state, log = ns.state, ns.state.sheet.log
	local now, note = ns.now(), Style.saga.note
	-- the pumps by game day, newest first; the pump in progress is the newest entry of the second it opened (its entry
	-- is written as it opens, Events.lua writeOffered, before a move of it is done; Stats.lua entryAt finds it so)
	local liveTs = Lc.pump and Lc.pump.offeredTs
	local days = {}
	for i = #log, 1, -1 do
		local e = log[i]
		local live = e.ts == liveTs
		if live then liveTs = nil end
		local heading = dayHeading(e.ts, now)
		local d = days[#days]
		if not (d and d.heading == heading) then
			d = { heading = heading, ts = e.ts, pumps = {} }
			days[#days + 1] = d
		end
		d.pumps[#d.pumps + 1] = { e = e, live = live }
	end
	local parts = {}
	for i, d in ipairs(days) do
		local sec = p.days[i] or { head = Kit.Section(p.body, "", false, ""), list = pageList(p.body) }
		p.days[i] = sec
		Kit.SetCaps(sec.head.words, d.heading)
		local c = ns.DayCount(state, d.ts)
		local aside = ns.Plural("log.day.moves", c.moves):format(c.moves)
		if c.pumps then aside = aside .. " · " .. ns.Plural("log.day.pumps", c.pumps):format(c.pumps) end
		sec.head.aside:SetText(aside)
		for j, pump in ipairs(d.pumps) do
			local e, live = pump.e, pump.live
			local row = kitRow(sec.list, j, function()
				return { picture = true, line = "", control = logState(sec.list) },
					function(r) r.line:SetWordWrap(false) end -- one line, cut with an ellipsis
			end)
			local title, proof = logTitle(e), ns.Proof(e.ts)
			local names, first = {}, nil
			for _, mv in ipairs(e.moves) do
				names[#names + 1] = moveName(mv.move)
				if not first and doneOut(logOut(mv)) then first = mv.move end
			end
			row.name:SetText(title)
			row.line:SetText(table.concat(names, " · "))
			rowPicture(row, first or (e.moves[1] and e.moves[1].move))
			-- the picture dimmed when no move was done, the title white either way (G4)
			Kit.Choose(row, first ~= nil or live, true)
			row.control.time:SetText(clockTime(e.ts))
			local summary, quiet = logSummary(e, proof, live)
			row.control.result:SetText(summary)
			if quiet then
				row.control.result:SetTextColor(note[1], note[2], note[3])
			else
				row.control.result:SetTextColor(1, 1, 1)
			end
			row.tooltip = logTooltip(e, title, live, proof)
		end
		kitRows(sec.list, #d.pumps + 1)
		parts[#parts + 1] = { sec.head, i == 1 and 0 or 16, show = true }
		parts[#parts + 1] = { sec.list, 14, Kit.LayList(sec.list), show = true }
	end
	-- the days no longer shown
	for i = #days + 1, #p.days do
		p.days[i].head:Hide()
		p.days[i].list:Hide()
	end
	parts[#parts + 1] = { p.empty, 40, show = #days == 0 }
	p.body:SetHeight(math.max(1, Kit.Stack(p.body, 0, parts) + 8))
end

-- Records, Today, Deeds and Guild -------------------------------------------------------------------------------------

-- Records (2026-10-16 on the kit: sheet SPEC 7): the player's bests as one stats row (the longest streak, the most
-- moves in a day, the longest hold: the Saga's best day, ns.DayRank, every source's moves), then each ladder's
-- records in the trees' order, Not on a ladder last: a row a move with its key pose edged in its step's colour, its
-- step and quality, and its record at the right with its stars (a strength move's) or its weight ("10 kg × 16"); its
-- tooltip the move's with the record's span, its click opening its card on Talents (Sheet.ShowMoveCard), its
-- right-click its record menu (slice 44: Correct a count's window, or the record cleared, which Settings.lua asks
-- first). The weekly hard-set counts went (the muscles' tooltips and Today's safety line say them).
local BESTS_H = 52

function Pages.BuildRecords(p)
	local body = listPage(p, L["sheet.tab.records"])
	p.bests = CreateFrame("Frame", nil, body)
	p.bests:SetSize(PAGE.scrollW, BESTS_H)
	local w = PAGE.scrollW / 3
	p.best = {}
	for i, key in ipairs({ "streak", "mostMoves", "hold" }) do
		local cell = Kit.Stat(p.bests, { rule = i > 1 })
		cell:SetSize(w, BESTS_H)
		cell:SetPoint("TOPLEFT", (i - 1) * w, 0)
		Kit.Hover(cell, function(tooltip) if cell.tooltip then cell.tooltip(tooltip) end end)
		p.best[key] = cell
	end
	p.bestsRule = Kit.Rule(body, Style.saga.rule)
	p.bestsRule:SetWidth(PAGE.scrollW)
	p.ladders = {}
	p.empty = Kit.Empty(body, PAGE.scrollW, { name = L["records.empty"] })
end

-- The bests: each cell's number and word, and its tooltip.
local function renderBests(p, state)
	local bestHold, bestHoldMove = 0, nil
	for id, rec in pairs(state.moves) do
		local m = ns.Library[id]
		if m and m.dose == "hold" and rec.best and (rec.best > bestHold or (rec.best == bestHold and id < bestHoldMove)) then
			bestHold, bestHoldMove = rec.best, id
		end
	end
	local streak = ns.Streak(state, ns.now())
	local best = p.best
	Kit.FillStat(best.streak, number(state.sheet.best), L["records.streak"])
	best.streak.tooltip = Kit.Tip(L["records.streak"], ns.Plural("records.streak.now", streak):format(streak))
	-- the best day in moves and the day that set it: the Saga's count (ns.DayRank), every source's moves since
	-- 2026-10-15 (the game's rows alone until then), today's with them
	local top = ns.DayRank(state).top
	Kit.FillStat(best.mostMoves, number(top and top.moves or 0), L["records.mostMoves"])
	best.mostMoves.tooltip = Kit.Tip(L["records.mostMoves"], top
		and L["records.mostMoves.value"]:format(BreakUpLargeNumbers(top.moves), Sheet.DayDate(top.day)) or nil)
	Kit.FillStat(best.hold, bestHoldMove and L["unit.seconds"]:format(bestHold) or "0", L["records.hold"])
	best.hold.tooltip = Kit.Tip(L["records.hold"], bestHoldMove
		and L["records.holdValue"]:format(bestHold, moveName(bestHoldMove)) or nil)
end

-- A recorded move's row: its picture, its step and quality, its record and stars or weight, its tooltip, its click (the
-- move's card on the Talents tab, as the flyout's) and its right-click: the move's record menu, "Correct a count..."
-- and "Clear this move's record" (Settings.lua Settings_.RecordMenu, slice 44).
local function recordRow(row, id)
	local state, m = ns.state, ns.Library[id]
	local step, q = ns.MoveStep(id), ns.MoveQuality(id)
	row.name:SetText(moveName(id))
	row.line:SetText(step and L["records.step"]:format(step, Sheet.QualityWord(q)) or "")
	row.line:SetShown(step ~= nil)
	rowPicture(row, id, q)
	Kit.Choose(row, true, true)
	local best = ns.Record(state, id)
	local topLoad, topReps = ns.LoadRecord(state, id) -- a loaded move's weighted record after its counts
	local value = topLoad and Sheet.LoadAmount(id, topLoad, topReps) or moveAmount(id, best)
	if not topLoad and m.ladder and ns.STRENGTH[m.ladder] then
		value = value .. " " .. Style.Stars(ns.Stars(m.ladder, m.rung, id, best), 3)
	end
	row.value:SetText(value)
	row.tooltip = function(tooltip)
		Sheet.MoveTooltip(tooltip, id, nil, { span = true, record = true, click = "talents" })
	end
	row.click = function() Sheet.ShowMoveCard(id) end
	row.onRight = function(r) ns.Settings.RecordMenu(r, id) end
end

function Pages.RenderRecords()
	local p, state = Sheet.pages.records, ns.state
	renderBests(p, state)
	-- each record by its ladder in the trees' order, Not on a ladder (0) last
	local groups, count = {}, 0
	for _, group in ipairs(ns.RecordedMoves(state)) do
		groups[group.ladder] = group.moves
		count = count + #group.moves
	end
	p.intro:SetText(ns.Plural("records.count", count):format(count))
	local order = Sheet.TreeOrder()
	order[#order + 1] = 0
	local parts = { { p.bests, 0, BESTS_H }, { p.bestsRule, 0, 1 } }
	local n = 0
	for _, l in ipairs(order) do
		local ids = groups[l]
		if ids then
			n = n + 1
			local sec = p.ladders[n]
			if not sec then
				sec = { head = Kit.Section(p.body, ""), list = pageList(p.body) }
				p.ladders[n] = sec
			end
			Kit.SetCaps(sec.head.words, l > 0 and L["ladder." .. l] or L["records.other"])
			for i, id in ipairs(ids) do
				local row = kitRow(sec.list, i, function()
					return { picture = true, line = "", value = "", onClick = true, rightClick = true },
						function(r) Style.SetFont(r.value, "large") end
				end)
				recordRow(row, id)
			end
			kitRows(sec.list, #ids + 1)
			parts[#parts + 1] = { sec.head, 16, show = true }
			parts[#parts + 1] = { sec.list, 14, Kit.LayList(sec.list), show = true }
		end
	end
	for i = n + 1, #p.ladders do
		p.ladders[i].head:Hide()
		p.ladders[i].list:Hide()
	end
	parts[#parts + 1] = { p.empty, 40, show = n == 0 }
	p.body:SetHeight(math.max(1, Kit.Stack(p.body, 0, parts) + 8))
end

-- Today (sheet SPEC 6; the day card in the sheet since 2026-10-16): what the day did and won. The page's head says
-- "Today" with the date and the QP level on its row; a scroll list (Kit.Scroll) holds, top down, the unfinished moves
-- as one row that acts, only while some wait (the HUD's unfinished line: its words, its tooltip, its click and menus),
-- then the day's body (Streamer.BuildDayBody, the day card's) in the tab's order: the hero (moves first, the player's
-- own count: David, 2026-10-04), the stats row, the records and unlocks above the fold, the strip, the moves by region
-- with the hard-set note under them (safety.md row 10, in the notes' grey, never red), and the week's vault as three
-- cards; Screenshot and Share line stay pinned under a hairline. No pump yet today: the hero reads 0, the stats row
-- stays, and the empty state takes the place of the wins, the strip and the regions. The numbers are ns.DayTally's
-- and ns.Today's: the moves, pumps and records every source's, the app's share in the tooltips (slice 26), the wins,
-- stars and XP the game's sheet's.
local VAULT_H = 76 -- a vault card's least height

-- A vault card's words and bar laid out: the check before a done goal's gold name, the count toward the next tier at
-- the right ("9/15"), a goal of tiers' pips under its name (the embers' three, slice 55), the bar along the foot; as
-- tall as its words need.
local function layVault(card, v)
	local w = card:GetWidth()
	Kit.Choose(card, v.done, not v.done) -- done: the gold edge, light and name; else the plain card, its name white
	card.mark:SetShown(v.done)
	card.name:SetText(L["vault.label." .. v.goal])
	card.count:SetText(L["deeds.progress.count"]:format(v.value, v.need))
	local x = v.done and 12 + 16 + 6 or 12
	card.name:ClearAllPoints()
	card.name:SetPoint("TOPLEFT", x, -14)
	card.name:SetWidth(math.max(1, w - x - 12 - card.count:GetStringWidth() - 8))
	card.pips:SetShown(v.tiers > 1)
	card.pips:SetText(v.tiers > 1 and Style.Pips(v.paid, v.tiers) or "")
	card.bar:Set(v.value, v.need)
	local words = math.max(card.name:GetStringHeight(), card.count:GetStringHeight())
	if v.tiers > 1 then words = math.max(words, card.name:GetStringHeight() + 4 + 12) end
	return math.max(VAULT_H, 14 + words + 18 + 6 + 16)
end

function Pages.BuildToday(p)
	local saga = Style.saga
	local t = Kit.TIER.page
	Kit.Head(p, L["sheet.tab.today"], "", t)
	local top = Kit.Top(p)
	local hairline = t.H - t.BOTTOM - t.BUTTON - 8
	p.scroll = Kit.Scroll(p, t.scrollW, hairline - 2 - top)
	p.scroll:SetPoint("TOPLEFT", t.IN, -top)
	local rule = Kit.Rule(p, saga.listRule)
	rule:SetPoint("TOPLEFT", t.IN, -hairline)
	rule:SetPoint("TOPRIGHT", -t.IN, -hairline)
	local c, w = p.scroll.content, t.scrollW
	-- the unfinished moves: one row of a framed list, the HUD line's words with its tooltip, click and menus
	p.owed = Kit.List(c, w, w - 40)
	local function act(row, button)
		if p.owedLine then Sheet.Act(row.button, p.owedLine, button) end
	end
	p.owedRow = Kit.Row(p.owed, { name = "", chevron = true, onClick = function(row) act(row, "LeftButton") end,
		rightClick = function(row) act(row, "RightButton") end })
	Kit.Choose(p.owedRow, false, true)
	p.day = ns.Streamer.BuildDayBody(c, w)
	p.empty = Kit.Empty(c, w, { name = L["today.empty"] })
	p.hard = Kit.Text(c, "bodyWhite", "LEFT", saga.note, w)
	p.week = Kit.Section(c, "", false, "") -- named by the week's omen (Pages.RenderToday)
	p.vault = CreateFrame("Frame", nil, c)
	p.vault:SetSize(w, VAULT_H)
	p.vault.cards = {}
	local cw, gap = Kit.Grid(3, w)
	for i = 1, 3 do
		local card = Kit.Card(p.vault, cw, VAULT_H)
		card:SetPoint("TOPLEFT", (i - 1) * (cw + gap), 0)
		card.mark = card:CreateTexture(nil, "ARTWORK")
		card.mark:SetSize(16, 16)
		card.mark:SetPoint("TOPLEFT", 12, -14)
		Style.SetMark(card.mark, true)
		card.name = Kit.Text(card, "bodyWhite", "LEFT")
		card.count = Kit.Text(card, "large", "RIGHT")
		card.count:SetPoint("TOPRIGHT", -12, -12)
		card.pips = Kit.Text(card, "bodyWhite", "LEFT")
		card.pips:SetPoint("TOPLEFT", card.name, "BOTTOMLEFT", 0, -4)
		card.bar = Kit.Bar(card, cw - 24)
		card.bar:SetPoint("BOTTOMLEFT", 12, 16)
		-- its goal: the vault's own words and what it pays
		Kit.Hover(card, Kit.Tip(function() return L["vault.label." .. card.goal.goal] end,
			function() return L["toast.vault"]:format(L["vault." .. card.goal.goal]:format(card.goal.need)) end,
			{ facts = function() return { L["toast.xp"]:format(C.xp_vault_goal) } end }))
		p.vault.cards[i] = card
	end
	-- the tab's own tooltips on the day's blocks (the card window shows none): the hero's moves with their pumps and the
	-- app's share, the day's wins and stars; the XP with the rested bonus; each region's max sets
	local day = p.day
	Kit.Hover(day.hero, Kit.Tip(L["today.moves"], function()
		return ns.Plural("today.moves.tip", p.today.pumps):format(p.today.pumps)
	end, { facts = function()
		local d, facts = p.today, {}
		if d.app.moves > 0 then facts[#facts + 1] = L["today.app"]:format(d.app.moves) end
		facts[#facts + 1] = L["today.sets"]:format(L["today.wins"], d.wins) .. " · "
			.. L["today.sets"]:format(L["today.stars"], d.stars)
		facts[#facts + 1] = L["today.wins.tip"]
		return facts
	end }))
	Kit.Hover(day.stats.cells[3], Kit.Tip(L["today.xp"], function()
		return p.today.rested > 0 and L["tip.hud.rested"]:format(number(p.today.rested)) or nil
	end))
	for i, col in ipairs(day.regions.cols) do
		Kit.Hover(col, Kit.Tip(function() return L["region." .. p.today.regions[i].region] end,
			function() return L["today.region.sets"]:format(p.today.regions[i].sets) end))
	end
	p.foot = Kit.Footer(p)
	p.share = select(2, ns.Streamer.DayButtons(p.foot))
	p.foot:Lay()
end

function Pages.RenderToday()
	local p = Sheet.pages.today
	local now = ns.now()
	local t, d = ns.DayTally(ns.state, now), ns.Today(ns.state, now)
	p.today = d
	p.intro:SetText(Sheet.DayDate(ns.day(now)) .. " " .. ns.Streamer.LevelWords())
	local day = p.day
	ns.Streamer.FillDayBody(day, t, now)
	-- the unfinished moves, as the HUD's line has them; under the felt-unwell lock the line only says why
	local owed = d.unfinished > 0
	if owed then
		local line = owedLine(d.unfinished, Lc.Locked())
		p.owedLine = line
		p.owedRow.name:SetText(line.text)
		p.owedRow.chevron:SetShown((line.action or line.menu) ~= nil)
		Kit.Hover(p.owedRow.button, line.tooltip)
		Kit.LayList(p.owed)
	else
		p.owedLine = nil
	end
	-- the records' share from the app, on their section label's tooltip while it has one
	local head = day.wins.head
	if t.records > 0 and d.app.records > 0 then
		Kit.Hover(head, Kit.Tip(L["today.records"], L["today.app"]:format(d.app.records)))
	else
		head:SetScript("OnEnter", nil)
		head:EnableMouse(false)
	end
	local hard = {}
	for _, h in ipairs(d.hard) do hard[#hard + 1] = L["today.hardSets"]:format(L["ladder." .. h.ladder], h.sets) end
	p.hard:SetText(table.concat(hard, "\n"))
	-- the vault's title names the week's omen (slice 54, ns.Omen: "Hare week", "This week" until then)
	Kit.SetCaps(p.week.words,
		L["today.week." .. ns.Omen(now, ns.FilterContext(ns.profile, nil), ns.Rung(ns.state, ns.CARDIO)).key])
	p.week.aside:SetText(L["today.vault"]:format(date("%a %H:%M", d.reset)))
	local vaultH = VAULT_H
	for i, card in ipairs(p.vault.cards) do
		local v = d.vault[i]
		card:SetShown(v ~= nil)
		if v then
			card.goal = v
			vaultH = math.max(vaultH, layVault(card, v))
		end
	end
	for _, card in ipairs(p.vault.cards) do card:SetHeight(vaultH) end
	p.vault:SetHeight(vaultH)
	local empty = t.moves == 0
	if empty then p.empty:Lay() end
	p.share:SetEnabled(not empty) -- nothing to share before the day's first move (Streamer.DayButtons says why)
	local bottom = Kit.Stack(nil, 0, {
		{ p.owed, 0, show = owed },
		{ day.who, 0, show = false }, -- the head carries the date and the level, the title bar the name
		{ day.hero, owed and 12 or 0 },
		{ day.stats, 6 },
		{ day.wins, 16, show = not empty and day.wins.count > 0 },
		{ day.strip, 16, show = not empty },
		{ day.regions, 16, show = not empty },
		{ p.empty, 16, show = empty },
		{ p.hard, 10, show = #hard > 0 },
		{ p.week, 18 },
		{ p.vault, 10 },
	})
	p.scroll.content:SetHeight(bottom + 4)
end

-- Deeds (2026-10-16, the titles in it since: docs/ui.md, "The character sheet"): every title the player can earn, by
-- its tier (ns.TITLE_TIERS, Legacy only while one is held), each row with its shield (a ladder top's its move's key
-- pose), the title, the deed's own sentence under it, and at its right the bar and the count, or "Earned", or the gold
-- "Your title" badge on the one worn; an earned row wears its title at a click, and the worn one takes it off. A chain
-- (the streak's, the Paragon's) shows its titles earned and its next; the ladder tops not yet earned fold under one
-- More in Legendary; the Feats of Strength come last, once one is earned (ns.Feats). The head says the title worn.
local DEED_BAR = 110
local TOPS = "legendary" -- the tier whose ladder tops fold under the More

-- An unearned title's bar and count, an earned one's check and word, the worn one's badge: the row's control.
local function deedState(parent)
	local ctl = CreateFrame("Frame", nil, parent)
	ctl:SetHeight(24)
	ctl.bar = Kit.Bar(ctl, DEED_BAR)
	ctl.bar:SetPoint("LEFT")
	ctl.count = Kit.Text(ctl, "bodyWhite", "RIGHT")
	ctl.count:SetPoint("RIGHT")
	ctl.word = Kit.Text(ctl, "large", "LEFT", { GREEN_FONT_COLOR:GetRGB() })
	ctl.word:SetPoint("RIGHT")
	ctl.word:SetText(L["deeds.earned"])
	ctl.mark = ctl:CreateTexture(nil, "ARTWORK")
	ctl.mark:SetSize(18, 18)
	ctl.mark:SetPoint("RIGHT", ctl.word, "LEFT", -6, 0)
	Style.SetMark(ctl.mark, true)
	ctl.badge = Kit.Badge(ctl, "gold", L["deeds.wornBadge"])
	ctl.badge:SetPoint("RIGHT")
	return ctl
end

-- A title's count as its deed counts it: miles for the yards on foot, minutes for seconds, else the count.
local function deedCount(d)
	local per = d.unit == "yards" and 1760 or d.unit == "seconds" and 60 or 1
	return L["deeds.progress." .. (d.unit or "count")]:format(number(math.floor(d.value / per)),
		number(math.floor(d.goal / per)))
end

-- The worn title off at a click on it, an earned one on: the profile's (ns.Touch), and the sheet's title bar with it.
local function wear(id)
	local off = ns.ChosenTitle(ns.state, ns.profile) == id
	ns.profile.title = not off and id or nil
	ns.Touch()
	Style.Play(off and "checkOff" or "checkOn")
	Sheet.Refresh()
end

-- A list of titles drawn: each row's words, picture, state and tooltip, the counts' column as wide as the widest, so
-- the bars stand in one column. Answers the list's height.
local function fillDeeds(list, deeds, worn)
	local column = 0
	for i, d in ipairs(deeds) do
		local row = kitRow(list, i, function()
			return { picture = true, line = "", control = deedState(list), onClick = true }
		end)
		row.name:SetText(L["title." .. d.id])
		row.line:SetText(L["deed." .. d.id])
		local move = d.top and ns.Anchor(d.top.ladder, d.top.rung)
		if move then
			rowPicture(row, move, ns.Quality(d.top.ladder, d.top.rung))
		else
			rowPicture(row, nil, nil, "deedShield")
		end
		local ctl, isWorn = row.control, d.id == worn
		ctl.bar:SetShown(not d.earned)
		ctl.count:SetShown(not d.earned)
		ctl.word:SetShown(d.earned and not isWorn)
		ctl.mark:SetShown(d.earned and not isWorn)
		ctl.badge:SetShown(isWorn)
		if not d.earned then
			ctl.bar:Set(d.value, d.goal)
			ctl.count:SetText(deedCount(d))
			column = math.max(column, ctl.count:GetStringWidth())
		end
		-- only an earned row acts at a click (it wears or takes off its title), so only it lights up under the cursor
		-- (Kit.Choose reads row.clicks); in colour once earned, the gold light and name on the one worn (G4: an earned
		-- row's name stays white)
		row.click = d.earned and function() wear(d.id) end or nil
		row.clicks = d.earned
		Kit.Choose(row, d.earned, not isWorn)
		row.tooltip = Kit.Tip(L["title." .. d.id], L["deed." .. d.id], {
			facts = d.tier == "legacy" and { L["titles.legacy.tip"] } or nil,
			hint = d.earned and L[isWorn and "deeds.click.off" or "deeds.click.wear"] or nil })
	end
	for i, d in ipairs(deeds) do
		local ctl = list.rows[i].control
		ctl:SetWidth(not d.earned and DEED_BAR + 12 + column
			or d.id == worn and ctl.badge:GetWidth() or 18 + 6 + ctl.word:GetStringWidth())
	end
	kitRows(list, #deeds + 1)
	return Kit.LayList(list)
end

-- The Mastered steps row under Milestones (slice 42, docs/ui.md "Mastered"; no title, never worn): the achievement
-- shield, in colour from the first mastered step (ns.Mastered: kept until its strength ladder starts over),
-- "Mastered steps", and its count at the right with no bar (the steps a setup offers change, so there is no fixed
-- whole); its tooltip the steps mastered, newest first ("Squat step 6 · Rare · 16 Oct", in the kit's three facts, the
-- third "+2 more steps" past three), and a click opens the newest one's card on Talents. Answers the list's height.
local MASTERED = "deed" -- the tier whose list it follows
local MASTERED_TIP = 3

local function fillMastered(list, state)
	local steps = {}
	for l = 1, ns.LADDER_MAX do
		for r in ipairs(ns.Ladders[l] and ns.Ladders[l].rungs or {}) do
			local ts = ns.Mastered(state, l, r)
			if ts then steps[#steps + 1] = { ladder = l, rung = r, ts = ts } end
		end
	end
	table.sort(steps, function(a, b)
		if a.ts ~= b.ts then return a.ts > b.ts end
		if a.ladder ~= b.ladder then return a.ladder < b.ladder end
		return a.rung < b.rung
	end)
	local row = kitRow(list, 1, function() return { picture = true, value = "", onClick = true } end)
	local any = #steps > 0
	row.name:SetText(L["deeds.mastered"])
	row.value:SetText(number(#steps))
	rowPicture(row, nil, nil, "deedShield")
	row.clicks = any
	row.click = any and function() Sheet.ShowStepCard(steps[1].ladder, steps[1].rung) end or nil
	Kit.Choose(row, any, true)
	local facts = {}
	for i, s in ipairs(steps) do
		if i == MASTERED_TIP and #steps > MASTERED_TIP then
			facts[i] = ns.Plural("deeds.mastered.more", #steps - i + 1):format(#steps - i + 1)
			break
		end
		facts[i] = L["deeds.mastered.step"]:format(L["ladder." .. s.ladder], ns.Step(s.ladder, s.rung),
			Sheet.QualityWord(ns.Quality(s.ladder, s.rung)), Sheet.ShortDate(ns.DayMid(s.ts, 0)))
	end
	row.tooltip = Kit.Tip(L["deeds.mastered"], L["deeds.mastered.line"],
		{ facts = facts, hint = any and L["tip.move.click"] or nil })
	kitRows(list, 2)
	return Kit.LayList(list)
end

function Pages.BuildDeeds(p)
	local body = listPage(p, L["sheet.tab.deeds"])
	p.tiers = {}
	for _, tier in ipairs(ns.TITLE_TIERS) do
		p.tiers[tier] = { head = Kit.Section(body, L["titles.tier." .. tier]), list = pageList(body) }
	end
	p.mastered = pageList(body)
	p.tops = Kit.More(body, PAGE.scrollW, nil, function(content)
		p.tops.list = pageList(content)
		p.tops.list:SetPoint("TOPLEFT")
	end)
	p.tops.onToggle = function() Pages.RenderDeeds() end
	p.feats = { head = Kit.Section(body, L["feats.title"]), list = pageList(body) }
end

function Pages.RenderDeeds()
	local p, state = Sheet.pages.deeds, ns.state
	local worn = ns.ChosenTitle(state, ns.profile)
	p.intro:SetText(worn and L["deeds.worn"]:format(colored(HIGHLIGHT_FONT_COLOR, L["title." .. worn]))
		or L["deeds.worn.none"])
	local deeds = ns.Deeds(state)
	-- a chain's next title: its first not reached (ns.Titles lists a chain in rising goals)
	local nextOf = {}
	for _, d in ipairs(deeds) do
		if d.chain and not d.earned and d.order < (nextOf[d.chain] or math.huge) then nextOf[d.chain] = d.order end
	end
	local byTier, tops = {}, {}
	for _, tier in ipairs(ns.TITLE_TIERS) do byTier[tier] = {} end
	for _, d in ipairs(deeds) do
		if d.top and not d.earned then
			tops[#tops + 1] = d
		elseif d.earned or not d.chain or nextOf[d.chain] == d.order then
			table.insert(byTier[d.tier], d)
		end
	end
	local parts, first = {}, true
	local function block(head, shown)
		parts[#parts + 1] = { head, first and 0 or 16, show = shown }
		first = first and not shown
	end
	for _, tier in ipairs(ns.TITLE_TIERS) do
		local sec, list = p.tiers[tier], byTier[tier]
		local folded = tier == TOPS and #tops or 0
		block(sec.head, #list + folded > 0 or tier == MASTERED)
		parts[#parts + 1] = { sec.list, 14, fillDeeds(sec.list, list, worn), show = #list > 0 }
		if tier == MASTERED then
			parts[#parts + 1] = { p.mastered, #list > 0 and 8 or 14, fillMastered(p.mastered, state) }
		end
		if tier == TOPS then
			local more = p.tops
			more.words:SetText(L["deeds.tops"]:format(#tops))
			local open = folded > 0 and more:IsOpen()
			local h = open and fillDeeds(more.list, tops, worn) or 0
			more.content:SetHeight(math.max(1, h))
			parts[#parts + 1] = { more, #list > 0 and 8 or 14, show = folded > 0 }
			parts[#parts + 1] = { more.content, 6, h, show = open }
		end
	end
	local feats = ns.Feats(state)
	block(p.feats.head, #feats > 0)
	for i, feat in ipairs(feats) do
		local row = kitRow(p.feats.list, i, function() return { picture = true, line = "", value = "", onClick = true } end)
		local name, color = moveName(feat.move), qualityColor(feat.quality)
		row.name:SetText(name)
		row.line:SetText(L["records.step"]:format(ns.Step(feat.ladder, feat.rung), Sheet.QualityWord(feat.quality)))
		row.value:SetText(feat.ts and fullDate(feat.ts) or "")
		rowPicture(row, feat.move, feat.quality)
		Kit.Choose(row, true, true)
		row.tooltip = Kit.Tip(name, L["feats.tip"]:format(L["ladder." .. feat.ladder], ns.Step(feat.ladder, feat.rung),
			L["quality." .. feat.quality]), { color = color, hint = L["tip.move.click"] })
		row.click = function() Sheet.ShowMoveCard(feat.move) end
	end
	kitRows(p.feats.list, #feats + 1)
	parts[#parts + 1] = { p.feats.list, 14, Kit.LayList(p.feats.list), show = #feats > 0 }
	p.body:SetHeight(math.max(1, Kit.Stack(p.body, 0, parts) + 8))
end

-- Guild (only in a guild, 2026-10-16 on the kit: sheet SPEC 9): the guildmates who run QuestPump, from the cards seen
-- in this session (nothing about them is stored), sorted by streak, never by a typed number, each with a mark when
-- active today and never a word for who is not; the week's pumps and cardio minutes of all of them with the player's
-- own share as three stats, the milestone under them, and the weekly line the player sends, the page's one red at its
-- foot. The head names the guild, with no line beside it. At rest (QuestPump asleep, an instance it is off in) or with
-- others' cards switched off, the empty state names which, with a grey button to the Social page, where both switch.
local GUILD_FOOT = PAGE.BUTTON + 10 -- the scroll stops over the bar's button, a hairline between

function Pages.BuildGuild(p)
	local body = listPage(p, L["sheet.tab.guild"], GUILD_FOOT)
	p.rule = Kit.Rule(p, Style.saga.rule)
	p.rule:SetPoint("BOTTOMLEFT", PAGE.IN, PAGE.BOTTOM + GUILD_FOOT - 2)
	p.rule:SetWidth(PAGE.TEXT)
	p.foot = Kit.Footer(p)
	p.line = p.foot:Primary(L["guild.sendLine"], function() ns.Social.SendGuildLine() end)
	p.line:SetWidth(math.max(PAGE.PRIMARY, p.line:GetTextWidth() + 40))
	p.foot:Lay()
	p.stats = CreateFrame("Frame", nil, body)
	p.stats:SetSize(PAGE.scrollW, BESTS_H)
	local w = PAGE.scrollW / 3
	p.week = {}
	for i, key in ipairs({ "pumps", "cardio", "share" }) do
		local cell = Kit.Stat(p.stats, { rule = i > 1 })
		cell:SetSize(w, BESTS_H)
		cell:SetPoint("TOPLEFT", (i - 1) * w, 0)
		p.week[key] = cell
	end
	p.milestone = Kit.Text(body, "bodyWhite", "LEFT", Style.saga.note, PAGE.scrollW)
	p.list = pageList(body)
	p.empty = Kit.Empty(body, PAGE.scrollW, { name = L["guild.empty"] })
	p.rest = Kit.Empty(body, PAGE.scrollW,
		{ button = { L["guild.openSocial"], function() ns.Settings.Open("social") end } })
end

function Pages.RenderGuild()
	local p = Sheet.pages.guild
	local guild = IsInGuild() and GetGuildInfo("player")
	p.title:SetText(guild or L["sheet.tab.guild"]) -- the guild's name once the client knows it
	local listening = ns.Social.Listening()
	local shown = listening and ns.profile.showCards
	p.line:SetShown(shown and ns.Social.GuildLine() ~= nil or false)
	-- at rest (QuestPump asleep, an instance it is off in) or others' cards switched off: the rest state says which
	local week = shown and ns.Social.GuildWeek() or { rows = {} }
	if shown then
		Kit.FillStat(p.week.pumps, number(week.pumps), L["guild.pumps"])
		Kit.FillStat(p.week.cardio, number(week.cardio), L["guild.cardio"])
		Kit.FillStat(p.week.share, number(week.share), L["guild.share"])
		p.milestone:SetText(week.milestone and L["guild.milestone"]:format(number(week.milestone)) or "")
	else
		p.rest:Set(L[not listening and "guild.quiet" or "guild.off"])
	end
	for i, r in ipairs(week.rows) do
		local c = r.card
		local row = kitRow(p.list, i, function() return { line = "", onClick = true } end)
		row.name:SetText(L["guild.row"]:format(r.key, c.L) .. (c.title and " · " .. L["title." .. c.title] or ""))
		local line = (c.S or 0) > 0 and ns.Plural("guild.streak", c.S):format(c.S) or ""
		if c.D == 1 then line = line .. (line ~= "" and " · " or "") .. L["guild.today"] .. " " .. Style.Check() end
		row.line:SetText(line)
		Kit.Choose(row, true, true)
		row.click = function() ns.Social.OpenCard(r.key) end
		row.tooltip = function(tooltip)
			GameTooltip_SetTitle(tooltip, r.key)
			for _, l in ipairs(ns.Social.CardLines({ card = c })) do tooltip:AddLine(l, 1, 1, 1, true) end
			GameTooltip_AddInstructionLine(tooltip, L["guild.click"])
		end
	end
	kitRows(p.list, #week.rows + 1)
	p.empty:Lay()
	p.rest:Lay()
	p.body:SetHeight(math.max(1, Kit.Stack(p.body, 0, {
		{ p.stats, 0, BESTS_H, show = shown and true or false },
		{ p.milestone, 8, show = week.milestone ~= nil },
		{ p.list, 16, Kit.LayList(p.list), show = #week.rows > 0 },
		{ p.empty, 24, show = shown and #week.rows == 0 or false },
		{ p.rest, 24, show = not shown },
	}) + 8))
end
