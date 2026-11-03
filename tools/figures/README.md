# tools/figures: the move figure build

The motion files `contract/art/<move>.json` in; the load-on-demand pack `QuestPump_FigArmor/`, the key-pose icons
(`Media/MoveIcons.png` with `FigureIcons.lua`), `contract/fixtures/figure-steps.json`, the phones' and the desktop app's
films and the key-pose stills in `contract/figures/` (which the site's pages also import), the contact sheets and the review page out (`docs/plan.md` slice 10; the design in slice
2, "The move figure"). Built 2026-09-28 from the scratchpad prototype (`v3/armor`, `v4/pack`, `v4/tint`, `v4/props`).

## Commands

| Command | What it does | Needs |
|---|---|---|
| `make figures-check` | bakes every motion file into its 30 Hz track and runs the checks, a fault's among them | Python 3 only |
| `python3 tools/figures/figures.py preview <moves>` | a quick contact sheet per move in the real armour (9 frames, small, few samples), the fault still last | Blender |
| `python3 tools/figures/figures.py pose <move> <pose>` | one pose's joints and reach shortfalls, for writing a motion, and the target the move's fault breaks measured at that pose | Python 3 |
| `python3 tools/figures/figures.py clip [moves]` (`--json[=<file>]`) | plate-on-plate clipping in the real armour, measured ("Clipping" below): every frame and the fault still posed as the renderer poses them, each part against every part it does not meet at a joint; named moves print each pair from 0.5 cm deep (its deepest frame and phase, the part that reaches in, its frames from 0.5 cm) and a line a move with its worst, no moves that line for the whole library, worst first; `--json` writes the whole result (default `build/clip.json`) to compare a fix round with. `make check` does not run it | Blender |
| `python3 tools/figures/figures.py fmt` | rewrites the motion files in the house layout | Python 3 |
| `make figures` (`MOVES=a,b`, `ARGS=--prune`) | the whole build (renders, pack, icons, films, review page); a move that fails to load, fails its checks or fails in its build keeps its pack entry as it was, and the run writes everything else, then lists those moves last and exits nonzero | setup below |
| `tools/figures/.venv/bin/python tools/figures/figures.py films [moves]` | the films (both encodings) and the phones' fault pairs alone, then `publish` (slice 29), only when the films' form alone changed: renders what the cache lacks, never writes the pack, the icons, the fixture, the contact sheets or the review page, and names a move whose frames would change its pack entry instead of filming it (`make figures` rebuilds both); a timing variant is filmed with its reference, its own run table over the reference's frames, and naming a variant builds its reference (`films tempo_squat` films `air_squat` and `tempo_squat`); a style, Blender script, ground or fault-pose change gives new pixels under the same entry, which it cannot see, so it needs `make figures` | setup below |
| `python3 tools/figures/figures.py approve <moves>` | records David's approval in `contract/art/approvals.json` | Python 3 |
| `tools/figures/.venv/bin/python tools/figures/figures.py publish` | the apps' figures in `contract/figures/`: `loops/<owner>.mp4` and `films/<owner>.mp4` (this build's films, HEVC and H.264; after any build, a full one too, only the moves it built whole, since the build folder is a cache other runs share: a move not rebuilt, or whose film failed, keeps its committed files, 2026-10-05; a render's ground pass counts as done only with each frame's floor and, where the props show, its prop layer whole), `stills/<owner>.png` (each film's key-pose frame in crimson, cut from the film in its own square, the card keyed out) with its tint `stills/<owner>.t.png`, the right-versus-wrong pair of each owner with a fault (`pairs/<owner>.key.png` and `.fault.png`, slice 22, with their tints `.key.t.png` and `.fault.t.png`, slice 29, as the build wrote them; a timing variant's are its reference's files under its own name), each extra's still `stills/<extra>.png` with its tint as its build wrote them ("Writing an extra") and `figures.json` (version 6: each move's owner, the move itself, key-pose second, mirror and fault caption, and the same of each of its gear variants under `gear` (owner `<move>.<gear>`); each owner's box (its figure and props over the whole loop, the bottom where it stands), the edges that cut its floor light, its tile's square and its `loop`, the film's length in seconds; each extra's box, fade and tile under `extras`; the crimson and the films' layout; each file's sha256; `films_digest`, the H.264 set's name); the rest is pruned, player.js is copied again to the site and the desktop app, and last the watches' pack is brought up to the pack (`--watch` below); `make figures` and `films` run it | setup below |
| `tools/figures/.venv/bin/python tools/figures/publish.py --watch` | the watches' own pack alone, `contract/figures/watch/` ("The watches' pack" below): `pack.json` and `s/<owner>.<hash>.png` with its tint `s/<owner>.t.<hash>.png`, cut from `QuestPump_FigArmor/`; an owner whose table and game sheets are unchanged and whose watch sheets exist keeps them, and a file no table names is deleted | setup below |
| `python3 tools/figures/publish.py --player` | `player.js`'s two copies (2026-10-09): the site's `site/src/scripts/player.js`, a module its pages import to composite the films (`addTint`), and the desktop app's `helper/dist/lib/player.js`; `publish` writes them too. The site's pages import the films and stills straight from `contract/figures/` at build time, so the site's hero of pack sheets (`site/public/figure.js` and `site/public/figure/`, `--site <move>`) went that day | Python 3 |
| `python3 tools/figures/publish.py --zip <folder>` | `<folder>/figures-<films_digest>.zip`, the desktop app's download of the H.264 films (each under its table name, stored, in name order, dated 1980-01-01, so a set gives the same bytes), refused while `--check` fails; `helper-release.yml` uploads it to the `helper-updater` release once per set | Python 3 |
| `tools/figures/.venv/bin/python tools/figures/figures.py prune` (`--dry-run` lists) | deletes the renders this tree no longer uses: every folder in `build/render/<move>/` but the move's current loop and fault-still folders, the folder of a move no motion file names, and in `build/loops/<move>/` what no build reads (the frames of a deleted folder, the crimson loops' cells and what a stopped run left; a move whose file does not bake keeps its folders); a build prunes the moves it built, and filming a move drops its frames of an older layout | setup below |

