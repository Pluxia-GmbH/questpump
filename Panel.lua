-- What the player sees of a pump (docs/plan.md slice 1, "Pump panel"; docs/ui.md): the dim and the pump card with one
-- view at a time (Ready, the countdown, Running, Result and "I feel unwell"), the non-modal "Stop to pump" card and
-- the setup screen. All of it is built from the kit (Kit.lua) on Blizzard's templates, fonts, atlases and sound kits.
-- The panel draws what ns.Lifecycle announces and hands every click and key back to it; it decides nothing about a
-- pump. No secure template anywhere (the beta cannot compile secure snippets).

local ADDON, ns = ...
local C, E, L = ns.C, ns.Enum, ns.L
local K, O = E.kind, E.outcome
local Kit, Lc, Setup, Style = ns.Kit, ns.Lifecycle, ns.Setup, ns.Style
local saga = Style.saga

local Panel = { frames = { views = {} } }
ns.Panel = Panel
local F = Panel.frames
local T = ns.Timers()

-- The card (2026-10-16, the game's look, decision G1: the setup window's flat frame and kit, a quest page on
-- DefaultPanelTemplate's rock, 740 wide, until then): the kit's medium tier (TM), 600 wide, each view's move or
-- question as its title over the Options window's faded rule and one soft line, a column of words at the left beside
-- the move figure's screen at the right (TM.split), and the kit's bottom bar: the ways back or out grey at the left,
-- the one way on red at the right. Every view lays itself out at its render from its words' heights (Kit.Stack), so a
-- larger Text Size makes the card taller, never its words overlap; past its budget it folds (renderReady), past its
-- cap it scales (Kit.Fit).
local TM = Kit.TIER.M
local LEFT_W, SPLIT_GAP = TM.split[1], TM.split[2]
local SIDE_X = TM.IN + LEFT_W + SPLIT_GAP -- the right column's left edge
local SCREEN_W, SCREEN_H = TM.screen[1], TM.screen[2] -- the figure's screen round its C.figure_ready_px box
-- The card's bottom edge stands where a card of the tier's budget, CARD_Y above the screen's middle, would end (low
-- enough for a card of the tier's cap), and keeps it through the pump: a taller view grows upward, so Start, Done,
-- Record and Next stand under the pointer one after the other, and a click that changes its own view keeps the top
-- (placeCard); its top stays SCREEN_MARGIN inside the screen.
local CARD_Y, SCREEN_MARGIN = 40, Kit.MARGIN
-- The Streamer layout's card (slice 23): in the top-left corner, unless the player dragged it by its title somewhere
-- else (cardPos), growing down. Of the four corners only this one clears Blizzard's action bars, chat and minimap on a
-- 16:9 screen (Forever 70205's Edit Mode presets: the minimap, the buffs and the tracker top right, the bar stack
-- bottom centre to right, the chat bottom left up to 453 units from the top in Modern), 4 units in. Classic's player
-- and target frames sit under it, which a pump never needs.
local CORNER = { point = "TOPLEFT", x = 4, y = -4 }
-- The character's local reactions on the recorded Result (AnimationData ids on 69913; names from a community list
-- until the beta check): Flex on a record, Cheer on a star or a first-try beat, Salute on a rung-up.
local FLEX, CHEER, SALUTE = 82, 68, 113
-- The plan's movement bindings plus the rest of Forever's movement group that moves the character
-- (Bindings_Camelot.xml): mouse-steer, auto-run on, pitch, follow, and interact, which walks to the target.
local MOVEMENT = { "MOVEFORWARD", "MOVEBACKWARD", "STRAFELEFT", "STRAFERIGHT", "TURNLEFT", "TURNRIGHT", "JUMP",
	"TOGGLEAUTORUN", "STARTAUTORUN", "MOVEANDSTEER", "PITCHUP", "PITCHDOWN", "FOLLOWTARGET", "INTERACTTARGET" }
local GRIP_H = 24 -- the flat frame's title bar, by which the Streamer layout's card is dragged
local DISPLAY_TICK_S = 0.2
local LIST_LINES = 6 -- a long pump's list (every unfinished move rides along) shows these around the current move
local DOTS_MOST = 8 -- the Ready view's step dots beside where the move stands, for a pump of at most this many moves
local OUTCOME = {} -- outcome code → name, for the locale keys
for name, code in pairs(O) do OUTCOME[code] = name end
-- a counted set's answers to "All out and clean?" by their rows' order (slice 53): Yes, to the end (0), Form slipped,
-- Not all out (ns.Enum.answer)
local FORM_ANSWERS = { 0, E.answer.slipped, E.answer.notAllOut }
-- the answers to "How close to the goal pose?" by their key: each digit is its answer's code (ns.Enum.mark), 1 Not yet
-- to 4 Perfect, so a stray 1 is the lowest answer (2026-10-31); the rows list them from Perfect down (markKeys)
local MARK_BY_KEY = {}
for _, code in pairs(E.mark) do MARK_BY_KEY[code] = code end
local NOTE = CreateColor(saga.note[1], saga.note[2], saga.note[3]) -- the notes' grey inside a line of white words
local WHITE = { HIGHLIGHT_FONT_COLOR:GetRGB() } -- the goal until it is met (decision G1; gold once met: paintGoal)

-- Small builders ---------------------------------------------------------------------------------------------------

local buildSide, placeCard

local text = Style.Text

-- The key that does a bar button's job as its tooltip's green hint ("Key: Space"), while the panel takes keys (none in
-- the mouse-only mode): `keyWord` a key's locale key, or a function answering it where the key changes with the state.
local function keyHint(keyWord)
	return function()
		if Panel.InputMode() == "mouse" then return nil end
		return L["ui.key"]:format(L[type(keyWord) == "function" and keyWord() or keyWord])
	end
end

-- A bar button's tooltip in the kit's shape, over it.
local function barTip(b, title, body, hint)
	Kit.Hover(b, Kit.Tip(title, body, { hint = hint }), "ANCHOR_TOP")
	return b
end

local function clock(seconds)
	seconds = math.max(0, math.floor(seconds + 0.5))
	return string.format("%d:%02d", math.floor(seconds / 60), seconds % 60)
end
Panel.Clock = clock -- the day's tally's cardio time too (Streamer.lua)

local function moveName(slot)
	local m = ns.Library[slot.move]
	local name = L["move." .. m.key]
	return m.side and L["ui.side.each"]:format(name) or name
end

local function timedKind(slot)
	return slot.kind == K.cardio or slot.kind == K.flow
end

-- A move's setup line or how lines ("setup", "how") for the implement the player's gear carries its load with
-- (ns.Implement: "move.<key>.setup.<gear>" where the locale has one, so a row names the dumbbell, not a bag), else its
-- own; /qp how prints the same setup line.
function Panel.MoveText(moveId, part)
	local key = "move." .. ns.Library[moveId].key .. "." .. part
	local gear = Panel.MoveGear(moveId, part)
	return gear and L[key .. "." .. gear] or L[key]
end

-- A move's tips ("\n" between them) and last, where the move names a helping pose, its advice (move.<key>.hint), which
-- the Talents card's Helps row no longer carries under its name (2026-10-22).
function Panel.MoveTips(moveId)
	local key = "move." .. ns.Library[moveId].key
	local advice = rawget(L, key .. ".hint")
	return advice and L[key .. ".tips"] .. "\n" .. advice or L[key .. ".tips"]
end

-- A move's stop line, the form break that ends its set; nil for a timed move with no form break (David, 2026-10-24:
-- only what is legally necessary, so no move carries a stop line that only warns of a symptom).
function Panel.MoveStop(moveId)
	return rawget(L, "move." .. ns.Library[moveId].key .. ".stop")
end

-- The implement whose own words a move's part is shown in (ns.Implement, where the locale has the part for it), else
-- nil: the guide reads that take (Speech.GuideTakes).
function Panel.MoveGear(moveId, part)
	local gear = ns.Implement(moveId, ns.FilterContext(ns.profile, ns.state).gear)
	return gear and rawget(L, "move." .. ns.Library[moveId].key .. "." .. part .. "." .. gear) and gear or nil
end

-- The move's name with the side a side move is on once its countdown has started (`p.side`: "Ankle rocks: first
-- side"); before it, after it and in a /qp try, which runs both sides as one, the move's name.
local function sideName(p, slot)
	local name = L["move." .. ns.Library[slot.move].key]
	if p.side == 1 then return L["ui.side.first"]:format(name) end
	if p.side == 2 then return L["ui.side.other"]:format(name) end
	return moveName(slot)
end

-- Two parts of a text, `sep` between them when both have words.
local function join(a, b, sep)
	if a == "" then return b end
	if b == "" then return a end
	return a .. sep .. b
end

-- Keys: the panel swallows only what it handles ------------------------------------------------------------------

local movement = {}

-- Bindings match on their last key token plus the modifier state, because GetBindingKey can return "SHIFT-W".
function Panel.ReadBindings()
	wipe(movement)
	for _, action in ipairs(MOVEMENT) do
		for _, binding in ipairs({ GetBindingKey(action) }) do
			movement[#movement + 1] = { key = binding:match("[^-]+$"), shift = binding:find("SHIFT%-") ~= nil,
				ctrl = binding:find("CTRL%-") ~= nil, alt = binding:find("ALT%-") ~= nil }
		end
	end
end

function Panel.IsMovement(key)
	for _, b in ipairs(movement) do
		if b.key == key and (not b.shift or IsShiftKeyDown()) and (not b.ctrl or IsControlKeyDown())
			and (not b.alt or IsAltKeyDown()) then
			return true
		end
	end
	return false
end

-- A click that changes the view it is on (How's bar, the guide's Read, a swap): `fn(...)`, whose render keeps the
-- card's top where it stands in the standard layout (placeCard), so the control clicked stays under the pointer.
local function keepingTop(fn, ...)
	local p, card = Lc.pump, F.card
	local top = card and card:IsVisible() and card:GetTop()
	if p and top and not Panel.Streamer() then p.keepTop = top * card:GetScale() end
	fn(...)
	if p then p.keepTop = nil end -- a click that drew nothing keeps nothing
end

-- The count's input mode (a profile setting from slice 3): the typed box by default, the spinner (+/- buttons) with a
-- gamepad or when chosen, and the mouse-only mode, in which the panel never captures the keyboard. The pump's count is
-- on its dial in every mode (slice 66: its - and + and the wheel, the keys while the panel takes them); Correct a
-- count's box keeps the mode (Settings.lua).
function Panel.InputMode()
	local mode = ns.profile and ns.profile.inputMode or "auto"
	if mode == "auto" then return C_GamePad.IsEnabled() and "spinner" or "type" end
	return mode
end

local function typed()
	return Panel.InputMode() == "type"
end

-- The Streamer layout (General > Layout, slice 23): no dim, the card in a corner, the day's tally on the screen
-- (Streamer.lua). The pump asks at once as in the standard layout.
function Panel.Streamer()
	return ns.profile ~= nil and ns.profile.layout == "streamer"
end

-- The count on the dial (Panel.SetCount), or nil while nothing is entered.
function Panel.InputValue()
	return F.count
end

-- The weight box's weight in tenths of a kg, or nil: no loaded move, an empty box or 0 (the set counts reps alone).
function Panel.LoadValue()
	if not (F.kg and F.kg:IsShown()) then return nil end
	return ns.Sheet.ParseKg(F.kg:GetText())
end

local function countEmpty()
	return Panel.InputValue() == nil
end

-- Lets go of the keyboard: the weight box.
local function letGo()
	if F.kg then F.kg:ClearFocus() end
end

