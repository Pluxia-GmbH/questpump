"""The pack's image work (docs/plan.md slice 2, "The pack" and "The class colour"; ported from the scratchpad
prototype's v4/pack/packlib.py, v4/tint/tintlib.py and v4/props/build_grounded.py on 2026-09-28): compositing the
renders into final-size cells, the pose distance between frames measured on their pixels, pose-indexed frame
selection with a run-length playback table, sheet packing, the palette PNG searched to a quality bar, the base and
tint split of the cloth, and the manifest's Lua. Deterministic: sorted inputs, stable tie-breaks, fixed encoder
settings.

Runs under tools/figures/.venv (numpy, scipy, pillow); ImageMagick, ssimulacra2, pngquant and oxipng from Homebrew,
(setup.sh checks for them).
"""
import hashlib
import json
import math
import os
import re
import subprocess
from concurrent.futures import ThreadPoolExecutor

import numpy as np
from PIL import Image
from scipy import ndimage

import figures

HERE = os.path.dirname(os.path.abspath(__file__))
with open(os.path.join(HERE, "style.json")) as _fh:
    BG_HEX = json.load(_fh)["card"]                 # the card every cell is composited on
BG = tuple(int(BG_HEX[i:i + 2], 16) for i in (1, 3, 5))
BOX = 192            # the in-game figure box (px)
MARGIN_2X = 12       # room around the figure's union alpha box at render size (the bloom)
HZ = 30
PAD = 2              # card colour around every cell's content
MAX_SHEET = 2048
EDGE_T = 0.10        # Sobel magnitude (luma 0..1) that counts as a drawn edge
EDGE_K = 8           # a frame pair still matches when at most this many edge pixels moved further than eps
EPS, REL, GAMMA = 1.5, 1.0, 0.15    # pose tolerance (px), fast-phase allowance (x motion per tick), glow match
TARGET = 82.0        # worst stored cell's SSIMULACRA2 in the 192 px box
PALETTES = (64, 96, 128, 160, 192, 224, 256)   # the base sheet's palette sizes, tried smallest first
LEVELS = {"g000": 0.0, "g025": 0.25, "g050": 0.5, "g075": 0.75, "g100": 1.0}   # grey cloth, sRGB-encoded
FADE_PX, FADE_BOTTOM_PX = 10, 4     # the floor layer fades out at the cell's side and top edges, and its bottom
PROP_MARGIN_2X = 12
POOL = ThreadPoolExecutor(max(4, (os.cpu_count() or 8) - 2))


def sh(*a):
    return subprocess.run([str(x) for x in a], check=True, capture_output=True, text=True).stdout


def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as fh:
        for b in iter(lambda: fh.read(1 << 20), b""):
            h.update(b)
    return h.hexdigest()


# ---------------------------------------------------------------- crop and compositing

def bbox(path):
    """The alpha box (x0, y0, x1, y1) of a render, or None when it is empty."""
    g = sh("magick", path, "-alpha", "extract", "-threshold", "1%", "-format", "%@", "info:").strip()
    wh, x, y = g.split("+")[0], int(g.split("+")[1]), int(g.split("+")[2])
    w, h = map(int, wh.split("x"))
    return (x, y, x + w, y + h) if w > 1 and h > 1 else None


