"""Motion files (`contract/art/<move>.json`) baked into 30 Hz tracks (pure Python; docs/plan.md slice 10).

A motion file names its poses and a timeline:

    {"move": "air_squat", "view": "side", "camera": {"yaw": 40, "pitch": 8},
     "props": [], "items": [], "working": ["thigh"],
     "poses": {"tall": {...}, "deep": {"from": "tall", ...}, "heels_up": {"from": "deep", ...}},
     "timeline": [{"pose": "tall"}, {"hold": 0.4, "phase": "tall"},
                  {"to": "deep", "s": 1.2, "phase": "sit_back", "effort": 0.6}, ...],
     "key": "deep",
     "fault": {"pose": "heels_up", "caption": "heels_lift",
               "breaks": {"check": "offset", "joint": "ankle_near", "from": "toe_near", "axis": "u", "max": 2}},
     "targets": [{"at": "deep", "check": "height", "joint": "pelvis", "max": 40}, ...], "mirror": false}

"working" names the armour parts whose runes light with the effort; "ballistic": true lifts a rep's phase bounds (a
jump, a swing); "breathe": true cross-fades a hold's cells; "fast_phase" (0-1) narrows the pack's fast-phase allowance
(`pack.REL`, 1: a stored frame stands in for the ticks up to one tick of motion away) for a move that is fast all
through its loop (butt kicks), which the default leaves a few frames each held for several ticks; "eyes": "closed"
darkens the visor's light for the whole loop (a balance held with the eyes shut); "rep_ends" names the phases whose end
finishes a rep where one loop shows more than one (a dead bug's two sides, a monster walk's two steps: each run of those
phases ends a rep, the last at the loop's end; without it the loop is one rep, ended by the loop's end), which the voice
counts (Speech.lua, the rep count); "note" is a line for the review page. A timing variant is a file of its own:
"variant_of" with "phases" (seconds per phase) or "pause", and nothing else of the above.

An extra (`"extra": true`, slice 45) is a figure that is no library move: the streak card's standard-bearer
(`streak_banner`). It bakes and passes every check a move does but the dose's tempo (it has no dose: its name is no
key of contract/moves.json), takes no fault and no rep_ends, and builds into its key pose's still alone, never a pack
entry, a key-pose icon or a film (build.py `build_extra`, publish.py's `extras`): its timeline is usually a pose held.

A gear variant is a file of its own too, `<move>.<gear>.json`, its figure key "<move>.<gear>" (2026-10-09, loaded
moves: the figure of a move whose load a real implement carries shows the one the player has, a dumbbell row's
dumbbell): "gear_of" names the move and "gear" the equipment key (ns.Gear) whose implement it draws, with "items" in
place of the move's and, where the implement sits in other hands, "poses" whose arm fields ("arms", "arm_near",
"arm_far") replace those of the move's poses of the same names (two dumbbells held apart where a bag hung from both
hands together); every other pose field, the timeline, the key, the fault, the targets and the camera are the move's
(`geared`). The gear variant of a timing variant names no items or poses: it is that timing variant of its reference's
gear variant of the same gear. Its figure is its own, rendered and packed like any move's; the key-pose icon stays the
move's.

A pose: `pelvis` [f, u, l] cm; `trunk`, `chest`, `head` [pitch, roll, yaw] deg (chest and head relative to the one
below; pitch leans forward, roll toward the far side, yaw turns toward the far side); `arms` or `arm_near`/`arm_far`
and `legs` or `leg_near`/`leg_far`, each a limb spec; `contacts`, the joints allowed to touch the floor or a prop;
`from`, a pose to start from (its fields are overridden); `line`, a helper that places a straight body over its feet
or knees (planks, push-ups); `fit`, a helper that raises or lowers the pelvis until a joint stands at a height.

Limb specs (lateral values are outward distances, positive away from the midline):
  arm  {"fk": [flex, abduct, elbow, twist, wrist]}                  angles, deg; flex 0 hangs, 90 forward, 180 up
       {"ik": [f, u, out], "pole": [f, u, out], "hand": [f, u, out]}  the wrist to a point, the elbow toward the pole
       {"ik": [f, u, out], "rel": "neck", ...}                      the point (and the hand's direction) in the chest's
                                                                    frame from a spine joint (a bar on the back rides
                                                                    with the torso)
       {"ik": [f, u, out], "orbit": {"r": 17, "deg": 90, "plane": "fu"}, ...}   the point on a circle of radius r
                                                                    about ik, at deg in the plane named ("fu": 0
                                                                    forward, 90 up; "fl": 0 forward, 90 out; "ul": 0
                                                                    up, 90 out); poses blend deg, so the hand or foot
                                                                    runs round the circle (a pedal stroke, an arm
                                                                    circle, a halo)
  leg  {"fk": [flex, abduct, knee, ankle]}                          ankle 0: the foot square to the shin
       {"foot": [f, out], "y": 0, "turn": 8, "lift": 0, "knee_out": 6, "pole": [...]}   planted on a surface
       {"ik": [f, u, out], "pole": [f, u, out], "toe": [f, u, out]}   the ankle to a point (a foot in the air;
                                                                    `orbit` as for the arm)

Form targets (`targets`, each read at the pose `at` names, or at a time in seconds; `checks.py` holds the loop to
them), body space:
  {"at": "deep", "check": "height", "joint": "pelvis", "max": 40}                  a joint's height (cm)
  {"at": "deep", "check": "angle", "joints": ["hip_near", "knee_near", "ankle_near"], "max": 55}
                                                                    the angle at the middle joint (deg), read in the
                                                                    side view's plane for a side-view move
  {"at": "bottom", "check": "offset", "joint": "shoulder_near", "from": "wrist_near", "axis": "f", "min": -4}
                                                                    the signed distance (cm) of `joint` from `from`
                                                                    along a body axis: "f" forward, "u" up, "l"
                                                                    outward on the joint's own side (a midline
                                                                    joint's: toward the far side)
Each takes `min`, `max` or both.

`fault` is the wrong picture of the New-move view's still right-versus-wrong pair (slice 22), beside the key pose:
`pose` names a pose of the file that need not be on the timeline, `caption` the key of its line in the locale
(`figure.fault.<caption>`, lower_snake), and `breaks` a form target without `at`, which holds at the key pose and fails
at the fault pose, so the picture shows the fault its caption names (`checks.check_fault`). The still stands where the
key pose stands in the loop and is cut with the loop's camera and crop, so the pose stays inside the loop's frame. A
timing variant shares its reference move's.

`bake` returns the track the renderer reads (the prototype's `tracks_armor.json` entry format): per frame `t`,
`phase`, `glow`, `joints` (flat `<joint>_<side>` keys, the view's coordinates), `sash` (five points), `sash_rest`
(how much each lies on a surface, 0-1), `items` (items.place's poses), `held_arms` and `planted_arms` (the arms that
hold an item and the ones resting on a surface: arm_holds), with the camera fitted to the whole loop.
"""
import copy
import json
import math
import os
import re

import items
import props
import rig
from rig import BONES, SIDES, SIGN, add, mul, sub, unit, Frame, two_bone, bent, fk_first

HZ = 30
VIEWS = ("side", "front")
EASES = ("inout", "in", "out", "linear")
ARM_DEPTH, LEG_DEPTH = 26.0, 11.0      # the renderer adds these (cm) to a limb's lateral value in the side view
SOLE_FRONT = BONES["sole_front"]      # the sabaton's front edge, cm past the ankle along the sole
KNEE_FLOOR = rig.CLEARANCE["knee"]     # a kneeling knee's centre above the floor (the poleyn resting on it)
FOOT_TOP = 3.0                         # the sabaton's top above its ankle-toe line (a foot laces down rests on it)
SASH_POINTS, SASH_LINK = 5, 9.0
SASH_R = 1.2                           # cm the cloth keeps off the floor and every prop
SASH_REST = 3.0                        # cm above a surface from which a sash point starts to lie on it
SASH_TRAIL = (-0.6, -1.0, 0.0)         # the way the chain starts from its knot: down and behind
SASH_GRIP = 0.85                       # the share of its sideways speed a point lying on a surface loses each substep
# the near leg's plates as cones the sash keeps off (cm): from, to, the radius at each end (blender/armor_model.py,
# the cuisse's and the greave's side radii)
SASH_LEG = (("hip", "knee", 11.5, 9.0), ("knee", "ankle", 8.5, 8.5))
DEFAULT_POLE_ARM = (-0.35, -1.0, 0.45)
LINE_DEFAULTS = {"pivot": "toes", "width": 10.0, "hips": 0.0, "gap": 0.3, "y": 0.0, "shift": 0.0, "face": "down"}
# A planted foot's fields when a pose omits them; a zero pole means "bend the knee forward, knee_out degrees out"
# (expand makes it that pole, so it blends with another pose's own).
LEG_DEFAULTS = {"y": 0.0, "turn": 8.0, "lift": 0.0, "knee_out": 6.0, "pole": [0.0, 0.0, 0.0]}
TOP_FIELDS = ("move", "view", "camera", "working", "ballistic", "breathe", "fast_phase", "props", "items", "poses",
              "timeline", "key", "fault", "targets", "mirror", "note", "eyes", "rep_ends", "extra")
