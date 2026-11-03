"""The move figures outside the game: the phones' (design v3 §11, 2026-10-04: bundled in both phone apps, never
downloaded), the watches' (their own pack, 2026-10-21), the desktop app's (slice 35) and questpump.com's. The one
committed source of them all is `contract/figures/`, which the iPhone app bundles as folder references
(apple/project.yml) and the Android phone app as its assets (android/phone figureAssets, a FolderAssets task), both
without `films/`, the desktop app's alone, and `watch/`, the watch apps' alone, and whose films and stills the site's
pages import at build time (since 2026-10-09, sharper than the pack's sheets the site's hero played before); written
and pruned here and checked by `--check`.

    contract/figures/loops/<owner>.mp4    each owner's film of the current build (HEVC hvc1, one rep at 30 fps), the
                                          bytes build.py wrote, never re-encoded; every library move is its own owner
                                          (2026-10-09: a timing variant's film is its own run table over its reference
                                          move's frames, so it plays at the variant's tempo); since slice 29 a stacked
                                          film, SIDE wide and TINT + SIDE high: the base (the sash and the crest a very
                                          dark indigo, the cloth's light removed) on the card in rows 0 to SIDE - 1, a
                                          GUARD-px black band, the tint (the cloth's light on black) in rows TINT to
                                          TINT + SIDE - 1, which a phone shows as clamp(base + tint x colour / 255) per
                                          channel on the stored sRGB values, as the pack's sheets and Figure.lua's ADD
                                          tint draw it in the game
    contract/figures/films/<owner>.mp4    the same film in H.264 (avc1; build.py ENCODINGS), encoded from the same
                                          frames and listing, so its layout, frames and colour tags are the loop's:
                                          the desktop app's, whose Windows web view (WebView2) plays HEVC only through
                                          an extra Windows extension (slice 35). The test builds bundle the folder;
                                          from the public repository's first release (docs/tasks.md) the desktop app
                                          downloads it once as figures-<films_digest>.zip (`--zip`) from the release
                                          feed and keeps it
    contract/figures/stills/<owner>.png   each film's key-pose frame in crimson (the base plus the tint in
                                          `figureCrimson`, so the still is the film's crimson frame), cut from the
                                          film itself (2026-10-05, QA finding F1: a still framed apart from its loop
                                          jumped when the loop started), in the film's SIDE-px square, its card keyed
                                          out to transparency both ways (a pixel lighter or darker than the card
                                          keeps that much of its light or shade as alpha), so drawn over the card it
                                          is the film's frame exactly and over a tile's well the figure keeps its dark
                                          lines; a 256-colour palette PNG. The tiles show it as it is, in crimson
    contract/figures/stills/<owner>.t.png that frame's tint half, the cloth's light on black (a truecolour PNG, a few
                                          KB: write_tint), with which a phone turns the still drawn over the card (S:
                                          keyed, the still alone holds the cloth's light only at its alpha) into the
                                          class colour as clamp(S - tint x crimson + tint x colour)
    contract/figures/stills/<extra>.png and <extra>.t.png
                                          an extra's still and its tint (slice 45: a figure that is no library move,
                                          tools/figures/motion.py `extra`; the streak card's standard-bearer,
                                          `streak_banner`): its key pose's render cut with a crop of that frame into an
                                          EXTRA_SIDE-px square (no film: the share cards draw it larger than a move's),
                                          in crimson and keyed as an owner's still is, beside it the same frame's tint,
                                          written by build.py `build_extra` and taken from the build as a film is
    contract/figures/pairs/<owner>.key.png and <owner>.fault.png
                                          the right-versus-wrong pair of an owner with a fault (slice 22): the key pose
                                          and the fault still, each a cell of the loop on its card (build.py
                                          write_stills), so the two register with each other and with the loop, in
                                          crimson; beside each its tint, <owner>.key.t.png and <owner>.fault.t.png
                                          (slice 29), which takes the pair into the class colour the same way (its
                                          stills are opaque, the card in them), as the game's View:ShowPair tints it; a
                                          timing variant's pair is its reference move's files under its own name, as the
                                          pack gives a variant its reference's fault cell
    contract/figures/figures.json         the table the apps read: per library move its owner (the move itself), the key
                                          pose's second in that film, whether the pack mirrors the move, with a pair,
                                          its fault's caption key (`fault`, its words in contract/moves.json
                                          `faults`), where its loop shows more than one rep, the second at which each
                                          rep but the last ends (`counts`, the pack's ticks over hz, from the motion's
                                          `rep_ends`), and where the pack has gear variants of it (`<move>.<gear>`,
                                          tools/figures/motion.py; 2026-10-09) `gear`, the same for each equipment key
                                          whose implement they draw, which a player whose load that implement carries
                                          is shown (the core's implement with the player's gear; any other is shown the
                                          move's own); per owner the box its figure and props fill over the whole loop
                                          (`box`, its bottom where the figure stands: the plinth's line, and the timer
                                          ring fits the box in its circle, QA finding N6 of 2026-10-05), which edges of
                                          the square cut the loop's floor light (`fade`, so the apps feather them: QA
                                          findings F8 and F10, "a lighter square box"), and the square a tile shows of
                                          its still (`tile`), each as shares of the side, unmirrored, read off the
                                          crimson frames, and the film's own length (`loop`, in seconds: its video
                                          track's samples' durations, its frames / 30, which both encodings share), one
                                          loop at real tempo, the beat the apps count reps by (2026-10-09); the card
                                          colour, the crimson (`crimson`, tokens.json `figureCrimson`) and the films'
                                          layout (`film`: `side`, the square, and `tint`, the tint half's first row);
                                          per extra (`extras`) its still's box, fade and tile, read off the still drawn
                                          over the card (no loop: it has no film); every file's sha256; and
                                          `films_digest`, the H.264 films' set (films_digest():
                                          the first 16 hex digits of the sha256 over each films/ name and its sha256),
                                          which names the desktop app's download and its folder
    contract/figures/watch/pack.json      the watches' run tables (2026-10-21: the game's sheets made the watchOS
                                          variant 104 MB, over App Store Connect's 75 MB, ITMS-90389 on build 17),
                                          which both watch apps bundle in place of QuestPump_FigArmor/pack.json at the
                                          same paths (pack.json, s/), cut from it by watch_tables, stdlib: per move
                                          only what the watches read (seq, ticks, fade, mirror, key; a variant its ref;
                                          an owner its sheet's layout, file and tint, and `from`, the game sheets and
                                          cells it is cut from), each owner with only the cells a watch shows (the runs
                                          its own and its variants' tables start in WATCH_PLAY_S, each such run's
                                          cross-fade partner, the key cells), a longer loop cut to those runs; with
                                          `source`, the game pack's digest, `hz` and `height`, WATCH_HEIGHT
    contract/figures/watch/s/<owner>.<sha8>.png and s/<owner>.t.<sha8>.png
                                          each owner's watch sheets, its base on the card and its tint on black: its
                                          kept cells cut from the game's sheets and shrunk to at most WATCH_HEIGHT px
                                          high, in a tight grid (write_watch); the base the smallest palette PNG whose
                                          worst cell meets the game's bar in the game's box at the watch's scale, the
                                          tint the game's 32 colours; named by their content as the game's are
    site/src/scripts/player.js            player.js as it stands, the site's copy (its pages composite the films with
                                          `addTint`; a processed module, so Vite hashes it; `--player` and `publish`
                                          write it)
    helper/dist/lib/player.js             player.js as it stands, the desktop app's copy (its figures' tint and
                                          still recolour, helper/dist/lib/figure.js; `--player` and `publish` write
                                          it)

`figures.py publish` runs it, and `make figures` or `figures.py films` after the build (the figures venv: numpy,
Pillow, ffmpeg, pngquant, oxipng): a film, pair or extra's still of this build (`build/loops/<owner>.mp4` with
`build/films/<owner>.mp4`, `build/stills/<owner>.key.png`, `.fault.png` and their `.t.png` tints,
`build/stills/<extra>.png` with its tint) replaces the committed one, a move or extra the build did not render keeps
its committed files (after a build only the moves it rendered:
the build folder is a cache other trees and runs share), every still, its tint and every owner's numbers are cut
afresh from the films (the same film gives the same bytes), a file the table no longer names is deleted (git keeps the
history), player.js is copied again to the site and the desktop app, and then the watches' pack is brought up to the
game's (write_watch, as `--watch` does). Names carry no hash but the watches' sheets: the app version is the cache
key, and an unchanged file keeps its blob.

    python3 -B tools/figures/publish.py --check   stdlib (`make check`, CI): fails when a library move
                                                  (contract/moves.json) has no entry, film (both encodings), still or
                                                  tint still, an owner with a fault no pair or no pair tints, an extra
                                                  (a motion file with "extra": true) no entry, numbers, or still and
                                                  tint of EXTRA_SIDE square, or a name a library move or owner holds,
                                                  when an
                                                  entry differs from what QuestPump_FigArmor/pack.json gives, when an
                                                  owner's numbers are missing or out of the square, when the crimson
                                                  differs from tokens.json's `figureCrimson` or the film layout from
                                                  this file's, when a film's track is not SIDE x (TINT + SIDE), not its
                                                  folder's codec (loops/ hvc1, films/ avc1) or not as many frames and
                                                  seconds as its owner's other film, when an owner's `loop` is not its
                                                  film's length, when a timing variant's pair is not its reference
                                                  move's, when `films_digest` is not the films', when a
                                                  file is missing or its sha256 differs from the table, when the folder
                                                  holds a file the table does not name, when watch/pack.json is not
                                                  what watch_tables gives from the game's pack (its source that pack's
                                                  digest), a watch sheet is missing, not named by its sha256 or not its
                                                  table's size, watch/ holds a file its table does not name or passes
                                                  WATCH_BUDGET bytes, or when the site's site/src/scripts/player.js or
                                                  the desktop app's helper/dist/lib/player.js is not player.js
    python3 -B tools/figures/publish.py --player  stdlib: player.js's two copies (the files above)
    tools/figures/.venv/bin/python -B tools/figures/publish.py --watch
                                                  the figures venv (numpy, Pillow, pngquant, oxipng, ssimulacra2): the
                                                  watches' pack alone, from QuestPump_FigArmor (every publish runs it
                                                  last); an owner whose table is unchanged and whose sheets exist keeps
                                                  them, and a file no table names is deleted
    python3 -B tools/figures/publish.py --zip <folder>
                                                  stdlib: <folder>/figures-<films_digest>.zip, the desktop app's
                                                  download (helper-release.yml): each films/ file under its table name,
                                                  stored, in name order with a fixed date, so a set gives the same
                                                  bytes; refused unless --check passes
"""
import hashlib
import json
import math
import os
import re
import shutil
import struct
import subprocess
import sys
import tempfile
from concurrent.futures import ThreadPoolExecutor

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
BUILD = os.path.join(HERE, "build")
PACK = os.path.join(ROOT, "QuestPump_FigArmor", "pack.json")
MOVES = os.path.join(ROOT, "contract", "moves.json")
ART = os.path.join(ROOT, "contract", "art")          # the motion files: an extra's says "extra": true
TOKENS = os.path.join(ROOT, "contract", "design", "tokens.json")
OUT = os.path.join(ROOT, "contract", "figures")
TABLE = "figures.json"
VERSION = 6
LOOPS, FILMS, STILLS, PAIRS = "loops", "films", "stills", "pairs"
# Each film folder's codec, as its MP4's sample entry names it: the phones' HEVC, the desktop app's H.264 (slice 35)
CODECS = {LOOPS: b"hvc1", FILMS: b"avc1"}
SIDES = ("key", "fault")       # a pair's two files, pairs/<owner>.<side>.png, each with its pairs/<owner>.<side>.t.png
TINTED = ".t.png"              # a still's tint beside it: stills/<owner>.t.png, pairs/<owner>.<side>.t.png
# The stacked film (slice 29): a SIDE-px square of base over a GUARD-px black band over a square of tint, so one
# decoder keeps the two in step. TINT is a multiple of x265's 64-px coding tree block (build.py pins ctu 64), so no
# block, nor its SAO parameters, holds both the card and the tint, where a phone would add the card in the class
# colour; the band ends the block row above the tint, so the deblocking filter across row TINT (3 px each side) mixes
# in only its black. The H.264 film keeps the layout: TINT is a multiple of its 16-px macroblock (and of the 8-px
# chroma block at half the height), and its deblocking filter also reaches 3 px across an edge
SIDE, GUARD = 480, 32
TINT = SIDE + GUARD            # the tint half's first row
# An extra's still (slice 45: the streak card's standard-bearer) is drawn larger than a move's on the share cards
# (1080 px wide), so it is cut at twice the film's square from the 1280-px render (build.py CELL), never upscaled there
EXTRA_SIDE = 2 * SIDE
MP4_BOX, PNG_SIGNATURE = b"ftyp", b"\x89PNG\r\n\x1a\n"
PLAYER = os.path.join(HERE, "player.js")
# player.js's copies, each byte for byte: the site's (a module its pages import) and the desktop app's
COPIES = (os.path.join(ROOT, "site", "src", "scripts", "player.js"),
          os.path.join(ROOT, "helper", "dist", "lib", "player.js"))
