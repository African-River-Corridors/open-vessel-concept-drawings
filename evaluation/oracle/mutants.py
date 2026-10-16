"""Generate the boundary mutants of the evaluation corpus from the oracle (issue #184).

    python3 evaluation/oracle/mutants.py           # rewrites evaluation/mutants.yaml
    python3 evaluation/oracle/mutants.py --check   # fails if the file differs from what is generated

A boundary mutant is a base case moved just across one criterion's limit, and just inside it, so each
criterion is tested from both sides. Mutants are generated here, never written by hand. For each base
case and each of the six criteria this makes four mutants: the criterion's governing value is set to
its limit plus or minus a step, with a big step and a small one.

  * The big step is above the engine's grid error. The engine integrates the GZ curve by the
    trapezoid rule on a 1 degree grid and reports a whole-degree angle of maximum GZ. A big step gives
    a verdict the engine must get right: a wrong one is a false pass or a false fail.
  * The small step is inside that grid error, on purpose, and is marked `grid: true`. The engine may
    differ from the oracle there. Grid mutants are excluded from the false-pass gate.

How the input is moved:

  * KG, for GM and the three areas and GZ at 30 degrees or more. At fixed displacement a change of KG
    changes GZ by -dKG sin(phi), so GM moves by -dKG and each area by -dKG x (cos a0 - cos a1): the
    KG for a wanted area or GM is exact. For GZ at 30 degrees or more the oracle finds the KG by bisection.
  * Depth, for the angle of maximum GZ: a lower depth brings the deck edge under sooner and the maximum
    earlier. The oracle finds the depth by bisection (the angle rises with the depth).

The expected verdict of EVERY criterion is recorded for each mutant, from the oracle, not only the
targeted one. A criterion whose margin is inside the grid error is marked `grid: true` for that mutant.
"""
from __future__ import annotations

import math
import sys
from pathlib import Path

import yaml

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import box_oracle as oracle  # noqa: E402
import write_cases  # noqa: E402

OUT = HERE.parent / "mutants.yaml"

#: Step from the limit, in the unit of the criterion: (big, small).
STEPS = {
    "area_0_30": (0.002, 0.00002),
    "area_0_40": (0.002, 0.00002),
    "area_30_40": (0.002, 0.00002),
    "gz_30_plus": (0.01, 0.0001),
    "angle_max_gz": (2.0, 0.2),
    "gm": (0.005, 0.001),
}
#: Inside this distance of its limit a criterion is within the engine's grid error. The engine is
#: exact for GM, so GM has none beyond rounding.
GRID_TOL = {
    "area_0_30": 0.0005,
    "area_0_40": 0.0005,
    "area_30_40": 0.0005,
    "gz_30_plus": 0.002,
    "angle_max_gz": 1.0,
    "gm": 1e-6,
}
#: A step is a grid mutant (a small one) when it is inside the grid tolerance of its criterion.
AREA_BAND = {"area_0_30": (0.0, 30.0), "area_0_40": (0.0, 40.0), "area_30_40": (30.0, 40.0)}


class Unreachable(Exception):
    """No physical input crosses this limit: the KG would have to be above the deck, or the search
    interval does not hold the target."""


def _bisect(f, lo, hi, target, increasing):
    """x in [lo, hi] with f(x) = target, f monotone (increasing or decreasing)."""
    flo, fhi = f(lo), f(hi)
    if not (min(flo, fhi) <= target <= max(flo, fhi)):
        raise Unreachable("no KG between the keel and the deck gives the wanted value")
    for _ in range(60):
        mid = 0.5 * (lo + hi)
        v = f(mid)
        if (v < target) == increasing:
            lo = mid
        else:
            hi = mid
    return 0.5 * (lo + hi)


def _kg_on_the_hull(name, kg, depth):
    """A KG must lie on the hull: between the keel and the deck."""
    if not 0.0 < kg <= depth:
        raise Unreachable("the KG that crosses the limit is not between the keel and the deck")
    return name, kg


