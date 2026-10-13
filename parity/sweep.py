"""Tier-2 parity cases: a seeded sweep across the valid input space of every entry point.

    python3 parity/sweep.py          # rewrite the tier-2 cases in parity/cases/*.json

Only `cargo xtask regen` should run this. It keeps every tier-1 and tier-3 case as it is and
replaces the tier-2 cases with the points generated here. Rust never needs a random generator:
the points are committed as data (spec 0001).

Rules:
- one `random.Random` per (entry point, function), seeded from SEED and the names, so adding a
  function never moves another function's points;
- floats are rounded to 4 decimals (lump sums to whole numbers), so case files stay readable;
  4 decimals rarely land on a tie of the 1- and 2-decimal formats in check messages;
- a point is kept only if the frozen Python returns a normal result (tier 2 is the VALID input
  space; tier 3 holds the error cases) and the point is not on a knife edge: every float input
  except the speed-grid inputs (NOT_NUDGED) is moved by a relative 1e-12 both ways (about a
  thousand ulps), and every discrete output (integers, booleans, text, list lengths) must stay
  the same and every float output must stay within the parity tolerance. A knife-edge point is dropped and another one drawn, so a last-digit difference between
  Python and Rust cannot flip a branch;
- the number of points per function keeps all parity files under about 5 MB.
Standard library only (plus the frozen package, through the driver).
"""
from __future__ import annotations

import importlib.util
import json
import random
import sys
from pathlib import Path

PARITY = Path(__file__).resolve().parent
_spec = importlib.util.spec_from_file_location("parity_driver", PARITY / "driver.py")
driver = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(driver)

SEED = 20261013
NUDGE = 1e-12           # relative move of every float input in the knife-edge test
# A float output has jumped when the nudge moves it outside the parity tolerance itself.
TOLERANCE = json.loads((PARITY / "tolerance.json").read_text(encoding="utf-8"))
MAX_DRAWS = 40          # draws per wanted point before the generator gives up
# Speed-grid inputs are not nudged. The grid is built by repeated addition of these exact
# values, which Rust repeats bit for bit; a nudge would only move the last grid point.
NOT_NUDGED = {"speed_min_knots", "speed_max_knots", "speed_step_knots", "speed_max_kn"}


# Why a function's sweep points last changed. The oracle guard needs a new reason on every
# changed case, so a change to a sampler adds (or replaces) its entry here.
REVISIONS = {
    ("barge_cost", "compute"): "issue #40: the tug hull steel, battery energy and propulsion "
                               "power are varied, not left at their defaults",
    ("gas_arrangement", "fit"): "issue #40: the block coefficient and the tank diameter cap are "
                                "varied, not left at their defaults",
}


def reason(entry_point: str = "", fn: str = "") -> str:
    text = f"tier 2: seeded sweep (parity/sweep.py, seed {SEED})"
    note = REVISIONS.get((entry_point, fn))
    return f"{text}; last changed: {note}" if note else text


# --- Knife-edge test ---------------------------------------------------------------------------

def _nudge(value, factor):
    if isinstance(value, bool) or value is None or isinstance(value, (int, str)):
        return value
    if isinstance(value, float):
        return value * factor
    if isinstance(value, dict):
        return {k: v if k in NOT_NUDGED else _nudge(v, factor) for k, v in value.items()}
    if isinstance(value, list):
        return [_nudge(v, factor) for v in value]
    return value


def jumps(a: float, b: float) -> bool:
    """True if two float outputs differ by more than the parity tolerance (`math.isclose`)."""
    if isinstance(a, str) or isinstance(b, str):        # tagged non-finite values
        return a != b
    return abs(a - b) > max(TOLERANCE["abs"], TOLERANCE["rel"] * max(abs(a), abs(b)))


def _same_shape(a, b) -> bool:
    if isinstance(a, float) and isinstance(b, float):
        return not jumps(a, b)
    if type(a) is not type(b):
        return False
    if isinstance(a, dict):
        return list(a) == list(b) and all(_same_shape(a[k], b[k]) for k in a)
    if isinstance(a, list):
        return len(a) == len(b) and all(_same_shape(x, y) for x, y in zip(a, b))
    return a == b


