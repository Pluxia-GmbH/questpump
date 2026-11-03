-- The pixel strip (docs/plan.md slice 5): two rows of 65 cells of 6 × 6 physical pixels in the screen's top-left
-- corner, painted from login for every player, so the desktop helper reads the game's events whenever it runs and
-- nothing ever reaches the game (addon Lua has no network, file or process access). One frame per event, held at least
-- ns.C.strip_hold_ms so a burst never overwrites a frame the helper has not seen, and a heartbeat every second. A frame
-- is 96 bytes, painted as 2-bit symbols beside calibration cells the helper measures, so the game's brightness,
-- contrast and gamma cannot change what it reads. The byte table and the painting are the contract in wow-addon-dev's
-- pixel-strip.md: questpump-core decodes them from the same table. The helper writes the mirror from these frames:
-- every row, the profile and the character; the sheet summary it only serves to the apps, and a feedback message
-- (slice 24) it posts.

local ADDON, ns = ...
local C, E = ns.C, ns.Enum
local K = E.kind

local B = {}
ns.Bridge = B

-- Fixed by the contract, never tunable: the helper crops exactly COLUMNS × CELL by ROWS × CELL physical pixels.
local CELL, COLUMNS, ROWS = 6, 65, 2
local BYTES = 96 -- a frame, as the table counts it: cells 1-32 of three bytes
local VERSION = 2
local ANCHOR = { 42, 0, 69 }
local LEVELS = { 32, 96, 160, 224 } -- the level a 2-bit symbol paints in a channel; row 0's cells 1-4 show them in grey
local DATA_FROM = { 5, 1 } -- each row's first data cell (0-based): after its anchor, and in row 0 the calibration
local SYMBOL = { 64, 16, 4, 1 } -- a byte's four symbols, the most significant two bits first
local CHUNK = 73 -- blob payload bytes a frame carries after the total length (cells 6-13 and 15-31, 75 bytes)
local MAX_CHUNKS = 255
local RECENT = 32 -- painted frames kept for the fixture generator (tools/gen-fixtures.lua: rows.json's frames)

B.EVENT = { heartbeat = 0, countdown_start = 1, quest_complete = 2, turn_in = 3, pump_offered = 4, pump_done = 5,
	row = 6, profile = 7, leaving = 8, ping = 9, sheet = 10, feedback = 11, keep = 12 }
B.STATE = { idle = 0, countdown = 1, running = 2, input = 3, leaving = 4 }
-- How a pump ended, in its pump_done frame's outcome byte (slice 29), which the helper hands the phones as the
-- command's `ended`: an addon before it paints 0 there, so its pumps read as done.
B.ENDED = { done = 0, banked = 1, paused = 2, unwell = 3, skipped = 4 }
-- The setup word, cell 2's flags bits 4-7 on every frame (2026-10-13), from which the helper names the one step a
-- pairing still asks on every device (contract/companion-api.yaml, Helper.step): `present` in every frame of this
-- addon (an older one paints 0 there: the helper cannot tell), `check_again` while the setup window asks for its Check
-- again (on screen, or held back by a fight, a pump or a cinematic), `helper_seen` when this load found the helper
-- running, `save_unread` on a client that does not read its own save back.
B.SETUP = { present = 16, check_again = 32, helper_seen = 64, save_unread = 128 }
local EV, ST = B.EVENT, B.STATE

-- An id per addon load, so a helper that restarts never logs the last painted event again (it dedupes on the load id,
-- the event counter and the row): the load's start second, so two loads in a row never share one (a /reload takes
-- longer than a second; the id repeats only after 18 hours to the second).
B.loadId = time() % 65535 + 1

local floor, max, min = math.floor, math.max, math.min

-- The encoder (pure) --------------------------------------------------------------------------------------------

