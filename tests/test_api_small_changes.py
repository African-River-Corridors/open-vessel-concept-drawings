"""Small behaviour rows of issue #128: error types, loose inputs and NaN, against 0.1.0.

Every expected value and message below was printed by the frozen Python at commit ce96e66.
"""
import math

import pytest

from vessel_designer.core import barge_capacity as B
from vessel_designer.core import cargo, gas_arrangement
from vessel_designer.core import protective_location as PL
from vessel_designer.simulate import simulate

# `max_tank_od_m` is the 12 m tank diameter cap of 0.1.0: it is an input, 16 m by default, since
# issue #175, and the numbers in this file were printed by 0.1.0.
CN_II = dict(vessel="gas_barge", cargo="LPG_propane", loa_m=90, beam_m=14.8, depth_m=6.5, cb=0.85,
             freeboard_min_m=0.3, draught_limit_m=4.4, self_propelled=True, ruleset="IGC_2G",
             fill_ratio=0.95, max_tank_od_m=12.0)
BOX = dict(vessel="box_barge", cargo="bauxite", loa_m=76.5, beam_m=11.4, depth_m=3.5, cb=0.928,
           lightship_k_t_per_m3=0.1093, freeboard_min_m=0.3)


@pytest.mark.parametrize("ruleset,text", [
    (None, "Unknown ruleset None."), (5, "Unknown ruleset 5."), (1.5, "Unknown ruleset 1.5."),
    (True, "Unknown ruleset True."), (["a", 1], "Unknown ruleset ['a', 1]."),
])
def test_simulate_reports_a_ruleset_that_is_not_text_as_unknown(ruleset, text):
    """Label: property"""
    assert simulate(dict(CN_II, ruleset=ruleset)) == {"error": text}          # ce96e66


@pytest.mark.parametrize("text", ["80", "  80 ", "8e1", "٨٠"])
def test_simulate_reads_numbers_written_as_text(text):
    """Label: snapshot"""
    out = simulate(dict(CN_II, loa_m=text))
    assert out["dims"]["loa_m"] == 80.0                                         # ce96e66
    # Re-pinned (issue #277: the wheelhouse sightline blocks the search); was 1986.5486284249137.
    # Re-pinned (issue #410: the covering above D is in the lightship); was 1392.4601578818447.
    assert out["results"]["cargo_t"] == pytest.approx(1124.1630064872984, rel=1e-9, abs=1e-9)


def test_simulate_reads_every_number_as_text():
    """Label: snapshot"""
    out = simulate(dict(CN_II, beam_m="14.8", draught_limit_m="4.4", fill_ratio="0.95"))
    # Re-pinned (issue #237); was 2058.228164213275 at ce96e66. Re-pinned (issue #277: the
    # wheelhouse sightline blocks the search); was 2535.1977193509288. Re-pinned (issue #410: the
    # covering above D is in the lightship); was 1763.374037130099.
    assert out["results"]["cargo_t"] == pytest.approx(1405.8516419707348, rel=1e-9, abs=1e-9)
    out = simulate(dict(BOX, lightship_k_t_per_m3="0.1093"))
    # Re-pinned (#448: the hull's sections give the volume, not the cb input); was 2256.166305 at
    # ce96e66.
    assert out["results"]["cargo_t"] == pytest.approx(2346.1989450000005, rel=1e-9, abs=1e-9)
    out = simulate(dict(BOX, channel=dict(depth_m="5", ukc_m="0.5", bottom_width_m="40")))
    assert out["results"]["draught_max_m"] == pytest.approx(3.2, rel=1e-9, abs=1e-9)           # ce96e66


@pytest.mark.parametrize("text", ["0x10", "", "80,5"])
def test_simulate_refuses_text_that_is_not_a_number_as_python_did(text):
    """Label: property"""
    with pytest.raises(ValueError) as e:
        simulate(dict(CN_II, loa_m=text))
    assert str(e.value) == f"could not convert string to float: {text!r}"       # ce96e66


