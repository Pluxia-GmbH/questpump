"""Render one move's baked track (Blender 5.2 LTS, headless; docs/plan.md slice 10):

    blender -b --factory-startup -P render.py -- <track.json> <style.json> <outdir> <cell_px> figure <frames> <cloths>
            [samples]
    blender -b --factory-startup -P render.py -- <track.json> <style.json> <outdir> <cell_px> ground <frames> [samples]

<frames> is "all" or a comma list of track frames; <cloths> a comma list of cloth colours: "crimson" (the icons, the
selection and the phones' loops) or a grey level g000, g025, g050, g075, g100 (the tint split, pack.py).

figure  the armour and its held implements, the stage hidden from the camera: <outdir>/<cloth>/main/f####.png
        (2x cell, 16-bit straight alpha) and <outdir>/<cloth>/glow/glow_####.png (the bloom outside the silhouette,
        added on top). One process renders every cloth of a frame by swapping the cloth material only, so every
        other pixel is the same render.
ground  the floor pool with the figure's contact shadow (the armour posed but hidden from the camera, still casting
        its shadow): <outdir>/floor/g####.png; and when the move has a prop, the props with the figure (the armour,
        its held implements and its cloth) as a holdout, so the layer holds only what of the props the figure leaves
        in sight, its edges antialiased against the figure's: <outdir>/prop/p####.png, which the build lays over the
        figure (figures.layers). Same camera, size and pixel filter as the figure pass (the prop layer its samples
        too), so the layers register pixel for pixel.
"""
import json
import math
import os
import sys

import bpy
from mathutils import Vector

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import armor_model as am  # noqa: E402
import scene as S  # noqa: E402

CRIMSON = (0.55, 0.08, 0.08)     # the cloth's lit band in linear light (armor_model.build's default)
LEVELS = {"g000": 0.0, "g025": 0.25, "g050": 0.5, "g075": 0.75, "g100": 1.0}   # sRGB-encoded grey levels
FIGURE_SAMPLES = 16              # the figure pass's TAA samples, which the prop layer takes too: the same sample
#                                  positions give the figure's alpha and the prop's holdout one edge (figures.layers)


def cloth_rgb(name):
    if name == "crimson":
        return CRIMSON
    v = S.srgb_to_lin(LEVELS[name])
    return (v, v, v)


def frame_list(arg, n):
    return list(range(n)) if arg == "all" else [int(x) for x in arg.split(",")]


def figure(track, outdir, cell, frames, cloths, samples=FIGURE_SAMPLES):
    sc = bpy.context.scene
    parts = am.build()
    if track.get("eyes") == "closed":
        am.dim_visor(0.08)
    am.stage(sc, [])
    for ob in bpy.data.collections[am.STAGE_COLL].all_objects:
        ob.visible_camera = False
    held = S.Items(track["items"], track["view"], bpy.data.collections[am.ARMOR_COLL])
    am.render_setup(sc, cell, samples=samples, glow_dir=os.path.join(outdir, cloths[0], "glow"), glow_name="glow")
    am.apply_camera(sc, track["camera"])
    fo = next(n for n in sc.compositing_node_group.nodes if n.bl_idname == "CompositorNodeOutputFile")
    base = bpy.data.materials["qp_cloth"]
    mats = {c: am.mat_cloth(f"qp_cloth_{c}", cloth_rgb(c)) for c in cloths}
    slots = [s for ob in bpy.data.objects if ob.type == "MESH" for s in ob.material_slots if s.material == base]
    for c in cloths:
        for d in ("main", "glow"):
            os.makedirs(os.path.join(outdir, c, d), exist_ok=True)
    turns = am.arm_turns(track["frames"], track["view"])
    for i in frames:
        fr = track["frames"][i]
        am.place(parts, fr["joints"], turns[i], view=track["view"], sash=fr["sash"], head_l=fr.get("head_l"),
                 sash_rest=fr["sash_rest"])
        am.set_glow(parts, track["working"], fr["glow"])
        held.place(fr["items"])
        sc.frame_current = i
        for c, m in mats.items():
            for s in slots:
                s.material = m
            fo.directory = os.path.join(outdir, c, "glow")
            sc.render.filepath = os.path.join(outdir, c, "main", f"f{i:04d}.png")
            bpy.ops.render.render(write_still=True)


