"""An independent oracle for the liquid bulk Rules of `adn-tank` and `marpol-i` (issue #338).

It never calls the engine and shares no code with it (standard library only). It lays the cargo
tanks out from the case's inputs by the rules the spec gives (a cargo zone, a cofferdam at each end,
the tank block split into equal tank positions, a slop tank share at the aft end, a double side and a
double bottom), and works out each Rule's limit and verdict from the text of the regulation, not from
the Rule Set data: the capacity table is written as the text prints it (180 + (L x B x H - 600) x
0.0635), where the Rule Set holds 141.9 + 0.0635 x L x B x H.

A case (`kind: tank`) gives: the hull (loa, beam, depth), the cargo zone (aft, fwd), the layout
(double side, double bottom, tanks in line, centreline bulkhead, cofferdam, pump room, slop share),
the deadweight (a particular, not worked out), the Service Area and the ADN Tank Vessel Type. The
boundary cases move one input to each side of a limit, written here, never by hand.

Sources: ADN 2025 9.3.2.11.1, 9.3.2.11.3, 9.3.2.11.7 (type C) and 9.3.3.11.1, 9.3.3.11.3, 9.3.3.11.7
(type N); MARPOL Annex I, resolution MEPC.117(52), regulations 19.6 and 29. See evaluation/README.md.

    python3 evaluation/oracle/tank_oracle.py CASE.yaml      # print the verdicts of one case
"""
from __future__ import annotations

import sys
from pathlib import Path

def layout(i: dict) -> dict:
    """The tank block and the cargo tanks of a case's inputs, by the spec's layout rules."""
    zone = i["zone_fwd_m"] - i["zone_aft_m"]
    pump = i.get("pump_room_m", 0.0)
    block = zone - pump - 2.0 * i["cofferdam_m"]
    n = i["tanks_in_line"]
    slop_len = i["slop_share"] * block
    cargo_len = (block - slop_len) / n
    inner = i["beam_m"] - 2.0 * i["double_side_m"]
    breadth = inner / (2.0 if i["centreline_bulkhead"] else 1.0)
    height = i["depth_m"] - i["double_bottom_m"]
    count = n * (2 if i["centreline_bulkhead"] else 1)
    return {"block_m": block, "slop_m": slop_len, "tank_length_m": cargo_len, "tank_breadth_m": breadth,
            "tank_height_m": height, "tank_count": count, "tank_volume_m3": cargo_len * breadth * height,
            "slop_share": slop_len / block if block > 0 else 0.0}


def adn_capacity_limit(lbh: float) -> float:
    """ADN 9.3.2.11.1 (a) and 9.3.3.11.1 (a), as printed."""
    if lbh <= 600.0:
        return lbh * 0.3
    if lbh <= 3750.0:
        return 180.0 + (lbh - 600.0) * 0.0635
    return 380.0


def adn_length_limit(loa: float) -> float:
    """ADN 9.3.2.11.1 (d) and 9.3.3.11.1 (d): 10.00 m up to 50.00 m, else 0.20 L."""
    return 10.0 if loa <= 50.0 else 0.20 * loa


def band(dw: float) -> str:
    """MARPOL Annex I Reg. 19: 600 tonnes deadweight and above; 5,000 tonnes for 19.2 and 19.6."""
    if dw < 600.0:
        return "under-600-t"
    if dw < 5000.0:
        return "600-to-5000-t"
    return "5000-t-and-above"


def marpol_tank_length_limit(i: dict) -> float:
    """MARPOL Annex I Reg. 23.3.2: 10 m or the formula, whichever is the greater. The case has one tank across
    (23.3.2.1: (0.5 bi/B + 0.1) L, not above 0.2 L) or two with a centre line bulkhead (23.3.2.2:
    (0.25 bi/B + 0.15) L); bi is the double side width."""
    ratio = i["double_side_m"] / i["beam_m"]
    if i["centreline_bulkhead"]:
        formula = (0.25 * ratio + 0.15) * i["loa_m"]
    else:
        formula = min((0.5 * ratio + 0.1) * i["loa_m"], 0.2 * i["loa_m"])
    return max(10.0, formula)


