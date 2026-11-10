-- Every tunable, key for key the constants table of docs/plan.md ("Build spec for slices 1-3"); tools/lint.lua reads
-- that table (make lint) and fails when a key is missing on either side or a plain number disagrees. A change here that
-- raises intensity updates docs/safety.md in the same commit. `coach_daily_lines` is a Firebase Remote Config value in
-- the phone apps and is not mirrored here. Later slices read most of these; slice 1 reads the lifecycle, the owed
-- queue, the dim, the countdown, the odometer, the cardio dose, the stand-up move, the input limits, XP per move and
-- the day.

local ADDON, ns = ...

ns.C = {
	-- the pump lifecycle (David, 2026-09-23: the pump asks at once); settle_s is also the hub merge's grace
	settle_s = 1, still_s = 1, card_after_move_s = 0,
	-- the X on a Ready view hides the pump this long, then it asks again as an offer does (David, 2026-10-04)
	snooze_s = 10,
	-- the HUD's unfinished line: its tooltip's time for the queue (game-design.md: a move takes 60-90 s) and how many
	-- of its moves it names
	move_estimate_s = 75, owed_tip_moves = 5,
	-- the lifecycle's tick and its polls; an NPC window flagged open while none of its frames shows this long lost its
	-- close event; a popup's, a cinematic's or a movie's own frame shows this long after the event that announced it
	lifecycle_tick_s = 0.25, npc_stuck_s = 3, popup_grace_s = 0.5,
	-- the helper's sign of life may be this old at a load and still say the helper runs: one that runs writes it every
	-- 10 to 12 s while the game does, and the rest is room for a computer busy starting the game (2026-10-03)
	helper_alive_slack_s = 30,
	-- the setup screen's cadence (David, 2026-10-09): at every login for this many game days after it first showed,
	-- then at most once in this many; a paired phone quiet this many days opens its step again
	setup_first_days = 7, setup_every_days = 7, phone_quiet_days = 14,
	-- the companions' chat nudges: at most one in this many game days, the streak's from a streak this long
	nudge_every_days = 7, nudge_streak_days = 7,
	xp_owed_paid = 5,
	dim_alone = 0.85, dim_group = 0.5, dim_streamer = 0,
	-- the move figure (slice 10); a move tooltip's, smaller (2026-10-16, the game's look)
	figure_ready_px = 192, figure_new_hold_s = 2, figure_preview_s = 0.3,
	figure_pack_min = 10, figure_tip_px = 144,
	lead_ms = 500, countdown_s = 5,
	try_s = 20,
	odometer_tick_s = 0.25, travel_yd = 1500,
	-- the pixel strip (slice 5): the heartbeat, the least time a frame shows (above the 10 Hz battery mode's 100 ms
	-- capture interval), the profile and the sheet painted again for a helper started mid-session
	strip_heartbeat_s = 1, strip_hold_ms = 120, strip_blob_repeat_s = 300,
	-- the difficulty score and the tiers (slice 2)
	-- xr_scale from 1.60.1's QuestXP (2026-09-25; was 0.45): the ordinary quest is XP tier 5, xr 2.0, so s_xp 0.5
	xp_norm = 0.20, xr_scale = 0.25,
	w_xp = 0.40, w_lvl = 0.20, w_tag = 0.25, w_obj = 0.10, w_trav = 0.05,
	tag_raid = 1.0, tag_dungeon = 0.8, tag_legendary = 0.7, tag_elite = 0.6, tag_pvp = 0.5, tag_event = 0.2,
	obj_count_norm = 3, obj_req_norm = 30, obj_prior = 0.3, trav_norm_yd = 3000,
	bonus_boss = 0.15, bonus_finale = 0.15, bonus_group = 0.10, trivial_cap = 0.25,
	tier_t2 = 0.40, tier_t3 = 0.70,
	-- the moves a quest gives by its tier (T1, T2, T3), at the player's pace (General > Moves per quest; David,
	-- 2026-10-05): Balanced is the tiers' own sizes, a T3's all-out window among its three; a merged quest past the
	-- first gives the T1 size; the major move comes on top, but a quest gives at most its pace's T3 size (Engine.lua
	-- quota; David, 2026-10-10), so Light, whose most is one, gives no major move
	pace_moves = { light = { 1, 1, 1 }, balanced = { 1, 2, 3 }, extreme = { 2, 3, 4 } },
	-- the time between two moves that make sim and the engine fixture assume: no move starts itself, the player rests and
	-- presses Start (2026-10-02); the time estimate's (slice 51, Engine.lua ns.MoveTime) seconds a rep and least seconds
	-- of a rep or hold move
	transition_s = 10, time_rep_s = 3, time_move_min_s = 20,
	-- an expedition (slice 51; Rapha, 2026-10-20 and 2026-10-21): its inner rounds the Normal pump (tier 2) with no
	-- all-out effort, the last the Big pump; a breather between two rounds; the Quick pump's lengths in minutes, the
	-- first its default (Rapha, 2026-11-03: one length set in Settings); the warm-up's blocks and the
	-- cool-down's, a move of each in order; the cool-down adds no move past its estimate (ns.PumpTime)
	expedition_round_tier = 2, expedition_rest_s = 30, expedition_lengths_min = { 5, 10, 15, 20, 30 },
	expedition_warmup_blocks = { "mobility", "cardio" }, expedition_cooldown_blocks = { "yoga", "mobility" },
	expedition_cooldown_s = 150,
	-- selection (slice 2; calibrated in the simulator on 2026-09-24, `make sim`)
	w_due = 0.35, w_recovered = 0.15, w_variety = 0.15, w_fit = 0.30, w_theme = 0.05,
	-- by ladder id, every id from 1 (the phone cores read it as a list): the strength ladders 48, the holds 72, cardio 1,
	-- a pose family 24 (slice 21: its mark comes at most once a game day); 3 and 5, retired, keep the pull's and the
	-- core hold's; the sit-up 72 though its moves are reps (2026-10-17: Contreras and Schoenfeld 2011 ask 48 h between
	-- sessions of trunk flexion and, to be prudent, 72 or more)
	cadence_h = { [1] = 48, [2] = 48, [3] = 48, [4] = 48, [5] = 72, [6] = 1, [7] = 72, [8] = 48, [9] = 48, [10] = 72,
		[11] = 72, [12] = 24, [13] = 24, [14] = 24, [15] = 24, [16] = 24, [17] = 24, [18] = 24, [19] = 24, [20] = 72,
		[21] = 72, [22] = 72, [23] = 48, [24] = 72 },
	overdue_force = 1.5, -- a ladder this many cadences overdue takes the next slot its region allows
	flow_due_scale = 0.3,
	load_class = { 1, 2, 3 }, load_cap = 6,
	half_life_h = { strength = 36, ballistic = 36, allout = 36, tendon = 48, run = 12 },
	variety_once = 0.5, variety_recent_pumps = 5, variety_pattern_pumps = 2, novelty = 0.25, novelty_quiet_days = 14,
	-- the four winds (slice 54, 2026-10-23; Engine.lua windsFirst): each facing's share of a core slot's holds, McGill's
	-- normal endurance times in seconds (the flexors 136, a side 97, the extensors 161: McGill 1999), never shown
	facing_share = { front = 136, side = 97, back = 161 },
	fit_prior = { 1.0, 0.6, 0.3 },
	roll_top = 3, rung_reach = 1,
	-- the day plan (David, 2026-10-03: "its more improtant a good variety and trainingplan is followed than that the
	-- excercises follow what happened in the quest"): each block's share of a game day's moves, in percent, by the
	-- player's focus (ns.Focus; Balanced until a focus row says otherwise); a slot goes to the block furthest behind its
	-- share (ns.BLOCKS, Engine.lua). No block under 15 and strength never under 25, so every focus keeps every part;
	-- Balanced puts strength first, its largest block (David, 2026-10-16: "The most important are the strength moves.
	-- There should be several per day. The goal is to get better at strength and a good body"; Balanced was 35/25/20/20
	-- and the Strength focus 50/20/15/15 until then)
	focus_share = {
		balanced = { strength = 50, cardio = 20, mobility = 15, yoga = 15 },
		strength = { strength = 55, cardio = 15, mobility = 15, yoga = 15 },
		endurance = { strength = 30, cardio = 40, mobility = 15, yoga = 15 },
		flexibility = { strength = 25, cardio = 15, mobility = 30, yoga = 30 },
	},
	focus_level = 10, -- the QP level whose first pump's close asks for a focus (Blizzard's talent level)
	-- the dose
	-- the goal (ns.Goal): a step past the form of the last fresh sets (fresh: goal_fresh_h after the ladder's last set,
	-- or a new game day after goal_night_h, a night's sleep); a repeat aims lower with every set the ladder took that
	-- game day (the second at 90 % of the form, the sixth and later at 70 %)
	goal_fresh_h = 16, goal_night_h = 8, goal_form_n = 3, goal_rep_step = 1, goal_hold_pct = 5, goal_hold_min_s = 2,
	goal_hold_max_s = 5,
	goal_repeat_pct = { 90, 85, 80, 75, 70 },
	-- a hold ladder's goal in fives (slice 52, ns.Goal): from goal_hold_long_from_s of form a fresh goal is the multiple
	-- of goal_hold_round_s nearest the form × (100 + goal_hold_long_pct) %, at least goal_hold_min_s and at most
	-- goal_hold_long_max_s past the form; a fresh set at or past overpower_pct % of its window's top counts two days at
	-- the top (DB.lua anchorAttempt)
	goal_hold_long_from_s = 30, goal_hold_long_pct = 10, goal_hold_round_s = 5, goal_hold_long_max_s = 10,
	overpower_pct = 125,
	-- the Stamina Ledger's holds (slice 52, ns.HoldRead): a set's noise in centi-logs (an exact count, a multiple of 5),
	-- the first set's doubt, the doubt a game day adds, the bands' z × 100 (the raise's, the card's "Now"), A little's
	-- and A lot's offsets, the raise's most (% of the goal), the trend's cap, a climb's doubt, the layoff (the steady
	-- ledger's, ledger_layoff_days and ledger_fade_days: past 7 game days a gain over the rung's prior fades back to it
	-- over 45 more; the trend runs at most 7 game days ahead) and the doubt it adds, the doubt's floor and cap; the hold
	-- ask only while the doubt is ledger_ask_v or more; the top's forecast (ns.HoldTopIn) takes ledger_hold_due_pct % of
	-- the days the trend gives
	ledger_hold_noise = { 10, 15 }, ledger_hold_v0 = 625, ledger_hold_drift = 4, ledger_hold_band = { 52, 128 },
	ledger_hold_little = 14, ledger_hold_lot = 26, ledger_hold_raise_pct = 130, ledger_hold_trend_max = 40,
	ledger_hold_climb_v = 900, ledger_layoff_days = 7, ledger_fade_days = 45, ledger_hold_layoff_v = 100,
	ledger_hold_v_min = 16, ledger_hold_v_max = 1600, ledger_ask_v = 36, ledger_hold_due_pct = 125,
	-- a step's card adds "Records come back when you are fresh." once its ladder (off the ladders, its move) took this
	-- many sets that game day (slice 36, ns.StartFor's `tired`)
	start_tired_sets = 2,
	-- a loaded move's weight (2026-10-09; Progress.lua ns.LoadPlan): a set's strength is kg × (load_epley + reps)
	-- (Epley 1985), the goal at another weight the reps of the same strength; the weight goes up once load_top_sets
	-- fresh sets in a row (a tired set above the form among them, DB.lua recordCount) reach the top at it (the rung
	-- window's hi where the move stands for its rung or has a window of its own, else load_top_reps, about 67 % of a
	-- one-rep max: ACSM's 60-70 % for novices, Ratamess 2009), by load_step_pct of it and at least load_step_kg (a
	-- barbell load_bar_step_kg, a kettlebell the next of load_bells);
	-- a typed weight above load_max_kg is refused, and one above really_load × the planned one asks again; a step
	-- up waits until the form's strength makes load_floor_reps at the heavier weight; the weight box's - and + step
	-- load_box_step tenths of a kg (a kettlebell's: its sizes; ns.LoadStep)
	load_epley = 30, load_top_reps = 15, load_top_sets = 2, load_step_pct = 5, load_step_kg = 1, load_bar_step_kg = 2.5,
	load_bells = { 4, 6, 8, 10, 12, 14, 16, 20, 24, 28, 32, 36, 40, 44, 48 },
	load_max_kg = 300, really_load = 2, load_floor_reps = 5, load_box_step = 5,
	-- a cardio move is short and hard (David, 2026-10-04: "you want the excercises to count as you leave your computer
	-- for them"): 20-30 s by the quest's active minutes, × the steady ledger's M up to 45 s (slice 52; × P until then),
	-- × a form's `span` (the pad walks' 2, Library.lua), an uncounted all-out window its build and 20 s × M
	cardio_base_s = 20, cardio_per_min_s = 1, cardio_cap_s = 30,
	allout_s = 20, allout_build_s = 10,
	-- the Stamina Ledger's steady side (slice 52, ns.SteadyM): the prior, ledger_prior_s at the cardio step it starts
	-- on; a form ledger_rung_ln centi-logs (×1.35) harder a rung; the dose's margin under what the player keeps; the
	-- doubt's start, floor and top; M's range in thousandths; an answer's move of the estimate in steps of the doubt's
	-- third (Yes, easily; Yes, just; I eased off; stopped), a Yes, easily's floor ledger_easy_floor over its seconds;
	-- the ask's doubts (it asks at the first, twice a game day from the second), its most a game day and the game days
	-- after which it asks anyway; a layoff past ledger_layoff_days (the holds', above) fades a gain over the prior over
	-- ledger_fade_days more days with ledger_layoff_doubt more doubt (Rekindle); the play days the forecast's rise reads;
	-- the game days between two of the best M's steps of 50 (mHigh, the cardio stat's: one step a week at most, as P's
	-- weekly steps paid it, so a new ledger's doubt never pays a season's Agility in its first days)
	ledger_prior_s = 36, ledger_rung_ln = 30, ledger_margin = 21, ledger_doubt = { 35, 6, 35 },
	ledger_m = { 1000, 1500 }, ledger_moves = { 2, 1, -4, -6 }, ledger_easy_floor = 10, ledger_ask_doubt = { 12, 20 },
	ledger_asks_day = 2, ledger_ask_days = 7, ledger_layoff_doubt = 10, ledger_plays = 8, ledger_m_days = 7,
	-- Keep going (slice 52): the control shows in a steady move's last keep_going_s, up to the full move, cardio_cap_s
	-- × ledger_m's top (45 s)
	keep_going_s = 5,
	-- the Stamina Ledger's hold reading (slice 52): 1000 × the inverse Mills ratio φ(z) / (1 − Φ(z)) at z = (i − 8) / 4,
	-- i counted from 0, the lift a hold that stopped near its goal gives the ledger's estimate
	ledger_lam = { 55, 90, 139, 204, 288, 389, 509, 646, 798, 964, 1141, 1329, 1525, 1729, 1939, 2154, 2373, 2596, 2823,
		3052, 3283 },
	-- a counted set (David, 2026-10-05: "Always 30 s"): an all-out window of a countable form, all out for its 30 s from
	-- the window's go with no build (2026-10-08), never × M, its count optional and never a goal (2026-10-16, slice 42),
	-- a typed count above count_max asks again; from cardio rung closing_rung a pump's first steady cardio move becomes
	-- its closing window
	count_s = 30, count_max = 80, closing_rung = 5,
	mark_s = 5, mark_to = 300, mark_voice_s = 30, mark_gap_s = 4,
	-- every all-out window's two shouts (slice 53, 2026-10-23; encouragement at short gaps raises a maximal effort,
	-- Andreacci 2002): "Halfway. Faster!" at the first's seconds left, "Ten more. Everything!" at the second's, each in
	-- that mark's place (Speech.lua; the callout family shout<s>, callout_takes' shout takes each)
	shout_at_s = { 15, 10 },
	-- a saved move's result holds this long before the next move's Ready view: a set saved with no pour, and since
	-- 2026-11-04 every pour once its rows are in (Rapha: "yes try for now with auto progress"; slices 65 and 66 waited
	-- for Next until then); the apps' own pumps read it from contract/library.json
	record_hold_s = 1.5,
	banner_plain_s = 4, banner_hold_s = 5, banner_row_s = 1.5, banner_fade_s = 0.5, banner_rows = 4,
	-- an orange or gold unlock's frame, the move in its colour's light, stays this long (a click closes it sooner)
	legend_s = 7,
	standup_after_min = 30, standup_s = 60, standup_retry_s = 60,
	major_h = 2,
	pause_hour_s = 3600, pause_four_hours_s = 14400,
	party_ask_s = 30, -- a mixed party's question stays this long; unanswered, it is Bank all
	-- progression (slice 2)
	-- the cardio ladder (2026-10-16, slice 42): a timed rung (steps 1-4, and any the setup leaves no countable form)
	-- opens after cardio_full_days full moves (slice 52: sessions with a Yes on a form of the step's pool dosed at the
	-- player's step that ran the full move's 45 s while the step's own M stood at the ledger's top; ready days, a kept
	-- pace at any dose, until then); a counted rung opens after its clean days, by the step it climbs from
	-- (cardio_clean_days: game days with a window of the step's own forms that ran its 30 s saved clean; its top count
	-- twice an hour apart until then, David, 2026-10-05); a step goes down after cardio_rung_down stops at it on as many
	-- game days (slice 53's footing: each a new session by the night rule, with no finished move of the step between
	-- them, never a proving window's stop and never on the game day of a climb; two in a row until then)
	cardio_full_days = 2, cardio_rung_down = 2,
	cardio_clean_days = { [5] = 3, [6] = 4, [8] = 5, [10] = 6 },
	-- 2026-10-18 18:24:04 UTC, slice 42's merge, which first wrote a counted window's answer: a window before it was
	-- never asked, so it is practice and counts no clean day (DB.lua cardioMove; David, 2026-10-31)
	counted_ask_ts = 1792347844,
	-- 2026-10-31 18:00:00 UTC, the build that took the first clean window from answered windows alone: a cardio step
	-- mastered before it may rest on windows nobody answered, so it masters again from answered ones (DB.lua's mastered
	-- rows)
	cardio_mastery_ts = 1793469600,
	-- 2026-11-06 18:00:00 UTC, the build that doubled the pad walks (a move's `span`): a row before it ran its form's
	-- plain length, so the steady ledger reads its seconds whole (ns.RowSpan)
	ledger_span_ts = 1793988000,
	star_fractions = { 1 / 3, 2 / 3, 1 },
	-- a rung unlocks with its anchor at the window's top on unlock_days separate game days, each a new session by the
	-- night rule (slice 20; two sets at the top until 2026-10-03, which one long evening could give); plateau_attempts
	-- counts days the same way
	unlock_days = 2, plateau_attempts = 4, plateau_fraction = 0.8,
	-- a climb back to a rung the player has done (its own move's record at the floor) after a step down takes this many
	-- (2026-10-10; ns.Returning)
	return_days = 1,
	-- a Legendary rung whose anchor is tendon-heavy asks for this many days at the top (slice 22; ns.UnlockDays)
	unlock_days_tendon = 3,
	-- a pose family's next step opens after a goal pose of its step (ns.Marks) was held its whole time and answered
	-- close enough to "How close to the goal pose?" (Good or Perfect, Perfect alone below a risky step: ns.MarkNeeds,
	-- 2026-10-09) on pose_unlock_days separate game days (slice 47, 2026-10-19: any goal pose of the step counts a day;
	-- its marking pose alone on 6 days from slice 21 until then, the honour system with no question from 2026-10-04),
	-- at least pose_anchor_days of them on the step's main pose (its anchor or the twin standing for it, ns.InChain),
	-- pose_anchor_days_risky where only Perfect counts, and at least pose_perfect_days answered Perfect (ns.PoseNeeds;
	-- the step card's words name one Perfect day: Locales ladders.step.short.*)
	pose_unlock_days = 3, pose_anchor_days = 1, pose_anchor_days_risky = 2, pose_perfect_days = 1,
	-- 2026-10-20 21:56:24 UTC, slice 47's merge, which gave every pose step its new goal poses: a pose family's step
	-- mastered before it masters again once those are topped (DB.lua's mastered rows; slice 47's F3)
	pose_mastery_ts = 1792533384,
	-- the Path's pace in the apps and the desktop app (2026-10-09, Pro's alone since 2026-10-14; the addon reads none):
	-- a ladder's pace to its next orange or gold step (else its top) from path_climbs dated climbs on, counted in days
	-- under the first of path_units, in weeks under the second, in months under the third, else in years
	path_climbs = 2, path_units = { 14, 56, 730 },
	rung_down = { under_lo = 2, under_half_lo = 1 },
	placement_idle_days = 14, start_rung = { none = 2, some = 3, training = 4 }, placement_low = 5,
	censor_s = 90,
	-- input validation: really_record is 2 × the record, 1.5 × once the record is 15 or more
	max_reps = 100, max_seconds = 300, really_placement = 2, really_record = { low = 2, high = 1.5, from = 15 },
	bad_day = 0.5,
	-- a timed move ended before this share of its seconds asks "Did you do the whole move?" before anything is
	-- saved, and a hold's stopwatch under this share of its goal (or record) leaves the box empty; the apps' own
	-- pumps read it from contract/library.json (David, 2026-10-05: "Do not assume that the time is the actual time
	-- they took")
	timed_ask_share = 0.5,
	-- a move ended under min_reps reps (a counted set's count too) or min_seconds seconds (a hold's, or a timed move's
	-- clock at Done) counts nothing: it is banked without a word (Events.lua Lc.ShortBank; the apps' own pumps read both
	-- from contract/library.json; Rapha, 2026-11-03: "so we dont have records e.g. saying 1 s or 1 rep ... only count
	-- from 3 reps and 7 seconds as valid"), but a first max set is never banked (ns.FirstMaxSet)
	min_reps = 3, min_seconds = 7,
	-- a hold the app counted (the stopwatch) saves at most that count times 1 + count_over_share, rounded down, never
	-- under min_seconds; a rep move it counted (the reps the figure ended by Done, at the figure's pace) at most that
	-- share of the higher of that count and the number it has to beat (the higher of the goal and the record), never under
	-- min_reps, and one with no number to beat (a first max set, a placement) one set's most; the apps' own pumps read
	-- it from contract/library.json (slice 66; Rapha, 2026-11-04: "dont allow it to do more than 50% more of what the
	-- app counted"; David, 2026-11-04: "you fix and decide on the best")
	count_over_share = 0.5,
	-- an ask after a move (the whole move's, the goal pose's, the pace's, the hold's, a counted set's answers, the
	-- count) takes no digit and no click this long after it shows, so the key or the click that ended the move never
	-- answers it (2026-10-31); the apps' settle wait, contract/design/tokens.json motion.settle, is the same half second
	answer_settle_s = 0.5,
	-- a hold's seconds pour (slice 65, Rapha, 2026-11-03; the apps' contract/design/tokens.json motion.pour, pourFly
	-- and stepReveal): one grain a second held leaves the dial pour_hold_s after the save, their starts spread over
	-- min(pour_s, pour_base_s + pour_step_s a second held) and closer as they go, each flying pour_fly_s into its
	-- second's place on the hold bar with a tick, none closer than pour_tick_gap_s after the one before; a step-up's
	-- reveal opens the next step at reveal_open_s and folds at reveal_fold_s, at reveal_fold_colour_s for a new colour
	pour_s = 1.425, pour_hold_s = 0.19, pour_base_s = 0.525, pour_step_s = 0.016, pour_fly_s = 0.42,
	pour_tick_gap_s = 0.028, reveal_open_s = 1.24, reveal_fold_s = 3.0, reveal_fold_colour_s = 3.4,
	undo_min = 5, hard_set_note = 10,
	-- Correct a count (slice 44, 2026-10-18; Progress.lua ns.Correctable): the sets of the last correct_days game days
	-- that the game's ring still holds, the newest correct_sets of them listed, the record's set too where older; the
	-- game, the phones and the desktop app read the same (contract/library.json)
	correct_days = 14, correct_sets = 10,
	titles_retired_ts = 1791072000, -- 2026-10-04 00:00 UTC: rows after it earn no retired title (Stats.lua, ns.Titles)
	-- 2026-10-01 00:00 UTC: an unfinished move written on its own with quest 0 before it was a level-up's (none is made
	-- since 2026-09-30); after it, one that is no death tax's joins the pump in progress (Stats.lua logMove)
	ding_retired_ts = 1790812800,
	-- XP (slice 3; the per-move part also drives slice 1's preview; xp_first_try and stat_first_try pay only the
	-- first-try beats of rows written before 2026-10-03, when a first appearance stopped aiming at a number). A cardio or
	-- flow move pays what an easy strength move pays (cardio 20 and flow 10 until 2026-10-03), so the focus moves no
	-- base pay: the gaps left come from the wins and the cardio rungs a mix earns (`make sim`). The rung's share stops
	-- at rung_cap (x 1.7 at rung 8; slice 22, when the ladders grew past eight rungs)
	xp_move = { [1] = 15, [2] = 30, [3] = 40, cardio = 15, flow = 15, rung_step = 0.1, rung_cap = 8 },
	xp_record = 25, xp_first_try = 15, xp_lastbeat = 10, xp_star = 40, xp_rung_up = 100, xp_first_pattern = 20,
	xp_first_pump = 30, xp_ding = 50, xp_welcome = 50, welcome_after_days = 2, xp_vault_goal = 150,
	streak_step = 0.025, streak_cap = 10,
	-- rested XP fills while the player is away (3 % of a level a day, up to WoW's 1.5 levels) and doubles the gains
	-- after it until it is used; every gain counts in full (the 600-a-day line went on 2026-10-03). The rate keeps a
	-- one-evening player's pace where the line had it (level 60 in about 58 weeks, `make sim`); WoW's own 15 % a day
	-- would double every evening of a daily player, whose level takes days here and hours there
	rested_per_day = 0.03, rested_min_h = 1, rested_cap_levels = 1.5,
	xp_base = 70, xp_exp = 1.1, level_cap = 60, paragon_xp = 10000,
	day_offset_h = 4,
	-- rest tokens: token_start held from the start (slice 51, Rapha, 2026-10-21: the streak's freeze), one every
	-- token_days active days, token_max held at most, earned and never bought (David, 2026-10-09: "More tokens"; 7 and 2
	-- until then)
	token_start = 1, token_days = 5, token_max = 3,
	-- the streak's milestones (slice 32): the largest one a game day's streak reaches or passes is that day's milestone
	-- (rest tokens can jump one: 29 to 31 reaches 30), a Result moment of its own in the game and the phones; past a
	-- year a moment each year and at 1,000 days (2026-10-09)
	streak_milestones = { 7, 30, 100, 365, 730, 1000, 1095, 1460, 1825 },
	-- the weekly vault: its moves done (every performed move counts one; 2026-10-27, David: "Instead of pumps count moves
	-- please": 20 pumps finished until then), its records and since slice 55 its embers, three goals of the week's
	-- embers (a day of three, three days, five days), each paying xp_vault_goal; a cardio move done in full lights an
	-- ember, embers_day a game day at most (the cardio minutes, 10, until then: a quiet player reached them in 0 of 22
	-- simulated weeks)
	vault_moves = 40, vault_records = 3, vault_embers = { 3, 9, 15 }, embers_day = 3,
	-- the Long Haul (slice 55): one steady move the player starts from the cardio step's card, at one of long_haul_s's
	-- lengths, the next opening after long_haul_days game days with a Yes at the one open (David's call 1, 2026-10-23:
	-- the one exception to "cardio is short and hard, 20-45 s"); a layoff (ledger_layoff_days, ledger_fade_days) fades
	-- its length to the nearest one, its days started again (ns.HaulPlace)
	long_haul_s = { 90, 120, 150, 180 }, long_haul_days = 2,
	-- stats (slice 3). Since slice 21 a move raises its family's stat (ns.Ladders' `stat`, ns.MoveStat); a move no ladder
	-- holds by its pattern (a ballistic or strike move Agility: the ball slam, since slice 22's power family took the
	-- jumps; the carries Stamina), else the stat of the ladder it scales by, else Spirit. A rung reached pays its
	-- ladder's stat by the steps it climbs (`stat_step` a step; `stat_rung`, 2 a rung, until slice 21), so a colour pays
	-- alike on every ladder, and the cardio progression's step (`stat_p_step`: each new 50 of the steady ledger's best
	-- M since slice 52, each P step until then) pays the cardio family's Agility (Stamina
	-- until slice 21; 6 a step from slice 22, whose record families outgrew a family that kept no records, 3 from
	-- 2026-10-05, when its counted sets gave it records again, 2 in slice 42, when the core's rep moves beside its hold
	-- steps stopped at their own tops and the Stamina records they paid past them went, and 3 again since slice 42 took
	-- main in, 2026-10-17, when the sit-up's Stamina put the Strength focus's Stamina well past its Agility; 4 in
	-- slice 52, whose best M pays a step a week at most and later in the year than P did; 3 again since its
	-- double-check, 2026-10-24: every year reaches the top M, so its steps pay every focus alike, and at 4 the
	-- Flexibility focus ended "of the Tiger" once the week's omen always named a road of the player's pool)
	stat_rule = { ballistic = "agility", strike = "agility", carry = "stamina", other = "spirit" },
	stat_base = 10, stat_record = 1, stat_first_try = 1, stat_star = 1, stat_step = 2, stat_p_step = 3,
	fam_familiar_days = 3, fam_learned_days = 8, stat_fam = { tried = 0, familiar = 1, learned = 2 },
	stat_pattern_new = 1, stat_skill_familiar = 3,
	stat_practice_week = 1, stat_learning_week = 1, stat_rested_day = 1, stat_welcome = 2,
	rating_days = 28, map_steps = { 4, 10 },
	-- a pattern's skill (ns.Skill): min(skill_cap, floor(units ^ skill_exp)), its tiers from these skill lines up
	-- (journeyman, expert, artisan; apprentice below)
	skill_cap = 300, skill_exp = 0.62, skill_tiers = { 75, 150, 225 },
	sheet_log_pumps = 20, -- the pumps the sheet's Log keeps
	crit_base = 5, crit_span = 30, crit_half = 400, crit_day_wins = 3,
	-- a step's colour (slice 20, David 2026-10-03: grey to orange, two steps a colour): Enum.ItemQuality, Poor 0 (shown
	-- as "Starter"), Common 1, Uncommon 2, Rare 3, Epic 4, Legendary 5, and since 2026-10-05 Artifact 6 above orange
	-- (orange the clip's freestanding handstand push-up, the elite skills gold); each ladder's rungs name their step
	-- (Progress.lua)
	quality_by_step = { 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6 },
	ring_rows = 2000, ring_batch = 100,
	log_lines = 500, log_batch = 50, log_show = 15, log_line_chars = 300, -- the addon's log (Log.lua)
	voice_recheck_s = 30, -- a silent voice is named only when it is still silent this much later (Speech.lua)
	-- the voice's callouts (2026-10-09; Stats.lua ns.PickCallout): at a move's Ready view, after its name and cue, at
	-- most one recorded line from facts the engine knows, the first of callout_order that holds (the most specific
	-- first: the streak's milestone day and Welcome back on a pump's first move, one more day at the top before the next
	-- step, a move that can give an orange or gold step its proof, a first max set, a record attempt on a ladder fresh
	-- again after a day of repeats, a record attempt, the last move of a pump of callout_last_moves or more), in
	-- callout_takes takes each (each milestone its own), one other than the take played last; callout_gap_s after the
	-- move's line, between the guide's How and its Tips, and after a line said over the guide before it goes on
	callout_order = { "day", "back", "unlock", "proof", "first", "fresh", "record", "last" },
	callout_takes = { day = 2, back = 3, unlock = 3, proof = 3, first = 3, fresh = 3, record = 4, last = 3, shout = 3 },
	callout_last_moves = 3, callout_gap_s = 0.4,
	-- a move's guide line stopped (a pause, a line said over it, its move's end) fades out over this (StopSound's fade)
	guide_fade_ms = 200,
	-- a duel's daily caps (slice 23): the pumps scored a game day, those of them with no quest, the cardio minutes
	duel_pumps_day = 5, duel_started_day = 1, duel_cardio_min_day = 20,
	-- a duel's game days, its rivals at most, the QP levels apart a challenge may cross (a Paragon player stands at the
	-- cap), its points (a scored pump, an active day, a cardio minute) and how long after its last day the rivals' final
	-- cards may come; an offer of a pact or a duel stands this long (its popup's timeout)
	duel_days = 7, duel_max = 5, duel_band = 10, duel_pump_pts = 10, duel_active_pts = 5, duel_cardio_pts = 1,
	duel_grace_days = 7, invite_s = 120,
	-- the streamer kit (slice 23): entering a rested area after this many pumps in a game day opens the session card,
	-- once a game day; the virtual partner's bar runs this far ahead of the player's own on a hold
	session_card_pumps = 3, partner_lead_s = 1,
	-- the social core (slice 13): the card protocol's probes and its own token bucket, the jitter of the hellos and
	-- their replies, the cards the session keeps, the HUD's social lines, the borders and the guild's weekly milestones
	probe_s = 30, reprobe_s = 300, bucket_burst = 10, bucket_rate = 1,
	hello_jitter_s = 10, peer_max = 500,
	join_line_s = 60,
	cheer_line_s = 120, cheer_digest_peers = 10,
	border_levels = { 10, 20, 40, 60 }, guild_milestones = { 100, 250, 500, 1000 },
	qr_max_bytes = 500,
	-- the feedback button (slice 24, contract/feedback-api.yaml): the text's most characters, the game days its numbers
	-- count over (today and the 13 before), the most moves they list
	feedback_max_chars = 500, feedback_days = 14, feedback_moves_max = 40,
	-- the phones' reminders and sitting cue (slice 32) and since 2026-10-09 the desktop app's, exported for them as the
	-- feedback caps are (the addon reads none): the daily cap's default and its most, quiet hours' default and the streak
	-- reminder's default time in minutes after local midnight, the most reminders a player sets, Android's window from a
	-- reminder's time, the retry of a streak reminder held back by a running pump or a moving player, the days the
	-- iPhone plans ahead; the streak reminder after remind_taper_days game days with no performed move only every
	-- remind_taper_every game days, and from remind_weekly_days once a week; the sitting cue after an hour with fewer
	-- steps than sit_steps or an idle stand hour (on the computer an hour of input), at most one each sit_after_min;
	-- since 2026-10-27 the computer's streak reminder only once QuestPump there has run remind_run_min in this run
	remind_cap = 3, remind_cap_max = 6, remind_quiet_from_min = 1320, remind_quiet_to_min = 480, remind_streak_min = 1140,
	remind_own_max = 5, remind_window_min = 30, remind_retry_min = 30, remind_plan_days = 7,
	remind_taper_days = 7, remind_taper_every = 3, remind_weekly_days = 21, remind_run_min = 60,
	sit_after_min = 60, sit_steps = 250,
	-- the sitting clock's time away (the desktop app's since slice 48; the work clocks' on every device since slice
	-- 67): a pause in the input this long starts the minutes sat again at zero (the addon reads none)
	sit_away_s = 300,
	-- work mode (slice 67), exported for the desktop app, the phones and the browser (the addon reads none): a work pump
	-- is due once the work clock counts work_every_min minutes of the player's time at it (the player picks one of
	-- work_every_choices); every work_set_blocks-th pump of a set is the long one, one move more; once due and nothing
	-- holds it, the ask comes at the first pause of work_moment_s seconds without input, at most work_late_s seconds
	-- after it was due; a work pump opens on its warm move only once the device's sitting clock holds work_warm_min
	work_every_min = 25, work_every_choices = { 25, 30, 45, 60 }, work_set_blocks = 4, work_moment_s = 30,
	work_late_s = 120, work_warm_min = 60,
	-- the account's unfinished list (slice 67): an owed row a device without a computer banks (the browser, a phone not
	-- linked to the desktop app) carries in its questID a bank number from bank_quest_min up, never a quest's id
	-- (contract/sync-api.yaml, "The account's unfinished list"), so two devices' banks of one second keep apart and
	-- Lc.QuestTitle asks no title of it
	bank_quest_min = 1073741824,
	-- the phones' game links (slice 33), exported for them as the reminders are (the addon reads none): a look at every
	-- link's feed each link_look_s while the app is in front, the background refresh's earliest spacing in minutes, the
	-- age past which a finish offers nothing, the least spacing between two game offers, and the age from which an
	-- offer says how long ago the game or the stream finished
	link_look_s = 60, link_refresh_min = 15, link_offer_max_min = 60, link_gap_min = 60, link_late_min = 10,
	-- the games on the computer (slice 35), exported for the desktop app as the links' are (the addon reads none): the
	-- match-over card's life and the match_end command's on the held link in minutes, the least a picked program's
	-- process runs before its close offers (since slice 57's fix round also the least a feed's match runs before its
	-- end offers: a shorter one offers nothing, its match going on), and the wait after a close in which a relaunch
	-- continues the match (and, since slice 48, takes its card back); since slice 48 (docs/computer-games.md) the
	-- minutes sat at the computer from which a match pump opens on its warm move (since slice 57 that alone: every
	-- match asks; since its fix round counted from a move or a time away, never from an offer), the minutes a size
	-- step takes (half up, Small to Big; since slice 57 of the match's own minutes, of the minutes sat until then; 12
	-- since slice 59, 20 until then), and the least League of Legends' game process runs before its close offers (a
	-- remake's vote runs to 2:25 of game time; the process alone, never its lead); since slice 59 the minutes before
	-- League's game process started from which its match counts (champion select, which runs in the client the app
	-- never hears), never from before League's previous close, the latest move or the latest return to the computer;
	-- since slice 48's double-check the seconds of no input at the computer while a match offer's card shows after
	-- which the player has left the desk and the offer goes to the phone and the watch; since 2026-10-28 the least a
	-- checkpoint game's feed finish (a mission handed in) waits from the game's start or the last that offered; since
	-- 2026-11-04 (Rapha, after an MTG Arena match under 5 minutes offered nothing: "drop it to 2 min", "also for other
	-- games") the least is 2 minutes (5 until then), and a match under match_short_min minutes composes its pump at the
	-- Light pace, one move ("2 min to 5 be 1 move pump"), on every device
	match_offer_min = 10, match_min_s = 120, match_relaunch_s = 90, match_short_min = 5,
	match_sat_min = 20, match_step_min = 12, match_min_league_s = 480, match_lead_league_min = 5, match_away_s = 120,
	match_checkpoint_min = 20,
	-- the games and apps on the phone (docs/phone-games.md, 2026-10-16), exported for the phones as the links' are (the
	-- addon reads none): a watched game away for less than phone_rejoin_s keeps its session, a session shorter than the
	-- player's Shortest session (its default and its most in minutes) offers nothing, a game's minutes per match before
	-- any answer and the answers its guess takes the median of, the most matches the "Good game!" sheet counts, and the
	-- minutes Pump first's "Open for 15 minutes" opens an app that waits for today's pump
	phone_rejoin_s = 120, phone_min_default_min = 10, phone_min_max_min = 60, phone_match_guess_min = 20,
	phone_match_answers = 5, phone_matches_max = 9, pump_first_open_min = 15,
	-- the sheet imports (slice 44), exported for the phones as the links' are: a pasted, scanned, linked or shared text
	-- longer than import_max_chars characters, one that unpacks past import_max_unpacked bytes, or one whose CBOR
	-- decodes to more than import_max_items items, is refused, on every way in; the game reads import_max_chars alone,
	-- its own import's cap in bytes (DB.lua ns.Import; a sheet string is ASCII; 2026-10-31)
	import_max_chars = 1048576, import_max_unpacked = 4194304, import_max_items = 262144,
	-- a row from outside a device's own writing (2026-10-31, the release's security review): one dated before
	-- row_min_ts, 2026-09-21 00:00 UTC, before QuestPump wrote a row, or more than row_ahead_s past the reading device's
	-- clock is refused, and a reset, Start over or felt-unwell row, which floors or locks a save until its game day ends,
	-- more than row_ahead_floor_s past it, so no crafted one in the future floors or locks a save for long (DB.lua
	-- ns.RowTimely: an import string whole, the mirror's row by row); the helper's store and the phones' take-ins read
	-- the same through contract/library.json
	row_min_ts = 1789948800, row_ahead_s = 86400, row_ahead_floor_s = 600,
	-- the apps' unfinished lists (2026-10-09), exported for them as the reminders are (the addon reads none): a move
	-- unfinished for more than this many game days stays listed and rides the next game pump, but puts no dot on the
	-- Pump tab and makes Catch up no primary
	owed_fresh_days = 7,
	-- the Saga's day (2026-10-15; contract/fixtures/history.json `saga`), exported for the phones and the desktop app
	-- and read by the game's fold: a day gets a quality word or a best day only after this many earlier game days with a
	-- move (so a new player's every day is no "best day"); its quality looks back over the game days in the window of
	-- day_rank_days calendar days before it, and is Epic, Rare or Uncommon while fewer than day_rank_shares percent of
	-- that window's days have more moves (else Common). Shown, never awarded: no XP, title or badge (safety.md hazard 12)
	best_day_min_days = 7, day_rank_days = 56,
	day_rank_shares = { 10, 25, 50 },
	-- the sheet summary's dx (the Saga's fix review, 2026-10-15; history.json saga rule 15): the XP of this many game
	-- days at most, the sheet's own included, so a sheet the strip paints and the apps keep is as small after a year as
	-- after a month; a day before them reads unknown
	dx_days = 60,
}
