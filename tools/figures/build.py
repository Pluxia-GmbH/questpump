"""The figure build (make figures; docs/plan.md slice 10): motion files in, QuestPump_FigArmor/, the key-pose icons,
the phones' and the desktop app's films, the contact sheets and the review page out. A move is never rendered while
its motion file fails to load or fails a check, and one move's trouble never stops the others: a move that fails its
checks or its build keeps its pack entry as it was (or stays without one), a fault still the render's edge cuts is
left out of the move's sheet, and the run writes everything else, then lists those moves and exits nonzero (`run`
returns the list, `figures.py build` prints it last).

Per move, in build/render/<move>/<track sha8>/ (a changed motion or style renders into a new folder; prune deletes
the old ones):
  1. the crimson figure pass of every 30 Hz frame of one loop (the frame selection, the icon and the fault pair);
  2. the frame selection on those figure-only cells (pack.select, at the motion file's `fast_phase`): the stored
     frames and the run table;
  3. the stored frames again at the five grey cloth levels, and their floor and prop layers (with the key frame);
  4. the cells composited card -> floor -> figure -> prop -> glow per level (figures.layers: the prop layer holds
     only what the figure leaves in sight, so a prop hides the figure where it stands in front of it), split into
     base and tint (pack.split);
  5. the base sheet (the smallest palette PNG whose worst cell scores SSIMULACRA2 >= 82) and the tint sheet
     (32-colour PNG);
then the variants' tables, the manifest and pack.json, the icon sheet, the films, the contact sheets, the review page
and contract/fixtures/figure-steps.json. A gear variant (`<move>.<gear>`, motion.py) builds as a move of its own, the
gear variant of a timing variant as a timing variant of its reference's; the icon sheet holds library moves alone, so a
gear variant's key pose stays its move's icon.

The phones' film (write_loop; slice 29) is the same split at the film's size: each stored frame's five grey levels
composited at publish.SIDE px for the 192 px box and split as the sheets are, the base on the card over the tint on
black, stacked in one frame with a black band between (publish.py's SIDE, GUARD and TINT), so one decoder plays both
in step and a phone adds the tint in the class colour; its colour is tagged as what it is (FILM_COLOUR). The same
frames are encoded twice (ENCODINGS; slice 35): HEVC for the phones (build/loops/<move>.mp4) and H.264 for the desktop
app (build/films/<move>.mp4), whose Windows web view plays HEVC only through an extra Windows extension. A timing
variant gets films of its own (2026-10-09), its run table over its reference move's frames, written whenever its
reference is built, so the apps play it at its own tempo as the game does.

A move with a fault (motion.py, `fault`; slice 22) also renders that one still, in build/render/<move>/fault.<sha8>/
(the fault pose in the key frame's place of the loop's track, so a changed fault renders one frame and never the
loop). The still is cut with the loop's crop and laid out as the last cell of both sheets, after the stored frames
(the entry's `fault`, with `fault_cap`, its caption key), and the phones get the pair as two stills with their tints
(build/stills/<move>.key.png and <move>.fault.png, <move>.key.t.png and <move>.fault.t.png), for which the key tick
renders at the grey levels too.

`run(films=True)` (figures.py films) renders what is missing and writes only the films and the phones' pairs, never the
pack, the icons, the fixture, the contact sheets or the review page, and only for a move whose selection gives its
pack entry as it stands, with each of its timing variants whose table then still gives its own: one it would change is
named, and make figures rebuilds both. It is for a change to the films' form alone: a style, Blender script, ground or
fault-pose change gives new pixels under the same entry, which its guard cannot see, so it needs make figures, or the
phones would show what the pack's sheets do not.
"""
import glob
import hashlib
import json
import math
import os
import re
import shutil
import subprocess
import sys
import tempfile
import time
from concurrent.futures import ThreadPoolExecutor

import numpy as np
from PIL import Image
from scipy import ndimage

import figures
import motion
import pack as P
import publish
import steps

HERE = figures.HERE
ROOT = figures.ROOT
BUILD = figures.BUILD
PACK_DIR = os.path.join(ROOT, "QuestPump_FigArmor")
PACK_NAME = "QuestPump_FigArmor"
# the key-pose icon sheets, 256 cells each (slice 21: the poses outgrew one); a cell's index runs on across them
ICONS_PNG = [os.path.join(ROOT, "Media", "MoveIcons.png"), os.path.join(ROOT, "Media", "MoveIcons2.png"),
             os.path.join(ROOT, "Media", "MoveIcons3.png")]                        # the third since slice 47's poses
# The files this build added to the game's folders: the client on macOS finds a new addon file only at its start, so a
# /reload shows them as the green missing-texture fill (2026-10-04, a re-rendered figure's new content-hashed name)
NEW_FILES = []
ICONS_LUA = os.path.join(ROOT, "FigureIcons.lua")
CELL = 640                    # render at twice this (1280 px), Lanczos down into the 192 px box
ICON_CELL, ICON_INNER, ICON_GRID = 64, 60, 16
LOOP_PX = publish.SIDE        # the phones' film square (publish.py owns the film's layout)


def render_key():
    """What else decides a render: the style, the Blender scripts and the cell size. Part of every render folder's
    name, so changing any of them renders afresh instead of reusing old frames."""
    h = hashlib.sha256(str(CELL).encode())
    scripts = os.path.join(HERE, "blender")
    for path in [figures.STYLE] + sorted(os.path.join(scripts, f) for f in os.listdir(scripts) if f.endswith(".py")):
        with open(path, "rb") as fh:
            h.update(fh.read())
    return h.hexdigest()[:12]


RENDER_KEY = render_key()
LEVELS = sorted(P.LEVELS)


def sha8(data):
    return hashlib.sha256(json.dumps(data, sort_keys=True).encode()).hexdigest()[:8]


def moves_json():
    with open(os.path.join(ROOT, "contract", "moves.json")) as fh:
        return {m["key"]: int(k) for k, m in json.load(fh)["moves"].items()}


# ---------------------------------------------------------------- rendering

def render_folder(track, prefix=""):
    """build/render/<move>/<prefix><sha8>: the folder one track renders into, named by everything that decides its
    pixels (the track as the renderer reads it and the render key)."""
    return os.path.join(BUILD, "render", track["move"],
                        prefix + sha8([{k: v for k, v in track.items() if not k.startswith("_")}, RENDER_KEY]))


QUIET_S = 3600   # a render folder touched this recently may be another session's build at work: never pruned
# build/loops/<move>/: film_frame's frames (f), by render folder and size, and what no build reads since slice 29: the
# crimson loops' cells (c; by render folder, or before that by size alone) and a grey level's (.g###) a stopped run
# left, as loop_cell writes into a run's own folder now
LOOP_CELL = re.compile(r"^([cf])\d{4}_(?:(.+)_)?[0-9a-f]{8}(\.g\d{3})?\.png$")
SCRATCH = "cut-"   # a run's own folder in build/loops/<move>/ (scratch), deleted after use


