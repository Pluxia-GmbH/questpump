"""Held implements, placed from the joints each frame (pure Python; docs/plan.md slice 10). They move with the figure,
so the renderer draws them in the figure pass (`blender/scene.py`), never in the static prop layer that lies under the
figure. `shape` is each placed item's solid (spheres, boxes, lines with their radii) for the checks, which keep the
items out of the floor and the armour; the renderer builds the same parts from this module's measures.

An item in a motion file:

  {"kind": "barbell"}                          a light bar with technique plates, gripped by both hands
  {"kind": "kettlebell"}                       held in both hands, the bell hanging on along the arms; "hand": "near"
                                               or "far" holds it in one hand; "hold": "down" hangs the bell straight
                                               below the handle (a goblet at the chest, a carry), "up" stands it
                                               bottom up above the handle (a halo); the handle sits in the closed
                                               fingers (`hook`), its bar running from hand to hand, and the bell
                                               hangs BELL_DROP from it, clear of the fingertips, as a real bell's
                                               window holds the fist; a hand that points away from the bell (a
                                               goblet's, up its horns) holds the horns under the bar, the bell
                                               clear of the wrist; "on": "hips" rests the bell on the front of
                                               the hips, its handle above it and across the body, for the hands on
                                               its horns (a hip thrust)
  {"kind": "backpack", "hand": "near"}         a loaded backpack hanging from one hand by its top strap in the closed
                                               fingers, its broad face along the body's side ("both": the top handle in
                                               both hands, its broad face square to the body); "on": "back" worn on the
                                               back, seated on the backplate and the cuirass's lower edge; "on":
                                               "chest" hugged to the chest by both arms, lying across the breastplate
                                               (a weighted sit-up); "on": "hips" lying across the hips and up the
                                               belly (a hip thrust)
  {"kind": "rope", "turns": -2, "phase": 0.25}   a skipping rope from hand to hand, `turns` per loop, its middle
                                               starting at `phase` of a turn past straight down (under the feet) and
                                               bowing `r` cm (default 96) out from the hands, the ends at the hands
                                               ROPE_LEAD degrees ahead of it, so the rope trails the hands in a curve;
                                               a positive turn sweeps down, forward, up, back (backward skipping for a
                                               figure facing +f), so forward skipping takes negative turns
  {"kind": "wheel"}                            an ab wheel under both hands, rolling on the floor
  {"kind": "towel", "feet": "both", "at": "heels"}   a folded towel on the floor under the heels (or "toes"), of
                                               both feet or one side's; "under": a joint (a kneeling "knee_near",
                                               the forehead's "head") puts it on the floor under that joint
  {"kind": "dumbbell", "hand": "both", "grip": "neutral"}   a dumbbell in each hand ("near" or "far": one), its
                                               handle across the fist: "neutral" (thumbs forward when the arms
                                               hang), "across" (along the shoulders), "palms_down" (level and square
                                               to the hand: a lateral raise); "grip": "goblet" is one dumbbell held
                                               upright at the chest by both hands, and with "on": "hips" that one
                                               dumbbell lies across the front of the hips under both hands (a hip
                                               thrust; the renderer builds one dumbbell for the goblet grip); with
                                               "on": "feet" it stands between the feet, gripped by them alone: its
                                               handle between the ankles along the shins, one head under the soles,
                                               the other above the feet (a weighted reverse hyper)
  {"kind": "ball", "r": 12}                    a medicine ball between the palms (`off` [f, u, l] cm from the grips'
                                               midpoint); `free` windows let it go (see below)
  {"kind": "vest"}                             a weighted vest: plates on the chest and the back, riding the torso
  {"kind": "band", "strands": [[a, b], ...]}   an elastic band, one straight strand per pair of ends; an end is a
                                               point [f, u, l] (an anchor in a door) or a body point: "grip_near",
                                               "grip_far", "grips" (between both), "sole_near", "sole_far", "knee_near",
                                               "knee_far", "ankle_near", "ankle_far", "toe_near", "toe_far" (a strap
                                               looped round a raised foot, its tails over the toes), "neck", or
                                               {"at": name, "off":
                                               [f, u, l]} (or "torso": [f, u, l], the offset in the torso's frame,
                                               forward, up the spine and across, the near side negative: a band
                                               round the back of the neck that hinges with it)
  {"kind": "landmine", "pivot": [f, u, l], "len": 190}   a rigid bar from its end in a corner (the pivot) to the
                                               hands `len` cm along it, a plate on the sleeve just below the grip
                                               ("hand": "near", "far" or "both"); the hands stay `len` from the pivot
  {"kind": "pedals", "at": [f, u], "r": 17}    a bike's cranks: an arm from the bottom bracket to each foot's pedal
                                               under the ball of the foot, the ball's circle the crank's
  {"kind": "straps", "at": [f, u], "half": 22, "to": "grips"}   gym rings on straps from ceiling anchors at (f, u,
                                               +-half) to the hands (or "heels": the heels in the straps), so the
                                               rings swing with the body
  {"kind": "doortowel", "knot": [f, u, l], "ends": ["grip_near", "grip_far"], "loose": [f, u, l]}
                                               a bath towel knotted and shut in a door (slice 21): a tail from the knot
                                               at the door's edge to each end (a body point, as a band's), the towel
                                               twisted into a soft rope; "loose" hangs one more tail from the knot to
                                               that point (a one-arm move, the other hand let go). "top": [f, u, l]
                                               in place of "knot" hangs the towel over the top of a door, its knot on
                                               the far side out of sight: its tails come over the top edge at that
                                               point (on the door's near face) side by side, ordered across as their
                                               ends are, and down to the ends and the loose one. "over": [f, u, l]
                                               in place of "knot" hangs the towel over a bar (props.py's) at that
                                               point on its axis: folded over the bar's top, the first end's tail down
                                               its front, the other's down its back. Each tail sags a little under its
                                               weight and ends wrapped in the fist that holds it. A towel the body
                                               leans on keeps its length: the checks hold each held tail's span (its
                                               top to the grip) to it
  {"kind": "chair", "hand": "both", "off": 12}   a light chair without wheels (props.py's chair: seat, legs, back)
                                               held upright: "both" by the seat's sides, the seat `off` cm forward of
                                               the grips and its back away from the body (a chair deadlift); "near" or
                                               "far" by the middle of the seat's side next to the leg, the chair
                                               beside it and its back toward the body's rear (a one-arm chair row);
                                               "set": true when the move sets it down between reps (the checks hold
                                               its lowest frame to the floor)
  {"kind": "banner", "hand": "near", "foot": [f, l]}   a standard (slice 45, the streak card's standard-bearer): a
                                               wooden pole `len` cm long (default 250) with a gold finial, standing
                                               on the floor at (f, 0, l) and running up through that hand's grip; or
                                               "hand": "both", the pole through both grips, its butt `low` cm (default
                                               60) below the lower one. A gold crossbar under the finial carries a
                                               cloth in the class colour (the armour's own cloth, so the tint shows
                                               the player's colour on it as on the sash) `width` cm wide (default 52)
                                               and `drop` cm long (default 92), its inner edge along the pole, flying
                                               forward of the pole (`"fly": -1`: behind it), its foot cut into two
                                               tails `notch` cm deep (default 18), edged in gold, with a device of two
                                               gold chevrons on both faces

A ball's `free` windows ({"leave": s, "land": s, "lift": s, "back": s, "at": [f, u, l]}, times in the loop) let it
fly from the hands at `leave` to `at` by `land`, rest there until `lift`, and fly back into the hands by `back` (a
slam: land on the floor, lift when the hands reach it; a wall pass: land and lift together at the wall). A window may
wrap past the loop's end (a ball that starts on the floor). `arc` (cm) bows the flights upward; `ease` "in" speeds a
drop up like a fall.

`place(item, J, t, loop)` returns the item's pose for one frame in body space (cm): a dict the renderer reads, with
the points named per kind above, and `low` the item's lowest point's height (for the floor check). `shape(pose)`
gives its solid from the same pose.
"""
import math