ZIP_DATE = (1980, 1, 1, 0, 0, 0)   # the zip format's first day, every entry's: one film set gives one zip's bytes
ABOUT = ("The move figures of the phones (design v3 §11) and the desktop app (slice 35), written by "
         "tools/figures/publish.py, never by hand: per library move the owner whose loops/<owner>.mp4 (HEVC, the "
         "phones'; films/<owner>.mp4, the same film in H.264, the desktop app's) and stills/<owner>.png it shows, the "
         "move itself (a timing variant's film is its own run table over its reference move's frames), the key pose's "
         "second in that film, whether the pack mirrors the move, where the owner has a fault, its caption key "
         "(`fault`: pairs/<owner>.key.png and pairs/<owner>.fault.png are its right-versus-wrong pair, slice 22; a "
         "timing variant's are its reference move's), where its loop shows more than one rep (a dead bug's two "
         "sides), the second in the film at which each rep but the last ends (counts; the last ends with the loop), "
         "and where the move has gear variants, `gear`: by equipment key the same for the figure drawn with that "
         "implement (its owner `<move>.<gear>`), which a player whose load the implement carries is shown (the core's "
         "implement with the player's gear), any other the move's own; "
         "per owner, as shares of the film's square and unmirrored, the "
         "box its figure and props fill over the whole loop (box: left, top, right, bottom, the bottom where it "
         "stands), the edges that cut its floor light (fade: left, right, bottom, null where none does) and the square "
         "a tile shows of its still (tile: x, y, side), and the film's own length in seconds (loop: its video track's "
         "samples' durations summed, its frames / 30, the same in both encodings), one loop at real tempo, the beat a "
         "rep count follows; the card the films are drawn on; the crimson (tokens.json figureCrimson) the stills are "
         "drawn in; the films' layout (film, slice 29): each film is side wide and tint + side high, the base in its "
         "top square, the tint (the cloth's light on black) in the square from row tint, shown as clamp(base + tint x "
         "colour / 255) per channel on the stored values, crimson where no game sheet names a colour; each still's and "
         "pair still's tint beside it (<name>.t.png), which takes the still drawn over the card (S; a pair still is "
         "opaque already) into a colour as clamp(S - tint x crimson + tint x colour); each file's sha256; and "
         "films_digest, the first 16 hex digits of the sha256 over a `<name> <sha256>` line per films/ file in name "
         "order, which names the desktop app's download, figures-<films_digest>.zip (each films/ file under its name "
         "here), and the folder it keeps it in. Beside the moves, `extras` names each figure that is no library "
         "move (tools/figures/motion.py `extra`; the streak card's standard-bearer, streak_banner) with its still's "
         "box, fade and tile as an owner's, read off its still: stills/<extra>.png, " + str(EXTRA_SIDE) + " px "
         "square, its key pose in crimson keyed as an owner's still is, with its tint stills/<extra>.t.png; an "
         "extra has no film and no loop.")
