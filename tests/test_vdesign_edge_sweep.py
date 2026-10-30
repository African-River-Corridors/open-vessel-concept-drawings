"""Validation is the only guard (issues #24, #41): a file `ovd validate` passes with 0 errors never
makes `simulate` raise, return an error value, or return a physically impossible
result (a negative cargo mass, cargo volume, draught, displacement, or hold width or height).

Label: property

The sweep is seeded, so it is the same every run. It writes one Design whose Options sit at the
hard-range edges of the built-in Defaults Profile, just inside and just outside them, at the soft
edges and in between, and at the cross-check boundaries (channel depth equal to its clearance,
freeboard just below the moulded depth). It runs the real `ovd validate` on the file and feeds every
Option with 0 errors to `simulate` (the Rust-backed package). None may raise, return
`{"error": ...}`, or return a negative result.

Risk stopped: an input combination the core refuses with an exception, or turns into a confident
wrong number (a 1 m beam gave negative cargo and draught), gets past validation.
"""
import json
import math
import random
import re
import subprocess
import sys
from pathlib import Path

import pytest

from ovd.simulate import simulate

ROOT = Path(__file__).resolve().parents[1]
SEED = 24
OPTIONS = 3000

# Glossary input name -> `simulate` key. A small test helper only; the full mapping belongs
# to the engine tickets (#23, #25).
TOP_LEVEL = {
    "cargo": "cargo",
    "length_overall_m": "loa_m",
    "beam_m": "beam_m",
    "moulded_depth_m": "depth_m",
    "min_freeboard_m": "freeboard_min_m",
    "lightship_factor_t_per_m3": "lightship_k_t_per_m3",
    "ruleset": "ruleset",
    "fill_ratio": "fill_ratio",
    "max_tank_od_m": "max_tank_od_m",
    "optimise_depth": "optimise_depth",
    "propulsive_efficiency": "eta_d",
    "max_speed_kn": "speed_max_kn",
    "air_draft_limit_m": "air_draft_limit_m",
    "draught_limit_m": "draught_limit_m",
}
CHANNEL = {
    "channel_depth_m": "depth_m",
    "channel_under_keel_clearance_m": "ukc_m",
    "channel_bottom_width_m": "bottom_width_m",
    "channel_bank_slope": "bank_slope",
    "channel_current_kn": "current_kn",
}
VESSEL = {"box_hold": "box_barge", "sloped_hopper_hold": "hopper_barge",
          "type_c_tanks": "gas_barge"}
CARGOS = {"dry_bulk": ["bauxite"], "liquid_gas": ["LNG", "LNG_ethane", "LPG_propane",
                                                  "LPG_butane", "LPG_mix"]}
CONTAINMENT_PHASE = {"box_hold": "dry_bulk", "sloped_hopper_hold": "dry_bulk",
                     "type_c_tanks": "liquid_gas"}
ONLY_FOR = {"lightship_factor_t_per_m3": "box_hold", "ruleset": "type_c_tanks",
            "fill_ratio": "type_c_tanks", "max_tank_od_m": "type_c_tanks",
            "optimise_depth": "type_c_tanks"}
NULLABLE = {"air_draft_limit_m", "draught_limit_m", "channel_depth_m", "channel_bottom_width_m"}
# Results that can never be negative on a real barge. Each is checked when present.
NON_NEGATIVE_RESULTS = ("cargo_t", "cargo_volume_m3", "draught_m", "displacement_t", "lightship_t")
NON_NEGATIVE_GEOMETRY = ("hold_top_width_m", "hold_bottom_width_m", "hold_height_m")
# The deepest hull the gas depth search tries (the core's DEFAULT_DEPTHS_M ends at 7.0 m).
DEPTH_SEARCH_MAX_M = 7.0
NUMBERS = [n for n in list(TOP_LEVEL) + list(CHANNEL)
           if n not in ("cargo", "ruleset", "optimise_depth")]


def to_simulate(inputs: dict) -> dict:
    """An Option's inputs (glossary names) as the `simulate` parameters."""
    p = {"vessel": VESSEL[inputs["cargo_containment"]],
         "self_propelled": inputs["formation"]["kind"] == "self_propelled"}
    for name, key in TOP_LEVEL.items():
        if name in inputs:
            p[key] = inputs[name]
    p["channel"] = {key: inputs[name] for name, key in CHANNEL.items()}
    return p


@pytest.fixture(scope="module")
def ovd() -> str:
    """Build the `ovd` binary and return its path."""
    out = subprocess.run(
        ["cargo", "build", "-q", "-p", "ovd-cli", "--bin", "ovd", "--message-format=json"],
        cwd=ROOT, check=True, capture_output=True, text=True).stdout
    exes = [m["executable"] for m in map(json.loads, out.splitlines())
            if m.get("reason") == "compiler-artifact" and m.get("executable")]
    assert exes, "cargo built no ovd binary"
    return exes[-1]