#: The tolerance of the Rules whose limit is worked out (a table row, a formula): 1e-9 in the unit of the
#: limit, as in the Rule Sets. A constant limit has none.
TOL = 1e-9


def _entry(rule: str, what: str, unit: str, compare: str, value: float, limit: float, tol: float = 0.0) -> dict:
    ok = value >= limit - tol if compare == "ge" else value <= limit + tol
    margin = (value - limit) if compare == "ge" else (limit - value)
    return {"rule": rule, "what": what, "unit": unit, "compare": compare,
            "verdict": "pass" if ok else "fail", "value": float(f"{value:.10g}"),
            "limit": float(f"{limit:.10g}"), "margin": float(f"{margin:.10g}")}


def verdicts(i: dict) -> dict:
    """The expected verdict of every evaluated Rule that applies to the case, by rule id. A per-tank Rule
    gives the verdict of the worst tank (all tanks of a layout are equal but for rounding)."""
    lay = layout(i)
    out: dict[str, dict] = {}
    t = i["adn_vessel_type"]
    c_type = t == "C"
    lbh = i["loa_m"] * i["beam_m"] * i["depth_m"]
    par = "9.3.2" if c_type else "9.3.3"
    out[f"adn/{par}.11.1(a)/cargo-tank-capacity"] = _entry(
        "ADN " + par + ".11.1 (a)", "capacity of each cargo tank", "m3", "le",
        lay["tank_volume_m3"], adn_capacity_limit(lbh), TOL)
    out[f"adn/{par}.11.1(d)/cargo-tank-length"] = _entry(
        "ADN " + par + ".11.1 (d)", "length of each cargo tank", "m", "le",
        lay["tank_length_m"], adn_length_limit(i["loa_m"]), TOL)
    out[f"adn/{par}.11.3(a)/cofferdam-width"] = _entry(
        "ADN " + par + ".11.3 (a)", "width of the cofferdams", "m", "ge", i["cofferdam_m"], 0.60)
    side = 1.0 if c_type else 0.60
    bottom = 0.70 if c_type else 0.50
    out[f"adn/{par}.11.7/wing-width-min"] = _entry(
        "ADN " + par + ".11.7", "distance from the side wall to the cargo tanks", "m", "ge",
        i["double_side_m"], side)
    out[f"adn/{par}.11.7/double-bottom-depth"] = _entry(
        "ADN " + par + ".11.7", "distance from the bottom to the cargo tanks", "m", "ge",
        i["double_bottom_m"], bottom)
    if c_type:
        out["adn/9.3.2.11.7/double-bottom-min"] = _entry(
            "ADN 9.3.2.11.7", "depth of the double bottom, never less than", "m", "ge",
            i["double_bottom_m"], 0.60)
    if i["service_area"] == "coastal-and-river":
        out["marpol/29/slop-tank-capacity"] = _entry(
            "MARPOL Annex I Reg. 29.2.3", "slop tank capacity share", "-", "ge", lay["slop_share"], 0.03, TOL)
        if i["deadweight_t"] < 5000.0:
            out["marpol/23.3.2/cargo-tank-length"] = _entry(
                "MARPOL Annex I Reg. 23.3.2", "cargo tank length over its limit", "-", "le",
                lay["tank_length_m"] / marpol_tank_length_limit(i), 1.0, TOL)
        if band(i["deadweight_t"]) == "600-to-5000-t":
            out["marpol/19.6.1/double-bottom-depth"] = _entry(
                "MARPOL Annex I Reg. 19.6.1", "double bottom depth", "m", "ge", i["double_bottom_m"],
                max(i["beam_m"] / 15.0, 0.76), TOL)
            out["marpol/19.6.2/double-side-width"] = _entry(
                "MARPOL Annex I Reg. 19.6.2", "wing tank width", "m", "ge", i["double_side_m"],
                max(0.4 + 2.4 * i["deadweight_t"] / 20000.0, 0.76), TOL)
    return out