# Reading a loop's frames (8-bit RGB as ffmpeg decodes them): the card itself decodes within NOISE levels of its mode;
# the figure and its props are more than STRONG levels lighter than the card; a floor's light is FLOOR to STRONG
# levels lighter and warm (red at least WARM above blue: the dark armour is blue); an edge cuts it where at least CUT
# such pixels lie within EDGE px inside it in one frame, in the floor's band (BAND of the side above the feet).
NOISE, STRONG, FLOOR, WARM, CUT, EDGE, BAND = 2, 40, 6, 3, 10, 3, 0.125
TILE_MARGIN = 0.06     # a tile's square: the key pose's figure and props, with this share of their side round them
PLACES = 4             # the decimals of the shares and the seconds
NUMBERS = {"box", "fade", "tile", "loop"}   # an owner's numbers in the table (an extra's: these but the loop)
# The watches' own pack, contract/figures/watch/ (2026-10-21): pack.json and s/, cut from the game's pack (write_watch)
WATCH, WATCH_TABLE, WATCH_SHEETS = "watch", "pack.json", "s"
# A watch plays a figure this many seconds at most, then holds its key pose (apple/Watch/Views/FigureView.swift
# playLimit, android/wear ui/FigureView.kt PLAY_LIMIT; with Reduce Motion min(loop, this)): a run that starts later
# never shows there
WATCH_PLAY_S = 5
# A watch cell's height at most (px): the countdown draws the figure 44 pt high (88 px at 2x) and the legend unlock 96
# pt (192 px), each fitted to that height, so 128 px serves the countdown with room and the unlock at 1.5x
WATCH_HEIGHT = 128
# A watch sheet's longest side: 16 MiB decoded at most, the game's largest sheet (pack.MAX_SHEET), as each watch keeps
# six sheets decoded (WatchFigures keptImages, KEPT_IMAGES)
WATCH_MAX_SIDE = 2048
# The palettes a watch sheet tries before the game's (pack.PALETTES), smallest first: a cell at under half the game's
# area holds fewer colours, and most sheets meet the bar at 24 to 48
WATCH_PALETTES = (16, 24, 32, 48)
# The watches' pack in bytes at most (its sheets and pack.json): App Store Connect refuses a watchOS variant over 75 MB
# after app thinning (ITMS-90389 on build 17, 2026-10-21, 104 MB with the game's sheets), less about 20 MB kept for the
# rest of the thinned watch app (one binary slice about 5 MB, the fonts, the asset catalog, the JSON it bundles)
WATCH_BUDGET = 55_000_000


