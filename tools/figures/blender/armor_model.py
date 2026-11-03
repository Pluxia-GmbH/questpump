"""armor_model: hero stylised plate armor for the QuestPump move renders (Blender 5.2 LTS, EEVEE, headless). Ported
unchanged from the scratchpad prototype (v3/armor) on 2026-09-28; tools/figures/blender/render.py drives it, and the
motion side it names (motion_armor.py, clipcheck2.py) is tools/figures/motion.py and checks.py now.

    import sys; sys.path.insert(0, "<folder of this file>")      # plate_kit.py must sit next to it
    import armor_model as am
    parts = am.build(class_rgb=(0.55, 0.08, 0.08))   # linear RGB; ~4.6 s; dict name -> object
    am.stage(scene, move.get("props", []))           # stone circle, faint gold rune ring, walls/boxes (idempotent)
    am.render_setup(scene, cell_px, glow_dir=None)   # 2x cell, filter 1.5, dither 0, 16-bit PNG RGBA, TAA 48, bloom
    am.apply_camera(scene, move["camera"])           # {yaw_deg, pitch_deg, center_x, center_y (cm), ortho_scale_m}
    turns = am.arm_turns(track_frames, view)        # the arms' turns past the tassets, over the whole loop
    per frame: am.place(parts, joints, turns[i], view, sash=points); am.set_glow(parts, names, level); render

build() returns the rigid parts named per the interface (head, chest, abdomen, fauld and, per side near/far: pauldron,
upper_arm, elbow, forearm, hand, thigh, tasset, knee, shin, foot), their rune inlays 'rune_<part>' and joint lights
'glow_<joint>_<side>' / 'glow_neck' (children of their part, custom property 'glow' 0..1), 'sash' (a class-colour
ribbon bent through the motion side's points each frame) and 'sash_knot'.

Units and frames follow the model/motion interface: rig units are cm (x forward, y up, z depth, negative z toward
the camera); Blender X = x*0.01, Z = y*0.01, Y = z*0.01 + the side depth (side view: arms -/+0.26, legs -/+0.11,
near first; front view: 0, the figure turned to face +Y). Every rigid part has its pivot at its object origin:
    upper_arm/forearm/thigh/shin: proximal joint, long axis -Z, front +X
    hand: wrist, fingers -Z, back of the hand +X          foot: ankle, toes +X, sole at Z = -0.07
    abdomen, fauld: pelvis, +Z to the waist               chest: waist, +Z to the neck, front +X
    tasset_<side>: the thigh's frame, slid down the thigh
    head: neck, +Z up, face +X                            pauldron_<side>: shoulder, chest-oriented, outer -Y near
    knee_<side>/elbow_<side>: the joint, half-way (slerp) rotation between the two limb frames
place() also turns each pauldron with the upper arm's swing (PAD, per view: side views follow 60 % of the full swing
and lift 4 cm, so a forward-raised arm leaves through the opening; front views, rendered from behind, follow 30 %
of the abduction, lift 3.5 cm and, while the chest faces along the view (PAD_BACK_GATE), slide the pauldron 17 cm
back over the shoulder blade, so an overhead arm passes beyond it, hidden from the camera behind, instead of cutting
through its lame; toward overhead the swing blends into a fixed overhead one, PAD_TOP, so the pauldron holds still
where the swing's own axis spins), turns a front view's limbs with the body as it turns off the view axis
(FRONT_TURN, body_turned), lifts the helm HEAD_LIFT up its neck axis, and applies one clearance rule (arm_clear,
arm_turns): a hanging or swinging arm turns outward about the shoulder enough to pass its own side's tasset, easing
in and out over the loop, a planted hand slipping 2 cm at most in the picture and a holding one staying put. The
gauntlet's bell cuff belongs to the forearm (its wrist end), so a bent wrist leaves the cuff instead of cutting
through it.

v3 clip fixes (clipcheck2.py measures them; the v2 values sit next to each constant): the couter and poleyn caps
are shaped over the finished tubes (joint_grid + cleared: every cap point pushed out of the tubes over the joint's
bend range), the tube ends are trimmed where a neighbour swings in (ELBOW_BEND, KNEE_BACK with a leather guard behind
the knee, WRIST_CUT on the cuff, ANKLE_CUT on the greave, HIP on the cuisse and tasset), abdomen lame a tucks under
a slightly flared cuirass edge (WAIST), and the fauld lames became a back-only culet (FAULD_ARC) with wider,
lower tassets covering the hips' sides.

r5 (armor side of the last clip and contact fixes; motion_armor.py carries the motion side):
  hip  the squat's thighs used to rise into the belly plates past ~50 deg of hip flexion (53 mm, seen). Now lame a's
       front edge arches up over the thighs (HIP5 arch), the lower belly is the fauld: a lame riveted to the hips
       (the gold pivot bosses on the broad war-belt) that rides up about the pivot as the thighs rise (fauld_ramp,
       a smoothstep of hip flexion kept above fauld_turn, the smallest turn that clears thighs and tassets), the
       tassets are their own parts sliding down their thighs in a deep squat (tasset_slide), and the cuisse's top is
       trimmed at its inner front where the belt and buckle meet it (HIP5 inner_cut, 32 cage columns so the trim
       survives subdivision). The belt is narrow at the buckle and broad over the hips.
  helm the long class-colour plume hung back and down behind the helm and lay 16 cm into the floor in the glute
       bridge; the class colour now rides the crest as a horsehair brush in tufts along a lower steel fin (BRUSH),
       inside the helm's own outline from any side, so a resting helm lies on its bell and bucket.
  cuff the flared bell of the gauntlet cuff is cut short and unflared over the palm side (WRIST_CUT heel), where it
       met the wall at the bottom of the wall push-up; the back of the wrist keeps the full bell.
Each plate is a parametric quad cage run through plate_kit's stack (Subdivision, [hammer wobble], Solidify with a
rim slot, angle-limited Bevel with a polished slot, GN edge mask), rolled gold edges on its open borders and GN
rivets; the pieces of one part are frozen and joined into one mesh with a painted per-plate gradient ('grad'), a
baked self-occlusion ('cav') and an inverted-hull outline sized in final pixels.
"""
import math
import os
import sys

import bmesh
import bpy
from mathutils import Matrix, Vector
from mathutils.bvhtree import BVHTree

HERE = os.path.dirname(os.path.abspath(__file__))
if HERE not in sys.path:
    sys.path.insert(0, HERE)
import plate_kit as pk  # noqa: E402

U = 0.01
ARM_DEPTH = 0.26
LEG_DEPTH = 0.11
SEG = {"thigh": 0.45, "shin": 0.43, "upper_arm": 0.29, "forearm": 0.27, "abdomen": 0.20, "chest": 0.32,
       "head": 0.20, "hand": 0.17, "foot": 0.15}
SOLE_Z = -0.07
PAULDRON_FOLLOW = 0.42        # v2 schedule, kept for reference: follow 0.42, lift 0.022 m from 40 to 180 deg
PAULDRON_LIFT = 0.022
# pauldron schedule per view (place() -> pauldron_matrix): follow = the fraction of the arm's swing the pauldron
# turns with ('swing': all of it; 'hinge': its part about the shoulder's own hinge, chest-down x out, the abduction),
# lift/out/back in metres reached between a0 and a1 degrees of that swing
PAD = {
    # side views (flexion, elbows flaring): the pauldron turns with 60 % of the swing and rises 4 cm by 90 deg, so
    # the forward-raised arm leaves through its opening
    "side": {"axis": "swing", "follow": 0.6, "tilt0": 4.0, "lift0": 0.012, "out0": 0.02, "lift": 0.04, "out": 0.0,
             "back": 0.0, "a0": 20.0, "a1": 90.0},
    # front views (abduction, rendered from behind): the pauldron turns with 30 % of the abduction, rises 3.5 cm and
    # slides 17 cm back over the shoulder blade by 45 deg, so the raised arm passes in front of it (hidden from the
    # camera behind) instead of cutting through its lame
    "front": {"axis": "hinge", "follow": 0.3, "tilt0": 4.0, "lift0": 0.012, "out0": 0.02, "lift": 0.035, "out": 0.0,
              "back": 0.17, "a0": 5.0, "a1": 45.0},
}
# toward overhead the swing's axis (chest-down x arm) turns with every small move of the arm: from PAD_TOP[0] degrees of
# swing (an arm rising toward the view's own overhead: forward in side views, outward in front views) or PAD_TOP[1]
# (one rising the opposite way, back or across; in between by the swing's direction) the swing blends into that
# overhead one, reached at PAD_TOP[2], so an arm at or near overhead holds its pauldron still. The opposite way starts
# early because there the blend turns the pauldron back across its rest (an arm reaching back over a wheel), spread
# over 90 degrees of the arm instead of 50
PAD_TOP = (120.0, 80.0, 170.0)
# the slide back holds while the chest's front lies along the view axis (|world Y| of it, smoothstep between these),
# where it moves the pauldron in depth; a folded or turned chest would carry it up or sideways off the shoulder
PAD_BACK_GATE = (0.5, 0.85)
JOINT_CLEAR = 0.006           # couter/poleyn: gap kept over the tubes (plate wobble and subdivision shrink included)
# couter (v2: R 0.074, H 0.080, cx 0.004, kz 1, polar 80, fan 1/1/0.072, no clearance push)
ELBOW = {"R": 0.085, "H": 0.102, "cx": 0.0, "kz": 1.25, "polar1": 74.0, "fan_scale": 1.0, "fan_kz": 1.0,
         "fan_y": 0.092, "clear": True, "clear_cop": True, "clear_fan": False, "alphas": (0, 25, 50, 75, 100)}
# poleyn (v2: R 0.092, H 0.104, cx 0.02, kz 1, polar 80, fan 1/0.102, lames at +-0.076 r 0.108/0.101, no push)
KNEE = {"R": 0.100, "H": 0.112, "cx": 0.035, "kz": 1.15, "polar1": 70.0, "fan_scale": 1.0, "fan_y": 0.110,
        "lame_z": 0.080, "lame_r": (0.120, 0.110), "lame_inner": 55.0, "ky": 0.92, "clear": True,
        "clear_cop": True, "clear_fan": False, "clear_lames": True, "alphas": (0, 30, 60, 90, 115),
        "back": (0.07, 0.056, 62.0)}     # leather guard behind the knee: half height, radius, half arc (deg)
# trimmed ends (depth m, centre deg, half width deg; 0 deg = front): the bend of the elbow, the back of the knee
ELBOW_BEND = {"rerebrace": (0.025, 25.0, 95.0), "ua_lame": (0.045, 25.0, 110.0), "vambrace": (0.05, 25.0, 110.0)}
ELBOW_TUBE = {"ua_end": 0.9, "vamb_top": 0.92, "vamb_keel0": 0.25}   # radius factors at the elbow ends
KNEE_BACK = {"cuisse": (0.085, 180.0, 120.0), "greave": (0.085, 180.0, 120.0)}
# waist: abdomen lame a tucks WAIST["tuck"] m up under the cuirass, tapering to "top" (front, back, side radii)
WAIST = {"tuck": 0.02, "t_mid": 0.7, "top": (0.090, 0.078, 0.114), "keel_end": 0.65, "flare": 0.012}
# arm vs tasset clearance (arm_clear, arm_turns): the capsules (m), the largest turn (deg), a planted hand's most
# slip in the picture (m) and the turn's most change a tick (deg)
ARM_CLEAR = {"tasset_len": 0.15, "tasset_r": 0.166, "elbow_r": 0.100, "forearm_r": 0.100, "hand_r": 0.060,
             "max_deg": 25.0, "slip": 0.02, "rate_deg": 1.5}
HEAD_LIFT = 0.008             # the helm sits this far up its neck axis, so a nodding helm clears the cuirass
# a front view's limbs turn from the view's own rule (local X = world +Y, the front of a figure facing away) to the side
# view's (the hip or shoulder line as local Y) as the body turns off the view axis: by its line's turn (body_turned,
# degrees, smoothstep between these). A skater's 22 and a pec stretch's 30 keep the view's rule; a turned pistol's hips
# (25-55) blend; a side crow (90-180) and a flag facing the camera (180) take the side view's, which their bent knees
# and elbows face
FRONT_TURN = (30.0, 60.0)
FRONT_OPPOSE = math.radians(120.0)   # a front-view limb's two fronts this far apart blend toward the line's
WRIST_CUT = {"back": (0.065, 0.0, 90.0), "palm": (0.02, 180.0, 60.0), "thumb": (0.035, 75.0),
             "little": (0.015, 45.0),      # the cuff's end trimmed where the hand bends (depth m, half width deg)
             "heel": (0.075, 180.0, 40.0, 60.0)}  # r5: the bell cut short (and unflared) over the palm side
#                               (depth m, centre deg, full depth +-deg, fading over deg), where it met the wall at the
#                               bottom of the wall push-up
ANKLE_CUT = ((0.035, 0.0, 75.0), (0.02, 180.0, 60.0))   # greave ankle end trimmed at the instep and the heel
# hip: tasset arc (deg from the front, + toward the outer side), its top/bottom (m, thigh frame), trims
FAULD_ARC = (145.0, 215.0)    # the fauld lames' arc around the pelvis (deg from the front)
HIP = {"tasset_th": (0.0, 165.0), "tasset_z": (0.0, -0.15), "tasset_cut0": (0.05, 0.0, 70.0),
       "cuisse_cut0": (0.09, 0.0, 80.0)}     # v2: tasset -30..125 deg, +0.055..-0.13 m, no trims
KNEE_KEEL_END = 0.88          # the cuisse keel fades out before the poleyn (v2: ran to the end)
# r5 hip articulation (the squat's thigh tops used to rise into the belly plates past ~50 deg of hip flexion):
#   lame a: its front edge arches up (depth m, flat to +-deg, back to the plain edge at +-deg) so the thighs pass
#   under it; the fauld: a lame between the belt and that arch, riveted at the hips (pivot: x, z m in the abdomen
#   frame), which rides up over the rising thighs (place(): the smallest turn that clears both thighs and tassets,
#   margin m, at most max_deg); the tassets: their own parts, sliding down their thighs (slide: from deg, full at
#   deg, metres) so their tops stay clear of the belly in a deep squat
HIP5 = {"arch": (0.105, 50.0, 85.0), "tuck": 0.012, "arc": 76.0, "pivot": (-0.02, 0.13), "margin": 0.006,
        "max_deg": 100.0, "step_deg": 0.25, "slide": (40.0, 110.0, 0.16),
        "belt": (0.05, 0.11, 30.0, 75.0),          # belt top: m at the buckle, m over the hips, from deg, full deg
        "tasset_grow": 0.016,                      # tassets stand off the cuisse (its keel) while they slide
        "inner_cut": (0.13, 65.0, 45.0),           # cuisse top trimmed at its inner front (depth m, deg in, half)
        "buckle_w": 0.030, "cuisse_nu": 32,        # buckle half width (m); cuisse cage columns (keeps the trims)
        "turn": (40.0, 108.0, 70.0),               # the fauld's ride: from deg of hip flexion, full at deg, max deg
        "tasset_rune": (48.0, -0.092, 0.07)}       # the tasset's rune: deg out from the front, height m, size m
if os.environ.get("QP_HIP5"):              # lab override (JSON): HIP5 keys, and cuisse/tasset trims in HIP
    import json as _json
    _o = _json.loads(os.environ["QP_HIP5"])
    for _k in ("tasset_cut0", "cuisse_cut0", "tasset_z", "tasset_th"):
        if _k in _o:
            HIP[_k] = tuple(_o.pop(_k))
    HIP5.update({k: tuple(v) if isinstance(v, list) else v for k, v in _o.items()})
HELM_SCALE = 1.07             # hero proportion: the helm reads a little larger than a head
# the helm's crest: a steel fin (rise m; v3: 0.078) carrying a class-colour horsehair brush (fin parameter range,
# height m, sweep back, width m, tufts)
BRUSH = {"fin": 0.040, "u": (0.05, 0.84), "h": 0.075, "front": 0.75, "sweep": 0.55, "thick": 0.036, "tufts": 7,
         "tuft_depth": 0.16, "seat": 0.008}
HAMMER = True                 # low-frequency hand-made wobble (plate_kit hammer group) on the big plates
KEY_ENERGY = math.pi          # a white Lambert facing the key reads 1.0 (measured: value = energy / pi * N.L)
ARMOR_COLL, STAGE_COLL = "qp_armor", "qp_stage"
OUTLINE_PX = {"default": 1.7, "head": 2.0, "chest": 2.0, "abdomen": 2.0, "pauldron": 2.0, "thigh": 1.9,
              "sash": 1.4}

RIGID = ["head", "chest", "abdomen", "fauld", "culet"] + [f"{p}_{s}" for s in ("near", "far") for p in
                                                 ("pauldron", "upper_arm", "elbow", "forearm", "hand",
                                                  "thigh", "tasset", "knee", "shin", "foot")]


# ================================================================ colour

def hexc(h):
    return pk.hexc(h)


def _mix_lin(a, b, t):
    return tuple(x + (y - x) * t for x, y in zip(a, b))


def _scale(c, k):
    return tuple(min(1.0, x * k) for x in c)


PAL = {
    "steel": ("#1b2233", "#10141f", "#46526c", "#96a8c6", "#f6faff"),     # shadow, core, mid, lit, highlight
    "steel_hi": ("#2e3850", "#1c2336", "#76869f", "#c9d4e8", "#ffffff"),
    "steel_dark": ("#0b0e17", "#070910", "#222a3d", "#46526c", "#a9b5cf"),
    "gold": ("#3a1e06", "#2a1404", "#96601a", "#e0ad44", "#fff3c4"),
    "gold_hi": ("#6a3c0c", "#4a2a08", "#c98a28", "#ffd46e", "#ffffff"),
    "leather": ("#1c1009", "#130a05", "#4a2c17", "#7e5230", "#a8784c"),
}
OUTLINE_HEX = "#0b0e16"


# ================================================================ node helpers

def _sock(node, key, out=False):
    for s in (node.outputs if out else node.inputs):
        if not s.is_unavailable and (s.name == key or s.identifier == key):
            return s
    raise KeyError(f"{node.bl_idname}: no {'output' if out else 'input'} {key!r}")


def _new_mat(name):
    m = bpy.data.materials.new(name)
    nt = m.node_tree
    for n in list(nt.nodes):
        if n.type != "OUTPUT_MATERIAL":
            nt.nodes.remove(n)
    return m, nt, next(n for n in nt.nodes if n.type == "OUTPUT_MATERIAL")


def _ramp(nt, stops, interp="LINEAR"):
    r = nt.nodes.new("ShaderNodeValToRGB")
    cr = r.color_ramp
    cr.interpolation = interp
    cr.elements[0].position = stops[0][0]
    cr.elements[0].color = (*stops[0][1], 1.0)
    cr.elements[-1].position = stops[-1][0]
    cr.elements[-1].color = (*stops[-1][1], 1.0)
    for pos, col in stops[1:-1]:
        e = cr.elements.new(pos)
        e.color = (*col, 1.0)
    return r


def _math(nt, op, a=None, b=None, c=None, clamp=False):
    n = nt.nodes.new("ShaderNodeMath")
    n.operation = op
    n.use_clamp = clamp
    for i, v in enumerate((a, b, c)):
        if v is not None:
            n.inputs[i].default_value = v
    return n


def _mix(nt, blend, fac=1.0, clamp=False):
    n = nt.nodes.new("ShaderNodeMix")
    n.data_type = "RGBA"
    n.blend_type = blend
    n.clamp_result = clamp
    _sock(n, "Factor").default_value = fac
    return n


def _maprange(nt, fmin, fmax, tmin=0.0, tmax=1.0, clamp=True):
    n = nt.nodes.new("ShaderNodeMapRange")
    _sock(n, "From Min").default_value = fmin
    _sock(n, "From Max").default_value = fmax
    _sock(n, "To Min").default_value = tmin
    _sock(n, "To Max").default_value = tmax
    n.clamp = clamp
    return n


def _attr(nt, name, kind="GEOMETRY"):
    a = nt.nodes.new("ShaderNodeAttribute")
    a.attribute_type = kind
    a.attribute_name = name
    return a