`make check` and CI run `figures.py check` and `publish.py --check` (`make figures-check`): every motion file bakes and
passes every check (`test_motion.py`, which also held each approval to its motion file's hash, went on 2026-10-03 with
the tests), and every library move has its two films, still and tint still in `contract/figures/` (each fault its pair
with both tints), every extra its still and tint, both 960 px square, and its numbers, every film the stacked size in its folder's codec (`loops/` hvc1, `films/` avc1) with as many frames
and seconds as its owner's other film, each owner's `loop` its film's length, each timing variant's pair its
reference's, the table's crimson tokens.json's `figureCrimson`, `films_digest` the films', the table matching the pack
and the files, with no stray file, the watches' pack what the game's pack gives within its budget, and the site's
`site/src/scripts/player.js` and the desktop app's `helper/dist/lib/player.js` byte for byte `player.js`. Both phone
apps bundle that folder without `films/` and `watch/` (design v3 §11, 2026-10-04: no download): the iPhone app as
folder references copied to `figures/` (`apple/project.yml`), the Android phone app as its assets (`figureAssets`, a
`FolderAssets` task that skips them), and each watch app carries `watch/`'s `pack.json` and `s/` at the bundle's top,
where it carried the game pack's until 2026-10-21. The desktop app takes `films/`, `stills/` and `pairs/` (its test
builds bundle them; from the public release, `docs/tasks.md`, it downloads the films).

## The watches' pack (2026-10-21)

App Store Connect refused iPhone build 17: its watchOS variant was 104 MB after app thinning, over the 75 MB a watch app
may take (ITMS-90389), 94.4 MB of it the game pack's 1,088 sheets. A watch draws the figure 44 pt high on the countdown
(88 px at 2x) and 96 pt on the legend unlock, plays it at most 5 s and then holds the key pose, so `publish.py`
(`write_watch`, `watch_tables`) cuts its own pack from the game's, deterministically:

