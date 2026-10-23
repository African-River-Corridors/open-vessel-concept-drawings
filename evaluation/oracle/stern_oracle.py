"""An independent oracle for the stern, wheelhouse and propeller Rules (issue #355).

It never calls the engine and shares no code with it (standard library only). It works out each Rule's
limit and verdict from the text of ES-TRIN 2025/1 Articles 5.06(1), 7.02(2) and 28.04(1)(a), SOLAS V/22.1.1
(MSC.99(73)), Rhine Police Regulations 1.07(2) and the Checks of spec 0007, not from the Rule Set data.

A case (`kind: stern`) gives the particulars of a self-propelled craft and the numbers a run would read
from its results: the length overall, the bow thruster and the number of propulsors; the Blind Distance
ahead of the bow in the light and the laden condition; the speed through the water in deep water (km/h)
and the Design Speed (kn); the propeller diameter, the largest the Stern Form allows, whether it is ducted,
the lightest aft draught and the blade load. The boundary cases move one number to each side of a limit.

    python3 evaluation/oracle/stern_oracle.py CASE.yaml      # print the verdicts of one case
"""
from __future__ import annotations

import sys
from pathlib import Path

KN_TO_KMH = 1.852
#: Beveridge 1971 (a guide): the shaft axis at least this many diameters under the water; Veth rating basis:
#: kW per m2 of disc. Open, then ducted.
IMMERSION = (0.76, 0.71)
BLADE_LOAD = (400.0, 500.0)
#: The tolerance of a Rule whose limit is a ratio or worked out: 1e-9, as in the Rule Sets.
TOL = 1e-9


def _entry(rule: str, what: str, unit: str, compare: str, value: float, limit: float, tol: float = 0.0) -> dict:
    ok = value >= limit - tol if compare == "ge" else value <= limit + tol
    margin = (value - limit) if compare == "ge" else (limit - value)
    return {"rule": rule, "what": what, "unit": unit, "compare": compare,
            "verdict": "pass" if ok else "fail", "value": float(f"{value:.10g}"),
            "limit": float(f"{limit:.10g}"), "margin": float(f"{margin:.10g}")}


def verdicts(i: dict) -> dict:
    """The expected verdict of every Rule that applies to the case, by rule id."""
    out: dict[str, dict] = {}
    loa = i["length_overall_m"]
    out["es-trin/7.02(2)/blind-zone-unladen"] = _entry(
        "ES-TRIN 7.02(2)", "area of obstructed vision ahead, unladen", "m", "le",
        i["blind_distance_light_m"], 250.0)
    out["es-trin/5.06(1)/speed-13kmh"] = _entry(
        "ES-TRIN 5.06(1)", "speed through the water, laden, deep water", "km/h", "ge",
        i["speed_deep_water_kmh"], 13.0)
    # 28.04(1)(a): a craft of 110 m or less is outside it; a longer one needs a bow thruster (with one
    # or more propulsors, the thruster controlled from the wheelhouse).
    redundant = loa <= 110.0 or (i["bow_thruster"] != "none" and i["propulsor_count"] >= 1)
    out["es-trin/28.04(1)(a)/propulsion-redundancy"] = _entry(
        "ES-TRIN 28.04(1)(a)", "propulsion redundancy of a craft over 110 m", "-", "ge",
        1.0 if redundant else 0.0, 1.0)
    # SOLAS V/22.1.1: from 45 m; not more than 2 L or 500 m, whichever is less.
    if loa >= 45.0:
        limit = min(2.0 * loa, 500.0)
        for cond, key in (("light", "blind_distance_light_m"), ("laden", "blind_distance_laden_m")):
            out[f"solas/V.22.1.1/blind-zone-{cond}"] = _entry(
                "SOLAS V/22.1.1", f"blind zone ahead, {cond}", "m", "le", i[key], limit, TOL)
    out["rheinschpv/1.07(2)/blind-zone-laden"] = _entry(
        "Rhine Police Regulations 1.07(2)", "blind zone ahead, laden", "m", "le", i["blind_distance_laden_m"], 350.0)
    out["ovcd/powering/installed-covers-design-speed"] = _entry(
        "spec 0007", "service speed over the Design Speed", "-", "ge",
        i["speed_deep_water_kmh"] / KN_TO_KMH / i["design_speed_kn"], 1.0, TOL)
    d = i["propeller_diameter_m"]
    out["ovcd/propulsion/propeller-fits-stern"] = _entry(
        "spec 0007", "propeller diameter over the largest the stern allows", "-", "le",
        d / i["propeller_d_max_m"], 1.0, TOL)
    k = 1 if i["propeller_ducted"] else 0
    submergence = i["aft_draught_light_m"] - d / 2.0
    out["ovcd/propulsion/immersion-light"] = _entry(
        "spec 0007", "shaft submergence over the diameter, over its limit", "-", "ge",
        submergence / d / IMMERSION[k], 1.0, TOL)
    out["ovcd/propulsion/blade-load"] = _entry(
        "spec 0007", "blade load over its limit", "-", "le", i["blade_load_kw_m2"] / BLADE_LOAD[k], 1.0, TOL)
    return out


