"""The checks every baked motion passes before it is rendered (pure Python; docs/plan.md slice 10).

Each check reads the body-space joints `motion.bake` keeps (`_body`, cm, f forward, u up, l lateral) and returns
problems as strings; `run` gathers them. An error keeps the move from being rendered (`make figures` builds the
others and names it at the end); the review page still shows the frame each one names, so a wrong pose is found by
looking, not only by number.

  reach      a limb whose hand or foot is a contact falls short of its target (more than 0.5 cm)
  solids     a joint's armour enters the floor or a prop (props.py; a declared contact may touch, never sink), so
             does the helm past the head's ball while the head is no contact (its points as helm.py measured them),
             or the sash's cloth past its knot
  touch      a contact held through a step floats more than 2.5 cm off every surface
  items      a held implement enters the floor, the armour or the helm (the holding hands and forearms aside; its solid
             is items.shape's, from the measures its placement uses: a sphere, a box such as a backpack in hand, worn or
             on the hips, or a vest's plate, or a line), a skipping rope, a band or a door towel's tail cuts through the
             armour or the helm (its distance grid as helm.py measured it) past what holds it, a skipping rope's middle
             passes straight down with a sole on or under it, an ab wheel leaves the floor, a ring strap or a door
             towel's held tail stretches or slackens, a held chair enters a prop, misses the hands on its seat's sides,
             or (set down at the bottom of a lift) never reaches the floor, a standard's pole or cloth enters the armour
             (the holding hands and forearms aside)
  bend       a knee bends backwards, or a knee or elbow folds past 165 degrees
  range      an ankle folds the foot toward its shin past ANKLE_MAX, a foot turns about its bent shin more than
             TWIST_MAX away from its knee, or a wrist bends past WRIST_MAX
  pop        a joint jumps: its travel changes by more than POP_CM in one tick (the loop's seam too)
  balance    the centre of mass leaves the support (the soles and the other contacts, 3 cm margin) while every
             contact rests on the floor or a prop's top: front to back, and side to side in the front view; weight on
             ring straps is a support too, as a bar is
  limbs      a thigh, shin, upper arm, forearm or the spine passes through a prop between its joints (a bone whose end
             is a declared contact may rest on it)
  tempo      a controlled rep's moving phase outside 0.8-4 s, any step under 0.1 s, a loop outside 0.5-12 s (a flow's
             up to 20 s: one round of a sun salutation) or one that does not end where it
             starts, a move contract/moves.json lacks; an extra (motion.py, `extra`) has no dose, and its name is no
             library move's
  entry      a hold that starts at its key pose (less than ENTRY_CM from it at every joint) or a pose (a flow) whose loop
             never leaves its key pose by that much: the figure shows the way into its position
  targets    the move's own targets (`"targets"` in the motion file): a joint's height, a joint angle (in the side
             view's plane for a side-view move) or a joint's offset from another along a body axis, at a pose
  fault      the fault pose of a move that has one (`"fault"`: the wrong picture of the New-move view's pair), a still
             in the key frame's place: it passes reach, solids, limbs, touch, items, bend and range as every frame does
             (never balance or tempo: a fault may be a body falling out of its line), the target it `breaks` holds
             at the key pose and fails at the fault pose, and the pose stays inside the loop's camera frame
  show       a prop's faces fall below SHOW_MIN against the card at their darkest, a part narrower than THIN_CM
             across (a rod, a post, a leg, a strap, a slat) below THIN_MIN
  gear       a gear variant's gear (`<move>.<gear>`, motion.py) is no equipment key of the game's (ns.Gear)

A gear variant passes every check as a move of its own: its implement against the floor, the props and the armour, its
poses (the holding arms its own) against reach, solids and the rest, the move's tempo, targets and fault.
"""
import hashlib
import json
import math
import os
import sys

import rig
from rig import BONES, CLEARANCE, SIDES, sub, dot, angle, unit
import motion
import items
import props

sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))   # tools/, for tokens.py
from tokens import luminance  # noqa: E402  (the WCAG 2 luminance of tools/tokens.py's contrast pairs)

REACH_TOL = 0.5
SINK_TOL = 1.5
TOUCH_TOL = 0.5
# The helm past the head's ball, tested while the head is no contact (2026-10-22, the animation ratings' "helmets
# press into walls"): helm.py's points on the real helm, (cm up the head's axis from the head joint, cm forward, cm to
# one side), mirrored to the other side, and its distance grid over one side (HELM_GRID: its first point, its spacing,
# its size in points, its distances in cm, c fastest); HELM_REACH is the farthest point, past which a box is too far to
# test them.
HELM_FILE = os.path.join(os.path.dirname(os.path.abspath(__file__)), "helm.json")
with open(HELM_FILE) as _fh:
    _helm = json.load(_fh)
