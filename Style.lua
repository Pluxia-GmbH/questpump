-- Every font object, atlas, texture file and sound kit the addon draws with, in one table (docs/ui.md, "The rules"):
-- Blizzard's own, so the frames look like the client on Forever and on retail. Font objects and SOUNDKIT constants are
-- read at load time, and luacheck checks them against the hand-kept .luacheckrc list only: a name a new
-- client build renames reads as nil in the client (SetFont falls back to GameFontNormal, Play stays silent), so the
-- list is checked again against Ketho's `forever` branch whenever the build moves (code-quality's pins.mjs prints
-- both). Forever-only art is guarded through ns.Style.SetAtlas; every sound sits behind the profile's sounds switch.

local ADDON, ns = ...

ns.Style = {
	font = {
		-- body text, notes and sublines follow the client's text-size setting, never below GameFontNormal (docs/ui.md,
		-- "Fonts"; the user-scaled styles of Blizzard_Fonts_Shared FontStyles.xml, forever branch, read 2026-10-04)
		body = UserScaledFontGameNormal,
		bodyWhite = UserScaledFontGameHighlight,
		label = GameFontNormal,
		labelWhite = GameFontHighlight,      -- also the text on a progress bar, whose art has a fixed height
		large = GameFontHighlightLarge,
		disabled = GameFontDisable,
		move = Fancy24Font or QuestTitleFontBlackShadow, -- the move's name, Morpheus 24 (two lines at most)
		clock = Game72Font,                  -- the running move's countdown or stopwatch
		go = GameFont72Normal,               -- the GO after the digits
		repBox = Game60Font,
		goal = GameFontNormalHuge2,          -- the pump card's goal in words ("As many as you can"), white
		hudHeader = ObjectiveTrackerHeaderFont,
		hudLine = ObjectiveTrackerLineFont,
		levelLine = GameFontNormalLarge,     -- the sheet's "QP 12 · of the Owl"
		fanfare = GameFont72Normal,          -- the QP level-up
		banner = QuestFont_Enormous,         -- the banner that closes a pump, the boss banner's title font
		-- the nameplate line under another player's name: the plate's own name font (Blizzard_NamePlates.xml), so it
		-- reads as part of the plate (GameFonts.xml, forever branch, read 2026-10-05)
		plateLine = SystemFont_NamePlate,
		-- the Streamer layout's tally (slice 23): its digits, readable on a 720p capture, and the words under them
		tally = Game40Font,
		tallyLabel = GameFontNormalLarge,
		-- the day card and the setup window (2026-10-15; the D boards, whose Cinzel stands for the game's Morpheus and
		-- their condensed numbers for Friz Quadrata; Gethe `forever` Blizzard_Fonts_Shared Shared/Fonts.xml and
		-- GameFonts.xml at 70374): a page's question, a choice's or a section's name, the day's moves and their word, the
		-- best day or the day's quality word, a stat's number and the pool's; their small caps headings and grey notes
		-- are `label` and `labelWhite` tinted (no addon text is smaller than GameFontNormal: docs/ui.md, "Fonts")
		pageTitle = Fancy30Font,
		cardTitle = Fancy18Font,
		sectionTitle = Fancy20Font,
		dayNumber = Game72Font_Shadow,
		dayWord = Fancy24Font,
		dayBadge = Fancy20Font,
		statNumber = GameFontHighlightHuge,
		poolNumber = Game40Font,
		-- the kit (Kit.lua, 2026-10-16): a list row's name, Friz 15 with its shadow that follows the client's Text Size
		-- (UserScaledFontGame15Shadow, Gethe forever 70425 Shared/FontStyles.xml), and a Settings page's title,
		-- Blizzard's own on its pages (GameFontHighlightHuge, the stat number's object)
		rowName = UserScaledFontGame15Shadow,
		canvasTitle = GameFontHighlightHuge,
	},
	atlas = {
		check = "common-icon-checkmark",
		cross = "common-icon-redx",
		heart = "GarrisonTroops-Health", -- 15 x 13, on Forever (70009) and retail
		hudHeader = "UI-QuestTracker-Secondary-Objective-Header",
		nub = "UI-QuestTracker-Objective-Nub",
		done = "UI-QuestTracker-Tracker-Check",
		recordBar = "LevelUp-Bar-Gold",
		glow = "UI-HUD-ActionBar-Proc-Loop-Flipbook",
		-- the sheet (docs/ui.md, "The character sheet"; the Character-Info atlases are Forever-only and guarded)
		generalBG = "UI-Character-Info-General-BG",
		statBG = "UI-Character-Info-Stat-BG",
		divider = "common-framedivider",
		gearSlot = "UI-Character-Info-GearSlot",
		levelLine = "UI-Character-Info-ItemLevel-Bounce",
		statTitle = "UI-Character-Info-Title",
		statLine = "UI-Character-Info-Line-Bounce2",
		xpFill = "UI-HUD-ExperienceBar-Fill-Experience-Camelot",
		xpRested = "UI-HUD-ExperienceBar-Fill-Rested-Camelot",
		xpBackground = "UI-HUD-ExperienceBar-Background-Camelot", -- the HUD bar's own track and silver capsule, 1020 x 13
		xpFrame = "UI-HUD-ExperienceBar-Frame-Camelot",
		barFill = "UI-Frame-Bar-Fill-Blue",
		levelGlow = "LevelUp-Glow-Gold",
		-- the banner that closes a pump: the boss banner's pieces and the loot banner's item plate (BossBannerToast.xml)
		bannerTop = "BossBanner-BgBanner-Top",
		bannerMid = "BossBanner-BgBanner-Mid",
		bannerBottom = "BossBanner-BgBanner-Bottom",
		bannerRow = "LootBanner-ItemBg",
		earned = "bags-glow-flash",
		-- the timed move's dial: the gold titan-line ring round the professions specialisation dial
		-- (Blizzard_ProfessionsSpecializations.xml), 281 px, so a dial about 205 units across (1.5 px a unit at 1440p
		-- and 0.8) draws it near its own size; on Forever (70009) and retail
		dial = "SpecDial_Outer_TitanLineRing",
		-- a Legendary unlock (slice 22): the legendary loot toast's own pieces (LegendaryItemAlertFrameTemplate,
		-- AlertFrameSystems.xml; on Forever 70205 and retail), behind SetAtlas's guard like the rest
		legendBG = "LegendaryToast-background",
		legendRing = "LegendaryToast-ring1",
		legendParticles = "LegendaryToast-particles1",
		legendGlow = "LegendaryToast-OrangeStarglow",
		-- QuestPump beside another player (Badges.lua, slice 23): Blizzard's follower portrait
		-- (GarrisonFollowerPortraitTemplate, Blizzard_GarrisonBase): the ring round the portrait, tinted in item
		-- quality's colours for borders 1-4, the champion's ring for border 5 (a Paragon rank), the level plate on the
		-- ring's foot that carries "QP 12" and the champion's plate; the auction house's clock while that player is in a
		-- pump; Blizzard's round portrait mask. Each on Forever (Ketho `forever`, 70009) and retail (Ketho `live`,
		-- 12.1.0), guarded as every atlas: without one the border, the plate or the clock is left out.
		ring = "GarrMission_PortraitRing_Quality",
		ringTop = "legionmission-portraitring-epicplus",
		plate = "GarrMission_PortraitRing_LevelBorder",
		plateTop = "legionmission-portraitring_levelborder_epicplus",
		clock = "auctionhouse-icon-clock",
		circle = "CircleMask",
		-- the card window's rows of what two players share (Social.lua, 2026-10-16): the pact's, the social queue's two
		-- players (29 x 26), and the duel's, War Mode's crossed swords (46 x 43); each on Forever (Ketho `forever`,
		-- 70338) and retail (Ketho `live`)
		pact = "socialqueuing-icon-group",
		duel = "pvptalents-warmode-swords",
		-- QuestPump's own menus (Popups.lua): the radio ring and its yellow tick of Blizzard's menus (Blizzard_Menu
		-- Mainline MenuVariants.lua CreateRadio, forever 70235)
		menuRadio = "common-dropdown-tickradial",
		menuRadioTick = "common-dropdown-icon-radialtick-yellow",
		-- the guide's button that reads a move's How and Tips aloud and pauses them (2026-10-10, the barber shop's
		-- play and stop until then): the credits screen's play and pause, a pair drawn for 22 units, and where a
		-- client lacks them the menus' small play glyph and their stop for the pause (the menus draw no pause); each on
		-- Forever (Ketho `forever` AtlasInfo, 70205) and retail
		voicePlay = "CreditsScreen-Assets-Buttons-Play",
		voicePause = "CreditsScreen-Assets-Buttons-Pause",
		voicePlayGlyph = "common-dropdown-icon-play",
		voicePauseGlyph = "common-dropdown-icon-stop",
		-- the Talents tab's trees (slice 37), Forever's own talent art (Blizzard_SharedTalentUI, Gethe `forever` 70374;
		-- in Ketho's `forever` and `live` AtlasInfo, 2026-10-15): the ring round the player's step is the yellow square
		-- node's border, a link the talent arrow's line and head, yellow once its step is reached, else locked; the part
		-- of a ladder's line still ahead the flight map's dashed line, which tiles; a search match the talent search's
		-- outline; a column's diamond the progress bar's diamond pip, a mask over its colour
		nodeRing = "talents-node-square-yellow",
		arrowLine = "talents-arrow-line-yellow",
		arrowLineLocked = "talents-arrow-line-locked",
		arrowHead = "talents-arrow-head-yellow",
		arrowHeadLocked = "talents-arrow-head-locked",
		aheadLine = "_UI-Taxi-Line-horizontal",
		searchMatch = "talents-search-match",
		searchExact = "talents-search-exactmatch",
		diamond = "progress-bar-diamond-pip-mask",
		-- a mastered step's seal inside its node's bottom-right corner (slice 42): the Adventures table's gold check in its
		-- ring (42 x 41; Ketho `forever` c5a49d3, 70338, and `live` 36dd01d AtlasInfo, 2026-10-17), the kit's check where a
		-- client lacks it; the beads under a ringed step are CircleMask (`circle`) over a colour
		mastered = "Adventures-Checkmark",
		-- the day card and the setup window (2026-10-15): the Options window's faded rule under a page's question, the
		-- minimal check box and its mark of the game's Options, the arrow between a day's levels ("QP 23 > 24"), and
		-- the setup's "With them" pictures beside the heart and the clock above: the achievement shield for the kept
		-- progress, the mobile companion's quest mark for the phone's catch-up; each in Ketho's `forever` AtlasInfo
		-- (70338) and on retail; and for the streak on days without the game the gamepad menu's flat calendar (Ketho
		-- `forever` c5a49d3, 70338, 127 x 104; Forever's alone, not in Ketho's `live` at 69587: QuestPump's own mark
		-- where the client lacks it)
		rule = "Options_HorizontalDivider",
		box = "checkbox-minimal",
		boxCheck = "checkmark-minimal",
		arrow = "common-icon-forwardarrow",
		kept = "UI-Achievement-Shield-1",
		-- a title's row on the sheet's Deeds tab (2026-10-16): the same shield, in colour once the title is earned
		deedShield = "UI-Achievement-Shield-1",
		phoneQuest = "Mobile-QuestIcon",
		streak = "gamepad-radial-icon-calendar",
		-- the kit (Kit.lua, 2026-10-16), each in Ketho's `forever` (70338) and `live` AtlasInfo, guarded as every atlas:
		-- the minimal box's disabled mark; the Settings panel's expandable section (More: its left end, its tiled
		-- middle, its right end closed and open); the dropdown's arrow at rest, under the cursor, open and greyed (the
		-- select); the tutorial's yellow glow round a red button (the talent frame's apply button wears it: Gethe forever
		-- Blizzard_SharedTalentFrameTemplates.xml); the quest log's setting gear (a row the setup keeps out); the
		-- professions list's header triangles, closed and open (a group's row)
		boxOff = "checkmark-minimal-disabled",
		expandLeft = "Options_ListExpand_Left",
		expandMid = "_Options_ListExpand_Middle",
		expandRight = "Options_ListExpand_Right",
		expandOpen = "Options_ListExpand_Right_Expanded",
		selectArrow = "common-dropdown-a-button",
		selectArrowHover = "common-dropdown-a-button-hover",
		selectArrowOpen = "common-dropdown-a-button-open",
		selectArrowOff = "common-dropdown-a-button-disabled",
		nextGlowLeft = "newplayertutorial-yellowglow-redbutton-left",
		nextGlowMid = "newplayertutorial-yellowglow-redbutton-middle",
		nextGlowRight = "newplayertutorial-yellowglow-redbutton-right",
		gear = "QuestLog-icon-setting",
		groupClosed = "Professions-recipe-header-expand",
		groupOpen = "Professions-recipe-header-collapse",
		-- who sees the QuestPump card (the Social page's tiles: nobody, friends, guild and group, anyone who asks), each
		-- in Ketho's `forever` (70338) and `live` AtlasInfo (the settings spec's checker, 2026-10-16)
		shareNobody = "communities-icon-lock",
		shareCircle = "socialqueuing-icon-group",
		shareAnyone = "socialqueuing-icon-eye",
		-- an ember lit and one not yet (slice 55: the HUD's embers line), the Emerald Dream's keyflame on and off, 32 x 32
		-- each, in Ketho's `forever` (d1ee569, 70532) and `live` (f3b334b, 70448) AtlasInfo, 2026-10-23
		emberOn = "keyflameon-32x32",
		emberOff = "keyflameoff-32x32",
		-- a locked step's padlock in the step-up reveal after a hold's pour (slice 65), the communities' lock
		-- (shareNobody's art); the reveal's other art is legendRing turning behind a step of a new colour, levelGlow
		-- behind the new move and as the light along the ladder's rail, and the follower portrait's `ring` as its shockwave
		lock = "communities-icon-lock",
	},
	-- The stat chips' icons, the classic stat buffs (Strength of Earth, Grace of Air, Arcane Intellect, Power Word:
	-- Fortitude, Divine Spirit); the names wait for the beta check (docs/ui.md), the stat's name always beside them.
	statIcon = {
		strength = "Interface\\Icons\\Spell_Nature_EarthBindTotem",
		agility = "Interface\\Icons\\Spell_Nature_InvisibilityTotem",
		intellect = "Interface\\Icons\\Spell_Holy_MagicalSentry",
		stamina = "Interface\\Icons\\Spell_Holy_WordFortitude",
		spirit = "Interface\\Icons\\Spell_Holy_DivineSpirit",
	},
	xpIcon = "Interface\\Icons\\XP_Icon",
	-- a row under the cursor (the focus window's, a menu entry, the Ready view's backup tile), the quest log's own title
	-- highlight, which Blizzard's menus also use
	rowHighlight = "Interface\\QuestFrame\\UI-QuestTitleHighlight",
	-- QuestPump's own menus (Popups.lua), in Blizzard's art: the menu's divider and submenu arrow (Blizzard_Menu
	-- MenuVariants.lua), forever 70235
	menuDivider = "Interface\\Common\\UI-TooltipDivider-Transparent",
	menuArrow = "Interface\\ChatFrame\\ChatFrameExpandArrow",
	-- The ready check's own marks, where the client lacks the check and cross atlases (Style.SetMark).
	markFile = { check = "Interface\\RaidFrame\\ReadyCheck-Ready", cross = "Interface\\RaidFrame\\ReadyCheck-NotReady" },
	-- QuestPump's icon inside Blizzard's frames, a full-bleed game icon as they expect (they crop an icon's border and
	-- mask a portrait round): the Character and Inspect windows' QuestPump tabs, the card window's portrait and the chat
	-- mark. The AddOns list's and the Addon Compartment's icon, the TOC's IconTexture, is QuestPump's own
	-- (Media/Icon.tga, the rounded square inside the gold rim, which those crops would cut). The sheet's Guild tab wears
	-- the guild tabard.
	addonIcon = "Interface\\Icons\\Ability_Rogue_Sprint",
	-- QuestPump's own icon (Media/Icon.tga) as a picture: the setup window's card of QuestPump on this computer
	-- (2026-10-15) and its streak chip where the client lacks the calendar (atlas.streak)
	mark = "Interface\\AddOns\\QuestPump\\Media\\Icon.tga",
	guildIcon = "Interface\\Icons\\INV_Shirt_GuildTabard_01",
	-- an inset's own marble (InsetFrameTemplate's Bg, SharedUIPanelTemplates.xml): the Inspect window's QuestPump pane
	insetBackground = "Interface\\FrameGeneral\\UI-Background-Marble",
	-- The empty-slot silhouette a muscle button shows for a move without a key-pose icon (the nearest gear
	-- slot, read through C_PaperDollInfo.GetInventorySlotInfo; the weapon row holds Heart, Breath and Balance).
	muscleSlot = { "ShoulderSlot", "ChestSlot", "HandsSlot", "WristSlot", "WaistSlot", "WaistSlot", "LegsSlot",
		"LegsSlot", "NeckSlot", "BackSlot", "BackSlot", "HandsSlot", "WaistSlot", "LegsSlot", "LegsSlot", "FeetSlot",
		"MainHandSlot", "SecondaryHandSlot", "RangedSlot" },
	-- The body map's figures, female and male (the sheet draws the character's, ns.CharBody), each with its atlas and
	-- cells: BodyArt.lua, generated by tools/bodymap.py with the atlases, fills this in (nil without the art, and the
	-- sheet stages the Hero view). Its colours (the atlas's grey shading multiplies them): the figure, an
	-- untrained muscle, This week's steps (under 4 moves, 4 and more) from one ember-to-gold ramp that also orders by
	-- brightness, the iron of the 10-and-more outline, and the light of a muscle under the cursor (tools/bodymap.py's
	-- preview reads these).
	bodymap = nil,
	bodyColor = {
		base = { 0.1, 0.105, 0.125 },
		none = { 0.36, 0.38, 0.44 },
		[0] = { 0.36, 0.38, 0.44 },
		[1] = { 0.84, 0.43, 0.18 },
		[2] = { 0.93, 0.68, 0.26 },
		outline = { 0.6, 0.63, 0.67 },
		hover = { 1, 0.96, 0.86 },
	},
	-- The glow's alpha (ADD) under a trained muscle by This week's step, under the one the cursor is on, and the alpha
	-- of every other muscle meanwhile (the spotlight).
	bodyGlow = { [1] = 0.12, [2] = 0.3, hover = 0.55, dim = 0.45 },
	-- Blizzard's pull-countdown digits (Blizzard_FrameXML/Timer.lua, TIMER_NUMBERS_SETS.BigGold): 256 × 170 per digit
	-- in four columns of a 1024 × 512 file, the glow on the same path plus "Glow".
	digits = {
		file = "Interface\\Timer\\BigTimerNumbers",
		glow = "Interface\\Timer\\BigTimerNumbersGlow",
		w = 256, h = 170, texW = 1024, texH = 512,
	},
	-- A progress row's bar (the Ladders, Deeds and Today tabs), the achievement window's own
	-- (AchievementProgressBarTemplate; SharedXML's search bar wears the same art, so every client has it; Forever 70124):
	-- the status-bar texture in Blizzard's green on a black backing, the bronze border in three pieces (left, centre,
	-- right as the template cuts the file's top three quarters) reaching 6 units out left and right and 5 above and below.
	progressBar = {
		fill = "Interface\\TargetingFrame\\UI-StatusBar",
		color = { 0, 0.6, 0 },
		backing = 0.4,
		border = "Interface\\AchievementFrame\\UI-Achievement-ProgressBar-Border",
		cuts = { { 0, 0.0625 }, { 0.0625, 0.812 }, { 0.812, 0.8745 } },
		h = 14, out = 6, over = 5,
	},
	-- The virtual partner's bars on a hold (slice 23), on the progress bar's fill: the player's in the goal's gold, the
	-- partner's in blue, grey once the partner has let go.
	partnerBar = { you = { 1, 0.82, 0 }, partner = { 0.3, 0.6, 1 }, done = { 0.5, 0.5, 0.5 } },
	-- The day card and the setup window (2026-10-15; the D boards' iron page): the grey of notes and labels, the softer
	-- grey of an intro, a rule, a cell's ground and edge, a picture's dark screen, the light of a chosen choice and its
	-- edge, a check box's ground where the client lacks its atlas, a list's ground, edge and row rules, the step dots
	-- behind and ahead (the current one gold), a choice's light under the cursor, the strip's ground and grid, a pump's
	-- bar in the game (gold) and outside it (a frost blue outline round a faint fill of it: never the colour alone).
	-- Gold words, the green check's and the red cross's are Blizzard's (NORMAL_FONT_COLOR, GREEN_FONT_COLOR,
	-- RED_FONT_COLOR, HIGHLIGHT_FONT_COLOR for white).
	saga = {
		note = { 0.64, 0.6, 0.55 },
		soft = { 0.79, 0.76, 0.71 },
		rule = { 0.88, 0.82, 0.69, 0.14 },
		cell = { 1, 1, 1, 0.035 },
		edge = { 0.78, 0.76, 0.73, 0.2 },
		screen = { 0.03, 0.027, 0.024, 1 },
		chosen = { 1, 0.82, 0, 0.13 },
		chosenEdge = { 1, 0.82, 0, 0.55 },
		box = { 0.08, 0.07, 0.05, 1 },
		list = { 0, 0, 0, 0.3 },
		listEdge = { 0.37, 0.32, 0.24, 1 },
		listRule = { 1, 1, 1, 0.08 },
		dotBehind = { 0.55, 0.48, 0.24, 1 },
		dotAhead = { 0.29, 0.26, 0.22, 1 },
		hover = { 1, 1, 1, 0.05 },
		track = { 1, 1, 1, 0.025 },
		grid = { 1, 1, 1, 0.06 },
		game = { 0.91, 0.72, 0.29, 1 },
		away = { 0.61, 0.78, 0.9, 1 },
		awayFill = { 0.61, 0.78, 0.9, 0.22 },
	},
	sound = {
		start = SOUNDKIT.IG_MAINMENU_OPTION,                   -- Start and the panel's buttons
		countdownStart = SOUNDKIT.UI_COUNTDOWN_MEDIUM_NUMBER_FINISHED,
		tick = SOUNDKIT.UI_COUNTDOWN_TIMER,                    -- each digit
		go = SOUNDKIT.UI_COUNTDOWN_FINISHED,                   -- the go and the end of a timed move
		lastFive = SOUNDKIT.UI_BATTLEGROUND_COUNTDOWN_TIMER,   -- each of a timed move's last 5 s
		record = SOUNDKIT.UI_70_CHALLENGE_MODE_NEW_RECORD,     -- a typed count beats the record, a first record
		                                                       -- (slice 72); a Rare unlock
		win = SOUNDKIT.IG_QUEST_LIST_COMPLETE,                 -- the small chime: "+N on last time", a goal reached
		                                                       -- while the move runs (a hold's, a set's counted
		                                                       -- beat), a Starter or Common unlock and a Rekindled one
		pause = SOUNDKIT.RAID_WARNING,                         -- combat hides the dim
		attention = SOUNDKIT.READY_CHECK,                      -- the "Stop to pump" card comes, or the pump opens
		                                                       -- and no voice can read it
		open = SOUNDKIT.IG_CHARACTER_INFO_OPEN,
		close = SOUNDKIT.IG_CHARACTER_INFO_CLOSE,
		checkOn = SOUNDKIT.IG_MAINMENU_OPTION_CHECKBOX_ON,
		checkOff = SOUNDKIT.IG_MAINMENU_OPTION_CHECKBOX_OFF,
		tab = SOUNDKIT.IG_CHARACTER_INFO_TAB,
		levelUp = SOUNDKIT.UI_ADVENTURES_ADVENTURER_LEVEL_UP, -- a QP level
		star = SOUNDKIT.UI_WORLDQUEST_COMPLETE,              -- a star, an Uncommon unlock, a vault goal; Welcome back
		                                                     -- and Clean Slate (slice 72)
		title = SOUNDKIT.UI_EPICLOOT_TOAST,                  -- a title, an Epic unlock
		-- a Legendary unlock: the legendary loot toast's sound where the client has it (63971, on Forever 70205 and
		-- retail), else the Epic one
		legendary = SOUNDKIT.UI_LEGENDARY_LOOT_TOAST or SOUNDKIT.UI_EPICLOOT_TOAST,
		-- a hold's seconds pour (slice 65; the apps' pour_tick, pour_post and pour_top, contract/design/tokens.json
		-- pourTick, pourPost and pourTop): a second landing on the hold bar the backpack's coin click, a goal or a best
		-- post passed the loot window's coin, the bar's top the profession pip's max-rank lock-in (Gethe forever
		-- Blizzard_SharedXML Mainline SoundKitConstants.lua, 70532)
		pourTick = SOUNDKIT.IG_BACKPACK_COIN_SELECT,
		pourPost = SOUNDKIT.LOOT_WINDOW_COIN_SOUND,
		pourTop = SOUNDKIT.UI_PROFESSION_SPEC_PIP_MAX_RANK_LOCKIN,
		-- the feel pass (slice 72; Gethe forever Blizzard_SharedXML Mainline SoundKitConstants.lua, 70888, read
		-- 2026-11-11): the streak day's row on the pump's banner, the quest tracker's auto-complete chime (23404), its
		-- own cue apart from the small chime and a star; a pump that finished with no win, the quest window's close (876)
		streak = SOUNDKIT.UI_AUTO_QUEST_COMPLETE,
		finish = SOUNDKIT.IG_QUEST_LIST_CLOSE,
	},
	-- The feel pass's motion (slice 72), in seconds, after the apps' contract/design/tokens.json where a token names it:
	-- a frame that comes by itself fades in over `appear` (the dim with its card, the "Stop to pump" card: the phones'
	-- `stage` is 200 ms; the spec's 0.25 s here, the banner's own fade); the banner's rows come `row` apart, each over
	-- `rowIn` from `rowFrom` units to its right (the pour's rows' entrance); a value carried home moves over `carry`
	-- (the tokens' carry and xpFill, 600 ms), a bar that passes a level holding `turn` at full under its flash
	-- (levelRoll); a number that changes pops from `popFrom` of its size over `pop` (digitPop: 300 ms from 1.25); the
	-- level-up's bars grow over `bars` and its number comes over `number` from `numberFrom` of its size, as the
	-- countdown's digits do (fanfareBars 500 ms; Blizzard's startNumbers: 0.3 s from 0.25). With the game's reduced
	-- motion (Tree.Moving) each shows its end at once (Kit.lua, "Motion").
	motion = { appear = 0.25, row = 0.3, rowIn = 0.3, rowFrom = 48, carry = 0.6, turn = 0.25, pop = 0.3, popFrom = 1.25,
		bars = 0.5, number = 0.3, numberFrom = 0.25, tick = 0.03 },
}

