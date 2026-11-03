"""plate_kit: stylised hero plate armor, built procedurally for Blender 5.2 LTS (headless, EEVEE).

Every armor piece is an open quad *shell* (the cage) run through one modifier stack:
    Subdivision -> Solidify (plate thickness, rim gets material slot 1) -> Bevel (angle-limited, 3 segments,
    bevel faces get material slot 2 = polished edge) -> Geometry Nodes edge mask ("edge" point attribute).
A linked twin of the cage carries a Geometry Nodes modifier that turns every open border of the plate into a
rolled gold edge (Mesh to Curve -> Curve to Mesh with a circle profile). Rivets are Instance on Points.
Parts hang under one Empty per rig bone, so posing sets one matrix per part and nothing is joined.

Demo:  blender -b --factory-startup -P plate_kit.py -- <out.png> [res]
Units: metres, figure 1.8 m tall. At 384 px over a 2.09 m ortho frame one pixel is ~5.4 mm, so no detail here is
smaller than ~2 px at 384 (bevels 6-8 mm, rolled edges 10-16 mm radius, rivets 7-10 mm).
API names checked against docs.blender.org/api/5.2 and the blender-v5.2-release sources; not executed.
"""
import math
import sys

import bmesh
import bpy
from mathutils import Matrix, Vector

EXPOSURE = 0.40          # light value -> 0..1 band input (as in armor.py)


# ================================================================ colour and materials

def hexc(h):
    """#RRGGBB (sRGB) -> linear RGB tuple."""
    h = h.lstrip("#")
    c = [int(h[i:i + 2], 16) / 255 for i in (0, 2, 4)]
    return tuple(((x + 0.055) / 1.055) ** 2.4 if x > 0.04045 else x / 12.92 for x in c)


def _clear(m):
    """5.x: bpy.data.materials.new() already carries a node tree (Material.use_nodes is deprecated)."""
    nt = m.node_tree
    for n in list(nt.nodes):
        if n.type != "OUTPUT_MATERIAL":
            nt.nodes.remove(n)
    return nt, next(n for n in nt.nodes if n.type == "OUTPUT_MATERIAL")


def _ramp(nt, stops, interp="LINEAR"):
    """Colour ramp from [(position, rgb)], robust to the element list re-sorting itself."""
    r = nt.nodes.new("ShaderNodeValToRGB")
    cr = r.color_ramp
    cr.interpolation = interp
    cr.elements[0].position = stops[0][0]
    cr.elements[-1].position = stops[-1][0]
    for pos, _ in stops[1:-1]:
        cr.elements.new(pos)
    by_pos = {round(p, 3): c for p, c in stops}
    for e in cr.elements:
        c = by_pos[round(e.position, 3)]
        e.color = (*c, 1.0)
    return r


def _math(nt, op, a=None, b=None):
    n = nt.nodes.new("ShaderNodeMath")
    n.operation = op
    if a is not None:
        n.inputs[0].default_value = a
    if b is not None:
        n.inputs[1].default_value = b
    return n


def _mix(nt, blend, fac=1.0):
    n = nt.nodes.new("ShaderNodeMix")
    n.data_type = "RGBA"
    n.blend_type = blend
    n.inputs["Factor"].default_value = fac
    return n


def mat_metal(name, shadow, mid, light, spec, rim="#9FC2FF", spec_amt=0.85, rim_amt=0.45, edge_amt=0.55,
              z_lo=0.0, z_hi=1.9, use_ao=False):
    """Hand-painted metal for EEVEE, output as emission so the palette is exact:
    three crisp cel bands (Shader to RGB) x a painted top light (world height) [x baked AO]
    + a painted chrome horizon band (reflection vector height) + a polished-edge lift from the GN "edge" attribute
    + a cool rim from the facing term. Every term is a hard value break, which is what reads as crafted plate."""
    m = bpy.data.materials.new(name)
    nt, out = _clear(m)
    N, L = nt.nodes.new, nt.links.new
    S, M, H, SP = hexc(shadow), hexc(mid), hexc(light), hexc(spec)
    # 1. cel bands from the lit value: narrow 6 % transitions stay crisp without stair-stepping
    dif = N("ShaderNodeBsdfDiffuse")
    s2r = N("ShaderNodeShaderToRGB")
    bw = N("ShaderNodeRGBToBW")
    ex = _math(nt, "MULTIPLY", b=EXPOSURE)
    bands = _ramp(nt, [(0.0, S), (0.30, S), (0.36, M), (0.62, M), (0.68, H), (1.0, H)])
    L(dif.outputs["BSDF"], s2r.inputs["Shader"])
    L(s2r.outputs["Color"], bw.inputs["Color"])
    L(bw.outputs["Val"], ex.inputs[0])
    L(ex.outputs["Value"], bands.inputs["Fac"])
    # 2. painted top light: darker at the feet, brighter at the helm
    geo = N("ShaderNodeNewGeometry")
    sep = N("ShaderNodeSeparateXYZ")
    hz = N("ShaderNodeMapRange")
    hz.inputs["From Min"].default_value = z_lo
    hz.inputs["From Max"].default_value = z_hi
    hz.inputs["To Min"].default_value = 0.72
    hz.inputs["To Max"].default_value = 1.12
    L(geo.outputs["Position"], sep.inputs["Vector"])
    L(sep.outputs["Z"], hz.inputs["Value"])
    base = _mix(nt, "MULTIPLY")
    L(bands.outputs["Color"], base.inputs["A"])
    L(hz.outputs["Result"], base.inputs["B"])
    cur = base.outputs["Result"]
    # 2b. optional baked ambient occlusion (colour attribute "ao", see bake_to_color): A * (0.55 + 0.45 * ao)
    if use_ao:
        ca = N("ShaderNodeVertexColor")
        ca.layer_name = "ao"
        aom = _mix(nt, "MULTIPLY", fac=0.45)
        L(cur, aom.inputs["A"])
        L(ca.outputs["Color"], aom.inputs["B"])
        cur = aom.outputs["Result"]
    # 3. painted chrome: reflection Z in [-1, 1] -> dark ground, hot horizon line, soft sky
    tc = N("ShaderNodeTexCoord")
    rs = N("ShaderNodeSeparateXYZ")
    rz = N("ShaderNodeMapRange")
    rz.inputs["From Min"].default_value = -1.0
    rz.inputs["From Max"].default_value = 1.0
    chrome = _ramp(nt, [(0.0, (0, 0, 0)), (0.47, (0, 0, 0)), (0.52, (1, 1, 1)), (0.62, (0.22, 0.22, 0.22)),
                        (1.0, (0.4, 0.4, 0.4))])
    L(tc.outputs["Reflection"], rs.inputs["Vector"])
    L(rs.outputs["Z"], rz.inputs["Value"])
    L(rz.outputs["Result"], chrome.inputs["Fac"])
    ch_amt = _math(nt, "MULTIPLY", b=spec_amt)
    L(chrome.outputs["Color"], ch_amt.inputs[0])
    spec_add = _mix(nt, "ADD")
    spec_add.inputs["B"].default_value = (*SP, 1)
    L(cur, spec_add.inputs["A"])
    L(ch_amt.outputs["Value"], spec_add.inputs["Factor"])
    # 4. polished edges: the GN edge mask (0 where the attribute is missing, so this is always safe)
    at = N("ShaderNodeAttribute")
    at.attribute_type = "GEOMETRY"
    at.attribute_name = "edge"
    e_amt = _math(nt, "MULTIPLY", b=edge_amt)
    L(at.outputs["Fac"], e_amt.inputs[0])
    edge_add = _mix(nt, "ADD")
    edge_add.inputs["B"].default_value = (*SP, 1)
    L(spec_add.outputs["Result"], edge_add.inputs["A"])
    L(e_amt.outputs["Value"], edge_add.inputs["Factor"])
    # 5. cool rim
    lw = N("ShaderNodeLayerWeight")
    lw.inputs["Blend"].default_value = 0.35
    rr = N("ShaderNodeMapRange")
    rr.inputs["From Min"].default_value = 0.55
    rr.inputs["From Max"].default_value = 0.9
    rr.inputs["To Max"].default_value = rim_amt
    L(lw.outputs["Facing"], rr.inputs["Value"])
    rim_add = _mix(nt, "ADD")
    rim_add.inputs["B"].default_value = (*hexc(rim), 1)
    L(edge_add.outputs["Result"], rim_add.inputs["A"])
    L(rr.outputs["Result"], rim_add.inputs["Factor"])
    em = N("ShaderNodeEmission")
    L(rim_add.outputs["Result"], em.inputs["Color"])
    L(em.outputs["Emission"], out.inputs["Surface"])
    return m


