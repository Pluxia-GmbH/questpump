"""The scene around the armour (Blender 5.2 LTS, headless): the floor pool and its contact shadow, the static props of
`props.py` and the held implements of `items.py` (docs/plan.md slice 2, "Grounding"; ported from the scratchpad
prototype's v4/props/render_ground.py on 2026-09-28 and grown from its one wall to every prop kind).

Body space (cm; f forward, u up, l lateral, far side positive) maps to Blender metres per view: the side view looks from
the near side (X = f, Y = l, Z = u), the front view from behind (X = -l, Y = f, Z = u).
"""
import math
import os
import sys

import bmesh
import bpy
from mathutils import Matrix, Vector

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.dirname(HERE))
import armor_model as am  # noqa: E402
import items as I  # noqa: E402
import props as P  # noqa: E402

GROUND = "qp_ground"


def to_bl(v, view):
    f, u, l = v
    return Vector((f, l, u)) * am.U if view == "side" else Vector((-l, f, u)) * am.U


def srgb_to_lin(c):
    return c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4


def hexlin(h):
    h = h.lstrip("#")
    return tuple(srgb_to_lin(int(h[i:i + 2], 16) / 255.0) for i in (0, 2, 4))


# ---------------------------------------------------------------- node helpers

def node(nt, kind, **inputs):
    n = nt.nodes.new(kind)
    for k, v in inputs.items():
        am._sock(n, k).default_value = v
    return n


def out(n, key):
    return am._sock(n, key, True)


def inp(n, key):
    return am._sock(n, key)


def vmath(nt, op, b=None):
    n = nt.nodes.new("ShaderNodeVectorMath")
    n.operation = op
    if b is not None:
        n.inputs[1].default_value = b
    return n


def fmath(nt, op, a=None, b=None, clamp=False):
    return am._math(nt, op, a, b, clamp=clamp)


def lit_value(nt):
    """Shader to RGB of a white Lambert: the light reaching this point from the lights linked to it, shadows included
    (the world is black in these passes, so nothing else adds)."""
    return am._lit_value(nt, nt.nodes.new, nt.links.new)


def blank_material(name, blended=False):
    m = bpy.data.materials.new(name)
    if blended:
        m.surface_render_method = "BLENDED"
        m.use_transparency_overlap = False
    nt = m.node_tree
    for n in list(nt.nodes):
        if n.type != "OUTPUT_MATERIAL":
            nt.nodes.remove(n)
    return m, nt, next(n for n in nt.nodes if n.type == "OUTPUT_MATERIAL")


# ---------------------------------------------------------------- floor

def mat_floor(st, centre, uax, vax, ru, rv, norm):
    """The implied floor as one blended surface over the card:
        pool   = pool_alpha * (1 - e)^pool_power, e = (u/ru)^2 + (v/rv)^2  (a soft pool of floor light)
        shadow = shadow_alpha * (1 - lit) * clamp(pool shape * shadow_reach)
                 lit = the floor sun's light / its unshadowed value
    drawn as the shadow colour at `shadow` over the pool colour at `pool` over nothing (transparent)."""
    m, nt, mo = blank_material("qp_g_floor", blended=True)
    L = nt.links.new
    geo = nt.nodes.new("ShaderNodeNewGeometry")
    rel = vmath(nt, "SUBTRACT", b=centre)
    L(out(geo, "Position"), rel.inputs[0])
    shape = None
    for ax, r in ((uax, ru), (vax, rv)):
        d = vmath(nt, "DOT_PRODUCT", b=ax)
        L(rel.outputs[0], d.inputs[0])
        s = fmath(nt, "DIVIDE", b=r)
        L(out(d, "Value"), s.inputs[0])
        q = fmath(nt, "POWER", b=2.0)
        L(s.outputs[0], q.inputs[0])
        if shape is None:
            shape = q
        else:
            e = fmath(nt, "ADD")
            L(shape.outputs[0], e.inputs[0])
            L(q.outputs[0], e.inputs[1])
            shape = e
    one_m = fmath(nt, "SUBTRACT", a=1.0, clamp=True)
    L(shape.outputs[0], one_m.inputs[1])
    pw = fmath(nt, "POWER", b=st["pool_power"])
    L(one_m.outputs[0], pw.inputs[0])
    pool = fmath(nt, "MULTIPLY", b=st["pool_alpha"], clamp=True)
    L(pw.outputs[0], pool.inputs[0])
    lit = fmath(nt, "MULTIPLY", b=norm, clamp=True)
    L(lit_value(nt), lit.inputs[0])
    sh = fmath(nt, "SUBTRACT", a=1.0, clamp=True)
    L(lit.outputs[0], sh.inputs[1])
    sha = fmath(nt, "MULTIPLY", b=st["shadow_alpha"], clamp=True)
    L(sh.outputs[0], sha.inputs[0])
    reach = fmath(nt, "MULTIPLY", b=st["shadow_reach"], clamp=True)
    L(pw.outputs[0], reach.inputs[0])
    conf = fmath(nt, "MULTIPLY", clamp=True)
    L(sha.outputs[0], conf.inputs[0])
    L(reach.outputs[0], conf.inputs[1])
    tr = nt.nodes.new("ShaderNodeBsdfTransparent")
    ep = node(nt, "ShaderNodeEmission", Color=(*hexlin(st["pool_rgb"]), 1.0), Strength=1.0)
    es = node(nt, "ShaderNodeEmission", Color=(*hexlin(st["shadow_rgb"]), 1.0), Strength=1.0)
    m1 = nt.nodes.new("ShaderNodeMixShader")
    L(pool.outputs[0], m1.inputs[0])
    L(out(tr, "BSDF"), m1.inputs[1])
    L(out(ep, "Emission"), m1.inputs[2])
    m2 = nt.nodes.new("ShaderNodeMixShader")
    L(conf.outputs[0], m2.inputs[0])
    L(m1.outputs[0], m2.inputs[1])
    L(out(es, "Emission"), m2.inputs[2])
    L(m2.outputs[0], inp(mo, "Surface"))
    return m