function ns.Style.Play(key)
	if ns.profile and ns.profile.sounds == false then return end
	local id = ns.Style.sound[key]
	if id then PlaySound(id) end
end

function ns.Style.SetFont(fontString, key)
	fontString:SetFontObject(ns.Style.font[key] or GameFontNormal)
end

-- A font string on `parent` in a font key, left-aligned unless `justify` says otherwise; with a width it wraps within
-- it (a text without one runs on in a single line).
function ns.Style.Text(parent, key, justify, width)
	local fs = parent:CreateFontString(nil, "OVERLAY")
	ns.Style.SetFont(fs, key)
	fs:SetJustifyH(justify or "LEFT")
	if width then
		fs:SetWidth(width)
		fs:SetWordWrap(true)
	end
	return fs
end

-- Sets a named atlas when the client has it (Forever-only art is absent on retail and the reverse); false otherwise.
function ns.Style.SetAtlas(texture, key, useAtlasSize)
	local name = ns.Style.atlas[key]
	if name and C_Texture.GetAtlasInfo(name) then
		texture:SetAtlas(name, useAtlasSize)
		return true
	end
	return false
end

-- The same at the atlas's own shape inside a square `size` units across (C_Texture.GetAtlasInfo's width and height),
-- never stretched to the square (a 15 x 13 heart at 24 x 21): the setup window's "With them" chips and the phone
-- card's heart and clock; false where the client lacks it.
function ns.Style.FitAtlas(texture, key, size)
	local name = ns.Style.atlas[key]
	local info = name and C_Texture.GetAtlasInfo(name)
	if not (info and ns.Style.SetAtlas(texture, key)) then return false end
	local scale = size / math.max(info.width, info.height)
	texture:SetSize(info.width * scale, info.height * scale)
	return true