def _run(entry_point: str, case_input: dict):
    try:
        return driver.run_case(entry_point, case_input)
    except Exception as exc:  # noqa: BLE001 - any other exception is not a valid sweep point
        return {"crash": f"{type(exc).__name__}: {exc}"}


def is_error(output: dict) -> bool:
    ok = output.get("ok")
    return "ok" not in output or (isinstance(ok, dict) and "error" in ok)


def is_knife_edge(entry_point: str, case_input: dict) -> bool:
    """True if a relative NUDGE of the float inputs changes the result's shape."""
    base = _run(entry_point, case_input)
    for factor in (1.0 + NUDGE, 1.0 - NUDGE):
        moved = {"fn": case_input["fn"], "args": _nudge(case_input.get("args", {}), factor)}
        if not _same_shape(base, _run(entry_point, moved)):
            return True
    return False


# --- Samplers ----------------------------------------------------------------------------------

def u(rng: random.Random, lo: float, hi: float, nd: int = 4) -> float:
    return round(rng.uniform(lo, hi), nd)


GAS = ["LNG", "LNG_ethane", "LPG_propane", "LPG_butane", "LPG_mix"]
RULESETS = ["IGC_2G", "IGC_2G_EXACT", "ADN_G"]
VESSEL_TYPES = ["IGF", "IGC_1G", "IGC_2G", "IGC_3G", "ADN_G", "IGC_2G_EXACT"]
MATERIALS = ["304L", "316L", "9Ni"]
HEADS = ["hemisphere", "ellipsoidal_2to1"]
STANDARD_IDS = ['CEMT_IV', 'CEMT_Va', 'CEMT_Vb', 'CEMT_VIa', 'CEMT_VIa_motor_NL2011', 'CN_I_motor',
                'CN_II_motor', 'CN_III_motor', 'CN_III_convoy_2ab', 'CN_IV_motor',
                'CN_II_convoy_2ab', 'CN_I_convoy_2ab']


def s_get_cargo(rng):
    return {"cargo_type": rng.choice(GAS + ["bauxite"])}


def s_distance_d_vc(rng):
    return {"vc": u(rng, 50.0, 40000.0)}


def s_d_vc_exact(rng):
    return {"vc_individual_m3": u(rng, 50.0, 40000.0)}


def s_side_protection(rng):
    return {"vessel_type": rng.choice(VESSEL_TYPES), "beam": u(rng, 6.0, 25.0),
            "d_vc": u(rng, 0.8, 2.0)}


def s_bottom_protection(rng):
    return {"vessel_type": rng.choice(VESSEL_TYPES + [None]), "beam": u(rng, 6.0, 40.0),
            "d_vc": u(rng, 0.8, 2.0)}


def s_required_clearances(rng):
    return {"vessel_type_str": rng.choice(VESSEL_TYPES), "beam": u(rng, 6.0, 25.0),
            "d_vc": u(rng, 0.8, 2.0)}


def s_check_protective_location(rng):
    return {"vessel_type_str": rng.choice(VESSEL_TYPES), "beam": u(rng, 6.0, 25.0),
            "total_tank_volume": u(rng, 100.0, 30000.0), "side_clearance": u(rng, 0.5, 3.0),
            "double_bottom_height": u(rng, 0.4, 1.8), "tank_support_height": u(rng, 0.2, 0.8)}


def s_get_tank_material(rng):
    return {"material_key": rng.choice(MATERIALS)}


def _thickness_args(rng, size_key, lo, hi):
    return {"design_pressure_mpa": u(rng, 0.1, 2.0), size_key: u(rng, lo, hi),
            "allowable_stress_mpa": u(rng, 100.0, 250.0), "weld_efficiency": u(rng, 0.85, 1.0)}


def s_cylinder(rng):
    return _thickness_args(rng, "inner_radius_mm", 500.0, 6000.0)


def s_hemisphere(rng):
    return _thickness_args(rng, "inner_radius_mm", 500.0, 6000.0)


def s_ellipsoidal(rng):
    return _thickness_args(rng, "inner_diameter_mm", 1000.0, 12000.0)