HELM = [(a, b, s * c) for a, b, c in _helm["points"] for s in ((1.0, -1.0) if c else (1.0,))]
HELM_ARMOUR = _helm["armour"]
_grid = _helm["grid"]
HELM_GRID = (_grid["origin"], _grid["step"], _grid["size"], [mm / 10.0 for row in _grid["mm"] for mm in row])
HELM_PAD = _grid["pad"]
HELM_REACH = max(math.sqrt(a * a + b * b + c * c) for a, b, c in HELM) + TOUCH_TOL
BALANCE_MARGIN = 3.0
MAX_BEND = 165.0
# The joints' ranges as the armour draws them (2026-10-22, the animation ratings' "foot folds onto the shin", "foot
# turns 174 deg against its knee", "wrist folds 156 deg"). Ankle: the shin-to-foot angle, 90 standing, 0 pointed in
# line with the shin, 139 a deep squat with the heels down (deep_squat); past ANKLE_MAX the foot folds onto the shin.
# Twist: the foot's heading about the shin against the side the bent knee stands out to; the crow and frog stands'
# turned-out hips reach 111. Wrist: the forearm-to-hand angle, 90 a palm flat under an upright forearm.
ANKLE_MAX = 145.0
TWIST_MAX = 120.0
TWIST_KNEE = 25.0        # a knee bent less than this has no side to judge the foot's turn against
WRIST_MAX = 120.0
# A pop: a joint's travel in one tick changes by more than this (cm, the second difference at 30 Hz). A jog's or a
# sprint's arm swing reaches 22; the ratings' flipping elbows, snapping seams and crouches that teleport into a jump
# reached 29 to 89.
POP_CM = 25.0
FOOT_TOP = motion.FOOT_TOP
PHASE_S = (0.8, 4.0)
STEP_MIN = 0.1
LOOP_S = (0.5, 12.0)
LOOP_FLOW_S = 20.0
# A hold's loop starts where the player sets up and shows the way into the position and out (David, 2026-10-26: "the
# tuck l sit animation is not animated. Generally it should always show how to get into that position"): its first
# frame stands this far (cm, at its farthest joint) from the key pose, and a pose's loop leaves its key pose by as
# much. The entries of the holds that showed one then started 13 to 235 cm away; the holds without one, 0.
ENTRY_CM = 12.0
ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
_DOSES = None
_GEAR = None
STYLE = os.path.join(os.path.dirname(os.path.abspath(__file__)), "style.json")
# The phones draw the figure lighten over a page of nearly the card's colour, so a prop's face this close to the card
# vanishes there (WCAG 2 contrast; the wall push-up's wall at 1.05:1 did, 2026-10-05). A part narrower than THIN_CM
# across (a rod, a bar's post, a leg, a strap, a slat) is a few px wide in the 192 px box, where its edge pixels blend
# with the card, so it clears THIN_MIN (the flag's pole at 1.65:1 at its darkest lost 18 % of its pixels under 1.6:1
# at 192 px, at 2:1 2 %; 2026-10-05).
SHOW_MIN = 1.6
THIN_MIN = 2.0
THIN_CM = 10.0

# Segment masses (fractions of the body, after Dempster) placed at their segments' midpoints.
MASS = (
    ("head", ("neck", "head"), 0.081),
    ("trunk", ("pelvis", "neck"), 0.497),
    ("upper_arm", ("shoulder", "elbow"), 0.028),
    ("forearm", ("elbow", "hand_tip"), 0.022),
    ("thigh", ("hip", "knee"), 0.100),
    ("shin", ("knee", "ankle"), 0.0465),
    ("foot", ("ankle", "toe"), 0.0145),
)


def clearance(joint):
    for side in SIDES:
        if joint.endswith("_" + side):
            return CLEARANCE[joint[: -len(side) - 1]]
    return CLEARANCE[joint]


def com(J):
    tot, acc = 0.0, [0.0, 0.0, 0.0]
    for _, (a, b), m in MASS:
        pairs = [(a, b)] if a in J else [(f"{a}_{s}", f"{b}_{s}") for s in SIDES]
        for x, y in pairs:
            mid = rig.lerp(J[x], J[y], 0.5)
            for i in range(3):
                acc[i] += mid[i] * m
            tot += m
    return tuple(c / tot for c in acc)


ENDS = {"arm": ("wrist", "hand_tip"), "leg": ("knee", "ankle", "toe")}


def check_reach(track):
    """A limb short of its target while its end is a declared contact (a hand that misses the desk, a foot off its
    step); a free limb that falls short only hangs straight."""
    out = []
    for i, (short, fr) in enumerate(zip(track["_short"], track["frames"])):
        contacts = set(fr["contacts"])
        for limb, s in short.items():
            kind, side = limb.split("_")
            if s > REACH_TOL and any(f"{j}_{side}" in contacts for j in ENDS[kind]):
                out.append(f"reach: frame {i}: {limb} falls {s:.1f} cm short of its contact")
    return out


def foot_points(J, side, view):
    """The sabaton's box under a foot (body space): heel, ball and front edge on its sole, 7 cm below the ankle-toe
    line, and the same three on its top, 3 cm above it. The foot turns as the renderer turns it
    (`blender/armor_model.py` frame_foot): in the side view about the hips' lateral axis only, so a foot pointing
    back from the hips lies laces down (a rear foot on a bench) and a figure turned about keeps its soles down; in the
    front view with its heading and pitch (a foot on its ball), its sole square to it, and a foot pointing almost
    straight up or down drawn forward, 8 degrees out. A tilted rigid sole sinks at its edge first, so the feet are
    tested by these points, not by spheres."""
    a, t = J[f"ankle_{side}"], J[f"toe_{side}"]
    d = sub(t, a)
    if view == "side":
        lat = unit(sub(J["hip_far"], J["hip_near"]), (0.0, 0.0, 1.0))
        h = unit(sub(d, rig.mul(lat, dot(d, lat))), (1.0, 0.0, 0.0))
        down = rig.cross(h, lat)
    else:
        flat = (d[0], 0.0, d[2])
        if rig.norm(flat) > 0.35 * rig.norm(d):
            h = unit(d)
        else:
            s = math.sin(math.radians(8)) * (-1.0 if d[2] < 0 else 1.0)
            h = flat = (math.cos(math.radians(8)), 0.0, s)
        down = rig.cross(h, rig.cross(unit(flat), (0.0, 1.0, 0.0)))
    out = []
    for k in (-BONES["heel_back"], BONES["ball_forward"], BONES["sole_front"]):
        on = rig.add(a, rig.mul(h, k))
        out += [rig.add(on, rig.mul(down, BONES["ankle_height"])), rig.add(on, rig.mul(down, -FOOT_TOP))]
    return out


def body_space(v, view):
    """A point or a direction the track holds in the renderer's coordinates (motion.to_view, no limb's plane) in body
    space."""
    return (v[2], v[1], -v[0]) if view == "front" else tuple(v)


def helm_frame(J, fr, view):
    """The helm's frame in body space: the head joint, the head's axis, its facing and its lateral axis (the frame's
    head_l)."""
    up = unit(sub(J["head"], J["neck"]))
    lat = unit(rig.square_to(body_space(fr["head_l"], view), up))
    return J["head"], up, rig.cross(up, lat), lat


def helm_points(J, fr, view):
    """The helm's points (HELM) in body space."""
    h, up, fwd, lat = helm_frame(J, fr, view)
    return [(h[0] + up[0] * a + fwd[0] * b + lat[0] * c, h[1] + up[1] * a + fwd[1] * b + lat[1] * c,
             h[2] + up[2] * a + fwd[2] * b + lat[2] * c) for a, b, c in HELM]