-- An unsigned value in n bytes, big-endian, from byte index `at`. A value past the width is clamped: a row's fields
-- fit by construction (the input limits, the stretch row's own cap).
local function unsigned(b, at, v, n)
	v = max(0, min(256 ^ n - 1, floor(v or 0)))
	for k = n - 1, 0, -1 do
		b[at + k] = v % 256
		v = floor(v / 256)
	end
end

local function byteOf(v)
	return max(0, min(255, floor(v or 0)))
end

-- The 96 bytes of one frame (cells 1-32, three bytes each, 1-based) from its fields, named as questpump-core names
-- them: the anchor, the data cells, the checksum (the sum of cells 2-31, modulo 256). `out` is reused when given.
function B.Bytes(f, out)
	local b = out or {}
	for i = 1, BYTES do b[i] = 0 end
	b[1], b[2], b[3] = ANCHOR[1], ANCHOR[2], ANCHOR[3]
	b[4], b[5] = VERSION, byteOf(f.seq)
	b[6] = (f.heartbeat and 1 or 0) + (f.reemit and 2 or 0) + min(f.tier or 0, 3) * 4
		+ floor((f.setup or 0) / 16) % 16 * 16 -- the setup word (B.SETUP), bits 4-7 alone
	b[7], b[8], b[9] = byteOf(f.event), byteOf(f.level), byteOf(f.kind)
	unsigned(b, 14, f.load_id, 2)
	unsigned(b, 40, f.event_no, 2)
	if f.blob then -- a blob's chunk: chunk index · count · serial, then the payload
		b[10], b[11], b[12] = byteOf(f.chunk), byteOf(f.chunks), byteOf(f.serial)
		local k = 0
		for i = 16, 39 do
			k = k + 1
			b[i] = f.blob:byte(k) or 0
		end
		for i = 43, 93 do
			k = k + 1
			b[i] = f.blob:byte(k) or 0
		end
	else
		unsigned(b, 10, f.quest_id, 4)     -- cells 4-5
		unsigned(b, 16, f.ts_client, 3)    -- cell 6
		unsigned(b, 19, f.start_at, 3)     -- cell 7
		unsigned(b, 22, f.yards_foot, 3)   -- cell 8
		unsigned(b, 25, f.yards_mount, 3)  -- cell 9
		unsigned(b, 28, f.yards_taxi, 3)   -- cell 10
		unsigned(b, 31, f.reps, 2)         -- cell 11
		b[33] = byteOf(f.state)
		unsigned(b, 34, f.seconds, 2)      -- cell 12
		b[36] = byteOf(f.outcome)
		unsigned(b, 37, f.yards_swim, 3)   -- cell 13
		b[42] = byteOf(f.slot)             -- cell 14, after the event counter
		b[43], b[44] = byteOf(f.ladder), byteOf(f.rung)
		local flags = max(0, floor(f.row_flags or 0)) % 65536
		b[45], b[55] = flags % 256, floor(flags / 256) -- cell 15 byte 2, cell 19 byte 0
		unsigned(b, 46, f.move_id, 2)      -- cell 16
		b[48] = byteOf(f.char_index)
		unsigned(b, 49, f.row_ts, 4)       -- cells 17-18
		b[56], b[57] = byteOf(f.owed_reps), byteOf(f.owes) -- cell 19; cell 20 stays 0
	end
	local sum = 0
	for i = 4, 93 do sum = sum + b[i] end
	b[94] = sum % 256
	return b
end

-- The strip's cells for a frame's bytes: 390 levels (0-255), three a cell (red, green, blue), row 0's 65 cells then
-- row 1's. Each row's first cell is the anchor, row 0's cells 1-4 the four levels in grey, and bytes 4-96 fill the
-- other cells' channels in order, four symbols a byte, the most significant two bits first. `out` is reused when given.
function B.Cells(bytes, out)
	local c = out or {}
	for row = 0, ROWS - 1 do
		local at = row * COLUMNS * 3
		c[at + 1], c[at + 2], c[at + 3] = ANCHOR[1], ANCHOR[2], ANCHOR[3]
	end
	for k = 1, 4 do
		c[k * 3 + 1], c[k * 3 + 2], c[k * 3 + 3] = LEVELS[k], LEVELS[k], LEVELS[k]
	end
	local slot = DATA_FROM[1] * 3 -- the channels in order, 0-based: row 0's data cells, then row 1's
	for i = 4, BYTES do
		for _, place in ipairs(SYMBOL) do
			c[slot + 1] = LEVELS[floor(bytes[i] / place) % 4 + 1]
			slot = slot + 1
			if slot == COLUMNS * 3 then slot = (COLUMNS + DATA_FROM[2]) * 3 end -- row 0 is full
		end
	end
	return c
end

-- A row as it travels: every field of the row grammar in its own cell, its own ts in cells 17-18, so the helper
-- writes the mirror row as the exact string the addon wrote (ns.MergeMirror unions rows by string).
function B.RowFields(row)
	return { event = EV.row, quest_id = row.questID, tier = row.tier, kind = row.kind, slot = row.slot,
		ladder = row.ladder, rung = row.rung, move_id = row.move, reps = row.reps, seconds = row.seconds,
		outcome = row.outcome, row_flags = row.flags, char_index = row.char, row_ts = row.ts }
end

-- A move of a pump: its quest, tier, kind, position, ladder rung, move, the target (reps or seconds; 0 for a max
-- set) and its planned seconds; the pump's offered time in cells 17-18 is the pump's id.
local function slotFields(f, p, i)
	local slot = p.moves[i]
	local timed = slot.kind == K.cardio or slot.kind == K.flow
	f.quest_id, f.tier, f.kind, f.slot = slot.questID or 0, slot.tier or 0, slot.kind or 0, i - 1
	f.ladder, f.rung, f.move_id = slot.ladder or 0, slot.rung or 0, slot.move or 0
	f.reps = slot.target or (timed and slot.seconds) or 0
	f.seconds = timed and slot.seconds or 0
	f.row_ts = p.offeredTs or 0
	return f
end

-- One move of an offered pump: the move count rides in the outcome byte (the helper knows when the list is
-- complete), and cell 19 carries what the addon's owed row for it would hold at a logout (Events.lua, oweSlot): the
-- reps it would carry (1 an all-out window) and whether it would be owed at all (not a rider, not a pump asked for).
-- The helper writes those rows itself, because no frame renders after PLAYER_LOGOUT. Cell 20 (a closing move's jog
-- until 2026-10-15) stays 0.
function B.OfferedFields(p, i)
	local slot = p.moves[i]
	local f = slotFields({ event = EV.pump_offered }, p, i)
	f.outcome = #p.moves
	f.owed_reps = slot.allout and 1 or 0
	f.owes = (not slot.owed and p.kind ~= "demand") and 1 or 0
	return f
end

-- A blob event's payload (the profile, the sheet, a feedback message) in frames: every chunk starts with the total
-- length (uint16), then 73 payload bytes, the last zero-padded; nil when the payload needs more than 255 frames.
function B.Chunks(event, payload, serial)
	local n = #payload
	local count = max(1, math.ceil(n / CHUNK))
	if count > MAX_CHUNKS then return nil end
	local len = string.char(floor(n / 256), n % 256)
	local frames = {}
	for i = 0, count - 1 do
		frames[#frames + 1] = { event = event, blob = len .. payload:sub(i * CHUNK + 1, (i + 1) * CHUNK), chunk = i,
			chunks = count, serial = serial }
	end
	return frames
end

-- The strip -------------------------------------------------------------------------------------------------------

-- One unit is one physical pixel: the strip ignores the UI scale and sets its own (768 units span the screen height).
local strip = CreateFrame("Frame", nil, UIParent)
strip:SetIgnoreParentScale(true)
strip:SetIgnoreParentAlpha(true)
strip:SetFrameStrata("TOOLTIP")
strip:SetFrameLevel(10000)
strip:SetSize(CELL * COLUMNS, CELL * ROWS)
strip:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 0, 0) -- fixed: the helper's crop never moves, so no Edit Mode