def mat_glow(name, on=(1.0, 0.74, 0.12), off=None, strength=6.0):
    """Emissive light. With `off` set, the per-object custom property ob["glow"] (0..1, read by an Attribute node
    of type OBJECT) blends from the dim `off` colour to the bright `on` light, so runes toggle per frame by setting
    one float instead of swapping materials."""
    m = bpy.data.materials.new(name)
    nt, out = _clear(m)
    N, L = nt.nodes.new, nt.links.new
    em = N("ShaderNodeEmission")
    if off is None:
        em.inputs["Color"].default_value = (*on, 1)
        em.inputs["Strength"].default_value = strength
    else:
        at = N("ShaderNodeAttribute")
        at.attribute_type = "OBJECT"
        at.attribute_name = "glow"
        col = _mix(nt, "MIX")
        col.inputs["A"].default_value = (*off, 1)
        col.inputs["B"].default_value = (*on, 1)
        L(at.outputs["Fac"], col.inputs["Factor"])
        st = N("ShaderNodeMapRange")
        st.inputs["To Min"].default_value = 1.0
        st.inputs["To Max"].default_value = strength
        L(at.outputs["Fac"], st.inputs["Value"])
        L(col.outputs["Result"], em.inputs["Color"])
        L(st.outputs["Result"], em.inputs["Strength"])
    L(em.outputs["Emission"], out.inputs["Surface"])
    return m


def palette():
    return {
        "steel": mat_metal("steel", "#1E2640", "#4B5A82", "#93A5CC", "#E8EEFA"),
        "steel_hi": mat_metal("steel_hi", "#4B5A82", "#93A5CC", "#D5DEF2", "#FFFFFF", spec_amt=1.0),
        "steel_dark": mat_metal("steel_dark", "#11152A", "#2A3352", "#56627F", "#A9B3CC", rim_amt=0.35),
        "gold": mat_metal("gold", "#4A2A0C", "#A0701E", "#E3B54B", "#FFF3BF", rim="#FFE7A0", rim_amt=0.3,
                          spec_amt=1.0),
        "gold_hi": mat_metal("gold_hi", "#A0701E", "#E3B54B", "#FFE08A", "#FFFFFF", rim="#FFE7A0", rim_amt=0.3),
        "leather": mat_metal("leather", "#1C0F07", "#402512", "#6E4424", "#9C6A3E", rim_amt=0.2, spec_amt=0.1,
                             edge_amt=0.15),
        "visor": mat_glow("visor", on=(1.0, 0.86, 0.45), strength=7.0),
        "rune": mat_glow("rune", on=(1.0, 0.74, 0.12), off=(0.35, 0.22, 0.05), strength=6.0),
    }


# ================================================================ scene helpers

def link(ob):
    bpy.context.scene.collection.objects.link(ob)
    return ob


def part_root(name):
    """One Empty per rig bone: pose it with matrix_world; every plate, trim and rivet of the part is its child."""
    e = link(bpy.data.objects.new(name, None))
    e.empty_display_size = 0.05
    return e


def adopt(root, *obs):
    for o in obs:
        o.parent = root          # both built at the origin, so the child's local transform is its own transform
    return root


