"""The helm's shape for the checks, measured on the real armour (figures.py helm; Blender 5.2 LTS, headless):

    blender -b --factory-startup --python-exit-code 1 -P tools/figures/helm.py -- <out.json>

checks.py tests the helm past the head's own ball (CLEARANCE) against the floor and the props by points on it, and
the lines a move holds (a rope, a band, a towel) by a distance grid round it. This builds the armour as the renderer
does (blender/armor_model.build: the helm at HELM_SCALE, HEAD_LIFT up its neck) and writes both in cm from the head
joint: a up the neck-to-head axis, b forward (the helm's facing), c out to one side. The helm is mirror symmetric:
every point and grid point has c >= 0, and checks.py mirrors the points with c > 0 and reads the grid at |c|.

  the outline   the helm's farthest vertex in each of OUTLINE directions spread over the sphere: a flat face (a wall,
                the floor, a bench's top) that enters the helm reaches one of them within about a centimetre
  the middle    a GRID grid over the helm's middle plane (c = 0), from its back to its front at each height: a rod or a
                rail (props.RAIL_R) that crosses the visor, the brow, the crest fin or the plume reaches one of them

A point inside the head's own ball is left out (the head joint is tested with it).

  the grid      the distance to the helm's surface every STEP cm over its box grown by PAD, negative inside, PAD at
                most, in mm: a point is inside when a ray along none of ESCAPE directions leaves the helm without
                meeting it, since its parts overlap and face every way, so neither a ray's count of crossings nor a
                face's normal can tell; the hollow round the head, open at the neck, reads outside (the head's own
                ball is the neck-to-head capsule's)

<out.json> also gets the SHA-256 of blender/armor_model.py it was measured from, and checks.helm_stale names the file
stale once that has changed.

This script lives outside blender/ on purpose: build.render_key hashes every .py file there into each render folder's
name. Measured on 2026-10-22: OUTLINE 80 reaches within 1.0 cm of the helm's extent in any of 2000 directions (30
directions left 3.1 cm); the helm reaches 18.5 cm ahead of the head joint at its brow and gem, 14.6 behind at the plume
and 30.7 over it at the brush's top; the grid holds 6187 points inside the helm's c >= 0 half.
"""
import hashlib
import json
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "blender"))
sys.path.insert(0, HERE)
import armor_model  # noqa: E402
from mathutils import Vector  # noqa: E402
from mathutils.bvhtree import BVHTree  # noqa: E402
from rig import BONES, CLEARANCE  # noqa: E402

OUTLINE = 80             # directions over the sphere for the outline's farthest vertices
GRID = 3.0               # cm between the middle plane's points: under the thinnest rod's 4.4 cm (props.RAIL_R)
MID = 2.5                # cm either side of the middle plane a vertex counts as on it (the crest fin is 3.8 thick)
STEP = 1.0               # cm between the distance grid's points
PAD = 3.0                # cm the grid reaches past the helm: past the thickest line's radius (items.TOWEL_ROPE_R)
ESCAPE = 26              # directions a ray tries to leave the helm along from a grid point


def sphere(n):
    """n directions spread evenly over the sphere (a Fibonacci lattice)."""
    g = math.pi * (3.0 - math.sqrt(5.0))
    out = []
    for i in range(n):
        y = 1.0 - 2.0 * (i + 0.5) / n
        r = math.sqrt(1.0 - y * y)
        out.append((y, r * math.cos(g * i), r * math.sin(g * i)))
    return out


def middle(pts):
    """The grid over the middle plane: a row for each GRID band up the axis, at the mean height of the band's vertices
    on the plane (so the top row stays on the brush, never over it), from the rearmost of them to the foremost, its
    ends on them."""
    mid = [p for p in pts if p[2] < MID]
    lo, hi = min(p[0] for p in mid), max(p[0] for p in mid)
    out = []
    for k in range(int((hi - lo) / GRID) + 1):
        band = [p for p in mid if abs(p[0] - (lo + GRID * (k + 0.5))) <= GRID / 2]
        if band:
            a = sum(p[0] for p in band) / len(band)
            b0, b1 = min(p[1] for p in band), max(p[1] for p in band)
            n = max(1, round((b1 - b0) / GRID))
            out += [(a, b0 + (b1 - b0) * i / n, 0.0) for i in range(n + 1)]
    return out


def grid(verts, polys):
    """The distance grid (the docstring's): its first point, its size in points along a, b and c, and its rows of mm,
    one per (a, b) with c rising."""
    bvh = BVHTree.FromPolygons([Vector(v) for v in verts], polys)
    rays = [Vector(d) for d in sphere(ESCAPE)]
    lo = [math.floor((min(v[k] for v in verts) - PAD) / STEP) for k in range(2)] + [0]
    hi = [math.ceil((max(v[k] for v in verts) + PAD) / STEP) for k in range(3)]
    size = [h - l + 1 for l, h in zip(lo, hi)]
    rows, inside = [], 0
    for i in range(lo[0], hi[0] + 1):
        for j in range(lo[1], hi[1] + 1):
            row = []
            for k in range(hi[2] + 1):
                p = Vector((i * STEP, j * STEP, k * STEP))
                d = bvh.find_nearest(p, PAD)[3]
                if d is not None and all(bvh.ray_cast(p, r)[0] is not None for r in rays):
                    d, inside = -d, inside + 1
                row.append(round(10.0 * (PAD if d is None else d)))
            rows.append(row)
    return [x * STEP for x in lo], size, rows, inside


def main():
    out = sys.argv[sys.argv.index("--") + 1]
    ob = armor_model.build()["head"]
    lift, neck = armor_model.HEAD_LIFT * 100.0, BONES["neck_head"]
    verts = [(v.co.z * 100.0 + lift - neck, v.co.x * 100.0, v.co.y * 100.0) for v in ob.data.vertices]
    pts = [(a, b, abs(c)) for a, b, c in verts]
    both = pts + [(a, b, -c) for a, b, c in pts]
    outline = [max(both, key=lambda p: p[0] * d[0] + p[1] * d[1] + p[2] * d[2]) for d in sphere(OUTLINE)]
    keep = sorted({tuple(round(x, 1) for x in (a, b, abs(c))) for a, b, c in outline + middle(pts)
                   if math.sqrt(a * a + b * b + c * c) > CLEARANCE["head"]})
    with open(os.path.join(HERE, "blender", "armor_model.py"), "rb") as fh:
        armour = hashlib.sha256(fh.read()).hexdigest()
    origin, size, rows, inside = grid(verts, [tuple(p.vertices) for p in ob.data.polygons])
    with open(out, "w") as fh:
        fh.write('{"armour": "%s", "points": [\n' % armour)
        fh.write(",\n".join(json.dumps(list(p)) for p in keep))
        fh.write('\n], "grid": {"origin": %s, "step": %s, "pad": %s, "size": %s, "mm": [\n'
                 % (json.dumps(origin), STEP, PAD, json.dumps(size)))
        fh.write(",\n".join(json.dumps(r, separators=(",", ":")) for r in rows))
        fh.write("\n]}}\n")
    print(f"helm: {len(keep)} points, {sum(1 for p in keep if p[2] > 0)} of them mirrored; a grid of "
          f"{'x'.join(map(str, size))} points, {inside} inside")


if __name__ == "__main__":
    main()
