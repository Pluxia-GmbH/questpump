"""The props a move stands on, pushes against or hangs from, as solid boxes and rods in body space (pure Python;
docs/plan.md slice 10). One definition serves both sides: the checks test each joint's armour against the boxes, and
the Blender prop pass (`blender/scene.py`) draws the same boxes, so what the checks allow is what the frames show.

A prop in a motion file (cm; f forward, u up, l lateral with the far side positive):

  {"kind": "wall", "f": 60}                        a wall whose face stands at f, the wall beyond it (+f), 182 cm
                                                   high (or "u": a wall angel's hands reach 215)
  {"kind": "wall", "f": -30, "behind": true}       a wall behind the figure (its face toward +f): the wall sits
  {"kind": "chair", "f": -45, "back": "-f"}        a chair centred at f (and l, default 0), seat 46 cm high, the
                                                   backrest on the side named (+f, -f, +l, -l)
  {"kind": "desk", "f": 40}                        a desk whose front edge is at f, top 74 cm (or "top"), running +f
  {"kind": "desk", "l": 40}                        a desk beside the figure, its edge along f at l, running away
  {"kind": "table", "f0": -40, "f1": 60}           a dining table's top between f0 and f1, 72 cm high (or "top")
  {"kind": "doorframe", "f": 42}                   a door jamb whose edge faces the figure at f, the wall running +l
                                                   (or "wall": "-l", the doorway on the far side)
  {"kind": "door", "f": 60}                        a closed door in a wall, its face at f (or "l": beside the figure,
                                                   the face at l, the wall beyond it), a band's anchor point on it:
                                                   the leaf set back in a steel frame, with a lever handle near its -l
                                                   edge (-f beside the figure); it opens away from the figure, so no
                                                   hinge shows on this face
  {"kind": "bar", "f": 8, "u": 222}                a fixed bar along l at (f, u), hung from two posts that run up
                                                   out of the frame; "ground": true stands it on two posts from the
                                                   ground instead, the space above it free (an outdoor bar: a
                                                   muscle-up rises over it); the posts at l -60 and 60, or at
                                                   -"half" and "half" (a wide grip: an archer's elbows clear them)
  {"kind": "pole", "f": 30, "top": 230}            an upright steel post fixed in the ground at f (and l, default 0),
                                                   up to `top` (a human flag's pole outdoors)
  {"kind": "rings", "f": 30, "u": 112}             two straps from the ceiling to handles at (f, u), 44 cm apart, or
                                                   at -"half" and "half" (a wider hang: a pack worn on the chest passes
                                                   between the handles)
  {"kind": "bench", "f0": -70, "f1": -18, "u": 44} a low padded bench (a sofa's front edge): top at u, legs at its ends
  {"kind": "step", "f0": -15, "f1": 25, "u": 20}   a wooden step or box: top at u between f0 and f1, across l0..l1
                                                   (default -40..40)
  {"kind": "stairs", "f": 30}                      a flight rising toward +f: the first riser's face at f, "rise" 18
                                                   and "run" 28 cm, "steps" [first, last] (default [1, 4]: step k's
                                                   tread at k * rise between f + (k - 1) * run and f + k * run; a
                                                   step below 1 lies under the floor's level, for an endless flight),
                                                   "half" 50 cm wide each side, "rail" "near" or "far" (a handrail 90 cm
                                                   above the nosings), "frame" [first, last] the steps the camera keeps
                                                   (their treads, one rise deep), "dir": "-f" a flight rising behind the
                                                   figure (a step-down off the bottom stair)
  {"kind": "bike", "f": 0, "u": 30, "seat": [-22, 92], "bars": [30, 102]}   a stationary bike: the bottom bracket at
                                                   (f, u), the saddle's top and the handlebar's centre where named, a
                                                   flywheel ahead of the cranks (the cranks are the "pedals" item)
  {"kind": "pad", "f0": -60, "f1": 60}             a walking pad's deck, its belt 12 cm high (or "u"), 26 cm each side;
                                                   "belt": the belt's travel over one loop (cm, negative toward -f), its
                                                   slats running with a planted foot; "rail": true, a front bar 100 cm up
  {"kind": "roller", "f": 0}                       a foam roller lying on the floor across the body (along l), or
                                                   "along": "f" under the spine; "l", "len" 90 and "r" 7.5 cm
  {"kind": "mat", "f0": -90, "f1": 90}             an exercise mat, 1 cm thick, across l0..l1 (default -32..32)
  {"kind": "runner", "f0": -200, "f1": 200}        a hallway runner, 1 cm thick, across l0..l1 (default -35..35), its
                                                   stripes carried by "belt" (as the pad's) so a walk in place walks
                                                   along it; the camera keeps "frame" [f0, f1] of it (default -60..60)

Any prop may carry "scroll": [df, du], its move over one loop (cm): an endless flight of stairs slides down and back
under a climber by two steps a loop. A "belt" moves only its slats or stripes, wrapping round its length. `parts(p,
at)` and `solids(p, at)` give the prop at `at` of the loop (0..1), in the same part order at every `at`.

Each kind gives `solids(prop)`, the boxes the checks see, and `parts(prop)`, the boxes and rods the renderer draws with
a material name ("stone", "wood", "gold", "iron", "leather", "mat"). Boxes are (f0, f1, u0, u1, l0, l1); a rod is
(a, b, radius) from point a to point b.
"""
import math