def _lit_value(nt, N, L):
    """Lambert-only lit value of the key (plus ambient), through Shader to RGB: the toon bands' input."""
    dif = N("ShaderNodeBsdfDiffuse")
    _sock(dif, "Color").default_value = (1, 1, 1, 1)
    s2r = N("ShaderNodeShaderToRGB")
    bw = N("ShaderNodeRGBToBW")
    L(_sock(dif, "BSDF", True), _sock(s2r, "Shader"))
    L(_sock(s2r, "Color", True), _sock(bw, "Color"))
    return _sock(bw, "Val", True)


def _multiply_by(nt, col, fac_socket, lo, hi):
    """col * (lo + (hi - lo) * fac)."""
    mr = _maprange(nt, 0.0, 1.0, lo, hi)
    nt.links.new(fac_socket, _sock(mr, "Value"))
    m = _mix(nt, "MULTIPLY")
    nt.links.new(col, _sock(m, "A"))
    nt.links.new(_sock(mr, "Result", True), _sock(m, "B"))
    return _sock(m, "Result", True)


# ================================================================ materials

def mat_toon(name, pal, spec_amt=1.0, spec_rough=0.22, band_amt=0.35, edge_amt=0.55, rim_amt=0.30,
             rim="#9fc2ff", wear=0.07, grad=(0.70, 1.12), bands=(0.18, 0.30, 0.66), soft=0.025, scratches=0.5,
             cavity=0.62):
    """Painted plate for EEVEE, emission out (exact palette under Standard):
    Lambert value -> 4 flat bands (reflected shadow, core shadow, mid, lit) x the painted per-plate gradient
    ('grad' attribute) x a top light from the normal x brush mottling; + one hard specular shape (a sharp Glossy
    through Shader to RGB, stepped), + a painted chrome horizon line, + polished edges (GN 'edge'), + light
    scratches, + a cool rim on the lit side. Clamped to 1 so only real emitters pass the bloom threshold."""
    S, C, M, Li, H = (hexc(h) for h in pal)
    m, nt, out = _new_mat(name)
    N, L = nt.nodes.new, nt.links.new
    v = _lit_value(nt, N, L)
    a, b, c = bands
    rb = _ramp(nt, [(0.0, S), (a, S), (a + soft, C), (b, C), (b + soft, M), (c, M), (c + soft, Li), (1.0, Li)])
    L(v, _sock(rb, "Fac"))
    col = _sock(rb, "Color", True)
    # painted per-plate gradient (0 at the bottom of each plate, 1 at its top, part space)
    col = _multiply_by(nt, col, _sock(_attr(nt, "grad"), "Fac", True), grad[0], grad[1])
    # painted occlusion in the crevices (baked 'cav' attribute; 0 where missing)
    cv = _maprange(nt, 0.12, 0.62, 0.0, 1.0)
    L(_sock(_attr(nt, "cav"), "Fac", True), _sock(cv, "Value"))
    col = _multiply_by(nt, col, _sock(cv, "Result", True), 1.0, 1.0 - cavity)
    # top light: faces looking up read lighter
    geo = N("ShaderNodeNewGeometry")
    sep = N("ShaderNodeSeparateXYZ")
    L(_sock(geo, "Normal", True), _sock(sep, "Vector"))
    nz = _maprange(nt, -1.0, 1.0, 0.0, 1.0)
    L(_sock(sep, "Z", True), _sock(nz, "Value"))
    col = _multiply_by(nt, col, _sock(nz, "Result", True), 0.90, 1.07)
    # brush mottling (object space so it sticks to the part)
    tc = N("ShaderNodeTexCoord")
    nzt = N("ShaderNodeTexNoise")
    _sock(nzt, "Scale").default_value = 7.0
    _sock(nzt, "Detail").default_value = 2.0
    L(_sock(tc, "Object", True), _sock(nzt, "Vector"))
    col = _multiply_by(nt, col, _sock(nzt, "Fac", True), 1.0 - wear * 2.2, 1.0 + wear * 1.2)
    # hard specular shape from a sharp Glossy lobe of the key
    gls = N("ShaderNodeBsdfAnisotropic")
    _sock(gls, "Roughness").default_value = spec_rough
    _sock(gls, "Color").default_value = (1, 1, 1, 1)
    s2 = N("ShaderNodeShaderToRGB")
    b2 = N("ShaderNodeRGBToBW")
    L(_sock(gls, "BSDF", True), _sock(s2, "Shader"))
    L(_sock(s2, "Color", True), _sock(b2, "Color"))
    hstep = _maprange(nt, 1.6, 1.9, 0.0, spec_amt)
    L(_sock(b2, "Val", True), _sock(hstep, "Value"))
    hm = _mix(nt, "MIX")
    _sock(hm, "B").default_value = (*H, 1)
    L(_sock(hstep, "Result", True), _sock(hm, "Factor"))
    L(col, _sock(hm, "A"))
    col = _sock(hm, "Result", True)
    # painted chrome horizon line (reflection vector crossing the horizon), plus a faint sky above it
    rs = N("ShaderNodeSeparateXYZ")
    L(_sock(tc, "Reflection", True), _sock(rs, "Vector"))
    rz = _maprange(nt, -1.0, 1.0, 0.0, 1.0)
    L(_sock(rs, "Z", True), _sock(rz, "Value"))
    hz = _ramp(nt, [(0.0, (0, 0, 0)), (0.505, (0, 0, 0)), (0.53, (1, 1, 1)), (0.575, (0.18, 0.18, 0.18)),
                    (1.0, (0.10, 0.10, 0.10))])
    L(_sock(rz, "Result", True), _sock(hz, "Fac"))
    lit_gate = _maprange(nt, a, c + soft, 0.35, 1.0)          # the line is brightest where the plate is lit
    L(v, _sock(lit_gate, "Value"))
    hzm = _math(nt, "MULTIPLY")
    L(_sock(hz, "Color", True), hzm.inputs[0])
    L(_sock(lit_gate, "Result", True), hzm.inputs[1])
    hzs = _math(nt, "MULTIPLY", b=band_amt)
    L(hzm.outputs[0], hzs.inputs[0])
    ha = _mix(nt, "ADD")
    _sock(ha, "B").default_value = (*H, 1)
    L(col, _sock(ha, "A"))
    L(hzs.outputs[0], _sock(ha, "Factor"))
    col = _sock(ha, "Result", True)
    # polished convex edges (GN 'edge' mask; 0 where missing)
    em = _math(nt, "MULTIPLY", b=edge_amt)
    L(_sock(_attr(nt, "edge"), "Fac", True), em.inputs[0])
    ea = _mix(nt, "ADD")
    _sock(ea, "B").default_value = (*H, 1)
    L(col, _sock(ea, "A"))
    L(em.outputs[0], _sock(ea, "Factor"))
    col = _sock(ea, "Result", True)
    # painted scratches: sparse thin light streaks (stretched noise, stepped)
    if scratches > 0:
        mp = N("ShaderNodeMapping")
        _sock(mp, "Scale").default_value = (38.0, 38.0, 4.0)
        _sock(mp, "Rotation").default_value = (0.3, 0.5, 0.2)
        L(_sock(tc, "Object", True), _sock(mp, "Vector"))
        sn = N("ShaderNodeTexNoise")
        _sock(sn, "Scale").default_value = 1.0
        _sock(sn, "Detail").default_value = 0.0
        L(_sock(mp, "Vector", True), _sock(sn, "Vector"))
        ss = _maprange(nt, 0.72, 0.745, 0.0, scratches)
        L(_sock(sn, "Fac", True), _sock(ss, "Value"))
        sa = _mix(nt, "SCREEN")
        _sock(sa, "B").default_value = (*_mix_lin(Li, H, 0.5), 1)
        L(col, _sock(sa, "A"))
        L(_sock(ss, "Result", True), _sock(sa, "Factor"))
        col = _sock(sa, "Result", True)
    # cool rim on the lit side
    lw = N("ShaderNodeLayerWeight")
    _sock(lw, "Blend").default_value = 0.3
    rstep = _maprange(nt, 0.66, 0.70, 0.0, rim_amt)
    L(_sock(lw, "Facing", True), _sock(rstep, "Value"))
    litm = _maprange(nt, b, c, 0.4, 1.0)
    L(v, _sock(litm, "Value"))
    rm = _math(nt, "MULTIPLY")
    L(_sock(rstep, "Result", True), rm.inputs[0])
    L(_sock(litm, "Result", True), rm.inputs[1])
    ra = _mix(nt, "SCREEN", clamp=True)
    _sock(ra, "B").default_value = (*hexc(rim), 1)
    L(col, _sock(ra, "A"))
    L(rm.outputs[0], _sock(ra, "Factor"))
    col = _sock(ra, "Result", True)
    e = N("ShaderNodeEmission")
    L(col, _sock(e, "Color"))
    L(_sock(e, "Emission", True), _sock(out, "Surface"))
    return m


def mat_cloth(name, rgb):
    """Class-colour cloth: the same flat bands, hue-shifted cool shadows, a soft sheen instead of a spec."""
    lit = tuple(min(1.0, x) for x in rgb)
    mid = _scale(lit, 0.52)
    core = tuple(x * 0.16 for x in _mix_lin(lit, (0.12, 0.05, 0.25), 0.35))
    shadow = tuple(x * 0.26 for x in _mix_lin(lit, (0.12, 0.05, 0.25), 0.3))
    hi = _mix_lin(lit, (1.0, 0.95, 0.85), 0.35)
    to_hex = lambda c: "#" + "".join(f"{int(round(255 * (1.055 * x ** (1 / 2.4) - 0.055 if x > 0.0031308 else 12.92 * x))):02x}" for x in c)
    return mat_toon(name, tuple(to_hex(c) for c in (shadow, core, mid, lit, hi)), spec_amt=0.0, band_amt=0.0,
                    edge_amt=0.0, rim_amt=0.15, rim="#ffd9c0", wear=0.05, grad=(0.82, 1.08), scratches=0.0)


def mat_glow(name, on, off, strength=4.5, base=0.0):
    """Emitter switched by the object's custom property 'glow' (0..1): off colour at strength 1, on colour at
    `strength` (above the bloom threshold). `base` is added to the property (joint lights never go fully dark)."""
    m, nt, out = _new_mat(name)
    N, L = nt.nodes.new, nt.links.new
    at = _attr(nt, "glow", "OBJECT")
    g = _math(nt, "ADD", b=base, clamp=True)
    L(_sock(at, "Fac", True), g.inputs[0])
    col = _mix(nt, "MIX")
    _sock(col, "A").default_value = (*off, 1)
    _sock(col, "B").default_value = (*on, 1)
    L(g.outputs[0], _sock(col, "Factor"))
    st = _maprange(nt, 0.0, 1.0, 1.0, strength)
    L(g.outputs[0], _sock(st, "Value"))
    e = N("ShaderNodeEmission")
    L(_sock(col, "Result", True), _sock(e, "Color"))
    L(_sock(st, "Result", True), _sock(e, "Strength"))
    L(_sock(e, "Emission", True), _sock(out, "Surface"))
    return m


def mat_emit(name, rgb, strength):
    m, nt, out = _new_mat(name)
    e = nt.nodes.new("ShaderNodeEmission")
    _sock(e, "Color").default_value = (*rgb, 1)
    _sock(e, "Strength").default_value = strength
    nt.links.new(_sock(e, "Emission", True), _sock(out, "Surface"))
    return m


def dim_visor(level):
    """The visor's light at `level` of its build strength (a move with its eyes closed shows a dark slit)."""
    m = bpy.data.materials.get("qp_visor")
    if m:
        e = next(n for n in m.node_tree.nodes if n.type == "EMISSION")
        _sock(e, "Strength").default_value *= level


def mat_outline():
    m = bpy.data.materials.get("qp_outline")
    if m:
        return m
    m, nt, out = _new_mat("qp_outline")
    m.use_backface_culling = True
    m.use_backface_culling_shadow = True
    e = nt.nodes.new("ShaderNodeEmission")
    _sock(e, "Color").default_value = (*hexc(OUTLINE_HEX), 1)
    nt.links.new(_sock(e, "Emission", True), _sock(out, "Surface"))
    return m


def mat_stone(name, shadow, lit, hi, speck=0.10, scale=3.0, lift=0.0):
    """Stone lit by the soft floor light: a smooth (not banded) value ramp so the contact shadow stays soft,
    with painted mottling and the per-slab gradient."""
    m, nt, out = _new_mat(name)
    N, L = nt.nodes.new, nt.links.new
    v = _lit_value(nt, N, L)
    if lift:
        lv = _math(nt, "MULTIPLY_ADD", b=1.0 - lift, c=lift)
        L(v, lv.inputs[0])
        v = lv.outputs[0]
    rp = _ramp(nt, [(0.0, hexc(shadow)), (0.85, hexc(lit)), (1.0, hexc(hi))])
    L(v, _sock(rp, "Fac"))
    col = _sock(rp, "Color", True)
    col = _multiply_by(nt, col, _sock(_attr(nt, "grad"), "Fac", True), 0.86, 1.06)
    tc = N("ShaderNodeTexCoord")
    nz = N("ShaderNodeTexNoise")
    _sock(nz, "Scale").default_value = scale
    _sock(nz, "Detail").default_value = 4.0
    L(_sock(tc, "Object", True), _sock(nz, "Vector"))
    col = _multiply_by(nt, col, _sock(nz, "Fac", True), 1.0 - speck * 2, 1.0 + speck)
    e = N("ShaderNodeEmission")
    L(col, _sock(e, "Color"))
    L(_sock(e, "Emission", True), _sock(out, "Surface"))
    return m


def palette(class_rgb):
    M = {k: mat_toon(f"qp_{k}", v) for k, v in PAL.items()}
    M["leather"] = mat_toon("qp_leather", PAL["leather"], spec_amt=0.0, band_amt=0.05, edge_amt=0.15, rim_amt=0.1,
                            scratches=0.0)
    M["cloth"] = mat_cloth("qp_cloth", class_rgb)
    M["groove"] = mat_toon("qp_groove", ("#07080d", "#05060a", "#10141f", "#1c2232", "#3a4458"), spec_amt=0.0,
                           band_amt=0.0, edge_amt=0.0, rim_amt=0.0, scratches=0.0, wear=0.0)
    M["rune"] = mat_glow("qp_rune", on=(1.0, 0.64, 0.10), off=hexc("#7a5420"), strength=1.55)
    M["joint"] = mat_glow("qp_joint_glow", on=(1.0, 0.62, 0.16), off=hexc("#a86e1e"), strength=1.45, base=0.0)
    M["visor"] = mat_emit("qp_visor", (1.0, 0.82, 0.40), 3.2)
    M["outline"] = mat_outline()
    return M


# ================================================================ geometry helpers

def section(th, rf, rb, ry, e=2.0):
    """Point on a superellipse section: th = 0 front (+X), 90 deg = +Y; rf/rb front/back half-depths."""
    c, s = math.cos(th), math.sin(th)
    rx = rf if c >= 0 else rb
    k = (abs(c) ** e + abs(s) ** e) ** (-1.0 / e)
    return rx * k * c, ry * k * s


def lerp_prof(pts, t):
    """Smooth piecewise profile through [(t, value), ...]."""
    if t <= pts[0][0]:
        return pts[0][1]
    for (t0, a), (t1, b) in zip(pts, pts[1:]):
        if t0 <= t <= t1:
            u = (t - t0) / (t1 - t0) if t1 > t0 else 0.0
            u = u * u * (3 - 2 * u)
            return a + (b - a) * u
    return pts[-1][1]


def _cut(cut, th):
    """How far (m) a trimmed end moves in at angle th: cut = (depth, centre_deg, half_width_deg), or a list of them
    (the deepest wins), or a callable th -> depth."""
    if not cut:
        return 0.0
    if callable(cut):
        return cut(th)
    if isinstance(cut[0], (tuple, list)):
        return max(_cut(c, th) for c in cut)
    dep, c, hw = cut
    d = abs((th - math.radians(c) + math.pi) % (2 * math.pi) - math.pi)
    x = max(0.0, 1.0 - d / math.radians(hw))
    return dep * x * x * (3 - 2 * x)


def _bend(cut, o):
    """An elbow-bend trim (depth, centre offset toward the arm's inner side in deg, half width) for side o."""
    dep, off, hw = cut
    return (dep, -o * off, hw)


def tube(z0, z1, rf, rb, ry, dx=None, e=2.2, bump=None, th0=None, th1=None, cut0=None, cut1=None):
    """Parametric tube along Z (t = 0 at z0 .. 1 at z1). rf/rb/ry/dx: profiles [(t, value)]. bump(t, th) adds
    radius (a keel or a ridge). th0/th1 limit the angle range (open plates). cut0/cut1 = (depth m, centre deg,
    half width deg) trim the z0/z1 end back toward the middle around that angle (where a joint's neighbour swings
    in: the bend of the elbow, the back of the knee)."""
    full = th0 is None
    sg = 1.0 if z1 > z0 else -1.0

    def f(u, v):
        th = 2 * math.pi * u if full else th0 + (th1 - th0) * u
        t = v
        a = z0 + sg * _cut(cut0, th)
        b = z1 - sg * _cut(cut1, th)
        z = a + (b - a) * t
        x, y = section(th, lerp_prof(rf, t), lerp_prof(rb, t), lerp_prof(ry, t), e)
        if bump:
            b = bump(t, th)
            x += b * math.cos(th)
            y += b * math.sin(th)
        return (x + (lerp_prof(dx, t) if dx else 0.0), y, z)
    return f


def keel(width_deg, height, t0=0.0, t1=1.0, center=0.0):
    """A ridge along the tube at angle `center` (radians), fading at the ends of [t0, t1]."""
    w = math.radians(width_deg)

    def b(t, th):
        d = abs((th - center + math.pi) % (2 * math.pi) - math.pi)
        if d > w or t < t0 or t > t1:
            return 0.0
        edge = min(1.0, (t - t0) / 0.12 if t0 > 0 else 1.0, (t1 - t) / 0.12 if t1 < 1 else 1.0)
        return height * (1 - d / w) ** 1.6 * edge
    return b


def crease(ob, pred, value=1.0):
    """Edge crease on cage edges whose two vertex positions satisfy pred(a, b)."""
    me = ob.data
    cr = me.edge_creases_ensure()
    for ed in me.edges:
        a, b = (me.vertices[i].co for i in ed.vertices)
        if pred(a, b):
            cr.data[ed.index].value = value


def on_tube(f, u, v, lift=0.003):
    """Point on a tube() surface at (u, v), pushed `lift` along the surface normal (pointing away from the Z axis)."""
    e = 1e-3
    p = Vector(f(u, v))
    du = Vector(f(min(1.0, u + e), v)) - Vector(f(max(0.0, u - e), v))
    dv = Vector(f(u, min(1.0, v + e))) - Vector(f(u, max(0.0, v - e)))
    n = du.cross(dv)
    if n.length < 1e-12:
        n = Vector((p.x, p.y, 0.0))
    n.normalize()
    if n.dot(Vector((p.x, p.y, 0.0))) < 0:
        n = -n
    return tuple(p + n * lift)


def mesh_obj(name, verts, faces, mat=None, smooth=True):
    me = bpy.data.meshes.new(name)
    me.from_pydata([tuple(v) for v in verts], [], [tuple(f) for f in faces])
    me.validate()
    if smooth:
        me.shade_smooth()
    if mat:
        me.materials.append(mat)
    return pk.link(bpy.data.objects.new(name, me))


def sweep(name, pts, r, mat, ring=10, caps=True):
    """A round tube along a polyline (gold ridges and cords)."""
    pts = [Vector(p) for p in pts]
    verts, faces = [], []
    n = len(pts)
    prev = None
    for i, p in enumerate(pts):
        t = (pts[min(n - 1, i + 1)] - pts[max(0, i - 1)]).normalized()
        if prev is None:
            prev = t.orthogonal().normalized()
        b1 = (prev - t * prev.dot(t)).normalized()
        b2 = t.cross(b1)
        prev = b1
        for k in range(ring):
            a = 2 * math.pi * k / ring
            verts.append(tuple(p + (b1 * math.cos(a) + b2 * math.sin(a)) * r))
    for i in range(n - 1):
        for k in range(ring):
            a0, a1 = i * ring + k, i * ring + (k + 1) % ring
            faces.append((a0, a1, a1 + ring, a0 + ring))
    if caps:
        faces.append(tuple(range(ring))[::-1])
        faces.append(tuple((n - 1) * ring + k for k in range(ring)))
    return mesh_obj(name, verts, faces, mat)