EXTRA_TAKES_NOT = ("fault", "rep_ends")    # an extra shows no right-versus-wrong pair and counts no reps
VARIANT_FIELDS = ("move", "variant_of", "phases", "pause", "mirror", "note")
GEAR_FIELDS = ("move", "gear_of", "gear", "items", "poses", "note")
GEAR_POSE_FIELDS = ("arms", "arm_near", "arm_far")      # what a gear variant may change in a pose: the holding arms
FIELDS = ("pelvis", "trunk", "chest", "head", "arms", "arm_near", "arm_far", "legs", "leg_near", "leg_far",
          "contacts", "from", "line", "fit")
# A form target's kinds and the fields each takes beside `check`, `min` and `max` (and `at` in `targets`).
TARGET_FIELDS = {"height": ("joint",), "angle": ("joints",), "offset": ("joint", "from", "axis")}
AXES = ("f", "u", "l")
FAULT_FIELDS = ("pose", "caption", "breaks")
CAPTION = re.compile(r"[a-z0-9_]+")


class MotionError(ValueError):
    pass


def rep_counts(ends, tick_phase):
    """The pack entry's `counts`, where a loop shows more than one rep (the motion's `rep_ends`, the phases that end
    one): the tick at which each rep but the last ends, the end of each run of those phases before the loop's own end;
    none for a loop of one rep."""
    if not ends:
        return {}
    n = len(tick_phase)
    counts = [t + 1 for t in range(n - 1) if tick_phase[t] in ends and tick_phase[t + 1] != tick_phase[t]]
    return {"counts": counts} if counts else {}


# --- loading and poses ---------------------------------------------------------------------------------------------

def load(path):
    """A motion file's spec; a gear variant's as the motion it draws (`geared`, from its move's file beside it)."""
    with open(path, encoding="utf-8") as f:
        spec = json.load(f)
    spec.setdefault("move", os.path.splitext(os.path.basename(path))[0])
    validate(spec)
    if spec.get("gear_of"):
        base = os.path.join(os.path.dirname(path), spec["gear_of"] + ".json")
        if not os.path.exists(base):
            raise MotionError(f"{spec['move']}: no motion file {spec['gear_of']} (gear_of)")
        spec = geared(spec, load(base))
    return spec


def drawn(key):
    """(the library move, the gear) a figure key draws: a gear variant's `<move>.<gear>`, else (the key, None)."""
    move, _, gear = key.partition(".")
    return move, gear or None


def geared(spec, base):
    """A gear variant (the module docstring) as the motion it draws: the move's spec `base` under the variant's key,
    with its items and its poses' arms; the gear variant of a timing variant is that timing variant of its reference's
    gear variant."""
    move, gear = spec["move"], spec["gear"]
    if base.get("variant_of"):
        if "items" in spec or "poses" in spec:
            raise MotionError(f"{move}: the gear variant of a timing variant takes no items or poses (it draws "
                              f"{base['variant_of']}.{gear} on {spec['gear_of']}'s table)")
        out = {k: v for k, v in base.items() if k != "note"}
        out.update(move=move, variant_of=f"{base['variant_of']}.{gear}")
        return out
    if not spec.get("items"):
        raise MotionError(f"{move}: a gear variant draws its implement (`items`)")
    out = {k: copy.deepcopy(v) for k, v in base.items() if k != "note"}
    out.update(move=move, items=copy.deepcopy(spec["items"]))
    for name, fields in spec.get("poses", {}).items():
        if name not in out["poses"]:
            raise MotionError(f"{move}: poses: {spec['gear_of']} has no pose {name}")
        for k in fields:
            if k not in GEAR_POSE_FIELDS:
                raise MotionError(f"{move}: poses: {name}: a gear variant changes only the arms that hold the "
                                  f"implement ({', '.join(GEAR_POSE_FIELDS)}), not {k}")
        pose = out["poses"][name]
        if "arms" in fields:                       # the shared arms replace the side-specific ones, as in `from`
            for s in SIDES:
                pose.pop(f"arm_{s}", None)
        pose.update(copy.deepcopy(fields))
    if spec.get("note"):
        out["note"] = spec["note"]
    return out


def validate(spec):
    move = spec.get("move")
    if spec.get("gear_of"):
        for k in spec:
            if k not in GEAR_FIELDS:
                raise MotionError(f"{move}: a gear variant takes only {', '.join(GEAR_FIELDS)}")
        if not isinstance(spec.get("gear"), str) or not CAPTION.fullmatch(spec["gear"]):
            raise MotionError(f"{move}: gear must be an equipment key of a-z, 0-9 and _ (ns.Gear)")
        if "." in spec["gear_of"]:
            raise MotionError(f"{move}: {spec['gear_of']} is a gear variant itself (gear_of names a move)")
        if move != f"{spec['gear_of']}.{spec['gear']}":
            raise MotionError(f"{move}: a gear variant's file is <move>.<gear>.json ({spec['gear_of']}."
                              f"{spec['gear']}.json)")
        if not isinstance(spec.get("poses", {}), dict):
            raise MotionError(f"{move}: poses must map pose names to their arm fields")
        return
    if spec.get("variant_of"):
        if not spec.get("phases") and not spec.get("pause"):
            raise MotionError(f"{move}: a variant sets `phases` (seconds per phase) or `pause`")
        for k in spec:
            if k not in VARIANT_FIELDS:
                raise MotionError(f"{move}: a variant takes only {', '.join(VARIANT_FIELDS)}")
        return
    for k in spec:
        if k not in TOP_FIELDS:
            raise MotionError(f"{move}: unknown field {k}")
    if "extra" in spec:
        if spec["extra"] is not True:
            raise MotionError(f"{move}: extra is true or absent (a figure that is no library move)")
        if not isinstance(move, str) or not CAPTION.fullmatch(move):
            raise MotionError(f"{move}: an extra's name is a-z, 0-9 and _ (its still is stills/<name>.png)")
        for k in EXTRA_TAKES_NOT:
            if k in spec:
                raise MotionError(f"{move}: an extra takes no {k} (no pair, no rep count: it builds a still)")
    if spec.get("view", "side") not in VIEWS:
        raise MotionError(f"{move}: view must be one of {VIEWS}")
    fast = spec.get("fast_phase")
    if fast is not None and (isinstance(fast, bool) or not isinstance(fast, (int, float)) or not 0 <= fast <= 1):
        raise MotionError(f"{move}: fast_phase must be a number from 0 to 1 (the pack's fast-phase allowance)")
    poses, tl = spec.get("poses"), spec.get("timeline")
    if not poses or not tl:
        raise MotionError(f"{move}: poses and timeline are required")
    for name, p in poses.items():
        for k in p:
            if k not in FIELDS:
                raise MotionError(f"{move}: pose {name}: unknown field {k}")
    if "pose" not in tl[0]:
        raise MotionError(f"{move}: the timeline starts with {{\"pose\": ...}}")
    yaw = spec.get("camera", {}).get("yaw", 30.0)
    if abs(yaw) >= 85:
        raise MotionError(f"{move}: camera yaw {yaw} (keep |yaw| under 85: the aim point is solved in the floor plane)")
    for i, st in enumerate(tl[1:], 1):
        if not st.get("phase"):
            raise MotionError(f"{move}: step {i} needs a phase (its caption)")
        if "to" in st:
            if st["to"] not in poses:
                raise MotionError(f"{move}: step {i}: unknown pose {st['to']}")
            if "s" not in st:
                raise MotionError(f"{move}: step {i}: `s` (seconds) missing")
            if st.get("ease", "inout") not in EASES:
                raise MotionError(f"{move}: step {i}: ease must be one of {EASES}")
        elif "hold" not in st:
            raise MotionError(f"{move}: step {i}: needs `to` or `hold`")
    key = spec.get("key")
    if key is not None and key not in poses and not isinstance(key, (int, float)):
        raise MotionError(f"{move}: key must be a pose name or a time (s)")
    ends = spec.get("rep_ends")
    if ends is not None:
        phases = [st["phase"] for st in tl[1:]]
        if not (isinstance(ends, list) and ends and all(e in phases for e in ends)):
            raise MotionError(f"{move}: rep_ends must list phases of the timeline")
        if phases[-1] not in ends:
            raise MotionError(f"{move}: rep_ends must hold the last step's phase (the loop ends on a rep's end)")
    for i, t in enumerate(spec.get("targets", [])):
        validate_target(move, f"target {i}", t, ("at",))
        if t.get("at") not in poses and not isinstance(t.get("at"), (int, float)):
            raise MotionError(f"{move}: target {i}: `at` must be a pose name or a time (s)")
    fault = spec.get("fault")
    if fault is not None:
        if sorted(fault) != sorted(FAULT_FIELDS):
            raise MotionError(f"{move}: fault takes {', '.join(FAULT_FIELDS)}")
        if fault["pose"] not in poses:
            raise MotionError(f"{move}: fault: unknown pose {fault['pose']}")
        if not isinstance(fault["caption"], str) or not CAPTION.fullmatch(fault["caption"]):
            raise MotionError(f"{move}: fault: caption must be a key of a-z, 0-9 and _ (figure.fault.<caption>)")
        validate_target(move, "fault: breaks", fault["breaks"])