import props
import rig
from rig import BONES, SIDES, add, cross, dot, mul, sub, unit

BAR_HALF, PLATE_R, PLATE_AT = 100.0, 22.5, 70.0
# A handle held in the closed fingers (a kettlebell's, a bag's top strap) sits HOOK cm from the wrist along the hand.
# The gauntlet's fingers reach FINGERS from the wrist (blender/armor_model.py build_hand, measured 2026-10-22), so
# they end FINGERS - HOOK past the handle, and what hangs from it clears them by WINDOW, as a real bell's window holds
# the fist.
HOOK, FINGERS, WINDOW = 12.5, 18.1, 1.5
BELL_R = 10.0
BELL_DROP = FINGERS - HOOK + WINDOW + BELL_R     # the bell's centre from the handle's, a real bell's 17 cm
BELL_HORN, BELL_HORN_AT, BELL_HANDLE_R = 8.5, 4.5, 1.3   # the handle at its widest, how far under the bar that is,
#                                                    and its round
BELL_SHOULDER = 38.0                             # deg from the bell's top where the horns meet it
BELL_POINTS = 15
# A backpack's box: across its broad face, its depth, its height (a daypack's: hung below the fingertips by its top
# strap, it clears the floor at backpack_deadlift's bottom, the key pose)
PACK_W, PACK_D, PACK_H = 30.0, 16.0, 32.0
PACK_DROP = FINGERS - HOOK + WINDOW              # a bag's top under the handle in the fist
PACK_LOOP = 5.0                                  # half the top handle's span across the bag's top
# The cuirass measured in its own frame (blender/armor_model.py build_chest, 2026-10-22; cm up the spine from the
# waist, cm behind or before the waist-neck line, across |l| < 15): a pack worn on the back rests on the backplate's
# rearmost point and on the cuirass's lower edge, its face along the line through both, since the back slopes from
# the shoulder blades to the waist; a pack hugged to the chest, and a vest's front plate, rest on the breastplate's
# foremost point, the vest's back plate on the backplate's rearmost.
BACK_SEAT = ((17.0, 14.7), (-4.0, 12.0))
CHEST_FRONT = 18.1
ROPE_R, ROPE_THICK = 96.0, 0.6
ROPE_LEAD = 40.0         # deg the rope's ends at the hands lead its middle round the turn: the rope trails the hands
ROPE_FLARE = 10.0        # cm the rope's sides swing out past the hands at their widest, led out by the handles
ROPE_POINTS = 25
WHEEL_R = 9.0
TOWEL_T = 1.0
DB_HALF, DB_HEAD_R, DB_HEAD_AT = 15.0, 5.5, 10.5   # half the dumbbell's length, its heads' radius and centres
BALL_R = 12.0
VEST_W, VEST_T, VEST_H = 30.0, 4.0, 32.0  # a vest's plate: across, its thickness, its height
VEST_AT = 0.55                           # the plates' middles this far from the waist to the neck
HIPS_UP = 6.0                            # a bell or a dumbbell on the hips lies this far up the spine from the pelvis
FEET_PAST = 9.0                          # a dumbbell between the feet: its middle lies this far past the ankles
# A pack hugged to the chest: its centre this far from the waist to the neck, its lower edge clear of the thighs at a
# weighted sit-up's top. A pack on the hips bridges the crease: its back HIPS_FRONT before the pelvis (the abdomen's
# plates stand 16.2 cm out there, probed 2026-10-22) and its centre PACK_HIPS up the spine, its lower edge 6 cm over
# the hip joints, clear of the thighs at a hip thrust's bottom (the tassets lift with the flexed thigh)
CHEST_AT, HIPS_FRONT, PACK_HIPS = 0.62, 17.5, 21.0
LANDMINE_OVER, LANDMINE_PLATE_R, LANDMINE_PLATE_AT = 6.0, 17.0, 16.0
STRAP_HALF, RING_R = 22.0, 9.0
BAND_R = 1.5                             # an elastic band's strand
TOWEL_ROPE_R, KNOT_R = 2.6, 5.0          # a bath towel twisted into a rope, and its knot at the door's edge
# A door towel's tail sags under its weight by TOWEL_SAG of its span (gravity's part square to it) and ends wrapped in
# the fist that holds it, TOWEL_WRAP round for TOWEL_THROUGH cm past the grip and closing to TOWEL_TIP round over
# TOWEL_WRAP / 2 more, so no end hangs out of the fist along a raised forearm; over a bar it lies on the bar flattened
# to TOWEL_FLAT of its round along TOWEL_ARC points, and its two layers bow TOWEL_BOW cm apart
TOWEL_SAG, TOWEL_WRAP, TOWEL_THROUGH, TOWEL_FLAT, TOWEL_BOW = 0.06, 4.0, 3.0, 0.55, 1.5
TOWEL_TIP = TOWEL_ROPE_R / 2
TOWEL_POINTS = 10                        # points along a tail from its top to the fist
TOWEL_ARC = 7                            # points over the bar, from where the first tail leaves it to the other's
CHAIR_GRIP = 1.0                         # the seat's top under a hand that wraps its edge
CHAIR_FIT = 3.0                          # cm the hands may sit off the seat's sides
# A standard (slice 45): the pole's length and radius, how far under the pole's top the crossbar carries the cloth (the
# finial's ferrule above it), the finial's height over the top, the cloth's width, drop and tails' notch, and how far
# under the lower grip a pole held in both hands ends
BANNER_LEN, BANNER_POLE_R, BANNER_CAP, BANNER_FINIAL = 250.0, 2.2, 10.0, 16.0
BANNER_WIDTH, BANNER_DROP, BANNER_NOTCH, BANNER_LOW = 52.0, 92.0, 18.0, 60.0
KINDS = ("barbell", "kettlebell", "backpack", "rope", "wheel", "towel", "dumbbell", "ball", "vest", "band", "landmine",
         "pedals", "straps", "doortowel", "chair", "banner")