def keep_folders(baked):
    """{move: the names of the render folders this tree renders it into}: its loop's and, where it has a fault, its
    still's (render_fault)."""
    keep = {}
    for move, (spec, track) in baked.items():
        names = {os.path.basename(render_folder(track))}
        if spec.get("fault"):
            names.add(os.path.basename(render_folder(motion.fault_track(track), "fault.")))
        keep[move] = names
    return keep


def quiet(path):
    """Untouched for QUIET_S: left by a run that stopped or finished, never another session's build at work (nor a
    path another run's prune deleted first)."""
    try:
        return time.time() - os.path.getmtime(path) >= QUIET_S
    except FileNotFoundError:
        return False


def drop(path):
    """Deletes a file or a folder; one another run's prune deleted first is gone as wanted."""
    try:
        if os.path.isdir(path):
            shutil.rmtree(path)
        else:
            os.remove(path)
    except FileNotFoundError:
        pass


def unread(name, render):
    """Whether build/loops/<move>/`name` is one no build reads (prune): a frame of a render folder no longer in
    `render` (build/render/<move>/), a cell of the crimson loops or a grey level's (LOOP_CELL), the listing of a film
    from before the listings went into a run's own folder (list.txt), or what a stopped run left (a .part.png, a
    SCRATCH folder)."""
    if name == "list.txt" or name.startswith(SCRATCH) or name.endswith(".part.png"):
        return True
    m = LOOP_CELL.match(name)
    if not m:
        return False
    kind, folder, level = m.groups()
    return kind == "c" or bool(level) or not folder or not os.path.isdir(os.path.join(render, folder))


def prune(baked, moves=None, dry=False):
    """Deletes the renders no motion file of this tree uses any more (David, 2026-10-06: "always make it a standing
    rule to clean up after yourself"): in build/render/<move>/ every folder but the move's current ones, the folder of
    a move no motion file names, and in build/loops/<move>/ and build/stills/ what no build reads (`unread`, and a
    pair's .part.png a stopped run left), and without `moves` a retired move's films (build/loops/ and build/films/)
    and its loop folder in build/loops/ (the film frames of an older layout go when write_loop films the move).
    `moves` limits it to those (a build prunes the moves it built); a move whose file did not bake keeps its folders,
    since its current ones cannot be named. A folder or file touched in the last QUIET_S is left, as another session's
    build from another tree may be writing it. `dry` deletes nothing. Returns the render folders deleted (or, dry, that
    would be)."""
    keep = keep_folders(baked)
    root = os.path.join(BUILD, "render")
    named = {os.path.splitext(os.path.basename(p))[0] for p in figures.motion_paths()}
    gone = []
    for move in sorted(moves or os.listdir(root)):
        d = os.path.join(root, move)
        if not os.path.isdir(d) or (move in named and move not in keep):
            continue
        for name in sorted(os.listdir(d)):
            f = os.path.join(d, name)
            if not os.path.isdir(f) or name in keep.get(move, ()):
                continue
            touched = max([os.path.getmtime(f)] + [os.path.getmtime(e.path) for e in os.scandir(f) if e.is_dir()])
            if time.time() - touched < QUIET_S:
                continue
            gone.append(f)
            if not dry:
                shutil.rmtree(f)
        if dry:
            continue
        cells = os.path.join(BUILD, "loops", move)
        left = [e.path for e in os.scandir(cells) if unread(e.name, d)] if os.path.isdir(cells) else []
        for path in left + glob.glob(os.path.join(BUILD, "stills", f"{move}.*.part.png")):
            if quiet(path):
                drop(path)
        if not os.listdir(d):
            os.rmdir(d)
    for folder in ENCODINGS:                    # a retired move's films, and in build/loops its frames' folder
        films = os.path.join(BUILD, folder)
        for e in list(os.scandir(films)) if moves is None and not dry and os.path.isdir(films) else []:
            if e.name.removesuffix(".mp4") not in named and quiet(e.path):    # a gear variant's name holds a dot
                drop(e.path)
    return gone


def whole(path):
    """A PNG Blender finished writing: it ends on its IEND chunk (a render killed by the timeout or a Ctrl-C can leave
    an empty or cut-off frame, which the next run would otherwise take as rendered)."""
    try:
        with open(path, "rb") as fh:
            fh.seek(-8, os.SEEK_END)
            return fh.read() == b"IEND\xaeB`\x82"
    except OSError:
        return False


def have(d, cloth, frames):
    return all(whole(os.path.join(d, cloth, "main", f"f{i:04d}.png")) and
               whole(os.path.join(d, cloth, "glow", f"glow_{i:04d}.png")) for i in frames)


def render_figure(tp, d, frames, cloths, log):
    todo = [c for c in cloths if not have(d, c, frames)]
    if todo:
        figures.blender([tp, figures.STYLE, d, CELL, "figure", ",".join(map(str, frames)), ",".join(todo)], log)


def render_ground(tp, d, frames, props, log):
    """The ground layers of `frames` (render.py's ground pass), rendered unless every frame's floor, and with `props`
    (the track has some) its prop, is whole: the pass writes a frame's floor before its prop, so a pass cut off between
    the two is rendered again."""
    g = os.path.join(d, "ground")
    if not all(whole(os.path.join(g, "floor", f"g{i:04d}.png")) and
               (not props or whole(os.path.join(g, "prop", f"p{i:04d}.png"))) for i in frames):
        figures.blender([tp, figures.STYLE, g, CELL, "ground", ",".join(map(str, frames))], log)


# ---------------------------------------------------------------- one move

def fig_paths(d, cloth, i):
    return (os.path.join(d, cloth, "main", f"f{i:04d}.png"), os.path.join(d, cloth, "glow", f"glow_{i:04d}.png"))


def render_fault(track):
    """The fault still's renders, in a folder of their own: frame `key` of motion.fault_track, crimson and the five
    grey levels with its floor and prop layers, as a stored frame has them. Returns (the folder, its armour's alpha
    box), the box for the loop's crop to hold (pack.geometry's `hold`), as the still is cut with that crop."""
    still = motion.fault_track(track)
    tp = figures.write_track(still, ".fault")
    d = render_folder(still, "fault.")
    key = track["key"]
    render_figure(tp, d, [key], ["crimson"] + LEVELS, os.path.join(d, "log_figure.txt"))
    render_ground(tp, d, [key], bool(track["props"]), os.path.join(d, "log_ground.txt"))
    return d, P.bbox(fig_paths(d, "crimson", key)[0])