def validate_target(move, where, t, more=()):
    """A form target's grammar (the module docstring's "Form targets"); `more` names the fields its place adds."""
    fields = TARGET_FIELDS.get(t.get("check")) if isinstance(t, dict) else None
    if fields is None:
        raise MotionError(f"{move}: {where}: check must be one of {', '.join(TARGET_FIELDS)}")
    for k in t:
        if k not in ("check", "min", "max") + fields + tuple(more):
            raise MotionError(f"{move}: {where}: unknown field {k}")
    for k in fields:
        if k not in t:
            raise MotionError(f"{move}: {where}: needs `{k}` ({t['check']})")
    joints = t["joints"] if "joints" in fields else [t[k] for k in ("joint", "from") if k in fields]
    if "joints" in fields and not (isinstance(joints, list) and len(joints) == 3):
        raise MotionError(f"{move}: {where}: `joints` names three joints (the angle at the middle one)")
    for j in joints:
        if j not in rig.JOINTS:
            raise MotionError(f"{move}: {where}: unknown joint {j}")
    if "axis" in fields and t["axis"] not in AXES:
        raise MotionError(f"{move}: {where}: axis must be one of {', '.join(AXES)}")
    bounds = [t[k] for k in ("min", "max") if k in t]
    if not bounds or not all(isinstance(b, (int, float)) for b in bounds) or bounds != sorted(bounds):
        raise MotionError(f"{move}: {where}: needs `min`, `max` or both (numbers, min <= max)")


def measure(t, J, view):
    """(value, label) of a form target on one pose's joints (body space): a joint's height, the angle at the middle
    of three joints (in the side view's plane for a side-view move: form angles are read as the side view shows them)
    or a joint's signed distance from another along a body axis (lateral: outward on the joint's own side)."""
    if t["check"] == "height":
        v = J[t["joint"]][1]
        return v, f"{t['joint']} height {v:.1f} cm"
    if t["check"] == "angle":
        a, b, c = (J[k] for k in t["joints"])
        if view == "side":
            a, b, c = ((p[0], p[1], 0.0) for p in (a, b, c))
        v = rig.angle(sub(a, b), sub(c, b))
        return v, f"angle {'-'.join(t['joints'])} {v:.0f} deg"
    i = AXES.index(t["axis"])
    v = J[t["joint"]][i] - J[t["from"]][i]
    if t["axis"] == "l" and t["joint"].endswith("_near"):
        v = -v
    return v, f"offset {t['joint']} from {t['from']} along {t['axis']} {v:.1f} cm"


def resolve_pose(spec, name, seen=()):
    """A pose with its `from` chain merged (a later pose's fields replace the earlier one's, limb by limb)."""
    poses = spec["poses"]
    if name in seen:
        raise MotionError(f"{spec['move']}: pose {name} inherits from itself")
    p = copy.deepcopy(poses[name])
    base = p.pop("from", None)
    if base:
        merged = resolve_pose(spec, base, seen + (name,))
        for k in ("arms", "legs"):                 # a side-specific limb in the child beats a shared one inherited
            if k in p:
                for s in SIDES:
                    merged.pop(f"{k[:-1]}_{s}", None)
        for k in ("line", "fit"):
            if k in p:
                merged.pop(k, None)
        merged.update(p)
        p = merged
    return p


def limb_specs(p, kind):
    """{side: spec} for 'arm' or 'leg' from a pose's shared and side-specific fields."""
    out = {}
    for s in SIDES:
        sp = p.get(f"{kind}_{s}", p.get(f"{kind}s"))
        if sp is None:
            raise MotionError(f"pose lacks {kind}s")
        out[s] = sp
    return out


# --- the pose helpers ----------------------------------------------------------------------------------------------

def sole_pivot(lift):
    """Where along the sole a raised foot turns (cm past the ankle). The sabaton is rigid, so a heel raise that
    turned on the ball would sink the toe box into the floor: the pivot moves toward the sole's front edge until the
    edge dips at most 1 cm (at 35 degrees the foot stands 19.3 cm forward, at 90 on its tips). A negative lift
    raises the toes and turns on the heel (heels on a towel)."""
    if lift < 0.0:
        return -BONES["heel_back"]                 # toes up: the foot rocks back onto its heel
    if lift <= 4.0:
        return BONES["ball_forward"]
    return max(BONES["ball_forward"], SOLE_FRONT - 1.0 / math.sin(math.radians(lift)))


def planted(base, heading, lift):
    """(ankle, toe) of a foot on a surface, raised `lift` deg about its sole pivot (see sole_pivot)."""
    pd, ah = sole_pivot(lift), BONES["ankle_height"]
    axis = rig.cross(heading, rig.U0)
    pivot = add(base, mul(heading, pd))
    back = add(mul(heading, -pd), mul(rig.U0, ah))
    ankle = add(pivot, rig.rotate(back, axis, -lift))
    toe = add(ankle, mul(rig.rotate(heading, axis, -lift), BONES["foot"]))
    return ankle, toe


def apply_line(p):
    """`line`, a straight body over its feet or knees (planks, push-ups, leaning rows):
      pivot   "toes" (the feet's base at `at`) or "knees" (the knees on the floor at `at`)
      angle   the body's angle above the floor (deg); width: the feet's (or knees') outward distance
      hips    how far the torso rises above the legs' line (deg; a sag positive, a pike negative)
      gap     cm the legs fall short of straight (toes; 0.3, knees at 171 degrees, since 2026-09-30: 1.0 left them at
              163, a visible bend in every plank); y: the surface under the pivot; shift: the pelvis's move
              toward the far side (an archer push-up)
      lift    the feet's lift (on the toes 90 - angle, kneeling the tucked toes' 75, face up -60)
      feet    "flat": kneeling feet laces down (under a sofa's edge) instead of on tucked toes
      face    "up": a supine body on its heels (toes up), rising toward -f (an inverted row)
    Sets pelvis and the trunk's pitch, and each leg the pose does not set itself (a mountain climber drives one knee
    from a plank); the arms stay the pose's own. Two poses whose lines share a pivot blend the line's numbers (see
    timeline); otherwise the resolved fields blend."""
    ln = p.pop("line")
    th = math.radians(ln["angle"])
    y = ln.get("y", 0.0)
    w = ln.get("width", 10.0)
    up = ln.get("face") == "up"                     # supine: the heels at `at`, the body rising toward -f
    along = (-math.cos(th), math.sin(th), 0.0) if up else (math.cos(th), math.sin(th), 0.0)
    if ln.get("pivot", "toes") == "toes":
        lift = ln.get("lift", 90.0 - ln["angle"])
        ankle, _ = planted((ln["at"], y, 0.0), (1.0, 0.0, 0.0), lift)
        reach = BONES["thigh"] + BONES["shin"] - ln.get("gap", LINE_DEFAULTS["gap"])
        spread = abs(w) - BONES["hip_half"]                 # wide feet: the leg's length left in the side plane
        hip = add(ankle, mul(along, math.sqrt(max(0.0, reach * reach - spread * spread))))
        # the knees give toward the floor in a plank, forward when leaning back upright, up when lying face up
        pole = [0, 1, 0] if up else [1, 0, 0] if ln["angle"] > 60 else [0, -1, 0]
        legs = {"foot": [ln["at"], w], "y": y, "turn": 0, "lift": lift, "knee_out": 0, "pole": pole}
    elif ln.get("feet") == "flat":
        # the knees on the floor at `at`, the shins along the floor behind, the feet laces down (under a sofa's edge)
        knee = (ln["at"], y + KNEE_FLOOR, 0.0)
        ankle_u = y + BONES["ankle_height"] - FOOT_TOP
        ankle_f = knee[0] - math.sqrt(max(0.0, BONES["shin"] ** 2 - (knee[1] - ankle_u) ** 2))
        hip = add(knee, mul(along, BONES["thigh"]))
        legs = {"foot": [ankle_f, w], "y": y - FOOT_TOP, "turn": 180, "lift": 0, "knee_out": 0, "pole": [0, -1, 0]}
    else:
        # the knees on the floor at `at`, the shins rising behind to the feet on their tucked toes
        lift = ln.get("lift", 75.0)
        rel, _ = planted((0.0, y, 0.0), (1.0, 0.0, 0.0), lift)       # the ankle over a foot based at 0
        knee = (ln["at"], y + KNEE_FLOOR, 0.0)
        rise = rel[1] - knee[1]
        ankle_f = knee[0] - math.sqrt(max(0.0, BONES["shin"] ** 2 - rise ** 2))
        hip = add(knee, mul(along, BONES["thigh"]))
        legs = {"foot": [ankle_f - rel[0], w], "y": y, "turn": 0, "lift": lift, "knee_out": 0, "pole": [0, -1, 0]}
    p["pelvis"] = [hip[0], hip[1], ln.get("shift", 0.0)]
    tr = (list(p.get("trunk", [0, 0, 0])) + [0, 0, 0])[:3]
    tr[0] = 90.0 - (ln["angle"] + ln.get("hips", 0.0))
    if up:
        tr[0] = -tr[0]
    p["trunk"] = tr
    shared = p.pop("legs", None)
    for s in SIDES:
        p.setdefault(f"leg_{s}", copy.deepcopy(shared or legs))
    return p


