-- The in-client voice (docs/plan.md slice 1; recorded since 2026-09-26): a recorded line reads the move and its cue
-- when a move opens, then at most one callout from what the engine knows (ns.PickCallout: a record attempt, a step
-- one day away, a streak day; 2026-10-09), and the cadence when a timed move starts, one line at a time (a new line
-- stops the one playing). The move's How and Tips are read only on a click of the guide's button (Panel.lua), at the
-- end of the How heading's line on the Ready view and on the bar of the countdown and the Running view (2026-10-09;
-- one button for both, reading on through the move's countdown and run, since 2026-10-10; the only way since
-- 2026-10-13, when the setting that read them after the move's line went, David: "remove the autoplay setting for how
-- and tips and make it default off").
-- The recordings sit in Media/Voice/ (tools/voice/, `make voice`: ElevenLabs Eleven v4, one stock voice), and
-- VoiceClips.lua lists each line with the hash of the words it says, so a line whose words change fails `make lint`
-- until it is recorded again. The quest's flavour line stays on the card as text. With no recording, or with speech
-- off, the pump's opening plays a sound instead.

local ADDON, ns = ...
local C, L = ns.C, ns.L
local K = ns.Enum.kind
local T = ns.Timers()

local Speech = {}
ns.Speech = Speech

local PATH = "Interface\\AddOns\\" .. ADDON .. "\\Media\\Voice\\"

-- A move's guide, the parts the Ready view shows under How and Tips (Locales/enUS.lua, guide()), in
-- Media/Voice/guide/: each line of a part a take of its own, "<part>.<key>.<n>" (n its row on the card, from 1), cut
-- from one recording of the part so the voice keeps its flow (tools/voice/voice.mjs; 2026-10-10, so a line said over
-- the guide pauses it at a line and a pause resumes there); a part written for the implement that carries the move's
-- load (Locales/enUS.lua implement(), 2026-10-09: the dumbbell's steps, not the bag's) has takes of its own,
-- "<part>.<key>.<gear>.<n>". The voice reads the parts as one guide, in this order (2026-10-10).
local GUIDE = { "how", "tips" }

-- A part's takes' id before its line's number.
function Speech.GuideId(part, key, gear)
	return "guide/" .. part .. "." .. key .. (gear and "." .. gear or "")
end