def candidates(entry: dict, rng: random.Random) -> list:
    """(inside, outside): values at, just inside and just outside the hard range, at the soft
    edges and between. An edge value goes to whichever side the hard range puts it."""
    hard, soft = entry.get("hard", {}), entry["soft"]
    vals = [soft["min"], soft["max"], rng.uniform(soft["min"], soft["max"]),
            rng.uniform(soft["min"], soft["max"]), soft["max"] * 3, soft["max"] * 10]
    if "default" in entry and entry["default"] is not None:
        vals.append(entry["default"])
    for key, inward in (("gt", math.inf), ("ge", math.inf), ("lt", -math.inf), ("le", -math.inf)):
        if key in hard:
            b = hard[key]
            vals += [b, math.nextafter(b, inward), math.nextafter(b, -inward),
                     b + (1e-3 if inward > 0 else -1e-3)]
            lo = b if inward > 0 else soft["min"]
            hi = soft["max"] if inward > 0 else b
            vals.append(rng.uniform(min(lo, hi), max(lo, hi)))
    ops = {"gt": float.__gt__, "ge": float.__ge__, "lt": float.__lt__, "le": float.__le__}
    inside = [v for v in vals if all(ops[k](float(v), b) for k, b in hard.items())]
    outside = [v for v in vals if v not in inside]
    return inside, outside


def sweep_options(profile: dict) -> list:
    rng = random.Random(SEED)
    entries = profile["inputs"]
    cand = {n: candidates(entries[n], rng) for n in NUMBERS}
    formations = [{"kind": "pushed_barge"}, {"kind": "self_propelled"},
                  {"kind": "pushed_convoy", "abreast": 2, "in_line": 3}]
    options = []
    for i in range(OPTIONS):
        containment = rng.choice(list(VESSEL))
        inputs = {
            "cargo": rng.choice(CARGOS[CONTAINMENT_PHASE[containment]]),
            "hull_form": "raked_box",
            "formation": rng.choice(formations),
            "cargo_containment": containment,
        }
        for name in NUMBERS:
            if ONLY_FOR.get(name, containment) != containment:
                continue
            if name in NULLABLE and rng.random() < 0.25:
                inputs[name] = None
            else:
                inside, outside = cand[name]
                inputs[name] = rng.choice(outside if outside and rng.random() < 0.03 else inside)
        if containment == "type_c_tanks":
            inputs["ruleset"] = "IGC_2G"      # the one rule set since issue #237
            inputs["optimise_depth"] = rng.random() < 0.3
        # The cross-check boundaries, which a random pick would rarely hit.
        roll = rng.random()
        if roll < 0.15 and inputs["channel_depth_m"] is not None:
            inputs["channel_depth_m"] = inputs["channel_under_keel_clearance_m"]
        elif roll < 0.25 and inputs["channel_depth_m"] is not None:
            inputs["channel_depth_m"] = math.nextafter(
                inputs["channel_under_keel_clearance_m"], -math.inf)
        elif roll < 0.35:
            inputs["min_freeboard_m"] = math.nextafter(inputs["moulded_depth_m"], 0.0)
        elif roll < 0.4:
            inputs["min_freeboard_m"] = inputs["moulded_depth_m"]
        elif roll < 0.5 and inputs.get("optimise_depth"):
            inputs["moulded_depth_m"] = 12.0
            inputs["min_freeboard_m"] = rng.choice(
                [DEPTH_SEARCH_MAX_M, math.nextafter(DEPTH_SEARCH_MAX_M, math.inf)])
        options.append({"name": f"S{i:04d}",
                        "inputs": {k: {"value": v, "source": "typed"} for k, v in inputs.items()}})
    return options + boundary_options()


# The built-in defaults, written out so each boundary Option holds every input.
TYPICAL = {"min_freeboard_m": 0.3, "lightship_factor_t_per_m3": 0.1093,
           "ruleset": "IGC_2G", "fill_ratio": 0.98, "max_tank_od_m": 16.0,
           "optimise_depth": False, "propulsive_efficiency": 0.45, "max_speed_kn": 10.0,
           "air_draft_limit_m": None, "draught_limit_m": None, "channel_depth_m": None,
           "channel_under_keel_clearance_m": 0.5, "channel_bank_slope": 3.0,
           "channel_current_kn": 0.0}