def disc_plate(name, R, H, polar0, polar1, nu=24, nv=5, sx=1.0, rot=None, offs=None):
    """A domed cap (sphere zone) around +Z: polar angle polar0..polar1 (deg), radius R, height scale H/R."""
    def f(u, v):
        th = 2 * math.pi * u
        ph = math.radians(polar0 + (polar1 - polar0) * v)
        p = Vector((sx * R * math.sin(ph) * math.cos(th), R * math.sin(ph) * math.sin(th), H * math.cos(ph)))
        if rot:
            p = rot @ p
        if offs:
            p = p + Vector(offs)
        return tuple(p)
    return f


# ================================================================ joint clearance (caps that ride over the tubes)

def _verts_np(ob):
    import numpy as np
    me = ob.data
    V = np.empty(len(me.vertices) * 3)
    me.vertices.foreach_get("co", V)
    return V.reshape(-1, 3)


def joint_grid(pieces, alphas, zlo=-0.20, zhi=0.20, dz=0.004, nphi=72, dil_z=2, dil_phi=2):
    """Max distance from the joint's Z axis of the neighbouring tube meshes, per (z, phi) bin, over the flexion
    angles `alphas` (deg). pieces: [(object, to_joint(alpha_rad) -> 4x4 Matrix)]. Returns req(z, phi) -> radius
    (0 where no tube passes)."""
    import numpy as np
    nz = int(round((zhi - zlo) / dz)) + 1
    G = np.zeros((nz, nphi))
    for ob, fn in pieces:
        V = _verts_np(ob)
        for a in alphas:
            M = np.array([list(r) for r in fn(math.radians(a))])
            P = V @ M[:3, :3].T + M[:3, 3]
            rho = np.hypot(P[:, 0], P[:, 1])
            iz = np.floor((P[:, 2] - zlo) / dz).astype(int)
            ip = (np.floor((np.arctan2(P[:, 1], P[:, 0]) + math.pi) / (2 * math.pi) * nphi).astype(int)) % nphi
            ok = (iz >= 0) & (iz < nz)
            np.maximum.at(G, (iz[ok], ip[ok]), rho[ok])
    D = G.copy()
    for k in range(1, dil_z + 1):
        D[k:] = np.maximum(D[k:], G[:-k])
        D[:-k] = np.maximum(D[:-k], G[k:])
    E = D.copy()
    for k in range(1, dil_phi + 1):
        E = np.maximum(E, np.roll(D, k, axis=1))
        E = np.maximum(E, np.roll(D, -k, axis=1))

    def req(z, phi):
        i = int(math.floor((z - zlo) / dz))
        if i < 0 or i >= nz:
            return 0.0
        j = int(math.floor((phi + math.pi) / (2 * math.pi) * nphi)) % nphi
        return float(E[i, j])
    return req


def cleared(f, req, need, rim_extra=0.0, rim_v=None):
    """Wrap a cage function so every point clears the tubes: pushed out from the joint axis to at least
    req(z, phi) + need (+ rim_extra on the row v == rim_v, where the rolled edge sits)."""
    def g(u, v):
        x, y, z = f(u, v)
        r = req(z, math.atan2(y, x))
        if r > 0.0:
            r += need + (rim_extra if rim_v is not None and abs(v - rim_v) < 1e-9 else 0.0)
            rho = math.hypot(x, y)
            if 1e-6 < rho < r:
                x, y = x * r / rho, y * r / rho
        return (x, y, z)
    return g


def _rot_y(a, z_shift=0.0):
    M = Matrix.Rotation(a, 4, "Y") @ Matrix.Translation((0.0, 0.0, z_shift))
    return M


# ================================================================ part builders (all in part space, metres)

class Part:
    def __init__(self, name):
        self.name = name
        self.root = pk.part_root("tmp_" + name)
        self.hull = {}           # piece object name -> hull weight
        self.runes = []          # (glyph, anchor point, direction, height)
        self.glows = []          # (name, builder)

    def add(self, ob, hull=1.0):
        pk.adopt(self.root, ob)
        self.hull[ob.name] = hull
        return ob

    def plate(self, name, f, nu, nv, M, face="steel", rim="gold", edge="steel_hi", thick=0.018, bev=0.007,
              segs=3, level=2, roll=0.0, roll_mat="gold", cyclic=False, cap=None, inside=(0, 0, 0), skip=None,
              post=None, creases=None, offset=-1.0, hull=1.0, hammer=None):
        ob = pk.shell(name, f, nu, nv, cyclic=cyclic, cap=cap, inside=inside, skip=skip, post=post)
        if creases:
            crease(ob, creases)
        if hammer is None:                      # hand-hammered wobble on the big steel plates only
            hammer = HAMMER and face == "steel" and thick >= 0.016
        pk.plate(ob, M[face], M[rim], M[edge], thick=thick, bev=bev, segs=segs, level=level, offset=offset,
                 hammer=hammer)
        self.add(ob, hull)
        if roll > 0:
            tw = pk.add_rolled_edge(self.root, ob, M[roll_mat], radius=roll)
            self.hull[tw.name] = 0.55
        return ob

    def rivets(self, name, pts, M, r=0.008, mat="gold"):
        ob = pk.add_rivets(self.root, name, pts, M[mat], radius=r)
        self.hull[ob.name] = 0.0
        return ob

    def blob(self, name, size, loc, M, mat, bevel=0.012, hull=1.0):
        ob = pk.blob(name, size, loc=loc, mat=M[mat], bevel=bevel)
        return self.add(ob, hull)

    def ell(self, name, radii, loc, M, mat, hull=1.0, segs=24):
        ob = pk.ellipsoid(name, radii, loc=loc, mat=M[mat], segs=segs)
        return self.add(ob, hull)


def build_helm(M, cloth=True):
    """Great helm: boxy superellipse bucket with a nasal keel, a glowing T visor, a proud gold brow band with a
    glowing gem, gold side roundels, a flared bell rim, a swept crest fin and a class-colour plume."""
    P = Part("head")
    Z = [0.045, 0.075, 0.105, 0.14, 0.175, 0.205, 0.24, 0.275, 0.305, 0.33, 0.348]
    RX = [0.118, 0.119, 0.122, 0.126, 0.128, 0.128, 0.126, 0.119, 0.103, 0.076, 0.042]
    RY = [0.112, 0.108, 0.110, 0.114, 0.116, 0.116, 0.113, 0.106, 0.092, 0.066, 0.036]
    DX = [0.012, 0.010, 0.009, 0.008, 0.006, 0.004, 0.002, -0.002, -0.006, -0.008, -0.008]
    NU, EXP = 32, 2.5
    eye_row, nose_rows = 5, (3, 4)

    def lerp_rows(lst, z):
        if z <= Z[0]:
            return lst[0]
        for k in range(len(Z) - 1):
            if Z[k] <= z <= Z[k + 1]:
                t = (z - Z[k]) / (Z[k + 1] - Z[k])
                return lst[k] + (lst[k + 1] - lst[k]) * t
        return lst[-1]

    def at(th, z, grow=0.0):
        rx, ry, dxz = lerp_rows(RX, z), lerp_rows(RY, z), lerp_rows(DX, z)
        x, y = section(th, rx + grow, rx * 0.95 + grow, ry + grow, EXP)
        if z >= Z[eye_row + 1] - 1e-6:                  # nasal keel above the eye slit
            d = abs((th + math.pi) % (2 * math.pi) - math.pi)
            if d < math.radians(24):
                x += 0.016 * (1 - d / math.radians(24)) ** 1.5 * max(0.0, min(1.0, (0.345 - z) / 0.05))
        return (x + dxz, y, z)

    def bucket(u, v):
        return at(2 * math.pi * u, Z[round(v * (len(Z) - 1))])

    def t_slot(i, j):
        thc = ((i + 0.5) * 360.0 / NU + 180.0) % 360.0 - 180.0
        return (j == eye_row and abs(thc) < 62.0) or (j in nose_rows and abs(thc) < 7.0)

    P.plate("helm", bucket, NU, len(Z) - 1, M, face="steel", rim="steel_dark", thick=0.018, bev=0.007, roll=0.011,
            cyclic=True, cap="v1", inside=(0, 0, 0.19), skip=t_slot,
            creases=lambda a, b: abs(a.y) < 1e-4 and abs(b.y) < 1e-4 and a.x > 0 and b.x > 0 and min(a.z, b.z) > 0.23)
    P.ell("helm_glow", (0.092, 0.086, 0.09), (0.012, 0.0, 0.185), M, "visor", hull=0.0)
    # bell rim at the bottom: a flared band with a gold roll
    P.plate("helm_bell", lambda u, v: at(2 * math.pi * u, 0.03 + 0.05 * v, grow=0.012 * (1 - v) + 0.003),
            NU, 2, M, face="steel", rim="gold", thick=0.014, bev=0.006, roll=0.013, cyclic=True, inside=(0, 0, 0.05))
    # brow band: gold, proud, over the eye slit, following the bucket
    zb0, zb1 = Z[eye_row + 1] - 0.003, Z[eye_row + 1] + 0.036
    P.plate("helm_brow", lambda u, v: at(math.radians(-68 + 136 * u), zb0 + (zb1 - zb0) * v, grow=0.010),
            16, 2, M, face="gold", rim="gold", edge="gold_hi", thick=0.012, bev=0.005, segs=2, roll=0.008,
            inside=(0, 0, 0.26))
    gx = at(0.0, (zb0 + zb1) / 2, grow=0.022)[0]
    P.ell("helm_gem", (0.012, 0.018, 0.02), (gx, 0.0, (zb0 + zb1) / 2), M, "visor", hull=0.4)
    # side roundels (ear discs) with a centre rivet
    rv = []
    for sgn in (-1, 1):
        rot = Matrix.Rotation(sgn * -math.pi / 2, 3, "X")
        base = at(sgn * math.pi / 2, 0.19, grow=0.002)
        P.plate(f"helm_roundel_{sgn}", disc_plate("", 0.046, 0.014, 10, 88, rot=rot, offs=(0.0, base[1], 0.19)),
                16, 3, M, face="gold", rim="gold", edge="gold_hi", thick=0.010, bev=0.004, segs=2,
                cyclic=True, cap="v0", inside=(0.0, 0.0, 0.19), hull=0.8)
        rv.append((0.0, base[1] + sgn * 0.016, 0.19))
    # crest fin: swept from the brow over the crown to the back
    def crest(u, v):
        ps = math.radians(26 + 136 * u)
        rise = 0.012 + BRUSH["fin"] * math.sin(math.pi * min(1.0, u * 1.12)) ** 1.3
        ps2 = ps + math.radians(10) * math.sin(math.pi * u)
        bx, bz = 0.985 * 0.113 * math.cos(ps), 0.215 + 0.985 * 0.128 * math.sin(ps)
        tx, tz = (0.113 + rise) * math.cos(ps2), 0.215 + (0.128 + rise) * math.sin(ps2)
        return (bx + (tx - bx) * v - 0.004, 0.0, bz + (tz - bz) * v)
    P.plate("helm_crest", crest, 18, 2, M, face="steel", rim="steel_dark", edge="steel_hi", thick=0.018, bev=0.005,
            segs=2, level=1, offset=0.0, roll=0.009, inside=(0, -1, 0.25))
    if cloth:
        # r5: the class-colour horsehair crest rides along the fin's top, swept back in tufts (the v3 plume hung
        # back and down behind the helm and went through the floor when the figure lay down)
        B = BRUSH
        u0, u1 = B["u"]

        def fin_top(u):
            ps = math.radians(26 + 136 * u)
            rise = 0.012 + B["fin"] * math.sin(math.pi * min(1.0, u * 1.12)) ** 1.3
            ps2 = ps + math.radians(10) * math.sin(math.pi * u)
            return ps2, ((0.113 + rise) * math.cos(ps2) - 0.004, 0.215 + (0.128 + rise) * math.sin(ps2))

        def brush(uu, v):
            u = u0 + (u1 - u0) * uu
            ps2, (fx, fz) = fin_top(u)
            n = (math.cos(ps2), math.sin(ps2))
            t = (-math.sin(ps2), math.cos(ps2))
            env = math.sin(math.pi * uu ** B["front"]) ** 0.7
            tuft = 1.0 - B["tuft_depth"] * abs(math.sin(math.pi * B["tufts"] * uu)) ** 0.8
            h = B["h"] * env * (tuft if v > 0.5 else 1.0)
            k = -B["seat"] + (h + B["seat"]) * v
            sw = B["sweep"] * h * v * v
            return (fx + n[0] * k + t[0] * sw, 0.0, fz + n[1] * k + t[1] * sw)
        P.plate("helm_brush", brush, 44, 2, M, face="cloth", rim="cloth", edge="cloth", thick=B["thick"],
                bev=0.009, segs=2, level=2, offset=0.0, inside=(0, -1, 0.25))
    rv += [at(math.radians(a), 0.105, grow=0.003) for a in (-150, -120, -90, 90, 120, 150)]
    P.rivets("helm_rivets", rv, M, r=0.0075)
    return P


def build_chest(M):
    """Cuirass: a full barrel with a creased keel and spine, swelling chest, sloped shoulders, a gold-rolled
    collar (gorget), a raised rune medallion front and back, rivets."""
    P = Part("chest")
    fl = WAIST["flare"]                 # the cuirass's lower edge flares a little over the tucked lame
    rf = [(0, 0.118 + fl), (0.12, 0.124), (0.35, 0.138), (0.62, 0.150), (0.85, 0.128), (1, 0.080)]
    rb = [(0, 0.104 + fl), (0.12, 0.106), (0.45, 0.112), (0.7, 0.118), (0.9, 0.100), (1, 0.074)]
    ry = [(0, 0.140 + fl), (0.12, 0.146), (0.35, 0.160), (0.62, 0.188), (0.82, 0.186), (0.93, 0.156), (1, 0.100)]
    dx = [(0, 0.0), (1, 0.0)]
    kb = keel(26, 0.022, 0.0, 0.62)
    sb = keel(18, 0.010, 0.05, 0.9, center=math.pi)

    def pec(t, th):
        if math.cos(th) < 0:
            return 0.0
        return 0.016 * math.exp(-((abs(th) - 0.62) / 0.33) ** 2) * math.exp(-((t - 0.60) / 0.17) ** 2)
    f = tube(-0.025, 0.30, rf, rb, ry, dx, e=2.3, bump=lambda t, th: kb(t, th) + sb(t, th) + pec(t, th))
    P.plate("cuirass", f, 28, 8, M, thick=0.02, bev=0.008, roll=0.014, cyclic=True, inside=(0, 0, 0.15),
            creases=lambda a, b: abs(a.y) < 1e-4 and abs(b.y) < 1e-4)
    # gorget: a short flared collar standing on the cuirass
    P.plate("gorget", tube(0.265, 0.335, [(0, 0.098), (1, 0.088)], [(0, 0.092), (1, 0.084)],
                           [(0, 0.104), (1, 0.094)], e=2.2),
            24, 2, M, thick=0.014, bev=0.006, roll=0.012, cyclic=True, inside=(0, 0, 0.3))
    # rune medallions (front on the keel, back on the spine): a raised dark-steel oval in a gold roll that
    # follows the cuirass surface
    z0c, zspan = -0.025, 0.325
    for nm, thc, sgn in (("front", 0.0, 1), ("back", math.pi, -1)):
        tc = (0.175 - z0c) / zspan

        def med(u, v, thc=thc):
            a2 = 2 * math.pi * u
            r = 0.12 + 0.88 * v
            th = thc + math.radians(21) * r * math.sin(a2)
            t = tc + (0.074 / zspan) * r * math.cos(a2)
            p = Vector(f(th / (2 * math.pi) % 1.0, t))
            n = Vector((math.cos(th), math.sin(th), 0.0))
            return tuple(p + n * (0.011 + 0.010 * (1 - r * r)))
        P.plate(f"medallion_{nm}", med, 20, 3, M, face="steel_dark", rim="gold", edge="gold_hi", thick=0.012,
                bev=0.005, segs=2, roll=0.011, cyclic=True, cap="v0", inside=(0, 0, 0.175))
        P.runes.append(("dor", (0, 0, 0.175), (sgn, 0, 0), 0.105))
    pts = [on_tube(f, a / 360.0, 0.06, 0.004) for a in (300, 325, 35, 60)]
    pts += [on_tube(f, a / 360.0, 0.72, 0.004) for a in (72, 108, 252, 288)]
    P.rivets("chest_rivets", pts, M, r=0.008)
    return P


def lame_a_bottom(th):
    """Bottom edge (m, abdomen frame) of abdomen lame a at angle th: 0.105, arched up by HIP5['arch'] in front."""
    dep, flat, zero = HIP5["arch"]
    d = abs(math.degrees((th + math.pi) % (2 * math.pi) - math.pi))
    x = min(1.0, max(0.0, (zero - d) / (zero - flat)))
    return 0.105 + dep * x * x * (3 - 2 * x)


LAME_A = {"z": (0.105, 0.235), "rf": [(0, 0.118), (0.7, 0.110), (1, 0.090)],
          "rb": [(0, 0.106), (0.7, 0.100), (1, 0.078)], "ry": [(0, 0.146), (0.7, 0.134), (1, 0.114)]}


def lame_a_radii(z):
    """Lame a's (front, back, side) outer radii at height z (m, abdomen frame), from its absolute profile."""
    z0, z1 = LAME_A["z"]
    t = (z - z0) / (z1 - z0)
    return lerp_prof(LAME_A["rf"], t), lerp_prof(LAME_A["rb"], t), lerp_prof(LAME_A["ry"], t)


def build_abdomen(M):
    """Abdomen lame a tucked roof-tile style under the cuirass, its front edge arched over the thighs (the fauld
    covers below it and rides up in a squat) and a leather belt with a gold buckle at the hips; the culet over the
    seat is its own part (build_culet)."""
    P = Part("abdomen")
    W = WAIST
    z0, z1 = LAME_A["z"][0], LAME_A["z"][1] - 0.02 + W["tuck"]
    kb = keel(20, 0.008, 0.0, W["keel_end"])

    def lame_a(u, v):
        th = 2 * math.pi * u
        zb = lame_a_bottom(th)
        z = zb + (z1 - zb) * v
        rf, rb, ry = lame_a_radii(z)
        tt = (z - z0) / (z1 - z0)
        x, y = section(th, rf, rb, ry, 2.3)
        b = kb(tt, th)
        return (x + b * math.cos(th), y + b * math.sin(th), z)
    P.plate("ab_lame_a", lame_a, 32, 3, M, thick=0.016, bev=0.006, roll=0.011, cyclic=True, inside=(0, 0, 0.19))
    bt0, bt1, bth0, bth1 = HIP5["belt"]           # the belt is narrow at the buckle and broad over the hips

    def belt_trim(th):
        d = abs(math.degrees((th + math.pi) % (2 * math.pi) - math.pi))
        x = min(1.0, max(0.0, (d - bth0) / (bth1 - bth0)))
        return (bt1 - bt0) * (1.0 - x * x * (3 - 2 * x))
    P.plate("belt", tube(-0.005, bt1, [(0, 0.140), (1, 0.136)], [(0, 0.124), (1, 0.12)],
                         [(0, 0.176), (1, 0.170)], e=2.3, cut1=belt_trim),
            28, 3, M, face="leather", rim="leather", edge="leather", thick=0.016, bev=0.005, segs=2, cyclic=True,
            inside=(0, 0, 0.03))
    bw = HIP5["buckle_w"]
    P.plate("buckle", lambda u, v: (0.155 + 0.008 * math.sin(math.pi * u), -bw + 2 * bw * u, -0.012 + 0.074 * v),
            4, 3, M, face="gold", rim="gold", edge="gold_hi", thick=0.014, bev=0.005, segs=2,
            inside=(0, 0, 0.02), hull=0.8)
    P.runes.append(("hald", (0.172, 0, 0.025), (1, 0, 0), 0.058))
    px, pz = HIP5["pivot"]                     # the fauld's pivot bosses, on the belt over the hips
    bosses = []
    for sgn in (-1, 1):
        th = math.atan2(sgn * 0.17, px)
        x, y = section(th, 0.14, 0.122, 0.173, 2.3)
        k = 1.0 + 0.009 / math.hypot(x, y)
        bosses.append((x * k, y * k, pz))
    P.rivets("belt_pivots", bosses, M, r=0.013)
    return P