def s_wall_thicknesses(rng):
    return {"outer_diameter_m": u(rng, 1.5, 12.0), "design_pressure_mpa": u(rng, 0.1, 2.0),
            "material_key": rng.choice(MATERIALS), "head_type": rng.choice(HEADS),
            "weld_efficiency": u(rng, 0.85, 1.0)}


def _hull(rng):
    depth = u(rng, 2.5, 7.5)
    return {"loa": u(rng, 30.0, 135.0), "beam": u(rng, 6.0, 24.0), "depth": depth,
            "bow_rake_length": rng.choice([0.0, u(rng, 2.0, 12.0)]),
            "bow_rake_rise": u(rng, 0.5, depth),
            "stern_rake_length": rng.choice([0.0, u(rng, 2.0, 8.0)]),
            "stern_rake_rise": u(rng, 0.5, depth)}


def s_hull_quantities(rng):
    h = _hull(rng)
    return {"hull": h, "draft": u(rng, 0.2, h["depth"])}


def _barge_inputs(rng):
    """Plain BargeInputs data with a tank arrangement that usually fits."""
    depth = u(rng, 3.0, 7.5)
    beam = u(rng, 8.0, 22.0)
    inp = {"loa": u(rng, 50.0, 135.0), "beam": beam, "depth": depth,
           "max_draft": round(depth - u(rng, 0.3, 1.5), 3),
           "vertical_clearance": u(rng, 6.0, 20.0),
           "vessel_type": rng.choice(VESSEL_TYPES),
           "bow_rake_length": rng.choice([0.0, u(rng, 2.0, 10.0)]),
           "stern_rake_length": rng.choice([0.0, u(rng, 2.0, 6.0)]),
           "side_clearance": u(rng, 0.8, 2.5),
           "n_tanks_wide": rng.choice([1, 1, 2, 3]) if beam > 14 else 1,
           "n_tanks_long": rng.choice([1, 2, 3, 4]),
           "tank_wall_thickness": rng.choice([None, 25.0, u(rng, 10.0, 40.0)]),
           "tank_head_type": rng.choice(HEADS),
           "tank_material": rng.choice(MATERIALS),
           "design_pressure": u(rng, 0.2, 1.0),
           "insulation_thickness": u(rng, 100.0, 400.0),
           "cargo_type": rng.choice(GAS),
           "fill_ratio": u(rng, 0.85, 0.98),
           "block_coefficient": u(rng, 0.75, 0.95),
           "water_density": rng.choice([1.0, 1.025]),
           "double_bottom_height": u(rng, 0.6, 1.6),
           "tank_support_height": u(rng, 0.3, 0.8)}
    if rng.random() < 0.3:
        inp["bow_rake_rise"] = u(rng, 0.5, depth)
    return inp


def s_tank_arrangement(rng):
    return {"inputs": _barge_inputs(rng)}


def s_free_surface(rng):
    return {"inner_radius": u(rng, 1.0, 6.0), "cylinder_length": u(rng, 5.0, 40.0),
            "fill_ratio": u(rng, 0.05, 0.98), "cargo_density": u(rng, 0.4, 0.6)}


def s_weights(rng):
    return {"inputs": _barge_inputs(rng)}


def s_equilibrium_draft(rng):
    h = _hull(rng)
    full = h["loa"] * h["beam"] * h["depth"]
    return {"hull": h, "total_weight": round(full * u(rng, 0.1, 0.8), 1),
            "water_density": rng.choice([1.0, 1.025]), "max_draft": h["depth"],
            "block_coefficient": u(rng, 0.75, 1.0)}


def s_stability(rng):
    inp = _barge_inputs(rng)
    return {"inputs": inp, "cargo_mass": u(rng, 200.0, 5000.0),
            "cargo_density_t": u(rng, 0.42, 0.6), "draft": u(rng, 1.0, inp["max_draft"])}


def s_trim(rng):
    inp = _barge_inputs(rng)
    return {"inputs": inp, "cargo_mass": u(rng, 200.0, 5000.0),
            "draft": u(rng, 1.0, inp["max_draft"])}


