"""The figure player's pure core in Python, line for line the twin of Figure.lua's (Resolve, CellCoords, NewState,
Step), and the writer of contract/fixtures/figure-steps.json (docs/plan.md slice 2, "Outside the game"): sample run
tables from the pack with the cell, the blended cell, its alpha and the texture coordinates after each jittered step,
the numbers Figure.lua and the watch players step to (player.js, the JavaScript twin the review page and the site's
hero play, stepped to all of them in a one-off probe on 2026-10-08).

The arithmetic keeps Lua's order (IEEE doubles in both), so the two agree exactly.
"""
import copy
import json
import math
import os
import random

HERE = os.path.dirname(os.path.abspath(__file__))
FIXTURE = os.path.join(HERE, "..", "..", "contract", "fixtures", "figure-steps.json")


def resolve(fig, name):
    """(the move's entry, the entry whose sheet it draws on): a variant names its reference in `ref`."""
    m = fig["moves"].get(name)
    if m is None:
        return None, None
    return m, (fig["moves"][m["ref"]] if "ref" in m else m)


def cell_coords(base, c, mirror):
    """SetTexCoord(l, r, t, b) of cell c (0-based); a mirrored move swaps left and right."""
    cols = base["cols"]
    col = c % cols
    row = math.floor(c / cols)
    sw, shh = base["sheet"][0], base["sheet"][1]
    x = col * base["cell"][0] + base["pad"]
    y = row * base["cell"][1] + base["pad"]
    l, r = x / sw, (x + base["size"][0]) / sw
    t, b = y / shh, (y + base["size"][1]) / shh
    if mirror:
        l, r = r, l
    return l, r, t, b


def new_state():
    return {"t": 0.0, "run": 1, "runStart": 0}


def step(st, m, hz, elapsed):
    """Advance by `elapsed` s; returns (cell, the cell blended over it or None, the blend's alpha). Runs are 1-based
    as in Lua; the loop wraps with an explicit floor."""
    seq = m["seq"]
    nruns = len(seq) // 2
    loop = m["ticks"] / hz
    t = st["t"] + elapsed
    if t >= loop:
        t = t - math.floor(t / loop) * loop
        st["run"], st["runStart"] = 1, 0
    st["t"] = t
    tick = t * hz
    while st["run"] < nruns and tick >= st["runStart"] + seq[2 * st["run"] - 1]:
        st["runStart"] = st["runStart"] + seq[2 * st["run"] - 1]
        st["run"] = st["run"] + 1
    cell = seq[2 * st["run"] - 2]
    for f in m["fade"]:
        if f == st["run"]:
            nxt = st["run"] % nruns + 1
            return cell, seq[2 * nxt - 2], (tick - st["runStart"]) / seq[2 * st["run"] - 1]
    return cell, None, 0.0


def play(fig, name, dts):
    m, base = resolve(fig, name)
    st = new_state()
    out = []
    for dt in dts:
        cell, blend, alpha = step(st, m, fig["hz"], dt)
        out.append({"dt": dt, "cell": cell, "blend": blend, "alpha": alpha,
                    "coords": list(cell_coords(base, cell, m["mirror"]))})
    return out


def jitter(n, seed, hitch_at=None):
    """n frame times around 60 fps (13-21 ms), from a fixed seed; one 0.5 s hitch at `hitch_at`."""
    rnd = random.Random(seed)
    dts = [round(rnd.uniform(0.013, 0.021), 6) for _ in range(n)]
    if hitch_at is not None:
        dts[hitch_at] = 0.5
    return dts


def pick(moves, want):
    for w in want:
        if w in moves:
            return w
    raise KeyError(f"none of {want} in the pack")


def write_fixture(pack):
    """The cases the plan's acceptance names: a hold, a timing variant on its reference move's sheet, a mirror, a
    cross-faded breathing hold and a hitch in the frame time."""
    moves = pack["moves"]
    hold = pick(moves, ["plank", "wall_sit_90", "dead_hang"])
    variant = pick([m for m in moves if "ref" in moves[m]], ["tempo_squat", "nordic_hold", "nordic_assisted"])
    rep = moves[variant]["ref"]
    one = pick(moves, ["split_squat", "air_squat"])
    sub = {k: copy.deepcopy(moves[k]) for k in {hold, variant, rep, one}}
    for e in sub.values():
        e.pop("file", None)
        e.pop("tint", None)
    mirror = dict(copy.deepcopy(sub[one]), ref=one, mirror=True)
    for k in ("cols", "sheet", "cell", "pad", "size", "n"):
        mirror.pop(k, None)
    sub["mirrored"] = mirror
    fig = {"hz": pack["hz"], "moves": sub}
    cases = [
        {"name": "a hold that cross-fades its breathing", "move": hold, "steps": play(fig, hold, jitter(400, 1))},
        {"name": "a timing variant on its reference move's sheet", "move": variant,
         "steps": play(fig, variant, jitter(500, 2))},
        {"name": "a mirrored move", "move": "mirrored", "steps": play(fig, "mirrored", jitter(300, 3))},
        {"name": "a hitch in the frame time", "move": rep, "steps": play(fig, rep, jitter(300, 4, hitch_at=120))},
    ]
    data = {"about": "Written by tools/figures/steps.py from the pack (make figures): each case's steps with the "
                     "cells, blends, alphas and texture coordinates Figure.lua and the watch players step to.",
            "hz": fig["hz"], "moves": sub, "cases": cases}
    with open(FIXTURE, "w") as fh:
        json.dump(data, fh, indent=1, sort_keys=True)
        fh.write("\n")
    return FIXTURE