# --- solving -------------------------------------------------------------------------------------------------------

ORBIT_PLANES = {"fu": ((1.0, 0.0, 0.0), (0.0, 1.0, 0.0)), "fl": ((1.0, 0.0, 0.0), (0.0, 0.0, 1.0)),
                "ul": ((0.0, 1.0, 0.0), (0.0, 0.0, 1.0))}


def ik_target(sp):
    """An ik spec's target [f, u, out]: the point itself, or its place on the spec's `orbit` circle."""
    c = [float(x) for x in sp["ik"]]
    o = sp.get("orbit")
    if not o:
        return c
    if o.get("plane", "fu") not in ORBIT_PLANES:
        raise MotionError(f"orbit plane must be one of {', '.join(ORBIT_PLANES)}")
    a, b = ORBIT_PLANES[o.get("plane", "fu")]
    th = math.radians(o["deg"])
    return [c[i] + o["r"] * (math.cos(th) * a[i] + math.sin(th) * b[i]) for i in range(3)]


def vec3(v, sign=1.0):
    return (float(v[0]), float(v[1]), float(v[2]) * sign)


def solve(p):
    """Joints (body space) and the reach shortfalls (cm) of one resolved pose."""
    p = copy.deepcopy(p)
    fit = p.pop("fit", None)
    if fit:
        return solve_fit(p, fit)
    return solve_plain(p)


def solve_fit(p, fit):
    """Move the pelvis up or down (everything else kept) until `fit["joint"]` stands at height `fit["u"]`
    (bisection; the joint rises with the pelvis, as a hand or a knee does while the feet stay planted)."""
    lo, hi = p["pelvis"][1] - 80.0, p["pelvis"][1] + 80.0
    joint, want = fit["joint"], float(fit["u"])
    for _ in range(50):
        mid = (lo + hi) / 2.0
        q = dict(p, pelvis=[p["pelvis"][0], mid, p["pelvis"][2]])
        J, short = solve_plain(q)
        if J[joint][1] > want:
            hi = mid
        else:
            lo = mid
    if abs(J[joint][1] - want) > 0.5:
        raise MotionError(f"fit: {joint} cannot reach u={want} by moving the pelvis")
    return J, short


def torso_frames(p):
    """A pose's abdomen, chest and head frames, each turned from the one below by the pose's trunk, chest and head
    angles."""
    tr, ch, hd = ((tuple(p.get(k, (0, 0, 0))) + (0, 0, 0))[:3] for k in ("trunk", "chest", "head"))
    abd = Frame().turned(*tr)
    chest = abd.turned(*ch)
    return abd, chest, chest.turned(*hd)


def solve_plain(p):
    P = vec3(p.get("pelvis", (0.0, 94.0, 0.0)))
    abd, chest, head = torso_frames(p)
    J = {"pelvis": P}
    J["waist"] = add(P, mul(abd.u, BONES["abdomen"]))
    J["neck"] = add(J["waist"], mul(chest.u, BONES["chest"]))
    J["head"] = add(J["neck"], mul(head.u, BONES["neck_head"]))
    short = {}
    for s, sp in limb_specs(p, "arm").items():
        sg = SIGN[s]
        sh = add(sub(J["neck"], mul(chest.u, BONES["shoulder_drop"])), mul(chest.l, sg * BONES["shoulder_half"]))
        J[f"shoulder_{s}"] = sh
        if "fk" in sp:
            fl, ab, el, tw, wr = (list(sp["fk"]) + [0, 0, 0, 0, 0])[:5]
            d1, front = fk_first(chest, sg, fl, ab)
            front = rig.rotate(front, d1, -sg * tw)
            elb = add(sh, mul(d1, BONES["upper_arm"]))
            d2 = bent(d1, el, front)
            wri = add(elb, mul(d2, BONES["forearm"]))
            # the forearm's own front turns with it, so a hand's bend never flips past a 90-degree elbow
            ea = math.radians(el)
            hand = bent(d2, wr, add(mul(front, math.cos(ea)), mul(d1, -math.sin(ea))))
        elif "ik" in sp:
            if "rel" in sp:                        # the target rides on the torso: an offset in the chest's frame
                tgt = add(J[sp["rel"]], chest.at(ik_target(sp), sg))
            else:
                tgt = vec3(ik_target(sp), sg)
            pole = vec3(sp.get("pole", DEFAULT_POLE_ARM), sg)
            elb, wri, short[f"arm_{s}"] = two_bone(sh, tgt, BONES["upper_arm"], BONES["forearm"], pole)
            if sp.get("hand"):
                hand = unit(chest.at(sp["hand"], sg) if "rel" in sp else vec3(sp["hand"], sg))
            else:
                hand = unit(sub(wri, elb))
        else:
            raise MotionError(f"arm_{s}: needs fk or ik")
        J[f"elbow_{s}"], J[f"wrist_{s}"] = elb, wri
        J[f"hand_tip_{s}"] = add(wri, mul(hand, BONES["hand"]))
    for s, sp in limb_specs(p, "leg").items():
        sg = SIGN[s]
        hip = add(P, mul(abd.l, sg * BONES["hip_half"]))
        J[f"hip_{s}"] = hip
        if "fk" in sp:
            fl, ab, kn, an = (list(sp["fk"]) + [0, 0, 0, 0])[:4]
            d1, front = fk_first(abd, sg, fl, ab)
            knee = add(hip, mul(d1, BONES["thigh"]))
            shin = bent(d1, kn, mul(front, -1.0))
            ankle = add(knee, mul(shin, BONES["shin"]))
            # the shin's own front turns with it, so the foot never flips past a 90-degree knee
            ka = math.radians(kn)
            foot = bent(shin, 90.0 + an, add(mul(front, math.cos(ka)), mul(d1, math.sin(ka))))
            toe = add(ankle, mul(foot, BONES["foot"]))
        elif "foot" in sp:
            t = math.radians(sp.get("turn", LEG_DEFAULTS["turn"]))
            heading = (math.cos(t), 0.0, sg * math.sin(t))
            base = (float(sp["foot"][0]), float(sp.get("y", 0.0)), sg * float(sp["foot"][1]))
            ankle, toe = planted(base, heading, float(sp.get("lift", 0.0)))
            knee, ankle2, short[f"leg_{s}"] = two_bone(hip, ankle, BONES["thigh"], BONES["shin"], vec3(sp["pole"], sg))
            if short[f"leg_{s}"] > 0:
                toe = add(toe, sub(ankle2, ankle))
                ankle = ankle2
        elif "ik" in sp:
            tgt = vec3(ik_target(sp), sg)
            pole = vec3(sp.get("pole", (1.0, 0.0, 0.1)), sg)
            knee, ankle, short[f"leg_{s}"] = two_bone(hip, tgt, BONES["thigh"], BONES["shin"], pole)
            shin = unit(sub(ankle, knee))
            toe_dir = unit(vec3(sp["toe"], sg)) if sp.get("toe") else bent(shin, 90.0, (1.0, 0.0, 0.0))
            toe = add(ankle, mul(toe_dir, BONES["foot"]))
        else:
            raise MotionError(f"leg_{s}: needs fk, foot or ik")
        J[f"knee_{s}"], J[f"ankle_{s}"], J[f"toe_{s}"] = knee, ankle, toe
    return J, short