def s_aft(rng):
    inp = {"beam": u(rng, 7.0, 22.0), "depth": u(rng, 2.5, 7.5),
           "n_thrusters": rng.choice([1, 2, 3, 4]),
           "n_battery_units": rng.choice([0, 1, 2]), "n_accom_units": rng.choice([1, 2, 3]),
           "wheelhouse_width_m": u(rng, 4.0, 8.0)}
    if rng.random() < 0.5:
        inp["sightline_obstruction_m"] = u(rng, 6.0, 16.0)
    return {"inputs": inp}


def s_separation(rng):
    return {"inputs": {"cofferdam_m": u(rng, 0.0, 1.5), "opening_setback_m": u(rng, 0.0, 4.0),
                       "fwd_end_bulkhead_m": u(rng, 0.0, 1.0),
                       "is_pressure_tank": rng.choice([True, False]),
                       "side_clearance_m": rng.choice([0.0, u(rng, 0.5, 2.5)]),
                       "double_bottom_m": rng.choice([0.0, u(rng, 0.4, 1.5)])}}


def _capacity_inputs(rng, gas=None):
    gas = rng.random() < 0.6 if gas is None else gas
    depth = u(rng, 3.0, 7.0)
    inp = {"loa": u(rng, 50.0, 130.0), "beam": u(rng, 8.0, 22.0), "depth": depth,
           "cargo_type": rng.choice(GAS) if gas else "bauxite",
           "block_coefficient": u(rng, 0.78, 0.92),
           "self_propelled": rng.choice([True, False])}
    if gas:
        inp["vessel_type"] = rng.choice(RULESETS)
        inp["fill_ratio"] = u(rng, 0.9, 0.98)
        if rng.random() < 0.3:
            inp["n_tanks_wide"] = rng.choice([1, 2])
    else:
        inp["hold_type"] = rng.choice(["sloped_hopper", "box"])
        if inp["hold_type"] == "box":
            inp["lightship_k_t_per_m3"] = u(rng, 0.08, 0.14, 4)
    if rng.random() < 0.3:
        inp["air_draft_limit_m"] = u(rng, 5.0, 12.0)
    if rng.random() < 0.3:
        inp["ballast_capacity_t"] = u(rng, 50.0, 800.0)
    return inp


def s_capacity(rng):
    return {"inputs": _capacity_inputs(rng)}


def s_tanks_wide(rng):
    return {"beam_m": u(rng, 6.0, 40.0), "side_clearance_m": u(rng, 0.8, 3.0),
            "tank_gap_transverse_m": u(rng, 0.0, 2.0), "max_od_m": u(rng, 6.0, 12.0)}


def s_journey(rng):
    cap = _capacity_inputs(rng)
    keep = {k: cap[k] for k in ("loa", "beam", "depth")}
    return {"capacity_inputs": cap,
            "inputs": dict(keep, block_coefficient=cap["block_coefficient"],
                           design_depth_m=u(rng, 1.5, 8.0), ukc_m=u(rng, 0.0, 1.0))}


def _fit_args(rng):
    ruleset = rng.choice(RULESETS)
    args = {"loa": u(rng, 55.0, 120.0), "beam": u(rng, 8.0, 18.0), "cargo": rng.choice(GAS),
            "t_max": u(rng, 2.5, 5.0), "ruleset": ruleset, "fill_ratio": u(rng, 0.9, 0.98),
            "self_propelled": rng.choice([True, False]),
            "depths": sorted(rng.sample([4.0, 4.5, 5.0, 5.5, 6.0, 6.5, 7.0], 2))}
    if ruleset == "ADN_G":
        args["adn_tank_cap_m3"] = rng.choice([380.0, 1000.0])
    return args


def s_fit(rng):
    # Drawn after the shared fit arguments, so the assess points (which reuse them) stay put.
    args = _fit_args(rng)
    args["block_coefficient"] = u(rng, 0.78, 0.92)
    args["max_tank_od_m"] = u(rng, 6.0, 12.0)
    return args