def helm_depth(p, frame):
    """How far a point lies inside the helm (cm, negative outside, -HELM_PAD at most): HELM_GRID read trilinearly at
    the point's place in the helm's frame (helm_frame), on the grid's side (the helm is mirror symmetric)."""
    h, up, fwd, lat = frame
    d = sub(p, h)
    (a0, b0, c0), step, (na, nb, nc), grid = HELM_GRID
    x, y, z = (dot(d, up) - a0) / step, (dot(d, fwd) - b0) / step, (abs(dot(d, lat)) - c0) / step
    if not (0.0 <= x < na - 1 and 0.0 <= y < nb - 1 and 0.0 <= z < nc - 1):
        return -HELM_PAD
    i, j, k = int(x), int(y), int(z)
    fx, fy, fz = x - i, y - j, z - k
    out = 0.0
    for di, wx in ((0, 1.0 - fx), (1, fx)):
        for dj, wy in ((0, 1.0 - fy), (1, fy)):
            row = ((i + di) * nb + j + dj) * nc + k
            out -= wx * wy * ((1.0 - fz) * grid[row] + fz * grid[row + 1])
    return out


def helm_stale():
    """Why helm.json no longer fits the armour (blender/armor_model.py changed since helm.py measured it), or None."""
    with open(os.path.join(os.path.dirname(HELM_FILE), "blender", "armor_model.py"), "rb") as fh:
        if hashlib.sha256(fh.read()).hexdigest() != HELM_ARMOUR:
            return ("tools/figures/helm.json was measured from another blender/armor_model.py: run "
                    "python3 tools/figures/figures.py helm")
    return None


def surfaces(track, i=0):
    """The floor and every prop's boxes at frame i (a scrolling prop moves through the loop)."""
    out = [("floor", props.FLOOR)]
    at = i / len(track["frames"])
    for p in track["props"]:
        out += [(p["kind"], b) for b in props.solids(p, at)]
    return out


def scrolls(track):
    return any(p.get("scroll") for p in track["props"])


def check_solids(track):
    """Each joint's armour, and each sole, against the floor and every prop box: a declared contact may touch (sink
    at most SINK_TOL), anything else stays clear (TOUCH_TOL of slack for the rounded plates). A foot counts as a
    contact when its toe or its ankle is one. The helm past the head's ball stays clear of every box within its
    reach while the head is no contact (one at rest on a surface is the head's ball's, CLEARANCE), and the sash's
    cloth stays out of every box (its knot rides the belt, as the armour does)."""
    out = []
    solids = surfaces(track)
    for i, (J, fr) in enumerate(zip(track["_body"], track["frames"])):
        if scrolls(track):
            solids = surfaces(track, i)
        contacts = set(fr["contacts"])
        tests = [(j, v, clearance(j), j in contacts) for j, v in J.items() if not j.startswith(("toe_", "ankle_"))]
        for s in SIDES:
            on = f"toe_{s}" in contacts or f"ankle_{s}" in contacts
            tests += [(f"foot_{s}", p, 0.0, on) for p in foot_points(J, s, track["view"])]
        for j, v, c, contact in tests:
            for name, box in solids:
                g = props.gap(v, box) - c
                if contact and g < -SINK_TOL:
                    out.append(f"{name}: frame {i}: {j} sinks {-g:.1f} cm into it")
                elif not contact and g < -TOUCH_TOL:
                    out.append(f"{name}: frame {i}: {j} touches it ({-g:.1f} cm in) but is not a contact")
        near = [] if "head" in contacts else [s for s in solids if props.gap(J["head"], s[1]) < HELM_REACH]
        helm = helm_points(J, fr, track["view"]) if near else ()
        for name, box in near:
            g = min(props.gap(p, box) for p in helm)
            if g < -TOUCH_TOL:
                out.append(f"{name}: frame {i}: helm touches it ({-g:.1f} cm in) but is not a contact")
        cloth = [body_space(p, track["view"]) for p in fr["sash"][1:]]
        for name, box in solids:
            g = min(props.gap(p, box) for p in cloth)
            if g < -TOUCH_TOL:
                out.append(f"{name}: frame {i}: sash enters it ({-g:.1f} cm in)")
    return sorted(set(out), key=out.index)


TOUCH_GAP = 2.5          # a contact both poses of a step declare lies within this of a surface (cm)


def check_touch(track):
    """Every contact held through a step (declared by both its poses) touches the floor or a prop: a hand by its
    wrist or fingertips, a foot by any point of its sabaton, any other joint by its armour. Without this a body
    floating above the floor would pass."""
    out = []
    solids = surfaces(track)
    for i, (J, fr) in enumerate(zip(track["_body"], track["frames"])):
        if scrolls(track):
            solids = surfaces(track, i)
        groups = {}
        for c in fr.get("touch", []):
            part, _, side = c.rpartition("_") if c.endswith(("_near", "_far")) else (c, "", "")
            if part in ("toe", "ankle"):
                groups.setdefault(f"foot_{side}", foot_points(J, side, track["view"]))
            elif part in ("wrist", "hand_tip"):
                g = groups.setdefault(f"hand_{side}", [])
                g.append((J[c], clearance(c)))
            else:
                groups[c] = [(J[c], clearance(c))]
        for name, pts in groups.items():
            if name.startswith("foot_"):
                pts = [(p, 0.0) for p in pts]
            gap = min(props.gap(p, box) - cl for p, cl in pts for _, box in solids)
            if gap > TOUCH_GAP:
                out.append(f"touch: frame {i}: {name} is a contact but floats {gap:.1f} cm off every surface")
    return out


def seg_dist(p, a, b):
    ab = sub(b, a)
    n = dot(ab, ab)
    k = 0.0 if n < 1e-9 else max(0.0, min(1.0, dot(sub(p, a), ab) / n))
    return rig.dist(p, rig.add(a, rig.mul(ab, k)))


# The armour's capsules a held implement must stay outside, (joint, joint) with the smaller clearance for radius (the
# hands and forearms that hold it are left out), and those a line (items.shape: a skipping rope, a band, a door towel)
# passes around: the feet too, which a rope passes under.
BODY_SEGMENTS = [("pelvis", "waist"), ("waist", "neck"), ("neck", "head")] + [
    (f"{a}_{s}", f"{b}_{s}") for s in SIDES for a, b in (("hip", "knee"), ("knee", "ankle"), ("shoulder", "elbow"))]