WALL_THICK, WALL_HALF, WALL_H = 24.0, 110.0, 182.0
SEAT_H, SEAT_HALF, SEAT_T, LEG_W, BACK_H, BACK_T = 46.0, 22.0, 5.0, 4.0, 44.0, 4.0
DESK_TOP, DESK_T, DESK_DEPTH, DESK_HALF = 74.0, 4.0, 70.0, 70.0
TABLE_TOP, TABLE_HALF = 72.0, 60.0
JAMB_T, JAMB_W, DOOR_H = 12.0, 10.0, 205.0
DOOR_HALF, DOOR_PROUD = 45.0, 0.6
# A door reads as one at 192 px by its frame and its handle (2026-10-26, David: "show the object better"). Every door
# here opens away from the figure, and hinges show only on the side a door swings toward, so this face has none (two
# hinge leaves stood on it until 2026-10-28, which read as a door that opens toward the player). A steel frame
# FRAME_W wide runs round its sides and top, its face FRAME_OUT in front of the leaf, which sits back in it as a door
# does on the side it opens away from (iron beside the wood leaf: a wood or leather casing hardly showed against it at
# 192 px, 2026-10-28); a lever handle HANDLE_IN from the -l edge at HANDLE_U, its rose ROSE_H high, its lever standing
# HANDLE_OUT off the leaf and HANDLE_LONG long toward the leaf's middle
FRAME_W, FRAME_OUT = 5.0, 3.0
HANDLE_U, HANDLE_IN, ROSE_W, ROSE_H, ROSE_T = 105.0, 7.0, 4.0, 18.0, 0.8
HANDLE_OUT, HANDLE_LONG, HANDLE_R = 6.0, 12.0, 1.1
BAR_R, BAR_HALF = 1.6, 60.0
POLE_R = 3.0
RING_HALF, RING_R, CEILING = 22.0, 9.0, 250.0
BENCH_HALF = 70.0
STEP_HALF = 40.0
RISE, RUN, STAIR_HALF, RAIL_H, RAIL_R = 18.0, 28.0, 50.0, 90.0, 2.2
PAD_U, PAD_HALF, PAD_RAIL_U = 12.0, 26.0, 100.0
ROLLER_LEN, ROLLER_R = 90.0, 7.5
MAT_T, MAT_HALF, RUNNER_HALF, RUNNER_FRAME = 1.0, 32.0, 35.0, (-60.0, 60.0)
SLAT_W, SLAT_SPACING = 1.5, 12.0
KINDS = ("wall", "chair", "desk", "table", "doorframe", "door", "bar", "pole", "rings", "bench", "step", "stairs",
         "bike", "pad", "roller", "mat", "runner")