def floor_plane(coll, mat, size=14.0):
    bm = bmesh.new()
    bmesh.ops.create_grid(bm, x_segments=1, y_segments=1, size=size / 2)
    me = bpy.data.meshes.new("qp_g_floor")
    bm.to_mesh(me)
    bm.free()
    me.materials.append(mat)
    ob = bpy.data.objects.new("qp_g_floor", me)
    coll.objects.link(ob)
    ob.visible_shadow = False
    return ob


def pool_geometry(track, contacts, st, view):
    """Pool centre and axes on the floor from the contact joints over the whole loop (and the props' feet): u along
    the camera's right, v along the view direction; the radii span the contacts plus a margin."""
    yaw = math.radians(track["camera"]["yaw_deg"])
    uax = Vector((math.cos(yaw), math.sin(yaw), 0.0))
    vax = Vector((-math.sin(yaw), math.cos(yaw), 0.0))
    pts = []
    for fr in track["frames"]:
        for j in contacts:
            p = am._v(fr["joints"][j], 0.0)
            if view == "side":
                side = j.rsplit("_", 1)[-1] if j.endswith(("_near", "_far")) else None
                if side:
                    limb = j[: -len(side) - 1]
                    sg = -1.0 if side == "near" else 1.0
                    p.y += sg * (am.ARM_DEPTH if limb in ("shoulder", "elbow", "wrist", "hand_tip") else am.LEG_DEPTH)
            pts.append(Vector((p.x, p.y, 0.0)))
    for pr in track["props"]:
        for v in P.frame_points(pr):
            if v[1] < 1.0:
                b = to_bl(v, view)
                pts.append(Vector((b.x, b.y, 0.0)))
    if not pts:
        pts = [Vector((0.0, 0.0, 0.0))]
    us = [p.dot(uax) for p in pts]
    vs = [p.dot(vax) for p in pts]
    cu, cv = (min(us) + max(us)) / 2, (min(vs) + max(vs)) / 2
    ru = (max(us) - min(us)) / 2 + st["pool_margin_m"]
    rv = (max(vs) - min(vs)) / 2 + st["pool_depth_m"]
    return uax * cu + vax * cv, uax, vax, ru, rv


# ---------------------------------------------------------------- props

def mat_ramp(name, stops, brick=None):
    """A flat toon surface: the lit value through a colour ramp, with optional brick courses on a wall face (the
    plane X = const in the side view)."""
    m, nt, mo = blank_material(name)
    L = nt.links.new
    rp = am._ramp(nt, [(p, hexlin(c)) for p, c in stops], interp="LINEAR")
    L(lit_value(nt), inp(rp, "Fac"))
    col = out(rp, "Color")
    if brick:
        tc = nt.nodes.new("ShaderNodeTexCoord")
        sep = nt.nodes.new("ShaderNodeSeparateXYZ")
        L(out(tc, "Object"), inp(sep, "Vector"))
        comb = nt.nodes.new("ShaderNodeCombineXYZ")
        L(out(sep, "Y"), inp(comb, "X"))
        L(out(sep, "Z"), inp(comb, "Y"))
        L(out(sep, "X"), inp(comb, "Z"))
        bt = nt.nodes.new("ShaderNodeTexBrick")
        bt.offset = 0.5
        bt.offset_frequency = 2
        inp(bt, "Scale").default_value = 1.0
        inp(bt, "Mortar Size").default_value = brick["mortar"]
        inp(bt, "Mortar Smooth").default_value = 0.0
        inp(bt, "Brick Width").default_value = brick["width"]
        inp(bt, "Row Height").default_value = brick["height"]
        inp(bt, "Color1").default_value = (1.0, 1.0, 1.0, 1)
        k = brick["block_var"]
        inp(bt, "Color2").default_value = (1.0 - k, 1.0 - k, 1.0 - k, 1)
        inp(bt, "Mortar").default_value = (brick["joint"], brick["joint"], brick["joint"], 1)
        inp(bt, "Bias").default_value = 0.0
        L(out(comb, "Vector"), inp(bt, "Vector"))
        mul = am._mix(nt, "MULTIPLY")
        L(col, inp(mul, "A"))
        L(out(bt, "Color"), inp(mul, "B"))
        col = out(mul, "Result")
    em = node(nt, "ShaderNodeEmission", Strength=1.0)
    L(col, inp(em, "Color"))
    L(out(em, "Emission"), inp(mo, "Surface"))
    return m


