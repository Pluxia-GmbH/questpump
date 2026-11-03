"""The review page (docs/plan.md slice 2, "Review and approvals"): build/review/index.html plays every move of the pack
from its own sheets at real tempo, with the tint in a chosen class colour, beside its build numbers, its key-pose icon,
its right-versus-wrong pair where it has a fault (the phones' two stills, the key pose then the fault, with the
caption's key) and its approval; ticking moves builds the command that records them in contract/art/approvals.json
(`figures.py approve <moves>`: the move, the sha256 of its motion file, the date). A changed motion file shows as
unapproved again. The page plays them with player.js, the JavaScript twin of Figure.lua's core (steps.py is the Python
one), written into its module script, since a page opened from disk cannot import a module.
"""
import hashlib
import html
import json
import os

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
ART = os.path.join(ROOT, "contract", "art")
APPROVALS = os.path.join(ART, "approvals.json")
OUT = os.path.join(HERE, "build", "review", "index.html")
PLAYER = os.path.join(HERE, "player.js")
CLASSES = {"crimson": [0.7674, 0.3132, 0.3132], "warrior": [0.78, 0.61, 0.43], "paladin": [0.96, 0.55, 0.73],
           "hunter": [0.67, 0.83, 0.45], "rogue": [1.0, 0.96, 0.41], "priest": [1.0, 1.0, 1.0],
           "shaman": [0.0, 0.44, 0.87], "mage": [0.25, 0.78, 0.92], "warlock": [0.53, 0.53, 0.93],
           "druid": [1.0, 0.49, 0.04]}


def motion_hash(move):
    with open(os.path.join(ART, move + ".json"), "rb") as fh:
        return hashlib.sha256(fh.read()).hexdigest()


def approvals():
    if not os.path.exists(APPROVALS):
        return {}
    with open(APPROVALS) as fh:
        return json.load(fh).get("moves", {})


def write(pack, reports, specs):
    os.makedirs(os.path.dirname(OUT), exist_ok=True)
    ok = approvals()
    rows = []
    for move in sorted(pack["moves"]):
        e = pack["moves"][move]
        base = pack["moves"][e.get("ref", move)]
        approved = ok.get(move, {}).get("hash") == motion_hash(move)
        rep = reports.get(e.get("ref", move), {})
        spec = specs.get(move, {})
        facts = [f"{e['ticks'] / pack['hz']:.2f} s loop", f"{base['n']} cells"]
        if rep:
            facts += [f"worst SSIMULACRA2 {rep['ssim2_min']}", f"pose error max {rep['pose_err_max']} px"]
        if "ref" in e:
            facts.append(f"timing variant of {e['ref']}")
        state = ("ok", "approved") if approved else ("todo", "not approved")
        owner = html.escape(e.get("ref", move))
        pair = "".join(f'<img class="still" src="../stills/{owner}.{side}.png" alt="{side}">'
                       for side in ("key", "fault")) + f'<br>fault: {html.escape(e["fault_cap"])}<br>' \
            if "fault" in e else ""
        rows.append(f'''<figure class="move" data-move="{move}">
  <canvas width="192" height="192"></canvas>
  <img class="icon" src="../icons/{owner}.png" alt="">
  <figcaption>{pair}<b>{html.escape(move)}</b> <span class="{state[0]}">{state[1]}</span><br>
  {html.escape(" · ".join(facts))}<br><small>{html.escape(spec.get("note", ""))}</small><br>
  <label><input type="checkbox" value="{move}"{" checked" if approved else ""}> approve</label></figcaption>
</figure>''')
    fig = json.dumps({"hz": pack["hz"], "box": pack["box"], "bg": pack["bg"], "moves": pack["moves"]})
    with open(PLAYER) as fh:
        player = fh.read()
    page = f'''<!doctype html><html lang="en"><head><meta charset="utf-8"><title>Figure Review</title>
<meta name="viewport" content="width=device-width, initial-scale=1">
<style>
:root {{ --bg:#15130f; --fg:#e8dfc8; --muted:#a39a86; --card:#0E0D0B; --ok:#7fbf6a; --todo:#d49a4a; }}
body {{ background:var(--bg); color:var(--fg); font:14px/1.4 system-ui, sans-serif; margin:0; padding:16px; }}
header {{ position:sticky; top:0; background:var(--bg); padding:8px 0; z-index:1; }}
.grid {{ display:grid; grid-template-columns:repeat(auto-fill, minmax(240px, 1fr)); gap:16px; }}
.move {{ margin:0; background:#1d1a15; padding:8px; border-radius:6px; position:relative; }}
.move canvas {{ background:var(--card); width:192px; height:192px; display:block; cursor:pointer; }}
.icon {{ position:absolute; top:8px; right:8px; width:64px; height:64px; background:#2a261f; }}
.still {{ max-width:112px; max-height:112px; margin-right:4px; }}
.ok {{ color:var(--ok); }} .todo {{ color:var(--todo); }}
code {{ display:block; background:#0b0a08; padding:8px; white-space:pre-wrap; word-break:break-all; }}
small {{ color:var(--muted); }}
</style></head><body>
<header><b>QuestPump figure review</b> · pack {pack["pack"]} · {len(pack["moves"])} moves ·
colour <select id="colour">{"".join(f'<option>{c}</option>' for c in CLASSES)}</select> ·
click a figure to pause it
<code id="cmd"></code></header>
<div class="grid">{"".join(rows)}</div>
<script type="module">
{player}
const FIG = {fig};
const PATH = "../../../../QuestPump_FigArmor/";
const CLASSES = {json.dumps(CLASSES)};
const figures = [];
for (const el of document.querySelectorAll(".move")) {{
  const [, base] = resolve(FIG, el.dataset.move);
  const sheet = new Image(), tint = new Image();
  sheet.src = PATH + base.file; tint.src = PATH + base.tint;
  const canvas = el.querySelector("canvas");
  const f = new Figure(canvas, FIG, el.dataset.move, sheet, tint, CLASSES.crimson);
  canvas.onclick = () => {{ f.paused = !f.paused; }};
  figures.push(f);
}}
document.getElementById("colour").onchange = (e) => {{
  for (const f of figures) {{ f.colour = CLASSES[e.target.value]; f.draw(); }}
}};
play(figures);
function cmd() {{
  const on = [...document.querySelectorAll('input[type=checkbox]:checked')].map(i => i.value);
  document.getElementById('cmd').textContent = on.length ?
    'python3 tools/figures/figures.py approve ' + on.join(',') : 'tick the moves you approve';
}}
document.querySelectorAll('input[type=checkbox]').forEach(i => i.onchange = cmd); cmd();
</script></body></html>
'''
    with open(OUT, "w") as fh:
        fh.write(page)
    return OUT


def approve(moves, today):
    """Record each move's motion-file hash and the date in contract/art/approvals.json."""
    data = {"about": "David's approval of each move's motion (docs/plan.md slice 2, 'Review and approvals'): the "
                     "sha256 of contract/art/<move>.json and the date; a changed file needs a new approval.",
            "moves": approvals()}
    for m in moves:
        data["moves"][m] = {"hash": motion_hash(m), "date": today}
    data["moves"] = dict(sorted(data["moves"].items()))
    with open(APPROVALS, "w") as fh:
        json.dump(data, fh, indent=1)
        fh.write("\n")
    return data