def shell(name, f, nu, nv, cyclic=False, cap=None, inside=(0, 0, 0), skip=None, post=None):
    """Open quad shell from a parametric surface f(u, v) -> (x, y, z), u and v in [0, 1].
    cyclic: u wraps around. cap="v0"/"v1" closes that ring with one n-gon. skip(i, j) -> True leaves face (i, j)
    out (holes such as a visor slot). post(bm, faces) edits the cage with bmesh (inset panels, ridges, support
    loops, creases); faces[j][i] is face (i, j) or None. Faces end up pointing away from `inside`."""
    bm = bmesh.new()
    cols = nu if cyclic else nu + 1
    g = [[bm.verts.new(f(i / nu, j / nv)) for i in range(cols)] for j in range(nv + 1)]
    faces = [[None] * nu for _ in range(nv)]
    for j in range(nv):
        for i in range(nu):
            if skip and skip(i, j):
                continue
            a, b = i, (i + 1) % cols
            faces[j][i] = bm.faces.new((g[j][a], g[j][b], g[j + 1][b], g[j + 1][a]))
    if cap and cyclic:
        bm.faces.new(g[0] if cap == "v0" else g[nv])
    loose = [v for v in bm.verts if not v.link_faces]
    if loose:
        bmesh.ops.delete(bm, geom=loose, context="VERTS")
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:])
    bm.normal_update()
    c = Vector(inside)
    if sum(fc.normal.dot(fc.calc_center_median() - c) for fc in bm.faces) < 0:
        bmesh.ops.reverse_faces(bm, faces=bm.faces[:])
        bm.normal_update()
    if post:                                  # normals already point outward here
        post(bm, faces)
        bm.normal_update()
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me)
    bm.free()
    me.shade_smooth()
    return link(bpy.data.objects.new(name, me))


def blob(name, size, loc=(0, 0, 0), mat=None, bevel=0.012):
    """Rounded box (leather under-glove, straps)."""
    bm = bmesh.new()
    bmesh.ops.create_cube(bm, size=1.0, matrix=Matrix.Translation(loc) @ Matrix.Diagonal((*size, 1.0)))
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me)
    bm.free()
    me.shade_smooth()
    ob = link(bpy.data.objects.new(name, me))
    if mat:
        me.materials.append(mat)
    bv = ob.modifiers.new("round", "BEVEL")
    bv.width = bevel
    bv.segments = 3
    bv.limit_method = "NONE"
    bv.use_clamp_overlap = True
    return ob


def ellipsoid(name, radii, loc=(0, 0, 0), mat=None, segs=24):
    bm = bmesh.new()
    bmesh.ops.create_uvsphere(bm, u_segments=segs, v_segments=segs // 2, radius=1.0,
                              matrix=Matrix.Translation(loc) @ Matrix.Diagonal((*radii, 1.0)))
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me)
    bm.free()
    me.shade_smooth()
    if mat:
        me.materials.append(mat)
    return link(bpy.data.objects.new(name, me))


# ================================================================ geometry node groups (built once, shared)

def _gn(name):
    ng = bpy.data.node_groups.get(name)
    if ng:
        return ng, False
    ng = bpy.data.node_groups.new(name, "GeometryNodeTree")
    ng.is_modifier = True
    return ng, True


def _sock(ng, name, kind, io="INPUT", default=None):
    s = ng.interface.new_socket(name, in_out=io, socket_type=kind)
    if default is not None:
        s.default_value = default
    return s


def set_input(mod, name, value):
    """Per-modifier value of a group input. 5.2 changed this API: modifier.properties.inputs.<identifier>.value
    (before 5.2: modifier[identifier] = value)."""
    item = next(i for i in mod.node_group.interface.items_tree
                if i.item_type == "SOCKET" and i.in_out == "INPUT" and i.name == name)
    try:
        getattr(mod.properties.inputs, item.identifier).value = value
    except AttributeError:
        mod[item.identifier] = value


def rolled_edge_group():
    """Cage -> same subdivision as the plate -> pull borders half a plate thickness inward -> border edges
    (Edge Neighbors: Face Count < 1.5) -> Mesh to Curve -> Curve to Mesh with a circle profile -> gold."""
    ng, new = _gn("QP_RolledEdge")
    if not new:
        return ng
    _sock(ng, "Geometry", "NodeSocketGeometry")
    _sock(ng, "Level", "NodeSocketInt", default=2)
    _sock(ng, "Radius", "NodeSocketFloat", default=0.013)
    _sock(ng, "Inset", "NodeSocketFloat", default=-0.01)
    _sock(ng, "Material", "NodeSocketMaterial")
    _sock(ng, "Geometry", "NodeSocketGeometry", io="OUTPUT")
    N, L = ng.nodes.new, ng.links.new
    gi, go = N("NodeGroupInput"), N("NodeGroupOutput")
    sub = N("GeometryNodeSubdivisionSurface")
    L(gi.outputs["Geometry"], sub.inputs["Mesh"])
    L(gi.outputs["Level"], sub.inputs["Level"])
    nrm = N("GeometryNodeInputNormal")
    scl = N("ShaderNodeVectorMath")
    scl.operation = "SCALE"
    L(nrm.outputs["Normal"], scl.inputs[0])
    L(gi.outputs["Inset"], scl.inputs["Scale"])
    sp = N("GeometryNodeSetPosition")
    L(sub.outputs["Mesh"], sp.inputs["Geometry"])
    L(scl.outputs["Vector"], sp.inputs["Offset"])
    nb = N("GeometryNodeInputMeshEdgeNeighbors")
    lt = N("ShaderNodeMath")
    lt.operation = "LESS_THAN"
    lt.inputs[1].default_value = 1.5
    L(nb.outputs["Face Count"], lt.inputs[0])
    m2c = N("GeometryNodeMeshToCurve")
    L(sp.outputs["Geometry"], m2c.inputs["Mesh"])
    L(lt.outputs["Value"], m2c.inputs["Selection"])          # float > 0 converts to True
    circ = N("GeometryNodeCurvePrimitiveCircle")
    circ.inputs["Resolution"].default_value = 12
    L(gi.outputs["Radius"], circ.inputs["Radius"])
    c2m = N("GeometryNodeCurveToMesh")
    L(m2c.outputs["Curve"], c2m.inputs["Curve"])
    L(circ.outputs["Curve"], c2m.inputs["Profile Curve"])
    ss = N("GeometryNodeSetShadeSmooth")                     # 5.2: its geometry socket is named "Mesh"
    L(c2m.outputs["Mesh"], ss.inputs["Mesh"])
    sm = N("GeometryNodeSetMaterial")
    L(ss.outputs["Mesh"], sm.inputs["Geometry"])
    L(gi.outputs["Material"], sm.inputs["Material"])
    L(sm.outputs["Geometry"], go.inputs["Geometry"])
    return ng