CULET_PIVOT = (-0.13, 0.0)      # the culet's hinge on the belt's back (x, z m in the abdomen frame)
CULET_FOLLOW, CULET_MAX = 0.6, 60.0   # the culet swings this share of the hip flexion toward the thighs, at most deg


def build_culet(M):
    """The culet: two lames over the seat (FAULD_ARC, the back only; the sides belong to the tassets), riveted to the
    belt's back. Its own part, hung from the belt (CULET_PIVOT) in the abdomen frame: place() swings it toward the
    thighs as the hips flex, as a real culet lies over the seat, so a deep hinge no longer points it straight back
    from the hips like a tail (2026-09-30)."""
    P = Part("culet")
    a0, a1 = FAULD_ARC
    for k, (z0, z1, r0, r1) in enumerate(((-0.004, -0.075, 0.176, 0.192), (-0.058, -0.13, 0.186, 0.204))):
        f = tube(z0, z1, [(0, r0 * 0.80), (1, r1 * 0.80)], [(0, r0 * 0.74), (1, r1 * 0.76)],
                 [(0, r0), (1, r1)], e=2.3, th0=math.radians(a0), th1=math.radians(a1))
        P.plate(f"fauld_{k}", f, max(6, int(22 * (a1 - a0) / 284)), 2, M, thick=0.016, bev=0.006, roll=0.011,
                inside=(0, 0, (z0 + z1) / 2))
    P.rivets("fauld_rivets", [(section(math.radians(a), 0.143, 0.127, 0.179, 2.3)[0],
                               section(math.radians(a), 0.143, 0.127, 0.179, 2.3)[1], 0.022)
                              for a in (-130, 130, 180)], M, r=0.0075)
    return P


def culet_matrix(Mab, flex):
    """The culet's world matrix: the abdomen frame turned about the belt's back by CULET_FOLLOW of the hip flexion
    (at most CULET_MAX), its lower edge toward the thighs."""
    px, pz = CULET_PIVOT
    beta = math.radians(min(CULET_MAX, CULET_FOLLOW * flex))
    return (Mab @ Matrix.Translation((px, 0.0, pz)) @ Matrix.Rotation(-beta, 4, "Y")
            @ Matrix.Translation((-px, 0.0, -pz)))


FAULD = {"bottom": (0.120, 0.104, 0.154), "belt_gap": 0.014, "a_gap": 0.041, "flank_gap": 0.03,
         "flank_from": 30.0}
# the fauld's bottom sits inside the belt's top edge (the belt's inner radii front/back/side, less its own rolled
# edge and 3 mm), its top inside lame a (lame a's outer radius less its plate, both rolled edges and 3 mm)


def fauld_bottom(th):
    """The fauld's lower edge: 1 cm inside the belt's top edge (narrow at the buckle, broad over the hips)."""
    bt0, bt1, bth0, bth1 = HIP5["belt"]
    d = abs(math.degrees((th + math.pi) % (2 * math.pi) - math.pi))
    x = min(1.0, max(0.0, (d - bth0) / (bth1 - bth0)))
    return bt0 + (bt1 - bt0) * x * x * (3 - 2 * x) - 0.01


def fauld_section(th, v):
    """Point (x, y, z) of the fauld lame at angle th, height fraction v (0 = its lower edge, 1 = its top): tapering
    from inside the belt to inside lame a, and wherever it overlaps lame a's edge (the rolled edge included) never
    wider than inside lame a."""
    zb, zt = fauld_bottom(th), lame_a_bottom(th) + HIP5["tuck"]
    z = zb + (zt - zb) * v
    g = FAULD["belt_gap"]
    b = [r - g for r in FAULD["bottom"]]
    t = [r - FAULD["a_gap"] for r in lame_a_radii(zt)]
    w = v * v * (3 - 2 * v)
    r = [p + (q - p) * w for p, q in zip(b, t)]
    if z > lame_a_bottom(th) - 0.016:
        r = [min(p, q - FAULD["a_gap"]) for p, q in zip(r, lame_a_radii(max(z, LAME_A["z"][0])))]
    x, y = section(th, r[0], r[1], r[2], 2.3)
    cap = FAULD.get("flank_gap")          # the fauld's sides ride up into lame a's flanks: never wider than inside
    if cap is not None:
        d = abs(math.degrees((th + math.pi) % (2 * math.pi) - math.pi))
        if d > FAULD["flank_from"]:
            za = lame_a_bottom(th)
            ra = lame_a_radii(za)
            xa, ya = section(th, ra[0], ra[1], ra[2], 2.3)
            lim = math.hypot(xa, ya) - cap
            w = min(1.0, (d - FAULD["flank_from"]) / 15.0)
            rr = math.hypot(x, y)
            if rr > lim:
                k = 1.0 + (lim / rr - 1.0) * w
                x, y = x * k, y * k
    return x, y, z


def build_fauld(M):
    """The fauld: one lame across the front and sides between the belt and lame a's arched edge, tucked under both,
    riveted to the hips with gold bosses on its pivot (HIP5['pivot']). Part space = the abdomen frame; place()
    turns it about the pivot's lateral axis so its lower edge rides over the rising thighs."""
    P = Part("fauld")
    arc = math.radians(HIP5["arc"])
    kb = keel(18, 0.006, 0.2, 0.75)

    def f(u, v):
        th = -arc + 2 * arc * u
        x, y, z = fauld_section(th, v)
        b = kb(v, th)
        return (x + b * math.cos(th), y + b * math.sin(th), z)
    P.plate("fauld_lame", f, 26, 4, M, thick=0.016, bev=0.006, roll=0.011, inside=(0, 0, 0.11))
    P.rivets("fauld_rivets", [on_tube(f, u, 0.5, 0.004) for u in (0.3, 0.7)], M, r=0.0075)
    return P


def build_pauldron(side, M, R=0.112, H=0.085, tilt=26.0, shift=0.016, drop=0.036, lames=1):
    """Big domed cop with a fluted keel, two flaring lames tucked under it, rolled gold edges, rivet rows and a
    gold boss carrying a rune; outer side -Y (near) / +Y (far)."""
    P = Part(f"pauldron_{side}")
    o = -1.0 if side == "near" else 1.0
    rot = Matrix.Rotation(-o * math.radians(tilt), 4, "X")
    kr = R / 0.19
    kh = H / 0.13

    def T(p):
        return tuple(rot @ Vector((p[0], o * p[1], p[2])) + Vector((0.0, o * shift, -drop)))

    def cop(u, v):
        th = 2 * math.pi * u
        ph = math.radians(10 + 90 * v)
        flare = 1.0 + 0.07 * max(0.0, (v - 0.75) / 0.25) ** 2
        r = R * math.sin(ph) * flare
        ridge = 0.012 * max(0.0, 1 - abs(math.sin(th)) / 0.35) * math.sin(math.pi * min(1.0, v * 1.1))
        return T((1.1 * (r + ridge) * math.cos(th), r * math.sin(th), H * math.cos(ph) + ridge * 0.8))
    P.plate(f"pd_cop_{side}", cop, 28, 7, M, thick=0.022, bev=0.008, roll=0.017, cyclic=True, cap="v0",
            inside=T((0, 0, 0.02)))
    ridge = [cop(0.0, 1.0 - i / 12.0) for i in range(12)] + [cop(0.5, i / 12.0) for i in range(13)]
    ridge = [tuple(Vector(q) + (Vector(q) - Vector(T((0, 0, -0.03)))).normalized() * 0.002) for q in ridge[1:-1]]
    P.add(sweep(f"pd_ridge_{side}", ridge, 0.0095, M["gold"]), hull=0.5)
    a0, a1 = math.radians(-55), math.radians(235)
    z_top = H * math.cos(math.radians(100)) + 0.03
    rivets = []
    for k in range(lames):
        zt, h = z_top - 0.047 * kh * k, 0.066 * kh
        r0 = (0.158 + 0.009 * k) * kr * 1.02
        r1 = r0 + 0.05 * kr

        def band(u, v, zt=zt, h=h, r0=r0, r1=r1):
            th = a0 + (a1 - a0) * u
            r = r0 + (r1 - r0) * v
            return (1.1 * r * math.cos(th), r * math.sin(th), zt - h * v)
        P.plate(f"pd_lame{k}_{side}", lambda u, v, b=band: T(b(u, v)), 22, 2, M, thick=0.018, bev=0.007,
                roll=0.013, inside=T((0, 0, zt - h / 2)))
        for i in range(7):
            th = a0 + (a1 - a0) * (0.08 + 0.84 * i / 6)
            x, y, z = band(0.08 + 0.84 * i / 6, 0.62)
            rivets.append(T((x + 0.006 * math.cos(th), y + 0.006 * math.sin(th), z)))
    P.rivets(f"pd_rivets_{side}", rivets, M, r=0.0085)
    # boss: a gold disc plate on the crown
    bot = Matrix.Translation(Vector(T((0, 0, H + 0.004)))) @ rot
    P.plate(f"pd_boss_{side}", disc_plate("", 0.05 * kr / 0.83, 0.016 * kh, 0, 88, rot=bot.to_3x3(),
                                          offs=tuple(bot.translation)),
            16, 3, M, face="gold", rim="gold", edge="gold_hi", thick=0.012, bev=0.004, segs=2, cyclic=True, cap="v0",
            inside=tuple(bot.translation), hull=0.8)
    P.runes.append(("sul", T((0, 0, 0.02)), T((0.0, 0.8, 0.55)), 0.066))
    return P


def build_upper_arm(side, M):
    P = Part(f"upper_arm_{side}")
    o = -1.0 if side == "near" else 1.0
    P.plate(f"rerebrace_{side}", tube(-0.02, -0.228, [(0, 0.064), (0.6, 0.063), (1, 0.058)],
                                      [(0, 0.062), (1, 0.056)], [(0, 0.064), (0.6, 0.064), (1, 0.059)],
                                      e=2.1, bump=keel(22, 0.007, 0.1, 0.95, center=o * math.pi / 2),
                                      cut1=_bend(ELBOW_BEND["rerebrace"], o)),
            20, 5, M, thick=0.016, bev=0.006, roll=0.011, cyclic=True, inside=(0, 0, -0.12))
    ke = ELBOW_TUBE["ua_end"]
    P.plate(f"ua_lame_{side}", tube(-0.212, -0.262, [(0, 0.066), (1, 0.064 * ke)], [(0, 0.064), (1, 0.062 * ke)],
                                    [(0, 0.068), (1, 0.066 * ke)], e=2.1, cut1=_bend(ELBOW_BEND["ua_lame"], o)),
            20, 1, M, thick=0.014, bev=0.006, roll=0.010, cyclic=True, inside=(0, 0, -0.24))
    P.runes.append(("tal", (0, 0, -0.15), (math.cos(math.radians(60)), o * math.sin(math.radians(60)), 0), 0.078))
    P.runes.append(("tal", (0, 0, -0.15), (math.cos(math.radians(150)), o * math.sin(math.radians(150)), 0), 0.078))
    return P


def build_elbow(side, M, ua=None, fa=None):
    """Couter: a cop over the point of the elbow (-X) and a cupped heart-shaped fan on the outer side. The cop is
    an oval (ELBOW: radius R, height H, its extent along the arm scaled by kz, centre cx) so its rim stays over the
    tube ends; with the finished upper arm and forearm (ua, fa) any point still inside a tube (straight to fully
    bent) is pushed out over it, so the rims ride over the rerebrace and vambrace instead of cutting into them."""
    E = ELBOW
    P = Part(f"elbow_{side}")
    o = -1.0 if side == "near" else 1.0
    rot = Matrix.Diagonal((1.0, 1.0, E["kz"])) @ Matrix.Rotation(-math.pi / 2, 3, "Y")   # dome axis +Z -> -X
    cop = disc_plate("", E["R"], E["H"], 10, E["polar1"], rot=rot, offs=(E["cx"], 0, 0))
    fs = E["fan_scale"]

    def fan(u, v):
        th = 2 * math.pi * u
        r = 0.064 * fs * (0.1 + 0.9 * v) * (1.0 + 0.28 * math.cos(th) ** 2 - 0.12 * math.cos(th))
        x = -0.03 + r * math.cos(th)
        z = r * math.sin(th) * 1.1 * E["fan_kz"]
        cup = 0.022 * (1 - (0.1 + 0.9 * v) ** 2)
        return (x, o * (E["fan_y"] + cup), z)
    if E["clear"] and ua is not None and fa is not None:
        L = SEG["upper_arm"]
        req_f = joint_grid([(ua, lambda a: _rot_y(a / 2, L)), (fa, lambda a: _rot_y(-a / 2))], E["alphas"])
        # front views bend the elbow sideways (about the arm's X): add those poses to the clearance
        req_s = joint_grid([(ua, lambda a: Matrix.Rotation(a / 2, 4, "X") @ Matrix.Translation((0, 0, L))),
                            (fa, lambda a: Matrix.Rotation(-a / 2, 4, "X"))], (-25, -12, 12, 25))

        def req(z, phi):
            return max(req_f(z, phi), req_s(z, phi))
        if E.get("clear_cop", True):
            cop = cleared(cop, req, 0.016 + JOINT_CLEAR, rim_extra=0.004, rim_v=1.0)
        if E.get("clear_fan", True):
            fan = cleared(fan, req, 0.014 + JOINT_CLEAR, rim_extra=0.004, rim_v=1.0)
    P.plate(f"couter_{side}", cop, 20, 4, M,
            thick=0.016, bev=0.006, roll=0.012, cyclic=True, cap="v0", inside=(0.03, 0, 0))
    P.plate(f"couter_fan_{side}", fan, 20, 3, M, thick=0.014, bev=0.006, roll=0.012, cyclic=True, cap="v0",
            inside=(-0.03, 0, 0))
    ax = Vector(cop(0.0, 0.0))
    fc = Vector(fan(0.0, 0.0))
    P.rivets(f"couter_rivet_{side}", [(fc.x, fc.y + o * 0.006, fc.z), (ax.x - 0.006, 0.0, 0.0)], M, r=0.011)
    P.glows.append(("elbow", (0.0, 0.0, 0.0), 0.042))
    return P


def build_forearm(side, M):
    """Vambrace with a front ridge and a flared bell cuff (the gauntlet cuff) at the wrist end."""
    P = Part(f"forearm_{side}")
    o = -1.0 if side == "near" else 1.0
    kt = ELBOW_TUBE["vamb_top"]
    P.plate(f"vambrace_{side}", tube(-0.03, -0.19, [(0, 0.056 * kt), (0.35, 0.061), (1, 0.052)],
                                     [(0, 0.056 * kt), (0.35, 0.060), (1, 0.050)], [(0, 0.057 * kt), (0.35, 0.061),
                                                                                     (1, 0.052)],
                                     e=2.1, bump=keel(18, 0.008, ELBOW_TUBE["vamb_keel0"], 1.0),
                                     cut0=_bend(ELBOW_BEND["vambrace"], o)),
            20, 5, M, thick=0.015, bev=0.006, roll=0.010, cyclic=True, inside=(0, 0, -0.1),
            creases=lambda a, b: abs(a.y) < 1e-4 and abs(b.y) < 1e-4 and a.x > 0 and b.x > 0)

    def cuff(u, v):
        th = 2 * math.pi * u
        point = max(0.0, o * math.sin(th)) ** 3
        z0 = -0.278 + 0.10 + 0.05 * point
        z1 = -0.278 + _cut(WRIST_CUT["back"], th)      # trimmed where the hand bends
        z1 += _cut((WRIST_CUT["thumb"][0], -o * 90.0, WRIST_CUT["thumb"][1]), th)
        z1 += _cut((WRIST_CUT["little"][0], o * 90.0, WRIST_CUT["little"][1]), th)
        hd, hcen, hp, hf = WRIST_CUT["heel"]
        hc = _cut([(hd, hcen - hp, hf), (hd, hcen + hp, hf), (hd, hcen, hp + 1.0)], th)
        z1 += hc
        z = z0 + (z1 - z0) * v
        r = 0.055 + 0.028 * (v * (1.0 - hc / 0.1)) ** 1.4    # the heel cut takes the flare with it
        return (1.08 * r * math.cos(th), r * math.sin(th), z)
    P.plate(f"cuff_{side}", cuff, 24, 3, M, thick=0.016, bev=0.006, roll=0.014, cyclic=True,
            inside=(0, 0, -0.23))
    rv = []
    for a in range(0, 360, 45):
        th = math.radians(a)
        x, y, z = cuff(a / 360.0, 0.55)
        r = 0.055 + 0.028 * 0.55 ** 1.4
        rv.append((x * (r + 0.004) / r, y * (r + 0.004) / r, z - 0.004))
    P.rivets(f"cuff_rivets_{side}", rv, M, r=0.0075)
    P.runes.append(("orn", (0, 0, -0.105), (math.cos(math.radians(62)), o * math.sin(math.radians(62)), 0), 0.078))
    P.runes.append(("orn", (0, 0, -0.105), (math.cos(math.radians(150)), o * math.sin(math.radians(150)), 0), 0.078))
    return P


def build_hand(side, M):
    """Gauntlet: leather palm and fingers, a domed back plate (+X) with a gold knuckle roll and rivets, three
    finger lames that wrap over the fingers roof-tile style, and a plated thumb on the inner side."""
    P = Part(f"hand_{side}")
    o = -1.0 if side == "near" else 1.0
    P.blob(f"glove_{side}", (0.044, 0.092, 0.10), (-0.006, 0.0, -0.047), M, "leather", bevel=0.018)
    P.blob(f"glove_fingers_{side}", (0.036, 0.088, 0.084), (-0.008, 0.0, -0.126), M, "leather", bevel=0.015)

    def back(u, v):
        th = math.radians(-80 + 160 * u)
        xc, rx, ry = -0.004, 0.036 + 0.006 * math.sin(math.pi * v), 0.052
        return (xc + rx * math.cos(th), ry * math.sin(th), 0.016 - 0.104 * v)
    P.plate(f"gl_back_{side}", back, 10, 4, M, thick=0.014, bev=0.006, roll=0.010, inside=(-0.05, 0, -0.04))
    kn = tube(-0.086, -0.100, [(0, 0.041), (1, 0.041)], [(0, 0.03), (1, 0.03)], [(0, 0.056), (1, 0.056)],
              dx=[(0, -0.004), (1, -0.004)], e=2.4, th0=math.radians(-80), th1=math.radians(80))
    P.plate(f"gl_knuckle_{side}", kn, 10, 1, M, face="gold", rim="gold", edge="gold_hi", thick=0.012, bev=0.005,
            segs=2, roll=0.0, inside=(-0.05, 0, -0.093), hull=0.7)
    P.rivets(f"gl_knuckles_{side}", [on_tube(kn, u, 0.5, 0.004) for u in (0.2, 0.4, 0.6, 0.8)], M, r=0.0085)
    for k in range(3):
        z0 = -0.100 - 0.023 * k

        def lame(u, v, z0=z0, k=k):
            th = math.radians(-138 + 276 * u)
            rx = 0.030 - 0.002 * k - 0.004 * v
            ry = 0.050 - 0.002 * k
            return (-0.007 - 0.003 * k + rx * math.cos(th), ry * math.sin(th), z0 - 0.029 * v)
        P.plate(f"gl_finger{k}_{side}", lame, 10, 1, M, thick=0.011, bev=0.005, segs=2, roll=0.0065,
                inside=(-0.05, 0, z0 - 0.015))
    # thumb along the inner side (toward the body midline: -o * Y)
    P.ell(f"gl_thumb_{side}", (0.020, 0.018, 0.046), (-0.008, -o * 0.055, -0.072), M, "leather", hull=0.9)
    th = tube(-0.035, -0.085, [(0, 0.022), (1, 0.02)], [(0, 0.016), (1, 0.014)], [(0, 0.022), (1, 0.02)], e=2.2,
              th0=math.radians(-100), th1=math.radians(100))
    thumb_shell = lambda u, v: (lambda p: (p[0] - 0.006, p[1] - o * 0.055, p[2]))(th(u, v))
    P.plate(f"gl_thumbplate_{side}", thumb_shell, 10, 2, M, thick=0.010, bev=0.004, segs=2, roll=0.006,
            inside=(-0.03, -o * 0.055, -0.06))
    P.glows.append(("wrist", (0.0, 0.0, 0.004), 0.044))
    return P


