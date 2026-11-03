"""The figure's skeleton and its solvers (pure Python, no Blender, no numpy; docs/plan.md slice 10).

Body space is centimetres with f forward, u up and l lateral (the far side positive, the near side negative: the near
side faces the camera in the side view). `solve` turns one pose into the twenty joints the armour renderer reads
(`blender/armor_model.py`, `place`): the spine by angles from the pelvis, each arm and leg by angles (FK) or by
two-bone IK to a target bent toward a pole (the feet are motion.py's: planted, raised or carried).

Lateral values in a pose are outward distances (positive away from the midline on the limb's own side), so one spec
serves both sides and mirrored moves need no signs. The bone lengths are the armour's own (the renderer's SEG table
and the prototype's tracks, 2026-09-24).
"""
import math

BONES = {
    "abdomen": 24.0, "chest": 28.0, "neck_head": 20.0, "shoulder_drop": 5.0, "upper_arm": 29.0, "forearm": 27.0,
    "hand": 16.0, "thigh": 45.0, "shin": 43.0, "foot": 15.0, "shoulder_half": 26.0, "hip_half": 11.0,
    "ankle_height": 7.0, "ball_forward": 14.0, "heel_back": 6.0, "sole_front": 21.0,
}
SIDES = ("near", "far")
SIGN = {"near": -1.0, "far": 1.0}
LIMB_JOINTS = ("shoulder", "elbow", "wrist", "hand_tip", "hip", "knee", "ankle", "toe")
JOINTS = ("pelvis", "waist", "neck", "head") + tuple(f"{j}_{s}" for s in SIDES for j in LIMB_JOINTS)

# How far the armour's outer surface stands from each joint centre (cm), for the floor and prop checks: a joint
# whose centre is nearer the floor than this is inside it, unless the pose declares it a contact.
CLEARANCE = {
    "pelvis": 14.5, "waist": 13.5, "neck": 13.0, "head": 12.0, "shoulder": 11.0, "elbow": 8.5, "wrist": 3.3,
    "hand_tip": 1.0, "hip": 12.0, "knee": 9.0, "ankle": 7.0, "toe": 7.0,
}


# --- vectors -------------------------------------------------------------------------------------------------------

def add(a, b):
    return (a[0] + b[0], a[1] + b[1], a[2] + b[2])


def sub(a, b):
    return (a[0] - b[0], a[1] - b[1], a[2] - b[2])


def mul(a, k):
    return (a[0] * k, a[1] * k, a[2] * k)


def dot(a, b):
    return a[0] * b[0] + a[1] * b[1] + a[2] * b[2]


def cross(a, b):
    return (a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0])


def norm(a):
    return math.sqrt(dot(a, a))


def unit(a, fallback=(0.0, 1.0, 0.0)):
    n = norm(a)
    return mul(a, 1.0 / n) if n > 1e-9 else fallback


def dist(a, b):
    return norm(sub(a, b))


def lerp(a, b, t):
    return tuple(x + (y - x) * t for x, y in zip(a, b))


def rotate(v, axis, deg):
    """Rodrigues: v turned `deg` about the axis, right-handed."""
    a = math.radians(deg)
    c, s = math.cos(a), math.sin(a)
    k = unit(axis)
    return add(add(mul(v, c), mul(cross(k, v), s)), mul(k, dot(k, v) * (1.0 - c)))


def angle(a, b):
    """The angle (deg) between two vectors."""
    na, nb = norm(a), norm(b)
    if na < 1e-9 or nb < 1e-9:
        return 0.0
    return math.degrees(math.acos(max(-1.0, min(1.0, dot(a, b) / (na * nb)))))


def square_to(v, axis):
    """The part of v square to the unit axis."""
    return sub(v, mul(axis, dot(v, axis)))


F0, U0, L0 = (1.0, 0.0, 0.0), (0.0, 1.0, 0.0), (0.0, 0.0, 1.0)


class Frame:
    """An orthonormal frame, f forward, u up, l lateral (right-handed: f x u = l)."""

    __slots__ = ("f", "u", "l")

    def __init__(self, f=F0, u=U0, l=L0):
        self.f, self.u, self.l = f, u, l

    def turned(self, pitch=0.0, roll=0.0, yaw=0.0):
        """Yaw about u (toward the far side positive), then pitch about l (a forward lean positive), then roll about
        f (a lean toward the far side positive)."""
        f, u, l = self.f, self.u, self.l
        if yaw:
            f, l = rotate(f, u, -yaw), rotate(l, u, -yaw)
        if pitch:
            f, u = rotate(f, l, -pitch), rotate(u, l, -pitch)
        if roll:
            u, l = rotate(u, f, roll), rotate(l, f, roll)
        return Frame(f, u, l)

    def at(self, v, side_sign=1.0):
        """A frame-relative vector (f, u, out) in body space; out is lateral times the side's sign."""
        return add(add(mul(self.f, v[0]), mul(self.u, v[1])), mul(self.l, v[2] * side_sign))


# --- limbs ---------------------------------------------------------------------------------------------------------

def two_bone(root, target, l1, l2, pole):
    """The middle joint of a two-bone chain from `root` reaching `target`, bent toward the direction `pole`.
    Returns (middle, end, shortfall): the end is the target, or the nearest reachable point toward it when it is out
    of reach (shortfall > 0 cm, which the checks report as an error)."""
    d = sub(target, root)
    n = norm(d)
    reach = l1 + l2 - 1e-6
    short = max(0.0, n - reach)
    n = min(max(n, abs(l1 - l2) + 1e-6), reach)
    axis = unit(d, U0)
    a = (l1 * l1 - l2 * l2 + n * n) / (2.0 * n)
    h = math.sqrt(max(0.0, l1 * l1 - a * a))
    p = unit(square_to(pole, axis), unit(cross(axis, L0), F0))
    mid = add(add(root, mul(axis, a)), mul(p, h))
    return mid, add(root, mul(axis, n)), short


def bent(first, deg, toward):
    """A direction `deg` away from `first`, turned toward `toward` (its part square to `first`)."""
    p = unit(square_to(toward, first), unit(cross(first, L0), F0))
    a = math.radians(deg)
    return unit(add(mul(first, math.cos(a)), mul(p, math.sin(a))))


def fk_first(frame, sign, flex, abduct):
    """A limb's first bone by angles: it hangs along -u, turns `flex` toward f (forward), then `abduct` out to its own
    side. Returns (direction, front): front is the bone's forward face (where a forearm bends to)."""
    d = rotate(mul(frame.u, -1.0), frame.l, flex)
    front = rotate(frame.f, frame.l, flex)
    if abduct:
        axis = rotate(mul(frame.f, sign), frame.l, flex)   # about the (flexed) forward axis, out to its own side
        d, front = rotate(d, axis, -abduct), rotate(front, axis, -abduct)
    return d, front