def rivet_group():
    """Points (any mesh's vertices) -> Instance on Points (UV sphere) -> Realize -> smooth -> material."""
    ng, new = _gn("QP_Rivets")
    if not new:
        return ng
    _sock(ng, "Geometry", "NodeSocketGeometry")
    _sock(ng, "Radius", "NodeSocketFloat", default=0.008)
    _sock(ng, "Material", "NodeSocketMaterial")
    _sock(ng, "Geometry", "NodeSocketGeometry", io="OUTPUT")
    N, L = ng.nodes.new, ng.links.new
    gi, go = N("NodeGroupInput"), N("NodeGroupOutput")
    sph = N("GeometryNodeMeshUVSphere")
    sph.inputs["Segments"].default_value = 12
    sph.inputs["Rings"].default_value = 6
    L(gi.outputs["Radius"], sph.inputs["Radius"])
    iop = N("GeometryNodeInstanceOnPoints")
    L(gi.outputs["Geometry"], iop.inputs["Points"])
    L(sph.outputs["Mesh"], iop.inputs["Instance"])
    real = N("GeometryNodeRealizeInstances")
    L(iop.outputs["Instances"], real.inputs["Geometry"])
    ss = N("GeometryNodeSetShadeSmooth")
    L(real.outputs["Geometry"], ss.inputs["Mesh"])
    sm = N("GeometryNodeSetMaterial")
    L(ss.outputs["Mesh"], sm.inputs["Geometry"])
    L(gi.outputs["Material"], sm.inputs["Material"])
    L(sm.outputs["Geometry"], go.inputs["Geometry"])
    return ng


def edge_mask_group(lo=0.08, hi=0.35):
    """EEVEE has no Bevel or Pointiness node: store convexity as point attribute "edge" instead.
    Edge Angle's Signed Angle is negative on convex edges; read on the point domain it is averaged from the edges."""
    ng, new = _gn("QP_EdgeMask")
    if not new:
        return ng
    _sock(ng, "Geometry", "NodeSocketGeometry")
    _sock(ng, "Geometry", "NodeSocketGeometry", io="OUTPUT")
    N, L = ng.nodes.new, ng.links.new
    gi, go = N("NodeGroupInput"), N("NodeGroupOutput")
    ang = N("GeometryNodeInputMeshEdgeAngle")
    neg = N("ShaderNodeMath")
    neg.operation = "MULTIPLY"
    neg.inputs[1].default_value = -1.0
    L(ang.outputs["Signed Angle"], neg.inputs[0])
    mr = N("ShaderNodeMapRange")
    mr.inputs["From Min"].default_value = lo
    mr.inputs["From Max"].default_value = hi
    L(neg.outputs["Value"], mr.inputs["Value"])
    st = N("GeometryNodeStoreNamedAttribute")
    st.data_type = "FLOAT"                                    # set type and domain before linking "Value"
    st.domain = "POINT"
    st.inputs["Name"].default_value = "edge"
    L(gi.outputs["Geometry"], st.inputs["Geometry"])
    L(mr.outputs["Result"], st.inputs["Value"])
    L(st.outputs["Geometry"], go.inputs["Geometry"])
    return ng


def hammer_group(amp=0.0025, scale=5.0):
    """Optional low-frequency 'hand-made' wobble (Set Position + Noise along the normal). Insert it between the
    Subdivision and Solidify modifiers so the thickness follows. Sub-pixel below ~768 px renders: keep amp small."""
    ng, new = _gn("QP_Hammer")
    if not new:
        return ng
    _sock(ng, "Geometry", "NodeSocketGeometry")
    _sock(ng, "Geometry", "NodeSocketGeometry", io="OUTPUT")
    N, L = ng.nodes.new, ng.links.new
    gi, go = N("NodeGroupInput"), N("NodeGroupOutput")
    pos = N("GeometryNodeInputPosition")
    nz = N("ShaderNodeTexNoise")
    nz.inputs["Scale"].default_value = scale
    nz.inputs["Detail"].default_value = 1.0
    L(pos.outputs["Position"], nz.inputs["Vector"])
    ctr = N("ShaderNodeMath")
    ctr.operation = "MULTIPLY_ADD"                            # (fac * 2 amp) - amp
    ctr.inputs[1].default_value = 2 * amp
    ctr.inputs[2].default_value = -amp
    L(nz.outputs["Fac"], ctr.inputs[0])
    nrm = N("GeometryNodeInputNormal")
    scl = N("ShaderNodeVectorMath")
    scl.operation = "SCALE"
    L(nrm.outputs["Normal"], scl.inputs[0])
    L(ctr.outputs["Value"], scl.inputs["Scale"])
    sp = N("GeometryNodeSetPosition")
    L(gi.outputs["Geometry"], sp.inputs["Geometry"])
    L(scl.outputs["Vector"], sp.inputs["Offset"])
    L(sp.outputs["Geometry"], go.inputs["Geometry"])
    return ng


# ================================================================ the plate stack

def plate(ob, face, rim, edge, thick=0.02, bev=0.007, segs=3, level=2, offset=-1.0, mask=True, hammer=False):
    """Subdivision -> [hammer] -> Solidify (rim -> slot 1) -> Bevel (new faces -> slot 2) -> edge mask."""
    me = ob.data
    for m in (face, rim, edge):
        me.materials.append(m)
    sub = ob.modifiers.new("subdiv", "SUBSURF")
    sub.levels = level
    sub.render_levels = level
    if hammer:
        hm = ob.modifiers.new("hammer", "NODES")
        hm.node_group = hammer_group()
    sol = ob.modifiers.new("thickness", "SOLIDIFY")
    sol.thickness = thick
    sol.offset = offset                   # -1: the outer surface stays where it was modelled
    sol.use_even_offset = True
    sol.use_quality_normals = True
    sol.use_rim = True
    sol.material_offset_rim = 1           # rim band -> material slot 1 (gold or a darker steel)
    bv = ob.modifiers.new("bevel", "BEVEL")
    bv.width = bev
    bv.segments = segs
    bv.profile = 0.7                      # 0.5 = round, above 0.5 = fuller, "rolled" look
    bv.limit_method = "ANGLE"
    bv.angle_limit = math.radians(35)     # only the rims and hard folds, not the smooth subdivided curvature
    bv.use_clamp_overlap = True
    bv.harden_normals = True
    bv.miter_outer = "MITER_ARC"
    bv.material = 2                       # bevel faces -> slot 2 (polished edge material)
    if mask:
        gm = ob.modifiers.new("edge_mask", "NODES")
        gm.node_group = edge_mask_group()
    return ob