LINE_SEGMENTS = BODY_SEGMENTS + [(f"ankle_{s}", f"toe_{s}") for s in SIDES]
# The capsules a worn pack (items.py "on") or a vest rests on, which its boxes may enter: the back's, the chest's and
# the vest's the trunk, the hips' the pelvis
RESTS = {"back": {("pelvis", "waist"), ("waist", "neck")}, "chest": {("pelvis", "waist"), ("waist", "neck")},
         "hips": {("pelvis", "waist")}, "vest": {("pelvis", "waist"), ("waist", "neck")}}
ITEM_SINK = 1.5          # a held implement may rest against the armour this far (a goblet against the breastplate)
END_SKIP = 12.0          # cm of a line either side of a point a hand holds or a loop ties to the body, left untested
#                          (the fist round a towel or a rope's handle, a band's loop round a foot, a knee or the neck)
LINE_STEP = 2.0          # cm between the points a line is tested at
ROPE_UNDER = 1.0         # cm a skipping rope's top passes under the lower sole, at least
STRAP_DRIFT = 3.0        # cm a ring strap, or a towel the body leans on, may stretch or slacken over the loop
CHAIR_SET = 1.5          # cm over the floor a chair set down at the bottom of a lift may still float


def chair_box(c, half):
    """A held chair's part as a box (f0, f1, u0, u1, l0, l1): the chair stays upright in the body's frame."""
    return (c[0] - half[1], c[0] + half[1], c[1] - half[2], c[1] + half[2], c[2] - half[0], c[2] + half[0])


def box_gap(p, center, axes, half):
    """Distance (cm) from a point to a box along `axes` with `half` its half extents: positive outside, negative
    inside (props.gap in the box's own frame)."""
    d = sub(p, center)
    return props.gap(tuple(dot(d, ax) for ax in axes), (-half[0], half[0], -half[1], half[1], -half[2], half[2]))


def seg_box_gap(a, b, box, n=16):
    """The least distance from a bone (a to b) to a box (center, axes, half), by points along the bone (bones are
    longer than the 3 cm the armour's thinnest clearance needs between them)."""
    return min(box_gap(rig.lerp(a, b, k / n), *box) for k in range(n + 1))


def line_samples(points, radii, held):
    """A line's points every LINE_STEP cm or closer, with their radii, but those within END_SKIP of a held point
    (measured along the line)."""
    at = [0.0]
    for p, q in zip(points, points[1:]):
        at.append(at[-1] + rig.dist(p, q))
    out = []
    for k in range(len(points) - 1):
        n = max(1, math.ceil((at[k + 1] - at[k]) / LINE_STEP))
        for j in range(n + (k == len(points) - 2)):
            s = j / n
            if all(abs(at[k] + (at[k + 1] - at[k]) * s - at[h]) >= END_SKIP for h in held):
                out.append((rig.lerp(points[k], points[k + 1], s), radii[k] + (radii[k + 1] - radii[k]) * s))
    return out


def check_shape(i, J, fr, view, pose):
    """A held implement's solid (items.shape) in the armour: a sphere or a box past ITEM_SINK inside a BODY_SEGMENTS
    capsule (a worn pack's or a vest's in any but the trunk it rests on) or, while the head is no contact, round one of
    the helm's points (helm_points, as check_solids holds a prop's box); a line's points, but near what holds it, inside
    a LINE_SEGMENTS capsule (a skipping rope's at all: it passes round the body, never against it; a band's or a towel's
    past ITEM_SINK) or, while the head is no contact, in the helm by its distance grid (helm_depth)."""
    sh, name, out = items.shape(pose), pose["kind"], []
    free = "head" not in fr["contacts"]
    helm = helm_points(J, fr, view) if free and (sh["spheres"] or sh["boxes"]) else ()
    for c, r in sh["spheres"]:
        for a, b in BODY_SEGMENTS:
            depth = r + min(clearance(a), clearance(b)) - seg_dist(c, J[a], J[b])
            if depth > ITEM_SINK:
                out.append(f"items: frame {i}: the {name} is {depth:.1f} cm inside the {a.split('_')[0]}")
        if helm and rig.dist(c, J["head"]) < HELM_REACH + r:
            depth = r - min(rig.dist(c, q) for q in helm)
            if depth > ITEM_SINK:
                out.append(f"items: frame {i}: the {name} is {depth:.1f} cm inside the helm")
    rests = RESTS.get(pose.get("on") or name, ())
    for box in sh["boxes"]:
        for a, b in BODY_SEGMENTS:
            depth = min(clearance(a), clearance(b)) - seg_box_gap(J[a], J[b], box)
            if (a, b) not in rests and depth > ITEM_SINK:
                out.append(f"items: frame {i}: the {name} is {depth:.1f} cm inside the {a.split('_')[0]}")
        depth = -min(box_gap(q, *box) for q in helm) if helm else 0.0
        if depth > ITEM_SINK:
            out.append(f"items: frame {i}: the {name} is {depth:.1f} cm inside the helm")
    sink = 0.0 if name == "rope" else ITEM_SINK
    frame = None
    for pts, radii, held in sh["lines"]:
        samples = line_samples(pts, radii, held)
        if not samples:                 # a short band from hand to hand: all of it in the fists
            continue
        for a, b in LINE_SEGMENTS:
            r = min(clearance(a), clearance(b))
            depth = max(r + rad - seg_dist(p, J[a], J[b]) for p, rad in samples)
            if depth > sink:
                out.append(f"items: frame {i}: the {name} cuts {depth:.1f} cm into the {a.split('_')[0]}")
        near = [(p, rad) for p, rad in samples if rig.dist(p, J["head"]) < HELM_REACH + rad] if free else []
        if near:
            frame = frame or helm_frame(J, fr, view)
            deep = max(helm_depth(p, frame) + rad for p, rad in near)
            if deep > sink:
                out.append(f"items: frame {i}: the {name} cuts {deep:.1f} cm into the helm")
    return out


def check_chair(i, pose, solids):
    """A held chair outside the props (a desk beside a row); check_shape holds it outside the armour."""
    out = []
    for c, half in pose["boxes"]:
        f0, f1, u0, u1, l0, l1 = chair_box(c, half)
        for kind, s in solids:
            overlap = min(f1 - s[0], s[1] - f0, u1 - s[2], s[3] - u0, l1 - s[4], s[5] - l0)
            if overlap > TOUCH_TOL:
                out.append(f"items: frame {i}: the chair is {overlap:.1f} cm inside the {kind}")
    return out


