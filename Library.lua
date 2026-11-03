-- The move catalogue, data only (docs/plan.md slice 2, "The exercise library"). Ids are stable uint16 and never reused
-- (retired 2026-10-02: 63, 105, 106, 107, 110, 116, 117; 2026-10-03: 6, the victory dance, and 1, marching in place,
-- "just too boring", whose part as the floor went to the step jacks, and 104, the farmer march, "too easy"; 2026-10-04:
-- 3, the brisk march, "too lame", whose place as the cardio ladder's rung 2 went to shadow boxing, and the cardio
-- forms done at a talking pace, "too easy and feel like a timewaster" (David: "you want the excercises to count as you
-- leave your computer for them"): 4, the high-knee march, 5, the shuttle, 139, the standing strokes, 181, the stair
-- walk, 205, the vest march, 220, the backpack march, and 240, the power march; merged into another move on
-- 2026-10-04: 3, 4, 5, 139, 140, 160, 167, 168, 181, 205, 220, 226, 233 and 240, ns.Merged below; 2026-10-26: 16, the
-- wall sit at 135°, David: "I also feel we can remove the shallow wall sit as first. the quarter one as first is enough
-- no?", whose ladder starts at the quarter sit since, ns.Shifted below); the string keys are for ns.L only (move.<key>,
-- .setup, .cue, .stop) and outlive a new name (backpack_row is the one-arm row with any weight).
-- Every move is tagged from a closed vocabulary (tools/lint.lua checks it), so the engine acts on tags, never on names:
--
-- group     the Exercises page's group: squat lunge hinge push pull core situp calf cardio carry mobility yoga, and
--           the seventeen equipment sets (bar bands kettlebell dumbbells barbell rope bike stairs pad
--           step wheel rings vest ball roller backpack pole)
-- pattern   squat hinge lunge push pull core_anti_ext core_anti_rot core_flex carry iso mobility balance ballistic
--           cardio run calf strike (core_flex: the trunk curled in reps, the sit-up's, 2026-10-17)
-- region    legs push pull trunk (rep and hold moves; a pump's own moves take one record move a region), nil
--           otherwise
-- class     1-3 (L1-L3); dose rep | hold | cardio | flow; secs: a flow move's seconds, each side's on a side move
--           (2026-10-01: 30 s each, the top of ACSM's 10-30 s a stretch, Garber 2011). A rep or hold move ends at
--           form failure well inside censor_s for most players; a move whose limit is the load or the patience (a
--           carry, the unloaded good morning) is a flow move that pushes through its load or its tempo instead
-- ladder    ns.Enum.ladder (Core.lua): 1 push, 2 squat, 4 hinge, 6 cardio, 7 wall sit, 8 pull-up, 9 row, 10 core,
--           11 side, 12-19 the stretching and yoga families (slice 21; 3 pull and 5 core hold retired on 2026-10-04),
--           20 handstand, 21 L-sit, 22 planche, 23 power (slice 22), 24 sit-up (2026-10-17);
--           rung: its place on that ladder, or on its region's ladder scale for a move off the ladders; anchor: the
--           move whose record (a pose's mark) decides the rung
-- regress, progress, twin   move ids; twin: the substitute when the move needs floor, space, a stable chair, a table,
--           the towel and door (2026-10-28) or equipment (an anchor that needs one of the first five has one wherever
--           its ladder holds a move without them at or under its rung: tools/lint.lua), the rung's own move without
--           them first where one exists
--           (2026-10-01), then the stand-ins from below, which stand for the rung but never climb to it or from it
--           (ns.CanRungUp, ns.CanClimb); a chain of twins ends where the ladder tops out
-- flags     floor loud space tendon impact strain head_down skill
-- pain      wrist elbow knee achilles back shoulder neck hip (the player's pain flags remove the tagged moves)
-- needs     equipment, each entry required; an entry that is a list is "any of": { { "stairs", "step" } }
-- theme     bear crab frog duck worm spider axe pick oar bow hammer climb swim carry dig run sneak fight ride
-- road      every cardio-ladder form's and every finisher form's (slice 54, 2026-10-23; ns.ROADS): run strike jump
--           floor standing, and gear for the forms that need equipment; never shown, it orders the cardio picks' fair
--           draw (Engine.lua) and a cardio step's forms on its card (Tree.lua)
-- facing    every hold's of the core and side ladders (slice 54; ns.FACINGS): front, side (the side ladder's) or back,
--           the side of the trunk it holds against; never shown, it orders the four winds (Engine.lua)
-- finisher  the lowest cardio rung at which the move is an all-out form; endOnly: only ever the all-out end of a move;
--           count: true on a countable form (2026-10-05): as an all-out window it is a counted set, 30 s all out from
--           its go, its count typed if the player wants it and the window saved Yes, to the end, Form slipped or Not
--           all out (slice 53); its
--           best clean count is its record, never a goal, and its clean windows count the clean days that open the next
--           cardio rung, a fresh one of an orange rung's own form earning it (2026-10-16, slice 42; until then
--           { lo, hi }, its window of counts in 30 s, whose top reached twice an hour apart opened the rung)
-- crung     the cardio rung an impact move off the cardio ladder needs (on the power ladder too); side: one side,
--           then the other, each side its own run with the countdown again between them ("Switch sides"): a timed
--           move runs secs on each, a hold past its goal until Done (David, 2026-10-04), a move of reps every rep on
--           one side until the player presses Switch sides, the weaker side's count typed (standing, at a desk, a door
--           or a bar since 2026-10-10; on the floor or lying down too since 2026-10-13, David: "single leg bridge
--           should have switch side button", where both sides ran as one set from 2026-10-03 so that nobody got up
--           mid-set to reach the computer); pairs: a move of reps whose sides take turns every rep (the bird dog, the
--           archer push-ups), no side move: one count is a rep on each side, its figure's loop, so its stopwatch
--           covers both sides (ns.Classify; David, 2026-10-12: "Or I should alternate but then that needds to be
--           said clearer")
-- support   "chair" or "table": the body rests on it (the lint wants the matching need); "bed": it rests on a bed or
--           a sofa, which every home has (the plan counts one as given since 2026-10-01), so no checkbox asks for it
--           and the lint wants no stable_chair need; jump: both feet leave the floor (the lint wants impact);
--           strainFree: a named hold that is not taken to a breath-holding effort
-- load      "total" or "each" on a rep move that takes a typed weight in kg (2026-10-09, David: "Let people put their
--           exact weight (kgs) and how many reps they made and suggest the progressive overload next time"): "each" a
--           weight in each hand, the kg of one; kg: { lo, hi }, the whole kg a first set starts with, about the
--           15-20-rep weight of beginner to novice lifters on Strength Level's standards (2026-10-09), a hint only
-- holds     the gear that carries the move's load, best first: a real weight takes the place of a makeshift one on the
--           same move, whose setup line then names the one the player has (Locales: move.<key>.setup.<gear>)
-- upgrade   moves that take this move's place in a pump whose pool holds one of them, the same exercise with a better
--           tool (2026-10-09; never on a bodyweight move, an anchor or a move of a rung's chain: tools/lint.lua)
-- optIn     off until the player checks it on the Exercises page (the profile's `include`; the headstand, slice 21)
-- noGoal    a pose of a stretching or yoga family that names no goal pose (slice 47, 2026-10-19): one done as a
--           movement, never held in one shape (hip circles, cat-cow, the roller's rolls), or one that trains none of
--           its family's measure (warrior II and chair pose stand on two feet in the balance family, the child's pose
--           points the feet in the ankle family); practice alone, never asked and never a day (ns.HasGoal); every other
--           pose of a family but an opt-in one has a goal pose
-- checks    what the move waits for before a pump, Swap or a Start offers it (2026-10-16, slice 42), in a rung need's
--           shape (Progress.lua): the step of another ladder (a one-time stretch check where it names a pose family's
--           step or pose), a named move's record, or `any` of a list; done once, kept
-- hint      the pose that helps the move, a soft prerequisite and never a gate (2026-10-17, slice 42): the move's card
--           names it under "Helps", with the advice line move.<key>.hint where the locale gives one; a pose of a
--           stretching or yoga family (tools/lint.lua)
-- goalSince on a move whose form changed under its id, the moment (Unix seconds) from which its sets measure it
--           (DB.lua ns.BeforeForm): an older row keeps its record and stars and counts toward no recent form, no day
--           at its top, no climb's day at the top and no check (a pose's: no best and no first pass either, markDays,
--           2026-10-17; every changed move's since 2026-10-19): the wall angels' wall test, plank shoulder taps' feet
--           together
-- hrAfter  the hands bear the body through a dynamic set (the push-ups, the burpee family, the inchworm): a
--           wrist's optical heart rate drops out during it, so the heart-rate verdict scores the 30 s after it (the
--           helper reads it from contract/moves.json, slice 8); holds are scored from the go, as every isometric one
-- mp, ms    the primary and secondary muscles (glossary ids; 17 Heart, 18 Breath, 19 Balance): the body map, the pump
--           card and the tooltips only; the engine selects by pattern and region, never by muscle
--
-- The move figures are slice 10's pre-rendered 3D frames (David, 2026-09-24), looked up by move id.

local ADDON, ns = ...

local Sh, Ch, Bi, Fa, Ab, Ob, Qu, Ad, Nk, UB, La, Tr, LB, Gl, Ha, Ca = 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14,
	15, 16
local HEART, BREATH, BALANCE = 17, 18, 19
local LAD, MARK = ns.Enum.ladder, ns.Enum.mark

-- Fills the empty lists every entry carries, so a lookup never meets nil; it adds nothing else.
local function move(t)
	t.flags, t.pain, t.theme, t.ms = t.flags or {}, t.pain or {}, t.theme or {}, t.ms or {}
	return t
end

ns.Library = {
	-- Squat
	-- The sit to stand rests on a bed or sofa edge (2026-10-12; a stable chair until then, which the default setup
	-- shuts, so rung 1 climbed on the quarter squat, which never reaches chair depth).
	[7] = move({ key = "chair_stand", group = "squat", pattern = "squat", region = "legs", class = 1, dose = "rep",
		ladder = 2, rung = 1, anchor = true, progress = 2, support = "bed", mp = { Qu, Gl }, ms = { Ca, Ad } }),
	[8] = move({ key = "quarter_squat", group = "squat", pattern = "squat", region = "legs", class = 1, dose = "rep",
		ladder = 2, rung = 1, progress = 2, mp = { Qu, Gl }, ms = { Ca, Ad } }),
	[2] = move({ key = "chair_squat", group = "squat", pattern = "squat", region = "legs", class = 1, dose = "rep",
		ladder = 2, rung = 2, anchor = true, regress = 7, progress = 9, mp = { Qu, Gl }, ms = { Ad } }),
	[9] = move({ key = "air_squat", group = "squat", pattern = "squat", region = "legs", class = 1, dose = "rep",
		ladder = 2, rung = 3, anchor = true, regress = 2, progress = 10, hint = 271, pain = { "knee" }, mp = { Qu, Gl },
		ms = { Ad } }),
	[10] = move({ key = "tempo_squat", group = "squat", pattern = "squat", region = "legs", class = 2, dose = "rep",
		ladder = 2, rung = 4, anchor = true, regress = 9, progress = 20, pain = { "knee" }, mp = { Qu, Gl },
		ms = { Ad } }),
	[11] = move({ key = "sumo_squat", group = "squat", pattern = "squat", region = "legs", class = 1, dose = "rep",
		ladder = 2, rung = 3, regress = 2, pain = { "hip" }, mp = { Qu, Ad, Gl } }),
	[13] = move({ key = "squat_punch", group = "squat", pattern = "strike", region = "legs", class = 2, dose = "rep",
		ladder = 2, rung = 3, regress = 2, pain = { "elbow" }, theme = { "fight" }, mp = { Qu, Gl },
		ms = { Sh, Ob, HEART } }),
	[14] = move({ key = "onehalf_squat", group = "squat", pattern = "squat", region = "legs", class = 2, dose = "rep",
		ladder = 2, rung = 4, regress = 9, pain = { "knee" }, mp = { Qu, Gl }, ms = { Ad } }),
	[15] = move({ key = "deep_squat_hold", group = "squat", pattern = "iso", region = "legs", class = 1, dose = "hold",
		ladder = 2, rung = 3, regress = 2, strainFree = true, pain = { "knee" }, mp = { Qu }, ms = { Gl, Ad, Ca } }),
	[20] = move({ key = "split_squat", group = "squat", pattern = "lunge", region = "legs", class = 2, dose = "rep",
		ladder = 2, rung = 5, anchor = true, regress = 10, progress = 21, side = true, pain = { "knee" },
		mp = { Qu, Gl }, ms = { Ad, Ha, BALANCE } }),
	-- The Bulgarian's rear foot rests on a bed or a sofa (David, 2026-10-04; a stable chair until then): the rear foot
	-- carries about a sixth of the load (Helme 2022), and the slow split squat stands in where the move is unchecked.
	[21] = move({ key = "bulgarian", group = "squat", pattern = "lunge", region = "legs", class = 2, dose = "rep",
		ladder = 2, rung = 6, anchor = true, regress = 20, progress = 23, twin = 22, side = true, support = "bed",
		hint = 245, pain = { "knee" }, mp = { Qu, Gl }, ms = { Ad, Ha, BALANCE } }),
	[22] = move({ key = "split_squat_slow", group = "squat", pattern = "lunge", region = "legs", class = 2,
		dose = "rep", ladder = 2, rung = 6, regress = 20, progress = 23, side = true, pain = { "knee" },
		mp = { Qu, Gl }, ms = { Ad, Ha, BALANCE } }),
	-- the pistol with both hands on the desk beside the Bulgarian (2026-10-12; Hybrid Calisthenics' assisted one-leg
	-- squat): full depth on one foot, heel flat, the free leg up, the arms taking what the leg cannot yet
	[391] = move({ key = "pistol_two_hands", group = "squat", pattern = "squat", region = "legs", class = 3,
		dose = "rep", ladder = 2, rung = 6, regress = 22, progress = 23, side = true, flags = { "skill" },
		checks = { { ladder = LAD.ankle, step = 2 } }, pain = { "knee" }, mp = { Qu, Gl }, ms = { Ca, Ab, BALANCE } }),
	[23] = move({ key = "assisted_pistol", group = "squat", pattern = "squat", region = "legs", class = 3, dose = "rep",
		ladder = 2, rung = 7, anchor = true, regress = 391, progress = 24, side = true, flags = { "skill" },
		pain = { "knee" }, mp = { Qu, Gl }, ms = { Ca, Ab, BALANCE } }),
	-- Beside the assisted pistol (2026-10-10, David: "You should be strong enough to easily handle the next rung"): the
	-- pistol to a bed or sofa edge, no hands (the box pistol), and the one-leg lowering in 5 s (the eccentric pistol;
	-- Rep Fitness, Breaking Muscle), each on its own window, the pistol's balance and its bottom, which the desk takes
	-- off the assisted one.
	[375] = move({ key = "box_pistol", group = "squat", pattern = "squat", region = "legs", class = 3, dose = "rep",
		ladder = 2, rung = 7, regress = 23, side = true, flags = { "skill" }, support = "bed", pain = { "knee" },
		mp = { Qu, Gl }, ms = { Ca, Ab, BALANCE } }),
	[376] = move({ key = "pistol_lowering", group = "squat", pattern = "squat", region = "legs", class = 3,
		dose = "rep", ladder = 2, rung = 7, regress = 23, side = true, flags = { "skill" }, pain = { "knee" },
		mp = { Qu, Gl }, ms = { Ca, Ab, BALANCE } }),
	[24] = move({ key = "pistol", group = "squat", pattern = "squat", region = "legs", class = 3, dose = "rep",
		ladder = 2, rung = 8, anchor = true, regress = 23, side = true, flags = { "skill", "strain" },
		pain = { "knee" }, mp = { Qu, Gl }, ms = { Ca, Ab, BALANCE } }),
	-- Past the pistol (slice 22): the shrimp squat (the back foot held off the floor behind you, the knee down to a
	-- cushion), then with the foot held in both hands, then the dragon pistol, the free leg threaded behind the
	-- standing one. Standing moves, so the squat reaches orange with no equipment and no floor space.
	[302] = move({ key = "shrimp_squat", group = "squat", pattern = "squat", region = "legs", class = 3, dose = "rep",
		ladder = 2, rung = 9, anchor = true, regress = 24, side = true, flags = { "skill", "strain" },
		pain = { "knee", "hip" }, mp = { Qu, Gl }, ms = { Ha, Ca, BALANCE } }),
	[303] = move({ key = "pistol_pause", group = "squat", pattern = "squat", region = "legs", class = 3, dose = "rep",
		ladder = 2, rung = 9, regress = 24, side = true, flags = { "skill", "strain" }, pain = { "knee" },
		mp = { Qu, Gl }, ms = { Ca, Ab, BALANCE } }),
	[304] = move({ key = "shrimp_squat_held", group = "squat", pattern = "squat", region = "legs", class = 3,
		dose = "rep", ladder = 2, rung = 10, anchor = true, regress = 302, side = true, flags = { "skill", "strain" },
		pain = { "knee", "hip" }, mp = { Qu, Gl }, ms = { Ha, Sh, BALANCE } }),
	-- the dragon pistol with one hand on the desk, beside the held shrimp squat (2026-10-10): nothing below it trains the
	-- crossed leg
	[377] = move({ key = "dragon_pistol_assisted", group = "squat", pattern = "squat", region = "legs", class = 3,
		dose = "rep", ladder = 2, rung = 10, regress = 24, side = true, flags = { "skill", "strain" },
		checks = { { ladder = LAD.midsplit, step = 5, move = 440, mark = MARK.good }, { ladder = LAD.ankle, step = 6 } },
		pain = { "knee", "hip" }, mp = { Qu, Gl }, ms = { Ad, Ab, BALANCE } }),
	[305] = move({ key = "dragon_pistol", group = "squat", pattern = "squat", region = "legs", class = 3, dose = "rep",
		ladder = 2, rung = 11, anchor = true, regress = 377, side = true, flags = { "skill", "strain" },
		pain = { "knee", "hip" }, mp = { Qu, Gl }, ms = { Ad, Ab, BALANCE } }),

	-- Wall sit, its own ladder by knee angle: 120° (the quarter sit), 105°, 90° (strain from 105°), one leg at 105° and
	-- 90°; the 135° sit below the quarter sit went on 2026-10-26 (16, retired above)
	[17] = move({ key = "wall_sit_120", group = "squat", pattern = "iso", region = "legs", class = 1, dose = "hold",
		ladder = 7, rung = 1, anchor = true, progress = 18, strainFree = true, pain = { "knee" }, mp = { Qu },
		ms = { Gl } }),
	[18] = move({ key = "wall_sit_105", group = "squat", pattern = "iso", region = "legs", class = 2, dose = "hold",
		ladder = 7, rung = 2, anchor = true, regress = 17, progress = 19, flags = { "strain" }, pain = { "knee" },
		mp = { Qu }, ms = { Gl } }),
	[19] = move({ key = "wall_sit_90", group = "squat", pattern = "iso", region = "legs", class = 2, dose = "hold",
		ladder = 7, rung = 3, anchor = true, regress = 18, progress = 289, flags = { "strain" }, pain = { "knee" },
		mp = { Qu }, ms = { Gl } }),
	-- Beside the 90° sit (slice 54, 2026-10-23): the punches, the same sit with both fists working, and the staggered
	-- sit, one foot a half step ahead with most of the load on it, the path stage between two legs and one (the ladder
	-- jumped from two legs at 90° to one leg at 105°), timed a side.
	[642] = move({ key = "staggered_wall_sit", group = "squat", pattern = "iso", region = "legs", class = 2,
		dose = "hold", ladder = 7, rung = 3, regress = 19, progress = 289, side = true, flags = { "strain" },
		pain = { "knee" }, mp = { Qu }, ms = { Gl, BALANCE } }),
	[643] = move({ key = "wall_sit_punches", group = "squat", pattern = "iso", region = "legs", class = 2,
		dose = "hold", ladder = 7, rung = 3, regress = 19, flags = { "strain" }, pain = { "knee", "elbow" },
		theme = { "fight" }, mp = { Qu }, ms = { Gl, Sh } }),
	[289] = move({ key = "sl_wall_sit_105", group = "squat", pattern = "iso", region = "legs", class = 2, dose = "hold",
		ladder = 7, rung = 4, anchor = true, regress = 19, progress = 290, side = true, flags = { "strain" },
		pain = { "knee" }, mp = { Qu }, ms = { Gl } }),
	[290] = move({ key = "sl_wall_sit_90", group = "squat", pattern = "iso", region = "legs", class = 2, dose = "hold",
		ladder = 7, rung = 5, anchor = true, regress = 289, side = true, flags = { "strain" }, pain = { "knee" },
		mp = { Qu }, ms = { Gl } }),

	-- Lunge and single leg
	[26] = move({ key = "reverse_lunge", group = "lunge", pattern = "lunge", region = "legs", class = 2, dose = "rep",
		ladder = 2, rung = 4, regress = 29, side = true, pain = { "knee" }, theme = { "sneak" }, mp = { Qu, Gl },
		ms = { Ad, Ha, BALANCE } }),
	-- the lateral and the curtsy lunge on rung 4 since 2026-10-16 (slice 42): mostly one leg, as the reverse lunge beside
	-- them, which stands in for the lateral lunge where room to move is off
	[27] = move({ key = "lateral_lunge", group = "lunge", pattern = "lunge", region = "legs", class = 1, dose = "rep",
		ladder = 2, rung = 4, twin = 26, side = true, flags = { "space" }, pain = { "knee", "hip" },
		mp = { Ad, Gl, Qu } }),
	[28] = move({ key = "curtsy_lunge", group = "lunge", pattern = "lunge", region = "legs", class = 2, dose = "rep",
		ladder = 2, rung = 4, side = true, pain = { "knee", "hip" }, mp = { Gl, Qu }, ms = { Ad } }),
	[29] = move({ key = "knee_drive", group = "lunge", pattern = "lunge", region = "legs", class = 1, dose = "rep",
		ladder = 2, rung = 2, side = true, mp = { Qu, Gl }, ms = { Ab, Ca, BALANCE } }),
	[136] = move({ key = "spider_lunge", group = "lunge", pattern = "lunge", region = "legs", class = 1, dose = "rep",
		ladder = 2, rung = 3, twin = 26, side = true, flags = { "floor" }, pain = { "knee", "hip", "wrist" },
		theme = { "spider" }, mp = { Qu, Gl }, ms = { Ad, Ha } }),
	[135] = move({ key = "duck_walk", group = "lunge", pattern = "squat", region = "legs", class = 2, dose = "hold",
		ladder = 2, rung = 4, twin = 15, strainFree = true, flags = { "space" }, hint = 271, pain = { "knee" },
		theme = { "duck" }, mp = { Qu, Gl }, ms = { Ad, Ca } }),

	-- Hinge: on the floor from the bridge to the Nordics, each rung's twins its ways without the floor or a stable
	-- chair, then the stand-ins from the rungs below.
	-- The unloaded good morning is timed: hands on the hips, a max set runs into the hundreds before the form fails.
	[31] = move({ key = "good_morning", group = "hinge", pattern = "hinge", class = 1, dose = "flow", secs = 40,
		rung = 1, pain = { "back" }, mp = { Ha, Gl }, ms = { LB } }),
	[32] = move({ key = "single_leg_rdl", group = "hinge", pattern = "hinge", region = "legs", class = 1, dose = "rep",
		ladder = 4, rung = 1, side = true, pain = { "back" }, theme = { "dig" }, mp = { Ha, Gl },
		ms = { LB, BALANCE } }),
	[33] = move({ key = "glute_bridge", group = "hinge", pattern = "hinge", region = "legs", class = 1, dose = "rep",
		ladder = 4, rung = 1, anchor = true, progress = 34, twin = 32, flags = { "floor" }, mp = { Gl },
		ms = { Ha, LB } }),
	[34] = move({ key = "single_leg_bridge", group = "hinge", pattern = "hinge", region = "legs", class = 2,
		dose = "rep", ladder = 4, rung = 2, anchor = true, regress = 33, progress = 35, twin = 219, side = true,
		flags = { "floor" }, mp = { Gl }, ms = { Ha, LB } }),
	[35] = move({ key = "hamstring_walkout", group = "hinge", pattern = "hinge", region = "legs", class = 2,
		dose = "rep", ladder = 4, rung = 3, anchor = true, regress = 34, progress = 36, twin = 235,
		flags = { "floor" }, pain = { "back" }, mp = { Ha }, ms = { Gl, Ca } }),
	[36] = move({ key = "sliding_curl", group = "hinge", pattern = "hinge", region = "legs", class = 2, dose = "rep",
		ladder = 4, rung = 4, anchor = true, regress = 35, progress = 37, twin = 287, flags = { "floor" }, mp = { Ha },
		ms = { Gl, Ca } }),
	-- one leg slides out in 3 s, both pull back (2026-10-10; Overcoming Gravity's "1 Leg hamstring slide ecc"): the
	-- single-leg sliding curl's lowering before its pull
	[378] = move({ key = "sl_slide_lowering", group = "hinge", pattern = "hinge", region = "legs", class = 2,
		dose = "rep", ladder = 4, rung = 4, regress = 36, side = true, flags = { "floor" }, mp = { Ha },
		ms = { Gl, Ca } }),
	[37] = move({ key = "sl_sliding_curl", group = "hinge", pattern = "hinge", region = "legs", class = 3,
		dose = "rep", ladder = 4, rung = 5, anchor = true, regress = 378, progress = 38, twin = 288, side = true,
		flags = { "floor" }, mp = { Ha }, ms = { Gl, Ca } }),
	[38] = move({ key = "nordic_assisted", group = "hinge", pattern = "hinge", region = "legs", class = 3,
		dose = "rep", ladder = 4, rung = 6, anchor = true, regress = 37, progress = 39, twin = 37,
		flags = { "floor", "tendon", "strain", "skill" }, needs = { "stable_chair" }, support = "chair",
		pain = { "knee", "wrist" }, mp = { Ha }, ms = { Ca } }),
	-- at the bottom of the Nordic, both hands lifted for 2 s (2026-10-12; Natera's long-length yielding isometric):
	-- the lowering's last part, where control breaks and the assisted Nordic's hands catch
	[392] = move({ key = "nordic_hover", group = "hinge", pattern = "hinge", region = "legs", class = 3,
		dose = "rep", ladder = 4, rung = 6, regress = 427, flags = { "floor", "tendon", "strain", "skill" },
		needs = { "stable_chair" }, support = "chair", pain = { "knee", "wrist" }, mp = { Ha }, ms = { Ca } }),
	-- the same hover with the hands on a stack of books 15-20 cm high, its easier version (2026-10-13, the readiness
	-- check; Natera's Nordic sheet: "Hand supported with small bench to hands on floor"): the lift starts higher in the
	-- lowering, where the lever is shorter
	[427] = move({ key = "nordic_hover_books", group = "hinge", pattern = "hinge", region = "legs", class = 3,
		dose = "rep", ladder = 4, rung = 6, regress = 38, flags = { "floor", "tendon", "strain", "skill" },
		needs = { "stable_chair" }, support = "chair", pain = { "knee", "wrist" }, mp = { Ha }, ms = { Ca } }),
	[39] = move({ key = "nordic_eccentric", group = "hinge", pattern = "hinge", region = "legs", class = 3,
		dose = "rep", ladder = 4, rung = 7, anchor = true, regress = 392, progress = 379, twin = 38,
		flags = { "floor", "tendon", "strain", "skill" }, needs = { "stable_chair" }, support = "chair",
		pain = { "knee", "wrist" }, mp = { Ha }, ms = { Ca } }),
	-- The Nordic pull-back anchors rung 8 since 2026-10-10 (Natera's "Isometric holds with no hand support"): lower as
	-- far as you can still come back from, then pull back up with no hands, which no move below trains; the hold
	-- stays beside it.
	[379] = move({ key = "nordic_pullback", group = "hinge", pattern = "hinge", region = "legs", class = 3,
		dose = "rep", ladder = 4, rung = 8, anchor = true, regress = 39, twin = 39,
		flags = { "floor", "tendon", "strain", "skill" }, needs = { "stable_chair" }, support = "chair",
		pain = { "knee", "wrist" }, mp = { Ha }, ms = { Ca, Gl } }),
	-- the Nordic with a hold on rung 7 since 2026-10-16 (slice 42): the step after the lowering, before the pull-back
	[40] = move({ key = "nordic_hold", group = "hinge", pattern = "hinge", region = "legs", class = 3, dose = "rep",
		ladder = 4, rung = 7, regress = 39, twin = 39, flags = { "floor", "tendon", "strain", "skill" },
		needs = { "stable_chair" }, support = "chair", pain = { "knee", "wrist" }, mp = { Ha }, ms = { Ca } }),
	-- the Nordic curl with one small fingertip push at the bottom (2026-10-12; Natera's band-assisted Nordic, the
	-- load taken off at long lengths): the full Nordic's pull up from the floor, which the pull-back never reaches
	[393] = move({ key = "nordic_fingertip", group = "hinge", pattern = "hinge", region = "legs", class = 3,
		dose = "rep", ladder = 4, rung = 8, regress = 379, flags = { "floor", "tendon", "strain", "skill" },
		needs = { "stable_chair" }, support = "chair", pain = { "knee", "wrist" }, mp = { Ha }, ms = { Ca, Gl } }),
	-- The full Nordic (slice 22): down and up again with no push from the hands, the hinge's orange top; tendon-heavy,
	-- so its unlock asks for three days (ns.UnlockDays).
	[306] = move({ key = "nordic_full", group = "hinge", pattern = "hinge", region = "legs", class = 3, dose = "rep",
		ladder = 4, rung = 9, anchor = true, regress = 393, twin = 379, flags = { "floor", "tendon", "strain", "skill" },
		needs = { "stable_chair" }, support = "chair", pain = { "knee", "wrist" }, mp = { Ha }, ms = { Ca, Gl } }),
	-- The single-leg hip thrust from the sofa is a floor variant beside the sliding curl (2026-10-04; until then it
	-- stood for the Nordic rungs, a glute lift under their colours, and without the Nordics the hinge now ends on the
	-- single-leg sliding curl).
	[41] = move({ key = "hip_thrust_sl", group = "hinge", pattern = "hinge", region = "legs", class = 2, dose = "rep",
		ladder = 4, rung = 4, regress = 34, side = true, flags = { "floor" }, needs = { "stable_chair" },
		support = "chair", load = "total", kg = { 5, 10 }, holds = { "dumbbells", "kettlebell" }, mp = { Gl },
		ms = { Ha, Qu } }),
	-- The hinge without floor space, by the load a leg takes (2026-10-04; a bed or a sofa stands for the floor since
	-- 2026-09-30): the deadlift with a weight at rung 2 (any heavy object in both hands, 10-20 kg to start; the
	-- backpack, kettlebell and dumbbell deadlifts, the chair deadlift and the hinge pick-ups are this one move since
	-- then, ns.Merged) with the bed's single-leg bridge as its other way, the walkout on a bed at rung 3, the
	-- single-leg deadlift with a weight in one hand at rung 4 and its slow version from a book at rung 5; rungs 6 and
	-- up (the Nordics) need the floor. A kettlebell or dumbbells take the bag's place on each (2026-10-09: holds), and
	-- every set takes its weight in kg (load).
	[219] = move({ key = "backpack_deadlift", group = "hinge", pattern = "hinge", region = "legs", class = 2,
		dose = "rep", ladder = 4, rung = 2, regress = 32, twin = 234, pain = { "back", "wrist" }, theme = { "dig" },
		load = "total", kg = { 10, 20 }, holds = { "kettlebell", "dumbbells" }, mp = { Ha, Gl }, ms = { LB, Fa } }),
	[234] = move({ key = "bed_single_leg_bridge", group = "hinge", pattern = "hinge", region = "legs", class = 2,
		dose = "rep", ladder = 4, rung = 2, regress = 32, twin = 32, side = true, flags = { "seated" }, mp = { Gl },
		ms = { Ha, LB } }),
	[235] = move({ key = "bed_hamstring_walkout", group = "hinge", pattern = "hinge", region = "legs", class = 2,
		dose = "rep", ladder = 4, rung = 3, regress = 234, twin = 219, flags = { "seated" }, pain = { "back" }, mp = { Ha },
		ms = { Gl, Ca } }),
	[287] = move({ key = "backpack_sl_rdl", group = "hinge", pattern = "hinge", region = "legs", class = 2,
		dose = "rep", ladder = 4, rung = 4, regress = 32, twin = 235, side = true, pain = { "back", "wrist" },
		theme = { "dig" }, load = "total", kg = { 5, 15 }, holds = { "dumbbells", "kettlebell" }, mp = { Ha, Gl },
		ms = { LB, Fa, BALANCE } }),
	[288] = move({ key = "backpack_sl_rdl_slow", group = "hinge", pattern = "hinge", region = "legs", class = 2,
		dose = "rep", ladder = 4, rung = 5, regress = 287, twin = 287, side = true, pain = { "back", "wrist" },
		theme = { "dig" }, load = "total", kg = { 5, 15 }, holds = { "dumbbells", "kettlebell" }, mp = { Ha, Gl },
		ms = { LB, Fa, BALANCE } }),

	-- Push
	[12] = move({ key = "wall_pushup", group = "push", pattern = "push", region = "push", class = 1, dose = "rep",
		ladder = 1, rung = 1, anchor = true, progress = 43, pain = { "wrist" }, hrAfter = true, mp = { Ch },
		ms = { Sh, Tr, Ab } }),
	[42] = move({ key = "scap_pushup", group = "push", pattern = "push", region = "push", class = 1, dose = "rep",
		ladder = 1, rung = 2, pain = { "wrist" }, hrAfter = true, mp = { Sh }, ms = { Ch } }),
	[43] = move({ key = "desk_pushup", group = "push", pattern = "push", region = "push", class = 1, dose = "rep",
		ladder = 1, rung = 2, anchor = true, regress = 12, progress = 44, pain = { "wrist" },
		hrAfter = true, mp = { Ch }, ms = { Sh, Tr, Ab } }),
	[44] = move({ key = "knee_pushup", group = "push", pattern = "push", region = "push", class = 2, dose = "rep",
		ladder = 1, rung = 3, anchor = true, regress = 43, progress = 47, twin = 45, flags = { "floor" },
		pain = { "wrist" }, hrAfter = true, mp = { Ch, Tr }, ms = { Sh, Ab } }),
	[45] = move({ key = "chair_pushup", group = "push", pattern = "push", region = "push", class = 1, dose = "rep",
		ladder = 1, rung = 3, regress = 43, twin = 46, needs = { "stable_chair" }, support = "chair",
		pain = { "wrist" }, hrAfter = true, mp = { Ch }, ms = { Sh, Tr, Ab } }),
	[46] = move({ key = "desk_pushup_slow", group = "push", pattern = "push", region = "push", class = 2, dose = "rep",
		ladder = 1, rung = 3, regress = 43, pain = { "wrist" }, hrAfter = true, mp = { Ch }, ms = { Sh, Tr, Ab } }),
	[47] = move({ key = "negative_pushup", group = "push", pattern = "push", region = "push", class = 2, dose = "rep",
		ladder = 1, rung = 4, anchor = true, regress = 44, progress = 49, twin = 48, flags = { "floor" },
		pain = { "wrist", "elbow" }, hrAfter = true, mp = { Ch, Tr }, ms = { Sh, Ab } }),
	[48] = move({ key = "step_pushup", group = "push", pattern = "push", region = "push", class = 2, dose = "rep",
		ladder = 1, rung = 4, regress = 45, twin = 221, needs = { { "stairs", "step" } }, pain = { "wrist" },
		hrAfter = true, mp = { Ch }, ms = { Sh, Tr, Ab } }),
	[49] = move({ key = "pushup", group = "push", pattern = "push", region = "push", class = 2, dose = "rep",
		ladder = 1, rung = 5, anchor = true, regress = 47, progress = 51, twin = 222, flags = { "floor" },
		pain = { "wrist", "elbow" }, hrAfter = true, mp = { Ch, Tr }, ms = { Sh, Ab } }),
	[50] = move({ key = "pushup_hold", group = "push", pattern = "iso", region = "push", class = 2, dose = "hold",
		ladder = 1, rung = 5, regress = 44, flags = { "floor", "strain" }, pain = { "wrist" }, mp = { Ch, Tr },
		ms = { Sh, Ab } }),
	-- The feet rest on a bed or a sofa (David, 2026-10-04; a stable chair until then): at 45-55 cm about 72 % of the
	-- body is on the hands (Ebben 2011, between its 30 and 61 cm), and what is soft under the feet cannot roll away.
	[51] = move({ key = "feet_up_pushup", group = "push", pattern = "push", region = "push", class = 3, dose = "rep",
		ladder = 1, rung = 6, anchor = true, regress = 49, progress = 54, twin = 52, flags = { "floor" },
		support = "bed", pain = { "wrist", "elbow", "shoulder" }, hrAfter = true, mp = { Ch, Sh }, ms = { Tr, Ab } }),
	[52] = move({ key = "pushup_slow", group = "push", pattern = "push", region = "push", class = 2, dose = "rep",
		ladder = 1, rung = 6, regress = 49, twin = 223, flags = { "floor" }, pain = { "wrist", "elbow" },
		hrAfter = true, mp = { Ch, Tr }, ms = { Sh, Ab } }),
	-- The moves in between that the climb waits for (the ladder rethink, 2026-10-10, David: "You should be strong
	-- enough to easily handle the next rung"), each on the rung climbed from with its own window (Progress.lua `alt`):
	-- the close push-up, hands together under the chest, the triceps' push the archer's bent arm needs (Convict
	-- Conditioning's "Step Six: Close Pushups" between the full and the uneven push-up; Overcoming Gravity 2's diamond
	-- push-up, level 2), and the one-arm push-up with the hand on the edge of a bed or a sofa, the one-arm push-up at a
	-- lighter load (Overcoming Gravity 2's elevated one-arm push-up; Calisthenics Corner).
	[364] = move({ key = "close_pushup", group = "push", pattern = "push", region = "push", class = 3, dose = "rep",
		ladder = 1, rung = 6, regress = 49, flags = { "floor" }, pain = { "wrist", "elbow" }, hrAfter = true,
		mp = { Tr, Ch }, ms = { Sh, Ab } }),
	[365] = move({ key = "incline_one_arm_pushup", group = "push", pattern = "push", region = "push", class = 3,
		dose = "rep", ladder = 1, rung = 7, regress = 223, side = true, flags = { "skill", "strain" }, support = "bed",
		pain = { "wrist", "elbow", "shoulder" }, hrAfter = true, mp = { Ch, Tr }, ms = { Sh, Ob, Ab } }),
	-- The ladder gaps (2026-10-12, David: "there needs to be more steps inbetween"): the staggered push-up, one hand
	-- under the chest and the other two hand-lengths further out, the first push carried mostly by one arm before the
	-- archer (Hybrid Calisthenics' side-staggered push-ups, between the narrow and the archer push-up), and the sliding
	-- one-arm push-up, the helping hand on a towel sliding out, so the working arm carries nearly the one-arm push-up's
	-- load on its own path (Hybrid Calisthenics' sliding one-arm push-ups, between the archer and the one-arm push-up).
	-- Done on the floor, each runs every rep on one side, then Switch sides and the other.
	[394] = move({ key = "staggered_pushup", group = "push", pattern = "push", region = "push", class = 3, dose = "rep",
		ladder = 1, rung = 6, regress = 49, side = true, flags = { "floor" }, pain = { "wrist", "elbow", "shoulder" },
		hrAfter = true, mp = { Ch, Tr }, ms = { Sh, Ob, Ab } }),
	[395] = move({ key = "sliding_one_arm_pushup", group = "push", pattern = "push", region = "push", class = 3,
		dose = "rep", ladder = 1, rung = 7, regress = 54, side = true, flags = { "floor", "strain", "skill" },
		pain = { "wrist", "elbow", "shoulder" }, hrAfter = true, mp = { Ch, Tr }, ms = { Sh, Ob, Ab } }),
	[54] = move({ key = "archer_pushup", group = "push", pattern = "push", region = "push", class = 3, dose = "rep",
		ladder = 1, rung = 7, anchor = true, regress = 394, progress = 55, twin = 224, pairs = true,
		flags = { "floor", "skill" }, pain = { "wrist", "elbow", "shoulder" }, theme = { "bow" },
		hrAfter = true, mp = { Ch, Tr },
		ms = { Sh, Ob, Ab } }),
	[55] = move({ key = "one_arm_pushup", group = "push", pattern = "push", region = "push", class = 3, dose = "rep",
		ladder = 1, rung = 8, anchor = true, regress = 395, twin = 224, side = true, flags = { "floor", "strain", "skill" },
		pain = { "wrist", "elbow", "shoulder" }, hrAfter = true, mp = { Ch, Tr }, ms = { Sh, Ob, Ab } }),
	-- The one-arm push-up's other forms (slice 22): the lowering alone, an orange move beside the wide-stance push-up
	-- (the gold rung's named proof from 2026-10-10 to 2026-10-13, when the anchor's own top of 8 had passed its 6
	-- lowerings first), and the feet-together push-up, the ladder's gold top (Overcoming Gravity's straight-body one-arm
	-- push-up, level 8). Between them since 2026-10-13 (the readiness check) the one-arm push-up with the feet
	-- hip-width, the gold rung's proof and the feet-together push-up's easier version (Hybrid Calisthenics: "Bringing
	-- our feet together increases the resistance on our working arm").
	[300] = move({ key = "one_arm_negative", group = "push", pattern = "push", region = "push", class = 3, dose = "rep",
		ladder = 1, rung = 8, regress = 54, side = true, flags = { "floor", "strain", "skill" },
		pain = { "wrist", "elbow", "shoulder" }, hrAfter = true, mp = { Ch, Tr }, ms = { Sh, Ob, Ab } }),
	[428] = move({ key = "one_arm_pushup_hip", group = "push", pattern = "push", region = "push", class = 3,
		dose = "rep", ladder = 1, rung = 8, regress = 55, side = true, flags = { "floor", "strain", "skill" },
		pain = { "wrist", "elbow", "shoulder" }, hrAfter = true, mp = { Ch, Tr }, ms = { Sh, Ob, Ab } }),
	[301] = move({ key = "one_arm_pushup_close", group = "push", pattern = "push", region = "push", class = 3,
		dose = "rep", ladder = 1, rung = 9, anchor = true, regress = 428, twin = 224, side = true,
		flags = { "floor", "strain", "skill" }, pain = { "wrist", "elbow", "shoulder" }, hrAfter = true,
		mp = { Ch, Tr }, ms = { Sh, Ob, Ab } }),
	-- Push at the desk, no floor (David, 2026-09-30: a backup "the same level just different method"): the desk edge
	-- loads about 35 % of the body (Ebben 2011, extrapolated), so the level comes from one arm: the archer at rung 4,
	-- the one-arm lowering at 5, the one-arm push-up at 6, a slow one at 7; rung 8 has no desk version.
	[221] = move({ key = "desk_archer_pushup", group = "push", pattern = "push", region = "push", class = 2,
		dose = "rep", ladder = 1, rung = 4, regress = 46, twin = 46, pairs = true, flags = { "skill" },
		pain = { "wrist", "elbow" }, theme = { "bow" }, hrAfter = true, mp = { Ch, Tr }, ms = { Sh, Ob, Ab } }),
	[222] = move({ key = "desk_one_arm_negative", group = "push", pattern = "push", region = "push", class = 2,
		dose = "rep", ladder = 1, rung = 5, regress = 221, twin = 48, side = true, flags = { "skill" },
		pain = { "wrist", "elbow", "shoulder" }, hrAfter = true, mp = { Ch, Tr }, ms = { Sh, Ob, Ab } }),
	[223] = move({ key = "desk_one_arm_pushup", group = "push", pattern = "push", region = "push", class = 3,
		dose = "rep", ladder = 1, rung = 6, regress = 222, twin = 48, side = true, flags = { "skill" },
		pain = { "wrist", "elbow", "shoulder" }, hrAfter = true, mp = { Ch, Tr }, ms = { Sh, Ob, Ab } }),
	[224] = move({ key = "desk_one_arm_slow", group = "push", pattern = "push", region = "push", class = 3,
		dose = "rep", ladder = 1, rung = 7, regress = 223, twin = 48, side = true, flags = { "skill", "strain" },
		pain = { "wrist", "elbow", "shoulder" }, hrAfter = true, mp = { Ch, Tr }, ms = { Sh, Ob, Ab } }),

	-- The handstand (slice 22; Intellect): the pike push-up, then with the feet raised, the wall handstand (chest to
	-- the wall) and its push-up, the freestanding handstand and its push-up. Every rung asks for the floor and, from
	-- the wall handstand, a clear 2 x 2 m and a wall, never beside the desk, so no rung has a desk twin. Open once a
	-- push-up record (or the desk one-arm lowering's) reaches its window's floor (ns.Open). The holds are `iso`: a hold
	-- takes no hrAfter.
	[53] = move({ key = "pike_pushup", group = "push", pattern = "push", region = "push", class = 2, dose = "rep",
		ladder = 20, rung = 1, anchor = true, regress = 49, flags = { "floor", "head_down", "skill" },
		hint = 118, pain = { "wrist", "elbow", "shoulder", "neck" }, hrAfter = true, mp = { Sh }, ms = { Tr, Nk, Ch } }),
	[329] = move({ key = "pike_pushup_feet_up", group = "push", pattern = "push", region = "push", class = 3,
		dose = "rep", ladder = 20, rung = 2, anchor = true, regress = 53, flags = { "floor", "head_down", "skill" },
		pain = { "wrist", "elbow", "shoulder", "neck" }, hrAfter = true, mp = { Sh }, ms = { Tr, Nk, Ch } }),
	[330] = move({ key = "wall_handstand", group = "push", pattern = "iso", region = "push", class = 3, dose = "hold",
		ladder = 20, rung = 3, anchor = true, regress = 396,
		flags = { "floor", "space", "head_down", "strain", "skill" }, pain = { "wrist", "elbow", "shoulder", "neck" },
		mp = { Sh }, ms = { Tr, Ab, Fa, BALANCE } }),
	[331] = move({ key = "wall_hspu", group = "push", pattern = "push", region = "push", class = 3, dose = "rep",
		ladder = 20, rung = 4, anchor = true, regress = 366,
		flags = { "floor", "space", "head_down", "strain", "skill" }, pain = { "wrist", "elbow", "shoulder", "neck" },
		hrAfter = true, mp = { Sh, Tr }, ms = { Nk, Ch, Ab } }),
	[332] = move({ key = "handstand", group = "push", pattern = "iso", region = "push", class = 3, dose = "hold",
		ladder = 20, rung = 5, anchor = true, regress = 367,
		flags = { "floor", "space", "head_down", "strain", "skill" }, pain = { "wrist", "elbow", "shoulder", "neck" },
		mp = { Sh }, ms = { Fa, Tr, Ab, BALANCE } }),
	[333] = move({ key = "handstand_pushup", group = "push", pattern = "push", region = "push", class = 3,
		dose = "rep", ladder = 20, rung = 6, anchor = true, regress = 368,
		flags = { "floor", "space", "head_down", "strain", "skill" }, pain = { "wrist", "elbow", "shoulder", "neck" },
		hrAfter = true, mp = { Sh, Tr }, ms = { Nk, Ch, Ab, BALANCE } }),
	-- The moves in between that the climb waits for (2026-10-10), each on the rung climbed from: the wall handstand
	-- push-up's lowering alone (Overcoming Gravity 2's wall headstand push-up eccentric, level 3), the wall handstand
	-- with the toes drifted off the wall, timed while they are off (pull-aways; Calisthenics Corner: "Hold 5-10
	-- seconds off the wall"), and the freestanding handstand push-up's lowering, a wall within reach behind the heels.
	[366] = move({ key = "wall_hspu_lowering", group = "push", pattern = "push", region = "push", class = 3,
		dose = "rep", ladder = 20, rung = 3, regress = 330,
		flags = { "floor", "space", "head_down", "strain", "skill" }, pain = { "wrist", "elbow", "shoulder", "neck" },
		hrAfter = true, mp = { Sh, Tr }, ms = { Nk, Ch, Ab } }),
	[367] = move({ key = "wall_handstand_off", group = "push", pattern = "iso", region = "push", class = 3,
		dose = "hold", ladder = 20, rung = 4, regress = 330,
		flags = { "floor", "space", "head_down", "strain", "skill" }, pain = { "wrist", "elbow", "shoulder", "neck" },
		mp = { Sh }, ms = { Fa, Tr, Ab, BALANCE } }),
	[368] = move({ key = "hspu_lowering", group = "push", pattern = "push", region = "push", class = 3,
		dose = "rep", ladder = 20, rung = 5, regress = 331,
		flags = { "floor", "space", "head_down", "strain", "skill" }, pain = { "wrist", "elbow", "shoulder", "neck" },
		hrAfter = true, mp = { Sh, Tr }, ms = { Nk, Ch, Ab, BALANCE } }),
	-- The ladder gaps (2026-10-12): the L-hold at the wall, the feet on the wall at hip height and the hips over the
	-- hands, straight arms by the ears and a held time upside down before the wall handstand (keepitsimpelle's L-stand,
	-- "work up to 1 minute per set"; the Bible of Calisthenics puts a wall step between the elevated pike push-up and the
	-- wall holds), and the kick-up to the wall with its turn-out, the kick-up and the bail the freestanding handstand
	-- starts and ends with (Heather Blackmon's beginner handstand series: "cartwheel out confidently five times in a
	-- row" before any chest-to-wall practice). Neither puts the head on the floor.
	[396] = move({ key = "wall_l_hold", group = "push", pattern = "iso", region = "push", class = 3, dose = "hold",
		ladder = 20, rung = 2, regress = 329, flags = { "floor", "space", "head_down", "strain", "skill" },
		pain = { "wrist", "elbow", "shoulder" }, mp = { Sh }, ms = { Tr, Ab, Fa } }),
	[397] = move({ key = "wall_kickup_turnout", group = "push", pattern = "push", region = "push", class = 3,
		dose = "rep", ladder = 20, rung = 3, regress = 330, flags = { "floor", "space", "head_down", "strain", "skill" },
		pain = { "wrist", "elbow", "shoulder" }, hrAfter = true, mp = { Sh }, ms = { Tr, Ab, Fa, BALANCE } }),

	-- The planche on the floor (slice 22; Strength): the lean, the frog stand (the crow's shape held for a record), the
	-- same on straight arms, the tuck planche, its flat-back form, the straddle planche; the lean's push-up beside
	-- them. A pillow in front of the hands, as for the crow. The tuck and above are tendon-heavy (straight
	-- arms under load), so the straddle's unlock asks for three days. Open as the handstand, by the push-up's record.
	[342] = move({ key = "planche_lean", group = "push", pattern = "iso", region = "push", class = 2, dose = "hold",
		ladder = 22, rung = 1, anchor = true, regress = 72, flags = { "floor", "strain" },
		pain = { "wrist", "elbow", "shoulder" }, mp = { Sh }, ms = { Ch, Ab, Fa } }),
	[343] = move({ key = "frog_stand", group = "push", pattern = "iso", region = "push", class = 2, dose = "hold",
		ladder = 22, rung = 2, anchor = true, regress = 425, flags = { "floor", "space", "strain", "skill" },
		pain = { "wrist", "elbow", "shoulder" }, mp = { Sh, Tr }, ms = { Ab, Fa, BALANCE } }),
	[344] = move({ key = "pseudo_planche_pushup", group = "push", pattern = "push", region = "push", class = 3,
		dose = "rep", ladder = 22, rung = 3, regress = 49, flags = { "floor", "skill" },
		pain = { "wrist", "elbow", "shoulder" }, hrAfter = true, mp = { Ch, Sh }, ms = { Tr, Ab } }),
	[345] = move({ key = "crane_stand", group = "push", pattern = "iso", region = "push", class = 3, dose = "hold",
		ladder = 22, rung = 3, anchor = true, regress = 343, flags = { "floor", "space", "strain", "skill" },
		pain = { "wrist", "elbow", "shoulder" }, mp = { Sh, Tr }, ms = { Ab, Fa, BALANCE } }),
	[346] = move({ key = "tuck_planche", group = "push", pattern = "iso", region = "push", class = 3, dose = "hold",
		ladder = 22, rung = 4, anchor = true, regress = 370,
		flags = { "floor", "space", "strain", "skill", "tendon" }, pain = { "wrist", "elbow", "shoulder" },
		mp = { Sh }, ms = { Ch, Ab, Bi, Fa } }),
	[347] = move({ key = "adv_tuck_planche", group = "push", pattern = "iso", region = "push", class = 3,
		dose = "hold", ladder = 22, rung = 5, anchor = true, regress = 371,
		flags = { "floor", "space", "strain", "skill", "tendon" }, pain = { "wrist", "elbow", "shoulder" },
		mp = { Sh }, ms = { Ch, Ab, Bi, LB } }),
	[348] = move({ key = "straddle_planche", group = "push", pattern = "iso", region = "push", class = 3,
		dose = "hold", ladder = 22, rung = 6, anchor = true, regress = 347,
		flags = { "floor", "space", "strain", "skill", "tendon" }, hint = 413, pain = { "wrist", "elbow", "shoulder" },
		mp = { Sh }, ms = { Ch, Ab, Bi, Gl } }),
	-- The in-between holds (the ladder rethink, 2026-10-10; David: "I feel it's too difficult to go to frog stand right
	-- away", then "You should be strong enough to easily handle the next rung"): each sits on the rung climbed from,
	-- with its own window (Progress.lua `alt`), and trains what the next rung needs that the rung below does not. The
	-- frog stand with one foot up, each side timed (GMB's crow steps: "Point Your Toes ... one foot at a time"), the
	-- frog stand from books' easier version; the tuck planche with the toes resting light on a stack of books behind
	-- (Calisthenics Corner's "Assisted tuck planche"), the tuck planche's; the tuck with the hips opened halfway, the
	-- knees halfway back from the chest and the back flat; the flat-back tuck with one leg straight back, each side
	-- timed (Overcoming Gravity 2 has no level 7 between the advanced tuck, 6, and the straddle, 8, and rates the
	-- one-leg planche 9 with the half lay, so its window stays short: 2-5 s a side, 2026-10-13). The two above the
	-- tuck are tendon-heavy, as their rungs' anchors are. Since 2026-10-13 the one-foot frog stand tops at 30 s a side
	-- (Antranik: "30sec One-Foot Supported Crow" before "20sec Crow Pose"): David held 25 s and the frog stand still
	-- felt far off.
	[369] = move({ key = "frog_stand_one_foot", group = "push", pattern = "iso", region = "push", class = 2,
		dose = "hold", ladder = 22, rung = 1, regress = 342, side = true, flags = { "floor", "space", "strain", "skill" },
		pain = { "wrist", "elbow", "shoulder" }, mp = { Sh, Tr }, ms = { Ab, Fa, BALANCE } }),
	-- The frog stand from books beside it (2026-10-13, the readiness check): the toes start on a stack of books about
	-- 12 cm high and the lean lifts both feet off them, the balance point with nothing under the feet, which the
	-- one-foot hold's planted foot never meets (Antranik's crow progression: "30sec One-Foot Supported Crow Using
	-- Block", then "20sec Crow Pose using Block"; "Elevating your feet by a few inches makes it such that you don't
	-- have to compress as much"). Rung 1's climb waits for both, its path's two stages, the one-foot hold and then this
	-- (slice 42; `proofAll` until then), and it is the frog stand's easier version.
	[425] = move({ key = "frog_stand_books", group = "push", pattern = "iso", region = "push", class = 2,
		dose = "hold", ladder = 22, rung = 1, regress = 369, flags = { "floor", "space", "strain", "skill" },
		pain = { "wrist", "elbow", "shoulder" }, mp = { Sh, Tr }, ms = { Ab, Fa, BALANCE } }),
	[370] = move({ key = "tuck_planche_assisted", group = "push", pattern = "iso", region = "push", class = 3,
		dose = "hold", ladder = 22, rung = 3, regress = 345, flags = { "floor", "space", "strain", "skill" },
		pain = { "wrist", "elbow", "shoulder" }, mp = { Sh }, ms = { Ch, Ab, Bi, Fa } }),
	[371] = move({ key = "open_tuck_planche", group = "push", pattern = "iso", region = "push", class = 3,
		dose = "hold", ladder = 22, rung = 4, regress = 346,
		flags = { "floor", "space", "strain", "skill", "tendon" }, pain = { "wrist", "elbow", "shoulder" },
		mp = { Sh }, ms = { Ch, Ab, Bi, LB } }),
	[372] = move({ key = "one_leg_planche", group = "push", pattern = "iso", region = "push", class = 3,
		dose = "hold", ladder = 22, rung = 5, regress = 347, side = true,
		flags = { "floor", "space", "strain", "skill", "tendon" }, pain = { "wrist", "elbow", "shoulder" },
		mp = { Sh }, ms = { Ch, Ab, Bi, LB, Gl } }),

	-- Power (slice 22; Agility): the squat jump (off the ladders until then; a finisher form still) with the strict
	-- burpee beside it, the explosive and the clap push-up with the tuck jump as their twin without floor, then on a
	-- bar the explosive pull-up to the muscle-up (group bar, below). Every jump keeps `crung`: cardio rung 5, where the
	-- family opens (ns.Open).
	[25] = move({ key = "squat_jump", group = "squat", pattern = "ballistic", region = "legs", class = 3, dose = "rep",
		road = "jump", ladder = 23, rung = 1, anchor = true, regress = 9, flags = { "impact", "loud", "skill" }, jump = true,
		crung = 5, pain = { "knee", "achilles" }, theme = { "ride" }, finisher = 5, mp = { Qu, Gl, Ca },
		ms = { Ha, HEART } }),
	-- the strict burpee on rung 2 at cardio rung 7 since 2026-10-16 (slice 42): on rung 1 at cardio rung 8 it almost
	-- never came, a family's first step waiting for the cardio ladder's step 10
	[349] = move({ key = "burpee_strict", group = "cardio", pattern = "ballistic", region = "legs", class = 3,
		dose = "rep", ladder = 23, rung = 2, regress = 25, flags = { "floor", "loud", "impact" }, jump = true, crung = 7,
		pain = { "wrist", "knee", "achilles" }, hrAfter = true, mp = { Qu, Ch }, ms = { Sh, Ab, Gl, Ca, HEART } }),
	[134] = move({ key = "frog_jumps", group = "lunge", pattern = "ballistic", region = "legs", class = 3,
		dose = "rep", ladder = 23, rung = 2, regress = 25, flags = { "impact", "loud", "space", "skill" }, jump = true,
		crung = 5, pain = { "knee", "achilles" }, theme = { "frog" }, mp = { Qu, Gl, Ca }, ms = { Ha, HEART } }),
	[350] = move({ key = "tuck_jump", group = "squat", pattern = "ballistic", region = "legs", class = 3, dose = "rep",
		ladder = 23, rung = 2, regress = 25, flags = { "impact", "loud", "skill" }, jump = true, crung = 5,
		pain = { "knee", "achilles" }, mp = { Qu, Gl, Ca }, ms = { Ab, HEART } }),
	[351] = move({ key = "explosive_pushup", group = "push", pattern = "ballistic", region = "push", class = 3,
		dose = "rep", ladder = 23, rung = 2, anchor = true, regress = 49, twin = 350, flags = { "floor", "skill" },
		checks = { { ladder = LAD.push, step = 6, count = 12 } },
		pain = { "wrist", "elbow", "shoulder" }, hrAfter = true, mp = { Ch, Tr }, ms = { Sh, Ab } }),
	-- the clap push-up waits for the 12 feet-up push-ups the explosive and the knee clap push-up wait for (2026-10-24:
	-- without them the tuck jump's climb opened step 7 and offered it to a jumper short of the NSCA's push strength)
	[352] = move({ key = "clap_pushup", group = "push", pattern = "ballistic", region = "push", class = 3,
		dose = "rep", ladder = 23, rung = 3, anchor = true, regress = 403, twin = 350,
		flags = { "floor", "loud", "skill" }, checks = { { ladder = LAD.push, step = 6, count = 12 } },
		pain = { "wrist", "elbow", "shoulder" }, hrAfter = true, mp = { Ch, Tr }, ms = { Sh, Ab } }),
	-- The clap push-up on the knees beside the explosive push-up (the ladder gaps, 2026-10-12): the hands carry 49 % of
	-- the body there against 64 % on the toes (Ebben 2011), so the same push gives the air time to clap and trains the
	-- catch, which the explosive push-up's small hop never does (Koch 2012 reaches the clap through small drops first).
	[403] = move({ key = "knee_clap_pushup", group = "push", pattern = "ballistic", region = "push", class = 3,
		dose = "rep", ladder = 23, rung = 2, regress = 351, flags = { "floor", "loud", "skill" },
		checks = { { ladder = LAD.push, step = 6, count = 12 } },
		pain = { "wrist", "elbow", "shoulder", "knee" }, hrAfter = true, mp = { Ch, Tr }, ms = { Sh, Ab } }),

	-- The row (the pull ladder until 2026-10-04), by the load an arm takes (2026-10-04; the towel's hold and its
	-- shoulder-blade pull held rungs 4 and 5 until then, lighter an arm than the rungs below them): both arms on the
	-- doorframe, one arm with a weight in the hand (any heavy object, 5-15 kg to start and more once 20 reps come; the
	-- backpack, chair and dumbbell rows are this one move since then, ns.Merged), both arms under a sturdy table, on a
	-- bath towel over the top of a door or on rings, or one arm on the doorframe with the toes a foot's length back
	-- (the strap lean-back loads 53 % of the body at 45 degrees, Melrose and Dawes 2015, so about a quarter on each
	-- arm), one arm on the doorframe (a one-arm lean takes the whole load: at a 25-30 degree lean about 31-39 % of the
	-- body), one arm with the toes at its base and a slow lowering; then one arm on the towel, lowering first, or under
	-- the table with the heels up, a backpack on the chest and a hold at the top. The slow lowering of the weighted row
	-- and the holds and shoulder-blade pulls are variants, never a rung: tempo alone climbs nothing.
	-- The towel rows need the Towel and door box since 2026-10-28 (David: "That seems a bit complicated. I mean we can
	-- have it if people say they have towels and we need to properly explain in the how to. Otherwise research a
	-- different excercise to replace it with."): off for every player, the knot now over the top of a door the player
	-- pushes open, as the door anchors of TRX, Gaiam and inSPORTline go for that door, where it sat behind the door by
	-- the top hinge; a doorknob anchors nothing (no maker does; docs/research/towel-door-rows-2026-10-28.md). Without
	-- the towel and door or a table the row ends at rung 5, the steep one-arm doorframe row (a bar or rings alone too:
	-- their moves start at rung 9): nothing else at home carries rungs 6-8's 40-50 % of the body on one arm.
	[56] = move({ key = "doorframe_row", group = "pull", pattern = "pull", region = "pull", class = 1, dose = "rep",
		ladder = 9, rung = 1, anchor = true, progress = 57, pain = { "elbow" }, mp = { UB, La }, ms = { Bi, Fa } }),
	[58] = move({ key = "doorframe_iso", group = "pull", pattern = "iso", region = "pull", class = 1, dose = "hold",
		ladder = 9, rung = 1, strainFree = true, mp = { UB, La }, ms = { Bi, Fa } }),
	[57] = move({ key = "backpack_row", group = "pull", pattern = "pull", region = "pull", class = 2, dose = "rep",
		ladder = 9, rung = 2, anchor = true, regress = 56, progress = 59, side = true, pain = { "elbow" },
		theme = { "oar" }, load = "total", kg = { 5, 15 }, holds = { "dumbbells", "kettlebell" }, mp = { La, UB },
		ms = { Bi, Fa, Ob } }),
	[61] = move({ key = "ytw_raise", group = "pull", pattern = "pull", region = "pull", class = 1, dose = "rep",
		ladder = 9, rung = 2, flags = { "floor" }, mp = { UB }, ms = { Sh, Nk, LB } }),
	[230] = move({ key = "towel_door_scap_pull", group = "pull", pattern = "pull", region = "pull", class = 2,
		dose = "rep", ladder = 9, rung = 2, regress = 56, needs = { "door_towel" }, pain = { "shoulder" },
		mp = { UB, La }, ms = { Nk, Fa } }),
	-- The doorframe's shoulder-blade pull beside it (2026-10-28, the towel's for a setup without the towel): rung 1's
	-- grip and lean, the arms straight, the shoulder blades drawn back and down, then let slide forward; about a
	-- quarter of the body in all at that lean (0.68 x tan of the lean, the model of the one-arm rows above).
	[647] = move({ key = "doorframe_scap_pull", group = "pull", pattern = "pull", region = "pull", class = 1,
		dose = "rep", ladder = 9, rung = 2, regress = 56, pain = { "shoulder" }, mp = { UB, La }, ms = { Nk, Fa } }),
	-- The table row with the knees bent and the feet flat (2026-10-13, the readiness check): the legs take part of the
	-- body, so the pull under the table starts lighter than with the heels out (Convict Conditioning's bent-leg
	-- horizontal pulls before horizontal pulls); rung 2's proof, the table row's easier version.
	[430] = move({ key = "table_row_bent", group = "pull", pattern = "pull", region = "pull", class = 2, dose = "rep",
		ladder = 9, rung = 2, regress = 432, needs = { "table" }, support = "table", pain = { "shoulder", "elbow" },
		mp = { La, UB, Bi }, ms = { Sh, Nk, LB, Gl } }),
	-- Its lowering (2026-10-14, David: the table row "needs steps before"): the feet push the body up to the edge and
	-- the arms lower it over 3 s, the hips resting on the floor between reps. The largest jump on the row is the
	-- doorframe row's quarter of the body to a table row's 55-65 % (Melrose and Dawes 2015; Fenwick 2009), and a
	-- lowering takes about 1.4 times what a pull lifts (Nuzzo 2023), so it trains the table before its first full rep
	-- (Eleiko's bent-knee inverted row eccentric); the knees-bent row's easier version.
	[432] = move({ key = "table_row_lowering", group = "pull", pattern = "pull", region = "pull", class = 2,
		dose = "rep", ladder = 9, rung = 2, regress = 57, needs = { "table" }, support = "table",
		pain = { "shoulder", "elbow" }, mp = { La, UB, Bi }, ms = { Sh, Nk, LB, Gl } }),
	[59] = move({ key = "inverted_row", group = "pull", pattern = "pull", region = "pull", class = 2, dose = "rep",
		ladder = 9, rung = 3, anchor = true, regress = 430, progress = 225, twin = 228, needs = { "table" },
		support = "table", pain = { "shoulder", "elbow" }, mp = { La, UB, Bi }, ms = { Sh, Nk, LB, Gl } }),
	[228] = move({ key = "towel_door_row", group = "pull", pattern = "pull", region = "pull", class = 2, dose = "rep",
		ladder = 9, rung = 3, regress = 56, twin = 199, needs = { "door_towel" }, pain = { "elbow", "shoulder" },
		theme = { "oar" }, mp = { La, UB }, ms = { Bi, Fa, Ab } }),
	[60] = move({ key = "backpack_row_slow", group = "pull", pattern = "pull", region = "pull", class = 2,
		dose = "rep", ladder = 9, rung = 3, regress = 57, side = true, pain = { "elbow" }, theme = { "oar" },
		load = "total", kg = { 5, 15 }, holds = { "dumbbells", "kettlebell" }, mp = { La, UB }, ms = { Bi, Fa, Ob } }),
	[229] = move({ key = "towel_door_hold", group = "pull", pattern = "iso", region = "pull", class = 2, dose = "hold",
		ladder = 9, rung = 3, regress = 58, strainFree = true, needs = { "door_towel" }, pain = { "elbow", "shoulder" },
		mp = { Fa, La }, ms = { UB, Bi } }),
	-- The one-arm doorframe row with the toes a foot's length back (2026-10-28): rung 3's own move without a table, the
	-- towel or rings, after them in the rung's chain, so a bare player climbs 3 to 5 on the doorframe. The toes set the
	-- lean, about 15-20 degrees, so about 18-25 % of the body on the arm (0.68 x tan of the lean), the two-arm towel
	-- row's share an arm (Melrose and Dawes 2015); the one-arm doorframe row's easier version. Its hold at the top, the
	-- elbow back and the shoulder down, stands beside it in the towel hold's seconds, a side each.
	[648] = move({ key = "one_arm_doorframe_row_back", group = "pull", pattern = "pull", region = "pull", class = 2,
		dose = "rep", ladder = 9, rung = 3, regress = 56, progress = 225, side = true, pain = { "elbow" },
		theme = { "oar" }, mp = { La, UB }, ms = { Bi, Fa, Ob } }),
	[649] = move({ key = "one_arm_doorframe_hold", group = "pull", pattern = "iso", region = "pull", class = 2,
		dose = "hold", ladder = 9, rung = 3, regress = 58, side = true, strainFree = true, pain = { "elbow", "shoulder" },
		mp = { Fa, La }, ms = { UB, Bi } }),
	-- The wall angels' goal pose is the wall test since slice 42 (arms straight by the ears, the thumbs a fist from the
	-- wall, the low back within a hand of it; the thumbs on the wall until 2026-10-27, a step 1 most adults cannot
	-- reach; the old mark fixed no height): its answers count toward the checks that name it, its best and its first
	-- passes only from 2026-10-17 00:00 UTC on (`goalSince`, DB.lua markDays; slice 42's review), so an answer to the
	-- old mark never passes the handstand's or the standing rollout's check; the softer wall test keeps that date.
	[62] = move({ key = "wall_angels", group = "pull", pattern = "mobility", class = 1, dose = "flow", secs = 45,
		ladder = 15, rung = 1, anchor = true, goalSince = 1792195200, mp = { UB }, ms = { Sh, BREATH } }),
	[225] = move({ key = "one_arm_doorframe_row", group = "pull", pattern = "pull", region = "pull", class = 2,
		dose = "rep", ladder = 9, rung = 4, anchor = true, regress = 648, progress = 227, twin = 228, side = true,
		pain = { "elbow" }, theme = { "oar" }, mp = { La, UB }, ms = { Bi, Fa, Ob } }),
	[227] = move({ key = "steep_doorframe_row", group = "pull", pattern = "pull", region = "pull", class = 2,
		dose = "rep", ladder = 9, rung = 5, anchor = true, regress = 225, progress = 231, twin = 225, side = true,
		pain = { "elbow" }, theme = { "oar" }, mp = { La, UB }, ms = { Bi, Fa, Ob } }),
	[231] = move({ key = "towel_door_lowering", group = "pull", pattern = "pull", region = "pull", class = 3,
		dose = "rep", ladder = 9, rung = 6, anchor = true, regress = 227, progress = 232, twin = 650, side = true,
		needs = { "door_towel" }, pain = { "elbow", "shoulder" }, mp = { La, Bi }, ms = { UB, Fa, Ab } }),
	-- The table row with the heels up on a sofa, a bed or a chair without wheels (2026-10-28): the body flat, about
	-- 73-76 % of the body on the hands, about 37 % an arm (Vural 2023, load cells on straps with the feet up,
	-- https://pmc.ncbi.nlm.nih.gov/articles/PMC10516423/), rung 6's own move for a table without the towel; then a
	-- backpack of 5-15 kg on the chest (rung 7; about 90 % of a chest-worn load reaches the hands, so 10 kg adds about
	-- 6 % an arm at 75 kg) and a 2-s hold at the top with it (rung 8), the table's way to rung 9's archer row. Two-arm
	-- rows: the same level by difficulty, under the one-arm towel rows' 44-53 % an arm, since a one-arm table row
	-- hangs the whole body on one edge (the tipping case). A weighted vest carries the pack's load where there is one.
	[650] = move({ key = "table_row_feet_up", group = "pull", pattern = "pull", region = "pull", class = 3,
		dose = "rep", ladder = 9, rung = 6, regress = 59, progress = 651, twin = 653, needs = { "table" },
		support = "table", pain = { "shoulder", "elbow" }, mp = { La, UB, Bi }, ms = { Sh, Nk, LB, Gl } }),
	-- The one-arm towel row with the toes a foot's length back from the door beside the lowering (2026-10-12, the
	-- ladder gaps): a one-arm pull up between the steep doorframe row's load and the towel's with the toes at the door,
	-- where rung 6 only lowers (Overcoming Gravity's easier one-arm row before the full one; the feet's distance from
	-- the anchor sets a suspension row's load).
	[401] = move({ key = "towel_door_one_arm_row_back", group = "pull", pattern = "pull", region = "pull", class = 3,
		dose = "rep", ladder = 9, rung = 6, regress = 227, side = true, needs = { "door_towel" },
		pain = { "elbow", "shoulder" }, theme = { "oar" }, mp = { La, Bi }, ms = { UB, Fa, Ob } }),
	[232] = move({ key = "towel_door_one_arm_row", group = "pull", pattern = "pull", region = "pull", class = 3,
		dose = "rep", ladder = 9, rung = 7, anchor = true, regress = 401, twin = 651, side = true,
		needs = { "door_towel" }, pain = { "elbow", "shoulder" }, theme = { "oar" }, mp = { La, Bi },
		ms = { UB, Fa, Ob } }),
	[651] = move({ key = "table_row_feet_up_pack", group = "pull", pattern = "pull", region = "pull", class = 3,
		dose = "rep", ladder = 9, rung = 7, regress = 650, progress = 652, twin = 654, needs = { "table" },
		support = "table", pain = { "shoulder", "elbow" }, load = "total", kg = { 5, 15 }, holds = { "vest" },
		mp = { La, UB, Bi }, ms = { Sh, Nk, LB, Gl } }),
	-- The row past the towel's one-arm row (slice 22). With no equipment it ends at green, the steep one-arm doorframe
	-- row (at blue, the one-arm towel row with a hold at the top, until the towel took a box on 2026-10-28); with the
	-- towel and door it reaches blue. A sturdy table reaches purple with two hands on its edge (the archer row: a
	-- one-arm row would hang the whole body on one edge, the tipping case), through its feet-up rows since 2026-10-28.
	-- A bar or rings carry the front lever's path to orange from rung 9 (rungs 6-8 want the towel and door, a table or
	-- rings since 2026-10-28): the tuck lever and its rows, the flat-back rows, the lever (straight arms under load:
	-- tendon-heavy). Since 2026-10-10 (the ladder rethink) the tuck lever's hold anchors
	-- rung 9 with its row and the table's archer row beside it, and the holds Overcoming Gravity puts between the levels
	-- (the flat-back tuck, level 5, before its rows; one leg out, level 7, before the lever) stand beside rungs 9 and 10
	-- as the moves their climbs wait for.
	[313] = move({ key = "towel_door_row_pause", group = "pull", pattern = "pull", region = "pull", class = 3,
		dose = "rep", ladder = 9, rung = 8, anchor = true, regress = 232, twin = 652, side = true,
		needs = { "door_towel" }, pain = { "elbow", "shoulder" }, theme = { "oar" }, mp = { La, Bi },
		ms = { UB, Fa, Ob } }),
	[652] = move({ key = "table_row_feet_up_pause", group = "pull", pattern = "pull", region = "pull", class = 3,
		dose = "rep", ladder = 9, rung = 8, regress = 651, progress = 315, twin = 655, needs = { "table" },
		support = "table", pain = { "shoulder", "elbow" }, load = "total", kg = { 5, 15 }, holds = { "vest" },
		mp = { La, UB, Bi }, ms = { Sh, Nk, LB, Gl } }),
	-- The same three on rings or straps hung low (2026-10-28): rungs 6-8 for a player with rings and neither the towel
	-- nor a table, who climbed through the towel's rows until they took a box. Vural 2023 measured these loads on
	-- straps (the ring row with the feet up, 73-76 % of the body); the pack and the pause as on the table, then the
	-- tuck lean hang (rung 8's path) and the tuck lever's rows, which rings carry.
	[653] = move({ key = "ring_row_feet_up", group = "rings", pattern = "pull", region = "pull", class = 3,
		dose = "rep", ladder = 9, rung = 6, regress = 199, progress = 654, twin = 227, needs = { "rings" },
		pain = { "shoulder", "elbow" }, mp = { La, UB, Bi }, ms = { Sh, Nk, LB, Gl } }),
	[654] = move({ key = "ring_row_feet_up_pack", group = "rings", pattern = "pull", region = "pull", class = 3,
		dose = "rep", ladder = 9, rung = 7, regress = 653, progress = 655, twin = 231, needs = { "rings" },
		pain = { "shoulder", "elbow" }, load = "total", kg = { 5, 15 }, holds = { "vest" }, mp = { La, UB, Bi },
		ms = { Sh, Nk, LB, Gl } }),
	[655] = move({ key = "ring_row_feet_up_pause", group = "rings", pattern = "pull", region = "pull", class = 3,
		dose = "rep", ladder = 9, rung = 8, regress = 654, progress = 314, twin = 232, needs = { "rings" },
		pain = { "shoulder", "elbow" }, load = "total", kg = { 5, 15 }, holds = { "vest" }, mp = { La, UB, Bi },
		ms = { Sh, Nk, LB, Gl } }),
	-- The tuck lean-back hang beside it (2026-10-12, the ladder gaps): the hang, the straight-arm pull and the tuck that
	-- the tuck lever needs and no row trains, the back halfway to level (Overcoming Gravity's straight-arm basics come
	-- before the tuck lever; 20 s there predicts about 10 s level, Frey Law and Avin 2010).
	[402] = move({ key = "tuck_lean_hang", group = "bar", pattern = "iso", region = "pull", class = 3, dose = "hold",
		ladder = 9, rung = 8, regress = 64, flags = { "strain", "skill" }, needs = { { "bar", "fixed_bar", "rings" } },
		pain = { "elbow", "shoulder" }, mp = { La, Ab }, ms = { UB, Fa } }),
	-- The tuck lever's moves on the bar wait for 8 pull-ups (2026-10-16, slice 42: the straight-arm pull needs the
	-- bent-arm one first; their checks, so the table's archer row still climbs to the step without a bar).
	[316] = move({ key = "tuck_front_lever", group = "bar", pattern = "iso", region = "pull", class = 3, dose = "hold",
		ladder = 9, rung = 9, anchor = true, regress = 402, twin = 315, flags = { "strain", "skill", "tendon" },
		checks = { { ladder = LAD.pullup, step = 8, count = 8 } },
		needs = { { "bar", "fixed_bar", "rings" } }, pain = { "elbow", "shoulder" }, mp = { La, Ab }, ms = { UB, Fa } }),
	[314] = move({ key = "tuck_fl_row", group = "bar", pattern = "pull", region = "pull", class = 3, dose = "rep",
		ladder = 9, rung = 9, regress = 316, flags = { "strain", "skill", "tendon" },
		checks = { { ladder = LAD.pullup, step = 8, count = 8 } },
		needs = { { "bar", "fixed_bar", "rings" } }, pain = { "elbow", "shoulder" }, mp = { La, UB }, ms = { Ab, Bi, Fa } }),
	[315] = move({ key = "table_archer_row", group = "pull", pattern = "pull", region = "pull", class = 3,
		dose = "rep", ladder = 9, rung = 9, regress = 652, side = true, flags = { "skill" },
		needs = { "table" }, support = "table", pain = { "shoulder", "elbow" }, theme = { "bow" }, mp = { La, UB, Bi },
		ms = { Sh, Fa, Ob } }),
	[389] = move({ key = "adv_tuck_front_lever", group = "bar", pattern = "iso", region = "pull", class = 3,
		dose = "hold", ladder = 9, rung = 9, regress = 316, progress = 390, flags = { "strain", "skill", "tendon" },
		checks = { { ladder = LAD.pullup, step = 8, count = 8 } },
		needs = { { "bar", "fixed_bar", "rings" } }, pain = { "elbow", "shoulder" }, mp = { La, Ab }, ms = { UB, Fa } }),
	[317] = move({ key = "adv_tuck_fl_row", group = "bar", pattern = "pull", region = "pull", class = 3, dose = "rep",
		ladder = 9, rung = 10, anchor = true, regress = 314, flags = { "strain", "skill", "tendon" },
		needs = { { "bar", "fixed_bar", "rings" } }, pain = { "elbow", "shoulder" }, mp = { La, UB }, ms = { Ab, Bi, LB } }),
	[390] = move({ key = "one_leg_front_lever", group = "bar", pattern = "iso", region = "pull", class = 3,
		dose = "hold", ladder = 9, rung = 10, regress = 389, progress = 318, side = true,
		flags = { "strain", "skill", "tendon" }, needs = { { "bar", "fixed_bar", "rings" } }, pain = { "elbow", "shoulder" },
		mp = { La, Ab }, ms = { UB, Tr, Gl } }),
	-- The half-lay front lever beside it (2026-10-13, the readiness check): both hips open, the knees bent at a right
	-- angle, both legs' lever at once where the one-leg lever loads one (Wikipedia's front lever progression: "single
	-- leg front lever, straddle front lever, half lay front lever and finally on to the full front lever"; Hybrid
	-- Calisthenics: "Keep your front lever holds at around 5-15 seconds"); rung 10's proof, the front lever's easier
	-- version.
	[431] = move({ key = "front_lever_half_lay", group = "bar", pattern = "iso", region = "pull", class = 3,
		dose = "hold", ladder = 9, rung = 10, regress = 390, flags = { "strain", "skill", "tendon" },
		needs = { { "bar", "fixed_bar", "rings" } }, pain = { "elbow", "shoulder" }, mp = { La, Ab },
		ms = { UB, Tr, Gl } }),
	[318] = move({ key = "front_lever", group = "bar", pattern = "iso", region = "pull", class = 3, dose = "hold",
		ladder = 9, rung = 11, anchor = true, regress = 431, flags = { "strain", "skill", "tendon" },
		needs = { { "bar", "fixed_bar", "rings" } }, pain = { "elbow", "shoulder" }, mp = { La, Ab }, ms = { UB, Tr, Gl } }),

	-- The core and the side (the core hold until 2026-10-04): the planks, and the side planks to the Copenhagen. Without
	-- floor space the core rests on the desk planks and the side on the desk side planks.
	[70] = move({ key = "incline_plank", group = "core", pattern = "core_anti_ext", region = "trunk", class = 1,
		dose = "hold", facing = "front", ladder = 10, rung = 1, anchor = true, progress = 71, strainFree = true, mp = { Ab },
		ms = { Ob, Sh } }),
	[71] = move({ key = "knee_plank", group = "core", pattern = "core_anti_ext", region = "trunk", class = 1,
		dose = "hold", facing = "front", ladder = 10, rung = 2, anchor = true, regress = 70, progress = 72, twin = 236,
		strainFree = true, flags = { "floor" }, mp = { Ab }, ms = { Ob, Sh } }),
	[78] = move({ key = "bird_dog", group = "core", pattern = "core_anti_rot", region = "trunk", class = 1,
		dose = "rep", ladder = 10, rung = 2, pairs = true, flags = { "floor" }, pain = { "wrist" }, mp = { LB, Gl },
		ms = { Ab, Sh } }),
	[79] = move({ key = "dead_bug", group = "core", pattern = "core_anti_ext", region = "trunk", class = 1,
		dose = "rep", ladder = 10, rung = 2, flags = { "floor" }, mp = { Ab }, ms = { Ob, Qu } }),
	[137] = move({ key = "wood_chopper", group = "core", pattern = "core_anti_rot", region = "trunk", class = 1,
		dose = "rep", ladder = 10, rung = 2, side = true, pain = { "back", "shoulder" }, theme = { "axe" }, mp = { Ob },
		ms = { Sh, Ab, Gl } }),
	[138] = move({ key = "overhead_chop", group = "core", pattern = "core_anti_ext", region = "trunk", class = 1,
		dose = "rep", ladder = 10, rung = 2, pain = { "shoulder" }, theme = { "pick" }, mp = { Ab },
		ms = { Sh, La, Ob } }),
	[72] = move({ key = "plank", group = "core", pattern = "core_anti_ext", region = "trunk", class = 1, dose = "hold",
		facing = "front", ladder = 10, rung = 3, anchor = true, regress = 71, progress = 73, twin = 236,
		flags = { "floor", "strain" }, mp = { Ab }, ms = { Ob, Sh } }),
	-- McGill's curl-up on rung 2 since 2026-10-16 (slice 42), beside the other first trunk drills
	[80] = move({ key = "curl_up", group = "core", pattern = "core_anti_ext", region = "trunk", class = 1,
		dose = "rep", ladder = 10, rung = 2, flags = { "floor" }, mp = { Ab }, ms = { Ob } }),
	[81] = move({ key = "bear_plank", group = "core", pattern = "core_anti_ext", region = "trunk", class = 2,
		dose = "hold", facing = "front", ladder = 10, rung = 3, strainFree = true, flags = { "floor" }, pain = { "wrist" },
		theme = { "bear" }, mp = { Ab, Qu }, ms = { Sh, Tr } }),
	[84] = move({ key = "inchworm", group = "core", pattern = "core_anti_ext", region = "trunk", class = 2,
		dose = "rep", ladder = 10, rung = 3, flags = { "floor", "head_down" }, pain = { "wrist", "back" },
		theme = { "worm" }, hrAfter = true, mp = { Ab, Sh }, ms = { Ha, Tr } }),
	[133] = move({ key = "crab_walk", group = "core", pattern = "core_anti_ext", region = "trunk", class = 2,
		dose = "hold", facing = "back", ladder = 10, rung = 3, strainFree = true, flags = { "floor", "space" },
		pain = { "wrist", "shoulder" }, theme = { "crab" }, mp = { Tr, Gl }, ms = { Sh, Ab } }),
	[73] = move({ key = "side_plank_knees", group = "core", pattern = "core_anti_rot", region = "trunk", class = 1,
		dose = "hold", facing = "side", ladder = 11, rung = 1, anchor = true, regress = 72, progress = 74, twin = 238,
		side = true, strainFree = true, flags = { "floor" }, pain = { "shoulder" }, mp = { Ob }, ms = { Gl, Sh, LB } }),
	[82] = move({ key = "bear_crawl", group = "core", pattern = "core_anti_ext", region = "trunk", class = 2,
		dose = "hold", facing = "front", ladder = 10, rung = 3, twin = 81, strainFree = true, flags = { "floor", "space" },
		pain = { "wrist" }, theme = { "bear", "sneak" }, mp = { Ab, Qu }, ms = { Sh, Tr, HEART } }),
	[74] = move({ key = "side_plank", group = "core", pattern = "core_anti_rot", region = "trunk", class = 2,
		dose = "hold", facing = "side", ladder = 11, rung = 2, anchor = true, regress = 73, progress = 83, twin = 239,
		side = true, flags = { "floor", "strain" }, pain = { "shoulder" }, mp = { Ob }, ms = { Gl, Sh, LB } }),
	-- the Copenhagen with the bed or sofa edge under the middle of the top thigh, beside the side plank (2026-10-12, the
	-- ladder gaps; Harøy 2019's level below the short lever, Sportsmith: "progress ... by gradually increasing the
	-- support distance"): the top leg's inner-thigh hold, which no side plank trains, at a shorter lever
	[408] = move({ key = "copenhagen_thigh", group = "core", pattern = "core_anti_rot", region = "trunk", class = 2,
		dose = "hold", facing = "side", ladder = 11, rung = 2, regress = 74, side = true, flags = { "floor", "strain" },
		support = "bed", pain = { "hip", "shoulder" }, mp = { Ad }, ms = { Ob, Gl, Sh } }),
	[75] = move({ key = "long_lever_plank", group = "core", pattern = "core_anti_ext", region = "trunk", class = 3,
		dose = "hold", facing = "front", ladder = 10, rung = 4, anchor = true, regress = 72, progress = 76, twin = 236,
		flags = { "floor", "strain", "skill" }, pain = { "back" }, mp = { Ab, Ob }, ms = { Sh, Gl } }),
	-- The Copenhagen's top knee rests on a bed or a sofa (David, 2026-10-04; a stable chair until then), the short
	-- lever the plan starts it with, so floor space alone opens the side's rung 3.
	[83] = move({ key = "copenhagen", group = "core", pattern = "core_anti_rot", region = "trunk", class = 3,
		dose = "hold", facing = "side", ladder = 11, rung = 3, anchor = true, regress = 408, twin = 74, side = true,
		flags = { "floor", "strain", "skill" }, support = "bed", pain = { "hip", "knee", "shoulder" }, mp = { Ad },
		ms = { Ob, Gl, Sh } }),
	-- Plank shoulder taps are done with the feet together since #138 (wide until then, a much easier hold under the same
	-- id): a set before its merge, 2026-10-18 18:24:04 UTC, keeps its record and stars and counts toward no recent form,
	-- no day at the top and no check (`goalSince`, DB.lua ns.BeforeForm; 2026-10-19, the program's double-check).
	[76] = move({ key = "plank_taps", group = "core", pattern = "core_anti_rot", region = "trunk", class = 2,
		dose = "hold", facing = "front", ladder = 10, rung = 5, anchor = true, regress = 75, progress = 77, twin = 236,
		strainFree = true, goalSince = 1792347844, flags = { "floor" }, pain = { "wrist" }, mp = { Ab, Ob },
		ms = { Sh, Gl } }),
	[77] = move({ key = "body_saw", group = "core", pattern = "core_anti_ext", region = "trunk", class = 3,
		dose = "hold", facing = "front", ladder = 10, rung = 6, anchor = true, regress = 75, twin = 236,
		flags = { "floor", "strain", "skill" }, pain = { "back", "shoulder" }, mp = { Ab }, ms = { Ob, Sh, La } }),
	-- The tuck plow raise beside the body saw (2026-10-10; Hybrid Calisthenics' "Tuck Plow Raises"): the hips lifted onto
	-- the upper back by the belly and the hands' press, the dragon flag's lift that the face-down saw does not train
	[380] = move({ key = "tuck_plow_raise", group = "core", pattern = "core_anti_ext", region = "trunk", class = 2,
		dose = "rep", ladder = 10, rung = 6, regress = 75, flags = { "floor", "head_down" }, pain = { "back", "neck" },
		mp = { Ab }, ms = { Ob, La } }),
	-- The core's top (slice 22): the dragon flag, the shoulders on the floor and the hands on a heavy anchor behind
	-- the head (a bed's or a heavy sofa's frame, never a desk or a chair): tucked, then lowered straight, then lowered
	-- and raised straight. An ab wheel adds the standing rollout beside it (the wheel set, below).
	[319] = move({ key = "dragon_flag_tuck", group = "core", pattern = "core_anti_ext", region = "trunk", class = 3,
		dose = "rep", ladder = 10, rung = 7, anchor = true, regress = 380, twin = 237,
		flags = { "floor", "head_down", "strain", "skill" }, pain = { "back", "neck", "shoulder" }, mp = { Ab },
		ms = { Ob, La, Qu } }),
	-- the hips open, the knees still bent, beside the tuck (2026-10-10): the step before the straight body
	[381] = move({ key = "dragon_flag_adv_tuck", group = "core", pattern = "core_anti_ext", region = "trunk", class = 3,
		dose = "rep", ladder = 10, rung = 7, regress = 319, flags = { "floor", "head_down", "strain", "skill" },
		pain = { "back", "neck", "shoulder" }, mp = { Ab }, ms = { Ob, La, Qu } }),
	[320] = move({ key = "dragon_flag_negative", group = "core", pattern = "core_anti_ext", region = "trunk", class = 3,
		dose = "rep", ladder = 10, rung = 8, anchor = true, regress = 319, twin = 237,
		flags = { "floor", "head_down", "strain", "skill" }, pain = { "back", "neck", "shoulder" }, mp = { Ab },
		ms = { Ob, La, Gl } }),
	-- the legs straight and wide, lowered and lifted back up, beside the lowering (2026-10-12, the ladder gaps; the
	-- r/bodyweightfitness wiki's tuck, advanced tuck, straddle, full): the lift from the bottom, which the lowering
	-- never trains, on a shorter lever than the dragon flag's
	[406] = move({ key = "dragon_flag_straddle", group = "core", pattern = "core_anti_ext", region = "trunk", class = 3,
		dose = "rep", ladder = 10, rung = 8, regress = 320, flags = { "floor", "head_down", "strain", "skill" },
		hint = 413, pain = { "back", "neck", "shoulder" }, mp = { Ab }, ms = { Ob, La, Gl } }),
	[321] = move({ key = "dragon_flag", group = "core", pattern = "core_anti_ext", region = "trunk", class = 3,
		dose = "rep", ladder = 10, rung = 9, anchor = true, regress = 406, twin = 237,
		flags = { "floor", "head_down", "strain", "skill" }, pain = { "back", "neck", "shoulder" }, mp = { Ab },
		ms = { Ob, La, Gl } }),
	-- The four winds' back holds (slice 54, 2026-10-23): the core ladder held only front holds, and McGill's guide wants
	-- the back's endurance at least the front's and well above each side's (his normal ratios: front to back under 1.0,
	-- side to back under 0.75). The hinge hold standing at step 2, the bird dog hold on the hands and knees at step 2,
	-- its sides switched as the desk planks' (timed a side, half the step's window), the Superman hold lying face down
	-- at step 3; the hinge's pattern for the two that hold the back straight against a fold, so the variety rules part
	-- them from the planks.
	[639] = move({ key = "hinge_hold", group = "core", pattern = "hinge", region = "trunk", class = 1, dose = "hold",
		facing = "back", ladder = 10, rung = 2, strainFree = true, pain = { "back" }, mp = { LB }, ms = { Gl, Ha } }),
	[641] = move({ key = "bird_dog_hold", group = "core", pattern = "core_anti_rot", region = "trunk", class = 1,
		dose = "hold", facing = "back", ladder = 10, rung = 2, side = true, strainFree = true, flags = { "floor" },
		pain = { "wrist" }, mp = { LB, Gl }, ms = { Ab, Sh } }),
	[640] = move({ key = "superman_hold", group = "core", pattern = "hinge", region = "trunk", class = 2,
		dose = "hold", facing = "back", ladder = 10, rung = 3, regress = 639, flags = { "floor", "strain" },
		pain = { "back" }, mp = { LB, Gl }, ms = { Ha, UB } }),
	-- The core holds at the desk, no floor (2026-09-30): forearms or hands on the desk edge, the level from a longer
	-- lever, one support lifted or the body turned side-on (the floor's own levers, Calatayud 2017, Schoenfeld 2014).
	-- The desk plank with a foot up is the top of the desk path: the core's rungs from 3 have no desk version that
	-- matches, so without floor space the core tops out at rung 2. The tuck plank stood for rung 3 until 2026-10-04:
	-- hands on the desk, arms straight and a knee drawn in, it loads less than the forearm plank a rung below it
	-- (hands higher, lever shorter: Ebben 2011, Suprak 2011), so it is a rung-1 variant now.
	[236] = move({ key = "desk_plank_foot_up", group = "core", pattern = "core_anti_ext", region = "trunk", class = 1,
		dose = "hold", facing = "front", ladder = 10, rung = 2, regress = 70, twin = 70, side = true, strainFree = true,
		mp = { Ab }, ms = { Ob, Gl, Sh } }),
	[237] = move({ key = "desk_tuck_plank", group = "core", pattern = "core_anti_ext", region = "trunk", class = 1,
		dose = "hold", facing = "front", ladder = 10, rung = 1, side = true, strainFree = true, pain = { "wrist" },
		mp = { Ab }, ms = { Qu, Ob, Sh } }),
	[238] = move({ key = "desk_side_plank", group = "core", pattern = "core_anti_rot", region = "trunk", class = 2,
		dose = "hold", facing = "side", ladder = 11, rung = 1, regress = 236, side = true, strainFree = true,
		pain = { "shoulder" }, mp = { Ob }, ms = { Gl, Sh, LB } }),
	[239] = move({ key = "desk_side_plank_leg", group = "core", pattern = "core_anti_rot", region = "trunk", class = 2,
		dose = "hold", facing = "side", ladder = 11, rung = 2, regress = 238, twin = 238, side = true, strainFree = true,
		pain = { "shoulder" }, mp = { Ob, Ad }, ms = { Gl, Sh } }),
	-- The side past the Copenhagen (slice 22): its long lever (the ankle on the edge, not the knee) and its raises, the
	-- top ankle on a bed or a sofa as the Copenhagen's knee (2026-10-12, the ladder gaps; a stable chair until then, so
	-- a player with a sofa and no Stable chair box topped out at rung 4), with the star side plank as their twin, the
	-- side's blue. Purple and orange are the flags on a pole outdoors (the pole set, below).
	[323] = move({ key = "copenhagen_long", group = "core", pattern = "core_anti_rot", region = "trunk", class = 3,
		dose = "hold", facing = "side", ladder = 11, rung = 4, anchor = true, regress = 83, twin = 324, side = true,
		flags = { "floor", "strain", "skill" }, support = "bed", pain = { "hip", "knee", "shoulder" }, mp = { Ad },
		ms = { Ob, Gl, Sh } }),
	[324] = move({ key = "side_plank_star", group = "core", pattern = "core_anti_rot", region = "trunk", class = 2,
		dose = "hold", facing = "side", ladder = 11, rung = 4, regress = 74, twin = 239, side = true,
		flags = { "floor", "strain" }, pain = { "shoulder" }, mp = { Ob }, ms = { Gl, Ad, Sh } }),
	[325] = move({ key = "copenhagen_raises", group = "core", pattern = "core_anti_rot", region = "trunk", class = 3,
		dose = "rep", ladder = 11, rung = 5, anchor = true, regress = 323, twin = 324, side = true,
		flags = { "floor", "strain", "skill" }, support = "bed", pain = { "hip", "knee", "shoulder" }, mp = { Ad },
		ms = { Ob, Gl, Sh } }),

	-- The L-sit (slice 22; Stamina), on the bare floor since 2026-10-08 (David: "the l sit should be worked towards
	-- just on the floor"; the r/bodyweightfitness wiki: "L-sits are best done on the floor"): palms flat beside the
	-- hips, from both heels down through one heel down, the tuck and one straight leg to the L-sit, then the legs
	-- rising to the V-sit; since 2026-10-10 the high-tuck V-sit anchors rung 6 with the low V-sit beside it, the V-sit's
	-- named proof. Each floor rung sits one step above the same form
	-- with the hands raised (the floor leaves the feet less room). The raised forms (hands on parallettes or two stacks
	-- of books, the anchors of rungs 1-4 until 2026-10-08) keep their ids, rungs and steps as variants for a player
	-- with the box, each on its own record. No rung has a desk twin; since slice 54 (2026-10-23) the chair forms below
	-- twin steps 4-6 for a setup with floor moves off and a stable chair. Its first rung is a white hold, so the family
	-- opens at once, as the side does.
	[358] = move({ key = "lsit_heels", group = "core", pattern = "core_anti_ext", region = "trunk", class = 1,
		dose = "hold", ladder = 21, rung = 1, anchor = true, twin = 644, strainFree = true, flags = { "floor" },
		pain = { "wrist", "shoulder" }, mp = { Ab }, ms = { Tr, Sh, Qu, Fa } }),
	[359] = move({ key = "lsit_one_heel", group = "core", pattern = "core_anti_ext", region = "trunk", class = 2,
		dose = "hold", ladder = 21, rung = 2, regress = 358, side = true, flags = { "floor", "strain" },
		checks = { { ladder = LAD.pike, step = 4 } },
		pain = { "wrist", "shoulder", "hip" }, mp = { Ab, Qu }, ms = { Tr, Sh, Fa } }),
	[360] = move({ key = "lsit_tuck_floor", group = "core", pattern = "core_anti_ext", region = "trunk", class = 2,
		dose = "hold", ladder = 21, rung = 3, anchor = true, twin = 646, regress = 362,
		flags = { "floor", "strain", "skill" }, pain = { "wrist", "shoulder" }, mp = { Ab, Qu }, ms = { Tr, Sh, Fa } }),
	[361] = move({ key = "lsit_one_leg_floor", group = "core", pattern = "core_anti_ext", region = "trunk", class = 3,
		dose = "hold", ladder = 21, rung = 4, anchor = true, regress = 360, side = true,
		flags = { "floor", "strain", "skill" }, pain = { "wrist", "shoulder", "hip" }, mp = { Ab, Qu },
		ms = { Tr, Sh, Fa } }),
	[338] = move({ key = "lsit_floor", group = "core", pattern = "core_anti_ext", region = "trunk", class = 3,
		dose = "hold", ladder = 21, rung = 5, anchor = true, regress = 361, flags = { "floor", "strain", "skill" },
		pain = { "wrist", "shoulder", "hip" }, mp = { Ab, Qu }, ms = { Tr, Sh, Fa } }),
	-- The tuck's own easier version and its other way at its level, on the bare floor too (David, 2026-10-09: "The Tuck
	-- L sit has no alternative move and should also have an easier version move imo"): the tuck with one foot down,
	-- the step right under it (Antranik's floor progression, "an intermediary step for the tucked L-sit"; GMB: "bring
	-- one foot off the ground at a time"), and the cross-legged lift, the same push and lift with the shins crossed
	-- (Yoga Journal's scale pose from Easy Seat), never a foot on a thigh. The tuck with one foot down anchors rung 2
	-- since 2026-10-10 (the ladder rethink: its lifted knee is the tuck's, which the one heel down never trains), the
	-- one heel down beside it; the cross-legged lift is a variant beside the tuck, on its own record. The tuck and the
	-- lift carry skill, so a pump offers their easier version at the player's own rung too (Engine.lua easierMove), as
	-- the rungs above do.
	[362] = move({ key = "lsit_tuck_one_foot", group = "core", pattern = "core_anti_ext", region = "trunk", class = 2,
		dose = "hold", ladder = 21, rung = 2, anchor = true, twin = 645, regress = 359, side = true,
		flags = { "floor", "strain" }, pain = { "wrist", "shoulder" }, mp = { Ab, Qu }, ms = { Tr, Sh, Fa } }),
	[363] = move({ key = "lsit_cross", group = "core", pattern = "core_anti_ext", region = "trunk", class = 2,
		dose = "hold", ladder = 21, rung = 3, regress = 362, flags = { "floor", "strain", "skill" },
		pain = { "wrist", "shoulder", "hip", "knee" }, mp = { Ab, Qu }, ms = { Tr, Sh, Fa } }),
	-- The chair forms (slice 54, 2026-10-23), hands on the seat of a stable chair beside the hips, for a setup with
	-- floor moves off, which met no L-sit at all: the tuck, one leg out and the L-sit, each the twin of steps 4-6's
	-- floor anchor; the feet have the room the floor never leaves, so each sits two steps below the floor form of its
	-- shape (the floor tuck, one leg out and L-sit hold steps 6-8), and the straight legs wait for the pike's range as
	-- the raised forms do. Above step 6 the floor stays the L-sit's own.
	[644] = move({ key = "chair_tuck_hold", group = "core", pattern = "core_anti_ext", region = "trunk", class = 2,
		dose = "hold", ladder = 21, rung = 1, flags = { "seated", "strain" }, needs = { "stable_chair" },
		support = "chair", pain = { "wrist", "shoulder" }, mp = { Ab, Qu }, ms = { Tr, Sh, Fa } }),
	[645] = move({ key = "chair_one_leg_hold", group = "core", pattern = "core_anti_ext", region = "trunk", class = 2,
		dose = "hold", ladder = 21, rung = 2, regress = 644, side = true, flags = { "seated", "strain", "skill" },
		needs = { "stable_chair" }, support = "chair", checks = { { ladder = LAD.pike, step = 4 } },
		pain = { "wrist", "shoulder", "hip" }, mp = { Ab, Qu }, ms = { Tr, Sh, Fa } }),
	[646] = move({ key = "chair_lsit", group = "core", pattern = "core_anti_ext", region = "trunk", class = 3,
		dose = "hold", ladder = 21, rung = 3, regress = 645, flags = { "seated", "strain", "skill" },
		needs = { "stable_chair" }, support = "chair", checks = { { ladder = LAD.pike, step = 4 } },
		pain = { "wrist", "shoulder", "hip" }, mp = { Ab, Qu }, ms = { Tr, Sh, Fa } }),
	-- the raised forms, hands on parallettes or two equal stacks of hard books
	[334] = move({ key = "lsit_foot_supported", group = "core", pattern = "core_anti_ext", region = "trunk", class = 1,
		dose = "hold", ladder = 21, rung = 1, strainFree = true, flags = { "floor" },
		needs = { "parallettes" }, pain = { "wrist", "shoulder" }, mp = { Ab }, ms = { Tr, Sh, Qu } }),
	[335] = move({ key = "lsit_tuck", group = "core", pattern = "core_anti_ext", region = "trunk", class = 2,
		dose = "hold", ladder = 21, rung = 2, regress = 334, flags = { "floor", "strain" },
		needs = { "parallettes" }, pain = { "wrist", "shoulder" }, mp = { Ab, Qu }, ms = { Tr, Sh } }),
	[336] = move({ key = "lsit_one_leg", group = "core", pattern = "core_anti_ext", region = "trunk", class = 2,
		dose = "hold", ladder = 21, rung = 3, regress = 335, side = true,
		flags = { "floor", "strain", "skill" }, needs = { "parallettes" }, checks = { { ladder = LAD.pike, step = 4 } },
		pain = { "wrist", "shoulder", "hip" }, mp = { Ab, Qu }, ms = { Tr, Sh } }),
	[337] = move({ key = "lsit_books", group = "core", pattern = "core_anti_ext", region = "trunk", class = 3,
		dose = "hold", ladder = 21, rung = 4, regress = 336, flags = { "floor", "strain", "skill" },
		needs = { "parallettes" }, checks = { { ladder = LAD.pike, step = 4 } }, pain = { "wrist", "shoulder", "hip" },
		mp = { Ab, Qu }, ms = { Tr, Sh } }),
	-- The floor L-sit's compression drill (2026-10-10, the ladder rethink): seated, legs straight, both heels lifted
	-- by the hip flexors alone, the V-sit's limit (Antranik's "piked compression leg lifts"), which the L-sit's own
	-- hold never trains; the high-tuck V-sit anchors rung 6 since that day, the low V-sit beside it.
	[385] = move({ key = "seated_leg_lifts", group = "core", pattern = "core_anti_ext", region = "trunk", class = 2,
		dose = "rep", ladder = 21, rung = 5, progress = 339, flags = { "floor" }, pain = { "hip" },
		mp = { Qu, Ab } }),
	[339] = move({ key = "vsit_tuck", group = "core", pattern = "core_anti_ext", region = "trunk", class = 3,
		dose = "hold", ladder = 21, rung = 6, anchor = true, regress = 338, flags = { "floor", "strain", "skill" },
		pain = { "wrist", "shoulder", "hip" }, mp = { Ab, Qu }, ms = { Tr, Sh, Fa } }),
	[340] = move({ key = "vsit_low", group = "core", pattern = "core_anti_ext", region = "trunk", class = 3,
		dose = "hold", ladder = 21, rung = 6, regress = 339, flags = { "floor", "strain", "skill" },
		pain = { "wrist", "shoulder", "hip" }, mp = { Ab, Qu }, ms = { Tr, Sh, Fa } }),
	-- one leg straight up from the high tuck, beside it (2026-10-12, the ladder gaps; Antranik: "From the high tuck,
	-- extend just one leg"): a leg at the V-sit's hip angle, which the low V-sit's halfway legs never reach
	[410] = move({ key = "vsit_one_leg", group = "core", pattern = "core_anti_ext", region = "trunk", class = 3,
		dose = "hold", ladder = 21, rung = 6, regress = 339, side = true, flags = { "floor", "strain", "skill" },
		pain = { "wrist", "shoulder", "hip" }, mp = { Ab, Qu }, ms = { Tr, Sh, Fa } }),
	[341] = move({ key = "vsit", group = "core", pattern = "core_anti_ext", region = "trunk", class = 3,
		dose = "hold", ladder = 21, rung = 7, anchor = true, regress = 340, flags = { "floor", "strain", "skill" },
		pain = { "wrist", "shoulder", "hip" }, mp = { Ab, Qu }, ms = { Tr, Sh, Fa } }),

	-- The sit-up (ladder 24, 2026-10-17, David: "yes add"): the trunk curled in reps on the floor, from the crunch to the
	-- V-up and the sit-up with a weight, every move tagged for the back, the neck where the head lifts and the hip on
	-- every full sit-up and where the legs lift straight or the feet are held (the psoas: Juker 1998), the hands on the
	-- chest or at the temples (the crunch's, as its figure shows) and never behind the head wherever it curls up.
	-- Dynamic flexion is low risk without a back problem
	-- (Contreras and Schoenfeld 2011; Childs 2010: 1,141 soldiers, no more injuries with sit-ups than with a stability
	-- program). The leg raises and the hollow rock keep the lower back flat against the legs' lever (core_anti_ext); the
	-- curls bend it (core_flex). Beside the crunch the knee
	-- raise, beside the reverse crunch the slow sit-up lowering and the sit-up with the feet under a sofa, beside the
	-- sit-up the leg raise bent up and lowered straight (Hybrid Calisthenics' "alternating" leg raise), beside the tuck-up
	-- the straight leg raise and the one-leg V-up (Rogue's V-up regressions), beside the V-up the hollow rock
	[443] = move({ key = "crunch", group = "situp", pattern = "core_flex", region = "trunk", class = 1, dose = "rep",
		ladder = 24, rung = 1, anchor = true, progress = 445, flags = { "floor" }, pain = { "back", "neck" }, mp = { Ab },
		ms = { Ob } }),
	[444] = move({ key = "lying_knee_raise", group = "situp", pattern = "core_anti_ext", region = "trunk", class = 1,
		dose = "rep", ladder = 24, rung = 1, progress = 445, flags = { "floor" }, pain = { "back" }, mp = { Ab, Qu } }),
	[445] = move({ key = "reverse_crunch", group = "situp", pattern = "core_flex", region = "trunk", class = 1,
		dose = "rep", ladder = 24, rung = 2, anchor = true, regress = 444, progress = 448, flags = { "floor" },
		pain = { "back" }, mp = { Ab }, ms = { Ob, Qu } }),
	[446] = move({ key = "situp_lowering", group = "situp", pattern = "core_flex", region = "trunk", class = 2,
		dose = "rep", ladder = 24, rung = 2, regress = 445, progress = 448, flags = { "floor" },
		pain = { "back", "neck", "hip" }, mp = { Ab }, ms = { Ob, Qu } }),
	[447] = move({ key = "situp_anchored", group = "situp", pattern = "core_flex", region = "trunk", class = 2,
		dose = "rep", ladder = 24, rung = 2, regress = 445, progress = 448, flags = { "floor" }, support = "bed",
		pain = { "back", "neck", "hip" }, mp = { Ab, Qu }, ms = { Ob } }),
	[448] = move({ key = "situp", group = "situp", pattern = "core_flex", region = "trunk", class = 2, dose = "rep",
		ladder = 24, rung = 3, anchor = true, regress = 446, progress = 450, flags = { "floor" },
		pain = { "back", "neck", "hip" }, mp = { Ab }, ms = { Ob, Qu } }),
	[449] = move({ key = "leg_raise_bent", group = "situp", pattern = "core_anti_ext", region = "trunk", class = 2,
		dose = "rep", ladder = 24, rung = 3, regress = 444, progress = 451, flags = { "floor" }, pain = { "back", "hip" },
		mp = { Ab, Qu } }),
	[450] = move({ key = "tuck_up", group = "situp", pattern = "core_flex", region = "trunk", class = 2, dose = "rep",
		ladder = 24, rung = 4, anchor = true, regress = 448, progress = 453, flags = { "floor" },
		pain = { "back", "neck", "hip" }, mp = { Ab, Qu }, ms = { Ob } }),
	[451] = move({ key = "leg_raise", group = "situp", pattern = "core_anti_ext", region = "trunk", class = 2,
		dose = "rep", ladder = 24, rung = 4, regress = 449, progress = 454, flags = { "floor" }, pain = { "back", "hip" },
		mp = { Ab, Qu } }),
	[452] = move({ key = "vup_one_leg", group = "situp", pattern = "core_flex", region = "trunk", class = 2, dose = "rep",
		ladder = 24, rung = 4, regress = 450, progress = 453, pairs = true, flags = { "floor" },
		pain = { "back", "neck", "hip" }, mp = { Ab, Qu }, ms = { Ob } }),
	[453] = move({ key = "vup", group = "situp", pattern = "core_flex", region = "trunk", class = 3, dose = "rep",
		ladder = 24, rung = 5, anchor = true, regress = 452, progress = 455, flags = { "floor", "skill" },
		pain = { "back", "neck", "hip" }, mp = { Ab, Qu }, ms = { Ob } }),
	[454] = move({ key = "hollow_rock", group = "situp", pattern = "core_anti_ext", region = "trunk", class = 2,
		dose = "rep", ladder = 24, rung = 5, regress = 451, flags = { "floor" }, pain = { "back", "neck", "hip" },
		mp = { Ab }, ms = { Qu, Ob } }),
	-- the top rung climbs by its weight, hugged to the chest with the feet free (the anchored form loads the spine most:
	-- McGill), 6-15 reps (Contreras and Schoenfeld 2011: "External resistance should be used when necessary")
	[455] = move({ key = "situp_weighted", group = "situp", pattern = "core_flex", region = "trunk", class = 2,
		dose = "rep", ladder = 24, rung = 6, anchor = true, regress = 448, flags = { "floor" },
		pain = { "back", "neck", "hip" }, load = "total", kg = { 5, 10 }, holds = { "dumbbells", "kettlebell" },
		mp = { Ab }, ms = { Ob, Qu } }),

	-- Calf and ankle
	[86] = move({ key = "heel_raise", group = "calf", pattern = "calf", region = "legs", class = 1, dose = "rep",
		rung = 1, pain = { "achilles" }, mp = { Ca } }),
	[88] = move({ key = "tibialis_raise", group = "calf", pattern = "calf", region = "legs", class = 1, dose = "rep",
		rung = 1, mp = { Ca } }),
	[87] = move({ key = "sl_heel_raise_slow", group = "calf", pattern = "calf", region = "legs", class = 2,
		dose = "rep", rung = 3, regress = 86, side = true, flags = { "tendon" }, pain = { "achilles" }, mp = { Ca },
		ms = { BALANCE } }),
	[89] = move({ key = "ankle_rocks", group = "calf", pattern = "mobility", class = 1, dose = "flow", noGoal = true,
		secs = 30, ladder = 17, rung = 1, side = true, pain = { "achilles" }, mp = { Ca }, ms = { BREATH } }),

	-- Cardio and ballistic, the travel family's home. From cardio rung 7 a steady move is a form of rungs 4 to 6 (slice
	-- 52, 2026-10-23; of rung 3 and below until then), and the rung's own forms (endOnly) run only as an all-out window,
	-- never a whole long move at an all-out form.
	-- Step jacks are the floor: always legal, the one move without a checkbox on the Exercises page, the cardio
	-- ladder's first rung and a rung-1 finisher form that every filter falls back to (marching in place until
	-- 2026-10-03).
	[95] = move({ key = "step_jacks", group = "cardio", pattern = "cardio", class = 1, dose = "cardio", road = "standing",
		ladder = 6, rung = 1, anchor = true, progress = 101, finisher = 1, mp = { HEART }, ms = { Ca, Gl, Ad, Sh } }),
	[101] = move({ key = "shadow_boxing", group = "cardio", pattern = "strike", class = 2, dose = "cardio",
		road = "strike", ladder = 6, rung = 2, anchor = true, regress = 95, progress = 90, twin = 291, pain = { "elbow" },
		theme = { "fight" }, finisher = 1, mp = { HEART }, ms = { Sh, Ob, Tr } }),
	-- Rung 2 and 3 without jumping, an elbow or a sound (2026-10-04, in the high-knee and the power march's place): a
	-- knee driven up to the other hand as fast as it goes, then tiny fast steps that never leave the floor; each the
	-- twin that stands for its rung where the anchor's elbow or jump is filtered out
	[291] = move({ key = "cross_knee_drives", group = "cardio", pattern = "cardio", class = 2, dose = "cardio",
		road = "standing", ladder = 6, rung = 2, regress = 95, theme = { "fight" }, finisher = 1, mp = { HEART },
		ms = { Ob, Ab, Qu } }),
	[292] = move({ key = "quick_steps", group = "cardio", pattern = "run", class = 2, dose = "cardio", road = "run",
		ladder = 6, rung = 3, regress = 291, theme = { "run" }, mp = { HEART }, ms = { Qu, Ca, Gl } }),
	[90] = move({ key = "jog", group = "cardio", pattern = "run", class = 2, dose = "cardio", road = "run", ladder = 6,
		rung = 3, anchor = true, regress = 101, progress = 91, twin = 292, flags = { "impact" }, jump = true,
		theme = { "run" }, mp = { HEART }, ms = { Qu, Ca, Gl } }),
	[92] = move({ key = "butt_kicks", group = "cardio", pattern = "run", class = 2, dose = "cardio", road = "run",
		ladder = 6, rung = 3, flags = { "impact" }, jump = true, theme = { "run" }, mp = { HEART }, ms = { Ha, Ca } }),
	[91] = move({ key = "high_knees", group = "cardio", pattern = "run", class = 3, dose = "cardio", road = "run",
		ladder = 6, rung = 4, anchor = true, regress = 90, progress = 94, flags = { "impact", "loud" }, jump = true,
		pain = { "achilles" }, theme = { "run" }, mp = { HEART }, ms = { Qu, Ca, Ab } }),
	[93] = move({ key = "fast_feet", group = "cardio", pattern = "run", class = 3, dose = "cardio", road = "run",
		ladder = 6, rung = 4, flags = { "impact" }, jump = true, theme = { "run" }, finisher = 1, mp = { HEART },
		ms = { Qu, Ca, Gl } }),
	[94] = move({ key = "jumping_jacks", group = "cardio", pattern = "cardio", class = 3, dose = "cardio", road = "jump",
		ladder = 6, rung = 5, anchor = true, regress = 91, progress = 85, flags = { "impact", "loud" }, jump = true,
		pain = { "achilles" }, finisher = 5, count = true, mp = { HEART }, ms = { Ca, Gl, Ad, Sh } }),
	[96] = move({ key = "seal_jacks", group = "cardio", pattern = "cardio", class = 2, dose = "cardio", road = "jump",
		ladder = 6, rung = 5, flags = { "impact", "loud" }, jump = true, pain = { "achilles" }, finisher = 5, count = true,
		mp = { HEART }, ms = { Ch, UB, Ca } }),
	-- Skater steps beside the jacks (the ladder gaps, 2026-10-12): the skater hop without the flight, each landing on one
	-- foot held for a moment, the one-foot landing that rung 6's skater hops ask and two-footed jacks never train (The
	-- Prehab Guys: "a single leg landing while sidestepping" before the hop; the NSCA rates one-leg lateral landings
	-- above two-footed hops); with no jump it is a counted form for a setup that leaves jumping out too.
	[411] = move({ key = "skater_steps", group = "cardio", pattern = "cardio", class = 3, dose = "cardio",
		road = "standing", ladder = 6, rung = 5, regress = 292, flags = { "space" }, pain = { "knee" }, finisher = 5,
		count = true, mp = { Gl, Qu }, ms = { Ad, Ca, HEART, BALANCE } }),
	-- skater hops land on one foot: rung 6, after the two-footed jacks (2026-10-10, the ladder rethink)
	[30] = move({ key = "skater_hops", group = "cardio", pattern = "ballistic", class = 3, dose = "cardio", road = "jump",
		ladder = 6, rung = 6, regress = 411, flags = { "impact", "loud", "space" }, jump = true,
		pain = { "knee", "achilles" }, theme = { "ride" }, finisher = 6, count = true, mp = { Gl, Qu },
		ms = { Ad, Ca, HEART, BALANCE } }),
	[99] = move({ key = "pogo_hops", group = "cardio", pattern = "ballistic", class = 3, dose = "cardio", road = "jump",
		ladder = 6, rung = 5, flags = { "tendon", "loud", "impact" }, jump = true, pain = { "achilles" }, mp = { HEART },
		ms = { Ca } }),
	[85] = move({ key = "mountain_climbers", group = "cardio", pattern = "ballistic", class = 3, dose = "cardio",
		road = "floor", ladder = 6, rung = 6, anchor = true, regress = 94, progress = 98, twin = 243, flags = { "floor" },
		pain = { "wrist" }, theme = { "climb", "swim" }, finisher = 6, count = true, hrAfter = true, mp = { Ab, Qu },
		ms = { Sh, Ob, HEART } }),
	[97] = move({ key = "squat_thrusts", group = "cardio", pattern = "ballistic", class = 3, dose = "cardio",
		road = "floor", ladder = 6, rung = 6, flags = { "floor", "impact" }, jump = true, pain = { "wrist" }, finisher = 6,
		count = true, hrAfter = true, mp = { HEART }, ms = { Qu, Ch, Sh, Ab } }),
	[141] = move({ key = "sprawl", group = "cardio", pattern = "ballistic", class = 3, dose = "cardio", road = "floor",
		ladder = 6, rung = 6, flags = { "floor", "impact" }, jump = true, pain = { "wrist" }, theme = { "fight" },
		finisher = 6, count = true, hrAfter = true, mp = { HEART }, ms = { Ab, Ch, Qu } }),
	-- The burpees climb by the work a rep takes (2026-10-05, David: "burpees are harder than sprint on spot no? And
	-- their difficulty can be further increased?"): the burpee at 7 (feet back, in, a jump), the chest-to-floor
	-- burpee at 8, the tuck jump at 9. In the lab a burpee costs about 40 ml/kg/min and 95 % of peak heart rate in
	-- 30-s bouts, the highest of the bodyweight forms measured (Machado 2020), and against running sprints the same
	-- heart rate with more lactate and effort (Bingley 2019); no study measured sprinting on the spot, a run without
	-- the forward drive, so it moved down to an all-out end from rung 4.
	[98] = move({ key = "burpee", group = "cardio", pattern = "ballistic", class = 3, dose = "cardio", road = "floor",
		ladder = 6, rung = 7, anchor = true, regress = 97, progress = 357, twin = 244, endOnly = true, finisher = 7,
		count = true, flags = { "floor", "loud", "impact" }, jump = true, pain = { "wrist", "knee", "achilles" },
		hrAfter = true, mp = { HEART }, ms = { Qu, Ch, Sh, Ab, Gl, Tr } }),
	[357] = move({ key = "burpee_chest", group = "cardio", pattern = "ballistic", class = 3, dose = "cardio",
		road = "floor", ladder = 6, rung = 8, anchor = true, regress = 98, progress = 307, twin = 244, endOnly = true,
		finisher = 8, count = true, flags = { "floor", "loud", "impact" }, jump = true,
		pain = { "wrist", "knee", "achilles" }, hrAfter = true, mp = { HEART }, ms = { Qu, Ch, Sh, Ab, Gl, Tr } }),
	[100] = move({ key = "sprint", group = "cardio", pattern = "run", class = 3, dose = "cardio", road = "run", ladder = 6,
		rung = 4, regress = 91, endOnly = true, finisher = 4, flags = { "impact" }, jump = true, pain = { "achilles" },
		theme = { "run" }, mp = { HEART }, ms = { Qu, Ca, Gl, Ha } }),
	-- Cardio's orange top (slice 22): the burpee with a tuck jump, an all-out form that opens at rung 9. All-out
	-- whole-body intervals are the best-measured hard cardio beside a desk (Gist 2014, McRae 2012), and no source
	-- ranks them beside a planche: orange here means the hardest form, not an equal feat.
	[307] = move({ key = "burpee_tuck", group = "cardio", pattern = "ballistic", class = 3, dose = "cardio",
		road = "floor", ladder = 6, rung = 9, anchor = true, regress = 357, twin = 94, endOnly = true, finisher = 9,
		count = true, flags = { "floor", "loud", "impact" }, jump = true, pain = { "wrist", "knee", "achilles" },
		hrAfter = true, mp = { HEART }, ms = { Qu, Ch, Sh, Ab, Gl, Ca } }),
	-- Cardio with no jumping, quietly and without floor space (2026-09-30): standing forms that climb by tempo and range
	-- (the quick steps above, a squat that ends in a reach, punches with a duck, climbers on the wall, a burpee stepped
	-- back from the desk), for the rungs above 2 whose forms otherwise jump or need the floor; rungs 8 and 9 have none.
	[241] = move({ key = "squat_reach", group = "cardio", pattern = "cardio", class = 2, dose = "cardio",
		road = "standing", ladder = 6, rung = 4, regress = 292, pain = { "knee" }, mp = { HEART }, ms = { Qu, Gl, Sh } }),
	[242] = move({ key = "punch_duck", group = "cardio", pattern = "strike", class = 3, dose = "cardio", road = "strike",
		ladder = 6, rung = 5, regress = 101, pain = { "elbow", "knee" }, theme = { "fight" }, mp = { HEART },
		ms = { Sh, Ob, Qu } }),
	[243] = move({ key = "wall_climbers", group = "cardio", pattern = "cardio", class = 3, dose = "cardio",
		road = "standing", ladder = 6, rung = 6, regress = 242, pain = { "wrist" }, theme = { "climb" }, finisher = 6,
		count = true, mp = { HEART }, ms = { Ab, Qu, Sh } }),
	[244] = move({ key = "desk_burpee", group = "cardio", pattern = "cardio", class = 3, dose = "cardio",
		road = "standing", ladder = 6, rung = 7, regress = 243, pain = { "wrist", "knee" }, finisher = 7, count = true,
		hrAfter = true, mp = { HEART }, ms = { Qu, Ch, Ab } }),

	-- Five roads (slice 54, 2026-10-23): every cardio step holds forms on the run, strike, jump, floor and standing
	-- roads, so the top of the tree holds five forms where it held one, and quiet, no-jump and floor-off setups climb to
	-- step 11 (until then each stopped at step 8: no form of steps 10 and 11 passed them, and step 5 held the skater
	-- steps alone). Each form's regress is the form below it on its road. The ladder readiness rule takes the higher
	-- standard: a quiet form keeps every part of its step's jump form but the jump and adds a harder part in its place
	-- (a push-up, a spider step, a knee drive), so it is never the easy way up; a strike form's rep is one whole cycle,
	-- counted as one rep, as the chest burpee's push-up is. The two at step 1 are timed steady moves, the rest counted;
	-- from step 8 up they are end-only; only the side steps need room to move, a step's width to each side, as the
	-- lateral lunge and the skater steps do.
	[620] = move({ key = "side_steps", group = "cardio", pattern = "cardio", class = 1, dose = "cardio",
		road = "standing", ladder = 6, rung = 1, flags = { "space" }, finisher = 1, mp = { HEART },
		ms = { Ad, Ca, Gl } }),
	[621] = move({ key = "punch_march", group = "cardio", pattern = "strike", class = 1, dose = "cardio",
		road = "strike", ladder = 6, rung = 1, finisher = 1, pain = { "elbow" }, theme = { "fight" }, mp = { HEART },
		ms = { Sh, Ob } }),
	[622] = move({ key = "knee_strikes", group = "cardio", pattern = "strike", class = 3, dose = "cardio",
		road = "strike", ladder = 6, rung = 5, regress = 291, pain = { "hip" }, theme = { "fight" }, finisher = 5,
		count = true, mp = { HEART }, ms = { Ab, Qu, Ob, BALANCE } }),
	[623] = move({ key = "speed_squats", group = "cardio", pattern = "cardio", class = 3, dose = "cardio",
		road = "standing", ladder = 6, rung = 5, regress = 241, pain = { "knee" }, finisher = 5, count = true,
		mp = { HEART }, ms = { Qu, Gl } }),
	[624] = move({ key = "plank_jacks", group = "cardio", pattern = "ballistic", class = 3, dose = "cardio",
		road = "floor", ladder = 6, rung = 5, flags = { "floor", "impact" }, jump = true, pain = { "wrist" },
		finisher = 5, count = true, hrAfter = true, mp = { HEART }, ms = { Ab, Sh, Ad } }),
	[625] = move({ key = "cross_climbers", group = "cardio", pattern = "ballistic", class = 3, dose = "cardio",
		road = "floor", ladder = 6, rung = 6, regress = 624, flags = { "floor" }, pain = { "wrist" },
		theme = { "climb" }, finisher = 6, count = true, hrAfter = true, mp = { HEART }, ms = { Ob, Ab, Sh } }),
	[626] = move({ key = "lunge_knee_drives", group = "cardio", pattern = "cardio", class = 3, dose = "cardio",
		road = "standing", ladder = 6, rung = 6, regress = 623, pain = { "knee" }, finisher = 6, count = true,
		mp = { HEART }, ms = { Qu, Gl, Ab, BALANCE } }),
	[627] = move({ key = "step_back_burpee", group = "cardio", pattern = "cardio", class = 3, dose = "cardio",
		road = "floor", ladder = 6, rung = 7, regress = 85, endOnly = true, flags = { "floor" },
		pain = { "wrist", "knee" }, finisher = 7, count = true, hrAfter = true, mp = { HEART }, ms = { Qu, Ch, Ab } }),
	[628] = move({ key = "jump_lunges", group = "cardio", pattern = "ballistic", class = 3, dose = "cardio",
		road = "jump", ladder = 6, rung = 7, regress = 30, endOnly = true, flags = { "impact", "loud" }, jump = true,
		pain = { "knee", "achilles" }, finisher = 7, count = true, mp = { HEART }, ms = { Qu, Gl, Ca, BALANCE } }),
	[629] = move({ key = "sprawl_strike", group = "cardio", pattern = "strike", class = 3, dose = "cardio",
		road = "strike", ladder = 6, rung = 7, regress = 141, endOnly = true, flags = { "floor", "impact" },
		jump = true, pain = { "wrist", "elbow" }, theme = { "fight" }, finisher = 7, count = true, hrAfter = true,
		mp = { HEART }, ms = { Ab, Ch, Sh } }),
	[630] = move({ key = "lunge_strike", group = "cardio", pattern = "strike", class = 3, dose = "cardio",
		road = "strike", ladder = 6, rung = 7, regress = 626, endOnly = true, pain = { "knee", "elbow" },
		theme = { "fight" }, finisher = 7, count = true, mp = { HEART }, ms = { Qu, Sh, Ob } }),
	[631] = move({ key = "stepped_chest_burpee", group = "cardio", pattern = "cardio", class = 3, dose = "cardio",
		road = "floor", ladder = 6, rung = 8, regress = 627, endOnly = true, flags = { "floor" },
		pain = { "wrist", "knee" }, finisher = 8, count = true, hrAfter = true, mp = { HEART },
		ms = { Ch, Qu, Ab, Tr } }),
	[632] = move({ key = "desk_pushup_burpee", group = "cardio", pattern = "cardio", class = 3, dose = "cardio",
		road = "standing", ladder = 6, rung = 8, regress = 244, endOnly = true, pain = { "wrist", "knee" },
		finisher = 8, count = true, hrAfter = true, mp = { HEART }, ms = { Ch, Qu, Ab, Tr } }),
	[633] = move({ key = "desk_burpee_knees", group = "cardio", pattern = "strike", class = 3, dose = "cardio",
		road = "strike", ladder = 6, rung = 8, regress = 244, endOnly = true, pain = { "wrist", "knee" },
		theme = { "fight" }, finisher = 8, count = true, hrAfter = true, mp = { HEART }, ms = { Qu, Ab, Sh } }),
	[634] = move({ key = "star_jumps", group = "cardio", pattern = "ballistic", class = 3, dose = "cardio",
		road = "jump", ladder = 6, rung = 8, regress = 628, endOnly = true, flags = { "impact", "loud" }, jump = true,
		pain = { "knee", "achilles" }, finisher = 8, count = true, mp = { HEART }, ms = { Qu, Gl, Ca, Sh } }),
	[635] = move({ key = "stepped_spider_burpee", group = "cardio", pattern = "cardio", class = 3, dose = "cardio",
		road = "floor", ladder = 6, rung = 9, regress = 631, endOnly = true, flags = { "floor" },
		pain = { "wrist", "knee" }, theme = { "spider" }, finisher = 9, count = true, hrAfter = true, mp = { HEART },
		ms = { Ch, Ab, Ob, Qu } }),
	[636] = move({ key = "desk_spider_burpee", group = "cardio", pattern = "cardio", class = 3, dose = "cardio",
		road = "standing", ladder = 6, rung = 9, regress = 632, endOnly = true, pain = { "wrist", "knee" },
		theme = { "spider" }, finisher = 9, count = true, hrAfter = true, mp = { HEART }, ms = { Ch, Ab, Ob, Qu } }),
	[637] = move({ key = "desk_burpee_combo", group = "cardio", pattern = "strike", class = 3, dose = "cardio",
		road = "strike", ladder = 6, rung = 9, regress = 633, endOnly = true, pain = { "wrist", "knee", "elbow" },
		theme = { "fight" }, finisher = 9, count = true, hrAfter = true, mp = { HEART }, ms = { Qu, Ab, Sh, Ob } }),
	[638] = move({ key = "tuck_jump_rally", group = "cardio", pattern = "ballistic", class = 3, dose = "cardio",
		road = "jump", ladder = 6, rung = 9, regress = 634, endOnly = true, flags = { "impact", "loud", "tendon" },
		jump = true, pain = { "knee", "achilles" }, finisher = 9, count = true, mp = { HEART },
		ms = { Qu, Gl, Ca, Ab } }),

	-- Carry: timed, never a record (2026-10-01). With what a home holds the grip and the trunk outlast any max set by
	-- minutes, so a carry runs its seconds with a load heavy enough that the last ones are hard. (The farmer march, the
	-- pack worn, left on 2026-10-03: "too easy".)
	[103] = move({ key = "suitcase_carry", group = "carry", pattern = "carry", class = 1, dose = "flow", secs = 30,
		rung = 1, side = true, flags = { "space" }, needs = { "backpack" }, upgrade = { 158 }, pain = { "wrist" },
		theme = { "carry" }, mp = { Ob, Fa }, ms = { LB, Nk, Gl } }),

	-- Mobility: stretches at their default dose, no record (David, 2026-10-02: breathing and easy drills are "just too
	-- easy"; the slow moves are stretches and yoga poses)
	[108] = move({ key = "pec_stretch", group = "mobility", pattern = "mobility", class = 1, dose = "flow",
		secs = 30, ladder = 15, rung = 1, side = true, mp = { BREATH }, ms = { Ch, Sh } }),
	-- the shoulders' opening behind the back (2026-10-16, slice 42), the pose the dips' and the muscle-up's hint names
	[442] = move({ key = "chest_opener", group = "mobility", pattern = "mobility", class = 1, dose = "flow",
		secs = 30, ladder = 15, rung = 1, pain = { "shoulder" }, mp = { BREATH }, ms = { Ch, Sh } }),
	[109] = move({ key = "open_book", group = "mobility", pattern = "mobility", class = 1, dose = "flow", noGoal = true,
		secs = 30, ladder = 16, rung = 1, side = true, flags = { "floor" }, mp = { BREATH }, ms = { UB } }),
	[111] = move({ key = "hip_flexor_stretch", group = "mobility", pattern = "mobility", class = 1, dose = "flow",
		secs = 30, ladder = 12, rung = 1, side = true, flags = { "floor" }, pain = { "knee" }, mp = { BREATH },
		ms = { Qu, Gl } }),
	[112] = move({ key = "hip_cars", group = "mobility", pattern = "mobility", class = 1, dose = "flow", noGoal = true,
		secs = 30, ladder = 13, rung = 1, side = true, mp = { BREATH }, ms = { Gl, Ad } }),
	[113] = move({ key = "worlds_greatest", group = "mobility", pattern = "mobility", class = 1, dose = "flow",
		noGoal = true, secs = 30, ladder = 12, rung = 1, side = true, flags = { "floor" }, pain = { "knee", "hip", "wrist" },
		mp = { BREATH }, ms = { Qu, Ha, Ad, UB } }),
	[114] = move({ key = "calf_stretch", group = "mobility", pattern = "mobility", class = 1, dose = "flow",
		secs = 30, ladder = 17, rung = 1, anchor = true, side = true, pain = { "achilles" }, mp = { BREATH },
		ms = { Ca } }),

	-- Yoga poses and flows (flow moves; every pose carries Balance as a secondary)
	[118] = move({ key = "downward_dog", group = "yoga", pattern = "mobility", class = 1, dose = "flow", secs = 45,
		ladder = 14, rung = 3, flags = { "floor", "head_down" }, pain = { "wrist" }, mp = { Sh, Ha },
		ms = { Ca, Ob, BALANCE } }),
	[119] = move({ key = "cat_cow", group = "yoga", pattern = "mobility", class = 1, dose = "flow", noGoal = true,
		secs = 45, ladder = 15, rung = 1, flags = { "floor" }, pain = { "wrist" }, mp = { LB, Ab }, ms = { BALANCE } }),
	[120] = move({ key = "cobra", group = "yoga", pattern = "mobility", class = 1, dose = "flow", secs = 40,
		ladder = 15, rung = 2, flags = { "floor" }, pain = { "back", "wrist" }, mp = { LB }, ms = { Gl, Ob, BALANCE } }),
	-- The child's pose points the feet, never the bent ankle the ankle family measures, so it is practice alone (`noGoal`
	-- since 2026-10-20, slice 47's fix round), and it takes the head below the heart (hazard 9).
	[121] = move({ key = "childs_pose", group = "yoga", pattern = "mobility", class = 1, dose = "flow", noGoal = true,
		secs = 45, ladder = 17, rung = 1, flags = { "floor", "head_down" }, pain = { "knee" }, mp = { LB, La },
		ms = { Gl, BALANCE } }),
	[122] = move({ key = "thread_needle", group = "yoga", pattern = "mobility", class = 1, dose = "flow", noGoal = true,
		secs = 30, ladder = 16, rung = 1, side = true, flags = { "floor" }, pain = { "shoulder", "neck" }, mp = { UB },
		ms = { Sh, BALANCE } }),
	[123] = move({ key = "low_lunge", group = "yoga", pattern = "mobility", class = 1, dose = "flow", secs = 30,
		ladder = 12, rung = 2, side = true, flags = { "floor" }, pain = { "knee", "hip" }, mp = { Qu },
		ms = { Gl, Ad, BALANCE } }),
	[124] = move({ key = "pigeon", group = "yoga", pattern = "mobility", class = 1, dose = "flow", secs = 30,
		ladder = 13, rung = 3, side = true, flags = { "floor" }, pain = { "hip", "knee" }, mp = { Gl },
		ms = { Qu, BALANCE } }),
	-- the middle split's step 1 beside the butterfly since 2026-10-19 (slice 47; its anchor until then): the outer
	-- hip's turn-out, which the 90-90 and the pigeon take further, where the butterfly trains the inner thigh the side
	-- lunge needs
	[125] = move({ key = "figure_four", group = "yoga", pattern = "mobility", class = 1, dose = "flow", secs = 30,
		ladder = 13, rung = 1, side = true, flags = { "seated" }, pain = { "hip", "knee" }, mp = { Gl },
		ms = { Qu, BALANCE } }),
	[126] = move({ key = "forward_fold", group = "yoga", pattern = "mobility", class = 1, dose = "flow", secs = 40,
		ladder = 14, rung = 2, anchor = true, twin = 433, flags = { "head_down" }, pain = { "back" }, mp = { Ha },
		ms = { LB, Ca, BALANCE } }),
	[127] = move({ key = "spinal_twist", group = "yoga", pattern = "mobility", class = 1, dose = "flow", secs = 30,
		ladder = 16, rung = 2, side = true, flags = { "floor" }, pain = { "back" }, mp = { Ob },
		ms = { LB, Gl, BALANCE } }),
	-- The standing poses train balance and the legs' hold together; they share no pattern with the wall sit and the
	-- squat holds (iso), so a pose does not cost a hold ladder its turn under the variety rule (2026-10-03).
	[128] = move({ key = "warrior2", group = "yoga", pattern = "balance", class = 2, dose = "flow", noGoal = true,
		secs = 30, ladder = 18, rung = 1, side = true, mp = { Qu, Gl }, ms = { Ad, Sh, BALANCE } }),
	[129] = move({ key = "chair_pose", group = "yoga", pattern = "balance", class = 2, dose = "flow", noGoal = true,
		secs = 40, ladder = 18, rung = 2, pain = { "knee" }, mp = { Qu, Gl }, ms = { LB, Sh, BALANCE } }),
	[130] = move({ key = "tree_pose", group = "yoga", pattern = "balance", class = 1, dose = "flow", secs = 30,
		ladder = 18, rung = 1, anchor = true, side = true, mp = { BALANCE }, ms = { Ca, Gl } }),
	[131] = move({ key = "plank_dog_flow", group = "yoga", pattern = "mobility", class = 2, dose = "flow", noGoal = true,
		secs = 60, ladder = 14, rung = 3, flags = { "floor", "head_down" }, pain = { "wrist" }, mp = { Sh, Ab },
		ms = { Ha, Ch, Tr, BALANCE } }),
	[132] = move({ key = "sun_salutation", group = "yoga", pattern = "mobility", class = 2, dose = "flow", noGoal = true,
		secs = 90, ladder = 15, rung = 2, flags = { "floor", "head_down" }, pain = { "wrist" }, mp = { Sh, Ab },
		ms = { Ha, Ch, Tr, BALANCE } }),

	-- The stretching and yoga families (slice 21; docs/plan.md, "Slice 21" and "Slice 47"): every pose held in one shape
	-- names its goal pose (Locales: move.<key>.mark), asked after its hold, and since slice 47 (2026-10-19) any goal pose
	-- of the player's step answered close enough counts the family's day; three such game days open the next step, at
	-- least one on the step's main pose and one answered Perfect (ns.PoseNeeds). The first rungs stand (a setup with
	-- floor moves off, the default until 2026-10-04, starts every family too), the floor and space poses higher up, each
	-- anchor with its twin from below. Steps by hand from Iyengar's ratings in Light on Yoga (research of 2026-10-04, the
	-- plan's slice 21).
	-- The front split and the hip flexors (Spirit)
	[245] = move({ key = "quad_stretch", group = "mobility", pattern = "mobility", class = 1, dose = "flow", secs = 30,
		ladder = 12, rung = 1, anchor = true, side = true, pain = { "knee" }, mp = { BREATH }, ms = { Qu } }),
	[246] = move({ key = "high_lunge", group = "mobility", pattern = "mobility", class = 1, dose = "flow", secs = 30,
		ladder = 12, rung = 2, anchor = true, regress = 245, side = true, pain = { "knee", "hip" }, mp = { BREATH },
		ms = { Qu, Gl, Ha } }),
	[247] = move({ key = "pyramid", group = "mobility", pattern = "mobility", class = 2, dose = "flow", secs = 30,
		ladder = 12, rung = 3, anchor = true, regress = 246, twin = 438, side = true, flags = { "head_down" },
		pain = { "back", "hip" },
		mp = { BREATH }, ms = { Ha, Gl, LB } }),
	-- Beside the pyramid since 2026-10-12 (the ladder gaps), its rung's proof: the couch stretch, the back shin up a bed's
	-- or a sofa's front, which takes the back hip to the split's depth the pyramid never asks (Antranik's splits routine,
	-- the hip flexors' level 3 after the static and the foot-bind lunge).
	-- The pyramid with the hands on the desk and the chest forward (2026-10-16, slice 42): the rung's own pose with no
	-- head-down, so the front split climbs past step 3 and the flying splits' need can be met with head-down positions
	-- off.
	[438] = move({ key = "pyramid_desk", group = "mobility", pattern = "mobility", class = 2, dose = "flow",
		secs = 30, ladder = 12, rung = 3, regress = 246, twin = 246, side = true, pain = { "back", "hip" },
		mp = { BREATH }, ms = { Ha, Gl, LB } }),
	[412] = move({ key = "couch_stretch", group = "mobility", pattern = "mobility", class = 2, dose = "flow", secs = 30,
		ladder = 12, rung = 3, regress = 123, side = true, flags = { "floor" }, support = "bed", pain = { "knee", "hip" },
		mp = { BREATH }, ms = { Qu, Gl } }),
	[248] = move({ key = "split_prep", group = "mobility", pattern = "mobility", class = 2, dose = "flow", secs = 30,
		ladder = 12, rung = 4, anchor = true, regress = 247, twin = 247, side = true, flags = { "floor", "space" },
		pain = { "hip", "knee", "back" }, mp = { BREATH }, ms = { Ha, Qu } }),
	[249] = move({ key = "split_low", group = "mobility", pattern = "mobility", class = 2, dose = "flow", secs = 30,
		ladder = 12, rung = 5, anchor = true, regress = 248, twin = 247, side = true, flags = { "floor", "space" },
		pain = { "hip", "knee", "back" }, mp = { BREATH }, ms = { Ha, Qu } }),
	[250] = move({ key = "front_split", group = "mobility", pattern = "mobility", class = 3, dose = "flow", secs = 30,
		ladder = 12, rung = 6, anchor = true, regress = 249, twin = 247, side = true,
		flags = { "floor", "space", "skill" }, pain = { "hip", "knee", "back" }, mp = { BREATH }, ms = { Ha, Qu } }),
	-- More goal poses on every step (slice 47, docs/plan.md "Slice 47"): standing, lying on a bed, kneeling and seated
	-- forms, press and sink (contract-relax: the ACSM's PNF, Antranik's splits) and lift and hold (Emmet Louis's active
	-- leg raises), each at its step's level.
	[456] = move({ key = "hip_stretch_desk", group = "mobility", pattern = "mobility", class = 1, dose = "flow",
		secs = 30, ladder = 12, rung = 1, side = true, pain = { "hip", "back" }, mp = { BREATH }, ms = { Qu, Gl } }),
	[457] = move({ key = "bed_hip_stretch", group = "mobility", pattern = "mobility", class = 1, dose = "flow", secs = 30,
		ladder = 12, rung = 1, side = true, support = "bed", flags = { "seated" }, pain = { "hip", "knee" }, mp = { BREATH },
		ms = { Qu, Gl } }),
	[458] = move({ key = "quad_stretch_press", group = "mobility", pattern = "mobility", class = 1, dose = "flow",
		secs = 30, ladder = 12, rung = 1, side = true, pain = { "knee" }, mp = { BREATH }, ms = { Qu } }),
	[459] = move({ key = "bed_foot_lunge", group = "mobility", pattern = "mobility", class = 1, dose = "flow",
		secs = 30, ladder = 12, rung = 2, regress = 245, side = true, support = "bed", pain = { "knee", "hip" },
		mp = { BREATH }, ms = { Qu, Gl } }),
	[460] = move({ key = "half_split_books", group = "mobility", pattern = "mobility", class = 1, dose = "flow",
		secs = 30, ladder = 12, rung = 2, regress = 245, side = true, flags = { "floor" },
		pain = { "knee", "hip", "back" }, mp = { BREATH }, ms = { Ha } }),
	[461] = move({ key = "lunge_foot_hold", group = "mobility", pattern = "mobility", class = 1, dose = "flow",
		secs = 30, ladder = 12, rung = 2, regress = 245, side = true, flags = { "floor" }, pain = { "knee", "hip" },
		mp = { BREATH }, ms = { Qu, Gl } }),
	[462] = move({ key = "pyramid_desk_press", group = "mobility", pattern = "mobility", class = 2, dose = "flow",
		secs = 30, ladder = 12, rung = 3, regress = 246, side = true, pain = { "back", "hip" }, mp = { BREATH },
		ms = { Ha, Gl, LB } }),
	[463] = move({ key = "lizard_books", group = "mobility", pattern = "mobility", class = 2, dose = "flow", secs = 30,
		ladder = 12, rung = 3, regress = 246, side = true, flags = { "floor" }, pain = { "hip", "knee" },
		mp = { BREATH }, ms = { Qu, Ad, Gl } }),
	[464] = move({ key = "half_split_lift", group = "mobility", pattern = "mobility", class = 2, dose = "flow",
		secs = 30, ladder = 12, rung = 3, regress = 246, side = true, flags = { "floor" },
		pain = { "knee", "hip", "back" }, mp = { BREATH }, ms = { Ha, Qu } }),
	[465] = move({ key = "lizard_forearms", group = "mobility", pattern = "mobility", class = 2, dose = "flow",
		secs = 30, ladder = 12, rung = 4, regress = 247, side = true, flags = { "floor" }, pain = { "hip", "knee" },
		mp = { BREATH }, ms = { Qu, Ad, Gl } }),
	[466] = move({ key = "couch_arms_up", group = "mobility", pattern = "mobility", class = 2, dose = "flow",
		secs = 30, ladder = 12, rung = 4, regress = 412, side = true, flags = { "floor" }, support = "bed",
		pain = { "knee", "hip", "back" }, mp = { BREATH }, ms = { Qu, Gl } }),
	[467] = move({ key = "split_prep_press", group = "mobility", pattern = "mobility", class = 2, dose = "flow",
		secs = 30, ladder = 12, rung = 4, regress = 247, side = true, flags = { "floor", "space" },
		pain = { "hip", "knee", "back" }, mp = { BREATH }, ms = { Ha, Qu } }),
	[468] = move({ key = "leg_lift_desk", group = "mobility", pattern = "mobility", class = 2, dose = "flow",
		secs = 30, ladder = 12, rung = 4, regress = 247, side = true, pain = { "hip" }, mp = { BREATH },
		ms = { Ha, Qu } }),
	[469] = move({ key = "split_low_tall", group = "mobility", pattern = "mobility", class = 2, dose = "flow",
		secs = 30, ladder = 12, rung = 5, regress = 248, side = true, flags = { "floor", "space" },
		pain = { "hip", "knee", "back" }, mp = { BREATH }, ms = { Ha, Qu } }),
	[470] = move({ key = "split_low_press", group = "mobility", pattern = "mobility", class = 2, dose = "flow",
		secs = 30, ladder = 12, rung = 5, regress = 248, side = true, flags = { "floor", "space" },
		pain = { "hip", "knee", "back" }, mp = { BREATH }, ms = { Ha, Qu } }),
	[471] = move({ key = "bed_leg_pull", group = "mobility", pattern = "mobility", class = 2, dose = "flow",
		secs = 30, ladder = 12, rung = 5, regress = 248, side = true, support = "bed", flags = { "seated" }, pain = { "hip" },
		mp = { BREATH }, ms = { Ha } }),
	[472] = move({ key = "split_arms_up", group = "mobility", pattern = "mobility", class = 3, dose = "flow",
		secs = 30, ladder = 12, rung = 6, regress = 250, side = true, flags = { "floor", "space", "skill" },
		pain = { "hip", "knee", "back" }, mp = { BREATH }, ms = { Ha, Qu } }),
	[473] = move({ key = "split_fold", group = "mobility", pattern = "mobility", class = 3, dose = "flow", secs = 30,
		ladder = 12, rung = 6, regress = 250, side = true, flags = { "floor", "space", "skill" },
		pain = { "hip", "knee", "back" }, mp = { BREATH }, ms = { Ha, Qu } }),
	[474] = move({ key = "leg_lift_hip", group = "mobility", pattern = "mobility", class = 3, dose = "flow",
		secs = 30, ladder = 12, rung = 6, regress = 468, side = true, pain = { "hip" }, mp = { BREATH },
		ms = { Ha, Qu } }),
	-- The pancake and the middle split, with the hips' turn-out poses (Spirit)
	-- Slice 47 (2026-10-19): four goal poses or more on every step, each changing the method (a static hold, press and
	-- sink, lift and hold), the body's position or a joint's angle. Step 1 is the butterfly on the bed (Baddha Konasana's
	-- starter, Light on Yoga 3), the inner thigh the side lunge needs, since the research of that day (the figure four,
	-- its anchor until then, trains the outer hip), beside it lying on the bed, the wide-leg chair stretch (on a stable
	-- chair since 2026-10-20: leaning forward on straight legs pushes a chair that rolls or tips away) and the horse
	-- stance held halfway (Antranik's horse stance), the inner thigh's range held by the legs' own muscles.
	[476] = move({ key = "butterfly", group = "mobility", pattern = "mobility", class = 1, dose = "flow", secs = 30,
		ladder = 13, rung = 1, anchor = true, support = "bed", flags = { "seated" }, pain = { "hip", "knee" },
		mp = { BREATH }, ms = { Ad, Gl } }),
	[477] = move({ key = "lying_butterfly", group = "mobility", pattern = "mobility", class = 1, dose = "flow", secs = 30,
		ladder = 13, rung = 1, support = "bed", flags = { "seated" }, pain = { "hip", "knee" }, mp = { BREATH },
		ms = { Ad, Gl } }),
	[478] = move({ key = "wide_chair_stretch", group = "mobility", pattern = "mobility", class = 1, dose = "flow",
		secs = 30, ladder = 13, rung = 1, needs = { "stable_chair" }, support = "chair", flags = { "seated" },
		pain = { "back", "hip" }, mp = { BREATH }, ms = { Ad, Ha, LB } }),
	[479] = move({ key = "horse_stance", group = "mobility", pattern = "mobility", class = 1, dose = "flow", secs = 30,
		ladder = 13, rung = 1, pain = { "knee", "hip" }, mp = { Qu, Ad }, ms = { Gl } }),
	[251] = move({ key = "side_lunge", group = "mobility", pattern = "mobility", class = 1, dose = "flow", secs = 30,
		ladder = 13, rung = 2, anchor = true, regress = 476, side = true, pain = { "knee", "hip" }, mp = { BREATH },
		ms = { Ad, Gl, Qu } }),
	-- Step 3's (slice 47): the butterfly folded forward, the knees a fist from the bed and the heels a hand in (on the
	-- way to Baddha Konasana in full, the knees down and the heels at the body, until 2026-10-27), the side lunge pressed
	-- and sunk (contract-relax: about half effort for 5 s, relax, ease deeper, three rounds; Garber 2011), the wide-knee
	-- child's pose kneeling and the legs opened up the wall lying down, two thirds of the way to the floor, gravity's
	-- stretch.
	[480] = move({ key = "butterfly_deep", group = "mobility", pattern = "mobility", class = 1, dose = "flow", secs = 30,
		ladder = 13, rung = 2, regress = 476, support = "bed", flags = { "seated" }, pain = { "hip", "knee" },
		mp = { BREATH }, ms = { Ad, Gl } }),
	[481] = move({ key = "side_lunge_press", group = "mobility", pattern = "mobility", class = 1, dose = "flow",
		secs = 30, ladder = 13, rung = 2, regress = 476, side = true, pain = { "knee", "hip" }, mp = { BREATH },
		ms = { Ad, Gl, Qu } }),
	[482] = move({ key = "wide_childs_pose", group = "mobility", pattern = "mobility", class = 1, dose = "flow",
		secs = 30, ladder = 13, rung = 2, regress = 476, flags = { "floor", "head_down" }, pain = { "knee", "hip" },
		mp = { BREATH }, ms = { Ad, LB, Gl } }),
	[483] = move({ key = "wall_straddle", group = "mobility", pattern = "mobility", class = 1, dose = "flow",
		secs = 30, ladder = 13, rung = 2, regress = 476, flags = { "floor" }, pain = { "hip" }, mp = { BREATH },
		ms = { Ad, Ha } }),
	[252] = move({ key = "prasarita", group = "mobility", pattern = "mobility", class = 2, dose = "flow", secs = 30,
		ladder = 13, rung = 3, anchor = true, regress = 251, twin = 439, flags = { "head_down" }, pain = { "back", "hip" },
		mp = { BREATH }, ms = { Ad, Ha, LB } }),
	-- Beside the wide-leg fold since 2026-10-16 (slice 42): the same stretch with the hands on the desk and the chest
	-- level (the rung's own pose with no head-down, so the middle split climbs past step 3 with head-down positions off),
	-- and the 90-90 sit, the hips' turn in and out, the check the dragon pistols wait for.
	[439] = move({ key = "straddle_desk", group = "mobility", pattern = "mobility", class = 2, dose = "flow",
		secs = 30, ladder = 13, rung = 3, regress = 251, twin = 251, pain = { "back", "hip" }, mp = { BREATH },
		ms = { Ad, Ha, LB } }),
	[440] = move({ key = "ninety_ninety", group = "mobility", pattern = "mobility", class = 2, dose = "flow", secs = 30,
		ladder = 13, rung = 3, regress = 125, side = true, support = "bed", flags = { "seated" }, pain = { "hip", "knee" },
		mp = { Gl }, ms = { Ad, Qu } }),
	-- Step 5's (slice 47): the frog on the forearms, kneeling; the wide sit on the bed, upright Upavistha Konasana's
	-- legs a right angle apart, which the straddle fold then folds over; the horse stance with the thighs level.
	[484] = move({ key = "frog_pose", group = "mobility", pattern = "mobility", class = 2, dose = "flow", secs = 30,
		ladder = 13, rung = 3, regress = 482, flags = { "floor" }, pain = { "knee", "hip" }, mp = { BREATH },
		ms = { Ad, Gl } }),
	[485] = move({ key = "wide_sit_bed", group = "mobility", pattern = "mobility", class = 2, dose = "flow", secs = 30,
		ladder = 13, rung = 3, regress = 483, support = "bed", flags = { "seated" }, pain = { "hip", "knee" },
		mp = { BREATH }, ms = { Ad, Ha } }),
	[486] = move({ key = "horse_stance_deep", group = "mobility", pattern = "mobility", class = 2, dose = "flow",
		secs = 30, ladder = 13, rung = 3, regress = 479, pain = { "knee", "hip" }, mp = { Qu, Ad }, ms = { Gl } }),
	[253] = move({ key = "straddle_fold", group = "mobility", pattern = "mobility", class = 2, dose = "flow", secs = 30,
		ladder = 13, rung = 4, anchor = true, regress = 252, twin = 252, flags = { "floor", "space" },
		pain = { "hip", "back", "knee" }, mp = { BREATH }, ms = { Ad, Ha } }),
	-- Step 7's (slice 47): the straddle fold pressed and sunk; the straddle's heels lifted and held off the floor by the
	-- legs (an active end-range lift, Functional Range Conditioning's lift-offs); the standing straddle facing the bed,
	-- the hands on it and the seat a forearm from the floor (down to the bed's height until 2026-10-27, which beds vary),
	-- the legs about 130 degrees apart, past the fold's right angle; and the 90-90 with its back shin lifted, the hip's
	-- turn in held by its own muscles.
	[487] = move({ key = "straddle_press", group = "mobility", pattern = "mobility", class = 2, dose = "flow",
		secs = 30, ladder = 13, rung = 4, regress = 252, flags = { "floor", "space" }, pain = { "hip", "back", "knee" },
		mp = { BREATH }, ms = { Ad, Ha } }),
	[488] = move({ key = "straddle_heel_lift", group = "mobility", pattern = "mobility", class = 2, dose = "flow",
		secs = 30, ladder = 13, rung = 4, regress = 485, flags = { "floor", "space" }, pain = { "hip", "back", "knee" },
		mp = { Qu, Ab }, ms = { Ad, Ha } }),
	[489] = move({ key = "standing_straddle_bed", group = "mobility", pattern = "mobility", class = 2, dose = "flow",
		secs = 30, ladder = 13, rung = 4, regress = 439, support = "bed", pain = { "hip", "knee", "back" },
		mp = { BREATH }, ms = { Ad, Ha } }),
	[490] = move({ key = "ninety_ninety_lift", group = "mobility", pattern = "mobility", class = 2, dose = "flow",
		secs = 30, ladder = 13, rung = 4, regress = 440, side = true, support = "bed", flags = { "seated" },
		pain = { "hip", "knee" }, mp = { Gl }, ms = { Ad } }),
	-- The pancake below the middle split since 2026-10-10 (the ladder rethink): the full middle split, the hips on the
	-- floor or a folded towel, is the family's top, the pancake its step 9 (rung 6 and 5 until then).
	[255] = move({ key = "pancake", group = "mobility", pattern = "mobility", class = 3, dose = "flow", secs = 30,
		ladder = 13, rung = 5, anchor = true, regress = 253, twin = 252, flags = { "floor", "space", "skill" },
		pain = { "hip", "back", "knee" }, mp = { BREATH }, ms = { Ad, Ha, LB } }),
	-- Beside the pancake since 2026-10-12 (the ladder gaps), its rung's proof: the side split with the hips a hand up
	-- (30 cm until 2026-10-27), the hips straight and the slide on the hands, which the seated pancake never trains (as
	-- the front split's prep).
	[413] = move({ key = "middle_split_prep", group = "mobility", pattern = "mobility", class = 2, dose = "flow",
		secs = 30, ladder = 13, rung = 5, regress = 253, flags = { "floor", "space" }, pain = { "hip", "knee" },
		mp = { BREATH }, ms = { Ad, Ha } }),
	-- Step 9's (slice 47): the pancake pressed and sunk; the side split pressed and sunk at the bed, the hands on it and
	-- the hips a hand from the floor (Kurz's isometric side split: the feet pressed down and in, then eased a little
	-- wider with the weight in the hands, barefoot on a floor that does not slip; socks until 2026-10-20, which slip at
	-- that depth); the frog with the chest and belly down.
	[491] = move({ key = "pancake_press", group = "mobility", pattern = "mobility", class = 3, dose = "flow",
		secs = 30, ladder = 13, rung = 5, regress = 253, flags = { "floor", "space", "skill" },
		pain = { "hip", "back", "knee" }, mp = { BREATH }, ms = { Ad, Ha, LB } }),
	[492] = move({ key = "side_split_press_bed", group = "mobility", pattern = "mobility", class = 2, dose = "flow",
		secs = 30, ladder = 13, rung = 5, regress = 489, flags = { "space" }, support = "bed", pain = { "hip", "knee" },
		mp = { BREATH }, ms = { Ad, Ha } }),
	[493] = move({ key = "frog_chest", group = "mobility", pattern = "mobility", class = 2, dose = "flow", secs = 30,
		ladder = 13, rung = 5, regress = 484, flags = { "floor" }, pain = { "knee", "hip" }, mp = { BREATH },
		ms = { Ad, Gl } }),
	[254] = move({ key = "middle_split", group = "mobility", pattern = "mobility", class = 3, dose = "flow", secs = 30,
		ladder = 13, rung = 6, anchor = true, regress = 413, twin = 252, flags = { "floor", "space", "skill" },
		pain = { "hip", "knee" }, mp = { BREATH }, ms = { Ad, Ha } }),
	-- Step 11's (slice 47): the middle split folded onto the forearms, the pelvis tipped forward, and the middle split
	-- pressed and sunk. No pose of the split's depth stands without the floor (the hips on it or a towel).
	[494] = move({ key = "middle_split_fold", group = "mobility", pattern = "mobility", class = 3, dose = "flow",
		secs = 30, ladder = 13, rung = 6, regress = 413, flags = { "floor", "space", "skill" },
		pain = { "hip", "knee", "back" }, mp = { BREATH }, ms = { Ad, Ha, LB } }),
	[495] = move({ key = "middle_split_press", group = "mobility", pattern = "mobility", class = 3, dose = "flow",
		secs = 30, ladder = 13, rung = 6, regress = 413, flags = { "floor", "space", "skill" }, pain = { "hip", "knee" },
		mp = { BREATH }, ms = { Ad, Ha } }),
	-- The hamstrings and the pike (Spirit)
	[256] = move({ key = "chair_hamstring", group = "mobility", pattern = "mobility", class = 1, dose = "flow", secs = 30,
		ladder = 14, rung = 1, anchor = true, side = true, flags = { "seated" }, pain = { "back", "hip" }, mp = { BREATH },
		ms = { Ha, Ca } }),
	[257] = move({ key = "toe_hold_fold", group = "mobility", pattern = "mobility", class = 1, dose = "flow", secs = 30,
		ladder = 14, rung = 3, anchor = true, regress = 126, twin = 434, flags = { "head_down" }, pain = { "back", "hip" },
		mp = { BREATH }, ms = { Ha, LB, Ca } }),
	[258] = move({ key = "seated_fold", group = "mobility", pattern = "mobility", class = 2, dose = "flow", secs = 30,
		ladder = 14, rung = 4, anchor = true, regress = 257, twin = 435, flags = { "floor" }, pain = { "back", "hip" },
		mp = { BREATH }, ms = { Ha, LB, Ca } }),
	[259] = move({ key = "forehead_to_knees", group = "mobility", pattern = "mobility", class = 2, dose = "flow",
		secs = 30, ladder = 14, rung = 5, anchor = true, regress = 257, twin = 436, flags = { "head_down" },
		pain = { "back", "hip" }, mp = { BREATH }, ms = { Ha, LB, Ca } }),
	[260] = move({ key = "pike_fold", group = "mobility", pattern = "mobility", class = 3, dose = "flow", secs = 30,
		ladder = 14, rung = 6, anchor = true, regress = 258, twin = 437, flags = { "floor" }, pain = { "back", "hip" },
		mp = { BREATH }, ms = { Ha, LB, Ca } }),
	-- The pike's own-level twins with no head-down and no floor (2026-10-16, slice 42): the chair toe reach, the staff
	-- pose, the seated folds and the pike fold on a bed, so a setup that avoids head-down positions or has no floor
	-- climbs the pike to its top and meets the checks that name its steps (the L-sit's, the handstand's); each
	-- falls back to the one below, as the folds fell back to the chair stretch until then.
	[433] = move({ key = "chair_toe_reach", group = "mobility", pattern = "mobility", class = 1, dose = "flow", secs = 30,
		ladder = 14, rung = 2, regress = 256, twin = 256, side = true, flags = { "seated" }, pain = { "back", "hip" },
		mp = { BREATH }, ms = { Ha, Ca } }),
	[434] = move({ key = "staff_pose_bed", group = "mobility", pattern = "mobility", class = 1, dose = "flow", secs = 30,
		ladder = 14, rung = 3, regress = 433, twin = 433, support = "bed", flags = { "seated" }, pain = { "back", "hip" },
		mp = { BREATH }, ms = { Ha, LB, Ab } }),
	[435] = move({ key = "seated_fold_bed", group = "mobility", pattern = "mobility", class = 2, dose = "flow", secs = 30,
		ladder = 14, rung = 4, regress = 434, twin = 434, support = "bed", flags = { "seated" }, pain = { "back", "hip" },
		mp = { BREATH }, ms = { Ha, LB, Ca } }),
	[436] = move({ key = "chest_to_knees_bed", group = "mobility", pattern = "mobility", class = 2, dose = "flow",
		secs = 30, ladder = 14, rung = 5, regress = 435, twin = 435, support = "bed", flags = { "seated" },
		pain = { "back", "hip" }, mp = { BREATH }, ms = { Ha, LB, Ca } }),
	[437] = move({ key = "pike_fold_bed", group = "mobility", pattern = "mobility", class = 3, dose = "flow", secs = 30,
		ladder = 14, rung = 6, regress = 436, twin = 436, support = "bed", flags = { "seated" }, pain = { "back", "hip" },
		mp = { BREATH }, ms = { Ha, LB, Ca } }),
	-- Each step's other goal poses (slice 47, 2026-10-19): the hamstrings lying on the bed (a strap, then the hands on
	-- the shin), through a doorway on the floor, the heel on the bed's edge, one leg at a time sitting on the bed, the
	-- desk's half fold, the leg lifted and held by its own muscles (lying, then sitting tall), the standing reach to the
	-- heels and the hold of the calves, and the press and sink (half effort for 5 s, relax, breathe out, ease deeper) on
	-- the chair and the bed folds; the lying, the doorway and the one-leg poses come at a lower angle than the folds of
	-- their step, since the back and the other leg take no part. None is head-down but the two standing folds, which the
	-- bed's poses of their step stand beside.
	[496] = move({ key = "lying_hamstring", group = "mobility", pattern = "mobility", class = 1, dose = "flow", secs = 30,
		ladder = 14, rung = 1, side = true, support = "bed", flags = { "seated" }, pain = { "hip" }, mp = { BREATH },
		ms = { Ha, Ca } }),
	[497] = move({ key = "doorway_hamstring", group = "mobility", pattern = "mobility", class = 1, dose = "flow",
		secs = 30, ladder = 14, rung = 1, side = true, flags = { "floor" }, pain = { "hip" }, mp = { BREATH },
		ms = { Ha, Ca } }),
	[498] = move({ key = "chair_hamstring_press", group = "mobility", pattern = "mobility", class = 1, dose = "flow",
		secs = 30, ladder = 14, rung = 1, regress = 256, side = true, flags = { "seated" }, pain = { "back", "hip" },
		mp = { BREATH }, ms = { Ha, Ca } }),
	[499] = move({ key = "lying_leg_lift", group = "mobility", pattern = "mobility", class = 1, dose = "flow",
		secs = 30, ladder = 14, rung = 2, regress = 496, side = true, support = "bed", flags = { "seated" }, pain = { "hip" },
		mp = { BREATH }, ms = { Ha, Qu } }),
	[500] = move({ key = "bed_heel_hamstring", group = "mobility", pattern = "mobility", class = 1, dose = "flow",
		secs = 30, ladder = 14, rung = 2, regress = 256, side = true, support = "bed", pain = { "back", "hip" },
		mp = { BREATH }, ms = { Ha, Ca } }),
	[501] = move({ key = "head_knee_reach", group = "mobility", pattern = "mobility", class = 1, dose = "flow", secs = 30,
		ladder = 14, rung = 2, regress = 256, side = true, support = "bed", flags = { "seated" },
		pain = { "back", "hip", "knee" }, mp = { BREATH }, ms = { Ha, LB, Ca } }),
	[502] = move({ key = "desk_half_fold", group = "mobility", pattern = "mobility", class = 1, dose = "flow",
		secs = 30, ladder = 14, rung = 3, regress = 433, pain = { "back", "hip" }, mp = { BREATH },
		ms = { Ha, LB, Ca } }),
	[503] = move({ key = "doorway_hamstring_wall", group = "mobility", pattern = "mobility", class = 1, dose = "flow",
		secs = 30, ladder = 14, rung = 3, regress = 497, side = true, flags = { "floor" }, pain = { "hip" },
		mp = { BREATH }, ms = { Ha, Ca } }),
	[504] = move({ key = "heel_reach_fold", group = "mobility", pattern = "mobility", class = 2, dose = "flow",
		secs = 30, ladder = 14, rung = 4, regress = 257, flags = { "head_down" }, pain = { "back", "hip" },
		mp = { BREATH }, ms = { Ha, LB, Ca } }),
	[505] = move({ key = "head_knee_hold", group = "mobility", pattern = "mobility", class = 2, dose = "flow", secs = 30,
		ladder = 14, rung = 4, regress = 501, side = true, support = "bed", flags = { "seated" },
		pain = { "back", "hip", "knee" }, mp = { BREATH }, ms = { Ha, LB, Ca } }),
	[506] = move({ key = "lying_hamstring_deep", group = "mobility", pattern = "mobility", class = 2, dose = "flow",
		secs = 30, ladder = 14, rung = 4, regress = 496, side = true, support = "bed", flags = { "seated" }, pain = { "hip" },
		mp = { BREATH }, ms = { Ha, Ca } }),
	[507] = move({ key = "chair_leg_lift", group = "mobility", pattern = "mobility", class = 2, dose = "flow",
		secs = 30, ladder = 14, rung = 4, regress = 499, side = true, flags = { "seated" }, pain = { "back", "hip" },
		mp = { BREATH }, ms = { Ha, Qu } }),
	[508] = move({ key = "seated_fold_bed_press", group = "mobility", pattern = "mobility", class = 2, dose = "flow",
		secs = 30, ladder = 14, rung = 4, regress = 435, support = "bed", flags = { "seated" }, pain = { "back", "hip" },
		mp = { BREATH }, ms = { Ha, LB, Ca } }),
	[509] = move({ key = "head_knee_chest", group = "mobility", pattern = "mobility", class = 2, dose = "flow", secs = 30,
		ladder = 14, rung = 5, regress = 505, side = true, support = "bed", flags = { "seated" },
		pain = { "back", "hip", "knee" }, mp = { BREATH }, ms = { Ha, LB, Ca } }),
	[510] = move({ key = "lying_toes_to_chin", group = "mobility", pattern = "mobility", class = 2, dose = "flow",
		secs = 30, ladder = 14, rung = 5, regress = 506, side = true, support = "bed", flags = { "seated" },
		pain = { "back", "hip" }, mp = { BREATH }, ms = { Ha, Ca } }),
	[511] = move({ key = "chest_to_knees_press", group = "mobility", pattern = "mobility", class = 2, dose = "flow",
		secs = 30, ladder = 14, rung = 5, regress = 436, support = "bed", flags = { "seated" }, pain = { "back", "hip" },
		mp = { BREATH }, ms = { Ha, LB, Ca } }),
	[512] = move({ key = "head_knee_face", group = "mobility", pattern = "mobility", class = 3, dose = "flow", secs = 30,
		ladder = 14, rung = 6, regress = 509, side = true, support = "bed", flags = { "seated" },
		pain = { "back", "hip", "knee" }, mp = { BREATH }, ms = { Ha, LB, Ca } }),
	[513] = move({ key = "face_to_shins", group = "mobility", pattern = "mobility", class = 3, dose = "flow",
		secs = 30, ladder = 14, rung = 6, regress = 259, flags = { "head_down" }, pain = { "back", "hip" },
		mp = { BREATH }, ms = { Ha, LB, Ca } }),
	[514] = move({ key = "lying_leg_to_face", group = "mobility", pattern = "mobility", class = 3, dose = "flow",
		secs = 30, ladder = 14, rung = 6, regress = 510, side = true, support = "bed", flags = { "seated" },
		pain = { "back", "hip" }, mp = { BREATH }, ms = { Ha, Ca } }),
	[515] = move({ key = "pike_fold_bed_press", group = "mobility", pattern = "mobility", class = 3, dose = "flow",
		secs = 30, ladder = 14, rung = 6, regress = 437, support = "bed", flags = { "seated" }, pain = { "back", "hip" },
		mp = { BREATH }, ms = { Ha, LB, Ca } }),
	-- The backbend: wall angels to the wheel to stand-to-stand (Spirit); the wheel and above ask for a clear 2 x 2 m floor
	-- and a wall, never beside the desk. The wheel with the hands on a bed's or a sofa's edge stands for rung 3 since
	-- 2026-10-10 (the ladder rethink; Hybrid Calisthenics' incline bridges before full bridges): it presses to straight
	-- arms with the head free, which the camel does not train and the wheel's entry otherwise rests on the crown of the
	-- head for; the camel stays on rung 3 beside it.
	[261] = move({ key = "standing_backbend", group = "yoga", pattern = "mobility", class = 1, dose = "flow", secs = 30,
		ladder = 15, rung = 2, anchor = true, regress = 62, pain = { "back", "neck" }, mp = { BREATH },
		ms = { LB, Ch, Qu } }),
	[262] = move({ key = "camel", group = "yoga", pattern = "mobility", class = 2, dose = "flow", secs = 30, ladder = 15,
		rung = 3, regress = 261, twin = 441, flags = { "floor" }, pain = { "back", "knee", "neck" }, mp = { BREATH },
		ms = { Qu, Ch, LB } }),
	-- The backbend's rung 3 standing at the wall (2026-10-16, slice 42): the hands walk down the wall to head height,
	-- the rung's own pose with no floor, so the dancer's need can be met with Floor moves off.
	[441] = move({ key = "wall_lean_back", group = "yoga", pattern = "mobility", class = 2, dose = "flow", secs = 30,
		ladder = 15, rung = 3, regress = 261, twin = 261, pain = { "back", "neck", "shoulder" }, mp = { BREATH },
		ms = { LB, Ch, Sh } }),
	[374] = move({ key = "bed_wheel", group = "yoga", pattern = "mobility", class = 2, dose = "flow", secs = 30,
		ladder = 15, rung = 3, anchor = true, regress = 262, twin = 262, support = "bed",
		flags = { "floor", "space", "head_down", "strain", "skill" }, pain = { "wrist", "shoulder", "back" },
		mp = { BREATH }, ms = { Sh, Ch, LB, Qu } }),
	-- The backbend's in-between moves since 2026-10-12 (the ladder gaps), each its rung's proof: the wheel with the hands
	-- on books against a wall beside the bed wheel (Ekhart Yoga's blocks on the baseboard), the wall walk to hip height
	-- beside the wheel (Hybrid Calisthenics' easier wall walk) and the drop back to a bed's edge beside the wall walk (its
	-- stand-to-stand regression, "a low bed or couch, provided it is safe and secure").
	[414] = move({ key = "wall_books_wheel", group = "yoga", pattern = "mobility", class = 2, dose = "flow", secs = 30,
		ladder = 15, rung = 3, regress = 374, flags = { "floor", "space", "head_down", "strain", "skill" },
		pain = { "wrist", "shoulder", "back" }, mp = { BREATH }, ms = { Sh, Ch, LB, Qu } }),
	-- The wheel's mark asks five presses, the last held 20 s, since 2026-10-12 (three, the last 10 s, in 40 s until then):
	-- Hybrid Calisthenics asks full and wheel bridges 2 x 15 before wall walking.
	[263] = move({ key = "wheel", group = "yoga", pattern = "mobility", class = 3, dose = "flow", secs = 50, ladder = 15,
		rung = 4, anchor = true, regress = 414, twin = 261, flags = { "floor", "space", "head_down", "strain", "skill" },
		pain = { "wrist", "shoulder", "back" }, mp = { BREATH }, ms = { Sh, Ch, LB, Qu } }),
	[415] = move({ key = "wall_walk_half", group = "yoga", pattern = "mobility", class = 3, dose = "flow", secs = 60,
		ladder = 15, rung = 4, regress = 263, flags = { "space", "head_down", "strain", "skill" },
		pain = { "back", "shoulder", "wrist" }, mp = { BREATH }, ms = { Sh, Ch, LB, Qu } }),
	[264] = move({ key = "wall_walk", group = "yoga", pattern = "mobility", class = 3, dose = "flow", secs = 60,
		ladder = 15, rung = 5, anchor = true, regress = 415, twin = 261,
		flags = { "floor", "space", "head_down", "strain", "skill" }, pain = { "back", "shoulder", "wrist" },
		mp = { BREATH }, ms = { Sh, Ch, LB, Qu } }),
	[416] = move({ key = "bed_drop_back", group = "yoga", pattern = "mobility", class = 3, dose = "flow", secs = 40,
		ladder = 15, rung = 5, regress = 264, flags = { "space", "head_down", "strain", "skill" }, support = "bed",
		pain = { "back", "wrist", "shoulder" }, mp = { BREATH }, ms = { Sh, Ch, LB, Qu } }),
	[265] = move({ key = "stand_to_stand", group = "yoga", pattern = "mobility", class = 3, dose = "flow", secs = 40,
		ladder = 15, rung = 6, anchor = true, regress = 416, twin = 261,
		flags = { "floor", "space", "head_down", "strain", "skill" }, pain = { "back", "wrist", "shoulder" },
		mp = { BREATH }, ms = { Sh, Ch, LB, Qu } }),
	-- The backbend's goal poses beside each step's main pose (slice 47, 2026-10-19): static holds, press and sink at
	-- the desk, and lift and hold face down; from step 6 the wheel's shoulder lean, foot lift, one leg, walk-in and
	-- heels (Light on Yoga's Eka Pada Urdhva Dhanurasana; flashmavi's and Alo Moves' wheel drills), and the standing
	-- and kneeling hang-backs of the drop back, never landing.
	[516] = move({ key = "sphinx", group = "yoga", pattern = "mobility", class = 1, dose = "flow", secs = 40, ladder = 15,
		rung = 1, flags = { "floor" }, pain = { "back" }, mp = { LB }, ms = { Ch, Sh, BREATH } }),
	[517] = move({ key = "towel_reach", group = "mobility", pattern = "mobility", class = 1, dose = "flow", secs = 40,
		ladder = 15, rung = 1, flags = { "floor" }, pain = { "shoulder", "back" }, mp = { UB },
		ms = { Sh, Ch, La, BREATH } }),
	[518] = move({ key = "desk_chest_sink", group = "mobility", pattern = "mobility", class = 1, dose = "flow",
		secs = 30, ladder = 15, rung = 1, pain = { "shoulder" }, mp = { UB }, ms = { Sh, La, Ch, BREATH } }),
	[519] = move({ key = "locust", group = "yoga", pattern = "mobility", class = 1, dose = "flow", secs = 30, ladder = 15,
		rung = 2, flags = { "floor" }, pain = { "back", "neck" }, mp = { LB }, ms = { Gl, Ha, UB } }),
	[520] = move({ key = "upward_plank", group = "yoga", pattern = "mobility", class = 2, dose = "flow", secs = 30,
		ladder = 15, rung = 2, flags = { "floor" }, pain = { "wrist", "shoulder" }, mp = { Sh },
		ms = { Ch, Gl, Ha, Tr } }),
	[521] = move({ key = "desk_updog", group = "yoga", pattern = "mobility", class = 1, dose = "flow", secs = 30,
		ladder = 15, rung = 2, pain = { "wrist", "back" }, mp = { LB }, ms = { Ch, Sh, Ab } }),
	[522] = move({ key = "bow", group = "yoga", pattern = "mobility", class = 2, dose = "flow", secs = 30, ladder = 15,
		rung = 3, flags = { "floor" }, pain = { "back", "knee", "shoulder" }, mp = { LB }, ms = { Qu, Ch, Sh } }),
	[523] = move({ key = "locust_hands_head", group = "yoga", pattern = "mobility", class = 2, dose = "flow",
		secs = 30, ladder = 15, rung = 3, flags = { "floor" }, pain = { "back", "neck", "shoulder" }, mp = { LB },
		ms = { UB, Gl, Ha } }),
	[524] = move({ key = "standing_knee_backbend", group = "yoga", pattern = "mobility", class = 2, dose = "flow",
		secs = 30, ladder = 15, rung = 3, pain = { "back", "neck" }, mp = { BREATH }, ms = { LB, Ch, Qu } }),
	[525] = move({ key = "wheel_lean", group = "yoga", pattern = "mobility", class = 3, dose = "flow", secs = 40,
		ladder = 15, rung = 4, regress = 263, flags = { "floor", "space", "head_down", "strain", "skill" },
		pain = { "wrist", "shoulder", "back" }, mp = { BREATH }, ms = { Sh, Ch, LB, Qu } }),
	[526] = move({ key = "wheel_foot_lift", group = "yoga", pattern = "mobility", class = 3, dose = "flow", secs = 40,
		ladder = 15, rung = 4, regress = 263, flags = { "floor", "space", "head_down", "strain", "skill" },
		pain = { "wrist", "shoulder", "back" }, mp = { BREATH }, ms = { Sh, Ch, LB, Gl } }),
	[527] = move({ key = "standing_calf_backbend", group = "yoga", pattern = "mobility", class = 3, dose = "flow",
		secs = 30, ladder = 15, rung = 4, regress = 524, flags = { "space", "head_down", "strain", "skill" },
		pain = { "back", "neck", "knee" }, mp = { BREATH }, ms = { LB, Ch, Qu } }),
	[528] = move({ key = "one_leg_wheel", group = "yoga", pattern = "mobility", class = 3, dose = "flow", secs = 30,
		ladder = 15, rung = 5, regress = 526, side = true, flags = { "floor", "space", "head_down", "strain", "skill" },
		pain = { "wrist", "shoulder", "back" }, mp = { BREATH }, ms = { Sh, Ch, LB, Gl } }),
	[529] = move({ key = "kneeling_reach_back", group = "yoga", pattern = "mobility", class = 3, dose = "flow",
		secs = 30, ladder = 15, rung = 5, regress = 262, flags = { "floor", "space", "head_down", "strain", "skill" },
		pain = { "back", "knee", "neck", "shoulder" }, mp = { BREATH }, ms = { Qu, Ch, LB, Sh } }),
	[530] = move({ key = "wheel_walk_in", group = "yoga", pattern = "mobility", class = 3, dose = "flow", secs = 50,
		ladder = 15, rung = 5, regress = 525, flags = { "floor", "space", "head_down", "strain", "skill" },
		pain = { "wrist", "shoulder", "back" }, mp = { BREATH }, ms = { Sh, Ch, LB, Qu } }),
	[531] = move({ key = "wheel_heels", group = "yoga", pattern = "mobility", class = 3, dose = "flow", secs = 50,
		ladder = 15, rung = 6, regress = 530, flags = { "floor", "space", "head_down", "strain", "skill" },
		pain = { "wrist", "shoulder", "back" }, mp = { BREATH }, ms = { Sh, Ch, LB, Qu } }),
	[532] = move({ key = "standing_hang_back", group = "yoga", pattern = "mobility", class = 3, dose = "flow",
		secs = 30, ladder = 15, rung = 6, regress = 529, flags = { "space", "head_down", "strain", "skill" },
		pain = { "back", "knee", "neck" }, mp = { BREATH }, ms = { LB, Ch, Qu, Ab } }),
	-- Twists (Spirit)
	[266] = move({ key = "chair_twist", group = "mobility", pattern = "mobility", class = 1, dose = "flow", secs = 30,
		ladder = 16, rung = 1, anchor = true, side = true, flags = { "seated" }, pain = { "back", "neck" }, mp = { BREATH },
		ms = { Ob, LB } }),
	-- Beside the chair twist since slice 47 (2026-10-19), each a chest turned halfway to the side over still hips in
	-- another position or method: standing side-on to a wall with both palms on it (Yogavastu's wall-supported
	-- standing twist), lying on the side with the top knee held down (Physitrack's side-lying thoracic rotation, held
	-- 30 s), sitting cross-legged on the floor (Parivrtta Sukhasana), and the chair twist pressed and sunk
	-- (contract-relax, the largest gains in range: Konrad 2024).
	[536] = move({ key = "wall_twist", group = "mobility", pattern = "mobility", class = 1, dose = "flow", secs = 30,
		ladder = 16, rung = 1, side = true, pain = { "back", "neck" }, mp = { BREATH }, ms = { Ob, LB, Ch } }),
	[537] = move({ key = "side_lying_twist", group = "mobility", pattern = "mobility", class = 1, dose = "flow", secs = 30,
		ladder = 16, rung = 1, side = true, flags = { "floor" }, pain = { "shoulder" }, mp = { BREATH }, ms = { UB, Ch } }),
	[538] = move({ key = "easy_twist", group = "mobility", pattern = "mobility", class = 1, dose = "flow", secs = 30,
		ladder = 16, rung = 1, side = true, flags = { "floor" }, pain = { "back", "hip" }, mp = { BREATH },
		ms = { Ob, LB, Gl } }),
	[539] = move({ key = "chair_twist_press", group = "mobility", pattern = "mobility", class = 1, dose = "flow",
		secs = 30, ladder = 16, rung = 1, side = true, flags = { "seated" }, pain = { "back", "neck" }, mp = { BREATH },
		ms = { Ob, LB } }),
	[267] = move({ key = "revolved_chair", group = "mobility", pattern = "mobility", class = 1, dose = "flow", secs = 30,
		ladder = 16, rung = 2, anchor = true, regress = 266, side = true, pain = { "back", "knee" }, mp = { BREATH },
		ms = { Ob, Qu, Gl } }),
	-- The twists' in-between moves since 2026-10-12 (the ladder gaps), each its rung's proof: the twisted triangle with
	-- the palm on books beside the twisted chair (the straight legs and the narrow stance at a raised floor; YogaUOnline,
	-- Yoga Matters), and the lunge twist with the palms together beside the bound twist (Yoga International: Parivrtta
	-- Parsvakonasana before its clasped form, the bound lunge twist).
	[417] = move({ key = "twisted_triangle_books", group = "mobility", pattern = "mobility", class = 2, dose = "flow",
		secs = 30, ladder = 16, rung = 2, regress = 267, side = true, pain = { "back", "hip", "neck" }, mp = { BREATH },
		ms = { Ob, Ha, LB } }),
	-- Beside the twisted chair since slice 47 (2026-10-19): standing on one straight leg with the other foot up on the
	-- bed (Yogavastu's Utthita Marichyasana), on hands and knees with a hand behind the head (Yoga International's
	-- kneeling chest turn), and the seated twist pressed and sunk.
	[540] = move({ key = "foot_up_twist", group = "mobility", pattern = "mobility", class = 1, dose = "flow", secs = 30,
		ladder = 16, rung = 2, regress = 536, side = true, support = "bed", pain = { "back", "knee" }, mp = { BREATH },
		ms = { Ob, LB, Gl, BALANCE } }),
	[541] = move({ key = "kneeling_turn", group = "mobility", pattern = "mobility", class = 1, dose = "flow", secs = 30,
		ladder = 16, rung = 2, regress = 537, side = true, flags = { "floor" }, pain = { "wrist", "knee", "neck" },
		mp = { BREATH }, ms = { UB, Sh } }),
	[542] = move({ key = "spinal_twist_press", group = "mobility", pattern = "mobility", class = 1, dose = "flow",
		secs = 30, ladder = 16, rung = 2, regress = 538, side = true, flags = { "floor" }, pain = { "back" }, mp = { BREATH },
		ms = { Ob, LB, Gl } }),
	[268] = move({ key = "revolved_triangle", group = "mobility", pattern = "mobility", class = 2, dose = "flow",
		secs = 30, ladder = 16, rung = 3, anchor = true, regress = 417, side = true, pain = { "back", "hip", "neck" },
		mp = { BREATH }, ms = { Ob, Ha, LB } }),
	-- Beside the twisted triangle since slice 47 (2026-10-19), each on straight legs or with the bent knee unloaded: the
	-- wide-leg fold turned open over a palm under the shoulder (YogaUOnline's revolved wide-legged forward bend), the
	-- twisted half moon with the hand on books (Yogamatters), and the seated twist with the foot over the long knee
	-- (Ardha Matsyendrasana with the bottom leg long, its common easier form).
	[543] = move({ key = "revolved_wide_fold", group = "mobility", pattern = "mobility", class = 2, dose = "flow",
		secs = 30, ladder = 16, rung = 3, regress = 417, side = true, flags = { "head_down" },
		pain = { "back", "hip", "neck" }, mp = { BREATH }, ms = { Ob, Ha, Ad } }),
	[544] = move({ key = "revolved_half_moon_books", group = "mobility", pattern = "mobility", class = 2, dose = "flow",
		secs = 30, ladder = 16, rung = 3, regress = 417, side = true, pain = { "back", "hip", "neck" }, mp = { BREATH },
		ms = { Ob, Ha, Gl, BALANCE } }),
	[545] = move({ key = "half_fish_twist", group = "mobility", pattern = "mobility", class = 2, dose = "flow", secs = 30,
		ladder = 16, rung = 3, regress = 127, side = true, flags = { "floor" }, pain = { "back", "hip" }, mp = { BREATH },
		ms = { Ob, LB, Gl } }),
	[269] = move({ key = "bound_twist", group = "mobility", pattern = "mobility", class = 2, dose = "flow", secs = 30,
		ladder = 16, rung = 4, anchor = true, regress = 127, twin = 268, side = true, flags = { "floor" },
		pain = { "back", "shoulder" }, mp = { BREATH }, ms = { Ob, LB, Sh } }),
	[418] = move({ key = "lunge_prayer_twist", group = "mobility", pattern = "mobility", class = 2, dose = "flow",
		secs = 30, ladder = 16, rung = 4, regress = 267, side = true, pain = { "back", "knee" }, mp = { BREATH },
		ms = { Ob, Qu, Gl } }),
	-- Beside the bound seated twist since slice 47 (2026-10-19): the twisted half moon with the hand on the floor
	-- (Yogamatters) and the seated twist with the far arm straight down outside the knee to the foot (the Art of
	-- Living's Ardha Matsyendrasana).
	[546] = move({ key = "revolved_half_moon", group = "mobility", pattern = "mobility", class = 2, dose = "flow",
		secs = 30, ladder = 16, rung = 4, regress = 544, side = true, pain = { "back", "hip", "neck" }, mp = { BREATH },
		ms = { Ob, Ha, Gl, BALANCE } }),
	[547] = move({ key = "half_fish_foot_hold", group = "mobility", pattern = "mobility", class = 2, dose = "flow",
		secs = 30, ladder = 16, rung = 4, regress = 545, side = true, flags = { "floor" },
		pain = { "back", "hip", "shoulder" }, mp = { BREATH }, ms = { Ob, Gl, Sh } }),
	[270] = move({ key = "bound_side_angle", group = "mobility", pattern = "mobility", class = 3, dose = "flow",
		secs = 30, ladder = 16, rung = 5, anchor = true, regress = 418, side = true, flags = { "skill" },
		pain = { "back", "knee", "shoulder" }, mp = { BREATH }, ms = { Ob, Qu, Sh } }),
	-- Beside the bound lunge twist since slice 47 (2026-10-19), the step's binds with no bent knee bearing the body: the
	-- twisted triangle bound (Baddha Parivrtta Trikonasana) and the half lord of the fishes (Ardha Matsyendrasana I,
	-- 18 in Light on Yoga), sitting on the towel beside the folded foot.
	[548] = move({ key = "bound_revolved_triangle", group = "mobility", pattern = "mobility", class = 3, dose = "flow",
		secs = 30, ladder = 16, rung = 5, regress = 268, side = true, flags = { "skill" },
		pain = { "back", "hip", "shoulder" }, mp = { BREATH }, ms = { Ob, Ha, Sh } }),
	[549] = move({ key = "half_fish_bound", group = "mobility", pattern = "mobility", class = 3, dose = "flow", secs = 30,
		ladder = 16, rung = 5, regress = 547, side = true, flags = { "floor", "skill" },
		pain = { "back", "hip", "knee", "shoulder" }, mp = { BREATH }, ms = { Ob, Gl, Sh } }),
	-- The ankles and the deep squat (Spirit)
	[271] = move({ key = "desk_squat", group = "mobility", pattern = "mobility", class = 1, dose = "flow", secs = 30,
		ladder = 17, rung = 2, anchor = true, regress = 114, pain = { "knee", "achilles" }, mp = { BREATH },
		ms = { Ca, Qu, Gl } }),
	[272] = move({ key = "deep_squat", group = "mobility", pattern = "mobility", class = 1, dose = "flow", secs = 30,
		ladder = 17, rung = 3, anchor = true, regress = 271, pain = { "knee", "achilles" }, mp = { BREATH },
		ms = { Ca, Ad, Gl } }),
	-- The ladder gaps' poses (2026-10-12, David: "there needs to be more steps inbetween"), each beside the anchor of
	-- the rung below the step it prepares: the feet a fist apart with the toes ahead, half the stance change to the
	-- feet-together squat (a narrower stance asks the most extra of the ankle: Demers 2018), and the turn of the bound
	-- squat twist with both hands on a wall before its bind (Yoga International's first Pasasana variation).
	[419] = move({ key = "narrow_squat", group = "mobility", pattern = "mobility", class = 1, dose = "flow", secs = 30,
		ladder = 17, rung = 3, regress = 272, pain = { "knee", "achilles" }, mp = { BREATH }, ms = { Ca, Qu, Gl } }),
	[273] = move({ key = "feet_together_squat", group = "mobility", pattern = "mobility", class = 2, dose = "flow",
		secs = 30, ladder = 17, rung = 4, anchor = true, regress = 419, pain = { "knee", "achilles", "back" },
		mp = { BREATH }, ms = { Ca, Ad, LB } }),
	[274] = move({ key = "bound_squat", group = "mobility", pattern = "mobility", class = 2, dose = "flow", secs = 30,
		ladder = 17, rung = 5, anchor = true, regress = 273, pain = { "knee", "achilles", "shoulder", "back" },
		mp = { BREATH }, ms = { Ca, LB, Sh } }),
	[420] = move({ key = "squat_twist_wall", group = "mobility", pattern = "mobility", class = 2, dose = "flow",
		secs = 30, ladder = 17, rung = 5, regress = 274, side = true, pain = { "knee", "achilles", "back" },
		mp = { BREATH }, ms = { Ca, Ob, LB } }),
	[275] = move({ key = "noose", group = "mobility", pattern = "mobility", class = 3, dose = "flow", secs = 30,
		ladder = 17, rung = 6, anchor = true, regress = 420, side = true, flags = { "skill" },
		pain = { "knee", "achilles", "back", "shoulder" }, mp = { BREATH }, ms = { Ca, Ob, Sh } }),
	-- Slice 47's poses (2026-10-19), each at its own step: the bent-knee calf stretch for the low calf (soleus), the
	-- knee to the wall with the toes four fingers from it, half kneeling the knee a fist past the toes (until 2026-10-27
	-- 10 cm and Squat University's 12.7 cm knee-to-wall standard, the clinical normal and the unsupported squat's),
	-- and the knee over toe kneel and hover (the ATG split squat's bottom); the squats with the heels on a book or a
	-- towel (the deep squat test's raised-heel form, Cook 2006), a light bag hugged to the chest (the goblet hold, from
	-- step 4: a counterweight that keeps the trunk upright, never a load on the stretched calf; held far out it loads the
	-- back, whose pain flag removes it since 2026-10-20) and the arms reaching up and forward (the deep squat test's
	-- reach, straight up until 2026-10-27, its top score, which most adults miss); press and sink
	-- in the desk, deep, feet-together and bound twist squats (contract-relax, three rounds of a half-effort press);
	-- the squat prayer twist before the noose (Yoga Journal), and the noose's turn held by the body's own muscles, the
	-- arms reaching with no bind (Light on Yoga's way into it).
	[556] = move({ key = "soleus_stretch", group = "mobility", pattern = "mobility", class = 1, dose = "flow",
		secs = 30, ladder = 17, rung = 1, side = true, pain = { "achilles" }, mp = { BREATH }, ms = { Ca } }),
	[557] = move({ key = "knee_to_wall", group = "mobility", pattern = "mobility", class = 1, dose = "flow",
		secs = 30, ladder = 17, rung = 1, side = true, pain = { "achilles" }, mp = { BREATH }, ms = { Ca } }),
	[558] = move({ key = "heel_raised_desk_squat", group = "mobility", pattern = "mobility", class = 1, dose = "flow",
		secs = 30, ladder = 17, rung = 1, pain = { "knee", "achilles" }, mp = { BREATH }, ms = { Qu, Gl, Ca } }),
	[559] = move({ key = "half_kneel_ankle", group = "mobility", pattern = "mobility", class = 1, dose = "flow",
		secs = 30, ladder = 17, rung = 2, regress = 271, side = true, flags = { "floor" }, pain = { "knee", "achilles" },
		mp = { BREATH }, ms = { Ca, Qu } }),
	[560] = move({ key = "desk_squat_press", group = "mobility", pattern = "mobility", class = 1, dose = "flow",
		secs = 30, ladder = 17, rung = 2, regress = 271, pain = { "knee", "achilles" }, mp = { BREATH },
		ms = { Ca, Qu, Gl } }),
	[561] = move({ key = "heel_raised_squat", group = "mobility", pattern = "mobility", class = 1, dose = "flow",
		secs = 30, ladder = 17, rung = 2, regress = 271, pain = { "knee", "achilles" }, mp = { BREATH },
		ms = { Qu, Gl, Ad } }),
	[562] = move({ key = "deep_squat_press", group = "mobility", pattern = "mobility", class = 1, dose = "flow",
		secs = 30, ladder = 17, rung = 3, regress = 272, pain = { "knee", "achilles" }, mp = { BREATH },
		ms = { Ca, Ad, Gl } }),
	[563] = move({ key = "bag_hug_squat", group = "mobility", pattern = "mobility", class = 1, dose = "flow",
		secs = 30, ladder = 17, rung = 3, regress = 272, pain = { "knee", "achilles", "back" }, mp = { BREATH },
		ms = { Ca, Qu, Gl } }),
	[564] = move({ key = "overhead_squat", group = "mobility", pattern = "mobility", class = 1, dose = "flow",
		secs = 30, ladder = 17, rung = 3, regress = 272, pain = { "knee", "achilles", "shoulder" }, mp = { BREATH },
		ms = { Ca, Gl, Sh } }),
	[565] = move({ key = "knee_over_toe_kneel", group = "mobility", pattern = "mobility", class = 2, dose = "flow",
		secs = 30, ladder = 17, rung = 4, regress = 273, side = true, flags = { "floor" }, pain = { "knee", "achilles" },
		mp = { BREATH }, ms = { Ca, Qu } }),
	[566] = move({ key = "feet_together_reach", group = "mobility", pattern = "mobility", class = 2, dose = "flow",
		secs = 30, ladder = 17, rung = 4, regress = 273, pain = { "knee", "achilles", "back" }, mp = { BREATH },
		ms = { Ca, Qu, LB } }),
	[567] = move({ key = "feet_together_press", group = "mobility", pattern = "mobility", class = 2, dose = "flow",
		secs = 30, ladder = 17, rung = 4, regress = 273, pain = { "knee", "achilles", "back" }, mp = { BREATH },
		ms = { Ca, Ad, LB } }),
	[568] = move({ key = "knee_over_toe_hover", group = "mobility", pattern = "mobility", class = 2, dose = "flow",
		secs = 30, ladder = 17, rung = 5, regress = 274, side = true, pain = { "knee", "achilles" }, mp = { BREATH },
		ms = { Ca, Qu, Gl } }),
	[569] = move({ key = "squat_prayer_twist", group = "mobility", pattern = "mobility", class = 2, dose = "flow",
		secs = 30, ladder = 17, rung = 5, regress = 274, side = true, pain = { "knee", "achilles", "back" },
		mp = { BREATH }, ms = { Ca, Ob, LB } }),
	[570] = move({ key = "noose_press", group = "mobility", pattern = "mobility", class = 3, dose = "flow", secs = 30,
		ladder = 17, rung = 6, regress = 275, side = true, flags = { "skill" },
		pain = { "knee", "achilles", "back", "shoulder" }, mp = { BREATH }, ms = { Ca, Ob, Sh } }),
	[571] = move({ key = "noose_reach", group = "mobility", pattern = "mobility", class = 3, dose = "flow", secs = 30,
		ladder = 17, rung = 6, regress = 420, side = true, flags = { "skill" }, pain = { "knee", "achilles", "back" },
		mp = { BREATH }, ms = { Ca, Ob, LB } }),
	-- Balance (Intellect)
	[276] = move({ key = "eagle", group = "yoga", pattern = "balance", class = 1, dose = "flow", secs = 30, ladder = 18,
		rung = 2, anchor = true, regress = 130, side = true, pain = { "knee", "shoulder" }, mp = { BALANCE },
		ms = { Qu, Sh, Gl } }),
	-- The ladder gaps' poses (2026-10-12): the one-leg hinge to level with the hands on a wall (YogaUOnline) before
	-- Warrior III, and the leg held straight at about hip height with its heel on the desk (the raised heel of
	-- YogaUOnline's versions) before the big toe hold.
	[421] = move({ key = "wall_warrior3", group = "yoga", pattern = "balance", class = 1, dose = "flow", secs = 30,
		ladder = 18, rung = 2, regress = 130, side = true, pain = { "back", "knee", "hip" }, mp = { BALANCE },
		ms = { Gl, Ha, LB } }),
	[277] = move({ key = "warrior3", group = "yoga", pattern = "balance", class = 2, dose = "flow", secs = 30,
		ladder = 18, rung = 3, anchor = true, regress = 421, side = true, pain = { "back", "knee", "hip" },
		mp = { BALANCE }, ms = { Gl, Ha, LB } }),
	[278] = move({ key = "tree_eyes_closed", group = "yoga", pattern = "balance", class = 2, dose = "flow", secs = 30,
		ladder = 18, rung = 4, anchor = true, regress = 130, side = true, pain = { "knee" }, mp = { BALANCE },
		ms = { Ca, Gl } }),
	[422] = move({ key = "desk_heel_hold", group = "yoga", pattern = "balance", class = 2, dose = "flow", secs = 30,
		ladder = 18, rung = 4, regress = 256, side = true, pain = { "back", "hip" }, mp = { BALANCE },
		ms = { Ha, Ca, Gl } }),
	[279] = move({ key = "big_toe_hold", group = "yoga", pattern = "balance", class = 2, dose = "flow", secs = 30,
		ladder = 18, rung = 5, anchor = true, regress = 422, side = true, flags = { "skill" },
		pain = { "back", "hip", "knee" }, mp = { BALANCE }, ms = { Ha, Gl, Ab } }),
	[280] = move({ key = "dancer", group = "yoga", pattern = "balance", class = 3, dose = "flow", secs = 20, ladder = 18,
		rung = 6, anchor = true, regress = 277, side = true, flags = { "skill" },
		pain = { "shoulder", "back", "knee", "hip" }, mp = { BALANCE }, ms = { Qu, Sh, LB } }),
	-- Slice 47's balance poses (2026-10-19), each a one-leg balance at its own step beside the desk, a hand free
	-- to catch: the step's task with its stance, arms, gaze or surface changed (a thick-folded towel underfoot),
	-- the eyes closed only from step 7 (Shirley Ryan AbilityLab's balance progressions; balance gains stay with the
	-- task trained, Kümmel 2016). The half moon climbs from books (step 5, after the one-leg hinge at the wall) to the
	-- hand floating (9) and the half moon bow (11, Ardha Chandra Chapasana, the top hand holding the top foot); the
	-- standing figure four sits back (3), then folds (5).
	[576] = move({ key = "stork_head_turns", group = "yoga", pattern = "balance", class = 1, dose = "flow", secs = 30,
		ladder = 18, rung = 1, side = true, pain = { "neck" }, mp = { BALANCE }, ms = { Ca, Gl, Nk } }),
	[577] = move({ key = "knee_hug_stand", group = "yoga", pattern = "balance", class = 1, dose = "flow", secs = 30,
		ladder = 18, rung = 1, side = true, pain = { "hip" }, mp = { BALANCE }, ms = { Gl, Ca, Ab } }),
	[578] = move({ key = "towel_stand", group = "yoga", pattern = "balance", class = 1, dose = "flow", secs = 30,
		ladder = 18, rung = 1, side = true, mp = { BALANCE }, ms = { Ca, Gl, Ab } }),
	[579] = move({ key = "standing_figure_four", group = "yoga", pattern = "balance", class = 1, dose = "flow",
		secs = 30, ladder = 18, rung = 2, regress = 130, side = true, pain = { "knee", "hip" }, mp = { BALANCE },
		ms = { Qu, Gl, Ad } }),
	[580] = move({ key = "tree_eyes_up", group = "yoga", pattern = "balance", class = 1, dose = "flow", secs = 30,
		ladder = 18, rung = 2, regress = 130, side = true, pain = { "neck" }, mp = { BALANCE }, ms = { Ca, Gl, Sh } }),
	[581] = move({ key = "half_moon_books", group = "yoga", pattern = "balance", class = 2, dose = "flow", secs = 30,
		ladder = 18, rung = 3, regress = 421, side = true, pain = { "hip", "back" }, mp = { BALANCE },
		ms = { Gl, Ob, Ha } }),
	[582] = move({ key = "knee_out_hold", group = "yoga", pattern = "balance", class = 2, dose = "flow", secs = 30,
		ladder = 18, rung = 3, regress = 577, side = true, pain = { "hip", "knee" }, mp = { BALANCE },
		ms = { Gl, Ad, Ab } }),
	[583] = move({ key = "figure_four_fold", group = "yoga", pattern = "balance", class = 2, dose = "flow", secs = 30,
		ladder = 18, rung = 3, regress = 579, side = true, pain = { "knee", "hip", "back" }, mp = { BALANCE },
		ms = { Qu, Gl, LB } }),
	[584] = move({ key = "tiptoe_tree", group = "yoga", pattern = "balance", class = 2, dose = "flow", secs = 30,
		ladder = 18, rung = 4, regress = 130, side = true, pain = { "achilles" }, mp = { BALANCE }, ms = { Ca, Gl } }),
	[585] = move({ key = "towel_eyes_closed", group = "yoga", pattern = "balance", class = 2, dose = "flow", secs = 30,
		ladder = 18, rung = 4, regress = 578, side = true, mp = { BALANCE }, ms = { Ca, Gl, Ab } }),
	[586] = move({ key = "eagle_towel", group = "yoga", pattern = "balance", class = 2, dose = "flow", secs = 30,
		ladder = 18, rung = 4, regress = 276, side = true, pain = { "knee", "shoulder" }, mp = { BALANCE },
		ms = { Qu, Sh, Ca } }),
	[587] = move({ key = "side_toe_hold", group = "yoga", pattern = "balance", class = 2, dose = "flow", secs = 30,
		ladder = 18, rung = 5, regress = 582, side = true, flags = { "skill" }, pain = { "hip", "back", "knee" },
		mp = { BALANCE }, ms = { Ad, Ha, Gl } }),
	[588] = move({ key = "half_moon_float", group = "yoga", pattern = "balance", class = 2, dose = "flow", secs = 30,
		ladder = 18, rung = 5, regress = 581, side = true, flags = { "skill" }, pain = { "hip", "back" },
		mp = { BALANCE }, ms = { Gl, Ob, Ab } }),
	[589] = move({ key = "tiptoe_eagle", group = "yoga", pattern = "balance", class = 2, dose = "flow", secs = 30,
		ladder = 18, rung = 5, regress = 276, side = true, pain = { "knee", "achilles", "shoulder" }, mp = { BALANCE },
		ms = { Ca, Qu, Sh } }),
	[590] = move({ key = "half_moon_bow", group = "yoga", pattern = "balance", class = 3, dose = "flow", secs = 20,
		ladder = 18, rung = 6, regress = 581, side = true, flags = { "skill" },
		pain = { "hip", "back", "knee", "shoulder" }, mp = { BALANCE }, ms = { Qu, Gl, Sh } }),
	[591] = move({ key = "leg_up_no_hands", group = "yoga", pattern = "balance", class = 3, dose = "flow", secs = 20,
		ladder = 18, rung = 6, regress = 279, side = true, flags = { "skill" }, pain = { "hip", "back" },
		mp = { BALANCE }, ms = { Ab, Ha, Gl } }),
	-- Arm balances (Intellect): the wrist lean at the desk is the gate, a forward fall lands on a pillow; the headstand,
	-- in the corner of two walls, comes only once checked on the Exercises page (`optIn`)
	[281] = move({ key = "wrist_lean", group = "yoga", pattern = "balance", class = 1, dose = "flow", secs = 30,
		ladder = 19, rung = 1, anchor = true, pain = { "wrist" }, mp = { BALANCE }, ms = { Fa, Sh } }),
	[282] = move({ key = "crow_prep", group = "yoga", pattern = "balance", class = 1, dose = "flow", secs = 30,
		ladder = 19, rung = 2, anchor = true, regress = 281, twin = 600, flags = { "floor" }, pain = { "wrist", "knee" },
		mp = { BALANCE }, ms = { Fa, Sh, Ab } }),
	-- The ladder gaps' poses (2026-10-12): one foot off the floor before both (Peloton's crow, as the planche's frog
	-- stand with one foot up), and the front leg floating with the back toes still down before the flying splits
	-- (Yoga International's step by step).
	[423] = move({ key = "crow_one_foot", group = "yoga", pattern = "balance", class = 2, dose = "flow", secs = 30,
		ladder = 19, rung = 2, regress = 282, side = true, flags = { "floor", "space", "strain", "skill" },
		checks = { { ladder = LAD.push, step = 5, count = 10 } },
		pain = { "wrist", "elbow", "shoulder" }, mp = { BALANCE }, ms = { Tr, Sh, Ab, Fa } }),
	-- The crow from books beside them (2026-10-13, the readiness check): the toes start on a stack of books and both
	-- feet lift off, the crow's balance from a raised start before the floor (Antranik's crow progression: "20sec Crow
	-- Pose using Block" before "20sec Crow Pose"); rung 2's climb waits for both, its path's two stages, the one-foot
	-- crow and then this (slice 42; `proofAll` until then), and it is the crow's easier version.
	[426] = move({ key = "crow_books", group = "yoga", pattern = "balance", class = 2, dose = "flow", secs = 20,
		ladder = 19, rung = 2, regress = 423, flags = { "floor", "space", "strain", "skill" },
		checks = { { ladder = LAD.push, step = 5, count = 10 } },
		pain = { "wrist", "elbow", "shoulder" }, mp = { BALANCE }, ms = { Tr, Sh, Ab, Fa } }),
	[283] = move({ key = "crow", group = "yoga", pattern = "balance", class = 2, dose = "flow", secs = 30, ladder = 19,
		rung = 3, anchor = true, regress = 426, twin = 281, flags = { "floor", "space", "strain", "skill" },
		pain = { "wrist", "elbow", "shoulder" }, mp = { BALANCE }, ms = { Tr, Sh, Ab, Fa } }),
	[284] = move({ key = "headstand", group = "yoga", pattern = "balance", class = 2, dose = "flow", secs = 30,
		ladder = 19, rung = 3, regress = 282, optIn = true, flags = { "floor", "space", "head_down", "skill" },
		pain = { "neck", "shoulder" }, mp = { BALANCE }, ms = { Sh, Ab, Nk } }),
	[285] = move({ key = "side_crow", group = "yoga", pattern = "balance", class = 3, dose = "flow", secs = 20,
		ladder = 19, rung = 4, anchor = true, regress = 283, twin = 281, side = true,
		flags = { "floor", "space", "strain", "skill" }, pain = { "wrist", "elbow", "shoulder", "back" },
		mp = { BALANCE }, ms = { Tr, Ob, Sh, Fa } }),
	[424] = move({ key = "flying_splits_prep", group = "yoga", pattern = "balance", class = 3, dose = "flow",
		secs = 15, ladder = 19, rung = 4, regress = 285, side = true, flags = { "floor", "space", "strain", "skill" },
		checks = { { ladder = LAD.frontsplit, step = 7 }, { ladder = LAD.push, step = 6 } },
		pain = { "wrist", "elbow", "shoulder", "hip" }, mp = { BALANCE }, ms = { Tr, Sh, Ha, Ad } }),
	[286] = move({ key = "flying_splits", group = "yoga", pattern = "balance", class = 3, dose = "flow", secs = 15,
		ladder = 19, rung = 5, anchor = true, regress = 424, twin = 281, side = true,
		flags = { "floor", "space", "strain", "skill" }, pain = { "wrist", "elbow", "shoulder", "hip" },
		mp = { BALANCE }, ms = { Tr, Sh, Ha, Ad } }),
	-- Slice 47 (2026-10-19): the same level beside each step's main pose, each loading the wrist. Step 1, at the desk:
	-- the wrist's other directions and grips (GMB's and Mark Sisson's wrist work: fingers back and out to the side),
	-- the crow's grip with the palms lifted (Yoga International's crow: finger pads pressing), and the wrist lean
	-- pressed and sunk. Step 3: the planche lean at the desk (the crow prep's twin, so a setup without the floor climbs
	-- to the step) and pressed and sunk, the wrist leans on all fours with more weight on the hands, and the pendant's
	-- prep on books (hips and knees up, the feet still down). Steps 8 to 11 by Iyengar's ratings in Light on Yoga (the
	-- research's catalog of 2026-10-19): beside the crow, the step's main pose on bent arms and easier than Light on
	-- Yoga's Bakasana, the pendant from books (Lolasana, 6), the crane (Bakasana on straight arms, 9) and the peacock
	-- (Mayurasana, 9), the two harder than the bent-arm crow and below the side crow (16), so they stay at step 8 (at
	-- step 10 they would be easier than its main pose); the side crow with its top leg straight, the side crane (the
	-- side crow on straight arms) and the flying pigeon with its back foot on books beside the side crow; the firefly
	-- (Tittibhasana, 22), the side crow with straight legs (Dwi Pada Koundinyasana, 22) and the flying pigeon
	-- (Galavasana, 16) beside the flying splits (24). The planche leans at the desk, the pendant's prep and every pose
	-- from step 8 up carry the shoulder's pain tag, as the floor's planche lean does, and every one with both feet off
	-- the floor needs Room to move (2026-10-20). The straight-leg poses wait for the hamstrings' range and the pigeon's
	-- for the hip's, a one-time stretch check each (`checks`), so steps 10 and 11 offer all their poses only once those
	-- checks are in.
	[596] = move({ key = "reverse_wrist_lean", group = "yoga", pattern = "balance", class = 1, dose = "flow",
		secs = 30, ladder = 19, rung = 1, pain = { "wrist", "elbow" }, mp = { BALANCE }, ms = { Fa, Bi } }),
	[597] = move({ key = "side_wrist_lean", group = "yoga", pattern = "balance", class = 1, dose = "flow", secs = 30,
		ladder = 19, rung = 1, side = true, pain = { "wrist" }, mp = { BALANCE }, ms = { Fa, Sh } }),
	[598] = move({ key = "fingertip_lean", group = "yoga", pattern = "balance", class = 1, dose = "flow", secs = 30,
		ladder = 19, rung = 1, pain = { "wrist" }, mp = { BALANCE }, ms = { Fa, Sh } }),
	[599] = move({ key = "wrist_lean_press", group = "yoga", pattern = "balance", class = 1, dose = "flow", secs = 30,
		ladder = 19, rung = 1, pain = { "wrist" }, mp = { BALANCE }, ms = { Fa, Sh } }),
	[600] = move({ key = "desk_planche_lean", group = "yoga", pattern = "balance", class = 1, dose = "flow",
		secs = 30, ladder = 19, rung = 2, regress = 281, twin = 281, pain = { "wrist", "elbow", "shoulder" },
		mp = { BALANCE }, ms = { Fa, Sh, Ab } }),
	[601] = move({ key = "desk_planche_press", group = "yoga", pattern = "balance", class = 1, dose = "flow",
		secs = 30, ladder = 19, rung = 2, regress = 599, pain = { "wrist", "elbow", "shoulder" }, mp = { BALANCE },
		ms = { Fa, Sh, Ab } }),
	[602] = move({ key = "floor_wrist_lean", group = "yoga", pattern = "balance", class = 1, dose = "flow", secs = 30,
		ladder = 19, rung = 2, regress = 281, flags = { "floor" }, pain = { "wrist" }, mp = { BALANCE },
		ms = { Fa, Sh } }),
	[603] = move({ key = "floor_reverse_lean", group = "yoga", pattern = "balance", class = 1, dose = "flow",
		secs = 30, ladder = 19, rung = 2, regress = 596, flags = { "floor" }, pain = { "wrist", "elbow" },
		mp = { BALANCE }, ms = { Fa, Bi } }),
	[604] = move({ key = "pendant_prep", group = "yoga", pattern = "balance", class = 1, dose = "flow", secs = 20,
		ladder = 19, rung = 2, regress = 281, flags = { "floor", "strain" }, pain = { "wrist", "knee", "shoulder" },
		mp = { BALANCE }, ms = { Ab, Tr, Fa } }),
	[605] = move({ key = "crane", group = "yoga", pattern = "balance", class = 2, dose = "flow", secs = 20, ladder = 19,
		rung = 3, regress = 283, flags = { "floor", "space", "strain", "skill" }, pain = { "wrist", "elbow", "shoulder" },
		mp = { BALANCE }, ms = { Tr, Sh, Ab, Fa } }),
	[606] = move({ key = "pendant", group = "yoga", pattern = "balance", class = 2, dose = "flow", secs = 20,
		ladder = 19, rung = 3, regress = 604, flags = { "floor", "space", "strain", "skill" },
		pain = { "wrist", "elbow", "shoulder", "knee" }, mp = { BALANCE }, ms = { Ab, Tr, Fa } }),
	[607] = move({ key = "peacock", group = "yoga", pattern = "balance", class = 2, dose = "flow", secs = 20,
		ladder = 19, rung = 3, regress = 603, flags = { "floor", "space", "strain", "skill" },
		pain = { "wrist", "elbow", "shoulder" }, mp = { BALANCE }, ms = { Fa, Tr, Ab, Sh } }),
	[608] = move({ key = "side_crow_one_leg", group = "yoga", pattern = "balance", class = 3, dose = "flow",
		secs = 15, ladder = 19, rung = 4, regress = 285, side = true, flags = { "floor", "space", "strain", "skill" },
		checks = { { ladder = LAD.pike, step = 4 } }, pain = { "wrist", "elbow", "shoulder", "back" },
		mp = { BALANCE }, ms = { Tr, Ob, Sh, Ha } }),
	[609] = move({ key = "flying_pigeon_prep", group = "yoga", pattern = "balance", class = 3, dose = "flow",
		secs = 20, ladder = 19, rung = 4, regress = 283, side = true, flags = { "floor", "space", "strain", "skill" },
		checks = { { ladder = LAD.midsplit, step = 5, move = 440, mark = MARK.good } },
		pain = { "wrist", "elbow", "shoulder", "hip", "knee" }, mp = { BALANCE }, ms = { Tr, Sh, Gl, Fa } }),
	[610] = move({ key = "side_crane", group = "yoga", pattern = "balance", class = 3, dose = "flow", secs = 20,
		ladder = 19, rung = 4, regress = 285, side = true, flags = { "floor", "space", "strain", "skill" },
		pain = { "wrist", "elbow", "shoulder", "back" }, mp = { BALANCE }, ms = { Tr, Ob, Sh, Fa } }),
	[611] = move({ key = "firefly", group = "yoga", pattern = "balance", class = 3, dose = "flow", secs = 15,
		ladder = 19, rung = 5, regress = 605, flags = { "floor", "space", "strain", "skill" },
		checks = { { ladder = LAD.pike, step = 6 } }, pain = { "wrist", "elbow", "shoulder", "hip" }, mp = { BALANCE },
		ms = { Tr, Ab, Ha, Ad } }),
	[612] = move({ key = "side_crow_straight", group = "yoga", pattern = "balance", class = 3, dose = "flow",
		secs = 15, ladder = 19, rung = 5, regress = 285, side = true, flags = { "floor", "space", "strain", "skill" },
		checks = { { ladder = LAD.pike, step = 4 } }, pain = { "wrist", "elbow", "shoulder", "back" },
		mp = { BALANCE }, ms = { Tr, Ob, Sh, Ha } }),
	[613] = move({ key = "flying_pigeon", group = "yoga", pattern = "balance", class = 3, dose = "flow", secs = 15,
		ladder = 19, rung = 5, regress = 609, side = true, flags = { "floor", "space", "strain", "skill" },
		checks = { { ladder = LAD.midsplit, step = 5, move = 440, mark = MARK.good } },
		pain = { "wrist", "elbow", "shoulder", "hip", "knee" }, mp = { BALANCE }, ms = { Tr, Sh, Gl, Ha } }),

	-- Equipment. The wall, a doorframe and the desk edge are assumed (a bath towel shut in a door too, until it took
	-- the Towel and door box on 2026-10-28); everything below is a checkbox (/qp gear), each off by default.
	-- Pull-up bar: the pull-up ladder, every rung on the bar (the pull ladder's rungs 4-8 until 2026-10-04)
	[64] = move({ key = "dead_hang", group = "bar", pattern = "iso", region = "pull", class = 1, dose = "hold",
		ladder = 8, rung = 1, anchor = true, regress = 59, progress = 65, strainFree = true,
		needs = { { "bar", "fixed_bar" } }, pain = { "shoulder" }, mp = { Fa, La }, ms = { UB, Sh } }),
	[65] = move({ key = "scap_pulls", group = "bar", pattern = "pull", region = "pull", class = 2, dose = "rep",
		ladder = 8, rung = 2, anchor = true, regress = 64, progress = 386, needs = { { "bar", "fixed_bar" } },
		pain = { "shoulder" }, mp = { La, UB }, ms = { Nk, Fa } }),
	-- Rungs 3 and 4 on the bar alone (2026-10-10, the ladder rethink): the flexed-arm hang holds the top the negative
	-- starts from (Lind: "flexed hang at least 5 sec"; the r/bodyweightfitness routine's arch hangs between the
	-- shoulder-blade pulls and the negatives), and the 5-s lowering is rung 4's anchor, since the assisted pull-up needs
	-- a band or the Stable chair box and a player with only a bar stopped at rung 3. The negative at any speed and the
	-- assisted pull-up stay beside them as variants, each in its own window (Progress.lua `alt`).
	[386] = move({ key = "flexed_arm_hang", group = "bar", pattern = "iso", region = "pull", class = 2, dose = "hold",
		ladder = 8, rung = 3, anchor = true, regress = 228, progress = 387, flags = { "strain" },
		needs = { { "bar", "fixed_bar" } }, pain = { "elbow", "shoulder" }, mp = { La, Bi }, ms = { UB, Fa } }),
	[66] = move({ key = "pullup_negative", group = "bar", pattern = "pull", region = "pull", class = 3, dose = "rep",
		ladder = 8, rung = 3, regress = 228, progress = 387, flags = { "strain" },
		needs = { { "bar", "fixed_bar" } }, pain = { "elbow", "shoulder" }, mp = { La, Bi }, ms = { UB, Fa, Ab } }),
	[387] = move({ key = "pullup_negative_slow", group = "bar", pattern = "pull", region = "pull", class = 3,
		dose = "rep", ladder = 8, rung = 4, anchor = true, regress = 66, progress = 68, flags = { "strain" },
		needs = { { "bar", "fixed_bar" } }, pain = { "elbow", "shoulder" }, mp = { La, Bi }, ms = { UB, Fa, Ab } }),
	[67] = move({ key = "pullup_assisted", group = "bar", pattern = "pull", region = "pull", class = 3, dose = "rep",
		ladder = 8, rung = 4, regress = 66, progress = 68,
		needs = { { "bar", "fixed_bar" }, { "bands", "stable_chair" } }, pain = { "elbow", "shoulder" }, mp = { La, Bi },
		ms = { UB, Fa } }),
	-- The half pull-up beside the 5-s lowering (2026-10-12, the ladder gaps): the first bar move that pulls up, from
	-- the elbows at a right angle to the chin over the bar, where rungs 3 and 4 only hold and lower (Convict
	-- Conditioning's half pull-ups, 2x15 before the first full pull-ups; a lowering is about 1.4 times as strong as a
	-- pull, Nuzzo 2023).
	[398] = move({ key = "half_pullup", group = "bar", pattern = "pull", region = "pull", class = 3, dose = "rep",
		ladder = 8, rung = 4, regress = 387, progress = 68, flags = { "strain" }, needs = { { "bar", "fixed_bar" } },
		pain = { "elbow", "shoulder" }, mp = { La, Bi }, ms = { UB, Fa } }),
	[68] = move({ key = "pullup", group = "bar", pattern = "pull", region = "pull", class = 3, dose = "rep", ladder = 8,
		rung = 5, anchor = true, regress = 398, flags = { "strain" }, needs = { { "bar", "fixed_bar" } },
		pain = { "elbow", "shoulder" }, theme = { "climb" }, mp = { La, Bi }, ms = { UB, Fa, Ab } }),
	-- the hanging knee raise waits for a dead hang of 30 s (2026-10-16, slice 42): the grip holds the set
	[69] = move({ key = "hanging_knee_raise", group = "bar", pattern = "core_anti_ext", region = "trunk", class = 2,
		dose = "rep", ladder = 10, rung = 3, needs = { { "bar", "fixed_bar" } },
		checks = { { ladder = LAD.pullup, step = 4, move = 64, count = 30 } }, pain = { "shoulder" }, mp = { Ab },
		ms = { Qu, Fa, La } }),
	-- The pull-up's purple and orange (slice 22): the chest-to-bar and the L-sit pull-up, the archer, then one arm
	-- lowering alone, the other hand on a towel beside it, then one arm alone (tendon-heavy from rung 8: three days of
	-- proof). Since 2026-10-10 (the ladder rethink) the chest-to-bar pull-up anchors rung 6, a pull-up taken higher,
	-- and the L-sit pull-up is its variant; the one-arm lowering (Overcoming Gravity's "OAC eccentric", level 8)
	-- anchors rung 8, with the one-arm pull-up beside it whose free hand holds a towel hung over the bar at chest
	-- height (2026-10-12, the ladder gaps; the hand on the wrist until then, which Hybrid Calisthenics puts before the
	-- archer: "The lower you grip with your supporting arm, the harder").
	[309] = move({ key = "chest_to_bar_pullup", group = "bar", pattern = "pull", region = "pull", class = 3,
		dose = "rep", ladder = 8, rung = 6, anchor = true, regress = 68, flags = { "strain" },
		needs = { { "bar", "fixed_bar" } }, pain = { "elbow", "shoulder" }, mp = { La, UB }, ms = { Bi, Fa, Ab } }),
	-- The L-sit pull-up waits for the floor L-sit at step 8, or for a setup without the floor the chair L-sit held its
	-- top, 60 s (slice 54, 2026-10-23: the chair forms brought the L-sit to that setup, whose climb stops at step 6)
	[308] = move({ key = "lsit_pullup", group = "bar", pattern = "pull", region = "pull", class = 3, dose = "rep",
		ladder = 8, rung = 6, regress = 68, flags = { "strain", "skill" }, needs = { { "bar", "fixed_bar" } },
		checks = { { any = { { ladder = LAD.lsit, step = 8 }, { ladder = LAD.lsit, step = 8, move = 646,
			count = 60 } } } },
		pain = { "elbow", "shoulder" }, mp = { La, Bi }, ms = { Ab, Qu, UB, Fa } }),
	-- A two-hand pull biased to one arm before the archer, then the one-arm hold every one-arm lowering starts from
	-- (2026-10-12, the ladder gaps): the uneven towel pull-up beside the chest-to-bar (Convict Conditioning's uneven
	-- pull-ups, Hörst's uneven-grip pull-up), the one-arm lock-off beside the archer (Hooper's Beta: a two-arm pull and
	-- a one-arm hold before the one-arm lowering).
	[399] = move({ key = "uneven_towel_pullup", group = "bar", pattern = "pull", region = "pull", class = 3,
		dose = "rep", ladder = 8, rung = 6, regress = 309, side = true, flags = { "strain", "skill" },
		needs = { { "bar", "fixed_bar" } }, pain = { "elbow", "shoulder" }, mp = { La, Bi }, ms = { UB, Fa, Ob } }),
	[310] = move({ key = "archer_pullup", group = "bar", pattern = "pull", region = "pull", class = 3, dose = "rep",
		ladder = 8, rung = 7, anchor = true, regress = 399, side = true, flags = { "strain", "skill" },
		needs = { { "bar", "fixed_bar" } }, pain = { "elbow", "shoulder" }, theme = { "bow" }, mp = { La, Bi },
		ms = { UB, Fa, Ob } }),
	[400] = move({ key = "one_arm_lockoff", group = "bar", pattern = "iso", region = "pull", class = 3, dose = "hold",
		ladder = 8, rung = 7, regress = 310, side = true, flags = { "strain", "skill", "tendon" },
		needs = { { "bar", "fixed_bar" } }, pain = { "elbow", "shoulder" }, mp = { La, Bi }, ms = { UB, Fa, Ob } }),
	[388] = move({ key = "one_arm_pullup_negative", group = "bar", pattern = "pull", region = "pull", class = 3,
		dose = "rep", ladder = 8, rung = 8, anchor = true, regress = 400, progress = 312, side = true,
		flags = { "strain", "skill", "tendon" }, needs = { { "bar", "fixed_bar" } }, pain = { "elbow", "shoulder" },
		mp = { La, Bi }, ms = { UB, Fa, Ob } }),
	[311] = move({ key = "one_arm_pullup_assisted", group = "bar", pattern = "pull", region = "pull", class = 3,
		dose = "rep", ladder = 8, rung = 8, regress = 310, side = true,
		flags = { "strain", "skill", "tendon" }, needs = { { "bar", "fixed_bar" } }, pain = { "elbow", "shoulder" },
		mp = { La, Bi }, ms = { UB, Fa, Ob } }),
	[312] = move({ key = "one_arm_pullup", group = "bar", pattern = "pull", region = "pull", class = 3, dose = "rep",
		ladder = 8, rung = 9, anchor = true, regress = 388, side = true, flags = { "strain", "skill", "tendon" },
		needs = { { "bar", "fixed_bar" } }, pain = { "elbow", "shoulder" }, mp = { La, Bi }, ms = { UB, Fa, Ob } }),
	-- Power on the bar (slice 22), rungs 4-7 of the power ladder: a fixed bar with free space above it, never a doorway
	-- bar (the Fixed bar box: every other bar move takes either bar, 2026-10-05), and a box to step down on (each setup
	-- line says so); the jumping muscle-up jumps from that box (Step or box, and a jump: no jumping leaves it out), and
	-- the muscle-up is tendon-heavy.
	[353] = move({ key = "explosive_pullup", group = "bar", pattern = "ballistic", region = "pull", class = 3,
		dose = "rep", ladder = 23, rung = 4, anchor = true, regress = 309, flags = { "strain", "skill" },
		needs = { "fixed_bar" }, pain = { "elbow", "shoulder" }, mp = { La, Bi }, ms = { UB, Fa, Ab } }),
	[354] = move({ key = "high_pullup", group = "bar", pattern = "ballistic", region = "pull", class = 3, dose = "rep",
		ladder = 23, rung = 5, anchor = true, regress = 353, flags = { "strain", "skill" }, needs = { "fixed_bar" },
		pain = { "elbow", "shoulder" }, mp = { La, Bi }, ms = { UB, Fa, Ab } }),
	-- The press over the bar beside the high pull-up (the ladder rethink, 2026-10-10): nothing below the jumping
	-- muscle-up pressed on top of a bar, so its climb waits for the straight-bar dip (the r/bodyweightfitness
	-- Recommended Routine's dips come before the muscle-up; CrossFit's bar muscle-up plan asks for "3-5+ strict
	-- ring-dips"; Overcoming Gravity 2 rates bar dips level 3). Its easier version is its lowering beside it (the ladder
	-- gaps, 2026-10-12: the Recommended Routine's dips go from the support hold through negative dips to dips), whose own
	-- is the support hold below (2026-10-13), and the hold's the feet-up push-up, a press of the push ladder.
	[373] = move({ key = "straight_bar_dip", group = "bar", pattern = "push", region = "push", class = 3, dose = "rep",
		ladder = 23, rung = 5, regress = 405, flags = { "strain", "skill" }, needs = { "fixed_bar" },
		hint = 442, pain = { "wrist", "elbow", "shoulder" }, hrAfter = true, mp = { Tr, Ch }, ms = { Sh, Fa, Ab } }),
	-- The lowering and the support hold on rung 4 since 2026-10-16 (slice 42), beside the explosive pull-up: the hold
	-- first, the lowering once the hold reached its top (its check), the dip third, on rung 5 (its path).
	[405] = move({ key = "straight_bar_dip_lowering", group = "bar", pattern = "push", region = "push", class = 3,
		dose = "rep", ladder = 23, rung = 4, regress = 429, flags = { "strain", "skill" }, needs = { "fixed_bar" },
		checks = { { ladder = LAD.power, step = 8, move = 429, count = 60 } },
		hint = 442, pain = { "wrist", "elbow", "shoulder" }, hrAfter = true, mp = { Tr, Ch }, ms = { Sh, Fa, Ab } }),
	-- The support hold on top of the bar, the lowering's easier version (2026-10-13, the readiness check; the
	-- Recommended Routine's support hold before negative dips, CrossFit's bar muscle-up plan: "10 seconds or more ...
	-- static bar dip support hold", Bullbarfit: "Goal: 3 sets of 20-40 seconds"); since the same day each lowering takes
	-- about 5 s (the r/bodyweightfitness dip progression: "Work up to 10 sec descents"; Bullbarfit: "Lowering time: 5-8
	-- seconds").
	[429] = move({ key = "bar_support_hold", group = "bar", pattern = "iso", region = "push", class = 3, dose = "hold",
		ladder = 23, rung = 4, regress = 51, flags = { "strain", "skill" }, needs = { "fixed_bar" },
		hint = 442, pain = { "wrist", "elbow", "shoulder" }, mp = { Tr, Ch }, ms = { Sh, Fa, Ab } }),
	[355] = move({ key = "muscle_up_jump", group = "bar", pattern = "ballistic", region = "pull", class = 3,
		dose = "rep", ladder = 23, rung = 6, anchor = true, regress = 354, flags = { "impact", "strain", "skill" },
		jump = true, crung = 5, needs = { "fixed_bar", "step" }, hint = 442, pain = { "elbow", "shoulder", "wrist" },
		mp = { La, Ch }, ms = { Bi, Tr, UB, Ab } }),
	-- The muscle-up's lowering beside the jumping muscle-up (the ladder gaps, 2026-10-12): the jump from the box to the
	-- top, then down through the turn in 5 s or more, both elbows together, the turn the jump carries and the muscle-up
	-- does not (Overcoming Gravity 2: muscle-up negatives level 3, the strict bar muscle-up 7; Bullbarfit: "Lower
	-- yourself through the transition with a 5-10 second descent"). It jumps to the top as the jumping muscle-up does:
	-- under a bar with room for the muscle-up, no box lets the arms press there.
	[404] = move({ key = "muscle_up_negative", group = "bar", pattern = "pull", region = "pull", class = 3,
		dose = "rep", ladder = 23, rung = 6, regress = 373, flags = { "impact", "strain", "skill", "tendon" },
		jump = true, crung = 5, needs = { "fixed_bar", "step" }, hint = 442, pain = { "elbow", "shoulder", "wrist" },
		mp = { La, Ch }, ms = { Bi, Tr, UB, Ab } }),
	[356] = move({ key = "muscle_up", group = "bar", pattern = "ballistic", region = "pull", class = 3, dose = "rep",
		ladder = 23, rung = 7, anchor = true, regress = 355, flags = { "strain", "skill", "tendon" },
		needs = { "fixed_bar" }, hint = 442, pain = { "elbow", "shoulder", "wrist" }, mp = { La, Ch },
		ms = { Bi, Tr, UB, Ab } }),
	-- Bands with a door anchor
	[152] = move({ key = "pull_apart", group = "bands", pattern = "pull", region = "pull", class = 1, dose = "rep",
		ladder = 9, rung = 1, needs = { "bands" }, mp = { UB }, ms = { Sh } }),
	[150] = move({ key = "band_row", group = "bands", pattern = "pull", region = "pull", class = 1, dose = "rep",
		ladder = 9, rung = 2, needs = { "bands" }, pain = { "elbow" }, theme = { "oar" }, mp = { UB, La },
		ms = { Bi, Fa } }),
	[153] = move({ key = "band_curl", group = "bands", pattern = "pull", region = "pull", class = 1, dose = "rep",
		ladder = 9, rung = 2, needs = { "bands" }, upgrade = { 166 }, pain = { "elbow" }, mp = { Bi }, ms = { Fa } }),
	[151] = move({ key = "face_pull", group = "bands", pattern = "pull", region = "pull", class = 1, dose = "rep",
		ladder = 9, rung = 3, needs = { "bands" }, pain = { "shoulder" }, mp = { UB }, ms = { Sh, Nk } }),
	[154] = move({ key = "pallof_press", group = "bands", pattern = "core_anti_rot", region = "trunk", class = 1,
		dose = "rep", ladder = 10, rung = 3, side = true, needs = { "bands" }, pain = { "wrist" }, mp = { Ob },
		ms = { Ab, Sh } }),
	[155] = move({ key = "band_good_morning", group = "bands", pattern = "hinge", region = "legs", class = 1,
		dose = "rep", ladder = 4, rung = 2, needs = { "bands" }, pain = { "back" }, mp = { Ha, Gl }, ms = { LB } }),
	[156] = move({ key = "monster_walk", group = "bands", pattern = "lunge", region = "legs", class = 1, dose = "rep",
		ladder = 2, rung = 2, needs = { "bands" }, pain = { "hip" }, mp = { Gl }, ms = { Qu, Ad } }),
	-- Kettlebell (its deadlift is the deadlift with a weight since 2026-10-04, ns.Merged, and the bell takes the bag's
	-- place on the moves with a weight, 2026-10-09: holds; its carries take dumbbells too)
	[157] = move({ key = "kb_goblet_squat", group = "kettlebell", pattern = "squat", region = "legs", class = 2,
		dose = "rep", ladder = 2, rung = 4, needs = { "kettlebell" }, pain = { "knee", "wrist" }, load = "total",
		kg = { 8, 16 }, mp = { Qu, Gl }, ms = { Ad, Ab } }),
	[162] = move({ key = "kb_swing", group = "kettlebell", pattern = "ballistic", region = "legs", class = 3,
		dose = "rep", ladder = 4, rung = 4, regress = 219, flags = { "skill" }, needs = { "kettlebell" },
		pain = { "back", "wrist" }, load = "total", kg = { 8, 16 }, mp = { Gl, Ha }, ms = { LB, Fa, HEART } }),
	[158] = move({ key = "kb_suitcase_carry", group = "kettlebell", pattern = "carry", class = 1, dose = "flow",
		secs = 30, rung = 2, twin = 159, side = true, flags = { "space" }, needs = { { "kettlebell", "dumbbells" } },
		holds = { "kettlebell", "dumbbells" }, pain = { "wrist" }, theme = { "carry" }, mp = { Ob, Fa },
		ms = { LB, Nk, Gl } }),
	[159] = move({ key = "kb_farmer_march", group = "kettlebell", pattern = "carry", class = 1, dose = "flow",
		secs = 30, rung = 2, side = true, needs = { { "kettlebell", "dumbbells" } }, holds = { "kettlebell", "dumbbells" },
		pain = { "wrist" }, theme = { "carry" }, mp = { Fa, Nk }, ms = { Ab, Ca } }),
	[161] = move({ key = "kb_halo", group = "kettlebell", pattern = "mobility", class = 1, dose = "flow", noGoal = true,
		secs = 40, ladder = 15, rung = 1, needs = { "kettlebell" }, pain = { "shoulder" }, mp = { Sh },
		ms = { Ab, BREATH } }),
	-- Dumbbells (their row and their deadlift are the one-arm row and the deadlift with a weight since 2026-10-04)
	[165] = move({ key = "db_lateral_raise", group = "dumbbells", pattern = "push", region = "push", class = 1,
		dose = "rep", ladder = 1, rung = 2, needs = { "dumbbells" }, pain = { "shoulder" }, load = "each", kg = { 2, 5 },
		mp = { Sh }, ms = { Nk } }),
	[166] = move({ key = "db_hammer_curl", group = "dumbbells", pattern = "pull", region = "pull", class = 1,
		dose = "rep", ladder = 9, rung = 2, needs = { "dumbbells" }, pain = { "elbow" }, load = "each", kg = { 4, 8 },
		mp = { Bi, Fa } }),
	-- with a kettlebell too, the kettlebell's goblet squat takes its place: its horns are the classic goblet grip
	[163] = move({ key = "db_goblet_squat", group = "dumbbells", pattern = "squat", region = "legs", class = 2,
		dose = "rep", ladder = 2, rung = 4, needs = { "dumbbells" }, upgrade = { 157 }, pain = { "knee", "wrist" },
		load = "total", kg = { 8, 16 }, mp = { Qu, Gl }, ms = { Ad, Ab } }),
	[164] = move({ key = "db_reverse_lunge", group = "dumbbells", pattern = "lunge", region = "legs", class = 2,
		dose = "rep", ladder = 2, rung = 5, side = true, needs = { "dumbbells" }, pain = { "knee", "wrist" },
		load = "each", kg = { 4, 10 }, mp = { Qu, Gl }, ms = { Ad, Ha, Fa } }),
	-- Barbell with a rack beside the desk (each lift's kg the bar and its plates; the landmine's the plates on its end)
	[172] = move({ key = "landmine_press", group = "barbell", pattern = "push", region = "push", class = 2,
		dose = "rep", ladder = 1, rung = 4, side = true, needs = { "barbell" }, pain = { "shoulder" }, load = "total",
		kg = { 5, 15 }, mp = { Sh, Ch }, ms = { Tr, Ob } }),
	[170] = move({ key = "bb_rdl", group = "barbell", pattern = "hinge", region = "legs", class = 2, dose = "rep",
		ladder = 4, rung = 4, regress = 219, flags = { "skill" }, needs = { "barbell" }, pain = { "back", "wrist" },
		load = "total", kg = { 20, 40 }, mp = { Ha, Gl }, ms = { LB, Fa } }),
	[169] = move({ key = "bb_squat", group = "barbell", pattern = "squat", region = "legs", class = 2, dose = "rep",
		ladder = 2, rung = 5, regress = 10, flags = { "skill" }, needs = { "barbell" },
		pain = { "knee", "back", "wrist", "shoulder" }, load = "total", kg = { 20, 40 }, mp = { Qu, Gl }, ms = { Ad, LB } }),
	[171] = move({ key = "bb_press", group = "barbell", pattern = "push", region = "push", class = 2, dose = "rep",
		ladder = 1, rung = 5, regress = 172, flags = { "strain", "skill" }, needs = { "barbell" },
		pain = { "shoulder", "back" }, load = "total", kg = { 15, 25 }, mp = { Sh }, ms = { Tr, Nk, Ab } }),
	-- Jump rope
	[173] = move({ key = "shadow_skip", group = "rope", pattern = "cardio", class = 2, dose = "cardio", road = "gear",
		ladder = 6, rung = 3, flags = { "impact" }, jump = true, needs = { "rope" }, pain = { "achilles" }, mp = { HEART },
		ms = { Ca } }),
	[174] = move({ key = "rope_bounce", group = "rope", pattern = "cardio", class = 2, dose = "cardio", road = "gear",
		ladder = 6, rung = 4, flags = { "impact" }, jump = true, needs = { "rope" }, pain = { "achilles" }, mp = { HEART },
		ms = { Ca, Sh } }),
	[175] = move({ key = "rope_skips", group = "rope", pattern = "cardio", class = 3, dose = "cardio", road = "gear",
		ladder = 6, rung = 5, flags = { "impact", "loud" }, jump = true, needs = { "rope" }, pain = { "achilles" },
		mp = { HEART }, ms = { Ca, Sh, Fa } }),
	[176] = move({ key = "rope_alternate", group = "rope", pattern = "run", class = 3, dose = "cardio", road = "gear",
		ladder = 6, rung = 5, flags = { "impact", "loud" }, jump = true, needs = { "rope" }, pain = { "achilles" },
		mp = { HEART }, ms = { Ca, Qu } }),
	[177] = move({ key = "double_unders", group = "rope", pattern = "ballistic", class = 3, dose = "cardio", road = "gear",
		ladder = 6, rung = 7, regress = 175, endOnly = true, finisher = 7,
		flags = { "impact", "loud", "skill", "tendon" }, jump = true, needs = { "rope" }, pain = { "achilles" },
		mp = { HEART }, ms = { Ca, Sh, Fa } }),
	-- Exercise bike
	[178] = move({ key = "bike_easy", group = "bike", pattern = "cardio", class = 1, dose = "cardio", road = "gear",
		ladder = 6, rung = 1, flags = { "seated" }, needs = { "bike" }, mp = { HEART }, ms = { Qu, Gl } }),
	[179] = move({ key = "bike_moderate", group = "bike", pattern = "cardio", class = 2, dose = "cardio", road = "gear",
		ladder = 6, rung = 3, flags = { "seated" }, needs = { "bike" }, mp = { HEART }, ms = { Qu, Gl, Ha } }),
	[180] = move({ key = "bike_sprint", group = "bike", pattern = "cardio", class = 3, dose = "cardio", road = "gear",
		ladder = 6, rung = 1, endOnly = true, finisher = 1, flags = { "seated" }, needs = { "bike" }, mp = { HEART },
		ms = { Qu, Gl, Ha } }),
	-- Stairs
	[182] = move({ key = "stairs_brisk", group = "stairs", pattern = "cardio", class = 2, dose = "cardio", road = "gear",
		ladder = 6, rung = 3, needs = { "stairs" }, pain = { "knee" }, theme = { "climb" }, mp = { HEART },
		ms = { Qu, Gl, Ca } }),
	[184] = move({ key = "stairs_two", group = "stairs", pattern = "cardio", class = 3, dose = "cardio", road = "gear",
		ladder = 6, rung = 4, needs = { "stairs" }, pain = { "knee" }, theme = { "climb" }, mp = { HEART },
		ms = { Qu, Gl } }),
	[183] = move({ key = "stairs_allout", group = "stairs", pattern = "run", class = 3, dose = "cardio", road = "gear",
		ladder = 6, rung = 5, endOnly = true, finisher = 5, needs = { "stairs" }, pain = { "knee" }, theme = { "climb" },
		mp = { HEART }, ms = { Qu, Gl, Ca } }),
	[185] = move({ key = "heel_drops", group = "stairs", pattern = "calf", region = "legs", class = 1, dose = "rep",
		rung = 3, regress = 86, side = true, flags = { "tendon" }, needs = { { "stairs", "step" } },
		pain = { "achilles" }, mp = { Ca }, ms = { BALANCE } }),
	[186] = move({ key = "step_downs", group = "stairs", pattern = "squat", region = "legs", class = 2, dose = "rep",
		ladder = 2, rung = 4, side = true, needs = { { "stairs", "step" } }, pain = { "knee" }, mp = { Qu },
		ms = { Gl, BALANCE } }),
	-- Walking pad
	[187] = move({ key = "pad_easy", group = "pad", pattern = "cardio", class = 1, dose = "cardio", road = "gear",
		ladder = 6, rung = 1, needs = { "pad" }, theme = { "run" }, mp = { HEART }, ms = { Ca, Qu } }),
	[188] = move({ key = "pad_brisk", group = "pad", pattern = "cardio", class = 1, dose = "cardio", road = "gear",
		ladder = 6, rung = 2, needs = { "pad" }, theme = { "run" }, mp = { HEART }, ms = { Ca, Qu, Gl } }),
	[189] = move({ key = "pad_fast", group = "pad", pattern = "cardio", class = 2, dose = "cardio", road = "gear",
		ladder = 6, rung = 3, needs = { "pad" }, theme = { "run" }, mp = { HEART }, ms = { Ca, Qu, Gl } }),
	-- Step or box
	[190] = move({ key = "step_ups", group = "step", pattern = "squat", region = "legs", class = 2, dose = "rep",
		ladder = 2, rung = 3, side = true, needs = { "step" }, pain = { "knee" }, theme = { "climb" },
		mp = { Qu, Gl }, ms = { Ca, BALANCE } }),
	[191] = move({ key = "lateral_step_ups", group = "step", pattern = "lunge", region = "legs", class = 2,
		dose = "rep", ladder = 2, rung = 3, side = true, needs = { "step" }, pain = { "knee" }, mp = { Qu, Gl },
		ms = { Ad } }),
	-- the rear foot on a low step: rung 5 since 2026-10-16 (slice 42), easier than the Bulgarian's bed
	[193] = move({ key = "rfe_split_squat", group = "step", pattern = "lunge", region = "legs", class = 2,
		dose = "rep", ladder = 2, rung = 5, regress = 20, side = true, needs = { "step" }, pain = { "knee" },
		mp = { Qu, Gl }, ms = { Ad, Ha, BALANCE } }),
	[194] = move({ key = "step_over", group = "step", pattern = "cardio", class = 2, dose = "cardio", road = "gear",
		ladder = 6, rung = 3, needs = { "step" }, pain = { "knee" }, mp = { HEART }, ms = { Qu, Ca } }),
	-- Ab wheel
	[195] = move({ key = "rollout_short", group = "wheel", pattern = "core_anti_ext", region = "trunk", class = 2,
		dose = "rep", ladder = 10, rung = 3, flags = { "floor" }, needs = { "wheel" }, pain = { "back", "shoulder" },
		mp = { Ab }, ms = { La, Sh } }),
	[198] = move({ key = "rollout_wall", group = "wheel", pattern = "core_anti_ext", region = "trunk", class = 2,
		dose = "rep", ladder = 10, rung = 3, flags = { "floor" }, needs = { "wheel" }, pain = { "back", "shoulder" },
		mp = { Ab }, ms = { La, Sh } }),
	[197] = move({ key = "rollout_hold", group = "wheel", pattern = "core_anti_ext", region = "trunk", class = 3,
		dose = "hold", facing = "front", ladder = 10, rung = 4, flags = { "floor", "strain" }, needs = { "wheel" },
		pain = { "back", "shoulder" }, mp = { Ab }, ms = { La, Sh } }),
	[196] = move({ key = "rollout_full", group = "wheel", pattern = "core_anti_ext", region = "trunk", class = 3,
		dose = "rep", ladder = 10, rung = 5, regress = 198, flags = { "floor", "strain", "skill" }, needs = { "wheel" },
		pain = { "back", "shoulder" }, mp = { Ab }, ms = { La, Sh, Ob } }),
	-- the standing rollout, out and back, is Overcoming Gravity's full ab wheel (L8): the core's Artifact rung since
	-- 2026-10-05, above the dragon flag (a wheel's reach: without one the core tops out at orange); since 2026-10-10 on
	-- rung 11, above its lowering (OG's "Ab Wheel Ecc", L7; Kavadlo: "Start by just working the negative"): out from
	-- standing in 3 s, knees down, back in from the knees, the orange rung 10
	[382] = move({ key = "rollout_standing_negative", group = "wheel", pattern = "core_anti_ext", region = "trunk",
		class = 3, dose = "rep", ladder = 10, rung = 10, anchor = true, regress = 321, twin = 237,
		flags = { "floor", "strain", "skill" }, needs = { "wheel" }, pain = { "back", "shoulder" }, mp = { Ab },
		ms = { La, Sh, Ob } }),
	-- out from standing until the wheel meets a wall a full stretch ahead, and back to standing, beside the lowering
	-- (2026-10-12, the ladder gaps; Legendary Strength: "rolling against a wall"; StrongFirst: standing toward a wall,
	-- then farther): the pull back to standing, which the lowering never trains, from just short of the full reach
	[407] = move({ key = "rollout_standing_wall", group = "wheel", pattern = "core_anti_ext", region = "trunk",
		class = 3, dose = "rep", ladder = 10, rung = 10, regress = 382, flags = { "floor", "strain", "skill" },
		needs = { "wheel" }, pain = { "back", "shoulder" }, mp = { Ab }, ms = { La, Sh, Ob } }),
	[322] = move({ key = "rollout_standing", group = "wheel", pattern = "core_anti_ext", region = "trunk", class = 3,
		dose = "rep", ladder = 10, rung = 11, anchor = true, regress = 407, twin = 237,
		flags = { "floor", "strain", "skill" }, needs = { "wheel" },
		pain = { "back", "shoulder" }, mp = { Ab }, ms = { La, Sh, Ob } }),
	-- Rings or a suspension trainer; the ring row stands for the row's rung 3 in its chain after the table and the
	-- towel since 2026-10-28, so a player with rings and neither of them climbs that rung on the rings (and rungs 6-8
	-- on its feet-up rows, 653-655)
	[199] = move({ key = "ring_row", group = "rings", pattern = "pull", region = "pull", class = 2, dose = "rep",
		ladder = 9, rung = 3, twin = 648, needs = { "rings" }, pain = { "elbow", "shoulder" }, mp = { La, UB, Bi },
		ms = { Sh, Nk } }),
	[201] = move({ key = "ring_y_raise", group = "rings", pattern = "pull", region = "pull", class = 2, dose = "rep",
		ladder = 9, rung = 4, needs = { "rings" }, pain = { "shoulder" }, mp = { UB }, ms = { Sh, Nk } }),
	[200] = move({ key = "ring_press", group = "rings", pattern = "push", region = "push", class = 2, dose = "rep",
		ladder = 1, rung = 5, needs = { "rings" }, pain = { "wrist", "shoulder" },
		hrAfter = true, mp = { Ch, Tr }, ms = { Sh, Ab } }),
	[204] = move({ key = "ring_curl", group = "rings", pattern = "hinge", region = "legs", class = 2, dose = "rep",
		ladder = 4, rung = 4, flags = { "floor" }, needs = { "rings" }, mp = { Ha }, ms = { Gl, Ca } }),
	[203] = move({ key = "ring_fallout", group = "rings", pattern = "core_anti_ext", region = "trunk", class = 3,
		dose = "rep", ladder = 10, rung = 4, flags = { "strain" }, needs = { "rings" }, pain = { "back", "shoulder" },
		mp = { Ab }, ms = { La, Sh } }),
	[202] = move({ key = "ring_pistol", group = "rings", pattern = "squat", region = "legs", class = 3, dose = "rep",
		ladder = 2, rung = 6, regress = 22, side = true, flags = { "skill" }, needs = { "rings" }, pain = { "knee" },
		checks = { { ladder = LAD.ankle, step = 2 } },
		mp = { Qu, Gl }, ms = { Ca, Ab, BALANCE } }),
	-- Weighted vest
	[208] = move({ key = "vest_wall_sit", group = "vest", pattern = "iso", region = "legs", class = 2, dose = "hold",
		ladder = 7, rung = 1, strainFree = true, needs = { "vest" }, pain = { "knee" }, mp = { Qu }, ms = { Gl } }),
	[206] = move({ key = "vest_squat", group = "vest", pattern = "squat", region = "legs", class = 2, dose = "rep",
		ladder = 2, rung = 4, needs = { "vest" }, pain = { "knee" }, load = "total", kg = { 5, 10 }, mp = { Qu, Gl },
		ms = { Ad } }),
	[207] = move({ key = "vest_step_ups", group = "vest", pattern = "squat", region = "legs", class = 2, dose = "rep",
		ladder = 2, rung = 4, side = true, needs = { "vest", "step" }, pain = { "knee" }, theme = { "climb" },
		load = "total", kg = { 5, 10 }, mp = { Qu, Gl }, ms = { Ca, BALANCE } }),
	[209] = move({ key = "vest_stairs", group = "vest", pattern = "cardio", class = 3, dose = "cardio", road = "gear",
		ladder = 6, rung = 4, needs = { "vest", "stairs" }, pain = { "knee" }, theme = { "climb" }, mp = { HEART },
		ms = { Qu, Gl, Ca } }),
	-- Medicine or slam ball
	[210] = move({ key = "ball_goblet_squat", group = "ball", pattern = "squat", region = "legs", class = 1,
		dose = "rep", ladder = 2, rung = 3, needs = { "ball" }, pain = { "knee", "wrist" }, mp = { Qu, Gl },
		ms = { Ad, Ab } }),
	[214] = move({ key = "ball_chest_pass", group = "ball", pattern = "push", region = "push", class = 2, dose = "rep",
		ladder = 1, rung = 3, flags = { "loud" }, needs = { "ball" }, pain = { "wrist", "shoulder" }, mp = { Ch, Tr },
		ms = { Sh } }),
	[212] = move({ key = "ball_chop", group = "ball", pattern = "core_anti_rot", region = "trunk", class = 2,
		dose = "rep", ladder = 10, rung = 3, side = true, needs = { "ball" }, pain = { "back", "wrist", "shoulder" },
		theme = { "axe" }, mp = { Ob }, ms = { Sh, Ab } }),
	[211] = move({ key = "ball_overhead_carry", group = "ball", pattern = "carry", class = 2, dose = "flow",
		secs = 40, rung = 3, needs = { "ball" }, pain = { "shoulder" }, theme = { "carry" }, mp = { Sh, Ab },
		ms = { Nk, Tr } }),
	[213] = move({ key = "ball_slam", group = "ball", pattern = "ballistic", region = "trunk", class = 3, dose = "rep",
		rung = 3, flags = { "loud" }, needs = { "ball" }, pain = { "back", "wrist", "shoulder" }, theme = { "hammer" },
		mp = { La, Ab }, ms = { Sh, Gl, HEART } }),
	-- Foam roller
	[115] = move({ key = "roller_thoracic", group = "roller", pattern = "mobility", class = 1, dose = "flow",
		noGoal = true, secs = 60, ladder = 15, rung = 1, flags = { "floor" }, needs = { "roller" }, mp = { BREATH },
		ms = { UB } }),
	[215] = move({ key = "roller_calf_quad", group = "roller", pattern = "mobility", class = 1, dose = "flow",
		noGoal = true, secs = 60, ladder = 17, rung = 1, flags = { "floor" }, needs = { "roller" }, mp = { BREATH },
		ms = { Ca, Qu } }),
	[216] = move({ key = "roller_glute_lat", group = "roller", pattern = "mobility", class = 1, dose = "flow",
		noGoal = true, secs = 60, ladder = 13, rung = 1, flags = { "floor" }, needs = { "roller" }, mp = { BREATH },
		ms = { Gl, La } }),
	[217] = move({ key = "roller_dead_bug", group = "roller", pattern = "core_anti_ext", region = "trunk", class = 2,
		dose = "rep", ladder = 10, rung = 3, flags = { "floor" }, needs = { "roller" }, mp = { Ab }, ms = { Ob, Qu } }),
	-- Backpack with books or bottles, worn (the row and the deadlifts take any weight and need no checkbox since
	-- 2026-10-04: the hinge and the row above; the suitcase carry, in the Carry group, still names the pack; a weighted
	-- vest, a kettlebell or dumbbells take the squat's place, a kettlebell or dumbbells the carry's, 2026-10-09: upgrade)
	[218] = move({ key = "backpack_squat", group = "backpack", pattern = "squat", region = "legs", class = 2,
		dose = "rep", ladder = 2, rung = 4, needs = { "backpack" }, upgrade = { 206, 157, 163 }, pain = { "knee" },
		load = "total", kg = { 8, 12 }, mp = { Qu, Gl }, ms = { Ad, LB } }),
	-- the pack's cardio form since 2026-10-04 (the march before it went): worn, a squat that drives up onto the toes
	-- with the arms overhead, fast; a weighted vest does it too and comes first (2026-10-09)
	[293] = move({ key = "backpack_squat_drives", group = "backpack", pattern = "cardio", class = 3, dose = "cardio",
		road = "gear", ladder = 6, rung = 4, needs = { { "backpack", "vest" } }, holds = { "vest", "backpack" },
		pain = { "knee" }, theme = { "carry" }, mp = { HEART }, ms = { Qu, Gl, Ca, Sh } }),
	-- A pole outdoors (slice 22; David, 2026-10-03: "if you ever make it this far you can go outdors to find the
	-- pole"): the flags, the side ladder's purple and orange, on a steel post fixed in the ground. The lower arm pushes
	-- on a straight elbow and the upper arm hangs: tendon-heavy. Since 2026-10-10 (the ladder rethink) the clutch flag
	-- comes first, beside the Copenhagen raises, the lower elbow wedged at the hip so the body is short and the arms
	-- bent (Kavadlo: "When you can do a clutch flag for 20 seconds or longer, you might be ready to start training for
	-- the full human flag"), and the straddle flag, legs wide (Overcoming Gravity's flag level 7), is the step under
	-- the human flag, a rung added at the top.
	[383] = move({ key = "clutch_flag", group = "pole", pattern = "core_anti_rot", region = "trunk", class = 3,
		dose = "hold", facing = "side", ladder = 11, rung = 5, regress = 325, progress = 326, side = true,
		flags = { "strain", "skill" }, needs = { "pole" }, pain = { "shoulder", "elbow", "wrist" }, mp = { Ob, La },
		ms = { Bi, Sh, Ab } }),
	-- the vertical flag beside the clutch flag (2026-10-12, the ladder gaps; Kavadlo: a body "closer to being vertical"
	-- first, then the hips lowered; Antranik: "work up to a 20-30 second hold with straight arms"): the body straight up
	-- beside the pole, legs to the sky, the low arm locked and pushing and the high one pulling, the flag's straight-arm
	-- press and pull on a short lever, which the clutch flag's wedged elbow never trains
	[409] = move({ key = "vertical_flag", group = "pole", pattern = "core_anti_rot", region = "trunk", class = 3,
		dose = "hold", facing = "side", ladder = 11, rung = 5, regress = 383, side = true,
		flags = { "strain", "skill", "head_down" },
		checks = { { any = { { ladder = LAD.pullup, step = 8, count = 8 }, { ladder = LAD.row, step = 8, move = 313,
			count = 8 } } }, { ladder = LAD.push, step = 6 } },
		needs = { "pole" }, pain = { "shoulder", "elbow", "wrist" }, mp = { Ob, La }, ms = { Sh, Ab, Fa } }),
	[326] = move({ key = "flag_tuck", group = "pole", pattern = "core_anti_rot", region = "trunk", class = 3,
		dose = "hold", facing = "side", ladder = 11, rung = 6, anchor = true, regress = 409, side = true,
		flags = { "strain", "skill", "tendon" }, needs = { "pole" }, pain = { "shoulder", "elbow", "wrist" },
		mp = { Ob, La }, ms = { Sh, Ab, Fa } }),
	[327] = move({ key = "flag_one_leg", group = "pole", pattern = "core_anti_rot", region = "trunk", class = 3,
		dose = "hold", facing = "side", ladder = 11, rung = 7, anchor = true, regress = 326, side = true,
		flags = { "strain", "skill", "tendon" }, needs = { "pole" }, pain = { "shoulder", "elbow", "wrist" },
		mp = { Ob, La }, ms = { Sh, Ab, Gl } }),
	[384] = move({ key = "straddle_flag", group = "pole", pattern = "core_anti_rot", region = "trunk", class = 3,
		dose = "hold", facing = "side", ladder = 11, rung = 8, anchor = true, regress = 327, progress = 328, side = true,
		flags = { "strain", "skill", "tendon" }, needs = { "pole" }, hint = 413, pain = { "shoulder", "elbow", "wrist" },
		mp = { Ob, La }, ms = { Sh, Ab, Gl } }),
	[328] = move({ key = "human_flag", group = "pole", pattern = "core_anti_rot", region = "trunk", class = 3,
		dose = "hold", facing = "side", ladder = 11, rung = 9, anchor = true, regress = 384, side = true,
		flags = { "strain", "skill", "tendon" }, needs = { "pole" }, pain = { "shoulder", "elbow", "wrist" },
		mp = { Ob, La }, ms = { Sh, Ab, Gl } }),
}

-- The moves merged into another on 2026-10-04 (David: "instead of chair row (or similar excercise) or backpack row or
-- object row. Just consolidate it all into one with the instruction on how heave it should be"), old id to the move
-- that took it: the chair row and the single-arm dumbbell row into the one-arm row (57), the chair deadlift, the
-- hinge pick-ups, the kettlebell deadlift and the dumbbell deadlift into the deadlift with a weight (219). Their ids
-- are never reused, and their rows, records and unfinished moves are read as the surviving move's wherever rows are
-- read (DB.lua ns.Translate and ns.NewState; contract/library.json and moves.json carry the map to the phone apps
-- and the helper).
-- the brisk march's rows read as shadow boxing (its place on the cardio ladder's rung 2), and the talking-pace cardio
-- forms' as the hard form that took their place (the cross-knee drives, the quick steps, the step jacks, the brisk
-- stairs, the backpack squat drives), so an unfinished one is paid and never waits for a move that is gone (David,
-- 2026-10-04)
ns.Merged = { [226] = 57, [167] = 57, [233] = 219, [140] = 219, [160] = 219, [168] = 219, [3] = 101, [4] = 291,
	[5] = 95, [139] = 95, [181] = 182, [205] = 292, [220] = 293, [240] = 292 }

-- The holds that switch sides since 2026-10-10 22:00 UTC, each side on its own clock (David, 2026-10-10: "the chest
-- stretch in door frame is 45s but never told me to switch. fix and any others that have similar problems"): until
-- then the desk planks with a foot up and a knee in held both sides on one clock, switching halfway, so a set of
-- theirs from before that time held each side half its seconds and reads as that (DB.lua ns.Translate, both cores).
ns.Sided = { [236] = 1791669600, [237] = 1791669600 }

-- The ladders that lost their first rung, each at the moment it went (Unix seconds): the wall sit's 135° sit (16) at
-- 2026-10-27 08:00 UTC (David, 2026-10-26: "I also feel we can remove the shallow wall sit as first. the quarter one as
-- first is enough no?"), so its quarter sit is rung 1 and each rung above it one lower. A rung written before that
-- moment keeps its meaning and reads one lower, the first as the first: a row's (DB.lua ns.ShiftRow: a set's, a rung
-- row's, a mastered row's, whose first rung is gone and masters nothing), a baseline's folded before it that names no
-- shift of the ladder (DB.lua ns.ShiftBaseline) and a sheet summary's before version 6 (Stats.lua ns.ShiftSummary);
-- both cores and the desktop app read it from contract/library.json's `shifted` the same way.
ns.Shifted = { [LAD.wallsit] = 1793088000 }

-- The theme tags, a closed list (docs/plan.md slice 2, "Themes"): the quest table picks up to two per quest (slice 4),
-- the moves above carry the ones they act out, and the engine's theme score matches the two.
ns.THEMES = { "bear", "crab", "frog", "duck", "worm", "spider", "axe", "pick", "oar", "bow", "hammer", "climb", "swim",
	"carry", "dig", "run", "sneak", "fight", "ride" }

-- The roads of the cardio forms, a closed list in the wheel's order (slice 54, 2026-10-23): the five the week's omens
-- name in turn (ns.OMENS, Progress.lua), then the equipment forms' gear; a cardio step's card lists its forms in it.
ns.ROADS = { "run", "strike", "jump", "floor", "standing", "gear" }

-- The sides of the trunk a hold of the core or side ladder holds against, a closed list (slice 54): the four winds'
-- shares (C.facing_share) are McGill's normal endurance times of each.
ns.FACINGS = { "front", "side", "back" }

-- The equipment checkboxes, in the order /qp gear lists them (the two supports first, then the towel and door, then
-- the seventeen sets, the pole outdoors last: slice 22), the bar's with a box of its own for a fixed bar (2026-10-05:
-- the bar's power moves never go on a doorway bar, and a fixed bar serves every bar move), one for parallettes or two
-- stacks of books beside the ab wheel (2026-10-08: the raised L-sits, variants beside the floor's; no group of its
-- own), and one for a bath towel and a solid door the player pushes open (2026-10-28: the towel rows, off for every
-- player like the rest, where a towel shut in a door counted as assumed; a household piece beside the supports, in no
-- set and no group of its own).
ns.Gear = { "stable_chair", "table", "door_towel", "bar", "fixed_bar", "bands", "kettlebell", "dumbbells", "barbell",
	"rope", "bike", "stairs", "pad", "step", "wheel", "parallettes", "rings", "vest", "ball", "roller", "backpack",
	"pole" }

-- The equipment in five groups, in order (Rapha, 2026-11-03: "we have a lot more equipment shuold have an expand button
-- or arrow"): each piece of ns.Gear in exactly one, in ns.Gear's order within it (tools/lint.lua checks both). Your
-- setup and the Equipment page head each group with its caption (settings.gear.group.<key>); the apps open and close
-- them (contract/sheet.json gear_groups, places.gear.group.<key>).
ns.GearGroups = {
	{ key = "home", gear = { "stable_chair", "table", "door_towel", "stairs", "backpack" } },
	{ key = "bars", gear = { "bar", "fixed_bar", "parallettes", "rings", "pole" } },
	{ key = "weights", gear = { "bands", "kettlebell", "dumbbells", "barbell", "vest" } },
	{ key = "cardio", gear = { "rope", "bike", "pad", "step" } },
	{ key = "core", gear = { "wheel", "ball", "roller" } },
}