def copy_modifier(src, dst_ob):
    """Same type, every writable RNA property copied (object pointers, axis arrays and enums included)."""
    new = dst_ob.modifiers.new(src.name, src.type)
    for p in src.bl_rna.properties:
        if p.is_readonly or p.identifier in {"name", "type", "rna_type", "is_active"}:
            continue
        try:
            setattr(new, p.identifier, getattr(src, p.identifier))
        except (AttributeError, TypeError, ValueError):
            pass
    return new


def add_rolled_edge(root, plate_ob, mat, radius=0.013):
    """Linked twin of the cage (same mesh data, own modifiers): every open border becomes a rolled edge.
    Modifiers stacked before "subdiv" (Mirror, Array, Curve, Shrinkwrap) are repeated so the edge follows the
    plate: a mirrored half gets no edge on its midline, every arrayed lame gets its own."""
    tw = link(bpy.data.objects.new(plate_ob.name + "_edge", plate_ob.data))
    tw.parent = plate_ob.parent
    tw.matrix_basis = plate_ob.matrix_basis.copy()
    for md in plate_ob.modifiers:
        if md.name == "subdiv":
            break
        copy_modifier(md, tw)
    mod = tw.modifiers.new("rolled_edge", "NODES")
    mod.node_group = rolled_edge_group()
    set_input(mod, "Level", plate_ob.modifiers["subdiv"].render_levels)
    set_input(mod, "Radius", radius)
    sol = plate_ob.modifiers["thickness"]
    set_input(mod, "Inset", 0.5 * sol.thickness * sol.offset)   # centre of the rim: -t/2 for offset -1, 0 for 0
    set_input(mod, "Material", mat)
    return tw


def add_rivets(root, name, points, mat, radius=0.008):
    me = bpy.data.meshes.new(name)
    me.from_pydata([tuple(p) for p in points], [], [])
    ob = link(bpy.data.objects.new(name, me))
    mod = ob.modifiers.new("rivets", "NODES")
    mod.node_group = rivet_group()
    set_input(mod, "Radius", radius)
    set_input(mod, "Material", mat)
    adopt(root, ob)
    return ob


# ================================================================ parts

def build_pauldron(side, M, R=0.19, H=0.16, tilt=18.0, shift=0.035):
    """Hero spaulder, 4 big shapes: a domed cop, two flaring lames tucked under it roof-tile style, a rolled gold
    edge on every plate, and a gold boss; plus a rivet row per lame. Local frame: origin at the shoulder joint,
    +Z up the torso, +X forward; outward is -Y for the near side and +Y for the far side. `shift` moves the whole
    pauldron outward so the cop clears the helm in front views (shoulders are only 0.20 m off the midline)."""
    o = -1.0 if side == "near" else 1.0
    rot = Matrix.Rotation(-o * math.radians(tilt), 4, "X")      # lean the dome axis outward over the arm
    k_r, k_h = R / 0.19, H / 0.16                                # lame sizes scale with the cop

    def T(p):                                                    # canonical (outward = +Y) -> part frame
        return rot @ Vector((p[0], o * p[1], p[2])) + Vector((0.0, o * shift, 0.0))

    root = part_root(f"pauldron_{side}")

    def cop(u, v):
        th = 2 * math.pi * u
        ph = math.radians(12 + 88 * v)                           # polar angle 12..100 deg: overhangs the arm
        flare = 1.0 + 0.05 * max(0.0, (v - 0.8) / 0.2) ** 2      # the rim kicks outward
        r = R * math.sin(ph) * flare
        return T((1.08 * r * math.cos(th), r * math.sin(th), H * math.cos(ph)))

    dome = plate(shell(f"pauldron_cop_{side}", cop, 24, 7, cyclic=True, cap="v0"),
                 M["steel"], M["gold"], M["steel_hi"], thick=0.022, bev=0.008)
    adopt(root, dome)
    add_rolled_edge(root, dome, M["gold"], radius=0.015)

    a0, a1 = math.radians(-50), math.radians(230)               # 280 deg around the outside, open to the neck
    z_top = H * math.cos(math.radians(100)) + 0.03 * k_h         # tucked 3 cm up under the cop's rim
    rivets = []
    for k in range(2):
        zt, h = z_top - 0.05 * k_h * k, 0.07 * k_h
        r0 = (0.158 + 0.008 * k) * k_r
        r1 = r0 + 0.05 * k_r

        def band_c(u, v, zt=zt, h=h, r0=r0, r1=r1):
            th = a0 + (a1 - a0) * u
            r = r0 + (r1 - r0) * v
            return (1.08 * r * math.cos(th), r * math.sin(th), zt - h * v), th

        lame = plate(shell(f"pauldron_lame{k}_{side}", lambda u, v, b=band_c: T(b(u, v)[0]), 20, 2,
                           inside=T((0, 0, zt - h / 2))),
                     M["steel"], M["gold"], M["steel_hi"], thick=0.018, bev=0.007)
        adopt(root, lame)
        add_rolled_edge(root, lame, M["gold"], radius=0.012)
        for i in range(7):                                       # rivet row at 75 % of the band height
            (x, y, z), th = band_c(0.07 + 0.86 * i / 6, 0.75)
            rivets.append(T((x + 0.004 * math.cos(th), y + 0.004 * math.sin(th), z)))
    add_rivets(root, f"pauldron_rivets_{side}", rivets, M["gold"], radius=0.008)
    boss = ellipsoid(f"pauldron_boss_{side}", (0.045, 0.045, 0.025), mat=M["gold"])
    boss.matrix_basis = Matrix.Translation(T((0, 0, H - 0.002))) @ rot      # sits on the tilted crown
    adopt(root, boss)
    return root