def build_move(move, spec, track, sheets=True):
    """Everything one move contributes: its manifest entry, its sheets (paths), its icon cell and film frames, and
    with a fault the still's render folder, its crimson cell and the phones' pair (write_stills). The crop holds the
    fault still; one whose armour still passes the crop's margin, where the render's own edge cuts it, is left out:
    the move is packed as one without a fault, its report naming the overshoot (`fault_dropped`, render px). Without
    `sheets` (run's films) it stops before the sheets: the entry, what the film needs and the pair, no sheets, icon or
    contact cells."""
    tp = figures.write_track(track)
    d = render_folder(track)
    work = os.path.join(d, "work")
    os.makedirs(work, exist_ok=True)
    n = len(track["frames"])
    frames = list(range(n))
    render_figure(tp, d, frames, ["crimson"], os.path.join(d, "log_dense.txt"))
    key = track["key"]
    # a moving prop (an endless flight of stairs, a belt's slats) moves every frame: its layer joins the selection, so
    # a stored frame never shows the steps where another tick has them, and it fades out at the cell's edges like the
    # floor
    scroll = any(p.get("scroll") or p.get("belt") for p in track["props"])
    # the ground of the key frame (the icon's prop) and, under a moving prop, of every frame (the selection)
    render_ground(tp, d, frames if scroll else [key], bool(track["props"]), os.path.join(d, "log_ground_key.txt"))
    fault, fault_d, fault_box = spec.get("fault"), None, None
    if fault:
        fault_d, fault_box = render_fault(track)
    # the crop: the figure over the whole loop and its fault still, grown toward the props as the camera holds them
    # where the cell has room
    geo = P.geometry([fig_paths(d, "crimson", i)[0] for i in frames], motion.prop_box(track, 2 * CELL),
                     room_2x=0 if track["props"] else 40,             # a few px of floor in front of the feet
                     hold=fault_box)
    w, h = geo["w"], geo["h"]
    tag = sha8(geo)

    def prop_of(i, src=d):
        return os.path.join(src, "ground", "prop", f"p{i:04d}.png") if track["props"] else None
    # 2. selection on figure-only cells (the static grounding never changes which frames are kept)
    metric = list(P.POOL.map(lambda i: P.compose(os.path.join(work, f"m{i:04d}_{tag}.png"), geo, 2,
                                                 figure=fig_paths(d, "crimson", i),
                                                 prop=prop_of(i) if scroll else None, depth=8), frames))
    dpath = os.path.join(work, f"d_{tag}.npy")
    if os.path.exists(dpath):
        dist = np.load(dpath)
    else:
        E, D = P.edges(np.stack([P.load8(p) for p in metric]))
        dist = P.displacement(E, D)
        np.save(dpath, dist)
    glow = np.array([fr["glow"] for fr in track["frames"]], np.float32)
    over = 0
    if fault:
        x0, y0, x1, y1 = fault_box
        cx, cy, cw, ch = geo["crop"]
        over = max(0, max(cx - x0, cy - y0, x1 - cx - cw, y1 - cy - ch) + P.MARGIN_2X)
        if over:
            fault = None                               # the render's edge cuts the still: packed without it
    rel = spec.get("fast_phase", P.REL)
    if fault:
        stored, assign, loose = P.select_spare(dist, glow, w, h, key, rel)
    else:
        stored, assign, loose = P.select_fit(dist, glow, P.capacity(w, h), rel)
    stored_frames = [frames[s] for s in stored]
    key_cell = assign[key]                             # the key pose: the cell shown at the key's tick
    runs = P.rle(assign)
    err = [float(dist[p, stored[assign[p]]]) for p in range(n)]
    # 3. grey cloth and ground for the stored frames, and the key tick's grey cloth for the phones' pair, whose key
    #    side is that tick (as the fault still's pose stands in its place)
    render_figure(tp, d, stored_frames, LEVELS, os.path.join(d, "log_cloth.txt"))
    if fault and key not in stored_frames:
        render_figure(tp, d, [key], LEVELS, os.path.join(d, "log_cloth_key.txt"))
    render_ground(tp, d, stored_frames, bool(track["props"]), os.path.join(d, "log_ground.txt"))
    cells = [(d, i) for i in stored_frames] + ([(fault_d, key)] if fault else [])
    lay = P.layout(len(cells), w, h)
    caps = [track["frames"][sum(r[1] for r in runs[:k])]["phase"] for k in range(len(runs))]
    tick_phase = [fr["phase"] for fr in track["frames"]]
    entry = dict(lay, ticks=n, seq=[x for r in runs for x in r], fade=list(range(1, len(runs) + 1))
                 if spec.get("breathe") else [], mirror=bool(spec.get("mirror")), key=key_cell,
                 caps=caps, key_cap=track.get("key_phase"))
    entry.update(motion.rep_counts(spec.get("rep_ends"), tick_phase))
    report = {"frames": n, "stored": len(stored_frames), "runs": len(runs), "pose_err_max": round(max(err), 2),
              "pose_err_mean": round(float(np.mean(err)), 3)}
    if loose > 1:
        report["loose"] = round(loose, 2)  # the pose tolerance widened to fit one sheet
    out = {"move": move, "entry": entry, "report": report, "tick_phase": tick_phase, "stored": stored_frames,
           "geo": geo, "d": d, "key": key}
    if over:
        report["fault_dropped"] = over
    if fault:
        entry.update(fault=len(stored_frames), fault_cap=fault["caption"])
        report["fault"] = fault["caption"]
        if P.grows(len(stored_frames), w, h):
            report["sheet_grew"] = lay["sheet"]        # no stored frame could make room for the still
        out["fault_d"] = fault_d
        write_stills(move, out)
    if not sheets:
        return out
    win = P.window_png(w, h, os.path.join(work, f"window_{w}x{h}.png"))

    def cell(cloth, i, depth=16, scale=1, src=d):
        """Frame i of render folder `src` (the loop's, or the fault still's) as a cell."""
        path = os.path.join(src, "work", f"c_{cloth}_{i:04d}_{tag}_{scale}_{depth}.png")
        os.makedirs(os.path.dirname(path), exist_ok=True)
        return P.compose(path, geo, scale, figure=fig_paths(src, cloth, i),
                         floor=os.path.join(src, "ground", "floor", f"g{i:04d}.png"), prop=prop_of(i, src),
                         window=win if scale == 1 else None, depth=depth, fade_prop=scroll)
    # 4. the split, the fault still as the last cell
    Y = {lv: np.stack(list(P.POOL.map(lambda c, lv=lv: P.load16(cell(lv, c[1], src=c[0])), cells))) for lv in LEVELS}
    base, tint = P.split(Y)
    base8, tint8 = [P.q8(c) for c in base], [P.q8(c) for c in tint]
    # 5. the sheets
    sdir = os.path.join(work, "sheets")
    os.makedirs(sdir, exist_ok=True)
    base_png, base_q = os.path.join(sdir, "base.png"), os.path.join(sdir, "base.q.png")
    Image.fromarray(P.pack_sheet(base8, lay)).save(base_png, compress_level=1)
    colours, scores = P.encode_to_target(base_png, base8, lay, base_q, os.path.join(sdir, "score"))
    tint_png, tint_q = os.path.join(sdir, "tint.png"), os.path.join(sdir, "tint.q.png")
    Image.fromarray(P.pack_sheet(tint8, lay, bg=(0, 0, 0))).save(tint_png, compress_level=1)
    P.encode_tint(tint_png, tint_q)
    report.update(colours=colours, ssim2_min=round(min(scores), 2))
    out.update(base=base_q, tint=tint_q, icon=icon(d, key, track),
               crimson=dict(zip(stored_frames, P.POOL.map(lambda i: cell("crimson", i, depth=8), stored_frames))))
    if fault:
        out["fault_cell"] = cell("crimson", key, depth=8, src=fault_d)
    return out