HANDS = {"near": ("near",), "far": ("far",), "both": SIDES}
HAND = {"backpack": "near", "banner": "near"}   # the hand a spec without "hand" means; "both" for the other kinds


def hand_of(item):
    """The hand an item's spec names, or its kind's default (HAND)."""
    return item.get("hand", HAND.get(item["kind"], "both"))


def holders(item, t=0.0, loop=1.0):
    """The sides whose hand holds the item at time t: none for what rests on the body or the floor (a vest, a pack on
    the back, a towel under a foot or a knee, the cranks under the feet, rings round the heels, a dumbbell gripped
    between the feet), a band's or a door towel's ends that are grips, a ball only while no `free` window has it out of
    the hands, and both for what lies on the chest or the hips (the hands hold it there); otherwise the item's hand
    (hand_of)."""
    k = item["kind"]
    if k in ("vest", "towel", "pedals") or (k == "backpack" and item.get("on") == "back") or (
            k == "straps" and item.get("to", "grips") == "heels") or (k == "dumbbell" and item.get("on") == "feet"):
        return ()
    if k in ("band", "doortowel"):
        ends = [e for pair in item["strands"] for e in pair] if k == "band" else item.get("ends", [])
        names = {e["at"] if isinstance(e, dict) else e for e in ends if not isinstance(e, (list, tuple))}
        return tuple(s for s in SIDES if f"grip_{s}" in names or "grips" in names)
    if k == "ball" and any(within(t, w["leave"], w["back"], loop)[0] for w in item.get("free", ())):
        return ()
    if item.get("on") in ("chest", "hips"):
        return SIDES
    return HANDS[hand_of(item)]


def grip(J, side):
    """Where a hand holds a handle: 40 % of the way from the wrist to the fingertips."""
    return rig.lerp(J[f"wrist_{side}"], J[f"hand_tip_{side}"], 0.4)


def grips(J, hand):
    """The grip of one hand, or the midpoint of both."""
    if hand == "both":
        return rig.lerp(grip(J, "near"), grip(J, "far"), 0.5)
    return grip(J, hand)


def hook(J, side):
    """Where the closed fingers hold a handle (a kettlebell's, a bag's strap): HOOK cm from the wrist along the hand."""
    w = J[f"wrist_{side}"]
    return add(w, mul(unit(sub(J[f"hand_tip_{side}"], w)), HOOK))


def fingertip(J, side):
    """The gauntlet's fingertips: FINGERS cm from the wrist along the hand."""
    w = J[f"wrist_{side}"]
    return add(w, mul(unit(sub(J[f"hand_tip_{side}"], w)), FINGERS))


def hooks(J, hand):
    """The hook of one hand, or the midpoint of both."""
    if hand == "both":
        return rig.lerp(hook(J, "near"), hook(J, "far"), 0.5)
    return hook(J, hand)


def flat(v, fallback):
    """A direction laid level (its up part dropped)."""
    return unit((v[0], 0.0, v[2]), fallback)


def bell_handle(handle, bell, axis):
    """A kettlebell's handle as one line of BELL_POINTS (body space): a squared arch (a superellipse, exponent 3)
    from the bell's shoulder on the axis's minus side up its horn, BELL_HORN out at its widest, BELL_HORN_AT from the
    bar, along the bar through the hands and down the other horn to the other shoulder."""
    d = unit(sub(bell, handle))
    xs = BELL_R * math.sin(math.radians(BELL_SHOULDER))              # the shoulders: along the axis, and from the bar
    ys = BELL_DROP - BELL_R * math.cos(math.radians(BELL_SHOULDER))
    c = (xs / BELL_HORN) ** 1.5
    s = math.sqrt(1.0 - c * c)
    low = (ys - BELL_HORN_AT) / s ** (2 / 3)                         # the arch's half height under its widest
    start = math.atan2(-s, c)
    out = []
    for k in range(BELL_POINTS):
        a = start + (math.pi - 2 * start) * (1 - k / (BELL_POINTS - 1))
        ca, sa = math.cos(a), math.sin(a)
        x = math.copysign(abs(ca) ** (2 / 3), ca) * BELL_HORN
        y = BELL_HORN_AT - math.copysign(abs(sa) ** (2 / 3), sa) * (BELL_HORN_AT if sa > 0 else low)
        out.append(add(handle, add(mul(axis, x), mul(d, y))))
    return out