def build_thigh(side, M):
    """Cuisse (full tube, front ridge) with a flared tasset lame over the hip."""
    P = Part(f"thigh_{side}")
    o = -1.0 if side == "near" else 1.0
    P.plate(f"cuisse_{side}", tube(-0.04, -0.395, [(0, 0.119), (0.5, 0.113), (1, 0.093)],
                                   [(0, 0.106), (0.5, 0.102), (1, 0.080)], [(0, 0.111), (0.5, 0.106), (1, 0.087)],
                                   dx=[(0, 0.004), (0.5, 0.01), (1, 0.004)], e=2.2,
                                   bump=keel(20, 0.010, 0.05, KNEE_KEEL_END),
                                   cut0=[HIP["cuisse_cut0"], _bend(HIP5["inner_cut"], o)],
                                   cut1=KNEE_BACK["cuisse"]),
            HIP5["cuisse_nu"], 6, M, thick=0.018, bev=0.007, roll=0.013, cyclic=True, inside=(0, 0, -0.2),
            creases=lambda a, b: abs(a.y) < 1e-4 and abs(b.y) < 1e-4 and a.x > 0 and b.x > 0)
    P.runes.append(("kor", (0.006, 0, -0.255), (math.cos(math.radians(55)), o * math.sin(math.radians(55)), 0),
                    0.112))
    P.runes.append(("kor", (0.006, 0, -0.255), (math.cos(math.radians(150)), o * math.sin(math.radians(150)), 0),
                    0.112))
    P.glows.append(("hip", (0.0, 0.0, 0.0), 0.05))
    return P


def build_tasset(side, M):
    """Tasset: a flared lame over the front and outer side of the hip. Its own part (thigh frame): place() slides it
    down the thigh in a deep squat (HIP5['slide']) so its top stays clear of the belly."""
    P = Part(f"tasset_{side}")
    o = -1.0 if side == "near" else 1.0
    H = HIP
    t0, t1 = H["tasset_th"]
    g = HIP5["tasset_grow"]                     # clears the cuisse's keel while it slides down
    f = tube(H["tasset_z"][0], H["tasset_z"][1], [(0, 0.136 + g), (1, 0.141 + g)], [(0, 0.126 + g), (1, 0.132 + g)],
             [(0, 0.136 + g), (1, 0.143 + g)], dx=[(0, 0.0), (1, 0.006)], e=2.2, th0=o * math.radians(t0),
             th1=o * math.radians(t1), cut0=H["tasset_cut0"])
    P.plate(f"tasset_lame_{side}", f, 16, 3, M, thick=0.016, bev=0.007, roll=0.012, inside=(0, 0, -0.03))
    tv = [(a - t0) / (t1 - t0) for a in (0, 45, 90)]
    P.rivets(f"tasset_rivets_{side}", [on_tube(f, u, 0.16, 0.003) for u in tv], M, r=0.008)
    ra = math.radians(HIP5["tasset_rune"][0])        # the thigh's rune again: it lights with the thigh and stays
    P.runes.append(("kor", (0.0, 0.0, HIP5["tasset_rune"][1]), (math.cos(ra), o * math.sin(ra), 0.0),   # in view
                    HIP5["tasset_rune"][2]))          # when the tasset slides over the cuisse's rune in a squat
    return P


def build_knee(side, M, th=None, sh=None):
    """Poleyn: a cop over the knee front (+X), a big cupped fan on the outer side, lames above and below (KNEE:
    cop radius R, height H, forward offset cx, extent along the leg kz). With the finished thigh and shin (th, sh)
    any point still inside a tube (straight to deep squat) is pushed out over it, so the poleyn rides over the
    cuisse and greave ends."""
    K = KNEE
    P = Part(f"knee_{side}")
    o = -1.0 if side == "near" else 1.0
    rot = Matrix.Diagonal((1.0, K["ky"], K["kz"])) @ Matrix.Rotation(math.pi / 2, 3, "Y")   # dome axis +Z -> +X
    cop = disc_plate("", K["R"], K["H"], 10, K["polar1"], rot=rot, offs=(K["cx"], 0, 0))

    def fan(u, v):
        th_ = 2 * math.pi * u
        r = 0.07 * K["fan_scale"] * (0.1 + 0.9 * v) * (1.0 + 0.25 * math.cos(th_) ** 2 + 0.1 * math.cos(th_))
        x = 0.012 + r * math.cos(th_)
        z = r * math.sin(th_) * 1.18
        cup = 0.022 * (1 - (0.1 + 0.9 * v) ** 2)
        return (x, o * (K["fan_y"] + cup), z)
    lames = []
    ta, tb = sorted((o * 78.0, -o * K["lame_inner"]))       # the lames reach less far round the inner side
    for k, (zc, rr) in enumerate(((K["lame_z"], K["lame_r"][0]), (-K["lame_z"], K["lame_r"][1]))):
        lames.append((k, zc, tube(zc + 0.024, zc - 0.024, [(0, rr), (1, rr)], [(0, rr * 0.9), (1, rr * 0.9)],
                                  [(0, rr * 0.99), (1, rr * 0.99)], e=2.2, th0=math.radians(ta),
                                  th1=math.radians(tb))))
    if K["clear"] and th is not None and sh is not None:
        L = SEG["thigh"]
        req = joint_grid([(th, lambda a: _rot_y(-a / 2, L)), (sh, lambda a: _rot_y(a / 2))], K["alphas"])
        if K["clear_cop"]:
            cop = cleared(cop, req, 0.018 + JOINT_CLEAR, rim_extra=0.004, rim_v=1.0)
        if K["clear_fan"]:
            fan = cleared(fan, req, 0.016 + JOINT_CLEAR, rim_extra=0.004, rim_v=1.0)
        if K["clear_lames"]:
            lames = [(k, zc, cleared(f, req, 0.014 + JOINT_CLEAR + 0.004)) for k, zc, f in lames]
    P.plate(f"poleyn_{side}", cop, 22, 4, M,
            thick=0.018, bev=0.007, roll=0.013, cyclic=True, cap="v0", inside=(-0.02, 0, 0))
    P.plate(f"poleyn_fan_{side}", fan, 20, 3, M, thick=0.016, bev=0.006, roll=0.013, cyclic=True, cap="v0",
            inside=(0.012, 0, 0))
    for k, zc, f in lames:
        P.plate(f"poleyn_lame{k}_{side}", f, 10, 1, M, thick=0.014, bev=0.006, roll=0.010, inside=(0, 0, zc))
    if K.get("back"):                   # a leather guard behind the knee, in the gap the trimmed tubes leave
        hz, rb, hw = K["back"]
        P.plate(f"poleyn_back_{side}", tube(hz, -hz, [(0, rb), (0.5, rb * 1.06), (1, rb)],
                                           [(0, rb), (0.5, rb * 1.06), (1, rb)],
                                           [(0, rb), (0.5, rb * 1.06), (1, rb)], e=2.0,
                                           th0=math.radians(180 - hw), th1=math.radians(180 + hw)),
                6, 2, M, face="leather", rim="leather", edge="leather", thick=0.010, bev=0.004, segs=2,
                inside=(0, 0, 0), hull=0.8)
    ax = Vector(cop(0.0, 0.0))
    fc = Vector(fan(0.0, 0.0))
    P.rivets(f"poleyn_rivet_{side}", [(fc.x, fc.y + o * 0.006, fc.z), (ax.x + 0.006, 0.0, 0.0)], M, r=0.012)
    P.glows.append(("knee", (0.0, 0.0, 0.0), 0.036))      # v2: (-0.02, 0, 0), r 0.04; now behind the guard
    return P


def build_shin(side, M):
    """Greave: shin keel, calf swell, flared ankle with a gold roll."""
    P = Part(f"shin_{side}")
    o = -1.0 if side == "near" else 1.0
    P.plate(f"greave_{side}", tube(-0.045, -0.395, [(0, 0.086), (0.3, 0.084), (0.8, 0.071), (1, 0.086)],
                                   [(0, 0.080), (0.35, 0.097), (0.8, 0.071), (1, 0.084)],
                                   [(0, 0.082), (0.35, 0.084), (0.8, 0.069), (1, 0.086)], e=2.1,
                                   bump=keel(18, 0.011, 0.04, 0.9), cut0=KNEE_BACK["greave"], cut1=ANKLE_CUT),
            22, 6, M, thick=0.017, bev=0.007, roll=0.013, cyclic=True, inside=(0, 0, -0.2),
            creases=lambda a, b: abs(a.y) < 1e-4 and abs(b.y) < 1e-4 and a.x > 0 and b.x > 0)
    gf = tube(-0.045, -0.395, [(0, 0.086), (0.3, 0.084), (0.8, 0.071), (1, 0.086)],
              [(0, 0.080), (0.35, 0.097), (0.8, 0.071), (1, 0.084)],
              [(0, 0.082), (0.35, 0.084), (0.8, 0.069), (1, 0.086)], e=2.1)
    P.rivets(f"greave_rivets_{side}", [on_tube(gf, a / 360.0, 0.93, 0.003) for a in (300, 60, 120, 240)], M,
             r=0.0075)
    P.runes.append(("rin", (0, 0, -0.2), (math.cos(math.radians(52)), o * math.sin(math.radians(52)), 0), 0.105))
    P.runes.append(("rin", (0, 0, -0.2), (math.cos(math.radians(150)), o * math.sin(math.radians(150)), 0), 0.105))
    return P


def build_foot(side, M):
    """Heavy pointed sabaton: sole, heel cup, three stepped instep lames and a pointed gold toe cap."""
    P = Part(f"foot_{side}")
    P.blob(f"sole_{side}", (0.30, 0.108, 0.022), (0.065, 0.0, SOLE_Z + 0.011), M, "steel_dark", bevel=0.008)
    P.plate(f"heel_{side}", tube(SOLE_Z + 0.012, 0.02, [(0, 0.03), (1, 0.02)], [(0, 0.078), (1, 0.07)],
                                 [(0, 0.058), (1, 0.056)], dx=[(0, 0.0), (1, 0.0)], e=2.2,
                                 th0=math.radians(80), th1=math.radians(280)),
            12, 2, M, thick=0.014, bev=0.006, roll=0.010, inside=(0.02, 0, -0.03))
    spans = ((-0.03, 0.055, 0.030, 0.006), (0.035, 0.105, 0.0, -0.016), (0.085, 0.15, -0.02, -0.034))
    for k, (x0, x1, zt0, zt1) in enumerate(spans):
        def lame(u, v, x0=x0, x1=x1, zt0=zt0, zt1=zt1, k=k):
            th = math.radians(-90 + 180 * u)
            x = x0 + (x1 - x0) * v
            zt = zt0 + (zt1 - zt0) * v
            w = 0.060 - 0.004 * k - 0.004 * v
            h = zt - (SOLE_Z + 0.014)
            return (x, w * math.sin(th), SOLE_Z + 0.014 + h * math.cos(th) ** 0.8)
        P.plate(f"sab_lame{k}_{side}", lame, 12, 2, M, thick=0.014, bev=0.006, roll=0.010,
                inside=((x0 + x1) / 2, 0, SOLE_Z))

    def toe(u, v):
        th = math.radians(-90 + 180 * u)
        x = 0.13 + 0.115 * v
        taper = (1 - v) ** 0.8
        w = 0.05 * taper + 0.002
        h = (0.043 * (1 - v) ** 1.2 + 0.004)
        return (x, w * math.sin(th), SOLE_Z + 0.012 + h * math.cos(th) ** 0.8)
    P.plate(f"sab_toe_{side}", toe, 12, 4, M, face="gold", rim="gold", edge="gold_hi", thick=0.013, bev=0.005,
            segs=2, roll=0.0, inside=(0.17, 0, SOLE_Z))
    P.glows.append(("ankle", (0.0, 0.0, 0.0), 0.045))
    return P


# ================================================================ runes

GLYPHS = {   # original glyphs on a grid x in [-1, 1], y in [0, 3]; strokes are polylines
    "kor": [[(0, 0), (0, 3)], [(0, 0.85), (0.72, 1.6), (0, 2.35), (-0.72, 1.6), (0, 0.85)]],
    "ves": [[(0, 0), (0, 3)], [(-0.8, 2.2), (0, 3.0), (0.8, 2.2)], [(-0.8, 1.2), (0, 2.0), (0.8, 1.2)]],
    "tal": [[(-0.75, 3), (0.75, 3), (-0.55, 1.55), (0.6, 1.55), (-0.7, 0)]],
    "orn": [[(-0.45, 0), (-0.45, 3)], [(-0.45, 3), (0.65, 2.25), (-0.45, 1.5)], [(-0.95, 0), (0.55, 0)]],
    "sul": [[(0, 1.35), (0.62, 1.8), (0.62, 2.55), (0, 3.0), (-0.62, 2.55), (-0.62, 1.8), (0, 1.35)],
            [(0, 1.35), (0, 0.55)], [(-0.7, 0), (0, 0.55), (0.7, 0)]],
    "hald": [[(-0.6, 0), (-0.6, 3)], [(0.6, 0), (0.6, 3)], [(-0.6, 0.95), (0.6, 2.05)], [(0.6, 0.95), (-0.6, 2.05)]],
    "rin": [[(0, 0), (0, 2.2)], [(-0.75, 3), (0, 2.2), (0.75, 3)], [(0, 1.0), (0.72, 1.6)]],
    "dor": [[(0, 0.4), (0.95, 1.5), (0, 2.6), (-0.95, 1.5), (0, 0.4)], [(-1.25, 1.5), (1.25, 1.5)],
            [(0, 2.6), (0, 3.1)], [(0, -0.1), (0, 0.4)]],
}


def _stroke_quads(pts, w):
    """Flat ribbon outline for a polyline in 2D: list of quads (4 corner points), with square caps."""
    quads = []
    for (x0, y0), (x1, y1) in zip(pts, pts[1:]):
        dx, dy = x1 - x0, y1 - y0
        n = math.hypot(dx, dy) or 1.0
        tx, ty = dx / n, dy / n
        nx, ny = -ty * w / 2, tx * w / 2
        ax, ay = x0 - tx * w / 2, y0 - ty * w / 2
        bx, by = x1 + tx * w / 2, y1 + ty * w / 2
        quads.append(((ax + nx, ay + ny), (bx + nx, by + ny), (bx - nx, by - ny), (ax - nx, ay - ny), n + w))
    return quads


def glyph_mesh(bvh, glyph, anchor, direction, height, up=(0, 0, 1), width_k=0.17, border=0.005, lift=0.0012):
    """Rune inlay conformed onto the part surface: a dark groove ribbon (slot 0) and a glowing inlay ribbon on top
    of it (slot 1). anchor/direction: a ray from anchor + direction*0.4 back toward anchor finds the surface."""
    d = Vector(direction).normalized()
    a = Vector(anchor)
    hit, n, _, _ = bvh.ray_cast(a + d * 0.4, -d, 0.6)
    if hit is None:
        return [], [], []
    n = n.normalized()
    upv = Vector(up)
    upv = (upv - n * upv.dot(n)).normalized()
    right = upv.cross(n).normalized()
    s = height / 3.0
    w = height * width_k
    verts, faces, mats = [], [], []

    def project(gx, gy, off):
        p = hit + right * (gx * s) + upv * ((gy - 1.5) * s)
        h2, n2, _, _ = bvh.ray_cast(p + n * 0.05, -n, 0.12)
        if h2 is None:
            return p + n * off
        return h2 + n2.normalized() * off

    for layer, (ww, off) in enumerate(((w + 2 * border, lift), (w, lift * 2.2))):
        for stroke in GLYPHS[glyph]:
            for q in _stroke_quads(stroke, ww / s):
                (p0, p1, p2, p3, ln) = q
                steps = max(1, int(ln * s / 0.012))
                base = len(verts)
                for i in range(steps + 1):
                    t = i / steps
                    for (qa, qb) in ((p0, p1), (p3, p2)):
                        gx = qa[0] + (qb[0] - qa[0]) * t
                        gy = qa[1] + (qb[1] - qa[1]) * t
                        verts.append(project(gx, gy, off))
                for i in range(steps):
                    i0 = base + 2 * i
                    faces.append((i0, i0 + 2, i0 + 3, i0 + 1))
                    mats.append(layer)
    return verts, faces, mats


# ================================================================ freeze and join

def _grad_attr(me, lo=None, hi=None):
    zs = [v.co.z for v in me.vertices]
    if not zs:
        return
    lo = min(zs) if lo is None else lo
    hi = max(zs) if hi is None else hi
    span = max(1e-5, hi - lo)
    at = me.attributes.get("grad") or me.attributes.new("grad", "FLOAT", "POINT")
    at.data.foreach_set("value", [max(0.0, min(1.0, (z - lo) / span)) for z in zs])


_AO_DIRS = [Vector((math.cos(2 * math.pi * (i + 0.5) / 10) * math.cos(math.radians(el)),
                    math.sin(2 * math.pi * (i + 0.5) / 10) * math.cos(math.radians(el)), math.sin(math.radians(el))))
            for i in range(10) for el in (30, 62)]


def bake_cavity(me, dist=0.045):
    """Self-occlusion per vertex ('cav', 0 open .. 1 buried) from 20 fixed hemisphere rays against the part's own
    mesh (BVH), a deterministic stand-in for a Cycles AO bake (that took ~100 s per part here, this ~0.3 s)."""
    bvh = BVHTree.FromPolygons([v.co for v in me.vertices], [p.vertices for p in me.polygons], epsilon=0.0)
    occ = []
    n_dirs = len(_AO_DIRS)
    for v in me.vertices:
        n = v.normal
        q = n.to_track_quat("Z", "Y").to_matrix()
        o = v.co + n * 0.0015
        h = 0
        for d in _AO_DIRS:
            if bvh.ray_cast(o, q @ d, dist)[0] is not None:
                h += 1
        occ.append(h / n_dirs)
    at = me.attributes.get("cav") or me.attributes.new("cav", "FLOAT", "POINT")
    at.data.foreach_set("value", occ)


def finalize(P, coll):
    """Freeze every mesh piece of the part (all modifiers applied), add the 'grad' attribute and hull weights,
    join into one object named P.name (origin = pivot), delete the pieces."""
    bpy.context.view_layer.update()
    dg = bpy.context.evaluated_depsgraph_get()
    pieces = [o for o in P.root.children_recursive if o.type in ("MESH", "CURVE")]
    frozen = []
    for ob in pieces:
        if ob.name not in P.hull and ob.parent is not P.root:
            continue
        ev = ob.evaluated_get(dg)
        me = bpy.data.meshes.new_from_object(ev, preserve_all_data_layers=True, depsgraph=dg)
        if not me.vertices:
            bpy.data.meshes.remove(me)
            continue
        me.transform(ob.matrix_world)
        _grad_attr(me)
        nb = bpy.data.objects.new(ob.name + "_f", me)
        coll.objects.link(nb)
        vg = nb.vertex_groups.new(name="hull")
        vg.add(list(range(len(me.vertices))), P.hull.get(ob.name, 1.0), "REPLACE")
        frozen.append(nb)
    for ob in list(P.root.children_recursive) + [P.root]:
        data = ob.data
        bpy.data.objects.remove(ob, do_unlink=True)
        if data is not None and data.users == 0 and isinstance(data, bpy.types.Mesh):
            bpy.data.meshes.remove(data)
    base = frozen[0]
    with bpy.context.temp_override(active_object=base, object=base, selected_objects=frozen,
                                   selected_editable_objects=frozen):
        bpy.ops.object.join()
    base.name = P.name
    base.data.name = P.name
    bake_cavity(base.data)
    return base


