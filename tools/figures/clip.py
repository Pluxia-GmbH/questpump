"""Plate-on-plate clipping in the real armour, measured (figures.py clip; Blender 5.2 LTS, headless):

    blender -b --factory-startup --python-exit-code 1 -P tools/figures/clip.py -- <job.json> <out.json>

<job.json> is {"moves": [{"move": key, "track": <track.json>, "fault": <the fault still's frame, or null>}]}, each track
as figures.py writes it for the renderer. Every frame is posed as blender/render.py poses it (armor_model.place: the
arms' turns past their tassets over the loop, arm_turns, the pauldrons' schedule, the fauld's ride and the tassets'
slide included), and every armour part (armor_model.RIGID) is measured against every other part it does not meet at a
joint (ADJACENT). <out.json> gets, per move, its frame count and, per pair that clips, its deepest point over the loop's
frames and, apart, in the fault still: the depth (cm), the frame, the part whose plates reach in and the number of
frames at least FLOOR deep. figures.py prints and ranks it.

This script lives outside blender/ on purpose: build.render_key hashes every .py file there into each render folder's
name, so a new or changed file there would render the whole library again (and armor_model.py cannot take these
helpers for the same reason).

The measure. A part's solid is what its plates enclose, in the part's own frame (armor_model's interface: the long axis
is local Z, the foot's X), cut in slices SLICE thick along that axis and SECTORS sectors around it. Where a slice's ring
of plate closes round its middle (no gap wider than GAP sectors: the helm, the cuirass, the belt and the lame above it,
the tubes of the arms and legs, the gauntlet, the sabaton, a pauldron's dome), the part is solid out to the ring's outer
surface, and a point inside sits as deep as its shortest way out: across the slice (rays from outside onto it, out from
the ring's middle and in WAYS directions, each stopping at the outermost face) or along the axis to where the closed
rings end. Everywhere else (a cop and its fan, a tasset, the fauld, the culet, the open front of the belly lames, a
trimmed tube end) an open plate encloses nothing, and a point sits only in the plate itself, as deep as its distance to
the plate's nearest face (inside where that face looks away from it). The points tested are the vertices of the other
part's raw mesh (ob.data, the frozen plates; the outline shell is a modifier on top, never read), so a pair's depth is
the deepest vertex of either part in the other. Two grids built once per part from its surface sampled every SAMPLE
(each sector's outermost sample, so the depth out from the ring's middle; the plates' signed distance on VOX voxels near
them, read trilinearly) give every vertex a depth in a few milliseconds a frame, and the deepest of them are measured on
the mesh itself (a BVH of the raw mesh: the rays, the nearest face) until the grids' depth plus SLACK cannot beat the
deepest found. armor_model._env_table is the closed-ring envelope for the two thigh-frame parts alone (it counts points
for fauld_turn); this one is any part's, with a depth.

Its limits: two open plates crossing read at most half a plate's thickness (5 to 11 mm), however far they cross; a
part hidden inside another reads as deep as one poking through it; a point in a closed ring's gap (a visor slit) reads
open; the depth along the axis is the slices'. Measured on 2026-10-14 over every sixth frame of eight moves
(childs_pose, forward_fold, bound_squat, crow, frog_stand, forehead_to_knees, air_squat, bear_crawl): of 3864 clipping
pairs, the search came within 1.2 mm of measuring each pair's 600 deepest vertices by the grids on the mesh (15 short by
more than 1 mm, none over), refining 30 vertices a pair and direction on average; at the 1.5 million vertices over 2 mm
deep the grids ran -0.8 to +6.9 mm from the mesh (5th to 95th percentile), deep where a ring is far from round.
"""
import json
import math
import os
import sys
import time

import bpy
import numpy as np
from mathutils import Vector
from mathutils.bvhtree import BVHTree

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "blender"))
import armor_model as am  # noqa: E402