-- The count's keys (Panel.TypeCount): a digit goes onto the dial's number, and onto the number it started at (a hold's
-- stopwatch, the voice's rep count) replaces it, and Backspace takes its last digit off, or clears that number whole.
local DIGIT = { BACKSPACE = false }
for d = 0, 9 do
	DIGIT[tostring(d)], DIGIT["NUMPAD" .. d] = tostring(d), tostring(d)
end

-- Whether the ask the Result view shows (the whole move's, the goal pose's, the pace's, the hold's, a counted set's
-- answers, the count) has stood C.answer_settle_s since it showed (renderResult's v.askAt; David, 2026-10-31: "I
-- had stretches that now are marked done in the desktop app even so I never said I did them perfect etc"): its digits
-- and its clicks answer only then, so the key or the click that ended the move never answers what came in its place.
local function answerSettled()
	local v = F.views.result
	return v ~= nil and v.askAt ~= nil and GetTime() - v.askAt >= C.answer_settle_s
end

-- An ask's answer as its row's click takes it: nothing before the ask has settled (answerSettled).
local function settledAnswer(answer)
	return function()
		if answerSettled() then answer() end
	end
end

-- True while the Result view's dial takes the count (slice 66): a set of reps', a hold's or a counted set's, until
-- Save or an answer saves it.
local function counting(p)
	local v = F.views.result
	return p ~= nil and p.state == "result" and p.view == "result" and not p.committed and p.pour == nil and v ~= nil
		and (v.state == "count" or v.state == "form")
end

-- The arrows on the count's dial (slice 66): Up and Right as its +, Down and Left as its - (David, 2026-11-04, QA-9).
local DIAL_STEP = { UP = 1, RIGHT = 1, DOWN = -1, LEFT = -1 }

-- True when the key is swallowed. Movement keys and Space always (Space starts a Ready view's move, on a side move's
-- first side switches sides and while Keep going shows keeps going, slice 52; it never ends a move); Escape only past
-- an open pump's views (on every
-- move's Ready view, and "I feel unwell" beside it, it opens the game menu: Escape never banks,
-- and the player can always log out); Enter while a move runs, the count waits on the dial or the result waits for a
-- click, and while "Did you do the whole move?" asks, where it does nothing (the answer takes a click, never the key
-- that just ended the move); while "How close to the goal pose?" asks, a digit 1-4 answers it with the answer of that
-- code (1 Not yet to 4 Perfect), while "Did you keep that pace to the end?" or "Could you have held longer?" asks a
-- digit 1-3 (Enter does nothing there); an answer's digit is swallowed and answers nothing before its ask has settled
-- (answerSettled); while the count waits on the dial (slice 66) the digits and Backspace type it and the arrows step
-- it, ahead of the movement bindings that hold the arrows by default; everything else goes on to the game, chat
-- included.
function Panel.HandleKey(key)
	local p = Lc.pump
	if not p then return false end
	if key == "SPACE" then -- before the movement check: Space is the jump binding by default
		if p.state == "open" and p.view == "ready" then
			Lc.Start()
		elseif p.state == "running" and p.side == 1 then
			Lc.Done() -- Switch sides, the key at hand in a desk plank (David, 2026-10-14: "yes space for switch sides")
		elseif Lc.KeepShows() then
			Lc.KeepGoing() -- the bar's secondary key; Enter stays Done (slice 52)
		end
		return true
	end
	local dial = counting(p)
	if dial and DIGIT[key] ~= nil then
		Panel.TypeCount(key)
		return true
	end
	if dial and DIAL_STEP[key] then
		Panel.StepCount(DIAL_STEP[key])
		return true
	end
	if Panel.IsMovement(key) then return true end
	if p.state == "result" and p.marking and not p.committed then
		local answer = MARK_BY_KEY[tonumber(DIGIT[key] or "") or 0]
		if answer then
			if answerSettled() then Lc.Mark(answer) end
			return true
		end
		if key == "ENTER" or key == "NUMPADENTER" then return true end
	end
	-- "Did you keep that pace to the end?" (slice 42; three answers since slice 52): the digits 1-3 answer in the ask's
	-- order (ns.Enum.paceAnswers), and Enter answers nothing
	if p.state == "result" and p.pacing and not p.committed then
		local answer = E.paceAnswers[tonumber(DIGIT[key] or "") or 0]
		if answer then
			if answerSettled() then Lc.Pace(E.pace[answer]) end
			return true
		end
		if key == "ENTER" or key == "NUMPADENTER" then return true end
	end
	-- "Could you have held longer?" (slice 52): the digits 1-3 answer in the codes' order (ns.Enum.longer), and Enter
	-- answers nothing
	if p.state == "result" and p.longerAsk and not p.committed then
		local answer = E.longerKeys[tonumber(DIGIT[key] or "") or 0]
		if answer then
			if answerSettled() then Lc.Longer(E.longer[answer]) end
			return true
		end
		if key == "ENTER" or key == "NUMPADENTER" then return true end
	end
	if key == "ENTER" or key == "NUMPADENTER" then
		if p.state == "running" then
			Lc.Done()
			return true
		end
		if p.state == "result" and p.asking then return true end
		-- a pour: Next from its start (its row saved first, Lc.Next), once it has settled (answerSettled)
		if p.state == "result" and p.pour then
			if answerSettled() then Lc.Next() end
			return true
		end
		if p.state == "result" and p.committed and p.view == "result" then
			Lc.Next()
			return true
		end
		if dial then
			-- Save with a count (a counted set's: the "Is that right?" row's Yes alone), never before the dial has
			-- settled: the press that ended the move, pressed twice, saves nothing unread (Panel.RecordInput); an empty
			-- dial's Enter is the panel's too, or it opens the chat, which then takes the digits
			if p.really or not countEmpty() then Panel.RecordInput() end
			return true
		end
		return false
	end
	if key == "ESCAPE" then
		if p.state == "open" or not p.started then return false end
		-- an Escape right behind the Save never takes the pour back before it has settled (slice 65)
		if p.state == "result" and p.pour and not answerSettled() then return true end
		if dial and p.really then
			Panel.ChangeCount() -- Change it
			return true
		end
		-- the Escape that ended the move, pressed twice, never saves an empty dial as done without a count
		if dial and not answerSettled() then return true end
		Lc.Escape(countEmpty())
		return true
	end
	return false
end

local function onKeyDown(self, key)
	if InCombatLockdown() then return end -- the propagate flag cannot change in combat; the frame is hidden there
	self:SetPropagateKeyboardInput(not Panel.HandleKey(key))
end

local function onKeyUp(self)
	if not InCombatLockdown() then self:SetPropagateKeyboardInput(true) end
end

-- The countdown: Blizzard's pull-countdown art inside the card -----------------------------------------------------

local fade = Kit.Alpha -- an Alpha animation in a group

-- Blizzard's startNumbers (Timer.xml) on `targets`: scale 0.25 → 1 in 0.3 s, then, `hold` s later, a pop to 1.2 and a
-- fade in 0.1 s. A digit holds 0.6 s, one second each; the go holds 0.1 s (buildRunning). Blizzard pops only the
-- digits; the digit's glow pops with it here, so the faded glow flashes back for that 0.1 s.
local function startNumbers(ag, targets, hold)
	for _, target in ipairs(targets) do
		local grow = ag:CreateAnimation("Scale")
		grow:SetTarget(target)
		grow:SetScaleFrom(0.25, 0.25)
		grow:SetScaleTo(1, 1)
		grow:SetDuration(0.3)
		grow:SetSmoothing("OUT")
		grow:SetOrder(1)
		local pop = ag:CreateAnimation("Scale")
		pop:SetTarget(target)
		pop:SetScaleFrom(1, 1)
		pop:SetScaleTo(1.2, 1.2)
		pop:SetStartDelay(hold)
		pop:SetDuration(0.1)
		pop:SetOrder(2)
		fade(ag, target, 1, 0, 0.1, 2):SetStartDelay(hold)
	end
end

-- The countdown's column, centred beside the figure: a side move's "First side" or "Switch sides" in the move's font,
-- "Ready..." and then Blizzard's digits with their glow at three quarters of their size on one spot, and the goal
-- under it (David, 2026-10-03: the record or goal to break "during the countdown 54321go and excercise itself").
local function buildCountdown(v)
	local d, col = Style.digits, v.left
	v.side = text(col, "move", "CENTER", LEFT_W)
	v.spot = CreateFrame("Frame", nil, col)
	v.spot:SetSize(LEFT_W, math.ceil(d.h * 0.75))
	v.ready = text(v.spot, "clock", "CENTER")
	v.ready:SetPoint("CENTER")
	v.ready:SetText(L["ui.panel.ready"])
	v.digit = v.spot:CreateTexture(nil, "ARTWORK")
	v.digit:SetTexture(d.file)
	v.digit:SetSize(d.w * 0.75, d.h * 0.75)
	v.digit:SetPoint("CENTER")
	v.glow = v.spot:CreateTexture(nil, "ARTWORK", nil, 1)
	v.glow:SetTexture(d.glow)
	v.glow:SetBlendMode("ADD")
	v.glow:SetAllPoints(v.digit)
	v.goal = text(col, "large", "CENTER", LEFT_W)
	-- the glow fades while the digit grows, then the digit pops at 0.9 s
	local ag = v:CreateAnimationGroup()
	startNumbers(ag, { v.digit, v.glow }, 0.6)
	fade(ag, v.glow, 1, 0, 0.3, 1):SetSmoothing("IN")
	ag:SetToFinalAlpha(true)
	v.anim = ag
end

local function showDigit(n)
	local v = F.views.countdown
	v.ready:Hide()
	v.digit:SetTexCoord(Style.DigitCoords(n))
	v.glow:SetTexCoord(Style.DigitCoords(n))
	v.digit:SetAlpha(1)
	v.glow:SetAlpha(1)
	v.digit:Show()
	v.glow:Show()
	v.anim:Stop()
	v.anim:Play()
	Style.Play("tick")
end

local function cancelDigits()
	for n = 1, C.countdown_s do T.cancel("digit" .. n) end
end

-- One timer per digit, each scheduled from the one absolute go, so a late frame never shifts the go; a digit whose
-- second has passed (a render that comes late) is skipped, never shown late.
local function scheduleDigits(p)
	cancelDigits()
	local v = F.views.countdown
	v.ready:Show()
	v.digit:Hide()
	v.glow:Hide()
	Style.Play("countdownStart")
	for n = C.countdown_s, 1, -1 do
		local at = p.goAt - n - GetTime()
		if at >= 0 then T.after("digit" .. n, at, function() showDigit(n) end) end
	end
end

-- The pump's words ---------------------------------------------------------------------------------------------------

-- A loaded move's weight words after its reps ("with 11 kg", "with 6 kg each"); "" without a weight.
local function withLoad(moveId, load)
	if not load then return "" end
	return L[ns.Loaded(moveId) == "each" and "ui.goal.withLoadEach" or "ui.goal.withLoad"]:format(ns.Sheet.Kg(load))
end

-- The record a set is judged against, as a line names it: with a weight (`load`) and a weighted record that record
-- ("10 kg × 20", ns.LoadRecord), else the count (a hold's seconds), as ns.Classify and ns.Validate compare. `set`, a
-- recorded set's (Lc.Commit's p.last), holds the weighted record from before it, which the set may have replaced
-- (Panel.ReallyWords': the one of the state it judges against).
local function recordText(slot, best, load, set)
	local topLoad, topReps
	if set then
		topLoad, topReps = set.topLoad, set.topReps
	else
		topLoad, topReps = ns.LoadRecord(ns.state, slot.move)
	end
	if load and topLoad then return ns.Sheet.LoadAmount(slot.move, topLoad, topReps) end
	return ns.Sheet.Amount(best, slot.kind == K.hold)
end

-- The pump's name, on the banner that closes it (Sheet.lua) and on the card's title bar over its own move of no quest
-- (cardTitle): a pump asked for is "Unfinished moves" when it carries one, else what it is; a move started from the
-- Talents tab (`p.chosen`) is one asked for, the unfinished row it may pay included, and a stretch check started there
-- (its `ask`, slice 42) "Stretch check".
function Panel.Title(p)
	if p.kind == "practice" then return L["ui.panel.title.practice"] end
	if p.kind == "standup" then return L["ui.panel.title.standup"] end
	if p.chosen then return L[p.chosen.ask and "ui.panel.title.check" or "ui.panel.title.asked"] end
	if p.kind == "demand" then
		for _, slot in ipairs(p.moves) do
			if slot.owed then return L["ui.panel.title.owed"] end
		end
		return L["ui.panel.title.asked"]
	end
	local q = p.quests[1]
	return q and Lc.QuestTitle(q.questID) or L["ui.panel.title.pump"]
end
local title = Panel.Title

-- A loaded move's line under its goal once a set had a weight: the weighted record and the last set with theirs
-- ("Your record: 10 kg × 20 · last time: 10 kg × 20"; a goal from the form its strongest fresh set lately,
-- ns.LoadForm; a repeat's, done today already), the goal row naming the new weight after a step up (ns.LoadPlan). Nil
-- without a weighted record, or without a weight planned: ns.Goal then reads the goal by reps, and recordLine names
-- the count record, as recordText does.
local function loadLine(slot, kind, last)
	local id = slot.move
	local topLoad, topReps, lastLoad = ns.LoadRecord(ns.state, id)
	if not (topLoad and slot.load) then return nil end
	local record = ns.Sheet.LoadAmount(id, topLoad, topReps)
	if kind == "repeat" then return L["ui.target.repeat"]:format(record) end
	if kind == "form" then
		return L["ui.target.form"]:format(record, ns.Sheet.LoadAmount(id, ns.LoadForm(ns.state, id)))
	end
	if not last or (lastLoad == topLoad and last == topReps) then return L["ui.target.recordLast"]:format(record) end
	return L["ui.target.record"]:format(record, ns.Sheet.LoadAmount(id, lastLoad, last))
end

-- The line under a goal: the record and what the goal is built on, by its kind (ns.Goal): a record attempt names the
-- last count ("Your record: 22 · last time: 20"), a goal from the player's form their best fresh set lately, a repeat
-- that the move was done today already, a loaded move's with the weights (loadLine); "" without a record.
local function recordLine(slot)
	local best, last = ns.Record(ns.state, slot.move)
	if not best then return "" end
	local hold = slot.kind == K.hold
	local record = ns.Sheet.Amount(best, hold)
	local _, kind = ns.SlotGoal(slot)
	local weighted = loadLine(slot, kind, last)
	if weighted then return weighted end
	if kind == "form" then
		return L["ui.target.form"]:format(record, ns.Sheet.Amount(ns.Form(ns.state, slot.move), hold))
	end
	if kind == "repeat" then return L["ui.target.repeat"]:format(record) end
	if not last or last == best then return L["ui.target.recordLast"]:format(record) end
	return L["ui.target.record"]:format(record, ns.Sheet.Amount(last, hold))
end

-- The line under a goal for a move the climb waits on alone (ns.PathDue, the voice's path callout's own test): what
-- opens the next step, in the goal row's own words, beside this set's goal, which a repeat or a form goal keeps lower
-- (2026-10-16, David: "the goal is 8 while record is 10. idont know what thesentence is telling me"), with the game
-- days at it the step's path still needs (slice 42: two at a strength move's top); a pose's days close enough (or
-- Perfect, ns.MarkNeeds); a counted form's clean days, never a count (slice 42); "" otherwise.
local function nextStepLine(slot)
	local top, left = ns.PathDue(ns.state, slot, ns.FilterContext(ns.profile, ns.state), ns.now())
	if not top then return "" end
	if slot.counted then return ns.Plural("ui.target.nextClean", left):format(left) end
	if ns.POSE[slot.ladder] then
		return ns.Plural(top >= E.mark.perfect and "ui.target.nextPerfect" or "ui.target.nextPose", left):format(left)
	end
	local m = ns.Library[slot.move]
	local unit = slot.kind == K.hold and (m.side and "ui.goal.secsSide" or "ui.goal.secs")
		or (m.side and "ui.goal.repsSide" or "ui.goal.reps")
	local amount = L["ui.goal.atLeast"]:format(top) .. " " .. ns.Plural(unit, top)
	local kg, each = ns.ClimbKg(slot.move)
	if kg then amount = L[each and "amount.climbEach" or "amount.climb"]:format(amount, kg) end
	return ns.Plural("ui.target.next", left):format(amount, left)
end

-- The line under a pose's goal pose where its answer counts its family's day (slice 47, 2026-10-19: ns.MarkToward
-- "step" with the family's days, ns.PoseNeeds): the days toward the step so far, Perfect days alone below a harder step
-- (the slot's `ask`), and what they still lack (Tree.PoseShort: the days on the main pose, by name, and the Perfect
-- day); "" where the answer counts no family day or the days are in.
local function poseDaysLine(slot)
	if not (slot.mark and slot.ask) then return "" end
	local f = ns.FilterContext(ns.profile, ns.state)
	local toward, _, have, need, p = ns.MarkToward(ns.state, slot, f)
	if toward ~= "step" or not p or p.done then return "" end
	local line = L[slot.ask >= E.mark.perfect and "ui.target.posePerfect" or "ui.target.poseDays"]:format(have, need)
	return join(line, ns.Tree.PoseShort(slot.ladder, slot.rung, f, p) or "", " ")
end

-- A steady cardio move's pace in its cadence line's words (Speech.PaceLine; slice 42's fix round, 2026-10-17: the pace
-- ask's "that pace" stated without the voice), on its Ready view and its Running view (renderRunning); "" for any
-- other move. The week's omen said a cue after it from slice 54 until 2026-11-03 (Rapha: Fist's "Two punches, then a
-- knee." asked shadow boxing for a knee strike its figure and steps lack).
local function paceLine(slot)
	local pace = ns.Speech.PaceLine(slot)
	return pace and L[pace] or ""
end

-- A loaded move's start range before any weight is typed (the library's kg: "Start with 5-15 kg"); "" otherwise.
local function startLine(slot)
	local how, m = ns.Loaded(slot.move), ns.Library[slot.move]
	if not how or slot.load then return "" end
	return L[how == "each" and "ui.goal.loadStartEach" or "ui.goal.loadStart"]:format(m.kg[1], m.kg[2])
end

-- What the goal row shows for a move: the value (the count or seconds to reach now, "23+" because it is the least,
-- never the most; or the words of a set with no count to reach), its unit and the line under it. `set`, a recorded
-- set's (Lc.Commit's p.last), puts its goal at the weight typed (ns.TargetAt) in the planned one's place.
local function goalOf(slot, set)
	if slot.practice then return tostring(slot.limit), L["ui.goal.secs"], L["ui.target.practice"] end
	local m = ns.Library[slot.move]
	-- a counted set (2026-10-05; slice 42: never a number to reach): "All out, clean reps" in words, under it its best
	-- clean count once it has one, its window, and the clean days the next step still needs where its path waits on
	-- this form (nextStepLine)
	if slot.counted then
		local best = ns.Record(ns.state, slot.move)
		local line = best and ns.Plural("ui.target.best", best):format(best) or ""
		return L["ui.goal.allout"], "", join(join(line, L["ui.target.count"]:format(C.count_s), "\n"),
			nextStepLine(slot), "\n")
	end
	if slot.kind == K.cardio then
		local secs = L["ui.goal.secs"]
		if slot.allout then
			return tostring(slot.seconds), secs, L["ui.target.allout"]:format(C.allout_build_s,
				slot.seconds - C.allout_build_s)
		end
		return tostring(slot.seconds), secs, join(L["ui.target.cardio"], paceLine(slot), "\n")
	end
	if slot.kind == K.flow then -- a side move's seconds are each side's half; a carry pushes through its load
		local seconds = math.floor(m.side and (slot.seconds or 0) / 2 or (slot.seconds or 0))
		-- a ladder pose names the mark it asks about after the hold (slice 21), with the family's days it can count
		local target = slot.mark and L["ui.target.mark"]:format(ns.MarkText(slot.move))
			or L[m.pattern == "carry" and "ui.target.carry" or "ui.target.flow"]
		return tostring(seconds), L[m.side and "ui.goal.secsSide" or "ui.goal.secs"],
			join(join(target, poseDaysLine(slot), "\n"), nextStepLine(slot), "\n")
	end
	local hold = slot.kind == K.hold
	-- a loaded move's planned weight after the unit ("16+ reps with 11 kg"; slot.load, ns.LoadPlan), or with no weight
	-- typed yet the library's start range under the line
	local target, load = slot.target, slot.load
	if set then target, load = set.target, set.load end
	local start, weight = join(nextStepLine(slot), startLine(slot), "\n"), withLoad(slot.move, load)
	if target then
		local unit = hold and (m.side and "ui.goal.secsSide" or "ui.goal.secs")
			or (m.side and "ui.goal.repsSide" or "ui.goal.reps")
		return L["ui.goal.atLeast"]:format(target), join(ns.Plural(unit, target), weight, " "),
			join(recordLine(slot), start, "\n")
	end
	local most = L[hold and "ui.goal.longest" or "ui.goal.most"]
	-- a first max set says the sides too, and the weight
	local each = join(m.side and L["ui.goal.onEachSide"] or "", weight, " ")
	-- a placement after a break names the record that stays; an anchor with none of its own takes the first-set line
	local stays = slot.placing and ns.Record(ns.state, slot.move)
	if stays then return most, each, L["ui.target.placing"]:format(recordText(slot, stays, slot.load)) end
	if slot.hi then
		return most, each, join(L[hold and "ui.target.firstHiHold" or "ui.target.firstHi"]:format(
			ns.Sheet.Amount(slot.hi, hold)), start, "\n")
	end
	return most, each, join(L[hold and "ui.target.firstHold" or "ui.target.first"], start, "\n")
end

-- The goal in one line, for the countdown, Running and Result views ("Goal: 23+ reps · record 22", "Goal: 45
-- seconds", "Goal: 16+ reps with 11 kg · record 10 kg × 20", a counted set's "Goal: All out, clean reps"); a practice
-- move has none. A recorded set's (`set`, Lc.Commit's p.last) names the goal at the weight typed and the record the
-- set was judged against, from before it.
local function goalLine(slot, set)
	if slot.practice then return "" end
	local value, unit = goalOf(slot, set)
	local line = L["ui.goal.line"]:format(unit ~= "" and value .. " " .. unit or value)
	if not (slot.kind == K.rep or slot.kind == K.hold) then return line end
	local best, load
	if set then
		best, load = set.best, set.load
	else
		best, load = ns.Record(ns.state, slot.move), slot.load
	end
	if best then line = line .. " · " .. L["ui.goal.lineRecord"]:format(recordText(slot, best, load, set)) end
	return line
end

-- Where the player stands in the pump, the Ready view's line under the move. The unfinished moves riding along come
-- after the pump's own and count apart (David, 2026-10-04: a 34-move pump read "Move 1 of 34 · +31 unfinished", which
-- looked like 65): "Move 2 of 34 · then 31 unfinished" on an own move, "Unfinished 1 of 31 · move 4 of 34" on a rider,
-- "Unfinished 1 of 31" in a pump of them only. A move started from the Talents tab is the pump's own, an unfinished row
-- it pays too. The turn-ins merged into the pump are its tooltip's (introFacts).
local function contextLine(p)
	if p.chosen then return L["ui.panel.move"]:format(p.index, #p.moves) end
	local riders, at = 0, 0
	for i, slot in ipairs(p.moves) do
		if slot.owed then
			riders = riders + 1
			if i <= p.index then at = at + 1 end
		end
	end
	local slot = p.moves[p.index]
	if riders == #p.moves then return L["ui.panel.unfinishedOf"]:format(at, riders) end
	if slot and slot.owed then return L["ui.panel.riderOf"]:format(at, riders, p.index, #p.moves) end
	local line = L["ui.panel.move"]:format(p.index, #p.moves)
	return riders > 0 and line .. " · " .. L["ui.panel.riders"]:format(riders) or line
end

-- The pump's moves, one a line, the tooltip of the line under the move (the card's own list until 2026-10-16): the
-- done ones ticked, the current one bright. A pump longer than LIST_LINES shows the move before the current one and
-- the next ones, and counts the rest above and below.
local function listLine(p)
	local n = #p.moves
	if n < 2 then return "" end
	local first = n > LIST_LINES and math.max(1, math.min(p.index - 1, n - LIST_LINES + 1)) or 1
	local last = math.min(n, first + LIST_LINES - 1)
	local lines = {}
	if first > 1 then lines[1] = DISABLED_FONT_COLOR:WrapTextInColorCode(L["ui.panel.above"]:format(first - 1)) end
	for i = first, last do
		local slot = p.moves[i]
		local name = moveName(slot) .. (slot.allout and " " .. L["ui.panel.allout"] or "")
		if p.results[i] then
			lines[#lines + 1] = CreateAtlasMarkup(Style.atlas.done, 12, 12) .. " "
				.. DISABLED_FONT_COLOR:WrapTextInColorCode(name)
		elseif i == p.index then
			lines[#lines + 1] = HIGHLIGHT_FONT_COLOR:WrapTextInColorCode(i .. ". " .. name)
		else
			lines[#lines + 1] = i .. ". " .. name
		end
	end
	if last < n then lines[#lines + 1] = L["ui.panel.below"]:format(n - last) end
	return table.concat(lines, "\n")
end

-- The facts under the pump's moves in that tooltip: the turn-ins merged into it, and on its first move the ground the
-- character covered since the last pump (the odometer, its line on the view until 2026-10-16).
local function introFacts(p)
	local facts = {}
	if #p.quests > 1 then facts[1] = L["ui.panel.merged"]:format(#p.quests) end
	local s = p.stretch
	if p.index == 1 and s and (s.foot + s.mount) >= 1 then
		facts[#facts + 1] = L["ui.reason.stretch"]:format(BreakUpLargeNumbers(math.floor(s.foot)),
			BreakUpLargeNumbers(math.floor(s.mount)))
	end
	return facts
end

-- The Ready view's first line, soft: since when an unfinished move riding along waits, else the quest's own flavour
-- line in quotes (a turn-in's own move; Lc.Flavour has none for a rider or a pump asked for); nil for none.
local function leadLine(p, slot)
	if slot.owed then return L["ui.reason.owed"]:format(ns.Sheet.Since(slot.owed.ts)) end
	local line = Lc.Flavour(p, slot)
	return line and L["ui.flavour"]:format(line) or nil
end

-- A move's first appearance: no record yet, so the pump asks for a first max set (or a placement after a break), or a
-- timed move (a stretch, a pose, a cardio form) never performed; the Ready view then plays the New-move sequence, with
-- the fault pair where the pack has one (slice 22: until then only a max set opened it, so no pose ever did). An
-- all-out window keeps its Ready view: its form is among the pump's last efforts, with nothing to study first.
local function firstAppearance(slot)
	return ns.FirstMaxSet(ns.state, slot)
		or ((slot.kind == K.flow or slot.kind == K.cardio) and not slot.allout and not ns.state.seen[slot.move])
end

-- The countdown's and the Running view's line under the move: its stop line through the set (2026-10-04), a rep or
-- hold set's after "Max set." (a side hold's and a side move of reps' too), a timed move's alone where it has one, an
-- all-out window's and a counted set's too (2026-10-16: "Count every rep!" alone stood there until then); a practice
-- move's own words.
local function phaseLine(slot)
	if slot.practice then return L["ui.running.practice"] end
	local stop = Panel.MoveStop(slot.move)
	if timedKind(slot) then return stop end
	return L["ui.running.maxset"]:format(stop)
end

-- The Running view's phase word over the clock at `elapsed` on the run's clock (Lc.Elapsed): an all-out window's build
-- on the same move from the go, then all out; a counted set's count from its go, with no build (2026-10-08); nil on
-- any other move. renderRunning sets it before it lays the view out, the display's tick as the phase turns.
local function phaseWord(slot, elapsed)
	if not slot.allout then return nil end
	local build = slot.counted and 0 or C.allout_build_s
	return elapsed < build and L["ui.running.build"] or L[slot.counted and "ui.running.count" or "ui.running.allout"]
end

-- The rewards (2026-10-16; the quest page's Rewards section with its four chips until then): the chips a count decides
-- (a record, a first try, a star) each on its own, the ones any finished move earns (a familiarity step, a new
-- pattern, the week's practice) as one chip per stat (no rung: an unlock takes two days, slice 20). The Ready view
-- tells them in one line (rewardsLine), the recorded Result as rows marked earned or missed (renderRewardRows).
local CERTAIN = { fam = true, pattern = true, practice = true }

local function shownChips(chips)
	local out, byStat = {}, {}
	for _, c in ipairs(chips) do
		local merged = CERTAIN[c.kind] and byStat[c.stat]
		if merged then
			merged.amount = merged.amount + c.amount
			merged.parts[#merged.parts + 1] = c
		elseif CERTAIN[c.kind] then
			merged = { kind = "certain", stat = c.stat, amount = c.amount, parts = { c } }
			byStat[c.stat] = merged
			out[#out + 1] = merged
		else
			out[#out + 1] = c
		end
	end
	return out
end

-- A chip a result earned: a stat gain of the same stat for its reason (a merged chip: for any of its parts).
local REASONS = { record = { record = true }, star = { star = true }, pattern = { pattern = true },
	practice = { practice = true, active = true } }

local function earnedChip(c, earned)
	if c.parts then
		for _, part in ipairs(c.parts) do
			if earnedChip(part, earned) then return true end
		end
		return false
	end
	for _, e in ipairs(earned or {}) do
		if e.kind == "stat" and e.stat == c.stat then
			if c.kind == "fam" and e.reason == c.step then return true end
			if REASONS[c.kind] and REASONS[c.kind][e.reason] then return true end
		end
	end
	return false
end

local function certainWord(c)
	return c.kind == "fam" and L["fam." .. c.step] or L["ui.rewards." .. c.kind]
end

-- A chip's words: "+1 Strength" and its condition ("on a new record", two stars at 20, "tried, this week's practice").
-- After the result a merged chip that earned some of its parts counts those and dashes the others; one that earned
-- none greys whole, as any missed chip does.
local function chipText(c, earned)
	local gain, partial = c.amount, false
	if c.parts and earned then
		local got = 0
		for _, part in ipairs(c.parts) do
			if earnedChip(part, earned) then got = got + part.amount end
		end
		partial = got > 0 and got < c.amount
		if partial then gain = got end
	end
	local name = L["ui.rewards.stat"]:format(gain, ns.Sheet.StatName(c.stat))
	local sub
	if c.kind == "record" then
		sub = L["ui.rewards.onRecord"]
	elseif c.kind == "star" then
		sub = L["ui.rewards.star"]:format(Style.Stars(c.stars), ns.Sheet.Amount(c.at, c.hold))
	else
		local words = {}
		for i, part in ipairs(c.parts) do
			local missed = partial and not earnedChip(part, earned)
			words[i] = missed and L["ui.rewards.missed"]:format(certainWord(part)) or certainWord(part)
		end
		sub = table.concat(words, L["ui.rewards.join"])
	end
	return name, sub
end

-- What a move can earn, once per slot: a re-rolled pump (a new pain flag), the next move or a record cleared under the
-- move (Events.lua Lc.Cleared drops it) brings a new one; the result keeps the preview it was judged against.
local function previewOf(p, slot)
	if not p.preview or p.previewFor ~= slot then
		p.preview, p.previewFor = ns.RewardPreview(ns.state, slot), slot
		p.previewChips = shownChips(p.preview.chips)
	end
	return p.preview
end

-- The Ready view's rewards in one line: the XP the move pays done, then each chip as its stat's icon, its gain in
-- white and its condition in the notes' grey ("+40 XP · +1 Strength on a new record"); nil for a move that earns none
-- (practice, the stand-up move).
local function rewardsLine(p, slot)
	if slot.practice or slot.standup then return nil end
	local parts = { L["ui.rewards.gain"]:format(previewOf(p, slot).xp) }
	for _, c in ipairs(p.previewChips) do
		local name, sub = chipText(c)
		parts[#parts + 1] = Style.IconMark(Style.statIcon[c.stat], 14) .. " " .. name .. " "
			.. NOTE:WrapTextInColorCode(sub)
	end
	return table.concat(parts, " · ")
end

-- That line's tooltip: each chip's words in full.
local function rewardsTip(tooltip)
	GameTooltip_SetTitle(tooltip, L["ui.rewards.title"])
	local p = Lc.pump
	for _, c in ipairs(p and p.previewChips or {}) do
		local name, sub = chipText(c)
		tooltip:AddDoubleLine(Style.IconMark(Style.statIcon[c.stat], 14) .. " " .. name, sub, 1, 1, 1,
			saga.note[1], saga.note[2], saga.note[3])
	end
end

-- What the swap row offers (Lc.SwapOptions), the line its tooltip adds: after a swap the move it replaced; else the
-- backup, the same level done another way ("Closest" a rung below), else the easier version, else another move of the
-- same pattern.
local function swapTip(which, option)
	if which == "back" then return L["ui.backup.tip.back"] end
	if which == "easier" then return L["ui.backup.tip.easier"] end
	if which == "another" then return L["ui.backup.tip.another"] end
	if option.lower then return L["ui.backup.tip.closest"] end
	return L["ui.backup.tip"]
end

-- The offers in the row's order (a first appearance's puts the easier version first), and the words of each, the move
-- named, on the row and in the Swap menu.
local SWAP_ORDER = { "back", "backup", "easier", "another" }
local SWAP_FIRST = { "back", "easier", "backup", "another" }

local SWAP_WORDS = { back = "ui.swap.back", easier = "ui.swap.easier", another = "ui.swap.another",
	backup = "ui.swap.backup" }

local function swapWords(which, option)
	return L[which == "backup" and option.lower and "ui.swap.closest" or SWAP_WORDS[which]]:format(moveName(option))
end

-- The Swap menu, opened by the arrow at the swap row's right: every offer but the one on the row.
local function swapItems(_, root)
	local options, row = Lc.SwapOptions(), F.swapRow
	root:CreateTitle(L["ui.swap.title"])
	for _, which in ipairs(SWAP_ORDER) do
		local option = options[which]
		if option and not (row and row.which == which) then
			root:CreateButton(swapWords(which, option), function() keepingTop(Lc.Swap, which, option) end)
		end
	end
end

-- A counted set's outcome (slice 42): Form slipped or, since slice 53, Not all out saved with no clean day, a clean
-- window not counted, a clean count over the best (or the first, which becomes the best), one under it beside the best
-- that stands.
local function countedLine(last)
	if last.answer == E.answer.slipped then return L["ui.outcome.slipped"]:format(last.xp) end
	if last.answer == E.answer.notAllOut then return L["ui.outcome.notAllOut"]:format(last.xp) end
	local n = last.value
	if not n or n <= 0 then return L["ui.outcome.cleanNone"]:format(last.xp) end
	if last.outcome == O.beat or last.outcome == O.placement then
		return ns.Plural("ui.outcome.bestNew", n):format(n, last.xp)
	end
	return L["ui.outcome.clean"]:format(n, last.best or n, last.xp)
end

-- The words of a set the Result view records: its outcome line (a record, more than last time, the goal reached, a
-- miss, a timed move's seconds, a counted set's clean count) with the XP it paid.
local function outcomeLine(p, slot)
	local last = p.last
	if slot.practice then return L["ui.result.practiceDone"]:format(p.practiceValue or 0) end
	if not last then return "" end
	-- a counted window that ran its 30 s by its answer; a timed move's seconds (a counted one stopped early too)
	if slot.counted and last.outcome ~= O.stopped then return countedLine(last) end
	if timedKind(slot) then
		return L[last.outcome == O.done and "ui.result.finished" or "ui.result.stoppedAtXP"]:format(last.value, last.xp)
	end
	local o = last.outcome
	local hold = slot.kind == K.hold
	local function count(v) return ns.Sheet.Amount(v, hold) end -- a hold's count in seconds ("45 s")
	-- a set with a weight reads with it ("11 kg × 16"), against the weighted record it had before (Lc.Commit) when it
	-- had one, else the count's
	local set = last.load and ns.Sheet.LoadAmount(slot.move, last.load, last.value) or count(last.value)
	local record = recordText(slot, last.best or 0, last.load, last)
	if o == O.censored and slot.hi and slot.maxSet and last.value == slot.hi then
		return L["ui.outcome.enough"]:format(set, last.xp) -- a first max set ended at hi: the next rung
	end
	if o == O.placement and last.value == 0 then return L["ui.outcome.couldNot"]:format(last.xp) end
	-- a placement after a break at or under the record: the record stays (a first record has none before it)
	if o == O.placement and last.best then
		return L["ui.outcome.placed"]:format(set, record, last.xp)
	end
	if o == O.beat or o == O.placement or o == O.censored then
		return L["ui.outcome." .. OUTCOME[o]]:format(set, last.xp)
	elseif o == O.lastbeat and last.target and last.value >= last.target then -- the goal reached, under the record
		return L["ui.outcome.goal"]:format(set, record, last.xp)
	elseif o == O.lastbeat and last.load then -- more than last time by its score (ns.Classify): no count difference
		return L["ui.outcome.lastbeatLoad"]:format(set, record, last.xp)
	elseif o == O.lastbeat then
		return L["ui.outcome.lastbeat"]:format(count(last.value - (last.previous or 0)), record, last.xp)
	elseif o == O.miss then
		return L["ui.outcome.miss"]:format(set, record, last.xp)
	end
	return L["ui.outcome.done"]:format(last.xp)
end

-- After a set with a weight, the next fresh set's weight and goal (Lc.Commit, ns.NextLoad): "Next time: 11 kg, 15+
-- reps"; "" after a set without one.
local function nextLoadLine(p, slot)
	local last = p.last
	if not (p.committed and last and last.nextLoad and last.nextTarget) then return "" end
	return L[ns.Loaded(slot.move) == "each" and "ui.result.nextLoadEach" or "ui.result.nextLoad"]:format(
		ns.Sheet.Kg(last.nextLoad), last.nextTarget)
end

-- The stretch checks a pose's answer can pass (slice 42), read as its ask shows, before the answer (ns.PoseChecks, the
-- one list of the game, the desktop app and the phones since 2026-10-18, slice 42's last review): the check a card
-- started first (its Start's `check`, the one that Start runs: Tree.RunsCheck), then the one a pump's stretch slot was
-- given for (Engine.lua checkSlot's `slot.check`, 2026-10-17: the slot asks its answer, ns.CheckAsk), then every other
-- the pose answers, each live (not done, its opening not open already); with its family's rung then, which a pass may
-- lift. Kept on the pump for its move (p.checks), the Result's lines read them after the answer (afterLines).
local function readChecks(p, slot)
	if p.checksFor == p.index then return end
	local from = p.fromTree
	p.checksFor = p.index
	local first = {}
	if from and from.check then first[1] = from.check end
	first[#first + 1] = slot.check
	p.checks = ns.PoseChecks(ns.state, slot.move, first)
	p.checkRung = ns.Rung(ns.state, slot.ladder)
end

-- What a check opens, a move (or a family's first step: its anchor), or the climb into a step of another ladder.
local function checkOpens(c)
	local o = c.opens or {}
	if o.ladder then return nil, o.ladder, ns.Step(o.ladder, o.rung) end
	local id = o.move or (o.family and ns.Anchor(o.family, 1))
	return id and ns.Sheet.MoveName(id) or nil
end

-- A passed check's line on the Result (slice 42): what it opened, "Check passed: {move} is open" (a climb's "the climb
-- to {ladder} step {n}"), only where that is open after the set (ns.OpensNow, 2026-10-17, slice 42's review: passing
-- the 90-90 said the dragon pistol at the desk was open while it still waited for ankle step 6); else that it is not
-- open yet (its head) over what still holds it in that wait's own words (Tree.WaitWords). Answers the line, or the
-- head and the wait's words apart.
local function passedLine(c, f)
	local name, ladder, step = checkOpens(c)
	local open, why = ns.OpensNow(ns.state, c.opens, f)
	if open then
		return ladder and L["ui.result.checkClimb"]:format(L["ladder." .. ladder], step or 0)
			or L["ui.result.checkPassed"]:format(name or "")
	end
	local head = ladder and L["ui.result.checkClimbHeld"]:format(L["ladder." .. ladder], step or 0)
		or L["ui.result.checkHeld"]:format(name or "")
	return head, ns.Tree.WaitWords(c.opens, why)
end

-- The lines under a recorded set's outcome (slice 42): the cardio day it brought (a clean window's, a full move's, a
-- Long Haul's toward its next length: Lc.WriteResult's `day`), each stretch check its answer passed with what that
-- opens (passedLine), each line once and the openings one wait still holds together over its words, said once
-- (2026-10-18, slice 42's last review: the 90-90 passed for the dragon pistol at the desk and squat step 12 said ankle
-- step 6 twice, as the desktop app says it once), and the pose's family's lift to the check's step; after a check a
-- card started, that it waits where that check did not pass (the one its Start ran, whatever else the answer passed:
-- the desktop app's alike).
local function afterLines(p, slot)
	local last, lines = p.last, {}
	if not (p.committed and last) then return "" end
	local d = last.day
	if d and d.kind == "haul" then
		lines[1] = L["ui.result.haulDay"]:format(clock(d.time), d.have, d.need, clock(d.next))
	elseif d and (d.kind == "holds" or d.kind == "back") then -- a step's measure back from a break (slice 53)
		lines[1] = L[d.kind == "holds" and "ui.result.stepHolds" or "ui.result.stepBack"]:format(d.step)
	elseif d and d.kind == "waits" then -- a proving window that did not pass (slice 53)
		lines[1] = L["ui.result.climbWaits"]
	elseif d then
		lines[1] = L[d.kind == "clean" and "ui.result.cleanDay" or "ui.result.fullMove"]:format(d.have, d.need,
			L["ladder." .. ns.CARDIO], d.step)
	end
	if p.checksFor ~= p.index or slot.kind ~= K.flow then return table.concat(lines, "\n") end
	local f, started, ran = ns.FilterContext(ns.profile, ns.state), p.fromTree and p.fromTree.check, false
	local said, held, out = {}, {}, {}
	for _, c in ipairs(p.checks or {}) do
		if ns.NeedDone(ns.state, c.group) then
			ran = ran or c.need == started
			local line, wait = passedLine(c, f)
			if not said[line] then
				said[line] = true
				-- an opening a wait holds joins the heads over that wait, at the place of the first
				local group = wait and held[wait]
				if wait and not group then
					group = { wait = wait }
					held[wait], out[#out + 1] = group, group
				end
				if group then group[#group + 1] = line else out[#out + 1] = line end
			end
		end
	end
	for _, x in ipairs(out) do
		lines[#lines + 1] = type(x) == "table" and join(table.concat(x, "\n"), x.wait, "\n") or x
	end
	local now = ns.Rung(ns.state, slot.ladder)
	if next(said) and p.checkRung and now > p.checkRung then
		lines[#lines + 1] = L["ui.result.checkPlaced"]:format(L["ladder." .. slot.ladder], ns.Step(slot.ladder, now) or now)
	end
	if started and not ran then lines[#lines + 1] = L["ui.result.checkAgain"] end
	return table.concat(lines, "\n")
end

-- Building the frames ----------------------------------------------------------------------------------------------

-- A view: a page over the whole card on the card's content frame (a UI replacement's skin strips only the card's own
-- textures, Skins.lua), with its head (Kit.Head: the title, the faded rule and the soft line under it), its column of
-- words (`left`, which placeColumn places) and its bottom bar (Kit.Footer, whose buttons show and hide with the view).
-- The right column, the figure's screen, is the card's own (buildSide), shared by the views that show a picture.
local function view(name)
	local v = CreateFrame("Frame", nil, F.card.body)
	v:SetAllPoints()
	v:Hide()
	Kit.Head(v, "", nil, TM)
	v.title:SetMaxLines(2)
	v.left = CreateFrame("Frame", nil, v)
	v.left:SetSize(LEFT_W, 1)
	v.left:SetPoint("TOPLEFT", TM.IN, 0)
	v.left.inset, v.left.contentW = 0, LEFT_W -- its own content's measures (Kit.Stack, Kit.Section, Kit.LayNumber)
	v.foot = Kit.Footer(v)
	F.views[name] = v
	return v
end

-- The view's column of words: at the card's left beside the figure, or centred in the card where the view shows no
-- picture (`centred`), `w` units wide (LEFT_W without; TM.TEXT across the card).
local function placeColumn(v, centred, w)
	w = w or LEFT_W
	v.left:SetWidth(w)
	v.left.contentW = w
	v.left:ClearAllPoints()
	v.left:SetPoint("TOPLEFT", TM.IN + (centred and (TM.TEXT - w) / 2 or 0), 0)
end

-- The head's title centred over the card, as the rule and the line under it are, with `beside` (Blizzard's NEW on a
-- first appearance) 10 to the right of its words, the room for it kept on both sides; and its line with the step dots
-- 12 after its words, the two centred together.
local function placeTitle(v, beside)
	local room = beside and beside:GetWidth() + 10 or 0
	v.title:SetWidth(TM.TEXT - 2 * room)
	v.title:ClearAllPoints()
	v.title:SetPoint("TOP", 0, -TM.HEAD)
	if not beside then return end
	beside:ClearAllPoints()
	beside:SetPoint("LEFT", v.title, "CENTER", v.title:GetStringWidth() / 2 + 10, 0)
end

local function placeIntro(v, dots)
	local room = dots:IsShown() and dots:GetWidth() + 12 or 0
	v.intro:SetWidth(TM.TEXT - room)
	v.intro:ClearAllPoints()
	v.intro:SetPoint("TOP", v.rule, "BOTTOM", -room / 2, -TM.introGap)
	dots:ClearAllPoints()
	dots:SetPoint("LEFT", v.intro, "CENTER", v.intro:GetStringWidth() / 2 + 12, 0)
end

-- The head's words: its title and its line, which hides with no words and then takes no room (Kit.Top).
local function setHead(v, words, intro)
	v.title:SetText(words)
	v.intro:SetText(intro or "")
	v.intro:SetShown(intro ~= nil and intro ~= "")
end

-- A grey answer in a view's body (an answer to its question, never a way out: those are the bar's), the dialog's
-- button height, on `row`, a frame across the column that centreButtons lays out.
local ANSWER_H = Kit.TIER.S.BUTTON

local function answerButton(row, label, onClick)
	return Kit.Secondary(row, label, TM.BACK, onClick, ANSWER_H)
end

-- The answers that show on `row`, centred on it 10 apart, each as wide as its words + 40 and at least the tier's grey
-- button, or one over the other, each centred, where they do not fit the row's width side by side (the count's column
-- beside the figure); the row takes their height. Answers whether any shows.
local function centreButtons(row, buttons)
	local shown, w = {}, -10
	for _, b in ipairs(buttons) do
		if b:IsShown() then
			b:SetWidth(math.max(TM.BACK, (ns.Readable(b:GetTextWidth()) or 0) + 40))
			shown[#shown + 1] = b
			w = w + b:GetWidth() + 10
		end
	end
	local stacked = w > row:GetWidth()
	row:SetHeight(stacked and #shown * (ANSWER_H + 6) - 6 or ANSWER_H)
	local x = -w / 2
	for i, b in ipairs(shown) do
		b:ClearAllPoints()
		if stacked then
			b:SetPoint("TOP", row, "TOP", 0, -(i - 1) * (ANSWER_H + 6))
		else
			b:SetPoint("LEFT", row, "CENTER", x, 0)
			x = x + b:GetWidth() + 10
		end
	end
	return #shown > 0
end

-- A row across the column holding answers (centreButtons).
local function answerRow(parent, w)
	local row = CreateFrame("Frame", nil, parent)
	row:SetSize(w, ANSWER_H)
	return row
end

-- The red way on's width: its words + 40 and at least the tier's (Finish, "Back to your tree").
local function primaryWidth(b)
	return math.max(TM.PRIMARY, (ns.Readable(b:GetTextWidth()) or 0) + 40)
end

-- An answer row of a list (the whole move's, the goal pose's, the felt-unwell pause's, the check-in's), each a
-- sentence: the row's words white and no gold (Kit.Choose's care), a click answering.
local function answerRowOf(list, words, onClick, opts)
	opts = opts or {}
	local row = Kit.Row(list, { name = words, font = opts.font or "bodyWhite", below = true,
		chevron = opts.chevron, onClick = onClick })
	Kit.Choose(row, true, true)
	return row
end

-- The right column (the card's own, shared by the views that show a picture): the move figure's screen with the
-- figure at C.figure_ready_px over it, which loops on the Ready view (the New-move sequence on a first appearance, its
-- caption and the fault pair under the screen) and through the countdown and the run on the run's clock, mirrored on
-- a side move's other side, and holds its goal pose still while "How close to the goal pose?" asks; hovering it shows
-- the move card. Under it on the Ready view the swap row (renderSwap), and on a recorded win the character in a screen
-- of its own. Each render places it under its view's head (placeSide).
function buildSide()
	local side = CreateFrame("Frame", nil, F.card.body)
	side:SetSize(SCREEN_W, 1)
	F.side = side
	F.screen = Kit.Screen(side, SCREEN_W, SCREEN_H)
	F.screen:SetPoint("TOPLEFT")
	-- The move card: the move's own tooltip (Sheet.MoveTooltip, item grammar) on the figure's hover, without its name,
	-- which the view's title shows. A GameTooltip needs a name: its template names its lines after it. It stays in the
	-- tooltips' strata (TOOLTIP, its template's), fixed there: in the card's own it drew under the card's title bar
	-- and metal (the flat frame's TitleContainer at frame level 510, its NineSlice at 500) as it grew up from the
	-- figure, and the Streamer layout's strata change on the dim would carry it back down.
	local tip = CreateFrame("GameTooltip", "QuestPumpMoveCard", side, "GameTooltipTemplate")
	tip:SetFrameStrata("TOOLTIP")
	tip:SetFixedFrameStrata(true)
	-- its first line is the rung line, not a title: the body font, not the header's
	for _, where in ipairs({ "Left", "Right" }) do
		local line = _G["QuestPumpMoveCardText" .. where .. "1"]
		if line then line:SetFontObject(GameTooltipText) end
	end
	F.moveCard = tip
	-- the figure over the screen, never inside it (the screen clips what it holds, and the New-move caption and the
	-- fault pair hang under it)
	local fig = ns.Figure.Create(side, C.figure_ready_px, { width = SCREEN_W, under = F.screen })
	fig.frame:SetPoint("CENTER", F.screen)
	fig.frame:SetFrameLevel(F.screen:GetFrameLevel() + 2)
	-- a part of the guide under its head, its lines numbered in the notes' grey as the Ready view numbers How's steps
	local function guidePart(head, lines, numbered)
		tip:AddLine(" ")
		tip:AddLine(head, NORMAL_FONT_COLOR:GetRGB())
		local n = 0
		for line in (lines .. "\n"):gmatch("(.-)\n") do
			n = n + 1
			tip:AddLine(numbered and NOTE:WrapTextInColorCode(L["ui.panel.step"]:format(n)) .. " " .. line or line,
				1, 1, 1, true)
		end
	end
	fig.frame:SetScript("OnEnter", function(self)
		local p = Lc.pump
		local slot = p and p.moves[p.index]
		if not slot then return end
		tip:SetOwner(self, "ANCHOR_LEFT")
		tip:SetMinimumWidth(SCREEN_W)
		ns.Sheet.MoveTooltip(tip, slot.move, true)
		-- what the Ready view does not show (renderReady): How's steps where How folded for the card's height, so they
		-- are always one hover away, and the tips inside a folded How or folded for the height
		if F.howFolded then guidePart(L["ui.panel.how"], Panel.MoveText(slot.move, "how"), true) end
		if F.tipsFolded then guidePart(L["ui.panel.tips"], Panel.MoveTips(slot.move)) end
		if F.figure.moving then GameTooltip_AddInstructionLine(tip, L["ui.figure.pause"]) end
		tip:Show()
	end)
	fig.frame:SetScript("OnLeave", function() tip:Hide() end)
	-- shown again, it plays anew: from the start, or in the countdown and the run at the clock's phase
	fig.frame:HookScript("OnHide", function() F.figureFor = nil end)
	fig.frame:Hide()
	F.figure = fig
	-- the swap row's list; its row is made at its first showing, with the move it offers (swapRow)
	F.swap = Kit.List(side, SCREEN_W, SCREEN_W)
	F.swap:Hide()
	-- the character reacts to a record, a star, a first try or a rung-up, locally (never an emote others see)
	F.modelScreen = Kit.Screen(side, SCREEN_W, SCREEN_H)
	F.modelScreen:SetPoint("TOPLEFT")
	F.modelScreen:Hide()
	F.model = CreateFrame("ModelScene", nil, F.modelScreen, "PanningModelSceneMixinTemplate")
	F.model:SetPoint("TOPLEFT", 2, -2)
	F.model:SetPoint("BOTTOMRIGHT", -2, 2)
	F.model:Hide()
end

-- The swap row (renderSwap): the move it offers on a small screen, what the offer is with the move's name in one line
-- in white ("Same level: Knee push-up", swapWords); a click swaps it in, the arrow at its right opens the rest
-- (swapItems); its tooltip is the move's card, what the offer is and the click's hint.
local function swapRow(move)
	if F.swapRow then return F.swapRow end
	local row = Kit.Row(F.swap, { picture = move, name = "", font = "bodyWhite", below = true,
		menu = swapItems, onClick = function(self)
			Style.Play("start")
			keepingTop(Lc.Swap, self.which, self.option)
		end })
	Kit.Hover(row.button, function(tooltip)
		if not row.option then return end
		ns.Sheet.MoveTooltip(tooltip, row.option.move)
		tooltip:AddLine(swapTip(row.which, row.option), 1, 1, 1, true)
		GameTooltip_AddInstructionLine(tooltip, L["ui.backup.click"])
	end, "ANCHOR_LEFT")
	Kit.Hover(row.menu, Kit.Tip(L["ui.backup.menu"]), "ANCHOR_LEFT")
	F.swapRow = row
	return row
end

-- A move's guide (David, 2026-10-03: "Better UI of the How points"): one row a line, its mark in a gutter and its text
-- wrapping beside it, so a long step keeps its indent; the steps numbered in the notes' grey, each tip on the quest
-- tracker's objective nub, a little room between the rows.
local GUTTER, ROW_GAP = 20, 4

local function guideList(parent, numbered, w)
	local list = CreateFrame("Frame", nil, parent)
	list:SetWidth(w)
	list.numbered, list.rows, list.w = numbered, {}, w
	return list
end

local function guideRow(list, i)
	local row = list.rows[i]
	if row then return row end
	row = { text = text(list, "bodyWhite", "LEFT", list.w - GUTTER) }
	if list.numbered then
		row.mark = Kit.Text(list, "bodyWhite", "RIGHT", saga.note)
		row.mark:SetWidth(GUTTER - 6)
		row.mark:SetPoint("TOPRIGHT", row.text, "TOPLEFT", -6, 0)
	else
		row.mark = list:CreateTexture(nil, "ARTWORK")
		if not Style.SetAtlas(row.mark, "nub", true) then -- a client without the atlas: a small gold square
			row.mark:SetColorTexture(1, 0.82, 0)
			row.mark:SetSize(4, 4)
		end
	end
	list.rows[i] = row
	return row
end

-- Fills a guide list with the lines of a locale entry ("\n" between them); returns its height.
local function fillGuide(list, lines)
	local y, n = 0, 0
	for line in (lines .. "\n"):gmatch("(.-)\n") do
		n = n + 1
		local row = guideRow(list, n)
		if list.numbered then row.mark:SetText(L["ui.panel.step"]:format(n)) end
		row.text:SetText(line)
		row.text:ClearAllPoints()
		row.text:SetPoint("TOPLEFT", GUTTER, -y)
		if not list.numbered then -- the nub on the middle of the first line, at the text size the player chose
			local _, size = row.text:GetFont()
			row.mark:ClearAllPoints()
			row.mark:SetPoint("CENTER", row.text, "TOPLEFT", -GUTTER / 2, -(size or 12) / 2)
		end
		row.text:Show()
		row.mark:Show()
		y = y + row.text:GetStringHeight() + ROW_GAP
	end
	for i = n + 1, #list.rows do
		list.rows[i].text:Hide()
		list.rows[i].mark:Hide()
	end
	local h = math.max(0, y - ROW_GAP)
	list:SetHeight(h)
	return h
end

-- The guide's button (2026-10-09), one for How and Tips since 2026-10-10 (David: "how and tips and tricks should not
-- be separate but one with one stop and play button"), and since 2026-10-13 the only way a guide begins, so it says
-- what it does (David: "remove the autoplay setting for how and tips and make it default off. But place and label the
-- play button better"): the play or pause art with its words beside it, "Read How and Tips aloud", "Pause reading" or
-- "Go on reading", in a bar button's gold, white and the art lit under the cursor; at the right end of the How row on
-- the Ready view (a folded How opens as it begins reading: b.onRead), and in the middle of the bar on the countdown and
-- the Running view. A click reads the move's guide aloud, How then Tips (Speech.ToggleGuide, even with the voice's
-- switch off: the player asked), on through the countdown and the run until the set ends; another click pauses it and
-- the next goes on from the line it paused on. Its art and words follow Speech.GuideState (on "Voice"), and its width
-- holds the longest words, so the art stays where it was clicked; its tooltip says what it reads, and after a click
-- the game's sound settings silenced, which setting does.
local GUIDE_ART, GUIDE_GAP = 22, 4
-- the words for each Speech.GuideState, "idle" for a guide not begun
local GUIDE_WORDS = { idle = "ui.panel.guide.play", reading = "ui.panel.guide.pause", paused = "ui.panel.guide.resume" }

local function guideWords()
	return L[GUIDE_WORDS[ns.Speech.GuideState() or "idle"]]
end

local function guideArt(b)
	local art = ns.Speech.GuideState() == "reading" and "voicePause" or "voicePlay"
	if not (Style.SetAtlas(b.icon, art) and Style.SetAtlas(b.lit, art)) then
		Style.SetAtlas(b.icon, art .. "Glyph")
		Style.SetAtlas(b.lit, art .. "Glyph")
	end
	b.words:SetText(guideWords())
end

local function guideTip(b)
	GameTooltip:SetOwner(b, "ANCHOR_TOP")
	GameTooltip_SetTitle(GameTooltip, guideWords())
	GameTooltip:AddLine(L["ui.panel.guide.tip"], 1, 1, 1, true)
	if b.why then GameTooltip_AddErrorLine(GameTooltip, L["speech.silent"]:format(L["speech.silent." .. b.why])) end
	GameTooltip:Show()
end

local function guideButton(parent)
	local b = CreateFrame("Button", nil, parent)
	b.icon = b:CreateTexture(nil, "ARTWORK")
	b.icon:SetSize(GUIDE_ART, GUIDE_ART)
	b.icon:SetPoint("LEFT")
	b.lit = b:CreateTexture(nil, "HIGHLIGHT")
	b.lit:SetAllPoints(b.icon)
	b.lit:SetBlendMode("ADD")
	b.lit:SetAlpha(0.5)
	-- the words in a bar button's font, which the client's text size leaves as it is, so the width measured here holds
	b.words = text(b, "label", "LEFT")
	b.words:SetPoint("LEFT", b.icon, "RIGHT", GUIDE_GAP, 0)
	local w = 0
	for _, key in pairs(GUIDE_WORDS) do
		b.words:SetText(L[key])
		w = math.max(w, b.words:GetStringWidth())
	end
	b:SetSize(GUIDE_ART + GUIDE_GAP + math.ceil(w), Kit.BUTTON_H)
	b:SetScript("OnClick", function(self)
		local played, why = ns.Speech.ToggleGuide()
		self.why = not played and why or nil
		if self.onRead and ns.Speech.GuideState() == "reading" then self.onRead() end
		guideTip(self)
	end)
	b:SetScript("OnEnter", function(self)
		Style.SetFont(self.words, "labelWhite")
		guideTip(self)
	end)
	b:SetScript("OnLeave", function(self)
		Style.SetFont(self.words, "label")
		self.why = nil
		GameTooltip:Hide()
	end)
	return b
end

-- The guide's button in the middle of the bar on the countdown and the Running view, through which the guide reads on.
local function barGuide(v)
	v.guide = guideButton(v.foot)
	v.foot:Middle(v.guide)
end

-- The guide's button for the move on (`moveId`), once its guide has a take.
local function renderGuide(b, moveId)
	b.move = moveId
	b:SetShown(#ns.Speech.GuideTakes(moveId) > 0)
	guideArt(b)
end

-- Back (Blizzard's BACK), the bar's way back on the countdown, the Running view and the Result view until its count
-- is saved: it takes the move back to its Ready view with nothing saved (Lc.Back; David, 2026-10-05: "I want to be
-- able during the exercise to go back and have it like I never started the exercise. there needs to be a back
-- button"). Escape keeps its own meanings there.
local function backButton(v)
	local label = BACK or L["ui.panel.back"]
	return barTip(v.foot:Secondary(label, function() Lc.Back() end), label, L["ui.panel.back.tip"],
		L["ui.panel.back.click"])
end

-- "Settings", a grey button on the bar (David, 2026-10-03: "should be an actual button"; named "Your setup" until
-- 2026-10-05 and "Equipment" until 2026-10-10, David: "change the "Equipment" button to "Settings" button. That
-- logically includes equipment and lets you easily access the settings"): QuestPump's settings as /qp config opens
-- them, the pump waiting while they are open (Settings.lua); rare, so it folds into More where the bar is full.
local function settingsButton(v)
	return barTip(v.foot:Secondary(L["ui.panel.settings"], function() ns.Settings.Open() end, nil, true),
		L["ui.panel.settings"], L["ui.panel.settings.tip"], L["ui.panel.settings.click"])
end

-- How on the Ready view: a More bar (Kit.More) this wide on a move the player knows, with the steps under it while it
-- is open (play SPEC 2: open while the move is new or tried, folded from familiar on; a How the player opens or folds
-- stays so for that move until the next login, howOpen).
local HOW_W = 84
local howOpen = {}

local function buildReady()
	local v = view("ready")
	local col = v.left
	-- The head: the move, Blizzard's NEW beside it on a first appearance, and under them where the move stands in the
	-- pump with the step dots; the line's tooltip lists the pump's moves (the card's own list until 2026-10-16) with the
	-- turn-ins merged into it and the ground covered since the last pump.
	v.isNew = Kit.NewLabel(v)
	v.dots = Kit.Dots(v)
	v.introHover = CreateFrame("Frame", nil, v)
	v.introHover:SetAllPoints(v.intro)
	Kit.Hover(v.introHover, Kit.Tip(L["ui.panel.moves"], function() return Lc.pump and listLine(Lc.pump) or nil end,
		{ facts = function() return Lc.pump and introFacts(Lc.pump) or nil end }))
	-- The column, stacked at each render (renderReady): on the first pump of a session whose save the beta lost
	-- (Lc.Open, p.lostNote) where the pain flags are set again; the lead line (the quest's flavour, or why a move rides
	-- along); the goal (David, 2026-10-03: "How many reps you need to do now and what was your old record") and the
	-- record under it; what the move earns; its stop line; How. No question comes before a pump (David, 2026-10-04).
	v.lost = text(col, "bodyWhite", "LEFT", LEFT_W)
	v.lost:SetText(L["ui.panel.lost"])
	v.lead = Kit.Text(col, "bodyWhite", "LEFT", saga.soft, LEFT_W)
	v.lead:SetMaxLines(2)
	v.goal = Kit.NumberRow(col, L["ui.panel.goal"], "", WHITE)
	v.goalValue = Kit.Text(col, "goal", "LEFT", WHITE, LEFT_W) -- the goal in words, or too long beside its name
	v.note = Kit.Text(col, "bodyWhite", "LEFT", saga.note, LEFT_W)
	v.rewards = text(col, "bodyWhite", "LEFT", LEFT_W)
	v.rewards:SetMaxLines(2)
	v.rewardsHover = CreateFrame("Frame", nil, col)
	v.rewardsHover:SetAllPoints(v.rewards)
	Kit.Hover(v.rewardsHover, rewardsTip)
	v.stopLine = text(col, "bodyWhite", "LEFT", LEFT_W)
	-- its tooltip says what a click does, open or closed, and follows the click while the cursor stays on it
	v.howBar = barTip(Kit.More(col, HOW_W, L["ui.panel.how"], function() end), L["ui.panel.how"],
		L["ui.panel.how.tip"], function() return L[v.howBar:IsOpen() and "ui.panel.how.hide" or "ui.panel.how.show"] end)
	v.howBar.onToggle = function(open)
		local p = Lc.pump
		local slot = p and p.moves[p.index]
		if slot then howOpen[slot.move] = open end
		keepingTop(Panel.Render)
		if GameTooltip:IsOwned(v.howBar) then v.howBar:GetScript("OnEnter")(v.howBar) end
	end
	v.howHead = Kit.Section(col, L["ui.panel.how"]) -- the New-move view's How, open: a label with its rule
	v.guide = guideButton(col)
	v.guide.onRead = function()
		local p = Lc.pump
		local slot = p and p.moves[p.index]
		if slot and v.howBar:IsShown() and not v.howBar:IsOpen() then
			howOpen[slot.move] = true
			keepingTop(Panel.Render)
		end
	end
	v.how = guideList(col, true, LEFT_W)
	v.tipsHead = Kit.Section(col, L["ui.panel.tips"])
	v.tips = guideList(col, false, LEFT_W)
	-- The bar: the way out at the left (Bank it at one click, David, 2026-10-04; Bank the rest, or Close on the practice
	-- move), Skip on an unfinished move riding along (David, 2026-10-04: "the skip should be per unfinished move without
	-- a confirm. you just need to press 31 times then": one click and it is off the queue, nothing earned, Lc.Skip),
	-- Pause with its menu of how long and the body's two stops (Panel.PauseMenu) and Settings; Start at the right, with
	-- the tutorial's glow round it while the view waits for it.
	local foot = v.foot
	v.bank = barTip(foot:Secondary(L["ui.panel.bank"], function() Lc.Bank() end), L["ui.panel.bank"],
		L["ui.panel.bank.tip"], L["ui.panel.bank.click"])
	v.rest = barTip(foot:Secondary(L["ui.panel.rest"], function() Lc.BankRest() end), L["ui.panel.rest"],
		L["ui.panel.rest.tip"], L["ui.panel.rest.click"])
	v.close = barTip(foot:Secondary(L["ui.panel.close"], function() Lc.BankRest() end), L["ui.panel.close"],
		L["ui.panel.close.tip"], L["ui.panel.close.click"])
	v.skip = barTip(foot:Secondary(L["ui.panel.skip"], function()
		local card, p = F.card, Lc.pump
		if p then p.skipBottom, p.skipLeft = card:GetBottom() * card:GetScale(), card:GetLeft() * card:GetScale() end
		Lc.Skip()
	end), L["ui.panel.skip"], L["ui.panel.skip.tip"], L["ui.panel.skip.click"])
	v.pause = barTip(Kit.Caret(foot:Secondary(L["ui.panel.pause"], function(self) Panel.PauseMenu(self) end)),
		L["ui.panel.pause"], L["ui.panel.pause.tip"], L["ui.panel.pause.click"])
	v.settings = settingsButton(v)
	v.start = barTip(foot:Primary(L["ui.panel.start"], function() Lc.Start() end), L["ui.panel.start"], nil,
		keyHint("key.space"))
	v.glow = Kit.Glow(v.start)
end

-- The virtual partner (slice 23; Settings > Social, off until the player turns it on): on a hold, while the player is
-- not in a group, the bar of a partner who holds one hold step past the goal (ns.RecordStep), running partner_lead_s
-- ahead of the player's own bar under it, so the partner still holds as the player reaches the goal (a partner a
-- little better makes people hold on longer, Feltz 2014). A picture only: it changes no goal, record or count, and it
-- stops with the move. Both bars on one scale, to the partner's end, with the goal's mark.
local PARTNER_W, PARTNER_H = 280, 16

local function partnerBar(parent, labelKey, color)
	local art = Style.progressBar
	local b = CreateFrame("StatusBar", nil, parent)
	b:SetSize(PARTNER_W, PARTNER_H)
	b:SetStatusBarTexture(art.fill)
	b:SetStatusBarColor(unpack(color))
	b:SetMinMaxValues(0, 1)
	b.bg = b:CreateTexture(nil, "BACKGROUND")
	b.bg:SetAllPoints()
	b.bg:SetColorTexture(0, 0, 0, art.backing)
	b.mark = b:CreateTexture(nil, "OVERLAY")
	b.mark:SetColorTexture(1, 1, 1, 0.8)
	b.mark:SetSize(2, PARTNER_H)
	b.label = text(b, "labelWhite", "LEFT")
	b.label:SetPoint("LEFT", 4, 0)
	b.label:SetText(L[labelKey])
	b.time = text(b, "labelWhite", "RIGHT")
	b.time:SetPoint("RIGHT", -4, 0)
	b:Hide()
	return b
end

-- The goal the partner holds past: a hold's, with the switch on and the player not in a group; else nil.
local function partnerGoal(slot)
	if slot.kind ~= K.hold or slot.practice or not (ns.profile and ns.profile.partner) or IsInGroup() then return nil end
	return (ns.SlotGoal(slot))
end

-- The Running view, a glance from the floor: the clock in its dial, an all-out window's or a counted set's word over
-- it, the goal under it in sight through the whole set ("Goal: 23+ reps · record 22"), the stop line as the head's
-- line, the figure looping at the right; Back and Settings grey, Keep going grey after them in a steady move's last
-- seconds (slice 52: Lc.KeepShows), the guide in the bar's middle and Done (Switch sides on a side move's first side)
-- the one red.
-- The clock in its dial on `col`, its parts into `t` (the Running view's, a hold's pour's): Blizzard's titan-line ring
-- (Style.atlas.dial) dimmed as the whole time, and in its gold as the time left, which a Cooldown with the ring as its
-- swipe drains clockwise from 12 o'clock (the unreversed swipe is the part still to run). No CooldownFrameTemplate: its
-- edge is a 128-px action-button blade, a blurred arm through the numerals at this size. The ring's inner edge (124.5
-- of its 140.5-px radius) clears the clock's widest m:ss reading corner to corner by 8 units, measured in the clock's
-- own font. The dial's square stands in the column on every set, the ring only on a timed one (and a pour), so the
-- clock keeps its place.
local RING = { inner = 124.5, outer = 140.5 }
RING.tip = (RING.inner + RING.outer) / 2 / RING.outer -- the ring's line, a share of the dial's half (a pour's grains)

local function buildDial(t, col)
	t.clock = text(col, "clock", "CENTER")
	t.clock:SetText(clock(C.max_seconds))
	local w, h = t.clock:GetStringWidth(), t.clock:GetStringHeight()
	local size = math.ceil((math.sqrt(w * w + h * h) + 16) * RING.outer / RING.inner)
	t.spot = CreateFrame("Frame", nil, col)
	t.spot:SetSize(size, size)
	t.clock:SetPoint("CENTER", t.spot)
	t.dial = CreateFrame("Frame", nil, t.spot)
	t.dial:SetAllPoints()
	t.track = t.dial:CreateTexture(nil, "BACKGROUND")
	t.track:SetAllPoints()
	t.ring = CreateFrame("Cooldown", nil, t.dial)
	t.ring:SetAllPoints()
	t.ring:SetHideCountdownNumbers(true)
	t.ring:SetDrawSwipe(true)
	t.ring:SetDrawEdge(false)
	t.ring:SetDrawBling(false)
	t.dialArt = Style.SetAtlas(t.track, "dial") and Style.SetSwipeAtlas(t.ring, "dial")
	t.track:SetDesaturated(true)
	t.track:SetAlpha(0.3)
	t.ring:SetSwipeColor(1, 1, 1, 1)
end

local function buildRunning()
	local v = view("running")
	local col = v.left
	v.phase = text(col, "large", "CENTER", LEFT_W) -- the build, all out, count every rep
	buildDial(v, col)
	-- The go: the countdown's last beat on the clock's spot, the digits' grow and pop held 0.1 s, while the clock
	-- waits under it at alpha 0 and fades in once it has gone, 0.65 s into the run (Blizzard's player countdown draws
	-- nothing at its go but the sound; the word is the addon's own). The go rests at alpha 0 and the clock at 1, and a
	-- finished or stopped group gives each its own back, so the two never share the spot.
	v.go = text(col, "go", "CENTER")
	v.go:SetPoint("CENTER", v.clock)
	v.go:SetText(L["ui.panel.go"])
	v.go:SetAlpha(0)
	v.go:SetScaleAnimationMode(Enum.FontStringScaleAnimationMode.Vertex) -- a smooth grow, as Blizzard's warnings
	local ag = v:CreateAnimationGroup()
	fade(ag, v.go, 0, 1, 0, 1)
	fade(ag, v.clock, 0, 0, 0, 1)
	startNumbers(ag, { v.go }, 0.1)
	fade(ag, v.clock, 0, 1, 0.15, 3):SetSmoothing("OUT")
	v.goAnim = ag
	v.goal = text(col, "large", "CENTER", LEFT_W)
	v.goalWords, v.goalMet = "", false -- the goal line's words and whether its hold has met it (paintGoal)
	v.pace = Kit.Text(col, "bodyWhite", "CENTER", saga.note, LEFT_W) -- a steady cardio move's pace (paceLine)
	-- the partner's bar over the player's, above the goal line (a hold has no ring: the room is free)
	v.partner = partnerBar(col, "ui.partner.partner", Style.partnerBar.partner)
	v.you = partnerBar(col, "ui.partner.you", Style.partnerBar.you)
	v.partnerEnds = false -- the partner's end in seconds while it runs (renderRunning)
	v.reps = false -- the plan of the rep count the clock shows while one runs (renderRunning)
	v.back = backButton(v)
	v.settings = settingsButton(v) -- David, 2026-09-30: "open the settings during an exercise"
	-- Keep going (slice 52), Space its key: the clock runs on to the full move and the move saves without an ask
	v.keep = barTip(v.foot:Secondary(L["ui.panel.keepGoing"], function() Lc.KeepGoing() end), L["ui.panel.keepGoing"],
		nil, keyHint("key.space"))
	v.keep:Hide()
	barGuide(v)
	-- Switch sides on a side move's first side takes Space too (Panel.HandleKey)
	v.done = barTip(v.foot:Primary(L["ui.panel.done"], function() Lc.Done() end), function() return v.done:GetText() end,
		nil, keyHint(function() return Lc.pump and Lc.pump.side == 1 and "key.spaceEnter" or "key.enter" end))
end

-- The weight box (2026-10-09; David: "Let people put their exact weight (kgs) and how many reps they made"): on a
-- loaded move (ns.Loaded) under the count's dial, the kg the set was done with, pre-filled with the planned weight,
-- between - and + (Panel.StepLoad), grey, as the dial's count is between its own (slice 66). A plain EditBox, which a
-- click gives the keyboard: digits and one "." or "," with one decimal (cleanKg), three whole digits at most
-- (ns.Validate's "max" asks past load_max_kg); Enter saves the count from it and Escape lets it go (Change it on the
-- "Really?" row). It keeps the template's own font at KG_W x KG_H in every input mode, and a move without a weight
-- only hides it: with the old rep box's Game60Font (and its 100 x 60) the box crashed the client at the Result view in
-- a native stack overflow that ended in a font load, on the render that first set that font (2026-10-10, Crash_44953:
-- the first weighted Result view) and on the first Result view after the font moved to its build (2026-10-12,
-- Crash_39971: a move without a weight); every Result view before them, the box on its template font, ran.
local KG_W, KG_H = 50, 20

-- What a weight box keeps of a typed text: digits and the first "." or ",", one digit after it and three before.
local function cleanKg(s)
	local whole, sep, rest = (s:gsub("[^%d.,]", "")):match("^(%d*)([.,]?)(.*)$")
	return whole:sub(1, 3) .. sep .. (rest:gsub("%D", "")):sub(1, 1)
end
Panel.CleanKg = cleanKg -- Correct a count's weight box too (Settings.lua)

local function buildLoad(parent)
	local kg = CreateFrame("EditBox", nil, parent, "InputBoxTemplate") -- unnamed: the template keys its parts
	kg:SetAutoFocus(false)
	kg:SetMaxLetters(5)
	kg:SetJustifyH("CENTER")
	kg:SetSize(KG_W, KG_H)
	kg:SetScript("OnEnterPressed", function(self)
		-- Enter never saves an empty count: the box lets the keyboard go to the dial
		if countEmpty() or Panel.RecordInput() then self:ClearFocus() end
	end)
	kg:SetScript("OnEscapePressed", function(self)
		local p = Lc.pump
		if p and p.really then return Panel.ChangeCount() end -- Escape on the "Really?" row is Change it
		self:ClearFocus()
	end)
	kg:SetScript("OnTextChanged", function(self, userInput)
		local s = self:GetText()
		if userInput and cleanKg(s) ~= s then return self:SetText(cleanKg(s)) end -- the SetText comes back here
		-- redrawn through the count, which a heavier weight can leave over its most (Lc.CountMost's record at it)
		Panel.SetCount(F.count, F.countFresh)
	end)
	kg.label = text(parent, "large", "LEFT")
	kg.down = Kit.Secondary(parent, L["ui.result.kgDown"], 26, function() Panel.StepLoad(-1) end)
	kg.up = Kit.Secondary(parent, L["ui.result.kgUp"], 26, function() Panel.StepLoad(1) end)
	F.kg = kg
end

-- "Don't know the weight": the weight box emptied, so the set counts its reps alone (Panel.LoadValue); - or + brings
-- the planned weight back (Panel.StepLoad).
function Panel.ClearLoad()
	if not F.kg:IsShown() then return end
	F.kg:ClearFocus()
	F.kg:SetText("") -- its OnTextChanged redraws the preview and the view
end

-- The weight's - and +, the weight box's one rule on every surface (ns.LoadStep).
function Panel.StepLoad(dir)
	local slot = Lc.Slot()
	if not (slot and F.kg:IsShown()) then return end
	local gear = ns.FilterContext(ns.profile, ns.state).gear
	local load = ns.LoadStep(slot.move, Panel.LoadValue(), dir, gear, slot.load)
	F.kg:SetText(ns.Sheet.Kg(load)) -- its OnTextChanged redraws the preview
end

-- The weight's row on its holder (v.load) on a loaded move, centred: -, the box 10 after it (the template's border
-- reaches left of its rect), + 4 after it and its unit 6 after that; placed before it shows.
local function placeLoad(v, slot, loaded)
	local kg = F.kg
	for _, part in ipairs({ kg, kg.label, kg.down, kg.up }) do part:SetShown(loaded) end
	if not loaded then return end
	kg.label:SetText(L[ns.Loaded(slot.move) == "each" and "ui.result.kgEach" or "ui.result.kg"])
	for _, part in ipairs({ kg, kg.label, kg.down, kg.up }) do part:ClearAllPoints() end
	local w = kg.down:GetWidth() + 10 + KG_W + 4 + kg.up:GetWidth() + 6 + kg.label:GetStringWidth()
	kg.down:SetPoint("LEFT", v.load, "CENTER", -w / 2, 0)
	kg:SetPoint("LEFT", kg.down, "RIGHT", 10, 0)
	kg.up:SetPoint("LEFT", kg, "RIGHT", 4, 0)
	kg.label:SetPoint("LEFT", kg.up, "RIGHT", 6, 0)
end

-- The Result view, one view in seven states (renderResult): the count on the dial (slice 66), its question as the
-- title ("How many did you do?"), the move and its goal under it, the dial where the Running view's stood with the
-- count in its clock's place, - and + at its sides and under it the line at its most, the preview, the weight and the
-- other two answers, grey; "Is that right?" in the preview's place with its two answers once, Record hidden while it
-- asks; a counted set's form (slice 42; slice 53's "All out and clean?", the optional clean count on the dial under its
-- label, Yes, to the end, Form slipped and Not all out as equal rows); "Did you do the whole move?", "How close to the
-- goal pose?" and "Did you keep that pace to the end?" (slice 42) with their answers as rows, no red; the pour of what
-- was saved (below); and a set that saved no count, told once: the outcome, the day or the check it brought, what the
-- set earned as rows, the character on a win, and the way on.
-- Every move's pour (slices 65 and 66) -------------------------------------------------------------------------------

-- The pour (Lc.Save's p.pour), the Result view's state for every move that saved a count (slice 65's hold; slice 66,
-- Rapha, 2026-11-04: "so the collect animation for all moves (counts reps, seconds etc)?"): the move's figure dimmed
-- in the right column; the dial in the left with the count, its ring holding the count's share of its goal (of the
-- record without one) and emptying as one grain a unit (a rep, a second) flies from its tip into that unit's place on
-- the count bar across the card under them: on a set of reps or a hold the step's range from 0 to its top over
-- POUR.track of the track in the step's colour, a spill past the top, posts at the goal, at the record before this set
-- and at the top; on a counted set its best's post alone, on a timed move (cardio, a flow, a pose) its goal's alone.
-- The count on the dial counts each unit that lands, the gold record chip once it passes the best, the track's flash
-- and sparks at the top. The move's row is written as the pour lands (Lc.Land: its last grain, a click on the stage,
-- Next); a step it opened then plays its reveal over the figure's screen (the ladder's tiles, the light to the next
-- step, its lock springing, its burst, the new move rising with its step, name and colour), which folds into the
-- move's rows in the dial's place while the bar rolls over to the next step's. Built once on the result view
-- (buildPour), drawn by renderPour; the count step before it sets its count on the same dial (Pour.drawCount).
local POUR = {
	track = 0.86, barH = 62, trackY = 22, trackH = 12, postW = 2, postH = 26, spillGap = 4, barGap = 16, barRoll = 20,
	-- a grain a unit, `most` grains at most (each then stands for a few units); its sag under its straight line
	grainW = 3, grainH = 12, sag = 30, most = 120,
	tiles = 5, tile = 32, tileGap = 8, hero = 64, lock = 16, sparks = 24, rowStep = 0.32, rowIn = 0.3, rowFrom = 48,
}
-- the posts' words; the grains', the fill's past the top and a passed post's gold
POUR.words = { goal = "ui.pour.goal", best = "ui.pour.best", top = "ui.pour.top" }
POUR.gold = { NORMAL_FONT_COLOR:GetRGB() }
-- the pour's functions, in a table of their own: the file's main chunk is near Lua 5.1's 200 locals
local Pour = {}

-- A count of `slot`'s in its unit (slice 66): seconds as m:ss for a hold and for what saved a timed move's seconds
-- (`timed`: Lc.Save's p.pour.timed, a counted set stopped early's too), else a number (a set of reps' reps, a counted
-- set's clean count, which the dial takes).
function Pour.format(slot, n, timed)
	if timed or slot.kind == K.hold then return clock(n) end
	return tostring(n)
end

-- The goal the count bar and the dial's ring hold a count against: a timed move's planned seconds (`timed` as
-- Pour.format's), else the slot's goal (none on a max set or a counted set: ns.SlotGoal).
function Pour.goal(slot, timed)
	if timed then return slot.seconds end
	return (ns.SlotGoal(slot))
end

-- The share of the dial's ring a count `n` fills: of its goal, of the record (`best`) without one, else the whole ring
-- once it has a count; never more than the whole.
function Pour.share(slot, n, best, timed)
	local mark = Pour.goal(slot, timed) or best
	if mark and mark > 0 then return math.min(1, n / mark) end
	return n > 0 and 1 or 0
end

-- A Scale animation in `ag` on `target`, from `from` to `to` over `duration` s in its `order`.
function Pour.grow(ag, target, from, to, duration, order)
	local a = ag:CreateAnimation("Scale")
	a:SetTarget(target)
	a:SetScaleFrom(from, from)
	a:SetScaleTo(to, to)
	a:SetDuration(duration)
	a:SetOrder(order)
	return a
end

-- The count bar (the apps' CountBar; the hold bar until slice 66), `w` wide: its label over the track, the fill in the
-- step's colour, the spill past its top, the flash its top gives, and the three posts with their words (the best's
-- above, the others' under).
function Pour.buildCountBar(parent, w)
	local b = CreateFrame("Frame", nil, parent)
	b:SetSize(w, POUR.barH)
	b.trackW = math.floor(w * POUR.track)
	b.label = Kit.Caps(b, "LEFT")
	b.label:SetPoint("TOPLEFT")
	b.track = Style.Box(b)
	b.track:SetSize(b.trackW, POUR.trackH)
	b.track:SetPoint("TOPLEFT", 0, -POUR.trackY)
	b.fill = b.track:CreateTexture(nil, "ARTWORK")
	b.fill:SetPoint("TOPLEFT", 1, -1)
	b.fill:SetPoint("BOTTOMLEFT", 1, 1)
	b.spill = b:CreateTexture(nil, "ARTWORK")
	b.spill:SetHeight(POUR.trackH - 2)
	b.spill:SetPoint("LEFT", b.track, "RIGHT", POUR.spillGap, 0)
	b.spill:SetColorTexture(POUR.gold[1], POUR.gold[2], POUR.gold[3], 1)
	b.flash = b.track:CreateTexture(nil, "OVERLAY")
	b.flash:SetAllPoints()
	b.flash:SetBlendMode("ADD")
	b.flash:SetColorTexture(POUR.gold[1], POUR.gold[2], POUR.gold[3], 1)
	b.flash:SetAlpha(0)
	b.flashAnim = b.flash:CreateAnimationGroup()
	fade(b.flashAnim, b.flash, 0, 0.8, 0.2, 1)
	fade(b.flashAnim, b.flash, 0.8, 0, 0.7, 2)
	b.posts = {}
	for key in pairs(POUR.words) do
		local post = b:CreateTexture(nil, "OVERLAY")
		post:SetSize(POUR.postW, POUR.postH)
		post.words = Kit.Caps(b, "CENTER")
		if key == "best" then
			post.words:SetPoint("BOTTOM", post, "TOP", 0, 6)
		else
			post.words:SetPoint("TOP", post, "BOTTOM", 0, -6)
		end
		b.posts[key] = post
	end
	return b
end

-- The bar filled to a count of `n`: the fill to its range's end, the spill past it (as far as the bar reaches), each
-- post gold once passed (the best's once beaten).
function Pour.fillBar(b, n)
	local range = b.spec.range
	local w = (b.trackW - 2) * math.min(n, range) / range
	b.fill:SetShown(w >= 1)
	b.fill:SetWidth(math.max(1, w))
	local over = math.min(math.max(0, n - range) * b.trackW / range, b:GetWidth() - b.trackW - POUR.spillGap)
	b.spill:SetShown(over >= 1)
	b.spill:SetWidth(math.max(1, over))
	for key, post in pairs(b.posts) do
		local value = b.spec[key]
		local c = value and (key == "best" and n > value or key ~= "best" and n >= value) and POUR.gold or saga.note
		post:SetColorTexture(c[1], c[2], c[3], 1)
		post.words:SetTextColor(c[1], c[2], c[3])
	end
end

-- The bar for a step: `spec` { label, q (its colour), goal, best, top (the posts, each nil without one), range (the
-- count the track spans: the step's top, else what the bar shows), slot (the posts' unit: Pour.format), lit (the next
-- step's bar after a step-up: its edge and label in its colour) }; a post shows inside the range alone, the best's
-- only under the top. Empty.
function Pour.setBar(b, spec)
	b.spec = spec
	Kit.SetCaps(b.label, spec.label)
	local r, g, bl = ns.Sheet.QualityColor(spec.q):GetRGB()
	b.fill:SetColorTexture(r, g, bl, 1)
	local edge = spec.lit and { r, g, bl, 0.6 } or saga.listEdge
	b.track:Paint(saga.screen, edge)
	local words = spec.lit and { r, g, bl } or saga.note
	b.label:SetTextColor(words[1], words[2], words[3])
	for key, post in pairs(b.posts) do
		local value = spec[key]
		local shown = value ~= nil and value > 0 and value <= spec.range
			and (key ~= "best" or spec.top == nil or value < spec.top)
		post:SetShown(shown)
		post.words:SetShown(shown)
		if shown then
			post:ClearAllPoints()
			post:SetPoint("CENTER", b.track, "LEFT", b.trackW * value / spec.range, 0)
			Kit.SetCaps(post.words, L[POUR.words[key]]:format(Pour.format(spec.slot, value, spec.timed)))
		end
	end
	Pour.fillBar(b, 0)
end

-- Where unit `n` lands on the bar, from the track's left: its own place in the range, else in the spill.
function Pour.unitX(b, n)
	local range = b.spec.range
	local slot = b.trackW / range
	if n <= range then return (n - 0.5) * slot end
	return math.min(b.trackW + POUR.spillGap + (n - range - 0.5) * slot, b:GetWidth() - 1)
end

-- The reveal's ladder: a tile a step (the sheet's slot, smaller, on a holder so it can pop in place), its padlock,
-- flash and shockwave over it on a frame of their own; h.base the slot's scale to the tile.
function Pour.buildRevealTile(parent)
	local h = CreateFrame("Frame", nil, parent)
	h:SetSize(POUR.tile, POUR.tile)
	h.slot = ns.Sheet.SlotButton(h)
	h.base = POUR.tile / h.slot:GetWidth()
	h.slot:SetScale(h.base)
	h.slot:SetPoint("CENTER")
	h.slot:EnableMouse(false)
	local over = CreateFrame("Frame", nil, h)
	over:SetAllPoints()
	over:SetFrameLevel(h.slot:GetFrameLevel() + 3)
	h.lock = over:CreateTexture(nil, "OVERLAY")
	h.lock:SetSize(POUR.lock, POUR.lock)
	h.lock:SetPoint("CENTER")
	h.lockArt = Style.SetAtlas(h.lock, "lock")
	h.flash = over:CreateTexture(nil, "ARTWORK")
	h.flash:SetAllPoints()
	h.flash:SetBlendMode("ADD")
	h.flash:SetColorTexture(1, 1, 1, 1)
	h.flash:SetAlpha(0)
	h.shock = over:CreateTexture(nil, "ARTWORK", nil, 1)
	h.shock:SetSize(POUR.tile, POUR.tile)
	h.shock:SetPoint("CENTER")
	h.shock:SetBlendMode("ADD")
	h.shock:SetAlpha(0)
	if not Style.SetAtlas(h.shock, "ring") then h.shock:SetColorTexture(1, 1, 1, 0.4) end
	-- the lock shakes, then flies off up and fades (reveal.js: the shake 0.92 s in, the flight 0.06 s past the open)
	local open = C.reveal_open_s
	h.lockAnim = h.lock:CreateAnimationGroup()
	for i, step in ipairs({ { -14, 0.06 }, { 26, 0.075 }, { -20, 0.075 }, { 8, 0.09 } }) do
		local turn = h.lockAnim:CreateAnimation("Rotation")
		turn:SetTarget(h.lock)
		turn:SetDegrees(step[1])
		turn:SetDuration(step[2])
		turn:SetOrder(i)
		if i == 1 then turn:SetStartDelay(0.92) end
	end
	local off = h.lockAnim:CreateAnimation("Translation")
	off:SetTarget(h.lock)
	off:SetOffset(0, 34)
	off:SetDuration(0.46)
	off:SetOrder(5)
	off:SetStartDelay(open + 0.06 - 0.92 - 0.3)
	off:SetSmoothing("OUT")
	Pour.grow(h.lockAnim, h.lock, 1, 1.5, 0.46, 5):SetStartDelay(open + 0.06 - 0.92 - 0.3)
	fade(h.lockAnim, h.lock, 1, 0, 0.46, 5):SetStartDelay(open + 0.06 - 0.92 - 0.3)
	h.lockAnim:SetToFinalAlpha(true)
	-- the open: the tile's flash, its pop to 1.28 that settles at 1.1, and the shockwave out of it
	h.flashAnim = h.flash:CreateAnimationGroup()
	fade(h.flashAnim, h.flash, 0, 0.65, 0.2, 1)
	fade(h.flashAnim, h.flash, 0.65, 0, 0.45, 2)
	h.pop = h.slot:CreateAnimationGroup()
	Pour.grow(h.pop, h.slot, 1, 1.28, 0.22, 1):SetSmoothing("OUT")
	Pour.grow(h.pop, h.slot, 1, 1.1 / 1.28, 0.4, 2):SetSmoothing("IN_OUT")
	h.pop:SetScript("OnFinished", function() h.slot:SetScale(h.base * 1.1) end)
	h.shockAnim = h.shock:CreateAnimationGroup()
	Pour.grow(h.shockAnim, h.shock, 1, 2.1, 0.7, 1):SetStartDelay(0.04)
	fade(h.shockAnim, h.shock, 0.95, 0, 0.7, 1):SetStartDelay(0.04)
	return h
end

-- The step-up reveal (the apps' StepReveal) over the figure's screen: the glow and a new colour's turning rays behind
-- where the new move rises, its "Step n unlocked", its name and its colour's badge, and the ladder of tiles along the
-- rail at its foot with the light that fills the rail to the next step. Its beats are reveal.js's, scaled by none:
-- the open at reveal_open_s, the words after it.
function Pour.buildReveal(pv)
	local open = C.reveal_open_s
	local rv = CreateFrame("Frame", nil, pv)
	rv:SetSize(SCREEN_W, SCREEN_H)
	rv:SetFrameLevel(pv:GetFrameLevel() + 10)
	rv:Hide()
	rv.glow = rv:CreateTexture(nil, "BACKGROUND")
	rv.glow:SetSize(SCREEN_W, SCREEN_W * 0.6)
	rv.glow:SetPoint("CENTER", rv, "TOP", 0, -(8 + POUR.hero / 2))
	rv.glow:SetBlendMode("ADD")
	rv.glowArt = Style.SetAtlas(rv.glow, "levelGlow")
	rv.rays = rv:CreateTexture(nil, "BACKGROUND", nil, 1)
	rv.rays:SetSize(SCREEN_W * 0.8, SCREEN_W * 0.8)
	rv.rays:SetPoint("CENTER", rv.glow)
	rv.rays:SetBlendMode("ADD")
	rv.raysArt = Style.SetAtlas(rv.rays, "legendRing")
	rv.spin = rv.rays:CreateAnimationGroup()
	local turn = rv.spin:CreateAnimation("Rotation")
	turn:SetTarget(rv.rays)
	turn:SetDegrees(-360)
	turn:SetDuration(24)
	rv.spin:SetLooping("REPEAT")
	rv.cap = Kit.Caps(rv, "CENTER")
	rv.cap:SetTextColor(POUR.gold[1], POUR.gold[2], POUR.gold[3])
	rv.cap:SetPoint("TOP", rv, "TOP", 0, -(8 + POUR.hero + 6))
	rv.name = Kit.Text(rv, "move", "CENTER", nil, SCREEN_W)
	rv.name:SetMaxLines(2)
	rv.name:SetPoint("TOP", rv.cap, "BOTTOM", 0, -2)
	rv.badge = Kit.Badge(rv, "gold", "")
	rv.badge:SetPoint("TOP", rv.name, "BOTTOM", 0, -4)
	rv.ladder = CreateFrame("Frame", nil, rv)
	rv.ladder:SetSize(POUR.tiles * POUR.tile + (POUR.tiles - 1) * POUR.tileGap, POUR.tile)
	rv.ladder:SetPoint("BOTTOM", 0, 10)
	rv.rail = rv.ladder:CreateTexture(nil, "BACKGROUND")
	rv.rail:SetHeight(2)
	rv.rail:SetColorTexture(saga.listEdge[1], saga.listEdge[2], saga.listEdge[3], 1)
	rv.lit = rv.ladder:CreateTexture(nil, "BORDER")
	rv.lit:SetHeight(2)
	rv.lit:SetColorTexture(POUR.gold[1], POUR.gold[2], POUR.gold[3], 1)
	rv.lit:SetAlpha(0)
	rv.light = rv.ladder:CreateTexture(nil, "OVERLAY")
	rv.light:SetSize(20, 20)
	rv.light:SetBlendMode("ADD")
	rv.light:SetAlpha(0)
	if not Style.SetAtlas(rv.light, "levelGlow") then
		rv.light:SetColorTexture(POUR.gold[1], POUR.gold[2], POUR.gold[3], 0.6)
	end
	rv.holders = {}
	for k = 1, POUR.tiles do
		local h = Pour.buildRevealTile(rv.ladder)
		h:SetPoint("LEFT", (k - 1) * (POUR.tile + POUR.tileGap), 0)
		rv.holders[k] = h
	end
	-- the light leaves the current step's edge 0.56 s in and fills the rail to the next one in 0.44 s
	rv.litAnim = rv.lit:CreateAnimationGroup()
	fade(rv.litAnim, rv.lit, 0, 0, 0.56, 1)
	fade(rv.litAnim, rv.lit, 1, 1, 0.44, 2)
	local stretch = rv.litAnim:CreateAnimation("Scale")
	stretch:SetTarget(rv.lit)
	stretch:SetScaleFrom(0.01, 1)
	stretch:SetScaleTo(1, 1)
	stretch:SetOrigin("LEFT", 0, 0)
	stretch:SetDuration(0.44)
	stretch:SetOrder(2)
	stretch:SetSmoothing("IN_OUT")
	rv.litAnim:SetToFinalAlpha(true)
	rv.lightAnim = rv.light:CreateAnimationGroup()
	fade(rv.lightAnim, rv.light, 0, 0, 0.56, 1)
	rv.lightMove = rv.lightAnim:CreateAnimation("Translation")
	rv.lightMove:SetTarget(rv.light)
	rv.lightMove:SetDuration(0.44)
	rv.lightMove:SetOrder(2)
	rv.lightMove:SetSmoothing("IN_OUT")
	fade(rv.lightAnim, rv.light, 0, 1, 0.07, 2)
	fade(rv.lightAnim, rv.light, 1, 0, 0.1, 3)
	Pour.grow(rv.lightAnim, rv.light, 1, 1.8, 0.1, 3)
	-- the frame and the ladder come in, then past the open the glow, the rays, the words in turn
	rv.plays = { Kit.Entrance(rv, 0, 0.22), Kit.Entrance(rv.ladder, 0, 0.34, nil, -30),
		Kit.Entrance(rv.glow, open + 0.08, 0.7, nil, nil, 0.6), Kit.Entrance(rv.cap, open + 0.3, 0.3, nil, -8),
		Kit.Entrance(rv.name, open + 0.38, 0.46, nil, -10), Kit.Entrance(rv.badge, open + 0.52, 0.42, nil, nil, 0.6),
		rv.litAnim, rv.lightAnim }
	rv.raysIn = Kit.Entrance(rv.rays, open + 0.14, 0.9)
	rv.out = rv:CreateAnimationGroup()
	fade(rv.out, rv, 1, 0, 0.3, 1)
	rv.out:SetToFinalAlpha(true)
	rv.out:SetScript("OnFinished", function() rv:Hide() end)
	pv.reveal = rv
	-- the new move's figure, over the reveal and apart from it: it rises past the open and, as the reveal folds, flies
	-- into the unlock row's picture
	local hero = ns.Figure.Create(pv, POUR.hero)
	hero.frame:EnableMouse(false)
	hero.frame:SetFrameLevel(rv:GetFrameLevel() + 5)
	hero.frame:SetPoint("CENTER", rv.glow)
	hero.caption:Hide()
	hero.frame:Hide()
	pv.hero = hero
	pv.heroIn = Kit.Entrance(hero.frame, open + 0.16, 0.56, nil, -26, 0.84)
	pv.heroFly = hero.frame:CreateAnimationGroup()
	pv.heroMove = pv.heroFly:CreateAnimation("Translation")
	pv.heroMove:SetTarget(hero.frame)
	pv.heroMove:SetDuration(0.56)
	pv.heroMove:SetOrder(1)
	pv.heroMove:SetSmoothing("IN_OUT")
	pv.heroShrink = Pour.grow(pv.heroFly, hero.frame, 1, 0.5, 0.56, 1)
	pv.heroShrink:SetSmoothing("IN_OUT")
	pv.heroFly:SetScript("OnFinished", function() Pour.landHero(pv) end)
end

-- The move's rows (the dial's place once the pour landed): the move with its count, its record and its day at the
-- top; the step it opened; the stats it raised; its XP. Each comes in from the right in turn (row.enter), and after
-- them the lines the saved set brought, which a set saved with no count tells on its own view (slice 66: a loaded
-- move's next weight, nextLoadLine; a cardio day, a pose's stretch checks, afterLines).
function Pour.buildPourRows(pv)
	local list = Kit.List(pv, LEFT_W, LEFT_W)
	list:Hide()
	pv.rows = list
	pv.moveRow = Kit.Row(list, { picture = true, name = "", line = "", value = "", below = true })
	pv.moveRow.lead = Kit.Badge(pv.moveRow, "gold", L["ui.pour.record"])
	pv.unlockRow = Kit.Row(list, { picture = true, name = "", line = "", below = true })
	pv.statRows = {}
	for i = 1, 3 do pv.statRows[i] = Kit.Row(list, { icon24 = Style.statIcon.strength, name = "" }) end
	pv.xpRow = Kit.Row(list, { icon24 = Style.xpIcon, name = "" })
	for _, row in ipairs(list.rows) do row.enter = Kit.Entrance(row, 0, POUR.rowIn, POUR.rowFrom) end
	pv.after = Kit.Text(pv, "bodyWhite", "CENTER", nil, LEFT_W)
	pv.after:Hide()
	pv.after.enter = Kit.Entrance(pv.after, 0, POUR.rowIn, POUR.rowFrom)
end

-- The pour's frames on the result view `v` (v.pour), over its columns and the figure: the dial with its clock and the
-- record chip (the count step's dial too, Pour.drawCount), the two count bars (this step's and, after a step-up, the
-- next one's rolling in), the rows, the reveal and its figure, a layer for the grains and sparks over all of it, and
-- the stage's click (skipPour).
function Pour.buildPour(v)
	local pv = CreateFrame("Frame", nil, v)
	pv:SetAllPoints()
	pv:SetFrameLevel(v:GetFrameLevel() + 20)
	pv:Hide()
	v.pour = pv
	buildDial(pv, pv)
	pv.chip = Kit.Badge(pv, "gold", L["ui.pour.record"])
	pv.chip:SetPoint("TOP", pv.clock, "BOTTOM", 0, -8)
	pv.bars = { Pour.buildCountBar(pv, TM.TEXT), Pour.buildCountBar(pv, TM.TEXT) }
	local old, new = pv.bars[1], pv.bars[2]
	old.leave = old:CreateAnimationGroup()
	local up = old.leave:CreateAnimation("Translation")
	up:SetTarget(old)
	up:SetOffset(0, POUR.barRoll)
	up:SetDuration(0.38)
	up:SetSmoothing("IN")
	fade(old.leave, old, 1, 0, 0.38, 1)
	old.leave:SetToFinalAlpha(true)
	old.leave:SetScript("OnFinished", function() old:Hide() end)
	new.enter = Kit.Entrance(new, 0.38, 0.44, nil, -POUR.barRoll)
	Pour.buildPourRows(pv)
	Pour.buildReveal(pv)
	pv.layer = CreateFrame("Frame", nil, pv)
	pv.layer:SetAllPoints()
	pv.layer:SetFrameLevel(pv:GetFrameLevel() + 40)
	pv.grains = {}
	pv.sparks = {}
	for i = 1, POUR.sparks do
		local s = pv.layer:CreateTexture(nil, "OVERLAY")
		s:SetSize(8, 8)
		s:SetBlendMode("ADD")
		s:SetAlpha(0)
		if not Style.SetAtlas(s, "levelGlow") then s:SetColorTexture(1, 1, 1, 1) end
		s.anim = s:CreateAnimationGroup()
		s.fly = s.anim:CreateAnimation("Translation")
		s.fly:SetTarget(s)
		s.fly:SetSmoothing("OUT")
		s.parts = { s.fly, Pour.grow(s.anim, s, 1, 0.4, 1, 1), fade(s.anim, s, 1, 0, 1, 1) }
		pv.sparks[i] = s
	end
	pv.hit = CreateFrame("Frame", nil, pv)
	pv.hit:SetFrameLevel(pv:GetFrameLevel() + 50)
	pv.hit:EnableMouse(true)
	pv.hit:SetScript("OnMouseUp", function() Pour.skipPour() end)
	-- every group the pour plays, which stopPour stops
	pv.anims = { old.leave, new.enter, old.flashAnim, new.flashAnim, pv.reveal.out, pv.reveal.spin, pv.reveal.raysIn,
		pv.heroIn, pv.heroFly }
	for _, ag in ipairs(pv.reveal.plays) do pv.anims[#pv.anims + 1] = ag end
	for _, h in ipairs(pv.reveal.holders) do
		for _, ag in ipairs({ h.lockAnim, h.flashAnim, h.pop, h.shockAnim }) do pv.anims[#pv.anims + 1] = ag end
	end
	for _, row in ipairs(pv.rows.rows) do pv.anims[#pv.anims + 1] = row.enter end
	pv.anims[#pv.anims + 1] = pv.after.enter
	for _, s in ipairs(pv.sparks) do pv.anims[#pv.anims + 1] = s.anim end
end

-- A grain of the pour (made at its first use): a gold sliver on the layer that waits its start, flies to the middle
-- of its path, sagging, and on into its unit's place, dimming as it lands (landGrain).
function Pour.grain(pv, i)
	local g = pv.grains[i]
	if g then return g end
	g = pv.layer:CreateTexture(nil, "OVERLAY")
	g:SetSize(POUR.grainW, POUR.grainH)
	g:SetColorTexture(POUR.gold[1], POUR.gold[2], POUR.gold[3], 1)
	g:SetAlpha(0)
	g.anim = g:CreateAnimationGroup()
	g.wait = fade(g.anim, g, 0, 0, 0, 1)
	g.show = fade(g.anim, g, 1, 1, 0, 2)
	g.out = g.anim:CreateAnimation("Translation")
	g.out:SetTarget(g)
	g.out:SetOrder(2)
	g.out:SetSmoothing("IN")
	g.back = g.anim:CreateAnimation("Translation")
	g.back:SetTarget(g)
	g.back:SetOrder(3)
	g.dim = fade(g.anim, g, 1, 0.3, 0, 3)
	g.anim:SetScript("OnFinished", function() Pour.landGrain(pv, g.n) end)
	pv.grains[i] = g
	return g
end

local WHOLE_W = Kit.TIER.S.TEXT -- the whole move's two answers, centred: the dialog's width

local COUNT_STEP_W = 28 -- the dial's - and +, beside its ring

local function buildResult()
	local v = view("result")
	local col = v.left
	-- the count on the dial (slice 66; Rapha, 2026-11-04: "maybe just in the circle adjust the times and reps etc.
	-- instead of new screen (its bad ux anyways) then it collect?"): the pour's dial (Pour.buildPour), in its room at
	-- the column's top where the Running view's stood, the count in its clock's place, - and + at its sides and the
	-- wheel over it (Panel.StepCount), the digits, Backspace and the arrows through the panel's key frame
	-- (Panel.HandleKey); under it the line that says why + stopped while the count stands at its most (Lc.CountMost)
	Pour.buildPour(v)
	local spot = v.pour.spot
	v.dialRoom = CreateFrame("Frame", nil, col)
	v.dialRoom:SetSize(spot:GetWidth(), spot:GetHeight())
	v.less = Kit.Secondary(col, L["ui.result.kgDown"], COUNT_STEP_W, function() Panel.StepCount(-1) end)
	v.less:SetPoint("RIGHT", v.dialRoom, "LEFT", -4, 0)
	v.more = Kit.Secondary(col, L["ui.result.kgUp"], COUNT_STEP_W, function() Panel.StepCount(1) end)
	v.more:SetPoint("LEFT", v.dialRoom, "RIGHT", 4, 0)
	spot:EnableMouseWheel(true)
	spot:SetScript("OnMouseWheel", function(_, delta) Panel.StepCount(delta > 0 and 1 or -1) end)
	v.cap = Kit.Text(col, "bodyWhite", "CENTER", saga.note, LEFT_W)
	-- the weight's row on a loaded move (placeLoad)
	v.load = CreateFrame("Frame", nil, col)
	v.load:SetSize(LEFT_W, Kit.BUTTON_H)
	buildLoad(col)
	-- under it while it holds a weight, grey: "Don't know the weight" (David, 2026-11-06: "there should be a "dont know
	-- weight" option here"), which empties it (Panel.ClearLoad)
	v.kgNoneRow = answerRow(col, LEFT_W)
	v.kgNone = answerButton(v.kgNoneRow, L["ui.result.kgNone"], function() Panel.ClearLoad() end)
	-- the live preview under the dial once a number is in ("23 reps: +40 XP, new record")
	v.preview = text(col, "bodyWhite", "CENTER", LEFT_W)
	-- a counted set's form (slice 42, 2026-10-16; David: "The count in a time is not accurate enough and individual"):
	-- the optional clean count's label under the dial, the answers to "All out and clean?" (slice 53: Yes, to the end,
	-- Form slipped and Not all out) as equal rows of one list, none red nor pushed, a click on any once they have
	-- settled (answerSettled) saving the set with the count the dial holds (Panel.Counted); the digits type the count
	v.cleanLabel = Kit.Text(col, "bodyWhite", "CENTER", saga.note, LEFT_W)
	v.cleanLabel:SetText(L["ui.result.cleanLabel"])
	v.forms = Kit.List(col, LEFT_W, LEFT_W)
	for i, key in ipairs({ "ui.result.cleanEnd", "ui.result.slipped", "ui.result.notAllOut" }) do
		local row = answerRowOf(v.forms, L[key], settledAnswer(function()
			Panel.Counted(FORM_ANSWERS[i])
		end), { chevron = true })
		row.chevron:Hide()
		row.button:HookScript("OnEnter", function() row.chevron:Show() end)
		row.button:HookScript("OnLeave", function() row.chevron:Hide() end)
	end
	-- the other two answers to the count, grey and in the body (they answer the question; the bar's left holds the way
	-- back alone): done without a count, could not do one
	v.answers = answerRow(col, LEFT_W)
	v.empty = answerButton(v.answers, L["ui.result.empty"], settledAnswer(function() Lc.EmptyDone() end))
	v.couldNot = answerButton(v.answers, L["ui.result.couldNot"], settledAnswer(function() Lc.CouldNot() end))
	-- "Really?": a count far above the record (or a first max set far above the rung's top) or far below it (ns.Validate)
	-- asks once, in the preview's place with its two answers (David, 2026-10-03: asked to type it again, "Instead just
	-- have a confirm popup"): Yes records it as typed, marked confirmed, and Enter does the same; Change it, or Escape,
	-- empties the box. No "Beat it" button fills the goal in (David, 2026-10-03: "remove beat it button. Record is
	-- enough").
	v.really = text(col, "bodyWhite", "CENTER", LEFT_W)
	v.reallyRow = answerRow(col, LEFT_W)
	v.reallyYes = answerButton(v.reallyRow, "", settledAnswer(function() Panel.RecordInput() end))
	v.reallyChange = answerButton(v.reallyRow, L["ui.result.reallyChange"], function() Panel.ChangeCount() end)
	-- "Did you do the whole move?" (Lc.Whole; David, 2026-10-05: "If its an extreme number like 5s when 45s cooldown
	-- runs just ask if it was fully done. Do not just record 5s"): a timed move ended before timed_ask_share of its
	-- seconds, the clock said in the head's line, its two answers as rows. It takes a click once it has settled: Enter
	-- and Escape, which end a running move, never answer it, and a click before answerSettled does nothing, so a second
	-- press after Done records no guess (Panel.HandleKey, Lc.Escape).
	v.whole = Kit.List(col, WHOLE_W, WHOLE_W)
	answerRowOf(v.whole, L["ui.result.wholeYes"], settledAnswer(function() Lc.Whole(true) end))
	answerRowOf(v.whole, L["ui.result.wholeNo"], settledAnswer(function() Lc.Whole(false) end))
	-- "How close to the goal pose?" (Lc.Mark; David, 2026-10-09: "Reached the mark was very unclear. Think through a
	-- better ask. Maybe 4 options or so"; 2026-10-14: "whats to achieve, the "mark" (please rename for clarity), is not
	-- very clear. think it through and improve"): a pose whose slot asks (its family's mark, below the top or on an
	-- orange step no hold has earned, or a pose a step names as its proof) ran its time, its key pose in the right column
	-- captioned "Goal pose" (Figure.lua's "key" mode) and its goal sentence the head's line; the four answers from
	-- Perfect down as rows, their words alone as the apps' and the watches', each after the key of its code ("4.
	-- Perfect" to "1. Not yet", Panel.HandleKey: 2026-10-31, a stray 1 is the lowest answer), the light and a chevron
	-- under the cursor. A click or a digit answers it once it has settled (answerSettled), never Enter or Escape.
	v.marks = Kit.List(col, LEFT_W, LEFT_W)
	local words = { perfect = L["ui.result.markPerfect"], good = L["ui.result.markGood"], okay = L["ui.result.markOkay"],
		notyet = L["ui.result.markNotYet"] }
	for _, key in ipairs(E.markKeys) do
		local code = E.mark[key]
		local row = answerRowOf(v.marks, L["ui.result.markChoice"]:format(code, words[key]), settledAnswer(function()
			Lc.Mark(code)
		end), { chevron = true })
		row.chevron:Hide()
		row.button:HookScript("OnEnter", function() row.chevron:Show() end)
		row.button:HookScript("OnLeave", function() row.chevron:Hide() end)
	end
	-- "Did you keep that pace to the end?" (Lc.Pace; slice 42, 2026-10-17: the cadence lines set the pace by the talk
	-- test, so the ask is whether the player kept it): after a steady move the steady ledger asks after (slice 52,
	-- ns.LedgerAsk) that ran its whole time, the move's key pose held still in the right column, its three answers as
	-- rows in the ask's order (ns.Enum.paceAnswers, keys 1-3), the light and a chevron under the cursor; a click or a
	-- digit answers it once it has settled (answerSettled), never Enter or Escape
	local paceWords = { easily = "ui.result.paceEasily", just = "ui.result.paceJust", eased = "ui.result.paceEased" }
	v.paces = Kit.List(col, LEFT_W, LEFT_W)
	for i, key in ipairs(E.paceAnswers) do
		local code = E.pace[key]
		local row = answerRowOf(v.paces, L["ui.result.markChoice"]:format(i, L[paceWords[key]]),
			settledAnswer(function() Lc.Pace(code) end), { chevron = true })
		row.chevron:Hide()
		row.button:HookScript("OnEnter", function() row.chevron:Show() end)
		row.button:HookScript("OnLeave", function() row.chevron:Hide() end)
	end
	-- "Could you have held longer?" (Lc.Longer; slice 52: a hold stopped at its goal says nothing of what was left, so
	-- the Stamina Ledger asks): after a fresh hold set near its goal on a hold ladder's step (ns.HoldAsk), the move's
	-- key pose held still in the right column, its three answers as rows in their codes' order (ns.Enum.longer, keys
	-- 1-3), the light and a chevron under the cursor; a click or a digit answers it once it has settled
	-- (answerSettled), never Enter or Escape
	v.longers = Kit.List(col, LEFT_W, LEFT_W)
	local longer = { no = L["ui.result.longerNo"], little = L["ui.result.longerLittle"], lot = L["ui.result.longerLot"] }
	for _, key in ipairs(E.longerKeys) do
		local code = E.longer[key]
		local row = answerRowOf(v.longers, L["ui.result.markChoice"]:format(code, longer[key]), settledAnswer(function()
			Lc.Longer(code)
		end), { chevron = true })
		row.chevron:Hide()
		row.button:HookScript("OnEnter", function() row.chevron:Show() end)
		row.button:HookScript("OnLeave", function() row.chevron:Hide() end)
	end
	-- the recorded set: the outcome told once (on the gold bar and in gold on a record), its XP line by line under the
	-- cursor (the move, the wins, the day's first pump, the streak, rested XP); the next weight on a loaded move; the
	-- cardio day or the stretch check the set brought (afterLines)
	v.outcome = text(col, "large", "CENTER", LEFT_W)
	v.bar = col:CreateTexture(nil, "ARTWORK")
	v.bar:SetSize(LEFT_W, 40)
	v.bar:SetPoint("CENTER", v.outcome, "CENTER", 0, 0)
	Style.SetAtlas(v.bar, "recordBar")
	v.outcomeHover = CreateFrame("Frame", nil, col)
	v.outcomeHover:SetAllPoints(v.outcome)
	Kit.Hover(v.outcomeHover, function(tooltip)
		local p = Lc.pump
		GameTooltip_SetTitle(tooltip, L["ui.rewards.xpTitle"])
		for _, e in ipairs(p and p.last and p.last.earned or {}) do
			if e.kind == "xp" then
				local line = e.rested > 0 and L["ui.rewards.xpLineRested"]:format(e.amount, e.base, e.rested)
					or L["ui.rewards.xpLine"]:format(e.amount, e.base)
				tooltip:AddDoubleLine(L["xp.reason." .. e.reason], line, 1, 1, 1, 1, 1, 1)
			end
		end
	end)
	v.nextLoad = text(col, "bodyWhite", "CENTER", LEFT_W)
	v.after = text(col, "bodyWhite", "CENTER", LEFT_W)
	-- what the set earned, one row a chip (renderRewardRows): an earned one in gold with the green check, flashing once
	-- (bags-glow-flash, ADD, alpha 1 to 0 over 0.6 s), a missed one with a dash, its name grey and its icon greyed
	-- (never the colour alone)
	v.rewardsHead = Kit.Section(col, L["ui.rewards.title"])
	v.rewardList = Kit.List(col, LEFT_W, LEFT_W)
	v.rewardRows = {}
	for i = 1, 4 do
		local check = CreateFrame("Frame")
		check:SetSize(18, 18)
		check.mark = check:CreateTexture(nil, "ARTWORK")
		check.mark:SetAllPoints()
		Style.SetMark(check.mark, true)
		local row = Kit.Row(v.rewardList, { icon24 = Style.statIcon.strength, name = "", line = "", below = true,
			control = check })
		row.check = check
		row.flash = row:CreateTexture(nil, "OVERLAY")
		row.flash:SetAllPoints()
		row.flash:SetBlendMode("ADD")
		row.flash:SetAlpha(0)
		if not Style.SetAtlas(row.flash, "earned") then row.flash:SetColorTexture(1, 0.82, 0, 0.6) end
		row.anim = row.flash:CreateAnimationGroup()
		fade(row.anim, row.flash, 1, 0, 0.6, 1)
		v.rewardRows[i] = row
	end
	-- the bar: Back until the count is saved; Record, the way on while the count is entered in every input mode, which
	-- waits for a number; Next (Finish on the last move, "Back to your tree" for a move started there) once recorded
	v.back = backButton(v)
	v.record = barTip(v.foot:Primary(L["ui.result.record"], function() Panel.RecordInput() end), L["ui.result.record"],
		nil, keyHint("key.enter"))
	v.next = barTip(v.foot:Primary(L["ui.result.next"], function()
		local p = Lc.pump
		if p and p.pour and not answerSettled() then return end -- the click that saved the hold never moves on
		if p and p.committed and p.moves[p.index].practice then
			Lc.Finish("practice")
		else
			Lc.Next()
		end
	end), function() return v.next:GetText() end, nil, keyHint("key.enter"))
end

-- The Pause menu's "I feel unwell" (Lc.UnwellAsk), in the care tone (no red, no gold, no celebration): its title, then
-- whether to pause and how long, the pause's own choices as rows (Sheet.PauseOptions, "While grouped" first in a mixed
-- party; until 04:00 also locks every pump, Lc.PausePump, and its name says so: "Until 04:00 everywhere"), and Back to
-- the move; nothing happens until a choice, and a click on one pauses at once (this view is the ask).
local CHOICES_W = Kit.TIER.S.TEXT

local function buildUnwellAsk()
	local v = view("unwellAsk")
	setHead(v, L["ui.unwellAsk.title"])
	placeColumn(v, false, TM.TEXT)
	v.ask = text(v.left, "bodyWhite", "CENTER", TM.TEXT)
	v.ask:SetText(L["ui.unwellAsk.how"])
	v.choices = Kit.List(v.left, CHOICES_W, CHOICES_W)
	local function choice(mode, words)
		return answerRowOf(v.choices, words, function() Lc.PausePump(mode, true) end, { font = "rowName" })
	end
	v.group = choice(ns.PAUSE.group, L["hud.pause.group"])
	for _, option in ipairs(ns.Sheet.PauseOptions()) do
		choice(option[1], option[1] == ns.PAUSE.day and L["ui.unwellAsk.day"]:format(C.day_offset_h) or option[2])
	end
	local back = BACK or L["ui.panel.back"]
	v.back = barTip(v.foot:Secondary(back, function() Lc.BackToReady() end), back, L["ui.unwellAsk.back.tip"],
		L["ui.unwellAsk.back.click"])
end

function Panel.Build()
	if F.dim then return end
	-- The dim takes the mouse once, so both-button run, move-and-steer and click-to-move never reach the world.
	local dim = CreateFrame("Frame", "QuestPumpPanel", UIParent)
	dim:SetAllPoints(UIParent)
	dim:SetFrameStrata("FULLSCREEN_DIALOG")
	dim:EnableMouse(true)
	dim:Hide()
	dim.bg = dim:CreateTexture(nil, "BACKGROUND")
	dim.bg:SetAllPoints()
	dim.bg:SetColorTexture(0, 0, 0, C.dim_alone)
	dim.shadow = CreateFrame("Frame", nil, dim, "ShadowOverlayTemplate")
	dim.shadow:SetAllPoints()
	F.dim = dim
	-- The card: the kit's window at its medium tier, in the dim (it hides with it, at the dim's strata), never raised
	-- by a click (`toplevel` false: alone in its dim it needs no raise, and raised it could climb over ns.Menu's levels
	-- in the same strata, Pause's Something hurts and I feel unwell among them); its X on a Ready view hides the pump
	-- for a few seconds (Lc.Snooze), and it comes back once the character stands still.
	local card = Kit.Window(nil, TM, "", { strata = "FULLSCREEN_DIALOG", toplevel = false, onClose = function()
		Style.Play("close")
		Lc.Snooze()
	end })
	card:SetParent(dim)
	card:Show() -- the kit builds a window hidden; this one shows and hides with the dim
	F.card = card
	card.snooze = barTip(card.close, L["ui.panel.snooze"], L["ui.panel.snooze.tip"]:format(C.snooze_s),
		L["ui.panel.snooze.click"])
	-- In the Streamer layout the card moves by its title (slice 23): dragged, its top-left corner is kept in the profile
	-- (cardPos, in UIParent's units: the card's own are scaled by Kit.Fit), clamped to the screen; a right-click on the
	-- title puts it back in the corner. The standard layout's card keeps its place, and its title takes no drag.
	card:SetMovable(true)
	local grip = CreateFrame("Frame", nil, card)
	Kit.Hover(grip, Kit.Tip(L["ui.panel.grip"], L["ui.panel.grip.tip"]), "ANCHOR_TOP")
	grip:SetPoint("TOPLEFT")
	grip:SetPoint("TOPRIGHT", -28, 0) -- clear of the X
	grip:SetHeight(GRIP_H)
	grip:RegisterForDrag("LeftButton")
	grip:SetScript("OnDragStart", function() if not InCombatLockdown() then card:StartMoving() end end)
	-- a drag between two Skips moves the place the next Ready view keeps (placeCard), a reset forgets it
	grip:SetScript("OnDragStop", function()
		card:StopMovingOrSizing()
		local p, scale = Lc.pump, card:GetScale()
		if ns.profile and Panel.Streamer() then
			ns.profile.cardPos = { x = card:GetLeft() * scale, y = card:GetTop() * scale - UIParent:GetHeight() }
			ns.Touch()
		end
		if p and p.skipBottom then p.skipBottom, p.skipLeft = card:GetBottom() * scale, card:GetLeft() * scale end
		if p then placeCard(p) end
	end)
	grip:SetScript("OnMouseUp", function(_, mouse)
		if mouse ~= "RightButton" or not ns.profile or ns.profile.cardPos == nil then return end
		ns.profile.cardPos = nil
		ns.Touch()
		local p = Lc.pump
		if p then
			p.skipBottom, p.skipLeft = nil, nil
			placeCard(p)
		end
	end)
	grip:Hide()
	card.grip = grip
	-- The only keyboard frame, a child of the dim: it hides with it, and in combat by itself.
	local keys = CreateFrame("Frame", nil, dim)
	keys:SetAllPoints()
	keys:EnableKeyboard(true)
	if not InCombatLockdown() then keys:SetPropagateKeyboardInput(true) end -- Render sets it again before showing
	keys:SetScript("OnKeyDown", onKeyDown)
	keys:SetScript("OnKeyUp", onKeyUp)
	F.keys = keys
	buildSide()
	buildReady()
	local countdown = view("countdown")
	buildCountdown(countdown)
	countdown.back = backButton(countdown)
	barGuide(countdown)
	buildRunning()
	buildResult()
	buildUnwellAsk()
	Panel.BuildStopCard()
	Panel.BuildCheckin()
	Panel.ReadBindings()
end

-- Rendering --------------------------------------------------------------------------------------------------------

-- The view's height to the card's bottom: its content's `bottom`, the room under it and its bar (Kit.Fit's rule).
local function heightOf(v, bottom)
	return bottom + 20 + v.foot:Lay()
end

-- The card fitted round the view on (Kit.Fit: its height, scaled past its cap) and placed (placeCard).
local function fitCard(v, bottom)
	Kit.Fit(F.card, bottom, v.foot)
	placeCard(Lc.pump)
end

-- The move figure in the right column: `mode` "loop", "new" or "key" (Figure.lua), mirrored on a side move's other
-- side, and in the countdown and the run on `goClock`, Lc.SinceGo (View:Play), so the loop's start pose lands on the
-- go and each loop ends as the voice counts its rep (Speech.lua), and a play after a hide keeps the phase. A re-render
-- of the same move, mode, side and clock leaves it playing: the countdown and the run share one play, so nothing jumps
-- at the go, and Start plays it anew on the clock. A swap plays the move swapped in; the next move's sheets preload
-- while this one shows.
local function showFigure(p, slot, mode, goClock)
	local fig = F.figure
	fig:Fade(nil) -- a hold's pour dims it (renderPour), every other view shows it whole
	fig.frame:SetScale(1)
	local flip = p.side == 2 and (p.state == "countdown" or p.state == "running") -- a Ready view starts on side one
	local id = tostring(p) .. ":" .. p.index .. ":" .. slot.move .. ":" .. mode .. ":" .. tostring(flip) .. ":"
		.. tostring(goClock ~= nil)
	fig.frame:Show()
	if F.figureFor ~= id then
		F.figureFor = id
		fig:Play(slot.move, mode, flip, nil, goClock)
	end
	local nxt = p.moves[p.index + 1]
	if nxt then fig:Preload(nxt.move) end
end

local function hideFigure()
	F.figure.frame:Hide()
end

-- The right column under the view's head, at the split's right; hidden on a view with no picture.
local function placeSide(v)
	F.side:ClearAllPoints()
	F.side:SetPoint("TOPLEFT", SIDE_X, -Kit.Top(v))
	F.side:Show()
end

local function hideSide()
	hideFigure()
	F.side:Hide()
end

-- The figure's screen in the right column with the figure in `mode` (showFigure), the New-move caption (and the fault
-- pair) or the goal pose's caption under it; answers the column's height.
local function sideFigure(v, p, slot, mode, goClock)
	F.screen:Show()
	F.modelScreen:Hide()
	F.swap:Hide()
	showFigure(p, slot, mode, goClock)
	placeSide(v)
	return SCREEN_H + ((mode == "new" or mode == "key") and 6 + F.figure:CaptionHeight() or 0)
end

-- The swap row under the figure, `y` down the right column (David, 2026-10-01: "in the corner where it fits nicely show
-- the name of the other excericse you can swap to"; 2026-10-05: "I feel we should add it back below"): the first
-- offer the engine has (the backup, the same level done another way, "Closest" a rung below; on a first appearance the
-- easier version first; after a swap the move it replaced, one click there and one click back), and with more than
-- one offer the arrow at its right with the rest (the one place to swap since 2026-10-16, where a Swap button and an
-- Easier version button stood beside it). Answers the room it takes under `y`.
local function renderSwap(new, y)
	local options = Lc.SwapOptions()
	local which, n = nil, 0
	for _, key in ipairs(new and SWAP_FIRST or SWAP_ORDER) do
		if options[key] then
			n = n + 1
			which = which or key
		end
	end
	F.swap:SetShown(which ~= nil)
	if not which then return 0 end
	local tile = options[which]
	local row = swapRow(tile.move)
	row.which, row.option = which, tile
	Kit.Choose(row, true, true)
	row.name:SetText(swapWords(which, tile))
	Kit.SetIcon(row.icons[1], tile.move)
	row.menu:SetShown(n > 1)
	local gap = new and 8 or 10
	F.swap:ClearAllPoints()
	F.swap:SetPoint("TOPLEFT", F.side, "TOPLEFT", 0, -(y + gap))
	return gap + Kit.LayList(F.swap)
end

-- The goal row at `y` in the column: "Your goal" at the left and the count to reach at the right in the big number,
-- white (decision G1: gold once met, which only the Running view's line can be: paintGoal), its unit beside it; where
-- the two do not fit the column side by side, or the goal is words ("As many as you can"), the goal stands under its
-- name in the goal font. Answers the row's height.
local function layGoal(v, y, value, unit)
	local g = v.goal
	g.number:SetText(value)
	g.word:SetText(unit)
	local fits = value:find("^%d") ~= nil
		and g.name:GetStringWidth() + 12 + g.number:GetStringWidth() + 6 + g.word:GetStringWidth() <= LEFT_W
	g.number:SetShown(fits)
	g.word:SetShown(fits)
	v.goalValue:SetShown(not fits)
	if fits then return Kit.LayNumber(g, y, value) end
	v.goalValue:SetText(join(value, unit, " "))
	return Kit.Stack(v.left, y, { { g.name, 0 }, { v.goalValue, 2 } }) - y
end

-- The Ready view's bar (buildReady): the way out (Bank it on a turn-in's pump; Bank the rest, or Close on the practice
-- move, on one the player asked for and on a turn-in's once only unfinished moves are left, the banner then paying what
-- was done, Lc.Bank), Skip on an unfinished move riding along (a move started from the Talents tab pays one as its own
-- set, with no Skip and Close), Pause on every pump but the practice move, which stores nothing (David, 2026-10-04:
-- "Something hurts" and "I feel unwell" reachable in a pump the player asked for too) and Settings. The kit's fit rule
-- (Kit.Footer) shrinks the left buttons to their words where they meet Start, then folds Settings into More; Bank it,
-- Bank the rest, Skip and Pause never fold.
local function renderBar(p, slot)
	local v = F.views.ready
	local player = p.kind == "demand" or p.kind == "practice"
	local rest = player or (slot.owed ~= nil and next(p.results) ~= nil and Lc.OnlyRidersLeft(p))
	local close = rest and not (p.kind ~= "practice" and slot.owed and not p.chosen)
	v.bank:SetShown(not rest)
	v.rest:SetShown(rest and not close)
	v.close:SetShown(close)
	v.skip:SetShown(slot.owed ~= nil and not p.chosen)
	v.pause:SetShown(p.kind ~= "practice")
	v.settings:SetShown(true)
end

-- The Ready view (play SPEC 2 and 3; boards ready and ready-new): the move as the title, where it stands in the pump
-- under it; the column (the lead line, the goal row and the record under it, what it earns, the stop line where the
-- move has one, How) beside the figure and the swap row. On a first appearance (firstAppearance) the New-move view: NEW
-- beside the title, How open as a section with its steps, the figure's New-move sequence with the setup line as its
-- caption and the fault pair, and the easier version first in the swap row. The card folds past its budget (TM): the
-- tips go to the move card first (the figure's tooltip), then a How open by default folds into its bar, its steps to
-- the move card too (a first appearance's only past the cap); a How the player opened never folds (Kit.Fit scales the
-- card). Answers the content's bottom.
local function renderReady(p, slot)
	local v = F.views.ready
	local new = firstAppearance(slot)
	local ctx = #p.moves > 1 and contextLine(p) or nil
	setHead(v, moveName(slot), ctx)
	v.isNew:SetShown(new)
	placeTitle(v, new and v.isNew or nil)
	v.dots:Set(p.index, ctx and #p.moves <= DOTS_MOST and #p.moves or 0)
	placeIntro(v, v.dots)
	v.introHover:SetShown(ctx ~= nil)
	local lost, lead = p.lostNote == true, leadLine(p, slot)
	v.lead:SetText(lead or "")
	local value, unit, note = goalOf(slot)
	v.note:SetText(note)
	local rewards = rewardsLine(p, slot)
	v.rewards:SetText(rewards or "")
	v.rewardsHover:SetShown(rewards ~= nil)
	local stop = Panel.MoveStop(slot.move)
	v.stopLine:SetText(stop or "")
	local howH = fillGuide(v.how, Panel.MoveText(slot.move, "how"))
	local tipsH = fillGuide(v.tips, Panel.MoveTips(slot.move))
	renderGuide(v.guide, slot.move)
	renderBar(p, slot)
	local sideH = sideFigure(v, p, slot, new and "new" or "loop")
	sideH = sideH + renderSwap(new, sideH)
	local top = Kit.Top(v)
	-- the column from the top, only what shows: How as its bar (`how` open or not) or, on a first appearance with How
	-- open, as a section; the tips under the steps while `tips`
	local function lay(how, tips)
		local y = Kit.Stack(v.left, top, { { v.lost, 0, show = lost }, { v.lead, lost and 8 or 0, show = lead ~= nil } })
		y = y + ((lost or lead) and 12 or 0)
		y = y + layGoal(v, y, value, unit)
		y = Kit.Stack(v.left, y, { { v.note, 4, show = note ~= "" }, { v.rewards, 5, show = rewards ~= nil },
			{ v.stopLine, 11, show = stop ~= nil } })
		local section = new and how
		v.howBar:SetOpen(how)
		local head = section and v.howHead or v.howBar
		y = Kit.Stack(v.left, y, { { v.howHead, 16, show = section }, { v.howBar, section and 0 or 16, show = not section } })
		-- the guide at the How row's right end, the section's rule stopping before it
		local headH = head:GetHeight()
		v.guide:ClearAllPoints()
		v.guide:SetPoint("RIGHT", v.left, "TOPLEFT", LEFT_W, -(y - headH / 2))
		v.howHead:SetWidth(LEFT_W - (v.guide:IsShown() and v.guide:GetWidth() + 12 or 0))
		return Kit.Stack(v.left, y, { { v.how, section and 14 or 10, howH, show = how },
			{ v.tipsHead, 16, show = how and tips }, { v.tips, 14, tipsH, show = how and tips } })
	end
	local byHand = howOpen[slot.move]
	local how = byHand
	if how == nil then how = new or (ns.Familiarity(ns.state, slot.move)) < 2 end
	local room = UIParent:GetHeight() - 2 * TM.edge
	local tipsAt = math.min(TM.budget, room)
	local howAt = (new and TM.firstCap) and math.min(TM.cap, room) or tipsAt
	local tips = how
	local bottom = lay(how, tips)
	if tips and heightOf(v, math.max(bottom, top + sideH)) > tipsAt then
		tips = false
		bottom = lay(how, false)
	end
	-- a How open by default folds past its line, its steps then on the move card; one the player opened stays open,
	-- and Kit.Fit scales the card instead (folded again, its bar's click would do nothing)
	local howFolded = how and byHand == nil and heightOf(v, math.max(bottom, top + sideH)) > howAt
	if howFolded then tips, bottom = false, lay(false, false) end
	F.tipsFolded, F.howFolded = not tips, howFolded
	return math.max(bottom, top + sideH)
end

-- The Pause button's menu (David, 2026-10-04), over the button: how long this character pauses, each choice ending
-- the pump (Lc.PausePump), and under them, titled as their own group, the body's two stops before a move starts:
-- "Something hurts", a submenu of the joints not flagged yet (a click sets the flag and the pump re-rolls or re-fits
-- without it, as on the Body page: Lc.FlagPain), and "I feel unwell", which asks first (Lc.UnwellAsk).
function Panel.PauseMenu(owner)
	Kit.OpenMenu(owner, function(_, root)
		ns.Sheet.PauseChoices(root, Lc.PausePump)
		root:CreateDivider()
		root:CreateTitle(L["ui.pause.body"])
		local joints = {}
		for _, key in ipairs(E.jointKeys) do
			if not ns.state.pain[E.joint[key]] then joints[#joints + 1] = key end
		end
		if #joints > 0 then
			local hurts = root:CreateButton(L["ui.pause.pain"])
			hurts:CreateTitle(L["ui.joint.title"])
			for _, key in ipairs(joints) do
				hurts:CreateButton(L["joint." .. key], function() Lc.FlagPain({ [E.joint[key]] = true }) end)
			end
		end
		root:CreateButton(L["ui.pause.unwell"], function() Lc.UnwellAsk() end)
	end, true)
end

-- The countdown (play SPEC 4): five seconds to get into position with the goal in sight; Back and the guide on the
-- bar, no way on (the go comes by itself).
local function renderCountdown(p, slot)
	local v = F.views.countdown
	setHead(v, sideName(p, slot), phaseLine(slot))
	renderGuide(v.guide, slot.move)
	local side = p.side == 1 and L["ui.panel.firstSide"] or p.side == 2 and L["ui.panel.switch"] or nil
	v.side:SetText(side or "")
	local goal = goalLine(slot)
	v.goal:SetText(goal)
	if Panel.countdownFor ~= p.goAt then
		Panel.countdownFor = p.goAt
		scheduleDigits(p)
	end
	local sideH = sideFigure(v, p, slot, "loop", Lc.SinceGo)
	local top = Kit.Top(v)
	local y = Kit.Stack(v.left, top, { { v.side, 0, show = side ~= nil }, { v.spot, side and 8 or 0 },
		{ v.goal, 10, show = goal ~= "" } })
	return math.max(y, top + sideH)
end

-- A goal met on the Running view (decision G1): once a hold's stopwatch reaches it, or the counted beat of a set of
-- reps does (slice 72: `reps`, the voice's count on the figure's beat, Speech.RepsBy, which the dial then starts at;
-- none without a count, and a max set and a counted set have no number to reach), the goal line turns gold with the
-- check before it (never the colour alone) until the side or the move ends; white until then. The chime plays with it
-- (updateRunning).
local function goalMet(slot, elapsed, reps)
	if slot.target == nil or slot.counted then return false end
	if slot.kind == K.hold then return elapsed >= slot.target end
	return slot.kind == K.rep and reps ~= nil and reps ~= false and reps >= slot.target
end

local function paintGoal(v, met)
	v.goalMet = met
	v.goal:SetText(met and Style.Check() .. " " .. v.goalWords or v.goalWords)
	v.goal:SetTextColor((met and NORMAL_FONT_COLOR or HIGHLIGHT_FONT_COLOR):GetRGB())
end

-- The Running view (play SPEC 5). Answers the content's bottom.
local function renderRunning(p, slot)
	local v = F.views.running
	setHead(v, sideName(p, slot), phaseLine(slot))
	renderGuide(v.guide, slot.move)
	local goalWords = not slot.counted and goalLine(slot) or "" -- a counted set's phase word says it (slice 42)
	v.goalWords = goalWords
	local pace = paceLine(slot)
	v.pace:SetText(pace)
	-- a set of reps whose count runs (Speech.RepCount; David, 2026-10-13: "show the rep count increasing instead of the
	-- time") shows its count in the clock's place; its plan is read once a render, never on the display's tick
	v.reps = ns.Speech.RepCount(slot) or false
	-- before the stack, which measures its words
	paintGoal(v, goalMet(slot, Lc.Elapsed(), v.reps and ns.Speech.RepsBy(v.reps, Lc.Elapsed())))
	-- a side move's first side ends in the switch: a hold's only by this button, a timed move's by its clock too
	v.done:SetText(L[p.side == 1 and "ui.panel.switch" or "ui.panel.done"])
	v.done:SetWidth(primaryWidth(v.done))
	v.keep:SetShown(Lc.KeepShows())
	v.dial:SetShown(p.limit ~= nil and v.dialArt)
	if p.limit then
		v.ring:Show() -- a cooldown hides itself when it runs out
		v.ring:SetCooldown(GetTime() - Lc.Elapsed(), p.limit)
	end
	local goal = partnerGoal(slot)
	v.partnerEnds = goal and goal + ns.RecordStep(slot.move, goal) or false
	for _, b in ipairs({ v.partner, v.you }) do
		if goal then
			b.mark:ClearAllPoints()
			b.mark:SetPoint("CENTER", b, "LEFT", PARTNER_W * goal / v.partnerEnds, 0)
		end
	end
	-- the phase word before the stack, which measures its words
	v.phase:SetText(phaseWord(slot, Lc.Elapsed()) or "")
	local sideH = sideFigure(v, p, slot, "loop", Lc.SinceGo)
	local top = Kit.Top(v)
	local bars = (LEFT_W - PARTNER_W) / 2
	local y = Kit.Stack(v.left, top, { { v.phase, 0, show = slot.allout == true },
		{ v.spot, slot.allout and 8 or 0, x = (LEFT_W - v.spot:GetWidth()) / 2 },
		{ v.partner, 8, show = goal ~= nil, x = bars }, { v.you, 4, show = goal ~= nil, x = bars },
		{ v.goal, 10, show = goalWords ~= "" }, { v.pace, 6, show = pace ~= "" } })
	return math.max(y, top + sideH)
end

-- The partner's and the player's bars at `elapsed` (the running clock's ticker moves them).
local function updatePartner(v, elapsed)
	local ends = v.partnerEnds
	if not ends then return end
	local at = math.min(elapsed + C.partner_lead_s, ends)
	local done = at >= ends
	v.partner:SetValue(at / ends)
	v.partner:SetStatusBarColor(unpack(Style.partnerBar[done and "done" or "partner"]))
	v.partner.time:SetText(done and L["ui.partner.done"]:format(clock(ends)) or clock(at))
	v.you:SetValue(math.min(elapsed, ends) / ends)
	v.you.time:SetText(clock(elapsed))
end

local function updateRunning()
	local p, v = Lc.pump, F.views.running
	if not p or p.state ~= "running" then return end
	local slot, elapsed = p.moves[p.index], Lc.Elapsed()
	-- a counting set of reps: the reps the figure has ended by the run's clock, 0 from the go, each side's from 0
	local reps = v.reps and ns.Speech.RepsBy(v.reps, elapsed)
	if p.limit then
		local left = p.limit - elapsed
		-- a side hold counts up, as its spoken marks do, while its dial drains to the switch, and /qp try's counting
		-- reps show their count while the dial drains and the last five beep; past the plan after Keep going the clock
		-- counts up to the full move (slice 52)
		local up = slot.kind == K.hold or p.keep
		v.clock:SetText(reps and tostring(reps) or clock(up and elapsed or math.ceil(left)))
		local whole = math.ceil(left)
		if whole <= 5 and whole > 0 and whole ~= Panel.lastWhole then Style.Play("lastFive") end
		Panel.lastWhole = whole
	else
		v.clock:SetText(reps and tostring(reps) or clock(elapsed))
	end
	-- the goal is the least, never the most: the small chime once a side as a hold's stopwatch passes it or a set's
	-- counted beat reaches it (slice 72), and the clock or the count goes on until Done; never on a hold that runs to
	-- a limit, whose dial says it
	local met = goalMet(slot, elapsed, reps)
	if met and not Panel.goalChimed and not (p.limit and slot.kind == K.hold) then
		Panel.goalChimed = true
		Style.Play("win")
	end
	updatePartner(v, elapsed)
	if slot.allout then v.phase:SetText(phaseWord(slot, elapsed)) end
	-- the goal met: the view drawn again, its goal line gold with the check and laid out at its new words; and so as
	-- Keep going comes into the bar or leaves it
	if met ~= v.goalMet or Lc.KeepShows() ~= v.keep:IsShown() then Panel.Render() end
end

-- The count on the dial (slice 66): `n` (nil: none) held between 0 and the most the move takes (Lc.CountMost: half
-- again what the app counted, on a set of reps or the number it has to beat where that is higher, and at least a set's
-- least, else one set's most; a typed number above it becomes it), `fresh` while it is the number the count started
-- at, untouched (a digit then replaces it); drawn on the dial with its preview. Nothing while the dial takes no count.
function Panel.SetCount(n, fresh)
	local p, slot = Lc.pump, Lc.Slot()
	if not (counting(p) and slot) then return end
	if n then n = math.max(0, math.min(n, (Lc.CountMost(p, slot, Panel.LoadValue())))) end
	F.count, F.countFresh = n, fresh == true
	Panel.Preview()
end

-- A digit or Backspace on the dial (Panel.HandleKey's DIGIT): the digit after the count's, or in place of the number
-- the count started at; Backspace takes the last digit off, or that number whole.
function Panel.TypeCount(key)
	local d, n = tonumber(DIGIT[key] or ""), F.count
	if F.countFresh or not n then
		n = d
	elseif d then
		n = n * 10 + d
	else
		n = n >= 10 and math.floor(n / 10) or nil
	end
	Panel.SetCount(n)
end

-- The dial's - and + (its buttons, the arrows, the wheel): one rep or one second; + from no count is 1, - leaves no
-- count as it is.
function Panel.StepCount(dir)
	local n = F.count
	if not n and dir < 0 then return end
	Panel.SetCount((n or 0) + dir)
end

-- Records the dial's count, with the weight box's weight on a loaded move; false when there is none, the dial has not
-- settled (it may hold a hold's stopwatch or the voice's reps, and the press that ended the move, pressed twice, never
-- saves it unread) or it is past the heaviest weight the box takes (the preview says so). A counted set records by its
-- three answers (Panel.Counted): here only the "Really?" row's Yes keeps its count with the answer that asked it.
function Panel.RecordInput()
	local value, slot = Panel.InputValue(), Lc.Slot()
	if not slot or not answerSettled() then return false end
	if slot.counted then
		local p = Lc.pump
		if not (p and p.really) then return false end
		return Panel.Counted(p.reallyAnswer)
	end
	if not value then return false end
	local answer = Lc.Enter(value, Panel.LoadValue())
	if answer == "max" then
		F.views.result.preview:SetText(value > ns.CountMax(slot) and L["ui.result.max"]:format(ns.CountMax(slot))
			or L["ui.result.maxKg"]:format(C.load_max_kg))
		return false
	end
	return answer ~= "really" -- the "Really?" row asks (Lc.Enter set it and redrew the view)
end

-- A counted set's answer (slice 42; its code since slice 53, `code`: 0 Yes, to the end, E.answer.slipped or
-- E.answer.notAllOut), with the count the dial holds: Lc.Counted records it; false where it is over one set (the
-- preview says so) or "Is that right?" asks first.
function Panel.Counted(code)
	local answer = Lc.Counted(Panel.InputValue(), code)
	if answer == "max" then
		F.views.result.preview:SetText(L["ui.result.max"]:format(C.count_max))
		return false
	end
	return answer ~= "really"
end

-- "Change it" on the "Really?" row: the count goes from the dial for the right one; or, when it asked about the
-- weight (ns.Validate's "heavy", kept by Lc.Enter), the weight box takes the planned weight back.
function Panel.ChangeCount()
	local slot, p = Lc.Slot(), Lc.pump
	local heavy = p ~= nil and p.reallyCheck == "heavy"
	Lc.ClearReally()
	if slot and heavy and slot.load then
		F.kg:SetText(ns.Sheet.Kg(slot.load))
		if typed() then F.kg:SetFocus() end
	else
		Panel.SetCount(nil)
	end
end

-- The "Really?" row's words for count `n` (with weight `load`, nil none) on `slot`, asked as `check` (ns.Validate's
-- "really", "low" or "heavy") and judged against `state` (the count step's ns.state; Correct a count's, the state
-- before its set, Settings.lua): the line and its Yes. The count against the record (far above or far below it; a
-- first max set's against the rung's top), or a weight far over the planned one. A set with a weight names it with the
-- count. "low" only comes with a record (ns.Validate).
function Panel.ReallyWords(state, slot, check, n, load)
	local hold, best = slot.kind == K.hold, ns.Record(state, slot.move)
	local count = slot.counted and tostring(n) or ns.Sheet.LoadAmount(slot.move, load, n)
	-- a counted set's count far above its best clean count (slice 42): kept as typed, never "recorded", since the
	-- answer under it (Yes, to the end alone keeps one) decides whether it can be a best
	local low = check == "low" or slot.counted
	local line
	if slot.counted then
		line = ns.Plural("ui.result.confirmBest", n):format(n, best or 0)
	elseif check == "heavy" then
		line = L["ui.result.confirmKg"]:format(ns.Sheet.Kg(load), ns.Sheet.Kg(slot.load))
	elseif low or (best and not slot.placing) then
		local topLoad, topReps = ns.LoadRecord(state, slot.move)
		line = L[low and "ui.result.confirmLow" or "ui.result.confirm"]:format(count,
			recordText(slot, best, load, { topLoad = topLoad, topReps = topReps }))
	else
		line = L["ui.result.confirmFirst"]:format(count, ns.Sheet.Amount(ns.FirstTop(slot) or 0, hold))
	end
	return line, L[low and "ui.result.reallyKeep" or "ui.result.reallyYes"]:format(count)
end

-- The "Really?" row in the preview's place while the count and the weight in the inputs are the ones it asks about,
-- and its two answers (Panel.ReallyWords).
local function showReally(v, p, slot, n, load)
	local asking = p ~= nil and p.really ~= nil and p.really == n and p.reallyLoad == load
	v.preview:SetShown(not asking)
	v.really:SetShown(asking)
	v.reallyRow:SetShown(asking)
	if not asking then return false end
	local line, yes = Panel.ReallyWords(ns.state, slot, p.reallyCheck, n, load)
	v.really:SetText(line)
	v.reallyYes:SetText(yes)
	return true
end

-- The count's words while it is entered: the live preview ("23 reps: +40 XP, new record", "16 reps with 11 kg: +40
-- XP, new record"; none before a number, a counted set's before a clean count above 0 ("Not counted" until 2026-10-23),
-- on the practice move, which stores nothing, or under the minimum, which earns nothing: ns.ShortCount), or the
-- "Really?" row in its place, Record hidden while it asks so
-- no red pushes either answer and greyed until a number is in.
local function previewWords(v, p, slot)
	local n, load = Panel.InputValue(), Panel.LoadValue()
	local asking = showReally(v, p, slot, n, load)
	v.record:SetShown(not asking and not slot.counted)
	v.record:SetEnabled(n ~= nil)
	if asking then return end
	if slot.counted then
		local best = ns.Record(ns.state, slot.move)
		v.preview:SetText((not n or n <= 0 or ns.ShortCount(slot, n)) and ""
			or (not best or n > best) and ns.Plural("ui.result.cleanNew", n):format(n)
			or ns.Plural("ui.result.cleanUnder", n):format(n, best))
		return
	end
	if not n or slot.practice or ns.ShortCount(slot, n) then
		v.preview:SetText("")
		return
	end
	local outcome, value = ns.Judge(ns.state, slot, n, p and p.elapsed, load)
	local side = ns.Library[slot.move].side -- a side move counts one side
	local unit = join(L[slot.kind == K.hold and (side and "ui.unit.secondsSide" or "ui.unit.seconds")
		or (side and "ui.unit.repsSide" or "ui.unit.reps")], withLoad(slot.move, load), " ")
	local xp = ns.MoveXP(slot.move, outcome, slot.owed ~= nil, { rung = slot.rung, allout = slot.allout })
	local goal = ns.TargetAt(ns.state, slot, load)
	local word = (slot.hi and slot.maxSet and n >= slot.hi) and L["outcome.enough"]
		or (outcome == O.lastbeat and goal and n >= goal) and L["outcome.goal"]
		or (outcome == O.placement and ns.Record(ns.state, slot.move)) and L["outcome.placed"]
		or L["outcome." .. OUTCOME[outcome]]
	v.preview:SetText(L["ui.result.preview"]:format(value, unit, xp, word))
end

-- What the set earned, as rows under the recorded set's outcome (the Ready view's chips, judged): an earned one in
-- gold with its check, flashing once; a missed one with its dash and its icon greyed. Answers how many show.
local function renderRewardRows(p, slot)
	local v = F.views.result
	local done = p.committed and p.last and not p.pour and not slot.practice and not slot.standup
	if done then previewOf(p, slot) end
	local n = 0
	for i, row in ipairs(v.rewardRows) do
		local c = done and p.previewChips[i] or nil
		row:SetShown(c ~= nil)
		if c then
			n = n + 1
			local got = earnedChip(c, p.last.earned)
			local name, sub = chipText(c, p.last.earned)
			row.name:SetText(got and name or L["ui.rewards.missed"]:format(name))
			row.line:SetText(sub)
			row.icon:SetTexture(Style.statIcon[c.stat])
			Kit.Choose(row, got)
			row.check:SetShown(got)
			if got and row.flashedFor ~= p.id .. ":" .. p.index then
				row.flashedFor = p.id .. ":" .. p.index
				row.anim:Play()
			end
		end
	end
	Kit.LayList(v.rewardList)
	return n
end

-- The character's reaction to a won set, once per move (as the pour's rows arrive, Pour.react; on a set saved with no
-- pour, its Result): a salute for a step up, a cheer for a star, a flex for a record, a first one too (Pour.first);
-- answers whether it shows.
local function react(p)
	local last = p.last
	local anim
	if last then
		for _, e in ipairs(last.earned or {}) do
			if e.kind == "stat" and e.reason == "rung" then anim = SALUTE end
		end
		if not anim then
			for _, e in ipairs(last.earned or {}) do
				if e.kind == "stat" and e.reason == "star" then anim = CHEER end
			end
		end
		if not anim and (last.outcome == O.beat or Pour.first(p, p.moves[p.index])) then anim = FLEX end
	end
	-- the character sheet's own scene (Gethe forever ModelSceneUtil.lua, 70058); one that is missing or failed to set
	-- up leaves no empty box, on this render or the next: the model and its screen stay hidden, and a failure is
	-- reported once
	local shown = anim ~= nil and p.reactFailed ~= p.index
	F.model:SetShown(shown)
	if not anim or p.reacted == p.index then return shown end
	p.reacted = p.index
	local setUp = ModelSceneUtil and ModelSceneUtil.SetUpCharacterSheetScene
	local ok, err = setUp ~= nil, nil
	if setUp then ok, err = pcall(setUp, F.model) end
	if not ok then
		p.reactFailed = p.index
		F.model:Hide()
		if err then ns.Report(err) end
		return false
	end
	local actor = F.model.GetPlayerActor and F.model:GetPlayerActor()
	if actor then actor:SetAnimation(anim) end
	return true
end

-- The Result view's column laid out for its state (renderResult's v.state), the right column's height `sideH` beside
-- it; answers the content's bottom.
local function layResult(v)
	local top = Kit.Top(v)
	local state = v.state
	if state == "pour" then return v.pourBottom end
	if state == "count" or state == "form" then
		-- the dial at the column's top, where the Running view's stood and the pour's stands (Pour.placePour), then in
		-- order what the count step holds under it
		local really, form = v.really:IsShown(), state == "form"
		local answers = not form and not really and centreButtons(v.answers, { v.empty, v.couldNot })
		centreButtons(v.reallyRow, { v.reallyYes, v.reallyChange })
		-- "Don't know the weight" while the box holds a weight and "Is that right?" does not ask (Change it answers it);
		-- its row keeps its room under the weight box while it hides, so no answer under it moves into its place under a
		-- second click
		v.kgNone:SetShown(F.kg:IsShown() and not really and Panel.LoadValue() ~= nil)
		centreButtons(v.kgNoneRow, { v.kgNone })
		return math.max(top + v.sideH, Kit.Stack(v.left, top, {
			{ v.dialRoom, 0, x = (LEFT_W - v.dialRoom:GetWidth()) / 2 }, { v.cleanLabel, 4, show = form },
			{ v.cap, 6, show = v.cap:GetText() ~= "" },
			{ v.preview, 10, show = not really and v.preview:GetText() ~= "" }, { v.really, 10, show = really },
			{ v.reallyRow, 8, show = really }, { v.load, 12, show = F.kg:IsShown() },
			{ v.kgNoneRow, 6, show = F.kg:IsShown() and not really }, { v.forms, 16, show = form and not really },
			{ v.answers, 16, show = answers } }))
	elseif state == "whole" then
		return Kit.Stack(v.left, top, { { v.whole, 0, x = (TM.TEXT - WHOLE_W) / 2 } })
	elseif state == "mark" then
		return math.max(top + v.sideH, Kit.Stack(v.left, top, { { v.marks, 0 } }))
	elseif state == "pace" then
		return math.max(top + v.sideH, Kit.Stack(v.left, top, { { v.paces, 0 } }))
	elseif state == "longer" then
		return math.max(top + v.sideH, Kit.Stack(v.left, top, { { v.longers, 0 } }))
	end
	local bar, rows = v.bar:IsShown(), v.rows > 0
	return math.max(top + v.sideH, Kit.Stack(v.left, top, { { v.outcome, bar and 12 or 0 },
		{ v.nextLoad, bar and 14 or 8, show = v.nextLoad:GetText() ~= "" },
		{ v.after, 8, show = v.after:GetText() ~= "" }, { v.rewardsHead, 20, show = rows },
		{ v.rewardList, 14, show = rows } }))
end

-- The count's words while it is set (the dial's count and the weight box change): the dial drawn at it, its preview,
-- and the view laid out again, since the "Really?" row takes the preview's place and the cap's line comes and goes.
function Panel.Preview()
	local v, p, slot = F.views.result, Lc.pump, Lc.Slot()
	if not (slot and v and (v.state == "count" or v.state == "form")) then return end
	Pour.drawCount(v, p, slot)
	previewWords(v, p, slot)
	if v:IsShown() then fitCard(v, layResult(v)) end
end

-- The pour (buildPour) -----------------------------------------------------------------------------------------------

-- A first record (slice 72: a first record is a record): a count above 0 on a move with no record before it, the
-- outcome its line calls "your first record", read before its row is written (the pending result, the record now) or
-- after (p.last's, the record before it). It takes the record's jingle, chip and badge and the character's flex.
function Pour.first(p, slot)
	local set = p.last or p.pending
	if not set or set.outcome ~= O.placement or (set.value or 0) <= 0 then return false end
	if p.last then return p.last.best == nil end
	return ns.Record(ns.state, slot.move) == nil
end

-- The character's reaction as the pour's rows arrive (slice 72: every count pours since slice 66, so the Result's own
-- view, where react runs too, follows only a set saved with no count): once a move (react), in the right column in
-- the dimmed figure's place while the rows stand. None with the game's reduced motion, whose end state is the figure.
function Pour.react(p)
	if not p or not ns.Tree.Moving() or not react(p) then return end
	hideFigure()
	F.screen:Hide()
	F.modelScreen:Show()
end

-- The count's dial while the count step asks (slice 66): the pour's own over the column's room for it (v.dialRoom),
-- the pour's other parts hidden; the count in its clock's place in the move's unit (Pour.format; grey while it has
-- none), the ring held still at its share (Pour.share: a Cooldown paused there, whose unreversed swipe draws the part
-- still to run, so at 1 - the share; CooldownFrame_SetDisplayAsPercentage), - stopping at 0 and + at the most the move
-- takes (Lc.CountMost), and under it, while the count stands at the most what the app counted or the number to beat
-- set, the line that says why (Rapha asked for the reason).
function Pour.drawCount(v, p, slot)
	local pv, n = v.pour, F.count
	for _, part in ipairs({ pv.bars[1], pv.bars[2], pv.rows, pv.after, pv.reveal, pv.hero.frame, pv.chip, pv.hit }) do
		part:Hide()
	end
	pv.spot:Show()
	pv.clock:Show()
	pv.spot:ClearAllPoints()
	pv.spot:SetPoint("CENTER", v.dialRoom)
	pv.clock:SetText(Pour.format(slot, n or 0))
	local c = n and WHITE or saga.note
	pv.clock:SetTextColor(c[1], c[2], c[3])
	pv.dial:SetShown(pv.dialArt)
	pv.ring:Show() -- a cooldown hides itself when it runs out
	local load = Panel.LoadValue()
	CooldownFrame_SetDisplayAsPercentage(pv.ring, 1 - Pour.share(slot, n or 0, Lc.BestAt(slot, load)))
	local most, counted, beat = Lc.CountMost(p, slot, load)
	v.less:SetEnabled(n ~= nil and n > 0)
	v.more:SetEnabled(n == nil or n < most)
	local line = counted and L["ui.count.cap"]:format(Pour.format(slot, most), Pour.format(slot, counted))
		or beat and L["ui.count.cap.goal"]:format(Pour.format(slot, most), Pour.format(slot, beat))
	v.cap:SetText(n == most and line or "")
end

-- The unit grain `i` of the run lands on: each its own, or with more units than POUR.most grains its share.
function Pour.unit(run, i)
	return math.floor(i * run.held / run.grains + 0.5)
end

-- Whether the count passed `value` going from `prev` to `n`.
function Pour.crossed(prev, n, value)
	return value ~= nil and prev < value and n >= value
end

-- The move's one jingle (`key`, Style's), once a move: the record's as the pour passes the best under the top, the
-- top's (the world-quest chime) when the count reached it, the step-up's at its reveal (Sheet.UnlockSound), else the
-- small chime as the rows come in. A record and a top only show what the saved row brings once it has landed, so a
-- set that reaches its top keeps its jingle to the landing, where a step it opened takes it.
function Pour.jingle(run, key)
	if run.jingled then return end
	run.jingled = true
	Style.Play(key)
end

-- `n` sparks in `color` from `region`'s centre and (x, y) off it, `reach` out: up in a fan, or all round (`round`).
-- None with the game's reduced motion (Tree.Moving).
function Pour.sparks(pv, region, x, y, n, reach, color, round)
	if not ns.Tree.Moving() then return end
	for k = 1, math.min(n, #pv.sparks) do
		local s = pv.sparks[k]
		local deg = round and 360 * k / n + math.random() * 20
			or 10 + 160 * (k - 1) / math.max(1, n - 1) + (math.random() - 0.5) * 14
		local a, d = math.rad(deg), reach * (0.55 + math.random() * 0.6)
		s.anim:Stop()
		s:ClearAllPoints()
		s:SetPoint("CENTER", region, "CENTER", x, y)
		s:SetVertexColor(color[1], color[2], color[3])
		s.fly:SetOffset(math.cos(a) * d, math.sin(a) * d)
		local dur = 0.65 + math.random() * 0.25
		for _, part in ipairs(s.parts) do part:SetDuration(dur) end
		s.anim:Play()
	end
end

function Pour.stopGrains(pv)
	for _, g in ipairs(pv.grains) do g.anim:Stop() end
end

-- Stops what the pour plays (its timers, its grains, its groups), leaving the run to start again or end at once when
-- the view comes back (renderPour): a combat, the game menu or a popup that hid the dim, or another view.
function Pour.stopPour()
	local pv = F.views.result and F.views.result.pour
	if not pv or not pv.run or pv.run.phase == "stopped" then return end
	for _, name in ipairs({ "pourStage", "pourOpen", "pourGem", "pourFold" }) do T.cancel(name) end
	Pour.stopGrains(pv)
	for _, ag in ipairs(pv.anims) do ag:Stop() end
	if pv.run.phase ~= "done" then pv.run.phase = "stopped" end
end

-- The rung up the saved row brought on the move's own ladder (ns.Progression's events, p.last.events), or nil.
function Pour.stepUp(p, slot)
	local ladder = ns.Library[slot.move].ladder
	for _, e in ipairs(p.last and p.last.events or {}) do
		if e.kind == "rungUp" and e.ladder == ladder then return e end
	end
	return nil
end

-- The bar's label: the ladder's name and the step (Wall sit · step 5).
function Pour.barLabel(ladder, rung)
	local step = ns.Step(ladder, rung)
	return step and L["ui.pour.step"]:format(L["ladder." .. ladder], step) or ""
end

-- The day at the top the saved row counted (DB.lua's topDay on the row: a day toward the next step), as its pips and
-- words: after a step-up the days it took, else the climb's days so far; nil without one.
function Pour.topDayLine(p, slot, ev)
	local ladder = ns.Library[slot.move].ladder
	if not (ladder and p.last.row and p.last.row.topDay) then return nil end
	local have, need, toward
	if ev then
		need = ns.UnlockDays(ladder, ev.rung - 1, ns.state)
		have, toward = need, ns.Step(ladder, ev.rung)
	else
		have, need = ns.UnlockProgress(ns.state, ladder, ns.FilterContext(ns.profile, ns.state))
		toward = ns.Step(ladder, ns.Rung(ns.state, ladder) + 1)
	end
	if not (toward and need and need > 0) then return nil end
	return Style.Pips(have, need) .. " " .. L["ui.pour.topDay"]:format(have, need, toward)
end

-- The move's rows from what the saved row earned (Lc.earning, p.last.earned) and moved (its events): laid out hidden,
-- so the stage keeps their height (renderPour); the rung up they found becomes the run's.
function Pour.fillRows(pv, p, slot)
	local run, last = pv.run, p.last
	local ev = Pour.stepUp(p, slot)
	run.ev = ev
	local row = pv.moveRow
	Kit.SetIcon(row.icons[1], slot.move)
	row.name:SetText(moveName(slot))
	row.value:SetText(Pour.format(slot, run.held, run.timed))
	local record = run.beat and run.best ~= nil
	row.lead:SetShown(record or run.first) -- a first record is a record, with none before it to name
	-- the record it beat as the outcome line names it: a loaded move's weighted one with its weight (recordText)
	local was = record and (last.load and last.topLoad and recordText(slot, run.best, last.load, last)
		or Pour.format(slot, run.best, run.timed))
	local line = join(was and L["ui.pour.was"]:format(was) or "", Pour.topDayLine(p, slot, ev) or "", "  ")
	row.line:SetText(line)
	row.line:SetShown(line ~= "")
	local up = pv.unlockRow
	local id = ev and (ev.move or ns.Anchor(ev.ladder, ev.rung))
	up:SetShown(id ~= nil)
	if id then
		Kit.SetIcon(up.icons[1], id)
		Kit.PaintScreen(up.screen, ev.quality)
		up.name:SetText(ns.Sheet.MoveName(id))
		up.name:SetTextColor(ns.Sheet.QualityColor(ev.quality):GetRGB())
		up.line:SetText(L["ui.pour.unlockStep"]:format(ns.Step(ev.ladder, ev.rung), Kit.QualityWords(ev.quality)))
	end
	-- the stats it raised, each once with what it gained in all, in the order they came
	local gains, order = {}, {}
	for _, e in ipairs(last.earned or {}) do
		if e.kind == "stat" and e.amount > 0 then
			if not gains[e.stat] then order[#order + 1] = e.stat end
			gains[e.stat] = (gains[e.stat] or 0) + e.amount
		end
	end
	for i, r in ipairs(pv.statRows) do
		local stat = order[i]
		r:SetShown(stat ~= nil)
		if stat then
			r.icon:SetTexture(Style.statIcon[stat])
			r.name:SetText(L["ui.rewards.stat"]:format(gains[stat], ns.Sheet.StatName(stat)))
		end
	end
	pv.xpRow:SetShown((last.xp or 0) > 0)
	pv.xpRow.name:SetText(L["ui.rewards.gain"]:format(last.xp or 0))
	Kit.LayList(pv.rows)
	pv.after:SetText(join(nextLoadLine(p, slot), afterLines(p, slot), "\n"))
	pv.rowsFor = run.key
end

-- The pour's stage on the view under its head: the dial in the left column's middle, the rows there once they are in,
-- the bars under it across the card, the reveal over the figure's screen, the stage's click over all of it; its
-- height the dial's, the screen's or the rows', whichever is tallest, so the card keeps its height as they come.
function Pour.placePour(v, pv)
	local top = Kit.Top(v)
	local rowsH = 0
	if pv.rowsFor == pv.run.key then
		rowsH = pv.rows:GetHeight() + (pv.after:GetText() ~= "" and 10 + pv.after:GetStringHeight() or 0)
	end
	local h = math.max(pv.spot:GetHeight(), SCREEN_H, rowsH)
	pv.top, pv.h = top, h
	pv.spot:ClearAllPoints()
	pv.spot:SetPoint("CENTER", pv, "TOPLEFT", TM.IN + LEFT_W / 2, -(top + h / 2))
	pv.rows:ClearAllPoints()
	pv.rows:SetPoint("TOPLEFT", pv, "TOPLEFT", TM.IN, -(top + (h - rowsH) / 2))
	pv.after:ClearAllPoints()
	pv.after:SetPoint("TOP", pv.rows, "BOTTOM", 0, -10)
	for _, b in ipairs(pv.bars) do
		b:ClearAllPoints()
		b:SetPoint("TOPLEFT", pv, "TOPLEFT", TM.IN, -(top + h + POUR.barGap))
	end
	pv.reveal:ClearAllPoints()
	pv.reveal:SetAllPoints(F.screen)
	pv.hit:ClearAllPoints()
	pv.hit:SetPoint("TOPLEFT", pv, "TOPLEFT", TM.IN, -top)
	pv.hit:SetSize(TM.TEXT, h + POUR.barGap + POUR.barH)
	pv.hit:SetShown(pv.run.phase ~= "done")
	-- the reveal takes the figure's place while it plays
	if pv.run.phase == "reveal" then F.figure.frame:Hide() end
	v.pourBottom = top + h + POUR.barGap + POUR.barH
end

function Pour.showRecord(pv)
	pv.chip:Show()
	pv.clock:SetTextColor(POUR.gold[1], POUR.gold[2], POUR.gold[3])
end

-- The pour as it stands once landed: the dial at the count, the bar filled to it, the chip on a record.
function Pour.landBar(pv)
	local run = pv.run
	Pour.stopGrains(pv)
	pv.ring:Clear()
	pv.clock:SetText(Pour.format(run.slot, run.held, run.timed))
	Pour.fillBar(pv.bars[1], run.held)
	if run.first or run.beat and run.best then Pour.showRecord(pv) end
end

-- A run of the pour for the move on the view (`key`: the pump, the move and its go): its bar, from what the move's
-- kind shows (slice 66): a set of reps' or a hold's step on its ladder (its own window's top, the count a day at the
-- top needs; the goal and the record before this set as posts), a counted set's best alone, a timed move's goal alone
-- (it keeps no record); the dial at 0 and nothing else in, and the jingle it plans (`jingle`'s rule).
function Pour.newRun(pv, p, slot, key)
	Pour.stopPour()
	local m = ns.Library[slot.move]
	local held, best, timed = p.pour.value, p.pour.best, p.pour.timed
	local goal = Pour.goal(slot, timed)
	local top
	if slot.kind == K.rep or slot.kind == K.hold then
		if m.ladder and m.rung then top = select(2, ns.Window(m.ladder, m.rung, slot.move)) end
		top = top or math.max(held, goal or 0, best or 0, 1)
	end
	local q = m.ladder and m.rung and ns.Quality(m.ladder, m.rung) or ns.MoveQuality(slot.move) or E.quality.starter
	local run = { key = key, slot = slot, timed = timed, held = held, best = best, beat = p.pour.beat,
		grains = math.min(held, POUR.most), n = 0, tickAt = 0, f0 = Pour.share(slot, held, best, timed),
		first = Pour.first(p, slot) }
	run.jingle = top and held >= top and "top" or (run.first or run.beat and best) and "record" or "done"
	pv.run = run
	Pour.setBar(pv.bars[1], { label = m.ladder and m.rung and Pour.barLabel(m.ladder, m.rung) or "", q = q, goal = goal,
		best = best, top = top, range = top or math.max(held, goal or 0, best or 0, 1), slot = slot, timed = timed })
	pv.bars[1]:SetAlpha(1)
	pv.bars[1]:Show()
	pv.bars[2]:Hide()
	pv.rows:Hide()
	pv.after:Hide()
	pv.reveal:Hide()
	pv.hero.frame:Hide()
	pv.unlockRow.icons[1]:Show()
	pv.spot:Show()
	pv.clock:Show()
	pv.chip:Hide()
	pv.clock:SetTextColor(HIGHLIGHT_FONT_COLOR:GetRGB())
	pv.clock:SetText(Pour.format(slot, 0, timed))
	pv.dial:SetShown(pv.dialArt)
	return run
end

-- The pour from its start: the ring at the count's share (Pour.share, where the count step's dial held it), emptying
-- on the cooldown as the grains leave its tip, each grain i of n leaving pour_hold_s + span * ((i - 1) / n) ^ 0.62 s in
-- (closer as they go, span the shorter of pour_s and pour_base_s + pour_step_s a grain) and flying pour_fly_s from
-- where the ring's end stands then to its unit's place; a count of nothing lands at once.
function Pour.startPour(pv)
	local run = pv.run
	run.phase, run.n, run.tickAt = "pour", 0, 0
	pv.chip:Hide()
	pv.clock:SetTextColor(HIGHLIGHT_FONT_COLOR:GetRGB())
	pv.clock:SetText(Pour.format(run.slot, 0, run.timed))
	Pour.fillBar(pv.bars[1], 0)
	local n = run.grains
	if n <= 0 then return T.after("pourStage", 0, function() Lc.Land() end) end
	local hold, fly = C.pour_hold_s, C.pour_fly_s
	local span = math.min(C.pour_s, C.pour_base_s + C.pour_step_s * n)
	local total = hold + span
	pv.ring:Resume() -- the count step held it still (Pour.drawCount)
	pv.ring:SetCooldown(GetTime() - (1 - run.f0) * total / run.f0, total / run.f0)
	local cx, cy = TM.IN + LEFT_W / 2, -(pv.top + pv.h / 2)
	local r = pv.spot:GetWidth() / 2 * RING.tip
	local b = pv.bars[1]
	local ty = -(pv.top + pv.h + POUR.barGap + POUR.trackY + POUR.trackH / 2)
	for i = 1, n do
		local start = hold + span * ((i - 1) / n) ^ 0.62
		-- the ring's end then: the unreversed swipe's edge, clockwise from 12 o'clock past the part run
		local phi = (1 - run.f0 * (1 - start / total)) * 2 * math.pi
		local sx, sy = cx + r * math.sin(phi), cy + r * math.cos(phi)
		local tx = TM.IN + Pour.unitX(b, Pour.unit(run, i))
		local mx, my = (sx + tx) / 2, (sy + ty) / 2 - POUR.sag
		local g = Pour.grain(pv, i)
		g.n = i
		g.anim:Stop()
		g:ClearAllPoints()
		g:SetPoint("CENTER", pv, "TOPLEFT", sx, sy)
		g.wait:SetDuration(start)
		g.show:SetDuration(fly * 0.45)
		g.out:SetDuration(fly * 0.45)
		g.out:SetOffset(mx - sx, my - sy)
		g.back:SetDuration(fly * 0.55)
		g.back:SetOffset(tx - mx, ty - my)
		g.dim:SetDuration(fly * 0.55)
		g.anim:Play()
	end
end

-- A grain landed: the dial at its unit, the bar filled to it, the posts it passed lit, the record chip once past the
-- best, the top's flash and sparks; its sound (a passed post's double, the top's triple, the record's jingle in the
-- best's place, else a tick, none closer than pour_tick_gap_s to the last); the last grain lands the pour.
function Pour.landGrain(pv, i)
	local run, p = pv.run, Lc.pump
	if not run or run.phase ~= "pour" or not p or not p.pour or p.committed then return end
	local prev, n = run.n, Pour.unit(run, i)
	run.n = n
	pv.clock:SetText(Pour.format(run.slot, n, run.timed))
	local b = pv.bars[1]
	Pour.fillBar(b, n)
	local sound
	if Pour.crossed(prev, n, b.spec.goal) then sound = "pourPost" end
	if run.beat and run.best and Pour.crossed(prev, n, run.best + 1) then
		Pour.showRecord(pv)
		sound = "pourPost"
		if run.jingle == "record" then
			sound = false
			Pour.jingle(run, "record")
		end
	end
	if Pour.crossed(prev, n, b.spec.top) then
		b.flashAnim:Play()
		Pour.sparks(pv, b.track, b.trackW / 2, 0, 16, 60, POUR.gold)
		if sound ~= false then sound = "pourTop" end
	end
	local now = GetTime()
	if sound then
		Style.Play(sound)
		run.tickAt = now
	elseif sound == nil and now - run.tickAt >= C.pour_tick_gap_s then
		Style.Play("pourTick")
		run.tickAt = now
	end
	if i >= run.grains then Lc.Land() end
end

-- The move's rows in the dial's place: one after the other from the right (`still`: all at once, the end state), the
-- unlock row's picture empty while the reveal's figure flies into it (`waitTile`); the small chime as they come when
-- no other jingle was the move's.
function Pour.showRows(pv, still, waitTile)
	local run = pv.run
	run.phase = still and "done" or "rows"
	pv.spot:Hide()
	pv.clock:Hide()
	pv.chip:Hide()
	pv.rows:Show()
	local shown = 0
	for _, row in ipairs(pv.rows.rows) do
		row.enter:Stop()
		if row:IsShown() then
			row:SetAlpha(still and 1 or 0) -- an entrance shows nothing before its group runs
			if not still then
				row.enter.hold:SetDuration(shown * POUR.rowStep)
				row.enter:Play()
			end
			shown = shown + 1
		end
	end
	pv.after.enter:Stop()
	pv.after:SetShown(pv.after:GetText() ~= "")
	pv.after:SetAlpha(still and 1 or 0)
	if pv.after:IsShown() then
		if not still then
			pv.after.enter.hold:SetDuration(shown * POUR.rowStep)
			pv.after.enter:Play()
		end
		shown = shown + 1
	end
	pv.unlockRow.icons[1]:SetShown(not waitTile)
	if still then pv.hit:Hide() end
	if not still then
		T.after("pourStage", shown * POUR.rowStep + POUR.rowIn, function()
			if pv.run == run and run.phase == "rows" then
				run.phase = "done"
				pv.hit:Hide()
				Lc.GoOn()
			end
		end)
	else
		Lc.GoOn()
	end
	if not run.ev then Pour.jingle(run, "win") end
	Pour.react(Lc.pump)
end

-- The bar rolls over to the next step's after a step-up (its label and colour, and on a bar with a top the next
-- step's: the new move's first set is a max set, with no goal and no record yet): this one up and out, the next one
-- up from under it.
function Pour.rollBar(pv, still)
	local ev, old, new = pv.run.ev, pv.bars[1], pv.bars[2]
	local id = ev.move or ns.Anchor(ev.ladder, ev.rung)
	-- a step's top on a bar that shows one (a set of reps', a hold's); a counted set's and a timed move's keep their span
	local top = old.spec.top and (id and select(2, ns.Window(ev.ladder, ev.rung, id)) or old.spec.range)
	Pour.setBar(new, { label = Pour.barLabel(ev.ladder, ev.rung), q = ev.quality, top = top,
		range = top or old.spec.range, slot = old.spec.slot, timed = old.spec.timed, lit = true })
	new:Show()
	if still then
		old:Hide()
		new:SetAlpha(1)
		return
	end
	new:SetAlpha(0)
	old.leave:Play()
	new.enter:Play()
end

function Pour.landHero(pv)
	pv.hero.frame:Hide()
	pv.unlockRow.icons[1]:Show()
end

-- The move's end state at once (the game's reduced motion, a click past the reveal or the rows, the view back after
-- a combat or a menu hid it once the row was saved): the bar filled (rolled over after a step-up), the rows in, no
-- reveal, and the move's one jingle.
function Pour.endState(pv, p, slot)
	local run = pv.run
	Pour.stopPour()
	Pour.landBar(pv)
	Pour.fillRows(pv, p, slot)
	pv.reveal:Hide()
	F.figure.frame:Show()
	Pour.landHero(pv)
	if run.ev then
		Pour.rollBar(pv, true)
		Pour.jingle(run, ns.Sheet.UnlockSound(run.ev))
	elseif run.jingle == "top" then
		Pour.jingle(run, "star")
	elseif run.jingle == "record" then
		Pour.jingle(run, "record")
	end
	Pour.showRows(pv, true, false)
end

-- The reveal folds: the new move's figure flies into its row's picture, the reveal fades, the bar rolls over, the
-- rows come in and the figure comes back, dimmed.
function Pour.foldReveal(pv)
	local run = pv.run
	if not run or run.phase ~= "reveal" then return end
	T.cancel("pourGem")
	Pour.rollBar(pv, false)
	F.figure.frame:Show()
	Pour.showRows(pv, false, true) -- the character may take the figure's place (Pour.react)
	pv.reveal.out:Play()
	local hero, to = pv.hero.frame, pv.unlockRow.screen
	local hx, hy = hero:GetCenter()
	local tx, ty = to:GetCenter()
	if not (hx and tx) then return Pour.landHero(pv) end
	pv.heroMove:SetOffset(tx - hx, ty - hy)
	local k = to:GetHeight() / hero:GetHeight()
	pv.heroShrink:SetScaleTo(k, k)
	pv.heroFly:Play()
end

-- The next step opens (reveal_open_s in): its tile in its colour, its flash, its pop and its shockwave, the sparks
-- round it in its tone, the step-up's jingle.
function Pour.openStep(pv)
	local run = pv.run
	if not run or run.phase ~= "reveal" then return end
	local h = run.open
	if h then
		ns.Tree.SetTile(h.slot, h.slot.move, run.ev.quality, false)
		if h.slot.icon then h.slot.icon:SetVertexColor(1, 1, 1) end
		h.slot:SetAlpha(1)
		h.flashAnim:Play()
		h.pop:Play()
		h.shockAnim:Play()
		Pour.sparks(pv, h, 0, 0, run.newQ and 24 or 14, run.newQ and 80 or 55, run.tone, true)
	end
	Pour.jingle(run, ns.Sheet.UnlockSound(run.ev))
end

-- The step-up reveal over the figure's screen (buildReveal): the ladder round the step the hold climbed from (up to
-- POUR.tiles, the held ones muted, the ones ahead dark behind their locks, the current one no brighter than the rest),
-- the light along the rail to the next one, its lock's shake and flight, its opening (openStep), the new move rising
-- with its step, name and colour (a new colour's rays and a second burst at its badge), and the fold
-- (reveal_fold_s, reveal_fold_colour_s for a new colour).
function Pour.startReveal(pv)
	local run = pv.run
	local ev, rv = run.ev, pv.reveal
	run.phase = "reveal"
	local ladder, from = ev.ladder, ev.rung - 1
	local rungs = #ns.Ladders[ladder].rungs
	local count = math.min(POUR.tiles, rungs)
	local first = math.max(1, math.min(from - 2, rungs - count + 1))
	local tone = { ns.Sheet.QualityColor(ev.quality):GetRGB() }
	run.tone, run.newQ, run.open = tone, ns.Quality(ladder, from) ~= ev.quality, nil
	local cur
	for k, h in ipairs(rv.holders) do
		local r = first + k - 1
		local id = k <= count and (r == ev.rung and ev.move or ns.Anchor(ladder, r)) or nil
		h:SetShown(id ~= nil)
		if id then
			local ahead = r > from
			h.slot:SetScale(h.base)
			ns.Tree.SetTile(h.slot, id, nil, ahead)
			local tint = ahead and 0.35 or 0.75
			if h.slot.icon then h.slot.icon:SetVertexColor(tint, tint, tint) end
			h.slot:SetAlpha(ahead and 0.4 or 0.7)
			h.lock:SetShown(ahead and h.lockArt)
			h.lock:SetAlpha(1)
			h.lock:SetVertexColor(saga.note[1], saga.note[2], saga.note[3])
			h.flash:SetVertexColor(tone[1], tone[2], tone[3])
			h.shock:SetVertexColor(tone[1], tone[2], tone[3])
			if r == from then cur = k elseif r == ev.rung then run.open = h end
		end
	end
	local pitch = POUR.tile + POUR.tileGap
	rv.ladder:SetWidth(count * POUR.tile + (count - 1) * POUR.tileGap)
	rv.rail:ClearAllPoints()
	rv.rail:SetPoint("LEFT", rv.ladder, "LEFT", POUR.tile / 2, 0)
	rv.rail:SetPoint("RIGHT", rv.ladder, "RIGHT", -POUR.tile / 2, 0)
	local lit = cur and run.open
	if lit then
		local x0, x1 = cur * POUR.tile + (cur - 1) * POUR.tileGap, cur * pitch
		rv.lit:ClearAllPoints()
		rv.lit:SetPoint("LEFT", rv.ladder, "LEFT", x0, 0)
		rv.lit:SetWidth(x1 - x0)
		rv.light:ClearAllPoints()
		rv.light:SetPoint("CENTER", rv.ladder, "LEFT", x0, 0)
		rv.lightMove:SetOffset(x1 - x0, 0)
	end
	local id = ev.move or ns.Anchor(ladder, ev.rung)
	Kit.SetCaps(rv.cap, L["ui.pour.unlocked"]:format(ns.Step(ladder, ev.rung)))
	rv.name:SetText(id and ns.Sheet.MoveName(id) or L["ladder." .. ladder])
	rv.name:SetTextColor(tone[1], tone[2], tone[3])
	rv.badge:Set(ev.quality)
	rv.glow:SetShown(rv.glowArt)
	rv.glow:SetVertexColor(tone[1], tone[2], tone[3], 0.55)
	rv.rays:SetShown(run.newQ and rv.raysArt)
	rv.rays:SetVertexColor(tone[1], tone[2], tone[3], 0.6)
	F.figure.frame:Hide()
	rv:SetAlpha(0)
	rv:Show()
	for _, ag in ipairs(rv.plays) do
		if lit or (ag ~= rv.litAnim and ag ~= rv.lightAnim) then ag:Play() end
	end
	if run.newQ then
		rv.raysIn:Play()
		rv.spin:Play()
	end
	if run.open then run.open.lockAnim:Play() end
	if id then
		pv.hero.frame:SetAlpha(0)
		pv.hero.frame:Show()
		pv.hero:Play(id, "still")
		pv.heroIn:Play()
	end
	T.after("pourOpen", C.reveal_open_s, function() Pour.openStep(pv) end)
	if run.newQ then
		T.after("pourGem", C.reveal_open_s + 0.56, function()
			if pv.run == run and run.phase == "reveal" then Pour.sparks(pv, rv.badge, 0, 0, 10, 30, tone, true) end
		end)
	end
	T.after("pourFold", run.newQ and C.reveal_fold_colour_s or C.reveal_fold_s, function() Pour.foldReveal(pv) end)
end

-- What follows the landing: the reveal of a step the row opened, else the rows.
function Pour.nextStage(pv)
	local p = Lc.pump
	local run = pv.run
	if not (p and p.pour and p.committed and run and run.phase == "landed") then return end
	if run.ev then return Pour.startReveal(pv) end
	Pour.showRows(pv, false, false)
end

-- The pour landed (its row saved, Lc.Land): the bar and dial at the count, the rows made ready, the top's
-- jingle (or a skipped pour's record) where no step-up takes it, and 0.38 s on its reveal (0.3 s its rows; at once
-- after a click).
function Pour.landed(pv, p, slot)
	local run = pv.run
	Pour.landBar(pv)
	Pour.fillRows(pv, p, slot)
	if not run.ev and run.jingle == "top" then Pour.jingle(run, "star") end
	if not run.ev and run.jingle == "record" then Pour.jingle(run, "record") end
	run.phase = "landed"
	T.after("pourStage", run.skipped and 0 or run.ev and 0.38 or 0.3, function() Pour.nextStage(pv) end)
end

-- A click on the stage, once the ask has settled (answerSettled): lands one stage, the pour (its step-up still plays),
-- then the reveal or the rows at their end.
function Pour.skipPour()
	local p, v = Lc.pump, F.views.result
	local pv = v and v.pour
	local run = pv and pv.run
	if not (p and p.pour and run and answerSettled()) then return end
	if run.phase == "pour" then
		run.skipped = true
		Pour.stopGrains(pv)
		Lc.Land()
	elseif run.phase == "landed" then
		T.cancel("pourStage")
		Pour.nextStage(pv)
	elseif run.phase == "reveal" or run.phase == "rows" then
		Pour.endState(pv, p, Lc.Slot())
	end
end

-- The pour on the result view (renderResult, while p.pour): the figure dimmed beside the stage; a new run for a new
-- move or go (newRun) that starts pouring, or with the row already saved (the game's reduced motion: Lc.Save commits
-- at once) shows its end; the landing once the row is saved; a run stopped by a hidden dim starts again, or ends at
-- once once its row is saved. Every render lays the stage out again.
function Pour.renderPour(v, p, slot)
	local pv = v.pour
	placeColumn(v, false, LEFT_W)
	v.sideH = sideFigure(v, p, slot, "still")
	F.figure:Fade(0.3)
	F.figure.frame:SetScale(0.96)
	local key = p.id .. ":" .. p.index .. ":" .. (p.goAt or 0)
	local run = pv.run
	if not run or run.key ~= key then run = Pour.newRun(pv, p, slot, key) end
	Pour.placePour(v, pv)
	if run.phase == nil or run.phase == "stopped" then
		if p.committed then Pour.endState(pv, p, slot) else Pour.startPour(pv) end
	elseif run.phase == "pour" and p.committed then
		Pour.landed(pv, p, slot)
	elseif run.phase == "done" and p.committed then
		Lc.GoOn() -- again once a fight, a hold or a menu left the view, whole from there
	end
	Pour.placePour(v, pv)
	-- a draw while the rows stand keeps the character where its reaction put it (sideFigure showed the figure again)
	if (run.phase == "rows" or run.phase == "done") and p.committed then Pour.react(p) end
end

-- The Result view (play SPEC 6-10; boards result and goalpose): the count on the dial (slice 66), "Is that right?", a
-- counted set's form (slice 42), the whole move, the goal pose, the pace (slice 42), whether a hold could have gone
-- longer (slice 52), the pour (slices 65 and 66) or a set saved with no count (buildResult). Answers the content's
-- bottom.
local function renderResult(p, slot)
	local v = F.views.result
	local asking = p.asking == true and not p.committed
	local marking = p.marking == true and not p.committed
	local pacing = p.pacing == true and not p.committed
	local longer = p.longerAsk == true and not p.committed
	local pouring = p.pour ~= nil -- the move's count pours, saved or about to be (slices 65 and 66)
	local typing = not p.committed and not pouring and not asking and not marking and not pacing and not longer
		and (not timedKind(slot) or slot.counted == true)
	local form = typing and slot.counted == true -- a counted set's optional count and its three answers
	local name = moveName(slot)
	v.state = pouring and "pour" or form and "form" or typing and "count" or asking and "whole" or marking and "mark"
		or pacing and "pace" or longer and "longer" or "done"
	-- the moment an ask showed, from which its answers settle (answerSettled): stamped as the view comes into it for
	-- the move's run on (its go, so a run again after Back asks anew) and again as "Is that right?" comes or goes in it
	-- (p.really: a counted set's answers stand where its row stood), kept while it redraws in it (the count typed, a
	-- check read); the count is an ask too, its Save waiting as answers do (Panel.RecordInput)
	local shown = v.state ~= "done"
		and (p.id .. ":" .. p.index .. ":" .. (p.goAt or 0) .. ":" .. v.state .. (p.really and ":really" or "")) or nil
	if shown ~= v.askShown then v.askShown, v.askAt = shown, shown and GetTime() or nil end
	v.sideH = 0
	if pouring then
		setHead(v, name)
	elseif form then
		-- "All out and clean?" its title (slice 53), the move and its window under it (no goal: it had no number)
		setHead(v, L["ui.result.formAsk"], L["ui.result.intro"]:format(name, L["ui.result.window"]:format(C.count_s)))
	elseif typing then
		-- the question as the title (a per-side move's asks for the weaker side, never both added), the move and its
		-- goal under it
		local goal, hold = goalLine(slot), slot.kind == K.hold
		local ask = ns.Library[slot.move].side and L[hold and "ui.result.holdHintWeak" or "ui.result.hintWeak"]
			or L[hold and "ui.result.holdHint" or "ui.result.hint"]
		setHead(v, ask, goal ~= "" and L["ui.result.intro"]:format(name, goal) or name)
	elseif asking then
		-- when the player stopped, in the time the move ran ("You stopped at 0:05 of 0:45."; the Running view's clock
		-- showed the time left), both sides' seconds on a side move
		setHead(v, L["ui.result.wholeAsk"], L["ui.result.intro"]:format(name,
			L["ui.result.wholeClock"]:format(clock(p.elapsed or 0), clock(slot.seconds or 0))))
	elseif marking then
		readChecks(p, slot)
		setHead(v, L["ui.result.markAsk"], ns.MarkText(slot.move))
	elseif pacing then
		setHead(v, L["ui.result.paceAsk"], name)
	elseif longer then
		-- the move and the seconds just typed under the question
		setHead(v, L["ui.result.longerAsk"], L["ui.result.intro"]:format(name, ns.Sheet.Amount(p.pending.value, true)))
	else
		-- the set under the goal it was judged against: once recorded, p.last's, since the set may be the record now
		setHead(v, name, goalLine(slot, p.last))
	end
	-- the count on its dial with its - and +, the weight under it on a loaded move (practice stores nothing, so it
	-- takes none)
	for _, part in ipairs({ v.dialRoom, v.less, v.more, v.cap }) do part:SetShown(typing) end
	local loaded = typing and not slot.practice and ns.Loaded(slot.move) ~= nil
	placeLoad(v, slot, loaded)
	v.load:SetShown(loaded)
	if not loaded then v.kgNoneRow:Hide() end -- layResult shows it in the count's step
	for _, part in ipairs({ v.preview, v.really, v.reallyRow }) do part:SetShown(typing) end
	v.answers:SetShown(typing and not form)
	for _, part in ipairs({ v.cleanLabel, v.forms }) do part:SetShown(form) end
	v.empty:SetShown(typing and not form)
	v.couldNot:SetShown(typing and not form and not slot.practice)
	v.couldNot:SetText(L[slot.kind == K.hold and "ui.result.couldNotHold" or "ui.result.couldNot"])
	v.whole:SetShown(asking)
	v.marks:SetShown(marking)
	v.paces:SetShown(pacing)
	v.longers:SetShown(longer)
	if form then Kit.LayList(v.forms) end
	if marking then Kit.LayList(v.marks) end
	if pacing then Kit.LayList(v.paces) end
	if longer then Kit.LayList(v.longers) end
	if asking then Kit.LayList(v.whole) end
	-- the recorded set, told once: gold on a record (it was won), white otherwise; a counted set's new best too
	local done = v.state == "done"
	local beat = done and p.last ~= nil and p.last.outcome == O.beat
	v.outcome:SetText(done and outcomeLine(p, slot) or "")
	v.outcome:SetTextColor((beat and NORMAL_FONT_COLOR or HIGHLIGHT_FONT_COLOR):GetRGB())
	v.outcome:SetShown(done)
	v.bar:SetShown(beat)
	v.outcomeHover:SetShown(done and p.last ~= nil and p.last.earned ~= nil)
	v.nextLoad:SetText(nextLoadLine(p, slot))
	v.nextLoad:SetShown(done)
	v.after:SetText(done and afterLines(p, slot) or "")
	v.after:SetShown(done)
	v.rows = renderRewardRows(p, slot)
	v.rewardsHead:SetShown(done and v.rows > 0)
	v.rewardList:SetShown(done and v.rows > 0)
	-- the right column: the goal pose, held still and captioned "Goal pose", beside its ask; the steady move's or the
	-- hold's key pose, held still with no caption, beside the pace ask and the hold ask; the character beside a won set;
	-- else none, and the column takes the card's middle
	local win = done and react(p)
	v.pour:SetShown(pouring or typing)
	if done and p.last then Lc.GoOn() end -- a set saved with no pour, again once the view is the player's (renderPour)
	if pouring then
		Pour.renderPour(v, p, slot)
	elseif typing then
		-- the Running view's figure stays beside the dial (slice 66: the count is set where the move ran)
		placeColumn(v, false, LEFT_W)
		v.sideH = sideFigure(v, p, slot, "loop")
	elseif marking then
		placeColumn(v, false, LEFT_W)
		v.sideH = sideFigure(v, p, slot, "key")
	elseif pacing or longer then
		placeColumn(v, false, LEFT_W)
		v.sideH = sideFigure(v, p, slot, "still")
	elseif win then
		placeColumn(v, false, LEFT_W)
		hideFigure()
		F.screen:Hide()
		F.swap:Hide()
		F.modelScreen:Show()
		placeSide(v)
		v.sideH = SCREEN_H
	else
		placeColumn(v, done, done and LEFT_W or TM.TEXT)
		hideSide()
	end
	if typing then
		-- a fresh count once per move, at what the app counted (Lc.CountedBy: a hold's stopwatch, the voice's count of
		-- its reps), never past one set's most (a hold held past max_seconds), else none; the planned weight
		if p.inputFor ~= p.index then
			p.inputFor = p.index
			if loaded then F.kg:SetText(slot.load and ns.Sheet.Kg(slot.load) or "") end -- hidden: read by nothing
			local counted = Lc.CountedBy(p, slot)
			F.count, F.countFresh = counted and math.min(counted, ns.CountMax(slot)), true
		end
		Pour.drawCount(v, p, slot)
		previewWords(v, p, slot)
	else
		letGo()
	end
	-- the bar: Back until the count is saved, Record while it is entered, the way on once it is recorded; the last
	-- move's way out: a pump started from the Talents tab returns to its tree (Tree.Return), where no turn-in's pump
	-- queued behind it begins (Lc.QueueBegins)
	v.back:SetShown(not p.committed)
	if not typing then v.record:Hide() end
	v.next:SetShown(not typing and not asking and not marking and not pacing and not longer)
	-- a pour's way on reads as the move it goes to from the pour's start (slice 65)
	local last = (p.committed or pouring) and (p.index >= #p.moves or slot.practice)
	local back = last and p.fromTree ~= nil and not Lc.QueueBegins()
	if pouring and not last then
		v.next:SetText(L["ui.pour.next"]:format(moveName(p.moves[p.index + 1])))
	else
		v.next:SetText(L[back and "ui.result.backTree" or last and "ui.result.finish" or "ui.result.next"])
	end
	v.next:SetWidth(primaryWidth(v.next))
	return layResult(v)
end

-- The Pause menu's "I feel unwell" view (buildUnwellAsk). Answers the content's bottom.
local function renderUnwell()
	local v = F.views.unwellAsk
	hideSide()
	v.group:SetShown(Lc.MixedParty())
	Kit.LayList(v.choices)
	return Kit.Stack(v.left, Kit.Top(v), { { v.ask, 6 }, { v.choices, 12, x = (TM.TEXT - CHOICES_W) / 2 } })
end

local function sounds(p, slot)
	local mark = p.id .. ":" .. p.index .. ":" .. (p.side or 0) .. ":" .. p.state
	if Panel.soundMark == mark then return end
	local previous = Panel.soundMark
	Panel.soundMark = mark
	if p.state == "running" then
		Style.Play("go")
		Panel.lastWhole, Panel.goalChimed = nil, nil
	elseif p.state == "result" and previous == p.id .. ":" .. p.index .. ":" .. (p.side or 0) .. ":running"
		and timedKind(slot) and not p.asking
		and ((p.last or p.pending or {}).outcome == O.done or (slot.counted or p.marking) and not p.committed) then
		Style.Play("go") -- the end of a timed move that ran its time (saved, or pouring with its pending result; a
		-- counted set's: its count is asked next; a pose's: how close the player got)
	end
end

local function winSound(p)
	if not p.last or p.winPlayed == p.index or p.pour then return end -- a pour plays its own (renderPour's jingle)
	p.winPlayed = p.index
	if p.last.outcome == O.beat then
		Style.Play("record")
	elseif p.last.outcome == O.lastbeat or p.last.outcome == O.placement then
		Style.Play("win")
	end
end

local function showOnly(name)
	for key, v in pairs(F.views) do v:SetShown(key == name) end
end

-- The go once per run, keyed by its start as the digits are by the go (each side of a side move has its own), and
-- played on the shown view.
local function playGo(p)
	if Panel.goFor == p.runStart then return end
	Panel.goFor = p.runStart
	F.views.running.goAnim:Stop()
	F.views.running.goAnim:Play()
end

local function hideDim()
	cancelDigits()
	Pour.stopPour()
	T.cancel("display")
	Panel.whisper = nil -- the chat shows again
	if not F.dim then return end
	Kit.ShowGlow(F.views.ready.glow, false)
	F.views.running.goAnim:Stop() -- a go the menu or a popup cut short: the run comes back on its clock
	letGo()
	Kit.Leave(F.dim)
end

-- Where a move came from, for the card's title (David, 2026-10-12: "even with unfinished moves each should say at the
-- top for which quest it originally came from"): its quest's title, an unfinished move's own whichever pump pays it
-- and a merged pump's move its own, and until that title loads (Lc.QuestTitle) "Unfinished moves" or "Pump"; an
-- unfinished move a death owed (no quest) "Death tax", one of no quest otherwise "Unfinished moves"; nil for the
-- pump's own move of no quest (practice, the stand-up move, a pump asked for), which the pump's name titles.
local function origin(slot)
	if (slot.questID or 0) > 0 then
		return Lc.QuestTitle(slot.questID) or slot.owed and L["ui.panel.title.owed"] or L["ui.panel.title.pump"]
	end
	if not slot.owed then return nil end
	return ns.Reason(slot.owed.flags) == E.owedBy.death and L["log.death"] or L["ui.panel.title.owed"]
end

-- The card's title: where the move on it came from (origin), else the pump's name; then who whispered while the dim
-- hides the chat (until the dim lifts, never stored). The sender may be a secret value in a chat lockdown: it sits in
-- a table and is only formatted and shown, never tested.
local function cardTitle(p)
	local slot = p.moves[p.index]
	local name = slot and not p.chosen and origin(slot) or title(p)
	F.card:SetTitle(Panel.whisper and L["ui.panel.whisper"]:format(name, Panel.whisper.from) or name)
end

-- The card's place. The standard layout keeps its bottom edge for the whole pump where a card of the tier's budget,
-- CARD_Y above the screen's middle, would end, and no higher than lets a card of the tier's cap (or of the screen
-- less its dim's edge) stand SCREEN_MARGIN under the screen's top (112 on a 768-unit screen, 300 on one 1,080 high);
-- a taller view grows upward: Start, Done, Record and Next, and a row of Skips (David, 2026-10-04: "you just need to
-- press 31 times then"), stand under the pointer one after the other. A click that changes the view it is on keeps
-- the card's top instead (keepingTop: How's bar, the guide's Read, a swap), so the control clicked stays under the
-- pointer, the card growing or shrinking at its bottom, and the bottom it ends at is the one kept for the rest of the
-- pump (p.cardBottom). Every card stays inside the screen, its top SCREEN_MARGIN in: one still taller once Kit.Fit has
-- scaled it moves its bottom down. The Streamer layout's card stands in its corner or where the player dragged it,
-- growing down; after a Skip the Ready views that follow keep its bottom-left corner where it was, inside the screen.
-- Every place is in UIParent's units, and an offset is in the card's own, so each is divided by the card's scale
-- (Kit.Fit's).
function placeCard(p)
	local card = F.card
	local scale = card:GetScale()
	local tall = card:GetHeight() * scale
	local room = UIParent:GetHeight()
	card:ClearAllPoints()
	if not Panel.Streamer() then
		p.skipBottom, p.skipLeft = nil, nil
		local bottom = p.keepTop and p.keepTop - tall or p.cardBottom or math.min(room / 2 + CARD_Y - TM.budget / 2,
			room - SCREEN_MARGIN - math.min(TM.cap, room - 2 * TM.edge))
		bottom = math.max(0, math.min(bottom, room - SCREEN_MARGIN - tall))
		if p.keepTop then p.cardBottom, p.keepTop = bottom, nil end
		card:SetPoint("BOTTOM", UIParent, "BOTTOM", 0, bottom / scale)
		return
	end
	p.cardBottom = nil -- the standard layout's kept bottom; the Streamer layout keeps its corner
	if p.skipBottom and p.state == "open" and p.view == "ready" then
		local y = math.max(0, math.min(p.skipBottom, room - tall))
		if p.skipLeft then
			card:SetPoint("BOTTOMLEFT", UIParent, "BOTTOMLEFT", p.skipLeft / scale, y / scale)
		else
			card:SetPoint("BOTTOM", UIParent, "BOTTOM", 0, y / scale)
		end
		return
	end
	p.skipBottom, p.skipLeft = nil, nil
	local pos = ns.profile.cardPos
	if type(pos) == "table" and pos.x and pos.y then
		card:SetPoint("TOPLEFT", UIParent, "TOPLEFT", pos.x / scale, pos.y / scale)
	else
		card:SetPoint(CORNER.point, UIParent, CORNER.point, CORNER.x / scale, CORNER.y / scale)
	end
end

function Panel.Render()
	local p = Lc.pump
	Panel.RenderStopCard(p)
	-- a pump on its way (the 1-s wait, the "Stop to pump" card): its first move's sheets start loading now, so the
	-- figure's first frame is never blank
	if p and p.moves[1] and (p.state == "ready" or p.state == "card") and not Lc.combat then
		Panel.Build()
		F.figure:Preload(p.moves[1].move)
	end
	if not Lc.Visible() then
		hideDim()
		return
	end
	Panel.Build()
	local slot = p.moves[p.index]
	-- The Streamer layout (slice 23): no dim and no shadow, so a stream never goes black, and the dim at FULLSCREEN
	-- under the card; at alpha 0 it still takes the mouse and holds the only key frame, so the pump asks at once as in
	-- the standard layout and no click reaches the world.
	local streamer = Panel.Streamer()
	F.dim.bg:SetColorTexture(0, 0, 0, streamer and C.dim_streamer or IsInGroup() and C.dim_group or C.dim_alone)
	F.dim.shadow:SetShown(not streamer)
	local strata = streamer and "FULLSCREEN" or "FULLSCREEN_DIALOG"
	if F.dim:GetFrameStrata() ~= strata then F.dim:SetFrameStrata(strata) end
	F.card.grip:SetShown(streamer)
	cardTitle(p)
	-- the dim and the card in it fade in as they come (slice 72); the dim takes the mouse and the key frame its keys
	-- from the first frame
	Kit.Appear(F.dim)
	F.keys:SetShown(not p.taxi and not InCombatLockdown() and Panel.InputMode() ~= "mouse")
	if not InCombatLockdown() then F.keys:SetPropagateKeyboardInput(true) end
	if p.state ~= "countdown" then cancelDigits() end
	if p.state ~= "result" then
		letGo() -- Back from the result view: the hidden weight box lets go of the keyboard, so Space starts again
	end
	sounds(p, slot)
	F.moveCard:Hide()
	local name, bottom
	if p.state == "open" then
		p.inputFor = nil -- a run after Back starts its count afresh
		name = p.view == "unwellAsk" and "unwellAsk" or "ready"
		bottom = name == "ready" and renderReady(p, slot) or renderUnwell()
	elseif p.state == "countdown" then
		name = "countdown"
		bottom = renderCountdown(p, slot)
	elseif p.state == "running" then
		name = "running"
		bottom = renderRunning(p, slot)
		updateRunning()
	else
		name = "result"
		bottom = renderResult(p, slot)
		winSound(p)
	end
	if p.state ~= "countdown" then Panel.countdownFor = nil end
	if name ~= "result" or not p.pour then Pour.stopPour() end
	fitCard(F.views[name], bottom)
	showOnly(name)
	Kit.ShowGlow(F.views.ready.glow, name == "ready")
	F.card.snooze:SetShown(p.state == "open" and p.view == "ready")
	-- The run's go, and one display ticker while the running clock counts.
	if p.state == "running" then
		playGo(p)
		T.every("display", DISPLAY_TICK_S, function()
			local q = Lc.pump
			if not q or not Lc.Visible() or q.state ~= "running" then return T.cancel("display") end
			updateRunning()
		end)
	else
		T.cancel("display")
	end
end

-- The non-modal "Stop to pump" card: no dim, no mouse, no keyboard ------------------------------------------------

function Panel.BuildStopCard()
	local card = CreateFrame("Frame", nil, UIParent, "TooltipBackdropTemplate")
	card:SetSize(280, 52)
	card:SetPoint("TOP", 0, -140)
	card:SetFrameStrata("HIGH")
	card:EnableMouse(false)
	card:Hide()
	card.text = text(card, "large", "CENTER")
	card.text:SetPoint("TOP", 0, -9)
	card.text:SetText(L["ui.card.stop"])
	card.sub = text(card, "bodyWhite", "CENTER")
	card.sub:SetPoint("TOP", card.text, "BOTTOM", 0, -3)
	F.stop = card
end

-- It fades in, and the first time a pump shows it the attention cue says so (slice 72: with no dim, nothing else
-- tells a player who looks at the road that a pump waits; `p.stopCued`, which the voice's own fallback cue reads).
function Panel.RenderStopCard(p)
	if not F.stop then return end
	if not (p ~= nil and p.state == "card" and not Lc.combat) then return Kit.Leave(F.stop) end
	F.stop.sub:SetText(ns.Plural("ui.card.moves", #p.moves):format(#p.moves))
	F.stop:SetHeight(math.max(52, 9 + F.stop.text:GetStringHeight() + 3 + F.stop.sub:GetStringHeight() + 9))
	if Kit.Appear(F.stop) and not p.stopCued then
		p.stopCued = true
		Style.Play("attention")
	end
end

-- A mixed party's question (Lc.AskParty; David, 2026-10-04: "Yes offer the options at the first return. Bank all or
-- Pause during this group up."): QuestPump's own dialog (Popups.lua) with the two answers, both grey (`answers`: an
-- ask's answers are never red, and neither is pushed), what each does in its tooltip. Only "Pause while grouped"
-- pauses; "Bank all", Escape and the timeout all hide it, and a hide without the pause is Bank all (the dialog runs
-- OnHide on every close, after OnAccept). A dialog that cannot show (a cinematic) leaves the group to be asked again.
ns.Dialog.Register("QUESTPUMP_PARTY", {
	text = L["ui.party.ask"],
	button1 = L["ui.party.pause"],
	button2 = L["ui.party.bank"],
	answers = true,
	tip1 = L["ui.party.pause.tip"],
	tip2 = L["ui.party.bank.tip"],
	OnAccept = function() Lc.PartyAnswer(true) end,
	OnHide = function() Lc.PartyAnswer(false) end,
	timeout = C.party_ask_s,
})

ns.On("PartyAsk", function(asking)
	if not asking then return ns.Dialog.Hide("QUESTPUMP_PARTY") end
	if not ns.Dialog.Show("QUESTPUMP_PARTY") then Lc.PartyAnswer(nil) end
end)

-- The setup window: the companions, strongly recommended, then Your setup ------------------------------------------

-- Spoken's welcome window's lessons (2026-10-15; the Saga's D boards), built from the kit (Kit.lua) at its large tier
-- (TL): one window in the game's own frame, each page's question as its title over the Options window's faded rule with
-- no line under it, a picture and a name for every choice, gold only for what is chosen or done, the details in
-- tooltips, and one bottom bar: the way back (or Play without, named by what the player goes without) on the left, the
-- step's dots in the middle, and the way on in the red primary on the right. Each page lays itself out at its render
-- from its words' heights (they follow the client's Text Size), so a larger Text Size makes the window taller, never
-- its words overlap; 680 units fit a 768-unit screen at scale 1, and a taller window scales down to stay on the screen
-- (Kit.Fit). SU holds the setup's own content: its pictures, its pages and their renders.
local TL = Kit.TIER.L
local SU = {}

-- The pictures: the moves whose key pose stands for a choice (the paces, the strength training so far; the switches'
-- are Settings.lua's, ns.Settings.PICTURE), by library key; the companions' cards' and the "With them" chips' art is
-- Style.lua's.
SU.PACE = { light = { "pushup" }, balanced = { "air_squat", "pushup" }, extreme = { "air_squat", "pushup", "plank" } }
SU.EXPERIENCE = { none = "wall_pushup", some = "knee_pushup", training = "pushup" }
SU.PHONE = "pushup"

-- The companions' page ("Strongly recommended companions", the legal minimum's words in its title): each companion's
-- card (its picture, its name, its state with the check or the cross, the state's words saying where to get it while
-- it was never set up, and under them the remedy for a state that has one; the details in its tooltip; gold edged once
-- done), what they give as four chips (the long words in their tooltip), and the bar's Check again and Play without.
-- The cards and the chips are no buttons: nothing on them answers a click (2026-10-15). While the setup asks for no
-- phone (Setup.AsksPhone: no helper yet, or the phone apps are not in the stores) the page is the helper's alone: its
-- card centred, the title and the chips' words in the singular, and only the chips the helper gives
-- (SU.RenderCompanions).
function SU.Companions(s)
	local v = Kit.Page(s, L["ui.setup.welcome"])
	local w = Kit.Grid(2, TL.TEXT)
	v.cards = {}
	for i = 1, 2 do
		local c = Kit.Card(v, w, 292, function(tooltip)
			local done = select(i, Setup.Steps())
			local key = done and "ui.setup.step" .. i .. ".done" or "ui.setup.step" .. i .. ".tip"
			if i == 1 and not done and not Setup.AsksPhone() then key = "ui.setup.step1.tip.one" end
			GameTooltip_SetTitle(tooltip, L["ui.setup.step" .. i])
			tooltip:AddLine(L[key], 1, 1, 1, true)
		end)
		c.screen = Kit.Screen(c, w - 24, 164)
		c.screen:SetPoint("TOP", 0, -12)
		c.name = Kit.Text(c, "cardTitle", "LEFT", nil, w - 24)
		c.name:SetPoint("TOPLEFT", c.screen, "BOTTOMLEFT", 0, -14)
		c.name:SetText(L["ui.setup.step" .. i])
		c.mark = c:CreateTexture(nil, "ARTWORK")
		c.mark:SetSize(18, 18)
		c.mark:SetPoint("TOPLEFT", c.name, "BOTTOMLEFT", 0, -9)
		c.state = Kit.Text(c, "large", "LEFT", nil, w - 24 - 26)
		c.state:SetPoint("TOPLEFT", c.mark, "TOPRIGHT", 8, -1)
		c.how = Kit.Text(c, "bodyWhite", "LEFT", Style.saga.note, w - 24)
		v.cards[i] = c
	end
	-- QuestPump on this computer: the addon's icon; the phone app: the push-up's key pose with the heart and the clock
	local icon = v.cards[1].screen:CreateTexture(nil, "ARTWORK")
	icon:SetSize(Style.iconPx, Style.iconPx)
	icon:SetPoint("CENTER")
	icon:SetTexture(Style.mark)
	local phone = v.cards[2]
	phone.figure = Kit.Figure(phone.screen, SU.PHONE, C.figure_ready_px, -6)
	for _, part in ipairs({ { "heart", "TOPLEFT", 14, -12 }, { "clock", "TOPRIGHT", -14, -12 } }) do
		local t = phone.screen:CreateTexture(nil, "OVERLAY")
		t:SetPoint(part[2], part[3], part[4])
		t:SetShown(Style.FitAtlas(t, part[1], 26))
	end
	v.withHead = Kit.Section(v, L["ui.setup.with"], true)
	v.chips = {}
	local tip = Kit.Tip(function()
		return L[Setup.AsksPhone() and "ui.setup.with" or "ui.setup.with.one"]
	end, function()
		if Setup.AsksPhone() then return L[Setup.SaveUnread() and "ui.setup.gives.forever" or "ui.setup.gives"] end
		return L[Setup.SaveUnread() and "ui.setup.gives.one.forever" or "ui.setup.gives.one"]
	end)
	for i = 1, 4 do v.chips[i] = Kit.Chip(v, (Kit.Grid(4, TL.TEXT)), nil, nil, tip) end
	return v
end

-- The chips' words and pictures: on a client that showed it does not read its save back the kept progress first,
-- elsewhere the streak on days without the game; then the countdown on the wrist, the heart-rate check marks and the
-- phone's catch-up (David, 2026-10-09: what wins the player first). Each picture is a flat atlas of Style.atlas
-- (Kit.ChipArt), left out where the client lacks it, but the streak's, which falls back to QuestPump's own mark.
-- `phone`: a chip only the phone app gives, left out while the setup asks for no phone (the helper's own pumps keep the
-- streak, and a strap on it gives the check marks).
SU.CHIPS = { { "kept", "kept" }, { "wrist", "clock", phone = true }, { "heart", "heart" },
	{ "catchup", "phoneQuest", phone = true } }
SU.STREAK_CHIP = { "streak", "streak" }

-- The x of the i-th of `row`'s frames (one width each, 10 apart) centred in the page's content width: 0 for the first
-- of a full row.
function SU.Centred(v, row, i)
	local w = row[1]:GetWidth()
	return (v.contentW - (#row * w + (#row - 1) * 10)) / 2 + (i - 1) * (w + 10)
end

-- How each companion's step stands (Setup.Steps, ns.HelperState, ns.PhoneQuiet): its mark, its state's word in green
-- or red, and the remedy under it where the state has one (the helper stopped, blind, turned off or out of date; the
-- phone quiet since a date); both done: no Play without, no Check again, and Next on. Play without names what the
-- player goes without: the saved progress on a client that showed it does not read its save back while no helper
-- runs, the helper on a page that asks for it alone, the phone while the helper runs, else both. The cards and the
-- chips shown are centred in the content's width (all of it with every one). Returns whether the way on is Next and
-- the content's bottom.
function SU.RenderCompanions(s, v)
	local one = not Setup.AsksPhone()
	v.title:SetText(L[one and "ui.setup.welcome.one" or "ui.setup.welcome"])
	Kit.SetCaps(v.withHead.words, L[one and "ui.setup.with.one" or "ui.setup.with"])
	v.cards[2]:SetShown(not one)
	if not one then Kit.Pose(v.cards[2].figure) end
	local state, quiet = ns.HelperState(), ns.PhoneQuiet()
	local done = { Setup.Steps() }
	local words = {
		{ state = L["ui.setup.step1.state." .. (state or "none")],
			how = state and state ~= "on" and L["ui.setup.step1." .. state] or nil },
		{ state = quiet and L["ui.setup.step2.state.quiet"] or L[done[2] and "ui.setup.step2.state.done"
				or "ui.setup.step2.state.none"],
			how = quiet and L["ui.setup.step2.quiet"]:format(ns.Sheet.ShortDate(quiet)) or nil },
	}
	local cards = one and { v.cards[1] } or v.cards
	local y, cardH = Kit.Top(v), 0
	for i, c in ipairs(cards) do
		Style.SetMark(c.mark, done[i])
		c.state:SetText(words[i].state)
		c.state:SetTextColor((done[i] and GREEN_FONT_COLOR or RED_FONT_COLOR):GetRGB())
		local how = words[i].how
		c.how:SetText(how or "")
		c.how:SetShown(how ~= nil)
		-- the state wraps on a narrow card ("Pair it in QuestPump on this computer"): the remedy goes under its last line
		c.how:SetPoint("TOPLEFT", c.mark, "BOTTOMLEFT", 0, -7 - math.max(0, c.state:GetStringHeight() - 18))
		Kit.Choose(c, done[i])
		c.name:SetTextColor((done[i] and NORMAL_FONT_COLOR or HIGHLIGHT_FONT_COLOR):GetRGB())
		cardH = math.max(cardH, 12 + 164 + 14 + c.name:GetStringHeight() + 9
			+ math.max(18, c.state:GetStringHeight()) + (how and 7 + c.how:GetStringHeight() or 0) + 14)
	end
	for i, c in ipairs(cards) do
		c:SetHeight(cardH)
		Kit.At(c, v, y, SU.Centred(v, cards, i))
	end
	y = y + cardH + 18
	Kit.At(v.withHead, v, y)
	y = y + 28
	local unread, chipH = Setup.SaveUnread(), 88
	local chips = {}
	for i, tile in ipairs(v.chips) do
		local gives = (i == 1 and not unread) and SU.STREAK_CHIP or SU.CHIPS[i]
		local shown = not (one and gives.phone)
		tile:SetShown(shown)
		if shown then
			chips[#chips + 1] = tile
			tile.words:SetText(L["ui.setup.chip." .. gives[1]])
			Kit.ChipArt(tile.icon, gives[2], gives[2] == "streak")
			chipH = math.max(chipH, Kit.ChipHeight(tile))
		end
	end
	for i, tile in ipairs(chips) do
		tile:SetHeight(chipH)
		Kit.At(tile, v, y, SU.Centred(v, chips, i))
	end
	local open = not (done[1] and done[2])
	-- with the helper running its copy keeps the progress and a strap on it adds the check marks (neither needs the
	-- screen): playing without the phone gives up neither
	local helper = Setup.HelperRuns()
	s.foot:Label(s.without, L[unread and not helper and "ui.setup.without.forever"
		or one and "ui.setup.without.one" or helper and "ui.setup.without.helper" or "ui.setup.without"])
	s.without:SetShown(open)
	s.check:SetShown(open)
	return not open, y + chipH
end

-- Where do you play: the room's three tiles (Floor moves, Room to move, Quiet mode), each its key pose and its name
-- (the Body page's tip in its tooltip), with the pool they leave beside "Your room"; then the Body switches as a list
-- of rows, each its picture, name and check box (ns.Settings.BuildSwitchRows, the Body page's too). A tile's figure is
-- 150 units in a 128-high screen, 8 up from its middle: the standing poses' cells fill the pack's 192-pixel box, and
-- centred the skater hops' feet (to pixel 187) and the air squat's (to 188) fell under the screen's foot (measured on
-- the pack's key cells, 2026-10-15), while 8 up keeps the skater's head (from pixel 26) inside.
SU.PLACE_FIGURE, SU.PLACE_SCREEN, SU.PLACE_UP = 150, 128, 8

-- The pool the setup leaves (ns.Settings.Pool) beside a page's section name, large and gold.
function SU.PoolRow(v, name)
	v.pool = Kit.NumberRow(v, name, L["ui.setup.pool"], { NORMAL_FONT_COLOR:GetRGB() })
end

function SU.LayPool(v, y)
	return Kit.LayNumber(v.pool, y, tostring((ns.Settings.Pool(ns.profile, ns.state))))
end

function SU.Place(s)
	local v = Kit.Page(s, L["ui.setup.place"])
	SU.PoolRow(v, L["ui.setup.room"])
	v.tiles = {}
	local w = Kit.Grid(3, TL.TEXT)
	for _, name in ipairs(ns.Settings.PLACE) do
		local c = Kit.Choice(v, w, 214, function()
			local on = not ns.Settings.SwitchOn(name)
			Style.Play(on and "checkOn" or "checkOff")
			ns.Settings.SetSwitch(name, on)
		end, Kit.Tip(L["gear." .. name], L["settings.switch." .. name .. ".tip"]))
		c.screen = Kit.Screen(c, w - 20, SU.PLACE_SCREEN)
		c.screen:SetPoint("TOP", 0, -10)
		c.view = Kit.Figure(c.screen, ns.Settings.PICTURE[name], SU.PLACE_FIGURE, SU.PLACE_UP)
		c.box = Kit.Box(c.screen, 24)
		c.box:SetPoint("TOPRIGHT", -7, -7)
		c.name = Kit.Text(c, "cardTitle", "LEFT", nil, w - 20)
		c.name:SetPoint("TOPLEFT", c.screen, "BOTTOMLEFT", 0, -11)
		c.name:SetText(L["gear." .. name])
		c.switch = name
		v.tiles[#v.tiles + 1] = c
	end
	v.bodyHead = Kit.Section(v, L["settings.body"])
	v.list = Kit.List(v, TL.TEXT)
	ns.Settings.BuildSwitchRows(v.list, ns.Settings.BODY, { font = "bodyWhite" })
	return v
end

function SU.RenderPlace(_, v)
	local y = Kit.Top(v)
	y = y + SU.LayPool(v, y) + 10
	local tileH = 0
	for _, c in ipairs(v.tiles) do
		Kit.Pose(c.view)
		Kit.Choose(c, ns.Settings.SwitchOn(c.switch))
		tileH = math.max(tileH, 10 + SU.PLACE_SCREEN + 11 + c.name:GetStringHeight() + 12)
	end
	for i, c in ipairs(v.tiles) do
		c:SetHeight(tileH)
		Kit.At(c, v, y, (i - 1) * (c:GetWidth() + 11))
	end
	y = y + tileH + 16
	Kit.At(v.bodyHead, v, y)
	y = y + 26
	local listH = ns.Settings.RenderSwitchRows(v.list)
	Kit.At(v.list, v, y)
	return true, y + listH
end

-- What you have at hand: the pieces with the pool they leave (ns.Settings.BuildGear, the Equipment page's too).
function SU.Gear(s)
	local v = Kit.Page(s, L["ui.setup.gear"])
	SU.PoolRow(v, L["settings.equipment"])
	ns.Settings.BuildGear(v, TL.TEXT)
	return v
end

function SU.RenderGear(_, v)
	local y = Kit.Top(v)
	return true, ns.Settings.RenderGear(v, y + SU.LayPool(v, y) + 10)
end

-- How many moves per quest: the three paces as tiles (one, two or three key poses; the name with its count of moves),
-- the strength training so far while it is asked (Setup.AsksExperience; a push-up from the wall to the floor beside
-- each answer, its name), when pumps ask as two tiles (Lc.AskWhen, slice 70: at the turn-in or on the next flight; the
-- General page's row and its tooltip's words), the voice's "Count reps aloud" as a row, and before the start the
-- safety note, word for word, under them.
function SU.Hard(s)
	local v = Kit.Page(s, L["ui.setup.hard"])
	v.paces = {}
	local w = Kit.Grid(3, TL.TEXT)
	for i, key in ipairs(E.paceKeys) do
		local c = Kit.Choice(v, w, 160, function()
			if ns.Pace(ns.profile.pace) == key then return end -- a chosen answer stays chosen and writes nothing
			Style.Play("checkOn")
			ns.Settings.Set("pace", key)
		end)
		c.screen = Kit.Screen(c, w - 20, 72)
		c.screen:SetPoint("TOP", 0, -10)
		c.icons = {}
		local moves = SU.PACE[key]
		for k, moveKey in ipairs(moves) do
			local t = Kit.Icon(c.screen, Kit.MoveByKey(moveKey), Style.iconPx)
			t:ClearAllPoints()
			t:SetPoint("CENTER", (k - (#moves + 1) / 2) * 56, 0)
			c.icons[k] = t
		end
		c.box = Kit.Box(c.screen, 22)
		c.box:SetPoint("TOPRIGHT", -6, -6)
		c.name = Kit.Text(c, "cardTitle", "LEFT", nil, w - 20)
		c.name:SetPoint("TOPLEFT", c.screen, "BOTTOMLEFT", 0, -10)
		c.name:SetText(L["ui.setup.pace." .. key])
		c.value = key
		v.paces[i] = c
	end
	v.expHead = Kit.Section(v, L["experience"])
	v.exp = {}
	for i, key in ipairs(E.experienceKeys) do
		local c = Kit.Choice(v, w, 52, function()
			if ns.profile.experience == key then return end
			Style.Play("checkOn")
			ns.Settings.Set("experience", key)
		end)
		c.screen = Kit.Screen(c, 44, 40)
		c.screen:SetPoint("TOPLEFT", 6, -6)
		c.icons = { Kit.Icon(c.screen, Kit.MoveByKey(SU.EXPERIENCE[key]), 40) }
		-- the name on the tile's middle line, wrapping clear of the check box as a larger Text Size grows it
		c.name = Kit.Text(c, "bodyWhite", "LEFT", nil, w - 44 - 6 - 10 - 30)
		c.name:SetPoint("LEFT", 6 + 44 + 10, 0)
		c.name:SetText(L["experience." .. key])
		c.box = Kit.Box(c, 20)
		c.box:SetPoint("TOPRIGHT", -8, -8)
		c.value = key
		v.exp[i] = c
	end
	v.askHead = Kit.Section(v, L["settings.askWhen"])
	v.ask = {}
	local half, gap = Kit.Grid(2, TL.TEXT)
	v.askGap = gap
	for i, key in ipairs(Lc.ASK_WHEN) do
		local c = Kit.Choice(v, half, 44, function()
			if Lc.AskWhen() == key then return end
			Style.Play("checkOn")
			ns.Settings.Set("askWhen", key)
		end, Kit.Tip(L["settings.askWhen"], L["settings.askWhen.tip"]))
		c.name = Kit.Text(c, "bodyWhite", "LEFT", nil, half - 12 - 12 - 30)
		c.name:SetPoint("LEFT", 12, 0)
		c.name:SetText(L["settings.askWhen." .. key])
		c.box = Kit.Box(c, 22)
		c.box:SetPoint("RIGHT", -12, 0)
		c.value = key
		v.ask[i] = c
	end
	v.voiceHead = Kit.Section(v, L["settings.speech"])
	v.voice = Kit.Choice(v, TL.TEXT, 44, function()
		local on = ns.profile.countReps ~= true
		Style.Play(on and "checkOn" or "checkOff")
		ns.Settings.Set("countReps", on)
	end, Kit.Tip(L["settings.countReps"], L["settings.countReps.tip"]))
	v.voice.name = Kit.Text(v.voice, "bodyWhite", "LEFT", nil, TL.rowName - 12)
	v.voice.name:SetPoint("LEFT", 12, 0)
	v.voice.name:SetText(L["settings.countReps"])
	v.voice.box = Kit.Box(v.voice, 22)
	v.voice.box:SetPoint("RIGHT", -12, 0)
	v.safety = Kit.Text(v, "bodyWhite", "CENTER", Style.saga.note, TL.TEXT)
	v.safety:SetText(L["safety.note"])
	return v
end

function SU.RenderHard(_, v)
	local asks = Setup.AsksExperience()
	local pace = ns.Pace(ns.profile.pace)
	local y, tileH = Kit.Top(v), 0
	for _, c in ipairs(v.paces) do
		Kit.Choose(c, pace == c.value)
		tileH = math.max(tileH, 10 + 72 + 10 + c.name:GetStringHeight() + 12)
	end
	for i, c in ipairs(v.paces) do
		c:SetHeight(tileH)
		Kit.At(c, v, y, (i - 1) * (c:GetWidth() + 11))
	end
	y = y + tileH + 18
	v.expHead:SetShown(asks)
	local expH = 52
	for _, c in ipairs(v.exp) do
		c:SetShown(asks)
		Kit.Choose(c, ns.profile.experience == c.value)
		expH = math.max(expH, 8 + c.name:GetStringHeight() + 10)
	end
	if asks then
		Kit.At(v.expHead, v, y)
		for i, c in ipairs(v.exp) do
			c:SetHeight(expH)
			Kit.At(c, v, y + 24, (i - 1) * (c:GetWidth() + 11))
		end
		y = y + 24 + expH + 18
	end
	Kit.At(v.askHead, v, y)
	local askWhen, askH = Lc.AskWhen(), 44
	for _, c in ipairs(v.ask) do
		Kit.Choose(c, askWhen == c.value)
		askH = math.max(askH, c.name:GetStringHeight() + 12)
	end
	for i, c in ipairs(v.ask) do
		c:SetHeight(askH)
		Kit.At(c, v, y + 24, (i - 1) * (c:GetWidth() + v.askGap))
	end
	y = y + 24 + askH + 18
	Kit.At(v.voiceHead, v, y)
	Kit.Choose(v.voice, ns.profile.countReps == true)
	v.voice:SetHeight(math.max(44, v.voice.name:GetStringHeight() + 12))
	Kit.At(v.voice, v, y + 24)
	y = y + 24 + v.voice:GetHeight()
	-- the safety note before the start, under the answers
	local starting = not Lc.TrialBegun()
	v.safety:SetShown(starting)
	if starting then
		v.safety:ClearAllPoints()
		v.safety:SetPoint("TOP", v, "TOP", 0, -(y + 16))
		y = y + 16 + v.safety:GetStringHeight()
	end
	return true, y
end

SU.BUILD = { companions = SU.Companions, place = SU.Place, gear = SU.Gear, hard = SU.Hard }
SU.RENDER = { companions = SU.RenderCompanions, place = SU.RenderPlace, gear = SU.RenderGear, hard = SU.RenderHard }

function Panel.BuildSetup()
	-- Escape is the close button (Setup.Dismiss; David, 2026-10-03), every other key goes on to the game: the window's
	-- own Escape-only key frame, out of UISpecialFrames, whose CloseSpecialWindows would only hide it
	local s = Kit.Window("QuestPumpSetup", TL, L["ui.setup.title"], { escape = "own",
		onClose = function() Setup.Dismiss() end })
	s.views = {}
	for name, build in pairs(SU.BUILD) do
		-- each page's title over the rule alone: no line under it (2026-10-22)
		s.views[name] = build(s)
		s.views[name].intro:Hide()
	end
	-- the bar: its rule, the step's dots in the middle; the companions' page's Play without and its Check again (the
	-- reload that reads the mirror); Your setup's Back and Next (Start, or Done once the game has started)
	s.foot = Kit.Footer(s)
	-- "Play without": small, secondary, never the default, named by what the player goes without (RenderCompanions);
	-- one click chooses it (David, 2026-10-03: "generally to press 'play without them' is enough")
	s.without = s.foot:Secondary(L["ui.setup.without"], function() Setup.PlayWithout() end, 150)
	s.back = s.foot:Secondary(L["ui.setup.back"], function() Setup.Go(-1) end)
	s.check = s.foot:Primary(L["ui.setup.check"], function() C_UI.Reload() end)
	Kit.Hover(s.check, Kit.Tip(L["ui.setup.check"], L["ui.setup.check.tip"]))
	s.next = s.foot:Primary(L["ui.setup.next"], function() Setup.Go(1) end, "tab")
	F.setup = s
end

function Panel.RenderSetup()
	-- the window waits out a cinematic (a new character's intro) the way a pump does, and a pump itself (its Escape-only
	-- key frame would take the Escape meant for the game menu under the dim) (Lc.combat too: lockdown begins only after
	-- PLAYER_REGEN_DISABLED, while this already runs)
	local page, at, of = Setup.Page()
	local show = page ~= nil and not Lc.combat and not InCombatLockdown() and not Lc.holds.cinematic
		and not Lc.holds.movie and not InCinematic() and not Lc.Visible()
	-- built the first time it shows (most players see it only on their first days), never at every login
	if show and not F.setup then Panel.BuildSetup() end
	local s = F.setup
	if not s then return end
	if show then
		local opening = not s:IsShown()
		if opening then
			-- before each show, out of combat here: a stale Escape never swallows a key
			s.keys:SetPropagateKeyboardInput(true)
			Style.Play("open")
		end
		-- the companions' cadence counts the showings of their page (Setup.Shown)
		if page == "companions" and (opening or s.page ~= page) then Setup.Shown() end
		s.page = page
		for name, v in pairs(s.views) do v:SetShown(name == page) end
		-- the companions' page sets its own way out (RenderCompanions); Your setup's pages have none of it
		local companions = page == "companions"
		if not companions then
			s.without:Hide()
			s.check:Hide()
		end
		local onward, bottom = SU.RENDER[page](s, s.views[page])
		-- every page counts, the companions' too (Setup.Step; the D boards' dots); a window of one page has no step;
		-- Back on every page after the first, after Play without too (Setup.Go opens the companions' again)
		local step, steps = Setup.Step()
		s.back:SetShown(not companions and step > 1)
		s.next:SetShown(onward)
		s.next:SetText(L[at < of and "ui.setup.next" or Lc.TrialBegun() and "ui.setup.done" or "ui.setup.start"])
		s.next.sound = at < of and "tab" or "start"
		s.foot:Step(step, steps)
		-- the window as tall as its page and its bar, at least the tier's height, and scaled down to stay inside the
		-- screen where a large Text Size outgrows it
		Kit.Fit(s, bottom, s.foot)
	end
	s:SetShown(show)
end

-- The day-two check-in -----------------------------------------------------------------------------------------------

-- The kit's card window (the step card's size: a short ask needs no more), in the care tone (no gold, no alert, red
-- only for the bar's way on): "Muscles today?" over its three answers as care rows, the words white, a click
-- answering (three red buttons until 2026-10-16). The third, "Very sore, or dark urine", turns it to "Where does it
-- hurt?", the joints as two lists of four care rows (the Body page's look), and Confirm, which flags the ones ticked. A
-- joint flagged already shows ticked and unavailable, its tooltip saying where it clears (the Body page: a pain flag is
-- a safety filter, which the check-in never lifts). Closed only by its answers, as before the kit: no close button and
-- no Escape of its own, so an unanswered check-in comes back. Named for Skins.lua's ConsolePort cursor
-- (Panel.frames.checkin).
local CHECKIN = { "fine", "sore", "dark" }

local function flaggedRow(row)
	return ns.state.pain[row.joint] ~= nil
end

function Panel.BuildCheckin()
	local c = Kit.Window("QuestPumpCheckin", "card", L["ui.checkin.title"], { y = 80 })
	c.close:Hide()
	c.ask = Kit.Page(c, L["ui.checkin.question"])
	c.ask.intro:Hide()
	c.answers = Kit.List(c.ask, c.tier.TEXT, c.tier.TEXT)
	for i, key in ipairs(CHECKIN) do
		answerRowOf(c.answers, L["ui.checkin." .. key], function() Panel.Checkin(i) end, { font = "rowName" })
	end
	c.picker = Kit.Page(c, L["ui.joint.title"])
	c.picker.intro:Hide()
	local half, gap = Kit.Grid(2, c.tier.TEXT)
	c.joints, c.gap = {}, gap
	for side = 1, 2 do
		local list = Kit.List(c.picker, half, half)
		for i = side * 4 - 3, side * 4 do
			local key = E.jointKeys[i]
			local row
			row = Kit.Row(list, { name = L["joint." .. key], box = true, onClick = function(r)
				r.picked = not r.picked
				Style.Play(r.picked and "checkOn" or "checkOff")
				Kit.Choose(r, r.picked, true)
			end, tip = Kit.Tip(L["joint." .. key], nil, {
				reason = function() return flaggedRow(row) and L["ui.joint.flagged"] or nil end,
				hint = function() return not flaggedRow(row) and Kit.BoxHint(row.picked) or nil end }) })
			row.joint, row.picked = E.joint[key], false -- picked: ticked here, to flag at Confirm
		end
		c.joints[side] = list
	end
	c.foot = Kit.Footer(c)
	c.confirm = c.foot:Primary(L["ui.joint.confirm"], function()
		local picked = {}
		for _, list in ipairs(c.joints) do
			for _, row in ipairs(list.rows) do
				if row.picked then picked[row.joint] = true end
			end
		end
		Lc.FlagPain(picked)
		c.answered = false
		c:Hide()
	end)
	c.answered = false -- set, never left to a missing field
	F.checkin = c
end

-- The first two answers close the card; the third shows the pain flags before it closes.
function Panel.Checkin(answer)
	local c = F.checkin
	Lc.CheckinAnswer(answer)
	if answer < 3 then
		c:Hide()
		return
	end
	c.answered = true
	for _, list in ipairs(c.joints) do
		for _, row in ipairs(list.rows) do row.picked = false end
	end
	Panel.RenderCheckin()
end

function Panel.RenderCheckin()
	local c = F.checkin
	if not c then return end
	local due = Lc.CheckinDue() or c.answered
	local show = due and not Lc.combat and not InCombatLockdown() and not Lc.Visible() and not Setup.ShouldShow()
	local picker = c.answered == true
	c.ask:SetShown(not picker)
	c.picker:SetShown(picker)
	c.foot:SetShown(picker)
	-- as tall as the page it shows at the player's text size: the question and its answers, or the joints and Confirm
	local bottom
	if picker then
		local y, h = Kit.Top(c.picker), 0
		for side, list in ipairs(c.joints) do
			for _, row in ipairs(list.rows) do
				local flagged = flaggedRow(row)
				Kit.Choose(row, flagged or row.picked, true, flagged and L["ui.joint.flagged"] or nil)
			end
			h = math.max(h, Kit.LayList(list))
			Kit.At(list, c.picker, y, (side - 1) * (list:GetWidth() + c.gap))
		end
		bottom = y + h
	else
		Kit.LayList(c.answers)
		Kit.At(c.answers, c.ask, Kit.Top(c.ask))
		bottom = Kit.Top(c.ask) + c.answers:GetHeight()
	end
	Kit.Fit(c, bottom, picker and c.foot or nil)
	c:SetShown(show and true or false)
end

-- Wiring -----------------------------------------------------------------------------------------------------------

ns.On("State", Panel.Render)
ns.On("Hold", Panel.Render)
-- QuestPump's own menu gone from over a saved set: its draw runs the set's hold again (Lc.GoOn)
ns.On("Menu", function()
	local p = Lc.pump
	if p and p.state == "result" and p.committed and Lc.Visible() and not ns.Menu.Shown() then Panel.Render() end
end)
-- the voice began or ended a line, or a guide began, paused or ended (Speech.lua): the guide's buttons show pause or
-- play with their words, also while the UI is hidden (Alt+Z hides it with no render after), and an open tooltip follows
ns.On("Voice", function()
	for _, name in ipairs({ "ready", "countdown", "running" }) do
		local b = F.views[name] and F.views[name].guide
		if b then
			-- a guide whose takes went meanwhile (the client would not play the voice pack's file: Speech.lua play)
			-- takes its button with it
			if b.move and b:IsShown() and #ns.Speech.GuideTakes(b.move) == 0 then b:Hide() end
			guideArt(b)
			if GameTooltip:IsOwned(b) then guideTip(b) end
		end
	end
end)
-- the setup window redraws on its own changes, a pump's and Your setup's answers (Settings)
for _, message in ipairs({ "Hold", "Setup", "State", "Settings" }) do ns.On(message, Panel.RenderSetup) end
-- the check-in on its own changes, a pump's, and the Body page's (a pain flag set or cleared there while it asks)
for _, message in ipairs({ "CheckinDue", "Trial", "State", "Setup", "Settings" }) do
	ns.On(message, Panel.RenderCheckin)
end

-- A Text Size, UI scale or display size change (Kit.OnRelayout): the pump card, the "Stop to pump" card, the setup
-- window and the check-in, each while it shows, laid out again at their words' new heights and the new screen; the
-- pump forgets the bottom a click left (placeCard), which was the old screen's.
Kit.OnRelayout(function()
	if Lc.pump then Lc.pump.cardBottom = nil end
	Panel.RenderStopCard(Lc.pump)
	if F.dim and F.dim:IsShown() then Panel.Render() end
end)
Kit.OnRelayout(function()
	if F.setup and F.setup:IsShown() then Panel.RenderSetup() end
end)
Kit.OnRelayout(function()
	if F.checkin and F.checkin:IsShown() then Panel.RenderCheckin() end
end)

ns.RegisterEvent("PLAYER_LOGIN", function()
	Panel.Build()
	-- The game menu over an open pump: the dim, the card and the key frame hide while it (and what it opens) shows.
	if GameMenuFrame then
		GameMenuFrame:HookScript("OnShow", function() Lc.MenuShown() end)
		GameMenuFrame:HookScript("OnHide", function() Lc.MenuHidden() end)
	end
end)

ns.RegisterEvent("UPDATE_BINDINGS", function() Panel.ReadBindings() end)

-- A whisper while the dim covers the chat: its sender on the card's title (cardTitle).
local function whispered(_, sender)
	if not (F.dim and F.dim:IsShown() and Lc.pump) then return end
	Panel.whisper = { from = sender }
	cardTitle(Lc.pump)
end
ns.RegisterEvent("CHAT_MSG_WHISPER", whispered)
ns.RegisterEvent("CHAT_MSG_BN_WHISPER", whispered)

-- A quest's title that loaded after the card showed its fallback (cardTitle).
ns.On("QuestTitle", function()
	if F.dim and F.dim:IsShown() and Lc.pump then cardTitle(Lc.pump) end
end)

-- Combat: the key frame and the dim hide at once (Hide is legal in combat; the propagate flag is not), the boxes
-- let go of the keyboard, and the setup screen waits.
ns.RegisterEvent("PLAYER_REGEN_DISABLED", function()
	if F.keys then F.keys:Hide() end
	letGo()
	Panel.Render()
	Panel.RenderSetup()
	Panel.RenderCheckin()
end)

ns.RegisterEvent("PLAYER_REGEN_ENABLED", function()
	Panel.Render()
	Panel.RenderSetup()
	Panel.RenderCheckin()
end)