def add_hull(ob, M, px_key):
    ob.data.materials.append(M["outline"])
    sol = ob.modifiers.new("qp_outline", "SOLIDIFY")
    sol.thickness = 0.008
    sol.offset = 1.0
    sol.use_flip_normals = True
    sol.use_rim = False
    sol.use_even_offset = False          # even offset spikes at the sharp plate corners
    sol.use_quality_normals = True
    sol.material_offset = 100
    sol.vertex_group = "hull"
    sol.thickness_vertex_group = 0.0
    ob["qp_outline_px"] = OUTLINE_PX.get(px_key, OUTLINE_PX["default"])
    return sol


def _part_key(name):
    for k in ("pauldron", "head", "chest", "abdomen", "thigh"):
        if name.startswith(k):
            return k
    return {"fauld": "abdomen", "culet": "abdomen", "tasset": "thigh"}.get(name.split("_")[0], "default")


def _add_runes(P, ob, M, coll, parts):
    if not P.runes:
        return
    bvh = BVHTree.FromObject(ob, bpy.context.evaluated_depsgraph_get())
    verts, faces, mats = [], [], []
    for glyph, anchor, direction, height in P.runes:
        v, f, m = glyph_mesh(bvh, glyph, anchor, direction, height)
        base = len(verts)
        verts += v
        faces += [tuple(i + base for i in fc) for fc in f]
        mats += m
    if not faces:
        return
    me = bpy.data.meshes.new("rune_" + P.name)
    me.from_pydata([tuple(v) for v in verts], [], faces)
    me.materials.append(M["groove"])
    me.materials.append(M["rune"])
    me.polygons.foreach_set("material_index", mats)
    at = me.attributes.new("grad", "FLOAT", "POINT")
    at.data.foreach_set("value", [0.5] * len(verts))
    rn = bpy.data.objects.new("rune_" + P.name, me)
    coll.objects.link(rn)
    rn.parent = ob
    rn["glow"] = 0.0
    rn["qp_part"] = P.name
    parts["rune_" + P.name] = rn


def _add_glows(P, ob, M, coll, parts):
    for joint, loc, r in P.glows:
        side = P.name.rsplit("_", 1)[-1]
        name = f"glow_{joint}_{side}"
        bm = bmesh.new()
        bmesh.ops.create_uvsphere(bm, u_segments=20, v_segments=10, radius=r,
                                  matrix=Matrix.Translation(Vector(loc)))
        me = bpy.data.meshes.new(name)
        bm.to_mesh(me)
        bm.free()
        me.shade_smooth()
        me.materials.append(M["joint"])
        g = bpy.data.objects.new(name, me)
        coll.objects.link(g)
        g.parent = ob
        g["glow"] = 0.25
        g["qp_part"] = P.name
        g.visible_shadow = False
        parts[name] = g


# ================================================================ sash (class-colour cloth ribbon, per frame)

SASH_ROWS, SASH_COLS = 28, (-1.0, -0.78, 0.0, 0.78, 1.0)