# --- interpolation -------------------------------------------------------------------------------------------------

def ease(t, kind):
    t = max(0.0, min(1.0, t))
    if kind == "linear":
        return t
    if kind == "in":
        return t * t
    if kind == "out":
        return 1 - (1 - t) * (1 - t)
    return t * t * (3 - 2 * t)


def blend(a, b, t, path=""):
    """Two resolved poses blended field by field (numbers linearly; a limb's mode must match; the contacts of both
    poses count while one turns into the other)."""
    if path.endswith(".contacts"):
        return a if t <= 0.0 else b if t >= 1.0 else sorted(set(a) | set(b))   # in a transition either may touch
    if isinstance(a, (int, float)) and isinstance(b, (int, float)):
        return a + (b - a) * t
    if isinstance(a, list) and isinstance(b, list):
        if len(a) != len(b):
            raise MotionError(f"{path}: lists of different length")
        return [blend(x, y, t, path) for x, y in zip(a, b)]
    if isinstance(a, dict) and isinstance(b, dict):
        if set(a) != set(b):
            raise MotionError(f"{path}: different fields ({sorted(a)} vs {sorted(b)}); give both poses the same "
                              "limb mode and helper")
        return {k: blend(a[k], b[k], t, f"{path}.{k}") for k in a}
    if a == b:
        return a
    raise MotionError(f"{path}: cannot blend {a!r} and {b!r}")


def expand(p, lines=True):
    """A resolved pose with its plank helper applied (unless `lines` is false), shared limbs split per side and
    defaults filled, so two poses blend field by field."""
    p = copy.deepcopy(p)
    if "line" in p:
        ln = p["line"]
        for k, v in LINE_DEFAULTS.items():
            ln.setdefault(k, v)
        if ln["pivot"] != "toes":
            ln.setdefault("lift", 75.0)
        ln.setdefault("lift", 90.0 - ln["angle"] if ln["face"] == "down" else -60.0)
        if lines:
            apply_line(p)
    for kind in ("arm", "leg"):
        if f"{kind}s" in p:
            for s in SIDES:
                p.setdefault(f"{kind}_{s}", copy.deepcopy(p[f"{kind}s"]))
            del p[f"{kind}s"]
    for k in ("trunk", "chest", "head"):
        p[k] = (list(p.get(k, [0, 0, 0])) + [0, 0, 0])[:3]
    for s in SIDES:
        arm, leg = p.get(f"arm_{s}"), p.get(f"leg_{s}")
        if arm and "fk" in arm:
            arm["fk"] = (list(arm["fk"]) + [0, 0, 0, 0, 0])[:5]
        if leg and "fk" in leg:
            leg["fk"] = (list(leg["fk"]) + [0, 0, 0, 0])[:4]
        if leg and "foot" in leg:
            for k, v in LEG_DEFAULTS.items():
                leg.setdefault(k, v)
            if not any(leg["pole"]):
                ko = math.radians(leg["knee_out"])
                leg["pole"] = [math.cos(ko), 0.0, math.sin(ko)]
    p.setdefault("contacts", [])
    return p


def timeline(spec):
    """[(t0, t1, pose_a, pose_b, ease, phase, effort, dynamic, lined)] and the loop's length (s); `lined` holds the
    two poses with their plank helpers unapplied when both have one, so the helper's own numbers blend (the hips
    swing on an arc about the knees instead of cutting across it)."""
    tl = spec["timeline"]
    raw = {}

    def unlined(name):
        if name not in raw:
            raw[name] = expand(resolve_pose(spec, name), lines=False)
        return raw[name]
    first = tl[0]["pose"]
    cur, cur_raw = expand(resolve_pose(spec, first)), unlined(first)
    segs, t = [], 0.0
    for st in tl[1:]:
        if "to" in st:
            nxt, nxt_raw = expand(resolve_pose(spec, st["to"])), unlined(st["to"])
            both = None
            limbs = lambda p: {k for k in p if k[:4] in ("arm_", "leg_")}
            if "line" in cur_raw and "line" in nxt_raw and cur_raw["line"]["pivot"] == nxt_raw["line"]["pivot"] \
                    and limbs(cur_raw) == limbs(nxt_raw):     # a leg one pose sets itself blends resolved
                both = (cur_raw, nxt_raw)
            segs.append((t, t + st["s"], cur, nxt, st.get("ease", "inout"), st.get("phase"), st.get("effort", 0.5),
                         bool(st.get("dynamic")), both))
            cur, cur_raw, t = nxt, nxt_raw, t + st["s"]
        else:
            segs.append((t, t + st["hold"], cur, cur, "linear", st.get("phase"), st.get("effort", 0.3), False, None))
            t += st["hold"]
    return segs, t


def limb_mode(sp):
    """fk, ik, rel (an ik target riding on the torso) or foot: poses blend a limb's fields only within one mode."""
    mode = next(k for k in ("fk", "ik", "foot") if k in sp)
    return "rel" if mode == "ik" and "rel" in sp else mode


CHAINS = {"arm": ("shoulder", "elbow", "wrist", "hand_tip"), "leg": ("hip", "knee", "ankle", "toe")}
STRAIGHT = 3.0         # deg: a knee or an elbow bent less than this has no hinge of its own (HINGE gives it one)
HINGE = {"leg": -1.0, "arm": 1.0}   # a straight joint's hinge along the torso's l: knees bend back, elbows forward


def slerp_dir(a, b, t):
    """The unit direction t of the way from a to b along the great circle."""
    a, b = unit(a), unit(b)
    om = math.acos(max(-1.0, min(1.0, rig.dot(a, b))))
    if om < 1e-6:
        return a
    if om > math.pi - 1e-4:                    # opposite: no great circle is shorter, turn about any square axis
        return rig.rotate(a, unit(rig.cross(a, rig.L0), rig.F0), 180.0 * t)
    k = math.sin(om)
    return unit(add(mul(a, math.sin((1 - t) * om) / k), mul(b, math.sin(t * om) / k)))


def torso_frame(p, kind):
    """The frame a limb hangs from: the abdomen's for a leg, the chest's for an arm."""
    abd, chest, _ = torso_frames(p)
    return chest if kind == "arm" else abd


def local(v, F):
    """A body-space direction in frame F's axes (f, u, l)."""
    return (rig.dot(v, F.f), rig.dot(v, F.u), rig.dot(v, F.l))


def square_unit(v, axis, fallback):
    """v made square to the unit axis, as a unit vector; `fallback` (made square too) where v runs along the axis."""
    p = rig.square_to(v, axis)
    if rig.norm(p) > 0.2:
        return unit(p)
    return unit(rig.square_to(fallback, axis), unit(rig.cross(axis, rig.L0), rig.F0))


def end_frame(d2, h1, natural):
    """The frame the hand or the foot bends in: along the forearm or the shin, its hinge the elbow's or the knee's."""
    e2 = square_unit(h1, d2, natural)
    return d2, e2, rig.cross(d2, e2)


def chain_angles(J, side, kind, F, hinge=None):
    """A limb as angles about its own hinge: the first bone's direction and the middle joint's hinge in the torso
    frame F, the middle joint's bend, and the hand's or the foot's direction in its forearm's or shin's frame
    (end_frame). A middle joint too straight to have a hinge takes `hinge` (another pose's, in F's axes), else the one
    it bends on (HINGE)."""
    root, mid, end, tip = (J[f"{j}_{side}"] for j in CHAINS[kind])
    d1, d2, d3 = unit(sub(mid, root)), unit(sub(end, mid)), unit(sub(tip, end))
    bend1 = rig.angle(d1, d2)
    natural = mul(F.l, HINGE[kind])
    h1 = unit(rig.cross(d1, d2)) if bend1 > STRAIGHT else square_unit(natural, d1, F.f)
    if bend1 <= STRAIGHT and hinge:
        h1 = square_unit(F.at(hinge), d1, h1)
    return {"d1": local(d1, F), "h1": local(h1, F), "bend1": bend1,
            "d3": tuple(rig.dot(d3, e) for e in end_frame(d2, h1, natural)),
            "len": (rig.dist(root, mid), rig.dist(mid, end), rig.dist(end, tip))}