def check_banner(i, J, pose):
    """A standard's pole (butt to finial) and its cloth (its edges and middle, flat) outside the armour; the hands and
    forearms that hold it are outside BODY_SEGMENTS."""
    out = []
    tip = rig.add(pose["top"], rig.mul(pose["axis"], items.BANNER_FINIAL))
    w, d = pose["width"], pose["drop"] - pose["notch"]
    cloth = [items.banner_cloth(pose, w * a / 4, d * b / 4) for a in range(5) for b in range(5)]
    for a_j, b_j in BODY_SEGMENTS:
        r = min(clearance(a_j), clearance(b_j))
        pole = r + items.BANNER_POLE_R - min(seg_dist(rig.lerp(pose["butt"], tip, k / 40), J[a_j], J[b_j])
                                             for k in range(41))
        if pole > ITEM_SINK:
            out.append(f"items: frame {i}: the banner's pole is {pole:.1f} cm inside the {a_j.split('_')[0]}")
        sheet = r - min(seg_dist(p, J[a_j], J[b_j]) for p in cloth)
        if sheet > ITEM_SINK:
            out.append(f"items: frame {i}: the banner's cloth is {sheet:.1f} cm inside the {a_j.split('_')[0]}")
    return out


def check_rope(track, item):
    """Each pass of a skipping rope's middle straight down (items.rope_turn a whole number) finds the feet in the air
    over it: at that moment (the joints between its two frames) the top of its lowest point passes ROPE_UNDER under
    the lower sole, so the rope reads as passing under the feet, a double-under's two passes in one jump."""
    out = []
    body, n = track["_body"], len(track["_body"])
    loop = n / motion.HZ
    turn0, turns = items.rope_turn(item, 0.0, loop), item.get("turns", 1)
    for k in range(math.floor(min(turn0, turn0 + turns)), math.ceil(max(turn0, turn0 + turns)) + 1):
        t = (k - turn0) / turns * loop
        if not 0.0 <= t < loop:
            continue
        x = t * motion.HZ
        a, b, w = body[int(x) % n], body[(int(x) + 1) % n], x - int(x)
        J = {j: rig.lerp(a[j], b[j], w) for j in a}
        top = min(p[1] for p in items.place(item, J, t, loop)["points"]) + items.ROPE_THICK
        sole = min(p[1] for s in SIDES for p in foot_points(J, s, track["view"]))
        if sole - top < ROPE_UNDER:
            out.append(f"items: the rope passes under the feet at {t:.2f} s, its top {sole - top:.1f} cm under the "
                       f"lower sole (at least {ROPE_UNDER}: it passes under them in the air)")
    return out


def check_items(track, loop=True):
    """Every item's own problems (items.check); its solid in the armour (check_shape); a chair inside a prop; a
    standard's pole or cloth inside the armour; and over the loop (`loop`; a fault still's track is no loop) a skipping
    rope that passes straight down with a sole on or under it (check_rope), a ring strap whose length drifts (a
    straight strap from a fixed anchor keeps its length), a door towel's held tail whose span stretches or slackens,
    and a chair set down at the bottom of a lift (`set`) that never reaches the floor."""
    out, straps, towels, chairs = [], {}, {}, {}
    for i, (J, fr) in enumerate(zip(track["_body"], track["frames"])):
        for pose in fr["items"]:
            out += [f"items: frame {i}: {p}" for p in items.check(pose)]
            out += check_shape(i, J, fr, track["view"], pose)
            if pose["kind"] == "doortowel":
                for n, (a, b) in enumerate(pose["spans"]):
                    towels.setdefault(n, []).append(rig.dist(a, b))
            if pose["kind"] == "chair":
                out += check_chair(i, pose, [s for s in surfaces(track, i) if s[0] != "floor"])
                if pose["set"]:
                    chairs.setdefault("set", []).append(pose["low"])
            if pose["kind"] == "banner":
                out += check_banner(i, J, pose)
            if pose["kind"] == "straps":
                for ln in pose["lines"]:
                    straps.setdefault(ln["side"], []).append(rig.dist(ln["anchor"], ln["ring"]))
    if not loop:
        return out
    for item in track["items"]:
        if item["kind"] == "rope":
            out += check_rope(track, item)
    for side, lengths in straps.items():
        if max(lengths) - min(lengths) > STRAP_DRIFT:
            out.append(f"items: the {side} ring strap's length drifts {max(lengths) - min(lengths):.1f} cm over the "
                       "loop (a strap keeps its length: solve the pose to it)")
    for n, lengths in towels.items():
        if max(lengths) - min(lengths) > STRAP_DRIFT:
            out.append(f"items: the towel's tail {n + 1} drifts {max(lengths) - min(lengths):.1f} cm over the loop "
                       f"({min(lengths):.1f}-{max(lengths):.1f}; a towel the body leans on keeps its length: solve "
                       "the pose to it)")
    for lows in chairs.values():
        if min(lows) > CHAIR_SET:
            out.append(f"items: the chair never reaches the floor ({min(lows):.1f} cm over it at the lowest; it is "
                       "set down at the bottom of each lift)")
    return out


def check_bend(track):
    """A knee or an elbow folded past MAX_BEND, or a knee bent backwards: the knee must stand out from the hip-ankle
    line toward the thigh's front (the lateral axis crossed with the thigh), whatever the foot does. A knee that stands
    out mostly sideways (a hip turned out: a figure-four, a pigeon) is not judged, since the rig has no hip rotation to
    say which way its front faces."""
    out = []
    for i, J in enumerate(track["_body"]):
        lat = unit(sub(J["hip_far"], J["hip_near"]), (0.0, 0.0, 1.0))
        for s in SIDES:
            for a, b, c in (("hip", "knee", "ankle"), ("shoulder", "elbow", "wrist")):
                bend = angle(sub(J[f"{b}_{s}"], J[f"{a}_{s}"]), sub(J[f"{c}_{s}"], J[f"{b}_{s}"]))
                if bend > MAX_BEND:
                    out.append(f"bend: frame {i}: {b}_{s} folds {bend:.0f} deg")
            hip, knee, ankle = (J[f"{k}_{s}"] for k in ("hip", "knee", "ankle"))
            line = unit(sub(ankle, hip))
            off = sub(knee, hip)
            perp = sub(off, rig.mul(line, dot(off, line)))
            front = rig.cross(lat, unit(off))
            thigh = unit(off)
            side = unit(sub(lat, rig.mul(thigh, dot(lat, thigh))), lat)
            if rig.norm(perp) > 2.0 and abs(dot(unit(perp), side)) < 0.7 and dot(perp, front) < 0:
                out.append(f"bend: frame {i}: knee_{s} bends backwards")
    return out


