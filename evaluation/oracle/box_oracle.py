"""An independent oracle for the IS Code 2008 Part A 2.2 criteria of a box section (issue #184).

It never calls the engine and shares no code with it (standard library only). The method differs
from the core's: the core finds the waterline of a heeled box by bisection on a clipped polygon and
integrates the GZ curve by the trapezoid rule on a 1 degree grid. This script

  * finds the heeled waterline EXACTLY: the submerged area is a piecewise quadratic in the
    waterline height, so it solves the quadratic of the piece that holds the target area;
  * takes the submerged polygon of the heeled rectangle and its centroid (shoelace formulas), so GZ
    is exact for every heel angle, deck-edge immersion and bilge emergence included;
  * integrates GZ over heel angle with Simpson's rule on a 0.01 degree step, and finds the angle of
    maximum GZ to 1e-9 degree by golden-section search on the bracket the fine scan gives.

Conventions: Y is to starboard, Z is up, origin at the keel centreline. The hull is the rectangle
|Y| <= B/2, 0 <= Z <= D, with a constant displaced area B x T (the same box idealisation as the
engine). Heel rotates the hull by phi so that starboard lifts; GZ = Y_G - Y_B is the righting
lever, positive when it rights the hull.

The limits are IMO resolution MSC.267(85), annex, Part A 2.2.1 to 2.2.4 (see evaluation/README.md).
Run `python3 evaluation/oracle/box_oracle.py B D T KG [FSC]` to print one case.
"""
from __future__ import annotations

import math
import sys

# IS Code 2008, Part A 2.2 (MSC.267(85), annex, printed pages 11 and 12).
LIMITS = {
    "area_0_30": 0.055,    # 2.2.1, metre-radians
    "area_0_40": 0.090,    # 2.2.1, metre-radians (no downflooding angle in a box)
    "area_30_40": 0.030,   # 2.2.1, metre-radians
    "gz_30_plus": 0.20,    # 2.2.2, metres, at an angle of heel >= 30 degrees
    "angle_max_gz": 25.0,  # 2.2.3, degrees
    "gm": 0.15,            # 2.2.4, metres
}
CRITERIA = list(LIMITS)
#: The engine evaluates heel angles 0 to 60 degrees. 2.2.2 is checked over 30 to 60 degrees here.
MAX_ANGLE = 60.0


def _clip_below(poly, h):
    """The part of the convex polygon `poly` (list of (y, z)) with z <= h."""
    out = []
    n = len(poly)
    for i in range(n):
        p, q = poly[i], poly[(i + 1) % n]
        pin, qin = p[1] <= h, q[1] <= h
        if pin:
            out.append(p)
        if pin != qin:
            t = (h - p[1]) / (q[1] - p[1])
            out.append((p[0] + t * (q[0] - p[0]), h))
    return out


def _area_centroid_y(poly):
    """(area, centroid y) by the shoelace formulas."""
    a = cy = 0.0
    for i in range(len(poly)):
        (y0, z0), (y1, z1) = poly[i], poly[(i + 1) % len(poly)]
        c = y0 * z1 - y1 * z0
        a += c
        cy += (y0 + y1) * c
    if abs(a) < 1e-300:
        return 0.0, 0.0
    return abs(a) / 2.0, cy / (3.0 * a)


def _rotated_box(beam, depth, phi):
    c, s = math.cos(phi), math.sin(phi)
    hull = [(-beam / 2, 0.0), (beam / 2, 0.0), (beam / 2, depth), (-beam / 2, depth)]
    return [(y * c - z * s, y * s + z * c) for y, z in hull]


