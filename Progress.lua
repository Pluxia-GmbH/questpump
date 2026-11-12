-- Progression, pure (docs/plan.md slice 2, "Progression: beat your record"). Every rep and hold move keeps its record,
-- the best count typed on it, and its goal follows the player's recent form (ns.Goal): one step past the best of the
-- last fresh sets, a lighter goal for a repeat the same day; a move's first appearance is a max set with no number to
-- reach that becomes its record. A ladder of rungs a family: each rung's anchor (or the first twin the filters allow)
-- decides the rung through the rules below, which write rung rows; stars mark thirds of a rung's window; a step's path
-- orders the moves its climb also waits for (slice 42); the cardio ladder moves by clean days on its counted steps and
-- by full moves on its timed ones (slice 52: a Yes on a move dosed at the top M; ready days, slice 42, and finished
-- moves over distinct days before), a pose ladder by the mark its anchor names (slice 21); the steady ledger (slice 52)
-- sets the steady move from what the player keeps, in P's place. State comes from the fold (DB.lua), time from
-- ns.now(); nothing here stores anything.

local ADDON, ns = ...
local C, E = ns.C, ns.Enum
local O, K = E.outcome, E.kind

-- One ladder a family, each feeding one stat (slice 21: the stat rule moved from a move's kind to its family). The
-- rung windows (lo/hi, reps or seconds); `alt` gives a move its own window where it differs from the anchor's: a
-- substitute, or a variant counted in another unit (a hold beside reps, reps beside a hold: every move of a strength or
-- hold ladder has one in its own unit since 2026-10-16, slice 42, so it reaches a top, leaves the pumps of the step
-- above at it and can be mastered), whose top is its top on the step's path (ns.MoveTop). `step` places the rung's
-- anchor on one scale for every family (slice 20, David 2026-10-03: "a linear
-- progression from grey to orange rarity"): 1-14, two steps a colour (C.quality_by_step), set by hand against
-- Overcoming Gravity's levels, from blue up a step its level plus five since 2026-10-05 (plus four until then;
-- Iyengar's ratings in Light on Yoga for the poses), so a colour means the same class of feat on every ladder (the
-- push-up green, the pull-up blue, the pistol purple, the clip's freestanding handstand push-up and the one-arm push-up
-- orange, David 2026-10-05: "this is what orange should look like"; the feet-together one-arm push-up, the one-arm
-- pull-up, the front lever, the straddle planche and the human flag gold, Artifact). Rungs are only ever added
-- at the top: the rows carry the rung, so a rung put in between would move every one above it. A pose ladder (ns.POSE)
-- climbs by the mark its rung's anchor names (Locales: move.<key>.mark) and has no window.
--
-- A step's moves in the order the climb asks for them (2026-10-16, slice 42; David: "I feel we should have all that are
-- allowed by the setup in a proper order unlocked one after the others"): the rung's main move (its anchor, or the
-- move standing for it under the setup), then its `path`, the stages the climb from it waits for, in order, each
-- `{ ids, opens, days }`: `ids` the alternatives (one of them does the stage; a stage whose every one the setup shuts
-- is skipped), moves of the ladder at or under the rung outside its anchor's chain; `opens` when its moves are offered:
-- "step" with the step, "main" once the main move reached its top, a move id once that move reached its top, nil once
-- the stage before it did (ns.MoveOpen); `days` the game days at its top it takes, unlock_days unless named (a cardio
-- form's clean days, of the step's cardio_clean_days). "Top": a strength or hold move's own window top, a pose's mark
-- answered as the climb asks (ns.MarkNeeds), a countable form's clean window. Every other move of the rung is the same
-- level: variety, never required; topping every move the setup offers masters the step (ns.Mastered), a reward that
-- gates nothing. The path replaced `proof` and `proofAll` (2026-10-10 to 2026-10-16: the moves one of which, or every
-- one, the climb waited for at their top once; and under an orange or gold rung one more Epic move of the family).
--
-- `opens` (a list since 2026-10-16) names the steps of other ladders a family waits for, every one (ns.Open, slice 22):
-- the handstand opens with a push-up record of 15, the planche with one of 10 and the wrist lean answered Perfect, and
-- power with the cardio step its jumps have always needed; a rung's `needs` the steps of other ladders its climb waits
-- for, and a move's `checks` (Library.lua) those it waits for before it is offered (ns.RungNeed, ns.MoveOpen; the
-- one-time stretch checks among them, done from the card: ns.StartFor's "check"). Each entry is a step
-- `{ ladder, step }` done by its ladder's climb (a strength step by a record of its rung's own chain at its window's
-- floor, a pose step by its pose's mark, a cardio step by the ladder's rung), with a `count`, the record its anchor
-- must reach in place of the floor (2026-10-10), and with it a `move`, the move of that ladder whose record does it
-- (2026-10-12); a pose `move` with a `mark`, the least answer to its goal pose on one game day (2026-10-16); or
-- `{ any = { entries } }`, done when one of them is (2026-10-16: a pole and no bar).
-- The tops since 2026-10-13 (David, after 15 Bulgarian split squats brought the assisted pistol: "15 is by far not
-- enough for the next step ... tend to the higher ones to be sure people are sufficiently advanced to advance"): each
-- the highest published standard for the move (Hybrid Calisthenics' and Convict Conditioning's levels, Overcoming
-- Gravity 2, the Recommended Routine, the coaches' gates and the test norms) as one fresh set, sets of reps a quarter
-- to a half over the set's count and holds about 1.5 x a working hold, and no higher than censor_s allows a set at the
-- move's pace; a loaded move that stands for its rung (its top steps its weight up, ns.LoadTop) and a slower twin that
-- would pass censor_s keep the window they had as their own (docs/plan.md slice 22, "The higher tops").
local LAD = E.ladder
local MARK = E.mark
ns.Ladders = {
	-- The desk path keeps windows of its own (2026-10-12): the desk archer from 4 (20 slow desk push-ups predict 6-7, so
	-- a floor of 6 sat at the limit), the one-arm desk lowering to 12 (about 5 s a rep, a minute a side, under
	-- censor_s; twenty ran past it), the one-arm desk push-up from 2 to 15 (12 lowerings predict about 3; it had the
	-- feet-up push-up's 6-20). The push-up hold halfway down beside the push-up holds 10-45 s (2026-10-16).
	[1] = { key = "push", stat = "strength", region = "push", rungs = { { lo = 10, hi = 40, step = 1 },
		{ lo = 10, hi = 40, step = 2 },
		{ lo = 8, hi = 35, alt = { [46] = { lo = 8, hi = 20 } }, step = 3 },
		{ lo = 6, hi = 15, alt = { [221] = { lo = 4, hi = 20 }, [48] = { lo = 6, hi = 25 } }, step = 4 },
		{ lo = 6, hi = 30, alt = { [222] = { lo = 4, hi = 15 }, [50] = { lo = 10, hi = 45 } }, step = 5 },
		-- the moves in between that the climb waits for keep windows of their own (2026-10-10): the close push-up's,
		-- and the one-arm push-up's with the hand on the bed, a side; the staggered push-up's and the sliding one-arm
		-- push-up's, a side (2026-10-12); the one-arm push-up's with the feet hip-width, a side, gold's in place of the
		-- one-arm lowering, whose 6 the anchor's own top of 8 passes first (2026-10-13; the lowering stays beside it,
		-- the rung's entry). The archer's climb takes either the sliding one-arm push-up or the one with the hand on the
		-- bed (2026-10-13: alternate pairs never put a set of straight reps on one arm)
		{ lo = 6, hi = 25, alt = { [364] = { lo = 5, hi = 25 }, [223] = { lo = 2, hi = 15 },
			[394] = { lo = 6, hi = 25 }, [52] = { lo = 6, hi = 20 } },
			path = { { ids = { 364 }, opens = "step" }, { ids = { 394 } } }, step = 6 },
		{ lo = 4, hi = 20, alt = { [365] = { lo = 3, hi = 12 }, [395] = { lo = 4, hi = 18 }, [224] = { lo = 4, hi = 12 } },
			path = { { ids = { 395, 365 }, opens = "main" } }, step = 9 },
		{ lo = 3, hi = 12, alt = { [300] = { lo = 3, hi = 10 }, [428] = { lo = 3, hi = 10 } },
			path = { { ids = { 428 }, opens = "main" } }, step = 11 },
		{ lo = 1, hi = 10, step = 13 } } },
	-- rung 4's tempo squat (down 3, pause 1, up 1) tops at 15: twenty ran past censor_s (2026-10-04); rung 6's pistols
	-- with both hands on the desk or holding the straps beside the Bulgarian (2026-10-12), rung 7's assisted pistol to
	-- full depth with fingertips on the desk, the pistol to the bed edge and the one-leg lowering beside it, rung 8's
	-- pistol from 2, and rung 10's dragon pistol with a hand on the desk beside the held shrimp squat (2026-10-10)
	-- The Bulgarian's 3-s-lowering twin has 6-18 a side of its own, under censor_s, where the Bulgarian tops at 25
	-- (2026-10-13). The deep squat hold beside the air squat holds 30-120 s and the duck walk beside the tempo squat
	-- 20-60 s (2026-10-16: a sitting rest held a minute or two, the walk a minute at the most). The pistols wait for
	-- the ankle's range (2026-10-16): the desk pistols for its step 2, the assisted pistol's climb for its step 4, the
	-- pistol's for its step 6, the dragon pistols for the 90-90 sit too, the held shrimp squat's for the quad stretch.
	[2] = { key = "squat", stat = "strength", region = "legs", rungs = { { lo = 8, hi = 25, step = 1 },
		{ lo = 10, hi = 35, step = 2 }, { lo = 10, hi = 35, alt = { [15] = { lo = 30, hi = 120 } }, step = 3 },
		{ lo = 8, hi = 15, alt = { [135] = { lo = 20, hi = 60 } }, step = 4 },
		{ lo = 8, hi = 25, step = 5 },
		{ lo = 6, hi = 25, alt = { [22] = { lo = 6, hi = 18 }, [391] = { lo = 5, hi = 20 }, [202] = { lo = 5, hi = 20 } },
			path = { { ids = { 391, 202 }, opens = "main" } }, step = 6 },
		{ lo = 4, hi = 15, alt = { [375] = { lo = 4, hi = 15 }, [376] = { lo = 3, hi = 8 } },
			path = { { ids = { 376 }, opens = "main" } }, step = 7, needs = { { ladder = LAD.ankle, step = 4 } } },
		{ lo = 2, hi = 15, step = 9, needs = { { ladder = LAD.ankle, step = 6 } } },
		{ lo = 4, hi = 12, path = { { ids = { 303 }, opens = "step" } }, step = 10 },
		{ lo = 3, hi = 10, alt = { [377] = { lo = 2, hi = 8 } }, path = { { ids = { 377 }, opens = "main" } }, step = 11,
			needs = { { ladder = LAD.frontsplit, step = 1 } } },
		{ lo = 1, hi = 5, step = 12,
			needs = { { ladder = LAD.midsplit, step = 5, move = 440, mark = MARK.good },
				{ ladder = LAD.ankle, step = 6 } } } } },
	-- rung 4's single-leg slide out beside the sliding curl, rung 7's Nordic lowering all the way to the floor, and
	-- rung 8's Nordic pull-back, back up with no hands, its anchor from 2 to 6 (2026-10-10); rung 6's hover at the
	-- bottom of the Nordic and rung 8's fingertip push (2026-10-12); the hover with the hands on books beside it, the
	-- hover from 2 (2026-10-13); the Nordic hold on rung 7 since 2026-10-16, after the lowering
	[4] = { key = "hinge", stat = "strength", region = "legs", rungs = {
		{ lo = 12, hi = 40, alt = { [32] = { lo = 12, hi = 25 } }, step = 1 },
		{ lo = 8, hi = 25, alt = { [219] = { lo = 8, hi = 20 }, [234] = { lo = 8, hi = 20 } }, step = 2 },
		{ lo = 6, hi = 12, step = 3 },
		{ lo = 6, hi = 20, alt = { [287] = { lo = 8, hi = 15 }, [378] = { lo = 3, hi = 15 } },
			path = { { ids = { 378 }, opens = "main" } }, step = 4 },
		{ lo = 3, hi = 12, alt = { [288] = { lo = 6, hi = 12 } }, step = 7 },
		{ lo = 3, hi = 10, alt = { [392] = { lo = 2, hi = 15 }, [427] = { lo = 4, hi = 12 } },
			path = { { ids = { 427 }, opens = "main" }, { ids = { 392 } } }, step = 8 },
		{ lo = 3, hi = 8, path = { { ids = { 40 }, opens = "main" } }, step = 9 },
		{ lo = 2, hi = 8, alt = { [393] = { lo = 2, hi = 10 } }, path = { { ids = { 393 }, opens = "main" } }, step = 10 },
		{ lo = 1, hi = 5, step = 11 } } },
	-- The cardio ladder's paths (2026-10-16, slice 42): the forms a counted step's clean days must hold among them, with
	-- the step (the skater steps on one of step 5's, the squat thrusts or the sprawls on two of step 6's, the floor
	-- burpees on two of step 8's, so the desk path's step-back burpee alone never climbs a setup with the floor). Step
	-- 11 waits for 8 tuck jumps, or for a setup that shuts the tuck jump the squat's step 6, the stand of step 11's
	-- burpees (slice 54, 2026-10-23: the tuck jump alone until then, which shut step 11 to every no-jump setup).
	[6] = { key = "cardio", stat = "agility", rungs = { { step = 1 }, { step = 2 }, { step = 3 }, { step = 4 },
		{ step = 5, path = { { ids = { 411 }, opens = "step", days = 1 } } },
		{ step = 6, path = { { ids = { 97, 141 }, opens = "step", days = 2 } } },
		{ step = 8, path = { { ids = { 98 }, opens = "step", days = 2 } } },
		{ step = 10, needs = { { ladder = LAD.push, step = 6 } } },
		{ step = 11, needs = { { any = { { ladder = LAD.power, step = 6, move = 350, count = 8 },
			{ ladder = LAD.squat, step = 6 } } } } } } },
	-- The wall sit's windows shorten as the knee bends deeper, then its two one-leg rungs, timed a leg (2026-10-04).
	-- Since slice 54 (2026-10-23) the climb from two legs to one waits for the staggered sit, timed a side, 15-90 s (half
	-- the 90° sit's window), and the punches stand beside the 90° sit on its window. It starts at the quarter sit, step 2,
	-- since 2026-10-26, when the 135° sit's rung 1 (60-180 s, step 1) went (Library.lua ns.Shifted).
	[7] = { key = "wallsit", stat = "stamina", region = "legs", rungs = { { lo = 45, hi = 150, step = 2 },
		{ lo = 40, hi = 120, step = 4 },
		{ lo = 30, hi = 180, alt = { [642] = { lo = 15, hi = 90 } }, path = { { ids = { 642 }, opens = "main" } },
			step = 5 },
		{ lo = 15, hi = 75, step = 6 }, { lo = 20, hi = 90, step = 7 } } },
	-- The pull ladder split on 2026-10-04: the bar path (a bar for every move, from the dead hang to the one-arm
	-- pull-up) and the row (the doorframe, a weight in one hand and the table, in the order of the load an arm takes; a
	-- table to purple; a bar or rings to the front lever), each old rung on its old step. The pull-up's rung 3
	-- holds the flexed-arm hang and rung 4 the 5-s lowering since 2026-10-10 (the ladder rethink: the bar alone climbs
	-- to the pull-up), the negative and the assisted pull-up beside them in their old windows; the row's rung 9 the
	-- tuck lever's hold, its row and the table's archer row in reps beside it. The ladder gaps' moves keep windows of
	-- their own (2026-10-12): the half pull-up's reps beside the lowering, the one-arm lock-off's seconds a side beside
	-- the archer, the tuck lean-back hang's seconds beside rung 8's row. Since 2026-10-13 (the readiness check) the
	-- table row with the knees bent beside
	-- the one-arm row (a setup without a table climbs by the days alone), and the half-lay front lever's seconds beside
	-- the one-leg lever, the two levers 3-15 s. Since 2026-10-14 the table row's 3-s lowering with the knees bent beside
	-- them, 3-12 (about 5.5 s a rep); since 2026-10-16 the negative required after the flexed-arm hang, and the tuck
	-- lever's moves on the bar waiting for 8 pull-ups (their checks, Library.lua: the table's archer row climbs to the
	-- step without a bar, ns.CanRungUp). Since 2026-10-28 the one-arm doorframe row with the toes a foot back stands for
	-- rung 3 after the table and the rings in the rung's window (a side each), so the pull-up's 10 rows of step 3 come on
	-- the doorframe too, its hold at the top beside it in 10-45 s; the feet-up table rows, and after them the same rows
	-- on rings, stand for rungs 6-8 in their rungs' windows, the backpack's at 15 kg to climb (ns.LoadClimbs), the
	-- anchors since the towel rows went on 2026-11-06, and without a table or rings the row tops out at rung 5 (rungs
	-- 6-7 hold no bar move).
	[8] = { key = "pullup", stat = "strength", region = "pull", rungs = { { lo = 10, hi = 60, step = 4 },
		{ lo = 5, hi = 15, step = 5 },
		{ lo = 5, hi = 40, alt = { [66] = { lo = 3, hi = 6 } }, path = { { ids = { 66 }, opens = "main" } }, step = 6,
			needs = { { ladder = LAD.row, step = 3, count = 10 } } },
		{ lo = 3, hi = 8, alt = { [67] = { lo = 4, hi = 12 }, [398] = { lo = 4, hi = 18 } },
			path = { { ids = { 398 }, opens = "main" } }, step = 7 },
		{ lo = 1, hi = 15, step = 8 },
		{ lo = 3, hi = 12, alt = { [308] = { lo = 3, hi = 8 } }, path = { { ids = { 399 }, opens = "main" } }, step = 9 },
		{ lo = 2, hi = 12, alt = { [400] = { lo = 3, hi = 30 } }, path = { { ids = { 400 }, opens = "main" } }, step = 12 },
		{ lo = 2, hi = 6, alt = { [311] = { lo = 2, hi = 10 } }, path = { { ids = { 311 }, opens = "main" } }, step = 13 },
		{ lo = 1, hi = 3, step = 14 } } },
	[9] = { key = "row", stat = "strength", region = "pull", rungs = {
		{ lo = 10, hi = 35, alt = { [58] = { lo = 20, hi = 60 } }, step = 1 },
		{ lo = 10, hi = 20, alt = { [430] = { lo = 8, hi = 30 }, [432] = { lo = 3, hi = 12 } },
			path = { { ids = { 432 }, opens = "step" }, { ids = { 430 } } }, step = 2 },
		{ lo = 5, hi = 30, alt = { [60] = { lo = 5, hi = 15 }, [649] = { lo = 10, hi = 45 } }, step = 3 },
		{ lo = 8, hi = 25, step = 4 },
		{ lo = 4, hi = 15, step = 5 },
		{ lo = 3, hi = 10, step = 6 },
		{ lo = 3, hi = 12, step = 7 },
		{ lo = 4, hi = 12, alt = { [402] = { lo = 5, hi = 30 } }, path = { { ids = { 402 }, opens = "main" } }, step = 8 },
		{ lo = 5, hi = 30, alt = { [314] = { lo = 4, hi = 12 }, [315] = { lo = 4, hi = 12 }, [389] = { lo = 5, hi = 20 } },
			path = { { ids = { 389 }, opens = "main" }, { ids = { 314 }, opens = "main" } }, step = 10 },
		{ lo = 3, hi = 10, alt = { [390] = { lo = 3, hi = 15 }, [431] = { lo = 3, hi = 15 } },
			path = { { ids = { 390 }, opens = "main" }, { ids = { 431 } } }, step = 11 },
		{ lo = 3, hi = 10, step = 13 } } },
	-- The core hold split the same day: the planks (anti-extension) to the dragon flag, and the side (the side planks
	-- and the Copenhagen to blue, then the flags on a pole outdoors). A hold that stands in on a rung counted in reps
	-- keeps its own seconds (`alt`), and since 2026-10-16 (slice 42) every rep move beside a hold step its own reps,
	-- the higher of the published standards under censor_s at its pace: the dead bug 10-24 lowers (10-12 a side before
	-- harder variations, 3 x 10 a side the common baseline; the bird dog and the bird dog hold stood here until
	-- 2026-11-10, and the chops, 10-20 a side and 10-25, until 2026-11-11: timed drills off the ladders since,
	-- Library.lua's class line), McGill's curl-up 5-12 of its 5-s holds (his 6-4-2 pyramid of 8-10-s holds, a clinical
	-- start of 10),
	-- the hanging knee raise 6-15 (3 x 10-12 strict before straight legs), the inchworm 4-10, the Pallof press 8-15 a
	-- side, the short and the wall rollouts 8-25, the ball chop 8-15 a side, the roller dead bug 8-20, the ring fallout
	-- 5-15. The flags press on the low arm and pull on the high one, which nothing on the side ladder trains: their first
	-- rung waits for the pull-up (or, for a pole and no bar, rung 8's row on the table or the rings) and the push-up
	-- done (`needs`, 2026-10-09; the `any` since 2026-10-16). The clutch flag
	-- stands beside the Copenhagen raises in seconds, as the desk side plank, the last of that rung's chain, does in
	-- the seconds of the rung below (2026-11-11: it read the raises' reps as seconds until then, which tools/lint.lua
	-- refuses of any chain since), and the straddle flag's rung went in under the human flag at the top (2026-10-10,
	-- the ladder rethink).
	-- The desk planks switch sides (2026-10-10), so where they are dosed their windows are each side's: half the
	-- rung's, the same seconds in all. The core's top since 2026-10-10: the standing rollout's lowering the orange rung
	-- 10, the standing rollout the gold rung 11 above it; the tuck plow raise beside the body saw (rung 6's main move
	-- since 2026-11-11, when the body saw took the towel's box: the saw is the climb's stage for a setup with a towel,
	-- each in the window it had, and the incline plank, the last of the rung's chain, keeps the seconds the rung
	-- counted until then) and the hips-open tuck beside the tuck dragon flag. Since 2026-10-12 (the ladder
	-- gaps) the straddle dragon flag stands beside the dragon flag's lowering in its reps, and in windows of their own
	-- the standing rollout to a wall beside the standing lowering, the Copenhagen with the thigh on the edge beside the
	-- side plank and the vertical flag beside the clutch flag. The standing lowering's climb waits for the overhead
	-- range of the wall angels' goal pose (2026-10-16), answered Perfect since its grey goal eased to the thumbs a fist
	-- from the wall (2026-10-27; Good on the thumbs on the wall until then).
	[10] = { key = "core", stat = "stamina", region = "trunk", rungs = {
		{ lo = 30, hi = 90, alt = { [237] = { lo = 15, hi = 45 } }, step = 1 },
		{ lo = 30, hi = 90, alt = { [236] = { lo = 15, hi = 45 }, [79] = { lo = 10, hi = 24 }, [80] = { lo = 5, hi = 12 } },
			step = 2 },
		{ lo = 20, hi = 120, alt = { [236] = { lo = 10, hi = 60 }, [198] = { lo = 8, hi = 25 }, [69] = { lo = 6, hi = 15 },
			[84] = { lo = 4, hi = 10 }, [154] = { lo = 8, hi = 15 }, [195] = { lo = 8, hi = 25 }, [212] = { lo = 8, hi = 15 },
			[217] = { lo = 8, hi = 20 } }, step = 3 },
		{ lo = 10, hi = 45, alt = { [236] = { lo = 5, hi = 23 }, [203] = { lo = 5, hi = 15 } }, step = 6 },
		{ lo = 20, hi = 60, alt = { [236] = { lo = 10, hi = 30 }, [196] = { lo = 8, hi = 20 } }, step = 7 },
		{ lo = 8, hi = 20, alt = { [236] = { lo = 5, hi = 23 }, [70] = { lo = 10, hi = 45 }, [77] = { lo = 10, hi = 45 } },
			path = { { ids = { 77 }, opens = "step" } }, step = 8 },
		{ lo = 3, hi = 15, alt = { [237] = { lo = 15, hi = 45 }, [381] = { lo = 3, hi = 10 } },
			path = { { ids = { 381 }, opens = "main" } }, step = 9 },
		{ lo = 3, hi = 10, alt = { [237] = { lo = 15, hi = 45 } }, path = { { ids = { 406 }, opens = "main" } }, step = 10 },
		{ lo = 2, hi = 8, alt = { [237] = { lo = 15, hi = 45 } }, path = { { ids = { 196 }, opens = "step" } }, step = 11 },
		{ lo = 2, hi = 8, alt = { [237] = { lo = 15, hi = 45 }, [407] = { lo = 2, hi = 8 } },
			path = { { ids = { 407 }, opens = "main" } }, step = 12,
			needs = { { ladder = LAD.backbend, step = 1, move = 62, mark = MARK.perfect } } },
		{ lo = 1, hi = 5, alt = { [237] = { lo = 15, hi = 45 } }, step = 13 } } },
	[11] = { key = "side", stat = "stamina", region = "trunk", rungs = { { lo = 20, hi = 90, step = 4 },
		{ lo = 20, hi = 60, alt = { [408] = { lo = 10, hi = 60 } }, path = { { ids = { 408 }, opens = "step" } }, step = 5 },
		{ lo = 10, hi = 60, step = 6 },
		{ lo = 10, hi = 40, alt = { [324] = { lo = 20, hi = 60 }, [239] = { lo = 20, hi = 60 } }, step = 7 },
		{ lo = 5, hi = 15, alt = { [324] = { lo = 20, hi = 60 }, [239] = { lo = 20, hi = 60 }, [238] = { lo = 10, hi = 40 },
			[383] = { lo = 5, hi = 25 }, [409] = { lo = 5, hi = 30 } },
			path = { { ids = { 383 }, opens = "step" }, { ids = { 409 } } }, step = 9 },
		{ lo = 3, hi = 25, step = 10,
			needs = { { any = { { ladder = LAD.pullup, step = 8, count = 8 },
				{ ladder = LAD.row, step = 8, move = 652, count = 8 }, { ladder = LAD.row, step = 8, move = 655, count = 8 } } },
				{ ladder = LAD.push, step = 6 } } },
		{ lo = 3, hi = 20, step = 11 }, { lo = 3, hi = 15, step = 12 }, { lo = 2, hi = 7, step = 13 } } },
	-- The stretching and yoga families (slice 21): one rung a colour, its anchor standing where the pose allows (a
	-- setup with floor moves off, the default until 2026-10-04, starts every family too), the floor and space poses at
	-- the rungs above or as variants beside them; every pose of a family in one block of the day plan (`block`,
	-- ns.Block), so the family's turn there meets its mark. Steps by hand from Iyengar's ratings in Light on Yoga, which
	-- order a family's poses but do not compare families (a full split, pancake and pike are one class of feat, rated
	-- 36, 9 and 6 there). The skills a range decides wait for it (2026-10-16): the pancake for the seated fold, the
	-- twisted triangle and the big toe hold for the big toe fold, the dancer for the backbend's step 4 and the couch
	-- stretch (or, with Floor moves off, the quad stretch held Perfect: the dancer's own back leg standing).
	[12] = { key = "frontsplit", stat = "spirit", block = "mobility", rungs = { { step = 1 }, { step = 3 },
		{ step = 5, path = { { ids = { 412 }, opens = "step" } }, needs = { { ladder = LAD.pike, step = 4 } } },
		{ step = 7 }, { step = 9 }, { step = 11 } } },
	[13] = { key = "midsplit", stat = "spirit", block = "mobility", rungs = { { step = 1 }, { step = 3 }, { step = 5 },
		{ step = 7 }, { step = 9, path = { { ids = { 413 }, opens = "step" } }, needs = { { ladder = LAD.pike, step = 6 } } },
		{ step = 11 } } },
	[14] = { key = "pike", stat = "spirit", block = "mobility", rungs = { { step = 1 }, { step = 2 }, { step = 4 },
		{ step = 6 }, { step = 8 }, { step = 11 } } },
	[15] = { key = "backbend", stat = "spirit", block = "yoga", rungs = { { step = 1 }, { step = 2 },
		{ step = 4, path = { { ids = { 414 }, opens = "main" } } },
		{ step = 6, path = { { ids = { 415 }, opens = "main" } }, needs = { { ladder = LAD.push, step = 5 } } },
		{ step = 9, path = { { ids = { 416 }, opens = "main" } } }, { step = 11 } } },
	[16] = { key = "twist", stat = "spirit", block = "mobility", rungs = { { step = 1 },
		{ step = 3, path = { { ids = { 417 }, opens = "main" } } },
		{ step = 5, needs = { { ladder = LAD.pike, step = 4 } } },
		{ step = 8, path = { { ids = { 418 }, opens = "step" } } }, { step = 11 } } },
	[17] = { key = "ankle", stat = "spirit", block = "mobility", rungs = { { step = 1 }, { step = 2 },
		{ step = 4, path = { { ids = { 419 }, opens = "main" } } }, { step = 6 },
		{ step = 8, path = { { ids = { 420 }, opens = "step" } } },
		{ step = 11, needs = { { ladder = LAD.twist, step = 5 } } } } },
	[18] = { key = "balance", stat = "intellect", block = "yoga", rungs = { { step = 1 },
		{ step = 3, path = { { ids = { 421 }, opens = "step" } } }, { step = 5 },
		{ step = 7, path = { { ids = { 422 }, opens = "step" } } },
		{ step = 9, needs = { { ladder = LAD.pike, step = 4 } } },
		{ step = 11, needs = { { ladder = LAD.backbend, step = 4 }, { any = {
			{ ladder = LAD.frontsplit, step = 5, move = 412, mark = MARK.good },
			{ ladder = LAD.frontsplit, step = 1, move = 245, mark = MARK.perfect } } } } } } },
	[19] = { key = "armbalance", stat = "intellect", block = "yoga", rungs = { { step = 1 },
		{ step = 3, path = { { ids = { 423 }, opens = "main" }, { ids = { 426 } } } },
		{ step = 8, needs = { { ladder = LAD.push, step = 5, count = 10 } } },
		{ step = 10, path = { { ids = { 424 }, opens = "main" } }, needs = { { ladder = LAD.twist, step = 3 } } },
		{ step = 11, needs = { { ladder = LAD.push, step = 6 }, { ladder = LAD.frontsplit, step = 7 } } } } },
	-- Slice 22's families, each rung a record in reps or seconds: the handstand (the pike push-up, the wall handstand
	-- and its push-up, the freestanding handstand and its push-up), the L-sit (on the floor from both heels down to the
	-- L-sit and the V-sit, since 2026-10-08), the planche on the floor (the lean, the frog stand, the tuck, the
	-- straddle) and power (the squat jump, the explosive and the clap push-up, then on a bar the explosive pull-up to
	-- the muscle-up).
	[20] = { key = "handstand", stat = "intellect", region = "push",
		opens = { { ladder = LAD.push, step = 5, count = 15 } },
		-- the moves in between that the climb waits for keep windows of their own (2026-10-10): the wall handstand
		-- push-up's lowering in reps on the wall handstand's rung, the toes off the wall in seconds on its push-up's,
		-- the freestanding lowering in reps on the freestanding handstand's; the L-hold at the wall in seconds on the
		-- feet-up pike push-up's rung and the kick-up with its turn-out in reps on the wall handstand's (2026-10-12).
		-- The feet-up pike push-up waits for the hamstrings' and the overhead range (2026-10-16: the big toe fold and the
		-- wall angels' goal pose, answered Perfect since 2026-10-27, as the core's standing lowering asks it).
		rungs = { { lo = 5, hi = 18, step = 6 },
			{ lo = 5, hi = 15, alt = { [396] = { lo = 15, hi = 90 } }, path = { { ids = { 396 }, opens = "step" } },
				step = 7, needs = { { ladder = LAD.pike, step = 4 },
					{ ladder = LAD.backbend, step = 1, move = 62, mark = MARK.perfect } } },
			{ lo = 20, hi = 120, alt = { [366] = { lo = 2, hi = 6 }, [397] = { lo = 2, hi = 6 } },
				path = { { ids = { 397 }, opens = "step" }, { ids = { 366 }, opens = "main" } }, step = 8 },
			{ lo = 2, hi = 15, alt = { [367] = { lo = 2, hi = 15 } }, path = { { ids = { 367 }, opens = "step" } }, step = 9 },
			{ lo = 3, hi = 60, alt = { [368] = { lo = 2, hi = 8 } }, path = { { ids = { 368 }, opens = "main" } }, step = 10 },
			{ lo = 1, hi = 5, step = 11 } } },
	-- the seated leg lifts count reps beside the floor L-sit, and the low V-sit keeps its window beside the high-tuck
	-- V-sit, rung 6's anchor since 2026-10-10 (the ladder rethink), the one-leg V-sit its own a side beside them since
	-- 2026-10-12 (the ladder gaps); the legs lift straight from step 7 on, so those steps wait for the pike's range
	-- (2026-10-16: the big toe fold, the seated fold, forehead to knees)
	[21] = { key = "lsit", stat = "stamina", region = "trunk", rungs = { { lo = 20, hi = 60, step = 4 },
		{ lo = 10, hi = 45, step = 5 }, { lo = 5, hi = 60, path = { { ids = { 359 }, opens = "step" } }, step = 6 },
		{ lo = 10, hi = 60, step = 7, needs = { { ladder = LAD.pike, step = 4 } } },
		{ lo = 5, hi = 45, alt = { [385] = { lo = 6, hi = 15 } }, step = 8 },
		{ lo = 5, hi = 30, alt = { [340] = { lo = 5, hi = 20 }, [410] = { lo = 3, hi = 15 } },
			path = { { ids = { 385 }, opens = "step" }, { ids = { 410 }, opens = "main" }, { ids = { 340 } } }, step = 10,
			needs = { { ladder = LAD.pike, step = 6 } } },
		{ lo = 3, hi = 15, step = 12, needs = { { ladder = LAD.pike, step = 8 } } } } },
	[22] = { key = "planche", stat = "strength", region = "push",
		opens = { { ladder = LAD.push, step = 5, count = 10 }, { ladder = LAD.armbalance, step = 1, move = 281,
			mark = MARK.perfect } },
		rungs = { { lo = 15, hi = 60, alt = { [369] = { lo = 10, hi = 30 }, [425] = { lo = 3, hi = 20 } },
			path = { { ids = { 369 }, opens = "step" }, { ids = { 425 } } }, step = 5 },
			{ lo = 5, hi = 60, step = 8 },
			{ lo = 5, hi = 45, alt = { [344] = { lo = 5, hi = 15 }, [370] = { lo = 5, hi = 20 } },
				path = { { ids = { 370 }, opens = "main" } }, step = 9 },
			{ lo = 3, hi = 40, alt = { [371] = { lo = 3, hi = 20 } }, path = { { ids = { 371 }, opens = "main" } }, step = 10 },
			{ lo = 3, hi = 30, alt = { [372] = { lo = 2, hi = 8 } }, path = { { ids = { 372 }, opens = "main" } }, step = 11 },
			{ lo = 2, hi = 15, step = 13 } } },
	-- The explosive push-up, the clap push-up on the knees and the clap push-up wait for 12 feet-up push-ups, about 17 on
	-- the floor (2026-10-13: the NSCA asks for a bench press of 1-1.5 x body weight or 5 clap push-ups in a row before
	-- upper-body plyometrics; the step's own need until 2026-10-16, when it moved to the two moves, the clap push-up's
	-- since 2026-10-24): the step's stage takes the clap push-up on the knees or the frog jumps beside the tuck jump,
	-- either at its top on two game days (2026-10-17, slice 42's review: with the knee clap push-up alone the jumper's
	-- climb waited for the push record after all), so the tuck jump's path waits for no push record while Room to move is
	-- on, and for the 12 feet-up push-ups without it (the frog jumps need the room, and the one other jump at that level,
	-- the strict burpee, comes only from cardio rung 7; 2026-10-18, slice 42's last review). A jumper's climb ends at step
	-- 6, where the tuck jump stands, until the push record opens step 7's clap push-up: the tuck jump, of step 6's level,
	-- never climbs into it (ns.CanRungUp; until 2026-10-24 it did, and offered the clap push-up without the push record:
	-- David, "why would that unlock?"). The strict burpee beside the tuck jump and the jumping muscle-up's lowering
	-- keep their windows of their own, and the support hold on the bar and the dip's lowering stand beside the explosive
	-- pull-up since 2026-10-16, the lowering after the hold's top, the dip on the next step after the lowering's
	[23] = { key = "power", stat = "agility", region = "legs", opens = { { ladder = LAD.cardio, step = 5 } },
		rungs = { { lo = 6, hi = 20, step = 5 },
			{ lo = 4, hi = 12, alt = { [350] = { lo = 5, hi = 15 }, [134] = { lo = 5, hi = 12 }, [349] = { lo = 6, hi = 15 } },
				path = { { ids = { 403, 134 }, opens = "step" } }, step = 6 },
			{ lo = 3, hi = 12, alt = { [350] = { lo = 5, hi = 15 } }, step = 7 },
			{ lo = 3, hi = 10, alt = { [405] = { lo = 3, hi = 10 }, [429] = { lo = 10, hi = 60 } }, step = 8,
				needs = { { ladder = LAD.pullup, step = 9, count = 6 } } },
			{ lo = 2, hi = 6, alt = { [373] = { lo = 3, hi = 15 } }, path = { { ids = { 373 }, opens = 405 } }, step = 9 },
			{ lo = 2, hi = 8, alt = { [404] = { lo = 2, hi = 6 } }, path = { { ids = { 404 }, opens = "step" } }, step = 10 },
			{ lo = 1, hi = 5, step = 12 } } },
	-- The sit-up (2026-10-17, David: "yes add"): trunk flexion in reps on the floor, the crunch to the V-up and the sit-up
	-- with a weight hugged to the chest. Each climb waits for its in-between move at its top (the rung's `path`, slice
	-- 42): the knee raise for the reverse crunch, the slow lowering for the sit-up, the
	-- leg raise bent up for the tuck-up, the one-leg V-up for the V-up, whose step also waits for the pike's fourth (the
	-- straight legs' range). Every top fits censor_s at its move's pace (about 3 s a curl, 5 s the leg raise bent up)
	-- and the curls' stop at 25 or under, below the test norms (the YMCA's 49 a minute), for the flexion count
	-- (Contreras and Schoenfeld 2011: about 60 a session); the top rung climbs by its weight in 6-15 reps. It pays
	-- Stamina, as the trunk's other families (the curl's reps are the field tests' muscular endurance; Strength until
	-- slice 42 took main in, 2026-10-17, when with strength first it lifted a Balanced year's Strength past twice its
	-- median: docs/plan.md slice 42, "Built, main taken in").
	[24] = { key = "situp", stat = "stamina", region = "trunk",
		rungs = { { lo = 10, hi = 25, path = { { ids = { 444 }, opens = "step" } }, step = 2 },
			{ lo = 8, hi = 25, alt = { [446] = { lo = 3, hi = 12 } }, path = { { ids = { 446 }, opens = "main" } },
				step = 3 },
			{ lo = 5, hi = 25, alt = { [449] = { lo = 5, hi = 15 } }, path = { { ids = { 449 }, opens = "main" } },
				step = 4 },
			{ lo = 5, hi = 20, alt = { [451] = { lo = 8, hi = 25 }, [452] = { lo = 4, hi = 12 } },
				path = { { ids = { 452 }, opens = "main" } }, step = 5 },
			{ lo = 3, hi = 20, alt = { [454] = { lo = 10, hi = 30 } }, step = 6, needs = { { ladder = LAD.pike, step = 4 } } },
			{ lo = 6, hi = 15, step = 7 } } },
	-- The lower back (2026-11-10, David: "imo there should be whole line for it"): the back's extensors in reps to a
	-- straight line and never past it, grey to blue as the sit-up's, past its first step the legs lifted first and the
	-- trunk on top (the trunk lift is the heavier one: De Ridder 2013). The kneeling back raise anchors the first step,
	-- whose window is its own 8-16, with the hinge hold, the step's way without the floor, in its own seconds, and its
	-- climb waits for no stage: the bird dog (the anchor until 2026-11-11), its hold and the one-leg reverse hyper (the
	-- step's stage until then) are timed drills off the ladder, since no set of them ends at the form (Library.lua's
	-- class line). Each later climb waits for its in-between move at its top (the rung's `path`): the reverse hyper
	-- hold for the reverse hyper (the straight legs' lever held level), the chest lift for the
	-- back extension (the trunk lifted at a short lever, lighter than the reverse hyper beside it, so it opens with the
	-- step since 2026-11-11 and no longer waits for 20 reverse hypers), the back extension hold for the long back
	-- extension (the line held before the arms lengthen the lever); the pack adds load to the back extension, done with
	-- the fingertips at the temples. The tops: 25 back extensions and a 90-s level hold are one adult-average band
	-- (Alaranta 1994: 24-28 reps, 87-97 s), the reverse hypers 20 (the Recommended Routine's 3 x 8-12 as one fresh set,
	-- the higher end), and the Superman hold, the highest floor compression, 60 s beside the reverse hyper, on no path
	-- (120 s on the core ladder until then). The thirteen moves of 2026-11-11 (kneeling, prone on the floor, anchored,
	-- with a band, across a table: Library.lua) are variety at their rungs, on no path and, but the kneeling back raise,
	-- in no chain, each with a window of its own but the table reverse hyper, which takes the reverse hyper's, and the
	-- kneeling back raise, whose own is its rung's; each top is what fits censor_s at
	-- its figure's pace (the seconds of its motion file's loop; the voice counts at it): the kneeling back raise 16 (5.3
	-- s a rep), the kneeling good morning 12 (7.0 s with its 3-s hold) and the long one 10 (8.4 s, its fold slow), the
	-- swimmer lifts 15 pairs (9.6 s a pair, whose clock is a pair's, twice censor_s), the floor leg lift and the long
	-- chest lift 12 (7.0 s), the long back extension hold 60 s, the slow long back extension 10 (7.2 s), the band moves
	-- 15 (4.4 s a reach on a side, 3.6 s a lift), and the two with a typed weight in 6 reps to the count their weight
	-- steps up at: the chest lift with a pack 12 (7.0 s), the kneeling good morning with a weight 14 (6.2 s). The first
	-- build's paces: about 3.2 s a rep, 4.8 s a pair of one-leg lifts, 5.5 s the chest lift, 7.2 s the paused reverse
	-- hyper; the top rung climbs by its weight in 6-15 reps. It pays Strength, as the hinge, the
	-- back side's other family (its anchors work the back at strength level, 62-71 % of its most at 60 % of one rep's
	-- most: De Ridder 2013), though its region is the trunk: on Stamina, the trunk's other families' stat, it lifted
	-- Stamina past Agility under the Endurance focus (seeds 2 and 3) or past Spirit under the Flexibility focus (seeds
	-- 1 and 2), so on every seed two of the three focuses ended "of the Bear" (make sim, hub, seeds 1-3, 2026-11-10:
	-- docs/plan.md slice 22, "The lower back ladder"). It stands outside the Trial: its first max set places it.
	[25] = { key = "lowerback", stat = "strength", region = "trunk",
		rungs = { { lo = 8, hi = 16, alt = { [639] = { lo = 30, hi = 90 } }, step = 2 },
			{ lo = 8, hi = 20, alt = { [658] = { lo = 20, hi = 60 }, [668] = { lo = 5, hi = 12 },
					[669] = { lo = 8, hi = 15 }, [670] = { lo = 8, hi = 15 } },
				path = { { ids = { 658 }, opens = "main" } }, step = 3 },
			{ lo = 8, hi = 20, alt = { [660] = { lo = 6, hi = 15 }, [640] = { lo = 20, hi = 60 },
					[671] = { lo = 5, hi = 10 }, [672] = { lo = 5, hi = 12 } },
				path = { { ids = { 660 }, opens = "step" } }, step = 4 },
			{ lo = 10, hi = 25, alt = { [662] = { lo = 5, hi = 12 }, [663] = { lo = 30, hi = 90 },
					[674] = { lo = 5, hi = 12 }, [675] = { lo = 6, hi = 12 } },
				path = { { ids = { 663 }, opens = "main" } }, step = 5 },
			{ lo = 8, hi = 20, alt = { [665] = { lo = 6, hi = 15 }, [676] = { lo = 20, hi = 60 },
					[677] = { lo = 6, hi = 14 } }, step = 6 },
			{ lo = 6, hi = 15, alt = { [678] = { lo = 5, hi = 10 }, [679] = { lo = 8, hi = 15 } }, step = 7 } } },
}

-- The highest ladder id: the lists that go by id (the sheet summary's lad, the library's cadence) run 1..LADDER_MAX,
-- a retired id among them.
ns.LADDER_MAX = 0
for id in pairs(ns.Ladders) do ns.LADDER_MAX = math.max(ns.LADDER_MAX, id) end

-- The strength and hold ladders in the Trial's order (one placement a pump): push, squat, the core, the row, the hinge,
-- the wall sit, the side and the pull-up (a ladder no move of the setup serves waits outside the count: the pull-up
-- without a bar). Slice 22's families stand outside it: each is placed by its own first max set once it is open.
ns.TrialOrder = { LAD.push, LAD.squat, LAD.core, LAD.row, LAD.hinge, LAD.wallsit, LAD.side, LAD.pullup }
local IN_TRIAL = {}
for _, l in ipairs(ns.TrialOrder) do IN_TRIAL[l] = true end
ns.FLOOR_MOVE = 95 -- step jacks: always legal (marching in place until 2026-10-03; David: "its just too boring")

-- The day plan's blocks (David, 2026-10-03: "its more improtant a good variety and trainingplan is followed than that
-- the excercises follow what happened in the quest"), in the order a tie between them goes.
ns.BLOCKS = { "strength", "cardio", "mobility", "yoga" }

-- A rung row's `reps` says why the rung changed (the helper and the phone apps read the same codes): `easier` is the
-- step down an easier pick on the ladder's own attempt makes (ns.EasierDown, 2026-10-13), `check` a pose family lifted
-- to the step of a check answered close enough (2026-10-16, slice 42; ns.Progression: a placement, never a rung lost,
-- which the fold takes as a climb, never lowering a rung), each of which a reader without it folds as any other rung
-- set.
ns.RungCause = { up = 1, plateau = 2, down = 3, start = 4, placementLow = 5, placementHigh = 6, undo = 7, easier = 8,
	check = 9 }

local REGION_LADDER = { push = LAD.push, legs = LAD.squat, pull = LAD.row, trunk = LAD.core }
ns.REGION_LADDER = REGION_LADDER -- read by tools/gen-engine.lua for the phone cores' library
-- The regions in the order every list of them shows (the Today tab's max sets, the day card's moves by region, the
-- phones' Book filter through tools/gen-fixtures.lua; the Saga's spec, 2026-10-15).
ns.REGIONS = { "push", "pull", "legs", "trunk" }
local STRENGTH, POSE = ns.STRENGTH, ns.POSE

-- Indexes built once from the library: the anchor of every rung and its chain of twins, and the top rung of each
-- ladder that holds a move.
local anchors, chains, held, members = {}, {}, {}, {}
for id, m in pairs(ns.Library) do
	if m.anchor then
		anchors[m.ladder] = anchors[m.ladder] or {}
		anchors[m.ladder][m.rung] = id
	end
	if m.ladder then
		held[m.ladder] = math.max(held[m.ladder] or 0, m.rung)
		members[m.ladder] = members[m.ladder] or {}
		table.insert(members[m.ladder], id)
	end
end

function ns.Anchor(ladder, rung)
	return anchors[ladder] and anchors[ladder][rung]
end

-- The anchor and its twins in order, the moves that can stand for a rung.
function ns.RungChain(ladder, rung)
	local key = ladder * 16 + rung
	if chains[key] then return chains[key] end
	local list, seen = {}, {}
	local id = ns.Anchor(ladder, rung)
	while id and not seen[id] do
		seen[id] = true
		list[#list + 1] = id
		id = ns.Library[id].twin
	end
	chains[key] = list
	return list
end

function ns.InChain(ladder, rung, moveId)
	if not ladder or not rung or not ns.Ladders[ladder] or not ns.Ladders[ladder].rungs[rung] then return false end
	for _, id in ipairs(ns.RungChain(ladder, rung)) do
		if id == moveId then return true end
	end
	return false
end

-- A pose whose goal pose counts its family's day at a rung (slice 47, 2026-10-19: any goal pose of the step; its own
-- chain's alone from slice 21 until then): a pose of that family with a goal pose (ns.HasGoal) at the rung's own
-- level, never a stand-in from below nor one that names no goal pose. Its main pose, which the climb's floor counts
-- (ns.PoseNeeds) and an orange or gold step's colour is earned on (DB.lua earnMark), is the one of the rung's own chain
-- (ns.InChain) among them.
function ns.Marks(ladder, rung, moveId)
	local m = ns.Library[moveId]
	return POSE[ladder] == true and m ~= nil and m.ladder == ladder and m.rung == rung and ns.HasGoal(moveId)
end

-- Whether a pose is its step's main pose (slice 47): one of the rung's own chain (its anchor, or a twin of its step
-- standing for it where the setup shuts the anchor) among the step's goal poses (ns.Marks), the one whose days the
-- climb's floor counts (ns.PoseNeeds, DB.lua markAttempt) and an orange or gold step's colour is earned on (earnMark);
-- the test ns.Marks was from slice 21 until then.
function ns.MainPose(ladder, rung, moveId)
	return ns.Marks(ladder, rung, moveId) and ns.InChain(ladder, rung, moveId)
end

-- The mark a goal pose names (Locales: move.<key>.mark, under the lint's mark rules).
function ns.MarkText(moveId)
	return ns.L["move." .. ns.Library[moveId].key .. ".mark"]
end

-- The least answer to "How close did you get?" (E.mark) that counts a day of a pose family's mark toward the next step
-- (2026-10-09, David: "Maybe 4 options or so on how well the pose could be done between perfect, good, okay, bad?"):
-- Good, or Perfect alone where the next step's pose is risky (head-down, a balance, or flagged skill or strain), so a
-- riskier pose opens only once the one under it is done in full.
function ns.MarkNeeds(ladder, rung)
	local up = ns.Anchor(ladder, rung + 1)
	local risky = up ~= nil and (ns.Library[up].pattern == "balance" or ns.Flagged(up, "head_down")
		or ns.Flagged(up, "skill") or ns.Flagged(up, "strain"))
	return risky and E.mark.perfect or E.mark.good
end

-- The climb's days from a pose family's step, from its counts at the rung (slice 47, 2026-10-19; the fold's hiDays,
-- anchorDays and perfectDays, DB.lua markAttempt: a goal pose of the step, ns.Marks, held whole and answered as close
-- as ns.MarkNeeds asks, each day a new session by the night rule): the days of pose_unlock_days, those on the step's
-- main pose (ns.MainPose) of pose_anchor_days (pose_anchor_days_risky where only Perfect counts) and those answered
-- Perfect of pose_perfect_days where Good counts too (where only Perfect counts every counted day is one, so that
-- part asks none: perfectNeed 0). Each part's shortfall, `left` the least counted days still to come (one day can
-- bring every part), `have` the days toward the step (need less left, so it reaches need only with every part in) and
-- `done` whether they are in.
local function poseNeeds(ladder, rung, days, main, perfect)
	local risky = ns.MarkNeeds(ladder, rung) >= E.mark.perfect
	local need, perfectNeed = C.pose_unlock_days, risky and 0 or C.pose_perfect_days
	local mainNeed = risky and C.pose_anchor_days_risky or C.pose_anchor_days
	local daysLeft = math.max(0, need - days)
	local mainLeft, perfectLeft = math.max(0, mainNeed - main), math.max(0, perfectNeed - perfect)
	local left = math.max(daysLeft, mainLeft, perfectLeft)
	return { days = days, need = need, main = main, mainNeed = mainNeed, perfect = perfect, perfectNeed = perfectNeed,
		daysLeft = daysLeft, mainLeft = mainLeft, perfectLeft = perfectLeft, left = left, have = math.max(0, need - left),
		done = left == 0 }
end

-- The one reading of a pose family's days toward its next step (poseNeeds, from `state` at `rung`; a state with no
-- ladders, `{}`, the climb with nothing counted yet): the climb (ns.Progression, ns.PoseDaysIn), the progress every
-- card and ring shows (ns.UnlockProgress, with its fifth and sixth numbers), the pose due for the family's turn
-- (ns.DuePose) and the step card's words (Tree.lua, Panel.lua; the desktop app's and the phones' cards).
function ns.PoseNeeds(state, ladder, rung)
	local lad = state.ladders and state.ladders[ladder]
	return poseNeeds(ladder, rung, lad and lad.hiDays or 0, lad and lad.anchorDays or 0, lad and lad.perfectDays or 0)
end

-- Whether a pose family's days toward the step above `rung` are in (ns.PoseNeeds' `done`; its path, its setup and the
-- steps of other ladders aside).
function ns.PoseDaysIn(state, ladder, rung)
	return ns.PoseNeeds(state, ladder, rung).done
end

-- Whether row `r` (a pose family's hold that counted its day, DB.lua markAttempt: `topDay`, with `mainDay` and
-- `perfectDay` where it brought those parts) made the family's days in at `rung`: in after it and not before, so the
-- path's event and the top-out come once, on that hold (ns.Progression).
local function poseMadeIn(state, ladder, rung, r)
	if not r.topDay then return false end
	local p = ns.PoseNeeds(state, ladder, rung)
	return p.done and not poseNeeds(ladder, rung, p.days - 1, p.main - (r.mainDay and 1 or 0),
		p.perfect - (r.perfectDay and 1 or 0)).done
end

-- The game days a pose's mark was answered at least `need` (E.mark: Good, or Perfect) on, each a new session by the
-- night rule (the fold's `marks`, DB.lua markDays, 2026-10-12): a pose's days at its top on a step's path (ns.TopDays),
-- a pose step done and a goal pose a check names (stepDone) read them.
function ns.MarkDays(state, moveId, need)
	local d = state.marks and state.marks[moveId]
	if not d then return 0 end
	if need >= E.mark.perfect then return d.perfect or 0 end
	return d.good or 0
end

-- A move stands for a rung when it is in the chain of that rung or of a rung below: ns.RungMove walks down when the
-- filters leave the rung's own chain empty (a knee flag at squat rung 3 leaves the chair-height squat), and that
-- substitute's attempts keep the ladder going like the anchor's (its records, stars and the rung down), but only a set
-- of a move of the rung's own level counts the days that climb it (DB.lua anchorAttempt, ns.CanClimb; 2026-10-09).
function ns.StandsFor(ladder, rung, moveId)
	if not ladder or not rung then return false end
	for r = rung, 1, -1 do
		if ns.InChain(ladder, r, moveId) then return true end
	end
	return false
end

-- The ladder whose rung the rung rule compares a move with: its own, else its region's (the hinge pattern its own).
function ns.ScaleLadder(m)
	if m.ladder then return m.ladder end
	if m.dose == "cardio" then return ns.CARDIO end
	if m.pattern == "hinge" then return ns.Enum.ladder.hinge end
	return m.region and REGION_LADDER[m.region] or nil
end

-- The plan's block a move belongs to: cardio by its dose; a pose its family's (slice 21: the backbend, balance and
-- the arm balances yoga, the splits, the pike, twists and ankles mobility); off the families the timed poses of the
-- yoga group and the balance moves are yoga, the other stretches and mobility drills mobility; every rep or hold move
-- is strength, and so is every other timed move: one its load or tempo makes work (a carry, the good morning) and a
-- drill of a strength group that is timed because no set of it ends at the form (the bird dog, the chops).
function ns.Block(moveId)
	local m = ns.Library[moveId]
	if m.dose == "cardio" then return "cardio" end
	local family = m.ladder and ns.Ladders[m.ladder].block
	if family then return family end
	if m.dose == "flow" then
		if m.group == "yoga" or m.pattern == "balance" then return "yoga" end
		if m.pattern == "mobility" then return "mobility" end
	end
	return "strength"
end

-- Whether a move is a standing leg move (slice 48, docs/computer-games.md §5: a match pump after a long sit opens on
-- one, or on the day plan's cardio): a rep move of the squat, lunge or calf pattern that is no floor move, carries no
-- strain, skill or tendon flag and takes no typed weight (ns.Loaded): the squats, split squats, lunges and calf raises
-- a player does on their feet, the ball goblet squat among them (a ball's weight is fixed and light, never typed);
-- never a hinge, whose lying bridges carry no floor flag, a hold, a jump, a pistol, the heel drops, a dumbbell,
-- kettlebell or barbell squat or a vest's squat. contract/library.json carries it per move (`standing`), the phones'
-- ports read it there.
local STANDING = { squat = true, lunge = true, calf = true }

function ns.StandingLeg(moveId)
	local m = ns.Library[moveId]
	return m ~= nil and m.dose == "rep" and STANDING[m.pattern] == true and not ns.Flagged(moveId, "floor")
		and not ns.Flagged(moveId, "strain") and not ns.Flagged(moveId, "skill") and not ns.Flagged(moveId, "tendon")
		and not ns.Loaded(moveId)
end

-- Steps and colours (slice 20) --------------------------------------------------------------------------------

-- A rung's step on the one scale (1-12), and its colour: Enum.ItemQuality, Poor 0 ("Starter") to Legendary 5.
function ns.Step(ladder, rung)
	local r = ns.Ladders[ladder] and ns.Ladders[ladder].rungs[rung]
	return r and r.step
end

function ns.Quality(ladder, rung)
	local step = ns.Step(ladder, rung)
	return step and C.quality_by_step[step]
end

-- The highest rung of a ladder at or under a step on the one scale, nil when its first rung is higher.
function ns.RungAtStep(ladder, step)
	local found
	for r, w in ipairs(ns.Ladders[ladder].rungs) do
		if w.step <= step then found = r end
	end
	return found
end

-- The step a set trains at: its rung on `scale`, the ladder it scales by (ns.ScaleLadder; cardio for a cardio row),
-- clamped to that ladder's rungs; 0 for a move no ladder places.
function ns.StepOn(scale, rung)
	return scale and rung > 0 and ns.Step(scale, math.min(rung, #ns.Ladders[scale].rungs)) or 0
end

-- A move's step on the one scale and its colour: its rung's on its ladder, or on the ladder its region scales by
-- (ns.ScaleLadder); nil for a move no ladder places (the stretches and poses until slice 21 gives them ladders).
function ns.MoveStep(moveId)
	local m = ns.Library[moveId]
	local scale = m and ns.ScaleLadder(m)
	return scale and m.rung and ns.Step(scale, m.rung)
end

function ns.MoveQuality(moveId)
	local step = ns.MoveStep(moveId)
	return step and C.quality_by_step[step]
end

-- The top rung that holds a move: 0 for a ladder no move sits on, which is never offered and earns no colour (none
-- since slice 22 filled the handstand, the L-sit, the planche and power: tools/lint.lua holds every rung to its
-- anchor).
function ns.HeldTop(ladder)
	return held[ladder] or 0
end

-- The highest rung a ladder ever stood on: unlocks are kept (slice 20), so a rung down leaves its colour earned. The
-- rungs the sheet marks reached (Stats.lua `sheet.rungs`: every climb not undone, the game's and the app's, the one
-- record of them; slice 26) that still count (ns.ReachCounts), the rung the ladder started on and its rung now, from
-- its rows, its placement or, on a pose family, its first pose held (its first step is the player's from then, as a
-- strength ladder's start is); 0 for a ladder that holds no move or has none of them yet (the pull-up and the side,
-- whose first rung is above the start, until their first max set). An orange or gold rung counts once earned
-- (2026-10-09: the climb only opens it, ns.EarnFloor; ns.EarnCounts), so the rung now counts only from the highest one
-- under it that is not such a rung waiting for its set.
function ns.BestRung(state, ladder)
	if ns.HeldTop(ladder) == 0 then return 0 end
	local lad = ladder == ns.CARDIO and state.cardio or (state.ladders and state.ladders[ladder])
	-- its rung now once a row set it or a first max set placed it (a placement inside the window writes no rung row)
	local now = lad and (lad.rung or (lad.placedTs or POSE[ladder] and lad.lastTs) and ns.Rung(state, ladder)) or 0
	while now > 0 and ns.TopTier(ladder, now) and not ns.EarnCounts(state, ladder, now) do now = now - 1 end
	local best = math.max(now, lad and lad.start or 0)
	for r = #ns.Ladders[ladder].rungs, best + 1, -1 do
		if ns.TopTier(ladder, r) and ns.EarnCounts(state, ladder, r) or not ns.TopTier(ladder, r)
			and ns.ReachCounts(state, ladder, r) then
			return r
		end
	end
	return best
end

-- A rung's key in the sheet's map of the rungs reached (Stats.lua `sheet.rungs`): ladder × 100 + rung since the save's
-- schema 2 (ladder × 10 + rung before it, which a ladder of ten rungs would have run into the next ladder's keys).
function ns.RungKey(ladder, rung)
	return ladder * 100 + rung
end

-- When a rung was first reached: its date (true where a sheet from before 2026-10-03 had no date for it), nil for the
-- start rung and the rungs above the best.
function ns.RungReached(state, ladder, rung)
	return state.sheet and state.sheet.rungs[ns.RungKey(ladder, rung)]
end

-- Whether a rung reached still counts toward the ladder's best (ns.BestRung, and so its colour, the colour titles and
-- the Feats): always, but on a ladder the player started over (DB.lua clearRow's `restartTs`) only a reach dated after
-- that (an undated `true` from before 2026-10-03 counts as before it), until a climb reaches the rung again (Stats.lua
-- markReached dates it anew).
function ns.ReachCounts(state, ladder, rung)
	local at = ns.RungReached(state, ladder, rung)
	local lad = ladder ~= ns.CARDIO and state.ladders and state.ladders[ladder]
	local since = lad and lad.restartTs
	return at ~= nil and (not since or type(at) == "number" and at > since)
end

-- When an orange or gold rung was earned (2026-10-09; Stats.lua `sheet.earned`: the set at its floor, ns.EarnFloor,
-- its climb only opening it), nil before; and whether that still counts (ns.BestRung, its colour, its Feat of Strength
-- and its top title): on a ladder the player started over only an earning dated after that, as ns.ReachCounts.
function ns.RungEarned(state, ladder, rung)
	return state.sheet and state.sheet.earned and state.sheet.earned[ns.RungKey(ladder, rung)]
end

function ns.EarnCounts(state, ladder, rung)
	local at = ns.RungEarned(state, ladder, rung)
	local lad = ladder ~= ns.CARDIO and state.ladders and state.ladders[ladder]
	local since = lad and lad.restartTs
	return at ~= nil and (not since or at > since)
end

-- The rung a ladder started on: its start row's, or the lower one a Trial placement gave (DB.lua); without one, the
-- rung ns.Rung gives before any row (a pose family's first step, the first rung of a ladder that begins above the start
-- step, where a first max set places the player). The rungs up to it are the player's from the start (the sheet
-- summary's ear).
function ns.StartedOn(state, ladder)
	local lad = ladder == ns.CARDIO and state.cardio or (state.ladders and state.ladders[ladder])
	return lad and lad.start or ns.StartRung(ladder)
end

-- A ladder's colour earned (its best rung's quality) and the colour its top rung with a move holds, the collection's
-- and the colour titles' one reading of them; -1 for a ladder that holds no move, below every colour.
function ns.LadderColour(state, ladder)
	local best = ns.BestRung(state, ladder)
	return best > 0 and ns.Quality(ladder, best) or -1
end

function ns.LadderTop(ladder)
	local top = ns.HeldTop(ladder)
	return top > 0 and ns.Quality(ladder, top) or -1
end

-- The player's focus, the day plan's shares it picks (C.focus_share): the latest focus row's, Balanced before any.
function ns.Focus(state)
	return E.focusKeys[state and state.focus or 0] or "balanced"
end

-- Filters ---------------------------------------------------------------------------------------------------------

-- What the player has and how they train (the profile), and the pain flags (the fold), in one table the engine reads.
function ns.FilterContext(profile, state)
	profile = profile or ns.defaults
	return { gear = profile.gear or {}, floor = profile.floor == true, space = profile.space == true,
		quiet = profile.quiet == true, noJump = profile.noJump == true, noStrain = profile.noStrain == true,
		noHeadDown = profile.noHeadDown == true, exclude = profile.exclude or {}, include = profile.include or {},
		favourites = profile.favourites or {}, pain = state and state.pain or {} }
end

-- The arms rest (slice 48, docs/computer-games.md §5: a match pump while a game marked `aim` in contract/links.json
-- still runs): the filter context `f` with the wrist, elbow and shoulder pain flags as if set, and every move of the
-- push and pull regions left out as those flags leave a move out (`arms`, ns.Passes: of those regions the band
-- pull-apart, the doorframe isometric row and the Y-T-W raises alone carry no arm tag), so the pump keeps the legs, the
-- core, cardio and the stretches; a move of another region that holds a weight in the hands, swings, strikes, hangs or
-- bears on the hands leaves through the flags (Library.lua tags it: the carries, the goblet squats, the deadlifts with
-- a weight, the swing, the chops, the squat punch, the planks and crawls on the hands, the bird dog, the spider lunge,
-- the world's greatest stretch, the cobra, the burpees, the L-sits), while the forearm planks and the tuck plow raise,
-- a palm press lying down, carry none. A copy: the fold's own flags (state.pain) never change, and the setup alone
-- (setupOf) holds neither, so a step's stages, its checks and how a climb is measured stay the player's; inside the
-- compose it acts everywhere a pain flag acts (the candidates, the rung move, the Trial, Swap's options).
local ARMS_JOINTS = { E.joint.wrist, E.joint.elbow, E.joint.shoulder }

-- The regions the arms rest leaves out.
local ARMS_REGIONS = { push = true, pull = true }

function ns.ArmsRest(f)
	local rest = {}
	for k, v in pairs(f) do rest[k] = v end
	rest.pain = {}
	for joint, v in pairs(f.pain) do rest.pain[joint] = v end
	for _, joint in ipairs(ARMS_JOINTS) do rest.pain[joint] = rest.pain[joint] or true end
	rest.arms = true
	return rest
end

-- Whether the arms rest leaves a move out on its own (ns.ArmsRest's flags and regions, as ns.Passes reads them): a
-- wrist, elbow or shoulder pain tag, or the push or pull region; never the step jacks (ns.FLOOR_MOVE), nor a move this
-- version does not know. Such an unfinished move waits while a match pump keeps the rest (slice 61: the compose's
-- riderFits reads ns.ArmsRest's filters), and no owed move rides as one (Engine.lua ridesAs); the phones' twin is
-- loadsArms.
function ns.ArmsHeld(moveId)
	local m = ns.Library[moveId]
	if not m or moveId == ns.FLOOR_MOVE then return false end
	if ARMS_REGIONS[m.region] then return true end
	for _, tag in ipairs(m.pain) do
		local joint = E.joint[tag]
		for _, arm in ipairs(ARMS_JOINTS) do
			if joint == arm then return true end
		end
	end
	return false
end

local FLAG_RULE = { floor = "floor", space = "space", loud = "quiet", impact = "noJump", strain = "noStrain",
	head_down = "noHeadDown" }

-- Whether a set of gear ({ [piece] = true }) meets one entry of a move's needs: an item, or a list of which any one
-- does (Library.lua's `needs`); the setup window's equipment pictures ask it too (Panel.lua, 2026-10-15).
local function hasNeed(gear, need)
	if type(need) == "table" then
		for _, one in ipairs(need) do
			if gear[one] then return true end
		end
		return false
	end
	return gear[need] == true
end
ns.HasNeed = hasNeed

-- Whether a move passes the hard filters, and the first reason it does not: equipment (with "gear" the need that is
-- missing, an item or an any-of list), the exclusion list and the opt-in moves the player has not checked (the
-- headstand, slice 21: unchecked on the Exercises page until checked), floor and space, quiet mode, the Body switches
-- and the pain flags, the arms rest's push and pull regions among them (ns.ArmsRest). The step jacks, the floor, always
-- pass.
function ns.Passes(moveId, f)
	local m = ns.Library[moveId]
	if not m then return false, "unknown" end
	if moveId == ns.FLOOR_MOVE then return true end
	if f.exclude[moveId] or f.exclude[m.group] then return false, "excluded" end
	if m.optIn and not f.include[moveId] then return false, "excluded" end
	for _, need in ipairs(m.needs or {}) do
		if not hasNeed(f.gear, need) then return false, "gear", need end
	end
	for _, flag in ipairs(m.flags) do
		local rule = FLAG_RULE[flag]
		if rule == "floor" or rule == "space" then
			if not f[rule] then return false, rule end
		elseif rule and f[rule] then
			return false, rule
		end
	end
	for _, tag in ipairs(m.pain) do
		local joint = E.joint[tag]
		if joint and f.pain[joint] then return false, "pain" end
	end
	if f.arms and ARMS_REGIONS[m.region] then return false, "pain" end
	return true
end

-- The public test: a move id this version does not know (a newer addon's, through the mirror) is never offered.
function ns.Allowed(moveId, state, profile)
	return (ns.Passes(moveId, ns.FilterContext(profile, state)))
end

-- The move that takes this one's place (Library.lua `upgrade`, 2026-10-09: "Like rows dont need a backpack when you
-- have dumbbells"): the first of its upgrades in `pool` (a set of ids; a pump's candidates, Engine.lua), else nil.
function ns.Better(moveId, pool)
	local m = ns.Library[moveId]
	for _, id in ipairs(m and m.upgrade or {}) do
		if pool[id] then return id end
	end
	return nil
end

-- The first of a move's upgrades the filters pass (Swap's backup, the Book's and the tooltip's "Set aside: <move>
-- takes its place"), else nil.
function ns.BetterOpen(moveId, f)
	local m = ns.Library[moveId]
	for _, id in ipairs(m and m.upgrade or {}) do
		if ns.Passes(id, f) then return id end
	end
	return nil
end

-- What lifts each of ns.Passes' reasons in a filter context: the two that widen the pool go on, the four that narrow
-- it go off.
local LIFT = { floor = true, space = true, quiet = false, noJump = false, noStrain = false, noHeadDown = false }

-- Every filter that shuts a move out, in ns.Passes' order, each as { reason, need? }: ns.Passes is asked again with
-- each reason lifted in a copy of the context, so the filter rule stays in one place. Empty for a move that passes.
function ns.Shut(moveId, f)
	local open, why = {}, {}
	for k, v in pairs(f) do open[k] = v end
	open.gear = {}
	for k, v in pairs(f.gear) do open.gear[k] = v end
	for _ = 1, 16 do -- more reasons than one move can meet: six switches, the list, the pain flags, a few needs
		local ok, reason, need = ns.Passes(moveId, open)
		if ok then break end
		why[#why + 1] = { reason = reason, need = need }
		if reason == "gear" then
			open.gear[type(need) == "table" and need[1] or need] = true
		elseif reason == "excluded" then
			open.exclude, open.include = {}, { [moveId] = true }
		elseif reason == "pain" then
			open.pain = {}
		elseif LIFT[reason] ~= nil then
			open[reason] = LIFT[reason]
		else
			break -- a move this version does not know: nothing lifts it
		end
	end
	return why
end

-- The filters that shut a move out (ns.Shut), for a line that says what would open it: nil when it passes, and where a
-- pain flag is among them (a flag goes when the pain has gone, never to open a move).
function ns.Opener(moveId, f)
	local why = ns.Shut(moveId, f)
	for _, w in ipairs(why) do
		if w.reason == "pain" then return nil end
	end
	return #why > 0 and why or nil
end

-- The first of the setup's rules among the reasons ns.Shut lists (`why`; the exclusions, the opt-in moves and the pain
-- flags aside), as Your setup names it: { kind = "gear", key } a piece of equipment by its key, { kind = "off", key } a
-- switch off that lets the move in (floor, space), { kind = "on", key } a switch on that leaves it out (quiet, nojump,
-- nostrain, noheaddown); nil for none: a refused Start's `gap` (tools/apppump.lua A.start), the Book's setup lines and
-- the Talents tab's refusal line (Tree.lua Tree.Refusal).
local SWITCH_GAP = { floor = { "off", "floor" }, space = { "off", "space" }, quiet = { "on", "quiet" },
	noJump = { "on", "nojump" }, noStrain = { "on", "nostrain" }, noHeadDown = { "on", "noheaddown" } }

function ns.SetupGap(why)
	for _, w in ipairs(why) do
		if w.reason == "gear" then return { kind = "gear", key = type(w.need) == "table" and w.need[1] or w.need } end
		local switch = SWITCH_GAP[w.reason]
		if switch then return { kind = switch[1], key = switch[2] } end
	end
	return nil
end

-- Rungs -------------------------------------------------------------------------------------------------------------

-- The rung a ladder starts on: its highest at or under the experience's step (none 2, some 3, training 4), so every
-- ladder starts at the same class of feat (the core at its step 3 for training, where its next rung is step 6); rung 1
-- on a ladder that begins above it (the pull-up and the side, which a first max set at the top still places up at once)
-- and on a pose ladder (the experience asks about strength training).
function ns.StartRung(ladder, experience)
	if POSE[ladder] then return 1 end
	return ns.RungAtStep(ladder, C.start_rung[experience or "none"] or C.start_rung.none) or 1
end

-- The ladder's rung, from the rung rows; before any, the start rung of the default experience (none).
function ns.Rung(state, ladder)
	if ladder == ns.CARDIO then
		return state.cardio and state.cardio.rung or ns.StartRung(ladder)
	end
	local l = state.ladders and state.ladders[ladder]
	return l and l.rung or ns.StartRung(ladder)
end

-- The move that stands for a rung: its anchor, else the first twin the filters allow; when the whole chain is out
-- the rungs below are tried. `stepped` (a test, optional): the moves the player stepped down from (ns.SteppedDown),
-- which give way to the next move of the same chain the filters allow; none left, and the rung has no move
-- (2026-10-05). Returns the move and the rung it belongs to, or nil.
function ns.RungMove(ladder, rung, f, stepped)
	for r = math.min(rung, #ns.Ladders[ladder].rungs), 1, -1 do
		local passed = false
		for _, id in ipairs(ns.RungChain(ladder, r)) do
			if ns.Passes(id, f) then
				if not (stepped and stepped(id)) then return id, r end
				passed = true
			end
		end
		if passed then return nil end
	end
	return nil
end

-- A move of the cardio rung itself the filters allow: its anchor or a twin on that rung, else the lowest id among the
-- rung's other moves; nil when none passes. A cardio rung is open when it has one, and its unlock names it (the chain's
-- twins can stand on a lower rung, which the rung opens nothing of). `steady`: never a move that is only ever an
-- all-out window (endOnly; slice 36's Start of a timed rung).
local function cardioRungMove(rung, f, steady)
	local function fits(id, m)
		return m.rung == rung and not (steady and m.endOnly) and ns.Passes(id, f)
	end
	for _, id in ipairs(ns.RungChain(ns.CARDIO, rung)) do
		if fits(id, ns.Library[id]) then return id end
	end
	local best
	for id, m in pairs(ns.Library) do
		if m.ladder == ns.CARDIO and fits(id, m) and (not best or id < best) then best = id end
	end
	return best
end

-- How many rungs above the player's a ladder's moves reach: rung_reach on a strength ladder, none in a pose family
-- (slice 21: no pose above the player's step is offered). The engine's picks and riders and the sheet read it here.
function ns.Reach(ladder)
	return POSE[ladder] and 0 or C.rung_reach
end

-- A move above the player's reach (ns.Reach) on the ladder it scales by (cardio keeps its own rung rule). No pump
-- offers it past the rung rule and nothing starts it on its own: the sheet's flyouts grey it, and its tooltip and a
-- step's card show it as a step ahead, with its figure, How to and stop line (slice 37; David, 2026-10-14: "you can
-- still watch future excercises").
function ns.AboveReach(state, moveId)
	local m = ns.Library[moveId]
	local scale = ns.ScaleLadder(m)
	return scale ~= nil and scale ~= ns.CARDIO and (m.rung or 1) > ns.Rung(state, scale) + ns.Reach(scale)
end

-- A rung can be climbed when a move of its own level stands for it, its anchor or a twin of its rung (else the ladder
-- tops out with this setup: no floor, no bar, no sofa, a move the player left out); a stand-in from below never
-- climbs it, so no rung's colour, Legendary least of all, comes from a lower move. A cardio rung can be climbed when a
-- move of its own passes. With the player's `state`, the steps of other ladders the rung needs (ns.RungNeed) are done
-- too, and the move of its own level is open (ns.MoveOpen: its checks done, 2026-10-16: a bar's tuck front lever waits
-- for 8 pull-ups where the table's archer row climbs); without one (the Exercises page's warnings, which name what the
-- setup shuts) the setup alone counts.
local function closedIn(state, f)
	return state and function(id) return not ns.MoveOpen(state, id, f) end or nil
end

function ns.CanRungUp(ladder, rung, f, state)
	if rung >= #ns.Ladders[ladder].rungs then return false end
	if state and ns.RungNeed(state, ladder, rung + 1) then return false end
	if ladder == ns.CARDIO then return cardioRungMove(rung + 1, f) ~= nil end
	local up = ns.RungMove(ladder, rung + 1, f, closedIn(state, f))
	return up ~= nil and ns.Library[up].rung == rung + 1
end

-- Whether the ladder can climb from this rung with this setup (and, with the `state`, its needs: ns.CanRungUp): the
-- rung above opens, and a move of this rung's own level stands for it (on a pose family its main pose, ns.MainPose,
-- whose days the climb's floor counts: ns.PoseNeeds): a stand-in from below never climbs the rung it stands
-- for (slice 21's poses; 2026-10-09 the strength ladders, whose stand-in counts no day at the top, DB.lua
-- anchorAttempt).
function ns.CanClimb(ladder, rung, f, state)
	if not ns.CanRungUp(ladder, rung, f, state) then return false end
	if ladder == ns.CARDIO then return true end
	local id = ns.RungMove(ladder, rung, f, closedIn(state, f))
	return id ~= nil and ns.Library[id].rung == rung and (not POSE[ladder] or ns.MainPose(ladder, rung, id))
end

-- The move an unlock names: the cardio rung's own move, else the move that stands for the rung (ns.RungMove); with the
-- player's `state` (the rung-up's toast and the sheet summary's lu) the first of the rung's own level that is open
-- (ns.MoveOpen), the one a climb found (ns.CanRungUp), else as without it (2026-10-24: a jumper's climb into power step
-- 6 named the explosive push-up its check holds, where the tuck jump stands); without it (ns.ToEarn, the tree's and the
-- Book's steps) the setup's; nil where the setup leaves none.
function ns.UnlockMove(ladder, rung, f, state)
	if ladder == ns.CARDIO then return cardioRungMove(rung, f) end
	local open = state and ns.RungMove(ladder, rung, f, closedIn(state, f))
	if open and ns.Library[open].rung == rung then return open end
	return (ns.RungMove(ladder, rung, f))
end

-- Whether the player has tried a step or one above it: a move of the ladder at that rung or higher holds a record, or
-- had one cleared.
function ns.StepTried(state, l, rung)
	for id, m in pairs(ns.Library) do
		if m.ladder == l and m.rung >= rung and (ns.Record(state, id) ~= nil or state.cleared[id]) then return true end
	end
	return false
end

local CLIMB = { [ns.RungCause.up] = true, [ns.RungCause.plateau] = true, [ns.RungCause.placementHigh] = true }

-- The filter context every setup at once allows (every piece of equipment, floor and space, every opt-in move, no
-- switch, exclusion or pain flag): a move passes it when some setup passes it. Built once (tools/lint.lua's stage
-- checks read it too).
local everySetup

function ns.EverySetup()
	if not everySetup then
		local gear, include = {}, {}
		for _, key in ipairs(ns.Gear) do gear[key] = true end
		for id, m in pairs(ns.Library) do
			if m.optIn then include[id] = true end
		end
		everySetup = ns.FilterContext({ gear = gear, floor = true, space = true, include = include }, nil)
	end
	return everySetup
end

-- Whether rung row `r` (a strength ladder's latest the fold applied, ns.LatestRung) is a climb today's rules would not
-- make (2026-10-24, after a tuck jump climbed power to step 7, whose clap push-up waits for 12 feet-up push-ups: David,
-- "why would that unlock?"): a climb (up, a plateau or a high placement) that put the ladder on the rung it stands on,
-- a step the player never tried (ns.StepTried) whose climb no open move of its own level, or another ladder's step it
-- needs, allows now with any setup (ns.CanRungUp with the state and ns.EverySetup: the setup of the climb is unknown,
-- a phone's and the computer's differ, and a setup or a pain flag changed since never takes a step back). The repairs
-- take it back as Correct a count takes a step change back: Events.lua Lc.UndoOldClimbs for the game's, the desktop
-- app's sweep for the apps' (tools/apppump.lua Engine:sweep).
function ns.HeldClimb(state, r)
	local l, rung = r.ladder, r.rung
	if r.kind ~= K.rung or not CLIMB[r.reps] or not ns.STRENGTH[l] or rung <= 1 or ns.Rung(state, l) ~= rung then
		return false
	end
	return not ns.CanRungUp(l, rung - 1, ns.EverySetup(), state) and not ns.StepTried(state, l, rung)
end

-- What would open a ladder that cannot climb from this rung with this setup (David, 2026-10-04: the top-out line names
-- what opens the next rung): on a strength or pose ladder the first anchor the filters shut, from the rung above the
-- move that stands for this one (this rung's own anchor when a stand-in from below holds it, else the next rung's; the
-- first rung's on a ladder the setup leaves no move, the pull-up without a bar), and every filter that shuts it
-- (ns.Shut); with the `state`, where the setup lets it climb (the cardio ladder too, whose rung the setup shuts with no
-- anchor to name), the next rung's anchor and the step of another ladder it waits for (ns.RungNeed) as { reason =
-- "need", move, value }: the move ns.NeedAt names for it with `f` and the record that does it (nil on a pose or cardio
-- step, which that ladder's climb does), or a move of either rung's own level that waits for its check (ns.MoveOpen,
-- 2026-10-16) and
-- that check the same way (ns.NeedAt with `f`). Nil while the ladder can climb, at the top of the rungs that hold a
-- move, and where a pain flag is among the reasons: a flag goes when the pain has gone, never to open a rung.
function ns.TopOut(ladder, rung, f, state)
	if ns.CanClimb(ladder, rung, f, state) then return nil end
	if ladder == ns.CARDIO then
		if not ns.CanRungUp(ladder, rung, f) then return nil end
	else
		local id = ns.RungMove(ladder, rung, f)
		for r = id and math.min(rung, ns.Library[id].rung) + 1 or 1, rung + 1 do
			local shut = ns.Anchor(ladder, r)
			if shut and not ns.Passes(shut, f) then
				local why = ns.Opener(shut, f)
				if not why then return nil end
				return shut, why
			end
		end
	end
	local entry
	if state then entry = select(3, ns.RungNeed(state, ladder, rung + 1)) end
	if entry then
		-- named as a card names it (ns.NeedAt with `f`: an `any` group's entry the setup offers, the move of a step the
		-- setup has; 2026-10-17, slice 42's review: with a pole and no bar the side was told of a Pull-up record of 8)
		local move, value = ns.NeedAt(entry, f)
		return ns.Anchor(ladder, rung + 1), { { reason = "need", move = move, value = value } }
	end
	if state and ladder ~= ns.CARDIO then
		for _, r in ipairs({ rung + 1, rung }) do
			for _, id in ipairs(ns.RungChain(ladder, r)) do
				local open, why = ns.MoveOpen(state, id, f)
				if ns.Library[id].rung == r and ns.Passes(id, f) and not open and why.kind == "check" then
					local move, count = ns.NeedAt(why.need, f)
					return id, { { reason = "need", move = move, value = count } }
				end
			end
		end
	end
	return nil
end

-- The lowest rung from the player's at which the setup stops a ladder (the Exercises page's warnings name it before
-- the player reaches it; the setup alone, so no rung's needs), with what would open it (ns.TopOut); nil for a ladder
-- the setup lets climb to its top.
function ns.TopOutAhead(state, ladder, f)
	for rung = ns.Rung(state, ladder), ns.HeldTop(ladder) - 1 do
		if not ns.CanClimb(ladder, rung, f) then
			local shut, why = ns.TopOut(ladder, rung, f)
			if not shut then return nil end
			return rung, shut, why
		end
	end
	return nil
end

-- A rung's window for a move: lo, hi, and whether they are the move's own (`alt`) rather than the rung's.
function ns.Window(ladder, rung, moveId)
	local w = ns.Ladders[ladder] and ns.Ladders[ladder].rungs[rung]
	if not w or not w.lo then return nil end
	local alt = w.alt and w.alt[moveId]
	return (alt or w).lo, (alt or w).hi, alt ~= nil
end

-- Whether a rung's window counts in a move's own unit: the window is the move's own (`own`, ns.Window's third value),
-- or the move is counted as `standing`, the move that stands for the rung, is (reps beside reps, seconds beside
-- seconds).
function ns.OwnUnit(moveId, standing, own)
	return own or (standing ~= nil and ns.Library[standing].dose == ns.Library[moveId].dose)
end

-- The climb's rules: its days, the path, the checks and the families that open (slice 22; the path since 2026-10-16,
-- slice 42) ---------------------------------------------------------------------------------------------------------

local Q = E.quality
local EMPTY = {}

-- The setup in a filter context: `f` with no pain flag and no arms rest (ns.ArmsRest; a flag goes when the pain has
-- gone, never to open a step, to skip a stage or to change how a step is measured).
local function setupOf(f)
	if not next(f.pain) and not f.arms then return f end
	local setup = {}
	for k, v in pairs(f) do setup[k] = v end
	setup.pain, setup.arms = EMPTY, nil
	return setup
end

-- A rung up waits while a pain flag is set on a joint the ladder's anchors at this rung or the next one load (the
-- plan: "with no pain flag on the ladder"), even when a twin without the tag could stand in. The cardio ladder's steps
-- stand on their forms, with no anchor to wait on: there a rung up waits while the flags in the filter context `f`
-- leave this step no form its climb counts (a countable one or a steady one, as the pumps offer the step) or the next
-- step no form (ns.CanRungUp) where the setup alone (setupOf) leaves one; `f` is read on the cardio ladder alone.
function ns.PainOnLadder(state, ladder, rung, f)
	if ladder == ns.CARDIO then
		local setup = setupOf(f)
		local function here(g) return #ns.CountForms(rung, g) > 0 or cardioRungMove(rung, g, true) ~= nil end
		local function above(g) return cardioRungMove(rung + 1, g) ~= nil end
		return here(setup) and not here(f) or above(setup) and not above(f)
	end
	for _, r in ipairs({ rung, rung + 1 }) do
		local id = ns.Anchor(ladder, r)
		for _, tag in ipairs(id and ns.Library[id].pain or {}) do
			if state.pain[E.joint[tag]] then return true end
		end
	end
	return false
end

-- Whether a rung is orange or gold (Legendary or Artifact): the rungs only days climb to and a set earns; false past a
-- ladder's top.
function ns.TopTier(ladder, rung)
	return (ns.Quality(ladder, rung) or Q.starter) >= Q.legendary
end

-- The game days at the window's top a climb from `rung` takes: unlock_days, and unlock_days_tendon where the rung
-- above is orange or gold and its anchor is tendon-heavy (the assisted one-arm pull-up and the one-arm pull-up, the
-- planche, the full Nordic, the front lever, the human flag, the muscle-up: connective tissue adapts more slowly than
-- muscle). With the player's `state`, a climb back to a rung they have done already takes one (ns.Returning).
function ns.UnlockDays(ladder, rung, state)
	if state and ns.Returning(state, ladder, rung) then return C.return_days end
	local up = ns.Anchor(ladder, rung + 1)
	if up and ns.TopTier(ladder, rung + 1) and ns.Flagged(up, "tendon") then
		return C.unlock_days_tendon
	end
	return C.unlock_days
end

-- Whether the climb from `rung` comes only by the days at the window's top (ns.UnlockDays): never by a placement at
-- the top nor a plateau, to an orange or gold rung (slice 22) or, since 2026-10-10, to one whose anchor is flagged
-- skill or tendon (a max set at the top, or weeks near it, says nothing of a skill or a tendon ready for it; David:
-- "You should be strong enough to easily handle the next rung").
function ns.DaysOnly(ladder, rung)
	if ns.TopTier(ladder, rung + 1) then return true end
	local up = ns.Anchor(ladder, rung + 1)
	return up ~= nil and (ns.Flagged(up, "skill") or ns.Flagged(up, "tendon"))
end

-- The clean days a counted cardio rung's climb takes (C.cardio_clean_days by the step it climbs from, 2026-10-16:
-- game days with a window of the step's own forms that ran its 30 s saved clean, each a new session by the night rule,
-- DB.lua cardioMove); nil for a rung that names none (steps 1-4, which climb by full moves, and the top). With the
-- player's `state`, a climb back to a step they held before takes return_days (slice 53, ns.Returning), as a strength
-- ladder's does (ns.UnlockDays).
function ns.CleanDays(rung, state)
	local step = ns.Step(ns.CARDIO, rung)
	local need = step and C.cardio_clean_days[step] or nil
	if need and state and ns.Returning(state, ns.CARDIO, rung) then return C.return_days end
	return need
end

-- Whether a cardio rung climbs by clean days: a countable form of its own the setup allows (ns.CountForms with the
-- setup alone, setupOf) and its clean days named; else by full moves (slice 52; ready days, the pace answers, until
-- then), as steps 1-4 do and any rung whose countable forms the setup shuts every one of. A pain flag never changes how
-- a step is measured (2026-10-17, slice 42's review: with the knee and Achilles flagged on cardio step 5 the ready days
-- of then opened step 6 on one clean day of three); it may hold the climb.
function ns.CountedRung(rung, f)
	return ns.CleanDays(rung) ~= nil and #ns.CountForms(rung, setupOf(f)) > 0
end

-- The week's omens (slice 54, 2026-10-23), the wheel of five every player turns through, one a week: each the road of
-- ns.ROADS it names, first in the cardio picks' fair draw that week (Engine.lua roadClocks), and the key of its title
-- (`today.week.<key>`). It names no cue on a move (a cue a road's every move said, from slice 54 until 2026-11-03:
-- Rapha, after Fist's "Two punches, then a knee." asked shadow boxing for a knee strike it lacks; four of the five
-- asked a motion some of their road's moves never make).
ns.OMENS = {
	{ key = "hare", road = "run" },
	{ key = "fist", road = "strike" },
	{ key = "frog", road = "jump" },
	{ key = "bear", road = "floor" },
	{ key = "oak", road = "standing" },
}

-- The omen of a road; nil for gear, which none names.
local function omenOf(road)
	for _, omen in ipairs(ns.OMENS) do
		if omen.road == road then return omen end
	end
	return nil
end

-- The week's omen at `now` for the setup in `f` on cardio rung `rung` (its pain flags and the arms rest left out:
-- setupOf, so a flag never turns the wheel in the week): the wheel's turn by the vault's week (ns.WeekOf: the game's
-- region reset, the apps' Monday at the day's turn; the two clocks number the weeks alike, tools/lint.lua, each turning
-- at its own week's turn, so the apps name the next omen from their Monday to the region's reset), passed on to the
-- next omen while the setup's steady pool on the rung (ns.SteadyForms) holds no form of its road (the jump and floor
-- roads before cardio step 5, the jump road under No jumping), so the week's road is one the player's steady moves
-- run on. A pool of equipment forms alone names the omen of the step jacks' road, the floor every filter falls back
-- to (ns.FloorSlot).
function ns.Omen(now, f, rung)
	local roads = {}
	for _, id in ipairs(ns.SteadyForms(rung, setupOf(f))) do roads[ns.Library[id].road] = true end
	local n = #ns.OMENS
	local first = ns.WeekOf(now) % n
	for i = 0, n - 1 do
		local omen = ns.OMENS[(first + i) % n + 1]
		if roads[omen.road] then return omen end
	end
	return omenOf(ns.Library[ns.FLOOR_MOVE].road)
end

-- The count that earns an orange or gold rung (2026-10-09, David's goal: orange "only once you are ready"). The climb
-- only opens it; the player's first fresh set of a move of the rung's own level at the floor of its window earns it
-- (DB.lua earnMark): its colour (ns.BestRung), the Legendary or Artifact moment, the Feat of Strength and the top
-- title. The floor of `moveId`'s own window on `rung` for a strength move of the rung's own chain at its level,
-- counted in its own unit (ns.OwnUnit); nil for any other move, a pose (its mark held whole and answered Perfect earns
-- it, DB.lua markAttempt), a countable cardio form (a fresh clean window of the rung's own form earns it, DB.lua
-- cardioMove, since 2026-10-16: its count earned it at its window's floor until then) and a rung under orange.
function ns.EarnFloor(ladder, rung, moveId)
	local m = ns.Library[moveId]
	if not (m and m.ladder == ladder and m.rung == rung and ns.TopTier(ladder, rung)) then return nil end
	if not (STRENGTH[ladder] and ns.InChain(ladder, rung, moveId)) then return nil end
	local lo, _, own = ns.Window(ladder, rung, moveId)
	if lo and ns.OwnUnit(moveId, ns.Anchor(ladder, rung), own) then return lo end
	return nil
end

-- An orange or gold rung the ladder stands on that no set has earned yet (ns.EarnFloor): the rung, the move standing
-- for it with filter context `f` (ns.UnlockMove) and the count of one set of it that earns it (nil on a pose family,
-- whose mark answered Perfect earns it, on the cardio ladder, whose fresh clean window of the rung's own form does, and
-- where the setup leaves no move of the rung's level); nil while the rung now is earned or under orange.
function ns.ToEarn(state, ladder, f)
	if ns.HeldTop(ladder) == 0 then return nil end
	local rung = ns.Rung(state, ladder)
	if rung <= ns.BestRung(state, ladder) or not ns.TopTier(ladder, rung) then return nil end
	local move = ns.UnlockMove(ladder, rung, f)
	return rung, move, move and ns.EarnFloor(ladder, rung, move)
end

-- Indexes built once from the ladders and the library (2026-10-16, slice 42): the stage of a path each move is in
-- (tools/lint.lua: one at most), and the closest answer a need or a check asks of a pose (its `mark`).
local stageOf, goalMarks = {}, {}
local function nameIn(o)
	if o.any then
		for _, x in ipairs(o.any) do nameIn(x) end
	elseif o.move and POSE[o.ladder] then
		if o.mark and (goalMarks[o.move] or 0) < o.mark then goalMarks[o.move] = o.mark end
	end
end
for l, lad in pairs(ns.Ladders) do
	for _, o in ipairs(lad.opens or EMPTY) do nameIn(o) end
	for r, w in ipairs(lad.rungs) do
		for i, stage in ipairs(w.path or EMPTY) do
			for _, id in ipairs(stage.ids) do
				stageOf[id] = { ladder = l, rung = r, stage = i }
			end
		end
		for _, o in ipairs(w.needs or EMPTY) do nameIn(o) end
	end
end
for _, m in pairs(ns.Library) do
	for _, o in ipairs(m.checks or EMPTY) do nameIn(o) end
end

-- Whether a pose has a goal pose of its own (Locales: move.<key>.mark), the one "How close to the goal pose?" asks of
-- it: every pose of a stretching or yoga family held in one shape (slice 47, 2026-10-19: any goal pose of the player's
-- step counts the family's day), never one that names none nor the opt-in headstand (Library.lua `noGoal`, `optIn`;
-- tools/lint.lua holds the locale to it). A pose without one is held and never asked.
function ns.HasGoal(moveId)
	local m = ns.Library[moveId]
	return m ~= nil and POSE[m.ladder] == true and not m.noGoal and not m.optIn
end

-- The closest answer a pose's goal pose has had where the ask came (E.mark; the fold's marks, DB.lua markDays), 0
-- before one: what masters its step (ns.TopOnce) and does a check that names it (stepDone). A row without an answer
-- (one from before the ask, or a pose held where nothing asked) counts nowhere (DB.lua markDays; one from before the
-- ask counted toward its mark days as held whole until 2026-10-31).
function ns.MarkBest(state, moveId)
	local d = state.marks and state.marks[moveId]
	return d and d.best or 0
end

-- The answer a pose's goal pose still waits for, which its slot asks once a game day (2026-10-16, slice 42; Engine.lua
-- markSlot): its top (ns.MoveTop at its own step) until answered once, which can master the step, and the most a check
-- or a need names on it (a `mark`) until answered; the higher of them, nil when neither waits or it has no goal pose.
function ns.GoalAsk(state, moveId)
	local m = ns.Library[moveId]
	if not (m and ns.HasGoal(moveId)) then return nil end
	local best = ns.MarkBest(state, moveId)
	local want = math.max(ns.MoveTop(m.ladder, m.rung, moveId) or 0, goalMarks[moveId] or 0)
	return best < want and want or nil
end

-- The path ---------------------------------------------------------------------------------------------------------

-- A rung's path: the stages the climb from it waits for, in order (ns.Ladders' `path`); empty for none.
function ns.Path(ladder, rung)
	local w = ns.Ladders[ladder] and ns.Ladders[ladder].rungs[rung]
	return w and w.path or EMPTY
end

-- The game days at its top a stage takes: its own `days` (a cardio form's clean days), else unlock_days.
function ns.StageDays(stage)
	return stage.days or C.unlock_days
end

-- The stage of a path a move is in: its ladder, the rung whose climb waits for it and the stage's index; nil for a
-- move no path names.
function ns.StageOf(moveId)
	return stageOf[moveId]
end

-- The top a move reaches on the climb from `rung` of its ladder, the one reading of a move's top (2026-10-16): a
-- strength or hold move's own window's top (ns.EasierTop), a pose's least answer to its goal pose that counts there
-- (ns.MarkNeeds; none for a pose with no goal pose, ns.HasGoal), true for a countable cardio form (a clean window);
-- nil for a move with no top (a steady cardio form, a move of another ladder or of none).
function ns.MoveTop(ladder, rung, moveId)
	local m = ns.Library[moveId]
	if not (m and m.ladder == ladder) then return nil end
	if POSE[ladder] then return ns.HasGoal(moveId) and ns.MarkNeeds(ladder, rung) or nil end
	if ladder == ns.CARDIO then return m.count ~= nil or nil end
	return ns.EasierTop(moveId)
end

-- A strength or hold move's game days at its own window's top at a weight that climbs (the fold's topDays, DB.lua
-- topDay), and the set that counted the last; a record at the top from a baseline written before them (2026-10-16)
-- counts one, never one set before the move's form changed (DB.lua ns.BeforeForm, 2026-10-19).
local function strengthDays(state, moveId)
	local m = state.moves[moveId]
	if not m then return 0, nil end
	if m.topDays then return m.topDays, m.topDayTs end
	local top = ns.EasierTop(moveId)
	local best, ts = m.best, m.bestTs
	if ns.ClimbKg(moveId) then
		best, ts = m.topLoad and ns.LoadClimbs(moveId, m.topLoad) and m.topReps or nil, m.topTs
	end
	return (top and best and best >= top and not ns.BeforeForm(moveId, ts or 0)) and 1 or 0, nil
end
ns.TopDaysFrom = strengthDays -- DB.lua's fold starts a move's top days from it

-- The game days a move reached its top (ns.MoveTop) on the climb from `rung`, and the moment that counted the last
-- (nil where none is known): a strength or hold move's days at its own window's top (strengthDays), a pose's days its
-- goal pose was answered as close (ns.MarkDays), a countable form's clean days at the cardio rung the ladder stands on
-- now (the fold's cleanForms; none at another rung); 0 for a move with no top.
function ns.TopDays(state, ladder, rung, moveId)
	local top = ns.MoveTop(ladder, rung, moveId)
	if not top then return 0, nil end
	if POSE[ladder] then
		local d = state.marks and state.marks[moveId]
		local ts = d and (top >= E.mark.perfect and d.perfectTs or d.goodTs)
		return ns.MarkDays(state, moveId, top), ts
	end
	if ladder == ns.CARDIO then
		local c = state.cardio
		if not (c and ns.Rung(state, ns.CARDIO) == rung and c.cleanForms) then return 0, nil end
		return c.cleanForms[moveId] or 0, c.cleanFormTs and c.cleanFormTs[moveId]
	end
	return strengthDays(state, moveId)
end

-- Whether a move reached its top once, the test that opens the stage after it and masters its step: a strength or
-- hold move at its window's top once, a pose's goal pose answered where the ask came as close as its own step's climb
-- asks once (ns.MarkBest), a countable form in one clean window at any cardio rung (the fold's cleanEver).
function ns.TopOnce(state, moveId)
	local m = ns.Library[moveId]
	if not (m and m.ladder) then return false end
	if m.ladder == ns.CARDIO then
		return m.count ~= nil and state.cardio ~= nil and state.cardio.cleanEver ~= nil
			and state.cardio.cleanEver[moveId] ~= nil
	end
	if POSE[m.ladder] then
		local top = ns.MoveTop(m.ladder, m.rung, moveId)
		return top ~= nil and ns.MarkBest(state, moveId) >= top
	end
	return (ns.TopDays(state, m.ladder, m.rung, moveId)) >= 1
end

-- Whether the setup `f` shuts every move of a stage (ns.Passes with no pain flag), which skips it: the equipment,
-- Floor moves, Room to move, quiet mode, the Body switches and the player's own list (David, 2026-10-16: a setup
-- that offers none of them leaves the climb by its main move, as ns.ProofShut did from 2026-10-10).
local function stageShut(stage, f)
	local setup = setupOf(f)
	for _, id in ipairs(stage.ids) do
		if ns.Passes(id, setup) then return false end
	end
	return true
end

-- Whether stage `stage` of the climb from `rung` is in: one of its moves at its top on the stage's days, or (with the
-- filter context `f`) the setup shuts every one of them, skipped (the second value).
local function stageIn(state, l, rung, stage, f)
	local need = ns.StageDays(stage)
	for _, id in ipairs(stage.ids) do
		if (ns.TopDays(state, l, rung, id)) >= need then return true, false end
	end
	if f and stageShut(stage, f) then return true, true end
	return false, false
end

-- Whether the climb from `rung` has its path: every stage in (stageIn), or the ladder stood above the rung already
-- (ns.BestRung: unlocks are kept, so a step the ladder was placed past or climbed above counts its stages done).
-- Every climb waits for it, up, plateau or placement, a pose family's and a counted cardio rung's too.
function ns.PathIn(state, ladder, rung, f)
	local path = ns.Path(ladder, rung)
	if #path == 0 or ns.BestRung(state, ladder) > rung then return true end
	for _, stage in ipairs(path) do
		if not stageIn(state, ladder, rung, stage, f) then return false end
	end
	return true
end

-- Whether a move of rung `rung`'s own chain at its level (its main move, under any setup) reached its top once.
local function mainOnce(state, l, rung)
	for _, id in ipairs(ns.RungChain(l, rung)) do
		if ns.Library[id].rung == rung and ns.TopOnce(state, id) then return true end
	end
	return false
end

-- The stage whose moves a stage `i` with no `opens` waits for: the one before it, past those the setup `f` shuts every
-- move of; nil for the main move.
local function stageBefore(path, i, f)
	for k = i - 1, 1, -1 do
		if not (f and stageShut(path[k], f)) then return path[k] end
	end
	return nil
end

-- Whether a move already holds a trace of its own, so a stage it is in stays open: a record, a goal pose answered
-- Good or better, a clean window (2026-10-16: "a 'then' move already holding a record stays open").
local function traced(state, moveId)
	local m = state.moves[moveId]
	if m and m.best then return true end
	local d = state.marks and state.marks[moveId]
	if d and ((d.good or 0) > 0 or (d.perfect or 0) > 0) then return true end
	local c = state.cardio
	return c ~= nil and c.cleanEver ~= nil and c.cleanEver[moveId] ~= nil
end

-- The move a stage's `opens` names, where the setup `f` offers it (ns.Passes with no pain flag; with no `f` every
-- move): nil for "step", "main" and none, and for a named move the setup or the player's own list shuts, whose top
-- could never come, so the stage opens on the main move's top instead (2026-10-17, slice 42's review: with the dip
-- lowering left out, power step 9's straight-bar dip waited for good).
local function opensMove(opens, f)
	if opens == nil or opens == "step" or opens == "main" then return nil end
	if f and not ns.Passes(opens, setupOf(f)) then return nil end
	return opens
end

-- Whether stage `i` of the climb from `rung` is open, its moves offered (ns.MoveOpen; David, 2026-10-16: "in a proper
-- order unlocked one after the others"): "step" with the step, "main" once the main move reached its top, a move id
-- once that move did (the main move's top where the setup shuts it, opensMove), none once a move of the stage before
-- it did (stageBefore; the main move for the first); every stage once the ladder stood above the rung (ns.BestRung),
-- and a stage one of whose moves already holds a trace of its own (traced, 2026-10-17: the acceptance's "a player who
-- already has a desk pistol record sees it open", which the stage's row said waiting for until then).
local function stageOpen(state, l, rung, i, f)
	if ns.BestRung(state, l) > rung then return true end
	local path = ns.Path(l, rung)
	local stage = path[i]
	for _, id in ipairs(stage.ids) do
		if traced(state, id) then return true end
	end
	local opens = stage.opens
	if opens == "step" then return true end
	local named = opensMove(opens, f)
	if named then return ns.TopOnce(state, named) end
	local before = opens == nil and stageBefore(path, i, f)
	if not before then return mainOnce(state, l, rung) end
	for _, id in ipairs(before.ids) do
		if ns.TopOnce(state, id) then return true end
	end
	return false
end

-- Of a list of moves, the one to name: the first the setup offers (ns.Passes with `f`; any move without one), else
-- the first.
local function offered(ids, f)
	for _, id in ipairs(ids) do
		if not f or ns.Passes(id, f) then return id end
	end
	return ids[1]
end

-- What opens a waiting stage `i`: the move whose top opens it and that top (ns.MoveTop on the climb from `rung`): the
-- main move (the move standing for the rung under `f`, else its anchor; also where the setup shuts the move the stage
-- names, opensMove), the move it names, or a move of the stage before it (offered); nil for a stage that opens with
-- the step.
local function stageOpener(l, rung, i, f)
	local path = ns.Path(l, rung)
	local opens = path[i].opens
	if opens == "step" then return nil end
	local id = opensMove(opens, f)
	if not id then
		local before = opens == nil and stageBefore(path, i, f)
		if before then
			id = offered(before.ids, f)
		else
			id = f and ns.RungMove(l, rung, f) or nil
			if not (id and ns.Library[id].rung == rung) then id = ns.Anchor(l, rung) end
		end
	end
	local m = ns.Library[id]
	return id, ns.MoveTop(l, m.rung, id)
end

-- The climb from `rung`'s path as a card and its beads read it (2026-10-16, slice 42: the game's Talents tab, the
-- desktop app's and the phones' Book, "Then, for the next step"): each stage in order as { ids, days (the game days at
-- its top it takes), state = "done", "skipped" (the setup shuts every move), "open" (its moves offered) or "waiting",
-- move (the one to name: of those the setup `f` offers the nearest its days, a tie the first, else the first), have
-- (that move's days at its top; none on a done stage, which counts nothing more: 2026-10-17, slice 42's review, where
-- a step the ladder stood above read "0 of 2 clean days" beside its check mark), and on a waiting stage `after` and
-- `top` (stageOpener) }. A ladder that stood above `rung` has every stage done; a stage one of whose moves holds a
-- trace of its own is open (stageOpen).
function ns.Stages(state, ladder, rung, f)
	local out = {}
	local above = ns.BestRung(state, ladder) > rung
	for i, stage in ipairs(ns.Path(ladder, rung)) do
		local need = ns.StageDays(stage)
		local move, have = nil, -1
		for _, id in ipairs(stage.ids) do
			local days = ns.TopDays(state, ladder, rung, id)
			if (not f or ns.Passes(id, f)) and days > have then move, have = id, days end
		end
		if not move then move, have = stage.ids[1], (ns.TopDays(state, ladder, rung, stage.ids[1])) end
		local inIt, skipped = stageIn(state, ladder, rung, stage, f)
		local e = { ids = stage.ids, days = need, move = move, have = math.min(have, need) }
		if above or inIt and not skipped then
			e.state, e.have = "done", nil
		elseif skipped then
			e.state = "skipped"
		elseif stageOpen(state, ladder, rung, i, f) then
			e.state = "open"
		else
			e.state = "waiting"
			e.after, e.top = stageOpener(ladder, rung, i, f)
		end
		out[i] = e
	end
	return out
end

-- Whether a move is open (2026-10-16, slice 42): offered by the pumps, Swap and a Start. A move with `checks` once each
-- is done (ns.NeedDone; one-time checks, kept), or once it holds a trace of its own (traced: 2026-10-19, the program's
-- double-check, a returning player's pistol record kept the move open past the ankle check that came after it, as a
-- trace keeps a path stage open; the check still shows on its card and still lifts its stretch ladder when done); a
-- move a step's path names once its stage opens (stageOpen, a move of it holding a trace already among the ways). With
-- the filter context `f` (ns.FilterContext; nil reads every stage as the setup offering it). Returns true, or false and
-- what opens it: { kind = "check", need } the first check not done (an `any` group whole), or { kind = "path", ladder,
-- rung, stage, after, top }: its stage, and the move whose top opens it with that top (stageOpener).
function ns.MoveOpen(state, moveId, f)
	local m = ns.Library[moveId]
	if not m then return true end
	if m.checks and not traced(state, moveId) then
		for _, o in ipairs(m.checks) do
			if not ns.NeedDone(state, o) then return false, { kind = "check", need = o } end
		end
	end
	local waits = ns.PathWaits(state, moveId, f)
	if waits then return false, waits end
	return true
end

-- What holds a move by its step's path alone (ns.MoveOpen's second answer for it, whatever its checks: a move window
-- names it beside a check that holds the move too, 2026-10-17): { kind = "path", ladder, rung, stage, after, top }
-- while its stage is not open (stageOpen, which a trace of one of its moves opens); nil else, and for a move no path
-- names.
function ns.PathWaits(state, moveId, f)
	local at = stageOf[moveId]
	if not at or stageOpen(state, at.ladder, at.rung, at.stage, f) then return nil end
	local after, top = stageOpener(at.ladder, at.rung, at.stage, f)
	return { kind = "path", ladder = at.ladder, rung = at.rung, stage = at.stage, after = after, top = top }
end

-- When a move was last done, for the ties that turn by it (a path's turn here, every pick's in Engine.lua byScore since
-- 2026-10-31; 0 for one never done): a move that keeps a record its last set, a pose of a family its last ask or its
-- last hold its whole time, and a move that keeps neither its last row (the fold's formTs): a cardio form (slice 52)
-- and, since 2026-10-31, a flow move outside a pose family (the carries and the good morning, 0 until then, so the
-- lowest id took their ties).
function ns.DoneAt(state, id)
	local m = ns.Library[id]
	if m.dose == "flow" and POSE[m.ladder] then
		local d = state.marks and state.marks[id]
		return d and math.max(d.askTs or 0, d.heldTs or 0) or 0
	end
	if m.dose == "cardio" or m.dose == "flow" then return state.formTs and state.formTs[id] or 0 end
	local rec = state.moves[id]
	return rec and rec.lastTs or 0
end

-- The move that takes a climb's turn for its path (2026-10-16; ns.ProofNear's place, the pumps' turn in Engine.lua
-- candidates and finisherMove, and the Start's): of the first stage still out (not in, not skipped) that has one, the
-- move nearest its days among those the setup `f` offers that are open (ns.MoveOpen), the player has not stepped down
-- from (ns.SteppedDown) and whose cardio rung is reached (`crung`), a tie the one done longest ago (ns.DoneAt), then
-- the lowest id (slice 52, 2026-10-23: the lowest id alone until then, so the squat thrusts took every tie of step 6's
-- path and the sprawl never came); nil where none is out; second, its stage's index in the path (ns.PathNext names it).
-- Each caller says when the turn is the path's (the climb's days in, a counted rung's always).
function ns.PathTurn(state, ladder, f, rung)
	if ns.BestRung(state, ladder) > rung then return nil end
	for i, stage in ipairs(ns.Path(ladder, rung)) do
		if not stageIn(state, ladder, rung, stage, f) then
			local need, best, share, at = ns.StageDays(stage), nil, nil, nil
			for _, id in ipairs(stage.ids) do
				local crung = ns.Library[id].crung
				if ns.Passes(id, f) and ns.MoveOpen(state, id, f) and not ns.SteppedDown(state, id)
					and not (crung and ns.Rung(state, ns.CARDIO) < crung) then
					local s, t = (ns.TopDays(state, ladder, rung, id)) / need, ns.DoneAt(state, id)
					if not best or s > share or s == share and (t < at or t == at and id < best) then
						best, share, at = id, s, t
					end
				end
			end
			if best then return best, i end
		end
	end
	return nil
end

-- The move that holds a climb's path where no move of it can take the turn (ns.PathTurn nil; 2026-10-17, slice 42's
-- review: power step 6's knee clap push-up waits for 12 feet-up push-ups, and the step's Start offered a set that could
-- never move the climb): of the stages still out (not in, not skipped), in order, the first move the setup `f` offers
-- that is not open, and what opens it (ns.MoveOpen's answer: its check, or the move whose top opens its stage); nil
-- where none is out or none of them is held; third, the stage's index in the path. A step's Start runs what opens it
-- (waitingStart), and the pumps give that check's move the turn where they offer it (Engine.lua candidates).
function ns.PathHeld(state, ladder, f, rung)
	if ns.BestRung(state, ladder) > rung then return nil end
	for i, stage in ipairs(ns.Path(ladder, rung)) do
		if not stageIn(state, ladder, rung, stage, f) then
			for _, id in ipairs(stage.ids) do
				if ns.Passes(id, f) then
					local open, why = ns.MoveOpen(state, id, f)
					if not open then return id, why, i end
				end
			end
		end
	end
	return nil
end

-- The next stage of the climb from `rung`'s path, the one rule every Next and key line names it by (2026-10-19, the
-- program's double-check: the game's Talents tab, the desktop app's and the phones' Book each picked their own; the
-- move the turn runs since its fix review, the same day: power step 6 named the knee clap push-up its check holds
-- while the pumps and the Start ran the open frog jumps): nil where the path is in (ns.PathIn) or the rung has none.
-- Else the move that takes the climb's turn (ns.PathTurn, as the pumps and the step's Start run it) and its stage;
-- where none can, what holds the climb (ns.PathHeld: the first move the setup offers on a stage still out that is not
-- open, with its wait, a check of its own or the move whose top opens its stage, which the step's Start runs); else,
-- where a pain flag, a step down or a cardio rung not reached holds every move left, the first stage open with the
-- move ns.Stages names (of those the setup `f` offers the nearest its days; its own wait where a pain flag leaves the
-- stage none and that move is not open), else the first stage waiting with what opens it (stageOpener). Returns
-- { stage (its index in ns.Path), move, top (ns.MoveTop on the climb from `rung`: reps or seconds, a pose's least
-- answer, true for a countable form's clean window), days (the stage's, ns.StageDays), have (the move's game days at
-- its top, at most `days`), waits (none, or ns.MoveOpen's: { kind = "check", need } or { kind = "path", ladder, rung,
-- stage, after, top }) }.
function ns.PathNext(state, ladder, rung, f)
	if ns.PathIn(state, ladder, rung, f) then return nil end
	local path = ns.Path(ladder, rung)
	local function at(i, id, waits)
		local need = ns.StageDays(path[i])
		return { stage = i, move = id, top = ns.MoveTop(ladder, rung, id), days = need,
			have = math.min((ns.TopDays(state, ladder, rung, id)), need), waits = waits }
	end
	local turn, i = ns.PathTurn(state, ladder, f, rung)
	if turn then return at(i, turn) end
	local holder, why, k = ns.PathHeld(state, ladder, f, rung)
	if holder then return at(k, holder, why) end
	local stages = ns.Stages(state, ladder, rung, f)
	for s, st in ipairs(stages) do
		if st.state == "open" then
			local open, wait = ns.MoveOpen(state, st.move, f)
			return at(s, st.move, not open and wait or nil)
		end
	end
	for s, st in ipairs(stages) do
		if st.state == "waiting" then
			return at(s, st.move, { kind = "path", ladder = ladder, rung = rung, stage = s, after = st.after,
				top = st.top })
		end
	end
	return nil
end

-- The one stage of the climb from `rung` still out (not in, not skipped: stageIn) where it lists `id`; nil where none
-- is out, more than one is, or the one out does not list it.
local function soleStage(state, l, rung, f, id)
	local mine
	for _, stage in ipairs(ns.Path(l, rung)) do
		if not stageIn(state, l, rung, stage, f) then
			local listed = false
			for _, x in ipairs(stage.ids) do listed = listed or x == id end
			if not listed or mine then return nil end
			mine = stage
		end
	end
	return mine
end

-- What a slot's move still has to reach while its ladder's climb waits on this move's stage alone (2026-10-16; the
-- proof's ns.ProofDue until slice 42): the step's own days are in and its path is the one count still out
-- (ns.UnlockProgress), and the one stage still out lists this move (soleStage): a strength or hold ladder's, a pose
-- family's, a counted set's form on a counted cardio rung (a timed cardio move counts nothing toward it). Returns the
-- move's top (ns.MoveTop: reps or seconds, a pose's least answer, true for a countable form's clean window), the game
-- days at that top the stage still needs, and whether a set at that top at `now` brings the path (one day short, its
-- day not counted this session: the night rule after the set that counted the last); else nil. The voice's path
-- callout (Stats.lua) fires on the third and the Ready view's next-step line (Panel.lua) shows the first two, so the
-- line shows whenever the voice says the next step waits on the move (2026-10-16, David: "the goal is 8 while record
-- is 10. idont know what thesentence is telling me": a repeat set's goal sits under the top the callout meant, which
-- nothing on the screen named).
function ns.PathDue(state, slot, f, now)
	local l = slot and slot.ladder
	if not (l and (ns.STRENGTH[l] or POSE[l] or l == ns.CARDIO and slot.counted)) then return nil end
	local have, need, open = ns.UnlockProgress(state, l, f)
	if not (need > 0 and have >= need and open == 1) then return nil end
	local rung = ns.Rung(state, l)
	local stage = soleStage(state, l, rung, f, slot.move)
	if not stage then return nil end
	local days, last = ns.TopDays(state, l, rung, slot.move)
	local left = ns.StageDays(stage) - days
	return ns.MoveTop(l, rung, slot.move), left, left == 1 and (last == nil or ns.NewSession(last, now))
end

-- Mastered (2026-10-16, slice 42) -----------------------------------------------------------------------------------

-- The moves a step's mastery waits for: every move of the rung and every move its path names (a stage's on the rung
-- below too: L-sit step 6's one-heel L-sit, core step 11's full rollout; 2026-10-17, slice 42's review) with a top
-- (ns.MoveTop on the climb from the rung: a strength or hold move, a pose with a goal pose, a countable form) that the
-- setup offers (ns.Passes with `f` and no pain flag: the equipment, the switches, the player's own list and the opt-in
-- moves checked), never a twin of the rung's chain while a move before it in the chain (ns.RungChain, at the rung's
-- own level) passes, the move it stands in for, and never the anchor (hinge step 9's Nordic hold names the Nordic
-- lowering as its twin outside the chain, which dropped the step's main move until 2026-10-17); in id order. A
-- reward, never a gate: "all topped" opens nothing (David, 2026-10-16, decided on merit: a gate would cost weeks on
-- the crowded steps).
function ns.MasteryMoves(ladder, rung, f)
	local setup = setupOf(f)
	local standsIn, before = {}, false
	for _, id in ipairs(ns.RungChain(ladder, rung)) do
		if ns.Library[id].rung == rung then
			if before then standsIn[id] = true end
			before = before or ns.Passes(id, setup)
		end
	end
	local seen, list = {}, {}
	local function add(id)
		if not seen[id] and not standsIn[id] and ns.MoveTop(ladder, rung, id) and ns.Passes(id, setup) then
			list[#list + 1] = id
		end
		seen[id] = true
	end
	for _, id in ipairs(members[ladder] or EMPTY) do
		if ns.Library[id].rung == rung then add(id) end
	end
	for _, stage in ipairs(ns.Path(ladder, rung)) do
		for _, id in ipairs(stage.ids) do add(id) end
	end
	table.sort(list)
	return list
end

-- The moves of a step's mastery (ns.MasteryMoves) that have not reached their top once (ns.TopOnce).
function ns.MasteryLeft(state, ladder, rung, f)
	local left = {}
	for _, id in ipairs(ns.MasteryMoves(ladder, rung, f)) do
		if not ns.TopOnce(state, id) then left[#left + 1] = id end
	end
	return left
end

-- When a step was mastered (the fold's `mastered`, its first mastered row; DB.lua), nil before: kept through a step
-- down and a setup change, never past a start over of its ladder (DB.lua clearRow, 2026-10-31), and a pose family's
-- row from before slice 47's goal poses (pose_mastery_ts) and a cardio step's from before its forms' first clean
-- windows came from answers alone (cardio_mastery_ts, 2026-10-31) never count (DB.lua masteryCut).
function ns.Mastered(state, ladder, rung)
	return state.mastered and state.mastered[ns.RungKey(ladder, rung)]
end

-- The steps mastered, every ladder's (the Deeds' count).
function ns.MasteredCount(state)
	local n = 0
	for _ in pairs(state.mastered or EMPTY) do n = n + 1 end
	return n
end

-- The steps of other ladders (opens, needs, checks) --------------------------------------------------------------

-- A step another ladder waits for (`o`: an entry of a family's `opens`, a rung's `needs`, a move's `checks`; never an
-- `any` group): the anchor of the rung at that step and the count its record must reach, the step's own `count` where
-- it names one (2026-10-10: the handstand waits for a push-up record of 15, the push-up's floor only starting it),
-- else its window's floor ("Opens at a Push-up record of 6"; a twin of the rung does it too, stepDone); a step that
-- names its `move` (2026-10-12: cardio's burpees wait for a clap push-up record of 8, power's own step only starting
-- them) that move and its count, or a pose's goal pose answered at its `mark` (2026-10-16) that pose alone; on a
-- ladder whose rungs have no window (a pose family, cardio) that rung's anchor and no count: the ladder's climb doing
-- the step does it (stepDone; such a need counted as met until 2026-10-10). Nil for a step no rung holds.
local function stepAt(o)
	if o.move then return o.move, o.count end
	local rung = ns.RungAtStep(o.ladder, o.step)
	local id = rung and ns.Anchor(o.ladder, rung)
	if not id then return nil end
	local lo = ns.Window(o.ladder, rung, id)
	return id, lo and (o.count or lo) or nil
end

-- What a family waits for (`opens`): the first entry (with the player's `state`, the first not done) as stepAt reads
-- it, the move and the count where a record does it (no count on a pose or cardio step), with the entry itself; nil
-- for a family that waits for nothing, and with `state` for one whose every entry is done.
function ns.OpensAt(ladder, state)
	for _, o in ipairs(ns.Ladders[ladder].opens or EMPTY) do
		if not state or not ns.NeedDone(state, o) then
			local id, count = stepAt(o)
			return id, count, o
		end
	end
	return nil
end

-- The count a move's record must reach to do a step `o` with a window (stepAt): a move of the step's rung's own chain
-- at that rung's level (its anchor or a twin: the push-up, or the desk one-arm lowering for a player who left the
-- push-up out; a stand-in from below never does it, as it never climbs it), counted in its own unit (ns.OwnUnit), at
-- the step's count, which on a twin with a window of its own (`alt`) scales by the two windows' tops, rounded up
-- (2026-10-10: a push-up record of 15 of 20 is 8 on the lowering's 4-10), or where the step names none at the floor of
-- its own window; on a step that names its `move` that move alone, at the step's count; nil for any other move and for
-- a step with no window or a pose's goal pose.
local function stepFloor(o, moveId)
	if o.move then return moveId == o.move and o.count or nil end
	local anchor, count = stepAt(o)
	if not count then return nil end
	local rung = ns.RungAtStep(o.ladder, o.step)
	local m = ns.Library[moveId]
	if not (m and m.rung == rung and ns.InChain(o.ladder, rung, moveId)) then return nil end
	local lo, hi, own = ns.Window(o.ladder, rung, moveId)
	if not (lo and ns.OwnUnit(moveId, anchor, own)) then return nil end
	if not o.count then return lo end
	if not own then return o.count end
	local _, top = ns.Window(o.ladder, rung, anchor)
	return math.floor((o.count * hi + top - 1) / top)
end

-- Whether a pose step is done (2026-10-12) by its pose: a pose of the step's rung's own chain at its level (its anchor
-- or a twin of its rung: its main pose, ns.MainPose) answered at its goal pose where the ask came at least
-- as close as the climb from that rung counts (ns.MarkBest at ns.MarkNeeds; 2026-10-17, slice 42's review: a check
-- counts only an asked answer, where its mark days counted a hold with no ask too). Until 2026-10-12 a pose step was
-- done by standing on its rung, one rung early: the side crow waited for the chair twist's mark where the plan asked
-- for the twisted chair's.
local function poseStepDone(state, l, rung)
	local need = ns.MarkNeeds(l, rung)
	for _, id in ipairs(ns.RungChain(l, rung)) do
		if ns.Library[id].rung == rung and ns.MarkBest(state, id) >= need then return true end
	end
	return false
end

-- Whether a step `o` is done: a pose's goal pose named with its `mark` (2026-10-16) by that pose answered at least
-- that close on a game day (ns.MarkDays), its family's climb past it doing nothing for it (the named pose is the
-- check); else its ladder stands above the step's rung (ns.BestRung: unlocks are kept; 2026-10-12: a player placed past
-- a rung never does its moves, and the pull-up's flexed-arm hang waited for good for ten rows of the row's rung 3, a
-- need's named move for good once its ladder had climbed past the rungs that offer it); else on a step that names its
-- `move`, that move (other than `skip`) has its record at the step's count; on a step with a window, a move of its
-- rung's chain (stepFloor) other than `skip` has its record at its count; on a pose step, the pose of its rung done
-- (poseStepDone); on a cardio step, the ladder's best rung has reached the step.
local function stepDone(state, o, skip)
	if o.mark then return o.move ~= skip and ns.MarkBest(state, o.move) >= o.mark end
	local rung = ns.RungAtStep(o.ladder, o.step)
	if rung and ns.BestRung(state, o.ladder) > rung then return true end
	if o.move then return o.move ~= skip and (ns.Record(state, o.move) or 0) >= o.count end
	local _, count = stepAt(o)
	if not count then
		if POSE[o.ladder] and rung then return poseStepDone(state, o.ladder, rung) end
		return (ns.Step(o.ladder, ns.BestRung(state, o.ladder)) or 0) >= o.step
	end
	for _, id in ipairs(ns.RungChain(o.ladder, rung)) do
		local lo = id ~= skip and stepFloor(o, id)
		if lo and (ns.Record(state, id) or 0) >= lo then return true end
	end
	return false
end

-- Whether an entry of a family's `opens`, a rung's `needs` or a move's `checks` (library.json: { ladder, step, move?,
-- count?, mark? } or { any = { ... } }) is done: an `any` group when one of its entries is (2026-10-16), any other by
-- stepDone. The check beside each step a card names, so no card marks one done while the climb still waits for it,
-- nor one undone that it counts.
function ns.NeedDone(state, need)
	if need.any then
		for _, o in ipairs(need.any) do
			if stepDone(state, o) then return true end
		end
		return false
	end
	return stepDone(state, need)
end

-- The first step of another ladder the climb to `rung` waits for (the rung's `needs`, 2026-10-09: the flags wait for
-- the pull-up and the push-up done) that the player has not done (ns.NeedDone), as that step's anchor and the record
-- that does it (stepAt: none on a pose or cardio step; an `any` group its first entry's), with the entry itself; nil
-- when the rung needs none or every one is done. Read by ns.CanRungUp with the state and named by the top-out line
-- (ns.TopOut).
function ns.RungNeed(state, ladder, rung)
	local w = ns.Ladders[ladder].rungs[rung]
	for _, o in ipairs(w and w.needs or EMPTY) do
		if not ns.NeedDone(state, o) then
			local id, count = stepAt(o.any and o.any[1] or o)
			return id, count, o
		end
	end
	return nil
end

-- What step `need` of a rung's `needs` or a move's `checks` waits for (stepAt): the move whose record does it and
-- that record, or on a pose or cardio step its anchor alone, a pose's goal pose its pose: each need's own waits line
-- on a card, where ns.RungNeed names only the first undone. With the player's filter `f` (ns.FilterContext) a step
-- whose anchor the setup leaves out names the first move of its rung's chain at that rung's level the setup offers, at
-- its own count (stepFloor: a twin with its own window scales it), the move a card's Start would train, never one the
-- player cannot do (a Feet-up push-up record of 6 for a player with Floor moves off, whom a Desk one-arm push-up record
-- of 2 serves); a step that names its `move` keeps it; an `any` group names its first entry whose move the setup
-- offers (else its first).
function ns.NeedAt(need, f)
	if need.any then return ns.NeedAt(f and ns.EntryFor(need, f) or need.any[1], f) end
	local id, count = stepAt(need)
	if not (f and id) or need.move or ns.Passes(id, f) then return id, count end
	local rung = ns.RungAtStep(need.ladder, need.step)
	for _, other in ipairs(ns.RungChain(need.ladder, rung)) do
		if ns.Library[other].rung == rung and ns.Passes(other, f) then
			local floor = stepFloor(need, other)
			if floor or not count then return other, floor end
		end
	end
	return id, count
end

-- Whether a need is a one-time stretch check (2026-10-16, slice 42): a pose family's step or goal pose, which a card's
-- "Do the check" runs (ns.StartFor's target { check }); never an `any` group (each of its entries is its own).
function ns.IsCheck(need)
	return need.any == nil and POSE[need.ladder] == true
end

-- The entry of a need that a Start or a pump runs: an `any` group's first entry whose move (ns.NeedAt) the setup `f`
-- offers, else its first; any other need itself.
function ns.EntryFor(need, f)
	if not need.any then return need end
	for _, x in ipairs(need.any) do
		local id = ns.NeedAt(x, f)
		if id and ns.Passes(id, f) then return x end
	end
	return need.any[1]
end

-- The least answer to "How close to the goal pose?" a check counts (ns.IsCheck): its `mark`, else the answer its step's
-- climb counts a day at (ns.MarkNeeds: Good, Perfect below a risky step).
function ns.CheckAsk(need)
	if need.mark then return need.mark end
	return ns.MarkNeeds(need.ladder, ns.RungAtStep(need.ladder, need.step))
end

-- The rung a check's pose stands on (ns.IsCheck): its goal pose's own, else its step's (its twins stand on it too).
local function checkRung(need)
	if need.move then return ns.Library[need.move].rung end
	return ns.RungAtStep(need.ladder, need.step)
end

-- What runs before check `need` (2026-10-17, slice 42's review: a check never lifts a family past a step whose own
-- needs are not done, so ankle step 11's twist step 5 never skips the twisted triangle's own pike step 4): the first
-- entry not done (ns.RungNeed; an `any` group whole) of the climbs into the steps from the one above its family's up
-- to the check pose's own, else the check itself. ns.StartFor's check (checkStart) runs it (ns.RunsCheck names the
-- check row that run passes), and the pumps' stretch slot gives a waiting check's pose only where it is the check
-- itself (Engine.lua).
function ns.CheckFirst(state, need)
	local l = need.ladder
	for x = ns.Rung(state, l) + 1, checkRung(need) or 0 do
		local _, _, o = ns.RungNeed(state, l, x)
		if o then return o end
	end
	return need
end

-- The step a pose answered close enough above its family's lifts the family to (ns.Progression's check): its own
-- rung `k`, or where the climb into a step on the way waits for another ladder's step (ns.RungNeed) the rung under
-- that one, which is `from` where the first does (2026-10-17, slice 42's review: a family lifts only to the highest
-- step whose needs are all done).
local function liftTo(state, l, from, k)
	for x = from + 1, k do
		if ns.RungNeed(state, l, x) then return x - 1 end
	end
	return k
end

-- Whether the climb from `rung` is a climb back (2026-10-10, David: "Do we 'downgrade' again ... if I drop below a
-- certain level in plank holds, or push ups (when it was a fresh day) does it then show the old again until that top
-- level is reached again to 'unlock' the advanced rung? Think it through whats the best here"): the ladder stood on the
-- rung above already (ns.BestRung, so a ladder started over counts its reach no more) and a move of that rung's own
-- chain has its record at its window's floor (stepDone), so the player did that rung, and two fresh sets under its
-- floor that stepped the ladder down say more about a tired week than lost strength, which holds for weeks (Mujika and
-- Padilla 2000). One game day at the top of the rung below brings it back (C.return_days); a rung reached and failed at
-- once keeps the climb's days, the case the step down is for. The cardio ladder's too since slice 53 (2026-10-23): a
-- step it held before (stepDone: its best rung reached it) comes back on return_days clean days or full moves
-- (ns.CleanDays, Progress.lua cardioRule and cardioProgress), after a step down by its footing or by the return's
-- measure after a break.
function ns.Returning(state, ladder, rung)
	local w = ns.Ladders[ladder] and ns.Ladders[ladder].rungs[rung + 1]
	if not w or POSE[ladder] or ns.BestRung(state, ladder) < rung + 1 then return false end
	return stepDone(state, { ladder = ladder, step = w.step })
end

-- Whether a family is open: one without `opens` always; one with them once the player has done every step they name
-- (ns.NeedDone; a list since 2026-10-16: the planche waits for a push-up record of 10 and the wrist lean answered
-- Perfect), and for good once the family itself is placed or has a rung. A strength step is done when the record of a
-- move that stands for its rung at its level reaches its count (stepFloor: the push-up's, or the desk one-arm
-- lowering's own for a player who left the push-up out; David, 2026-10-05: "I have not even had normal push ups yet":
-- a first max set at the top of the rung below placed his push ladder on the push-up, never done, and the pike push-up
-- came; the ladder's step alone opened it until then), a cardio step when the ladder reaches it (its best rung: unlocks
-- are kept). A record is kept too, so an open family stays open. A closed family is never offered and shows no colour
-- (Engine.lua context, the Talents tab).
function ns.Open(state, ladder)
	local opens = ns.Ladders[ladder].opens
	if not opens then return true end
	local lad = state.ladders and state.ladders[ladder]
	if lad and (lad.rung or lad.placedTs) then return true end
	for _, o in ipairs(opens) do
		if not ns.NeedDone(state, o) then return false end
	end
	return true
end

-- What a stretch check opens, as the readers of the checks hold it (Tree.lua Tree.AllChecks, the phones' SheetCard,
-- tools/apppump.lua): { move } that move, { family } that family's first step, { ladder, rung } the climb into that
-- rung. The entries it waits for: the move's `checks`, the family's `opens`, the rung's `needs`.
local function waitsOf(opens)
	if opens.move then return ns.Library[opens.move].checks or EMPTY end
	if opens.family then return ns.Ladders[opens.family].opens or EMPTY end
	local w = ns.Ladders[opens.ladder].rungs[opens.rung]
	return w and w.needs or EMPTY
end

-- Whether what a stretch check opens is open now (2026-10-17, slice 42's review: "Check passed: Dragon pistol, hand on
-- the desk is open" came while it still waited for ankle step 6 and its stage): a move by ns.MoveOpen with `f` (its
-- checks and its stage of a step's path), a family by ns.Open, the climb into a rung by its `needs` (ns.RungNeed; its
-- days and its path are the climb's own). Returns true, or false and what still holds it, the next wait for the
-- Result to word: the move's ns.MoveOpen answer, or the first entry not done of a family's opens or a rung's needs as
-- { kind = "check", need } (an `any` group whole), as ns.MoveOpen names a move's.
function ns.OpensNow(state, opens, f)
	if opens.move then return ns.MoveOpen(state, opens.move, f) end
	if opens.family and ns.Open(state, opens.family) then return true end
	for _, o in ipairs(waitsOf(opens)) do
		if not ns.NeedDone(state, o) then return false, { kind = "check", need = o } end
	end
	return true
end

-- Every stretch check the rules hold (slice 42; ns.IsCheck: a need naming a pose family's step or goal pose), read
-- once: each entry of a family's `opens` (its first step), a rung's `needs` (the climb into that rung) and a move's
-- `checks` (that move, by id), an `any` group's alternatives each, as { need, group (its whole entry, the library's
-- own table), opens = { family } | { ladder, rung } | { move } }: the one list the game's
-- cards and pumps (Tree.lua) and the desktop app's pumps (tools/apppump.lua Pump:checks) read, as the phones' cores
-- port it (SheetCard).
local allChecks
function ns.AllChecks()
	if allChecks then return allChecks end
	local list = {}
	local function visit(n, opens)
		for _, e in ipairs(n.any or { n }) do
			if ns.IsCheck(e) then list[#list + 1] = { need = e, group = n, opens = opens } end
		end
	end
	for l = 1, ns.LADDER_MAX do
		local lad = ns.Ladders[l]
		for _, o in ipairs(lad and lad.opens or EMPTY) do visit(o, { family = l }) end
		for r, w in ipairs(lad and lad.rungs or EMPTY) do
			for _, o in ipairs(w.needs or EMPTY) do visit(o, { ladder = l, rung = r }) end
		end
	end
	local ids = {}
	for id, m in pairs(ns.Library) do
		if m.checks then ids[#ids + 1] = id end
	end
	table.sort(ids)
	for _, id in ipairs(ids) do
		for _, o in ipairs(ns.Library[id].checks) do visit(o, { move = id }) end
	end
	allChecks = list
	return list
end

-- Whether pose `id` answers check `need`: the pose a goal-pose check names, or a pose of the check's step's own chain
-- at its level (the poses that do a pose step, as ns.NeedDone reads them); ns.ChecksOf, Tree.lua and
-- tools/apppump.lua Pump:checks read it.
function ns.AnswersCheck(need, id)
	if need.move then return need.move == id end
	local m = ns.Library[id]
	local rung = ns.RungAtStep(need.ladder, need.step)
	return m ~= nil and m.ladder == need.ladder and m.rung == rung and ns.InChain(need.ladder, rung, id)
end

-- Whether check row `c` (ns.AllChecks) can still open something on `state` (2026-10-18, slice 42's last review: the
-- game's Result said "Check passed: the climb to Squat step 7 is open" to a player whose squat stood on step 9, while
-- the desktop app left such a check out): its whole entry not done (ns.NeedDone, an `any` group whole) and what it
-- opens not open already, a family open (ns.Open: placed, climbed, or every entry of its opens done) or a climb into a
-- rung its ladder stands on or above; a move's opening never is while its check waits (its `checks` hold it,
-- ns.MoveOpen). The one filter of the checks a pose can pass on every surface (ns.ChecksOf, tools/apppump.lua
-- Pump:checks, the phones' cores).
function ns.CheckLive(state, c)
	if ns.NeedDone(state, c.group) then return false end
	local o = c.opens
	if o.family then return not ns.Open(state, o.family) end
	if o.ladder then return ns.Rung(state, o.ladder) < o.rung end
	return true
end

-- The live stretch checks (ns.CheckLive) that pose `id` answers (ns.AnswersCheck), in the rules' order (ns.AllChecks):
-- what a pump's pose may pass beyond the ones ns.PoseChecks puts first (Panel.lua readChecks, tools/apppump.lua
-- Pump:checks, the phones' cores), and what a card's Start of a check names (Tree.StartedCheck).
function ns.ChecksOf(state, id)
	local out = {}
	for _, c in ipairs(ns.AllChecks()) do
		if ns.AnswersCheck(c.need, id) and ns.CheckLive(state, c) then out[#out + 1] = c end
	end
	return out
end

-- The stretch checks a pump's pose `id` can pass, as its Result names them (2026-10-18, slice 42's
-- last review: one list on the game, the desktop app and the phones): first the checks of the entries `first` names,
-- in its order (a check's Start's own, the row ns.RunsCheck gives; then the one the stretch slot was given for,
-- Engine.lua checkSlot's `slot.check`), then every other the pose answers (ns.ChecksOf), each row once and only where
-- the pose answers it (ns.AnswersCheck) and it is live (ns.CheckLive): never a check whose opening is open already,
-- never one the pose cannot pass. Rows of ns.AllChecks (each entry table of the library is one row, so an entry names
-- its row).
function ns.PoseChecks(state, id, first)
	local out, seen = {}, {}
	local function add(c)
		if not seen[c] and ns.AnswersCheck(c.need, id) and ns.CheckLive(state, c) then
			seen[c] = true
			out[#out + 1] = c
		end
	end
	for _, need in ipairs(first or EMPTY) do
		for _, c in ipairs(ns.AllChecks()) do
			if c.need == need then add(c) end
		end
	end
	for _, c in ipairs(ns.ChecksOf(state, id)) do add(c) end
	return out
end

-- The check a check's Start runs (2026-10-18, slice 42's last review; the game's Tree.RunsCheck until then):
-- ns.StartFor's checkStart runs first what the climb to the check pose's step still waits for (ns.CheckFirst), so
-- ankle step 11's twist step 5 check runs the pike's big toe fold. Check row `c` (ns.AllChecks) itself where nothing on
-- the way waits; else the row of that first entry, its alternative the setup offers (ns.EntryFor with `setup`, the
-- setup alone, as ns.StartFor reads it), which opens the climb on `c`'s family, followed again as checkStart follows
-- it; nil where the Start runs no stretch check (a step of a strength or cardio ladder first: needStart's `via` needs).
-- The pump's note and Result name the row it gives, and a check's pump says the check waits only where that row did not
-- pass (Tree.lua, Panel.lua, tools/apppump.lua Pump:checks, the phones' cores).
function ns.RunsCheck(state, c, setup)
	for _ = 0, 4 do -- checkStart's depth: the library never chains checks this far
		local first = ns.CheckFirst(state, c.need)
		if first == c.need then return c end
		first = ns.EntryFor(first, setup)
		if not ns.IsCheck(first) then return nil end
		local found
		for _, x in ipairs(ns.AllChecks()) do
			if x.need == first and x.opens.ladder == c.need.ladder then
				found = x
				break
			end
		end
		if not found then return nil end
		c = found
	end
	return nil
end

-- When a stretch check was first passed (2026-10-17, slice 42's review: "Passed {date}" moved with every later hold in
-- the game and the desktop app): the first time a pose that does it was answered where the ask came at least as close
-- as the check counts (ns.CheckAsk; the fold's goodFirst and perfectFirst, DB.lua markDays), its goal pose for a check
-- that names one, else the earliest of its step's own chain at that step's level; nil while it is not done or where no
-- answer did it (a step its family stood above), which a card words by a stretch ladder's step instead
-- (ns.CheckPassed).
function ns.CheckPassedAt(state, need)
	if not ns.NeedDone(state, need) then return nil end
	local key = ns.CheckAsk(need) >= E.mark.perfect and "perfectFirst" or "goodFirst"
	local rung = checkRung(need)
	local poses = need.move and { need.move } or ns.RungChain(need.ladder, rung)
	local first
	for _, id in ipairs(poses) do
		local d = ns.Library[id].rung == rung and state.marks and state.marks[id]
		local ts = d and d[key]
		if ts and (not first or ts < first) then first = ts end
	end
	return first
end

-- How a card's check row says a stretch check is done (2026-10-18, slice 42's last review; docs/ui.md, "Stretch
-- checks"), `need` the choice the row shows (an `any` group's the setup offers, ns.EntryFor) of entry `group` (nil:
-- `need` alone): the earliest first pass over the group's checks (ns.CheckPassedAt; "Passed 16 Oct"), else, where no
-- answer did it (a step its family stood above), nil, the stretch ladder of the choice that is done (the shown one
-- where it is) and its step now ("Passed: your Ankle is at step 4."); nothing while the group waits. Read by the game's
-- Tree.CheckDone and the desktop app's E:needs; the phones' cores' checkPassed (SheetCard.swift, SheetCard.kt) port it.
function ns.CheckPassed(state, need, group)
	group = group or need
	if not ns.NeedDone(state, group) then return nil end
	local ts, done
	for _, e in ipairs(group.any or { group }) do
		if ns.IsCheck(e) then
			local at = ns.CheckPassedAt(state, e)
			if at and (not ts or at < ts) then ts = at end
			if not done and ns.NeedDone(state, e) then done = e end
		end
	end
	if ts then return ts end
	local l = (ns.NeedDone(state, need) and need or done or need).ladder
	return nil, l, ns.Step(l, ns.Rung(state, l)) or 0
end

-- The full moves on the cardio step (DB.lua steadyRow), 0 before any.
local function fullMoves(state)
	return (state.cardio or EMPTY).full or 0
end

-- The cardio ladder's progress toward its next rung (ns.UnlockProgress): a counted rung's clean days, of
-- ns.CleanDays, and 1 while its path waits (ns.PathIn), else 0; a timed rung's full moves, of cardio_full_days
-- (slice 52; its ready days, 2026-10-16, and its finished moves and their days before); a climb back to a step held
-- before, of return_days (slice 53, ns.Returning).
local function cardioProgress(state, rung, f)
	local c = state.cardio or EMPTY
	if ns.CountedRung(rung, f) then
		local need = ns.CleanDays(rung, state)
		return math.min(c.clean or 0, need), need, ns.PathIn(state, ns.CARDIO, rung, f) and 0 or 1, 0
	end
	local need = ns.Returning(state, ns.CARDIO, rung) and C.return_days or C.cardio_full_days
	return math.min(fullMoves(state), need), need, 0, 0
end

-- How near a ladder's next rung is, the one reading of it (the Talents tab's rings, hover, card and Next line, the
-- sheet summary's nx): have, need, have2, need2. A strength or hold ladder: the game days at the window's top (at
-- most the days its climb takes), those days (ns.UnlockDays), and 1 while the climb's path is still out (ns.PathIn),
-- else 0; a pose family: its days toward the step (ns.PoseNeeds' `have`: pose_unlock_days less the least counted days
-- still to come, so it reaches the need only with the main pose's and the Perfect days in too; slice 47), of
-- pose_unlock_days, and 1 while its path is out, then (a fifth and a sixth number, slice 47) its main pose's days and
-- its Perfect days still to come (ns.PoseNeeds' mainLeft, perfectLeft), which the sheet summary's nx carries; the
-- cardio ladder on a counted rung (ns.CountedRung): its clean days, of ns.CleanDays, and 1 while its path is out; on a
-- timed rung: its full moves, of cardio_full_days (need2 is 0 on every ladder since 2026-10-16). All zeros where no
-- climb is open: a retired id, a ladder that holds no move, a family not open yet (ns.Open), the top, a next rung the
-- setup shuts or one waiting for another ladder's step (ns.CanClimb with the state, ns.RungNeed). A pain flag on the
-- ladder (ns.PainOnLadder) holds the climb but keeps what it counted, so the progress shows and each reader says the
-- flag holds it (the Talents tab's pain lines).
function ns.UnlockProgress(state, ladder, f)
	if not ns.Ladders[ladder] or ns.HeldTop(ladder) == 0 or not ns.Open(state, ladder) then return 0, 0, 0, 0 end
	local rung = ns.Rung(state, ladder)
	if not ns.CanClimb(ladder, rung, f, state) then return 0, 0, 0, 0 end
	if ladder == ns.CARDIO then return cardioProgress(state, rung, f) end
	local lad = state.ladders and state.ladders[ladder]
	local days = lad and lad.hiDays or 0
	local out = ns.PathIn(state, ladder, rung, f) and 0 or 1
	if POSE[ladder] then
		local p = ns.PoseNeeds(state, ladder, rung)
		return p.have, p.need, out, 0, p.mainLeft, p.perfectLeft
	end
	local need = ns.UnlockDays(ladder, rung, state)
	return math.min(days, need), need, out, 0
end

-- What a pose's answer to "How close to the goal pose?" counts toward (2026-10-14), for a slot that asks it (its `ask`,
-- Engine.lua markSlot) with filter context `f`, read from `state` before the answer (nothing is written until it) at
-- ns.now(): the one reading the game's Ready view's days line (Panel.lua poseDaysLine) and the desktop app's
-- (tools/apppump.lua Pump:toward) take. "earn" on the family's main pose (ns.MainPose: DB.lua earnMark) asking Perfect
-- on an orange or gold step no hold has earned yet, whose colour only Perfect earns (ns.ToEarn, as the Talents tab's
-- "Next:" line says it); "check"
-- on a check above its family's step (slice 42: an answer at least the check asks lifts the family to the pose's step,
-- ns.Progression), with the step it lifts to (never past one whose needs wait, 2026-10-17; where the first does,
-- "goal" or "next" as below); else "step" with the move standing for the step above (ns.RungMove, the name the
-- Talents tab gives it) and the days counted toward it, `have` of `need`: a goal pose of the family's step (ns.Marks,
-- the fold's test, DB.lua markAttempt) where its answer can count the family's day now, a new session by the night
-- rule since the day counted last (the fold's hiTs, ns.NewSession; 2026-10-20, slice 47's fix round: a pose's own goal
-- ask after the day's count said "counts a day: 1 of 3" and counted none), the climb's (ns.UnlockProgress), with the
-- parts still short (ns.PoseNeeds, the fifth value; slice 47), a pose of a stage of the step's path still out the days
-- its stage asks (ns.TopDays, ns.StageDays) once the family's own are in (ns.PoseDaysIn), where the setup and the
-- other ladders' steps let the family climb (ns.CanRungUp); "next", the step above unnamed, where none counts (the
-- setup or a pain flag shuts it, it waits for another ladder's step, or the pose is on no stage still out), where the
-- Talents tab names what holds the climb instead; "goal" where no day counts toward a step (the family's day cannot
-- count now, or the family stands on its top step: 2026-10-20) and the pose's own goal pose still waits for its
-- answer (ns.GoalAsk, slice 42: its top once, toward its step's Mastered, or a check's mark on it).
function ns.MarkToward(state, slot, f)
	local l = slot.ladder
	local rung = ns.Rung(state, l)
	local own = ns.Library[slot.move].rung
	if own > rung then
		-- the step it lifts the family to, never past one whose needs wait (liftTo); none, and it counts toward its own
		-- goal pose or nothing
		local to = liftTo(state, l, rung, own)
		if to > rung then return "check", to end
		return ns.GoalAsk(state, slot.move) and "goal" or "next"
	end
	local marking = slot.rung == rung and ns.Marks(l, rung, slot.move)
	if slot.ask >= E.mark.perfect and marking and ns.MainPose(l, rung, slot.move) and ns.ToEarn(state, l, f) == rung then
		return "earn"
	end
	-- a goal pose of the step counts the family's day only in a new session since the day counted last (markAttempt)
	local lad = state.ladders and state.ladders[l]
	local counts = marking and (lad == nil or lad.hiTs == nil or ns.NewSession(lad.hiTs, ns.now()))
	-- a pose of a stage of the step's path still out counts that stage's days once the family's own are in
	local at = stageOf[slot.move]
	local stage = at and at.ladder == l and at.rung == rung and ns.Path(l, rung)[at.stage]
	if stage and stageIn(state, l, rung, stage, f) then stage = nil end
	local have, need, p
	if counts and not (stage and ns.PoseDaysIn(state, l, rung)) then
		have, need = ns.UnlockProgress(state, l, f)
		p = ns.PoseNeeds(state, l, rung)
	elseif stage and ns.CanRungUp(l, rung, f, state) then
		need = ns.StageDays(stage)
		have = ns.TopDays(state, l, rung, slot.move)
	end
	if (not need or need == 0) and (not counts or rung >= ns.HeldTop(l)) and ns.GoalAsk(state, slot.move) then
		return "goal"
	end
	if not need or need == 0 then return "next" end
	return "step", ns.RungMove(l, rung + 1, f), have, need, p
end

-- The top of a move's own window on its strength ladder, the least count at which a move taken as the easier version
-- stops holding back the moves it stood in for (Swap, Engine.lua; the fold's `easier` mark and its `easierUntil`,
-- DB.lua easierMark; ns.SteppedDown) and the record at which a move of the rung below leaves the pumps (ns.AtTop); nil
-- for a move with none (a pose, a cardio form, a move off the ladders), which a pump never offers above the player's
-- step.
function ns.EasierTop(moveId)
	local m = ns.Library[moveId]
	if not (m and STRENGTH[m.ladder]) then return nil end
	local lo, hi, own = ns.Window(m.ladder, m.rung, moveId)
	if lo and ns.OwnUnit(moveId, ns.Anchor(m.ladder, m.rung), own) then return hi end
	return nil
end

-- Whether a move's record holds that top (ns.EasierTop): a move of the rung below a placed ladder's rung that does
-- leaves the pumps (Engine.lua candidates, 2026-10-14); false with no record, and for a move with no such top (its
-- window counted in another unit, a pose, a cardio form, a move off the ladders).
function ns.AtTop(state, moveId)
	local top, best = ns.EasierTop(moveId), ns.Record(state, moveId)
	return top ~= nil and best ~= nil and best >= top
end

-- Whether a move is held back because the player stepped down from it (David, 2026-10-05: "I had to press easier
-- version. And then at the next quest it tried again to make me do pike push ups"): a move the player has never done
-- (no record) whose named regression (its `regress` chain) leads to a move marked by an easier set (the fold's
-- `easier` mark, DB.lua easierMark), or to `cleared`, a mark the set being read has just cleared. A move the player
-- has done (a record, or one the player cleared: the fold's `cleared`) is never held, the marked one least of all: the
-- ladder's own rules set its level, so no hold locks a player out of a ladder they stand on. A move the player stepped
-- down to comes until a set of it reaches the mark's count (Engine.lua candidates; its top, or a step past a record at
-- the top), which clears its mark.
local function stepped(state, moveId, cleared)
	if ns.Record(state, moveId) ~= nil or (state.cleared and state.cleared[moveId]) then return false end
	local x, seen = ns.Library[moveId].regress, {}
	while x and not seen[x] do -- `regress` names a real move (tools/lint.lua)
		local rec = state.moves[x]
		if x == cleared or (rec ~= nil and rec.easier ~= nil) then return true end
		seen[x] = true
		x = ns.Library[x].regress
	end
	return false
end

function ns.SteppedDown(state, moveId)
	return stepped(state, moveId, nil)
end

-- Whether a family offers a first move: its first rung has one the filters allow that the player has not stepped down
-- from (ns.RungMove, ns.SteppedDown; `cleared` as there), so its "opened" event names a family a pump can bring.
local function offersFirst(state, l, f, cleared)
	return ns.RungMove(l, 1, f, function(id) return stepped(state, id, cleared) end) ~= nil
end

-- What one more piece of the setup would open (slice 22: "a pull-up bar opens 7 pull moves up to orange"): for each
-- equipment key not checked, and for Floor moves and Room to move while off, how many moves pass the filters with it
-- that do not without (on `ladder`, or in the whole library) and the best colour among them; the best colour first,
-- then the most moves. Empty when no single piece opens anything more.
function ns.SetupOpens(profile, state, ladder)
	local f = ns.FilterContext(profile, state)
	local out = {}
	local function with(change)
		local g = { gear = {} }
		for k, v in pairs(f) do
			if k ~= "gear" then g[k] = v end
		end
		for k, v in pairs(f.gear) do g.gear[k] = v end
		change(g)
		return g
	end
	local function try(key, g)
		local n, best = 0, -1
		for id, m in pairs(ns.Library) do
			if (not ladder or m.ladder == ladder) and not ns.Passes(id, f) and ns.Passes(id, g) then
				n, best = n + 1, math.max(best, ns.MoveQuality(id) or -1)
			end
		end
		if n > 0 then out[#out + 1] = { key = key, count = n, quality = best } end
	end
	for _, key in ipairs(ns.Gear) do
		if not f.gear[key] then try(key, with(function(g) g.gear[key] = true end)) end
	end
	for _, key in ipairs({ "floor", "space" }) do
		if not f[key] then try(key, with(function(g) g[key] = true end)) end
	end
	table.sort(out, function(a, b)
		if a.quality ~= b.quality then return a.quality > b.quality end
		if a.count ~= b.count then return a.count > b.count end
		return a.key < b.key
	end)
	return out
end

-- Whether every entry of family `l`'s opens but `skip` is done (ns.NeedDone).
local function othersDone(state, l, skip)
	for _, o in ipairs(ns.Ladders[l].opens) do
		if o ~= skip and not ns.NeedDone(state, o) then return false end
	end
	return true
end

-- The families a rung of `ladder` opens: those with an entry of their opens that is a step of `ladder` its climb does
-- (no count and no move: power's cardio step) between the rung left and the rung reached, every other entry done, and
-- that offer a first move (offersFirst: one the setup shuts opens in silence, its column on the Talents tab says why;
-- one the player stepped down from opens when the hold ends, freed).
local function opened(state, ladder, from, to, f)
	local list = {}
	local was, now = ns.Step(ladder, from) or 0, ns.Step(ladder, to) or 0
	for _, l in ipairs(ns.STRENGTH_ORDER) do
		for _, o in ipairs(ns.Ladders[l].opens or EMPTY) do
			if o.ladder == ladder and not o.move and not select(2, stepAt(o)) and was < o.step and now >= o.step
				and othersDone(state, l, o) and offersFirst(state, l, f) then
				list[#list + 1] = l
				break
			end
		end
	end
	return list
end

-- Whether a set did entry `o` of a family's opens: a pose's goal pose named with its `mark` when this hold counted
-- its first day at that mark (2026-10-16: the planche's wrist lean), a strength step whose rung its move stands for
-- (stepFloor) when this set took its record from under that move's floor to it or above while no other such move's
-- record was there.
local function didEntry(state, o, row)
	if o.mark then
		if row.move ~= o.move or row.kind ~= K.flow then return false end
		local d = state.marks and state.marks[o.move]
		local ts = d and (o.mark >= E.mark.perfect and d.perfectTs or d.goodTs)
		return ts == row.ts and ns.MarkDays(state, o.move, o.mark) == 1
	end
	local rec = state.moves[row.move]
	local value = row.kind == K.hold and row.seconds or row.reps
	if not (rec and rec.best == value and rec.bestTs == row.ts) then return false end
	local lo = stepFloor(o, row.move)
	return lo ~= nil and value >= lo and (rec.prev or 0) < lo and not stepDone(state, o, row.move)
end

-- The families a set opens: those with an entry of their opens this set did (didEntry), every other entry done, not
-- yet placed and offering a first move (offersFirst).
local function openedBy(state, row, f)
	local list = {}
	for _, l in ipairs(ns.STRENGTH_ORDER) do
		local lad = state.ladders[l]
		if not (lad and (lad.rung or lad.placedTs)) then
			for _, o in ipairs(ns.Ladders[l].opens or EMPTY) do
				if didEntry(state, o, row) and othersDone(state, l, o) and offersFirst(state, l, f) then
					list[#list + 1] = l
					break
				end
			end
		end
	end
	return list
end

-- The families a set that cleared its easier mark (the fold's `unmarked`, DB.lua easierMark) gives back: open, not yet
-- placed, whose first move that mark held (ns.SteppedDown) and that offer one now, so a family that opened in silence
-- behind the hold is named when its first move can come (one named before the player stepped down is named again).
local function freed(state, row, f, skip)
	local list = {}
	if not row.unmarked then return list end
	for _, l in ipairs(ns.STRENGTH_ORDER) do
		local lad = state.ladders[l]
		if ns.Ladders[l].opens and not skip[l] and not (lad and (lad.rung or lad.placedTs)) and ns.Open(state, l)
			and offersFirst(state, l, f) and not offersFirst(state, l, f, row.move) then
			list[#list + 1] = l
		end
	end
	return list
end

-- Stars: the anchor's record passing ⌈lo + ⅓ (hi − lo)⌉, ⌈lo + ⅔ (hi − lo)⌉ and hi (full push-ups 6/30: 14, 22, 30).
function ns.StarLine(lo, hi)
	local line = {}
	for i, frac in ipairs(C.star_fractions) do
		line[i] = math.ceil(lo + frac * (hi - lo) - 1e-9)
	end
	return line
end

function ns.Stars(ladder, rung, moveId, value)
	local lo, hi = ns.Window(ladder, rung, moveId)
	if not lo or not value then return 0 end
	local n = 0
	for _, step in ipairs(ns.StarLine(lo, hi)) do
		if value >= step then n = n + 1 end
	end
	return n
end

-- Records and targets --------------------------------------------------------------------------------------------

function ns.Record(state, moveId)
	local m = state.moves[moveId]
	return m and m.best, m and m.last
end

-- Whether a move holds a record the player sees (the Records tab) and can clear (Settings.lua): a rep or hold move's
-- best, or a countable cardio form's counted one (2026-10-05); a move that turned timed (the carries, 2026-10-01) keeps
-- its old rows but holds none.
function ns.HoldsRecord(state, moveId)
	local m = ns.Library[moveId]
	return m ~= nil and ns.Record(state, moveId) ~= nil and (m.dose == "rep" or m.dose == "hold" or m.count ~= nil)
end

-- Those moves by the ladder they climb or scale by (ns.ScaleLadder; 0 off the ladders), in ns.LADDER_ORDER
-- with the moves off the ladders last, each group by rung, then id: { { ladder, moves = { ids } } }.
function ns.RecordedMoves(state)
	local groups = {}
	for id in pairs(state.moves) do
		if ns.HoldsRecord(state, id) then
			local m = ns.Library[id]
			local l = m.ladder or ns.ScaleLadder(m) or 0
			groups[l] = groups[l] or {}
			table.insert(groups[l], id)
		end
	end
	local out = {}
	local function add(l)
		local ids = groups[l]
		if not ids then return end
		table.sort(ids, function(a, b)
			local ra, rb = ns.Library[a].rung or 0, ns.Library[b].rung or 0
			if ra ~= rb then return ra < rb end
			return a < b
		end)
		out[#out + 1] = { ladder = l, moves = ids }
	end
	for _, l in ipairs(ns.LADDER_ORDER) do add(l) end
	add(0)
	return out
end

-- A move's first typed count and when it came, then its record and when that came ("8 → 23 since 1 Nov"); nil for a
-- move without one. A record older than the first count kept (the ring dropped it) dates from the record.
function ns.RecordSpan(state, moveId)
	local m = moveId and state.moves[moveId]
	if not (m and m.best) then return nil end
	return m.first or m.best, m.firstTs or m.bestTs or 0, m.best, m.bestTs or 0
end

-- One step of progress from `value`: a rep on a rep move; on a hold 5 % of it, 2 to 5 s (a flat 5 s was a quarter of a
-- 20-s hold and a twentieth of a 100-s one; plank times vary about 14 % day to day, Bohannon 2018). Integers only,
-- so the phone engines compute the same step.
function ns.RecordStep(moveId, value)
	if ns.Library[moveId].dose ~= "hold" then return C.goal_rep_step end
	local step = math.floor((C.goal_hold_pct * (value or 0) + 99) / 100)
	return math.min(C.goal_hold_max_s, math.max(C.goal_hold_min_s, step))
end

-- Whether a set at `ts` is fresh: no rep or hold set on the move's strength ladder (off the ladders, on the move
-- itself) for goal_fresh_h, or none since the game day began and none for goal_night_h, a night's sleep (a player who
-- plays until 20:00 and from 08:00 meets every ladder fresh in the morning). A set to near-failure leaves the muscles
-- below their best for hours (Moran-Navarro 2017, Raastad & Hallen 2000), so a repeat the same evening is no fair
-- record attempt.
local function lastSetAt(state, moveId)
	local ladder = ns.Library[moveId].ladder
	if STRENGTH[ladder] then
		local l = state.ladders and state.ladders[ladder]
		return l and l.lastTs
	end
	local m = state.moves[moveId]
	return m and m.lastTs
end

function ns.FreshAt(state, moveId, ts)
	local prev = lastSetAt(state, moveId)
	if prev == nil or ts - prev >= C.goal_fresh_h * 3600 then return true end
	return ns.NewSession(prev, ts)
end

-- The night rule: a set on another game day, goal_night_h or more after the set at `prev` (a night's sleep), starts a
-- new session. Sets at 03:50 and 04:10 fall on two game days but in one evening.
function ns.NewSession(prev, ts)
	return ns.day(prev) ~= ns.day(ts) and ts - prev >= C.goal_night_h * 3600
end

-- The first moment a set starts a new session after the set at `prev` (ns.NewSession): the next game day's start or
-- goal_night_h after `prev`, whichever is later.
local function sessionFrom(prev)
	return math.max(ns.NextDayStart(prev), prev + C.goal_night_h * 3600)
end

-- When a set of a move is fresh again (ns.FreshAt; a step's card, slice 36): nil while one at `now` is, else the
-- first moment one is, goal_fresh_h after the last set on its strength ladder (off the ladders, on the move itself)
-- or a new session after it, whichever comes first.
function ns.FreshFrom(state, moveId, now)
	if ns.FreshAt(state, moveId, now) then return nil end
	local prev = lastSetAt(state, moveId)
	return math.min(prev + C.goal_fresh_h * 3600, sessionFrom(prev))
end

-- The record sets the move's strength ladder took on the game day of `now` (off the ladders, the move's own sets: a
-- countable cardio form's steady sets among them, which tire it as its counted ones do).
function ns.SetsToday(state, moveId, now)
	local day = ns.day(now)
	local ladder = ns.Library[moveId].ladder
	if STRENGTH[ladder] then
		local l = state.ladders and state.ladders[ladder]
		return l and l.days and l.days[day] or 0
	end
	local d = state.days and state.days[day]
	return d and d.done and d.done[moveId] or 0
end

-- The weight of a move's fresh count `i` in tenths of a kg, 0 for a set without one: `freshLoad` runs beside `fresh`
-- (DB.lua recordCount), and a baseline from before the weights (2026-10-09) kept none for the counts it holds.
local function freshLoadAt(m, i)
	local loads = m.freshLoad or {}
	return loads[i - (#(m.fresh or {}) - #loads)] or 0
end

-- The player's form on a move: the best of its last goal_form_n fresh counts (DB.lua keeps them, and a count above the
-- form on a tired set with them), else the record; a placement starts them again. Then the last of them, the one
-- "+2 on your last fresh set" compares with, and its weight (nil for a set without one). None where the record came
-- before the move's form changed and no set since has started a form (DB.lua ns.BeforeForm, 2026-10-19: plank
-- shoulder taps' feet together), so its goal is a max set (ns.Goal) and its record and stars stay.
function ns.Form(state, moveId)
	local m = state.moves[moveId]
	if not (m and m.best) then return nil end
	local form, lastFresh = nil, nil
	for _, v in ipairs(m.fresh or {}) do
		if not form or v > form then form = v end
		lastFresh = v
	end
	if not form and ns.BeforeForm(moveId, m.bestTs or 0) then return nil end
	local load = lastFresh and freshLoadAt(m, #m.fresh) or m.lastLoad or 0
	return form or m.best, lastFresh or m.last, load > 0 and load or nil
end

-- Loaded moves (2026-10-09, David: "Let people put their exact weight (kgs) and how many reps they made and suggest
-- the progressive overload next time"). A weight is an integer in tenths of a kg everywhere (12.5 kg is 125), and so is
-- every number below, so the phone cores and the desktop app plan and judge alike.

-- A move's typed weight: "total", "each" (a weight in each hand, the kg of one), or nil for a move that takes none.
function ns.Loaded(moveId)
	local m = ns.Library[moveId]
	return m and m.load or nil
end

-- Whether a loaded move's set at `load` (tenths of a kg, nil for none) counts toward a climb (2026-10-12, research P6:
-- 20 one-arm rows with a 2-kg bag climbed the row ladder to the table row, which takes about 27 kg an arm): a move
-- with a weight range climbs (its window's top on its days, a plateau's day, a placement at the top: DB.lua
-- anchorAttempt, ns.Judge) only at the top of that range or more (kg[2], each hand's for "each"); any other move at any
-- set. Records, goals and stars read every set as before.
function ns.LoadClimbs(moveId, load)
	local m = ns.Library[moveId]
	if not (m and m.load and m.kg) then return true end
	return (load or 0) >= m.kg[2] * 10
end

-- The weight a loaded move's set climbs at (ns.LoadClimbs) in whole kg and whether that is each hand's, for the lines
-- that name its top ("20 with 15 kg or more"); nil for a move that climbs at any set.
function ns.ClimbKg(moveId)
	local m = ns.Library[moveId]
	if not (m and m.load and m.kg) then return nil end
	return m.kg[2], m.load == "each"
end

-- The gear that is a real weight (the makeshift loads are a bag, a jug, a chair: no gear at all).
local IMPLEMENTS = { kettlebell = true, dumbbells = true, barbell = true, vest = true, backpack = true }

-- The gear that carries a move's load for a player with `gear`: the first of its `holds` the player has, else its own
-- need where that is a weight (one of a list of them: the first the player has); nil for a makeshift load.
function ns.Implement(moveId, gear)
	local m = ns.Library[moveId]
	for _, key in ipairs(m.holds or {}) do
		if gear[key] then return key end
	end
	for _, need in ipairs(m.needs or {}) do
		if type(need) ~= "table" then
			if IMPLEMENTS[need] then return need end
		else
			for _, key in ipairs(need) do
				if IMPLEMENTS[key] and gear[key] then return key end
			end
		end
	end
	return nil
end

-- A set's strength: Epley's estimated one-rep max (kg × (1 + reps / 30), Epley 1985) times load_epley, an integer.
function ns.LoadScore(load, reps)
	return load * (C.load_epley + reps)
end

-- The reps at `load` that match a strength score, never under 0: at the set's own weight its own count.
function ns.RepsAt(score, load)
	return math.max(0, math.floor(score / load) - C.load_epley)
end

-- The reps a loaded move's weight goes up at: its rung window's top where it stands for its rung or has a window of
-- its own, so the weight and the rung climb at the same top; else load_top_reps.
function ns.LoadTop(moveId)
	local m = ns.Library[moveId]
	local _, hi, own = ns.Window(m.ladder, m.rung, moveId)
	if hi and (own or ns.StandsFor(m.ladder, m.rung, moveId)) then return hi end
	return C.load_top_reps
end

-- The next weight up from `load`: a kettlebell's next size (none bigger: the same), a barbell's two small plates, any
-- other weight load_step_pct of it in whole kg, at least load_step_kg (ACSM 2009: 2-10 %).
function ns.LoadUp(moveId, load, gear)
	local implement = ns.Implement(moveId, gear)
	if implement == "kettlebell" then
		for _, kg in ipairs(C.load_bells) do
			if kg * 10 > load then return kg * 10 end
		end
		return load
	end
	if implement == "barbell" then return load + C.load_bar_step_kg * 10 end
	return load + math.max(C.load_step_kg * 10, math.floor((load * C.load_step_pct + 999) / 1000) * 10)
end

-- A typed weight, or nil for none (nil or 0).
local function weight(load)
	return load and load > 0 and load or nil
end

-- The weight box's - (`dir` < 0) and + (`dir` > 0), one rule on every surface (2026-10-10): an empty box goes to the
-- planned weight (`plan`), else the start range's low end (a kettlebell's first size at or above it); a kettlebell
-- steps through its sizes and stays at the smallest and the biggest; any other weight load_box_step tenths of a kg, to
-- the next half kilo from an odd weight, never under load_box_step nor over load_max_kg.
function ns.LoadStep(moveId, load, dir, gear, plan)
	local bell = ns.Implement(moveId, gear) == "kettlebell"
	load = weight(load)
	if not load then
		if plan then return plan end
		local lo = ns.Library[moveId].kg[1] * 10
		if bell then
			for _, kg in ipairs(C.load_bells) do
				if kg * 10 >= lo then return kg * 10 end
			end
		end
		return lo
	end
	if bell then
		local below, above
		for _, kg in ipairs(C.load_bells) do
			if kg * 10 < load then below = kg * 10 end
			if kg * 10 > load and not above then above = kg * 10 end
		end
		if dir > 0 then return above or load end
		return below or load
	end
	local step = C.load_box_step
	if dir > 0 then return math.min(C.load_max_kg * 10, load - load % step + step) end
	return math.max(step, load % step == 0 and load - step or load - load % step)
end

-- A loaded move's form in strength: the strongest fresh set with a weight (a tie the later) as its score and weight;
-- none: the weighted record's (nil before one).
local function loadForm(m)
	local formScore, formLoad
	for i, v in ipairs(m.fresh or {}) do
		local l = freshLoadAt(m, i)
		local score = l > 0 and ns.LoadScore(l, v)
		if score and (not formScore or score >= formScore) then formScore, formLoad = score, l end
	end
	if not formScore and m.topLoad then return ns.LoadScore(m.topLoad, m.topReps), m.topLoad end
	return formScore, formLoad
end

-- The weight a loaded move's next set at `now` is planned with, and whether it is a step up; nil before any set with
-- a weight. The working weight is the last fresh set's that had one (else the last set's; a tired set above the form
-- counts among the fresh ones, DB.lua recordCount); on a fresh set it goes up (ns.LoadUp) once the last load_top_sets
-- fresh sets were all at it and at the top (ns.LoadTop), as ACSM's "one to two
-- repetitions over the desired number on two consecutive training sessions" (Ratamess 2009, p. 690), a step up where
-- there is a heavier weight (the biggest kettlebell has none) and the form's strength still makes load_floor_reps
-- there (a jump too big for it, the next kettlebell or a barbell's 2.5 kg on a light weight, waits while the reps climb
-- on); a repeat keeps the day's weight, the last set's. Third, the working weight a step up came from.
function ns.LoadPlan(state, moveId, now, gear)
	local m = state.moves[moveId]
	if not m then return nil end
	local fresh, working = m.fresh or {}, nil
	for i = 1, #fresh do
		local load = freshLoadAt(m, i)
		if load > 0 then working = load end
	end
	working = working or m.lastLoad
	if not working then return nil end
	if not ns.FreshAt(state, moveId, now) then return m.lastLoad or working, false end
	local n, top = C.load_top_sets, ns.LoadTop(moveId)
	if #fresh < n then return working, false end
	for i = #fresh - n + 1, #fresh do
		if freshLoadAt(m, i) ~= working or fresh[i] < top then return working, false end
	end
	local up = ns.LoadUp(moveId, working, gear)
	if up <= working or ns.RepsAt((loadForm(m)), up) < C.load_floor_reps then return working, false end
	return up, true, working
end

-- The goal at a weight on a move with a weighted record (ns.Goal's rule in strength): the reps at `load` that match
-- the strongest fresh set with a weight (a tie the later; none: the weighted record), one step past them at that set's
-- weight or lighter, so at its own weight the conversion is exact and the goal is the rule by reps ("record" when
-- the goal's strength passes the weighted record, as ns.Goal's kind by reps); a heavier weight asks the same strength
-- and no step more ("heavier"). A repeat asks its share of those reps, never more than a step
-- past the reps at `load` that match the last set when it had a weight.
local function loadGoal(state, moveId, now, load, m)
	local topScore = ns.LoadScore(m.topLoad, m.topReps)
	local formScore, formLoad = loadForm(m)
	if ns.FreshAt(state, moveId, now) then
		local reps = ns.RepsAt(formScore, load)
		if load <= formLoad then reps = reps + ns.RecordStep(moveId, reps) end
		local goal = math.max(1, reps)
		return goal, load > formLoad and "heavier" or ns.LoadScore(load, goal) > topScore and "record" or "form"
	end
	local pcts = C.goal_repeat_pct
	local pct = pcts[math.min(#pcts, math.max(1, ns.SetsToday(state, moveId, now)))]
	local goal = math.max(1, math.floor(pct * ns.RepsAt(formScore, load) / 100))
	if m.lastTs and m.lastLoad and now - m.lastTs < C.goal_fresh_h * 3600 then
		local last = ns.RepsAt(ns.LoadScore(m.lastLoad, m.last), load)
		goal = math.min(goal, last + ns.RecordStep(moveId, last))
	end
	return goal, "repeat"
end

-- A loaded move's weighted record and its last set with a weight: topLoad, topReps, lastLoad and the last set's reps
-- (each nil when none; the last pair nil when the last set had no weight).
function ns.LoadRecord(state, moveId)
	local m = state.moves[moveId]
	if not m then return nil end
	return m.topLoad, m.topReps, m.lastLoad, m.lastLoad and m.last or nil
end

-- A loaded move's form in strength as a set (loadForm): its weight and the reps that strength makes there, as a
-- "form" goal's line names it ("your best lately: 10 kg × 12"); nil before a weighted record.
function ns.LoadForm(state, moveId)
	local m = state.moves[moveId]
	if not (m and m.topLoad) then return nil end
	local score, load = loadForm(m)
	return load, ns.RepsAt(score, load)
end

-- A fresh goal from a move's form before the hold ledger's raise: on a hold of the ledger's hold ladders (slice 52,
-- ns.HOLD_GOAL) from goal_hold_long_from_s of form the multiple of goal_hold_round_s nearest the form × 1.10 (a tie
-- up), raised by 5 s while under the form + goal_hold_min_s and lowered by 5 s while over the form +
-- goal_hold_long_max_s: about +5 s to 85 s and +10 s above, numbers players type (David typed the goal plus a round
-- number; tools/sim.lua walks it across the windows); under it, and on every other move, the form plus one step
-- (ns.RecordStep).
function ns.FreshGoal(moveId, form)
	local m = ns.Library[moveId]
	if not (ns.HOLD_GOAL[m.ladder] and m.dose == "hold") or form < C.goal_hold_long_from_s then
		return form + ns.RecordStep(moveId, form)
	end
	local five = C.goal_hold_round_s
	local goal = math.floor((form * (100 + C.goal_hold_long_pct) + 50 * five) / (100 * five)) * five
	while goal < form + C.goal_hold_min_s do goal = goal + five end
	while goal > form + C.goal_hold_long_max_s do goal = goal - five end
	return goal
end

-- The goal of a rep or hold move (David, 2026-10-03: "a progressive overload best practice algorithm for it and not
-- just +1"), and its kind; nil without a record (a first max set: whatever is typed is the record), and nil where the
-- move's form changed since its record and no set has started a form since (ns.Form: a max set again). Fresh, the best
-- of the last fresh sets (ns.Form) plus one step (ns.FreshGoal: a hold of the Stamina Ledger's hold ladders steps in
-- fives, and catches up with what its holds show, ns.HoldRaise, slice 52) ("record" when the goal passes the record,
-- so a hold's step from a lower form makes it a record attempt too; "form" when the record came on a better day and
-- the goal reaches it at most: an adaptive goal is hit more often than a fixed one, Adams 2022, and a moderate goal
-- works best, Kyllo & Landers 1995); a "repeat" before goal_fresh_h, a share of that
-- form that falls with every set the ladder took that game day (goal_repeat_pct: 90 % on the second set, 70 % from the
-- sixth; a long day brings a ladder back every major_h, and two or three hard sets a day are what a muscle uses,
-- Remmert 2025, so the later ones stay real work the player can still reach) and never more than one step past the
-- move's last set. A record still pays whenever a
-- count passes it (ns.Classify); the goal is the least, never the most. A loaded move's goal at a weight (`load`,
-- tenths of a kg) once it has a weighted record follows the same rule in strength (loadGoal, above). `warm` (slice
-- 48's double-check: a match pump's warm move, Engine.lua dose) takes the repeat share on a fresh set too, as the
-- ladder's second set would (90 %), an easy first effort after a long sit, never a record step; a warm move takes no
-- typed weight (ns.StandingLeg), so loadGoal never meets one.
function ns.Goal(state, moveId, now, load, warm)
	local m = state.moves[moveId]
	if not (m and m.best) then return nil end
	local form = ns.Form(state, moveId)
	if not form then return nil end -- its form changed since the record (ns.Form): a max set
	load = weight(load)
	if load and m.topLoad then return loadGoal(state, moveId, now, load, m) end
	if not warm and ns.FreshAt(state, moveId, now) then
		local goal = ns.HoldRaise(state, moveId, ns.FreshGoal(moveId, form), now)
		return goal, goal > m.best and "record" or "form"
	end
	local pcts = C.goal_repeat_pct
	local pct = pcts[math.min(#pcts, math.max(1, ns.SetsToday(state, moveId, now)))]
	local goal = math.max(1, math.floor(pct * form / 100))
	if m.lastTs and now - m.lastTs < C.goal_fresh_h * 3600 then
		goal = math.min(goal, m.last + ns.RecordStep(moveId, m.last))
	end
	return goal, "repeat"
end

-- The next fresh set of a loaded move after the sets so far: its planned weight and its goal at it, goal_fresh_h from
-- `now` (the Result's "Next time: 11 kg, 15+ reps"); nil for a move without a set with a weight.
function ns.NextLoad(state, moveId, now, gear)
	local at = now + C.goal_fresh_h * 3600
	local load = ns.LoadPlan(state, moveId, at, gear)
	if not load then return nil end
	return load, (ns.Goal(state, moveId, at, load))
end

-- What a fresh pick of a rep or hold move (its slot's `kind`) asks at `now` (Engine.lua dose; slice 36's Start label
-- reads it too, so the card and the pump agree): a loaded rep move planned at a weight with `gear` (ns.LoadPlan; a
-- placement keeps the working weight, since a break is no consecutive session for a step up, 2026-10-10) and its goal
-- at it, "heavier" on a step up; no goal on a placement or a move without a record (a max set). Returns the goal, its
-- kind, the weight in tenths of a kg, whether it is a step up and the weight it steps up from (each nil where none).
-- `warm`: a match pump's warm move's easy goal (ns.Goal). `f` the filter context (its gear, and its Body switches: a
-- fresh hold goal after A lot leaps, ns.HoldLeap, never with Avoid breath-hold strain on).
function ns.PickGoal(state, moveId, kind, now, f, placing, warm)
	local load, loadUp, loadFrom
	if kind == K.rep and ns.Loaded(moveId) then
		load, loadUp, loadFrom = ns.LoadPlan(state, moveId, now, f.gear)
		if placing and loadUp then load, loadUp, loadFrom = loadFrom, false, nil end
	end
	if placing then return nil, nil, load, loadUp, loadFrom end
	local target, goal = ns.Goal(state, moveId, now, load, warm)
	if target and loadUp then goal = "heavier" end
	local leap = target and goal ~= "repeat" and not f.noStrain and ns.HoldLeap(state, moveId)
	if leap and leap > target then
		target = leap
		goal = target > state.moves[moveId].best and "record" or "form"
	end
	return target, goal, load, loadUp, loadFrom
end

-- A break: after 14 days without a finished move, the first anchor attempt on each strength and hold ladder is a
-- placement set ("back from a break: find your level") that never lowers the record. The cardio ladder's since slice 53
-- (2026-10-23): after 14 days without a cardio move (its lastTs), on a step above the first, whose first measure holds
-- it or takes it a step down (DB.lua cardioMove).
function ns.Placing(state, ladder, now)
	local cardio = ladder == ns.CARDIO
	local l = cardio and state.cardio or not cardio and state.ladders and state.ladders[ladder]
	if cardio and not (l and l.lastTs and ns.Rung(state, ladder) > 1) then return false end
	if not (cardio or l and l.placedTs) then return false end -- a ladder never placed meets its first max set anyway
	if l.placing then return true end
	local last = cardio and l.lastTs or state.lastDoneTs
	return last ~= nil and (now or ns.now()) - last >= C.placement_idle_days * 86400
end

-- The outcome of a typed count. `opts` is the slot (placing, target, counted) or nil. Above the record a record; a rep
-- set past 90 s is a lower bound (one stopwatch over both sides of a move counted in pairs, so half of it; a side
-- move's the weaker side's), below the record never a miss; then a win of its own: the goal reached, or more than the
-- last fresh set (never a tired one the same day, which a fresh set would pass for free, but one above the form, which
-- joins the fresh ones). A counted set (`opts.counted`, 2026-10-16) has no goal and no win of its own: under its record
-- it is a miss ("Best: 14"), since a 30-s count moves by about two reps from day to day and "one more than last time"
-- asked for shorter reps. With a weight (`load`, tenths of a kg) on a move that has a weighted record, "past the
-- record" and "more than the last fresh set" (where that set had a weight too) compare strength (ns.LoadScore), so a
-- heavier set of fewer reps can beat both; a first weighted set compares its reps and starts the weighted record.
-- `opts.target` is the goal at the weight the set was done with.
function ns.Classify(state, moveId, value, elapsed, opts, load)
	if not value or value <= 0 then return O.done end
	local best = ns.Record(state, moveId)
	if not best then return O.placement end
	load = weight(load)
	local rec = state.moves[moveId]
	local past
	if load and rec.topLoad then
		past = ns.LoadScore(load, value) > ns.LoadScore(rec.topLoad, rec.topReps)
	else
		past = value > best
	end
	if opts and opts.placing then return past and O.beat or O.placement end
	if past then return O.beat end
	if opts and opts.counted then return O.miss end
	local m = ns.Library[moveId]
	-- one set's stopwatch covers both sides of a move counted in pairs (a rep on each side a count); a side move's is
	-- the weaker side's, each side on its own run
	if m.dose == "rep" and elapsed and elapsed / (m.pairs and 2 or 1) > C.censor_s then return O.censored end
	if opts and opts.target and value >= opts.target then return O.lastbeat end
	local _, lastFresh, lastLoad = ns.Form(state, moveId)
	if lastFresh then
		local more
		if load and lastLoad then
			more = ns.LoadScore(load, value) > ns.LoadScore(lastLoad, lastFresh)
		else
			more = value > lastFresh
		end
		if more then return O.lastbeat end
	end
	return O.miss
end

-- A slot's count goal and the goal's kind (ns.Goal): none on a counted set (2026-10-16: "All out, clean reps", no
-- number to reach; its own countGoal and countKind until then, since slot.target is a timed slot's seconds to the
-- helper and the phones).
function ns.SlotGoal(slot)
	if slot.counted then return nil, nil end
	return slot.target, slot.goal
end

-- The most one set may count (the box's limit and ns.Validate's "max"): a counted set's count, a hold's seconds, reps.
function ns.CountMax(slot)
	return slot.counted and C.count_max or slot.kind == K.hold and C.max_seconds or C.max_reps
end

-- The top a first max set is checked against: the rung's hi (a variant's cap); none on a counted set (2026-10-16: its
-- form's count window meant only that it is countable since; a count over count_max asks again).
function ns.FirstTop(slot)
	return slot.hi or slot.cap or nil
end

-- The goal a typed count is judged by: the slot's own at its planned weight, and with a weight other than the planned
-- one (`load`, tenths of a kg, nil or 0 none) the goal at that weight at `now` (the set's time, ns.now() when nil); nil
-- on a max set and a counted set. The Result view's "Goal reached" reads it too.
function ns.TargetAt(state, slot, load, now)
	if slot.counted then return nil end
	load = weight(load)
	if load ~= slot.load and slot.target then return (ns.Goal(state, slot.move, now or ns.now(), load)) end
	return slot.target
end

-- The count as stored: a placement on an anchor that reaches the rung's hi ends there ("enough: next rung") and is
-- stored censored at hi (a loaded move's only at the weight its climb asks, ns.LoadClimbs: a lighter set climbs
-- nothing, so its count stays as typed), judged against ns.TargetAt's goal with the weight typed (`load`) at `now`. A
-- counted set (2026-10-16) is judged with no goal, its count optional (none: done), and one saved "Form slipped" or,
-- since slice 53, "Not all out" (`slipped`: any answer but Yes, to the end) is done whatever its count: it keeps no
-- record and counts no clean day (DB.lua cardioMove). Returns the outcome and the value.
function ns.Judge(state, slot, value, elapsed, load, now, slipped)
	if slot.counted and slipped then return O.done, value end
	load = weight(load)
	local opts = slot
	if slot.counted then
		opts = { counted = true }
	elseif load ~= slot.load and slot.target then
		opts = { target = ns.TargetAt(state, slot, load, now) }
	end
	local outcome = ns.Classify(state, slot.move, value, elapsed, opts, load)
	if slot.hi and slot.maxSet and value and value >= slot.hi and ns.LoadClimbs(slot.move, load) then
		return O.censored, slot.hi
	end
	return outcome, value
end

-- Input validation beyond the box's limits: "max" over one set, "really" for a count far above the record (2 ×, or
-- 1.5 × once the record is 15 or more) or a first max set above 2 × the rung's hi (`hi`, or `cap` on a variant), "low"
-- for a count above 0 under half the record (David, 2026-10-04: "if it's extreme like 5 and record was 28 then you can
-- ask for a confirmation and thats it"); "really" and "low" are asked once, and kept as typed. With a weight (`load`,
-- tenths of a kg): "max" above load_max_kg, "heavy" above really_load × the slot's planned weight (its own question,
-- "18 kg is far above the 8 kg planned", asked once as "really" is), and on a move with a weighted record "really"
-- and "low" compare with the record's reps at that weight (ns.RepsAt; none under one rep).
function ns.Validate(state, slot, value, load)
	if value > ns.CountMax(slot) then return "max" end
	load = weight(load)
	if load and load > C.load_max_kg * 10 then return "max" end
	if load and slot.load and load > C.really_load * slot.load then return "heavy" end
	local best = ns.Record(state, slot.move)
	if not best or slot.placing then
		local hi = ns.FirstTop(slot)
		if hi and value > C.really_placement * hi then return "really" end
		return nil
	end
	local m = state.moves[slot.move]
	if load and m.topLoad then best = ns.RepsAt(ns.LoadScore(m.topLoad, m.topReps), load) end
	if best < 1 then return nil end
	local r = C.really_record
	if value > (best >= r.from and r.high or r.low) * best then return "really" end
	if value > 0 and value < C.bad_day * best then return "low" end
	return nil
end

-- Whether a count entered is too short to count (min_reps, min_seconds): 1 or 2 reps, a counted set's too, or a hold
-- under 7 s; 0 is no count.
function ns.ShortCount(slot, value)
	return value > 0 and value < (slot.kind == K.hold and C.min_seconds or C.min_reps)
end

-- A first max set: a placement (the Trial's or a break's) or a max set of a move with no record yet. Its Ready view is
-- a first appearance (Panel.lua), and nothing there is short: a count under min_reps or min_seconds saves as "Could
-- not do one" (Events.lua Lc.Enter) and a hold's Done under min_seconds goes on to its count (Lc.Done), where a bank
-- would ask the same set at every pump, the ladder unplaced and the queue growing (2026-11-03; the desktop app reads
-- it as `PlannedMove.firstMax`).
function ns.FirstMaxSet(state, slot)
	return slot.placing == true or (slot.maxSet == true and ns.Record(state, slot.move) == nil)
end

-- "Could not do one" (a hold: "Could not hold it"), the stored 0 of a strength or hold move, recorded at once: with a
-- record and no placement under way it is a miss; else a placement of 0, which places the ladder a rung lower
-- (ns.Progression). Returns the outcome.
function ns.CouldNot(state, slot)
	if ns.Record(state, slot.move) and not slot.placing then return O.miss end
	return O.placement
end

-- XP a finished move pays (slice 3's fold uses the same rule): its class times its rung, plus the win on top. The
-- rung's share stops at rung_cap (rung 8, x 1.7; slice 22: the ladders grew past eight rungs, and no rung pays twice
-- a rung-1 move). The typed count never enters XP; a first record pays the move, never the record bonus. `opts`:
-- { rung, allout, firstTry, owed }.
function ns.MoveXP(moveId, outcome, paidOwed, opts)
	opts = opts or {}
	local m = ns.Library[moveId]
	local x = C.xp_move
	local base
	if opts.allout then
		base = x[3]
	elseif m.dose == "cardio" then
		base = x.cardio
	elseif m.dose == "flow" then
		base = x.flow
	else
		base = x[m.class]
	end
	-- a pose pays at its rung as every ladder move does (slice 21); a flow move off the ladders (a carry, a roller) at
	-- 1
	local rung = (m.dose == "flow" and not m.ladder) and 1 or opts.rung or m.rung or 1
	local xp = base * (1 + x.rung_step * (math.min(rung, x.rung_cap) - 1))
	if outcome == O.beat then
		xp = xp + C.xp_record
	elseif outcome == O.lastbeat then
		xp = xp + C.xp_lastbeat
	end
	if opts.firstTry then xp = xp + C.xp_first_try end
	if paidOwed then xp = xp + C.xp_owed_paid end
	return math.floor(xp + 0.5)
end

-- The XP curve: xpToNext(L) = round(70 × L ^ 1.1), capped at 60.
function ns.XPToNext(level)
	return math.floor(C.xp_base * level ^ C.xp_exp + 0.5)
end

function ns.LevelFor(xp)
	local level = 1
	while level < C.level_cap and xp >= ns.XPToNext(level) do
		xp = xp - ns.XPToNext(level)
		level = level + 1
	end
	return level, xp
end

-- The ledger's scale ---------------------------------------------------------------------------------------------

-- The Stamina Ledger (slice 52) keeps every number whole, so the game, the desktop app and both phones reach the same
-- one: seconds and counts become centi-logs, ns.LN[s] = ⌊100 ln s + ½⌋ for s from 1 to 1000 (the phones read this
-- table from contract/library.json, never their own logarithm), and ns.EXP(x) is the s whose centi-log lies nearest
-- x, the lower on a tie and 1 at or under 0. ns.ISqrt(n) is ⌊√n⌋.
local LN = {}
for s = 1, 1000 do LN[s] = math.floor(100 * math.log(s) + 0.5) end
ns.LN = LN

function ns.EXP(x)
	if x <= 0 then return 1 end
	local lo, hi = 1, #LN
	while lo < hi do
		local mid = math.floor((lo + hi) / 2)
		if LN[mid] >= x then hi = mid else lo = mid + 1 end
	end
	if lo > 1 and x - LN[lo - 1] <= LN[lo] - x then return lo - 1 end
	return lo
end

function ns.ISqrt(n)
	return math.floor(math.sqrt(n))
end

-- A layoff (Rekindle: sprint-interval gains held at 4 weeks off and were gone by 8), one rule for both ledgers and the
-- Long Haul (DB.lua cardioMove): after `gap` game days away, over ledger_layoff_days, the share of a gain it keeps, in
-- ledger_fade_days' units, all of it at the layoff's edge and none ledger_fade_days days later; nil for a shorter gap.
function ns.LayoffKeep(gap)
	if gap <= C.ledger_layoff_days then return nil end
	return math.max(0, C.ledger_fade_days - (gap - C.ledger_layoff_days))
end

-- An estimate `x` after a layoff that keeps `keep` (ns.LayoffKeep): its gain over its prior `x0` fades; one at or under
-- the prior stays (the double-check of slices 52-55, 2026-10-24: a layoff fades a gain and never lifts a struggling
-- player's estimate, which the doubt each ledger adds beside it lowers).
local function layoffFade(x, x0, keep)
	if x <= x0 then return x end
	return x0 + math.floor((x - x0) * keep / C.ledger_fade_days)
end

-- The hold ledger (slice 52) ----------------------------------------------------------------------------------------

-- The ladders the Stamina Ledger reads sets on, by the row kind it reads: the wall sit, the core, the side and the
-- L-sit their holds (HOLD_GOAL: their goal steps in fives, catches up with the ledger and asks "Could you have held
-- longer?"), power its reps with the same maths, for its "Now" and its forecast alone. Overpower acts on all five
-- (DB.lua anchorAttempt).
local HOLD_GOAL = { [LAD.wallsit] = true, [LAD.core] = true, [LAD.side] = true, [LAD.lsit] = true }
local HOLD_LEDGER = { [LAD.wallsit] = K.hold, [LAD.core] = K.hold, [LAD.side] = K.hold, [LAD.lsit] = K.hold,
	[LAD.power] = K.rep }
ns.HOLD_GOAL, ns.HOLD_LEDGER = HOLD_GOAL, HOLD_LEDGER
local LONGER = E.longer

-- The multiple of goal_hold_round_s at or under `s`, and the one at or above it.
local function fivesDown(s)
	return s - s % C.goal_hold_round_s
end

local function fivesUp(s)
	return s + (-s) % C.goal_hold_round_s
end

-- A set's game day as a number (ns.DayIndex), the unit of the ledger's gaps.
local function gameDay(ts)
	return ns.DayIndex(ns.day(ts))
end

-- A ladder's entry, as DB.lua's fold keeps it in state.ledger.holds: `u` the estimate, the centi-logs the player's set
-- lies below the top of the move's own window (ns.EasierTop; 0 at the top), `v` its doubt (a variance in centi-logs²),
-- `b10` its trend (tenths of a centi-log a game day), `day` the game day (ns.DayIndex) of the last set read, `u0` the
-- rung's prior (ns.HoldPrior), `ask` the game day the hold ask was last answered, `lot` the set answered A lot (in u's
-- units) until the ladder's next fresh set (ns.HoldLeap), `prior` true from a rung change until the next set read
-- (ns.HoldClimb, ns.HoldRead); nil before the ladder's first set.
function ns.HoldEntry(state, ladder)
	local ledger = state.ledger
	return ledger and ledger.holds and ledger.holds[ladder]
end

-- The top of `move`'s own window (ns.EasierTop) where the hold ledger reads its sets on `ladder`: a move of the
-- ladder's step (the player's rung) in the ledger's row kind (power's reps, the others' holds) with a top of its own;
-- nil otherwise. The fold reads a set by it (DB.lua holdLedger), and the goal's raise, leap and ask, the card's Now and
-- Pro's date act by it.
function ns.LedgerTop(state, ladder, move)
	local m = ns.Library[move]
	if not (m and HOLD_LEDGER[ladder] and HOLD_LEDGER[ladder] == K[m.dose] and m.ladder == ladder
		and m.rung == ns.Rung(state, ladder)) then
		return nil
	end
	return ns.EasierTop(move)
end

-- The estimate a rung starts from: the middle of its window, in centi-logs below its top.
function ns.HoldPrior(ladder, rung)
	local lo, hi = ns.Window(ladder, rung)
	return math.floor((LN[lo] + LN[hi]) / 2) - LN[hi]
end

-- The estimate carried to game day `day` (80 % of the trend a day, for at most the layoff's days) and the gap in
-- game days.
local function predict(e, day)
	local gap = math.max(0, day - e.day)
	return e.u + math.floor((e.b10 * math.min(gap, C.ledger_layoff_days) * 4 + 25) / 50), gap
end

-- The spread of an exact count's reading at the entry's doubt.
local function spread(e)
	local r = C.ledger_hold_noise[1]
	return ns.ISqrt(e.v + r * r)
end

-- How a set reads (item 6): an answer to the hold ask first (No two-sided at the set, A little two-sided 14 above it,
-- A lot one-sided 26 above it); then a set that is no fresh one (a repeat in the session, a warm set) one-sided at the
-- set, which only raises; a max set (`goal` nil), a placement, a miss and a beat more than goal_hold_round_s past the
-- multiple of 5 at or above its goal two-sided; a beat near its goal and a max set stopped at the top (censored) the
-- lift, a stop that says the player had at least that. `value` the set as stored, `answer` its ns.Enum.longer code.
-- Returns "lot", "little", "no", "one", "two" or "lift" (ns.HoldRead).
function ns.HoldHow(value, goal, outcome, fresh, answer)
	if answer == LONGER.lot then return "lot" end
	if answer == LONGER.little then return "little" end
	if answer == LONGER.no then return "no" end
	if not fresh then return "one" end
	if outcome == O.censored then return "lift" end
	if not goal or outcome == O.placement or value < goal or value > fivesUp(goal) + C.goal_hold_round_s then
		return "two"
	end
	return "lift"
end

-- One set at `ts` read into entry `e` (nil: the ladder's first, which starts it at the set with doubt ledger_hold_v0
-- and the prior `u0`): `typed` its seconds (power's reps), `hi` the top of its move's own window, `how` as ns.HoldHow
-- reads it. Whole numbers and floor division only, as the phones port it, after the prototype that replayed 968
-- traced events (docs/research/endurance-2026-10-23.md, "The model"). Returns the entry, changed in place.
function ns.HoldRead(e, ts, typed, hi, how, u0)
	local day = gameDay(ts)
	local x = LN[math.min(#LN, math.max(1, typed))] - LN[hi]
	local r = C.ledger_hold_noise[typed % C.goal_hold_round_s == 0 and 2 or 1]
	if not e then
		e = { u = x, v = C.ledger_hold_v0, b10 = 0, day = day, u0 = u0 }
	else
		local uOld = e.u
		local u, gap = predict(e, day)
		local v = e.v
		local keep = ns.LayoffKeep(gap)
		if keep then -- a layoff: a gain fades back to the prior
			u = layoffFade(u, e.u0, keep)
			v = v + C.ledger_hold_layoff_v
		end
		v = math.min(C.ledger_hold_v_max, v + C.ledger_hold_drift * gap)
		local k = math.floor(1000 * v / (v + r * r))
		if how == "lot" or how == "one" then
			local y = how == "lot" and x + C.ledger_hold_lot or x
			if y > u then
				u = u + math.floor((k * (y - u) + 500) / 1000)
				v = math.max(C.ledger_hold_v_min, math.floor(v * (1000 - math.floor(k / 2)) / 1000))
			end
		elseif how == "lift" then
			local s = ns.ISqrt(v + r * r)
			local z4 = math.floor((8 * (x - u) + s) / (2 * s))
			if z4 >= -6 then
				local eff = u + math.floor((s * C.ledger_lam[math.min(20, z4 + 8) + 1] + 500) / 1000)
				u = u + math.floor((math.floor(k / 2) * (eff - u) + 500) / 1000)
				v = math.max(C.ledger_hold_v_min, math.floor(v * (1000 - math.floor(k / 4)) / 1000))
			end
		else -- two-sided: No, A little, a miss, a max set, a beat far past its goal
			local y = how == "little" and x + C.ledger_hold_little or x
			local excess = math.abs(y - u) - 2 * ns.ISqrt(v + r * r)
			if excess > 0 then -- a surprise widens the doubt first
				v = math.min(C.ledger_hold_v_max, v + excess * excess)
				k = math.floor(1000 * v / (v + r * r))
			end
			u = u + math.floor((k * (y - u) + 500) / 1000)
			v = math.max(C.ledger_hold_v_min, math.floor(v * (1000 - k) / 1000))
		end
		local rise = math.floor((10 * (u - uOld) + math.floor(gap / 2)) / math.max(1, gap))
		e.b10 = math.max(0, math.min(C.ledger_hold_trend_max, math.floor((3 * e.b10 + rise + 2) / 4)))
		e.u, e.v, e.day, e.prior = u, v, day, nil
	end
	if how == "lot" or how == "little" or how == "no" then e.ask = day end
	if how ~= "one" then e.lot = how == "lot" and x or nil end
	return e
end

-- A rung change (a climb, a step down, an undo): the entry starts at the new rung's prior `u0` with doubt
-- ledger_hold_climb_v and no trend, `prior` until a set of the new rung is read; its last day and its ask stay.
function ns.HoldClimb(e, u0)
	e.u, e.u0, e.v, e.b10, e.lot, e.prior = u0, u0, C.ledger_hold_climb_v, 0, nil, true
	return e
end

-- The ledger's raise (item 7): a fresh `goal` of a hold of the player's step on a hold ladder becomes at least the
-- multiple of 5 at or under the least of the move's top, ledger_hold_raise_pct % of the goal and what the holds show
-- (the estimate ledger_hold_band[1] / 100 spreads below its middle, about 70 % sure), so a player whose holds show more
-- than the goal gets one that catches up, never past the window's top.
function ns.HoldRaise(state, moveId, goal, now)
	local m = ns.Library[moveId]
	local hi = HOLD_GOAL[m.ladder] and ns.LedgerTop(state, m.ladder, moveId)
	local e = hi and ns.HoldEntry(state, m.ladder)
	if not e then return goal end
	local u = predict(e, gameDay(now))
	local shows = ns.EXP(LN[hi] + u - math.floor(C.ledger_hold_band[1] * spread(e) / 100))
	return math.max(goal, fivesDown(math.min(hi, math.floor(goal * C.ledger_hold_raise_pct / 100), shows)))
end

-- The leap (item 7): after A lot, the ladder's next fresh goal on a hold of the player's step is at least the
-- multiple of 5 at or under the least of the move's top and ledger_hold_raise_pct % of that set (in the move's own
-- window, so a variant of the step leaps as far); nil where none. ns.PickGoal gives it, never with Avoid breath-hold
-- strain on.
function ns.HoldLeap(state, moveId)
	local m = ns.Library[moveId]
	local hi = HOLD_GOAL[m.ladder] and ns.LedgerTop(state, m.ladder, moveId)
	local e = hi and ns.HoldEntry(state, m.ladder)
	if not (e and e.lot) then return nil end
	return fivesDown(math.min(hi, math.floor(ns.EXP(LN[hi] + e.lot) * C.ledger_hold_raise_pct / 100)))
end

-- Whether a hold set asks "Could you have held longer?" (item 8): a fresh goal (never a repeat, a warm set or a max
-- set) on a hold of the player's step on a hold ladder, beaten by at most goal_hold_round_s past the multiple of 5 at
-- or above it (`value` the set as judged), while the ladder's doubt is ledger_ask_v or more, once a ladder a game day
-- (the day of `now`); never on power. The answer goes in the hold row's reps (ns.ResultRow).
function ns.HoldAsk(state, slot, value, now)
	local l = slot.ladder
	if not (HOLD_GOAL[l] and slot.kind == K.hold and slot.target and not slot.warm) then return false end
	if slot.goal ~= "record" and slot.goal ~= "form" then return false end
	if not value or value < slot.target or value > fivesUp(slot.target) + C.goal_hold_round_s then return false end
	if not ns.LedgerTop(state, l, slot.move) then return false end
	local e = ns.HoldEntry(state, l)
	return e ~= nil and e.v >= C.ledger_ask_v and e.ask ~= gameDay(now or ns.now())
end

-- The entry that speaks for `move` on a ledger ladder's step (ns.HoldNow, ns.HoldTopIn), with that move's top: a move
-- whose sets the ledger reads (ns.LedgerTop), so never a rep anchor of the core's steps 6-11 or the side's step 5; and
-- an entry that read a set since its last rung change (`prior`: a prior with no set of its own behind it names no
-- number). Nil otherwise.
local function stepEntry(state, ladder, move)
	local e = ns.HoldEntry(state, ladder)
	local hi = e and not e.prior and move and ns.LedgerTop(state, ladder, move)
	if not hi then return nil end
	return e, hi
end

-- What the player can hold now of `move` on a ledger ladder's step (item 9, the step card's "Now"; stepEntry): the
-- estimate at `now` and ledger_hold_band[2] / 100 spreads either side, in the move's own unit on its own top (seconds,
-- power's reps), as lo, middle and hi; nil where no entry speaks for it and while `move` has no record, whose first set
-- sets its mark (Tree.CardNow; a placement's climb or a variant's set gives the entry a number before it).
function ns.HoldNow(state, ladder, move, now)
	local e, hi = stepEntry(state, ladder, move)
	if not (e and ns.Record(state, move)) then return nil end
	local u = predict(e, gameDay(now or ns.now()))
	local band = math.floor(C.ledger_hold_band[2] * spread(e) / 100)
	return ns.EXP(LN[hi] + u - band), ns.EXP(LN[hi] + u), ns.EXP(LN[hi] + u + band)
end

-- When `move`'s top on the player's step comes at the trend (item 9, Pro's Path; no game screen shows it):
-- ledger_hold_due_pct % of the game days the trend takes to close the estimate's gap to the top, rounded up, as half of
-- it, it and twice it; 0, 0, 0 at the top already, nil where no entry speaks for it (stepEntry) and while the trend is
-- not known (none rising).
function ns.HoldTopIn(state, ladder, move, now)
	local e = stepEntry(state, ladder, move)
	if not e then return nil end
	local u = predict(e, gameDay(now or ns.now()))
	if u >= 0 then return 0, 0, 0 end
	if e.b10 <= 0 then return nil end
	local days = math.floor((math.floor(-u * 10 * C.ledger_hold_due_pct / 100) + e.b10 - 1) / e.b10)
	return math.floor(days / 2), days, days * 2
end

-- The steady ledger (slice 52) ------------------------------------------------------------------------------------

-- The steady side of the Stamina Ledger (docs/plan.md slice 52, "The steady ledger"), which took the cardio progression
-- P's place: how long the player keeps cardio's steady pace, `state.ledger.steady`, folded from the steady rows (DB.lua
-- steadyRow: a cardio row of the ladder, not counted, `reps` 0, so never an all-out window nor a Long Haul) and kept in
-- the baseline. Whole numbers and floor division: `a` the centi-log seconds the player keeps rung 1's pace, a form of
-- rung r ledger_rung_ln harder a rung, c(r) = a - ledger_rung_ln × (r - 1); `a0` the prior, ledger_prior_s at the
-- cardio step the ledger starts on; `d` the doubt (ledger_doubt: its start, floor and top); `day` the last play day
-- (ns.LedgerDay); `askDay` and `asks` the last answer's game day and the answers on it; `bumpDay` the game day of the
-- last finish not asked that moved it; `mHigh` the best M played in steps of 50, never lowered (the cardio stat's
-- steps, Stats.lua), its last step's game day `mDay`; `ring` the last ledger_plays play days' { day, a }, oldest first.
-- The full moves it gives are the cardio state's (`full`, fullTs: DB.lua steadyRow), counts of the step as its clean
-- days are.

-- The full move's seconds, cardio_cap_s × 1.5 (45 s), a full quest's steady move at the top M and the most Keep going
-- runs to: a full move (DB.lua steadyRow) runs them, by its dose or Keep going's.
ns.FULL_MOVE_S = math.floor(C.cardio_cap_s * C.ledger_m[2] / 1000)

-- A steady form's `span`, how many times the plain steady length it runs (a pad walk's 2; Rapha, 2026-11-06). The
-- ledger measures the plain length, so its seconds are a row's over its span and its full move the plain one's span
-- times over.
function ns.Span(id)
	local m = ns.Library[id]
	return m and m.span or 1
end

-- The full move of form `id`: what a full quest at the top M runs it for, and what Keep going runs it on to.
function ns.FullMove(id)
	return ns.FULL_MOVE_S * ns.Span(id)
end

-- The span of steady row `r` of form `m`: 1 on a row from before the form ran longer (ledger_span_ts).
function ns.RowSpan(r, m)
	return r.ts >= C.ledger_span_ts and m.span or 1
end

-- The rung of a cardio step's own steady forms (slice 52: the steady pool and its cadence line top at rung 6, so from
-- step 8 a timed step's own forms are rung 6's): the rung whose M a full move and the pace ask's full-move reason
-- read, the card's Pace and the ledger's prior.
function ns.SteadyRung(rung)
	return math.min(rung, 6)
end

-- The steady pool on cardio rung `rung` under the filters `f`, the forms a steady move of the rung draws from
-- (Engine.lua's candidates; the week's omen on the setup, ns.Omen): the cardio forms that run steady (none all-out
-- alone, `endOnly`) of the step's own steady rung (ns.SteadyRung) and the two below it, all at its spoken cadence;
-- when the filters leave none there, those of the rungs further down to the first that holds one. Listed rung by rung
-- from the top.
function ns.SteadyForms(rung, f)
	local list, top = {}, ns.SteadyRung(rung)
	for level = top, 1, -1 do
		if #list > 0 and level < top - 2 then break end
		for id, m in pairs(ns.Library) do
			if m.ladder == ns.CARDIO and m.rung == level and not m.endOnly and ns.Passes(id, f) then
				list[#list + 1] = id
			end
		end
	end
	return list
end

-- A ledger at its prior on cardio step `rung`: ledger_prior_s on the step's own steady forms.
function ns.SteadyLedger(rung)
	local a0 = LN[C.ledger_prior_s] + C.ledger_rung_ln * (ns.SteadyRung(rung) - 1)
	return { a = a0, a0 = a0, d = C.ledger_doubt[1], asks = 0, mHigh = C.ledger_m[1], ring = {} }
end

-- The state's steady ledger; before its first steady row one at its prior on the cardio step now.
function ns.Steady(state)
	local L = state.ledger and state.ledger.steady
	return L or ns.SteadyLedger(ns.Rung(state, ns.CARDIO))
end

-- The ledger's day of `ts`: its game day as a day number (ns.DayIndex), so gaps are plain subtraction.
function ns.LedgerDay(ts)
	return ns.DayIndex(ns.day(ts))
end

-- The ledger's `a` and `d` on play day `day`: each new play day the doubt grows by a third of the game days since the
-- last one, and a gap over ledger_layoff_days fades a gain over the prior (ns.LayoffKeep, layoffFade: an `a` at or
-- under it stays), all of it after ledger_fade_days more days away, with ledger_layoff_doubt more doubt. The fold ages
-- the ledger so before each steady row (DB.lua steadyRow), and the dose, the ask and the card read it aged to the day
-- they look at, so the first move after a layoff is already the lower one.
function ns.SteadyAged(L, day)
	local a, d = L.a, L.d
	if L.day == nil or day <= L.day then return a, d end
	local gap, top = day - L.day, C.ledger_doubt[3]
	d = math.min(top, d + math.floor(gap / 3))
	local keep = ns.LayoffKeep(gap)
	if keep then
		a = layoffFade(a, L.a0, keep)
		d = math.min(top, d + C.ledger_layoff_doubt)
	end
	return a, d
end

-- The seconds a form of rung `r` keeps its pace by `a`, less the dose's margin (ledger_margin) and a quarter of the
-- doubt: EXP(c(r) - 21 - ⌊d/4⌋), what a dose of M takes as a full quest's base would run it.
local function steadyKeeps(a, d, r)
	return ns.EXP(a - C.ledger_rung_ln * (r - 1) - C.ledger_margin - math.floor(d / 4))
end

-- M from `a` and `d` for a form of rung `r`, in thousandths: those seconds over a full quest's base (cardio_cap_s),
-- in ledger_m's range, so the dose sits at about 0.80 of what the player keeps and never under the quest's base.
function ns.SteadyMOf(a, d, r)
	local m = math.floor(1000 * steadyKeeps(a, d, r) / C.cardio_cap_s)
	return math.min(C.ledger_m[2], math.max(C.ledger_m[1], m))
end

-- M, the steady dose's multiplier (ns.CardioSeconds, ns.AlloutSeconds), for a form of rung `r` at `now` (the ledger
-- aged to its game day).
function ns.SteadyM(state, r, now)
	local a, d = ns.SteadyAged(ns.Steady(state), ns.LedgerDay(now or ns.now()))
	return ns.SteadyMOf(a, d, r)
end

-- The step card's Pace on a timed cardio step `rung` (slice 52, Tree.lua): the seconds its own forms keep their pace
-- now less the margin, at most the full move (EXP(c(step) - 21 - ⌊d/4⌋), the move a full quest gives within its
-- range), and the full move's seconds.
function ns.SteadyPace(state, rung, now)
	local a, d = ns.SteadyAged(ns.Steady(state), ns.LedgerDay(now or ns.now()))
	return math.min(ns.FULL_MOVE_S, steadyKeeps(a, d, ns.SteadyRung(rung))), ns.FULL_MOVE_S
end

-- Whether the pace ask follows a steady move of `move` dosed at cardio rung `rung` for `seconds` at `now` (slice 52;
-- once a game day on the player's own steps 1-4 until then, slice 42): when there is room (no answer yet this game day,
-- or a doubt of ledger_ask_doubt[2] or more with fewer than ledger_asks_day answers today) and a reason (a doubt of
-- ledger_ask_doubt[1] or more; or a form of the player's timed step's pool dosed at the step for the full move's
-- seconds while the step's own M is at the top, its full moves are owed and one can count now, ns.DayIn: the move
-- whose Yes is a full move, DB.lua steadyRow; a shorter one counts a full move only by Keep going, which saves its own
-- answer; or ledger_ask_days game days since the last answer). In the replays about 0.5 asks a play day while it
-- learns, 0.17 once it is sure. The pump asks once (Engine.lua paceSlot), after a steady move of any step that ran its
-- time, never after an all-out window, a warm move, the stand-up move or a Long Haul (slice 55).
function ns.LedgerAsk(state, move, rung, seconds, now, f)
	local L = ns.Steady(state)
	local day = ns.LedgerDay(now)
	local a, d = ns.SteadyAged(L, day)
	if L.askDay == day and (L.asks >= C.ledger_asks_day or d < C.ledger_ask_doubt[2]) then return false end
	if d >= C.ledger_ask_doubt[1] then return true end
	local step = ns.Rung(state, ns.CARDIO)
	local own = ns.SteadyRung(step)
	if rung == step and seconds >= ns.FullMove(move) and ns.Library[move].rung <= own and not ns.CountedRung(step, f)
		and ns.SteadyMOf(a, d, own) == C.ledger_m[2] then
		local have, need = ns.UnlockProgress(state, ns.CARDIO, f)
		if have < need and ns.DayIn(state, ns.CARDIO, now, f) == nil then return true end
	end
	return L.askDay == nil or day - L.askDay >= C.ledger_ask_days
end

-- Whether game day `i` before `now`'s (0 its own) had a move, the game's or an app's: a play day (the forecast's).
local function played(state, now, i)
	local t = state.days and state.days[ns.day(ns.DayMid(now, -i))]
	return t ~= nil and (t.moves or 0) + (t.appMoves or 0) > 0
end

-- The play days among the `n` game days to `now`'s, its own among them.
local function playDays(state, now, n)
	local k = 0
	for i = 0, n - 1 do
		if played(state, now, i) then k = k + 1 end
	end
	return k
end

-- The forecast's play rate, per mille: the share of game days with a move among the last 28 to `now`, from the
-- oldest of them with one (the double-check of slices 52-55, 2026-10-24: the days before a newcomer's first move
-- count for nothing, where a daily newcomer read as playing one day in five); 0 with none.
local function playRate(state, now)
	local n, span = 0, 0
	for i = 0, 27 do
		if played(state, now, i) then n, span = n + 1, i + 1 end
	end
	return span > 0 and math.floor(n * 1000 / span) or 0
end

-- `plays` play days in game days at the play rate (per mille, at least 200), as Pro's Path says its date: { lo, mid,
-- hi }, half to twice its middle.
local function playSpan(state, now, plays)
	local mid = math.floor(plays * 1000 / math.max(200, playRate(state, now)))
	return { math.floor(mid / 2), mid, mid * 2 }
end

-- The play days the climb from counted cardio step `rung` (ns.CountedRung) still takes, its clean days `have` of
-- `need` (ns.UnlockProgress): the clean days to come at the clean days so far per play day since the step's climb
-- (climbTs) or a break after it (breakTs, DB.lua cardioBreak), each of which started its clean days again, rounded up,
-- and one more while the climb owes its proving window (ns.ProvingNeeded), so one where only the proving waits; nil
-- while its path is out (`out`, ns.PathIn), on a climb back (ns.Returning: its return days) and while the rate is not
-- known (no clean day yet, fewer than three play days since the climb or the break, or no climb dating the step).
local function cleanPlays(state, now, rung, f, have, need, out)
	local c = state.cardio
	if out ~= 0 or have == 0 or not (c and c.climbTs) or ns.Returning(state, ns.CARDIO, rung) then return nil end
	local since = playDays(state, now, ns.LedgerDay(now) - ns.LedgerDay(math.max(c.climbTs, c.breakTs or 0)) + 1)
	if since < 3 then return nil end
	local plays = math.floor(((need - have) * since + have - 1) / have)
	return ns.ProvingNeeded(state, rung, f) and plays + 1 or plays
end

-- Pro's Path's date for the climb from the player's cardio step (slice 52; the phones' and the computer's, never shown
-- in the game): on a timed step from the rise of `a` over the ring's play days, the play days until the step's own
-- forms keep the full move past the margin and the full moves still owed; on a counted step the play days its clean
-- days and proving window still take (cleanPlays, 2026-10-24); in game days (playSpan); nil where the climb is not
-- open, and while the rise is not known (fewer than three play days in the ring, or no rise) or a counted step's rate.
function ns.SteadyForecast(state, now, f)
	local rung = ns.Rung(state, ns.CARDIO)
	local have, need, out = ns.UnlockProgress(state, ns.CARDIO, f)
	if need == 0 then return nil end
	if ns.CountedRung(rung, f) then
		local plays = cleanPlays(state, now, rung, f, have, need, out)
		return plays and playSpan(state, now, plays)
	end
	local L = ns.Steady(state)
	local ring = L.ring
	if #ring < 3 then return nil end
	local a, d = ns.SteadyAged(L, ns.LedgerDay(now))
	local short = LN[ns.FULL_MOVE_S] + C.ledger_margin + math.floor(d / 4)
		- (a - C.ledger_rung_ln * (ns.SteadyRung(rung) - 1))
	local plays = need - have
	if short > 0 then
		local per = math.floor((ring[#ring][2] - ring[1][2]) / (#ring - 1))
		if per <= 0 then return nil end
		plays = math.floor((short + per - 1) / per) + plays
	end
	return playSpan(state, now, plays)
end

-- The Trial -------------------------------------------------------------------------------------------------------

-- How many of the Trial's ladders (ns.TrialOrder) have their first record and how many the Trial holds: with a filter
-- context, a ladder the filters leave no move for (the wall sit with a knee flag) waits outside the count until a move
-- is legal again.
function ns.TrialStatus(state, f)
	local placed, total = 0, 0
	for _, l in ipairs(ns.TrialOrder) do
		local lad = state.ladders and state.ladders[l]
		if lad and lad.placedTs then
			placed, total = placed + 1, total + 1
		elseif not f or ns.RungMove(l, ns.Rung(state, l), f) then
			total = total + 1
		end
	end
	return placed, total
end

-- The game day `n` days after the one `ts` falls in.
local function addDays(ts, n)
	return ns.day(ns.DayMid(ts, n))
end

-- The one-tap check-in: at the first login on day two after character creation (up to day four) and two days after a
-- 14-day placement, once each.
function ns.CheckinDue(state, now)
	local function due(from, first, last)
		if not from or (state.checkinTs and state.checkinTs >= from) then return false end
		local today = ns.day(now)
		for n = first, last do
			if today == addDays(from, n) then return true end
		end
		return false
	end
	return due(state.breakTs, 2, 4) or (not state.breakTs and due(state.createdTs, 1, 3))
end

-- The countable forms of a cardio rung the setup allows, by id (2026-10-05): the rung climbs on their clean days
-- (ns.CountedRung, 2026-10-16; on their count until then) and its all-out windows take them first (Engine.lua); none,
-- and it climbs by full moves (slice 52; ready days until then).
function ns.CountForms(rung, f)
	local list = {}
	for id, m in pairs(ns.Library) do
		if m.count and m.ladder == ns.CARDIO and m.rung == rung and ns.Passes(id, f) then list[#list + 1] = id end
	end
	table.sort(list)
	return list
end

-- Proving the next step (slice 53, 2026-10-23: a climb means "you can already do the next step"): the climb from
-- counted cardio rung `rung` (ns.CountedRung) asks for one clean window of the step above once its clean days and path
-- are in, where that step has a countable form of its own the setup allows (ns.CountForms with the setup alone: a pain
-- flag holds the proving as it holds a climb, never skips it) and the climb is no climb back (ns.Returning, whose
-- return_days alone bring it); else the climb comes with its days, as before.
function ns.ProvingNeeded(state, rung, f)
	return ns.CountedRung(rung, f) and #ns.CountForms(rung + 1, setupOf(f)) > 0
		and not ns.Returning(state, ns.CARDIO, rung)
end

-- The proving window's forms on the player's counted step `rung`: the step above's own countable forms the filters
-- `f` pass (pain flags and all) that are open (ns.MoveOpen), its anchor first, then by id (Engine.lua provingMove).
function ns.ProvingForms(state, rung, f)
	local anchor, list = ns.Anchor(ns.CARDIO, rung + 1), {}
	for _, id in ipairs(ns.CountForms(rung + 1, f)) do
		if ns.MoveOpen(state, id, f) then list[#list + 1] = id end
	end
	table.sort(list, function(a, b)
		if (a == anchor) ~= (b == anchor) then return a == anchor end
		return a < b
	end)
	return list
end

-- Where the proving stands on the player's cardio step at `now` (slice 53), with the form it names (ns.ProvingForms'
-- first): nil where the climb asks none (ns.ProvingNeeded), its days or its path are not in, the climb is shut
-- (ns.CanRungUp) or a pain flag holds it (ns.PainOnLadder), the ladder is back from a break (ns.Placing) or the filters
-- pass no form; else "due" where the next all-out window is the proving (a new session by the night rule since the last
-- counted window at the step, DB.lua cardioMove's countTs: the first window of the next session), "waits" where a try
-- did not pass this session (its proveTs; the card's "No step up yet."), "next" where the days came in this session.
function ns.Proving(state, f, now)
	now = now or ns.now()
	local c = state.cardio
	local rung = ns.Rung(state, ns.CARDIO)
	if not (c and ns.ProvingNeeded(state, rung, f)) or (c.clean or 0) < ns.CleanDays(rung) then return nil end
	if not (ns.CanRungUp(ns.CARDIO, rung, f, state) and ns.PathIn(state, ns.CARDIO, rung, f))
		or ns.PainOnLadder(state, ns.CARDIO, rung, f) then
		return nil
	end
	if ns.Placing(state, ns.CARDIO, now) then return nil end
	local form = ns.ProvingForms(state, rung, f)[1]
	if not form then return nil end
	if not c.countTs or ns.NewSession(c.countTs, now) then return "due", form end
	return c.proveTs and not ns.NewSession(c.proveTs, now) and "waits" or "next", form
end

-- The row a performed move writes (the lifecycle and the simulator build it the same way): a rep count in `reps`, a
-- hold's seconds in `seconds`, a cardio or flow move's seconds done in `seconds` with `reps = 1` marking an all-out
-- window, a counted set that ran its count_s (any outcome but stopped) its count in `reps` (0 none, 2026-10-16) over
-- count_s with the counted bit (2026-10-05); the ladder and rung it was dosed at; flags carry the owed, confirmed and
-- easier bits and, in bits 0-2, `reason`: since 2026-10-04 only an app pump's felt-unwell stop of a running timed move
-- (ns.Enum.stopped.unwell, tools/apppump.lua; the stopped-because row's reasons only in rows written before then); the
-- first-try bit only in rows written before 2026-10-03, which the sheet still pays. `answer` is the player's answer
-- after the move: a pose's to "How close to the goal pose?" (E.mark, the slot's `ask`; 0 none) in a flow row's `reps`
-- with the mark bit, and since 2026-10-16 a counted set's E.answer ("Form slipped", since slice 53 "Not all out"; nil
-- Yes, to the end) and a steady cardio move's to "Did you keep that pace to the end?" (E.pace, the slot's `pace`,
-- 2026-10-17; nil none) or Keep going's (slice 52: its seconds done past the plan) in flags bits 14-15
-- (ns.WithAnswer), where a rep or hold row of a match pump's warm move (the slot's `warm`, slice 48's double-check)
-- carries E.set.warm; since slice 52 a hold's to "Could you have held longer?" (E.longer, ns.HoldAsk; 0 not asked)
-- in its `reps`.
function ns.ResultRow(slot, index, outcome, value, elapsed, confirmed, reason, answer)
	local flags = reason or 0
	if slot.owed then flags = ns.WithFlag(flags, "owed") end
	if confirmed then flags = ns.WithFlag(flags, "confirmed") end
	if slot.easier then flags = ns.WithFlag(flags, "easier") end -- Swap's easier version (in the feedback numbers)
	local reps, seconds = 0, math.floor(elapsed or 0)
	-- a match pump's warm move's easy set (slice 48's double-check: Engine.lua dose) says so, for the fold's fresh sets
	if (slot.kind == K.rep or slot.kind == K.hold) and slot.warm then flags = ns.WithAnswer(flags, E.set.warm) end
	if slot.kind == K.rep then
		reps = value or 0
	elseif slot.kind == K.hold then
		reps, seconds = answer or 0, value or 0
	elseif slot.kind == K.cardio and slot.counted and outcome ~= O.stopped then
		-- a counted set that ran its 30 s (2026-10-05): the count typed (0 none) in reps over count_s, the counted bit
		-- and the window's answer for the fold
		reps, seconds = value or 0, C.count_s
		flags = ns.WithAnswer(ns.WithFlag(flags, "counted"), answer)
	elseif slot.kind == K.cardio then
		-- an all-out window's reps 1, a Long Haul's its planned seconds (slice 55, ns.HaulRow), a steady move's 0
		reps = slot.allout and 1 or slot.haul and slot.seconds or 0
		seconds = math.floor(value or 0)
		if not slot.allout then flags = ns.WithAnswer(flags, answer) end
	elseif slot.kind == K.flow then
		reps, seconds = answer or 0, math.floor(value or 0)
		flags = ns.WithFlag(flags, "mark")
	end
	return { questID = slot.questID, tier = slot.tier, slot = index - 1, ladder = slot.ladder, rung = slot.rung,
		move = slot.move, kind = slot.kind, reps = reps, seconds = seconds, outcome = outcome, flags = flags }
end

-- The load row of a loaded set (E.kind.load, 2026-10-09): the weight `load` (tenths of a kg) of the set row `r`, whose
-- ts, char, questID, tier, slot, ladder, rung and move it repeats, with the set row's source bits (ns.Source); written
-- just before the set row, at its ts (DB.lua's fold reads the set with it).
local SOURCE_BITS = { "app", "watch", "phone", "computer" }

function ns.LoadRow(r, load)
	local flags = 0
	for _, bit in ipairs(SOURCE_BITS) do
		if ns.HasFlag(r.flags, bit) then flags = ns.WithFlag(flags, bit) end
	end
	return { ts = r.ts, char = r.char, questID = r.questID, tier = r.tier, slot = r.slot, ladder = r.ladder,
		rung = r.rung, move = r.move, kind = K.load, reps = load, seconds = 0, outcome = 0, flags = flags }
end

-- The paid row that clears an owed move from the queue: it repeats the owed row's ts, questID and slot. `skipped`
-- when no move paid it: the player skipped it (Lc.Skip), and the row says so in its skipped flag.
function ns.PaidRow(o, skipped)
	local flags = ns.WithFlag(0, "owed")
	if skipped then flags = ns.WithFlag(flags, "skipped") end
	return { ts = o.ts, char = o.char, questID = o.questID, tier = o.tier, slot = o.slot, ladder = o.ladder,
		rung = o.rung, move = o.move, kind = o.kind, outcome = O.paid, flags = flags }
end

-- The rules after a row --------------------------------------------------------------------------------------------

local function rungRow(ts, ladder, rung, cause)
	return { ts = ts, kind = K.rung, ladder = ladder, rung = rung, reps = cause }
end

-- A performed move row: a typed count, or done (a timed move, a pose, an all-out window, a counted set saved with no
-- count or slipped).
local function performedRow(r)
	return (r.kind == K.rep or r.kind == K.hold or r.kind == K.cardio or r.kind == K.flow)
		and (ns.TYPED[r.outcome] or r.outcome == O.done)
end

-- The event of a ladder the setup stops at this rung, or that waits for another ladder's step, with the anchor that
-- would open it and why it is shut (ns.TopOut); nil where nothing is to name.
local function topout(state, ladder, rung, f)
	local shut, why = ns.TopOut(ladder, rung, f, state)
	return shut and { kind = "topout", ladder = ladder, rung = rung, move = shut, why = why } or nil
end

-- The event of a climb whose days are in while its path is not (2026-10-16; its proof's until then): the first stage
-- still out (ns.Stages) with the move to bring to its top, that top (ns.MoveTop: a count, a pose's least answer, true
-- for a countable form's clean window), the game days it takes and those it has, and whether it waits for the move
-- that opens it (`after`, `top` of that one); where the setup with the pain flags leaves the move out, what would open
-- it (ns.Opener, as the top-out line says); nil where a pain flag shuts it or nothing is out.
local function pathEvent(state, l, rung, f)
	for _, st in ipairs(ns.Stages(state, l, rung, f)) do
		if st.state == "open" or st.state == "waiting" then
			local why
			if not ns.Passes(st.move, f) then
				why = ns.Opener(st.move, f)
				if not why then return nil end
			end
			return { kind = "path", ladder = l, rung = rung, move = st.move, value = ns.MoveTop(l, rung, st.move),
				days = st.days, have = st.have, waiting = st.state == "waiting" or nil, after = st.after,
				afterTop = st.top, why = why }
		end
	end
	return nil
end

-- A rung up: its row, its event in its colour (and whether it was earned before, a smaller win: "Rekindled", as the
-- sheet pays it, Stats.lua markReached: before a rung down took it, or before the player started the ladder over; an
-- orange or gold rung not earned yet only opens, `opened` with the count of one set that earns it, ns.EarnFloor),
-- and an event for each family the new rung opens (ns.Open).
local function climb(state, rows, events, ts, l, rung, cause, f)
	rows[#rows + 1] = rungRow(ts, l, rung + 1, cause)
	local move = ns.UnlockMove(l, rung + 1, f, state)
	local toEarn = ns.TopTier(l, rung + 1) and not ns.EarnCounts(state, l, rung + 1)
	local rekindled = not toEarn and (ns.RungReached(state, l, rung + 1) ~= nil or rung + 1 <= ns.BestRung(state, l))
	events[#events + 1] = { kind = "rungUp", ladder = l, rung = rung + 1, move = move,
		quality = ns.Quality(l, rung + 1), rekindled = rekindled, opened = toEarn or nil,
		floor = toEarn and move and ns.EarnFloor(l, rung + 1, move) or nil }
	if rekindled then return end
	for _, family in ipairs(opened(state, l, rung, rung + 1, f)) do
		local lad = state.ladders[family]
		if not (lad and (lad.rung or lad.placedTs)) then events[#events + 1] = { kind = "opened", ladder = family } end
	end
end

-- Whether every move of step `rung` of ladder `l`'s mastery (ns.MasteryMoves with the setup `f` reads; none, and it
-- never is) has reached its top once (ns.TopOnce).
local function allTopped(state, l, rung, f)
	local moves = ns.MasteryMoves(l, rung, f)
	if #moves == 0 then return false end
	for _, id in ipairs(moves) do
		if not ns.TopOnce(state, id) then return false end
	end
	return true
end

-- Whether step `rung` of ladder `l` is mastered now and was not before (2026-10-16, slice 42): its moves all reached
-- their top once (allTopped).
local function mastery(state, l, rung, f)
	return not ns.Mastered(state, l, rung) and allTopped(state, l, rung, f)
end

-- Whether the mastered sweep (ns.MasterySweep) holds step `rung` of ladder `l` mastered in `state`, its row written
-- already or not: a step the ladder has reached whose moves all reached their top once (Correct a count's step line,
-- ns.CorrectionRows, reads it as the sweep after a correction leaves the step).
local function sweeps(state, l, rung, f)
	return ns.Ladders[l] ~= nil and ns.HeldTop(l) > 0 and rung <= ns.BestRung(state, l) and allTopped(state, l, rung, f)
end

-- The mastered rows a state still lacks (2026-10-17, slice 42's review): every step a ladder has reached (ns.BestRung)
-- whose moves the setup `f` offers have all reached their top once (mastery) and that holds no mastered row yet, each
-- its row at `ts`, by ladder id and rung. The game writes them at login, after a setup or exclusion change and after a
-- merge or an import of rows (Events.lua), the apps' engines at load and after a set or a catch-up, so a step whose
-- last top came in a catch-up, in an old save or by a move left out masters too, and so does one whose old row the
-- fold no longer counts (DB.lua masteryCut) once its tops stand on answers (a cardio form's on an answered clean
-- window, the fold's cleanEver: 2026-10-31); such a row shows no toast (a live set's, ns.Progression's, does), the seal
-- and the Deeds count follow the fold.
function ns.MasterySweep(state, f, ts)
	local rows = {}
	for l = 1, ns.LADDER_MAX do
		if ns.Ladders[l] and ns.HeldTop(l) > 0 then
			for r = 1, ns.BestRung(state, l) do
				if mastery(state, l, r, f) then rows[#rows + 1] = { ts = ts, kind = K.mastered, ladder = l, rung = r } end
			end
		end
	end
	return rows
end

-- The steps a result masters (mastery): the step of a performed set's own move where the ladder has reached it
-- (ns.BestRung), and the step whose path names it from the step below (L-sit step 6's one-heel L-sit; 2026-10-17,
-- slice 42's review), and the step each of its climbs brings the ladder to, whose moves may all have reached their
-- tops before (a move of the step above comes within the rung rule). Each its row (kind mastered, after the set's
-- rung rows) and its event, once.
local function masters(state, row, rows, f, events)
	local list = {}
	local function add(l, rung)
		for _, x in ipairs(list) do
			if x.ladder == l and x.rung == rung then return end
		end
		if mastery(state, l, rung, f) then list[#list + 1] = { ladder = l, rung = rung } end
	end
	local m = ns.Library[row.move]
	if performedRow(row) and m and m.ladder and ns.Ladders[m.ladder] and ns.TopOnce(state, row.move) then
		local at = stageOf[row.move]
		for _, rung in ipairs({ m.rung, at and at.rung }) do
			if ns.BestRung(state, m.ladder) >= rung then add(m.ladder, rung) end
		end
	end
	for _, r in ipairs(rows) do
		if r.kind == K.rung and r.rung > ns.BestRung(state, r.ladder) then add(r.ladder, r.rung) end
	end
	for _, x in ipairs(list) do
		rows[#rows + 1] = { ts = row.ts, kind = K.mastered, ladder = x.ladder, rung = x.rung }
		events[#events + 1] = { kind = "mastered", ladder = x.ladder, rung = x.rung, move = row.move }
	end
end

-- The rung rows a counted or timed cardio row calls for and the events (2026-10-16, slice 42): a counted rung
-- (ns.CountedRung) climbs after its clean days (ns.CleanDays, DB.lua cardioMove), with its path (ns.PathIn: the forms
-- it names on their own clean days), and since slice 53 (2026-10-23) on the proving window that passed where it asks
-- one (ns.ProvingNeeded: the row's `proved`); a timed one (steps 1-4, or a rung whose countable forms the setup shuts)
-- after its full moves (cardio_full_days, slice 52: sessions with a Yes dosed at the player's step while the step's
-- own M stood at the top, DB.lua steadyRow; ready days, a kept pace at any length, until then); a climb back to a
-- step held before after return_days of either (cardioProgress). Stops at it on cardio_rung_down game days step it
-- down (slice 53's footing: two in a row until then; a stop leaves the full moves as they are), and so does the
-- return's measure that did not hold the step after a break (the row's `holds` false). A pain flag holds the climb as
-- on every ladder (ns.PainOnLadder, 2026-10-24). `made`: the row counted the last day, so the path still out is said
-- once.
local function cardioRule(state, row, f, rows, events)
	local c = state.cardio
	local rung = c.rung or ns.Rung(state, ns.CARDIO)
	if row.holds == false and rung > 1 then
		rows[#rows + 1] = rungRow(row.ts, ns.CARDIO, rung - 1, ns.RungCause.down)
		events[#events + 1] = { kind = "rungDown", ladder = ns.CARDIO, rung = rung - 1 }
		return
	end
	local have, need = cardioProgress(state, rung, f)
	local due, made = have >= need, ns.CountedRung(rung, f) and row.cleanDay and c.clean == need
	local open = due and ns.CanRungUp(ns.CARDIO, rung, f, state) and not ns.PainOnLadder(state, ns.CARDIO, rung, f)
	local proves = open and ns.ProvingNeeded(state, rung, f)
	if open and ns.PathIn(state, ns.CARDIO, rung, f) and (not proves or row.proved) then
		climb(state, rows, events, row.ts, ns.CARDIO, rung, ns.RungCause.up, f)
	elseif c.stops >= C.cardio_rung_down and rung > 1 then
		rows[#rows + 1] = rungRow(row.ts, ns.CARDIO, rung - 1, ns.RungCause.down)
		events[#events + 1] = { kind = "rungDown", ladder = ns.CARDIO, rung = rung - 1 }
	elseif open and made then
		-- the clean days are in and the path is not: said once, on the window that brought them
		events[#events + 1] = pathEvent(state, ns.CARDIO, rung, f)
	end
end

-- The rows a result calls for (rung rows, a step mastered) and the events worth a toast, read from the state after the
-- fold applied the row (it marks the row: anchor attempt, first placement, star count, the steady ledger's new best M
-- in steps of 50, an easier mark cleared, an orange or gold rung earned, a clean day or a full move). `f` is the
-- filter context.
function ns.Progression(state, row, f)
	local rows, events = {}, {}
	local l = row.ladder
	local function done()
		masters(state, row, rows, f, events)
		return rows, events
	end
	if row.mSteps then events[#events + 1] = { kind = "p", value = ns.Steady(state).mHigh } end
	-- an orange or gold rung this set earned (DB.lua earnMark): the colour's moment, which its climb only opened
	if row.earns then
		events[#events + 1] = { kind = "earned", ladder = l, rung = row.earns, move = row.move,
			quality = ns.Quality(l, row.earns), rekindled = row.rekindles == true }
	end
	local named = {}
	for _, family in ipairs(openedBy(state, row, f)) do
		named[family] = true
		events[#events + 1] = { kind = "opened", ladder = family }
	end
	for _, family in ipairs(freed(state, row, f, named)) do events[#events + 1] = { kind = "opened", ladder = family } end
	if row.placed then
		local placed, total = ns.TrialStatus(state, f)
		events[#events + 1] = { kind = "trial", placed = placed, total = total, move = row.move, value = row.value,
			trial = IN_TRIAL[l] } -- a family outside the Trial (slice 22's) places its ladder and completes nothing
	end
	if row.stars then events[#events + 1] = { kind = "star", ladder = l, stars = row.stars, move = row.move } end
	if row.anchorAttempt and STRENGTH[l] then
		local lad = state.ladders[l]
		local rung = lad.rung or ns.Rung(state, l)
		local lo = ns.Window(l, rung, row.move)
		-- the Trial's first max set: under 5, one rung lower (a count is still the ladder's first record)
		if row.trialPlacement and row.value and row.value < C.placement_low then
			if rung > 1 then rows[#rows + 1] = rungRow(row.ts, l, rung - 1, ns.RungCause.placementLow) end
			return done()
		end
		-- a climb comes by the top on the days it asks (ns.UnlockDays), by a placement at the top (the ladder's first
		-- record or the first after a break, DB.lua anchorAttempt: any other first set at the top counts its first day,
		-- 2026-10-10) or by a plateau, each with its path (ns.PathIn); an orange or gold rung, and one whose anchor is
		-- a skill or tendon-heavy, by the days alone (ns.DaysOnly)
		local daysOnly = ns.DaysOnly(l, rung)
		local cause
		if row.firstPlacement and row.hitHi and not daysOnly then -- stored censored at hi, or /qp undo's count up to it
			cause = ns.RungCause.placementHigh
		elseif lad.hiDays >= ns.UnlockDays(l, rung, state) then
			cause = ns.RungCause.up
		elseif lad.plateau >= C.plateau_attempts and not daysOnly then
			cause = ns.RungCause.plateau
		end
		local down = lo and rung > 1
			and (lad.underLo >= C.rung_down.under_lo or (lad.halfLo or 0) >= C.rung_down.under_half_lo)
		-- the Trial's own first max set below half the window (a 10-s plank against 30) places one rung lower too: a
		-- placement, which lowers the start rung, never a rung lost
		if down and row.trialPlacement then
			rows[#rows + 1] = rungRow(row.ts, l, rung - 1, ns.RungCause.placementLow)
			return done()
		end
		-- only a set of a move of the rung's own level climbs it (2026-10-09; its days are its own, DB.lua
		-- anchorAttempt); `brought` the days at the top this set counted, two on Overpower (slice 52)
		local own = ns.Library[row.move].rung == rung
		local brought = row.topDay and (row.overpower and 2 or 1) or 0
		local open = cause and own and ns.CanClimb(l, rung, f, state) and not ns.PainOnLadder(state, l, rung, f)
		if open and ns.PathIn(state, l, rung, f) then
			climb(state, rows, events, row.ts, l, rung, cause, f)
		elseif down then
			rows[#rows + 1] = rungRow(row.ts, l, rung - 1, ns.RungCause.down)
			events[#events + 1] = { kind = "rungDown", ladder = l, rung = rung - 1 }
		elseif open then
			-- the days are in and the path is not: said once, on the set that brought the last day
			local need = ns.UnlockDays(l, rung, state)
			if brought > 0 and lad.hiDays >= need and lad.hiDays - brought < need then
				events[#events + 1] = pathEvent(state, l, rung, f)
			end
		elseif cause and rung < ns.HeldTop(l) and not ns.PainOnLadder(state, l, rung, f)
			and (cause == ns.RungCause.placementHigh or lad.hiDays - brought < ns.UnlockDays(l, rung, state)
			and (daysOnly or lad.plateau - (row.plateauDay and 1 or 0) < C.plateau_attempts)) then
			-- once, on the set that made the climb due, below the ladder's top (at the top nothing is shut; a plateau
			-- makes no climb by the days alone due): no rung row resets the counts here, and chat takes one warning a
			-- session, naming what would open the rung
			events[#events + 1] = topout(state, l, rung, f)
		elseif row.hitHi and not own and rung < ns.HeldTop(l) and not ns.PainOnLadder(state, l, rung, f) then
			-- a stand-in from below at its window's top counts no day (DB.lua anchorAttempt): the top-out line names
			-- the rung's own anchor the setup shuts (chat takes one warning a session)
			events[#events + 1] = topout(state, l, rung, f)
		end
	elseif STRENGTH[l] and (row.kind == K.rep or row.kind == K.hold) and state.ladders[l] then
		-- another move of the ladder: its day at its top can bring the path the climb still waited for, with the
		-- anchor's days in already; the unlock then comes on this set
		local lad = state.ladders[l]
		local rung = lad.rung or ns.Rung(state, l)
		if #ns.Path(l, rung) > 0 and lad.hiDays >= ns.UnlockDays(l, rung, state)
			and ns.CanClimb(l, rung, f, state) and not ns.PainOnLadder(state, l, rung, f) and ns.PathIn(state, l, rung, f) then
			climb(state, rows, events, row.ts, l, rung, ns.RungCause.up, f)
		end
	elseif row.anchorAttempt and POSE[l] then
		-- a pose ladder (slice 21): a goal pose of its step (ns.Marks, slice 47) answered close enough (ns.MarkNeeds)
		-- on the days ns.PoseNeeds counts (pose_unlock_days, the main pose's and the Perfect ones among them), each a new
		-- session by the night rule (DB.lua markAttempt), opens the next step, with its path (ns.PathIn, 2026-10-16:
		-- the poses it names, each answered as close on unlock_days game days); a pose ladder never goes down, and a
		-- pain flag on it holds the climb as on any ladder
		local lad = state.ladders[l]
		local rung = lad.rung or ns.Rung(state, l)
		if ns.PoseDaysIn(state, l, rung) and not ns.PainOnLadder(state, l, rung, f) then
			local open = ns.CanRungUp(l, rung, f, state)
			if open and ns.PathIn(state, l, rung, f) then
				climb(state, rows, events, row.ts, l, rung, ns.RungCause.up, f)
			elseif open then
				-- the days are in and the path is not: said once, on the hold that brought the last of them
				if poseMadeIn(state, l, rung, row) then events[#events + 1] = pathEvent(state, l, rung, f) end
			elseif rung < ns.HeldTop(l) and poseMadeIn(state, l, rung, row) then
				events[#events + 1] = topout(state, l, rung, f) -- once, on the hold that made it due
			end
		end
	elseif POSE[l] and row.kind == K.flow and state.ladders[l] then
		local lad = state.ladders[l]
		local rung = lad.rung or ns.Rung(state, l)
		local k = ns.Library[row.move] and ns.Library[row.move].rung or 0
		if k > rung then
			-- a check (2026-10-16, slice 42: "Do the check", ns.StartFor's check, or a waiting check's pose in a pump's
			-- stretch slot): held its whole time and answered at least as close as the climb from its own step counts
			-- (ns.MarkNeeds), it lifts the family to that step, a placement (cause check) however low it stood, but
			-- never past a step whose own needs wait (liftTo, 2026-10-17)
			local to = liftTo(state, l, rung, k)
			if to > rung and row.outcome == O.done and ns.HasFlag(row.flags, "mark")
				and (row.reps or 0) >= ns.MarkNeeds(l, k) then
				rows[#rows + 1] = rungRow(row.ts, l, to, ns.RungCause.check)
				events[#events + 1] = { kind = "check", ladder = l, rung = to, move = row.move }
			end
		elseif #ns.Path(l, rung) > 0 and ns.PoseDaysIn(state, l, rung) and ns.CanRungUp(l, rung, f, state)
			and not ns.PainOnLadder(state, l, rung, f) and ns.PathIn(state, l, rung, f) then
			-- another pose of the family: its mark can bring the path the climb still waited for, with the step's days
			-- in already; the unlock then comes on this hold
			climb(state, rows, events, row.ts, l, rung, ns.RungCause.up, f)
		end
	elseif l == ns.CARDIO and row.kind == K.cardio and state.cardio then
		cardioRule(state, row, f, rows, events)
	end
	return done()
end

-- The step down an easier pick makes (David, 2026-10-13: "let 'Easier version' on your own step's main move step the
-- ladder down"; he could not do the assisted pistol, his squat rung 7's, and every easier pick kept the ladder there):
-- the result of a slot Swap's easier version took for the attempt at the rung a placed ladder stands on (Engine.lua
-- easierSlot's `down` and `downFrom`) moves that ladder one rung down, while it still stands on that rung, is placed,
-- and none of the set's own rung rows (`rows`, ns.Progression's for `row`, the set row as written) moved it. A rung row
-- with the set's time and the `easier` cause, and the rung-down event; nothing for any other slot. Written after
-- ns.Progression's rows, and never by /qp undo, whose replay asks ns.Progression alone (so the step stays).
function ns.EasierDown(state, slot, row, rows)
	local l = slot.easier and slot.down
	local lad = l and state.ladders[l]
	if not (lad and lad.placedTs and ns.Rung(state, l) == slot.downFrom) then return {}, {} end
	for _, r in ipairs(rows) do
		if r.ladder == l then return {}, {} end
	end
	local rung = slot.downFrom - 1
	return { rungRow(row.ts, l, rung, ns.RungCause.easier) }, { { kind = "rungDown", ladder = l, rung = rung } }
end

-- Correct a count (slice 44, 2026-10-18) -----------------------------------------------------------------------------
-- One correction, from any device: it rewrites one saved set as if typed right, and every fold reads it at the set's
-- own time (DB.lua correct, by the round: ns.CorrectionBeats). The game, the phones and the desktop app write it; the
-- rung rules are asked again only by a device of the set's own source, and only while its steps are the latest.

-- What a correction of set row `r` (as stored) may change, nil where it may change nothing: "count" (a rep set's or a
-- counted set's count), "seconds" (a hold's) or "answer" (a pose's answer to "How close to the goal pose?", on a pose
-- that ran its time and was answered, DB.lua ns.PoseAnswered, the fold's rule too; never a catch-up's, which is
-- practice). Never an owed, banked, paid or skipped row, a timed cardio row, a move this addon lacks, or a set at or
-- before the latest clear row that took its move's record (its own or its ladder's: the fold's `cleared`), whose
-- record went with it.
function ns.CorrectableRow(state, r)
	local id = ns.Merged[r.move] or r.move
	if not ns.Library[id] or not (ns.TYPED[r.outcome] or r.outcome == O.done) then return nil end
	local cleared = state.cleared and state.cleared[id]
	if cleared and r.ts <= cleared then return nil end
	if r.kind == K.rep then return "count" end
	if r.kind == K.hold then return "seconds" end
	if r.kind == K.cardio then return ns.HasFlag(r.flags, "counted") and "count" or nil end
	if ns.PoseAnswered(r) and ns.Source(r.flags) ~= E.source.catchup then return "answer" end
	return nil
end

-- The window's sets (Correct a count): the sets of move `move` (as the fold reads it, a merged move's the move that
-- took it) in `db`'s log that a correction may change (ns.CorrectableRow), of the last correct_days game days to
-- `now`'s and dated after the window's reach (DB.lua ns.CorrectReach over that log and `more`, the times of the rows
-- the game's ring holds beside an app's log, its owed, banked and paid move rows: a margin of rows past the game's
-- ring horizon, `horizon`, the game's sheet summary's rh, for an app's log, which holds rows the game's ring let go,
-- or the log's own baseline's, ns.Horizon, whichever is later; the game's own log passes neither), newest first: the
-- newest correct_sets of them, then the set holding the move's record (its best, and a loaded move's weighted record)
-- where it is older. Each { row (as stored), kind (ns.CorrectableRow's), value (as the fold reads it now, ns.SetValue:
-- a count, a hold's seconds, a pose's answer), outcome (as the fold reads it now, ns.SetOutcome: done where a count of
-- 0 is "Done without a count", a corrected one too; slice 44's review), load (its weight in tenths of a kg, nil none),
-- round (the round a correction of it takes now, ns.CorrectionRound), record (true for a set holding the record),
-- ownSide (DB.lua ns.OwnSide: where another side's correction won, how the set's own side reads it, { value, load };
-- a window of the set's own source offers Save at `value` with its line, 2026-10-19) }.
function ns.Correctable(state, db, move, now, horizon, more)
	local id = ns.Merged[move] or move
	local first = ns.day(ns.DayMid(now, 1 - C.correct_days))
	local log, found = db.log or {}, {}
	local after = ns.CorrectReach(log, math.max(horizon or 0, ns.Horizon(db)), more)
	for i, str in ipairs(log) do
		local r = ns.DecodeRow(str)
		if r and (ns.Merged[r.move] or r.move) == id and r.ts > after and ns.day(r.ts) >= first then
			local kind = ns.CorrectableRow(state, r)
			if kind then found[#found + 1] = { row = r, kind = kind, at = i } end
		end
	end
	table.sort(found, function(a, b)
		if a.row.ts ~= b.row.ts then return a.row.ts > b.row.ts end
		return a.at > b.at
	end)
	local rec, out = state.moves[id], {}
	for n, e in ipairs(found) do
		local r = e.row
		local record = rec ~= nil and (r.ts == rec.bestTs or r.ts == rec.topTs)
		if n <= C.correct_sets or record then
			out[#out + 1] = { row = r, kind = e.kind, value = ns.SetValue(state, r), outcome = ns.SetOutcome(state, r),
				load = ns.SetLoad(state, r), round = ns.CorrectionRound(state, r), record = record,
				ownSide = ns.OwnSide(state, db, r) }
		end
	end
	return out
end

-- The count check a correction meets: ns.Validate's answer for `value` (a count, a hold's seconds) at `load` (tenths of
-- a kg; nil the set's own weight) against the state just before set row `set` folds (ns.Replay ahead), the same limits
-- and "Is that right?" as typing it: "max" refused, "heavy", "really" or "low" asked once, nil; never "low" on a
-- counted set, whose count step asks about no low count (Events.lua Lc.Counted; slice 44's review, 2026-10-18). A
-- placement (a first max set, one back from a break, a capped max set: DB.lua ns.CappedSet) is checked against its
-- window's top where the window counts in the move's unit (ns.FirstTop), a long censored set against the record
-- before it as any set; a pose's answer against nothing. nil too where the set has left the log. Then the state
-- before the set and the slot it was judged as, from which a window words its ask as the count step does (the record
-- before the set, its weight as planned, its window's top: the game's Panel.lua Panel.ReallyWords).
function ns.CorrectionCheck(db, set, value, load)
	if set.kind == K.flow then return nil end
	local s = ns.Replay(db, set, true)
	if not s then return nil end
	local id = ns.Merged[set.move] or set.move
	local read = ns.ShiftRow(CopyTable(set)) -- its rung as the fold reads it (a ladder that lost its first rung since)
	local lo, hi, own = ns.Window(read.ladder, read.rung, id)
	local anchor = ns.Anchor(read.ladder, read.rung)
	local placing = set.outcome == O.placement or ns.CappedSet(s, read)
	local slot = { move = id, kind = set.kind, counted = ns.HasFlag(set.flags, "counted") or nil,
		placing = placing or nil, cap = lo and ns.OwnUnit(id, anchor, own) and hi or nil, load = ns.SetLoad(s, set) }
	local check = ns.Validate(s, slot, value, load or slot.load)
	if check == "low" and slot.counted then check = nil end
	return check, s, slot
end

-- A set's step changes, from ns.Progression's rows for it: its rung rows (`rungs`, { ladder, to, row } each) and the
-- steps it masters (`mastered`, { ladder, rung } each), in their order.
local function stepsOf(rows)
	local out = { rungs = {}, mastered = {} }
	for _, r in ipairs(rows) do
		if r.kind == K.rung then
			out.rungs[#out.rungs + 1] = { ladder = r.ladder, to = r.rung, row = r }
		elseif r.kind == K.mastered then
			out.mastered[#out.mastered + 1] = { ladder = r.ladder, rung = r.rung }
		end
	end
	return out
end

-- The change of `list` on ladder `l` (on its step `rung`, where given), or nil.
local function changeOn(list, l, rung)
	for _, x in ipairs(list) do
		if x.ladder == l and (rung == nil or x.rung == rung) then return x end
	end
	return nil
end

-- The step rows after set row `set` in `log` (from its index `at`) that a fold reads (DB.lua ns.UndoPairs' `gone`
-- aside), of every character, in the log's order: each ladder's rung rows but start rows, which change nothing on a
-- ladder with a rung (`rungs[ladder]`), and each step's mastered rows but undo rows, which master nothing
-- (`steps["ladder:rung"]`, ns.RungKey), each { str, row }; then the times of the sets of the set's character after it
-- (`sets`: a step row dated at one of them was written with that set), and the step rows of its character dated at its
-- own time (`typed`: written with it, never an undo or an easier pick's step down, which the player chose), in their
-- order.
local function stepRowsAfter(log, at, set, gone)
	local rungs, steps, sets, typed = {}, {}, {}, {}
	for i = at + 1, #log do
		local q = ns.DecodeRow(log[i])
		q = q and ns.ShiftRow(q) -- its rung as the fold reads it (a ladder that lost its first rung since)
		if q and not gone[log[i]] then
			local e = { str = log[i], row = q }
			if q.kind == K.rung and q.reps ~= ns.RungCause.start then
				local list = rungs[q.ladder] or {}
				list[#list + 1] = e
				rungs[q.ladder] = list
			elseif q.kind == K.mastered and q.reps ~= ns.RungCause.undo then
				local key = ns.RungKey(q.ladder, q.rung)
				local list = steps[key] or {}
				list[#list + 1] = e
				steps[key] = list
			elseif q.char == set.char and q.ts ~= set.ts and (ns.TYPED[q.outcome] or q.outcome == O.done)
				and (q.kind == K.rep or q.kind == K.hold or q.kind == K.cardio or q.kind == K.flow) then
				sets[q.ts] = true
			end
			if (q.kind == K.rung or q.kind == K.mastered) and q.char == set.char and q.ts == set.ts
				and q.reps ~= ns.RungCause.undo and not (q.kind == K.rung and q.reps == ns.RungCause.easier) then
				typed[#typed + 1] = e
			end
		end
	end
	return rungs, steps, sets, typed
end

-- The set's step changes as they stand (ns.CorrectionRows' `before`; slice 44's review, 2026-10-18), each with the
-- logged row it stands as (`str`): the step rows its character logged with it (stepRowsAfter's `typed`), and of the
-- changes `replay` gives (ns.Progression on the replay of its own side, `stepsOf`), on a ladder or step none was logged
-- with it, the ones the log holds as a row of its character that no later set wrote (a correction's of its own side,
-- written when it was made). So a climb or a step mastered the set brought goes back whatever a replay with today's
-- setup, or with the other side's correction, would now read, and a change the log never held is none to take back.
local function standingSteps(log, at, set, gone, replay)
	local rungs, steps, sets, typed = stepRowsAfter(log, at, set, gone)
	local out = { rungs = {}, mastered = {} }
	for _, e in ipairs(typed) do
		local q = e.row
		if q.kind == K.rung and not changeOn(out.rungs, q.ladder) then
			out.rungs[#out.rungs + 1] = { ladder = q.ladder, to = q.rung, row = q, str = e.str }
		elseif q.kind == K.mastered and not changeOn(out.mastered, q.ladder, q.rung) then
			out.mastered[#out.mastered + 1] = { ladder = q.ladder, rung = q.rung, str = e.str }
		end
	end
	-- a row of the set's character, after it, that no later set wrote
	local function logged(list, rung)
		for _, e in ipairs(list or {}) do
			local q = e.row
			if q.char == set.char and q.ts ~= set.ts and not sets[q.ts] and q.reps ~= ns.RungCause.undo
				and (rung == nil or q.rung == rung) then
				return e
			end
		end
		return nil
	end
	for _, x in ipairs(replay.rungs) do
		local e = not changeOn(out.rungs, x.ladder) and logged(rungs[x.ladder], x.to)
		if e then out.rungs[#out.rungs + 1] = { ladder = x.ladder, to = x.to, row = x.row, str = e.str } end
	end
	for _, x in ipairs(replay.mastered) do
		local e = not changeOn(out.mastered, x.ladder, x.rung) and logged(steps[ns.RungKey(x.ladder, x.rung)])
		if e then out.mastered[#out.mastered + 1] = { ladder = x.ladder, rung = x.rung, str = e.str } end
	end
	return out
end

-- Whether set row `set`'s step changes are still the latest (the guard under which a correction asks the rung rules
-- again, /qp undo's too): `before`, the changes the set makes as it stands (each with the logged row it stands as,
-- `str`: ns.CorrectionRows' standingSteps), and `after`, those the corrected set makes (each { rungs, mastered }). On
-- every ladder and step either touches, the rows after the set in `db`'s log that a fold reads (a rung change an undo
-- reversed, and that undo, never happened: ns.UndoPairs), of every character (slice 44's review, 2026-10-18: a later
-- climb stands on the set's step whoever made it), must be the set's own: on a ladder `before` moves, its row alone,
-- the one an undo of its character written now pairs with; on a ladder only `after` moves, none; on a step `before`
-- masters, its row alone, the one a mastered undo pairs with; on a step only `after` masters, none. Else a later
-- change (another set's step, a sweep's step, the player's step down, another device's) stands on the set's, and the
-- step stays: an undo would take back the wrong one (before slice 44, /qp undo's paired with a later climb).
function ns.RungsLatest(db, set, before, after)
	local log = db.log or {}
	local at = ns.SetIndex(log, set)
	if not at then return false end
	local gone, last, latest = ns.UndoPairs(log, db.baseline)
	local rungs, steps = stepRowsAfter(log, at, set, gone)
	for _, x in ipairs(before.rungs) do
		local list = rungs[x.ladder]
		if not (list and #list == 1 and list[1].str == x.str and last[x.ladder .. ":" .. set.char] == x.str) then
			return false
		end
	end
	for _, x in ipairs(after.rungs) do
		if rungs[x.ladder] and not changeOn(before.rungs, x.ladder) then return false end
	end
	for _, x in ipairs(before.mastered) do
		local list = steps[ns.RungKey(x.ladder, x.rung)]
		if not (list and #list == 1 and list[1].str == x.str
			and latest[x.ladder .. ":" .. x.rung .. ":" .. set.char] == x.str) then
			return false
		end
	end
	for _, x in ipairs(after.mastered) do
		if steps[ns.RungKey(x.ladder, x.rung)] and not changeOn(before.mastered, x.ladder, x.rung) then return false end
	end
	return true
end

-- Whether `left`, the state a correction's rows leave (the writer's fold, with `f` the writer's setup the mastered
-- sweep after them reads), lacks a step change the corrected set calls for, which the step line then names (slice 44's
-- review, 2026-10-18: worked out as the sweep leaves the steps, so no line says a step stays that the sweep masters at
-- once): where the rung changes differ (`same` false), a ladder whose rung is not the one the corrected set leaves it
-- on (its change's, else the rung before the set, `s0`'s); a step it no longer masters that the sweep would not master
-- again (`lost`); a step it now masters that the sweep neither holds nor masters (`won`).
local function lacks(left, f, s0, before, after, same, lost, won)
	if not same then
		local ladders = {}
		for _, x in ipairs(before.rungs) do ladders[#ladders + 1] = x.ladder end
		for _, x in ipairs(after.rungs) do ladders[#ladders + 1] = x.ladder end
		for _, l in ipairs(ladders) do
			local x = changeOn(after.rungs, l)
			if ns.Rung(left, l) ~= (x and x.to or ns.Rung(s0, l)) then return true end
		end
	end
	for _, x in ipairs(lost) do
		if not sweeps(left, x.ladder, x.rung, f) then return true end
	end
	for _, x in ipairs(won) do
		if not (ns.Mastered(left, x.ladder, x.rung) or sweeps(left, x.ladder, x.rung, f)) then return true end
	end
	return false
end

-- What a correction's rows `rows` (ns.CorrectionRows', its step rows among them) change on the player's steps, as the
-- done view and /qp undo's line say it (2026-10-19, the program's double-check: a climb, a stretch check's lift or a
-- Mastered step went back with no word): the fold of `trial`'s log (the log with the correction's load and correction
-- rows) with the step rows after it, against `state`, the fold before them, on each ladder and step the set's changes
-- touch (`before` and `after`, `lost` and `won`): `rungs`, each ladder whose rung moved, { ladder, from, to } (a
-- stretch check's lift taken back among them); `mastered`, each step whose seal changed, { ladder, rung, won }, a lost
-- one only where the mastered sweep after the rows (with the writer's `profile`) would not seal it again; in the
-- order the rows name them. nil where none changed.
local function stepChanges(state, trial, rows, before, after, lost, won, profile)
	local log = {}
	for i, str in ipairs(trial.log) do log[i] = str end
	for _, r in ipairs(rows) do
		if r.kind == K.rung or r.kind == K.mastered then log[#log + 1] = ns.EncodeRow(r) end
	end
	local left = ns.FoldOf({ log = log, baseline = trial.baseline, sheetPart = trial.sheetPart })
	local f = ns.FilterContext(profile, left)
	local out, seen = { rungs = {}, mastered = {} }, {}
	for _, list in ipairs({ before.rungs, after.rungs }) do
		for _, x in ipairs(list) do
			local l = x.ladder
			local from, to = ns.Rung(state, l), ns.Rung(left, l)
			if not seen[l] and from ~= to then out.rungs[#out.rungs + 1] = { ladder = l, from = from, to = to } end
			seen[l] = true
		end
	end
	for _, x in ipairs(lost) do
		if ns.Mastered(state, x.ladder, x.rung) and not (ns.Mastered(left, x.ladder, x.rung)
			or sweeps(left, x.ladder, x.rung, f)) then
			out.mastered[#out.mastered + 1] = { ladder = x.ladder, rung = x.rung, won = false }
		end
	end
	for _, x in ipairs(won) do
		if not ns.Mastered(state, x.ladder, x.rung) and ns.Mastered(left, x.ladder, x.rung) then
			out.mastered[#out.mastered + 1] = { ladder = x.ladder, rung = x.rung, won = true }
		end
	end
	if #out.rungs == 0 and #out.mastered == 0 then return nil end
	return out
end

-- The rows that correct set row `set` (as stored in `db`'s log: ns.Correctable's) in `state`, the fold of that log, in
-- the order they are written, and why its steps stay where they do. `value` is the corrected count (a rep set's or a
-- counted set's count, a hold's seconds, per side on a sided hold, as ns.SetValue reads it; nil for a pose), `answer` a
-- pose's (ns.Enum.mark; nil for any other set), `load` a loaded rep set's weight (tenths of a kg, 0 none; nil keeps the
-- set's own), `now` the time the step rows are written, `profile` the writer's (the mastered sweep's filters, and the
-- rung rules' where `db` holds no profile of the set's character) and `own` true where the writer is of the set's own
-- source, which only the writer's store can tell (the game for its own sets and its catch-ups', every row without the
-- app bit; an app for the app's own rows, which a phone keeps without the app bit until it sends them). The rows:
-- - a load row where the weight changes, or where it is the weight the set reads with now and the set's own side reads
--   it otherwise (another side's correction of its weight won: ns.OwnSide, whose Save writes it again here; 2026-10-19,
--   the program's double-check, as the correction row of the same count), its round in outcome;
-- - the correction row, its round (ns.CorrectionRound) in outcome;
-- - where the corrected set calls for other step changes than the set as it stands, the writer is of the set's own
--   source (`own`) and its steps are still the latest (ns.RungsLatest): an undo row (cause undo, at the rung the ladder
--   stood on before the set) for each rung change the set makes now and a rung row for each the corrected set makes,
--   all when they differ, then a mastered undo row for each step it no longer masters and a mastered row for each it
--   now masters, each at `now` with the set's character (the fold pairs an undo with its own character's rows: DB.lua
--   undone), each undo naming the row it reverses (the logged step row the set's change stands as: its ts in questID,
--   2026-10-19).
-- The steps (slice 44's review, 2026-10-18): the rung rules are asked of a replay whose corrections, and correcting
-- load rows, are the set's own side's alone (the game's, without the app bit, for a set without it; an app's, with it,
-- for an app set: the step rows in the log followed only those, since only the set's own side asks the rung rules),
-- the corrected one with this correction's rows, each with the profile of the set's own character (DB.lua
-- ns.CharProfile; the writer's where `db` holds none: an app's sets); the set's changes as they stand are the step rows
-- the log holds for it (standingSteps), never an easier pick's step down, which the player chose.
-- The load and correction rows carry no character and no source bits: the writer's are its own (the game's AddRow, an
-- app's write). `stays`: nil where the steps follow, "source" where another source's set keeps its steps, "later"
-- where a later change stands on them (the window says so), each only where the state the rows leave, with the
-- mastered sweep after them, lacks a change the corrected set calls for (`lacks`). Third, where step rows are written,
-- what they change on the player's steps (stepChanges: { rungs = { { ladder, from, to } }, mastered = { { ladder,
-- rung, won } } }), which the done view and /qp undo's line say; nil where they change none. nil (no rows) where the
-- set may not be corrected (ns.CorrectableRow), its value or answer is none, or it has left the log.
function ns.CorrectionRows(db, state, set, value, load, answer, now, profile, own)
	local kind = ns.CorrectableRow(state, set)
	if kind == "answer" then
		if not (answer and answer >= E.mark.notyet and answer <= E.mark.perfect) then return nil end
	elseif not (kind and value and value >= 0) then
		return nil
	end
	local log = db.log or {}
	local at = ns.SetIndex(log, set)
	if not at then return nil end
	-- the set's own side: the corrections and the load rows its step rows followed
	local app = ns.Source(set.flags) == E.source.app
	local function side(r) return (ns.Source(r.flags) == E.source.app) == app end
	local s0, read0 = ns.Replay(db, set, nil, side)
	if not s0 then return nil end
	local id = ns.Merged[set.move] or set.move
	local round = ns.CorrectionRound(state, set)
	local rows = {}
	if load and set.kind == K.rep and ns.Loaded(id)
		and (load ~= (ns.SetLoad(state, set) or 0) or load ~= (ns.SetLoad(s0, set) or 0)) then
		local l = ns.LoadRow(set, load)
		l.char, l.outcome, l.flags = nil, round, 0
		rows[1] = l
	end
	local sided = ns.Sided[id]
	local fix = { ts = set.ts, questID = set.questID, tier = set.tier, slot = set.slot, ladder = set.ladder,
		rung = set.rung, move = set.move, kind = K.correction, reps = 0, seconds = 0, outcome = round, flags = 0 }
	if kind == "answer" then
		fix.reps = answer
	elseif kind == "seconds" then
		-- a sided hold from before its sides held both on one clock: the fold reads its seconds, and so its
		-- correction's, as half (DB.lua sidedHalf)
		fix.seconds = (sided and set.ts < sided) and value * 2 or value
	else
		fix.reps = value
	end
	rows[#rows + 1] = fix
	-- the corrected set, replayed on the log with these rows, its own side's corrections and this one
	local trial, written = { log = {}, baseline = db.baseline, sheetPart = db.sheetPart }, {}
	for i, str in ipairs(log) do trial.log[i] = str end
	for _, r in ipairs(rows) do
		local str = ns.EncodeRow(r)
		trial.log[#trial.log + 1], written[str] = str, true
	end
	local s1, read1 = ns.Replay(trial, set, nil, function(r, str) return written[str] or side(r) end)
	local mine = ns.CharProfile(db, set.char) or profile
	local before = standingSteps(log, at, set, (ns.UndoPairs(log, db.baseline)),
		stepsOf((ns.Progression(s0, read0, ns.FilterContext(mine, s0)))))
	local after = stepsOf((ns.Progression(s1, read1, ns.FilterContext(mine, s1))))
	local same = #before.rungs == #after.rungs
	for _, x in ipairs(before.rungs) do
		local y = changeOn(after.rungs, x.ladder)
		same = same and y ~= nil and y.to == x.to
	end
	local lost, won = {}, {}
	for _, x in ipairs(before.mastered) do
		if not changeOn(after.mastered, x.ladder, x.rung) then lost[#lost + 1] = x end
	end
	for _, x in ipairs(after.mastered) do
		if not changeOn(before.mastered, x.ladder, x.rung) then won[#won + 1] = x end
	end
	if same and #lost == 0 and #won == 0 then return rows, nil end
	local stays = not own and "source" or not ns.RungsLatest(db, set, before, after) and "later" or nil
	if stays then
		local left = ns.FoldOf(trial)
		return rows, lacks(left, ns.FilterContext(profile, left), s0, before, after, same, lost, won) and stays or nil
	end
	-- each undo names the row it reverses (its ts in questID, DB.lua pairing), the logged row it stands as
	local function named(x) return (ns.DecodeRow(x.str) or {}).ts or 0 end
	if not same then
		for _, x in ipairs(before.rungs) do
			rows[#rows + 1] = { ts = now, char = set.char, questID = named(x), kind = K.rung, ladder = x.ladder,
				rung = ns.Rung(s0, x.ladder), reps = ns.RungCause.undo }
		end
		for _, x in ipairs(after.rungs) do
			rows[#rows + 1] = { ts = now, char = set.char, kind = K.rung, ladder = x.ladder, rung = x.to,
				reps = x.row.reps }
		end
	end
	for _, x in ipairs(lost) do
		rows[#rows + 1] = { ts = now, char = set.char, questID = named(x), kind = K.mastered, ladder = x.ladder,
			rung = x.rung, reps = ns.RungCause.undo }
	end
	for _, x in ipairs(won) do
		rows[#rows + 1] = { ts = now, char = set.char, kind = K.mastered, ladder = x.ladder, rung = x.rung }
	end
	return rows, nil, stepChanges(state, trial, rows, before, after, lost, won, profile)
end

-- The start rungs the profile's experience picks (ns.StartRung: none step 2, some 3, training 4), each lowered to a
-- rung the setup can serve (the wall sit to its four angles; "avoid breath-hold strain" leaves its two deepest out
-- through ns.RungMove).
-- One rung row per strength, hold and cardio ladder that holds a move and has a rung at or under the start step; a
-- pose ladder starts on rung 1 without one, and so do the pull-up and the side below step 4 (no colour is given there).
function ns.StartRows(experience, f, ts)
	local rows = {}
	local step = C.start_rung[experience or "none"] or C.start_rung.none
	for ladder = 1, ns.LADDER_MAX do -- by id, as the phone cores write them
		local r = ns.Ladders[ladder] and ns.StartRung(ladder, experience)
		if not r or POSE[ladder] or ns.HeldTop(ladder) == 0 or not ns.RungAtStep(ladder, step) then
			r = nil
		elseif ladder ~= ns.CARDIO then
			local _, at = ns.RungMove(ladder, r, f)
			r = at or 1
		else
			while r > 1 do
				local ok = false
				for id, m in pairs(ns.Library) do
					if m.ladder == ns.CARDIO and m.rung == r and not m.endOnly and ns.Passes(id, f) then ok = true end
				end
				if ok then break end
				r = r - 1
			end
		end
		if r then rows[#rows + 1] = rungRow(ts, ladder, r, ns.RungCause.start) end
	end
	return rows
end

-- The talent trees (slice 36) ---------------------------------------------------------------------------------------
-- The rules every surface's trees read (the game's Talents tab, the desktop app's and the phones' Book; the phones'
-- cores port them against contract/fixtures/engine.json's `move` and `tree` events): what a step's Start starts, the
-- ring around the player's step, a ladder's entry in its tree, the step that glows and the rows a tree draws in full,
-- each over the numbers a Book entry holds, in whole numbers and in each tree's own order, so the four surfaces cannot
-- drift.

-- The countable form a Start of a counted cardio rung runs and its ring names (2026-10-16; the form nearest the top
-- of its count's window until then, ns.CountedNear): the form the rung's path takes its turn on (ns.PathTurn: a stage
-- still short of its clean days), else of the forms the setup `f` allows (ns.CountForms) the one with the fewest clean
-- days at the rung (ns.TopDays), a tie the lowest id; without a `state` (the setup's own tests) the lowest id. Returns
-- it and its record (its best clean count, nil for none); nil on a rung the setup leaves no countable form (a timed
-- rung).
function ns.CountedForm(state, rung, f)
	local forms = ns.CountForms(rung, f)
	if #forms == 0 then return nil end
	if not state then return forms[1] end
	local best = ns.PathTurn(state, ns.CARDIO, f, rung)
	if not best then
		local fewest
		for _, id in ipairs(forms) do
			local n = ns.TopDays(state, ns.CARDIO, rung, id)
			if not fewest or n < fewest then best, fewest = id, n end
		end
	end
	return best, (ns.Record(state, best))
end

-- The ladder the Trial places next (Engine.lua trialSlot's order, ns.TrialOrder), of talent tree `tree` alone where
-- one is given: the first not placed yet that a move the setup `f` allows stands for and the player has not stepped
-- down from; nil once each the setup serves is placed.
function ns.TrialNext(state, f, tree)
	local function down(id) return stepped(state, id, nil) end
	for _, l in ipairs(ns.TrialOrder) do
		local lad = state.ladders and state.ladders[l]
		if (not tree or ns.TreeOf(l) == tree) and not (lad and lad.placedTs)
			and ns.RungMove(l, ns.Rung(state, l), f, down) then
			return l
		end
	end
	return nil
end

-- The move a refused Start names: the move, its ladder and its rung.
local function subjectOf(moveId)
	local m = ns.Library[moveId]
	return { move = moveId, ladder = m.ladder, rung = m.rung }
end

-- Why a Start starts nothing, for a move the filters with the pain flags shut, and its subject, that move ({ move,
-- ladder, rung, off }): "pain" where the setup alone (`setup`, no pain flag) passes it, so a flag shuts it (it goes
-- when the pain has gone); else "setup" where the setup shuts it (the equipment, Floor moves, Room to move, quiet mode,
-- the Body switches), with `off` where the player's own list leaves it out too, so a card names both at once; "off"
-- where only that list does (the exclusion list, an opt-in move not checked).
local function refusal(moveId, setup)
	local subject = subjectOf(moveId)
	if ns.Passes(moveId, setup) then return "pain", subject end
	local why = "off"
	for _, w in ipairs(ns.Shut(moveId, setup)) do
		if w.reason == "excluded" then subject.off = true else why = "setup" end
	end
	return why, subject
end

-- A Start's answer with `key` set to `value` (`via` on a start that runs another ladder's climb, `back` on a step the
-- player has above their step), a refusal passed on as it is.
local function tagged(key, value, s, ...)
	if s then s[key] = value end
	return s, ...
end

-- The move a stepped-down move gives way to (ns.SteppedDown): the first down its named regression that an easier set
-- marked (the fold's `easier`), which the pumps offer at its own rung until a set clears the mark (Engine.lua
-- candidates), and only while the filters pass it.
local function easierOf(state, moveId)
	local x, seen = ns.Library[moveId].regress, {}
	while x and not seen[x] do
		local rec = state.moves[x]
		if rec and rec.easier ~= nil then return x end
		seen[x] = true
		x = ns.Library[x].regress
	end
	return moveId
end

-- A move of a step's own level that the filters `f` pass: on a counted cardio rung its form as a counted set
-- (ns.CountedForm), else the rung's own steady move (cardioRungMove), else the all-out form it runs as a window; on
-- another ladder a move of its rung's chain at that rung that is open (ns.MoveOpen: a check of its own done), and with
-- the player's `state` the next of the chain where they stepped down from one, the easier move they stepped down to
-- where they stepped down from every one. Returns the move and whether it runs all out; nil where none passes, nil and
-- the easier move where the filters shut that one, and nil, nil and the move where every one that passes waits for
-- what opens it (a check: the explosive push-up's 12 feet-up push-ups, 2026-10-16).
local function levelMove(state, l, r, f)
	if l == ns.CARDIO then
		local form = ns.CountedForm(state, r, f)
		if form then return form, true end
		local id = cardioRungMove(r, f, true)
		if id then return id, false end
		id = cardioRungMove(r, f)
		if id then return id, true end
		return nil
	end
	local first, waiting
	for _, id in ipairs(ns.RungChain(l, r)) do
		if ns.Library[id].rung == r and ns.Passes(id, f) then
			if state and not ns.MoveOpen(state, id, f) then
				waiting = waiting or id
			elseif not (state and stepped(state, id, nil)) then
				return id, false
			else
				first = first or id
			end
		end
	end
	if not first then return nil, nil, waiting end
	local e = easierOf(state, first)
	if not ns.Passes(e, f) then return nil, e end
	return e, false
end

-- Whether the days the climb from a ladder's `rung` counts are in (its path and the steps of other ladders aside):
-- the anchor's game days at its window's top (ns.UnlockDays), a pose family's mark days, a counted cardio rung's clean
-- days (ns.CleanDays), a timed one's full moves (cardio_full_days; each return_days on a climb back, cardioProgress).
local function daysIn(state, l, rung, f)
	if l == ns.CARDIO then
		local have, need = cardioProgress(state, rung, f)
		return have >= need
	end
	if POSE[l] then return ns.PoseDaysIn(state, l, rung) end
	local lad = state.ladders and state.ladders[l]
	return (lad and lad.hiDays or 0) >= ns.UnlockDays(l, rung, state)
end

-- Whether a pose family's mark came this game day (a goal pose of its step held, whole or not: the fold's anchorTs,
-- DB.lua markAttempt; its marking pose alone until slice 47), so the pumps ask it at most once a game day (Engine.lua
-- candidates and markSlot; docs/safety.md). A hold still in the night after a day counted on an earlier game day (the
-- fold's hiTs, no new session by the night rule since it: ns.NewSession) is not the mark: no answer could count then,
-- so the pumps asked none (ns.DuePose, Engine.lua marks; 2026-10-20, slice 47's fix round: a Good at 01:00 counted
-- Sunday's day, and a pose held at 08:30 the next morning, under the night rule's 8 hours, took Monday's one ask for
-- nothing). Every hold of a game day after such a one is out of that night too (ns.NewSession only grows with the
-- time), so the last hold says it.
function ns.MarkedToday(state, ladder, now)
	local lad = state.ladders and state.ladders[ladder]
	local at = lad and lad.anchorTs
	if not (at and ns.day(at) == ns.day(now)) then return false end
	local hi = lad.hiTs
	return not (hi and ns.day(hi) ~= ns.day(at) and not ns.NewSession(hi, at))
end

-- A step's goal poses in the step's order (slice 47), as its card lists them (Tree.lua: Your main move, Then, for the
-- next step, Same level): the main pose `main` (the move of the rung's chain standing for it) first, then the poses
-- of its path stage by stage, then the rest by id; every pose of the rung's own level with a goal pose (ns.Marks),
-- never another of the main pose's chain (one of a chain stands for the step, its twins only where the setup shuts it).
local function stepPoses(ladder, rung, main)
	local out, seen = {}, {}
	local function add(id)
		if not seen[id] and ns.Marks(ladder, rung, id) and (id == main or not ns.InChain(ladder, rung, id)) then
			out[#out + 1] = id
		end
		seen[id] = true
	end
	if main then add(main) end
	for _, stage in ipairs(ns.Path(ladder, rung)) do
		for _, id in ipairs(stage.ids) do add(id) end
	end
	local rest = {}
	for _, id in ipairs(members[ladder] or EMPTY) do
		if not seen[id] then rest[#rest + 1] = id end
	end
	table.sort(rest)
	for _, id in ipairs(rest) do add(id) end
	return out
end

-- A move the pumps leave out of its step (ns.PoseMain, ns.DuePose, ownMove): one the player stepped down from or
-- that is not open (ns.MoveOpen), as Engine.lua's context holds it back.
local function leftOut(state, id, f)
	return ns.SteppedDown(state, id) or not ns.MoveOpen(state, id, f)
end

-- The main pose standing for a pose family's step with filter context `f` (slice 47): the move of the rung's chain the
-- pumps bring for it, as Engine.lua's rung move (ns.RungMove: one the player stepped down from or that is not open
-- gives way to the next of its chain), when it is the step's own (ns.MainPose); nil where a stand-in from below stands
-- for the step or none does. The pose the climb's floor names (ns.PoseNeeds' mainLeft: Tree.lua, Panel.lua and the
-- desktop app's and the phones' cards name it) and the rotation forces (ns.DuePose).
function ns.PoseMain(state, ladder, f)
	if not POSE[ladder] then return nil end
	local rung = ns.Rung(state, ladder)
	local id = ns.RungMove(ladder, rung, f, function(x) return leftOut(state, x, f) end)
	return id and ns.MainPose(ladder, rung, id) and id or nil
end

-- The pose due for a pose family's first turn of the game day (slice 47, 2026-10-19, David: "I'm stuck at two starter
-- excercises"; the marking pose every game day from slice 21 until then), with filter context `f` at `now`: nil once
-- the family's mark came today (ns.MarkedToday), where it is not open, and while its day cannot count now (ns.DayIn:
-- the night rule's wait after the day counted last while the family climbs, or the day's ask already came; 2026-10-20,
-- slice 47's fix round, as the pace ask's full-move reason, ns.LedgerAsk); else the step's main pose (the move of its
-- chain standing for it, as Engine.lua's rung move: one the player stepped down from or that is not open gives way to
-- the next) while its own days would otherwise fall short (ns.PoseNeeds: its shortfall at least the counted days the
-- step still needs) and on the top step while its orange or gold colour waits for the Perfect hold that earns it
-- (ns.ToEarn), else the step's goal pose (stepPoses: the filters pass it, it is open and the player did not step down
-- from it) whose last answered ask is oldest (the fold's marks[move].askTs, DB.lua markDays), on the top step, where no
-- pose is asked once its own goal pose is reached (Engine.lua markSlot), whose last hold is oldest (marks[move].heldTs;
-- 2026-10-20), one never asked or held first, ties in the step's order. Engine.lua's candidates give it the marking
-- treatment the anchor had (due, the overload engine's dueAnchor); every other goal pose of the step joins as one more
-- pose of the family.
function ns.DuePose(state, ladder, f, now)
	if not POSE[ladder] or not ns.Open(state, ladder) or ns.MarkedToday(state, ladder, now)
		or ns.DayIn(state, ladder, now, f) ~= nil then
		return nil
	end
	local rung = ns.Rung(state, ladder)
	local main = ns.PoseMain(state, ladder, f)
	local top = rung >= ns.HeldTop(ladder)
	if main then
		if top then
			if ns.ToEarn(state, ladder, f) == rung then return main end
		else
			local p = ns.PoseNeeds(state, ladder, rung)
			if p.mainLeft > 0 and p.mainLeft >= p.left then return main end
		end
	end
	local best, bestTs
	for _, id in ipairs(stepPoses(ladder, rung, main)) do
		if id == main or ns.Passes(id, f) and not leftOut(state, id, f) then
			local d = state.marks and state.marks[id]
			local ts = d and d[top and "heldTs" or "askTs"] or 0
			if not best or ts < bestTs then best, bestTs = id, ts end
		end
	end
	return best
end

-- When a set can next count toward the climb from a ladder's rung, or toward earning it (slice 36: the card's day
-- line, and no glow on the ladder until then), with filter context `f`, and why it waits: nil while one at `now` can,
-- and where nothing is left to count (no climb open, ns.UnlockProgress's need 0: the top, a family not open, a rung
-- the setup shuts or one that waits for another ladder's step; or the days in, ns.UnlockDays, so the climb waits for
-- its path or another ladder); else the first moment one can and its kind: "session" after a set that counted its
-- day this session (the fold's hiTs on a strength, hold or pose ladder, DB.lua anchorAttempt and markAttempt;
-- cleanTs on a counted cardio rung, fullTs on a timed one, slice 52), a new session by the night
-- rule; "day" where no set counts again before the next game day: a pose family's pose asked once a game day
-- (Engine.lua markSlot; a timed cardio rung's pace ask answered today too, slice 42, until slice 52): a goal pose of
-- its step held today (ns.MarkedToday; its marking pose alone until slice 47) while it climbs or its orange or gold
-- step waits for the Perfect hold that earns it (ns.ToEarn), and the pose its path takes its turn on once the days are
-- in (ns.PathTurn), held today. A counted cardio step whose days are in waits for its proving window's session (slice
-- 53, ns.Proving: "session", from the last counted window at the step).
function ns.DayIn(state, ladder, now, f)
	local rung = ns.Rung(state, ladder)
	local _, need = ns.UnlockProgress(state, ladder, f)
	local climbing = need > 0 and not daysIn(state, ladder, rung, f)
	if POSE[ladder] then
		if need > 0 and not climbing then
			local turn = ns.PathTurn(state, ladder, f, rung)
			if turn and ns.SetsToday(state, turn, now) > 0 then return ns.NextDayStart(now), "day" end
			return nil
		end
		if not (climbing or ns.ToEarn(state, ladder, f) == rung) then return nil end
		local lad = state.ladders and state.ladders[ladder]
		local at, kind
		if climbing and lad and lad.hiTs and not ns.NewSession(lad.hiTs, now) then
			at, kind = sessionFrom(lad.hiTs), "session"
		end
		if ns.MarkedToday(state, ladder, now) and (not at or ns.NextDayStart(now) > at) then
			at, kind = ns.NextDayStart(now), "day"
		end
		return at, kind
	end
	if ladder == ns.CARDIO and need > 0 and not climbing then
		-- the proving window waits for the next session after the last counted window at the step (slice 53)
		local at = ns.Proving(state, f, now)
		if at == "next" or at == "waits" then return sessionFrom(state.cardio.countTs), "session" end
		return nil
	end
	if not climbing then return nil end
	if ladder == ns.CARDIO then
		local c = state.cardio or EMPTY
		if ns.CountedRung(rung, f) then
			if c.cleanTs and not ns.NewSession(c.cleanTs, now) then return sessionFrom(c.cleanTs), "session" end
			return nil
		end
		if c.fullTs and not ns.NewSession(c.fullTs, now) then return sessionFrom(c.fullTs), "session" end
		return nil
	end
	local lad = state.ladders and state.ladders[ladder]
	if lad and lad.hiTs and not ns.NewSession(lad.hiTs, now) then return sessionFrom(lad.hiTs), "session" end
	return nil
end

-- A Start's answer: the move, `allout` on a counted set or another all-out window (the window alone, from its go:
-- Engine.lua's chosen compose), its kind (`as`), the ladder and the rung its slot is dosed at (the compose takes it as
-- `chosen.rung`: the player's rung for the move standing for it and a form of the cardio rung, a move's own rung
-- below it, so a step the player has never climbs and never steps them down), and what the card says beside it: the
-- goal as the pump will ask it (ns.PickGoal; none on a max set, a counted set, a pose or a timed move) and its kind,
-- when a set is fresh again (ns.FreshFrom) and whether the day's sets reached start_tired_sets, on a move that keeps a
-- record (a counted set's best clean count among them); and, on the move standing for the player's rung (`stands`),
-- when the climb's day can count again and why (ns.DayIn).
local function started(state, f, now, moveId, as, l, rung, allout, placing, stands)
	local m = ns.Library[moveId]
	local s = { move = moveId, as = as, ladder = l, rung = rung, allout = allout or nil }
	local typed = m.dose == "rep" or m.dose == "hold"
	if typed then s.goal, s.goalKind = ns.PickGoal(state, moveId, K[m.dose], now, f, placing) end
	if typed or allout and m.count then
		s.freshFrom = ns.FreshFrom(state, moveId, now)
		s.tired = ns.SetsToday(state, moveId, now) >= C.start_tired_sets or nil
	end
	if stands and l then s.dayIn, s.dayKind = ns.DayIn(state, l, now, f) end
	return s
end

local startAt -- below: a start can run another ladder's (a family not open yet, a step another climb waits for)

-- A start of `id`, or the cardio climb it waits for: a jump before its cardio rung (`crung`: power's jumps before
-- cardio rung 5) starts the climb toward that rung instead (`via` reach), as no pump offers it there (Engine.lua
-- jumpReady).
local function startOrJump(state, f, setup, now, depth, id, ...)
	local crung = ns.Library[id].crung
	if crung and ns.Rung(state, ns.CARDIO) < crung then
		return tagged("via", "reach", startAt(state, f, setup, { ladder = ns.CARDIO, rung = crung }, now, depth + 1))
	end
	return started(state, f, now, id, ...)
end

local needStart -- below: a check whose family's climb to its step waits for another step runs that one first

-- A check (2026-10-16, slice 42: "Do the check" on a waiting step's card or move's window): the pose a pose need names
-- (its `move`), else the first of its step's own chain at its level the setup offers (ns.NeedAt), held its time at its
-- own step however low its family stands and asked "How close to the goal pose?" at the check's least answer
-- (`ask`, ns.CheckAsk), which at that answer lifts the family to the pose's step (ns.Progression's cause check); the
-- setup, the player's own list or a pain flag refuses it as any Start. Where a climb on the way to the pose's step
-- waits for another step (ns.CheckFirst, 2026-10-17), that step's Start runs first (needStart: its own check, or its
-- record's move).
local function checkStart(state, f, setup, now, depth, need)
	if depth > 4 then return nil, "setup", { ladder = need.ladder } end -- the library never chains checks this far
	local first = ns.CheckFirst(state, need)
	if first ~= need then return needStart(state, f, setup, now, depth + 1, first) end
	local id = ns.NeedAt(need, setup)
	local m = id and ns.Library[id]
	if not (m and POSE[m.ladder]) then return nil, "setup", { ladder = need.ladder } end
	if not ns.Passes(id, setup) then return nil, refusal(id, setup) end
	if not ns.Passes(id, f) then return nil, "pain", subjectOf(id) end
	local s = started(state, f, now, id, "check", m.ladder, m.rung)
	s.ask = ns.CheckAsk(need)
	return s
end

-- The Start of a step another ladder's need or a move's check waits for (`o`, an entry: an `any` group's first entry
-- whose move the setup offers, ns.EntryFor): a pose step or goal pose its check (checkStart, `via` check), else that
-- step's own Start, or its named move's (`via` needs).
needStart = function(state, f, setup, now, depth, o)
	o = ns.EntryFor(o, setup)
	if ns.IsCheck(o) then return tagged("via", "check", checkStart(state, f, setup, now, depth, o)) end
	local target = o.move and { move = o.move } or { ladder = o.ladder, rung = ns.RungAtStep(o.ladder, o.step) }
	return tagged("via", "needs", startAt(state, f, setup, target, now, depth + 1))
end

-- The Start of a move that waits for what opens it (ns.MoveOpen's answer `why`, 2026-10-16): its check (needStart), or
-- the move whose top opens its stage of the path (`via` path).
local function waitingStart(state, f, setup, now, depth, why)
	if why.kind == "check" then return needStart(state, f, setup, now, depth, why.need) end
	return tagged("via", "path", startAt(state, f, setup, { move = why.after }, now, depth + 1))
end

-- The player's own step on ladder `l` (rung `cur`) as the pumps offer it with filter context `f`: on the cardio
-- ladder a counted rung's form as a counted set (ns.CountedForm) or a timed rung's own steady move; elsewhere the move
-- standing for the rung (ns.RungMove: a stand-in from below where the filters shut the rung's own, the next of its
-- chain where the player stepped down from one or one waits for its check) and where they stepped down from every one
-- the easier move they stepped down to. Returns the move, whether it is a counted set and whether its slot is the
-- player's rung's, as the pumps dose it (on a pose family only its main pose, ns.MainPose: a stand-in from below
-- runs at its own step, Engine.lua candidates); nil where none passes, nil and the easier move where the filters
-- shut that one, and nil, nil and the move where the one standing waits for what opens it (ns.MoveOpen).
local function ownMove(state, f, l, cur)
	if l == ns.CARDIO then
		local form = ns.CountedForm(state, cur, f)
		if form then return form, true, true end
		local id = cardioRungMove(cur, f, true)
		if id then return id, false, true end
		return nil
	end
	local id = ns.RungMove(l, cur, f, function(x) return leftOut(state, x, f) end)
	if id then return id, false, not POSE[l] or ns.MainPose(l, cur, id) end
	local standing = ns.RungMove(l, cur, f)
	if not standing then return nil end
	if not ns.MoveOpen(state, standing, f) then return nil, nil, standing end
	local e = easierOf(state, standing)
	if not ns.Passes(e, f) then return nil, e end
	return e, false, false
end

-- Why the player's own step starts nothing (ownMove found no move under the pain flags): "pain" where the setup alone
-- leaves it one, that move the subject; else the setup's or the player's own list's refusal of the easier move the
-- player stepped down to where the setup shuts that one, else of the rung's anchor.
local function ownRefusal(state, setup, l, cur)
	local id, shut = ownMove(state, setup, l, cur)
	if id then return "pain", subjectOf(id) end
	return refusal(shut or ns.Anchor(l, cur), setup)
end

-- A Start of the player's step's own move (`as`: own, place, earn or climb), dosed at the player's rung where the
-- pumps dose it there, any other at its own rung, its kind what its set can do: a set dosed at the player's rung
-- places a ladder (DB.lua anchorAttempt: a move standing for the rung), and only a move of the rung's own level
-- climbs it, which alone shows the climb's day (a stand-in from below never counts one); an orange or gold step
-- earns only by a fresh set of a move of its own level at its floor (ns.EarnFloor, `earn`, kept beside "own" while
-- the set is not fresh) or, on a pose family, its main pose while no goal pose of the step was held today
-- (ns.MarkedToday); any other is "own". A move standing
-- for the step that waits for what opens it starts that (waitingStart).
local function ownStart(state, f, setup, now, depth, l, cur, as, placing)
	local id, counted, atRung = ownMove(state, f, l, cur)
	if not id then
		-- ownMove's third value without a move: the move standing for the step, which waits for what opens it
		if atRung then return waitingStart(state, f, setup, now, depth, select(2, ns.MoveOpen(state, atRung, f))) end
		return nil, ownRefusal(state, setup, l, cur)
	end
	local m = ns.Library[id]
	local own = atRung and (l == ns.CARDIO or m.rung == cur)
	if as == "climb" and not own or as == "place" and not atRung then as = "own" end
	local s, why, subject = startOrJump(state, f, setup, now, depth, id, as, m.ladder or l, atRung and cur or m.rung,
		counted, atRung and placing or nil, own)
	if s and as == "earn" and s.move == id then
		s.earn = ns.EarnFloor(l, cur, id)
		local can
		if POSE[l] then
			can = own and not ns.MarkedToday(state, l, now)
		elseif l == ns.CARDIO then
			can = counted and s.freshFrom == nil
		else
			can = s.earn ~= nil and s.freshFrom == nil
		end
		if not can then s.as = "own" end
	end
	return s, why, subject
end

-- The climb's path (ns.PathTurn: the move of the first stage still out nearest its days), at its own rung (a cardio
-- form's at the cardio rung, a counted set), with the day it counts again (ns.DayIn: a pose of the path held today,
-- which Engine.lua markSlot asks once a game day).
local function pathStart(state, f, setup, now, depth, l, cur, turn)
	local s, why, subject = startOrJump(state, f, setup, now, depth, turn, "path", l,
		l == ns.CARDIO and cur or ns.Library[turn].rung, l == ns.CARDIO)
	if s and s.move == turn then s.dayIn, s.dayKind = ns.DayIn(state, l, now, f) end
	return s, why, subject
end

-- The climb from the player's step `cur` (a step ahead's Start), in the order of what it lacks: the days (the
-- step's own move), then the path (its turn, or where a check holds every move of it that the setup offers, what
-- opens the first: ns.PathHeld, waitingStart, 2026-10-17), then the step of another ladder it waits for (the rung's
-- `needs`, ns.NeedDone: that step's own Start, or its named move's, `via` needs; a pose step or goal pose its check,
-- `via` check), then a counted cardio step's proving window, else the step's own move.
local function climbStart(state, f, setup, now, depth, l, cur)
	if not daysIn(state, l, cur, f) then return ownStart(state, f, setup, now, depth, l, cur, "climb") end
	local turn = ns.PathTurn(state, l, f, cur)
	if turn then return pathStart(state, f, setup, now, depth, l, cur, turn) end
	local holding, why = ns.PathHeld(state, l, f, cur)
	if holding then return waitingStart(state, f, setup, now, depth, why) end
	local w = ns.Ladders[l].rungs[cur + 1]
	for _, o in ipairs(w and w.needs or EMPTY) do
		if not ns.NeedDone(state, o) then return needStart(state, f, setup, now, depth, o) end
	end
	-- a counted cardio step's proving window (slice 53, ns.Proving): the step above's form as a counted set at the
	-- player's step once it is due, the step's own move until then, which counts nothing toward the climb
	local proving, form = nil, nil
	if l == ns.CARDIO then proving, form = ns.Proving(state, f, now) end
	if proving == "due" then return started(state, f, now, form, "climb", l, cur, true, nil, true) end
	return ownStart(state, f, setup, now, depth, l, cur, proving and "own" or "climb")
end

-- The first level from `from` to `to` on ladder `l` that the setup (`setup`: no pain flag) leaves no move: nil, or its
-- refusal (setup or off) and its anchor. A stand-in from below never climbs a level, so a shut one stops every step
-- above it.
local function shutOnTheWay(l, from, to, setup)
	for k = from, to do
		if not levelMove(nil, l, k, setup) then return refusal(ns.Anchor(l, k), setup) end
	end
	return nil
end

-- A step `(l, r)`.
local function stepStart(state, f, setup, now, depth, l, r)
	local lad = ns.Ladders[l]
	if not (lad and lad.rungs[r]) or ns.HeldTop(l) == 0 then return nil, "setup", { ladder = l, rung = r } end
	local cur = ns.Rung(state, l)
	-- a family not open yet: the climb on the ladder that opens it (the first step it waits for not done: a check
	-- where it names a pose, 2026-10-16), or that step's own move once the player stands on it (the handstand at a
	-- Push-up record of 15)
	if not ns.Open(state, l) then
		local why, subject = shutOnTheWay(l, math.min(cur, r), r, setup)
		if why then return nil, why, subject end
		local _, _, o = ns.OpensAt(l, state)
		if ns.IsCheck(o) then return tagged("via", "opens", checkStart(state, f, setup, now, depth, o)) end
		local target = o.move and { move = o.move } or { ladder = o.ladder, rung = ns.RungAtStep(o.ladder, o.step) }
		return tagged("via", "opens", startAt(state, f, setup, target, now, depth + 1))
	end
	if r < cur then -- a step the player has: its own move at its own rung, for its record and its XP, never a climb
		local id, allout, waiting = levelMove(state, l, r, f)
		if waiting then return waitingStart(state, f, setup, now, depth, select(2, ns.MoveOpen(state, waiting, f))) end
		if not id then
			local why, subject = shutOnTheWay(l, r, r, setup)
			if why then return nil, why, subject end
			local pick, shut = levelMove(state, l, r, setup)
			if pick then return nil, "pain", subjectOf(pick) end
			return nil, refusal(shut or ns.Anchor(l, r), setup)
		end
		local m = ns.Library[id]
		return startOrJump(state, f, setup, now, depth, id, "held", m.ladder or l, m.rung or r, allout)
	end
	local placing = STRENGTH[l] and ns.Placing(state, l, now)
	local lst = state.ladders and state.ladders[l]
	local unplaced = STRENGTH[l] and (placing or not (lst and lst.placedTs))
	-- the step ahead and every step on the way, and the player's own level (a stand-in from below never climbs it):
	-- one the setup shuts starts nothing (the card names the rule; a pain flag only holds the climb); a step the player
	-- reached before a step down (ns.BestRung keeps it) climbs back to it (`back`), a placement aside
	if r > cur then
		local why, subject = shutOnTheWay(l, cur, r, setup)
		if why then return nil, why, subject end
		if unplaced then return ownStart(state, f, setup, now, depth, l, cur, "place", placing) end
		return tagged("back", r <= ns.BestRung(state, l) or nil, climbStart(state, f, setup, now, depth, l, cur))
	end
	if unplaced then return ownStart(state, f, setup, now, depth, l, cur, "place", placing) end
	-- the player's step: an orange or gold step to earn (ns.ToEarn), its path's turn once the days are in (or what
	-- opens the path's move where a check holds it), else its move
	if ns.ToEarn(state, l, f) == cur then
		return ownStart(state, f, setup, now, depth, l, cur, "earn")
	end
	local climbing = daysIn(state, l, cur, f) and ns.CanClimb(l, cur, setup)
	local turn = climbing and ns.PathTurn(state, l, f, cur)
	if turn then return pathStart(state, f, setup, now, depth, l, cur, turn) end
	-- a path whose every move the setup offers waits for what opens it (2026-10-17): that, as a step ahead's does
	local holding, why = nil, nil
	if climbing then holding, why = ns.PathHeld(state, l, f, cur) end
	if holding then return waitingStart(state, f, setup, now, depth, why) end
	return ownStart(state, f, setup, now, depth, l, cur, "own")
end

-- A move's window: the setup and the player's own list refuse first (a pain flag refuses only a move the Start would
-- run itself); a ladder move at or under the player's rung starts itself at its own goal and rung (the pumps offer
-- ladder moves only at the rung and the one below, so a next-rung anchor, within ns.Reach, never starts early; a move
-- of the rung's own chain on a ladder not placed, or back from a break, starts its placement, as its set places it),
-- a move above runs its ladder's climb (a family not open yet, its opening's); a move that waits for what opens it
-- (ns.MoveOpen, 2026-10-16) starts that, its check or the move whose top opens its stage (waitingStart); a cardio form
-- runs as the pumps run it: one that only ever runs all out (a countable form, an endOnly one) as a counted set or
-- window, alone, from its finisher rung, another as a steady move at or under the cardio rung and above it as the
-- all-out end its finisher rung lets it be (Engine.lua finisherMove), credited to the rung's climb (own) when it is a
-- form of the rung itself or a window it runs at a timed rung; a form whose rung no pump reaches yet runs the cardio
-- climb; a move off the ladders within the rung rule starts itself, one above it the climb of the ladder it scales by
-- (`via` reach); a jump before its cardio rung (`crung`) the cardio climb; a move the player stepped down from its
-- easier move.
local function moveStart(state, f, setup, now, depth, id)
	local m = ns.Library[id]
	if not m then return nil, "setup", { move = id } end
	if not ns.Passes(id, setup) then return nil, refusal(id, setup) end
	local l = m.ladder
	if l and not ns.Open(state, l) then return stepStart(state, f, setup, now, depth, l, m.rung) end
	local scale = l or ns.ScaleLadder(m)
	local cur = scale and ns.Rung(state, scale)
	local window = l == ns.CARDIO and (m.count ~= nil or m.endOnly == true)
	local at = m.rung
	if window then
		at = m.finisher or m.rung
	elseif l == ns.CARDIO then
		at = math.min(m.finisher or m.rung, m.rung)
	end
	local reach = (l or scale == nil) and 0 or ns.Reach(scale)
	if scale and at > cur + reach then
		local target = { ladder = scale, rung = at - reach }
		if scale ~= l then return tagged("via", "reach", startAt(state, f, setup, target, now, depth + 1)) end
		return startAt(state, f, setup, target, now, depth + 1)
	end
	local open, why = ns.MoveOpen(state, id, f)
	if not open then return waitingStart(state, f, setup, now, depth, why) end
	if not ns.Passes(id, f) then return nil, refusal(id, setup) end
	if stepped(state, id, nil) then -- the easier move the player stepped down to, at its own rung
		local e = easierOf(state, id)
		if not ns.Passes(e, f) then return nil, refusal(e, setup) end
		local em = ns.Library[e]
		return startOrJump(state, f, setup, now, depth, e, "held", em.ladder, em.rung)
	end
	if l == ns.CARDIO then
		local allout = window or m.rung > cur
		-- a window on a rung the setup leaves no countable form of its own is the rung's own; the setup alone decides it,
		-- never a pain flag (2026-10-17, slice 42's review)
		local own = m.rung == cur or allout and #ns.CountForms(cur, setup) == 0
		return startOrJump(state, f, setup, now, depth, id, own and "own" or "held", l, (own or allout) and cur or m.rung,
			allout, nil, own)
	end
	if not l then return startOrJump(state, f, setup, now, depth, id, "own", nil, m.rung) end
	local lst = state.ladders and state.ladders[l]
	local placing = STRENGTH[l] and ns.Placing(state, l, now)
	if STRENGTH[l] and (placing or not (lst and lst.placedTs)) and ns.InChain(l, cur, id) then
		return startOrJump(state, f, setup, now, depth, id, "place", l, cur, false, placing, true)
	end
	-- the move standing for the step, whose set counts the climb's day (on a pose family any goal pose of the step,
	-- ns.Marks, slice 47)
	local stands = m.rung == cur and (POSE[l] and ns.Marks(l, cur, id) or not POSE[l] and ns.InChain(l, cur, id))
	return startOrJump(state, f, setup, now, depth, id, m.rung == cur and "own" or "held", l, m.rung, false, nil, stands)
end

startAt = function(state, f, setup, target, now, depth)
	if depth > 4 then -- the library's links never chain this far; a guard against a cycle
		return nil, "setup", { move = target.move, ladder = target.ladder, rung = target.rung }
	end
	if target.check then return checkStart(state, f, setup, now, depth, target.check) end
	if target.move then return moveStart(state, f, setup, now, depth, target.move) end
	return stepStart(state, f, setup, now, depth, target.ladder, target.rung)
end

-- What a Start from a step's card or a move's window starts (slice 36; David, 2026-10-13: "a use should be able to
-- start a move from the tree anytime"), the one rule for every surface: `target` a step, { ladder, rung }, a move, {
-- move }, or since 2026-10-16 a check, { check = need } (a pose need's entry: ns.IsCheck, "Do the check"). The step the
-- player stands on starts its move (place on a strength ladder not placed yet or back from a break: its max set; earn
-- on an orange or gold step no set earned yet, ns.ToEarn, whose fresh set at its floor earns it; path while the days at
-- its top are in and the climb waits for its path, the move the pumps would give that turn; else own); a step the
-- player has starts its own move at its own rung (held), never a climb; a step ahead runs the climb from the player's
-- step (climb, or path, or the climb of the other ladder's step it waits for), never the step's own move early (the
-- readiness rule, 2026-10-10), `back` where the player reached it before a step down; a family not open yet runs the
-- climb that opens it (`via` opens), a step another climb waits for that one (`via` needs; a pose step or goal pose its
-- check, `via` check), a move that waits for a step of the ladder it scales by (a jump before its cardio rung, a move
-- off the ladders above the rung rule) that ladder's climb (`via` reach), a move that waits for what opens it its check
-- or the move whose top opens its stage (`via` check, `via` path). A check runs its pose at its own step with the least
-- answer it counts (`ask`). A kind promises only what the set can do (ownStart): "own" where it cannot place, climb or
-- earn. Returns { move, allout, as = own|held|climb|path|earn|place|check, ladder, rung, goal, goalKind, freshFrom,
-- tired, dayIn, dayKind, earn, ask, via, back }, or nil, why and its subject { move, ladder, rung, off }: "setup" or
-- "off" for a step the setup or the player's own list shuts (on the way to it too: the level that stops it, its anchor
-- the subject), "pain" where a pain flag shuts the move the setup alone would run (Start stays for a move of the
-- ladder that loads no flagged joint). Nothing starts a move above the rung rule, and a counted set is its window alone
-- (Engine.lua's chosen compose, which takes `move`, `allout`, `rung` and `ask` as they are).
function ns.StartFor(state, profile, target, now)
	local f = ns.FilterContext(profile, state)
	return startAt(state, f, ns.FilterContext(profile, nil), target, now or ns.now(), 0)
end

-- The Long Haul's place in long_haul_s for a cardio row or a look at `ts`, from the cardio state `c` (the double-check
-- of slices 52-55, 2026-10-24): more than ledger_layoff_days game days after the last cardio move (lastTs), at any
-- step, its place above the first length fades as the steady ledger's gain does (ns.LayoffKeep), to the nearest length
-- (180 s stays 180 s through 14 game days away, 150 s from 15, 120 s from 30, 90 s from 45), and its days start again
-- (`faded` true). DB.lua cardioMove folds it so at the next cardio row; the readers see it so until then (ns.Haul), as
-- ns.Placing reads a break.
function ns.HaulPlace(c, ts)
	local place = c.haul or 1
	local keep = c.lastTs and ns.LayoffKeep(ns.LedgerDay(ts) - ns.LedgerDay(c.lastTs))
	if not keep then return place, false end
	local fade = C.ledger_fade_days
	return 1 + math.floor(((place - 1) * keep * 2 + fade) / (2 * fade)), true
end

-- The Long Haul (slice 55; David's call 1, 2026-10-23: the one exception to "cardio is short and hard, 20-45 s"), the
-- player's own on the cardio step's card, at `now`: its length open in seconds (long_haul_s at the fold's place, DB.lua
-- haulMove, faded by a layoff, ns.HaulPlace), the game days with a Yes at it, the days that open the next
-- (long_haul_days) and the next length (nil at the longest).
function ns.Haul(state, now)
	local c = state.cardio
	local place, faded = ns.HaulPlace(c, now or ns.now())
	local i = math.min(place, #C.long_haul_s)
	return C.long_haul_s[i], not faded and c.haulDays or 0, C.long_haul_days, C.long_haul_s[i + 1]
end

-- The cardio day a set's row brought (slice 42; Events.lua Lc.WriteResult's `day`, which the game's Result and the
-- desktop app's, tools/apppump.lua Engine:finish, show), read from `state` right after the row folded (its fold marks:
-- DB.lua cardioMove, steadyRow) and before a climb its rung rows make resets the counts: { kind = "clean" or "full",
-- have, need, step } toward the next cardio step (a full move since slice 52, a ready day until then; a climb back
-- takes return_days, slice 53), or a Long Haul's that counted a day (DB.lua haulMove's haulDay), { kind = "haul", time,
-- have, need, next } toward its next length (its seconds; none at the longest, slice 55); since slice 53 a step's
-- measure back from a break, { kind = "holds" or "back", step } (the step it holds, the step it goes back to), or a
-- proving window that did not pass, { kind = "waits" } ("No step up yet."); or nil. A clean day on a rung that names
-- no clean days (the top's tuck burpee) says nothing, never a full move's count.
function ns.CardioDay(state, row)
	if ns.HaulRow(row) then
		-- all in where this one opened it
		local open, have, need, nextS = ns.Haul(state, row.ts)
		if row.haulUp then have, nextS = need, open end
		return row.haulDay and nextS and { kind = "haul", time = row.reps, have = have, need = need, next = nextS } or nil
	elseif row.holds ~= nil then
		local rung = ns.Rung(state, ns.CARDIO)
		return { kind = row.holds and "holds" or "back", step = ns.Step(ns.CARDIO, row.holds and rung or rung - 1) }
	elseif row.proving and not row.proved then
		return { kind = "waits" }
	elseif row.cleanDay or row.fullMove then
		local rung, clean = ns.Rung(state, ns.CARDIO), row.cleanDay == true
		local need = clean and ns.CleanDays(rung, state)
			or not clean and (ns.Returning(state, ns.CARDIO, rung) and C.return_days or C.cardio_full_days) or nil
		local have = clean and state.cardio.clean or state.cardio.full
		local step = ns.Step(ns.CARDIO, rung + 1)
		if step and need then
			return { kind = clean and "clean" or "full", have = math.min(have, need), need = need, step = step }
		end
	end
	return nil
end

-- The Long Haul's form under filter context `f` from cardio rung `cur`: the steady move of the player's step where the
-- step is timed (its card's own Start, cardioRungMove), else of the highest timed step under it (a counted step's
-- Start is its 30-s window, whose forms are built to go all out: from step 5 the haul runs step 4's high knees, a pace
-- a 90-180-s move holds), at the latest step 1's, whose step jacks (ns.FLOOR_MOVE) pass every filter.
local function haulForm(f, cur)
	for r = cur, 1, -1 do
		if not ns.CountedRung(r, f) then
			local id = cardioRungMove(r, f, true)
			if id then return id end
		end
	end
	return ns.FLOOR_MOVE
end

-- A Long Haul's Start from the player's cardio step's card, as ns.StartFor answers a Start: { move, as = "haul",
-- ladder, rung (the player's cardio rung, as the pumps dose a steady move), haul (its seconds, ns.Haul) }, which
-- Engine.lua's chosen compose runs alone, its pace asked every time. No setup or pain flag refuses it (haulForm), it
-- is never rationed, and nothing starts it but the player.
function ns.HaulStart(state, profile, now)
	local f = ns.FilterContext(profile, state)
	local cur = ns.Rung(state, ns.CARDIO)
	local s = started(state, f, now or ns.now(), haulForm(f, cur), "haul", ns.CARDIO, cur)
	s.haul = (ns.Haul(state, now))
	return s
end

-- The ring around the player's step (slice 36), in per mille from 0 to 1000 (whole numbers, as the phones' ports): a
-- strength or hold step's record (`record` of `move`, the move standing for rung `rung`) up its window for the first
-- half, then the climb's days of those it takes for the second half (`p`: ns.UnlockProgress's have, need, have2,
-- need2), with its path one more where the climb waits for one (in once have2 is 0); a pose step's mark days and a
-- counted cardio step's clean days over the whole ring, with the path's pip; a timed cardio step's full moves over the
-- whole ring (slice 52; its ready days from 2026-10-16, when a counted step's form's record up its count's window for
-- the first half and a timed step's finished moves then their days went). A closed climb (need 0: a step the setup
-- shuts or one that waits for another ladder) fills only a strength step's first half; a ladder's top, which no climb
-- is left from, fills whole on a pose family and the cardio ladder and, on a strength or hold ladder, its record up its
-- window over the whole ring. Returns the fill and, on a strength or hold step, the ticks of its stars (ns.StarLine) on
-- the span its record fills; nil while a strength or hold step has no mark (no record and no day yet: "Your first set
-- sets your mark").
local RING, HALF = 1000, 500

local function part(num, den, span)
	if den <= 0 then return 0 end
	return math.floor(span * math.max(0, math.min(num, den)) / den)
end

local function climbPart(ladder, rung, have, need, open, span)
	if need <= 0 then return 0 end
	local pip = #ns.Path(ladder, rung) > 0 and 1 or 0
	return part(math.min(have, need) + (open == 0 and pip or 0), need + pip, span)
end

-- A strength or hold step's record up the window of `move` on `span` and its stars' ticks; 0 and no ticks where the
-- window is not in the move's own unit (ns.OwnUnit).
local function recordPart(ladder, rung, move, record, span)
	local lo, hi, own = ns.Window(ladder, rung, move or 0)
	if not (move and ns.Library[move] and lo and ns.OwnUnit(move, ns.Anchor(ladder, rung), own)) then return 0, nil end
	local ticks = {}
	for i, star in ipairs(ns.StarLine(lo, hi)) do ticks[i] = part(star - lo, hi - lo, span) end
	return part((record or 0) - lo, hi - lo, span), ticks
end

function ns.StepRing(ladder, rung, move, record, p)
	local top = rung >= ns.HeldTop(ladder)
	if top and (ladder == ns.CARDIO or POSE[ladder]) then return RING end
	local have, need, have2 = p[1] or 0, p[2] or 0, p[3] or 0
	if POSE[ladder] or ladder == ns.CARDIO then return climbPart(ladder, rung, have, need, have2, RING) end
	if (record or 0) <= 0 and have == 0 then return nil end
	if top then return recordPart(ladder, rung, move, record, RING) end
	local first, ticks = recordPart(ladder, rung, move, record, HALF)
	return first + climbPart(ladder, rung, have, need, have2, HALF), ticks
end

-- A ladder's entry in its tree (slice 36), the one reading the game's Talents tab and every Book take: `src` the
-- ladder's numbers as the tree shows them (the game's sheet where it is ahead of the app's own, tools/apppump.lua's
-- source): { open (its family open), rung (its rung, 0 before any), best (its best rung, ns.BestRung), progress
-- (ns.UnlockProgress's four), standing (the move standing for its rung), record (that move's record), placed }, with
-- `f` the filters, `setup` the setup alone, at `now`. Returns { rung, best, waits, placing, placingBy, open, pain,
-- dayIn, dayKind, ring, ticks, move, record }: `waits` where the ladder waits whole, "closed" for a family not open
-- yet, else the setup's refusal of the anchor of a ladder it leaves no move (setup or off; the cardio ladder never: its
-- card names what the setup leaves its step); `placing` on a strength ladder back from a break (ns.Placing), whose
-- first set finds its step and wipes the days it counted, and since slice 53 on the cardio ladder, whose first measure
-- does, with `placingBy` what measures it there (DB.lua cardioMeasure): "window" from closing_rung, where every pump
-- closes on an all-out window, else "steady", a steady move at the step; `open` where a set can move its climb
-- (placed, a climb open, not placing); `pain` where a flag holds a climb the setup alone leaves open (ns.CanClimb;
-- ns.PainOnLadder, on the cardio ladder a flag that leaves its step or the next no form the setup has); when its day
-- can count again (ns.DayIn); and the ring around its step (ns.StepRing: its fill and ticks, the move and record
-- filling it, a counted cardio step's form, ns.CountedForm), none while it waits or is placing.
function ns.TreeEntry(state, l, src, f, setup, now)
	local e = { rung = src.rung, best = src.best }
	if not src.open then
		e.waits = "closed"
		return e
	end
	local r = math.max(1, src.rung)
	if l ~= ns.CARDIO and not ns.RungMove(l, r, setup) then
		e.waits = (refusal(ns.Anchor(l, r), setup))
		return e
	end
	local p = src.progress
	e.placing = (STRENGTH[l] and src.placed or l == ns.CARDIO) and ns.Placing(state, l, now) or nil
	if e.placing and l == ns.CARDIO then e.placingBy = r >= C.closing_rung and "window" or "steady" end
	e.open = src.placed and p[2] > 0 and not e.placing
	e.pain = ns.CanClimb(l, r, setup, state) and ns.PainOnLadder(state, l, r, f)
	e.dayIn, e.dayKind = ns.DayIn(state, l, now, f)
	if not e.placing then
		local move, record = src.standing, src.record
		if l == ns.CARDIO then move, record = ns.CountedForm(state, r, f) end
		e.ring, e.ticks = ns.StepRing(l, r, move, record, p)
		e.move, e.record = move, record
	end
	return e
end

-- The step that glows in tree `tree` (slice 36; one a tree, static), from its ladders' entries (`entries` by ladder
-- id, ns.TreeEntry): the player's step on the ladder whose ring is the highest among the climbs a set can move now
-- (open, its day not in, no pain flag holding it), a tie by the tree's own column order ("climb"); with none, the
-- tree's ladder the Trial places next (`trial`, ns.TrialNext with the tree), whose first set finds its step ("trial"),
-- so each tree with a ladder to place says so. Returns the ladder and
-- its kind, or nil and why none glows, in this order: "rest" while felt unwell locks every pump (`locked`,
-- ns.LockedToday); "day" where a climb's day is in, with the earliest moment one counts again (ns.DayIn); "pain"
-- where a flag holds a climb; "break" where a ladder back from a break waits for the set that finds its step; else
-- "none" (every climb at its top, closed or waiting).
function ns.TreeGlow(tree, entries, trial, locked)
	if locked then return nil, "rest" end
	local best, top
	for _, l in ipairs(ns.TreeLadders(tree)) do
		local e = entries[l]
		if e and e.open and e.ring and not e.dayIn and not e.pain and (not best or e.ring > top) then
			best, top = l, e.ring
		end
	end
	if best then return best, "climb" end
	if trial then return trial, "trial" end
	local day, pain, rest
	for _, l in ipairs(ns.TreeLadders(tree)) do
		local e = entries[l]
		if e then
			if e.open and e.dayIn and not e.pain then day = math.min(day or e.dayIn, e.dayIn) end
			pain, rest = pain or e.pain == true, rest or e.placing == true
		end
	end
	if day then return nil, "day", day end
	if pain then return nil, "pain" end
	if rest then return nil, "break" end
	return nil, "none"
end

-- The rows a tree draws in full (slice 36), by step from 1 (Starter, the board's top) to the scale's last: true where
-- a ladder of the tree holds a step there that the player has (its best rung, `best`) or that is within reach (their
-- step, `rung`, and the next), false for a thin row, each of its steps a dot on its ladder's line. A ladder that waits
-- whole (`waits`: closed, setup or off) is a note under its name and fills no row. `entries` by ladder id
-- (ns.TreeEntry). Returns the rows and how many steps the thin rows hold (the expander's count).
function ns.TreeRows(tree, entries)
	local rows, hidden, ladders = {}, 0, ns.TreeLadders(tree)
	for step = 1, #C.quality_by_step do rows[step] = false end
	for _, l in ipairs(ladders) do
		local e = entries[l]
		for r, w in ipairs(e and not e.waits and ns.Ladders[l].rungs or {}) do
			if r <= (e.best or 0) or r <= (e.rung or 0) + 1 then rows[w.step] = true end
		end
	end
	for _, l in ipairs(ladders) do
		local e = entries[l]
		for _, w in ipairs(e and not e.waits and ns.Ladders[l].rungs or {}) do
			if not rows[w.step] then hidden = hidden + 1 end
		end
	end
	return rows, hidden
end