end

-- The action bar's proc glow round a frame (the HUD's lifecycle chip, the Ready view's Start), under the frame's own
-- art and `x` and `y` units out round it: the flipbook of 6 x 5 frames in 1 s (ActionButtonSpellAlerts.xml), which
-- plays only while it shows (Style.ShowGlow); a soft gold fill on a client without the atlas.
function ns.Style.ProcGlow(frame, x, y)
	local glow = frame:CreateTexture(nil, "BACKGROUND", nil, -8)
	glow:SetPoint("TOPLEFT", -x, y)
	glow:SetPoint("BOTTOMRIGHT", x, -y)
	glow:SetAlpha(0.6)
	if ns.Style.SetAtlas(glow, "glow") then
		glow.pulse = frame:CreateAnimationGroup()
		glow.pulse:SetLooping("REPEAT")
		local flip = glow.pulse:CreateAnimation("FlipBook")
		flip:SetTarget(glow)
		flip:SetFlipBookRows(6)
		flip:SetFlipBookColumns(5)
		flip:SetFlipBookFrames(30)
		flip:SetFlipBookFrameWidth(0)
		flip:SetFlipBookFrameHeight(0)
		flip:SetDuration(1)
	else
		glow:SetColorTexture(1, 0.82, 0, 0.25)
	end
	glow:Hide()
	return glow
