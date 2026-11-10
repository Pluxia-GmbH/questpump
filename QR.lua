-- /qp qr (docs/plan.md slice 11): the sheet as a QR code a phone camera reads, painted from textures. The payload is
-- the sheet strip event's (Bridge.lua's ns.SheetPayload: the sheet summary with the character's body and figure colour)
-- less the keys a snapshot's reader cannot use (SNAPSHOT_LEAVES), as CBOR, raw DEFLATE and Base64 behind
-- ns.SNAPSHOT_PREFIX, at most C.qr_max_bytes; the phone apps read it with the same decoder as the export string
-- (contract/fixtures/export.json). The encoder is ISO/IEC 18004 in byte mode, error correction L, the version the
-- payload needs and the mask of the least penalty, after Project Nayuki's reference implementation (MIT). The in-game
-- text calls it "a snapshot of your sheet for a phone camera" and names no app and no URL (Blizzard add-on policy
-- rule 4).

local ADDON, ns = ...
local C, L, Kit = ns.C, ns.L, ns.Kit

local floor = math.floor

local QR = {}
ns.QR = QR

-- Level L's error correction codewords per block and blocks, by version (1-40), from the standard's table 9.
local ECC_PER_BLOCK = { 7, 10, 15, 20, 26, 18, 20, 24, 30, 18, 20, 24, 26, 30, 22, 24, 28, 30, 28, 28, 28, 28, 30, 30,
	26, 28, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30 }
local BLOCKS = { 1, 1, 1, 1, 1, 2, 2, 2, 2, 4, 4, 4, 4, 4, 6, 6, 6, 6, 7, 8, 8, 9, 9, 10, 12, 12, 12, 13, 14, 15, 16,
	17, 18, 19, 19, 20, 21, 22, 24, 25 }
local FORMAT_LEVEL = 1 -- level L's two bits in the format information

-- Byte arithmetic without the bit library (tools/gen-fixtures.lua runs the snapshot on PUC Lua 5.1) ----------------

local function xor(a, b)
	local r, p = 0, 1
	while a > 0 or b > 0 do
		if a % 2 ~= b % 2 then r = r + p end
		a, b, p = floor(a / 2), floor(b / 2), p * 2
	end
	return r
end

local function bitAt(x, i) return floor(x / 2 ^ i) % 2 == 1 end

-- GF(256) with the polynomial 0x11D: exp and log tables.
local EXP, LOG = {}, {}
do
	local x = 1
	for i = 0, 254 do
		EXP[i], LOG[x] = x, i
		x = x * 2
		if x >= 256 then x = xor(x, 0x11D) end
	end
	for i = 255, 511 do EXP[i] = EXP[i - 255] end
end

local function mul(a, b)
	if a == 0 or b == 0 then return 0 end
	return EXP[LOG[a] + LOG[b]]
end

-- The generator's coefficients for `degree` error codewords, highest first (without the leading 1).
local function divisor(degree)
	local d = {}
	for i = 1, degree - 1 do d[i] = 0 end
	d[degree] = 1
	local root = 1
	for _ = 1, degree do
		for j = 1, degree do
			d[j] = mul(d[j], root)
			if j < degree then d[j] = xor(d[j], d[j + 1]) end
		end
		root = mul(root, 2)
	end
	return d
end