def blend_chain(J, side, kind, A, B, F, f):
    """Rebuild a limb from its root in J, its angles blended f of the way from A to B (chain_angles): the first bone
    and the hinge slerped in the blended torso frame F, the bend blended as an angle and the hand or the foot slerped
    in its forearm's or shin's frame, so a knee or an ankle never passes through a bend or a turn neither pose has."""
    j = [f"{n}_{side}" for n in CHAINS[kind]]
    natural = mul(F.l, HINGE[kind])
    d1 = F.at(slerp_dir(A["d1"], B["d1"], f))
    h1 = square_unit(F.at(slerp_dir(A["h1"], B["h1"], f)), d1, natural)
    d2 = rig.rotate(d1, h1, A["bend1"] + (B["bend1"] - A["bend1"]) * f)
    a, b, c = slerp_dir(A["d3"], B["d3"], f)
    e1, e2, e3 = end_frame(d2, h1, natural)
    d3 = add(add(mul(e1, a), mul(e2, b)), mul(e3, c))
    for k, d in enumerate((d1, d2, d3)):
        J[j[k + 1]] = add(J[j[k]], mul(d, A["len"][k]))


def pose_at(segs, t):
    """The pose at time t: the step's two poses blended field by field; a limb whose mode changes between them (a
    planted leg that becomes an FK swing) is marked, and `solve_at` blends its angles instead (blend_chain)."""
    for s in segs:
        if s[0] <= t < s[1] or s is segs[-1]:
            f = ease((t - s[0]) / (s[1] - s[0]) if s[1] > s[0] else 0.0, s[4])
            a, b = s[8] or (s[2], s[3])
            mixed = [k for k in a if k[:4] in ("arm_", "leg_") and limb_mode(a[k]) != limb_mode(b[k])]
            pose = blend(a, dict(b, **{k: a[k] for k in mixed}), f)
            if s[8]:
                apply_line(pose)
            if mixed:
                pose["_mixed"] = (mixed, s[2], s[3], f)
            return pose, s
    raise AssertionError("empty timeline")


def solve_at(pose):
    """solve() for a pose from pose_at: each mixed limb rebuilt from its root with its angles blended between the
    step's two solved poses (blend_chain), so its bones keep their lengths, a knee swings on its hinge and a foot or a
    hand bends and turns only within what the two poses hold."""
    mixed = pose.pop("_mixed", None)
    J, short = solve(pose)
    if mixed:
        names, a, b, f = mixed
        Ja, Jb = solve(a)[0], solve(b)[0]
        for k in names:
            kind, side = k.split("_")
            Fa, Fb = torso_frame(a, kind), torso_frame(b, kind)
            # a straight end bends on the other end's hinge (the first's when both are straight), so the joint
            # bends from straight in the plane it ends in and the hand or the foot keeps one frame
            A = chain_angles(Ja, side, kind, Fa)
            B = chain_angles(Jb, side, kind, Fb, A["h1"])
            if A["bend1"] <= STRAIGHT < B["bend1"]:
                A = chain_angles(Ja, side, kind, Fa, B["h1"])
            blend_chain(J, side, kind, A, B, torso_frame(pose, kind), f)
            short[k] = 0.0
    return J, short


# --- the track -----------------------------------------------------------------------------------------------------

def rounded(v):
    """Numbers in nested dicts, lists and tuples rounded to 0.001 cm (tracks stay small and diff cleanly)."""
    if isinstance(v, float):
        return round(v, 3)
    if isinstance(v, (list, tuple)):
        return [rounded(x) for x in v]
    if isinstance(v, dict):
        return {k: rounded(x) for k, x in v.items()}
    return v


def head_lateral(pose, view):
    """The head's lateral axis (toward the far side) in the renderer's coordinates, from the pose's trunk, chest and
    head angles: the helm turns with a head that turns on its own (a gaze over the hand)."""
    f, u, l = torso_frames(pose)[2].l
    return [round(-l, 4), round(u, 4), round(f, 4)] if view == "front" else [round(f, 4), round(u, 4), round(l, 4)]


def to_view(J, view):
    """Body-space joints in the renderer's coordinates: side view (x forward, y up, z the limb's lateral offset from
    its own plane, the renderer adding the plane); front view (x lateral with the near side positive, y up, z
    forward)."""
    out = {}
    for k, v in J.items():
        f, u, l = v
        if view == "front":
            out[k] = [round(-l, 3), round(u, 3), round(f, 3)]
            continue
        base = 0.0
        for s in SIDES:
            if k.endswith("_" + s):
                limb = k[: -len(s) - 1]
                if limb in ("shoulder", "elbow", "wrist", "hand_tip"):
                    base = SIGN[s] * ARM_DEPTH
                elif limb in ("hip", "knee", "ankle", "toe"):
                    base = SIGN[s] * LEG_DEPTH
        out[k] = [round(f, 3), round(u, 3), round(l - base, 3)]
    return out


def sash_knot(J):
    """The sash's knot: on the belt at the near hip, outside the tasset (body space)."""
    P, W = J["pelvis"], J["waist"]
    up = unit(sub(W, P))
    lat = unit(sub(J["hip_near"], P), (0.0, 0.0, -1.0))
    fwd = unit(rig.cross(up, lat))
    return add(add(add(P, mul(up, 12.8)), mul(lat, 24.0)), mul(fwd, -1.0))


def sash_reach(knots):
    """The box (lo, hi per axis) the sash can reach from these knots."""
    reach = SASH_LINK * (SASH_POINTS - 1) + SASH_R
    return [min(k[i] for k in knots) - reach for i in range(3)], [max(k[i] for k in knots) + reach for i in range(3)]


def sash_boxes(props_list, at, reach):
    """The floor and the props' solids at `at` (0-1 of the loop) inside the sash's reach (sash_reach)."""
    lo, hi = reach
    boxes = [b for p in props_list for b in props.solids(p, at)]
    return [props.FLOOR] + [b for b in boxes if all(b[2 * i] <= hi[i] and b[2 * i + 1] >= lo[i] for i in range(3))]


def sash_out(p, came, boxes):
    """A sash point kept SASH_R off every box, pushed back out through the face it came in by (`came`, where it was
    before this step): one that falls across a top lands on it, however thin (cloth stays on a bed, a bench or a mat),
    one beside a box stays beside it, and only a box standing clear of the floor has an underside to hang below; one
    that was inside already leaves through the top from the box's upper half, else through its nearest face. A push
    out of one box can land in another, so the boxes are gone through again until none moves it (three passes)."""
    r = SASH_R
    cf, cu, cl = came
    for _ in range(3):
        start = p
        for f0, f1, u0, u1, l0, l1 in boxes:
            f, u, l = p
            top = u1 + r
            under = u0 > 2 * r                             # clear of the floor: an underside to hang below
            if cu >= top - 0.01 and u < top:               # fell to or through the top this step
                t = (cu - top) / (cu - u) if cu > u else 0.0
                if f0 - r < cf + (f - cf) * t < f1 + r and l0 - r < cl + (l - cl) * t < l1 + r:
                    p = (f, top, l)
                    continue
            if not (f0 - r < f < f1 + r and u0 - r < u < top and l0 - r < l < l1 + r):
                continue
            faces = ((cf <= f0 - r + 0.01, (f0 - r, u, l)), (cf >= f1 + r - 0.01, (f1 + r, u, l)),
                     (cl <= l0 - r + 0.01, (f, u, l0 - r)), (cl >= l1 + r - 0.01, (f, u, l1 + r)),
                     (cu <= u0 - r + 0.01 and under, (f, u0 - r, l)))
            p = next((q for through, q in faces if through), None)
            if p is None:
                p = (f, top, l) if u >= (u0 + u1) / 2.0 else min(
                    [(f - f0 + r, (f0 - r, u, l)), (f1 + r - f, (f1 + r, u, l)), (l - l0 + r, (f, u, l0 - r)),
                     (l1 + r - l, (f, u, l1 + r))] + ([(u - u0 + r, (f, u0 - r, l))] if under else []))[1]
        if p == start:
            break
    return p


def sash_leg(J):
    """This frame's near leg as the cones the sash keeps off (SASH_LEG): (root, bone, its length squared, the radius
    at each end with SASH_R)."""
    out = []
    for a, b, ra, rb in SASH_LEG:
        A = J[a + "_near"]
        ab = sub(J[b + "_near"], A)
        out.append((A, ab, rig.dot(ab, ab) or 1e-6, ra + SASH_R, rb + SASH_R))
    return out