def test_required_side_protection_wants_the_enum():
    """Label: snapshot"""
    with pytest.raises(ValueError) as e:
        PL.required_side_protection("IGC_2G", 14.8, 1.0)
    assert str(e.value) == "Unknown vessel type: IGC_2G"                         # ce96e66
    with pytest.raises(ValueError) as e:
        PL.required_side_protection(None, 14.8, 1.0)
    assert str(e.value) == "Unknown vessel type: None"                           # ce96e66
    assert PL.required_side_protection(PL.VesselType.IGC_2G, 14.8, 1.0) == 1.0


def test_required_bottom_protection_is_one_formula_for_every_vessel_type():
    # ce96e66: only the ADN_G member gave 1.1; text and every other member gave the general
    # formula. ADN_G was removed (issue #175), so the formula is the same for all.
    """Label: snapshot"""
    assert PL.required_bottom_protection(14.8, 1.0, "ADN_G") == 1.0
    assert PL.required_bottom_protection(14.8, 1.0, PL.VesselType.IGC_2G) == 1.0


@pytest.mark.parametrize("key", [5, None])
def test_get_cargo_with_a_key_that_is_not_text_is_a_value_error(key):
    """Label: property"""
    with pytest.raises(ValueError) as e:
        cargo.get_cargo(key)
    assert str(e.value) == (f"Unknown cargo type '{key}'. Valid types: LNG, LNG_ethane, "
                            "LPG_propane, LPG_butane, LPG_mix, bauxite, crude_oil")   # ce96e66 + crude oil (spec 0006)


def test_get_cargo_with_an_unhashable_key_is_a_type_error():
    """Label: snapshot"""
    with pytest.raises(TypeError, match="unhashable type: 'list'"):                # ce96e66
        cargo.get_cargo([1])


def test_smallest_viable_loa_names_the_keyword_it_does_not_know():
    """Label: property"""
    with pytest.raises(TypeError) as e:
        gas_arrangement.smallest_viable_loa(14.8, "LPG_propane", 4.4, depth=6.5)
    assert str(e.value) == "fit() got an unexpected keyword argument 'depth'"      # ce96e66


def test_capacity_takes_nan_and_infinity_as_0_1_0_did():
    """Label: snapshot"""
    nan, inf = float("nan"), float("inf")
    bulk = dict(beam=16.0, depth=6.0, cargo_type="bauxite")
    # ce96e66 gave a cargo of 0.0 for a NaN length and inf for an infinite one. Re-pinned (#448):
    # the hull is its Sections Table, and a length that is not a finite number writes no sections.
    for loa in (nan, inf):
        with pytest.raises(ValueError) as e:
            B.compute(B.BargeCapacityInputs(loa=loa, **bulk))
        assert str(e.value) == "length, beam and moulded depth must be positive numbers"
    with pytest.raises(ValueError) as e:
        B.compute(B.BargeCapacityInputs(loa=90.0, beam=nan, depth=6.5, cargo_type="LPG_propane"))
    assert str(e.value) == "cannot convert float NaN to integer"                          # ce96e66
    assert not math.isnan(B.compute(B.BargeCapacityInputs(loa=100.0, **bulk)).cargo_capacity_t)


def test_the_core_modules_no_longer_claim_to_run_in_pyodide():
    # Issue #142: these modules call the Rust core now.
    """Label: property"""
    from pathlib import Path
    core = Path(__file__).resolve().parents[1] / "src" / "vessel_designer" / "core"
    names = ("barge_cost", "design_grid", "electric_energy", "gas_separation", "squat", "waves",
             "aft_arrangement", "vessel_standards")
    assert core.is_dir(), "scanning nothing"
    assert [n for n in names if "pyodide" in (core / f"{n}.py").read_text(encoding="utf-8").lower()] == []