def foot_twist(J, side):
    """The angle (deg) between the foot's heading about its shin and the side its bent knee stands out to, or None
    while the knee is nearly straight or the foot runs along the shin."""
    hip, knee, ankle, toe = (J[f"{k}_{side}"] for k in ("hip", "knee", "ankle", "toe"))
    shin, foot = unit(sub(ankle, knee)), unit(sub(toe, ankle))
    if angle(sub(knee, hip), shin) < TWIST_KNEE:
        return None
    out = rig.square_to(sub(knee, hip), shin)          # the side the knee stands out to, square to the shin
    heading = rig.square_to(foot, shin)
    if rig.norm(heading) < 0.3:
        return None
    return angle(out, heading)


def check_range(track):
    """An ankle, a foot's turn about its shin or a wrist past what the joint can do (ANKLE_MAX, TWIST_MAX,
    WRIST_MAX): a pose or a blend that folds a foot onto its shin, turns it to face behind its knee, or bends a hand
    back onto its forearm."""
    out = []
    for i, J in enumerate(track["_body"]):
        for s in SIDES:
            ankle = angle(sub(J[f"ankle_{s}"], J[f"knee_{s}"]), sub(J[f"toe_{s}"], J[f"ankle_{s}"]))
            if ankle > ANKLE_MAX:
                out.append(f"range: frame {i}: ankle_{s} folds {ankle:.0f} deg")
            twist = foot_twist(J, s)
            if twist is not None and twist > TWIST_MAX:
                out.append(f"range: frame {i}: foot_{s} turns {twist:.0f} deg away from its knee")
            wrist = angle(sub(J[f"wrist_{s}"], J[f"elbow_{s}"]), sub(J[f"hand_tip_{s}"], J[f"wrist_{s}"]))
            if wrist > WRIST_MAX:
                out.append(f"range: frame {i}: wrist_{s} bends {wrist:.0f} deg")
    return out


def check_pop(track):
    """A joint whose travel in one tick changes by more than POP_CM (its second difference; the loop's last frame
    runs into its first): an elbow or a knee flipping over, a seam that snaps back, a pose that teleports."""
    out = []
    body = track["_body"]
    n = len(body)
    if n < 3:
        return out
    for i in range(n):
        a, b, c = body[i - 1], body[i], body[(i + 1) % n]
        for j in b:
            jolt = rig.norm(sub(rig.add(a[j], c[j]), rig.mul(b[j], 2.0)))
            if jolt > POP_CM:
                out.append(f"pop: frame {i}: {j} jumps {jolt:.0f} cm in one tick")
    return out


def resting(J, joint, props_list, view, at=0.0):
    """Whether a contact joint bears weight on the floor or a prop top: its armour (a foot: any point of its box)
    within 3 cm of the surface under it."""
    if joint.startswith(("toe_", "ankle_")):
        pts, c = foot_points(J, joint.split("_", 1)[1], view), 0.0
    else:
        pts, c = [J[joint]], clearance(joint)
    return any(abs(p[1] - props.top_under(p, props_list, at) - c) <= 3.0 for p in pts)


SOLE_HALF = 5.0          # the sabaton's half width, for the side-to-side support


def check_balance(track):
    out = []
    if any(it.get("kind") == "straps" for it in track["items"]):
        return out            # weight on ring straps: supported from above, as on a bar
    for i, (J, fr) in enumerate(zip(track["_body"], track["frames"])):
        contacts = fr["contacts"]
        at = i / len(track["frames"])
        if fr.get("dynamic") or not contacts or not all(resting(J, c, track["props"], track["view"], at)
                                                        for c in contacts):
            continue          # airborne, pushing off, hanging, or leaning on a wall or a bar: not balanced on a base
        pts = [it["center"] for it in fr["items"] if it["kind"] == "wheel"]    # an ab wheel bears weight too
        for c in contacts:
            if c.startswith(("toe_", "ankle_")):
                pts += foot_points(J, c.split("_", 1)[1], track["view"])
            else:
                pts.append(J[c])
        g = com(J)
        xs = [p[0] for p in pts]
        if not (min(xs) - BALANCE_MARGIN <= g[0] <= max(xs) + BALANCE_MARGIN):
            out.append(f"balance: frame {i}: the centre of mass at f={g[0]:.0f} is outside the support "
                       f"{min(xs):.0f}..{max(xs):.0f}")
        if track["view"] == "front":            # side to side shows only from behind
            ls = [p[2] for p in pts]
            lo, hi = min(ls) - SOLE_HALF - BALANCE_MARGIN, max(ls) + SOLE_HALF + BALANCE_MARGIN
            if not lo <= g[2] <= hi:
                out.append(f"balance: frame {i}: the centre of mass at l={g[2]:.0f} is outside the support "
                           f"{lo:.0f}..{hi:.0f} side to side")
    return out


LIMBS = [("pelvis", "waist"), ("waist", "neck")] + [
    (f"{a}_{s}", f"{b}_{s}") for s in SIDES for a, b in (("hip", "knee"), ("knee", "ankle"), ("shoulder", "elbow"),
                                                          ("elbow", "wrist"))]