def s_smallest_loa(rng):
    return {"beam": u(rng, 8.0, 16.0), "cargo": rng.choice(GAS), "t_max": u(rng, 2.5, 4.5)}


def s_required_side_clearance(rng):
    per = u(rng, 100.0, 3000.0)
    return {"ruleset": rng.choice(RULESETS), "beam": u(rng, 8.0, 18.0),
            "total_m3": round(per * rng.choice([2, 4, 6]), 1), "per_tank_m3": per,
            "insulation_mm": u(rng, 100.0, 400.0)}


def s_box_gz(rng):
    depth = u(rng, 3.0, 7.0)
    return {"beam": u(rng, 7.0, 20.0), "depth": depth, "draught": u(rng, 1.0, depth - 0.3),
            "kg": u(rng, 1.5, depth), "phi_deg": u(rng, -45.0, 60.0)}


def s_segment_fill(rng):
    return {"radius": u(rng, 0.5, 6.0), "volume_fraction": u(rng, 0.01, 0.99)}


def s_assess(rng):
    fit = _fit_args(rng)
    return {"fit": fit, "criteria_set": rng.choice(["IS_CODE_2008", "ADN_9_3_1_14_2"]),
            "block_coefficient": u(rng, 0.78, 0.92)}


def s_squat(rng):
    draft = u(rng, 1.5, 5.0)
    depth = round(draft + u(rng, 0.3, 4.0), 3)
    beam = u(rng, 8.0, 22.0)
    return {"inputs": {"speed_through_water_kn": u(rng, 0.0, 12.0), "draft_m": draft,
                       "beam_m": beam, "lpp_m": u(rng, 50.0, 130.0),
                       "block_coefficient": u(rng, 0.7, 0.95),
                       "water_depth_m": rng.choice([0.0, depth]),
                       "channel_area_m2": rng.choice([0.0, round(depth * beam * u(rng, 3.0, 12.0), 1)]),
                       "safety_margin_m": u(rng, 0.1, 0.5)}}


def s_max_speed(rng):
    draft = u(rng, 1.5, 5.0)
    depth = round(draft + u(rng, 0.3, 4.0), 3)
    beam = u(rng, 8.0, 22.0)
    return {"draft_m": draft, "beam_m": beam, "lpp_m": u(rng, 50.0, 130.0),
            "water_depth_m": depth, "channel_area_m2": round(depth * beam * u(rng, 3.0, 12.0), 1),
            "block_coefficient": u(rng, 0.7, 0.95), "safety_margin_m": u(rng, 0.1, 0.5)}


def s_waves(rng):
    return {"inputs": {"significant_wave_height_m": u(rng, 0.0, 3.0),
                       "beam_waterline_m": u(rng, 7.0, 24.0), "lpp_m": u(rng, 50.0, 130.0),
                       "bow_waterline_length_m": rng.choice([0.0, u(rng, 5.0, 30.0)]),
                       "heading_deg_off_bow": u(rng, 0.0, 180.0),
                       "peak_wave_period_s": rng.choice([0.0, u(rng, 3.0, 10.0)]),
                       "water_density_t_m3": rng.choice([1.0, 1.025]),
                       "speed_kn": u(rng, 0.0, 12.0)}}


def _environment(rng, i):
    depth = u(rng, 3.0, 20.0)
    bottom = u(rng, 30.0, 300.0)
    return {"name": f"env{i}", "water_depth": depth, "channel_width_bottom": bottom,
            "channel_width_surface": round(bottom + 2.0 * 3.0 * depth, 2),
            "current_speed_knots": u(rng, 0.0, 2.0),
            "water_density": rng.choice([1.0, 1.025]),
            "significant_wave_height_m": rng.choice([0.0, 0.0, u(rng, 0.3, 2.0)]),
            "apply_squat_limit": rng.choice([True, False])}


