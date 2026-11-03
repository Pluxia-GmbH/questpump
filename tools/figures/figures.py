"""The figure build (docs/plan.md slice 10): motion files in, the pack, the icons, the films and the review page out.

    python3 tools/figures/figures.py check [moves]      bake every motion file and run the checks (stdlib only);
                                                        FIGURE_CHECK_SHARD=k/n checks every nth file from the kth
    python3 tools/figures/figures.py preview [moves]    a quick contact sheet per move (Blender, small, few samples),
                                                        its fault still last where it has one
    python3 tools/figures/figures.py pose <move> <pose> one pose's joints and shortfalls (for writing a motion), and
                                                        the target the move's fault breaks, measured at that pose
    python3 tools/figures/figures.py clip [moves] [--json[=<file>]]
                                                        plate-on-plate clipping in the real armour (Blender, clip.py):
                                                        each pair of parts that do not meet at a joint, its deepest
                                                        frame; no moves ranks the library, worst first
    python3 tools/figures/figures.py helm               measure the helm's points and distance grid for the checks
                                                        into helm.json
                                                        (Blender, helm.py; again after any blender/armor_model.py
                                                        change, which check names until then)
    python3 tools/figures/figures.py fmt [moves]        rewrite motion files in the house layout
    python3 tools/figures/figures.py approve <moves>    record David's approval (contract/art/approvals.json)
    tools/figures/.venv/bin/python tools/figures/figures.py build [moves]
                                                        render, select, pack, icons, films, review (make figures); an
                                                        extra (motion.py `extra`) its key pose's still alone, and a
                                                        run naming extras alone leaves the game's files as they are
    tools/figures/.venv/bin/python tools/figures/figures.py films [moves]
                                                        the films (both encodings) and the phones' fault pairs
                                                        alone, then publish, when only the films' form changed:
                                                        renders what is missing, never writes the pack, the icons or
                                                        the fixture, and names a move whose frames would change its
                                                        pack entry (make figures rebuilds both); a style, Blender,
                                                        ground or fault-pose change needs make figures, as it keeps
                                                        the entry
    tools/figures/.venv/bin/python tools/figures/figures.py publish
                                                        the phones' and the desktop app's films, the stills and tint
                                                        stills cut from them, the fault pairs and the table in
                                                        contract/figures/, player.js's copies again, then the
                                                        watches' pack cut from the game's (contract/figures/watch/;
                                                        build and films run it; publish.py --check checks them)
    tools/figures/.venv/bin/python tools/figures/figures.py prune [moves] [--dry-run]
                                                        delete the renders this tree no longer uses (old render
                                                        folders, and the loop cells and frames no build reads; build
                                                        prunes the moves it built)

[moves] is a comma list of move keys or extras' names, or the names as separate words (default: every motion file in
contract/art). Renders and intermediates live in
tools/figures/build/ (gitignored); the pack goes to QuestPump_FigArmor/, the icon sheet to Media/, the steps fixture
to contract/fixtures/figure-steps.json, the films (base over tint; the phones' HEVC, the desktop app's H.264),
key-pose stills with their tints, fault pairs and their table to contract/figures/ (design v3 §11: both phone apps
bundle them, without the desktop app's films/), and the watches' own pack to contract/figures/watch/.
"""
import contextlib
import glob
import hashlib
import json
import os
import shutil
import subprocess
import sys
import threading
import time
from concurrent.futures import ProcessPoolExecutor, ThreadPoolExecutor