def waterline(beam, depth, draught, phi):
    """The waterline height (Z, earth frame) that leaves the displaced area B x T under it, exactly."""
    poly = _rotated_box(beam, depth, phi)
    target = beam * draught
    zs = sorted({p[1] for p in poly})
    area_at = lambda h: _area_centroid_y(_clip_below(poly, h))[0]
    for lo, hi in zip(zs, zs[1:]):
        a_lo, a_hi = area_at(lo), area_at(hi)
        if a_lo <= target <= a_hi:
            if hi - lo < 1e-15 or a_hi - a_lo < 1e-300:
                return lo
            # area(h) is a quadratic on [lo, hi]: fit it through three points, solve it exactly.
            mid = 0.5 * (lo + hi)
            a_mid = area_at(mid)
            # area(lo + u) = a_lo + b u + c u^2, with u = h - lo.
            half = 0.5 * (hi - lo)
            c = (a_hi - 2 * a_mid + a_lo) / (2 * half * half)
            b = (a_mid - a_lo) / half - c * half
            need = target - a_lo
            # Area increases with the waterline, so b >= 0 and the wanted root is
            # u = 2 need / (b + sqrt(b^2 + 4 c need)), which is stable when c is tiny or zero.
            disc = max(b * b + 4 * c * need, 0.0)
            denom = b + math.sqrt(disc)
            if denom > 0.0:
                return lo + 2.0 * need / denom
            return lo
    raise ValueError("the box does not hold the displaced area (draught above depth?)")


def gz(beam, depth, draught, kg, phi_deg, fsc=0.0):
    """Exact righting lever (m) at heel `phi_deg`, with the free surface correction `fsc` (m of GM)."""
    if phi_deg == 0.0:
        return 0.0
    phi = math.radians(phi_deg)
    poly = _rotated_box(beam, depth, phi)
    h = waterline(beam, depth, draught, phi)
    _, y_b = _area_centroid_y(_clip_below(poly, h))
    y_g = -kg * math.sin(phi)
    return (y_g - y_b) - fsc * math.sin(phi)


def gm(beam, draught, kg, fsc=0.0):
    """GM = KB + BM - KG - FSC, with KB = T/2 and BM = B^2 / (12 T)."""
    return draught / 2.0 + beam * beam / (12.0 * draught) - kg - fsc


def _simpson(f, a, b, step):
    n = max(2, int(round((b - a) / step)))
    n += n % 2
    h = (b - a) / n
    total = f(a) + f(b)
    for i in range(1, n):
        total += (4 if i % 2 else 2) * f(a + i * h)
    return total * h / 3.0


def _max_gz(f, lo, hi):
    """The angle and value of the largest f on [lo, hi]: a 0.01 degree scan, then golden section."""
    best_a, best = lo, f(lo)
    steps = int(round((hi - lo) / 0.01))
    for i in range(1, steps + 1):
        a = lo + i * 0.01
        v = f(a)
        if v > best:
            best_a, best = a, v
    a, b = max(lo, best_a - 0.01), min(hi, best_a + 0.01)
    r = (math.sqrt(5) - 1) / 2
    c, d = b - r * (b - a), a + r * (b - a)
    for _ in range(80):
        if f(c) > f(d):
            b, d = d, c
            c = b - r * (b - a)
        else:
            a, c = c, d
            d = a + r * (b - a)
    x = 0.5 * (a + b)
    return (x, f(x)) if f(x) >= best else (best_a, best)


def assess(beam, depth, draught, kg, fsc=0.0):
    """The six criteria for a box. Returns {criterion: {value, limit, margin, ok}} plus `max_gz`."""
    f = lambda deg: gz(beam, depth, draught, kg, deg, fsc)
    rad = math.pi / 180.0
    step = 0.01
    area = lambda a0, a1: _simpson(lambda x: f(x), a0, a1, step) * rad
    angle, peak = _max_gz(f, 0.0, MAX_ANGLE)
    _, gz30 = _max_gz(f, 30.0, MAX_ANGLE)
    values = {
        "area_0_30": area(0.0, 30.0),
        "area_0_40": area(0.0, 40.0),
        "area_30_40": area(30.0, 40.0),
        "gz_30_plus": gz30,
        "angle_max_gz": angle,
        "gm": gm(beam, draught, kg, fsc),
    }
    out = {}
    for name, value in values.items():
        out[name] = {"value": value, "limit": LIMITS[name], "margin": value - LIMITS[name],
                     "ok": value >= LIMITS[name]}
    out["max_gz"] = {"value": peak, "angle_deg": angle}
    return out


def main(argv):
    if not 5 <= len(argv) <= 6:
        print(__doc__)
        return 2
    b, d, t, kg = (float(x) for x in argv[1:5])
    fsc = float(argv[5]) if len(argv) == 6 else 0.0
    for name, r in assess(b, d, t, kg, fsc).items():
        print(name, r)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