def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as fh:
        for chunk in iter(lambda: fh.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def key_seconds(owner, hz):
    """Where the key pose shows in the owner's loop, in seconds: the middle of the first run on its key cell (the loop
    plays the stored frames by the run table at `hz`, build.py write_loop)."""
    seq, cell, ticks = owner["seq"], owner.get("key", 0), 0
    for i in range(0, len(seq) - 1, 2):
        if seq[i] == cell:
            return round((ticks + seq[i + 1] / 2) / hz, PLACES)
        ticks += seq[i + 1]
    raise SystemExit(f"publish: the key cell {cell} is not in the run table")


def entries(pack):
    """{move: {owner, key, mirror[, fault][, counts][, gear]}} from the pack's run tables, every move of the pack, each
    its own owner (a timing variant's film plays its own table, build.py write_loop; its fault is its reference's, as
    the pack gives it); `counts`, where its loop shows more than one rep, the second in the film at which each rep but
    the last ends (the pack's ticks over hz; the last ends with the film, at its owner's `loop`); a gear variant's (its
    key `<move>.<gear>`, tools/figures/motion.py) under its move's `gear`, by the gear."""
    out, geared = {}, {}
    for name, entry in sorted(pack["moves"].items()):
        e = {"owner": name, "key": key_seconds(entry, pack["hz"]), "mirror": bool(entry.get("mirror"))}
        if entry.get("fault_cap"):
            e["fault"] = entry["fault_cap"]
        if entry.get("counts"):
            e["counts"] = [round(t / pack["hz"], PLACES) for t in entry["counts"]]
        move, _, gear = name.partition(".")
        if gear:
            geared.setdefault(move, {})[gear] = e
        else:
            out[move] = e
    for move, gear in geared.items():
        if move not in out:
            sys.exit(f"publish: the pack holds gear variants of {move}, which it does not hold")
        out[move]["gear"] = gear
    return out


def shown(moves):
    """Every figure the table's moves show: each move's own entry and its gear variants'."""
    for e in moves.values():
        yield e
        yield from e.get("gear", {}).values()


def pair_files(owner):
    """An owner's pair files: pairs/<owner>.<side>.png and their tints (an owner's name may hold a dot: a gear
    variant's `<move>.<gear>`)."""
    return [f"{PAIRS}/{owner}.{side}{ext}" for side in SIDES for ext in (".png", TINTED)]


def pair_owner(name):
    """A pair file's owner and the rest of its name ("key.png", "fault.t.png"), read from the end, since an owner's
    name may hold a dot."""
    base = name[len(PAIRS) + 1:]
    for side in SIDES:
        for ext in (TINTED, ".png"):                 # the tint first: ".t.png" ends with ".png" too
            tail = f".{side}{ext}"
            if base.endswith(tail):
                return base[:-len(tail)], tail[1:]
    raise ValueError(f"{name}: no pair file")


def drawn_from(pack, owner):
    """The move whose cells an owner's pack entry plays: a timing variant's reference, else the owner itself; its pair
    files are that move's (build.py writes a pair per rendered move)."""
    return pack["moves"][owner].get("ref", owner)


def pair_names(moves):
    """The pair files the table's moves show: those of each owner with a fault."""
    return {n for e in shown(moves) if e.get("fault") for n in pair_files(e["owner"])}


def library():
    with open(MOVES) as fh:
        return {m["key"] for m in json.load(fh)["moves"].values()}


def extras():
    """The extras' names, sorted: every motion file in contract/art that says `"extra": true` (motion.py; its `move`,
    else its file's name). A file that does not read is left to `figures.py check`, which fails on it."""
    out = []
    for name in sorted(os.listdir(ART)):
        if not name.endswith(".json") or name == "approvals.json":
            continue
        try:
            spec = load(os.path.join(ART, name))
        except (OSError, ValueError):
            continue
        if isinstance(spec, dict) and spec.get("extra") is True:
            out.append(spec.get("move", name[:-len(".json")]))
    return out


def extra_files(name):
    """An extra's two files in the table: its still and its tint."""
    return [f"{STILLS}/{name}.png", f"{STILLS}/{name}{TINTED}"]


def png_size(path):
    """A PNG's (width, height) from its IHDR chunk (stdlib), or None for a file that is no PNG."""
    with open(path, "rb") as fh:
        head = fh.read(24)
    if not head.startswith(PNG_SIGNATURE) or head[12:16] != b"IHDR":
        return None
    return struct.unpack(">II", head[16:24])


def load(path):
    with open(path) as fh:
        return json.load(fh)


def crimson():
    """The figure's default colour as #RRGGBB: tokens.json `figureCrimson`, Figure.lua's CRIMSON to the byte."""
    return load(TOKENS)["color"]["figureCrimson"]


def film_track(path):
    """An MP4's video track as its boxes declare it (stdlib: a walk of moov to the trak whose tkhd has a width):
    (width, height, its sample entry's codec, its frame count, its length in seconds), or None when it has none."""
    with open(path, "rb") as fh:
        data = fh.read()

    def boxes(start, end):
        while start + 8 <= end:
            size, kind = struct.unpack(">I4s", data[start:start + 8])
            head = 8
            if size == 1:                                    # a 64-bit size follows the type
                size, head = struct.unpack(">Q", data[start + 8:start + 16])[0], 16
            elif size == 0:                                  # the box runs to the end of its parent
                size = end - start
            if size < head:
                return
            yield kind, start + head, start + size
            start += size

    def child(start, end, path):
        """The payload (start, end) of the box at `path`, a list of box types, under start..end; None without it."""
        for kind, a, b in boxes(start, end):
            if kind == path[0]:
                return (a, b) if len(path) == 1 else child(a, b, path[1:])
        return None

    for kind, a, b in boxes(0, len(data)):
        for trak, c, d in boxes(a, b) if kind == b"moov" else ():
            tkhd = child(c, d, [b"tkhd"]) if trak == b"trak" else None
            if not tkhd:
                continue
            at = tkhd[0] + (88 if data[tkhd[0]] == 1 else 76)    # version 1 widens the times and the duration
            w, h = struct.unpack(">II", data[at:at + 8])
            if not w:                                            # 16.16 fixed point; an audio track declares 0
                continue
            mdhd = child(c, d, [b"mdia", b"mdhd"])
            stbl = child(c, d, [b"mdia", b"minf", b"stbl"])
            stsd, stsz, stts = (child(*stbl, [box]) if stbl else None for box in (b"stsd", b"stsz", b"stts"))
            # stsd: version and flags, the entry count, then the first entry's size and type; stsz: version and flags,
            # one size for every sample (or 0), the sample count; mdhd (ISO/IEC 14496-12 8.4.2): version and flags,
            # the creation and modification times (64-bit in version 1), then the timescale (units a second); stts:
            # version and flags, the entry count, then each run's sample count and sample duration in those units
            codec = data[stsd[0] + 12:stsd[0] + 16] if stsd else None
            count = struct.unpack(">I", data[stsz[0] + 8:stsz[0] + 12])[0] if stsz else None
            # the length is the samples' durations summed (its frames / 30), never mdhd's duration, which the H.264
            # muxer lengthens by its B-frame delay on some films (12 of the desktop app's, 2026-10-09)
            seconds = None
            if mdhd and stts:
                scale = struct.unpack_from(">I", data, mdhd[0] + (20 if data[mdhd[0]] == 1 else 12))[0]
                runs = struct.unpack_from(">I", data, stts[0] + 4)[0]
                table = data[stts[0] + 8:stts[0] + 8 + 8 * runs]
                ticks = sum(n * delta for n, delta in struct.iter_unpack(">II", table))
                seconds = ticks / scale if scale else None
            return w >> 16, h >> 16, codec, count, seconds
    return None


# ---------------------------------------------------------------- the stills and the owners' numbers (the venv)

def frames(path):
    """Every frame of a film as ffmpeg decodes it, split into its halves: (base, tint), each (n, SIDE, SIDE, 3)
    uint8. A film of another size (a loop from before slice 29) stops the run."""
    import numpy as np
    track = film_track(path)
    size = track and track[:2]
    if size != (SIDE, TINT + SIDE):
        sys.exit(f"publish: {os.path.relpath(path, ROOT)} is {size}, not the {SIDE} x {TINT + SIDE} film "
                 f"(figures.py films rebuilds it)")
    raw = subprocess.run(["ffmpeg", "-v", "error", "-i", path, "-f", "rawvideo", "-pix_fmt", "rgb24", "-"],
                         capture_output=True, check=True).stdout
    f = np.frombuffer(raw, np.uint8).reshape(-1, TINT + SIDE, SIDE, 3)
    return f[:, :SIDE], f[:, TINT:]


def composite(base, tint, colour):
    """The frames as a phone shows them in `colour` (#RRGGBB): clamp(base + tint x colour / 255) per channel,
    rounded."""
    import numpy as np
    c = np.array([int(colour[i:i + 2], 16) for i in (1, 3, 5)], np.float32) / 255
    out = np.empty_like(base)
    for s in range(0, len(base), 32):           # a long flow's frames as floats at once would take gigabytes
        out[s:s + 32] = np.clip(np.rint(base[s:s + 32] + tint[s:s + 32] * c), 0, 255)
    return out


def write_tint(tint, out):
    """A tint still: the cloth's light on black (an 8-bit RGB array) as a truecolour PNG through oxipng, lossless: a
    few KB, as the tint is nearly all black (a 256-colour palette saved half of that and lost up to 6 levels on the
    probe of 2026-10-06)."""
    from PIL import Image
    Image.fromarray(tint, "RGB").save(out)
    subprocess.run(["oxipng", "-q", "-o", "4", "--strip", "all", out], check=True)


def keyed(frame, card):
    """A frame with its card keyed out both ways: alpha is the most any channel moved off the card (as a share of the
    room it had to move), and the colour what lands on the card at that alpha; within NOISE of the card is clear."""
    import numpy as np
    p, c = frame.astype(np.float64), card.astype(np.float64)
    d = p - c
    d = np.sign(d) * np.maximum(np.abs(d) - NOISE, 0)
    a = np.where(d > 0, d / (255 - c), -d / np.maximum(c, 1)).max(-1)
    rgb = np.where(a[..., None] > 0, c + d / np.maximum(a[..., None], 1e-9), 0)
    out = np.zeros(frame.shape[:2] + (4,))
    out[..., :3] = np.clip(rgb, 0, 255)
    out[..., 3] = a * 255
    return np.round(out).astype(np.uint8)


def share(px, side, down=False):
    """`px` as a share of the side, to PLACES decimals; rounded down for a tile, which must stay inside the square."""
    if down:
        return math.floor(px / side * 10 ** PLACES) / 10 ** PLACES
    return round(px / side, PLACES)


def owner_numbers(f, card, key):
    """An owner's numbers from its loop's frames `f`: box, fade and tile as shares of the side."""
    import numpy as np
    n, side = f.shape[0], f.shape[1]
    lift = (f.astype(np.int16) - card).max(-1)                   # how much lighter than the card, per pixel
    strong = lift > STRONG
    sy, sx = np.nonzero(strong.any(0))
    left, top, right, feet = int(sx.min()), int(sy.min()), int(sx.max()) + 1, int(sy.max()) + 1
    ys, xs = np.nonzero((lift > NOISE + 1).any(0))
    x0, x1, y1 = int(xs.min()), int(xs.max()) + 1, int(ys.max()) + 1
    floor = (lift >= FLOOR) & (lift <= STRONG) & ((f[..., 0].astype(np.int16) - f[..., 2]) >= WARM)
    band = slice(max(0, feet - round(BAND * side)), side)

    def cut(region):
        return int(region.reshape(n, -1).sum(1).max()) >= CUT

    fade = [share(x0, side) if cut(floor[:, band, x0:x0 + EDGE]) else None,
            share(x1, side) if cut(floor[:, band, x1 - EDGE:x1]) else None,
            share(y1, side) if cut(floor[:, y1 - EDGE:y1, :]) else None]
    ky, kx = np.nonzero(strong[key])
    bx0, bx1, by0, by1 = int(kx.min()), int(kx.max()) + 1, int(ky.min()), int(ky.max()) + 1
    s = min(side, round(max(bx1 - bx0, by1 - by0) * (1 + 2 * TILE_MARGIN)))
    tx = min(max(0, round((bx0 + bx1 - s) / 2)), side - s)
    ty = min(max(0, round((by0 + by1 - s) / 2)), side - s)
    box = [share(left, side), share(top, side), share(right, side), share(feet, side)]
    return {"box": box, "fade": fade, "tile": [share(tx, side, True), share(ty, side, True), share(s, side, True)]}


def write_still(shown, tint, card, out):
    """Writes a still: `shown` (a frame in its colour on the card, `card` as that frame holds it) with the card keyed
    out (`keyed`) as a 256-colour palette PNG at `out`, and the frame's tint half `tint` beside it (<out without
    .png>.t.png, write_tint)."""
    import numpy as np
    from PIL import Image
    raw = out + ".rgba.png"
    Image.fromarray(keyed(shown, card), "RGBA").save(raw)
    subprocess.run(["pngquant", "--force", "--speed", "1", "256", "--output", out, raw], check=True)
    subprocess.run(["oxipng", "-q", "-o", "4", "--strip", "all", out], check=True)
    os.remove(raw)
    write_tint(np.ascontiguousarray(tint), out[:-len(".png")] + TINTED)


def cut_still(owner, key_s, hz, out, colour):
    """Writes an owner's still (its film's key-pose frame in `colour`, the crimson, keyed) to `out` and that frame's
    tint half beside it (<out without .png>.t.png), and returns its numbers, read off the frames in `colour`. The frame
    is the one a player seeking to the key's second shows: the last frame that starts at or before it."""
    import numpy as np
    base, tint = frames(os.path.join(OUT, LOOPS, owner + ".mp4"))
    f = composite(base, tint, colour)
    values, counts = np.unique(f[0].reshape(-1, 3), axis=0, return_counts=True)
    card = values[counts.argmax()].astype(np.int16)              # the card as this film decodes it
    key = min(int(key_s * hz + 1e-6), f.shape[0] - 1)
    write_still(f[key], tint[key], card, out)
    return owner_numbers(f, card, key)


def still_numbers(path, card):
    """An extra's numbers (an owner's but the loop: it has no film), read off its still drawn over the card `card`
    (#RRGGBB), which gives the frame it was cut from to within the palette's error (`keyed`)."""
    import numpy as np
    from PIL import Image
    rgba = np.asarray(Image.open(path).convert("RGBA")).astype(np.float64)
    bg = np.array([int(card[i:i + 2], 16) for i in (1, 3, 5)], np.float64)
    a = rgba[..., 3:4] / 255
    shown = np.clip(np.rint(rgba[..., :3] * a + bg * (1 - a)), 0, 255).astype(np.uint8)
    return owner_numbers(shown[None], bg.astype(np.int16), 0)


def take(src, dst):
    """Copies the build's `src` over the committed `dst` where they differ."""
    if not os.path.exists(dst) or sha256(src) != sha256(dst):
        shutil.copyfile(src, dst)


def films_digest(files):
    """The H.264 films' set (the table's `files`): the first 16 hex digits of the sha256 over a `<name> <sha256>` line
    per films/ file in name order."""
    lines = "".join(f"{n} {files[n]}\n" for n in sorted(files) if n.startswith(FILMS + "/"))
    return hashlib.sha256(lines.encode()).hexdigest()[:16]


def publish(built=None):
    """Brings the build's films (both encodings) and pairs into contract/figures, every owner's, or with `built` (the
    moves a build filmed) only theirs, the other owners keeping their committed files (the build folder is a cache
    that may hold renders of other trees and runs); a timing variant's pair is its reference move's build stills under
    its own name (drawn_from). Then cuts every owner's still, its tint and its numbers from its film, reads its loop
    off the film, writes the table and copies player.js again (players()). An extra's still and tint come from the build
    as a film does (`built` naming it, or every extra), and its numbers are read off its still."""
    import pack as P
    pack = load(PACK)
    moves = entries(pack)
    owners = sorted({e["owner"] for e in shown(moves)})
    extra = extras()
    fresh = set(owners) | set(extra) if built is None else set(built)
    colour = crimson()
    files, missing = {}, []
    for folder in (LOOPS, FILMS, STILLS, PAIRS):
        os.makedirs(os.path.join(OUT, folder), exist_ok=True)
    for owner in owners:
        for folder in (LOOPS, FILMS):
            name = f"{folder}/{owner}.mp4"
            src, dst = os.path.join(BUILD, folder, owner + ".mp4"), os.path.join(OUT, name)
            if owner in fresh and os.path.exists(src):
                take(src, dst)
            elif not os.path.exists(dst):
                missing.append(name)
                continue
            files[name] = sha256(dst)
    for name in sorted(pair_names(moves)):
        owner, rest = pair_owner(name)
        src, dst = os.path.join(BUILD, STILLS, f"{drawn_from(pack, owner)}.{rest}"), os.path.join(OUT, name)
        if owner in fresh and os.path.exists(src):
            take(src, dst)
        elif not os.path.exists(dst):
            missing.append(name)
            continue
        files[name] = sha256(dst)
    for name in extra:
        for rel in extra_files(name):
            src, dst = os.path.join(BUILD, rel), os.path.join(OUT, rel)
            if name in fresh and os.path.exists(src):
                take(src, dst)
            elif not os.path.exists(dst):
                missing.append(rel)
                continue
            files[rel] = sha256(dst)
    if missing:
        sys.exit(f"publish: {len(missing)} files neither built nor committed (make figures MOVES=...): {missing}")
    key_of = {e["owner"]: e["key"] for e in shown(moves)}
    # the stills as cut, before they replace the committed ones: a folder of this run's own, since the build folder is
    # shared by every tree and two runs at once would take each other's cuts
    os.makedirs(BUILD, exist_ok=True)
    tmp = tempfile.mkdtemp(prefix="cut-", dir=BUILD)

    def one(owner):
        cut = os.path.join(tmp, owner + ".png")
        numbers = cut_still(owner, key_of[owner], pack["hz"], cut, colour)
        numbers["loop"] = round(film_track(os.path.join(OUT, LOOPS, owner + ".mp4"))[4], PLACES)
        for name in (owner + ".png", owner + TINTED):
            take(os.path.join(tmp, name), os.path.join(OUT, STILLS, name))
        return owner, numbers

    try:
        with ThreadPoolExecutor(max(2, (os.cpu_count() or 8) - 2)) as pool:
            numbers = dict(pool.map(one, owners))
    finally:
        shutil.rmtree(tmp, ignore_errors=True)
    for owner in owners:
        for name in (owner + ".png", owner + TINTED):
            files[f"{STILLS}/{name}"] = sha256(os.path.join(OUT, STILLS, name))
    pruned = []
    for folder in (LOOPS, FILMS, STILLS, PAIRS):
        for name in sorted(os.listdir(os.path.join(OUT, folder))):
            if not name.startswith(".") and f"{folder}/{name}" not in files:
                os.remove(os.path.join(OUT, folder, name))
                pruned.append(f"{folder}/{name}")
    table = {"about": ABOUT, "version": VERSION, "card": P.BG_HEX, "crimson": colour,
             "film": {"side": SIDE, "tint": TINT}, "moves": moves, "owners": numbers,
             "extras": {name: still_numbers(os.path.join(OUT, STILLS, name + ".png"), P.BG_HEX) for name in extra},
             "files": files, "films_digest": films_digest(files)}
    with open(os.path.join(OUT, TABLE), "w") as fh:
        json.dump(table, fh, indent=1, sort_keys=True, ensure_ascii=False)
        fh.write("\n")
    players()
    size = {kind: sum(os.path.getsize(os.path.join(OUT, n)) for n in files if n.startswith(kind + "/"))
            for kind in (LOOPS, FILMS, STILLS, PAIRS)}
    tints = sum(os.path.getsize(os.path.join(OUT, n)) for n in files if n.endswith(TINTED))
    print(json.dumps({"moves": len(moves), "owners": len(owners), "extras": len(extra), "film_bytes": size[LOOPS],
                      "desktop_film_bytes": size[FILMS], "still_bytes": size[STILLS], "pair_bytes": size[PAIRS],
                      "tint_still_bytes": tints, "pruned": pruned}))
    write_watch()
    return table


# ---------------------------------------------------------------- the watches' pack (tables stdlib, sheets the venv)

def watch_runs(entry, until):
    """The runs (1-based, a prefix) of a run table a watch plays in its first `until` ticks: each run that starts
    before then (every run of a loop no longer than that), and after the last of them the next run where it
    cross-fades into it, its blend partner."""
    seq, kept, start = entry["seq"], [], 0
    runs = len(seq) // 2
    for run in range(1, runs + 1):
        if start >= until:
            break
        kept.append(run)
        start += seq[2 * run - 1]
    if kept[-1] in entry["fade"] and kept[-1] < runs:
        kept.append(kept[-1] + 1)
    return kept


def watch_size(size):
    """An owner's drawn size (w, h) on the watch, scaled to at most WATCH_HEIGHT high (never up), and that scale."""
    w, h = size
    s = min(1.0, WATCH_HEIGHT / h)
    return [round(w * s), round(h * s)], s


def watch_layout(n, size, pad):
    """A watch sheet's layout for n cells of `size` with `pad` px of card round each: a tight grid (no power of two,
    which would only add decoded memory on the watch) of the columns that give the shortest longest side, then the
    least area, no side over WATCH_MAX_SIDE."""
    cw, ch = size[0] + 2 * pad, size[1] + 2 * pad
    best = None
    for cols in range(1, n + 1):
        W, H = cols * cw, -(-n // cols) * ch
        if W <= WATCH_MAX_SIDE and H <= WATCH_MAX_SIDE and (best is None or (max(W, H), W * H) < best[0]):
            best = ((max(W, H), W * H), [W, H], cols)
    if best is None:
        sys.exit(f"publish: {n} watch cells of {size[0]} x {size[1]} fit no sheet of {WATCH_MAX_SIDE} px a side")
    return {"sheet": best[1], "cell": [cw, ch], "cols": best[2], "pad": pad, "size": size, "n": n}


def watch_tables(pack):
    """The watches' pack.json from the game's pack (its file and tint names aside, which the encoded sheets give):
    per move only what the watches read (seq, ticks, fade, mirror, key; a variant its ref; an owner its sheet's layout
    at the watch's size, `n` and `from`, the game sheets and cells it is cut from). Each owner keeps the cells its own
    and its variants' tables show in a watch's play (watch_runs) and their key cells, renumbered in their order; a table
    whose loop runs longer is cut to its kept runs, its ticks their sum (WATCH_PLAY_S or more: the play and Reduce
    Motion's one rep run as before), its fade those runs' but a cut table's last, whose partner the cut dropped."""
    hz, moves = pack["hz"], pack["moves"]
    until = WATCH_PLAY_S * hz
    kept = {name: watch_runs(e, until) for name, e in moves.items()}
    used = {}
    for name, e in moves.items():
        owner = e.get("ref", name)
        used.setdefault(owner, set()).update([e["seq"][2 * r - 2] for r in kept[name]] + [e.get("key", 0)])
    out = {}
    for name, e in sorted(moves.items()):
        owner = e.get("ref", name)
        number = {c: i for i, c in enumerate(sorted(used[owner]))}
        runs, cut = kept[name], len(kept[name]) < len(e["seq"]) // 2
        seq = e["seq"]
        t = {"seq": [v for r in runs for v in (number[seq[2 * r - 2]], seq[2 * r - 1])],
             "ticks": sum(seq[2 * r - 1] for r in runs) if cut else e["ticks"],
             "fade": [r for r in e["fade"] if r < len(runs) or (r == len(runs) and not cut)],
             "mirror": bool(e.get("mirror")), "key": number[e.get("key", 0)]}
        if "ref" in e:
            t["ref"] = e["ref"]
        else:
            size, _ = watch_size(e["size"])
            t.update(watch_layout(len(number), size, e["pad"]))
            t["from"] = {"file": e["file"], "tint": e["tint"], "cells": sorted(number)}
        out[name] = t
    return {"hz": hz, "source": pack["digest"], "height": WATCH_HEIGHT, "moves": out}


def watch_file(owner, kind, path):
    """The table's name for an owner's watch sheet `kind` ("file" or "tint") holding the bytes at `path`:
    s/<owner>.<sha8>.png and s/<owner>.t.<sha8>.png, each sha8 the first 8 hex digits of the file's sha256, as the
    game's sheets are named."""
    return f"{WATCH_SHEETS}/{owner}.{'t.' if kind == 'tint' else ''}{sha256(path)[:8]}.png"


def write_watch():
    """Writes contract/figures/watch/ from the game's pack (the figures venv: numpy, Pillow, pngquant, oxipng,
    ssimulacra2): pack.json (watch_tables) and each owner's sheets, its kept cells cut from the game's sheets and
    shrunk to the watch's size (Lanczos on RGB), packed on the card (the tint on black), the base the smallest of
    WATCH_PALETTES and then the game's palettes whose worst cell scores pack.TARGET in the game's box at the watch's
    scale (no smaller than the cell), the tint as the game's (pack.encode_tint). An owner whose table (its `from`
    among it) is unchanged and whose sheets exist keeps them; a file no table names is deleted."""
    import numpy as np
    from PIL import Image
    import pack as P
    game = load(PACK)
    table = watch_tables(game)
    folder = os.path.join(OUT, WATCH)
    os.makedirs(os.path.join(folder, WATCH_SHEETS), exist_ok=True)
    path = os.path.join(folder, WATCH_TABLE)
    old = load(path)["moves"] if os.path.exists(path) else {}
    os.makedirs(BUILD, exist_ok=True)
    tmp = tempfile.mkdtemp(prefix="watch-", dir=BUILD)       # this run's own: two runs at once never share a cut

    def one(owner):
        entry, was = table["moves"][owner], old.get(owner, {})
        if ({k: v for k, v in was.items() if k not in ("file", "tint")} == entry
                and all(os.path.isfile(os.path.join(folder, was.get(k, ""))) for k in ("file", "tint"))):
            return owner, {k: was[k] for k in ("file", "tint")}, None
        src = game["moves"][owner]
        lay = {k: entry[k] for k in ("sheet", "cell", "cols", "pad", "size", "n")}
        scale = watch_size(src["size"])[1]
        work = os.path.join(tmp, owner)
        os.makedirs(work)
        names, result = {}, None
        for kind, bg in (("file", P.BG), ("tint", (0, 0, 0))):
            cells = P.unpack(P.load8(os.path.join(os.path.dirname(PACK), src[kind])),
                             {k: src[k] for k in ("sheet", "cell", "cols", "pad", "size", "n")})
            if scale < 1:
                cells = [np.asarray(Image.fromarray(np.ascontiguousarray(cells[c])).resize(
                    tuple(entry["size"]), Image.Resampling.LANCZOS)) for c in entry["from"]["cells"]]
            else:
                cells = [cells[c] for c in entry["from"]["cells"]]
            raw, out = os.path.join(work, f"{kind}.src.png"), os.path.join(work, f"{kind}.png")
            Image.fromarray(P.pack_sheet(cells, lay, bg)).save(raw, compress_level=1)
            if kind == "file":
                box = max(*entry["size"], round(game["box"] * scale))
                colours, scores = P.encode_to_target(raw, cells, lay, out, os.path.join(work, "score"),
                                                     palettes=WATCH_PALETTES + P.PALETTES, box=box)
                result = (colours, min(scores))
            else:
                P.encode_tint(raw, out)
            names[kind] = watch_file(owner, kind, out)
            dst = os.path.join(folder, names[kind])
            if not os.path.exists(dst):
                os.replace(out, dst)
        shutil.rmtree(work, ignore_errors=True)
        return owner, names, result

    owners = sorted(n for n, e in table["moves"].items() if "ref" not in e)
    try:
        with ThreadPoolExecutor(max(2, (os.cpu_count() or 8) // 4)) as pool:
            done = list(pool.map(one, owners))
    finally:
        shutil.rmtree(tmp, ignore_errors=True)
    encoded = {}
    for owner, names, result in done:
        table["moves"][owner].update(names)
        if result:
            encoded[owner] = result
    with open(path, "w") as fh:
        json.dump(table, fh, sort_keys=True, separators=(",", ":"))
        fh.write("\n")
    named = {WATCH_TABLE} | {e[k] for e in table["moves"].values() for k in ("file", "tint") if k in e}
    pruned = []
    for base, _, files in os.walk(folder):
        for n in sorted(files):
            rel = os.path.relpath(os.path.join(base, n), folder)
            if not n.startswith(".") and rel not in named:
                os.remove(os.path.join(base, n))
                pruned.append(rel)
    sheets = sum(os.path.getsize(os.path.join(folder, n)) for n in named if n != WATCH_TABLE)
    palettes = {}
    for colours, _ in encoded.values():
        palettes[colours] = palettes.get(colours, 0) + 1
    largest = max((e["sheet"] for e in table["moves"].values() if "sheet" in e), key=lambda s: s[0] * s[1])
    print(json.dumps({"watch_owners": len(owners), "watch_encoded": len(encoded),
                      "watch_kept": len(owners) - len(encoded),
                      "watch_cells": sum(e["n"] for e in table["moves"].values() if "n" in e),
                      "watch_sheet_bytes": sheets, "watch_table_bytes": os.path.getsize(path),
                      "watch_bytes": sheets + os.path.getsize(path), "watch_budget": WATCH_BUDGET,
                      "watch_palettes": dict(sorted(palettes.items())),
                      "watch_worst": min((s for _, s in encoded.values()), default=None),
                      "watch_largest_sheet": largest, "watch_pruned": pruned}))


def watch_problems():
    """The problems of contract/figures/watch/ (stdlib): its pack.json not what watch_tables gives from the game's
    pack (its source the game pack's digest), a sheet missing, misnamed (its sha8) or not its `sheet` size, a file no
    table names, or the folder over WATCH_BUDGET."""
    folder = os.path.join(OUT, WATCH)
    path = os.path.join(folder, WATCH_TABLE)
    if not os.path.exists(path):
        return [f"{WATCH}/{WATCH_TABLE} is missing (publish.py --watch)"]
    have, game = load(path), load(PACK)
    want, problems = watch_tables(game), []
    if have.get("source") != game["digest"]:
        problems.append(f"{WATCH}/{WATCH_TABLE}: source {have.get('source')!r}, the game's pack is "
                        f"{game['digest']!r} (publish.py --watch)")
    for key in ("hz", "height"):
        if have.get(key) != want[key]:
            problems.append(f"{WATCH}/{WATCH_TABLE}: {key} {have.get(key)!r}, not {want[key]!r}")
    moves, derived = have.get("moves", {}), want["moves"]
    at = f"{WATCH}/{WATCH_TABLE}"
    problems += [f"{at}: no entry for {m} (publish.py --watch)" for m in sorted(set(derived) - set(moves))]
    problems += [f"{at}: {m} is no move of the game's pack" for m in sorted(set(moves) - set(derived))]
    named = {WATCH_TABLE}
    for name in sorted(set(moves) & set(derived)):
        entry = moves[name]
        if {k: v for k, v in entry.items() if k not in ("file", "tint")} != derived[name]:
            problems.append(f"{at}: {name} differs from what the game's pack gives (publish.py --watch)")
        if "ref" in derived[name]:
            continue
        for kind in ("file", "tint"):
            rel = entry.get(kind)
            full = os.path.join(folder, rel) if isinstance(rel, str) else None
            if not (full and os.path.isfile(full)):
                problems.append(f"{at}: {name}.{kind} {rel!r} is missing")
                continue
            named.add(rel)
            if rel != watch_file(name, kind, full):
                problems.append(f"{WATCH}/{rel}: its name is not {name}'s and its sha256's first 8 hex digits")
            if png_size(full) != tuple(derived[name]["sheet"]):
                problems.append(f"{WATCH}/{rel}: {png_size(full)}, its table's sheet {tuple(derived[name]['sheet'])}")
    total = 0
    for base, dirs, files in os.walk(folder):
        dirs[:] = [d for d in dirs if not d.startswith(".")]
        for n in (n for n in files if not n.startswith(".")):
            rel = os.path.relpath(os.path.join(base, n), folder)
            total += os.path.getsize(os.path.join(base, n))
            if rel not in named:
                problems.append(f"{WATCH}/{rel}: a stray file (publish.py --watch prunes it)")
    if total > WATCH_BUDGET:
        problems.append(f"{WATCH}/: {total:,} bytes, over its budget of {WATCH_BUDGET:,} (App Store Connect's 75 MB "
                        f"for a thinned watchOS variant, less the rest of the watch app; publish.py WATCH_BUDGET)")
    return problems


# ---------------------------------------------------------------- player.js's copies and the desktop app's download

def players():
    """Writes player.js's copies (COPIES) where they differ."""
    for copy in COPIES:
        os.makedirs(os.path.dirname(copy), exist_ok=True)
        take(PLAYER, copy)


def player_problems():
    """A copy of player.js that is missing or differs from it; an empty list when both are it."""
    return [f"{os.path.relpath(copy, ROOT)} is not tools/figures/player.js (publish.py --player)" for copy in COPIES
            if not os.path.isfile(copy) or sha256(copy) != sha256(PLAYER)]


def write_zip(folder):
    """The desktop app's download, <folder>/figures-<films_digest>.zip: every films/ file under its name in the table,
    stored (the films are compressed already), in name order, each dated ZIP_DATE with fixed attributes, so one set
    gives the same bytes on every system. Refused while --check finds a problem. Returns its path."""
    import zipfile
    found = check()
    if found:
        sys.exit(f"publish: no zip while --check fails ({len(found)} problems, the first: {found[0]})")
    table = load(os.path.join(OUT, TABLE))
    os.makedirs(folder, exist_ok=True)
    path = os.path.join(folder, f"figures-{table['films_digest']}.zip")
    part = path + ".part"
    with zipfile.ZipFile(part, "w", zipfile.ZIP_STORED) as z:
        for name in sorted(n for n in table["files"] if n.startswith(FILMS + "/")):
            info = zipfile.ZipInfo(name, ZIP_DATE)
            info.create_system, info.external_attr = 3, 0o644 << 16    # Unix, rw-r--r--, on every system
            with open(os.path.join(OUT, name), "rb") as fh:
                z.writestr(info, fh.read())
    os.replace(part, path)
    return path


# ---------------------------------------------------------------- the check (stdlib)

def number_problems(owner, numbers, where="owners"):
    """What is wrong with an owner's numbers: each a share of the side, the box a rectangle and the tile a square
    inside it, and the loop a length in seconds (check() holds it to the film); an extra's (`where` "extras") are the
    same but the loop, as it has no film."""
    def is_share(v):
        return isinstance(v, (int, float)) and not isinstance(v, bool) and 0 <= v <= 1

    out = []
    at = f"{TABLE}: {where}.{owner}"
    box, fade, tile = numbers.get("box"), numbers.get("fade"), numbers.get("tile")
    if not (isinstance(box, list) and len(box) == 4 and all(is_share(v) for v in box) and box[0] < box[2]
            and box[1] < box[3]):
        out.append(f"{at}.box is {box!r}, not [left, top, right, bottom] inside the still")
    if not (isinstance(fade, list) and len(fade) == 3 and all(v is None or is_share(v) for v in fade)):
        out.append(f"{at}.fade is {fade!r}, not [left, right, bottom] (shares or null)")
    if not (isinstance(tile, list) and len(tile) == 3 and all(is_share(v) for v in tile) and tile[2] > 0
            and tile[0] + tile[2] <= 1 and tile[1] + tile[2] <= 1):
        out.append(f"{at}.tile is {tile!r}, not a square [x, y, side] inside the still")
    allowed = NUMBERS if where == "owners" else NUMBERS - {"loop"}
    loop = numbers.get("loop")
    if where == "owners" and not (isinstance(loop, (int, float)) and not isinstance(loop, bool) and loop > 0):
        out.append(f"{at}.loop is {loop!r}, not the film's length in seconds")
    if set(numbers) - allowed:
        out.append(f"{at} holds {sorted(set(numbers) - allowed)}")
    return out


def check():
    """The problems of contract/figures against the library, the pack and its own table; an empty list when sound."""
    path = os.path.join(OUT, TABLE)
    if not os.path.exists(path):
        return [f"{os.path.relpath(path, ROOT)} is missing: run python3 tools/figures/figures.py publish"]
    table, problems = load(path), []
    if table.get("version") != VERSION:
        problems.append(f"{TABLE}: version {table.get('version')}, publish.py writes {VERSION}")
    if not re.fullmatch(r"#[0-9A-Fa-f]{6}", str(table.get("card"))):
        problems.append(f"{TABLE}: card is {table.get('card')!r}, not #RRGGBB")
    if table.get("crimson") != crimson():
        problems.append(f"{TABLE}: crimson is {table.get('crimson')!r}, tokens.json figureCrimson {crimson()} (the "
                        f"stills are drawn in it: republish)")
    if table.get("film") != {"side": SIDE, "tint": TINT}:
        problems.append(f"{TABLE}: film is {table.get('film')!r}, publish.py's layout {{'side': {SIDE}, 'tint': "
                        f"{TINT}}}")
    moves, owners, files = table.get("moves", {}), table.get("owners", {}), table.get("files", {})
    lib = library()
    problems += [f"{TABLE}: no entry for the library move {m}" for m in sorted(lib - set(moves))]
    problems += [f"{TABLE}: {m} is not a library move" for m in sorted(set(moves) - lib)]
    pack = load(PACK)
    want = entries(pack)
    for move in sorted(set(moves) & set(want)):
        if moves[move] != want[move]:
            problems.append(f"{TABLE}: {move} is {moves[move]}, the pack gives {want[move]} (republish)")
    seen = {e["owner"] for e in shown(moves)}
    problems += [f"{TABLE}: no numbers for the owner {o}" for o in sorted(seen - set(owners))]
    problems += [f"{TABLE}: numbers for {o}, which no move shows" for o in sorted(set(owners) - seen)]
    for owner in sorted(seen & set(owners)):
        problems += number_problems(owner, owners[owner])
    named_extras, table_extras = set(extras()), table.get("extras", {})
    problems += [f"{TABLE}: no entry for the extra {e} (republish)" for e in sorted(named_extras - set(table_extras))]
    problems += [f"{TABLE}: extras.{e} is no extra (a motion file in contract/art with \"extra\": true)"
                 for e in sorted(set(table_extras) - named_extras)]
    problems += [f"{TABLE}: the extra {e} has the name of a library move or an owner (its still would be theirs)"
                 for e in sorted(set(table_extras) & (lib | seen))]
    for e in sorted(set(table_extras) & named_extras):
        problems += number_problems(e, table_extras[e], "extras")
        still, tint = (os.path.join(OUT, n) for n in extra_files(e))
        sizes = [png_size(p) if os.path.isfile(p) else None for p in (still, tint)]
        if all(sizes) and sizes != [(EXTRA_SIDE, EXTRA_SIDE)] * 2:
            problems.append(f"{STILLS}/{e}.png and its tint are {sizes[0]} and {sizes[1]}, not both {EXTRA_SIDE} "
                            f"square (make figures MOVES={e})")
    named = {f"{folder}/{o}{ext}" for o in seen
             for folder, ext in ((LOOPS, ".mp4"), (FILMS, ".mp4"), (STILLS, ".png"), (STILLS, TINTED))} \
        | pair_names(moves) | {n for e in table_extras for n in extra_files(e)}
    problems += [f"{TABLE}: no sha256 for {n}" for n in sorted(named - set(files))]
    problems += [f"{TABLE}: names {n}, which no move or extra shows" for n in sorted(set(files) - named)]
    for name in sorted(pair_names(moves)):              # a timing variant's pair is its reference's, byte for byte
        owner, rest = pair_owner(name)
        twin = f"{PAIRS}/{drawn_from(pack, owner)}.{rest}" if owner in pack["moves"] else name
        if twin != name and twin in files and files.get(name) != files[twin]:
            problems.append(f"{name}: not {twin}, the pair of the move whose cells it plays (republish)")
    lengths = {}                                     # (owner, folder): the film's frames and seconds (PLACES)
    for name in sorted(files):
        p = os.path.join(OUT, name)
        if not os.path.isfile(p):
            problems.append(f"{name}: missing")
            continue
        if sha256(p) != files[name]:
            problems.append(f"{name}: its sha256 differs from {TABLE} (republish)")
        with open(p, "rb") as fh:
            head = fh.read(12)
        png = name.startswith((STILLS + "/", PAIRS + "/"))
        if (png and not head.startswith(PNG_SIGNATURE)) or (not png and head[4:8] != MP4_BOX):
            problems.append(f"{name}: not a {'PNG' if png else 'MP4'} file")
            continue
        if png:
            continue
        folder, owner = name.split("/", 1)
        track = film_track(p) or (None,) * 5
        if track[:2] != (SIDE, TINT + SIDE):
            problems.append(f"{name}: its track is {track[:2]}, not the {SIDE} x {TINT + SIDE} film (base, band, tint)")
        if track[2] != CODECS.get(folder):
            problems.append(f"{name}: its codec is {track[2]!r}, {folder}/ holds {CODECS.get(folder)!r}")
        lengths[(owner[:-len(".mp4")], folder)] = (track[3], track[4] and round(track[4], PLACES))
    for owner, folder in sorted(lengths):
        if folder != LOOPS:
            continue
        (frames, seconds), other = lengths[(owner, LOOPS)], lengths.get((owner, FILMS))
        if other and other != (frames, seconds):
            problems.append(f"{FILMS}/{owner}.mp4 has {other[0]} frames in {other[1]} s, {LOOPS}/{owner}.mp4 {frames} "
                            f"in {seconds} s: both encode the same frames (make figures MOVES=...)")
        loop = owners.get(owner, {}).get("loop")
        if seconds is None:
            problems.append(f"{LOOPS}/{owner}.mp4: its track declares no length (mdhd, stts)")
        elif loop is not None and loop != seconds:                      # a missing loop: number_problems
            problems.append(f"{TABLE}: owners.{owner}.loop is {loop!r}, its film runs {seconds} s (republish)")
    if table.get("films_digest") != films_digest(files):
        problems.append(f"{TABLE}: films_digest is {table.get('films_digest')!r}, the films' is {films_digest(files)} "
                        f"(republish)")
    for base, dirs, names in os.walk(OUT):
        # the watches' pack holds its own files, which watch_problems checks
        dirs[:] = [d for d in dirs if not d.startswith(".") and not (base == OUT and d == WATCH)]
        for n in names:
            rel = os.path.relpath(os.path.join(base, n), OUT)
            if not n.startswith(".") and rel != TABLE and rel not in files:
                problems.append(f"{rel}: a stray file in contract/figures (publish prunes it)")
    return problems + watch_problems() + player_problems()


def main():
    args = sys.argv[1:]
    if args == ["--player"]:
        players()
        print("site/src/scripts/player.js and helper/dist/lib/player.js are tools/figures/player.js")
    elif args[:1] == ["--zip"] and len(args) == 2:
        print(write_zip(args[1]))
    elif args == ["--watch"]:
        write_watch()
    elif args == ["--check"]:
        found = check()
        for line in found:
            print(f"contract/figures: {line}", file=sys.stderr)
        if found:
            sys.exit(1)
        print("contract/figures: every library move has its two films, still, tint still and numbers (its loop the "
              "film's length), each fault its pair with its tints, each extra its still, tint and numbers, the table "
              "matches the files, the watches' pack is the game pack's cut within its budget, and the site's and the "
              "desktop app's player.js are tools/figures/player.js")
    else:
        sys.exit(__doc__)


if __name__ == "__main__":
    main()