def torso(J):
    """The torso's frame (forward, up, lateral) in body space: up along the spine, lateral across the shoulders."""
    up = unit(sub(J["neck"], J["waist"]))
    lat = sub(J["shoulder_far"], J["shoulder_near"])
    lat = unit(sub(lat, mul(up, dot(lat, up))), (0.0, 0.0, 1.0))
    return cross(up, lat), up, lat


def back_face(J, at):
    """The back's face, behind the spine along the line through BACK_SEAT's two points and square to the torso's
    side: its point level with `at` (a share of the way from the waist to the neck) and its frame (lateral, out of the
    face forward, up the line)."""
    fwd, up, lat = torso(J)
    (z1, b1), (z0, b0) = BACK_SEAT
    top = add(add(J["waist"], mul(up, z1)), mul(fwd, -b1))
    bottom = add(add(J["waist"], mul(up, z0)), mul(fwd, -b0))
    up = unit(sub(top, bottom))
    mid = rig.lerp(J["waist"], J["neck"], at)
    return add(bottom, mul(up, dot(sub(mid, bottom), up))), (lat, cross(up, lat), up)


def sole(J, side):
    """The floor under a foot's arch: the ankle's point 6 cm along the foot, down by the ankle's height."""
    a, t = J[f"ankle_{side}"], J[f"toe_{side}"]
    h = unit(sub(t, a), (1.0, 0.0, 0.0))
    p = add(a, mul(h, 6.0))
    return (p[0], max(0.2, p[1] - BONES["ankle_height"]), p[2])


def ball(J, side):
    """The pedal's spindle under a foot: the ball of the foot (ball_forward along it), just under the sole."""
    a, t = J[f"ankle_{side}"], J[f"toe_{side}"]
    h = unit((t[0] - a[0], t[1] - a[1], 0.0), (1.0, 0.0, 0.0))
    down = (h[1], -h[0], 0.0)
    return add(add(a, mul(h, BONES["ball_forward"])), mul(down, BONES["ankle_height"] - 1.0))


def body_point(J, end):
    """A band's end: a fixed point, a named body point, or a named point with an offset in body space or in the
    torso's frame."""
    if isinstance(end, (list, tuple)):
        return tuple(float(x) for x in end)
    off = (0.0, 0.0, 0.0)
    if isinstance(end, dict) and "torso" in end:
        f, u, l = end["torso"]
        fwd, up, lat = torso(J)
        end, off = end["at"], add(add(mul(fwd, f), mul(up, u)), mul(lat, l))
    elif isinstance(end, dict):
        end, off = end["at"], tuple(end.get("off", off))
    if end == "grips":
        p = grips(J, "both")
    elif end.startswith("grip_"):
        p = grip(J, end[5:])
    elif end.startswith("sole_"):
        p = sole(J, end[5:])
    elif end in J:
        p = J[end]
    else:
        raise ValueError(f"band: unknown end {end!r}")
    return add(p, off)


def rope_turn(item, t, loop):
    """How far a skipping rope's middle has turned at time t (turns; a whole number: straight down, under the
    feet)."""
    return item.get("turns", 1) * t / loop + item.get("phase", 0.0)


def rope_points(item, a, b, t, loop):
    """A skipping rope's ROPE_POINTS points from the grip a to the grip b at time t: each `bulge` (sin^0.6 of its
    place along the rope) of `r` out from the hands' line, turned ROPE_LEAD less the bulge's share of it ahead of
    the middle's angle, so the ends at the hands lead round the turn and the rope bows back from them to its middle,
    the part furthest out, as a turning rope trails its handles; its sides swing out past the hands by up to
    ROPE_FLARE where it is half out, clear of the forearms and the elbows. Where its turn would take it under the
    floor it lies along the floor, as a rope long enough to clear the helm's crest slaps the floor."""
    axis = unit(sub(b, a), (0.0, 0.0, 1.0))
    mid = 2 * math.pi * rope_turn(item, t, loop)
    lead = math.copysign(math.radians(ROPE_LEAD), item.get("turns", 1))
    r = item.get("r", ROPE_R)
    pts = []
    for i in range(ROPE_POINTS):
        s = i / (ROPE_POINTS - 1)
        bulge = math.sin(math.pi * s) ** 0.6
        th = mid + lead * (1.0 - bulge)
        out = unit(rig.square_to((math.sin(th), -math.cos(th), 0.0), axis))   # 0: straight down, then forward, up
        side = math.copysign(4 * bulge * (1.0 - bulge) * ROPE_FLARE, s - 0.5)
        p = add(add(rig.lerp(a, b, s), mul(out, r * bulge)), mul(axis, side))
        pts.append((p[0], max(p[1], ROPE_THICK), p[2]))
    return pts


def towel_tail(top, end, bow=(0.0, 0.0, 0.0), held=False):
    """A door towel's tail from its top (the knot, or where it leaves a bar) to `end` as (points, radii, the grip's
    index in a list, empty for a loose tail): TOWEL_POINTS points sagging by TOWEL_SAG of the span under gravity's
    part square to it, bowed `bow` more at the middle; a held tail ends wrapped in the fist at `end`, TOWEL_WRAP round
    from the grip to TOWEL_THROUGH past it, closing there."""
    span = sub(end, top)
    along = unit(span, (0.0, -1.0, 0.0))
    sag = mul(rig.square_to((0.0, -1.0, 0.0), along), TOWEL_SAG * rig.norm(span))
    off = add(sag, bow)
    pts = [add(rig.lerp(top, end, k / (TOWEL_POINTS - 1)),
               mul(off, 4 * k * (TOWEL_POINTS - 1 - k) / (TOWEL_POINTS - 1) ** 2)) for k in range(TOWEL_POINTS)]
    if not held:
        return pts, [TOWEL_ROPE_R] * len(pts), []
    way = unit(sub(pts[-1], pts[-2]), along)
    pts += [add(end, mul(way, TOWEL_THROUGH)), add(end, mul(way, TOWEL_THROUGH + TOWEL_WRAP / 2))]
    return pts, [TOWEL_ROPE_R] * (TOWEL_POINTS - 1) + [TOWEL_WRAP, TOWEL_WRAP, TOWEL_TIP], [TOWEL_POINTS - 1]