def _legs(f0, f1, l0, l1, top, w=LEG_W, mat="wood"):
    return [((a, a + w, 0.0, top, b, b + w), mat) for a in (f0, f1 - w) for b in (l0, l1 - w)]


def _door_parts():
    """A door's frame and lever handle in the door's own place, as ([(box, material)], [(rod, material)]): a box is
    (out0, out1, u0, u1, a0, a1) and a rod's ends are (out, u, a), `out` cm out of the wall's face toward the figure and
    `a` along the face, the leaf between -DOOR_HALF and +DOOR_HALF, its handle near -DOOR_HALF."""
    h, w, out = DOOR_HALF, FRAME_W, DOOR_PROUD + FRAME_OUT
    boxes = [((0.0, out, 0.0, DOOR_H + w, a0, a1), "iron") for a0, a1 in ((-h - w, -h), (h, h + w))]   # the sides
    boxes.append(((0.0, out, DOOR_H, DOOR_H + w, -h, h), "iron"))                                     # the head
    a, stem = -h + HANDLE_IN, DOOR_PROUD + HANDLE_OUT
    boxes.append(((DOOR_PROUD, DOOR_PROUD + ROSE_T, HANDLE_U - ROSE_H / 2, HANDLE_U + ROSE_H / 2, a - ROSE_W / 2,
                   a + ROSE_W / 2), "iron"))                                                 # the rose
    rods = [(((DOOR_PROUD + ROSE_T, HANDLE_U, a), (stem, HANDLE_U, a), HANDLE_R), "iron"),     # the stem
            (((stem, HANDLE_U, a), (stem, HANDLE_U - 1.0, a + HANDLE_LONG), HANDLE_R), "iron")]   # the lever
    return boxes, rods


def _across(f, u, l0, l1, r):
    """A rod along l at (f, u)."""
    return ((f, u, l0), (f, u, l1), r)