def s_resistance(rng):
    lwl = u(rng, 50.0, 130.0)
    return {"inputs": {"loa": round(lwl + u(rng, 0.0, 5.0), 2), "lwl": lwl,
                       "beam": u(rng, 8.0, 24.0), "draft": u(rng, 1.5, 5.0),
                       "block_coefficient": u(rng, 0.75, 0.92),
                       "tug": {"engine_power_kw": u(rng, 500.0, 4000.0),
                               "propulsive_efficiency": u(rng, 0.35, 0.6)},
                       "environments": [_environment(rng, i) for i in range(rng.choice([1, 2]))],
                       "speed_min_knots": 2.0, "speed_max_knots": u(rng, 6.0, 10.0),
                       "speed_step_knots": 1.0}}


def s_energy(rng):
    lwl = u(rng, 50.0, 120.0)
    return {"inputs": {"beam": u(rng, 8.0, 18.0), "loa": round(lwl + 2.0, 2), "lwl": lwl,
                       "draft": u(rng, 1.5, 4.0), "block_coefficient": u(rng, 0.78, 0.92),
                       "channel_depth_m": u(rng, 3.0, 8.0),
                       "channel_bottom_width_m": u(rng, 25.0, 120.0),
                       "channel_top_width_m": u(rng, 130.0, 200.0),
                       "current_knots": u(rng, 0.0, 1.5),
                       "fresh_water": rng.choice([True, False]),
                       "also_open_sea": rng.choice([True, False]),
                       "target_service_speed_knots": u(rng, 4.0, 9.0),
                       "one_way_km": u(rng, 20.0, 400.0),
                       "trips_per_year": u(rng, 50.0, 350.0),
                       "speed_step_knots": 0.5}}


RATE_KEYS = ["hull_steel_rate", "tank_shell_rate", "tank_supports_rate", "insulation_rate",
             "piping_rate", "outfitting_rate", "cargo_system_lump", "paint_anodes_misc",
             "battery_rate_per_kwh", "propulsion_rate_per_kw", "wheelhouse_accom_lump",
             "tug_outfitting_lump", "yard_margin_fraction", "delivery_lump"]


def _rates(rng):
    r = {k: u(rng, 500.0, 20000.0) for k in RATE_KEYS}
    for k in ("cargo_system_lump", "paint_anodes_misc", "wheelhouse_accom_lump",
              "tug_outfitting_lump", "delivery_lump"):
        r[k] = u(rng, 5e4, 2e6, 0)
    r["yard_margin_fraction"] = u(rng, 0.0, 0.3)
    return r


def s_cost(rng):
    inputs = {k: u(rng, lo, hi) for k, lo, hi in (
        ("hull_steel_t", 100.0, 600.0), ("wing_plate_t", 10.0, 80.0),
        ("weather_shield_t", 10.0, 90.0), ("tank_shell_t", 50.0, 500.0),
        ("tank_supports_t", 10.0, 150.0), ("insulation_t", 5.0, 60.0),
        ("piping_machinery_t", 10.0, 60.0), ("outfitting_t", 10.0, 60.0),
        ("tug_hull_steel_t", 60.0, 200.0), ("battery_kwh", 200.0, 2000.0),
        ("propulsion_kw", 100.0, 600.0))}
    return {"inputs": inputs, "rates": rng.choice([None, _rates(rng)])}


def s_parse_yaml(rng):
    keys = rng.sample(RATE_KEYS, rng.randint(1, 6))
    lines = ["# sweep rates"] + [f"{k}: {u(rng, 0.0, 1e6)}" for k in keys]
    if rng.random() < 0.5:
        lines.insert(1, "")
    return {"text": "\n".join(lines) + "\n"}


def s_grid(rng):
    return {"barge_lengths_m": sorted(u(rng, 40.0, 100.0) for _ in range(2)),
            "beams_m": sorted(u(rng, 8.0, 15.0) for _ in range(2)),
            "depth_m": u(rng, 4.5, 7.0), "cargo_type": rng.choice(GAS)}


def s_n_tanks(rng):
    return {"barge_length_m": u(rng, 20.0, 200.0), "max_tank_length_m": u(rng, 10.0, 40.0)}


def s_to_ga(rng):
    kw = {"motor_vessel": rng.choice([True, False])}
    if rng.random() < 0.5:
        kw["laden_draft_m"] = u(rng, 1.5, 3.0)
    return {"inputs": _capacity_inputs(rng, gas=True), "kwargs": kw}