- Cells: per owner the cells its own table and its variants' show in a watch's play, the runs that start before 5 s
  (`WATCH_PLAY_S`; every run of a shorter loop), each such run's cross-fade partner and the key cells, renumbered in
  their order (15,016 of the pack's 19,175). A longer loop is cut to those runs, its `ticks` their sum (5 s or more,
  so the play and Reduce Motion's one rep run as before) and its `fade` theirs but a cut table's last, whose partner
  the cut dropped.
- Size: each owner's cell scaled to at most 128 px high (`WATCH_HEIGHT`), Lanczos on RGB from the game's decoded
  cells, the base on the card and the tint on black, in a tight grid whose columns give the shortest longest side (no
  power of two, which would only add decoded memory; at most 2048 px a side).
- Encode: the base the smallest palette of 16, 24, 32, 48 (`WATCH_PALETTES`) and then the game's whose worst cell
  scores SSIMULACRA2 82 (`pack.TARGET`), each cell scored centred on the card in the game's 192 px box at the
  watch's scale (in the 192 px box a small cell scores higher than it looks); the tint the game's 32 colours.
- `pack.json` carries `hz`, `source` (the game pack's `digest`), `height` and per move only what the watches read,
  plus each owner's `from` (the game sheets and cells it is cut from, which `--watch` compares to keep a sheet);
  `publish.py --check` recomputes the tables from the game's pack (stdlib), reads each sheet's size from its IHDR and
  fails past `WATCH_BUDGET`, 55,000,000 bytes (75 MB less about 20 MB for the rest of the thinned watch app).

Measured on 2026-10-21 over all 544 owners (`--watch`, 315 s on David's Mac under load, `nice`): 39.2 MB, 38.9 MB of
sheets and 0.32 MB of `pack.json` (the game's: 94.4 MB and 1.2 MB); palettes 16 (2 owners), 24 (133), 32 (292), 48
(108), 64 (9), none truecolour; the worst cell 82.0; the largest sheet 1260 x 1188 px. With it the Apple Watch app
of an unsigned archive build is 53.3 MB, 48.1 MB thinned to its larger binary slice (against 75 MB), and Wear's
unsigned release APK 44.9 MB.

## The films (slices 29 and 35)

Each owner's `loops/<owner>.mp4` is one stacked film, 480 wide and 992 high (`publish.py` `SIDE`, `GUARD`, `TINT`;
`figures.json` `film`): the base in rows 0-479 (the sash and the crest a very dark indigo, on the card with its floor
and props), a 32-px black band, then the tint in rows 512-991 (the cloth's light on black). It is the pack's class
colour split at the film's size: `build.py` composites each stored frame's five grey cloth levels at 480 px and splits
them with `pack.split`, as the sheets' cells are, so a phone shows `clamp(base + tint x colour / 255)` per channel on
the stored sRGB values, exactly as the game's ADD tint draws the sheets, and one decoder keeps both halves in step. The
tint starts on x265's 64-px coding tree block boundary (512, with `ctu=64` pinned), so no block or its SAO parameters
hold both the card and the tint, and the band ends the block row above it, so HEVC's deblocking filter across row 512
mixes in only black (the band was 16 px, the tint at 496, until 2026-10-06's review).
Both halves ship or neither: a base alone shows a near-black crest. `stills/<owner>.png` stays the film's key frame in
crimson (`figures.json` `crimson`, tokens.json's `figureCrimson`, Figure.lua's CRIMSON), the tiles' still, and
`stills/<owner>.t.png` is that frame's tint half, so a phone recolours the still drawn over the card (S; keyed, the
still alone holds the cloth's light only at its alpha) as `clamp(S - tint x crimson + tint x colour)`; the pairs' tints
work the same way on their opaque stills, as the game's `View:ShowPair` tints its pair. The tint stills are truecolour
PNGs (a few KB each).
The film is libx265 at the loops' crf 28 with `aq-mode=3` (x265's adaptive quantisation biased to dark scenes), its
colour tagged as what it is (`build.py` `FILM_COLOUR`: ffmpeg's BT.601 matrix in the limited range, BT.709 primaries
and transfer, in the VUI and the MP4's `colr` box), since an untagged 480 x 992 film would get BT.709's matrix from
Android's default by frame size (AOSP `ColorUtils`) while `publish.py`'s ffmpeg decodes BT.601.
Measured on 2026-10-06: at plain crf 28 the crimson composite scored as the old crimson loop did against the direct
crimson render (the archer pull-up: SSIMULACRA2 worst 65.9 against 65.4, mean 75.5 against 73.5), seven class colours
scored alike against the exact split (worst 65.6 to 65.9) and the tint half decoded black away from the cloth, but the
encoder left ghosts of the moving cloth in the tint half for a frame, pixels up to 100 levels above black more than
4 px from any cloth (432 such pixels over every frame of 15 sampled films), which a bright class colour would light;
`aq-mode=3` cut them to 86 and lowered both halves' mean error, for 10.7 % more film bytes over the library (44.4 MB
against 40.1 MB). The 32-px band, re-encoded from the same frames the same day: 44.5 MB over the library, and on 16
films (every 20th by size) the tint's pixels more than 8 levels above black and more than 4 px from the exact tint's
cloth fell from 709 to 319 over every frame, the brightest from 59 to 49 levels.
Each owner's `films/<owner>.mp4` is the desktop app's copy (slice 35, 2026-10-08): the same frames through the same
listing in H.264 (`build.py` `ENCODINGS`, libx264 High at crf 24 with x264's own `aq-mode=3`, `avc1`, the index first
in the file), since WebView2 plays HEVC only through an extra Windows extension; its layout and colour tags are the
loop's (row 512 is a multiple of H.264's 16-px macroblock and of the 8-px chroma block, and its deblocking filter also
reaches 3 px across an edge). The first set was encoded from the build cache's film frames without a render (each
owner's HEVC encoded again from them came out byte for byte the committed loop). Measured on the same 16 films against
their exact frames: the HEVC loops 2.56 MB with 333 ghost pixels (the brightest 52 levels), mean errors 1.84 (base) and
0.037 (tint); H.264 at crf 20 4.97 MB, 0 ghosts; crf 22 4.14 MB, 30; crf 24 3.42 MB, 3 (the brightest 13), mean errors
1.59 and 0.033; crf 26 2.80 MB, 36, 1.70 and 0.038; crf 28 2.29 MB, 23, 1.84 and 0.044. Crf 24 keeps every error under
the phones' film for a third more bytes; on six moving films its tint's first four rows sit within 1 level of the exact
frames (the HEVC's within 8) and the band's brightest leak is 7 levels (the HEVC's 13).
Every library move owns its films (2026-10-09, for the rep count that follows the figure): a timing variant's film is
its own run table over its reference move's stored frames (`write_loop` given the reference's build), written whenever
the reference is built, so the apps play `tempo_squat` at its 5.3 s and not `air_squat`'s 2.87 s, as the game does; its
still and numbers are cut from its own film, and its pair is its reference's files under its own name, since the apps
find a pair by owner. The ten variants (nine rep moves and `burpee_chest`) were filmed from the cache's frames without
a render, each reference's films coming out byte for byte the committed ones again: 1.59 MB of loops, 2.06 MB of
desktop films, 0.42 MB of stills and 0.72 MB of pairs (byte copies, one blob each in git). Each owner's `loop` is its
film's length in seconds, the samples' durations summed over the track's timescale (its frames / 30), never the `mdhd`
duration, which the H.264 muxer lengthens by its B-frame delay on 12 of the desktop films. 104 films, all of them of an
odd-ticked run table, hold one frame more than their pack entry's ticks (the concat listing at 30 fps; the cause is not
traced), so an app counts by `loop`, the film it shows, never by the pack's ticks.

A loop that shows more than one rep (a dead bug's two sides, a squat to punch's two punches, a monster walk's two
steps) names the phases whose end finishes one, `rep_ends` in its motion file (2026-10-09). The pack entry then carries
`counts`, the tick at which each rep but the last ends (`build.py` `rep_counts`, a timing variant's from its
reference's `rep_ends`), and `figures.json` the move's `counts` in seconds, so every surface's rep count lands as the
figure finishes a rep, the loop's end the last one's. A file without it is one rep a loop. `figures.py check` holds
each pack entry's `counts` to its motion's `rep_ends`, so a motion edit without `make figures` fails `make check`.

## Setup (on the Mac that renders)

`tools/figures/setup.sh` builds the venv (`requirements.txt`: numpy, pillow, scipy) and, run again after a pin there
moves, brings the venv's packages to the new pins; it checks for Homebrew's
`blender` (5.2 LTS), `magick`, `ssimulacra2`, `oxipng`, `pngquant` and `ffmpeg`. Renders and
intermediates go to `tools/figures/build/` (gitignored): each move renders into a folder named by its track's hash and
the style, so a changed motion or style renders fresh. A build then deletes the older folders of the moves it built,
and `figures.py prune` deletes every render folder the tree no longer uses (David, 2026-10-06: "clean up after
yourself"; the cache had grown to 361 GB, 223 GB of it renders no tree used); a folder touched in the last hour stays,
since another session's build may be writing it.

EEVEE renders on the GPU: with the game open a render job manages about one frame every 7 s (2026-09-28), so a full
library build takes several hours; with the game closed it is much faster. Blender jobs run under `nice` with a 30-min
timeout each, so a stalled GPU fails the build instead of hanging it. While a WoW client runs, a job waits until fewer
than `FIGURE_GAME_JOBS` renders (default 1, 0 for none; any session's) run on the Mac, because a render's GPU time comes
out of the game's frame rate and `nice` lowers only the CPU (2026-10-05).

## The pieces

| File | Role |
|---|---|
| `rig.py` | the skeleton (the armour's bone lengths), vectors, frames, two-bone IK, FK limbs, the armour's clearances |
| `motion.py` | the motion file format (its docstring is the reference), poses and helpers (`line` for planks and leaning bodies, `fit`, `orbit` for a hand or foot on a circle), blending (field by field, a limb that changes mode by its angles about its own hinge, planks by the helper's own numbers), the sash (a chain that lands on the floor and the props and keeps off the near leg), the glow, the camera fit, the props' box the crop grows toward (`prop_box`: the points the camera keeps) and the track, whose frames name the arms that hold an item and the ones resting on a surface (`held_arms` by `items.holders`, `planted_arms` by the contacts) |
| `props.py` | the props as solid boxes and rods: wall, chair, desk, table, doorframe, door, bar (hung from above, or an outdoor bar on two posts from the ground with free space above it; its posts at `half` apart from the middle for a wide grip, 60 cm by default), pole (a steel post fixed in the ground, for the human flag), rings, bench, step, stairs (an endless flight scrolls under a climber), bike, walking pad, foam roller, mat, runner; one definition for the checks and the Blender prop pass |
| `items.py` | the held implements placed from the joints: barbell, kettlebell (its handle in the closed fingers and the bell clear of them, or resting on the hips), backpack (in hand, worn on the backplate, hugged to the breastplate or on the hips), a skipping rope that trails the hands, ab wheel, towel, dumbbells (in hand, or one across the hips), a medicine ball (free to fly for a slam or a wall pass), weighted vest, bands, landmine bar, bike cranks, rings on straps, a bath towel knotted and shut in a door or hung over a bar (its tails bowed, their ends wrapped in the fists), a light chair held by its seat (slice 21), a standard whose cloth takes the class colour (slice 45, the standard-bearer); `holders` names the hands that hold an item at a moment (the renderer leaves their arms as posed); `shape` is each placed item's solid for the checks (spheres, boxes, lines with their radii and held points), from the same pose and measures `blender/scene.py` draws |
| `checks.py` | reach, solids (joints, soles, the sash's cloth past its knot and, while the head is no contact, the helm by `helm.json`'s points, against the floor and the props), items (`items.shape`'s spheres and boxes, a vest's plates among them, in the armour or the helm by `helm.json`'s points, a rope's, a band's or a door towel's line through the armour or the helm by `helm.json`'s distance grid past what holds it, a skipping rope's pass under the feet while they are in the air), bend, range (an ankle, a foot's turn against its knee, a wrist), pop (a joint's jolt in one tick), balance, tempo (by the move's dose in `contract/moves.json`), targets, the fault still, variants, show (every prop's faces clear the card by 1.6:1 at their darkest, a part narrower than 10 cm across by 2:1, since the phones draw the figure lighten over a page of nearly the card's colour; the held implements' palettes in `blender/scene.py` keep even their shadow bands about 2:1) |
| `figures.py` | the command line: check, preview, pose, clip, helm, fmt, approve, build, films, publish, prune; and `layers`, the one order the render layers are laid in ("Layers" below) |
| `helm.py`, `helm.json` | the helm's points for the checks, measured on the real armour by `figures.py helm` (its docstring is the reference): the helm's farthest vertex in 80 directions and a 3-cm grid over its middle plane, past the head's own ball, and the distance to its surface (negative inside) every 1 cm round it; `helm.json` carries the SHA-256 of `blender/armor_model.py` it was measured from, and `figures.py check` stops on any other until `figures.py helm` runs again; `helm.py` sits beside `blender/` for `clip.py`'s reason |
| `clip.py` | the clipping measure Blender runs for `figures.py clip` (its docstring is the reference): each part's solid, the pairs it skips, a point's depth; it sits beside `blender/`, never in it, since `build.render_key` hashes every script there into each render folder's name and a new one would render the whole library again |
| `build.py` | the build: dense crimson render, frame selection, grey-cloth and ground passes, the fault still, the tint split, the sheets, variants, the manifest, icons, the films (the split at their size, HEVC and H.264) and the phones' fault pairs with their tints, contact sheets, the review page, the fixture; an extra's still (`build_extra`) |
| `pack.py` | the crop (the figure first, its fault still held: a prop fills only room the cell has), compositing, the pose distance, selection and the run table, sheet packing, the base sheet as the smallest palette PNG scoring SSIMULACRA2 >= 82 (the client shows an addon's JPEG as a green box), the tint split, the manifest's Lua |
| `steps.py` | the Python twin of `Figure.lua`'s core and the writer of `contract/fixtures/figure-steps.json` |
| `player.js` | the JavaScript twin of `Figure.lua`'s core (its steps equal `figure-steps.json`'s, checked once on 2026-10-08) and of its four-layer draw on a canvas, one ES module for the review page (written into it), the site (`site/src/scripts/player.js`, whose `addTint` its pages composite the films with; on import it also plays any `canvas[data-figure]` of pack sheets, with its pause button and the key pose under reduced motion) and the desktop app (`helper/dist/lib/player.js`, whose `addTint` and `recolour` its `lib/figure.js` draws the films and the stills in a colour with) |
| `review.py` | `build/review/index.html` (every move playing from its sheets through `player.js`, in a chosen class colour, with its fault pair where it has one) and the approvals |
| `publish.py` | the figures outside the game: owns the films' layout, the extras' still size and the watches' pack, writes and prunes `contract/figures/` (both films, the key-pose stills and their tints, the fault pairs with theirs, the extras' stills with theirs, and `watch/`, the watches' pack cut from the game's), `--player` (`player.js`'s copies), `--watch` and `--zip`, and `--check` (stdlib) holds them to the library, the extras' motion files, the pack, tokens.json's crimson, `player.js`, the watches' budget and its own tables |
| `blender/armor_model.py`, `blender/plate_kit.py` | the armour: the prototype's plates, posed by `armor_model.place` (each limb's frame by its view, a front view's turning with the body as it turns off the view axis; the pauldrons' schedule, held still near overhead; `arm_turns`, which turns a free arm clear of its tasset, a planted one only while its hand stays within 2 cm of its surface in the picture, and leaves a holding one as posed, each turn worked out over the whole loop so it eases in and out, 1.5° a tick at most; the sash); their docstrings hold the rules |
| `blender/scene.py`, `blender/render.py` | the floor pool, the props, the held implements; the figure pass and the ground pass (the floor, and the props with the figure held out) |
| `style.json` | the card, the floor pool, the prop materials and their lights |

## Layers

Each frame is three renders that register pixel for pixel (`blender/render.py`): the figure pass (the armour, its held
implements and its cloth, with the glow outside its silhouette as a layer of its own), and the ground pass's floor (the
pool with every shadow, the figure and the props hidden from the camera) and, where the move has a prop, its prop layer:
the props with the figure as a holdout (Blender's `is_holdout`), so the layer holds only what of the props the figure
leaves in sight. `figures.layers` lays them card, floor, figure, props, glow, for every image the build makes (the
cells and so the sheets, the films and the stills and pairs cut from them, the icons, an extra's still) and for the
preview: a prop nearer the camera than the figure hides it (a bar in front of the hips, a hand gripping a bar from
behind, the near post of a dip bar, a wall's near end), and where the figure stands in front the prop layer is empty.
Where the two share an edge pixel the figure keeps min(1, its alpha / (1 - the prop's)) under the prop, so each covers
its own part of the pixel (a disjoint over): on `straight_bar_dip`'s layers that came within 0.002 levels of the
composite in floating point, where laying the prop plainly over the figure was off by up to 2.5 levels at those edges
and the old order, the prop under the figure, drew the bar behind the body (2026-10-22). It costs no render: the prop
layer was rendered per stored frame already, for the figure's shadow on it. The figure's shadows on the props and the
floor are the same renders as before, and a move without a prop gives the same cells.

## Clipping

The checks see the skeleton, never the plates, so how far the armour's parts pass into each other is measured on the
armour itself (2026-10-14, after David: "generally with this one and others there is an aweful lot of clipping. Can we
improve?"). `figures.py clip` bakes the moves, then `clip.py` builds the suit once in one headless Blender process (it
renders nothing, so it takes no turn at the game's render gate), poses every frame and the fault still as
`blender/render.py` does (`armor_model.place`: `arm_clear`, the pauldrons' schedule, the fauld's ride and the tassets'
slide) and measures every part against every part it does not meet at a joint. A part's solid is what its plates
enclose: where a slice across its long axis has a closed ring of plate (the helm, the cuirass, the belt, the arm and leg
tubes, the gauntlet, the sabaton, a pauldron's dome), everything inside the ring; elsewhere (a cop and its fan, a
tasset, the fauld, the culet, the open front of the belly lames, a trimmed tube end) the plate alone. A vertex of the
other part's raw mesh (never the outline shell) sits as deep as its shortest way out of that solid, and a pair's depth
is its deepest vertex either way, with its frame, its phase and the part that reaches in. Two open plates crossing read
at most half a plate (5 to 11 mm), and a part hidden inside another reads as deep as one poking through it. `clip.py`'s
docstring holds the method and its accuracy: on eight moves the search came within 1.2 mm of measuring each pair's 600
deepest candidate vertices on the mesh.

The pairs it skips meet at a joint and touch there by design (`clip.ADJACENT`, by the names it prints): `head` with
`chest`; the torso's plates among themselves (`chest`, `abdomen`, `fauld`, `culet`), which telescope as the waist bends
and the hips flex; and on each side `chest` with `pauldron` and `upper_arm`, `pauldron` with `upper_arm`; `upper_arm`
with `elbow` (the couter) and `forearm`, `elbow` with `forearm`, `forearm` with `hand`; `abdomen`, `fauld` and `culet`
each with `thigh` and `tasset`, `thigh` with `tasset`; `thigh` with `knee` (the poleyn) and `shin`, `knee` with `shin`;
`shin` with `foot`. That leaves 257 of the 300 pairs; at `air_squat`'s first frame, standing, only skipped pairs overlap
(`abdomen` 4.4 cm in `chest`, `fauld` 3.2 cm), and at `jumping_jacks`' the two `thigh`s 0.3 cm besides.

Calibrated on 2026-10-14 against the moves the visual survey rated, and the child's pose before its rework:

| Move | Worst | Where |
|---|---|---|
| `childs_pose` (reworked) | 3.2 cm | `knee` in `hand` as the body folds forward (52 of 450 frames); 1.1 cm, `thigh` in `chest`, through the hold |
| `childs_pose` before its rework (`c47b5723`) | 8.8 cm | `chest` in `thigh` in all 150 frames; `knee` 7.7 cm in `head` |
| `bear_plank`, `bear_crawl` | 0.7, 0.8 cm | the two `thigh`s against each other |
| `forward_fold` | 6.2 cm | `knee` in `chest`, head down |
| `bound_squat` | 9.9 cm | `chest` in `thigh`, heels down |
| `crow`, `frog_stand` | 8.1 cm each | `elbow` in `shin` |
| `forehead_to_knees` | 15.6 cm | `knee` in `chest`, head down |

The library (395 moves, the timing variants playing their references' frames: 61,733 frames and the fault stills) took
705 s in one process on 2026-10-14 with the game running, about 11 ms a frame after 18 s building the suit; 41 moves
reached 10 cm and 117 reached 5 cm, `chest` and `thigh` the commonest worst pair (a trunk folded onto the thighs), then
a `knee`, an `elbow` or a `forearm` in `chest`. For a fix round, measure the moves with `--json=<before>`, change their
motions, measure again with `--json=<after>` and compare the pairs' `depth`s. `make check` does not gate on it.

## Writing a motion

Start from a similar move, bake with `make figures-check`, look with `preview`, and read the numbers with `pose`. A
problem line names the fault, the worst value and the frames (`reach: arm_far falls 6.8 cm short of its contact
(frames 0-12)`). Timing variants (`variant_of` with `phases` or `pause`) cost no art: the build retimes the reference
move's run table. A hold or a pose shows the way into its position (David, 2026-10-26: "Generally it should always
show how to get into that position"): its loop starts where the move's `how` (contract/moves.json) has the player set
up (seated between the books, the back on the wall, on hands and knees, under the bar), reaches the key pose within
about 3 s (a watch plays the first 5 s, then holds the key pose), breathes there and goes back to the start, as
`lsit_tuck`, `tuck_planche` and `handstand` do; `checks.py` `entry` holds a hold's first frame `ENTRY_CM` (12 cm) from
its key pose and a pose's loop to that much motion. Start from the nearest setup the `how` names, never a stand that
adds nothing: the crop covers the whole loop, so a far start makes the held position smaller. Holds set
`"breathe": true`, so the pack cross-fades their runs. A move fast all
through its loop sets `"fast_phase"` below 1, the pack's fast-phase allowance (`pack.REL`, 1: a stored frame stands in
for the ticks up to one tick of motion away): at 1 butt kicks kept 4 frames for its 18-tick loop, held up to 5 ticks,
and froze and jumped; at 0.3 its 22-tick loop keeps 17, none held over 2 ticks (2026-10-10). Read the move's `seq` in
`QuestPump_FigArmor/pack.json` after its build: a run longer than 2 ticks in a fast loop is a hold the player sees,
unless the motion itself stands still there (a `hold` step). Take the largest value that ends those holds: jog 0.4 (its
one run of 3 a dwell inside the tolerance, 2026-10-23), high knees 0.6, sprint 0.45, fast feet 0.4, seal jacks 0.8 (its
3 is the arms-open hold), cross knee drives 0.3, pogo hops 0.9, shadow skip 0.8 (its 3 moves about 1.5 px a tick)
(2026-10-10); wall climbers 0.4 (at 1 it kept 5 of its 31 frames, held one for up to 6 ticks while the pose moved 11 px,
then jumped, and showed the near leg's frames on the far leg's drive; at 0.4 it keeps 15, its two runs of 3 the knee's
own stop at the wall, inside 1.5 px), mountain climbers 0.4, jumping jacks 0.6, skater hops 0.55, squat thrusts 0.55,
sprawl 0.65, shadow boxing 0.5, quick steps 0.6 (2026-10-23), punch and duck 0.45, step jacks 0.6, stairs all out 0.85,
backpack squat drives 0.25, which its vest figure takes with the rest of its motion (2026-10-15; David: "The animation
for wall climbers is broken. stutters"). No `fast_phase` shortens a dwell inside the 1.5 px pose tolerance, where every
part slows at once: an `out` ease into a pose met by an `in` ease out of it, or a short `hold` between them. Change the
motion there: pogo hops lost its ground hold and bounces on `linear` contact steps, and shadow skip's rope arms, which
stopped at each eased pose, turn on `linear` steps, its flight's arc drawn by a pose each way (`rise`, `fall`). A limb
that reaches a pose at full speed and stops there snaps: cross knee drives' arm came back to its guard on an `in` ease,
50 cm in the loop's last tick, and its steps are `inout` now.

### Writing a fault

A fault is the wrong picture of the New-move view's still right-versus-wrong pair (slice 22): only a form break the
side camera shows (the hips sagging or piked, a rounded back, heels lifting, the head dropping, bent elbows in a
straight-arm hold); a knee caving in needs a front-view move. Add a pose to `poses` (usually `"from"` the key pose,
changing only what breaks the form; it need not be on the timeline) and a `fault` block naming it:

    "fault": {"pose": "sag", "caption": "hips_sag",
              "breaks": {"check": "angle", "joints": ["ankle_near", "pelvis", "neck"], "min": 172}}

`caption` is the key of its line in `Locales/enUS.lua` (`figure.fault.<caption>`); `breaks` is a form target without
`at`, in the grammar of `targets` (`height`, `angle`, or `offset`: a joint's signed distance from another along `f`,
`u` or `l`), usually the target the key pose already holds. `make figures-check` holds the fault to it: the fault pose
passes reach, solids, limbs, touch, items, bend and range in the key frame's place (never balance or tempo), the target holds
at the key pose and fails at the fault pose, and the camera fitted with the fault pose added is the loop's (a joint
of the fault pose past where every frame of the loop keeps it changes the camera, and fails).
`pose <move> <fault pose>` prints the measured target while you tune it; `preview` shows the still beside the loop's
frames. The build renders the still once, in a folder of its own beside the loop's (a changed fault renders one
frame, never the loop), as the sheet's last cell (the entry's `fault` and `fault_cap`; one stored frame fewer where
the cell would grow the sheet, so only a hold of one frame grows), and writes the phones' pair to
`build/stills/`; the contact sheet ends with the pair. A still whose armour passes the loop's crop (its alpha box plus
the 12 px margin; the joint-level check above cannot see armour, so a pose can pass it and still overshoot) is left
out: the move is packed without a fault and named at the end of the run with the overshoot in render px; keep contact
limbs no lower and the trunk no further back than the loop has them. A timing variant shares its reference move's
fault.

### Writing a gear variant

A move whose load a real implement can carry (Library.lua `holds`: the bag row, the bag deadlifts, the carries, the
squat drives, the hip thrust; 2026-10-09) shows the implement the player has: the game plays `<move>.<gear>` from the
pack where `ns.Implement` names a gear the pack has a variant for, the apps the move's `gear` entry in
`contract/figures/figures.json` for the core's implement, and every other player the move's own figure. A gear
variant is `contract/art/<move>.<gear>.json`:

    {"move": "backpack_row.dumbbells", "gear_of": "backpack_row", "gear": "dumbbells",
     "items": [{"kind": "dumbbell", "hand": "near"}]}

`gear` is an equipment key (ns.Gear; `make figures-check` holds it to `contract/sheet.json` `gear_order`) and `items`
replace the move's; where the implement sits in other hands, `poses` replaces the arm fields (`arms`, `arm_near`,
`arm_far`) of the move's poses of those names (two dumbbells held apart where a bag hung from both hands together, the
hands on a dumbbell's heads where they held a bag's top). Everything else is the move's: the timeline, the key, the
fault, the targets, the props, the camera's angles. The gear variant of a timing variant names only `gear_of` and
`gear`: it is that timing variant's table on its reference's gear variant of the same gear. It bakes, passes every
check and renders as a move of its own (its own crop, frames, sheets, film, stills and pair; `make figures
MOVES=<move>.<gear>`), while the key-pose icon stays the move's. Since the checks see joints, never contact, measure
each grip against its implement (`items.grip` against a dumbbell's heads, `items.hook` against a kettlebell's handle or
a bag's strap) before a render.

### Writing an extra

An extra is a figure that is no library move (slice 45, 2026-10-16): the streak card's standard-bearer,
`contract/art/streak_banner.json`, which the phones' and the desktop app's share cards draw in the player's colour. Its
motion file says `"extra": true`, its name is lower_snake and no key of `contract/moves.json`, and it takes no `fault`
or `rep_ends`; its timeline is a pose held (`[{"pose": "stand"}, {"hold": 1.0, "phase": "stand"}]`). It bakes and
passes every check a move does but the dose's tempo. `make figures MOVES=<extra>` renders its key pose alone (crimson,
the five grey levels, its floor and props) at the build's 1280 px, cuts it with a crop of that frame (the figure with
its held items, a few px of floor before the feet) into `publish.EXTRA_SIDE`'s square (960 px, twice the films': the
share cards are 1080 px wide and draw it larger than a move's still), splits it into base and tint as a film's frame
is, and writes `build/stills/<extra>.png` (crimson, the card keyed out, a 256-colour palette PNG, as an owner's still)
with its tint `<extra>.t.png`; `publish` takes both into `contract/figures/stills/` and lists the extra under
`figures.json` `extras` with its still's `box`, `fade` and `tile` (an owner's numbers but `loop`: it has no film),
read off the still drawn over the card. It never enters the pack, the key-pose icons, the films, the fixture or the
review page, and a run that names extras alone leaves those as they are (a partial build otherwise rewrites the icon
sheets from the build folder's cache). An app draws it as an owner's still: recoloured with its tint as
`clamp(S - tint x crimson + tint x colour)`.

The standard (`items.py` `banner`) is a wooden pole with a gold finial, standing on the floor at the item's `foot`
through one hand's grip or held in both hands (its butt `low` cm under the lower grip), and a gold crossbar carrying a
cloth flying forward of the pole, its foot cut into two tails, hemmed in gold, with a device of two gold chevrons (the
days a streak stacks) on both faces. The cloth is the armour's own material (`qp_cloth`), which `blender/render.py`
swaps for the grey levels with the sash's, so the tint carries the player's colour on the banner as on the sash and
the crest. The device stands 1.4 cm off the cloth's middle, clear of the cloth's outline hull (inside it, it stood in
the hull's shadow and rendered dark), with an outline of its own. The checks keep the pole and the cloth out of the
armour.

The standard-bearer was picked on 2026-10-16 from three poses rendered side by side at the 192 px box (and at 480 px,
in crimson and two class colours): a guard with the standard planted, the standard raised high in both hands, and the
march with the standard carried in stride. The march shows the streak as what it is, a march of days that goes on (it
fits a milestone and a streak shared as it stands alike), and its stride and carried standard read at 192 px; the
planted pole split a narrow, static figure, and the raised arm crossed the visor.

The body map is the sibling tool `tools/bodymap.py` (`make bodymap`): `contract/art/body.svg` in, `Media/BodyMap.tga`
and `BodyArt.lua` out.