local cells = {} -- row 0's cells, then row 1's
for i = 1, COLUMNS * ROWS do
	local t = strip:CreateTexture(nil, "OVERLAY")
	t:SetSize(CELL, CELL)
	t:SetPoint("TOPLEFT", strip, "TOPLEFT", (i - 1) % COLUMNS * CELL, -floor((i - 1) / COLUMNS) * CELL)
	t:SetSnapToPixelGrid(true)
	t:SetTexelSnappingBias(0)
	cells[i] = t
end

local function rescale()
	local _, height = GetPhysicalScreenSize()
	if height and height > 0 then
		strip:SetScale(768 / height)
		B.scale = 768 / height
	end
end
rescale()

local painted, buffer, levels = {}, {}, {}

-- Only the cells whose colour changed are painted again.
local function paintBytes(bytes)
	B.Cells(bytes, levels)
	for i = 1, COLUMNS * ROWS do
		local k = (i - 1) * 3
		local r, g, b = levels[k + 1], levels[k + 2], levels[k + 3]
		if painted[k + 1] ~= r or painted[k + 2] ~= g or painted[k + 3] ~= b then
			cells[i]:SetColorTexture(r / 255, g / 255, b / 255, 1)
			painted[k + 1], painted[k + 2], painted[k + 3] = r, g, b
		end
	end
end

-- The cells on the strip now, as B.Cells lists them (read only).
function B.Painted()
	return painted
end

-- The queue ---------------------------------------------------------------------------------------------------------