def check_limbs(track):
    """A bone passing through a prop between its joints (the joint checks test only the joints): three points along
    each bone against every prop box, at the bone's thinner clearance, unless an end of the bone is a declared
    contact (a forearm on the desk, a shin on the step)."""
    if not track["props"]:
        return []
    out = []
    solids = [s for s in surfaces(track) if s[0] != "floor"]
    for i, (J, fr) in enumerate(zip(track["_body"], track["frames"])):
        if scrolls(track) or any(p.get("belt") for p in track["props"]):
            solids = [s for s in surfaces(track, i) if s[0] != "floor"]
        contacts = set(fr["contacts"])
        for a, b in LIMBS:
            if a in contacts or b in contacts:
                continue
            r = min(clearance(a), clearance(b))
            for k in (0.25, 0.5, 0.75):
                p = rig.lerp(J[a], J[b], k)
                for name, box in solids:
                    g = props.gap(p, box) - r
                    if g < -TOUCH_TOL:
                        out.append(f"{name}: frame {i}: the {a.rsplit('_', 1)[0]}-{b.rsplit('_', 1)[0]} bone passes "
                                   f"{-g:.1f} cm into it")
    return sorted(set(out), key=out.index)


def dose(move):
    """The dose (rep, hold, cardio or flow) contract/moves.json gives the move a figure key draws (a gear variant's
    move's), or None for a move the library lacks."""
    global _DOSES
    if _DOSES is None:
        with open(os.path.join(ROOT, "contract", "moves.json")) as fh:
            _DOSES = {m["key"]: m["dose"] for m in json.load(fh)["moves"].values()}
    return _DOSES.get(motion.drawn(move)[0])


def gear_problems(move):
    """A gear variant's gear (`<move>.<gear>`) is an equipment key: one of contract/sheet.json's gear_order (ns.Gear,
    the game's list)."""
    global _GEAR
    gear = motion.drawn(move)[1]
    if gear is None:
        return []
    if _GEAR is None:
        with open(os.path.join(ROOT, "contract", "sheet.json")) as fh:
            _GEAR = set(json.load(fh)["gear_order"])
    return [] if gear in _GEAR else [f"gear: {gear} is no equipment key (contract/sheet.json gear_order, ns.Gear)"]


def phase_spans(steps):
    """[(phase, seconds)] of the moving steps, contiguous steps of one phase summed."""
    out, moving = [], False
    for st in steps:
        if "to" not in st:
            moving = False                     # a hold ends a movement: the same phase after it is a new one
            continue
        if moving and out and out[-1][0] == st.get("phase"):
            out[-1] = (out[-1][0], out[-1][1] + st["s"])
        else:
            out.append((st.get("phase"), st["s"]))
        moving = True
    return out


def check_tempo(spec, track):
    """A controlled rep's moving phases last 0.8-4 s each (the plan's 0.8-2 s, stretched for the tempo and
    lowering moves); a ballistic rep's or a cardio move's steps at least 0.1 s; every loop 0.5-12 s (a flow's up to
    20 s) and closed."""
    out = []
    d = dose(spec["move"])
    if spec.get("extra"):
        if d is not None:
            out.append(f"tempo: the extra {spec['move']} is named as a library move (contract/moves.json): an extra "
                       "is no move")
        d = None
    elif d is None:
        out.append(f"tempo: {spec['move']} is not in contract/moves.json")
    if d == "rep" and not spec.get("ballistic"):
        for ph, s in phase_spans(spec["timeline"][1:]):
            if not PHASE_S[0] <= s <= PHASE_S[1]:
                out.append(f"tempo: phase {ph} lasts {s:.2f} s (a controlled rep's phases take "
                           f"{PHASE_S[0]}-{PHASE_S[1]} s)")
    for i, st in enumerate(spec["timeline"][1:], 1):
        if "to" in st and st["s"] < STEP_MIN:
            out.append(f"tempo: step {i} lasts {st['s']} s (at least {STEP_MIN})")
    top = LOOP_FLOW_S if d == "flow" else LOOP_S[1]
    if not LOOP_S[0] <= track["loop_s"] <= top:
        out.append(f"tempo: the loop lasts {track['loop_s']} s (allowed {LOOP_S[0]}-{top})")
    segs = motion.timeline(spec)[0]
    start, end = motion.solve(segs[0][2])[0], motion.solve(segs[-1][3])[0]   # the poses themselves, not a blend
    if max(rig.dist(start[j], end[j]) for j in start) > 0.5:
        out.append("tempo: the loop does not end at its start pose")
    return out


def check_entry(spec, track):
    """A hold's loop starts ENTRY_CM or more from its key pose and a pose's (a flow's) loop leaves its key pose by as
    much: the figure shows how the player gets into the position, never a still pose that only breathes."""
    d = dose(spec["move"])
    if spec.get("extra") or d not in ("hold", "flow"):
        return []
    frames = track["frames"]
    key = frames[track["key"]]["joints"]

    def gap(fr):
        return max(rig.dist(fr["joints"][j], key[j]) for j in key)
    if d == "hold" and gap(frames[0]) < ENTRY_CM:
        return [f"entry: the hold starts {gap(frames[0]):.1f} cm from its key pose (at least {ENTRY_CM:g}: the loop "
                "starts where the player sets up and shows the way in)"]
    if d == "flow" and max(map(gap, frames)) < ENTRY_CM:
        return [f"entry: the pose's loop stays within {max(map(gap, frames)):.1f} cm of its key pose (at least "
                f"{ENTRY_CM:g}: the loop shows the way into the pose)"]
    return []


def frame_of(spec, track, at):
    """The frame where the timeline first arrives at pose `at` (or the time `at`, s)."""
    return motion.key_frame(dict(spec, key=at), motion.timeline(spec)[0], len(track["frames"]))


def misses(t, v):
    """How a measured value misses a target's bounds ("above 40", "below 170"), or None when the target holds."""
    if "max" in t and v > t["max"]:
        return f"above {t['max']}"
    if "min" in t and v < t["min"]:
        return f"below {t['min']}"
    return None


def check_targets(spec, track):
    out = []
    for t in spec.get("targets", []):
        v, label = motion.measure(t, track["_body"][frame_of(spec, track, t["at"])], track["view"])
        miss = misses(t, v)
        if miss:
            out.append(f"targets: at {t['at']}: {label} {miss}")
    return out


def check_poses(track, loop=True):
    """The checks every frame passes by its pose alone, in check order, with the items' checks over the loop when the
    track is one (`loop`)."""
    return (check_reach(track) + check_solids(track) + check_limbs(track) + check_touch(track)
            + check_items(track, loop) + check_bend(track) + check_range(track))