def sash_off_leg(p, cones):
    """A sash point pushed out of the near leg's plates (sash_leg), square to the bone: the cloth falls beside a
    raised or splayed thigh, never through it."""
    for A, ab, ab2, ra, rb in cones:
        t = max(0.0, min(1.0, rig.dot(sub(p, A), ab) / ab2))
        c = add(A, mul(ab, t))
        d = sub(p, c)
        dn, r = rig.norm(d), ra + (rb - ra) * t
        if 1e-6 < dn < r:
            p = add(c, mul(d, r / dn))
    return p


def sash_top(p, boxes):
    """The top a sash point lies on or over: the highest under it, out to SASH_R past a box's edges, where sash_out
    lands cloth on it."""
    return props.top_in(p, boxes, SASH_R, SASH_R)


def sash_rest(p, boxes):
    """How much a sash point lies on the surface under it: 1 on it, 0 from SASH_REST above it (the ribbon lies flat
    where it rests and faces the camera where it hangs, blender/armor_model.py place_sash)."""
    return round(max(0.0, min(1.0, 1.0 - (p[1] - sash_top(p, boxes) - SASH_R) / SASH_REST)), 3)


def simulate_sash(frames_J, props_list=(), at=None):
    """A five-point chain hanging from the knot (Verlet, gravity, damping), run over three loops so it settles; the
    last loop is kept, each frame's points with how much each rests on a surface (sash_rest). It starts trailing
    behind the knot (SASH_TRAIL), so cloth that reaches a surface lies along it instead of folding into a post; points
    stay off the floor and the props (sash_out; a moving prop where it stands at each frame, or at `at` for a still),
    slide on a surface only as SASH_GRIP lets them, and stay outside the near leg and off its plates (sash_off_leg)."""
    n = len(frames_J)
    dt = 1.0 / HZ
    knots = [sash_knot(J) for J in frames_J]
    moving = at is None and any(p.get("scroll") or p.get("belt") for p in props_list)
    reach = sash_reach(knots)
    still = None if moving else sash_boxes(props_list, at or 0.0, reach)
    trail, boxes = mul(unit(SASH_TRAIL), SASH_LINK), still or sash_boxes(props_list, 0.0, reach)
    pts = [knots[0]]
    for _ in range(1, SASH_POINTS):                # laid link by link, each resolved from the one before it
        pts.append(sash_out(add(pts[-1], trail), pts[-1], boxes))
    prev = list(pts)
    out, rests = [], []
    for loop in range(3):
        for i in range(n):
            J = frames_J[i]
            knot = knots[i]
            boxes = still or sash_boxes(props_list, i / n, reach)
            cones = sash_leg(J)
            edge = J["hip_near"][2] - 10.0             # outside the near tasset
            for _ in range(4):                     # substeps
                h = dt / 4
                for k in range(1, SASH_POINTS):
                    v = mul(sub(pts[k], prev[k]), 0.93)
                    prev[k] = pts[k]
                    pts[k] = add(add(pts[k], v), (0.0, -981.0 * h * h, 0.0))
                pts[0] = prev[0] = knot
                for it in range(6):
                    for k in range(1, SASH_POINTS):
                        a, b = pts[k - 1], pts[k]
                        d = sub(b, a)
                        dn = rig.norm(d) or 1e-6
                        corr = mul(d, (dn - SASH_LINK) / dn)
                        if k == 1:
                            pts[k] = sub(b, corr)
                        else:
                            pts[k - 1] = add(a, mul(corr, 0.5))
                            pts[k] = sub(b, mul(corr, 0.5))
                    pts[0] = knot
                    if it % 2:                             # the solids every other pass, the last pass among them
                        for k in range(1, SASH_POINTS):
                            x, y, z = pts[k]
                            pts[k] = sash_out(sash_off_leg((x, y, min(z, edge)), cones), prev[k], boxes)
                slide = 1.0 - SASH_GRIP
                for k in range(1, SASH_POINTS):        # a point on a surface keeps a share of its sideways speed
                    p, q = pts[k], prev[k]
                    if p[1] - sash_top(p, boxes) <= SASH_R + 0.05:
                        prev[k] = (p[0] - (p[0] - q[0]) * slide, q[1], p[2] - (p[2] - q[2]) * slide)
            if loop == 2:
                out.append([tuple(round(c, 3) for c in p) for p in pts])
                rests.append([sash_rest(p, boxes) for p in pts])
    return out, rests


def glow_curve(efforts):
    """The runes' light per frame: 0.25 at rest to 1.0 at full effort, smoothed over 0.15 s (circular)."""
    n = len(efforts)
    w = max(1, int(0.15 * HZ))
    out = []
    for i in range(n):
        s = sum(efforts[(i + k) % n] for k in range(-w, w + 1)) / (2 * w + 1)
        out.append(round(0.25 + 0.75 * s, 3))
    return out


def blender_xyz(v, view):
    """A body-space point (cm) in Blender's axes (cm): the side view looks from the near side (X forward, Y = l, Z up);
    the front view from behind (X = -l, the near side to the right, Y = f, Z up)."""
    f, u, l = v
    return (f, l, u) if view == "side" else (-l, f, u)


def item_points(spec_items, frames_items):
    """Points (body space) of the held items the camera must keep: a band's fixed anchors, the landmine's pivot, a
    door towel's knot (or where it comes over a door's top) and loose end, the ring straps from 25 cm above their
    rings, a held chair's corners, a standard's butt, finial, crossbar end and cloth (items.banner_points)."""
    pts = []
    for it in spec_items:
        if it["kind"] == "band":
            pts += [tuple(float(x) for x in end) for pair in it["strands"] for end in pair if isinstance(end, list)]
        elif it["kind"] == "landmine":
            pts.append(tuple(float(x) for x in it["pivot"]))
        elif it["kind"] == "doortowel":
            pts += [tuple(float(x) for x in it[key]) for key in ("knot", "top", "loose") if key in it]
    for placed in frames_items:
        for pose in placed:
            if pose["kind"] == "chair":
                pts += [p for c, half in pose["boxes"] for p in items.box_corners(c, pose["axes"], half)]
            if pose["kind"] == "straps":
                for ln in pose["lines"]:
                    a, r = ln["anchor"], ln["ring"]
                    d = rig.dist(a, r)
                    pts.append(rig.lerp(r, a, min(1.0, 25.0 / d)) if d > 1e-6 else r)
            if pose["kind"] == "banner":
                pts += items.banner_points(pose)
    return pts


def screen_axes(yaw, pitch):
    """The screen's right and up (Blender axes) of the camera at `yaw` and `pitch` degrees (apply_camera's)."""
    y, p = math.radians(yaw), math.radians(pitch)
    back = (math.sin(y) * math.cos(p), -math.cos(y) * math.cos(p), math.sin(p))
    right = unit(rig.cross(mul(back, -1.0), (0.0, 0.0, 1.0)), (1.0, 0.0, 0.0))
    return right, rig.cross(right, mul(back, -1.0))


def fit_camera(body_frames, view, props_list, yaw, pitch, fill=0.86, margin=22.0, crest=32.0, extra=()):
    """The orthographic camera (`blender/armor_model.py` apply_camera) that holds every frame: the joints projected
    with the armour's margins (the crest above the head), the props' frame points and `extra` points (the held
    items' fixed points)."""
    right, up = screen_axes(yaw, pitch)
    xs, ys = [], []
    for J in body_frames:
        for k, v in J.items():
            b = blender_xyz(v, view)
            sx, sy = rig.dot(b, right), rig.dot(b, up)
            m = crest if k == "head" else margin
            xs += [sx - m, sx + m]
            ys += [sy - margin, sy + m]
    for v in [v for pr in props_list for v in props.frame_points(pr)] + list(extra):
        b = blender_xyz(v, view)
        xs.append(rig.dot(b, right))
        ys.append(rig.dot(b, up))
    x0, x1, y0, y1 = min(xs), max(xs), min(ys), max(ys)
    cx, cy = (x0 + x1) / 2, (y0 + y1) / 2
    # the aim point (X, 0, Z) whose projection is the box's centre
    a, b_, c, d = right[0], right[2], up[0], up[2]
    det = a * d - b_ * c
    X = (cx * d - b_ * cy) / det
    Z = (a * cy - c * cx) / det
    size = max(x1 - x0, y1 - y0) / fill
    return {"yaw_deg": yaw, "pitch_deg": pitch, "center_x": round(X, 2), "center_y": round(Z, 2),
            "ortho_scale_m": round(size / 100.0, 4), "aspect": 1.0, "fill": fill,
            "bbox_units": [round(x1 - x0, 1), round(y1 - y0, 1)]}