# ---------------------------------------------------------------- icons

def rgba16(path):
    w, h = map(int, P.sh("magick", "identify", "-format", "%w %h", path).split())
    raw = subprocess.run(["magick", path, "-depth", "16", "-endian", "MSB", "rgba:-"], capture_output=True,
                         check=True).stdout
    return np.frombuffer(raw, ">u2").reshape(h, w, 4).astype(np.float64) / 65535.0


def icon(d, key, track):
    """The key pose as a 64 px icon cell: the crimson figure fitted into 60 px, no card, no floor, no glow layer; a move
    with a prop keeps the part of it within 40-80 render px of the figure (the wall face by the hands, the seat under a
    foot), laid with the figure as a cell's is (figures.layers: over it where it stands in front), so the pose reads.
    (The phones' stills are cut from their loops: publish.py.)"""
    main = fig_paths(d, "crimson", key)[0]
    fig = rgba16(main)
    if track["props"]:
        h, w = fig.shape[:2]
        t = np.clip((80.0 - ndimage.distance_transform_edt(fig[..., 3] < 0.05)) / 40.0, 0, 1)
        near = np.ones((h, w, 4))                     # the prop's alpha falls off with the distance from the figure
        near[..., 3] = t * t * (3 - 2 * t)
        raw = ["-size", f"{w}x{h}", "-depth", "16", "-endian", "MSB"]
        prop = [os.path.join(d, "ground", "prop", f"p{key:04d}.png"), "(", *raw, "rgba:-", ")", "-compose", "DstIn",
                "-composite"]
        laid = subprocess.run(["magick", *raw, "xc:none", *figures.layers(prop=prop, main=[main]), "rgba:-"],
                              input=(near * 65535).round().astype(">u2").tobytes(), capture_output=True,
                              check=True).stdout
        fig = np.frombuffer(laid, ">u2").reshape(h, w, 4).astype(np.float64) / 65535.0
    fig[..., :3] *= fig[..., 3:4]
    a = fig[..., 3]
    ys, xs = np.nonzero(a > 0.004)
    x0, x1, y0, y1 = xs.min(), xs.max() + 1, ys.min(), ys.max() + 1
    s = max(x1 - x0, y1 - y0)
    bx, by = int(round((x0 + x1 - s) / 2)), int(round((y0 + y1 - s) / 2))
    sq = np.zeros((s, s, 4))
    sx0, sy0, sx1, sy1 = max(0, bx), max(0, by), min(fig.shape[1], bx + s), min(fig.shape[0], by + s)
    sq[sy0 - by:sy1 - by, sx0 - bx:sx1 - bx] = fig[sy0:sy1, sx0:sx1]
    small = np.clip(np.stack([np.asarray(Image.fromarray(sq[..., c].astype(np.float32), "F")
                                         .resize((ICON_INNER, ICON_INNER), Image.LANCZOS)) for c in range(4)], -1),
                    0, 1)
    rgb = np.where(small[..., 3:4] > 1e-4, small[..., :3] / np.maximum(small[..., 3:4], 1e-4), 0)
    out = np.zeros((ICON_CELL, ICON_CELL, 4))
    o = (ICON_CELL - ICON_INNER) // 2
    out[o:o + ICON_INNER, o:o + ICON_INNER, :3] = np.clip(rgb, 0, 1)
    out[o:o + ICON_INNER, o:o + ICON_INNER, 3] = small[..., 3]
    return (out * 255).round().astype(np.uint8)


def write_icons(cells, ids):
    """Media/MoveIcons.png and, past 256 cells, Media/MoveIcons2.png, past 512 Media/MoveIcons3.png (16 x 16 cells of
    64 px each, a 256-colour palette PNG) and FigureIcons.lua, the cell of each move id, its index running on from one
    sheet into the next (generated, never hand-edited)."""
    order = sorted(cells, key=lambda m: ids[m])
    per = ICON_GRID * ICON_GRID
    if len(order) > per * len(ICONS_PNG):
        raise SystemExit(f"{len(order)} key-pose icons: the sheets hold {per * len(ICONS_PNG)} (build.py ICONS_PNG)")
    for n, path in enumerate(ICONS_PNG):
        part = order[n * per:(n + 1) * per]
        if not part:
            break
        sheet = np.zeros((ICON_GRID * ICON_CELL, ICON_GRID * ICON_CELL, 4), np.uint8)
        for k, m in enumerate(part):
            r, q = divmod(k, ICON_GRID)
            sheet[r * ICON_CELL:(r + 1) * ICON_CELL, q * ICON_CELL:(q + 1) * ICON_CELL] = cells[m]
        raw = os.path.join(BUILD, f"icons_rgba{n + 1}.png")
        Image.fromarray(sheet, "RGBA").save(raw)
        P.sh("pngquant", "--force", "--speed", "1", "256", "--output", path, raw)
        P.sh("oxipng", "-q", "-o", "4", "--strip", "all", path)
    lines = [f"\t[{ids[m]}] = {k}, -- {m}" for k, m in enumerate(order)]
    with open(ICONS_LUA, "w") as fh:
        fh.write("-- Generated by tools/figures (make figures): the key-pose icon cell of each move id in\n"
                 "-- Media/MoveIcons.png, cells 0-255, Media/MoveIcons2.png, cells 256-511, and Media/MoveIcons3.png,\n"
                 "-- cells 512-767 (16 x 16 cells of 64 px each, row-major). Never hand-edited.\n"
                 "local _, ns = ...\n\nns.FigureIcons = {\n" + "\n".join(lines) + "\n}\n")


# ---------------------------------------------------------------- variants