# --- boundary cases -----------------------------------------------------------------------------

#: Step from the limit, in the unit of the rule: (big, small). There is no grid error: both give a
#: verdict the engine must get right; the small one tests the comparison at the limit.
STEPS = {"m": (0.01, 0.0001), "m3": (1.0, 0.005), "-": (0.001, 0.00001)}


def _zone_for_length(i: dict, length: float) -> float:
    """The cargo zone end that gives a cargo tank `length` (m)."""
    block = length * i["tanks_in_line"] / (1.0 - i["slop_share"])
    return i["zone_aft_m"] + block + i.get("pump_room_m", 0.0) + 2.0 * i["cofferdam_m"]


def move(i: dict, rule_id: str, target: float) -> tuple[str, float]:
    """The input (name, value) that makes `rule_id`'s governing value equal `target`."""
    if rule_id == "marpol/23.3.2/cargo-tank-length":
        return "zone_fwd_m", _zone_for_length(i, target * marpol_tank_length_limit(i))
    if rule_id.endswith("/cargo-tank-capacity"):
        lay = layout(i)
        length = target / (lay["tank_breadth_m"] * lay["tank_height_m"])
        return "zone_fwd_m", _zone_for_length(i, length)
    if rule_id.endswith("/cargo-tank-length"):
        return "zone_fwd_m", _zone_for_length(i, target)
    if rule_id.endswith("/cofferdam-width"):
        return "cofferdam_m", target
    if rule_id.endswith(("wing-width-min", "double-side-width")):
        return "double_side_m", target
    if rule_id.endswith(("double-bottom-depth", "double-bottom-min")):
        return "double_bottom_m", target
    if rule_id == "marpol/29/slop-tank-capacity":
        return "slop_share", target
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
    # `pass` is on the passing side of the limit by one step; `fail` on the other.
    target = entry["limit"] + (sign * step if side == "pass" else -sign * step)
    if target <= 0.0:
        raise Unreachable("the wanted value is not positive")
    name, value = move(i, rule_id, target)
    inputs = dict(i)
    inputs[name] = float(f"{value:.12g}")
    if name == "zone_fwd_m" and inputs[name] > inputs["loa_m"]:
        raise Unreachable("the cargo zone would reach past the hull")
    if name == "double_side_m" and inputs[name] >= inputs["beam_m"] / 2.0:
        raise Unreachable("the double side would take the whole breadth")
    expected = verdicts(inputs)
    return {
        "id": f"{base['id']}__{rule_id.split('/')[0]}_{rule_id.split('/')[1]}_{rule_id.split('/')[2]}__{side}_{magnitude}".replace("(", "").replace(")", ""),
        "title": f"{base['title']}, {rule_id} just {'inside' if side == 'pass' else 'across'} its limit ({magnitude} step)",
        "kind": "tank",
        "inputs": inputs,
        "expected": {k: {kk: vv for kk, vv in v.items() if kk != "compare"} for k, v in expected.items()},
        "mutant": {"parent": base["id"], "criterion": rule_id, "side": side, "magnitude": magnitude,
                   "moved": name, "target_value": float(f"{target:.10g}"), "grid": False},
        "provenance": {
            "verdict_basis": "derived",
            "source": "evaluation/oracle/tank_oracle.py from the parent case",
            "edition": base["provenance"]["edition"], "locator": base["provenance"]["locator"],
            "licence": base["provenance"]["licence"],
            "confidence": "high for the verdicts; a mutant is not an external published example",
        },
    }


def generate(bases: list[dict]) -> tuple[list[dict], list[dict]]:
    """(boundary cases, skipped) of the tank base cases."""
    docs, skipped = [], []
    for base in bases:
        for rule_id in verdicts(base["inputs"]):
            for side in ("pass", "fail"):
                for magnitude in ("big", "small"):
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