local function remainder(data, div)
	local r = {}
	for i = 1, #div do r[i] = 0 end
	for _, b in ipairs(data) do
		local factor = xor(b, r[1])
		table.remove(r, 1)
		r[#r + 1] = 0
		for i = 1, #div do r[i] = xor(r[i], mul(div[i], factor)) end
	end
	return r
end

-- The version's layout ----------------------------------------------------------------------------------------------

local function rawModules(ver)
	local n = (16 * ver + 128) * ver + 64
	if ver >= 2 then
		local align = floor(ver / 7) + 2
		n = n - ((25 * align - 10) * align - 55)
		if ver >= 7 then n = n - 36 end
	end
	return n
end

local function dataCodewords(ver)
	return floor(rawModules(ver) / 8) - ECC_PER_BLOCK[ver] * BLOCKS[ver]
end

local function alignment(ver)
	if ver == 1 then return {} end
	local n = floor(ver / 7) + 2
	local step = ver == 32 and 26 or math.ceil((ver * 4 + 4) / (n * 2 - 2)) * 2
	local out = { 6 }
	local size = ver * 4 + 17
	local list = {}
	local pos = size - 7
	for _ = 1, n - 1 do
		table.insert(list, 1, pos)
		pos = pos - step
	end
	for _, p in ipairs(list) do out[#out + 1] = p end
	return out
end

-- The data codewords: byte mode, the count, the bytes, the terminator and the padding.
local function codewords(bytes, ver)
	local bits = {}
	local function put(v, n)
		for i = n - 1, 0, -1 do bits[#bits + 1] = bitAt(v, i) and 1 or 0 end
	end
	put(4, 4) -- byte mode
	put(#bytes, ver <= 9 and 8 or 16)
	for i = 1, #bytes do put(bytes:byte(i), 8) end
	local capacity = dataCodewords(ver) * 8
	put(0, math.min(4, capacity - #bits))
	put(0, (8 - #bits % 8) % 8)
	local out = {}
	for i = 1, #bits, 8 do
		local v = 0
		for k = 0, 7 do v = v * 2 + bits[i + k] end
		out[#out + 1] = v
	end
	local pad = { 0xEC, 0x11 }
	local k = 1
	while #out * 8 < capacity do
		out[#out + 1] = pad[k]
		k = 3 - k
	end
	return out
end

-- The data split into blocks, each with its error codewords, interleaved.
local function interleave(data, ver)
	local nblocks, ecc = BLOCKS[ver], ECC_PER_BLOCK[ver]
	local raw = floor(rawModules(ver) / 8)
	local short = nblocks - raw % nblocks
	local shortLen = floor(raw / nblocks)
	local div = divisor(ecc)
	local blocks, at = {}, 1
	for i = 1, nblocks do
		local n = shortLen - ecc + (i > short and 1 or 0)
		local block = {}
		for k = 1, n do block[k] = data[at + k - 1] end
		at = at + n
		local e = remainder(block, div)
		local full = {}
		for k = 1, n do full[#full + 1] = block[k] end
		if i <= short then full[#full + 1] = false end -- a short block's gap, skipped below
		for k = 1, ecc do full[#full + 1] = e[k] end
		blocks[i] = full
	end
	local out = {}
	for k = 1, #blocks[1] do
		for i = 1, nblocks do
			local v = blocks[i][k]
			if v then out[#out + 1] = v end
		end
	end
	return out
end

-- The matrix ------------------------------------------------------------------------------------------------------

local function newMatrix(size)
	local m, fn = {}, {}
	for y = 0, size - 1 do
		m[y], fn[y] = {}, {}
		for x = 0, size - 1 do m[y][x], fn[y][x] = false, false end
	end
	return m, fn
end

local function set(q, x, y, dark)
	q.m[y][x] = dark
	q.fn[y][x] = true
end

local function finder(q, cx, cy)
	for dy = -4, 4 do
		for dx = -4, 4 do
			local x, y = cx + dx, cy + dy
			if x >= 0 and x < q.size and y >= 0 and y < q.size then
				local d = math.max(math.abs(dx), math.abs(dy))
				set(q, x, y, d ~= 2 and d ~= 4)
			end
		end
	end
end

local function formatBits(q, mask)
	local data = FORMAT_LEVEL * 8 + mask
	local rem = data
	for _ = 1, 10 do
		rem = rem * 2
		if rem >= 1024 then rem = xor(rem, 0x537) end
	end
	local bits = xor(data * 1024 + rem, 0x5412)
	local size = q.size
	for i = 0, 5 do set(q, 8, i, bitAt(bits, i)) end
	set(q, 8, 7, bitAt(bits, 6))
	set(q, 8, 8, bitAt(bits, 7))
	set(q, 7, 8, bitAt(bits, 8))
	for i = 9, 14 do set(q, 14 - i, 8, bitAt(bits, i)) end
	for i = 0, 7 do set(q, size - 1 - i, 8, bitAt(bits, i)) end
	for i = 8, 14 do set(q, 8, size - 15 + i, bitAt(bits, i)) end
	set(q, 8, size - 8, true) -- the dark module
end

local function versionBits(q)
	if q.ver < 7 then return end
	local rem = q.ver
	for _ = 1, 12 do
		rem = rem * 2
		if rem >= 4096 then rem = xor(rem, 0x1F25) end
	end
	local bits = q.ver * 4096 + rem
	for i = 0, 17 do
		local a, b = q.size - 11 + i % 3, floor(i / 3)
		set(q, a, b, bitAt(bits, i))
		set(q, b, a, bitAt(bits, i))
	end
end

local function functionPatterns(q)
	local size = q.size
	for i = 0, size - 1 do
		set(q, 6, i, i % 2 == 0)
		set(q, i, 6, i % 2 == 0)
	end
	finder(q, 3, 3)
	finder(q, size - 4, 3)
	finder(q, 3, size - 4)
	local pos = alignment(q.ver)
	local n = #pos
	for i = 1, n do
		for j = 1, n do
			if not ((i == 1 and j == 1) or (i == 1 and j == n) or (i == n and j == 1)) then
				for dy = -2, 2 do
					for dx = -2, 2 do
						set(q, pos[i] + dx, pos[j] + dy, math.max(math.abs(dx), math.abs(dy)) ~= 1)
					end
				end
			end
		end
	end
	formatBits(q, 0) -- reserves the format areas; drawn again with the chosen mask
	versionBits(q)
end

local function placeData(q, data)
	local size, i = q.size, 0
	local total = #data * 8
	local right = size - 1
	while right >= 1 do
		if right == 6 then right = 5 end
		for vert = 0, size - 1 do
			for j = 0, 1 do
				local x = right - j
				local upward = (floor((right + 1) / 2)) % 2 == 0
				local y = upward and size - 1 - vert or vert
				if not q.fn[y][x] and i < total then
					q.m[y][x] = bitAt(data[floor(i / 8) + 1], 7 - i % 8)
					i = i + 1
				end
			end
		end
		right = right - 2
	end
end

local MASKS = {
	[0] = function(x, y) return (x + y) % 2 == 0 end,
	function(_, y) return y % 2 == 0 end,
	function(x) return x % 3 == 0 end,
	function(x, y) return (x + y) % 3 == 0 end,
	function(x, y) return (floor(x / 3) + floor(y / 2)) % 2 == 0 end,
	function(x, y) return x * y % 2 + x * y % 3 == 0 end,
	function(x, y) return (x * y % 2 + x * y % 3) % 2 == 0 end,
	function(x, y) return ((x + y) % 2 + x * y % 3) % 2 == 0 end,
}

local function applyMask(q, mask)
	local f = MASKS[mask]
	for y = 0, q.size - 1 do
		for x = 0, q.size - 1 do
			if not q.fn[y][x] and f(x, y) then q.m[y][x] = not q.m[y][x] end
		end
	end
end

-- The penalty score (the standard's rules N1-N4), as the reference computes it.
local N1, N2, N3, N4 = 3, 3, 40, 10

local function penalty(q)
	local size, m, score = q.size, q.m, 0
	-- the last seven run lengths, newest first: a 1:1:3:1:1 dark-light pattern with 4 light modules on a side
	local function finders(h)
		local n = h[2]
		local core = n > 0 and h[3] == n and h[4] == n * 3 and h[5] == n and h[6] == n
		return (core and h[1] >= n * 4 and h[7] >= n and 1 or 0) + (core and h[7] >= n * 4 and h[1] >= n and 1 or 0)
	end
	local function push(h, len)
		if h[1] == 0 then len = len + size end -- the light border before the first run
		table.insert(h, 1, len)
		h[8] = nil
	end
	for pass = 1, 2 do
		for a = 0, size - 1 do
			local runDark, run = false, 0
			local h = { 0, 0, 0, 0, 0, 0, 0 }
			for b = 0, size - 1 do
				local dark
				if pass == 1 then dark = m[a][b] else dark = m[b][a] end
				if dark == runDark then
					run = run + 1
					if run == 5 then
						score = score + N1
					elseif run > 5 then
						score = score + 1
					end
				else
					push(h, run)
					if not runDark then score = score + finders(h) * N3 end
					runDark, run = dark, 1
				end
			end
			if runDark then -- the line ends: its last run, then the light border after it
				push(h, run)
				run = 0
			end
			push(h, run + size)
			score = score + finders(h) * N3
		end
	end
	for y = 0, size - 2 do
		for x = 0, size - 2 do
			local c = m[y][x]
			if c == m[y][x + 1] and c == m[y + 1][x] and c == m[y + 1][x + 1] then score = score + N2 end
		end
	end
	local dark = 0
	for y = 0, size - 1 do
		for x = 0, size - 1 do
			if m[y][x] then dark = dark + 1 end
		end
	end
	local total = size * size
	return score + (math.ceil(math.abs(dark * 20 - total * 10) / total) - 1) * N4
end

-- The QR matrix of a byte string at level L: `m[y][x]` true for a dark module, 0-based, `size` modules a side, the
-- `ver` and `mask` chosen; nil when the text is longer than version 40 holds.
function QR.Encode(text)
	local ver
	for v = 1, 40 do
		local header = 4 + (v <= 9 and 8 or 16)
		if header + #text * 8 <= dataCodewords(v) * 8 then
			ver = v
			break
		end
	end
	if not ver then return nil end
	local data = interleave(codewords(text, ver), ver)
	local size = ver * 4 + 17
	local best, bestScore, bestMask
	for mask = 0, 7 do
		local m, fn = newMatrix(size)
		local q = { m = m, fn = fn, size = size, ver = ver }
		functionPatterns(q)
		placeData(q, data)
		applyMask(q, mask)
		formatBits(q, mask)
		local s = penalty(q)
		if not bestScore or s < bestScore then best, bestScore, bestMask = q, s, mask end
	end
	return { m = best.m, size = size, ver = ver, mask = bestMask }
end

-- The snapshot ------------------------------------------------------------------------------------------------------

-- What the snapshot leaves out of the strip's sheet payload (slice 27, 2026-10-06): a phone that reads a snapshot has
-- no helper to pay a catch-up (held), shows no pump result from it (the last pump's ls, lu, ld and lv, and since slice
-- 29 which game pump that is with the titles it earned, lt, and since slice 34 what the Log's pumps paid, lx), keeps it
-- in no history and lists no characters beside it (the save's stamp, the week's start and the character's index: saved,
-- ws, char) and has no room for every step's date and every unlock's progress (ear, nx) nor, since 2026-10-09, for
-- the weekly vault, whose week a picture outlives (va), nor, since 2026-10-15, for the Saga's XP of each day, the day
-- it covers from and its two best days (dx, dxFrom, bd): the text stays under qr_max_bytes; nor, since slice 55, the
-- embers of at's game day (em), a day a picture outlives too; nor, since slice 44, the
-- save's ring horizon (rh), which only a phone that corrects the game's sets from its rows reads, and a snapshot
-- brings no rows. It keeps the rested pool
-- and the last move's time (rest, rt), from which the phone reads the rested XP and the streak as of now, as from the
-- strip's sheet, and the focus (fc, slice 30), which the phone shows and plans its own pumps on while it holds no focus
-- row. The strip's sheet event carries them all, and the export every one of them the summary holds (all but char and
-- lt).
local SNAPSHOT_LEAVES = { "held", "aside", "ls", "lu", "ld", "lv", "lt", "lx", "ear", "nx", "saved", "ws", "char", "va",
	"em", "dx", "dxFrom", "bd", "rh" }

-- The sheet as the QR's text: the strip's sheet payload but SNAPSHOT_LEAVES, CBOR, raw DEFLATE, Base64, behind the
-- prefix. nil and why: "none" without a save, "large" when the text, prefix included, runs past qr_max_bytes
-- characters.
function QR.Snapshot()
	local sheet = ns.SheetPayload and ns.SheetPayload() -- a table of its own, made for this call
	if not sheet then return nil, "none" end
	for _, key in ipairs(SNAPSHOT_LEAVES) do sheet[key] = nil end
	local packed = C_EncodingUtil.CompressString(C_EncodingUtil.SerializeCBOR(sheet), Enum.CompressionMethod.Deflate)
	local text = ns.SNAPSHOT_PREFIX .. C_EncodingUtil.EncodeBase64(packed)
	if #text > C.qr_max_bytes then return nil, "large" end
	return text
end

-- The frame: the kit's medium window (its flat panel, the bar's name, the snapshot's name alone in its head) with
-- the code on white, its quiet zone round it, in a picture's screen under the head: one texture per dark run of a
-- row; no bottom bar. Named for UISpecialFrames: Escape closes it, as it does the sheet and the focus window; kept out
-- of a UI replacement's frame skin (Kit.Window's `skin`), whose strip would take the code's own textures; never scaled
-- (Kit.Window's `fit` "fixed", as before the kit), so each module stays 3 whole units for a phone's camera, the window
-- kept on the screen (Kit.Window clamps it).
local MODULE, QUIET, MARGIN = 3, 4, 8

local function frame()
	if QR.frame then return QR.frame end
	local f = Kit.Window("QuestPumpQR", "M", L["qr.bar"], { escape = "special", skin = false, fit = "fixed" })
	Kit.Head(f.body, L["qr.title"], nil, "M")
	f.body.intro:Hide()
	f.screen = Kit.Screen(f.body, 1, 1)
	f.paper = f.screen:CreateTexture(nil, "ARTWORK", nil, 0)
	f.paper:SetColorTexture(1, 1, 1, 1)
	f.paper:SetPoint("CENTER")
	f.runs = {}
	QR.frame = f
	return f
end

-- Paints a matrix: the window as wide as the code needs (the medium tier's width at least), as tall as the head and
-- the code (Kit.Fit, at scale 1), dark runs reused from earlier paints; kept (f.code) for a lay-out again.
function QR.Paint(code)
	local f = frame()
	f.code = code
	local M = Kit.TIER.M
	local side = (code.size + QUIET * 2) * MODULE
	f:SetWidth(math.max(M.W, side + 2 * (MARGIN + M.IN)))
	f.paper:SetSize(side, side)
	f.screen:SetSize(side + 2 * MARGIN, side + 2 * MARGIN)
	local top = Kit.Top(f.body)
	f.screen:ClearAllPoints()
	f.screen:SetPoint("TOP", f.body, "TOP", 0, -top)
	local n = 0
	for y = 0, code.size - 1 do
		local x = 0
		while x < code.size do
			if code.m[y][x] then
				local start = x
				while x < code.size and code.m[y][x] do x = x + 1 end
				n = n + 1
				local t = f.runs[n]
				if not t then
					t = f.screen:CreateTexture(nil, "ARTWORK", nil, 1)
					t:SetColorTexture(0, 0, 0, 1)
					f.runs[n] = t
				end
				t:ClearAllPoints()
				t:SetPoint("TOPLEFT", f.paper, "TOPLEFT", (QUIET + start) * MODULE, -(QUIET + y) * MODULE)
				t:SetSize((x - start) * MODULE, MODULE)
				t:Show()
			else
				x = x + 1
			end
		end
	end
	for i = n + 1, #f.runs do f.runs[i]:Hide() end
	f:Show()
	Kit.Fit(f, top + side + 2 * MARGIN)
	return n
end

-- a Text Size, UI scale or screen change: the head's height moves the code, painted again while the window shows
Kit.OnRelayout(function()
	local f = QR.frame
	if f and f:IsShown() then QR.Paint(f.code) end
end)

ns.AddCommand("qr", function()
	if not ns.db then return end
	local text, why = QR.Snapshot()
	if not text then return ns.Print(why == "large" and "qr.large" or "qr.none") end
	QR.Paint(QR.Encode(text))
end, "cmd.qr.help")