def box_mesh(name, lo, hi, mat, coll):
    bm = bmesh.new()
    c = (lo + hi) / 2
    bmesh.ops.create_cube(bm, size=1.0, matrix=Matrix.Translation(c) @ Matrix.Diagonal((*(hi - lo), 1)))
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me)
    bm.free()
    me.materials.append(mat)
    me.shade_flat()
    ob = bpy.data.objects.new(name, me)
    coll.objects.link(ob)
    return ob


def cylinder(name, a, b, r, mat, coll, verts=16):
    """A cylinder from point a to point b (Blender metres)."""
    d = b - a
    bm = bmesh.new()
    bmesh.ops.create_cone(bm, cap_ends=True, segments=verts, radius1=r, radius2=r, depth=d.length)
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me)
    bm.free()
    me.materials.append(mat)
    ob = bpy.data.objects.new(name, me)
    ob.matrix_world = Matrix.Translation((a + b) / 2) @ d.to_track_quat("Z", "Y").to_matrix().to_4x4()
    coll.objects.link(ob)
    return ob


def prop_materials(style):
    s = style["props"]
    return {
        "stone": mat_ramp("qp_g_stone", s["stone_ramp"], brick=s["brick"]),
        "wood": mat_ramp("qp_g_wood", s["wood_ramp"]),
        "iron": mat_ramp("qp_g_iron", s["iron_ramp"]),
        "leather": mat_ramp("qp_g_leather", s["leather_ramp"]),
        "gold": mat_ramp("qp_g_gold", s["gold_ramp"]),
        "mat": mat_ramp("qp_g_mat", s["mat_ramp"]),
    }


def moving(pr):
    """A prop whose parts move through the loop: a scrolling flight, a belt's slats."""
    return bool(pr.get("scroll") or pr.get("belt"))


def build_props(track, coll, mats, view, style):
    """Every prop's boxes and cylinders (props.parts), each wall with a thin gold edge on its near vertical edge and
    along its top, as the prototype's wall had. Returns [(prop, [(object, its part's centre at the loop's start)])],
    so a moving prop's parts follow props.parts through the loop (move_props)."""
    g = style["props"]["gold_w"]
    built = []
    for k, pr in enumerate(track["props"]):
        boxes, cyls = P.parts(pr)
        obs = []
        for i, (bx, mat) in enumerate(boxes):
            f0, f1, u0, u1, l0, l1 = bx
            a, b = to_bl((f0, u0, l0), view), to_bl((f1, u1, l1), view)
            lo = Vector((min(a.x, b.x), min(a.y, b.y), min(a.z, b.z)))
            hi = Vector((max(a.x, b.x), max(a.y, b.y), max(a.z, b.z)))
            obs.append((box_mesh(f"prop_{k}_{i}", lo, hi, mats[mat], coll), (lo + hi) / 2))
            if pr["kind"] == "wall":
                face = f0 if not pr.get("behind") else f1
                edge = (face - 0.5, face + 0.5 + g, 0.0, u1 + 0.5, l0 - 0.5, l0 + g)
                top = (f0 - 0.5, f1 + 0.5, u1 - g, u1 + 0.5, l0 - 0.5, l1)
                for j, e in enumerate((edge, top)):
                    a, b = to_bl(e[0::2], view), to_bl(e[1::2], view)
                    box_mesh(f"prop_{k}_{i}_gold{j}", Vector(map(min, a, b)), Vector(map(max, a, b)),
                             mats["gold"], coll)
        for i, ((a, b, r), mat) in enumerate(cyls):
            ob = cylinder(f"prop_{k}_c{i}", to_bl(a, view), to_bl(b, view), r * am.U, mats[mat], coll,
                          verts=48 if r > 10 else 16)
            obs.append((ob, (to_bl(a, view) + to_bl(b, view)) / 2))
        built.append((pr, obs))
    return built


def move_props(built, view, at):
    """Sets each moving prop's parts where props.parts puts them at `at` of the loop: a part keeps its shape and
    moves by its centre's offset from the loop's start."""
    for pr, obs in built:
        if not moving(pr):
            continue
        boxes, cyls = P.parts(pr, at)
        centres = [(to_bl(bx[0::2], view) + to_bl(bx[1::2], view)) / 2 for bx, _ in boxes]
        centres += [(to_bl(a, view) + to_bl(b, view)) / 2 for (a, b, _), _ in cyls]
        for (ob, start), c in zip(obs, centres):
            base = _BASE.setdefault(ob.name, ob.matrix_world.copy())
            ob.matrix_world = Matrix.Translation(c - start) @ base


_BASE = {}                    # each moving prop part's matrix at the loop's start


# ---------------------------------------------------------------- held implements