def door_top(top, tips):
    """Where each tail of a towel hung over a door's top leaves its top edge: side by side along the edge about
    `top`, a towel's round apart, in the order of their tips across (l)."""
    n = len(tips)
    starts = [None] * n
    for rank, i in enumerate(sorted(range(n), key=lambda i: tips[i][2])):
        starts[i] = (top[0], top[1], top[2] + (rank - (n - 1) / 2) * 2 * TOWEL_ROPE_R)
    return starts


def towel_over(over, ends):
    """A towel hung over a bar whose axis passes `over` (along l), its two ends in the hands: one line (points, radii,
    the grips' indices) from the first end's hanging tip up through its fist, up the bar's front, over its top
    flattened to TOWEL_FLAT of its round, from where each tail leaves it, down its back and through the other fist to
    its tip; and each tail's span (where it leaves the bar to its grip)."""
    if len(ends) != 2:
        raise ValueError("doortowel: a towel over a bar takes its two ends (\"ends\"), down the bar's front and back")
    g = rig.lerp(ends[0], ends[1], 0.5)
    flat = TOWEL_ROPE_R * TOWEL_FLAT
    rb = props.BAR_R + flat
    v = (g[0] - over[0], g[1] - over[1])
    d = max(math.hypot(*v), rb + TOWEL_ROPE_R)                 # the fists under the bar, never in it
    toward = math.atan2(v[1], v[0])
    leave = math.acos(rb / d)                                   # where the tails leave the bar, either side of `toward`
    lo, hi = toward + leave, toward - leave + 2 * math.pi      # round the far side of the bar, over its top
    if math.cos(lo) < math.cos(hi):                            # the first tail down the front (+f)
        lo, hi = hi, lo
    layer = [ends[n][2] - g[2] for n in (0, 1)]                # each layer's lateral offset, as its fist holds it
    n = TOWEL_ARC
    arc = []
    for j in range(n):
        a = lo + (hi - lo) * j / (n - 1)
        arc.append((over[0] + rb * math.cos(a), over[1] + rb * math.sin(a), over[2] + layer[0] + (layer[1] - layer[0])
                    * j / (n - 1)))
    apart = unit(sub(arc[0], arc[-1]))
    front, fr, (gf,) = towel_tail(arc[0], ends[0], mul(apart, TOWEL_BOW), held=True)
    back, br, (gb,) = towel_tail(arc[-1], ends[1], mul(apart, -TOWEL_BOW), held=True)
    fr[0] = br[0] = flat                                       # still on the bar where each tail leaves it
    pts = front[::-1] + arc[1:-1] + back
    radii = fr[::-1] + [flat] * (n - 2) + br
    grips = [len(front) - 1 - gf, len(front) + n - 2 + gb]
    return [(pts, radii, grips)], [(arc[0], ends[0]), (arc[-1], ends[1])]


def chair_boxes(back):
    """The light chair as oriented boxes in its own frame (x across the seat, y along it, z up; origin the seat's
    centre): the seat, four legs and the back on the edge at y = back * SEAT_HALF, props.py's measures."""
    s, t, w = props.SEAT_HALF, props.SEAT_T, props.LEG_W
    leg = (props.SEAT_H - t) / 2
    boxes = [((0.0, 0.0, 0.0), (s, s, t / 2))]
    for x in (-1.0, 1.0):
        for y in (-1.0, 1.0):
            boxes.append(((x * (s - w / 2), y * (s - w / 2), -t / 2 - leg), (w / 2, w / 2, leg)))
    boxes.append(((0.0, back * (s - props.BACK_T / 2), t / 2 + props.BACK_H / 2),
                  (s, props.BACK_T / 2, props.BACK_H / 2)))
    return boxes


def banner_cloth(pose, x, z):
    """A point of a standard's cloth (body space, flat: the renderer's ripple is a few cm): `x` cm from the pole along
    its fly, `z` cm down from the crossbar along the pole."""
    return add(add(pose["bar"], mul(pose["fly"], x)), mul(pose["axis"], -z))


def banner_points(pose):
    """The points of a placed standard that bound it: its butt, the finial's tip, the crossbar's end and the cloth's
    corners (each tail's tip and the notch's top)."""
    w, d, n = pose["width"], pose["drop"], pose["notch"]
    tip = add(pose["top"], mul(pose["axis"], BANNER_FINIAL))
    return [pose["butt"], tip, banner_cloth(pose, w + 3.0, 0.0), banner_cloth(pose, 0.0, d),
            banner_cloth(pose, w, d), banner_cloth(pose, w / 2, d - n)]


def box_corners(center, axes, half):
    """The eight corners of an oriented box (body space)."""
    out = []
    for sx in (-1.0, 1.0):
        for sy in (-1.0, 1.0):
            for sz in (-1.0, 1.0):
                p = center
                for ax, h, sg in zip(axes, half, (sx, sy, sz)):
                    p = add(p, mul(ax, sg * h))
                out.append(p)
    return out


def ease(s, kind):
    """A flight's progress: "in" speeds up like a fall, anything else is even."""
    s = max(0.0, min(1.0, s))
    return s * s if kind == "in" else s


def within(t, a, b, loop):
    """Whether t lies in the window from a to b of a loop (a window may wrap past the loop's end), and how far in."""
    span = (b - a) % loop
    d = (t - a) % loop
    return d <= span, d