def build_helm(M):
    """Great helm, 4 big shapes: a boxy bucket with a T-shaped visor slot (glow shows through it), a proud gold
    brow band, a swept gold crest fin, and a rolled gold edge around the neck and the T. Local frame: origin at
    the neck joint, +Z up, +X forward (the rig's 'head' part)."""
    root = part_root("head")
    Z = [0.03, 0.06, 0.09, 0.12, 0.15, 0.18, 0.21, 0.24, 0.27, 0.295, 0.31]
    RX = [0.098, 0.112, 0.12, 0.124, 0.126, 0.125, 0.121, 0.112, 0.095, 0.07, 0.04]
    RY = [0.088, 0.100, 0.108, 0.112, 0.114, 0.114, 0.110, 0.102, 0.087, 0.064, 0.036]
    DX = [0.012, 0.012, 0.010, 0.008, 0.006, 0.004, 0.0, -0.004, -0.006, -0.006, -0.005]
    NU, EXP = 32, 2.6

    def lerp_list(lst, z):
        for k in range(len(Z) - 1):
            if Z[k] <= z <= Z[k + 1]:
                t = (z - Z[k]) / (Z[k + 1] - Z[k])
                return lst[k] + (lst[k + 1] - lst[k]) * t
        return lst[0] if z < Z[0] else lst[-1]

    def at(th, z, grow=0.0):                                     # superellipse section: boxy face, round sides
        c, s = math.cos(th), math.sin(th)
        k = (abs(c) ** EXP + abs(s) ** EXP) ** (-1.0 / EXP)
        return ((lerp_list(RX, z) + grow) * k * c + lerp_list(DX, z), (lerp_list(RY, z) + grow) * k * s, z)

    def bucket(u, v):
        return at(2 * math.pi * u, Z[round(v * (len(Z) - 1))])

    def t_slot(i, j):                                            # face (i, j): column centre angle, row j
        thc = ((i + 0.5) * 360.0 / NU + 180.0) % 360.0 - 180.0
        return (j == 4 and abs(thc) < 45.0) or (j in (2, 3) and abs(thc) < 6.0)

    helm = plate(shell("helm", bucket, NU, len(Z) - 1, cyclic=True, cap="v1", inside=(0, 0, 0.17), skip=t_slot),
                 M["steel"], M["steel_dark"], M["steel_hi"], thick=0.018, bev=0.007)
    adopt(root, helm)
    add_rolled_edge(root, helm, M["gold"], radius=0.010)          # neck opening and the T get a gold lip
    glow = ellipsoid("helm_glow", (0.085, 0.08, 0.09), loc=(0.01, 0.0, 0.15), mat=M["visor"])
    adopt(root, glow)
    # brow band: the same surface, 12 mm proud, 110 deg wide above the slot
    brow = plate(shell("helm_brow", lambda u, v: at(math.radians(-55 + 110 * u), 0.18 + 0.035 * v, grow=0.012),
                       10, 1, inside=(0, 0, 0.19)),
                 M["gold"], M["gold"], M["gold_hi"], thick=0.014, bev=0.005, segs=2)
    adopt(root, brow)
    # crest: a quad strip in the XZ plane between the helm's top arc and a swept-back top curve, solidified centred

    def crest(u, v):
        ps = math.radians(30 + 130 * u)                          # front (30 deg) over the top to the back
        rise = 0.018 + 0.085 * math.sin(math.pi * u) ** 1.5
        ps2 = ps + math.radians(12) * math.sin(math.pi * u)      # the top curve sweeps back
        bx, bz = 0.97 * 0.118 * math.cos(ps), 0.2 + 0.97 * 0.105 * math.sin(ps)
        tx, tz = (0.118 + rise) * math.cos(ps2), 0.2 + (0.105 + rise) * math.sin(ps2)
        return (bx + (tx - bx) * v, 0.0, bz + (tz - bz) * v)

    fin = plate(shell("helm_crest", crest, 16, 2, inside=(0, -1, 0.25)),
                M["gold"], M["gold"], M["gold_hi"], thick=0.016, bev=0.005, segs=2, level=1, offset=0.0)
    adopt(root, fin)
    add_rolled_edge(root, fin, M["gold_hi"], radius=0.009)
    return root


def build_gauntlet(side, M):
    """Heavy gauntlet, 4 big shapes: a flared bell cuff pointed on the outer side, a domed back-of-hand plate,
    three curling finger lames (one lame + Array with an object offset), and a dark leather hand under them; gold
    rolled edges and knuckle rivets. Local frame: origin at the wrist, fingers toward -Z, +X forward (hand width),
    the back of the hand faces outward (-Y near side, +Y far side)."""
    o = -1.0 if side == "near" else 1.0
    root = part_root(f"hand_{side}")

    def cuff(u, v):
        th = 2 * math.pi * u
        point = max(0.0, o * math.sin(th)) ** 3                  # 1 on the outer side: a pointed hero cuff
        z = -0.01 + (0.095 + 0.045 * point) * v
        r = 0.056 + 0.036 * v ** 1.6                             # bell flare toward the forearm
        return (1.12 * r * math.cos(th), r * math.sin(th), z)

    c = plate(shell(f"gauntlet_cuff_{side}", cuff, 24, 3, cyclic=True, inside=(0, 0, 0.04)),
              M["steel"], M["gold"], M["steel_hi"], thick=0.016, bev=0.006)
    adopt(root, c)
    add_rolled_edge(root, c, M["gold"], radius=0.011)

    def back(u, v):
        x = -0.052 + 0.104 * u
        bulge = 0.012 * (1 - (2 * u - 1) ** 2)
        return (x, o * (0.026 + bulge + 0.006 * v), -0.005 - 0.085 * v)

    bp = plate(shell(f"gauntlet_back_{side}", back, 6, 4, inside=(0, 0, -0.04)),
               M["steel"], M["steel_dark"], M["steel_hi"], thick=0.014, bev=0.006)
    adopt(root, bp)
    add_rolled_edge(root, bp, M["gold"], radius=0.008)

    def lame(u, v):
        x = -0.054 + 0.108 * u
        bulge = 0.010 * (1 - (2 * u - 1) ** 2)
        return (x, o * (0.030 + bulge), -0.026 * v)

    fl = shell(f"gauntlet_fingers_{side}", lame, 6, 1, inside=(0, 0, -0.013))
    fl.location = (0.0, 0.0, -0.088)                             # origin on the knuckle line = the curl pivot
    adopt(root, fl)
    step = link(bpy.data.objects.new(f"gauntlet_finger_step_{side}", None))
    step.parent = fl                                             # the step stays relative to the lame when posed
    step.location = (0.0, -o * 0.004, -0.022)                    # next lame 22 mm down, 4 mm under (palm side)
    step.rotation_euler = (-o * math.radians(22), 0.0, 0.0)      # and curled 22 deg toward the palm
    arr = fl.modifiers.new("fingers", "ARRAY")
    arr.count = 3
    arr.use_relative_offset = False
    arr.use_object_offset = True
    arr.offset_object = step
    plate(fl, M["steel"], M["steel_dark"], M["steel_hi"], thick=0.012, bev=0.005, segs=2)
    add_rolled_edge(root, fl, M["gold"], radius=0.006)
    glove = blob(f"gauntlet_glove_{side}", (0.10, 0.05, 0.145), loc=(0.0, 0.0, -0.0775), mat=M["leather"])
    thumb = ellipsoid(f"gauntlet_thumb_{side}", (0.022, 0.02, 0.042), loc=(0.058, 0.0, -0.06), mat=M["steel"])
    adopt(root, glove, thumb)
    knuckles = [(x, o * (0.035 + 0.012 * (1 - (x / 0.052) ** 2)), -0.084) for x in (-0.036, -0.012, 0.012, 0.036)]
    add_rivets(root, f"gauntlet_rivets_{side}", knuckles, M["gold"], radius=0.007)
    return root