def check_fault(spec, track, loop):
    """The fault still of a move that has one, on the track it renders from (motion.fault_track): `loop` holds the
    loop's own pose problems, so every other line check_poses finds there is the still's; the target the fault
    `breaks` is measured at the key pose, where it holds, and at the fault pose, where it fails; and the camera
    fitted with the fault pose added is the loop's, since the still is cut with the loop's camera and crop."""
    if "_fault" not in track:
        return []
    fault, still = spec["fault"], track["_fault"]
    name, t = fault["pose"], fault["breaks"]
    out = []
    for line in check_poses(motion.fault_track(track), loop=False):
        if line not in loop:
            kind, _, text = line.partition(": ")
            out.append(f"fault: pose {name}: {kind}: {text.split(': ', 1)[-1] if text.startswith('frame ') else text}")
    v, label = motion.measure(t, track["_body"][track["key"]], track["view"])
    miss = misses(t, v)
    if miss:
        out.append(f"fault: the key pose misses the target the fault breaks: {label} {miss}")
    v, label = motion.measure(t, still["body"], track["view"])
    if not misses(t, v):
        outside = " or ".join(f"{word} {t[k]}" for k, word in (("min", "below"), ("max", "above")) if k in t)
        out.append(f"fault: pose {name} does not break its target: {label}, not {outside}")
    cam = motion.camera_of(spec, track["_body"] + [still["body"]], track["frames"] + [still["frame"]])
    if cam != track["camera"]:
        was = track["camera"]
        diff = ", ".join(f"{k} {cam[k]} (the loop's {was.get(k)})" for k in cam if cam[k] != was.get(k))
        out.append(f"fault: pose {name} leaves the loop's frame: the camera's {diff}")
    return out


def check_show(track):
    """Every prop part clears the card at its darkest: a part narrower than THIN_CM across (its second-largest extent:
    a rod's or a post's thickness, a leg's side) by THIN_MIN, any other part by SHOW_MIN. Its darkest is the darkest
    stop of its material's ramp (style.json; a face turned from both prop lights, or in a shadow, takes it), on stone a
    brick at its darkest block (its mortar lines are lines, not faces). The ramp blends its stops in linear light, so
    no colour between two stops is darker than both. The held implements are the figure's own toon materials
    (blender/scene.py `Items`), outside this check."""
    with open(STYLE) as fh:
        style = json.load(fh)
    ps = style["props"]
    card = luminance(style["card"])
    out = []
    for p in track["props"]:
        boxes, rods = props.parts(p)
        sides = [(sorted((f1 - f0, u1 - u0, l1 - l0))[1], m) for (f0, f1, u0, u1, l0, l1), m in boxes]
        sides += [(sorted((2 * r, 2 * r, math.dist(a, b)))[1], m) for (a, b, r), m in rods]
        for thin, mat in sorted({(side < THIN_CM, m) for side, m in sides}):
            dark = min(luminance(c) for _, c in ps[mat + "_ramp"])
            if mat == "stone":
                dark *= 1 - ps["brick"]["block_var"]
            r = (dark + 0.05) / (card + 0.05)
            floor = THIN_MIN if thin else SHOW_MIN
            if r < floor:
                out.append(f"show: a {p['kind']}'s {'thin part' if thin else 'face'} in style.json {mat}_ramp is "
                           f"{r:.2f}:1 against the card at its darkest, below {floor}")
    return out


def run(spec, track):
    """Every problem of a baked motion, in check order (an empty list passes)."""
    poses = check_poses(track)
    return (poses + check_balance(track) + check_pop(track) + check_tempo(spec, track) + check_entry(spec, track)
            + check_targets(spec, track)
            + check_fault(spec, track, poses) + check_show(track) + gear_problems(spec["move"]))


def summary(found):
    """Problems grouped by what they say, with the frames and the worst of each line's first number: one line per
    fault, not per frame."""
    import re
    num = r"-?\d+(?:\.\d+)?"
    groups = {}
    for line in found:
        m = re.match(r"(\w+): frame (\d+): (.*)", line)
        if not m:
            groups.setdefault(line, [])
            continue
        kind, frame, text = m.group(1), int(m.group(2)), m.group(3)
        nums = [float(x) for x in re.findall(num, text)]
        key = f"{kind}: " + re.sub(num, "#", text)
        groups.setdefault(key, []).append((frame, nums[0] if nums else None))
    out = []
    for key, hits in groups.items():
        if not hits:
            out.append(key)
            continue
        frames = [f for f, _ in hits]
        worst = max((v for _, v in hits if v is not None), key=abs, default=None)
        text = key.replace("#", f"{worst:g}", 1).replace("#", "..") if worst is not None else key
        out.append(f"{text} (frames {min(frames)}-{max(frames)}, {len(frames)} of them)")
    return out


def variant_problems(spec, ref_spec):
    """A timing variant against its reference move: every phase it names exists there, and the phases it retimes stay
    within a controlled rep's bounds (its dose permitting)."""
    out = []
    runs = {}                                  # the reference's baked phases and how many runs of ticks each has
    last = None
    for fr in motion.bake(ref_spec)["frames"]:
        if fr["phase"] != last:
            runs[fr["phase"]] = runs.get(fr["phase"], 0) + 1
            last = fr["phase"]
    names = {p for p, n in runs.items() if n == 1}
    for ph, s in (spec.get("phases") or {}).items():
        if ph not in names:
            out.append(f"variant: {spec['move']} retimes phase {ph}, which is not one run of ticks in "
                       f"{ref_spec['move']}")
        elif dose(spec["move"]) == "rep" and not PHASE_S[0] <= s <= PHASE_S[1]:
            out.append(f"variant: phase {ph} lasts {s} s (a controlled rep's phases take {PHASE_S[0]}-{PHASE_S[1]} s)")
    pause = spec.get("pause")
    if pause:
        if pause["phase"] not in names:
            out.append(f"variant: {spec['move']} pauses in phase {pause['phase']}, which is not one run of ticks "
                       f"in {ref_spec['move']}")
        if not 0.0 <= pause.get("at", 0.5) <= 1.0 or pause.get("s", 0) <= 0:
            out.append("variant: a pause needs 0 <= at <= 1 and s > 0")
    if dose(spec["move"]) is None:
        out.append(f"variant: {spec['move']} is not in contract/moves.json")
    return out + gear_problems(spec["move"])