def free_point(item, held, t, loop):
    """Where a ball is at time t: in the hands (`held`) or on one of its `free` windows' flights and rests."""
    for w in item.get("free", ()):
        inside, d = within(t, w["leave"], w["back"], loop)
        if not inside:
            continue
        at = tuple(float(x) for x in w["at"])
        out = (w["land"] - w["leave"]) % loop
        rest = (w["lift"] - w["leave"]) % loop
        arc = w.get("arc", 0.0)
        if d < out:                                   # flying from the hands (where they let go) to the rest point
            s = ease(d / out, w.get("ease", "linear"))
            p = rig.lerp(held, at, s)
        elif d < rest:
            return at
        else:                                         # flying back into the hands
            back = (w["back"] - w["lift"]) % loop or 1e-6
            s = (d - rest) / back
            p = rig.lerp(at, held, s)
        return add(p, (0.0, arc * 4 * s * (1 - s), 0.0))
    return held


def place(item, J, t=0.0, loop=1.0):
    k = item["kind"]
    if k == "barbell":
        a, b = grip(J, "near"), grip(J, "far")
        mid, axis = rig.lerp(a, b, 0.5), unit(sub(b, a), (0.0, 0.0, 1.0))
        return {"kind": k, "center": mid, "axis": axis, "low": mid[1] - PLATE_R}
    if k == "kettlebell" and item.get("on") == "hips":
        fwd, up, lat = torso(J)
        bell = add(add(J["pelvis"], mul(fwd, rig.CLEARANCE["pelvis"] + BELL_R)), mul(up, HIPS_UP))
        g = add(bell, mul(fwd, BELL_DROP))
        line = bell_handle(g, bell, lat)
        return {"kind": k, "handle": g, "bell": bell, "axis": lat, "line": line,
                "low": min(bell[1] - BELL_R, min(p[1] for p in line) - BELL_HANDLE_R)}
    if k == "kettlebell":
        g = hooks(J, hand_of(item))
        hold = item.get("hold", "hang")
        if hold == "down":
            d = (0.0, -1.0, 0.0)
        elif hold == "up":
            d = (0.0, 1.0, 0.0)
        else:
            sh = rig.lerp(J["shoulder_near"], J["shoulder_far"], 0.5)
            d = unit(sub(g, sh), (0.0, -1.0, 0.0))
        # the bell clears the hands along its hang: a hand that points away from it (a goblet's, up the horns) holds
        # the horns under the bar, which stays BELL_DROP over the bell
        hands = HANDS[hand_of(item)]
        reach = max(dot(sub(p, g), d) for s in hands for p in (J[f"wrist_{s}"], fingertip(J, s)))
        bell = add(g, mul(d, max(BELL_DROP, reach + WINDOW + BELL_R)))
        g = add(bell, mul(d, -BELL_DROP))
        if hand_of(item) == "both":
            axis = unit(sub(hook(J, "far"), hook(J, "near")), torso(J)[2])
        else:
            axis = torso(J)[2]
        axis = unit(sub(axis, mul(d, dot(axis, d))), torso(J)[2])     # the handle square to the bell's hang
        line = bell_handle(g, bell, axis)
        return {"kind": k, "handle": g, "bell": bell, "axis": axis, "line": line,
                "low": min(bell[1] - BELL_R, min(p[1] for p in line) - BELL_HANDLE_R)}
    if k == "backpack":
        on = item.get("on")
        fwd, up, lat = torso(J)
        if on not in (None, "back", "chest", "hips"):
            raise ValueError(f"backpack: unknown \"on\" {on!r} (back, chest or hips; none for a pack in hand)")
        if on == "back":
            face, axes = back_face(J, 0.45)            # the pack's width across, its depth out of the back's face
            c = add(face, mul(axes[1], -PACK_D / 2))
        elif on == "chest":
            c = add(rig.lerp(J["waist"], J["neck"], CHEST_AT), mul(fwd, CHEST_FRONT + PACK_D / 2))
            axes = (up, fwd, lat)                      # lying across the breastplate: clear of the chin and thighs
        elif on == "hips":
            c = add(add(J["pelvis"], mul(fwd, HIPS_FRONT + PACK_D / 2)), mul(up, PACK_HIPS))
            axes = (up, fwd, lat)                      # across the hips, up the belly: the thighs clear it
        else:
            g = hooks(J, hand_of(item))
            up, lat = (0.0, 1.0, 0.0), flat(lat, (0.0, 0.0, 1.0))
            top = add(g, mul(up, -PACK_DROP))
            c = add(top, mul(up, -PACK_H / 2))
            if hand_of(item) == "both":                # held in front by its top handle: broad face to the body
                axes = (lat, cross(up, lat), up)
            else:                                      # hanging by the side: broad face along it
                axes = (cross(up, lat), lat, up)
        half = (PACK_W / 2, PACK_D / 2, PACK_H / 2)
        low = c[1] - sum(abs(ax[1]) * h for ax, h in zip(axes, half))
        pose = {"kind": k, "on": on, "center": c, "axes": axes, "low": low}
        if not on:
            pose.update(handle=g, top=top)
        return pose
    if k == "rope":
        pts = rope_points(item, grip(J, "near"), grip(J, "far"), t, loop)
        return {"kind": k, "points": pts, "low": min(p[1] for p in pts)}
    if k == "wheel":
        g = rig.lerp(grip(J, "near"), grip(J, "far"), 0.5)
        return {"kind": k, "center": g, "low": g[1] - WHEEL_R}
    if k == "towel" and item.get("under"):
        p = J[item["under"]]
        return {"kind": k, "center": (p[0], TOWEL_T / 2, p[2]), "wide": False, "low": 0.0}
    if k == "towel":
        sides = SIDES if item.get("feet", "both") == "both" else (item["feet"],)
        heels = [J[f"{'toe' if item.get('at') == 'toes' else 'ankle'}_{s}"] for s in sides]
        c = (sum(h[0] for h in heels) / len(heels), TOWEL_T / 2, sum(h[2] for h in heels) / len(heels))
        return {"kind": k, "center": c, "wide": len(sides) == 2, "low": 0.0}
    if k == "dumbbell":
        fwd, up, lat = torso(J)
        bells = []
        if item.get("grip") == "goblet" and item.get("on") == "hips":
            c = add(add(J["pelvis"], mul(fwd, rig.CLEARANCE["pelvis"] + DB_HEAD_R)), mul(up, HIPS_UP))
            bells.append({"center": c, "axis": lat})
        elif item.get("grip") == "goblet" and item.get("on") == "feet":
            a = rig.lerp(J["ankle_near"], J["ankle_far"], 0.5)
            shin = unit(sub(a, rig.lerp(J["knee_near"], J["knee_far"], 0.5)), (0.0, -1.0, 0.0))
            bells.append({"center": add(a, mul(shin, FEET_PAST)), "axis": shin})
        elif item.get("grip") == "goblet":
            c = add(grips(J, "both"), mul(up, -(DB_HALF - 4.0)))   # the palms cup the top head
            bells.append({"center": c, "axis": up})
        else:
            for s in HANDS[hand_of(item)]:
                if item.get("grip", "neutral") == "across":
                    axis = lat
                elif item.get("grip") == "palms_down":
                    h = unit(sub(J[f"hand_tip_{s}"], J[f"wrist_{s}"]))
                    axis = unit(cross(h, (0.0, 1.0, 0.0)), fwd)
                else:
                    h = unit(sub(J[f"hand_tip_{s}"], J[f"wrist_{s}"]))
                    axis = unit(cross(lat, h), fwd)
                bells.append({"center": grip(J, s), "axis": axis})
        low = min(b["center"][1] - abs(b["axis"][1]) * DB_HALF - DB_HEAD_R for b in bells)
        return {"kind": k, "bells": bells, "low": low}
    if k == "ball":
        r = item.get("r", BALL_R)
        held = add(grips(J, "both"), tuple(item.get("off", (0.0, 0.0, 0.0))))
        c = free_point(item, held, t, loop)
        return {"kind": k, "center": c, "r": r, "low": c[1] - r}
    if k == "vest":
        fwd, up, lat = torso(J)
        front = add(rig.lerp(J["waist"], J["neck"], VEST_AT), mul(fwd, CHEST_FRONT + VEST_T / 2))
        face, back_axes = back_face(J, VEST_AT)
        back = add(face, mul(back_axes[1], -VEST_T / 2))
        return {"kind": k, "front": front, "back": back, "axes": (lat, fwd, up), "back_axes": back_axes,
                "low": min(front[1], back[1]) - VEST_H / 2}
    if k == "band":
        strands = [[body_point(J, a), body_point(J, b)] for a, b in item["strands"]]
        held = [[n for n, end in enumerate(pair) if not isinstance(end, list)] for pair in item["strands"]]
        return {"kind": k, "strands": strands, "held": held, "low": min(p[1] for s in strands for p in s)}
    if k == "landmine":
        pivot = tuple(float(x) for x in item["pivot"])
        g = grips(J, hand_of(item))
        axis = unit(sub(g, pivot))
        end = add(pivot, mul(axis, item["len"] + LANDMINE_OVER))
        plate = add(g, mul(axis, -LANDMINE_PLATE_AT))
        low = min(pivot[1], plate[1] - LANDMINE_PLATE_R * math.sqrt(max(0.0, 1 - axis[1] ** 2)))
        return {"kind": k, "pivot": pivot, "end": end, "plate": plate, "axis": axis, "len": item["len"],
                "reach": rig.dist(g, pivot), "low": low}
    if k == "pedals":
        f, u = item["at"]
        cranks = []
        for s in SIDES:
            cranks.append({"side": s, "hub": (float(f), float(u), rig.SIGN[s] * 6.0), "pedal": ball(J, s)})
        return {"kind": k, "cranks": cranks, "r": item.get("r", 17.0),
                "low": min(c["pedal"][1] for c in cranks) - 2.0}
    if k == "straps":
        f, u = item["at"]
        half = item.get("half", STRAP_HALF)
        lines = []
        for s in SIDES:
            if item.get("to", "grips") == "heels":
                end = add(J[f"ankle_{s}"], (-4.0, -2.0, 0.0))
            else:
                end = grip(J, s)
            lines.append({"side": s, "anchor": (float(f), float(u), rig.SIGN[s] * half), "ring": end})
        return {"kind": k, "lines": lines, "low": min(ln["ring"][1] for ln in lines) - RING_R}
    if k == "doortowel":
        ends = [body_point(J, end) for end in item.get("ends", ())]
        if "over" in item:
            if "knot" in item or "loose" in item or "top" in item:
                raise ValueError("doortowel: a towel over a bar (\"over\") has no \"knot\", \"top\" or \"loose\" tail")
            over = tuple(float(x) for x in item["over"])
            lines, spans = towel_over(over, ends)
            pose = {"kind": k, "over": over}
        elif "top" in item:
            if "knot" in item:
                raise ValueError("doortowel: a towel over a door's top (\"top\") has its knot out of sight: no \"knot\"")
            top = tuple(float(x) for x in item["top"])
            loose = [tuple(float(x) for x in item["loose"])] if item.get("loose") else []
            starts = door_top(top, ends + loose)
            lines = [towel_tail(s, g, held=True) for s, g in zip(starts, ends)]
            lines += [towel_tail(s, e) for s, e in zip(starts[len(ends):], loose)]
            spans = list(zip(starts, ends))
            pose = {"kind": k, "top": top}
        else:
            knot = tuple(float(x) for x in item["knot"])
            lines = [towel_tail(knot, g, held=True) for g in ends]
            spans = [(knot, g) for g in ends]
            if item.get("loose"):
                lines.append(towel_tail(knot, tuple(float(x) for x in item["loose"])))
            pose = {"kind": k, "knot": knot}
        pose.update(lines=[ln[0] for ln in lines], radii=[ln[1] for ln in lines], held=[ln[2] for ln in lines],
                    spans=spans, low=min(p[1] - r for pts, rs, _ in lines for p, r in zip(pts, rs)))
        return pose
    if k == "chair":
        # held upright in the body's own frame (f forward, u up, l lateral), whatever the trunk does
        fwd, up, lat = (1.0, 0.0, 0.0), (0.0, 1.0, 0.0), (0.0, 0.0, 1.0)
        hand = hand_of(item)
        drop = mul(up, -(props.SEAT_T / 2 + CHAIR_GRIP))
        if hand == "both":
            a, b = grip(J, "near"), grip(J, "far")
            center = add(add(rig.lerp(a, b, 0.5), mul(fwd, item.get("off", 0.0))), drop)
            back, fit = 1.0, abs(b[2] - a[2]) / 2 - props.SEAT_HALF
        else:
            g = grip(J, hand)
            center = add(add(add(g, mul(lat, rig.SIGN[hand] * props.SEAT_HALF)), mul(fwd, item.get("off", 0.0))), drop)
            back, fit = -1.0, None
        axes = (lat, fwd, up)
        boxes = []
        for c, half in chair_boxes(back):
            boxes.append((add(add(add(center, mul(lat, c[0])), mul(fwd, c[1])), mul(up, c[2])), half))
        low = min(p[1] for c, half in boxes for p in box_corners(c, axes, half))
        return {"kind": k, "center": center, "axes": axes, "back": back, "boxes": boxes, "fit": fit, "low": low,
                "set": item.get("set", False)}
    if k == "banner":
        hand = hand_of(item)
        if hand == "both":
            a, b = grip(J, "near"), grip(J, "far")
            lo, hi = (a, b) if a[1] <= b[1] else (b, a)
            axis = unit(sub(hi, lo))
            butt = add(lo, mul(axis, -item.get("low", BANNER_LOW)))
        elif hand in SIDES and "foot" in item:
            butt = (float(item["foot"][0]), 0.0, float(item["foot"][1]))
            axis = unit(sub(grip(J, hand), butt))
        else:
            raise ValueError("banner: \"hand\": \"near\" or \"far\" with its `foot` [f, l] on the floor, or \"both\"")
        top = add(butt, mul(axis, item.get("len", BANNER_LEN)))
        ahead = (float(item.get("fly", 1)), 0.0, 0.0)          # forward, or behind with -1, square to the pole
        fly = unit(sub(ahead, mul(axis, dot(ahead, axis))), ahead)
        pose = {"kind": k, "butt": butt, "top": top, "axis": axis, "fly": fly, "bar": add(top, mul(axis, -BANNER_CAP)),
                "width": item.get("width", BANNER_WIDTH), "drop": item.get("drop", BANNER_DROP),
                "notch": item.get("notch", BANNER_NOTCH)}
        pose["low"] = min(p[1] for p in banner_points(pose))
        return pose
    raise ValueError(f"unknown item kind {k!r} (known: {', '.join(KINDS)})")