SAMPLE = 0.003         # m: the spacing a part's surface is sampled at
SLICE = 0.004          # m along a part's axis
SECTORS = 120          # around it (3 degrees each)
GAP = 10               # sectors: a ring whose widest gap is at most this (30 degrees) is closed, the gap bridged
VOX = 0.004            # m: the grid of the plates' own signed distance
NEAR = 3               # voxels round the plates' surface it covers (12 mm: a 22-mm plate's middle inside)
WAYS = 12              # directions across a slice a point's way out of a closed ring is sought in (every 30 degrees:
#                        at most 3.5 % longer than the straight way to a flat face)
CHUNK = 0.03           # m: the cubes a part's vertices are grouped in
REFINE = 256           # vertices at most a pair measures on the mesh each way, the grids' deepest first, until
SLACK = 0.004          # m added to the grids' depth no longer beats the deepest found (the grids run short of the mesh
#                        by 0.8 mm at the 5th percentile: the docstring's limits)
FLOOR = 0.5            # cm: a frame counts toward a pair's `frames` from this depth


def _pairs(*pairs):
    return {tuple(sorted(p)) for p in pairs}


# Parts that meet at a joint and touch there by design, never measured against each other: the torso's plates (the
# cuirass, the belly lames, the fauld and the culet) telescope as the waist bends and the hips flex; a cop rides over
# its two tubes, which fold into each other at full bend under the trims ELBOW_BEND and KNEE_BACK; a pauldron over the
# shoulder; the fauld, the culet and the tassets over the hips. Every other pair is measured.
ADJACENT = _pairs(("head", "chest"), ("chest", "abdomen"), ("chest", "fauld"), ("chest", "culet"),
                  ("abdomen", "fauld"), ("abdomen", "culet"), ("fauld", "culet"))
for _s in ("near", "far"):
    ADJACENT |= _pairs(
        ("chest", f"pauldron_{_s}"), ("chest", f"upper_arm_{_s}"), (f"pauldron_{_s}", f"upper_arm_{_s}"),
        (f"upper_arm_{_s}", f"elbow_{_s}"), (f"elbow_{_s}", f"forearm_{_s}"), (f"upper_arm_{_s}", f"forearm_{_s}"),
        (f"forearm_{_s}", f"hand_{_s}"),
        ("abdomen", f"thigh_{_s}"), ("abdomen", f"tasset_{_s}"), ("fauld", f"thigh_{_s}"), ("fauld", f"tasset_{_s}"),
        ("culet", f"thigh_{_s}"), ("culet", f"tasset_{_s}"), (f"thigh_{_s}", f"tasset_{_s}"),
        (f"thigh_{_s}", f"knee_{_s}"), (f"knee_{_s}", f"shin_{_s}"), (f"thigh_{_s}", f"shin_{_s}"),
        (f"shin_{_s}", f"foot_{_s}"))


def _runs(occ):
    """For a boolean (n, ...) array: per cell, how many cells of its run of True along axis 0 lie at or before it and
    at or after it (0 where False)."""
    back, ahead = np.zeros(occ.shape, int), np.zeros(occ.shape, int)
    for k in range(len(occ)):
        back[k] = np.where(occ[k], (back[k - 1] if k else 0) + 1, 0)
    for k in range(len(occ) - 1, -1, -1):
        ahead[k] = np.where(occ[k], (ahead[k + 1] if k + 1 < len(occ) else 0) + 1, 0)
    return back, ahead


def _sector(x, y):
    """(radius, sector) of points relative to a slice's middle; sector 0 starts at -180 degrees."""
    phi = np.arctan2(y, x) + math.pi
    return np.hypot(x, y), np.minimum((phi * SECTORS / (2 * math.pi)).astype(int), SECTORS - 1)


def _surface(me, V):
    """The raw mesh's surface, densely: every triangle sampled on a barycentric grid no coarser than SAMPLE (a long
    cap triangle as finely as a small one), with the vertices (local, m)."""
    me.calc_loop_triangles()
    T = np.empty(len(me.loop_triangles) * 3, dtype=np.int64)
    me.loop_triangles.foreach_get("vertices", T)
    A, B, C = (V[T[i::3]] for i in range(3))
    edge = np.max([np.linalg.norm(A - B, axis=1), np.linalg.norm(B - C, axis=1), np.linalg.norm(C - A, axis=1)], axis=0)
    m = np.maximum(1, np.ceil(edge / SAMPLE)).astype(int)
    out = [V]
    for n in np.unique(m):
        i, j = (x.ravel() for x in np.meshgrid(np.arange(n + 1), np.arange(n + 1), indexing="ij"))
        keep = i + j <= n
        w1, w2 = i[keep] / n, j[keep] / n
        t = m == n
        out.append((A[t, None] * (1 - w1 - w2)[:, None] + B[t, None] * w1[:, None] + C[t, None] * w2[:, None])
                   .reshape(-1, 3))
    return np.concatenate(out)