def build_sash(M, coll):
    nv = (SASH_ROWS + 1) * len(SASH_COLS)
    verts = [(0.05 * SASH_COLS[i % len(SASH_COLS)], 0.0, -0.4 * (i // len(SASH_COLS)) / SASH_ROWS)
             for i in range(nv)]
    faces, mats = [], []
    nc = len(SASH_COLS)
    for r in range(SASH_ROWS):
        for c in range(nc - 1):
            a = r * nc + c
            faces.append((a, a + 1, a + nc + 1, a + nc))
            mats.append(1 if c in (0, nc - 2) else 0)
    me = bpy.data.meshes.new("sash")
    me.from_pydata(verts, [], faces)
    me.materials.append(M["cloth"])
    me.materials.append(M["gold"])
    me.polygons.foreach_set("material_index", mats)
    me.shade_smooth()
    at = me.attributes.new("grad", "FLOAT", "POINT")
    at.data.foreach_set("value", [1.0 - (i // nc) / SASH_ROWS * 0.8 for i in range(nv)])
    ob = bpy.data.objects.new("sash", me)
    coll.objects.link(ob)
    th = ob.modifiers.new("cloth_thickness", "SOLIDIFY")
    th.thickness = 0.007
    th.offset = 0.0
    vg = ob.vertex_groups.new(name="hull")
    vg.add(list(range(nv)), 1.0, "REPLACE")
    add_hull(ob, M, "sash")
    # knot: a cloth bundle in a gold ring, placed at the sash's first point
    bm = bmesh.new()
    bmesh.ops.create_uvsphere(bm, u_segments=16, v_segments=8, radius=1.0,
                              matrix=Matrix.Diagonal((0.034, 0.03, 0.045, 1.0)))
    for v in bm.verts:
        v.co.z += 0.01 * math.sin(v.co.x * 90)
    me2 = bpy.data.meshes.new("sash_knot")
    bm.to_mesh(me2)
    bm.free()
    me2.shade_smooth()
    me2.materials.append(M["cloth"])
    kn = bpy.data.objects.new("sash_knot", me2)
    coll.objects.link(kn)
    at = me2.attributes.new("grad", "FLOAT", "POINT")
    at.data.foreach_set("value", [0.8] * len(me2.vertices))
    ring = pk.ellipsoid("sash_ring", (0.012, 0.036, 0.036), loc=(0, 0, 0), mat=M["gold"], segs=16)
    for c in ring.users_collection:
        c.objects.unlink(ring)
    coll.objects.link(ring)
    ring.parent = kn
    ring.location = (0.0, -0.028, 0.0)
    at = ring.data.attributes.new("grad", "FLOAT", "POINT")
    at.data.foreach_set("value", [0.7] * len(ring.data.vertices))
    return ob, kn


def _catmull(pts, n):
    """Centripetal-ish Catmull-Rom through pts (Vectors): n+1 samples with tangents."""
    P = [pts[0] + (pts[0] - pts[1])] + list(pts) + [pts[-1] + (pts[-1] - pts[-2])]
    seg = len(pts) - 1
    out = []
    for k in range(n + 1):
        s = k / n * seg
        i = min(int(s), seg - 1)
        t = s - i
        p0, p1, p2, p3 = P[i], P[i + 1], P[i + 2], P[i + 3]
        t2, t3 = t * t, t * t * t
        pos = 0.5 * ((2 * p1) + (-p0 + p2) * t + (2 * p0 - 5 * p1 + 4 * p2 - p3) * t2 + (-p0 + 3 * p1 - 3 * p2 + p3) * t3)
        tan = 0.5 * ((-p0 + p2) + 2 * (2 * p0 - 5 * p1 + 4 * p2 - p3) * t + 3 * (-p0 + 3 * p1 - 3 * p2 + p3) * t2)
        out.append((pos, tan.normalized() if tan.length > 1e-8 else Vector((0, 0, -1))))
    return out


def sash_face(t, rest):
    """The ribbon's face at a row: toward the camera (-Y) where the sash hangs, up where it lies on a surface (rest 1,
    motion.sash_rest), square to the row's tangent t; so the cloth on a floor, a bed or a mat lies flat on it instead
    of standing on edge."""
    face = Vector((0, -1, 0)).lerp(Vector((0, 0, 1)), _smooth(rest))
    n = face - t * face.dot(t)
    if n.length < 1e-6:                       # the tangent runs along the face: the other face
        n = Vector((0, 0, 1)) if rest < 0.5 else Vector((0, -1, 0))
        n = n - t * n.dot(t)
    return n.normalized() if n.length > 1e-6 else Vector((0, -1, 0))


def place_sash(parts, points, rest, width=0.095):
    """Bend the sash ribbon through the motion side's sash points (rig units (x, y, z)); the cloth faces the camera
    (-Y) where it hangs and lies flat where it rests on a surface (`rest`, each point's weight 0-1, sash_face; one per
    point). The knot sits at the first point."""
    ob = parts.get("sash")
    kn = parts.get("sash_knot")
    show = bool(ob is not None and points and len(points) >= 2)
    for o in (ob, kn) + tuple(kn.children if kn else ()):
        if o is not None:
            o.hide_render = not show
            o.hide_viewport = not show
    if not show:
        return
    pts = [Vector((p[0] * U, (p[2] if len(p) > 2 else 0.0) * U, p[1] * U)) for p in points]
    if len(rest) != len(points):
        raise ValueError(f"sash: {len(rest)} rest weights for {len(points)} points")
    samples = _catmull(pts, SASH_ROWS)
    me = ob.data
    nc = len(SASH_COLS)
    co = []
    seg = len(points) - 1
    for r, (p, t) in enumerate(samples):
        u = r / SASH_ROWS
        s = u * seg
        i = min(int(s), seg - 1)
        face_n = sash_face(t, rest[i] + (rest[i + 1] - rest[i]) * (s - i))
        wv = t.cross(face_n)
        if wv.length < 1e-6:
            wv = Vector((1, 0, 0))
        wv.normalize()
        half = width / 2 * (1.0 - 0.1 * u)
        if u > 0.8:
            half *= max(0.0, (1.0 - u) / 0.2) ** 0.9
        for cx in SASH_COLS:
            bow = 0.012 * (1 - cx * cx)
            q = p + wv * (cx * half) + face_n * bow
            co.extend(q)
    ob.matrix_world = Matrix.Identity(4)
    me.vertices.foreach_set("co", co)
    me.update()
    kn = parts.get("sash_knot")
    if kn:
        ab = parts.get("abdomen")
        R = ab.matrix_world.to_3x3() if ab else Matrix.Identity(3)
        kn.matrix_world = Matrix.Translation(pts[0]) @ R.to_4x4()


# ================================================================ build

def _collection(name, scene=None):
    scene = scene or bpy.context.scene
    c = bpy.data.collections.get(name)
    if c is None:
        c = bpy.data.collections.new(name)
    if c.name not in scene.collection.children:
        scene.collection.children.link(c)
    return c


def build(class_rgb=(0.55, 0.08, 0.08)):
    """Build the whole suit at rest (every part at the origin in its own frame). Returns name -> object for the
    rigid parts (RIGID), their rune inlays ('rune_<part>', child of the part), joint glows ('glow_<joint>_<side>',
    child of the part), 'sash' and 'sash_knot'."""
    coll = _collection(ARMOR_COLL)
    M = palette(class_rgb)
    builders = [lambda: build_helm(M), lambda: build_chest(M), lambda: build_abdomen(M), lambda: build_culet(M),
                lambda: build_fauld(M)]
    parts = {}
    for side in ("near", "far"):       # the caps last: they are shaped over the finished tubes
        builders += [lambda s=side: build_pauldron(s, M), lambda s=side: build_upper_arm(s, M),
                     lambda s=side: build_forearm(s, M),
                     lambda s=side: build_elbow(s, M, parts[f"upper_arm_{s}"], parts[f"forearm_{s}"]),
                     lambda s=side: build_hand(s, M), lambda s=side: build_thigh(s, M),
                     lambda s=side: build_tasset(s, M),
                     lambda s=side: build_shin(s, M),
                     lambda s=side: build_knee(s, M, parts[f"thigh_{s}"], parts[f"shin_{s}"]),
                     lambda s=side: build_foot(s, M)]
    for b in builders:
        P = b()
        ob = finalize(P, coll)
        if P.name == "head":
            ob.data.transform(Matrix.Scale(HELM_SCALE, 4))
        parts[P.name] = ob
        _add_runes(P, ob, M, coll, parts)
        _add_glows(P, ob, M, coll, parts)
        add_hull(ob, M, _part_key(P.name))
    neck = _glow_ring("glow_neck", 0.082, 0.30, 0.012, M, coll)
    neck.parent = parts["chest"]
    parts["glow_neck"] = neck
    sash, knot = build_sash(M, coll)
    parts["sash"], parts["sash_knot"] = sash, knot
    for o in list(pk_leftovers()):
        bpy.data.objects.remove(o, do_unlink=True)
    for o in coll.all_objects:
        o.hide_render = False
    bpy.context.scene["qp_materials"] = [m.name for m in M.values()]
    return parts


def _glow_ring(name, R, z, r, M, coll):
    bm = bmesh.new()
    segs, rings = 32, 8
    vs = []
    for i in range(segs):
        a = 2 * math.pi * i / segs
        for j in range(rings):
            b = 2 * math.pi * j / rings
            vs.append(bm.verts.new(((R + r * math.cos(b)) * math.cos(a), (R + r * math.cos(b)) * math.sin(a) * 1.1,
                                    z + r * math.sin(b))))
    for i in range(segs):
        for j in range(rings):
            a0 = i * rings + j
            a1 = i * rings + (j + 1) % rings
            b0 = ((i + 1) % segs) * rings + j
            b1 = ((i + 1) % segs) * rings + (j + 1) % rings
            bm.faces.new((vs[a0], vs[b0], vs[b1], vs[a1]))
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me)
    bm.free()
    me.shade_smooth()
    me.materials.append(M["joint"])
    g = bpy.data.objects.new(name, me)
    coll.objects.link(g)
    g["glow"] = 0.25
    g["qp_part"] = "chest"
    g.visible_shadow = False
    return g


def pk_leftovers():
    """Objects plate_kit linked into the scene root that were not adopted (none expected)."""
    return [o for o in bpy.context.scene.collection.objects if o.name.startswith(("tmp_", "gauntlet_finger_step"))]


# ================================================================ glow control

GLOW_NEIGHBOURS = {"thigh": ("hip", "knee"), "shin": ("knee", "ankle"), "upper_arm": ("elbow",),
                   "forearm": ("elbow", "wrist"), "chest": ("neck",), "abdomen": ("neck",), "hand": ("wrist",),
                   "foot": ("ankle",)}


GLOW_ALIASES = {"shoulder": ("pauldron",), "shoulders": ("pauldron",), "core": ("abdomen",), "abs": ("abdomen",),
                "arm": ("upper_arm", "forearm"), "arms": ("upper_arm", "forearm"), "leg": ("thigh", "tasset", "shin"),
                "legs": ("thigh", "tasset", "shin"), "calf": ("shin",), "back": ("chest",),
                "thigh": ("thigh", "tasset"),     # r5: the tassets carry a thigh rune (they slide over the cuisse's)
                "thigh_near": ("thigh_near", "tasset_near"), "thigh_far": ("thigh_far", "tasset_far")}


def _expand(names):
    out = []
    for n in names:
        out += list(GLOW_ALIASES.get(n, (n,)))
    return out


def _matches(part, names):
    return any(part == n or part.startswith(n + "_") for n in names)


def set_glow(parts, names, level):
    """Light the runes of the working parts: every rune on a part named in `names` (a base name like 'thigh'
    matches both sides, 'thigh_near' only one) gets level (0..1), every other rune 0. The joint lights next to a
    working part rise with it (a faint 0.25 at rest)."""
    names = _expand(list(names or []))
    level = max(0.0, min(1.0, float(level)))
    hot_joints = set()
    for n in names:
        base = n.rsplit("_", 1)[0] if n.endswith(("_near", "_far")) else n
        hot_joints.update(GLOW_NEIGHBOURS.get(base, ()))
    for key, ob in parts.items():
        if key.startswith("rune_"):
            ob["glow"] = level if _matches(key[5:], names) else 0.0
            ob.update_tag()
        elif key.startswith("glow_"):
            joint = key.split("_")[1]
            ob["glow"] = 0.25 + 0.45 * level if joint in hot_joints else 0.25
            ob.update_tag()


# ================================================================ posing (the renderer's side of the interface)

def _v(p, depth=0.0):
    z = p[2] if len(p) > 2 else 0.0
    return Vector((p[0] * U, z * U + depth, p[1] * U))


def _basis(x, y, z, t):
    M = Matrix((x, y, z)).transposed().to_4x4()
    M.translation = t
    return M


def frame_axis(a, b, view, up=False, lat=None, turned=1.0):
    """Part frame with its local -Z (or +Z when up) along a->b, front +X. `lat` (world, toward the figure's far side:
    the shoulder line for the chest and the arms, the hip line for the abdomen and the legs) sets the part's local Y,
    so a turned or twisted body turns its plates; without it, side view: local Y = world Y; front view: local X =
    world +Y, the front of the figure facing away, whose only singular direction (a limb along the view) the camera
    looks down. A front-view limb takes the line's frame by `turned` (0..1, body_turned), turned about its own axis
    from the view's: a body facing away keeps the view's frame, one turned off the view axis takes the side view's."""
    d = b - a
    if d.length < 1e-9:
        d = Vector((0, 0, 1 if up else -1))
    d.normalize()
    z = d if up else -d
    x_lat = None
    if lat is not None:
        y = lat - z * lat.dot(z)
        if y.length > 1e-6:
            y.normalize()
            if view != "front" or turned >= 1.0:
                return _basis(y.cross(z), y, z, a)
            x_lat = y.cross(z)
    if view == "front":
        y = z.cross(Vector((0, 1, 0)))
        if y.length < 1e-6:
            y = Vector((-1, 0, 0))
        y.normalize()
        x = y.cross(z)
        if x_lat is not None and turned > 0.0:
            raw = math.atan2(x.cross(x_lat).dot(z), x.dot(x_lat))
            # where the two fronts near opposite ways (past FRONT_OPPOSE), the line's takes over, so the turn never
            # jumps by a share of a whole turn as raw passes from +pi to -pi
            t = turned + (1.0 - turned) * _smooth((abs(raw) - FRONT_OPPOSE) / (math.pi - FRONT_OPPOSE))
            ang = t * raw
            x = x * math.cos(ang) + z.cross(x) * math.sin(ang)
            y = z.cross(x)
    else:
        x = Vector((0, 1, 0)).cross(z)
        if x.length < 1e-6:
            x = Vector((1, 0, 0))
        x.normalize()
        y = z.cross(x)
    return _basis(x, y, z, a)


def frame_foot(ankle, toe, view, side, lat=None):
    """A foot's frame: +X along the foot. Side view: the foot turns about the hips' lateral axis (`lat`, world Y for
    an unturned body) only, so a foot pointing back from the hips lies laces down (a rear foot on a bench) and a figure
    turned about keeps its soles down. Front view: the foot keeps its heading (sideways too) and its pitch (on the
    ball of the foot), its sole square to it; a foot pointing almost straight up or down points forward, 8 degrees
    out. checks.foot_points mirrors both."""
    d = toe - ankle
    if view == "front":
        flat = Vector((d.x, d.y, 0.0))
        if d.length > 1e-9 and flat.length > 0.35 * d.length:
            x = d.normalized()
            y = Vector((0, 0, 1)).cross(flat).normalized()
        else:
            out = 1.0 if d.x > 0 else -1.0
            x = Vector((math.sin(math.radians(8)) * out, math.cos(math.radians(8)), 0.0))
            y = Vector((0, 0, 1)).cross(x).normalized()
        z = x.cross(y)
    else:
        ly = (lat if lat is not None and lat.length > 1e-9 else Vector((0, 1, 0))).normalized()
        x = d - ly * d.dot(ly)
        x = x.normalized() if x.length > 1e-9 else Vector((1, 0, 0))
        z = x.cross(ly).normalized()
        y = z.cross(x)
    return _basis(x, y, z, ankle)


def body_turned(line, front):
    """How far a front view's body has turned off the view axis, 0..1 (FRONT_TURN), for the limbs that hang from
    `line` (the hips' or the shoulders', near to far) under a trunk whose front is `front`: the heading of line x Z,
    which a bow or a fold about the line leaves alone, and the front's own as the line stands up (a body on its side:
    a side plank facing away reads 0, a flag facing the camera 1)."""
    h = line.normalized().cross(Vector((0, 0, 1)))
    h = h + Vector((front.x, front.y, 0.0)) * (1.0 - h.length)
    turn = abs(math.degrees(math.atan2(-h.x, h.y)))
    return _smooth((turn - FRONT_TURN[0]) / (FRONT_TURN[1] - FRONT_TURN[0]))


def _half(M1, M2, t):
    q = M1.to_quaternion().slerp(M2.to_quaternion(), 0.5)
    R = q.to_matrix().to_4x4()
    R.translation = t
    return R


def normalize_joints(joints, move_view="side"):
    """Accept {'pelvis','waist','neck','head','near':{...},'far':{...}}, flat 'elbow_near' keys, or rig.solve's
    {'limbs': [...]} form; derive waist (20/52 up the torso) and hand_tip (17 cm past the wrist) when missing."""
    J = {k: joints[k] for k in ("pelvis", "waist", "neck", "head") if k in joints}
    sides = {"near": {}, "far": {}}
    for side in sides:
        if isinstance(joints.get(side), dict):
            sides[side].update(joints[side])
        for k in ("shoulder", "elbow", "wrist", "hand_tip", "hip", "knee", "ankle", "toe"):
            if f"{k}_{side}" in joints:
                sides[side][k] = joints[f"{k}_{side}"]
    for L in joints.get("limbs", []):
        sides[L["side"]].update({k: v for k, v in L.items() if k != "side"})
    if "waist" not in J:
        p, n = J["pelvis"], J["neck"]
        J["waist"] = tuple(a + (b - a) * 20.0 / 52.0 for a, b in zip(p, n))
    for side, S in sides.items():
        if "hand_tip" not in S and "wrist" in S:
            e, w = S["elbow"], S["wrist"]
            dx, dy = w[0] - e[0], w[1] - e[1]
            n = math.hypot(dx, dy) or 1.0
            S["hand_tip"] = (w[0] + dx / n * 17.0, w[1] + dy / n * 17.0) + tuple(w[2:3])
    J["near"], J["far"] = sides["near"], sides["far"]
    return J


def _smooth(x):
    x = min(1.0, max(0.0, x))
    return x * x * (3 - 2 * x)


def pauldron_matrix(Mch, sh, el, neck, view, pad=None):
    """The pauldron's world matrix: chest-oriented at the shoulder joint and turned by `follow` of the upper arm's
    swing from chest-down (the rotation taking chest-down onto the arm): all of it ('swing', side views) or its part
    about the shoulder's hinge (chest-down x out, the abduction: 'hinge', front views), both read in the chest's frame
    so a folded or turned trunk keeps them. Toward overhead the swing blends into the view's own overhead swing
    (PAD_TOP: about chest-down x front in side views, about the hinge in front views). The pauldron moves by `lift`
    (up the chest), `out` (away from the neck) and `back` (behind the chest, while its front lies along the view axis:
    PAD_BACK_GATE) as that swing grows from a0 to a1 degrees (smoothstep; side views read the arm's own swing)."""
    P = dict(PAD[view if view in PAD else "side"])
    if pad:
        P.update(pad)
    up = Mch.col[2].xyz.normalized()
    fwd = Mch.col[0].xyz.normalized()
    down = -up
    out = sh - neck
    out = out - up * out.dot(up)
    out = out.normalized() if out.length > 1e-6 else Vector((0, 0, 0))
    side = Mch.col[1].xyz.normalized()
    hinge = down.cross(side if (sh - neck).dot(side) >= 0 else -side)
    arm = (el - sh).normalized()
    axis = down.cross(arm)
    ang = math.atan2(axis.length, down.dot(arm))
    swing = axis.normalized() * ang if axis.length > 1e-9 else Vector((0, 0, 0))
    top = hinge if P["axis"] == "hinge" else down.cross(fwd)
    toward = swing.normalized().dot(top) if swing.length > 1e-9 else 1.0
    t0 = PAD_TOP[0] + (PAD_TOP[1] - PAD_TOP[0]) * (1.0 - toward) / 2.0
    w = _smooth((math.degrees(ang) - t0) / (PAD_TOP[2] - t0))
    swing = swing * (1.0 - w) + top * (math.pi * w)
    if P["axis"] == "hinge":
        swing = hinge * swing.dot(hinge)
        a = math.degrees(swing.length)
    else:
        a = math.degrees(ang)
    beta = swing * P["follow"]
    tilt = math.radians(P.get("tilt0", 0.0)) * (1.0 if out.dot(fwd.cross(up)) >= 0 else -1.0)
    R = (Matrix.Rotation(beta.length, 4, beta.normalized() if beta.length > 1e-9 else up)
         @ Matrix.Rotation(tilt, 4, fwd) @ Matrix(Mch.to_3x3()).to_4x4())
    r = _smooth((a - P["a0"]) / (P["a1"] - P["a0"]))
    back = P["back"] * r * _smooth((abs(fwd.y) - PAD_BACK_GATE[0]) / (PAD_BACK_GATE[1] - PAD_BACK_GATE[0]))
    R.translation = (sh + up * (P.get("lift0", 0.0) + P["lift"] * r) + out * (P.get("out0", 0.0) + P["out"] * r)
                     - fwd * back)
    return R


def _seg_dist(p1, q1, p2, q2):
    """Closest points of segments p1q1 and p2q2: (distance, point on the first, point on the second)."""
    d1, d2, r = q1 - p1, q2 - p2, p1 - p2
    a, e, f = d1.dot(d1), d2.dot(d2), d2.dot(r)
    if a < 1e-12 and e < 1e-12:
        return (p1 - p2).length, p1, p2
    if a < 1e-12:
        s, t = 0.0, min(1.0, max(0.0, f / e))
    else:
        c = d1.dot(r)
        if e < 1e-12:
            t, s = 0.0, min(1.0, max(0.0, -c / a))
        else:
            b = d1.dot(d2)
            den = a * e - b * b
            s = min(1.0, max(0.0, (b * f - c * e) / den)) if den > 1e-12 else 0.0
            t = (b * s + f) / e
            if t < 0.0:
                t, s = 0.0, min(1.0, max(0.0, -c / a))
            elif t > 1.0:
                t, s = 1.0, min(1.0, max(0.0, (b - c) / a))
    c1, c2 = p1 + d1 * s, p2 + d2 * t
    return (c1 - c2).length, c1, c2


def arm_clear(sh, el, wr, tip, hp, kn, out, up, slip=None):
    """Clearance rule (one frame's, for arm_turns): a hanging or swinging arm must pass outside its own side's tasset.
    Capsules: the tasset around the thigh's first ARM_CLEAR['tasset_len'] m, the couter (a sphere at the elbow), the
    forearm with its bell cuff and the gauntlet. When they overlap, the whole arm turns outward about the shoulder
    (abduction in the chest's frame). Returns the turn's axis (None where the arm lies along it), the smallest angle
    that clears them (rad; 0 when they are clear, max_deg when none does: a turn that cannot clear them still takes the
    arm the most out of them) and the largest that `slip` allows (m, a held hand's 0 or a planted one's
    ARM_CLEAR['slip']; max_deg without one): the wrist and the fingertips move at most that far in the picture (the
    view's x and up; its depth is free, so a hand on the floor, a bar or a pole slides along it, out of the picture, and
    the overlap left is the motion's to fix)."""
    C = ARM_CLEAR
    top = math.radians(C["max_deg"])
    d = kn - hp
    out = out - up * out.dot(up)
    if d.length < 1e-6 or out.length < 1e-6:
        return None, 0.0, 0.0
    axis = (wr - sh).cross(out.normalized())
    if axis.length < 1e-6:
        return None, 0.0, 0.0
    axis.normalize()
    t0, t1 = hp, hp + d.normalized() * C["tasset_len"]

    def deficit(e, w, t):
        worst = -1.0
        for p, q, r in ((e, e, C["elbow_r"]), (e, w, C["forearm_r"]), (w, t, C["hand_r"])):
            dist = _seg_dist(p, q, t0, t1)[0]
            worst = max(worst, r + C["tasset_r"] - dist)
        return worst
    need = 0.0 if deficit(el, wr, tip) <= 0.0 else None
    cap = top if slip is None else None
    step = math.radians(0.25)
    ang = 0.0
    while (need is None or cap is None) and ang < top:
        ang += step
        R = Matrix.Rotation(ang, 3, axis)
        e2, w2, t2 = (sh + R @ (p - sh) for p in (el, wr, tip))
        if cap is None and max(math.hypot((b - a).x, (b - a).z) for a, b in ((wr, w2), (tip, t2))) > slip:
            cap = ang - step
        if need is None and deficit(e2, w2, t2) <= 0.0:
            need = ang
    return axis, top if need is None else need, top if cap is None else cap


def arm_turns(frames, view):
    """Each frame's turn of each arm past its tasset over a loop of track frames, as place() takes it ({side: (axis,
    angle)}, a side left out keeping its pose): arm_clear's smallest turn that clears, widened so it changes at most
    ARM_CLEAR['rate_deg'] a tick (the loop's seam too), so an arm turns out ahead of its tasset and back after it,
    never in one jump; then lowered under what each frame's hand allows (the track's held_arms keep their pose, its
    planted_arms slip ARM_CLEAR['slip'] at most: motion.arm_holds) at the same rate, so a hand that lands or lets go
    eases into its hold and out of it."""
    rate = math.radians(ARM_CLEAR["rate_deg"])
    poses = [normalize_joints(fr["joints"], view) for fr in frames]
    ups = [_chest_frame(J, view).col[2].xyz for J in poses]
    out = [{} for _ in frames]
    for side in ("near", "far"):
        axes, need, cap = [], [], []
        for fr, J, up in zip(frames, poses, ups):
            sh, el, wr, tip, hp, kn = _limb_points(J, view, side)[:6]
            slip = 0.0 if side in fr["held_arms"] else ARM_CLEAR["slip"] if side in fr["planted_arms"] else None
            axis, a, c = arm_clear(sh, el, wr, tip, hp, kn, sh - _v(J["neck"]), up, slip)
            axes.append(axis)
            need.append(a)
            cap.append(c)
        turn = _spread([min(a, c) for a, c in zip(_spread(need, rate, max), cap)], rate, min)
        for o, axis, a in zip(out, axes, turn):
            if axis is not None and a > 0.0:
                o[side] = (axis, a)
    return out


def _spread(xs, rate, pick):
    """A loop's values carried to their neighbours: with max, the smallest series at or above xs, with min the
    largest at or below it, whose steps are at most `rate` (cyclic, the last value running into the first)."""
    s, n = list(xs), len(xs)
    sign = -1.0 if pick is max else 1.0
    for order in (range(2 * n), range(2 * n - 1, -1, -1)):
        prev = None
        for i in order:
            j = i % n
            if prev is not None:
                s[j] = pick(s[j], s[prev] + sign * rate)
            prev = j
    return s


def _limb_points(J, view, side):
    """One side's shoulder, elbow, wrist, fingertips, hip, knee, ankle and toe as place() draws them (m): a side
    view's arms and legs out on their own planes (ARM_DEPTH, LEG_DEPTH)."""
    sgn = 0.0 if view == "front" else (-1.0 if side == "near" else 1.0)
    S = J[side]
    return (tuple(_v(S[k], sgn * ARM_DEPTH) for k in ("shoulder", "elbow", "wrist", "hand_tip"))
            + tuple(_v(S[k], sgn * LEG_DEPTH) for k in ("hip", "knee", "ankle", "toe")))


def _chest_frame(J, view):
    """The chest's frame: from the waist to the neck, its side along the shoulders."""
    lat_sh = _limb_points(J, view, "far")[0] - _limb_points(J, view, "near")[0]
    return frame_axis(_v(J["waist"]), _v(J["neck"]), view, up=True, lat=lat_sh)


_HIP_CACHE = {}


def _env_table(ob, shell=False, dz=0.005, zlo=-0.05, zhi=0.55, nphi=72):
    """Radial envelope of a thigh-frame part (thigh or tasset) per (distance down the leg, angle) bin, dilated one
    bin each way: E = max distance from the leg axis (0 where no plate), I = min distance. A shell (the tasset)
    occupies [I, E]; the cuisse holds the leg, so where its ring is closed it occupies [0, E] (where its end is
    trimmed open, like the checker's filled solid, only the plate)."""
    import numpy as np
    V = _verts_np(ob)
    s = -V[:, 2]
    rho = np.hypot(V[:, 0], V[:, 1])
    nz = int(round((zhi - zlo) / dz))
    iz = np.floor((s - zlo) / dz).astype(int)
    ip = (np.floor((np.arctan2(V[:, 1], V[:, 0]) + math.pi) / (2 * math.pi) * nphi).astype(int)) % nphi
    ok = (iz >= 0) & (iz < nz)
    G = np.zeros((nz, nphi))
    np.maximum.at(G, (iz[ok], ip[ok]), rho[ok])
    Gi = np.full((nz, nphi), 9.0)
    np.minimum.at(Gi, (iz[ok], ip[ok]), rho[ok])

    def dil(A, op):
        D = A.copy()
        D[1:] = op(D[1:], A[:-1])
        D[:-1] = op(D[:-1], A[1:])
        return op(D, op(np.roll(D, 1, axis=1), np.roll(D, -1, axis=1)))
    E = dil(G, np.maximum)
    Ii = dil(Gi, np.minimum)
    if not shell:                                  # a slice whose ring is closed holds the leg: filled from 0
        ring = G > 0.0
        ring = ring | np.roll(ring, 1, axis=1) | np.roll(ring, -1, axis=1)
        Ii[ring.all(axis=1)] = 0.0
    return {"E": E, "I": Ii, "dz": dz, "zlo": zlo, "nphi": nphi, "nz": nz}


def _hip_cache(parts):
    import numpy as np
    key = parts["fauld"].name + str(len(parts["fauld"].data.vertices))
    if _HIP_CACHE.get("key") != key:
        V = _verts_np(parts["fauld"])
        step = max(1, len(V) // 2500)
        _HIP_CACHE.clear()
        _HIP_CACHE.update(key=key, fauld=np.ascontiguousarray(V[::step]),
                          env={n: _env_table(parts[n], shell=n.startswith("tasset_")) for n in parts
                               if n.startswith(("thigh_", "tasset_")) and n.endswith(("_near", "_far"))})
    return _HIP_CACHE


def _conflicts(P, M, T, margin):
    """How many world points P (n, 3) lie in the thigh-frame part with world matrix M and envelope T (+ margin)."""
    import numpy as np
    Mi = np.linalg.inv(M)
    L = P @ Mi[:3, :3].T + Mi[:3, 3]
    s = -L[:, 2]
    iz = np.floor((s - T["zlo"]) / T["dz"]).astype(int)
    ok = (iz >= 0) & (iz < T["nz"])
    if not ok.any():
        return 0
    L, iz = L[ok], iz[ok]
    ip = (np.floor((np.arctan2(L[:, 1], L[:, 0]) + math.pi) / (2 * math.pi) * T["nphi"]).astype(int)) % T["nphi"]
    e, i = T["E"][iz, ip], T["I"][iz, ip]
    r = np.hypot(L[:, 0], L[:, 1])
    return int(((e > 0.0) & (r < e + margin) & (r > i - margin)).sum())


def fauld_turn(parts, Mab, placed):
    """The fauld's turn (deg) about its hip pivots: the smallest that keeps it `margin` clear of both thighs and
    both tassets as placed (`placed`: name -> world Matrix; so it rides up over the rising thighs), coarse 4 deg steps then HIP5['step_deg'];
    when no turn up to max_deg is clear, the one with the fewest points inside."""
    import numpy as np
    C = _hip_cache(parts)
    F = C["fauld"]
    px, pz = HIP5["pivot"]
    A = np.array([list(r) for r in Mab])
    mats = {n: np.array([list(r) for r in placed[n]]) for n in C["env"]}

    def count(beta):
        b = math.radians(beta)
        cb, sb = math.cos(b), math.sin(b)
        x, z = F[:, 0] - px, F[:, 2] - pz
        L = np.stack([px + x * cb - z * sb, F[:, 1], pz + x * sb + z * cb], axis=1)
        W = L @ A[:3, :3].T + A[:3, 3]
        return sum(_conflicts(W, mats[n], C["env"][n], HIP5["margin"]) for n in mats)
    if count(0.0) == 0:
        return 0.0
    mx, prev, best = HIP5["max_deg"], 0.0, (1 << 30, 0.0)
    b = 4.0
    while b <= mx + 1e-9:
        c = count(b)
        best = min(best, (c, b))
        if c == 0:
            f = prev + HIP5["step_deg"]
            while f < b:
                if count(f) == 0:
                    return f
                f += HIP5["step_deg"]
            return b
        prev, b = b, b + 4.0
    return best[1]


def fauld_matrix(Mab, beta):
    px, pz = HIP5["pivot"]
    return (Mab @ Matrix.Translation((px, 0.0, pz)) @ Matrix.Rotation(-math.radians(beta), 4, "Y")
            @ Matrix.Translation((-px, 0.0, -pz)))


def hip_flexion(pel, wai, hp, kn):
    """Hip flexion (deg): the angle between the trunk's downward axis and the thigh."""
    down, leg = pel - wai, kn - hp
    if down.length < 1e-9 or leg.length < 1e-9:
        return 0.0
    return math.degrees(down.angle(leg))


def fauld_ramp(flex):
    """The fauld's turn (deg) for the deeper of the two hip flexions: HIP5['turn'] = (from deg, full at deg, max
    deg), a smoothstep, so the lame rides up steadily with the thighs (measured: fauld_turn, the smallest clear
    turn, is 0 up to ~88 deg of flexion and 69 deg at 112; the ramp stays above it)."""
    a0, a1, b = HIP5["turn"]
    return b * _smooth((flex - a0) / (a1 - a0))


def tasset_slide(pel, wai, hp, kn):
    """How far (m) a tasset slides down its thigh: HIP5['slide'] = (from deg, full at deg, metres) of hip flexion."""
    a0, a1, d = HIP5["slide"]
    return d * _smooth((hip_flexion(pel, wai, hp, kn) - a0) / (a1 - a0))


def place(parts, joints, turns, view="side", sash=None, pad=None, head_l=None, sash_rest=None):
    """Set matrix_world of every rigid part from one frame's joints (rig units), per the interface. The torso's plates
    face where its hip and shoulder lines say and the helm where `head_l` (the head's lateral axis, the track's
    per-frame direction) says, so a twist or a turn shows; the limbs turn with the hips and shoulders too (side view:
    the lines set their frames, the feet's too; front view: as far as the body has turned off the view axis,
    body_turned); the arms turn clear of their tassets as `turns` (the frame's arm_turns) says; the sash lies flat
    where `sash_rest` says it rests (place_sash). `pad` overrides the pauldron schedule (PAD[view]) for
    experiments."""
    J = normalize_joints(joints, view)
    pel, wai, nec, hed = (_v(J[k]) for k in ("pelvis", "waist", "neck", "head"))
    depth = {s: (0.0 if view == "front" else (-1.0 if s == "near" else 1.0)) for s in ("near", "far")}
    hips = {s: _v(J[s]["hip"], depth[s] * LEG_DEPTH) for s in depth}
    shoulders = {s: _v(J[s]["shoulder"], depth[s] * ARM_DEPTH) for s in depth}
    lat_hip, lat_sh = hips["far"] - hips["near"], shoulders["far"] - shoulders["near"]
    Mab = frame_axis(pel, wai, view, up=True, lat=lat_hip)
    Mch = frame_axis(wai, nec, view, up=True, lat=lat_sh)
    parts["abdomen"].matrix_world = Mab
    placed, flex = {}, 0.0
    parts["chest"].matrix_world = Mch
    Mh = frame_axis(nec, hed, view, up=True, lat=_v(head_l) if head_l else lat_sh)
    Mh.translation = nec + Mh.col[2].xyz * HEAD_LIFT
    parts["head"].matrix_world = Mh
    arm_turned, leg_turned = ((body_turned(lat_sh, Mch.col[0].xyz), body_turned(lat_hip, Mab.col[0].xyz))
                              if view == "front" else (1.0, 1.0))
    for side in ("near", "far"):
        sh, el, wr, tip, hp, kn, an, to = _limb_points(J, view, side)
        if side in turns:
            R = Matrix.Rotation(turns[side][1], 3, turns[side][0])
            el, wr, tip = (sh + R @ (p - sh) for p in (el, wr, tip))
        Mua = frame_axis(sh, el, view, lat=lat_sh, turned=arm_turned)
        Mfa = frame_axis(el, wr, view, lat=lat_sh, turned=arm_turned)
        Mth = frame_axis(hp, kn, view, lat=lat_hip, turned=leg_turned)
        Msh = frame_axis(kn, an, view, lat=lat_hip, turned=leg_turned)
        parts[f"upper_arm_{side}"].matrix_world = Mua
        parts[f"forearm_{side}"].matrix_world = Mfa
        parts[f"thigh_{side}"].matrix_world = Mth
        Mta = Mth @ Matrix.Translation((0.0, 0.0, -tasset_slide(pel, wai, hp, kn)))
        parts[f"tasset_{side}"].matrix_world = Mta
        placed[f"thigh_{side}"], placed[f"tasset_{side}"] = Mth, Mta
        flex = max(flex, hip_flexion(pel, wai, hp, kn))
        parts[f"shin_{side}"].matrix_world = Msh
        parts[f"hand_{side}"].matrix_world = frame_axis(wr, tip, view, lat=lat_sh, turned=arm_turned)
        parts[f"foot_{side}"].matrix_world = frame_foot(an, to, view, side, lat=lat_hip)
        parts[f"elbow_{side}"].matrix_world = _half(Mua, Mfa, el)
        parts[f"knee_{side}"].matrix_world = _half(Mth, Msh, kn)
        parts[f"pauldron_{side}"].matrix_world = pauldron_matrix(Mch, sh, el, _v(J["neck"]), view, pad)
    parts["fauld"].matrix_world = fauld_matrix(Mab, fauld_ramp(flex))
    flexes = [hip_flexion(pel, wai, _v(J[s]["hip"], depth[s] * LEG_DEPTH), _v(J[s]["knee"], depth[s] * LEG_DEPTH))
              for s in ("near", "far")]
    parts["culet"].matrix_world = culet_matrix(Mab, sum(flexes) / 2)
    place_sash(parts, sash, sash_rest)


# ================================================================ stage

def _stone_slab(name, r0, r1, a0, a1, top, depth, mat, coll, bevel=0.012):
    bm = bmesh.new()
    n = max(2, int((a1 - a0) / math.radians(6)))
    ring_in, ring_out = [], []
    for i in range(n + 1):
        a = a0 + (a1 - a0) * i / n
        ring_in.append((r0 * math.cos(a), r0 * math.sin(a)))
        ring_out.append((r1 * math.cos(a), r1 * math.sin(a)))
    outline = ring_out + ring_in[::-1] if r0 > 0 else ring_out
    vt = [bm.verts.new((x, y, top)) for x, y in outline]
    vb = [bm.verts.new((x, y, top - depth)) for x, y in outline]
    bm.faces.new(vt)
    bm.faces.new(vb[::-1])
    k = len(outline)
    for i in range(k):
        j = (i + 1) % k
        bm.faces.new((vt[j], vt[i], vb[i], vb[j]))
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:])
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me)
    bm.free()
    me.materials.append(mat)
    ob = bpy.data.objects.new(name, me)
    coll.objects.link(ob)
    bv = ob.modifiers.new("bevel", "BEVEL")
    bv.width = bevel
    bv.segments = 2
    bv.limit_method = "ANGLE"
    bv.use_clamp_overlap = True
    me.shade_flat()
    return ob


def _freeze_in_place(ob):
    bpy.context.view_layer.update()
    dg = bpy.context.evaluated_depsgraph_get()
    me = bpy.data.meshes.new_from_object(ob.evaluated_get(dg), preserve_all_data_layers=True, depsgraph=dg)
    old = ob.data
    ob.modifiers.clear()
    ob.data = me
    if old.users == 0:
        bpy.data.meshes.remove(old)
    return ob


def stage(scene, props=(), radius=1.0):
    """Stone training circle (flagstone rings, a raised kerb, a faint gold rune ring) plus props:
    {"kind": "wall", "x": 100} (a stone wall whose face is at x cm, facing the figure) and
    {"kind": "box", "rect": [x, y, w, h]} (a stone block). Lit by its own soft light (render_setup)."""
    coll = _collection(STAGE_COLL, scene)
    for ob in list(coll.objects):                      # idempotent: a new call replaces the previous stage
        me = ob.data if ob.type == "MESH" else None
        bpy.data.objects.remove(ob, do_unlink=True)
        if me is not None and me.users == 0:
            bpy.data.meshes.remove(me)
    stone = mat_stone("qp_stone", "#26221e", "#6d655b", "#a39a8d")
    stone2 = mat_stone("qp_stone_dark", "#1d1a17", "#56504a", "#857c70", scale=5.0)
    gold = mat_stone("qp_floor_gold", "#2e1d08", "#8c6424", "#c69a48", speck=0.05, scale=12.0)
    inlay = mat_stone("qp_floor_inlay", "#100e0c", "#2c2722", "#3e3730", speck=0.08, scale=8.0)
    faint = mat_emit("qp_floor_rune", (0.95, 0.68, 0.26), 0.62)
    objs = []
    objs.append(_stone_slab("floor_center", 0.0, 0.30, 0, 2 * math.pi, 0.0, 0.08, stone, coll))
    rings = ((0.308, 0.58, 8, stone), (0.588, 0.80, 12, stone2), (0.892, radius, 16, stone))
    for k, (r0, r1, n, mat) in enumerate(rings):
        for i in range(n):
            a0 = 2 * math.pi * i / n + 0.004 + (0.13 if k == 1 else 0.0)
            a1 = 2 * math.pi * (i + 1) / n - 0.004 + (0.13 if k == 1 else 0.0)
            top = 0.012 if k == 2 else 0.0
            objs.append(_stone_slab(f"floor_{k}_{i}", r0, r1, a0, a1, top, 0.08 + top, mat, coll))
    # gold inlay band carrying the rune ring
    objs.append(_stone_slab("floor_band", 0.814, 0.878, 0, 2 * math.pi, -0.003, 0.07, inlay, coll, bevel=0.003))
    objs.append(_stone_slab("floor_band_in", 0.804, 0.813, 0, 2 * math.pi, 0.0015, 0.07, gold, coll, bevel=0.002))
    objs.append(_stone_slab("floor_band_out", 0.879, 0.888, 0, 2 * math.pi, 0.0015, 0.07, gold, coll, bevel=0.002))
    for ob in objs:
        _freeze_in_place(ob)
        _grad_attr(ob.data)
    # glyphs lying on the band
    verts, faces = [], []
    names = list(GLYPHS)
    for i in range(16):
        a = 2 * math.pi * i / 16
        g = names[i % len(names)]
        s = 0.052 / 3.0
        cx, cy = 0.846 * math.cos(a), 0.846 * math.sin(a)
        tang = Vector((-math.sin(a), math.cos(a), 0))
        rad = Vector((math.cos(a), math.sin(a), 0))
        for stroke in GLYPHS[g]:
            for (p0, p1, p2, p3, _) in _stroke_quads(stroke, 0.0085 / s):
                base = len(verts)
                for gx, gy in (p0, p1, p2, p3):
                    p = Vector((cx, cy, -0.0022)) + tang * (gx * s) + rad * ((gy - 1.5) * s)
                    verts.append(tuple(p))
                faces.append((base, base + 1, base + 2, base + 3))
    rr = mesh_obj("floor_runes", verts, faces, faint, smooth=False)
    for c in rr.users_collection:
        c.objects.unlink(rr)
    coll.objects.link(rr)
    rr.visible_shadow = False
    for p in props or ():
        if p.get("kind") == "wall":
            _wall(p["x"] * U, mat_stone("qp_wall", "#2a2520", "#766d62", "#a59b8e", lift=0.42),
                  mat_stone("qp_wall_dark", "#221e1a", "#625a50", "#8a8074", scale=5.0, lift=0.42), coll)
        elif p.get("kind") == "box":
            x, y, w, h = p["rect"]
            ob = _block(f"box_{x}_{y}", (x + w / 2) * U, 0.0, (y + h / 2) * U, w * U, 0.7, h * U, stone, coll)
            _freeze_in_place(ob)
            _grad_attr(ob.data)
    return coll


def _block(name, cx, cy, cz, sx, sy, sz, mat, coll, bevel=0.018):
    bm = bmesh.new()
    bmesh.ops.create_cube(bm, size=1.0, matrix=Matrix.Translation((cx, cy, cz)) @ Matrix.Diagonal((sx, sy, sz, 1)))
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me)
    bm.free()
    me.materials.append(mat)
    ob = bpy.data.objects.new(name, me)
    coll.objects.link(ob)
    bv = ob.modifiers.new("bevel", "BEVEL")
    bv.width = bevel
    bv.segments = 2
    bv.use_clamp_overlap = True
    return ob


def _wall(x, stone, stone2, coll, depth=0.30, width=1.5, height=2.1):
    """Running-bond stone wall section, face at X = x toward the figure (-X)."""
    rows = 7
    rh = height / rows
    k = 0
    for r in range(rows):
        off = 0.5 if r % 2 else 0.0
        n = 4
        bw = width / n
        for c in range(-1, n + 1):
            y0 = -width / 2 + (c + off) * bw
            y1 = y0 + bw
            y0, y1 = max(y0, -width / 2), min(y1, width / 2)
            if y1 - y0 < 0.05:
                continue
            jitter = 0.012 * math.sin(r * 3.1 + c * 1.7)
            ob = _block(f"wall_{r}_{c}", x + depth / 2 + jitter, (y0 + y1) / 2, r * rh + rh / 2,
                        depth, y1 - y0 - 0.012, rh - 0.012, stone if (r + c) % 3 else stone2, coll)
            _freeze_in_place(ob)
            _grad_attr(ob.data)
            k += 1
    cap = _block("wall_cap", x + depth / 2, 0.0, height + 0.05, depth + 0.06, width + 0.06, 0.1, stone2, coll)
    _freeze_in_place(cap)
    _grad_attr(cap.data)
    foot = _block("wall_footing", x + depth / 2 - 0.02, 0.0, -0.045, depth + 0.16, width + 0.16, 0.09, stone2, coll)
    _freeze_in_place(foot)
    _grad_attr(foot.data)


# ================================================================ render settings, lights, bloom, outlines

def _sun(scene, name, direction, energy, angle_deg, color=(1, 1, 1)):
    ld = bpy.data.lights.get(name) or bpy.data.lights.new(name, "SUN")
    ld.energy = energy
    ld.color = color
    ld.angle = math.radians(angle_deg)
    ld.use_shadow = True
    ld.use_shadow_jitter = False
    ld.shadow_filter_radius = 1.0
    ob = bpy.data.objects.get(name) or bpy.data.objects.new(name, ld)
    if ob.name not in scene.collection.objects:
        scene.collection.objects.link(ob)
    ob.rotation_euler = (-Vector(direction)).normalized().to_track_quat("-Z", "Y").to_euler()
    return ob


def aim_key(scene):
    """Key light from the viewer's upper right, a little toward the camera: screen-consistent lit shapes for
    every camera. Call again after moving the camera."""
    key = bpy.data.objects.get("qp_key")
    cam = scene.camera
    if key is None:
        return
    if cam is None:
        src = Vector((0.55, -0.45, 0.75))
    else:
        R = cam.matrix_world.to_3x3()
        right, upv, back = R.col[0], R.col[1], R.col[2]
        src = right * 0.62 + upv * 0.78 + back * 0.52
    key.rotation_euler = (-src).normalized().to_track_quat("-Z", "Y").to_euler()
    fl = bpy.data.objects.get("qp_floor_light")
    if fl is not None:
        back = Vector((0, -1, 0)) if cam is None else cam.matrix_world.to_3x3().col[2]
        bh = Vector((back.x, back.y, 0.0))
        bh = bh.normalized() if bh.length > 1e-6 else Vector((0, -1, 0))
        fsrc = Vector((0, 0, 1)) - bh * 0.32
        fl.rotation_euler = (-fsrc).normalized().to_track_quat("-Z", "Y").to_euler()


def apply_camera(scene, spec, distance=12.0):
    """The motion side's per-move camera {yaw_deg, pitch_deg, center_x, center_y, ortho_scale_m}: orthographic,
    looking at (center_x, center_y) in rig units (cm) at depth 0; yaw 0 looks from -Y (side view: the figure's near
    side; front view: its back), positive yaw swings toward +X, positive pitch looks down. Re-aims the lights and
    re-sizes the outlines for this frame scale."""
    cam = scene.camera
    if cam is None or cam.type != "CAMERA":
        cd = bpy.data.cameras.new("qp_cam")
        cam = bpy.data.objects.new("qp_cam", cd)
        scene.collection.objects.link(cam)
        scene.camera = cam
    cam.data.type = "ORTHO"
    cam.data.ortho_scale = float(spec["ortho_scale_m"])
    cam.data.clip_start = 0.1
    cam.data.clip_end = distance * 4
    y, p = math.radians(spec.get("yaw_deg", 0.0)), math.radians(spec.get("pitch_deg", 0.0))
    back = Vector((math.sin(y) * math.cos(p), -math.cos(y) * math.cos(p), math.sin(p)))
    target = Vector((spec.get("center_x", 0.0) * U, 0.0, spec.get("center_y", 0.0) * U))
    cam.matrix_world = (-back).to_track_quat("-Z", "Y").to_matrix().to_4x4()
    cam.location = target + back * distance
    aim_key(scene)
    size_outlines(scene)
    return cam


def render_setup(scene, cell_px, samples=48, glow_dir=None, glow_name="glow", stage_rgb=None,
                 bloom_strength=0.55, bloom_size=0.11):
    """Render at 2x the final cell (downscale 50 % afterwards), Gaussian filter 1.5 px, no dither, 16-bit PNG RGBA,
    TAA `samples`, Standard view, a crisp key sun lighting the armor (light-linked) and a soft sun lighting only the
    stage (a soft contact shadow), dim cool ambient, and a 5.x Glare bloom (menu inputs Type=BLOOM, Quality=HIGH,
    threshold 1.2: only the runes, joint lights and the visor pass). Transparent film by default: the main image
    keeps the bloom inside the silhouette; with glow_dir the bloom outside it is written as an RGB layer
    (<glow_dir>/<glow_name>_####.png, add it with 'plus'/'lighter'). stage_rgb=(r,g,b) linear: opaque frames with
    that backdrop under the bloom."""
    r = scene.render
    r.engine = "BLENDER_EEVEE"
    r.resolution_x = r.resolution_y = int(cell_px) * 2
    r.resolution_percentage = 100
    r.filter_size = 1.5
    r.dither_intensity = 0.0
    r.film_transparent = True             # stage_rgb is laid under the figure by the compositor (Alpha Over)
    r.use_compositing = True
    im = r.image_settings
    im.media_type = "IMAGE"
    im.file_format = "PNG"
    im.color_mode = "RGBA"
    im.color_depth = "16"
    im.compression = 15
    ee = scene.eevee
    ee.taa_render_samples = samples
    ee.use_shadows = True
    ee.shadow_ray_count = 2
    ee.shadow_step_count = 8
    ee.use_raytracing = False
    scene.display_settings.display_device = "sRGB"
    vs = scene.view_settings
    vs.view_transform = "Standard"
    vs.look = "None"
    vs.exposure = 0.0
    vs.gamma = 1.0
    vs.use_curve_mapping = False
    scene["qp_cell_px"] = int(cell_px)
    w = scene.world or bpy.data.worlds.new("qp_world")
    scene.world = w
    bg = next(n for n in w.node_tree.nodes if n.type == "BACKGROUND")
    bg.inputs["Color"].default_value = (*hexc("#8a96b8"), 1)
    bg.inputs["Strength"].default_value = 0.16
    key = _sun(scene, "qp_key", (0.55, -0.45, 0.75), KEY_ENERGY, 1.5, color=(1.0, 0.97, 0.92))
    floor = _sun(scene, "qp_floor_light", (0.25, -0.35, 1.0), 2.6, 16.0, color=(1.0, 0.96, 0.9))
    floor.data.shadow_filter_radius = 3.0
    armor = bpy.data.collections.get(ARMOR_COLL)
    stg = bpy.data.collections.get(STAGE_COLL)
    if armor is not None and stg is not None:
        key.light_linking.receiver_collection = armor
        floor.light_linking.receiver_collection = stg
    aim_key(scene)
    _compositor(scene, glow_dir, glow_name, stage_rgb, bloom_strength, bloom_size)
    size_outlines(scene)
    return scene


def _menu(sock, *values):
    """5.2 menu sockets take the item name ('Bloom'); older builds took the identifier ('BLOOM'). Fail loudly."""
    for v in values:
        try:
            sock.default_value = v
            return
        except TypeError:
            continue
    raise ValueError(f"menu {sock.name}: none of {values}")


def _compositor(scene, glow_dir, glow_name, stage_rgb, strength, size):
    ng = bpy.data.node_groups.new("qp_comp", "CompositorNodeTree")
    ng.interface.new_socket(name="Image", in_out="OUTPUT", socket_type="NodeSocketColor")
    N, L = ng.nodes.new, ng.links.new
    rl = N("CompositorNodeRLayers")
    out = N("NodeGroupOutput")
    src = _sock(rl, "Image", True)
    if stage_rgb is not None:
        ao = N("CompositorNodeAlphaOver")
        _sock(ao, "Background").default_value = (*stage_rgb, 1.0)
        L(src, _sock(ao, "Foreground"))
        src = _sock(ao, "Image", True)
    gl = N("CompositorNodeGlare")
    _menu(_sock(gl, "Type"), "Bloom", "BLOOM")
    _menu(_sock(gl, "Quality"), "High", "HIGH")
    _sock(gl, "Threshold").default_value = 1.2
    _sock(gl, "Smoothness").default_value = 0.2
    _sock(gl, "Strength").default_value = strength
    _sock(gl, "Size").default_value = size
    L(src, _sock(gl, "Image"))
    L(_sock(gl, "Image", True), out.inputs[0])
    if glow_dir and stage_rgb is None:
        sep = N("CompositorNodeSeparateColor")
        L(_sock(rl, "Image", True), _sock(sep, "Image"))
        inv = N("ShaderNodeMath")
        inv.operation = "SUBTRACT"
        inv.inputs[0].default_value = 1.0
        L(_sock(sep, "Alpha", True), inv.inputs[1])
        mul = N("ShaderNodeMix")
        mul.data_type = "RGBA"
        mul.blend_type = "MULTIPLY"
        _sock(mul, "Factor").default_value = 1.0
        L(_sock(gl, "Glare", True), _sock(mul, "A"))
        L(inv.outputs[0], _sock(mul, "B"))
        sa = N("CompositorNodeSetAlpha")
        _sock(sa, "Alpha").default_value = 1.0
        L(_sock(mul, "Result", True), _sock(sa, "Image"))
        fo = N("CompositorNodeOutputFile")
        fo.directory = glow_dir
        fo.file_name = ""
        fo.format.media_type = "IMAGE"
        fo.format.file_format = "PNG"
        fo.format.color_mode = "RGB"
        fo.format.color_depth = "16"
        fo.file_output_items.new("RGBA", glow_name + "_####")
        L(_sock(sa, "Image", True), fo.inputs[0])
    scene.compositing_node_group = ng
    return ng


def size_outlines(scene, parts=None):
    """Outline (inverted hull) width in FINAL pixels: thickness = px * ortho_scale / cell_px, per part from its
    'qp_outline_px'. Call after the camera is framed (render_setup calls it when a camera exists)."""
    cam = scene.camera
    if cam is None or cam.data.type != "ORTHO":
        return
    cell = scene.get("qp_cell_px", scene.render.resolution_x // 2)
    m_per_px = cam.data.ortho_scale / float(cell)
    objs = parts.values() if parts else bpy.data.objects
    for ob in objs:
        md = ob.modifiers.get("qp_outline") if ob.type == "MESH" else None
        if md:
            md.thickness = ob.get("qp_outline_px", OUTLINE_PX["default"]) * m_per_px