def variant_entry(spec, ref_entry, tick_phase, ends=None):
    """A timing variant's table on its reference move's cells: `phases` retimes each named phase to its seconds (the
    ticks resampled), `pause` holds the phase's frame at `at` of it for `s` seconds; 0 bytes of art. The key pose, the
    fault still and the phases that end a rep (`ends`, the reference's rep_ends) are its reference's."""
    shown = P.shown_at_ticks([ref_entry["seq"][i:i + 2] for i in range(0, len(ref_entry["seq"]), 2)])
    phase = list(tick_phase)
    for ph, secs in (spec.get("phases") or {}).items():
        idx = [t for t, p in enumerate(phase) if p == ph]
        a, b = idx[0], idx[-1]
        assert idx == list(range(a, b + 1)), f"{spec['move']}: phase {ph} is not one run of ticks"
        seg = shown[a:b + 1]
        m = max(1, round(secs * P.HZ))
        shown = shown[:a] + [seg[min(len(seg) - 1, int(i * len(seg) / m))] for i in range(m)] + shown[b + 1:]
        phase = phase[:a] + [ph] * m + phase[b + 1:]
    pause = spec.get("pause")
    if pause:
        idx = [t for t, p in enumerate(phase) if p == pause["phase"]]
        at = idx[0] + round(pause.get("at", 0.5) * (len(idx) - 1))
        m = round(pause["s"] * P.HZ)
        shown = shown[:at] + [shown[at]] * m + shown[at:]
        phase = phase[:at] + [pause["phase"]] * m + phase[at:]
    runs = P.rle(shown)
    starts = np.cumsum([0] + [n for _, n in runs])[:-1]
    entry = {"ref": spec["variant_of"], "ticks": len(shown), "seq": [x for r in runs for x in r],
             "fade": list(ref_entry["fade"]), "mirror": bool(spec.get("mirror")), "key": ref_entry["key"],
             "caps": [phase[s] for s in starts], "key_cap": ref_entry["key_cap"]}
    entry.update({k: ref_entry[k] for k in ("fault", "fault_cap") if k in ref_entry})   # its reference's fault still
    entry.update(motion.rep_counts(ends, phase))
    return entry


# ---------------------------------------------------------------- the pack

def write_pack(entries, files):
    """QuestPump_FigArmor/: s/<move>.<sha8>.png and s/<move>.t.<sha8>.png, pack.json, Manifest.lua and the TOC, the
    content version raised when the moves' data changed. Sheets no entry names are listed (prune them with --prune)."""
    # QuestPump's Interface line (the packager uploads every TOC's values) and its icon (the AddOns list's row), read
    # before anything is written so a TOC without them stops the build with the pack untouched
    with open(os.path.join(ROOT, "QuestPump.toc"), encoding="utf-8") as fh:
        main = [l.strip() for l in fh]
    interface, icon = (next((l for l in main if l.startswith(k)), None) for k in ("## Interface:", "## IconTexture:"))
    if not interface or not icon:
        raise SystemExit("QuestPump.toc lacks its ## Interface: or ## IconTexture: line; the pack's TOC copies both")
    sdir = os.path.join(PACK_DIR, "s")
    os.makedirs(sdir, exist_ok=True)
    moves = {}
    for m, e in entries.items():
        e = dict(e)
        if m in files:
            for k, src, ext in (("file", files[m][0], "png"), ("tint", files[m][1], "png")):
                h = P.sha256(src)[:8]
                rel = f"s/{m}.{h}.{ext}" if k == "file" else f"s/{m}.t.{h}.{ext}"
                dst = os.path.join(PACK_DIR, rel)
                if not os.path.exists(dst):
                    shutil.copyfile(src, dst)
                    NEW_FILES.append(rel)
                e[k] = rel
        moves[m] = e
    digest = sha8(moves)
    old = {}
    pj = os.path.join(PACK_DIR, "pack.json")
    if os.path.exists(pj):
        with open(pj) as fh:
            old = json.load(fh)
    version = old.get("pack", 0) + (0 if old.get("digest") == digest else 1)
    data = {"figure": "armor", "pack": version, "digest": digest, "hz": P.HZ, "box": P.BOX, "bg": list(P.BG),
            "path": f"Interface\\AddOns\\{PACK_NAME}\\", "moves": moves}
    lua = json.loads(json.dumps(data))
    for e in lua["moves"].values():
        for k in ("file", "tint"):
            if k in e:
                e[k] = e[k].replace("/", "\\")
    with open(os.path.join(PACK_DIR, "Manifest.lua"), "w") as fh:
        fh.write("-- Generated from pack.json by tools/figures (make figures); never hand-edited. Cells are 0-based;\n"
                 "-- seq = { cell, ticks, ... } at hz ticks per second; fade lists the 1-based runs that cross-fade\n"
                 "-- into the next; caps the phase of each run; mirror flips the cell; a variant draws on its ref's\n"
                 "-- sheet. key is the key pose's cell and key_cap its phase; fault, where a move has one, is the\n"
                 "-- cell of its fault still (the sheet's last, never in seq) and fault_cap its caption's key; counts,\n"
                 "-- where a loop shows more than one rep, the tick at which each rep but the last ends.\n"
                 f"QuestPumpFigures = QuestPumpFigures or {{}}\nQuestPumpFigures.armor = {P.lua_value(lua)}\n")
    with open(pj, "w") as fh:
        json.dump(data, fh, indent=1, sort_keys=True)
        fh.write("\n")
    with open(os.path.join(PACK_DIR, f"{PACK_NAME}.toc"), "w") as fh:
        fh.write(f"{interface}\n## Title: QuestPump Figures\n"
                 "## Notes: The animated move figures for QuestPump (loaded on demand).\n"
                 f"{icon}\n"
                 "## Group: QuestPump\n## LoadOnDemand: 1\n## Dependencies: QuestPump\n"
                 f"## Version: @project-version@\n## X-QuestPump-Pack: {version}\n\nManifest.lua\n")
    named = {os.path.basename(e[k]) for e in moves.values() for k in ("file", "tint") if k in e}
    orphans = sorted(set(os.listdir(sdir)) - named)
    if orphans:
        if "--prune" in sys.argv:
            for o in orphans:
                os.remove(os.path.join(sdir, o))
            print("pruned", len(orphans), "old sheets")
        else:
            print("old sheets no move names (make figures ARGS=--prune removes them):", ", ".join(orphans))
    return data


# ---------------------------------------------------------------- films, the pairs, contact sheets

def scratch(move):
    """A folder of the calling run's own in build/loops/<move>/ (SCRATCH) for what it reads once and deletes: the
    build folder is a cache that runs share, and a run must never read a cell another is writing or deleting."""
    d = os.path.join(BUILD, "loops", move)
    os.makedirs(d, exist_ok=True)
    return tempfile.mkdtemp(prefix=SCRATCH, dir=d)