def prop_box(track, px):
    """The screen box [x0, y0, x1, y1] (render px, y down, in the `px` square the camera renders) of the frame points
    the camera holds of the track's props (fit_camera), or None: the crop grows toward it as far as the cell has room
    (pack.geometry), so a pole a flag's body lies across shows down to its foot."""
    cam = track["camera"]
    right, up = screen_axes(cam["yaw_deg"], cam["pitch_deg"])
    aim = (cam["center_x"], 0.0, cam["center_y"])
    ax, ay = rig.dot(aim, right), rig.dot(aim, up)
    size = cam["ortho_scale_m"] * 100.0
    xs, ys = [], []
    for v in [v for pr in track["props"] for v in props.frame_points(pr)]:
        b = blender_xyz(v, track["view"])
        xs.append(px / 2 + (rig.dot(b, right) - ax) / size * px)
        ys.append(px / 2 - (rig.dot(b, up) - ay) / size * px)
    return [min(xs), min(ys), max(xs), max(ys)] if xs else None


def bake(spec):
    """The 30 Hz track of a motion file (see the module docstring), with its body-space joints for the checks and,
    for a move with a fault, its fault still (`_fault`, fault_still)."""
    if spec.get("variant_of"):
        raise MotionError(f"{spec['move']}: a variant is baked from its reference (variants.py)")
    segs, loop = timeline(spec)
    n = max(1, round(loop * HZ))
    view = spec.get("view", "side")
    body, frames, efforts, shorts, heads = [], [], [], [], []
    for i in range(n):
        t = i / HZ
        pose, seg = pose_at(segs, t)
        heads.append(head_lateral(pose, view))
        J, short = solve_at(pose)
        body.append(J)
        shorts.append(short)
        efforts.append(seg[6])
        touch = sorted(set(seg[2]["contacts"]) & set(seg[3]["contacts"]))    # held through the whole step
        frames.append({"t": round(t, 4), "phase": seg[5], "contacts": pose.get("contacts", []), "touch": touch})
        if seg[7]:
            frames[-1]["dynamic"] = True
    sash, rests = simulate_sash(body, spec.get("props", []))
    glow = glow_curve(efforts)
    view_joints = [to_view(J, view) for J in body]
    sash_view = [[to_view({"s": p}, view)["s"] for p in pts] for pts in sash]
    held = spec.get("items", [])
    for i, fr in enumerate(frames):
        fr["glow"] = glow[i]
        fr["joints"] = view_joints[i]
        fr["head_l"] = heads[i]
        fr["sash"] = sash_view[i]
        fr["sash_rest"] = rests[i]
        fr["items"] = [rounded(items.place(it, body[i], fr["t"], n / HZ)) for it in held]   # the loop as played
        fr["held_arms"], fr["planted_arms"] = arm_holds(fr["contacts"], held, fr["t"], n / HZ)
    for pr in spec.get("props", []):
        props.parts(pr)                            # an unknown kind fails here, before any render
    track = {
        "move": spec["move"], "view": view, "camera": camera_of(spec, body, frames),
        "props": spec.get("props", []), "fps": HZ,
        "loop_s": round(loop, 4), "frames": frames, "bones": BONES,
        "units": "rig units = cm; Blender X = x*0.01, Z = y*0.01, Y = (side base + z)*0.01; sash z absolute",
        "working": spec.get("working", []), "items": spec.get("items", []), "key": key_step(spec, segs, n)[0],
        "key_phase": key_step(spec, segs, n)[1],
        "mirror": bool(spec.get("mirror")),
        "eyes": spec.get("eyes", "open"),
        "_body": body, "_short": shorts,
    }
    if spec.get("fault"):
        track["_fault"] = fault_still(spec, track)
    return track


def camera_of(spec, body, frames):
    """The camera that holds these poses of a move (fit_camera), with its props and its held items' fixed points."""
    cam = spec.get("camera", {})
    return fit_camera(body, spec.get("view", "side"), spec.get("props", []), cam.get("yaw", 30.0),
                      cam.get("pitch", 6.0), extra=item_points(spec.get("items", []), [fr["items"] for fr in frames]))


def fault_still(spec, track):
    """The fault pose as a still standing where the key pose stands in the loop: its frame (the key frame's time,
    phase and glow, so the pair differs in the pose alone; the sash hanging at rest; the items placed at that moment
    of the loop), its body-space joints and its reach shortfalls. Kept on the track under `_fault`: outside the
    frames, so the loop and its renders are the same with or without a fault."""
    pose = expand(resolve_pose(spec, spec["fault"]["pose"]))
    J, short = solve(pose)
    view, key = track["view"], track["frames"][track["key"]]
    sash, rests = simulate_sash([J] * HZ, track["props"], at=track["key"] / len(track["frames"]))
    frame = {"t": key["t"], "phase": key["phase"], "contacts": pose["contacts"], "touch": pose["contacts"],
             "glow": key["glow"], "joints": to_view(J, view), "head_l": head_lateral(pose, view),
             "sash": [to_view({"s": p}, view)["s"] for p in sash[-1]], "sash_rest": rests[-1],
             "items": [rounded(items.place(it, J, key["t"], len(track["frames"]) / HZ)) for it in track["items"]]}
    frame["held_arms"], frame["planted_arms"] = arm_holds(pose["contacts"], track["items"], key["t"],
                                                          len(track["frames"]) / HZ)
    return {"frame": frame, "body": J, "short": short}


def arm_holds(contacts, specs, t, loop):
    """The track's `held_arms` and `planted_arms` at t: the sides whose hand holds an item (items.holders), which the
    renderer leaves where the pose put them, since turning one clear of its tasset takes the hand off the item; and
    the other sides whose elbow, forearm, wrist or fingertips are a contact, which it turns clear only as far as the
    hand stays on its surface in the picture (armor_model.arm_turns)."""
    held = sorted({s for it in specs for s in items.holders(it, t, loop)})
    planted = [s for s in SIDES if s not in held
               and any(f"{j}_{s}" in contacts for j in ("elbow", "forearm", "wrist", "hand_tip"))]
    return held, planted


def fault_track(track):
    """The track the fault still renders from and is checked on: the loop's own with the fault pose in the key
    frame's place, so the camera, the floor pool, the props (a scrolling one where the key pose has it) and the
    items' moment are the loop's, and frame `key` is the still. The body-space joints and shortfalls ride along for
    the checks."""
    k, fault = track["key"], track["_fault"]

    def swapped(seq, one):
        return seq[:k] + [one] + seq[k + 1:]
    return dict({name: v for name, v in track.items() if name != "_fault"},
                frames=swapped(track["frames"], fault["frame"]), _body=swapped(track["_body"], fault["body"]),
                _short=swapped(track["_short"], fault["short"]))


def key_step(spec, segs, n):
    """(frame, phase) of the key pose: the time given, or the moment the timeline first arrives at the named pose (the
    start pose's is frame 0), with the phase of the step that arrives there (the New-move view's caption)."""
    key = spec.get("key")
    if isinstance(key, (int, float)):
        f = min(n - 1, round(key * HZ))
        return f, pose_at(segs, f / HZ)[1][5]
    if key is None or key == spec["timeline"][0]["pose"]:
        return 0, segs[0][5]
    want = expand(resolve_pose(spec, key))
    for s in segs:
        if s[3] == want:
            return min(n - 1, round((s[0] if s[2] is s[3] else s[1]) * HZ)), s[5]
    raise MotionError(f"{spec['move']}: the key pose {key} never appears in the timeline")


def key_frame(spec, segs, n):
    return key_step(spec, segs, n)[0]


def dumps(value, indent=0, width=118, lead=0):
    """A motion file's JSON as people write it: objects one key per line, anything that fits on one line (a list of
    numbers, a limb spec, a timeline step) kept on one line. `lead` is what precedes the value on its line."""
    one = json.dumps(value, separators=(", ", ": "))
    if indent + lead + len(one) <= width or not isinstance(value, (dict, list)):
        return one
    pad = " " * (indent + 2)
    if isinstance(value, list):
        body = [pad + dumps(v, indent + 2, width) for v in value]
    else:
        body = [pad + json.dumps(k) + ": " + dumps(v, indent + 2, width, len(json.dumps(k)) + 2)
                for k, v in value.items()]
    o, c = "[]" if isinstance(value, list) else "{}"
    return o + "\n" + ",\n".join(body) + "\n" + " " * indent + c