def s_is_gas(rng):
    return {"inputs": _capacity_inputs(rng)}


def s_get_standard(rng):
    return {"standard_id": rng.choice(STANDARD_IDS)}


def s_standards_for(rng):
    return {"cargo_mode": rng.choice(["dry_bulk", "liquid_gas"])}


def s_validate(rng):
    lo_l = u(rng, 60.0, 190.0)
    lo_t, lo_w = u(rng, 1.5, 3.5), u(rng, 500.0, 5000.0, 0)
    flip = rng.random() < 0.3
    convoy = rng.random() < 0.4
    std = {"standard_id": "SWEEP", "family": rng.choice(["CEMT", "CN"]), "class_name": "S",
           "formation": "pushed_convoy" if convoy else "motor_vessel",
           "loa_m": [lo_l, round(lo_l + (-5.0 if flip else u(rng, 0.0, 20.0)), 1)],
           "beam_m": u(rng, 8.0, 35.0),
           "draught_m": [lo_t, round(lo_t + u(rng, 0.0, 1.0), 2)],
           "dwt_t": [lo_w, round(lo_w + u(rng, 0.0, 3000.0, 0), 0)],
           "source": rng.choice(["", "sweep point"])}
    if convoy:
        std["n_abreast"] = rng.choice([1, 2, 3])
        std["unit_beam_m"] = u(rng, 8.0, 12.0)
    return {"extra": {"SWEEP": std}}


def s_simulate(rng):
    vessel = rng.choice(["box_barge", "hopper_barge", "gas_barge", "gas_barge"])
    p = {"vessel": vessel, "loa_m": u(rng, 55.0, 110.0), "beam_m": u(rng, 8.0, 15.0),
         "depth_m": u(rng, 3.5, 7.0), "cb": u(rng, 0.8, 0.92),
         "freeboard_min_m": u(rng, 0.2, 0.6), "self_propelled": rng.choice([True, False]),
         "speed_max_kn": 8.0}
    if vessel == "gas_barge":
        p["cargo"] = rng.choice(GAS)
        p["ruleset"] = rng.choice(RULESETS)
        p["fill_ratio"] = rng.choice([None, u(rng, 0.9, 0.98)])
    else:
        p["cargo"] = "bauxite"
        if vessel == "box_barge":
            p["lightship_k_t_per_m3"] = u(rng, 0.08, 0.14, 4)
    if rng.random() < 0.5:
        p["draught_limit_m"] = u(rng, 2.5, 4.5)
    if rng.random() < 0.4:
        p["channel"] = {"depth_m": u(rng, 3.5, 8.0), "ukc_m": u(rng, 0.3, 0.8),
                        "bottom_width_m": u(rng, 30.0, 120.0)}
    if rng.random() < 0.3:
        p["air_draft_limit_m"] = u(rng, 6.0, 12.0)
    return {"params": p}