local T = ns.Timers()
local queue = { urgent = {}, events = {}, blobs = {} } -- drained in this order
local ORDER = { "urgent", "events", "blobs" }
local shown, shownAt       -- the frame on the strip and when it was painted
local seq, lastEvent = 0, 0 -- the frame sequence (uint8) and the last event counter (uint16), per load
local leaving, paused, hiddenSince = false, false, nil
B.recent = {}

-- What is left of the shown frame's hold, in seconds; under a millisecond counts as none (a timer that short would
-- only fire again on the same frame).
local function holdLeft()
	if not shownAt then return 0 end
	local left = shownAt + C.strip_hold_ms / 1000 - GetTime()
	return left > 0.001 and left or 0
end

local function queued()
	return #queue.urgent + #queue.events + #queue.blobs
end

-- Nothing paints under a cinematic, a movie, a hidden interface or a loading screen (after `leaving`).
local function visible()
	return strip:IsVisible() and not paused and not leaving
end

-- The current pump's move, when one shows (a practice move stores nothing and paints nothing).
local function current()
	local p = ns.Lifecycle and ns.Lifecycle.pump
	if not p or p.kind == "practice" or not p.opened or not p.moves[p.index] then return nil end
	return p
end

local STATE_OF = { countdown = ST.countdown, running = ST.running, result = ST.input }

-- The pump's state as the helper reads it; a move held by combat in its result view is still being typed (the
-- addon records it itself at a logout, `Lc.OweOpen`, so the helper must not owe it).
local function stateCode()
	if leaving then return ST.leaving end
	local p = current()
	if not p then return ST.idle end
	if p.heldFrom == "result" then return ST.input end
	return STATE_OF[p.state] or ST.idle
end

local function startAt(p)
	if not (p and p.goAt and (p.state == "countdown" or p.state == "running")) then return 0 end
	return floor((p.goAt - C.countdown_s) * 1000) % 0x1000000
end

-- The setup word (B.SETUP) as this load tells it now: the setup window asks for its Check again (Setup.CheckAgainDue:
-- on screen, or held back by a fight, a pump or a cinematic), a helper this load found running (Setup.HelperRuns: its
-- copy came in at the load) and a client that does not read its own save back (Setup.SaveUnread). A change reaches
-- the helper with the next frame, the heartbeat's within a second.
local function setupWord()
	local S = B.SETUP
	return S.present + (ns.Setup.CheckAgainDue() and S.check_again or 0)
		+ (ns.Setup.HelperRuns() and S.helper_seen or 0) + (ns.Setup.SaveUnread() and S.save_unread or 0)
end