def move_input(base: dict, criterion: str, target: float) -> tuple[str, float]:
    """The input (`kg_m` or `depth_m`) and its new value that make `criterion` equal `target`."""
    i = base["inputs"]
    b, d, t, kg = i["beam_m"], i["depth_m"], i["draught_m"], i["kg_m"]
    if criterion == "gm":
        return _kg_on_the_hull("kg_m", oracle.gm(b, t, 0.0) - target, d)
    if criterion in AREA_BAND:
        a0, a1 = AREA_BAND[criterion]
        value = oracle.assess(b, d, t, kg)[criterion]["value"]
        per_m = math.cos(math.radians(a0)) - math.cos(math.radians(a1))
        return _kg_on_the_hull("kg_m", kg + (value - target) / per_m, d)
    if criterion == "gz_30_plus":
        f = lambda k: oracle._max_gz(lambda a: oracle.gz(b, d, t, k, a), 30.0, oracle.MAX_ANGLE)[1]
        return "kg_m", _bisect(f, 0.0, d, target, increasing=False)
    if criterion == "angle_max_gz":
        def f(depth):
            return oracle._max_gz(lambda a: oracle.gz(b, depth, t, kg, a), 0.0, oracle.MAX_ANGLE)[0]
        return "depth_m", _bisect(f, t + 0.05, t + 12.0, target, increasing=True)
    raise KeyError(criterion)


def mutant_document(base: dict, criterion: str, side: str, magnitude: str) -> dict:
    big, small = STEPS[criterion]
    step = big if magnitude == "big" else small
    limit = oracle.LIMITS[criterion]
    target = limit + step if side == "pass" else limit - step
    variable, value = move_input(base, criterion, target)
    inputs = dict(base["inputs"])
    inputs[variable] = write_cases.sig(value, 12)
    r = oracle.assess(inputs["beam_m"], inputs["depth_m"], inputs["draught_m"], inputs["kg_m"],
                      inputs["free_surface_m"])
    expected = {}
    for name in oracle.CRITERIA:
        rule, what, unit = write_cases.RULES[name]
        e = r[name]
        expected[name] = {
            "rule": rule, "what": what, "unit": unit,
            "verdict": "pass" if e["ok"] else "fail",
            "value": write_cases.sig(e["value"]), "limit": e["limit"], "margin": write_cases.sig(e["margin"]),
            "grid": abs(e["margin"]) < GRID_TOL[name],
        }
    cid = f"{base['id']}__{criterion}__{side}_{magnitude}"
    grid = magnitude == "small" and step < GRID_TOL[criterion] * 4
    return {
        "id": cid,
        "title": f"{base['title']}, {criterion} just {'inside' if side == 'pass' else 'across'} its limit ({magnitude} step)",
        "kind": "box",
        "inputs": inputs,
        "expected": expected,
        "mutant": {
            "parent": base["id"], "criterion": criterion, "side": side, "magnitude": magnitude,
            "moved": variable, "target_value": write_cases.sig(target, 10), "grid": grid,
        },
        "provenance": {
            "verdict_basis": "derived",
            "source": "evaluation/oracle/mutants.py from the parent case, checked by "
                      "evaluation/oracle/box_oracle.py",
            "edition": base["provenance"]["edition"],
            "locator": base["provenance"]["locator"],
            "licence": base["provenance"]["licence"],
            "confidence": "high for the verdicts; a mutant is not an external published example",
        },
    }


def generate() -> tuple[list[dict], list[dict]]:
    """(mutants, skipped). A mutant is skipped when no KG on the hull (or depth) crosses the limit."""
    docs, skipped = [], []
    for c in write_cases.BASE_CASES:
        base = write_cases.case_document(c)
        for criterion in oracle.CRITERIA:
            for side in ("pass", "fail"):
                for magnitude in ("big", "small"):
                    try:
                        docs.append(mutant_document(base, criterion, side, magnitude))
                    except Unreachable as e:
                        skipped.append({"parent": base["id"], "criterion": criterion, "side": side,
                                        "magnitude": magnitude, "reason": str(e)})
    return docs, skipped


def render(docs: list[dict], skipped: list[dict]) -> str:
    return ("# Generated by evaluation/oracle/mutants.py. Do not edit by hand.\n"
            + yaml.safe_dump({"mutants": docs, "skipped": skipped}, sort_keys=False,
                             allow_unicode=True, width=100))


def main(argv: list[str]) -> int:
    text = render(*generate())
    if "--check" in argv:
        diff = "missing" if not OUT.exists() else write_cases.first_difference(
            yaml.safe_load(OUT.read_text(encoding="utf-8")), yaml.safe_load(text))
        if diff:
            print(f"mutants: {OUT} differs from what the oracle generates at {diff}", file=sys.stderr)
            return 1
        return 0
    OUT.write_text(text, encoding="utf-8")
    print(f"wrote {OUT} ({len(text.splitlines())} lines)")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