def toward(lo, hi, want_lo, want_hi, side, limit, floor_first):
    """The span lo..hi grown toward want_lo..want_hi inside 0..limit, to at most `side` long: with `floor_first` (the
    vertical, y down) the far end, where a prop stands on the floor, takes the room before the near end; else both
    ends share it."""
    want_up, want_down = max(0, lo - max(0, want_lo)), max(0, min(limit, want_hi) - hi)
    room = side - (hi - lo)
    if want_up + want_down <= room:
        up, down = want_up, want_down
    elif floor_first:
        down = min(want_down, room)
        up = min(want_up, room - down)
    else:
        up = min(want_up, room // 2)
        down = min(want_down, room - up)
        up = min(want_up, room - down)
    return lo - up, hi + down


def geometry(mains, prop_box=None, room_2x=0, hold=None):
    """The crop [x0, y0, bw, bh] (render px) and the cell size (w, h): the figure's union alpha box over every frame
    plus a margin and `room_2x` render px of floor below the figure, widened to hold the alpha box `hold` (the fault
    still's armour, cut with this crop) with the same margin, grown toward the props' screen box (with its own margin)
    only as far as that box's longer side, so a prop fills room the cell has and never makes the figure smaller, then
    fitted into the 192 px box (longest side 192, both sides even)."""
    boxes = [b for b in POOL.map(bbox, mains) if b]
    W, H = map(int, sh("magick", "identify", "-format", "%w %h", mains[0]).split())
    x0 = max(0, min(b[0] for b in boxes) - MARGIN_2X)
    y0 = max(0, min(b[1] for b in boxes) - MARGIN_2X)
    x1 = min(W, max(b[2] for b in boxes) + MARGIN_2X)
    y1 = min(H, max(b[3] for b in boxes) + MARGIN_2X + room_2x)
    if hold:
        x0, y0 = max(0, min(x0, hold[0] - MARGIN_2X)), max(0, min(y0, hold[1] - MARGIN_2X))
        x1, y1 = min(W, max(x1, hold[2] + MARGIN_2X)), min(H, max(y1, hold[3] + MARGIN_2X))
    if prop_box:
        side = max(x1 - x0, y1 - y0)
        x0, x1 = toward(x0, x1, math.floor(prop_box[0]) - PROP_MARGIN_2X, math.ceil(prop_box[2]) + PROP_MARGIN_2X,
                        side, W, False)
        y0, y1 = toward(y0, y1, math.floor(prop_box[1]) - PROP_MARGIN_2X, math.ceil(prop_box[3]) + PROP_MARGIN_2X,
                        side, H, True)
    bw, bh = x1 - x0, y1 - y0
    s = BOX / max(bw, bh)
    return {"crop": [x0, y0, bw, bh], "w": round(bw * s / 2) * 2, "h": round(bh * s / 2) * 2, "render": [W, H]}


def window_png(w, h, path):
    """The floor layer's alpha window: 1 inside, a smoothstep to 0 over FADE_PX at the left, right and top edges and
    FADE_BOTTOM_PX at the bottom, so a pool or shadow is never cut hard by the cell."""
    if not os.path.exists(path):
        ys, xs = np.mgrid[0:h, 0:w].astype(np.float64) + 0.5
        d = np.minimum(np.minimum(xs, w - xs), ys) / FADE_PX
        d = np.minimum(d, (h - ys) / FADE_BOTTOM_PX)
        t = np.clip(d, 0, 1)
        a = (t * t * (3 - 2 * t) * 65535).round().astype(np.uint16)
        rgba = np.zeros((h, w, 4), np.uint16)
        rgba[..., :3] = 65535
        rgba[..., 3] = a
        subprocess.run(["magick", "-size", f"{w}x{h}", "-depth", "16", "-endian", "MSB", "rgba:-", f"PNG64:{path}"],
                       input=rgba.astype(">u2").tobytes(), check=True)
    return path


def compose(out, geo, scale=1, figure=None, floor=None, prop=None, window=None, depth=16, fade_prop=False):
    """One cell on the card: the floor (windowed), the figure's main render, the prop over it where it stands in front
    (windowed too with `fade_prop`: an endless flight that runs out of the cell) and the glow ('plus'), each cropped to
    the geometry and Lanczos-resized to the cell (times `scale`), laid as figures.layers lays them. Cached by path."""
    if os.path.exists(out):
        return out
    x0, y0, bw, bh = geo["crop"]
    w, h = geo["w"] * scale, geo["h"] * scale
    crop = ["-crop", f"{bw}x{bh}+{x0}+{y0}", "+repage", "-filter", "Lanczos", "-resize", f"{w}x{h}!"]
    fade = [window, "-compose", "DstIn", "-composite"] if window else []
    main, glow = figure or (None, None)
    sh("magick", "-size", f"{w}x{h}", f"xc:{BG_HEX}",
       *figures.layers(floor=[floor, *crop, *fade] if floor else None,
                       prop=[prop, *crop, *(fade if fade_prop else [])] if prop else None,
                       main=[main, *crop] if main else None, glow=[glow, *crop] if glow else None),
       "-depth", str(depth), f"PNG{48 if depth == 16 else 24}:{out}")
    return out


def load16(path):
    raw = subprocess.run(["magick", path, "-depth", "16", "-endian", "MSB", "rgb:-"], capture_output=True,
                         check=True).stdout
    w, h = map(int, sh("magick", "identify", "-format", "%w %h", path).split())
    return np.frombuffer(raw, ">u2").reshape(h, w, 3).astype(np.float64) / 65535.0


def load8(path):
    return np.asarray(Image.open(path).convert("RGB"))


def q8(a):
    return np.clip(np.round(np.asarray(a) * 255.0), 0, 255).astype(np.uint8)


# ---------------------------------------------------------------- pose distance on pixels

def edges(cells):
    """Edge maps (the outlines and toon band edges) and their distance transforms, one per frame."""
    lum = cells.astype(np.float32) @ np.array([0.2126, 0.7152, 0.0722], np.float32) / 255.0
    E, D = [], []
    for L in lum:
        g = np.hypot(ndimage.sobel(L, 0), ndimage.sobel(L, 1)) / 4.0
        e = g > EDGE_T
        E.append(e)
        D.append(ndimage.distance_transform_edt(~e).astype(np.float32))
    return E, D


def displacement(E, D, scale=2.0):
    """d[i][j]: how far (px at final size) frame i's edges lie from frame j's, symmetric, robust to EDGE_K stray
    pixels, measured on the 2x metric copies and divided by `scale`."""
    n = len(E)
    d = np.zeros((n, n), np.float32)
    pts = [np.nonzero(e) for e in E]
    for i in range(n):
        for j in range(i + 1, n):
            a = np.sort(D[j][pts[i]])[::-1]
            b = np.sort(D[i][pts[j]])[::-1]
            v = max(a[min(EDGE_K, len(a) - 1)] if len(a) else 0.0, b[min(EDGE_K, len(b) - 1)] if len(b) else 0.0)
            d[i, j] = d[j, i] = v / scale
    return d


# ---------------------------------------------------------------- selection and the playback table

def tolerance(d, eps, rel):
    """Per-frame error allowed: `eps` px, or `rel` x the local motion per tick where that is larger (fast phases
    step at about 15 fps instead of strobing); the loop's first and last frames neighbour."""
    n = len(d)
    v = np.array([(d[(p - 1) % n, p] + d[p, (p + 1) % n]) / 2.0 for p in range(n)], np.float32)
    return np.maximum(eps, rel * v)


def select(d, glow, eps=EPS, gamma=GAMMA, rel=REL, switch_cost=0.35):
    """Pose-indexed frames: each tick's frame p is shown by a stored frame s with d[p][s] within its tolerance and
    the glow within gamma; a greedy cover picks the fewest (the earliest wins ties), then a Viterbi pass assigns each
    tick the stored frame of least error plus `switch_cost` per change, so runs stay long and near twins never
    flicker. Holds and returns that retrace the way out reuse frames. Returns (stored positions in first-use order,
    per-tick index into that list)."""
    n = len(d)
    tol = tolerance(d, eps, rel)
    ok = (d <= tol[:, None]) & (np.abs(glow[:, None] - glow[None, :]) <= gamma)
    uncovered = np.ones(n, bool)
    chosen = []
    while uncovered.any():
        gain = (ok & uncovered[:, None]).sum(0)
        best = int(np.argmax(gain))
        chosen.append(best)
        uncovered &= ~ok[:, best]
    chosen.sort()
    S = np.array(chosen)
    cost = np.where(ok[:, S], d[:, S], np.inf)
    k = len(S)
    acc = cost[0].copy()
    back = np.zeros((n, k), np.int32)
    for t in range(1, n):
        move = acc.min() + switch_cost
        arg = int(np.argmin(acc))
        better = acc <= move
        back[t] = np.where(better, np.arange(k), arg)
        acc = np.where(better, acc, move) + cost[t]
    path = np.zeros(n, np.int32)
    path[-1] = int(np.argmin(acc))
    for t in range(n - 1, 0, -1):
        path[t - 1] = back[t, path[t]]
    used = sorted(set(path.tolist()), key=lambda s: int(np.argmax(path == s)))
    remap = {s: r for r, s in enumerate(used)}
    return [int(S[s]) for s in used], [remap[int(p)] for p in path]


def select_fit(d, glow, cells, rel=REL):
    """select() at the fast-phase allowance `rel` (the motion file's `fast_phase`), with the pose tolerance widened by
    a quarter at a time until the stored frames fit `cells`: a long flow (a sun salutation's 90 s) can hold more
    distinct poses than one sheet has cells. Returns the stored frames, the assignment and the factor (1.0 when
    select() fit at once)."""
    loose = 1.0
    stored, assign = select(d, glow, rel=rel)
    while len(stored) > cells:
        loose *= 1.25
        stored, assign = select(d, glow, eps=EPS * loose, rel=rel * loose)
    return stored, assign, loose


def one_fewer(d, glow, stored, assign, keep):
    """The selection with one stored frame fewer: each tick of a dropped frame goes to the nearest other stored frame
    (by pose distance, a frame within the glow match first), and the frame dropped is the one whose worst moved tick
    moves least, never `keep` (an index into `stored`: the key pose's frame). Returns (stored, assign) as select()
    does."""
    def nearest(p, k):
        return min(((abs(glow[p] - glow[stored[j]]) > GAMMA, float(d[p, stored[j]]), j)
                    for j in range(len(stored)) if j != k))
    best = None
    for k in range(len(stored)):
        if k != keep:
            moved = {p: nearest(p, k) for p, a in enumerate(assign) if a == k}
            cost = max(moved.values())[:2]
            if best is None or cost < best[0]:
                best = (cost, {p: m[2] for p, m in moved.items()})
    path = [best[1].get(p, a) for p, a in enumerate(assign)]
    used = sorted(set(path), key=path.index)
    remap = {j: r for r, j in enumerate(used)}
    return [stored[j] for j in used], [remap[j] for j in path]


def select_spare(d, glow, w, h, key, rel=REL):
    """select_fit() for a move whose sheet also holds its fault still, one cell after the stored frames: where that
    cell would take a larger sheet (or none, past the largest), one stored frame fewer (one_fewer; the frame shown
    at tick `key` stays). A hold of one frame has none to give, and its sheet grows."""
    stored, assign, loose = select_fit(d, glow, capacity(w, h), rel)
    if grows(len(stored), w, h) and len(stored) > 1:
        stored, assign = one_fewer(d, glow, stored, assign, assign[key])
    return stored, assign, loose


def capacity(w, h):
    """The most w x h frames one MAX_SHEET sheet holds, each in its padded cell (cell_size)."""
    cw, ch = cell_size(w, h)
    return (MAX_SHEET // cw) * (MAX_SHEET // ch)


def rle(assign):
    """The run-length table [[cell, ticks], ...] of a per-tick assignment."""
    out = []
    for c in assign:
        if out and out[-1][0] == c:
            out[-1][1] += 1
        else:
            out.append([c, 1])
    return out


def shown_at_ticks(runs):
    return [c for c, n in runs for _ in range(n)]


# ---------------------------------------------------------------- sheets

def cell_size(w, h):
    return (w + 2 * PAD, h + 2 * PAD)


def sheet_size(n, cw, ch):
    """The smallest power-of-two sheet (at most MAX_SHEET a side) holding n cells: least area, then the squarer."""
    sides = [2 ** k for k in range(6, int(math.log2(MAX_SHEET)) + 1)]
    best = None
    for W in sides:
        for H in sides:
            cols, rows = W // cw, H // ch
            if cols and rows and cols * rows >= n:
                key = (W * H, abs(math.log2(W / H)))
                if best is None or key < best[0]:
                    best = (key, (W, H, cols))
    if best is None:
        raise ValueError(f"{n} cells of {cw}x{ch} do not fit one {MAX_SHEET} px sheet")
    return best[1]


def layout(n, w, h):
    cw, ch = cell_size(w, h)
    W, H, cols = sheet_size(n, cw, ch)
    return {"sheet": [W, H], "cell": [cw, ch], "cols": cols, "pad": PAD, "size": [w, h], "n": n}


def grows(n, w, h):
    """Whether one cell more than n takes a larger sheet (or none: n fills the largest)."""
    if n >= capacity(w, h):
        return True
    (w0, h0), (w1, h1) = layout(n, w, h)["sheet"], layout(n + 1, w, h)["sheet"]
    return w1 * h1 > w0 * h0


def pack_sheet(cells8, lay, bg=BG):
    W, H = lay["sheet"]
    cw, ch = lay["cell"]
    w, h = lay["size"]
    sheet = np.empty((H, W, 3), np.uint8)
    sheet[:] = bg
    for k, c in enumerate(cells8):
        r, q = divmod(k, lay["cols"])
        x, y = q * cw + PAD, r * ch + PAD
        sheet[y:y + h, x:x + w] = c
    return sheet


def unpack(img, lay):
    cw, ch = lay["cell"]
    w, h = lay["size"]
    out = []
    for k in range(lay["n"]):
        r, q = divmod(k, lay["cols"])
        x, y = q * cw + PAD, r * ch + PAD
        out.append(img[y:y + h, x:x + w])
    return out


# ---------------------------------------------------------------- the encoder and the quality bar

def in_box(a, box=BOX):
    """A cell as the game shows it, centred in the `box` px square on the card: the game's 192 px box, or for a cell
    drawn smaller that box at the cell's scale (publish.py's watch sheets; in the 192 px box a small cell scores
    higher than it looks). SSIMULACRA2's coarse scales degenerate on a bare narrow cell."""
    c = np.empty((box, box, 3), np.uint8)
    c[:] = BG
    y, x = (box - a.shape[0]) // 2, (box - a.shape[1]) // 2
    c[y:y + a.shape[0], x:x + a.shape[1]] = a
    return c


def ssim2_cells(refs, decs, tmp, box=BOX):
    os.makedirs(tmp, exist_ok=True)

    def one(k):
        a, b = os.path.join(tmp, f"{k:03d}_ref.png"), os.path.join(tmp, f"{k:03d}_dec.png")
        Image.fromarray(in_box(refs[k], box)).save(a, compress_level=1)
        Image.fromarray(in_box(decs[k], box)).save(b, compress_level=1)
        return float(sh("ssimulacra2", a, b).strip())
    return list(POOL.map(one, range(len(refs))))


def encode_to_target(sheet_png, refs, lay, out, tmp, target=TARGET, palettes=PALETTES, box=BOX):
    """The base sheet as the smallest palette PNG (pngquant without dithering, then oxipng) of `palettes`, tried
    smallest first, whose worst cell still scores SSIMULACRA2 >= target in its `box` (in_box), or a truecolour PNG
    when the largest falls short. PNG because the Forever client shows an addon's JPEG as a green box (1.60.1 on
    macOS, 2026-09-30) and loads palette PNGs. Returns (the colours, 0 for truecolour, and the cells' scores)."""
    for colours in palettes:
        sh("pngquant", "--force", "--nofs", "--speed", "1", str(colours), "--output", out, sheet_png)
        sh("oxipng", "-q", "-o", "4", "--strip", "all", out)
        scores = ssim2_cells(refs, unpack(load8(out), lay), tmp, box)
        if min(scores) >= target:
            return colours, scores
    sh("oxipng", "-q", "-o", "4", "--strip", "all", "--out", out, sheet_png)
    return 0, ssim2_cells(refs, unpack(load8(out), lay), tmp, box)


def encode_tint(png, out):
    """The tint sheet: 32 colours, no dithering (pngquant), then oxipng; returns the bytes."""
    sh("pngquant", "--force", "--nofs", "--speed", "1", "32", "--output", out, png)
    sh("oxipng", "-q", "-o", "4", "--strip", "all", out)
    return os.path.getsize(out)


# ---------------------------------------------------------------- the class-colour split

def split(Y):
    """Base and tint from the cells rendered at the five grey cloth levels: per pixel and channel, the least-squares
    line through (level, value), slope clamped to 0..1 (the tint) and intercept to 0..1 (the base), for the game's
    composite  shown = base + tint * vertexColour  on stored values."""
    xs = np.array([LEVELS[k] for k in sorted(LEVELS)])
    ys = np.stack([Y[k] for k in sorted(LEVELS)])
    xm = xs.mean()
    sxx = ((xs - xm) ** 2).sum()
    ym = ys.mean(0)
    s = np.clip(np.tensordot(xs - xm, ys - ym, axes=(0, 0)) / sxx, 0.0, 1.0)
    return np.clip(ym - s * xm, 0.0, 1.0), s


# ---------------------------------------------------------------- the manifest

def lua_value(v, level=0):
    if isinstance(v, bool):
        return "true" if v else "false"
    if isinstance(v, (int, np.integer)):
        return str(int(v))
    if isinstance(v, float):
        return repr(v)
    if isinstance(v, str):
        return '"' + v.replace("\\", "\\\\").replace('"', '\\"') + '"'
    if isinstance(v, list):
        if not v:
            return "{}"
        parts = [lua_value(x, level) for x in v]
        one = "{ " + ", ".join(parts) + " }"
        if len(one) + 4 * level <= 100:
            return one
        pad, lines, cur = "\t" * (level + 1), [], ""
        for x in parts:
            if cur and len(cur) + len(x) + 2 > 100 - 4 * level:
                lines.append(cur)
                cur = ""
            cur += x + ", "
        lines.append(cur)
        return "{\n" + "\n".join(pad + ln.rstrip() for ln in lines) + "\n" + "\t" * level + "}"
    if isinstance(v, dict):
        pad = "\t" * (level + 1)
        items = []
        for k in sorted(v):
            key = k if re.match(r"^[A-Za-z_][A-Za-z0-9_]*$", k) else f'["{k}"]'
            items.append(f"{pad}{key} = {lua_value(v[k], level + 1)},")
        return "{\n" + "\n".join(items) + "\n" + "\t" * level + "}"
    raise TypeError(type(v))