-- The fields every frame takes when it is painted: the load id, the client clock (game milliseconds modulo 2^24;
-- the helper unwraps it and reads the countdown's start against it), the state, the setup word, the character's level
-- and index, the stretch's yards so far.
local function stamp(f)
	f.load_id = B.loadId
	f.ts_client = floor(GetTime() * 1000) % 0x1000000
	f.level = ns.Readable(UnitLevel("player")) or 0
	f.state = stateCode()
	f.setup = setupWord()
	f.char_index = f.char_index or ns.charIndex or 0
	local s = ns.Stretch.Current()
	f.yards_foot, f.yards_mount, f.yards_swim, f.yards_taxi = s.foot, s.mount, s.swim, s.taxi
	if not f.blob then f.start_at = f.start_at or startAt(current()) end
end

local heartbeatFrame

local function paintFrame(f)
	if f.heartbeat then f = heartbeatFrame() end -- the pump's move as it is at the paint
	if f.event ~= EV.heartbeat and not f.event_no then -- the counter counts events, never frames or repaints
		lastEvent = (lastEvent + 1) % 65536
		f.event_no = lastEvent
	end
	f.event_no = f.event_no or lastEvent
	seq = (seq + 1) % 256
	f.seq = seq
	stamp(f)
	paintBytes(B.Bytes(f, buffer))
	shown, shownAt = f, GetTime()
	table.insert(B.recent, f)
	if #B.recent > RECENT then table.remove(B.recent, 1) end
	local done = f.onPainted -- a feedback message's last chunk (B.SendFeedback): its sender hears once
	if done then
		f.onPainted = nil
		local ok, err = pcall(done)
		if not ok then ns.Report(err) end
	end
end

local drain

local function nextFrame()
	for _, name in ipairs(ORDER) do
		if #queue[name] > 0 then return table.remove(queue[name], 1) end
	end
end

-- Paints the next queued frame once the shown one has had its hold; the heartbeat resumes a strip that was hidden.
function drain()
	T.cancel("drain")
	if not visible() or queued() == 0 then return end
	local wait = holdLeft()
	if wait > 0 then return T.after("drain", wait, drain) end
	paintFrame(nextFrame())
	if queued() > 0 then T.after("drain", C.strip_hold_ms / 1000, drain) end
end

-- Queues a frame: "urgent" (the countdown, a ping) before the events (rows, moves, turn-ins), the blobs' chunks last
-- (a feedback message's ahead of the profile's and the sheet's, B.SendFeedback).
function B.Send(f, lane)
	table.insert(queue[lane or "events"], f)
	drain()
end
local enqueue = B.Send

-- A frame shown again (after a cinematic, a movie or a hidden UI): the same event counter and fields, so the helper
-- acts on it once.
local function reemit()
	if not shown or shown.event == EV.heartbeat or shown.event == EV.leaving then return end
	local f = {}
	for k, v in pairs(shown) do f[k] = v end
	f.reemit = true
	shownAt = nil
	paintFrame(f)
end

function heartbeatFrame()
	local f = { event = EV.heartbeat, heartbeat = true }
	local p = current()
	if p then slotFields(f, p, p.index) end
	return f
end

local function beat()
	T.cancel("beat")
	if not visible() or queued() > 0 then return drain() end
	local wait = holdLeft()
	if wait > 0 then return T.after("beat", wait, beat) end
	paintFrame(heartbeatFrame())
end

-- The pump's state changed: a heartbeat goes in the urgent lane, ahead of queued rows and chunks, so the helper learns
-- a move's count is being typed before a logout can cut the queue short.
local lastState
local function soon()
	local code = stateCode()
	if code == lastState then return end
	lastState = code
	for _, f in ipairs(queue.urgent) do
		if f.heartbeat then return end -- one waits already; it is filled in at the paint
	end
	enqueue({ event = EV.heartbeat, heartbeat = true }, "urgent")
end

-- The profile and the sheet ---------------------------------------------------------------------------------------

local dirty = { [EV.profile] = false, [EV.sheet] = false }
local serial = { [EV.profile] = 0, [EV.sheet] = 0, [EV.feedback] = 0 }
local blobAt = {}

local function saving()
	return ns.db ~= nil and not ns.readOnly -- a newer addon's save is read, never carried into the mirror
end

-- The session's last game pump that ended done (slice 29): { p = its offered time, the pump's id on the strip, t = the
-- titles it earned, their codes (ns.Enum.title) in the closed list's order, absent when none }; nil before one, and a
-- /reload forgets it.
local lastDone

-- lt, which game pump the summary's last pump is (slice 29): that pump while the summary's last pump (ns.LastPump,
-- whose `last`, ls, lu and ld the summary tells) is it, so a phone takes a game pump's rewards and titles only from a
-- summary that names the pump, never from a newer pump's (its offered row makes it the last pump at once, done or
-- not) nor from an older one's; nil else.
local function lastTitles(state)
	local last = lastDone and ns.LastPump(state)
	if last and last == state.sheet.pump and last.ts == lastDone.p then return lastDone end
	return nil
end

-- The sheet summary (Stats.lua) for the apps, with the character index, its body, since slice 27 the colour the game
-- tints the move figure in (Figure.lua) and since slice 29 lt (lastTitles), by which a phone knows the summary's last
-- pump is the game pump it shows and names the titles it earned: the sheet event's payload and /qp qr's (QR.lua,
-- which leaves lt out), so the phone reads both with one decoder; nil before the save is bound. Its `held` list is
-- the engine's alone (ns.OwedHeld) since 2026-10-04: until then an open pain question held its pump's moves back too.
function ns.SheetPayload()
	if not (ns.db and ns.state) then return nil end
	local sheet = ns.SheetSummary(ns.state, ns.now(), ns.db.savedAt, ns.profile, ns.Horizon(ns.db))
	sheet.char, sheet.body, sheet.fig, sheet.lt = ns.charIndex, ns.CharBody(), ns.Figure.ColorCode(),
		lastTitles(ns.state)
	return sheet
end

-- The active profile with its name and the character it belongs to, so the mirror restores profiles, profileKeys and
-- chars; the sheet summary for the apps.
local function payload(event)
	local db = ns.db
	local char = db.chars[ns.charIndex]
	if event == EV.profile then
		return { name = ns.ProfileName(db, char and char.guid), profile = ns.profile, char = char,
			index = ns.charIndex }
	end
	return ns.SheetPayload()
end

-- A blob event's value as its chunks, CBOR then raw Deflate (the export's codec), with the event's next serial; nil
-- past 255 chunks, reported, never dropped in silence (a profile is well under 1 KB, a feedback message under 4 KB).
local function pack(event, value)
	local cbor = C_EncodingUtil.SerializeCBOR(value)
	local packed = C_EncodingUtil.CompressString(cbor, Enum.CompressionMethod.Deflate)
	local frames = B.Chunks(event, packed, (serial[event] + 1) % 256)
	if not frames then
		ns.Report(("QuestPump: a strip payload of %d bytes is more than 255 frames carry"):format(#packed))
		return nil
	end
	serial[event] = (serial[event] + 1) % 256
	return frames
end

local function sendBlob(event)
	dirty[event] = false
	blobAt[event] = GetTime()
	local frames = pack(event, payload(event))
	if not frames then return end
	for i = #queue.blobs, 1, -1 do -- the unsent chunks of the previous copy are stale
		if queue.blobs[i].event == event then table.remove(queue.blobs, i) end
	end
	for _, f in ipairs(frames) do queue.blobs[#queue.blobs + 1] = f end
	drain()
end

-- A message from the feedback window (slice 24; contract/feedback-api.yaml, GameFeedback: `text`, `version`, `at` and,
-- ticked, `numbers`), handed to the helper, which posts it: its own event and serial in the blobs lane, ahead of the
-- profile's and the sheet's chunks waiting there and behind a message still painting (so no message's chunks
-- interleave with another's), painted once and never again (refreshBlobs repaints only the profile and the sheet), so
-- a message the helper did not read whole is lost. `onPainted()` runs once its last chunk is on the strip. Even a newer
-- addon's read-only save sends one: it carries nothing into the mirror. False when it was not queued.
function B.SendFeedback(message, onPainted)
	local frames = pack(EV.feedback, message)
	if not frames then return false end
	frames[#frames].onPainted = onPainted
	local at = 1 -- the waiting feedback chunks always lead the lane
	while queue.blobs[at] and queue.blobs[at].event == EV.feedback do at = at + 1 end
	for i, f in ipairs(frames) do table.insert(queue.blobs, at + i - 1, f) end
	drain()
	return true
end

-- Built at the heartbeat, so a burst of changes (a slider dragged, a pump's rows) sends one copy; painted again every
-- strip_blob_repeat_s for a helper started after login.
local function refreshBlobs()
	if not saving() or not ns.state then return end
	for _, event in ipairs({ EV.profile, EV.sheet }) do
		if dirty[event] or (blobAt[event] and GetTime() - blobAt[event] >= C.strip_blob_repeat_s) then
			sendBlob(event)
		end
	end
end

-- The heartbeat ---------------------------------------------------------------------------------------------------

local function tick()
	refreshBlobs()
	if not visible() then
		hiddenSince = hiddenSince or GetTime()
		return
	end
	if hiddenSince then
		hiddenSince = nil
		reemit()
	end
	beat()
end

-- The lifecycle's events ---------------------------------------------------------------------------------------------

local offered = setmetatable({}, { __mode = "k" }) -- [pump] = the moves as last painted
local counted = setmetatable({}, { __mode = "k" }) -- [pump] = the countdown last painted (its go time)
local kept = setmetatable({}, { __mode = "k" }) -- [pump] = the go of the move whose Keep going was last painted
local titlesAt = setmetatable({}, { __mode = "k" }) -- [pump] = the titles earned when its offer first went out (a set)

-- The moves as the helper needs them; a pump composed again (a new pain flag, a pain stop's re-roll) paints again,
-- a steady move that became an all-out window of the same move and length (cell 19's owed reps) too.
local function signature(p)
	local parts = {}
	for i, s in ipairs(p.moves) do
		parts[i] = table.concat({ s.move, s.allout and 1 or 0, s.target or 0, s.seconds or 0, s.owed and 1 or 0,
			s.questID or 0, s.tier or 0 }, ":")
	end
	return table.concat(parts, ",")
end

-- Lc.Finish's way out (the bus's second argument) as B.ENDED tells it: a Bank it and the moves going owed (a turn-in
-- before a held pump resumes, an instance, a mixed party) leave the pump's moves to a later pump; a pump whose last
-- unfinished move is skipped before any result, a stand-up pump an instance or a mixed party drops before it started
-- (it pays no turn-in, so nothing is owed: Lc.OweIfOwing) and one a reset ends (/qp reset all, a restored backup:
-- Lc.Drop ends it unwritten, its moves owed to no one) are skipped.
local ENDED_BY = { done = B.ENDED.done, banked = B.ENDED.banked, owed = B.ENDED.banked, dropped = B.ENDED.skipped,
	reset = B.ENDED.skipped, paused = B.ENDED.paused, unwell = B.ENDED.unwell, skipped = B.ENDED.skipped }

local function endedCode(p, how)
	-- "Bank the rest" after a move of it pays what was done, as the game's own close does (Sheet.lua)
	if how == "rest" then return next(p.results) and B.ENDED.done or B.ENDED.banked end
	return ENDED_BY[how] or B.ENDED.skipped
end

-- A pump that ended done as lastDone holds it, with the titles it earned since its offer first went out (t left out
-- when none: CBOR cannot tell an empty Lua table from an empty map).
local function doneOf(p)
	local new = ns.NewTitles(titlesAt[p], ns.state)
	for i, id in ipairs(new) do new[i] = E.title[id] end
	return { p = p.offeredTs, t = #new > 0 and new or nil }
end

ns.On("State", function(p, how)
	if p and p.kind ~= "practice" and saving() then
		if p.state == "done" then
			if offered[p] then
				offered[p] = nil
				local first, ended = p.quests[1], endedCode(p, how)
				enqueue({ event = EV.pump_done, quest_id = first and first.questID or 0, row_ts = p.offeredTs,
					outcome = ended })
				if ended == B.ENDED.done then lastDone = doneOf(p) end
				titlesAt[p] = nil
				dirty[EV.sheet] = true
			end
		elseif p.opened and p.offeredTs and #p.moves > 0 then
			local sig = signature(p)
			if offered[p] ~= sig then
				if not offered[p] then titlesAt[p] = ns.TitlesEarned(ns.state) end
				offered[p] = sig
				-- the frames of its composition before that have not painted yet are stale (the helper builds the list
				-- anew from each composition's first move): a row of skips queues one list, never one a click
				for i = #queue.events, 1, -1 do
					local f = queue.events[i]
					if f.event == EV.pump_offered and f.row_ts == p.offeredTs then table.remove(queue.events, i) end
				end
				for i = 1, #p.moves do enqueue(B.OfferedFields(p, i)) end
			end
			if p.state == "countdown" and counted[p] ~= p.goAt then
				counted[p] = p.goAt
				local f = slotFields({ event = EV.countdown_start }, p, p.index)
				f.start_at = startAt(p)
				enqueue(f, "urgent")
			end
			-- Keep going on the running move (Lc.KeepGoing, slice 52): its seconds the full move's, so the phone or the
			-- watch that follows the move counts up to it from the press, as the game does (2026-10-24); urgent, since
			-- the plan ends within keep_going_s
			if p.state == "running" and p.keep and kept[p] ~= p.goAt then
				kept[p] = p.goAt
				local f = slotFields({ event = EV.keep }, p, p.index)
				f.seconds = p.moves[p.index].keep
				enqueue(f, "urgent")
			end
		end
	end
	soon()
end)

-- The owed moves the sheet holds back (ns.OwedHeld) follow a pain flag and every move the queue gains outside a pump
-- (a turn-in owed in an instance, a mixed party or a felt-unwell day, Bank it), so a phone's catch-up never pays one
-- before the sheet repaints; a skipped move (Lc.Skip) leaves the queue with no move done, so the sheet's unfinished
-- count repaints for it too.
local QUEUED = { [E.outcome.owed] = true, [E.outcome.banked] = true }

ns.On("Row", function(row)
	if saving() then enqueue(B.RowFields(row)) end
	if row.kind == K.pain or QUEUED[row.outcome] or ns.HasFlag(row.flags, "skipped") then dirty[EV.sheet] = true end
end)

ns.On("TurnIn", function(q)
	if saving() then enqueue({ event = EV.turn_in, quest_id = q.questID, tier = q.tier }) end
end)

ns.On("DB", function()
	dirty[EV.profile], dirty[EV.sheet] = true, true
end)

-- Every profile change: the Settings pages ("Settings"), and the ones that only stamp the save (/qp instances,
-- the HUD's place, a title: "Touch"). Each repaints the sheet too: a Body switch or an exclusion changes the owed moves
-- held back, and the summary names the chosen title (ttl, slice 27).
ns.On("Settings", function()
	dirty[EV.profile], dirty[EV.sheet] = true, true
end)

ns.On("Touch", function()
	dirty[EV.profile], dirty[EV.sheet] = true, true
end)

-- A quest whose objectives are all done: read at the QUEST_LOG_UPDATE that follows the objective's step (the log is
-- not updated yet when QUEST_WATCH_UPDATE fires), painted once until the quest is turned in or abandoned.
local watched, told = {}, {}

ns.RegisterEvent("QUEST_WATCH_UPDATE", function(questID)
	if questID then watched[questID] = true end
end)

ns.RegisterEvent("QUEST_LOG_UPDATE", function()
	for questID in pairs(watched) do
		watched[questID] = nil
		if not told[questID] and C_QuestLog.IsComplete(questID) and saving() then
			told[questID] = true
			enqueue({ event = EV.quest_complete, quest_id = questID })
		end
	end
end)

ns.RegisterEvent("QUEST_TURNED_IN", function(questID)
	told[questID] = nil
end)

ns.RegisterEvent("QUEST_REMOVED", function(questID)
	told[questID], watched[questID] = nil, nil
end)

-- Leaving the world (a loading screen, a logout, a /reload): painted at once, since the next frame may be the last
-- the client renders; the helper suspends its mirror writes. A frame shown for less than its hold goes back to the
-- front and paints again after a loading screen, with its counter, so the helper keeps one copy.
local function leave()
	if leaving then return end
	leaving = true
	if shown and shown.event ~= EV.heartbeat and shown.event ~= EV.leaving and holdLeft() > 0 then
		table.insert(queue.urgent, 1, shown)
	end
	T.cancel("drain")
	shownAt = nil
	paintFrame({ event = EV.leaving })
end

ns.RegisterEvent("PLAYER_LEAVING_WORLD", leave)
ns.RegisterEvent("PLAYER_LOGOUT", leave)

ns.RegisterEvent("PLAYER_ENTERING_WORLD", function()
	leaving, hiddenSince = false, nil
	rescale()
	soon()
	drain()
end)

-- A movie hides UIParent and an in-game cinematic covers it: nothing paints while the lifecycle holds for one (from
-- its start event to its end event, or to the poll that finds it gone when one cut short sends none), and the last
-- frame shows again when it ends.
local function pause()
	paused = true
	T.cancel("drain")
end

local function resume()
	if not paused then return end
	paused, hiddenSince = false, nil
	reemit()
	drain()
end

-- Every hold and its end ("Hold"): a popup or the game menu can take a counting-down pump back to Ready without a
-- "State" message, so the pump's state goes out (before the strip pauses for a cinematic or a movie, after it resumes).
ns.On("Hold", function()
	local holds = ns.Lifecycle.holds
	if holds.cinematic or holds.movie then
		soon()
		pause()
	else
		resume()
		soon()
	end
end)

-- One heartbeat: a strip a cinematic or a movie paused resumes once neither holds, since Lc.Unhold sends "Hold" only
-- when the last hold ends (one that ends while the game menu or a popup still holds sends none), then the tick.
local function heartbeat()
	local holds = ns.Lifecycle.holds
	if paused and not (holds.cinematic or holds.movie) then resume() end
	tick()
end

ns.RegisterEvent("DISPLAY_SIZE_CHANGED", rescale)
ns.RegisterEvent("UI_SCALE_CHANGED", rescale)

ns.RegisterEvent("PLAYER_LOGIN", function()
	if T.active("heartbeat") then return end
	rescale()
	-- the ticker's own errors are reported, as every handler's are: one never stops the heartbeat
	T.every("heartbeat", C.strip_heartbeat_s, function()
		local ok, err = pcall(heartbeat)
		if not ok then ns.Report(err) end
	end)
	tick()
end)

-- Diagnostics: the strip is on from login for every player, so there is no switch, only these two.
ns.AddCommand("bridge", function()
	local w, h = GetPhysicalScreenSize()
	ns.Print("status.bridge.strip", CELL * COLUMNS, CELL * ROWS, w or 0, h or 0, B.scale or 0)
	ns.Print(visible() and "status.bridge.shown" or "status.bridge.hidden", B.loadId, lastEvent, queued())
end, "cmd.bridge.help")

ns.AddCommand("ping", function()
	enqueue({ event = EV.ping }, "urgent")
	ns.Print("status.bridge.ping")
end, "cmd.ping.help")