# (entry point, function, number of points, sampler). The count is set by the output size of
# one point: the whole parity folder must stay under about 5 MB.
SPACES = [
    ("cargo", "get_cargo", 6, s_get_cargo),
    ("protective_location", "distance_d_vc", 25, s_distance_d_vc),
    ("protective_location", "d_vc_exact", 25, s_d_vc_exact),
    ("protective_location", "required_side_protection", 20, s_side_protection),
    ("protective_location", "required_bottom_protection", 20, s_bottom_protection),
    ("protective_location", "compute_required_clearances", 20, s_required_clearances),
    ("protective_location", "check_protective_location", 25, s_check_protective_location),
    ("pressure_vessel", "get_tank_material", 3, s_get_tank_material),
    ("pressure_vessel", "cylinder_wall_thickness", 20, s_cylinder),
    ("pressure_vessel", "hemisphere_head_thickness", 20, s_hemisphere),
    ("pressure_vessel", "ellipsoidal_head_thickness", 20, s_ellipsoidal),
    ("pressure_vessel", "calculate_wall_thicknesses", 30, s_wall_thicknesses),
    ("hull", "quantities", 40, s_hull_quantities),
    ("aft_arrangement", "compute", 30, s_aft),
    ("gas_separation", "compute", 30, s_separation),
    ("tanks", "calculate_tank_arrangement", 40, s_tank_arrangement),
    ("tanks", "free_surface_moment_cylindrical", 25, s_free_surface),
    ("weights", "calculate_weights", 40, s_weights),
    ("hydrostatics", "find_equilibrium_draft", 30, s_equilibrium_draft),
    ("hydrostatics", "calculate_stability", 30, s_stability),
    ("hydrostatics", "calculate_trim", 30, s_trim),
    ("barge_capacity", "compute", 25, s_capacity),
    ("barge_capacity", "tanks_wide_for_beam", 25, s_tanks_wide),
    ("journey_derating", "compute", 30, s_journey),
    ("gas_arrangement", "fit", 12, s_fit),
    ("gas_arrangement", "smallest_viable_loa", 8, s_smallest_loa),
    ("gas_arrangement", "required_side_clearance", 20, s_required_side_clearance),
    ("gas_stability", "box_gz", 30, s_box_gz),
    ("gas_stability", "segment_fill", 20, s_segment_fill),
    ("gas_stability", "assess", 8, s_assess),
    ("squat", "compute", 30, s_squat),
    ("squat", "max_speed_for_ukc", 20, s_max_speed),
    ("waves", "compute", 30, s_waves),
    ("resistance", "compute", 8, s_resistance),
    ("electric_energy", "compute", 10, s_energy),
    ("barge_cost", "compute", 25, s_cost),
    ("barge_cost", "parse_flat_yaml", 15, s_parse_yaml),
    ("design_grid", "sweep", 10, s_grid),
    ("design_grid", "n_tanks_for_length", 25, s_n_tanks),
    ("ga_adapter", "to_ga", 10, s_to_ga),
    ("ga_adapter", "is_gas", 15, s_is_gas),
    ("vessel_standards", "get_standard", 12, s_get_standard),
    ("vessel_standards", "standards_for", 2, s_standards_for),
    ("vessel_standards", "validate_catalogue", 15, s_validate),
    ("simulate", "simulate", 10, s_simulate),
]

# Entry points whose spec functions have nothing to sweep still appear, with their swept ones.
ENTRY_POINTS = sorted({ep for ep, _, _, _ in SPACES})


def points(entry_point: str, fn: str, n: int, sampler) -> list[dict]:
    rng = random.Random(f"{SEED}:{entry_point}:{fn}")
    out, draws = [], 0
    while len(out) < n:
        draws += 1
        if draws > n * MAX_DRAWS:
            raise SystemExit(f"{entry_point}.{fn}: only {len(out)} of {n} valid sweep points")
        case_input = {"fn": fn, "args": sampler(rng)}
        if is_error(_run(entry_point, case_input)) or is_knife_edge(entry_point, case_input):
            continue
        out.append({"id": f"{fn}/sweep/{len(out):03d}", "tier": 2, "reason": reason(entry_point, fn),
                    "input": case_input})
    return out


def generate() -> dict[str, list[dict]]:
    """Tier-2 cases by entry point, in SPACES order. Deterministic for a given SEED."""
    cases: dict[str, list[dict]] = {ep: [] for ep in ENTRY_POINTS}
    for ep, fn, n, sampler in SPACES:
        cases[ep].extend(points(ep, fn, n, sampler))
    return cases


def render(entry_point: str, tier2: list[dict]) -> str:
    """The case file text with its tier-2 cases replaced by `tier2` (tier 1 and 3 kept)."""
    path = PARITY / "cases" / f"{entry_point}.json"
    doc = json.loads(path.read_text(encoding="utf-8"))
    doc["cases"] = [c for c in doc["cases"] if c["tier"] != 2] + tier2
    return driver.dumps(doc)


def main() -> int:
    for ep, tier2 in generate().items():
        text = render(ep, tier2)
        (PARITY / "cases" / f"{ep}.json").write_text(text, encoding="utf-8", newline="\n")
    return 0


if __name__ == "__main__":
    if len(sys.argv) != 1:
        raise SystemExit(__doc__)
    sys.exit(main())
