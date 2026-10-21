"""The tank-barge lightship fit (issue #330): the generated file is current, and the Rust constants
equal it.

Label: property
"""
import re
import sys
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
import fit_tank_lightship as fit  # noqa: E402

RUST = (ROOT / "crates/ovcd-core/src/liquid_bulk/lightship.rs").read_text(encoding="utf-8")


def _const(name: str) -> float:
    m = re.search(rf"pub const {name}: f64 = ([0-9.]+);", RUST)
    assert m, name
    return float(m.group(1))


def test_the_generated_file_is_what_the_fit_writes():
    """Label: property"""
    assert fit.OUT.read_text(encoding="utf-8") == fit.text()


def test_the_rust_constants_equal_the_fit():
    """Label: property"""
    got = yaml.safe_load(fit.OUT.read_text(encoding="utf-8"))
    assert _const("TANK_LIGHTSHIP_K_T_PER_M3") == got["tank_lightship_k_t_per_m3"]["value"]
    assert _const("PUMPS_T_PER_M3") == got["outfit"]["pumps_t_per_m3"]["value"]
    assert _const("PIPING_T_PER_M3") == got["outfit"]["piping_t_per_m3"]["value"]
    assert _const("VENTS_GAUGING_T_PER_M3") == got["outfit"]["vents_gauging_t_per_m3"]["value"]
    assert _const("HEATING_T_PER_KW") == got["outfit"]["heating_t_per_kw"]["value"]


def test_the_fit_uses_only_the_declared_subset_and_the_rest_are_held_out():
    """Label: property"""
    inputs = yaml.safe_load((fit.CAL / "tank_lightship_inputs.yaml").read_text(encoding="utf-8"))
    got = yaml.safe_load(fit.OUT.read_text(encoding="utf-8"))
    fitted = {r["case"] for r in got["fit_cases"]}
    held = {r["case"] for r in got["held_out_cases"]}
    assert fitted == set(inputs["fit_subset"])
    assert not fitted & held
    assert got["tank_lightship_k_t_per_m3"]["n"] == len(fitted)
    # Every tank record that can be run is in one of the two groups.
    cases = fit.tank_cases(yaml.safe_load((fit.CAL / "cases.yaml").read_text(encoding="utf-8")))
    cb = got["tank_lightship_k_t_per_m3"]["block_coefficient_used"]
    coeff = {k: v["value"] for k, v in got["outfit"].items()}
    runnable = {c["id"] for c in cases if fit.implied(c, cb, coeff)}
    assert runnable == fitted | held


def test_every_value_of_the_fit_is_marked_an_estimate_with_its_basis():
    """Label: property"""
    got = yaml.safe_load(fit.OUT.read_text(encoding="utf-8"))
    assert got["tank_lightship_k_t_per_m3"]["status"] == "estimate"
    for v in got["outfit"].values():
        assert v["basis"].startswith("Estimate.") and len(v["basis"]) > 40