end

-- Shows or hides a proc glow, its flipbook playing only while it shows (a playing one keeps its frame).
function ns.Style.ShowGlow(glow, on)
	glow:SetShown(on)
	if not glow.pulse then return end
	if not on then
		glow.pulse:Stop()
	elseif not glow.pulse:IsPlaying() then
		glow.pulse:Play()
	end
end

-- Holds a showing proc glow still on the flipbook frame it reached (a paused group keeps its progress), for a glow
-- that marks without moving: the Talents tab's halo, which pulses once at most (WCAG 2.2.2; slice 36's design). A
-- ShowGlow afterwards plays it on.
function ns.Style.HoldGlow(glow)
	if glow.pulse and glow.pulse:IsPlaying() then glow.pulse:Pause() end
end

-- A box on `parent`, 2026-10-15 (the day card's cells, the setup window's cards, tiles, chips and list): a frame with
-- its ground and a border one unit wide, in four textures, which box:Paint colours ({ r, g, b, a } each, the ground or
-- the edge nil to leave it off). They are the box's own textures, never its window's, so a UI replacement's skin that
-- strips a window's textures (ElvUI's HandleFrame, Skins.lua) leaves them. `kind`: the frame's type, "Button" for a
-- control whose own words and highlight draw over its box (a child frame's box would cover them: a frame draws over
-- its parent's textures and font strings whatever their layers).
local EDGES = { { "TOPLEFT", "TOPRIGHT" }, { "BOTTOMLEFT", "BOTTOMRIGHT" }, { "TOPLEFT", "BOTTOMLEFT" },
	{ "TOPRIGHT", "BOTTOMRIGHT" } }

local function paint(box, ground, edge)
	box.ground:SetShown(ground ~= nil)
	if ground then box.ground:SetColorTexture(unpack(ground)) end
	for _, t in ipairs(box.edges) do
		t:SetShown(edge ~= nil)
		if edge then t:SetColorTexture(unpack(edge)) end
	end
end

function ns.Style.Box(parent, kind)
	local box = CreateFrame(kind or "Frame", nil, parent)
	box.ground = box:CreateTexture(nil, "BACKGROUND")
	box.ground:SetAllPoints()
	box.edges = {}
	for i, ends in ipairs(EDGES) do
		local t = box:CreateTexture(nil, "BORDER")
		t:SetPoint(ends[1])
		t:SetPoint(ends[2])
		if i <= 2 then t:SetHeight(1) else t:SetWidth(1) end
		box.edges[i] = t
	end
	box.Paint = paint
	return box
end

-- The arrow between two numbers as inline markup ("QP 23 > 24" on the day card): the forward arrow atlas at the line's
-- height, or the locale's word where the client lacks it (whether the client's fonts carry an arrow glyph is
-- unverified, as for the star and the check).
function ns.Style.Arrow(size)
	local name = ns.Style.atlas.arrow
	if C_Texture.GetAtlasInfo(name) then return CreateAtlasMarkup(name, size, size) end
	return ns.L["streamer.card.to"]