def shape(pose):
    """A placed item's solid for the checks, in body space (cm): {"spheres": [(centre, r)], "boxes": [(centre, axes,
    half)], "lines": [(points, radii, held)]}, `held` the indices of a line's points a hand holds or a loop ties to the
    body (checks.py leaves the stretch next to them untested). What the hands hold away from the body is a
    sphere (a dumbbell's heads, a bell, a ball) or a box (a backpack, a chair's parts, along `axes`); what bends or
    swings is a line (a skipping rope, a band's strands, a door towel); the rest (a bar, a wheel on the floor, the
    cranks, the straps, a standard: checks.check_banner) has none here."""
    k = pose["kind"]
    out = {"spheres": [], "boxes": [], "lines": []}
    if k == "dumbbell":
        out["spheres"] = [(add(b["center"], mul(b["axis"], s * DB_HEAD_AT)), DB_HEAD_R)
                          for b in pose["bells"] for s in (-1.0, 1.0)]
    elif k == "kettlebell":
        out["spheres"] = [(pose["bell"], BELL_R)]
    elif k == "ball":
        out["spheres"] = [(pose["center"], pose["r"])]
    elif k == "backpack":
        out["boxes"] = [(pose["center"], pose["axes"], (PACK_W / 2, PACK_D / 2, PACK_H / 2))]
    elif k == "vest":
        out["boxes"] = [(pose[p], pose[a], (VEST_W / 2, VEST_T / 2, VEST_H / 2))
                        for p, a in (("front", "axes"), ("back", "back_axes"))]
    elif k == "chair":
        out["boxes"] = [(c, pose["axes"], half) for c, half in pose["boxes"]]
    elif k == "rope":
        n = len(pose["points"])
        out["lines"] = [(pose["points"], [ROPE_THICK] * n, [0, n - 1])]
    elif k == "band":
        out["lines"] = [(s, [BAND_R] * 2, held) for s, held in zip(pose["strands"], pose["held"])]
    elif k == "doortowel":
        out["lines"] = list(zip(pose["lines"], pose["radii"], pose["held"]))
    return out