-- The takes a move's guide reads, How's lines then Tips', each part the implement's own where the card shows its words
-- (Panel.MoveGear), only those with a take; and the index each part after the first starts at (`starts`, a set: the
-- guide waits callout_gap_s before it).
function Speech.GuideTakes(moveId)
	local takes, starts = {}, {}
	for _, part in ipairs(GUIDE) do
		local base = Speech.GuideId(part, ns.Library[moveId].key, ns.Panel.MoveGear(moveId, part))
		if #takes > 0 and ns.VoiceClips[base .. ".1"] then starts[#takes + 1] = true end
		local n = 1
		while ns.VoiceClips[base .. "." .. n] do
			takes[#takes + 1] = base .. "." .. n
			n = n + 1
		end
	end
	return takes, starts
end

-- Every line the voice says, by id: each move's name and cue and its guide's parts, the name of each move an orange or
-- gold unlock can name (the moves that stand for such a rung: its anchor and the twins on the rung, slice 22; gold
-- since 2026-10-05), the cardio rungs' cadences, the all-out build and the counted
-- set's go, the pace ask's question (`ask.pace`, said as the ask opens; slice 42, after any step's steady move since
-- slice 52), a side move's switch, the timers' counts, a rep set's count (every number to max_reps, the timers' takes
-- among them), the callouts' takes (ns.CalloutFamilies) and the two lines only QuestPump on the computer says
-- (`desk.offer`, the match card's; `desk.ping`, the answer to /qp ping), which play from this list's takes so no system
-- voice speaks (2026-10-16). The recordings are made from this list (tools/voice/lines.lua).
function Speech.Lines()
	local lines = {}
	for id, m in pairs(ns.Library) do
		lines["move." .. m.key] = L["move." .. m.key] .. ". " .. L["move." .. m.key .. ".cue"]
		for _, part in ipairs(GUIDE) do
			local function add(gear, text) -- each row of the card's part, as fillGuide (Panel.lua) splits it
				local n = 0
				for line in (text .. "\n"):gmatch("(.-)\n") do
					n = n + 1
					lines[Speech.GuideId(part, m.key, gear) .. "." .. n] = line
				end
			end
			add(nil, L["move." .. m.key .. "." .. part])
			for _, gear in ipairs(m.holds or {}) do
				local text = rawget(L, "move." .. m.key .. "." .. part .. "." .. gear)
				if text then add(gear, text) end
			end
		end
		-- an orange or gold move's unlock names its colour (Sheet.Legend)
		local q = m.ladder and ns.Quality(m.ladder, m.rung)
		if q and q >= ns.Enum.quality.legendary and (m.ladder == ns.CARDIO or ns.InChain(m.ladder, m.rung, id)) then
			lines["legend." .. m.key] = L[q >= ns.Enum.quality.artifact and "voice.artifact" or "voice.legendary"]
				:format(L["move." .. m.key])
		end
	end
	for rung = 1, 6 do lines["cadence.rung" .. rung] = L["cadence.rung" .. rung] end
	lines["cadence.allout"] = L["cadence.allout"]
	lines["cadence.standup"] = L["cadence.standup"]
	lines["cadence.count"] = L["cadence.count"]
	lines["ask.pace"] = L["ui.result.paceAsk"]
	lines["side.switch"] = L["voice.switch"]
	lines["desk.offer"] = L["voice.desk.offer"]
	lines["desk.ping"] = L["voice.desk.ping"]
	for n = C.mark_s, C.mark_to, C.mark_s do lines["count." .. n] = L["voice.count"]:format(n) end
	for n = 1, C.max_reps do lines["count." .. n] = L["voice.count"]:format(n) end
	for _, family in ipairs(ns.CalloutFamilies()) do
		for _, id in ipairs(ns.CalloutTakes(family)) do lines[id] = L[id] end
	end
	return lines
end

local playing, playingId -- the handle of the line playing, and its id, until its take's length has passed
local guide -- the move's guide: its run (guideRun), its takes (Speech.GuideTakes), the line on (`i`), its handle and
-- end, and whether it stands `halted` (`paused` by the player, or `held` while another line is said over it)
local spoken -- the pump id, move index and move last read aloud on a Ready view
local follow -- what follows a move's line on its Ready view (below)
local cued -- the switch or the run last cued, by its go: a State repeated within it says nothing
local lastTake = {} -- each callout family's take played last: the next one is another
local lineAt = -math.huge -- when the last line other than a count began: a count never cuts one off

-- One recorded take of callout family `family` (ns.CalloutTakes), other than the family's take played last where it
-- has more; nil where none is recorded.
local function freshTake(family)
	local takes = {}
	for _, take in ipairs(ns.CalloutTakes(family)) do
		if ns.VoiceClips[take] then takes[#takes + 1] = take end
	end
	if #takes > 1 then
		for i = #takes, 1, -1 do
			if takes[i] == lastTake[family] then table.remove(takes, i) end
		end
	end
	return takes[1] and takes[math.random(#takes)] or nil
end

-- Why the voice cannot be heard now, as the key of its reason, or nil: the game's sound settings as the client keeps
-- them (another addon can switch the Dialog channel off: the Spoken player left it off after a logout mid-line,
-- 2026-09-30). The voice's own switch is the player's choice, never a reason.
function Speech.Silence()
	local get = C_CVar.GetCVar
	if get("Sound_EnableAllSound") ~= "1" then return "allOff" end
	if (tonumber(get("Sound_MasterVolume")) or 1) <= 0 then return "masterZero" end
	if get("Sound_EnableDialog") ~= "1" then return "dialogOff" end
	if (tonumber(get("Sound_DialogVolume")) or 1) <= 0 then return "dialogZero" end
	return nil
end

local warned -- the session's one line about a voice that cannot be heard
local refused -- a move's line the Dialog channel refused, with the move it was read for

-- Once a session, the setting that keeps the voice silent, named only when it still does voice_recheck_s later: an
-- addon that mutes Dialog while its own line speaks (the Spoken player does, on every line) lifts the mute itself,
-- which is no setting to fix.
local function warnSilent()
	if warned or T.active("recheck") then return end
	T.after("recheck", C.voice_recheck_s, function()
		local why = Speech.Silence()
		if why and not warned and ns.profile and ns.profile.speech then
			warned = true
			ns.Print("speech.silent", L["speech.silent." .. why])
		end
	end)
end

-- The line playing from now (none: nil), announced as "Voice" as a guide's beginning and end are (the guide's buttons
-- show Speech.GuideState()); none once its take's length has passed.
local function nowPlaying(handle, id)
	playing, playingId = handle, id
	T.cancel("ended")
	if id then T.after("ended", ns.VoiceSeconds[id] or 0, function() nowPlaying(nil, nil) end) end
	ns.Emit("Voice")
end

-- Plays a take on the Dialog channel (the player's voice volume): true and its handle, or false, nil and why the client
-- did not play it (a key of `speech.silent`'s reasons, or nil), logged.
local function play(id)
	local willPlay, handle = PlaySoundFile(PATH .. (id:gsub("/", "\\")) .. ".ogg", "Dialog")
	if willPlay then return true, handle end
	local why = Speech.Silence()
	ns.Log("W", "voice %s did not play: %s", id, why or "the client refused the file")
	return false, nil, why
end

-- A move's run in a pump, its pump, its index and its move: a swapped move is another run (`spoken`, a guide's).
local function runId(p, slot)
	return p.id .. ":" .. p.index .. ":" .. slot.move
end

-- The run a move's guide reads for: the move on while the pump shows its Ready view, its countdown or its run, nothing
-- holding it; else nil.
local function guideRun(p)
	local slot = p and p.moves[p.index]
	if not slot or next(ns.Lifecycle.holds) then return nil end
	if (p.state == "open" and p.view == "ready") or p.state == "countdown" or p.state == "running" then
		return runId(p, slot)
	end
	return nil
end

-- The guide's line playing fades out over guide_fade_ms (StopSound's fade; WoW sets no volume on a sound it plays, so
-- a line said over the guide cannot duck it: 2026-10-10).
local function fadeGuide()
	if guide.handle then StopSound(guide.handle, C.guide_fade_ms) end
	guide.handle = nil
end

local function stopGuide()
	T.cancel("guide")
	if not guide then return end
	fadeGuide()
	guide = nil
	ns.Emit("Voice")
end

-- Reads the guide's line i, then the next as it ends (callout_gap_s later where a part starts); the guide ends after
-- its last. Returns play's answers, its handle aside.
local function readTake(i)
	local id = guide.takes[i]
	if not id then
		stopGuide()
		return true
	end
	local willPlay, handle, why = play(id)
	if not willPlay then
		stopGuide()
		return false, why
	end
	guide.i, guide.handle, guide.endsAt = i, handle, GetTime() + (ns.VoiceSeconds[id] or 0)
	T.after("guide", (ns.VoiceSeconds[id] or 0) + (guide.starts[i + 1] and C.callout_gap_s or 0), function()
		readTake(i + 1)
	end)
	return true
end

-- The guide stands where it reads, to go on from the line it faded out (or from the next, stopped in the pause before
-- Tips).
local function haltGuide()
	T.cancel("guide")
	if guide.halted then return end
	guide.halted = true
	if GetTime() >= guide.endsAt then guide.i = guide.i + 1 end
	fadeGuide()
end

-- The guide goes on from its line, from the line's start (WoW plays a sound from its start only).
local function resumeGuide()
	guide.halted, guide.paused, guide.held = nil, nil, nil
	local ok, why = readTake(guide.i)
	ns.Emit("Voice")
	return ok, why
end

-- Plays a line's recording, stopping the line before; false when speech is off or the line has no recording. A line
-- said while a move's guide reads holds the guide: the guide's line fades out and the guide goes on from it
-- callout_gap_s after this one ends, except under a number (`count.<n>`: a rep's, a timer's mark), which plays over the
-- guide (David, 2026-10-10: "the go all out audio etc. Basically all. Should overlay while lowering the audio of the
-- other no? Or resaerch and think through how to best do it?"; a number is a word over a sentence, a sentence over a
-- sentence is heard as neither, and WoW cannot lower one sound under another). A line the client does not play returns
-- false and the reason, a key of `speech.silent`'s reasons, or nil.
function Speech.Say(id)
	if not (ns.profile and ns.profile.speech) or not ns.VoiceClips[id] then return false end
	local hold = guide ~= nil and not guide.paused and not id:find("^count%.")
	if hold then
		haltGuide()
		guide.held = true
	end
	if playing then StopSound(playing) end
	local willPlay, handle, why = play(id)
	nowPlaying(handle, willPlay and id or nil)
	if hold then T.after("guide", (willPlay and ns.VoiceSeconds[id] or 0) + C.callout_gap_s, resumeGuide) end
	if not willPlay then
		if why then warnSilent() end
		refused = why and id:find("^move%.") and { id = id, spoken = spoken } or nil
		return false, why
	end
	if not id:find("^count%.") then lineAt = GetTime() end
	return true
end

-- Begins the guide of the move on (`run`, guideRun(p)), How then Tips, stopping the line playing; whatever was still
-- to follow the move's line on its Ready view stays unsaid. Returns readTake's answers; false for a move without one.
local function beginGuide(p, run)
	T.cancel("callout")
	if playing then
		StopSound(playing)
		nowPlaying(nil, nil)
	end
	local takes, starts = Speech.GuideTakes(p.moves[p.index].move)
	if #takes == 0 then return false end
	guide = { run = run, takes = takes, starts = starts }
	local ok, why = readTake(1)
	if ok then ns.Emit("Voice") end
	return ok, why
end

-- A guide reads on through its move's countdown and run (David, 2026-10-10: "it should not stop during the exercise
-- (only when its done)"): the set's end, another move, a fight, a popup, a cinematic, the game menu, "I feel unwell"
-- and the pump's end stop it.
local function keepGuide(p)
	if guide and guide.run ~= guideRun(p) then stopGuide() end
end

-- The guide's button (Panel.lua; play and pause since 2026-10-10, David: "just add the play/pause button"; the only way
-- a guide begins since 2026-10-13): the guide of the move on read aloud, How then Tips, even with the voice's switch
-- off (the player asked), paused while it reads (held by a line said over it too) and resumed from the line it paused
-- on, once a line said meanwhile has ended. Returns true for a pause or a resume that waits, else readTake's answers.
function Speech.ToggleGuide()
	if guide and guide.paused and playingId and not playingId:find("^count%.") then
		-- resumed while a line is said: held until it ends, as a line said over it holds it (Speech.Say)
		guide.paused, guide.held = nil, true
		T.after("guide", math.max(0, (ns.VoiceSeconds[playingId] or 0) - (GetTime() - lineAt)) + C.callout_gap_s,
			resumeGuide)
		ns.Emit("Voice")
		return true
	end
	if guide and guide.paused then return resumeGuide() end
	if guide then
		haltGuide()
		guide.paused, guide.held = true, nil
		ns.Emit("Voice")
		return true
	end
	local p = ns.Lifecycle.pump
	local run = guideRun(p)
	if not run then return false end
	return beginGuide(p, run)
end

-- The move's guide: "reading" (held by a line said over it too), "paused" or nil (only ever the move on's: keepGuide).
function Speech.GuideState()
	return guide and (guide.paused and "paused" or "reading") or nil
end

-- The sound back on (another addon lifted its Dialog mute; CVAR_UPDATE, Gethe `forever` Console documentation): a
-- move's line it refused is said now, while that move still waits on the Ready view.
ns.RegisterEvent("CVAR_UPDATE", function()
	if not refused or Speech.Silence() then return end
	local line, p = refused, ns.Lifecycle.pump
	refused = nil
	if p and p.state == "open" and p.view == "ready" and spoken == line.spoken and Speech.Say(line.id) then
		follow(p, line.spoken, line.id)
	end
end)

-- At the login and each /reload, with the voice switched on: one line when the game's sound settings silence it.
ns.RegisterEvent("PLAYER_ENTERING_WORLD", function(isInitialLogin, isReloadingUi)
	if not (isInitialLogin or isReloadingUi) then return end
	T.after("silence", 2, function()
		if ns.profile and ns.profile.speech and Speech.Silence() then warnSilent() end
	end)
end)

-- The marks every mark_s on every timer, each on the lifecycle's clock (Lc.Elapsed), so a pause cancels them and the
-- next State or Hold schedules them again. A hold says its elapsed seconds ("count with me": breathing through a
-- hold, never holding the breath; safety.md hazard 3). A timed move marks the seconds left (David, 2026-10-01: "every
-- time there is a timer it should count in 5 seconds intervals"): a flow move says each one, a cardio move ticks until
-- mark_voice_s are left and says them from there, and the last five are the panel's beeps. A counted set only ticks
-- (2026-10-05: the player counts its reps out loud), every all-out window's shouts aside (shout, slice 53), and each
-- side of a side move counts on its own clock. A count due within mark_gap_s of another line ticks instead, and so
-- does one with speech off or not recorded yet; a count plays over a move's guide (David, 2026-10-10: "Can we play
-- the counting even when how and tips and tricks is still playing (just overlay)?").
-- A rep move counts its reps at the move figure's pace (David, 2026-10-09: "the voice counts for you in the exact speed
-- of the animation so you can follow along"): each number as the figure's loop ends a rep (Speech.RepCount; a loop of
-- a dead bug's two sides ends two), on every rep set
-- (a max set, a target set and on past its goal, a catch-up's, /qp try's) up to max_reps, a pace the player counts out
-- loud with (safety.md hazard 3). The Running view shows the same count in the clock's place, and Done starts the
-- count step at it for the player to keep or change (Speech.RepsBy, Panel.lua; David, 2026-10-13). It never ticks (a
-- take the voice cannot say stays silent), no other line holds it back (mark_gap_s), and a count its timer brings too
-- late is skipped for the next rep's.
-- A side move counts each side's run from 1, past its goal, until the player presses Switch sides, then the other
-- side's until Done (David, 2026-10-13: "dont just stop counting and switching at the goal"; from 2026-10-10 one done
-- on the floor or lying down was one set, counted each side to its goal, the voice saying "Switch sides." itself);
-- the countdown between the sides says "Switch sides." (State).
local lastCount -- the mark made last: its run (pump id, index, side and start, as `cued`), its elapsed second and n

-- Every all-out window's shouts (slice 53, 2026-10-23; encouragement at short gaps raises a maximal effort, Andreacci
-- 2002), counted or not: at each of shout_at_s's seconds left its family's take ("Halfway. Faster!" at 15, "Ten more.
-- Everything!" at 10, callout family shout<s>), in that mark's place, with the voice on (the Callouts switch is for the
-- Ready view's lines; the go line and the last five beeps stay); a mark with no take recorded ticks.
local SHOUT = {}
for _, s in ipairs(C.shout_at_s) do SHOUT[s] = "shout" .. s end

local function shout(slot, n)
	local family = slot.allout and SHOUT[n]
	local take = family and freshTake(family)
	if not (take and Speech.Say(take)) then return false end
	lastTake[family] = take
	return true
end

local function mark(slot, n)
	if shout(slot, n) then return end
	local say = (slot.kind ~= K.cardio or n <= C.mark_voice_s) and not slot.counted
	if not (say and GetTime() - lineAt >= C.mark_gap_s and Speech.Say("count." .. n)) then ns.Style.Play("tick") end
end

-- The second of count n on the run's clock: the loop's rep ends (`ends`, the last the loop's end) every beat.
local function countAt(beat, ends, n)
	return math.floor((n - 1) / #ends) * beat + ends[(n - 1) % #ends + 1]
end

-- The rep count of a set of reps, for a rep move with the voice and "Count reps aloud" on (the General page and Your
-- setup, on by default): its beat, one loop of the move's figure, and the seconds in the loop at which its reps end
-- (Figure.Loop; the loop keeps to the same clock, Panel.lua showFigure); nil without the figure's pack. A counted set
-- is a cardio slot. The switches decide it, never whether the voice is heard: the Running view's number and the count
-- step's start (Panel.lua) keep to it through a silenced game (Speech.Silence), a missing take and a count skipped as
-- late.
function Speech.RepCount(slot)
	if slot.kind ~= K.rep or not (ns.profile.speech and ns.profile.countReps) then return nil end
	local beat, ends = ns.Figure.Loop(slot.move)
	if not beat then return nil end
	return { beat = beat, ends = ends }
end

-- How many reps the figure has ended by the clock's `elapsed`, a rep that ends at that very second among them.
local function countsBy(beat, ends, elapsed)
	local loops = math.floor(elapsed / beat)
	local into, n = elapsed - loops * beat, loops * #ends
	for _, e in ipairs(ends) do
		if e <= into then n = n + 1 end
	end
	return n
end

-- The reps a count (Speech.RepCount's plan) has reached by the clock's `elapsed`, up to max_reps, where the voice
-- stops: the Running view's number and the count step's start (Panel.lua). It never falls as the clock runs, so a side
-- move's shorter run gives its weaker side's count.
function Speech.RepsBy(plan, elapsed)
	return math.min(countsBy(plan.beat, plan.ends, elapsed), C.max_reps)
end

-- The next mark after the one made, so a timer that fires a hair early never marks a second twice.
local function scheduleMarks()
	T.cancel("count")
	local Lc = ns.Lifecycle
	local p = Lc.pump
	if not p or p.state ~= "running" or p.pausedAt then return end
	local slot = p.moves[p.index]
	local elapsed, step = Lc.Elapsed(), C.mark_s
	local run = p.id .. ":" .. p.index .. ":" .. (p.side or 0) .. ":" .. tostring(p.runStart)
	local last = lastCount and lastCount.run == run and lastCount or nil
	local said = last and last.at
	local plan = Speech.RepCount(slot)
	local at, n
	if slot.kind == K.hold then
		at = math.max((math.floor(elapsed / step) + 1) * step, (said or 0) + step)
		n = at <= C.mark_to and at or nil
	elseif plan then -- the next rep's end past the clock and the count said last
		n = math.max(countsBy(plan.beat, plan.ends, elapsed) + 1, (last and last.n or 0) + 1)
		at = countAt(plan.beat, plan.ends, n)
		if n > C.max_reps then n = nil end
	elseif p.limit and not slot.practice then
		n = (math.ceil((p.limit - elapsed) / step) - 1) * step
		if said then n = math.min(n, p.limit - said - step) end
		at = p.limit - n
		if n < 2 * step then n = nil end
	end
	if not n then return end
	T.after("count", math.max(0, at - elapsed), function()
		lastCount = { run = run, at = at, n = n }
		if not plan then
			mark(slot, n)
		elseif Lc.Elapsed() - at < (countAt(plan.beat, plan.ends, n + 1) - at) / 2 and not Speech.Silence() then
			-- nearer its rep's end than the next's; with the game's voice silenced it counts on unsaid, logging nothing
			Speech.Say("count." .. n)
		end
		scheduleMarks()
	end)
end

-- An all-out window's line, always at its go: the build's, or a counted set's all out at once with its count
-- (2026-10-05; no build since 2026-10-08).
local function windowLine(slot)
	return slot.counted and "cadence.count" or "cadence.allout"
end

-- A steady cardio move's pace, the take the voice says at its go and the words its Ready and Running views show
-- (Panel.lua; slice 42's fix round, 2026-10-17: the pace ask's "that pace" stated without the voice, the apps'
-- pump.pace.rung1-rung6): its rung's cadence, from rung 7 rung 6's (its steady moves are the forms of rungs 4-6:
-- slice 52, 2026-10-23; rung 3's until then), a Long Haul's rung 1's at every step (slice 55: quick and steady at a
-- talking pace, held 90-180 s); nil for an all-out window, a counted set, the stand-up move (its own easy pace) and any
-- other move.
local HAUL_PACE = "cadence.rung1"

function Speech.PaceLine(slot)
	if slot.kind ~= K.cardio or slot.allout or slot.counted or slot.standup then return nil end
	if slot.haul then return HAUL_PACE end
	return "cadence.rung" .. math.max(1, math.min(slot.rung or 1, 6))
end

-- The callout for the move a Ready view opened: with the General page's Callouts on, one take of the family
-- ns.PickCallout picks, other than the family's take played last where it has more; the take and its family, or nil.
local function calloutTake(p)
	if not ns.profile.callouts then return nil end
	local family = ns.PickCallout(ns.state, ns.profile, p.moves[p.index], p.index, #p.moves, nil,
		ns.StreakAhead(ns.state))
	local take = family and freshTake(family)
	if not take then return nil end
	return take, family
end

-- What follows the line of the move a Ready view opened (`id`, as `spoken`; `line`, playing), with the voice on: its
-- callout callout_gap_s after the line ends, said only while that move still waits on its Ready view.
function follow(p, id, line)
	T.cancel("callout")
	if not (ns.profile and ns.profile.speech and ns.state) or playingId ~= line then return end
	local take, family = calloutTake(p)
	if not take then return end
	T.after("callout", (ns.VoiceSeconds[line] or 0) + C.callout_gap_s, function()
		local q = ns.Lifecycle.pump
		if not (q and q.state == "open" and q.view == "ready" and spoken == id) or next(ns.Lifecycle.holds) then
			return
		end
		lastTake[family] = take
		Speech.Say(take)
	end)
end

-- The line a move's Ready view says after the move's line, which stops once the move leaves it: its callout.
local function readyLine(id)
	return id ~= nil and id:find("^callout%.") ~= nil
end

-- A move leaves its Ready view: its callout still waiting stays unsaid, one still speaking stops (its guide reads on:
-- keepGuide).
local function leaveReady()
	T.cancel("callout")
	if playing and readyLine(playingId) then
		StopSound(playing)
		nowPlaying(nil, nil)
	end
end

ns.On("Hold", function()
	-- a popup, a cinematic or the game menu hides the Ready view
	local p = ns.Lifecycle.pump
	if next(ns.Lifecycle.holds) then leaveReady() end
	keepGuide(p)
	scheduleMarks()
end)

ns.On("State", function(p)
	if not p then return end
	keepGuide(p)
	local slot = p.moves[p.index]
	-- the countdown, a fight, the pump's end (a Skip of its last move leaves no slot)
	if not (slot and p.state == "open" and p.view == "ready") then leaveReady() end
	if not slot then return end
	if p.state == "open" and p.view == "ready" then
		local id = runId(p, slot) -- a swapped move is read again
		local line = "move." .. ns.Library[slot.move].key
		if spoken ~= id then
			spoken = id
			leaveReady() -- the move before's callout (a swap, a skip), with the voice's switch off too
			if not Speech.Say(line) and p.index == 1 then ns.Style.Play("attention") end
			follow(p, id, line)
		elseif playing and playingId ~= line and not readyLine(playingId) then
			-- the move back on its Ready view (Back, Escape in the countdown, a fight): it is not read again, and a
			-- line of its run stops with it (its callout, said on this Ready view, and its guide speak on)
			StopSound(playing)
			nowPlaying(nil, nil)
		end
	elseif p.state == "countdown" and p.side == 2 then
		local id = p.id .. ":" .. p.index .. ":switch:" .. tostring(p.goAt)
		if cued == id then return end
		cued = id
		Speech.Say("side.switch")
	elseif p.state == "running" and slot.kind == K.cardio then
		local id = p.id .. ":" .. p.index .. ":run:" .. tostring(p.runStart)
		if cued == id then return end
		cued = id
		-- the rung's spoken cadence (Speech.PaceLine); an all-out window opens with
		-- its window's line; the stand-up move, a break from sitting, keeps its own easy pace (the rungs' cadences are
		-- hard since 2026-10-04)
		local line = slot.allout and windowLine(slot) or slot.standup and "cadence.standup" or Speech.PaceLine(slot)
		Speech.Say(line)
	elseif p.state == "result" and p.pacing and not p.committed then
		-- "Did you keep that pace to the end?" said as its ask opens (slice 42, 2026-10-17: asked aloud while the player
		-- catches their breath, maybe away from the screen), once for the move
		local id = p.id .. ":" .. p.index .. ":pace"
		if cued == id then return end
		cued = id
		Speech.Say("ask.pace")
	end
	scheduleMarks()
end)