def loop_cell(geo, src, i, cloth, into, px=LOOP_PX):
    """Frame i of render folder `src` as the phones see it: the figure in `cloth` (crimson, or a grey level at 16 bits
    for the split) on the card at `px` (the film's LOOP_PX, an extra's still publish.EXTRA_SIDE) for the 192 px box, its
    floor under it and its prop over it (figures.layers), cut with the loop's crop (`geo`, a move's own), so every such
    cell of a move registers with the others. Written into `into`, a folder of the caller's own (scratch)."""
    scale = px / P.BOX
    geo = dict(geo, w=round(geo["w"] * scale / 2) * 2, h=round(geo["h"] * scale / 2) * 2)
    p = os.path.join(into, f"{i:04d}_{os.path.basename(src)}_{cloth}.png")
    g = os.path.join(src, "ground")
    prop = os.path.join(g, "prop", f"p{i:04d}.png")
    return P.compose(p, geo, 1, figure=fig_paths(src, cloth, i), floor=os.path.join(g, "floor", f"g{i:04d}.png"),
                     prop=prop if os.path.exists(prop) else None, depth=8 if cloth == "crimson" else 16)


def loop_split(move, geo, src, i, px=LOOP_PX):
    """Frame i of render folder `src` at the loop's size (or `px`, loop_cell's) split into base and tint (pack.split
    over its five grey levels, as the sheets' cells are): two 8-bit RGB arrays. The level cells live only for the
    split, in a folder of this call's own: the film frame, the pair's tint or the extra's still keeps it (at 16 bits
    they would add about 40 MB a move to the cache)."""
    d = scratch(move)
    try:
        base, tint = P.split({lv: P.load16(loop_cell(geo, src, i, lv, d, px)) for lv in LEVELS})
    finally:
        shutil.rmtree(d, ignore_errors=True)
    return P.q8(base), P.q8(tint)


def film_frame(move, geo, src, i):
    """Frame i of render folder `src` as a frame of the phones' film: the base on the card in the top LOOP_PX square,
    the tint on black in the square from row publish.TINT, black between, each centred as the loop's cell. In
    build/loops/<move>/, named by its render folder and size, written through a .part.png of this process's own (two
    runs may make one frame at once)."""
    size = dict(geo, film=[LOOP_PX, publish.TINT])
    p = os.path.join(BUILD, "loops", move, f"f{i:04d}_{os.path.basename(src)}_{sha8(size)}.png")
    if not os.path.exists(p):
        base, tint = loop_split(move, geo, src, i)
        h, w = base.shape[:2]
        y, x = (LOOP_PX - h) // 2, (LOOP_PX - w) // 2
        frame = np.zeros((publish.TINT + LOOP_PX, LOOP_PX, 3), np.uint8)
        frame[:LOOP_PX] = P.BG
        frame[y:y + h, x:x + w] = base
        frame[publish.TINT + y:publish.TINT + y + h, x:x + w] = tint
        part = f"{p}.{os.getpid()}.part.png"
        Image.fromarray(frame).save(part, compress_level=1)
        os.replace(part, p)
    return p


# The films' colour, tagged as what it is: ffmpeg's RGB to YUV with BT.601's matrix in the limited range (its own
# default, made explicit), named so in the HEVC VUI and the MP4's colr box with BT.709's primaries and transfer, so no
# decoder guesses the matrix from the frame size (AOSP ColorUtils' default takes BT.601 only up to 720 x 576, BT.709
# at the stacked film's size) and every one reads the frames back as publish.py's ffmpeg does (ffprobe, 2026-10-06:
# the tags in the stream, the pixels the untagged encode's)
FILM_COLOUR = ("scale=out_color_matrix=bt601:out_range=tv,format=yuv420p,"
               "setparams=colorspace=smpte170m:color_primaries=bt709:color_trc=bt709:range=tv")
# The two encodings of every film, by the folder each goes to (build/<folder>/<move>.mp4, publish.py's
# contract/figures/<folder>/): the phones' HEVC (hvc1) at crf 28 with x265's dark-biased adaptive quantisation
# (aq-mode 3), which keeps the encoder's ghosts of a moving cloth out of the tint half, where a phone would light them
# in the class colour (README, "The films"), and its 64-px coding tree blocks pinned (ctu 64, its default), on which
# publish.py lays the tint; and the desktop app's H.264 (avc1, slice 35) at crf FILM_H264_CRF with x264's own aq-mode
# 3, its index first in the file (faststart) for the web view's player.
FILM_H264_CRF = "24"
ENCODINGS = {
    publish.LOOPS: ["-c:v", "libx265", "-crf", "28", "-x265-params", "aq-mode=3:ctu=64", "-tag:v", "hvc1"],
    publish.FILMS: ["-c:v", "libx264", "-crf", FILM_H264_CRF, "-x264-params", "aq-mode=3", "-tag:v", "avc1",
                    "-movflags", "+faststart"],
}


def encode(frames, tick, into):
    """The film of `frames` played by `tick` (each tick's index into `frames`) at P.HZ in FILM_COLOUR, once in each of
    ENCODINGS, into `into` (a folder of the caller's own): {folder: its file}. Raises when ffmpeg fails."""
    listing = os.path.join(into, "list.txt")
    with open(listing, "w") as fh:
        for c in tick:
            fh.write(f"file '{frames[c]}'\nduration {1 / P.HZ:.6f}\n")
    made = {}
    for folder, codec in ENCODINGS.items():
        made[folder] = os.path.join(into, f"{folder}.mp4")
        subprocess.run(["ffmpeg", "-y", "-loglevel", "error", "-f", "concat", "-safe", "0", "-i", listing,
                        "-vf", f"fps={P.HZ},{FILM_COLOUR}", *codec, made[folder]], check=True)
    return made


def write_loop(move, out, entry):
    """A move's films (slice 29; a crimson loop from slice 11): one rep, the stored frames of `out` (the build of the
    move whose cells `entry` plays: the move's own, or a timing variant's reference move's) played by `entry`'s run
    table at 30 fps, each a film_frame (the base over the tint, publish.py's layout), in both ENCODINGS, which replace
    the last good pair of `move`'s films only once both are written. So a timing variant's film is its own table over
    its reference's frames, at the variant's tempo (2026-10-09). Then the frames of that render folder the films do not
    use (another layout's, an older build's, which prune cannot tell from this one's) go, but for one touched in the
    last QUIET_S."""
    tick = P.shown_at_ticks([entry["seq"][i:i + 2] for i in range(0, len(entry["seq"]), 2)])
    d = os.path.join(BUILD, "loops", out["move"])
    frames = list(P.POOL.map(lambda i: film_frame(out["move"], out["geo"], out["d"], i), out["stored"]))
    own = scratch(out["move"])
    try:
        for folder, part in encode(frames, tick, own).items():
            os.makedirs(os.path.join(BUILD, folder), exist_ok=True)
            os.replace(part, os.path.join(BUILD, folder, f"{move}.mp4"))
    finally:
        shutil.rmtree(own, ignore_errors=True)
    folder, used = os.path.basename(out["d"]), set(frames)
    for e in os.scandir(d):
        m = LOOP_CELL.match(e.name)
        if m and m.group(1) == "f" and m.group(2) == folder and e.path not in used and quiet(e.path):
            drop(e.path)