def check(pose):
    """Problems of one placed item: in the floor (more than 0.5 cm), a wheel off it, a crank arm that cannot reach its
    pedal, the hands off a chair's seat."""
    if pose["kind"] == "wheel":
        if abs(pose["low"]) > 1.5:
            return [f"wheel: {pose['low']:.1f} cm off the floor (the grips must sit {WHEEL_R:.0f} cm up)"]
        return []
    out = []
    if pose["kind"] == "landmine" and abs(pose["reach"] - pose["len"]) > 2.5:
        out.append(f"landmine: the grip is {pose['reach'] - pose['len']:.1f} cm off the bar's end (the hands stay "
                   "`len` from the pivot)")
    if pose["kind"] == "pedals":
        for c in pose["cranks"]:
            d = math.hypot(c["pedal"][0] - c["hub"][0], c["pedal"][1] - c["hub"][1])
            if abs(d - pose["r"]) > 2.5:
                out.append(f"pedals: the {c['side']} pedal is {d - pose['r']:.1f} cm off its crank's circle")
    if pose["kind"] == "chair" and pose["fit"] is not None and abs(pose["fit"]) > CHAIR_FIT:
        out.append(f"chair: the hands are {pose['fit']:+.1f} cm off the seat's sides (they hold its sides, "
                   f"{2 * props.SEAT_HALF:.0f} cm apart)")
    if pose["low"] < -0.5:
        out.append(f"{pose['kind']}: {-pose['low']:.1f} cm into the floor")
    return out