import checks
import motion
import rig

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
ART = os.path.join(ROOT, "contract", "art")
BUILD = os.path.join(HERE, "build")
STYLE = os.path.join(HERE, "style.json")
RENDER = os.path.join(HERE, "blender", "render.py")
CLIP = os.path.join(HERE, "clip.py")               # outside blender/: build.render_key hashes every script there
HELM = os.path.join(HERE, "helm.py")               # the same
FONT = "/System/Library/Fonts/Supplemental/Arial.ttf"   # labels on the contact sheets (macOS)
JOBS = max(2, (os.cpu_count() or 8) // 3)          # EEVEE uses the GPU; a third of the cores keeps it fed
CHECK_JOBS = min(8, os.cpu_count() or 1)           # processes baking and checking moves (pure Python, CPU bound)
BLENDER_TIMEOUT = 1800                             # s per Blender job: a stalled GPU fails the build, never hangs it
GAME = "MacOS/World of Warcraft( -|$)"             # pgrep -f: a running WoW client of any flavour, never its launcher
RENDERS = "blender/render.py"                      # pgrep -f: a figure render from any checkout or scratch copy
GAME_JOBS = int(os.environ.get("FIGURE_GAME_JOBS", "1"))   # renders at once on the Mac while the game runs (0: none)
SHARD = os.environ.get("FIGURE_CHECK_SHARD")       # k/n: `check` of every nth motion file from the kth (CI's jobs)
_gate = threading.Lock()                           # one job at a time counts the renders and starts its own


def motion_paths(moves=None):
    paths = sorted(p for p in glob.glob(os.path.join(ART, "*.json")) if os.path.basename(p) != "approvals.json")
    if moves:
        want = set(moves.split(","))
        paths = [p for p in paths if os.path.splitext(os.path.basename(p))[0] in want]
        missing = want - {os.path.splitext(os.path.basename(p))[0] for p in paths}
        if missing:
            sys.exit(f"no motion file for: {', '.join(sorted(missing))}")
    return paths


def shard_moves(spec):
    """The comma list of shard `k/n`'s moves: every nth motion file in name order from the kth, so n shards check each
    file once."""
    k, _, n = spec.partition("/")
    try:
        k, n = int(k), int(n)
    except ValueError:
        k = n = 0
    names = [os.path.splitext(os.path.basename(p))[0] for p in motion_paths()]
    if not 1 <= k <= n or not names[k - 1::n]:
        sys.exit(f"FIGURE_CHECK_SHARD={spec}: want k/n with 1 <= k <= n, naming at least one motion file")
    return ",".join(names[k - 1::n])


def failure(stage, err):
    """A motion file's error as its problem line: a MotionError in its own words, anything else (bad JSON, a field
    of the wrong type) with its type."""
    return f"{stage}: {err}" if isinstance(err, motion.MotionError) else f"{stage}: {type(err).__name__}: {err}"


def load_all(moves=None):
    """({move: spec}, {move: [problem]}) for every motion file (variants included): a file that does not load is
    named by its file name with its problem, so one malformed file never stops the others."""
    specs, broken = {}, {}
    for p in motion_paths(moves):
        try:
            s = motion.load(p)
        except Exception as err:
            broken[os.path.splitext(os.path.basename(p))[0]] = [failure("load", err)]
            continue
        specs[s["move"]] = s
    return specs, broken


def write_track(track, suffix=""):
    """build/tracks/<move><suffix>.json, the track as the renderer reads it (its `_` keys are the checks' own)."""
    d = os.path.join(BUILD, "tracks")
    os.makedirs(d, exist_ok=True)
    path = os.path.join(d, track["move"] + suffix + ".json")
    body = json.dumps({k: v for k, v in track.items() if not k.startswith("_")}, sort_keys=True)
    if not os.path.exists(path) or open(path).read() != body:
        with open(path, "w") as fh:
            fh.write(body)
    return path


def check_one(spec):
    """One move's bake and checks, in a worker process of `check`: (spec, track or None when it did not bake, its
    problems). The spec comes back with the track, as the bake read it."""
    track = None
    try:
        if spec.get("variant_of"):
            # only the reference's own file: a half-written motion file elsewhere never fails this move's check
            path = os.path.join(ART, spec["variant_of"] + ".json")
            if not os.path.exists(path):
                found = [f"variant: no motion file {spec['variant_of']}"]
            else:
                found = checks.variant_problems(spec, motion.load(path))
        else:
            track = motion.bake(spec)
            found = checks.summary(checks.run(spec, track))
            write_track(track)
    except Exception as err:
        found = [failure("variant" if spec.get("variant_of") else "bake", err)]
    return spec, track, found


def check(moves=None):
    """Bake every motion file and run the checks, printing each move's problems. Returns ({move: (spec, track)} of
    every move that baked, variants aside: build.py derives their tables; {move: [problem]} of every move that does
    not load, bake or pass). A file that raises is that move's problem, never the end of the check. The moves are
    independent, so CHECK_JOBS processes share them. A helm.json that no longer fits the armour stops it first: every
    move's helm would be tested against the old one."""
    stale = checks.helm_stale()
    if stale:
        sys.exit(stale)
    specs, problems = load_all(moves)
    baked = {}
    todo = [spec for _, spec in sorted(specs.items())]
    jobs = min(CHECK_JOBS, len(todo))
    with ProcessPoolExecutor(jobs) if jobs > 1 else contextlib.nullcontext() as pool:
        done = pool.map(check_one, todo, chunksize=4) if pool else map(check_one, todo)
        for spec, track, found in done:
            if track is not None:
                baked[spec["move"]] = (spec, track)
            if found:
                problems[spec["move"]] = found
    for move, found in sorted(problems.items()):
        print(f"{move}:")
        for line in found:
            print("  " + line)
    print(f"{len(baked)} moves baked, {len(problems)} with problems")
    return baked, problems


def counts_problems(baked):
    """Each baked move whose pack entry's `counts` is not what its motion's `rep_ends` give (motion.rep_counts), so a
    motion edit without `make figures` fails the check: the game and the apps would count a loop's reps wrong. A
    timing variant's follow its reference's at the build."""
    path = os.path.join(ROOT, "QuestPump_FigArmor", "pack.json")
    if not os.path.exists(path):
        return []
    with open(path) as fh:
        pack = json.load(fh)["moves"]
    out = []
    for move, (spec, track) in sorted(baked.items()):
        want = motion.rep_counts(spec.get("rep_ends"), [fr["phase"] for fr in track["frames"]]).get("counts")
        have = pack.get(move, {}).get("counts")
        if move in pack and want != have:
            out.append(f"{move}: the pack's counts are {have}, its rep_ends give {want}: run make figures MOVES={move}")
    return out


def pose(move, name):
    """The joints (body space, cm) and reach shortfalls of one pose: for writing a motion. A move with a fault also
    gets the target the fault breaks, measured at this pose (it holds at the key pose and fails at the fault pose)."""
    spec = motion.load(os.path.join(ART, move + ".json"))
    J, short = motion.solve(motion.expand(motion.resolve_pose(spec, name)))
    for k in rig.JOINTS:
        print(f"{k:14s} " + " ".join(f"{c:7.1f}" for c in J[k]))
    print("short", {k: round(v, 1) for k, v in short.items() if v > 0})
    if spec.get("fault"):
        t = spec["fault"]["breaks"]
        v, label = motion.measure(t, J, spec.get("view", "side"))
        print("fault breaks:", label, f"({checks.misses(t, v) or 'holds'})")


def running(pattern):
    """How many processes on this Mac match the `pgrep -f` pattern."""
    return len(subprocess.run(["pgrep", "-f", pattern], capture_output=True, text=True).stdout.split())


def blender(args, log):
    """One Blender render. While a WoW client runs, the job first waits until fewer than GAME_JOBS figure renders run on
    the Mac, any session's: EEVEE's GPU time comes straight out of the game's frame rate, and `nice` lowers only the CPU
    (2026-10-05: six renders beside the game took up to 41 % of the GPU in a 5-s window)."""
    os.makedirs(os.path.dirname(log), exist_ok=True)
    with open(log, "w") as fh:
        with _gate:
            while running(GAME) and running(RENDERS) >= GAME_JOBS:
                time.sleep(15)
            p = subprocess.Popen(["nice", "-n", "10", "blender", "-b", "--factory-startup", "--python-exit-code", "1",
                                  "-P", RENDER, "--", *map(str, args)], stdout=fh, stderr=subprocess.STDOUT)
        try:
            code = p.wait(timeout=BLENDER_TIMEOUT)
        except subprocess.TimeoutExpired:
            p.kill()
            p.wait()
            raise
    if code:
        raise RuntimeError(f"blender failed, see {log}")


def layers(floor=None, prop=None, main=None, glow=None):
    """The ImageMagick arguments that lay one frame's render layers over the image before them (the card, or a
    transparent canvas), each layer given as the arguments that load it (its file, cropped, resized or windowed) or
    None: the floor, the figure (`main`), the props, then the glow added ('plus'). The only order of them: the build's
    cells, films and stills (pack.compose), its icons (build.icon) and the preview's frames all lay them so.

    The prop layer holds only what of the props the figure leaves in sight (render.py's ground pass renders the figure
    as a holdout there), so it lies over the figure: a prop nearer the camera hides the figure, and where the figure
    stands in front the layer is empty. Where the two share an edge pixel, the figure keeps min(1, its alpha / (1 -
    the prop's)) under the prop laid over it: each then covers its own part of the pixel and no card shows between
    them (a 'disjoint over': within 0.002 levels of that composite in floating point on straight_bar_dip's layers,
    where a plain 'over' was off by up to 2.5, 2026-10-22). The glow, the bloom outside the silhouette, goes last, over
    the props as over the floor."""
    args = []
    if floor:
        args += ["(", *floor, ")", "-compose", "over", "-composite"]
    if prop:
        args += ["(", *prop, "-write", "mpr:qp_prop", "+delete", ")"]
    if main:
        args += ["(", *main]
        if prop:                                    # its alpha over (1 - the prop's), at most 1
            args += ["(", "+clone", "-alpha", "extract", "(", "mpr:qp_prop", "-alpha", "extract", "-negate", ")",
                     "-compose", "DivideDst", "-composite", "-clamp", ")", "-compose", "CopyOpacity", "-composite"]
        args += [")", "-compose", "over", "-composite"]
    if prop:
        args += ["mpr:qp_prop", "-compose", "over", "-composite"]
    if glow:
        args += ["(", *glow, ")", "-compose", "plus", "-composite"]
    return args


def preview(moves=None):
    """Nine frames per move (eight spread over the loop and the key pose) at a 160 px cell, 16 samples, laid out
    with their frame numbers and phases, and last the fault still of a move that has one (labelled `fault` with its
    caption key, in the loop's camera), in build/preview/<move>.<track sha8>.png: the sheet Claude reads while
    writing a motion. Each frame's layers lie as the build's cells' do (layers), at the render's size."""
    baked, bad = check(moves)
    todo = []
    for move, (spec, track) in sorted(baked.items()):
        n = len(track["frames"])
        frames = sorted({round(i * n / 8) % n for i in range(8)} | {track["key"]})
        tp = write_track(track)
        ftp = write_track(motion.fault_track(track), ".fault") if "_fault" in track else None
        h = hashlib.sha256()                         # a folder and a sheet per track and renderer: no stale layer,
        for f in [tp, STYLE] + sorted(glob.glob(os.path.join(HERE, "blender", "*.py"))) + ([ftp] if ftp else []):
            with open(f, "rb") as fh:                # no stale view
                h.update(fh.read())
        tag = h.hexdigest()[:8]
        todo.append((move, frames, os.path.join(BUILD, "preview", move, tag), tp, tag, ftp))

    def render(tp, d, frames):
        fl = ",".join(map(str, frames))
        blender([tp, STYLE, d, 160, "figure", fl, "crimson", 16], os.path.join(d, "log.txt"))
        blender([tp, STYLE, os.path.join(d, "ground"), 160, "ground", fl, 16], os.path.join(d, "log_ground.txt"))

    def run(t):
        move, frames, d, tp, tag, ftp = t
        spec, track = baked[move]
        render(tp, d, frames)
        shown = [(d, f, f"{f}{' key' if f == track['key'] else ''} {track['frames'][f]['phase'] or ''}")
                 for f in frames]
        if ftp:
            render(ftp, os.path.join(d, "fault"), [track["key"]])
            shown.append((os.path.join(d, "fault"), track["key"], f"fault {spec['fault']['caption']}"))
        per = []
        for src, f, label in shown:
            prop = os.path.join(src, "ground", "prop", f"p{f:04d}.png")
            per.append(["(", "-size", "320x320", "xc:#0E0D0B",
                        *layers(floor=[os.path.join(src, "ground", "floor", f"g{f:04d}.png")],
                                prop=[prop] if os.path.exists(prop) else None,
                                main=[os.path.join(src, "crimson", "main", f"f{f:04d}.png")],
                                glow=[os.path.join(src, "crimson", "glow", f"glow_{f:04d}.png")]),
                        "-font", FONT, "-fill", "#c8b88a", "-pointsize", "18", "-gravity", "northwest",
                        "-annotate", "+6+4", label, ")"])
        out = os.path.join(BUILD, "preview", f"{move}.{tag}.png")
        rows = []
        for r in range(0, len(per), 5):
            rows += ["(", *[x for t in per[r:r + 5] for x in t], "+append", ")"]
        subprocess.run(["magick", *rows, "-background", "#0E0D0B", "-append", "-resize", "80%", out], check=True)
        shutil.copyfile(out, os.path.join(BUILD, "preview", f"{move}.png"))     # the latest, at a fixed name too
        return out
    with ThreadPoolExecutor(JOBS) as pool:
        for out in pool.map(run, todo):
            print(out)


def clip(moves=None, out=None):
    """Plate-on-plate clipping in the real armour (clip.py, one Blender process; it renders nothing, so it takes no
    turn at the game's render gate): every frame of every named move (default: the library), and its fault still.
    Named moves print each pair from clip.FLOOR deep (its deepest frame and phase, the part that reaches in) and a
    line per move with its worst; the library prints that line for every move, worst first. `out` gets the whole
    result as JSON, the phases filled in, for comparing a fix round."""
    baked, _ = check(moves)
    left = sorted(set(moves.split(",")) - set(baked)) if moves else []
    if left:
        print(f"not measured: {', '.join(left)} (a timing variant plays its reference's frames; a move that does not "
              "bake has none)")
    if not baked:
        return
    d = os.path.join(BUILD, "clip")
    os.makedirs(d, exist_ok=True)
    job, raw, log = (os.path.join(d, n) for n in ("job.json", "raw.json", "log.txt"))
    with open(job, "w") as fh:
        json.dump({"moves": [{"move": m, "track": write_track(t), "fault": t["_fault"]["frame"] if "_fault" in t
                              else None} for m, (_, t) in sorted(baked.items())]}, fh)
    frames = sum(len(t["frames"]) for _, t in baked.values())
    print(f"measuring {len(baked)} moves, {frames} frames (log: {os.path.relpath(log, ROOT)})", flush=True)
    t0 = time.time()
    with open(log, "w") as fh:
        code = subprocess.run(["nice", "-n", "10", "blender", "-b", "--factory-startup", "--python-exit-code", "1",
                               "-P", CLIP, "--", job, raw], stdout=fh, stderr=subprocess.STDOUT).returncode
    if code:
        sys.exit(f"clip.py failed, see {log}")
    with open(raw) as fh:
        result = json.load(fh)
    rows = []
    for move, r in result["moves"].items():
        track = baked[move][1]
        for kind in ("pairs", "fault"):
            for w in r.get(kind, {}).values():
                w["phase"] = "fault" if w["frame"] == "fault" else track["frames"][w["frame"]]["phase"]
        worst = max(r["pairs"].items(), key=lambda kv: kv[1]["depth"], default=None)
        r["worst"] = dict(worst[1], pair=worst[0]) if worst else {"depth": 0.0}
        rows.append((r["worst"]["depth"], move))

    def where(pair, w):
        a, b = pair.split("/")
        at = "" if w["frame"] == "fault" else f", frame {w['frame']} ({w['phase'] or '-'})"
        return f"{w['inside']} in {b if w['inside'] == a else a}{at}"
    floor = result["floor"]
    order = sorted(rows, key=lambda x: x[1]) if moves else sorted(rows, key=lambda x: (-x[0], x[1]))
    for depth, move in order:
        r = result["moves"][move]
        print(f"{depth:5.1f} cm  {move}" + (f": {where(r['worst']['pair'], r['worst'])}" if depth else ""))
        for kind in ("pairs", "fault") if moves else ():
            for pair, w in sorted(r.get(kind, {}).items(), key=lambda kv: -kv[1]["depth"]):
                if w["depth"] >= floor:
                    print(f"         {w['depth']:4.1f} cm  " + (f"{where(pair, w)}, {w['frames']} of {r['frames']} "
                          f"frames from {floor} cm" if kind == "pairs" else f"fault still: {where(pair, w)}"))
    print(f"{len(rows)} moves, {frames} frames in {time.time() - t0:.0f} s")
    if out:
        with open(out, "w") as fh:
            json.dump(result, fh, indent=1, sort_keys=True)
        print("wrote", out)


def helm():
    """The helm's points and distance grid for the checks (helm.py, one Blender process; it renders nothing) into
    checks.HELM_FILE."""
    log = os.path.join(BUILD, "helm.log")
    os.makedirs(BUILD, exist_ok=True)
    with open(log, "w") as fh:
        code = subprocess.run(["nice", "-n", "10", "blender", "-b", "--factory-startup", "--python-exit-code", "1",
                               "-P", HELM, "--", checks.HELM_FILE], stdout=fh, stderr=subprocess.STDOUT).returncode
    if code:
        sys.exit(f"helm.py failed, see {log}")
    print("wrote", os.path.relpath(checks.HELM_FILE, ROOT))


def main():
    cmd = sys.argv[1] if len(sys.argv) > 1 else "check"
    words = [a for a in sys.argv[2:] if not a.startswith("--")]    # flags: the build's --prune, clip's --json
    moves = ",".join(words[:1] if cmd == "pose" else words) or None    # a comma list, or the names as words
    if cmd == "check":
        baked, problems = check(moves or (shard_moves(SHARD) if SHARD else None))
        stale = counts_problems(baked)
        for line in stale:
            print(line)
        sys.exit(1 if problems or stale else 0)
    elif cmd == "helm":
        helm()
    elif cmd == "fmt":
        for p in motion_paths(moves):
            with open(p) as fh:
                spec = json.load(fh)
            text = motion.dumps(spec) + "\n"
            with open(p) as fh:
                if fh.read() != text:
                    with open(p, "w") as out:
                        out.write(text)
                    print("formatted", os.path.basename(p))
    elif cmd == "pose":
        pose(moves, words[1])
    elif cmd == "preview":
        preview(moves)
    elif cmd == "clip":
        out = [a.partition("=")[2] or os.path.join(BUILD, "clip.json") for a in sys.argv[2:]
               if a.partition("=")[0] == "--json"]
        clip(moves, out[0] if out else None)
    elif cmd == "approve":
        import datetime
        import review
        if not moves:
            sys.exit("usage: figures.py approve <move>[,<move>...]")
        missing = [m for m in moves.split(",") if not os.path.exists(os.path.join(ART, m + ".json"))]
        if missing:
            sys.exit(f"no motion file for: {', '.join(missing)}")
        review.approve(moves.split(","), datetime.date.today().isoformat())
        print("approved", moves)
    elif cmd in ("build", "films"):
        import build
        import publish
        built, left = build.run(moves, films=cmd == "films")
        publish.publish(built)                  # every build, the full one too: the build folder is a shared cache
        if left:
            print(f"{len(left)} {'move needs' if len(left) == 1 else 'moves need'} another look:")
            for line in left:
                print("  " + line)
            sys.exit(1)
    elif cmd == "publish":
        import publish
        publish.publish()
    elif cmd == "prune":
        import build
        baked, _ = check(moves)
        dry = "--dry-run" in sys.argv
        gone = build.prune(baked, moves.split(",") if moves else None, dry)
        for f in gone if dry else []:
            print("  " + os.path.relpath(f, BUILD))
        word = "folder" if len(gone) == 1 else "folders"
        print(f"{'would prune' if dry else 'pruned'} {len(gone)} old render {word}")
    else:
        sys.exit(__doc__)


if __name__ == "__main__":
    main()