def write_stills(move, out):
    """The phones' right-versus-wrong pair of a move with a fault (slice 22): build/stills/<move>.key.png, the key
    pose, and <move>.fault.png, the fault still, each as a cell of the loop is (loop_cell), so the two register with
    each other and with the loop; a 256-colour palette PNG as the icon sheets are (about a third of the truecolour
    bytes, SSIMULACRA2 about 90 on the pilot's stills, 2026-10-04). Beside each its tint (slice 29), <move>.key.t.png
    and <move>.fault.t.png: the same cell's tint (loop_split) written as publish.write_tint writes a still's, with
    which a phone draws the pair in the class colour as the game's View:ShowPair does. Each is written through a
    .part.png of this process's own, and the crimson cells in a folder of this call's own."""
    d = os.path.join(BUILD, "stills")
    os.makedirs(d, exist_ok=True)
    cells = scratch(move)
    part = f".{os.getpid()}.part.png"
    done = []
    try:
        for side, src in (("key", out["d"]), ("fault", out["fault_d"])):
            path = os.path.join(d, f"{move}.{side}.png")
            P.sh("pngquant", "--force", "--speed", "1", "256", "--output", path + part,
                 loop_cell(out["geo"], src, out["key"], "crimson", cells))
            P.sh("oxipng", "-q", "-o", "4", "--strip", "all", path + part)
            tint = os.path.join(d, f"{move}.{side}{publish.TINTED}")
            publish.write_tint(loop_split(move, out["geo"], src, out["key"])[1], tint + part)
            done += [path, tint]
    finally:
        shutil.rmtree(cells, ignore_errors=True)
    for path in done:                  # the pair replaces the old one only once all four are written
        os.replace(path + part, path)


def build_extra(name, track):
    """An extra (motion.py `extra`; slice 45): its key pose alone, rendered in crimson and the five grey levels with its
    floor and props, cut with a crop of that frame (the figure with its held items, a few px of floor before the feet
    where it has no prop, grown toward the props as a move's crop is) into publish.EXTRA_SIDE's square, split into base
    and tint (pack.split, as a film's frame is) and written as the still the apps draw: build/stills/<name>.png in
    crimson with the card keyed out, and its tint <name>.t.png (publish.write_still), which replace the old two only
    once both are written. No pack entry, icon, film or contact sheet. Returns its report line."""
    tp = figures.write_track(track)
    d = render_folder(track)
    key = track["key"]
    render_figure(tp, d, [key], ["crimson"] + LEVELS, os.path.join(d, "log_figure.txt"))
    render_ground(tp, d, [key], bool(track["props"]), os.path.join(d, "log_ground.txt"))
    geo = P.geometry([fig_paths(d, "crimson", key)[0]], motion.prop_box(track, 2 * CELL),
                     room_2x=0 if track["props"] else 40)
    side = publish.EXTRA_SIDE
    base, tint = loop_split(name, geo, d, key, side)
    h, w = base.shape[:2]
    y, x = (side - h) // 2, (side - w) // 2
    frame = np.empty((side, side, 3), np.uint8)
    frame[:] = P.BG
    frame[y:y + h, x:x + w] = base
    light = np.zeros_like(frame)
    light[y:y + h, x:x + w] = tint
    shown = publish.composite(frame[None], light[None], publish.crimson())[0]
    own = scratch(name)
    try:
        publish.write_still(shown, light, np.array(P.BG, np.int16), os.path.join(own, name + ".png"))
        out = os.path.join(BUILD, publish.STILLS)
        os.makedirs(out, exist_ok=True)
        for f in (name + ".png", name + publish.TINTED):
            os.replace(os.path.join(own, f), os.path.join(out, f))
    finally:
        shutil.rmtree(own, ignore_errors=True)
    return {"move": name, "extra": True, "still": side, "cell": [w, h]}


def contact_sheet(move, out, entry):
    """Every stored cell of a move, crimson on the card: what Claude reads before David's review. A move with a
    fault ends with a row of its own, the pair as the New-move view shows it: the key pose's cell framed green, then
    the fault still's framed red."""
    cells = [P.load8(out["crimson"][i]) for i in out["stored"]]
    w, h = entry["size"]
    cols = min(8, len(cells))
    rows = math.ceil(len(cells) / cols)
    pair = [(cells[entry["key"]], (70, 150, 70)), (P.load8(out["fault_cell"]), (190, 60, 50))] if "fault" in entry \
        else []
    img = np.empty(((rows + bool(pair)) * (h + 4), max(cols, len(pair)) * (w + 4), 3), np.uint8)
    img[:] = (40, 36, 30)
    for k, c in enumerate(cells):
        r, q = divmod(k, cols)
        img[r * (h + 4) + 2:r * (h + 4) + 2 + h, q * (w + 4) + 2:q * (w + 4) + 2 + w] = c
    for q, (c, frame) in enumerate(pair):
        img[rows * (h + 4):(rows + 1) * (h + 4), q * (w + 4):(q + 1) * (w + 4)] = frame
        img[rows * (h + 4) + 2:rows * (h + 4) + 2 + h, q * (w + 4) + 2:q * (w + 4) + 2 + w] = c
    path = os.path.join(BUILD, "contact", f"{move}.png")
    os.makedirs(os.path.dirname(path), exist_ok=True)
    Image.fromarray(img).save(path)
    return path


# ---------------------------------------------------------------- run