# --- boundary cases -----------------------------------------------------------------------------

#: Step from the limit in the unit of the rule: (big, small).
#: A step in metres is smaller than the 0.01 m step of a rule mutant, so a mutant of a constant limit
#: (one step either way) lies across a case.
STEPS = {"m": (0.005, 0.0001), "km/h": (0.01, 0.0001), "-": (0.001, 0.00001)}

#: The Rules with no number to move: a case moves them by its particulars.
NO_BOUNDARY = ("es-trin/28.04(1)(a)/propulsion-redundancy",)


def move(i: dict, rule_id: str, target: float) -> tuple[str, float]:
    """The input (name, value) that makes `rule_id`'s governing value equal `target`."""
    k = 1 if i["propeller_ducted"] else 0
    d = i["propeller_diameter_m"]
    if rule_id in ("es-trin/7.02(2)/blind-zone-unladen", "solas/V.22.1.1/blind-zone-light"):
        return "blind_distance_light_m", target
    if rule_id in ("solas/V.22.1.1/blind-zone-laden", "rheinschpv/1.07(2)/blind-zone-laden"):
        return "blind_distance_laden_m", target
    if rule_id == "es-trin/5.06(1)/speed-13kmh":
        return "speed_deep_water_kmh", target
    if rule_id == "ovcd/powering/installed-covers-design-speed":
        return "speed_deep_water_kmh", target * i["design_speed_kn"] * KN_TO_KMH
    if rule_id == "ovcd/propulsion/propeller-fits-stern":
        return "propeller_diameter_m", target * i["propeller_d_max_m"]
    if rule_id == "ovcd/propulsion/immersion-light":
        return "aft_draught_light_m", target * IMMERSION[k] * d + d / 2.0
    if rule_id == "ovcd/propulsion/blade-load":
        return "blade_load_kw_m2", target * BLADE_LOAD[k]
    raise KeyError(rule_id)


class Unreachable(Exception):
    pass


def boundary_case(base: dict, rule_id: str, side: str, magnitude: str) -> dict:
    """A base case moved to one side of one Rule's limit, with the oracle's verdicts for every Rule."""
    i = base["inputs"]
    entry = verdicts(i)[rule_id]
    big, small = STEPS[entry["unit"]]
    step = big if magnitude == "big" else small
    sign = 1.0 if entry["compare"] == "ge" else -1.0
    target = entry["limit"] + (sign * step if side == "pass" else -sign * step)
    if target <= 0.0:
        raise Unreachable("the wanted value is not positive")
    name, value = move(i, rule_id, target)
    inputs = dict(i)
    inputs[name] = float(f"{value:.12g}")
    expected = verdicts(inputs)
    slug = rule_id.replace("/", "_").replace("(", "").replace(")", "")
    return {
        "id": f"{base['id']}__{slug}__{side}_{magnitude}",
        "title": f"{base['title']}, {rule_id} just {'inside' if side == 'pass' else 'across'} its limit ({magnitude} step)",
        "kind": "stern",
        "inputs": inputs,
        "expected": {k: {kk: vv for kk, vv in v.items() if kk != "compare"} for k, v in expected.items()},
        "mutant": {"parent": base["id"], "criterion": rule_id, "side": side, "magnitude": magnitude,
                   "moved": name, "target_value": float(f"{target:.10g}"), "grid": False},
        "provenance": {
            "verdict_basis": "derived",
            "source": "evaluation/oracle/stern_oracle.py from the parent case",
            "edition": base["provenance"]["edition"], "locator": base["provenance"]["locator"],
            "licence": base["provenance"]["licence"],
            "confidence": "high for the verdicts; a mutant is not an external published example",
        },
    }


def generate(bases: list[dict]) -> tuple[list[dict], list[dict]]:
    """(boundary cases, skipped) of the stern base cases."""
    docs, skipped = [], []
    for base in bases:
        for rule_id in verdicts(base["inputs"]):
            if rule_id in NO_BOUNDARY:
                continue
            for side in ("pass", "fail"):
                # One step: the cases are exact in the Rule's own unit, so there is no grid error to test inside.
                for magnitude in ("big",):
                    try:
                        docs.append(boundary_case(base, rule_id, side, magnitude))
                    except Unreachable as e:
                        skipped.append({"parent": base["id"], "criterion": rule_id, "side": side,
                                        "magnitude": magnitude, "reason": str(e)})
    return docs, skipped


def main(argv: list[str]) -> int:
    import yaml
    case = yaml.safe_load(Path(argv[1]).read_text(encoding="utf-8"))
    for rule_id, e in verdicts(case["inputs"]).items():
        print(f"{rule_id}: {e['verdict']} (value {e['value']}, limit {e['limit']})")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