class Items:
    """The held implements of one move, built once in the figure's palette (a door towel's lines at its first pose,
    which sets their points) and placed per frame from the track's `items` (items.py poses, body space). They join the
    armour's collection, so the key light reaches them and the ground pass treats them as the armour: hidden from the
    floor's camera, held out of the prop layer. Rigid parts are meshes with the armour's outline hull; what stretches,
    swings or bends (a band, a strap, a crank, a rope, a towel, a bell's handle) is a bevelled curve."""

    def __init__(self, spec_items, view, coll):
        self.view, self.coll, self.obs = view, coll, []
        # held iron in a warm cast-iron tone: the armour's steel is cool blue, so a bell or a dumbbell against the
        # breastplate still reads as its own object at 192 px. Every implement keeps even its shadow bands about 2:1
        # against the card (an implement carries no 'grad' attribute, so mat_toon darkens it to 0.70), because the
        # phones draw the figure lighten over a page of nearly the card's colour
        steel = am.mat_toon("qp_item_iron", ("#615040", "#58483b", "#7b624c", "#977b5f", "#b8a084"), spec_amt=0.6)
        wood = am.mat_toon("qp_item_wood", ("#6a4d34", "#604631", "#8c5c30", "#aa753f", "#c7955c"), spec_amt=0.2)
        canvas = am.mat_toon("qp_item_canvas", ("#495368", "#434b5d", "#596783", "#6f819f", "#91a0b8"),
                             spec_amt=0.0, band_amt=0.05, edge_amt=0.1, scratches=0.0)
        towel = am.mat_toon("qp_item_towel", ("#5a5246", "#514a3f", "#5e564a", "#8f8573", "#b7ad99"),
                            spec_amt=0.0, band_amt=0.0, edge_amt=0.0, scratches=0.0)
        # a bath towel shut in a door: cream cloth, so the twisted rope of it reads as fabric, never a pole, its
        # shadow bands light too, since a towel in a door hangs mostly in the shade of its own twist (2026-10-26: at
        # #5d5240 / #a89a7c it read as a grey strap against the wood)
        bath = am.mat_toon("qp_item_bathtowel", ("#8a7f68", "#7d735d", "#d9ceb3", "#efe6cf", "#fbf6ea"),
                           spec_amt=0.0, band_amt=0.0, edge_amt=0.0, scratches=0.0)
        rubber = am.mat_toon("qp_item_rubber", ("#365a51", "#335149", "#2f6b5c", "#4f9c88", "#8fd0bd"),
                             spec_amt=0.1, band_amt=0.05, edge_amt=0.1, scratches=0.0)
        hide = am.mat_toon("qp_item_hide", ("#664f3d", "#5b473a", "#855f43", "#a77554", "#c6956c"),
                           spec_amt=0.15, band_amt=0.1, edge_amt=0.2, scratches=0.0)
        self.mats = {"steel": steel, "wood": wood, "canvas": canvas, "towel": towel, "bath": bath, "rubber": rubber,
                     "hide": hide}
        for it in spec_items:
            self.obs.append(self.build(it))

    def mesh(self, name, build, mat):
        bm = bmesh.new()
        build(bm)
        me = bpy.data.meshes.new(name)
        bm.to_mesh(me)
        bm.free()
        me.materials.append(mat)
        ob = bpy.data.objects.new(name, me)
        self.coll.objects.link(ob)
        vg = ob.vertex_groups.new(name="hull")         # the armour's outline hull reads this group's weights
        vg.add(range(len(me.vertices)), 1.0, "REPLACE")
        am.add_hull(ob, {"outline": am.mat_outline()}, "default")
        return ob

    def curve(self, name, points, radius, mat):
        """A bevelled poly curve of `points` points (a strand, a strap, a crank), moved point by point; a point's
        `radius` scales the bevel there (a towel flattened over a bar)."""
        cd = bpy.data.curves.new(name, "CURVE")
        cd.dimensions = "3D"
        cd.bevel_depth = radius * am.U
        cd.bevel_resolution = 2
        sp = cd.splines.new("POLY")
        sp.points.add(points - 1)
        ob = bpy.data.objects.new(name, cd)
        ob.data.materials.append(mat)
        self.coll.objects.link(ob)
        return ob

    def rod(self, name, r, length, mat, segments=12):
        """A rigid cylinder along its local Z from 0 to `length` cm."""
        U = am.U
        return self.mesh(name, lambda bm: bmesh.ops.create_cone(
            bm, cap_ends=True, segments=segments, radius1=r * U, radius2=r * U, depth=length * U,
            matrix=Matrix.Translation((0, 0, length * U / 2))), mat)

    def dumbbell(self, name):
        M, U = self.mats, am.U
        handle = self.mesh(name, lambda bm: bmesh.ops.create_cone(
            bm, cap_ends=True, segments=10, radius1=1.6 * U, radius2=1.6 * U, depth=14 * U), M["steel"])
        for i, z in enumerate((-I.DB_HEAD_AT, I.DB_HEAD_AT)):
            head = self.mesh(f"{name}_head{i}", lambda bm, z=z: bmesh.ops.create_cone(
                bm, cap_ends=True, segments=6, radius1=I.DB_HEAD_R * U, radius2=I.DB_HEAD_R * U, depth=7 * U,
                matrix=Matrix.Translation((0, 0, z * U))), M["steel"])
            head.parent = handle
        return handle

    def box(self, name, size, mat):
        return self.mesh(name, lambda bm: bmesh.ops.create_cube(
            bm, size=1.0, matrix=Matrix.Diagonal((size[0] * am.U, size[1] * am.U, size[2] * am.U, 1))), mat)

    def build(self, it):
        k, M, U = it["kind"], self.mats, am.U
        if k == "barbell":
            bar = self.mesh("item_bar", lambda bm: bmesh.ops.create_cone(
                bm, cap_ends=True, segments=12, radius1=1.4 * U, radius2=1.4 * U, depth=2 * I.BAR_HALF * U),
                M["steel"])
            plates = [self.mesh(f"item_plate{i}", lambda bm, z=z: bmesh.ops.create_cone(
                bm, cap_ends=True, segments=32, radius1=I.PLATE_R * U, radius2=I.PLATE_R * U, depth=3.5 * U,
                matrix=Matrix.Translation((0, 0, z * U))), M["canvas"])
                for i, z in enumerate((-I.PLATE_AT, I.PLATE_AT))]
            for p in plates:
                p.parent = bar
            return bar
        if k == "kettlebell":
            bell = self.mesh("item_bell", lambda bm: bmesh.ops.create_uvsphere(
                bm, u_segments=20, v_segments=12, radius=I.BELL_R * U), M["steel"])
            handle = self.curve("item_bell_handle", I.BELL_POINTS, I.BELL_HANDLE_R, M["steel"])
            return {"bell": bell, "handle": handle}
        if k == "backpack":
            pack = self.box("item_pack", (I.PACK_W, I.PACK_D, I.PACK_H), M["canvas"])
            if it.get("on") == "back":                  # two straps over the shoulders, as a pack is worn
                return {"pack": pack, "straps": [self.curve(f"item_pack_strap{i}", 3, 1.8, M["hide"])
                                                 for i in range(2)]}
            if not it.get("on"):                        # the top strap from the pack's top up into the fist
                return {"pack": pack, "handle": self.curve("item_pack_handle", 3, 1.3, M["hide"])}
            return {"pack": pack}
        if k == "rope":
            return self.curve("item_rope", I.ROPE_POINTS, I.ROPE_THICK, M["hide"])
        if k == "wheel":
            wheel = self.mesh("item_wheel", lambda bm: bmesh.ops.create_cone(
                bm, cap_ends=True, segments=28, radius1=9 * U, radius2=9 * U, depth=5 * U), M["hide"])
            axle = self.mesh("item_axle", lambda bm: bmesh.ops.create_cone(
                bm, cap_ends=True, segments=10, radius1=1.5 * U, radius2=1.5 * U, depth=30 * U), M["wood"])
            axle.parent = wheel
            return wheel
        if k == "towel":
            return self.mesh("item_towel", lambda bm: bmesh.ops.create_cube(bm, size=1.0), M["towel"])
        if k == "dumbbell":
            n = 1 if it.get("grip") == "goblet" or I.hand_of(it) != "both" else 2
            return [self.dumbbell(f"item_db{i}") for i in range(n)]
        if k == "ball":
            r = it.get("r", I.BALL_R)
            return self.mesh("item_ball", lambda bm: bmesh.ops.create_uvsphere(
                bm, u_segments=24, v_segments=14, radius=r * U), M["hide"])
        if k == "vest":
            plate = (I.VEST_W, I.VEST_T, I.VEST_H)
            front = self.box("item_vest_front", plate, M["canvas"])
            back = self.box("item_vest_back", plate, M["canvas"])
            straps = [self.curve(f"item_vest_strap{i}", 3, 1.6, M["canvas"]) for i in range(2)]
            return {"front": front, "back": back, "straps": straps}
        if k == "band":
            return [self.curve(f"item_band{i}", 2, I.BAND_R, M["rubber"]) for i in range(len(it["strands"]))]
        if k == "landmine":
            bar = self.rod("item_landmine", 1.4, it["len"] + 6.0, M["steel"])
            plate = self.mesh("item_landmine_plate", lambda bm: bmesh.ops.create_cone(
                bm, cap_ends=True, segments=32, radius1=17 * U, radius2=17 * U, depth=3.5 * U), M["canvas"])
            return {"bar": bar, "plate": plate}
        if k == "pedals":
            arms = [self.curve(f"item_crank{i}", 2, 1.6, M["steel"]) for i in range(2)]
            pedals = [self.box(f"item_pedal{i}", (10, 8, 2.2), M["steel"]) for i in range(2)]
            hub = self.mesh("item_crank_hub", lambda bm: bmesh.ops.create_cone(
                bm, cap_ends=True, segments=40, radius1=11 * U, radius2=11 * U, depth=1.2 * U), M["steel"])
            return {"arms": arms, "pedals": pedals, "hub": hub}
        if k == "straps":
            lines = [self.curve(f"item_strap{i}", 2, 1.3, M["hide"]) for i in range(2)]
            rings = [self.mesh(f"item_ring{i}", lambda bm: bmesh.ops.create_cone(
                bm, cap_ends=True, segments=12, radius1=1.6 * U, radius2=1.6 * U, depth=12 * U), M["wood"])
                for i in range(2)]
            return {"lines": lines, "rings": rings}
        if k == "doortowel":                            # its lines' curves come with the first pose (place)
            knot = self.mesh("item_towel_knot", lambda bm: bmesh.ops.create_uvsphere(
                bm, u_segments=14, v_segments=8, radius=I.KNOT_R * U), M["bath"]) if "knot" in it else None
            return {"lines": [], "knot": knot}
        if k == "chair":
            back = 1.0 if I.hand_of(it) == "both" else -1.0

            def chair(bm):
                for c, half in I.chair_boxes(back):
                    bmesh.ops.create_cube(bm, size=1.0, matrix=Matrix.Translation(Vector(c) * U) @ Matrix.Diagonal(
                        (2 * half[0] * U, 2 * half[1] * U, 2 * half[2] * U, 1)))
            return self.mesh("item_chair", chair, M["wood"])
        if k == "banner":
            return self.banner(it)
        raise ValueError(k)

    def banner(self, it):
        """A standard (items.py `banner`): the wooden pole along its local Z from its butt, the gold finial (a ball
        under a four-sided spearhead) on its top, the gold crossbar with its knobs along the fly (local X) and the
        cloth hanging from it with its device, each in the frame place_banner gives it. The cloth is the armour's own
        material (`qp_cloth`), which render.py swaps for the grey levels, so the tint carries the player's colour on it
        as on the sash; its hems (under the crossbar, its outer edge, its tails) and its device are the armour's
        gold."""
        M, U = self.mats, am.U
        cloth_mat, gold = bpy.data.materials["qp_cloth"], bpy.data.materials["qp_gold"]
        length = it.get("len", I.BANNER_LEN)
        pole = self.rod("item_banner_pole", I.BANNER_POLE_R, length, M["wood"], segments=14)

        def finial(bm):
            bmesh.ops.create_uvsphere(bm, u_segments=16, v_segments=10, radius=3.4 * U,
                                      matrix=Matrix.Translation((0, 0, 3.0 * U)))
            bmesh.ops.create_cone(bm, cap_ends=True, segments=4, radius1=3.6 * U, radius2=0.0,
                                  depth=(I.BANNER_FINIAL - 5.0) * U,
                                  matrix=Matrix.Translation((0, 0, (5.0 + (I.BANNER_FINIAL - 5.0) / 2) * U)))
            bmesh.ops.create_cone(bm, cap_ends=True, segments=14, radius1=2.9 * U, radius2=2.9 * U,
                                  depth=I.BANNER_CAP * U, matrix=Matrix.Translation((0, 0, -I.BANNER_CAP / 2 * U)))
        top = self.mesh("item_banner_finial", finial, gold)
        width = it.get("width", I.BANNER_WIDTH)

        def crossbar(bm):
            bmesh.ops.create_cone(bm, cap_ends=True, segments=12, radius1=1.5 * U, radius2=1.5 * U,
                                  depth=(width + 8.0) * U, matrix=Matrix.Translation(((width / 2 + 1.0) * U, 0, 0))
                                  @ Matrix.Rotation(math.pi / 2, 4, "Y"))
            for x in (-4.0, width + 5.0):
                bmesh.ops.create_uvsphere(bm, u_segments=12, v_segments=8, radius=2.6 * U,
                                          matrix=Matrix.Translation((x * U, 0, 0)))
        bar = self.mesh("item_banner_bar", crossbar, gold)
        drop = it.get("drop", I.BANNER_DROP)
        x0 = I.BANNER_POLE_R + 0.4                     # the cloth's inner edge, at the pole's side

        def ripple(x, z):
            """The cloth's still ripple out of its plane (cm) at x along the fly and z down: nothing at the pole,
            growing toward the free edge, so it reads as cloth, never a board."""
            s = (x - x0) / (width - x0)
            return 1.9 * math.sin(2.0 * math.pi * 1.3 * s + 0.9 * z / drop) * s ** 0.8 + 1.4 * math.sin(math.pi * s)
        cloth = self.banner_cloth(width, drop, it.get("notch", I.BANNER_NOTCH), x0, ripple, cloth_mat, gold)
        emblem = self.banner_emblem(width, x0, ripple, gold)
        return {"pole": pole, "finial": top, "bar": bar, "cloth": cloth, "emblem": emblem}

    def banner_cloth(self, width, drop, notch, x0, ripple, cloth_mat, gold):
        """The cloth in its local frame (cm x U): x from the pole's edge `x0` to `width` along the fly, z down from the
        crossbar to each column's foot (the tails `drop` long, the notch between them `notch` up), its still `ripple`
        out of its plane. A gold hem `hem` cm wide runs under the crossbar, down the outer edge and along the tails."""
        U, hem = am.U, 2.6
        xs = [x0 + (width - hem - x0) * j / 12 for j in range(13)] + [width]
        rows = 22
        verts, faces, mats, grad = [], [], [], []
        for x in xs:
            s = (x - x0) / (width - x0)
            foot = drop - notch * (1.0 - abs(2.0 * s - 1.0))
            zs = [0.0, hem] + [hem + (foot - 2 * hem) * i / rows for i in range(1, rows)] + [foot - hem, foot]
            for z in zs:
                verts.append((x * U, ripple(x, z) * U, -z * U))
                grad.append(1.0 - 0.5 * z / drop)
        n = rows + 3
        for j in range(len(xs) - 1):
            for i in range(n - 1):
                a = j * n + i
                faces.append((a, a + n, a + n + 1, a + 1))
                mats.append(1 if i == 0 or i == n - 2 or j == len(xs) - 2 else 0)
        me = bpy.data.meshes.new("item_banner_cloth")
        me.from_pydata(verts, [], faces)
        me.materials.append(cloth_mat)
        me.materials.append(gold)
        me.polygons.foreach_set("material_index", mats)
        me.shade_smooth()
        at = me.attributes.new("grad", "FLOAT", "POINT")
        at.data.foreach_set("value", grad)
        ob = bpy.data.objects.new("item_banner_cloth", me)
        self.coll.objects.link(ob)
        th = ob.modifiers.new("cloth_thickness", "SOLIDIFY")
        th.thickness = 0.6 * U
        th.offset = 0.0
        vg = ob.vertex_groups.new(name="hull")
        vg.add(list(range(len(verts))), 1.0, "REPLACE")
        am.add_hull(ob, {"outline": am.mat_outline()}, "sash")
        return ob

    def banner_emblem(self, width, x0, ripple, gold):
        """The standard's device, two gold chevrons pointing up (the days a streak stacks, as a rank's stripes), in the
        cloth's upper middle on both its faces, each band 0.4 cm thick with the armour's outline, following the
        cloth's ripple OFF cm off its middle, so it reads from either side; in the cloth's local frame (place_banner).
        OFF clears the cloth's outline hull (its 0.3 cm half thickness plus the hull, 1.4 px at render.py's cell, under
        0.9 cm for a camera up to 4 m across): inside the hull the device stood in its shadow and rendered dark."""
        U, OFF = am.U, 1.4
        cx, half, slope, band = (x0 + width) / 2, 0.26 * (width - x0), 0.72, 4.2
        verts, faces = [], []
        for apex in (17.0, 28.0):                      # cm under the crossbar: the upper chevron, then the lower
            for face in (-1.0, 1.0):
                start = len(verts)
                steps = 16
                for k in range(steps + 1):
                    x = cx - half + 2 * half * k / steps
                    z = apex + slope * abs(x - cx)
                    for dz in (0.0, band):
                        verts.append((x * U, (ripple(x, z + dz) + face * OFF) * U, -(z + dz) * U))
                for k in range(steps):
                    a = start + 2 * k
                    faces.append((a, a + 2, a + 3, a + 1) if face > 0 else (a, a + 1, a + 3, a + 2))
        me = bpy.data.meshes.new("item_banner_emblem")
        me.from_pydata(verts, [], faces)
        me.materials.append(gold)
        me.shade_smooth()
        at = me.attributes.new("grad", "FLOAT", "POINT")
        at.data.foreach_set("value", [0.9] * len(verts))
        ob = bpy.data.objects.new("item_banner_emblem", me)
        self.coll.objects.link(ob)
        th = ob.modifiers.new("emblem_thickness", "SOLIDIFY")
        th.thickness = 0.4 * U
        th.offset = 0.0
        vg = ob.vertex_groups.new(name="hull")
        vg.add(list(range(len(verts))), 1.0, "REPLACE")
        am.add_hull(ob, {"outline": am.mat_outline()}, "sash")
        return ob

    def place_banner(self, ob, pose):
        """The standard in its pose: the pole and the finial along its axis, the crossbar and the cloth in the frame
        of the fly (local X), the axis (local Z) and their normal, made right-handed in Blender's axes (the side view
        swaps two of the body's), so the cloth flies where the pose says."""
        ob["pole"].matrix_world = Matrix.Translation(to_bl(pose["butt"], self.view)) @ self.along(pose["axis"])
        ob["finial"].matrix_world = Matrix.Translation(to_bl(pose["top"], self.view)) @ self.along(pose["axis"])
        x = (to_bl(pose["fly"], self.view) / am.U).normalized()
        z = (to_bl(pose["axis"], self.view) / am.U).normalized()
        R = Matrix((x, z.cross(x), z)).transposed().to_4x4()
        frame = Matrix.Translation(to_bl(pose["bar"], self.view)) @ R
        for part in ("bar", "cloth", "emblem"):
            ob[part].matrix_world = frame

    def lat(self):
        return Vector((0, 1, 0)) if self.view == "side" else Vector((1, 0, 0))

    def along(self, v):
        """The rotation that turns local Z onto the body-space direction v."""
        return (to_bl(v, self.view) / am.U).normalized().to_track_quat("Z", "Y").to_matrix().to_4x4()

    def framed(self, center, axes):
        """A box's world matrix from its centre and its three body-space axes (local X, Y, Z); a mirrored frame (the
        side view swaps two axes) flips X, which a box does not show."""
        cols = [(to_bl(a, self.view) / am.U).normalized() for a in axes]
        if cols[0].cross(cols[1]).dot(cols[2]) < 0:
            cols[0] = -cols[0]
        R = Matrix((cols[0], cols[1], cols[2])).transposed().to_4x4()
        return Matrix.Translation(to_bl(center, self.view)) @ R

    def line(self, ob, pts, radii=None):
        for n, (p, q) in enumerate(zip(ob.data.splines[0].points, pts)):
            p.co = (*to_bl(q, self.view), 1.0)
            if radii:
                p.radius = radii[n] / radii[0]

    def place(self, poses):
        V = lambda p: to_bl(p, self.view)
        for ob, pose in zip(self.obs, poses):
            k = pose["kind"]
            if k == "barbell":
                c, ax = V(pose["center"]), (V(pose["axis"]) / am.U).normalized()
                ob.matrix_world = Matrix.Translation(c) @ ax.to_track_quat("Z", "Y").to_matrix().to_4x4()
            elif k == "kettlebell":
                ob["bell"].matrix_world = Matrix.Translation(V(pose["bell"]))
                self.line(ob["handle"], pose["line"])
            elif k == "backpack":
                ob["pack"].matrix_world = self.framed(pose["center"], pose["axes"])
                if "straps" in ob:
                    lat, fwd, up = pose["axes"]
                    for strap, s in zip(ob["straps"], (-1.0, 1.0)):
                        top = tuple(c + u * (I.PACK_H / 2 - 2.0) + l * s * 9.0 + f * 7.0 for c, u, l, f in
                                    zip(pose["center"], up, lat, fwd))
                        over = tuple(t + u * 8.0 + f * 16.0 for t, u, f in zip(top, up, fwd))
                        front = tuple(o + f * 18.0 - u * 16.0 for o, f, u in zip(over, fwd, up))
                        self.line(strap, [top, over, front])
                if "handle" in ob:                      # a loop across the top's width, through the fist
                    w = [x * I.PACK_LOOP for x in pose["axes"][0]]
                    self.line(ob["handle"], [tuple(t - x for t, x in zip(pose["top"], w)), pose["handle"],
                                             tuple(t + x for t, x in zip(pose["top"], w))])
            elif k == "rope":
                self.line(ob, pose["points"])
            elif k == "wheel":
                R = self.lat().to_track_quat("Z", "Y").to_matrix().to_4x4()
                ob.matrix_world = Matrix.Translation(V(pose["center"])) @ R
            elif k == "towel":
                w = 46 if pose["wide"] else 26
                sx, sy = (30, w) if self.view == "side" else (w, 30)
                ob.matrix_world = Matrix.Translation(V(pose["center"])) @ Matrix.Diagonal(
                    (sx * am.U, sy * am.U, 1.0 * am.U, 1))
            elif k == "dumbbell":
                for d, b in zip(ob, pose["bells"]):
                    d.matrix_world = Matrix.Translation(V(b["center"])) @ self.along(b["axis"])
            elif k == "ball":
                ob.matrix_world = Matrix.Translation(V(pose["center"]))
            elif k == "vest":
                lat, fwd, up = pose["axes"]
                back_up = pose["back_axes"][2]
                ob["front"].matrix_world = self.framed(pose["front"], pose["axes"])
                ob["back"].matrix_world = self.framed(pose["back"], pose["back_axes"])
                for i, s in enumerate((-1.0, 1.0)):
                    side = tuple(x * s * 9.0 for x in lat)
                    f = tuple(a + b + u * I.VEST_H / 2 for a, b, u in zip(pose["front"], side, up))
                    b = tuple(a + b + u * I.VEST_H / 2 for a, b, u in zip(pose["back"], side, back_up))
                    over = tuple((x + y) / 2 + u * 7.0 for x, y, u in zip(f, b, up))
                    self.line(ob["straps"][i], [f, over, b])
            elif k == "band":
                for c, (a, b) in zip(ob, pose["strands"]):
                    self.line(c, [a, b])
            elif k == "landmine":
                ob["bar"].matrix_world = Matrix.Translation(V(pose["pivot"])) @ self.along(pose["axis"])
                ob["plate"].matrix_world = Matrix.Translation(V(pose["plate"])) @ self.along(pose["axis"])
            elif k == "pedals":
                near = pose["cranks"][0]
                # the chainring on the near side, the cranks just outside the feet, so the circle reads in profile
                ob["hub"].matrix_world = Matrix.Translation(V(near["hub"])) @ self.lat().to_track_quat(
                    "Z", "Y").to_matrix().to_4x4()
                for arm, ped, c in zip(ob["arms"], ob["pedals"], pose["cranks"]):
                    out = c["pedal"][2] + (-3.0 if c["side"] == "near" else 3.0)
                    self.line(arm, [(c["hub"][0], c["hub"][1], out), (c["pedal"][0], c["pedal"][1], out)])
                    ped.matrix_world = Matrix.Translation(V(c["pedal"]))
            elif k == "straps":
                for ln, ring, s in zip(ob["lines"], ob["rings"], pose["lines"]):
                    self.line(ln, [s["anchor"], s["ring"]])
                    ring.matrix_world = Matrix.Translation(V(s["ring"])) @ self.lat().to_track_quat(
                        "Z", "Y").to_matrix().to_4x4()
            elif k == "doortowel":
                if not ob["lines"]:
                    ob["lines"] = [self.curve(f"item_towel_line{i}", len(pts), radii[0], self.mats["bath"])
                                   for i, (pts, radii) in enumerate(zip(pose["lines"], pose["radii"]))]
                for c, pts, radii in zip(ob["lines"], pose["lines"], pose["radii"]):
                    self.line(c, pts, radii)
                if ob["knot"]:
                    ob["knot"].matrix_world = Matrix.Translation(V(pose["knot"]))
            elif k == "chair":
                ob.matrix_world = self.framed(pose["center"], pose["axes"])
            elif k == "banner":
                self.place_banner(ob, pose)


# ---------------------------------------------------------------- lights

def sun(scene, name, src, energy, angle_deg, coll):
    ld = bpy.data.lights.new(name, "SUN")
    ld.energy = energy
    ld.angle = math.radians(angle_deg)
    ld.use_shadow = True
    ld.use_shadow_jitter = True
    ld.shadow_jitter_overblur = 0.0
    ld.shadow_filter_radius = 1.0
    ob = bpy.data.objects.new(name, ld)
    scene.collection.objects.link(ob)
    ob.rotation_euler = (-Vector(src)).normalized().to_track_quat("-Z", "Y").to_euler()
    ob.light_linking.receiver_collection = coll
    return ob


def cam_basis(scene):
    R = scene.camera.matrix_world.to_3x3()
    return R.col[0].copy(), R.col[1].copy(), R.col[2].copy()      # right, up, back (toward the camera)