end

-- The same for a Cooldown's swipe, which takes a file, not an atlas: the atlas's file and its corners, as Blizzard's
-- Journeys and rune cooldowns set theirs (Blizzard_Journeys.lua, RuneFrame.lua); false without the atlas.
function ns.Style.SetSwipeAtlas(cooldown, key)
	local info = C_Texture.GetAtlasInfo(ns.Style.atlas[key])
	if not info then return false end
	cooldown:SetSwipeTexture(info.file or info.filename)
	cooldown:SetTexCoordRange({ x = info.leftTexCoord, y = info.topTexCoord },
		{ x = info.rightTexCoord, y = info.bottomTexCoord })
	return true
end

-- A move's key-pose cell for the icons (the muscle buttons, their flyouts, the Exercises page, the figure's
-- fallback): file, left, right, top, bottom on the icon sheets tools/figures writes (16 x 16 cells of 64 px each:
-- cells 0-255 on Media/MoveIcons.png, 256-511 on Media/MoveIcons2.png since slice 21's poses, 512-767 on
-- Media/MoveIcons3.png since slice 47's; FigureIcons.lua maps the move id to its cell), or nil for a move without an
-- icon (the callers then show a slot silhouette).
local ICONS = { "Interface\\AddOns\\QuestPump\\Media\\MoveIcons.png",
	"Interface\\AddOns\\QuestPump\\Media\\MoveIcons2.png", "Interface\\AddOns\\QuestPump\\Media\\MoveIcons3.png" }
local ICON_GRID = 16

-- The pixel size of a key-pose cell and of Blizzard's slot silhouettes: the figure draws them at one UI unit a pixel,
-- as it draws the pack's sheets, because a UI unit is already the screen's height / 768 x the UI scale in pixels (1.5
-- at 1440p and 0.8), so a 128-unit draw would stretch 64 pixels to 192 there.
ns.Style.iconPx = 64

function ns.Style.MoveIcon(moveId)
	local c = ns.FigureIcons and ns.FigureIcons[moveId]
	if not c then return nil end
	local cells = ICON_GRID * ICON_GRID
	local file = ICONS[math.floor(c / cells) + 1]
	if not file then return nil end
	c = c % cells
	local col, row = c % ICON_GRID, math.floor(c / ICON_GRID)
	return file, col / ICON_GRID, (col + 1) / ICON_GRID, row / ICON_GRID, (row + 1) / ICON_GRID
end

-- A move's picture where no figure plays (the figure's fallback, the swap tile, the day card's cells, the setup
-- window's): its key-pose cell (MoveIcon), else its primary muscle's gear-slot silhouette, uncropped; file, left,
-- right, top, bottom, or nil for neither.
function ns.Style.MovePicture(moveId)
	local file, l, r, t, b = ns.Style.MoveIcon(moveId)
	if file then return file, l, r, t, b end
	local move = moveId and ns.Library[moveId]
	file = move and ns.Style.SlotIcon(move.mp[1])
	if file then return file, 0, 1, 0, 1 end
end

-- Stars and checks as inline markup: the client's fonts carry no star or check glyph, so a star is the classic star
-- raid marker (on every client since the first) and a check the check-mark atlas; an empty star is the marker in grey.
-- The marker's file is the best day's star too (its toast, the day card: 2026-10-15).
ns.Style.star = "Interface\\TargetingFrame\\UI-RaidTargetingIcon_1"

function ns.Style.Stars(n, max)
	local full = "|T" .. ns.Style.star .. ":12:12|t"
	local empty = "|T" .. ns.Style.star .. ":12:12:0:0:64:64:0:64:0:64:110:110:110|t"
	return string.rep(full, n) .. string.rep(empty, math.max(0, (max or n) - n))
end

function ns.Style.Check()
	return CreateAtlasMarkup(ns.Style.atlas.check, 12, 12)
end

-- The red cross as inline markup `size` units square (a state not done inside a line: the General page's beta line),
-- the ready check's own file where the client lacks the atlas.
function ns.Style.Cross(size)
	local name = ns.Style.atlas.cross
	if C_Texture.GetAtlasInfo(name) then return CreateAtlasMarkup(name, size, size) end
	return ("|T%s:%d:%d|t"):format(ns.Style.markFile.cross, size, size)
end

-- Day pips as inline markup (the Talents tab's hover and card, slice 37): `have` of `need` as the tracker's check, the
-- rest as its objective nub, so a day never shows by colour alone.
function ns.Style.Pips(have, need)
	local done = CreateAtlasMarkup(ns.Style.atlas.done, 12, 12)
	local open = CreateAtlasMarkup(ns.Style.atlas.nub, 12, 12)
	have = math.max(0, math.min(have or 0, need or 0))
	return string.rep(done, have) .. string.rep(open, math.max(0, (need or 0) - have))
end

-- Embers as inline markup (slice 55: the HUD's embers line): `have` of `max` lit, the rest unlit, as the keyflame on
-- and off at the line's 14 units; the day pips where the client lacks either (Style.Pips), so an ember never shows by
-- colour alone.
function ns.Style.Embers(have, max)
	local on, off = ns.Style.atlas.emberOn, ns.Style.atlas.emberOff
	if not (C_Texture.GetAtlasInfo(on) and C_Texture.GetAtlasInfo(off)) then return ns.Style.Pips(have, max) end
	have = math.max(0, math.min(have or 0, max or 0))
	return string.rep(CreateAtlasMarkup(on, 14, 14), have) .. string.rep(CreateAtlasMarkup(off, 14, 14), max - have)
end

-- A game icon (a 64-px Interface\Icons file) as inline markup `size` units high (0: the line's height), its icon
-- border cropped: the pump card's rewards line's stat icons.
function ns.Style.IconMark(file, size)
	return ("|T%s:%d:%d:0:0:64:64:5:59:5:59|t"):format(file, size, size)
end

-- The chat mark before a QuestPump player's name: the addon's icon at the line's height, as Blizzard puts its
-- Timerunning icon before a name (ChatFrameUtil.GetDecoratedSenderName, 70205).
function ns.Style.AddonMark()
	return ns.Style.IconMark(ns.Style.addonIcon, 0)
end

-- A state mark on a texture: the green check when done, the red cross when not (the setup screen's steps), with the
-- ready check's own files where the client lacks the atlas.
function ns.Style.SetMark(texture, done)
	local key = done and "check" or "cross"
	if not ns.Style.SetAtlas(texture, key) then texture:SetTexture(ns.Style.markFile[key]) end
end

-- The heart-rate check mark: a pump the heart rate proved (slice 8), on its row in the sheet's Log.
function ns.Style.Heart()
	return CreateAtlasMarkup(ns.Style.atlas.heart, 14, 12) -- width, height: the atlas's own shape at the check's height
end

-- The empty-slot silhouette of a muscle's nearest gear slot (a texture file id), or nil.
function ns.Style.SlotIcon(muscle)
	local slot = ns.Style.muscleSlot[muscle]
	if not slot then return nil end
	local ok, _, texture = pcall(C_PaperDollInfo.GetInventorySlotInfo, slot) -- a slot the client lacks raises
	return ok and texture or nil
end

-- Texture coordinates of one digit on the BigTimerNumbers file (Blizzard's StartTimer_SetTexNumbers).
function ns.Style.DigitCoords(n)
	local d = ns.Style.digits
	local columns = math.floor(d.texW / d.w)
	local l = (n % columns) * d.w / d.texW
	local t = math.floor(n / columns) * d.h / d.texH
	return l, l + d.w / d.texW, t, t + d.h / d.texH
end