def run(moves=None, films=False):
    """The build of `moves` (default: every motion file; a timing variant named builds with its reference move, whose
    frames it plays). Returns the moves it filmed (their files in the build folder are this build's, publish.py takes
    only theirs: a rebuilt move's and each of its timing variants', whose film is its own table over the reference's
    frames) and a line per move that needs another look: a move left unbuilt by its checks or by a failure in its
    build (its pack entry kept as it was), one packed without its fault still, one whose film or contact sheet failed,
    and a timing variant left out of the pack because its reference was rebuilt while its own file does not load,
    fails its check or does not fit the new table (an old table would point at the old cells; without an entry the
    game shows the key-pose icon). With `films` it writes only the films and pairs (the module's docstring), each
    move's films, its variants' with them, as soon as its build returns. An extra (motion.py `extra`) builds its still
    alone (build_extra) and is returned with the moves filmed, for publish.py to take; a run that names extras alone
    leaves the game's pack, icons, fixture and review page as they are, and `films` builds none (an extra has no
    film)."""
    specs, broken = figures.load_all()                    # every motion file: the variants and the pack's moves
    old = {}                                              # the pack as it stands
    pj = os.path.join(PACK_DIR, "pack.json")
    if os.path.exists(pj):
        with open(pj) as fh:
            old = json.load(fh)["moves"]
    # each timing variant's reference move (None for every other move), from its file or, where it does not load, from
    # the pack
    ref_of = {m: specs[m].get("variant_of") if m in specs else old.get(m, {}).get("ref")
              for m in set(specs) | set(broken)}
    if moves:
        named = moves.split(",")
        moves = ",".join(dict.fromkeys(named + [ref_of[m] for m in named if ref_of.get(m)]))
    baked, problems = figures.check(moves)
    todo = sorted(m for m in baked if m not in problems)
    extras = [m for m in todo if baked[m][0].get("extra")]
    todo = [m for m in todo if m not in extras]
    # a run that names extras alone never touches the game's pack, icons, fixture or review page
    game = moves is None or any(not specs.get(m, {}).get("extra") for m in moves.split(","))
    ids = moves_json()
    left = {m: f"not built, its motion check fails: {found[0]}" for m, found in problems.items()}
    results, filmed = {}, []        # filmed: whole in the build folder (a failed film may be another run's)
    stills = []                     # the extras whose stills this run wrote
    if films:
        left.update({m: "not built: an extra has no film (make figures MOVES=... builds its still)"
                     for m in extras if moves})
        extras = []

    def variants(ref):
        return sorted(m for m, r in ref_of.items() if r == ref)

    def table(m):
        """A timing variant's table on its rebuilt reference's cells (variant_entry); raises when its file does not
        load, fails its check or does not fit."""
        if m in problems or m in broken:
            raise ValueError((problems.get(m) or broken[m])[0])
        ref = results[ref_of[m]]
        return variant_entry(specs[m], ref["entry"], ref["tick_phase"], specs[ref_of[m]].get("rep_ends"))

    def film(move, out, entry):
        try:
            write_loop(move, out, entry)
            filmed.append(move)
        except Exception as err:
            left[move] = f"its film failed: {type(err).__name__}: {err}"

    def packed(move):
        return {k: v for k, v in old.get(move, {}).items() if k not in ("file", "tint")}

    with ThreadPoolExecutor(figures.JOBS) as pool:
        running = {m: pool.submit(build_move, m, *baked[m], sheets=not films) for m in todo}
        posing = {m: pool.submit(build_extra, m, baked[m][1]) for m in extras}
        for m in extras:
            try:
                print(json.dumps(posing[m].result()), flush=True)
                stills.append(m)
            except Exception as err:
                left[m] = f"its still was not built: {type(err).__name__}: {err}"
        for move in todo:
            try:
                out = running[move].result()
            except Exception as err:
                left[move] = f"not built, its pack entry kept: {type(err).__name__}: {err}"
                continue
            results[move] = out
            print(json.dumps({"move": move, **out["report"]}), flush=True)
            if out["report"].get("fault_dropped"):
                left[move] = (f"packed without its fault still: its armour passes the loop's crop margin by "
                              f"{out['report']['fault_dropped']} render px")
            if not films:
                continue
            if json.loads(json.dumps(out["entry"])) != packed(move):
                left[move] = ("not filmed: its frames would change its pack entry (make figures rebuilds both, as "
                              "it must after a style, Blender, ground or fault-pose change, which this cannot see)")
                continue
            film(move, out, out["entry"])
            for v in variants(move):
                try:
                    entry = table(v)
                except Exception as err:
                    left[v] = f"not filmed, its table could not follow {move}: {err}"
                    continue
                if json.loads(json.dumps(entry)) != packed(v):
                    left[v] = "not filmed: its table would change its pack entry (make figures rebuilds both)"
                    continue
                film(v, out, entry)
    pruned = len(prune(baked, sorted(results) + stills))
    if pruned:
        print(f"pruned {pruned} old render {'folder' if pruned == 1 else 'folders'}")
    if films or not game:
        return sorted(filmed + stills), [f"{m}: {line}" for m, line in sorted(left.items())]
    # the pack keeps every move already in it that this run did not rebuild
    entries = {m: e for m, e in old.items()}
    files = {}
    for m, out in results.items():
        entries[m] = out["entry"]
        files[m] = (out["base"], out["tint"])
    derived = []                                          # the timing variants of the rebuilt moves
    for m in sorted(ref_of):
        if ref_of[m] not in results:
            continue
        try:
            entries[m] = table(m)
            derived.append(m)
        except Exception as err:
            entries.pop(m, None)
            left[m] = f"left out of the pack, {ref_of[m]} was rebuilt and its table could not follow: {err}"
    for m in list(entries):
        if m not in specs and m not in broken:            # its motion file is gone
            del entries[m]
    for m in files:                                       # an unchanged entry keeps its old file names
        entries[m].pop("file", None)
        entries[m].pop("tint", None)
    for m, e in old.items():
        if m in entries and m not in files and "file" in e:
            entries[m]["file"], entries[m]["tint"] = e["file"], e["tint"]
    data = write_pack(entries, files)
    icons_dir = os.path.join(BUILD, "icons")
    os.makedirs(icons_dir, exist_ok=True)
    for m, out in results.items():
        Image.fromarray(out["icon"], "RGBA").save(os.path.join(icons_dir, f"{m}.png"))
    cells = {}
    for m in sorted(ref_of):                              # a library move's icon is its sheet owner's
        p = os.path.join(icons_dir, f"{ref_of[m] or m}.png")
        if m in ids and os.path.exists(p):
            cells[m] = np.asarray(Image.open(p).convert("RGBA"))
    NEW_FILES.extend(os.path.relpath(p, ROOT) for p in ICONS_PNG if not os.path.exists(p))
    write_icons(cells, ids)
    for m, out in results.items():
        try:
            write_loop(m, out, data["moves"][m])
            contact_sheet(m, out, data["moves"][m])
            filmed.append(m)
        except Exception as err:
            left[m] = f"packed, but its film or contact sheet failed: {type(err).__name__}: {err}"
    for m in derived:                                     # their own tables over their references' frames
        film(m, results[ref_of[m]], data["moves"][m])
    steps.write_fixture(data)
    import review
    review.write(data, {m: out["report"] for m, out in results.items()}, specs)
    print(json.dumps({"pack": data["pack"], "moves": len(data["moves"]),
                      "bytes": sum(os.path.getsize(os.path.join(PACK_DIR, e[k])) for e in data["moves"].values()
                                   for k in ("file", "tint") if k in e)}))
    if NEW_FILES:
        print(f"{len(NEW_FILES)} new files in the game's folders: quit and start the game client again to see them "
              "(a /reload shows the green missing-texture fill)")
    return sorted(filmed + stills), [f"{m}: {line}" for m, line in sorted(left.items())]
