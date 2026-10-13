"""The parity case set is complete, fixed and small (spec 0001, issue #9).

- every public function the spec lists has parity cases;
- tier 2 is exactly what the seeded sweep (`parity/sweep.py`) generates, so nobody picks or
  edits sweep points by hand;
- the sweep drops knife-edge points, where a last-digit difference would flip a discrete result;
- a tier-3 case is an error path, or a named degenerate input where Python has no error path;
- the committed parity files stay under about 5 MB.

Risk stopped: a port ticket chooses its own cases, a function reaches the switch untested, or
the oracle grows too big to keep in git. This file is deleted with the parity folder.
"""
import importlib.util
import json
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[1]
PARITY = ROOT / "parity"


def _load(name):
    spec = importlib.util.spec_from_file_location(f"parity_{name}", PARITY / f"{name}.py")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


sweep = _load("sweep")

# The public functions spec 0001 lists, by entry point, as the parity `fn` names.
SPEC_FUNCTIONS = {
    "cargo": ["get_cargo", "cargo_database"],
    "protective_location": ["distance_d_vc", "d_vc_exact", "required_side_protection",
                            "required_bottom_protection", "compute_required_clearances",
                            "check_protective_location"],
    "pressure_vessel": ["get_tank_material", "cylinder_wall_thickness",
                        "hemisphere_head_thickness", "ellipsoidal_head_thickness",
                        "calculate_wall_thicknesses"],
    "hull": ["quantities"],
    "aft_arrangement": ["compute"],
    "gas_separation": ["compute"],
    "tanks": ["calculate_tank_arrangement", "free_surface_moment_cylindrical"],
    "weights": ["calculate_weights"],
    "hydrostatics": ["find_equilibrium_draft", "calculate_stability", "calculate_trim"],
    "barge_capacity": ["compute", "tanks_wide_for_beam"],
    "journey_derating": ["compute"],
    "gas_arrangement": ["fit", "smallest_viable_loa", "required_side_clearance"],
    "gas_stability": ["box_gz", "segment_fill", "assess", "criteria_set_for"],
    "squat": ["compute", "max_speed_for_ukc"],
    "waves": ["compute"],
    "resistance": ["compute"],
    "electric_energy": ["compute"],
    "barge_cost": ["compute", "parse_flat_yaml"],
    "design_grid": ["sweep", "n_tanks_for_length"],
    "ga_adapter": ["to_ga", "is_gas"],
    "vessel_standards": ["get_standard", "standards_for", "validate_catalogue"],
    "simulate": ["simulate", "cargo_options"],
}

# Functions whose input space has nothing to sweep (a fixed list or a lookup by key): tier 1
# already holds every distinct input.
NO_SWEEP = {("cargo", "cargo_database"), ("simulate", "cargo_options"),
            ("gas_stability", "criteria_set_for")}


def _cases(ep):
    return json.loads((PARITY / "cases" / f"{ep}.json").read_text(encoding="utf-8"))["cases"]


def _results(ep):
    doc = json.loads((PARITY / "reference" / f"{ep}.json").read_text(encoding="utf-8"))
    return {r["id"]: r["output"] for r in doc["results"]}


@pytest.mark.parametrize("ep", sorted(SPEC_FUNCTIONS))
def test_every_spec_function_has_named_and_swept_cases(ep):
    cases = _cases(ep)
    for fn in SPEC_FUNCTIONS[ep]:
        tiers = {c["tier"] for c in cases if c["input"]["fn"] == fn}
        assert 1 in tiers, f"{ep}.{fn}: no tier-1 case"
        if (ep, fn) not in NO_SWEEP:
            assert 2 in tiers, f"{ep}.{fn}: no tier-2 sweep case"
    assert {c["tier"] for c in cases} == {1, 2, 3}, f"{ep}: needs tier 1, 2 and 3 cases"


@pytest.fixture(scope="module")
def generated():
    return sweep.generate()


def test_case_files_are_byte_for_byte_what_the_sweep_writes(generated):
    assert set(generated) == set(SPEC_FUNCTIONS)
    for ep, tier2 in generated.items():
        committed = (PARITY / "cases" / f"{ep}.json").read_text(encoding="utf-8")
        assert sweep.render(ep, tier2) == committed, \
            f"{ep}: tier-2 cases differ from the seeded sweep; run `cargo xtask regen`"


def test_sweep_is_seeded_and_says_so(generated):
    assert sweep.generate() == generated
    for ep, cases in generated.items():
        for c in cases:
            assert f"seed {sweep.SEED}" in c["reason"], (ep, c["id"])


def test_knife_edge_points_are_dropped():
    # (24 + 0) / (12 + 0) = 2 exactly: a hair more beam needs a third tank abreast.
    edge = {"fn": "tanks_wide_for_beam",
            "args": {"beam_m": 26.0, "side_clearance_m": 1.0, "tank_gap_transverse_m": 0.0}}
    calm = {"fn": "tanks_wide_for_beam",
            "args": {"beam_m": 20.0, "side_clearance_m": 1.0, "tank_gap_transverse_m": 0.0}}
    assert sweep.is_knife_edge("barge_capacity", edge)
    assert not sweep.is_knife_edge("barge_capacity", calm)


def test_a_continuous_jump_is_a_knife_edge_too():
    # Wall thickness rounds up to 0.5 mm: a float output that jumps, with no discrete output.
    assert sweep.jumps(1.0, 1.5)
    assert not sweep.jumps(1.0, 1.0 + 1e-12)


@pytest.mark.parametrize("ep", sorted(SPEC_FUNCTIONS))
def test_tier_3_is_an_error_path_or_a_named_degenerate_input(ep):
    results = _results(ep)
    for c in (c for c in _cases(ep) if c["tier"] == 3):
        out = results[c["id"]]
        is_error = "error" in out or (isinstance(out.get("ok"), dict) and "error" in out["ok"])
        if c["reason"].startswith("tier 3: degenerate input"):
            assert not is_error, f"{ep} {c['id']}: errors, so call it an error path"
        else:
            assert is_error, f"{ep} {c['id']}: tier 3 error case did not error"


def test_committed_parity_files_stay_under_5_mb():
    total = sum(p.stat().st_size for d in ("cases", "reference")
                for p in (PARITY / d).glob("*.json"))
    assert total < 5_000_000, f"parity files are {total / 1e6:.2f} MB"
