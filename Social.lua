-- The social core (docs/plan.md slice 13): the QuestPump card and its pull protocol over addon messages (no library),
-- who sees the card, the tooltip line, "QuestPump card" on the right-click menu with its side-by-side window, the
-- share lines' link, the party (the Join line and each member's moves this game day), cheers and the guild's week
-- from the cards seen; and since slice 23 pacts and duels with friends and Battle.net game data for a friend a whisper
-- cannot reach. Nothing about another player outlives the session but a pact partner's
-- and a duel's rivals' names while the pact or the duel lasts, nothing received changes XP, records or the unfinished
-- moves, and nothing typed is ranked: a card carries what QuestPump counts, and typed records only when their owner
-- shares them, labelled typed. The sheet's Guild tab, the Character-window tab and the HUD draw from here (Sheet.lua),
-- the Social settings page is Settings.lua's, the card question Panel.lua's.

local ADDON, ns = ...
local C, E, L = ns.C, ns.Enum, ns.L
local Lc, Sheet, Style, Kit = ns.Lifecycle, ns.Sheet, ns.Style, ns.Kit

local Social = { peers = {}, mute = {}, hide = {} }
ns.Social = Social
local T = ns.Timers()
local readable = ns.Readable

-- The protocol's fixed facts: one prefix (at most 16 characters), Blizzard's 255-byte message cap, and the marker a
-- share line ends with, which a receiver running QuestPump turns into a link to the sender's card.
local PREFIX = "QuestPump"
local MAX_BYTES = 255
local HEAD = #"C00000000000:" -- a card's "C<version>:" at its longest, the version in whole seconds
Social.TOKEN = "[QuestPump]"
local GROUP_CHANNEL = { PARTY = true, RAID = true, INSTANCE_CHAT = true }
local R = Enum.SendAddonMessageResult
-- A throttle means the server's own bucket ran dry: sent again once ours has refilled. Not in the group, not in a
-- guild or offline: the message has no one to reach, dropped. Any other code is a bug, reported once.
local THROTTLE = { [R.AddonMessageThrottle] = true, [R.ChannelThrottle] = true }
local DROP = { [R.NotInGroup] = true, [R.NotInGuild] = true, [R.TargetOffline] = true }
-- Everything a card field may hold, clamped on arrival: any addon can forge a message, only the sender's name comes
-- from the server. A week holds 10,080 minutes.
local BOUNDS = { L = { 1, C.level_cap }, P = { 0, 9999 }, T = { 0, 65535 }, B = { 0, #C.border_levels + 1 },
	S = { 0, 9999 }, A = { 0, C.rating_days }, W = { 0, 9999 }, R = { 0, 9999 }, M = { 0, 10080 }, F = { 0, 1 },
	D = { 0, 1 }, N = { 0, 9999 } }
local TITLE_OF = {} -- title code → id
for id, code in pairs(E.title) do TITLE_OF[code] = id end
local CHEERS = {} -- cheer code → name, in the codes' order (ns.Enum.cheer is the one list)
for name, code in pairs(E.cheer) do CHEERS[code] = name end
-- Who sees the card: nobody (private), friends, guild and group, or anyone who asks (the question offers two of them).
Social.SHARE = { "nobody", "circle", "anyone" }
-- The chat that may carry another player's share line: the player-sent lines' channels and the whispers received.
local CHAT_EVENTS = { "CHAT_MSG_PARTY", "CHAT_MSG_PARTY_LEADER", "CHAT_MSG_RAID", "CHAT_MSG_RAID_LEADER",
	"CHAT_MSG_INSTANCE_CHAT", "CHAT_MSG_INSTANCE_CHAT_LEADER", "CHAT_MSG_GUILD", "CHAT_MSG_OFFICER",
	"CHAT_MSG_WHISPER", "CHAT_MSG_SAY", "CHAT_MSG_YELL" }
-- A pact's joint plans (slice 23) by their id (ns.Enum.pact), and their words.
local PLANS = E.pactKeys
local PLAN_TEXT = { L["social.pact.daily"], L["social.pact.evening"] }
-- The most a duel's score can be: every capped point on every one of its days.
local DUEL_TOP = C.duel_days * (C.duel_pumps_day * C.duel_pump_pts + C.duel_active_pts
	+ C.duel_cardio_min_day * C.duel_cardio_pts)

-- Names -----------------------------------------------------------------------------------------------------------

-- A player's key is the name as the client gives it (Forever's region-unique "First Surname", never "Name-Realm"); a
-- retail name on the player's own realm loses its "-Realm", which the chat sender carries and a unit's name does not.
function Social.Key(name)
	local realm = GetNormalizedRealmName and readable(GetNormalizedRealmName())
	if realm and realm ~= "" then
		local suffix = "-" .. realm
		if name:sub(-#suffix) == suffix then return name:sub(1, -#suffix - 1) end
	end
	return name
end

local function fullName(unit)
	if NameUtil and NameUtil.GetUnmodifiedUnitFullName then return readable(NameUtil.GetUnmodifiedUnitFullName(unit)) end
	local name, realm = UnitName(unit)
	name, realm = readable(name), readable(realm)
	if name and realm and realm ~= "" then return name .. "-" .. realm end
	return name
end

-- The key of a player unit, nil for anything else, while the client keeps the unit's identity secret (asked first:
-- Blizzard's name helpers test the name they read) or before it has filled the unit in ("Unknown").
function Social.UnitKey(unit)
	if C_Secrets.ShouldUnitIdentityBeSecret(unit) or not readable(UnitIsPlayer(unit)) then return nil end
	local name = fullName(unit)
	if not name or name == "" or name == UNKNOWNOBJECT then return nil end
	return Social.Key(name)
end

-- The group's other members online, by key (key → unit); the second value is false while one of them cannot be read
-- (a secret name, a member the client has not filled in, a follower that is no player). A member who is offline is
-- left out: no message reaches them and they wait for no one.
function Social.GroupKeys()
	local keys, complete = {}, true
	if not IsInGroup() then return keys, complete end
	local raid = IsInRaid()
	local n = GetNumGroupMembers()
	for i = 1, raid and n or n - 1 do
		local unit = (raid and "raid" or "party") .. i
		if not (raid and readable(UnitIsUnit(unit, "player"))) and readable(UnitIsConnected(unit)) ~= false then
			local key = Social.UnitKey(unit)
			if not key then
				complete = false
			elseif key ~= Social.me then -- the player's own raid unit, when the comparison is secret
				keys[key] = unit
			end
		end
	end
	return keys, complete
end

-- Whether every other member of the group online is known to run QuestPump (a card, a private answer or any message
-- of theirs this session): the lifecycle's mixed party (Events.lua) is a group where this is false. Unknown until then,
-- so a group's first seconds, before the hellos, owe like a mixed party.
function Social.AllMembersRun()
	local keys, complete = Social.GroupKeys()
	if not complete then return false end
	for key in pairs(keys) do
		local p = Social.peers[key]
		if not (p and p.known) then return false end
	end
	return true
end

-- The session's players ------------------------------------------------------------------------------------------

local count = 0

-- A player seen this session: { card, ver, private, known (runs QuestPump), guild, seen, asked, answered, cheered (my
-- cheers to their pumps), cheers (theirs to mine), legend (their Legendary unlock shown), duel (their score in our
-- duel: { id, score, final, top }), today (their moves this game day: { day, moves }) }. At most C.peer_max; the one
-- seen longest ago goes.
local function peer(key)
	local p = Social.peers[key]
	if p then return p end
	if count >= C.peer_max then
		local oldest
		for k, q in pairs(Social.peers) do
			if not oldest or (q.seen or 0) < (Social.peers[oldest].seen or 0) then oldest = k end
		end
		Social.peers[oldest] = nil
		count = count - 1
	end
	p = { cheered = {}, cheers = {} }
	Social.peers[key] = p
	count = count + 1
	return p
end

local function ignored(key)
	return C_FriendList and C_FriendList.IsIgnored and C_FriendList.IsIgnored(key) == true
end

-- The key of a Battle.net friend's character in this game, from its game account: the name alone where names are
-- unique across the region, as Forever's are; elsewhere with its realm, which the sender's name carries without
-- spaces. Nil for a game account in another game or with no character.
local function bnKey(a)
	local name = a and a.wowProjectID == WOW_PROJECT_ID and a.characterName
	if not name or name == "" then return nil end
	-- Forever's names are unique across the region; retail has no RegionalUniqueNamesEnabled (Ketho live 12.1.0)
	if RegionalUniqueNamesEnabled and RegionalUniqueNamesEnabled() then return name end
	local realm = a.realmName and (a.realmName:gsub("[%s%-]", ""))
	return realm and realm ~= "" and Social.Key(name .. "-" .. realm) or nil
end

-- The game account of a Battle.net friend's character by key (the client lists the game accounts of friends online).
local function bnAccount(key)
	if not (BNGetNumFriends and C_BattleNet and C_BattleNet.GetFriendNumGameAccounts) then return nil end
	for i = 1, BNGetNumFriends() or 0 do
		for j = 1, C_BattleNet.GetFriendNumGameAccounts(i) or 0 do
			local a = C_BattleNet.GetFriendGameAccountInfo(i, j)
			if bnKey(a) == key then return a end
		end
	end
	return nil
end

-- A friend by name: the character friends list, then the Battle.net friends' characters in this game. The GUID a card
-- carries is never trusted for this (any addon can forge one).
local function friend(key)
	if C_FriendList and C_FriendList.GetFriendInfo and C_FriendList.GetFriendInfo(key) then return true end
	return bnAccount(key) ~= nil
end

-- Whether a friend is online as the client lists them: a character friend connected, a Battle.net friend's game
-- account online. A pact's and a duel's messages go only to one who is (a whisper to a name offline prints "No player
-- named ...").
local function online(key)
	local info = C_FriendList and C_FriendList.GetFriendInfo and C_FriendList.GetFriendInfo(key)
	if info then return info.connected == true end
	local a = bnAccount(key)
	return a ~= nil and a.isOnline == true
end

-- Battle.net game data (slice 23): a whisper reaches a character of the player's own faction on the player's own
-- realm; a Battle.net friend's character elsewhere (another faction, realm or game: the realm says which) gets the
-- same messages through C_BattleNet.SendGameData, as does one whose messages came that way this session (bnRoute:
-- key → game account, never stored). Its reach across Forever's factions and rulesets is unverified until the
-- two-account check: without it, this function, its call in drain and the BN_CHAT_MSG_ADDON handler go, and whispers
-- alone remain.
local bnRoute = {}

local function gameAccount(key)
	if bnRoute[key] then return bnRoute[key] end
	local a = bnAccount(key)
	if not (a and a.isOnline) then return nil end
	if a.factionName == readable(UnitFactionGroup("player")) and a.realmID == GetRealmID() then return nil end
	return a.gameAccountID
end

-- A guildmate by name, from the guild roster the client holds (asked for at login, `C_GuildInfo.GuildRoster`), or a
-- player whose card came on the guild's channel since the player joined this guild.
local function guildmate(key, p)
	return (p and p.guild) or (IsInGuild() and C_GuildInfo.MemberExistsByName(key) == true)
end

-- Audience --------------------------------------------------------------------------------------------------------

-- Whether the layer listens at all: not while QuestPump sleeps (the General page's switch, as if disabled), nor in an
-- instance while the player keeps it off there (on by default).
function Social.Listening()
	local p = ns.profile
	return ns.db ~= nil and p ~= nil and p.enabled ~= false and not (p.socialOffInInstances and IsInInstance())
end

-- Whether the player shares the card itself (a private player sends nothing but "-C").
function Social.Sharing()
	return Social.Listening() and ns.profile.share ~= "nobody"
end

-- Whether a player gets the card: anyone who asks, or friends, the guild and the group; never one the player hid from
-- this session or ignores.
local function sharesWith(key)
	if not Social.Sharing() or Social.hide[key] or ignored(key) then return false end
	if ns.profile.share == "anyone" then return true end
	return Social.GroupKeys()[key] ~= nil or guildmate(key, Social.peers[key]) or friend(key)
end

-- Whether a message on a group's or the guild's channel would reach a player the player hid from this session or
-- ignores: then nothing goes on that channel (the ignore list and "hide" hold both ways), and a party gets it by
-- whisper. Blocked too while that cannot be told: before the guild roster has come (the roster is what knows a
-- guildmate), or while a member's name cannot be read.
local function blocked(channel)
	if channel == "GUILD" then
		if not Social.rosterReady then return true end
		for key in pairs(Social.hide) do
			if guildmate(key, Social.peers[key]) then return true end
		end
		for i = 1, C_FriendList.GetNumIgnores() do
			local name = C_FriendList.GetIgnoreName(i)
			if name and guildmate(Social.Key(name)) then return true end
		end
		return false
	end
	local keys, complete = Social.GroupKeys()
	if not complete then return true end
	for key in pairs(keys) do
		if Social.hide[key] or ignored(key) then return true end
	end
	return false
end

-- The card's audience from the question or the Social page, and the hellos a new answer owes: the group and the guild
-- learn it at once, not at the next login, and a card taken back is "-C" to them, which drops it from their session.
function Social.SetShare(v)
	if not ns.profile then return end
	local was = ns.profile.share
	ns.profile.share = v
	ns.Touch()
	ns.Emit("Settings")
	if (v ~= "nobody") ~= (was ~= "nobody") then Social.Hello(true) end
	if was == "nobody" and v ~= "nobody" then Social.ResumeStakes() end
end

-- Sending ---------------------------------------------------------------------------------------------------------

-- The token bucket (Blizzard's MessageQueue.lua shape: 10 tokens, 1 a second, a keyed message replaced in place; no
-- OnUpdate, a timer while messages wait), paused while addon chat is locked down (encounters, Mythic+, PvP matches).

local queue, tokens, refilled, paused = {}, C.bucket_burst, nil, false
local reported = {}

local drain, recheck

-- Whether a message may go now, checked when it is queued and again when it leaves: the answer may have changed, a
-- player been hidden, an instance entered. Everything but a private "-C" and the end of a pact or a duel ("P-", "V-",
-- which carry nothing of the card) needs the card shared (a hidden player gets "-C" as a stranger does), and nothing
-- goes on a group's or the guild's channel that reaches a hidden or ignored player.
local function allowed(m)
	if not Social.Listening() or (m.target and ignored(m.target)) then return false end
	local bare = m.text == "-C" or m.text:find("^[PV]%-") ~= nil
	if not bare and (not Social.Sharing() or (m.target and Social.hide[m.target])) then return false end
	return m.channel == "WHISPER" or not blocked(m.channel)
end

local function refill()
	local now = GetTime()
	if refilled then tokens = math.min(C.bucket_burst, tokens + (now - refilled) * C.bucket_rate) end
	refilled = now
end

local function schedule()
	if #queue > 0 and not paused and not T.active("drain") then
		T.after("drain", (1 - tokens) / C.bucket_rate, drain)
	end
end

function drain()
	refill()
	if C_ChatInfo.InChatMessagingLockdown() then
		paused = true
		return
	end
	while #queue > 0 and tokens >= 1 do
		local m = queue[1]
		if not allowed(m) then
			table.remove(queue, 1)
			return drain()
		end
		local account = m.channel == "WHISPER" and gameAccount(m.target)
		local result
		if account then
			result = C_BattleNet.SendGameData(account, PREFIX, m.text)
		else
			result = C_ChatInfo.SendAddonMessage(PREFIX, m.text, m.channel, m.target)
		end
		if THROTTLE[result] then
			tokens = 0
			break
		elseif result == R.AddOnMessageLockdown then
			-- refused as locked down: read the lockdown again a token's time later, whether or not an event comes
			paused = true
			T.after("lockdown", 1 / C.bucket_rate, recheck)
			return
		end
		table.remove(queue, 1)
		tokens = tokens - 1
		if result ~= R.Success and not DROP[result] and not reported[result] then
			reported[result] = true
			ns.Report(("QuestPump: an addon message was refused (%s)"):format(tostring(result)))
		end
	end
	schedule()
end

-- Queues one message; `key` coalesces (a newer message with the same key takes the waiting one's place), `urgent` goes
-- to the front (a pump's start, whose Join line the party sees at once). At most C.peer_max wait: a flood of forged
-- requests never grows the queue past the players a session keeps.
local function send(text, channel, target, key, urgent)
	local m = { text = text, channel = channel, target = target, key = key }
	if not allowed(m) then return end
	for i, q in ipairs(queue) do
		if key and q.key == key then
			queue[i] = m
			return
		end
	end
	if #queue >= C.peer_max then return end
	table.insert(queue, urgent and 1 or #queue + 1, m)
	if not paused then drain() end
end
Social.Send = send

-- Chat's lockdown turns on or off: read again a moment after the event, never during its dispatch (the restriction
-- APIs answer false then).
function recheck()
	paused = C_ChatInfo.InChatMessagingLockdown() == true
	if not paused then drain() end
end

-- The group channel: the instance group's, the raid's or the party's.
local function groupChannel()
	if IsInGroup(LE_PARTY_CATEGORY_INSTANCE) then return "INSTANCE_CHAT" end
	if IsInRaid() then return "RAID" end
	if IsInGroup() then return "PARTY" end
	return nil
end

-- One message about the player's pump to the party (never a raid's forty): on the party's channel, or, while that
-- would reach a player the player hides from or ignores, to each other member by whisper (the queue drops theirs).
local function toParty(text, key, urgent)
	local channel = groupChannel()
	if not channel or IsInRaid() then return end
	if not blocked(channel) then return send(text, channel, nil, key, urgent) end
	for member in pairs(Social.GroupKeys()) do send(text, "WHISPER", member, key .. ":" .. member, urgent) end
end

-- Whether a message came from the party: on its channel, or by whisper from one of its members (the way round a
-- hidden or ignored member); never in a raid.
local function fromParty(key, channel)
	if IsInRaid() then return false end
	return GROUP_CHANNEL[channel] or (channel == "WHISPER" and Social.GroupKeys()[key] ~= nil)
end

-- The card --------------------------------------------------------------------------------------------------------

-- Today's game day as a day number (Stats.lua's), the pact's and the duel's calendar.
local gameDay = ns.Stats.DayIndex

-- Whether the player kept their side of the pact today (slice 23): a pump counted this game day (ns.CountedPumps, as
-- the titles count it), and for "we pay every pump the same evening" no unfinished move.
local function pactDone(plan)
	return ns.CountedPumps(ns.state) > 0 and (plan ~= E.pact.evening or ns.OwedCount(ns.state) == 0)
end

-- The player's score in the duel `d` and whether it is final. Final from the first reading after its last game day,
-- and kept (d.final): unfinished moves paid later, or the app's pumps the mirror brings back with their own times, may
-- add to a past day's count, which would change a score its rivals have already settled on.
local function duelScore(d, now)
	if d.final then return d.final, true end
	local score = ns.DuelScore(ns.state, d.start)
	if now <= d.start + C.duel_days - 1 then return score, false end
	d.final = score
	ns.Touch()
	return score, true
end

-- A duel's field; a settled duel the player challenged others to adds its top final score (settleDuel).
local function duelField(d, now)
	local score, final = duelScore(d, now)
	local field = ("V%d.%d.%d"):format(d.id, score, final and 1 or 0)
	return d.top and field .. "." .. d.top or field
end

-- The last game day a settled duel's final score stays on the card for rivals still settling: its grace, and a day.
local function pastEnds(past)
	return past.start + C.duel_days - 1 + C.duel_grace_days + 1
end

-- SipHash-2-4 (Aumasson and Bernstein, 2012), the pact's sign: a keyed hash no one computes without its 128-bit key,
-- however many of its outputs they read. In plain arithmetic, each 64-bit word as its two 32-bit halves, high first,
-- every step exact in a double, so it needs no bit library. Checked once against the reference's vectors and Rust's
-- SipHasher, a probe outside the repo (2026-10-31).
local W32 = 4294967296
local XOR4 = {} -- the exclusive or of two 4-bit numbers a and b, at a * 16 + b + 1
for a = 0, 15 do
	for b = 0, 15 do
		local r, m, x, y = 0, 1, a, b
		for _ = 1, 4 do
			local xb, yb = x % 2, y % 2
			if xb ~= yb then r = r + m end
			x, y, m = (x - xb) / 2, (y - yb) / 2, m * 2
		end
		XOR4[a * 16 + b + 1] = r
	end
end

local function xor(a, b)
	local r, m = 0, 1
	for _ = 1, 8 do
		local x, y = a % 16, b % 16
		r, a, b, m = r + XOR4[x * 16 + y + 1] * m, (a - x) / 16, (b - y) / 16, m * 16
	end
	return r
end

local function add(ah, al, bh, bl)
	local l = al + bl
	if l >= W32 then return (ah + bh + 1) % W32, l - W32 end
	return (ah + bh) % W32, l
end

local function rotl(h, l, n) -- by 0 < n < 32
	local up, down = 2 ^ n, 2 ^ (32 - n)
	return h % down * up + math.floor(l / down), l % down * up + math.floor(h / down)
end

local function sipRound(ah, al, bh, bl, ch, cl, dh, dl)
	ah, al = add(ah, al, bh, bl)
	bh, bl = rotl(bh, bl, 13)
	bh, bl, ah, al = xor(bh, ah), xor(bl, al), al, ah
	ch, cl = add(ch, cl, dh, dl)
	dh, dl = rotl(dh, dl, 16)
	dh, dl = xor(dh, ch), xor(dl, cl)
	ah, al = add(ah, al, dh, dl)
	dh, dl = rotl(dh, dl, 21)
	dh, dl = xor(dh, ah), xor(dl, al)
	ch, cl = add(ch, cl, bh, bl)
	bh, bl = rotl(bh, bl, 17)
	bh, bl, ch, cl = xor(bh, ch), xor(bl, cl), cl, ch
	return ah, al, bh, bl, ch, cl, dh, dl
end

local function word(s, i) -- four bytes from i, little-endian
	local b0, b1, b2, b3 = s:byte(i, i + 3)
	return ((b3 * 256 + b2) * 256 + b1) * 256 + b0
end

local function hex32(n)
	return ("%04x%04x"):format(math.floor(n / 65536), n % 65536)
end

-- `msg` under the 16-byte `key`, as 16 hex digits.
local function sipHash(key, msg)
	local k0l, k0h, k1l, k1h = word(key, 1), word(key, 5), word(key, 9), word(key, 13)
	local ah, al = xor(k0h, 0x736f6d65), xor(k0l, 0x70736575)
	local bh, bl = xor(k1h, 0x646f7261), xor(k1l, 0x6e646f6d)
	local ch, cl = xor(k0h, 0x6c796765), xor(k0l, 0x6e657261)
	local dh, dl = xor(k1h, 0x74656462), xor(k1l, 0x79746573)
	local function absorb(mh, ml)
		dh, dl = xor(dh, mh), xor(dl, ml)
		for _ = 1, 2 do ah, al, bh, bl, ch, cl, dh, dl = sipRound(ah, al, bh, bl, ch, cl, dh, dl) end
		ah, al = xor(ah, mh), xor(al, ml)
	end
	local n = #msg
	for i = 1, n - n % 8, 8 do absorb(word(msg, i + 4), word(msg, i)) end
	local last = msg:sub(n - n % 8 + 1) .. ("\0"):rep(7 - n % 8) .. string.char(n % 256)
	absorb(word(last, 5), word(last, 1))
	cl = xor(cl, 0xff)
	for _ = 1, 4 do ah, al, bh, bl, ch, cl, dh, dl = sipRound(ah, al, bh, bl, ch, cl, dh, dl) end
	return hex32(xor(xor(ah, bh), xor(ch, dh))) .. hex32(xor(xor(al, bl), xor(cl, dl)))
end

-- A pact's id: 128 bits in eight draws, as 32 hex digits; it travels only in the offer and its answer, between the
-- two, and a sign (pactSign) is 16. An id of another shape (a pact sworn before 2026-10-31, nine digits a friend who
-- read a card could work out: the release's security review) signs nothing, so that pact ends as a partner's card
-- comes without it.
local PACT_ID, PACT_SIGN = 32, 16

local function drawPactId()
	local parts = {}
	for i = 1, PACT_ID / 4 do parts[i] = ("%04x"):format(math.random(0, 65535)) end
	return table.concat(parts)
end

-- `s` when it is `n` lowercase hex digits.
local function hexOf(s, n)
	return type(s) == "string" and #s == n and not s:find("[^0-9a-f]") and s or nil
end

-- A pact's id as one character shows it, on its card's Q field and its end ("P-"): that character's key under the
-- id (sipHash), so the pair, who hold the id, make it for any of their characters (the partner on an alt), and a
-- friend who read the player's card cannot make it for their own name. Nil for an id that is no pact's (above). The
-- signs are kept while the pact's id stays, since every card read asks for one again.
local signs, signsOf = {}, nil

local function pactSign(id, key)
	if not hexOf(id, PACT_ID) then return nil end
	if signsOf ~= id then signs, signsOf = {}, id end
	key = key or ""
	if not signs[key] then
		signs[key] = sipHash(id:gsub("%x%x", function(b) return string.char(tonumber(b, 16)) end), key)
	end
	return signs[key]
end

-- The card's pact and duel fields (slice 23): "Q<pact sign>.<done today>" and "V<duel id>.<score>.<final>[.<top>]",
-- for the duel and for a settled one whose final score its rivals may still ask for. They stand before the typed
-- records, which the byte cap cuts first.
local function stakes()
	local out, p, now = {}, ns.profile, gameDay()
	local sign = p.pact and pactSign(p.pact.id, Social.me)
	if sign then out[#out + 1] = ("Q%s.%d"):format(sign, pactDone(p.pact.plan) and 1 or 0) end
	if p.duel then out[#out + 1] = duelField(p.duel, now) end
	local past = p.duelPast
	if past and now <= pastEnds(past) then out[#out + 1] = duelField(past, now) end
	return out
end

-- The card as the protocol writes it, "L12|T7|B3|S9|A20|W34|R5|M42|F1|D1|N14" (N: the game day's moves) and the GUID,
-- the pact's and the duel's fields; with shared records one "Y" per strength ladder (ladder.rung.move.first.best.day),
-- as many as fit in the message.
local function body()
	local c = ns.Card(ns.state, ns.profile, ns.now())
	local parts = { "L" .. c.level, "T" .. c.title, "B" .. c.border, "S" .. c.streak, "A" .. c.active,
		"W" .. c.pumps, "R" .. c.beats, "M" .. c.cardio, "F" .. (Lc.InPump() and 1 or 0), "D" .. (c.today and 1 or 0),
		"N" .. c.movesToday }
	if c.paragon > 0 then parts[#parts + 1] = "P" .. c.paragon end
	local guid = readable(UnitGUID("player"))
	if guid then parts[#parts + 1] = "G" .. guid end
	for _, field in ipairs(stakes()) do parts[#parts + 1] = field end
	local text = table.concat(parts, "|")
	if ns.profile.shareRecords then
		for _, r in ipairs(ns.TypedRecords(ns.state)) do
			local y = ("|Y%d.%d.%d.%d.%d.%d"):format(r.ladder, r.rung, r.move, r.first, r.best, r.day)
			if HEAD + #text + #y > MAX_BYTES then break end
			text = text .. y
		end
	end
	return text
end

-- The card's version is the time of its last change, rising even when the beta forgets the save; the card is read
-- again whenever it is asked for, so a day or a week that turned needs no row.
local version, lastBody = 0, nil

local function current()
	local text = body()
	if text ~= lastBody then
		lastBody = text
		version = math.max(ns.now(), version + 1)
	end
	return version, text
end

local function cardMessage()
	local ver, text = current()
	return ("C%d:%s"):format(ver, text)
end

local function number(s, field)
	local n = tonumber(s:match("^%d%d?%d?%d?%d?%d?$"))
	local b = BOUNDS[field]
	return n and math.max(b[1], math.min(b[2], n)) or nil
end

-- One shared record, checked against the library (a move of that ladder) and clamped to what the rep box accepts. A
-- ladder and a rung take up to two digits (slice 21: ladders up to 23); an older card's single digits read the same,
-- and a record of a retired ladder (the pull, the core hold) fails the library check and is left out.
local function typed(s, today)
	local l, rung, move, first, best, day =
		s:match("^(%d%d?)%.(%d%d?)%.(%d%d?%d?%d?)%.(%d%d?%d?)%.(%d%d?%d?)%.(%d%d?%d?%d?%d?%d?)$")
	l, rung, move = tonumber(l), tonumber(rung), tonumber(move)
	local m = move and ns.Library[move]
	if not (l and ns.STRENGTH[l] and m and m.ladder == l and rung >= 1 and rung <= #ns.Ladders[l].rungs) then
		return nil
	end
	local cap = m.dose == "hold" and C.max_seconds or C.max_reps
	return { ladder = l, rung = rung, move = move, first = math.min(tonumber(first), cap),
		best = math.min(tonumber(best), cap), day = math.min(tonumber(day), today) }
end

-- A card's body read back: anchored fields, length-capped numbers, every value clamped, unknown fields skipped (a
-- newer addon's), never loaded as code. Nil when the level is missing. A pact's sign takes its 16 hex digits, a duel's
-- id nine digits at most, a duel's scores are clamped to what its caps allow.
function Social.Decode(text)
	local c = { typed = {}, duels = {} }
	local today = ns.DayIndex(ns.day())
	for field in text:gmatch("[^|]+") do
		local k, v = field:match("^(%u)(.*)$")
		if k == "Y" then
			local r = typed(v, today + 1)
			if r then c.typed[#c.typed + 1] = r end
		elseif k == "G" then
			c.guid = v:match("^Player%-%d+%-%x+$") and #v <= 40 and v or nil
		elseif k == "Q" then
			local sign, done = v:match("^(%x+)%.([01])$")
			if hexOf(sign, PACT_SIGN) then c.pact = { id = sign, done = done == "1" } end
		elseif k == "V" then
			local id, score, final, top = v:match("^(%d+)%.(%d+)%.([01])%.(%d+)$")
			if not id then id, score, final = v:match("^(%d+)%.(%d+)%.([01])$") end
			if id and #id <= 9 and #score <= 6 and #(top or "") <= 6 then
				c.duels[tonumber(id)] = { score = math.min(tonumber(score), DUEL_TOP), final = final == "1",
					top = top and math.min(tonumber(top), DUEL_TOP) }
			end
		elseif k and BOUNDS[k] then
			c[k] = number(v, k)
		end
	end
	if not c.L then return nil end
	c.title = TITLE_OF[c.T or 0]
	return c
end

-- My own card in the decoded shape, so both columns of the card window read the same way.
function Social.Mine()
	local _, text = current()
	return Social.Decode(text)
end

-- Asking and answering ---------------------------------------------------------------------------------------------

-- Asks a player for their card: at most once in C.probe_s, and one that never answered only after C.reprobe_s. A
-- private player never asks, and a player the card is hidden from or who is ignored is never asked.
local function ask(key)
	if not (key and key ~= Social.me and Social.Sharing()) or Social.hide[key] or ignored(key) then return end
	local p = peer(key)
	local now = GetTime()
	if p.asked and now - p.asked < (p.answered and C.probe_s or C.reprobe_s) then return end
	p.asked, p.answered = now, false
	send(("?C%d"):format(p.ver or 0), "WHISPER", key, "ask:" .. key)
end

-- The asks that show a card (hover, target, the menu, a link): only while the player shows others' cards. The party's
-- own asks (who runs QuestPump, for the mixed party) never wait on that switch.
function Social.Ask(key)
	if ns.profile.showCards then ask(key) end
end

-- A player unit at hand is asked when it could answer: a friendly player online (a whisper cannot cross the
-- factions, nor reach a player offline).
local function askUnit(unit)
	if unit and readable(UnitIsFriend("player", unit)) and readable(UnitIsConnected(unit)) ~= false then
		Social.Ask(Social.UnitKey(unit))
	end
end
Social.AskUnit = askUnit

-- My answer to a player: the card, "unchanged" when they hold this version, or "private".
local function answer(key, theirs)
	if not sharesWith(key) then return send("-C", "WHISPER", key, "reply:" .. key) end
	local ver = current()
	if theirs == ver then return send(("!C%d"):format(ver), "WHISPER", key, "reply:" .. key) end
	send(cardMessage(), "WHISPER", key, "reply:" .. key)
end

-- Once per login to the guild, once on joining a group: the card itself, so the group and the guild learn it without a
-- request each. A private player's group hello is "-C" (so the party learns who runs QuestPump); the guild hears from
-- them only when they take a shared card back. A party with a hidden or ignored member gets it by whisper, member by
-- member; a raid or a guild with one gets none (never forty whispers), and the guild's waits for its roster, which is
-- what knows a guildmate.
local function sendGuildHello()
	if not (IsInGuild() and Social.Listening()) then return end
	if not Social.rosterReady then
		Social.guildWaiting = true -- the roster's first update sends it
		return
	end
	Social.guildHello, Social.guildWaiting = true, nil
	send(Social.Sharing() and cardMessage() or "-C", "GUILD", nil, "hello:guild")
end

-- `retake`: an answer the player changed, told to the guild again (a card taken back goes as "-C").
local function helloGuild(retake)
	if not IsInGuild() or T.active("helloGuild") then return end
	if not (retake and Social.Listening()) and (Social.guildHello or not Social.Sharing()) then return end
	T.after("helloGuild", math.random() * C.hello_jitter_s, sendGuildHello)
end

function Social.HelloGroup()
	if not (Social.Listening() and groupChannel()) then return end
	T.after("helloGroup", math.random() * C.hello_jitter_s, function()
		local channel = groupChannel()
		if not (Social.Listening() and channel) then return end
		if not blocked(channel) then
			return send(Social.Sharing() and cardMessage() or "-C", channel, nil, "hello:group")
		end
		if IsInRaid() then return end
		for key in pairs(Social.GroupKeys()) do answer(key, nil) end
	end)
end

function Social.Hello(retake)
	helloGuild(retake)
	Social.HelloGroup()
end

-- The party's members not known yet are asked once the hellos and their replies had their time (two jitters), so a
-- member who joined, or whose hello the player missed, is learned without a second message for everyone; a raid's
-- members only from their own hellos (never an ask each for forty).
local function askParty()
	if T.active("askParty") then return end
	T.after("askParty", 2 * C.hello_jitter_s, function()
		if IsInRaid() then return end
		for key in pairs(Social.GroupKeys()) do
			if not (Social.peers[key] and Social.peers[key].known) then ask(key) end
		end
	end)
end

-- A card or a private answer came: the player runs QuestPump. A card on the guild's channel is a guildmate's; one
-- whose GUID disagrees with the unit at hand is forged and dropped.
local function known(p)
	local was = p.known
	p.known, p.seen, p.answered = true, GetTime(), true
	if not was then ns.Emit("Peers") end
end

-- A hello, from a login, a /reload or a join, is answered every time by whisper, after a jitter so a guild's replies
-- spread out; a private player answers only a group's (a card or "-C"), and a whisper is never answered.
local function replyToHello(key, channel)
	if not (channel == "GUILD" or GROUP_CHANNEL[channel]) then return end
	if channel == "GUILD" and not Social.Sharing() then return end
	T.after("reply:" .. key, math.random() * C.hello_jitter_s, function() answer(key, nil) end)
end

local readStake -- a pact's or a duel's card read (below, "Pacts and duels")

local function onCard(key, ver, text, channel)
	local c = Social.Decode(text)
	if not c then return end
	local unit = Social.GroupKeys()[key] or (Social.UnitKey("target") == key and "target")
		or (Social.UnitKey("mouseover") == key and "mouseover")
	local guid = unit and readable(UnitGUID(unit))
	if guid and c.guid and guid ~= c.guid then return end
	local p = peer(key)
	p.card, p.ver, p.private = c, ver, false
	if c.N then p.today = { day = gameDay(), moves = c.N } end
	if channel == "GUILD" then p.guild = true end
	known(p)
	readStake(key, c)
	replyToHello(key, channel)
	ns.Emit("Social", key)
end

-- The party -----------------------------------------------------------------------------------------------------------

-- A pump's id on the wire: a number drawn when the pump first needs one, never its time (the card and its messages
-- carry no pump times, plan slice 13); the same pump keeps it for its start and its end. A duel's id is drawn the same
-- way (nine digits at most, as the messages read them), a pact's in eight parts (drawPactId).
local ids = setmetatable({}, { __mode = "k" })

local function drawId()
	return math.random(1, 999999999)
end

local function pumpId(p)
	ids[p] = ids[p] or drawId()
	return ids[p]
end

-- The player's own moves this game day, as the card carries them (ns.Card, from the day's count).
local function myMoves()
	return ns.DayCount(ns.state, ns.now()).moves
end

-- The party's moves this game day, for its line on the HUD: a pump's end tells the party the player's moves when they
-- changed since last told, "N<moves>", one message a pump and never a pulse, and so does the game day's turn
-- (dayTurned), since a receiver dates a count by its own day and a member's day turns at their own 04:00; a member
-- who joins later reads them on the card's N field. They took the place of "Pump together" and its party countdown
-- (David, 2026-10-09: "it doesn't really do anything extra except spam your group"). Only what reaches a party counts
-- as told: a raid hears nothing.
local told = nil -- { day, moves }

local function tellMoves()
	if not (Social.Sharing() and groupChannel()) or IsInRaid() then return end
	local day, moves = gameDay(), myMoves()
	if told and told.day == day and told.moves == moves then return end
	told = { day = day, moves = moves }
	toParty(("N%d"):format(moves), "moves")
end

-- A party member's moves this game day, from their pump's end (a card brings them too: onCard).
local function onMoves(key, moves, channel)
	if not fromParty(key, channel) then return end
	peer(key).today = { day = gameDay(), moves = math.min(moves, BOUNDS.N[2]) }
	ns.Emit("Social", key)
end

-- The Join line and the latest finished pump someone cheers for; the guild's digest of the day.
local invite, finished, digest = nil, nil, nil

local function clearLater(name, seconds, clear)
	T.after(name, seconds, function()
		clear()
		ns.Emit("Social")
	end)
end

-- A party member's pump started: the Join line while the player has no pump, and their card's in-a-pump flag; nothing
-- from a muted player.
local function onStart(key, id, channel)
	local p = Social.peers[key]
	if p and p.card then p.card.F = 1 end
	if not fromParty(key, channel) or Social.mute[key] then return end
	if ns.profile.cheers and not Lc.pump then
		invite = { from = key, id = id }
		clearLater("invite", C.join_line_s, function() invite = nil end)
	end
	ns.Emit("Social")
end

-- A guild whose players seen this session pass C.cheer_digest_peers gets one line for the day instead of a line per
-- pump; the line counts pumps and players, never who has not paid.
local function guildSize()
	local n = 0
	for key, p in pairs(Social.peers) do
		if p.known and guildmate(key, p) then n = n + 1 end
	end
	return n
end

local function onDone(key, id, channel)
	local p = Social.peers[key]
	if p and p.card then p.card.F = 0 end
	local party = fromParty(key, channel)
	if invite and invite.from == key and invite.id == id then invite = nil end
	if not ns.profile.cheers or Social.mute[key] then return ns.Emit("Social") end
	if channel == "GUILD" and guildSize() > C.cheer_digest_peers then
		local day = ns.day()
		if not digest or digest.day ~= day then digest = { day = day, pumps = 0, people = {}, n = 0, seen = {} } end
		local pump = key .. ":" .. id -- each pump once, however often its message comes
		if digest.seen[pump] then return end
		digest.seen[pump] = true
		digest.pumps = digest.pumps + 1
		if not digest.people[key] then
			digest.people[key] = true
			digest.n = digest.n + 1
		end
	elseif channel == "GUILD" or party then
		finished = { from = key, id = id }
		clearLater("finished", C.cheer_line_s, function() finished = nil end)
	end
	ns.Emit("Social")
end

-- Cheers: fixed codes from the closed list (ns.Enum.cheer), one per sender per pump, by whisper, only from a player
-- who shares the card; received, a toast.
local mine = {} -- my pumps' ids announced this session, the ones a cheer may name

function Social.Cheer(key, id, code)
	local p = peer(key)
	if p.cheered[id] or not CHEERS[code] or not Social.Sharing() then return end
	p.cheered[id] = true
	send(("K%d:%d"):format(code, id), "WHISPER", key, "cheer:" .. key)
	if finished and finished.from == key and finished.id == id then finished = nil end
	ns.Emit("Social")
end

-- A Legendary unlock told to the party and the guild (slice 22): only from a player who shares the card and switched
-- "Announce Legendary unlocks" on (off by default), one message with the move's id. Received, a toast for the players
-- who show cheers, as a cheer's: a move this addon does not know as Legendary shows nothing, and a player shows one a
-- session (the message comes on the party's and the guild's channel both, and any addon can send any move id).
function Social.Legend(moveId)
	if not (ns.profile.tellLegend and Social.Sharing()) then return end
	local text = ("U%d"):format(moveId)
	toParty(text, "legend:group:" .. moveId)
	if IsInGuild() then send(text, "GUILD", nil, "legend:guild:" .. moveId) end
end

local function onLegend(key, id, channel)
	local m = ns.Library[id]
	local q = m and m.ladder and ns.Quality(m.ladder, m.rung)
	if not (q and q >= ns.Enum.quality.legendary) then return end
	if not ns.profile.cheers or Social.mute[key] or not (channel == "GUILD" or fromParty(key, channel)) then return end
	local p = peer(key)
	if p.legend then return end
	p.legend = true
	Sheet.Toast(nil, L["toast.legend.other"]:format(key, L["move." .. m.key]),
		L["toast.legend.other.sub"]:format(L["quality." .. q]), nil, "win")
end

local function onCheer(key, code, id)
	local p = peer(key)
	if not mine[id] or p.cheers[id] or Social.mute[key] or not ns.profile.cheers then return end
	p.cheers[id] = true
	local name = CHEERS[code]
	if name then Sheet.Toast(nil, L["toast.cheer." .. name]:format(key), nil, nil, "win") end
end

-- The State listener: a pump's first start in a party announces it (the Join line of the others); a pump that ends
-- done announces that (the cheers); any pump's end tells the party the day's moves when they changed.
local function onState(p, how)
	if not p or p.kind == "practice" or not p.opened then return end
	local id = pumpId(p)
	if p.state == "countdown" and p.index == 1 and mine[id] == nil then
		mine[id] = true
		if Social.Sharing() then toParty(("S%d"):format(id), "start", true) end
	elseif p.state == "done" then
		if how == "done" then
			mine[id] = true
			if Social.Sharing() then
				toParty(("D%d"):format(id), "done:group")
				if IsInGuild() then send(("D%d"):format(id), "GUILD", nil, "done:guild") end
			end
			ns.Emit("Social")
		end
		tellMoves()
	end
end

ns.On("State", onState)
-- A save bound again (a reset, an import, an undone rung-up) may change the day's moves with no pump's end; at the
-- login's bind Social.me is not known yet, and the group's hello carries the card then.
ns.On("DB", function() if Social.me then tellMoves() end end)

-- Pacts and duels (slice 23) ---------------------------------------------------------------------------------------

-- With friends only, one pact and one duel at a time, kept in the profile, which the helper's mirror keeps through the
-- beta's lost saves: the pact { id, with, plan, since }, the duel { id, start, rivals, own } and a settled duel's
-- { id, start, top } (duelPast, no names). A partner's and the rivals' names stay only while the pact or the duel
-- lasts (the one exception to keeping nothing about other players past the session), their numbers only from their
-- cards this session. Nothing here changes a pump, XP, a record or the unfinished moves.

-- The offers the player made this session, waiting for an answer: the pact's { id, to, plan, at } and the duel's
-- challenges { id, start, to = { [key] = when } }; who of the partner and the rivals was online at the last look;
-- when a pact or a duel began with a player this session (a card of theirs sent before it may still lack its id).
local offers, present, joinedAt = {}, {}, {}
-- The latest card this session that carried a pact's id, { id, key, day, done }: the partner's, or the partner's on
-- another character (the id is the pair's own; never stored). It speaks for the pact in use while the ids match.
local pactSeen

local function seen(pact)
	return pact and pactSeen and pactSeen.id == pact.id and pactSeen or nil
end
-- The card's pact and duel fields as the partner and the rivals last had them.
local stakeSent

-- Whether an offer made at `at` still stands: its popup's time and an answer's way back.
local function waiting(at)
	return at ~= nil and GetTime() - at < C.invite_s + C.probe_s
end

-- The players the pact's and the duel's card goes to: the partner (and the character their pact's card last came
-- from) and the rivals.
local function partners()
	local out, p = {}, ns.profile
	if p.pact then
		out[1] = p.pact.with
		local voice = seen(p.pact)
		if voice and voice.key ~= p.pact.with then out[2] = voice.key end
	end
	for _, key in ipairs(p.duel and p.duel.rivals or {}) do
		if not tIndexOf(out, key) then out[#out + 1] = key end
	end
	return out
end

local function pushTo(key)
	if sharesWith(key) then send(cardMessage(), "WHISPER", key, "push:" .. key) end
end

-- My card to the partner and the rivals online, once, when its pact or duel fields changed (a pump paid, a day
-- turned): pushed, never polled. While the layer is quiet (an instance it is off in, the card kept private) nothing
-- goes and the change waits for the layer to listen again (listen); who is online is read from the friends lists as it
-- goes, never from a look taken while the layer slept.
local function pushStakes()
	if not (ns.profile and Social.Sharing()) then return end
	local now = table.concat(stakes(), "|")
	if now == stakeSent then return end
	stakeSent = now
	for _, key in ipairs(partners()) do
		if online(key) then pushTo(key) end
	end
end

-- The game day's turn is looked at while there is a pact, a duel or a group: a timer to the next 04:00, never a
-- pulse. The turn also tells the party the new day's moves, so no member keeps yesterday's count as today's.
local lookAtDay

local function dayTurned()
	lookAtDay()
	tellMoves()
end

local function watchDay()
	local p = ns.profile
	if not (p and (p.pact or p.duel or p.duelPast or IsInGroup())) then return T.cancel("dayTurn") end
	local now = ns.now()
	T.after("dayTurn", ns.NextDayStart(now) - now + 1, dayTurned)
end

-- A pact or a duel just began with `key`: my card goes to them and theirs is asked for.
local function joined(key)
	joinedAt[key], present[key] = GetTime(), true -- they just answered
	ns.Touch()
	pushTo(key)
	ask(key)
	stakeSent = table.concat(stakes(), "|")
	watchDay()
	ns.Emit("Social")
end

-- An offer is heard only with invites on, from a player not muted or hidden from, while the player shares the card
-- (the answer is a card's message); only a friend's offer gets this far (onStake), and no ignored player's (onMessage).
local function hears(key)
	return ns.profile.invites and Social.Sharing() and not (Social.mute[key] or Social.hide[key])
end

-- The pact ---------------------------------------------------------------------------------------------------------

-- Whether the player may offer a pact now: invites on, the card shared, no pact and no offer standing, a friend
-- online.
local function canPact(key)
	local p = ns.profile
	return p.invites and Social.Sharing() and not p.pact and not (offers.pact and waiting(offers.pact.at))
		and not (Social.hide[key] or ignored(key)) and friend(key) and online(key)
end

function Social.OfferPact(key, plan)
	if not (PLANS[plan] and canPact(key)) then return end
	offers.pact = { id = drawPactId(), to = key, plan = plan, at = GetTime() }
	send(("P?%s.%d"):format(offers.pact.id, plan), "WHISPER", key, "pact:" .. key)
	ns.Print("social.pact.sent", key)
end

local function swear(key, id, plan)
	ns.profile.pact, pactSeen = { id = id, with = key, plan = plan, since = gameDay() }, nil
	ns.Print("social.pact.sworn", key)
	joined(key)
end

-- The popup's "Swear it": the pact, while nothing changed as the popup showed and the friend is still online (an
-- answer they cannot get leaves their offer to lapse). An offer of mine to them stays: should they swear it as well,
-- the smaller id stands (onPactJoin).
local function acceptPact(o)
	if not (Social.Sharing() and hears(o.key) and online(o.key)) or ns.profile.pact then return end
	send(("P+%s"):format(o.id), "WHISPER", o.key, "pact:" .. o.key)
	swear(o.key, o.id, o.plan)
end

-- An answer to my offer: the pact, or, for an offer that lapsed or a pact sworn meanwhile with someone else, "P-" (it
-- is over for them). Two friends who offered each other and both swore hold two ids: the smaller stands on both sides,
-- so the one whose own offer has it takes it, plan and all, and the other keeps what it holds.
local function onPactJoin(key, id)
	local pact, o = ns.profile.pact, offers.pact
	local ours = o and o.id == id and o.to == key and waiting(o.at)
	if pact and pact.with == key and (pact.id == id or ours) then -- said twice, or the crossing
		if pact.id ~= id and id < pact.id then
			pact.id, pact.plan = id, o.plan
			ns.Touch()
			pushStakes()
			ns.Emit("Social")
		end
		return
	end
	if pact or not ours then return send(("P-%s"):format(pactSign(id, Social.me)), "WHISPER", key, "pact:" .. key) end
	offers.pact = nil
	swear(key, id, o.plan)
end

-- The pact ends, its partner's name with it, and one chat line says so.
local function endPact(lineKey)
	local key = ns.profile.pact.with
	ns.profile.pact, pactSeen = nil, nil
	ns.Touch()
	ns.Print(lineKey, key)
	watchDay()
	ns.Emit("Social")
end

-- The pact's end, signed by its sender's key (pactSign): from the partner, or a friend that is the partner on another
-- character, as a card that carries the pact speaks for the partner (readStake).
local function onPactEnd(key, id)
	local pact = ns.profile.pact
	if pact and id == pactSign(pact.id, key) and (pact.with == key or friend(key)) then endPact("social.pact.over") end
end

-- "End the pact" (the HUD line's menu): it costs nothing; the partner is told while online, else by my next card,
-- which lacks the pact.
function Social.EndPact()
	local pact = ns.profile and ns.profile.pact
	if not pact then return end
	local voice, sign = seen(pact), pactSign(pact.id, Social.me)
	for _, key in ipairs({ pact.with, voice and voice.key ~= pact.with and voice.key or nil }) do
		if sign and online(key) then send(("P-%s"):format(sign), "WHISPER", key, "pact:" .. key) end
	end
	endPact("social.pact.ended")
end

-- The duel ---------------------------------------------------------------------------------------------------------

local function endDuel(lineKey)
	ns.profile.duel, offers.duel = nil, nil
	ns.Touch()
	ns.Print(lineKey)
	watchDay()
	ns.Emit("Social")
end

-- A rival's score this session for the duel `d`: { score, final, top } from their card, nil while none came.
local function rivalScore(key, d)
	local p = Social.peers[key]
	local v = p and p.duel
	return v and v.id == d.id and v or nil
end

-- After the duel's last game day. The challenger alone knows every rival (an accepter knows only the challenger, as
-- no name travels), so the challenger is the duel's hub: it settles once every rival's final card came this session,
-- and its settled card carries the duel's top final score; an accepter settles on that. A win is a final score at the
-- top (a tie shares it) and above 0: a duel nobody played crowns no one. Dropped with no title once duel_grace_days
-- passed without them. A win raises the profile's duelWins, which no row carries, so the Duelist title's toast goes
-- through the sheet's owner here (Sheet.Earned).
local function settleDuel()
	local d = ns.profile and ns.profile.duel
	-- only while the card goes out: a hub's settled card carries the top its rivals settle on
	if not (d and Social.Sharing()) then return end
	local now, last = gameDay(), d.start + C.duel_days - 1
	if now <= last then return end
	if #d.rivals == 0 then return endDuel("social.duel.empty") end
	local own = duelScore(d, now)
	local names, parts = CopyTable(d.rivals), { L["social.duel.you"]:format(own) }
	table.sort(names)
	local top = own
	for _, key in ipairs(names) do
		local v = rivalScore(key, d)
		if not (v and v.final and (d.own or v.top)) then
			-- a rival waits a day longer than the hub, whose settled card stays that day too (pastEnds), so a hub that
			-- settles on the grace's last day still reaches them
			if now > last + C.duel_grace_days + (d.own and 0 or 1) then endDuel("social.duel.dropped") end
			return
		end
		top = math.max(top, v.score, d.own and 0 or v.top)
		parts[#parts + 1] = L["social.duel.rival"]:format(key, v.score)
	end
	if not d.own and top > own and top > rivalScore(names[1], d).score then
		parts[#parts + 1] = L["social.duel.best"]:format(top)
	end
	local won, before = own > 0 and own >= top, ns.TitlesEarned(ns.state)
	ns.profile.duelPast = { id = d.id, start = d.start, final = own, top = d.own and top or nil }
	ns.profile.duel, offers.duel = nil, nil
	if won then ns.profile.duelWins = (ns.profile.duelWins or 0) + 1 end
	ns.Touch()
	ns.Print(won and "social.duel.won" or "social.duel.done", table.concat(parts, " · "))
	if d.own then -- the top score the rivals wait for, to those online
		for _, key in ipairs(names) do
			if online(key) then pushTo(key) end
		end
	end
	local live = {}
	for _, id in ipairs(ns.NewTitles(before, ns.state)) do live[#live + 1] = { kind = "title", id = id } end
	if #live > 0 then ns.Emit("Sheet", live) end
	watchDay()
	ns.Emit("Social")
end

-- A rival out of the duel: left it, or their card came without it.
local function dropRival(key)
	local d = ns.profile.duel
	local i = d and tIndexOf(d.rivals, key)
	if not i then return end
	table.remove(d.rivals, i)
	ns.Touch()
	ns.Print("social.duel.left", key)
	if #d.rivals == 0 then return endDuel("social.duel.empty") end
	settleDuel()
	ns.Emit("Social")
end

-- Whether the player may challenge `key` now: invites on, the card shared, a friend online whose card this session
-- stands within duel_band QP levels (the card's level stops at the cap, where a Paragon player stands); no duel, or
-- the player's own still on its first game day with room for one more rival (the challenges standing count).
local function canChallenge(key)
	local profile, p = ns.profile, Social.peers[key]
	if not (profile.invites and Social.Sharing() and p and p.card and not p.private) then return false end
	if Social.hide[key] or ignored(key) then return false end
	if math.abs(p.card.L - ns.Level(ns.state)) > C.duel_band then return false end
	local d, o, now = profile.duel, offers.duel, gameDay()
	if d and (not d.own or d.start ~= now or tIndexOf(d.rivals, key)) then return false end
	local taken = d and #d.rivals or 0
	if o and o.start ~= now then -- a challenge from before 04:00 still standing: a new one would answer its joins V-
		for _, at in pairs(o.to) do
			if waiting(at) then return false end
		end
	end
	if o and o.start == now then
		if waiting(o.to[key]) then return false end
		for _, at in pairs(o.to) do
			if waiting(at) then taken = taken + 1 end
		end
	end
	return taken < C.duel_max and friend(key) and online(key)
end

-- A challenge: the duel starts this game day; it begins for the player when the first rival joins.
function Social.Challenge(key)
	if not canChallenge(key) then return end
	local d, o, now = ns.profile.duel, offers.duel, gameDay()
	if not (o and o.start == now and (not d or o.id == d.id)) then
		o = { id = d and d.id or drawId(), start = now, to = {} }
		offers.duel = o
	end
	o.to[key] = GetTime()
	send(("V?%d.%d"):format(o.id, o.start), "WHISPER", key, "duel:" .. key)
	ns.Print("social.duel.sent", key)
end

-- A challenge's day: the challenger's game day, which may be a day before or after the player's (each day turns at
-- the player's own 04:00).
local function openDay(start)
	return math.abs(start - gameDay()) <= 1
end

-- The popup's "Join": the duel, as "Swear it" takes the pact (a challenge of mine to them stays, onDuelJoin).
local function acceptDuel(o)
	if not (Social.Sharing() and hears(o.key) and online(o.key)) or ns.profile.duel or not openDay(o.start) then return end
	send(("V+%d"):format(o.id), "WHISPER", o.key, "duel:" .. o.key)
	ns.profile.duel = { id = o.id, start = o.start, rivals = { o.key } }
	ns.Print("social.duel.accepted", o.key)
	joined(o.key)
end

-- A friend joined my duel: a rival, on its first game day while there is room; else "V-" (it is over for them). Two
-- friends who challenged each other and both joined hold two ids, as a crossed pact does: the smaller stands, the one
-- whose own challenge has it owns the duel, and the other stays in it.
local function onDuelJoin(key, id)
	local d, o = ns.profile.duel, offers.duel
	-- a challenge still standing, the day it was made or, an answer that crossed the challenger's 04:00, the next
	local ours = o and o.id == id and waiting(o.to[key]) and gameDay() - o.start <= 1
	if d and tIndexOf(d.rivals, key) and (d.id == id or (ours and not d.own and #d.rivals == 1)) then
		if d.id ~= id and id < d.id then -- the crossing, my id the smaller
			d.id, d.start, d.own, o.to[key] = id, o.start, true, nil
			ns.Touch()
			pushStakes()
			ns.Emit("Social")
		end
		return
	end
	if not (ours and (not d or (d.own and d.id == id and #d.rivals < C.duel_max))) then
		return send(("V-%d"):format(id), "WHISPER", key, "duel:" .. key)
	end
	o.to[key] = nil
	if not d then
		d = { id = id, start = o.start, own = true, rivals = {} }
		ns.profile.duel = d
	end
	d.rivals[#d.rivals + 1] = key
	ns.Print("social.duel.joined", key)
	joined(key)
end

local function onDuelLeave(key, id)
	local d = ns.profile.duel
	if d and d.id == id then dropRival(key) end
end

-- "Leave the duel" (the HUD line's menu): it costs nothing; the rivals online are told, the others by my next card.
function Social.LeaveDuel()
	local d = ns.profile and ns.profile.duel
	if not d then return end
	for _, key in ipairs(d.rivals) do
		if online(key) then send(("V-%d"):format(d.id), "WHISPER", key, "duel:" .. key) end
	end
	endDuel("social.duel.youLeft")
end

-- A partner's or a rival's card (or their "unchanged" for the card held): their side of the pact today and their duel
-- score. A friend's card with the pact signed for that friend's key (pactSign: only the pair hold the id) speaks for
-- the partner, on another character; the sworn name stays; the partner's own card without it ends the pact (they
-- ended it or swore another). A rival's card without the duel's id takes them out of it; a duel's id is all its
-- rivals', so a rival is known by the name they joined with only. A card sent before the pact or the duel began this
-- session may still lack its id.
function readStake(key, c)
	local p, profile = Social.peers[key], ns.profile
	if not (p and c) then return end
	local settled = not (joinedAt[key] and GetTime() - joinedAt[key] < C.probe_s)
	local pact = profile.pact
	if pact and c.pact and c.pact.id == pactSign(pact.id, key) and (key == pact.with or friend(key)) then
		pactSeen = { id = pact.id, key = key, day = gameDay(), done = c.pact.done }
		present[key] = online(key) or nil
	elseif pact and pact.with == key and settled then
		endPact("social.pact.over")
	end
	local d = profile.duel
	if d and tIndexOf(d.rivals, key) then
		local v = c.duels[d.id]
		if v then
			p.duel = { id = d.id, score = v.score, final = v.final, top = v.top }
			settleDuel()
		elseif settled then
			dropRival(key)
		end
	end
end

-- The layer listens again (listen) or the card is shared again (Social.SetShare): the partner and the rivals online
-- are asked for their cards (a look taken while the layer was quiet asked no one, and their own asks were not heard),
-- my card goes to them whatever went before, and a duel over is settled now that its top can go out.
function Social.ResumeStakes()
	if not (ns.profile and Social.Sharing()) then return end
	for _, key in ipairs(partners()) do
		local now = online(key)
		present[key] = now or nil
		if now then ask(key) end
	end
	stakeSent = nil
	settleDuel()
	pushStakes()
end

-- Who of the partner and the rivals is online, looked at again when the friends lists change (in an instance the
-- layer is off in too, so the look stays true): one who came online is asked for their card once, while the layer
-- listens.
local function followUp()
	if not ns.profile then return end
	local changed = false
	for _, key in ipairs(partners()) do
		local now = online(key)
		if now and not present[key] and Social.Listening() then ask(key) end
		if now ~= (present[key] == true) then changed = true end
		present[key] = now or nil
	end
	if changed then ns.Emit("Social") end
end

-- The game day at login and at its turn: a settled duel's final score leaves the card after its grace, a duel over is
-- settled or dropped, my card goes to the partner and the rivals online if its fields changed, and theirs are asked
-- for (their day turned too). A duel still unsettled once its last rival's grace passed (pastEnds: nobody settles it
-- later) is dropped with its rivals' names whether or not the card is shared: settleDuel, which drops it sooner, runs
-- only while the card goes out (2026-10-31, the release's security review: a card kept private kept the names for
-- good); nothing is sent.
function lookAtDay()
	local profile = ns.profile
	if not profile then return end
	local now, past, d = gameDay(), profile.duelPast, profile.duel
	if past and now > pastEnds(past) then
		profile.duelPast = nil
		ns.Touch()
	end
	if d and now > pastEnds(d) then endDuel("social.duel.dropped") end
	settleDuel()
	pushStakes()
	for _, key in ipairs(partners()) do
		if online(key) then ask(key) end
	end
	watchDay()
	ns.Emit("Social")
end

-- An offer received: QuestPump's own dialog (Popups.lua, the kit's small tier): the offer, the pact's plan on its own
-- line, the grey "Not now" at the left and the red accept at the right, named for what it does. Not now, Escape and the
-- timeout send nothing (no one is told they were turned down): only OnAccept acts. A second offer while one shows is
-- not shown (popupFree), and a pump on its way takes the dialog away (below): the offer lapses for its sender, as one
-- never shown.
ns.Dialog.Register("QUESTPUMP_PACT", {
	text = L["social.pact.offer"],
	name = function(data) return PLAN_TEXT[data.plan] end,
	button1 = L["social.pact.swear"],
	button2 = L["social.invite.later"],
	OnAccept = function(_, data) acceptPact(data) end,
	timeout = C.invite_s,
})

ns.Dialog.Register("QUESTPUMP_DUEL", {
	text = L["social.duel.offer"],
	button1 = L["social.duel.accept"],
	button2 = L["social.invite.later"],
	OnAccept = function(_, data) acceptDuel(data) end,
	timeout = C.invite_s,
})

-- Receiving --------------------------------------------------------------------------------------------------------

-- A number of at most `digits` digits at the start of `s`, where a later version may add fields after it.
local function leading(s, digits)
	local n = s and s:match("^(%d+)")
	return n and #n <= digits and tonumber(n) or nil
end

-- An offer's popup shows only while no pump is on its way or on the screen (the pump asks at once, and any popup up
-- would hold it: Lc.Unsafe) and none of its kind shows (a second offer never brings it back with a fresh timeout); an
-- offer not shown lapses for its sender, who may make it again. A pump that comes while one shows takes it away.
local function popupFree(which)
	return Lc.pump == nil and not ns.Dialog.Visible(which)
end

ns.On("State", function(p)
	if p and not p.opened then
		ns.Dialog.Hide("QUESTPUMP_PACT")
		ns.Dialog.Hide("QUESTPUMP_DUEL")
	end
end)

-- A pact's or a duel's message, by whisper or game data only: an offer ("P?<id>.<plan>", "V?<id>.<start day>"), its
-- acceptance ("P+<id>", "V+<id>") or an end ("P-<sign>", the sender's pactSign, "V-<id>"). Offers and acceptances
-- count from friends only (an end only ever matches the partner or a rival); an offer shows its popup only while the
-- player has none of its kind and hears offers (hears); a decline sends nothing.
local function onStake(key, text, channel)
	if channel ~= "WHISPER" then return end
	local what, sign, tail = text:match("^([PV])([%?%+%-])(.*)$")
	-- a pact's id, or its sign on an end; a duel's id
	local id = what == "P" and hexOf(tail:match("^%x+"), sign == "-" and PACT_SIGN or PACT_ID)
		or what == "V" and leading(tail, 9)
	if not id or (sign ~= "-" and not friend(key)) then return end
	local p = peer(key)
	if not p.known then known(p) end
	local rest = tail:match("^%x+%.(.*)$")
	if what == "P" and sign == "?" then
		local plan = leading(rest, 1)
		if PLANS[plan] and not ns.profile.pact and hears(key) and popupFree("QUESTPUMP_PACT") then
			ns.Dialog.Show("QUESTPUMP_PACT", key, nil, { key = key, id = id, plan = plan })
		end
	elseif what == "P" then
		if sign == "+" then onPactJoin(key, id) else onPactEnd(key, id) end
	elseif sign == "?" then
		local start = leading(rest, 6)
		if start and openDay(start) and not ns.profile.duel and hears(key) and popupFree("QUESTPUMP_DUEL") then
			ns.Dialog.Show("QUESTPUMP_DUEL", key, C.duel_days, { key = key, id = id, start = start })
		end
	elseif sign == "+" then
		onDuelJoin(key, id)
	else
		onDuelLeave(key, id)
	end
end

local function onMessage(prefix, text, channel, sender)
	if prefix ~= PREFIX then return end
	text, sender = readable(text), readable(sender)
	if not (text and sender) or #text > MAX_BYTES or not Social.Listening() then return end
	local key = Social.Key(sender)
	if key == Social.me then return end
	if ignored(key) then
		-- nothing of theirs is read or kept, but they run QuestPump: the party rule counts them (their messages
		-- reach the player whatever the ignore list says, and the player's never reach them)
		local p = peer(key)
		if not p.known then known(p) end
		return
	end
	local kind, tail = text:sub(1, 1), text:sub(2)
	if kind == "?" then
		local theirs = tail:sub(1, 1) == "C" and leading(tail:sub(2), 11)
		local p = peer(key)
		p.seen = GetTime()
		if not p.known then known(p) end
		if theirs then answer(key, theirs) end
	elseif kind == "C" then
		local ver, rest = text:match("^C(%d+):(.+)$")
		if ver and #ver <= 11 then onCard(key, tonumber(ver), rest, channel) end
	elseif kind == "!" then
		local p, ver = peer(key), tail:sub(1, 1) == "C" and leading(tail:sub(2), 11)
		if ver and p.ver == ver then
			known(p)
			readStake(key, p.card) -- the card held is today's still
		end
	elseif kind == "-" then
		if tail:sub(1, 1) == "C" then
			local p = peer(key)
			-- a later card, whatever its version, is asked for whole; the day's moves go with the card
			p.card, p.ver, p.private, p.today = nil, nil, true, nil
			known(p)
			if GROUP_CHANNEL[channel] then replyToHello(key, channel) end -- a private player's group hello
			ns.Emit("Social", key)
		end
	elseif kind == "S" or kind == "D" or kind == "N" then
		local n = leading(tail, kind == "N" and 6 or 9) -- a pump's id, or the day's moves
		if not n then return end
		local p = peer(key)
		if not p.known then known(p) end
		if kind == "S" then
			onStart(key, n, channel)
		elseif kind == "D" then
			onDone(key, n, channel)
		else
			onMoves(key, n, channel)
		end
	elseif kind == "U" then
		local id = leading(tail, 5)
		if not id then return end
		local p = peer(key)
		if not p.known then known(p) end
		onLegend(key, id, channel)
	elseif kind == "K" and channel == "WHISPER" then
		local code, id = text:match("^K(%d):(%d+)")
		if code and #id <= 9 then onCheer(key, tonumber(code), tonumber(id)) end
	elseif kind == "P" or kind == "V" then
		onStake(key, text, channel)
	end
end

ns.RegisterEvent("CHAT_MSG_ADDON", onMessage)

-- Battle.net game data (slice 23): BN_CHAT_MSG_ADDON(prefix, text, channel, senderID), its channel always "WHISPER"
-- and its sender a game account (ChatInfoDocumentation.lua, 70205), read as a whisper from that account's character,
-- whose answers go back the same way this session (gameAccount).
ns.RegisterEvent("BN_CHAT_MSG_ADDON", function(prefix, text, _, senderID)
	if prefix ~= PREFIX then return end
	local id = readable(senderID)
	local key = id and bnKey(C_BattleNet.GetGameAccountInfoByID(id))
	if not key then return end
	bnRoute[key] = id
	onMessage(prefix, text, "WHISPER", key)
end)

-- What the HUD shows (Sheet.lua's HUD.Lines): the pact and the duel, the party's moves today, the Join line, the latest
-- finished pump to cheer, or the guild's digest. { text, action?, menu?, rightMenu?, tooltip? }.

-- The pact as the HUD's line and the card window's row say it, never who has not paid: today's state ("social.pact.
-- open" while the player's own side is, ".both" once the partner's card this game day says so too, ".unknown" while
-- their card has not come today or the partner is offline with their side open) and the game days since it was sworn,
-- today counted.
local function pactToday(pact)
	local now = gameDay()
	local voice = seen(pact)
	local theirs = voice and voice.day == now and voice or nil
	local state = not pactDone(pact.plan) and "social.pact.open" or theirs and theirs.done and "social.pact.both"
		or theirs and present[theirs.key] and "social.pact.open" or "social.pact.unknown"
	return state, math.max(1, now - pact.since + 1)
end

-- The pact's tooltip in the kit's shape (Kit.Tip): its title, the plan, what done means and that it changes nothing in
-- the notes' grey, and on the HUD's line (`hint`) its right-click last.
local function pactTip(pact, hint)
	local done = pact.plan == E.pact.evening and "social.pact.done.evening" or "social.pact.done.daily"
	return Kit.Tip(L["social.pact.title"], PLAN_TEXT[pact.plan], { facts = { L[done], L["social.pact.free"] },
		hint = hint and L["social.pact.endTip"] or nil })
end

-- The pact's line, "Pact with Jaina: 12 days · both done today".
local function pactLine(out)
	local pact = ns.profile.pact
	if not pact then return end
	local state, days = pactToday(pact)
	out[#out + 1] = { text = ns.Plural("social.pact.hud", days):format(pact.with, days, L[state]),
		rightMenu = function(_, root) root:CreateButton(L["social.pact.end"], Social.EndPact) end,
		tooltip = pactTip(pact, true) }
end

-- The duel as the HUD's line and the card window's row say it: the player's score, its day (the last one once the duel
-- is over), the rivals in name order, and whether its last game day is behind.
local function duelToday(d)
	local now = gameDay()
	local names = CopyTable(d.rivals)
	table.sort(names)
	return duelScore(d, now), math.min(math.max(1, now - d.start + 1), C.duel_days), names,
		now > d.start + C.duel_days - 1
end

-- The duel's tooltip in the kit's shape: its title, its day, in the notes' grey the other rivals' scores where its row
-- shows one rival's alone (`shown`, the card window's: "Also in the duel: Jaina 165 · Thrall 120"), the rivals with no
-- score this session and its rules, and on the HUD's line (`hint`) its right-click last.
local function duelTip(d, day, names, hint, shown)
	local facts, others = {}, {}
	for _, key in ipairs(names) do
		local v = rivalScore(key, d)
		if not v then
			facts[#facts + 1] = L["social.duel.tip.unknown"]:format(key)
		elseif shown and key ~= shown then
			others[#others + 1] = L["social.duel.rival"]:format(key, v.score)
		end
	end
	if #others > 0 then table.insert(facts, 1, L["social.card.duelOthers"]:format(table.concat(others, " · "))) end
	facts[#facts + 1] = L["social.duel.tip.rules"]:format(C.duel_pump_pts, C.duel_pumps_day, C.duel_started_day,
		C.duel_active_pts, C.duel_cardio_pts, C.duel_cardio_min_day)
	return Kit.Tip(L["social.duel.title"], L["social.duel.tip.day"]:format(day, C.duel_days), { facts = facts,
		hint = hint and L["social.duel.leaveTip"] or nil })
end

-- The duel's line, "Duel day 4: you 180 · Jaina 165": the rivals whose card this session holds, the others in its
-- tooltip as unknown, never zero; after the last day, the wait for the final scores.
local function duelLine(out)
	local d = ns.profile.duel
	if not d then return end
	local own, day, names, over = duelToday(d)
	local parts = { over and L["social.duel.over"]:format(own) or L["social.duel.day"]:format(day, own) }
	for _, key in ipairs(names) do
		local v = rivalScore(key, d)
		if v then parts[#parts + 1] = L["social.duel.rival"]:format(key, v.score) end
	end
	if over then parts[#parts + 1] = L["social.duel.waiting"] end
	out[#out + 1] = { text = table.concat(parts, " · "),
		rightMenu = function(_, root) root:CreateButton(L["social.duel.leave"], Social.LeaveDuel) end,
		tooltip = duelTip(d, day, names, true) }
end

-- The party's line, "Party today: you 14 · Jaina 9 · Thrall 21": the player's own moves this game day first, then the
-- moves of each member whose count came this game day (their card, their pump's end), by name and never ranked; a
-- member not heard from today or private is left out, never shown as zero, and so is one Social.Shown hides (muted,
-- ignored, or others' cards switched off). Not in a raid, where no pump's end is told; nothing with no member's count,
-- where the line would only repeat the Today tab.
local function partyLine(out)
	if not IsInGroup() or IsInRaid() then return end
	local now, names = gameDay(), {}
	for key in pairs(Social.GroupKeys()) do
		local t = Social.peers[key] and Social.peers[key].today
		if t and t.day == now and Social.Shown(key) then names[#names + 1] = key end
	end
	if #names == 0 then return end
	table.sort(names)
	local parts = { L["social.hud.party"]:format(myMoves()) }
	for _, key in ipairs(names) do
		parts[#parts + 1] = L["social.hud.party.member"]:format(key, Social.peers[key].today.moves)
	end
	out[#out + 1] = { text = table.concat(parts, " · "),
		tooltip = Kit.Tip(L["social.hud.party.title"], L["social.hud.party.tip"]) }
end

local function cheerMenu(key, id)
	return function(_, root)
		root:CreateTitle(L["social.cheer.title"]:format(key))
		for code, name in ipairs(CHEERS) do
			root:CreateButton(L["cheer." .. name], function() Social.Cheer(key, id, code) end)
		end
	end
end

function Social.HudLines()
	local out = {}
	if not Social.Listening() then return out end
	pactLine(out)
	duelLine(out)
	partyLine(out)
	if not ns.profile.cheers then return out end
	if invite and not Lc.pump and not Lc.Locked() then
		out[#out + 1] = { text = L["social.hud.join"]:format(invite.from), action = Lc.Demand, glow = true }
	end
	local cheered = finished and Social.peers[finished.from] and Social.peers[finished.from].cheered[finished.id]
	if finished and not cheered and Social.Sharing() then
		out[#out + 1] = { text = L["social.hud.finished"]:format(finished.from), menu = cheerMenu(finished.from,
			finished.id) }
	elseif digest and digest.day == ns.day() then
		out[#out + 1] = { text = ns.Plural("social.hud.digest", digest.pumps):format(digest.pumps, digest.n) }
	end
	return out
end

-- The guild's week ----------------------------------------------------------------------------------------------------

-- The guildmates seen this session with their cards, sorted by streak (never a typed number), with the week's pumps
-- and cardio minutes of all of them and the player, the player's own share and the highest milestone reached.
function Social.GuildWeek()
	local rows, players = {}, 1
	local own = Social.Mine()
	local pumps, cardio = own.W or 0, own.M or 0
	for key, p in pairs(Social.peers) do
		if p.card and guildmate(key, p) then
			if not Social.mute[key] then rows[#rows + 1] = { key = key, card = p.card } end
			pumps, cardio = pumps + (p.card.W or 0), cardio + (p.card.M or 0)
			players = players + 1
		end
	end
	table.sort(rows, function(a, b)
		if (a.card.S or 0) ~= (b.card.S or 0) then return (a.card.S or 0) > (b.card.S or 0) end
		return a.key < b.key
	end)
	local milestone
	for _, at in ipairs(C.guild_milestones) do
		if pumps >= at then milestone = at end
	end
	return { rows = rows, pumps = pumps, cardio = cardio, share = own.W or 0, players = players, milestone = milestone }
end

-- The weekly guild line, once per reset week: the text the player sends, or nil once it was offered this week.
function Social.GuildLine()
	if not (IsInGuild() and ns.profile) or ns.profile.guildLineWeek == ns.WeekOf(ns.now()) then return nil end
	local week = Social.GuildWeek()
	return L["social.guild.line"]:format(week.pumps, week.players) .. " " .. Social.TOKEN
end

function Social.SendGuildLine()
	local line = Social.GuildLine()
	if not line then return end
	ns.profile.guildLineWeek = ns.WeekOf(ns.now())
	ns.Touch()
	ChatFrameUtil.OpenChat(SLASH_GUILD1 .. " " .. line)
	ns.Emit("Social")
end

-- The tooltip line ------------------------------------------------------------------------------------------------

-- The card's two lines, "QuestPump 12 · the Relentless" and "9-day streak · 34 pumps this week · 5 records beaten",
-- or "QuestPump (private)"; nil for a player without a card.
function Social.CardLines(p)
	if p.private then return { L["social.tip.private"] } end
	local c = p.card
	if not c then return nil end
	local head = c.P and c.P > 0 and L["social.tip.paragon"]:format(c.L, c.P) or L["social.tip.level"]:format(c.L)
	if c.title then head = head .. " · " .. L["title." .. c.title] end
	local parts = {}
	if (c.S or 0) > 0 then parts[#parts + 1] = ns.Plural("social.tip.streak", c.S):format(c.S) end
	if (c.W or 0) > 0 then parts[#parts + 1] = ns.Plural("social.tip.pumps", c.W):format(c.W) end
	if (c.R or 0) > 0 then parts[#parts + 1] = ns.Plural("social.tip.records", c.R):format(c.R) end
	local lines = { head }
	if #parts > 0 then lines[#lines + 1] = table.concat(parts, " · ") end
	if c.F == 1 then lines[#lines + 1] = L["social.tip.pumping"] end
	return lines
end

-- What the unit tooltip shows: its player's key and GUID, and whether the card's lines are on it, so a card that comes
-- while it shows joins it once.
local tip = {}

local function addLines(tooltip, key, p)
	local lines = p and Social.CardLines(p)
	if not lines then return false end
	local gold = NORMAL_FONT_COLOR
	for i, line in ipairs(lines) do
		if i == 1 then tooltip:AddLine(line, gold.r, gold.g, gold.b) else tooltip:AddLine(line, 1, 1, 1) end
	end
	if p.card and guildmate(key, p) then
		local week = Social.GuildWeek()
		if week.milestone then tooltip:AddLine(L["social.tip.milestone"]:format(week.milestone), 0.6, 0.8, 1) end
	end
	return true
end

-- Whether a surface beside a player may show what the session knows of them (the tooltip, the share line's link, the
-- badges, the nameplate line, the chat mark: Badges.lua): while the layer listens and the player shows others' cards,
-- never for a player muted or ignored, the player's own only while they share it. Its second value is what is known:
-- the peer, or the player's own card in the peer's shape.
function Social.Shown(key)
	if not (key and Social.Listening() and ns.profile.showCards) or Social.mute[key] or ignored(key) then return false end
	if key == Social.me then
		if not Social.Sharing() then return false end
		return true, { card = Social.Mine(), known = true }
	end
	return true, Social.peers[key]
end

-- The unit tooltip's post-call, from its data's GUID (never GetUnit, whose helper tests values that may be secret).
local function onUnitTooltip(tooltip, data)
	tip.key, tip.guid, tip.drawn = nil, nil, false
	if tooltip ~= GameTooltip then return end
	local guid = readable(data and data.guid)
	local unit = guid and readable(UnitTokenFromGUID(guid))
	local key = unit and Social.UnitKey(unit)
	local shown, p = Social.Shown(key)
	if not shown then return end
	tip.key, tip.guid = key, guid
	tip.drawn = addLines(tooltip, key, p)
	if key ~= Social.me then askUnit(unit) end
end

-- A card that came while its player's tooltip shows without one: its lines join the tooltip (never a RefreshData, which
-- would run the unit tooltip's rebuild from addon code); a card already drawn waits for the next hover.
ns.On("Social", function(key)
	if not key or key ~= tip.key or tip.drawn or not GameTooltip:IsShown() then return end
	local data = GameTooltip:GetPrimaryTooltipData()
	if readable(data and data.guid) ~= tip.guid then return end
	tip.drawn = addLines(GameTooltip, key, Social.peers[key])
	if tip.drawn then GameTooltip:Show() end
end)

-- A player's QuestPump choices ---------------------------------------------------------------------------------------

-- What QuestPump offers for a player, the entries the right-click menus carried until 2026-10-05, under their name:
-- their card, mute and hide while the player shows others' cards; a pact's plans and a duel's challenge while they may
-- be offered (canPact, canChallenge). In QuestPump's own menu (Popups.lua), from /qp player and the card window's
-- More, never in Blizzard's menus: an entry added there broke their protected entries. The card window's own
-- (`inCard`) leaves out the card and the offers (its red button makes them) and ends with the endings: the pact with
-- them, the duel they are in.
local function playerMenu(key, atHand, inCard)
	return function(_, root)
		if not key or key == Social.me or not Social.Listening() then return end
		root:CreateTitle(key)
		if ns.profile.showCards then
			if not inCard then root:CreateButton(L["social.menu.card"], function() Social.OpenCard(key, atHand) end) end
			root:CreateButton(L[Social.mute[key] and "social.menu.unmute" or "social.menu.mute"], function()
				Social.mute[key] = not Social.mute[key] or nil
				ns.Emit("Social")
			end)
			root:CreateButton(L[Social.hide[key] and "social.menu.unhide" or "social.menu.hide"], function()
				Social.hide[key] = not Social.hide[key] or nil
				ns.Emit("Social")
			end)
		end
		if inCard then
			local pact, d = Social.PactWith(key), ns.profile.duel
			local inDuel = d and tIndexOf(d.rivals, key) ~= nil
			if pact or inDuel then root:CreateDivider() end
			if pact then root:CreateButton(L["social.pact.end"], Social.EndPact) end
			if inDuel then root:CreateButton(L["social.duel.leave"], Social.LeaveDuel) end
			return
		end
		if canPact(key) then
			local plans = root:CreateButton(L["social.menu.pact"])
			for plan, name in ipairs(PLAN_TEXT) do
				plans:CreateButton(name, function() Social.OfferPact(key, plan) end)
			end
		end
		if canChallenge(key) then root:CreateButton(L["social.menu.duel"], function() Social.Challenge(key) end) end
	end
end

-- The pact the player holds with `key` (its partner, or the partner's character its card last came from), else nil.
function Social.PactWith(key)
	local pact = ns.profile.pact
	if not pact then return nil end
	local voice = seen(pact)
	return (pact.with == key or (voice and voice.key == key)) and pact or nil
end

-- The card window: each player's own card, then what the two share --------------------------------------------------

local window

-- A card's counted numbers as the card window's and the Inspect pane's stat rows, { number, words } (the kit's inline
-- stats): its streak, pumps, records, cardio minutes and active days; its level on the medallion's plate, its title
-- under its name, a pump on now as the clock. Never one player's typed count beside the other's: those are the typed
-- lines.
local function cardRows(c)
	c = c or {}
	return {
		{ c.S or 0, L["social.card.streak"] },
		{ c.W or 0, L["social.card.pumps"] },
		{ c.R or 0, L["social.card.records"] },
		{ c.M or 0, L["social.card.cardio"] },
		{ c.A or 0, L["social.card.active"]:format(C.rating_days) },
	}
end
Social.CardRows = cardRows

-- A side's own typed progress, "Thrall · Push: wall push-ups 8 → 23 since 1 Nov", from the records its owner shares
-- (the day number's middle, as the sheet dates a record), its owner's name in white.
local function typedLines(name, c, out)
	for _, r in ipairs(c and c.typed or {}) do
		local hold = ns.Library[r.move].dose == "hold"
		local line = L["social.card.typedLine"]:format(L["ladder." .. r.ladder], L["move." .. ns.Library[r.move].key],
			Sheet.Amount(r.first, hold), Sheet.Amount(r.best, hold), Sheet.ShortDate(r.day * 86400 + 43200))
		out[#out + 1] = L["social.card.typedBy"]:format(HIGHLIGHT_FONT_COLOR:WrapTextInColorCode(name), line)
	end
	return out
end

-- The window's measures: a card's least height, its stats' first row and pitch, its medallion's room at its left, a
-- duel score cell's least width.
local CARD_LEAST, STAT_TOP, STAT_PITCH, MEDAL_ROOM, SCORE_LEAST = 204, 72, 24, 72, 72

-- One player's card on the window: the medallion (Badges.CardBorders), the name and title, the clock while that player
-- pumps (its tooltip says so), the five stat rows.
local function buildSide(view, w)
	local card = Kit.Card(view, w, CARD_LEAST)
	card.name = Kit.Text(card, "cardTitle", "LEFT", nil, w - MEDAL_ROOM - 40)
	card.name:SetPoint("TOPLEFT", MEDAL_ROOM, -14)
	card.name:SetWordWrap(false)
	Kit.WholeWords(card.name) -- a long name, cut, whole in its tooltip
	card.title = Kit.Text(card, "bodyWhite", "LEFT", Style.saga.note, w - MEDAL_ROOM - 12)
	card.title:SetPoint("TOPLEFT", card.name, "BOTTOMLEFT", 0, -4)
	card.clock = CreateFrame("Frame", nil, card)
	card.clock:SetSize(20, 20)
	card.clock:SetPoint("TOPRIGHT", -14, -14)
	card.clock.art = card.clock:CreateTexture(nil, "ARTWORK")
	card.clock.art:SetPoint("CENTER")
	Kit.Hover(card.clock, Kit.Tip(L["social.card.pumping"]))
	Kit.Choose(card, false, true) -- at rest: its edge, the name white
	card.stats = {}
	for i = 1, #cardRows() do
		local st = Kit.Stat(card, { inline = true })
		st:SetSize(w - 24, STAT_PITCH)
		st:SetPoint("TOPLEFT", 12, -(STAT_TOP + (i - 1) * STAT_PITCH))
		card.stats[i] = st
	end
	return card
end

-- What the two share, rows of a framed list: the pact (its plan, how long it holds, today's state) and the duel (its
-- day, its rule, the player's score and this player's, the other rivals' in its tooltip), each with its HUD line's
-- tooltip.
local function buildTogether(f, view)
	local M = Kit.TIER.M
	f.togetherLabel = Kit.Section(view, "")
	f.together = Kit.List(view, M.TEXT, M.rowName)
	local state = CreateFrame("Frame", nil, f.together)
	state:SetHeight(24)
	state.mark = state:CreateTexture(nil, "ARTWORK")
	state.mark:SetSize(18, 18)
	state.mark:SetPoint("LEFT")
	state.words = Kit.Text(state, "large", "LEFT")
	f.pactRow = Kit.Row(f.together, { atlas = "pact", name = "", below = true, line = "", control = state,
		tip = function(tooltip) if f.pactTip then f.pactTip(tooltip) end end })
	f.pactRow.state = state
	local cells = CreateFrame("Frame", nil, f.together)
	cells:SetHeight(44)
	cells.list = {}
	f.duelRow = Kit.Row(f.together, { atlas = "duel", name = "", below = true, line = "", control = cells,
		tip = function(tooltip) if f.duelTip then f.duelTip(tooltip) end end })
	f.duelRow.cells = cells
	for _, row in ipairs({ f.pactRow, f.duelRow }) do Kit.Choose(row, true, true) end
end

local function build()
	local M = Kit.TIER.M
	local f = Kit.Window("QuestPumpCard", "M", L["social.card.title"], { escape = "special", y = 40 })
	local view = f.body
	Kit.Head(view, "", nil, "M")
	local w = Kit.Grid(2, M.TEXT)
	f.cards = { buildSide(view, w), buildSide(view, w) }
	f.medals = ns.Badges.CardMedals(f.cards)
	f.typedLabel = Kit.Section(view, L["social.card.typed"])
	f.typedScroll = Kit.Scroll(view, M.TEXT, 1)
	f.typed = Kit.Text(f.typedScroll.content, "bodyWhite", "LEFT", Style.saga.note, M.TEXT)
	f.typed:SetPoint("TOPLEFT")
	buildTogether(f, view)
	f.foot = Kit.Footer(f)
	f.more = Kit.Caret(f.foot:Secondary(L["ui.more"], function(self)
		Kit.OpenMenu(self, playerMenu(f.key, true, true), true)
	end))
	f.offer = f.foot:Primary("", function(self) self.act(self) end, "tab")
	window = f
	return f
end

-- The line under a player's name (the card window's intro, the Inspect window's pane: Badges.lua): a private answer,
-- the addon that reported it, an ask still on its way, or none this session.
function Social.CardStatus(key, p, theirs)
	local asking = p.asked and not p.answered and GetTime() - p.asked < C.probe_s
	return p.private and L["social.tip.private"] or theirs and L["social.card.reported"]:format(key)
		or asking and L["social.card.asking"] or L["social.card.none"]
end

-- One side's card: the name, and with a card its title, the clock while that player pumps and its stat rows; a side
-- without one (private, or none this session) the name alone, the intro saying why.
local function renderSide(card, name, c)
	card.name:SetText(name or "")
	card.title:SetText(c and c.title and L["title." .. c.title] or "")
	local pumping = c ~= nil and c.F == 1 and Style.FitAtlas(card.clock.art, "clock", 20)
	card.clock:SetShown(pumping == true)
	for i, r in ipairs(cardRows(c)) do
		card.stats[i]:SetShown(c ~= nil)
		Kit.FillStat(card.stats[i], BreakUpLargeNumbers(r[1]), r[2])
	end
end

-- A word of the HUD's line as a row's state, its first letter in capitals ("Both done today").
local function sentence(words)
	return (words:gsub("^%l", string.upper))
end

-- What the player and `key` share: the pact's row and the duel's, each shown while it holds; answers whether any.
local function renderTogether(f, key)
	local pact, d = Social.PactWith(key), ns.profile.duel
	if d and not tIndexOf(d.rivals, key) then d = nil end
	f.pactRow:SetShown(pact ~= nil)
	f.duelRow:SetShown(d ~= nil)
	f.pactTip, f.duelTip = nil, nil
	if pact then
		local state, days = pactToday(pact)
		local row, mark = f.pactRow, f.pactRow.state
		row.name:SetText(PLAN_TEXT[pact.plan])
		row.line:SetText(ns.Plural("social.card.pactLine", days):format(days))
		local both = state == "social.pact.both"
		mark.mark:SetShown(both)
		if both then Style.SetMark(mark.mark, true) end
		mark.words:SetText(sentence(L[state]))
		mark.words:SetTextColor(unpack(both and { GREEN_FONT_COLOR:GetRGB() } or Style.saga.note))
		mark.words:ClearAllPoints()
		mark.words:SetPoint("LEFT", both and 24 or 0, 0)
		mark:SetWidth((both and 24 or 0) + (ns.Readable(mark.words:GetStringWidth()) or 100))
		f.pactTip = pactTip(pact)
	end
	if d then
		local own, day, names, over = duelToday(d)
		local row, cells = f.duelRow, f.duelRow.cells
		row.name:SetText(over and L["social.duel.over"]:format(own) or L["social.card.duel"]:format(day, C.duel_days))
		-- under its name only once it is over, the wait for the rivals' last scores (its day and length are in its name)
		row.line:SetText(over and sentence(L["social.duel.waiting"]) or "")
		row.line:SetShown(over)
		-- two cells at most, the player's (none once the duel is over: its name says it) and this player's, so the row
		-- holds whatever the duel's size (duel_max); the other rivals' scores go in its tooltip (duelTip's `shown`)
		local v = rivalScore(key, d)
		local scores = over and {} or { { own, L["social.card.you"] } }
		scores[#scores + 1] = { v and v.score or L["mark.missed"], key }
		-- each cell as wide as its words and its number at full size need (the board's 106 for two), measured before
		-- Kit.FillStat fits the number, so a score never changes size from one draw to the next; the rule between two
		local x = 0
		for i, s in ipairs(scores) do
			local cell = cells.list[i] or Kit.Stat(cells, { rule = true })
			cells.list[i] = cell
			local score = type(s[1]) == "number" and BreakUpLargeNumbers(s[1]) or s[1]
			cell.number:SetTextScale(1)
			cell.number:SetText(score)
			cell.word:SetText(s[2])
			local w = math.max(SCORE_LEAST, (ns.Readable(cell.number:GetUnboundedStringWidth()) or 30) + 5
				+ (ns.Readable(cell.word:GetUnboundedStringWidth()) or 30) + 24)
			cell:SetSize(w, 44)
			cell:ClearAllPoints()
			cell:SetPoint("TOPLEFT", x, 0)
			cell:Show()
			Kit.FillStat(cell, score, s[2], true)
			x = x + w
		end
		for i = #scores + 1, #cells.list do cells.list[i]:Hide() end
		cells:SetWidth(math.max(1, x))
		f.duelTip = duelTip(d, day, names, nil, key)
	end
	return pact ~= nil or d ~= nil
end

-- The window's red button, only while something can be offered: "Swear a pact" with its two plans in QuestPump's menu
-- while a pact may be sworn with them, else "Challenge to a duel" while a duel is free; none else.
local function renderOffer(f, key)
	local b = f.offer
	local pact = key ~= Social.me and canPact(key)
	local duel = key ~= Social.me and not pact and canChallenge(key)
	b:SetShown(pact or duel)
	if not (pact or duel) then return end
	b:SetText(L[pact and "social.menu.pact" or "social.menu.duel"])
	b:SetWidth(math.max(Kit.TIER.M.PRIMARY, b:GetTextWidth() + 40))
	b.act = pact and function(self)
		Kit.OpenMenu(self, function(_, root)
			for plan, name in ipairs(PLAN_TEXT) do
				root:CreateButton(name, function()
					Social.OfferPact(key, plan)
					Social.RenderCard()
				end)
			end
		end, true)
	end or function()
		Social.Challenge(key)
		Social.RenderCard()
	end
end

-- The window laid out from its parts' heights (Kit.Stack): the head, the two cards, the typed lines (only while a
-- player shares records; past the medium window's cap they scroll in the room left), what the two share; its height
-- by Kit.Fit.
function Social.RenderCard()
	local f = window
	if not (f and f:IsShown() and f.key) then return end
	local M, view = Kit.TIER.M, f.body
	local p = Social.peers[f.key] or {}
	local mineCard = Social.Mine()
	local theirs = not p.private and p.card or nil
	view.title:SetText(f.key)
	view.intro:SetText(Social.CardStatus(f.key, p, theirs))
	renderSide(f.cards[1], Social.me, mineCard)
	renderSide(f.cards[2], f.key, theirs)
	ns.Badges.CardBorders(f, mineCard, theirs)
	local cardH = CARD_LEAST
	for _, card in ipairs(f.cards) do
		local h = 14 + (ns.Readable(card.name:GetStringHeight()) or 18) + 4
			+ (ns.Readable(card.title:GetStringHeight()) or 0)
		cardH = math.max(cardH, math.max(STAT_TOP, h + 12) + #cardRows() * STAT_PITCH + 12)
	end
	for _, card in ipairs(f.cards) do card:SetHeight(cardH) end
	local lines = typedLines(Social.me or "", mineCard, {})
	typedLines(f.key, theirs, lines)
	f.typed:SetText(table.concat(lines, "\n"))
	local typedH = #lines > 0 and (ns.Readable(f.typed:GetStringHeight()) or 0) or 0
	f.typedScroll.content:SetHeight(math.max(1, typedH))
	Kit.SetCaps(f.togetherLabel.words, L["social.card.together"]:format(f.key))
	local shared = renderTogether(f, f.key)
	local listH = shared and Kit.LayList(f.together) or 0
	f.more:SetShown(f.key ~= Social.me)
	renderOffer(f, f.key)
	-- the typed block's room: the window's cap (or the screen less its dim's edge) less everything else
	local top = Kit.Top(view)
	local w, gap = Kit.Grid(2, M.TEXT)
	local rest = top + cardH + (shared and 16 + 12 + 14 + listH or 0) + 20 + f.foot:Lay()
	local limit = math.min(M.cap, (ns.Readable(UIParent:GetHeight()) or 768) - 2 * M.edge)
	local room = limit - rest - (typedH > 0 and 16 + 12 + 14 or 0)
	f.typedScroll:SetHeight(math.max(1, math.min(typedH, math.max(40, room))))
	local bottom = Kit.Stack(view, top, {
		{ f.cards[1], 0 },
		{ f.typedLabel, 16, show = typedH > 0 },
		{ f.typedScroll, 14, show = typedH > 0 },
		{ f.togetherLabel, 16, show = shared },
		{ f.together, 14, show = shared },
	})
	Kit.At(f.cards[2], view, top, w + gap)
	Kit.Fit(f, bottom, f.foot)
end

-- Opens the window for a player (/qp player, a link, the Guild tab), not while the player keeps others' cards off, and
-- asks a player at hand for a card (the Guild tab's and an offline roster entry's show what the session holds); the
-- "Asking" line gives way to "No card" once an answer would have come.
function Social.OpenCard(key, atHand)
	if not (key and Social.Listening() and ns.profile.showCards) then return end
	local f = window or build()
	f.key = key
	if atHand and key ~= Social.me then
		Social.Ask(key)
		T.after("cardStatus", C.probe_s, Social.RenderCard)
	end
	f:Show()
	Social.RenderCard()
end

ns.On("Social", function(key)
	if window and (key == nil or key == window.key) then Social.RenderCard() end
end)
-- a Text Size, UI scale or screen change: the window laid out again while it shows
Kit.OnRelayout(Social.RenderCard)

-- /qp player -------------------------------------------------------------------------------------------------------

-- A typed name as the game spells it, the key a card and its answer travel under: a player seen this session, a
-- character friend, a Battle.net friend's character in this game, matched whatever the case; else the name as typed
-- with each word capitalized, as the game writes names.
local function spelled(name)
	local key = Social.Key(name)
	local lower = key:lower()
	for peerKey in pairs(Social.peers) do
		if peerKey:lower() == lower then return peerKey end
	end
	local info = C_FriendList and C_FriendList.GetFriendInfo and C_FriendList.GetFriendInfo(key)
	if info and info.name then return Social.Key(info.name) end
	if BNGetNumFriends and C_BattleNet and C_BattleNet.GetFriendNumGameAccounts then
		for i = 1, BNGetNumFriends() or 0 do
			for j = 1, C_BattleNet.GetFriendNumGameAccounts(i) or 0 do
				local bn = bnKey(C_BattleNet.GetFriendGameAccountInfo(i, j))
				if bn and bn:lower() == lower then return bn end
			end
		end
	end
	local base, realm = key:match("^([^%-]*)(.*)$")
	return base:lower():gsub("(%a)(%S*)", function(first, rest) return first:upper() .. rest end) .. realm
end

-- /qp player [name]: a player's QuestPump choices (playerMenu) at the cursor, by name or else for the target. A name
-- is asked for a card only while that player can hear it (a friend online, a group member, one heard this session),
-- as the right-click menus asked only a player online.
ns.AddCommand("player", function(arg)
	local name = strtrim(arg or "")
	local key, atHand
	if name == "" then
		key = Social.UnitKey("target")
		atHand = readable(UnitIsConnected("target")) ~= false
	else
		key = spelled(name)
		atHand = online(key) or Social.GroupKeys()[key] ~= nil or Social.peers[key] ~= nil
	end
	if not key or key == Social.me then return ns.Print("cmd.player.usage") end
	if not Social.Listening() then return ns.Print("guild.quiet") end
	if not (ns.profile.showCards or canPact(key) or canChallenge(key)) then return ns.Print("cmd.player.none", key) end
	ns.Menu.Open(nil, playerMenu(key, atHand))
end, "cmd.player.help")

-- Share lines: the sender's "[QuestPump]" becomes a gold link to their card ------------------------------------------

local LINK = "addon:" .. PREFIX .. ":card:"

local function linkFilter(_, _, msg, sender, ...)
	if not (Social.Listening() and ns.profile.showCards) then return false end -- before reading the line
	msg, sender = readable(msg), readable(sender)
	if not (msg and sender) or not msg:find(Social.TOKEN, 1, true) then return false end
	local key = Social.Key(sender)
	if key == Social.me or not Social.Shown(key) then return false end
	local link = NORMAL_FONT_COLOR:WrapTextInColorCode(("|H%s%s|h%s|h"):format(LINK, key, Social.TOKEN))
	local start, stop = msg:find(Social.TOKEN, 1, true)
	return false, msg:sub(1, start - 1) .. link .. msg:sub(stop + 1), sender, ...
end

local function onLink(_, link)
	local key = type(link) == "string" and link:sub(1, #LINK) == LINK and link:sub(#LINK + 1) or nil
	if key and key ~= "" then Social.OpenCard(key, true) end -- one who just spoke
end

-- Lifecycle -------------------------------------------------------------------------------------------------------

-- The prefix at every load (it never survives a /reload); the hooks once. A prefix the client refuses (too many
-- addons' prefixes) leaves the layer deaf, every party mixed: reported, never silent.
local registered = C_ChatInfo.RegisterAddonMessagePrefix(PREFIX)
if registered ~= Enum.RegisterAddonMessagePrefixResult.Success
	and registered ~= Enum.RegisterAddonMessagePrefixResult.DuplicatePrefix then
	ns.Report(("QuestPump: the addon-message prefix was refused (%s)"):format(tostring(registered)))
end
TooltipDataProcessor.AddTooltipPostCall(Enum.TooltipDataType.Unit, onUnitTooltip)
for _, event in ipairs(CHAT_EVENTS) do ChatFrameUtil.AddMessageEventFilter(event, linkFilter) end
EventRegistry:RegisterCallback("SetItemRef", onLink, Social)

-- The guild roster the client holds, asked for once per guild (what `C_GuildInfo.MemberExistsByName` reads); its first
-- update opens the guild's channel and sends a hello that waited for it.
local function askRoster()
	if IsInGuild() and not Social.roster then
		Social.roster = true
		C_GuildInfo.GuildRoster()
		-- one already loaded (another addon asked first, and a second request within the server's wait brings no update)
		if GetNumGuildMembers() > 0 then Social.rosterReady = true end
	end
end

ns.RegisterEvent("GUILD_ROSTER_UPDATE", function()
	if Social.rosterReady or not Social.roster then return end
	Social.rosterReady = true
	if Social.guildWaiting then sendGuildHello() end
end)

-- The player's guild, once the client knows its name: a new one (or none) ends the old one's session, its members
-- no longer guildmates by the channel their card came on, its roster and its hello to be had again.
local function guildChanged()
	local name = IsInGuild() and readable(GetGuildInfo("player")) or nil
	if IsInGuild() and not name then return end -- in a guild the client has not named yet
	if name == Social.guildName then return end
	if Social.guildName then
		for _, p in pairs(Social.peers) do p.guild = nil end
		Social.roster, Social.rosterReady, Social.guildHello, Social.guildWaiting, digest = nil, nil, nil, nil, nil
	end
	Social.guildName = name
end

-- Whether the layer listens now, and a return to listening (QuestPump woken, an instance left while the player keeps
-- it off there): the hellos then, which the group and the guild missed, and the party's unknown members asked.
local function listen()
	local now = Social.Listening()
	local was = Social.listening
	Social.listening = now
	if now and was == false then
		Social.Hello()
		askParty()
		Social.ResumeStakes()
	end
end

ns.RegisterEvent("PLAYER_LOGIN", function()
	Social.me = Social.UnitKey("player")
	Social.inGroup = IsInGroup()
	Social.listening = Social.Listening()
	paused = C_ChatInfo.InChatMessagingLockdown() == true
	guildChanged()
	askRoster()
	Social.Hello()
	if IsInGroup() then askParty() end -- a /reload in a party: whoever did not answer the hello
	lookAtDay() -- no one is online yet at the login: it only notes what the card's pact and duel fields hold
	followUp()
end)

-- The pact's and the duel's players: who is online, asked when the friends lists change (a friend's game account
-- online or offline, a character friend connected) and when another profile, with its own pact, comes in use; my card
-- when its fields change with a row; the day's turn at once when the dev clock moves.
for _, event in ipairs({ "FRIENDLIST_UPDATE", "BN_FRIEND_INFO_CHANGED", "BN_FRIEND_ACCOUNT_ONLINE",
	"BN_FRIEND_ACCOUNT_OFFLINE" }) do
	ns.RegisterEvent(event, followUp)
end
ns.On("Settings", function()
	watchDay()
	followUp()
end)
ns.On("Row", pushStakes)
ns.On("Clock", lookAtDay)

ns.RegisterEvent("PLAYER_ENTERING_WORLD", listen)
ns.On("Settings", listen)

-- The guild can be known only after the login: its hello waits for it, once per guild.
-- The ignore list changed: every surface that shows a player (the tooltip, the card window, the badges, the plates, the
-- chat mark) reads it again (Social.Shown).
ns.RegisterEvent("IGNORELIST_UPDATE", function() ns.Emit("Social") end)

ns.RegisterEvent("PLAYER_GUILD_UPDATE", function()
	guildChanged()
	askRoster()
	helloGuild()
	ns.Emit("Social")
end)

-- Joining a group: the hello to it. A member not known yet (one who joined, one whose hello the player missed) is
-- asked once the hellos had their time, so the party learns who runs QuestPump; the party's line follows the group
-- and turns with the game day (watchDay).
ns.RegisterEvent("GROUP_ROSTER_UPDATE", function()
	local inGroup = IsInGroup()
	if inGroup ~= Social.inGroup then watchDay() end
	if inGroup and not Social.inGroup then Social.HelloGroup() end
	if inGroup then
		askParty()
	else
		invite = nil
	end
	Social.inGroup = inGroup
	ns.Emit("Social")
end)

ns.RegisterEvent("ADDON_RESTRICTION_STATE_CHANGED", function()
	paused = true
	T.after("lockdown", 0.1, recheck)
end)

-- A target at hand is asked for its card (the tooltip asks on hover, the Inspect window when it opens: Badges.lua).
ns.RegisterEvent("PLAYER_TARGET_CHANGED", function()
	if Social.Listening() and ns.profile.showCards then askUnit("target") end
end)

-- The Social page's switches change the card window (the player's records shared or not, invites on or off).
ns.On("Settings", function() Social.RenderCard() end)

--@debug@
-- /qp dev peer [whisper|party|guild] <message>: a message from "Dev Peer" through the handler, as the client would
-- deliver it (a forged card: /qp dev peer party C1:L60|S9999|W99999).
function Social.DevPeer(arg)
	local channel, rest = (arg or ""):match("^(%a+)%s+(.+)$")
	channel = channel and ({ whisper = "WHISPER", party = "PARTY", guild = "GUILD" })[channel:lower()]
	if not channel then
		channel, rest = "WHISPER", arg
	end
	ns.Dispatch("CHAT_MSG_ADDON", PREFIX, rest or "", channel, "Dev Peer", Social.me or "")
end
--@end-debug@
