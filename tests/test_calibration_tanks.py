"""The Type C tank calibration cases (issue #292): run through the tank maths only, held out, banded.

`calibration/tank_cases.yaml` and `tank_bands.yaml` follow the conventions of `cases.yaml` and
`bands.yaml`: every case is held out, cites a listed web source, keeps the source text as published,
and a band miss is a warning, never a failure.
"""

import pytest
import yaml

from vessel_designer import calibration
from vessel_designer.calibration import (CAL, _assess_tanks, _load_tank_bands, _load_tank_cases, report,
                                         _run_tank, _tank_geometry, _tank_warnings_for)

CONFIDENCES = ("high", "medium", "low")
BASES_D = {"steel_outer", "steel_inner", "insulation_outer"}
BASES_L = {"steel_outer_overall", "inner_overall", "insulation_outer_overall"}


def _cases():
    return _load_tank_cases()["type_c_tank"]


def test_every_tank_case_is_held_out_and_states_its_datum_and_source():
    """Label: property
    """
    web = {w["id"] for w in yaml.safe_load((CAL / "sources.yaml").read_text(encoding="utf-8"))["web_sources"]}
    data = _load_tank_cases()
    assert len(_cases()) >= 2
    for c in _cases() + data["not_yet_runnable_tank_cases"]:
        assert str(c["held_out"]).strip(), c["id"]
        sid = c["source"].split(" ")[0]
        assert sid in web, (c["id"], sid)
        assert c["source"].startswith(f"{sid} — ") and c["source"].endswith("accessed 2026-10-16"), c["id"]
        assert c["published"].strip() and c["confidence"] in CONFIDENCES, c["id"]
    for c in _cases():
        assert c["diameter_basis"] in BASES_D and c["length_basis"] in BASES_L, c["id"]
        assert c["run_as_head"] in {"hemisphere", "ellipsoidal_2to1"}, c["id"]
        assert c["observed"]["shell_thickness_mm"] > 0
    for c in data["not_yet_runnable_tank_cases"]:
        assert c["not_yet_runnable"].strip(), c["id"]


def test_the_tank_bands_have_every_figure_for_every_confidence_with_a_basis():
    """Label: property
    """
    bands = _load_tank_bands()
    assert set(bands) == set(calibration._TANK_FIGURES)
    for figure, band in bands.items():
        assert set(band["pct"]) == set(CONFIDENCES), figure
        assert band["basis"].strip() and band["label"].strip(), figure
    # The existing barge bands are not touched.
    assert set(calibration.BANDS["bands"]) == {"lightship_t", "dwt_t"}


def test_the_diameter_and_the_length_are_brought_to_the_outside_of_the_insulation():
    """The conversions follow from the definitions: inner plus two walls is the steel outer diameter, and
    the insulation adds on each side.

    Label: property
    """
    base = {"observed": {"shell_thickness_mm": 50.0}, "insulation_mm": 200.0, "diameter_m": 13.4, "length_m": 40.0}
    steel, outer, length = _tank_geometry({**base, "diameter_basis": "steel_inner", "length_basis": "inner_overall"})
    assert steel == pytest.approx(13.5) and outer == pytest.approx(13.9) and length == pytest.approx(40.5)
    steel, outer, length = _tank_geometry({**base, "diameter_basis": "steel_outer", "length_basis": "steel_outer_overall"})
    assert steel == pytest.approx(13.4) and outer == pytest.approx(13.8) and length == pytest.approx(40.4)
    steel, outer, length = _tank_geometry({**base, "diameter_basis": "insulation_outer",
                                          "length_basis": "insulation_outer_overall"})
    assert steel == pytest.approx(13.0) and outer == pytest.approx(13.4) and length == pytest.approx(40.0)


def test_the_published_river_tank_gives_the_published_wall_by_the_formula():
    """The formula before the allowance and the rounding is 13.08 mm by hand (two passes of the ASME UG-27
    shell formula at 1.58 MPa, 285 MPa, E = 1.0, on the 4.73 m outer diameter); the paper prints 13.1 mm to
    the tenth of a millimetre.

    Label: sourced - Senjanovic et al., DESIGN 2006, river LPG tank: 4730 mm outer diameter, 15.8 bar, S690QL1 285 MPa, cylinder 13.1 mm; 13.08 mm is a hand calculation
    """
    row = _run_tank(next(c for c in _cases() if c["id"] == "senjanovic-2006-river-lpg"))
    assert row.required_mm == pytest.approx(13.08, abs=0.005)
    assert abs(row.required_mm - 13.1) <= 0.05
    # The engine's wall adds the allowance and rounds up to 0.5 mm.
    assert row.wall_mm == 14.0


def test_a_tank_run_uses_the_datum_it_states_and_the_volume_follows_the_geometry():
    """Label: property
    """
    for c in _cases():
        row = _run_tank(c)
        steel, outer, length = _tank_geometry(c)
        assert (row.steel_od_m, row.insulated_od_m, row.insulated_length_m) == (steel, outer, length)
        assert row.volume_m3 > 0
        # More insulation on the same steel diameter and the same steel overall length changes nothing
        # in the volume (the envelope grows, the steel does not).
        thicker = {**c, "insulation_mm": c.get("insulation_mm", 0.0) + 100.0}
        if c["diameter_basis"] == "steel_inner" and c["length_basis"] == "inner_overall":
            assert _run_tank(thicker).volume_m3 == pytest.approx(row.volume_m3, rel=1e-9)


def test_the_case_that_fixes_the_structure_factor_says_its_steel_error_is_in_sample():
    """The structure factor of each case equals the Defaults Profile value for its carriage condition: a Rust
    test (`type_c_defaults.rs`) reads this file and keeps the two equal, as the Python wheel does not hold
    the Rust sources.

    Label: property
    """
    assert _cases()[0]["steel_in_sample"].strip()
    for c in _cases():
        assert c["structure_factor"] >= 1.0, c["id"]


def test_every_tank_case_is_banded_and_reported():
    """Label: property
    """
    checks = _assess_tanks()
    for c in _cases():
        figures = {b.figure for b in checks if b.case == c["id"]}
        want = {"tank_volume_m3", "tank_wall_mm"} | ({"tank_steel_t"} if "tank_steel_t" in c["observed"] else set())
        assert figures == want, c["id"]
    text = report()
    for c in _cases() + _load_tank_cases()["not_yet_runnable_tank_cases"]:
        assert c["id"] in text
    assert "## 5. Type C tank cases" in text


def test_a_tank_band_miss_is_a_warning_annotation_and_not_a_failure():
    """Label: property
    """
    cases = _load_tank_cases()
    cases["type_c_tank"][0]["observed"]["gross_volume_m3"] = 100.0        # a miss on purpose
    checks = _assess_tanks(cases)
    lines = _tank_warnings_for(checks)
    assert any("senjanovic-2006-river-lpg" in x and "outside" in x for x in lines)
    assert all(x.startswith("::warning ") for x in lines)