def build_cuirass(M):
    """Cuirass, 4 big shapes: a mirrored chest plate with a creased keel ridge, a plackart shrinkwrapped over the
    belly, and three fauld lames (one strip + Array + Curve around the waist); gold rolled edges. Local frame:
    origin at the pelvis joint, +Z up the spine, +X forward."""
    root = part_root("chest")

    def chest(u, v):                           # half shell: u 0 = front midline .. 1 = 100 deg round the side
        th = math.radians(100) * u
        rx = 0.11 + 0.035 * math.sin(math.pi * min(1.0, v / 0.85))           # chest swell
        ry = 0.16 + 0.07 * math.sin(math.pi * min(1.0, v / 0.9)) * (1 - 0.4 * v)
        keel = 0.018 * max(0.0, 1 - th / math.radians(25)) * math.sin(math.pi * v)
        return ((rx + keel) * math.cos(th), ry * math.sin(th), 0.20 + 0.30 * v)

    ch = shell("cuirass", chest, 8, 6, inside=(0, 0, 0.35))
    mm = ch.modifiers.new("mirror", "MIRROR")
    mm.use_axis = (False, True, False)         # mirror across XZ: the half becomes the whole front
    mm.use_clip = True
    mm.use_mirror_merge = True
    mm.merge_threshold = 0.001
    cr = ch.data.edge_creases_ensure()         # "crease_edge" attribute; Subsurf use_creases is on by default
    for e in ch.data.edges:
        a, b = (ch.data.vertices[i].co for i in e.vertices)
        if abs(a.y) < 1e-6 and abs(b.y) < 1e-6:
            cr.data[e.index].value = 1.0       # midline -> a crisp keel after Subdivision
    plate(ch, M["steel"], M["gold"], M["steel_hi"], thick=0.02, bev=0.008)
    adopt(root, ch)
    add_rolled_edge(root, ch, M["gold"], radius=0.014)

    def belly(u, v):                           # rough placement; Shrinkwrap snaps it onto the chest plate
        th = math.radians(-50 + 100 * u)
        return (0.16 * math.cos(th), 0.2 * math.sin(th), 0.21 + 0.1 * v + 0.03 * math.cos(math.pi * (u - 0.5)) * v)

    pk = shell("plackart", belly, 8, 3, inside=(0, 0, 0.26))
    sw = pk.modifiers.new("fit", "SHRINKWRAP")  # before "subdiv", so the rolled-edge twin repeats it
    sw.target = ch
    sw.wrap_method = "NEAREST_SURFACEPOINT"
    sw.wrap_mode = "OUTSIDE_SURFACE"
    sw.offset = 0.008
    plate(pk, M["steel"], M["gold"], M["steel_hi"], thick=0.016, bev=0.007)
    adopt(root, pk)
    add_rolled_edge(root, pk, M["gold"], radius=0.012)

    cu = bpy.data.curves.new("waist", type="CURVE")
    cu.dimensions = "3D"
    sp = cu.splines.new("POLY")
    n = 40
    sp.points.add(n - 1)
    for i, pt in enumerate(sp.points):
        a = math.radians(-150 + 300 * i / (n - 1))                           # 300 deg arc, open at the back
        pt.co = (0.125 * math.cos(a), 0.175 * math.sin(a), 0.2, 1.0)
    waist = link(bpy.data.objects.new("waist", cu))
    adopt(root, waist)
    arc = sum((Vector(sp.points[i + 1].co[:3]) - Vector(sp.points[i].co[:3])).length for i in range(n - 1))
    fd = shell("fauld", lambda u, v: (arc * u, 0.0, -0.055 * v), 24, 1, inside=(arc / 2, -1.0, -0.03))
    arr = fd.modifiers.new("lames", "ARRAY")
    arr.count = 3
    arr.relative_offset_displace = (0.0, 0.0, -0.75)                          # 25 % overlap between lames
    cm = fd.modifiers.new("wrap", "CURVE")                                    # object X runs along the waist
    cm.object = waist
    cm.deform_axis = "POS_X"
    plate(fd, M["steel"], M["gold"], M["steel_hi"], thick=0.016, bev=0.006)  # flip `inside` if lames face in
    adopt(root, fd)
    add_rolled_edge(root, fd, M["gold"], radius=0.01)
    return root


def rune(root, name, strokes, place, plate_ob, mat, width=0.005):
    """Glowing inlay: strokes [[(u, v), ...], ...] in the plate's parameter window, place(u, v) -> point on the
    cage (the plate's own f). A round curve tube, shrinkwrapped so its lower half flattens onto the plate.
    Toggle with ob["glow"] = 0/1 (mat_glow(off=...)), then ob.update_tag()."""
    cu = bpy.data.curves.new(name, type="CURVE")
    cu.dimensions = "3D"
    cu.bevel_depth = width
    cu.bevel_resolution = 2
    cu.use_fill_caps = True
    for st in strokes:
        sp = cu.splines.new("POLY")
        sp.points.add(len(st) - 1)
        for pt, (u, v) in zip(sp.points, st):
            pt.co = (*place(u, v), 1.0)
    cu.materials.append(mat)
    ob = link(bpy.data.objects.new(name, cu))
    sw = ob.modifiers.new("fit", "SHRINKWRAP")
    sw.target = plate_ob
    sw.wrap_method = "NEAREST_SURFACEPOINT"
    sw.wrap_mode = "OUTSIDE_SURFACE"
    sw.offset = 0.0005
    ob["glow"] = 0.0
    adopt(root, ob)
    return ob