def boundary_options() -> list:
    """Each cross-check exactly at its boundary, for each Cargo Containment. All must pass."""
    out = []
    for containment in VESSEL:
        base = {"cargo": CARGOS[CONTAINMENT_PHASE[containment]][0], "hull_form": "raked_box",
                "formation": {"kind": "pushed_barge"}, "cargo_containment": containment,
                "length_overall_m": 76.5, "beam_m": 11.4, "moulded_depth_m": 3.5,
                "channel_bottom_width_m": 60.0}
        base.update({k: v for k, v in TYPICAL.items() if ONLY_FOR.get(k, containment) == containment})
        cases = [
            {"channel_depth_m": 0.5, "channel_under_keel_clearance_m": 0.5},
            {"channel_depth_m": 2.0, "channel_under_keel_clearance_m": 2.0},
            {"channel_depth_m": 0.1, "channel_under_keel_clearance_m": 0.1},
            # The block coefficient at the squat method's 0.3 floor was a case here; since issue #448
            # it is a result of the hull's sections, not an input.
            {"channel_depth_m": 3.0},
            {"min_freeboard_m": math.nextafter(3.5, 0.0)},
        ]
        if containment != "type_c_tanks":
            # The dry-bulk hold section at exactly zero width, and at exactly zero height: zero
            # cargo, which the engine reports as a failed check. Not negative, so not refused.
            cases.append({"beam_m": 2.0})
            cases.append({"moulded_depth_m": 0.2, "min_freeboard_m": 0.0})
        if containment == "type_c_tanks":
            cases.append({"moulded_depth_m": 12.0, "optimise_depth": True,
                          "min_freeboard_m": DEPTH_SEARCH_MAX_M})
        for j, case in enumerate(cases):
            inputs = {**base, **case}
            out.append({"name": f"B{containment[:4]}{j}",
                        "inputs": {k: {"value": v, "source": "typed"} for k, v in inputs.items()}})
    return out


def impossible(result: dict) -> list:
    """What is wrong with one `simulate` result: an error value or a negative result."""
    if "error" in result:
        return [f"error value: {result['error']}"]
    found = [f"{k} = {result['results'][k]}" for k in NON_NEGATIVE_RESULTS
             if result["results"].get(k) is not None and result["results"][k] < 0]
    found += [f"{k} = {result['geometry'][k]}" for k in NON_NEGATIVE_GEOMETRY
              if result["geometry"].get(k) is not None and result["geometry"][k] < 0]
    return found


def test_the_impossible_result_check_catches_an_error_value_and_a_negative():
    """The sweep's own check can fail: an error value and the 1 m beam's negatives are caught."""
    assert impossible({"error": "Bauxite cannot be carried in a gas barge."})
    one_metre = simulate(to_simulate({
        "cargo": "bauxite", "hull_form": "raked_box", "formation": {"kind": "pushed_barge"},
        "cargo_containment": "box_hold", "length_overall_m": 76.5, "beam_m": 1.0,
        "moulded_depth_m": 3.5, "channel_bottom_width_m": None, **{k: v for k, v in TYPICAL.items()
                                   if ONLY_FOR.get(k, "box_hold") == "box_hold"}}))
    found = impossible(one_metre)
    assert any(f.startswith("cargo_t = -") for f in found), found
    # Issue #448: the draught comes from the hull's sections, so a negative displacement floats at
    # zero draught, not at a negative one; the negative cargo and hold width still show.
    assert any(f.startswith("hold_top_width_m = -") for f in found), found


def test_every_option_validate_passes_runs_in_simulate(ovd, tmp_path):
    profile = json.loads(subprocess.run([ovd, "profile"], check=True, capture_output=True,
                                        text=True).stdout)
    options = sweep_options(profile)
    design = {"format": "vdesign", "format_version": 1, "engine_version": "0.1.0",
              "options": options}
    path = tmp_path / "sweep.vdesign"
    path.write_text(json.dumps(design), encoding="utf-8")
    out = subprocess.run([ovd, "validate", str(path)], capture_output=True, text=True)
    refused = set(re.findall(r"^error: Option '(\w+)':", out.stdout, re.M))
    assert out.returncode in (0, 1), out.stdout + out.stderr

    passed = [o for o in options if o["name"] not in refused]
    boundaries = sorted(n for n in refused if n.startswith("B"))
    assert not boundaries, f"boundary Options were refused: {boundaries}\n{out.stdout}"
    # The sweep must exercise both sides of the guard, or it proves nothing.
    assert len(passed) >= 200, f"only {len(passed)} Options passed validation"
    assert len(refused) >= 200, f"only {len(refused)} Options were refused"

    raised, wrong = [], []
    for o in passed:
        inputs = {k: v["value"] for k, v in o["inputs"].items()}
        try:
            result = simulate(to_simulate(inputs))
        except Exception as exc:  # noqa: BLE001 - any exception breaks the rule
            raised.append(f"{o['name']}: {type(exc).__name__}: {exc}\n  {json.dumps(inputs)}")
            continue
        found = impossible(result)
        if found:
            wrong.append(f"{o['name']}: {', '.join(found)}\n  {json.dumps(inputs)}")
    assert not raised, f"{len(raised)} valid Option(s) made simulate raise:\n" + "\n".join(
        raised[:10])
    assert not wrong, (f"{len(wrong)} valid Option(s) gave an error value or an impossible "
                       "result:\n" + "\n".join(wrong[:10]))
    # Every just-inside hard edge of every number appears in at least one valid Option.
    seen = {n: {o["inputs"][n]["value"] for o in passed if n in o["inputs"]} for n in NUMBERS}
    for name in NUMBERS:
        for key, b in profile["inputs"][name].get("hard", {}).items():
            edge = b if key in ("ge", "le") else math.nextafter(
                b, math.inf if key == "gt" else -math.inf)
            assert edge in seen[name], f"no valid Option holds {name} = {edge} ({key} {b})"