def ground(track, style, outdir, cell, frames, samples=None):
    sc = bpy.context.scene
    view = track["view"]
    parts = am.build()
    held = S.Items(track["items"], view, bpy.data.collections[am.ARMOR_COLL])
    held.place(track["frames"][frames[0]]["items"])                          # a door towel's lines come with a pose
    figure_obs = list(bpy.data.collections[am.ARMOR_COLL].all_objects)       # the held implements among them
    for ob in figure_obs:
        ob.is_holdout = True                  # in the prop layer: the figure cuts the props it stands in front of
        ob.visible_diffuse = ob.visible_glossy = ob.visible_transmission = ob.visible_volume_scatter = False
    r = sc.render
    r.engine = "BLENDER_EEVEE"
    r.resolution_x = r.resolution_y = cell * 2
    r.resolution_percentage = 100
    r.filter_size = 1.5
    r.dither_intensity = 0.0
    r.film_transparent = True
    r.use_compositing = False
    im = r.image_settings
    im.file_format, im.color_mode, im.color_depth, im.compression = "PNG", "RGBA", "16", 15
    ee = sc.eevee
    floor_samples, prop_samples = samples or style["samples"], samples or FIGURE_SAMPLES
    ee.use_shadows = True
    ee.shadow_ray_count = 4
    ee.shadow_step_count = 16
    ee.use_raytracing = False
    sc.display_settings.display_device = "sRGB"
    vs = sc.view_settings
    vs.view_transform, vs.look, vs.exposure, vs.gamma = "Standard", "None", 0.0, 1.0
    sc["qp_cell_px"] = cell
    w = bpy.data.worlds.new("qp_g_world")
    sc.world = w
    next(n for n in w.node_tree.nodes if n.type == "BACKGROUND").inputs["Strength"].default_value = 0.0
    am.apply_camera(sc, track["camera"])
    right, up, back = S.cam_basis(sc)
    fs = style["floor"]
    contacts = sorted({c for fr in track["frames"] for c in fr["contacts"]})
    centre, uax, vax, ru, rv = S.pool_geometry(track, contacts, fs, view)
    fcoll = S.am._collection(S.GROUND + "_floor")
    c = fs["sun_src_cam"]
    fsrc = (right * c[0] + up * c[1] + back * c[2]).normalized()
    S.sun(sc, "qp_g_floor_sun", fsrc, math.pi, fs["sun_angle_deg"], fcoll)
    S.floor_plane(fcoll, S.mat_floor(fs, centre, uax, vax, ru, rv, 1.0 / max(1e-3, fsrc.z)))
    pcoll = am._collection(S.GROUND + "_props")
    built = S.build_props(track, pcoll, S.prop_materials(style), view, style)
    if pcoll.objects:
        ps = style["props"]
        k = ps["key_src"]
        S.sun(sc, "qp_g_prop_key", right * k[0] + up * k[1] + back * k[2], math.pi * ps["key_energy"],
              ps["key_angle_deg"], pcoll)
        S.sun(sc, "qp_g_prop_fill", Vector((0.0, 0.0, 1.0)) + back * 0.3, math.pi * ps["fill_energy"],
              ps["fill_angle_deg"], pcoll)
    floor_obs, prop_obs = list(fcoll.objects), list(pcoll.objects)
    # on a scrolling flight the treads flow below the floor's level: no floor pool, only the shadow on the treads
    pool = not any(pr.get("scroll") for pr in track["props"])
    os.makedirs(os.path.join(outdir, "floor"), exist_ok=True)
    if prop_obs:
        os.makedirs(os.path.join(outdir, "prop"), exist_ok=True)
    turns = am.arm_turns(track["frames"], view)
    for i in frames:
        fr = track["frames"][i]
        am.place(parts, fr["joints"], turns[i], view=view, sash=fr["sash"], head_l=fr.get("head_l"),
                 sash_rest=fr["sash_rest"])
        held.place(fr["items"])
        S.move_props(built, view, i / len(track["frames"]))
        sc.frame_current = i
        # two layers, so the pack can fade the floor at the cell's edges without touching the prop: the floor whole
        # under the figure (the props and the figure hidden from the camera but casting their shadows on it), then
        # the props with the figure holding them out
        for ob in floor_obs:
            ob.visible_camera = pool
        for ob in prop_obs + figure_obs:
            ob.visible_camera = False
        ee.taa_render_samples = floor_samples
        r.filepath = os.path.join(outdir, "floor", f"g{i:04d}.png")
        bpy.ops.render.render(write_still=True)
        if prop_obs:
            for ob in floor_obs:
                ob.visible_camera = False
            for ob in prop_obs + figure_obs:
                ob.visible_camera = True
            ee.taa_render_samples = prop_samples
            r.filepath = os.path.join(outdir, "prop", f"p{i:04d}.png")
            bpy.ops.render.render(write_still=True)


def main():
    argv = sys.argv[sys.argv.index("--") + 1:]
    with open(argv[0]) as fh:
        track = json.load(fh)
    with open(argv[1]) as fh:
        style = json.load(fh)
    outdir, cell, kind = argv[2], int(argv[3]), argv[4]
    frames = frame_list(argv[5], len(track["frames"]))
    bpy.ops.wm.read_factory_settings(use_empty=True)
    if kind == "figure":
        figure(track, outdir, cell, frames, argv[6].split(","), int(argv[7]) if len(argv) > 7 else FIGURE_SAMPLES)
    elif kind == "ground":
        ground(track, style, outdir, cell, frames, int(argv[6]) if len(argv) > 6 else None)
    else:
        raise SystemExit(f"unknown pass {kind}")
    print("rendered", track["move"], kind, len(frames), "frames")


main()