# ================================================================ render crispness and bloom (5.2)

def crisp_render(sc, res=768, samples=64):
    """Render at 2x the 384 px display size (phones are 2-3x density) and narrow the pixel filter."""
    sc.render.engine = "BLENDER_EEVEE"                            # 5.0+: the id is BLENDER_EEVEE again
    sc.render.resolution_x = sc.render.resolution_y = res
    sc.render.filter_size = 1.0                                   # default 1.5 px softens every edge
    sc.render.film_transparent = True
    sc.view_settings.view_transform = "Standard"
    sc.eevee.taa_render_samples = samples


def bloom(sc, threshold=1.5, strength=0.5, size=0.2):
    """5.x compositor: node options are inputs now. The Glare menu input is named "Type" (default STREAKS,
    Threshold 1.0, Strength 1.0, Size 0.5), so setattr(glare, "glare_type", ...) silently does nothing in 5.2 and
    every pixel above 1.0 gets four streaks. Size is a fraction of the image: 0.2 keeps rune halos tight."""
    ng = bpy.data.node_groups.new("comp", "CompositorNodeTree")
    rl = ng.nodes.new("CompositorNodeRLayers")
    glare = ng.nodes.new("CompositorNodeGlare")
    out = ng.nodes.new("NodeGroupOutput")
    ng.interface.new_socket("Image", in_out="OUTPUT", socket_type="NodeSocketColor")
    for v in ("BLOOM", "Bloom"):
        try:
            glare.inputs["Type"].default_value = v
            break
        except (TypeError, ValueError):
            continue
    glare.inputs["Threshold"].default_value = threshold          # above the toon palette, below the glow
    glare.inputs["Strength"].default_value = strength
    glare.inputs["Size"].default_value = size
    ng.links.new(rl.outputs["Image"], glare.inputs["Image"])
    ng.links.new(glare.outputs["Image"], out.inputs[0])
    sc.compositing_node_group = ng


# ================================================================ freezing and baking (cut-out export, AO)

def freeze(ob, suffix="_frozen"):
    """Evaluated mesh (all modifiers, GN output, custom normals, attributes) as a plain object; the original is
    removed. Use for the cut-out export and before any bake into colour attributes."""
    dg = bpy.context.evaluated_depsgraph_get()
    me = bpy.data.meshes.new_from_object(ob.evaluated_get(dg), preserve_all_data_layers=True, depsgraph=dg)
    nb = link(bpy.data.objects.new(ob.name + suffix, me))
    nb.parent = ob.parent
    nb.matrix_basis = ob.matrix_basis.copy()
    bpy.data.objects.remove(ob)
    return nb


def bake_to_color(objs, kind="AO", name="ao", samples=64, distance=0.12):
    """Cycles bake (works in background mode) into a point colour attribute that EEVEE reads through a Color
    Attribute node (mat_metal(use_ao=True)). Pass the frozen meshes of ONE rigid part and hide the other parts
    (hide_render) first, so no neighbour leaves occlusion that is wrong once the parts move."""
    sc = bpy.context.scene
    prev = sc.render.engine
    sc.render.engine = "CYCLES"
    sc.cycles.samples = samples
    if sc.world:
        sc.world.light_settings.distance = distance            # AO ray length
    for ob in objs:
        me = ob.data
        if name not in me.color_attributes:
            me.color_attributes.new(name, "FLOAT_COLOR", "POINT")
        me.color_attributes.active_color_name = name
        if not me.uv_layers:
            me.uv_layers.new(name="UVMap")                      # the manual lists a UV map as a bake requirement
    for o in bpy.context.view_layer.objects:
        o.select_set(False)
    for ob in objs:
        ob.select_set(True)
    bpy.context.view_layer.objects.active = objs[0]
    bpy.ops.object.bake(type=kind, target="VERTEX_COLORS")
    sc.render.engine = prev


# ================================================================ demo

def demo(path, res):
    bpy.ops.wm.read_factory_settings(use_empty=True)
    sc = bpy.context.scene
    M = palette()
    helm = build_helm(M)
    pl = build_pauldron("near", M)
    ga = build_gauntlet("near", M)
    helm.location = (0.0, 0.0, 0.55)
    pl.location = (0.0, -0.45, 0.35)
    ga.location = (0.0, 0.45, 0.35)
    crisp_render(sc, res)
    bloom(sc)
    w = bpy.data.worlds.new("w")
    sc.world = w
    w.node_tree.nodes["Background"].inputs["Color"].default_value = (0.2, 0.22, 0.28, 1)
    w.node_tree.nodes["Background"].inputs["Strength"].default_value = 0.3
    for rot, energy, color in (((55, 0, -50), 4.0, (1.0, 0.86, 0.66)), ((-65, 0, 135), 5.0, (0.55, 0.72, 1.0))):
        ld = bpy.data.lights.new("sun", "SUN")
        ld.energy = energy
        ld.color = color
        lo = link(bpy.data.objects.new("sun", ld))
        lo.rotation_euler = tuple(math.radians(a) for a in rot)
    cd = bpy.data.cameras.new("cam")
    cd.type = "ORTHO"
    cd.ortho_scale = 1.4
    cam = link(bpy.data.objects.new("cam", cd))
    yaw, pitch, dist = math.radians(30), math.radians(6), 8.0
    target = Vector((0.0, 0.0, 0.55))
    cam.location = target + Vector((dist * math.sin(yaw) * math.cos(pitch), -dist * math.cos(yaw) * math.cos(pitch),
                                    dist * math.sin(pitch)))
    cam.rotation_euler = (target - cam.location).to_track_quat("-Z", "Y").to_euler()
    sc.camera = cam
    sc.render.filepath = path
    bpy.ops.render.render(write_still=True)


if __name__ == "__main__":
    argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    demo(argv[0] if argv else "//plate_kit.png", int(argv[1]) if len(argv) > 1 else 768)