def _slats(p, at, f0, f1, u, half, mat):
    """The stripes a "belt" carries across a deck's top, spaced evenly and wrapping round its length, so the loop
    ends where it starts (the spacing divides the belt's travel)."""
    travel = p.get("belt", 0.0)
    span = f1 - f0 - SLAT_W
    n = max(1, round(abs(travel) / SLAT_SPACING)) if travel else 1
    step = abs(travel) / n if travel else SLAT_SPACING
    count = max(1, int(span // step))
    span = count * step
    return [((f0 + (i * step + at * travel) % span, f0 + (i * step + at * travel) % span + SLAT_W, u - 0.3, u + 0.1,
              -half, half), mat) for i in range(count)]


def _stairs(p):
    rise, run, half = p.get("rise", RISE), p.get("run", RUN), p.get("half", STAIR_HALF)
    k0, k1 = p.get("steps", (1, 4))
    f = p["f"]
    base = min(0.0, (k0 - 1) * rise) - (10.0 if k0 < 1 else 0.0)
    boxes = [((f + (k - 1) * run, f + k * run, base, k * rise, -half, half), "wood") for k in range(k0, k1 + 1)]
    rods = []
    side = p.get("rail")
    if side:
        l = (half + 4.0) * (1.0 if side == "far" else -1.0)
        a = (f + (k0 - 1) * run, k0 * rise + RAIL_H, l)
        b = (f + k1 * run, (k1 + 1) * rise + RAIL_H, l)
        rods.append(((a, b, RAIL_R), "iron"))
        if k0 >= 1:                                         # a flight that starts on the floor: posts at its ends
            for k in (k0, k1):
                n = f + (k - 1) * run + run / 2
                top = k * rise + RAIL_H + rise / 2
                rods.append((((n, k * rise, l), (n, top, l), RAIL_R), "iron"))
    if p.get("dir") == "-f":                                # rising behind the figure: the flight mirrored about f
        boxes = [((2 * f - x1, 2 * f - x0, u0, u1, l0, l1), m) for (x0, x1, u0, u1, l0, l1), m in boxes]
        rods = [(((2 * f - a[0], a[1], a[2]), (2 * f - b[0], b[1], b[2]), r), m) for (a, b, r), m in rods]
    return boxes, rods


def _bike(p):
    f, u = p["f"], p["u"]
    sf, su = p.get("seat", (f - 22.0, 92.0))
    bf, bu = p.get("bars", (f + 30.0, 102.0))
    boxes = [
        ((f - 50.0, f - 42.0, 0.0, 5.0, -28.0, 28.0), "iron"),          # the rear foot
        ((f + 48.0, f + 56.0, 0.0, 5.0, -28.0, 28.0), "iron"),          # the front foot
        ((f - 46.0, f + 52.0, 5.0, 10.0, -3.0, 3.0), "iron"),           # the beam along the floor
        ((sf - 13.0, sf - 2.0, su - 6.0, su, -9.0, 9.0), "leather"),    # the saddle's seat
        ((sf - 2.0, sf + 11.0, su - 5.0, su, -3.5, 3.5), "leather"),    # its nose, between the thighs
    ]
    rods = [
        (((f - 30.0, 10.0, 0.0), (sf, su - 6.0, 0.0), 2.6), "iron"),     # the seat post
        (((f - 4.0, 10.0, 0.0), (f, u, 0.0), 3.0), "iron"),              # the bracket's stem
        (((f + 34.0, 10.0, 0.0), (bf - 4.0, bu - 4.0, 0.0), 3.0), "iron"),   # the front post
        (_across(bf, bu, -22.0, 22.0, 1.8), "leather"),                  # the handlebar
        (((f + 30.0, 38.0, -2.5), (f + 30.0, 38.0, 2.5), 24.0), "iron"),     # the flywheel
    ]
    return boxes, rods


def parts(p, at=0.0):
    """[(box, material)] and [(rod, material)] the renderer draws, at `at` of the loop for a scrolling prop."""
    k = p["kind"]
    boxes, rods = [], []
    if k == "wall":
        f = p["f"]
        f0, f1 = (f - WALL_THICK, f) if p.get("behind") else (f, f + WALL_THICK)
        boxes.append(((f0, f1, 0.0, p.get("u", WALL_H), -WALL_HALF, WALL_HALF), "stone"))
    elif k == "chair":
        f, l = p["f"], p.get("l", 0.0)
        f0, f1, l0, l1 = f - SEAT_HALF, f + SEAT_HALF, l - SEAT_HALF, l + SEAT_HALF
        boxes.append(((f0, f1, SEAT_H - SEAT_T, SEAT_H, l0, l1), "wood"))
        boxes += _legs(f0, f1, l0, l1, SEAT_H - SEAT_T)
        back = p.get("back", "-f")
        if back in ("+f", "-f"):
            a = f1 - BACK_T if back == "+f" else f0
            boxes.append(((a, a + BACK_T, SEAT_H, SEAT_H + BACK_H, l0, l1), "wood"))
        else:
            a = l1 - BACK_T if back == "+l" else l0
            boxes.append(((f0, f1, SEAT_H, SEAT_H + BACK_H, a, a + BACK_T), "wood"))
    elif k == "desk":
        top = p.get("top", DESK_TOP)
        if "l" in p:                               # beside the figure: the edge runs along f at l, the desk beyond
            l, f = p["l"], p.get("f", 0.0)
            box = (f - DESK_HALF, f + DESK_HALF, l, l + DESK_DEPTH) if l > 0 else \
                (f - DESK_HALF, f + DESK_HALF, l - DESK_DEPTH, l)
        else:
            f = p["f"]
            box = (f, f + DESK_DEPTH, -DESK_HALF, DESK_HALF)
        f0, f1, l0, l1 = box
        boxes.append(((f0, f1, top - DESK_T, top, l0, l1), "wood"))
        boxes += _legs(f0 + 2, f1 - 2, l0 + 2, l1 - 2, top - DESK_T, w=5.0)
    elif k == "table":
        f0, f1, top = p["f0"], p["f1"], p.get("top", TABLE_TOP)
        boxes.append(((f0, f1, top - DESK_T, top, -TABLE_HALF, TABLE_HALF), "wood"))
        boxes += _legs(f0 + 2, f1 - 2, -TABLE_HALF + 2, TABLE_HALF - 2, top - DESK_T, w=6.0)
    elif k == "doorframe":
        f, s = p["f"], -1.0 if p.get("wall") == "-l" else 1.0
        boxes.append(((f, f + JAMB_T, 0.0, DOOR_H, -JAMB_W / 2, JAMB_W / 2), "wood"))
        l0, l1 = sorted((s * JAMB_W / 2, s * (JAMB_W / 2 + WALL_HALF)))
        boxes.append(((f, f + JAMB_T, 0.0, WALL_H + 30, l0, l1), "stone"))
    elif k == "door":
        if "l" in p:                               # beside the figure: the face at l, the wall beyond it
            l, f = p["l"], p.get("f", 0.0)
            s = 1.0 if l > 0 else -1.0
            wall = sorted((l, l + s * WALL_THICK))
            leaf = sorted((l, l - s * DOOR_PROUD))
            boxes.append(((f - WALL_HALF, f + WALL_HALF, 0.0, WALL_H + 30, wall[0], wall[1]), "stone"))
            boxes.append(((f - DOOR_HALF, f + DOOR_HALF, 0.0, DOOR_H, leaf[0], leaf[1]), "wood"))
            fit, rod = _door_parts()               # laid out along f, out of the face toward -l * s
            for (o0, o1, u0, u1, a0, a1), m in fit:
                (n0, n1) = sorted((l - s * o0, l - s * o1))
                boxes.append(((f + a0, f + a1, u0, u1, n0, n1), m))
            rods += [(tuple((f + a, u, l - s * o) for o, u, a in (x, y)) + (r,), m) for (x, y, r), m in rod]
        else:
            f = p["f"]
            boxes.append(((f, f + WALL_THICK, 0.0, WALL_H + 30, -WALL_HALF, WALL_HALF), "stone"))
            boxes.append(((f - DOOR_PROUD, f, 0.0, DOOR_H, -DOOR_HALF, DOOR_HALF), "wood"))
            fit, rod = _door_parts()               # out of the face toward -f
            boxes += [((f - o1, f - o0, u0, u1, a0, a1), m) for (o0, o1, u0, u1, a0, a1), m in fit]
            rods += [(tuple((f - o, u, a) for o, u, a in (x, y)) + (r,), m) for (x, y, r), m in rod]
    elif k == "bar":
        half = p.get("half", BAR_HALF)
        rods.append((_across(p["f"], p["u"], -half, half, BAR_R), "iron"))
        u0, u1 = (0.0, p["u"] + 2) if p.get("ground") else (p["u"], CEILING)
        for s in (-1, 1):
            boxes.append(((p["f"] - 2, p["f"] + 2, u0, u1, s * half - 2, s * half + 2), "iron"))
    elif k == "pole":
        l = p.get("l", 0.0)
        rods.append((((p["f"], 0.0, l), (p["f"], p["top"], l), POLE_R), "iron"))
    elif k == "rings":
        half = p.get("half", RING_HALF)
        for s in (-1, 1):
            l = s * half
            boxes.append(((p["f"] - 1.2, p["f"] + 1.2, p["u"] + RING_R, CEILING, l - 1.8, l + 1.8), "leather"))
            rods.append((_across(p["f"], p["u"], l - 6, l + 6, 1.6), "wood"))
    elif k == "bench":
        f0, f1, u = p["f0"], p["f1"], p["u"]
        boxes.append(((f0, f1, u - 14, u, -BENCH_HALF, BENCH_HALF), "leather"))
        for a in (f0 + 2, f1 - 7):
            for b in (-BENCH_HALF + 2, BENCH_HALF - 7):
                boxes.append(((a, a + 5, 0.0, u - 14, b, b + 5), "wood"))
    elif k == "step":
        boxes.append(((p["f0"], p["f1"], 0.0, p["u"], p.get("l0", -STEP_HALF), p.get("l1", STEP_HALF)), "wood"))
    elif k == "stairs":
        boxes, rods = _stairs(p)
    elif k == "bike":
        boxes, rods = _bike(p)
    elif k == "pad":
        f0, f1, u = p["f0"], p["f1"], p.get("u", PAD_U)
        boxes.append(((f0, f1, 0.0, u - 1.5, -PAD_HALF, PAD_HALF), "iron"))
        boxes.append(((f0 + 4, f1 - 4, u - 1.5, u, -PAD_HALF + 3, PAD_HALF - 3), "leather"))   # the belt
        boxes += _slats(p, at, f0 + 4, f1 - 4, u, PAD_HALF - 3, "iron")
        if p.get("rail"):
            x = f1 - 3.0
            for s in (-1, 1):
                rods.append((((x, u, s * PAD_HALF), (x, PAD_RAIL_U, s * PAD_HALF), 1.8), "iron"))
            rods.append((((x, PAD_RAIL_U, -PAD_HALF), (x, PAD_RAIL_U, PAD_HALF), 1.8), "iron"))
    elif k == "mat":
        boxes.append(((p["f0"], p["f1"], 0.0, MAT_T, p.get("l0", -MAT_HALF), p.get("l1", MAT_HALF)), "mat"))
    elif k == "runner":
        f0, f1, l0, l1 = p["f0"], p["f1"], p.get("l0", -RUNNER_HALF), p.get("l1", RUNNER_HALF)
        boxes.append(((f0, f1, 0.0, MAT_T, l0, l1), "mat"))
        boxes += _slats(p, at, f0, f1, MAT_T, (l1 - l0) / 2 - 4, "leather")
    elif k == "roller":
        n, r, l = p.get("len", ROLLER_LEN) / 2, p.get("r", ROLLER_R), p.get("l", 0.0)
        if p.get("along") == "f":
            rods.append((((p["f"] - n, r, l), (p["f"] + n, r, l), r), "leather"))
        else:
            rods.append((_across(p["f"], r, l - n, l + n, r), "leather"))
    else:
        raise ValueError(f"unknown prop kind {k!r} (known: {', '.join(KINDS)})")
    if k == "runner":                                       # the stripes across the runner's centre line
        c = (p.get("l0", -RUNNER_HALF) + p.get("l1", RUNNER_HALF)) / 2
        boxes = boxes[:1] + [((a, b, u0, u1, c + l0, c + l1), m) for (a, b, u0, u1, l0, l1), m in boxes[1:]]
    sc = p.get("scroll")
    if sc and at:
        df, du = sc[0] * at, sc[1] * at
        boxes = [((f0 + df, f1 + df, u0 + du, u1 + du, l0, l1), m) for (f0, f1, u0, u1, l0, l1), m in boxes]
        rods = [(((a[0] + df, a[1] + du, a[2]), (b[0] + df, b[1] + du, b[2]), r), m) for (a, b, r), m in rods]
    return boxes, rods


def rod_boxes(a, b, r, step=4.0):
    """A rod as the checks see it: one box around it when it runs along an axis, else a chain of small boxes."""
    d = [y - x for x, y in zip(a, b)]
    if sum(1 for x in d if abs(x) > 1e-6) <= 1:              # along an axis: its length that way, r across it
        k = max(range(3), key=lambda i: abs(d[i]))
        box = []
        for i in range(3):
            lo, hi = min(a[i], b[i]), max(a[i], b[i])
            box += [lo, hi] if i == k else [lo - r, hi + r]
        return [tuple(box)]
    n = max(1, math.ceil(math.sqrt(sum(x * x for x in d)) / step))
    out = []
    for i in range(n + 1):
        c = [x + y * i / n for x, y in zip(a, d)]
        out.append((c[0] - r, c[0] + r, c[1] - r, c[1] + r, c[2] - r, c[2] + r))
    return out


FLOOR = (-1e4, 1e4, -1e3, 0.0, -1e4, 1e4)        # the floor as a box: its top at 0, deep and wide past any camera


def solids(p, at=0.0):
    """The boxes the checks test joints against: every drawn box, and each rod as its boxes."""
    boxes, rods = parts(p, at)
    out = [b for b, _ in boxes]
    for (a, b, r), _ in rods:
        out += rod_boxes(a, b, r)
    return out


def gap(point, box):
    """Distance (cm) from a point to a box: positive outside, negative inside (the depth to the nearest face)."""
    f, u, l = point
    f0, f1, u0, u1, l0, l1 = box
    d = [max(f0 - f, 0.0, f - f1), max(u0 - u, 0.0, u - u1), max(l0 - l, 0.0, l - l1)]
    if any(d):
        return math.sqrt(sum(x * x for x in d))
    return -min(f - f0, f1 - f, u - u0, u1 - u, l - l0, l1 - l)


def top_in(point, boxes, reach=2.0, margin=0.0):
    """The height of the highest of these boxes' tops directly under a point (or up to `reach` above it; out to
    `margin` past a box's edges), or 0 when none is (a top may lie below 0: a sunken step)."""
    f, u, l = point
    m = margin
    return max((b[3] for b in boxes if b[0] - m <= f <= b[1] + m and b[4] - m <= l <= b[5] + m and b[3] <= u + reach),
               default=0.0)


def top_under(point, props, at=0.0):
    """The height of the highest prop top directly under a point, or the floor (0)."""
    return max(0.0, top_in(point, [b for p in props for b in solids(p, at)]))


def frame_points(p):
    """Points (f, u, l) the camera keeps in frame: the prop's boxes near the figure's plane (|l| <= 30), a hanging
    prop's mounts cut 25 cm above its grip, a flight's `frame` steps only (their treads one rise deep, at the loop's
    start and end for a scrolling one), a runner's `frame` span only."""
    if p["kind"] == "stairs" and p.get("frame"):
        q = dict(p, steps=p["frame"], rail=None)
        rise = p.get("rise", RISE)
        pts = []
        for a in ((0.0, 1.0) if p.get("scroll") else (0.0,)):
            for (f0, f1, u0, u1, l0, l1), _ in parts(q, a)[0]:
                pts += [(f, u, max(-30.0, min(30.0, l))) for f in (f0, f1) for u in (u1, u1 - rise) for l in (l0, l1)]
        return pts
    if p["kind"] == "runner":
        f0, f1 = p.get("frame", RUNNER_FRAME)
        return [(f, u, l) for f in (f0, f1) for u in (0.0, MAT_T) for l in (-30.0, 30.0)]
    return _points(p, 0.0)


def _points(p, at):
    boxes, rods = parts(p, at)
    cap = p["u"] + 25.0 if p["kind"] in ("bar", "rings") else math.inf
    pts = []
    for (f0, f1, u0, u1, l0, l1), _ in boxes:
        l0, l1 = max(l0, -30.0), min(l1, 30.0)
        if l0 > l1:
            continue
        for f in (f0, f1):
            for u in (u0, min(u1, cap)):
                for l in (l0, l1):
                    pts.append((f, u, l))
    for (a, b, r), _ in rods:
        for q in (a, b):
            l = max(-30.0, min(30.0, q[2]))
            pts += [(q[0] - r, q[1] - r, l), (q[0] + r, q[1] + r, l)]
    return pts