class Solid:
    """One part in its own frame (the module's docstring): its closed rings' envelope, its plates' signed distance near
    their surface, its vertices in chunks (the points it brings to the other parts), its box and a bounding sphere."""

    def __init__(self, ob):
        me = ob.data
        V = am._verts_np(ob)
        S = _surface(me, V)
        # the points this part brings to the others: its vertices, grouped in CHUNK cubes, so a pair tests only the
        # chunks that reach the other part's box
        cube = np.unique(((V - V.min(axis=0)) // CHUNK).astype(int), axis=0, return_inverse=True)[1].ravel()
        self.verts = V[np.argsort(cube, kind="stable")]
        ends = np.cumsum(np.bincount(cube))
        self.chunks = list(zip(ends - np.bincount(cube), ends))
        lo = np.array([self.verts[a:b].min(axis=0) for a, b in self.chunks])
        hi = np.array([self.verts[a:b].max(axis=0) for a, b in self.chunks])
        self.chunk_mid = (lo + hi) / 2
        self.chunk_r = np.linalg.norm(hi - lo, axis=1) / 2
        self.box = S.min(axis=0), S.max(axis=0)
        self.axis = 0 if ob.name.startswith("foot_") else 2          # the foot's toes are +X; every other part's Z
        self.cross = [i for i in range(3) if i != self.axis]
        s = S[:, self.axis]
        self.lo = float(s.min())
        nz = self.nz = int((s.max() - self.lo) // SLICE) + 1
        iz = ((s - self.lo) // SLICE).astype(int)
        a, b = S[:, self.cross[0]], S[:, self.cross[1]]
        mid = np.zeros((nz, 2))
        for c, x in enumerate((a, b)):
            lo, hi = np.full(nz, np.inf), np.full(nz, -np.inf)
            np.minimum.at(lo, iz, x)
            np.maximum.at(hi, iz, x)
            seen = np.isfinite(lo)
            mid[seen, c] = (lo[seen] + hi[seen]) / 2
        r, j = _sector(a - mid[iz, 0], b - mid[iz, 1])
        E = np.zeros((nz, SECTORS))
        np.maximum.at(E, (iz, j), r)
        empty = E == 0.0
        gap_back, gap_ahead = _runs(np.concatenate([empty, empty, empty], axis=1).T)   # gaps round the ring
        widest = (gap_back + gap_ahead - 1).T[:, SECTORS:2 * SECTORS].max(axis=1, initial=0)
        self.closed = widest <= GAP
        ang = (np.arange(SECTORS) + 0.5) * 2 * math.pi / SECTORS
        for k in np.flatnonzero(self.closed):
            m = ~empty[k]
            E[k] = np.interp(ang, ang[m], E[k][m], period=2 * math.pi)
        self.mid, self.E = mid, E
        back, ahead = _runs(self.closed)                            # the closed rings' runs along the axis
        k = np.arange(nz)
        self.z0 = self.lo + (k - back + 1) * SLICE
        self.z1 = self.lo + (k + ahead) * SLICE
        # the plates' own signed distance (inside > 0) on the voxels near their surface, from a BVH of the raw mesh:
        # a voxel is inside when the nearest face looks away from it; every other voxel lies outside
        self.v0 = S.min(axis=0) - (NEAR + 1) * VOX
        g = np.unique(((S - self.v0) // VOX).astype(int), axis=0)
        near = np.zeros(g.max(axis=0) + NEAR + 2, bool)
        for dx in range(-NEAR, NEAR + 1):
            for dy in range(-NEAR, NEAR + 1):
                for dz in range(-NEAR, NEAR + 1):
                    near[g[:, 0] + dx, g[:, 1] + dy, g[:, 2] + dz] = True
        bvh = self.bvh = BVHTree.FromPolygons([tuple(v) for v in V], [tuple(p.vertices) for p in me.polygons])
        sd = np.full(near.shape, -VOX * NEAR)
        cells = np.argwhere(near)
        for c, x in zip(cells, self.v0 + (cells + 0.5) * VOX):
            x = Vector(x)
            at, normal, _, dist = bvh.find_nearest(x, VOX * NEAR)
            if at is not None:
                sd[tuple(c)] = dist if (x - at).dot(normal) < 0.0 else -dist
        self.sd = sd.ravel()                                       # flat, read at a cell's 8 corners
        self.step = np.array([sd.shape[1] * sd.shape[2], sd.shape[2], 1])
        self.top = np.array(sd.shape) - 1
        self.corners = [(int(np.dot(o, self.step)), o) for o in np.ndindex(2, 2, 2)]
        self.centre = (S.min(axis=0) + S.max(axis=0)) / 2
        self.radius = float(np.linalg.norm(S - self.centre, axis=1).max())
        self.reach = 2 * self.radius                                # from outside the part onto any point in it
        self.ways = np.zeros((WAYS, 3))
        turn = np.arange(WAYS) * 2 * math.pi / WAYS
        self.ways[:, self.cross[0]], self.ways[:, self.cross[1]] = np.cos(turn), np.sin(turn)

    def depth(self, P):
        """How deep each point P (n, 3; this part's frame, m) sits in this part: 0 outside."""
        out = np.zeros(len(P))
        s = P[:, self.axis]
        k = np.floor((s - self.lo) / SLICE).astype(int)
        ring = (k >= 0) & (k < self.nz)
        ring[ring] = self.closed[k[ring]]
        if ring.any():                                              # under a closed ring (its plate is inside it)
            q, kq, sq = P[ring], k[ring], s[ring]
            r, j = _sector(q[:, self.cross[0]] - self.mid[kq, 0], q[:, self.cross[1]] - self.mid[kq, 1])
            d = np.minimum.reduce([self.E[kq, j] - r, sq - self.z0[kq], self.z1[kq] - sq])
            out[ring] = np.maximum(d, 0.0)
        rest = np.flatnonzero(~ring)                                # in an open plate: its signed distance, trilinear
        u = (P[rest] - self.v0) / VOX - 0.5
        i = np.floor(u).astype(int)
        ok = ((i >= 0) & (i < self.top)).all(axis=1)
        if ok.any():
            i, f = i[ok], u[ok] - i[ok]
            g = 1.0 - f
            base = i @ self.step
            v = 0.0
            for off, (ox, oy, oz) in self.corners:
                v = v + self.sd[base + off] * (f[:, 0] if ox else g[:, 0]) * (f[:, 1] if oy else g[:, 1]) * \
                    (f[:, 2] if oz else g[:, 2])
            out[rest[ok]] = np.maximum(v, 0.0)
        return out

    def exact(self, p):
        """The depth of one point (this part's frame, m) on the mesh itself: under a closed ring, its shortest way out,
        across the slice's plane (rays from outside onto it, out from the ring's middle and in WAYS directions, each
        stopping at the outermost face) or along the axis to where the closed rings end, and outside when the ray out
        from the middle or two of the others meet no face (a single miss is a gap between plates); in an open plate,
        its distance to the nearest face, inside when that face looks away from it."""
        s = p[self.axis]
        k = int((s - self.lo) // SLICE)
        if 0 <= k < self.nz and self.closed[k]:
            v = p.copy()
            v[self.axis], v[self.cross] = 0.0, p[self.cross] - self.mid[k]
            r = float(np.linalg.norm(v))
            out, missed = min(s - self.z0[k], self.z1[k] - s), 0
            for u, outward in ([(v / r, True)] if r > 1e-6 else []) + [(u, False) for u in self.ways]:
                at, _, _, h = self.bvh.ray_cast(Vector(p + u * self.reach), Vector(-u), self.reach)
                if at is None:
                    if outward or missed:
                        return 0.0
                    missed = 1
                else:
                    out = min(out, self.reach - h)
            return max(0.0, out)
        q = Vector(p)
        at, normal, _, dist = self.bvh.find_nearest(q, VOX * NEAR)
        return dist if at is not None and (q - at).dot(normal) < 0.0 else 0.0


def measure(parts, solids, pairs):
    """The parts as placed now: [(pair index, depth m, the part that reaches in)] for every pair that clips."""
    M = {n: np.array(parts[n].matrix_world) for n in solids}
    Minv = {n: np.linalg.inv(m) for n, m in M.items()}
    world = {n: M[n][:3, :3] @ sol.centre + M[n][:3, 3] for n, sol in solids.items()}
    found = []
    for i, (a, b) in enumerate(pairs):
        A, B = solids[a], solids[b]
        if np.linalg.norm(world[a] - world[b]) >= A.radius + B.radius:
            continue
        best, into = 0.0, None
        for x, X, y, Y in ((a, A, b, B), (b, B, a, A)):
            R = Minv[y] @ M[x]                                   # x's frame -> y's
            c = X.chunk_mid @ R[:3, :3].T + R[:3, 3]
            r = X.chunk_r[:, None]
            hit = np.flatnonzero(((c + r > Y.box[0]) & (c - r < Y.box[1])).all(axis=1))
            if not len(hit):
                continue
            P = np.concatenate([X.verts[X.chunks[h][0]:X.chunks[h][1]] for h in hit]) @ R[:3, :3].T + R[:3, 3]
            d = Y.depth(P)
            for n in np.argsort(-d)[:REFINE]:          # the deepest by the grids, measured on the mesh
                if d[n] <= 0.0 or d[n] + SLACK <= best:
                    break
                e = Y.exact(P[n])
                if e > best:
                    best, into = e, x
        if best > 0.0:
            found.append((i, best, into))
    return found


def run(job, out):
    t0 = time.time()
    bpy.ops.wm.read_factory_settings(use_empty=True)
    parts = am.build()
    solids = {n: Solid(parts[n]) for n in am.RIGID}
    pairs = [(a, b) for i, a in enumerate(am.RIGID) for b in am.RIGID[i + 1:] if tuple(sorted((a, b))) not in ADJACENT]
    print(f"clip: armour built in {time.time() - t0:.1f} s, {len(pairs)} pairs", flush=True)
    result = {}
    for m in job["moves"]:
        with open(m["track"]) as fh:
            track = json.load(fh)
        loop, still = {}, {}
        turns = dict(enumerate(am.arm_turns(track["frames"], track["view"])))
        frames = list(enumerate(track["frames"]))
        if m.get("fault"):                           # the still turns its arms as the fault track does
            k = track["key"]
            turns["fault"] = am.arm_turns(track["frames"][:k] + [m["fault"]] + track["frames"][k + 1:],
                                          track["view"])[k]
            frames.append(("fault", m["fault"]))
        for f, fr in frames:
            am.place(parts, fr["joints"], turns[f], view=track["view"], sash=fr["sash"], head_l=fr.get("head_l"),
                     sash_rest=fr["sash_rest"])
            for i, d, into in measure(parts, solids, pairs):
                cm = round(d * 100, 2)
                if not cm:
                    continue
                w = (still if f == "fault" else loop).setdefault("/".join(pairs[i]), {"depth": 0.0, "frames": 0})
                w["frames"] += int(cm >= FLOOR)
                if cm > w["depth"]:
                    w.update(depth=cm, frame=f, inside=into)
        result[m["move"]] = {"frames": len(track["frames"]), "pairs": loop}
        if m.get("fault"):
            result[m["move"]]["fault"] = still
        print(f"clip: {m['move']} {max((w['depth'] for w in loop.values()), default=0.0):.1f} cm", flush=True)
    with open(out, "w") as fh:
        json.dump({"moves": result, "floor": FLOOR, "adjacent": sorted("/".join(p) for p in ADJACENT),
                   "seconds": round(time.time() - t0, 1)}, fh, indent=1, sort_keys=True)


if __name__ == "__main__":
    argv = sys.argv[sys.argv.index("--") + 1:]
    with open(argv[0]) as fh:
        run(json.load(fh), argv[1])
